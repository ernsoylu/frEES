//! Extended descriptive statistics, correlation, hypothesis tests, and
//! resampling methods — Phase 4.1 of the engineering roadmap.
//!
//! Every function in this module is a pure numerical kernel: no browser APIs,
//! no `wasm-bindgen`, no allocator tricks.  The evaluator dispatches into these
//! through the `strict!` / `lazy!` table in `eval.rs`.
//!
//! # Conventions
//!
//! * **Sample (n − 1) denominators** are the default for variance, covariance,
//!   skewness, and kurtosis — matching the existing `sample_variance` in
//!   `eval.rs` and the behaviour of NumPy `ddof=1` / SciPy `bias=False`.
//! * **Weighted mean/variance** use *reliability weights* (not frequency
//!   weights): the denominator for the weighted variance is
//!   `(Σwᵢ)² − Σwᵢ²` / `Σwᵢ`, the Bessel-corrected form.
//! * Missing-data policy: NaN inputs propagate naturally through arithmetic;
//!   no special filtering is applied.  Functions that require sorted data
//!   place NaN last (matching the existing `median` / `percentile`).
//! * Minimum sample sizes are enforced with clear errors; constant-data edge
//!   cases return 0 rather than NaN for spread measures, matching the existing
//!   `sample_variance` convention.

#![allow(clippy::needless_range_loop)]

use crate::diag::{FreesError, Result};

// ── helpers ────────────────────────────────────────────────────────────────

fn sort_finite(v: &[f64]) -> Vec<f64> {
    let mut s = v.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    s
}

/// Fractional rank for Spearman: ties get the mean of their ordinal positions.
fn fractional_ranks(v: &[f64]) -> Vec<f64> {
    let n = v.len();
    let mut indices: Vec<usize> = (0..n).collect();
    indices.sort_by(|&a, &b| v[a].partial_cmp(&v[b]).unwrap_or(std::cmp::Ordering::Equal));
    let mut ranks = vec![0.0; n];
    let mut i = 0;
    while i < n {
        let mut j = i + 1;
        while j < n
            && v[indices[j]]
                .partial_cmp(&v[indices[i]])
                .unwrap_or(std::cmp::Ordering::Equal)
                == std::cmp::Ordering::Equal
        {
            j += 1;
        }
        // Tie group [i, j): assign the mean rank (1-based).
        let mean_rank = (i + j) as f64 / 2.0 + 0.5;
        for k in i..j {
            ranks[indices[k]] = mean_rank;
        }
        i = j;
    }
    ranks
}

// ── 1. Weighted descriptive statistics ─────────────────────────────────────

/// Weighted arithmetic mean.  Weights must be non-negative and their sum
/// must be positive.
pub fn weighted_mean(w: &[f64], x: &[f64]) -> Result<f64> {
    if w.len() != x.len() {
        return Err(FreesError::evaluation(
            "wmean: weights and values must have equal length.",
        ));
    }
    if w.is_empty() {
        return Err(FreesError::evaluation(
            "wmean: at least 1 value is required.",
        ));
    }
    let sum_w: f64 = w.iter().sum();
    if sum_w <= 0.0 || sum_w.is_nan() {
        return Err(FreesError::evaluation(
            "wmean: sum of weights must be positive.",
        ));
    }
    let sum_wx: f64 = w.iter().zip(x.iter()).map(|(wi, xi)| wi * xi).sum();
    Ok(sum_wx / sum_w)
}

/// Bessel-corrected weighted sample variance (reliability weights).
///
/// Uses the unbiased estimator with denominator `Σwᵢ − Σwᵢ²/Σwᵢ`,
/// matching the convention of `numpy.average` + manual Bessel correction
/// and statsmodels `DescrStatsW`.
pub fn weighted_variance(w: &[f64], x: &[f64]) -> Result<f64> {
    if w.len() != x.len() {
        return Err(FreesError::evaluation(
            "wvar: weights and values must have equal length.",
        ));
    }
    if x.len() < 2 {
        return Err(FreesError::evaluation(
            "wvar: at least 2 values are required.",
        ));
    }
    let sum_w: f64 = w.iter().sum();
    if sum_w <= 0.0 || sum_w.is_nan() {
        return Err(FreesError::evaluation(
            "wvar: sum of weights must be positive.",
        ));
    }
    let mu = weighted_mean(w, x)?;
    let sum_w2: f64 = w.iter().map(|wi| wi * wi).sum();
    let denom = sum_w - sum_w2 / sum_w;
    if denom <= 0.0 {
        // All weight on one observation: variance is undefined.
        return Err(FreesError::evaluation(
            "wvar: effective sample size is too small (all weight on one observation).",
        ));
    }
    let ss: f64 = w
        .iter()
        .zip(x.iter())
        .map(|(wi, xi)| wi * (xi - mu) * (xi - mu))
        .sum();
    Ok(ss / denom)
}

// ── 2. Robust summaries ────────────────────────────────────────────────────

/// Median absolute deviation: `median(|xᵢ − median(x)|)`.
pub fn median_abs_deviation(v: &[f64]) -> Result<f64> {
    if v.is_empty() {
        return Err(FreesError::evaluation("mad: at least 1 value is required."));
    }
    let med = median_of(v);
    let deviations: Vec<f64> = v.iter().map(|x| libm::fabs(x - med)).collect();
    Ok(median_of(&deviations))
}

