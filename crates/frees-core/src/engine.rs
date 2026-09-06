//! End-to-end steady solve: source text in, solved variables out.
//!
//! This is the Rust counterpart of the two entry points the parent engine
//! exposes over HTTP (`../frEES/backend/web/.../CheckController.java` and
//! `SolveController`), both of which delegate to
//! `../frEES/backend/core/.../EquationSystemSolver.java`:
//!
//! * [`check`] mirrors `POST /api/check` — verify syntax and *structural*
//!   solvability (zero degrees of freedom plus a complete equation↔variable
//!   matching) and report the counts, **without solving anything**. The
//!   frontend gates its Solve button on this.
//! * [`solve`] mirrors `POST /api/solve` — the full pipeline
//!   `parse → collect equations → apply GUESS → block → Newton per block`.
//!
//! # The pipeline
//!
//! The stage order is the Java order, established by reading
//! `EquationParser.parseResult` and `EquationSystemSolver.solve`:
//! `parseResult` lexes/parses, collects `FUNCTION`/`PROCEDURE`/`MODULE`/`TABLE`
//! definitions, **expands components**, then **flattens statements** —
//! `CALL` statements become binding equations and matrix constructs become
//! scalar equations *at parse time* — and `solve` then applies
//! `ComplexExpansion.expand` (when complex mode is on) before blocking. Hence:
//!
//! 1. **Parse.** [`crate::parser::parse_document`]. A block construct the wasm
//!    port has not reached yet is an explicit error, never a silent skip
//!    (`SYMBOLIC` is refused here too).
//! 2. **Expand components.** [`expand_component_layer`] — the acausal
//!    `COMPONENT`/`connect` layer, at `EquationParser.parseResult:292-306`.
//!    It runs **before** every other expansion pass and seeds the equation
//!    list, because the Java writes `new BoundedEquationList(componentEquations)`
//!    and then flattens the statements *into* it. The component layer is a
//!    parser/expander, not a second solver: what comes out is flat scalar
//!    equations that take the same Newton/Tarjan path as everything else.
//! 3. **Flatten CALLs.** [`crate::procedures::flatten_calls`] — the CALL half
//!    of `EquationParser.flatten`. (Phase-4 stub: refuses CALLs by name.)
//! 4. **Expand matrices.** [`crate::parser::expand::expand_document`] — the
//!    matrix half of `EquationParser` (multiAssign, rangeAssign, linear-algebra
//!    intrinsics). Scalar documents pass through byte-identical.
//! 5. **Expand complex.** [`crate::parser::complex::expand_complex`] with
//!    [`SolverSettings::complex_mode`] — `EquationSystemSolver.solve`'s
//!    `ComplexExpansion.expand` site, *after* parse-time flattening.
//! 6. **Seed.** Every unknown starts at [`DEFAULT_GUESS`] (`1.0`, the Java
//!    `EquationSystemSolver.DEFAULT_GUESS`) with bounds `±∞`; in-text `GUESS`
//!    directives override the guess and narrow the bounds, and the guess is
//!    clamped into the bounds exactly as `withTextGuesses` does.
//! 7. **Block.** [`crate::solver::blocker::block_system`] — degrees of freedom,
//!    maximum bipartite matching, Tarjan SCC. The blocks come out in solve
//!    order.
//! 8. **Solve.** [`crate::solver::newton::newton_solve`] per block, in that
//!    order, inside the Java retry ladder (see `solve_block_with_fallback`
//!    below): transformed-guess retries, a univariate bracketing rescue, block
//!    merging, then a best-effort polish pass. Residuals evaluate through
//!    [`crate::eval::eval_with`] with the document's definitions in context,
//!    so user `FUNCTION`s and `TABLE`s work the moment their evaluator lands.
//!    Values solved by an earlier block are already in the shared scope
//!    when a later block is evaluated, which is what "feed knowns forward"
//!    means here — a downstream block never re-solves an upstream unknown.
//!
//! # Built-in constants fold at parse time
//!
//! Like the Java `AstBuilder.visitVarAtom`, the expression parser substitutes
//! `pi#`/`R#`/`g#` (every name [`crate::eval::lookup_constant`] knows) as
//! numeric literals carrying their raw SI unit string, so a constant never
//! reaches the solver as a variable and the unit checker can ground downstream
//! variables from it. [`builtin_constants`] survives as a knowns hook — it
//! collects nothing on documents from this parser (folding leaves no `#`
//! variables behind) but keeps hand-built ASTs and the component expander
//! honest.
//!
//! # Evaluation failures inside the Newton loop
//!
//! [`crate::eval`] turns domain errors (division by zero, `ln` of a
//! non-positive, `sqrt` of a negative) into hard errors where Java returns
//! `NaN`. That is the right call for a user-facing expression evaluation and
//! the wrong one inside a line search, where the solver *expects* to probe
//! invalid regions and back off. The engine bridges the two: the residuals are
//! evaluated once at the initial point and any failure there is reported
//! verbatim (that is where a genuinely broken document — unknown function,
//! wrong arity, a string used as a number — shows up); after that, an
//! `Evaluation`/`Property` failure is reported to Newton as a non-finite
//! residual, which is the signal its Jacobian probing and step halving are
//! already written to handle.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use crate::ast::{Equation, Expr, Statement};
use crate::diag::{Diagnostic, FreesError, Result};
use crate::differentiator::differentiate;
use crate::eval::{eval_with, lookup_constant, EvalContext, Scope};
use crate::integral::IntegralEquation;
use crate::parser::defs::Definitions;
use crate::parser::{parse_document, Document, GuessDirective};
use crate::procedures::flatten_calls_counted;
use crate::solver::blocker::{block_system, unknowns, Block, BlockingReport};
use crate::solver::newton::{
    newton_solve_problem, newton_solve_problem_prepared, validate_solver_settings, NewtonProblem,
    NewtonWorkspace, SolverSettings,
};
// Aliased: `slots` is also the natural name for a slot *vector* here, and the
// two would shadow each other inside `solve_block`.
use crate::solver::slots as slotexpr;
use crate::solver::slots::CompiledBlock;
use crate::units::registry::UnitRegistry;
pub mod prepared;
pub use prepared::PreparedDocument;

/// Initial value for an unknown with no `GUESS`.
///
/// `EquationSystemSolver.DEFAULT_GUESS` in the Java engine.
pub const DEFAULT_GUESS: f64 = 1.0;

/// One equation's residual `lhs - rhs` at the final solution.
///
/// The Java `EquationSystemSolver.EquationResidual` (`equation`, `residual`),
/// plus the index of the Tarjan block the equation was solved in, which the
/// Java side reconstructs from `Block.equations` when it needs it
/// (`failedBlockIndex` diagnostics in `api.ts`).
#[derive(Debug, Clone, PartialEq)]
pub struct EquationResidual {
    /// The equation quoted verbatim (`Equation.source_text` — the Java
    /// `Equation::sourceText`), never a mangled internal form.
    pub equation: String,
    /// `lhs - rhs` evaluated at the returned values. `NaN` when the equation
    /// cannot be evaluated there (Java catches and records `NaN` the same way
    /// in `enrichWithPartialResult`).
    pub residual: f64,
    /// 0-based index into [`Solution::blocks`] of the block that solved it.
    pub block: usize,
}

/// Solve-effort statistics — the subset of the Java `EquationSystemSolver.Stats`
/// the core can know. `equationCount`/`unknownCount`/`blockCount` are derivable
/// from the [`Solution`] itself (`residuals.len()`, `values.len()`,
/// `blocks.len()`), so they are not duplicated here.
#[derive(Debug, Clone, PartialEq)]
pub struct SolveStats {
    /// Total Newton iterations across every block (Java `Stats.iterations`).
    pub iterations: usize,
    /// Largest `|residual|` across the system at the returned solution
    /// (Java `Stats.maxResidual`).
    pub max_residual: f64,
    /// Wall-clock solve time (Java `Stats.elapsedMillis`). Always `None` in
    /// core: `wasm32-unknown-unknown` has no clock, so the boundary stage
    /// measures and fills this where a clock exists.
    pub elapsed_ms: Option<f64>,
}

/// A completed steady solve.
#[derive(Debug, Clone, PartialEq)]
pub struct Solution {
    /// Every **unknown** in the system with its solved value, keyed by the
    /// lowercase canonical name (frees identifiers are case-insensitive).
    ///
    /// Built-in `#` constants are deliberately *not* listed: the Java engine
    /// substitutes them as numeric literals at parse time, so they are never
    /// result rows (`fixtures/golden/constants.json` lists only `a`, `b`, `c`).
    pub values: BTreeMap<String, f64>,
    /// Lowercase canonical name → the spelling of its **first appearance** in
    /// the source, sigil suffixes kept as-is (the Java
    /// `ParseResult.displayNames`): `t_out` → `"T_out"`. Covers exactly the
    /// unknowns in [`Solution::values`].
    pub display_names: BTreeMap<String, String>,
    /// The blocks, in the order they were solved.
    pub blocks: Vec<Block>,
    /// Per block (aligned with [`Solution::blocks`]), the `source_text` of its
    /// equations, first occurrence of a repeated text kept — the Java
    /// `SolveDtos.toBlockDto` applies `.distinct()` so an expanded component
    /// never lists one user line twice.
    pub block_equations: Vec<Vec<String>>,
    /// `lhs - rhs` of every equation, in source order, evaluated at the final
    /// values (the Java `Result.residuals`).
    pub residuals: Vec<EquationResidual>,
    /// Effort statistics for the whole solve.
    pub stats: SolveStats,
    /// Lowercase variable name → display unit string: declared units
    /// (annotated literals, then external `VariableInfo` units on top) plus the
    /// units [`crate::units::checker`] derived dimensionally — the Java
    /// `SolverApiSupport.unitsByLowerName` composition, which fills the
    /// `variables[].units` column of the solve response.
    pub inferred_units: BTreeMap<String, String>,
    /// Unit-consistency warning sentences from [`crate::units::checker`], in
    /// Java emission order (`unitWarnings[]` in `api.ts`). Unit problems are
    /// warnings by the parent engine's invariant — they never block a solve.
    pub unit_warnings: Vec<String>,
    /// Parser diagnostics plus anything the solve wanted to say. Unit and
    /// bounds problems are warnings and never block a solve.
    pub diagnostics: Vec<Diagnostic>,
    /// Total Newton iterations across every block (same value as
    /// `stats.iterations`, kept for existing callers).
    pub iterations: usize,
    /// The document's **top-level** `COMPONENT` instantiations, in declaration
    /// order — the datasheet's input half.
    ///
    /// The Java `SolveController` re-parses `cleanText` and calls
    /// `ComponentMetadata.build(cleanText, variableDtos)`, which reads
    /// `program.componentInsts()`: the *unflattened*, top-level list. This
    /// carries the same list forward off the one parse instead, so the boundary
    /// does not re-lex the document.
    ///
    /// Empty for a document with no component layer.
    pub component_instances: Vec<crate::components::metadata::ComponentInstMeta>,
    /// The connection topology of the expanded network — the schematic's data
    /// layer (the Java `ParseResult.componentConnections`). Empty for a document
    /// with no component layer.
    pub component_connections: Vec<crate::components::expander::Connection>,
    /// One sampled trajectory per `DYNAMIC` block, in declaration order — the
    /// Java `Result.odeTables`, which the frontend renders as the Tables and
    /// Plots windows.
    ///
    /// **A solved `DYNAMIC` block contributes nothing to
    /// [`Solution::values`].** The trajectory is a first-class table, not a set
    /// of scalars, so a transient document's `variables` map holds only its
    /// analytic parameters. A parity fixture that compares `variables` alone
    /// therefore passes *vacuously* on a transient document — `tests/parity.rs`
    /// compares this field for exactly that reason.
    pub ode_tables: Vec<crate::ode::problem::OdeTableResult>,
    /// Propagated 1-sigma uncertainty of **every** variable of the system — the
    /// Java `Result.uncertainties`. A declared source maps to its stated value,
    /// a dependent variable to its propagated sigma, and a document that
    /// declares no source at all maps everything to `0.0`.
    pub uncertainties: BTreeMap<String, f64>,
    /// Per dependent variable, each source's signed first-order contribution,
    /// ranked — the Java `Result.uncertaintyContributions`, which the frontend
    /// renders as the tornado chart. Empty when nothing declares an
    /// uncertainty.
    pub uncertainty_contributions:
        BTreeMap<String, Vec<crate::analysis::uncertainty::UncertaintyContribution>>,
    /// The `PLOT '…' … END` blocks the document declares, in declaration
    /// order — the Java `SolveController`'s `plotsOf(parsed.plots())`.
    ///
    /// A plot never enters the equation system; it rides out on the result
    /// only so the frontend can render it. Carried forward off the one parse
    /// rather than re-parsed at the boundary (the Java re-parses `cleanText`).
    pub plots: Vec<crate::parser::blocks::PlotDef>,
}

/// Diagnostics captured at the point a block solve gave up — the Java
/// `SolverException.partialResult` built by `enrichWithPartialResult`: the
/// block structure, every equation's residual evaluated at the stalled
/// iterate (`NaN` where an unsolved upstream block leaves nothing to
/// evaluate), and partial stats. Deliberately carries **no solved values** —
/// the Java partial `Result` ships `Map.of()` for `variables`.
#[derive(Debug, Clone, PartialEq)]
pub struct PartialDiagnostics {
    /// The full block decomposition (known before the first block ran).
    pub blocks: Vec<Block>,
    /// Per block, the `source_text` of its equations (see
    /// [`Solution::block_equations`]).
    pub block_equations: Vec<Vec<String>>,
    /// Lowercase canonical name → first-seen source spelling. This is the whole
    /// `ParseResult.displayNames` map the Java partial `Result` carries, so it
    /// may name more identifiers than the system has unknowns — use
    /// [`PartialDiagnostics::unknown_count`] for the count.
    pub display_names: BTreeMap<String, String>,
    /// How many unknowns the stalled system had (the Java partial
    /// `Stats.variableCount`, i.e. `surfacedVarCount`).
    pub unknown_count: usize,
    /// Every equation's `lhs - rhs` at the stalled iterate, source order.
    pub residuals: Vec<EquationResidual>,
    /// Iterations spent before the stall + the largest finite `|residual|`.
    pub stats: SolveStats,
}

/// A failed solve: what went wrong, plus the structured diagnostics the Java
/// engine carries on `SolverException` (`FailureState.failedBlockIndex` +
/// `partialResult`). Pre-block failures (parse errors, structural rejections)
/// have `failed_block_index: None` and `partial: None`, exactly as Java
/// failures without a `FailureState` pass through unenriched.
#[derive(Debug, Clone, PartialEq)]
pub struct SolveFailure {
    /// The underlying error (its message already carries the block annotation
    /// where one applies).
    pub error: FreesError,
    /// 0-based index of the Tarjan block whose solve gave up.
    pub failed_block_index: Option<usize>,
    /// Boxed to keep the `Err` variant small on every `solve` return path
    /// (`clippy::result_large_err` — the payload is ~180 bytes inline).
    pub partial: Option<Box<PartialDiagnostics>>,
}

impl SolveFailure {
    /// The message without the `Display` kind prefix (delegates to
    /// [`FreesError::to_string_message`]).
    pub fn to_string_message(&self) -> String {
        self.error.to_string_message()
    }

    /// Source span of the underlying error, where one is known.
    pub fn span(&self) -> Option<crate::diag::Span> {
        self.error.span()
    }
}

impl std::fmt::Display for SolveFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.error.fmt(f)
    }
}

impl std::error::Error for SolveFailure {}

impl From<FreesError> for SolveFailure {
    fn from(error: FreesError) -> SolveFailure {
        SolveFailure {
            error,
            failed_block_index: None,
            partial: None,
        }
    }
}

/// Lets existing assertions compare a failure against a bare [`FreesError`]
/// (`assert_eq!(solve(..).unwrap_err(), FreesError::solver("…"))`).
impl PartialEq<FreesError> for SolveFailure {
    fn eq(&self, other: &FreesError) -> bool {
        self.error == *other
    }
}

/// One syntax error with its 1-based editor position, so the lint gutter can
/// mark the broken line. The Java `SolveDtos.SyntaxErrorDto`.
#[derive(Debug, Clone, PartialEq)]
pub struct SyntaxErrorInfo {
    /// 1-based line of the error.
    pub line: usize,
    /// 1-based column of the error.
    pub column: usize,
    pub message: String,
}

/// The result of a check-before-solve.
///
/// Field-for-field the Java `EquationSystemSolver.CheckResult`
/// (`solvable`, `equationCount`, `unknownCount`, `variables`, `message`), plus
/// the parser's diagnostics, which the Java side carries on a separate field of
/// the HTTP DTO.
#[derive(Debug, Clone, PartialEq)]
pub struct CheckReport {
    /// True when the system is structurally solvable: zero degrees of freedom
    /// and a complete equation↔variable matching.
    pub solvable: bool,
    pub equation_count: usize,
    pub unknown_count: usize,
    /// The unknowns, sorted, lowercase.
    pub variables: Vec<String>,
    /// Lowercase canonical name → first-seen source spelling, covering exactly
    /// [`CheckReport::variables`]. See [`Solution::display_names`].
    pub display_names: BTreeMap<String, String>,
    /// Human-readable summary — the "No syntax errors were detected…" sentence
    /// on success, the blocker's causality diagnosis on a structural failure,
    /// or `"Syntax error: …"` on a parse failure (the Java `CheckController`
    /// 400-with-body message).
    pub message: String,
    /// 1-based editor line of the syntax error this report describes, `None`
    /// for whole-system problems (the `errorLine` field `api.ts` parses).
    pub error_line: Option<usize>,
    /// Every syntax error with its position. Empty unless the parse failed.
    /// (The Rust parser stops at the first error, so at most one entry today;
    /// the Java engine collects up to eight.)
    pub errors: Vec<SyntaxErrorInfo>,
    /// Lowercase variable name → inferred display unit (`inferredUnits` in
    /// `api.ts`): units [`crate::units::checker`] derived dimensionally,
    /// overlaid by units read off annotated literal assignments — the Java
    /// `CheckController` composition (`deriveUnits` then `putAll(inferUnits)`),
    /// which deliberately leaves external `VariableInfo` units out.
    pub inferred_units: BTreeMap<String, String>,
    /// Unit-consistency warning sentences from [`crate::units::checker`], in
    /// Java emission order (`unitWarnings[]` in `api.ts`). Never affects
    /// [`CheckReport::solvable`].
    pub unit_warnings: Vec<String>,
    pub diagnostics: Vec<Diagnostic>,
    /// The `PLOT '…' … END` blocks the document declares — the Java
    /// `CheckController`'s `plotsOf(parsed.plots())`. Check reports them too so
    /// the Plots tab populates before the first solve, which is what the
    /// frontend's `result?.definedPlots ?? checkResult?.definedPlots` fallback
    /// is for.
    pub plots: Vec<crate::parser::blocks::PlotDef>,
    /// Connection topology for the schematic — the same payload Solve reports,
    /// so Check can draw and validate wiring without a solve.
    pub connections: Vec<crate::components::expander::Connection>,
    /// Top-level and nested instance identities with source lines.
    pub instances: Vec<InstanceIdentity>,
    /// User `COMPONENT` definitions (local) with source lines.
    pub definitions: Vec<DefinitionIdentity>,
}

/// One component instance as Check reports it for navigation and completion.
#[derive(Debug, Clone, PartialEq)]
pub struct InstanceIdentity {
    pub name: String,
    pub label: String,
    pub type_name: String,
    pub line: usize,
    pub local_type: bool,
}

/// A user-authored `COMPONENT` definition.
#[derive(Debug, Clone, PartialEq)]
pub struct DefinitionIdentity {
    pub name: String,
    pub line: usize,
}

/// Externally supplied per-variable solver information — one row of the
/// Variable Information window, the Java `SolverApiSupport.VariableInfoDto`
/// (`name`, `guess`, `lower`, `upper`, `units`; `uncertainty` is not ported).
///
/// Values are expressed in `unit` when one is given and are converted to SI
/// (`value * factor + offset`) before the solver sees them, exactly as
/// `VariableInfoDto.toSpec` does.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct VariableOverride {
    /// Variable name, any case (lowercased on use).
    pub name: String,
    pub guess: Option<f64>,
    pub lower: Option<f64>,
    pub upper: Option<f64>,
    /// Display unit the numbers above are written in. `None`, `""` and `"-"`
    /// all mean "already SI"; an *unknown* unit falls back to factor 1 /
    /// offset 0 silently, matching the Java `toSpec` catch-and-default.
    pub unit: Option<String>,
    /// Stated 1-sigma uncertainty — the Variable Information window's `±`
    /// column (`VariableInfoDto.uncertainty`). Scales by the unit's **factor
    /// only**, never its offset: it is an interval width, not a point. `None`
    /// is the Java's null, which `toSpec` turns into `0.0` — "not a source".
    pub uncertainty: Option<f64>,
}

/// What the document says about one unknown before the solve starts.
///
/// The Java `VariableSpec` carries more (display name, unit, format); only the
/// three fields the solver reads are needed here.
#[derive(Debug, Clone, Copy, PartialEq)]
struct VarSpec {
    guess: f64,
    lower: f64,
    upper: f64,
}

impl Default for VarSpec {
    fn default() -> VarSpec {
        VarSpec {
            guess: DEFAULT_GUESS,
            lower: f64::NEG_INFINITY,
            upper: f64::INFINITY,
        }
    }
}

impl VarSpec {
    /// The starting value: the guess clamped into the declared bounds, as
    /// `withTextGuesses` does with `Math.clamp`.
    fn initial(&self) -> f64 {
        self.guess.clamp(self.lower, self.upper)
    }

    fn is_bounded(&self) -> bool {
        self.lower.is_finite() || self.upper.is_finite()
    }
}

// ---------------------------------------------------------------------------
// Public entry points
// ---------------------------------------------------------------------------

/// Parse, block and solve `source`. Equivalent to [`solve_with`] with no
/// external variable information.
///
/// # Errors
///
/// * [`FreesError::Parse`] — a syntax error, or a block construct the wasm port
///   has not implemented (`DYNAMIC`, `LINEARIZE`; see
///   [`crate::parser::toplevel`] for what has since left that list).
/// * [`FreesError::Solver`] — no equations, nonzero degrees of freedom, a
///   structurally singular system, or a block that would not converge.
/// * whatever [`crate::eval`] raises when the residuals cannot be evaluated at
///   the initial point (unknown function, wrong arity, a string in a numeric
///   position).
pub fn solve(
    source: &str,
    settings: &SolverSettings,
) -> std::result::Result<Solution, SolveFailure> {
    solve_with(source, settings, &[])
}

/// [`solve`] with externally supplied per-variable guesses/bounds — the shape
/// `POST /api/solve` receives as `variableInfo`.
///
/// # How overrides and in-text `GUESS` directives merge
///
/// The rule is the Java one, verified against
/// `EquationSystemSolver.withTextGuesses` and `VariableInfoDto.toSpec`:
///
/// 1. Each override converts to a spec first (values pass through its `unit`
///    to SI). An override with bounds but no guess starts from `DEFAULT_GUESS`
///    clamped into those bounds.
/// 2. In-text `GUESS` directives then merge **over** the overrides, field by
///    field: a part the directive states wins; a part it omits falls back to
///    the override ("text wins, so a shared document solves identically for
///    its recipient").
/// 3. The merged guess is clamped into the merged bounds — a stale external
///    guess landing outside text bounds is pulled onto them (the bounds win).
///
/// # Errors
///
/// Everything [`solve`] raises, plus [`FreesError::Solver`] for an invalid
/// override (NaN, crossed bounds, or an explicit guess outside its own
/// bounds), mirroring the Java `VariableSpec` constructor's rejections.
pub fn solve_with(
    source: &str,
    settings: &SolverSettings,
    overrides: &[VariableOverride],
) -> std::result::Result<Solution, SolveFailure> {
    solve_with_parametric_tables(source, settings, overrides, None, &[])
}

/// [`solve_with`] with externally supplied Function Table definitions — the
/// request's `functionTables`, which the Java threads through every solving
/// endpoint as `SolveDtos.functionDefsOf(request.functionTables())` into
/// `EquationSystemSolver.solve(text, settings, specs, extraDefs)`.
///
/// The merge is [`crate::parser::defs::Definitions::merge_extra_tables`] —
/// the Java `withExtraDefs` rule (**source definitions win** on a name
/// collision) — applied at the Java position: *after* the parse/expand
/// pipeline (`parseResult` in the Java runs component expansion and CALL
/// flattening before the extras exist), *before* residual evaluation, the
/// Integral pass and the transient path, all of which see the merged map.
/// `extra_tables` must carry lowercase canonical names, as the boundary's
/// DTO conversion produces. An empty slice is byte-for-byte [`solve_with`].
pub fn solve_with_tables(
    source: &str,
    settings: &SolverSettings,
    overrides: &[VariableOverride],
    extra_tables: &[crate::parser::defs::FunctionTableDef],
) -> std::result::Result<Solution, SolveFailure> {
    solve_with_parametric_tables(source, settings, overrides, None, extra_tables)
}

/// [`solve_with`] with the parametric-accessor channel installed — the
/// per-row solve of a Tables-workbook sweep (`analysis::parametric::run_sweep`
/// drives it through the wasm boundary's `solve_table`). The accessors are
/// what `TableValue`/`TableAvg`/`IntegralValue`/… read; a plain solve passes
/// `None` and every accessor reports its empty-context default, exactly as
/// the Java's thread-bound `ParametricAccessors` behaves outside
/// `solveTableWithAccessors`.
pub fn solve_with_parametric(
    source: &str,
    settings: &SolverSettings,
    overrides: &[VariableOverride],
    parametric: Option<&crate::analysis::parametric::ParametricAccessors>,
) -> std::result::Result<Solution, SolveFailure> {
    solve_with_parametric_tables(source, settings, overrides, parametric, &[])
}

/// [`solve_with_parametric`] plus the [`solve_with_tables`] channel — the
/// per-row solve of a Tables-workbook sweep whose request carried GUI
/// Function Tables (the Java `SolveController.computeSolveTable` threads
/// `TableRowContext.functionDefs` into every row's solve, chunked re-dispatch
/// included).
pub fn solve_with_parametric_tables(
    source: &str,
    settings: &SolverSettings,
    overrides: &[VariableOverride],
    parametric: Option<&crate::analysis::parametric::ParametricAccessors>,
    extra_tables: &[crate::parser::defs::FunctionTableDef],
) -> std::result::Result<Solution, SolveFailure> {
    let mut prepared = PreparedDocument::new(source, settings, overrides, extra_tables)?;
    prepared.solve_with_pins(&[], parametric)
}

/// Bounded multiple-solution search (up to 32 solutions) using interval scanning and multi-start Newton.
pub fn solve_all(
    source: &str,
    settings: &SolverSettings,
) -> std::result::Result<Vec<Solution>, SolveFailure> {
    solve_all_with_tables(source, settings, &[], &[])
}

/// [`solve_all`] with externally supplied per-variable guesses/bounds.
pub fn solve_all_with(
    source: &str,
    settings: &SolverSettings,
    overrides: &[VariableOverride],
) -> std::result::Result<Vec<Solution>, SolveFailure> {
    solve_all_with_tables(source, settings, overrides, &[])
}

/// [`solve_all_with`] with externally supplied Function Table definitions.
pub fn solve_all_with_tables(
    source: &str,
    settings: &SolverSettings,
    overrides: &[VariableOverride],
    extra_tables: &[crate::parser::defs::FunctionTableDef],
) -> std::result::Result<Vec<Solution>, SolveFailure> {
    let mut prepared = PreparedDocument::new(source, settings, overrides, extra_tables)?;
    prepared.solve_all_with_pins(&[], None)
}

// ---------------------------------------------------------------------------
// The transient pass — port of EquationSystemSolver's DYNAMIC wiring
// ---------------------------------------------------------------------------

