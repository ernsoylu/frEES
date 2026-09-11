//! End-to-end cover for the Phase 4.5 sensor CALLs and peak intrinsics.
//!
//! The kernels themselves are unit-tested inside `frees_core::signal`; what is
//! checked here is the whole route a document takes — flattening into
//! `detrend$`/`smooth$`/`window$`/`filter$`/`xcorr$`/`welch$` synthetics, the
//! evaluator decoding them, and the values landing on the right output element.
//! These have no Java oracle (the Java `SignalProcessing` stops at the DFT), so
//! the expected numbers are hand-derived, not harvested.

use frees_core::engine::solve_with_parametric_tables;
use frees_core::solver::SolverSettings;
use std::collections::BTreeMap;

fn solve(document: &str) -> BTreeMap<String, f64> {
    let settings = SolverSettings::default();
    let outcome = solve_with_parametric_tables(document, &settings, &[], None, &[])
        .unwrap_or_else(|e| panic!("solve failed: {e}\n---\n{document}"));
    outcome.values.into_iter().collect()
}

fn error(document: &str) -> String {
    let settings = SolverSettings::default();
    solve_with_parametric_tables(document, &settings, &[], None, &[])
        .expect_err("expected this document to be refused")
        .to_string()
}

/// `v[1..=n]`, 1-based exactly as the document writes it.
fn vector(values: &BTreeMap<String, f64>, name: &str, n: usize) -> Vec<f64> {
    (1..=n)
        .map(|i| {
            let key = format!("{name}[{i}]");
            *values
                .get(&key)
                .unwrap_or_else(|| panic!("{key} missing from {values:?}"))
        })
        .collect()
}

fn close(got: &[f64], want: &[f64], what: &str) {
    assert_eq!(got.len(), want.len(), "{what}: length");
    for (i, (g, w)) in got.iter().zip(want).enumerate() {
        assert!(
            (g - w).abs() <= 1e-9 * w.abs().max(1.0),
            "{what}[{i}]: got {g}, wanted {w}"
        );
    }
}

const SERIES: &str = "x = [1, 3, 2, 6, 4, 9, 5, 12]\n";

#[test]
fn detrend_defaults_to_linear_and_accepts_constant() {
    let values = solve(&format!(
        "{SERIES}CALL Detrend(x : d)\nCALL Detrend(x, 'constant' : c)\n"
    ));
    let d = vector(&values, "d", 8);
    let c = vector(&values, "c", 8);
    // A linear detrend leaves neither a mean nor a slope behind.
    assert!(d.iter().sum::<f64>().abs() < 1e-9, "residual mean in {d:?}");
    let t_mean = 3.5;
    let slope_num: f64 = d
        .iter()
        .enumerate()
        .map(|(j, v)| (j as f64 - t_mean) * v)
        .sum();
    assert!(slope_num.abs() < 1e-9, "residual slope in {d:?}");
    // A constant detrend removes the mean (5.25) and nothing else.
    close(
        &c,
        &[-4.25, -2.25, -3.25, 0.75, -1.25, 3.75, -0.25, 6.75],
        "constant detrend",
    );
}

#[test]
fn smooth_averages_over_the_samples_that_exist_at_the_ends() {
    let values = solve(&format!("{SERIES}CALL Smooth(x, 3 : s)\n"));
    close(
        &vector(&values, "s", 8),
        &[2.0, 2.0, 11.0 / 3.0, 4.0, 19.0 / 3.0, 6.0, 26.0 / 3.0, 8.5],
        "moving average",
    );
}

#[test]
fn window_tapers_the_series_and_names_are_case_folded() {
    let values = solve(&format!(
        "{SERIES}CALL Window(x, 'Hann' : w)\nCALL Window(x, 'rect' : r)\n"
    ));
    let w = vector(&values, "w", 8);
    // A symmetric Hann closes on zero at both ends.
    assert_eq!(w[0], 0.0);
    assert_eq!(w[7], 0.0);
    // Rect is the identity taper, so it must hand the series back untouched.
    close(
        &vector(&values, "r", 8),
        &[1.0, 3.0, 2.0, 6.0, 4.0, 9.0, 5.0, 12.0],
        "rect window",
    );
}

#[test]
fn filter_runs_the_difference_equation_and_filtfilt_is_symmetric() {
    let values = solve(&format!(
        "{SERIES}b = [0.5, 0.5]\na = [1]\nCALL Filter(b, a, x : y)\nCALL FiltFilt(b, a, x : z)\n"
    ));
    // y[j] = 0.5·x[j] + 0.5·x[j−1], starting from rest.
    close(
        &vector(&values, "y", 8),
        &[0.5, 2.0, 2.5, 4.0, 5.0, 6.5, 7.0, 8.5],
        "causal filter",
    );
    // Two passes of [0.5, 0.5] is the zero-phase 3-tap [0.25, 0.5, 0.25], and
    // the odd extension supplies 2·x[1] − x[2] = −1 before the first sample.
    let z = vector(&values, "z", 8);
    close(&z[..1], &[0.5 * 1.0 + 0.25 * 3.0 - 0.25], "zero-phase head");
    close(
        &z[7..],
        &[0.25 * 5.0 + 0.5 * 12.0 + 0.25 * (2.0 * 12.0 - 5.0)],
        "zero-phase tail",
    );
}

