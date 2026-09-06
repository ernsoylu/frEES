//! Native tests for the `functionTables` request channel (Wave H, decision
//! D10) — the D10 equivalence oracle at the boundary: a GUI Function Table
//! sent on the request must answer **bit-identically** to the same data
//! written as an in-document `TABLE` block, across every export the Java
//! injects into (`solve`, `check`, `solve_table` per row, `monte_carlo`,
//! `parameter_fit`), plus the REPL cache. The DTO tolerance graded here is
//! `SolveDtos.functionDefsOf`'s, member for member.

use serde_json::{json, Value};

fn solve(source: &str, request: &Value) -> Value {
    serde_json::from_str(&frees::solve(source, &request.to_string())).expect("valid JSON out")
}

fn check(source: &str, request: &Value) -> Value {
    serde_json::from_str(&frees::check(source, &request.to_string())).expect("valid JSON out")
}

/// The `fcurve` fixture as the GUI sends it: one lone curve, three points.
fn fcurve_dto() -> Value {
    json!({
        "name": "fcurve",
        "argNames": ["x"],
        "xLog": false,
        "yLog": false,
        "curves": [{"param": null, "points": [[1, 10], [2, 20], [4, 25]]}],
    })
}

/// The same data as an in-document `TABLE` block.
const FCURVE_BLOCK: &str = "TABLE fcurve(x)\n1 10\n2 20\n4 25\nEND\n";

/// `variables[]` and the block count must agree bit for bit between the two
/// routes — the D10 acceptance bar.
fn assert_solves_identically(via_request: &Value, via_document: &Value) {
    assert_eq!(via_request["success"], true, "{via_request}");
    assert_eq!(via_document["success"], true, "{via_document}");
    let a = via_request["variables"].as_array().unwrap();
    let b = via_document["variables"].as_array().unwrap();
    assert_eq!(a.len(), b.len(), "different variable counts");
    for (va, vb) in a.iter().zip(b) {
        assert_eq!(va["name"], vb["name"]);
        assert_eq!(
            va["value"].as_f64().unwrap().to_bits(),
            vb["value"].as_f64().unwrap().to_bits(),
            "{}: {} vs {}",
            va["name"],
            va["value"],
            vb["value"]
        );
        assert_eq!(va["units"], vb["units"]);
    }
    assert_eq!(
        via_request["blocks"].as_array().unwrap().len(),
        via_document["blocks"].as_array().unwrap().len(),
        "block count"
    );
}

// ---------------------------------------------------------------------------
// POST /api/solve — SolveController line 217
// ---------------------------------------------------------------------------

#[test]
fn solve_answers_an_injected_table_exactly_like_the_table_block() {
    let via_request = solve(
        "y = fcurve(1.5)\nz = fcurve(3)\n",
        &json!({"functionTables": [fcurve_dto()]}),
    );
    let via_document = solve(
        &format!("{FCURVE_BLOCK}y = fcurve(1.5)\nz = fcurve(3)\n"),
        &json!({}),
    );
    assert_solves_identically(&via_request, &via_document);
    let y = via_request["variables"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["name"] == "y")
        .unwrap();
    assert_eq!(y["value"].as_f64().unwrap(), 15.0);
}

#[test]
fn solve_answers_a_curve_family_and_log_axes_like_their_blocks() {
    // 2-D family: nu(re, t) across two curves.
    let family = json!({"functionTables": [{
        "name": "nu",
        "argNames": ["re", "t"],
        "xLog": false,
        "yLog": false,
        "curves": [
            {"param": 100, "points": [[1, 10], [2, 20]]},
            {"param": 200, "points": [[1, 30], [2, 40]]},
        ],
    }]});
    let via_request = solve("a = nu(1.5, 100)\nb = nu(1.5, 150)\n", &family);
    let via_document = solve(
        "TABLE nu(re : t = 100, 200)\n1 10 30\n2 20 40\nEND\n\
         a = nu(1.5, 100)\nb = nu(1.5, 150)\n",
        &json!({}),
    );
    assert_solves_identically(&via_request, &via_document);

    // Log axes: xLog/yLog map onto the XLOG YLOG flags.
    let loglog = json!({"functionTables": [{
        "name": "damping",
        "argNames": ["f"],
        "xLog": true,
        "yLog": true,
        "curves": [{"param": null, "points": [[1, 10], [100, 1000]]}],
    }]});
    let via_request = solve("y = damping(10)\n", &loglog);
    let via_document = solve(
        "TABLE damping(f) XLOG YLOG\n1 10\n100 1000\nEND\ny = damping(10)\n",
        &json!({}),
    );
    assert_solves_identically(&via_request, &via_document);
    let y = via_request["variables"].as_array().unwrap()[0]["value"]
        .as_f64()
        .unwrap();
    assert!(
        (y - 100.0).abs() < 1e-9,
        "log-log midpoint should be 100: {y}"
    );
}