/// `α`-trimmed mean: drop the lowest and highest `α` fraction of the data
/// (each side), then take the mean of the rest.  `α` must be in `[0, 0.5)`.
pub fn trimmed_mean(alpha: f64, v: &[f64]) -> Result<f64> {
    if v.is_empty() {
        return Err(FreesError::evaluation(
            "trimmedmean: at least 1 value is required.",
        ));
    }
    if !(0.0..0.5).contains(&alpha) {
        return Err(FreesError::evaluation(format!(
            "trimmedmean: trim fraction must be in [0, 0.5), got {alpha}.",
        )));
    }
    let sorted = sort_finite(v);
    let n = sorted.len();
    let k = libm::floor(alpha * n as f64) as usize;
    let trimmed = &sorted[k..n - k];
    if trimmed.is_empty() {
        return Err(FreesError::evaluation(
            "trimmedmean: trim fraction too large for the sample size.",
        ));
    }
    Ok(trimmed.iter().sum::<f64>() / trimmed.len() as f64)
}

/// Sample skewness (bias-corrected, Fisher's definition).
/// Uses the `n(n−1)/(n−2)` adjusted formula matching SciPy `skew(bias=False)`.
pub fn skewness(v: &[f64]) -> Result<f64> {
    let n = v.len();
    if n < 3 {
        return Err(FreesError::evaluation(
            "skewness: at least 3 values are required.",
        ));
    }
    let nf = n as f64;
    let m = v.iter().sum::<f64>() / nf;
    let m2: f64 = v.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / nf;
    if m2 == 0.0 {
        return Ok(0.0); // constant data
    }
    let m3: f64 = v
        .iter()
        .map(|x| {
            let d = x - m;
            d * d * d
        })
        .sum::<f64>()
        / nf;
    let g1 = m3 / libm::pow(m2, 1.5);
    // Bias correction: G1 = g1 * sqrt(n(n-1)) / (n-2)
    Ok(g1 * libm::sqrt(nf * (nf - 1.0)) / (nf - 2.0))
}

/// Excess kurtosis (bias-corrected, Fisher's definition).
/// Uses the formula matching SciPy `kurtosis(bias=False, fisher=True)`.
pub fn kurtosis(v: &[f64]) -> Result<f64> {
    let n = v.len();
    if n < 4 {
        return Err(FreesError::evaluation(
            "kurtosis: at least 4 values are required.",
        ));
    }
    let nf = n as f64;
    let m = v.iter().sum::<f64>() / nf;
    let m2: f64 = v.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / nf;
    if m2 == 0.0 {
        return Ok(0.0); // constant data: excess kurtosis is 0 by convention
    }
    let m4: f64 = v
        .iter()
        .map(|x| {
            let d = x - m;
            d * d * d * d
        })
        .sum::<f64>()
        / nf;
    let g2 = m4 / (m2 * m2) - 3.0;
    // Bias correction: G2 = ((n-1)/((n-2)(n-3))) * ((n+1)*g2 + 6)
    Ok(((nf - 1.0) / ((nf - 2.0) * (nf - 3.0))) * ((nf + 1.0) * g2 + 6.0))
}

// ── 3. Covariance & Correlation ────────────────────────────────────────────

/// Unbiased (n − 1) sample covariance of `x` and `y`.
pub fn sample_covariance(x: &[f64], y: &[f64]) -> Result<f64> {
    if x.len() != y.len() {
        return Err(FreesError::evaluation(
            "cov: x and y must have equal length.",
        ));
    }
    if x.len() < 2 {
        return Err(FreesError::evaluation(
            "cov: at least 2 data points are required.",
        ));
    }
    let n = x.len() as f64;
    let mx: f64 = x.iter().sum::<f64>() / n;
    let my: f64 = y.iter().sum::<f64>() / n;
    let sxy: f64 = x
        .iter()
        .zip(y.iter())
        .map(|(xi, yi)| (xi - mx) * (yi - my))
        .sum();
    Ok(sxy / (n - 1.0))
}

/// Pearson product-moment correlation coefficient.
pub fn pearson_correlation(x: &[f64], y: &[f64]) -> Result<f64> {
    if x.len() != y.len() {
        return Err(FreesError::evaluation(
            "pearson: x and y must have equal length.",
        ));
    }
    if x.len() < 2 {
        return Err(FreesError::evaluation(
            "pearson: at least 2 data points are required.",
        ));
    }
    let n = x.len() as f64;
    let mx: f64 = x.iter().sum::<f64>() / n;
    let my: f64 = y.iter().sum::<f64>() / n;
    let mut sxx = 0.0_f64;
    let mut syy = 0.0_f64;
    let mut sxy = 0.0_f64;
    for (xi, yi) in x.iter().zip(y.iter()) {
        let dx = xi - mx;
        let dy = yi - my;
        sxx += dx * dx;
        syy += dy * dy;
        sxy += dx * dy;
    }
    let denom = libm::sqrt(sxx * syy);
    if denom == 0.0 {
        return Ok(0.0); // constant x or y
    }
    Ok(sxy / denom)
}

/// Spearman rank correlation coefficient.
pub fn spearman_correlation(x: &[f64], y: &[f64]) -> Result<f64> {
    if x.len() != y.len() {
        return Err(FreesError::evaluation(
            "spearman: x and y must have equal length.",
        ));
    }
    if x.len() < 2 {
        return Err(FreesError::evaluation(
            "spearman: at least 2 data points are required.",
        ));
    }
    let rx = fractional_ranks(x);
    let ry = fractional_ranks(y);
    pearson_correlation(&rx, &ry)
}

