//! Phase 4.3 end-to-end: `DistributionOf(X) = …` and `Correlation(A, B) = ρ`
//! from the document text through extraction, first-order propagation and the
//! Monte Carlo sampler.
//!
//! The propagated numbers here are all closed-form — `y = a + b` makes
//! `σ_y² = σ_a² + σ_b² + 2ρσ_aσ_b` exactly, with no linearization error — so a
//! wrong sign, a dropped cross term or a matrix indexed in the wrong order all
//! show up as a plainly wrong number rather than a plausible one.

use frees_core::analysis::distributions::Distribution;
use frees_core::analysis::montecarlo::{self, SamplingOptions};
use frees_core::analysis::sampling::Design;
use frees_core::analysis::uncertainty::UncertaintySpec;
use frees_core::engine::solve_with_parametric_tables;
use frees_core::solver::SolverSettings;
use std::collections::BTreeMap;

fn solve(document: &str) -> BTreeMap<String, f64> {
    solve_with_parametric_tables(document, &SolverSettings::default(), &[], None, &[])
        .unwrap_or_else(|e| panic!("solve failed: {e}\n---\n{document}"))
        .values
}

fn error(document: &str) -> String {
    solve_with_parametric_tables(document, &SolverSettings::default(), &[], None, &[])
        .expect_err("expected this document to be refused")
        .to_string()
}

/// `y = a + b` with the two sources' sigmas, plus whatever declarations.
fn sum_document(extra: &str) -> String {
    format!(
        "a = 2\nb = 3\ny = a + b\nUncertaintyOf(a) = 0.1\nUncertaintyOf(b) = 0.2\n\
         {extra}uy = UncertaintyOf(y)\n"
    )
}

#[test]
fn a_declared_correlation_changes_the_propagated_sigma_by_the_cross_term() {
    // Independent: sqrt(0.01 + 0.04).
    let values = solve(&sum_document(""));
    assert!(
        (values["uy"] - 0.05f64.sqrt()).abs() < 1e-12,
        "{}",
        values["uy"]
    );

    for (rho, expected) in [
        (1.0, 0.3),            // fully correlated: the sigmas add
        (-1.0, 0.1),           // anti-correlated: they subtract
        (0.5, 0.07f64.sqrt()), // 0.01 + 0.04 + 2·0.5·0.1·0.2
        (0.0, 0.05f64.sqrt()), // an explicit zero is the independent case
    ] {
        let values = solve(&sum_document(&format!("Correlation(a, b) = {rho}\n")));
        assert!(
            (values["uy"] - expected).abs() < 1e-12,
            "rho={rho}: got {}, wanted {expected}",
            values["uy"]
        );
    }
}

#[test]
fn correlation_reads_the_pair_in_either_order() {
    let forward = solve(&sum_document("Correlation(a, b) = 0.5\n"));
    let reverse = solve(&sum_document("Correlation(b, a) = 0.5\n"));
    assert_eq!(forward["uy"], reverse["uy"]);
}

#[test]
fn a_distribution_supplies_the_sigma_its_shape_implies() {
    // Uniform(0.9, 1.1) has sigma = 0.2/sqrt(12); y = 2d doubles it.
    let values =
        solve("d = 1\ny = 2 * d\nDistributionOf(d) = Uniform(0.9, 1.1)\nuy = UncertaintyOf(y)\n");
    let expected = 2.0 * 0.2 / 12f64.sqrt();
    assert!(
        (values["uy"] - expected).abs() < 1e-12,
        "got {}, wanted {expected}",
        values["uy"]
    );
    // And the source itself reports the shape's own standard deviation.
    assert!((values["uncertaintyof$d"] - 0.2 / 12f64.sqrt()).abs() < 1e-12);
}

#[test]
fn distribution_arguments_may_reference_solved_values() {
    let values = solve(
        "w = 0.25\nd = 1\ny = 2 * d\nDistributionOf(d) = Normal(1, w)\nuy = UncertaintyOf(y)\n",
    );
    assert!((values["uy"] - 0.5).abs() < 1e-12, "{}", values["uy"]);
}

