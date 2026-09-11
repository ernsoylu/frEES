//! Parameter estimation against measured data — calibration.
//!
//! Port of
//! `../frEES/backend/core/src/main/java/com/frees/backend/api/ParameterFit.java`
//! (298 LOC), plus the `measurement/SampledSeries` resampling it reduces
//! residuals through and Commons Math's `BrentOptimizer` for the
//! one-parameter case.
//!
//! # What a fit does
//!
//! Each objective evaluation replaces the parameters' defining assignments
//! (the terminal's override mechanism — replace, never append; see
//! [`crate::analysis::montecarlo::apply_overrides`]) and re-solves the
//! document. The named `DYNAMIC` block's column is linearly resampled onto the
//! measured raster and reduced to a sum of squared residuals, **skipping pairs
//! outside the model's own time span** — the compare view's semantics, and the
//! reason `pairs` is tracked separately from the raster length.
//!
//! The optimizer works on parameters normalized to the unit box. That is not
//! cosmetic: it keeps the search geometry sane when one parameter is a `UA` in
//! the hundreds and another a dimensionless efficiency. A failed solve scores a
//! flat penalty, so an infeasible *starting* point is detected and reported up
//! front rather than silently returning the initial guess — from a flat
//! landscape every direction looks identical.
//!
//! # The multivariate optimizer is Nelder–Mead, not BOBYQA — stated plainly
//!
//! The Java uses `BOBYQAOptimizer(2n+1, 0.2, 1e-6)` for two or more parameters
//! and `BrentOptimizer(1e-8, 1e-10)` for one.
//!
//! * The **one-parameter** path is a faithful transcription of Commons Math's
//!   `BrentOptimizer`, so its evaluation sequence — and therefore the tracked
//!   best iterate and the reported `evaluations` — follows the Java's.
//! * The **multi-parameter** path is a bound-constrained Nelder–Mead simplex,
//!   not BOBYQA. Porting BOBYQA means porting 2,470 lines of Fortran-derived
//!   trust-region code, which is a project of its own. The substitution is not
//!   invented for this port: the parent engine's own `core/Optimizer.java`
//!   treats `SimplexOptimizer` (Nelder–Mead) as its *default* multivariate
//!   optimizer and BOBYQA as the opt-in alternative, so this is the reference
//!   implementation's own second choice. The Java's two trust-region knobs keep
//!   their meaning: [`INITIAL_RADIUS`] sizes the initial simplex and
//!   [`STOPPING_RADIUS`] is the convergence radius.
//!
//!   **Consequence, so nobody is surprised:** for two or more parameters the
//!   *optimum* should agree with the Java to within the stopping radius, but
//!   `evaluations` and the exact fitted vector will not match iterate for
//!   iterate. Do not build an iteration-sensitive golden on this path.
//!
//! # The solve seam
//!
//! [`run`] never calls the solver itself: it hands the overridden document text
//! to a caller-supplied closure that returns the document's ODE tables (or
//! `None` when the solve failed). `Solution` does not carry `ode_tables` yet —
//! transient integration is being ported alongside this — so the seam keeps
//! this module complete and testable today, and turns the wiring into an
//! adapter from whatever `crate::ode` settles on to [`OdeTableView`].

use super::curvefit::Loss;
use crate::analysis::montecarlo::apply_overrides;
use crate::diag::{FreesError, Result};

/// The flat score a failed solve receives. Port of `ParameterFit.PENALTY`.
const PENALTY: f64 = 1.0e100;
/// Initial trust-region size on the normalized unit box. Port of
/// `ParameterFit.INITIAL_RADIUS`; here it sizes the initial simplex.
pub const INITIAL_RADIUS: f64 = 0.2;
/// Terminal trust-region size on the normalized unit box. Port of
/// `ParameterFit.STOPPING_RADIUS`.
pub const STOPPING_RADIUS: f64 = 1.0e-6;

/// A `(t, v)` series pair. Port of `ParameterFit.Series`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Series {
    pub t: Vec<f64>,
    pub v: Vec<f64>,
}

/// One solved `DYNAMIC` block's table, in the shape this module needs: the
/// block name, the column headers (column 0 is time) and the rows, with `None`
/// for a cell the integrator could not fill.
///
/// A minimal view of the Java `core.ode.OdeTableResult`, deliberately owned by
/// this module so parameter estimation does not have to wait on the transient
/// port to land its own result type.
#[derive(Debug, Clone, PartialEq)]
pub struct OdeTableView {
    pub name: String,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<Option<f64>>>,
}

/// Port of `ParameterFit.Outcome`.
#[derive(Debug, Clone, PartialEq)]
pub struct Outcome {
    pub parameters: Vec<String>,
    pub fitted: Vec<f64>,
    /// RMSE at the fitted point, over the pairs that actually overlapped.
    pub rmse: f64,
    /// RMSE at the initial point, for the "did this help?" comparison.
    pub initial_rmse: f64,
    pub evaluations: usize,
    /// True when the budget struck mid-search and the best iterate so far
    /// stands.
    pub truncated: bool,
    /// The fitted model resampled onto the measured raster, for overlay.
    pub fitted_series: Series,
    /// Phase 4.2: 1-sigma standard error per parameter, or `NaN` where the data
    /// cannot separate it. Never a plausible-looking number for an
    /// unidentifiable parameter.
    pub std_errors: Vec<f64>,
    /// Full parameter covariance, or empty when it could not be formed.
    pub covariance: Vec<Vec<f64>>,
    /// Residual degrees of freedom: compared pairs minus fitted parameters.
    pub residual_dof: usize,
    /// Numerical rank of the Jacobian at the optimum. Below the parameter count
    /// means the parameters are not separately identifiable from this data.
    pub rank: usize,
    /// Condition number of that Jacobian.
    pub condition_number: f64,
    /// True when the rank is deficient — the diagnostics above are then `NaN`.
    pub unidentifiable: bool,
    /// Reduced chi-square, only meaningful for a weighted fit (it is `NaN`
    /// otherwise: without σ there is nothing to compare the scatter against).
    pub reduced_chi_square: f64,
    /// Per parameter, whether the fit came to rest on one of its own bounds. A
    /// parameter at a bound has not been estimated by the data, and its standard
    /// error describes the wrong thing.
    pub at_bound: Vec<bool>,
    /// Weighted chi-square at the optimum, `Σ ((model − measured)/σ)²`. Equals
    /// the plain SSE when no σ was supplied.
    pub chi_square: f64,
}

/// Everything a fit needs about the problem. Bundled because the Java's `run`
/// takes fourteen arguments and a Rust function with that many is unreadable
/// (and trips `clippy::too_many_arguments`).
///
/// [`Default`] so a caller names only the fields it uses — Phase 4.2 added
/// three optional ones and every existing call site should keep reading as it
/// did. The default is the historical fit: unweighted, ordinary least squares.
#[derive(Debug, Clone, Default)]
pub struct FitRequest<'a> {
    /// The document exactly as the solve endpoints receive it.
    pub text: &'a str,
    /// Parameters to calibrate, lowercase.
    pub parameters: &'a [String],
    pub initial: &'a [f64],
    pub lower: &'a [f64],
    pub upper: &'a [f64],
    /// The `DYNAMIC` block whose table carries the compared column.
    pub ode_block: &'a str,
    /// The column within that table, matched case-insensitively.
    pub column: &'a str,
    pub measured_t: &'a [f64],
    pub measured_v: &'a [f64],
    pub max_evaluations: usize,
    /// Phase 4.2: per-measurement standard deviations. Empty means unweighted —
    /// every point counted equally, which is what this fit always did. A σ of
    /// zero or non-finite is rejected rather than silently treated as 1.
    pub sigma: &'a [f64],
    /// Robust loss applied to the scaled squared residual, exactly as in
    /// [`crate::analysis::curvefit`]. [`Loss::Linear`] is the historical
    /// behaviour and costs nothing.
    pub loss: Loss,
    /// The residual scale beyond which a robust loss starts discounting a
    /// point. Ignored for [`Loss::Linear`]; non-positive falls back to 1.
    pub f_scale: f64,
}

/// Why the search stopped early. Both are caught inside [`run`], exactly as the
/// Java catches `TooManyEvaluationsException` and its private `BudgetExhausted`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stop {
    /// The wall-clock budget struck — the Java `BudgetExhausted`.
    Budget,
    /// `MaxEval` exhausted — the Java `TooManyEvaluationsException`.
    Evaluations,
}