/// Looser tolerances for ODE-coupled solves.
///
/// Port of `EquationSystemSolver.relaxedOdeSettings`. The per-step algebraic
/// block and the analytic solve of an accessor constraint both sit on a
/// finite-difference / integration noise floor, so the default residual target
/// is physically unreachable; the loosened target is still far tighter than any
/// engineering tolerance.
/// The Java record is `(maxIterations, relTol, changeInVariables,
/// elapsedTimeSeconds, complexMode)` and the call is
/// `new SolverSettings(base.maxIterations(), relTol,
/// Math.max(base.changeInVariables(), 1e-9), …)`. This port has no
/// `changeInVariables` knob (its stop rule is the residual only) and no clock,
/// so the transcription is the one field that exists here: `rel_tolerance`.
/// `abs_tolerance` is left alone — it is already inert below the relaxed target.
fn relaxed_ode_settings(base: &SolverSettings, rel_tol: f64) -> SolverSettings {
    SolverSettings {
        rel_tolerance: rel_tol,
        ..*base
    }
}

/// Run every `DYNAMIC` block with the solved scalars as its base values, and
/// collect one ODE Table per block.
///
/// Port of `EquationSystemSolver.solveDynamicSystems`. The per-step algebraic
/// coupling reuses [`solve_pinned`] (states + time pinned), so the transient
/// path shares the analytic solver's Newton/Tarjan machinery rather than owning
/// a second one — and the inner solve runs at the relaxed `1e-7` tolerance for
/// the reason [`relaxed_ode_settings`] states.
fn solve_dynamic_systems(
    doc: &Document,
    base_values: &Scope,
    settings: &SolverSettings,
    specs: &BTreeMap<String, VarSpec>,
    ctx: EvalContext<'_>,
    reuse: Option<&crate::ode::accessors::DynamicAccessorContext<'_>>,
) -> Result<Vec<crate::ode::problem::OdeTableResult>> {
    if doc.dynamics.is_empty() {
        return Ok(Vec::new());
    }
    let inner = relaxed_ode_settings(settings, 1e-7);
    let mut tables = Vec::with_capacity(doc.dynamics.len());
    for (index, system) in doc.dynamics.iter().enumerate() {
        // A live accessor has already integrated this system at every Newton
        // iterate it was asked about, the last of which is the converged point
        // this pass is about to integrate at again. When the input signature
        // still matches, that table *is* this table — see
        // `DynamicAccessorContext::cached_table` for why the match is
        // sufficient. Without this, every accessor-bearing document integrates
        // its transient exactly twice.
        if let Some(table) = reuse.and_then(|bridge| bridge.cached_table(index, base_values)) {
            tables.push(table);
            continue;
        }
        let algebraic = pinned_solver(&inner, specs, ctx);
        tables.push(
            crate::ode::dynamic::DynamicSolver::new(system, base_values, &doc.defs, algebraic)
                .solve()?,
        );
    }
    Ok(tables)
}

/// The `AlgebraicSolve` the transient path hands to [`crate::ode::dynamic`]:
/// the Java's `(ordinary, pinned, warmStart) -> solvePinned(...).values()`.
///
/// Unlike the Java (whose JIT makes the per-call structural work cheap), this
/// implementation **caches the structural half** of `solve_pinned` across
/// calls: the subsystem build, the Tarjan blocking, the spec-map clone with
/// its property-argument seeding, and the unknown list depend only on the
/// equations and the *names* being pinned — all invariant across the
/// thousands of per-step solves one integration makes — while the pin
/// *values* change every step. Measured on `dyn_accessor_live`'s cooling
/// block: 371 µs/step → the Newton solve alone.
///
/// Soundness: the cache key is a **full structural equality** check of the
/// ordinary equations plus the pinned-name list on every call (no pointer
/// identity games), and everything cached is value-independent — blocking is
/// syntactic, `builtin_constants` reads names, and the seeding passes react
/// to property-call shapes, never to pin values. The pin equations' value and
/// source text (which the failure annotator quotes) are rewritten in place
/// each call, so residuals and diagnostics are byte-identical to the
/// uncached path.
fn pinned_solver<'a>(
    settings: &'a SolverSettings,
    specs: &'a BTreeMap<String, VarSpec>,
    ctx: EvalContext<'a>,
) -> Box<dyn crate::ode::dynamic::AlgebraicSolve + 'a> {
    Box::new(PreparedPinnedSolver {
        settings,
        specs,
        ctx,
        prep: None,
        watch: Vec::new(),
        pending: false,
    })
}

/// See [`pinned_solver`].
struct PreparedPinnedSolver<'a> {
    settings: &'a SolverSettings,
    specs: &'a BTreeMap<String, VarSpec>,
    ctx: EvalContext<'a>,
    prep: Option<PinnedPrep>,
    /// Wave R3: the names the caller reads out of every solve
    /// ([`crate::ode::dynamic::AlgebraicSolve::watch`]). Non-empty means the
    /// caller has taken on the deferred contract, so a dense solve may leave
    /// the map unbuilt; the dense slots behind them are resolved per prep, in
    /// [`PinnedPrep::watch_slots`].
    watch: Vec<String>,
    /// A solve's result is in `dense_values` and has not been written into the
    /// caller's buffer. Only ever true under the dense path — the string-keyed
    /// path publishes as it always did, so nothing downstream of it can read a
    /// stale map.
    pending: bool,
}

/// The structural half of one `solve_pinned` call, reusable while the
/// equations and pinned names stay the same.
struct PinnedPrep {
    /// The ordinary equations this prep was built from (full-equality key).
    template: Vec<Equation>,
    /// The pinned names, in call order (the other half of the key).
    pinned_names: Vec<String>,
    /// `template` + one `name = value` equation per pin, pins last, in the
    /// pinned order — the values are rewritten in place per call.
    subsystem: Vec<Equation>,
    report: BlockingReport,
    /// `specs` cloned and property-argument seeded (`Missing::Create`).
    seeded: BTreeMap<String, VarSpec>,
    unknowns: Vec<String>,
    /// Wave G3: the per-block value-independent pieces of [`solve_block`].
    block_cache: PinnedBlockCache,
    /// Wave I: the value map `run_blocks` solves in, kept across calls
    /// instead of being rebuilt from string-keyed inserts per step. The name
    /// set is invariant per prep — the built-in constants and the unknowns
    /// are fixed at build, and the uncertainty entries carried from the warm
    /// start only ever re-add the warm chain's own keys (the warm start is
    /// always a previous result of this same solver, so its key set is the
    /// prep's own) — while the *values* rewrite per call: every unknown slot
    /// is overwritten from the warm start or the initial guess before the
    /// solve, constants never change, and `run_blocks` writes only block
    /// variables (all unknowns). A failed call can leave stale iterate
    /// values behind; the next call's full unknown rewrite erases them.
    /// The caller receives a clone, which is the call's single full map
    /// materialisation — it used to be two (this build plus the warm-start
    /// clone in `solve_algebraic_at`).
    work_scope: Scope,
    /// Wave R3: the dense mirror of [`Self::work_scope`]'s key set, when every
    /// block of this prep compiled ([`dense_plan`]). `None` means this prep
    /// runs the string-keyed path exactly as it did before, with no dense
    /// bookkeeping anywhere.
    dense: Option<DensePlan>,
    /// The last **successful** dense state. Under the deferred contract this
    /// is the warm chain itself — the caller's buffer is no longer read — so
    /// it is written only on success, and only when the caller is running a
    /// chain at all (a `warm` of `None` is the linearizer asking for a
    /// one-shot solve, which must not disturb a trajectory's warm start; it
    /// did not disturb it before this wave either, because that path passes a
    /// local buffer).
    dense_last: Vec<f64>,
    /// [`PreparedPinnedSolver::watch`] resolved against this prep's layout.
    /// `None` = one of the names is not a `work_scope` key, so the caller must
    /// keep reading the map.
    watch_slots: Option<Vec<u32>>,
    /// The values behind [`Self::dense`], indexed by dense slot. Constants are
    /// written once at build and never move (a block variable is never a
    /// known); every unknown is overwritten at the top of each call, from the
    /// warm start or the initial guess, exactly as the `work_scope` fill does.
    dense_values: Vec<f64>,
}

/// Wave G3: what [`solve_block`] rebuilds on every call but never needs to —
/// hoisted per block, the way [`PinnedPrep`] hoisted the subsystem build.
///
/// Soundness follows the same rule as the rest of the prep: everything here is
/// a pure function of the equations' *structure*, the block's variable list
/// and the seeded specs. The pin values rewritten between calls never reach
/// any of it — a pin equation's residual is `var − c`, and `c` only ever
/// enters its derivative as `d(c)/d(var) = 0`, so the cached trees are
/// byte-identical to what per-call differentiation would rebuild. Measured on
/// the callgrind profile that motivated this (`hot-transient`, 2 160-step
/// ode45 cooling block): symbolic differentiation, `Equation::variables` and
/// the resulting `Expr` drops were ~24 % of the whole run, all of it repeated
/// per RHS evaluation.
struct PinnedBlockCache {
    /// Indexed exactly like the report's `blocks`; the merge rescue's
    /// synthetic blocks are not here and compute fresh, as before.
    per_block: Vec<PinnedBlockStruct>,
}

impl PinnedBlockCache {
    #[allow(dead_code)]
    pub(crate) fn invalidate_bounds(&self) {
        for b in &self.per_block {
            b.scratch.borrow_mut().ws.invalidate_bounds();
        }
    }
}

/// See [`PinnedBlockCache`].
struct PinnedBlockStruct {
    /// The Java IterationContext lo/hi arrays for this block's variables.
    bounds: Vec<(f64, f64)>,
    /// [`analytic_derivatives`] for this block, computed once. An outer `None`
    /// is itself a cached answer ("no analytic Jacobian — finite
    /// differences"), not a cache miss.
    derivs: Option<Vec<Vec<Option<Expr>>>>,
    /// [`NewtonProblem::row_dependencies`] for this block, computed once.
    row_deps: Vec<Vec<usize>>,
    /// Wave A2: this block's residuals and derivatives compiled to
    /// slot-indexed postfix, when every one of them is arithmetic — see
    /// [`crate::solver::slots`] for why that removes the string-keyed `Scope`
    /// from the whole Newton solve, and why it is byte-identical. `None` is a
    /// cached answer ("this block needs the `Expr`/`Scope` evaluator"), not a
    /// cache miss.
    compiled: Option<CompiledBlock>,
    /// Wave R3: where this block's compiled names and unknowns live in the
    /// prep's dense vector — the same two lists [`CompiledBlock::names`] and
    /// `block.variables` name, resolved to indices once. `None` whenever the
    /// prep has no dense plan (see [`dense_plan`]).
    dense: Option<BlockDense>,
    /// Wave I: this block's cross-call solver buffers — [`solve_block`]'s own
    /// per-call vectors plus the Newton workspace. Value-independent by
    /// construction: every buffer is fully rewritten before it is read (see
    /// [`crate::solver::newton::newton_solve_problem_with`]), so nothing can
    /// leak from one per-step solve into the next.
    ///
    /// `RefCell` because the cache is threaded through the retry ladder as a
    /// shared borrow. That is sound here: the per-step path is
    /// single-threaded, and the ladder borrows the cell for one
    /// `solve_block` call at a time — never reentrantly, because a nested
    /// accessor solve builds its own prep with its own cells
    /// ([`accessor_bridge`] constructs a fresh `pinned_solver` per run).
    scratch: RefCell<BlockScratch>,
}

/// See [`PinnedBlockStruct::scratch`].
#[derive(Default)]
struct BlockScratch {
    ws: NewtonWorkspace,
    /// The Newton iterate (`solve_block`'s `x`).
    x: Vec<f64>,
    /// The initial-point probe residuals (write-only sink).
    probe: Vec<f64>,
    /// Wave A2, compiled path only: this block's slot vector, refilled from
    /// the `Scope` at the top of every call.
    slots: Vec<f64>,
    /// Wave A2, compiled path only: the residual literals, refilled per call
    /// by [`crate::solver::slots::refresh_consts`] because the pin equations'
    /// values are rewritten between calls.
    consts: Vec<f64>,
    /// Wave A2, compiled path only: the postfix evaluation stack, cleared by
    /// every [`crate::solver::slots::eval_program`].
    stack: Vec<f64>,
}

/// Wave R3: the prep's `work_scope` key set, resolved to dense indices once.
///
/// Wave A2 made the *inside* of a block's Newton solve slot-indexed and left a
/// measured floor behind it: the compiled path still paid two string-keyed
/// passes per `solve_block` call — filling the slot vector out of the `Scope`
/// and writing the answer back into it — plus the per-call seeding of the map
/// itself. On the `hot-transient` stand-in that floor was 30 hashed reads and
/// 32 hashed writes per integrator step, ~14 % of the whole run, and A2 and Q3
/// both recorded it as "needs the `work_scope` to become slot-native".
///
/// This is that change, and it does **not** require the `Scope` to stop being
/// a `Scope`. The prep's key set is fixed at build (built-in constants, then
/// the subsystem's unknowns), so every name the per-step path touches can be
/// resolved to an index once, here; the per-step call then runs entirely on
/// `dense_values` and materialises the string-keyed map **once, at the end**,
/// for the caller — which is the same single materialisation Wave Q3's
/// in-place refresh already performed. The `&Scope` the caller indexes by name
/// is therefore unchanged, in content *and* in layout.
struct DensePlan {
    /// Dense index → name, in `work_scope`'s own iteration order. That order
    /// is what makes the end-of-call export the same *sequence* of writes the
    /// Q3 refresh made, not merely the same set.
    names: Vec<String>,
    /// Dense index of each entry of [`PinnedPrep::unknowns`], in that order.
    unknown_idx: Vec<u32>,
}

/// See [`PinnedBlockStruct::dense`].
struct BlockDense {
    /// Dense index of each [`CompiledBlock::names`] entry, in slot order.
    fill: Vec<u32>,
    /// Dense index of each `block.variables` entry, in block-variable order.
    write: Vec<u32>,
}

/// One per-step call's dense state, threaded down the block loop.
///
/// `live` is the authority flag, and it is what keeps this byte-identical: the
/// dense vector answers for the values only while every block has been able to
/// use it. The moment one cannot — a block that did not compile, a
/// `refresh_consts` refusal, a block that failed and must go through the retry
/// ladder — [`DenseRun::hand_over`] copies the dense values into the `Scope`,
/// which is exactly the state the string-keyed path would have been in at that
/// point, and the rest of the call proceeds on the `Scope` as before.
struct DenseRun<'a> {
    plan: &'a DensePlan,
    values: &'a mut Vec<f64>,
    live: bool,
}

impl DenseRun<'_> {
    /// Give the `Scope` the values and the authority. Idempotent by
    /// construction, and only ever called while `live`.
    fn hand_over(&mut self, scope: &mut Scope) {
        dense_export(self.plan, self.values, scope);
        self.live = false;
    }
}

/// Copy every dense value into `scope` under its own name.
///
/// The `insert` arm cannot fire from a plan built against this very `scope`;
/// it is there because [`DensePlan`] outlives one call and a `Scope` the retry
/// ladder has been through may have lost nothing but is not *proven* to have.
fn dense_export(plan: &DensePlan, values: &[f64], scope: &mut Scope) {
    for (name, value) in plan.names.iter().zip(values) {
        match scope.get_mut(name) {
            Some(slot) => *slot = *value,
            None => {
                scope.insert(name.clone(), *value);
            }
        }
    }
}

/// Resolve a prep's `work_scope` to [`DensePlan`] and fill in each block's
/// [`BlockDense`], or answer `None` — which leaves the prep on the Wave-Q3
/// path with no dense bookkeeping at all.
///
/// The gate is deliberately all-or-nothing: **every** block with unknowns must
/// have compiled, and every name it mentions must be a `work_scope` key. A
/// prep with one uncompiled block would otherwise pay a full `hand_over` per
/// step to reach the block that cannot use the dense vector, which is a
/// regression for documents that were never going to benefit — property-bearing
/// blocks refuse to compile at all (Wave A2), and those are the documents
/// where the property backend, not the map, is the cost.
///
/// A missing name cannot merely be skipped: `fill_slots` answers `false` for
/// one and the block falls back, so a plan that pretended otherwise would
/// evaluate against a slot that was never filled.
fn dense_plan(
    blocks: &[Block],
    cache: &mut PinnedBlockCache,
    work_scope: &Scope,
    unknowns: &[String],
) -> Option<(DensePlan, Vec<f64>)> {
    let mut names: Vec<String> = Vec::with_capacity(work_scope.len());
    let mut values: Vec<f64> = Vec::with_capacity(work_scope.len());
    let mut index: HashMap<&str, u32> = HashMap::with_capacity(work_scope.len());
    for (name, value) in work_scope.iter() {
        index.insert(name.as_str(), names.len() as u32);
        names.push(name.clone());
        values.push(*value);
    }
    let slot_of = |name: &str| index.get(name).copied();
    // Every unknown is a `work_scope` key by construction (the map is built
    // from the constants and this very list), so a miss here means the two
    // have drifted apart and the dense path must not be taken.
    let unknown_idx: Vec<u32> = unknowns.iter().map(|n| slot_of(n)).collect::<Option<_>>()?;

    let mut per_block: Vec<BlockDense> = Vec::with_capacity(blocks.len());
    for (block, cached) in blocks.iter().zip(&cache.per_block) {
        if block.variables.is_empty() {
            // `solve_block` returns before reading anything for these.
            per_block.push(BlockDense {
                fill: Vec::new(),
                write: Vec::new(),
            });
            continue;
        }
        let compiled = cached.compiled.as_ref()?;
        let fill: Option<Vec<u32>> = compiled.names.iter().map(|n| slot_of(n)).collect();
        let write: Option<Vec<u32>> = block.variables.iter().map(|n| slot_of(n)).collect();
        per_block.push(BlockDense {
            fill: fill?,
            write: write?,
        });
    }
    for (cached, dense) in cache.per_block.iter_mut().zip(per_block) {
        cached.dense = Some(dense);
    }
    Some((DensePlan { names, unknown_idx }, values))
}

/// Build [`PinnedBlockCache`] for a prep's blocking report — the same code
/// paths [`solve_block`] and `row_dependencies` run per call, executed once.
///
/// `template_len` is where the prep's *pin* equations start in `subsystem`.
/// Everything before it is template, which the prep's cache key re-checks for
/// full structural equality on every call; everything from it on is rewritten
/// in place per call. [`crate::solver::slots`] needs that split to know which
/// literals it may bake.
fn pinned_block_cache(
    blocks: &[Block],
    subsystem: &[Equation],
    template_len: usize,
    specs: &BTreeMap<String, VarSpec>,
) -> PinnedBlockCache {
    let per_block = blocks
        .iter()
        .map(|block| {
            let mut block_equations: Vec<&Equation> = Vec::with_capacity(block.equations.len());
            let mut volatile: Vec<bool> = Vec::with_capacity(block.equations.len());
            for &i in &block.equations {
                if let Some(equation) = subsystem.get(i) {
                    block_equations.push(equation);
                    volatile.push(i >= template_len);
                }
            }
            let bounds: Vec<(f64, f64)> = block
                .variables
                .iter()
                .map(|name| {
                    specs
                        .get(name)
                        .map(|spec| (spec.lower, spec.upper))
                        .unwrap_or((f64::NEG_INFINITY, f64::INFINITY))
                })
                .collect();
            let derivs = analytic_derivatives(&block_equations, &block.variables);
            let mentioned: Vec<BTreeSet<String>> =
                block_equations.iter().map(|e| e.variables()).collect();
            let row_deps: Vec<Vec<usize>> = block
                .variables
                .iter()
                .map(|name| {
                    mentioned
                        .iter()
                        .enumerate()
                        .filter(|(_, vars)| vars.contains(name))
                        .map(|(i, _)| i)
                        .collect()
                })
                .collect();
            let compiled = slotexpr::compile_block(
                &block_equations,
                &volatile,
                &block.variables,
                derivs.as_ref(),
            );
            PinnedBlockStruct {
                bounds,
                derivs,
                row_deps,
                compiled,
                dense: None,
                scratch: RefCell::new(BlockScratch::default()),
            }
        })
        .collect();
    PinnedBlockCache { per_block }
}

impl PreparedPinnedSolver<'_> {
    /// Write an outstanding dense result into the caller's buffer, restoring
    /// the undeferred contract — `warm` holds this prep's keys and values and
    /// nothing else. A no-op when nothing is outstanding.
    ///
    /// This is the *same* code the undeferred tail of `solve` runs, and it is
    /// the only thing the deferral moves: not what gets written, only when.
    fn publish(&mut self, warm: &mut Option<Scope>) {
        if !self.pending {
            return;
        }
        self.pending = false;
        let Some(prep) = self.prep.as_mut() else {
            return;
        };
        let Some(plan) = prep.dense.as_ref() else {
            return;
        };
        match warm.as_mut() {
            Some(previous) => {
                for (name, value) in plan.names.iter().zip(prep.dense_values.iter()) {
                    match previous.get_mut(name) {
                        Some(slot) => *slot = *value,
                        None => {
                            previous.insert(name.clone(), *value);
                        }
                    }
                }
                if previous.len() != plan.names.len() {
                    dense_export(plan, &prep.dense_values, &mut prep.work_scope);
                    *previous = prep.work_scope.clone();
                }
            }
            // The first call of a prep, and the rare buffer that came from a
            // different one: both hand back a *clone* of `work_scope`, so the
            // dense values have to reach it first.
            None => {
                dense_export(plan, &prep.dense_values, &mut prep.work_scope);
                *warm = Some(prep.work_scope.clone());
            }
        }
    }
}

impl crate::ode::dynamic::AlgebraicSolve for PreparedPinnedSolver<'_> {
    /// Accepted only before the first solve, which is where the caller offers
    /// it — there is then no warm chain in flight to reason about.
    fn watch(&mut self, names: &[String]) -> bool {
        if names.is_empty() || self.prep.is_some() {
            return false;
        }
        self.watch = names.to_vec();
        true
    }

    fn watched(&self, out: &mut [f64]) -> bool {
        // `pending` is the proof that the last solve ran dense and left its
        // answer in `dense_values`; without it the caller must read the map,
        // which is exactly what returning `false` makes it do.
        if !self.pending {
            return false;
        }
        let Some(prep) = self.prep.as_ref() else {
            return false;
        };
        let Some(slots) = prep.watch_slots.as_ref() else {
            return false;
        };
        if slots.len() != out.len() {
            return false;
        }
        for (value, &slot) in out.iter_mut().zip(slots) {
            *value = prep.dense_values[slot as usize];
        }
        true
    }

    fn materialise(&mut self, warm: &mut Option<Scope>) {
        self.publish(warm);
    }

    fn solve(
        &mut self,
        ordinary: &[Equation],
        pinned: &[(String, f64)],
        warm: &mut Option<Scope>,
    ) -> Result<()> {
        let matches = self.prep.as_ref().is_some_and(|p| {
            p.template.as_slice() == ordinary
                && p.pinned_names.len() == pinned.len()
                && p.pinned_names
                    .iter()
                    .zip(pinned)
                    .all(|(cached, (name, _))| cached == name)
        });
        // Wave R3: a deferred result is the warm chain, and it is only the
        // warm chain while the prep that produced it still stands and still
        // describes the caller's buffer. Anything else — a rebuilt prep, a
        // `work_scope` that has gained a key, a caller with no buffer at all —
        // and the buffer goes back to being the input it always was, which
        // means publishing into it first.
        let chain = matches
            && self.pending
            && warm.is_some()
            && self.prep.as_ref().is_some_and(|p| {
                p.dense
                    .as_ref()
                    .is_some_and(|plan| plan.names.len() == p.work_scope.len())
                    && p.dense_last.len() == p.dense_values.len()
            });
        if !chain {
            if warm.is_some() {
                self.publish(warm);
            } else if self.pending {
                // A buffer-less call *while a deferred result is outstanding*
                // is not this chain's caller — a chain's own first call has
                // nothing pending, which is what separates the two. It is
                // asking for a solve with no warm start (the linearizer's
                // one-shot) and it is about to overwrite the dense values the
                // chain was keeping, so deferral is switched off for good
                // rather than left pointing at values this call will replace.
                self.pending = false;
                self.watch.clear();
            }
        }
        let warm_start = warm.as_ref();
        if !matches {
            let mut subsystem: Vec<Equation> = ordinary.to_vec();
            for (name, value) in pinned {
                subsystem.push(Equation::new(
                    Expr::Var(name.clone()),
                    Expr::num(*value),
                    format!("{name} = {value}"),
                ));
            }
            let (constants, knowns) = builtin_constants(&subsystem);
            let report = block_system(&subsystem, &knowns)?;
            let mut seeded = self.specs.clone();
            seed_property_argument_guesses(&subsystem, &mut seeded, Missing::Create);
            let unknown_names = unknowns(&subsystem, &knowns);
            let mut block_cache =
                pinned_block_cache(&report.blocks, &subsystem, ordinary.len(), &seeded);
            // Wave I: the value map, built once in the exact insertion order
            // the per-call build used (constants, then unknowns), so its
            // layout — and every clone's — matches the old fresh build. The
            // unknown slots hold placeholders: every call overwrites them
            // below before anything reads the map.
            let mut work_scope: Scope = Scope::default();
            work_scope.extend(constants);
            for name in &unknown_names {
                work_scope.insert(name.clone(), 0.0);
            }
            // Wave R3: and its dense mirror, when every block compiled.
            let (dense, dense_values) = match dense_plan(
                &report.blocks,
                &mut block_cache,
                &work_scope,
                &unknown_names,
            ) {
                Some((plan, values)) => (Some(plan), values),
                None => (None, Vec::new()),
            };
            // The watch list is registered once, before any prep exists, and
            // resolved against each prep that follows. A linear scan because
            // it runs once per prep over a handful of names.
            let watch_slots = dense.as_ref().and_then(|plan: &DensePlan| {
                self.watch
                    .iter()
                    .map(|name| {
                        plan.names
                            .iter()
                            .position(|n| n == name)
                            .map(|index| index as u32)
                    })
                    .collect::<Option<Vec<u32>>>()
            });
            self.pending = false;
            self.prep = Some(PinnedPrep {
                template: ordinary.to_vec(),
                pinned_names: pinned.iter().map(|(name, _)| name.clone()).collect(),
                subsystem,
                report,
                seeded,
                unknowns: unknown_names,
                block_cache,
                work_scope,
                dense,
                dense_last: Vec::new(),
                watch_slots,
                dense_values,
            });
        }
        let prep = self.prep.as_mut().expect("prep just ensured");
        // Rewrite the pin values (and the value-bearing source text the
        // failure annotator quotes) into the reserved trailing slots. The
        // text is rewritten through the string's existing buffer (G3): same
        // bytes as `format!`, without a fresh allocation per pin per step —
        // the callgrind profile had 6.4 % of the whole run in this format.
        let base = prep.template.len();
        for (k, (name, value)) in pinned.iter().enumerate() {
            let pin = &mut prep.subsystem[base + k];
            // G3b: skip bit-identical pins. The analytic pins never change
            // across an integration, so only the time and state slots pay
            // the rewrite. Bit comparison, not `==`: −0.0 vs 0.0 Display
            // differently (so they must rewrite), and a bit-identical NaN's
            // text is already the "NaN" this write would produce.
            if let Expr::Num {
                value: old,
                unit: None,
                is_imaginary: false,
            } = &pin.rhs
            {
                if old.to_bits() == value.to_bits() {
                    continue;
                }
            }
            pin.rhs = Expr::num(*value);
            pin.source_text.clear();
            use std::fmt::Write as _;
            let _ = write!(pin.source_text, "{name} = {value}");
        }
        // The value-map fill and the Newton drive — the tail of
        // `solve_equation_list`, with the map's *allocation* hoisted into
        // the prep (Wave I; see [`PinnedPrep::work_scope`]). Every unknown
        // slot is overwritten here — including slots a failed previous call
        // left mid-iterate — so entry state equals the old fresh build's.
        // Wave R3: the dense mirror is valid only under the key set it was
        // built from. Nothing on the happy path can change that set — a block
        // writes back only its own variables, all of them unknowns — but the
        // retry ladder's merge rescue inserts, and so does an uncertainty
        // carry, and a prep that has seen either drops to the string-keyed
        // path for good rather than index a layout that has moved.
        if prep
            .dense
            .as_ref()
            .is_some_and(|plan| plan.names.len() != prep.work_scope.len())
        {
            prep.dense = None;
        }
        match &prep.dense {
            // Wave R3: under the deferred contract the warm start is this
            // prep's own record of its last successful solve — the same
            // numbers the caller's buffer would have handed back, since that
            // buffer is written from this record and by nothing else. One
            // copy replaces a probe per unknown, and the constants ride along
            // unchanged because they are in both.
            Some(_) if chain => {
                prep.dense_values.copy_from_slice(&prep.dense_last);
            }
            // The same rewrite, into slots instead of buckets. `initial_guess`
            // and the warm-start probe are untouched — only the destination of
            // the value changes, and the constants are already in place.
            Some(plan) => {
                for (name, &slot) in prep.unknowns.iter().zip(&plan.unknown_idx) {
                    let guess = warm_start
                        .and_then(|warm| warm.get(name).copied())
                        .unwrap_or_else(|| initial_guess(name, &prep.seeded));
                    prep.dense_values[slot as usize] = guess;
                }
            }
            None => {
                for name in &prep.unknowns {
                    let guess = warm_start
                        .and_then(|warm| warm.get(name).copied())
                        .unwrap_or_else(|| initial_guess(name, &prep.seeded));
                    match prep.work_scope.get_mut(name) {
                        Some(slot) => *slot = guess,
                        None => {
                            prep.work_scope.insert(name.clone(), guess);
                        }
                    }
                }
            }
        }
        if let Some(warm) = warm_start {
            // Unconditional, dense or not: this is the one thing that can add
            // a key, and the guard above is what reacts to it. A dense prep
            // cannot *read* these entries anyway — `UncertaintyOf(...)` is a
            // `Call`, which refuses to compile, so a block that mentions one
            // is a block that kept the prep off the dense path.
            crate::analysis::uncertainty::carry_uncertainty_entries(warm, &mut prep.work_scope);
        }
        let mut dense_run = match &prep.dense {
            Some(plan) => Some(DenseRun {
                plan,
                values: &mut prep.dense_values,
                live: true,
            }),
            None => None,
        };
        let outcome = run_blocks(
            &prep.report.blocks,
            &prep.subsystem,
            &mut prep.work_scope,
            self.settings,
            &prep.seeded,
            self.ctx,
            Some(&prep.block_cache),
            dense_run.as_mut(),
        );
        // A failed call must leave `warm` exactly as it found it, so the dense
        // state is simply abandoned — the next call rewrites every unknown of
        // it, which is the same contract `work_scope` has carried since Wave I.
        let dense_live = dense_run.is_some_and(|run| run.live);
        outcome.map_err(|failure| failure.error)?;

        // Wave Q3: hand the result back through the caller's own buffer. A
        // `Scope` clone allocates its table plus one `String` per key — nine
        // blocks a call on `hot-transient`, a quarter of everything the
        // per-step path allocates — and every one of those keys is already in
        // the buffer the caller lent us, because it is a former `work_scope`
        // of this same prep. Refreshing the values in place allocates nothing.
        //
        // The post-condition is the old `clone()`'s exactly: `warm` ends with
        // `work_scope`'s keys and values and nothing else. `work_scope` only
        // ever gains names, so after the loop the lent buffer is a superset;
        // the length check is what catches a buffer from a *different* prep
        // (the key sets differ) and rebuilds it outright.
        //
        // Wave R3: under the dense path this is the call's **only** hashed
        // pass — the source is `dense_values` rather than `work_scope`, walked
        // in `work_scope`'s own iteration order (that is what `plan.names`
        // is), so it writes the identical sequence of (name, value) pairs into
        // the identical buffer. The two rare arms need a current `work_scope`,
        // because what they hand back is a *clone* of it, so they export first.
        if dense_live {
            // The chain's record, kept only while the caller is running one: a
            // `warm` of `None` is a one-shot solve (the linearizer's), which
            // has never disturbed a trajectory's warm start and must not start
            // now.
            let defer = {
                let prep = self.prep.as_mut().expect("prep just ensured");
                if warm.is_some() {
                    prep.dense_last.clear();
                    prep.dense_last.extend_from_slice(&prep.dense_values);
                }
                warm.is_some() && prep.watch_slots.is_some() && !self.watch.is_empty()
            };
            self.pending = true;
            if !defer {
                self.publish(warm);
            }
            return Ok(());
        }
        // The string-keyed tail, unchanged since Wave Q3: `work_scope` is the
        // authority here, either because this prep has no dense plan or
        // because a block on this call handed the authority over.
        let prep = self.prep.as_mut().expect("prep just ensured");
        match warm {
            Some(previous) => {
                for (name, value) in prep.work_scope.iter() {
                    match previous.get_mut(name) {
                        Some(slot) => *slot = *value,
                        None => {
                            previous.insert(name.clone(), *value);
                        }
                    }
                }
                if previous.len() != prep.work_scope.len() {
                    *previous = prep.work_scope.clone();
                }
            }
            None => *warm = Some(prep.work_scope.clone()),
        }
        Ok(())
    }
}

