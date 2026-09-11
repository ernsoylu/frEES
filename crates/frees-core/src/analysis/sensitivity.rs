//! Global sensitivity analysis (Phase 4.6): Sobol' variance decomposition and
//! Morris elementary-effects screening.
//!
//! # Global is not local
//!
//! [`super::uncertainty`] answers "how much of *this* output's spread comes
//! from each input, at this operating point, to first order". That is a local,
//! linearized contribution and it is what the tornado chart shows. What is here
//! is a different question: over the input's **whole declared range**, how much
//! of the output variance does each input explain, interactions included. The
//! two disagree for anything nonlinear, and neither is a substitute for the
//! other — they are reported separately and must stay that way.
//!
//! # Cost
//!
//! Sobol' costs `n·(p + 2)` model evaluations for `p` inputs; Morris costs
//! `r·(p + 1)` and is the screening tool you reach for when that is too many.
//!
//! # Correlated inputs
//!
//! Refused. The Sobol' decomposition assumes independent inputs; run against
//! correlated ones it returns numbers that look fine and mean nothing, so a
//! declared correlation is an error here rather than a footnote.

use std::collections::{BTreeMap, BTreeSet};

use super::distributions::Distribution;
use super::sampling::{uniform_design, Design};
use crate::diag::{FreesError, Result};

/// What the model is asked over: one input per entry, with the distribution its
/// deviates are inverted through.
#[derive(Debug, Clone)]
pub struct InputSpace {
    pub names: Vec<String>,
    pub marginals: Vec<Distribution>,
    /// Optional truncation per input, `(lower, upper)`; use infinities for none.
    pub bounds: Vec<(f64, f64)>,
}

impl InputSpace {
    /// # Errors
    ///
    /// [`FreesError::Solver`] when the three vectors disagree in length, the
    /// space is empty, or a marginal is ill-parameterized.
    pub fn validate(&self) -> Result<()> {
        if self.names.is_empty() {
            return Err(FreesError::solver(
                "A sensitivity analysis needs at least one input.",
            ));
        }
        if self.names.len() != self.marginals.len() || self.names.len() != self.bounds.len() {
            return Err(FreesError::solver(
                "Sensitivity inputs, distributions and bounds must be the same length.",
            ));
        }
        for m in &self.marginals {
            m.validate()?;
        }
        Ok(())
    }

    fn dimension(&self) -> usize {
        self.names.len()
    }

    /// Map one row of `[0, 1)` deviates onto input values.
    fn to_values(&self, uniforms: &[f64]) -> Vec<f64> {
        self.marginals
            .iter()
            .zip(uniforms)
            .zip(&self.bounds)
            .map(|((d, &u), &(lo, hi))| d.sample_truncated(u, lo, hi).unwrap_or(f64::NAN))
            .collect()
    }
}

/// One input's share of one output's variance.
#[derive(Debug, Clone, PartialEq)]
pub struct SobolIndex {
    pub source: String,
    /// `Sᵢ` — the variance explained by this input *alone*.
    pub first_order: f64,
    /// `STᵢ` — this input's share including every interaction it takes part in.
    /// `STᵢ − Sᵢ` is how much of its influence is interaction.
    pub total: f64,
    /// Bootstrap standard error of `first_order`, or `NaN` when no bootstrap
    /// was requested. An index whose error bar spans zero is not evidence of
    /// influence.
    pub first_order_se: f64,
    /// Bootstrap standard error of `total`.
    pub total_se: f64,
}

/// The decomposition of one output variable.
#[derive(Debug, Clone, PartialEq)]
pub struct SobolOutcome {
    pub output: String,
    /// Total output variance over the whole input space. Indices divide by it,
    /// so a variance of zero makes every index meaningless and they are
    /// reported as `NaN` rather than as `0/0`.
    pub variance: f64,
    pub indices: Vec<SobolIndex>,
}

/// A Sobol' run's cost and honesty record.
#[derive(Debug, Clone, PartialEq)]
pub struct SensitivityDiagnostics {
    pub design: &'static str,
    /// Model evaluations actually performed.
    pub evaluations: usize,
    /// Design rows dropped because the model failed at one of their points.
    /// A dropped row is dropped from `A`, `B` **and** every `AB` — the
    /// estimators difference matched rows, so keeping a half row would bias
    /// every index rather than just lose a sample.
    pub dropped_rows: usize,
    /// Rows the estimators actually used.
    pub used_rows: usize,
    /// False when the budget stopped the run before the design finished.
    pub complete: bool,
}

