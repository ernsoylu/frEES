//! FUNCTION / PROCEDURE / MODULE execution.
//!
//! Port of `../frEES/backend/core/src/main/java/com/frees/backend/parser/ProcedureEvaluator.java`
//! plus the CALL-flattening half of `EquationParser`
//! (`flattenCallProc` / `flattenProcedureCall` / `flattenModuleCall` /
//! `namespaceExpr`) and the synthetic-output dispatch of
//! `Evaluator.evalProcedureOutput`.
//!
//! # The real CALL mechanism (verified against `EquationParser`)
//!
//! A `CALL p(inputs : outputs)` to a **PROCEDURE** does *not* execute the body
//! at flatten time. `flattenProcedureCall` emits one binding equation per
//! output slot:
//!
//! ```text
//! out_k = proc$<name>$<k>(inputs…)
//! ```
//!
//! with the inputs kept **symbolic**. The synthetic `proc$…` call is evaluated
//! by the expression evaluator at solve time (`Evaluator.evalProcedureOutput`),
//! which runs the whole body once per output and picks output `k`. That is what
//! [`flatten_calls`] emits and [`call_proc_output`] executes.
//!
//! A `CALL m(inputs : outputs)` to a **MODULE** grafts the body equations into
//! the caller's system with every variable renamed into a per-instance
//! namespace `<name>$<instance>$` (`flattenModuleCall`), plus input binding
//! equations `ns$param = inputExpr` and output binding equations
//! `outputVar = ns$outputParam`. The instance counter is per-flatten and
//! 1-based, exactly like the Java `moduleCounter().incrementAndGet()`.
//!
//! # Limits
//!
//! * `REPEAT`/`WHILE` iteration ceiling: [`MAX_ITERATIONS`], ported verbatim
//!   from `ProcedureEvaluator.MAX_ITERATIONS` (100 000) with the Java error
//!   messages.
//! * `FOR` ceiling: Java has none (a `FOR i = 1 TO 1e9` simply grinds); the
//!   wasm engine must not hang the browser tab, so the same 100 000 ceiling
//!   applies. **Deviation**, documented here.
//! * Recursion ceiling: Java has none (it rides the JVM stack; deep recursion
//!   is a `StackOverflowError`). A wasm stack overflow is an uncatchable trap,
//!   so user-function calls nested deeper than [`MAX_CALL_DEPTH`] are refused
//!   with a diagnostic instead. **Deviation**, documented here.

use std::cell::Cell;
use std::collections::{BTreeMap, HashMap};

use crate::ast::{Equation, Expr, Statement};
use crate::diag::{FreesError, Result};
use crate::eval::{eval_with, lookup_intrinsic, Body, EvalContext, Scope};
use crate::parser::defs::{Definitions, FunctionDef, ModuleDef, ProcStatement, ProcedureDef};

/// `ProcedureEvaluator.MAX_ITERATIONS` — the REPEAT/WHILE guard (also applied
/// to procedural FOR loops here; see the module docs).
const MAX_ITERATIONS: u64 = 100_000;

/// Maximum nesting of user FUNCTION/PROCEDURE calls. Java has no guard (see
/// the module docs); 64 is far beyond the hand-written recursion the corpus
/// shows (the oracle's deepest is `Factorial(5)`) while keeping the worst-case
/// native-debug stack (~16 KiB per call level through the evaluator) inside
/// the 2 MiB test-thread stack and the release/wasm stack with room to spare.
const MAX_CALL_DEPTH: u32 = 64;

/// Prefix of the synthetic per-output calls `flatten_calls` emits for
/// PROCEDURE calls. Mirrors the literal `"proc$"` in
/// `EquationParser.flattenProcedureCall` / `Evaluator.evalProcedureOutput`.
/// `$` cannot appear in a user identifier, so these names are unforgeable.
pub const PROC_OUTPUT_PREFIX: &str = "proc$";

thread_local! {
    /// Current user-call nesting depth (wasm is single-threaded; native tests
    /// get one counter per thread).
    static CALL_DEPTH: Cell<u32> = const { Cell::new(0) };
}

/// RAII guard for [`CALL_DEPTH`] — decrements on scope exit even when the body
/// errors out.
struct DepthGuard;

impl DepthGuard {
    fn enter(kind: &str, name: &str) -> Result<DepthGuard> {
        let depth = CALL_DEPTH.with(Cell::get);
        if depth >= MAX_CALL_DEPTH {
            return Err(FreesError::evaluation(format!(
                "{kind} calls nested more than {MAX_CALL_DEPTH} levels deep \
                 (is `{name}` recursing without a base case?)"
            )));
        }
        CALL_DEPTH.with(|d| d.set(depth + 1));
        Ok(DepthGuard)
    }
}

impl Drop for DepthGuard {
    fn drop(&mut self) {
        CALL_DEPTH.with(|d| d.set(d.get().saturating_sub(1)));
    }
}

// ---------------------------------------------------------------------------
// FUNCTION / PROCEDURE execution (ProcedureEvaluator)
// ---------------------------------------------------------------------------

/// Execute a single-output `FUNCTION` body for an inline expression call.
/// Arguments are already evaluated, positional, SI.
///
/// Port of `ProcedureEvaluator.callFunction`: locals start as a **copy of the
/// caller's scope** (dynamic scoping — the body sees the caller's variables),
/// parameters are bound over it, the body runs sequentially, and the value
/// assigned to the function's own name is the result.
pub fn call_function(
    def: &FunctionDef,
    args: &[f64],
    defs: &Definitions,
    caller_scope: &Scope,
) -> Result<f64> {
    if args.len() != def.params.len() {
        return Err(FreesError::evaluation(format!(
            "FUNCTION {} expects {} argument(s), got {}",
            def.name,
            def.params.len(),
            args.len()
        )));
    }
    let _guard = DepthGuard::enter("FUNCTION", &def.name)?;
    let mut locals = caller_scope.clone();
    for (param, value) in def.params.iter().zip(args) {
        locals.insert(param.clone(), *value);
    }
    execute_body(&def.body, &mut locals, defs)?;
    match locals.get(&def.name) {
        Some(value) => Ok(*value),
        None => Err(FreesError::evaluation(format!(
            "FUNCTION {} never assigned a return value ('{} := ...' missing)",
            def.name, def.name
        ))),
    }
}

/// Execute a `PROCEDURE` body and return its output variables as a
/// name → value map. Port of `ProcedureEvaluator.callProcedure`.
pub fn call_procedure(
    def: &ProcedureDef,
    inputs: &[f64],
    defs: &Definitions,
    caller_scope: &Scope,
) -> Result<HashMap<String, f64>> {
    if inputs.len() != def.inputs.len() {
        return Err(FreesError::evaluation(format!(
            "PROCEDURE {} expects {} input(s), got {}",
            def.name,
            def.inputs.len(),
            inputs.len()
        )));
    }
    let _guard = DepthGuard::enter("PROCEDURE", &def.name)?;
    let mut locals = caller_scope.clone();
    for (input, value) in def.inputs.iter().zip(inputs) {
        locals.insert(input.clone(), *value);
    }
    execute_body(&def.body, &mut locals, defs)?;
    let mut outputs = HashMap::with_capacity(def.outputs.len());
    for out in &def.outputs {
        match locals.get(out) {
            Some(value) => {
                outputs.insert(out.clone(), *value);
            }
            None => {
                return Err(FreesError::evaluation(format!(
                    "PROCEDURE {} never assigned output variable '{out}'",
                    def.name
                )))
            }
        }
    }
    Ok(outputs)
}

