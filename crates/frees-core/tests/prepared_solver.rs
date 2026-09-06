use frees_core::engine::{solve_with_parametric_tables, PreparedDocument};
use frees_core::solver::SolverSettings;

#[test]
fn test_prepared_pins_in_place_numeric_reuse() {
    let model = "x + y = 10\nz = x * y\n";
    let settings = SolverSettings::default();
    let mut prep = PreparedDocument::new(model, &settings, &[], &[]).expect("prep failed");

    // Initial state: nothing prepared yet
    assert_eq!(prep.prep_count(), 0);
    assert_eq!(prep.solve_count(), 0);

    // Solve 1: x = 1.0 -> y = 9.0, z = 9.0
    let sol1 = prep
        .solve_with_pins(&[("x".to_string(), 1.0)], None)
        .expect("solve 1 failed");
    assert!((sol1.values["y"] - 9.0).abs() < 1e-9);
    assert!((sol1.values["z"] - 9.0).abs() < 1e-9);
    assert_eq!(prep.prep_count(), 1);
    assert_eq!(prep.solve_count(), 1);

    // Solve 2: x = 2.0 -> in-place numeric update, no rebuild
    let sol2 = prep
        .solve_with_pins(&[("x".to_string(), 2.0)], None)
        .expect("solve 2 failed");
    assert!((sol2.values["y"] - 8.0).abs() < 1e-9);
    assert!((sol2.values["z"] - 16.0).abs() < 1e-9);
    assert_eq!(
        prep.prep_count(),
        1,
        "prep_count must not increment on numeric pin change"
    );
    assert_eq!(prep.solve_count(), 2);

    // Solve 3: x = 5.0 -> in-place numeric update, no rebuild
    let sol3 = prep
        .solve_with_pins(&[("x".to_string(), 5.0)], None)
        .expect("solve 3 failed");
    assert!((sol3.values["y"] - 5.0).abs() < 1e-9);
    assert!((sol3.values["z"] - 25.0).abs() < 1e-9);
    assert_eq!(
        prep.prep_count(),
        1,
        "prep_count must remain 1 across numeric pin updates"
    );
    assert_eq!(prep.solve_count(), 3);
}

#[test]
fn test_prepared_pins_structural_change_triggers_rebuild() {
    let model = "x + y = 10\nz = x * y\n";
    let settings = SolverSettings::default();
    let mut prep = PreparedDocument::new(model, &settings, &[], &[]).expect("prep failed");

    // Solve with x = 2.0
    let sol1 = prep
        .solve_with_pins(&[("x".to_string(), 2.0)], None)
        .expect("solve 1 failed");
    assert!((sol1.values["y"] - 8.0).abs() < 1e-9);
    assert_eq!(prep.prep_count(), 1);
    assert_eq!(prep.solve_count(), 1);

    // Structural change: pin y instead of x -> must trigger clean preparation
    let sol2 = prep
        .solve_with_pins(&[("y".to_string(), 4.0)], None)
        .expect("solve 2 failed");
    assert!((sol2.values["x"] - 6.0).abs() < 1e-9);
    assert!((sol2.values["z"] - 24.0).abs() < 1e-9);
    assert_eq!(
        prep.prep_count(),
        2,
        "changing pinned variable name must trigger preparation"
    );
    assert_eq!(prep.solve_count(), 2);

    // Numeric update on y: in-place reuse
    let sol3 = prep
        .solve_with_pins(&[("y".to_string(), 3.0)], None)
        .expect("solve 3 failed");
    assert!((sol3.values["x"] - 7.0).abs() < 1e-9);
    assert!((sol3.values["z"] - 21.0).abs() < 1e-9);
    assert_eq!(prep.prep_count(), 2);
    assert_eq!(prep.solve_count(), 3);
}

