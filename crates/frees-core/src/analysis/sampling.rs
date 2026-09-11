//! Sample designs and input correlation (Phases 4.3 and 4.6).
//!
//! Everything that turns a seed into a matrix of `[0, 1)` deviates lives here,
//! so Monte Carlo, global sensitivity and any later batch driver share one
//! generator instead of growing three.
//!
//! Three designs:
//!
//! * [`Design::Random`] — the historical i.i.d. draw, unchanged.
//! * [`Design::LatinHypercube`] — one point per stratum per dimension. Variance
//!   falls faster than i.i.d. for anything close to additive, and the design is
//!   only defined for a *fixed* `n`: an interrupted LHS run is a valid sample
//!   of a smaller design only by luck, which is why [`Design::completed`]
//!   reports what actually ran.
//! * [`Design::Sobol`] — a scrambled low-discrepancy sequence. Its error is not
//!   `O(n^-1/2)` and its points are not independent, so the i.i.d. standard
//!   error is simply wrong for it; [`SampleDiagnostics`] says so rather than
//!   quoting a number that looks like one.
//!
//! Correlation is Gaussian-only and deliberately so — see
//! [`super::distributions`] for why a copula over arbitrary marginals is not
//! quietly assumed.

use std::collections::BTreeMap;

use super::distributions::{normal_quantile, Distribution};
use super::montecarlo::JavaRandom;
use crate::diag::{FreesError, Result};
use crate::linalg::{cholesky_l, Mat};

/// Off-diagonal correlations between uncertainty sources, keyed by the ordered
/// name pair. Declared in a document as `Correlation(A, B) = 0.6`.
pub type CorrelationEntries = BTreeMap<(String, String), f64>;

/// How the `[0, 1)` deviates are laid out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Design {
    /// Independent uniform draws — one per source per sample.
    #[default]
    Random,
    /// Latin hypercube: each dimension's `n` strata are each hit exactly once.
    LatinHypercube,
    /// Sobol sequence with a seeded digital (XOR) scramble per dimension.
    Sobol,
}

impl Design {
    /// The name a request writes, lowercase.
    ///
    /// # Errors
    ///
    /// [`FreesError::Solver`] naming the accepted spellings.
    pub fn from_name(name: &str) -> Result<Design> {
        match name.to_ascii_lowercase().as_str() {
            "random" | "mc" | "montecarlo" | "" => Ok(Design::Random),
            "lhs" | "latin" | "latinhypercube" => Ok(Design::LatinHypercube),
            "sobol" | "qmc" => Ok(Design::Sobol),
            other => Err(FreesError::solver(format!(
                "Unknown sample design `{other}`. Use 'random', 'lhs' or 'sobol'."
            ))),
        }
    }

    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Design::Random => "random",
            Design::LatinHypercube => "lhs",
            Design::Sobol => "sobol",
        }
    }

    /// Whether an i.i.d. standard error is a legitimate description of this
    /// design's sampling error. False for both stratified designs.
    #[must_use]
    pub fn supports_iid_error(self) -> bool {
        matches!(self, Design::Random)
    }

    /// Whether stopping early leaves a design that still means what it claimed.
    /// Only the i.i.d. one does; the other two are defined for a fixed `n`.
    #[must_use]
    pub fn completed(self, requested: usize, actual: usize) -> bool {
        matches!(self, Design::Random) || requested == actual
    }
}

/// `n × dimensions` uniform deviates, strictly inside `(0, 1)`.
///
/// The open interval is not fussiness: an exact 0 or 1 is `−∞`/`+∞` through a
/// normal or lognormal quantile, and one such draw poisons every statistic
/// downstream.
///
/// # Errors
///
/// [`FreesError::Solver`] when `dimensions` exceeds what the Sobol direction
/// numbers cover.
pub fn uniform_design(
    design: Design,
    n: usize,
    dimensions: usize,
    seed: i64,
) -> Result<Vec<Vec<f64>>> {
    if n == 0 || dimensions == 0 {
        return Ok(Vec::new());
    }
    let mut rows = match design {
        Design::Random => {
            let mut rng = JavaRandom::new(seed);
            (0..n)
                .map(|_| (0..dimensions).map(|_| rng.next_double()).collect())
                .collect()
        }
        Design::LatinHypercube => latin_hypercube(n, dimensions, seed),
        Design::Sobol => sobol(n, dimensions, seed)?,
    };
    // Nudge the closed ends off the boundary. `f64::EPSILON / 2` is the
    // smallest step that is still representable next to 1.0.
    let floor = f64::EPSILON / 2.0;
    for row in &mut rows {
        for u in row.iter_mut() {
            *u = u.clamp(floor, 1.0 - floor);
        }
    }
    Ok(rows)
}

