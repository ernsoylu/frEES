//! Levenberg–Marquardt least-squares curve fitting.
//!
//! Port of
//! `../frEES/backend/core/src/main/java/com/frees/backend/core/CurveFitter.java`
//! (288 LOC) **plus** the optimizer it delegates to, Commons Math 3.6.1's
//! `org.apache.commons.math3.fitting.leastsquares.LevenbergMarquardtOptimizer`
//! — itself a transcription of MINPACK's `lmder`. The optimizer had to come
//! along: the fit reports `iterations`, and the damping schedule, the
//! column-norm scaling and the four convergence tests together decide both the
//! iteration count and the exact path to the optimum. A different LM converges
//! to the same optimum by a different route and reproduces neither.
//!
//! # The pipeline
//!
//! `y = a * exp(-b * x) + c` is parsed, the dependent variable is matched
//! against one side, and the *other* side becomes the model expression. The
//! parameters `[a, b, c]` are then fitted to the observed `(x, y)` pairs by
//! minimising `Σ (yᵢ − model(xᵢ; p))²`. The Jacobian handed to the optimizer is
//! central finite differences of the model with the step
//! `h = max(1e-8, |pⱼ| · 1e-8)` — the Java `FD_STEP` rule, which is *not* the
//! forward-difference rule the uncertainty engine uses.
//!
//! # Weighting and reported uncertainty — a documented deviation
//!
//! `CurveFitter.fit` reports a point estimate and nothing about how well the
//! data pins it down. Phase 4.2 adds measurement weighting
//! ([`CurveFitRequest::sigma`]) and the uncertainty a fitted parameter is
//! useless without: covariance, standard errors, rank and conditioning of the
//! Jacobian at the optimum, and residual degrees of freedom.
//!
//! Both are **opt-in and inert when unused**. A request with no `sigma` walks
//! the transcribed `lmder` path iterate for iterate, which is what the oracle
//! goldens in this module pin; the extra diagnostics are computed after the
//! optimizer has stopped and cannot move it. `r_squared` and `rmse` keep their
//! Java definitions — unweighted descriptive statistics of the raw residuals —
//! so that a weighted fit still reports the numbers the existing UI shows.
//!
//! The covariance is the **local-linear** one: `σ̂² (JᵀJ)⁻¹` read off an SVD of
//! the Jacobian at the optimum, valid to the extent the model is linear over
//! the parameter's own uncertainty. It is not a profile-likelihood interval
//! and does not know about curvature.
//!
//! # Parameter bounds — the Java's empty `if`, filled in
//!
//! `CurveFitter.fit` takes `lowerBounds` / `upperBounds` and its body contains
//! an empty `if` with the comment "LevenbergMarquardtOptimizer doesn't directly
//! support bounds in Commons Math 3.x, so we proceed without box constraints".
//! Phase 4.2 makes them real, on the
//! [`crate::analysis::paramfit::FitRequest`] convention: parallel finite
//! `lower`/`upper` slices with the initial guess already inside.
//!
//! The mechanism is **projected Levenberg–Marquardt** — each trial point is
//! clamped back into the box and the trust-region bookkeeping then measures the
//! clamped step, not the unconstrained one it came from. That is not the same
//! algorithm as an active-set or interior-point LM: a parameter pinned to a
//! bound can slow convergence, and the local-linear covariance computed at such
//! a point ignores the constraint entirely. [`FitResult::at_bound`] says which
//! parameters that applies to, so the standard error can be read with the right
//! amount of suspicion rather than none.
//!
//! # Robust losses
//!
//! [`Loss`] replaces the squared residual with soft-L1, Huber or Cauchy through
//! iteratively reweighted least squares: each outer pass reweights by
//! `ρ'((r/s)²)` and re-runs the same LM. The scale `s` defaults to the
//! residuals' own MAD-based robust spread — a fixed `f_scale` in raw data units
//! is a knob nobody can set correctly without seeing the residuals first — and
//! [`CurveFitRequest::f_scale`] pins it when the measurement scale is known.
//!
//! [`Loss::Linear`], the default, runs the outer loop exactly once with no
//! reweighting, which is the untouched `lmder` path.

// The LM core below is a line-by-line transcription of Commons Math's
// `LevenbergMarquardtOptimizer` (MINPACK `lmder`), an algorithm written
// entirely in terms of a column permutation: nearly every loop indexes
// `permutation[k]` rather than walking a slice in order. Rewriting the index
// arithmetic into iterator form would destroy the line-for-line correspondence
// with the reference and make a numerical divergence impossible to spot in
// review, so the range-loop lints are turned off for this module only.
#![allow(clippy::needless_range_loop)]

use crate::ast::{Equation, Expr};
use crate::diag::{FreesError, Result};
use crate::eval::{eval, Scope};
use crate::linalg::Mat;

/// `CurveFitter.MAX_EVALUATIONS`.
const MAX_EVALUATIONS: usize = 10_000;
/// `CurveFitter.MAX_ITERATIONS`.
const MAX_ITERATIONS: usize = 1_000;
/// `CurveFitter.FD_STEP` — the central-difference step scale.
const FD_STEP: f64 = 1e-8;

/// `LevenbergMarquardtOptimizer`'s default configuration: initial step-bound
/// factor 100, cost/parameter/orthogonality tolerances 1e-10, and
/// `Precision.SAFE_MIN` as the QR ranking threshold.
const INITIAL_STEP_BOUND_FACTOR: f64 = 100.0;
const COST_RELATIVE_TOLERANCE: f64 = 1e-10;
const PAR_RELATIVE_TOLERANCE: f64 = 1e-10;
const ORTHO_TOLERANCE: f64 = 1e-10;
/// `Precision.SAFE_MIN` = `0x1.0p-1022`.
const SAFE_MIN: f64 = f64::MIN_POSITIVE;
/// `2 * Precision.EPSILON`, where Commons Math's `EPSILON` is `0x1.0p-53`.
/// This is therefore `0x1.0p-52`, i.e. `f64::EPSILON`.
const TWO_EPS: f64 = f64::EPSILON;

/// How a residual is penalised. `ρ` is written as a function of the squared
/// scaled residual `z = (r/s)²`, the SciPy `least_squares` convention.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Loss {
    /// `ρ(z) = z` — ordinary least squares, and the only variant that leaves
    /// the transcribed `lmder` path untouched.
    #[default]
    Linear,
    /// `ρ(z) = 2(√(1+z) − 1)` — smooth L1, mildly outlier-tolerant.
    SoftL1,
    /// `ρ(z) = z` for `z ≤ 1` and `2√z − 1` beyond — Huber.
    Huber,
    /// `ρ(z) = ln(1 + z)` — Cauchy, the most aggressive of the three; far
    /// outliers are given nearly no weight at all.
    Cauchy,
}

impl Loss {
    /// The loss itself, `ρ(z)` for a squared scaled residual `z`. This is what
    /// a derivative-free minimiser sums directly (see
    /// [`crate::analysis::paramfit`]); the IRLS path below uses `ρ'` instead.
    pub(crate) fn value(self, z: f64) -> f64 {
        match self {
            Loss::Linear => z,
            Loss::SoftL1 => 2.0 * ((1.0 + z).sqrt() - 1.0),
            Loss::Huber => {
                if z <= 1.0 {
                    z
                } else {
                    2.0 * z.sqrt() - 1.0
                }
            }
            Loss::Cauchy => libm::log1p(z),
        }
    }

    /// The IRLS weight `ρ'(u²)` for a scaled residual `u`, i.e. the factor the
    /// squared residual is multiplied by to imitate this loss.
    pub(crate) fn weight(self, u: f64) -> f64 {
        let z = u * u;
        match self {
            Loss::Linear => 1.0,
            Loss::SoftL1 => 1.0 / (1.0 + z).sqrt(),
            Loss::Huber => {
                if z <= 1.0 {
                    1.0
                } else {
                    1.0 / z.sqrt()
                }
            }
            Loss::Cauchy => 1.0 / (1.0 + z),
        }
    }
}

/// Outer IRLS passes before a robust fit gives up refining its weights. Twenty
/// is far past where the parameter change test below normally stops it.
const MAX_IRLS_PASSES: usize = 20;

/// The IRLS outer loop stops when no parameter moves by more than this,
/// relative to its own magnitude.
const IRLS_PARAM_TOL: f64 = 1e-10;

/// Everything a curve fit needs about the problem.
///
/// Bundled rather than passed positionally because Phase 4.2 pushed the
/// argument list past `clippy::too_many_arguments`, and because [`Default`]
/// then lets a caller name only the fields it uses — the same shape
/// [`crate::analysis::paramfit::FitRequest`] settled on.
#[derive(Debug, Clone, Default)]
pub struct CurveFitRequest<'a> {
    /// The model equation, e.g. `"y = a * exp(-b * x) + c"`.
    pub model: &'a str,
    /// The dependent variable name, any case.
    pub y_variable: &'a str,
    /// The independent variable names, any case — one per column of
    /// [`x_data`](Self::x_data). A single-predictor fit passes one name.
    pub x_variables: &'a [String],
    /// The names to fit, any case; reported lowercased.
    pub parameters: &'a [String],
    /// Predictor columns, parallel to [`x_variables`](Self::x_variables). Every
    /// column must be as long as `y_data`.
    pub x_data: &'a [Vec<f64>],
    pub y_data: &'a [f64],
    /// Starting values. Entries past the end of the slice (and a `None` slice)
    /// default to `1.0`, matching the Java's per-index
    /// `initialGuess != null && i < size && get(i) != null` test. The Java's
    /// third clause — a `null` *inside* the list — has no counterpart in
    /// `&[f64]`; a caller that wants the default for one parameter passes a
    /// shorter slice or `1.0`.
    pub initial_guess: Option<&'a [f64]>,
    /// Per-point measurement standard deviations. When present the fit
    /// minimises `Σ ((yᵢ − f(xᵢ))/σᵢ)²` and the reported covariance is taken as
    /// **absolute** — σ is believed rather than rescaled by the observed
    /// residual spread. [`FitResult::reduced_chi_square`] is the check on that
    /// belief: far from 1 means the quoted σ do not describe the scatter.
    ///
    /// Every entry must be finite and strictly positive. Absent, the covariance
    /// is scaled by the estimated residual variance `SSres / (n − p)` instead,
    /// which is the right answer when only *relative* weights are known.
    pub sigma: Option<&'a [f64]>,
    /// Lower box constraints, one per parameter. Must be finite and strictly
    /// below [`upper`](Self::upper), with the initial guess already inside —
    /// the [`crate::analysis::paramfit`] rules, refused in its words.
    /// `None` on either side means an unconstrained fit.
    pub lower: Option<&'a [f64]>,
    /// Upper box constraints; see [`lower`](Self::lower).
    pub upper: Option<&'a [f64]>,
    /// The residual penalty. [`Loss::Linear`] is ordinary least squares.
    pub loss: Loss,
    /// The residual scale a robust [`loss`](Self::loss) measures outliers
    /// against, in the same units as the (σ-scaled, if σ was given) residual.
    /// `None` re-estimates it each pass from the residuals' own MAD spread,
    /// `1.4826 · median|r − median r|`, floored away from zero. Ignored for
    /// [`Loss::Linear`].
    pub f_scale: Option<f64>,
    /// Two-sided confidence level for the reported bands, e.g. `0.95`. Zero or
    /// out of `(0, 1)` — which includes the [`Default`] `0.0` — means 0.95.
    pub confidence: f64,
}