/// Everything a Sobol' run needs beyond the model itself.
#[derive(Debug, Clone)]
pub struct SobolPlan {
    pub space: InputSpace,
    /// Base sample count `n`. The model is called `n·(p + 2)` times.
    pub base_samples: usize,
    pub seed: i64,
    /// The design behind the `A` and `B` matrices. Sobol' points are the usual
    /// choice and the reason the estimator converges as fast as it does.
    pub design: Design,
    /// Bootstrap resamples for the index standard errors; 0 disables them.
    pub bootstrap: usize,
}

impl Default for SobolPlan {
    fn default() -> SobolPlan {
        SobolPlan {
            space: InputSpace {
                names: Vec::new(),
                marginals: Vec::new(),
                bounds: Vec::new(),
            },
            base_samples: 512,
            seed: 0,
            design: Design::Sobol,
            bootstrap: 200,
        }
    }
}

/// Sobol' first-order and total-order indices for every output the model
/// returns.
///
/// `evaluate` receives one point of the input space and returns the model's
/// outputs, or `None` when the model failed there.
///
/// `expired` is the budget predicate, as everywhere else in `analysis`; an
/// early stop truncates the design and is reported, never silently averaged
/// over whatever happened to run.
///
/// # Errors
///
/// [`FreesError::Solver`] for an invalid input space, a base sample count below
/// 2, or a design that produced no usable rows at all.
pub fn sobol<F, B>(
    plan: &SobolPlan,
    mut evaluate: F,
    mut expired: B,
) -> Result<(Vec<SobolOutcome>, SensitivityDiagnostics)>
where
    F: FnMut(&[f64]) -> Option<BTreeMap<String, f64>>,
    B: FnMut() -> bool,
{
    plan.space.validate()?;
    if plan.base_samples < 2 {
        return Err(FreesError::solver(
            "A Sobol' analysis needs at least 2 base samples.",
        ));
    }
    let p = plan.space.dimension();
    let n = plan.base_samples;

    // One design of 2p columns, split into A and B. Drawing both halves from
    // one low-discrepancy sequence is what keeps the pair jointly stratified;
    // two independent sequences would not be.
    let rows = uniform_design(plan.design, n, 2 * p, plan.seed)?;

    let mut evaluations = 0usize;
    let mut complete = true;
    // `values[matrix][row]` — matrix 0 is A, 1 is B, 2 + i is A with column i
    // taken from B.
    let mut values: Vec<Vec<Option<BTreeMap<String, f64>>>> = vec![Vec::with_capacity(n); p + 2];

    'rows: for row in rows.iter().take(n) {
        let a: Vec<f64> = row[..p].to_vec();
        let b: Vec<f64> = row[p..].to_vec();
        for (matrix, column) in values.iter_mut().enumerate() {
            if expired() {
                complete = false;
                break 'rows;
            }
            let uniforms: Vec<f64> = match matrix {
                0 => a.clone(),
                1 => b.clone(),
                i => {
                    let mut mixed = a.clone();
                    mixed[i - 2] = b[i - 2];
                    mixed
                }
            };
            let point = plan.space.to_values(&uniforms);
            column.push(evaluate(&point));
            evaluations += 1;
        }
    }

    // A row survives only if every one of its p+2 evaluations did.
    let built = values[0].len();
    let keep: Vec<usize> = (0..built)
        .filter(|&j| values.iter().all(|m| m.get(j).is_some_and(Option::is_some)))
        .collect();
    if keep.len() < 2 {
        return Err(FreesError::solver(
            "The Sobol' design produced fewer than two complete rows — the model \
             failed at almost every point.",
        ));
    }

    // Only outputs every surviving evaluation reported can be decomposed.
    let mut outputs: BTreeSet<String> = values[0][keep[0]]
        .as_ref()
        .expect("kept row")
        .keys()
        .cloned()
        .collect();
    for matrix in &values {
        for &j in &keep {
            let present: BTreeSet<String> = matrix[j]
                .as_ref()
                .expect("kept row")
                .keys()
                .cloned()
                .collect();
            outputs.retain(|name| present.contains(name));
        }
    }

    let column = |matrix: usize, output: &str| -> Vec<f64> {
        keep.iter()
            .map(|&j| values[matrix][j].as_ref().expect("kept row")[output])
            .collect()
    };

    let mut outcomes = Vec::with_capacity(outputs.len());
    for output in &outputs {
        let fa = column(0, output);
        let fb = column(1, output);
        let fab: Vec<Vec<f64>> = (0..p).map(|i| column(i + 2, output)).collect();
        let all: Vec<usize> = (0..keep.len()).collect();
        let (variance, first, total) = sobol_estimates(&fa, &fb, &fab, &all);

        let (first_se, total_se) = if plan.bootstrap == 0 {
            (vec![f64::NAN; p], vec![f64::NAN; p])
        } else {
            bootstrap_errors(&fa, &fb, &fab, plan.bootstrap, plan.seed)
        };

        outcomes.push(SobolOutcome {
            output: output.clone(),
            variance,
            indices: (0..p)
                .map(|i| SobolIndex {
                    source: plan.space.names[i].clone(),
                    first_order: first[i],
                    total: total[i],
                    first_order_se: first_se[i],
                    total_se: total_se[i],
                })
                .collect(),
        });
    }

    Ok((
        outcomes,
        SensitivityDiagnostics {
            design: plan.design.label(),
            evaluations,
            dropped_rows: built - keep.len(),
            used_rows: keep.len(),
            complete,
        },
    ))
}