/// One objective evaluation's result. Port of `ParameterFit.SseResult`,
/// extended in Phase 4.2 with the weighted and robust sums and with the record
/// of *which* measured points actually took part — the Jacobian at the optimum
/// has to be built over exactly those rows and no others.
struct SseResult {
    /// Unweighted `Σ (model − measured)²`, which is what `rmse` reports.
    sse: f64,
    /// `Σ ((model − measured)/σ)²`.
    chi_square: f64,
    /// What the minimiser actually descends: the robust loss applied to the
    /// scaled residuals, `Σ f_scale²·ρ(z)`. Identical to `chi_square` for
    /// [`Loss::Linear`].
    objective: f64,
    pairs: usize,
    model: SampledSeries,
    /// Indices into `measured_t` that produced a comparison, ascending.
    used: Vec<usize>,
}

/// Calibrates `request.parameters` so the model column matches the measurement.
/// Port of `ParameterFit.run`.
///
/// * `solve` receives the **overridden** document text and returns its ODE
///   tables, or `None` when the solve failed — the Java's
///   `catch (ParseException | SolverException | IllegalStateException)`.
/// * `expired` is the wall-clock budget predicate (core has no clock on
///   `wasm32-unknown-unknown`); pass `|| false` for an unbounded run.
///
/// # Errors
///
/// [`FreesError::Solver`] for a malformed request (no parameters, mismatched
/// array lengths, non-finite or crossed bounds, an initial value outside its
/// bounds, fewer than two measured samples), for an infeasible starting point,
/// or when the fitted point stops solving.
pub fn run<S, B>(request: &FitRequest<'_>, mut solve: S, mut expired: B) -> Result<Outcome>
where
    S: FnMut(&str) -> Option<Vec<OdeTableView>>,
    B: FnMut() -> bool,
{
    let FitRequest {
        text,
        parameters,
        initial,
        lower,
        upper,
        ode_block,
        column,
        measured_t,
        measured_v,
        max_evaluations,
        sigma,
        loss,
        f_scale,
    } = *request;
    let weighting = Weighting {
        sigma,
        loss,
        f_scale: if f_scale.is_finite() && f_scale > 0.0 {
            f_scale
        } else {
            1.0
        },
    };

    let n = parameters.len();
    if n == 0 || initial.len() != n || lower.len() != n || upper.len() != n {
        return Err(FreesError::solver(
            "Parameter estimation needs at least one parameter, each with an initial \
             value and finite lower/upper bounds.",
        ));
    }
    for i in 0..n {
        if !lower[i].is_finite() || !upper[i].is_finite() || lower[i] >= upper[i] {
            return Err(FreesError::solver(format!(
                "Bounds for {} must be finite with lower < upper.",
                parameters[i]
            )));
        }
        if initial[i] < lower[i] || initial[i] > upper[i] {
            return Err(FreesError::solver(format!(
                "The initial value of {} lies outside its bounds.",
                parameters[i]
            )));
        }
    }
    if measured_t.len() < 2 || measured_t.len() != measured_v.len() {
        return Err(FreesError::solver(
            "The measured series needs at least two (t, y) samples.",
        ));
    }
    if !sigma.is_empty() {
        if sigma.len() != measured_t.len() {
            return Err(FreesError::solver(
                "One measurement standard deviation is needed per measured sample.",
            ));
        }
        if let Some(bad) = sigma.iter().find(|s| !(s.is_finite() && **s > 0.0)) {
            return Err(FreesError::solver(format!(
                "Measurement standard deviations must be finite and positive (got {bad})."
            )));
        }
    }

    let mut evaluations = 0usize;
    let mut best_point = initial.to_vec();
    // The tracked best is by the **minimised** objective, which is the weighted
    // and robustly-discounted sum — not the raw SSE. Tracking the raw sum here
    // would let a run report a point the optimiser never preferred.
    let mut best_sse = f64::INFINITY;
    let mut best_pairs = 0usize;

    // The normalized objective, closing over the trackers exactly as the Java's
    // `MultivariateFunction normalized` closes over its one-element arrays.
    // `raw_sse` carries the *unweighted* sum of the evaluation just made, so the
    // initial RMSE can be read off the feasibility probe instead of paying for
    // a second solve of the same point.
    let mut normalized = |unit: &[f64],
                          evaluations: &mut usize,
                          best_point: &mut Vec<f64>,
                          best_sse: &mut f64,
                          best_pairs: &mut usize,
                          raw_sse: &mut f64|
     -> std::result::Result<f64, Stop> {
        if expired() {
            return Err(Stop::Budget);
        }
        let p: Vec<f64> = (0..n)
            .map(|i| {
                let u = clamp(unit[i], 0.0, 1.0);
                lower[i] + u * (upper[i] - lower[i])
            })
            .collect();
        *evaluations += 1;
        match evaluate(
            text, parameters, &p, ode_block, column, measured_t, measured_v, &weighting, &mut solve,
        ) {
            None => {
                *raw_sse = f64::INFINITY;
                Ok(PENALTY)
            }
            Some(r) => {
                *raw_sse = r.sse;
                if r.objective < *best_sse {
                    *best_sse = r.objective;
                    *best_pairs = r.pairs;
                    best_point.clone_from(&p);
                }
                Ok(r.objective)
            }
        }
    };

    // Validate the starting point before handing the landscape to the
    // optimizer: from an infeasible start every direction scores the same flat
    // penalty and the fit would silently return the initial guess.
    let unit0: Vec<f64> = (0..n)
        .map(|i| (initial[i] - lower[i]) / (upper[i] - lower[i]))
        .collect();
    let mut raw_sse = f64::INFINITY;
    let sse0 = normalized(
        &unit0,
        &mut evaluations,
        &mut best_point,
        &mut best_sse,
        &mut best_pairs,
        &mut raw_sse,
    )
    .map_err(|_| {
        FreesError::solver(
            "Parameter estimation ran out of budget before its first evaluation finished.",
        )
    })?;
    if sse0 >= PENALTY {
        return Err(FreesError::solver(
            "The model does not solve (or the target column has no overlap with the \
             measurement) at the initial parameter values. Fix the starting point first — \
             a fit cannot navigate out of an infeasible start.",
        ));
    }
    // RMSE is reported unweighted — a physical residual in the column's own
    // units — so it comes off the probe's raw SSE, not off the objective the
    // optimiser saw. For an unweighted linear fit the two are the same number.
    let initial_rmse = (raw_sse / best_pairs.max(1) as f64).sqrt();

    // `MaxEval` is the *optimizer's* budget in Java: the feasibility probe above
    // runs outside it (it is a direct `normalized.value(unit0)` call, not a
    // `computeObjectiveValue`), while `evaluations[0]` counts it. Commons Math's
    // `Incrementor` throws on the call that would exceed the maximum, so exactly
    // `max_evaluations` optimizer evaluations happen and the reported count is
    // one more than that.
    let mut optimizer_calls = 0usize;
    let mut objective = |unit: &[f64]| -> std::result::Result<f64, Stop> {
        optimizer_calls += 1;
        if optimizer_calls > max_evaluations {
            return Err(Stop::Evaluations);
        }
        normalized(
            unit,
            &mut evaluations,
            &mut best_point,
            &mut best_sse,
            &mut best_pairs,
            &mut raw_sse,
        )
    };

    let stop = if n == 1 {
        brent_minimize(&mut objective, 0.0, 1.0, unit0[0], 1.0e-8, 1.0e-10).err()
    } else {
        nelder_mead_unit_box(&mut objective, &unit0).err()
    };
    // `TooManyEvaluationsException`: the budget is the point, the best tracked
    // iterate stands. `BudgetExhausted`: same, and the outcome says so.
    let truncated = stop == Some(Stop::Budget);

    // Confirming solve at the best point: the fitted series on the measured
    // raster, for overlay and honest reporting. Deliberately *not* counted as
    // an evaluation — the Java calls `evaluate` directly here.
    let confirm = evaluate(
        text,
        parameters,
        &best_point,
        ode_block,
        column,
        measured_t,
        measured_v,
        &weighting,
        &mut solve,
    )
    .ok_or_else(|| {
        FreesError::solver(
            "The fitted point no longer solves — this indicates a sensitive model; \
             tighten the bounds around a feasible region.",
        )
    })?;
    let rmse = (confirm.sse / confirm.pairs.max(1) as f64).sqrt();

    // Phase 4.2: the diagnostics. The Jacobian at the optimum costs one extra
    // solve per parameter — the same price the fit paid for every simplex
    // vertex, and the only way to say anything about identifiability.
    // "At a bound" is judged against the optimiser's own resolution, not against
    // exact equality: Brent and Nelder-Mead converge to a relative 1e-8 and stop
    // a few ulps short of the box edge, so an exact test would report every
    // pinned parameter as free.
    let at_bound: Vec<bool> = (0..n)
        .map(|i| {
            let tol = 1e-6 * (upper[i] - lower[i]);
            best_point[i] - lower[i] <= tol || upper[i] - best_point[i] <= tol
        })
        .collect();
    // The scaled residual and its robust weight at each compared point, computed
    // once: the Jacobian scales its rows by the weight and the residual sum
    // squares the weighted residual, and the two must agree.
    let scaled: Vec<(f64, f64)> = confirm
        .used
        .iter()
        .map(|&k| {
            let u = (confirm.model.at(measured_t[k]) - measured_v[k]) / weighting.sigma_at(k);
            (u, weighting.jacobian_weight(u))
        })
        .collect();
    let row_weights: Vec<f64> = scaled.iter().map(|(_, w)| *w).collect();
    let robust_sum: f64 = scaled.iter().map(|(u, w)| (u * w) * (u * w)).sum();
    let jacobian = optimum_jacobian(
        text,
        parameters,
        &best_point,
        lower,
        upper,
        ode_block,
        column,
        measured_t,
        &confirm,
        &weighting,
        &row_weights,
        &mut solve,
    );
    let uncertainty = super::curvefit::parameter_uncertainty(
        &jacobian,
        n,
        confirm.pairs,
        robust_sum,
        weighting.is_weighted(),
    );

    Ok(Outcome {
        parameters: parameters.to_vec(),
        fitted: best_point,
        rmse,
        initial_rmse,
        evaluations,
        truncated,
        fitted_series: Series {
            t: measured_t.to_vec(),
            v: confirm.model.sample_on(measured_t),
        },
        std_errors: uncertainty.std_errors,
        covariance: uncertainty.covariance,
        residual_dof: uncertainty.dof,
        rank: uncertainty.rank,
        condition_number: uncertainty.condition_number,
        unidentifiable: uncertainty.unidentifiable,
        reduced_chi_square: uncertainty.reduced_chi_square,
        at_bound,
        chi_square: confirm.chi_square,
    })
}

/// Forward-difference Jacobian of the σ-scaled model response at the optimum,
/// one row per compared point and one column per parameter.
///
/// The step is relative to the parameter's own **bounded span**, not to its
/// magnitude: a calibrated parameter is often near zero inside a wide range, and
/// a magnitude-relative step there is either noise or nothing. A parameter whose
/// perturbed solve fails leaves its column zero, which the rank test then
/// reports as unidentifiable — the honest answer, since a model that will not
/// solve one step away tells the fit nothing about that direction.
#[allow(clippy::too_many_arguments)]
fn optimum_jacobian<S>(
    text: &str,
    parameters: &[String],
    point: &[f64],
    lower: &[f64],
    upper: &[f64],
    ode_block: &str,
    column: &str,
    measured_t: &[f64],
    base: &SseResult,
    weighting: &Weighting<'_>,
    row_weights: &[f64],
    solve: &mut S,
) -> Vec<Vec<f64>>
where
    S: FnMut(&str) -> Option<Vec<OdeTableView>>,
{
    let n = parameters.len();
    let rows = base.used.len();
    let mut jacobian = vec![vec![0.0; n]; rows];
    for i in 0..n {
        let span = upper[i] - lower[i];
        let mut h = 1e-6 * span;
        // Step inward when the optimum sits on the upper bound, so the probe
        // stays inside the feasible box.
        if point[i] + h > upper[i] {
            h = -h;
        }
        if h == 0.0 {
            continue;
        }
        let mut probe = point.to_vec();
        probe[i] += h;
        let overrides: Vec<String> = parameters
            .iter()
            .zip(&probe)
            .map(|(name, value)| format!("{name} = {}", plain(*value)))
            .collect();
        let Some(tables) = solve(&apply_overrides(text, &overrides)) else {
            continue;
        };
        let Some(series) = extract_column(&tables, ode_block, column) else {
            continue;
        };
        let perturbed = SampledSeries::linear(series.t, series.v);
        for (row, &k) in base.used.iter().enumerate() {
            let t = measured_t[k];
            let (a, b) = (base.model.at(t), perturbed.at(t));
            if a.is_nan() || b.is_nan() {
                continue;
            }
            let sigma = weighting.sigma_at(k);
            jacobian[row][i] = (b - a) / (h * sigma);
        }
    }
    // Fold the robust weights in, so a point the loss discounted contributes
    // less to the parameter uncertainty than a fully trusted one.
    for (row, &w) in row_weights.iter().enumerate().take(rows) {
        if w != 1.0 {
            for value in &mut jacobian[row] {
                *value *= w;
            }
        }
    }
    jacobian
}

/// One objective evaluation; `None` = solve failed or no usable overlap. Port
/// of `ParameterFit.evaluate`.
#[allow(clippy::too_many_arguments)] // Mirrors the Java's own parameter list.
fn evaluate<S>(
    text: &str,
    parameters: &[String],
    values: &[f64],
    ode_block: &str,
    column: &str,
    measured_t: &[f64],
    measured_v: &[f64],
    weighting: &Weighting<'_>,
    solve: &mut S,
) -> Option<SseResult>
where
    S: FnMut(&str) -> Option<Vec<OdeTableView>>,
{
    let overrides: Vec<String> = parameters
        .iter()
        .zip(values)
        .map(|(name, value)| format!("{name} = {}", plain(*value)))
        .collect();
    let tables = solve(&apply_overrides(text, &overrides))?;
    let series = extract_column(&tables, ode_block, column)?;
    let model = SampledSeries::linear(series.t, series.v);

    let mut sse = 0.0;
    let mut chi_square = 0.0;
    let mut objective = 0.0;
    let mut used = Vec::new();
    for (k, (t, m)) in measured_t.iter().zip(measured_v).enumerate() {
        if m.is_nan() {
            continue;
        }
        let s = model.at(*t);
        if s.is_nan() {
            // Outside the model's span, or a gap — not a comparison.
            continue;
        }
        let e = s - m;
        let u = e / weighting.sigma_at(k);
        sse += e * e;
        chi_square += u * u;
        objective += weighting.penalty(u);
        used.push(k);
    }
    if used.is_empty() {
        return None;
    }
    Some(SseResult {
        sse,
        chi_square,
        objective,
        pairs: used.len(),
        model,
        used,
    })
}

/// The per-point σ and the robust loss, together, because every residual passes
/// through both and separating them would mean two parallel lookups in the hot
/// loop.
#[derive(Debug, Clone, Copy)]
struct Weighting<'a> {
    sigma: &'a [f64],
    loss: Loss,
    f_scale: f64,
}