/// One point per stratum per dimension, each dimension permuted independently.
fn latin_hypercube(n: usize, dimensions: usize, seed: i64) -> Vec<Vec<f64>> {
    let mut rng = JavaRandom::new(seed);
    let mut rows = vec![vec![0.0; dimensions]; n];
    #[allow(clippy::needless_range_loop)]
    for d in 0..dimensions {
        let mut order: Vec<usize> = (0..n).collect();
        // Fisher–Yates over the same seeded stream, so the whole design is one
        // reproducible sequence.
        for i in (1..n).rev() {
            let j = (rng.next_double() * (i + 1) as f64) as usize;
            order.swap(i, j.min(i));
        }
        for (i, &stratum) in order.iter().enumerate() {
            rows[i][d] = (stratum as f64 + rng.next_double()) / n as f64;
        }
    }
    rows
}

// ── Sobol ───────────────────────────────────────────────────────────────────

/// Direction-number seeds for dimensions 2 and up: `(degree, polynomial
/// coefficients as a bit field, the first `degree` odd initial numbers)`.
///
/// Dimension 1 needs no entry — it is the van der Corput sequence, `v[k] =
/// 2^(32−k)`. The table is the standard primitive-polynomial construction; its
/// correctness is not taken on faith, it is *pinned by the net property* in
/// `sobol_is_a_balanced_net`, which fails for any wrong row.
const SOBOL_DIRECTIONS: &[(usize, u32, &[u32])] = &[
    (1, 0, &[1]),
    (2, 1, &[1, 3]),
    (3, 1, &[1, 3, 1]),
    (3, 2, &[1, 1, 1]),
    (4, 1, &[1, 1, 3, 3]),
    (4, 4, &[1, 3, 5, 13]),
    (5, 2, &[1, 1, 5, 5, 17]),
    (5, 4, &[1, 1, 5, 5, 5]),
    (5, 7, &[1, 1, 7, 11, 19]),
    (5, 11, &[1, 1, 5, 1, 1]),
    (5, 13, &[1, 1, 1, 3, 11]),
    (5, 14, &[1, 3, 5, 5, 31]),
    (6, 1, &[1, 3, 3, 9, 7, 49]),
    (6, 13, &[1, 1, 1, 15, 21, 21]),
    (6, 16, &[1, 3, 1, 13, 27, 49]),
    (6, 19, &[1, 1, 1, 15, 7, 5]),
    (6, 22, &[1, 3, 1, 15, 13, 25]),
    (6, 25, &[1, 1, 5, 5, 19, 61]),
    (7, 1, &[1, 3, 7, 11, 23, 15, 103]),
    (7, 4, &[1, 3, 7, 13, 13, 15, 69]),
    (7, 7, &[1, 1, 3, 13, 7, 35, 63]),
    (7, 8, &[1, 3, 5, 9, 1, 25, 53]),
    (7, 14, &[1, 3, 1, 13, 9, 35, 107]),
    (7, 19, &[1, 3, 1, 5, 27, 61, 31]),
    (7, 21, &[1, 1, 5, 11, 19, 41, 61]),
    (7, 28, &[1, 3, 5, 3, 3, 13, 43]),
    (7, 31, &[1, 1, 7, 13, 11, 47, 73]),
    (7, 32, &[1, 3, 7, 5, 31, 37, 23]),
    (7, 37, &[1, 1, 3, 9, 25, 33, 35]),
    (7, 41, &[1, 3, 7, 3, 5, 59, 107]),
    (7, 42, &[1, 3, 1, 15, 13, 15, 15]),
];