/// The live ODE Table bridge for the second-solve pass.
///
/// Port of `EquationSystemSolver.installAccessorContext`, minus the thread
/// binding. `invertDisplayNames` is inlined: the map is `display → flat`, both
/// lowercased, so an accessor may address a component's transient state by its
/// dotted display name (`'m.port.T'`) as well as by the flat solver name
/// (`m$port$t`). A plain `DYNAMIC` state is absent from the map and passes
/// through unchanged.
fn accessor_bridge<'a>(
    doc: &'a Document,
    inner: &'a SolverSettings,
    specs: &'a BTreeMap<String, VarSpec>,
    ctx: EvalContext<'a>,
) -> crate::ode::accessors::DynamicAccessorContext<'a> {
    let display_to_flat: BTreeMap<String, String> = doc
        .display_names
        .iter()
        .map(|(flat, display)| (display.to_ascii_lowercase(), flat.to_ascii_lowercase()))
        .collect();
    let defs = &doc.defs;
    let runner = move |system: &crate::ode::dynamic::DynamicSystem, values: &Scope| {
        let algebraic = pinned_solver(inner, specs, ctx);
        crate::ode::dynamic::DynamicSolver::new(system, values, defs, algebraic).solve()
    };
    crate::ode::accessors::DynamicAccessorContext::install(
        &doc.dynamics,
        display_to_flat,
        Box::new(runner),
    )
}

/// Add zero-valued terms `+ 0·v` — one per input variable of the `DYNAMIC`
/// block an accessor reads — to every accessor-bearing equation.
///
/// Port of `EquationSystemSolver.augmentAccessorDependencies`. Tarjan blocking
/// and the Newton Jacobian both work off the *syntactic* variables of an
/// equation, and `MaxValue('h')` mentions none of the ODE's inputs; without the
/// augmentation the accessor constraint blocks on its own and the analytic
/// variable feeding the ODE is never adjusted to satisfy it.
///
/// Only variables the analytic system **already** mentions are linked, so no new
/// unknown is introduced; the added terms are identically zero and never move a
/// residual. That restriction is also why `FinalValue('Temp') = 30` alone cannot
/// solve for a `k` the analytic system never names — an oracle-confirmed
/// limitation, not a port bug (fixture `dyn_accessor_inverse`).
fn augment_accessor_dependencies(doc: &Document, equations: Vec<Equation>) -> Vec<Equation> {
    let analytic_vars: BTreeSet<String> = equations
        .iter()
        .flat_map(|eq| {
            let mut vars = BTreeSet::new();
            vars.extend(eq.lhs.variables());
            vars.extend(eq.rhs.variables());
            vars
        })
        .collect();
    let display_to_flat: BTreeMap<String, String> = doc
        .display_names
        .iter()
        .map(|(flat, display)| (display.to_ascii_lowercase(), flat.to_ascii_lowercase()))
        .collect();

    equations
        .into_iter()
        .map(|eq| {
            let mut cols = Vec::new();
            crate::ode::accessors::collect_accessor_columns(&eq.lhs, &mut cols);
            crate::ode::accessors::collect_accessor_columns(&eq.rhs, &mut cols);
            if cols.is_empty() {
                return eq;
            }
            let mut deps: Vec<String> = Vec::new();
            for col in &cols {
                let flat = display_to_flat
                    .get(col)
                    .cloned()
                    .unwrap_or_else(|| col.clone());
                for v in crate::ode::accessors::input_vars_for_column(&doc.dynamics, &flat) {
                    if analytic_vars.contains(&v) && !deps.contains(&v) {
                        deps.push(v);
                    }
                }
            }
            let mut lhs = eq.lhs;
            for v in deps {
                lhs = Expr::BinOp {
                    op: crate::ast::BinOp::Add,
                    left: Box::new(lhs),
                    right: Box::new(Expr::BinOp {
                        op: crate::ast::BinOp::Mul,
                        left: Box::new(Expr::num(0.0)),
                        right: Box::new(Expr::Var(v)),
                    }),
                };
            }
            Equation::new(lhs, eq.rhs, eq.source_text)
        })
        .collect()
}

/// The read-only half of the document [`inject_linearizations`] needs, bundled
/// so the `display_names` map it *writes* can be borrowed disjointly from the
/// fields it reads.
#[derive(Clone, Copy)]
struct LinearizeInputs<'a> {
    dynamics: &'a [crate::ode::dynamic::DynamicSystem],
    linearizes: &'a [crate::parser::blocks::LinearizeSystem],
    defs: &'a Definitions,
}

/// Numerically linearize each `LINEARIZE` block's network about its operating
/// point and inject the resulting matrix entries as equations.
///
/// Port of `EquationSystemSolver.injectLinearizations`. Each block names a
/// `DYNAMIC` block; the states are that block's `der()` variables, and
/// `INPUT`/`OUTPUT` name the exogenous inputs and observed outputs. The emitted
/// equations are ordinary members of the analytic system, which is what lets the
/// control suite (`CALL ss/lqr/place/…`) consume `A`/`B`/`C`/`D` in the same
/// solve.
fn inject_linearizations(
    doc: LinearizeInputs<'_>,
    display_names: &mut BTreeMap<String, String>,
    equations: Vec<Equation>,
    settings: &SolverSettings,
    specs: &BTreeMap<String, VarSpec>,
    ctx: EvalContext<'_>,
) -> Result<Vec<Equation>> {
    let LinearizeInputs {
        dynamics,
        linearizes,
        defs,
    } = doc;
    // `extractScalarConstants(equations)`: the operating point's exogenous
    // inputs are the document's plain `var = number` assignments, taken off the
    // equation list rather than from a solved iterate.
    let constants: Scope = equations
        .iter()
        .filter_map(|eq| match (&eq.lhs, &eq.rhs) {
            (
                Expr::Var(name),
                Expr::Num {
                    value,
                    is_imaginary: false,
                    ..
                },
            ) => Some((name.to_ascii_lowercase(), *value)),
            _ => None,
        })
        .collect();
    let display_to_flat: BTreeMap<String, String> = display_names
        .iter()
        .map(|(flat, display)| (display.to_ascii_lowercase(), flat.to_ascii_lowercase()))
        .collect();
    let resolve = |names: &[String]| -> Vec<String> {
        names
            .iter()
            .map(|n| display_to_flat.get(n).cloned().unwrap_or_else(|| n.clone()))
            .collect()
    };

    let inner = relaxed_ode_settings(settings, 1e-7);
    let mut out = equations;
    for ls in linearizes {
        let Some(ds) = dynamics
            .iter()
            .find(|d| d.name.eq_ignore_ascii_case(&ls.dynamic_name))
        else {
            return Err(FreesError::solver(format!(
                "LINEARIZE {}: no DYNAMIC block named '{}' (it names the transient component \
                 network to linearize).",
                ls.name, ls.dynamic_name
            )));
        };
        let algebraic = pinned_solver(&inner, specs, ctx);
        let lin = crate::ode::dynamic::DynamicSolver::new(ds, &constants, defs, algebraic)
            .linearize(&resolve(&ls.inputs), &resolve(&ls.outputs))?;
        for (name, m) in [
            (&ls.a_name, &lin.a),
            (&ls.b_name, &lin.b),
            (&ls.c_name, &lin.c),
            (&ls.d_name, &lin.d),
        ] {
            emit_matrix(&mut out, name, m, display_names);
        }
    }
    Ok(out)
}

/// One matrix as `name[i,j] = value` equations — the Java `emitMatrix`,
/// including its column-vector special case: a single-column matrix *also* gets
/// the 1-subscript spelling `name[i]`, so a SISO control call written `B[1:n]`
/// resolves. Display names are registered `putIfAbsent`, in the header's case.
fn emit_matrix(
    out: &mut Vec<Equation>,
    name: &str,
    m: &[Vec<f64>],
    display_names: &mut BTreeMap<String, String>,
) {
    let lower = name.to_ascii_lowercase();
    for (i, row) in m.iter().enumerate() {
        for (j, &value) in row.iter().enumerate() {
            let k2 = format!("{lower}[{},{}]", i + 1, j + 1);
            out.push(Equation::new(
                Expr::Var(k2.clone()),
                Expr::num(value),
                format!("{k2} (linearized)"),
            ));
            display_names
                .entry(k2)
                .or_insert_with(|| format!("{name}[{},{}]", i + 1, j + 1));
            if row.len() == 1 {
                let k1 = format!("{lower}[{}]", i + 1);
                out.push(Equation::new(
                    Expr::Var(k1.clone()),
                    Expr::num(row[0]),
                    format!("{k1} (linearized)"),
                ));
                display_names
                    .entry(k1)
                    .or_insert_with(|| format!("{name}[{}]", i + 1));
            }
        }
    }
}

/// Verify syntax and structural solvability without solving. Equivalent to
/// [`check_with`] with no external variable information.
pub fn check(source: &str) -> Result<CheckReport> {
    check_with(source, &[])
}

/// Verify syntax and structural solvability without solving.
///
/// Mirrors `POST /api/check`. *Every* failure of the document itself is data,
/// not an `Err`:
///
/// * a **syntax** failure (or an unported construct) returns a report with
///   `solvable: false`, `error_line`/`errors` pointing at the offending line
///   and a `"Syntax error: …"` message — the body the Java `CheckController`
///   sends with its 400, which `api.ts` parses like any other check result;
/// * a **structural** failure (degrees of freedom, singular matching) returns
///   `solvable: false` with the blocker's diagnosis in `message`, because the
///   editor needs the counts and the variable list either way.
///
/// `overrides` cannot change a check's outcome — the Java check endpoint never
/// receives `variableInfo`, and structural solvability does not depend on
/// guesses. They are still *validated*, so a caller preparing a solve gets the
/// same early `Err` surface from both entry points.
pub fn check_with(source: &str, overrides: &[VariableOverride]) -> Result<CheckReport> {
    check_with_tables(source, overrides, &[])
}

/// [`check_with`] with externally supplied Function Table definitions — the
/// `POST /api/check` body's `functionTables`, which the Java threads as
/// `solver.check(parsed, complexMode, SolveDtos.functionDefsOf(...))`
/// (`CheckController`). Same merge and same position as
/// [`solve_with_tables`]: `withExtraDefs` semantics (source wins), applied
/// after the expansion pipeline, so the structural analysis and the Integral
/// pass see the merged map while the unit checker — which the Java feeds the
/// plain `parsed` — never does. An empty slice is byte-for-byte
/// [`check_with`].
pub fn check_with_tables(
    source: &str,
    overrides: &[VariableOverride],
    extra_tables: &[crate::parser::defs::FunctionTableDef],
) -> Result<CheckReport> {
    check_with_tables_complex(source, overrides, extra_tables, false)
}

/// [`check_with_tables`] with the complex-mode flag the Java `CheckController`
/// threads in as `solver.check(parsed, complexMode, functionDefs)`.
///
/// The flag reaches exactly one place — [`crate::parser::complex::expand_complex`]
/// — and it decides whether an imaginary literal is a hard error or a request
/// to split every equation into a real and an imaginary half. That matters to
/// *check* and not only to solve, because the frontend gates its Solve button
/// on this report: up to Wave T5 the boundary could not pass the flag, so a
/// document with `1i` in it was reported unsolvable with "enable Complex mode
/// to solve them" while Complex mode was enabled, and the Solve that would
/// have succeeded was never reachable.
pub fn check_with_tables_complex(
    source: &str,
    overrides: &[VariableOverride],
    extra_tables: &[crate::parser::defs::FunctionTableDef],
    complex_mode: bool,
) -> Result<CheckReport> {
    crate::props::tables::install_builtin_once();
    for o in overrides {
        override_spec(o)?;
    }

    let doc = match parse_document(source).and_then(|doc| {
        reject_unsupported(&doc)?;
        Ok(doc)
    }) {
        Ok(doc) => doc,
        Err(err @ FreesError::Parse { .. }) => return Ok(syntax_failure_report(source, &err)),
        Err(other) => return Err(other),
    };

    // The same expansion pipeline as `solve_with` (stages 2–4; Java `check`
    // runs `ComplexExpansion` too — complex mode is off here because the
    // check entry point carries no settings, exactly like the Java
    // `check(source)` overload defaulting `complexMode` to false). A failing
    // pass answers as report data, mirroring the two Java surfaces: a
    // `ParseException` becomes the 400-with-body syntax report, and anything
    // else the `catch (SolverException)` not-solvable report with the counts
    // of the equations as they stood when the pass refused.
    // Display names accumulate across the pipeline exactly as on the solve
    // path; the closure below works on a clone of the document, so the names
    // the CALL flattener generates are collected out here.
    let identities = component_identities(&doc, source);
    let mut parsed_names = doc.display_names.clone();
    let mut check_diagnostics = doc.diagnostics.clone();
    check_diagnostics.extend(identity_advisories(&doc, source));
    let mut member_units: BTreeMap<String, String> = BTreeMap::new();
    let mut layer_connections: Vec<crate::components::expander::Connection> = Vec::new();
    let expanded = (|| {
        let mut doc = doc.clone();
        // Stage 1b — the component layer, at the Java position (see
        // `expand_component_layer`). `check` runs it for the same reason
        // `solve` does: without it a component document has zero equations and
        // would be reported "solvable" with nothing in it.
        let components = expand_component_layer(&mut doc, &mut check_diagnostics)?;
        member_units = components.member_units;
        layer_connections = components.connections.clone();
        parsed_names = doc.display_names.clone();
        let statements = std::mem::take(&mut doc.statements);
        let (flattened, module_count) =
            flatten_calls_counted(statements, &doc.defs, &mut parsed_names)?;
        doc.statements = flattened;
        let mut equations = components.equations;
        equations.extend(crate::parser::expand::expand_document_with(
            &doc,
            module_count,
            &mut parsed_names,
        )?);
        // String variables leave the numeric system here too, so `check`
        // reports the same equation/variable balance the solve path sees.
        let equations = crate::parser::string_variables::resolve(equations, &parsed_names)?;
        // The request's GUI Function Tables, at the Java position (see
        // `solve_with_parametric_tables`): merged into this closure's clone
        // only, so `find_integrals` below sees them while the unit paths
        // outside — which the Java feeds the plain `parsed` — do not.
        doc.defs.merge_extra_tables(extra_tables);
        // The Integral pass, as in `solve_with` — but `check` builds the
        // *structural view* instead of driving the quadrature: a constant-limit
        // integral contributes a `resultVar = 0` placeholder, a variable-limit
        // one its inlined equation, and each integration variable is pinned
        // once. Without that pin `F = Integral(t^2, t, 0, 1)` is one equation
        // in two unknowns and the blocker would reject a valid document.
        let equations = crate::integral::hoist_nested(equations);
        // `UncertaintyOf(X) = expr` is a declaration, not an equation — the
        // Java `check` lifts it out at exactly this position, so a document
        // that states an uncertainty reports the same equation/variable balance
        // it solves at.
        let equations = crate::analysis::uncertainty::extract_uncertainty_equations(&equations)
            .active_equations;
        let integrals = find_integrals(&equations, &doc.defs, false)?;
        if integrals.is_empty() {
            crate::parser::complex::expand_complex(equations, complex_mode)
        } else {
            crate::integral::structural_view(&equations, &integrals)
        }
    })();
    let equations: Vec<Equation> = match expanded {
        Ok(equations) => equations,
        Err(err @ FreesError::Parse { .. }) => return Ok(syntax_failure_report(source, &err)),
        Err(err) => {
            let equations: Vec<Equation> = doc.equations().into_iter().cloned().collect();
            let (_, knowns) = builtin_constants(&equations);
            let variables = unknowns(&equations, &knowns);
            return Ok(CheckReport {
                solvable: false,
                equation_count: equations.len(),
                unknown_count: variables.len(),
                display_names: complete_display_names(&parsed_names, &equations),
                variables,
                message: err.to_string_message(),
                error_line: None,
                errors: Vec::new(),
                inferred_units: BTreeMap::new(),
                unit_warnings: Vec::new(),
                diagnostics: check_diagnostics,
                plots: doc.blocks.plots.clone(),
                connections: layer_connections,
                instances: identities.0.clone(),
                definitions: identities.1.clone(),
            });
        }
    };
    // "A document whose only content is DYNAMIC block(s) has no analytic
    // equations — the ODE system is self-contained and solvable directly."
    // Java `check`, at this exact position: it reports the block's own equation
    // count as *both* the equation and the variable count, so the balance reads
    // square, and never runs the blocker (which would reject an empty system).
    if equations.is_empty() && !doc.dynamics.is_empty() {
        let dyn_eqs: usize = doc
            .dynamics
            .iter()
            .map(|ds| ds.body_equations.len() + ds.initials.len())
            .sum();
        return Ok(CheckReport {
            solvable: true,
            equation_count: dyn_eqs,
            unknown_count: dyn_eqs,
            display_names: complete_display_names(&parsed_names, &equations),
            variables: Vec::new(),
            message: format!(
                "No syntax errors were detected. DYNAMIC system with {dyn_eqs} equation(s)."
            ),
            error_line: None,
            errors: Vec::new(),
            inferred_units: BTreeMap::new(),
            unit_warnings: Vec::new(),
            diagnostics: check_diagnostics,
            plots: doc.blocks.plots.clone(),
            connections: layer_connections.clone(),
            instances: identities.0.clone(),
            definitions: identities.1.clone(),
        });
    }

    let (_, knowns) = builtin_constants(&equations);
    let all_vars = unknowns(&equations, &knowns);
    // Java `check`: report the *surfaced* balance. Each ignored-output sink adds
    // one variable and the one equation that determines it, so hiding the sink
    // without also hiding its equation would report a bogus `n equations and
    // n-1 variables`. Java subtracts both
    // (`surfacedEqs = equations.size() - (allVars.size() - surfacedVars)`).
    let surfaced_vars = surfaced_count(all_vars.iter().map(String::as_str));
    let surfaced_eqs = equations.len() - (all_vars.len() - surfaced_vars);
    let variables: Vec<String> = all_vars
        .into_iter()
        .filter(|name| !crate::parser::toplevel::is_ignored_sink(name))
        .collect();

    let mut diagnostics = check_diagnostics;
    collect_unit_warnings(&equations, &mut diagnostics);

    // Dimensional check + SI unit inference (the Java `CheckController` path):
    // declared units — annotated literals plus external `VariableInfo` units —
    // feed the checker; the reported map is the derived units overlaid by the
    // *literal*-declared ones only. External units are deliberately left out of
    // the report (the caller already knows them), exactly as the Java check
    // response composes `deriveUnits` + `inferUnits`.
    //
    // Component stream members are the exception the Java makes explicitly:
    // `CheckController.addComponentMemberUnits` puts them back into the
    // *reported* map too (`putIfAbsent`), because a port member's unit comes
    // from its physical domain and nothing else can derive it.
    let declared = declared_units(&equations, overrides, &member_units);
    let unit_report = crate::units::checker::check_units(&equations, &declared);
    let mut inferred_units = unit_report.inferred;
    inferred_units.extend(literal_units(&equations));
    for (name, unit) in &member_units {
        inferred_units
            .entry(name.to_ascii_lowercase())
            .or_insert_with(|| unit.clone());
    }

    let base = CheckReport {
        solvable: false,
        equation_count: surfaced_eqs,
        unknown_count: surfaced_vars,
        display_names: complete_display_names(&parsed_names, &equations),
        variables,
        message: String::new(),
        error_line: None,
        errors: Vec::new(),
        inferred_units,
        unit_warnings: merge_advisory_warnings(unit_report.warnings, &diagnostics),
        diagnostics,
        plots: doc.blocks.plots.clone(),
        connections: layer_connections,
        instances: identities.0,
        definitions: identities.1,
    };

    match block_system(&equations, &knowns) {
        Ok(_) => Ok(CheckReport {
            solvable: true,
            message: format!(
                "No syntax errors were detected. There are {} equations and {} variables.",
                base.equation_count, base.unknown_count
            ),
            ..base
        }),
        Err(err) => Ok(CheckReport {
            message: err.to_string_message(),
            ..base
        }),
    }
}

// ---------------------------------------------------------------------------
// Pipeline steps
// ---------------------------------------------------------------------------

/// Refuse anything that parses but that the engine cannot honour.
fn reject_unsupported(doc: &Document) -> Result<()> {
    reject_unsupported_statements(&doc.statements)
}

/// Everything the acausal component layer contributes to a solve.
///
/// Built by [`expand_component_layer`] and consumed in three places: the
/// equations seed the equation list, `member_units` grounds the unit checker,
/// and the last two fields ride out on [`Solution`] for the datasheet and the
/// schematic.
#[derive(Debug, Default)]
struct ComponentLayer {
    /// The expanded scalar equations — component bodies then `connect` nodes.
    equations: Vec<Equation>,
    /// Flat solver name → SI unit (`s2$p` → `Pa`), the Java
    /// `ParseResult.componentMemberUnits`.
    member_units: BTreeMap<String, String>,
    /// The top-level instantiations, for [`Solution::component_instances`].
    instances: Vec<crate::components::metadata::ComponentInstMeta>,
    /// The connection topology, for [`Solution::component_connections`].
    connections: Vec<crate::components::expander::Connection>,
}

