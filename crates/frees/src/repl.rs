//! `POST /api/repl/evaluate` and `POST /api/repl/clear`, in the browser.
//!
//! Port of `../frEES/backend/web/.../api/ReplEvaluator.java` (1,568 LOC) plus
//! the slice of `SolveContextCache` it reads. The Java caches one solved
//! workspace per HTTP session and evaluates each REPL line against it; the
//! browser has exactly one session, so the workspace is a module-level cell
//! that [`crate::solve`] refreshes on every **successful** solve — a failed
//! solve leaves the previous workspace in place, as the Java cache does.
//!
//! # What is ported, and what is refused by name
//!
//! `ReplEvaluator.evaluate` is a seven-way dispatch. Four arms are here:
//!
//! | Arm | Status |
//! |---|---|
//! | `CAS_CALL` — the 13 symbolic ops | **ported**, via [`frees_core::cas`] |
//! | assignment — `x = 42` | **ported** (scalar) |
//! | bare-variable echo — `T_1` | **ported** |
//! | general expression — `2*sqrt(9)+4` | **ported** |
//! | `CALL name(in : out)` | **refused by name** |
//! | matrix/vector literal and range vector (`[1 2 3]`, `1:2:10`) | **refused by name** |
//! | single-unknown equation solve (`P = 50000 * v`) | **refused by name** |
//!
//! Each refusal says what it is and that it is not ported, so a user never
//! gets a wrong answer in place of a missing feature. `ReplTerminal.tsx`'s
//! own `help` text still advertises all seven; the three refusals print
//! instead of misbehaving.
//!
//! # The one behavioural divergence: units of a computed expression
//!
//! The Java derives a result's unit with `ReplDimensions.dimensionOf`, a
//! `backend/web` class that walks the expression combining its leaves'
//! `Quantity` dimensions. That class is **not ported** (nothing in
//! `frees-core` exposes an expression-level dimension walker), so:
//!
//! * a **bare variable** echoes its recorded display value and unit — exact;
//! * a **computed expression** reports its **SI** value with no unit, where
//!   the Java would report the display value and unit.
//!
//! For a dimensionless expression the two agree. For `2*T_1` with `T_1` in
//! `[C]` they do not, and that is stated rather than hidden.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

use frees_core::cas::{engine as cas, laplace};
use frees_core::engine::Solution;
use frees_core::units::registry::UnitSystem;
use frees_core::{Expr, FreesError};
use serde_json::{json, Value};

use crate::to_display;

// ---------------------------------------------------------------------------
// The cached workspace — `SolveContextCache.Session`
// ---------------------------------------------------------------------------

/// One workspace row: what the Solution window shows plus the SI value the
/// evaluator needs. The Java `SolveContextCache.ReplVar` carries the first
/// three; `si` is `session.siValues().get(name)`.
#[derive(Debug, Clone)]
struct ReplVar {
    si: f64,
    display: f64,
    unit: String,
    uncertainty: Option<f64>,
}

/// The last successful solve, as the REPL sees it.
#[derive(Debug, Default)]
struct Session {
    /// Source of that solve. Re-parsed per line so a document's `FUNCTION` /
    /// `PROCEDURE` / `TABLE` definitions are in scope, which is what
    /// `session.defs()` is on the Java side.
    source: String,
    /// Lowercase canonical name → its row. Ordered so `repl_clear` and the
    /// assigned-variable list are deterministic.
    vars: BTreeMap<String, ReplVar>,
    /// Lowercase canonical name → the source spelling to print.
    display_names: BTreeMap<String, String>,
    /// Names this REPL defined or overrode — the only ones `repl_clear` drops.
    defined: BTreeSet<String>,
    /// The solve request's GUI Function Tables, already converted to defs —
    /// the Java caches them merged into `replDefs`
    /// (`SolveController.computeSolve`: `replDefs.putAll(functionDefs)`, so
    /// the **request wins** over a document definition of the same name —
    /// note this is the opposite of the solve path's `withExtraDefs` rule).
    extra_tables: Vec<frees_core::parser::defs::FunctionTableDef>,
    /// True once a solve has stored a workspace.
    solved: bool,
}

thread_local! {
    static SESSION: RefCell<Session> = RefCell::new(Session::default());
}

