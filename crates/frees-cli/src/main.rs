//! Headless driver for the frees engine.
//!
//! Two jobs: solve a document from a file or stdin, and check one without
//! solving — the same two operations the parent engine exposes as
//! `POST /api/solve` and `POST /api/check`. Both print JSON on stdout so the
//! parity harness (`PLAN.md` §4) can diff them against the Java oracle's
//! `fixtures/golden/*.json`.
//!
//! ```text
//! frees-cli solve [FILE]     solve a document; JSON on stdout
//! frees-cli check [FILE]     structural check only; JSON on stdout
//! frees-cli version          engine version
//! ```
//!
//! With no `FILE` (or with `-`) the document is read from stdin, so the CLI
//! composes: `cat model.frees | frees-cli solve`.
//!
//! Exit status is `0` on success, `1` when the engine refuses the document
//! (still printing a JSON error object, so a caller never has to parse prose
//! off stderr), and `2` for a usage or I/O problem.

use std::io::{self, Read, Write};
use std::process::ExitCode;

use frees_core::engine::{self, CheckReport, Solution};
use frees_core::solver::SolverSettings;
use frees_core::{Diagnostic, FreesError};
use serde_json::{json, Map, Value};

const USAGE: &str = "\
frees-cli — headless frees engine

USAGE:
    frees-cli solve [FILE]    Solve a document and print the variables as JSON
    frees-cli check [FILE]    Check syntax and structural solvability only
    frees-cli version         Print the engine version

Reads stdin when FILE is omitted or is `-`.

OPTIONS (solve only):
    --request JSON_OR_FILE    Solve with full request JSON or file path
    --max-iterations N        Newton iteration limit        (default 100)
    --rel-tolerance F         Relative residual tolerance   (default 1e-9)
    --abs-tolerance F         Absolute residual tolerance   (default 1e-10)
    --jacobian-epsilon F      Finite-difference step        (default 1e-7)
    --max-step-halvings N     Line-search budget            (default 20)