// ── 4. Hypothesis tests ───────────────────────────────────────────────────
//
// Each test returns a struct with named fields.  The evaluator maps
// individual function names (`ttest1_stat`, `ttest1_pval`, `ttest1_df`)
// to the corresponding field.

/// Result of a hypothesis test producing a test statistic, degrees of
/// freedom, and a two-sided p-value.
#[derive(Debug, Clone, PartialEq)]
pub struct TestResult {
    pub statistic: f64,
    pub df: f64,
    pub p_value: f64,
}

/// Result of an ANOVA test.
#[derive(Debug, Clone, PartialEq)]
pub struct AnovaResult {
    pub f_statistic: f64,
    pub df_between: f64,
    pub df_within: f64,
    pub p_value: f64,
}

/// One-sample t-test: test whether the population mean equals `mu0`.
///
/// The test statistic is `t = (x̄ − μ₀) / (s / √n)` with `n − 1` degrees
/// of freedom.  The p-value is two-sided.
pub fn ttest_one_sample(
    mu0: f64,
    x: &[f64],
    t_cdf: fn(f64, f64) -> Result<f64>,
) -> Result<TestResult> {
    let n = x.len();
    if n < 2 {
        return Err(FreesError::evaluation(
            "ttest1: at least 2 observations are required.",
        ));
    }
    let nf = n as f64;
    let mean = x.iter().sum::<f64>() / nf;
    let var = x.iter().map(|v| (v - mean) * (v - mean)).sum::<f64>() / (nf - 1.0);
    let se = libm::sqrt(var / nf);
    if se == 0.0 {
        // Constant data: if mean == mu0 then p=1, else p=0.
        return Ok(TestResult {
            statistic: if (mean - mu0).abs() == 0.0 {
                0.0
            } else {
                f64::INFINITY
            },
            df: nf - 1.0,
            p_value: if (mean - mu0).abs() == 0.0 { 1.0 } else { 0.0 },
        });
    }
    let t = (mean - mu0) / se;
    let df = nf - 1.0;
    let p = two_sided_t_pvalue(t, df, t_cdf)?;
    Ok(TestResult {
        statistic: t,
        df,
        p_value: p,
    })
}

/// Paired-samples t-test: test whether the mean difference is zero.
///
/// Equivalent to a one-sample t-test on the pairwise differences `xᵢ − yᵢ`.
pub fn ttest_paired(
    x: &[f64],
    y: &[f64],
    t_cdf: fn(f64, f64) -> Result<f64>,
) -> Result<TestResult> {
    if x.len() != y.len() {
        return Err(FreesError::evaluation(
            "ttest_paired: x and y must have equal length.",
        ));
    }
    let diffs: Vec<f64> = x.iter().zip(y.iter()).map(|(a, b)| a - b).collect();
    ttest_one_sample(0.0, &diffs, t_cdf)
}

/// Welch's (two-sample, unequal-variance) t-test.
///
/// Tests whether two independent samples have the same population mean.
/// Uses the Welch–Satterthwaite approximation for degrees of freedom.
pub fn ttest_welch(x: &[f64], y: &[f64], t_cdf: fn(f64, f64) -> Result<f64>) -> Result<TestResult> {
    if x.len() < 2 {
        return Err(FreesError::evaluation(
            "ttest2: x must have at least 2 observations.",
        ));
    }
    if y.len() < 2 {
        return Err(FreesError::evaluation(
            "ttest2: y must have at least 2 observations.",
        ));
    }
    let nx = x.len() as f64;
    let ny = y.len() as f64;
    let mx = x.iter().sum::<f64>() / nx;
    let my = y.iter().sum::<f64>() / ny;
    let vx = x.iter().map(|v| (v - mx) * (v - mx)).sum::<f64>() / (nx - 1.0);
    let vy = y.iter().map(|v| (v - my) * (v - my)).sum::<f64>() / (ny - 1.0);
    let se2 = vx / nx + vy / ny;
    if se2 == 0.0 {
        let diff = (mx - my).abs();
        return Ok(TestResult {
            statistic: if diff == 0.0 { 0.0 } else { f64::INFINITY },
            df: nx + ny - 2.0,
            p_value: if diff == 0.0 { 1.0 } else { 0.0 },
        });
    }
    let t = (mx - my) / libm::sqrt(se2);
    // Welch–Satterthwaite degrees of freedom.
    let num = se2 * se2;
    let denom = (vx / nx) * (vx / nx) / (nx - 1.0) + (vy / ny) * (vy / ny) / (ny - 1.0);
    let df = num / denom;
    let p = two_sided_t_pvalue(t, df, t_cdf)?;
    Ok(TestResult {
        statistic: t,
        df,
        p_value: p,
    })
}

