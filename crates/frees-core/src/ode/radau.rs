//! Opt-in three-stage, fifth-order Radau IIA for explicit ODE right-hand sides.
//! Tableau: Hairer, https://unige.ch/~hairer/poly/poly.pdf, Table III.4.
#![allow(clippy::needless_range_loop)]

use super::methods::{
    error_norm, identity_minus, jacobian, java_max, java_min, solve, OdeMethod, StepResult,
};
use super::problem::{OdeProblem, OdeRhs};
use crate::diag::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RadauMethod;

fn tableau() -> ([f64; 3], [[f64; 3]; 3]) {
    let s = libm::sqrt(6.0);
    (
        [(4.0 - s) / 10.0, (4.0 + s) / 10.0, 1.0],
        [
            [
                (88.0 - 7.0 * s) / 360.0,
                (296.0 - 169.0 * s) / 1800.0,
                (-2.0 + 3.0 * s) / 225.0,
            ],
            [
                (296.0 + 169.0 * s) / 1800.0,
                (88.0 + 7.0 * s) / 360.0,
                (-2.0 - 3.0 * s) / 225.0,
            ],
            [(16.0 - s) / 36.0, (16.0 + s) / 36.0, 1.0 / 9.0],
        ],
    )
}

fn reject(h: f64) -> StepResult {
    StepResult {
        accepted: false,
        y_new: None,
        f_new: None,
        h_next: h,
    }
}

