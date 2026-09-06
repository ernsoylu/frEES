//! `SolveRequest.overrides` at the wasm boundary.
//!
//! The REPL terminal and the slider strip re-parameterise a document by
//! sending `"<name> = <value>"` lines beside it (`web/src/api.ts`,
//! `web/src/sliders.ts`), never by editing the editor text. Up to this test
//! the boundary parsed the field and dropped it, so a dragged slider moved
//! nothing and the REPL's assignments were invisible to Solve — while Check,
//! which gates the Solve button, reported on a different model again.
//!
//! `frees_core::analysis::montecarlo::apply_overrides` is the transcription of
//! the Java `SolverApiSupport.applyOverrides` and already had unit coverage;
//! what is asserted here is that Solve and Check both run it, on the same
//! text, in the same place.

use frees::{check, solve};
use serde_json::Value;

fn parsed(payload: &str) -> Value {
    serde_json::from_str(payload).unwrap_or_else(|e| panic!("not JSON ({e}): {payload}"))
}

fn solve_with(source: &str, overrides: &[&str]) -> Value {
    let request = serde_json::json!({ "overrides": overrides }).to_string();
    parsed(&solve(source, &request))
}

fn check_with(source: &str, overrides: &[&str]) -> Value {
    let request = serde_json::json!({ "overrides": overrides }).to_string();
    parsed(&check(source, &request))
}

/// A solved variable's value by name, case-insensitively — `variables[]` rows
/// carry display spellings.
fn value_of(v: &Value, name: &str) -> f64 {
    v["variables"]
        .as_array()
        .unwrap_or_else(|| panic!("no variables array: {v}"))
        .iter()
        .find(|row| {
            row["name"]
                .as_str()
                .is_some_and(|n| n.eq_ignore_ascii_case(name))
        })
        .unwrap_or_else(|| panic!("no variable {name:?} in {v}"))["value"]
        .as_f64()
        .unwrap_or_else(|| panic!("variable {name:?} has a non-numeric value in {v}"))
}

/// The plan's acceptance criterion, verbatim: an override on `x` must reach
/// the *dependent* variable too, which is only true if the document was
/// re-solved rather than patched after the fact.
#[test]
fn an_override_re_parameterises_the_document() {
    let v = solve_with("x = 2\ny = x^2\n", &["x = 3"]);
    assert_eq!(v["success"], true, "{v}");
    assert_eq!(value_of(&v, "x"), 3.0, "{v}");
    assert_eq!(value_of(&v, "y"), 9.0, "{v}");
}

/// The other half of the criterion: Check gates the Solve button, so it has to
/// report on the model Solve will run. `y = x^2` alone is one equation in two
/// unknowns; the override supplies the second equation.
#[test]
fn check_reports_on_the_overridden_model() {
    let bare = check_with("y = x^2\n", &[]);
    assert_eq!(
        bare["solvable"], false,
        "underdetermined without it: {bare}"
    );

    let with = check_with("y = x^2\n", &["x = 3"]);
    assert_eq!(with["solvable"], true, "{with}");
    assert_eq!(with["equations"], 2, "{with}");
    assert_eq!(with["unknowns"], 2, "{with}");
}

/// The document's own assignment is *struck*, not joined — leaving it would
/// make the model overdetermined, which is how a naive append would fail.
#[test]
fn the_overridden_assignment_is_replaced_not_added() {
    let v = check_with("x = 2\ny = x^2\n", &["x = 3"]);
    assert_eq!(v["solvable"], true, "{v}");
    assert_eq!(v["equations"], 2, "{v}");
    assert_eq!(v["unknowns"], 2, "{v}");
}

/// `App.tsx` appends the slider pins after the REPL's assignments and relies on
/// the boundary collapsing the list by name, last wins; names are matched the
/// way the solver matches them, case-insensitively.
#[test]
fn the_last_override_of_a_name_wins_case_insensitively() {
    let v = solve_with("x = 2\ny = x^2\n", &["x = 3", "X = 5"]);
    assert_eq!(v["success"], true, "{v}");
    assert_eq!(value_of(&v, "x"), 5.0, "{v}");
    assert_eq!(value_of(&v, "y"), 25.0, "{v}");
}

/// `sliderOverrideEquation` writes `name = value [unit]`, so an override is an
/// ordinary assignment line and converts to SI like one.
#[test]
fn an_override_carries_its_unit() {
    let v = solve_with("P = 1 [Pa]\nq = 2 * P\n", &["P = 250 [kPa]"]);
    assert_eq!(v["success"], true, "{v}");
    assert_eq!(value_of(&v, "P"), 250_000.0, "{v}");
    assert_eq!(value_of(&v, "q"), 500_000.0, "{v}");
}

/// The envelope minus the one field that is a clock reading rather than an
/// answer — two solves of the same document differ in `stats.elapsedMillis`
/// whenever the box is busy enough for the second to cross a millisecond.
fn without_elapsed(payload: &str) -> Value {
    let mut v = parsed(payload);
    if let Some(stats) = v.get_mut("stats").and_then(Value::as_object_mut) {
        stats.remove("elapsedMillis");
    }
    v
}