/// `SolveContextCache.put`: record the workspace this solve produced.
///
/// Rows are keyed by the **lowercase canonical** name, the key
/// `frees_core::eval` resolves a variable against, while the display spelling
/// and display-unit conversion mirror `SolveController.toVariableDto` so the
/// REPL echoes exactly what the Solution window shows.
pub fn store_session(
    source: &str,
    solution: &Solution,
    system: UnitSystem,
    explicit_units: &BTreeMap<String, String>,
    extra_tables: &[frees_core::parser::defs::FunctionTableDef],
) {
    let mut vars = BTreeMap::new();
    for (canonical, si) in &solution.values {
        let display_name = solution
            .display_names
            .get(canonical)
            .unwrap_or(canonical)
            .clone();
        let key = display_name.to_ascii_lowercase();
        let unit = solution
            .inferred_units
            .get(&key)
            .map(String::as_str)
            .unwrap_or("");
        let si_unc = solution.uncertainties.get(&key).copied();
        let (display, display_unit, display_unc) =
            to_display(&key, *si, si_unc, unit, system, explicit_units);
        vars.insert(
            canonical.clone(),
            ReplVar {
                si: *si,
                display,
                unit: display_unit,
                uncertainty: display_unc.filter(|u| u.is_finite() && *u != 0.0),
            },
        );
    }
    SESSION.with(|cell| {
        let mut session = cell.borrow_mut();
        // REPL-defined overlays survive a solve on the Java side too: the
        // cache stores the solve, and `session.define` rows sit on top until
        // `clear` drops them. Re-inserting them after the solve rows keeps
        // that precedence.
        let overlays: Vec<(String, ReplVar)> = session
            .defined
            .iter()
            .filter_map(|name| session.vars.get(name).map(|v| (name.clone(), v.clone())))
            .collect();
        session.source = source.to_string();
        session.display_names = solution.display_names.clone();
        session.extra_tables = extra_tables.to_vec();
        session.vars = vars;
        for (name, var) in overlays {
            session.vars.insert(name, var);
        }
        session.solved = true;
    });
}

/// `POST /api/repl/clear`: drop every REPL-defined overlay, or just one.
/// A fire-and-forget no-op when nothing matches, like the Java endpoint.
pub fn clear(name: Option<&str>) {
    SESSION.with(|cell| {
        let mut session = cell.borrow_mut();
        match name {
            Some(name) => {
                let key = name.to_ascii_lowercase();
                session.defined.remove(&key);
                session.vars.remove(&key);
                session.display_names.remove(&key);
            }
            None => {
                for key in std::mem::take(&mut session.defined) {
                    session.vars.remove(&key);
                    session.display_names.remove(&key);
                }
            }
        }
    });
}

// ---------------------------------------------------------------------------
// The Outcome record
// ---------------------------------------------------------------------------

/// Port of `ReplEvaluator.Outcome`, serialised as `api.ts`'s `ReplResponse`.
struct Outcome {
    success: bool,
    value: Option<f64>,
    text: Option<String>,
    unit: Option<String>,
    uncertainty: Option<f64>,
    error: Option<String>,
    name: Option<String>,
    assigned: Vec<Value>,
}

impl Outcome {
    fn ok(value: f64, text: String, unit: String, uncertainty: Option<f64>) -> Outcome {
        Outcome {
            success: true,
            value: Some(value),
            text: Some(text),
            unit: Some(unit),
            uncertainty,
            error: None,
            name: None,
            assigned: Vec::new(),
        }
    }

    fn fail(error: impl Into<String>) -> Outcome {
        Outcome {
            success: false,
            value: None,
            text: None,
            unit: None,
            uncertainty: None,
            error: Some(error.into()),
            name: None,
            assigned: Vec::new(),
        }
    }

    fn named(mut self, name: impl Into<String>) -> Outcome {
        self.name = Some(name.into());
        self
    }

    fn with_variables(mut self, assigned: Vec<Value>) -> Outcome {
        self.assigned = assigned;
        self
    }

    fn to_json(&self) -> String {
        json!({
            "success": self.success,
            "value": self.value.and_then(crate::json_number),
            "text": self.text,
            "units": self.unit,
            "uncertainty": self.uncertainty.and_then(crate::json_number),
            "error": self.error,
            "name": self.name,
            "assignedVariables": self.assigned,
        })
        .to_string()
    }
}

// ---------------------------------------------------------------------------
// Entry point — `ReplEvaluator.evaluate`
// ---------------------------------------------------------------------------

