//! Parametric run-tables — sweeping a document and reading the table back.
//!
//! Port of `../frEES/backend/core/src/main/java/com/frees/backend/core/ParametricAccessorContext.java`
//! (159 LOC) and the run driver `SolveController.computeSolveTable` /
//! `solveTableRows` / `solveTableWithAccessors` / `buildColumns` /
//! `columnsConverged` / `solveTableRow`
//! (`../frEES/backend/web/src/main/java/com/frees/backend/api/SolveController.java`).
//! The declaration this executes is [`ParametricTable`], parsed by
//! [`crate::parser::toplevel`] into [`crate::parser::blocks`].
//!
//! # What a parametric run is
//!
//! A `PARAMETRIC` block declares columns and their values:
//!
//! ```text
//! PARAMETRIC drive (t, v, P)
//!   t = 0:2:30
//! END
//! ```
//!
//! `t` is *swept* (16 values), `v` and `P` are declared **outputs** with no
//! cells. One run per row: the row's non-`None` cells are pinned, the document
//! is re-solved, and the solved values fill the output columns. That is why a
//! sweep document is legitimately underspecified on its own — the base system
//! has one degree of freedom that the swept column supplies — and why a plain
//! Solve of `fixtures/corpus-pending/corpus/driving-cycle-energy.frees` fails
//! with `SolverException` in the Java engine too.
//!
//! # How a row is pinned: text, not overrides
//!
//! `solveTableRow` appends `\n<name> = <value>` to the document for every
//! non-null cell and re-solves the result. That is not an implementation
//! shortcut to route around — pinning a swept variable has to make it a
//! **known**, i.e. an equation, and `VariableOverride` only carries a guess and
//! bounds. [`row_source`] does exactly what the Java does, which is also what
//! makes a row a self-contained job: a string plus settings, nothing shared.
//!
//! # The accessors, and why they need a fixed point
//!
//! `TableSum('P')`, `TableAvg`, `IntegralValue('P', 't')`, `TableValue(run,
//! col)`, `TableRun#` and `NParametricRuns` let an equation in *one* row read an
//! aggregate of *every* row — `E_total = IntegralValue('P', 't')` is the whole
//! cycle's energy, identical in every row. That is circular by construction, so
//! the Java solves the table as a Gauss–Seidel fixed point: pass 0 runs with no
//! table data installed (every accessor reads 0), each later pass installs the
//! columns collected from the previous pass, and the loop stops when the columns
//! stop moving or [`MAX_PARAMETRIC_PASSES`] is reached. [`run_sweep`] is that
//! loop.
//!
//! # Threading: the shape is the point (decision D3)
//!
//! `docs/decisions/0003-threading-model.md` chose a pool of independent workers
//! over shared memory, and named parametric sweeps as the case that pays for it.
//! Nothing here spawns, locks or shares: a run is described by a
//! [`RowJob`] — an owned source string and a run index — and answered by a
//! [`RowOutcome`], so a pool can drive [`ParametricTable::rows`]-derived jobs
//! directly and reassemble the outcomes in order. The one thing that cannot fan
//! out is an accessor table, which is table-wide by definition; the Java refuses
//! to chunk those for the same reason (`SolveController.solveTable`, "accessor
//! tables keep their table-wide fixed-point semantics and stay one serial
//! task").
//!
//! The Java's context is a `ThreadLocal` because its evaluator reaches it
//! through a static. A thread-bound global would be both meaningless and
//! unavailable in wasm, so the port passes [`ParametricAccessors`] explicitly
//! and models "no parametric solve is active" as `None` — the null the Java's
//! `CURRENT.get()` returns, with the same defaults on every accessor.

use std::collections::BTreeMap;

use crate::diag::{FreesError, Result};
use crate::parser::blocks::ParametricTable;

/// Maximum table-wide fixed-point passes when accessors are present. Port of
/// `SolveController.MAX_PARAMETRIC_PASSES`.
pub const MAX_PARAMETRIC_PASSES: usize = 12;

/// Convergence tolerance between two passes' columns, per
/// `SolveController.columnsConverged`: `|x - y| > 1e-9 * (1 + |y|)` is a move.
const COLUMN_TOLERANCE: f64 = 1e-9;

// ── the accessor context ────────────────────────────────────────────────────