#[test]
fn xcorr_puts_zero_lag_at_the_centre() {
    let values = solve(&format!("{SERIES}CALL XCorr(x, x : r)\n"));
    let r = vector(&values, "r", 15);
    // The autocorrelation is symmetric and peaks at Σx² on the centre element.
    let energy: f64 = [1.0, 3.0, 2.0, 6.0, 4.0, 9.0, 5.0, 12.0]
        .iter()
        .map(|v| v * v)
        .sum();
    assert!((r[7] - energy).abs() < 1e-9, "centre {} vs {energy}", r[7]);
    for k in 1..=7 {
        assert!((r[7 - k] - r[7 + k]).abs() < 1e-9, "asymmetric at lag {k}");
    }
}

#[test]
fn welch_locates_a_tone_and_conserves_its_power() {
    // 32 samples of a 4-bin tone at fs = 32 Hz, i.e. exactly 4 Hz.
    let samples: Vec<String> = (0..32)
        .map(|j| {
            let v = 2f64.sqrt() * (2.0 * std::f64::consts::PI * 4.0 * j as f64 / 32.0).sin();
            format!("{v:.12}")
        })
        .collect();
    let document = format!(
        "x = [{}]\nCALL Welch(x, 32, 16 : f, pxx)\n",
        samples.join(", ")
    );
    let values = solve(&document);
    let f = vector(&values, "f", 9);
    let pxx = vector(&values, "pxx", 9);
    // Bin spacing is fs/nperseg = 2 Hz, so the 4 Hz tone owns f[3] = 4 Hz.
    close(
        &f,
        &[0.0, 2.0, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0, 16.0],
        "bins",
    );
    let peak = (0..9)
        .max_by(|&i, &j| pxx[i].partial_cmp(&pxx[j]).unwrap())
        .unwrap();
    assert_eq!(peak, 2, "peak landed at {} Hz: {pxx:?}", f[peak]);
    // Σ Pxx·Δf is the mean square: 1 for a 1 Vrms tone.
    let power: f64 = pxx.iter().sum::<f64>() * 2.0;
    assert!((power - 1.0).abs() < 0.05, "integrated power {power}");
}

#[test]
fn peak_intrinsics_count_and_locate_with_1_based_indices() {
    let values = solve(&format!(
        "{SERIES}\
         n = peakcount(0, 0, x)\n\
         p1 = peakindex(1, 0, 0, x)\n\
         p2 = peakindex(2, 0, 0, x)\n\
         p3 = peakindex(3, 0, 0, x)\n\
         p4 = peakindex(4, 0, 0, x)\n\
         tall = peakcount(5, 0, x)\n\
         spaced = peakcount(0, 4, x)\n"
    ));
    // Strict local maxima of [1,3,2,6,4,9,5,12] sit at 1-based 2, 4 and 6.
    assert_eq!(values["n"], 3.0);
    assert_eq!(values["p1"], 2.0);
    assert_eq!(values["p2"], 4.0);
    assert_eq!(values["p3"], 6.0);
    // Past the last peak the answer is 0, not an error and not the last one.
    assert_eq!(values["p4"], 0.0);
    // A height floor of 5 keeps only the 6 and the 9.
    assert_eq!(values["tall"], 2.0);
    // A separation of 4 keeps the tallest (9, at index 6) and then the 3.
    assert_eq!(values["spaced"], 2.0);
}

#[test]
fn peakindex_refuses_a_non_ordinal() {
    let message = error(&format!("{SERIES}p = peakindex(0, 0, 0, x)\n"));
    assert!(
        message.contains("positive whole number"),
        "unexpected message: {message}"
    );
}

#[test]
fn the_flatteners_report_their_own_shape_errors() {
    for (document, expected) in [
        (
            format!("{SERIES}CALL Smooth(x, 4 : s)\n"),
            "odd window length",
        ),
        (
            format!("{SERIES}CALL Smooth(x, 11 : s)\n"),
            "longer than the series",
        ),
        (
            format!("{SERIES}CALL Window(x, 'gaussian' : w)\n"),
            "Unknown window",
        ),
        (
            format!("{SERIES}CALL Detrend(x, 'quadratic' : d)\n"),
            "must be 'linear' or 'constant'",
        ),
        (
            format!("{SERIES}CALL Welch(x, 32, 64 : f, p)\n"),
            "exceeds the series length",
        ),
        (
            format!("{SERIES}CALL Welch(x, 32, 1 : f, p)\n"),
            "segment length of at least 2",
        ),
        (
            format!("{SERIES}CALL XCorr(x : r)\n"),
            "XCorr expects 2 input vectors",
        ),
    ] {
        let message = error(&document);
        assert!(
            message.contains(expected),
            "wanted `{expected}` in: {message}"
        );
    }
}

#[test]
fn a_declared_output_of_the_wrong_length_is_refused() {
    let message = error(&format!("{SERIES}CALL Detrend(x : d[1:4])\n"));
    assert!(
        message.contains("match the input length"),
        "unexpected message: {message}"
    );
}