/// The estimators themselves, over the row subset `rows` (the whole set, or a
/// bootstrap resample).
///
/// * `Sᵢ` is Saltelli's 2010 estimator `E[f(B)·(f(AB^i) − f(A))] / V`.
/// * `STᵢ` is Jansen's `E[(f(A) − f(AB^i))²] / (2V)`, which is the one that
///   stays accurate when the index is small.
///
/// Both divide by the variance of `A` and `B` pooled.
fn sobol_estimates(
    fa: &[f64],
    fb: &[f64],
    fab: &[Vec<f64>],
    rows: &[usize],
) -> (f64, Vec<f64>, Vec<f64>) {
    let m = rows.len() as f64;
    let pooled: f64 = rows.iter().map(|&j| fa[j] + fb[j]).sum::<f64>() / (2.0 * m);
    let variance: f64 = rows
        .iter()
        .map(|&j| (fa[j] - pooled).powi(2) + (fb[j] - pooled).powi(2))
        .sum::<f64>()
        / (2.0 * m);

    let p = fab.len();
    if variance.is_nan() || variance <= 0.0 {
        // A constant output has no variance to apportion. `0/0` would print as
        // a number; NaN says what actually happened.
        return (variance, vec![f64::NAN; p], vec![f64::NAN; p]);
    }
    let mut first = Vec::with_capacity(p);
    let mut total = Vec::with_capacity(p);
    for column in fab {
        let s: f64 = rows
            .iter()
            .map(|&j| fb[j] * (column[j] - fa[j]))
            .sum::<f64>()
            / m;
        let t: f64 = rows
            .iter()
            .map(|&j| (fa[j] - column[j]).powi(2))
            .sum::<f64>()
            / (2.0 * m);
        first.push(s / variance);
        total.push(t / variance);
    }
    (variance, first, total)
}

/// Bootstrap standard errors: resample whole rows with replacement and take the
/// spread of the recomputed indices. Rows, not individual evaluations — the
/// estimators difference matched entries, and resampling those independently
/// would destroy the pairing the whole scheme rests on.
fn bootstrap_errors(
    fa: &[f64],
    fb: &[f64],
    fab: &[Vec<f64>],
    resamples: usize,
    seed: i64,
) -> (Vec<f64>, Vec<f64>) {
    let p = fab.len();
    let m = fa.len();
    let mut rng = super::montecarlo::JavaRandom::new(seed.wrapping_add(0x5017));
    let mut first_draws = vec![Vec::with_capacity(resamples); p];
    let mut total_draws = vec![Vec::with_capacity(resamples); p];
    let mut rows = vec![0usize; m];
    for _ in 0..resamples {
        for slot in rows.iter_mut() {
            *slot = ((rng.next_double() * m as f64) as usize).min(m - 1);
        }
        let (_, first, total) = sobol_estimates(fa, fb, fab, &rows);
        for i in 0..p {
            first_draws[i].push(first[i]);
            total_draws[i].push(total[i]);
        }
    }
    let sd = |draws: &[f64]| -> f64 {
        let usable: Vec<f64> = draws.iter().copied().filter(|v| v.is_finite()).collect();
        if usable.len() < 2 {
            return f64::NAN;
        }
        let mean = usable.iter().sum::<f64>() / usable.len() as f64;
        let ss: f64 = usable.iter().map(|v| (v - mean).powi(2)).sum();
        (ss / (usable.len() as f64 - 1.0)).sqrt()
    };
    (
        first_draws.iter().map(|d| sd(d)).collect(),
        total_draws.iter().map(|d| sd(d)).collect(),
    )
}

