//! The ODE driver: the time loop, step guards, dense-output sampling and event
//! detection.
//!
//! Port of
//! `../frEES/backend/core/src/main/java/com/frees/backend/core/ode/OdeIntegrator.java`.
//!
//! It owns everything except the single-step advance, which it delegates to a
//! pluggable [`OdeMethod`] (explicit Runge–Kutta or stiff Rosenbrock/BDF).
//! Accepted steps are stored as `(t, y, f)` knots; output is sampled at evenly
//! spaced times via cubic Hermite interpolation, and event switching functions
//! are monitored for zero crossings between knots (bracket, then the analytic
//! root of that same interpolant when the switching function is a state
//! residual, bisection on it otherwise).
//!
//! # Dense output is Hermite, not the method's own interpolant
//!
//! Every method — including the FSAL Dormand–Prince pair, which *has* a
//! published 4th-order dense-output formula — is sampled through the same cubic
//! [`hermite`] built from the two bracketing knots and their derivatives. That
//! is what the Java does, so it is what the oracle's `ode_tables` rows contain,
//! and substituting a higher-order interpolant would change every sampled value.
//! The same interpolant is what a crossing is refined on — see
//! [`refine_crossing`] and [`hermite_root`] — which keeps a crossing time
//! consistent with the row the table reports around it.
//!
//! # No clock
//!
//! The Java `guard` checks both a step budget and a `System.nanoTime()`
//! deadline. `wasm32-unknown-unknown` has no clock (see [`crate::integral`],
//! which drops the same check for `IntegralSolver`), so [`MAX_STEPS`] is the
//! only bound here.

// The event/interpolation helpers thread the two bracketing knots plus the
// switching state through as separate parameters, exactly as the Java does.
// Bundling them into a struct to satisfy an argument count would obscure the
// line-for-line correspondence with `OdeIntegrator.java`.
#![allow(clippy::too_many_arguments)]
// Numerical kernels index parallel arrays by the same loop variable, mirroring
// the Java being transcribed — the same call `crate::linalg` makes.
#![allow(clippy::needless_range_loop)]

use crate::diag::{FreesError, Result};
use crate::ode::hermite_root;
use crate::ode::methods::{
    BdfMethod, ButcherTableau, OdeMethod, RosenbrockMethod, RungeKuttaMethod,
};
use crate::ode::problem::{EventRecord, OdeEvent, OdeProblem, OdeResult, MAX_OUTPUT_SAMPLES};
// The `Double.toString` spelling the Java error messages interpolate. Shared
// rather than re-derived: the golden fixtures compare error text verbatim, and
// two copies of this formatting would eventually disagree.
use crate::props::hx::java_double_to_string;

/// `OdeIntegrator.MAX_STEPS` — the hard cap on internal steps.
pub const MAX_STEPS: usize = 1_000_000;

/// `OdeIntegrator.BISECTION_ITERS` — bisections used to refine a crossing.
const BISECTION_ITERS: usize = 60;

/// How many `set` events may fire back to back — with **no ordinary accepted
/// step in between** — before the run has to justify itself.
///
/// **A divergence from the Java, added deliberately** (`docs/status-phase1.md`
/// ledger item 20). A `set` action whose assigned value re-arms the very
/// crossing that fired it (`EVENT r: L = 4 | falling -> set L = 4.0000000001`)
/// turns the time loop into a restart loop: each pass refines a crossing with
/// [`BISECTION_ITERS`] right-hand-side evaluations, advances `t` by ~0, and
/// pushes another knot. [`MAX_STEPS`] does bound it — `step_with_retries`
/// charges at least one step per pass — but only after 10^6 passes, and a pass
/// that bisects costs ~60 switching-function evaluations, each a full algebraic
/// inner solve for a document-level block. (Since the analytic root landed,
/// a state-residual event — which that example is — costs one instead of
/// sixty; the guard still earns its keep, because the restart loop itself is
/// what does not terminate.) Measured: the stiff-on-explicit case reaches
/// [`MAX_STEPS`] in 182 s; the same budget spent bisecting had not finished at
/// 45 s and was still running at 15 minutes. The Java's second guard —
/// `OdeProblem`'s `deadlineNanos`, checked in its `guard` — would cut that off,
/// and this port has no clock to check (`ode/problem.rs`, *No clock*).
///
/// # This is a rate test, not a firing count
///
/// The first cut of this guard simply refused after this many consecutive
/// restarts, and **that was wrong**: a fast modelled switch legitimately fires
/// on every step once the adaptive step size grows past the switching period.
/// A 500 s ramp reset at `Level = 0.1` fires ~5 000 times with no ordinary step
/// between any two of them, and it is a perfectly good model.
///
/// So the count only opens the question. What decides it is whether the window
/// advanced time fast enough to *finish*: project the elapsed `tf - t` at the
/// window's own rate and compare against the steps left in [`MAX_STEPS`]. The
/// margin between the two cases is not subtle — the 500 s sawtooth projects
/// ~4 × 10^3 further steps, and a self-re-arming `set` projects ~9 × 10^10.
pub const MAX_CONSECUTIVE_SET_RESTARTS: usize = 1_000;

/// `Math.min` (NaN-propagating, `-0.0 < 0.0`). Rust's `f64::min` returns the
/// non-NaN operand instead, and a diverged controller can hand NaN in here.
fn java_min(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else if a < b {
        a
    } else if b < a {
        b
    } else if a.is_sign_negative() {
        a
    } else {
        b
    }
}

/// `Math.max` (NaN-propagating, `0.0 > -0.0`).
fn java_max(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else if a > b {
        a
    } else if b > a {
        b
    } else if a.is_sign_positive() {
        a
    } else {
        b
    }
}

// ---------------------------------------------------------------------------
// Entry points
// ---------------------------------------------------------------------------

/// Integrates `p` and returns the trajectory sampled at
/// [`OdeProblem::sample_count`] evenly spaced times over `[t0, endTime]`.
///
/// Port of `OdeIntegrator.integrate`.
pub fn integrate(p: &OdeProblem<'_>) -> Result<OdeResult> {
    // Held across the output sampling as well as `run`: materialising the rows
    // re-solves the algebraic system once per row, and those solves are block
    // loops that would otherwise report a sawtooth after the bar had already
    // reached the end of this transient's span.
    let _progress = crate::progress::claim();
    // Before `run`, not after: the point is to refuse the request rather than
    // spend the whole integration and then abort on the output allocation.
    let count = p.sample_count();
    if count > MAX_OUTPUT_SAMPLES {
        return Err(FreesError::solver(format!(
            "DYNAMIC: points = {count} would materialise more than {MAX_OUTPUT_SAMPLES} \
             output rows. Use fewer points."
        )));
    }
    let tr = run(p)?;
    let mut times = vec![0.0; count];
    for i in 0..count {
        times[i] = if count == 1 {
            tr.end_time
        } else {
            p.t0 + (tr.end_time - p.t0) * i as f64 / (count - 1) as f64
        };
    }
    let states = interpolate_at(&tr.knot_t, &tr.knot_y, &tr.knot_f, &times);
    Ok(OdeResult {
        times,
        states,
        events: tr.recorded,
        stopped: tr.stopped,
        end_time: tr.end_time,
        accepted_steps: tr.accepted,
        rejected_steps: tr.rejected,
    })
}

/// Integrates `p` and samples the trajectory at caller-chosen (ascending)
/// times. Port of `OdeIntegrator.integrateAndSampleAt`.
pub fn integrate_and_sample_at(p: &OdeProblem<'_>, target_times: &[f64]) -> Result<Vec<Vec<f64>>> {
    let _progress = crate::progress::claim();
    let tr = run(p)?;
    Ok(interpolate_at(
        &tr.knot_t,
        &tr.knot_y,
        &tr.knot_f,
        target_times,
    ))
}

// ---------------------------------------------------------------------------
// The time loop
// ---------------------------------------------------------------------------

/// The accepted-step knots plus event bookkeeping of one integration.
/// Port of the private `OdeIntegrator.Trajectory` record.
struct Trajectory {
    knot_t: Vec<f64>,
    knot_y: Vec<Vec<f64>>,
    knot_f: Vec<Vec<f64>>,
    recorded: Vec<EventRecord>,
    stopped: bool,
    end_time: f64,
    accepted: usize,
    rejected: usize,
}