/// The whole table, as the equations of one row see it.
///
/// Port of `ParametricAccessorContext`. `columns` maps a **lowercase** variable
/// name to its solved value in each run (`NaN` where that run did not solve);
/// `var_order` is the table's declared column order, so `TableValue(run, col)`
/// can address a cell by 1-based column index.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ParametricAccessors {
    /// 1-based index of the run being solved.
    current_run: usize,
    total_runs: usize,
    columns: BTreeMap<String, Vec<f64>>,
    var_order: Vec<String>,
}

impl ParametricAccessors {
    /// Port of `ParametricAccessorContext.install` — minus the thread binding,
    /// which the caller replaces by handing the value to the row it belongs to.
    pub fn install(
        current_run: usize,
        total_runs: usize,
        columns: BTreeMap<String, Vec<f64>>,
        var_order: Vec<String>,
    ) -> ParametricAccessors {
        ParametricAccessors {
            current_run,
            total_runs,
            columns,
            var_order,
        }
    }

    /// `TableRun#` — the 1-based index of the run being solved.
    pub fn current_run(&self) -> f64 {
        self.current_run as f64
    }

    /// Re-point an installed context at another run of the **same** table.
    ///
    /// The Java re-`install`s per row because its context is immutable and
    /// thread-bound; the columns it hands over are the same map every time. A
    /// pass over a 5 000-row table would otherwise clone the whole table 5 000
    /// times here, so [`run_sweep`] installs once per pass and moves the run
    /// index — the only field that varies within a pass.
    pub fn set_run(&mut self, run: usize) {
        self.current_run = run;
    }

    /// `NParametricRuns` — how many runs the table has.
    pub fn run_count(&self) -> f64 {
        self.total_runs as f64
    }

    /// `TableValue(run, col)` — one cell, by 1-based run and column index.
    ///
    /// # Errors
    ///
    /// [`FreesError::Solver`] when `col` is outside `1..=var_order.len()`, the
    /// one thing this type refuses rather than defaulting: a column index is a
    /// literal in the document, so an out-of-range one is a typo the user can
    /// fix, not missing data. An out-of-range *run* is missing data and reads 0.
    pub fn cell(&self, run: i64, col: i64) -> Result<f64> {
        if col < 1 || col > self.var_order.len() as i64 {
            return Err(FreesError::solver(format!(
                "TableValue: column {col} out of range 1..{}.",
                self.var_order.len()
            )));
        }
        Ok(self.value_at(&self.var_order[(col - 1) as usize], run))
    }

    /// A column aggregate over a named column: `sum`, `avg`, `min`, `max` or
    /// `stddev`. An unknown operation reads 0, exactly as the Java's `default`
    /// arm does.
    ///
    /// Non-finite (unsolved) entries are dropped first, so a table with a few
    /// failed rows still reports a meaningful mean of the rows that solved.
    pub fn aggregate(&self, op: &str, column: &str) -> f64 {
        let data = self.finite_column(column);
        if data.is_empty() {
            return 0.0;
        }
        match op {
            "sum" => data.iter().sum(),
            "avg" => data.iter().sum::<f64>() / data.len() as f64,
            // `DoubleStream.min/max`, which order by `Double.compare` — total,
            // and NaN-free here because `finite_column` already filtered.
            "min" => data.iter().copied().fold(f64::INFINITY, f64::min),
            "max" => data.iter().copied().fold(f64::NEG_INFINITY, f64::max),
            "stddev" => std_dev(&data),
            _ => 0.0,
        }
    }

    /// `IntegralValue(y, x)` — the trapezoid integral of column `y` with
    /// respect to column `x`. Segments touching a non-finite endpoint are
    /// skipped rather than poisoning the total.
    pub fn integral(&self, y_column: &str, x_column: &str) -> f64 {
        let ys = self.column(y_column);
        let xs = self.column(x_column);
        let mut total = 0.0;
        for i in 1..xs.len() {
            // The Java indexes `ys` with `xs`'s loop bound; a shorter `ys`
            // would throw there, so guard rather than diverge.
            let (Some(&y1), Some(&y0)) = (ys.get(i), ys.get(i - 1)) else {
                continue;
            };
            if xs[i].is_finite() && xs[i - 1].is_finite() && y1.is_finite() && y0.is_finite() {
                total += 0.5 * (xs[i] - xs[i - 1]) * (y1 + y0);
            }
        }
        total
    }

    /// The declared column order this context was installed with.
    pub fn var_order(&self) -> &[String] {
        &self.var_order
    }

    /// One run's value of a column, 0 when the run is out of range or the value
    /// is not finite.
    fn value_at(&self, column: &str, run: i64) -> f64 {
        let data = self.column(column);
        if run < 1 || run > data.len() as i64 {
            return 0.0;
        }
        let v = data[(run - 1) as usize];
        if v.is_finite() {
            v
        } else {
            0.0
        }
    }