/// Evaluate one REPL line against the cached workspace. Returns a
/// `ReplResponse` JSON string; never a JS exception, because
/// `ReplTerminal.tsx` calls it without a `catch`.
pub fn evaluate(expression: &str, unit_system: UnitSystem) -> String {
    let _ = unit_system; // the workspace is already stored in display units
    let input = expression.trim();
    if input.is_empty() {
        return Outcome::fail("Empty expression.").to_json();
    }
    // `evaluate`'s first two guards, in the Java's order: the session check
    // precedes even the CAS branch, so a CAS call before the first solve says
    // "solve the document first" rather than answering.
    if !SESSION.with(|cell| cell.borrow().solved) {
        return Outcome::fail("No solved context for this session yet — solve the document first.")
            .to_json();
    }

    if let Some((function, args)) = split_cas_call(input) {
        return evaluate_cas(&function, &args).to_json();
    }
    if starts_with_word(input, "call") {
        return Outcome::fail(
            "CALL lines are not supported by the browser REPL yet — put the CALL in the \
             document and press Solve.",
        )
        .to_json();
    }
    if let Some((target, rhs)) = split_assignment(input) {
        return assignment(&target, &rhs).to_json();
    }
    if looks_like_range(input) {
        return Outcome::fail(
            "Range vectors are not supported by the browser REPL yet — declare the array in \
             the document.",
        )
        .to_json();
    }

    // Anything with a top-level `=` that was not an assignment is the Java's
    // `Expr.Compare` arm — solve-for-the-single-unknown. This port's expression
    // grammar does not admit a bare `=` at all (it would be a trailing-input
    // syntax error), so the refusal has to be by name here rather than after a
    // parse.
    if has_top_level_equals(input) {
        return Outcome::fail(
            "Solving an equation for its single unknown is not supported by the browser REPL \
             yet — put the equation in the document and press Solve.",
        )
        .to_json();
    }

    let expr = match cas::parse_expression(input) {
        Ok(expr) => expr,
        Err(err) => return Outcome::fail(format!("Syntax error: {err}")).to_json(),
    };
    if matches!(expr, Expr::ArrayLiteral(_)) {
        return Outcome::fail(
            "Matrix and vector literals are not supported by the browser REPL yet — declare \
             the array in the document.",
        )
        .to_json();
    }

    // A bare variable query echoes exactly what the workspace shows.
    if let Expr::Var(name) = &expr {
        let key = name.to_ascii_lowercase();
        if let Some(var) = SESSION.with(|cell| cell.borrow().vars.get(&key).cloned()) {
            define(
                "ans",
                var.si,
                var.display,
                &var.unit,
                var.uncertainty,
                false,
            );
            return Outcome::ok(
                var.display,
                format(var.display, &var.unit, var.uncertainty),
                var.unit,
                var.uncertainty,
            )
            .named("ans")
            .to_json();
        }
    }

    evaluated(&expr, None).to_json()
}

/// The general-expression arm of `ReplEvaluator.evaluate`, plus the
/// `assignTo` branch `assignment` shares with it (`ReplEvaluator.evaluated`).
///
/// The unit is empty by construction — see the module docs' divergence note.
fn evaluated(expr: &Expr, assign_to: Option<&str>) -> Outcome {
    let si = match eval_in_workspace(expr) {
        Ok(value) => value,
        Err(err) => return Outcome::fail(err.to_string_message()),
    };
    if si.is_nan() {
        return Outcome::fail("Result is undefined (NaN).");
    }
    let body = format(si, "", None);
    match assign_to {
        None => {
            define("ans", si, si, "", None, false);
            Outcome::ok(si, body, String::new(), None).named("ans")
        }
        Some(target) => {
            let key = target.to_ascii_lowercase();
            define(&key, si, si, "", None, true);
            remember_spelling(&key, target);
            Outcome::ok(si, format!("{target} = {body}"), String::new(), None)
                .named(target)
                .with_variables(vec![json!({
                    "name": target,
                    "value": crate::json_number(si),
                    "units": "",
                    "uncertainty": Value::Null,
                })])
        }
    }
}

/// Port of the scalar path of `ReplEvaluator.assignment`.
fn assignment(target: &str, rhs: &str) -> Outcome {
    if !is_identifier(target) {
        return Outcome::fail(format!("Invalid assignment target: {target}"));
    }
    if looks_like_range(rhs) {
        return Outcome::fail(
            "Range vectors are not supported by the browser REPL yet — declare the array in \
             the document.",
        );
    }
    let expr = match cas::parse_expression(rhs) {
        Ok(expr) => expr,
        Err(err) => return Outcome::fail(format!("Syntax error: {err}")),
    };
    if matches!(expr, Expr::ArrayLiteral(_)) {
        return Outcome::fail(
            "Matrix and vector literals are not supported by the browser REPL yet — declare \
             the array in the document.",
        );
    }
    evaluated(&expr, Some(target))
}

