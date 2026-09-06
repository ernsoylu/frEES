use frees_core::analysis::optimizer::{optimize, Problem};
use frees_core::solver::SolverSettings;

fn make_problem(
    text: &str,
    objective: &str,
    decisions: &[&str],
    lowers: &[f64],
    uppers: &[f64],
    constraints: &[&str],
) -> Problem {
    Problem {
        text: text.to_string(),
        settings: SolverSettings::default(),
        overrides: Vec::new(),
        objective: objective.to_string(),
        decisions: decisions.iter().map(|s| (*s).to_string()).collect(),
        lowers: lowers.to_vec(),
        uppers: uppers.to_vec(),
        method: Some("nelder-mead".to_string()),
        maximize: false,
        constraints: constraints.iter().map(|s| (*s).to_string()).collect(),
    }
}

#[test]
fn multi_constraint_three_inequalities() {
    let p = make_problem(
        "f = (x - 10)^2 + (y - 10)^2 + (z - 10)^2\n",
        "f",
        &["x", "y", "z"],
        &[0.0, 0.0, 0.0],
        &[15.0, 15.0, 15.0],
        &["x + y <= 8", "y + z <= 8", "x >= 1"],
    );
    let result = optimize(&p).unwrap();
    assert!(result.warning.is_none(), "{:?}", result.warning);
    let x = result.decision_values[0];
    let y = result.decision_values[1];
    let z = result.decision_values[2];
    assert!(x + y <= 8.0 + 1e-2, "x + y = {}", x + y);
    assert!(y + z <= 8.0 + 1e-2, "y + z = {}", y + z);
    assert!(x >= 1.0 - 1e-2, "x = {}", x);
    assert!(result.solution.values.contains_key("x"));
    assert!(!result
        .solution
        .values
        .keys()
        .any(|k| k.starts_with("zz_opt_con_")));
    for name in result.solution.display_names.keys() {
        assert!(
            !name.starts_with("zz_opt_con_"),
            "display_name leak: {}",
            name
        );
    }
    assert!(result.evaluations > 0);
}

#[test]
fn mixed_equality_and_inequality_constraints() {
    let p = make_problem(
        "f = x^2 + y^2 + z^2\n",
        "f",
        &["x", "y", "z"],
        &[-5.0, -5.0, -5.0],
        &[5.0, 5.0, 5.0],
        &["x + y + z = 3", "x >= 0.5", "y >= 0.5"],
    );
    let result = optimize(&p).unwrap();
    assert!(result.warning.is_none(), "{:?}", result.warning);
    let x = result.decision_values[0];
    let y = result.decision_values[1];
    let z = result.decision_values[2];
    assert!(((x + y + z) - 3.0).abs() < 2e-2, "sum = {}", x + y + z);
    assert!(x >= 0.5 - 1e-2, "x = {}", x);
    assert!(y >= 0.5 - 1e-2, "y = {}", y);
    assert!(!result
        .solution
        .values
        .keys()
        .any(|k| k.starts_with("zz_opt_con_")));
}

#[test]
fn unsatisfiable_multi_constraints_emit_warning() {
    let p = make_problem(
        "y = x^2\n",
        "y",
        &["x"],
        &[0.0],
        &[10.0],
        &["x <= -10", "x >= 20"],
    );
    let result = optimize(&p).unwrap();
    let warning = result.warning.expect("should have warning");
    assert!(warning.contains("x <= -10"), "warning: {}", warning);
    assert!(warning.contains("x >= 20"), "warning: {}", warning);
    assert!(!result
        .solution
        .values
        .keys()
        .any(|k| k.starts_with("zz_opt_con_")));
}

#[test]
fn solve_count_reduction_benchmark() {
    let p = make_problem(
        "f = (x - 10)^2 + (y - 10)^2 + (z - 10)^2\n",
        "f",
        &["x", "y", "z"],
        &[0.0, 0.0, 0.0],
        &[15.0, 15.0, 15.0],
        &["x + y <= 8", "y + z <= 8", "x >= 1"],
    );
    let start = std::time::Instant::now();
    let result = optimize(&p).unwrap();
    let elapsed = start.elapsed();
    assert!(result.warning.is_none());
    let evals = result.evaluations;
    println!(
        "Phase 3 Multi-Constraint Benchmark: evals = {}, elapsed = {:?}",
        evals, elapsed
    );
    assert!(evals > 50, "evals = {}", evals);
}
