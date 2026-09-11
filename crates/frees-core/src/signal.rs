//! Discrete-signal kernels: `FFT` / `IFFT` / `Convolve`.
//!
//! Originally a port of `../frEES/backend/core/src/main/java/com/frees/backend/core/SignalProcessing.java`
//! (62 LOC), whose transform is a **direct DFT** (O(n²)). The direct kernel is
//! still here and is still what small transforms run — it is the exact base
//! case, and every frozen fixture goes through it unchanged — but lengths past
//! [`DIRECT_MAX`] now take an O(n log n) path: radix-2 Cooley–Tukey when the
//! length is a power of two, Bluestein's chirp-z otherwise. Bluestein keeps the
//! "any length, including primes" property the direct DFT had; there is no
//! power-of-two restriction at any size.
//!
//! All trigonometry goes through [`libm`] so native and wasm runs agree bit
//! for bit (the crate-wide determinism rule).

use crate::diag::{FreesError, Result};
use std::f64::consts::PI;

/// Lengths at or below this run the direct O(n²) DFT.
///
/// Two reasons, not one. It is the honest base case — at n = 32 the direct
/// transform is ~1k multiply-adds and beats the setup cost of either fast path
/// outright. And it is the *same arithmetic in the same order* the engine has
/// always run, so every golden fixture (all of which transform 4-point
/// sequences) keeps its bits exactly.
const DIRECT_MAX: usize = 32;

/// Discrete Fourier transform of the complex sequence `re + i·im`.
/// Returns `(out_re, out_im)`. When `inverse` is true, computes the inverse
/// transform (including the 1/n normalization).
///
/// # Errors
///
/// [`FreesError::Evaluation`] when the real and imaginary parts differ in
/// length.
pub fn dft(re: &[f64], im: &[f64], inverse: bool) -> Result<(Vec<f64>, Vec<f64>)> {
    let n = re.len();
    if im.len() != n {
        return Err(FreesError::evaluation(
            "FFT real and imaginary parts must have equal length.",
        ));
    }
    let sign = if inverse { 1.0 } else { -1.0 };

    let (mut out_re, mut out_im) = if n <= DIRECT_MAX {
        direct(re, im, sign)
    } else if n.is_power_of_two() {
        let mut r = re.to_vec();
        let mut i = im.to_vec();
        radix2(&mut r, &mut i, sign);
        (r, i)
    } else {
        bluestein(re, im, sign)
    };

    if inverse {
        let scale = 1.0 / n as f64;
        for k in 0..n {
            out_re[k] *= scale;
            out_im[k] *= scale;
        }
    }
    Ok((out_re, out_im))
}

/// The textbook O(n²) transform, unnormalized. `sign` is −1 forward, +1 inverse.
fn direct(re: &[f64], im: &[f64], sign: f64) -> (Vec<f64>, Vec<f64>) {
    let n = re.len();
    let mut out_re = vec![0.0; n];
    let mut out_im = vec![0.0; n];
    for k in 0..n {
        let mut sum_re = 0.0;
        let mut sum_im = 0.0;
        for j in 0..n {
            let angle = sign * 2.0 * PI * (j as f64) * (k as f64) / (n as f64);
            let cos = libm::cos(angle);
            let sin = libm::sin(angle);
            sum_re += re[j] * cos - im[j] * sin;
            sum_im += re[j] * sin + im[j] * cos;
        }
        out_re[k] = sum_re;
        out_im[k] = sum_im;
    }
    (out_re, out_im)
}

/// In-place iterative radix-2 Cooley–Tukey. Requires a power-of-two length and
/// applies **no** normalization; `sign` is −1 forward, +1 inverse.
///
/// Stage twiddles are computed once per stage straight from their angle (never
/// by a recurrence), so the values are the same ones the direct kernel would
/// use and do not drift across a long stage.
fn radix2(re: &mut [f64], im: &mut [f64], sign: f64) {
    let n = re.len();
    debug_assert!(n.is_power_of_two(), "radix2 needs a power-of-two length");
    if n <= 1 {
        return;
    }

    // Decimation in time: permute into bit-reversed order first.
    let mut j = 0usize;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j |= bit;
        if i < j {
            re.swap(i, j);
            im.swap(i, j);
        }
    }

    let mut len = 2usize;
    while len <= n {
        let half = len / 2;
        let twiddles: Vec<(f64, f64)> = (0..half)
            .map(|k| {
                let angle = sign * 2.0 * PI * (k as f64) / (len as f64);
                (libm::cos(angle), libm::sin(angle))
            })
            .collect();
        let mut start = 0usize;
        while start < n {
            for (k, &(wc, ws)) in twiddles.iter().enumerate() {
                let lo = start + k;
                let hi = lo + half;
                let tr = re[hi] * wc - im[hi] * ws;
                let ti = re[hi] * ws + im[hi] * wc;
                re[hi] = re[lo] - tr;
                im[hi] = im[lo] - ti;
                re[lo] += tr;
                im[lo] += ti;
            }
            start += len;
        }
        len <<= 1;
    }
}