/// Every existing caller that sends no overrides — and every fixture replay —
/// must be answered exactly as the pre-override boundary answered. `null` is
/// included because the Java record's field is nullable: it has to mean
/// "none", not "Invalid request".
#[test]
fn an_absent_or_empty_override_list_changes_nothing() {
    let source = "x = 2\ny = x^2\n";
    let baseline = without_elapsed(&solve(source, "{}"));
    assert_eq!(
        without_elapsed(&solve(source, r#"{"overrides": []}"#)),
        baseline
    );
    assert_eq!(
        without_elapsed(&solve(source, r#"{"overrides": null}"#)),
        baseline
    );

    let baseline = check(source, "{}");
    assert_eq!(check(source, r#"{"overrides": []}"#), baseline);
    assert_eq!(check(source, r#"{"overrides": null}"#), baseline);
}

/// A row with no `=`, or an empty name, is dropped rather than parsed — the
/// Java tolerance. It must not become a syntax error in the user's document.
#[test]
fn a_malformed_override_is_ignored() {
    for bad in ["no equals sign", "= 5", ""] {
        let v = solve_with("x = 2\ny = x^2\n", &[bad]);
        assert_eq!(v["success"], true, "{bad:?} broke the solve: {v}");
        assert_eq!(value_of(&v, "y"), 4.0, "{bad:?}: {v}");
    }
}

/// An override that *does* break the document reports the document's own
/// failure envelope, not a boundary exception.
#[test]
fn an_override_that_breaks_the_model_answers_as_data() {
    let v = solve_with("x = 2\ny = x^2\n", &["x = "]);
    assert_eq!(v["success"], false, "{v}");
    assert!(v["error"].is_string(), "{v}");
}

// ─────────────────────────────────────────────────────────────────────────────
// The substitution is textual, per `;`-segment, per line. These three tests
// pin what that means where the document is not a flat list of assignments —
// the cases the plan asked to cover before this helper took a second caller.
// ─────────────────────────────────────────────────────────────────────────────

/// `x = 2; y = x^2` on one line: only the overridden *segment* is struck, and
/// the survivors are re-joined with `;`. Striking the whole line would delete
/// `y`'s defining equation with it.
#[test]
fn only_the_overridden_semicolon_segment_is_struck() {
    let v = solve_with("x = 2; y = x^2\n", &["x = 3"]);
    assert_eq!(v["success"], true, "{v}");
    assert_eq!(value_of(&v, "x"), 3.0, "{v}");
    assert_eq!(value_of(&v, "y"), 9.0, "{v}");
}

/// A comment is not an assignment: `// x = 9` does not begin with the name, so
/// it survives, and the real `x = 2` a line below is the one struck.
#[test]
fn a_comment_is_not_mistaken_for_the_assignment() {
    let v = solve_with("// x = 9 would be too big\nx = 2\ny = x^2\n", &["x = 3"]);
    assert_eq!(v["success"], true, "{v}");
    assert_eq!(value_of(&v, "y"), 9.0, "{v}");
}

/// A `FUNCTION` body assigns with `:=`, which is not `=`, so a formal or a
/// local of the same name as an overridden variable is left alone.
#[test]
fn a_function_body_assignment_is_left_alone() {
    let source = "FUNCTION Twice(f)\n  Twice := 2 * f\nEND\n\nf = 2\ny = Twice(f)\n";
    let v = solve_with(source, &["f = 5"]);
    assert_eq!(v["success"], true, "{v}");
    assert_eq!(value_of(&v, "y"), 10.0, "{v}");
}

/// A `MODULE` body *does* assign with `=`, and the substitution cannot see
/// that the line is block-local: overriding a body name deletes the module's
/// own equation. The engine reports that as an ordinary document failure, and
/// the UI already keeps a user out of it — `pinnableParameters` (`App.tsx`)
/// offers a slider only for names the document assigns a *literal*, which a
/// body equation is not. Pinned here so a future widening of that rule has to
/// come past this test.
#[test]
fn overriding_a_module_body_name_breaks_the_module_as_data() {
    let source = "MODULE Doubler(a : b)\n  b = 2 * a\nEND\n\nCALL Doubler(5 : q)\n";
    let clean = solve_with(source, &[]);
    assert_eq!(clean["success"], true, "{clean}");
    assert_eq!(value_of(&clean, "q"), 10.0, "{clean}");

    let v = solve_with(source, &["b = 7"]);
    assert_eq!(v["success"], false, "the module lost its body: {v}");
    assert!(v["error"].is_string(), "{v}");
}

/// The one comment form the textual substitution *can* damage: a `{ … }`
/// comment spanning lines, whose inner line happens to read `name = …`. That
/// line is struck like any other, the `{` loses its `}`, and the document
/// stops parsing. Pinned as a known edge rather than fixed: making the
/// substitution comment-aware would mean re-tokenising the document inside a
/// helper that Monte Carlo and parameter estimation call once per sample, and
/// whose behaviour their fixtures already encode. The failure is at least
/// loud — a syntax error, never a silently different answer.
#[test]
fn a_multiline_comment_containing_an_assignment_is_a_known_edge() {
    let source = "{ note:\nx = 9 in the old design }\nx = 2\ny = x^2\n";
    assert_eq!(solve_with(source, &[])["success"], true, "clean baseline");

    let v = solve_with(source, &["x = 3"]);
    assert_eq!(v["success"], false, "{v}");
    assert!(
        v["error"]
            .as_str()
            .is_some_and(|e| e.contains("unterminated comment")),
        "{v}"
    );
}
