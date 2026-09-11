//! Input uncertainty distributions (Phase 4.3).
//!
//! The engine's original uncertainty model had exactly one shape: an
//! independent normal per source, drawn and then **clamped** into the declared
//! bounds. Clamping is not truncation — it piles probability mass onto the
//! bound itself, so a `[0, ∞)` flow rate declared `1 ± 2` came back with a
//! visible spike at zero. This module supplies the marginals a measurement
//! model actually needs, and draws every one of them by **inverse CDF on the
//! truncated interval**, which puts no mass on the bounds at all.
//!
//! # Declaring one
//!
//! `DistributionOf(X) = Uniform(0.9, 1.1)` is lifted out of the equation system
//! the same way `UncertaintyOf(X) = 0.05` is — see
//! [`super::uncertainty::extract_uncertainty_equations`]. A source with only an
//! `UncertaintyOf` declaration keeps the historical meaning exactly: a normal
//! centred on the base solve with that standard deviation.
//!
//! # What is deliberately not here
//!
//! Correlation between **non-Gaussian** marginals. Imposing a rank correlation
//! on arbitrary marginals (an Iman–Conover or Gaussian-copula construction)
//! changes the marginals' realised shape in ways that need stating, and a wrong
//! answer here is invisible. [`super::sampling`] rejects that combination
//! rather than silently ignoring the correlation.

use crate::diag::{FreesError, Result};

/// A marginal input distribution.
///
/// Every variant carries the parameters in the order the document writes them.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Distribution {
    /// `Normal(mean, sigma)` — sigma is the standard deviation, not a variance.
    Normal { mean: f64, sigma: f64 },
    /// `Uniform(lo, hi)`.
    Uniform { lo: f64, hi: f64 },
    /// `Triangular(lo, mode, hi)`.
    Triangular { lo: f64, mode: f64, hi: f64 },
    /// `LogNormal(logMean, logSigma)` — the parameters of `ln(X)`, which is the
    /// convention SciPy, NumPy and every reliability text use. The distribution's
    /// own mean is `exp(logMean + logSigma²/2)`, not `logMean`.
    LogNormal { log_mean: f64, log_sigma: f64 },
    /// `Weibull(shape, scale)` — the two-parameter form, support `[0, ∞)`.
    Weibull { shape: f64, scale: f64 },
    /// `Beta(alpha, beta)` on `[0, 1]`, or `Beta(alpha, beta, lo, hi)` rescaled
    /// onto `[lo, hi]`.
    Beta {
        alpha: f64,
        beta: f64,
        lo: f64,
        hi: f64,
    },
}

/// The names `DistributionOf(X) = …` accepts, lowercase, with their arities.
/// Kept as one table so the parser, the error message and the docs cannot
/// disagree about what exists.
pub const DISTRIBUTION_FORMS: &[(&str, &str)] = &[
    ("normal", "Normal(mean, sigma)"),
    ("uniform", "Uniform(lo, hi)"),
    ("triangular", "Triangular(lo, mode, hi)"),
    ("lognormal", "LogNormal(logMean, logSigma)"),
    ("weibull", "Weibull(shape, scale)"),
    ("beta", "Beta(alpha, beta) or Beta(alpha, beta, lo, hi)"),
];

impl Distribution {
    /// Build one from a `DistributionOf(X) = Name(args…)` right-hand side.
    ///
    /// # Errors
    ///
    /// [`FreesError::Solver`] for an unknown name, the wrong number of
    /// arguments, or a parameter outside the distribution's domain.
    pub fn from_call(name: &str, args: &[f64]) -> Result<Distribution> {
        let lower = name.to_ascii_lowercase();
        let wrong_arity = |wanted: &str| {
            Err(FreesError::solver(format!(
                "{name} takes {wanted}, got {} argument(s).",
                args.len()
            )))
        };
        let built = match (lower.as_str(), args.len()) {
            ("normal" | "gaussian", 2) => Distribution::Normal {
                mean: args[0],
                sigma: args[1],
            },
            ("normal" | "gaussian", _) => return wrong_arity("(mean, sigma)"),
            ("uniform", 2) => Distribution::Uniform {
                lo: args[0],
                hi: args[1],
            },
            ("uniform", _) => return wrong_arity("(lo, hi)"),
            ("triangular", 3) => Distribution::Triangular {
                lo: args[0],
                mode: args[1],
                hi: args[2],
            },
            ("triangular", _) => return wrong_arity("(lo, mode, hi)"),
            ("lognormal", 2) => Distribution::LogNormal {
                log_mean: args[0],
                log_sigma: args[1],
            },
            ("lognormal", _) => return wrong_arity("(logMean, logSigma)"),
            ("weibull", 2) => Distribution::Weibull {
                shape: args[0],
                scale: args[1],
            },
            ("weibull", _) => return wrong_arity("(shape, scale)"),
            ("beta", 2) => Distribution::Beta {
                alpha: args[0],
                beta: args[1],
                lo: 0.0,
                hi: 1.0,
            },
            ("beta", 4) => Distribution::Beta {
                alpha: args[0],
                beta: args[1],
                lo: args[2],
                hi: args[3],
            },
            ("beta", _) => return wrong_arity("(alpha, beta) or (alpha, beta, lo, hi)"),
            _ => {
                let known: Vec<&str> = DISTRIBUTION_FORMS.iter().map(|(_, form)| *form).collect();
                return Err(FreesError::solver(format!(
                    "Unknown distribution `{name}`. Available: {}.",
                    known.join(", ")
                )));
            }
        };
        built.validate()?;
        Ok(built)
    }

