//! Monte Carlo uncertainty propagation — the sampling extension of the
//! first-order/RSS engine, for systems too nonlinear for a linearization.
//!
//! Port of
//! `../frEES/backend/core/src/main/java/com/frees/backend/api/MonteCarlo.java`
//! (153 LOC), together with the two pieces of `api/SolverApiSupport` it leans
//! on ([`apply_overrides`], and the `BigDecimal.valueOf(x).toPlainString()`
//! rendering of a drawn value) and the `java.util.Random` draw sequence
//! ([`JavaRandom`]).
//!
//! # Semantics, unchanged from the Java
//!
//! The uncertainty sources are the variables with a declared
//! `uncertainty > 0` that the base solve actually produced. Each source's
//! centre is its base-solve value; every sample redraws **all** sources
//! (independent normals, clamped into the declared bounds), replaces their
//! defining assignments through the terminal's own override mechanism —
//! *replace, never append* — and re-solves warm-started from the previous
//! sample. Per-sample specs carry no uncertainties, so no sample pays for its
//! own first-order propagation.
//!
//! Budget honesty: when the caller's budget predicate fires mid-run the loop
//! stops and returns what it has with `truncated = true`. Samples are i.i.d.,
//! so an early stop shrinks the sample count without biasing the statistics.
//!
//! # This is *not* a "stochastic, therefore unreproducible" port
//!
//! `java.util.Random` is a fully specified 48-bit LCG and `nextGaussian` is the
//! Marsaglia polar method with a cached second deviate. Both are transcribed
//! here, so a given `(seed, sample_count)` produces the **same draw sequence**
//! as the Java engine — verified against it, see the tests. The one place bit
//! equality is not guaranteed is `nextGaussian`'s `log`: Java uses
//! `StrictMath.log` (fdlibm) and this uses `libm::log`, which agree to within
//! an ulp rather than exactly.
//!
//! # Divergences from the Java, stated plainly
//!
//! * The Java re-solves each sample with `solvePermissive`, which blocks with
//!   `blockPermissive` and tolerates a structurally imperfect system. This port
//!   has no permissive entry point, so samples go through the strict
//!   [`crate::engine::solve_with`]; a document Java would sample permissively
//!   is refused here instead.
//! * Warm starting is expressed as a guess on each [`VariableOverride`], since
//!   `solve_with` takes no warm-start map. That differs in one corner: an
//!   in-text `GUESS` directive wins over an override in this port, whereas the
//!   Java warm-start map wins over both.
//! * `firstOrderSigma` is the base solve's RSS uncertainty. `Solution` carries
//!   no uncertainties yet, so it is a **parameter** here rather than something
//!   read back off the base result — pass what
//!   [`crate::analysis::uncertainty::propagate`] returned, or an empty map for
//!   the Java's `getOrDefault(variable, 0.0)` behaviour.

use std::collections::BTreeMap;

use crate::analysis::uncertainty::UncertaintySpec;
use crate::diag::{FreesError, Result};
use crate::engine::{solve_with_tables, VariableOverride};
use crate::parser::defs::FunctionTableDef;
use crate::solver::SolverSettings;

/// How many samples [`run`] pre-reserves room for, however many are requested.
///
/// The Java `SolveController` refuses a request outside `[2, 1000]`
/// (`frees.solver.max-mc-samples`), so no in-range run ever exceeds this and
/// the reservation stays exact. See [`run`] for why the raw count is not used.
const MAX_PREALLOCATED_SAMPLES: usize = 1_000;

/// Aggregate statistics for one variable across the successful samples. Port of
/// `MonteCarlo.VariableStats`; `first_order_sigma` is the base solve's RSS
/// value, carried for side-by-side comparison.
#[derive(Debug, Clone, PartialEq)]
pub struct VariableStats {
    pub variable: String,
    pub mean: f64,
    pub sigma: f64,
    pub p5: f64,
    pub p50: f64,
    pub p95: f64,
    pub first_order_sigma: f64,
}

/// One sample: the solved values, or the failure that discarded it. Port of
/// `MonteCarlo.Sample`.
#[derive(Debug, Clone, PartialEq)]
pub struct Sample {
    pub success: bool,
    pub values: BTreeMap<String, f64>,
    pub error: Option<String>,
}