// ── Morris screening ────────────────────────────────────────────────────────

/// One input's elementary-effect summary for one output.
#[derive(Debug, Clone, PartialEq)]
pub struct MorrisEffect {
    pub source: String,
    /// Mean elementary effect — signed, so cancelling effects show as ~0.
    pub mu: f64,
    /// Mean **absolute** elementary effect. This is the ranking statistic: an
    /// input whose effect flips sign has a small `mu` and a large `mu_star`.
    pub mu_star: f64,
    /// Standard deviation of the elementary effects. Large relative to
    /// `mu_star` means the input's effect depends on where the others are —
    /// interaction or nonlinearity, which Morris cannot tell apart.
    pub sigma: f64,
    /// Elementary effects that were actually computed for this input.
    pub samples: usize,
}

/// One output's screening result.
#[derive(Debug, Clone, PartialEq)]
pub struct MorrisOutcome {
    pub output: String,
    pub effects: Vec<MorrisEffect>,
}

/// A Morris screening design.
#[derive(Debug, Clone)]
pub struct MorrisPlan {
    pub space: InputSpace,
    /// Trajectories. The model is called `trajectories·(p + 1)` times.
    pub trajectories: usize,
    /// Grid levels per input; must be even and at least 4. The step is
    /// `levels / (2·(levels − 1))`, the choice that makes the design's
    /// elementary effects equally likely across the grid.
    pub levels: usize,
    pub seed: i64,
}

impl Default for MorrisPlan {
    fn default() -> MorrisPlan {
        MorrisPlan {
            space: InputSpace {
                names: Vec::new(),
                marginals: Vec::new(),
                bounds: Vec::new(),
            },
            trajectories: 20,
            levels: 4,
            seed: 0,
        }
    }
}