/// Bluestein's chirp-z transform: any length, unnormalized, via three
/// power-of-two transforms of size `m ≥ 2n − 1`.
///
/// `X[k] = c[k] · Σⱼ (x[j]·c[j]) · conj(c[k−j])` with the chirp
/// `c[j] = exp(sign·iπj²/n)`, which turns the DFT into a linear convolution.
/// The squared index is reduced mod `2n` before it reaches the angle, so the
/// chirp stays accurate for large `n` instead of losing bits in `j²`.
fn bluestein(re: &[f64], im: &[f64], sign: f64) -> (Vec<f64>, Vec<f64>) {
    let n = re.len();
    let m = (2 * n - 1).next_power_of_two();

    let mut chirp_re = vec![0.0; n];
    let mut chirp_im = vec![0.0; n];
    let modulus = 2 * n as u128;
    for j in 0..n {
        let sq = ((j as u128 * j as u128) % modulus) as f64;
        let angle = sign * PI * sq / n as f64;
        chirp_re[j] = libm::cos(angle);
        chirp_im[j] = libm::sin(angle);
    }

    // a = x · chirp, zero-padded to m.
    let mut a_re = vec![0.0; m];
    let mut a_im = vec![0.0; m];
    for j in 0..n {
        a_re[j] = re[j] * chirp_re[j] - im[j] * chirp_im[j];
        a_im[j] = re[j] * chirp_im[j] + im[j] * chirp_re[j];
    }

    // b = conj(chirp), wrapped so index m−k carries the negative lag −k. The
    // cyclic convolution then reproduces the linear one because m ≥ 2n − 1.
    let mut b_re = vec![0.0; m];
    let mut b_im = vec![0.0; m];
    b_re[0] = chirp_re[0];
    b_im[0] = -chirp_im[0];
    for k in 1..n {
        b_re[k] = chirp_re[k];
        b_im[k] = -chirp_im[k];
        b_re[m - k] = chirp_re[k];
        b_im[m - k] = -chirp_im[k];
    }

    radix2(&mut a_re, &mut a_im, -1.0);
    radix2(&mut b_re, &mut b_im, -1.0);
    for i in 0..m {
        let pr = a_re[i] * b_re[i] - a_im[i] * b_im[i];
        let pi = a_re[i] * b_im[i] + a_im[i] * b_re[i];
        a_re[i] = pr;
        a_im[i] = pi;
    }
    radix2(&mut a_re, &mut a_im, 1.0);

    let inv = 1.0 / m as f64;
    let mut out_re = vec![0.0; n];
    let mut out_im = vec![0.0; n];
    for k in 0..n {
        let vr = a_re[k] * inv;
        let vi = a_im[k] * inv;
        out_re[k] = vr * chirp_re[k] - vi * chirp_im[k];
        out_im[k] = vr * chirp_im[k] + vi * chirp_re[k];
    }
    (out_re, out_im)
}

/// Linear convolution of `a` and `b` (length `a.len() + b.len() − 1`).
///
/// The Java computes `new double[m + n - 1]`, which for two empty inputs is a
/// negative array size (an unchecked crash); this port refuses empty input
/// with an explicit error instead.
///
/// # Errors
///
/// [`FreesError::Evaluation`] when either sequence is empty.
pub fn convolve(a: &[f64], b: &[f64]) -> Result<Vec<f64>> {
    let m = a.len();
    let n = b.len();
    if m == 0 || n == 0 {
        return Err(FreesError::evaluation(
            "Convolve requires two non-empty sequences.",
        ));
    }
    // ponytail: direct O(m·n) sum. The FFT route below would win past a few
    // thousand samples per side, but the evaluator's equation budget rejects
    // vectors long before that, so the crossover is unreachable from a document.
    let mut c = vec![0.0; m + n - 1];
    for (i, &ai) in a.iter().enumerate() {
        for (j, &bj) in b.iter().enumerate() {
            c[i + j] += ai * bj;
        }
    }
    Ok(c)
}

// ── Sensor signal processing (Phase 4.5) ────────────────────────────────────
//
// Conventions, stated once because every kernel below inherits them:
//
// * **Sample rate.** Uniform sampling is assumed throughout; `fs` is in Hz and
//   sample `j` sits at time `j / fs`. Irregularly sampled data must be
//   resampled onto a uniform raster first — none of these kernels inspect a
//   time column, and none of them will notice if one was uneven.
// * **Edges.** Causal filtering ([`lfilter`]) starts from rest, so the first
//   samples carry the filter's start-up transient. Zero-phase filtering
//   ([`filtfilt`]) odd-extends the signal before filtering, which suppresses
//   that transient but does not abolish it.
// * **Spectral scaling.** [`welch`] returns a one-sided **power spectral
//   density** in units of `x²/Hz`: window-power normalized, and doubled on
//   every bin except DC and (for an even segment) Nyquist, so that summing
//   `Pxx · Δf` recovers the signal's mean square.
// * **Indices.** Everything here is 0-based internally. The `CALL` layer is
//   what presents 1-based positions to a document.

