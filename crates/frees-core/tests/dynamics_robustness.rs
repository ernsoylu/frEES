//! Adversarial robustness for the Phase 7–8 surface: `DYNAMIC` integration,
//! events, and the analysis/design drivers.
//!
//! The rule is [`tests/robustness.rs`](robustness.rs)'s, restated for a surface
//! that can *run forever* rather than merely crash: **every entry point answers
//! with a `Result` in bounded time.** Not a panic, not an abort, not a hang,
//! and not a plausible-looking wrong answer.
//!
//! Integrators are the classic hang risk, so the interesting cases are the ones
//! where a bound exists only because something enforces it. Three of these
//! tests are regressions for defects this audit found and fixed:
//!
//! * [`an_absurd_points_count_is_refused_before_anything_is_allocated`] —
//!   `points = 1e9` reached `vec![0.0; count]` in `ode::integrator::integrate`
//!   with no ceiling. Measured: the process **aborted** with
//!   `memory allocation of 8000000000 bytes failed` before taking a single
//!   step. `panic = "abort"` is the wasm profile, so in the browser that kills
//!   the worker outright — nothing downstream can turn it into a diagnostic.
//!   (`ode/problem.rs::MAX_OUTPUT_SAMPLES`, `ode/integrator.rs::integrate`)
//! * [`a_set_event_that_re_arms_its_own_crossing_is_cut_off`] — a `set` action
//!   whose assigned value sits on the wrong side of its own switching function
//!   turns the time loop into a restart loop. It *was* bounded, by
//!   [`MAX_STEPS`] — but each pass spends ~60 right-hand-side evaluations
//!   bisecting to the crossing, so the bound arrived tens of minutes later.
//!   Measured before: killed at a 45 s CPU limit, twice. After: a diagnostic in
//!   1.1 s. (`ode/integrator.rs::MAX_CONSECUTIVE_SET_RESTARTS`)
//! * [`a_stiff_problem_on_an_explicit_method_terminates`] — not a fix, a
//!   *measurement* that pins the remaining exposure. This one really does need
//!   the full [`MAX_STEPS`] budget, and the port has no clock to cut it short
//!   (`ode/problem.rs`, *No clock*). At document level that was 182 s. The
//!   library-level test below drives the same ceiling with a cheap closure so
//!   the assertion costs milliseconds instead.
//!
//! The rest are the standing corpus: degenerate spans, degenerate tolerances,
//! degenerate sample counts, malformed state declarations, events that never
//! fire and events that fire constantly, non-finite initial conditions, and the
//! four analysis drivers at their degenerate inputs.

use std::collections::BTreeMap;

use frees_core::analysis::uncertainty::UncertaintySpec;
use frees_core::analysis::{curvefit, montecarlo, optimizer, pareto};
use frees_core::ode::integrator::{integrate, MAX_CONSECUTIVE_SET_RESTARTS, MAX_STEPS};
use frees_core::ode::problem::{
    OdeEvent, OdeProblem, OdeRhs, DEFAULT_SAMPLE_COUNT, MAX_OUTPUT_SAMPLES,
};
use frees_core::{parse_document, solve, FreesError, SolverSettings};

// ── helpers ─────────────────────────────────────────────────────────────────

fn settings() -> SolverSettings {
    SolverSettings::default()
}

/// Solve `src` and return its single ODE table's rows, failing the test with
/// the diagnostic if it did not solve.
fn table_rows(src: &str) -> Vec<Vec<f64>> {
    let solution = solve(src, &settings()).unwrap_or_else(|e| panic!("{src}\n=> {}", e.error));
    let table = solution
        .ode_tables
        .first()
        .unwrap_or_else(|| panic!("{src}\n=> solved but published no ODE table"));
    table.rows.clone()
}

/// Solve `src`, require a refusal, and return the message.
///
/// A refusal is the *point* of most of these documents; a success would mean
/// the engine quietly answered a question that has no answer.
fn refused(src: &str) -> String {
    match solve(src, &settings()) {
        Ok(s) => panic!(
            "{src}\n=> expected a refusal, got {} table(s) and values {:?}",
            s.ode_tables.len(),
            s.values
        ),
        Err(failure) => failure.error.to_string_message(),
    }
}

/// A one-state problem `der(y) = f(t, y)` with no events, ready to mutate.
fn problem<'a>(rhs: &'a dyn OdeRhs) -> OdeProblem<'a> {
    OdeProblem {
        method: "ode45".into(),
        t0: 0.0,
        tf: 1.0,
        y0: vec![1.0],
        rhs,
        points: None,
        fixed_step: None,
        rtol: 1e-6,
        atol: 1e-9,
        max_step: None,
        events: Vec::new(),
    }
}

