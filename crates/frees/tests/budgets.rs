use serde_json::Value;

#[test]
fn steady_solve_over_elapsed_budget_stops_with_deadline_error() {
    // A steady solve given a virtually zero elapsed-time budget
    let doc = "x^2 + y^2 = 25\nx - y = 1\n";
    let req = r#"{"stopCriteria": {"elapsedTimeSeconds": 1e-9}}"#;
    let out: Value = serde_json::from_str(&frees::solve(doc, req)).unwrap();

    assert_eq!(out["success"], false, "{out}");
    let error = out["error"].as_str().unwrap();
    assert!(
        error.contains("exceeded its 0-second wall-clock budget") || error.contains("budget"),
        "{error}"
    );

    // Subsequent solve with normal budget succeeds (deadline was cleaned up)
    let normal_out: Value = serde_json::from_str(&frees::solve(doc, "{}")).unwrap();
    assert_eq!(normal_out["success"], true, "{normal_out}");
}

#[test]
fn table_solve_honours_elapsed_budget_and_aborts_sweep() {
    let doc = "y = 2 * x\n";
    let req = r#"{
        "stopCriteria": {"elapsedTimeSeconds": 1e-9},
        "table": {
            "variables": ["x", "y"],
            "rows": [{"x": 1}, {"x": 2}, {"x": 3}, {"x": 4}, {"x": 5}]
        }
    }"#;
    let out: Value = serde_json::from_str(&frees::solve_table(doc, req)).unwrap();

    // Table solve returns error or marks results as failed
    if let Some(err) = out.get("error").and_then(|e| e.as_str()) {
        assert!(err.contains("budget") || err.contains("exceeded"), "{err}");
    } else if let Some(results) = out.get("results").and_then(|r| r.as_array()) {
        let any_failed = results.iter().any(|r| r["success"] == false);
        assert!(any_failed, "at least one row must be stopped: {out}");
    }
}