/// A taper applied before a transform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Window {
    /// No taper — the implicit rectangular window.
    Rect,
    Hann,
    Hamming,
    Blackman,
    Bartlett,
}

impl Window {
    /// The name a document writes, lowercase.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "rect" | "rectangular" | "boxcar" | "none" => Some(Window::Rect),
            "hann" | "hanning" => Some(Window::Hann),
            "hamming" => Some(Window::Hamming),
            "blackman" => Some(Window::Blackman),
            "bartlett" | "triangular" => Some(Window::Bartlett),
            _ => None,
        }
    }
}

/// The `n` window weights.
///
/// `periodic` picks the denominator, and the choice is not cosmetic. A
/// **symmetric** window (`n − 1`) is the right taper to multiply a finite
/// record by — it is what MATLAB's `hann(n)` and SciPy's `get_window(...,
/// fftbins=False)` return, and what [`apply_window`] uses. A **periodic**
/// window (`n`) is the right taper for spectral estimation, where segments tile
/// a longer record — SciPy's default, and what [`welch`] uses internally.
#[must_use]
pub fn window_weights(kind: Window, n: usize, periodic: bool) -> Vec<f64> {
    if kind == Window::Rect {
        return vec![1.0; n];
    }
    if n == 0 {
        return Vec::new();
    }
    if n == 1 {
        return vec![1.0];
    }
    let denom = if periodic { n as f64 } else { (n - 1) as f64 };
    (0..n)
        .map(|j| {
            let t = j as f64 / denom;
            let two_pi_t = 2.0 * PI * t;
            match kind {
                Window::Rect => 1.0,
                Window::Hann => 0.5 - 0.5 * libm::cos(two_pi_t),
                Window::Hamming => 0.54 - 0.46 * libm::cos(two_pi_t),
                Window::Blackman => {
                    0.42 - 0.5 * libm::cos(two_pi_t) + 0.08 * libm::cos(2.0 * two_pi_t)
                }
                // Symmetric triangle peaking at the centre, zero at both ends.
                Window::Bartlett => 1.0 - libm::fabs(2.0 * t - 1.0),
            }
        })
        .collect()
}

/// `x` multiplied by the symmetric `kind` window of the same length.
#[must_use]
pub fn apply_window(kind: Window, x: &[f64]) -> Vec<f64> {
    let w = window_weights(kind, x.len(), false);
    x.iter().zip(&w).map(|(v, wj)| v * wj).collect()
}

/// Remove a constant (`linear == false`) or a least-squares straight line
/// (`linear == true`) from `y`, sampled on the index raster `0, 1, …, n−1`.
///
/// A linear detrend of fewer than two samples has no line to fit; it falls back
/// to the constant removal, which for one sample is exactly zero.
///
/// # Errors
///
/// [`FreesError::Evaluation`] on an empty sequence.
pub fn detrend(y: &[f64], linear: bool) -> Result<Vec<f64>> {
    let n = y.len();
    if n == 0 {
        return Err(FreesError::evaluation(
            "Detrend requires a non-empty series.",
        ));
    }
    let mean = y.iter().sum::<f64>() / n as f64;
    if !linear || n < 2 {
        return Ok(y.iter().map(|v| v - mean).collect());
    }
    // Closed-form OLS on t = 0..n−1: the design is known exactly, so there is
    // nothing to factorize and no conditioning to worry about.
    let t_mean = (n - 1) as f64 / 2.0;
    let mut sxy = 0.0;
    let mut sxx = 0.0;
    for (j, &v) in y.iter().enumerate() {
        let dt = j as f64 - t_mean;
        sxy += dt * (v - mean);
        sxx += dt * dt;
    }
    let slope = if sxx > 0.0 { sxy / sxx } else { 0.0 };
    Ok(y.iter()
        .enumerate()
        .map(|(j, &v)| v - (mean + slope * (j as f64 - t_mean)))
        .collect())
}

/// Centred moving average over an odd window of `k` samples.
///
/// The ends are averaged over the samples that exist rather than zero-padded,
/// so a constant series comes back constant instead of sagging at the edges.
///
/// # Errors
///
/// [`FreesError::Evaluation`] on an empty series, an even `k`, a zero `k`, or a
/// `k` longer than the series.
pub fn moving_average(x: &[f64], k: usize) -> Result<Vec<f64>> {
    let n = x.len();
    if n == 0 {
        return Err(FreesError::evaluation(
            "Smooth requires a non-empty series.",
        ));
    }
    if k == 0 || k % 2 == 0 {
        return Err(FreesError::evaluation(format!(
            "Smooth requires an odd window length (got {k})."
        )));
    }
    if k > n {
        return Err(FreesError::evaluation(format!(
            "Smooth window ({k}) is longer than the series ({n})."
        )));
    }
    let half = k / 2;
    Ok((0..n)
        .map(|j| {
            let lo = j.saturating_sub(half);
            let hi = (j + half + 1).min(n);
            x[lo..hi].iter().sum::<f64>() / (hi - lo) as f64
        })
        .collect())
}

