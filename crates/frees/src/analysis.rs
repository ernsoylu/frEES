//! `POST /api/solve/table` — the Tables workbook's GUI Solve, in-browser.
//!
//! Port of `SolveController.computeSolveTable` and its helpers: the sweep
//! driver itself (`run_sweep`, `build_columns`, `columns_converged`,
//! `mentions_parametric_accessor`) was ported to
//! `frees_core::analysis::parametric` in Phase 8 and had **zero call sites**
//! until this module — ledger item 23's "with no boundary, there is no place
//! for the Java controllers' input validation to live" is closed here for the
//! table surface, with the Java caps transcribed and the wasm-native ones
//! marked as such.
//!
//! Same contract as every export in this crate: JSON string in, JSON string
//! out, never a JS exception. A cap breach or a blank document is *data* — the
//! envelope carries a top-level `"error"` beside empty `results`, which is the
//! same string the Java's 400/422 body would carry.

use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::{json, Value};
use wasm_bindgen::prelude::*;

use frees_core::analysis::parametric::{run_sweep, RowJob, RowOutcome};
use frees_core::components::cyclepath;
use frees_core::parser::blocks::ParametricTable;

use crate::{
    explicit_units_of, function_table_defs_of, now_ms, overrides_of, settings_of, unit_system_of,
    variable_entries, variable_rows, FunctionTableDto, SolveRequest, StopCriteriaDto,
    VariableInfoDto,
};

// ---------------------------------------------------------------------------
// Caps. The first two are the Java controller's, transcribed with their
// values and messages; the third is wasm-native (no Java analogue — the Java
// has a horizontal compute tier behind a queue, this build has one worker).
// ---------------------------------------------------------------------------

/// `frees.solver.max-table-rows` (default) — `SolveController`.
const MAX_TABLE_ROWS: usize = 5_000;

/// `frees.solver.max-table-seconds` (default) — the *cooperative* budget for
/// the whole request: checked between rows and per row inside each accessor
/// pass, never mid-solve, exactly like the Java `TableDeadline`.
const MAX_TABLE_SECONDS: f64 = 120.0;

/// wasm-native: a bound on the declared column count, so the dense grid this
/// boundary builds from the sparse row maps cannot be inflated independently
/// of the row cap.
const MAX_TABLE_COLUMNS: usize = 256;

/// `SolveController.tooManyRowsMessage`, verbatim.
fn too_many_rows_message(rows: usize) -> String {
    format!(
        "The parametric table has too many rows ({rows}; limit {MAX_TABLE_ROWS}). \
         Reduce the run count."
    )
}

/// `SolveController.deadlineExceededMessage`, verbatim.
fn deadline_message() -> String {
    format!(
        "The parametric run exceeded its {}-second budget and was stopped. Reduce \
         the number of runs, or tighten the stop criteria so each run converges \
         faster.",
        MAX_TABLE_SECONDS as u64
    )
}

// ---------------------------------------------------------------------------
// Request DTOs — the Java `SolveTableRequest` minus `text` (which rides as
// the `source` argument, like `solve`'s). `functionTables` is honoured since
// Wave H (decision D10): converted once per request and threaded into every
// per-row solve, the Java `TableRowContext.functionDefs`
// (`SolveController.computeSolveTable`, and line 531's chunked re-dispatch).
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct SolveTableRequest {
    stop_criteria: Option<StopCriteriaDto>,
    variable_info: Vec<VariableInfoDto>,
    display_unit_system: Option<String>,
    table: Option<TableDto>,
    function_tables: Option<Vec<FunctionTableDto>>,
}

/// `SolveController.TableDto`: `variables` (the declared column order) and
/// `rows` as sparse maps — an absent or `null` value means the cell is blank,
/// i.e. an output the solve fills in.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct TableDto {
    variables: Vec<String>,
    rows: Vec<BTreeMap<String, Option<f64>>>,
}

/// What the side-channel keeps per run beside `RowOutcome` (which carries
/// only the SI values the accessor columns need): the display-unit values and
/// DTOs the response is built from, and the per-row solve stats. The accessor
/// loop re-runs every row on every pass, so slot `run - 1` is simply
/// overwritten and the final pass wins.
struct RowSide {
    success: bool,
    values: BTreeMap<String, f64>,
    error: Option<String>,
    variables: Vec<Value>,
    iterations: usize,
    max_residual: f64,
    equations: usize,
    unknowns: usize,
}

/// Solve a Tables-workbook parametric sweep. `request_json` is the
/// `SolveTableRequest` body; returns a `SolveTableResponse` JSON string.
#[wasm_bindgen]
pub fn solve_table(source: &str, request_json: &str) -> String {
    match solve_table_inner(source, request_json) {
        Ok(value) => value.to_string(),
        Err(message) => json!({
            "results": [],
            "stats": Value::Null,
            "variables": [],
            "error": message,
        })
        .to_string(),
    }
}

