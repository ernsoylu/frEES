//! Native tests for the `solve_table` export — the first end-to-end drive of
//! `analysis::parametric::run_sweep` with a real `engine::solve_with` (the
//! module's own tests inject a stubbed `solve_row`; this is the integration
//! the Phase-8 status doc recorded as missing).

use serde_json::Value;

fn call(source: &str, request: &str) -> Value {
    serde_json::from_str(&frees::solve_table(source, request)).expect("valid JSON out")
}

#[test]
fn a_plain_sweep_solves_every_row_in_display_units() {
    let out = call(
        "y = 2 * x\n",
        r#"{"table": {"variables": ["x", "y"], "rows": [{"x": 1}, {"x": 2}, {"x": 3}]}}"#,
    );
    assert!(out.get("error").is_none(), "{out}");
    let results = out["results"].as_array().unwrap();
    assert_eq!(results.len(), 3);
    for (i, expect_y) in [(0, 2.0), (1, 4.0), (2, 6.0)] {
        let row = &results[i];
        assert_eq!(row["success"], true, "{row}");
        assert_eq!(row["error"], Value::Null);
        assert_eq!(row["values"]["y"].as_f64().unwrap(), expect_y);
        assert_eq!(row["values"]["x"].as_f64().unwrap(), (i + 1) as f64);
    }
    let stats = &out["stats"];
    assert_eq!(stats["runs"], 3);
    assert_eq!(stats["solved"], 3);
    assert_eq!(stats["failed"], 0);
    // Each row is base + one pin: 2 equations, 2 unknowns.
    assert_eq!(stats["equations"], 2);
    assert_eq!(stats["unknowns"], 2);
    assert!(stats["iterations"].as_u64().unwrap() >= 3, "{stats}");
    // `variables` is the last successful row's DTO list.
    let vars = out["variables"].as_array().unwrap();
    assert!(
        vars.iter()
            .any(|v| v["name"] == "y" && v["value"].as_f64() == Some(6.0)),
        "{vars:?}"
    );
}

#[test]
fn an_accessor_table_iterates_to_the_fixed_point() {
    // `avg` reads the whole `y` column, so pass 1 (no accessors installed)
    // sees the empty-context default and pass 2 must land on mean(2, 4, 6).
    // This is the first test anywhere that drives `RowJob::accessors` into a
    // real solve — it proves `solve_with_parametric` actually installs the
    // channel `eval.rs` reads.
    let out = call(
        "avg = TableAvg('y')\ny = 2 * x\n",
        r#"{"table": {"variables": ["x", "y"], "rows": [{"x": 1}, {"x": 2}, {"x": 3}]}}"#,
    );
    assert!(out.get("error").is_none(), "{out}");
    let results = out["results"].as_array().unwrap();
    assert_eq!(results.len(), 3);
    for row in results {
        assert_eq!(row["success"], true, "{row}");
        assert_eq!(row["values"]["avg"].as_f64().unwrap(), 4.0, "{row}");
    }
}

#[test]
fn a_failing_row_reports_its_error_and_the_rest_still_solve() {
    // Row 2 pins x = 0 and the document divides by x.
    let out = call(
        "y = 1 / x\n",
        r#"{"table": {"variables": ["x", "y"], "rows": [{"x": 2}, {"x": 0}]}}"#,
    );
    let results = out["results"].as_array().unwrap();
    assert_eq!(results[0]["success"], true, "{results:?}");
    assert_eq!(results[0]["values"]["y"].as_f64().unwrap(), 0.5);
    assert_eq!(results[1]["success"], false, "{results:?}");
    assert!(
        results[1]["error"]
            .as_str()
            .unwrap()
            .contains("division by zero"),
        "{results:?}"
    );
    assert_eq!(out["stats"]["solved"], 1);
    assert_eq!(out["stats"]["failed"], 1);
}