    /// A column by name, empty when the table has no such column.
    fn column(&self, column: &str) -> &[f64] {
        self.columns
            .get(&column.to_ascii_lowercase())
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// A column with the non-finite (unsolved) entries removed.
    fn finite_column(&self, column: &str) -> Vec<f64> {
        self.column(column)
            .iter()
            .copied()
            .filter(|v| v.is_finite())
            .collect()
    }
}

/// Population standard deviation, as `ParametricAccessorContext.stdDev`
/// computes it (divide by `n`, not `n - 1`).
fn std_dev(data: &[f64]) -> f64 {
    let mean = data.iter().sum::<f64>() / data.len() as f64;
    let sum_sq: f64 = data.iter().map(|v| (v - mean) * (v - mean)).sum();
    libm::sqrt(sum_sq / data.len() as f64)
}

// ── the Java statics, with `null` spelled `None` ────────────────────────────
//
// `ParametricAccessorContext`'s public surface is static and reads
// `CURRENT.get()`, which is null outside a parametric solve. Each of these
// mirrors one of those methods including its null default, so an evaluator can
// call them with whatever context it has — `None` included — and get the Java's
// answer. They are the port's join point for the accessor intrinsics, which
// `crate::eval` still refuses by name.

/// `TableRun#` — **1** outside a parametric solve.
pub fn current_run(ctx: Option<&ParametricAccessors>) -> f64 {
    ctx.map_or(1.0, ParametricAccessors::current_run)
}

/// `NParametricRuns` — **0** outside a parametric solve.
pub fn run_count(ctx: Option<&ParametricAccessors>) -> f64 {
    ctx.map_or(0.0, ParametricAccessors::run_count)
}

/// `TableValue(run, col)` — 0 outside a parametric solve, where there is no
/// column list to range-check against either.
pub fn cell(ctx: Option<&ParametricAccessors>, run: i64, col: i64) -> Result<f64> {
    match ctx {
        Some(ctx) => ctx.cell(run, col),
        None => Ok(0.0),
    }
}

/// `TableSum` / `TableAvg` / `TableMin` / `TableMax` / `TableStdDev` — 0
/// outside a parametric solve.
pub fn aggregate(ctx: Option<&ParametricAccessors>, op: &str, column: &str) -> f64 {
    ctx.map_or(0.0, |ctx| ctx.aggregate(op, column))
}

/// `IntegralValue(y, x)` — 0 outside a parametric solve.
pub fn integral(ctx: Option<&ParametricAccessors>, y_column: &str, x_column: &str) -> f64 {
    ctx.map_or(0.0, |ctx| ctx.integral(y_column, x_column))
}

// ── the run driver ──────────────────────────────────────────────────────────

/// One row of the sweep, ready to dispatch.
///
/// Owned and self-contained on purpose: `source` is the whole document with the
/// row's cells pinned, so a worker needs nothing but this and the solver
/// settings (decision D3). `accessors` is `None` for the common accessor-free
/// table — which is exactly the case a pool may fan out.
#[derive(Debug, Clone, PartialEq)]
pub struct RowJob<'a> {
    /// 1-based run index, the value `TableRun#` reports.
    pub run: usize,
    /// Total runs in the table, the value `NParametricRuns` reports.
    pub total_runs: usize,
    /// The document with this row's cells appended as equations.
    pub source: String,
    /// The table as the previous pass left it, or `None` on an accessor-free
    /// table and on the first pass of an accessor table.
    pub accessors: Option<&'a ParametricAccessors>,
    /// The row's non-empty pinned cells as `(name, value)` pairs.
    pub cells: &'a [(String, f64)],
}

/// What one run produced.
///
/// `si_values` are keyed by **lowercase** canonical name and hold SI values —
/// the same normalisation `SolveController.solveTableRow` applies before
/// `buildColumns` reads them, so a column means the same thing in every pass.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RowOutcome {
    pub si_values: BTreeMap<String, f64>,
    /// The failure message when this run did not solve; a failed run
    /// contributes no values and leaves its cells `NaN`.
    pub error: Option<String>,
}

impl RowOutcome {
    pub fn solved(si_values: BTreeMap<String, f64>) -> RowOutcome {
        RowOutcome {
            si_values,
            error: None,
        }
    }

    pub fn failed(message: impl Into<String>) -> RowOutcome {
        RowOutcome {
            si_values: BTreeMap::new(),
            error: Some(message.into()),
        }
    }