/// Port of `MonteCarlo.Outcome`.
#[derive(Debug, Clone, PartialEq)]
pub struct Outcome {
    pub stats: Vec<VariableStats>,
    pub samples: Vec<Sample>,
    pub sources: Vec<String>,
    pub failed_samples: usize,
    pub truncated: bool,
    pub base_values: BTreeMap<String, f64>,
}

/// Runs the sampling loop. Port of `MonteCarlo.run`.
///
/// * `text` — the document exactly as the solve endpoints receive it.
/// * `specs` — per-variable guesses, bounds and **declared uncertainties**
///   (already converted to SI by the boundary, as the Java DTO layer does).
/// * `first_order_sigma` — see the module docs.
/// * `expired` — the wall-clock budget. Core has no clock on
///   `wasm32-unknown-unknown` (see [`crate::engine::SolveStats::elapsed_ms`]),
///   so the deadline is a predicate the boundary supplies. Pass
///   `|| false` for an unbounded run.
///
/// # Errors
///
/// * whatever the base solve raises — a Monte Carlo run over a document that
///   does not solve is meaningless, and the Java lets the exception out too;
/// * [`FreesError::Solver`] when no variable declares an uncertainty.
///
/// Sample failures are *not* errors: they are recorded on the sample and
/// counted in [`Outcome::failed_samples`].
pub fn run<F>(
    text: &str,
    settings: &SolverSettings,
    specs: &BTreeMap<String, UncertaintySpec>,
    first_order_sigma: &BTreeMap<String, f64>,
    sample_count: usize,
    seed: i64,
    expired: F,
) -> Result<Outcome>
where
    F: FnMut() -> bool,
{
    run_with_tables(
        text,
        settings,
        specs,
        first_order_sigma,
        sample_count,
        seed,
        expired,
        &[],
    )
}

/// [`run`] with externally supplied Function Table definitions — the request's
/// `functionTables`, which the Java `SolveController.computeMonteCarlo`
/// converts once and `MonteCarlo.run` threads into the base solve *and* every
/// per-sample `solvePermissive`. An empty slice is byte-for-byte [`run`].
#[allow(clippy::too_many_arguments)]
pub fn run_with_tables<F>(
    text: &str,
    settings: &SolverSettings,
    specs: &BTreeMap<String, UncertaintySpec>,
    first_order_sigma: &BTreeMap<String, f64>,
    sample_count: usize,
    seed: i64,
    expired: F,
    extra_tables: &[FunctionTableDef],
) -> Result<Outcome>
where
    F: FnMut() -> bool,
{
    run_with_tables_and_base(
        text,
        settings,
        specs,
        first_order_sigma,
        sample_count,
        seed,
        expired,
        extra_tables,
        None,
    )
}