#[test]
fn the_row_cap_refuses_with_the_java_message() {
    let rows: Vec<String> = (0..5001).map(|i| format!("{{\"x\": {i}}}")).collect();
    let request = format!(
        "{{\"table\": {{\"variables\": [\"x\", \"y\"], \"rows\": [{}]}}}}",
        rows.join(",")
    );
    let out = call("y = 2 * x\n", &request);
    assert_eq!(
        out["error"].as_str().unwrap(),
        "The parametric table has too many rows (5001; limit 5000). Reduce the run count.",
        "{out}"
    );
    assert_eq!(out["results"].as_array().unwrap().len(), 0);
    assert_eq!(out["stats"], Value::Null);
}

#[test]
fn a_syntax_error_answers_once_not_per_row() {
    let out = call(
        "y = = x\n",
        r#"{"table": {"variables": ["x"], "rows": [{"x": 1}, {"x": 2}]}}"#,
    );
    let error = out["error"].as_str().unwrap();
    assert!(error.starts_with("Syntax error:"), "{error}");
    assert_eq!(out["results"].as_array().unwrap().len(), 0);
}

#[test]
fn a_blank_document_and_a_missing_table_are_data_not_exceptions() {
    let out = call("", r#"{"table": {"variables": [], "rows": []}}"#);
    assert_eq!(out["error"], "The document is empty.");
    let out = call("y = 2\n", "{}");
    assert_eq!(out["error"], "The request carries no table.");
}

// ── monte_carlo (Wave B2) ──────────────────────────────────────────────────

fn mc(source: &str, request: &str) -> Value {
    serde_json::from_str(&frees::monte_carlo(source, request)).expect("valid JSON out")
}

#[test]
fn a_seeded_run_reproduces_the_library_oracle_through_the_boundary() {
    // The same document/seed as montecarlo.rs's transcribed JVM oracle:
    // x = 2 with sigma 0.1, y = 3x, seed 42, 8 samples.
    let out = mc(
        "x = 2\ny = 3 * x\n",
        r#"{"samples": 8, "seed": 42,
            "variableInfo": [{"name": "x", "guess": 2, "uncertainty": 0.1}]}"#,
    );
    assert!(out.get("error").is_none(), "{out}");
    assert_eq!(out["requestedSamples"], 8);
    assert_eq!(out["failedSamples"], 0);
    assert_eq!(out["truncated"], false);
    assert_eq!(out["sources"], json_arr(&["x"]));
    let samples = out["samples"].as_array().unwrap();
    assert_eq!(samples.len(), 8);
    // First JVM-oracle draw (montecarlo.rs oracle test), boundary-for-library.
    let x0 = samples[0]["values"]["x"].as_f64().unwrap();
    assert!((x0 - 2.1141905315473055).abs() < 1e-12, "{x0}");
    let y0 = samples[0]["values"]["y"].as_f64().unwrap();
    assert!((y0 - 3.0 * x0).abs() < 1e-12, "{y0}");
    // Stats are sorted by |sigma| descending: y (3σ_x) before x (σ_x).
    let stats = out["stats"].as_array().unwrap();
    assert_eq!(stats[0]["variable"], "y", "{stats:?}");
    assert_eq!(stats[1]["variable"], "x", "{stats:?}");
    let ratio = stats[0]["sigma"].as_f64().unwrap() / stats[1]["sigma"].as_f64().unwrap();
    assert!((ratio - 3.0).abs() < 1e-9, "{ratio}");
    // firstOrderSigma rides in from the base solve's propagation pass.
    assert!(
        (stats[1]["firstOrderSigma"].as_f64().unwrap() - 0.1).abs() < 1e-12,
        "{stats:?}"
    );
}