impl OdeMethod for RadauMethod {
    fn name(&self) -> &str {
        "radau"
    }
    fn adaptive(&self) -> bool {
        true
    }
    fn order(&self) -> u32 {
        5
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
        let (c, a) = tableau();
        let n = y.len();
        let jac = jacobian(f, t, y, f0)?;
        // Full coupled simplified Newton: W = I_(3n) - h (A ⊗ J).
        // A scalar I-h*gamma*J alone cannot solve the three stage equations.
        // ponytail: dense O((3n)^3) factorization per iteration; use the
        // real/complex eigenvalue split and cached LU if large systems need it.
        let mut w = vec![vec![0.0; 3 * n]; 3 * n];
        for i in 0..3 {
            for j in 0..3 {
                for d in 0..n {
                    for e in 0..n {
                        w[i * n + d][j * n + e] = if i == j && d == e { 1.0 } else { 0.0 };
                        w[i * n + d][j * n + e] -= h * a[i][j] * jac[d][e];
                    }
                }
            }
        }
        // Solve for stage increments z_i = Y_i-y; avoid subtracting large
        // states in the residual. A constant predictor also avoids stiff
        // explicit-Euler overshoot before Newton has a chance to act.
        let mut z = vec![vec![0.0; n]; 3];
        let mut k = vec![vec![0.0; n]; 3];
        let mut converged = false;
        for _ in 0..20 {
            for i in 0..3 {
                let yi: Vec<f64> = (0..n).map(|d| y[d] + z[i][d]).collect();
                k[i] = f.eval(t + c[i] * h, &yi)?;
            }
            let mut residual = vec![0.0; 3 * n];
            for i in 0..3 {
                for d in 0..n {
                    residual[i * n + d] = -z[i][d];
                    for j in 0..3 {
                        residual[i * n + d] += h * a[i][j] * k[j][d];
                    }
                }
            }
            if !residual.iter().all(|v| v.is_finite()) {
                return Ok(reject(h * 0.2));
            }
            let Ok(delta) = solve(&w, &residual) else {
                return Ok(reject(h * 0.2));
            };
            let mut norm = 0.0;
            for i in 0..3 {
                for d in 0..n {
                    z[i][d] += delta[i * n + d];
                    let scale = p.atol + p.rtol * java_max(y[d].abs(), (y[d] + z[i][d]).abs());
                    norm = java_max(norm, (delta[i * n + d] / scale).abs());
                }
            }
            if !norm.is_finite() {
                return Ok(reject(h * 0.2));
            }
            if norm <= 0.01 {
                converged = true;
                break;
            }
        }
        if !converged {
            return Ok(reject(h * 0.2));
        }
        for i in 0..3 {
            let yi: Vec<f64> = (0..n).map(|d| y[d] + z[i][d]).collect();
            k[i] = f.eval(t + c[i] * h, &yi)?;
        }
        // Stiff accuracy: b is the last row of A, so y_new = Y_3.
        let y_new: Vec<f64> = (0..n).map(|d| y[d] + z[2][d]).collect();
        // Embedded order 3: b_hat0=gamma, b_hat_i=b_i-gamma*L_i(0),
        // where L_i interpolate at c. Thus moments through degree 2 match;
        // A*c=c^2/2 supplies the remaining third-order tree condition.
        // Smooth the O(h^4) difference with I-h*gamma*J, gamma=A[2][2]=1/9.
        // This preserves its order while damping the estimator's stiff modes.
        let gamma = a[2][2];
        let s = libm::sqrt(6.0);
        let l = [(2.0 + 3.0 * s) / 6.0, (2.0 - 3.0 * s) / 6.0, 1.0 / 3.0];
        let raw: Vec<f64> = (0..n)
            .map(|d| h * gamma * (f0[d] - (0..3).map(|i| l[i] * k[i][d]).sum::<f64>()))
            .collect();
        let Ok(err) = solve(&identity_minus(h * gamma, &jac), &raw) else {
            return Ok(reject(h * 0.2));
        };
        let norm = error_norm(&err, y, &y_new, p.rtol, p.atol);
        if !norm.is_finite() || !y_new.iter().chain(k[2].iter()).all(|v| v.is_finite()) {
            return Ok(reject(h * 0.2));
        }
        let scale = if norm == 0.0 {
            4.0
        } else {
            java_min(4.0, java_max(0.2, 0.9 * libm::pow(norm, -0.25)))
        };
        if norm > 1.0 {
            return Ok(reject(h * scale));
        }
        let h_next = p.max_step.map_or(h * scale, |max| java_min(h * scale, max));
        Ok(StepResult {
            accepted: true,
            y_new: Some(y_new),
            f_new: Some(k.pop().unwrap()),
            h_next,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ode::integrator::{integrate, resolve_method};

    fn problem(rhs: &dyn OdeRhs, y0: Vec<f64>, tf: f64) -> OdeProblem<'_> {
        OdeProblem {
            method: "radau".into(),
            t0: 0.0,
            tf,
            y0,
            rhs,
            points: Some(401),
            fixed_step: None,
            rtol: 1e-6,
            atol: 1e-10,
            max_step: None,
            events: vec![],
        }
    }

    #[test]
    fn resolve_aliases_and_preserve_defaults() {
        for alias in ["radau", "radau5", "RADAU", "radauiia"] {
            let m = resolve_method(alias).unwrap();
            assert_eq!(m.name(), "radau");
            assert_eq!(m.order(), 5);
            assert!(m.adaptive());
        }
        for (alias, name) in [
            ("", "ode45"),
            ("ode45", "ode45"),
            ("ode15s", "ode15s"),
            ("ode23t", "ode15s"),
            ("ode23tb", "ode15s"),
        ] {
            assert_eq!(resolve_method(alias).unwrap().name(), name);
        }
        assert!(resolve_method("not-a-method").is_err());
    }

    #[test]
    fn harmonic_oscillator() {
        let rhs = |_t: f64, y: &[f64]| Ok(vec![y[1], -y[0]]);
        let tf = 20.0 * core::f64::consts::PI;
        let r = integrate(&problem(&rhs, vec![1.0, 0.0], tf)).unwrap();
        assert_eq!(r.end_time, tf);
        for (t, y) in r.times.iter().zip(&r.states) {
            assert!((y[0] * y[0] + y[1] * y[1] - 1.0).abs() < 1e-5, "{y:?}");
            assert!((y[0] - libm::cos(*t)).abs() < 1e-5);
        }
    }

    #[test]
    fn robertson() {
        let rhs = |_t: f64, y: &[f64]| {
            let a = -0.04 * y[0] + 1e4 * y[1] * y[2];
            let b = 3e7 * y[1] * y[1];
            Ok(vec![a, -a - b, b])
        };
        let r = integrate(&problem(&rhs, vec![1.0, 0.0, 0.0], 1.0)).unwrap();
        assert_eq!(r.end_time, 1.0);
        for y in r.states.iter().skip(1) {
            assert!(
                y.iter().all(|v| v.is_finite() && *v > 0.0 && *v <= 1.0),
                "{y:?}"
            );
            assert!((y.iter().sum::<f64>() - 1.0).abs() < 1e-8, "{y:?}");
        }
        assert!((r.states.last().unwrap()[0] - 0.9664597373).abs() < 1e-6);
    }

    #[test]
    fn van_der_pol() {
        let rhs = |_t: f64, y: &[f64]| Ok(vec![y[1], 100.0 * (1.0 - y[0] * y[0]) * y[1] - y[0]]);
        let r = integrate(&problem(&rhs, vec![2.0, 0.0], 20.0)).unwrap();
        assert_eq!(r.end_time, 20.0);
        assert!(r.states.iter().flatten().all(|v| v.is_finite()));
    }

    #[test]
    fn fifth_order_and_stage_time_dependence() {
        // A single-step local error is O(h^6). Loose acceptance tolerances
        // leave step sizes under test control; linear Newton still solves fully.
        let rhs = |t: f64, y: &[f64]| Ok(vec![-y[0] + t]);
        let mut p = problem(&rhs, vec![1.0], 1.0);
        p.rtol = 1.0;
        let error = |h| {
            let r = RadauMethod.step(&rhs, 0.0, &[1.0], &[-1.0], h, &p).unwrap();
            assert!(r.accepted);
            (r.y_new.unwrap()[0] - (h - 1.0 + 2.0 * libm::exp(-h))).abs()
        };
        let ratio = error(0.2) / error(0.1);
        assert!(ratio > 50.0 && ratio < 75.0, "local error ratio {ratio}");
    }

    #[test]
    fn embedded_order_and_stiff_decay() {
        let rhs = |_t: f64, y: &[f64]| Ok(vec![-y[0]]);
        let mut p = problem(&rhs, vec![1.0], 1.0);
        // In the unclamped controller, inferred error scales as h^4.
        p.rtol = 1e-8;
        let estimate = |h| {
            let r = RadauMethod.step(&rhs, 0.0, &[1.0], &[-1.0], h, &p).unwrap();
            let scale = r.h_next / h;
            assert!(scale > 0.2 && scale < 4.0);
            libm::pow(0.9 / scale, 4.0)
        };
        let ratio = estimate(0.04) / estimate(0.02);
        assert!(ratio > 14.0 && ratio < 18.0, "embedded error ratio {ratio}");
        let stiff = |_t: f64, y: &[f64]| Ok(vec![-1e5 * y[0]]);
        p.rtol = 2.0;
        let r = RadauMethod
            .step(&stiff, 0.0, &[1.0], &[-1e5], 1.0, &p)
            .unwrap();
        assert!(r.accepted);
        assert!(r.y_new.unwrap()[0].abs() < 1e-4);
        let failing = |_t: f64, _y: &[f64]| Err(crate::diag::FreesError::solver("RHS failure"));
        assert!(RadauMethod
            .step(&failing, 0.0, &[1.0], &[-1.0], 1.0, &p)
            .unwrap_err()
            .to_string()
            .contains("RHS failure"));
    }

    #[test]
    fn rejection_nonconvergence_and_step_cap() {
        let rhs = |_t: f64, y: &[f64]| Ok(vec![-y[0]]);
        let mut p = problem(&rhs, vec![1.0], 1.0);
        let r = RadauMethod
            .step(&rhs, 0.0, &[1.0], &[-1.0], 1.0, &p)
            .unwrap();
        assert!(!r.accepted && r.y_new.is_none() && r.f_new.is_none());
        assert!(r.h_next > 0.0 && r.h_next < 1.0);
        p.max_step = Some(0.002);
        let r = RadauMethod
            .step(&rhs, 0.0, &[1.0], &[-1.0], 0.001, &p)
            .unwrap();
        assert!(r.accepted);
        assert_eq!(r.h_next, 0.002);
        assert_eq!(r.f_new.unwrap()[0], -r.y_new.unwrap()[0]);
        // No real collocation solution for this large step; must reject.
        let nonlinear = |_t: f64, y: &[f64]| Ok(vec![y[0] * y[0]]);
        let r = RadauMethod
            .step(&nonlinear, 0.0, &[1.0], &[1.0], 10.0, &p)
            .unwrap();
        assert!(!r.accepted);
        assert_eq!(r.h_next, 2.0);
        let divergent = |_t: f64, _y: &[f64]| Ok(vec![f64::NAN]);
        assert!(
            !RadauMethod
                .step(&divergent, 0.0, &[1.0], &[1.0], 1.0, &p)
                .unwrap()
                .accepted
        );
    }
}