/// Morris elementary-effects screening.
///
/// # Errors
///
/// [`FreesError::Solver`] for an invalid input space, fewer than 2
/// trajectories, an odd or too-small level count, or a design where the model
/// failed everywhere.
pub fn morris<F, B>(
    plan: &MorrisPlan,
    mut evaluate: F,
    mut expired: B,
) -> Result<(Vec<MorrisOutcome>, SensitivityDiagnostics)>
where
    F: FnMut(&[f64]) -> Option<BTreeMap<String, f64>>,
    B: FnMut() -> bool,
{
    plan.space.validate()?;
    if plan.trajectories < 2 {
        return Err(FreesError::solver(
            "A Morris screening needs at least 2 trajectories.",
        ));
    }
    if plan.levels < 4 || plan.levels % 2 != 0 {
        return Err(FreesError::solver(
            "Morris levels must be an even number of at least 4.",
        ));
    }
    let p = plan.space.dimension();
    let levels = plan.levels as f64;
    let delta = levels / (2.0 * (levels - 1.0));
    let mut rng = super::montecarlo::JavaRandom::new(plan.seed);

    // effects[output][input] — accumulated across trajectories.
    let mut effects: BTreeMap<String, Vec<Vec<f64>>> = BTreeMap::new();
    let mut evaluations = 0usize;
    let mut complete = true;
    let mut dropped = 0usize;
    let mut used = 0usize;

    'trajectories: for _ in 0..plan.trajectories {
        // A base point on the grid, low enough that every +delta stays inside.
        let grid_max = (plan.levels / 2) as f64;
        let mut current: Vec<f64> = (0..p)
            .map(|_| (rng.next_double() * grid_max).floor() / (levels - 1.0))
            .collect();
        let mut order: Vec<usize> = (0..p).collect();
        for i in (1..p).rev() {
            let j = (rng.next_double() * (i + 1) as f64) as usize;
            order.swap(i, j.min(i));
        }

        if expired() {
            complete = false;
            break;
        }
        let mut previous = evaluate(&plan.space.to_values(&current));
        evaluations += 1;

        for &input in &order {
            if expired() {
                complete = false;
                break 'trajectories;
            }
            current[input] += delta;
            let next = evaluate(&plan.space.to_values(&current));
            evaluations += 1;
            match (&previous, &next) {
                (Some(before), Some(after)) => {
                    used += 1;
                    for (name, value) in after {
                        if let Some(base) = before.get(name) {
                            effects
                                .entry(name.clone())
                                .or_insert_with(|| vec![Vec::new(); p])[input]
                                .push((value - base) / delta);
                        }
                    }
                }
                // One end of the step failed, so this elementary effect does
                // not exist. The trajectory continues from the new point.
                _ => dropped += 1,
            }
            previous = next;
        }
    }

    if effects.is_empty() {
        return Err(FreesError::solver(
            "The Morris design produced no usable elementary effects — the model \
             failed at almost every point.",
        ));
    }

    let outcomes = effects
        .into_iter()
        .map(|(output, per_input)| MorrisOutcome {
            output,
            effects: per_input
                .into_iter()
                .enumerate()
                .map(|(i, ee)| {
                    let count = ee.len();
                    let (mu, mu_star, sigma) = if count == 0 {
                        (f64::NAN, f64::NAN, f64::NAN)
                    } else {
                        let n = count as f64;
                        let mu = ee.iter().sum::<f64>() / n;
                        let mu_star = ee.iter().map(|v| v.abs()).sum::<f64>() / n;
                        let sigma = if count < 2 {
                            f64::NAN
                        } else {
                            (ee.iter().map(|v| (v - mu).powi(2)).sum::<f64>() / (n - 1.0)).sqrt()
                        };
                        (mu, mu_star, sigma)
                    };
                    MorrisEffect {
                        source: plan.space.names[i].clone(),
                        mu,
                        mu_star,
                        sigma,
                        samples: count,
                    }
                })
                .collect(),
        })
        .collect();

    Ok((
        outcomes,
        SensitivityDiagnostics {
            design: "morris",
            evaluations,
            dropped_rows: dropped,
            used_rows: used,
            complete,
        },
    ))
}

// ── Driving a document ──────────────────────────────────────────────────────

/// Build the input space from the document's own uncertainty declarations.
///
/// A source with a `DistributionOf` declaration contributes that shape; one
/// with only a `±` contributes a normal centred on its base value. Bounds come
/// from the variable's declared limits and truncate the marginal, so a global
/// analysis never steps a flow rate negative just because its sigma is wide.
///
/// # Errors
///
/// [`FreesError::Solver`] when the document declares correlations — the
/// variance decomposition assumes independence and there is no honest way to
/// report indices without it — or when nothing declares an uncertainty.
pub fn input_space_from_document(
    sources: &[String],
    base_values: &BTreeMap<String, f64>,
    specs: &BTreeMap<String, super::uncertainty::UncertaintySpec>,
    distributions: &BTreeMap<String, Distribution>,
    correlations: &super::sampling::CorrelationEntries,
) -> Result<InputSpace> {
    if !correlations.is_empty() {
        return Err(FreesError::solver(
            "Global sensitivity indices assume independent inputs, and this document              declares Correlation(...). Remove the correlations for a sensitivity run,              or use the Monte Carlo propagation, which does honour them.",
        ));
    }
    if sources.is_empty() {
        return Err(FreesError::solver(
            "A sensitivity analysis needs at least one variable with a declared              uncertainty or distribution.",
        ));
    }
    let mut space = InputSpace {
        names: sources.to_vec(),
        marginals: Vec::with_capacity(sources.len()),
        bounds: Vec::with_capacity(sources.len()),
    };
    for name in sources {
        let spec = specs.get(name).copied().unwrap_or_default();
        let marginal = distributions
            .get(name)
            .copied()
            .unwrap_or(Distribution::Normal {
                mean: base_values.get(name).copied().unwrap_or(0.0),
                sigma: spec.uncertainty,
            });
        marginal.validate()?;
        space.marginals.push(marginal);
        space.bounds.push((spec.lower, spec.upper));
    }
    Ok(space)
}