/// The outcome of a curve-fit run. Port of `CurveFitter.FitResult`, plus the
/// Phase 4.2 uncertainty block.
#[derive(Debug, Clone, PartialEq)]
pub struct FitResult {
    pub fitted_parameters: Vec<f64>,
    /// The parameter names, **lowercased** — the Java lowercases them to match
    /// the AST convention and reports the lowercased list.
    pub parameter_names: Vec<String>,
    /// `1 − SSres/SStot`, or exactly `1.0` when the observations have zero
    /// spread (the Java `ssTot == 0.0` guard). Unweighted even for a weighted
    /// fit — it describes the raw residuals, as the Java's does.
    pub r_squared: f64,
    /// `sqrt(SSres / n)` — divided by `n`, not by the degrees of freedom, and
    /// unweighted for the same reason as [`r_squared`](Self::r_squared).
    pub rmse: f64,
    /// Optimizer iterations, i.e. Commons Math's `Optimum.getIterations()`.
    pub iterations: usize,
    /// `observed − fitted`, per data point. Raw, not divided by σ.
    pub residuals: Vec<f64>,
    pub fitted_values: Vec<f64>,
    /// Residual degrees of freedom, `n − p`, floored at 0.
    pub residual_dof: usize,
    /// Standard error of each fitted parameter, in `parameter_names` order.
    /// Every entry is `NaN` when the fit is
    /// [unidentifiable](Self::unidentifiable) or has no residual degrees of
    /// freedom — a finite number there would be a lie.
    pub parameter_std_errors: Vec<f64>,
    /// The `p × p` parameter covariance matrix, row-major. Empty whenever the
    /// standard errors are unavailable.
    pub parameter_covariance: Mat,
    /// Numerical rank of the Jacobian at the optimum, weighted when σ was
    /// given. `rank < p` is what makes a fit unidentifiable.
    pub rank: usize,
    /// `s₀ / s_{p−1}` of that Jacobian's singular values; `f64::INFINITY` when
    /// the smallest is zero. Large means the parameters trade off against each
    /// other and the individual standard errors are near-meaningless even
    /// though they are finite.
    pub condition_number: f64,
    /// `rank < p`: the data does not determine every parameter separately.
    pub unidentifiable: bool,
    /// `χ²/dof` over the weighted residuals — the goodness-of-fit check on the
    /// quoted σ. `NaN` without [`sigma`](CurveFitRequest::sigma), because
    /// unweighted residuals have no absolute scale to be compared against, and
    /// `NaN` without residual degrees of freedom.
    pub reduced_chi_square: f64,
    /// Per parameter: does the optimum sit on one of its bounds? The standard
    /// error beside a `true` here was computed as though the parameter were
    /// free, so it overstates how well the data pins it down.
    pub at_bound: Vec<bool>,
    /// The confidence level the bands below were computed at.
    pub confidence: f64,
    /// Confidence band on the *fitted curve* at each data point: how well the
    /// mean response is known. `NaN` wherever the standard errors are.
    pub confidence_band_lo: Vec<f64>,
    pub confidence_band_hi: Vec<f64>,
    /// Prediction band at each data point: where a *new* measurement would
    /// fall. Always wider than the confidence band — it carries the
    /// measurement's own scatter as well as the curve's uncertainty.
    pub prediction_band_lo: Vec<f64>,
    pub prediction_band_hi: Vec<f64>,
}

/// Fits `request.model` to the observed data. Port of `CurveFitter.fit`, plus
/// the Phase 4.2 weighting and uncertainty described in the module docs.
///
/// # Errors
///
/// * [`FreesError::Solver`] — a validation failure (blank model, empty or
///   mismatched data, no parameters, a non-positive or non-finite σ), a model
///   whose dependent variable is not alone on one side, or an optimizer that
///   hit its evaluation/iteration budget or a Commons Math
///   `ConvergenceException`.
/// * [`FreesError::Parse`] — the model equation does not parse.
pub fn fit(request: &CurveFitRequest<'_>) -> Result<FitResult> {
    validate_fit_inputs(request)?;

    let n = request.y_data.len();
    let p = request.parameters.len();

    let model_expr = parse_model_expression(request.model, request.y_variable)?;

    // The AST stores identifiers lowercase, so the fit works in that alphabet.
    let param_lower: Vec<String> = request
        .parameters
        .iter()
        .map(|s| s.to_ascii_lowercase())
        .collect();
    let x_vars_lower: Vec<String> = request
        .x_variables
        .iter()
        .map(|s| s.to_ascii_lowercase())
        .collect();

    // Columns in, rows out: the model is evaluated one data point at a time and
    // the finite-difference loop walks each point 2p times, so the transpose is
    // paid once rather than per evaluation.
    let x_rows: Vec<Vec<f64>> = (0..n)
        .map(|i| request.x_data.iter().map(|col| col[i]).collect())
        .collect();

    let start: Vec<f64> = (0..p)
        .map(|i| {
            request
                .initial_guess
                .and_then(|g| g.get(i).copied())
                .unwrap_or(1.0)
        })
        .collect();

    let bounds = box_constraints(request, &start)?;

    // `1/σᵢ` per point, or `None` when no σ was given. The unweighted case must
    // not merely *equal* one — it must skip the multiplication entirely,
    // because scaling residuals and Jacobian rows by an exact 1.0 is still a
    // different float operation and the oracle goldens pin the iterate path.
    let base_weights: Option<Vec<f64>> = request
        .sigma
        .map(|sigma| sigma.iter().map(|s| 1.0 / s).collect());

    // Iteratively reweighted least squares. `Loss::Linear` runs the body once
    // with `effective == base_weights` and breaks, which is the plain LM path.
    let mut effective = base_weights.clone();
    let mut point = start;
    let mut iterations = 0usize;
    let mut passes = 0usize;
    loop {
        let observed: Vec<f64> = match &effective {
            None => request.y_data.to_vec(),
            Some(w) => request.y_data.iter().zip(w).map(|(y, w)| y * w).collect(),
        };
        let weights = effective.as_deref();

        // The model + central-difference Jacobian, i.e. `buildModelFunction`.
        let model_function = |params: &[f64]| -> (Vec<f64>, Mat) {
            let mut values = vec![0.0; n];
            let mut jacobian = vec![vec![0.0; p]; n];
            for i in 0..n {
                let xi = &x_rows[i];
                values[i] = evaluate(&model_expr, &x_vars_lower, xi, &param_lower, params);
                for j in 0..p {
                    let h = FD_STEP.max(params[j].abs() * FD_STEP);
                    let mut plus = params.to_vec();
                    let mut minus = params.to_vec();
                    plus[j] += h;
                    minus[j] -= h;
                    let f_plus = evaluate(&model_expr, &x_vars_lower, xi, &param_lower, &plus);
                    let f_minus = evaluate(&model_expr, &x_vars_lower, xi, &param_lower, &minus);
                    jacobian[i][j] = (f_plus - f_minus) / (2.0 * h);
                }
                if let Some(w) = weights {
                    values[i] *= w[i];
                    for j in 0..p {
                        jacobian[i][j] *= w[i];
                    }
                }
            }
            (values, jacobian)
        };

        let optimum = levenberg_marquardt(&point, &observed, model_function, bounds.as_deref())?;
        iterations += optimum.iterations;
        let settled = passes > 0 && converged(&point, &optimum.point);
        point = optimum.point;
        passes += 1;

        if request.loss == Loss::Linear || passes >= MAX_IRLS_PASSES || settled {
            break;
        }

        // Reweight against the residuals this pass actually left behind.
        let scaled: Vec<f64> = (0..n)
            .map(|i| {
                let r = request.y_data[i]
                    - evaluate(&model_expr, &x_vars_lower, &x_rows[i], &param_lower, &point);
                match &base_weights {
                    None => r,
                    Some(w) => r * w[i],
                }
            })
            .collect();
        let scale = request.f_scale.unwrap_or_else(|| robust_scale(&scaled));
        effective = Some(
            (0..n)
                .map(|i| {
                    let robust = request.loss.weight(scaled[i] / scale).max(0.0).sqrt();
                    match &base_weights {
                        None => robust,
                        Some(w) => w[i] * robust,
                    }
                })
                .collect(),
        );
    }
    let fitted = point;

    // Fitted values, residuals, R² and RMSE — recomputed at the optimum rather
    // than reused from the optimizer's last evaluation, as the Java does, and
    // in the *unweighted* alphabet the Java reports.
    let mut fitted_values = vec![0.0; n];
    let mut residuals = vec![0.0; n];
    let mut ss_res = 0.0;
    let mut chi_square = 0.0;
    let y_mean = request.y_data.iter().sum::<f64>() / n as f64;
    let mut ss_tot = 0.0;
    for i in 0..n {
        fitted_values[i] = evaluate(
            &model_expr,
            &x_vars_lower,
            &x_rows[i],
            &param_lower,
            &fitted,
        );
        residuals[i] = request.y_data[i] - fitted_values[i];
        ss_res += residuals[i] * residuals[i];
        if let Some(w) = &base_weights {
            let scaled = residuals[i] * w[i];
            chi_square += scaled * scaled;
        }
        ss_tot += (request.y_data[i] - y_mean) * (request.y_data[i] - y_mean);
    }

    let r_squared = if ss_tot == 0.0 {
        1.0
    } else {
        1.0 - ss_res / ss_tot
    };
    let rmse = (ss_res / n as f64).sqrt();

    // The Jacobian the uncertainty is read off is the one the final pass
    // minimised — for a robust fit that is the reweighted one, which is what
    // makes its covariance an approximation and not the exact thing.
    let final_jacobian = {
        let mut jacobian = vec![vec![0.0; p]; n];
        for i in 0..n {
            for j in 0..p {
                let h = FD_STEP.max(fitted[j].abs() * FD_STEP);
                let mut plus = fitted.clone();
                let mut minus = fitted.clone();
                plus[j] += h;
                minus[j] -= h;
                let f_plus = evaluate(&model_expr, &x_vars_lower, &x_rows[i], &param_lower, &plus);
                let f_minus =
                    evaluate(&model_expr, &x_vars_lower, &x_rows[i], &param_lower, &minus);
                jacobian[i][j] = (f_plus - f_minus) / (2.0 * h);
                if let Some(w) = &effective {
                    jacobian[i][j] *= w[i];
                }
            }
        }
        jacobian
    };

    let uncertainty = parameter_uncertainty(
        &final_jacobian,
        p,
        n,
        if base_weights.is_some() {
            chi_square
        } else {
            ss_res
        },
        base_weights.is_some(),
    );

    let confidence = if request.confidence > 0.0 && request.confidence < 1.0 {
        request.confidence
    } else {
        0.95
    };
    let bands = prediction_bands(
        &final_jacobian,
        &fitted_values,
        &uncertainty,
        request.sigma,
        confidence,
    );

    let at_bound = bounds.as_ref().map_or_else(
        || vec![false; p],
        |box_| {
            (0..p)
                .map(|j| fitted[j] <= box_[j].0 || fitted[j] >= box_[j].1)
                .collect()
        },
    );

    Ok(FitResult {
        fitted_parameters: fitted,
        parameter_names: param_lower,
        r_squared,
        rmse,
        iterations,
        residuals,
        fitted_values,
        residual_dof: uncertainty.dof,
        parameter_std_errors: uncertainty.std_errors,
        parameter_covariance: uncertainty.covariance,
        rank: uncertainty.rank,
        condition_number: uncertainty.condition_number,
        unidentifiable: uncertainty.unidentifiable,
        reduced_chi_square: uncertainty.reduced_chi_square,
        at_bound,
        confidence,
        confidence_band_lo: bands.confidence_lo,
        confidence_band_hi: bands.confidence_hi,
        prediction_band_lo: bands.prediction_lo,
        prediction_band_hi: bands.prediction_hi,
    })
}