/// The synthetic per-output call name `proc$<name>$<k>`.
pub fn proc_output_name(proc_name: &str, output_index: usize) -> String {
    format!("{PROC_OUTPUT_PREFIX}{proc_name}${output_index}")
}

/// Split a synthetic `proc$<name>$<k>` call name, mirroring the
/// `split("\\$", 3)` in `Evaluator.evalProcedureOutput`. `None` when the name
/// is not of that shape.
pub fn parse_proc_output_name(function: &str) -> Option<(&str, usize)> {
    let rest = function.strip_prefix(PROC_OUTPUT_PREFIX)?;
    let (name, index) = rest.split_once('$')?;
    Some((name, index.parse().ok()?))
}

/// Evaluate one synthetic procedure-output call `proc$<name>$<k>(args…)`.
/// Port of `Evaluator.evalProcedureOutput`: runs the whole PROCEDURE body and
/// returns the value of output slot `k`. The expression evaluator dispatches
/// `proc$`-prefixed call names here.
///
/// # Not memoised — checked against the Java
///
/// `Evaluator.evalProcedureOutput` builds a **fresh** `ProcedureEvaluator` and
/// calls `callProcedure` on *every* `proc$name$k` it evaluates; there is no
/// cache anywhere on that path. An N-output PROCEDURE therefore runs its body
/// N times per residual sweep in the reference engine, and this port does the
/// same. The difference is observable, not merely a cost: a body that calls
/// `Random`/`RandG`, or one whose intermediate `=` equations the caller can see
/// through the returned scope, would diverge under memoisation. Parity wins
/// over the constant factor.
pub fn call_proc_output(
    function: &str,
    args: &[f64],
    defs: &Definitions,
    caller_scope: &Scope,
) -> Result<f64> {
    let unknown = || FreesError::evaluation(format!("Unknown procedure output call: {function}"));
    let (name, index) = parse_proc_output_name(function).ok_or_else(unknown)?;
    let def = defs.procedure(name).ok_or_else(unknown)?;
    let key = def.outputs.get(index).ok_or_else(unknown)?.clone();
    let outputs = call_procedure(def, args, defs, caller_scope)?;
    // `call_procedure` errors on a missing output, so the key is present.
    Ok(outputs[&key])
}

// ---------------------------------------------------------------------------
// Body execution (ProcedureEvaluator.executeBody / executeOne)
// ---------------------------------------------------------------------------

fn execute_body(body: &[ProcStatement], locals: &mut Scope, defs: &Definitions) -> Result<()> {
    for statement in body {
        execute_one(statement, locals, defs)?;
    }
    Ok(())
}

fn execute_one(statement: &ProcStatement, locals: &mut Scope, defs: &Definitions) -> Result<()> {
    match statement {
        ProcStatement::Assign { var_name, value } => {
            let value = eval_proc_expr(value, locals, defs)?;
            locals.insert(var_name.clone(), value);
        }

        ProcStatement::IfElse {
            condition,
            then_branch,
            else_branch,
        } => {
            if eval_proc_expr(condition, locals, defs)? != 0.0 {
                execute_body(then_branch, locals, defs)?;
            } else {
                execute_body(else_branch, locals, defs)?;
            }
        }

        ProcStatement::RepeatUntil { body, condition } => {
            let mut iterations: u64 = 0;
            loop {
                execute_body(body, locals, defs)?;
                iterations += 1;
                if iterations > MAX_ITERATIONS {
                    return Err(FreesError::evaluation(format!(
                        "REPEAT-UNTIL exceeded {MAX_ITERATIONS} iterations"
                    )));
                }
                if eval_proc_expr(condition, locals, defs)? != 0.0 {
                    break;
                }
            }
        }

        // An equation inside a body: treated as an assignment when one side is
        // a simple variable; otherwise it is informational (a no-op), exactly
        // like the Java arm.
        ProcStatement::Eq(Equation { lhs, rhs, .. }) => {
            if let Expr::Var(name) = lhs {
                let value = eval_proc_expr(rhs, locals, defs)?;
                locals.insert(name.clone(), value);
            } else if let Expr::Var(name) = rhs {
                let value = eval_proc_expr(lhs, locals, defs)?;
                locals.insert(name.clone(), value);
            }
        }

        // Java: bounds round to integers, the step is ±1 from their order, and
        // the loop is inclusive — `FOR i = 1 TO 0` runs i = 1, 0. Each
        // iteration executes on a copy of the locals that is merged back
        // afterwards (`locals.putAll(loopLocals)`), so the loop variable stays
        // visible after the loop with its final value.
        ProcStatement::For {
            var_name,
            start,
            end,
            body,
        } => {
            let start_val = eval_proc_expr(start, locals, defs)?;
            let end_val = eval_proc_expr(end, locals, defs)?;
            // `(int) Math.round(...)`: floor(x + 0.5); NaN → 0, ±inf saturate.
            let start_int = libm::floor(start_val + 0.5) as i64;
            let end_int = libm::floor(end_val + 0.5) as i64;
            let step: i64 = if start_int <= end_int { 1 } else { -1 };
            // i128 keeps `end + step` from overflowing at the i64 rim.
            let sentinel = end_int as i128 + step as i128;
            let mut i = start_int as i128;
            let mut iterations: u64 = 0;
            while i != sentinel {
                iterations += 1;
                if iterations > MAX_ITERATIONS {
                    // Deviation: Java has no FOR ceiling (see module docs).
                    return Err(FreesError::evaluation(format!(
                        "FOR loop exceeded {MAX_ITERATIONS} iterations"
                    )));
                }
                let mut loop_locals = locals.clone();
                loop_locals.insert(var_name.clone(), i as f64);
                execute_body(body, &mut loop_locals, defs)?;
                *locals = loop_locals;
                i += i128::from(step);
            }
        }

        ProcStatement::While { condition, body } => {
            let mut iterations: u64 = 0;
            while eval_proc_expr(condition, locals, defs)? != 0.0 {
                execute_body(body, locals, defs)?;
                iterations += 1;
                if iterations > MAX_ITERATIONS {
                    return Err(FreesError::evaluation(format!(
                        "WHILE loop exceeded {MAX_ITERATIONS} iterations"
                    )));
                }
            }
        }
    }
    Ok(())
}

/// Evaluate a body expression under `locals`, dispatching user-defined
/// FUNCTION calls through `defs` — the counterpart of the Java
/// `Evaluator.eval(e, locals, defs)` third argument.
///
/// User calls in strict positions are resolved to literals first
/// ([`resolve_user_calls`]) and the residue goes through
/// [`crate::eval::eval_with`] with the definitions in context, so calls under
/// *lazy* intrinsics (`if` branches, `sum`/`product` bodies) are the
/// evaluator's to dispatch — same division of labour as the Java engine, where
/// all dispatch lives in `Evaluator.evalCall`.
fn eval_proc_expr(expr: &Expr, locals: &Scope, defs: &Definitions) -> Result<f64> {
    let resolved = resolve_user_calls(expr, locals, defs)?;
    eval_with(&resolved, locals, EvalContext::with_defs(defs))
}