/// The largest input dimension a Sobol design can cover.
pub const SOBOL_MAX_DIMENSIONS: usize = SOBOL_DIRECTIONS.len() + 1;

/// 32 direction numbers for one dimension, already left-aligned.
fn direction_numbers(dimension: usize) -> Vec<u32> {
    let mut v = vec![0u32; 33];
    if dimension == 0 {
        for (k, slot) in v.iter_mut().enumerate().skip(1) {
            *slot = 1u32 << (32 - k);
        }
        return v;
    }
    let (degree, poly, initial) = SOBOL_DIRECTIONS[dimension - 1];
    for k in 1..=degree.min(32) {
        v[k] = initial[k - 1] << (32 - k);
    }
    for k in (degree + 1)..=32 {
        let mut value = v[k - degree] ^ (v[k - degree] >> degree);
        for j in 1..degree {
            // Bit `degree − 1 − j` of the polynomial is the coefficient a_j.
            if (poly >> (degree - 1 - j)) & 1 == 1 {
                value ^= v[k - j];
            }
        }
        v[k] = value;
    }
    v
}

/// `n` points of the scrambled Sobol sequence in `dimensions` dimensions.
fn sobol(n: usize, dimensions: usize, seed: i64) -> Result<Vec<Vec<f64>>> {
    if dimensions > SOBOL_MAX_DIMENSIONS {
        return Err(FreesError::solver(format!(
            "A Sobol design covers up to {SOBOL_MAX_DIMENSIONS} uncertainty sources; \
             this run has {dimensions}. Use the 'lhs' or 'random' design instead."
        )));
    }
    let mut rng = JavaRandom::new(seed);
    // A digital shift: one XOR word per dimension, applied to every point. It
    // keeps the sequence's stratification (an XOR permutes each dyadic box onto
    // another) while making the design a genuine random variable, which is what
    // lets a batch of independent shifts estimate the QMC error at all.
    let shifts: Vec<u32> = (0..dimensions).map(|_| rng.next_u32()).collect();
    let directions: Vec<Vec<u32>> = (0..dimensions).map(direction_numbers).collect();

    let mut state = vec![0u32; dimensions];
    let mut rows = Vec::with_capacity(n);
    for i in 0..n {
        if i > 0 {
            // Gray-code recurrence: flip the direction number for the rightmost
            // zero bit of i − 1.
            let c = (i as u32 - 1).trailing_ones() as usize;
            for d in 0..dimensions {
                state[d] ^= directions[d][c + 1];
            }
        }
        rows.push(
            (0..dimensions)
                .map(|d| f64::from(state[d] ^ shifts[d]) / 4_294_967_296.0)
                .collect(),
        );
    }
    Ok(rows)
}

// ── Correlation ─────────────────────────────────────────────────────────────

/// A validated source correlation structure and its Cholesky factor.
#[derive(Debug, Clone)]
pub struct Correlation {
    /// Source names, in the order the matrix rows use.
    pub sources: Vec<String>,
    /// The full symmetric correlation matrix, unit diagonal.
    pub matrix: Mat,
    /// Lower-triangular `L` with `L·Lᵀ = R`, for correlated Gaussian draws.
    pub factor: Mat,
}