    pub fn is_solved(&self) -> bool {
        self.error.is_none()
    }
}

/// The result of a whole sweep.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Sweep {
    /// One outcome per row, in table order.
    pub outcomes: Vec<RowOutcome>,
    /// The solved columns of the final pass, keyed lowercase — what an
    /// accessor would have read had there been another pass.
    pub columns: BTreeMap<String, Vec<f64>>,
    /// How many passes ran. Always 1 for an accessor-free table.
    pub passes: usize,
    /// Whether the columns stopped moving before [`MAX_PARAMETRIC_PASSES`].
    /// `true` for an accessor-free table, which is a fixed point by definition.
    pub converged: bool,
}

impl Sweep {
    pub fn solved_count(&self) -> usize {
        self.outcomes.iter().filter(|o| o.is_solved()).count()
    }

    pub fn failed_count(&self) -> usize {
        self.outcomes.len() - self.solved_count()
    }
}

/// The rows of a table as `(name, value)` cell lists, dropping the `None`
/// cells the way `solveTableRow` skips its null values.
///
/// The names keep the case the header declared, because they are written back
/// into the document as equations and the engine is case-insensitive anyway.
pub fn row_cells(table: &ParametricTable) -> Vec<Vec<(String, f64)>> {
    table
        .rows
        .iter()
        .map(|row| {
            table
                .vars
                .iter()
                .zip(row)
                .filter_map(|(var, cell)| cell.map(|value| (var.clone(), value)))
                .collect()
        })
        .collect()
}

/// The document one row solves: the base text plus `\n<name> = <value>` for
/// every pinned cell.
///
/// Port of `SolveController.solveTableRow`'s `StringBuilder`. The value is
/// rendered with Rust's shortest round-tripping form rather than Java's
/// `Double.toString`; the two differ in spelling (`2` vs `2.0`, `0.0001` vs
/// `1.0E-4`) and never in value, because both are exact round trips and the
/// lexer parses both to the same `f64`.
pub fn row_source(base_source: &str, cells: &[(String, f64)]) -> String {
    let mut source = String::with_capacity(base_source.len() + cells.len() * 24);
    source.push_str(base_source);
    for (name, value) in cells {
        source.push('\n');
        source.push_str(name);
        source.push_str(" = ");
        source.push_str(&format!("{value}"));
    }
    source
}