/// Expand the acausal `COMPONENT`/`connect` layer into flat scalar equations
/// and rewrite the document's own dotted references onto the same flat names.
///
/// # Where this sits in the pipeline, and why
///
/// **Established by reading `EquationParser.parseResult`** (`../frEES/backend/
/// core/src/main/java/com/frees/backend/parser/EquationParser.java:265-345`),
/// which runs, in order:
///
/// ```text
///   AstBuilder.buildProgram(program)                       // parse
///   new ComponentExpander(ComponentLibrary.builtins(), …)  // <- HERE
///   componentEquations = components.expand()
///   statements         = components.rewriteStatements(statements)
///   …storage routing (steadyStorageEquations / routeStorageIntoDynamic)…
///   List<Equation> equations = new BoundedEquationList(componentEquations);
///   flatten(statements, …, equations, …)                   // CALL + matrix + FOR
///   equations = StringVariables.resolve(equations, displayNames)
/// ```
///
/// and only then does `EquationSystemSolver.solve` run `ComplexExpansion.expand`
/// and block. So expansion is **the first expansion pass**: before CALL
/// flattening, before matrix expansion, before complex, and well before
/// blocking. Two consequences are load-bearing rather than incidental:
///
/// * the component equations are the **seed** of the list (`new
///   BoundedEquationList(componentEquations)`), so they come *before* every
///   equation the document itself wrote — this port prepends for the same
///   reason, and the residual list, block ordering and `block_equations` all
///   inherit that order;
/// * `rewriteStatements` runs on the statements *before* they are flattened, so
///   a dotted `P1.out.h` inside a `FOR` body or a `CALL` argument is rewritten
///   once, at the AST level, and the later passes only ever see flat names.
///
/// # Storage
///
/// `hasStorage()` (any component body with `der(member) = …`) picks one of the
/// two branches `EquationParser.parseResult` spells, and Phase 7 makes both
/// reachable:
///
/// * **no `DYNAMIC` block** → the §8.2 steady/transient duality. Each
///   `der(X) = rhs` becomes the equilibrium constraint `rhs = 0` and the state
///   is an ordinary unknown. That is [`steady_storage_equations`].
/// * **exactly one `DYNAMIC` block** → `routeStorageIntoDynamic`. The component
///   equations *become* that block's body, the `init(member) = …` lines become
///   its initial conditions, and the steady equation list is emptied. The same
///   network then runs under the ODE engine, whose per-step algebraic solve
///   resolves it at each state.
///
/// Two or more blocks with storage is a hard parse error in the Java, because
/// nothing says which block supplies the time span.
///
/// Every `DYNAMIC` body is also put through `rewriteDynamicBodies` first: the
/// block's own equations live *inside* the block, so `rewrite_statements` never
/// saw them, and a scheduled input written as `RIN.out.mdot = f(time)` would
/// leave the dotted port variable free.
fn expand_component_layer(
    doc: &mut Document,
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<ComponentLayer> {
    // The Java constructs a `ComponentExpander` unconditionally, but its
    // `ComponentLibrary.builtins()` is a static already paid for at class-load.
    // Here the 122 KB of embedded library text is parsed lazily on first use, so
    // a document with no component layer must not touch it at all — otherwise
    // every scalar solve in the corpus pays for 295 component definitions.
    if doc.components.is_empty() {
        // One thing the Java's always-on expander still does to such a
        // document: its top-level rewrite mangles dotted names (`RET.in.P` →
        // `ret$in$p`) and records their display spellings, instances or no
        // instances. The 2026-08-22 docs harvest caught the early return
        // skipping that (`docs_components_05`). Reproduce it with an expander
        // over empty definition lists — no instances, no connects, and no
        // parse of the builtin library — and only when a dotted name actually
        // appears, so the corpus's scalar documents still pay nothing.
        if mentions_dotted_var(&doc.statements, &doc.dynamics) {
            let statements = std::mem::take(&mut doc.statements);
            let mut dynamics = std::mem::take(&mut doc.dynamics);
            let mut display_names = std::mem::take(&mut doc.display_names);
            let mut expander = crate::components::expander::ComponentExpander::new(
                &[],
                &[],
                &[],
                &[],
                &mut display_names,
            )?;
            doc.statements = expander.rewrite_statements(statements)?;
            rewrite_dynamic_bodies(&mut expander, &mut dynamics)?;
            doc.dynamics = dynamics;
            doc.display_names = display_names;
        }
        return Ok(ComponentLayer::default());
    }

    let components = std::mem::take(&mut doc.components);
    let statements = std::mem::take(&mut doc.statements);
    let mut dynamics = std::mem::take(&mut doc.dynamics);
    let mut display_names = std::mem::take(&mut doc.display_names);

    let builtins = crate::components::library::builtins()?;
    let (equations, statements, member_units, connections) = {
        let mut expander = crate::components::expander::ComponentExpander::new(
            builtins.defs(),
            &components.defs,
            &components.instances,
            &components.connects,
            &mut display_names,
        )?;
        let equations = expander.expand()?;
        for warning in expander.inactive_warnings() {
            diagnostics.push(crate::diag::Diagnostic::warning(warning.clone()));
        }
        let statements = expander.rewrite_statements(statements)?;
        let member_units = expander
            .member_units()
            .into_iter()
            .map(|(name, unit)| (name, unit.to_string()))
            .collect();
        // `rewriteDynamicBodies(components, programResult.dynamicSystems())`,
        // at the Java position — before the storage branch, and unconditional.
        rewrite_dynamic_bodies(&mut expander, &mut dynamics)?;
        let equations = if expander.has_storage() {
            if dynamics.is_empty() {
                // Steady/transient duality (§8.2).
                steady_storage_equations(equations, diagnostics)
            } else {
                // `routeStorageIntoDynamic`, then `componentEquations = List.of()`.
                route_storage_into_dynamic(equations, expander.component_initials(), &mut dynamics)?
            }
        } else {
            equations
        };
        (equations, statements, member_units, expander.connections())
    };

    doc.statements = statements;
    doc.dynamics = dynamics;
    doc.display_names = display_names;
    let instances = components
        .instances
        .iter()
        .map(crate::components::metadata::ComponentInstMeta::from)
        .collect();

    Ok(ComponentLayer {
        equations,
        member_units,
        instances,
        connections,
    })
}

/// Rewrite every `DYNAMIC` body so dotted component references resolve to flat
/// solver names — the same rewrite top-level statements get.
///
/// Port of `EquationParser.rewriteDynamicBodies`. A block's body lives *inside*
/// the block, so `rewrite_statements` never reaches it; without this a
/// scheduled input written in the body (`RIN.out.mdot = mdot_max * min(time/t_ramp, 1)`)
/// leaves the port variable free and the network is underdetermined. Bodies with
/// no component references come out unchanged.
///
/// A `set` action's target is a *variable name*, so it goes through the same
/// rewrite and keeps the flat name when the rewrite produces a plain `Var`.
/// Does anything the component layer's top-level rewrite would touch mention a
/// dotted variable (`a.b`, `inst.port.member`)? Gates the component-free
/// rewrite in [`expand_component_layer`]: the Java pays its rewrite on every
/// document, this port only where it can matter.
fn mentions_dotted_var(
    statements: &[Statement],
    dynamics: &[crate::ode::dynamic::DynamicSystem],
) -> bool {
    fn expr_dotted(e: &Expr) -> bool {
        e.variables().iter().any(|v| v.contains('.'))
    }
    fn statements_dotted(statements: &[Statement]) -> bool {
        statements.iter().any(|s| match s {
            Statement::Eq(eq) => expr_dotted(&eq.lhs) || expr_dotted(&eq.rhs),
            Statement::For {
                start, end, body, ..
            } => expr_dotted(start) || expr_dotted(end) || statements_dotted(body),
            Statement::Symbolic(_) => false,
            Statement::CallProc {
                inputs, outputs, ..
            } => inputs.iter().any(expr_dotted) || outputs.iter().any(expr_dotted),
        })
    }
    statements_dotted(statements)
        || dynamics.iter().any(|ds| {
            ds.body_equations
                .iter()
                .any(|eq| expr_dotted(&eq.lhs) || expr_dotted(&eq.rhs))
                || statements_dotted(&ds.for_blocks)
                || ds.initials.iter().any(|ic| expr_dotted(&ic.value))
                || ds.events.iter().any(|ev| {
                    expr_dotted(&ev.lhs)
                        || expr_dotted(&ev.rhs)
                        || ev.set_var.as_deref().is_some_and(|v| v.contains('.'))
                        || ev.set_expr.as_ref().is_some_and(expr_dotted)
                })
        })
}

fn rewrite_dynamic_bodies(
    expander: &mut crate::components::expander::ComponentExpander<'_, '_>,
    systems: &mut [crate::ode::dynamic::DynamicSystem],
) -> Result<()> {
    for ds in systems.iter_mut() {
        for eq in &mut ds.body_equations {
            *eq = expander.rewrite_top_equation(eq)?;
        }
        for ic in &mut ds.initials {
            ic.value = expander.rewrite_top_expr(&ic.value)?;
        }
        for ev in &mut ds.events {
            ev.lhs = expander.rewrite_top_expr(&ev.lhs)?;
            ev.rhs = expander.rewrite_top_expr(&ev.rhs)?;
            if let Some(target) = &ev.set_var {
                if let Expr::Var(flat) = expander.rewrite_top_expr(&Expr::Var(target.clone()))? {
                    ev.set_var = Some(flat);
                }
            }
            if let Some(value) = &ev.set_expr {
                ev.set_expr = Some(expander.rewrite_top_expr(value)?);
            }
        }
    }
    Ok(())
}

/// Merge a transient component network into the document's single `DYNAMIC`
/// block and return the (now empty) steady equation list.
///
/// Port of `EquationParser.routeStorageIntoDynamic`. The component equations —
/// state `der(X) = …` lines *and* the algebraic network — extend the block's
/// body, and the `init(member) = …` lines extend its initial conditions. The
/// block's own body comes first, so a document-authored equation is seen before
/// the expanded network.
fn route_storage_into_dynamic(
    component_equations: Vec<Equation>,
    initials: &[crate::components::expander::ComponentInitial],
    systems: &mut [crate::ode::dynamic::DynamicSystem],
) -> Result<Vec<Equation>> {
    if systems.len() != 1 {
        return Err(FreesError::parse(format!(
            "A transient component network (a component with der(...) storage) needs exactly \
             one DYNAMIC block to supply the time span and method; found {}. Add a \
             'DYNAMIC name(time = 0 .. T) END' block.",
            systems.len()
        )));
    }
    let ds = &mut systems[0];
    ds.body_equations.extend(component_equations);
    ds.initials.extend(
        initials
            .iter()
            .map(|ci| crate::ode::dynamic::InitialCondition {
                state: ci.state.clone(),
                indices: Vec::new(),
                value: ci.value.clone(),
            }),
    );
    Ok(Vec::new())
}

/// `der(X) = rhs` → `rhs = 0`, the Java `EquationParser.steadyStorageEquations`.
///
/// A storage network with no `DYNAMIC` block solves its **steady operating
/// point**: the state stops changing, so its derivative equation becomes the
/// equilibrium constraint and `X` is determined by the rest of the network.
/// The Java appends `" [steady: der=0]"` to the equation's source text; that is
/// kept verbatim so the block listing says which equations were rewritten.
///
/// The Java is silent about it. This port adds one *warning* diagnostic (never
/// an error — the parent invariant is that a document like this solves), because
/// in the browser there is no server log to read afterwards and the answer is
/// materially different from a transient run.
fn steady_storage_equations(
    equations: Vec<Equation>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<Equation> {
    let mut rewritten = 0usize;
    let out: Vec<Equation> = equations
        .into_iter()
        .map(|eq| {
            let is_der = matches!(
                &eq.lhs,
                Expr::Call { function, args } if function == "der" && args.len() == 1
            );
            if !is_der {
                return eq;
            }
            rewritten += 1;
            Equation::new(
                eq.rhs,
                Expr::num(0.0),
                format!("{} [steady: der=0]", eq.source_text),
            )
        })
        .collect();
    if rewritten > 0 {
        diagnostics.push(Diagnostic::warning(format!(
            "This component network stores energy or mass ({rewritten} der(...) \
             equation(s)) but the document declares no DYNAMIC block, so it was \
             solved for its steady operating point (each der(X) = rhs became \
             rhs = 0)."
        )));
    }
    out
}

/// Refuse a statement that parses but that the engine cannot honour.
///
/// The list is **empty**: every statement form the grammar admits now has a
/// home. `CALL` belongs to [`crate::procedures::flatten_calls`] (pipeline
/// stage 2) and [`crate::parser::expand`] (stage 3), which refuse the
/// intrinsics they do not implement *by name*; `SYMBOLIC` is consumed by
/// [`crate::parser::expand`]'s `flatten_identity`, which solves the identity
/// through [`crate::cas::engine::solve_coefficients`] (Phase 9). The function
/// is kept — rather than deleted with its call site — because it is the seam
/// where the next statement form the AST grows gets refused instead of
/// silently dropped, exactly as `parser::toplevel::unsupported_construct` is
/// for block forms.
fn reject_unsupported_statements(statements: &[Statement]) -> Result<()> {
    for statement in statements {
        match statement {
            Statement::Eq(_) | Statement::CallProc { .. } | Statement::Symbolic(_) => {}
            Statement::For { body, .. } => reject_unsupported_statements(body)?,
        }
    }
    Ok(())
}

/// Every `#`-suffixed built-in the document mentions, as `(name, value)` pairs
/// plus the same names as a `knowns` set for the blocker.
///
/// See the module docs: without this a constant is an unmatched free variable.
fn builtin_constants(equations: &[Equation]) -> (BTreeMap<String, f64>, HashSet<String>) {
    let mut constants = BTreeMap::new();
    for equation in equations {
        for name in equation.variables() {
            if constants.contains_key(&name) {
                continue;
            }
            if let Some(constant) = lookup_constant(&name) {
                constants.insert(name, constant.value);
            }
        }
    }
    let knowns = constants.keys().cloned().collect();
    (constants, knowns)
}

/// Warn about every literal whose unit annotation the registry could not parse.
///
/// `parser::expr::number_literal` deliberately keeps the value and the user's
/// text when a unit is unknown — "a bad unit never fails a parse", and unit
/// problems are warnings by the parent engine's invariant. But it has nowhere
/// to *put* a warning, so as written `P = 140 [zorp]` silently solved to 140
/// with no indication that the annotation was ignored. Since a converted
/// literal always carries the SI display name and only an unconverted one
/// carries the original text, an annotation that still fails to parse is
/// exactly the set of literals that were left alone.
///
/// This is a stand-in for the unported `UnitChecker`, which additionally
/// verifies dimensional consistency across an equation.
/// Append non-unit advisory diagnostics onto the unit-warning list so Check
/// and Solve both surface them on the existing `unitWarnings` channel.
pub(crate) fn component_identities(
    doc: &Document,
    source: &str,
) -> (Vec<InstanceIdentity>, Vec<DefinitionIdentity>) {
    let local_types: BTreeSet<String> =
        doc.components.defs.iter().map(|d| d.name.clone()).collect();
    let mut instances = Vec::new();
    collect_instances(
        &doc.components.instances,
        "",
        0,
        &doc.components.defs,
        &local_types,
        &mut Vec::new(),
        &mut instances,
    );
    let definitions = definition_identities(source);
    (instances, definitions)
}

fn collect_instances(
    insts: &[crate::components::def::ComponentInst],
    scope: &str,
    parent_line: usize,
    user_defs: &[crate::components::def::ComponentDef],
    local_types: &BTreeSet<String>,
    stack: &mut Vec<String>,
    out: &mut Vec<InstanceIdentity>,
) {
    for inst in insts {
        let name = if scope.is_empty() {
            inst.name.clone()
        } else {
            format!("{scope}.{}", inst.name)
        };
        let (_, label) = crate::components::metadata::display_identity(
            &crate::components::metadata::ComponentInstMeta::from(inst),
        );
        // Nested children share the COMPONENT-body declaration line. Point
        // them at the parent instantiation so a repeated child name in two
        // scopes navigates to the instance that created this one.
        let line = if scope.is_empty() {
            inst.line
        } else {
            parent_line
        };
        out.push(InstanceIdentity {
            name: name.clone(),
            label,
            type_name: inst.type_name.clone(),
            line,
            local_type: local_types.contains(&inst.type_name),
        });
        // Expansion refuses cycles; Check still walks identities first, so a
        // self-instantiation must not recurse here (wasm/native abort).
        if stack.contains(&inst.type_name) || stack.len() >= 64 {
            continue;
        }
        if let Some(def) = user_defs.iter().find(|d| d.name == inst.type_name) {
            stack.push(inst.type_name.clone());
            collect_instances(
                &def.sub_instances,
                &name,
                if scope.is_empty() {
                    inst.line
                } else {
                    parent_line
                },
                user_defs,
                local_types,
                stack,
                out,
            );
            stack.pop();
        }
    }
}

fn definition_identities(source: &str) -> Vec<DefinitionIdentity> {
    let mut out = Vec::new();
    for (i, line) in source.lines().enumerate() {
        let trimmed = line.trim_start();
        let rest = if trimmed.len() >= 9 && trimmed[..9].eq_ignore_ascii_case("component") {
            trimmed[9..].trim_start()
        } else {
            continue;
        };
        let name: String = rest
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if !name.is_empty() {
            out.push(DefinitionIdentity {
                name: name.to_ascii_lowercase(),
                line: i + 1,
            });
        }
    }
    out
}

fn identity_advisories(doc: &Document, source: &str) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    if !doc.components.defs.is_empty() {
        if let Ok(lib) = crate::components::library::builtins() {
            let builtin: BTreeSet<String> = lib.iter().map(|e| e.def.name.clone()).collect();
            let defs = definition_identities(source);
            for def in &doc.components.defs {
                if builtin.contains(&def.name) {
                    let line = defs
                        .iter()
                        .find(|d| d.name == def.name)
                        .map(|d| d.line)
                        .unwrap_or(0);
                    let where_ = if line > 0 {
                        format!(" (line {line})")
                    } else {
                        String::new()
                    };
                    out.push(Diagnostic::warning(format!(
                        "Local definition '{}'{} shadows the standard-library component of the same name. \
                         The local physics is used.",
                        def.name, where_
                    )));
                }
            }
        }
    }
    out.extend(case_collision_advisories(source));
    out
}

fn case_collision_advisories(source: &str) -> Vec<Diagnostic> {
    use std::collections::BTreeMap;
    let mut spellings: BTreeMap<String, Vec<(String, usize)>> = BTreeMap::new();
    let mut depth = 0usize;
    for (i, line) in source.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('{') || trimmed.starts_with("//") || trimmed.starts_with('"') {
            continue;
        }
        let head: String = trimmed
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect::<String>()
            .to_ascii_lowercase();
        // Body equations inside COMPONENT/FUNCTION/… expand to scoped names;
        // a source-level T/t scan there is a false conflict across instances.
        if matches!(
            head.as_str(),
            "component"
                | "subsystem"
                | "function"
                | "procedure"
                | "module"
                | "table"
                | "parametric"
                | "plot"
                | "dynamic"
                | "linearize"
        ) {
            depth += 1;
            continue;
        }
        if depth > 0 {
            if head == "end" {
                depth -= 1;
            }
            continue;
        }
        let ident: String = trimmed
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if ident.is_empty() {
            continue;
        }
        let rest = &trimmed[ident.len()..];
        if !rest.trim_start().starts_with('=') {
            continue;
        }
        let key = ident.to_ascii_lowercase();
        if matches!(
            key.as_str(),
            "if" | "for" | "end" | "function" | "procedure" | "component" | "connect"
        ) {
            continue;
        }
        spellings.entry(key).or_default().push((ident, i + 1));
    }
    let mut out = Vec::new();
    for (canon, hits) in spellings {
        let mut distinct: Vec<&(String, usize)> = Vec::new();
        for hit in &hits {
            if !distinct.iter().any(|(s, _)| s.as_str() == hit.0) {
                distinct.push(hit);
            }
        }
        if distinct.len() < 2 {
            continue;
        }
        let listed = distinct
            .iter()
            .map(|(s, line)| format!("'{s}' (line {line})"))
            .collect::<Vec<_>>()
            .join(" and ");
        out.push(Diagnostic::warning(format!(
            "Case-only naming conflict: {listed} are the same variable '{canon}'."
        )));
    }
    out
}

fn merge_advisory_warnings(
    mut unit_warnings: Vec<String>,
    diagnostics: &[Diagnostic],
) -> Vec<String> {
    for diagnostic in diagnostics {
        if diagnostic.severity != crate::diag::Severity::Warning {
            continue;
        }
        if !diagnostic.message.contains("is not used by the selected")
            && !diagnostic.message.contains("shadows the standard-library")
            && !diagnostic.message.contains("Case-only naming conflict")
        {
            continue;
        }
        if unit_warnings.iter().any(|w| w == &diagnostic.message) {
            continue;
        }
        unit_warnings.push(diagnostic.message.clone());
    }
    unit_warnings
}

fn collect_unit_warnings(equations: &[Equation], diagnostics: &mut Vec<Diagnostic>) {
    let mut seen = BTreeSet::new();
    for equation in equations {
        for side in [&equation.lhs, &equation.rhs] {
            walk_units(side, &mut |unit| {
                if UnitRegistry::parse(unit).is_err() && seen.insert(unit.to_string()) {
                    diagnostics.push(
                        Diagnostic::warning(format!(
                            "unknown unit `{unit}`: the literal was left unconverted, \
                             so this value is not in SI"
                        ))
                        .with_source_text(equation.source_text.clone()),
                    );
                }
            });
        }
    }
}

fn walk_units(expr: &Expr, report: &mut impl FnMut(&str)) {
    match expr {
        Expr::Num { unit: Some(u), .. } => report(u),
        Expr::Num { .. } | Expr::Str(_) | Expr::Var(_) => {}
        Expr::Neg(inner) | Expr::Not(inner) => walk_units(inner, report),
        Expr::BinOp { left, right, .. }
        | Expr::Compare { left, right, .. }
        | Expr::Logical { left, right, .. }
        | Expr::Range {
            start: left,
            end: right,
        } => {
            walk_units(left, report);
            walk_units(right, report);
        }
        Expr::ArrayLiteral(items) => items.iter().for_each(|e| walk_units(e, report)),
        Expr::Call { args, .. } => args.iter().for_each(|e| walk_units(e, report)),
        Expr::ArrayAccess { indices, .. } => indices.iter().for_each(|e| walk_units(e, report)),
    }
}

/// Split `a[1,2]` into `("a", "[1,2]")`; `None` for a name that is not an
/// expanded array/matrix element.
fn element_parts(name: &str) -> Option<(&str, &str)> {
    if !name.ends_with(']') {
        return None;
    }
    let open = name.find('[')?;
    (open > 0).then(|| name.split_at(open))
}

/// How many of `names` the user ever sees — the Java
/// `EquationSystemSolver.surfacedVarCount`. Ignored-output sinks are genuine
/// unknowns to the solver and invisible everywhere else.
fn surfaced_count<'a>(names: impl Iterator<Item = &'a str>) -> usize {
    names
        .filter(|name| !crate::parser::toplevel::is_ignored_sink(name))
        .count()
}

/// The Java `ParseResult.displayNames`, completed with what expansion adds.
///
/// The parser has already recorded every identifier the Java `AstBuilder`
/// registers (see [`crate::parser::Document::display_names`]) and the CALL
/// flattener has added its namespaced module variables. What remains is the
/// **element** rule, which the Java applies at four sites that all compute the
/// same thing (`EquationParser.expandExpr`'s `ArrayAccess` case,
/// `buildElementVars`, and the range expansions inside `parseMatrixInfo` /
/// `parseVectorInfo`, plus `flattenBareMatrixCreation`):
///
/// ```text
/// displayNames.put(base + "[i,j]",
///                  displayNames.getOrDefault(base, base) + "[i,j]")
/// ```
///
/// Because every one of those sites fires exactly when an element name enters
/// the equation list, replaying the rule over the expanded system's variables
/// reproduces the map without threading a mutable map through the whole
/// flattener. Note the Java uses `put`, not `putIfAbsent`: an element entry
/// always wins over anything with the same key, which is why this runs after
/// the parse-time map is in place and overwrites.
fn complete_display_names(
    parsed: &BTreeMap<String, String>,
    equations: &[Equation],
) -> BTreeMap<String, String> {
    let mut names = parsed.clone();
    let mut elements: BTreeSet<String> = BTreeSet::new();
    for equation in equations {
        elements.extend(
            equation
                .variables()
                .into_iter()
                // An internal matrix-library temporary (`backslash_temp_6[1]`)
                // is minted as an element Var directly, so in the Java it never
                // passes any of the four registration sites and `displayNames`
                // has no entry for it — while it *does* stay in the variables
                // map. Replaying the rule over the expanded system would invent
                // entries the oracle lacks (caught by the 2026-08-22 docs
                // harvest, `docs_matrix_algebra_03`).
                .filter(|v| {
                    element_parts(v).is_some() && !crate::parser::expand::is_internal_temp(v)
                }),
        );
    }
    for element in elements {
        let Some((base, suffix)) = element_parts(&element) else {
            continue;
        };
        let base_display = names.get(base).cloned().unwrap_or_else(|| base.to_string());
        let composed = format!("{base_display}{suffix}");
        // `putIfAbsent`, not `put`. Element names are normally absent here — the
        // parser registers only an array's *base* spelling — so this fills them
        // in from the base, exactly as before. The one producer that registers
        // an element name up front is `emit_matrix` (`A[1,1]` from a `LINEARIZE`
        // header), and composing `a` + `[1,1]` would silently downcase it.
        names.entry(element).or_insert(composed);
    }
    names
}

/// The check report for a document that did not parse — the body the Java
/// `CheckController` returns with its 400 (`solvable=false`, zero counts,
/// `"Syntax error: …"`, `errorLine`, `errors`), which `api.ts` consumes like
/// any other check result.
fn syntax_failure_report(source: &str, err: &FreesError) -> CheckReport {
    let message = err.to_string_message();
    // The Java message is "Syntax error: " + the first line of the parser's
    // report (multi-line detail stays in `errors`/diagnostics).
    let first_line = message.lines().next().unwrap_or_default().to_string();

    let (error_line, errors) = match err.span() {
        Some(span) => {
            let (line, column) = span.line_col(source);
            (
                Some(line),
                vec![SyntaxErrorInfo {
                    line,
                    column,
                    message: message.clone(),
                }],
            )
        }
        // A parse refusal with no position (mirrors Java semantic
        // ParseExceptions, whose syntaxErrors() list is empty).
        None => (None, Vec::new()),
    };

    let mut diagnostic = Diagnostic::error(message);
    if let Some(span) = err.span() {
        diagnostic = diagnostic.with_span(span);
    }

    CheckReport {
        solvable: false,
        equation_count: 0,
        unknown_count: 0,
        variables: Vec::new(),
        display_names: BTreeMap::new(),
        message: format!("Syntax error: {first_line}"),
        error_line,
        errors,
        inferred_units: BTreeMap::new(),
        unit_warnings: Vec::new(),
        diagnostics: vec![diagnostic],
        // A document that did not parse declares nothing.
        plots: Vec::new(),
        connections: Vec::new(),
        instances: Vec::new(),
        definitions: Vec::new(),
    }
}

/// Lowercase variable name → declared unit read off annotated literal
/// assignments: `P = 100 [bar]` declares `p`'s unit (as its SI display name,
/// since literals convert at parse time), first assignment wins. The Java
/// `EquationSystemSolver.inferUnits`.
fn literal_units(equations: &[Equation]) -> BTreeMap<String, String> {
    let mut units = BTreeMap::new();
    for equation in equations {
        let declared = match (&equation.lhs, &equation.rhs) {
            (
                Expr::Var(name),
                Expr::Num {
                    unit: Some(unit), ..
                },
            )
            | (
                Expr::Num {
                    unit: Some(unit), ..
                },
                Expr::Var(name),
            ) => Some((name, unit)),
            _ => None,
        };
        if let Some((name, unit)) = declared {
            units.entry(name.clone()).or_insert_with(|| unit.clone());
        }
    }
    units
}

/// Every unit declaration the unit checker should treat as ground truth: the
/// literal-annotated ones, overlaid by external `VariableInfo` units — an
/// explicit user unit always wins. The Java `SolverApiSupport.effectiveUnits`
/// (minus component member units, which wait on the component expander).
fn declared_units(
    equations: &[Equation],
    overrides: &[VariableOverride],
    component_member_units: &BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    let mut units = literal_units(equations);
    // `SolverApiSupport.effectiveUnits`: inferred literal units, then the
    // component stream members' domain-derived units (`s2$p` → `Pa`), then the
    // explicit Variable-Information units, which win over both. A port member is
    // one of the solver's own unknowns, so nothing in the document derives its
    // unit — its physical domain is the only thing that fixes it, and without
    // this the checker would propagate *from* a dimensionless port and warn.
    units.extend(
        component_member_units
            .iter()
            .map(|(name, unit)| (name.to_ascii_lowercase(), unit.clone())),
    );
    for o in overrides {
        if let Some(unit) = o.unit.as_deref() {
            if !unit.trim().is_empty() {
                units.insert(o.name.trim().to_ascii_lowercase(), unit.to_string());
            }
        }
    }
    units
}