/// The four band vectors, one entry per data point.
struct Bands {
    confidence_lo: Vec<f64>,
    confidence_hi: Vec<f64>,
    prediction_lo: Vec<f64>,
    prediction_hi: Vec<f64>,
}

/// Confidence and prediction bands on the fitted curve, evaluated at the data's
/// own predictors.
///
/// The curve's variance at a point is `jᵀ Σ j` with `j = ∂f/∂p` there — the
/// delta method, and local-linear for exactly the same reason the covariance
/// is. The half-width is `t(1 − α/2, dof) · se`. The prediction band adds the
/// measurement's own variance to the curve's: `σᵢ²` where σ was given, and the
/// estimated residual variance otherwise.
///
/// All `NaN` when the covariance was unavailable — a band drawn around an
/// unidentifiable fit would be a picture of a number that does not exist.
fn prediction_bands(
    jacobian: &Mat,
    fitted_values: &[f64],
    uncertainty: &Uncertainty,
    sigma: Option<&[f64]>,
    confidence: f64,
) -> Bands {
    let n = fitted_values.len();
    let nan_bands = || Bands {
        confidence_lo: vec![f64::NAN; n],
        confidence_hi: vec![f64::NAN; n],
        prediction_lo: vec![f64::NAN; n],
        prediction_hi: vec![f64::NAN; n],
    };
    if uncertainty.covariance.is_empty() || uncertainty.dof == 0 {
        return nan_bands();
    }
    let Ok(t_crit) =
        crate::eval::student_t_inv(1.0 - (1.0 - confidence) / 2.0, uncertainty.dof as f64)
    else {
        return nan_bands();
    };

    let p = uncertainty.covariance.len();
    // The residual variance a new measurement carries. With σ it is that point's
    // own; without, the one estimated from the residuals — which is precisely
    // the `scale` the covariance was multiplied by, recovered from it.
    let estimated_variance = uncertainty.residual_variance;

    let mut bands = Bands {
        confidence_lo: Vec::with_capacity(n),
        confidence_hi: Vec::with_capacity(n),
        prediction_lo: Vec::with_capacity(n),
        prediction_hi: Vec::with_capacity(n),
    };
    for i in 0..n {
        // The Jacobian rows are σ-weighted when σ was given; the band is wanted
        // in the observation's own units, so that weighting comes back off.
        let unweight = sigma.map_or(1.0, |s| s[i]);
        let mut variance = 0.0;
        for j in 0..p {
            for k in 0..p {
                variance += jacobian[i][j]
                    * unweight
                    * uncertainty.covariance[j][k]
                    * jacobian[i][k]
                    * unweight;
            }
        }
        let se_fit = variance.max(0.0).sqrt();
        let point_variance = sigma.map_or(estimated_variance, |s| s[i] * s[i]);
        let se_new = (variance.max(0.0) + point_variance).sqrt();
        bands.confidence_lo.push(fitted_values[i] - t_crit * se_fit);
        bands.confidence_hi.push(fitted_values[i] + t_crit * se_fit);
        bands.prediction_lo.push(fitted_values[i] - t_crit * se_new);
        bands.prediction_hi.push(fitted_values[i] + t_crit * se_new);
    }
    bands
}

/// Have the parameters stopped moving between IRLS passes?
fn converged(previous: &[f64], current: &[f64]) -> bool {
    previous
        .iter()
        .zip(current)
        .all(|(a, b)| (a - b).abs() <= IRLS_PARAM_TOL * a.abs().max(1.0))
}

/// The residuals' own robust spread, `1.4826 · MAD`, for a robust loss that was
/// given no `f_scale`. Falls back to the mean absolute residual and then to 1
/// so that a degenerate residual vector cannot divide by zero — a zero scale
/// would send every weight to `ρ'(∞)` and freeze the fit.
fn robust_scale(residuals: &[f64]) -> f64 {
    if let Ok(mad) = crate::descriptive::median_abs_deviation(residuals) {
        let scale = 1.4826 * mad;
        if scale > 0.0 {
            return scale;
        }
    }
    let mean_abs = residuals.iter().map(|r| r.abs()).sum::<f64>() / residuals.len().max(1) as f64;
    if mean_abs > 0.0 {
        mean_abs
    } else {
        1.0
    }
}

/// Validates and pairs up the box constraints, in the
/// [`crate::analysis::paramfit`] words. `None` when either side was omitted.
fn box_constraints(
    request: &CurveFitRequest<'_>,
    start: &[f64],
) -> Result<Option<Vec<(f64, f64)>>> {
    let (Some(lower), Some(upper)) = (request.lower, request.upper) else {
        return Ok(None);
    };
    let p = request.parameters.len();
    if lower.len() != p || upper.len() != p {
        return Err(FreesError::solver(
            "Parameter bounds need one lower and one upper value per fitted parameter.",
        ));
    }
    for i in 0..p {
        if !lower[i].is_finite() || !upper[i].is_finite() || lower[i] >= upper[i] {
            return Err(FreesError::solver(format!(
                "Bounds for {} must be finite with lower < upper.",
                request.parameters[i]
            )));
        }
        if start[i] < lower[i] || start[i] > upper[i] {
            return Err(FreesError::solver(format!(
                "The initial value of {} lies outside its bounds.",
                request.parameters[i]
            )));
        }
    }
    Ok(Some(
        lower.iter().copied().zip(upper.iter().copied()).collect(),
    ))
}

/// What [`parameter_uncertainty`] reads off the Jacobian at the optimum.
pub(crate) struct Uncertainty {
    pub(crate) dof: usize,
    /// The residual variance a new observation carries: `SSres / dof` for an
    /// unweighted fit, and `1` for a weighted one, where the σ that scaled the
    /// residuals already carries it.
    pub(crate) residual_variance: f64,
    pub(crate) std_errors: Vec<f64>,
    pub(crate) covariance: Mat,
    pub(crate) rank: usize,
    pub(crate) condition_number: f64,
    pub(crate) unidentifiable: bool,
    pub(crate) reduced_chi_square: f64,
}

/// Parameter covariance from an SVD of the Jacobian at the optimum.
///
/// `Σ = scale · V S⁻² Vᵀ`, summed only over singular values above the
/// rank tolerance — the pseudo-inverse form, so a rank-deficient Jacobian
/// yields a finite matrix instead of an overflow. The SVD is deliberate:
/// forming `JᵀJ` and inverting it squares the condition number, which is
/// exactly the information this function exists to report.
///
/// `scale` is `1` for a weighted fit — σ was believed, so the covariance is
/// already absolute — and the estimated residual variance `SSres / dof`
/// otherwise.
///
/// When the fit is rank-deficient or has no residual degrees of freedom, the
/// standard errors come back `NaN` and the covariance empty. A finite standard
/// error on a parameter the data cannot separate is worse than no number.
pub(crate) fn parameter_uncertainty(
    jacobian: &Mat,
    p: usize,
    n: usize,
    residual_sum: f64,
    weighted: bool,
) -> Uncertainty {
    let dof = n.saturating_sub(p);
    // Unweighted: the scatter has to be estimated. Weighted: sigma already
    // carries it and the residuals are dimensionless, so it is 1.
    let residual_variance = if weighted {
        1.0
    } else if dof > 0 {
        residual_sum / dof as f64
    } else {
        f64::NAN
    };
    let unavailable = |rank: usize, condition_number: f64, unidentifiable: bool| Uncertainty {
        dof,
        residual_variance,
        std_errors: vec![f64::NAN; p],
        covariance: Vec::new(),
        rank,
        condition_number,
        unidentifiable,
        reduced_chi_square: f64::NAN,
    };

    let Ok(decomposition) = crate::linalg::svd(jacobian) else {
        return unavailable(0, f64::INFINITY, true);
    };
    let s = &decomposition.s;
    let v = &decomposition.v;
    // Commons Math's `SingularValueDecomposition` rank rule, as `SvdSolver`
    // uses it: max(m·s₀·EPS, sqrt(SAFE_MIN)).
    let tol = (n as f64 * s.first().copied().unwrap_or(0.0) * TWO_EPS).max(SAFE_MIN.sqrt());
    let rank = s.iter().filter(|value| **value > tol).count();
    let smallest = s.last().copied().unwrap_or(0.0);
    let condition_number = if smallest > 0.0 {
        s[0] / smallest
    } else {
        f64::INFINITY
    };
    let unidentifiable = rank < p;

    let reduced_chi_square = if weighted && dof > 0 {
        residual_sum / dof as f64
    } else {
        f64::NAN
    };

    if unidentifiable || dof == 0 || v.len() < p {
        return Uncertainty {
            reduced_chi_square,
            ..unavailable(rank, condition_number, unidentifiable)
        };
    }

    let scale = if weighted {
        1.0
    } else {
        residual_sum / dof as f64
    };
    let mut covariance = vec![vec![0.0; p]; p];
    for j in 0..p {
        for k in 0..p {
            let mut sum = 0.0;
            for l in 0..rank {
                sum += v[j][l] * v[k][l] / (s[l] * s[l]);
            }
            covariance[j][k] = scale * sum;
        }
    }
    let std_errors = (0..p).map(|j| covariance[j][j].sqrt()).collect();

    Uncertainty {
        dof,
        residual_variance,
        std_errors,
        covariance,
        rank,
        condition_number,
        unidentifiable,
        reduced_chi_square,
    }
}

/// Port of `CurveFitter.validateFitInputs`, message for message, extended with
/// the Phase 4.2 shape rules for multiple predictors and σ.
fn validate_fit_inputs(request: &CurveFitRequest<'_>) -> Result<()> {
    if request.model.trim().is_empty() {
        return Err(FreesError::solver("Model equation is required."));
    }
    if request.x_variables.is_empty() {
        return Err(FreesError::solver(
            "At least one independent variable is required.",
        ));
    }
    if request.x_data.len() != request.x_variables.len() {
        return Err(FreesError::solver(format!(
            "Expected one data column per independent variable (got {} column(s) for {} variable(s)).",
            request.x_data.len(),
            request.x_variables.len()
        )));
    }
    if request.x_data.iter().all(Vec::is_empty) || request.y_data.is_empty() {
        return Err(FreesError::solver("Data points are required."));
    }
    for (name, column) in request.x_variables.iter().zip(request.x_data) {
        if column.len() != request.y_data.len() {
            return Err(FreesError::solver(format!(
                "x and y data must have the same length (got {} and {}) for '{name}'.",
                column.len(),
                request.y_data.len()
            )));
        }
    }
    if request.parameters.is_empty() {
        return Err(FreesError::solver(
            "At least one parameter to fit is required.",
        ));
    }
    if let Some(sigma) = request.sigma {
        if sigma.len() != request.y_data.len() {
            return Err(FreesError::solver(format!(
                "Measurement standard deviations must have one entry per data point (got {} and {}).",
                sigma.len(),
                request.y_data.len()
            )));
        }
        if let Some(bad) = sigma.iter().find(|s| !s.is_finite() || **s <= 0.0) {
            return Err(FreesError::solver(format!(
                "Measurement standard deviations must be finite and positive (got {bad})."
            )));
        }
    }
    Ok(())
}