fn integrate_err(p: &OdeProblem<'_>) -> String {
    match integrate(p) {
        Ok(r) => panic!(
            "expected a refusal, got {} sample(s) ending at {}",
            r.times.len(),
            r.end_time
        ),
        Err(e) => e.to_string_message(),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 1. The output-sample ceiling — the allocation that aborted the process
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn an_absurd_points_count_is_refused_before_anything_is_allocated() {
    // `vec![0.0; 1_000_000_000]` is 8 GB. The measured failure was
    // `memory allocation of 8000000000 bytes failed` — an abort, not a panic,
    // so `catch_unwind` cannot even observe it and the wasm worker dies.
    let message = refused(
        "k = 0.05\n\
         Tinf = 20\n\
         \n\
         DYNAMIC cooling (method = ode45, time = 0 .. 60, points = 1e9)\n  \
           der(Temp) = -k*(Temp - Tinf)\n  \
           Temp(0) = 95\n\
         END\n",
    );
    assert!(
        message.contains("would materialise more than 100000 output rows"),
        "{message}"
    );
}

#[test]
fn the_output_sample_ceiling_lands_on_its_documented_boundary() {
    let doc = |points: usize| {
        format!(
            "DYNAMIC d (method = ode45, time = 0 .. 1, points = {points})\n  \
               der(Temp) = -Temp\n  \
               Temp(0) = 1\n\
             END\n"
        )
    };
    // The last accepted count and the first refused one, either side of the
    // constant. `MAX_OUTPUT_SAMPLES` rows really are materialised at the
    // boundary — this is not a parse-time screen.
    assert_eq!(
        table_rows(&doc(MAX_OUTPUT_SAMPLES)).len(),
        MAX_OUTPUT_SAMPLES
    );
    let message = refused(&doc(MAX_OUTPUT_SAMPLES + 1));
    assert!(
        message.contains(&format!("points = {}", MAX_OUTPUT_SAMPLES + 1)),
        "{message}"
    );
}

#[test]
fn the_ceiling_is_enforced_at_the_library_boundary_too() {
    // The document path is not the only caller: `analysis` drives `integrate`
    // directly. The guard therefore lives at the allocation site.
    let rhs = |_t: f64, _y: &[f64]| Ok(vec![0.0]);
    let mut p = problem(&rhs);
    p.points = Some(MAX_OUTPUT_SAMPLES + 1);
    assert!(integrate_err(&p).contains("Use fewer points"));

    // …and it refuses *before* integrating, so the cost of the refusal does not
    // depend on the span. A ten-billion-second span still answers instantly.
    p.tf = 1e10;
    assert!(integrate_err(&p).contains("Use fewer points"));
}

#[test]
fn the_corpus_sample_counts_are_far_below_the_ceiling() {
    // The ceiling was chosen with the real corpus in hand, so it is the real
    // corpus that is asserted against — a document that creeps toward
    // `MAX_OUTPUT_SAMPLES` fails here rather than in the field. Measured when
    // the ceiling was set: the largest `points` across all 390 documents is
    // 1 201, in `ev-thermal-management`.
    let mut worst = (0usize, String::new());
    for dir in [
        "../../fixtures/corpus",
        "../../fixtures/corpus-pending/corpus",
    ] {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(dir);
        for entry in std::fs::read_dir(&dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display())) {
            let path = entry.expect("dir entry").path();
            if path.extension().and_then(|x| x.to_str()) != Some("frees") {
                continue;
            }
            let text = std::fs::read_to_string(&path).expect("read fixture");
            for points in declared_points(&text) {
                if points > worst.0 {
                    worst = (points, path.display().to_string());
                }
            }
        }
    }
    assert!(worst.0 > 0, "no `points = …` found — the scan is broken");
    assert!(
        worst.0 * 10 <= MAX_OUTPUT_SAMPLES,
        "{} declares points = {}, within 10x of the {MAX_OUTPUT_SAMPLES} ceiling",
        worst.1,
        worst.0
    );
}

/// Every `points = <integer>` in a document, as the option parser reads it.
/// Deliberately crude — it only has to find the corpus's own spellings.
fn declared_points(text: &str) -> Vec<usize> {
    text.match_indices("points")
        .filter_map(|(at, _)| {
            let rest = text[at + "points".len()..].trim_start();
            let rest = rest.strip_prefix('=')?.trim_start();
            let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
            digits.parse().ok()
        })
        .collect()
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. Degenerate sample counts and tolerances — bounded, and *correct*
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn points_below_two_fall_back_to_the_default_sample_count() {
    // `points = 0` and `points = 1` are not errors: `OdeProblem::sample_count`
    // floors at 2 and otherwise uses the Java default. A silently *empty* table
    // would be the wrong answer, so the row count is asserted, not just
    // "it solved".
    for points in ["0", "1"] {
        let rows = table_rows(&format!(
            "DYNAMIC d (method = ode45, time = 0 .. 1, points = {points})\n  \
               der(Temp) = -Temp\n  \
               Temp(0) = 1\n\
             END\n"
        ));
        assert_eq!(rows.len(), DEFAULT_SAMPLE_COUNT, "points = {points}");
        assert_eq!(rows[0], vec![0.0, 1.0], "points = {points}");
        // e^-1, to the tolerance the default settings earn.
        assert!(
            (rows[rows.len() - 1][1] - std::f64::consts::E.recip()).abs() < 1e-6,
            "points = {points}: {:?}",
            rows[rows.len() - 1]
        );
    }
}

#[test]
fn a_negative_points_count_is_clamped_rather_than_wrapped() {
    // The parser's `(int) Math.round(...)` transcription maps a negative count
    // to 0 explicitly. Casting `-5.0` straight to `usize` would saturate at 0 in
    // Rust but wrap in a C-like cast, so the clamp is worth pinning.
    let rows = table_rows(
        "DYNAMIC d (method = ode45, time = 0 .. 1, points = -5)\n  \
           der(Temp) = -Temp\n  \
           Temp(0) = 1\n\
         END\n",
    );
    assert_eq!(rows.len(), DEFAULT_SAMPLE_COUNT);
}

#[test]
fn degenerate_tolerances_still_produce_a_correct_trajectory() {
    // `rtol = 0` makes the error test rely entirely on `atol`; `rtol = 1e300`
    // makes every step pass on the first try. Neither may hang, and — the part
    // that matters — neither may silently return a *wrong* trajectory, so both
    // are checked against e^-1 rather than merely for having solved.
    for tol in ["rtol = 0", "rtol = 1e300", "atol = 0"] {
        let rows = table_rows(&format!(
            "DYNAMIC d (method = ode45, time = 0 .. 1, points = 5, {tol})\n  \
               der(Temp) = -Temp\n  \
               Temp(0) = 1\n\
             END\n"
        ));
        assert_eq!(rows.len(), 5, "{tol}");
        let final_value = rows[4][1];
        assert!(
            (final_value - std::f64::consts::E.recip()).abs() < 1e-3,
            "{tol}: e^-1 came out as {final_value}"
        );
    }
}

#[test]
fn both_tolerances_zero_is_refused_rather_than_answered_in_nan() {
    // `scale = atol + rtol*|y|` is then exactly 0, and the scaled norm is
    // `0/0`. There is no meaningful answer; the requirement is that the engine
    // says so instead of publishing NaN cells.
    let message = refused(
        "DYNAMIC d (method = ode45, time = 0 .. 1, points = 5, rtol = 0, atol = 0)\n  \
           der(Temp) = -Temp\n  \
           Temp(0) = 1\n\
         END\n",
    );
    assert!(!message.is_empty(), "{message}");
}

#[test]
fn a_negative_tolerance_does_not_spin_and_does_not_lie() {
    // Negative tolerances make `scale` negative, so `r*r` in the scaled norm is
    // computed against a nonsense yardstick and no step is ever rejected. The
    // requirement is not that this be an error — the measured behaviour is that
    // the initial-step estimate and `MAX_SCALE` between them keep the run
    // accurate anyway — but that it terminate and that whatever comes back be
    // the *right* trajectory rather than a plausible-looking wrong one.
    let rows = table_rows(
        "DYNAMIC d (method = ode45, time = 0 .. 1, points = 5, rtol = -1, atol = -1)\n  \
           der(Temp) = -Temp\n  \
           Temp(0) = 1\n\
         END\n",
    );
    assert_eq!(rows.len(), 5);
    assert!(
        (rows[4][1] - std::f64::consts::E.recip()).abs() < 1e-6,
        "{:?}",
        rows[4]
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. Degenerate time spans
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn a_zero_or_reversed_time_span_is_refused_with_both_endpoints_quoted() {
    for (span, quoted) in [("0 .. 0", "0.0 .. 0.0"), ("10 .. 0", "10.0 .. 0.0")] {
        let message = refused(&format!(
            "DYNAMIC d (method = ode45, time = {span}, points = 5)\n  \
               der(Temp) = -Temp\n  \
               Temp(0) = 1\n\
             END\n"
        ));
        assert!(
            message.contains("must satisfy t0 < tf"),
            "{span}: {message}"
        );
        // The user's own numbers come back, in the Java `Double.toString`
        // spelling the golden fixtures compare verbatim.
        assert!(message.contains(quoted), "{span}: {message}");
    }
}

#[test]
fn a_non_finite_time_span_is_refused_rather_than_producing_nan_rows() {
    // The defect this pins is a *silent wrong answer*, the worst of the three
    // failure modes. `tf = inf` sails past `tf <= t0`; `tf = NaN` sails past it
    // too, because every comparison against NaN is false. `span` and `min_step`
    // then go non-finite, the loop condition `t < tf - min_step` is false on the
    // first pass so nothing is integrated, and `integrate` publishes a
    // full-height table anyway. Measured before the fix: `tf = inf` returned
    // 200 rows of `[NaN, inf, inf, …]`, presented as a trajectory.
    let rhs = |_t: f64, _y: &[f64]| Ok(vec![0.0]);
    for (t0, tf) in [
        (0.0, f64::INFINITY),
        (0.0, f64::NAN),
        (f64::NEG_INFINITY, 1.0),
        (f64::NAN, 1.0),
    ] {
        let mut p = problem(&rhs);
        p.t0 = t0;
        p.tf = tf;
        match integrate(&p) {
            Err(e) => assert!(
                e.to_string_message().contains("time span must be finite"),
                "({t0}, {tf}): {e}"
            ),
            Ok(r) => panic!(
                "({t0}, {tf}) produced {} rows: {:?}",
                r.times.len(),
                &r.times[..r.times.len().min(4)]
            ),
        }
    }
    // The ordinary finite rejections keep their own, different sentence.
    let mut p = problem(&rhs);
    p.tf = 0.0;
    assert!(integrate_err(&p).contains("must satisfy t0 < tf"));
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Malformed state declarations
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn a_state_with_no_initial_condition_names_the_state() {
    let message = refused(
        "DYNAMIC d (method = ode45, time = 0 .. 1, points = 5)\n  \
           der(Temp) = -Temp\n\
         END\n",
    );
    assert!(
        message.contains("state 'temp' has no initial condition"),
        "{message}"
    );
}

#[test]
fn two_der_equations_for_one_state_are_refused() {
    let message = refused(
        "DYNAMIC d (method = ode45, time = 0 .. 1, points = 5)\n  \
           der(Temp) = -Temp\n  \
           der(Temp) = 1\n  \
           Temp(0) = 1\n\
         END\n",
    );
    assert!(
        message.contains("more than one explicit der() equation"),
        "{message}"
    );
}

#[test]
fn a_block_with_no_state_at_all_is_refused() {
    // `der()` over a non-variable leaves the block stateless. An empty state
    // vector would divide by zero in `scaled_norm` and integrate nothing.
    for body in ["der(3) = 1", "Temp(0) = 1", "x = 1"] {
        let message = refused(&format!(
            "DYNAMIC d (method = ode45, time = 0 .. 1, points = 5)\n  {body}\nEND\n"
        ));
        assert!(
            message.contains("no der(X) equation found") || message.contains("not a state"),
            "{body}: {message}"
        );
    }
}

#[test]
fn an_initial_condition_for_something_that_is_not_a_state_is_refused() {
    let message = refused(
        "DYNAMIC d (method = ode45, time = 0 .. 1, points = 5)\n  \
           der(Temp) = -Temp\n  \
           Temp(0) = 1\n  \
           Other(0) = 5\n\
         END\n",
    );
    assert!(message.contains("not a state"), "{message}");
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. Non-finite initial conditions and derivatives
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn a_non_finite_initial_condition_is_refused_at_the_document_level() {
    // Three layers close every route, and which one answers depends on how the
    // value is spelled: the evaluator refuses an outright domain error, the
    // integrator's initial-state check catches an overflow that is arithmetic
    // rather than a domain error, and (for a document with analytic equations
    // as well) the Newton solver refuses the non-finite residual first.
    for (ic, needle) in [
        ("sqrt(-1)", "square root of a negative number"),
        ("0/0", "division by zero"),
        ("ln(-1)", "logarithm of a negative number"),
        ("1e400 - 1e400", "non-finite initial state"),
        ("1e308*1e308", "non-finite initial state"),
    ] {
        let message = refused(&format!(
            "DYNAMIC d (method = ode45, time = 0 .. 1, points = 5)\n  \
               der(Temp) = -Temp\n  \
               Temp(0) = {ic}\n\
             END\n"
        ));
        assert!(message.contains(needle), "{ic}: {message}");
    }
}

#[test]
fn a_non_finite_state_reached_mid_flight_is_refused_not_published() {
    // `y' = y²`, `y(0) = 1` escapes to infinity at exactly t = 1, well inside
    // the span. The alternative to refusing is a published table with NaN or
    // Inf cells, which the frontend plots as a gap and the ODE accessors
    // average into garbage.
    let message = refused(
        "DYNAMIC d (method = ode45, time = 0 .. 5, points = 5)\n  \
           der(Y) = Y^2\n  \
           Y(0) = 1\n\
         END\n",
    );
    assert!(
        message.contains("non-finite") || message.contains("step size underflow"),
        "{message}"
    );
    // …and the blow-up is located, not merely reported: it happened at t ≈ 1.
    assert!(message.contains("t = 1.0"), "{message}");
}

#[test]
fn a_nan_initial_state_never_reaches_the_output_table() {
    // At the library boundary the document guards are gone, so this is the last
    // line of defence. The RHS here is constant and finite, so only a check on
    // the *state* can catch it.
    // Measured before the `check_finite(&y, …)` this pins: the NaN poisoned
    // `scale`, so every error test and every `h_use < min_step` comparison came
    // out false, nothing was ever rejected, and the run burned all 10^6 steps
    // before blaming stiffness.
    let rhs = |_t: f64, _y: &[f64]| Ok(vec![1.0]);
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut p = problem(&rhs);
        p.y0 = vec![bad];
        match integrate(&p) {
            Err(e) => {
                let message = e.to_string_message();
                assert!(message.contains("non-finite initial state"), "{bad}: {e}");
                assert!(
                    !message.contains("integration steps"),
                    "{bad}: diagnosed as stiffness after spending the whole budget: {message}"
                );
            }
            Ok(r) => panic!(
                "y0 = {bad} published {} rows, first state {:?}",
                r.times.len(),
                r.states.first()
            ),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. Events: never, always, and self-perpetuating
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn an_event_that_never_crosses_leaves_the_run_untouched() {
    let solution = solve(
        "DYNAMIC d (method = ode45, time = 0 .. 10, points = 11)\n  \
           der(Temp) = -Temp\n  \
           Temp(0) = 1\n  \
           EVENT never: Temp = -5 | falling -> stop\n\
         END\n",
        &settings(),
    )
    .unwrap_or_else(|e| panic!("{}", e.error));
    let table = &solution.ode_tables[0];
    assert!(table.events.is_empty(), "{:?}", table.events);
    assert!(!table.stopped);
    assert_eq!(table.end_time, 10.0);
    assert_eq!(table.rows.len(), 11);
}

#[test]
fn an_event_that_fires_on_almost_every_step_still_terminates() {
    // `sin(1000 t) = 0` crosses ~16 000 times over the span; the integrator only
    // sees the crossings its steps bracket. The run must finish, the recorded
    // hits must be in time order, and none may be duplicated.
    let solution = solve(
        "DYNAMIC d (method = ode45, time = 0 .. 50, points = 11)\n  \
           der(Y) = cos(time)*100\n  \
           Y(0) = 0\n  \
           EVENT tick: sin(time*1000) = 0 | rising -> record\n\
         END\n",
        &settings(),
    )
    .unwrap_or_else(|e| panic!("{}", e.error));
    let table = &solution.ode_tables[0];
    assert!(!table.events.is_empty(), "the event never fired at all");
    assert!(!table.stopped);
    assert_eq!(table.end_time, 50.0);
    let times: Vec<f64> = table.events.iter().map(|e| e.time).collect();
    assert!(
        times.windows(2).all(|w| w[0] < w[1]),
        "hits out of order: {times:?}"
    );
}

#[test]
fn a_legitimate_sawtooth_set_event_is_not_caught_by_the_restart_guard() {
    // The guard added below must not touch this: `dyn_event_set`'s shape, a ramp
    // reset ten times, with many ordinary steps between resets.
    let solution = solve(
        "DYNAMIC latch (method = ode45, time = 0 .. 10, points = 11)\n  \
           der(Level) = 1\n  \
           Level(0) = 0\n  \
           EVENT trip: Level = 4 | rising -> set Level = 0\n\
         END\n",
        &settings(),
    )
    .unwrap_or_else(|e| panic!("{}", e.error));
    let table = &solution.ode_tables[0];
    assert_eq!(table.events.len(), 2, "{:?}", table.events);
    assert!(!table.stopped);
    assert_eq!(table.end_time, 10.0);
}

#[test]
fn a_set_event_that_re_arms_its_own_crossing_is_cut_off() {
    // The defect: `set L = 4.0000000001` on a `L = 4 | falling` event leaves the
    // state above the threshold, so the very next step crosses again. Each pass
    // costs a 60-iteration bisection and advances `t` by 1e-10. `MAX_STEPS`
    // bounds it at 10^6 passes — tens of minutes. Measured before the fix: two
    // documents of this shape were both killed at a 45 s CPU limit. After: 1.1 s.
    for (der, start, threshold, reset, direction) in [
        ("-1", "5", "4", "4.0000000001", "falling"),
        ("1", "0", "1", "0.9999999999", "rising"),
    ] {
        let message = refused(&format!(
            "DYNAMIC d (method = ode45, time = 0 .. 10, points = 5)\n  \
               der(L) = {der}\n  \
               L(0) = {start}\n  \
               EVENT r: L = {threshold} | {direction} -> set L = {reset}\n\
             END\n"
        ));
        assert!(
            message.contains("re-arms its own crossing"),
            "{der}/{direction}: {message}"
        );
        assert!(
            message.contains(&MAX_CONSECUTIVE_SET_RESTARTS.to_string()),
            "the message should quote the budget it spent: {message}"
        );
        // The diagnostic has to say *where*, or it is unactionable.
        assert!(message.contains("t = 10.0"), "{der}: {message}");
    }
}

#[test]
fn the_restart_guard_judges_progress_not_the_number_of_firings() {
    // The regression for the guard's own first cut, which refused this. Once
    // the adaptive step grows past the 0.1-wide switching period, *every* step
    // brackets a crossing and there are no ordinary accepted steps left to
    // reset a naive counter — yet the model is fine and time is advancing at
    // full rate. Only the projected-completion test tells the two apart.
    let solution = solve(
        "DYNAMIC saw (method = ode45, time = 0 .. 500, points = 21)\n  \
           der(Level) = 1\n  \
           Level(0) = 0\n  \
           EVENT trip: Level = 0.1 | rising -> set Level = 0\n\
         END\n",
        &settings(),
    )
    .unwrap_or_else(|e| panic!("{}", e.error));
    let fired = solution.ode_tables[0].events.len();
    assert!(
        fired > MAX_CONSECUTIVE_SET_RESTARTS,
        "only {fired} firings — this document no longer exercises the property"
    );
}

#[test]
fn a_set_target_outside_the_state_vector_is_a_diagnostic_not_an_index_panic() {
    // `DynamicSolver` validates the target, so this is unreachable from a
    // document — but the library boundary is public and the wasm profile is
    // `panic = "abort"`, which would take the worker with it.
    let rhs = |_t: f64, _y: &[f64]| Ok(vec![1.0]);
    let mut p = problem(&rhs);
    p.tf = 10.0;
    p.events = vec![OdeEvent::with_set(
        "bad",
        Box::new(|_t: f64, y: &[f64]| Ok(y[0] - 2.0)),
        1,
        false,
        7, // there is exactly one state
        Box::new(|_t: f64, _y: &[f64]| Ok(0.0)),
    )];
    let message = integrate_err(&p);
    assert!(message.contains("outside the 1-state vector"), "{message}");
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. The step budget — the bound that is only a bound
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn a_stiff_problem_on_an_explicit_method_terminates() {
    // Two stiff shapes, two different exits. Which one you get depends on
    // whether the required step falls below `span * 1e-12` (underflow) or merely
    // below `span / MAX_STEPS` (budget). Both are `Err`; neither may run on.
    for (rhs, span) in [
        ("-1e10*Y + 1e10", "0 .. 100"),
        ("-1e12*(Y - cos(time))", "0 .. 1000"),
    ] {
        let message = refused(&format!(
            "DYNAMIC d (method = ode23, time = {span}, points = 11)\n  \
               der(Y) = {rhs}\n  \
               Y(0) = 0\n\
             END\n"
        ));
        assert!(
            message.contains("step size underflow") || message.contains("integration steps"),
            "{rhs}: {message}"
        );
    }
}

#[test]
fn the_step_budget_is_the_only_thing_bounding_a_slowly_diverging_run() {
    // The honest measurement behind `MAX_CONSECUTIVE_SET_RESTARTS`' doc comment.
    // A cheap constant RHS with a step capped far below the span burns the whole
    // budget and stops — proving the ceiling is reached, and that reaching it is
    // an `Err` rather than a truncated table presented as an answer.
    //
    // Cheap on purpose: at document level this same ceiling took 182 s, because
    // every RHS evaluation there is a full algebraic inner solve.
    let rhs = |_t: f64, _y: &[f64]| Ok(vec![1.0]);
    let mut p = problem(&rhs);
    p.tf = 1.0;
    p.fixed_step = Some(1.0 / (MAX_STEPS as f64 * 2.0));
    p.method = "heun".into(); // a fixed-step method honours `fixed_step` exactly
    let message = integrate_err(&p);
    assert!(
        message.contains(&format!("exceeded {MAX_STEPS} integration steps")),
        "{message}"
    );
}

#[test]
fn a_right_hand_side_that_fails_propagates_rather_than_being_swallowed() {
    // The RHS closure is fallible because a `DYNAMIC` block's RHS runs a full
    // algebraic solve. A failure there must surface, not be treated as a zero
    // derivative.
    let rhs = |t: f64, _y: &[f64]| {
        if t > 0.5 {
            Err(FreesError::solver("inner block did not converge"))
        } else {
            Ok(vec![1.0])
        }
    };
    let p = problem(&rhs);
    assert!(integrate_err(&p).contains("inner block did not converge"));
}

// ═══════════════════════════════════════════════════════════════════════════
// 8. Analysis and design drivers at their degenerate inputs
// ═══════════════════════════════════════════════════════════════════════════

fn opt_problem(
    text: &str,
    objective: &str,
    decisions: &[&str],
    lo: &[f64],
    hi: &[f64],
) -> optimizer::Problem {
    optimizer::Problem {
        text: text.to_string(),
        settings: settings(),
        overrides: Vec::new(),
        objective: objective.to_string(),
        decisions: decisions.iter().map(|s| (*s).to_string()).collect(),
        lowers: lo.to_vec(),
        uppers: hi.to_vec(),
        method: None,
        maximize: false,
        constraints: Vec::new(),
        extra_tables: Vec::new(),
    }
}

#[test]
fn an_optimizer_on_a_perfectly_flat_objective_returns_a_feasible_point() {
    // Every probe scores identically, so no search direction exists. Brent and
    // Nelder–Mead must both stop at their iteration budget and return a point
    // inside the box with the right objective value — not NaN, not a bound
    // violation, not an error.
    for method in [None, Some("brent"), Some("bobyqa"), Some("nelder-mead")] {
        let mut p = opt_problem("f = 7 + 0*x\n", "f", &["x"], &[0.0], &[10.0]);
        p.method = method.map(str::to_string);
        let result = optimizer::optimize(&p)
            .unwrap_or_else(|e| panic!("{method:?} on a flat objective: {e}"));
        assert_eq!(result.objective_value, 7.0, "{method:?}");
        assert_eq!(result.decision_values.len(), 1, "{method:?}");
        let x = result.decision_values[0];
        assert!(
            (0.0..=10.0).contains(&x),
            "{method:?}: x = {x} left the box"
        );
        assert!(result.evaluations > 0, "{method:?}");
    }
}

#[test]
fn an_optimizer_with_a_degenerate_box_is_refused_or_pinned() {
    // Crossed bounds are a validation error; a zero-width box is a single point.
    let mut crossed = opt_problem("f = x^2\n", "f", &["x"], &[5.0], &[1.0]);
    crossed.method = Some("brent".into());
    assert!(optimizer::optimize(&crossed).is_err());

    let mut pinned = opt_problem("f = x^2\n", "f", &["x"], &[3.0], &[3.0]);
    pinned.method = Some("brent".into());
    match optimizer::optimize(&pinned) {
        Ok(r) => assert_eq!(r.decision_values, vec![3.0]),
        Err(e) => assert!(!e.to_string_message().is_empty()),
    }

    // Non-finite bounds are the Java `validate`'s own rejection.
    for (lo, hi) in [(f64::NAN, 1.0), (0.0, f64::INFINITY)] {
        let mut p = opt_problem("f = x^2\n", "f", &["x"], &[lo], &[hi]);
        p.method = Some("brent".into());
        assert!(
            optimizer::optimize(&p).is_err(),
            "({lo}, {hi}) was accepted"
        );
    }
}

#[test]
fn an_objective_that_never_solves_is_an_error_not_a_penalty_value() {
    // Every probe fails, so the search sees only `PENALTY`. The *final* solve at
    // the returned point fails too, and that failure is what must surface —
    // returning 1e30 as an objective would be a silent wrong answer.
    let mut p = opt_problem("f = ln(x - 100)\n", "f", &["x"], &[0.0], &[1.0]);
    p.method = Some("brent".into());
    let result = optimizer::optimize(&p);
    match result {
        Err(e) => assert!(!e.to_string_message().is_empty()),
        Ok(r) => assert!(
            r.objective_value.is_finite() && r.objective_value < 1e29,
            "a penalty value escaped as the answer: {}",
            r.objective_value
        ),
    }
}

fn pareto_problem(population_size: usize, generations: usize) -> pareto::Problem {
    pareto::Problem {
        text: "f1 = x^2\nf2 = (x - 2)^2\n".to_string(),
        settings: settings(),
        overrides: Vec::new(),
        objectives: vec!["f1".into(), "f2".into()],
        maximize: vec![false, false],
        decisions: vec!["x".into()],
        lowers: vec![-5.0],
        uppers: vec![5.0],
        population_size,
        generations,
        seed: 42,
        constraints: Vec::new(),
        extra_tables: Vec::new(),
    }
}

#[test]
fn nsga_ii_with_a_population_of_one_is_floored_not_degenerate() {
    // `pop_size = max(8, requested)`. A population of 1 would make tournament
    // selection, crowding distance and the non-dominated sort all degenerate
    // (and crowding divides by `max - min` across the front).
    for requested in [0, 1, 2] {
        let result = pareto::optimize_multi(&pareto_problem(requested, 3))
            .unwrap_or_else(|e| panic!("population {requested}: {e}"));
        assert!(
            !result.front.is_empty(),
            "population {requested} produced an empty front"
        );
        for point in &result.front {
            assert_eq!(point.decisions.len(), 1);
            assert_eq!(point.objectives.len(), 2);
            assert!(
                point.objectives.iter().all(|o| o.is_finite()),
                "population {requested}: {:?}",
                point.objectives
            );
            assert!((-5.0..=5.0).contains(&point.decisions[0]));
        }
        assert!(result.evaluations >= 8, "population {requested}");
    }
}

#[test]
fn nsga_ii_with_zero_generations_returns_the_initial_front() {
    let result = pareto::optimize_multi(&pareto_problem(8, 0)).expect("zero generations");
    assert!(!result.front.is_empty());
    assert_eq!(result.evaluations, 8);
}

#[test]
fn levenberg_marquardt_on_a_singular_jacobian_is_an_error_not_a_hang() {
    // `a + b` is unidentifiable: the two columns of the Jacobian are identical,
    // so `J^T J` is exactly singular. Commons Math raises there, and this port
    // must too — silently returning one of the infinitely many optima would be
    // the wrong answer, and iterating forever would be worse.
    let x: Vec<f64> = (0..10).map(f64::from).collect();
    let y: Vec<f64> = x.iter().map(|_| 5.0).collect();
    let result = curvefit::fit(
        "y = a + b",
        "y",
        "x",
        &["a".into(), "b".into()],
        &x,
        &y,
        None,
    );
    match result {
        Err(e) => assert!(!e.to_string_message().is_empty()),
        Ok(fit) => {
            // If it does converge, the answer must at least be finite and fit.
            assert!(
                fit.fitted_parameters.iter().all(|p| p.is_finite()),
                "{:?}",
                fit.fitted_parameters
            );
            assert!(
                (fit.fitted_parameters.iter().sum::<f64>() - 5.0).abs() < 1e-6,
                "{:?}",
                fit.fitted_parameters
            );
        }
    }
}

#[test]
fn levenberg_marquardt_rejects_degenerate_data_rather_than_indexing_off_the_end() {
    let ok_x = vec![0.0, 1.0, 2.0];
    for (model, x, y, params) in [
        ("y = a*x", vec![], vec![], vec!["a".to_string()]),
        ("y = a*x", ok_x.clone(), vec![1.0], vec!["a".to_string()]),
        ("y = a*x", ok_x.clone(), vec![1.0, 2.0, 3.0], vec![]),
        ("", ok_x.clone(), vec![1.0, 2.0, 3.0], vec!["a".to_string()]),
    ] {
        assert!(
            curvefit::fit(model, "y", "x", &params, &x, &y, None).is_err(),
            "model {model:?} with {} x and {} y was accepted",
            x.len(),
            y.len()
        );
    }
}

#[test]
fn levenberg_marquardt_survives_non_finite_observations() {
    let x: Vec<f64> = (0..8).map(f64::from).collect();
    for bad in [f64::NAN, f64::INFINITY] {
        let mut y: Vec<f64> = x.iter().map(|v| 2.0 * v + 1.0).collect();
        y[3] = bad;
        // Either a refusal or a finite answer — never a panic and never a NaN
        // presented as a fitted parameter.
        if let Ok(fit) = curvefit::fit(
            "y = a*x + b",
            "y",
            "x",
            &["a".into(), "b".into()],
            &x,
            &y,
            None,
        ) {
            assert!(
                fit.fitted_parameters.iter().all(|p| p.is_finite()),
                "{bad}: {:?}",
                fit.fitted_parameters
            );
        }
    }
}

#[test]
fn a_parametric_sweep_of_a_billion_rows_is_refused_at_parse_time() {
    // A `PARAMETRIC` range is *materialised* by the parser, not lowered to a
    // loop, so an absurd step is an allocation request. `MAX_RANGE_ELEMENTS`
    // (100 000, transcribed from `AstBuilder`) already screens it — this pins
    // that the block form is covered and not just the bare `x = a:b:c` one.
    let message = parse_document(
        "y = 2*t\n\
         PARAMETRIC sweep (t, y)\n  \
           t = 0:1e-9:1\n\
         END\n",
    )
    .expect_err("a billion-row sweep must be refused")
    .to_string_message();
    assert!(message.contains("more than 100000 elements"), "{message}");

    // The boundary, and the fact that it is the *element count* that is
    // screened rather than the literal step: 100 000 rows still parse.
    let ok = parse_document(
        "y = 2*t\n\
         PARAMETRIC sweep (t, y)\n  \
           t = 0:1:99999\n\
         END\n",
    )
    .expect("100 000 rows are within the ceiling");
    assert_eq!(ok.blocks.parametric_tables[0].run_count(), 100_000);
}

#[test]
fn monte_carlo_does_not_preallocate_an_absurd_sample_count() {
    // The regression for a measured abort: `Vec::with_capacity(sample_count)`
    // on an untrusted count asked for **56 GB** at `samples = 1e9` and killed
    // the process — before the deadline predicate was consulted even once. The
    // Java never sees such a count because `SolveController` rejects anything
    // outside `[2, 1000]` first; this port has no controller, so the reservation
    // is bounded at the allocation site instead.
    let specs = BTreeMap::from([(
        "a".to_string(),
        UncertaintySpec {
            uncertainty: 0.5,
            ..UncertaintySpec::default()
        },
    )]);
    let mut drawn = 0usize;
    let outcome = montecarlo::run(
        "a = 10\nb = 2*a\n",
        &settings(),
        &specs,
        &BTreeMap::new(),
        1_000_000_000,
        42,
        || {
            drawn += 1;
            drawn > 8
        },
    )
    .expect("an absurd request is bounded by the deadline, not by an abort");
    assert!(outcome.truncated);
    assert_eq!(outcome.samples.len(), 8);
}

#[test]
fn nsga_ii_does_not_preallocate_an_absurd_population() {
    // The same defect class as the Monte Carlo count above, reachable the same
    // way: `Vec::with_capacity(pop_size)` over an untrusted `population_size`.
    // `OptimizeController.clampPositive(populationSize, 40, 200)` is the Java's
    // guard; the port applies the same ceiling in `optimize_multi`.
    let result = pareto::optimize_multi(&pareto_problem(usize::MAX, 1))
        .expect("an absurd population is clamped, not allocated");
    assert!(!result.front.is_empty());
    assert!(
        result.evaluations <= 2 * 200,
        "the population was not clamped: {} evaluations",
        result.evaluations
    );
}

#[test]
fn monte_carlo_with_zero_samples_returns_an_empty_outcome() {
    // Zero samples must not divide by zero computing the mean/σ, and must not
    // report statistics it has no data for.
    let mut specs = BTreeMap::new();
    specs.insert(
        "a".to_string(),
        UncertaintySpec {
            uncertainty: 0.5,
            ..UncertaintySpec::default()
        },
    );
    let outcome = montecarlo::run(
        "a = 10\nb = 2*a\n",
        &settings(),
        &specs,
        &BTreeMap::new(),
        0,
        42,
        || false,
    )
    .expect("zero samples is a legal request");
    assert!(outcome.samples.is_empty());
    // No statistics may be invented from no data — in particular no `0/0` mean
    // and no percentile indexed off the front of an empty vector.
    for stats in &outcome.stats {
        assert!(
            stats.mean.is_nan() || stats.mean == 0.0,
            "statistics claimed from zero samples: {stats:?}"
        );
    }
}

#[test]
fn monte_carlo_without_a_declared_uncertainty_is_refused() {
    let specs = BTreeMap::from([("a".to_string(), UncertaintySpec::default())]);
    let err = montecarlo::run(
        "a = 10\nb = 2*a\n",
        &settings(),
        &specs,
        &BTreeMap::new(),
        16,
        42,
        || false,
    )
    .expect_err("no uncertainty source is a validation failure");
    assert!(err
        .to_string_message()
        .contains("at least one variable with a declared uncertainty"));
}

#[test]
fn monte_carlo_honours_its_deadline_predicate_immediately() {
    // The wasm boundary supplies the clock. A predicate that is already expired
    // must truncate at zero samples rather than run the whole request.
    let specs = BTreeMap::from([(
        "a".to_string(),
        UncertaintySpec {
            uncertainty: 0.5,
            ..UncertaintySpec::default()
        },
    )]);
    let outcome = montecarlo::run(
        "a = 10\nb = 2*a\n",
        &settings(),
        &specs,
        &BTreeMap::new(),
        1_000_000_000,
        42,
        || true,
    )
    .expect("an expired budget is not an error");
    assert!(outcome.truncated);
    assert!(outcome.samples.is_empty());
}

#[test]
fn a_failing_monte_carlo_sample_is_counted_not_fatal() {
    // A draw that pushes the document out of its domain must be recorded as a
    // failed sample, leaving the run's other samples intact.
    let specs = BTreeMap::from([(
        "a".to_string(),
        UncertaintySpec {
            uncertainty: 20.0,
            ..UncertaintySpec::default()
        },
    )]);
    let outcome = montecarlo::run(
        "a = 1\nb = ln(a)\n",
        &settings(),
        &specs,
        &BTreeMap::new(),
        32,
        42,
        || false,
    )
    .expect("failing samples are not a fatal error");
    assert_eq!(outcome.samples.len(), 32);
    assert!(
        outcome.failed_samples > 0,
        "σ = 20 around a = 1 should have drawn a negative"
    );
    assert!(outcome.failed_samples < 32, "every sample failed");
}