/// Causal IIR/FIR filtering: `a[0]·y[j] = Σ b[i]·x[j−i] − Σ a[i]·y[j−i]`,
/// evaluated as a transposed direct-form II so only `max(len(a), len(b)) − 1`
/// state values are carried.
///
/// Initial conditions are zero, so the leading samples carry the start-up
/// transient. Pass `a = [1]` for a pure FIR filter.
///
/// # Errors
///
/// [`FreesError::Evaluation`] when either coefficient vector is empty, when
/// `a[0]` is zero, or when a coefficient is not finite.
pub fn lfilter(b: &[f64], a: &[f64], x: &[f64]) -> Result<Vec<f64>> {
    if b.is_empty() || a.is_empty() {
        return Err(FreesError::evaluation(
            "Filter requires non-empty numerator and denominator coefficients.",
        ));
    }
    if !a.iter().chain(b.iter()).all(|c| c.is_finite()) {
        return Err(FreesError::evaluation(
            "Filter coefficients must all be finite.",
        ));
    }
    if a[0] == 0.0 {
        return Err(FreesError::evaluation(
            "Filter denominator a[1] must be non-zero.",
        ));
    }
    let scale = 1.0 / a[0];
    let bn: Vec<f64> = b.iter().map(|v| v * scale).collect();
    let an: Vec<f64> = a.iter().map(|v| v * scale).collect();
    let order = bn.len().max(an.len()) - 1;
    let mut state = vec![0.0; order];
    let mut out = Vec::with_capacity(x.len());
    for &xj in x {
        let yj = bn[0] * xj + state.first().copied().unwrap_or(0.0);
        for i in 0..order {
            let next = state.get(i + 1).copied().unwrap_or(0.0);
            let bi = bn.get(i + 1).copied().unwrap_or(0.0);
            let ai = an.get(i + 1).copied().unwrap_or(0.0);
            state[i] = next + bi * xj - ai * yj;
        }
        out.push(yj);
    }
    Ok(out)
}

/// Zero-phase filtering: [`lfilter`] forward, then backward over the reversed
/// result, so the two passes' phase shifts cancel exactly. The magnitude
/// response is squared as a consequence — a `filtfilt` through a −3 dB design
/// is −6 dB at the same frequency.
///
/// The signal is odd-extended by `3·max(len(a), len(b))` samples (clipped to
/// `n − 1`) at both ends before filtering and the extension is discarded after,
/// which is what keeps the ends from ringing.
///
/// # Errors
///
/// As [`lfilter`], plus [`FreesError::Evaluation`] on a series shorter than two
/// samples (there is nothing to reflect about).
pub fn filtfilt(b: &[f64], a: &[f64], x: &[f64]) -> Result<Vec<f64>> {
    let n = x.len();
    if n < 2 {
        return Err(FreesError::evaluation(
            "FiltFilt requires at least two samples.",
        ));
    }
    // ponytail: odd extension without SciPy's steady-state `lfilter_zi` warm
    // start. The padding is what kills the bulk of the edge transient; the zi
    // refinement matters only for very short records against a long filter.
    let pad = (3 * a.len().max(b.len())).min(n - 1);
    let mut ext = Vec::with_capacity(n + 2 * pad);
    for j in (1..=pad).rev() {
        ext.push(2.0 * x[0] - x[j]);
    }
    ext.extend_from_slice(x);
    for j in 1..=pad {
        ext.push(2.0 * x[n - 1] - x[n - 1 - j]);
    }

    let forward = lfilter(b, a, &ext)?;
    let mut reversed: Vec<f64> = forward.into_iter().rev().collect();
    reversed = lfilter(b, a, &reversed)?;
    reversed.reverse();
    Ok(reversed[pad..pad + n].to_vec())
}

/// Full linear cross-correlation of `a` against `b`, length `m + n − 1`.
///
/// Output index `i` carries lag `i − (n − 1)`: `r[i] = Σⱼ a[j] · b[j − lag]`.
/// The middle element is the zero-lag correlation, a positive lag means `a`
/// leads `b`, and `xcorr(a, a)` peaks at the centre. Computed as the
/// convolution of `a` with `b` reversed, which is the same sum.
///
/// # Errors
///
/// [`FreesError::Evaluation`] when either sequence is empty.
pub fn xcorr(a: &[f64], b: &[f64]) -> Result<Vec<f64>> {
    if a.is_empty() || b.is_empty() {
        return Err(FreesError::evaluation(
            "XCorr requires two non-empty sequences.",
        ));
    }
    let flipped: Vec<f64> = b.iter().rev().copied().collect();
    convolve(a, &flipped)
}