/// Parses the model equation and returns the side that is *not* the dependent
/// variable. Port of `CurveFitter.parseModelExpression`, including its refusal
/// to guess when neither side is the bare dependent variable.
fn parse_model_expression(model: &str, y_variable: &str) -> Result<Expr> {
    let doc = crate::parser::parse_document(model)?;
    let equations: Vec<Equation> = crate::parser::expand::expand_document(&doc)?;
    let Some(eq) = equations.into_iter().next() else {
        return Err(FreesError::solver("Model equation could not be parsed."));
    };

    let y_lower = y_variable.to_ascii_lowercase();
    if matches!(&eq.lhs, Expr::Var(name) if *name == y_lower) {
        return Ok(eq.rhs);
    }
    if matches!(&eq.rhs, Expr::Var(name) if *name == y_lower) {
        return Ok(eq.lhs);
    }
    Err(FreesError::solver(format!(
        "Could not identify '{y_variable}' as the dependent variable in the model equation. \
         Expected a form like '{y_variable} = <expression>'."
    )))
}

/// Evaluates the model at one data point. Port of `CurveFitter.evaluate`: an
/// evaluation failure becomes `NaN` rather than an error, so the optimizer sees
/// a hostile point instead of a crash.
///
/// The Java uses the two-argument `Evaluator.eval(expr, values)` — no `defs` —
/// so a model equation cannot call a document `FUNCTION` or `TABLE`. Mirrored.
fn evaluate(
    expr: &Expr,
    x_vars: &[String],
    x_row: &[f64],
    param_names: &[String],
    param_values: &[f64],
) -> f64 {
    let mut scope: Scope =
        Scope::with_capacity_and_hasher(param_names.len() + x_vars.len(), Default::default());
    for (name, value) in x_vars.iter().zip(x_row) {
        scope.insert(name.clone(), *value);
    }
    for (name, value) in param_names.iter().zip(param_values) {
        scope.insert(name.clone(), *value);
    }
    eval(expr, &scope).unwrap_or(f64::NAN)
}

// ---------------------------------------------------------------------------
// Commons Math `LevenbergMarquardtOptimizer` (MINPACK lmder)
// ---------------------------------------------------------------------------

/// One evaluation of the least-squares problem: the point, the residuals
/// `target − model(point)`, the cost `‖residuals‖₂` and the model Jacobian.
/// Commons Math's `LeastSquaresProblem.Evaluation` for the unweighted case.
#[derive(Clone)]
struct LmEvaluation {
    point: Vec<f64>,
    residuals: Vec<f64>,
    cost: f64,
    jacobian: Mat,
}

/// What `Optimum` carries back out of the optimizer.
struct LmOptimum {
    point: Vec<f64>,
    iterations: usize,
}

/// The QR-with-column-pivoting state Commons Math calls `InternalData`.
struct InternalData {
    /// **Negated** Jacobian, overwritten in place by the Householder
    /// reflections — the Java comment reads "Code in this class assumes that the
    /// weighted Jacobian is -(W^(1/2) J), hence the multiplication by -1".
    weighted_jacobian: Mat,
    permutation: Vec<usize>,
    rank: usize,
    diag_r: Vec<f64>,
    jac_norm: Vec<f64>,
    beta: Vec<f64>,
}

/// The scratch vectors the Java allocates once per `optimize` call.
struct LmScratch {
    lm_dir: Vec<f64>,
    diag: Vec<f64>,
    old_x: Vec<f64>,
    qtf: Vec<f64>,
    work1: Vec<f64>,
    work2: Vec<f64>,
    work3: Vec<f64>,
}

/// Runs Commons Math's `LevenbergMarquardtOptimizer.optimize`.
///
/// `model(point) -> (values, jacobian)` is the `MultivariateJacobianFunction`;
/// residuals are `target − values`, exactly as `computeResiduals` defines them.
fn levenberg_marquardt<F>(
    start: &[f64],
    target: &[f64],
    mut model: F,
    bounds: Option<&[(f64, f64)]>,
) -> Result<LmOptimum>
where
    F: FnMut(&[f64]) -> (Vec<f64>, Mat),
{
    let n_r = target.len();
    let n_c = start.len();
    let solved_cols = n_r.min(n_c);

    let mut scratch = LmScratch {
        lm_dir: vec![0.0; n_c],
        diag: vec![0.0; n_c],
        old_x: vec![0.0; n_c],
        qtf: vec![0.0; n_r],
        work1: vec![0.0; n_c],
        work2: vec![0.0; n_c],
        work3: vec![0.0; n_c],
    };
    let mut lm_par = 0.0f64;
    let mut delta = 0.0f64;
    let mut x_norm = 0.0f64;
    let mut old_res = vec![0.0; n_r];

    let mut evaluations = 0usize;
    let mut iterations = 0usize;

    let mut evaluate_at = |point: &[f64], counter: &mut usize| -> Result<LmEvaluation> {
        *counter += 1;
        if *counter > MAX_EVALUATIONS {
            return Err(FreesError::solver(format!(
                "Curve fit gave up: more than {MAX_EVALUATIONS} model evaluations."
            )));
        }
        let (values, jacobian) = model(point);
        let residuals: Vec<f64> = target
            .iter()
            .zip(&values)
            .map(|(t, v)| t - v)
            .collect::<Vec<_>>();
        let cost = residuals.iter().map(|r| r * r).sum::<f64>().sqrt();
        Ok(LmEvaluation {
            point: point.to_vec(),
            residuals,
            cost,
            jacobian,
        })
    };

    let mut current = evaluate_at(start, &mut evaluations)?;
    let mut weighted_residual = current.residuals.clone();
    // Java keeps `currentPoint` as an array of its own, separate from
    // `current.getPoint()`. That matters when there are more parameters than
    // data points: a failed step restores only the first `solvedCols` permuted
    // components, and the rest keep the rejected step's values.
    let mut current_point = current.point.clone();

    let mut first_iteration = true;
    loop {
        iterations += 1;
        if iterations > MAX_ITERATIONS {
            return Err(FreesError::solver(format!(
                "Curve fit gave up: more than {MAX_ITERATIONS} iterations."
            )));
        }
        let previous = current.clone();

        let mut data = qr_decomposition(&current.jacobian, solved_cols, n_r, n_c)?;

        scratch.qtf[..n_r].copy_from_slice(&weighted_residual[..n_r]);
        q_t_y(&mut scratch.qtf, &data, n_r, n_c);

        // Q is no longer needed; let the Jacobian hold R with its diagonal.
        for k in 0..solved_cols {
            let pk = data.permutation[k];
            data.weighted_jacobian[k][pk] = data.diag_r[pk];
        }

        if first_iteration {
            // Scale the point by the initial Jacobian's column norms.
            x_norm = 0.0;
            for k in 0..n_c {
                let mut dk = data.jac_norm[k];
                if dk == 0.0 {
                    dk = 1.0;
                }
                let xk = dk * current_point[k];
                x_norm += xk * xk;
                scratch.diag[k] = dk;
            }
            x_norm = x_norm.sqrt();
            delta = if x_norm == 0.0 {
                INITIAL_STEP_BOUND_FACTOR
            } else {
                INITIAL_STEP_BOUND_FACTOR * x_norm
            };
        }

        // Orthogonality between the residual vector and the Jacobian columns.
        let mut max_cosine = 0.0f64;
        if current.cost != 0.0 {
            for j in 0..solved_cols {
                let pj = data.permutation[j];
                let s = data.jac_norm[pj];
                if s != 0.0 {
                    let mut sum = 0.0;
                    for i in 0..=j {
                        sum += data.weighted_jacobian[i][pj] * scratch.qtf[i];
                    }
                    max_cosine = max_cosine.max(sum.abs() / (s * current.cost));
                }
            }
        }
        if max_cosine <= ORTHO_TOLERANCE {
            return Ok(LmOptimum {
                point: current.point,
                iterations,
            });
        }

        for j in 0..n_c {
            scratch.diag[j] = scratch.diag[j].max(data.jac_norm[j]);
        }

        // Inner loop: shrink the trust region until a step is accepted.
        let mut ratio = 0.0f64;
        while ratio < 1.0e-4 {
            for j in 0..solved_cols {
                let pj = data.permutation[j];
                scratch.old_x[pj] = current_point[pj];
            }
            let previous_cost = current.cost;
            std::mem::swap(&mut weighted_residual, &mut old_res);

            lm_par = determine_lm_parameter(
                &scratch.qtf.clone(),
                delta,
                &mut scratch,
                &mut data,
                solved_cols,
                lm_par,
            );

            // The new point, and the norm of the step that reaches it.
            let mut lm_norm = 0.0;
            for j in 0..solved_cols {
                let pj = data.permutation[j];
                scratch.lm_dir[pj] = -scratch.lm_dir[pj];
                current_point[pj] = scratch.old_x[pj] + scratch.lm_dir[pj];
                // Projected LM. The step actually taken is the clamped one, so
                // the trust-region bookkeeping below must measure *that*, not
                // the unconstrained step it was derived from. Left strictly
                // alone when unbounded: recomputing `lm_dir` as
                // `(old_x + d) - old_x` is not bit-identical to `d`, and the
                // oracle goldens pin this path float for float.
                if let Some(box_) = bounds {
                    let clamped = current_point[pj].clamp(box_[pj].0, box_[pj].1);
                    if clamped != current_point[pj] {
                        current_point[pj] = clamped;
                        scratch.lm_dir[pj] = clamped - scratch.old_x[pj];
                    }
                }
                let s = scratch.diag[pj] * scratch.lm_dir[pj];
                lm_norm += s * s;
            }
            let lm_norm = lm_norm.sqrt();
            if first_iteration {
                delta = delta.min(lm_norm);
            }

            current = evaluate_at(&current_point, &mut evaluations)?;
            weighted_residual = current.residuals.clone();
            current_point.clone_from(&current.point);
            let current_cost = current.cost;

            // Scaled actual reduction.
            let mut act_red = -1.0;
            if 0.1 * current_cost < previous_cost {
                let r = current_cost / previous_cost;
                act_red = 1.0 - r * r;
            }

            // Scaled predicted reduction and directional derivative.
            for j in 0..solved_cols {
                let pj = data.permutation[j];
                let dir_j = scratch.lm_dir[pj];
                scratch.work1[j] = 0.0;
                for i in 0..=j {
                    scratch.work1[i] += data.weighted_jacobian[i][pj] * dir_j;
                }
            }
            let mut coeff1 = 0.0;
            for j in 0..solved_cols {
                coeff1 += scratch.work1[j] * scratch.work1[j];
            }
            let pc2 = previous_cost * previous_cost;
            coeff1 /= pc2;
            let coeff2 = lm_par * lm_norm * lm_norm / pc2;
            let pre_red = coeff1 + 2.0 * coeff2;
            let dir_der = -(coeff1 + coeff2);

            ratio = if pre_red == 0.0 {
                0.0
            } else {
                act_red / pre_red
            };

            // Update the step bound.
            if ratio <= 0.25 {
                let mut tmp = if act_red < 0.0 {
                    0.5 * dir_der / (dir_der + 0.5 * act_red)
                } else {
                    0.5
                };
                if (0.1 * current_cost >= previous_cost) || (tmp < 0.1) {
                    tmp = 0.1;
                }
                delta = tmp * delta.min(10.0 * lm_norm);
                lm_par /= tmp;
            } else if lm_par == 0.0 || ratio >= 0.75 {
                delta = 2.0 * lm_norm;
                lm_par *= 0.5;
            }

            if ratio >= 1.0e-4 {
                // Successful iteration: update the scaled point norm.
                first_iteration = false;
                x_norm = 0.0;
                for k in 0..n_c {
                    let xk = scratch.diag[k] * current_point[k];
                    x_norm += xk * xk;
                }
                x_norm = x_norm.sqrt();
                // Commons Math consults the problem's `ConvergenceChecker`
                // here. `LeastSquaresBuilder` was never given one, so it is
                // null and this branch is dead — noted rather than invented.
            } else {
                // Failed iteration: restore everything. Only the first
                // `solvedCols` permuted components come back, exactly as in
                // Java — see the note on `current_point`.
                for j in 0..solved_cols {
                    let pj = data.permutation[j];
                    current_point[pj] = scratch.old_x[pj];
                }
                std::mem::swap(&mut weighted_residual, &mut old_res);
                current = previous.clone();
            }

            // Default convergence criteria.
            if (act_red.abs() <= COST_RELATIVE_TOLERANCE
                && pre_red <= COST_RELATIVE_TOLERANCE
                && ratio <= 2.0)
                || delta <= PAR_RELATIVE_TOLERANCE * x_norm
            {
                return Ok(LmOptimum {
                    point: current.point,
                    iterations,
                });
            }

            // Termination on tolerances too stringent for this arithmetic.
            if act_red.abs() <= TWO_EPS && pre_red <= TWO_EPS && ratio <= 2.0 {
                return Err(FreesError::solver(
                    "Curve fit stalled: the cost relative tolerance (1e-10) is too small — \
                     no further reduction in the sum of squares is possible.",
                ));
            } else if delta <= TWO_EPS * x_norm {
                return Err(FreesError::solver(
                    "Curve fit stalled: the parameters relative tolerance (1e-10) is too small — \
                     no further improvement in the parameters is possible.",
                ));
            } else if max_cosine <= TWO_EPS {
                return Err(FreesError::solver(
                    "Curve fit stalled: the orthogonality tolerance (1e-10) is too small — \
                     the residual vector is already orthogonal to the model's Jacobian.",
                ));
            }
        }
    }
}