/// One-way ANOVA F-test.
///
/// Takes a slice of groups; each group is a slice of observations.
/// Tests whether all group means are equal.
pub fn anova_oneway(
    groups: &[&[f64]],
    f_cdf: fn(f64, f64, f64) -> Result<f64>,
) -> Result<AnovaResult> {
    let k = groups.len();
    if k < 2 {
        return Err(FreesError::evaluation(
            "anova1: at least 2 groups are required.",
        ));
    }
    for (i, g) in groups.iter().enumerate() {
        if g.len() < 2 {
            return Err(FreesError::evaluation(format!(
                "anova1: group {} must have at least 2 observations.",
                i + 1
            )));
        }
    }
    let n_total: usize = groups.iter().map(|g| g.len()).sum();
    let grand_mean: f64 = groups.iter().flat_map(|g| g.iter()).sum::<f64>() / n_total as f64;

    let mut ss_between = 0.0_f64;
    let mut ss_within = 0.0_f64;
    for g in groups {
        let ni = g.len() as f64;
        let gi_mean = g.iter().sum::<f64>() / ni;
        ss_between += ni * (gi_mean - grand_mean) * (gi_mean - grand_mean);
        ss_within += g.iter().map(|x| (x - gi_mean) * (x - gi_mean)).sum::<f64>();
    }

    let df_between = (k - 1) as f64;
    let df_within = (n_total - k) as f64;
    if df_within <= 0.0 {
        return Err(FreesError::evaluation(
            "anova1: not enough total observations for the number of groups.",
        ));
    }
    let ms_between = ss_between / df_between;
    let ms_within = ss_within / df_within;
    let f = if ms_within == 0.0 {
        if ms_between == 0.0 {
            0.0
        } else {
            f64::INFINITY
        }
    } else {
        ms_between / ms_within
    };

    let p = if f.is_infinite() {
        0.0
    } else {
        let cdf_val = f_cdf(f, df_between, df_within)?;
        1.0 - cdf_val
    };

    Ok(AnovaResult {
        f_statistic: f,
        df_between,
        df_within,
        p_value: p,
    })
}

/// Chi-square goodness-of-fit test.
///
/// `observed` and `expected` must have equal length and sum to the same
/// total (or `expected` is rescaled).  Returns (χ² statistic, df, p-value)
/// where `df = k − 1`.
pub fn chi2_goodness_of_fit(
    observed: &[f64],
    expected: &[f64],
    chi2_cdf: fn(f64, f64) -> Result<f64>,
) -> Result<TestResult> {
    if observed.len() != expected.len() {
        return Err(FreesError::evaluation(
            "chi2gof: observed and expected must have equal length.",
        ));
    }
    let k = observed.len();
    if k < 2 {
        return Err(FreesError::evaluation(
            "chi2gof: at least 2 categories are required.",
        ));
    }
    let sum_obs: f64 = observed.iter().sum();
    let sum_exp: f64 = expected.iter().sum();
    if sum_exp <= 0.0 {
        return Err(FreesError::evaluation(
            "chi2gof: sum of expected frequencies must be positive.",
        ));
    }
    // Rescale expected to match observed total.
    let scale = sum_obs / sum_exp;
    let mut chi2 = 0.0_f64;
    for i in 0..k {
        let ei = expected[i] * scale;
        if ei <= 0.0 {
            return Err(FreesError::evaluation(format!(
                "chi2gof: expected frequency for category {} is non-positive.",
                i + 1
            )));
        }
        let diff = observed[i] - ei;
        chi2 += diff * diff / ei;
    }
    let df = (k - 1) as f64;
    let cdf_val = chi2_cdf(chi2, df)?;
    let p = 1.0 - cdf_val;
    Ok(TestResult {
        statistic: chi2,
        df,
        p_value: p,
    })
}

/// Confidence interval for the population mean using the t-distribution.
///
/// Returns `(lower, upper)` such that the true mean lies within with
/// probability `confidence` (e.g. 0.95).
pub fn ci_mean(
    confidence: f64,
    x: &[f64],
    t_inv: fn(f64, f64) -> Result<f64>,
) -> Result<(f64, f64)> {
    if x.len() < 2 {
        return Err(FreesError::evaluation(
            "ci_mean: at least 2 observations are required.",
        ));
    }
    if !(0.0..1.0).contains(&confidence) {
        return Err(FreesError::evaluation(format!(
            "ci_mean: confidence level must be in (0, 1), got {confidence}.",
        )));
    }
    let n = x.len() as f64;
    let mean = x.iter().sum::<f64>() / n;
    let var = x.iter().map(|v| (v - mean) * (v - mean)).sum::<f64>() / (n - 1.0);
    let se = libm::sqrt(var / n);
    let alpha = 1.0 - confidence;
    let df = n - 1.0;
    // t_inv gives the inverse CDF: t such that P(T ≤ t) = p.
    let t_crit = t_inv(1.0 - alpha / 2.0, df)?;
    let margin = t_crit * se;
    Ok((mean - margin, mean + margin))
}

// ── 5. Resampling methods ─────────────────────────────────────────────────

/// Seeded bootstrap percentile confidence interval for the mean.
///
/// Draws `n_resamples` bootstrap samples (with replacement) from `x`,
/// computes the mean of each, and returns the `(α/2, 1−α/2)` quantiles
/// as `(lower, upper)`.
///
/// Uses the same `JavaRandom` seeded LCG as the Monte Carlo engine for
/// reproducibility.
pub fn bootstrap_ci_mean(
    confidence: f64,
    n_resamples: usize,
    seed: i64,
    x: &[f64],
) -> Result<(f64, f64)> {
    if x.is_empty() {
        return Err(FreesError::evaluation(
            "bootstrap_ci: at least 1 observation is required.",
        ));
    }
    if !(0.0..1.0).contains(&confidence) {
        return Err(FreesError::evaluation(format!(
            "bootstrap_ci: confidence level must be in (0, 1), got {confidence}.",
        )));
    }
    if n_resamples < 2 {
        return Err(FreesError::evaluation(
            "bootstrap_ci: at least 2 resamples are required.",
        ));
    }
    let n = x.len();
    let mut rng = JavaRandom::new(seed);
    let mut means = Vec::with_capacity(n_resamples);
    for _ in 0..n_resamples {
        let mut sum = 0.0_f64;
        for _ in 0..n {
            let idx = (rng.next_double() * n as f64) as usize;
            // Clamp index just in case of rounding (next_double is in [0, 1)).
            sum += x[idx.min(n - 1)];
        }
        means.push(sum / n as f64);
    }
    means.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let alpha = 1.0 - confidence;
    let lo_idx = libm::floor(alpha / 2.0 * means.len() as f64) as usize;
    let hi_idx = libm::ceil((1.0 - alpha / 2.0) * means.len() as f64) as usize;
    let lo_idx = lo_idx.min(means.len() - 1);
    let hi_idx = hi_idx.min(means.len() - 1);
    Ok((means[lo_idx], means[hi_idx]))
}