/// Evaluate against the workspace's **SI** values, with the last solved
/// document's definitions in scope — the Java
/// `Evaluator.eval(expr, session.siValues(), session.defs())`.
///
/// [`frees_core::eval`] is the engine's own numeric AST walker over an
/// already-parsed [`Expr`]: it resolves names against `scope` and intrinsics
/// against the registry, and executes nothing. There is no `eval`-of-source
/// path here.
fn eval_in_workspace(expr: &Expr) -> Result<f64, FreesError> {
    let (scope, source, extra_tables) = SESSION.with(|cell| {
        let session = cell.borrow();
        let scope: frees_core::eval::Scope = session
            .vars
            .iter()
            .map(|(name, var)| (name.clone(), var.si))
            .collect();
        (scope, session.source.clone(), session.extra_tables.clone())
    });
    // The document's definitions come from re-parsing the stored source. A
    // document that no longer parses (the editor changed under the REPL)
    // simply contributes none, which is the Java behaviour when the cached
    // `defs` are stale. The solve request's GUI Function Tables were converted
    // at request time, so they stay callable either way — and they merge with
    // the *REPL* direction, the Java's
    // `replDefs = new HashMap<>(parsed.defs()); replDefs.putAll(functionDefs)`
    // (`SolveController.computeSolve`): a request table REPLACES a document
    // definition of the same name, the opposite of the solve path's
    // `withExtraDefs` rule.
    let mut defs = match frees_core::parse_document(&source) {
        Ok(document) => document.defs,
        Err(_) => frees_core::parser::defs::Definitions::default(),
    };
    for table in extra_tables {
        defs.functions.retain(|f| f.name != table.name);
        defs.procedures.retain(|p| p.name != table.name);
        defs.modules.retain(|m| m.name != table.name);
        defs.tables.retain(|t| t.name != table.name);
        defs.tables.push(table);
    }
    frees_core::eval::eval_with(
        expr,
        &scope,
        frees_core::eval::EvalContext::with_defs(&defs),
    )
}

/// `session.define` — record a value the REPL produced. `overlay` marks it as
/// REPL-defined so `clear` knows to drop it; `ans` is not an overlay (the Java
/// `clear` endpoint leaves it alone).
fn define(name: &str, si: f64, display: f64, unit: &str, uncertainty: Option<f64>, overlay: bool) {
    SESSION.with(|cell| {
        let mut session = cell.borrow_mut();
        session.vars.insert(
            name.to_ascii_lowercase(),
            ReplVar {
                si,
                display,
                unit: unit.to_string(),
                uncertainty,
            },
        );
        if overlay {
            session.defined.insert(name.to_ascii_lowercase());
        }
    });
}

fn remember_spelling(key: &str, spelling: &str) {
    SESSION.with(|cell| {
        cell.borrow_mut()
            .display_names
            .insert(key.to_string(), spelling.to_string());
    });
}

// ---------------------------------------------------------------------------
// The CAS arm — `ReplEvaluator.evaluateCas`
// ---------------------------------------------------------------------------

/// The 13 spellings `ReplEvaluator.CAS_CALL` matches, lowercase.
const CAS_NAMES: [&str; 14] = [
    "factor",
    "expand",
    "simplify",
    "together",
    "cancel",
    "apart",
    "laplace",
    "inverselaplace",
    "ilaplace",
    "numerator",
    "denominator",
    "collect",
    "diff",
    "integrate",
];