/// Welch's averaged-periodogram power spectral density.
///
/// Splits `x` into 50 %-overlapped segments of `nperseg` samples, removes each
/// segment's mean, tapers it with a periodic Hann window, and averages the
/// squared spectra. Returns `(frequencies, pxx)`, both `nperseg/2 + 1` long:
/// frequency `k · fs / nperseg` in Hz, density in `x²/Hz`.
///
/// A trailing partial segment is dropped — it is not zero-padded, because
/// padding would quietly bias the average toward the record's tail.
///
/// # Errors
///
/// [`FreesError::Evaluation`] when `nperseg` is below 2, longer than the
/// series, or when `fs` is not positive and finite.
pub fn welch(x: &[f64], fs: f64, nperseg: usize) -> Result<(Vec<f64>, Vec<f64>)> {
    if !(fs.is_finite() && fs > 0.0) {
        return Err(FreesError::evaluation(
            "Welch requires a positive, finite sample rate.",
        ));
    }
    if nperseg < 2 {
        return Err(FreesError::evaluation(
            "Welch requires a segment length of at least 2.",
        ));
    }
    if nperseg > x.len() {
        return Err(FreesError::evaluation(format!(
            "Welch segment length ({nperseg}) exceeds the series length ({}).",
            x.len()
        )));
    }
    let w = window_weights(Window::Hann, nperseg, true);
    let win_power: f64 = w.iter().map(|v| v * v).sum();
    let scale = 1.0 / (fs * win_power);
    let step = nperseg / 2;
    let bins = nperseg / 2 + 1;
    let mut pxx = vec![0.0; bins];
    let mut segments = 0usize;

    let mut start = 0usize;
    while start + nperseg <= x.len() {
        let seg = &x[start..start + nperseg];
        let mean = seg.iter().sum::<f64>() / nperseg as f64;
        let re: Vec<f64> = seg.iter().zip(&w).map(|(v, wj)| (v - mean) * wj).collect();
        let (sp_re, sp_im) = dft(&re, &vec![0.0; nperseg], false)?;
        for k in 0..bins {
            let power = sp_re[k] * sp_re[k] + sp_im[k] * sp_im[k];
            // One-sided: fold the negative frequencies onto their twins. DC and,
            // for an even segment, Nyquist have no twin to fold.
            let fold = if k == 0 || (nperseg % 2 == 0 && k == nperseg / 2) {
                1.0
            } else {
                2.0
            };
            pxx[k] += power * scale * fold;
        }
        segments += 1;
        start += step;
    }

    let inv = 1.0 / segments as f64;
    for v in &mut pxx {
        *v *= inv;
    }
    let freqs = (0..bins).map(|k| k as f64 * fs / nperseg as f64).collect();
    Ok((freqs, pxx))
}