/// The shared time loop: advances knot by knot, handling stop events
/// (terminate) and set events (discrete state reassignment at the crossing,
/// then resume). Port of `OdeIntegrator.run`.
fn run(p: &OdeProblem<'_>) -> Result<Trajectory> {
    // Integration time is the only honest progress signal a transient has: the
    // step count is adaptive and its total is unknown in advance. The claim is
    // the load-bearing half — every RHS evaluation, event scan and per-step
    // algebraic solve goes through `engine::run_blocks`, which would otherwise
    // reset the bar to this block's `0` thousands of times. Taken before the
    // setup rather than at the loop, because `compute_initial_step` already
    // evaluates the system. See `crate::progress`.
    let progress = crate::progress::claim();
    let method = resolve_method(&p.method)?;
    // A non-finite endpoint has to be screened *before* the `tf <= t0` test,
    // which it slips past in both directions: `tf = inf` is greater than any
    // `t0`, and every comparison against `NaN` is false. What follows is worse
    // than a crash — `span` and `min_step` become non-finite, the loop condition
    // `t < tf - min_step` evaluates to false at the first pass, and `integrate`
    // then publishes a full-height table of `[NaN, inf, inf, …]` as if it were a
    // trajectory. Measured: `tf = inf` returned 200 such rows. The Java has the
    // same hole; a document cannot reach it because the parser's `signedNumber`
    // admits no infinite literal, but `OdeProblem` is public API here and
    // `analysis` builds one directly.
    if !p.t0.is_finite() || !p.tf.is_finite() {
        return Err(FreesError::solver(format!(
            "DYNAMIC: the time span must be finite (got {} .. {}).",
            java_double_to_string(p.t0),
            java_double_to_string(p.tf)
        )));
    }
    if p.tf <= p.t0 {
        return Err(FreesError::solver(format!(
            "DYNAMIC: the time span must satisfy t0 < tf (got {} .. {}).",
            java_double_to_string(p.t0),
            java_double_to_string(p.tf)
        )));
    }
    let span = p.tf - p.t0;
    let min_step = span * 1e-12;

    let mut knot_t: Vec<f64> = Vec::new();
    let mut knot_y: Vec<Vec<f64>> = Vec::new();
    let mut knot_f: Vec<Vec<f64>> = Vec::new();

    let mut t = p.t0;
    let mut y = p.y0.clone();
    // The Java checks only the initial *derivative*. A non-finite initial
    // *state* with a finite derivative slips through and poisons the step
    // controller instead: `scale = atol + rtol*|NaN|` is NaN, so every error
    // test and every `h_use < min_step` comparison is false, no step is ever
    // rejected and no underflow is ever declared. The run then burns the whole
    // `MAX_STEPS` budget and blames stiffness. Measured before this line: a
    // `y0 = [NaN]` problem with `der(y) = 1` reported "exceeded 1000000
    // integration steps". One comparison up front is both faster and true.
    check_finite(&y, t, "initial state")?;
    let mut f = p.rhs.eval(t, &y)?;
    check_finite(&f, t, "initial derivative")?;
    knot_t.push(t);
    knot_y.push(y.clone());
    knot_f.push(f.clone());

    let fixed = !method.adaptive();
    let h_fixed = match p.fixed_step {
        Some(step) => step,
        None => span / (p.sample_count() - 1) as f64,
    };
    let mut h = compute_initial_step(p, method.as_ref(), &y, &f, span, fixed, h_fixed)?;

    let mut recorded: Vec<EventRecord> = Vec::new();
    let mut g_prev = eval_events(p, t, &y)?;
    let mut accepted = 0usize;
    let mut rejected = 0usize;
    let mut steps = 0usize;
    let mut stopped = false;
    let mut end_time = p.tf;
    // Set-event restarts taken back to back with no ordinary accepted step in
    // between, and the time the current such window opened — see
    // [`MAX_CONSECUTIVE_SET_RESTARTS`].
    let mut consecutive_sets = 0usize;
    let mut window_start_t = p.t0;

    while t < p.tf - min_step {
        progress.report((t - p.t0) / span);
        let out = step_with_retries(method.as_ref(), p, t, &y, &f, h, min_step, steps)?;
        steps += out.extra_steps;
        rejected += out.rejections;
        let sr = out.sr;
        let t_new = t + out.h_use;
        // An accepted StepResult always carries both vectors; `step_with_retries`
        // only returns once `accepted` is true.
        let (y_new, f_new) = match (sr.y_new, sr.f_new) {
            (Some(yn), Some(fnw)) => (yn, fnw),
            _ => {
                return Err(FreesError::solver(
                    "DYNAMIC: internal error — an accepted step carried no state.",
                ))
            }
        };
        check_finite(&y_new, t_new, "state")?;
        check_finite(&f_new, t_new, "derivative")?;

        let g_new = eval_events(p, t_new, &y_new)?;
        let hit = earliest_event(p, t, &y, &f, t_new, &y_new, &f_new, &g_prev, &g_new)?;
        let mut stop_at = None;
        if let Some(hit) = hit.as_ref() {
            if record_events(p, hit, &mut recorded, &mut knot_t, &mut knot_y, &mut knot_f)? {
                stop_at = Some(hit.time);
            }
        }
        if let Some(time) = stop_at {
            end_time = time;
            stopped = true;
            break;
        }

        match hit {
            Some(hit) if p.events[hit.event].is_set() => {
                // Discrete reassignment: jump to the crossing, overwrite the
                // target state, and restart integration from the modified
                // state. The knot at the crossing carries the POST-set state so
                // the trajectory shows the switch; re-evaluating gPrev there
                // re-arms every event against the new state (direction guards
                // stop immediate retriggering).
                consecutive_sets += 1;
                if consecutive_sets >= MAX_CONSECUTIVE_SET_RESTARTS {
                    // Close the window and ask the only question that matters:
                    // at the rate this window advanced time, can the run reach
                    // `tf` inside the step budget it has left? A modelled
                    // switch that fires thousands of times still says yes by a
                    // wide margin; a `set` that re-arms its own crossing misses
                    // by ten orders of magnitude.
                    let advanced = hit.time - window_start_t;
                    let projected = if advanced > 0.0 {
                        (p.tf - hit.time) / advanced * MAX_CONSECUTIVE_SET_RESTARTS as f64
                    } else {
                        f64::INFINITY
                    };
                    // `is_nan` first, then a positive comparison: the negated
                    // form `!(… <= …)` says the same thing about NaN but trips
                    // `clippy::neg_cmp_op_on_partial_ord`.
                    if projected.is_nan() || steps as f64 + projected > MAX_STEPS as f64 {
                        return Err(FreesError::solver(format!(
                            "EVENT {}: the set action re-arms its own crossing — \
                             {MAX_CONSECUTIVE_SET_RESTARTS} consecutive set events fired \
                             between t = {} and t = {}, which is too little progress to reach \
                             t = {} within the {MAX_STEPS}-step budget. Move the assigned value \
                             clear of the crossing, or give the event a direction the new state \
                             cannot immediately satisfy.",
                            hit.name,
                            java_double_to_string(window_start_t),
                            java_double_to_string(hit.time),
                            java_double_to_string(p.tf)
                        )));
                    }
                    consecutive_sets = 0;
                    window_start_t = hit.time;
                }
                t = hit.time;
                y = hit.y.clone();
                let set = &p.events[hit.event].set;
                if let Some(reset) = set.as_ref() {
                    // `DynamicSolver.eventSetStateIndex` already rejects a
                    // target that is not a state, so this is caller-guaranteed
                    // — but the Java would throw ArrayIndexOutOfBounds here and
                    // the wasm profile is `panic = "abort"`, so it becomes an
                    // ordinary solver error instead of killing the worker.
                    let value = reset.value.eval(t, &y)?;
                    match y.get_mut(reset.index) {
                        Some(slot) => *slot = value,
                        None => {
                            return Err(FreesError::solver(format!(
                                "EVENT {}: set target index {} is outside the {}-state \
                                 vector of this DYNAMIC block.",
                                hit.name,
                                reset.index,
                                y.len()
                            )))
                        }
                    }
                }
                f = p.rhs.eval(t, &y)?;
                check_finite(&f, t, "post-event derivative")?;
                g_prev = eval_events(p, t, &y)?;
            }
            _ => {
                consecutive_sets = 0;
                window_start_t = t_new;
                accepted += 1;
                t = t_new;
                y = y_new;
                f = f_new;
                g_prev = g_new;
                h = if fixed { h_fixed } else { sr.h_next };
                if let Some(max) = p.max_step {
                    h = java_min(h, max);
                }
            }
        }
        knot_t.push(t);
        knot_y.push(y.clone());
        knot_f.push(f.clone());
    }

    Ok(Trajectory {
        knot_t,
        knot_y,
        knot_f,
        recorded,
        stopped,
        end_time,
        accepted,
        rejected,
    })
}

// ── Step driving ─────────────────────────────────────────────────────────

/// One accepted step plus the bookkeeping accrued reaching it.
/// Port of the private `OdeIntegrator.StepOutcome` record.
struct StepOutcome {
    sr: crate::ode::methods::StepResult,
    h_use: f64,
    extra_steps: usize,
    rejections: usize,
}

/// First-step size: fixed methods use `h_fixed`; adaptive methods honour an
/// explicit fixed step or fall back to the automatic initial-step estimate,
/// clamped to `max_step`. Port of `OdeIntegrator.computeInitialStep`.
fn compute_initial_step(
    p: &OdeProblem<'_>,
    method: &dyn OdeMethod,
    y: &[f64],
    f: &[f64],
    span: f64,
    fixed: bool,
    h_fixed: f64,
) -> Result<f64> {
    let mut h = if fixed {
        h_fixed
    } else {
        match p.fixed_step {
            Some(step) => step,
            None => initial_step(p, method, y, f, span)?,
        }
    };
    if let Some(max) = p.max_step {
        h = java_min(h, max);
    }
    Ok(h)
}

/// Attempts a step from `(t, y)`, shrinking and retrying on rejection until
/// accepted or the step size underflows. `steps_so_far` seeds the guard
/// counter; the returned outcome reports how many guard ticks and rejections it
/// consumed. Port of `OdeIntegrator.stepWithRetries`.
fn step_with_retries(
    method: &dyn OdeMethod,
    p: &OdeProblem<'_>,
    t: f64,
    y: &[f64],
    f: &[f64],
    h: f64,
    min_step: f64,
    steps_so_far: usize,
) -> Result<StepOutcome> {
    let mut steps = steps_so_far;
    let mut rejected = 0usize;
    steps += 1;
    guard(steps)?;
    let mut h_use = java_min(h, p.tf - t);
    let mut sr = method.step(p.rhs, t, y, f, h_use, p)?;
    while !sr.accepted {
        rejected += 1;
        h_use = java_min(sr.h_next, p.tf - t);
        if h_use < min_step {
            return Err(FreesError::solver(format!(
                "DYNAMIC: step size underflow near t = {} — the system may be too \
                 stiff for method '{}' (try ode23s or ode15s).",
                java_double_to_string(t),
                p.method
            )));
        }
        steps += 1;
        guard(steps)?;
        sr = method.step(p.rhs, t, y, f, h_use, p)?;
    }
    Ok(StepOutcome {
        sr,
        h_use,
        extra_steps: steps - steps_so_far,
        rejections: rejected,
    })
}

/// Records an event hit and returns true when it is a stop event, in which case
/// the crossing knot has been appended and the caller should terminate the
/// integration. Port of `OdeIntegrator.recordEvents` (its `hit == null` guard
/// is the caller's `if let`).
fn record_events(
    p: &OdeProblem<'_>,
    hit: &EventHit,
    recorded: &mut Vec<EventRecord>,
    knot_t: &mut Vec<f64>,
    knot_y: &mut Vec<Vec<f64>>,
    knot_f: &mut Vec<Vec<f64>>,
) -> Result<bool> {
    recorded.push(EventRecord {
        name: hit.name.clone(),
        time: hit.time,
        state: hit.y.clone(),
    });
    if hit.stop {
        knot_t.push(hit.time);
        knot_y.push(hit.y.clone());
        knot_f.push(p.rhs.eval(hit.time, &hit.y)?);
        return Ok(true);
    }
    Ok(false)
}

// ── Method resolution ───────────────────────────────────────────────────

/// Maps a solver name to its stepper. Port of `OdeIntegrator.resolveMethod`.
///
/// The Java guards `name == null` and substitutes `ode45`; `OdeProblem.method`
/// is a `String` here, so the empty name plays that role.
pub fn resolve_method(name: &str) -> Result<Box<dyn OdeMethod>> {
    let m = if name.is_empty() {
        "ode45".to_string()
    } else {
        name.to_ascii_lowercase()
    };
    let method: Box<dyn OdeMethod> = match m.as_str() {
        "ode1" | "euler" => Box::new(RungeKuttaMethod::new(ButcherTableau::euler())),
        "ode2" | "heun" => Box::new(RungeKuttaMethod::new(ButcherTableau::heun())),
        "ode3" => Box::new(RungeKuttaMethod::new(ButcherTableau::rk3())),
        "ode4" | "rk4" => Box::new(RungeKuttaMethod::new(ButcherTableau::rk4())),
        "ode5" => Box::new(RungeKuttaMethod::new(ButcherTableau::dopri5_fixed())),
        "ode45" => Box::new(RungeKuttaMethod::new(ButcherTableau::dopri54())),
        "ode23" => Box::new(RungeKuttaMethod::new(ButcherTableau::bogacki_shampine32())),
        "ode23s" => Box::new(RosenbrockMethod),
        "ode15s" | "ode23t" | "ode23tb" => Box::new(BdfMethod),
        _ => {
            return Err(FreesError::solver(format!(
                "DYNAMIC: unknown method '{name}'. Supported: ode1, ode2, ode3, \
                 ode4, ode5, ode45, ode23, ode23s (stiff), ode15s (stiff)."
            )))
        }
    };
    Ok(method)
}