/// Port of `ReplEvaluator.evaluateCas`.
///
/// The eleven rational/derivative/integral ops route into
/// [`frees_core::cas::engine`]; `laplace` / `inverselaplace` / `ilaplace` route
/// into [`frees_core::cas::laplace`], which is where the transform table lives
/// (`Op::from_repl_name` returns `None` for them on purpose).
///
/// `CasError::NoClosedForm` renders as the Java's
/// `"<fn>: no closed form found for this input."` — the sentence
/// `isUnevaluated` produced by sniffing Symja's echoed output. Every other
/// failure renders as `"CAS: " + message`, also verbatim.
fn evaluate_cas(function: &str, args_text: &str) -> Outcome {
    let name = function.to_ascii_lowercase();
    let args = split_top_level_commas(args_text);
    if args.is_empty() || args[0].trim().is_empty() {
        return Outcome::fail(format!(
            "{function} needs an expression, e.g. {function}(x^2 - 1)"
        ));
    }
    let expression = strip_quotes(&args[0]);
    let variable = |index: usize, fallback: &str| -> String {
        args.get(index)
            .map(|a| strip_quotes(a))
            .filter(|a| !a.is_empty())
            .unwrap_or_else(|| fallback.to_string())
    };

    // Laplace first: these two are the operations `cas::engine::Op` refuses by
    // name, because their implementation is a transform table rather than the
    // rational core.
    if matches!(name.as_str(), "laplace" | "inverselaplace" | "ilaplace") {
        let parsed = match cas::parse_expression(&expression) {
            Ok(expr) => expr,
            Err(err) => return Outcome::fail(format!("CAS: {err}")),
        };
        let transformed = if name == "laplace" {
            laplace::transform(&parsed, &variable(1, "t"), &variable(2, "s"))
        } else {
            laplace::inverse_transform(&parsed, &variable(1, "s"), &variable(2, "t"))
        };
        return match transformed {
            Ok(result) => Outcome::ok(0.0, cas_display(&result), String::new(), None),
            // The transform modules report a refusal as an ordinary engine
            // error; the REPL's contract for "I cannot do this one" is the
            // no-closed-form sentence.
            Err(_) => Outcome::fail(format!("{function}: no closed form found for this input.")),
        };
    }

    let Some(op) = cas::Op::from_repl_name(&name) else {
        return Outcome::fail(format!("Unknown CAS function: {function}"));
    };
    if op.needs_variable() && args.len() < 2 {
        return Outcome::fail(match name.as_str() {
            "apart" => "apart needs a variable: apart(expr, var), e.g. \
                        apart((s+3)/(s^2+3*s+2), s)"
                .to_string(),
            other => format!("{other} needs a variable: {other}(expr, var)"),
        });
    }
    let result = if op.needs_variable() {
        cas::apply_with_variable(op, &expression, &strip_quotes(&args[1]))
    } else {
        cas::apply(op, &expression)
    };
    match result {
        // Ledger item 25: a ceiling that silently declined work (an opaque
        // oversized power, an over-degree Apart) rides in `note`, printed
        // under the value so "returned unfactored" is distinguishable from
        // "proved irreducible". The value line itself stays byte-identical.
        Ok(value) => match value.note {
            Some(note) => Outcome::ok(
                0.0,
                format!("{}\n  (declined: {note})", value.text),
                String::new(),
                None,
            ),
            None => Outcome::ok(0.0, value.text, String::new(), None),
        },
        Err(cas::CasError::NoClosedForm { .. }) => {
            Outcome::fail(format!("{function}: no closed form found for this input."))
        }
        Err(err) => Outcome::fail(format!("CAS: {err}")),
    }
}

/// `casDisplay`: the engine's own printed form. The Java rewrote Symja's
/// `Log(`/`Log10(` here; `cas::ops::display` emits `ln(`/`log10(` already.
fn cas_display(expr: &Expr) -> String {
    frees_core::cas::ops::display(expr)
}

// ---------------------------------------------------------------------------
// Line shapes — the Java's four regexes, hand-written
// ---------------------------------------------------------------------------

/// `CAS_CALL`: `<name> ( <args> )` spanning the whole line. Returns the name
/// as written (the failure messages echo the user's spelling) and the argument
/// text between the outermost parentheses.
fn split_cas_call(input: &str) -> Option<(String, String)> {
    let open = input.find('(')?;
    if !input.trim_end().ends_with(')') {
        return None;
    }
    let name = input[..open].trim();
    if !CAS_NAMES.contains(&name.to_ascii_lowercase().as_str()) {
        return None;
    }
    let close = input.rfind(')')?;
    if close < open {
        return None;
    }
    Some((name.to_string(), input[open + 1..close].to_string()))
}

/// `ASSIGNMENT`: `name = rhs` where the `=` is not part of `==`, `<=`, `>=` or
/// `<>`. The Java regex also admits an `x[i]` target; an indexed assignment
/// reaches the matrix path this port refuses, so only the bare form is split
/// here and `x[1] = 2` falls through to the expression parser.
fn split_assignment(input: &str) -> Option<(String, String)> {
    let bytes = input.as_bytes();
    let eq = input.find('=')?;
    if bytes.get(eq + 1) == Some(&b'=') {
        return None;
    }
    if eq > 0 && matches!(bytes[eq - 1], b'<' | b'>' | b'=' | b'!') {
        return None;
    }
    let target = input[..eq].trim();
    let rhs = input[eq + 1..].trim();
    if target.is_empty() || rhs.is_empty() || !is_identifier(target) {
        return None;
    }
    Some((target.to_string(), rhs.to_string()))
}