#[test]
fn malformed_declarations_are_refused_with_their_own_message() {
    for (extra, expected) in [
        ("Correlation(a, b) = 1.5\n", "outside [-1, 1]"),
        ("Correlation(a, a) = 0.5\n", "correlation with itself"),
        ("Correlation(a, zz) = 0.5\n", "not an uncertainty source"),
        // Pairwise legal, jointly impossible.
        (
            "c = 1\nUncertaintyOf(c) = 0.3\nCorrelation(a, b) = 0.95\n\
             Correlation(a, c) = 0.95\nCorrelation(b, c) = -0.95\n",
            "positive semidefinite",
        ),
    ] {
        let message = error(&sum_document(extra));
        assert!(
            message.contains(expected),
            "wanted `{expected}` in: {message}"
        );
    }

    for (document, expected) in [
        (
            "d = 1\ny = d\nDistributionOf(d) = Cauchy(0, 1)\n",
            "Unknown distribution",
        ),
        (
            "d = 1\ny = d\nDistributionOf(d) = Uniform(1, 1)\n",
            "Uniform requires hi > lo",
        ),
        (
            "d = 1\ny = d\nUncertaintyOf(d) = 0.1\nDistributionOf(d) = Normal(1, 0.2)\n",
            "State the spread once",
        ),
    ] {
        let message = error(document);
        assert!(
            message.contains(expected),
            "wanted `{expected}` in: {message}"
        );
    }
}

#[test]
fn a_correlation_without_any_source_is_refused_rather_than_ignored() {
    let message = error("a = 2\nb = 3\ny = a + b\nCorrelation(a, b) = 0.5\n");
    assert!(
        message.contains("no uncertainty sources"),
        "unexpected message: {message}"
    );
}

// ── the sampler ─────────────────────────────────────────────────────────────

fn mc_specs() -> BTreeMap<String, UncertaintySpec> {
    BTreeMap::from([
        ("a".to_string(), UncertaintySpec::with_uncertainty(0.1)),
        ("b".to_string(), UncertaintySpec::with_uncertainty(0.2)),
    ])
}

const MC_DOC: &str = "a = 2\nb = 3\ny = a + b\n";

fn sigma_of(options: Option<&SamplingOptions>, samples: usize) -> f64 {
    let outcome = montecarlo::run_with_tables_and_base(
        MC_DOC,
        &SolverSettings::default(),
        &mc_specs(),
        &BTreeMap::new(),
        samples,
        7,
        || false,
        &[],
        None,
        options,
    )
    .expect("monte carlo");
    outcome
        .stats
        .iter()
        .find(|s| s.variable == "y")
        .expect("y in stats")
        .sigma
}

#[test]
fn the_default_sampler_is_untouched_by_the_new_options() {
    // `None` and an all-default `SamplingOptions` must both take the legacy
    // path, and therefore agree exactly — not approximately.
    let legacy = sigma_of(None, 200);
    let explicit_default = sigma_of(Some(&SamplingOptions::default()), 200);
    assert_eq!(legacy, explicit_default);
}

#[test]
fn correlated_sampling_reproduces_the_first_order_sigma() {
    let mut options = SamplingOptions {
        design: Design::LatinHypercube,
        ..SamplingOptions::default()
    };
    options.correlations.insert(("a".into(), "b".into()), 1.0);
    options.distributions.insert(
        "a".into(),
        Distribution::Normal {
            mean: 2.0,
            sigma: 0.1,
        },
    );
    options.distributions.insert(
        "b".into(),
        Distribution::Normal {
            mean: 3.0,
            sigma: 0.2,
        },
    );
    // Fully correlated sums add their sigmas: 0.1 + 0.2 = 0.3.
    let sigma = sigma_of(Some(&options), 400);
    assert!((sigma - 0.3).abs() < 0.02, "sampled sigma {sigma}");
}