/// Run a whole parametric table, one solve per row.
///
/// `solve_row` is the engine call, injected so this module drives sweeps
/// without owning a solver and so a worker pool can supply a dispatching
/// implementation instead (decision D3). It is handed an owned [`RowJob`] and
/// must answer a [`RowOutcome`] for every row, in order.
///
/// Which of the Java's two paths runs is decided by
/// [`mentions_parametric_accessor`] over the *base document*, exactly as
/// `computeSolveTable` decides it: a table whose text names no accessor is a
/// single pass of independent rows; one that does is re-solved whole until its
/// columns stabilise.
pub fn run_sweep<F>(table: &ParametricTable, base_source: &str, mut solve_row: F) -> Sweep
where
    F: FnMut(RowJob<'_>) -> RowOutcome,
{
    let cells = row_cells(table);
    let total_runs = cells.len();
    // Built once and cloned per job rather than rebuilt per pass: a row's
    // pinned text never changes, only what the accessors read.
    let sources: Vec<String> = cells
        .iter()
        .map(|row| row_source(base_source, row))
        .collect();

    // The common case: rows are independent, so one pass is the fixed point.
    if !mentions_parametric_accessor(base_source) {
        let outcomes: Vec<RowOutcome> = sources
            .iter()
            .zip(&cells)
            .enumerate()
            .map(|(i, (source, row))| {
                solve_row(RowJob {
                    run: i + 1,
                    total_runs,
                    source: source.clone(),
                    accessors: None,
                    cells: row.as_slice(),
                })
            })
            .collect();
        let columns = build_columns(&outcomes, total_runs);
        return Sweep {
            outcomes,
            columns,
            passes: 1,
            converged: true,
        };
    }

    // The accessor case: Gauss–Seidel over the whole table.
    //
    // `TableValue(run, col)` addresses a column by its 1-based *declared*
    // position, so the order is the header's; the names are lowercased because
    // that is the key space `build_columns` produces.
    let var_order: Vec<String> = table.vars.iter().map(|v| v.to_ascii_lowercase()).collect();
    let mut columns: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    let mut outcomes = Vec::new();
    let mut passes = 0;
    let mut converged = false;
    for _ in 0..MAX_PARAMETRIC_PASSES {
        passes += 1;
        // Pass 0 installs an empty table, which is what makes every accessor
        // read 0 on the first pass. Installed once per pass and re-pointed per
        // row: the table a row sees is the *previous* pass's, identical for
        // every row in this one.
        let mut installed =
            ParametricAccessors::install(1, total_runs, columns.clone(), var_order.clone());
        outcomes = Vec::with_capacity(total_runs);
        for (i, (source, row)) in sources.iter().zip(&cells).enumerate() {
            installed.set_run(i + 1);
            outcomes.push(solve_row(RowJob {
                run: i + 1,
                total_runs,
                source: source.clone(),
                accessors: Some(&installed),
                cells: row.as_slice(),
            }));
        }
        let next = build_columns(&outcomes, total_runs);
        converged = columns_converged(&columns, &next);
        columns = next;
        if converged {
            break;
        }
    }
    Sweep {
        outcomes,
        columns,
        passes,
        converged,
    }
}

/// Collect each solved variable's SI value across all runs into a column.
///
/// Port of `SolveController.buildColumns`: a column is `NaN`-filled first, so a
/// run that failed — or one that simply never mentioned the variable — leaves a
/// `NaN` the aggregates then filter out.
pub fn build_columns(outcomes: &[RowOutcome], run_count: usize) -> BTreeMap<String, Vec<f64>> {
    let mut columns: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    for (i, outcome) in outcomes.iter().enumerate() {
        if i >= run_count {
            break;
        }
        for (name, value) in &outcome.si_values {
            columns
                .entry(name.clone())
                .or_insert_with(|| vec![f64::NAN; run_count])[i] = *value;
        }
    }
    columns
}

/// Whether two passes produced the same table.
///
/// Port of `SolveController.columnsConverged`: the key sets must match — a pass
/// that solved a new variable has moved — `NaN`-ness must agree entry by entry,
/// and finite entries must sit within `1e-9 * (1 + |y|)` of each other.
pub fn columns_converged(a: &BTreeMap<String, Vec<f64>>, b: &BTreeMap<String, Vec<f64>>) -> bool {
    if a.len() != b.len() || !a.keys().all(|k| b.contains_key(k)) {
        return false;
    }
    for (key, av) in a {
        let bv = &b[key];
        for (i, &x) in av.iter().enumerate() {
            // A column that changed length changed; the Java would throw here,
            // so treat the mismatch as "not converged" and let the loop retry.
            let Some(&y) = bv.get(i) else {
                return false;
            };
            if x.is_nan() != y.is_nan() {
                return false;
            }
            if !x.is_nan() && libm::fabs(x - y) > COLUMN_TOLERANCE * (1.0 + libm::fabs(y)) {
                return false;
            }
        }
        if bv.len() != av.len() {
            return false;
        }
    }
    true
}

/// The accessor function names, longest-first within a shared prefix so
/// `TableRun#` is tried before `TableRun`. Transcribed from
/// `SolveController.PARAMETRIC_ACCESSOR_PATTERN`.
const ACCESSOR_NAMES: &[&str] = &[
    "TableRun#",
    "TableRun",
    "NParametricRuns",
    "TableValue",
    "TableSum",
    "TableAvg",
    "TableMin",
    "TableMax",
    "TableStdDev",
    "IntegralValue",
];

/// Whether a document calls any parametric accessor, and therefore needs the
/// table-wide fixed point instead of independent rows.
///
/// Port of `SolveController.mentionsParametricAccessor`, whose regex is
/// `(?i)\b(TableRun#|TableRun|NParametricRuns|TableValue|TableSum|TableAvg|TableMin|TableMax|TableStdDev|IntegralValue)\s*\(`.
/// Hand-scanned rather than compiled: the port takes no new dependencies, and a
/// literal alternation needs none. The three parts of the pattern are all kept
/// — a word boundary before the name, ASCII-case-insensitive matching, and the
/// `\s*\(` that makes it a *call* rather than a variable of the same name.
///
/// Deliberately textual, like the Java: it runs over the raw document before
/// anything is parsed, so a name inside a comment counts. That costs an extra
/// pass over an accessor-free table at worst, and never a wrong answer.
pub fn mentions_parametric_accessor(text: &str) -> bool {
    let bytes = text.as_bytes();
    for start in 0..bytes.len() {
        // `\b`: the character before the name must not be a word character.
        // A UTF-8 continuation or lead byte is `>= 0x80` and so is not one,
        // which agrees with Java's `\w` = `[a-zA-Z_0-9]` (no `UNICODE_CASE`).
        if start > 0 && is_word_byte(bytes[start - 1]) {
            continue;
        }
        for name in ACCESSOR_NAMES {
            let end = start + name.len();
            if end > bytes.len() || !ascii_eq_ignore_case(&bytes[start..end], name.as_bytes()) {
                continue;
            }
            // `\s*\(`, with Java's `\s` = `[ \t\n\x0B\f\r]`.
            let mut i = end;
            while i < bytes.len() && matches!(bytes[i], b' ' | b'\t' | b'\n' | 0x0B | 0x0C | b'\r')
            {
                i += 1;
            }
            if bytes.get(i) == Some(&b'(') {
                return true;
            }
        }
    }
    false
}

fn is_word_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

fn ascii_eq_ignore_case(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x.eq_ignore_ascii_case(y))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn columns(entries: &[(&str, &[f64])]) -> BTreeMap<String, Vec<f64>> {
        entries
            .iter()
            .map(|(name, values)| ((*name).to_string(), values.to_vec()))
            .collect()
    }

    fn ctx() -> ParametricAccessors {
        // The Java `ParametricAccessorContextTest` fixture, verbatim: run 2 of
        // 4, columns t = [0,1,2,3] and q = [1,2,3,4], declared order [t, q].
        ParametricAccessors::install(
            2,
            4,
            columns(&[("t", &[0.0, 1.0, 2.0, 3.0]), ("q", &[1.0, 2.0, 3.0, 4.0])]),
            vec!["t".into(), "q".into()],
        )
    }

    #[test]
    fn aggregates_cell_and_integral_resolve_against_columns() {
        let c = ctx();
        assert_eq!(c.current_run(), 2.0);
        assert_eq!(c.run_count(), 4.0);
        // TableValue(3, 2) is q's third run.
        assert_eq!(c.cell(3, 2).unwrap(), 3.0);
        assert_eq!(c.aggregate("sum", "q"), 10.0);
        assert_eq!(c.aggregate("avg", "q"), 2.5);
        assert_eq!(c.aggregate("min", "q"), 1.0);
        assert_eq!(c.aggregate("max", "q"), 4.0);
        // Trapezoid of q over t on a unit grid: 1.5 + 2.5 + 3.5.
        assert_eq!(c.integral("q", "t"), 7.5);
        assert_eq!(c.var_order(), ["t", "q"]);
    }

    #[test]
    fn a_column_name_is_matched_case_insensitively() {
        assert_eq!(ctx().aggregate("sum", "Q"), 10.0);
        assert_eq!(ctx().integral("Q", "T"), 7.5);
    }

    #[test]
    fn stddev_is_the_population_deviation() {
        // mean 2.5, deviations ±1.5/±0.5 → sqrt(5/4).
        let expected = libm::sqrt(1.25);
        assert!((ctx().aggregate("stddev", "q") - expected).abs() < 1e-15);
    }

    #[test]
    fn an_unknown_aggregate_and_an_unknown_column_read_zero() {
        assert_eq!(ctx().aggregate("median", "q"), 0.0);
        assert_eq!(ctx().aggregate("sum", "nope"), 0.0);
        assert_eq!(ctx().integral("nope", "t"), 0.0);
    }

    #[test]
    fn unsolved_entries_are_dropped_from_aggregates_and_segments_from_integrals() {
        let c = ParametricAccessors::install(
            1,
            3,
            columns(&[("t", &[0.0, 1.0, 2.0]), ("q", &[1.0, f64::NAN, 3.0])]),
            vec!["t".into(), "q".into()],
        );
        assert_eq!(c.aggregate("sum", "q"), 4.0);
        assert_eq!(c.aggregate("avg", "q"), 2.0);
        // Both segments touch the NaN, so nothing accumulates.
        assert_eq!(c.integral("q", "t"), 0.0);
        // A cell that did not solve reads 0 rather than NaN.
        assert_eq!(c.cell(2, 2).unwrap(), 0.0);
    }

    #[test]
    fn an_out_of_range_run_reads_zero_but_an_out_of_range_column_is_an_error() {
        let c = ctx();
        assert_eq!(c.cell(0, 1).unwrap(), 0.0);
        assert_eq!(c.cell(99, 1).unwrap(), 0.0);
        let err = c.cell(1, 3).unwrap_err();
        assert!(matches!(err, FreesError::Solver { .. }), "{err}");
        assert!(err.to_string().contains("column 3 out of range 1..2"));
        assert!(c.cell(1, 0).is_err());
    }

    #[test]
    fn the_statics_default_the_way_a_null_context_does() {
        assert_eq!(current_run(None), 1.0);
        assert_eq!(run_count(None), 0.0);
        assert_eq!(cell(None, 1, 99).unwrap(), 0.0);
        assert_eq!(aggregate(None, "sum", "q"), 0.0);
        assert_eq!(integral(None, "q", "t"), 0.0);

        let c = ctx();
        assert_eq!(current_run(Some(&c)), 2.0);
        assert_eq!(run_count(Some(&c)), 4.0);
        assert_eq!(cell(Some(&c), 3, 2).unwrap(), 3.0);
        assert_eq!(aggregate(Some(&c), "sum", "q"), 10.0);
        assert_eq!(integral(Some(&c), "q", "t"), 7.5);
    }

    // ── the run driver ──────────────────────────────────────────────────────

    fn sweep_table() -> ParametricTable {
        // PARAMETRIC drive (t, P): t swept 0, 2, 4; P an output column.
        ParametricTable {
            name: "drive".into(),
            vars: vec!["t".into(), "P".into()],
            rows: vec![
                vec![Some(0.0), None],
                vec![Some(2.0), None],
                vec![Some(4.0), None],
            ],
        }
    }

    #[test]
    fn a_row_pins_only_its_declared_cells() {
        let cells = row_cells(&sweep_table());
        assert_eq!(cells.len(), 3);
        assert_eq!(cells[1], vec![("t".to_string(), 2.0)]);
        assert_eq!(row_source("P = t * 3", &cells[1]), "P = t * 3\nt = 2");
        assert_eq!(row_source("x = 1", &[]), "x = 1");
    }

    #[test]
    fn an_accessor_free_table_runs_one_pass_of_independent_rows() {
        let table = sweep_table();
        let mut seen = Vec::new();
        let sweep = run_sweep(&table, "P = t * 3", |job| {
            assert!(job.accessors.is_none(), "no accessors without a mention");
            assert_eq!(job.total_runs, 3);
            seen.push((job.run, job.source.clone()));
            let t = (job.run as f64 - 1.0) * 2.0;
            RowOutcome::solved(
                [("t".to_string(), t), ("p".to_string(), t * 3.0)]
                    .into_iter()
                    .collect(),
            )
        });
        assert_eq!(sweep.passes, 1);
        assert!(sweep.converged);
        assert_eq!(sweep.solved_count(), 3);
        assert_eq!(sweep.failed_count(), 0);
        assert_eq!(seen.len(), 3);
        assert_eq!(seen[2].1, "P = t * 3\nt = 4");
        assert_eq!(sweep.columns["p"], vec![0.0, 6.0, 12.0]);
    }

    #[test]
    fn an_accessor_table_iterates_to_a_fixed_point() {
        // `total` reads the whole P column, so pass 0 sees 0 and pass 1 sees
        // the real sum; pass 2 reproduces it and the loop stops.
        let table = sweep_table();
        let source = "P = t * 3\ntotal = TableSum('P')";
        let mut runs_per_pass = Vec::new();
        let sweep = run_sweep(&table, source, |job| {
            runs_per_pass.push(job.run);
            // The installed context is re-pointed per row, so `TableRun#` and
            // `NParametricRuns` track the job even though the columns do not.
            assert_eq!(current_run(job.accessors), job.run as f64);
            assert_eq!(run_count(job.accessors), 3.0);
            // `TableValue(2, 1)` is the second run's `t`, the declared column 1.
            let t_of_run_2 = cell(job.accessors, 2, 1).unwrap();
            let t = (job.run as f64 - 1.0) * 2.0;
            let total = aggregate(job.accessors, "sum", "p");
            RowOutcome::solved(
                [
                    ("t".to_string(), t),
                    ("p".to_string(), t * 3.0),
                    ("total".to_string(), total),
                    ("probe".to_string(), t_of_run_2),
                ]
                .into_iter()
                .collect(),
            )
        });
        assert!(sweep.converged, "the table should reach a fixed point");
        // Pass 0 (all zero), pass 1 (the real total), pass 2 (unchanged).
        assert_eq!(sweep.passes, 3);
        assert_eq!(runs_per_pass.len(), 9);
        assert_eq!(sweep.columns["total"], vec![18.0, 18.0, 18.0]);
        // Column 1 is `t`, whose run 2 is 2.0 once a pass has filled it in.
        assert_eq!(sweep.columns["probe"], vec![2.0, 2.0, 2.0]);
    }

    #[test]
    fn a_table_that_never_settles_stops_at_the_pass_ceiling() {
        let table = sweep_table();
        let mut nudge = 0.0;
        let sweep = run_sweep(&table, "x = TableSum('x')", |_| {
            nudge += 1.0;
            RowOutcome::solved([("x".to_string(), nudge)].into_iter().collect())
        });
        assert_eq!(sweep.passes, MAX_PARAMETRIC_PASSES);
        assert!(!sweep.converged);
    }

    #[test]
    fn a_failed_row_leaves_nan_in_its_cells_and_is_counted() {
        let outcomes = vec![
            RowOutcome::solved([("p".to_string(), 1.0)].into_iter().collect()),
            RowOutcome::failed("did not converge"),
            RowOutcome::solved([("p".to_string(), 3.0)].into_iter().collect()),
        ];
        let built = build_columns(&outcomes, 3);
        assert_eq!(built["p"][0], 1.0);
        assert!(built["p"][1].is_nan());
        assert_eq!(built["p"][2], 3.0);
        let sweep = Sweep {
            outcomes,
            columns: built,
            passes: 1,
            converged: true,
        };
        assert_eq!(sweep.solved_count(), 2);
        assert_eq!(sweep.failed_count(), 1);
    }

    #[test]
    fn convergence_needs_the_same_keys_the_same_gaps_and_close_values() {
        let base = columns(&[("p", &[1.0, 2.0])]);
        assert!(columns_converged(&base, &columns(&[("p", &[1.0, 2.0])])));
        // Inside 1e-9 * (1 + |y|).
        assert!(columns_converged(
            &base,
            &columns(&[("p", &[1.0, 2.0 + 1e-10])])
        ));
        assert!(!columns_converged(&base, &columns(&[("p", &[1.0, 2.1])])));
        // A new column is a move, even if the shared one held still.
        assert!(!columns_converged(
            &base,
            &columns(&[("p", &[1.0, 2.0]), ("q", &[0.0, 0.0])])
        ));
        // A row that started or stopped solving is a move.
        assert!(!columns_converged(
            &base,
            &columns(&[("p", &[1.0, f64::NAN])])
        ));
        // Two NaNs agree.
        let gappy = columns(&[("p", &[f64::NAN, 2.0])]);
        assert!(columns_converged(&gappy, &gappy.clone()));
        // A shorter column is a move, not a panic.
        assert!(!columns_converged(&base, &columns(&[("p", &[1.0])])));
        assert!(!columns_converged(&columns(&[("p", &[1.0])]), &base));
        // The empty first pass never matches a pass that solved anything.
        assert!(!columns_converged(&BTreeMap::new(), &base));
        assert!(columns_converged(&BTreeMap::new(), &BTreeMap::new()));
    }

    #[test]
    fn accessor_calls_are_recognised_the_way_the_java_regex_recognises_them() {
        for text in [
            "E = IntegralValue('P', 't')",
            "n = NParametricRuns()",
            "r = TableRun#()",
            "r = TableRun()",
            "v = TableValue(1, 2)",
            "s = tablesum('p')",
            "s = TABLESTDDEV('p')",
            "a = TableAvg ('p')", // `\s*` between the name and the paren
            "a = TableAvg\t('p')",
            "x = 1 + TableMin('p')",   // not at the start of the document
            "{ note: TableMax('p') }", // a comment counts, as in the Java
        ] {
            assert!(mentions_parametric_accessor(text), "missed: {text}");
        }
        for text in [
            "P = t * 3",
            "x = MyTableSum('p')",   // no word boundary before the name
            "x = TableSummary('p')", // the name must be followed by `(`
            "TableSum = 3",          // a variable, not a call
            "x = TableSum",
            "",
            "µ = 1 [kg]", // multi-byte bytes must not misalign
        ] {
            assert!(!mentions_parametric_accessor(text), "false hit: {text}");
        }
        // A multi-byte character is still a word boundary, like Java's `\b`.
        assert!(mentions_parametric_accessor("— TableSum('p')"));
    }

    #[test]
    fn an_empty_table_sweeps_nothing() {
        let empty = ParametricTable {
            name: "none".into(),
            vars: vec!["t".into()],
            rows: Vec::new(),
        };
        let sweep = run_sweep(&empty, "x = 1", |_| unreachable!("no rows to solve"));
        assert!(sweep.outcomes.is_empty());
        assert!(sweep.columns.is_empty());
        assert_eq!(sweep.passes, 1);
    }
}