";

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(message) => {
            eprintln!("frees-cli: {message}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<ExitCode, String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(command) = args.first().map(String::as_str) else {
        print!("{USAGE}");
        return Ok(ExitCode::from(2));
    };

    match command {
        "version" | "--version" | "-V" => {
            println!("frees-cli {}", frees_core::VERSION);
            Ok(ExitCode::SUCCESS)
        }
        "help" | "--help" | "-h" => {
            print!("{USAGE}");
            Ok(ExitCode::SUCCESS)
        }
        "solve" => {
            let (path, settings, request_json) = parse_solve_args(&args[1..])?;
            let source = read_source(path)?;
            if let Some(req) = request_json {
                let response_str = frees::solve(&source, &req);
                let val: Value = serde_json::from_str(&response_str)
                    .unwrap_or_else(|_| json!({ "error": response_str }));
                let ok = val.get("success").and_then(Value::as_bool).unwrap_or(false);
                Ok(emit((val, ok)))
            } else {
                Ok(emit(solve_json(&source, &settings)))
            }
        }
        "check" => {
            let path = parse_path_only(&args[1..], "check")?;
            let source = read_source(path)?;
            Ok(emit(check_json(&source)))
        }
        other => Err(format!("unknown command `{other}`. Try `frees-cli help`.")),
    }
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

/// `(payload, ok)` — the JSON to print and whether the engine accepted the
/// document.
type Outcome = (Value, bool);

fn solve_json(source: &str, settings: &SolverSettings) -> Outcome {
    match engine::solve(source, settings) {
        Ok(solution) => (solution_value(&solution), true),
        Err(failure) => {
            let mut value = error_value(&failure.error, source);
            // Structured failure diagnostics (the Java `SolverException`
            // payload): which block stalled, and the residuals/stats captured
            // there — same data the wasm boundary ships to the frontend.
            if let Some(object) = value.as_object_mut() {
                if let Some(index) = failure.failed_block_index {
                    object.insert("failed_block_index".into(), json!(index));
                }
                if let Some(partial) = &failure.partial {
                    object.insert(
                        "partial".into(),
                        json!({
                            "blocks": partial.blocks.len(),
                            "iterations": partial.stats.iterations,
                            "max_residual": number(partial.stats.max_residual),
                            "residuals": partial
                                .residuals
                                .iter()
                                .map(|r| json!({
                                    "equation": r.equation,
                                    "value": number(r.residual),
                                    "block": r.block,
                                }))
                                .collect::<Vec<_>>(),
                        }),
                    );
                }
            }
            (value, false)
        }
    }
}

fn check_json(source: &str) -> Outcome {
    match engine::check(source) {
        // An unsolvable document — structurally *or* syntactically — is a
        // *reported* outcome, not a crash (syntax failures carry `error_line`
        // and `errors`, mirroring the Java 400-with-body). The exit status
        // still says "not solvable" so a shell caller can gate on it the way
        // the frontend gates its Solve button.
        Ok(report) => {
            let ok = report.solvable;
            (check_value(&report), ok)
        }
        Err(err) => (error_value(&err, source), false),
    }
}

fn solution_value(solution: &Solution) -> Value {
    // Key by the DISPLAY spelling, as the Java `Result.variables()` does. This
    // matters most for the component layer: a port member is solved as the flat
    // `s1$h` but the engine — and every other surface, including the wasm
    // boundary — reports it as `s1.h`. Emitting the canonical key here made the
    // CLI disagree with both Java and the product, which is a trap when the CLI
    // is what you reach for to diagnose a suspected divergence.
    let mut variables = Map::new();
    for (name, value) in &solution.values {
        let display = solution.display_names.get(name).unwrap_or(name);
        variables.insert(display.clone(), number(*value));
    }
    let blocks: Vec<Value> = solution
        .blocks
        .iter()
        .enumerate()
        .map(|(index, block)| {
            json!({
                "index": index,
                "equations": block.equations,
                "equation_text": solution.block_equations.get(index),
                "variables": block.variables,
                "simultaneous": !block.is_scalar(),
            })
        })
        .collect();
    let residuals: Vec<Value> = solution
        .residuals
        .iter()
        .map(|r| {
            json!({
                "equation": r.equation,
                "residual": number(r.residual),
                "block": r.block,
            })
        })
        .collect();

    json!({
        "ok": true,
        "variables": Value::Object(variables),
        "display_names": solution.display_names,
        "blocks": blocks,
        "block_count": solution.blocks.len(),
        "residuals": residuals,
        "stats": {
            "iterations": solution.stats.iterations,
            "max_residual": number(solution.stats.max_residual),
            "elapsed_ms": solution.stats.elapsed_ms,
        },
        "iterations": solution.iterations,
        "diagnostics": diagnostics_value(&solution.diagnostics),
        // The golden dumper's `ode_tables` shape, key for key, so a suspected
        // transient divergence can be diffed straight against a fixture.
        "ode_tables": solution
            .ode_tables
            .iter()
            .map(|t| json!({
                "name": t.name,
                "method": t.method,
                "stopped": t.stopped,
                "end_time": number(t.end_time),
                "columns": t.columns,
                "rows": t.rows
                    .iter()
                    .map(|row| row.iter().map(|v| number(*v)).collect::<Vec<_>>())
                    .collect::<Vec<_>>(),
                "events": t.events
                    .iter()
                    .map(|e| json!({ "name": e.name, "time": number(e.time) }))
                    .collect::<Vec<_>>(),
            }))
            .collect::<Vec<_>>(),
    })
}

fn check_value(report: &CheckReport) -> Value {
    let errors: Vec<Value> = report
        .errors
        .iter()
        .map(|e| json!({ "line": e.line, "column": e.column, "message": e.message }))
        .collect();
    json!({
        "ok": true,
        "solvable": report.solvable,
        "equations": report.equation_count,
        "unknowns": report.unknown_count,
        "variables": report.variables,
        "display_names": report.display_names,
        "message": report.message,
        "error_line": report.error_line,
        "errors": errors,
        "diagnostics": diagnostics_value(&report.diagnostics),
    })
}

fn diagnostics_value(diagnostics: &[Diagnostic]) -> Value {
    Value::Array(
        diagnostics
            .iter()
            .map(|d| {
                json!({
                    "severity": d.severity.to_string(),
                    "message": d.message,
                    "source_text": d.source_text,
                })
            })
            .collect(),
    )
}

/// The error shape mirrors `fixtures/golden/*.json`'s `error` object, so the
/// parity harness can compare classifications directly.
fn error_value(err: &FreesError, source: &str) -> Value {
    let kind = match err {
        FreesError::Parse { .. } => "ParseException",
        FreesError::Solver { .. } => "SolverException",
        FreesError::Property { .. } => "PropertyEvaluationException",
        FreesError::Evaluation { .. } => "EvaluationException",
        FreesError::UnknownUnit { .. } => "UnknownUnitException",
    };
    let mut error = Map::new();
    error.insert("type".into(), Value::String(kind.into()));
    error.insert("message".into(), Value::String(err.to_string_message()));
    if let Some(span) = err.span() {
        let (line, column) = span.line_col(source);
        error.insert("line".into(), json!(line));
        error.insert("column".into(), json!(column));
    }
    json!({ "ok": false, "error": Value::Object(error) })
}

/// `f64` into JSON, with the non-finite values JSON cannot represent spelled
/// out as strings instead of silently becoming `null`.
fn number(value: f64) -> Value {
    match serde_json::Number::from_f64(value) {
        Some(n) => Value::Number(n),
        None => Value::String(value.to_string()),
    }
}

fn emit((payload, ok): Outcome) -> ExitCode {
    let rendered = serde_json::to_string_pretty(&payload)
        .unwrap_or_else(|e| format!(r#"{{"ok":false,"error":{{"message":"{e}"}}}}"#));
    let mut stdout = io::stdout().lock();
    // A broken pipe (`frees-cli solve x.frees | head`) is not worth a nonzero
    // status, so the write result is deliberately dropped.
    let _ = writeln!(stdout, "{rendered}");
    let _ = stdout.flush();
    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

// ---------------------------------------------------------------------------
// Argument handling
// ---------------------------------------------------------------------------

fn parse_solve_args(
    args: &[String],
) -> Result<(Option<&str>, SolverSettings, Option<String>), String> {
    let mut settings = SolverSettings::default();
    let mut request_json: Option<String> = None;
    let mut path: Option<&str> = None;
    let mut rest = args.iter();

    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--request" | "-r" => {
                let val = take_value(&mut rest, arg)?;
                let content =
                    if val.trim_start().starts_with('{') || val.trim_start().starts_with('[') {
                        val.to_string()
                    } else if std::path::Path::new(val).exists() {
                        std::fs::read_to_string(val)
                            .map_err(|e| format!("cannot read request file `{val}`: {e}"))?
                    } else {
                        val.to_string()
                    };
                request_json = Some(content);
            }
            "--max-iterations" => {
                settings.max_iterations = take_value(&mut rest, arg)?
                    .parse()
                    .map_err(|_| format!("{arg} expects a positive integer"))?;
            }
            "--rel-tolerance" => settings.rel_tolerance = take_number(&mut rest, arg)?,
            "--abs-tolerance" => settings.abs_tolerance = take_number(&mut rest, arg)?,
            "--jacobian-epsilon" => settings.jacobian_epsilon = take_number(&mut rest, arg)?,
            "--max-step-halvings" => {
                settings.max_step_halvings = take_value(&mut rest, arg)?
                    .parse()
                    .map_err(|_| format!("{arg} expects a non-negative integer"))?;
            }
            flag if flag.starts_with("--") => {
                return Err(format!("unknown option `{flag}` for `solve`"))
            }
            file => {
                if path.replace(file).is_some() {
                    return Err("`solve` takes at most one FILE".into());
                }
            }
        }
    }
    Ok((path, settings, request_json))
}

fn parse_path_only<'a>(args: &'a [String], command: &str) -> Result<Option<&'a str>, String> {
    let mut path = None;
    for arg in args {
        if arg.starts_with("--") {
            return Err(format!("unknown option `{arg}` for `{command}`"));
        }
        if path.replace(arg.as_str()).is_some() {
            return Err(format!("`{command}` takes at most one FILE"));
        }
    }
    Ok(path)
}