/// A `=` outside brackets and quotes that is not part of `==`, `<=`, `>=`,
/// `<>` or `!=` — i.e. the equation form.
fn has_top_level_equals(input: &str) -> bool {
    let bytes = input.as_bytes();
    let mut depth = 0i32;
    let mut quote: Option<u8> = None;
    for (i, byte) in bytes.iter().copied().enumerate() {
        match quote {
            Some(q) => {
                if byte == q {
                    quote = None;
                }
            }
            None => match byte {
                b'\'' | b'"' => quote = Some(byte),
                b'(' | b'[' | b'{' => depth += 1,
                b')' | b']' | b'}' => depth -= 1,
                b'=' if depth == 0 => {
                    let after_operator = i > 0 && matches!(bytes[i - 1], b'<' | b'>' | b'=' | b'!');
                    let before_equals = bytes.get(i + 1) == Some(&b'=');
                    if !after_operator && !before_equals {
                        return true;
                    }
                }
                _ => {}
            },
        }
    }
    false
}

/// `tryParseAndEvaluateRange`'s shape test: `a:b` or `a:b:c`, optionally
/// wrapped in brackets. Used only to refuse by name.
fn looks_like_range(input: &str) -> bool {
    let mut text = input.trim();
    if text.len() >= 2 && text.starts_with('[') && text.ends_with(']') {
        text = text[1..text.len() - 1].trim();
    }
    let parts = text.split(':').count();
    (parts == 2 || parts == 3) && !text.is_empty()
}

/// A leading keyword, case-insensitively, followed by a word boundary — the
/// Java `CALL_PREFIX` pattern.
fn starts_with_word(input: &str, word: &str) -> bool {
    let trimmed = input.trim_start();
    if trimmed.len() < word.len() || !trimmed[..word.len()].eq_ignore_ascii_case(word) {
        return false;
    }
    match trimmed.as_bytes().get(word.len()) {
        None => true,
        Some(byte) => !byte.is_ascii_alphanumeric() && *byte != b'_',
    }
}

fn is_identifier(text: &str) -> bool {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) if first.is_ascii_alphabetic() || first == '_' => {
            chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
        }
        _ => false,
    }
}

/// `splitTopLevelCommas`: split on commas outside brackets and quotes.
fn split_top_level_commas(text: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut quote: Option<char> = None;
    let mut current = String::new();
    for c in text.chars() {
        match quote {
            Some(q) => {
                current.push(c);
                if c == q {
                    quote = None;
                }
                continue;
            }
            None => match c {
                '\'' | '"' => {
                    quote = Some(c);
                    current.push(c);
                    continue;
                }
                '(' | '[' | '{' => depth += 1,
                ')' | ']' | '}' => depth -= 1,
                ',' if depth == 0 => {
                    parts.push(std::mem::take(&mut current));
                    continue;
                }
                _ => {}
            },
        }
        current.push(c);
    }
    if !current.trim().is_empty() || !parts.is_empty() {
        parts.push(current);
    }
    parts
}

/// `stripQuotes`: drop one layer of surrounding single/double quotes.
fn strip_quotes(text: &str) -> String {
    let trimmed = text.trim();
    let bytes = trimmed.as_bytes();
    if trimmed.len() >= 2 {
        let first = bytes[0];
        let last = bytes[trimmed.len() - 1];
        if (first == b'\'' && last == b'\'') || (first == b'"' && last == b'"') {
            return trimmed[1..trimmed.len() - 1].trim().to_string();
        }
    }
    trimmed.to_string()
}

// ---------------------------------------------------------------------------
// Formatting — `ReplEvaluator.format` / `number`
// ---------------------------------------------------------------------------

/// `format(value, unit, uncertainty)`, transcribed: `"-"` is the checker's
/// dimensionless marker and is never bracketed.
fn format(value: f64, unit: &str, uncertainty: Option<f64>) -> String {
    let mut out = number(value);
    if let Some(u) = uncertainty {
        if u != 0.0 && !u.is_nan() {
            out.push_str(" ± ");
            out.push_str(&number(u));
        }
    }
    if !unit.trim().is_empty() && unit != "-" {
        out.push_str(" [");
        out.push_str(unit);
        out.push(']');
    }
    out
}