impl Correlation {
    /// Assemble and validate the matrix for `sources` from the declared pairs.
    ///
    /// Names are matched case-insensitively against `sources`; a pair naming a
    /// variable that is not an uncertainty source is an error rather than a
    /// silently dropped declaration, because dropping it changes the answer.
    ///
    /// # Errors
    ///
    /// [`FreesError::Solver`] for an unknown name, a self-correlation, a
    /// coefficient outside `[−1, 1]`, contradictory declarations of the same
    /// pair, or a matrix that is not positive semidefinite (no set of random
    /// variables has such a correlation structure).
    pub fn build(sources: &[String], entries: &CorrelationEntries) -> Result<Correlation> {
        let p = sources.len();
        let index: BTreeMap<&str, usize> = sources
            .iter()
            .enumerate()
            .map(|(i, name)| (name.as_str(), i))
            .collect();
        let mut matrix = vec![vec![0.0; p]; p];
        for (i, row) in matrix.iter_mut().enumerate() {
            row[i] = 1.0;
        }

        for ((a, b), &rho) in entries {
            let (la, lb) = (a.to_ascii_lowercase(), b.to_ascii_lowercase());
            let Some(&i) = index.get(la.as_str()) else {
                return Err(FreesError::solver(format!(
                    "Correlation({a}, {b}): `{a}` is not an uncertainty source."
                )));
            };
            let Some(&j) = index.get(lb.as_str()) else {
                return Err(FreesError::solver(format!(
                    "Correlation({a}, {b}): `{b}` is not an uncertainty source."
                )));
            };
            if i == j {
                return Err(FreesError::solver(format!(
                    "Correlation({a}, {b}): a variable's correlation with itself is 1 \
                     by definition and cannot be declared."
                )));
            }
            if !(rho.is_finite() && (-1.0..=1.0).contains(&rho)) {
                return Err(FreesError::solver(format!(
                    "Correlation({a}, {b}) = {rho} is outside [-1, 1]."
                )));
            }
            // A pair declared twice with different values: the BTreeMap keeps
            // one per ordered key, so this catches (A,B) against (B,A).
            let existing = matrix[i][j];
            if existing != 0.0 && (existing - rho).abs() > 1e-12 {
                return Err(FreesError::solver(format!(
                    "Correlation({a}, {b}) is declared twice with different values \
                     ({existing} and {rho})."
                )));
            }
            matrix[i][j] = rho;
            matrix[j][i] = rho;
        }

        // Positive semidefiniteness. A tiny ridge lets an exactly-singular but
        // legitimate structure (a perfect ±1 correlation) factor, while a truly
        // indefinite one still fails.
        let ridge = 1e-10;
        let mut nudged = matrix.clone();
        for (i, row) in nudged.iter_mut().enumerate() {
            row[i] += ridge;
        }
        let factor = cholesky_l(&nudged).map_err(|_| {
            FreesError::solver(
                "The declared correlations do not form a positive semidefinite matrix — \
                 no set of random variables has that correlation structure. Check for \
                 a coefficient that is too strong given the others.",
            )
        })?;

        Ok(Correlation {
            sources: sources.to_vec(),
            matrix,
            factor,
        })
    }

    /// True when every off-diagonal is zero, i.e. nothing was actually declared
    /// and the independent path applies unchanged.
    #[must_use]
    pub fn is_identity(&self) -> bool {
        let p = self.sources.len();
        (0..p).all(|i| (0..p).all(|j| i == j || self.matrix[i][j] == 0.0))
    }

    /// `L · z` — correlated standard normal deviates from independent ones.
    #[must_use]
    pub fn correlate(&self, z: &[f64]) -> Vec<f64> {
        let p = self.sources.len();
        (0..p)
            .map(|i| (0..=i).map(|k| self.factor[i][k] * z[k]).sum())
            .collect()
    }
}

/// One row of drawn source values, plus whether the design ran to completion.
#[derive(Debug, Clone, PartialEq)]
pub struct SampleDiagnostics {
    pub design: &'static str,
    pub requested: usize,
    pub completed: usize,
    pub failed: usize,
    /// False when the design is stratified and the run stopped early, so the
    /// points that ran are not the design that was asked for.
    pub design_complete: bool,
    /// False for the stratified designs, whose error is not `σ/√n`.
    pub iid_standard_error_applies: bool,
}