/// The standard automatic initial step-size estimate from the ODE literature.
///
/// Port of `OdeIntegrator.initialStep`. A fixed fraction of the span is a poor
/// first step for systems with fast early dynamics — too large a first step can
/// diverge before the controller recovers. This bases the first step on the
/// scaled norms of `y0` and `f(t0,y0)` and a trial derivative.
fn initial_step(
    p: &OdeProblem<'_>,
    method: &dyn OdeMethod,
    y0: &[f64],
    f0: &[f64],
    span: f64,
) -> Result<f64> {
    let n = y0.len();
    let mut scale = vec![0.0; n];
    for i in 0..n {
        scale[i] = p.atol + p.rtol * y0[i].abs();
    }
    let d0 = scaled_norm(y0, &scale);
    let d1 = scaled_norm(f0, &scale);
    let mut h0 = if d0 < 1e-5 || d1 < 1e-5 {
        1e-6
    } else {
        0.01 * d0 / d1
    };
    h0 = java_min(h0, span);

    let mut y1 = vec![0.0; n];
    for i in 0..n {
        y1[i] = y0[i] + h0 * f0[i];
    }
    let f1 = p.rhs.eval(p.t0 + h0, &y1)?;
    let mut df = vec![0.0; n];
    for i in 0..n {
        df[i] = f1[i] - f0[i];
    }
    let d2 = scaled_norm(&df, &scale) / h0;

    let max_d = java_max(d1, d2);
    let h1 = if max_d <= 1e-15 {
        java_max(1e-6, h0 * 1e-3)
    } else {
        libm::pow(0.01 / max_d, 1.0 / f64::from(method.order() + 1))
    };
    let h = java_min(100.0 * h0, h1);
    Ok(java_min(java_max(h, span * 1e-9), span))
}

fn scaled_norm(v: &[f64], scale: &[f64]) -> f64 {
    let mut sum = 0.0;
    for i in 0..v.len() {
        let r = v[i] / scale[i];
        sum += r * r;
    }
    (sum / v.len() as f64).sqrt()
}

// ── Guards ──────────────────────────────────────────────────────────────

/// `OdeIntegrator.guard` — both halves now: the step cap, and the wall-clock
/// deadline the Java kept in `System.nanoTime()` and this port dropped for
/// nine phases because core has no clock. Wave C1 restores it through
/// [`crate::ode::deadline`]: the boundary installs the predicate and its
/// message; the native/parity path installs nothing and can never strike.
fn guard(steps: usize) -> Result<()> {
    if steps > MAX_STEPS {
        return Err(FreesError::solver(format!(
            "DYNAMIC: exceeded {MAX_STEPS} integration steps — the system may be \
             too stiff or the tolerances too tight."
        )));
    }
    if let Some(message) = crate::ode::deadline::strike() {
        return Err(FreesError::solver(message));
    }
    Ok(())
}

fn check_finite(v: &[f64], t: f64, what: &str) -> Result<()> {
    for &x in v {
        if x.is_nan() || x.is_infinite() {
            return Err(FreesError::solver(format!(
                "DYNAMIC: non-finite {what} (NaN/Inf) at t = {} — check the model \
                 for division by zero or domain errors.",
                java_double_to_string(t)
            )));
        }
    }
    Ok(())
}

// ── Events ──────────────────────────────────────────────────────────────

/// A refined zero crossing. Port of the package-visible `OdeIntegrator.EventHit`
/// record; the Java holds the `OdeEvent` itself, this holds its index into
/// [`OdeProblem::events`] so the borrow checker stays out of the way.
#[derive(Debug, Clone, PartialEq)]
struct EventHit {
    name: String,
    time: f64,
    y: Vec<f64>,
    stop: bool,
    event: usize,
}

fn eval_events(p: &OdeProblem<'_>, t: f64, y: &[f64]) -> Result<Vec<f64>> {
    if p.events.is_empty() {
        return Ok(Vec::new());
    }
    let mut g = vec![0.0; p.events.len()];
    for i in 0..g.len() {
        g[i] = p.events[i].g.eval(t, y)?;
    }
    Ok(g)
}

/// Earliest matching zero crossing across all events on `(t, tNew]`, refined on
/// the Hermite interpolant; `None` if none. Port of
/// `OdeIntegrator.earliestEvent`.
fn earliest_event(
    p: &OdeProblem<'_>,
    t: f64,
    y: &[f64],
    f: &[f64],
    t_new: f64,
    y_new: &[f64],
    f_new: &[f64],
    g_prev: &[f64],
    g_new: &[f64],
) -> Result<Option<EventHit>> {
    let mut best: Option<EventHit> = None;
    for i in 0..p.events.len() {
        let ev = &p.events[i];
        if !ev.triggers(g_prev[i], g_new[i]) {
            continue;
        }
        let tc = refine_crossing(ev, t, y, f, t_new, y_new, f_new, g_prev[i], g_new[i])?;
        let yc = hermite(t, y, f, t_new, y_new, f_new, tc);
        if best.as_ref().is_none_or(|b| tc < b.time) {
            best = Some(EventHit {
                name: ev.name.clone(),
                time: tc,
                y: yc,
                stop: ev.stop,
                event: i,
            });
        }
    }
    Ok(best)
}

/// Zero crossing on the Hermite interpolant. Port of
/// `OdeIntegrator.refineCrossing`, with the analytic short cut below taken
/// first: when the switching function is a residual of one state — what
/// `EVENT x = value` compiles to — the crossing is a root of a cubic and is
/// solved in closed form instead of by 60 bisections. Neither path evaluates
/// the right-hand side.
fn refine_crossing(
    ev: &OdeEvent<'_>,
    t: f64,
    y: &[f64],
    f: &[f64],
    t_new: f64,
    y_new: &[f64],
    f_new: &[f64],
    g_prev: f64,
    g_new: f64,
) -> Result<f64> {
    if let Some(tc) = analytic_crossing(ev, t, y, f, t_new, y_new, f_new, g_prev, g_new)? {
        return Ok(tc);
    }
    let mut lo = t;
    let mut hi = t_new;
    let mut g_lo = ev.g.eval(lo, y)?;
    for _ in 0..BISECTION_ITERS {
        let mid = 0.5 * (lo + hi);
        let ym = hermite(t, y, f, t_new, y_new, f_new, mid);
        let gm = ev.g.eval(mid, &ym)?;
        if gm == 0.0 {
            return Ok(mid);
        }
        if (g_lo < 0.0) != (gm < 0.0) {
            hi = mid;
        } else {
            lo = mid;
            g_lo = gm;
        }
    }
    Ok(0.5 * (lo + hi))
}

/// The crossing as the analytic root of the cubic Hermite interpolant, or
/// `None` when this event is not a plain state residual and the caller must
/// bisect.
///
/// Two knot values do not determine a cubic in a general `g` — that would need
/// `g'`, which nobody has — so the closed form is only available when `g` *is*
/// a state, up to an offset and an orientation. That is recovered from numbers
/// already in hand (`g_prev`/`g_new` were evaluated by the step loop): if
/// `g = ±(y[j] − level)` then `y[j] ∓ g` is the same `level` at both knots.
/// Two exact hits is a strong signal but not a proof, so one — not sixty —
/// evaluation inside the step confirms the identity away from the knots
/// before the analytic root is trusted. Any doubt returns `None`, and the
/// bisection answers.
fn analytic_crossing(
    ev: &OdeEvent<'_>,
    t: f64,
    y: &[f64],
    f: &[f64],
    t_new: f64,
    y_new: &[f64],
    f_new: &[f64],
    g_prev: f64,
    g_new: f64,
) -> Result<Option<f64>> {
    let dt = t_new - t;
    if !dt.is_finite() || dt <= 0.0 {
        return Ok(None);
    }
    let Some((j, sign, level)) = state_residual(y, y_new, g_prev, g_new) else {
        return Ok(None);
    };
    let Some(theta) = hermite_root::cubic_hermite_root(
        y[j],
        f[j],
        y_new[j],
        f_new[j],
        dt,
        level,
        sign,
        ev.direction,
    ) else {
        return Ok(None);
    };
    // A third of the way in, not half: the mid-point is where a symmetric
    // interpolant crosses, and a probe sitting on the crossing confirms
    // nothing.
    let probe = t + dt / 3.0;
    let yp = hermite(t, y, f, t_new, y_new, f_new, probe);
    let gp = ev.g.eval(probe, &yp)?;
    if yp[j] - sign * gp != level {
        return Ok(None);
    }
    let tc = t + theta * dt;
    // `theta ∈ (0, 1]`, but the rescale can still land on a knot; the caller's
    // contract is a time inside the step it bracketed.
    Ok(Some(tc.clamp(t, t_new)))
}

/// The `(index, orientation, level)` of the state this switching function is a
/// residual of — `g = orientation · (y[index] − level)` — as witnessed at both
/// knots, or `None`.
fn state_residual(y: &[f64], y_new: &[f64], g_prev: f64, g_new: f64) -> Option<(usize, f64, f64)> {
    for j in 0..y.len().min(y_new.len()) {
        for sign in [1.0f64, -1.0] {
            let level = y[j] - sign * g_prev;
            if level.is_finite() && y_new[j] - sign * g_new == level {
                return Some((j, sign, level));
            }
        }
    }
    None
}

// ── Dense output (cubic Hermite) ────────────────────────────────────────

/// Cubic Hermite interpolation between two `(t, y, f)` knots. Port of the
/// package-visible `OdeIntegrator.hermite`.
pub(crate) fn hermite(
    t0: f64,
    y0: &[f64],
    f0: &[f64],
    t1: f64,
    y1: &[f64],
    f1: &[f64],
    tau: f64,
) -> Vec<f64> {
    let n = y0.len();
    let dt = t1 - t0;
    if dt == 0.0 {
        return y1.to_vec();
    }
    let mut out = vec![0.0; n];
    let th = (tau - t0) / dt;
    let th2 = th * th;
    let th3 = th2 * th;
    let h00 = 2.0 * th3 - 3.0 * th2 + 1.0;
    let h10 = th3 - 2.0 * th2 + th;
    let h01 = -2.0 * th3 + 3.0 * th2;
    let h11 = th3 - th2;
    for d in 0..n {
        out[d] = h00 * y0[d] + h10 * dt * f0[d] + h01 * y1[d] + h11 * dt * f1[d];
    }
    out
}