/// QR decomposition with column pivoting of the **negated** Jacobian. Port of
/// `LevenbergMarquardtOptimizer.qrDecomposition`.
fn qr_decomposition(
    jacobian: &Mat,
    solved_cols: usize,
    n_r: usize,
    n_c: usize,
) -> Result<InternalData> {
    let mut weighted_jacobian: Mat = jacobian
        .iter()
        .map(|row| row.iter().map(|v| -v).collect())
        .collect();

    let mut permutation: Vec<usize> = (0..n_c).collect();
    let mut diag_r = vec![0.0; n_c];
    let mut jac_norm = vec![0.0; n_c];
    let mut beta = vec![0.0; n_c];

    for k in 0..n_c {
        let mut norm2 = 0.0;
        for i in 0..n_r {
            let akk = weighted_jacobian[i][k];
            norm2 += akk * akk;
        }
        jac_norm[k] = norm2.sqrt();
    }

    for k in 0..n_c {
        // Pick the remaining column with the greatest norm on active rows.
        let mut next_column = usize::MAX;
        let mut ak2 = f64::NEG_INFINITY;
        for i in k..n_c {
            let mut norm2 = 0.0;
            for j in k..n_r {
                let aki = weighted_jacobian[j][permutation[i]];
                norm2 += aki * aki;
            }
            if !norm2.is_finite() {
                return Err(FreesError::solver(
                    "Curve fit failed: the model's Jacobian is not finite — check the \
                     model equation and the starting values.",
                ));
            }
            if norm2 > ak2 {
                next_column = i;
                ak2 = norm2;
            }
        }
        if ak2 <= SAFE_MIN {
            return Ok(InternalData {
                weighted_jacobian,
                permutation,
                rank: k,
                diag_r,
                jac_norm,
                beta,
            });
        }
        permutation.swap(next_column, k);
        let pk = permutation[k];

        // Choose alpha so that Hk·u = alpha·ek.
        let akk = weighted_jacobian[k][pk];
        let alpha = if akk > 0.0 { -ak2.sqrt() } else { ak2.sqrt() };
        let betak = 1.0 / (ak2 - akk * alpha);
        beta[pk] = betak;

        diag_r[pk] = alpha;
        weighted_jacobian[k][pk] -= alpha;

        // Apply the reflection to the remaining columns.
        for dk in (1..(n_c - k)).rev() {
            let pdk = permutation[k + dk];
            let mut gamma = 0.0;
            for j in k..n_r {
                gamma += weighted_jacobian[j][pk] * weighted_jacobian[j][pdk];
            }
            gamma *= betak;
            for j in k..n_r {
                let v = weighted_jacobian[j][pk];
                weighted_jacobian[j][pdk] -= gamma * v;
            }
        }
    }

    Ok(InternalData {
        weighted_jacobian,
        permutation,
        rank: solved_cols,
        diag_r,
        jac_norm,
        beta,
    })
}

/// `y ← Qᵀ·y` using the reflections stored in the decomposition. Port of
/// `LevenbergMarquardtOptimizer.qTy`.
fn q_t_y(y: &mut [f64], data: &InternalData, n_r: usize, n_c: usize) {
    for k in 0..n_c {
        let pk = data.permutation[k];
        let mut gamma = 0.0;
        for i in k..n_r {
            gamma += data.weighted_jacobian[i][pk] * y[i];
        }
        gamma *= data.beta[pk];
        for i in k..n_r {
            y[i] -= gamma * data.weighted_jacobian[i][pk];
        }
    }
}

/// The Levenberg–Marquardt parameter for this step, by MINPACK's `lmpar`
/// bisection-with-Newton on `‖D·p(λ)‖ − Δ`. Port of
/// `LevenbergMarquardtOptimizer.determineLMParameter`.
fn determine_lm_parameter(
    qy: &[f64],
    delta: f64,
    scratch: &mut LmScratch,
    data: &mut InternalData,
    solved_cols: usize,
    lm_par_in: f64,
) -> f64 {
    let mut lm_par = lm_par_in;
    let n_c = data.weighted_jacobian.first().map_or(0, Vec::len);
    let rank = data.rank;

    // The Gauss-Newton direction (least-squares one if R is rank-deficient).
    for j in 0..rank {
        scratch.lm_dir[data.permutation[j]] = qy[j];
    }
    for j in rank..n_c {
        scratch.lm_dir[data.permutation[j]] = 0.0;
    }
    for k in (0..rank).rev() {
        let pk = data.permutation[k];
        let ypk = scratch.lm_dir[pk] / data.diag_r[pk];
        for i in 0..k {
            scratch.lm_dir[data.permutation[i]] -= ypk * data.weighted_jacobian[i][pk];
        }
        scratch.lm_dir[pk] = ypk;
    }

    // Accept the Gauss-Newton step if it already sits inside the trust region.
    let mut dx_norm = 0.0;
    for j in 0..solved_cols {
        let pj = data.permutation[j];
        let s = scratch.diag[pj] * scratch.lm_dir[pj];
        scratch.work1[pj] = s;
        dx_norm += s * s;
    }
    let mut dx_norm = dx_norm.sqrt();
    let mut fp = dx_norm - delta;
    if fp <= 0.1 * delta {
        return 0.0;
    }

    // Lower bound `parl` — zero when R is rank-deficient.
    let mut parl = 0.0;
    if rank == solved_cols {
        for j in 0..solved_cols {
            let pj = data.permutation[j];
            scratch.work1[pj] *= scratch.diag[pj] / dx_norm;
        }
        let mut sum2 = 0.0;
        for j in 0..solved_cols {
            let pj = data.permutation[j];
            let mut sum = 0.0;
            for i in 0..j {
                sum += data.weighted_jacobian[i][pj] * scratch.work1[data.permutation[i]];
            }
            let s = (scratch.work1[pj] - sum) / data.diag_r[pj];
            scratch.work1[pj] = s;
            sum2 += s * s;
        }
        parl = fp / (delta * sum2);
    }

    // Upper bound `paru`.
    let mut sum2 = 0.0;
    for j in 0..solved_cols {
        let pj = data.permutation[j];
        let mut sum = 0.0;
        for i in 0..=j {
            sum += data.weighted_jacobian[i][pj] * qy[i];
        }
        sum /= scratch.diag[pj];
        sum2 += sum * sum;
    }
    let g_norm = sum2.sqrt();
    let mut paru = g_norm / delta;
    if paru == 0.0 {
        paru = SAFE_MIN / delta.min(0.1);
    }

    lm_par = paru.min(lm_par.max(parl));
    if lm_par == 0.0 {
        lm_par = g_norm / dx_norm;
    }

    for _ in 0..=10 {
        if lm_par == 0.0 {
            lm_par = SAFE_MIN.max(0.001 * paru);
        }
        let s_par = lm_par.sqrt();
        for j in 0..solved_cols {
            let pj = data.permutation[j];
            scratch.work1[pj] = s_par * scratch.diag[pj];
        }
        determine_lm_direction(qy, scratch, data, solved_cols);

        dx_norm = 0.0;
        for j in 0..solved_cols {
            let pj = data.permutation[j];
            let s = scratch.diag[pj] * scratch.lm_dir[pj];
            scratch.work3[pj] = s;
            dx_norm += s * s;
        }
        dx_norm = dx_norm.sqrt();
        let previous_fp = fp;
        fp = dx_norm - delta;

        // Close enough, or the exceptional `parl == 0` case.
        if fp.abs() <= 0.1 * delta || (parl == 0.0 && fp <= previous_fp && previous_fp < 0.0) {
            return lm_par;
        }

        // Newton correction.
        for j in 0..solved_cols {
            let pj = data.permutation[j];
            scratch.work1[pj] = scratch.work3[pj] * scratch.diag[pj] / dx_norm;
        }
        for j in 0..solved_cols {
            let pj = data.permutation[j];
            scratch.work1[pj] /= scratch.work2[j];
            let tmp = scratch.work1[pj];
            for i in (j + 1)..solved_cols {
                scratch.work1[data.permutation[i]] -= data.weighted_jacobian[i][pj] * tmp;
            }
        }
        let mut sum2 = 0.0;
        for j in 0..solved_cols {
            let s = scratch.work1[data.permutation[j]];
            sum2 += s * s;
        }
        let correction = fp / (delta * sum2);

        if fp > 0.0 {
            parl = parl.max(lm_par);
        } else if fp < 0.0 {
            paru = paru.min(lm_par);
        }
        lm_par = parl.max(lm_par + correction);
    }

    lm_par
}