#[test]
fn test_multi_variable_pins_rebuild_on_pin_set_change() {
    let model = "a + b + c = 30\nd = a * b * c\n";
    let settings = SolverSettings::default();
    let mut prep = PreparedDocument::new(model, &settings, &[], &[]).expect("prep failed");

    // Pin (a, b) = (2, 3) -> c = 25, d = 150
    let pins1 = vec![("a".to_string(), 2.0), ("b".to_string(), 3.0)];
    let sol1 = prep.solve_with_pins(&pins1, None).expect("solve 1 failed");
    assert!((sol1.values["c"] - 25.0).abs() < 1e-9);
    assert!((sol1.values["d"] - 150.0).abs() < 1e-9);
    assert_eq!(prep.prep_count(), 1);
    assert_eq!(prep.solve_count(), 1);

    // Pin (a, b) = (5, 5) -> c = 20, d = 500 (in-place)
    let pins2 = vec![("a".to_string(), 5.0), ("b".to_string(), 5.0)];
    let sol2 = prep.solve_with_pins(&pins2, None).expect("solve 2 failed");
    assert!((sol2.values["c"] - 20.0).abs() < 1e-9);
    assert!((sol2.values["d"] - 500.0).abs() < 1e-9);
    assert_eq!(prep.prep_count(), 1);
    assert_eq!(prep.solve_count(), 2);

    // Pin (a, c) = (5, 10) -> b = 15, d = 750 (pin set changed from (a,b) to (a,c))
    let pins3 = vec![("a".to_string(), 5.0), ("c".to_string(), 10.0)];
    let sol3 = prep.solve_with_pins(&pins3, None).expect("solve 3 failed");
    assert!((sol3.values["b"] - 15.0).abs() < 1e-9);
    assert!((sol3.values["d"] - 750.0).abs() < 1e-9);
    assert_eq!(
        prep.prep_count(),
        2,
        "pin set change (a,b)->(a,c) must trigger rebuild"
    );
    assert_eq!(prep.solve_count(), 3);
}

#[test]
fn test_complex_mode_isolated_preparation() {
    let model = "x^2 = -4\ny = x + 1\n";
    let settings = SolverSettings {
        complex_mode: true,
        ..Default::default()
    };

    let mut prep = PreparedDocument::new(model, &settings, &[], &[]).expect("prep complex failed");
    let sol = prep
        .solve_with_pins(&[], None)
        .expect("solve complex failed");

    let x_r = sol.values["x_r"];
    let x_i = sol.values["x_i"];
    assert!(x_r.abs() < 1e-6, "real part should be 0, got {x_r}");
    assert!(
        (x_i.abs() - 2.0).abs() < 1e-6,
        "imag part magnitude should be 2, got {x_i}"
    );
    assert_eq!(prep.prep_count(), 1);
    assert_eq!(prep.solve_count(), 1);
}

#[test]
fn test_prepared_solver_throughput_speedup() {
    let model = r#"
FUNCTION f(t)
  f = t^2 + 2*t + 1
END
x + y = 100
y = f(t)
z = x * t
"#;
    let settings = SolverSettings::default();
    let n_iterations = 50;

    // Fresh solves (baseline)
    let start_fresh = std::time::Instant::now();
    for i in 1..=n_iterations {
        let t = i as f64;
        let sol =
            solve_with_parametric_tables(&format!("{model}\nt = {t}\n"), &settings, &[], None, &[])
                .expect("fresh solve failed");
        let expected_y = t * t + 2.0 * t + 1.0;
        assert!((sol.values["y"] - expected_y).abs() < 1e-6);
    }
    let elapsed_fresh = start_fresh.elapsed();

    // Prepared solve (prepare once, solve repeatedly)
    let start_prep = std::time::Instant::now();
    let mut prep = PreparedDocument::new(model, &settings, &[], &[]).expect("prep failed");
    for i in 1..=n_iterations {
        let t = i as f64;
        let sol = prep
            .solve_with_pins(&[("t".to_string(), t)], None)
            .expect("prep solve failed");
        let expected_y = t * t + 2.0 * t + 1.0;
        assert!((sol.values["y"] - expected_y).abs() < 1e-6);
    }
    let elapsed_prep = start_prep.elapsed();

    assert_eq!(prep.prep_count(), 1);
    assert_eq!(prep.solve_count(), n_iterations);

    println!(
        "Prepared solver benchmark ({n_iterations} solves): fresh = {:?}, prepared = {:?}, speedup = {:.2}x",
        elapsed_fresh,
        elapsed_prep,
        elapsed_fresh.as_secs_f64() / elapsed_prep.as_secs_f64()
    );

    assert!(
        elapsed_prep < elapsed_fresh,
        "Prepared solver ({:?}) should be faster than fresh solves ({:?})",
        elapsed_prep,
        elapsed_fresh
    );
}