fn solve_table_inner(source: &str, request_json: &str) -> Result<Value, String> {
    frees_core::props::tables::install_builtin_once();

    let request: SolveTableRequest = if request_json.trim().is_empty() {
        SolveTableRequest::default()
    } else {
        serde_json::from_str(request_json).map_err(|e| format!("Invalid request: {e}"))?
    };

    // The Java 400s: blank text, missing table. Same strings as data.
    if source.trim().is_empty() {
        return Err("The document is empty.".to_string());
    }
    let Some(table_dto) = request.table else {
        return Err("The request carries no table.".to_string());
    };
    if table_dto.rows.len() > MAX_TABLE_ROWS {
        return Err(too_many_rows_message(table_dto.rows.len()));
    }
    if table_dto.variables.len() > MAX_TABLE_COLUMNS {
        return Err(format!(
            "The parametric table has too many columns ({}; limit {MAX_TABLE_COLUMNS}). \
             Reduce the declared variables.",
            table_dto.variables.len()
        ));
    }

    // The Java parse gate before anything is enqueued: a syntax error answers
    // once, not once per row.
    if let Err(failure) = frees_core::parse_document(source) {
        return Err(format!("Syntax error: {}", failure.to_string_message()));
    }

    // The dense grid `ParametricTable` wants, from the sparse row maps: each
    // row aligned to `variables`, absent/null ⇒ `None` (an output cell). Key
    // lookup is exact, as the Java's `row.get(var)` is; the workbook keys its
    // rows by the declared names, so nothing else arrives in practice.
    let table = ParametricTable {
        name: String::new(),
        vars: table_dto.variables.clone(),
        rows: table_dto
            .rows
            .iter()
            .map(|row| {
                table_dto
                    .variables
                    .iter()
                    .map(|var| row.get(var).copied().flatten().filter(|v| v.is_finite()))
                    .collect()
            })
            .collect(),
    };

    // The `solve` helpers, reused verbatim through a facade request. The
    // request's Function Tables are converted once, outside it — the Java
    // `TableRowContext.functionDefs`, built once per request too.
    let facade = SolveRequest {
        variable_info: request.variable_info,
        stop_criteria: request.stop_criteria,
        display_unit_system: request.display_unit_system.clone(),
        fill_missing: None,
        function_tables: None,
    };
    let extra_tables = function_table_defs_of(&request.function_tables);
    let settings = settings_of(&facade);
    let overrides = overrides_of(&facade);
    let system = unit_system_of(&facade);
    let explicit_units = explicit_units_of(&facade);

    // Wave T5: the Tables workbook drives the same Solve button, so it gets the
    // same bar. The guard clears the sink on every exit path.
    let _progress = crate::install_progress();

    let started = now_ms();
    let run_count = table.run_count();
    let mut sides: Vec<Option<RowSide>> = Vec::new();
    sides.resize_with(run_count, || None);
    let mut deadline_hit = false;

    let sweep = run_sweep(&table, source, |job: RowJob<'_>| {
        // The cooperative deadline, at the Java's two check sites collapsed
        // into one: entry to every row solve, on every pass.
        if deadline_hit || (now_ms() - started) / 1000.0 > MAX_TABLE_SECONDS {
            deadline_hit = true;
            return RowOutcome::failed(deadline_message());
        }
        // Wave T5: a sweep's honest progress is rows done, and `RowJob` already
        // carries the pair — so the bar comes from the boundary here rather
        // than from a second reporting site inside `run_sweep`. The row's own
        // blocks subdivide this slice (`crate::progress`); an accessor sweep
        // makes several passes over the same rows, so the bar restarts per
        // pass rather than pretending pass 2 of 3 is two thirds done.
        let _row = frees_core::progress::enter(
            (job.run - 1) as f64 / job.total_runs.max(1) as f64,
            1.0 / job.total_runs.max(1) as f64,
        );
        let slot = &mut sides[job.run - 1];
        match frees_core::solve_with_parametric_tables(
            &job.source,
            &settings,
            &overrides,
            job.accessors,
            &extra_tables,
        ) {
            Ok(mut solution) => {
                // The Java table row fills missing properties unconditionally,
                // scoped to the table's own columns — not gated on the
                // Preferences switch like `/api/solve`.
                let added = cyclepath::resolve_missing_properties(
                    &mut solution.values,
                    &mut solution.display_names,
                    &job.source,
                    Some(&table.vars),
                    &[],
                );
                for name in &added {
                    if let Some(unit) = cyclepath::si_unit_for_state_variable(name) {
                        solution
                            .inferred_units
                            .entry(name.clone())
                            .or_insert_with(|| unit.to_string());
                    }
                }
                let (rows, uncertainties) = variable_rows(&solution, system, &explicit_units);
                let values: BTreeMap<String, f64> = rows
                    .iter()
                    .map(|row| (row.name.clone(), row.value))
                    .collect();
                *slot = Some(RowSide {
                    success: true,
                    values,
                    error: None,
                    variables: variable_entries(&rows, &uncertainties),
                    iterations: solution.stats.iterations,
                    max_residual: solution.stats.max_residual,
                    equations: solution.residuals.len(),
                    unknowns: solution.values.len(),
                });
                RowOutcome::solved(solution.values)
            }
            Err(failure) => {
                let message = failure.to_string_message();
                *slot = Some(RowSide {
                    success: false,
                    values: BTreeMap::new(),
                    error: Some(message.clone()),
                    variables: Vec::new(),
                    iterations: 0,
                    max_residual: 0.0,
                    equations: 0,
                    unknowns: 0,
                });
                RowOutcome::failed(message)
            }
        }
    });

    if deadline_hit {
        return Err(deadline_message());
    }

    // `SolveTableResponse`: per-row results in display units, the aggregated
    // stats (iterations summed, residual maxed, equations/unknowns assigned
    // from each successful row — last one wins, as the Java loop does), and
    // the last successful row's variable DTOs.
    let mut iterations = 0usize;
    let mut max_residual = 0.0f64;
    let mut equations = 0usize;
    let mut unknowns = 0usize;
    let mut solved = 0usize;
    let mut last_variables: Vec<Value> = Vec::new();
    let results: Vec<Value> = sides
        .iter()
        .map(|side| match side {
            Some(side) => {
                iterations += side.iterations;
                if side.success {
                    solved += 1;
                    max_residual = max_residual.max(side.max_residual);
                    equations = side.equations;
                    unknowns = side.unknowns;
                    if !side.variables.is_empty() {
                        last_variables = side.variables.clone();
                    }
                }
                json!({
                    "success": side.success,
                    "values": side.values,
                    "error": side.error,
                })
            }
            None => json!({
                "success": false,
                "values": {},
                "error": "The row was not solved.",
            }),
        })
        .collect();
    let _ = &sweep; // the sweep's columns fed the accessors; results are the answer

    Ok(json!({
        "results": results,
        "stats": {
            "runs": run_count,
            "solved": solved,
            "failed": run_count - solved,
            "equations": equations,
            "unknowns": unknowns,
            "iterations": iterations,
            "elapsedMillis": (now_ms() - started).round().max(0.0) as u64,
            "maxResidual": if max_residual.is_finite() { max_residual } else { 0.0 },
        },
        "variables": last_variables,
    }))
}

// ---------------------------------------------------------------------------
// POST /api/solve/montecarlo — Wave B2.
// Port of `SolveController.computeMonteCarlo` + its post-processing block.
// ---------------------------------------------------------------------------

/// `frees.solver.max-mc-samples` (default) — `SolveController`.
const MAX_MC_SAMPLES: usize = 1_000;

/// The Java's `request.samples()` fallback.
const MC_DEFAULT_SAMPLES: usize = 200;

/// `frees.solver.max-mc-seconds` (default). Unlike the table budget, running
/// out is **not** an error: the run truncates and answers with
/// `truncated: true`, exactly as the Java's deadline does.
const MAX_MC_SECONDS: f64 = 120.0;

/// The Java's `request.seed()` fallback (and the old modal's default field).
const MC_DEFAULT_SEED: i64 = 42;