#[test]
fn a_uniform_marginal_samples_inside_its_own_support() {
    let mut options = SamplingOptions {
        design: Design::Sobol,
        ..SamplingOptions::default()
    };
    options
        .distributions
        .insert("a".into(), Distribution::Uniform { lo: 1.0, hi: 3.0 });
    options.distributions.insert(
        "b".into(),
        Distribution::Normal {
            mean: 3.0,
            sigma: 0.2,
        },
    );
    let outcome = montecarlo::run_with_tables_and_base(
        MC_DOC,
        &SolverSettings::default(),
        &mc_specs(),
        &BTreeMap::new(),
        256,
        7,
        || false,
        &[],
        None,
        Some(&options),
    )
    .expect("monte carlo");
    for sample in outcome.samples.iter().filter(|s| s.success) {
        let a = sample.values["a"];
        assert!(
            (1.0..=3.0).contains(&a),
            "a sampled outside its support: {a}"
        );
    }
    // Uniform(1, 3) has sigma = 2/sqrt(12) = 0.5774.
    let sigma = outcome
        .stats
        .iter()
        .find(|s| s.variable == "a")
        .unwrap()
        .sigma;
    assert!((sigma - 2.0 / 12f64.sqrt()).abs() < 0.03, "sigma {sigma}");
    // …and the diagnostics say plainly that this is not an i.i.d. design.
    assert_eq!(outcome.diagnostics.design, "sobol");
    assert!(!outcome.diagnostics.iid_standard_error_applies);
    assert!(outcome.diagnostics.design_complete);
}

#[test]
fn requested_output_quantiles_come_back_in_order() {
    let options = SamplingOptions {
        design: Design::LatinHypercube,
        quantiles: vec![0.1, 0.9, 42.0],
        ..SamplingOptions::default()
    };
    let outcome = montecarlo::run_with_tables_and_base(
        MC_DOC,
        &SolverSettings::default(),
        &mc_specs(),
        &BTreeMap::new(),
        200,
        7,
        || false,
        &[],
        None,
        Some(&options),
    )
    .expect("monte carlo");
    let y = outcome.stats.iter().find(|s| s.variable == "y").unwrap();
    // The out-of-range 42 is dropped, not reported as a value.
    assert_eq!(y.quantiles.len(), 2);
    assert_eq!(y.quantiles[0].0, 0.1);
    assert_eq!(y.quantiles[1].0, 0.9);
    assert!(y.quantiles[0].1 < y.quantiles[1].1);
}

#[test]
fn a_non_gaussian_marginal_under_correlation_is_refused() {
    let mut options = SamplingOptions::default();
    options.correlations.insert(("a".into(), "b".into()), 0.5);
    options
        .distributions
        .insert("a".into(), Distribution::Uniform { lo: 1.0, hi: 3.0 });
    let message = montecarlo::run_with_tables_and_base(
        MC_DOC,
        &SolverSettings::default(),
        &mc_specs(),
        &BTreeMap::new(),
        50,
        7,
        || false,
        &[],
        None,
        Some(&options),
    )
    .expect_err("should be refused")
    .to_string();
    assert!(message.contains("normal marginals only"), "{message}");
}

// ── Phase 4.6: the document driver ──────────────────────────────────────────