/// Permutation test for equal means of two independent samples.
///
/// Computes the observed difference in means `|x̄ − ȳ|`, then randomly
/// reassigns labels `n_perms` times and counts how many permutations produce
/// a difference at least as extreme.  The p-value is `(count + 1) / (n_perms + 1)`.
pub fn permutation_test_means(
    n_perms: usize,
    seed: i64,
    x: &[f64],
    y: &[f64],
) -> Result<TestResult> {
    if x.is_empty() || y.is_empty() {
        return Err(FreesError::evaluation(
            "permtest: both samples must be non-empty.",
        ));
    }
    if n_perms < 1 {
        return Err(FreesError::evaluation(
            "permtest: at least 1 permutation is required.",
        ));
    }
    let nx = x.len();
    let mut combined: Vec<f64> = Vec::with_capacity(nx + y.len());
    combined.extend_from_slice(x);
    combined.extend_from_slice(y);
    let n_total = combined.len();

    let obs_diff =
        libm::fabs(x.iter().sum::<f64>() / nx as f64 - y.iter().sum::<f64>() / y.len() as f64);

    let mut rng = JavaRandom::new(seed);
    let mut count = 0usize;
    for _ in 0..n_perms {
        // Fisher–Yates shuffle using the seeded RNG.
        fisher_yates_shuffle(&mut combined, &mut rng);
        let perm_mean_x = combined[..nx].iter().sum::<f64>() / nx as f64;
        let perm_mean_y = combined[nx..].iter().sum::<f64>() / (n_total - nx) as f64;
        if libm::fabs(perm_mean_x - perm_mean_y) >= obs_diff {
            count += 1;
        }
    }

    Ok(TestResult {
        statistic: obs_diff,
        df: f64::NAN, // not applicable
        p_value: (count + 1) as f64 / (n_perms + 1) as f64,
    })
}

fn fisher_yates_shuffle(v: &mut [f64], rng: &mut JavaRandom) {
    let n = v.len();
    for i in (1..n).rev() {
        let j = (rng.next_double() * (i + 1) as f64) as usize;
        let j = j.min(i);
        v.swap(i, j);
    }
}

// ── internal: median helper (reuses eval.rs convention) ───────────────────

fn median_of(v: &[f64]) -> f64 {
    let sorted = sort_finite(v);
    let n = sorted.len();
    if n == 0 {
        return f64::NAN;
    }
    if n % 2 == 1 {
        sorted[n / 2]
    } else {
        0.5 * (sorted[n / 2 - 1] + sorted[n / 2])
    }
}

// ── Two-sided p-value helper ──────────────────────────────────────────────

fn two_sided_t_pvalue(t: f64, df: f64, t_cdf: fn(f64, f64) -> Result<f64>) -> Result<f64> {
    let abs_t = libm::fabs(t);
    let tail = 1.0 - t_cdf(abs_t, df)?;
    Ok(2.0 * tail)
}

// ── JavaRandom (re-export from monte carlo) ───────────────────────────────
// We reproduce the same LCG here to keep this module self-contained and
// avoid a circular dependency, since montecarlo.rs is in the `analysis`
// module.  The implementation is a 48-bit LCG matching java.util.Random
// bit for bit (see `montecarlo.rs` for the derivation and test oracle).

/// Bit-exact port of `java.util.Random` for seeded reproducibility.
struct JavaRandom {
    seed: i64,
    have_next_gaussian: bool,
    next_gaussian: f64,
}

impl JavaRandom {
    fn new(seed: i64) -> Self {
        Self {
            seed: (seed ^ 0x5DEECE66D_i64) & ((1_i64 << 48) - 1),
            have_next_gaussian: false,
            next_gaussian: 0.0,
        }
    }

    fn next(&mut self, bits: u32) -> i32 {
        self.seed =
            (self.seed.wrapping_mul(0x5DEECE66D_i64).wrapping_add(0xB)) & ((1_i64 << 48) - 1);
        (self.seed >> (48 - bits)) as i32
    }

    fn next_double(&mut self) -> f64 {
        let hi = self.next(26) as i64;
        let lo = self.next(27) as i64;
        (hi * (1_i64 << 27) + lo) as f64 / ((1_i64 << 53) as f64)
    }

