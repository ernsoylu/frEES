//! The integration schemes: explicit Runge–Kutta (fixed and embedded-adaptive),
//! Shampine's modified Rosenbrock `ode23s`, the step-doubling BDF `ode15s`, and
//! the finite-difference Jacobian / dense linear solve they share.
//!
//! Port of
//! `../frEES/backend/core/src/main/java/com/frees/backend/core/ode/OdeMethod.java`,
//! `ButcherTableau.java`, `RungeKuttaMethod.java`, `RosenbrockMethod.java`,
//! `BdfMethod.java` and `OdeLinearAlgebra.java`.
//!
//! # The tableaux are data
//!
//! Every coefficient in [`ButcherTableau`] is transcribed character for
//! character from the Java. None of them is derived, simplified, or re-expressed
//! (`19372.0 / 6561` stays a division of two integer literals; folding it to a
//! decimal would change the last bits and drift from the oracle).
//!
//! # Two step-size controllers, deliberately not unified
//!
//! `RungeKuttaMethod` caps an accepted step with `capStep` (`h > maxStep ?
//! maxStep : h`) while `BdfMethod` and `RosenbrockMethod` use `Math.min`. They
//! agree for every finite input; the difference is kept because each is
//! transcribed from its own Java method. `MAX_SCALE` likewise differs — 5.0 for
//! the explicit pair and Rosenbrock, 4.0 for the BDF — and is *not* hoisted into
//! a shared constant.
//!
//! # `Math.min` / `Math.max` are not Rust's
//!
//! Java's `Math.min`/`Math.max` propagate NaN; Rust's `f64::min`/`f64::max`
//! return the non-NaN operand. A diverged stage makes `errNorm` NaN, and the
//! controllers feed that straight into `Math.max(MIN_SCALE, …)`, so the
//! difference is reachable. [`java_min`] / [`java_max`] restore the Java
//! semantics.

// Numerical kernels index several parallel arrays (and 2-D `a[i][j]` slices) by
// the same loop variable, mirroring the Java being transcribed. Iterator
// rewrites obscure that correspondence, so the indexed form stays — the same
// call made in `crate::linalg`.
#![allow(clippy::needless_range_loop)]

use crate::diag::{FreesError, Result};
use crate::ode::problem::{OdeProblem, OdeRhs};

// ---------------------------------------------------------------------------
// Java float helpers
// ---------------------------------------------------------------------------