#[test]
fn sensitivity_over_a_document_ranks_its_inputs() {
    use frees_core::analysis::sampling::Design;
    use frees_core::analysis::sensitivity::{
        document_evaluator, input_space_from_document, sobol, SobolPlan,
    };
    use frees_core::engine::VariableOverride;

    // y = 10a + b: a owns 100/101 of the variance when both have sigma 1.
    let document = "a = 1\nb = 1\ny = 10 * a + b\n";
    let specs = BTreeMap::from([
        ("a".to_string(), UncertaintySpec::with_uncertainty(1.0)),
        ("b".to_string(), UncertaintySpec::with_uncertainty(1.0)),
    ]);
    let base =
        frees_core::engine::solve_with_tables(document, &SolverSettings::default(), &[], &[])
            .expect("base solve");

    let sources = vec!["a".to_string(), "b".to_string()];
    let space = input_space_from_document(
        &sources,
        &base.values,
        &specs,
        &BTreeMap::new(),
        &Default::default(),
    )
    .expect("input space");
    // No DistributionOf, so each source is a normal centred on its base value.
    assert_eq!(space.names, sources);
    assert!((space.marginals[0].std_dev() - 1.0).abs() < 1e-12);

    let sample_specs: Vec<VariableOverride> = Vec::new();
    let settings = SolverSettings::default();
    let plan = SobolPlan {
        space: space.clone(),
        base_samples: 256,
        seed: 2,
        design: Design::Sobol,
        bootstrap: 0,
    };
    let evaluate = document_evaluator(document, &settings, &sample_specs, &[], &space.names);
    let (outcomes, diagnostics) = sobol(&plan, evaluate, || false).expect("sobol");
    assert_eq!(diagnostics.evaluations, 256 * 4);
    assert_eq!(diagnostics.dropped_rows, 0);

    let y = outcomes
        .iter()
        .find(|o| o.output == "y")
        .expect("y among the outputs");
    let a = &y.indices[0];
    let b = &y.indices[1];
    assert!(
        (a.first_order - 100.0 / 101.0).abs() < 0.02,
        "S_a = {}",
        a.first_order
    );
    assert!(
        (b.first_order - 1.0 / 101.0).abs() < 0.02,
        "S_b = {}",
        b.first_order
    );
    // Additive model: no interaction, so total equals first order.
    assert!((a.total - a.first_order).abs() < 0.02);
}

#[test]
fn sensitivity_refuses_a_correlated_document() {
    use frees_core::analysis::sampling::CorrelationEntries;
    use frees_core::analysis::sensitivity::input_space_from_document;

    let mut correlations = CorrelationEntries::new();
    correlations.insert(("a".into(), "b".into()), 0.5);
    let message = input_space_from_document(
        &["a".to_string(), "b".to_string()],
        &BTreeMap::from([("a".to_string(), 1.0), ("b".to_string(), 1.0)]),
        &BTreeMap::from([
            ("a".to_string(), UncertaintySpec::with_uncertainty(1.0)),
            ("b".to_string(), UncertaintySpec::with_uncertainty(1.0)),
        ]),
        &BTreeMap::new(),
        &correlations,
    )
    .expect_err("correlated inputs must be refused")
    .to_string();
    assert!(
        message.contains("assume independent inputs"),
        "unexpected message: {message}"
    );
}

#[test]
fn a_document_with_no_sources_cannot_be_screened() {
    use frees_core::analysis::sensitivity::input_space_from_document;
    let message = input_space_from_document(
        &[],
        &BTreeMap::new(),
        &BTreeMap::new(),
        &BTreeMap::new(),
        &Default::default(),
    )
    .expect_err("no sources")
    .to_string();
    assert!(message.contains("at least one variable"), "{message}");
}

#[test]
fn a_stratified_design_refuses_an_unbounded_sample_count() {
    // The legacy path grows amortised and lets the budget stop it; a stratified
    // design cannot, so it must refuse rather than allocate n x p doubles.
    let options = SamplingOptions {
        design: Design::LatinHypercube,
        ..SamplingOptions::default()
    };
    let message = montecarlo::run_with_tables_and_base(
        MC_DOC,
        &SolverSettings::default(),
        &mc_specs(),
        &BTreeMap::new(),
        1_000_000_000,
        7,
        || true, // the budget fires immediately; the refusal must come first
        &[],
        None,
        Some(&options),
    )
    .expect_err("an out-of-range stratified design must be refused")
    .to_string();
    assert!(message.contains("capped at"), "{message}");
}