/// `number(v)`: integers without a decimal point, everything else to six
/// significant figures with trailing zeros trimmed.
///
/// The decimal exponent is read off Rust's own `{:e}` rendering rather than
/// computed with `log10`, so no transcendental is involved and the digit count
/// cannot drift at an exact power of ten. Java's `%.6g` and this agree inside
/// the fixed-notation window; outside it the exponent *spelling* differs
/// (`1.00000e-7` here, `1.00000e-07` in Java). That is display-only — no
/// parity fixture reads it.
fn number(v: f64) -> String {
    if !v.is_finite() {
        return v.to_string();
    }
    // `fract() == 0.0` is the integer test this workspace already uses
    // (`components::expander`, `ode::dynamic`); it is exact for every finite
    // f64, not an approximate comparison.
    if v.fract() == 0.0 && v.abs() < 1e15 {
        return format!("{}", v as i64);
    }
    let magnitude = v.abs();
    let mut text = if (1e-4..1e6).contains(&magnitude) {
        // `%.6g` inside the fixed-notation window: six significant digits.
        let decimals = (5 - decimal_exponent(v)).max(0) as usize;
        format!("{v:.decimals$}")
    } else {
        format!("{v:.5e}")
    };
    if text.contains('.') && !text.contains('e') && !text.contains('E') {
        while text.ends_with('0') {
            text.pop();
        }
        if text.ends_with('.') {
            text.pop();
        }
    }
    text
}