impl Weighting<'_> {
    /// The σ for measurement `k`, or 1 when the fit is unweighted. An empty
    /// `sigma` is the historical unweighted fit, bit for bit.
    fn sigma_at(&self, k: usize) -> f64 {
        self.sigma.get(k).copied().unwrap_or(1.0)
    }

    /// `f_scale²·ρ((u/f_scale)²)` — what the minimiser sums. For
    /// [`Loss::Linear`] this is exactly `u²`, with no extra arithmetic, so the
    /// unweighted linear fit descends the same surface it always did.
    fn penalty(&self, u: f64) -> f64 {
        if self.loss == Loss::Linear {
            return u * u;
        }
        let scale = self.f_scale;
        let z = (u / scale) * (u / scale);
        scale * scale * self.loss.value(z)
    }

    /// The IRLS weight the covariance Jacobian folds in, so a point the robust
    /// loss discounted also counts less towards the parameter uncertainty.
    fn jacobian_weight(&self, u: f64) -> f64 {
        if self.loss == Loss::Linear {
            return 1.0;
        }
        self.loss.weight(u / self.f_scale).sqrt()
    }

    fn is_weighted(&self) -> bool {
        !self.sigma.is_empty()
    }
}

/// The named `DYNAMIC` table's column as `(t, v)`; `None` when the table or the
/// column is absent. Port of `ParameterFit.extractColumn`, names matched
/// case-insensitively and a `null` cell becoming `NaN`.
pub fn extract_column(tables: &[OdeTableView], ode_block: &str, column: &str) -> Option<Series> {
    let table = tables
        .iter()
        .find(|t| t.name.eq_ignore_ascii_case(ode_block))?;
    let col = table
        .columns
        .iter()
        .position(|c| c.eq_ignore_ascii_case(column))?;
    let mut t = Vec::with_capacity(table.rows.len());
    let mut v = Vec::with_capacity(table.rows.len());
    for row in &table.rows {
        t.push(row.first().copied().flatten().unwrap_or(f64::NAN));
        v.push(row.get(col).copied().flatten().unwrap_or(f64::NAN));
    }
    Some(Series { t, v })
}

/// `BigDecimal.valueOf(v).toPlainString()`; see
/// [`crate::analysis::montecarlo`] for why the rendering must not use an
/// exponent.
fn plain(value: f64) -> String {
    format!("{value}")
}