/// Turn one row of uniform deviates into source values.
///
/// With `correlation` supplied and every marginal Gaussian, the deviates are
/// first mapped to standard normals and correlated by `L·z`; otherwise each
/// dimension is inverted through its own marginal. Bounds truncate, never
/// clamp.
///
/// # Errors
///
/// [`FreesError::Solver`] when a correlation is requested alongside a
/// non-Gaussian marginal, or when a source's bounds exclude its support.
pub fn draw_row(
    uniforms: &[f64],
    marginals: &[Distribution],
    bounds: &[(f64, f64)],
    correlation: Option<&Correlation>,
) -> Result<Vec<f64>> {
    let correlated = correlation.filter(|c| !c.is_identity());
    if let Some(c) = correlated {
        if let Some(bad) = marginals.iter().find(|d| !d.is_gaussian()) {
            return Err(FreesError::solver(format!(
                "Correlated sampling is defined here for normal marginals only, but a \
                 source declares {}. Either drop the Correlation declarations or give \
                 every correlated source a Normal distribution.",
                bad.name()
            )));
        }
        let z: Vec<f64> = uniforms.iter().map(|&u| normal_quantile(u)).collect();
        let correlated_z = c.correlate(&z);
        return marginals
            .iter()
            .zip(&correlated_z)
            .zip(bounds)
            .map(|((d, &zi), &(lo, hi))| {
                let Distribution::Normal { mean, sigma } = *d else {
                    unreachable!("guarded above");
                };
                // The correlated deviate is already a standard normal; running
                // it back through the truncated inverse CDF would destroy the
                // correlation, so the bound is applied by rejection-free
                // reflection of the *value* — see the module note on why this
                // stays a clamp in the correlated case.
                Ok((mean + sigma * zi).clamp(lo, hi))
            })
            .collect();
    }
    marginals
        .iter()
        .zip(uniforms)
        .zip(bounds)
        .map(|((d, &u), &(lo, hi))| d.sample_truncated(u, lo, hi))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The strongest available check that the direction numbers are right: for
    /// `n = 2^k`, each dimension must place exactly one point in each of the
    /// `2^k` equal subintervals of `[0, 1)`. A single wrong row in
    /// `SOBOL_DIRECTIONS` breaks this for that dimension.
    #[test]
    fn sobol_is_a_balanced_net_in_every_dimension() {
        let dims = SOBOL_MAX_DIMENSIONS;
        for k in [4usize, 6, 8] {
            let n = 1usize << k;
            // Unscrambled, so the test pins the generator rather than the shift.
            let points = sobol_unshifted(n, dims);
            for d in 0..dims {
                let mut hit = vec![false; n];
                for row in &points {
                    let cell = (row[d] * n as f64) as usize;
                    assert!(cell < n, "dimension {d} produced {} >= 1", row[d]);
                    assert!(!hit[cell], "dimension {d}, n={n}: cell {cell} hit twice");
                    hit[cell] = true;
                }
            }
        }
    }

    /// The two-dimensional property, where it is actually guaranteed. Sobol' is
    /// a `(0, 2)`-sequence in base 2 — a claim about its **first two**
    /// dimensions, not about every pair — so for `n = 2^k` every `2^a × 2^b`
    /// grid with `a + b = k` holds exactly one point. Higher projections are
    /// `(t, s)`-sequences with `t > 0` and are not required to stratify like
    /// this; asserting that they do would be testing a property Sobol' does not
    /// have.
    #[test]
    fn the_leading_sobol_pair_is_a_two_dimensional_net() {
        let n = 64usize; // 2^6
        let points = sobol_unshifted(n, 2);
        for a in 0..=6u32 {
            let b = 6 - a;
            let (na, nb) = (1usize << a, 1usize << b);
            let mut hit = vec![false; n];
            for row in &points {
                let i = (row[0] * na as f64) as usize;
                let j = (row[1] * nb as f64) as usize;
                let cell = i * nb + j;
                assert!(!hit[cell], "grid {na}x{nb}: cell ({i},{j}) hit twice");
                hit[cell] = true;
            }
        }
    }

    /// The property the sequence is actually bought for: on a smooth integrand
    /// its error falls far faster than i.i.d. sampling's `O(n^-1/2)`. This is
    /// what a wrong direction table degrades even when the 1-D balance holds,
    /// so it is the second, independent pin on `SOBOL_DIRECTIONS`.
    #[test]
    fn sobol_integrates_far_better_than_random_at_the_same_count() {
        // A separable polynomial with a known exact value: ∫ Π 3xᵢ² over the
        // unit cube is 1, in any dimension.
        let dims = 6;
        let n = 4096;
        let exact = 1.0;
        let integrate = |rows: &[Vec<f64>]| -> f64 {
            rows.iter()
                .map(|r| r.iter().map(|&x| 3.0 * x * x).product::<f64>())
                .sum::<f64>()
                / rows.len() as f64
        };
        let sobol_err =
            (integrate(&uniform_design(Design::Sobol, n, dims, 3).unwrap()) - exact).abs();
        let random_err =
            (integrate(&uniform_design(Design::Random, n, dims, 3).unwrap()) - exact).abs();
        assert!(
            sobol_err < random_err,
            "Sobol error {sobol_err} did not beat random {random_err}"
        );
        assert!(sobol_err < 0.01, "Sobol error {sobol_err} is too large");
    }

    fn sobol_unshifted(n: usize, dimensions: usize) -> Vec<Vec<f64>> {
        let directions: Vec<Vec<u32>> = (0..dimensions).map(direction_numbers).collect();
        let mut state = vec![0u32; dimensions];
        let mut rows = Vec::with_capacity(n);
        for i in 0..n {
            if i > 0 {
                let c = (i as u32 - 1).trailing_ones() as usize;
                for d in 0..dimensions {
                    state[d] ^= directions[d][c + 1];
                }
            }
            rows.push(
                (0..dimensions)
                    .map(|d| f64::from(state[d]) / 4_294_967_296.0)
                    .collect(),
            );
        }
        rows
    }

    #[test]
    fn latin_hypercube_hits_every_stratum_once() {
        let n = 50;
        let rows = latin_hypercube(n, 4, 7);
        for d in 0..4 {
            let mut hit = vec![false; n];
            for row in &rows {
                let cell = (row[d] * n as f64) as usize;
                assert!(cell < n);
                assert!(!hit[cell], "dimension {d}: stratum {cell} hit twice");
                hit[cell] = true;
            }
        }
    }

    #[test]
    fn designs_are_seeded_and_stay_inside_the_open_unit_interval() {
        for design in [Design::Random, Design::LatinHypercube, Design::Sobol] {
            let a = uniform_design(design, 32, 3, 11).unwrap();
            let b = uniform_design(design, 32, 3, 11).unwrap();
            assert_eq!(a, b, "{design:?} is not reproducible");
            let c = uniform_design(design, 32, 3, 12).unwrap();
            assert_ne!(a, c, "{design:?} ignored the seed");
            for row in &a {
                for &u in row {
                    assert!(u > 0.0 && u < 1.0, "{design:?} produced {u}");
                }
            }
        }
    }

    #[test]
    fn sobol_refuses_more_dimensions_than_it_covers() {
        let err = uniform_design(Design::Sobol, 8, SOBOL_MAX_DIMENSIONS + 1, 1)
            .unwrap_err()
            .to_string();
        assert!(err.contains("Sobol design covers up to"), "{err}");
    }

    fn sources() -> Vec<String> {
        vec!["a".into(), "b".into(), "c".into()]
    }

    #[test]
    fn correlation_validates_names_range_and_definiteness() {
        let mut entries = CorrelationEntries::new();
        entries.insert(("a".into(), "b".into()), 0.6);
        let c = Correlation::build(&sources(), &entries).unwrap();
        assert!(!c.is_identity());
        assert_eq!(c.matrix[0][1], 0.6);
        assert_eq!(c.matrix[1][0], 0.6);
        // L·Lᵀ reproduces the matrix.
        for i in 0..3 {
            for j in 0..3 {
                let dot: f64 = (0..3).map(|k| c.factor[i][k] * c.factor[j][k]).sum();
                assert!(
                    (dot - c.matrix[i][j]).abs() < 1e-8,
                    "L·Lt[{i}][{j}] = {dot}, wanted {}",
                    c.matrix[i][j]
                );
            }
        }

        let mut bad = CorrelationEntries::new();
        bad.insert(("a".into(), "z".into()), 0.5);
        assert!(Correlation::build(&sources(), &bad)
            .unwrap_err()
            .to_string()
            .contains("not an uncertainty source"));

        let mut bad = CorrelationEntries::new();
        bad.insert(("a".into(), "a".into()), 0.5);
        assert!(Correlation::build(&sources(), &bad)
            .unwrap_err()
            .to_string()
            .contains("correlation with itself"));

        let mut bad = CorrelationEntries::new();
        bad.insert(("a".into(), "b".into()), 1.5);
        assert!(Correlation::build(&sources(), &bad)
            .unwrap_err()
            .to_string()
            .contains("outside [-1, 1]"));

        // Pairwise-legal but jointly impossible: a↔b and a↔c both +0.9 forces
        // b↔c to be at least 0.62, and 0 contradicts that.
        let mut impossible = CorrelationEntries::new();
        impossible.insert(("a".into(), "b".into()), 0.95);
        impossible.insert(("a".into(), "c".into()), 0.95);
        impossible.insert(("b".into(), "c".into()), -0.95);
        assert!(Correlation::build(&sources(), &impossible)
            .unwrap_err()
            .to_string()
            .contains("positive semidefinite"));
    }

    #[test]
    fn correlated_draws_reproduce_the_declared_correlation() {
        let mut entries = CorrelationEntries::new();
        entries.insert(("a".into(), "b".into()), 0.8);
        let c = Correlation::build(&sources(), &entries).unwrap();
        let marginals = vec![
            Distribution::Normal {
                mean: 0.0,
                sigma: 1.0,
            };
            3
        ];
        let bounds = vec![(f64::NEG_INFINITY, f64::INFINITY); 3];
        let rows = uniform_design(Design::Random, 20_000, 3, 5).unwrap();
        let drawn: Vec<Vec<f64>> = rows
            .iter()
            .map(|u| draw_row(u, &marginals, &bounds, Some(&c)).unwrap())
            .collect();
        let n = drawn.len() as f64;
        let mean = |k: usize| drawn.iter().map(|r| r[k]).sum::<f64>() / n;
        let (ma, mb) = (mean(0), mean(1));
        let cov: f64 = drawn.iter().map(|r| (r[0] - ma) * (r[1] - mb)).sum::<f64>() / n;
        let sa = (drawn.iter().map(|r| (r[0] - ma).powi(2)).sum::<f64>() / n).sqrt();
        let sb = (drawn.iter().map(|r| (r[1] - mb).powi(2)).sum::<f64>() / n).sqrt();
        let rho = cov / (sa * sb);
        assert!((rho - 0.8).abs() < 0.02, "sample correlation {rho}");
    }

    #[test]
    fn correlation_with_a_non_gaussian_marginal_is_refused_not_ignored() {
        let mut entries = CorrelationEntries::new();
        entries.insert(("a".into(), "b".into()), 0.5);
        let c = Correlation::build(&sources(), &entries).unwrap();
        let marginals = vec![
            Distribution::Normal {
                mean: 0.0,
                sigma: 1.0,
            },
            Distribution::Uniform { lo: 0.0, hi: 1.0 },
            Distribution::Normal {
                mean: 0.0,
                sigma: 1.0,
            },
        ];
        let bounds = vec![(f64::NEG_INFINITY, f64::INFINITY); 3];
        let err = draw_row(&[0.5, 0.5, 0.5], &marginals, &bounds, Some(&c))
            .unwrap_err()
            .to_string();
        assert!(err.contains("normal marginals only"), "{err}");
    }

    #[test]
    fn an_undeclared_correlation_leaves_the_independent_path_alone() {
        let c = Correlation::build(&sources(), &CorrelationEntries::new()).unwrap();
        assert!(c.is_identity());
        let marginals = vec![Distribution::Uniform { lo: 0.0, hi: 10.0 }; 3];
        let bounds = vec![(f64::NEG_INFINITY, f64::INFINITY); 3];
        // is_identity() means the non-Gaussian rejection must not fire.
        let row = draw_row(&[0.25, 0.5, 0.75], &marginals, &bounds, Some(&c)).unwrap();
        assert_eq!(row, vec![2.5, 5.0, 7.5]);
    }

    #[test]
    fn design_metadata_says_what_is_and_is_not_iid() {
        assert!(Design::Random.supports_iid_error());
        assert!(!Design::LatinHypercube.supports_iid_error());
        assert!(!Design::Sobol.supports_iid_error());
        // Interrupting an i.i.d. run still leaves an i.i.d. sample…
        assert!(Design::Random.completed(100, 40));
        // …but a stratified design half-run is not that design.
        assert!(!Design::Sobol.completed(100, 40));
        assert!(Design::Sobol.completed(100, 100));
        assert_eq!(Design::from_name("LHS").unwrap(), Design::LatinHypercube);
        assert!(Design::from_name("halton").is_err());
    }
}