/// Samples the trajectory at the given (ascending) times via cubic Hermite
/// interpolation between accepted-step knots; one state row per time. Port of
/// `OdeIntegrator.interpolateAt`.
///
/// `run` always leaves at least two knots, so the `knots < 2` arms are
/// unreachable from [`integrate`]; the Java indexes `knots - 2` unguarded and
/// would throw there.
fn interpolate_at(
    knot_t: &[f64],
    knot_y: &[Vec<f64>],
    knot_f: &[Vec<f64>],
    taus: &[f64],
) -> Vec<Vec<f64>> {
    let knots = knot_t.len();
    if knots == 0 {
        return vec![Vec::new(); taus.len()];
    }
    if knots == 1 {
        return taus.iter().map(|_| knot_y[0].clone()).collect();
    }
    let mut out = Vec::with_capacity(taus.len());
    let mut knot = 0usize;
    for &tau in taus {
        while knot < knots - 2 && knot_t[knot + 1] < tau {
            knot += 1;
        }
        let lo = knot.min(knots - 2);
        let hi = lo + 1;
        out.push(hermite(
            knot_t[lo],
            &knot_y[lo],
            &knot_f[lo],
            knot_t[hi],
            &knot_y[hi],
            &knot_f[hi],
            tau,
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ode::problem::{OdeRhs, OdeScalarFn};

    /// A problem over `rhs` with the `DynamicSystem.Options` defaults
    /// (`rtol = 1e-6`, `atol = 1e-9`) and the `DynamicSolver.solve` `maxStep`
    /// default of `span / 100`.
    fn dynamic_problem<'a>(
        method: &str,
        rhs: &'a dyn OdeRhs,
        y0: Vec<f64>,
        t0: f64,
        tf: f64,
        points: usize,
    ) -> OdeProblem<'a> {
        OdeProblem {
            method: method.to_string(),
            t0,
            tf,
            y0,
            rhs,
            points: Some(points),
            fixed_step: None,
            rtol: 1e-6,
            atol: 1e-9,
            max_step: Some((tf - t0) / 100.0),
            events: Vec::new(),
        }
    }

    pub(super) fn scalar<'a>(
        f: impl Fn(f64, &[f64]) -> Result<f64> + 'a,
    ) -> Box<dyn OdeScalarFn + 'a> {
        Box::new(f)
    }

    pub(super) fn assert_close(got: f64, want: f64, tol: f64, what: &str) {
        assert!(
            (got - want).abs() <= tol,
            "{what}: got {got}, want {want} (|Δ| = {})",
            (got - want).abs()
        );
    }

    // ── Dense output ────────────────────────────────────────────────────────

    #[test]
    fn hermite_reproduces_a_cubic_exactly() {
        // p(t) = 1 + 2t - 3t^2 + 4t^3, p'(t) = 2 - 6t + 12t^2 on [0, 1].
        let p = |t: f64| 1.0 + 2.0 * t - 3.0 * t * t + 4.0 * t * t * t;
        let dp = |t: f64| 2.0 - 6.0 * t + 12.0 * t * t;
        for &tau in &[0.0, 0.1, 0.25, 0.5, 0.75, 0.9, 1.0] {
            let out = hermite(0.0, &[p(0.0)], &[dp(0.0)], 1.0, &[p(1.0)], &[dp(1.0)], tau);
            assert_close(out[0], p(tau), 1e-13, "hermite cubic");
        }
    }

    #[test]
    fn hermite_matches_the_endpoints_and_slopes() {
        let out0 = hermite(2.0, &[5.0], &[7.0], 4.0, &[9.0], &[11.0], 2.0);
        assert_eq!(out0[0], 5.0);
        let out1 = hermite(2.0, &[5.0], &[7.0], 4.0, &[9.0], &[11.0], 4.0);
        assert_eq!(out1[0], 9.0);
        // Slope at the left knot, by a tight finite difference.
        let eps = 1e-6;
        let a = hermite(2.0, &[5.0], &[7.0], 4.0, &[9.0], &[11.0], 2.0 + eps)[0];
        assert_close((a - 5.0) / eps, 7.0, 1e-3, "left slope");
    }

    #[test]
    fn hermite_on_a_degenerate_interval_returns_the_right_knot() {
        let out = hermite(3.0, &[1.0], &[0.0], 3.0, &[2.0], &[0.0], 3.0);
        assert_eq!(out, vec![2.0]);
    }

    #[test]
    fn interpolate_at_walks_the_knots_in_order() {
        // y = t, f = 1 on knots 0,1,2.
        let kt = vec![0.0, 1.0, 2.0];
        let ky = vec![vec![0.0], vec![1.0], vec![2.0]];
        let kf = vec![vec![1.0], vec![1.0], vec![1.0]];
        let taus = [0.0, 0.5, 1.0, 1.5, 2.0];
        let out = interpolate_at(&kt, &ky, &kf, &taus);
        for (i, &tau) in taus.iter().enumerate() {
            assert_close(out[i][0], tau, 1e-13, "linear knot walk");
        }
    }

    // ── Fixed-step methods vs the analytic solution ─────────────────────────

    #[test]
    fn every_fixed_step_method_integrates_exponential_decay() {
        // y' = -y, y(0) = 1  =>  y = e^-t.
        let rhs = |_t: f64, y: &[f64]| Ok(vec![-y[0]]);
        // ode1 (Euler) is only first order; give each method its own tolerance.
        for (method, tol) in [
            ("ode1", 2e-3),
            ("ode2", 1e-5),
            ("ode3", 1e-7),
            ("ode4", 1e-9),
            ("ode5", 1e-11),
        ] {
            let mut p = dynamic_problem(method, &rhs, vec![1.0], 0.0, 1.0, 11);
            p.fixed_step = Some(0.001);
            let r = integrate(&p).unwrap();
            assert_eq!(r.times.len(), 11);
            assert!(!r.stopped);
            assert_eq!(r.end_time, 1.0);
            for i in 0..r.times.len() {
                let want = libm::exp(-r.times[i]);
                assert_close(r.states[i][0], want, tol, method);
            }
        }
    }

    #[test]
    fn method_aliases_resolve_to_the_same_stepper() {
        for (a, b) in [("ode1", "euler"), ("ode2", "heun"), ("ode4", "rk4")] {
            assert_eq!(
                resolve_method(a).unwrap().name(),
                resolve_method(b).unwrap().name()
            );
        }
        for alias in ["ode15s", "ode23t", "ode23tb"] {
            assert_eq!(resolve_method(alias).unwrap().name(), "ode15s");
        }
        // Case-insensitive, and the empty name is the Java `null` default.
        assert_eq!(resolve_method("ODE45").unwrap().name(), "ode45");
        assert_eq!(resolve_method("").unwrap().name(), "ode45");
    }

    #[test]
    fn an_unknown_method_names_the_supported_set() {
        // `.err()` rather than `unwrap_err()`: the Ok side is a boxed trait
        // object, which has no `Debug`.
        let err = resolve_method("ode99").err().unwrap();
        let msg = format!("{err}");
        assert!(msg.contains("unknown method 'ode99'"), "{msg}");
        assert!(msg.contains("ode23s (stiff)"), "{msg}");
    }

    // ── Adaptive methods ────────────────────────────────────────────────────

    #[test]
    fn ode45_integrates_the_harmonic_oscillator_over_many_periods() {
        // x'' = -x  =>  y = [x, v], y' = [v, -x]. x(0)=1, v(0)=0 => x = cos t.
        let rhs = |_t: f64, y: &[f64]| Ok(vec![y[1], -y[0]]);
        let tf = 20.0 * core::f64::consts::PI;
        let p = dynamic_problem("ode45", &rhs, vec![1.0, 0.0], 0.0, tf, 41);
        let r = integrate(&p).unwrap();
        for i in 0..r.times.len() {
            let t = r.times[i];
            assert_close(r.states[i][0], libm::cos(t), 2e-5, "x(t)");
            assert_close(r.states[i][1], -libm::sin(t), 2e-5, "v(t)");
        }
        assert!(r.accepted_steps > 0);
    }

    #[test]
    fn ode23_integrates_a_forced_linear_system() {
        // y' = -2y + 4, y(0) = 0  =>  y = 2(1 - e^-2t).
        let rhs = |_t: f64, y: &[f64]| Ok(vec![-2.0 * y[0] + 4.0]);
        let p = dynamic_problem("ode23", &rhs, vec![0.0], 0.0, 3.0, 13);
        let r = integrate(&p).unwrap();
        for i in 0..r.times.len() {
            let want = 2.0 * (1.0 - libm::exp(-2.0 * r.times[i]));
            assert_close(r.states[i][0], want, 1e-5, "forced linear");
        }
    }

    #[test]
    fn stiff_methods_integrate_a_two_timescale_system_the_explicit_pair_struggles_with() {
        // y' = -1000(y - cos t) - sin t, y(0) = 1  =>  y = cos t exactly.
        let rhs = |t: f64, y: &[f64]| Ok(vec![-1000.0 * (y[0] - libm::cos(t)) - libm::sin(t)]);
        for method in ["ode23s", "ode15s"] {
            let p = dynamic_problem(method, &rhs, vec![1.0], 0.0, 2.0, 21);
            let r = integrate(&p).unwrap();
            for i in 0..r.times.len() {
                assert_close(r.states[i][0], libm::cos(r.times[i]), 1e-3, method);
            }
        }
    }

    #[test]
    fn ode23s_handles_van_der_pol_at_a_stiff_mu() {
        // The classic ode23s exercise: mu = 100 is far beyond explicit reach.
        let mu = 100.0;
        let rhs = move |_t: f64, y: &[f64]| Ok(vec![y[1], mu * (1.0 - y[0] * y[0]) * y[1] - y[0]]);
        let p = dynamic_problem("ode23s", &rhs, vec![2.0, 0.0], 0.0, 20.0, 21);
        let r = integrate(&p).unwrap();
        // The limit cycle keeps |x| bounded near 2; the point is that it finishes.
        assert_eq!(r.end_time, 20.0);
        for row in &r.states {
            assert!(row[0].is_finite() && row[0].abs() < 4.0, "x = {}", row[0]);
        }
    }

    #[test]
    fn ode15s_handles_robertson_kinetics() {
        // Robertson: the textbook stiff chemical system.
        let rhs = |_t: f64, y: &[f64]| {
            Ok(vec![
                -0.04 * y[0] + 1.0e4 * y[1] * y[2],
                0.04 * y[0] - 1.0e4 * y[1] * y[2] - 3.0e7 * y[1] * y[1],
                3.0e7 * y[1] * y[1],
            ])
        };
        let mut p = dynamic_problem("ode15s", &rhs, vec![1.0, 0.0, 0.0], 0.0, 1.0, 11);
        p.rtol = 1e-4;
        p.atol = 1e-8;
        let r = integrate(&p).unwrap();
        for row in &r.states {
            // Mass is conserved: y1 + y2 + y3 = 1.
            let mass: f64 = row.iter().sum();
            assert_close(mass, 1.0, 1e-4, "Robertson mass balance");
        }
    }

    #[test]
    fn an_explicit_fixed_step_overrides_the_adaptive_initial_step() {
        let rhs = |_t: f64, y: &[f64]| Ok(vec![-y[0]]);
        let mut p = dynamic_problem("ode45", &rhs, vec![1.0], 0.0, 1.0, 5);
        p.fixed_step = Some(0.01);
        p.max_step = None;
        let r = integrate(&p).unwrap();
        // Only the FIRST step is 0.01; the controller takes over afterwards, so
        // the run carries the ordinary rtol = 1e-6 truncation error rather than
        // the 1e-12 a genuinely fixed 0.01 step would give.
        for i in 0..r.times.len() {
            assert_close(r.states[i][0], libm::exp(-r.times[i]), 1e-5, "ode45 seeded");
        }
    }

    #[test]
    fn max_step_bounds_the_accepted_step_growth() {
        let rhs = |_t: f64, y: &[f64]| Ok(vec![-y[0]]);
        let mut p = dynamic_problem("ode45", &rhs, vec![1.0], 0.0, 10.0, 3);
        p.max_step = Some(0.05);
        let r = integrate(&p).unwrap();
        // 10 / 0.05 = 200 steps minimum.
        assert!(
            r.accepted_steps >= 200,
            "accepted = {} with maxStep 0.05",
            r.accepted_steps
        );
    }

    // ── Sampling ────────────────────────────────────────────────────────────

    #[test]
    fn samples_are_evenly_spaced_over_the_realised_span() {
        let rhs = |_t: f64, _y: &[f64]| Ok(vec![1.0]);
        let p = dynamic_problem("ode45", &rhs, vec![0.0], 2.0, 6.0, 5);
        let r = integrate(&p).unwrap();
        assert_eq!(r.times, vec![2.0, 3.0, 4.0, 5.0, 6.0]);
        assert_eq!(r.dimension(), 1);
    }

    #[test]
    fn the_default_sample_count_is_200() {
        let rhs = |_t: f64, _y: &[f64]| Ok(vec![1.0]);
        let mut p = dynamic_problem("ode45", &rhs, vec![0.0], 0.0, 1.0, 2);
        p.points = None;
        let r = integrate(&p).unwrap();
        assert_eq!(r.times.len(), 200);
    }

    #[test]
    fn integrate_and_sample_at_honours_caller_times() {
        let rhs = |_t: f64, y: &[f64]| Ok(vec![-y[0]]);
        let p = dynamic_problem("ode45", &rhs, vec![1.0], 0.0, 1.0, 2);
        let taus = [0.0, 0.125, 0.6, 1.0];
        let states = integrate_and_sample_at(&p, &taus).unwrap();
        assert_eq!(states.len(), 4);
        for (i, &tau) in taus.iter().enumerate() {
            assert_close(states[i][0], libm::exp(-tau), 1e-7, "sampled");
        }
    }

    // ── Events ──────────────────────────────────────────────────────────────

    #[test]
    fn a_stop_event_terminates_at_the_crossing_and_shortens_the_span() {
        // Projectile: y = [h, v], h' = v, v' = -9.81. Apogee at v = 0, t = 2.
        let rhs = |_t: f64, y: &[f64]| Ok(vec![y[1], -9.81]);
        let mut p = dynamic_problem("ode45", &rhs, vec![0.0, 19.62], 0.0, 10.0, 5);
        p.events = vec![OdeEvent::new(
            "apogee",
            scalar(|_t, y: &[f64]| Ok(y[1])),
            0,
            true,
        )];
        let r = integrate(&p).unwrap();
        assert!(r.stopped);
        assert_close(r.end_time, 2.0, 1e-9, "apogee time");
        assert_eq!(r.events.len(), 1);
        assert_eq!(r.events[0].name, "apogee");
        assert_close(r.events[0].time, 2.0, 1e-9, "recorded apogee");
        assert_close(r.events[0].state[0], 19.62, 1e-7, "apogee height");
        // Samples now span [0, 2], not [0, 10].
        assert_close(*r.times.last().unwrap(), 2.0, 1e-9, "last sample");
        assert_close(r.states.last().unwrap()[0], 19.62, 1e-7, "final height");
    }

    #[test]
    fn a_record_event_is_logged_without_stopping() {
        let rhs = |_t: f64, y: &[f64]| Ok(vec![y[1], -9.81]);
        let mut p = dynamic_problem("ode45", &rhs, vec![0.0, 19.62], 0.0, 4.0, 5);
        p.events = vec![OdeEvent::new(
            "apogee",
            scalar(|_t, y: &[f64]| Ok(y[1])),
            0,
            false,
        )];
        let r = integrate(&p).unwrap();
        assert!(!r.stopped);
        assert_eq!(r.end_time, 4.0);
        assert_eq!(r.events.len(), 1);
        assert_close(r.events[0].time, 2.0, 1e-9, "recorded apogee");
    }

    #[test]
    fn direction_filters_which_crossing_fires() {
        // g = sin(t): rises through 0 at t=0 (excluded, it is the start) and
        // falls at pi, rises at 2pi.
        let rhs = |_t: f64, _y: &[f64]| Ok(vec![1.0]);
        let make = |direction: i32| {
            let mut p = dynamic_problem("ode45", &rhs, vec![0.0], 0.1, 7.0, 5);
            p.events = vec![OdeEvent::new(
                "cross",
                scalar(|t: f64, _y: &[f64]| Ok(libm::sin(t))),
                direction,
                false,
            )];
            integrate(&p).unwrap()
        };
        let falling = make(-1);
        assert_eq!(falling.events.len(), 1);
        assert_close(falling.events[0].time, core::f64::consts::PI, 1e-8, "pi");

        let rising = make(1);
        assert_eq!(rising.events.len(), 1);
        assert_close(
            rising.events[0].time,
            2.0 * core::f64::consts::PI,
            1e-8,
            "2pi",
        );

        let any = make(0);
        assert_eq!(any.events.len(), 2);
    }

    #[test]
    fn the_earliest_of_several_crossings_wins() {
        let rhs = |_t: f64, _y: &[f64]| Ok(vec![1.0]);
        let mut p = dynamic_problem("ode45", &rhs, vec![0.0], 0.0, 5.0, 3);
        p.events = vec![
            OdeEvent::new("late", scalar(|t: f64, _y: &[f64]| Ok(t - 3.0)), 0, true),
            OdeEvent::new("early", scalar(|t: f64, _y: &[f64]| Ok(t - 1.0)), 0, true),
        ];
        let r = integrate(&p).unwrap();
        assert!(r.stopped);
        assert_eq!(r.events.len(), 1);
        assert_eq!(r.events[0].name, "early");
        assert_close(r.end_time, 1.0, 1e-9, "earliest crossing");
    }

    #[test]
    fn a_set_event_reassigns_the_state_and_integration_resumes() {
        // Bouncing ball with a perfectly elastic set: h' = v, v' = -10.
        // When h crosses 0 downward, set v := -v.
        let rhs = |_t: f64, y: &[f64]| Ok(vec![y[1], -10.0]);
        let mut p = dynamic_problem("ode45", &rhs, vec![5.0, 0.0], 0.0, 3.0, 61);
        p.events = vec![OdeEvent::with_set(
            "bounce",
            scalar(|_t, y: &[f64]| Ok(y[0])),
            -1,
            false,
            1,
            scalar(|_t, y: &[f64]| Ok(-y[1])),
        )];
        let r = integrate(&p).unwrap();
        assert!(!r.stopped);
        // First contact at h = 0: 5 = 5 t^2 => t = 1.
        assert!(!r.events.is_empty());
        assert_close(r.events[0].time, 1.0, 1e-7, "first bounce");
        // The ball never goes appreciably below the floor afterwards.
        for (i, row) in r.states.iter().enumerate() {
            assert!(row[0] > -0.05, "h[{i}] = {} went through the floor", row[0]);
        }
        // And it climbs back: energy is conserved, so it returns near 5 m.
        let peak = r.states.iter().map(|s| s[0]).fold(f64::MIN, f64::max);
        assert_close(peak, 5.0, 0.05, "post-bounce apex");
    }

    #[test]
    fn a_set_target_outside_the_state_vector_is_an_error_not_a_panic() {
        let rhs = |_t: f64, _y: &[f64]| Ok(vec![1.0]);
        let mut p = dynamic_problem("ode45", &rhs, vec![0.0], 0.0, 2.0, 3);
        p.events = vec![OdeEvent::with_set(
            "bad",
            scalar(|t: f64, _y: &[f64]| Ok(t - 1.0)),
            0,
            false,
            7, // only one state exists
            scalar(|_t, _y: &[f64]| Ok(0.0)),
        )];
        let msg = format!("{}", integrate(&p).unwrap_err());
        assert!(msg.contains("EVENT bad"), "{msg}");
        assert!(msg.contains("outside the 1-state vector"), "{msg}");
    }

    #[test]
    fn a_switching_function_that_never_crosses_records_nothing() {
        let rhs = |_t: f64, _y: &[f64]| Ok(vec![1.0]);
        let mut p = dynamic_problem("ode45", &rhs, vec![0.0], 0.0, 1.0, 3);
        p.events = vec![OdeEvent::new(
            "never",
            scalar(|_t, _y: &[f64]| Ok(1.0)),
            0,
            true,
        )];
        let r = integrate(&p).unwrap();
        assert!(!r.stopped);
        assert!(r.events.is_empty());
        assert_eq!(r.end_time, 1.0);
    }

    /// The analytic short cut is invisible from outside — a fallback to
    /// bisection gives the same number — so it is asserted directly here.
    /// Without this, the whole cubic path could quietly never run.
    #[test]
    fn a_state_residual_event_takes_the_analytic_path_and_a_nonlinear_one_does_not() {
        // y[0] = 2t, y[1] = 4 − t, over one unit step. The two states are
        // deliberately not affinely dependent: if they were, state 0 would
        // also satisfy the residual identity and the detection would pick it.
        let y = [0.0, 4.0];
        let f = [2.0, -1.0];
        let y_new = [2.0, 3.0];
        let f_new = [2.0, -1.0];

        // g = y[1] − 3.5: a residual of state 1, both orientations.
        for sign in [1.0f64, -1.0] {
            let ev = OdeEvent::new(
                "level",
                scalar(move |_t, y: &[f64]| Ok(sign * (y[1] - 3.5))),
                0,
                false,
            );
            let g_prev = sign * (y[1] - 3.5);
            let g_new = sign * (y_new[1] - 3.5);
            let tc = analytic_crossing(&ev, 0.0, &y, &f, 1.0, &y_new, &f_new, g_prev, g_new)
                .unwrap()
                .expect("a state residual is analytic");
            assert_close(tc, 0.5, 1e-14, "analytic crossing");
        }

        // g = y[0]·y[1] − 2: not a residual of any single state, so no level
        // survives both knots and the caller must bisect.
        let ev = OdeEvent::new(
            "product",
            scalar(|_t, y: &[f64]| Ok(y[0] * y[1] - 2.0)),
            0,
            false,
        );
        let g_prev = y[0] * y[1] - 2.0;
        let g_new = y_new[0] * y_new[1] - 2.0;
        assert_eq!(
            analytic_crossing(&ev, 0.0, &y, &f, 1.0, &y_new, &f_new, g_prev, g_new).unwrap(),
            None
        );
        // …and bisecting it still lands on a finite time inside the step.
        let tc = refine_crossing(&ev, 0.0, &y, &f, 1.0, &y_new, &f_new, g_prev, g_new).unwrap();
        assert!(tc.is_finite() && (0.0..=1.0).contains(&tc), "bisected {tc}");
        let yc = hermite(0.0, &y, &f, 1.0, &y_new, &f_new, tc);
        assert_close(yc[0] * yc[1] - 2.0, 0.0, 1e-12, "bisected residual");
    }

    /// A switching function that only *looks* like a state residual at the two
    /// knots is caught by the mid-point check, not trusted.
    #[test]
    fn a_coincidental_residual_match_falls_back_to_bisection() {
        let y = [1.0];
        let f = [0.0];
        let y_new = [-1.0];
        let f_new = [0.0];
        // g = y[0]³: equals y[0] − 0 at both knots (1³ = 1, (−1)³ = −1) and
        // nowhere else.
        let ev = OdeEvent::new("cube", scalar(|_t, y: &[f64]| Ok(y[0].powi(3))), 0, false);
        assert_eq!(
            analytic_crossing(&ev, 0.0, &y, &f, 1.0, &y_new, &f_new, 1.0, -1.0).unwrap(),
            None
        );
    }

    #[test]
    fn an_event_evaluation_failure_propagates() {
        let rhs = |_t: f64, _y: &[f64]| Ok(vec![1.0]);
        let mut p = dynamic_problem("ode45", &rhs, vec![0.0], 0.0, 1.0, 3);
        p.events = vec![OdeEvent::new(
            "boom",
            scalar(|_t, _y: &[f64]| Err(FreesError::solver("switching function blew up"))),
            0,
            true,
        )];
        let err = integrate(&p).unwrap_err();
        assert!(format!("{err}").contains("switching function blew up"));
    }

    // ── Guards ──────────────────────────────────────────────────────────────

    #[test]
    fn a_non_increasing_span_is_rejected() {
        let rhs = |_t: f64, _y: &[f64]| Ok(vec![1.0]);
        let p = dynamic_problem("ode45", &rhs, vec![0.0], 5.0, 5.0, 3);
        let err = integrate(&p).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("t0 < tf"), "{msg}");
        assert!(msg.contains("5.0 .. 5.0"), "{msg}");

        let back = dynamic_problem("ode45", &rhs, vec![0.0], 5.0, 1.0, 3);
        assert!(format!("{}", integrate(&back).unwrap_err()).contains("t0 < tf"));
    }

    #[test]
    fn a_non_finite_initial_derivative_is_reported_with_its_time() {
        let rhs = |_t: f64, _y: &[f64]| Ok(vec![f64::NAN]);
        let p = dynamic_problem("ode45", &rhs, vec![0.0], 1.5, 2.0, 3);
        let msg = format!("{}", integrate(&p).unwrap_err());
        assert!(msg.contains("non-finite initial derivative"), "{msg}");
        assert!(msg.contains("t = 1.5"), "{msg}");
    }

    #[test]
    fn a_state_that_diverges_mid_run_is_reported() {
        // Finite at t0 but infinite once t passes 0.5.
        let rhs = |t: f64, _y: &[f64]| Ok(vec![if t > 0.5 { f64::INFINITY } else { 1.0 }]);
        let mut p = dynamic_problem("ode4", &rhs, vec![0.0], 0.0, 1.0, 3);
        p.fixed_step = Some(0.25);
        let msg = format!("{}", integrate(&p).unwrap_err());
        assert!(msg.contains("non-finite"), "{msg}");
    }

    #[test]
    fn a_rhs_failure_propagates_out_of_the_driver() {
        let rhs = |_t: f64, _y: &[f64]| Err(FreesError::solver("the algebraic block is singular"));
        let p = dynamic_problem("ode45", &rhs, vec![0.0], 0.0, 1.0, 3);
        let msg = format!("{}", integrate(&p).unwrap_err());
        assert!(msg.contains("the algebraic block is singular"), "{msg}");
    }

    #[test]
    fn step_underflow_names_the_method_and_suggests_the_stiff_pair() {
        // An RHS that always makes the embedded estimate huge forces endless
        // rejection until the step underflows.
        let rhs = |t: f64, _y: &[f64]| Ok(vec![if t == 0.0 { 0.0 } else { 1.0 / t }]);
        let mut p = dynamic_problem("ode45", &rhs, vec![0.0], 0.0, 1.0, 3);
        p.rtol = 1e-15;
        p.atol = 1e-300;
        p.max_step = None;
        let err = integrate(&p);
        // Either underflow or non-finite is a legitimate outcome here; the
        // point is that the driver terminates with a solver error rather than
        // spinning.
        assert!(err.is_err());
        let msg = format!("{}", err.unwrap_err());
        assert!(
            msg.contains("step size underflow") || msg.contains("non-finite"),
            "{msg}"
        );
    }

    #[test]
    fn the_step_guard_message_quotes_the_budget() {
        let err = guard(MAX_STEPS + 1).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("1000000 integration steps"), "{msg}");
        assert!(guard(MAX_STEPS).is_ok());
    }

    // ── Initial step heuristic ──────────────────────────────────────────────

    #[test]
    fn the_initial_step_is_clamped_into_the_span() {
        let rhs = |_t: f64, y: &[f64]| Ok(vec![-1.0e6 * y[0]]);
        let p = dynamic_problem("ode45", &rhs, vec![1.0], 0.0, 1.0, 3);
        let method = resolve_method("ode45").unwrap();
        let f0 = rhs(0.0, &p.y0).unwrap();
        let h = initial_step(&p, method.as_ref(), &p.y0, &f0, 1.0).unwrap();
        assert!(h > 0.0 && h <= 1.0, "h = {h}");
        // Fast dynamics must produce a small first step, not a fraction of span.
        assert!(h < 1e-3, "h = {h} is too large for a 1e6 rate");
    }

    #[test]
    fn the_initial_step_floors_at_a_billionth_of_the_span() {
        // A dead system (f = 0) takes the d1/d2 <= 1e-15 branch.
        let rhs = |_t: f64, _y: &[f64]| Ok(vec![0.0]);
        let p = dynamic_problem("ode45", &rhs, vec![0.0], 0.0, 1000.0, 3);
        let method = resolve_method("ode45").unwrap();
        let h = initial_step(&p, method.as_ref(), &p.y0, &[0.0], 1000.0).unwrap();
        assert!(h >= 1000.0 * 1e-9, "h = {h}");
        assert!(h <= 1000.0, "h = {h}");
    }

    #[test]
    fn compute_initial_step_prefers_the_fixed_step_and_respects_max_step() {
        let rhs = |_t: f64, y: &[f64]| Ok(vec![-y[0]]);
        let method = resolve_method("ode45").unwrap();
        let mut p = dynamic_problem("ode45", &rhs, vec![1.0], 0.0, 1.0, 3);
        p.max_step = None;
        p.fixed_step = Some(0.25);
        let h = compute_initial_step(&p, method.as_ref(), &p.y0, &[-1.0], 1.0, false, 0.5).unwrap();
        assert_eq!(h, 0.25);
        p.max_step = Some(0.1);
        let capped =
            compute_initial_step(&p, method.as_ref(), &p.y0, &[-1.0], 1.0, false, 0.5).unwrap();
        assert_eq!(capped, 0.1);
        // Fixed-step methods ignore `fixedStep` resolution and take hFixed.
        let fixed =
            compute_initial_step(&p, method.as_ref(), &p.y0, &[-1.0], 1.0, true, 0.05).unwrap();
        assert_eq!(fixed, 0.05);
    }

    #[test]
    fn a_fixed_step_method_without_an_explicit_step_uses_the_sample_spacing() {
        // span / (points - 1) = 1 / 4 = 0.25.
        let rhs = |_t: f64, _y: &[f64]| Ok(vec![1.0]);
        let mut p = dynamic_problem("ode1", &rhs, vec![0.0], 0.0, 1.0, 5);
        p.max_step = None;
        let r = integrate(&p).unwrap();
        // Euler on y' = 1 is exact, and there are exactly 4 accepted steps.
        assert_eq!(r.accepted_steps, 4);
        assert_eq!(r.rejected_steps, 0);
        for i in 0..r.times.len() {
            assert_close(r.states[i][0], r.times[i], 1e-13, "y = t");
        }
    }

    /// `maxStep` clamps a **fixed-step** method too — `run` applies
    /// `h = Math.min(h, maxStep)` after `h = fixed ? hFixed : hNext`, so the
    /// `DynamicSolver` default of `span / 100` silently overrides a larger
    /// `step =` in the header. Pinned because it is surprising.
    #[test]
    fn max_step_also_clamps_a_fixed_step_method() {
        let rhs = |_t: f64, _y: &[f64]| Ok(vec![1.0]);
        // hFixed = 1 / 4 = 0.25, but maxStep = span / 100 = 0.01 wins.
        let p = dynamic_problem("ode1", &rhs, vec![0.0], 0.0, 1.0, 5);
        assert_eq!(p.max_step, Some(0.01));
        let r = integrate(&p).unwrap();
        assert_eq!(r.accepted_steps, 100);
    }

    #[test]
    fn rejected_steps_are_counted() {
        // Tight tolerances on a fast decay force some rejections.
        let rhs = |_t: f64, y: &[f64]| Ok(vec![-50.0 * y[0]]);
        let mut p = dynamic_problem("ode23", &rhs, vec![1.0], 0.0, 1.0, 3);
        p.rtol = 1e-10;
        p.atol = 1e-14;
        p.max_step = None;
        let r = integrate(&p).unwrap();
        assert!(r.accepted_steps > 0);
        for i in 0..r.times.len() {
            assert_close(
                r.states[i][0],
                libm::exp(-50.0 * r.times[i]),
                1e-8,
                "fast decay",
            );
        }
    }

    // ── Java float semantics ────────────────────────────────────────────────

    #[test]
    fn java_min_max_propagate_nan() {
        assert!(java_min(1.0, f64::NAN).is_nan());
        assert!(java_max(1.0, f64::NAN).is_nan());
        assert_eq!(java_min(1.0, 2.0), 1.0);
        assert_eq!(java_max(1.0, 2.0), 2.0);
        assert!(java_min(0.0, -0.0).is_sign_negative());
        assert!(java_max(0.0, -0.0).is_sign_positive());
    }

    #[test]
    fn scaled_norm_is_the_rms_of_the_scaled_vector() {
        let n = scaled_norm(&[3.0, 4.0], &[1.0, 1.0]);
        assert_close(n, (25.0f64 / 2.0).sqrt(), 1e-15, "scaled norm");
    }
}