/// `Math.clamp` (Java 21): `min(max(value, min), max)`, NaN in / NaN out.
fn clamp(value: f64, min: f64, max: f64) -> f64 {
    value.max(min).min(max)
}

// ---------------------------------------------------------------------------
// The Java's measurement/SampledSeries — the LINEAR half
// ---------------------------------------------------------------------------

/// A sampled series with linear interpolation. Port of the Java's
/// `measurement/SampledSeries` restricted to `Interp.LINEAR`, which is the mode
/// `ParameterFit` constructs. This crate has no measurement module of its own
/// — D11 removed it — so this is the only transcription of that class left.
///
/// Two behaviours are load-bearing and are transcribed rather than
/// approximated: the value **before the first sample is `NaN`** (there is
/// nothing to hold), and a gap — a `NaN` on either side of the bracket — stays
/// a gap instead of being bridged.
#[derive(Debug, Clone, PartialEq)]
pub struct SampledSeries {
    t: Vec<f64>,
    v: Vec<f64>,
}

impl SampledSeries {
    pub fn linear(t: Vec<f64>, v: Vec<f64>) -> SampledSeries {
        SampledSeries { t, v }
    }

    /// Value at time `x`. Port of `SampledSeries.at` for `Interp.LINEAR`.
    pub fn at(&self, x: f64) -> f64 {
        let n = self.t.len();
        if n == 0 {
            return f64::NAN;
        }
        let lb = lower_bound(&self.t, x);
        if lb < n && self.t[lb] == x {
            return self.v[lb];
        }
        if lb == 0 {
            // Nothing strictly before x.
            return f64::NAN;
        }
        let i = lb - 1;
        if lb >= n {
            // Past the last sample: LINEAR has nothing to interpolate towards,
            // so the Java falls through to holding the last value.
            return self.v[i];
        }
        let (t0, t1) = (self.t[i], self.t[lb]);
        let (v0, v1) = (self.v[i], self.v[lb]);
        if v0.is_nan() || v1.is_nan() {
            return f64::NAN;
        }
        v0 + (v1 - v0) * (x - t0) / (t1 - t0)
    }

    /// Materialize the series on an output raster. Port of
    /// `SampledSeries.sampleOn`.
    pub fn sample_on(&self, raster: &[f64]) -> Vec<f64> {
        raster.iter().map(|x| self.at(*x)).collect()
    }
}