/// [`run_with_tables`] with an optional precomputed base solve's variable values.
///
/// When `precomputed_base` is `Some`, the redundant base solve is skipped.
#[allow(clippy::too_many_arguments)]
pub fn run_with_tables_and_base<F>(
    text: &str,
    settings: &SolverSettings,
    specs: &BTreeMap<String, UncertaintySpec>,
    first_order_sigma: &BTreeMap<String, f64>,
    sample_count: usize,
    seed: i64,
    mut expired: F,
    extra_tables: &[FunctionTableDef],
    precomputed_base: Option<&BTreeMap<String, f64>>,
) -> Result<Outcome>
where
    F: FnMut() -> bool,
{
    let base_values: BTreeMap<String, f64> = match precomputed_base {
        Some(values) => values.clone(),
        None => {
            let base = solve_with_tables(text, settings, &overrides_from(specs), extra_tables)
                .map_err(|e| e.error)?;
            base.values
        }
    };

    // Sorted by construction: `specs` is a BTreeMap, and the Java sorts too.
    let sources: Vec<String> = specs
        .iter()
        .filter(|(name, spec)| spec.uncertainty > 0.0 && base_values.contains_key(*name))
        .map(|(name, _)| name.clone())
        .collect();
    if sources.is_empty() {
        return Err(FreesError::solver(
            "Monte Carlo needs at least one variable with a declared uncertainty \
             (set one in the Variable Information window).",
        ));
    }

    // Per-sample specs: same guesses and bounds, no uncertainties. Dropping the
    // uncertainty is automatic here — `VariableOverride` has no such field.
    let sample_specs = overrides_from(specs);

    let mut random = JavaRandom::new(seed);
    // `new ArrayList<>(sampleCount)` in the Java — but `sampleCount` is
    // untrusted here in a way it never is there. `SolveController` clamps the
    // request to `[2, frees.solver.max-mc-samples]` (default **1000**) and
    // rejects anything outside *before* `MonteCarlo.run` is called; this port
    // has no controller, and `run` is a public library entry point. Reserving
    // the raw count meant `samples = 1e9` allocated 56 GB up front — measured,
    // and an abort rather than a `Result`, because the `panic = "abort"` wasm
    // profile cannot unwind an allocation failure. Reserving the Java
    // controller's own ceiling keeps every in-range request pre-sized exactly
    // and lets an out-of-range one grow amortised until `expired` stops it.
    let mut samples: Vec<Sample> = Vec::with_capacity(sample_count.min(MAX_PREALLOCATED_SAMPLES));
    let mut warm = base_values.clone();
    let mut failed = 0usize;
    let mut truncated = false;

    for _ in 0..sample_count {
        if expired() {
            truncated = true;
            break;
        }
        let mut overrides = Vec::with_capacity(sources.len());
        for v in &sources {
            let spec = &specs[v];
            let draw = base_values[v] + spec.uncertainty * random.next_gaussian();
            let draw = clamp(draw, spec.lower, spec.upper);
            overrides.push(format!("{v} = {}", to_plain_string(draw)));
        }
        // Warm start: the previous sample's values ride in as guesses.
        let mut info = sample_specs.clone();
        apply_warm_start(&mut info, &warm);

        match solve_with_tables(
            &apply_overrides(text, &overrides),
            settings,
            &info,
            extra_tables,
        ) {
            Ok(solution) => {
                warm.clone_from(&solution.values);
                samples.push(Sample {
                    success: true,
                    values: solution.values,
                    error: None,
                });
            }
            Err(failure) => {
                failed += 1;
                samples.push(Sample {
                    success: false,
                    values: BTreeMap::new(),
                    error: Some(failure.to_string_message()),
                });
            }
        }
    }

    let stats = aggregate(&base_values, &samples, first_order_sigma);
    Ok(Outcome {
        stats,
        samples,
        sources,
        failed_samples: failed,
        truncated,
        base_values,
    })
}

/// Port of `MonteCarlo.aggregate`. Variables with fewer than two usable samples
/// are skipped — a single point has no spread to report.
fn aggregate(
    base_values: &BTreeMap<String, f64>,
    samples: &[Sample],
    first_order_sigma: &BTreeMap<String, f64>,
) -> Vec<VariableStats> {
    let mut stats = Vec::new();
    for variable in base_values.keys() {
        let mut values: Vec<f64> = samples
            .iter()
            .filter(|s| s.success)
            .filter_map(|s| s.values.get(variable).copied())
            .filter(|v| v.is_finite())
            .collect();
        if values.len() < 2 {
            continue;
        }
        values.sort_by(f64::total_cmp);
        let n = values.len() as f64;
        let mean = values.iter().sum::<f64>() / n;
        let ss: f64 = values.iter().map(|v| (v - mean) * (v - mean)).sum();
        let sigma = (ss / (n - 1.0)).sqrt();
        stats.push(VariableStats {
            variable: variable.clone(),
            mean,
            sigma,
            p5: percentile(&values, 0.05),
            p50: percentile(&values, 0.50),
            p95: percentile(&values, 0.95),
            first_order_sigma: first_order_sigma.get(variable).copied().unwrap_or(0.0),
        });
    }
    stats
}

/// Linear-interpolated percentile on a sorted slice. Port of
/// `MonteCarlo.percentile`.
fn percentile(sorted: &[f64], q: f64) -> f64 {
    let rank = q * (sorted.len() - 1) as f64;
    let lo = rank.floor() as usize;
    let hi = rank.ceil() as usize;
    if lo == hi {
        return sorted[lo];
    }
    sorted[lo] + (rank - lo as f64) * (sorted[hi] - sorted[lo])
}

/// `Math.clamp(value, min, max)` — Java 21 semantics: `min(max(value, min), max)`,
/// which leaves a NaN value NaN. Rust's `f64::clamp` panics when `min > max`;
/// the specs this is called with always have `lower <= upper` (defaults are
/// `±∞`), but the explicit form is used so a malformed spec degrades instead of
/// aborting the sample loop.
fn clamp(value: f64, min: f64, max: f64) -> f64 {
    value.max(min).min(max)
}