fn take_value<'a>(
    rest: &mut impl Iterator<Item = &'a String>,
    flag: &str,
) -> Result<&'a str, String> {
    rest.next()
        .map(String::as_str)
        .ok_or_else(|| format!("{flag} expects a value"))
}

fn take_number<'a>(rest: &mut impl Iterator<Item = &'a String>, flag: &str) -> Result<f64, String> {
    take_value(rest, flag)?
        .parse()
        .map_err(|_| format!("{flag} expects a number"))
}

/// Read the document from `path`, or from stdin when it is absent or `-`.
fn read_source(path: Option<&str>) -> Result<String, String> {
    match path {
        None | Some("-") => {
            let mut buffer = String::new();
            io::stdin()
                .read_to_string(&mut buffer)
                .map_err(|e| format!("cannot read stdin: {e}"))?;
            Ok(buffer)
        }
        Some(file) => {
            std::fs::read_to_string(file).map_err(|e| format!("cannot read `{file}`: {e}"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solve_out(source: &str) -> Value {
        solve_json(source, &SolverSettings::default()).0
    }

    #[test]
    fn solve_emits_variables_blocks_and_iterations() {
        let (payload, ok) = solve_json("a = 2\nb = a * 3\n", &SolverSettings::default());
        assert!(ok);
        assert_eq!(payload["ok"], json!(true));
        assert_eq!(payload["variables"]["a"], json!(2.0));
        assert_eq!(payload["variables"]["b"], json!(6.0));
        assert_eq!(payload["block_count"], json!(2));
        assert_eq!(payload["blocks"][0]["variables"], json!(["a"]));
        assert_eq!(payload["blocks"][0]["simultaneous"], json!(false));
        assert!(payload["iterations"].as_u64().unwrap() >= 2);
    }

    #[test]
    fn a_simultaneous_block_is_flagged() {
        let payload = solve_out("u + v = 10\nu - v = 2\n");
        assert_eq!(payload["blocks"][0]["simultaneous"], json!(true));
        assert_eq!(payload["blocks"][0]["variables"], json!(["u", "v"]));
    }

    #[test]
    fn a_solver_failure_is_json_not_a_panic() {
        let (payload, ok) = solve_json("z = 1\nz = 2\n", &SolverSettings::default());
        assert!(!ok);
        assert_eq!(payload["ok"], json!(false));
        assert_eq!(payload["error"]["type"], json!("SolverException"));
        assert!(payload["error"]["message"]
            .as_str()
            .unwrap()
            .contains("overspecified"));
        // A structural failure has no single line to point at.
        assert!(payload["error"].get("line").is_none());
    }

    #[test]
    fn a_parse_failure_carries_a_line_and_column() {
        let (payload, ok) = solve_json("a = 1\nb = = 2\n", &SolverSettings::default());
        assert!(!ok);
        assert_eq!(payload["error"]["type"], json!("ParseException"));
        assert_eq!(payload["error"]["line"], json!(2));
        assert!(payload["error"]["column"].as_u64().unwrap() >= 1);
    }

    #[test]
    fn an_unsupported_block_is_reported_as_a_parse_error() {
        // `DYNAMIC` is still an unported block type, and the parser refuses it
        // by name rather than skipping it. (`PLOT` used to stand here; it
        // parses into `Document::blocks` since Phase 8.)
        let (payload, ok) = solve_json(
            "DYNAMIC d(method = ode45)\n  der = 1\nEND\n",
            &SolverSettings::default(),
        );
        assert!(!ok);
        assert_eq!(payload["error"]["type"], json!("ParseException"));
        assert!(payload["error"]["message"]
            .as_str()
            .unwrap()
            .contains("DYNAMIC"));
    }

    #[test]
    fn a_component_definition_with_no_instance_has_nothing_to_solve() {
        // Phase 6 removed the capability gate that used to refuse this as a
        // ParseException. A `COMPONENT` template that is never instantiated
        // contributes no equations, so the document is empty and the *solver*
        // says so. Verified against the oracle, message included: Java answers
        // `SolverException: "No equations to solve."` for this exact source.
        let (payload, ok) = solve_json(
            "COMPONENT p(a)\n  a.T = 1\nEND\n",
            &SolverSettings::default(),
        );
        assert!(!ok);
        assert_eq!(payload["error"]["type"], json!("SolverException"));
        assert_eq!(payload["error"]["message"], json!("No equations to solve."));
    }

    #[test]
    fn a_component_network_solves_end_to_end() {
        // The other half of removing that gate: a document that *does*
        // instantiate solves, and its port members come back under the DISPLAY
        // spelling — `n2.v`, not the flat `n2$v` the solver works in. Verified
        // against the Java oracle, which reports exactly these four keys.
        let (payload, ok) = solve_json(
            "Resistor R1(n1, n2, R = 10)\nGround G1(n2)\nn1.V = 12\n",
            &SolverSettings::default(),
        );
        assert!(ok, "{payload}");
        assert_eq!(payload["variables"]["n2.v"], json!(0.0));
        assert_eq!(payload["variables"]["n1.i"], json!(1.2));
    }

    #[test]
    fn check_reports_counts_and_solvability() {
        let (payload, ok) = check_json("x^2 + y^3 = 77\nx/y = 1.23456\n");
        assert!(ok);
        assert_eq!(payload["solvable"], json!(true));
        assert_eq!(payload["equations"], json!(2));
        assert_eq!(payload["unknowns"], json!(2));
        assert_eq!(payload["variables"], json!(["x", "y"]));
    }

    #[test]
    fn check_reports_an_unsolvable_system_as_data_but_fails_the_exit_status() {
        let (payload, ok) = check_json("m + n = 5\n");
        assert!(!ok, "an unsolvable check must not exit 0");
        assert_eq!(payload["ok"], json!(true), "it still parsed");
        assert_eq!(payload["solvable"], json!(false));
        assert!(payload["message"]
            .as_str()
            .unwrap()
            .contains("underspecified"));
    }

    #[test]
    fn solve_emits_display_names_residuals_and_stats() {
        let payload = solve_out("Tin = 300\nT_out = TIN * 2\n");
        assert_eq!(payload["display_names"]["tin"], json!("Tin"));
        assert_eq!(payload["display_names"]["t_out"], json!("T_out"));

        let residuals = payload["residuals"].as_array().unwrap();
        assert_eq!(residuals.len(), 2);
        assert_eq!(residuals[0]["equation"], json!("Tin = 300"));
        assert_eq!(residuals[0]["block"], json!(0));

        assert_eq!(payload["stats"]["iterations"], payload["iterations"]);
        assert_eq!(payload["stats"]["elapsed_ms"], json!(null));
        assert!(payload["stats"]["max_residual"].as_f64().unwrap() < 1e-7);

        assert_eq!(payload["blocks"][0]["equation_text"], json!(["Tin = 300"]));
    }

    #[test]
    fn check_reports_a_syntax_error_as_a_positioned_report() {
        let (payload, ok) = check_json("a = 1\nb = = 2\n");
        assert!(!ok, "a syntax failure must not exit 0");
        assert_eq!(payload["ok"], json!(true), "it is still a report");
        assert_eq!(payload["solvable"], json!(false));
        assert_eq!(payload["error_line"], json!(2));
        assert_eq!(payload["errors"][0]["line"], json!(2));
        assert!(payload["message"]
            .as_str()
            .unwrap()
            .starts_with("Syntax error: "));
    }

    #[test]
    fn diagnostics_are_carried_into_the_payload() {
        let payload = solve_out("P = 140 [zorp]\n");
        let diagnostics = payload["diagnostics"].as_array().unwrap();
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0]["severity"], json!("warning"));
        assert!(diagnostics[0]["message"].as_str().unwrap().contains("zorp"));
        assert_eq!(diagnostics[0]["source_text"], json!("P = 140 [zorp]"));
    }

    #[test]
    fn units_are_si_in_the_output() {
        // Keyed by the display spelling `P`, matching the Java oracle's
        // `{"P": 140000.0}` — not the lowercase canonical name.
        let payload = solve_out("P = 140 [kPa]\n");
        assert_eq!(payload["variables"]["P"], json!(140000.0));
    }

    #[test]
    fn non_finite_values_are_strings_not_null() {
        assert_eq!(number(1.5), json!(1.5));
        assert_eq!(number(f64::INFINITY), json!("inf"));
        assert_eq!(number(f64::NAN), json!("NaN"));
    }

    #[test]
    fn solve_flags_are_parsed() {
        let args: Vec<String> = [
            "--max-iterations",
            "7",
            "--rel-tolerance",
            "1e-6",
            "m.frees",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        let (path, settings, request_raw) = parse_solve_args(&args).unwrap();
        assert_eq!(path, Some("m.frees"));
        assert_eq!(settings.max_iterations, 7);
        assert_eq!(settings.rel_tolerance, 1e-6);
        assert_eq!(request_raw, None);
    }

    #[test]
    fn request_flag_is_parsed() {
        let args: Vec<String> = ["--request", "{\"findAllSolutions\": true}", "m.frees"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let (path, _settings, request_raw) = parse_solve_args(&args).unwrap();
        assert_eq!(path, Some("m.frees"));
        assert_eq!(
            request_raw,
            Some("{\"findAllSolutions\": true}".to_string())
        );
    }

    #[test]
    fn bad_flags_are_usage_errors() {
        let one = |s: &str| vec![s.to_string()];
        assert!(parse_solve_args(&one("--nope")).is_err());
        assert!(parse_solve_args(&one("--max-iterations")).is_err());
        assert!(
            parse_solve_args(&["--rel-tolerance".to_string(), "not-a-number".to_string()]).is_err()
        );
        assert!(parse_solve_args(&["a".to_string(), "b".to_string()]).is_err());
        assert!(parse_path_only(&one("--nope"), "check").is_err());
    }

    #[test]
    fn stdin_is_the_default_source() {
        assert!(matches!(parse_path_only(&[], "check"), Ok(None)));
        let dash = vec!["-".to_string()];
        assert_eq!(parse_path_only(&dash, "check").unwrap(), Some("-"));
    }
}