/// Every equation's residual at the final values, in source order, plus the
/// largest finite `|residual|` — the Java `buildResult` loop.
///
/// An equation that cannot be evaluated at the solution records `NaN` rather
/// than failing the whole solve (Java catches and stores `NaN` the same way);
/// non-finite residuals do not contribute to `max_residual`.
fn residuals_at(
    equations: &[Equation],
    blocks: &[Block],
    values: &Scope,
    ctx: EvalContext<'_>,
) -> (Vec<EquationResidual>, f64) {
    // Which Tarjan block solved each equation. The blocker assigns every
    // equation of a square system to exactly one block; `unwrap_or(0)` is a
    // defensive default, not an expected path.
    let mut block_of: HashMap<usize, usize> = HashMap::with_capacity(equations.len());
    for (block_index, block) in blocks.iter().enumerate() {
        for &equation_index in &block.equations {
            block_of.insert(equation_index, block_index);
        }
    }

    let mut residuals = Vec::with_capacity(equations.len());
    let mut max_residual = 0.0f64;
    for (index, equation) in equations.iter().enumerate() {
        let residual = match (
            eval_with(&equation.lhs, values, ctx),
            eval_with(&equation.rhs, values, ctx),
        ) {
            (Ok(lhs), Ok(rhs)) => lhs - rhs,
            _ => f64::NAN,
        };
        if residual.is_finite() {
            max_residual = max_residual.max(residual.abs());
        }
        residuals.push(EquationResidual {
            equation: equation.source_text.clone(),
            residual,
            block: block_of.get(&index).copied().unwrap_or(0),
        });
    }
    (residuals, max_residual)
}

/// Per block, the source text of its equations — first occurrence of a
/// repeated text kept, as the Java `toBlockDto`'s `.distinct()` does, so a
/// future component expansion never lists one user line twice.
fn block_equation_texts(blocks: &[Block], equations: &[Equation]) -> Vec<Vec<String>> {
    blocks
        .iter()
        .map(|block| {
            let mut seen = BTreeSet::new();
            block
                .equations
                .iter()
                .filter_map(|&index| equations.get(index))
                .filter(|equation| seen.insert(equation.source_text.as_str()))
                .map(|equation| equation.source_text.clone())
                .collect()
        })
        .collect()
}

/// Initial guesses and bounds for every unknown.
///
/// Port of `EquationSystemSolver.withTextGuesses` over the externally supplied
/// specs. **The merge rule, verified against the Java source:** the overrides
/// are applied first (they are the caller's `variableInfo` specs); each in-text
/// `GUESS` directive then merges *over* them field by field — a part the
/// directive states wins, a part it omits falls back to the override — and the
/// merged guess is clamped into the merged bounds. In-text wins on conflict;
/// the external value only survives where the directive is silent.
///
/// A directive naming something that is not an unknown is a warning, not an
/// error — the Java engine merges it into a spec map that the solver then never
/// reads. An *override* naming something that is not an unknown is silently
/// ignored, also the Java behaviour: the Variable Information window keeps
/// stale rows around and posts them with every request.
fn variable_specs(
    equations: &[Equation],
    knowns: &HashSet<String>,
    doc: &Document,
    overrides: &[VariableOverride],
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<BTreeMap<String, VarSpec>> {
    let mut specs: BTreeMap<String, VarSpec> = unknowns(equations, knowns)
        .into_iter()
        .map(|name| (name, VarSpec::default()))
        .collect();

    for o in overrides {
        let (name, spec) = override_spec(o)?;
        // Only real unknowns take a spec: `specs.keys()` defines the result
        // rows, so a stale override must not add a phantom variable.
        // Public member paths (`hx.in.p`) are the same names Variable
        // Information shows; map them onto the expanded `$` scalar.
        if let Some(key) = guess_spec_key(&name, &specs) {
            if let Some(slot) = specs.get_mut(&key) {
                *slot = spec;
            }
        }
    }

    for guess in &doc.guesses {
        let name = guess.name.to_ascii_lowercase();
        let key = guess_spec_key(&name, &specs);
        match key.and_then(|k| specs.get_mut(&k)) {
            Some(spec) => apply_guess(spec, guess),
            None if knowns.contains(&name) || knowns.contains(&name.replace('.', "$")) => {
                diagnostics.push(Diagnostic::warning(format!(
                    "GUESS for `{name}`: that name is a built-in constant, \
                     not an unknown — the directive is ignored"
                )))
            }
            None => diagnostics.push(Diagnostic::warning(format!(
                "GUESS for `{name}`: no such variable in the system — \
                 the directive is ignored"
            ))),
        }
    }

    Ok(specs)
}

/// One override converted to `(lowercase name, spec)`, the port of
/// `VariableInfoDto.toSpec` plus the `VariableSpec` constructor's validation.
///
/// The numbers are written in `unit` and convert to SI as
/// `value * factor + offset`; an unknown unit falls back to factor 1 / offset 0
/// **silently**, exactly as the Java `toSpec` catch-and-default does — being
/// noisier here would be a visible behavioural divergence. A missing guess
/// defaults to [`DEFAULT_GUESS`] clamped into the bounds.
///
/// # Errors
///
/// The Java `VariableSpec` compact constructor's three rejections, as
/// [`FreesError::Solver`]: any NaN, a lower bound above the upper, or an
/// explicit guess outside its own bounds.
/// The stated `±` of every override, in SI, keyed by canonical name.
///
/// Port of `VariableInfoDto.toSpec`'s `uncertainty * factor`: an uncertainty is
/// an interval width, so the unit's **offset does not apply** — `1 [C]` of
/// uncertainty is 1 K, not 274.15 K. A `None`, non-finite or non-positive value
/// is left out; only `> 0.0` makes a variable a source
/// (`partitionVariables`), so carrying a zero would change nothing.
// `!(unc > 0.0)` rather than `unc <= 0.0` is the port's NaN-rejecting form: the
// Java writes `uncertainty() > 0.0` as the *positive* test, and negating the
// positive form is what keeps a NaN on the reject side (`NaN <= 0.0` is false,
// so the readable spelling would let a NaN uncertainty through as a source).
// `neg_cmp_op_on_partial_ord` wants the readable form; the parity rule wins, so
// the lint is silenced here exactly as `analysis/uncertainty.rs` silences it
// module-wide for the same two guards.
#[allow(clippy::neg_cmp_op_on_partial_ord)]
fn override_uncertainties(overrides: &[VariableOverride]) -> BTreeMap<String, f64> {
    let mut out = BTreeMap::new();
    for o in overrides {
        let Some(unc) = o.uncertainty else { continue };
        if !(unc > 0.0) || !unc.is_finite() {
            continue;
        }
        let factor = match o.unit.as_deref().map(str::trim) {
            Some(unit) if !unit.is_empty() && unit != "-" => UnitRegistry::parse_with_offset(unit)
                .map(|q| q.factor)
                .unwrap_or(1.0),
            _ => 1.0,
        };
        out.insert(o.name.trim().to_ascii_lowercase(), unc * factor);
    }
    out
}

fn override_spec(o: &VariableOverride) -> Result<(String, VarSpec)> {
    let name = o.name.trim().to_ascii_lowercase();

    let (factor, offset) = match o.unit.as_deref().map(str::trim) {
        Some(unit) if !unit.is_empty() && unit != "-" => {
            match UnitRegistry::parse_with_offset(unit) {
                Ok(quantity) => (quantity.factor, quantity.offset),
                Err(_) => (1.0, 0.0),
            }
        }
        _ => (1.0, 0.0),
    };
    let to_si = |value: f64| value * factor + offset;

    let lower = o.lower.map(to_si).unwrap_or(f64::NEG_INFINITY);
    let upper = o.upper.map(to_si).unwrap_or(f64::INFINITY);
    let explicit_guess = o.guess.map(to_si);

    if lower.is_nan() || upper.is_nan() || explicit_guess.is_some_and(f64::is_nan) {
        return Err(FreesError::solver(format!(
            "Variable information for {name} contains NaN."
        )));
    }
    if lower > upper {
        return Err(FreesError::solver(format!(
            "Lower bound exceeds upper bound for variable {name}."
        )));
    }
    let guess = match explicit_guess {
        Some(guess) if guess < lower || guess > upper => {
            return Err(FreesError::solver(format!(
                "The guess value {guess} for variable {name} is outside \
                 its bounds [{lower}, {upper}]."
            )));
        }
        Some(guess) => guess,
        None => DEFAULT_GUESS.clamp(lower, upper),
    };

    Ok((
        name,
        VarSpec {
            guess,
            lower,
            upper,
        },
    ))
}

/// Map a public GUESS name onto the solver key. Dotted member paths
/// (`hx.in.p`) expand to `$`-flattened scalars (`hx$in$p`).
fn guess_spec_key(name: &str, specs: &BTreeMap<String, VarSpec>) -> Option<String> {
    if specs.contains_key(name) {
        return Some(name.to_string());
    }
    let mangled = name.replace('.', "$");
    if mangled != name && specs.contains_key(&mangled) {
        return Some(mangled);
    }
    None
}

fn apply_guess(spec: &mut VarSpec, guess: &GuessDirective) {
    if let Some(lower) = guess.lower {
        spec.lower = lower;
    }
    if let Some(upper) = guess.upper {
        spec.upper = upper;
    }
    if let Some(value) = guess.guess {
        spec.guess = value;
    }
    // A reversed pair would make `clamp` panic; treat it as no ordering
    // information rather than aborting the solve over a typo in a hint.
    if spec.lower > spec.upper {
        std::mem::swap(&mut spec.lower, &mut spec.upper);
    }
}

// ---------------------------------------------------------------------------
// Phase-A guess seeding — port of `seedPropertyArgumentGuesses`
// ---------------------------------------------------------------------------

/// Domain-aware nominal guesses for property-call arguments, keyed by the
/// encoded input indicator (`prop$enthalpy$water$p$h` → the args carry
/// indicators `p`, `h`). Port of `PROP_ARG_NOMINAL`.
///
/// A property argument left at [`DEFAULT_GUESS`] means **1 Pa / 1 J/kg / 1 K**,
/// which is below every fluid's valid range, so the very first residual is
/// `NaN` and the solve never starts.
fn prop_arg_nominal(indicator: &str) -> Option<f64> {
    Some(match indicator {
        "p" => 1.0e5,     // pressure ~1 bar
        "t" => 300.0,     // temperature ~ambient
        "h" => 1.0e5,     // enthalpy — inside the liquid range of most fluids
        "s" => 1.0e3,     // entropy
        "u" => 1.0e5,     // internal energy
        "d" => 1.0,       // density
        "x" | "q" => 0.5, // quality (two-phase; `x` in frees, `Q` in CoolProp)
        "v" => 1.0e-3,    // specific volume
        _ => return None,
    })
}

/// Physical nominal for a component/stream member, keyed by the token after the
/// last `$`. Port of `MEMBER_NOMINAL`: only the members where the default guess
/// is clearly unphysical (a 1 Pa pressure sends a `√(ΔP)` resistance imaginary;
/// a 1 K temperature breaks property calls). Ambiguous members are deliberately
/// omitted — `v` is voltage as often as specific volume — as is enthalpy, whose
/// reference point is fluid-dependent. (In principle
/// [`seed_consistent_enthalpy`] supplies the fluid-aware enthalpy instead; in
/// practice the generic 1e5 nominal from [`seed_prop_args_in`] lands first and
/// the fluid-aware gate never opens — measured corpus-wide in Wave-3 F7,
/// `docs/status-wave3-f7.md` §6, and faithful to the Java's identical order.)
fn member_nominal(member: &str) -> Option<f64> {
    match member {
        "p" => Some(1.0e5),
        "t" => Some(300.0),
        _ => None,
    }
}

/// The `$`-split tokens of an encoded property call, or `None` for any other
/// function. The encoding is `prop$<output>$<fluid>$<ind…>`.
fn prop_tokens(function: &str) -> Option<Vec<&str>> {
    function
        .starts_with(crate::props::propfun::PREFIX)
        .then(|| function.split('$').collect())
}

/// The input indicator belonging to argument `k` of an `n`-argument encoded
/// call: the encoding ends with exactly one name token per argument, so the
/// indicators are the *last* `n` tokens (Java: `ti = tok.length - n + k`, with
/// a negative `ti` skipped).
fn arg_indicator<'a>(tokens: &[&'a str], n: usize, k: usize) -> Option<&'a str> {
    let ti = tokens.len() as isize - n as isize + k as isize;
    usize::try_from(ti)
        .ok()
        .and_then(|ti| tokens.get(ti))
        .copied()
}

/// Seed an initial guess for every unknown that appears as a bare argument of a
/// property call and still sits at [`DEFAULT_GUESS`]. Port of
/// `EquationSystemSolver.seedPropertyArgumentGuesses` (the Java "Phase A"
/// consistent-state init), called from the same position — once per equation
/// list about to be solved, before any value map is built from the specs.
///
/// This puts an implicit-property base point inside the table's valid box so
/// the first residual evaluates, which is what the monotonic inversions
/// (`Temperature(P,h)`, `Density(T,P)`, …) need in order to converge at all.
/// **User and GUI guesses always win** — only an untouched default is replaced.
///
/// Without it, a closed refrigerant loop cold-starts with every port enthalpy
/// at 1 J/kg and every port pressure at 1 Pa; `ev-battery-cooling-pid` is the
/// document that showed it, dying on `T(R134a, P=350000, Hmass=1)` in the
/// evaporator's 14-equation block before Newton could take a single step.
/// Whether a seeder may *add* a spec for a variable that has none.
///
/// The two call sites need different answers, and the difference is not
/// cosmetic — getting it wrong silently un-fixes a document at one site or
/// corrupts a result at the other:
///
/// * [`solve_with`] seeds the document-level map, which is built over
///   `unknowns()` and already holds every unknown. A name missing there is a
///   *known*, and `specs.keys()` is what reports `unknown_count` — so adding
///   would invent a variable. [`Missing::Skip`].
/// * [`solve_equation_list`] seeds a throwaway clone for one subsystem, whose
///   variables are not all in the document-level map it started from (a
///   transient's per-step algebraic system is the case that matters). Adding
///   is what makes the seed reach them at all, and the map dies with the call.
///   [`Missing::Create`], which is also what the Java does with its
///   `expandedSpecs` over every name.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Missing {
    Skip,
    Create,
}

fn seed_property_argument_guesses(
    equations: &[Equation],
    specs: &mut BTreeMap<String, VarSpec>,
    missing: Missing,
) {
    // The fluid-aware refrigerant pressure seed runs **first**, so the generic
    // 1-bar nominal below cannot overwrite it.
    for equation in equations {
        seed_refrigerant_pressure(&equation.lhs, specs, missing);
        seed_refrigerant_pressure(&equation.rhs, specs, missing);
    }
    for equation in equations {
        seed_prop_args_in(&equation.lhs, specs, missing);
        seed_prop_args_in(&equation.rhs, specs, missing);
    }
    seed_stream_member_guesses(equations, specs, missing);
    // Last: the reference-dependent enthalpies, which need the pressures the
    // passes above have just seeded. Running last also means seed_prop_args_in
    // has already taken every default `h` with its generic 1e5, so the
    // fluid-aware branch below is corpus-wide dead — see its doc comment.
    for equation in equations {
        seed_consistent_enthalpy(&equation.lhs, specs, missing);
        seed_consistent_enthalpy(&equation.rhs, specs, missing);
    }
}

/// Would [`apply_nominal_guess`] actually change this variable?
///
/// Both fluid-aware seeders reach a *property lookup* to compute their nominal,
/// and both then hand it to a function that discards it when the variable
/// already carries a user guess. Asking first is behaviour-identical and keeps
/// the seeding pass off the hot path: it runs once per equation list, and
/// inside a `DYNAMIC` that is once per integrator stage, where the answer is
/// the same every time.
fn needs_seed(var: &str, specs: &BTreeMap<String, VarSpec>) -> bool {
    specs
        .get(var)
        .is_none_or(|spec| spec.guess == DEFAULT_GUESS)
}

/// Seed the pressure argument of a condensable-refrigerant property call to a
/// sub-critical operating nominal (~0.35·Pcrit) instead of the generic 1 bar,
/// so a floating-pressure refrigerant cycle cold-starts in-band. Port of
/// `seedRefrigerantPressure`.
fn seed_refrigerant_pressure(expr: &Expr, specs: &mut BTreeMap<String, VarSpec>, missing: Missing) {
    match expr {
        Expr::Call { function, args } => {
            if let Some(tokens) = prop_tokens(function) {
                if tokens.len() >= 3 {
                    // Collect the pressure arguments that would actually take a
                    // seed *before* paying for the Pcrit lookup.
                    let pending: Vec<String> = args
                        .iter()
                        .enumerate()
                        .filter_map(
                            |(k, arg)| match (arg_indicator(&tokens, args.len(), k), arg) {
                                (Some("p"), Expr::Var(name)) => Some(name.to_ascii_lowercase()),
                                _ => None,
                            },
                        )
                        .filter(|name| needs_seed(name, specs))
                        .collect();
                    if !pending.is_empty() {
                        let p_nom = crate::props::propfun::nominal_pressure(tokens[2]);
                        if p_nom.is_finite() {
                            // p_nom = 0.35·Pcrit ⇒ physical bounds [10 kPa, ~1.5·Pcrit],
                            // which keep the floating pressure positive and in-table so
                            // Newton's line-search clamp cannot step it to NaN.
                            let p_lo = 1.0e4;
                            let p_hi = p_nom / 0.35 * 1.5;
                            for name in pending {
                                apply_nominal_guess_with_bounds(
                                    &name, p_nom, p_lo, p_hi, specs, missing,
                                );
                            }
                        }
                    }
                }
            }
            for arg in args {
                seed_refrigerant_pressure(arg, specs, missing);
            }
        }
        Expr::BinOp { left, right, .. } => {
            seed_refrigerant_pressure(left, specs, missing);
            seed_refrigerant_pressure(right, specs, missing);
        }
        Expr::Neg(inner) => seed_refrigerant_pressure(inner, specs, missing),
        // Leaf or non-arithmetic node: nothing to seed. The Java walker descends
        // exactly these three node kinds and no others; widening it here would
        // seed variables the oracle leaves alone.
        _ => {}
    }
}

/// Seed each property-call argument from [`prop_arg_nominal`]. Port of
/// `seedPropArgsIn`.
fn seed_prop_args_in(expr: &Expr, specs: &mut BTreeMap<String, VarSpec>, missing: Missing) {
    match expr {
        Expr::Call { function, args } => {
            if let Some(tokens) = prop_tokens(function) {
                for (k, arg) in args.iter().enumerate() {
                    if let (Some(indicator), Expr::Var(name)) =
                        (arg_indicator(&tokens, args.len(), k), arg)
                    {
                        if let Some(nominal) = prop_arg_nominal(indicator) {
                            apply_nominal_guess(
                                &name.to_ascii_lowercase(),
                                nominal,
                                specs,
                                missing,
                            );
                        }
                    }
                }
            }
            for arg in args {
                seed_prop_args_in(arg, specs, missing);
            }
        }
        Expr::BinOp { left, right, .. } => {
            seed_prop_args_in(left, specs, missing);
            seed_prop_args_in(right, specs, missing);
        }
        Expr::Neg(inner) => seed_prop_args_in(inner, specs, missing),
        _ => {}
    }
}

/// Seed the enthalpy argument of every property call — if still at the default
/// guess — to a thermodynamically consistent `Enthalpy(fluid, P, x=0.5)` (or
/// `T ≈ 300 K` for an incompressible), using the call's *own* pressure
/// argument, which the earlier passes have already seeded. Port of
/// `seedConsistentEnthalpy`, the pass *designed* as the principled fix for the
/// closed-loop cold-start NaN (an enthalpy stuck at 1 J/kg is below every
/// fluid's table range). Measured corpus-wide in Wave-3 F7
/// (`docs/status-wave3-f7.md` §6): the fluid-aware nominal never actually
/// runs, because [`seed_prop_args_in`] has already replaced the default guess
/// with its flat 1e5 `h` nominal by the time this pass asks [`needs_seed`].
/// The Java has the identical ordering and early-return, so the no-op is
/// parity, not a bug; reordering the passes would change initial guesses at
/// 28 call sites and is a parity decision, not a cleanup.
fn seed_consistent_enthalpy(expr: &Expr, specs: &mut BTreeMap<String, VarSpec>, missing: Missing) {
    match expr {
        Expr::Call { function, args } => {
            if let Some(tokens) = prop_tokens(function) {
                let mut h_var: Option<String> = None;
                let mut p_seed = 1.0e5;
                for (k, arg) in args.iter().enumerate() {
                    let Some(indicator) = arg_indicator(&tokens, args.len(), k) else {
                        continue;
                    };
                    match (indicator, arg) {
                        ("h", Expr::Var(name)) => h_var = Some(name.to_ascii_lowercase()),
                        ("p", Expr::Var(name)) => {
                            if let Some(spec) = specs.get(&name.to_ascii_lowercase()) {
                                p_seed = spec.guess;
                            }
                        }
                        ("p", Expr::Num { value, .. }) => p_seed = *value,
                        _ => {}
                    }
                }
                if let Some(h_var) = h_var {
                    // `nominal_enthalpy` is a real property-table lookup, so ask
                    // whether it can change anything before paying for it.
                    if tokens.len() >= 3 && needs_seed(&h_var, specs) {
                        let h = crate::props::propfun::nominal_enthalpy(tokens[2], p_seed);
                        if h.is_finite() {
                            apply_nominal_guess(&h_var, h, specs, missing);
                        }
                    }
                }
            }
            for arg in args {
                seed_consistent_enthalpy(arg, specs, missing);
            }
        }
        Expr::BinOp { left, right, .. } => {
            seed_consistent_enthalpy(left, specs, missing);
            seed_consistent_enthalpy(right, specs, missing);
        }
        Expr::Neg(inner) => seed_consistent_enthalpy(inner, specs, missing),
        _ => {}
    }
}

/// Seed physical initial guesses for component/stream member unknowns: a flat
/// name like `s$p` or `vlv$in$p` still at the default gets a domain nominal, so
/// a coupled fluid-resistance network does not start with a negative ΔP. Port
/// of `seedStreamMemberGuesses`. Derivative names are skipped — `der$x$p` is a
/// rate, not a pressure.
fn seed_stream_member_guesses(
    equations: &[Equation],
    specs: &mut BTreeMap<String, VarSpec>,
    missing: Missing,
) {
    for equation in equations {
        for name in equation.variables() {
            if name.starts_with("der$") {
                continue;
            }
            let Some(dollar) = name.rfind('$') else {
                continue;
            };
            if let Some(nominal) = member_nominal(&name[dollar + 1..]) {
                apply_nominal_guess(&name, nominal, specs, missing);
            }
        }
    }
}

/// Install `nominal` as `var`'s guess unless the user already set one. Port of
/// `applyNominalGuess`.
fn apply_nominal_guess(
    var: &str,
    nominal: f64,
    specs: &mut BTreeMap<String, VarSpec>,
    missing: Missing,
) {
    // Java's `applyNominalGuess` always creates when the spec is absent; see
    // [`Missing`] for why the port has to decide that per call site.
    let Some(spec) = specs.get_mut(var) else {
        if missing == Missing::Create {
            specs.insert(
                var.to_string(),
                VarSpec {
                    guess: nominal,
                    ..VarSpec::default()
                },
            );
        }
        return;
    };
    if spec.guess != DEFAULT_GUESS {
        return; // a user/GUI guess is already set — it always wins
    }
    spec.guess = nominal.clamp(spec.lower, spec.upper);
}

/// [`apply_nominal_guess`] plus physical bounds where the user left them open,
/// so Newton's line-search clamp keeps the variable in a valid region. Port of
/// `applyNominalGuessWithBounds`. User guesses and user-set bounds always win.
fn apply_nominal_guess_with_bounds(
    var: &str,
    nominal: f64,
    lo: f64,
    hi: f64,
    specs: &mut BTreeMap<String, VarSpec>,
    missing: Missing,
) {
    let Some(spec) = specs.get_mut(var) else {
        if missing == Missing::Create {
            specs.insert(
                var.to_string(),
                VarSpec {
                    guess: nominal,
                    lower: lo,
                    upper: hi,
                },
            );
        }
        return;
    };
    if spec.guess != DEFAULT_GUESS {
        return; // user/GUI guess wins — and keeps its bounds too
    }
    let use_lo = if spec.lower.is_infinite() {
        lo
    } else {
        spec.lower
    };
    let use_hi = if spec.upper.is_infinite() {
        hi
    } else {
        spec.upper
    };
    spec.lower = use_lo;
    spec.upper = use_hi;
    spec.guess = nominal.clamp(use_lo, use_hi);
}

// ---------------------------------------------------------------------------
// The per-block solve, its analytic Jacobian, and the Java retry ladder
// ---------------------------------------------------------------------------

/// The residual system of one block, evaluated against the shared scope, with
/// the per-entry symbolic derivatives where the differentiator produced them —
/// the object handed to [`newton_solve`].
struct BlockProblem<'a> {
    names: &'a [String],
    equations: &'a [&'a Equation],
    scope: &'a mut Scope,
    ctx: EvalContext<'a>,
    /// `derivs[i][j] = ∂(lhs_i − rhs_i)/∂var_j` as an expression; `None`
    /// entries are structural zeros (equation `i` does not mention `var_j`).
    /// The whole field is `None` when any *dependent* entry failed to
    /// differentiate — the Java `analyticalJacobian` returning null, which
    /// pins the block to finite differences. Borrowed since Wave G3, so the
    /// per-step path can hand in [`PinnedBlockStruct`]'s cached trees instead
    /// of re-differentiating every call.
    derivs: Option<&'a Vec<Vec<Option<Expr>>>>,
    /// Cached [`NewtonProblem::row_dependencies`] result, same provenance.
    row_deps: Option<&'a Vec<Vec<usize>>>,
    /// The last property/evaluation refusal seen while evaluating residuals,
    /// so a NaN stall still names its physical cause (the Java
    /// `NewtonSolver.lastPropertyError` → `propertyErrorSuffix`). Owned by
    /// [`solve_block`], which reads it back after Newton has consumed the
    /// problem.
    last_property_error: &'a mut Option<String>,
}

impl NewtonProblem for BlockProblem<'_> {
    fn residual(&mut self, x: &[f64], out: &mut [f64]) -> Result<()> {
        for (name, value) in self.names.iter().zip(x) {
            // get_mut-or-insert: after the first iteration the key exists, and
            // a plain `insert(name.clone(), …)` would clone-and-drop the name
            // once per variable per residual evaluation (G3 profile).
            match self.scope.get_mut(name) {
                Some(slot) => *slot = *value,
                None => {
                    self.scope.insert(name.clone(), *value);
                }
            }
        }
        residuals_lenient(
            self.equations,
            self.scope,
            self.ctx,
            out,
            self.last_property_error,
        )
    }

    /// `varToEquations`: equation `i` depends on variable `j` exactly when it
    /// mentions it, which is the same structural test
    /// [`analytic_derivatives`] uses to decide a structural zero.
    fn row_dependencies(&self) -> Option<Vec<Vec<usize>>> {
        if let Some(cached) = self.row_deps {
            return Some(cached.clone());
        }
        let mentioned: Vec<BTreeSet<String>> =
            self.equations.iter().map(|e| e.variables()).collect();
        Some(
            self.names
                .iter()
                .map(|name| {
                    mentioned
                        .iter()
                        .enumerate()
                        .filter(|(_, vars)| vars.contains(name))
                        .map(|(i, _)| i)
                        .collect()
                })
                .collect(),
        )
    }

    /// Evaluate the pre-differentiated entries at `x`, straight into the
    /// solver's reused matrix (G3b) — the evaluation half of the Java
    /// `NewtonSolver.analyticalJacobian`. Any evaluation failure answers
    /// `false`, falling back to finite differences for this iteration exactly
    /// like the Java `catch` → `return null` (the solver zero-fills the
    /// buffer again on that path, so a partial write here is harmless).
    fn analytic_jacobian_into(&mut self, x: &[f64], out: &mut [Vec<f64>]) -> bool {
        let Some(derivs) = self.derivs else {
            return false;
        };
        for (name, value) in self.names.iter().zip(x) {
            match self.scope.get_mut(name) {
                Some(slot) => *slot = *value,
                None => {
                    self.scope.insert(name.clone(), *value);
                }
            }
        }
        for row in out.iter_mut() {
            row.fill(0.0);
        }
        for (i, row) in derivs.iter().enumerate() {
            for (j, entry) in row.iter().enumerate() {
                if let Some(expr) = entry {
                    match eval_with(expr, self.scope, self.ctx) {
                        Ok(value) => out[i][j] = value,
                        Err(_) => return false,
                    }
                }
            }
        }
        true
    }
}