/// Trajectories checked against the **Java oracle**, not against an analytic
/// solution.
///
/// Every `ROWS` table below is the `ode_tables[0].rows` array the real engine
/// emitted for the quoted `DYNAMIC` document, produced with
/// `tools/golden-dumper/run.sh`. An analytic check only proves the port solves
/// *an* ODE correctly; these prove it reproduces the oracle's own truncation
/// error, its adaptive step sequence, and its Hermite sampling — which is what
/// parity actually means.
///
/// Each test rebuilds the problem exactly as `DynamicSolver.solve` would:
/// header options verbatim, `rtol`/`atol` defaulting to `1e-6`/`1e-9`
/// (`DynamicSystem.Options`), and `maxStep` defaulting to `span / 100`.
///
/// The documents deliberately name states `Temp` / `posx` / `yv` and the time
/// variable `time`: frees identifiers are case-insensitive, so a state `T`
/// collides with a time variable `t` and the table comes back with duplicate
/// columns.
#[cfg(test)]
mod oracle {
    use super::tests::*;
    use super::*;
    use crate::ode::problem::OdeRhs;

    /// `|got − want| <= abs_tol + rel_tol·|want|`, applied cell by cell to
    /// `[time, states…]` rows.
    fn assert_rows(r: &OdeResult, rows: &[&[f64]], rel_tol: f64, abs_tol: f64, doc: &str) {
        assert_eq!(r.times.len(), rows.len(), "{doc}: row count");
        for (i, want) in rows.iter().enumerate() {
            let mut got = vec![r.times[i]];
            got.extend_from_slice(&r.states[i]);
            assert_eq!(got.len(), want.len(), "{doc}: row {i} width");
            for (c, (&g, &w)) in got.iter().zip(want.iter()).enumerate() {
                let tol = abs_tol + rel_tol * w.abs();
                assert!(
                    (g - w).abs() <= tol,
                    "{doc}: row {i} col {c}: got {g}, oracle {w} (|Δ| = {}, tol {tol})",
                    (g - w).abs()
                );
            }
        }
    }