    /// Rejects parameters the distribution has no meaning for.
    ///
    /// # Errors
    ///
    /// [`FreesError::Solver`] naming the offending parameter.
    pub fn validate(&self) -> Result<()> {
        let bad = |what: &str| Err(FreesError::solver(what.to_string()));
        if !self.parameters().iter().all(|v| v.is_finite()) {
            return bad("Distribution parameters must all be finite.");
        }
        match *self {
            Distribution::Normal { sigma, .. } => {
                if sigma <= 0.0 {
                    return bad("Normal requires sigma > 0.");
                }
            }
            Distribution::Uniform { lo, hi } => {
                if hi <= lo {
                    return bad("Uniform requires hi > lo.");
                }
            }
            Distribution::Triangular { lo, mode, hi } => {
                if hi <= lo {
                    return bad("Triangular requires hi > lo.");
                }
                if mode < lo || mode > hi {
                    return bad("Triangular requires lo <= mode <= hi.");
                }
            }
            Distribution::LogNormal { log_sigma, .. } => {
                if log_sigma <= 0.0 {
                    return bad("LogNormal requires logSigma > 0.");
                }
            }
            Distribution::Weibull { shape, scale } => {
                if shape <= 0.0 || scale <= 0.0 {
                    return bad("Weibull requires shape > 0 and scale > 0.");
                }
            }
            Distribution::Beta {
                alpha,
                beta,
                lo,
                hi,
                ..
            } => {
                if alpha <= 0.0 || beta <= 0.0 {
                    return bad("Beta requires alpha > 0 and beta > 0.");
                }
                if hi <= lo {
                    return bad("Beta requires hi > lo.");
                }
            }
        }
        Ok(())
    }

    fn parameters(&self) -> Vec<f64> {
        match *self {
            Distribution::Normal { mean, sigma } => vec![mean, sigma],
            Distribution::Uniform { lo, hi } => vec![lo, hi],
            Distribution::Triangular { lo, mode, hi } => vec![lo, mode, hi],
            Distribution::LogNormal {
                log_mean,
                log_sigma,
            } => vec![log_mean, log_sigma],
            Distribution::Weibull { shape, scale } => vec![shape, scale],
            Distribution::Beta {
                alpha,
                beta,
                lo,
                hi,
            } => vec![alpha, beta, lo, hi],
        }
    }

    /// True only for the normal — the one marginal correlated sampling is
    /// defined for here.
    #[must_use]
    pub fn is_gaussian(&self) -> bool {
        matches!(self, Distribution::Normal { .. })
    }