/// Indices of the local maxima of `x`, in ascending index order.
///
/// A peak is a sample strictly greater than both neighbours; the first and last
/// samples are never peaks, having only one. `min_height` rejects peaks at or
/// below a threshold (pass `f64::NEG_INFINITY` for none). `min_distance`
/// enforces a minimum index separation by keeping the **tallest** peak of each
/// contested group first, then discarding anything within the distance of an
/// already-kept peak — so a broad, noisy shoulder does not out-vote the crest
/// beside it.
#[must_use]
pub fn peak_indices(x: &[f64], min_height: f64, min_distance: usize) -> Vec<usize> {
    let mut found: Vec<usize> = (1..x.len().saturating_sub(1))
        .filter(|&j| x[j] > x[j - 1] && x[j] > x[j + 1] && x[j] > min_height)
        .collect();
    if min_distance <= 1 || found.len() < 2 {
        return found;
    }
    // Tallest first, ties to the lower index so the result is deterministic.
    found.sort_by(|&i, &j| {
        x[j].partial_cmp(&x[i])
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(i.cmp(&j))
    });
    let mut kept: Vec<usize> = Vec::new();
    for candidate in found {
        if kept.iter().all(|&k| candidate.abs_diff(k) >= min_distance) {
            kept.push(candidate);
        }
    }
    kept.sort_unstable();
    kept
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) {
        assert!(
            (a - b).abs() <= 1e-12 * b.abs().max(1.0),
            "expected {b}, got {a}"
        );
    }

    #[test]
    fn dft_of_a_unit_impulse_is_flat() {
        let (re, im) = dft(&[1.0, 0.0, 0.0, 0.0], &[0.0; 4], false).unwrap();
        for k in 0..4 {
            close(re[k], 1.0);
            close(im[k], 0.0);
        }
    }

    #[test]
    fn dft_known_values_for_a_small_real_sequence() {
        // X = DFT([0, 1, 0, 1]) = [2, 0, −2, 0] (all real).
        let (re, im) = dft(&[0.0, 1.0, 0.0, 1.0], &[0.0; 4], false).unwrap();
        let expected = [2.0, 0.0, -2.0, 0.0];
        for k in 0..4 {
            close(re[k], expected[k]);
            close(im[k], 0.0);
        }
    }

    #[test]
    fn inverse_dft_round_trips_and_normalizes() {
        let x_re = [3.0, -1.0, 0.5, 2.25, -7.0];
        let x_im = [0.0, 1.0, -2.0, 0.0, 4.5];
        let (f_re, f_im) = dft(&x_re, &x_im, false).unwrap();
        let (b_re, b_im) = dft(&f_re, &f_im, true).unwrap();
        for k in 0..5 {
            close(b_re[k], x_re[k]);
            close(b_im[k], x_im[k]);
        }
    }

    #[test]
    fn dft_rejects_mismatched_component_lengths() {
        let err = dft(&[1.0, 2.0], &[0.0], false).unwrap_err().to_string();
        assert!(err.contains("equal length"), "{err}");
    }

    /// A deterministic, badly-behaved-on-purpose complex sequence: mixed
    /// magnitudes, both signs, no symmetry the transform could exploit.
    fn sample(n: usize) -> (Vec<f64>, Vec<f64>) {
        let re = (0..n)
            .map(|j| libm::sin(0.7 * j as f64) * (1.0 + 0.3 * j as f64))
            .collect();
        let im = (0..n)
            .map(|j| libm::cos(1.3 * j as f64) - 0.05 * j as f64)
            .collect();
        (re, im)
    }

    /// The one check that matters: every fast path must reproduce the direct
    /// kernel it replaced, at power-of-two, composite, prime and just-past-the
    /// threshold lengths, forward and inverse.
    #[test]
    fn fast_paths_reproduce_the_direct_transform() {
        for &n in &[33usize, 37, 60, 64, 97, 100, 128, 127, 256, 360] {
            let (re, im) = sample(n);
            for inverse in [false, true] {
                let (fast_re, fast_im) = dft(&re, &im, inverse).unwrap();
                let (mut slow_re, mut slow_im) = direct(&re, &im, if inverse { 1.0 } else { -1.0 });
                if inverse {
                    let scale = 1.0 / n as f64;
                    for k in 0..n {
                        slow_re[k] *= scale;
                        slow_im[k] *= scale;
                    }
                }
                let norm = slow_re
                    .iter()
                    .chain(slow_im.iter())
                    .fold(0.0f64, |m, v| m.max(v.abs()));
                for k in 0..n {
                    let tol = 1e-11 * norm.max(1.0);
                    assert!(
                        (fast_re[k] - slow_re[k]).abs() <= tol
                            && (fast_im[k] - slow_im[k]).abs() <= tol,
                        "n={n} inverse={inverse} k={k}: fast ({}, {}) vs direct ({}, {})",
                        fast_re[k],
                        fast_im[k],
                        slow_re[k],
                        slow_im[k]
                    );
                }
            }
        }
    }

    /// Lengths at or below the base case must be bit-identical to the kernel
    /// the frozen fixtures were generated with — not merely close to it.
    #[test]
    fn small_lengths_keep_the_direct_kernel_bit_for_bit() {
        for n in 0..=DIRECT_MAX {
            let (re, im) = sample(n);
            let (got_re, got_im) = dft(&re, &im, false).unwrap();
            let (want_re, want_im) = direct(&re, &im, -1.0);
            assert_eq!(got_re, want_re, "n={n} real part drifted");
            assert_eq!(got_im, want_im, "n={n} imaginary part drifted");
        }
    }

    #[test]
    fn long_transforms_round_trip_at_prime_and_power_of_two_lengths() {
        for &n in &[127usize, 512, 1000] {
            let (re, im) = sample(n);
            let (f_re, f_im) = dft(&re, &im, false).unwrap();
            let (b_re, b_im) = dft(&f_re, &f_im, true).unwrap();
            let norm = re
                .iter()
                .chain(im.iter())
                .fold(0.0f64, |m, v| m.max(v.abs()));
            for k in 0..n {
                let tol = 1e-11 * norm.max(1.0);
                assert!(
                    (b_re[k] - re[k]).abs() <= tol && (b_im[k] - im[k]).abs() <= tol,
                    "n={n} k={k}: round trip gave ({}, {}), wanted ({}, {})",
                    b_re[k],
                    b_im[k],
                    re[k],
                    im[k]
                );
            }
        }
    }

    /// A pure tone must land entirely in its own bin at a length that forces
    /// Bluestein — the property a wrong chirp sign or a short convolution
    /// buffer would break while still round-tripping.
    #[test]
    fn bluestein_places_a_pure_tone_in_one_bin() {
        let n = 105; // 3·5·7 — not a power of two
        let bin = 8usize;
        let re: Vec<f64> = (0..n)
            .map(|j| libm::cos(2.0 * PI * bin as f64 * j as f64 / n as f64))
            .collect();
        let im: Vec<f64> = (0..n)
            .map(|j| libm::sin(2.0 * PI * bin as f64 * j as f64 / n as f64))
            .collect();
        let (out_re, out_im) = dft(&re, &im, false).unwrap();
        for k in 0..n {
            let mag = libm::hypot(out_re[k], out_im[k]);
            if k == bin {
                // forward sign is e^{−2πi jk/n}, so e^{+2πi·bin·j/n} sums to n
                // at k = bin and cancels everywhere else.
                assert!((mag - n as f64).abs() < 1e-9, "k={k} magnitude {mag}");
            } else {
                assert!(mag < 1e-9, "leaked {mag} into bin {k}");
            }
        }
    }

    #[test]
    fn convolve_known_values() {
        // [1,2,3] * [4,5] = [4, 13, 22, 15]
        let c = convolve(&[1.0, 2.0, 3.0], &[4.0, 5.0]).unwrap();
        assert_eq!(c.len(), 4);
        close(c[0], 4.0);
        close(c[1], 13.0);
        close(c[2], 22.0);
        close(c[3], 15.0);
    }

    #[test]
    fn convolve_with_a_unit_impulse_is_identity() {
        let c = convolve(&[1.0], &[7.0, -2.0, 0.5]).unwrap();
        assert_eq!(c, vec![7.0, -2.0, 0.5]);
    }

    #[test]
    fn convolve_refuses_empty_input() {
        let err = convolve(&[], &[1.0]).unwrap_err().to_string();
        assert!(err.contains("non-empty"), "{err}");
    }

    // ── Phase 4.5 sensor kernels ────────────────────────────────────────────

    #[test]
    fn windows_are_normalized_and_shaped() {
        // Symmetric windows start and end at the textbook endpoint value.
        let hann = window_weights(Window::Hann, 5, false);
        close(hann[0], 0.0);
        close(hann[4], 0.0);
        close(hann[2], 1.0);
        let hamming = window_weights(Window::Hamming, 5, false);
        close(hamming[0], 0.08);
        close(hamming[2], 1.0);
        let bartlett = window_weights(Window::Bartlett, 5, false);
        assert_eq!(bartlett, vec![0.0, 0.5, 1.0, 0.5, 0.0]);
        // A periodic window omits the closing zero — that is the whole
        // difference, and getting it backwards biases every Welch estimate.
        let periodic = window_weights(Window::Hann, 4, true);
        assert_eq!(periodic.len(), 4);
        close(periodic[0], 0.0);
        close(periodic[2], 1.0);
        assert!(periodic[3] > 0.0, "a periodic Hann must not close on zero");
        // Rect is the identity taper at any length.
        assert_eq!(window_weights(Window::Rect, 3, false), vec![1.0; 3]);
    }

    #[test]
    fn window_names_resolve_case_folded_by_the_caller() {
        assert_eq!(Window::from_name("hanning"), Some(Window::Hann));
        assert_eq!(Window::from_name("boxcar"), Some(Window::Rect));
        assert_eq!(Window::from_name("triangular"), Some(Window::Bartlett));
        assert_eq!(Window::from_name("gaussian"), None);
    }

    #[test]
    fn detrend_removes_exactly_the_trend_it_claims() {
        // A pure line detrends to zero.
        let line: Vec<f64> = (0..10).map(|j| 3.0 + 2.5 * j as f64).collect();
        for v in detrend(&line, true).unwrap() {
            assert!(v.abs() < 1e-12, "linear detrend left {v}");
        }
        // Constant detrend leaves the slope alone and only re-centres.
        let flat = detrend(&line, false).unwrap();
        close(flat.iter().sum::<f64>(), 0.0);
        close(flat[1] - flat[0], 2.5);
        // Detrending is linear, so adding a line to a signal must not change
        // what detrend returns for it: detrend(line + tone) == detrend(tone).
        let tone: Vec<f64> = (0..64).map(|j| libm::sin(0.3 * j as f64)).collect();
        let mixed: Vec<f64> = tone
            .iter()
            .enumerate()
            .map(|(j, v)| 1.0 - 0.4 * j as f64 + v)
            .collect();
        let out = detrend(&mixed, true).unwrap();
        let expected = detrend(&tone, true).unwrap();
        for j in 0..64 {
            assert!(
                (out[j] - expected[j]).abs() < 1e-10,
                "j={j}: {} vs {}",
                out[j],
                expected[j]
            );
        }
        assert!(detrend(&[], true).is_err());
    }

    #[test]
    fn moving_average_preserves_a_constant_at_the_edges() {
        let out = moving_average(&[5.0; 7], 3).unwrap();
        assert_eq!(out, vec![5.0; 7]);
        let out = moving_average(&[1.0, 2.0, 3.0, 4.0, 5.0], 3).unwrap();
        // Ends average over what exists: (1+2)/2 and (4+5)/2.
        assert_eq!(out, vec![1.5, 2.0, 3.0, 4.0, 4.5]);
        assert!(moving_average(&[1.0, 2.0], 2).is_err(), "even window");
        assert!(moving_average(&[1.0, 2.0], 5).is_err(), "window > series");
        assert!(moving_average(&[], 3).is_err());
    }

    #[test]
    fn lfilter_matches_the_difference_equation_it_documents() {
        // y[j] = 0.5·x[j] + 0.5·x[j−1] — a two-tap FIR.
        let y = lfilter(&[0.5, 0.5], &[1.0], &[1.0, 2.0, 3.0, 4.0]).unwrap();
        assert_eq!(y, vec![0.5, 1.5, 2.5, 3.5]);
        // y[j] = x[j] + 0.5·y[j−1] — a one-pole IIR, checked by hand.
        let y = lfilter(&[1.0], &[1.0, -0.5], &[1.0, 0.0, 0.0, 0.0]).unwrap();
        for (j, v) in y.iter().enumerate() {
            close(*v, libm::pow(0.5, j as f64));
        }
        // a[0] scales the whole equation.
        let y = lfilter(&[2.0], &[2.0], &[1.0, 2.0]).unwrap();
        assert_eq!(y, vec![1.0, 2.0]);
        assert!(lfilter(&[1.0], &[0.0], &[1.0]).is_err(), "a[0] = 0");
        assert!(lfilter(&[], &[1.0], &[1.0]).is_err());
        assert!(lfilter(&[f64::NAN], &[1.0], &[1.0]).is_err());
    }

    #[test]
    fn filtfilt_is_zero_phase_and_preserves_a_constant() {
        // A DC-gain-1 lowpass must leave a constant untouched, ends included —
        // the property that a wrong padding or an unnormalized pass breaks.
        let b = [0.2, 0.2, 0.2, 0.2, 0.2];
        let x = vec![4.0; 40];
        for v in filtfilt(&b, &[1.0], &x).unwrap() {
            assert!((v - 4.0).abs() < 1e-9, "constant became {v}");
        }
        // Zero phase: a symmetric pulse must stay symmetric about its centre.
        let mut pulse = vec![0.0; 41];
        pulse[20] = 1.0;
        let out = filtfilt(&b, &[1.0], &pulse).unwrap();
        for k in 1..=15 {
            assert!(
                (out[20 - k] - out[20 + k]).abs() < 1e-9,
                "asymmetric at lag {k}: {} vs {}",
                out[20 - k],
                out[20 + k]
            );
        }
        assert!(filtfilt(&b, &[1.0], &[1.0]).is_err(), "needs two samples");
    }

    #[test]
    fn xcorr_peaks_at_the_true_lag() {
        let a = [0.0, 0.0, 1.0, 2.0, 1.0, 0.0, 0.0];
        // b is a delayed by 1 sample.
        let b = [0.0, 1.0, 2.0, 1.0, 0.0, 0.0, 0.0];
        let r = xcorr(&a, &b).unwrap();
        assert_eq!(r.len(), a.len() + b.len() - 1);
        let centre = b.len() - 1;
        let best = (0..r.len())
            .max_by(|&i, &j| r[i].partial_cmp(&r[j]).unwrap())
            .unwrap();
        assert_eq!(best as i64 - centre as i64, 1, "a leads b by one sample");
        // Autocorrelation peaks at zero lag.
        let auto = xcorr(&a, &a).unwrap();
        let best = (0..auto.len())
            .max_by(|&i, &j| auto[i].partial_cmp(&auto[j]).unwrap())
            .unwrap();
        assert_eq!(best, a.len() - 1);
        assert!(xcorr(&[], &[1.0]).is_err());
    }

    #[test]
    fn welch_finds_a_tone_and_conserves_its_power() {
        // 1 Vrms tone (amplitude √2) at 50 Hz, sampled at 1 kHz.
        let fs = 1000.0;
        let n = 4096;
        let amp = std::f64::consts::SQRT_2;
        let x: Vec<f64> = (0..n)
            .map(|j| amp * libm::sin(2.0 * PI * 50.0 * j as f64 / fs))
            .collect();
        let (f, pxx) = welch(&x, fs, 256).unwrap();
        assert_eq!(f.len(), 129);
        assert_eq!(pxx.len(), 129);
        close(f[1], fs / 256.0);
        let peak = (0..pxx.len())
            .max_by(|&i, &j| pxx[i].partial_cmp(&pxx[j]).unwrap())
            .unwrap();
        assert!(
            (f[peak] - 50.0).abs() <= f[1],
            "peak landed at {} Hz",
            f[peak]
        );
        // Σ Pxx·Δf is the mean square, which for a 1 Vrms tone is 1.
        let df = f[1];
        let power: f64 = pxx.iter().sum::<f64>() * df;
        assert!(
            (power - 1.0).abs() < 0.05,
            "integrated power {power}, wanted 1"
        );
        assert!(welch(&x, 0.0, 256).is_err(), "fs must be positive");
        assert!(welch(&x, fs, 1).is_err(), "nperseg must exceed 1");
        assert!(welch(&x, fs, n + 1).is_err(), "nperseg must fit");
    }

    #[test]
    fn peak_detection_respects_height_and_distance() {
        //          0    1    2    3    4    5    6    7    8
        let x = [0.0, 3.0, 0.0, 1.0, 0.0, 5.0, 4.5, 6.0, 0.0];
        assert_eq!(peak_indices(&x, f64::NEG_INFINITY, 0), vec![1, 3, 5, 7]);
        assert_eq!(peak_indices(&x, 2.0, 0), vec![1, 5, 7]);
        // Distance 4 keeps the tallest (7) and then 1, dropping 5 as too close
        // to 7 — the "tallest wins the group" rule, not "first wins".
        assert_eq!(peak_indices(&x, f64::NEG_INFINITY, 4), vec![1, 7]);
        // Endpoints are never peaks, and a monotone series has none.
        assert!(peak_indices(&[1.0, 2.0, 3.0], f64::NEG_INFINITY, 0).is_empty());
        assert!(peak_indices(&[], f64::NEG_INFINITY, 0).is_empty());
        // A plateau is not a strict maximum.
        assert!(peak_indices(&[0.0, 1.0, 1.0, 0.0], f64::NEG_INFINITY, 0).is_empty());
    }
}