#[test]
fn the_callable_name_is_case_insensitive_and_trimmed() {
    // The DTO name arrives padded and mixed-case (`functionDefsOf`:
    // `name.trim().toLowerCase()`); the document calls it in another case.
    let via_request = solve(
        "y = FCurve(1.5)\n",
        &json!({"functionTables": [{
            "name": "  fCURVE  ",
            "argNames": ["x"],
            "curves": [{"param": null, "points": [[1, 10], [2, 20], [4, 25]]}],
        }]}),
    );
    let via_document = solve(&format!("{FCURVE_BLOCK}y = FCurve(1.5)\n"), &json!({}));
    assert_solves_identically(&via_request, &via_document);
}

#[test]
fn on_a_name_collision_the_document_definition_wins() {
    // `EquationSystemSolver.withExtraDefs`, verbatim from its own doc comment:
    // "source definitions win on name collision" (`merged = new
    // HashMap<>(extraDefs); merged.putAll(parsed.defs())`). So the request
    // CANNOT override an in-document TABLE of the same name on the solve
    // path — only in the REPL cache does the request win (tested below).
    let out = solve(
        &format!("{FCURVE_BLOCK}y = fcurve(1.5)\n"),
        &json!({"functionTables": [{
            "name": "fcurve",
            "argNames": ["x"],
            "curves": [{"param": null, "points": [[1, 1000], [2, 2000]]}],
        }]}),
    );
    assert_eq!(out["success"], true, "{out}");
    let y = out["variables"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["name"] == "y")
        .unwrap();
    assert_eq!(
        y["value"].as_f64().unwrap(),
        15.0,
        "the DOCUMENT's table must answer, not the request's"
    );
}

#[test]
fn malformed_tables_are_tolerated_exactly_like_functiondefsof() {
    // Every arm of the Java tolerance in one request: a null list member
    // set, a nameless table, a blank-named table, null curves, a curve of
    // null/short/null-membered points, and one healthy table whose points
    // arrive UNSORTED and with extra members (both ignored/sorted). Only the
    // healthy table lands; the document only calls that one.
    let out = solve(
        "y = good(1.5)\n",
        &json!({"functionTables": [
            {"argNames": ["x"], "curves": [{"param": null, "points": [[1, 2]]}]},
            {"name": "   ", "curves": [{"param": null, "points": [[1, 2]]}]},
            {"name": "nocurves"},
            {"name": "emptycurves", "curves": []},
            {"name": "emptypoints", "curves": [
                {"param": null, "points": []},
                {"param": null, "points": null},
                {"param": null, "points": [null, [1], [null, 5], [5, null]]},
            ]},
            {"name": "good", "argNames": ["x"], "curves": [
                {"param": null, "points": [[4, 25, 99], [1, 10], [2, 20]]},
            ]},
        ]}),
    );
    let via_document = solve(
        &format!("{FCURVE_BLOCK}y = good(1.5)\n").replace("fcurve", "good"),
        &json!({}),
    );
    assert_solves_identically(&out, &via_document);

    // And a name the tolerance dropped stays undefined: the call fails with
    // the ordinary unknown-function error, never a panic.
    let dropped = solve(
        "y = emptypoints(1)\n",
        &json!({"functionTables": [
            {"name": "emptypoints", "curves": [{"param": null, "points": []}]},
        ]}),
    );
    assert_eq!(dropped["success"], false, "{dropped}");
    assert!(
        dropped["error"]
            .as_str()
            .unwrap()
            .contains("unknown function"),
        "{dropped}"
    );
}

#[test]
fn a_null_or_absent_functiontables_field_changes_nothing() {
    let baseline = solve("y = 2 * 3\n", &json!({}));
    let with_null = solve("y = 2 * 3\n", &json!({"functionTables": null}));
    let with_empty = solve("y = 2 * 3\n", &json!({"functionTables": []}));
    assert_solves_identically(&with_null, &baseline);
    assert_solves_identically(&with_empty, &baseline);
}