/// Rewrite `expr` with every user-`FUNCTION` call in a strict (eagerly
/// evaluated) position replaced by its computed value. Argument positions of
/// lazy intrinsics are left untouched — evaluating them here would break
/// `if`'s laziness (and with it recursion guarded by `if`) and `sum`/
/// `product`'s index binding.
fn resolve_user_calls(expr: &Expr, locals: &Scope, defs: &Definitions) -> Result<Expr> {
    Ok(match expr {
        Expr::Num { .. } | Expr::Str(_) | Expr::Var(_) => expr.clone(),

        Expr::Call { function, args } => {
            if let Some(def) = defs.function(function) {
                let mut values = Vec::with_capacity(args.len());
                for arg in args {
                    values.push(eval_proc_expr(arg, locals, defs)?);
                }
                return Ok(Expr::num(call_function(def, &values, defs, locals)?));
            }
            if is_lazy_call(function) {
                return Ok(expr.clone());
            }
            let mut resolved = Vec::with_capacity(args.len());
            for arg in args {
                resolved.push(resolve_user_calls(arg, locals, defs)?);
            }
            Expr::Call {
                function: function.clone(),
                args: resolved,
            }
        }

        Expr::Neg(inner) => Expr::Neg(Box::new(resolve_user_calls(inner, locals, defs)?)),
        Expr::Not(inner) => Expr::Not(Box::new(resolve_user_calls(inner, locals, defs)?)),
        Expr::BinOp { op, left, right } => Expr::BinOp {
            op: *op,
            left: Box::new(resolve_user_calls(left, locals, defs)?),
            right: Box::new(resolve_user_calls(right, locals, defs)?),
        },
        Expr::Compare { op, left, right } => Expr::Compare {
            op: *op,
            left: Box::new(resolve_user_calls(left, locals, defs)?),
            right: Box::new(resolve_user_calls(right, locals, defs)?),
        },
        Expr::Logical { op, left, right } => Expr::Logical {
            op: *op,
            left: Box::new(resolve_user_calls(left, locals, defs)?),
            right: Box::new(resolve_user_calls(right, locals, defs)?),
        },
        Expr::Range { start, end } => Expr::Range {
            start: Box::new(resolve_user_calls(start, locals, defs)?),
            end: Box::new(resolve_user_calls(end, locals, defs)?),
        },
        Expr::ArrayLiteral(elements) => Expr::ArrayLiteral(
            elements
                .iter()
                .map(|e| resolve_user_calls(e, locals, defs))
                .collect::<Result<_>>()?,
        ),
        Expr::ArrayAccess { name, indices } => Expr::ArrayAccess {
            name: name.clone(),
            indices: indices
                .iter()
                .map(|i| resolve_user_calls(i, locals, defs))
                .collect::<Result<_>>()?,
        },
    })
}

/// True for intrinsics whose arguments must not be eagerly rewritten: the
/// registered lazy forms (`if`, `sum`, `product`, string intrinsics) plus the
/// calculus binders, which bind their integration variable.
fn is_lazy_call(function: &str) -> bool {
    if matches!(function, "integral" | "gaussintegral") {
        return true;
    }
    matches!(
        lookup_intrinsic(function),
        Some(intrinsic) if matches!(intrinsic.body, Body::Lazy(_))
    )
}

// ---------------------------------------------------------------------------
// CALL flattening (EquationParser.flattenCallProc)
// ---------------------------------------------------------------------------

/// CALL targets whose flattener lives in [`crate::parser::expand`] (pipeline
/// stage 3), not here. They must pass through this stage unchanged — refusing
/// them would make the expansion-side flatteners unreachable, which is exactly
/// what happened to `LUDecompose` and `Interp2` before this list existed.
const EXPANDED_CALL_TARGETS: &[&str] = &[
    "ludecompose",
    "interp2",
    // The Java `LIN_ALG_SIGNAL_STATS_CALLS` set, flattened by
    // `expand::flatten_call_proc` into the `qr$`/`chol$`/`expm$`/`svd$`/
    // `fft$`/`ifft$`/`conv$`/`linfit$`/`polyfit$` kernel synthetics.
    "qr",
    "cholesky",
    "matexp",
    "singularvalues",
    "svd",
    "fft",
    "ifft",
    "convolve",
    "linfit",
    "polyfit",
    // Phase 4.5: the sensor kernels, flattened by `expand::flatten_call_proc`
    // into the `detrend$`/`smooth$`/`window$`/`filter$`/`xcorr$`/`welch$`
    // synthetics `eval::eval_synthetic` decodes.
    "detrend",
    "smooth",
    "window",
    "filter",
    "filtfilt",
    "xcorr",
    "welch",
    // Ledger item 34: the eigen pair, flattened by `expand::flatten_eigen`
    // into the `eigen$val|re|im|vec$…` synthetics `linalg::eval_intrinsic`
    // decodes.
    "eigenvalues",
    "eigen",
    // Phase 9: the control-systems suite. `expand::flatten_call_proc` hands
    // these to `control::flatten`, which emits the `ss2tf$`/`step$`/`lqr$`/…
    // synthetics `control::eval` decodes. The membership test below pins this
    // list against `control::flatten::CALL_NAMES` so the two cannot drift.
    "ss2tf",
    "ss2tfij",
    "tf2ss",
    "zp2tf",
    "tf2zp",
    "series",
    "parallel",
    "feedback",
    "pole",
    "zero",
    "bode",
    "nyquist",
    "margin",
    "step",
    "impulse",
    "lsim",
    "lqr",
    "dlqr",
    "dare",
    "lyap",
    "dlyap",
    "place",
    "acker",
    "lqe",
    "gram",
    "balreal",
    "pidtune",
    "rank",
    "ctrb",
    "obsv",
    "ss2ss",
    "stepinfo",
    "pade",
    "rlocus",
    "routh",
    "c2d",
    "d2c",
    "residue",
    "nichols",
    "errorconst",
    // `mason` is a real Java CALL intrinsic (`flattenMason` → `mason$<n>` →
    // `evalMason`) that no earlier list named, so it was refused here and
    // never reached the flattener at all.
    "mason",
];

/// CALL targets the Java `flattenCallProc` implements as intrinsics that this
/// port does not. Naming them keeps the refusal honest — "not yet supported",
/// not "unknown". (The eigen pair left for `EXPANDED_CALL_TARGETS` when
/// ledger item 34 closed.)
const INTRINSIC_CALL_TARGETS: &[&str] = &["eulerdecompose", "eulerrotate"];

/// Per-flatten module instance counter — the Java engine uses a shared
/// `AtomicInteger` in the flatten context; a per-call counter is deterministic
/// across runs, which the parity harness needs. First instance is `1`
/// (`incrementAndGet`).
struct ModuleCounter(u32);

/// Display names the CALL flattener *generates*, in the two places the Java
/// `EquationParser` does: `flattenModuleCall` registers each namespaced
/// parameter and each namespaced body variable as its own display name
/// (`putIfAbsent(nsParam, nsParam)`, `nsLhs.variables().forEach(…)`), and
/// `flattenProcedureCall` re-registers the CALL's output variables (already
/// recorded at parse time, so that one is a no-op here).
type GeneratedNames = BTreeMap<String, String>;

/// Flatten `CALL name(inputs : outputs)` statements into binding equations
/// (`Statement::CallProc` — "At flatten time this generates equations that
/// bind the outputs", `ast/Statement.java`). Non-CALL statements pass through
/// untouched; a CALL to an unknown name is an error naming it.
///
/// * **PROCEDURE**: one `out_k = proc$name$k(inputs…)` equation per output
///   slot, inputs symbolic (`flattenProcedureCall`).
/// * **MODULE**: input bindings, namespaced body equations, output bindings
///   (`flattenModuleCall`).
/// * A `FUNCTION` (or `TABLE`) name is refused with the Java message — those
///   are called inline in expressions, not with CALL.
///
/// CALLs inside `FOR` bodies are flattened in place when the loop has not been
/// unrolled yet; that is exact for PROCEDURE calls (the generated equations
/// keep the loop variable symbolic). A MODULE call inside a FOR **passes
/// through untouched** instead: Java instantiates one namespace per
/// *iteration* (its flattening runs after unrolling), so the instantiation
/// belongs to the expansion stage, which owns the unroller —
/// `parser::expand::Flattener::flatten_module_call`, seeded with this stage's
/// final instance count so the numbering continues where it left off.
///
/// One recorded numbering divergence survives the split: a top-level MODULE
/// call written *after* a FOR containing MODULE calls is numbered here
/// (before the FOR's instances) where Java numbers it after them. No corpus
/// document has that shape; the harvest boundary is the reason.
pub fn flatten_calls(statements: Vec<Statement>, defs: &Definitions) -> Result<Vec<Statement>> {
    flatten_calls_into(statements, defs, &mut BTreeMap::new())
}