    #[allow(dead_code)]
    fn next_gaussian(&mut self) -> f64 {
        if self.have_next_gaussian {
            self.have_next_gaussian = false;
            return self.next_gaussian;
        }
        loop {
            let v1 = 2.0 * self.next_double() - 1.0;
            let v2 = 2.0 * self.next_double() - 1.0;
            let s = v1 * v1 + v2 * v2;
            if s < 1.0 && s != 0.0 {
                let mul = libm::sqrt(-2.0 * libm::log(s) / s);
                self.next_gaussian = v2 * mul;
                self.have_next_gaussian = true;
                return v1 * mul;
            }
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64, tol: f64) {
        assert!(
            (a - b).abs() <= tol * b.abs().max(1.0),
            "expected {b}, got {a} (tol {tol})"
        );
    }

    // -- weighted mean/variance -----------------------------------------------

    #[test]
    fn weighted_mean_uniform_weights_equals_arithmetic_mean() {
        let w = [1.0, 1.0, 1.0, 1.0];
        let x = [1.0, 2.0, 3.0, 4.0];
        close(weighted_mean(&w, &x).unwrap(), 2.5, 1e-12);
    }

    #[test]
    fn weighted_mean_known_values() {
        // wmean([2, 1, 3], [10, 20, 30]) = (20 + 20 + 90) / 6 = 130/6
        let w = [2.0, 1.0, 3.0];
        let x = [10.0, 20.0, 30.0];
        close(weighted_mean(&w, &x).unwrap(), 130.0 / 6.0, 1e-12);
    }

    #[test]
    fn weighted_variance_uniform_weights_matches_sample_variance() {
        let w = [1.0, 1.0, 1.0, 1.0, 1.0];
        let x = [1.0, 2.0, 3.0, 4.0, 5.0];
        // sample variance = 2.5
        close(weighted_variance(&w, &x).unwrap(), 2.5, 1e-12);
    }

    #[test]
    fn weighted_mean_rejects_mismatched() {
        assert!(weighted_mean(&[1.0], &[1.0, 2.0]).is_err());
    }

    #[test]
    fn weighted_variance_rejects_single_value() {
        assert!(weighted_variance(&[1.0], &[5.0]).is_err());
    }

    // -- robust summaries -----------------------------------------------------

    #[test]
    fn mad_of_symmetric_data() {
        // [1, 2, 3, 4, 5]: median = 3, deviations = [2, 1, 0, 1, 2], MAD = 1
        close(
            median_abs_deviation(&[1.0, 2.0, 3.0, 4.0, 5.0]).unwrap(),
            1.0,
            1e-12,
        );
    }

    #[test]
    fn trimmed_mean_zero_trim_is_plain_mean() {
        close(
            trimmed_mean(0.0, &[1.0, 2.0, 3.0, 4.0, 5.0]).unwrap(),
            3.0,
            1e-12,
        );
    }

    #[test]
    fn trimmed_mean_twenty_percent() {
        // 20% trim of [1, 2, 3, 4, 5] = trim 1 from each end => mean([2, 3, 4]) = 3
        close(
            trimmed_mean(0.2, &[1.0, 2.0, 3.0, 4.0, 5.0]).unwrap(),
            3.0,
            1e-12,
        );
    }

    #[test]
    fn skewness_of_symmetric_data_is_zero() {
        close(skewness(&[1.0, 2.0, 3.0, 4.0, 5.0]).unwrap(), 0.0, 1e-12);
    }

    #[test]
    fn skewness_known_value() {
        // Bias-corrected (Fisher) skewness of [1, 2, 3, 4, 100]:
        // g1 ≈ 1.4975, G1 = g1 * sqrt(n(n-1)) / (n-2) ≈ 2.2324
        let val = skewness(&[1.0, 2.0, 3.0, 4.0, 100.0]).unwrap();
        close(val, 2.2324, 1e-3);
    }

    #[test]
    fn kurtosis_of_normal_like_symmetric_is_negative() {
        // Bias-corrected excess kurtosis of [1, 2, 3, 4, 5]:
        // g2 = -1.3, G2 = ((n-1)/((n-2)(n-3)))*((n+1)*g2 + 6) = -1.2
        let val = kurtosis(&[1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();
        close(val, -1.2, 1e-10);
    }

    #[test]
    fn skewness_rejects_too_few() {
        assert!(skewness(&[1.0, 2.0]).is_err());
    }

    #[test]
    fn kurtosis_rejects_too_few() {
        assert!(kurtosis(&[1.0, 2.0, 3.0]).is_err());
    }

    // -- covariance & correlation ---------------------------------------------

    #[test]
    fn covariance_of_identical_vectors_is_variance() {
        let x = [1.0, 2.0, 3.0, 4.0, 5.0];
        close(sample_covariance(&x, &x).unwrap(), 2.5, 1e-12);
    }

    #[test]
    fn covariance_known_values() {
        // cov([1, 2, 3], [4, 5, 6]) = 1.0
        close(
            sample_covariance(&[1.0, 2.0, 3.0], &[4.0, 5.0, 6.0]).unwrap(),
            1.0,
            1e-12,
        );
    }

    #[test]
    fn pearson_perfect_positive() {
        close(
            pearson_correlation(&[1.0, 2.0, 3.0], &[2.0, 4.0, 6.0]).unwrap(),
            1.0,
            1e-12,
        );
    }

    #[test]
    fn pearson_perfect_negative() {
        close(
            pearson_correlation(&[1.0, 2.0, 3.0], &[6.0, 4.0, 2.0]).unwrap(),
            -1.0,
            1e-12,
        );
    }

    #[test]
    fn spearman_monotonic_positive() {
        close(
            spearman_correlation(&[1.0, 2.0, 3.0], &[10.0, 100.0, 1000.0]).unwrap(),
            1.0,
            1e-12,
        );
    }

    #[test]
    fn spearman_with_ties() {
        // [1, 2, 2, 3] vs [1, 2, 3, 4]: Spearman via Pearson on fractional ranks.
        let r = spearman_correlation(&[1.0, 2.0, 2.0, 3.0], &[1.0, 2.0, 3.0, 4.0]).unwrap();
        assert!(
            r > 0.9 && r <= 1.0,
            "expected high positive Spearman, got {r}"
        );
    }

    // -- hypothesis tests (use a mock t_cdf / f_cdf) --------------------------

    /// Approximate Student-t CDF for testing: uses the regularized beta identity.
    /// This is the same formula we implement in eval.rs.
    fn mock_t_cdf(x: f64, df: f64) -> Result<f64> {
        // For large df, approximate as normal.
        if df > 1000.0 {
            Ok(0.5 * libm::erfc(-x / std::f64::consts::SQRT_2))
        } else {
            let t2 = x * x;
            let beta_val = regularized_beta_approx(df / (df + t2), df / 2.0, 0.5);
            if x >= 0.0 {
                Ok(1.0 - 0.5 * beta_val)
            } else {
                Ok(0.5 * beta_val)
            }
        }
    }

    fn mock_t_inv(p: f64, df: f64) -> Result<f64> {
        // Simple bisection for tests.
        if p <= 0.0 {
            return Ok(f64::NEG_INFINITY);
        }
        if p >= 1.0 {
            return Ok(f64::INFINITY);
        }
        let mut lo = -100.0_f64;
        let mut hi = 100.0_f64;
        for _ in 0..200 {
            let mid = 0.5 * (lo + hi);
            let cdf = mock_t_cdf(mid, df)?;
            if cdf < p {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        Ok(0.5 * (lo + hi))
    }

    fn mock_f_cdf(x: f64, d1: f64, d2: f64) -> Result<f64> {
        if x <= 0.0 {
            return Ok(0.0);
        }
        let z = d1 * x / (d1 * x + d2);
        Ok(regularized_beta_approx(z, d1 / 2.0, d2 / 2.0))
    }

    /// Simple regularized beta via continued fraction (for test mocks).
    fn regularized_beta_approx(x: f64, a: f64, b: f64) -> f64 {
        if x <= 0.0 {
            return 0.0;
        }
        if x >= 1.0 {
            return 1.0;
        }
        let flip = x > (a + 1.0) / (a + b + 2.0);
        let (xx, aa, bb) = if flip { (1.0 - x, b, a) } else { (x, a, b) };
        let pref = libm::exp(
            aa * libm::log(xx) + bb * libm::log(1.0 - xx) - libm::lgamma(aa) - libm::lgamma(bb)
                + libm::lgamma(aa + bb),
        );
        // Continued fraction coefficients for I_x(a, b).
        let mut c = 1.0_f64;
        let mut d = 1.0 - (aa + bb) * xx / (aa + 1.0);
        if libm::fabs(d) < 1e-30 {
            d = 1e-30;
        }
        d = 1.0 / d;
        let mut h = d;
        for m in 1..=200 {
            let mf = m as f64;
            // Even step.
            let num_even = mf * (bb - mf) * xx / ((aa + 2.0 * mf - 1.0) * (aa + 2.0 * mf));
            d = 1.0 + num_even * d;
            if libm::fabs(d) < 1e-30 {
                d = 1e-30;
            }
            c = 1.0 + num_even / c;
            if libm::fabs(c) < 1e-30 {
                c = 1e-30;
            }
            d = 1.0 / d;
            h *= d * c;
            // Odd step.
            let num_odd =
                -((aa + mf) * (aa + bb + mf) * xx) / ((aa + 2.0 * mf) * (aa + 2.0 * mf + 1.0));
            d = 1.0 + num_odd * d;
            if libm::fabs(d) < 1e-30 {
                d = 1e-30;
            }
            c = 1.0 + num_odd / c;
            if libm::fabs(c) < 1e-30 {
                c = 1e-30;
            }
            d = 1.0 / d;
            let delta = d * c;
            h *= delta;
            if libm::fabs(delta - 1.0) < 1e-12 {
                break;
            }
        }
        let val = pref * h / aa;
        if flip {
            1.0 - val
        } else {
            val
        }
    }

    #[test]
    fn ttest_one_sample_known_values() {
        // Test against mu0 = 0 with data [1, 2, 3, 4, 5].
        // mean = 3, s = sqrt(2.5), se = sqrt(2.5/5) = sqrt(0.5)
        // t = 3 / sqrt(0.5) ≈ 4.2426, df = 4
        let r = ttest_one_sample(0.0, &[1.0, 2.0, 3.0, 4.0, 5.0], mock_t_cdf).unwrap();
        close(r.statistic, 3.0 / libm::sqrt(0.5), 1e-10);
        close(r.df, 4.0, 1e-12);
        assert!(
            r.p_value < 0.02,
            "expected small p-value, got {}",
            r.p_value
        );
    }

    #[test]
    fn ttest_paired_zero_difference() {
        // Identical data should give t ≈ 0, p ≈ 1.
        let x = [1.0, 2.0, 3.0, 4.0, 5.0];
        let r = ttest_paired(&x, &x, mock_t_cdf).unwrap();
        close(r.statistic, 0.0, 1e-12);
        close(r.p_value, 1.0, 1e-6);
    }

    #[test]
    fn ttest_welch_separated_samples() {
        let x = [10.0, 11.0, 12.0, 13.0, 14.0];
        let y = [0.0, 1.0, 2.0, 3.0, 4.0];
        let r = ttest_welch(&x, &y, mock_t_cdf).unwrap();
        assert!(r.statistic > 5.0, "expected large t, got {}", r.statistic);
        assert!(r.p_value < 0.001, "expected tiny p, got {}", r.p_value);
    }

    #[test]
    fn anova_all_same_mean_gives_large_p() {
        let a = [1.0, 2.0, 3.0];
        let b = [1.5, 2.5, 3.5];
        let c = [0.5, 1.5, 2.5];
        let r = anova_oneway(&[&a, &b, &c], mock_f_cdf).unwrap();
        // Group means are similar; F should be small and p large.
        assert!(r.p_value > 0.1, "expected large p, got {}", r.p_value);
    }

    #[test]
    fn anova_well_separated_gives_small_p() {
        let a = [1.0, 1.1, 0.9, 1.0];
        let b = [100.0, 100.1, 99.9, 100.0];
        let r = anova_oneway(&[&a, &b], mock_f_cdf).unwrap();
        assert!(r.p_value < 0.001, "expected tiny p, got {}", r.p_value);
    }

    #[test]
    fn chi2_gof_uniform_expectation() {
        // Observed [10, 10, 10, 10], expected [10, 10, 10, 10]: chi2 = 0, p ≈ 1.
        let obs = [10.0, 10.0, 10.0, 10.0];
        let exp = [10.0, 10.0, 10.0, 10.0];
        fn chi2_cdf_mock(x: f64, df: f64) -> Result<f64> {
            // regularized gamma P (same as chi_square in eval.rs).
            if x <= 0.0 {
                return Ok(0.0);
            }
            // Use a rough approximation: for small chi2, P ≈ 0.
            let a = df / 2.0;
            let xh = x / 2.0;
            // Series expansion of the regularized gamma.
            let mut sum = 1.0 / a;
            let mut term = 1.0 / a;
            for n in 1..200 {
                term *= xh / (a + n as f64);
                sum += term;
                if libm::fabs(term / sum) < 1e-15 {
                    break;
                }
            }
            Ok(libm::exp(-xh + a * libm::log(xh) - libm::lgamma(a)) * sum)
        }
        let r = chi2_goodness_of_fit(&obs, &exp, chi2_cdf_mock).unwrap();
        close(r.statistic, 0.0, 1e-12);
        close(r.p_value, 1.0, 1e-6);
    }

    // -- CI -------------------------------------------------------------------

    #[test]
    fn ci_mean_contains_true_mean() {
        let x = [1.0, 2.0, 3.0, 4.0, 5.0];
        let (lo, hi) = ci_mean(0.95, &x, mock_t_inv).unwrap();
        assert!(lo < 3.0 && hi > 3.0, "CI [{lo}, {hi}] should contain 3.0");
    }

    // -- bootstrap ------------------------------------------------------------

    #[test]
    fn bootstrap_ci_is_reproducible() {
        let x = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let (lo1, hi1) = bootstrap_ci_mean(0.95, 1000, 42, &x).unwrap();
        let (lo2, hi2) = bootstrap_ci_mean(0.95, 1000, 42, &x).unwrap();
        assert_eq!(lo1, lo2);
        assert_eq!(hi1, hi2);
    }

    #[test]
    fn bootstrap_ci_brackets_true_mean() {
        let x = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let (lo, hi) = bootstrap_ci_mean(0.90, 5000, 123, &x).unwrap();
        assert!(lo < 5.5 && hi > 5.5, "CI [{lo}, {hi}] should contain 5.5");
    }

    // -- permutation test -----------------------------------------------------

    #[test]
    fn permtest_same_distribution_large_p() {
        let x = [1.0, 2.0, 3.0, 4.0, 5.0];
        let y = [1.5, 2.5, 3.5, 4.5, 5.5];
        let r = permutation_test_means(999, 42, &x, &y).unwrap();
        assert!(r.p_value > 0.1, "expected large p, got {}", r.p_value);
    }

    #[test]
    fn permtest_different_distribution_small_p() {
        let x = [1.0, 1.1, 0.9, 1.0, 1.05];
        let y = [100.0, 100.1, 99.9, 100.0, 100.05];
        let r = permutation_test_means(999, 42, &x, &y).unwrap();
        assert!(r.p_value < 0.01, "expected small p, got {}", r.p_value);
    }

    #[test]
    fn permtest_is_reproducible() {
        let x = [1.0, 2.0, 3.0];
        let y = [4.0, 5.0, 6.0];
        let r1 = permutation_test_means(500, 7, &x, &y).unwrap();
        let r2 = permutation_test_means(500, 7, &x, &y).unwrap();
        assert_eq!(r1.p_value, r2.p_value);
    }

    // -- fractional ranks -----------------------------------------------------

    #[test]
    fn fractional_ranks_no_ties() {
        let ranks = fractional_ranks(&[30.0, 10.0, 20.0]);
        close(ranks[0], 3.0, 1e-12); // 30 is largest
        close(ranks[1], 1.0, 1e-12); // 10 is smallest
        close(ranks[2], 2.0, 1e-12); // 20 is middle
    }

    #[test]
    fn fractional_ranks_with_ties() {
        let ranks = fractional_ranks(&[10.0, 20.0, 20.0, 30.0]);
        close(ranks[0], 1.0, 1e-12);
        close(ranks[1], 2.5, 1e-12); // tie
        close(ranks[2], 2.5, 1e-12); // tie
        close(ranks[3], 4.0, 1e-12);
    }
}
