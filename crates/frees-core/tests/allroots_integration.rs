use frees_core::engine::{solve_all, solve_all_with_tables, PreparedDocument};
use frees_core::solver::SolverSettings;

#[test]
fn test_solve_all_single_variable_quadratic() {
    let model = "x^2 = 4\n";
    let settings = SolverSettings::default();
    let solutions = solve_all(model, &settings).expect("solve_all should succeed");
    assert_eq!(solutions.len(), 2, "Expected 2 solutions for x^2 = 4");
    let roots: Vec<f64> = solutions.iter().map(|s| s.values["x"]).collect();
    assert!((roots[0] - (-2.0)).abs() < 1e-6);
    assert!((roots[1] - 2.0).abs() < 1e-6);
}

#[test]
fn test_solve_all_multi_variable_system() {
    let model = "x^2 = 4\ny = x + 10\n";
    let settings = SolverSettings::default();
    let solutions = solve_all(model, &settings).expect("solve_all should succeed");
    assert_eq!(solutions.len(), 2, "Expected 2 solutions");
    assert!((solutions[0].values["x"] - (-2.0)).abs() < 1e-6);
    assert!((solutions[0].values["y"] - 8.0).abs() < 1e-6);
    assert!((solutions[1].values["x"] - 2.0).abs() < 1e-6);
    assert!((solutions[1].values["y"] - 12.0).abs() < 1e-6);
}

#[test]
fn test_prepared_document_solve_all_with_pins() {
    let model = "x^2 = c\ny = x + 10\n";
    let settings = SolverSettings::default();
    let mut prep = PreparedDocument::new(model, &settings, &[], &[]).expect("prep failed");

    // Pin c = 9 -> roots x = -3, 3
    let sols1 = prep
        .solve_all_with_pins(&[("c".to_string(), 9.0)], None)
        .expect("solve_all_with_pins failed");
    assert_eq!(sols1.len(), 2);
    assert!((sols1[0].values["x"] - (-3.0)).abs() < 1e-6);
    assert!((sols1[0].values["y"] - 7.0).abs() < 1e-6);
    assert!((sols1[1].values["x"] - 3.0).abs() < 1e-6);
    assert!((sols1[1].values["y"] - 13.0).abs() < 1e-6);

    // Pin c = 16 -> in-place solve_all, roots x = -4, 4
    let sols2 = prep
        .solve_all_with_pins(&[("c".to_string(), 16.0)], None)
        .expect("solve_all_with_pins failed");
    assert_eq!(sols2.len(), 2);
    assert!((sols2[0].values["x"] - (-4.0)).abs() < 1e-6);
    assert!((sols2[0].values["y"] - 6.0).abs() < 1e-6);
    assert!((sols2[1].values["x"] - 4.0).abs() < 1e-6);
    assert!((sols2[1].values["y"] - 14.0).abs() < 1e-6);
}

#[test]
fn test_solve_all_with_tables() {
    let model = "x^2 = 25\n";
    let settings = SolverSettings::default();
    let solutions = solve_all_with_tables(model, &settings, &[], &[])
        .expect("solve_all_with_tables should succeed");
    assert_eq!(solutions.len(), 2);
    assert!((solutions[0].values["x"] - (-5.0)).abs() < 1e-6);
    assert!((solutions[1].values["x"] - 5.0).abs() < 1e-6);
}