/// [`flatten_calls`], additionally accumulating the display names the flatten
/// step generates (see [`GeneratedNames`]). The solve path threads its
/// document map through here so a MODULE's namespaced variables surface with
/// the same spelling the Java engine reports.
pub fn flatten_calls_into(
    statements: Vec<Statement>,
    defs: &Definitions,
    display_names: &mut BTreeMap<String, String>,
) -> Result<Vec<Statement>> {
    Ok(flatten_calls_counted(statements, defs, display_names)?.0)
}

/// [`flatten_calls_into`], additionally returning the number of MODULE
/// instances this stage created — the base the expansion stage's per-iteration
/// module instantiation continues from, so the two stages share one numbering
/// exactly as the Java's single `moduleCounter` does.
pub fn flatten_calls_counted(
    statements: Vec<Statement>,
    defs: &Definitions,
    display_names: &mut BTreeMap<String, String>,
) -> Result<(Vec<Statement>, u32)> {
    let mut counter = ModuleCounter(0);
    let mut out = Vec::with_capacity(statements.len());
    for statement in statements {
        flatten_statement(
            statement,
            defs,
            &mut counter,
            false,
            &mut out,
            display_names,
        )?;
    }
    Ok((out, counter.0))
}

fn flatten_statement(
    statement: Statement,
    defs: &Definitions,
    counter: &mut ModuleCounter,
    inside_for: bool,
    out: &mut Vec<Statement>,
    display_names: &mut GeneratedNames,
) -> Result<()> {
    match statement {
        Statement::CallProc {
            name,
            inputs,
            outputs,
            source_text,
        } => flatten_call(
            &name,
            inputs,
            outputs,
            &source_text,
            defs,
            counter,
            inside_for,
            out,
            display_names,
        ),
        Statement::For {
            var_name,
            start,
            end,
            body,
        } => {
            let mut inner = Vec::with_capacity(body.len());
            for s in body {
                flatten_statement(s, defs, counter, true, &mut inner, display_names)?;
            }
            out.push(Statement::For {
                var_name,
                start,
                end,
                body: inner,
            });
            Ok(())
        }
        other => {
            out.push(other);
            Ok(())
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn flatten_call(
    name: &str,
    inputs: Vec<Expr>,
    outputs: Vec<Expr>,
    source_text: &str,
    defs: &Definitions,
    counter: &mut ModuleCounter,
    inside_for: bool,
    out: &mut Vec<Statement>,
    display_names: &mut GeneratedNames,
) -> Result<()> {
    if let Some(def) = defs.procedure(name) {
        return flatten_procedure_call(def, inputs, outputs, source_text, out, display_names);
    }
    if let Some(def) = defs.module(name) {
        if inside_for {
            // See the `flatten_calls` doc — Java instantiates one namespace
            // per iteration, so the CALL rides through to the expansion
            // stage, whose unroller instantiates it with the loop variable
            // bound (`Flattener::flatten_module_call`).
            out.push(Statement::CallProc {
                name: name.to_string(),
                inputs,
                outputs,
                source_text: source_text.to_string(),
            });
            return Ok(());
        }
        return flatten_module_call(def, inputs, outputs, counter, out, display_names);
    }
    if defs.function(name).is_some() || defs.table(name).is_some() {
        // FunctionTableDef falls into the same `default` arm in the Java switch.
        return Err(FreesError::parse(format!(
            "'{name}' is a FUNCTION, not callable with CALL (use it directly in \
             an expression)"
        )));
    }
    if EXPANDED_CALL_TARGETS.contains(&name) {
        // The Java flattens PROCEDURE/MODULE calls and the matrix intrinsics in
        // one pass (`flattenCallProc`); this port splits that in two, so a CALL
        // whose flattener lives in `parser::expand` has to survive this stage
        // untouched instead of being refused here.
        out.push(Statement::CallProc {
            name: name.to_string(),
            inputs,
            outputs,
            source_text: source_text.to_string(),
        });
        return Ok(());
    }
    if INTRINSIC_CALL_TARGETS.contains(&name) {
        // Same refusal (message and kind) the Phase-3 stub gave every CALL.
        return Err(FreesError::evaluation(format!(
            "CALL `{name}` is not yet supported by the wasm engine"
        )));
    }
    Err(FreesError::parse(format!(
        "Unknown PROCEDURE or MODULE: '{name}'"
    )))
}

/// Port of `flattenProcedureCall`: `out_k = proc$name$k(inputs…)` per output.
fn flatten_procedure_call(
    def: &ProcedureDef,
    inputs: Vec<Expr>,
    outputs: Vec<Expr>,
    source_text: &str,
    out: &mut Vec<Statement>,
    display_names: &mut GeneratedNames,
) -> Result<()> {
    if outputs.len() != def.outputs.len() {
        return Err(FreesError::parse(format!(
            "CALL {} provides {} output variable(s) but PROCEDURE declares {}",
            def.name,
            outputs.len(),
            def.outputs.len()
        )));
    }
    for (k, output) in outputs.into_iter().enumerate() {
        let Expr::Var(var_name) = &output else {
            return Err(output_not_a_variable(source_text));
        };
        // `flattenProcedureCall`: putIfAbsent(varName, varName).
        display_names
            .entry(var_name.clone())
            .or_insert_with(|| var_name.clone());
        let call = Expr::Call {
            function: proc_output_name(&def.name, k),
            args: inputs.clone(),
        };
        out.push(Statement::Eq(Equation::new(
            output,
            call,
            format!("CALL {}", def.name),
        )));
    }
    Ok(())
}

/// Port of `flattenModuleCall`: input bindings, namespaced body equations,
/// output bindings. The namespace is `<name>$<instance>$` (e.g. `heatex$1$`).
fn flatten_module_call(
    def: &ModuleDef,
    inputs: Vec<Expr>,
    outputs: Vec<Expr>,
    counter: &mut ModuleCounter,
    out: &mut Vec<Statement>,
    display_names: &mut GeneratedNames,
) -> Result<()> {
    // Java increments before the arity checks; a failing CALL still consumed
    // an instance number.
    counter.0 += 1;
    let ns = format!("{}${}$", def.name, counter.0);

    if inputs.len() != def.inputs.len() {
        return Err(FreesError::parse(format!(
            "CALL {} provides {} input(s) but MODULE declares {}",
            def.name,
            inputs.len(),
            def.inputs.len()
        )));
    }
    if outputs.len() != def.outputs.len() {
        return Err(FreesError::parse(format!(
            "CALL {} provides {} output variable(s) but MODULE declares {}",
            def.name,
            outputs.len(),
            def.outputs.len()
        )));
    }

    // Input binding equations: ns$param = inputExpr.
    for (param, input) in def.inputs.iter().zip(inputs) {
        let ns_param = format!("{ns}{param}");
        // `flattenModuleCall`: putIfAbsent(nsParam, nsParam) — a namespaced
        // name is its own display spelling.
        display_names
            .entry(ns_param.clone())
            .or_insert_with(|| ns_param.clone());
        out.push(Statement::Eq(Equation::new(
            Expr::Var(ns_param),
            input,
            format!("MODULE {} input {param}", def.name),
        )));
    }

    // Module body equations with namespaced variables. Java grafts
    // `Statement.Eq` bodies and **silently drops** anything else; dropping
    // equations changes the degrees of freedom, so this port refuses instead
    // (deviation — loud, not silent).
    for body_statement in &def.body {
        match body_statement {
            Statement::Eq(eq) => {
                let lhs = namespace_expr(&eq.lhs, &ns);
                let rhs = namespace_expr(&eq.rhs, &ns);
                // `nsLhs.variables().forEach(v -> putIfAbsent(v, v))`, same for
                // the RHS: every namespaced body variable displays as itself.
                for var in lhs.variables().into_iter().chain(rhs.variables()) {
                    display_names.entry(var.clone()).or_insert(var);
                }
                out.push(Statement::Eq(Equation::new(
                    lhs,
                    rhs,
                    eq.source_text.clone(),
                )));
            }
            other => {
                let kind = match other {
                    Statement::For { .. } => "a FOR block",
                    Statement::CallProc { .. } => "a CALL",
                    Statement::Symbolic(_) => "a SYMBOLIC declaration",
                    Statement::Eq(_) => unreachable!(),
                };
                return Err(FreesError::parse(format!(
                    "MODULE {} body contains {kind}; only `=` equations can be \
                     grafted into the caller's system",
                    def.name
                )));
            }
        }
    }

    // Output binding equations: outputVar = ns$outputParam.
    for (param, output) in def.outputs.iter().zip(outputs) {
        let Expr::Var(var_name) = &output else {
            return Err(output_not_a_variable(&format!("CALL {}", def.name)));
        };
        display_names
            .entry(var_name.clone())
            .or_insert_with(|| var_name.clone());
        out.push(Statement::Eq(Equation::new(
            output,
            Expr::Var(format!("{ns}{param}")),
            format!("MODULE {} output {param}", def.name),
        )));
    }
    Ok(())
}

fn output_not_a_variable(context: &str) -> FreesError {
    FreesError::parse(format!(
        "CALL output argument must resolve to a variable (in `{context}`)"
    ))
}

/// Rewrite every variable (and array name) in `expr` into the module instance
/// namespace. Port of `namespaceExpr` — note it renames **every** variable,
/// including the declared inputs/outputs (they connect to the caller through
/// the binding equations), and leaves call *names* alone so intrinsics and
/// user functions still resolve.
pub(crate) fn namespace_expr(expr: &Expr, ns: &str) -> Expr {
    match expr {
        Expr::Num { .. } | Expr::Str(_) => expr.clone(),
        Expr::Var(name) => Expr::Var(format!("{ns}{name}")),
        Expr::Neg(inner) => Expr::Neg(Box::new(namespace_expr(inner, ns))),
        Expr::Not(inner) => Expr::Not(Box::new(namespace_expr(inner, ns))),
        Expr::BinOp { op, left, right } => Expr::BinOp {
            op: *op,
            left: Box::new(namespace_expr(left, ns)),
            right: Box::new(namespace_expr(right, ns)),
        },
        Expr::Compare { op, left, right } => Expr::Compare {
            op: *op,
            left: Box::new(namespace_expr(left, ns)),
            right: Box::new(namespace_expr(right, ns)),
        },
        Expr::Logical { op, left, right } => Expr::Logical {
            op: *op,
            left: Box::new(namespace_expr(left, ns)),
            right: Box::new(namespace_expr(right, ns)),
        },
        Expr::Call { function, args } => Expr::Call {
            function: function.clone(),
            args: args.iter().map(|a| namespace_expr(a, ns)).collect(),
        },
        Expr::ArrayAccess { name, indices } => Expr::ArrayAccess {
            name: format!("{ns}{name}"),
            indices: indices.iter().map(|i| namespace_expr(i, ns)).collect(),
        },
        Expr::Range { start, end } => Expr::Range {
            start: Box::new(namespace_expr(start, ns)),
            end: Box::new(namespace_expr(end, ns)),
        },
        Expr::ArrayLiteral(elements) => {
            Expr::ArrayLiteral(elements.iter().map(|e| namespace_expr(e, ns)).collect())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_document;

    fn defs_of(source: &str) -> Definitions {
        parse_document(source)
            .unwrap_or_else(|e| panic!("`{source}` should parse: {e}"))
            .defs
    }

    fn empty_scope() -> Scope {
        Scope::default()
    }

    // ── call_function: the ProceduralFeaturesTest oracle cases ──────────────

    #[test]
    fn function_factorial_recurses_to_120() {
        let defs = defs_of(
            "FUNCTION Factorial(n)\n  IF n <= 1 THEN\n    Factorial := 1\n  ELSE\n    Factorial := n * Factorial(n-1)\n  END\nEND",
        );
        let f = defs.function("factorial").unwrap();
        let y = call_function(f, &[5.0], &defs, &empty_scope()).unwrap();
        assert_eq!(y, 120.0, "5! should be 120");
    }

    #[test]
    fn function_simple_conditional() {
        let defs = defs_of(
            "FUNCTION AbsVal(x)\n  IF x >= 0 THEN\n    AbsVal := x\n  ELSE\n    AbsVal := -x\n  END\nEND",
        );
        let f = defs.function("absval").unwrap();
        assert_eq!(
            call_function(f, &[-7.0], &defs, &empty_scope()).unwrap(),
            7.0
        );
        assert_eq!(
            call_function(f, &[3.0], &defs, &empty_scope()).unwrap(),
            3.0
        );
    }

    #[test]
    fn function_repeat_until_sums_1_to_10() {
        let defs = defs_of(
            "FUNCTION SumN(n)\n  i := 1\n  s := 0\n  REPEAT\n    s := s + i\n    i := i + 1\n  UNTIL i > n\n  SumN := s\nEND",
        );
        let f = defs.function("sumn").unwrap();
        assert_eq!(
            call_function(f, &[10.0], &defs, &empty_scope()).unwrap(),
            55.0
        );
    }

    #[test]
    fn function_while_loop_sums_1_to_10() {
        let defs = defs_of(
            "FUNCTION SumWhile(n)\n  i := 1\n  s := 0\n  WHILE i <= n DO\n    s := s + i\n    i := i + 1\n  END\n  SumWhile := s\nEND",
        );
        let f = defs.function("sumwhile").unwrap();
        assert_eq!(
            call_function(f, &[10.0], &defs, &empty_scope()).unwrap(),
            55.0
        );
    }

    #[test]
    fn nested_for_inside_function_accumulates() {
        // Oracle: DoubleSum(3) = sum over i,j in 1..3 of i*j = 36. The FOR
        // bodies are `=` equations, executed as assignments.
        let defs = defs_of(
            "FUNCTION DoubleSum(n)\n  s := 0\n  FOR i = 1 TO n\n    FOR j = 1 TO n\n      s = s + i * j\n    END\n  END\n  DoubleSum := s\nEND",
        );
        let f = defs.function("doublesum").unwrap();
        assert_eq!(
            call_function(f, &[3.0], &defs, &empty_scope()).unwrap(),
            36.0
        );
    }

    #[test]
    fn functions_may_call_other_functions() {
        let defs = defs_of(
            "FUNCTION Square(x)\n  Square := x * x\nEND\nFUNCTION SumSquares(a, b)\n  SumSquares := Square(a) + Square(b)\nEND",
        );
        let f = defs.function("sumsquares").unwrap();
        assert_eq!(
            call_function(f, &[4.0, 3.0], &defs, &empty_scope()).unwrap(),
            25.0
        );
    }

    #[test]
    fn the_caller_scope_is_visible_inside_the_body() {
        // ProcedureEvaluator copies outerValues into the locals.
        let defs = defs_of("FUNCTION AddK(x)\n  AddK := x + k\nEND");
        let f = defs.function("addk").unwrap();
        let mut scope = empty_scope();
        scope.insert("k".into(), 100.0);
        assert_eq!(call_function(f, &[1.0], &defs, &scope).unwrap(), 101.0);
    }

    #[test]
    fn a_descending_for_loop_runs_inclusive() {
        // Java: step = -1 when start > end; `FOR i = 3 TO 1` runs 3, 2, 1.
        let defs =
            defs_of("FUNCTION CountDown(n)\n  s := 0\n  FOR i = n TO 1\n    s = s + i\n  END\n  CountDown := s\nEND");
        let f = defs.function("countdown").unwrap();
        assert_eq!(
            call_function(f, &[3.0], &defs, &empty_scope()).unwrap(),
            6.0
        );
    }

    #[test]
    fn the_loop_variable_survives_the_loop() {
        // `locals.putAll(loopLocals)` copies the loop variable back out.
        let defs =
            defs_of("FUNCTION LastI(n)\n  FOR i = 1 TO n\n    x = i\n  END\n  LastI := i\nEND");
        let f = defs.function("lasti").unwrap();
        assert_eq!(
            call_function(f, &[4.0], &defs, &empty_scope()).unwrap(),
            4.0
        );
    }

    // ── error paths ─────────────────────────────────────────────────────────

    #[test]
    fn wrong_arity_names_the_function_and_counts() {
        let defs = defs_of("FUNCTION f(a, b)\n  f := a + b\nEND");
        let f = defs.function("f").unwrap();
        let err = call_function(f, &[1.0], &defs, &empty_scope()).unwrap_err();
        assert!(
            err.to_string()
                .contains("FUNCTION f expects 2 argument(s), got 1"),
            "{err}"
        );
    }

    #[test]
    fn a_function_that_never_assigns_its_name_is_an_error() {
        let defs = defs_of("FUNCTION f(x)\n  y := x\nEND");
        let f = defs.function("f").unwrap();
        let err = call_function(f, &[1.0], &defs, &empty_scope()).unwrap_err();
        assert!(
            err.to_string()
                .contains("FUNCTION f never assigned a return value ('f := ...' missing)"),
            "{err}"
        );
    }

    #[test]
    fn runaway_while_and_repeat_loops_hit_the_java_ceiling() {
        let defs = defs_of("FUNCTION f(x)\n  WHILE 1 > 0 DO\n    y := 1\n  END\n  f := y\nEND");
        let err =
            call_function(defs.function("f").unwrap(), &[0.0], &defs, &empty_scope()).unwrap_err();
        assert!(
            err.to_string()
                .contains("WHILE loop exceeded 100000 iterations"),
            "{err}"
        );

        let defs = defs_of("FUNCTION g(x)\n  REPEAT\n    y := 1\n  UNTIL 0 > 1\n  g := y\nEND");
        let err =
            call_function(defs.function("g").unwrap(), &[0.0], &defs, &empty_scope()).unwrap_err();
        assert!(
            err.to_string()
                .contains("REPEAT-UNTIL exceeded 100000 iterations"),
            "{err}"
        );
    }

    #[test]
    fn baseless_recursion_hits_the_depth_ceiling_instead_of_the_stack() {
        // A dedicated big stack decouples this test from debug-build frame
        // sizes: it asserts the *guard* fires, not that 64 levels squeeze
        // into a particular test-thread stack.
        let err = std::thread::Builder::new()
            .stack_size(16 * 1024 * 1024)
            .spawn(|| {
                let defs = defs_of("FUNCTION Loop(n)\n  Loop := Loop(n) + 1\nEND");
                call_function(
                    defs.function("loop").unwrap(),
                    &[1.0],
                    &defs,
                    &empty_scope(),
                )
                .unwrap_err()
            })
            .expect("spawn")
            .join()
            .expect("join");
        assert!(
            err.to_string().contains("nested more than 64 levels"),
            "{err}"
        );
        // The guard unwinds cleanly: a fresh call still gets the full budget.
        let defs2 = defs_of(
            "FUNCTION Fact(n)\n  IF n <= 1 THEN\n    Fact := 1\n  ELSE\n    Fact := n * Fact(n-1)\n  END\nEND",
        );
        assert_eq!(
            call_function(
                defs2.function("fact").unwrap(),
                &[6.0],
                &defs2,
                &empty_scope()
            )
            .unwrap(),
            720.0
        );
    }

    #[test]
    fn an_equation_with_no_variable_side_is_informational() {
        // Java: `1 + 1 = 2` inside a body assigns nothing and errors nothing.
        let defs = defs_of("FUNCTION f(x)\n  1 + 1 = 2\n  f := x\nEND");
        assert_eq!(
            call_function(defs.function("f").unwrap(), &[9.0], &defs, &empty_scope()).unwrap(),
            9.0
        );
    }

    // ── call_procedure ──────────────────────────────────────────────────────

    #[test]
    fn procedure_swap_binds_both_outputs() {
        let defs = defs_of("PROCEDURE Swap(a, b : c, d)\n  c := b\n  d := a\nEND");
        let p = defs.procedure("swap").unwrap();
        let outs = call_procedure(p, &[3.0, 7.0], &defs, &empty_scope()).unwrap();
        assert_eq!(outs["c"], 7.0);
        assert_eq!(outs["d"], 3.0);
    }

    #[test]
    fn procedure_with_conditional_orders_min_max() {
        let defs = defs_of(
            "PROCEDURE MinMax(a, b : lo, hi)\n  IF a < b THEN\n    lo := a\n    hi := b\n  ELSE\n    lo := b\n    hi := a\n  END\nEND",
        );
        let p = defs.procedure("minmax").unwrap();
        let outs = call_procedure(p, &[8.0, 3.0], &defs, &empty_scope()).unwrap();
        assert_eq!((outs["lo"], outs["hi"]), (3.0, 8.0));
    }

    #[test]
    fn a_missing_output_and_a_wrong_input_count_are_named() {
        let defs = defs_of("PROCEDURE p(a : x, y)\n  x := a\nEND");
        let p = defs.procedure("p").unwrap();
        let err = call_procedure(p, &[1.0], &defs, &empty_scope()).unwrap_err();
        assert!(
            err.to_string()
                .contains("PROCEDURE p never assigned output variable 'y'"),
            "{err}"
        );
        let err = call_procedure(p, &[1.0, 2.0], &defs, &empty_scope()).unwrap_err();
        assert!(
            err.to_string()
                .contains("PROCEDURE p expects 1 input(s), got 2"),
            "{err}"
        );
    }

    // ── the synthetic proc$ output calls ────────────────────────────────────

    #[test]
    fn proc_output_names_round_trip() {
        assert_eq!(proc_output_name("swap", 1), "proc$swap$1");
        assert_eq!(parse_proc_output_name("proc$swap$1"), Some(("swap", 1)));
        assert_eq!(parse_proc_output_name("proc$swap$x"), None);
        assert_eq!(parse_proc_output_name("swap$1"), None);
        assert_eq!(parse_proc_output_name("proc$swap"), None);
    }

    #[test]
    fn call_proc_output_runs_the_body_and_picks_the_slot() {
        let defs = defs_of("PROCEDURE Swap(a, b : c, d)\n  c := b\n  d := a\nEND");
        let scope = empty_scope();
        assert_eq!(
            call_proc_output("proc$swap$0", &[3.0, 7.0], &defs, &scope).unwrap(),
            7.0
        );
        assert_eq!(
            call_proc_output("proc$swap$1", &[3.0, 7.0], &defs, &scope).unwrap(),
            3.0
        );
        let err = call_proc_output("proc$swap$9", &[3.0, 7.0], &defs, &scope).unwrap_err();
        assert!(
            err.to_string().contains("Unknown procedure output call"),
            "{err}"
        );
        let err = call_proc_output("proc$nope$0", &[], &defs, &scope).unwrap_err();
        assert!(
            err.to_string().contains("Unknown procedure output call"),
            "{err}"
        );
    }

    #[test]
    fn a_multi_output_function_executes_as_a_procedure() {
        // Oracle `multiOutputFunctionBasic`: DivMod(17, 5) → 3, 2.
        let defs =
            defs_of("FUNCTION [q, r] = DivMod(a, b)\n  q := trunc(a / b)\n  r := a - q * b\nEND");
        let p = defs.procedure("divmod").expect("desugared to a PROCEDURE");
        let outs = call_procedure(p, &[17.0, 5.0], &defs, &empty_scope()).unwrap();
        assert_eq!((outs["q"], outs["r"]), (3.0, 2.0));
    }

    // ── flatten_calls ───────────────────────────────────────────────────────

    fn flattened(source: &str) -> Vec<Statement> {
        let doc = parse_document(source).unwrap();
        flatten_calls(doc.statements, &doc.defs).unwrap()
    }

    fn eq_texts(statements: &[Statement]) -> Vec<String> {
        statements
            .iter()
            .map(|s| match s {
                Statement::Eq(eq) => eq.source_text.clone(),
                other => panic!("expected an equation, got {other:?}"),
            })
            .collect()
    }

    #[test]
    fn a_procedure_call_flattens_to_one_synthetic_equation_per_output() {
        let statements = flattened(
            "PROCEDURE Swap(a, b : c, d)\n  c := b\n  d := a\nEND\nCALL Swap(3, 7 : x, y)",
        );
        assert_eq!(statements.len(), 2);
        match &statements[0] {
            Statement::Eq(eq) => {
                assert_eq!(eq.lhs, Expr::var("x"));
                assert_eq!(
                    eq.rhs,
                    Expr::Call {
                        function: "proc$swap$0".into(),
                        args: vec![Expr::num(3.0), Expr::num(7.0)],
                    }
                );
                assert_eq!(eq.source_text, "CALL swap");
            }
            other => panic!("{other:?}"),
        }
        match &statements[1] {
            Statement::Eq(eq) => {
                assert_eq!(eq.lhs, Expr::var("y"));
                assert!(
                    matches!(&eq.rhs, Expr::Call { function, .. } if function == "proc$swap$1")
                );
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn procedure_inputs_stay_symbolic() {
        // The Java flattener does NOT evaluate inputs; `proc$…` calls carry
        // the expressions and Newton evaluates them per iterate.
        let statements =
            flattened("PROCEDURE p(a : b)\n  b := a * 2\nEND\nCALL p(q + 1 : w)\nq = 4");
        match &statements[0] {
            Statement::Eq(eq) => match &eq.rhs {
                Expr::Call { args, .. } => {
                    assert_eq!(args[0].variables().into_iter().collect::<Vec<_>>(), ["q"]);
                }
                other => panic!("{other:?}"),
            },
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn a_module_call_grafts_bindings_and_namespaced_body() {
        let statements = flattened("MODULE Doubler(x : y)\n  y = 2 * x\nEND\nCALL Doubler(5 : a)");
        assert_eq!(
            eq_texts(&statements),
            vec![
                "MODULE doubler input x",
                "y = 2 * x",
                "MODULE doubler output y"
            ]
        );
        // ns$x = 5
        match &statements[0] {
            Statement::Eq(eq) => {
                assert_eq!(eq.lhs, Expr::Var("doubler$1$x".into()));
                assert_eq!(eq.rhs, Expr::num(5.0));
            }
            other => panic!("{other:?}"),
        }
        // doubler$1$y = 2 * doubler$1$x
        match &statements[1] {
            Statement::Eq(eq) => {
                assert_eq!(eq.lhs, Expr::Var("doubler$1$y".into()));
                let vars: Vec<_> = eq.rhs.variables().into_iter().collect();
                assert_eq!(vars, ["doubler$1$x"]);
            }
            other => panic!("{other:?}"),
        }
        // a = doubler$1$y
        match &statements[2] {
            Statement::Eq(eq) => {
                assert_eq!(eq.lhs, Expr::var("a"));
                assert_eq!(eq.rhs, Expr::Var("doubler$1$y".into()));
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn each_module_call_gets_its_own_instance_namespace() {
        // Oracle `moduleBasicGrafting`: two calls → two namespaced copies.
        let statements = flattened(
            "MODULE Doubler(x : y)\n  y = 2 * x\nEND\nCALL Doubler(5 : a)\nCALL Doubler(10 : b)",
        );
        assert_eq!(statements.len(), 6);
        let all_vars: std::collections::BTreeSet<String> = statements
            .iter()
            .flat_map(|s| match s {
                Statement::Eq(eq) => eq.variables(),
                other => panic!("{other:?}"),
            })
            .collect();
        assert!(all_vars.contains("doubler$1$x") && all_vars.contains("doubler$1$y"));
        assert!(all_vars.contains("doubler$2$x") && all_vars.contains("doubler$2$y"));
    }

    #[test]
    fn module_bodies_namespace_negation_and_call_arguments_but_not_names() {
        // Oracle `moduleNamespacesFunctionCallsAndNegation`.
        let statements = flattened(
            "MODULE Shift(x : y)\n  buf = -x + sin(0)\n  y = buf + 1\nEND\nCALL Shift(4 : out)",
        );
        match &statements[1] {
            Statement::Eq(eq) => {
                assert_eq!(eq.lhs, Expr::Var("shift$1$buf".into()));
                // -x namespaced; the sin *name* untouched.
                let vars: Vec<_> = eq.rhs.variables().into_iter().collect();
                assert_eq!(vars, ["shift$1$x"]);
                let rendered = format!("{:?}", eq.rhs);
                assert!(rendered.contains("\"sin\""), "{rendered}");
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn multi_assign_flattens_like_an_explicit_call() {
        let statements = flattened(
            "FUNCTION [q, r] = DivMod(a, b)\n  q := trunc(a / b)\n  r := a - q * b\nEND\n[whole, rem] = DivMod(17, 5)",
        );
        assert_eq!(statements.len(), 2);
        match &statements[0] {
            Statement::Eq(eq) => {
                assert_eq!(eq.lhs, Expr::var("whole"));
                assert!(
                    matches!(&eq.rhs, Expr::Call { function, .. } if function == "proc$divmod$0")
                );
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn non_call_statements_pass_through_untouched() {
        let doc = parse_document("x = 1\nSYMBOLIC s\nFOR i = 1 TO 2\n  y[i] = i\nEND").unwrap();
        let before = doc.statements.clone();
        let after = flatten_calls(doc.statements, &doc.defs).unwrap();
        assert_eq!(before, after);
    }

    #[test]
    fn a_procedure_call_inside_a_for_flattens_in_place() {
        // The generated equation keeps the loop variable symbolic, so a later
        // FOR unroll substitutes it exactly as the Java pipeline does.
        let statements =
            flattened("PROCEDURE p(a : b)\n  b := a\nEND\nFOR i = 1 TO 2\n  CALL p(i : w)\nEND");
        match &statements[0] {
            Statement::For { body, .. } => match &body[0] {
                Statement::Eq(eq) => {
                    assert_eq!(eq.lhs, Expr::var("w"));
                    match &eq.rhs {
                        Expr::Call { function, args } => {
                            assert_eq!(function, "proc$p$0");
                            assert_eq!(args, &[Expr::var("i")]);
                        }
                        other => panic!("{other:?}"),
                    }
                }
                other => panic!("{other:?}"),
            },
            other => panic!("{other:?}"),
        }
    }

    /// A MODULE call inside a FOR is no longer refused here (Wave A4): it
    /// rides through with the CALL intact so the expansion stage — which owns
    /// the unroller — can instantiate one namespace per iteration, exactly as
    /// Java's flatten-after-unroll does. Fixture `module_inside_for_loop` is
    /// the end-to-end witness.
    #[test]
    fn a_module_call_inside_a_for_passes_through_to_the_expansion_stage() {
        let doc =
            parse_document("MODULE m(x : y)\n  y = x\nEND\nFOR i = 1 TO 2\n  CALL m(i : w)\nEND")
                .unwrap();
        let (out, module_count) =
            flatten_calls_counted(doc.statements, &doc.defs, &mut BTreeMap::new()).unwrap();
        // No instance was created at this stage…
        assert_eq!(module_count, 0);
        // …and the CALL survives inside the FOR body for stage 3.
        match out.as_slice() {
            [Statement::For { body, .. }] => {
                assert!(
                    matches!(body.as_slice(), [Statement::CallProc { name, .. }] if name == "m"),
                    "{body:?}"
                );
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn calling_a_function_or_table_with_call_is_refused_with_the_java_message() {
        let doc = parse_document("FUNCTION f(x)\n  f := x\nEND\nCALL f(1 : y)").unwrap();
        let err = flatten_calls(doc.statements, &doc.defs).unwrap_err();
        assert!(
            err.to_string()
                .contains("'f' is a FUNCTION, not callable with CALL"),
            "{err}"
        );

        let doc = parse_document("TABLE t(x)\n  1 2\nEND\nCALL t(1 : y)").unwrap();
        let err = flatten_calls(doc.statements, &doc.defs).unwrap_err();
        assert!(err.to_string().contains("'t' is a FUNCTION"), "{err}");
    }

    #[test]
    fn an_unknown_call_name_is_the_java_unknown_error() {
        let doc = parse_document("CALL mystery(1 : y)").unwrap();
        let err = flatten_calls(doc.statements, &doc.defs).unwrap_err();
        assert!(
            err.to_string()
                .contains("Unknown PROCEDURE or MODULE: 'mystery'"),
            "{err}"
        );
    }

    #[test]
    fn an_unported_intrinsic_call_keeps_the_refusal_message() {
        for name in ["eulerdecompose", "eulerrotate"] {
            let doc = parse_document(&format!("[a, b] = {name}(m, n)")).unwrap();
            let err = flatten_calls(doc.statements, &doc.defs).unwrap_err();
            assert!(
                err.to_string()
                    .contains(&format!("CALL `{name}` is not yet supported")),
                "{err}"
            );
        }
    }

    /// The dense linear-algebra / signal / statistics CALLs are flattened in
    /// stage 3 ([`crate::parser::expand`]), so this stage must let them through
    /// untouched — refusing them here would make those flatteners unreachable,
    /// exactly as happened to `LUDecompose`/`Interp2` before this list existed.
    #[test]
    fn kernel_intrinsic_calls_pass_through_to_the_expansion_stage() {
        for name in [
            "qr",
            "cholesky",
            "matexp",
            "singularvalues",
            "svd",
            "eigenvalues",
            "eigen",
            "fft",
            "ifft",
            "convolve",
            "linfit",
            "polyfit",
            "ludecompose",
            "interp2",
            "detrend",
            "smooth",
            "window",
            "filter",
            "filtfilt",
            "xcorr",
            "welch",
        ]
        .into_iter()
        // Phase 9 put the control-systems suite on the same route.
        .chain(crate::control::flatten::CALL_NAMES)
        .chain(["mason"])
        {
            assert!(
                EXPANDED_CALL_TARGETS.contains(&name),
                "`{name}` must reach parser::expand"
            );
            assert!(
                !INTRINSIC_CALL_TARGETS.contains(&name),
                "`{name}` must not be refused here"
            );
            let doc = parse_document(&format!("CALL {name}(a : b)")).unwrap();
            let out = flatten_calls(doc.statements, &doc.defs).expect("passes through");
            assert!(
                matches!(out.as_slice(), [Statement::CallProc { name: n, .. }] if n == name),
                "`{name}` was rewritten instead of passed through: {out:?}"
            );
        }
    }

    /// The stage-2 allowance and the stage-3 dispatcher must name the same
    /// control-systems set. A name in `control::flatten::CALL_NAMES` but not
    /// here is refused before it ever reaches its flattener — the exact defect
    /// `mason` had.
    #[test]
    fn every_control_call_name_passes_this_stage() {
        for name in crate::control::flatten::CALL_NAMES {
            assert!(
                EXPANDED_CALL_TARGETS.contains(&name),
                "`{name}` is flattened by control::flatten but refused at stage 2"
            );
        }
        for name in EXPANDED_CALL_TARGETS {
            if crate::control::flatten::handles(name) {
                continue;
            }
            assert!(
                !INTRINSIC_CALL_TARGETS.contains(name),
                "`{name}` is in both stage-2 lists"
            );
        }
    }

    #[test]
    fn output_count_mismatches_are_named() {
        let doc = parse_document("PROCEDURE p(a : b, c)\n  b := a\n  c := a\nEND\nCALL p(1 : y)")
            .unwrap();
        let err = flatten_calls(doc.statements, &doc.defs).unwrap_err();
        assert!(
            err.to_string()
                .contains("CALL p provides 1 output variable(s) but PROCEDURE declares 2"),
            "{err}"
        );

        let doc = parse_document("MODULE m(x : y)\n  y = x\nEND\nCALL m(1, 2 : w)").unwrap();
        let err = flatten_calls(doc.statements, &doc.defs).unwrap_err();
        assert!(
            err.to_string()
                .contains("CALL m provides 2 input(s) but MODULE declares 1"),
            "{err}"
        );
    }

    #[test]
    fn a_non_variable_output_is_refused() {
        let doc = parse_document("PROCEDURE p(a : b)\n  b := a\nEND\nCALL p(1 : y + 1)").unwrap();
        let err = flatten_calls(doc.statements, &doc.defs).unwrap_err();
        assert!(
            err.to_string()
                .contains("CALL output argument must resolve to a variable"),
            "{err}"
        );
    }

    #[test]
    fn ignored_output_sinks_flatten_like_ordinary_variables() {
        let statements =
            flattened("FUNCTION [a, b] = Two(x)\n  a := x\n  b := x + 1\nEND\n[~, keep] = Two(4)");
        assert_eq!(statements.len(), 2);
        match &statements[0] {
            Statement::Eq(eq) => match &eq.lhs {
                Expr::Var(name) => {
                    assert!(crate::parser::toplevel::is_ignored_sink(name), "{name}")
                }
                other => panic!("{other:?}"),
            },
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn module_bodies_with_non_equation_statements_are_refused_not_dropped() {
        // DEVIATION from Java (which silently drops them): dropping equations
        // changes the degrees of freedom, so the port refuses.
        let doc = parse_document(
            "MODULE m(x : y)\n  FOR i = 1 TO 2\n    y = x\n  END\nEND\nCALL m(1 : w)",
        )
        .unwrap();
        let err = flatten_calls(doc.statements, &doc.defs).unwrap_err();
        assert!(
            err.to_string()
                .contains("MODULE m body contains a FOR block"),
            "{err}"
        );
    }
}