#[test]
fn among_duplicate_names_the_last_table_wins() {
    // `functionDefsOf` keys a HashMap by name — `put` keeps the last.
    let out = solve(
        "y = fcurve(1.5)\n",
        &json!({"functionTables": [
            {"name": "fcurve", "curves": [{"param": null, "points": [[1, 10], [2, 20]]}]},
            {"name": "FCURVE", "curves": [{"param": null, "points": [[1, 100], [2, 200]]}]},
        ]}),
    );
    assert_eq!(out["success"], true, "{out}");
    let y = out["variables"].as_array().unwrap()[0]["value"]
        .as_f64()
        .unwrap();
    assert_eq!(y, 150.0);
}

// ---------------------------------------------------------------------------
// POST /api/check — CheckController line 142
// ---------------------------------------------------------------------------

#[test]
fn check_sees_an_injected_table_like_the_table_block() {
    let via_request = check(
        "y = fcurve(1.5)\n",
        &json!({"functionTables": [fcurve_dto()]}),
    );
    let via_document = check(&format!("{FCURVE_BLOCK}y = fcurve(1.5)\n"), &json!({}));
    assert_eq!(via_request["solvable"], true, "{via_request}");
    assert_eq!(via_request["solvable"], via_document["solvable"]);
    assert_eq!(via_request["equations"], via_document["equations"]);
    assert_eq!(via_request["unknowns"], via_document["unknowns"]);
    assert_eq!(via_request["message"], via_document["message"]);
}

// ---------------------------------------------------------------------------
// POST /api/solve/table — SolveController line 611 (and 531's chunk path):
// the per-row solves of a parametric sweep carry the request's tables.
// ---------------------------------------------------------------------------

#[test]
fn a_parametric_sweep_calls_an_injected_table_in_every_row() {
    let request = json!({
        "table": {"variables": ["x", "y"], "rows": [{"x": 1}, {"x": 2}, {"x": 4}]},
        "functionTables": [fcurve_dto()],
    });
    let via_request: Value = serde_json::from_str(&frees::solve_table(
        "y = fcurve(x)\n",
        &request.to_string(),
    ))
    .expect("valid JSON out");
    assert!(via_request.get("error").is_none(), "{via_request}");

    let document_request = json!({
        "table": {"variables": ["x", "y"], "rows": [{"x": 1}, {"x": 2}, {"x": 4}]},
    });
    let via_document: Value = serde_json::from_str(&frees::solve_table(
        &format!("{FCURVE_BLOCK}y = fcurve(x)\n"),
        &document_request.to_string(),
    ))
    .expect("valid JSON out");

    let rows_a = via_request["results"].as_array().unwrap();
    let rows_b = via_document["results"].as_array().unwrap();
    assert_eq!(rows_a.len(), 3);
    assert_eq!(rows_b.len(), 3);
    for (i, (a, b)) in rows_a.iter().zip(rows_b).enumerate() {
        assert_eq!(a["success"], true, "row {i}: {a}");
        assert_eq!(
            a["values"]["y"].as_f64().unwrap().to_bits(),
            b["values"]["y"].as_f64().unwrap().to_bits(),
            "row {i} diverged"
        );
    }
    assert_eq!(rows_a[0]["values"]["y"].as_f64().unwrap(), 10.0);
    assert_eq!(rows_a[1]["values"]["y"].as_f64().unwrap(), 20.0);
    assert_eq!(rows_a[2]["values"]["y"].as_f64().unwrap(), 25.0);
    assert_eq!(via_request["stats"]["solved"], 3);
}

#[test]
fn a_sweep_with_accessors_still_carries_the_injected_table() {
    // The accessor path re-runs every row per pass — the tables must survive
    // every one of those solves, not just the first.
    let request = json!({
        "table": {"variables": ["x", "y"], "rows": [{"x": 1}, {"x": 2}, {"x": 4}]},
        "functionTables": [fcurve_dto()],
    });
    let out: Value = serde_json::from_str(&frees::solve_table(
        "avg = TableAvg('y')\ny = fcurve(x)\n",
        &request.to_string(),
    ))
    .expect("valid JSON out");
    assert!(out.get("error").is_none(), "{out}");
    for row in out["results"].as_array().unwrap() {
        assert_eq!(row["success"], true, "{row}");
        // mean(10, 20, 25)
        let avg = row["values"]["avg"].as_f64().unwrap();
        assert!((avg - 55.0 / 3.0).abs() < 1e-9, "{row}");
    }
}

// ---------------------------------------------------------------------------
// POST /api/solve/montecarlo — SolveController line 741: the base solve and
// every per-sample solve carry the tables.
// ---------------------------------------------------------------------------