/// Specs → the engine's external variable information. The uncertainty is
/// deliberately dropped even though [`VariableOverride`] now carries one: that
/// is the Java's per-sample `new VariableSpec(name, guess, lower, upper)`
/// stripping — a Monte Carlo sample has already consumed the sigma by *drawing*
/// with it, and re-declaring it would run the analytic propagation on top of
/// the sampling.
fn overrides_from(specs: &BTreeMap<String, UncertaintySpec>) -> Vec<VariableOverride> {
    specs
        .iter()
        .map(|(name, spec)| VariableOverride {
            name: name.clone(),
            guess: Some(spec.guess),
            lower: spec.lower.is_finite().then_some(spec.lower),
            upper: spec.upper.is_finite().then_some(spec.upper),
            unit: None,
            uncertainty: None,
        })
        .collect()
}

/// Folds a previous solution in as the guess of every variable it names — the
/// port's stand-in for the Java `warmStart` map (see the module docs).
///
/// A warm value is clamped into the variable's declared bounds before it
/// becomes a guess: `solve_with` rejects a guess outside its own bounds, and a
/// warm start must never be the reason a sample fails. The Java reaches the
/// same place through `withTextGuesses`' `Math.clamp` and
/// `checkAndAdjustGuesses`.
fn apply_warm_start(info: &mut Vec<VariableOverride>, warm: &BTreeMap<String, f64>) {
    let mut named: Vec<String> = Vec::with_capacity(info.len());
    for entry in info.iter_mut() {
        named.push(entry.name.clone());
        if let Some(value) = warm.get(&entry.name) {
            let lower = entry.lower.unwrap_or(f64::NEG_INFINITY);
            let upper = entry.upper.unwrap_or(f64::INFINITY);
            entry.guess = Some(clamp(*value, lower, upper));
        }
    }
    for (name, value) in warm {
        if !named.iter().any(|n| n == name) {
            info.push(VariableOverride {
                name: name.clone(),
                guess: Some(*value),
                ..VariableOverride::default()
            });
        }
    }
}

// ---------------------------------------------------------------------------
// SolverApiSupport.applyOverrides
// ---------------------------------------------------------------------------

/// Replaces each overridden variable's defining assignment in the document
/// text. Port of `api/SolverApiSupport.applyOverrides` — the *terminal's* own
/// override mechanism, and the reason Monte Carlo and parameter estimation can
/// resample a document without re-parsing it into an AST first.
///
/// It lives in this module because Monte Carlo is its first user in the crate;
/// [`crate::analysis::paramfit`] calls it too.
///
/// Every `;`-separated segment of every line that reads `<name> = …` for an
/// overridden name is **deleted**, and the override lines are appended at the
/// end. Replace, never append: leaving the original assignment in place would
/// make the document overdetermined.
pub fn apply_overrides(clean_text: &str, overrides: &[String]) -> String {
    if overrides.is_empty() {
        return clean_text.to_string();
    }
    // `LinkedHashMap`: last write per name wins, first insertion order kept.
    let mut names: Vec<String> = Vec::new();
    let mut by_name: BTreeMap<String, String> = BTreeMap::new();
    for ov in overrides {
        let Some(eq) = ov.find('=') else {
            continue;
        };
        if eq == 0 {
            continue;
        }
        let name = ov[..eq].trim().to_ascii_lowercase();
        if name.is_empty() {
            continue;
        }
        if !by_name.contains_key(&name) {
            names.push(name.clone());
        }
        by_name.insert(name, ov.trim().to_string());
    }
    if by_name.is_empty() {
        return clean_text.to_string();
    }

    let mut out = String::with_capacity(clean_text.len() + 64);
    // `cleanText.split("\n", -1)` — trailing empty lines are kept.
    for line in clean_text.split('\n') {
        let kept: Vec<&str> = java_split_semicolons(line)
            .into_iter()
            .filter(|seg| !names.iter().any(|n| is_assignment_to(seg, n)))
            .collect();
        out.push_str(&kept.join(";"));
        out.push('\n');
    }
    for name in &names {
        out.push_str(&by_name[name]);
        out.push('\n');
    }
    out
}