/// [`BlockProblem`] for a block whose every expression compiled to
/// slot-indexed postfix (Wave A2).
///
/// Same contract, same arithmetic, same evaluation order — the difference is
/// that the iterate lives in a dense `Vec<f64>` instead of the string-keyed
/// `Scope`, so a residual read costs an index rather than a hash, a probe and
/// a `bcmp`. [`crate::solver::slots`] carries the parity argument;
/// `solve_block` only ever builds one of these for a block the cache compiled,
/// and loads the slot vector out of `values` before it does.
struct SlotBlockProblem<'a> {
    compiled: &'a CompiledBlock,
    /// The block's values, indexed by `compiled.names`' slot order.
    slots: &'a mut Vec<f64>,
    /// This call's volatile literals (see
    /// [`crate::solver::slots::refresh_consts`]).
    consts: &'a [f64],
    /// The postfix evaluation stack, sized once per call and reused across
    /// every program.
    stack: &'a mut [f64],
    row_deps: &'a Vec<Vec<usize>>,
    /// As [`BlockProblem::last_property_error`]. The compiled subset cannot
    /// raise a property refusal, but the arm is kept so the two paths read the
    /// same and a widened subset stays correct.
    last_property_error: &'a mut Option<String>,
}

impl SlotBlockProblem<'_> {
    /// The counterpart of `BlockProblem`'s `get_mut`-or-insert loop: place the
    /// iterate where the compiled programs will read it.
    fn write_iterate(&mut self, x: &[f64]) {
        for (slot, value) in self.compiled.unknown_slots.iter().zip(x) {
            self.slots[*slot as usize] = *value;
        }
    }
}

impl NewtonProblem for SlotBlockProblem<'_> {
    /// [`residuals_lenient`] over compiled programs: an invalid state point
    /// poisons only its own slot with `NaN`, and the message is recorded, in
    /// exactly the same per-equation positions.
    fn residual(&mut self, x: &[f64], out: &mut [f64]) -> Result<()> {
        self.write_iterate(x);
        for (slot, program) in out.iter_mut().zip(&self.compiled.residuals) {
            let residual = slotexpr::eval_program(program, self.slots, self.consts, self.stack);
            match residual {
                Ok(value) => *slot = value,
                Err(err @ (FreesError::Evaluation { .. } | FreesError::Property { .. })) => {
                    *slot = f64::NAN;
                    *self.last_property_error = Some(err.to_string_message());
                }
                Err(other) => return Err(other),
            }
        }
        Ok(())
    }

    fn row_dependencies(&self) -> Option<Vec<Vec<usize>>> {
        Some(self.row_deps.clone())
    }

    fn analytic_jacobian_into(&mut self, x: &[f64], out: &mut [Vec<f64>]) -> bool {
        let Some(derivs) = &self.compiled.derivs else {
            return false;
        };
        for (slot, value) in self.compiled.unknown_slots.iter().zip(x) {
            self.slots[*slot as usize] = *value;
        }
        for row in out.iter_mut() {
            row.fill(0.0);
        }
        for (i, row) in derivs.iter().enumerate() {
            for (j, entry) in row.iter().enumerate() {
                if let Some(program) = entry {
                    match slotexpr::eval_program(program, self.slots, self.consts, self.stack) {
                        Ok(value) => out[i][j] = value,
                        Err(_) => return false,
                    }
                }
            }
        }
        true
    }
}

/// Pre-differentiate every dependent (equation, variable) pair of a block —
/// the symbolic half of the Java `NewtonSolver.analyticalJacobian`: residual
/// `lhs − rhs` differentiated w.r.t. each block variable, entries skipped for
/// equations that do not mention the variable (they stay structural zeros).
/// Returns `None` — no analytic source at all — as soon as any dependent
/// entry cannot be differentiated, the Java all-or-nothing fallback.
fn analytic_derivatives(
    block_equations: &[&Equation],
    variables: &[String],
) -> Option<Vec<Vec<Option<Expr>>>> {
    let n = variables.len();
    let mut derivs: Vec<Vec<Option<Expr>>> = vec![vec![None; n]; n];
    for (i, equation) in block_equations.iter().enumerate() {
        let mentioned = equation.variables();
        let residual_expr = equation.residual();
        for (j, var) in variables.iter().enumerate() {
            if !mentioned.contains(var) {
                continue; // structural zero — Java never differentiates these
            }
            {
                let d = differentiate(&residual_expr, var)?;
                derivs[i][j] = Some(d)
            }
        }
    }
    Some(derivs)
}

/// Solve one block in place, leaving its unknowns' values in `values`.
///
/// Returns the Newton iteration count. This is one rung's worth of work — the
/// Java `NewtonSolver.solveBlock`; the ladder around it lives in
/// [`solve_block_with_fallback`].
#[allow(clippy::too_many_arguments)]
fn solve_block(
    index: usize,
    block: &Block,
    equations: &[Equation],
    values: &mut Scope,
    settings: &SolverSettings,
    specs: &BTreeMap<String, VarSpec>,
    ctx: EvalContext<'_>,
    cache: Option<&PinnedBlockStruct>,
    mut dense: Option<&mut DenseRun<'_>>,
) -> Result<usize> {
    let n = block.variables.len();
    if n == 0 {
        return Ok(0);
    }
    // Wave Q3: the compacted `Vec<&Equation>` this used to materialise on every
    // call was the single largest allocation site in the whole per-step path —
    // 176 448 of the 381 050 blocks a `hot-transient` run allocates, one per
    // `solve_block` call at each of `solve_block_with_fallback`'s two call
    // sites. It is only *needed* on the `Expr`/`Scope` fallback and in
    // diagnostics, so it is built lazily by `block_equations()` below and the
    // compiled path indexes `equations` directly.
    //
    // The precondition it used to carry is checked here without allocating,
    // and reports the same count in the same message.
    let resolved = block
        .equations
        .iter()
        .filter(|&&i| i < equations.len())
        .count();
    if resolved != n {
        // The blocker guarantees this; a mismatch means the two modules
        // disagree, which must be loud rather than mysterious.
        return Err(FreesError::solver(format!(
            "internal error: block {} has {} equations for {} unknowns",
            index + 1,
            resolved,
            n
        )));
    }
    // `filter_map` *drops* an out-of-range index, so compacted position `p`
    // equals `block.equations[p]` only when nothing was dropped. With
    // `resolved == n` already established, that is exactly
    // `block.equations.len() == n`. The blocker never produces anything else;
    // when it somehow does, the compiled path is simply skipped, which is
    // observationally identical to taking it (see [`crate::solver::slots`]).
    let aligned = block.equations.len() == n;
    let block_equations = || -> Vec<&Equation> {
        block
            .equations
            .iter()
            .filter_map(|&i| equations.get(i))
            .collect()
    };

    // Evaluate once at the initial point so a genuinely broken expression is
    // reported as itself instead of as "did not converge". See the module docs.
    //
    // A *property* refusal is the one exception, and excluding it is what the
    // oracle does: Java has no fatal probe at all — `NewtonSolver` line 83
    // enters the loop through the same NaN-tolerant `residuals()` as every
    // later iteration. The default guess is 1 Pa / 1 J/kg / 1 K, which lies
    // outside every real fluid's table, so a refusal there says nothing about
    // whether the block is solvable; only entering Newton lets the line search
    // and the LM rescue walk out. The message is kept and re-attached below,
    // so a block that genuinely cannot be served (an untabulated output, an
    // unknown fluid) still reports its cause rather than "did not converge".
    // Wave I: the per-call vectors (`probe`, `x`) and the Newton workspace
    // come from the per-block cache when the caller has one, so a warm
    // per-step solve allocates nothing here. Contents never carry over:
    // `probe` is a write-only sink and `x` is rebuilt from `values` below.
    let mut probe_local: Vec<f64>;
    let mut x_local: Vec<f64>;
    let mut block_scratch = cache.map(|c| c.scratch.borrow_mut());
    type SlotBufs<'s> = Option<(&'s mut Vec<f64>, &'s mut Vec<f64>, &'s mut Vec<f64>)>;
    let (x, probe, ws, slot_bufs): (
        &mut Vec<f64>,
        &mut Vec<f64>,
        Option<&mut NewtonWorkspace>,
        SlotBufs<'_>,
    ) = match block_scratch.as_deref_mut() {
        Some(BlockScratch {
            ws,
            x,
            probe,
            slots,
            consts,
            stack,
        }) => {
            probe.clear();
            probe.resize(n, 0.0);
            (x, probe, Some(ws), Some((slots, consts, stack)))
        }
        None => {
            probe_local = vec![0.0; n];
            x_local = Vec::new();
            (&mut x_local, &mut probe_local, None, None)
        }
    };

    // Wave A2: take the slot-indexed path when the cache compiled this block
    // *and* this call's expressions still match what it compiled — the literal
    // walk and the name resolution are both checked here, and either failing
    // falls back to the `Expr`/`Scope` evaluator below with nothing observable
    // changed (only scratch buffers have been touched). See
    // [`crate::solver::slots`].
    /// The compiled block plus the three buffers its evaluation needs: the
    /// slot vector, this call's volatile literals, and the postfix stack.
    type SlotState<'s> = Option<(
        &'s CompiledBlock,
        &'s mut Vec<f64>,
        &'s mut Vec<f64>,
        &'s mut [f64],
    )>;
    let mut slot_state: SlotState<'_> = None;
    // Wave R3: set when the slot vector was filled from the *dense* values,
    // which is also what says the answer must be written back there.
    let mut dense_block: Option<&BlockDense> = None;
    if let (true, Some(compiled), Some((slots, consts, stack))) = (
        aligned,
        cache.and_then(|cached| cached.compiled.as_ref()),
        slot_bufs,
    ) {
        if slotexpr::refresh_consts(equations, &block.equations, &compiled.volatile, consts)
            && consts.len() == compiled.const_count
        {
            // The dense gather cannot fail where `fill_slots` could: the plan
            // resolved every one of these names when it was built, and it is
            // built from the very map `fill_slots` probes. That equivalence is
            // the whole reason `dense_plan` refuses a prep with an unresolved
            // name instead of leaving the slot unfilled.
            let filled = match dense.as_deref() {
                Some(run) if run.live => match cache.and_then(|cached| cached.dense.as_ref()) {
                    Some(block_dense) => {
                        slots.clear();
                        slots.extend(
                            block_dense
                                .fill
                                .iter()
                                .map(|&slot| run.values[slot as usize]),
                        );
                        dense_block = Some(block_dense);
                        true
                    }
                    None => fill_slots(&compiled.names, values, slots),
                },
                _ => fill_slots(&compiled.names, values, slots),
            };
            if filled {
                if stack.len() < compiled.depth {
                    stack.resize(compiled.depth, 0.0);
                }
                slot_state = Some((compiled, slots, consts, stack.as_mut_slice()));
            }
        }
    }
    // Everything from here on reads `values` — the fallback evaluator, the
    // diagnostics, the retry ladder above this frame — so a dense run that did
    // not get to drive this block gives up the authority now, while the two
    // are still in agreement.
    if dense_block.is_none() {
        if let Some(run) = dense.as_deref_mut() {
            if run.live {
                run.hand_over(values);
            }
        }
    }

    // The fallback evaluator, the analytic derivatives and `BlockProblem` all
    // borrow the compacted list, so it is materialised once here when the
    // compiled path did not take. `Vec::new()` does not allocate, so a
    // compiled block still pays nothing for it.
    let fallback_equations: Vec<&Equation> = match slot_state {
        Some(_) => Vec::new(),
        None => block_equations(),
    };

    let mut last_property_error: Option<String> = None;
    let probe_result = match &mut slot_state {
        Some((compiled, slots, consts, stack)) => {
            slot_residuals_into(compiled, slots, consts, stack, probe)
        }
        None => residuals_into(&fallback_equations, values, ctx, probe),
    };
    match probe_result {
        Ok(()) => {}
        Err(err @ FreesError::Property { .. }) => {
            last_property_error = Some(err.to_string_message());
        }
        // Diagnostics only, and only on the way out: build the list rather
        // than carry an unused one through every successful call.
        Err(other) => return Err(annotate(other, index, &block_equations())),
    }

    x.clear();
    match &slot_state {
        // The slot vector was filled from `values` moments ago and the probe
        // does not write it, so these are the same numbers the string-keyed
        // arm reads — and `fill_slots` refused the call outright if a name was
        // missing, which is the only case `DEFAULT_GUESS` covers.
        Some((compiled, slots, _, _)) => x.extend(
            compiled
                .unknown_slots
                .iter()
                .map(|slot| slots[*slot as usize]),
        ),
        None => x.extend(
            block
                .variables
                .iter()
                .map(|name| values.get(name).copied().unwrap_or(DEFAULT_GUESS)),
        ),
    }

    // Per-variable bounds from the specs — the Java IterationContext lo/hi
    // arrays, threaded into all three clamp sites inside `newton_solve`.
    // Borrowed from the per-step cache when the caller has one (G3).
    let bounds_local: Vec<(f64, f64)>;
    let bounds: &[(f64, f64)] = match cache {
        Some(cached) => &cached.bounds,
        None => {
            bounds_local = block
                .variables
                .iter()
                .map(|name| {
                    specs
                        .get(name)
                        .map(|spec| (spec.lower, spec.upper))
                        .unwrap_or((f64::NEG_INFINITY, f64::INFINITY))
                })
                .collect();
            &bounds_local
        }
    };

    let names = &block.variables;
    let derivs_local: Option<Vec<Vec<Option<Expr>>>>;
    let derivs: Option<&Vec<Vec<Option<Expr>>>> = match cache {
        Some(cached) => cached.derivs.as_ref(),
        None => {
            derivs_local = analytic_derivatives(&fallback_equations, names);
            derivs_local.as_ref()
        }
    };
    let outcome = match slot_state {
        // Wave A2. A compiled block always came from the cache, so the
        // workspace and the row dependencies are always there.
        Some((compiled, slots, consts, stack)) => {
            let problem = SlotBlockProblem {
                compiled,
                slots,
                consts: consts.as_slice(),
                stack,
                row_deps: &cache.expect("a compiled block is a cached block").row_deps,
                last_property_error: &mut last_property_error,
            };
            let ws = ws.expect("a compiled block is a cached block");
            newton_solve_problem_prepared(problem, x, settings, Some(bounds), ws)
        }
        None => {
            let problem = BlockProblem {
                names,
                equations: &fallback_equations,
                scope: &mut *values,
                ctx,
                derivs,
                row_deps: cache.map(|cached| &cached.row_deps),
                last_property_error: &mut last_property_error,
            };
            match ws {
                // The cached path hands Newton this block's cross-call buffers;
                // the fresh path is the old allocate-per-call entry, verbatim.
                Some(ws) => newton_solve_problem_prepared(problem, x, settings, Some(bounds), ws),
                None => newton_solve_problem(problem, x, settings, Some(bounds)),
            }
        }
    };

    // Write back before propagating: on failure the last iterate is what makes
    // a stall report actionable, and it is what the Java engine leaves behind.
    // Wave R3: into whichever of the two drove this block. `BlockDense::write`
    // is `block.variables` resolved to indices, so this is the same loop over
    // the same pairs with the probe removed.
    match dense_block {
        Some(block_dense) => {
            let run = dense.expect("a dense fill implies a live dense run");
            for (&slot, value) in block_dense.write.iter().zip(x.iter()) {
                run.values[slot as usize] = *value;
            }
        }
        None => {
            for (name, value) in names.iter().zip(x.iter()) {
                match values.get_mut(name) {
                    Some(slot) => *slot = *value,
                    None => {
                        values.insert(name.clone(), *value);
                    }
                }
            }
        }
    }

    let report = outcome.map_err(|err| {
        // The Java `propertyErrorSuffix`: a stall whose residuals were `NaN`
        // is unreadable without the property refusal that made them `NaN`.
        let err = match &last_property_error {
            Some(message) => FreesError::solver(format!("{} {message}", err.to_string_message())),
            None => err,
        };
        annotate(err, index, &block_equations())
    })?;
    Ok(report.iterations)
}

/// `out[k] = lhs_k(x) - rhs_k(x)` for each equation in the block, evaluated
/// with the document's definitions in context so user `FUNCTION`s and
/// `TABLE`s resolve (the Java `Evaluator.eval(expr, values, defs)` triple).
fn residuals_into(
    equations: &[&Equation],
    scope: &Scope,
    ctx: EvalContext<'_>,
    out: &mut [f64],
) -> Result<()> {
    for (slot, equation) in out.iter_mut().zip(equations) {
        *slot = eval_with(&equation.lhs, scope, ctx)? - eval_with(&equation.rhs, scope, ctx)?;
    }
    Ok(())
}

/// Fill `slots` with the compiled block's variables, in slot order.
///
/// Answers `false` — and the caller keeps the ordinary path — the moment a
/// name is not in `values`. That is what makes the compiled arm's `x` exactly
/// the string-keyed arm's `x`: `DEFAULT_GUESS` and the `lookup_constant`
/// fallback inside [`crate::eval::eval_in`] are the only behaviours a missing
/// name could have triggered, and neither can be reached from here.
fn fill_slots(names: &[String], values: &Scope, slots: &mut Vec<f64>) -> bool {
    slots.clear();
    for name in names {
        match values.get(name) {
            Some(value) => slots.push(*value),
            None => return false,
        }
    }
    true
}

/// [`residuals_into`] over compiled programs — same order, same first error.
fn slot_residuals_into(
    compiled: &CompiledBlock,
    slots: &[f64],
    consts: &[f64],
    stack: &mut [f64],
    out: &mut [f64],
) -> Result<()> {
    for (slot, program) in out.iter_mut().zip(&compiled.residuals) {
        *slot = slotexpr::eval_program(program, slots, consts, stack)?;
    }
    Ok(())
}