#[test]
fn monte_carlo_samples_through_an_injected_table() {
    let request = json!({
        "samples": 8,
        "seed": 42,
        "variableInfo": [{"name": "x", "guess": 2.0, "uncertainty": 0.1}],
        "functionTables": [fcurve_dto()],
    });
    let out: Value = serde_json::from_str(&frees::monte_carlo(
        "x = 2\ny = fcurve(x)\n",
        &request.to_string(),
    ))
    .expect("valid JSON out");
    assert!(out.get("error").is_none(), "{out}");
    assert_eq!(out["requestedSamples"], 8);
    assert_eq!(out["failedSamples"], 0, "{out}");
    assert!(
        out["stats"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s["variable"] == "y"),
        "{out}"
    );

    // Without the injection the same document cannot even base-solve —
    // proof the tables reached the engine rather than the run accidentally
    // succeeding.
    let without: Value = serde_json::from_str(&frees::monte_carlo(
        "x = 2\ny = fcurve(x)\n",
        &json!({
            "samples": 8,
            "variableInfo": [{"name": "x", "guess": 2.0, "uncertainty": 0.1}],
        })
        .to_string(),
    ))
    .expect("valid JSON out");
    assert!(
        without["error"]
            .as_str()
            .unwrap()
            .contains("unknown function"),
        "{without}"
    );
}

// ---------------------------------------------------------------------------
// POST /api/measurements/parameter-fit — OptimizeController line 570: every
// fit evaluation's solve carries the tables.
// ---------------------------------------------------------------------------

#[test]
fn parameter_fit_evaluates_through_an_injected_table() {
    // der(y) = -k * gain(y) * y with gain ≡ 1 from the injected table: the
    // fit recovers k against a series generated from k = 2.
    let text = "k = 1\n\
                DYNAMIC decay (method = ode45, time = 0 .. 1, points = 11)\n  \
                der(y) = -k * gain(y) * y\n  y(0) = 1\nEND\n";
    let measured_t: Vec<f64> = (0..11).map(|i| i as f64 * 0.1).collect();
    let measured_v: Vec<f64> = measured_t.iter().map(|t| (-2.0 * t).exp()).collect();
    let request = json!({
        "text": text,
        "parameters": ["k"],
        "initial": [1.0],
        "lower": [0.1],
        "upper": [10.0],
        "odeBlock": "decay",
        "column": "y",
        "measuredT": measured_t,
        "measuredV": measured_v,
        "functionTables": [{
            "name": "gain",
            "argNames": ["y"],
            "curves": [{"param": null, "points": [[0, 1], [2, 1]]}],
        }],
    });
    let out: Value = serde_json::from_str(&frees::parameter_fit(&request.to_string()))
        .expect("valid JSON out");
    assert_eq!(out["success"], true, "{out}");
    let fitted = out["fittedValues"].as_array().unwrap()[0].as_f64().unwrap();
    assert!((fitted - 2.0).abs() < 1e-2, "fitted k = {fitted}, want ≈ 2");
}

// ---------------------------------------------------------------------------
// The REPL cache — `SolveController.computeSolve`'s `replDefs`: request wins.
// ---------------------------------------------------------------------------

#[test]
fn the_repl_calls_the_injected_table_and_the_request_wins_there() {
    // One test, one thread: the REPL session is thread-local state shared by
    // `solve` and `repl_evaluate`.
    //
    // The document defines fcurve ≡ 1000-scale, the request 10-scale. The
    // SOLVE answers from the document (`withExtraDefs`: source wins); the
    // REPL answers from the request (`replDefs.putAll(functionDefs)`:
    // request wins). Both directions are the Java's, each asserted where it
    // holds.
    let out = solve(
        "TABLE fcurve(x)\n1 1000\n2 2000\nEND\ny = fcurve(1.5)\n",
        &json!({"functionTables": [fcurve_dto()]}),
    );
    assert_eq!(out["success"], true, "{out}");
    let y = out["variables"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["name"] == "y")
        .unwrap();
    assert_eq!(y["value"].as_f64().unwrap(), 1500.0, "solve: document wins");

    let repl: Value = serde_json::from_str(&frees::repl_evaluate(
        &json!({"expression": "fcurve(1.5)"}).to_string(),
    ))
    .expect("valid JSON out");
    assert_eq!(repl["success"], true, "{repl}");
    assert_eq!(
        repl["value"].as_f64().unwrap(),
        15.0,
        "REPL: the request's table wins over the document's"
    );
}