/// `Math.min` (NaN-propagating, `-0.0 < 0.0`).
pub(crate) fn java_min(a: f64, b: f64) -> f64 {
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
pub(crate) fn java_max(a: f64, b: f64) -> f64 {
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
// OdeMethod
// ---------------------------------------------------------------------------

/// Outcome of one attempted step. On rejection `y_new`/`f_new` are `None`
/// (the Java `null`) and `h_next` carries the reduced step to retry with.
///
/// Port of `OdeMethod.StepResult`.
#[derive(Debug, Clone, PartialEq)]
pub struct StepResult {
    pub accepted: bool,
    pub y_new: Option<Vec<f64>>,
    pub f_new: Option<Vec<f64>>,
    pub h_next: f64,
}

impl StepResult {
    fn accept(y_new: Vec<f64>, f_new: Vec<f64>, h_next: f64) -> StepResult {
        StepResult {
            accepted: true,
            y_new: Some(y_new),
            f_new: Some(f_new),
            h_next,
        }
    }

    fn reject(h_next: f64) -> StepResult {
        StepResult {
            accepted: false,
            y_new: None,
            f_new: None,
            h_next,
        }
    }
}

/// One integration scheme.
///
/// Port of `OdeMethod.java`. The generic driver in [`crate::ode::integrator`]
/// owns the time loop, event detection and output sampling; a method only knows
/// how to attempt a single step.
pub trait OdeMethod {
    fn name(&self) -> &str;

    fn adaptive(&self) -> bool;

    /// The order of the method's primary solution (used for initial step
    /// sizing).
    fn order(&self) -> u32;

    /// Attempt one step of size `h` from `(t, y)` given the already known
    /// derivative `f0 = f(t, y)`.
    fn step(
        &self,
        f: &dyn OdeRhs,
        t: f64,
        y: &[f64],
        f0: &[f64],
        h: f64,
        problem: &OdeProblem<'_>,
    ) -> Result<StepResult>;
}

// ---------------------------------------------------------------------------
// Butcher tableaux
// ---------------------------------------------------------------------------

/// A Butcher tableau for an explicit Runge–Kutta method.
///
/// Port of `ButcherTableau.java`. `a` is the strict lower-triangular
/// stage-coefficient matrix (`a[i]` has length `i`), `c` the stage nodes, `b`
/// the high-order solution weights. When `b_err` (= `b − b_hat`) is `Some` the
/// method is an embedded pair usable for adaptive step-size control, with
/// `error_order` the lower order of the pair (the exponent in the step-size
/// controller). `fsal` marks a First-Same-As-Last pair (last stage equals the
/// next step's first derivative).
#[derive(Debug, Clone, PartialEq)]
pub struct ButcherTableau {
    pub name: &'static str,
    pub c: Vec<f64>,
    pub a: Vec<Vec<f64>>,
    pub b: Vec<f64>,
    pub b_err: Option<Vec<f64>>,
    pub error_order: u32,
    pub fsal: bool,
    pub stages: usize,
}

impl ButcherTableau {
    fn new(
        name: &'static str,
        c: Vec<f64>,
        a: Vec<Vec<f64>>,
        b: Vec<f64>,
        b_err: Option<Vec<f64>>,
        error_order: u32,
        fsal: bool,
    ) -> ButcherTableau {
        let stages = b.len();
        ButcherTableau {
            name,
            c,
            a,
            b,
            b_err,
            error_order,
            fsal,
            stages,
        }
    }

    pub fn adaptive(&self) -> bool {
        self.b_err.is_some()
    }

    // ── Fixed-step explicit methods (orders ode1–ode5) ────────────────────

    /// `ode1` — explicit (forward) Euler, order 1.
    pub fn euler() -> ButcherTableau {
        ButcherTableau::new("ode1", vec![0.0], vec![vec![]], vec![1.0], None, 1, false)
    }

    /// `ode2` — Heun's method (explicit trapezoid), order 2.
    pub fn heun() -> ButcherTableau {
        ButcherTableau::new(
            "ode2",
            vec![0.0, 1.0],
            vec![vec![], vec![1.0]],
            vec![0.5, 0.5],
            None,
            2,
            false,
        )
    }

    /// `ode3` — Kutta's third-order method.
    pub fn rk3() -> ButcherTableau {
        ButcherTableau::new(
            "ode3",
            vec![0.0, 0.5, 1.0],
            vec![vec![], vec![0.5], vec![-1.0, 2.0]],
            vec![1.0 / 6.0, 2.0 / 3.0, 1.0 / 6.0],
            None,
            3,
            false,
        )
    }

    /// `ode4` — the classic fourth-order Runge–Kutta method.
    pub fn rk4() -> ButcherTableau {
        ButcherTableau::new(
            "ode4",
            vec![0.0, 0.5, 0.5, 1.0],
            vec![vec![], vec![0.5], vec![0.0, 0.5], vec![0.0, 0.0, 1.0]],
            vec![1.0 / 6.0, 1.0 / 3.0, 1.0 / 3.0, 1.0 / 6.0],
            None,
            4,
            false,
        )
    }

    /// `ode5` — Dormand–Prince fifth-order weights used as a fixed-step method.
    pub fn dopri5_fixed() -> ButcherTableau {
        let pair = ButcherTableau::dopri54();
        ButcherTableau::new("ode5", pair.c, pair.a, pair.b, None, 5, false)
    }

    // ── Adaptive embedded pairs ─────────────────────────────────────────────

    /// `ode45` — Dormand–Prince 5(4), the default adaptive method (FSAL).
    pub fn dopri54() -> ButcherTableau {
        let c = vec![0.0, 1.0 / 5.0, 3.0 / 10.0, 4.0 / 5.0, 8.0 / 9.0, 1.0, 1.0];
        let a = vec![
            vec![],
            vec![1.0 / 5.0],
            vec![3.0 / 40.0, 9.0 / 40.0],
            vec![44.0 / 45.0, -56.0 / 15.0, 32.0 / 9.0],
            vec![
                19372.0 / 6561.0,
                -25360.0 / 2187.0,
                64448.0 / 6561.0,
                -212.0 / 729.0,
            ],
            vec![
                9017.0 / 3168.0,
                -355.0 / 33.0,
                46732.0 / 5247.0,
                49.0 / 176.0,
                -5103.0 / 18656.0,
            ],
            vec![
                35.0 / 384.0,
                0.0,
                500.0 / 1113.0,
                125.0 / 192.0,
                -2187.0 / 6784.0,
                11.0 / 84.0,
            ],
        ];
        let b = vec![
            35.0 / 384.0,
            0.0,
            500.0 / 1113.0,
            125.0 / 192.0,
            -2187.0 / 6784.0,
            11.0 / 84.0,
            0.0,
        ];
        let b_hat = [
            5179.0 / 57600.0,
            0.0,
            7571.0 / 16695.0,
            393.0 / 640.0,
            -92097.0 / 339200.0,
            187.0 / 2100.0,
            1.0 / 40.0,
        ];
        let b_err: Vec<f64> = (0..b.len()).map(|i| b[i] - b_hat[i]).collect();
        ButcherTableau::new("ode45", c, a, b, Some(b_err), 4, true)
    }

    /// `ode23` — Bogacki–Shampine 3(2) adaptive pair (FSAL).
    pub fn bogacki_shampine32() -> ButcherTableau {
        let c = vec![0.0, 1.0 / 2.0, 3.0 / 4.0, 1.0];
        let a = vec![
            vec![],
            vec![1.0 / 2.0],
            vec![0.0, 3.0 / 4.0],
            vec![2.0 / 9.0, 1.0 / 3.0, 4.0 / 9.0],
        ];
        let b = vec![2.0 / 9.0, 1.0 / 3.0, 4.0 / 9.0, 0.0];
        let b_hat = [7.0 / 24.0, 1.0 / 4.0, 1.0 / 3.0, 1.0 / 8.0];
        let b_err: Vec<f64> = (0..b.len()).map(|i| b[i] - b_hat[i]).collect();
        ButcherTableau::new("ode23", c, a, b, Some(b_err), 2, true)
    }
}

// ---------------------------------------------------------------------------
// Explicit Runge–Kutta
// ---------------------------------------------------------------------------

/// `RungeKuttaMethod.SAFETY`.
const RK_SAFETY: f64 = 0.9;
/// `RungeKuttaMethod.MIN_SCALE`.
const RK_MIN_SCALE: f64 = 0.2;
/// `RungeKuttaMethod.MAX_SCALE`.
const RK_MAX_SCALE: f64 = 5.0;

/// Explicit Runge–Kutta stepper driven by a [`ButcherTableau`].
///
/// Port of `RungeKuttaMethod.java`. Handles both fixed-step methods (no
/// embedded error estimate) and adaptive embedded pairs with a PI-style
/// step-size controller (rtol/atol). The driver in [`crate::ode::integrator`]
/// owns the time loop and dense-output sampling.
#[derive(Debug, Clone, PartialEq)]
pub struct RungeKuttaMethod {
    t: ButcherTableau,
}

impl RungeKuttaMethod {
    pub fn new(tableau: ButcherTableau) -> RungeKuttaMethod {
        RungeKuttaMethod { t: tableau }
    }

    /// The tableau this stepper was built from.
    pub fn tableau(&self) -> &ButcherTableau {
        &self.t
    }
}

impl OdeMethod for RungeKuttaMethod {
    fn name(&self) -> &str {
        self.t.name
    }

    fn adaptive(&self) -> bool {
        self.t.adaptive()
    }

    fn order(&self) -> u32 {
        if self.t.adaptive() {
            self.t.error_order + 1
        } else {
            self.t.error_order
        }
    }

    fn step(
        &self,
        f: &dyn OdeRhs,
        time: f64,
        y: &[f64],
        f0: &[f64],
        h: f64,
        problem: &OdeProblem<'_>,
    ) -> Result<StepResult> {
        let t = &self.t;
        let n = y.len();
        let s = t.stages;
        let mut k: Vec<Vec<f64>> = Vec::with_capacity(s);
        k.push(f0.to_vec());
        for i in 1..s {
            let yi = stage_state(y, &t.a[i], &k, h, n);
            k.push(f.eval(time + t.c[i] * h, &yi)?);
        }

        let mut y_new = y.to_vec();
        accumulate_weighted(&mut y_new, &t.b, &k, h, s, n);

        let Some(b_err) = t.b_err.as_ref() else {
            let f_new = f.eval(time + h, &y_new)?;
            return Ok(StepResult::accept(y_new, f_new, h));
        };

        // Embedded error estimate.
        let mut err_vec = vec![0.0; n];
        accumulate_weighted(&mut err_vec, b_err, &k, h, s, n);
        let err = error_norm(&err_vec, y, &y_new, problem.rtol, problem.atol);
        let exponent = 1.0 / f64::from(t.error_order + 1);
        if !err.is_finite() || !all_finite(&y_new) {
            // A stage diverged (NaN/Inf) — reject hard and shrink the step.
            return Ok(StepResult::reject(h * RK_MIN_SCALE));
        }
        if err <= 1.0 {
            let f_new = f.eval(time + h, &y_new)?;
            let scale = if err == 0.0 {
                RK_MAX_SCALE
            } else {
                java_min(
                    RK_MAX_SCALE,
                    java_max(RK_MIN_SCALE, RK_SAFETY * libm::pow(err, -exponent)),
                )
            };
            return Ok(StepResult::accept(
                y_new,
                f_new,
                cap_step(h * scale, problem),
            ));
        }
        let scale = java_max(RK_MIN_SCALE, RK_SAFETY * libm::pow(err, -exponent));
        Ok(StepResult::reject(h * scale))
    }
}

/// Intermediate stage state `yi = y + h·Σ_j a[j]·k[j]` (zero coefficients
/// skipped, exactly as the Java does — the skip is what keeps a fixed-step
/// tableau's structural zeros out of the sum).
fn stage_state(y: &[f64], ai: &[f64], k: &[Vec<f64>], h: f64, n: usize) -> Vec<f64> {
    let mut yi = y.to_vec();
    for j in 0..ai.len() {
        let aij = ai[j];
        if aij == 0.0 {
            continue;
        }
        for d in 0..n {
            yi[d] += h * aij * k[j][d];
        }
    }
    yi
}

/// Adds `h·Σ_i coeff[i]·k[i]` into `out` (zero coefficients skipped).
fn accumulate_weighted(out: &mut [f64], coeff: &[f64], k: &[Vec<f64>], h: f64, s: usize, n: usize) {
    for i in 0..s {
        let ci = coeff[i];
        if ci == 0.0 {
            continue;
        }
        for d in 0..n {
            out[d] += h * ci * k[i][d];
        }
    }
}

/// RMS norm of the error vector scaled by `atol + rtol·max(|y|,|yNew|)`.
///
/// Port of the package-visible `RungeKuttaMethod.errorNorm`, which
/// [`RosenbrockMethod`] and [`BdfMethod`] also call.
pub fn error_norm(err_vec: &[f64], y: &[f64], y_new: &[f64], rtol: f64, atol: f64) -> f64 {
    let n = err_vec.len();
    let mut sum = 0.0;
    for d in 0..n {
        let sc = atol + rtol * java_max(y[d].abs(), y_new[d].abs());
        let r = err_vec[d] / sc;
        sum += r * r;
    }
    (sum / n as f64).sqrt()
}

fn all_finite(v: &[f64]) -> bool {
    v.iter().all(|x| x.is_finite())
}

/// `RungeKuttaMethod.capStep` — written as `h > maxStep ? maxStep : h`, not
/// `Math.min`, and kept that way.
fn cap_step(h: f64, problem: &OdeProblem<'_>) -> f64 {
    match problem.max_step {
        Some(max) if h > max => max,
        _ => h,
    }
}

// ---------------------------------------------------------------------------
// Rosenbrock — ode23s
// ---------------------------------------------------------------------------

/// `RosenbrockMethod.SAFETY`.
const ROS_SAFETY: f64 = 0.9;
/// `RosenbrockMethod.MIN_SCALE`.
const ROS_MIN_SCALE: f64 = 0.2;
/// `RosenbrockMethod.MAX_SCALE`.
const ROS_MAX_SCALE: f64 = 5.0;

/// `ode23s` — Shampine's modified Rosenbrock (2,3) pair, a linearly implicit
/// one-step method for stiff systems.
///
/// Port of `RosenbrockMethod.java`. Each step forms the iteration matrix
/// `W = I − h·d·J` once from a finite-difference Jacobian, then takes three
/// linear solves; the third stage yields an embedded order-3 error estimate for
/// adaptive step control. L-stable and well-suited to Van der Pol (large μ) and
/// Robertson kinetics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RosenbrockMethod;

impl RosenbrockMethod {
    /// `RosenbrockMethod.D` — `1 / (2 + √2)`.
    fn d() -> f64 {
        1.0 / (2.0 + 2.0f64.sqrt())
    }

    /// `RosenbrockMethod.E32` — `6 + √2`.
    fn e32() -> f64 {
        6.0 + 2.0f64.sqrt()
    }
}

impl OdeMethod for RosenbrockMethod {
    fn name(&self) -> &str {
        "ode23s"
    }

    fn adaptive(&self) -> bool {
        true
    }

    fn order(&self) -> u32 {
        2
    }

    fn step(
        &self,
        f: &dyn OdeRhs,
        t: f64,
        y: &[f64],
        f0: &[f64],
        h: f64,
        p: &OdeProblem<'_>,
    ) -> Result<StepResult> {
        let d = RosenbrockMethod::d();
        let e32 = RosenbrockMethod::e32();
        let n = y.len();
        let span = p.tf - p.t0;
        let jac = jacobian(f, t, y, f0)?;
        let dfdt = dfdt(f, t, y, f0, span)?;
        let w = identity_minus(h * d, &jac);

        // Stage 1: W·k1 = f0 + h·d·(∂f/∂t)
        let mut b1 = vec![0.0; n];
        for i in 0..n {
            b1[i] = f0[i] + h * d * dfdt[i];
        }
        let k1 = solve(&w, &b1)?;

        // Stage 2: W·(k2−k1) = f(t+h/2, y+h/2·k1) − k1
        let mut y1 = vec![0.0; n];
        for i in 0..n {
            y1[i] = y[i] + 0.5 * h * k1[i];
        }
        let f1 = f.eval(t + 0.5 * h, &y1)?;
        let mut b2 = vec![0.0; n];
        for i in 0..n {
            b2[i] = f1[i] - k1[i];
        }
        let mut k2 = solve(&w, &b2)?;
        for i in 0..n {
            k2[i] += k1[i];
        }

        let mut y_new = vec![0.0; n];
        for i in 0..n {
            y_new[i] = y[i] + h * k2[i];
        }
        let f2 = f.eval(t + h, &y_new)?;

        // Stage 3 (error estimate): W·k3 = f2 − e32·(k2−f1) − 2·(k1−f0) + h·d·(∂f/∂t)
        let mut b3 = vec![0.0; n];
        for i in 0..n {
            b3[i] = f2[i] - e32 * (k2[i] - f1[i]) - 2.0 * (k1[i] - f0[i]) + h * d * dfdt[i];
        }
        let k3 = solve(&w, &b3)?;

        let mut err = vec![0.0; n];
        for i in 0..n {
            err[i] = (h / 6.0) * (k1[i] - 2.0 * k2[i] + k3[i]);
        }
        let err_norm = error_norm(&err, y, &y_new, p.rtol, p.atol);
        let exponent = 1.0 / 3.0;
        if err_norm <= 1.0 {
            let scale = if err_norm == 0.0 {
                ROS_MAX_SCALE
            } else {
                java_min(
                    ROS_MAX_SCALE,
                    java_max(ROS_MIN_SCALE, ROS_SAFETY * libm::pow(err_norm, -exponent)),
                )
            };
            let mut h_next = h * scale;
            if let Some(max) = p.max_step {
                h_next = java_min(h_next, max);
            }
            return Ok(StepResult::accept(y_new, f2, h_next));
        }
        let scale = java_max(ROS_MIN_SCALE, ROS_SAFETY * libm::pow(err_norm, -exponent));
        Ok(StepResult::reject(h * scale))
    }
}

// ---------------------------------------------------------------------------
// BDF — ode15s
// ---------------------------------------------------------------------------

/// `BdfMethod.SAFETY`.
const BDF_SAFETY: f64 = 0.9;
/// `BdfMethod.MIN_SCALE`.
const BDF_MIN_SCALE: f64 = 0.2;
/// `BdfMethod.MAX_SCALE` — 4.0, *not* the 5.0 the explicit pair uses.
const BDF_MAX_SCALE: f64 = 4.0;
/// `BdfMethod.NEWTON_MAX`.
const BDF_NEWTON_MAX: usize = 25;
/// `BdfMethod.NEWTON_TOL`.
const BDF_NEWTON_TOL: f64 = 1e-10;

/// `ode15s` — a stiff implicit BDF integrator.
///
/// Port of `BdfMethod.java`. Each step uses backward (implicit) Euler, which is
/// L-stable, and obtains an error estimate and an order-2 solution by step
/// doubling (Richardson extrapolation): one full step of size `h` versus two
/// half steps. The implicit stage is solved with a damped Newton iteration
/// using a finite-difference Jacobian. Also serves the `ode23t`/`ode23tb`
/// aliases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BdfMethod;

impl OdeMethod for BdfMethod {
    fn name(&self) -> &str {
        "ode15s"
    }

    fn adaptive(&self) -> bool {
        true
    }

    fn order(&self) -> u32 {
        2
    }

    fn step(
        &self,
        f: &dyn OdeRhs,
        t: f64,
        y: &[f64],
        f0: &[f64],
        h: f64,
        p: &OdeProblem<'_>,
    ) -> Result<StepResult> {
        let n = y.len();
        let y_big = implicit_euler(f, t, y, h, f0)?;
        let y_half = implicit_euler(f, t, y, 0.5 * h, f0)?;
        let f_half = f.eval(t + 0.5 * h, &y_half)?;
        let y_half2 = implicit_euler(f, t + 0.5 * h, &y_half, 0.5 * h, &f_half)?;

        let mut err = vec![0.0; n];
        for i in 0..n {
            err[i] = y_half2[i] - y_big[i];
        }
        let err_norm = error_norm(&err, y, &y_half2, p.rtol, p.atol);

        // Richardson extrapolation lifts the order-1 result to order 2.
        let mut y_new = vec![0.0; n];
        for i in 0..n {
            y_new[i] = 2.0 * y_half2[i] - y_big[i];
        }
        let exponent = 1.0 / 2.0;
        if err_norm <= 1.0 {
            let f_new = f.eval(t + h, &y_new)?;
            let scale = if err_norm == 0.0 {
                BDF_MAX_SCALE
            } else {
                java_min(
                    BDF_MAX_SCALE,
                    java_max(BDF_MIN_SCALE, BDF_SAFETY * libm::pow(err_norm, -exponent)),
                )
            };
            let mut h_next = h * scale;
            if let Some(max) = p.max_step {
                h_next = java_min(h_next, max);
            }
            return Ok(StepResult::accept(y_new, f_new, h_next));
        }
        let scale = java_max(BDF_MIN_SCALE, BDF_SAFETY * libm::pow(err_norm, -exponent));
        Ok(StepResult::reject(h * scale))
    }
}

/// Solves `y1 = y + h·f(t+h, y1)` by Newton with an FD Jacobian.
///
/// Port of `BdfMethod.implicitEuler`. Note the Java refactors the Jacobian
/// *inside* the loop — it is rebuilt at every Newton iterate, never reused
/// across iterations or across the three implicit solves of one step. That is
/// expensive and deliberate (the RHS is a closure over an algebraic solve, so a
/// stale Jacobian can stall the iteration); the reuse policy is "no reuse" and
/// is preserved.
fn implicit_euler(f: &dyn OdeRhs, t: f64, y: &[f64], h: f64, f0: &[f64]) -> Result<Vec<f64>> {
    let n = y.len();
    let tn = t + h;
    let mut y1 = vec![0.0; n];
    for i in 0..n {
        y1[i] = y[i] + h * f0[i]; // explicit-Euler predictor
    }
    for _ in 0..BDF_NEWTON_MAX {
        let f_eval = f.eval(tn, &y1)?;
        let mut residual = vec![0.0; n];
        for i in 0..n {
            residual[i] = -(y1[i] - y[i] - h * f_eval[i]);
        }
        let jac = jacobian(f, tn, &y1, &f_eval)?;
        let newton_matrix = identity_minus(h, &jac); // I − h·J
        let delta = solve(&newton_matrix, &residual)?;
        let mut dnorm = 0.0;
        let mut ynorm = 0.0;
        for i in 0..n {
            y1[i] += delta[i];
            dnorm += delta[i] * delta[i];
            ynorm += y1[i] * y1[i];
        }
        if dnorm.sqrt() <= BDF_NEWTON_TOL * (1.0 + ynorm.sqrt()) {
            break;
        }
    }
    Ok(y1)
}

// ---------------------------------------------------------------------------
// OdeLinearAlgebra
// ---------------------------------------------------------------------------

/// Commons Math `LUDecomposition` default singularity threshold — the same
/// `1e-11` [`crate::linalg`] transcribes.
const LU_SINGULARITY_THRESHOLD: f64 = 1e-11;

/// `Math.sqrt(Math.ulp(1.0))` — the forward-difference step scale.
fn fd_eps() -> f64 {
    f64::EPSILON.sqrt()
}

/// Forward-difference Jacobian `J[i][j] = ∂f_i/∂y_j` at `(t, y)`.
///
/// Port of `OdeLinearAlgebra.jacobian`. The RHS of a frees `DYNAMIC` block is a
/// closure over the algebraic solve, so a symbolic Jacobian of the whole
/// closure is impractical — a forward-difference Jacobian is the robust choice
/// (the symbolic `Differentiator` still grounds the per-step Newton inside the
/// algebraic block).
///
/// The divisor is `yp[col] - yj`, the *realised* perturbation after rounding,
/// not the nominal `delta`.
pub fn jacobian(f: &dyn OdeRhs, t: f64, y: &[f64], f0: &[f64]) -> Result<Vec<Vec<f64>>> {
    let n = y.len();
    let mut j = vec![vec![0.0; n]; n];
    for col in 0..n {
        let yj = y[col];
        let delta = fd_eps() * java_max(yj.abs(), 1.0);
        let mut yp = y.to_vec();
        yp[col] = yj + delta;
        let fp = f.eval(t, &yp)?;
        let inv = 1.0 / (yp[col] - yj);
        for row in 0..n {
            j[row][col] = (fp[row] - f0[row]) * inv;
        }
    }
    Ok(j)
}

/// Forward-difference partial `∂f/∂t` at `(t, y)`.
///
/// Port of `OdeLinearAlgebra.dfdt`.
pub fn dfdt(f: &dyn OdeRhs, t: f64, y: &[f64], f0: &[f64], span: f64) -> Result<Vec<f64>> {
    let dt = fd_eps() * java_max(t.abs(), span);
    if dt == 0.0 {
        return Ok(vec![0.0; y.len()]);
    }
    let fp = f.eval(t + dt, y)?;
    let mut out = vec![0.0; y.len()];
    for i in 0..out.len() {
        out[i] = (fp[i] - f0[i]) / dt;
    }
    Ok(out)
}

/// `I − scale·J`, the iteration matrix shared by the stiff methods.
///
/// Port of `OdeLinearAlgebra.identityMinus`.
pub fn identity_minus(scale: f64, jac: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let n = jac.len();
    let mut w = vec![vec![0.0; n]; n];
    for i in 0..n {
        for k in 0..n {
            w[i][k] = (if i == k { 1.0 } else { 0.0 }) - scale * jac[i][k];
        }
    }
    w
}

/// Solves `A x = b` — LU with partial pivoting, falling back to least-squares
/// QR when LU reports the matrix singular.
///
/// Port of `OdeLinearAlgebra.solve`, which is Commons Math
/// `LUDecomposition(m).getSolver().solve(rhs)` with a
/// `catch (SingularMatrixException) -> QRDecomposition(m).getSolver().solve(rhs)`.
/// Both decompositions are transcribed from Commons Math 3.6.1 (the same
/// Crout-order LU and Householder QR [`crate::linalg`] already carries) so the
/// arithmetic order — and therefore the last bits — match the oracle.
///
/// When the QR fallback is *also* singular Commons Math throws again and the
/// Java propagates an unchecked exception out of the integrator; here that
/// becomes a solver error.
pub fn solve(a: &[Vec<f64>], b: &[f64]) -> Result<Vec<f64>> {
    match lu_solve(a, b) {
        Some(x) => Ok(x),
        None => qr_least_squares(a, b),
    }
}

/// Commons Math `LUDecomposition` + `solve(RealVector)`. `None` mirrors
/// `SingularMatrixException`.
fn lu_solve(a: &[Vec<f64>], b: &[f64]) -> Option<Vec<f64>> {
    let m = a.len();
    let mut lu: Vec<Vec<f64>> = a.to_vec();
    let mut pivot: Vec<usize> = (0..m).collect();
    for col in 0..m {
        // Upper part.
        for row in 0..col {
            let mut sum = lu[row][col];
            for i in 0..row {
                sum -= lu[row][i] * lu[i][col];
            }
            lu[row][col] = sum;
        }
        // Lower part, tracking the largest pivot candidate.
        let mut max = col;
        let mut largest = f64::NEG_INFINITY;
        for row in col..m {
            let mut sum = lu[row][col];
            for i in 0..col {
                sum -= lu[row][i] * lu[i][col];
            }
            lu[row][col] = sum;
            if sum.abs() > largest {
                largest = sum.abs();
                max = row;
            }
        }
        if lu[max][col].abs() < LU_SINGULARITY_THRESHOLD {
            return None; // SingularMatrixException
        }
        if max != col {
            lu.swap(max, col);
            pivot.swap(max, col);
        }
        let diag = lu[col][col];
        for row in (col + 1)..m {
            lu[row][col] /= diag;
        }
    }

    // Apply permutations to b, then forward/back substitution.
    let mut bp: Vec<f64> = pivot.iter().map(|&p| b[p]).collect();
    for col in 0..m {
        let bp_col = bp[col];
        for i in (col + 1)..m {
            bp[i] -= bp_col * lu[i][col];
        }
    }
    for col in (0..m).rev() {
        bp[col] /= lu[col][col];
        let bp_col = bp[col];
        for i in 0..col {
            bp[i] -= bp_col * lu[i][col];
        }
    }
    Some(bp)
}

/// Commons Math `QRDecomposition` (threshold 0) + `solve(RealVector)`.
fn qr_least_squares(a: &[Vec<f64>], b: &[f64]) -> Result<Vec<f64>> {
    let m = a.len();
    let n = if m == 0 { 0 } else { a[0].len() };
    // Commons Math stores the TRANSPOSE of A and reflects in place.
    let mut qrt: Vec<Vec<f64>> = (0..n).map(|j| (0..m).map(|i| a[i][j]).collect()).collect();
    let p = m.min(n);
    let mut r_diag = vec![0.0; p];
    for minor in 0..p {
        let norm_sq: f64 = (minor..m).map(|i| qrt[minor][i] * qrt[minor][i]).sum();
        let norm = norm_sq.sqrt();
        // Sign choice: a = -sign(pivot) * norm, exactly as Commons Math.
        let alpha = if qrt[minor][minor] > 0.0 { -norm } else { norm };
        r_diag[minor] = alpha;
        if alpha != 0.0 {
            qrt[minor][minor] -= alpha;
            for col in (minor + 1)..n {
                let mut dot = 0.0;
                for i in minor..m {
                    dot -= qrt[col][i] * qrt[minor][i];
                }
                let factor = dot / (alpha * qrt[minor][minor]);
                for i in minor..m {
                    let v = qrt[minor][i];
                    qrt[col][i] -= factor * v;
                }
            }
        }
    }
    // `isNonSingular()` with the default threshold of exactly 0.
    if r_diag.iter().any(|d| d.abs() <= 0.0) {
        return Err(FreesError::solver(
            "DYNAMIC: the stiff iteration matrix is singular — the system may be \
             index-2 or the Jacobian degenerate.",
        ));
    }

    let mut x = vec![0.0; n];
    let mut y = b.to_vec();
    // Apply the Householder transforms to solve Q·y = b.
    for minor in 0..p {
        let mut dot_product = 0.0;
        for row in minor..m {
            dot_product += y[row] * qrt[minor][row];
        }
        dot_product /= r_diag[minor] * qrt[minor][minor];
        for row in minor..m {
            y[row] += dot_product * qrt[minor][row];
        }
    }
    // Solve the triangular system R·x = y.
    for row in (0..r_diag.len()).rev() {
        y[row] /= r_diag[row];
        let y_row = y[row];
        x[row] = y_row;
        for i in 0..row {
            y[i] -= y_row * qrt[row][i];
        }
    }
    Ok(x)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn problem<'a>(rhs: &'a dyn OdeRhs, y0: Vec<f64>) -> OdeProblem<'a> {
        OdeProblem {
            method: "ode45".into(),
            t0: 0.0,
            tf: 1.0,
            y0,
            rhs,
            points: None,
            fixed_step: None,
            rtol: 1e-6,
            atol: 1e-9,
            max_step: None,
            events: Vec::new(),
        }
    }

    // ── Tableaux ────────────────────────────────────────────────────────────

    /// A Butcher tableau is consistent iff `Σ b = 1` and `Σ_j a[i][j] = c[i]`
    /// for every stage. This catches a mistyped coefficient that a
    /// trajectory test would only reveal as a tolerance failure.
    fn assert_consistent(t: &ButcherTableau) {
        let sum_b: f64 = t.b.iter().sum();
        assert!((sum_b - 1.0).abs() < 1e-14, "{}: sum(b) = {sum_b}", t.name);
        for i in 0..t.stages {
            let row: f64 = t.a[i].iter().sum();
            assert!(
                (row - t.c[i]).abs() < 1e-14,
                "{}: row {i} sums to {row}, c = {}",
                t.name,
                t.c[i]
            );
        }
    }

    #[test]
    fn every_tableau_is_row_consistent() {
        for t in [
            ButcherTableau::euler(),
            ButcherTableau::heun(),
            ButcherTableau::rk3(),
            ButcherTableau::rk4(),
            ButcherTableau::dopri5_fixed(),
            ButcherTableau::dopri54(),
            ButcherTableau::bogacki_shampine32(),
        ] {
            assert_consistent(&t);
        }
    }

    #[test]
    fn embedded_pairs_have_consistent_low_order_weights() {
        // sum(bErr) = sum(b) - sum(bHat) = 1 - 1 = 0.
        for t in [
            ButcherTableau::dopri54(),
            ButcherTableau::bogacki_shampine32(),
        ] {
            let s: f64 = t.b_err.as_ref().unwrap().iter().sum();
            assert!(s.abs() < 1e-14, "{}: sum(bErr) = {s}", t.name);
        }
    }

    #[test]
    fn dopri54_coefficients_are_the_transcribed_literals() {
        let t = ButcherTableau::dopri54();
        assert_eq!(t.stages, 7);
        assert_eq!(t.error_order, 4);
        assert!(t.fsal);
        assert!(t.adaptive());
        assert_eq!(t.c[4], 8.0 / 9.0);
        assert_eq!(t.a[4][0], 19372.0 / 6561.0);
        assert_eq!(t.a[4][1], -25360.0 / 2187.0);
        assert_eq!(t.a[5][4], -5103.0 / 18656.0);
        assert_eq!(t.b[2], 500.0 / 1113.0);
        assert_eq!(t.b_err.as_ref().unwrap()[6], 0.0 - 1.0 / 40.0);
        assert_eq!(
            t.b_err.as_ref().unwrap()[0],
            35.0 / 384.0 - 5179.0 / 57600.0
        );
        // FSAL: the last a-row equals b (minus its trailing zero).
        for i in 0..6 {
            assert_eq!(t.a[6][i], t.b[i]);
        }
    }

    #[test]
    fn bogacki_shampine_is_fsal_and_order_two_embedded() {
        let t = ButcherTableau::bogacki_shampine32();
        assert_eq!(t.stages, 4);
        assert_eq!(t.error_order, 2);
        assert!(t.fsal);
        for i in 0..3 {
            assert_eq!(t.a[3][i], t.b[i]);
        }
    }

    #[test]
    fn ode5_reuses_the_dopri_weights_without_the_error_estimate() {
        let fixed = ButcherTableau::dopri5_fixed();
        let pair = ButcherTableau::dopri54();
        assert_eq!(fixed.name, "ode5");
        assert!(!fixed.adaptive());
        assert_eq!(fixed.error_order, 5);
        assert_eq!(fixed.b, pair.b);
        assert_eq!(fixed.c, pair.c);
        assert_eq!(fixed.a, pair.a);
    }

    #[test]
    fn method_order_lifts_only_for_embedded_pairs() {
        assert_eq!(RungeKuttaMethod::new(ButcherTableau::euler()).order(), 1);
        assert_eq!(RungeKuttaMethod::new(ButcherTableau::rk4()).order(), 4);
        assert_eq!(
            RungeKuttaMethod::new(ButcherTableau::dopri5_fixed()).order(),
            5
        );
        // Adaptive: errorOrder + 1.
        assert_eq!(RungeKuttaMethod::new(ButcherTableau::dopri54()).order(), 5);
        assert_eq!(
            RungeKuttaMethod::new(ButcherTableau::bogacki_shampine32()).order(),
            3
        );
        assert_eq!(RosenbrockMethod.order(), 2);
        assert_eq!(BdfMethod.order(), 2);
        assert_eq!(RosenbrockMethod.name(), "ode23s");
        assert_eq!(BdfMethod.name(), "ode15s");
        assert!(RosenbrockMethod.adaptive());
        assert!(BdfMethod.adaptive());
    }

    // ── One step against the analytic solution ──────────────────────────────

    /// A single step of `y' = y`, `y(0) = 1`, `h = 0.1`. Each fixed-step
    /// tableau must reproduce the truncated exponential series to its own
    /// order: `Σ_{k<=order} h^k / k!`.
    #[test]
    fn fixed_step_methods_hit_their_taylor_order() {
        let rhs = |_t: f64, y: &[f64]| Ok(vec![y[0]]);
        let p = problem(&rhs, vec![1.0]);
        let h = 0.1;
        for (tableau, order) in [
            (ButcherTableau::euler(), 1u32),
            (ButcherTableau::heun(), 2),
            (ButcherTableau::rk3(), 3),
            (ButcherTableau::rk4(), 4),
        ] {
            let name = tableau.name;
            let m = RungeKuttaMethod::new(tableau);
            let sr = m.step(&rhs, 0.0, &[1.0], &[1.0], h, &p).unwrap();
            assert!(sr.accepted);
            let mut series = 0.0;
            let mut term = 1.0;
            for k in 0..=order {
                if k > 0 {
                    term *= h / f64::from(k);
                }
                series += term;
            }
            let got = sr.y_new.unwrap()[0];
            assert!(
                (got - series).abs() < 1e-15,
                "{name}: got {got}, Taylor({order}) = {series}"
            );
            // Fixed-step methods never change h.
            assert_eq!(sr.h_next, h);
        }
    }

    #[test]
    fn adaptive_step_grows_on_an_easy_problem_and_reports_f_new() {
        let rhs = |_t: f64, y: &[f64]| Ok(vec![y[0]]);
        let p = problem(&rhs, vec![1.0]);
        let m = RungeKuttaMethod::new(ButcherTableau::dopri54());
        let sr = m.step(&rhs, 0.0, &[1.0], &[1.0], 1e-3, &p).unwrap();
        assert!(sr.accepted);
        assert!(sr.h_next > 1e-3, "step should grow, got {}", sr.h_next);
        assert!(sr.h_next <= 5.0 * 1e-3, "growth is capped at MAX_SCALE");
        let y_new = sr.y_new.unwrap();
        assert!((y_new[0] - libm::exp(1e-3)).abs() < 1e-12);
        // f_new = f(t+h, y_new) = y_new for this RHS.
        assert_eq!(sr.f_new.unwrap()[0], y_new[0]);
    }

    #[test]
    fn adaptive_step_is_rejected_and_shrunk_when_the_error_is_large() {
        // Tight tolerances plus a big step force a rejection.
        let rhs = |_t: f64, y: &[f64]| Ok(vec![-100.0 * y[0]]);
        let mut p = problem(&rhs, vec![1.0]);
        p.rtol = 1e-12;
        p.atol = 1e-14;
        let m = RungeKuttaMethod::new(ButcherTableau::bogacki_shampine32());
        let sr = m.step(&rhs, 0.0, &[1.0], &[-100.0], 0.5, &p).unwrap();
        assert!(!sr.accepted);
        assert!(sr.y_new.is_none() && sr.f_new.is_none());
        assert!(sr.h_next < 0.5);
        assert!(sr.h_next >= 0.2 * 0.5, "shrink is floored at MIN_SCALE");
    }

    #[test]
    fn a_diverged_stage_rejects_hard_at_min_scale() {
        // The RHS blows up to infinity away from the origin.
        let rhs = |_t: f64, y: &[f64]| Ok(vec![if y[0] > 0.5 { f64::INFINITY } else { 1.0 }]);
        let p = problem(&rhs, vec![0.0]);
        let m = RungeKuttaMethod::new(ButcherTableau::dopri54());
        let sr = m.step(&rhs, 0.0, &[0.0], &[1.0], 1.0, &p).unwrap();
        assert!(!sr.accepted);
        assert_eq!(sr.h_next, 1.0 * RK_MIN_SCALE);
    }

    #[test]
    fn max_step_caps_an_accepted_adaptive_step() {
        let rhs = |_t: f64, y: &[f64]| Ok(vec![y[0]]);
        let mut p = problem(&rhs, vec![1.0]);
        p.max_step = Some(2e-3);
        let m = RungeKuttaMethod::new(ButcherTableau::dopri54());
        let sr = m.step(&rhs, 0.0, &[1.0], &[1.0], 1e-3, &p).unwrap();
        assert!(sr.accepted);
        assert_eq!(sr.h_next, 2e-3);
    }

    #[test]
    fn rhs_failure_propagates_out_of_a_step() {
        let rhs = |_t: f64, _y: &[f64]| Err(FreesError::solver("inner block did not converge"));
        let p = problem(&rhs, vec![1.0]);
        let m = RungeKuttaMethod::new(ButcherTableau::rk4());
        let err = m.step(&rhs, 0.0, &[1.0], &[1.0], 0.1, &p).unwrap_err();
        assert!(format!("{err}").contains("did not converge"));
    }

    // ── Stiff steppers ──────────────────────────────────────────────────────

    #[test]
    fn rosenbrock_matches_the_exponential_to_its_order() {
        let rhs = |_t: f64, y: &[f64]| Ok(vec![-y[0]]);
        let p = problem(&rhs, vec![1.0]);
        let h = 0.01;
        let sr = RosenbrockMethod
            .step(&rhs, 0.0, &[1.0], &[-1.0], h, &p)
            .unwrap();
        assert!(sr.accepted);
        let got = sr.y_new.unwrap()[0];
        // Order-2 (locally order-3) accurate: |e^-h - y| ~ h^3.
        assert!(
            (got - libm::exp(-h)).abs() < 1e-7,
            "got {got}, exp(-h) = {}",
            libm::exp(-h)
        );
    }

    /// Richardson extrapolation lifts the two backward-Euler solves to order 2,
    /// so the accepted state tracks `e^-h` to `O(h^3)`.
    #[test]
    fn bdf_matches_the_exponential_to_its_order() {
        let rhs = |_t: f64, y: &[f64]| Ok(vec![-y[0]]);
        let p = problem(&rhs, vec![1.0]);
        let h = 0.001;
        let sr = BdfMethod.step(&rhs, 0.0, &[1.0], &[-1.0], h, &p).unwrap();
        assert!(sr.accepted);
        let got = sr.y_new.unwrap()[0];
        assert!(
            (got - libm::exp(-h)).abs() < 1e-9,
            "got {got}, exp(-h) = {}",
            libm::exp(-h)
        );
    }

    /// The step-doubling estimator is only order 1 before extrapolation, so at
    /// the default `rtol = 1e-6` a step of 0.01 on `y' = -y` is *rejected* —
    /// `|yHalf2 − yBig| ≈ 2.45e-5` against a scale of `1e-6`. Pinned because it
    /// looks like a failure and is in fact the controller working.
    #[test]
    fn bdf_rejects_a_step_whose_richardson_difference_exceeds_the_tolerance() {
        let rhs = |_t: f64, y: &[f64]| Ok(vec![-y[0]]);
        let p = problem(&rhs, vec![1.0]);
        let sr = BdfMethod
            .step(&rhs, 0.0, &[1.0], &[-1.0], 0.01, &p)
            .unwrap();
        assert!(!sr.accepted);
        assert!(sr.h_next < 0.01 && sr.h_next >= 0.2 * 0.01);
    }

    /// The whole point of the stiff path: a step far larger than the fast
    /// time constant must stay bounded, where explicit Euler would blow up.
    #[test]
    fn stiff_steppers_are_stable_at_a_step_far_beyond_the_time_constant() {
        let lambda = -1.0e5;
        let rhs = move |_t: f64, y: &[f64]| Ok(vec![lambda * y[0]]);
        let mut p = problem(&rhs, vec![1.0]);
        p.rtol = 1e-3;
        p.atol = 1e-6;
        let h = 1.0; // 100_000x the time constant
        for (label, sr) in [
            (
                "ode23s",
                RosenbrockMethod
                    .step(&rhs, 0.0, &[1.0], &[lambda], h, &p)
                    .unwrap(),
            ),
            (
                "ode15s",
                BdfMethod.step(&rhs, 0.0, &[1.0], &[lambda], h, &p).unwrap(),
            ),
        ] {
            // Accepted or not, the proposed state must be bounded and decaying.
            let y = sr
                .y_new
                .unwrap_or_else(|| vec![0.0])
                .first()
                .copied()
                .unwrap();
            assert!(y.abs() < 1.0, "{label}: |y| = {} is not damped", y.abs());
        }
    }

    #[test]
    fn bdf_solves_a_coupled_two_state_system() {
        // y1' = -2 y1 + y2, y2' = y1 - 2 y2. Eigenvalues -1, -3.
        let rhs = |_t: f64, y: &[f64]| Ok(vec![-2.0 * y[0] + y[1], y[0] - 2.0 * y[1]]);
        let mut p = problem(&rhs, vec![1.0, 0.0]);
        // `y2` starts at exactly 0, so its error scale is `atol + rtol·|y2|`
        // with `y2 ≈ 1e-3` — the default `atol = 1e-9` floor, not the stiffness,
        // is what makes the default tolerances reject this step.
        p.rtol = 1e-4;
        p.atol = 1e-6;
        let h = 0.001;
        let f0 = rhs(0.0, &[1.0, 0.0]).unwrap();
        let sr = BdfMethod.step(&rhs, 0.0, &[1.0, 0.0], &f0, h, &p).unwrap();
        assert!(sr.accepted);
        let y = sr.y_new.unwrap();
        // Analytic: y1 = (e^-t + e^-3t)/2, y2 = (e^-t - e^-3t)/2. The
        // extrapolated result is order 2, so `O(h^3) ≈ 2e-9` is the floor
        // even with an exact linear solve.
        let e1 = libm::exp(-h);
        let e3 = libm::exp(-3.0 * h);
        assert!((y[0] - 0.5 * (e1 + e3)).abs() < 1e-8, "y0 = {}", y[0]);
        assert!((y[1] - 0.5 * (e1 - e3)).abs() < 1e-8, "y1 = {}", y[1]);
    }

    // ── Linear algebra ──────────────────────────────────────────────────────

    #[test]
    fn finite_difference_jacobian_matches_the_analytic_one() {
        // f = [y0*y1, sin(y0) + 3*y1]
        let rhs = |_t: f64, y: &[f64]| Ok(vec![y[0] * y[1], libm::sin(y[0]) + 3.0 * y[1]]);
        let y = [2.0, -0.5];
        let f0 = rhs(0.0, &y).unwrap();
        let j = jacobian(&rhs, 0.0, &y, &f0).unwrap();
        let expect = [[y[1], y[0]], [libm::cos(y[0]), 3.0]];
        for r in 0..2 {
            for c in 0..2 {
                assert!(
                    (j[r][c] - expect[r][c]).abs() < 1e-6,
                    "J[{r}][{c}] = {}, expected {}",
                    j[r][c],
                    expect[r][c]
                );
            }
        }
    }

    #[test]
    fn dfdt_matches_the_analytic_time_derivative() {
        // f = [3t + y0]  =>  df/dt = [3]
        let rhs = |t: f64, y: &[f64]| Ok(vec![3.0 * t + y[0]]);
        let y = [1.0];
        let f0 = rhs(2.0, &y).unwrap();
        let d = dfdt(&rhs, 2.0, &y, &f0, 10.0).unwrap();
        assert!((d[0] - 3.0).abs() < 1e-6, "df/dt = {}", d[0]);
    }

    #[test]
    fn dfdt_is_zero_when_the_perturbation_underflows() {
        let rhs = |_t: f64, _y: &[f64]| Ok(vec![1.0, 2.0]);
        let f0 = [1.0, 2.0];
        // t = 0 and span = 0 makes dt exactly 0.
        let d = dfdt(&rhs, 0.0, &[0.0, 0.0], &f0, 0.0).unwrap();
        assert_eq!(d, vec![0.0, 0.0]);
    }

    #[test]
    fn identity_minus_builds_the_iteration_matrix() {
        let j = vec![vec![1.0, 2.0], vec![3.0, 4.0]];
        let w = identity_minus(0.5, &j);
        assert_eq!(w, vec![vec![0.5, -1.0], vec![-1.5, -1.0]]);
    }

    #[test]
    fn lu_solve_is_exact_on_a_well_conditioned_system() {
        let a = vec![
            vec![4.0, -2.0, 1.0],
            vec![-2.0, 4.0, -2.0],
            vec![1.0, -2.0, 4.0],
        ];
        let x_true = [1.0, -2.0, 3.0];
        let b: Vec<f64> = (0..3)
            .map(|i| (0..3).map(|j| a[i][j] * x_true[j]).sum())
            .collect();
        let x = solve(&a, &b).unwrap();
        for i in 0..3 {
            assert!((x[i] - x_true[i]).abs() < 1e-12, "x[{i}] = {}", x[i]);
        }
    }

    #[test]
    fn lu_solve_pivots_when_the_leading_entry_is_zero() {
        let a = vec![vec![0.0, 1.0], vec![1.0, 0.0]];
        let x = solve(&a, &[3.0, 7.0]).unwrap();
        assert_eq!(x, vec![7.0, 3.0]);
    }

    #[test]
    fn a_consistent_singular_system_falls_through_lu_to_the_qr_least_squares_path() {
        // Rank-1 but consistent: LU hits the 1e-11 threshold and bails, and the
        // QR fallback returns a least-squares solution — exact here. Note QR's
        // own singularity threshold is exactly 0, so the round-off left in
        // rDiag[1] (~1e-16, not 0) keeps `isNonSingular()` true, which is the
        // Commons Math behaviour being reproduced.
        let a = vec![vec![1.0, 2.0], vec![2.0, 4.0]];
        let b = [3.0, 6.0];
        assert!(lu_solve(&a, &b).is_none());
        let x = solve(&a, &b).unwrap();
        for (i, row) in a.iter().enumerate() {
            let residual = row[0] * x[0] + row[1] * x[1] - b[i];
            assert!(residual.abs() < 1e-12, "residual[{i}] = {residual}");
        }
    }

    #[test]
    fn a_structurally_zero_column_exhausts_both_decompositions() {
        // `rDiag[0]` is *exactly* 0, which is what the zero threshold catches —
        // the point where the Java throws a second SingularMatrixException out
        // of `OdeLinearAlgebra.solve` and the integration dies.
        let a = vec![vec![0.0, 0.0], vec![0.0, 1.0]];
        assert!(lu_solve(&a, &[1.0, 2.0]).is_none());
        let err = solve(&a, &[1.0, 2.0]).unwrap_err();
        assert!(format!("{err}").contains("singular"), "{err}");
    }

    #[test]
    fn qr_fallback_solves_a_system_lu_rejects_for_being_near_singular() {
        // Pivot magnitude 1e-13 < 1e-11 threshold, but the matrix is invertible.
        let a = vec![vec![1.0e-13, 0.0], vec![0.0, 1.0]];
        assert!(lu_solve(&a, &[1.0e-13, 2.0]).is_none());
        let x = solve(&a, &[1.0e-13, 2.0]).unwrap();
        assert!((x[0] - 1.0).abs() < 1e-6, "x[0] = {}", x[0]);
        assert!((x[1] - 2.0).abs() < 1e-12, "x[1] = {}", x[1]);
    }

    // ── Java float semantics ────────────────────────────────────────────────

    #[test]
    fn java_min_max_propagate_nan_where_rust_would_not() {
        assert!(java_max(0.2, f64::NAN).is_nan());
        assert!(java_max(f64::NAN, 0.2).is_nan());
        assert!(java_min(0.2, f64::NAN).is_nan());
        assert!(java_min(f64::NAN, 0.2).is_nan());
        // Rust's own min/max do the opposite — this is exactly the divergence.
        assert_eq!(0.2f64.max(f64::NAN), 0.2);
        // Signed zero, as Java specifies it.
        assert!(java_max(0.0, -0.0).is_sign_positive());
        assert!(java_min(0.0, -0.0).is_sign_negative());
        assert_eq!(java_max(2.0, 3.0), 3.0);
        assert_eq!(java_min(2.0, 3.0), 2.0);
    }

    #[test]
    fn error_norm_is_the_scaled_rms() {
        // err = [1e-3, -1e-3], y = yNew = [1, 1], rtol = 1e-3, atol = 0.
        // sc = 1e-3 each, so r = [1, -1] and the RMS is 1.
        let e = error_norm(&[1e-3, -1e-3], &[1.0, 1.0], &[1.0, 1.0], 1e-3, 0.0);
        assert!((e - 1.0).abs() < 1e-15, "err = {e}");
    }

    #[test]
    fn error_norm_takes_the_larger_of_y_and_y_new_for_the_scale() {
        // |yNew| dominates: sc = atol + rtol*10.
        let e = error_norm(&[1.0], &[1.0], &[10.0], 0.1, 0.0);
        assert!((e - 1.0).abs() < 1e-15, "err = {e}");
    }

    #[test]
    fn error_norm_is_nan_when_the_new_state_diverged() {
        let e = error_norm(&[f64::NAN], &[1.0], &[f64::NAN], 1e-6, 1e-9);
        assert!(e.is_nan());
    }

    #[test]
    fn cap_step_leaves_a_step_under_the_cap_alone() {
        let rhs = |_t: f64, _y: &[f64]| Ok(vec![0.0]);
        let mut p = problem(&rhs, vec![0.0]);
        assert_eq!(cap_step(3.0, &p), 3.0);
        p.max_step = Some(2.0);
        assert_eq!(cap_step(3.0, &p), 2.0);
        assert_eq!(cap_step(1.0, &p), 1.0);
        assert_eq!(cap_step(2.0, &p), 2.0);
    }
}