/// [`residuals_into`], but an invalid *state point* poisons only **its own**
/// slot with `NaN` instead of the whole vector — the Java
/// `NewtonSolver.residuals`, whose `catch (PropertyEvaluationException)` sits
/// inside the per-equation loop and writes `result[i] = NaN`.
///
/// The distinction is load-bearing at scale. One out-of-table property call in
/// a 79-equation block used to blank all 79 residuals, and the LM rescue's
/// row-skipping (`newton::damped_rescue`) then had no finite row left to build
/// `JᵀJ` from, so the block stalled where Java walks out of the bad region on
/// the other 78. `ev-battery-cooling-pid` is the document where that shows.
///
/// `Evaluation` failures are folded in for the reason the module docs give:
/// [`crate::eval`] raises domain errors (`ln` of a non-positive, `sqrt` of a
/// negative) that the Java `Evaluator` answers with `NaN` outright, so
/// declining them here would be stricter than the oracle inside the very loop
/// that is supposed to probe invalid regions.
///
/// The last such message is recorded in `last_error` — the Java
/// `lastPropertyError` field, which exists so a stall report can still name
/// the physical cause behind a vector of `NaN`s.
fn residuals_lenient(
    equations: &[&Equation],
    scope: &Scope,
    ctx: EvalContext<'_>,
    out: &mut [f64],
    last_error: &mut Option<String>,
) -> Result<()> {
    for (slot, equation) in out.iter_mut().zip(equations) {
        let residual = eval_with(&equation.lhs, scope, ctx)
            .and_then(|lhs| Ok(lhs - eval_with(&equation.rhs, scope, ctx)?));
        match residual {
            Ok(value) => *slot = value,
            // An invalid region, not a broken document: hand Newton the
            // non-finite residual its line search knows how to reject.
            Err(err @ (FreesError::Evaluation { .. } | FreesError::Property { .. })) => {
                *slot = f64::NAN;
                *last_error = Some(err.to_string_message());
            }
            Err(other) => return Err(other),
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// The retry ladder — port of EquationSystemSolver.solveBlockWithFallback
// ---------------------------------------------------------------------------

/// Retry attempts cap iterations: healthy Newton converges quickly, and a
/// generous user limit must not multiply across the whole ladder
/// (`EquationSystemSolver.MAX_RETRY_ITERATIONS`).
const MAX_RETRY_ITERATIONS: usize = 500;

/// Relative tolerance for accepting a bracketed root
/// (`EquationSystemSolver.BRACKET_RESIDUAL_TOL`): `|resid| / max(|lhs|, 1)`.
const BRACKET_RESIDUAL_TOL: f64 = 1.0e-6;

/// `EquationSystemSolver.retrySettings`: the ladder's retries run with the
/// user's stop criteria but the iteration budget capped at
/// [`MAX_RETRY_ITERATIONS`].
fn retry_settings(settings: &SolverSettings) -> SolverSettings {
    SolverSettings {
        max_iterations: settings.max_iterations.min(MAX_RETRY_ITERATIONS),
        ..*settings
    }
}

/// `EquationSystemSolver.polishSettings`: near-zero residual tolerance so the
/// polisher keeps iterating until the variable change drops below 1e-15
/// (`newton`'s `CHANGE_IN_VARIABLES`). This is critical for multiple roots,
/// where residual ≈ error^m drops below tolerance long before the variable
/// has converged. The Java record is `(50, 1e-30, 1e-15, elapsed, complex)`.
fn polish_settings(settings: &SolverSettings) -> SolverSettings {
    SolverSettings {
        max_iterations: 50,
        rel_tolerance: 1e-30,
        // Inert (strictly below every reachable `rel_tolerance * scale`), as
        // the Java criterion is purely relative.
        abs_tolerance: 0.0,
        ..*settings
    }
}

/// One alternative start for a failed block —
/// `EquationSystemSolver.GuessTransform`: a zero guess becomes ±`zero_offset`
/// (off the invariant manifold), a nonzero guess is rescaled (toward a distant
/// Newton basin), `conjugate` flips imaginary components (`_i`), and `jitter`
/// staggers each variable by its position to break exchange symmetries.
struct GuessTransform {
    zero_offset: f64,
    scale: f64,
    conjugate: bool,
    jitter: bool,
}

/// `EquationSystemSolver.buildGuessTransforms`, in the Java order: the 20
/// uniform transforms (conjugate × scale × zero-offset), then the 6
/// symmetry-breaking jitter variants.
fn guess_transforms() -> Vec<GuessTransform> {
    let mut transforms = Vec::with_capacity(26);
    for conjugate in [false, true] {
        for scale in [1.0, 1.0e-2, 1.0e-4, 1.0e2, 1.0e4] {
            for zero_offset in [1.0, -1.0] {
                transforms.push(GuessTransform {
                    zero_offset,
                    scale,
                    conjugate,
                    jitter: false,
                });
            }
        }
    }
    // Symmetry-breaking variants last: a system symmetric under a variable
    // exchange (x <-> y) traps every symmetric iteration on the invariant
    // manifold (identical Jacobian columns there), and the uniform transforms
    // above preserve that symmetry. Staggering each variable by its position
    // in the block is deterministic and leaves the manifold.
    for scale in [1.0, 1.0e-2, 1.0e2] {
        for zero_offset in [1.0, -1.0] {
            transforms.push(GuessTransform {
                zero_offset,
                scale,
                conjugate: false,
                jitter: true,
            });
        }
    }
    transforms
}

/// The initial guess a variable restarts from —
/// `EquationSystemSolver.initialGuess` without the warm-start map (no warm
/// starts in this port yet): the spec's guess clamped into its bounds, or
/// [`DEFAULT_GUESS`].
fn initial_guess(name: &str, specs: &BTreeMap<String, VarSpec>) -> f64 {
    specs
        .get(name)
        .map(|spec| spec.initial())
        .unwrap_or(DEFAULT_GUESS)
}

/// `EquationSystemSolver.applyTransform`: rewrite every block variable's value
/// from its *initial* guess (not the stalled iterate) through the transform,
/// clamped into the variable's bounds.
fn apply_transform(
    block: &Block,
    transform: &GuessTransform,
    values: &mut Scope,
    specs: &BTreeMap<String, VarSpec>,
) {
    for (position, name) in block.variables.iter().enumerate() {
        let base = initial_guess(name, specs);
        let mut guess = if base == 0.0 {
            transform.zero_offset
        } else {
            base * transform.scale
        };
        if transform.conjugate && name.ends_with("_i") {
            guess = -guess;
        }
        if transform.jitter {
            // Java: `guess *= 1.0 + 0.07 * ++position` — 1-based stagger.
            guess *= 1.0 + 0.07 * (position + 1) as f64;
        }
        if let Some(spec) = specs.get(name) {
            guess = guess.clamp(spec.lower, spec.upper);
        }
        values.insert(name.clone(), guess);
    }
}

/// **Ladder rung 1** — `EquationSystemSolver.retryWithTransformedGuesses`:
/// retry the failed block alone from each transformed start, with the
/// iteration-capped retry settings. Returns the iterations of the first
/// attempt that converges; on total failure restores the block's variables to
/// their unmodified initial guesses (so later rungs — and the failure
/// diagnostics — start from the guesses, exactly as the Java does) and
/// returns `None`.
#[allow(clippy::too_many_arguments)]
fn retry_with_transformed_guesses(
    index: usize,
    block: &Block,
    equations: &[Equation],
    values: &mut Scope,
    settings: &SolverSettings,
    specs: &BTreeMap<String, VarSpec>,
    ctx: EvalContext<'_>,
    cache: Option<&PinnedBlockStruct>,
) -> Option<usize> {
    let relaxed = retry_settings(settings);
    for transform in guess_transforms() {
        apply_transform(block, &transform, values, specs);
        if let Ok(iterations) =
            // Wave R3: the ladder runs on the `Scope`, which by now holds the
            // dense values (`solve_block_with_fallback` handed them over before
            // the first rung).
            solve_block(
                index, block, equations, values, &relaxed, specs, ctx, cache, None,
            )
        {
            return Some(iterations);
        }
    }
    for name in &block.variables {
        values.insert(name.clone(), initial_guess(name, specs));
    }
    None
}

/// True if either side of the equation contains a CoolProp `prop$` call —
/// `EquationSystemSolver.usesPropertyCall`. The recursion mirrors the Java
/// `exprUsesPropertyCall` exactly: `Call` (name prefix, then arguments),
/// `BinOp`, `Neg`; every other node answers false.
fn uses_property_call(equation: &Equation) -> bool {
    expr_uses_property_call(&equation.lhs) || expr_uses_property_call(&equation.rhs)
}

fn expr_uses_property_call(expr: &Expr) -> bool {
    match expr {
        Expr::Call { function, args } => {
            function.starts_with("prop$") || args.iter().any(expr_uses_property_call)
        }
        Expr::BinOp { left, right, .. } => {
            expr_uses_property_call(left) || expr_uses_property_call(right)
        }
        Expr::Neg(operand) => expr_uses_property_call(operand),
        _ => false,
    }
}

/// **Ladder rung 2** — `EquationSystemSolver.tryUnivariateBracketingSolve`:
/// a bracketing root-find for a one-equation, one-unknown block that Newton
/// could not solve. Newton needs a non-zero gradient; an implicit property
/// inversion crossing the two-phase dome is flat there (`dT/dh ≈ 0`) yet the
/// overall residual is monotonic and sign-changing (§8.7), so a sign-bracketed
/// bisection crosses the plateau where Newton stalls. Scoped to property
/// inversions (the `prop$` gate) exactly as the Java is — for ordinary algebra
/// a bracketing rescue would bypass the user's iteration-limit stop criterion
/// and could pick a different root than Newton's basin — which makes this rung
/// inert until CoolProp lands, but ready. The root is committed only if it
/// drives the residual within [`BRACKET_RESIDUAL_TOL`], so a wrong root can
/// never be silently accepted. Returns the work done on success, `None` when
/// no valid bracket/root was found (with the variable restored).
fn try_univariate_bracketing_solve(
    block: &Block,
    equations: &[Equation],
    values: &mut Scope,
    specs: &BTreeMap<String, VarSpec>,
    ctx: EvalContext<'_>,
) -> Option<usize> {
    if block.variables.len() != 1 || block.equations.len() != 1 {
        return None;
    }
    let equation = equations.get(block.equations[0])?;
    if !uses_property_call(equation) {
        return None;
    }
    let name = &block.variables[0];
    let (lower, upper) = specs
        .get(name)
        .map(|spec| (spec.lower, spec.upper))
        .unwrap_or((f64::NEG_INFINITY, f64::INFINITY));

    let saved = values.get(name).copied().unwrap_or(DEFAULT_GUESS);
    let x0 = if saved.is_finite() {
        saved
    } else {
        specs
            .get(name)
            .map(|spec| spec.guess)
            .filter(|guess| guess.is_finite())
            .unwrap_or(1.0)
    }
    .clamp(lower, upper);

    // Residual of the single equation with var = x; NaN inside invalid regions.
    let f = |x: f64, values: &mut Scope| -> f64 {
        values.insert(name.clone(), x);
        match (
            eval_with(&equation.lhs, values, ctx),
            eval_with(&equation.rhs, values, ctx),
        ) {
            (Ok(lhs), Ok(rhs)) => lhs - rhs,
            _ => f64::NAN,
        }
    };

    // Sample x0 plus geometrically growing symmetric offsets (clamped to the
    // box); keep only finite (in-range) evaluations. (The Java loop also
    // checks its elapsed-time deadline here; wasm32 has no clock, so the
    // sample count — ~69 evaluations — is the budget.)
    let magnitude = x0.abs().max(1.0);
    let mut samples: Vec<(f64, f64)> = Vec::new();
    let record = |x: f64, samples: &mut Vec<(f64, f64)>, values: &mut Scope| {
        if samples.iter().any(|&(seen, _)| seen == x) {
            return;
        }
        let value = f(x, values);
        if value.is_finite() {
            samples.push((x, value));
        }
    };
    record(x0, &mut samples, values);
    let mut multiplier = 0.125;
    while multiplier <= 1.0e9 {
        let step = magnitude * multiplier;
        record((x0 + step).clamp(lower, upper), &mut samples, values);
        record((x0 - step).clamp(lower, upper), &mut samples, values);
        multiplier *= 2.0;
    }
    samples.sort_by(|a, b| a.0.total_cmp(&b.0));

    // Pick the adjacent finite-sample pair straddling zero whose midpoint is
    // nearest x0 (bias toward the local root).
    let mut bracket: Option<(f64, f64)> = None;
    let mut best_distance = f64::INFINITY;
    for pair in samples.windows(2) {
        let (xa, fa) = pair[0];
        let (xb, fb) = pair[1];
        if fa * fb < 0.0 {
            let distance = (0.5 * (xa + xb) - x0).abs();
            if distance < best_distance {
                best_distance = distance;
                bracket = Some((xa, xb));
            }
        }
    }
    let Some((mut a, mut b)) = bracket else {
        values.insert(name.clone(), saved);
        return None;
    };

    // Bisection on the bracket to a tight relative width.
    let mut fa = f(a, values);
    let mut iterations = 0usize;
    while iterations < 200 {
        iterations += 1;
        let c = 0.5 * (a + b);
        let fc = f(c, values);
        if !fc.is_finite() {
            break;
        }
        if fc == 0.0 || (b - a) <= 1.0e-12 * c.abs().max(1.0) {
            a = c;
            b = c;
            break;
        }
        if fa * fc < 0.0 {
            b = c;
        } else {
            a = c;
            fa = fc;
        }
    }

    // Validate the root against the residual before committing.
    let root = 0.5 * (a + b);
    values.insert(name.clone(), root);
    match (
        eval_with(&equation.lhs, values, ctx),
        eval_with(&equation.rhs, values, ctx),
    ) {
        (Ok(lhs), Ok(rhs)) => {
            let residual = lhs - rhs;
            let scale = lhs.abs().max(1.0);
            if residual.is_finite() && residual.abs() / scale <= BRACKET_RESIDUAL_TOL {
                return Some(iterations + 1);
            }
            values.insert(name.clone(), saved);
            None
        }
        _ => {
            values.insert(name.clone(), saved);
            None
        }
    }
}

/// **Ladder rung 3** — `EquationSystemSolver.tryMergeBidirectional`: merge the
/// failed block with blocks that share variable dependencies, scanning forward
/// then backward, skipping blocks a previous merge already solved. Returns the
/// merged block and the indices it swallowed, or `None` when nothing merges.
fn try_merge_bidirectional(
    blocks: &[Block],
    failed_index: usize,
    equations: &[Equation],
    skip: &HashSet<usize>,
) -> Option<(Block, Vec<usize>)> {
    let failed = &blocks[failed_index];
    let mut involved_vars: HashSet<String> = HashSet::new();
    let mut merged_equations = failed.equations.clone();
    let mut merged_vars = failed.variables.clone();
    let mut merged_var_set: HashSet<String> = failed.variables.iter().cloned().collect();
    for &equation_index in &failed.equations {
        if let Some(equation) = equations.get(equation_index) {
            involved_vars.extend(equation.variables());
        }
    }
    let mut merged_indices = Vec::new();

    let consider = |candidate_index: usize,
                    merged_equations: &mut Vec<usize>,
                    merged_vars: &mut Vec<String>,
                    merged_var_set: &mut HashSet<String>,
                    involved_vars: &mut HashSet<String>,
                    merged_indices: &mut Vec<usize>| {
        let candidate = &blocks[candidate_index];
        if !should_merge(candidate, equations, involved_vars, merged_var_set) {
            return;
        }
        // Java addBlock: absorb the candidate's equations, variables, and
        // every variable its equations reference.
        merged_equations.extend(candidate.equations.iter().copied());
        merged_vars.extend(candidate.variables.iter().cloned());
        merged_var_set.extend(candidate.variables.iter().cloned());
        for &equation_index in &candidate.equations {
            if let Some(equation) = equations.get(equation_index) {
                involved_vars.extend(equation.variables());
            }
        }
        merged_indices.push(candidate_index);
    };

    // Scan forward: blocks whose variables are referenced by the merged block,
    // or whose equations reference merged variables.
    for candidate_index in failed_index + 1..blocks.len() {
        if skip.contains(&candidate_index) {
            continue;
        }
        consider(
            candidate_index,
            &mut merged_equations,
            &mut merged_vars,
            &mut merged_var_set,
            &mut involved_vars,
            &mut merged_indices,
        );
    }
    // Scan backward: earlier blocks whose variables the failed block's
    // equations reference, or that reference our variables.
    for candidate_index in (0..failed_index).rev() {
        if skip.contains(&candidate_index) {
            continue;
        }
        consider(
            candidate_index,
            &mut merged_equations,
            &mut merged_vars,
            &mut merged_var_set,
            &mut involved_vars,
            &mut merged_indices,
        );
    }

    if merged_indices.is_empty() {
        return None;
    }
    Some((
        Block {
            equations: merged_equations,
            variables: merged_vars,
        },
        merged_indices,
    ))
}

/// `EquationSystemSolver.shouldMerge`: the candidate determines a variable the
/// merged block's equations reference, or its equations reference a merged
/// variable.
fn should_merge(
    candidate: &Block,
    equations: &[Equation],
    involved_vars: &HashSet<String>,
    merged_var_set: &HashSet<String>,
) -> bool {
    if candidate
        .variables
        .iter()
        .any(|name| involved_vars.contains(name))
    {
        return true;
    }
    candidate.equations.iter().any(|&equation_index| {
        equations.get(equation_index).is_some_and(|equation| {
            equation
                .variables()
                .iter()
                .any(|v| merged_var_set.contains(v))
        })
    })
}

/// Solve one block through the whole Java retry ladder — the port of
/// `EquationSystemSolver.solveBlockWithFallback`, rung by rung:
///
/// 1. the plain Newton solve (`config.newton().solveBlock`);
/// 2. on failure, **transformed-guess retries** — local and cheap, keeping all
///    other blocks' solutions intact (`retryWithTransformedGuesses`);
/// 3. then the **univariate bracketing rescue** for one-unknown property
///    inversions (`tryUnivariateBracketingSolve`);
/// 4. then **bidirectional block merging**: re-solve the combined system from
///    scratch — all merged variables reset to their initial guesses, because
///    previously solved blocks may carry incorrect values from the
///    rank-deficient/least-squares path — and mark the swallowed blocks
///    solved (`tryMergeBidirectional` + `skipIndices`); a merge that itself
///    fails to converge propagates *its* error, while an impossible merge
///    rethrows the original one;
/// 5. finally a best-effort **polish pass** over whatever was solved
///    (`polishSettings`) — its iterations count only when it converges, and
///    its failure is ignored (the main solution is still valid), exactly like
///    the Java `catch (SolverException ignored)`.
#[allow(clippy::too_many_arguments)]
fn solve_block_with_fallback(
    index: usize,
    blocks: &[Block],
    equations: &[Equation],
    values: &mut Scope,
    settings: &SolverSettings,
    specs: &BTreeMap<String, VarSpec>,
    ctx: EvalContext<'_>,
    skip: &mut HashSet<usize>,
    cache: Option<&PinnedBlockCache>,
    mut dense: Option<&mut DenseRun<'_>>,
) -> Result<usize> {
    if let Some(message) = crate::ode::deadline::strike() {
        return Err(FreesError::solver(message));
    }
    let block = &blocks[index];
    let mut actual_solved: Option<Block> = None; // None = the original block
    let mut iterations = 0usize;
    // The cache entry is valid for `blocks[index]` and only that — the merge
    // rescue's synthetic blocks below compute fresh, as they always did.
    let block_cache = cache.map(|c| &c.per_block[index]);

    match solve_block(
        index,
        block,
        equations,
        values,
        settings,
        specs,
        ctx,
        block_cache,
        dense.as_deref_mut(),
    ) {
        Ok(count) => iterations += count,
        Err(first_error) => {
            // A struck wall-clock budget is terminal, and the ladder below must
            // not run on it. The deadline is monotone, so no rung can succeed —
            // but a rung *can* fail differently, and the merge rescue's `?`
            // propagates its own error, which is how a 60-second transient
            // budget used to surface as "Newton iteration stalled … unable to
            // bracket the (p,X) solution" from a re-run started at transformed
            // guesses. Reporting the budget directly is both honest and ~2.4 s
            // cheaper. Native callers install no deadline, so this is `None`
            // there and the ladder is untouched.
            if let Some(message) = crate::ode::deadline::strike() {
                return Err(FreesError::solver(message));
            }
            // Wave R3: the whole ladder below reads and writes the `Scope` —
            // including the last iterate `solve_block` just wrote — so the
            // dense vector hands over here, before the first of them looks.
            // `solve_block` has already written that iterate into whichever of
            // the two it was driving, so the hand-over reproduces exactly the
            // map the string-keyed path would present at this point.
            if let Some(run) = dense.as_deref_mut() {
                if run.live {
                    run.hand_over(values);
                }
            }
            if let Some(count) = retry_with_transformed_guesses(
                index,
                block,
                equations,
                values,
                settings,
                specs,
                ctx,
                block_cache,
            ) {
                iterations += count;
            } else if let Some(count) =
                try_univariate_bracketing_solve(block, equations, values, specs, ctx)
            {
                iterations += count;
            } else {
                let merged = try_merge_bidirectional(blocks, index, equations, skip)
                    .filter(|(merged, _)| merged.variables.len() > block.variables.len());
                match merged {
                    Some((merged, merged_indices)) => {
                        // Reset ALL variables in the merged block to initial
                        // guesses: previously solved blocks may have incorrect
                        // values from the rank-deficient fallback.
                        for name in &merged.variables {
                            values.insert(name.clone(), initial_guess(name, specs));
                        }
                        // A merge that fails to converge propagates its own
                        // error (Java: the uncaught `config.newton().solveBlock`
                        // inside the catch block).
                        iterations += solve_block(
                            index, &merged, equations, values, settings, specs, ctx, None, None,
                        )?;
                        skip.extend(merged_indices);
                        actual_solved = Some(merged);
                    }
                    None => return Err(first_error),
                }
            }
        }
    }

    // Polish pass — best-effort; the main solution is still valid if it fails.
    let polish = polish_settings(settings);
    let (polished_block, polish_cache) = match actual_solved.as_ref() {
        Some(merged) => (merged, None),
        None => (block, block_cache),
    };
    if let Ok(count) = solve_block(
        index,
        polished_block,
        equations,
        values,
        &polish,
        specs,
        ctx,
        polish_cache,
        // A merge rescue solved a *different* block, whose variables the dense
        // plan does not describe; the hand-over above has already happened in
        // that case, so this is `None` for the same reason `polish_cache` is.
        if actual_solved.is_some() { None } else { dense },
    ) {
        iterations += count;
    }
    Ok(iterations)
}

// ---------------------------------------------------------------------------
// The block loop, shared by the main solve and the Integral driver
// ---------------------------------------------------------------------------

/// What a completed inner solve leaves behind — the Java
/// `EquationSystemSolver.InnerSolve` minus its `blocks`, which no caller of
/// [`solve_equation_list`] needs.
struct InnerSolve {
    /// Every variable of the system at the solution, plus the built-in
    /// constants that were seeded so the evaluator could read them.
    values: Scope,
    iterations: usize,
}

/// A block that gave up, with the index it happened at and the work done
/// before it (the Java `SolverException.FailureState`, minus the copies of the
/// blocks and values the caller already holds).
struct BlockLoopFailure {
    error: FreesError,
    failed_block_index: usize,
    iterations: usize,
}

/// Solve every block in order through the retry ladder, writing each block's
/// answer into `values` as it goes (that *is* the "feed knowns forward"
/// mechanism). The Java `solveEquationList`'s block loop, factored out so the
/// pinned subsystems the `Integral` stepper solves run through exactly the
/// same path as the top-level system.
#[allow(clippy::too_many_arguments)]
fn run_blocks(
    blocks: &[Block],
    equations: &[Equation],
    values: &mut Scope,
    settings: &SolverSettings,
    specs: &BTreeMap<String, VarSpec>,
    ctx: EvalContext<'_>,
    cache: Option<&PinnedBlockCache>,
    mut dense: Option<&mut DenseRun<'_>>,
) -> std::result::Result<usize, BlockLoopFailure> {
    if let Err(e) = validate_solver_settings(settings) {
        return Err(BlockLoopFailure {
            error: e,
            failed_block_index: 0,
            iterations: 0,
        });
    }
    let mut iterations = 0usize;
    // Blocks a merge rescue already solved (Java's `skipIndices`).
    let mut skip: HashSet<usize> = HashSet::new();
    let count = blocks.len();
    for index in 0..blocks.len() {
        if skip.contains(&index) {
            continue;
        }
        // This block owns its share of whatever slice of the progress bar the
        // enclosing scope handed us — the whole bar at the top level, one
        // block's slice for the pinned subsystems this loop also serves. A
        // transient inside the block subdivides it further; see
        // `crate::progress`. Costs one thread-local read when nothing listens.
        let _span = crate::progress::enter(index as f64 / count as f64, 1.0 / count as f64);
        match solve_block_with_fallback(
            index,
            blocks,
            equations,
            values,
            settings,
            specs,
            ctx,
            &mut skip,
            cache,
            dense.as_deref_mut(),
        ) {
            Ok(block_iterations) => iterations += block_iterations,
            Err(error) => {
                return Err(BlockLoopFailure {
                    error,
                    failed_block_index: index,
                    iterations,
                })
            }
        }
    }
    Ok(iterations)
}

/// Block and solve a standalone equation list, seeded from `warm_start` where
/// it has a value for a variable and from `specs` otherwise — the Java
/// `EquationSystemSolver.solveEquationList` (`initialGuess(name, specs,
/// warmStart)`).
///
/// Only the `Integral` driver uses this. [`solve_with`] drives
/// [`block_system`] and [`run_blocks`] itself because it needs the blocking
/// report, the specs and the diagnostics for its own reporting.
fn solve_equation_list(
    equations: &[Equation],
    settings: &SolverSettings,
    specs: &BTreeMap<String, VarSpec>,
    ctx: EvalContext<'_>,
    warm_start: Option<&Scope>,
) -> Result<InnerSolve> {
    let (constants, knowns) = builtin_constants(equations);
    let report = block_system(equations, &knowns)?;

    // Java `solveEquationListPermissive`, in order: expand the specs for this
    // equation list, then seed property-call arguments, then build the value
    // map from them. The seeded copy is local because it may *create* specs
    // (the Java `applyNominalGuess` does), and `specs.keys()` upstream defines
    // the result rows — a guess must never invent a variable.
    let mut seeded = specs.clone();
    seed_property_argument_guesses(equations, &mut seeded, Missing::Create);
    let specs = &seeded;

    let mut values: Scope = Scope::default();
    values.extend(constants);
    for name in unknowns(equations, &knowns) {
        let guess = warm_start
            .and_then(|warm| warm.get(&name).copied())
            .unwrap_or_else(|| initial_guess(&name, specs));
        values.insert(name, guess);
    }
    // `uncertaintyof$<var>` is not a variable of the system, so the loop above
    // drops it. The Java `solveEquationList` carries those entries across from
    // the warm start explicitly, and without them the evaluator's
    // `UncertaintyOf(...)` arm falls back to 0.0 — a wrong answer, not a
    // failure. Port of that block.
    if let Some(warm) = warm_start {
        crate::analysis::uncertainty::carry_uncertainty_entries(warm, &mut values);
    }

    let iterations = run_blocks(
        &report.blocks,
        equations,
        &mut values,
        settings,
        specs,
        ctx,
        None,
        None,
    )
    .map_err(|failure| failure.error)?;
    Ok(InnerSolve { values, iterations })
}

// ---------------------------------------------------------------------------
// The Integral pass — port of EquationSystemSolver.solveWithIntegrals
// ---------------------------------------------------------------------------

/// Every `Integral` equation of the system, or an empty list when the document
/// mentions none.
///
/// The Java `EquationSystemSolver.findIntegrals`: detection runs on the raw
/// equations (before complex expansion), and complex mode is refused outright
/// rather than producing an expansion the quadrature cannot drive.
fn find_integrals(
    equations: &[Equation],
    defs: &Definitions,
    complex_mode: bool,
) -> Result<Vec<IntegralEquation>> {
    let mentions = equations.iter().any(|equation| {
        crate::integral::mentions_integral(&equation.lhs)
            || crate::integral::mentions_integral(&equation.rhs)
    });
    if !mentions {
        return Ok(Vec::new());
    }
    if complex_mode {
        return Err(FreesError::solver(
            "Integral is not supported in complex mode.",
        ));
    }
    crate::integral::extract(equations, defs)
}

/// Lower every `Integral` equation into the equation list the solver actually
/// blocks — the Java `solveWithIntegrals` plus `appendIntegralEquations`.
///
/// * A **variable-limit** integral (`F = Integral(2*t, t, 0, b)`) becomes the
///   inlined quadrature equation, which the evaluator recomputes at every
///   Newton residual, plus — once per integration variable — a pin
///   `t = <upper expression>`.
/// * A **constant-limit** integral is driven *here*, before the main solve:
///   [`crate::integral::integrate`] sweeps `t` from `a` to `b`, re-solving the
///   ordinary subsystem with `t` and the running total pinned at every
///   quadrature point, and the result becomes a numeric equation
///   `F = <value>`, with `t` pinned at the upper limit.
///
/// Those pins are what make an integral document square: without them
/// `F = Integral(t^2, t, 0, 1)` is one equation in two unknowns. They are also
/// why the integration variable **survives as a result variable** sitting at
/// the upper limit, which is the parent engine's documented behaviour.
///
/// Returns the lowered equations and the Newton iterations the sweeps
/// consumed (the Java `IntegrationState.iterations`, which is added to the
/// reported total).
fn lower_integrals(
    equations: &[Equation],
    integrals: &[IntegralEquation],
    settings: &SolverSettings,
    specs: &BTreeMap<String, VarSpec>,
    ctx: EvalContext<'_>,
) -> Result<(Vec<Equation>, usize)> {
    let ordinary = crate::integral::ordinary_equations(equations, integrals);
    let mut lowered = ordinary.clone();
    let mut pinned_integration_vars: BTreeSet<String> = BTreeSet::new();
    // The Java `IntegrationState` is built once per document and shared by
    // every integral, so a later sweep warm-starts from the last subsystem
    // solve of an earlier one.
    let mut warm_start: Option<Scope> = None;
    let mut stepping_iterations = 0usize;

    for ie in integrals {
        if !ie.constant_limits() {
            lowered.push(crate::integral::inlined_equation(ie, &ordinary)?);
            if pinned_integration_vars.insert(ie.integration_var.clone()) {
                lowered.push(Equation::new(
                    Expr::Var(ie.integration_var.clone()),
                    ie.upper_expr.clone(),
                    format!(
                        "{} = upper limit of {}",
                        ie.integration_var, ie.original.source_text
                    ),
                ));
            }
            continue;
        }

        let value = crate::integral::integrate(
            |t, running_total| {
                let solved = solve_pinned(
                    &ordinary,
                    &[
                        (ie.integration_var.clone(), t),
                        (ie.result_var.clone(), running_total),
                    ],
                    settings,
                    specs,
                    ctx,
                    warm_start.as_ref(),
                )
                .map_err(|err| integral_point_failure(ie, t, &err))?;
                stepping_iterations += solved.iterations;
                let point = eval_with(&ie.integrand, &solved.values, ctx)
                    .map_err(|err| integral_point_failure(ie, t, &err));
                warm_start = Some(solved.values);
                point
            },
            ie.lower(),
            ie.upper(),
            ie.fixed_step,
        )?;

        if pinned_integration_vars.insert(ie.integration_var.clone()) {
            lowered.push(Equation::new(
                Expr::Var(ie.integration_var.clone()),
                Expr::num(ie.upper()),
                format!("{} = {}", ie.integration_var, ie.upper()),
            ));
        }
        lowered.push(Equation::new(
            Expr::Var(ie.result_var.clone()),
            Expr::num(value),
            ie.original.source_text.clone(),
        ));
    }
    Ok((lowered, stepping_iterations))
}

/// Solve `ordinary` with a set of variables pinned to fixed values, seeded
/// from `warm_start`. Each pinned variable is appended as a `var = value`
/// equation (insertion order preserved) so the existing Newton/Tarjan pipeline
/// treats it as a constant for this solve.
///
/// The Java `EquationSystemSolver.solvePinned` — the one place the
/// "solve the rest of the system with the integration variable fixed"
/// behaviour lives.
fn solve_pinned(
    ordinary: &[Equation],
    pinned: &[(String, f64)],
    settings: &SolverSettings,
    specs: &BTreeMap<String, VarSpec>,
    ctx: EvalContext<'_>,
    warm_start: Option<&Scope>,
) -> Result<InnerSolve> {
    let mut subsystem: Vec<Equation> = ordinary.to_vec();
    for (name, value) in pinned {
        subsystem.push(Equation::new(
            Expr::Var(name.clone()),
            Expr::num(*value),
            format!("{name} = {value}"),
        ));
    }
    solve_equation_list(&subsystem, settings, specs, ctx, warm_start)
}

/// Name the quadrature point a subsystem solve or an integrand evaluation gave
/// up at — the Java `integrandAt`'s `catch` wrapper.
fn integral_point_failure(ie: &IntegralEquation, t: f64, err: &FreesError) -> FreesError {
    FreesError::solver(format!(
        "While evaluating Integral at {} = {t}: {}",
        ie.integration_var,
        err.to_string_message()
    ))
}

/// Prefix a block failure with the equations it came from, quoted verbatim.
///
/// The parent engine's rule is that diagnostics are source-mapped and quote the
/// user's own line; a bare "did not converge" from a 4×4 block is unactionable.
fn annotate(err: FreesError, index: usize, equations: &[&Equation]) -> FreesError {
    let quoted: Vec<&str> = equations.iter().map(|e| e.source_text.as_str()).collect();
    let plural = if quoted.len() == 1 { "" } else { "s" };
    let message = format!(
        "Block {} ({} equation{}) failed: {}\n  {}",
        index + 1,
        quoted.len(),
        plural,
        err.to_string_message(),
        quoted.join("\n  ")
    );
    match err {
        FreesError::Parse { span, .. } => FreesError::Parse { message, span },
        _ => FreesError::solver(message),
    }
}

/// Warn about any solved value that left the bounds its `GUESS` declared.
///
/// Since [`crate::solver::newton::newton_solve`] enforces bounds at every
/// probe (the three Java
/// `Math.clamp` sites), a bounded solve can no longer end outside its box and
/// this warning should be unreachable for bounded variables. It is kept as a
/// safety net — a future warm-start path, or a spec produced outside
/// `variable_specs`, must still never return a silently out-of-range answer.
fn check_bounds(
    specs: &BTreeMap<String, VarSpec>,
    values: &Scope,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for (name, spec) in specs {
        if !spec.is_bounded() {
            continue;
        }
        let Some(&value) = values.get(name) else {
            continue;
        };
        if value < spec.lower || value > spec.upper {
            diagnostics.push(Diagnostic::warning(format!(
                "`{name}` solved to {value}, outside the GUESS bounds \
                 [{}, {}]; bounds are not enforced during iteration",
                spec.lower, spec.upper
            )));
        }
    }
}

// ---------------------------------------------------------------------------
// Small helper on the error type
// ---------------------------------------------------------------------------

impl FreesError {
    /// The message without the `Display` impl's `"<kind> error: "` prefix.
    ///
    /// Re-wrapping an error would otherwise stutter
    /// (`solver error: Block 1 failed: solver error: …`).
    pub fn to_string_message(&self) -> String {
        match self {
            FreesError::Parse { message, .. }
            | FreesError::Solver { message }
            | FreesError::Property { message }
            | FreesError::Evaluation { message } => message.clone(),
            FreesError::UnknownUnit { unit } => format!("unknown unit: {unit}"),
        }
    }
}

/// The variables the document mentions that are neither unknowns nor built-in
/// constants — currently always empty, kept as the hook the component expander
/// will need. Exposed so callers can assert the invariant.
pub fn known_constants(equations: &[Equation]) -> BTreeMap<String, f64> {
    builtin_constants(equations).0
}

/// All variable names in a parsed document, sorted — unknowns and constants
/// alike. Handy for editors that want to autocomplete against the document.
pub fn document_variables(doc: &Document) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for equation in doc.equations() {
        out.extend(equation.variables());
    }
    out
}

// ---------------------------------------------------------------------------
// Seams for `crate::analysis`
// ---------------------------------------------------------------------------
//
// The analysis layer (optimisation, root enumeration, sweeps) *drives* this
// module rather than re-implementing it. Two things it needs are computed here
// and nowhere else, so they are exported rather than duplicated:
//
//  * `variable_override_spec` — the Java `VariableInfoDto.toSpec`, which the
//    Java `Optimizer.initialGuess` reads through `spec.guess()`;
//  * `solve_block_newton` — the Java `NewtonSolver.solveBlock`, which the Java
//    `AllRootsSolver` calls directly (deliberately *without* the retry ladder:
//    enumerating roots means accepting that a start point may simply fail).

/// One [`VariableOverride`] resolved to `(lowercase name, guess, lower, upper)`
/// in SI — the Java `VariableInfoDto.toSpec` plus the `VariableSpec` compact
/// constructor's validation.
///
/// # Errors
///
/// The same three rejections [`solve_with`] applies to an override: any NaN, a
/// lower bound above the upper, or an explicit guess outside its own bounds.
pub fn variable_override_spec(o: &VariableOverride) -> Result<(String, f64, f64, f64)> {
    let (name, spec) = override_spec(o)?;
    Ok((name, spec.guess, spec.lower, spec.upper))
}

/// Solve one Tarjan block in place with plain Newton, leaving its unknowns'
/// values in `values` and returning the iteration count.
///
/// The Java `NewtonSolver.solveBlock(block, values, deadline, specs)` as
/// `AllRootsSolver` calls it: **one rung only**, none of the
/// [`solve_block_with_fallback`] retry ladder, because a multi-start root search
/// expects most starts to fail and must not spend the ladder's budget on each.
/// `bounds` is the per-variable box the Java reads off its `VariableSpec` map;
/// a variable missing from it is unbounded.
///
/// On failure `values` holds the last iterate, exactly as the Java leaves its
/// partially updated variable map behind.
///
/// # Errors
///
/// [`FreesError::Solver`] when Newton does not converge, plus whatever the
/// residual evaluation raises at the initial point.
pub fn solve_block_newton(
    block: &Block,
    equations: &[Equation],
    values: &mut Scope,
    settings: &SolverSettings,
    bounds: &BTreeMap<String, (f64, f64)>,
    ctx: EvalContext<'_>,
) -> Result<usize> {
    let specs: BTreeMap<String, VarSpec> = bounds
        .iter()
        .map(|(name, &(lower, upper))| {
            (
                name.clone(),
                VarSpec {
                    guess: DEFAULT_GUESS,
                    lower,
                    upper,
                },
            )
        })
        .collect();
    solve_block(
        0, block, equations, values, settings, &specs, ctx, None, None,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solved(source: &str) -> Solution {
        solve(source, &SolverSettings::default())
            .unwrap_or_else(|err| panic!("solve failed for {source:?}: {err}"))
    }

    fn value(solution: &Solution, name: &str) -> f64 {
        *solution
            .values
            .get(name)
            .unwrap_or_else(|| panic!("no value for {name}: {:?}", solution.values))
    }

    fn assert_close(actual: f64, expected: f64) {
        let tolerance = 1e-9 * expected.abs().max(1.0);
        assert!(
            (actual - expected).abs() <= tolerance,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn a_single_assignment_solves() {
        let solution = solved("a = 2");
        assert_close(value(&solution, "a"), 2.0);
        assert_eq!(solution.blocks.len(), 1);
        assert!(solution.blocks[0].is_scalar());
    }

    #[test]
    fn a_dotted_guess_maps_onto_the_expanded_member() {
        let report = check(
            "\
Resistor R1(R=10)
VoltageSource V1(E=12)
Ground G1()
connect(V1.p, R1.a)
connect(R1.b, V1.n, G1.port)
GUESS R1.a.V = 12
",
        )
        .expect("check");
        assert!(
            !report
                .diagnostics
                .iter()
                .any(|d| d.message.contains("no such variable")),
            "dotted GUESS should resolve, got {:?}",
            report.diagnostics
        );
        assert!(
            report
                .instances
                .iter()
                .any(|i| i.name == "r1" && i.line >= 1),
            "expected instance identity for R1, got {:?}",
            report.instances
        );
    }

    #[test]
    fn local_component_shadowing_is_an_advisory() {
        let report = check(
            "\
COMPONENT Pipe(in, out)
  out.mdot = in.mdot
END
",
        )
        .expect("check");
        assert!(
            report
                .unit_warnings
                .iter()
                .any(|w| w.contains("shadows the standard-library")),
            "got {:?}",
            report.unit_warnings
        );
        assert!(report
            .definitions
            .iter()
            .any(|d| d.name == "pipe" && d.line == 1));
    }

    #[test]
    fn case_only_assignment_conflict_is_an_advisory() {
        let report = check("T = 300\nt = 2\n").expect("check");
        assert!(
            report
                .unit_warnings
                .iter()
                .any(|w| w.contains("Case-only naming conflict")
                    && w.contains("'T'")
                    && w.contains("'t'")),
            "got {:?}",
            report.unit_warnings
        );
    }

    #[test]
    fn a_harmless_case_insensitive_reference_is_not_a_conflict() {
        let report = check("T = 300\nx = T + 1\n").expect("check");
        assert!(
            report
                .unit_warnings
                .iter()
                .all(|w| !w.contains("Case-only naming conflict")),
            "got {:?}",
            report.unit_warnings
        );
    }

    #[test]
    fn a_self_instantiating_component_does_not_overflow_check() {
        // Identities walk before expansion; a cycle must not abort the process.
        let report = check(
            "\
COMPONENT SelfLoop(a, b)
  SelfLoop again(a, b)
END
SelfLoop L(s1, s2)
s1.sig = 1
",
        )
        .expect("check must return, not overflow");
        assert!(!report.solvable);
    }

    #[test]
    fn nested_child_identities_point_at_the_parent_scope() {
        let source = "\
COMPONENT Box(in, out)
  PARAM fluid$
  Pipe inner(in, out, fluid$=fluid$, L=1, D=0.05, rough=1e-4)
END
Source SUP(fluid$=Water, mdot=1, P=2e5, T=300)
Box A(fluid$=Water)
Box B(fluid$=Water)
Sink RET()
connect(SUP.out, A.in)
connect(A.out, B.in)
connect(B.out, RET.in)
";
        let report = check(source).expect("check");
        let a = report
            .instances
            .iter()
            .find(|i| i.name == "a.inner")
            .expect("a.inner");
        let b = report
            .instances
            .iter()
            .find(|i| i.name == "b.inner")
            .expect("b.inner");
        let a_parent = report.instances.iter().find(|i| i.name == "a").unwrap();
        let b_parent = report.instances.iter().find(|i| i.name == "b").unwrap();
        assert_eq!(a.line, a_parent.line);
        assert_eq!(b.line, b_parent.line);
        assert_ne!(a.line, b.line, "repeated child names must keep their scope");
        assert!(a_parent.local_type);
        assert!(b_parent.local_type);
    }

    #[test]
    fn a_dotted_guess_steers_the_same_scalar_as_variable_information() {
        let source = "\
Resistor R1(R=10)
VoltageSource V1(E=12)
Ground G1()
connect(V1.p, R1.a)
connect(R1.b, V1.n, G1.port)
GUESS R1.a.V [0, 1]
";
        let report = check(source).expect("check");
        assert!(
            !report
                .diagnostics
                .iter()
                .any(|d| d.message.contains("no such variable")),
            "got {:?}",
            report.diagnostics
        );
        let bounded = solve(source, &SolverSettings::default());
        let free = solve(
            "\
Resistor R1(R=10)
VoltageSource V1(E=12)
Ground G1()
connect(V1.p, R1.a)
connect(R1.b, V1.n, G1.port)
",
            &SolverSettings::default(),
        );
        assert!(
            bounded.is_err()
                || free.as_ref().ok().is_some_and(|s| {
                    s.values
                        .get("r1$a$v")
                        .or_else(|| s.values.get("r1$a$V"))
                        .copied()
                        != bounded
                            .as_ref()
                            .ok()
                            .and_then(|s| s.values.get("r1$a$v").or_else(|| s.values.get("r1$a$V")))
                            .copied()
                }),
            "bounds on R1.a.V should address the expanded scalar; bounded={bounded:?} free={free:?}"
        );
    }

    #[test]
    fn check_reports_connection_topology() {
        let report = check(
            "\
Source SUP(fluid$=Water, mdot=1, P=2e5, T=300)
Pipe LINE(fluid$=Water, L=10, D=0.05, rough=1e-4)
Sink RET()
connect(SUP.out, LINE.in)
connect(LINE.out, RET.in)
",
        )
        .expect("check");
        assert!(
            report
                .connections
                .iter()
                .any(|c| c.endpoints.iter().any(|e| e.contains("sup"))
                    && c.endpoints.iter().any(|e| e.contains("line"))),
            "expected a SUP–LINE connection, got {:?}",
            report.connections
        );
    }

    #[test]
    fn check_advises_on_an_inactive_variant_parameter() {
        let report = check(
            "\
COMPONENT C(in, out)
  PARAM model$ = a, r, q
  out.mdot = in.mdot
  VARIANT a REQUIRE r
    out.P = in.P * r
  END
  VARIANT b REQUIRE q
    out.P = in.P * q
  END
END
C X(s1, s2, r=2, q=3)
",
        )
        .expect("check");
        assert!(
            report
                .unit_warnings
                .iter()
                .any(|w| w.contains("'q'") && w.contains("'a'")),
            "expected inactive-parameter advisory, got {:?}",
            report.unit_warnings
        );
    }

    #[test]
    fn a_chain_solves_in_dependency_order_not_source_order() {
        let solution = solved("c = b + 1\nb = a * 3\na = 2\n");
        assert_close(value(&solution, "a"), 2.0);
        assert_close(value(&solution, "b"), 6.0);
        assert_close(value(&solution, "c"), 7.0);

        let order: Vec<&str> = solution
            .blocks
            .iter()
            .map(|b| b.variables[0].as_str())
            .collect();
        assert_eq!(order, vec!["a", "b", "c"]);
    }

    #[test]
    fn a_simultaneous_pair_is_one_block() {
        let solution = solved("u + v = 10\nu - v = 2\n");
        assert_close(value(&solution, "u"), 6.0);
        assert_close(value(&solution, "v"), 4.0);
        assert_eq!(solution.blocks.len(), 1);
        assert_eq!(solution.blocks[0].variables, vec!["u", "v"]);
    }

    #[test]
    fn built_in_constants_are_knowns_not_unknowns() {
        let solution = solved("a = pi#\nb = R#\nc = g#\n");
        assert_close(value(&solution, "a"), std::f64::consts::PI);
        assert_close(value(&solution, "b"), 8.314_462_618);
        assert_close(value(&solution, "c"), 9.806_65);
        // Three equations, three unknowns — the constants must not have been
        // counted as free variables.
        assert_eq!(solution.blocks.len(), 3);
    }

    #[test]
    fn guess_directives_seed_the_iterate() {
        // x^2 = 9 has two roots; the guess picks which one is found.
        assert_close(value(&solved("GUESS x = 3\nx ^ 2 = 9\n"), "x"), 3.0);
        assert_close(value(&solved("GUESS x = -3\nx ^ 2 = 9\n"), "x"), -3.0);
    }

    #[test]
    fn a_guess_for_an_unknown_name_is_a_warning_not_a_failure() {
        let solution = solved("GUESS zzz = 3\nx = 1\n");
        assert_close(value(&solution, "x"), 1.0);
        assert_eq!(solution.diagnostics.len(), 1);
        assert!(solution.diagnostics[0].message.contains("zzz"));
        assert_eq!(
            solution.diagnostics[0].severity,
            crate::diag::Severity::Warning
        );
    }

    #[test]
    fn guess_bounds_clamp_the_starting_point() {
        let mut spec = VarSpec::default();
        apply_guess(
            &mut spec,
            &GuessDirective {
                name: "x".into(),
                guess: Some(100.0),
                lower: Some(0.0),
                upper: Some(10.0),
            },
        );
        assert_eq!(spec.initial(), 10.0);
    }

    #[test]
    fn reversed_guess_bounds_are_repaired_not_panicked() {
        let mut spec = VarSpec::default();
        apply_guess(
            &mut spec,
            &GuessDirective {
                name: "x".into(),
                guess: Some(5.0),
                lower: Some(10.0),
                upper: Some(0.0),
            },
        );
        assert_eq!(spec.lower, 0.0);
        assert_eq!(spec.upper, 10.0);
        assert_eq!(spec.initial(), 5.0);
    }

    #[test]
    fn units_convert_to_si_at_parse_time() {
        let solution = solved("P = 140 [kPa]\nQ = P * 2\n");
        assert_close(value(&solution, "p"), 140_000.0);
        assert_close(value(&solution, "q"), 280_000.0);
    }

    #[test]
    fn an_empty_document_is_a_solver_error() {
        let err = solve("{ only a comment }", &SolverSettings::default()).unwrap_err();
        assert_eq!(err, FreesError::solver("No equations to solve."));
    }

    #[test]
    fn an_overdetermined_document_names_the_redundant_relation() {
        let err = solve("z = 1\nz = 2\n", &SolverSettings::default()).unwrap_err();
        let message = err.to_string_message();
        assert!(matches!(err.error, FreesError::Solver { .. }), "{err:?}");
        assert!(message.contains("2 equations and 1 variables"), "{message}");
        assert!(message.contains("overspecified"), "{message}");
    }

    #[test]
    fn an_underdetermined_document_names_the_free_quantity() {
        let err = solve("m + n = 5\n", &SolverSettings::default()).unwrap_err();
        let message = err.to_string_message();
        assert!(message.contains("underspecified"), "{message}");
        assert!(message.contains('n'), "{message}");
    }

    #[test]
    fn an_unsupported_block_is_refused_by_name() {
        let err = solve(
            "DYNAMIC d(method = ode45)\n  der = 1\nEND\n",
            &SolverSettings::default(),
        )
        .unwrap_err();
        assert!(matches!(err.error, FreesError::Parse { .. }), "{err:?}");
        assert!(err.to_string_message().contains("DYNAMIC"));
    }

    /// `PARAMETRIC` / `PLOT` / `STATE TABLE` parse into `Document::blocks`
    /// since Phase 8 and contribute no equations, so a document that declares
    /// only one of them is a *solver* refusal ("No equations to solve"), not a
    /// parse one — the same classification the Java oracle records for the
    /// sweep fixtures in `fixtures/corpus-pending/`.
    #[test]
    fn a_declarative_block_is_no_longer_a_parse_refusal() {
        for source in [
            "PARAMETRIC s (t)\n  t = 0:1:2\nEND\n",
            "PLOT 'x'\n  kind = xy\nEND\n",
            "STATE TABLE c(P1)\n  FLUID = Water\nEND\n",
        ] {
            let err = solve(source, &SolverSettings::default()).unwrap_err();
            assert!(
                matches!(err.error, FreesError::Solver { .. }),
                "{source}: {err:?}"
            );
        }
    }

    /// The three component forms after Phase 6 wired the expander in. The
    /// capability gate that used to refuse all three is gone; each now gets the
    /// answer the reference engine gives, and each of these three messages was
    /// checked against the Java oracle character for character (modulo the
    /// port-wide `source_text` convention, which keeps the user's whitespace
    /// where ANTLR's `getText()` drops it).
    #[test]
    fn the_three_component_forms_answer_like_the_reference_engine() {
        // A template nobody instantiates contributes nothing, so the document is
        // empty — a *solver* verdict, not a parse one.
        let err = solve(
            "COMPONENT pump(in, out)\n  out.P = in.P\nEND\n",
            &SolverSettings::default(),
        )
        .unwrap_err();
        assert!(matches!(err.error, FreesError::Solver { .. }), "{err:?}");
        assert_eq!(err.to_string_message(), "No equations to solve.");

        // An instantiation missing a required parameter is refused by name at
        // expansion time — the library ships no defaults for physical inputs.
        let err = solve("Pump P1(s1, s2)\nx = 1\n", &SolverSettings::default()).unwrap_err();
        assert!(matches!(err.error, FreesError::Parse { .. }), "{err:?}");
        assert_eq!(
            err.to_string_message(),
            "Component 'p1' (pump): parameter 'eta' has no value \
             (give it a default or pass eta=value)."
        );

        // A `connect` naming something that is neither an instance port nor a
        // stream is refused, quoting the declaration.
        let err = solve("connect(a.out, b.in)\nx = 1\n", &SolverSettings::default()).unwrap_err();
        assert!(matches!(err.error, FreesError::Parse { .. }), "{err:?}");
        assert!(
            err.to_string_message().starts_with(
                "connect(...): 'a.out' is not a port (instance.port) or a stream name."
            ),
            "{err:?}"
        );

        // …and a document with no component layer never touches the expander.
        assert!(solve("x = 1\n", &SolverSettings::default()).is_ok());
    }

    /// The end-to-end shape the gate used to block: a built-in instantiated,
    /// wired, and solved through the ordinary Newton/Tarjan path.
    #[test]
    fn a_component_network_expands_and_solves() {
        let solution = solve(
            "Resistor R1(n1, n2, R = 10)\nGround G1(n2)\nn1.V = 12\n",
            &SolverSettings::default(),
        )
        .expect("the network solves");
        assert_eq!(solution.values["n2$v"], 0.0);
        assert_eq!(solution.values["n1$i"], 1.2);
        // Port members carry the display spelling the expander minted…
        assert_eq!(solution.display_names["n1$i"], "n1.i");
        // …and the SI unit their physical domain fixes, which nothing in the
        // document could have derived.
        assert_eq!(solution.inferred_units["n1$i"], "A");
        // The datasheet payload rides out on the solution.
        assert_eq!(solution.component_instances.len(), 2);
        assert_eq!(solution.component_instances[0].type_name, "resistor");
    }

    /// A storage network with no `DYNAMIC` block solves its steady operating
    /// point (`der(X) = rhs` → `rhs = 0`) and says so in a diagnostic.
    #[test]
    fn a_storage_network_without_a_dynamic_block_solves_steady() {
        let solution = solve(
            "TorqueSource     TQ(T = 5)\n\
             RotationalDamper DP(c = 0.25)\n\
             Inertia          IN(J = 0.4, w0 = 0)\n\
             MechGround       G1()\n\
             MechGround       G2()\n\
             connect(TQ.a, IN.port, DP.a)\n\
             connect(TQ.b, G1.port)\n\
             connect(DP.b, G2.port)\n",
            &SolverSettings::default(),
        )
        .expect("the steady network solves");
        // der(w) = tau/J with tau summing to zero at the node: the damper takes
        // the whole 5 N.m, so w = 5 / 0.25 = 20 rad/s.
        assert_eq!(solution.values["in$port$w"], 20.0);
        assert!(
            solution
                .diagnostics
                .iter()
                .any(|d| d.message.contains("steady operating point")),
            "{:?}",
            solution.diagnostics
        );
    }

    #[test]
    fn a_syntax_error_is_a_parse_error_with_a_span() {
        let err = solve("x = = 2\n", &SolverSettings::default()).unwrap_err();
        assert!(matches!(err.error, FreesError::Parse { .. }), "{err:?}");
        assert!(err.span().is_some());
    }

    #[test]
    fn check_reports_a_solvable_system_without_solving() {
        let report = check("x^2 + y^3 = 77\nx/y = 1.23456\n").unwrap();
        assert!(report.solvable);
        assert_eq!(report.equation_count, 2);
        assert_eq!(report.unknown_count, 2);
        assert_eq!(report.variables, vec!["x", "y"]);
        assert_eq!(
            report.message,
            "No syntax errors were detected. There are 2 equations and 2 variables."
        );
    }

    #[test]
    fn check_reports_structural_failure_without_erroring() {
        let report = check("m + n = 5\n").unwrap();
        assert!(!report.solvable);
        assert_eq!(report.equation_count, 1);
        assert_eq!(report.unknown_count, 2);
        assert!(
            report.message.contains("underspecified"),
            "{}",
            report.message
        );
    }

    #[test]
    fn check_reports_an_empty_document_as_unsolvable() {
        let report = check("{ nothing }").unwrap();
        assert!(!report.solvable);
        assert_eq!(report.equation_count, 0);
        assert_eq!(report.message, "No equations to solve.");
    }

    #[test]
    fn check_reports_a_syntax_error_as_data_not_an_err() {
        // Mirrors the Java 400-with-body: the report is the payload api.ts
        // parses, so a parse failure must come back as a report, not an Err.
        let report = check("x = = 2").unwrap();
        assert!(!report.solvable);
        assert_eq!(report.equation_count, 0);
        assert_eq!(report.unknown_count, 0);
        assert!(report.variables.is_empty());
        assert!(
            report.message.starts_with("Syntax error: "),
            "{}",
            report.message
        );
        assert_eq!(report.error_line, Some(1));
        assert_eq!(report.errors.len(), 1);
        assert_eq!(report.errors[0].line, 1);
        assert!(report.errors[0].column >= 1);
    }

    #[test]
    fn check_error_line_is_the_line_of_the_broken_statement() {
        let report = check("a = 1\nb = 2\nc = = 3\n").unwrap();
        assert_eq!(report.error_line, Some(3));
        assert_eq!(report.errors[0].line, 3);
    }

    #[test]
    fn check_does_not_solve() {
        // `1/0` would be an evaluation error if this were solved; check must
        // not touch the evaluator.
        let report = check("x = 1 / 0\n").unwrap();
        assert!(report.solvable);
    }

    #[test]
    fn a_domain_error_at_the_initial_point_is_reported_verbatim() {
        let err = solve("x = 1 / 0\n", &SolverSettings::default()).unwrap_err();
        let message = err.to_string_message();
        assert!(message.contains("division by zero"), "{message}");
        // and it quotes the offending equation
        assert!(message.contains("x = 1 / 0"), "{message}");
    }

    #[test]
    fn an_unknown_function_is_reported_not_silently_nan() {
        let err = solve("x = nosuchfn(2)\n", &SolverSettings::default()).unwrap_err();
        assert!(
            err.to_string_message().contains("nosuchfn"),
            "{}",
            err.to_string_message()
        );
    }

    #[test]
    fn an_invalid_region_reached_mid_iteration_does_not_abort_the_solve() {
        // ln(x) = 0 → x = 1. Starting at the default guess 1.0 is fine, but the
        // Newton path for `ln(x) - 0` probes and can step towards x <= 0, where
        // `eval` raises rather than returning NaN. The engine must survive it.
        let solution = solved("ln(x) = 0\n");
        assert_close(value(&solution, "x"), 1.0);
    }

    #[test]
    fn iterations_are_summed_across_blocks() {
        // Each block's cost is added, so a two-block document costs strictly
        // more than either block alone.
        let one = solved("a = 2\n");
        let two = solved("a = 2\nb = a * 3\n");
        assert_eq!(one.iterations, 1);
        assert_eq!(two.blocks.len(), 2);
        assert!(two.iterations > one.iterations, "{}", two.iterations);
        assert!(
            two.iterations <= 4,
            "linear blocks should be cheap: {}",
            two.iterations
        );

        // A nonlinear block genuinely iterates.
        let nonlinear = solved("x ^ 3 - 2 * x - 5 = 0\n");
        assert!(nonlinear.iterations > 1, "{}", nonlinear.iterations);
        assert_close(value(&nonlinear, "x"), 2.094_551_481_542_326_6);
    }

    #[test]
    fn case_insensitive_names_collapse_to_one_variable() {
        let solution = solved("Tin = 300\nT_out = TIN * 2\nresult = t_Out + tin\n");
        assert_eq!(solution.values.len(), 3);
        assert_close(value(&solution, "tin"), 300.0);
        assert_close(value(&solution, "t_out"), 600.0);
        assert_close(value(&solution, "result"), 900.0);
    }

    #[test]
    fn for_block_bodies_are_flattened_into_the_system() {
        // `expand_document` unrolls FOR bodies per iteration with the loop
        // variable substituted (the Java `EquationParser.flatten` rule), so
        // the body must use the index — a constant body would correctly be
        // rejected as the same equation stated twice.
        let solution = solved("FOR i = 1 TO 2\n  a[i] = 5 * i\nEND\nb = a[1] + a[2]\n");
        assert_close(value(&solution, "a[1]"), 5.0);
        assert_close(value(&solution, "a[2]"), 10.0);
        assert_close(value(&solution, "b"), 15.0);
    }

    /// Phase 9 replaced the `SYMBOLIC` refusal with `CasIdentity`: an equation
    /// involving the declared variable is an identity solved for its
    /// coefficients, and a `SYMBOLIC` declaration on its own still leaves the
    /// rest of the document alone.
    #[test]
    fn a_symbolic_declaration_alone_does_not_disturb_the_document() {
        let solution = solved("SYMBOLIC s\nx = 1\n");
        assert_close(value(&solution, "x"), 1.0);
        // The symbolic name is a declaration, never a solver unknown.
        assert!(!solution.values.contains_key("s"), "{:?}", solution.values);
    }

    #[test]
    fn a_symbolic_identity_is_solved_for_its_coefficients() {
        // The partial-fraction residues of (s+3)/((s+1)(s+2)): A = 2, B = -1.
        let solution = solved("SYMBOLIC s\ntf([1, 3], [1, 3, 2]) = A/(s+1) + B/(s+2)\n");
        assert_close(value(&solution, "a"), 2.0);
        assert_close(value(&solution, "b"), -1.0);
    }

    #[test]
    fn an_identity_over_two_symbolic_variables_is_refused_by_name() {
        let err = solve("SYMBOLIC s, t\ns = t\n", &SolverSettings::default()).unwrap_err();
        assert!(
            err.to_string_message()
                .contains("only one SYMBOLIC variable"),
            "{err}"
        );
    }

    #[test]
    fn call_statements_are_refused_rather_than_dropped() {
        // The refusal comes from the `flatten_calls` pipeline stage: a CALL
        // to an unknown name is an error naming it (the Java flattenCallProc
        // behaviour), never a silently dropped statement.
        let err = solve("CALL mix(1, 2 : y)\nx = 1\n", &SolverSettings::default()).unwrap_err();
        let message = err.to_string_message();
        assert!(message.contains("mix"), "{message}");
    }

    #[test]
    fn a_block_failure_quotes_its_equations() {
        // exp(x) = -1 has no real root: the block cannot converge, and the
        // message must say which equation failed.
        let err = solve("exp(x) = -1\n", &SolverSettings::default()).unwrap_err();
        let message = err.to_string_message();
        assert!(message.contains("exp(x) = -1"), "{message}");
        assert!(message.starts_with("Block 1"), "{message}");
    }

    #[test]
    fn error_messages_do_not_stutter_their_kind_prefix() {
        let err = FreesError::solver("did not converge");
        assert_eq!(err.to_string(), "solver error: did not converge");
        assert_eq!(err.to_string_message(), "did not converge");
    }

    #[test]
    fn constants_fold_at_parse_time_like_the_java_ast_builder() {
        // `AstBuilder.visitVarAtom` substitutes built-in `#` constants as
        // numeric literals at parse time, so they never appear as document
        // variables and the knowns hook has nothing left to collect.
        let doc = parse_document("a = pi# + b\n").unwrap();
        let vars: Vec<String> = document_variables(&doc).into_iter().collect();
        assert_eq!(vars, vec!["a", "b"]);
        assert!(
            known_constants(&doc.equations().into_iter().cloned().collect::<Vec<_>>()).is_empty()
        );

        // The folded literal carries the raw SI unit string, grounding the
        // unit checker: v = g#*t infers v as m/s (m/s^2 times s).
        let g = parse_document("x = g#\n").unwrap();
        match &g.equations()[0].rhs {
            Expr::Num { value, unit, .. } => {
                assert!((value - 9.806_65).abs() < 1e-12);
                assert_eq!(unit.as_deref(), Some("m/s^2"));
            }
            other => panic!("g# should fold to a literal, got {other:?}"),
        }

        // Unknown `#` names stay variables, exactly like the Java lookup miss.
        let unknown = parse_document("y = zz#\n").unwrap();
        assert_eq!(
            document_variables(&unknown).into_iter().collect::<Vec<_>>(),
            vec!["y", "zz#"]
        );
    }

    #[test]
    fn constants_are_used_but_not_reported_as_variables() {
        let solution = solved("a = 2 * pi#\n");
        assert_close(value(&solution, "a"), 2.0 * std::f64::consts::PI);
        // `fixtures/golden/constants.json` lists only the document's own
        // variables — a folded constant is not a result row.
        assert_eq!(
            solution.values.keys().collect::<Vec<_>>(),
            vec![&"a".to_string()]
        );
    }

    /// Bounds are enforced now (the Java `Math.clamp` sites): a document whose
    /// equations force a value outside its declared box cannot solve — the
    /// whole retry ladder runs inside the box and exhausts, exactly as the
    /// Java engine fails this document. (It used to "solve" to 5.0 with a
    /// warning; that was ranked divergence #3 in `docs/status-phase1.md`.)
    #[test]
    fn a_solution_outside_the_guess_bounds_is_a_ladder_exhaustion_failure() {
        let failure = solve("GUESS x = 0.5 [0, 1]\nx = 5\n", &SolverSettings::default())
            .expect_err("x = 5 is unreachable inside [0, 1]");
        assert_eq!(failure.failed_block_index, Some(0));
        let partial = failure
            .partial
            .as_deref()
            .expect("ladder exhaustion still ships diagnostics");
        assert_eq!(partial.blocks.len(), 1);
        assert_eq!(partial.residuals.len(), 1);
        // The residual is evaluated at the restored initial guess (0.5 - 5).
        assert!(
            (partial.residuals[0].residual + 4.5).abs() < 1e-12,
            "{:?}",
            partial.residuals
        );
    }
}