/// First index whose value is `>= x`. Port of the Java's
/// `measurement/EnvelopeDecimator.lowerBound`.
fn lower_bound(a: &[f64], x: f64) -> usize {
    let (mut lo, mut hi) = (0usize, a.len());
    while lo < hi {
        let mid = (lo + hi) / 2;
        if a[mid] < x {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    lo
}

// ---------------------------------------------------------------------------
// Commons Math BrentOptimizer — the one-parameter path
// ---------------------------------------------------------------------------

/// Brent's derivative-free minimizer on `[lo, hi]` starting from `start`.
/// Transcription of `org.apache.commons.math3.optim.univariate.BrentOptimizer`
/// for `GoalType.MINIMIZE`.
///
/// The return value is the best `(point, value)` seen. `ParameterFit` ignores
/// it and reads its own tracked best iterate instead — but the *evaluation
/// sequence* is what feeds that tracker, which is why this is transcribed
/// rather than replaced by any other line search.
fn brent_minimize<F>(
    f: &mut F,
    lo: f64,
    hi: f64,
    start: f64,
    relative_threshold: f64,
    absolute_threshold: f64,
) -> std::result::Result<(f64, f64), Stop>
where
    F: FnMut(&[f64]) -> std::result::Result<f64, Stop>,
{
    // `0.5 * (3 - FastMath.sqrt(5))`, computed rather than written out: a
    // decimal literal for it lands a ulp away, and Brent's very first golden
    // section step then diverges from the reference by a ulp too. `sqrt` is
    // correctly rounded in IEEE-754 and the rest is exact, so this reproduces
    // Java's constant bit for bit.
    let golden_section = 0.5 * (3.0 - 5.0f64.sqrt());

    let (mut a, mut b) = if lo < hi { (lo, hi) } else { (hi, lo) };
    let mut x = start;
    let mut v = x;
    let mut w = x;
    let mut d = 0.0f64;
    let mut e = 0.0f64;
    let mut fx = f(&[x])?;
    let mut fv = fx;
    let mut fw = fx;

    let mut best = (x, fx);
    loop {
        let m = 0.5 * (a + b);
        let tol1 = relative_threshold * x.abs() + absolute_threshold;
        let tol2 = 2.0 * tol1;

        if (x - m).abs() <= tol2 - 0.5 * (b - a) {
            // Brent's own termination.
            return Ok(better(best, (x, fx)));
        }

        let mut u;
        if e.abs() > tol1 {
            // Fit a parabola through (v, fv), (w, fw), (x, fx).
            let mut r = (x - w) * (fx - fv);
            let mut q = (x - v) * (fx - fw);
            let mut p = (x - v) * q - (x - w) * r;
            q = 2.0 * (q - r);
            if q > 0.0 {
                p = -p;
            } else {
                q = -q;
            }
            r = e;
            e = d;
            if p > q * (a - x) && p < q * (b - x) && p.abs() < (0.5 * q * r).abs() {
                d = p / q;
                u = x + d;
                // f must not be evaluated too close to a or b.
                if u - a < tol2 || b - u < tol2 {
                    d = if x <= m { tol1 } else { -tol1 };
                }
            } else {
                e = if x < m { b - x } else { a - x };
                d = golden_section * e;
            }
        } else {
            e = if x < m { b - x } else { a - x };
            d = golden_section * e;
        }

        // Move by at least tol1.
        if d.abs() < tol1 {
            u = if d >= 0.0 { x + tol1 } else { x - tol1 };
        } else {
            u = x + d;
        }

        let fu = f(&[u])?;
        best = better(best, (u, fu));

        if fu <= fx {
            if u < x {
                b = x;
            } else {
                a = x;
            }
            v = w;
            fv = fw;
            w = x;
            fw = fx;
            x = u;
            fx = fu;
        } else {
            if u < x {
                a = u;
            } else {
                b = u;
            }
            if fu <= fw || precision_equals(w, x) {
                v = w;
                fv = fw;
                w = u;
                fw = fu;
            } else if fu <= fv || precision_equals(v, x) || precision_equals(v, w) {
                v = u;
                fv = fu;
            }
        }
    }
}

fn better(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    if a.1 <= b.1 {
        a
    } else {
        b
    }
}

/// `org.apache.commons.math3.util.Precision.equals(x, y)` — the one-argument
/// form, which is `equals(x, y, 1)`: **within one ULP**, not `==`. Brent's
/// bookkeeping branches on it, so the distinction matters.
///
/// The bit patterns are compared as **signed** 64-bit integers, because that is
/// what `Double.doubleToRawLongBits` returns in Java: `-0.0` comes back as
/// `Long.MIN_VALUE`, so `xInt < yInt` orders `+0.0` *after* `-0.0`. Comparing
/// the same bits as `u64` flips that branch and makes `equals(0.0, -0.0)` come
/// out false, which it is not.
fn precision_equals(x: f64, y: f64) -> bool {
    /// `Long.MIN_VALUE` — the sign bit, and `doubleToRawLongBits(-0.0)`.
    const SGN_MASK: i64 = i64::MIN;
    const MAX_ULPS: i64 = 1;
    if x.is_nan() || y.is_nan() {
        return false;
    }
    let xi = x.to_bits() as i64;
    let yi = y.to_bits() as i64;
    if (xi ^ yi) & SGN_MASK == 0 {
        // Same sign: no overflow risk.
        xi.wrapping_sub(yi).wrapping_abs() <= MAX_ULPS
    } else {
        // Opposite signs: each side's distance from its own zero.
        //   POSITIVE_ZERO_DOUBLE_BITS = 0, NEGATIVE_ZERO_DOUBLE_BITS = SGN_MASK
        let (delta_plus, delta_minus) = if xi < yi {
            (yi, xi.wrapping_sub(SGN_MASK))
        } else {
            (xi, yi.wrapping_sub(SGN_MASK))
        };
        delta_plus <= MAX_ULPS && delta_minus <= MAX_ULPS - delta_plus
    }
}

// ---------------------------------------------------------------------------
// Bound-constrained Nelder-Mead — the BOBYQA stand-in
// ---------------------------------------------------------------------------

/// Minimizes `f` over the unit box `[0, 1]ⁿ` from `start`.
///
/// See the module docs for why this stands in for `BOBYQAOptimizer`. The
/// simplex is built with edge [`INITIAL_RADIUS`] (folded inwards where a vertex
/// would leave the box), every trial point is clamped back into the box, and
/// the search stops when the simplex is smaller than [`STOPPING_RADIUS`].
/// Deterministic: no restarts, no randomization.
fn nelder_mead_unit_box<F>(f: &mut F, start: &[f64]) -> std::result::Result<Vec<f64>, Stop>
where
    F: FnMut(&[f64]) -> std::result::Result<f64, Stop>,
{
    let n = start.len();
    let clamp_box = |p: &[f64]| -> Vec<f64> { p.iter().map(|x| clamp(*x, 0.0, 1.0)).collect() };

    // Vertices: the start, plus one step of INITIAL_RADIUS along each axis,
    // folded to the inside when the step would leave the box.
    let mut simplex: Vec<(Vec<f64>, f64)> = Vec::with_capacity(n + 1);
    let p0 = clamp_box(start);
    let f0 = f(&p0)?;
    simplex.push((p0.clone(), f0));
    for i in 0..n {
        let mut p = p0.clone();
        p[i] = if p0[i] + INITIAL_RADIUS <= 1.0 {
            p0[i] + INITIAL_RADIUS
        } else {
            p0[i] - INITIAL_RADIUS
        };
        let p = clamp_box(&p);
        let value = f(&p)?;
        simplex.push((p, value));
    }

    loop {
        simplex.sort_by(|a, b| a.1.total_cmp(&b.1));
        // Converged once the simplex has collapsed below the stopping radius.
        let size = simplex[1..]
            .iter()
            .map(|(p, _)| {
                p.iter()
                    .zip(&simplex[0].0)
                    .map(|(a, b)| (a - b).abs())
                    .fold(0.0f64, f64::max)
            })
            .fold(0.0f64, f64::max);
        if size <= STOPPING_RADIUS {
            return Ok(simplex[0].0.clone());
        }

        // Centroid of everything but the worst vertex.
        let worst = simplex.len() - 1;
        let centroid: Vec<f64> = (0..n)
            .map(|i| simplex[..worst].iter().map(|(p, _)| p[i]).sum::<f64>() / worst as f64)
            .collect();
        let step = |coefficient: f64| -> Vec<f64> {
            clamp_box(
                &(0..n)
                    .map(|i| centroid[i] + coefficient * (centroid[i] - simplex[worst].0[i]))
                    .collect::<Vec<_>>(),
            )
        };

        let reflected = step(1.0);
        let f_reflected = f(&reflected)?;
        if f_reflected < simplex[0].1 {
            // Better than the best: try to go further.
            let expanded = step(2.0);
            let f_expanded = f(&expanded)?;
            simplex[worst] = if f_expanded < f_reflected {
                (expanded, f_expanded)
            } else {
                (reflected, f_reflected)
            };
            continue;
        }
        if f_reflected < simplex[worst - 1].1 {
            simplex[worst] = (reflected, f_reflected);
            continue;
        }
        // Contract.
        let contracted = step(-0.5);
        let f_contracted = f(&contracted)?;
        if f_contracted < simplex[worst].1 {
            simplex[worst] = (contracted, f_contracted);
            continue;
        }
        // Shrink towards the best vertex.
        let best = simplex[0].0.clone();
        for slot in simplex[1..].iter_mut() {
            let p = clamp_box(
                &(0..n)
                    .map(|i| best[i] + 0.5 * (slot.0[i] - best[i]))
                    .collect::<Vec<_>>(),
            );
            let value = f(&p)?;
            *slot = (p, value);
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

    /// A synthetic transient: `Temp(t) = Tinf + (T0 - Tinf) * exp(-k t)`,
    /// tabulated the way a solved `DYNAMIC` block would be. `time` and `temp`
    /// are named distinctly on purpose — frees identifiers are
    /// case-insensitive, so a `T`/`t` pair would collide into one column.
    fn cooling_table(k: f64, t_inf: f64, t0: f64, times: &[f64]) -> Vec<OdeTableView> {
        vec![OdeTableView {
            name: "cool".into(),
            columns: vec!["time".into(), "temp".into()],
            rows: times
                .iter()
                .map(|t| vec![Some(*t), Some(t_inf + (t0 - t_inf) * (-k * t).exp())])
                .collect(),
        }]
    }

    /// Reads back the `k = <value>` line `apply_overrides` appended.
    fn overridden_k(text: &str) -> f64 {
        text.lines()
            .rev()
            .find_map(|line| line.strip_prefix("k = "))
            .expect("k override")
            .parse()
            .expect("number")
    }

    fn request<'a>(
        parameters: &'a [String],
        initial: &'a [f64],
        lower: &'a [f64],
        upper: &'a [f64],
        measured_t: &'a [f64],
        measured_v: &'a [f64],
    ) -> FitRequest<'a> {
        FitRequest {
            text: "k = 0.05\n",
            parameters,
            initial,
            lower,
            upper,
            ode_block: "cool",
            column: "temp",
            measured_t,
            measured_v,
            max_evaluations: 400,
            ..FitRequest::default()
        }
    }

    // -- SampledSeries ----------------------------------------------------

    #[test]
    fn linear_resampling_matches_the_java_rules() {
        let s = SampledSeries::linear(vec![0.0, 1.0, 2.0], vec![0.0, 10.0, 20.0]);
        assert_eq!(s.at(0.0), 0.0);
        assert_eq!(s.at(0.5), 5.0);
        assert_eq!(s.at(2.0), 20.0);
        // Before the first sample there is nothing to hold.
        assert!(s.at(-0.1).is_nan());
        // Past the last sample the Java falls through to the last value.
        assert_eq!(s.at(3.0), 20.0);
    }

    #[test]
    fn a_gap_is_never_bridged_by_interpolation() {
        let s = SampledSeries::linear(vec![0.0, 1.0, 2.0], vec![0.0, f64::NAN, 20.0]);
        assert!(s.at(0.5).is_nan());
        assert!(s.at(1.5).is_nan());
        assert_eq!(s.at(0.0), 0.0);
    }

    #[test]
    fn lower_bound_is_the_first_index_at_or_after_x() {
        let a = [0.0, 1.0, 1.0, 3.0];
        assert_eq!(lower_bound(&a, -1.0), 0);
        assert_eq!(lower_bound(&a, 1.0), 1);
        assert_eq!(lower_bound(&a, 2.0), 3);
        assert_eq!(lower_bound(&a, 9.0), 4);
    }

    #[test]
    fn an_empty_series_samples_to_nan() {
        let s = SampledSeries::linear(Vec::new(), Vec::new());
        assert!(s.at(0.0).is_nan());
        assert_eq!(s.sample_on(&[0.0, 1.0]).len(), 2);
    }

    // -- extract_column ---------------------------------------------------

    #[test]
    fn the_named_block_and_column_are_matched_case_insensitively() {
        let tables = cooling_table(0.05, 20.0, 95.0, &[0.0, 10.0, 20.0]);
        let s = extract_column(&tables, "COOL", "Temp").expect("column");
        assert_eq!(s.t, [0.0, 10.0, 20.0]);
        close(s.v[0], 95.0, 1e-12);
        assert!(extract_column(&tables, "other", "temp").is_none());
        assert!(extract_column(&tables, "cool", "missing").is_none());
    }

    #[test]
    fn a_null_cell_becomes_nan() {
        let tables = vec![OdeTableView {
            name: "cool".into(),
            columns: vec!["time".into(), "temp".into()],
            rows: vec![vec![Some(0.0), None], vec![Some(1.0), Some(2.0)]],
        }];
        let s = extract_column(&tables, "cool", "temp").expect("column");
        assert!(s.v[0].is_nan());
        assert_eq!(s.v[1], 2.0);
    }

    // -- the fit itself ---------------------------------------------------

    #[test]
    fn a_single_parameter_is_recovered_from_a_clean_measurement() {
        // Measurement generated with k = 0.05; the fit starts at 0.2.
        let times: Vec<f64> = (0..=12).map(|i| i as f64 * 5.0).collect();
        let measured: Vec<f64> = times
            .iter()
            .map(|t| 20.0 + 75.0 * (-0.05 * t).exp())
            .collect();
        let parameters = ["k".to_string()];
        let out = run(
            &request(&parameters, &[0.2], &[0.005], &[0.5], &times, &measured),
            |text| Some(cooling_table(overridden_k(text), 20.0, 95.0, &times)),
            || false,
        )
        .expect("fit");

        assert_eq!(out.parameters, ["k"]);
        close(out.fitted[0], 0.05, 1e-4);
        assert!(out.rmse < 1e-4, "rmse {}", out.rmse);
        assert!(
            out.initial_rmse > out.rmse,
            "the fit must improve on the start ({} -> {})",
            out.initial_rmse,
            out.rmse
        );
        assert!(!out.truncated);
        assert!(out.evaluations > 1);
        // The overlay rides on the measured raster.
        assert_eq!(out.fitted_series.t, times);
        close(out.fitted_series.v[0], 95.0, 1e-9);
    }

    #[test]
    fn two_parameters_are_recovered_on_the_unit_box() {
        // Both the rate and the ambient temperature are unknown.
        let times: Vec<f64> = (0..=20).map(|i| i as f64 * 3.0).collect();
        let measured: Vec<f64> = times
            .iter()
            .map(|t| 22.0 + (95.0 - 22.0) * (-0.07 * t).exp())
            .collect();
        let parameters = ["k".to_string(), "tinf".to_string()];
        let times_for_solve = times.clone();
        let out = run(
            &FitRequest {
                text: "k = 0.05\ntinf = 20\n",
                parameters: &parameters,
                initial: &[0.03, 15.0],
                lower: &[0.005, 5.0],
                upper: &[0.3, 40.0],
                ode_block: "cool",
                column: "temp",
                measured_t: &times,
                measured_v: &measured,
                max_evaluations: 2000,
                ..FitRequest::default()
            },
            |text| {
                let k = overridden_k(text);
                let t_inf = text
                    .lines()
                    .rev()
                    .find_map(|l| l.strip_prefix("tinf = "))
                    .expect("tinf override")
                    .parse()
                    .expect("number");
                Some(cooling_table(k, t_inf, 95.0, &times_for_solve))
            },
            || false,
        )
        .expect("fit");

        close(out.fitted[0], 0.07, 1e-3);
        close(out.fitted[1], 22.0, 1e-3);
        assert!(out.rmse < 1e-2, "rmse {}", out.rmse);
    }

    #[test]
    fn an_infeasible_start_is_reported_rather_than_silently_returned() {
        let times = [0.0, 5.0, 10.0];
        let measured = [95.0, 80.0, 70.0];
        let parameters = ["k".to_string()];
        let err = run(
            &request(&parameters, &[0.2], &[0.005], &[0.5], &times, &measured),
            |_| None, // every solve fails
            || false,
        )
        .unwrap_err();
        assert!(
            err.to_string_message()
                .starts_with("The model does not solve"),
            "{}",
            err.to_string_message()
        );
    }

    #[test]
    fn a_measurement_that_does_not_overlap_the_model_is_infeasible() {
        // The model starts at t = 100; the measurement sits before it, where
        // `SampledSeries.at` answers NaN (there is nothing to hold), so no pair
        // is comparable. Note the asymmetry, which is the Java's: *past* the
        // last sample LINEAR holds the final value instead of giving up.
        let times: [f64; 3] = [0.0, 1.0, 2.0];
        let measured: [f64; 3] = [95.0, 80.0, 70.0];
        let parameters = ["k".to_string()];
        let err = run(
            &request(&parameters, &[0.2], &[0.005], &[0.5], &times, &measured),
            |text| {
                Some(cooling_table(
                    overridden_k(text),
                    20.0,
                    95.0,
                    &[100.0, 105.0, 110.0],
                ))
            },
            || false,
        )
        .unwrap_err();
        assert!(err
            .to_string_message()
            .starts_with("The model does not solve"));
    }

    #[test]
    fn nan_measurement_samples_are_skipped_not_compared() {
        let times: Vec<f64> = (0..=12).map(|i| i as f64 * 5.0).collect();
        let mut measured: Vec<f64> = times
            .iter()
            .map(|t| 20.0 + 75.0 * (-0.05 * t).exp())
            .collect();
        measured[3] = f64::NAN;
        measured[7] = f64::NAN;
        let parameters = ["k".to_string()];
        let out = run(
            &request(&parameters, &[0.2], &[0.005], &[0.5], &times, &measured),
            |text| Some(cooling_table(overridden_k(text), 20.0, 95.0, &times)),
            || false,
        )
        .expect("fit");
        close(out.fitted[0], 0.05, 1e-4);
        assert!(out.rmse < 1e-4);
    }

    #[test]
    fn a_budget_strike_truncates_and_keeps_the_best_iterate() {
        let times: Vec<f64> = (0..=12).map(|i| i as f64 * 5.0).collect();
        let measured: Vec<f64> = times
            .iter()
            .map(|t| 20.0 + 75.0 * (-0.05 * t).exp())
            .collect();
        let parameters = ["k".to_string()];
        let mut calls = 0;
        let out = run(
            &request(&parameters, &[0.2], &[0.005], &[0.5], &times, &measured),
            |text| Some(cooling_table(overridden_k(text), 20.0, 95.0, &times)),
            || {
                calls += 1;
                calls > 4
            },
        )
        .expect("fit");
        assert!(out.truncated);
        assert!(out.evaluations <= 4);
        // The tracked best still solves, so an outcome is still produced.
        assert!(out.rmse.is_finite());
    }

    #[test]
    fn the_evaluation_budget_stops_the_search_without_truncating() {
        let times: Vec<f64> = (0..=12).map(|i| i as f64 * 5.0).collect();
        let measured: Vec<f64> = times
            .iter()
            .map(|t| 20.0 + 75.0 * (-0.05 * t).exp())
            .collect();
        let parameters = ["k".to_string()];
        let mut req = request(&parameters, &[0.2], &[0.005], &[0.5], &times, &measured);
        req.max_evaluations = 6;
        let out = run(
            &req,
            |text| Some(cooling_table(overridden_k(text), 20.0, 95.0, &times)),
            || false,
        )
        .expect("fit");
        // `TooManyEvaluationsException` is not truncation: the budget was the
        // point, and the best tracked iterate stands. Six optimizer evaluations
        // plus the feasibility probe, which Java counts but does not budget.
        assert!(!out.truncated);
        assert_eq!(out.evaluations, 7);
    }

    // -- end to end, against the reference engine ---------------------------

    /// The reference `ParameterFitTest`'s model, verbatim.
    const DECAY_MODEL: &str = "k = 0.3\nx0 = 5\nDYNAMIC decay(t = 0 .. 5, points = 120)\nder(x) = -k * x\nx(0) = x0\nEND\n";

    /// The reference test's synthetic measurement: an exponential decay plus a
    /// deterministic ripple, sampled 60 times over `t` in `[0, 5]`. Note the
    /// ripple is indexed by *sample number*, not by time — transcribed as-is.
    fn synthetic_decay(k_true: f64, x0: f64) -> (Vec<f64>, Vec<f64>) {
        let n = 60;
        (0..n)
            .map(|i| {
                let t = 5.0 * i as f64 / (n - 1) as f64;
                (t, x0 * (-k_true * t).exp() + 0.01 * (13.0 * i as f64).sin())
            })
            .unzip()
    }

    /// The last `name = <number>` line of the overridden document, or the
    /// fallback when the document never assigns it.
    fn assigned(text: &str, name: &str, fallback: f64) -> f64 {
        let prefix = format!("{name} = ");
        text.lines()
            .rev()
            .find_map(|line| line.trim().strip_prefix(&prefix))
            .and_then(|rest| rest.trim().parse().ok())
            .unwrap_or(fallback)
    }

    /// `DYNAMIC decay(t = 0 .. 5, points = 120)` solved analytically.
    ///
    /// Standing in for the transient integrator is honest here rather than
    /// convenient: the reference engine's own table for this document ends at
    /// `x(5) = 1.1156508007425125` against the closed form's
    /// `5·e^-1.5 = 1.1156508007421491` — agreement to 3 parts in 10^13, which is
    /// far below anything the fit's tolerances can see.
    fn decay_table(text: &str) -> Vec<OdeTableView> {
        let k = assigned(text, "k", 0.3);
        let x0 = assigned(text, "x0", 5.0);
        vec![OdeTableView {
            name: "decay".into(),
            columns: vec!["t".into(), "x".into()],
            rows: (0..120)
                .map(|i| {
                    let t = 5.0 * i as f64 / 119.0;
                    vec![Some(t), Some(x0 * (-k * t).exp())]
                })
                .collect(),
        }]
    }

    #[test]
    fn oracle_one_parameter_calibration_reproduces_the_reference_run() {
        // `ParameterFit.run(solver, DECAY_MODEL, DEFAULTS, {}, {}, ["k"],
        //                   [0.3], [0.05], [3.0], "decay", "x", t, v, 120, …)`
        //   fitted      [0.700091670038771]
        //   rmse        0.0069698589277249106
        //   initialRmse 1.2395454095368792
        //   evaluations 16
        // The evaluation count is the strong claim: it is decided entirely by
        // the Brent path, which is transcribed rather than substituted.
        let (t, v) = synthetic_decay(0.7, 5.0);
        let parameters = ["k".to_string()];
        let out = run(
            &FitRequest {
                text: DECAY_MODEL,
                parameters: &parameters,
                initial: &[0.3],
                lower: &[0.05],
                upper: &[3.0],
                ode_block: "decay",
                column: "x",
                measured_t: &t,
                measured_v: &v,
                max_evaluations: 120,
                ..FitRequest::default()
            },
            |text| Some(decay_table(text)),
            || false,
        )
        .expect("fit");

        assert_eq!(out.evaluations, 16);
        assert!(!out.truncated);
        close(out.fitted[0], 0.700091670038771, 1e-6);
        close(out.rmse, 0.0069698589277249106, 1e-6);
        close(out.initial_rmse, 1.2395454095368792, 1e-9);
        assert_eq!(out.fitted_series.t.len(), 60);
        close(out.fitted_series.v[0], 5.0, 1e-9);
        close(out.fitted_series.v[59], 0.15091772809465526, 1e-5);
    }

    #[test]
    fn oracle_two_parameter_calibration_lands_on_the_reference_optimum() {
        // Same reference run with ["k", "x0"]:
        //   fitted      [0.7009254245691521, 4.204854659927836]
        //   rmse        0.006837653868351035
        //   initialRmse 1.4391421696594269
        //   evaluations 39
        // `evaluations` is deliberately NOT asserted: this is the Nelder-Mead
        // path standing in for BOBYQA (see the module docs), so the route to the
        // optimum differs even though the optimum does not.
        let (t, v) = synthetic_decay(0.7, 4.2);
        let parameters = ["k".to_string(), "x0".to_string()];
        let out = run(
            &FitRequest {
                text: DECAY_MODEL,
                parameters: &parameters,
                initial: &[0.3, 5.0],
                lower: &[0.05, 1.0],
                upper: &[3.0, 10.0],
                ode_block: "decay",
                column: "x",
                measured_t: &t,
                measured_v: &v,
                max_evaluations: 250,
                ..FitRequest::default()
            },
            |text| Some(decay_table(text)),
            || false,
        )
        .expect("fit");

        close(out.initial_rmse, 1.4391421696594269, 1e-9);
        // The reference test's own acceptance tolerances on the true values.
        assert!((out.fitted[0] - 0.7).abs() < 0.03, "k = {}", out.fitted[0]);
        assert!((out.fitted[1] - 4.2).abs() < 0.05, "x0 = {}", out.fitted[1]);
        // And, more tightly, agreement with the Java's own optimum.
        close(out.fitted[0], 0.7009254245691521, 1e-3);
        close(out.fitted[1], 4.204854659927836, 1e-3);
        assert!(
            out.rmse <= 0.006837653868351035 * 1.001,
            "rmse {} must reach the reference optimum",
            out.rmse
        );
    }

    #[test]
    fn oracle_a_wrong_column_is_an_infeasible_start() {
        // The reference test `infeasibleStartIsReportedNotReturned`.
        let (t, v) = synthetic_decay(0.7, 5.0);
        let parameters = ["k".to_string()];
        let err = run(
            &FitRequest {
                text: DECAY_MODEL,
                parameters: &parameters,
                initial: &[0.3],
                lower: &[0.05],
                upper: &[3.0],
                ode_block: "decay",
                column: "no_such_column",
                measured_t: &t,
                measured_v: &v,
                max_evaluations: 50,
                ..FitRequest::default()
            },
            |text| Some(decay_table(text)),
            || false,
        )
        .unwrap_err();
        assert!(
            err.to_string_message().contains("initial parameter values"),
            "{}",
            err.to_string_message()
        );
    }

    // -- request validation ------------------------------------------------

    #[test]
    fn a_request_with_no_parameters_is_refused() {
        let err = run(
            &request(&[], &[], &[], &[], &[0.0, 1.0], &[1.0, 2.0]),
            |_| None,
            || false,
        )
        .unwrap_err();
        assert!(err
            .to_string_message()
            .starts_with("Parameter estimation needs at least one parameter"));
    }

    #[test]
    fn infinite_or_crossed_bounds_are_refused() {
        let parameters = ["k".to_string()];
        for (lo, hi) in [(f64::NEG_INFINITY, 1.0), (0.0, f64::INFINITY), (1.0, 1.0)] {
            let err = run(
                &request(&parameters, &[0.5], &[lo], &[hi], &[0.0, 1.0], &[1.0, 2.0]),
                |_| None,
                || false,
            )
            .unwrap_err();
            assert_eq!(
                err.to_string_message(),
                "Bounds for k must be finite with lower < upper."
            );
        }
    }

    #[test]
    fn an_initial_value_outside_its_bounds_is_refused() {
        let parameters = ["k".to_string()];
        let err = run(
            &request(
                &parameters,
                &[9.0],
                &[0.0],
                &[1.0],
                &[0.0, 1.0],
                &[1.0, 2.0],
            ),
            |_| None,
            || false,
        )
        .unwrap_err();
        assert_eq!(
            err.to_string_message(),
            "The initial value of k lies outside its bounds."
        );
    }

    #[test]
    fn a_measurement_shorter_than_two_samples_is_refused() {
        let parameters = ["k".to_string()];
        let err = run(
            &request(&parameters, &[0.5], &[0.0], &[1.0], &[0.0], &[1.0]),
            |_| None,
            || false,
        )
        .unwrap_err();
        assert_eq!(
            err.to_string_message(),
            "The measured series needs at least two (t, y) samples."
        );
    }

    // -- the optimizers ----------------------------------------------------

    /// Runs `brent_minimize` with the `ParameterFit` settings and records every
    /// point it evaluated, so the trace can be compared with the reference.
    fn brent_trace(f: impl Fn(f64) -> f64, start: f64) -> (Vec<f64>, f64, f64) {
        let mut trace = Vec::new();
        let mut traced = |p: &[f64]| -> std::result::Result<f64, Stop> {
            trace.push(p[0]);
            Ok(f(p[0]))
        };
        let (point, value) =
            brent_minimize(&mut traced, 0.0, 1.0, start, 1.0e-8, 1.0e-10).expect("brent");
        (trace, point, value)
    }

    #[test]
    fn oracle_brent_reproduces_the_reference_evaluation_sequence_exactly() {
        // Commons Math `BrentOptimizer(1e-8, 1e-10)` on `SearchInterval(0, 1, 0.5)`,
        // minimizing (x - 0.3)^2 + 1. Twenty-one evaluations, captured from a
        // JVM probe; pure arithmetic, so this comparison is bit-for-bit.
        let expected: [f64; 21] = [
            0.5,
            0.30901699437494745,
            0.1909830056250526,
            0.29999999999999993,
            0.3000000030999999,
            0.30000000619999995,
            0.303444189206674,
            0.3013155670450746,
            0.3005025057285499,
            0.30019194394057525,
            0.30007331989317587,
            0.30002800953895237,
            0.3000107025236813,
            0.30000409183209154,
            0.30000156677259343,
            0.30000060228568876,
            0.3000002338844729,
            0.3000000931677299,
            0.3000000394187169,
            0.30000001888842076,
            0.30000001104654545,
        ];
        let (trace, point, value) = brent_trace(|x| (x - 0.3) * (x - 0.3) + 1.0, 0.5);
        assert_eq!(trace.len(), expected.len());
        assert_eq!(trace, expected);
        assert_eq!(point, 0.29999999999999993);
        assert_eq!(value, 1.0);
    }

    #[test]
    fn oracle_brent_on_a_calibration_shaped_objective() {
        // The SSE landscape a one-parameter fit actually sees: an exponential
        // whose rate is the unit-box parameter, compared against a measurement
        // generated at k = 0.05. Sixteen evaluations in the reference, landing
        // on u = 0.09090909089177841 (k = 0.05).
        //
        // `exp` is the one transcendental here, so the trace is compared to a
        // tolerance rather than bit-for-bit; the *count* is exact.
        let expected: [f64; 16] = [
            0.39,
            0.6229992668625641,
            0.24103325561245903,
            0.14896674438754104,
            0.09206651122491802,
            0.056900233162623055,
            0.10423018689985954,
            0.08311555640237238,
            0.09159423093099703,
            0.09099983282337075,
            0.09090249618890373,
            0.0909087010681699,
            0.09090909521146412,
            0.09090909089177841,
            0.09090909190086932,
            0.0909090898826875,
        ];
        let sse = |u: f64| {
            let k = 0.005 + u * (0.5 - 0.005);
            (0..=12)
                .map(|i| {
                    let t = i as f64 * 5.0;
                    let model = 20.0 + 75.0 * (-k * t).exp();
                    let meas = 20.0 + 75.0 * (-0.05 * t).exp();
                    (model - meas) * (model - meas)
                })
                .sum::<f64>()
        };
        let (trace, point, _) = brent_trace(sse, 0.39);
        assert_eq!(trace.len(), expected.len());
        for (got, want) in trace.iter().zip(expected) {
            close(*got, want, 1e-9);
        }
        close(point, 0.09090909089177841, 1e-12);
    }

    #[test]
    fn precision_equals_is_a_one_ulp_test_not_an_identity_test() {
        let x = 1.0f64;
        let next = f64::from_bits(x.to_bits() + 1);
        assert!(precision_equals(x, x));
        assert!(precision_equals(x, next));
        assert!(!precision_equals(x, f64::from_bits(x.to_bits() + 2)));
        assert!(!precision_equals(f64::NAN, f64::NAN));
        assert!(precision_equals(0.0, -0.0));
    }

    #[test]
    fn nelder_mead_stays_inside_the_unit_box() {
        // The unconstrained minimum sits outside the box, so the search must
        // stop on the boundary rather than walking out of it.
        let mut seen_outside = false;
        let mut f = |p: &[f64]| -> std::result::Result<f64, Stop> {
            if p.iter().any(|x| *x < 0.0 || *x > 1.0) {
                seen_outside = true;
            }
            Ok((p[0] + 0.5).powi(2) + (p[1] - 0.4).powi(2))
        };
        let best = nelder_mead_unit_box(&mut f, &[0.5, 0.5]).expect("simplex");
        assert!(!seen_outside);
        close(best[0], 0.0, 1e-4);
        close(best[1], 0.4, 1e-4);
    }
    // ── Phase 4.2: weighting, robust loss and diagnostics ───────────────────

    /// The oracle run above must be untouched by the new fields — this is the
    /// same document with an explicit all-default request, and the numbers have
    /// to be identical, not merely close.
    #[test]
    fn the_default_request_reproduces_the_unweighted_fit_exactly() {
        let (t, v) = synthetic_decay(0.7, 5.0);
        let parameters = ["k".to_string()];
        let base = FitRequest {
            text: DECAY_MODEL,
            parameters: &parameters,
            initial: &[0.3],
            lower: &[0.05],
            upper: &[3.0],
            ode_block: "decay",
            column: "x",
            measured_t: &t,
            measured_v: &v,
            max_evaluations: 120,
            ..FitRequest::default()
        };
        let plain = run(&base, |text| Some(decay_table(text)), || false).expect("fit");
        let explicit = run(
            &FitRequest {
                sigma: &[],
                loss: Loss::Linear,
                f_scale: 1.0,
                ..base
            },
            |text| Some(decay_table(text)),
            || false,
        )
        .expect("fit");
        assert_eq!(plain.fitted, explicit.fitted);
        assert_eq!(plain.rmse, explicit.rmse);
        assert_eq!(plain.evaluations, explicit.evaluations);
        // Chi-square with no sigma is the plain SSE, so it must square back to
        // the reported RMSE.
        let pairs = t.len() as f64;
        close((plain.chi_square / pairs).sqrt(), plain.rmse, 1e-12);
    }

    #[test]
    fn a_fit_reports_identifiability_diagnostics() {
        let (t, v) = synthetic_decay(0.7, 5.0);
        let parameters = ["k".to_string()];
        let out = run(
            &FitRequest {
                text: DECAY_MODEL,
                parameters: &parameters,
                initial: &[0.3],
                lower: &[0.05],
                upper: &[3.0],
                ode_block: "decay",
                column: "x",
                measured_t: &t,
                measured_v: &v,
                max_evaluations: 120,
                ..FitRequest::default()
            },
            |text| Some(decay_table(text)),
            || false,
        )
        .expect("fit");
        assert_eq!(out.residual_dof, t.len() - 1);
        assert_eq!(out.rank, 1);
        assert!(!out.unidentifiable);
        assert!(out.std_errors[0].is_finite() && out.std_errors[0] > 0.0);
        assert_eq!(out.covariance.len(), 1);
        assert!(out.condition_number.is_finite());
        // Unweighted, so a reduced chi-square would be comparing the scatter
        // against nothing.
        assert!(out.reduced_chi_square.is_nan());
        assert_eq!(out.at_bound, vec![false]);
    }

    #[test]
    fn a_parameter_resting_on_its_bound_is_flagged() {
        // Data generated at k = 0.7 but the upper bound pinned at 0.4: the fit
        // can only go as far as the bound, and must say so.
        let (t, v) = synthetic_decay(0.7, 5.0);
        let parameters = ["k".to_string()];
        let out = run(
            &FitRequest {
                text: DECAY_MODEL,
                parameters: &parameters,
                initial: &[0.3],
                lower: &[0.05],
                upper: &[0.4],
                ode_block: "decay",
                column: "x",
                measured_t: &t,
                measured_v: &v,
                max_evaluations: 200,
                ..FitRequest::default()
            },
            |text| Some(decay_table(text)),
            || false,
        )
        .expect("fit");
        close(out.fitted[0], 0.4, 1e-3);
        assert_eq!(out.at_bound, vec![true], "fitted {}", out.fitted[0]);
    }

    #[test]
    fn a_large_sigma_removes_a_point_from_the_fit() {
        // Data at k = 0.7 with one sample corrupted. Given an honest sigma for
        // the corrupted point, the fit must land where the clean data says.
        let (t, mut v) = synthetic_decay(0.7, 5.0);
        let bad = v.len() / 2;
        v[bad] += 3.0;
        let mut sigma = vec![0.01; v.len()];
        sigma[bad] = 1.0e4;
        let parameters = ["k".to_string()];
        let fit = |sigma: &[f64]| {
            run(
                &FitRequest {
                    text: DECAY_MODEL,
                    parameters: &parameters,
                    initial: &[0.3],
                    lower: &[0.05],
                    upper: &[3.0],
                    ode_block: "decay",
                    column: "x",
                    measured_t: &t,
                    measured_v: &v,
                    max_evaluations: 300,
                    sigma,
                    ..FitRequest::default()
                },
                |text| Some(decay_table(text)),
                || false,
            )
            .expect("fit")
        };
        let unweighted = fit(&[]);
        let weighted = fit(&sigma);
        // The corrupted point drags an unweighted fit away from 0.7…
        assert!(
            (unweighted.fitted[0] - 0.7).abs() > 0.02,
            "unweighted landed at {} — the outlier was supposed to bite",
            unweighted.fitted[0]
        );
        // …and discounting it brings the fit back.
        close(weighted.fitted[0], 0.7, 5e-3);
        // A weighted fit does have a reduced chi-square.
        assert!(weighted.reduced_chi_square.is_finite());
    }

    #[test]
    fn a_robust_loss_survives_an_outlier_without_being_told_where_it_is() {
        let (t, mut v) = synthetic_decay(0.7, 5.0);
        let bad = v.len() / 2;
        v[bad] += 3.0;
        let parameters = ["k".to_string()];
        let fit = |loss: Loss| {
            run(
                &FitRequest {
                    text: DECAY_MODEL,
                    parameters: &parameters,
                    initial: &[0.3],
                    lower: &[0.05],
                    upper: &[3.0],
                    ode_block: "decay",
                    column: "x",
                    measured_t: &t,
                    measured_v: &v,
                    max_evaluations: 300,
                    loss,
                    f_scale: 0.05,
                    ..FitRequest::default()
                },
                |text| Some(decay_table(text)),
                || false,
            )
            .expect("fit")
        };
        let plain = fit(Loss::Linear);
        let robust = fit(Loss::Cauchy);
        assert!(
            (robust.fitted[0] - 0.7).abs() < (plain.fitted[0] - 0.7).abs(),
            "robust {} was no closer to 0.7 than plain {}",
            robust.fitted[0],
            plain.fitted[0]
        );
    }

    #[test]
    fn a_malformed_sigma_is_refused() {
        let (t, v) = synthetic_decay(0.7, 5.0);
        let parameters = ["k".to_string()];
        let make = |sigma: &[f64]| {
            run(
                &FitRequest {
                    text: DECAY_MODEL,
                    parameters: &parameters,
                    initial: &[0.3],
                    lower: &[0.05],
                    upper: &[3.0],
                    ode_block: "decay",
                    column: "x",
                    measured_t: &t,
                    measured_v: &v,
                    max_evaluations: 50,
                    sigma,
                    ..FitRequest::default()
                },
                |text| Some(decay_table(text)),
                || false,
            )
            .map(|_| ())
            .unwrap_err()
            .to_string()
        };
        assert!(make(&[1.0, 2.0]).contains("per measured sample"));
        let mut zeroed = vec![1.0; v.len()];
        zeroed[0] = 0.0;
        assert!(make(&zeroed).contains("finite and positive"));
    }
}