    /// The distribution's own name, for diagnostics.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Distribution::Normal { .. } => "Normal",
            Distribution::Uniform { .. } => "Uniform",
            Distribution::Triangular { .. } => "Triangular",
            Distribution::LogNormal { .. } => "LogNormal",
            Distribution::Weibull { .. } => "Weibull",
            Distribution::Beta { .. } => "Beta",
        }
    }

    /// Population mean.
    #[must_use]
    pub fn mean(&self) -> f64 {
        match *self {
            Distribution::Normal { mean, .. } => mean,
            Distribution::Uniform { lo, hi } => 0.5 * (lo + hi),
            Distribution::Triangular { lo, mode, hi } => (lo + mode + hi) / 3.0,
            Distribution::LogNormal {
                log_mean,
                log_sigma,
            } => libm::exp(log_mean + 0.5 * log_sigma * log_sigma),
            Distribution::Weibull { shape, scale } => scale * libm::tgamma(1.0 + 1.0 / shape),
            Distribution::Beta {
                alpha,
                beta,
                lo,
                hi,
            } => lo + (hi - lo) * alpha / (alpha + beta),
        }
    }

    /// Population standard deviation — the number a first-order propagation
    /// consumes when a source declares a shape instead of a bare `±`.
    #[must_use]
    pub fn std_dev(&self) -> f64 {
        match *self {
            Distribution::Normal { sigma, .. } => sigma,
            Distribution::Uniform { lo, hi } => (hi - lo) / libm::sqrt(12.0),
            Distribution::Triangular { lo, mode, hi } => {
                let v = (lo * lo + mode * mode + hi * hi - lo * mode - lo * hi - mode * hi) / 18.0;
                libm::sqrt(v.max(0.0))
            }
            Distribution::LogNormal {
                log_mean,
                log_sigma,
            } => {
                let s2 = log_sigma * log_sigma;
                libm::sqrt((libm::exp(s2) - 1.0).max(0.0)) * libm::exp(log_mean + 0.5 * s2)
            }
            Distribution::Weibull { shape, scale } => {
                let g1 = libm::tgamma(1.0 + 1.0 / shape);
                let g2 = libm::tgamma(1.0 + 2.0 / shape);
                scale * libm::sqrt((g2 - g1 * g1).max(0.0))
            }
            Distribution::Beta {
                alpha,
                beta,
                lo,
                hi,
            } => {
                let s = alpha + beta;
                (hi - lo) * libm::sqrt(alpha * beta / (s * s * (s + 1.0)))
            }
        }
    }

    /// Cumulative distribution function.
    #[must_use]
    pub fn cdf(&self, x: f64) -> f64 {
        match *self {
            Distribution::Normal { mean, sigma } => normal_cdf((x - mean) / sigma),
            Distribution::Uniform { lo, hi } => ((x - lo) / (hi - lo)).clamp(0.0, 1.0),
            Distribution::Triangular { lo, mode, hi } => {
                if x <= lo {
                    0.0
                } else if x >= hi {
                    1.0
                } else if x < mode {
                    (x - lo) * (x - lo) / ((hi - lo) * (mode - lo))
                } else if x > mode {
                    1.0 - (hi - x) * (hi - x) / ((hi - lo) * (hi - mode))
                } else {
                    (mode - lo) / (hi - lo)
                }
            }
            Distribution::LogNormal {
                log_mean,
                log_sigma,
            } => {
                if x <= 0.0 {
                    0.0
                } else {
                    normal_cdf((libm::log(x) - log_mean) / log_sigma)
                }
            }
            Distribution::Weibull { shape, scale } => {
                if x <= 0.0 {
                    0.0
                } else {
                    -libm::expm1(-libm::pow(x / scale, shape))
                }
            }
            Distribution::Beta {
                alpha,
                beta,
                lo,
                hi,
            } => {
                let t = ((x - lo) / (hi - lo)).clamp(0.0, 1.0);
                crate::eval::regularized_beta(t, alpha, beta).unwrap_or(f64::NAN)
            }
        }
    }

    /// Inverse CDF. `u` outside `(0, 1)` saturates at the support's ends.
    #[must_use]
    pub fn quantile(&self, u: f64) -> f64 {
        if !(0.0..=1.0).contains(&u) {
            return f64::NAN;
        }
        match *self {
            Distribution::Normal { mean, sigma } => mean + sigma * normal_quantile(u),
            Distribution::Uniform { lo, hi } => lo + u * (hi - lo),
            Distribution::Triangular { lo, mode, hi } => {
                let split = (mode - lo) / (hi - lo);
                if u < split {
                    lo + libm::sqrt(u * (hi - lo) * (mode - lo))
                } else {
                    hi - libm::sqrt((1.0 - u) * (hi - lo) * (hi - mode))
                }
            }
            Distribution::LogNormal {
                log_mean,
                log_sigma,
            } => libm::exp(log_mean + log_sigma * normal_quantile(u)),
            Distribution::Weibull { shape, scale } => {
                // `-log1p(-u)` rather than `-ln(1 - u)`: for the small `u` a
                // low-discrepancy sequence deliberately produces, the naive form
                // loses every bit of the tail.
                scale * libm::pow(-libm::log1p(-u), 1.0 / shape)
            }
            Distribution::Beta {
                alpha,
                beta,
                lo,
                hi,
            } => lo + (hi - lo) * inverse_regularized_beta(u, alpha, beta),
        }
    }

    /// Draw the value at uniform deviate `u ∈ (0, 1)`, **truncated** to
    /// `[lower, upper]`.
    ///
    /// This is inverse-CDF sampling on the conditional distribution: the
    /// deviate is mapped into `[F(lower), F(upper)]` before inversion, so the
    /// result is a genuine draw from the truncated law with no mass piled on
    /// either bound and no rejection loop to make the sample count
    /// unpredictable. Infinite bounds simply leave that end at 0 or 1.
    ///
    /// # Errors
    ///
    /// [`FreesError::Solver`] when the bounds exclude the distribution's entire
    /// support, which would otherwise return a silent `NaN`.
    pub fn sample_truncated(&self, u: f64, lower: f64, upper: f64) -> Result<f64> {
        let a = if lower.is_finite() {
            self.cdf(lower)
        } else {
            0.0
        };
        let b = if upper.is_finite() {
            self.cdf(upper)
        } else {
            1.0
        };
        // A NaN from either CDF means the bound is nowhere on the support.
        if a.is_nan() || b.is_nan() || b <= a {
            return Err(FreesError::solver(format!(
                "The bounds [{lower}, {upper}] exclude the whole support of {}.",
                self.name()
            )));
        }
        let value = self.quantile(a + u * (b - a));
        // Inversion is exact in exact arithmetic; in floating point the very
        // ends can land an ulp outside. Clamping there is a rounding fix, not
        // the old clamp-the-draw behaviour this method exists to replace.
        Ok(value.clamp(lower, upper))
    }
}