/// `floor(log10(|v|))` for a finite non-zero `v`, taken from the exponent
/// `{:e}` prints. `0` for a zero or a value whose rendering carries no `e`.
fn decimal_exponent(v: f64) -> i32 {
    let rendered = format!("{v:e}");
    match rendered.split_once('e') {
        Some((_, exponent)) => exponent.parse().unwrap_or(0),
        None => 0,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn parsed(json: &str) -> Value {
        serde_json::from_str(json).expect("ReplResponse is JSON")
    }

    /// Each test drives the whole boundary, so the session has to be seeded
    /// the way `solve` seeds it.
    fn seed(source: &str) {
        let solution = frees_core::solve(source, &frees_core::SolverSettings::default())
            .expect("the fixture document solves");
        store_session(source, &solution, UnitSystem::Si, &BTreeMap::new(), &[]);
    }

    fn line(input: &str) -> Value {
        parsed(&evaluate(input, UnitSystem::Si))
    }

    #[test]
    fn without_a_solve_every_line_says_so() {
        SESSION.with(|cell| *cell.borrow_mut() = Session::default());
        let v = line("2 + 2");
        assert_eq!(v["success"], false, "{v}");
        assert!(
            v["error"].as_str().unwrap().contains("solve the document"),
            "{v}"
        );
    }

    #[test]
    fn a_bare_variable_echoes_its_workspace_row() {
        seed("T_out = 300 [K]\n");
        let v = line("T_out");
        assert_eq!(v["success"], true, "{v}");
        assert_eq!(v["value"].as_f64(), Some(300.0), "{v}");
        assert_eq!(v["text"], "300 [K]", "{v}");
        assert_eq!(v["name"], "ans", "{v}");
    }

    #[test]
    fn an_expression_evaluates_against_the_workspace() {
        seed("x = 9\n");
        let v = line("2 * sqrt(x) + 4");
        assert_eq!(v["success"], true, "{v}");
        assert_eq!(v["value"].as_f64(), Some(10.0), "{v}");
        assert_eq!(v["text"], "10", "{v}");
    }

    #[test]
    fn an_assignment_defines_a_variable_visible_to_later_lines() {
        seed("x = 2\n");
        let v = line("y = 3 * x");
        assert_eq!(v["success"], true, "{v}");
        assert_eq!(v["text"], "y = 6", "{v}");
        assert_eq!(v["name"], "y", "{v}");
        assert_eq!(v["assignedVariables"][0]["name"], "y", "{v}");
        assert_eq!(line("y + 1")["value"].as_f64(), Some(7.0));
        clear(None);
        assert_eq!(line("y + 1")["success"], false, "cleared");
    }

    #[test]
    fn a_document_function_is_in_scope() {
        seed("FUNCTION twice(a)\n  twice := 2 * a\nEND\nz = twice(4)\n");
        let v = line("twice(10)");
        assert_eq!(v["value"].as_f64(), Some(20.0), "{v}");
    }

    /// All eleven rational/derivative/integral spellings, asserted against the
    /// engine's own printed form (ascending-power ordering, which is the
    /// Symja convention `cas::ops::display` reproduces).
    #[test]
    fn the_cas_commands_all_answer() {
        seed("x = 1\n");
        for (input, expected) in [
            ("Factor(x^2 - 1)", "(-1+x)*(1+x)"),
            ("Expand((x+1)^2)", "1+2*x+x^2"),
            ("Simplify((x^2-1)/(x-1))", "1+x"),
            ("Together(1/x + 1/y)", "(x+y)/(x*y)"),
            ("Cancel((x^2-1)/(x-1))", "1+x"),
            ("Numerator((s+1)/(s+2))", "1+s"),
            ("Denominator((s+1)/(s+2))", "2+s"),
            ("Collect(a*x^2+b*x^2+x, x)", "x+(a+b)*x^2"),
            ("Diff(x^3, x)", "3*x^2"),
            ("Integrate(x^2, x)", "x^3/3"),
            ("Apart((s+3)/(s^2+3*s+2), s)", "2/(1+s)-1/(2+s)"),
        ] {
            let v = line(input);
            assert_eq!(v["success"], true, "{input}: {v}");
            assert_eq!(v["text"], expected, "{input}: {v}");
        }
    }

    /// The two ops `Op::from_repl_name` deliberately returns `None` for: they
    /// route to `cas::laplace`, not the rational core. `ilaplace` is the
    /// alias `ReplEvaluator.CAS_CALL` also matches.
    #[test]
    fn laplace_and_its_inverse_route_to_the_transform_table() {
        seed("x = 1\n");
        assert_eq!(line("Laplace(t, t, s)")["text"], "1/s^2");
        assert_eq!(line("InverseLaplace(1/s^2, s, t)")["text"], "t");
        assert_eq!(line("ilaplace(1/s^2, s, t)")["text"], "t");
        // Defaulted variable names: `laplace(f)` means `laplace(f, t, s)`.
        assert_eq!(line("Laplace(t)")["text"], "1/s^2");
    }

    #[test]
    fn a_cas_op_that_needs_a_variable_says_which() {
        seed("x = 1\n");
        let v = line("apart(1/(s+1))");
        assert_eq!(v["success"], false, "{v}");
        assert!(
            v["error"].as_str().unwrap().contains("needs a variable"),
            "{v}"
        );
    }

    #[test]
    fn an_unreachable_cas_answer_is_the_no_closed_form_sentence() {
        seed("x = 1\n");
        // `Integrate` is a pattern table by design; anything outside it is a
        // refusal by name, never a guess.
        let v = line("Integrate(exp(x^2), x)");
        assert_eq!(v["success"], false, "{v}");
        assert!(
            v["error"]
                .as_str()
                .unwrap()
                .contains("no closed form found for this input."),
            "{v}"
        );
    }

    #[test]
    fn the_unported_arms_refuse_by_name() {
        seed("x = 1\n");
        for input in ["CALL Routh(den : n, s)", "[1 2 3]", "1:2:10", "x + 1 = 5"] {
            let v = line(input);
            assert_eq!(v["success"], false, "{input}: {v}");
            assert!(
                v["error"].as_str().unwrap().contains("not supported"),
                "{input}: {v}"
            );
        }
    }

    #[test]
    fn an_unknown_name_is_an_error_not_a_zero() {
        seed("x = 1\n");
        let v = line("definitely_not_defined + 1");
        assert_eq!(v["success"], false, "{v}");
        assert!(v["error"].is_string(), "{v}");
    }

    #[test]
    fn numbers_render_like_the_java() {
        assert_eq!(number(600.0), "600");
        assert_eq!(number(0.5), "0.5");
        assert_eq!(number(1.0 / 3.0), "0.333333");
        assert_eq!(number(-2.5), "-2.5");
        assert_eq!(format(300.0, "K", Some(0.5)), "300 ± 0.5 [K]");
        assert_eq!(format(300.0, "-", None), "300");
        assert_eq!(format(300.0, "", None), "300");
    }

    #[test]
    fn line_shapes_are_classified_the_java_way() {
        assert_eq!(
            split_assignment("x = 1 + 2"),
            Some(("x".into(), "1 + 2".into()))
        );
        assert_eq!(split_assignment("x == 1"), None);
        assert_eq!(split_assignment("x <= 1"), None);
        assert_eq!(split_assignment("a + b = 1"), None);
        assert_eq!(
            split_cas_call("Apart(1/(s+1), s)"),
            Some(("Apart".into(), "1/(s+1), s".into()))
        );
        assert_eq!(split_cas_call("sqrt(4)"), None);
        assert!(starts_with_word("CALL foo(a : b)", "call"));
        assert!(!starts_with_word("calling", "call"));
        assert_eq!(split_top_level_commas("f(a, b), c"), vec!["f(a, b)", " c"]);
    }
}