/// The Java regex `^\s*<quoted name>\s*=.*` under `Matcher.matches()` and
/// `CASE_INSENSITIVE`. A segment never contains a newline (lines are split
/// first), so `.*` always covers the remainder and a full match reduces to this
/// prefix test.
fn is_assignment_to(segment: &str, lowercase_name: &str) -> bool {
    let rest = segment.trim_start_matches(is_java_space);
    let Some(after) = rest.get(..lowercase_name.len()) else {
        return false;
    };
    if !after.eq_ignore_ascii_case(lowercase_name) {
        return false;
    }
    let tail = &rest[lowercase_name.len()..];
    tail.trim_start_matches(is_java_space).starts_with('=')
}

/// `\s` in `java.util.regex` without `UNICODE_CHARACTER_CLASS`:
/// `[ \t\n\x0B\f\r]`.
fn is_java_space(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\u{0B}' | '\u{0C}' | '\r')
}

/// `String.split(";")` with the default limit: **all** trailing empty strings
/// are removed, so `"a=1;"` yields one segment and `";"` yields none.
fn java_split_semicolons(line: &str) -> Vec<&str> {
    let mut parts: Vec<&str> = line.split(';').collect();
    while parts.last().is_some_and(|s| s.is_empty()) {
        parts.pop();
    }
    parts
}

/// `BigDecimal.valueOf(value).toPlainString()` — a decimal rendering with no
/// exponent, so the override line is always a literal the parser accepts.
///
/// `BigDecimal.valueOf(double)` goes through `Double.toString`, which on Java 19
/// and later is the shortest representation that round-trips; Rust's `Display`
/// for `f64` is the same shortest-round-trip rendering and never uses an
/// exponent, so the two agree on the value even where they differ on trailing
/// zeros (`"2"` here versus `"2.0"` there — the same double either way).
fn to_plain_string(value: f64) -> String {
    format!("{value}")
}

// ---------------------------------------------------------------------------
// java.util.Random
// ---------------------------------------------------------------------------

/// `java.util.Random`, transcribed from its specification: a 48-bit linear
/// congruential generator, and `nextGaussian`'s Marsaglia polar method with the
/// cached second deviate.
///
/// The specification is normative (the `java.util.Random` class documentation
/// gives the exact algorithms), which is what makes a seeded Monte Carlo run
/// reproducible across the two engines rather than merely
/// statistically-equivalent.
pub struct JavaRandom {
    seed: u64,
    next_next_gaussian: Option<f64>,
}

/// `0x5DEECE66DL`, the LCG multiplier.
const MULTIPLIER: u64 = 0x0005_DEEC_E66D;
/// `0xBL`, the LCG addend.
const ADDEND: u64 = 0xB;
/// `(1L << 48) - 1`.
const MASK: u64 = (1 << 48) - 1;

impl JavaRandom {
    /// `new Random(seed)`: the seed is scrambled with the multiplier and masked
    /// to 48 bits.
    pub fn new(seed: i64) -> JavaRandom {
        JavaRandom {
            seed: ((seed as u64) ^ MULTIPLIER) & MASK,
            next_next_gaussian: None,
        }
    }

    /// `protected int next(int bits)`.
    fn next(&mut self, bits: u32) -> u64 {
        self.seed = self.seed.wrapping_mul(MULTIPLIER).wrapping_add(ADDEND) & MASK;
        self.seed >> (48 - bits)
    }

    /// `nextDouble()` = `(((long) next(26) << 27) + next(27)) * 0x1.0p-53`.
    pub fn next_double(&mut self) -> f64 {
        let hi = self.next(26) << 27;
        let lo = self.next(27);
        (hi + lo) as f64 * (1.0 / (1u64 << 53) as f64)
    }

