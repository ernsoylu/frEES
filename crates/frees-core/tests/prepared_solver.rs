use frees_core::engine::{solve_with_parametric_tables, PreparedDocument};
use frees_core::solver::SolverSettings;

#[test]
fn prepared_pins_match_parsed_equations() {
    for (model, name, complex) in [
        ("Y = 2 * X", "X", false),
        ("y = 2 * x", "x", true),
        ("y = 2 * x[1]", "X[1]", false),
        ("y = 2 * part.x", "part.X", false),
    ] {
        let settings = SolverSettings {
            complex_mode: complex,
            ..Default::default()
        };
        let mut prep = PreparedDocument::new(model, &settings, &[], &[]).unwrap();
        for value in [2.0, 3.0] {
            let fresh = solve_with_parametric_tables(
                &format!("{model}\n{name} = {value}"),
                &settings,
                &[],
                None,
                &[],
            )
            .unwrap();
            let cached = prep.solve_with_pins(&[(name.into(), value)], None).unwrap();
            assert_eq!(cached.values, fresh.values, "{model}, {name}={value}");
            assert_eq!(cached.display_names, fresh.display_names);
        }
    }
}

#[test]
fn prepared_ode_uses_pinned_parameters() {
    let model = "DYNAMIC d (time = 0 .. 1, points = 2)\n der(y) = rate\n y(0) = 0\nEND";
    let settings = SolverSettings::default();
    let mut prep = PreparedDocument::new(model, &settings, &[], &[]).unwrap();
    for rate in [2.0, 3.0] {
        let solution = prep
            .solve_with_pins(&[("rate".into(), rate)], None)
            .unwrap();
        assert!((solution.ode_tables[0].rows.last().unwrap()[1] - rate).abs() < 1e-8);
        assert_eq!(solution.values["rate"], rate);
    }
}

// Frees notebook, Holman, Heat Transfer (10th ed.), Examples 1-2/1-3,
// p. 17: convection plus radiation through a steel plate. SI inputs;
// the book rounds 2156.25 W to 2.156 kW before its temperature calculation.
#[test]
fn holman_plate_heat_balance_sweep() {
    let model = "Area = 0.5 * 0.75\nQconv = H * Area * (250 - 20)\n\
        Qtotal = Qconv + 300\nQtotal = 43 * Area * (Tin - 250) / 0.02";
    let mut prep = PreparedDocument::new(model, &SolverSettings::default(), &[], &[]).unwrap();
    for h in [25.0, 50.0] {
        let sol = prep.solve_with_pins(&[("H".into(), h)], None).unwrap();
        let heat = h * 0.375 * 230.0 + 300.0;
        assert!((sol.values["qtotal"] - heat).abs() < 1e-7);
        assert!((sol.values["tin"] - (250.0 + heat * 0.02 / (43.0 * 0.375))).abs() < 1e-7);
    }
    assert_eq!(prep.prep_count(), 1);
}

// Same source, Example 4-1, p. 143: a 5 cm steel ball cools from 450 C
// in a 100 C environment. Published time to 150 C: 5819 s (rounded).
#[test]
fn holman_cooling_ball_sweep_and_stop_event() {
    for method in ["ode45", "radau"] {
        let model = format!(
            "DYNAMIC cooling (method = {method}, time = 0 .. 6000, points = 31)\n\
            der(Temp) = -6 * H / (7800 * 460 * 0.05) * (Temp - 100)\n\
            Temp(0) = 450\nEVENT cooled: Temp = 150 | falling -> stop\nEND"
        );
        let mut prep = PreparedDocument::new(&model, &SolverSettings::default(), &[], &[]).unwrap();
        for h in [10.0, 20.0] {
            let sol = prep.solve_with_pins(&[("H".into(), h)], None).unwrap();
            let table = &sol.ode_tables[0];
            let decay = 6.0 * h / (7800.0 * 460.0 * 0.05);
            let expected_time = 7.0_f64.ln() / decay;
            assert!(table.stopped);
            assert_eq!(table.events.len(), 1);
            assert!(
                (table.end_time - expected_time).abs() < 0.05,
                "{method}: {} vs {expected_time}",
                table.end_time
            );
            for row in &table.rows {
                let expected = 100.0 + 350.0 * (-decay * row[0]).exp();
                assert!((row[1] - expected).abs() < 0.01, "{method}: {row:?}");
            }
        }
    }
}

// Nhut Ho, ME584, Modeling Electrical Systems, slides 12 and 16:
// impedance and Kirchhoff's law. Adapted phasor case, chosen here:
// 10 V across R + j3 ohms, giving I = 10(R-j3)/(R^2+9) amperes.
#[test]
fn series_rl_phasor_sweep() {
    let settings = SolverSettings {
        complex_mode: true,
        ..Default::default()
    };
    let model = "Voltage = 10\nZ = Resistance + 3i\nCurrent = Voltage / Z";
    let mut prep = PreparedDocument::new(model, &settings, &[], &[]).unwrap();
    for r in [4.0, 8.0] {
        let sol = prep
            .solve_with_pins(&[("Resistance".into(), r)], None)
            .unwrap();
        assert!((sol.values["current_r"] - 10.0 * r / (r * r + 9.0)).abs() < 1e-7);
        assert!((sol.values["current_i"] + 30.0 / (r * r + 9.0)).abs() < 1e-7);
    }
}

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
fn test_prepared_function_reuse_matches_fresh_solves() {
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
    for i in 1..=n_iterations {
        let t = i as f64;
        let sol =
            solve_with_parametric_tables(&format!("{model}\nt = {t}\n"), &settings, &[], None, &[])
                .expect("fresh solve failed");
        let expected_y = t * t + 2.0 * t + 1.0;
        assert!((sol.values["y"] - expected_y).abs() < 1e-6);
    }

    // Prepared solve (prepare once, solve repeatedly)
    let mut prep = PreparedDocument::new(model, &settings, &[], &[]).expect("prep failed");
    for i in 1..=n_iterations {
        let t = i as f64;
        let sol = prep
            .solve_with_pins(&[("t".to_string(), t)], None)
            .expect("prep solve failed");
        let expected_y = t * t + 2.0 * t + 1.0;
        assert!((sol.values["y"] - expected_y).abs() < 1e-6);
    }

    assert_eq!(prep.prep_count(), 1);
    assert_eq!(prep.solve_count(), n_iterations);
}