#[test]
fn the_sample_count_cap_refuses_with_the_java_message() {
    let out = mc("x = 2\n", r#"{"samples": 1001}"#);
    assert_eq!(
        out["error"].as_str().unwrap(),
        "Monte Carlo sample count must be between 2 and 1000 (got 1001). \
         Adjust the sample count.",
        "{out}"
    );
    let out = mc("x = 2\n", r#"{"samples": 1}"#);
    assert!(out["error"].as_str().unwrap().contains("(got 1)"), "{out}");
}

#[test]
fn a_run_without_uncertainty_sources_refuses_with_the_library_message() {
    let out = mc("x = 2\n", r#"{"samples": 8}"#);
    assert_eq!(
        out["error"].as_str().unwrap(),
        "Monte Carlo needs at least one variable with a declared uncertainty \
         (set one in the Variable Information window).",
        "{out}"
    );
}

#[test]
fn the_seed_defaults_to_42_and_is_deterministic() {
    let request = r#"{"samples": 4,
        "variableInfo": [{"name": "x", "guess": 2, "uncertainty": 0.1}]}"#;
    let a = mc("x = 2\ny = 3 * x\n", request);
    let b = mc("x = 2\ny = 3 * x\n", request);
    assert_eq!(a, b);
    let seeded = mc(
        "x = 2\ny = 3 * x\n",
        r#"{"samples": 4, "seed": 42,
            "variableInfo": [{"name": "x", "guess": 2, "uncertainty": 0.1}]}"#,
    );
    assert_eq!(a["samples"], seeded["samples"]);
}

fn json_arr(items: &[&str]) -> Value {
    Value::Array(
        items
            .iter()
            .map(|s| Value::String((*s).to_string()))
            .collect(),
    )
}

// ── Wave B3: the four OptimizeController endpoints ─────────────────────────

#[test]
fn curve_fit_reproduces_the_library_oracle_through_the_boundary() {
    // The curvefit.rs oracle's exponential-decay data, default start.
    let xs: Vec<f64> = (0..9).map(|i| i as f64 * 0.5).collect();
    let ys: Vec<f64> = xs.iter().map(|x| 5.0 * (-1.3 * x).exp() + 0.7).collect();
    let request = serde_json::json!({
        "model": "y = a * exp(-b * x) + c",
        "yVariable": "y",
        "xVariable": "x",
        "parameters": ["a", "b", "c"],
        "xData": xs,
        "yData": ys,
    });
    let out: Value = serde_json::from_str(&frees::curve_fit(&request.to_string())).unwrap();
    assert_eq!(out["success"], true, "{out}");
    assert_eq!(out["parameterNames"], json_arr(&["a", "b", "c"]));
    let fitted = out["fittedParameters"].as_array().unwrap();
    for (i, expect) in [(0, 5.0), (1, 1.3), (2, 0.7)] {
        let got = fitted[i].as_f64().unwrap();
        assert!((got - expect).abs() < 1e-6, "param {i}: {got} vs {expect}");
    }
    assert!(out["rSquared"].as_f64().unwrap() > 0.999999, "{out}");
}

#[test]
fn curve_fit_validation_speaks_the_java_messages() {
    let out: Value = serde_json::from_str(&frees::curve_fit(r#"{"model": ""}"#)).unwrap();
    assert_eq!(out["error"], "Model equation is required.");
    let out: Value = serde_json::from_str(&frees::curve_fit(
        r#"{"model": "y = a*x", "xVariable": "x", "yVariable": "y",
            "parameters": ["a"], "xData": [1, 2], "yData": [1]}"#,
    ))
    .unwrap();
    assert_eq!(
        out["error"],
        "x and y data must have the same length (got 2 and 1)."
    );
}

#[test]
fn optimize_finds_a_univariate_minimum_in_display_shape() {
    // f = (x - 3)^2 + 1: minimum at x = 3, f = 1.
    let out: Value = serde_json::from_str(&frees::optimize(
        "f = (x - 3)^2 + 1\n",
        r#"{"objective": "f", "decisions": ["x"], "lowers": [0], "uppers": [10]}"#,
    ))
    .unwrap();
    assert_eq!(out["success"], true, "{out}");
    let x = out["decision"]["value"].as_f64().unwrap();
    assert!((x - 3.0).abs() < 1e-6, "{x}");
    let f = out["objective"]["value"].as_f64().unwrap();
    assert!((f - 1.0).abs() < 1e-9, "{f}");
    assert!(out["evaluations"].as_u64().unwrap() > 0);
    assert_eq!(out["decisions"].as_array().unwrap().len(), 1);
    assert!(!out["variables"].as_array().unwrap().is_empty());
}

#[test]
fn optimize_validation_speaks_the_java_messages() {
    let out: Value = serde_json::from_str(&frees::optimize("", "{}")).unwrap();
    assert_eq!(out["error"], "No equations entered.");
    let out: Value =
        serde_json::from_str(&frees::optimize("f = x\n", r#"{"objective": "f"}"#)).unwrap();
    assert_eq!(out["error"], "Independent variable name is required.");
}

#[test]
fn optimize_multi_returns_a_sorted_front_with_echoed_names() {
    // Two competing objectives over one decision: f1 = x^2, f2 = (x - 2)^2.
    let out: Value = serde_json::from_str(&frees::optimize_multi(
        "f1 = x^2\nf2 = (x - 2)^2\n",
        r#"{"objectives": ["f1", "f2"], "decisions": ["x"],
            "lowers": [0], "uppers": [2],
            "populationSize": 16, "generations": 8}"#,
    ))
    .unwrap();
    assert_eq!(out["success"], true, "{out}");
    assert_eq!(out["decisionNames"], json_arr(&["x"]));
    assert_eq!(out["objectiveNames"], json_arr(&["f1", "f2"]));
    let front = out["front"].as_array().unwrap();
    assert!(front.len() >= 2, "{out}");
    // Sorted by the first objective, and every point trades off: f1 up, f2 down.
    let f1s: Vec<f64> = front
        .iter()
        .map(|p| p["objectives"][0].as_f64().unwrap())
        .collect();
    assert!(f1s.windows(2).all(|w| w[0] <= w[1]), "{f1s:?}");
}

#[test]
fn optimize_multi_needs_two_objectives() {
    let out: Value = serde_json::from_str(&frees::optimize_multi(
        "f1 = x^2\n",
        r#"{"objectives": ["f1"], "decisions": ["x"], "lowers": [0], "uppers": [2]}"#,
    ))
    .unwrap();
    assert_eq!(
        out["error"],
        "Multi-objective optimization needs at least two objective variables."
    );
}

#[test]
fn parameter_fit_calibrates_a_decay_rate_against_its_own_trajectory() {
    // Generate the measured series from the true k = 0.05, then fit from a
    // wrong start: the boundary's solve callback must drive the real engine's
    // DYNAMIC path per candidate.
    let doc = |k: f64| {
        format!(
            "k = {k}\nTinf = 20\nDYNAMIC cooling (method = ode45, time = 0 .. 60, points = 31)\n  der(Temp) = -k*(Temp - Tinf)\n  Temp(0) = 95\nEND\n"
        )
    };
    let truth: Value = serde_json::from_str(&frees::solve(&doc(0.05), "{}")).unwrap();
    let rows = truth["odeTables"][0]["rows"].as_array().unwrap();
    let ts: Vec<f64> = rows.iter().map(|r| r[0].as_f64().unwrap()).collect();
    let vs: Vec<f64> = rows.iter().map(|r| r[1].as_f64().unwrap()).collect();

    let request = serde_json::json!({
        "text": doc(0.2),
        "parameters": ["k"],
        "initial": [0.2],
        "lower": [0.001],
        "upper": [1.0],
        "odeBlock": "cooling",
        "column": "temp",
        "measuredT": ts,
        "measuredV": vs,
    });
    let out: Value =
        serde_json::from_str(&frees::parameter_fit(&request.to_string())).unwrap();
    assert_eq!(out["success"], true, "{out}");
    assert_eq!(out["parameterNames"], json_arr(&["k"]));
    let k = out["fittedValues"][0].as_f64().unwrap();
    assert!((k - 0.05).abs() < 1e-3, "fitted k = {k}");
    assert!(
        out["rmse"].as_f64().unwrap() < out["initialRmse"].as_f64().unwrap(),
        "{out}"
    );
    assert_eq!(out["truncated"], false);
    assert!(!out["fittedT"].as_array().unwrap().is_empty());
}

#[test]
fn parameter_fit_caps_speak_the_java_messages() {
    let out: Value = serde_json::from_str(&frees::parameter_fit(r#"{"text": ""}"#)).unwrap();
    assert_eq!(out["error"], "The model document is required.");
    let big: Vec<f64> = vec![0.0; 200_001];
    let request = serde_json::json!({
        "text": "x = 1\n",
        "parameters": ["k"], "initial": [1], "lower": [0], "upper": [2],
        "odeBlock": "d", "column": "c",
        "measuredT": big, "measuredV": big,
    });
    let out: Value =
        serde_json::from_str(&frees::parameter_fit(&request.to_string())).unwrap();
    assert_eq!(
        out["error"],
        "The measured series has too many samples (200001; limit 200000). Decimate it first."
    );
}

// ── Wave B4: pid_tune + extract_plant ──────────────────────────────────────

#[test]
fn pid_tune_echoes_the_suggested_crossover_and_tunes_a_first_order_plant() {
    // The Java controller test's plant 1/(5s+1): suggestWc == 0.2 when wc is
    // omitted, and the tuned loop must report finite gains and a step trace.
    let out: Value = serde_json::from_str(&frees::pid_tune(
        r#"{"num": [1], "den": [5, 1], "type": "pi"}"#,
    ))
    .unwrap();
    assert!(out.get("error").is_none(), "{out}");
    assert!((out["wc"].as_f64().unwrap() - 0.2).abs() < 1e-12, "{out}");
    assert_eq!(out["pm"].as_f64().unwrap(), 60.0);
    assert!(out["kp"].as_f64().unwrap().is_finite());
    assert!(out["ki"].as_f64().unwrap().is_finite());
    assert!(out["t"].as_array().unwrap().len() >= 50);
    assert_eq!(
        out["t"].as_array().unwrap().len(),
        out["y"].as_array().unwrap().len()
    );
}

#[test]
fn pid_tune_validation_speaks_the_java_messages() {
    let out: Value =
        serde_json::from_str(&frees::pid_tune(r#"{"num": [], "den": []}"#)).unwrap();
    assert_eq!(
        out["error"],
        "A plant transfer function (num and den coefficients) is required."
    );
    let out: Value = serde_json::from_str(&frees::pid_tune(
        r#"{"num": [1], "den": [5, 1], "type": "ZN"}"#,
    ))
    .unwrap();
    assert_eq!(
        out["error"],
        "Controller type must be one of p, pi, pid (got 'ZN')."
    );
}

#[test]
fn extract_plant_recovers_the_first_order_plant_from_a_closed_loop() {
    // The ControlControllerPlantTest loop: SP(k=5) -> PID(2,1,0) -> PLANT
    // (first order, tau=2). The recovered G must equal 1/(2s+1) as a rational
    // function at several frequencies.
    let text = "SigConstant SP(k=5)\nSigPID PID(Kp=2, Ki=1, Kd=0, tau=0.1, i0=0, d0=0)\nSigFirstOrder PLANT(tau=2, y0=0)\nconnect(SP.out, PID.sp)\nconnect(PLANT.out, PID.pv)\nconnect(PID.out, PLANT.in)\nDYNAMIC loop(method = ode23s, time = 0 .. 40, points = 400)\nEND\n";
    let request = serde_json::json!({
        "text": text,
        "dynamic": "loop",
        "reference": "SP",
        "output": "plant.out.sig",
        "referenceOnSp": true,
        "type": "pi",
        "kp": 2.0,
        "ki": 1.0,
        "kd": 0.0,
    });
    let out: Value =
        serde_json::from_str(&frees::extract_plant(&request.to_string())).unwrap();
    assert!(out.get("error").is_none(), "{out}");
    let num: Vec<f64> = out["num"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_f64().unwrap())
        .collect();
    let den: Vec<f64> = out["den"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_f64().unwrap())
        .collect();
    // Rational-function equality against 1/(2s+1) at w in {0, 0.1, 0.5, 1, 3}
    // (robust to uncancelled common factors), tol 1e-3 — the Java test's own
    // acceptance rule.
    let eval_poly = |p: &[f64], w: f64| -> (f64, f64) {
        // p(jw) with descending coefficients: returns (re, im).
        let mut re = 0.0;
        let mut im = 0.0;
        for &c in p {
            // multiply (re, im) by jw, then add c
            let (nre, nim) = (-im * w, re * w);
            re = nre + c;
            im = nim;
        }
        (re, im)
    };
    for w in [0.0, 0.1, 0.5, 1.0, 3.0] {
        let (gn_re, gn_im) = eval_poly(&num, w);
        let (gd_re, gd_im) = eval_poly(&den, w);
        // expected 1/(2jw+1): num 1, den (1, 2w)
        // cross-multiplied: G_num * (1 + 2jw) == G_den * 1
        let lhs = (
            gn_re * 1.0 - gn_im * (2.0 * w),
            gn_re * (2.0 * w) + gn_im * 1.0,
        );
        let scale = (gd_re * gd_re + gd_im * gd_im).sqrt().max(1e-12);
        assert!(
            ((lhs.0 - gd_re).powi(2) + (lhs.1 - gd_im).powi(2)).sqrt() / scale < 1e-3,
            "w={w}: num={num:?} den={den:?}"
        );
    }
}

#[test]
fn extract_plant_names_a_missing_reference_constant() {
    let out: Value = serde_json::from_str(&frees::extract_plant(
        r#"{"text": "x = 1\n", "dynamic": "loop", "reference": "SP",
            "output": "plant.out.sig", "referenceOnSp": true,
            "type": "pi", "kp": 1, "ki": 0, "kd": 0}"#,
    ))
    .unwrap();
    assert_eq!(
        out["error"],
        "Could not find a constant to perturb on reference source 'SP' \
         (expected a SigConstant with a k= value)."
    );
}

// ── Wave C1: the transient wall-clock budget ───────────────────────────────

#[test]
fn a_transient_over_its_elapsed_budget_stops_with_the_named_deadline_error() {
    // An expired budget (elapsedTimeSeconds is honoured since C1, clamped to
    // 60 s): a maxstep-forced many-step integration must strike immediately
    // rather than spin. The classification is the boundary's own message —
    // never a Java-classified error, so no golden can ever meet it.
    let doc = "k = 0.05\nTinf = 20\nDYNAMIC cooling (method = ode45, time = 0 .. 60, \
               points = 5, maxstep = 0.00001)\n  der(Temp) = -k*(Temp - Tinf)\n  \
               Temp(0) = 95\nEND\n";
    let out: Value = serde_json::from_str(&frees::solve(
        doc,
        r#"{"stopCriteria": {"elapsedTimeSeconds": 1e-9}}"#,
    ))
    .unwrap();
    assert_eq!(out["success"], false, "{out}");
    let error = out["error"].as_str().unwrap();
    assert!(
        error.contains("exceeded its 0-second wall-clock budget"),
        "{error}"
    );
    assert!(error.contains("Stop Criteria"), "{error}");

    // The same document with the default budget solves — and the deadline
    // guard cleared the thread-local, so this call cannot inherit a strike.
    let quick = "k = 0.05\nTinf = 20\nDYNAMIC cooling (method = ode45, time = 0 .. 60, \
                 points = 5)\n  der(Temp) = -k*(Temp - Tinf)\n  Temp(0) = 95\nEND\n";
    let out: Value = serde_json::from_str(&frees::solve(quick, "{}")).unwrap();
    assert_eq!(out["success"], true, "{out}");
}