    /// The problem `DynamicSolver.solve` builds for a header with no `rtol`,
    /// `atol`, `step` or `maxstep` override.
    fn oracle_problem<'a>(
        method: &str,
        rhs: &'a dyn OdeRhs,
        y0: Vec<f64>,
        t0: f64,
        tf: f64,
        points: usize,
    ) -> OdeProblem<'a> {
        OdeProblem {
            method: method.to_string(),
            t0,
            tf,
            y0,
            rhs,
            points: Some(points),
            fixed_step: None,
            rtol: 1e-6,
            atol: 1e-9,
            max_step: Some((tf - t0) / 100.0),
            events: Vec::new(),
        }
    }

    // ── ode45 ───────────────────────────────────────────────────────────────

    /// ```text
    /// k    = 0.05
    /// Tinf = 20
    /// DYNAMIC cooling (method = ode45, time = 0 .. 60, points = 4)
    ///   der(Temp) = -k * (Temp - Tinf)
    ///   Temp(0)   = 95
    /// END
    /// ```
    #[test]
    fn ode45_newton_cooling_matches_the_oracle() {
        const ROWS: [&[f64]; 4] = [
            &[0.0, 95.0],
            &[20.0, 47.590_958_030_463_33],
            &[40.0, 30.150_146_238_537_44],
            &[60.0, 23.734_030_127_668_667],
        ];
        let rhs = |_t: f64, y: &[f64]| Ok(vec![-0.05 * (y[0] - 20.0)]);
        let p = oracle_problem("ode45", &rhs, vec![95.0], 0.0, 60.0, 4);
        let r = integrate(&p).unwrap();
        assert!(!r.stopped);
        assert_eq!(r.end_time, 60.0);
        assert_rows(&r, &ROWS, 1e-12, 1e-12, "ode45 Newton cooling");
    }

    /// ```text
    /// mass = 1.0
    /// kspr = 20.0
    /// damp = 0.5
    /// DYNAMIC osc (method = ode45, time = 0 .. 20, points = 9, rtol = 1e-9)
    ///   der(posx) = vel
    ///   der(vel)  = -(damp/mass) * vel - (kspr/mass) * posx
    ///   posx(0)   = 1.0
    ///   vel(0)    = 0.0
    /// END
    /// ```
    #[test]
    fn ode45_damped_oscillator_matches_the_oracle() {
        const ROWS: [&[f64]; 9] = [
            &[0.0, 1.0, 0.0],
            &[2.5, 0.059_572_376_571_802_026, 2.364_043_669_701_800_4],
            &[5.0, -0.275_886_258_743_062_44, 0.421_380_971_507_676_56],
            &[7.5, -0.066_243_346_316_189_33, -0.602_200_397_767_184_9],
            &[10.0, 0.067_235_132_804_241_94, -0.228_067_397_792_127_54],
            &[12.5, 0.030_963_420_563_056_364, 0.131_881_239_481_410_7],
            &[15.0, -0.013_744_082_772_767_627, 0.088_849_651_077_356_75],
            &[17.5, -0.011_320_989_563_147_414, -0.021_947_515_045_977_77],
            &[20.0, 0.001_919_825_875_960_991, -0.029_367_907_756_199_924],
        ];
        let rhs = |_t: f64, y: &[f64]| Ok(vec![y[1], -0.5 * y[1] - 20.0 * y[0]]);
        let mut p = oracle_problem("ode45", &rhs, vec![1.0, 0.0], 0.0, 20.0, 9);
        p.rtol = 1e-9;
        let r = integrate(&p).unwrap();
        assert_rows(&r, &ROWS, 1e-9, 1e-11, "ode45 damped oscillator");
    }

    /// ```text
    /// DYNAMIC decay (method = ode45, time = 0 .. 10, points = 6,
    ///                maxstep = 0.05, rtol = 1e-10, atol = 1e-12)
    ///   der(yv) = -yv
    ///   yv(0)   = 1
    /// END
    /// ```
    /// An explicit `maxstep` overrides the `span / 100` default, so this also
    /// pins the max-step clamp on the accepted branch.
    #[test]
    fn ode45_with_an_explicit_max_step_matches_the_oracle() {
        const ROWS: [&[f64]; 6] = [
            &[0.0, 1.0],
            &[2.0, 0.135_335_283_196_807_7],
            &[4.0, 0.018_315_638_791_648_253],
            &[6.0, 0.002_478_752_137_474_66],
            &[8.0, 0.000_335_462_622_661_889_03],
            &[10.0, 4.539_992_978_900_557e-5],
        ];
        let rhs = |_t: f64, y: &[f64]| Ok(vec![-y[0]]);
        let mut p = oracle_problem("ode45", &rhs, vec![1.0], 0.0, 10.0, 6);
        p.rtol = 1e-10;
        p.atol = 1e-12;
        p.max_step = Some(0.05);
        let r = integrate(&p).unwrap();
        assert_rows(&r, &ROWS, 1e-11, 1e-15, "ode45 maxstep");
    }

    // ── ode23 ───────────────────────────────────────────────────────────────

    /// ```text
    /// DYNAMIC forced (method = ode23, time = 0 .. 3, points = 7)
    ///   der(yv) = -2 * yv + 4
    ///   yv(0)   = 0
    /// END
    /// ```
    #[test]
    fn ode23_forced_first_order_matches_the_oracle() {
        const ROWS: [&[f64]; 7] = [
            &[0.0, 0.0],
            &[0.5, 1.264_241_880_781_124_2],
            &[1.0, 1.729_331_180_679_158_3],
            &[1.5, 1.900_427_451_901_406_8],
            &[2.0, 1.963_369_649_401_926_8],
            &[2.5, 1.986_524_574_737_698_3],
            &[3.0, 1.995_042_713_529_155_5],
        ];
        let rhs = |_t: f64, y: &[f64]| Ok(vec![-2.0 * y[0] + 4.0]);
        let p = oracle_problem("ode23", &rhs, vec![0.0], 0.0, 3.0, 7);
        let r = integrate(&p).unwrap();
        assert_rows(&r, &ROWS, 1e-12, 1e-14, "ode23 forced");
    }

    // ── Fixed-step ladder ode1..ode5 ────────────────────────────────────────

    /// `der(yv) = -yv, yv(0) = 1` on `[0, 1]` with `step = 0.05, points = 5`,
    /// once per fixed-step method. The columns diverge from `e^-t` by exactly
    /// each method's own truncation error, which is the point: a substituted
    /// integrator would land on different digits.
    ///
    /// These rows also pin a behaviour that is easy to get wrong. The header
    /// asks for `step = 0.05`, but the oracle's `ode1` column at `t = 1` is
    /// `0.36603234127322987` = `0.99^100`, not `0.95^20 = 0.3584859…` — so the
    /// engine actually stepped at **0.01**. `DynamicSolver.solve` defaults
    /// `maxStep` to `span / 100`, and `run` applies `h = Math.min(h, maxStep)`
    /// *after* `h = fixed ? hFixed : hNext`, so the cap silently overrides a
    /// larger explicit `step`.
    #[test]
    fn every_fixed_step_method_matches_the_oracle() {
        const ODE1: [&[f64]; 5] = [
            &[0.0, 1.0],
            &[0.25, 0.777_821_359_399_146_8],
            &[0.5, 0.605_006_067_137_536_8],
            &[0.75, 0.470_586_641_585_650_5],
            &[1.0, 0.366_032_341_273_229_87],
        ];
        const ODE2: [&[f64]; 5] = [
            &[0.0, 1.0],
            &[0.25, 0.778_804_052_516_401_5],
            &[0.5, 0.606_535_752_215_969_8],
            &[0.75, 0.472_372_501_821_881_23],
            &[1.0, 0.367_885_618_716_192_5],
        ];
        const ODE3: [&[f64]; 5] = [
            &[0.0, 1.0],
            &[0.25, 0.778_800_774_893_725_5],
            &[0.5, 0.606_530_646_975_067_5],
            &[0.75, 0.472_366_537_860_975_3],
            &[1.0, 0.367_879_425_719_993_8],
        ];
        const ODE5: [&[f64]; 5] = [
            &[0.0, 1.0],
            &[0.25, 0.778_800_783_071_410_7],
            &[0.5, 0.606_530_659_712_642_3],
            &[0.75, 0.472_366_552_741_025],
            &[1.0, 0.367_879_441_171_452_94],
        ];
        let rhs = |_t: f64, y: &[f64]| Ok(vec![-y[0]]);
        for (method, rows) in [
            ("ode1", &ODE1),
            ("ode2", &ODE2),
            ("ode3", &ODE3),
            ("ode5", &ODE5),
        ] {
            let mut p = oracle_problem(method, &rhs, vec![1.0], 0.0, 1.0, 5);
            p.fixed_step = Some(0.05);
            let r = integrate(&p).unwrap();
            assert_rows(&r, rows, 1e-13, 1e-15, method);
        }
    }

    /// ```text
    /// DYNAMIC fixed4 (method = ode4, time = 0 .. 1, points = 6, step = 0.01)
    ///   der(yv) = -yv
    ///   yv(0)   = 1
    /// END
    /// ```
    #[test]
    fn ode4_fixed_step_matches_the_oracle() {
        const ROWS: [&[f64]; 6] = [
            &[0.0, 1.0],
            &[0.2, 0.818_730_753_091_741_4],
            &[0.4, 0.670_320_046_058_170_5],
            &[0.6, 0.548_811_636_121_697_1],
            &[0.8, 0.449_328_964_147_427_73],
            &[1.0, 0.367_879_441_202_355_55],
        ];
        let rhs = |_t: f64, y: &[f64]| Ok(vec![-y[0]]);
        let mut p = oracle_problem("ode4", &rhs, vec![1.0], 0.0, 1.0, 6);
        p.fixed_step = Some(0.01);
        let r = integrate(&p).unwrap();
        assert_rows(&r, &ROWS, 1e-13, 1e-15, "ode4 fixed");
    }

    // ── Stiff path ──────────────────────────────────────────────────────────

    /// ```text
    /// DYNAMIC stiff23s (method = ode23s, time = 0 .. 2, points = 5)
    ///   der(yv) = -1000 * (yv - cos(time)) - sin(time)
    ///   yv(0)   = 1
    /// END
    /// ```
    /// The RHS depends on `t` explicitly, so this exercises `dfdt` as well as
    /// the Jacobian and the three Rosenbrock linear solves.
    #[test]
    fn ode23s_matches_the_oracle_on_a_two_timescale_system() {
        const ROWS: [&[f64]; 5] = [
            &[0.0, 1.0],
            &[0.5, 0.877_582_956_717_407_8],
            &[1.0, 0.540_302_634_937_358],
            &[1.5, 0.070_737_239_442_431_92],
            &[2.0, -0.416_146_982_994_148_9],
        ];
        let rhs = |t: f64, y: &[f64]| Ok(vec![-1000.0 * (y[0] - libm::cos(t)) - libm::sin(t)]);
        let p = oracle_problem("ode23s", &rhs, vec![1.0], 0.0, 2.0, 5);
        let r = integrate(&p).unwrap();
        assert_rows(&r, &ROWS, 1e-8, 1e-10, "ode23s two-timescale");
    }

    /// The same document with `method = ode15s`. The two methods land on
    /// visibly different digits — reproducing *both* is what proves each
    /// stepper was ported rather than aliased onto one implementation.
    #[test]
    fn ode15s_matches_the_oracle_on_a_two_timescale_system() {
        const ROWS: [&[f64]; 5] = [
            &[0.0, 1.0],
            &[0.5, 0.877_582_456_731_375_7],
            &[1.0, 0.540_302_229_222_848_8],
            &[1.5, 0.070_737_188_779_700_41],
            &[2.0, -0.416_146_787_893_553_1],
        ];
        let rhs = |t: f64, y: &[f64]| Ok(vec![-1000.0 * (y[0] - libm::cos(t)) - libm::sin(t)]);
        let p = oracle_problem("ode15s", &rhs, vec![1.0], 0.0, 2.0, 5);
        let r = integrate(&p).unwrap();
        assert_rows(&r, &ROWS, 1e-8, 1e-10, "ode15s two-timescale");
        // The two stiff steppers must NOT agree to their last digits — if they
        // did, one of them is not the method it claims to be.
        assert!((0.877_582_456_731_375_7f64 - 0.877_582_956_717_407_8).abs() > 1e-10);
    }

    /// ```text
    /// DYNAMIC robert (method = ode15s, time = 0 .. 1, points = 5,
    ///                 rtol = 1e-4, atol = 1e-8)
    ///   der(ya) = -0.04*ya + 1e4*yb*yc
    ///   der(yb) = 0.04*ya - 1e4*yb*yc - 3e7*yb*yb
    ///   der(yc) = 3e7*yb*yb
    ///   ya(0) = 1 / yb(0) = 0 / yc(0) = 0
    /// END
    /// ```
    #[test]
    fn ode15s_robertson_kinetics_matches_the_oracle() {
        const ROWS: [&[f64]; 5] = [
            &[0.0, 1.0, 0.0, 0.0],
            &[
                0.25,
                0.990_473_115_242_114_5,
                3.479_584_710_584_067e-5,
                0.009_492_088_910_780_423,
            ],
            &[
                0.5,
                0.981_791_816_630_600_6,
                3.328_091_818_508_209e-5,
                0.018_174_902_451_213_937,
            ],
            &[
                0.75,
                0.973_822_466_934_300_3,
                3.194_098_273_920_971e-5,
                0.026_145_592_082_959_97,
            ],
            &[
                1.0,
                0.966_459_802_960_852_9,
                3.074_627_628_085_128_5e-5,
                0.033_509_450_762_865_23,
            ],
        ];
        let rhs = |_t: f64, y: &[f64]| {
            Ok(vec![
                -0.04 * y[0] + 1.0e4 * y[1] * y[2],
                0.04 * y[0] - 1.0e4 * y[1] * y[2] - 3.0e7 * y[1] * y[1],
                3.0e7 * y[1] * y[1],
            ])
        };
        let mut p = oracle_problem("ode15s", &rhs, vec![1.0, 0.0, 0.0], 0.0, 1.0, 5);
        p.rtol = 1e-4;
        p.atol = 1e-8;
        let r = integrate(&p).unwrap();
        assert_rows(&r, &ROWS, 1e-6, 1e-12, "ode15s Robertson");
    }

    /// ```text
    /// DYNAMIC vdp (method = ode23s, time = 0 .. 20, points = 6)
    ///   der(posx) = vel
    ///   der(vel)  = 100 * (1 - posx*posx) * vel - posx
    ///   posx(0)   = 2 / vel(0) = 0
    /// END
    /// ```
    /// Van der Pol at `μ = 100` on the slow manifold — the case the Rosenbrock
    /// pair exists for.
    #[test]
    fn ode23s_van_der_pol_matches_the_oracle() {
        const ROWS: [&[f64]; 6] = [
            &[0.0, 2.0, 0.0],
            &[4.0, 1.973_052_963_826_915_2, -0.006_820_105_268_672_041],
            &[8.0, 1.945_445_276_684_853_1, -0.006_985_898_837_580_852],
            &[12.0, 1.917_147_390_239_408_7, -0.007_165_519_101_026_667],
            &[16.0, 1.888_099_977_900_643, -0.007_361_041_278_912_325],
            &[20.0, 1.858_234_502_902_325_3, -0.007_575_016_515_824_413],
        ];
        let rhs = |_t: f64, y: &[f64]| Ok(vec![y[1], 100.0 * (1.0 - y[0] * y[0]) * y[1] - y[0]]);
        let p = oracle_problem("ode23s", &rhs, vec![2.0, 0.0], 0.0, 20.0, 6);
        let r = integrate(&p).unwrap();
        assert_rows(&r, &ROWS, 1e-6, 1e-9, "ode23s Van der Pol");
    }

    // ── Events ──────────────────────────────────────────────────────────────

    /// ```text
    /// DYNAMIC shot (method = ode45, time = 0 .. 10, points = 5)
    ///   der(hgt) = vel / der(vel) = -9.81
    ///   hgt(0) = 0 / vel(0) = 19.62
    ///   EVENT apogee: vel = 0 -> stop
    /// END
    /// ```
    /// The stop event shortens the sampling window from `[0, 10]` to
    /// `[0, endTime]`, so every row moves — a driver that recorded the event
    /// but kept sampling to `tf` would fail on row 1, not just the last one.
    #[test]
    fn a_stop_event_matches_the_oracle_including_the_reshaped_sample_grid() {
        const ROWS: [&[f64]; 5] = [
            &[0.0, 0.0, 19.62],
            &[0.5, 8.583_750_000_000_002, 14.714_999_999_999_996],
            &[1.0, 14.714_999_999_999_998, 9.809_999_999_999_993],
            &[1.5, 18.393_749_999_999_986, 4.904_999_999_999_997],
            &[2.0, 19.619_999_999_999_98, -5.828_670_879_282_072e-16],
        ];
        let rhs = |_t: f64, y: &[f64]| Ok(vec![y[1], -9.81]);
        let mut p = oracle_problem("ode45", &rhs, vec![0.0, 19.62], 0.0, 10.0, 5);
        p.events = vec![OdeEvent::new(
            "apogee",
            scalar(|_t, y: &[f64]| Ok(y[1])),
            0,
            true,
        )];
        let r = integrate(&p).unwrap();
        assert!(r.stopped, "the oracle reports stopped = true");
        assert_close(r.end_time, 2.0, 1e-12, "oracle end_time");
        assert_eq!(r.events.len(), 1);
        assert_eq!(r.events[0].name, "apogee");
        assert_close(r.events[0].time, 2.0, 1e-12, "oracle event time");
        assert_rows(&r, &ROWS, 1e-11, 1e-13, "ode45 stop event");
    }

    /// The same document with `-> record` and `time = 0 .. 4`: the event is
    /// logged, the run continues to `tf`, and `stopped` stays false.
    #[test]
    fn a_record_event_matches_the_oracle_and_does_not_shorten_the_run() {
        const ROWS: [&[f64]; 5] = [
            &[0.0, 0.0, 19.62],
            &[1.0, 14.714_999_999_999_986, 9.809_999_999_999_985],
            &[2.0, 19.619_999_999_999_955, -2.068_559_489_280_461e-15],
            &[3.0, 14.714_999_999_999_952, -9.809_999_999_999_99],
            &[4.0, -2.238_888_328_204_291_5e-14, -19.620_000_000_000_008],
        ];
        let rhs = |_t: f64, y: &[f64]| Ok(vec![y[1], -9.81]);
        let mut p = oracle_problem("ode45", &rhs, vec![0.0, 19.62], 0.0, 4.0, 5);
        p.events = vec![OdeEvent::new(
            "apogee",
            scalar(|_t, y: &[f64]| Ok(y[1])),
            0,
            false,
        )];
        let r = integrate(&p).unwrap();
        assert!(!r.stopped);
        assert_eq!(r.end_time, 4.0);
        assert_eq!(r.events.len(), 1);
        assert_close(r.events[0].time, 2.0, 1e-11, "oracle event time");
        assert_rows(&r, &ROWS, 1e-10, 1e-12, "ode45 record event");
    }

    /// ```text
    /// kc   = 0.05
    /// Tinf = 20
    /// DYNAMIC cooling (method = ode45, time = 0 .. 60, points = 4)
    ///   der(Temp) = -kc * (Temp - Tinf)
    ///   Temp(0)   = 95
    ///   EVENT warm: Temp = 30 | falling -> record
    /// END
    /// ```
    /// The crossing time is the sharpest available test of the dense output:
    /// the true root of `20 + 75·e^(−0.05 t) = 30` is `ln(7.5)/0.05 =
    /// 40.298060410845294`, but the oracle reports `40.29806037473327` — it is
    /// off by 3.6e-8 because the bisection refines on the **cubic Hermite
    /// interpolant between knots**, not on the exact solution. Matching the
    /// oracle here therefore pins `hermite`, `refine_crossing`'s 60 bisections,
    /// and the accepted-step knot spacing all at once; matching the analytic
    /// root instead would mean the interpolation was wrong.
    #[test]
    fn an_event_crossing_time_matches_the_oracle_not_the_analytic_root() {
        let rhs = |_t: f64, y: &[f64]| Ok(vec![-0.05 * (y[0] - 20.0)]);
        let mut p = oracle_problem("ode45", &rhs, vec![95.0], 0.0, 60.0, 4);
        p.events = vec![OdeEvent::new(
            "warm",
            scalar(|_t, y: &[f64]| Ok(y[0] - 30.0)),
            -1,
            false,
        )];
        let r = integrate(&p).unwrap();
        assert!(!r.stopped);
        assert_eq!(r.events.len(), 1);
        assert_eq!(r.events[0].name, "warm");
        assert_close(
            r.events[0].time,
            40.298_060_374_733_27,
            1e-9,
            "oracle crossing time",
        );
        // The analytic root is a *different* number, ~3.6e-8 away. If this
        // assertion ever starts failing, the interpolant changed.
        let analytic = libm::log(7.5) / 0.05;
        assert!(
            (r.events[0].time - analytic).abs() > 1e-9,
            "the oracle's crossing carries interpolation error; got {} vs analytic {analytic}",
            r.events[0].time
        );
        // The recorded state at the crossing is the interpolated one.
        assert_close(r.events[0].state[0], 30.0, 1e-9, "state at the crossing");
    }

    /// ```text
    /// DYNAMIC ball (method = ode45, time = 0 .. 3, points = 13)
    ///   der(hgt) = vel / der(vel) = -10
    ///   hgt(0) = 5 / vel(0) = 0
    ///   EVENT bounce: hgt = 0 | falling -> set vel = -vel
    /// END
    /// ```
    /// A `set` event fires twice: the direction guard must re-arm against the
    /// post-set state without retriggering at the same instant, and the knot at
    /// the crossing must carry the POST-set velocity (row `t = 1.0` shows
    /// `vel = +10`, not `−10`).
    #[test]
    fn a_set_event_matches_the_oracle_across_two_bounces() {
        const ROWS: [&[f64]; 13] = [
            &[0.0, 5.0, 0.0],
            &[0.25, 4.687_499_999_999_998, -2.5],
            &[0.5, 3.749_999_999_999_999_6, -4.999_999_999_999_997],
            &[0.75, 2.187_500_000_000_005, -7.499_999_999_999_993],
            &[1.0, -9.857_917_987_875_52e-15, 10.000_000_000_000_014],
            &[1.25, 2.187_499_999_999_990_7, 7.500_000_000_000_011_5],
            &[1.5, 3.749_999_999_999_991_6, 5.000_000_000_000_015],
            &[1.75, 4.687_499_999_999_996_4, 2.500_000_000_000_019],
            &[2.0, 4.999_999_999_999_996_4, 2.135_097_654_232_254_2e-14],
            &[2.25, 4.687_499_999_999_996, -2.499_999_999_999_993],
            &[2.5, 3.749_999_999_999_989, -5.000_000_000_000_008],
            &[2.75, 2.187_499_999_999_975_6, -7.500_000_000_000_022],
            &[3.0, 4.368_727_601_901_003e-14, 9.999_999_999_999_954],
        ];
        let rhs = |_t: f64, y: &[f64]| Ok(vec![y[1], -10.0]);
        let mut p = oracle_problem("ode45", &rhs, vec![5.0, 0.0], 0.0, 3.0, 13);
        p.events = vec![OdeEvent::with_set(
            "bounce",
            scalar(|_t, y: &[f64]| Ok(y[0])),
            -1,
            false,
            1,
            scalar(|_t, y: &[f64]| Ok(-y[1])),
        )];
        let r = integrate(&p).unwrap();
        assert!(!r.stopped);
        assert_eq!(r.end_time, 3.0);
        assert_eq!(r.events.len(), 2, "the oracle records two bounces");
        assert_close(r.events[0].time, 1.000_000_000_000_000_9, 1e-9, "bounce 1");
        assert_close(r.events[1].time, 2.999_999_999_999_995_6, 1e-9, "bounce 2");
        // Positions near zero are compared absolutely — the oracle's own value
        // there is ~1e-14, i.e. pure round-off, and a relative test would be
        // meaningless.
        assert_rows(&r, &ROWS, 1e-9, 1e-9, "ode45 set event");
    }
}