/// Turn a document into the model closure [`sobol`] and [`morris`] expect.
///
/// Each design point becomes a set of `name = value` override lines applied
/// through the same `apply_overrides` mechanism Monte Carlo uses — replace,
/// never append — and a failed solve becomes `None` rather than an error, so
/// one infeasible corner of the input space does not abandon the run.
pub fn document_evaluator<'a>(
    text: &'a str,
    settings: &'a crate::solver::SolverSettings,
    sample_specs: &'a [crate::engine::VariableOverride],
    extra_tables: &'a [crate::parser::defs::FunctionTableDef],
    names: &'a [String],
) -> impl FnMut(&[f64]) -> Option<BTreeMap<String, f64>> + 'a {
    move |point: &[f64]| {
        let overrides: Vec<String> = names
            .iter()
            .zip(point)
            .map(|(name, value)| format!("{name} = {}", super::montecarlo::to_plain_string(*value)))
            .collect();
        crate::engine::solve_with_tables(
            &super::montecarlo::apply_overrides(text, &overrides),
            settings,
            sample_specs,
            extra_tables,
        )
        .ok()
        .map(|solution| solution.values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PI: f64 = std::f64::consts::PI;

    fn uniform_space(names: &[&str], lo: f64, hi: f64) -> InputSpace {
        InputSpace {
            names: names.iter().map(|s| (*s).to_string()).collect(),
            marginals: vec![Distribution::Uniform { lo, hi }; names.len()],
            bounds: vec![(f64::NEG_INFINITY, f64::INFINITY); names.len()],
        }
    }

    /// The Ishigami function — the standard Sobol' benchmark, chosen because
    /// its indices are known in closed form: `x3` has **zero** first-order
    /// effect but a large total, which is exactly the case a first-order-only
    /// or a local method gets wrong.
    fn ishigami(x: &[f64]) -> Option<BTreeMap<String, f64>> {
        let (a, b) = (7.0, 0.1);
        let y = libm::sin(x[0]) + a * libm::sin(x[1]).powi(2) + b * x[2].powi(4) * libm::sin(x[0]);
        Some(BTreeMap::from([("y".to_string(), y)]))
    }

    /// Closed-form Ishigami indices for a = 7, b = 0.1.
    fn ishigami_reference() -> (Vec<f64>, Vec<f64>) {
        let (a, b) = (7.0f64, 0.1f64);
        let pi4 = PI.powi(4);
        let pi8 = PI.powi(8);
        let v1 = 0.5 * (1.0 + b * pi4 / 5.0).powi(2);
        let v2 = a * a / 8.0;
        let v13 = 8.0 * b * b * pi8 / 225.0;
        let v = v2 + b * pi4 / 5.0 + b * b * pi8 / 18.0 + 0.5;
        (
            vec![v1 / v, v2 / v, 0.0],
            vec![(v1 + v13) / v, v2 / v, v13 / v],
        )
    }

    #[test]
    fn sobol_indices_match_the_ishigami_closed_form() {
        let plan = SobolPlan {
            space: uniform_space(&["x1", "x2", "x3"], -PI, PI),
            base_samples: 8192,
            seed: 3,
            design: Design::Sobol,
            bootstrap: 0,
        };
        let (outcomes, diag) = sobol(&plan, ishigami, || false).unwrap();
        assert_eq!(outcomes.len(), 1);
        let outcome = &outcomes[0];
        assert_eq!(outcome.output, "y");
        assert_eq!(diag.evaluations, 8192 * 5);
        assert_eq!(diag.dropped_rows, 0);
        assert!(diag.complete);

        let (want_first, want_total) = ishigami_reference();
        // The exact total variance, not just the ratios.
        let (a, b) = (7.0f64, 0.1f64);
        let v = a * a / 8.0 + b * PI.powi(4) / 5.0 + b * b * PI.powi(8) / 18.0 + 0.5;
        assert!(
            (outcome.variance - v).abs() < 0.15,
            "variance {} vs {v}",
            outcome.variance
        );
        for (i, index) in outcome.indices.iter().enumerate() {
            assert!(
                (index.first_order - want_first[i]).abs() < 0.01,
                "{}: S = {}, wanted {}",
                index.source,
                index.first_order,
                want_first[i]
            );
            assert!(
                (index.total - want_total[i]).abs() < 0.01,
                "{}: ST = {}, wanted {}",
                index.source,
                index.total,
                want_total[i]
            );
        }
        // The property that makes this benchmark worth using: x3 does nothing
        // on its own but a great deal in interaction with x1.
        assert!(outcome.indices[2].first_order.abs() < 0.02);
        assert!(outcome.indices[2].total > 0.2);
        // No bootstrap requested, so no error bars are invented.
        assert!(outcome.indices[0].first_order_se.is_nan());
    }

    #[test]
    fn bootstrap_errors_shrink_as_the_design_grows() {
        let mut plan = SobolPlan {
            space: uniform_space(&["x1", "x2", "x3"], -PI, PI),
            base_samples: 256,
            seed: 3,
            design: Design::Sobol,
            bootstrap: 100,
        };
        let small = sobol(&plan, ishigami, || false).unwrap().0[0].indices[0].first_order_se;
        plan.base_samples = 2048;
        let large = sobol(&plan, ishigami, || false).unwrap().0[0].indices[0].first_order_se;
        assert!(small.is_finite() && large.is_finite());
        assert!(large < small, "SE grew: {small} -> {large}");
    }

    #[test]
    fn a_purely_additive_model_has_first_order_indices_that_sum_to_one() {
        // y = 3·x1 + 1·x2, both U(0, 1): variances 9/12 and 1/12.
        let plan = SobolPlan {
            space: uniform_space(&["x1", "x2"], 0.0, 1.0),
            base_samples: 4096,
            seed: 11,
            design: Design::Sobol,
            bootstrap: 0,
        };
        let model = |x: &[f64]| Some(BTreeMap::from([("y".to_string(), 3.0 * x[0] + x[1])]));
        let outcome = &sobol(&plan, model, || false).unwrap().0[0];
        let sum: f64 = outcome.indices.iter().map(|i| i.first_order).sum();
        assert!((sum - 1.0).abs() < 0.01, "first-order sum {sum}");
        assert!((outcome.indices[0].first_order - 0.9).abs() < 0.01);
        // With no interaction, total equals first order.
        for index in &outcome.indices {
            assert!(
                (index.total - index.first_order).abs() < 0.01,
                "{}: ST {} vs S {}",
                index.source,
                index.total,
                index.first_order
            );
        }
    }

    #[test]
    fn a_constant_output_reports_nan_indices_not_zero() {
        let plan = SobolPlan {
            space: uniform_space(&["x1", "x2"], 0.0, 1.0),
            base_samples: 64,
            seed: 1,
            design: Design::Sobol,
            bootstrap: 0,
        };
        let outcome = &sobol(
            &plan,
            |_| Some(BTreeMap::from([("y".into(), 5.0)])),
            || false,
        )
        .unwrap()
        .0[0];
        assert_eq!(outcome.variance, 0.0);
        assert!(outcome.indices.iter().all(|i| i.first_order.is_nan()));
    }

    #[test]
    fn failed_points_drop_whole_rows_and_are_counted() {
        let plan = SobolPlan {
            space: uniform_space(&["x1", "x2"], 0.0, 1.0),
            base_samples: 128,
            seed: 5,
            design: Design::Sobol,
            bootstrap: 0,
        };
        let mut calls = 0usize;
        // Fail on a scattered subset so several rows lose one evaluation each.
        let model = |x: &[f64]| {
            calls += 1;
            if calls % 37 == 0 {
                None
            } else {
                Some(BTreeMap::from([("y".to_string(), x[0] + 2.0 * x[1])]))
            }
        };
        let (outcomes, diag) = sobol(&plan, model, || false).unwrap();
        assert!(diag.dropped_rows > 0, "expected some rows to be dropped");
        assert_eq!(diag.used_rows + diag.dropped_rows, 128);
        // The surviving rows still give the right answer for a linear model.
        let sum: f64 = outcomes[0].indices.iter().map(|i| i.first_order).sum();
        assert!((sum - 1.0).abs() < 0.05, "first-order sum {sum}");
    }

    #[test]
    fn a_budget_that_fires_truncates_the_design_and_says_so() {
        let plan = SobolPlan {
            space: uniform_space(&["x1", "x2"], 0.0, 1.0),
            base_samples: 512,
            seed: 5,
            design: Design::Sobol,
            bootstrap: 0,
        };
        let mut ticks = 0usize;
        let (_, diag) = sobol(
            &plan,
            |x: &[f64]| Some(BTreeMap::from([("y".to_string(), x[0] + x[1])])),
            || {
                ticks += 1;
                ticks > 200
            },
        )
        .unwrap();
        assert!(!diag.complete);
        assert!(diag.used_rows < 512);
    }

    #[test]
    fn sobol_refuses_a_degenerate_request() {
        let mut plan = SobolPlan {
            space: uniform_space(&["x1"], 0.0, 1.0),
            base_samples: 1,
            ..SobolPlan::default()
        };
        assert!(sobol(&plan, |_| Some(BTreeMap::new()), || false).is_err());
        plan.base_samples = 16;
        // A model that always fails cannot be decomposed.
        assert!(sobol(&plan, |_| None, || false)
            .unwrap_err()
            .to_string()
            .contains("fewer than two complete rows"));
    }

    #[test]
    fn morris_ranks_the_inputs_the_way_their_coefficients_do() {
        // y = 10·x1 + 1·x2 + 0·x3 over the unit cube.
        let plan = MorrisPlan {
            space: uniform_space(&["x1", "x2", "x3"], 0.0, 1.0),
            trajectories: 40,
            levels: 4,
            seed: 9,
        };
        let model = |x: &[f64]| Some(BTreeMap::from([("y".to_string(), 10.0 * x[0] + x[1])]));
        let (outcomes, diag) = morris(&plan, model, || false).unwrap();
        assert_eq!(diag.evaluations, 40 * 4);
        assert!(diag.complete);
        let effects = &outcomes[0].effects;
        // A linear model has an exactly constant elementary effect, so sigma is
        // zero and mu* equals the coefficient.
        assert!((effects[0].mu_star - 10.0).abs() < 1e-9, "{:?}", effects[0]);
        assert!((effects[1].mu_star - 1.0).abs() < 1e-9, "{:?}", effects[1]);
        assert!(effects[2].mu_star.abs() < 1e-9, "{:?}", effects[2]);
        for effect in effects {
            assert!(effect.sigma.abs() < 1e-9, "{effect:?}");
        }
    }

    #[test]
    fn morris_separates_a_sign_flipping_input_from_an_inert_one() {
        // y = (x1 − 0.5)·x2·20: x2's effect flips sign with x1, so its mu is
        // near zero while its mu* and sigma are large. x3 does nothing at all.
        let plan = MorrisPlan {
            space: uniform_space(&["x1", "x2", "x3"], 0.0, 1.0),
            trajectories: 80,
            levels: 6,
            seed: 4,
        };
        let model = |x: &[f64]| {
            Some(BTreeMap::from([(
                "y".to_string(),
                (x[0] - 0.5) * x[1] * 20.0,
            )]))
        };
        let effects = &morris(&plan, model, || false).unwrap().0[0].effects;
        let x2 = &effects[1];
        let x3 = &effects[2];
        assert!(x2.mu_star > 1.0, "x2 mu* {}", x2.mu_star);
        assert!(
            x2.mu.abs() < x2.mu_star,
            "x2 mu {} vs mu* {}",
            x2.mu,
            x2.mu_star
        );
        assert!(x2.sigma > 1.0, "x2 sigma {}", x2.sigma);
        assert_eq!(x3.mu_star, 0.0, "x3 should be inert");
        assert_eq!(x3.sigma, 0.0);
    }

    #[test]
    fn morris_refuses_an_odd_or_tiny_level_count() {
        let plan = MorrisPlan {
            space: uniform_space(&["x1"], 0.0, 1.0),
            trajectories: 10,
            levels: 5,
            seed: 1,
        };
        assert!(morris(&plan, |_| Some(BTreeMap::new()), || false)
            .unwrap_err()
            .to_string()
            .contains("even number of at least 4"));
    }
}