/// `SolveController.badSampleCountMessage`, verbatim.
fn bad_sample_count_message(n: i64) -> String {
    format!(
        "Monte Carlo sample count must be between 2 and {MAX_MC_SAMPLES} (got {n}). \
         Adjust the sample count."
    )
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct MonteCarloRequest {
    stop_criteria: Option<StopCriteriaDto>,
    variable_info: Vec<VariableInfoDto>,
    display_unit_system: Option<String>,
    samples: Option<i64>,
    seed: Option<i64>,
    /// Honoured since Wave H (D10): the Java `computeMonteCarlo` converts
    /// them once and `MonteCarlo.run` threads them into the base solve and
    /// every per-sample solve.
    function_tables: Option<Vec<FunctionTableDto>>,
}

/// Run a Monte Carlo uncertainty propagation. `request_json` is the
/// `MonteCarloRequest` body; returns a `MonteCarloResponse` JSON string —
/// on a refused request the envelope carries a top-level `"error"` (the
/// api.ts side turns that into the rejection the modal's catch shows).
#[wasm_bindgen]
pub fn monte_carlo(source: &str, request_json: &str) -> String {
    match monte_carlo_inner(source, request_json) {
        Ok(value) => value.to_string(),
        Err(message) => json!({
            "stats": [],
            "samples": [],
            "sources": [],
            "requestedSamples": 0,
            "failedSamples": 0,
            "truncated": false,
            "error": message,
        })
        .to_string(),
    }
}

fn monte_carlo_inner(source: &str, request_json: &str) -> Result<Value, String> {
    frees_core::props::tables::install_builtin_once();

    let request: MonteCarloRequest = if request_json.trim().is_empty() {
        MonteCarloRequest::default()
    } else {
        serde_json::from_str(request_json).map_err(|e| format!("Invalid request: {e}"))?
    };

    if source.trim().is_empty() {
        return Err("The document is empty.".to_string());
    }
    let n = request.samples.unwrap_or(MC_DEFAULT_SAMPLES as i64);
    if n < 2 || n > MAX_MC_SAMPLES as i64 {
        return Err(bad_sample_count_message(n));
    }
    let seed = request.seed.unwrap_or(MC_DEFAULT_SEED);

    // The Java parse gate: a syntax error answers once, before any sampling.
    if let Err(failure) = frees_core::parse_document(source) {
        return Err(format!("Syntax error: {}", failure.to_string_message()));
    }

    let facade = SolveRequest {
        variable_info: request.variable_info,
        stop_criteria: request.stop_criteria,
        display_unit_system: request.display_unit_system.clone(),
        fill_missing: None,
        function_tables: None,
    };
    let extra_tables = function_table_defs_of(&request.function_tables);
    let settings = settings_of(&facade);
    let overrides = overrides_of(&facade);
    let system = unit_system_of(&facade);
    let explicit_units = explicit_units_of(&facade);

    // `VariableInfoDto.toSpec`, exactly: guess/lower/upper convert to SI with
    // the full factor + offset, the uncertainty (an interval width) by the
    // factor alone, and an unknown unit falls back to factor 1 / offset 0.
    let mut specs: BTreeMap<String, frees_core::analysis::uncertainty::UncertaintySpec> =
        BTreeMap::new();
    for dto in &facade.variable_info {
        let name = dto.name.trim().to_ascii_lowercase();
        if name.is_empty() {
            continue;
        }
        let (factor, offset) = match dto.units.as_deref().map(str::trim) {
            Some(unit) if !unit.is_empty() && unit != "-" => {
                match frees_core::units::registry::UnitRegistry::parse_with_offset(unit) {
                    Ok(recorded) => (recorded.factor, recorded.offset),
                    Err(_) => (1.0, 0.0),
                }
            }
            _ => (1.0, 0.0),
        };
        let mut spec = frees_core::analysis::uncertainty::UncertaintySpec::default();
        if let Some(lower) = dto.lower {
            spec.lower = lower * factor + offset;
        }
        if let Some(upper) = dto.upper {
            spec.upper = upper * factor + offset;
        }
        if let Some(guess) = dto.guess {
            spec.guess = guess * factor + offset;
        } else {
            spec.guess = spec.guess.clamp(spec.lower, spec.upper);
        }
        if let Some(uncertainty) = dto.uncertainty {
            spec.uncertainty = uncertainty * factor;
        }
        specs.insert(name, spec);
    }

    // The base solve the Java reads `base.uncertainties()` from —
    // `montecarlo::run` keeps only the base *values*, so the first-order
    // sigmas (and the display-name/unit maps every conversion below needs)
    // come from this boundary-side solve. One extra solve; the Java pays the
    // same shape differently (its `run` returns the whole base result).
    let base = frees_core::solve_with_tables(source, &settings, &overrides, &extra_tables)
        .map_err(|failure| failure.to_string_message())?;

    let started = now_ms();
    let outcome = frees_core::analysis::montecarlo::run_with_tables(
        source,
        &settings,
        &specs,
        &base.uncertainties,
        n as usize,
        seed,
        || (now_ms() - started) / 1000.0 > MAX_MC_SECONDS,
        &extra_tables,
    )
    .map_err(|e| e.to_string_message())?;

    // The controller's post-processing block: internal temporaries filtered,
    // names remapped to display spellings, values to display units — sigmas
    // as interval widths (factor only), percentiles as plain values — and
    // stats sorted by |sigma| descending.
    let display_value = |key: &str, si: f64| -> f64 {
        let unit = base
            .inferred_units
            .get(key)
            .map(String::as_str)
            .unwrap_or("");
        crate::to_display(key, si, None, unit, system, &explicit_units).0
    };
    let display_width = |key: &str, about: f64, width: f64| -> f64 {
        let unit = base
            .inferred_units
            .get(key)
            .map(String::as_str)
            .unwrap_or("");
        crate::to_display(key, about, Some(width), unit, system, &explicit_units)
            .2
            .unwrap_or(width)
    };

    let mut stats: Vec<&frees_core::analysis::montecarlo::VariableStats> = outcome
        .stats
        .iter()
        .filter(|s| !frees_core::parser::expand::is_internal_temp(&s.variable))
        .collect();
    stats.sort_by(|a, b| b.sigma.abs().total_cmp(&a.sigma.abs()));
    let stats_json: Vec<Value> = stats
        .iter()
        .map(|s| {
            let display = crate::display_of(&base.display_names, &s.variable).clone();
            let key = display.to_ascii_lowercase();
            json!({
                "variable": display,
                "mean": display_value(&key, s.mean),
                "sigma": display_width(&key, s.mean, s.sigma),
                "p5": display_value(&key, s.p5),
                "p50": display_value(&key, s.p50),
                "p95": display_value(&key, s.p95),
                "firstOrderSigma": display_width(&key, s.mean, s.first_order_sigma),
            })
        })
        .collect();

    let samples_json: Vec<Value> = outcome
        .samples
        .iter()
        .map(|sample| {
            let values: BTreeMap<String, f64> = sample
                .values
                .iter()
                .filter(|(name, _)| !frees_core::parser::expand::is_internal_temp(name))
                .map(|(name, si)| {
                    let display = crate::display_of(&base.display_names, name).clone();
                    let key = display.to_ascii_lowercase();
                    (display, display_value(&key, *si))
                })
                .collect();
            json!({
                "success": sample.success,
                "values": values,
                "error": sample.error,
            })
        })
        .collect();

    let sources: Vec<String> = outcome
        .sources
        .iter()
        .map(|name| crate::display_of(&base.display_names, name).clone())
        .collect();

    Ok(json!({
        "stats": stats_json,
        "samples": samples_json,
        "sources": sources,
        "requestedSamples": n,
        "failedSamples": outcome.failed_samples,
        "truncated": outcome.truncated,
    }))
}

// ---------------------------------------------------------------------------
// Wave B3: the four OptimizeController endpoints. All four share the Java
// class's helpers, transcribed once here:
// ---------------------------------------------------------------------------

/// `SolverApiSupport.NO_EQUATIONS_MESSAGE`, verbatim.
const NO_EQUATIONS_MESSAGE: &str = "No equations entered.";

/// `SolverApiSupport.SYNTAX_ERROR_PREFIX` + first line, the idiom all four
/// endpoints use for a parse failure.
fn syntax_error(failure: &frees_core::diag::FreesError) -> String {
    let message = failure.to_string_message();
    format!(
        "Syntax error: {}",
        message.lines().next().unwrap_or(&message)
    )
}

/// `OptimizeController.clampPositive`: null-or-nonpositive falls back, else
/// capped. Population and generations both use (40, 200).
fn clamp_positive(value: Option<i64>, fallback: usize, max: usize) -> usize {
    match value {
        Some(v) if v > 0 => (v as usize).min(max),
        _ => fallback,
    }
}

// ---------------------------------------------------------------------------
// POST /api/curve-fit — the cheapest of the four: a pure expression fitter
// with no engine dependency, no unit conversion, and a thin pass-through
// controller. `lowerBounds`/`upperBounds` are accepted and ignored exactly as
// the Java does (its Commons Math 3.x LM has no box constraints — the guard
// block is an empty comment), so the omission is behaviour-identical.
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct CurveFitRequest {
    model: String,
    y_variable: String,
    x_variable: String,
    parameters: Vec<String>,
    x_data: Vec<f64>,
    y_data: Vec<f64>,
    initial_guess: Option<Vec<f64>>,
}

/// Least-squares curve fit. Returns a `CurveFitResponse` JSON string.
#[wasm_bindgen]
pub fn curve_fit(request_json: &str) -> String {
    match curve_fit_inner(request_json) {
        Ok(value) => value.to_string(),
        Err(message) => json!({
            "success": false,
            "error": message,
            "fittedParameters": [],
            "parameterNames": [],
            "rSquared": 0.0,
            "rmse": 0.0,
            "iterations": 0,
            "residuals": [],
            "fittedValues": [],
        })
        .to_string(),
    }
}

fn curve_fit_inner(request_json: &str) -> Result<Value, String> {
    frees_core::props::tables::install_builtin_once();
    let request: CurveFitRequest =
        serde_json::from_str(request_json).map_err(|e| format!("Invalid request: {e}"))?;

    // `validateCurveFitRequest`, in the Java's order and words.
    if request.model.trim().is_empty() {
        return Err("Model equation is required.".to_string());
    }
    if request.x_variable.trim().is_empty() {
        return Err("Independent variable name is required.".to_string());
    }
    if request.y_variable.trim().is_empty() {
        return Err("Dependent variable name is required.".to_string());
    }
    if request.parameters.is_empty() {
        return Err("At least one parameter to fit is required.".to_string());
    }
    if request.x_data.is_empty() || request.y_data.is_empty() {
        return Err("Data points are required.".to_string());
    }
    if request.x_data.len() != request.y_data.len() {
        return Err(format!(
            "x and y data must have the same length (got {} and {}).",
            request.x_data.len(),
            request.y_data.len()
        ));
    }

    let result = frees_core::analysis::curvefit::fit(
        &request.model,
        &request.y_variable,
        &request.x_variable,
        &request.parameters,
        &request.x_data,
        &request.y_data,
        request.initial_guess.as_deref(),
    )
    .map_err(|e| match e {
        // Parse → the shared syntax prefix; everything else → the Java's
        // catch-all wrapper ("Curve fitting failed: " + message).
        frees_core::diag::FreesError::Parse { .. } => {
            let message = e.to_string_message();
            format!(
                "Syntax error: {}",
                message.lines().next().unwrap_or(&message)
            )
        }
        other => format!("Curve fitting failed: {}", other.to_string_message()),
    })?;

    Ok(json!({
        "success": true,
        "error": Value::Null,
        "fittedParameters": result.fitted_parameters,
        "parameterNames": result.parameter_names,
        "rSquared": result.r_squared,
        "rmse": result.rmse,
        "iterations": result.iterations,
        "residuals": result.residuals,
        "fittedValues": result.fitted_values,
    }))
}

// ---------------------------------------------------------------------------
// POST /api/optimize — single/multi-decision minimisation with the library's
// own evaluation budgets; the controller adds no numeric clamps, only shape
// checks and display-unit conversion on the way out.
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct OptimizeRequest {
    stop_criteria: Option<StopCriteriaDto>,
    variable_info: Vec<VariableInfoDto>,
    display_unit_system: Option<String>,
    objective: String,
    // The legacy scalar triple beside the list form, exactly as the Java DTO.
    decision: Option<String>,
    lower: Option<f64>,
    upper: Option<f64>,
    maximize: Option<bool>,
    decisions: Vec<String>,
    lowers: Vec<f64>,
    uppers: Vec<f64>,
    method: Option<String>,
    constraints: Vec<String>,
}

/// Constrained/unconstrained optimisation. Returns an `OptimizeResponse`
/// JSON string.
#[wasm_bindgen]
pub fn optimize(source: &str, request_json: &str) -> String {
    match optimize_inner(source, request_json) {
        Ok(value) => value.to_string(),
        Err(message) => json!({
            "success": false,
            "error": message,
            "warning": Value::Null,
            "objective": Value::Null,
            "decision": Value::Null,
            "decisions": [],
            "evaluations": 0,
            "variables": [],
        })
        .to_string(),
    }
}

fn optimize_inner(source: &str, request_json: &str) -> Result<Value, String> {
    frees_core::props::tables::install_builtin_once();
    let request: OptimizeRequest = if request_json.trim().is_empty() {
        OptimizeRequest::default()
    } else {
        serde_json::from_str(request_json).map_err(|e| format!("Invalid request: {e}"))?
    };

    // `validateOptimizeRequest`, in the Java's order and words.
    if source.trim().is_empty() {
        return Err(NO_EQUATIONS_MESSAGE.to_string());
    }
    let (decisions, lowers, uppers) = if request.decisions.is_empty() {
        let Some(decision) = request.decision.clone() else {
            return Err("Independent variable name is required.".to_string());
        };
        let (Some(lower), Some(upper)) = (request.lower, request.upper) else {
            return Err("Both bounds of the independent variable are required.".to_string());
        };
        (vec![decision], vec![lower], vec![upper])
    } else {
        if request.lowers.len() != request.decisions.len()
            || request.uppers.len() != request.decisions.len()
        {
            return Err("Each independent variable requires lower and upper bounds.".to_string());
        }
        (
            request.decisions.clone(),
            request.lowers.clone(),
            request.uppers.clone(),
        )
    };
    if let Err(failure) = frees_core::parse_document(source) {
        return Err(syntax_error(&failure));
    }

    // No `function_tables`: the Java `OptimizeRequest` record carries none.
    let facade = SolveRequest {
        variable_info: request.variable_info,
        stop_criteria: request.stop_criteria,
        display_unit_system: request.display_unit_system.clone(),
        fill_missing: None,
        function_tables: None,
    };
    let settings = settings_of(&facade);
    let overrides = overrides_of(&facade);
    let system = unit_system_of(&facade);
    let explicit_units = explicit_units_of(&facade);

    let problem = frees_core::analysis::optimizer::Problem {
        text: source.to_string(),
        settings,
        overrides,
        objective: request.objective.clone(),
        decisions: decisions.clone(),
        lowers,
        uppers,
        method: Some(request.method.unwrap_or_else(|| "brent".to_string())),
        maximize: request.maximize == Some(true),
        constraints: request.constraints.clone(),
    };
    let result =
        frees_core::analysis::optimizer::optimize(&problem).map_err(|e| e.to_string_message())?;

    // `buildOptimizeResponse`: every DTO in display units. The solved system
    // rides on `result.solution`, so the objective/decision rows are looked
    // up out of the same `variable_rows` pass the plain solve uses.
    let (rows, uncertainties) = variable_rows(&result.solution, system, &explicit_units);
    let entries = variable_entries(&rows, &uncertainties);
    let dto_for = |name: &str| -> Value {
        let lower = name.to_ascii_lowercase();
        entries
            .iter()
            .find(|e| {
                e["name"]
                    .as_str()
                    .is_some_and(|n| n.to_ascii_lowercase() == lower)
            })
            .cloned()
            .unwrap_or(Value::Null)
    };
    let decision_dtos: Vec<Value> = decisions.iter().map(|d| dto_for(d)).collect();
    Ok(json!({
        "success": true,
        "error": Value::Null,
        "warning": result.warning,
        "objective": dto_for(&request.objective),
        "decision": decision_dtos.first().cloned().unwrap_or(Value::Null),
        "decisions": decision_dtos,
        "evaluations": result.evaluations,
        "variables": entries,
    }))
}

// ---------------------------------------------------------------------------
// POST /api/optimize/multi — NSGA-II. Raw SI numbers out (the Java sends no
// unit system); the [40, 200] clamps on population AND generations are the
// controller's (the library floors population at 8 and caps it at 200 itself
// — ledger 23 — but generations are unclamped there, so the boundary clamp
// is load-bearing).
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct MultiObjectiveRequest {
    stop_criteria: Option<StopCriteriaDto>,
    variable_info: Vec<VariableInfoDto>,
    objectives: Vec<String>,
    maximize: Vec<bool>,
    decisions: Vec<String>,
    lowers: Vec<f64>,
    uppers: Vec<f64>,
    population_size: Option<i64>,
    generations: Option<i64>,
    constraints: Vec<String>,
}

/// Multi-objective (Pareto) optimisation. Returns a `ParetoResponse` JSON
/// string.
#[wasm_bindgen]
pub fn optimize_multi(source: &str, request_json: &str) -> String {
    match optimize_multi_inner(source, request_json) {
        Ok(value) => value.to_string(),
        Err(message) => json!({
            "success": false,
            "error": message,
            "decisionNames": [],
            "objectiveNames": [],
            "front": [],
            "evaluations": 0,
        })
        .to_string(),
    }
}

fn optimize_multi_inner(source: &str, request_json: &str) -> Result<Value, String> {
    frees_core::props::tables::install_builtin_once();
    let request: MultiObjectiveRequest = if request_json.trim().is_empty() {
        MultiObjectiveRequest::default()
    } else {
        serde_json::from_str(request_json).map_err(|e| format!("Invalid request: {e}"))?
    };

    // `validateMultiObjectiveRequest`, in the Java's order and words.
    if source.trim().is_empty() {
        return Err(NO_EQUATIONS_MESSAGE.to_string());
    }
    if request.objectives.len() < 2 {
        return Err(
            "Multi-objective optimization needs at least two objective variables.".to_string(),
        );
    }
    if request.decisions.is_empty()
        || request.lowers.len() != request.decisions.len()
        || request.uppers.len() != request.decisions.len()
    {
        return Err("Each decision variable requires matching lower and upper bounds.".to_string());
    }
    if let Err(failure) = frees_core::parse_document(source) {
        return Err(syntax_error(&failure));
    }

    // No `function_tables`: the Java `MultiObjectiveRequest` record carries none.
    let facade = SolveRequest {
        variable_info: request.variable_info,
        stop_criteria: request.stop_criteria,
        display_unit_system: None,
        fill_missing: None,
        function_tables: None,
    };
    let settings = settings_of(&facade);
    let overrides = overrides_of(&facade);

    let mut maximize = request.maximize.clone();
    maximize.resize(request.objectives.len(), false);
    let problem = frees_core::analysis::pareto::Problem {
        text: source.to_string(),
        settings,
        overrides,
        objectives: request.objectives.clone(),
        maximize,
        decisions: request.decisions.clone(),
        lowers: request.lowers.clone(),
        uppers: request.uppers.clone(),
        population_size: clamp_positive(request.population_size, 40, 200),
        generations: clamp_positive(request.generations, 40, 200),
        seed: 42,
        constraints: request.constraints.clone(),
    };
    let result = frees_core::analysis::pareto::optimize_multi(&problem)
        .map_err(|e| e.to_string_message())?;

    let front: Vec<Value> = result
        .front
        .iter()
        .map(|p| json!({ "decisions": p.decisions, "objectives": p.objectives }))
        .collect();
    Ok(json!({
        "success": true,
        "error": Value::Null,
        // Echoed verbatim from the request, exactly as the Java does.
        "decisionNames": request.decisions,
        "objectiveNames": request.objectives,
        "front": front,
        "evaluations": result.evaluations,
    }))
}

// ---------------------------------------------------------------------------
// POST /api/measurements/parameter-fit — calibrate DYNAMIC-block parameters
// against a measured series. The only one of the four whose library entry
// takes a solve callback and an `expired` predicate: the callback adapts the
// engine's `OdeTableResult` into paramfit's `OdeTableView` (the `Option`
// cells exist for the view's own generality — a real table's cells are all
// `Some`, so a genuine NaN and an unfilled cell are indistinguishable there,
// which is fine: the fitter treats both as penalty). `functionTables` are
// honoured since Wave H (D10): converted once and threaded through the solve
// callback into every fit evaluation, the Java
// `ParameterFit.run(solver, …, SolveDtos.functionDefsOf(request.functionTables()), …)`
// (`OptimizeController.computeParameterFit`).
// ---------------------------------------------------------------------------

/// `frees.solver.max-fit-evaluations` (default) and the Java's `[10, …]`
/// floor / `150` fallback.
const MAX_FIT_EVALUATIONS: usize = 300;
const DEFAULT_FIT_EVALUATIONS: usize = 150;

/// `OptimizeController.MAX_FIT_SAMPLES`.
const MAX_FIT_SAMPLES: usize = 200_000;

/// `frees.solver.max-fit-seconds` (default) — the same 120 s budget the
/// table and Monte Carlo runs use.
const MAX_FIT_SECONDS: f64 = 120.0;

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct ParameterFitRequest {
    text: String,
    stop_criteria: Option<StopCriteriaDto>,
    variable_info: Vec<VariableInfoDto>,
    parameters: Vec<String>,
    initial: Vec<f64>,
    lower: Vec<f64>,
    upper: Vec<f64>,
    ode_block: String,
    column: String,
    measured_t: Vec<f64>,
    measured_v: Vec<f64>,
    max_evaluations: Option<i64>,
    function_tables: Option<Vec<FunctionTableDto>>,
}

/// Fit DYNAMIC-block parameters to a measured series. Returns a
/// `ParameterFitResponse` JSON string.
#[wasm_bindgen]
pub fn parameter_fit(request_json: &str) -> String {
    match parameter_fit_inner(request_json) {
        Ok(value) => value.to_string(),
        Err(message) => json!({
            "success": false,
            "error": message,
            "parameterNames": [],
            "fittedValues": [],
            "rmse": 0.0,
            "initialRmse": 0.0,
            "evaluations": 0,
            "truncated": false,
            "fittedT": [],
            "fittedV": [],
        })
        .to_string(),
    }
}

fn parameter_fit_inner(request_json: &str) -> Result<Value, String> {
    frees_core::props::tables::install_builtin_once();
    let request: ParameterFitRequest =
        serde_json::from_str(request_json).map_err(|e| format!("Invalid request: {e}"))?;

    // `validateParameterFit`, in the Java's order and words.
    if request.text.trim().is_empty() {
        return Err("The model document is required.".to_string());
    }
    if request.parameters.is_empty()
        || request.initial.len() != request.parameters.len()
        || request.lower.len() != request.parameters.len()
        || request.upper.len() != request.parameters.len()
    {
        return Err("Each parameter needs an initial value and lower/upper bounds.".to_string());
    }
    if request.ode_block.trim().is_empty() || request.column.trim().is_empty() {
        return Err("Pick the DYNAMIC block and the column to fit against.".to_string());
    }
    if request.measured_t.len() != request.measured_v.len() || request.measured_t.len() < 2 {
        return Err(
            "The measured series needs at least two (t, y) samples of equal length.".to_string(),
        );
    }
    if request.measured_t.len() > MAX_FIT_SAMPLES {
        return Err(format!(
            "The measured series has too many samples ({}; limit {MAX_FIT_SAMPLES}). \
             Decimate it first.",
            request.measured_t.len()
        ));
    }
    if let Err(failure) = frees_core::parse_document(&request.text) {
        return Err(syntax_error(&failure));
    }

    let facade = SolveRequest {
        variable_info: request.variable_info,
        stop_criteria: request.stop_criteria,
        display_unit_system: None,
        fill_missing: None,
        function_tables: None,
    };
    let extra_tables = function_table_defs_of(&request.function_tables);
    let settings = settings_of(&facade);
    let overrides = overrides_of(&facade);

    let max_evaluations = match request.max_evaluations {
        Some(v) => (v.max(10) as usize).min(MAX_FIT_EVALUATIONS),
        None => DEFAULT_FIT_EVALUATIONS.min(MAX_FIT_EVALUATIONS),
    };
    let parameters: Vec<String> = request
        .parameters
        .iter()
        .map(|p| p.trim().to_ascii_lowercase())
        .collect();
    let fit_request = frees_core::analysis::paramfit::FitRequest {
        text: &request.text,
        parameters: &parameters,
        initial: &request.initial,
        lower: &request.lower,
        upper: &request.upper,
        ode_block: &request.ode_block,
        column: &request.column,
        measured_t: &request.measured_t,
        measured_v: &request.measured_v,
        max_evaluations,
    };
    let started = now_ms();
    let solve = |text: &str| -> Option<Vec<frees_core::analysis::paramfit::OdeTableView>> {
        // The Java `ParameterFit.evaluate` threads the request's function
        // defs into every fit solve; the callback carries them here.
        frees_core::solve_with_tables(text, &settings, &overrides, &extra_tables)
            .ok()
            .map(|solution| {
                solution
                    .ode_tables
                    .iter()
                    .map(|table| frees_core::analysis::paramfit::OdeTableView {
                        name: table.name.clone(),
                        columns: table.columns.clone(),
                        rows: table
                            .rows
                            .iter()
                            .map(|row| row.iter().map(|v| Some(*v)).collect())
                            .collect(),
                    })
                    .collect()
            })
    };
    let outcome = frees_core::analysis::paramfit::run(&fit_request, solve, || {
        (now_ms() - started) / 1000.0 > MAX_FIT_SECONDS
    })
    .map_err(|e| e.to_string_message())?;

    Ok(json!({
        "success": true,
        "error": Value::Null,
        "parameterNames": outcome.parameters,
        "fittedValues": outcome.fitted,
        "rmse": outcome.rmse,
        "initialRmse": outcome.initial_rmse,
        "evaluations": outcome.evaluations,
        "truncated": outcome.truncated,
        "fittedT": outcome.fitted_series.t,
        "fittedV": outcome.fitted_series.v,
    }))
}

// ---------------------------------------------------------------------------
// Wave B4: the two ControlController endpoints. Both are unit-free by
// construction — bare SI coefficient arrays in and out, no display-unit or
// variableInfo plumbing anywhere on the path.
// ---------------------------------------------------------------------------

/// The Java's `min(points, 2000)` cap and its 400 fallback.
const MAX_TUNE_POINTS: usize = 2_000;
const DEFAULT_TUNE_POINTS: usize = 400;

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct PidTuneRequest {
    num: Vec<f64>,
    den: Vec<f64>,
    #[serde(rename = "type")]
    kind: Option<String>,
    wc: Option<f64>,
    pm: Option<f64>,
    horizon: Option<f64>,
    points: Option<i64>,
}

/// Loop-shaping PID tuning (`POST /api/control/pidtune`). Returns a
/// `TuneResponse` JSON string; a refused request carries a top-level
/// `"error"` — the same body the Java's 400 sends.
#[wasm_bindgen]
pub fn pid_tune(request_json: &str) -> String {
    match pid_tune_inner(request_json) {
        Ok(value) => value.to_string(),
        Err(message) => json!({ "error": message }).to_string(),
    }
}

fn pid_tune_inner(request_json: &str) -> Result<Value, String> {
    let request: PidTuneRequest =
        serde_json::from_str(request_json).map_err(|e| format!("Invalid request: {e}"))?;

    // The controller's checks and defaults, in its order and words.
    if request.num.is_empty() || request.den.is_empty() {
        return Err(
            "A plant transfer function (num and den coefficients) is required.".to_string(),
        );
    }
    let raw_kind = request.kind.clone();
    let kind = raw_kind
        .as_deref()
        .map(str::to_ascii_lowercase)
        .unwrap_or_else(|| "pi".to_string());
    if !matches!(kind.as_str(), "p" | "pi" | "pid") {
        // The Java interpolates the raw, un-lowercased type.
        return Err(format!(
            "Controller type must be one of p, pi, pid (got '{}').",
            raw_kind.as_deref().unwrap_or("")
        ));
    }
    let wc = match request.wc {
        Some(w) if w > 0.0 => w,
        _ => frees_core::control::pid::suggest_wc(&request.num, &request.den),
    };
    let pm = match request.pm {
        Some(p) if p > 0.0 && p < 90.0 => p,
        _ => 60.0,
    };
    let horizon = request.horizon.unwrap_or(0.0);
    let points = match request.points {
        Some(p) if p > 0 => (p as usize).min(MAX_TUNE_POINTS),
        _ => DEFAULT_TUNE_POINTS,
    };

    let result =
        frees_core::control::pid::tune(&request.num, &request.den, &kind, wc, pm, horizon, points)
            .map_err(|e| e.to_string_message())?;

    // `wc`/`pm` echo the *resolved request* values (the Java contract); the
    // realized margins ride in gainMargin/phaseMargin. `w_gm`/`w_pm` have no
    // slot in the Java DTO and are dropped.
    Ok(json!({
        "kp": result.kp,
        "ki": result.ki,
        "kd": result.kd,
        "wc": wc,
        "pm": pm,
        "t": result.t,
        "y": result.y,
        "riseTime": result.rise_time,
        "peakTime": result.peak_time,
        "settlingTime": result.settling_time,
        "overshoot": result.overshoot,
        "gainMargin": result.gain_margin,
        "phaseMargin": result.phase_margin,
    }))
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct PlantRequest {
    text: String,
    // `Option` because the Java 400s on *null* while a blank string passes
    // through to fail later — transcribed as-is.
    dynamic: Option<String>,
    reference: Option<String>,
    output: Option<String>,
    reference_on_sp: bool,
    #[serde(rename = "type")]
    kind: Option<String>,
    kp: f64,
    ki: f64,
    kd: f64,
}

/// Linearize a closed PID loop and recover the open-loop plant
/// (`POST /api/control/plant`). Returns `{num, den}` or `{"error": …}`.
///
/// Composition of the Java `computePlant`, step for step: shrink the DYNAMIC
/// header to a 2-point/1-second run, perturb the reference `SigConstant`
/// through an injected free variable, append the `LINEARIZE freespidlin`
/// block, run one ordinary solve (LINEARIZE rides the normal `solve_with`
/// path — no accessor bridge involved), read the `freespid_a…d` matrices
/// back out of `solution.values`, and undo the loop algebra with
/// `ss_to_tf`/`controller_tf`/`recover_plant`.
///
/// One budget divergence, recorded rather than faked: the Java hands the
/// solve a 40 s wall-clock cap (`LINEARIZE_BUDGET_S`); this port's
/// `SolverSettings` has no time field (core has no clock on wasm32), so the
/// single solve runs uncapped. The three numeric settings are identical to
/// `SolverSettings::default()`.
#[wasm_bindgen]
pub fn extract_plant(request_json: &str) -> String {
    match extract_plant_inner(request_json) {
        Ok(value) => value.to_string(),
        Err(message) => json!({ "error": message }).to_string(),
    }
}

fn extract_plant_inner(request_json: &str) -> Result<Value, String> {
    frees_core::props::tables::install_builtin_once();
    let request: PlantRequest =
        serde_json::from_str(request_json).map_err(|e| format!("Invalid request: {e}"))?;

    if request.text.trim().is_empty() {
        return Err("The document text is required to linearize the loop.".to_string());
    }
    let (Some(dynamic), Some(reference), Some(output)) = (
        request.dynamic.clone(),
        request.reference.clone(),
        request.output.clone(),
    ) else {
        return Err(
            "The DYNAMIC block name, reference source and measured output are required."
                .to_string(),
        );
    };
    let kind = request
        .kind
        .as_deref()
        .map(str::to_ascii_lowercase)
        .unwrap_or_else(|| "pi".to_string());

    let shrunk = shrink_dynamic(&request.text, &dynamic);
    let Some(injected) = inject_reference_variable(&shrunk, &reference, "freespidref") else {
        return Err(format!(
            "Could not find a constant to perturb on reference source '{reference}' \
             (expected a SigConstant with a k= value)."
        ));
    };
    let doc = format!(
        "{injected}\nLINEARIZE freespidlin(block = {dynamic}, a = freespid_a, \
         b = freespid_b, c = freespid_c, d = freespid_d)\n  INPUT freespidref\n  \
         OUTPUT {output}\nEND\n"
    );

    let solution = frees_core::solve_with(&doc, &frees_core::SolverSettings::default(), &[])
        .map_err(|failure| failure.to_string_message())?;

    let a = read_matrix(&solution.values, "freespid_a");
    let n = a.len();
    if n == 0 {
        return Err("The linearized loop has no states — nothing to identify.".to_string());
    }
    let b_mat = read_matrix(&solution.values, "freespid_b");
    let b: Vec<f64> = (0..n)
        .map(|i| {
            b_mat
                .get(i)
                .and_then(|row| row.first())
                .copied()
                .unwrap_or(0.0)
        })
        .collect();
    let c_mat = read_matrix(&solution.values, "freespid_c");
    let c: Vec<f64> = match c_mat.first() {
        Some(row) => {
            let mut row = row.clone();
            row.resize(n, 0.0);
            row
        }
        None => vec![0.0; n],
    };
    let d = read_matrix(&solution.values, "freespid_d")
        .first()
        .and_then(|row| row.first())
        .copied()
        .unwrap_or(0.0);

    let (m_num, m_den) = frees_core::control::pid::ss_to_tf(&a, &b, &c, d);
    let (c_num, c_den) =
        frees_core::control::pid::controller_tf(&kind, request.kp, request.ki, request.kd)
            .map_err(|e| e.to_string_message())?;
    let (g_num, g_den) = frees_core::control::pid::recover_plant(
        &m_num,
        &m_den,
        &c_num,
        &c_den,
        request.reference_on_sp,
    );
    Ok(json!({ "num": g_num, "den": g_den }))
}

/// `ControlController.shrinkDynamic`, hand-scanned (this crate carries no
/// regex engine): inside the named block's header parentheses, the time span
/// becomes `0 .. 1` (keeping the key as written) and `points` becomes 2. No
/// matching header leaves the text unchanged, exactly like the Java.
fn shrink_dynamic(text: &str, dynamic: &str) -> String {
    let lower = text.to_ascii_lowercase();
    let needle = "dynamic";
    let name_lower = dynamic.to_ascii_lowercase();
    let mut search = 0usize;
    while let Some(pos) = lower[search..].find(needle) {
        let start = search + pos;
        // Word boundary on both sides of `dynamic`.
        let before_ok = start == 0
            || !lower.as_bytes()[start - 1].is_ascii_alphanumeric()
                && lower.as_bytes()[start - 1] != b'_';
        let mut cursor = start + needle.len();
        let bytes = text.as_bytes();
        // Require whitespace, then the block name, then optional ws and '('.
        let mut ok = before_ok && cursor < bytes.len() && bytes[cursor].is_ascii_whitespace();
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        let name_end = cursor + name_lower.len();
        ok = ok && name_end <= bytes.len() && lower[cursor..name_end] == name_lower && {
            let mut after = name_end;
            while after < bytes.len() && bytes[after].is_ascii_whitespace() {
                after += 1;
            }
            after < bytes.len() && bytes[after] == b'(' && {
                cursor = after;
                true
            }
        };
        if !ok {
            search = start + needle.len();
            continue;
        }
        let args_start = cursor + 1;
        let Some(rel_close) = text[args_start..].find(')') else {
            return text.to_string();
        };
        let args_end = args_start + rel_close;
        let rewritten: Vec<String> = text[args_start..args_end]
            .split(',')
            .map(|piece| {
                let trimmed = piece.trim();
                let key = trimmed
                    .split('=')
                    .next()
                    .map(str::trim)
                    .unwrap_or("")
                    .to_ascii_lowercase();
                if (key == "time" || key == "t") && trimmed.contains("..") {
                    let written_key = trimmed.split('=').next().map(str::trim).unwrap_or("time");
                    format!("{written_key} = 0 .. 1")
                } else if key == "points" {
                    "points = 2".to_string()
                } else {
                    trimmed.to_string()
                }
            })
            .collect();
        return format!(
            "{}{}{}",
            &text[..args_start],
            rewritten.join(", "),
            &text[args_end..]
        );
    }
    text.to_string()
}

/// `ControlController.injectReferenceVariable`, hand-scanned: the first
/// `<reference>( … k = <value> … )` gets its `k` bound to `var_name`, and the
/// original value becomes a prepended free assignment. `None` when either
/// the instance or its `k=` is missing.
fn inject_reference_variable(text: &str, reference: &str, var_name: &str) -> Option<String> {
    let bytes = text.as_bytes();
    let mut search = 0usize;
    let instance_start = loop {
        let pos = text[search..].find(reference)?;
        let start = search + pos;
        let before_ok =
            start == 0 || !(bytes[start - 1].is_ascii_alphanumeric() || bytes[start - 1] == b'_');
        let mut after = start + reference.len();
        while after < bytes.len() && bytes[after].is_ascii_whitespace() {
            after += 1;
        }
        if before_ok && after < bytes.len() && bytes[after] == b'(' {
            break (after + 1, start);
        }
        search = start + reference.len();
    };
    let (args_start, _) = (instance_start.0, instance_start.1);
    let args_end = args_start + text[args_start..].find(')')?;
    let args = &text[args_start..args_end];
    // Find `k = <value>` (k as its own word) inside the args.
    let args_lower = args.to_ascii_lowercase();
    let mut k_search = 0usize;
    let (value_start, value_end) = loop {
        let pos = args_lower[k_search..].find('k')?;
        let at = k_search + pos;
        let before_ok = at == 0
            || !(args.as_bytes()[at - 1].is_ascii_alphanumeric()
                || args.as_bytes()[at - 1] == b'_');
        let mut cursor = at + 1;
        while cursor < args.len() && args.as_bytes()[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if before_ok && cursor < args.len() && args.as_bytes()[cursor] == b'=' {
            cursor += 1;
            while cursor < args.len() && args.as_bytes()[cursor].is_ascii_whitespace() {
                cursor += 1;
            }
            let rel_end = args[cursor..].find(',').unwrap_or(args.len() - cursor);
            break (cursor, cursor + rel_end);
        }
        k_search = at + 1;
    };
    let value = args[value_start..value_end].trim().to_string();
    if value.is_empty() {
        return None;
    }
    let mut out = String::with_capacity(text.len() + var_name.len() * 2 + value.len() + 8);
    out.push_str(var_name);
    out.push_str(" = ");
    out.push_str(&value);
    out.push('\n');
    out.push_str(&text[..args_start + value_start]);
    out.push_str(var_name);
    out.push_str(&text[args_start + value_end..]);
    Some(out)
}

/// `ControlController.readMatrix`: gather `name[i,j]` cells (lowercase,
/// 1-indexed) from the solved values, size the matrix to the maxima, and
/// default missing cells to zero.
fn read_matrix(values: &BTreeMap<String, f64>, name: &str) -> Vec<Vec<f64>> {
    let prefix = format!("{name}[");
    let mut cells: Vec<(usize, usize, f64)> = Vec::new();
    let mut rows = 0usize;
    let mut cols = 0usize;
    for (key, &value) in values.range(prefix.clone()..) {
        if !key.starts_with(&prefix) {
            break;
        }
        let inner = &key[prefix.len()..key.len().saturating_sub(1)];
        let Some((i_txt, j_txt)) = inner.split_once(',') else {
            continue; // the 1-D single-column spelling; the [i,j] one carries it
        };
        let (Ok(i), Ok(j)) = (i_txt.trim().parse::<usize>(), j_txt.trim().parse::<usize>()) else {
            continue;
        };
        if i == 0 || j == 0 {
            continue;
        }
        rows = rows.max(i);
        cols = cols.max(j);
        cells.push((i - 1, j - 1, value));
    }
    let mut out = vec![vec![0.0; cols]; rows];
    for (i, j, value) in cells {
        out[i][j] = value;
    }
    out
}