    /// `nextGaussian()` — Marsaglia polar, returning the first deviate and
    /// caching the second.
    ///
    /// `libm::log` stands in for `StrictMath.log`; they agree to within an ulp,
    /// which is the one place this generator is not bit-for-bit Java.
    pub fn next_gaussian(&mut self) -> f64 {
        if let Some(cached) = self.next_next_gaussian.take() {
            return cached;
        }
        loop {
            let v1 = 2.0 * self.next_double() - 1.0;
            let v2 = 2.0 * self.next_double() - 1.0;
            let s = v1 * v1 + v2 * v2;
            if s < 1.0 && s != 0.0 {
                let multiplier = (-2.0 * libm::log(s) / s).sqrt();
                self.next_next_gaussian = Some(v2 * multiplier);
                return v1 * multiplier;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(actual: f64, expected: f64, tol: f64) {
        assert!(
            (actual - expected).abs() <= tol * expected.abs().max(1.0),
            "expected {expected}, got {actual}"
        );
    }

    // -- java.util.Random, against the JVM -------------------------------

    #[test]
    fn oracle_gaussian_draw_sequence_matches_java() {
        // `new Random(42).nextGaussian()` x 8, printed by a JVM probe.
        let expected = [
            1.1419053154730547,
            0.9194079489827879,
            -0.9498666368908959,
            -1.1069902863993377,
            0.2809776380727795,
            0.6846227956326554,
            -0.8172214073987268,
            -1.3966434026780434,
        ];
        let mut r = JavaRandom::new(42);
        for (i, want) in expected.iter().enumerate() {
            close(r.next_gaussian(), *want, 1e-15);
            assert!(i < 8);
        }

        let expected0: [f64; 4] = [
            0.8025330637390305,
            -0.9015460884175122,
            2.080920790428163,
            0.7637707684364894,
        ];
        let mut r = JavaRandom::new(0);
        for want in expected0 {
            close(r.next_gaussian(), want, 1e-15);
        }

        let expected12345: [f64; 3] = [-0.187808989658912, 0.5884363051154796, 0.9488047804400426];
        let mut r = JavaRandom::new(12345);
        for want in expected12345 {
            close(r.next_gaussian(), want, 1e-15);
        }
    }

    #[test]
    fn oracle_next_double_matches_java_exactly() {
        // `new Random(7).nextDouble()` twice — no transcendental involved, so
        // this one *is* bit-for-bit.
        let mut r = JavaRandom::new(7);
        assert_eq!(r.next_double(), 0.7306990420600421);
        assert_eq!(r.next_double(), 0.7491696031336331);
    }

    // -- applyOverrides ---------------------------------------------------

    #[test]
    fn an_override_replaces_the_defining_assignment() {
        let text = "x = 1\ny = 2 * x\n";
        let out = apply_overrides(text, &["x = 3.5".to_string()]);
        assert_eq!(out, "\ny = 2 * x\n\nx = 3.5\n");
    }

    #[test]
    fn overrides_are_matched_case_insensitively_and_across_semicolons() {
        let text = "Alpha = 1; beta = 2\ngamma = Alpha + beta\n";
        let out = apply_overrides(text, &["ALPHA = 9".to_string()]);
        assert_eq!(out, " beta = 2\ngamma = Alpha + beta\n\nALPHA = 9\n");
    }

    #[test]
    fn a_variable_that_merely_appears_on_a_right_hand_side_is_untouched() {
        let text = "q = 4\nw = q * 2\n";
        let out = apply_overrides(text, &["w = 1".to_string()]);
        assert!(out.contains("q = 4"));
        assert!(!out.contains("w = q * 2"));
    }

    #[test]
    fn a_malformed_override_is_ignored() {
        let text = "x = 1\n";
        assert_eq!(apply_overrides(text, &["no equals sign".to_string()]), text);
        assert_eq!(apply_overrides(text, &["= 5".to_string()]), text);
        assert_eq!(apply_overrides(text, &[]), text);
    }

    #[test]
    fn java_split_drops_trailing_empty_segments() {
        assert_eq!(java_split_semicolons("a=1;"), vec!["a=1"]);
        assert_eq!(java_split_semicolons(";"), Vec::<&str>::new());
        assert_eq!(java_split_semicolons("a;b"), vec!["a", "b"]);
        assert_eq!(java_split_semicolons(""), Vec::<&str>::new());
    }

    #[test]
    fn a_drawn_value_renders_without_an_exponent() {
        assert_eq!(to_plain_string(2.0), "2");
        assert_eq!(to_plain_string(1e-5), "0.00001");
        assert!(!to_plain_string(1e20).contains('e'));
        assert!(!to_plain_string(1e-20).contains('e'));
    }

    // -- the sampling loop ------------------------------------------------

    fn spec(guess: f64, uncertainty: f64) -> UncertaintySpec {
        UncertaintySpec {
            guess,
            uncertainty,
            ..UncertaintySpec::default()
        }
    }

    #[test]
    fn sampling_a_linear_document_recovers_the_analytic_sigma() {
        // y = 3x with x ~ N(2, 0.1): sigma_y must be 3 * 0.1 = 0.3, and the
        // mean must sit on 6. A linear model is where Monte Carlo and the
        // first-order engine agree exactly, which is what makes it a usable
        // assertion on a stochastic method.
        let text = "x = 2\ny = 3 * x\n";
        let specs = BTreeMap::from([("x".to_string(), spec(2.0, 0.1))]);
        let outcome = run(
            text,
            &SolverSettings::default(),
            &specs,
            &BTreeMap::from([("y".to_string(), 0.3)]),
            2000,
            42,
            || false,
        )
        .expect("monte carlo");

        assert_eq!(outcome.sources, ["x"]);
        assert_eq!(outcome.failed_samples, 0);
        assert!(!outcome.truncated);
        assert_eq!(outcome.samples.len(), 2000);

        let y = outcome
            .stats
            .iter()
            .find(|s| s.variable == "y")
            .expect("y stats");
        // 2000 draws: the sample mean's standard error is 0.3/sqrt(2000) ~ 0.0067
        // and the sigma estimate's is ~0.0047, so 5-sigma bands are these.
        close(y.mean, 6.0, 0.006);
        assert!(
            (y.sigma - 0.3).abs() < 0.025,
            "sigma {} should be near 0.3",
            y.sigma
        );
        assert_eq!(y.first_order_sigma, 0.3);
        // Percentiles of a normal: p5/p95 sit at ~1.645 sigma either side.
        assert!(y.p5 < y.p50 && y.p50 < y.p95);
        close(y.p50, 6.0, 0.02);
        assert!((y.p95 - y.p5 - 2.0 * 1.645 * 0.3).abs() < 0.12);
    }

    #[test]
    fn oracle_a_seeded_run_reproduces_the_java_sample_for_sample() {
        // `MonteCarlo.run(solver, "x = 2\ny = 3 * x\n", DEFAULTS,
        //                 {x: guess 2, unbounded, uncertainty 0.1}, {}, 8, 42L, …)`
        // in the reference engine. This is the whole chain in one assertion:
        // the LCG, `nextGaussian`'s polar method, the override rendering, the
        // override splice, and the re-solve.
        let expected_x: [f64; 8] = [
            2.1141905315473055,
            2.091940794898279,
            1.9050133363109105,
            1.8893009713600661,
            2.028097763807278,
            2.0684622795632657,
            1.9182778592601273,
            1.8603356597321956,
        ];
        let expected_y: [f64; 8] = [
            6.3425715946419166,
            6.275822384694837,
            5.715040008932732,
            5.667902914080198,
            6.084293291421835,
            6.2053868386897975,
            5.754833577780381,
            5.581006979196587,
        ];
        let specs = BTreeMap::from([("x".to_string(), spec(2.0, 0.1))]);
        let outcome = run(
            "x = 2\ny = 3 * x\n",
            &SolverSettings::default(),
            &specs,
            &BTreeMap::from([
                ("x".to_string(), 0.1),
                ("y".to_string(), 0.30000000000000004),
            ]),
            8,
            42,
            || false,
        )
        .expect("monte carlo");

        assert_eq!(outcome.sources, ["x"]);
        assert_eq!(outcome.failed_samples, 0);
        assert_eq!(outcome.base_values["x"], 2.0);
        assert_eq!(outcome.base_values["y"], 6.0);
        for (i, sample) in outcome.samples.iter().enumerate() {
            assert!(sample.success, "sample {i}: {:?}", sample.error);
            close(sample.values["x"], expected_x[i], 1e-14);
            close(sample.values["y"], expected_y[i], 1e-14);
        }

        // The aggregate half, also from the reference run.
        let x = outcome
            .stats
            .iter()
            .find(|s| s.variable == "x")
            .expect("x stats");
        close(x.mean, 1.9844523995599284, 1e-13);
        close(x.sigma, 0.1017677395769102, 1e-13);
        close(x.p5, 1.8704735188019503, 1e-13);
        close(x.p50, 1.9731878115337027, 1e-13);
        close(x.p95, 2.1064031237201464, 1e-13);
        assert_eq!(x.first_order_sigma, 0.1);
        let y = outcome
            .stats
            .iter()
            .find(|s| s.variable == "y")
            .expect("y stats");
        close(y.mean, 5.953357198679785, 1e-13);
        close(y.sigma, 0.30530321873073063, 1e-13);
        close(y.p5, 5.611420556405851, 1e-13);
        close(y.p50, 5.919563434601108, 1e-13);
        close(y.p95, 6.319209371160438, 1e-13);
        assert_eq!(y.first_order_sigma, 0.30000000000000004);
    }

    #[test]
    fn the_same_seed_reproduces_the_run_exactly() {
        let text = "x = 2\ny = 3 * x\n";
        let specs = BTreeMap::from([("x".to_string(), spec(2.0, 0.1))]);
        let opts = || {
            run(
                text,
                &SolverSettings::default(),
                &specs,
                &BTreeMap::new(),
                25,
                7,
                || false,
            )
            .expect("monte carlo")
        };
        assert_eq!(opts().samples, opts().samples);
    }

    #[test]
    fn bounds_clamp_the_draws() {
        // x is pinned to [1.99, 2.01] while the draw sigma is 1.0, so almost
        // every sample lands on a bound and y = 3x must stay inside [5.97, 6.03].
        let text = "x = 2\ny = 3 * x\n";
        let specs = BTreeMap::from([(
            "x".to_string(),
            UncertaintySpec {
                guess: 2.0,
                lower: 1.99,
                upper: 2.01,
                uncertainty: 1.0,
            },
        )]);
        let outcome = run(
            text,
            &SolverSettings::default(),
            &specs,
            &BTreeMap::new(),
            100,
            3,
            || false,
        )
        .expect("monte carlo");
        for sample in &outcome.samples {
            assert!(sample.success, "{:?}", sample.error);
            let y = sample.values["y"];
            assert!((5.97 - 1e-9..=6.03 + 1e-9).contains(&y), "y = {y}");
        }
    }

    #[test]
    fn a_budget_strike_truncates_instead_of_failing() {
        let text = "x = 2\ny = 3 * x\n";
        let specs = BTreeMap::from([("x".to_string(), spec(2.0, 0.1))]);
        let mut drawn = 0;
        let outcome = run(
            text,
            &SolverSettings::default(),
            &specs,
            &BTreeMap::new(),
            1000,
            1,
            || {
                drawn += 1;
                drawn > 5
            },
        )
        .expect("monte carlo");
        assert!(outcome.truncated);
        assert_eq!(outcome.samples.len(), 5);
    }

    #[test]
    fn a_document_with_no_declared_uncertainty_is_refused() {
        let text = "x = 2\ny = 3 * x\n";
        let specs = BTreeMap::from([("x".to_string(), spec(2.0, 0.0))]);
        let err = run(
            text,
            &SolverSettings::default(),
            &specs,
            &BTreeMap::new(),
            10,
            1,
            || false,
        )
        .unwrap_err();
        assert!(
            err.to_string_message().starts_with("Monte Carlo needs"),
            "{}",
            err.to_string_message()
        );
    }

    #[test]
    fn a_source_the_base_solve_never_produced_is_not_a_source() {
        // `ghost` carries an uncertainty but appears nowhere in the document.
        let text = "x = 2\ny = 3 * x\n";
        let specs = BTreeMap::from([
            ("x".to_string(), spec(2.0, 0.1)),
            ("ghost".to_string(), spec(1.0, 5.0)),
        ]);
        let outcome = run(
            text,
            &SolverSettings::default(),
            &specs,
            &BTreeMap::new(),
            5,
            1,
            || false,
        )
        .expect("monte carlo");
        assert_eq!(outcome.sources, ["x"]);
    }

    #[test]
    fn a_variable_with_fewer_than_two_usable_samples_is_skipped() {
        let base = BTreeMap::from([("x".to_string(), 1.0), ("y".to_string(), 2.0)]);
        let samples = vec![Sample {
            success: true,
            values: BTreeMap::from([("x".to_string(), 1.0), ("y".to_string(), 2.0)]),
            error: None,
        }];
        assert!(aggregate(&base, &samples, &BTreeMap::new()).is_empty());
    }

    #[test]
    fn percentiles_interpolate_linearly() {
        let sorted = [0.0, 1.0, 2.0, 3.0, 4.0];
        assert_eq!(percentile(&sorted, 0.0), 0.0);
        assert_eq!(percentile(&sorted, 1.0), 4.0);
        assert_eq!(percentile(&sorted, 0.5), 2.0);
        // rank = 0.05 * 4 = 0.2 → 0 + 0.2 * (1 - 0) = 0.2
        close(percentile(&sorted, 0.05), 0.2, 1e-15);
    }
}
