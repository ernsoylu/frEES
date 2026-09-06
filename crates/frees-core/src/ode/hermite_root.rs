//! Analytic zero of the cubic Hermite dense output.
//!
//! [`integrator::hermite`](super::integrator::hermite) interpolates one state
//! component as a cubic in `θ ∈ [0, 1]`, so `y_j(θ) = level` is a cubic
//! equation and its roots are a closed form — no bisection, and (like the
//! bisection it replaces) no right-hand-side evaluations at all.
//!
//! The event surface uses this only when the switching function is a residual
//! of a single state, `g = ±(y[j] − level)`, which is what `EVENT x = v`
//! compiles to; a general nonlinear `g` still bisects, because two knot values
//! do not determine a cubic in `g`.
//!
//! Transcendentals go through `libm` so wasm32 and native agree bit for bit,
//! the same rule the rest of `frees-core` follows.

// A knot pair is eight scalars; the sibling `integrator` module carries the
// same allow for the same reason.
#![allow(clippy::too_many_arguments)]

/// Cubic coefficients `[a, b, c, d]` of `H(θ) − level` for one component,
/// expanded from the Hermite basis
/// `h00 y0 + h10 dt f0 + h01 y1 + h11 dt f1`.
fn coefficients(y0: f64, f0: f64, y1: f64, f1: f64, dt: f64, level: f64) -> [f64; 4] {
    let d0 = dt * f0;
    let d1 = dt * f1;
    [
        2.0 * y0 + d0 - 2.0 * y1 + d1,
        -3.0 * y0 - 2.0 * d0 + 3.0 * y1 - d1,
        d0,
        y0 - level,
    ]
}

fn poly(a: f64, b: f64, c: f64, d: f64, x: f64) -> f64 {
    ((a * x + b) * x + c) * x + d
}

fn slope(a: f64, b: f64, c: f64, x: f64) -> f64 {
    (3.0 * a * x + 2.0 * b) * x + c
}

/// Earliest `θ ∈ (0, 1]` where the component's Hermite interpolant reaches
/// `level` in the requested direction, or `None` when there is no such root.
///
/// `sign` is the orientation of the switching function against the state
/// (`g = sign · (y[j] − level)`), so `direction` — `+1` rising, `−1` falling,
/// `0` any — is applied to `sign · H'(θ)`, not to `H'` itself. `None` is always
/// a safe answer: the caller falls back to bisection.
pub(crate) fn cubic_hermite_root(
    y0: f64,
    f0: f64,
    y1: f64,
    f1: f64,
    dt: f64,
    level: f64,
    sign: f64,
    direction: i32,
) -> Option<f64> {
    if dt == 0.0 || !dt.is_finite() {
        return None;
    }
    let [a, b, c, d] = coefficients(y0, f0, y1, f1, dt, level);
    if ![a, b, c, d].iter().all(|v| v.is_finite()) {
        return None;
    }
    let mut buf = [0.0f64; 3];
    let n = real_roots(a, b, c, d, &mut buf);
    let mut best: Option<f64> = None;
    for &raw in buf.iter().take(n) {
        let theta = polish(a, b, c, d, raw);
        if !theta.is_finite() || theta <= 0.0 || theta > 1.0 {
            continue;
        }
        let dir = sign * slope(a, b, c, theta);
        let matches = match direction {
            1 => dir > 0.0,
            -1 => dir < 0.0,
            _ => true,
        };
        if matches && best.is_none_or(|t| theta < t) {
            best = Some(theta);
        }
    }
    best
}

/// Real roots of `a x³ + b x² + c x + d`, written into `out`; returns how many.
fn real_roots(a: f64, b: f64, c: f64, d: f64, out: &mut [f64; 3]) -> usize {
    let scale = a.abs().max(b.abs()).max(c.abs()).max(d.abs());
    if scale == 0.0 {
        return 0;
    }
    // A leading coefficient below the rounding noise of the others cannot move
    // a root inside [0, 1] (there |a x³| ≤ |a|), and Cardano's b/(3a) shift
    // would be catastrophic. Solve the quadratic instead; the third root has
    // magnitude ~|b/a| and is nowhere near the unit interval.
    if a.abs() <= 1e-14 * scale {
        return quadratic_roots(b, c, d, out);
    }

    // Depressed cubic t³ + p t + q, with x = t − b/(3a).
    let b1 = b / a;
    let c1 = c / a;
    let d1 = d / a;
    let shift = b1 / 3.0;
    let p = c1 - b1 * shift;
    let q = 2.0 * shift * shift * shift - shift * c1 + d1;

    let disc = 0.25 * q * q + p * p * p / 27.0;
    if disc > 0.0 {
        let s = disc.sqrt();
        let t = libm::cbrt(-0.5 * q + s) + libm::cbrt(-0.5 * q - s);
        out[0] = t - shift;
        1
    } else if p == 0.0 {
        out[0] = libm::cbrt(-q) - shift;
        1
    } else {
        // Three real roots (disc ≤ 0 forces p < 0): the trigonometric form,
        // which needs no complex arithmetic and stays conditioned.
        let r = 2.0 * (-p / 3.0).sqrt();
        let arg = (-4.0 * q / (r * r * r)).clamp(-1.0, 1.0);
        let phi = libm::acos(arg) / 3.0;
        let step = 2.0 * core::f64::consts::PI / 3.0;
        for (k, slot) in out.iter_mut().enumerate() {
            *slot = r * libm::cos(phi - step * k as f64) - shift;
        }
        3
    }
}