/// Solves `(RᵀR + DᵀD)·x = Rᵀ·Qᵀ·y` by Givens elimination of `D`, MINPACK's
/// `qrsolv`. Port of `LevenbergMarquardtOptimizer.determineLMDirection`.
///
/// `scratch.work1` is the scaled diagonal `D`, `scratch.work2` receives the
/// diagonal of `S`, and `scratch.work3` is the work vector. The strictly-lower
/// triangle of `weighted_jacobian` is left holding `S`, which
/// [`determine_lm_parameter`] then reads for its Newton correction — that
/// aliasing is the reference's, kept deliberately.
fn determine_lm_direction(
    qy: &[f64],
    scratch: &mut LmScratch,
    data: &mut InternalData,
    solved_cols: usize,
) {
    // Copy R and Qᵀy, saving R's diagonal in `lm_dir`.
    for j in 0..solved_cols {
        let pj = data.permutation[j];
        for i in (j + 1)..solved_cols {
            data.weighted_jacobian[i][pj] = data.weighted_jacobian[j][data.permutation[i]];
        }
        scratch.lm_dir[j] = data.diag_r[pj];
        scratch.work3[j] = qy[j];
    }

    // Eliminate the diagonal matrix D with Givens rotations.
    for j in 0..solved_cols {
        let pj = data.permutation[j];
        let dpj = scratch.work1[pj];
        if dpj != 0.0 {
            // The Java fills to `lmDiag.length` — the full nC, not solvedCols.
            for slot in scratch.work2.iter_mut().skip(j + 1) {
                *slot = 0.0;
            }
        }
        scratch.work2[j] = dpj;

        let mut qtbpj = 0.0;
        for k in j..solved_cols {
            let pk = data.permutation[k];
            if scratch.work2[k] != 0.0 {
                let rkk = data.weighted_jacobian[k][pk];
                let (sin, cos) = if rkk.abs() < scratch.work2[k].abs() {
                    let cotan = rkk / scratch.work2[k];
                    let sin = 1.0 / (1.0 + cotan * cotan).sqrt();
                    (sin, sin * cotan)
                } else {
                    let tan = scratch.work2[k] / rkk;
                    let cos = 1.0 / (1.0 + tan * tan).sqrt();
                    (cos * tan, cos)
                };

                data.weighted_jacobian[k][pk] = cos * rkk + sin * scratch.work2[k];
                let temp = cos * scratch.work3[k] + sin * qtbpj;
                qtbpj = -sin * scratch.work3[k] + cos * qtbpj;
                scratch.work3[k] = temp;

                for i in (k + 1)..solved_cols {
                    let rik = data.weighted_jacobian[i][pk];
                    let temp2 = cos * rik + sin * scratch.work2[i];
                    scratch.work2[i] = -sin * rik + cos * scratch.work2[i];
                    data.weighted_jacobian[i][pk] = temp2;
                }
            }
        }

        // Store S's diagonal element and restore R's.
        scratch.work2[j] = data.weighted_jacobian[j][data.permutation[j]];
        data.weighted_jacobian[j][data.permutation[j]] = scratch.lm_dir[j];
    }

    // Back-substitute; a singular system yields a least-squares solution.
    let mut n_sing = solved_cols;
    for j in 0..solved_cols {
        if scratch.work2[j] == 0.0 && n_sing == solved_cols {
            n_sing = j;
        }
        if n_sing < solved_cols {
            scratch.work3[j] = 0.0;
        }
    }
    if n_sing > 0 {
        for j in (0..n_sing).rev() {
            let pj = data.permutation[j];
            let mut sum = 0.0;
            for i in (j + 1)..n_sing {
                sum += data.weighted_jacobian[i][pj] * scratch.work3[i];
            }
            scratch.work3[j] = (scratch.work3[j] - sum) / scratch.work2[j];
        }
    }

    for j in 0..scratch.lm_dir.len() {
        scratch.lm_dir[data.permutation[j]] = scratch.work3[j];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every expectation below is the Java `CurveFitter`'s own output, captured
    /// by running the reference engine off `tools/golden-dumper/classpath.sh`
    /// with the identical model, data and start. `iterations` is quoted too —
    /// it is the value that would drift first if the damping schedule were
    /// paraphrased rather than transcribed.
    fn close(actual: f64, expected: f64, tol: f64) {
        assert!(
            (actual - expected).abs() <= tol * expected.abs().max(1.0),
            "expected {expected}, got {actual} (tol {tol})"
        );
    }

    fn names(list: &[String]) -> Vec<&str> {
        list.iter().map(String::as_str).collect()
    }

    /// A single-predictor request named `x`/`y`, the shape every oracle golden
    /// below uses.
    fn request<'a>(
        model: &'a str,
        params: &'a [String],
        x: &'a [Vec<f64>],
        y: &'a [f64],
        start: Option<&'a [f64]>,
    ) -> CurveFitRequest<'a> {
        CurveFitRequest {
            model,
            y_variable: "y",
            x_variables: X_ONLY.get_or_init(|| vec!["x".to_string()]),
            parameters: params,
            x_data: x,
            y_data: y,
            initial_guess: start,
            ..Default::default()
        }
    }

    static X_ONLY: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();

    fn fit_of(
        model: &str,
        params: &[&str],
        x: &[f64],
        y: &[f64],
        start: Option<&[f64]>,
    ) -> FitResult {
        let params: Vec<String> = params.iter().map(|s| (*s).to_string()).collect();
        let columns = vec![x.to_vec()];
        fit(&request(model, &params, &columns, y, start)).expect("fit")
    }

    #[test]
    fn oracle_exponential_decay_from_the_default_start() {
        // Java: params [4.999999999999999, 0.7000000000000001, 1.5000000000000002]
        //       r2 1.0, rmse 3.6259732146947156E-16, iters 9
        let x: [f64; 9] = [0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0];
        let y: Vec<f64> = x.iter().map(|x| 5.0 * (-0.7 * x).exp() + 1.5).collect();
        let r = fit_of("y = a * exp(-b * x) + c", &["a", "b", "c"], &x, &y, None);
        assert_eq!(names(&r.parameter_names), ["a", "b", "c"]);
        close(r.fitted_parameters[0], 4.999999999999999, 1e-9);
        close(r.fitted_parameters[1], 0.7000000000000001, 1e-9);
        close(r.fitted_parameters[2], 1.5000000000000002, 1e-9);
        close(r.r_squared, 1.0, 1e-12);
        assert!(r.rmse < 1e-12, "rmse {} should be ~0", r.rmse);
        assert_eq!(r.iterations, 9);
    }

    #[test]
    fn oracle_exponential_decay_with_scatter_and_a_custom_start() {
        // Java: params [5.007706988613813, 0.690465839847928, 1.4747194620489636]
        //       r2 0.9994820045657309, rmse 0.034237712312263545, iters 5
        let x: [f64; 9] = [0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0];
        let y: Vec<f64> = x
            .iter()
            .map(|x| 5.0 * (-0.7 * x).exp() + 1.5 + 0.05 * (9.0 * x).sin())
            .collect();
        let r = fit_of(
            "y = a * exp(-b * x) + c",
            &["a", "b", "c"],
            &x,
            &y,
            Some(&[4.0, 1.0, 1.0]),
        );
        close(r.fitted_parameters[0], 5.007706988613813, 1e-8);
        close(r.fitted_parameters[1], 0.690465839847928, 1e-8);
        close(r.fitted_parameters[2], 1.4747194620489636, 1e-8);
        close(r.r_squared, 0.9994820045657309, 1e-10);
        close(r.rmse, 0.034237712312263545, 1e-9);
        assert_eq!(r.iterations, 5);
        // Residuals are `observed - fitted`, in data order.
        close(r.residuals[0], 0.017573549337223326, 1e-7);
        close(r.residuals[8], -0.036614686745172875, 1e-7);
        close(r.fitted_values[0], 6.482426450662777, 1e-9);
    }

    #[test]
    fn oracle_linear_model() {
        // Java: params [3.0, -2.0], r2 1.0, rmse 0.0, iters 3
        let x: [f64; 5] = [1.0, 2.0, 3.0, 4.0, 5.0];
        let y: Vec<f64> = x.iter().map(|x| 3.0 * x - 2.0).collect();
        let r = fit_of("y = m * x + q", &["m", "q"], &x, &y, None);
        close(r.fitted_parameters[0], 3.0, 1e-10);
        close(r.fitted_parameters[1], -2.0, 1e-10);
        assert_eq!(r.iterations, 3);
        assert!(r.rmse < 1e-12);
    }

    #[test]
    fn oracle_power_law_from_a_bad_start() {
        // Java: params [2.499999999999999, 1.8000000000000003], r2 1.0, iters 7
        let x: [f64; 6] = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let y: Vec<f64> = x.iter().map(|x| 2.5 * x.powf(1.8)).collect();
        let r = fit_of("y = k * x^n", &["k", "n"], &x, &y, Some(&[1.0, 1.0]));
        close(r.fitted_parameters[0], 2.499999999999999, 1e-9);
        close(r.fitted_parameters[1], 1.8000000000000003, 1e-9);
        assert_eq!(r.iterations, 7);
    }

    #[test]
    fn oracle_overdetermined_quadratic_with_scatter() {
        // Java: params [0.5003377168298639, -1.2596151418030361, 3.0045971425232336]
        //       r2 0.999415980745421, rmse 0.06957503490214675, iters 2
        let x: [f64; 8] = [-3.0, -2.0, -1.0, 0.0, 1.0, 2.0, 3.0, 4.0];
        let y: Vec<f64> = x
            .iter()
            .map(|x| 0.5 * x * x - 1.25 * x + 3.0 + 0.1 * (4.0 * x).cos())
            .collect();
        let r = fit_of(
            "y = a2 * x^2 + a1 * x + a0",
            &["a2", "a1", "a0"],
            &x,
            &y,
            None,
        );
        close(r.fitted_parameters[0], 0.5003377168298639, 1e-9);
        close(r.fitted_parameters[1], -1.2596151418030361, 1e-9);
        close(r.fitted_parameters[2], 3.0045971425232336, 1e-9);
        close(r.r_squared, 0.999415980745421, 1e-10);
        close(r.rmse, 0.06957503490214675, 1e-9);
        assert_eq!(r.iterations, 2);
    }

    #[test]
    fn oracle_single_parameter() {
        // Java: params [7.0], r2 1.0, rmse 0.0, iters 3
        let x: [f64; 3] = [1.0, 2.0, 3.0];
        let y: [f64; 3] = [7.0, 14.0, 21.0];
        let r = fit_of("y = a * x", &["a"], &x, &y, None);
        close(r.fitted_parameters[0], 7.0, 1e-10);
        assert_eq!(r.iterations, 3);
        assert!(r.rmse < 1e-12);
    }

    // -- the validation half ---------------------------------------------

    #[test]
    fn a_blank_model_is_refused() {
        let params = ["a".to_string()];
        let columns = vec![vec![1.0]];
        let err = fit(&request("  ", &params, &columns, &[1.0], None)).unwrap_err();
        assert_eq!(err.to_string_message(), "Model equation is required.");
    }

    #[test]
    fn mismatched_data_lengths_are_refused() {
        let params = ["a".to_string()];
        let columns = vec![vec![1.0, 2.0]];
        let err = fit(&request("y = a * x", &params, &columns, &[1.0], None)).unwrap_err();
        assert_eq!(
            err.to_string_message(),
            "x and y data must have the same length (got 2 and 1) for 'x'."
        );
    }

    #[test]
    fn empty_data_and_empty_parameters_are_refused() {
        let params = ["a".to_string()];
        let empty = vec![Vec::new()];
        let err = fit(&request("y = a * x", &params, &empty, &[], None)).unwrap_err();
        assert_eq!(err.to_string_message(), "Data points are required.");
        let columns = vec![vec![1.0]];
        let err = fit(&request("y = a * x", &[], &columns, &[1.0], None)).unwrap_err();
        assert_eq!(
            err.to_string_message(),
            "At least one parameter to fit is required."
        );
    }

    #[test]
    fn a_model_that_does_not_isolate_the_dependent_variable_is_refused() {
        let params = ["a".to_string()];
        let columns = vec![vec![1.0]];
        let err = fit(&request("y + z = a * x", &params, &columns, &[1.0], None)).unwrap_err();
        assert!(
            err.to_string_message()
                .starts_with("Could not identify 'y' as the dependent variable"),
            "{}",
            err.to_string_message()
        );
    }

    #[test]
    fn the_model_may_be_written_with_the_dependent_variable_on_the_right() {
        // `parseModelExpression` accepts `<expr> = y` too.
        let x: [f64; 3] = [1.0, 2.0, 3.0];
        let y: [f64; 3] = [7.0, 14.0, 21.0];
        let r = fit_of("a * x = y", &["a"], &x, &y, None);
        close(r.fitted_parameters[0], 7.0, 1e-10);
    }

    #[test]
    fn a_short_initial_guess_defaults_the_rest_to_one() {
        // Only `a` is seeded; `b` and `c` start at 1.0 like the Java's
        // per-index `i < initialGuess.size()` test.
        let x: [f64; 9] = [0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0];
        let y: Vec<f64> = x.iter().map(|x| 5.0 * (-0.7 * x).exp() + 1.5).collect();
        let seeded = fit_of(
            "y = a * exp(-b * x) + c",
            &["a", "b", "c"],
            &x,
            &y,
            Some(&[1.0]),
        );
        let defaulted = fit_of("y = a * exp(-b * x) + c", &["a", "b", "c"], &x, &y, None);
        assert_eq!(seeded.iterations, defaulted.iterations);
        close(
            seeded.fitted_parameters[1],
            defaulted.fitted_parameters[1],
            1e-12,
        );
    }

    #[test]
    fn parameter_names_are_reported_lowercase() {
        let x: [f64; 3] = [1.0, 2.0, 3.0];
        let y: [f64; 3] = [7.0, 14.0, 21.0];
        let params = ["Slope".to_string()];
        let x_vars = ["X".to_string()];
        let columns = vec![x.to_vec()];
        let r = fit(&CurveFitRequest {
            model: "y = Slope * x",
            y_variable: "Y",
            x_variables: &x_vars,
            parameters: &params,
            x_data: &columns,
            y_data: &y,
            ..Default::default()
        })
        .expect("fit");
        assert_eq!(names(&r.parameter_names), ["slope"]);
        close(r.fitted_parameters[0], 7.0, 1e-10);
    }

    #[test]
    fn r_squared_is_one_when_the_observations_have_no_spread() {
        // ssTot == 0 → the Java returns 1.0 rather than dividing by zero.
        let x: [f64; 3] = [1.0, 2.0, 3.0];
        let y: [f64; 3] = [4.0, 4.0, 4.0];
        let r = fit_of("y = c + 0 * x", &["c"], &x, &y, None);
        assert_eq!(r.r_squared, 1.0);
    }

    // -- Phase 4.2: weighting, uncertainty and multiple predictors ----------

    /// The shared `y = a·x + b` case. Every closed-form expectation below is
    /// weighted least squares worked through by hand on this data, not a
    /// previous run of this code.
    const LINE_X: [f64; 5] = [0.0, 1.0, 2.0, 3.0, 4.0];
    const LINE_Y: [f64; 5] = [1.1, 2.9, 5.2, 6.8, 9.1];

    fn line_fit(sigma: Option<&[f64]>) -> FitResult {
        let params = ["a".to_string(), "b".to_string()];
        let x_vars = ["x".to_string()];
        let columns = vec![LINE_X.to_vec()];
        fit(&CurveFitRequest {
            model: "y = a * x + b",
            y_variable: "y",
            x_variables: &x_vars,
            parameters: &params,
            x_data: &columns,
            y_data: &LINE_Y,
            sigma,
            ..Default::default()
        })
        .expect("fit")
    }

    #[test]
    fn unweighted_standard_errors_match_the_ordinary_least_squares_formula() {
        // OLS on LINE_X/LINE_Y: a = 1.99, b = 1.04, SSres = 0.107,
        // sigmahat^2 = SSres/(n-2) = 0.03566666..., and the textbook
        // SE(a) = sigmahat/sqrt(Sxx), SE(b) = sigmahat*sqrt(Sxx_bar/(n*Sxx)).
        let r = line_fit(None);
        // 1e-7, not 1e-9: the Jacobian is central finite differences with
        // FD_STEP = 1e-8, so the optimum carries that much truncation error.
        close(r.fitted_parameters[0], 1.99, 1e-7);
        close(r.fitted_parameters[1], 1.04, 1e-7);
        assert_eq!(r.residual_dof, 3);
        assert_eq!(r.rank, 2);
        assert!(!r.unidentifiable);
        close(r.parameter_std_errors[0], 0.0597215762238965, 1e-9);
        close(r.parameter_std_errors[1], 0.146287388383278, 1e-9);
        // Covariance is symmetric and its diagonal is the squared errors.
        close(
            r.parameter_covariance[0][0],
            r.parameter_std_errors[0].powi(2),
            1e-12,
        );
        close(
            r.parameter_covariance[0][1],
            r.parameter_covariance[1][0],
            1e-12,
        );
        // No sigma means no absolute scale to test the fit against.
        assert!(r.reduced_chi_square.is_nan());
    }

    #[test]
    fn a_believed_sigma_gives_absolute_standard_errors_and_a_reduced_chi_square() {
        // Uniform sigma leaves the WLS optimum where OLS put it but replaces
        // the estimated residual scale with the quoted one: SE(a) = 0.0316228
        // = 0.1/sqrt(Sxx), which is *smaller* than the unweighted 0.0597
        // because the data scatters more than sigma = 0.1 claims. That is what
        // reduced chi-square is for, and here it is 5.35, not 1.
        let sigma = [0.1; 5];
        let r = line_fit(Some(&sigma));
        close(r.fitted_parameters[0], 1.99, 1e-7);
        close(r.fitted_parameters[1], 1.04, 1e-7);
        close(r.parameter_std_errors[0], 0.1 / (10.0f64).sqrt(), 1e-9);
        close(r.reduced_chi_square, 0.107 / 0.01 / 3.0, 1e-7);
    }

    #[test]
    fn a_down_weighted_point_moves_the_optimum() {
        // sigma = [0.1, 0.1, 0.1, 0.1, 5.0]: the last point is 2500x less
        // trusted, and the closed-form WLS answer swings to a = 1.94005,
        // b = 1.08995 with absolute SE(a) = 0.04471018, SE(b) = 0.08366003.
        let sigma = [0.1, 0.1, 0.1, 0.1, 5.0];
        let r = line_fit(Some(&sigma));
        close(r.fitted_parameters[0], 1.940_049_970_017_99, 1e-8);
        close(r.fitted_parameters[1], 1.089_950_029_982_01, 1e-8);
        close(r.parameter_std_errors[0], 0.044_710_184_518_073_9, 1e-7);
        close(r.parameter_std_errors[1], 0.083_660_029_880_703_6, 1e-7);
        close(r.reduced_chi_square, 2.734_166_166_966_49, 1e-7);
        // The residuals reported stay raw, not divided by sigma.
        close(r.residuals[0], LINE_Y[0] - 1.089_950_029_982_01, 1e-7);
    }

    #[test]
    fn an_unidentifiable_fit_reports_no_standard_errors_rather_than_a_lie() {
        // `a + b` has two identical Jacobian columns: the sum is determined,
        // neither parameter is. A finite standard error here would be false.
        let x: Vec<f64> = (0..10).map(f64::from).collect();
        let y = vec![5.0; 10];
        let params = ["a".to_string(), "b".to_string()];
        let x_vars = ["x".to_string()];
        let columns = vec![x];
        let r = fit(&CurveFitRequest {
            model: "y = a + b",
            y_variable: "y",
            x_variables: &x_vars,
            parameters: &params,
            x_data: &columns,
            y_data: &y,
            ..Default::default()
        })
        .expect("fit");
        assert!(r.unidentifiable, "rank {} of 2", r.rank);
        assert!(r.rank < 2);
        assert!(r.parameter_std_errors.iter().all(|e| e.is_nan()));
        assert!(r.parameter_covariance.is_empty());
        assert!(r.condition_number > 1e12, "{}", r.condition_number);
    }

    #[test]
    fn a_fit_with_no_residual_degrees_of_freedom_reports_no_standard_errors() {
        // Two points, two parameters: the line goes exactly through them and
        // there is nothing left over to estimate a spread from.
        let params = ["a".to_string(), "b".to_string()];
        let x_vars = ["x".to_string()];
        let columns = vec![vec![0.0, 1.0]];
        let r = fit(&CurveFitRequest {
            model: "y = a * x + b",
            y_variable: "y",
            x_variables: &x_vars,
            parameters: &params,
            x_data: &columns,
            y_data: &[1.0, 3.0],
            ..Default::default()
        })
        .expect("fit");
        assert_eq!(r.residual_dof, 0);
        assert!(r.parameter_std_errors.iter().all(|e| e.is_nan()));
        assert!(r.parameter_covariance.is_empty());
    }

    #[test]
    fn two_predictor_columns_recover_both_coefficients() {
        // y = 2*x1 - 3*x2 exactly, so the fit must land on (2, -3).
        let params = ["a".to_string(), "b".to_string()];
        let x_vars = ["x1".to_string(), "x2".to_string()];
        let columns = vec![vec![1.0, 2.0, 3.0, 4.0, 5.0], vec![1.0, 0.0, 2.0, 1.0, 3.0]];
        let r = fit(&CurveFitRequest {
            model: "y = a * x1 + b * x2",
            y_variable: "y",
            x_variables: &x_vars,
            parameters: &params,
            x_data: &columns,
            y_data: &[-1.0, 4.0, 0.0, 5.0, 1.0],
            ..Default::default()
        })
        .expect("fit");
        close(r.fitted_parameters[0], 2.0, 1e-7);
        close(r.fitted_parameters[1], -3.0, 1e-7);
        close(r.r_squared, 1.0, 1e-9);
        assert_eq!(r.rank, 2);
    }

    #[test]
    fn a_malformed_sigma_is_refused() {
        let params = ["a".to_string()];
        let columns = vec![LINE_X.to_vec()];
        let with_sigma = |sigma: &[f64]| {
            fit(&CurveFitRequest {
                model: "y = a * x",
                y_variable: "y",
                x_variables: X_ONLY.get_or_init(|| vec!["x".to_string()]),
                parameters: &params,
                x_data: &columns,
                y_data: &LINE_Y,
                sigma: Some(sigma),
                ..Default::default()
            })
            .unwrap_err()
            .to_string_message()
        };
        assert!(
            with_sigma(&[0.1; 3])
                .starts_with("Measurement standard deviations must have one entry"),
            "{}",
            with_sigma(&[0.1; 3])
        );
        assert!(with_sigma(&[0.1, 0.1, 0.0, 0.1, 0.1]).contains("finite and positive"));
        assert!(with_sigma(&[0.1, 0.1, -1.0, 0.1, 0.1]).contains("finite and positive"));
        assert!(with_sigma(&[0.1, 0.1, f64::NAN, 0.1, 0.1]).contains("finite and positive"));
    }

    #[test]
    fn a_column_count_that_does_not_match_the_variable_count_is_refused() {
        let params = ["a".to_string()];
        let x_vars = ["x1".to_string(), "x2".to_string()];
        let columns = vec![LINE_X.to_vec()];
        let err = fit(&CurveFitRequest {
            model: "y = a * x1",
            y_variable: "y",
            x_variables: &x_vars,
            parameters: &params,
            x_data: &columns,
            y_data: &LINE_Y,
            ..Default::default()
        })
        .unwrap_err();
        assert!(
            err.to_string_message()
                .starts_with("Expected one data column per independent variable"),
            "{}",
            err.to_string_message()
        );
    }

    // -- Phase 4.2: bounds and robust losses --------------------------------

    fn bounded_line_fit(lower: &[f64], upper: &[f64], start: Option<&[f64]>) -> Result<FitResult> {
        let params = ["a".to_string()];
        let x_vars = ["x".to_string()];
        let columns = vec![LINE_X.to_vec()];
        // OLS through the origin on this data wants a = 2.0257...
        fit(&CurveFitRequest {
            model: "y = a * x",
            y_variable: "y",
            x_variables: &x_vars,
            parameters: &params,
            x_data: &columns,
            y_data: &LINE_Y,
            initial_guess: start,
            lower: Some(lower),
            upper: Some(upper),
            ..Default::default()
        })
    }

    #[test]
    fn bounds_that_do_not_bind_leave_the_optimum_where_it_was() {
        let free = bounded_line_fit(&[-100.0], &[100.0], None).expect("fit");
        let params = ["a".to_string()];
        let columns = vec![LINE_X.to_vec()];
        let unbounded = fit(&request("y = a * x", &params, &columns, &LINE_Y, None)).expect("fit");
        close(
            free.fitted_parameters[0],
            unbounded.fitted_parameters[0],
            1e-9,
        );
        assert_eq!(free.at_bound, [false]);
    }

    #[test]
    fn an_active_upper_bound_holds_and_is_reported() {
        // The unconstrained optimum is above 1.5, so the bound must bind.
        let r = bounded_line_fit(&[0.0], &[1.5], Some(&[1.0])).expect("fit");
        assert!(
            r.fitted_parameters[0] <= 1.5 + 1e-12,
            "{}",
            r.fitted_parameters[0]
        );
        close(r.fitted_parameters[0], 1.5, 1e-9);
        assert_eq!(r.at_bound, [true]);
    }

    #[test]
    fn malformed_bounds_are_refused_in_the_calibration_workflow_words() {
        let err = bounded_line_fit(&[0.0, 0.0], &[1.0], None).unwrap_err();
        assert!(
            err.to_string_message()
                .starts_with("Parameter bounds need one"),
            "{}",
            err.to_string_message()
        );
        let err = bounded_line_fit(&[2.0], &[1.0], None).unwrap_err();
        assert_eq!(
            err.to_string_message(),
            "Bounds for a must be finite with lower < upper."
        );
        let err = bounded_line_fit(&[0.0], &[f64::INFINITY], None).unwrap_err();
        assert_eq!(
            err.to_string_message(),
            "Bounds for a must be finite with lower < upper."
        );
        // The default start is 1.0, which is outside [5, 10].
        let err = bounded_line_fit(&[5.0], &[10.0], None).unwrap_err();
        assert_eq!(
            err.to_string_message(),
            "The initial value of a lies outside its bounds."
        );
    }

    /// `y = 2x + 1` with one gross outlier at x = 5. Least squares is dragged
    /// to a = 2.539394, b = 7.472727; a robust loss should not be.
    fn outlier_fit(loss: Loss) -> FitResult {
        let x: Vec<f64> = (0..10).map(f64::from).collect();
        let mut y: Vec<f64> = x.iter().map(|x| 2.0 * x + 1.0).collect();
        y[5] = 100.0;
        let params = ["a".to_string(), "b".to_string()];
        let x_vars = ["x".to_string()];
        let columns = vec![x];
        fit(&CurveFitRequest {
            model: "y = a * x + b",
            y_variable: "y",
            x_variables: &x_vars,
            parameters: &params,
            x_data: &columns,
            y_data: &y,
            loss,
            ..Default::default()
        })
        .expect("fit")
    }

    #[test]
    fn least_squares_is_dragged_by_a_gross_outlier() {
        let r = outlier_fit(Loss::Linear);
        close(r.fitted_parameters[0], 2.539_393_939_393_94, 1e-6);
        close(r.fitted_parameters[1], 7.472_727_272_727_27, 1e-6);
    }

    #[test]
    fn a_robust_loss_recovers_the_line_the_outlier_hid() {
        for loss in [Loss::SoftL1, Loss::Huber, Loss::Cauchy] {
            let r = outlier_fit(loss);
            assert!(
                (r.fitted_parameters[0] - 2.0).abs() < 0.05,
                "{loss:?} slope {}",
                r.fitted_parameters[0]
            );
            assert!(
                (r.fitted_parameters[1] - 1.0).abs() < 0.3,
                "{loss:?} intercept {}",
                r.fitted_parameters[1]
            );
            // The reported residuals stay raw, so the outlier is still visible.
            assert!(r.residuals[5] > 80.0, "{loss:?} {}", r.residuals[5]);
        }
        // Cauchy is the most aggressive and should land nearest of the three.
        let cauchy = outlier_fit(Loss::Cauchy);
        close(cauchy.fitted_parameters[0], 2.0, 1e-3);
        close(cauchy.fitted_parameters[1], 1.0, 1e-2);
    }

    #[test]
    fn a_fixed_f_scale_overrides_the_estimated_one() {
        // A scale far larger than any residual makes every weight ~1, so the
        // robust fit collapses back onto least squares.
        let x: Vec<f64> = (0..10).map(f64::from).collect();
        let mut y: Vec<f64> = x.iter().map(|x| 2.0 * x + 1.0).collect();
        y[5] = 100.0;
        let params = ["a".to_string(), "b".to_string()];
        let x_vars = ["x".to_string()];
        let columns = vec![x];
        let r = fit(&CurveFitRequest {
            model: "y = a * x + b",
            y_variable: "y",
            x_variables: &x_vars,
            parameters: &params,
            x_data: &columns,
            y_data: &y,
            loss: Loss::Cauchy,
            f_scale: Some(1e6),
            ..Default::default()
        })
        .expect("fit");
        close(r.fitted_parameters[0], 2.539_393_939_393_94, 1e-4);
    }

    // -- Phase 4.2: confidence and prediction bands -------------------------

    #[test]
    fn bands_match_the_textbook_simple_regression_formulas() {
        // For OLS `y = a·x + b`, se_fit(x) = sigmahat·sqrt(1/n + (x-xbar)²/Sxx)
        // and se_pred adds a 1 under the root. On LINE_X/LINE_Y that is
        // sigmahat = 0.188856206, and t(0.975, 3) = 3.1824463052837064 from
        // the closed-form df = 3 CDF, F(t) = ½ + (1/π)[(t/√3)/(1+t²/3) +
        // atan(t/√3)], solved by bisection — not from a quadrature of the
        // density, which is what got these numbers wrong the first time.
        let r = line_fit(None);
        assert_eq!(r.confidence, 0.95);
        close(r.confidence_band_lo[0], 0.574_448_241_330, 1e-7);
        close(r.confidence_band_hi[0], 1.505_551_758_670, 1e-7);
        close(r.prediction_band_lo[0], 0.279_757_161_602, 1e-7);
        close(r.prediction_band_hi[0], 1.800_242_838_398, 1e-7);
        // The narrowest point of both bands is the centroid of the predictors.
        close(r.confidence_band_lo[2], 4.751_213_566_810, 1e-7);
        close(r.confidence_band_hi[2], 5.288_786_433_190, 1e-7);
        // Prediction is wider than confidence everywhere, and both bracket the
        // fitted curve.
        for i in 0..5 {
            assert!(
                r.prediction_band_lo[i] < r.confidence_band_lo[i],
                "point {i}"
            );
            assert!(
                r.prediction_band_hi[i] > r.confidence_band_hi[i],
                "point {i}"
            );
            assert!(r.confidence_band_lo[i] < r.fitted_values[i], "point {i}");
            assert!(r.confidence_band_hi[i] > r.fitted_values[i], "point {i}");
        }
    }

    #[test]
    fn a_tighter_confidence_level_gives_a_narrower_band() {
        let params = ["a".to_string(), "b".to_string()];
        let x_vars = ["x".to_string()];
        let columns = vec![LINE_X.to_vec()];
        let at = |confidence: f64| {
            fit(&CurveFitRequest {
                model: "y = a * x + b",
                y_variable: "y",
                x_variables: &x_vars,
                parameters: &params,
                x_data: &columns,
                y_data: &LINE_Y,
                confidence,
                ..Default::default()
            })
            .expect("fit")
        };
        let wide = at(0.99);
        let narrow = at(0.80);
        assert!(narrow.confidence_band_hi[0] < wide.confidence_band_hi[0]);
        assert!(narrow.confidence_band_lo[0] > wide.confidence_band_lo[0]);
        // Out of range falls back to 0.95 rather than producing nonsense.
        assert_eq!(at(0.0).confidence, 0.95);
        assert_eq!(at(1.5).confidence, 0.95);
    }

    #[test]
    fn a_weighted_prediction_band_carries_the_points_own_sigma() {
        // The last point is quoted at sigma = 5, the rest at 0.1, so its
        // prediction band must be far wider than theirs.
        let sigma = [0.1, 0.1, 0.1, 0.1, 5.0];
        let r = line_fit(Some(&sigma));
        let width = |i: usize| r.prediction_band_hi[i] - r.prediction_band_lo[i];
        assert!(width(4) > 10.0 * width(0), "{} vs {}", width(4), width(0));
        // The confidence band on the curve itself is not so lopsided.
        let curve = |i: usize| r.confidence_band_hi[i] - r.confidence_band_lo[i];
        assert!(curve(4) < width(4) / 2.0);
    }

    #[test]
    fn an_unidentifiable_fit_reports_no_bands() {
        let x: Vec<f64> = (0..10).map(f64::from).collect();
        let y = vec![5.0; 10];
        let params = ["a".to_string(), "b".to_string()];
        let x_vars = ["x".to_string()];
        let columns = vec![x];
        let r = fit(&CurveFitRequest {
            model: "y = a + b",
            y_variable: "y",
            x_variables: &x_vars,
            parameters: &params,
            x_data: &columns,
            y_data: &y,
            ..Default::default()
        })
        .expect("fit");
        assert!(r.confidence_band_lo.iter().all(|v| v.is_nan()));
        assert!(r.prediction_band_hi.iter().all(|v| v.is_nan()));
    }
}