/// Standard normal CDF.
#[must_use]
pub fn normal_cdf(z: f64) -> f64 {
    0.5 * libm::erfc(-z / std::f64::consts::SQRT_2)
}

/// Standard normal quantile (probit).
#[must_use]
pub fn normal_quantile(u: f64) -> f64 {
    std::f64::consts::SQRT_2 * crate::eval::erf_inv(2.0 * u - 1.0)
}

/// `I⁻¹(u; a, b)` by bisection on the regularized incomplete beta.
///
/// Bisection rather than Newton on purpose: `I` is already the expensive part,
/// its derivative near `a < 1` or `b < 1` is unbounded, and 80 halvings of
/// `[0, 1]` reach the last representable bit of the answer deterministically —
/// which matters more here than iteration count, because a seeded run must
/// reproduce.
fn inverse_regularized_beta(u: f64, a: f64, b: f64) -> f64 {
    if u <= 0.0 {
        return 0.0;
    }
    if u >= 1.0 {
        return 1.0;
    }
    let (mut lo, mut hi) = (0.0f64, 1.0f64);
    for _ in 0..80 {
        let mid = 0.5 * (lo + hi);
        if mid <= lo || mid >= hi {
            break;
        }
        match crate::eval::regularized_beta(mid, a, b) {
            Ok(p) if p < u => lo = mid,
            Ok(_) => hi = mid,
            Err(_) => return f64::NAN,
        }
    }
    0.5 * (lo + hi)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64, tol: f64, what: &str) {
        assert!((a - b).abs() <= tol, "{what}: got {a}, wanted {b}");
    }

    /// The one property every distribution must have: `quantile(cdf(x)) == x`
    /// on the interior of its support. A wrong parameterization, a wrong branch
    /// in a piecewise CDF and a mis-scaled support all break exactly this.
    #[test]
    fn cdf_and_quantile_are_inverse_on_every_shape() {
        let shapes = [
            Distribution::Normal {
                mean: 3.0,
                sigma: 1.5,
            },
            Distribution::Uniform { lo: -2.0, hi: 7.0 },
            Distribution::Triangular {
                lo: 0.0,
                mode: 1.0,
                hi: 5.0,
            },
            Distribution::LogNormal {
                log_mean: 0.4,
                log_sigma: 0.6,
            },
            Distribution::Weibull {
                shape: 2.3,
                scale: 4.0,
            },
            Distribution::Beta {
                alpha: 2.0,
                beta: 5.0,
                lo: 1.0,
                hi: 3.0,
            },
        ];
        for d in shapes {
            for step in 1..20 {
                let u = step as f64 / 20.0;
                let x = d.quantile(u);
                let back = d.cdf(x);
                close(back, u, 1e-8, &format!("{} round trip at u={u}", d.name()));
            }
        }
    }

    /// Closed-form moments, checked against the textbook formulas the code is
    /// not allowed to merely restate: these are compared to a fine quadrature
    /// of the quantile function, which shares no algebra with `mean`/`std_dev`.
    #[test]
    fn moments_match_a_numerical_integration_of_the_quantile() {
        let shapes = [
            Distribution::Uniform { lo: -2.0, hi: 7.0 },
            Distribution::Triangular {
                lo: 0.0,
                mode: 1.0,
                hi: 5.0,
            },
            Distribution::LogNormal {
                log_mean: 0.4,
                log_sigma: 0.6,
            },
            Distribution::Weibull {
                shape: 2.3,
                scale: 4.0,
            },
            Distribution::Beta {
                alpha: 2.0,
                beta: 5.0,
                lo: 1.0,
                hi: 3.0,
            },
        ];
        for d in shapes {
            let n = 200_000;
            let (mut m1, mut m2) = (0.0, 0.0);
            for k in 0..n {
                let u = (k as f64 + 0.5) / n as f64;
                let x = d.quantile(u);
                m1 += x;
                m2 += x * x;
            }
            m1 /= n as f64;
            m2 /= n as f64;
            let sd = libm::sqrt((m2 - m1 * m1).max(0.0));
            let scale = d.std_dev().max(1e-9);
            close(
                m1,
                d.mean(),
                2e-3 * scale.max(1.0),
                &format!("{} mean", d.name()),
            );
            close(
                sd,
                d.std_dev(),
                3e-3 * scale.max(1.0),
                &format!("{} sd", d.name()),
            );
        }
    }

    #[test]
    fn truncation_puts_no_mass_on_the_bounds() {
        let d = Distribution::Normal {
            mean: 0.0,
            sigma: 1.0,
        };
        // Truncated to [0, ∞): the median of the truncation is the 75th
        // percentile of the parent, 0.6744897501960817.
        let mid = d.sample_truncated(0.5, 0.0, f64::INFINITY).unwrap();
        close(mid, 0.674_489_750_196_081_7, 1e-9, "truncated median");
        // Every interior deviate lands strictly inside, unlike the clamped
        // sampler this replaces, which piled everything below 0 onto 0.
        for step in 1..50 {
            let u = step as f64 / 50.0;
            let v = d.sample_truncated(u, -1.0, 1.0).unwrap();
            assert!(v > -1.0 && v < 1.0, "u={u} gave {v}");
        }
        // Bounds that exclude the support are an error, not a NaN.
        let weibull = Distribution::Weibull {
            shape: 2.0,
            scale: 1.0,
        };
        assert!(weibull.sample_truncated(0.5, -5.0, -1.0).is_err());
    }

    #[test]
    fn parsing_rejects_bad_names_arities_and_parameters() {
        assert_eq!(
            Distribution::from_call("Uniform", &[0.0, 1.0]).unwrap(),
            Distribution::Uniform { lo: 0.0, hi: 1.0 }
        );
        // Beta takes two or four, never three.
        assert!(Distribution::from_call("Beta", &[2.0, 5.0]).is_ok());
        assert!(Distribution::from_call("Beta", &[2.0, 5.0, 0.0, 1.0]).is_ok());
        let msg = Distribution::from_call("Beta", &[2.0, 5.0, 0.0])
            .unwrap_err()
            .to_string();
        assert!(msg.contains("alpha, beta"), "{msg}");
        let msg = Distribution::from_call("Cauchy", &[0.0, 1.0])
            .unwrap_err()
            .to_string();
        assert!(msg.contains("Unknown distribution"), "{msg}");
        for bad in [
            ("Normal", vec![0.0, 0.0]),
            ("Uniform", vec![1.0, 1.0]),
            ("Triangular", vec![0.0, 9.0, 5.0]),
            ("Weibull", vec![-1.0, 2.0]),
            ("Beta", vec![0.0, 5.0]),
            ("LogNormal", vec![0.0, f64::NAN]),
        ] {
            assert!(
                Distribution::from_call(bad.0, &bad.1).is_err(),
                "{} {:?} should be refused",
                bad.0,
                bad.1
            );
        }
    }

    #[test]
    fn only_the_normal_reports_itself_gaussian() {
        assert!(Distribution::Normal {
            mean: 0.0,
            sigma: 1.0
        }
        .is_gaussian());
        assert!(!Distribution::LogNormal {
            log_mean: 0.0,
            log_sigma: 1.0
        }
        .is_gaussian());
    }
}