/// Real roots of `b x² + c x + d` (degrading to the linear case), written into
/// `out`; returns how many. Uses the cancellation-free pairing.
fn quadratic_roots(b: f64, c: f64, d: f64, out: &mut [f64; 3]) -> usize {
    if b == 0.0 {
        if c == 0.0 {
            return 0;
        }
        out[0] = -d / c;
        return 1;
    }
    let disc = c * c - 4.0 * b * d;
    if disc < 0.0 {
        return 0;
    }
    let s = disc.sqrt();
    let q = -0.5 * (c + if c >= 0.0 { s } else { -s });
    out[0] = q / b;
    if q == 0.0 {
        return 1;
    }
    out[1] = d / q;
    2
}

/// A few Newton steps on the cubic itself, keeping the iterate with the
/// smallest residual. Cardano and the trigonometric form both lose digits when
/// the cubic is nearly degenerate; this costs four multiply-adds and puts the
/// root back at machine precision, which is what the oracle crossing time —
/// the bisection's `dt/2^60` answer — demands.
fn polish(a: f64, b: f64, c: f64, d: f64, x0: f64) -> f64 {
    let mut x = x0;
    let mut best = x0;
    let mut best_r = poly(a, b, c, d, x0).abs();
    for _ in 0..6 {
        let fx = poly(a, b, c, d, x);
        let dfx = slope(a, b, c, x);
        if dfx == 0.0 {
            break;
        }
        let nx = x - fx / dfx;
        if !nx.is_finite() {
            break;
        }
        let r = poly(a, b, c, d, nx).abs();
        if r < best_r {
            best_r = r;
            best = nx;
        }
        x = nx;
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Hermite knots of a manufactured cubic `p` on `[t0, t1]`.
    fn knots(p: impl Fn(f64) -> f64, dp: impl Fn(f64) -> f64, t0: f64, t1: f64) -> [f64; 5] {
        [p(t0), dp(t0), p(t1), dp(t1), t1 - t0]
    }

    #[test]
    fn recovers_a_manufactured_cubics_unique_root() {
        // p(x) = (x − 0.375)(x² + 2) on [0, 1] scaled to θ: one real root.
        let p = |x: f64| (x - 0.375) * (x * x + 2.0);
        let dp = |x: f64| (x * x + 2.0) + (x - 0.375) * 2.0 * x;
        let [y0, f0, y1, f1, dt] = knots(p, dp, 0.0, 1.0);
        let theta = cubic_hermite_root(y0, f0, y1, f1, dt, 0.0, 1.0, 0).unwrap();
        assert!((theta - 0.375).abs() < 1e-12, "got {theta}");
    }

    #[test]
    fn recovers_a_root_of_a_three_real_root_cubic() {
        // (x − 0.2)(x − 0.6)(x − 1.4): two roots inside (0, 1], earliest wins.
        let p = |x: f64| (x - 0.2) * (x - 0.6) * (x - 1.4);
        let dp = |x: f64| (x - 0.6) * (x - 1.4) + (x - 0.2) * (x - 1.4) + (x - 0.2) * (x - 0.6);
        let [y0, f0, y1, f1, dt] = knots(p, dp, 0.0, 1.0);
        let theta = cubic_hermite_root(y0, f0, y1, f1, dt, 0.0, 1.0, 0).unwrap();
        assert!((theta - 0.2).abs() < 1e-12, "got {theta}");
    }

    #[test]
    fn the_direction_filter_skips_the_wrong_crossing() {
        // Same cubic: 0.2 is a rising crossing, 0.6 a falling one.
        let p = |x: f64| (x - 0.2) * (x - 0.6) * (x - 1.4);
        let dp = |x: f64| (x - 0.6) * (x - 1.4) + (x - 0.2) * (x - 1.4) + (x - 0.2) * (x - 0.6);
        let [y0, f0, y1, f1, dt] = knots(p, dp, 0.0, 1.0);
        let rising = cubic_hermite_root(y0, f0, y1, f1, dt, 0.0, 1.0, 1).unwrap();
        assert!((rising - 0.2).abs() < 1e-12, "rising got {rising}");
        let falling = cubic_hermite_root(y0, f0, y1, f1, dt, 0.0, 1.0, -1).unwrap();
        assert!((falling - 0.6).abs() < 1e-12, "falling got {falling}");
        // A negatively-oriented switching function swaps them.
        let flipped = cubic_hermite_root(y0, f0, y1, f1, dt, 0.0, -1.0, 1).unwrap();
        assert!((flipped - 0.6).abs() < 1e-12, "flipped got {flipped}");
    }

    #[test]
    fn a_near_linear_interpolant_still_roots() {
        // Leading coefficient in the rounding noise: the quadratic degradation.
        let theta = cubic_hermite_root(1.0, -2.0, -1.0, -2.0, 1.0, 0.0, 1.0, -1).unwrap();
        assert!((theta - 0.5).abs() < 1e-12, "got {theta}");
    }

    #[test]
    fn a_root_at_the_left_knot_is_not_a_crossing() {
        // y0 == level exactly: the root at θ = 0 is excluded, the interpolant
        // never returns, so there is nothing on (0, 1].
        assert_eq!(
            cubic_hermite_root(0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 0),
            None
        );
    }

    #[test]
    fn the_right_knot_is_included() {
        let theta = cubic_hermite_root(-1.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, 0).unwrap();
        assert!((theta - 1.0).abs() < 1e-12, "got {theta}");
    }

    #[test]
    fn a_degenerate_interval_or_a_flat_component_declines() {
        assert_eq!(
            cubic_hermite_root(1.0, 0.0, 2.0, 0.0, 0.0, 1.5, 1.0, 0),
            None
        );
        assert_eq!(
            cubic_hermite_root(5.0, 0.0, 5.0, 0.0, 1.0, 5.0, 1.0, 0),
            None
        );
        assert_eq!(
            cubic_hermite_root(f64::NAN, 0.0, 1.0, 0.0, 1.0, 0.5, 1.0, 0),
            None
        );
    }
}
