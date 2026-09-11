//! Matrix/array expansion: turns matrix-valued statements into scalar
//! equations over flattened element variables (`a[1]`, `c[1,2]`).
//!
//! Port of the matrix half of `EquationParser.java` (the `flatten*` /
//! `compileMatrix*` machinery): matrix and vector literals, bare-name
//! references (`x = SolveLinear(A, b)`), the dense linear-algebra functions
//! (`Inverse`/`inv`, `Transpose`/postfix `'`, `det`/`Determinant`, `Dot`,
//! `norm`/`nrm2`, `asum`, `trace`, `MatrixNorm`/`FroNorm`, `cross`,
//! `SolveLinear`, backslash), the BLAS-style helpers (`axpy`, `scal`, `gemv`,
//! `gemm`, `ger`, `copy`), the generators (`zeros`, `ones`, `eye`/`identity`,
//! `diag`, `linspace`), `rangeAssign` arrays, element-wise operators, `FOR`
//! flattening with loop-variable substitution, and the CALL intrinsics
//! (`LUDecompose`, `Interp2`, and the Java `LIN_ALG_SIGNAL_STATS_CALLS` set:
//! `QR`, `Cholesky`, `MatExp`, `SingularValues`, `SVD`, `FFT`, `IFFT`,
//! `Convolve`, `LinFit`, `PolyFit`) with
//! `autoSizeCallOutputs`/`padOmittedOutputs`.
//!
//! # Architecture (non-negotiable, from the Java engine)
//!
//! frees expands matrix operations into **scalar equations over flattened
//! element variables at parse/expand time — the solver stays scalar.** Result
//! keys look like `x[1]`, `c[1,2]` (lowercase canonical; the display layer
//! restores the author's spelling of the base name).
//!
//! * `SolveLinear(A, b)` / `A \ b` / `Inverse(A)` do **not** run a numeric
//!   kernel: they emit the *defining equations* (`Σ A[i,k]·x[k] = b[i]`,
//!   `Σ A[i,k]·A⁻¹[k,j] = δ(i,j)`) and let Newton solve them.
//! * A matrix function used *inside* a larger expression materialises helper
//!   unknowns with the [`INTERNAL_TEMP_PREFIXES`] prefixes
//!   (`x = Inverse(A) * b` → `inverse_temp_N[i,j]`), which the result layer
//!   filters via [`is_internal_temp`] (Java `SolverApiSupport`).
//! * `det` of a matrix larger than 3×3 emits a runtime `det$<n>` call whose
//!   kernel lives in [`crate::linalg`] (the Java `Evaluator` ↔
//!   `LinearAlgebra` split).
//! * The dense linear-algebra / signal / statistics CALLs take that second
//!   route too: each output element is bound to a synthetic `$`-call
//!   (`qr$q$…`, `chol$l$…`, `expm$…`, `svd$u|smat|v|s$…`, `fft$re|im$…`,
//!   `ifft$…`, `conv$…`, `linfit$…`, `polyfit$…`, `interp2$…`) carrying the
//!   whole input row-major, which [`crate::eval`] routes into
//!   [`crate::linalg`] / [`crate::signal`] / [`crate::statistics`] /
//!   [`crate::interp2`].
//!
//! # The `range` intrinsic
//!
//! The Java builder materialises `speed = 0:10:100` into
//! `speed[1:N] = [v1, …]` at parse time; this port's [`crate::parser::toplevel`]
//! keeps it symbolic as `speed = range(start, middle, stop, '<spacing>')`.
//! [`expand_document`] performs the identical materialisation here, with the
//! same validation and value formulas (`| Log` = geometric spacing with an
//! exact endpoint).
//!
//! # Behaviour ported bug-for-bug
//!
//! * Bare-name resolution is **statement-order dependent**: a name resolves
//!   only after the statement that declared its shape has been flattened
//!   (plus `inferElementwiseShapes` pre-registration for element-by-element
//!   writes).
//! * Each `FOR` iteration flattens its body with a **fresh shape scope**
//!   (the Java `flatten` recursion allocates a new map), so top-level matrix
//!   names do not bare-resolve inside loop bodies.
//! * `resolveShapes` does not descend into array literals, comparisons or
//!   `not` — a matrix name inside `[a; b]` stays a scalar variable.
//! * Scalar call arguments that are slices flatten to multiple scalar
//!   arguments (`sqrt(A)` becomes `sqrt(a[1,1], a[1,2], …)`), exactly like
//!   `expandExpr`.
//! * `alpha`/`beta` arguments of the BLAS helpers are used *raw* (no
//!   loop-variable substitution), as in the Java code.
//!
//! Scalar documents pass through **byte-identical** to [`Document::equations`]
//! — the golden corpus freezes them; `tests/matrix_expansion.rs`
//! property-tests it.

// Numerical kernels index several parallel arrays (and 2-D `a[i][j]` slices)
// by the same loop variable, mirroring the Java/Fortran sources being
// transcribed. Iterator rewrites obscure that correspondence, so the indexed
// form stays.
#![allow(clippy::needless_range_loop)]

use std::collections::{BTreeMap, HashMap, HashSet};

use crate::ast::{BinOp, Equation, Expr, Statement};
use crate::control;
use crate::diag::{FreesError, Result};
use crate::eval::{self, EvalContext, Scope};
use crate::parser::defs::Definitions;
use crate::parser::toplevel::{IGNORED_OUTPUT_PREFIX, RANGE_INTRINSIC};
use crate::parser::Document;

/// Prefixes of the helper unknowns the matrix library materialises for a
/// matrix function used inside a larger expression (`x = Inverse(A) * b`).
/// They are real solver variables but an implementation detail, so the result
/// layer must filter them from the user-facing solution. Port of
/// `SolverApiSupport.INTERNAL_TEMP_PREFIXES`.
pub const INTERNAL_TEMP_PREFIXES: [&str; 3] =
    ["inverse_temp_", "backslash_temp_", "solvelinear_temp_"];

/// True for an indexed element of an internal matrix-library temporary, e.g.
/// `inverse_temp_12[1,2]`. Matches on the base name (before any `[`), so a
/// user variable such as `motor_temp_5` is never affected. Port of
/// `SolverApiSupport.isInternalTemp`.
pub fn is_internal_temp(name: &str) -> bool {
    let base = name.split('[').next().unwrap_or(name).to_ascii_lowercase();
    INTERNAL_TEMP_PREFIXES
        .iter()
        .any(|prefix| base.starts_with(prefix))
}

/// Largest span a single `FOR` loop, array range or generator may expand to.
/// Port of `EquationParser.MAX_RANGE_SPAN` (a denial-of-service guard).
const MAX_RANGE_SPAN: i128 = 1_000_000;

/// Backstop on the total number of equations a document may generate. Port of
/// `EquationParser.MAX_GENERATED_EQUATIONS` (default, no system-property
/// override in the wasm engine).
const MAX_GENERATED_EQUATIONS: usize = 25_000;

/// Maximum elements a `rangeAssign` may generate. Port of
/// `AstBuilder.MAX_RANGE_ELEMENTS`.
const MAX_RANGE_ELEMENTS: i64 = 100_000;

/// Largest matrix expanded by the closed-form cofactor determinant; larger
/// matrices emit a runtime `det$<n>` LU intrinsic instead. Port of
/// `EquationParser.DET_CLOSED_FORM_MAX`.
const DET_CLOSED_FORM_MAX: usize = 3;

const TOO_MANY_EQUATIONS: &str =
    "Too many equations generated (over 25000). Reduce loop or array sizes.";

/// Expand every matrix-valued construct in `doc` into scalar equations.
///
/// Documents with no matrix content pass through byte-identical to
/// [`Document::equations`] — the scalar pipeline's behaviour is frozen by the
/// golden corpus.
pub fn expand_document(doc: &Document) -> Result<Vec<Equation>> {
    expand_document_with(doc, 0, &mut BTreeMap::new())
}

/// [`expand_document`], threading the two pieces of state the per-iteration
/// MODULE instantiation needs: the instance count `procedures::flatten_calls`
/// finished at (the shared numbering the Java keeps in one `moduleCounter`),
/// and the document display-name map the instantiation registers namespaced
/// variables into (`putIfAbsent` semantics — an existing spelling wins).
pub fn expand_document_with(
    doc: &Document,
    module_counter_base: u32,
    display_names: &mut BTreeMap<String, String>,
) -> Result<Vec<Equation>> {
    let constants = extract_constants(&doc.statements);
    let symbolic = collect_symbolic(&doc.statements);
    let mut flattener = Flattener {
        constants: &constants,
        defs: &doc.defs,
        symbolic: &symbolic,
        out: Vec::new(),
        sinks: next_sink_index(&doc.statements),
        module_counter: module_counter_base,
        generated_names: display_names,
    };
    flattener.flatten(&doc.statements, &Scope::default())?;
    Ok(flattener.out)
}

fn parse_err(message: impl Into<String>) -> FreesError {
    FreesError::parse(message)
}

/// `Math.round` semantics: half-up (not half-away-from-zero), NaN → 0.
/// `libm` for wasm/native bit determinism, per the port convention.
fn java_round(v: f64) -> i64 {
    if v.is_nan() {
        0
    } else {
        libm::floor(v + 0.5) as i64
    }
}

fn kronecker_delta(a: usize, b: usize) -> f64 {
    if a == b {
        1.0
    } else {
        0.0
    }
}

/// Declared shape of a matrix/vector variable. Port of the `int[]` values in
/// `FlattenContext.shapes` — `dims` is present only for shapes registered by
/// `inferElementwiseShapes` (the 3-element form).
#[derive(Debug, Clone, Copy)]
struct Shape {
    rows: usize,
    cols: usize,
    dims: Option<usize>,
}

/// A compiled matrix of element expressions (row-major, rectangular, both
/// dimensions >= 1). The Rust `Expr[][]`.
type Matrix = Vec<Vec<Expr>>;

/// An explicit `A[r1:r2, c1:c2]` reference resolved to its element variables.
struct MatrixInfo {
    rows: usize,
    cols: usize,
    elements: Matrix,
}

/// An explicit `v[a:b]` reference resolved to its element variables.
struct VectorInfo {
    size: usize,
    elements: Vec<Expr>,
}

// ---------------------------------------------------------------------------
// Document-level pre-passes
// ---------------------------------------------------------------------------

/// Fixed-point constant extraction over the top-level statements. Port of
/// `EquationParser.extractConstants` / `tryExtractConstant`: an equation with
/// a variable on one side and an evaluable expression on the other pins that
/// variable, enabling constant-sized matrices (`n = 3` … `A = zeros(n, n)`).
/// (`crate::eval` is the engine's numeric AST evaluator — safe by
/// construction, no code execution.)
fn extract_constants(statements: &[Statement]) -> Scope {
    let mut constants = Scope::default();
    let mut progress = true;
    while progress {
        progress = false;
        for statement in statements {
            let Statement::Eq(eq) = statement else {
                continue;
            };
            let lhs_is_new = match &eq.lhs {
                Expr::Var(name) => !constants.contains_key(name),
                _ => false,
            };
            if lhs_is_new {
                if let (Expr::Var(name), Ok(value)) = (&eq.lhs, eval::eval(&eq.rhs, &constants)) {
                    constants.insert(name.clone(), value);
                    progress = true;
                }
            } else if let Expr::Var(name) = &eq.rhs {
                if !constants.contains_key(name) {
                    if let Ok(value) = eval::eval(&eq.lhs, &constants) {
                        constants.insert(name.clone(), value);
                        progress = true;
                    }
                }
            }
        }
    }
    constants
}

/// Recursively gathers all `SYMBOLIC`-declared names. Port of
/// `EquationParser.collectSymbolic`.
fn collect_symbolic(statements: &[Statement]) -> HashSet<String> {
    let mut names = HashSet::new();
    fn walk(statements: &[Statement], names: &mut HashSet<String>) {
        for statement in statements {
            match statement {
                Statement::Symbolic(declared) => {
                    names.extend(declared.iter().map(|n| n.to_ascii_lowercase()))
                }
                Statement::For { body, .. } => walk(body, names),
                _ => {}
            }
        }
    }
    walk(statements, &mut names);
    names
}

/// First free `~ignored~N` sink index, so sinks minted while padding omitted
/// CALL outputs never collide with the parser's own (per-document counter).
fn next_sink_index(statements: &[Statement]) -> u32 {
    let mut next = 0u32;
    fn walk(statements: &[Statement], next: &mut u32) {
        for statement in statements {
            match statement {
                Statement::CallProc { outputs, .. } => {
                    for output in outputs {
                        if let Expr::Var(name) = output {
                            if let Some(rest) = name.strip_prefix(IGNORED_OUTPUT_PREFIX) {
                                if let Ok(index) = rest.parse::<u32>() {
                                    *next = (*next).max(index + 1);
                                }
                            }
                        }
                    }
                }
                Statement::For { body, .. } => walk(body, next),
                _ => {}
            }
        }
    }
    walk(statements, &mut next);
    next
}

// ---------------------------------------------------------------------------
// The flattener
// ---------------------------------------------------------------------------

struct Flattener<'a> {
    constants: &'a Scope,
    defs: &'a Definitions,
    symbolic: &'a HashSet<String>,
    out: Vec<Equation>,
    /// Next `~ignored~N` id for padded CALL outputs.
    sinks: u32,
    /// MODULE instance numbering, continued from where
    /// `procedures::flatten_calls` stopped — the Java keeps one
    /// `moduleCounter` across parse-time flattening, and this is its second
    /// half (the per-iteration instantiations inside unrolled FORs).
    module_counter: u32,
    /// The document display-name map (`putIfAbsent` semantics), which
    /// [`Flattener::flatten_module_call`] registers namespaced names into.
    generated_names: &'a mut BTreeMap<String, String>,
}

impl Flattener<'_> {
    /// Append an equation, enforcing the generation budget on every emission
    /// site (the Java `BoundedEquationList`).
    fn push(&mut self, equation: Equation) -> Result<()> {
        if self.out.len() >= MAX_GENERATED_EQUATIONS {
            return Err(parse_err(TOO_MANY_EQUATIONS));
        }
        self.out.push(equation);
        Ok(())
    }

    /// Same budget as [`Flattener::push`], asserted *before* a batch is built —
    /// the port of `BoundedEquationList.addAll`, which makes exactly this
    /// `size() + more.size() > MAX` check.
    ///
    /// The kernel flatteners copy the whole input matrix into every equation
    /// they emit, so an oversized request costs O(equations × entries) memory
    /// before `push` would ever reject it. The Java holds one shared argument
    /// list and can afford to discover the limit on insertion; this port cannot,
    /// so it refuses up front with the identical message.
    ///
    /// Callers compute `planned` with saturating arithmetic: `usize` is 32 bits
    /// on wasm32, and a 10^6 × 1 slice (the `MAX_RANGE_SPAN` ceiling) squares
    /// past `u32::MAX`.
    fn reserve(&self, planned: usize) -> Result<()> {
        if self.out.len() + planned > MAX_GENERATED_EQUATIONS {
            return Err(parse_err(TOO_MANY_EQUATIONS));
        }
        Ok(())
    }

    fn new_sink(&mut self) -> Expr {
        let sink = Expr::Var(format!("{IGNORED_OUTPUT_PREFIX}{}", self.sinks));
        self.sinks += 1;
        sink
    }

    /// Port of `EquationParser.flatten`: each invocation opens a fresh shape
    /// scope (which is why bare names registered at the top level do not
    /// resolve inside FOR bodies — a Java behaviour kept as-is).
    fn flatten(&mut self, statements: &[Statement], loop_vars: &Scope) -> Result<()> {
        let mut shapes: HashMap<String, Shape> = HashMap::new();
        self.infer_elementwise_shapes(statements, loop_vars, &mut shapes);
        for statement in statements {
            match statement {
                Statement::For {
                    var_name,
                    start,
                    end,
                    body,
                } => self.flatten_for(var_name, start, end, body, loop_vars)?,
                Statement::Eq(eq) => {
                    self.flatten_eq(&eq.lhs, &eq.rhs, &eq.source_text, loop_vars, &mut shapes)?
                }
                Statement::CallProc {
                    name,
                    inputs,
                    outputs,
                    source_text,
                } => self.flatten_call_proc(
                    name,
                    inputs,
                    outputs,
                    source_text,
                    loop_vars,
                    &mut shapes,
                )?,
                Statement::Symbolic(_) => {
                    // Declaration only; the names were pre-collected.
                }
            }
        }
        Ok(())
    }

    /// Pre-registers the shape of matrices declared element-by-element
    /// (`A[1,1] = …; A[2,2] = …`). Port of
    /// `EquationParser.inferElementwiseShapes`, including the scalar-name
    /// exclusion (case-insensitive `k` vs `K[i,j]`) and the skip of range
    /// slices and non-constant indices.
    fn infer_elementwise_shapes(
        &self,
        statements: &[Statement],
        loop_vars: &Scope,
        shapes: &mut HashMap<String, Shape>,
    ) {
        let mut scalar_names: HashSet<String> = HashSet::new();
        for statement in statements {
            if let Statement::Eq(eq) = statement {
                if let Expr::Var(name) = &eq.lhs {
                    scalar_names.insert(name.to_ascii_lowercase());
                }
            }
        }
        // name -> (max row, max col, dims)
        let mut max_idx: HashMap<String, (i64, i64, usize)> = HashMap::new();
        for statement in statements {
            let Statement::Eq(eq) = statement else {
                continue;
            };
            let Expr::ArrayAccess { name, indices } = &eq.lhs else {
                continue;
            };
            if scalar_names.contains(&name.to_ascii_lowercase()) {
                continue; // ambiguous: a scalar of the same name exists
            }
            if indices.len() != 1 && indices.len() != 2 {
                continue;
            }
            if indices.iter().any(|ix| matches!(ix, Expr::Range { .. })) {
                continue; // a range slice declares its own shape
            }
            let Ok(row) = self.const_index(&indices[0], loop_vars) else {
                continue; // non-constant index (e.g. a loop variable)
            };
            let col = if indices.len() == 2 {
                match self.const_index(&indices[1], loop_vars) {
                    Ok(col) => col,
                    Err(_) => continue,
                }
            } else {
                1
            };
            let entry = max_idx
                .entry(name.to_ascii_lowercase())
                .or_insert((0, 0, indices.len()));
            entry.0 = entry.0.max(row);
            entry.1 = entry.1.max(col);
            entry.2 = entry.2.max(indices.len()); // promote to 2-D
        }
        for (name, (rows, cols, dims)) in max_idx {
            if shapes.contains_key(&name) {
                continue; // explicit shape wins
            }
            if rows <= 0 {
                continue;
            }
            let cols = if dims == 2 { cols } else { 1 };
            if cols <= 0 {
                continue;
            }
            shapes.insert(
                name,
                Shape {
                    rows: rows as usize,
                    cols: cols as usize,
                    dims: Some(dims),
                },
            );
        }
    }

    /// Port of `EquationParser.flattenFor`: bounds are compile-time constants;
    /// each iteration binds the loop variable and flattens the body.
    fn flatten_for(
        &mut self,
        var_name: &str,
        start: &Expr,
        end: &Expr,
        body: &[Statement],
        loop_vars: &Scope,
    ) -> Result<()> {
        let start_val = self.eval_index_expr(&self.expand_expr(start, loop_vars)?, loop_vars)?;
        let end_val = self.eval_index_expr(&self.expand_expr(end, loop_vars)?, loop_vars)?;
        let start_int = java_round(start_val);
        let end_int = java_round(end_val);
        let span = (end_int as i128 - start_int as i128).abs() + 1;
        if span > MAX_RANGE_SPAN {
            return Err(parse_err(format!(
                "FOR loop range is too large ({span} iterations; limit {MAX_RANGE_SPAN}). \
                 Reduce the loop bounds."
            )));
        }
        let step: i64 = if start_int <= end_int { 1 } else { -1 };
        let var_name = var_name.to_ascii_lowercase();
        let mut i = start_int;
        loop {
            let done = if start_int <= end_int {
                i > end_int
            } else {
                i < end_int
            };
            if done {
                break;
            }
            let mut new_loop_vars = loop_vars.clone();
            new_loop_vars.insert(var_name.clone(), i as f64);
            self.flatten(body, &new_loop_vars)?;
            i += step;
        }
        Ok(())
    }

    /// Port of `EquationParser.flattenEq`, plus the Rust-specific
    /// materialisation of the `range` intrinsic (see the module docs).
    fn flatten_eq(
        &mut self,
        lhs: &Expr,
        rhs: &Expr,
        source: &str,
        loop_vars: &Scope,
        shapes: &mut HashMap<String, Shape>,
    ) -> Result<()> {
        if let Some((range_lhs, range_rhs)) = desugar_range_intrinsic(lhs, rhs)? {
            return self.flatten_eq(&range_lhs, &range_rhs, source, loop_vars, shapes);
        }

        // An equation that involves a SYMBOLIC variable is a CAS identity:
        // solve it for the remaining coefficients rather than treating the
        // symbolic variable as a numeric unknown.
        if let Some(variable) = self.identity_variable(lhs, rhs)? {
            return self.flatten_identity(lhs, rhs, &variable, source);
        }

        // Rewrite bare references to known matrix/vector variables (e.g. the
        // A, b in SolveLinear(A, b)) into their explicit A[1:r,1:c] form.
        let lhs = resolve_shapes(lhs, shapes);
        let rhs = resolve_shapes(rhs, shapes);

        // Dedicated matrix-function handlers run first: they write the output
        // directly (no helper temp variable leaks into the solution).
        if self.try_flatten_matrix_function(&lhs, &rhs, source, loop_vars, shapes)? {
            return Ok(());
        }

        // Bare creation: A = [1 2; 3 4], v = [1 2 3], Z = zeros(2,2).
        if let Expr::Var(name) = &lhs {
            if matches!(rhs, Expr::ArrayLiteral(_))
                || is_matrix_expr(&rhs)
                || contains_elementwise_op(&rhs)
            {
                return self.flatten_bare_matrix_creation(name, &rhs, source, loop_vars, shapes);
            }
        }

        if is_matrix_expr(&lhs)
            || is_matrix_expr(&rhs)
            || contains_elementwise_op(&lhs)
            || contains_elementwise_op(&rhs)
        {
            return self.flatten_matrix_assignment(&lhs, &rhs, source, loop_vars, shapes);
        }

        let expanded_lhs = self.expand_expr(&lhs, loop_vars)?;
        let expanded_rhs = self.expand_expr(&rhs, loop_vars)?;
        self.push(Equation::new(expanded_lhs, expanded_rhs, source))
    }

    /// The single `SYMBOLIC` variable this equation involves, or `None` when
    /// it involves none. Port of `EquationParser.identityVariable`, including
    /// its refusal of more than one — an identity is solved with respect to
    /// one independent variable.
    fn identity_variable(&self, lhs: &Expr, rhs: &Expr) -> Result<Option<String>> {
        if self.symbolic.is_empty() {
            return Ok(None);
        }
        let mut present: Vec<String> = lhs
            .variables()
            .into_iter()
            .chain(rhs.variables())
            .filter(|name| self.symbolic.contains(name))
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        if present.is_empty() {
            return Ok(None);
        }
        if present.len() > 1 {
            return Err(parse_err(format!(
                "An identity may involve only one SYMBOLIC variable, but found: [{}]",
                present.join(", ")
            )));
        }
        Ok(Some(present.remove(0)))
    }

    /// Solve a CAS identity — an equation that must hold for *all* values of
    /// the symbolic variable — for its coefficients, and emit each as a
    /// concrete `name = value` equation so the ordinary solver reports it.
    /// Port of `EquationParser.flattenIdentity`.
    ///
    /// The Java runs `TransferFunction.expandCalls` over both sides first, so
    /// an identity may be written `tf([1,3],[1,3,2]) = A/(s+1) + B/(s+2)`.
    /// That expander belongs to the control suite, not the CAS, which is why
    /// `cas::engine::solve_coefficients_with` takes it as a hook; running it
    /// here instead keeps `FreesError` propagation intact (routing it through
    /// the hook would wrap a parse error inside a `CasError` and print the
    /// prefix twice). A `CasException` becomes a `ParseException`, so the
    /// message reaches the user verbatim.
    fn flatten_identity(
        &mut self,
        lhs: &Expr,
        rhs: &Expr,
        variable: &str,
        source: &str,
    ) -> Result<()> {
        let lhs = crate::control::tf::expand_calls(lhs, variable)?;
        let rhs = crate::control::tf::expand_calls(rhs, variable)?;
        let coefficients = crate::cas::engine::solve_coefficients(&lhs, &rhs, variable)
            .map_err(|e| parse_err(e.to_string()))?;
        for (name, value) in coefficients {
            self.push(Equation::new(Expr::Var(name), Expr::num(value), source))?;
        }
        Ok(())
    }

    /// Routes `lhs = f(args)` to a dedicated matrix-function handler when `f`
    /// is one. Port of `EquationParser.tryFlattenMatrixFunction`.
    fn try_flatten_matrix_function(
        &mut self,
        lhs: &Expr,
        rhs: &Expr,
        source: &str,
        loop_vars: &Scope,
        shapes: &mut HashMap<String, Shape>,
    ) -> Result<bool> {
        let Expr::Call { function, args } = rhs else {
            return Ok(false);
        };
        let func = match function.as_str() {
            "inv" => "inverse", // array-language aliases
            "det" => "determinant",
            other => other,
        };
        match func {
            "inverse" | "transpose" => {
                let first = arg_at(args, 0, func)?;
                self.flatten_matrix_transform(func, lhs, first, source, loop_vars, shapes)?;
                Ok(true)
            }
            "dot" | "norm" | "nrm2" | "determinant" | "asum" | "trace" | "matrixnorm"
            | "fronorm" => {
                self.flatten_vector_or_det(func, lhs, args, source, loop_vars)?;
                Ok(true)
            }
            "cross" => {
                self.flatten_cross_product(lhs, args, source, loop_vars, shapes)?;
                Ok(true)
            }
            "solvelinear" => {
                self.flatten_solve_linear(lhs, args, source, loop_vars, shapes)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    /// Compiles a general matrix/elementwise assignment into per-element
    /// equations. Port of `EquationParser.flattenMatrixAssignment`.
    fn flatten_matrix_assignment(
        &mut self,
        lhs: &Expr,
        rhs: &Expr,
        source: &str,
        loop_vars: &Scope,
        shapes: &mut HashMap<String, Shape>,
    ) -> Result<()> {
        let lhs_mat = self.compile_matrix_expr(lhs, loop_vars, shapes)?;
        let rhs_mat = self.compile_matrix_expr(rhs, loop_vars, shapes)?;
        // Remember an explicitly-dimensioned LHS (A[1:r,1:c] = ...) so a later
        // bare reference to it resolves.
        if let Expr::ArrayAccess { name, indices } = lhs {
            if !indices.is_empty() {
                register_shape(shapes, name, lhs_mat.len(), lhs_mat[0].len(), None);
            }
        }
        let rhs_mat = conform_rhs_to_lhs(&lhs_mat, rhs_mat)?;
        for (lhs_row, rhs_row) in lhs_mat.iter().zip(&rhs_mat) {
            for (l, r) in lhs_row.iter().zip(rhs_row) {
                self.push(Equation::new(l.clone(), r.clone(), source))?;
            }
        }
        Ok(())
    }

    /// Bare creation `A = [1 2; 3 4]`, `v = [1, 2, 3]` or `v = [1; 2; 3]`:
    /// emits element equations (`a[i,j]` or `v[k]`) and registers the shape.
    /// Port of `EquationParser.flattenBareMatrixCreation`.
    fn flatten_bare_matrix_creation(
        &mut self,
        name: &str,
        rhs: &Expr,
        source: &str,
        loop_vars: &Scope,
        shapes: &mut HashMap<String, Shape>,
    ) -> Result<()> {
        let matrix = self.compile_matrix_expr(rhs, loop_vars, shapes)?;
        let rows = matrix.len();
        let cols = matrix[0].len();
        let vector = rows == 1 || cols == 1;
        for (i, row) in matrix.iter().enumerate() {
            for (j, element) in row.iter().enumerate() {
                let canonical = if vector {
                    format!("{name}[{}]", i.max(j) + 1)
                } else {
                    format!("{name}[{},{}]", i + 1, j + 1)
                };
                self.push(Equation::new(Expr::Var(canonical), element.clone(), source))?;
            }
        }
        register_shape(shapes, name, rows, cols, None);
        Ok(())
    }

    /// Direct `lhs = Inverse(A)` / `lhs = Transpose(A)` (also the postfix
    /// `'`). Port of `EquationParser.flattenMatrixTransform`.
    fn flatten_matrix_transform(
        &mut self,
        func: &str,
        lhs: &Expr,
        first_arg: &Expr,
        source: &str,
        loop_vars: &Scope,
        shapes: &mut HashMap<String, Shape>,
    ) -> Result<()> {
        let rhs_mat = self.parse_matrix_info(first_arg, loop_vars)?;
        // Allow a bare output name (C = Inverse(A)): size it from the operation.
        let (out_rows, out_cols) = if func == "transpose" {
            (rhs_mat.cols, rhs_mat.rows)
        } else {
            (rhs_mat.rows, rhs_mat.cols)
        };
        let lhs = explicit_matrix_output(lhs, out_rows, out_cols, shapes);
        let lhs_mat = self.parse_matrix_info(&lhs, loop_vars)?;
        if func == "transpose" {
            self.emit_transpose_equations(&lhs_mat, &rhs_mat, source)
        } else {
            self.emit_inverse_equations(&lhs_mat, &rhs_mat, source)
        }
    }

    fn emit_transpose_equations(
        &mut self,
        lhs_mat: &MatrixInfo,
        rhs_mat: &MatrixInfo,
        source: &str,
    ) -> Result<()> {
        if lhs_mat.rows != rhs_mat.cols || lhs_mat.cols != rhs_mat.rows {
            return Err(parse_err(format!(
                "Dimension mismatch for Transpose: LHS is {}x{}, RHS is {}x{}",
                lhs_mat.rows, lhs_mat.cols, rhs_mat.cols, rhs_mat.rows
            )));
        }
        for i in 0..lhs_mat.rows {
            for j in 0..lhs_mat.cols {
                self.push(Equation::new(
                    lhs_mat.elements[i][j].clone(),
                    rhs_mat.elements[j][i].clone(),
                    source,
                ))?;
            }
        }
        Ok(())
    }

    /// Emits the defining equations of an inverse: `(RHS · LHS)[i,j] = δ(i,j)`.
    fn emit_inverse_equations(
        &mut self,
        lhs_mat: &MatrixInfo,
        rhs_mat: &MatrixInfo,
        source: &str,
    ) -> Result<()> {
        if lhs_mat.rows != lhs_mat.cols
            || rhs_mat.rows != rhs_mat.cols
            || lhs_mat.rows != rhs_mat.rows
        {
            return Err(parse_err(
                "Inverse requires square matrices of identical size.",
            ));
        }
        let n = lhs_mat.rows;
        for i in 0..n {
            for j in 0..n {
                let mut sum: Option<Expr> = None;
                for k in 0..n {
                    let term = Expr::bin(
                        BinOp::Mul,
                        rhs_mat.elements[i][k].clone(),
                        lhs_mat.elements[k][j].clone(),
                    );
                    sum = Some(match sum {
                        None => term,
                        Some(acc) => Expr::bin(BinOp::Add, acc, term),
                    });
                }
                let sum = sum.expect("n >= 1");
                self.push(Equation::new(sum, Expr::num(kronecker_delta(i, j)), source))?;
            }
        }
        Ok(())
    }

    /// `lhs = dot(...)/norm(...)/det(...)/trace(...)/…` — scalar-valued matrix
    /// functions. Port of `EquationParser.flattenVectorOrDet`.
    fn flatten_vector_or_det(
        &mut self,
        func: &str,
        lhs: &Expr,
        args: &[Expr],
        source: &str,
        loop_vars: &Scope,
    ) -> Result<()> {
        let expanded_lhs = self.expand_expr(lhs, loop_vars)?;
        let rhs = match func {
            "dot" => self.dot_product(args, loop_vars)?,
            "norm" | "nrm2" => self.vector_norm(args, loop_vars)?,
            "asum" => self.vector_abs_sum(args, loop_vars)?,
            "determinant" => self.matrix_determinant(args, loop_vars)?,
            "trace" => self.matrix_trace(args, loop_vars)?,
            "matrixnorm" | "fronorm" => self.matrix_frobenius_norm(args, loop_vars)?,
            other => {
                return Err(parse_err(format!(
                    "Unsupported vector/scalar function: {other}"
                )))
            }
        };
        self.push(Equation::new(expanded_lhs, rhs, source))
    }

    fn dot_product(&self, args: &[Expr], loop_vars: &Scope) -> Result<Expr> {
        let u = self.parse_vector_info(arg_at(args, 0, "dot")?, loop_vars)?;
        let v = self.parse_vector_info(arg_at(args, 1, "dot")?, loop_vars)?;
        if u.size != v.size {
            return Err(parse_err("Dot product requires vectors of identical size."));
        }
        let mut sum: Option<Expr> = None;
        for i in 0..u.size {
            let term = Expr::bin(BinOp::Mul, u.elements[i].clone(), v.elements[i].clone());
            sum = Some(match sum {
                None => term,
                Some(acc) => Expr::bin(BinOp::Add, acc, term),
            });
        }
        Ok(sum.expect("vectors are non-empty"))
    }

    fn vector_norm(&self, args: &[Expr], loop_vars: &Scope) -> Result<Expr> {
        let v = self.parse_vector_info(arg_at(args, 0, "norm")?, loop_vars)?;
        let mut sum_sq: Option<Expr> = None;
        for element in &v.elements {
            let term = Expr::bin(BinOp::Mul, element.clone(), element.clone());
            sum_sq = Some(match sum_sq {
                None => term,
                Some(acc) => Expr::bin(BinOp::Add, acc, term),
            });
        }
        Ok(Expr::call("sqrt", vec![sum_sq.expect("non-empty")]))
    }

    fn vector_abs_sum(&self, args: &[Expr], loop_vars: &Scope) -> Result<Expr> {
        let v = self.parse_vector_info(arg_at(args, 0, "asum")?, loop_vars)?;
        let mut sum_abs: Option<Expr> = None;
        for element in &v.elements {
            let term = Expr::call("abs", vec![element.clone()]);
            sum_abs = Some(match sum_abs {
                None => term,
                Some(acc) => Expr::bin(BinOp::Add, acc, term),
            });
        }
        Ok(sum_abs.expect("non-empty"))
    }

    /// Closed-form cofactor expansion for n <= 3; a runtime `det$<n>` LU
    /// intrinsic beyond that (numeric, evaluated by `crate::linalg`).
    fn matrix_determinant(&self, args: &[Expr], loop_vars: &Scope) -> Result<Expr> {
        let m = self.parse_matrix_info(arg_at(args, 0, "det")?, loop_vars)?;
        if m.rows != m.cols {
            return Err(parse_err("Determinant requires a square matrix."));
        }
        if m.rows <= DET_CLOSED_FORM_MAX {
            return Ok(expand_determinant(&m.elements));
        }
        let entries: Vec<Expr> = m.elements.iter().flatten().cloned().collect();
        Ok(Expr::Call {
            function: format!("det${}", m.rows),
            args: entries,
        })
    }

    /// Trace: sum of the diagonal entries of a square matrix.
    fn matrix_trace(&self, args: &[Expr], loop_vars: &Scope) -> Result<Expr> {
        let m = self.parse_matrix_info(arg_at(args, 0, "trace")?, loop_vars)?;
        if m.rows != m.cols {
            return Err(parse_err("Trace requires a square matrix."));
        }
        let mut sum: Option<Expr> = None;
        for i in 0..m.rows {
            let diag = m.elements[i][i].clone();
            sum = Some(match sum {
                None => diag,
                Some(acc) => Expr::bin(BinOp::Add, acc, diag),
            });
        }
        Ok(sum.expect("non-empty"))
    }

    /// Frobenius norm: sqrt of the sum of squares of all entries.
    fn matrix_frobenius_norm(&self, args: &[Expr], loop_vars: &Scope) -> Result<Expr> {
        let m = self.parse_matrix_info(arg_at(args, 0, "matrixnorm")?, loop_vars)?;
        let mut sum_sq: Option<Expr> = None;
        for row in &m.elements {
            for element in row {
                let term = Expr::bin(BinOp::Mul, element.clone(), element.clone());
                sum_sq = Some(match sum_sq {
                    None => term,
                    Some(acc) => Expr::bin(BinOp::Add, acc, term),
                });
            }
        }
        Ok(Expr::call("sqrt", vec![sum_sq.expect("non-empty")]))
    }

    /// Port of `EquationParser.flattenCrossProduct`.
    fn flatten_cross_product(
        &mut self,
        lhs: &Expr,
        args: &[Expr],
        source: &str,
        loop_vars: &Scope,
        shapes: &mut HashMap<String, Shape>,
    ) -> Result<()> {
        let lhs = explicit_vector_output(lhs, 3, shapes); // allow w = cross(u, v)
        let w = self.parse_vector_info(&lhs, loop_vars)?;
        let u = self.parse_vector_info(arg_at(args, 0, "cross")?, loop_vars)?;
        let v = self.parse_vector_info(arg_at(args, 1, "cross")?, loop_vars)?;
        if w.size != 3 || u.size != 3 || v.size != 3 {
            return Err(parse_err(
                "Cross product is only defined for 3-dimensional vectors.",
            ));
        }
        let component = |a: usize, b: usize| {
            Expr::bin(
                BinOp::Sub,
                Expr::bin(BinOp::Mul, u.elements[a].clone(), v.elements[b].clone()),
                Expr::bin(BinOp::Mul, u.elements[b].clone(), v.elements[a].clone()),
            )
        };
        let w1 = component(1, 2);
        let w2 = component(2, 0);
        let w3 = component(0, 1);
        self.push(Equation::new(w.elements[0].clone(), w1, source))?;
        self.push(Equation::new(w.elements[1].clone(), w2, source))?;
        self.push(Equation::new(w.elements[2].clone(), w3, source))
    }

    /// Direct `x = SolveLinear(A, b)`: emits the `A·x = b` row equations with
    /// `x` written straight to the output (no temp). Port of
    /// `EquationParser.flattenSolveLinear`.
    fn flatten_solve_linear(
        &mut self,
        lhs: &Expr,
        args: &[Expr],
        source: &str,
        loop_vars: &Scope,
        shapes: &mut HashMap<String, Shape>,
    ) -> Result<()> {
        let a = self.parse_matrix_info(arg_at(args, 0, "solvelinear")?, loop_vars)?;
        let b = self.parse_vector_info(arg_at(args, 1, "solvelinear")?, loop_vars)?;
        // Allow a bare output name (x = SolveLinear(A, b)): size it from b.
        let lhs = explicit_vector_output(lhs, b.size, shapes);
        let x = self.parse_vector_info(&lhs, loop_vars)?;
        if a.rows != a.cols || a.rows != x.size || b.size != x.size {
            return Err(parse_err(
                "SolveLinear requires square matrix A and vectors x, b of compatible size.",
            ));
        }
        let n = x.size;
        for i in 0..n {
            let mut sum: Option<Expr> = None;
            for j in 0..n {
                let term = Expr::bin(BinOp::Mul, a.elements[i][j].clone(), x.elements[j].clone());
                sum = Some(match sum {
                    None => term,
                    Some(acc) => Expr::bin(BinOp::Add, acc, term),
                });
            }
            self.push(Equation::new(
                sum.expect("n >= 1"),
                b.elements[i].clone(),
                source,
            ))?;
        }
        Ok(())
    }

    // ── CALL statements ─────────────────────────────────────────────────────

    /// Port of the matrix slice of `EquationParser.flattenCallProc`:
    /// shape-resolves the arguments, pads omitted trailing outputs with sink
    /// variables, auto-sizes bare output names, then dispatches. In scope here
    /// are `LUDecompose`, the Java `LIN_ALG_SIGNAL_STATS_CALLS` set and — via
    /// the [`ControlHost`] adapter, exactly as the Java reaches
    /// `ControlSystemsFlattener` through its `csFlattener` back-reference —
    /// the whole control-systems suite. The eigen/Euler decompositions are
    /// still refused by name, and user PROCEDURE/MODULE calls are expected to
    /// have been flattened upstream (`procedures::flatten_calls`).
    fn flatten_call_proc(
        &mut self,
        name: &str,
        inputs: &[Expr],
        outputs: &[Expr],
        source: &str,
        loop_vars: &Scope,
        shapes: &mut HashMap<String, Shape>,
    ) -> Result<()> {
        let def_name = name.to_ascii_lowercase();
        let inputs: Vec<Expr> = inputs.iter().map(|e| resolve_shapes(e, shapes)).collect();
        let mut outputs: Vec<Expr> = outputs.iter().map(|e| resolve_shapes(e, shapes)).collect();

        self.pad_omitted_outputs(&def_name, &inputs, &mut outputs);
        // Sizing failures defer to the flattener's own (more specific) error.
        let _ = self.auto_size_call_outputs(&def_name, &inputs, &mut outputs, loop_vars);

        match def_name.as_str() {
            "ludecompose" => self.flatten_lu_decompose(&inputs, &outputs, source, loop_vars),
            "interp2" => self.flatten_interp2(&inputs, &outputs, source, loop_vars),
            "qr" => self.flatten_qr(&inputs, &outputs, source, loop_vars),
            "cholesky" => self.flatten_cholesky(&inputs, &outputs, source, loop_vars),
            "matexp" => self.flatten_mat_exp(&inputs, &outputs, source, loop_vars),
            "singularvalues" => self.flatten_singular_values(&inputs, &outputs, source, loop_vars),
            "svd" => self.flatten_svd(&inputs, &outputs, source, loop_vars),
            "eigenvalues" => self.flatten_eigen(false, &inputs, &outputs, source, loop_vars),
            "eigen" => self.flatten_eigen(true, &inputs, &outputs, source, loop_vars),
            "fft" => self.flatten_fft(false, &inputs, &outputs, source, loop_vars),
            "ifft" => self.flatten_fft(true, &inputs, &outputs, source, loop_vars),
            "convolve" => self.flatten_convolve(&inputs, &outputs, source, loop_vars),
            // Phase 4.5 sensor kernels.
            "detrend" => self.flatten_detrend(&inputs, &outputs, source, loop_vars),
            "smooth" => self.flatten_smooth(&inputs, &outputs, source, loop_vars),
            "window" => self.flatten_window(&inputs, &outputs, source, loop_vars),
            "filter" => self.flatten_filter(false, &inputs, &outputs, source, loop_vars),
            "filtfilt" => self.flatten_filter(true, &inputs, &outputs, source, loop_vars),
            "xcorr" => self.flatten_xcorr(&inputs, &outputs, source, loop_vars),
            "welch" => self.flatten_welch(&inputs, &outputs, source, loop_vars),
            "linfit" => self.flatten_lin_fit(&inputs, &outputs, source, loop_vars),
            "polyfit" => self.flatten_poly_fit(&inputs, &outputs, source, loop_vars),
            _ if control::flatten::handles(&def_name) => {
                let mut host = ControlHost {
                    flattener: self,
                    loop_vars,
                    shapes,
                };
                control::flatten::flatten(&mut host, &def_name, &inputs, &outputs, source)
            }
            _ if UNPORTED_CALL_INTRINSICS.contains(&def_name.as_str()) => Err(parse_err(format!(
                "`CALL {def_name}` is not supported by the wasm engine yet"
            ))),
            _ if self.defs.module(&def_name).is_some() => {
                // A MODULE call that rode through `procedures::flatten_calls`
                // inside a FOR body: Java flattens after unrolling, and this
                // is the after-unrolling position — `loop_vars` carries the
                // iteration binding, so each pass through the loop body
                // instantiates its own namespace.
                let def = self.defs.module(&def_name).expect("guard checked");
                self.flatten_module_call(def, &inputs, &outputs, loop_vars)
            }
            _ => {
                if self.defs.procedure(&def_name).is_some() {
                    Err(parse_err(format!(
                        "CALL {def_name}: PROCEDURE calls must be flattened before \
                         matrix expansion"
                    )))
                } else if self.defs.function(&def_name).is_some() {
                    Err(parse_err(format!(
                        "'{def_name}' is a FUNCTION, not callable with CALL \
                         (use it directly in an expression)"
                    )))
                } else {
                    Err(parse_err(format!(
                        "Unknown PROCEDURE or MODULE: '{def_name}'"
                    )))
                }
            }
        }
    }

    /// Pads a partial output list with hidden sink variables so
    /// array-language-style trailing omission works. Port of
    /// `EquationParser.padOmittedOutputs` / `expectedOutputCount`.
    fn pad_omitted_outputs(&mut self, def_name: &str, inputs: &[Expr], outputs: &mut Vec<Expr>) {
        if outputs.is_empty() {
            return; // nothing to destructure into
        }
        let expected = expected_output_count(def_name, inputs);
        while expected > 0 && outputs.len() < expected as usize {
            let sink = self.new_sink();
            outputs.push(sink);
        }
    }

    /// Expands bare-name CALL outputs into full `1..size` slices, sizing them
    /// from the inputs exactly as the flattener does, so `CALL QR(A : Q, R)`
    /// needs no restated lengths. Port of the `autoSizeCallOutputs` arms for
    /// the intrinsics in the matrix-expansion scope; the control-systems arms
    /// live in [`control::flatten::auto_size`] and are reached through the
    /// fall-through below.
    fn auto_size_call_outputs(
        &self,
        def_name: &str,
        inputs: &[Expr],
        outputs: &mut [Expr],
        loop_vars: &Scope,
    ) -> Result<()> {
        if !outputs.iter().any(|o| matches!(o, Expr::Var(_))) {
            return Ok(());
        }
        // Each arm mirrors the Java statement order: a size is read, the slot it
        // sizes is written, and only then is the next size read. A failure part
        // way through therefore leaves the earlier slots already sized, exactly
        // as the Java `catch (ParseException ignored)` around the switch does.
        match def_name {
            "ludecompose" => {
                let n = self.in_mat_rows(inputs, 0, loop_vars)?;
                set_mat(outputs, 0, n, n);
                set_mat(outputs, 1, n, n);
            }
            "qr" => {
                let m = self.in_mat_rows(inputs, 0, loop_vars)?;
                set_mat(outputs, 0, m, m);
                set_mat(outputs, 1, m, self.in_mat_cols(inputs, 0, loop_vars)?);
            }
            "cholesky" | "matexp" => {
                let n = self.in_mat_rows(inputs, 0, loop_vars)?;
                set_mat(outputs, 0, n, n);
            }
            "singularvalues" => {
                let rows = self.in_mat_rows(inputs, 0, loop_vars)?;
                let cols = self.in_mat_cols(inputs, 0, loop_vars)?;
                set_vec(outputs, 0, rows.min(cols));
            }
            "eigenvalues" => {
                let n = self.in_mat_rows(inputs, 0, loop_vars)?;
                set_vec(outputs, 0, n);
                if outputs.len() == 2 {
                    set_vec(outputs, 1, n);
                }
            }
            "eigen" => {
                let n = self.in_mat_rows(inputs, 0, loop_vars)?;
                set_vec(outputs, 0, n);
                set_mat(outputs, 1, n, n);
            }
            "svd" => {
                let m = self.in_mat_rows(inputs, 0, loop_vars)?;
                let n = self.in_mat_cols(inputs, 0, loop_vars)?;
                let p = m.min(n);
                set_mat(outputs, 0, m, p);
                set_mat(outputs, 1, p, p);
                set_mat(outputs, 2, n, p);
            }
            "fft" | "ifft" => {
                let n = self.in_vec_len(inputs, 0, loop_vars)?;
                set_vec(outputs, 0, n);
                set_vec(outputs, 1, n);
            }
            "convolve" => {
                let len = self.in_vec_len(inputs, 0, loop_vars)?
                    + self.in_vec_len(inputs, 1, loop_vars)?;
                set_vec(outputs, 0, len - 1);
            }
            // Phase 4.5: same-length transforms of the leading series…
            "detrend" | "smooth" | "window" => {
                set_vec(outputs, 0, self.in_vec_len(inputs, 0, loop_vars)?);
            }
            // …the filters, whose series is the third input, after b and a…
            "filter" | "filtfilt" => {
                set_vec(outputs, 0, self.in_vec_len(inputs, 2, loop_vars)?);
            }
            "xcorr" => {
                let len = self.in_vec_len(inputs, 0, loop_vars)?
                    + self.in_vec_len(inputs, 1, loop_vars)?;
                set_vec(outputs, 0, len - 1);
            }
            // …and Welch, whose two outputs are the one-sided bin count.
            "welch" => {
                let nperseg = self.in_scalar_int(inputs, 2, loop_vars)?;
                let bins = usize::try_from(nperseg).map_or(0, |n| n / 2 + 1);
                set_vec(outputs, 0, bins);
                set_vec(outputs, 1, bins);
            }
            "polyfit" => {
                let degree = self.in_scalar_int(inputs, 2, loop_vars)?;
                // `setVec` no-ops on a non-positive size, so a degree that is
                // negative (or absurd enough to saturate) simply leaves the
                // output as written and the flattener reports the real error.
                let size = usize::try_from(degree.saturating_add(1)).unwrap_or(0);
                set_vec(outputs, 0, size);
            }
            // The control-systems half of the same Java switch.
            name if control::flatten::handles(name) => {
                let view = ControlShapes {
                    flattener: self,
                    loop_vars,
                };
                control::flatten::auto_size(&view, name, inputs, outputs)?;
            }
            // Scalar-output or value-declared outputs: leave as written.
            _ => {}
        }
        Ok(())
    }

    /// Rows of an input treated as a matrix (vector length for a 1-D slice,
    /// 1 for a scalar). Port of `EquationParser.inMatRows`.
    fn in_mat_rows(&self, inputs: &[Expr], idx: usize, loop_vars: &Scope) -> Result<usize> {
        let Some(expr) = inputs.get(idx) else {
            return Err(parse_err(format!("CALL input {} is missing", idx + 1)));
        };
        if let Expr::ArrayAccess { indices, .. } = expr {
            if indices.len() == 2 {
                return Ok(self.parse_matrix_info(expr, loop_vars)?.rows);
            }
            if indices.len() == 1 {
                return Ok(self.parse_vector_info(expr, loop_vars)?.size);
            }
        }
        Ok(1)
    }

    /// Columns of an input treated as a matrix (1 for a 1-D slice or a
    /// scalar). Port of `EquationParser.inMatCols`.
    fn in_mat_cols(&self, inputs: &[Expr], idx: usize, loop_vars: &Scope) -> Result<usize> {
        let Some(expr) = inputs.get(idx) else {
            return Err(parse_err(format!("CALL input {} is missing", idx + 1)));
        };
        if let Expr::ArrayAccess { indices, .. } = expr {
            if indices.len() == 2 {
                return Ok(self.parse_matrix_info(expr, loop_vars)?.cols);
            }
            if indices.len() == 1 {
                return Ok(1);
            }
        }
        Ok(1)
    }

    /// Length of an input read as a vector. Port of
    /// `EquationParser.inVecLen` (which lets `parseVectorInfo`'s own error
    /// escape for a non-vector input).
    fn in_vec_len(&self, inputs: &[Expr], idx: usize, loop_vars: &Scope) -> Result<usize> {
        let Some(expr) = inputs.get(idx) else {
            return Err(parse_err(format!("CALL input {} is missing", idx + 1)));
        };
        Ok(self.parse_vector_info(expr, loop_vars)?.size)
    }

    /// Compile-time integer input (e.g. a PolyFit degree). Port of
    /// `EquationParser.inScalarInt`.
    fn in_scalar_int(&self, inputs: &[Expr], idx: usize, loop_vars: &Scope) -> Result<i64> {
        let Some(expr) = inputs.get(idx) else {
            return Err(parse_err(format!("CALL input {} is missing", idx + 1)));
        };
        self.const_index(expr, loop_vars)
    }

    /// Port of `EquationParser.flattenInterp2`:
    /// `CALL Interp2(x, y, Z, xq, yq : zq)` becomes the single equation
    /// `zq = interp2$<m>$<n>(x…, y…, Z row-major…, xq, yq)`, which
    /// `eval::eval_synthetic` routes into [`crate::interp2::interpolate`].
    ///
    /// The argument packing order is load-bearing and matches the Java exactly:
    /// the `m` x-nodes, then the `n` y-nodes, then the `m*n` grid entries
    /// row-major, then the two query coordinates.
    fn flatten_interp2(
        &mut self,
        inputs: &[Expr],
        outputs: &[Expr],
        source: &str,
        loop_vars: &Scope,
    ) -> Result<()> {
        if inputs.len() != 5 || outputs.len() != 1 {
            return Err(parse_err(
                "Interp2 expects (x[1:m], y[1:n], Z[1:m,1:n], xq, yq : zq), \
                 e.g. CALL Interp2(x, y, Z, 1.5, 2.5 : zq)",
            ));
        }
        let x = self.parse_vector_info(&inputs[0], loop_vars)?;
        let y = self.parse_vector_info(&inputs[1], loop_vars)?;
        let z = self.parse_matrix_info(&inputs[2], loop_vars)?;
        let (m, n) = (x.size, y.size);
        if z.rows != m || z.cols != n {
            return Err(parse_err(format!(
                "Interp2 requires Z to be m x n ({m}x{n}) matching x and y."
            )));
        }
        let xq = self.expand_expr(&inputs[3], loop_vars)?;
        let yq = self.expand_expr(&inputs[4], loop_vars)?;
        let mut entries = Vec::with_capacity(m + n + m * n + 2);
        entries.extend(x.elements);
        entries.extend(y.elements);
        entries.extend(matrix_entries(&z));
        entries.push(xq);
        entries.push(yq);
        let out = self.expand_expr(&outputs[0], loop_vars)?;
        self.push(Equation::new(
            out,
            Expr::Call {
                function: format!("interp2${m}${n}"),
                args: entries,
            },
            source,
        ))
    }

    /// Port of `EquationParser.flattenLuDecompose`: `A = L·U` with the fixed
    /// triangular structure pinned by equations (`L` unit-lower, `U` upper) —
    /// no numeric kernel involved.
    fn flatten_lu_decompose(
        &mut self,
        inputs: &[Expr],
        outputs: &[Expr],
        source: &str,
        loop_vars: &Scope,
    ) -> Result<()> {
        if inputs.len() != 1 || outputs.len() != 2 {
            return Err(parse_err(
                "LUDecompose expects exactly 1 input matrix and 2 output matrices, \
                 e.g. CALL LUDecompose(A[1:3,1:3] : L[1:3,1:3], U[1:3,1:3])",
            ));
        }
        let a = self.parse_matrix_info(&inputs[0], loop_vars)?;
        let l = self.parse_matrix_info(&outputs[0], loop_vars)?;
        let u = self.parse_matrix_info(&outputs[1], loop_vars)?;
        // The Java check omits `u` vs `a`; added here because mismatched sizes
        // would otherwise panic instead of erroring (Java throws an
        // ArrayIndexOutOfBounds through the same user-visible failure).
        if a.rows != a.cols
            || l.rows != l.cols
            || u.rows != u.cols
            || a.rows != l.rows
            || a.rows != u.rows
        {
            return Err(parse_err(
                "LUDecompose requires all matrices to be square and of identical size.",
            ));
        }
        let n = a.rows;
        for i in 0..n {
            for j in 0..n {
                self.emit_lu_triangular_entry(&l, &u, i, j, source)?;
                let mut sum: Option<Expr> = None;
                for k in 0..n {
                    let term = Expr::bin(
                        BinOp::Mul,
                        l.elements[i][k].clone(),
                        u.elements[k][j].clone(),
                    );
                    sum = Some(match sum {
                        None => term,
                        Some(acc) => Expr::bin(BinOp::Add, acc, term),
                    });
                }
                self.push(Equation::new(
                    sum.expect("n >= 1"),
                    a.elements[i][j].clone(),
                    source,
                ))?;
            }
        }
        Ok(())
    }

    /// Pins the fixed triangular structure of an LU factorization.
    fn emit_lu_triangular_entry(
        &mut self,
        l: &MatrixInfo,
        u: &MatrixInfo,
        i: usize,
        j: usize,
        source: &str,
    ) -> Result<()> {
        if i < j {
            self.push(Equation::new(
                l.elements[i][j].clone(),
                Expr::num(0.0),
                source,
            ))?;
        } else if i == j {
            self.push(Equation::new(
                l.elements[i][j].clone(),
                Expr::num(1.0),
                source,
            ))?;
        }
        if i > j {
            self.push(Equation::new(
                u.elements[i][j].clone(),
                Expr::num(0.0),
                source,
            ))?;
        }
        Ok(())
    }

    // ── Dense linear algebra / signal / statistics CALLs ─────────────────────
    //
    // Port of the Java `LIN_ALG_SIGNAL_STATS_CALLS` half of `flattenCallProc`.
    // Unlike `SolveLinear`/`Inverse`/`LUDecompose` — which emit *defining*
    // equations for Newton to solve — these bind each output element to a
    // synthetic `$`-call that runs the numeric kernel at evaluation time
    // (`crate::linalg`, `crate::signal`, `crate::statistics`, dispatched by
    // `eval::eval_synthetic`). Every equation carries the whole input matrix or
    // vector pair in its argument list, exactly as the Java does, so the
    // dependency graph sees the real inputs and the solver orders the blocks
    // correctly.

    /// Port of `EquationParser.flattenQr`: `Q` is m×m, `R` is m×n.
    fn flatten_qr(
        &mut self,
        inputs: &[Expr],
        outputs: &[Expr],
        source: &str,
        loop_vars: &Scope,
    ) -> Result<()> {
        if inputs.len() != 1 || outputs.len() != 2 {
            return Err(parse_err(
                "QR expects 1 input matrix and 2 output matrices, \
                 e.g. CALL QR(A[1:3,1:3] : Q[1:3,1:3], R[1:3,1:3])",
            ));
        }
        let a = self.parse_matrix_info(&inputs[0], loop_vars)?;
        let q = self.parse_matrix_info(&outputs[0], loop_vars)?;
        let r = self.parse_matrix_info(&outputs[1], loop_vars)?;
        let (m, n) = (a.rows, a.cols);
        if q.rows != m || q.cols != m {
            return Err(parse_err(format!(
                "QR requires Q to be {m}x{m} (m x m for an m x n input)."
            )));
        }
        if r.rows != m || r.cols != n {
            return Err(parse_err(format!(
                "QR requires R to match the input shape ({m}x{n})."
            )));
        }
        self.reserve(m.saturating_mul(m).saturating_add(m.saturating_mul(n)))?;
        let entries = matrix_entries(&a);
        for i in 0..m {
            for j in 0..m {
                self.push(Equation::new(
                    q.elements[i][j].clone(),
                    Expr::Call {
                        function: format!("qr$q${i}${j}${m}${n}"),
                        args: entries.clone(),
                    },
                    source,
                ))?;
            }
        }
        for i in 0..m {
            for j in 0..n {
                self.push(Equation::new(
                    r.elements[i][j].clone(),
                    Expr::Call {
                        function: format!("qr$r${i}${j}${m}${n}"),
                        args: entries.clone(),
                    },
                    source,
                ))?;
            }
        }
        Ok(())
    }

    /// Port of `EquationParser.flattenCholesky`: the lower factor `L` of
    /// `A = L·Lᵀ`.
    fn flatten_cholesky(
        &mut self,
        inputs: &[Expr],
        outputs: &[Expr],
        source: &str,
        loop_vars: &Scope,
    ) -> Result<()> {
        if inputs.len() != 1 || outputs.len() != 1 {
            return Err(parse_err(
                "Cholesky expects 1 input matrix and 1 output matrix, \
                 e.g. CALL Cholesky(A[1:3,1:3] : L[1:3,1:3])",
            ));
        }
        let a = self.parse_matrix_info(&inputs[0], loop_vars)?;
        let l = self.parse_matrix_info(&outputs[0], loop_vars)?;
        if a.rows != a.cols || l.rows != a.rows || l.cols != a.cols {
            return Err(parse_err(
                "Cholesky requires square matrices of identical size.",
            ));
        }
        let n = a.rows;
        self.reserve(n.saturating_mul(n))?;
        let entries = matrix_entries(&a);
        for i in 0..n {
            for j in 0..n {
                self.push(Equation::new(
                    l.elements[i][j].clone(),
                    Expr::Call {
                        function: format!("chol$l${i}${j}${n}"),
                        args: entries.clone(),
                    },
                    source,
                ))?;
            }
        }
        Ok(())
    }

    /// Port of `EquationParser.flattenMatExp`: the matrix exponential `e^A`.
    fn flatten_mat_exp(
        &mut self,
        inputs: &[Expr],
        outputs: &[Expr],
        source: &str,
        loop_vars: &Scope,
    ) -> Result<()> {
        if inputs.len() != 1 || outputs.len() != 1 {
            return Err(parse_err(
                "MatExp expects 1 input matrix and 1 output matrix, \
                 e.g. CALL MatExp(A[1:2,1:2] : E[1:2,1:2])",
            ));
        }
        let a = self.parse_matrix_info(&inputs[0], loop_vars)?;
        let e = self.parse_matrix_info(&outputs[0], loop_vars)?;
        if a.rows != a.cols || e.rows != a.rows || e.cols != a.cols {
            return Err(parse_err(
                "MatExp requires square matrices of identical size.",
            ));
        }
        let n = a.rows;
        self.reserve(n.saturating_mul(n))?;
        let entries = matrix_entries(&a);
        for i in 0..n {
            for j in 0..n {
                self.push(Equation::new(
                    e.elements[i][j].clone(),
                    Expr::Call {
                        function: format!("expm${i}${j}${n}"),
                        args: entries.clone(),
                    },
                    source,
                ))?;
            }
        }
        Ok(())
    }

    /// Port of `EquationParser.flattenSingularValues`: the `min(m, n)`
    /// singular values, non-increasing.
    fn flatten_singular_values(
        &mut self,
        inputs: &[Expr],
        outputs: &[Expr],
        source: &str,
        loop_vars: &Scope,
    ) -> Result<()> {
        if inputs.len() != 1 || outputs.len() != 1 {
            return Err(parse_err(
                "SingularValues expects 1 input matrix and 1 output vector, \
                 e.g. CALL SingularValues(A[1:3,1:2] : s[1:2])",
            ));
        }
        let a = self.parse_matrix_info(&inputs[0], loop_vars)?;
        let s = self.parse_vector_info(&outputs[0], loop_vars)?;
        let (m, n) = (a.rows, a.cols);
        if s.size != m.min(n) {
            return Err(parse_err(format!(
                "SingularValues requires an output vector of length min(rows, cols) = {}.",
                m.min(n)
            )));
        }
        self.reserve(s.size)?;
        let entries = matrix_entries(&a);
        for k in 0..s.size {
            self.push(Equation::new(
                s.elements[k].clone(),
                Expr::Call {
                    function: format!("svd$s${k}${m}${n}"),
                    args: entries.clone(),
                },
                source,
            ))?;
        }
        Ok(())
    }

    /// Port of `EquationParser.flattenSvd`: the thin factorisation
    /// `A = U·S·Vᵀ` with `U` m×p, `S` p×p and `V` n×p for p = min(m, n).
    fn flatten_svd(
        &mut self,
        inputs: &[Expr],
        outputs: &[Expr],
        source: &str,
        loop_vars: &Scope,
    ) -> Result<()> {
        if inputs.len() != 1 || outputs.len() != 3 {
            return Err(parse_err(
                "SVD expects 1 input matrix and 3 output matrices, e.g. CALL SVD(A : U, S, V)",
            ));
        }
        let a = self.parse_matrix_info(&inputs[0], loop_vars)?;
        let u = self.parse_matrix_info(&outputs[0], loop_vars)?;
        let s = self.parse_matrix_info(&outputs[1], loop_vars)?;
        let v = self.parse_matrix_info(&outputs[2], loop_vars)?;
        let (m, n) = (a.rows, a.cols);
        let p = m.min(n);
        if u.rows != m || u.cols != p || s.rows != p || s.cols != p || v.rows != n || v.cols != p {
            return Err(parse_err(format!(
                "SVD of a {m}x{n} matrix requires outputs U ({m}x{p}), S ({p}x{p}), \
                 and V ({n}x{p})."
            )));
        }
        self.reserve(
            m.saturating_mul(p)
                .saturating_add(p.saturating_mul(p))
                .saturating_add(n.saturating_mul(p)),
        )?;
        let entries = matrix_entries(&a);
        for i in 0..m {
            for j in 0..p {
                self.push(Equation::new(
                    u.elements[i][j].clone(),
                    Expr::Call {
                        function: format!("svd$u${i}${j}${m}${n}"),
                        args: entries.clone(),
                    },
                    source,
                ))?;
            }
        }
        for i in 0..p {
            for j in 0..p {
                self.push(Equation::new(
                    s.elements[i][j].clone(),
                    Expr::Call {
                        function: format!("svd$smat${i}${j}${m}${n}"),
                        args: entries.clone(),
                    },
                    source,
                ))?;
            }
        }
        for i in 0..n {
            for j in 0..p {
                self.push(Equation::new(
                    v.elements[i][j].clone(),
                    Expr::Call {
                        function: format!("svd$v${i}${j}${m}${n}"),
                        args: entries.clone(),
                    },
                    source,
                ))?;
            }
        }
        Ok(())
    }

    /// Port of `EquationParser.flattenModuleCall`, at the Java's own pipeline
    /// position — *after* FOR unrolling, so `loop_vars` carries the iteration
    /// binding and every pass through a loop body gets its own namespace
    /// (`twice$1$…`, `twice$2$…`). Inputs and outputs go through
    /// [`Self::expand_expr`] exactly as the Java's `expandExpr`: an output
    /// written `r[i]` resolves to the element variable `r[1]` before the
    /// must-be-a-variable check, which is how an array cell receives a module
    /// output. Body equations are namespaced and pushed as-is; display names
    /// register with `putIfAbsent` semantics, all per the Java.
    ///
    /// Top-level MODULE calls never reach this: `procedures::flatten_calls`
    /// (stage 2) instantiates them and this counter continues its numbering —
    /// see `flatten_calls_counted` for the one recorded ordering divergence.
    fn flatten_module_call(
        &mut self,
        def: &crate::parser::defs::ModuleDef,
        inputs: &[Expr],
        outputs: &[Expr],
        loop_vars: &Scope,
    ) -> Result<()> {
        // Java increments before the arity checks; a failing CALL still
        // consumed an instance number.
        self.module_counter += 1;
        let ns = format!("{}${}$", def.name, self.module_counter);

        if inputs.len() != def.inputs.len() {
            return Err(parse_err(format!(
                "CALL {} provides {} input(s) but MODULE declares {}",
                def.name,
                inputs.len(),
                def.inputs.len()
            )));
        }
        if outputs.len() != def.outputs.len() {
            return Err(parse_err(format!(
                "CALL {} provides {} output variable(s) but MODULE declares {}",
                def.name,
                outputs.len(),
                def.outputs.len()
            )));
        }

        // Input binding equations: ns$param = inputExpr.
        for (param, input) in def.inputs.iter().zip(inputs) {
            let ns_param = format!("{ns}{param}");
            let input_expr = self.expand_expr(input, loop_vars)?;
            self.push(Equation::new(
                Expr::Var(ns_param.clone()),
                input_expr,
                format!("MODULE {} input {param}", def.name),
            ))?;
            self.generated_names
                .entry(ns_param.clone())
                .or_insert(ns_param);
        }

        // Module body equations with namespaced variables. Same deviation as
        // the stage-2 instantiation: Java silently drops non-Eq bodies, this
        // port refuses (dropping equations changes the degrees of freedom).
        for body_statement in &def.body {
            match body_statement {
                Statement::Eq(eq) => {
                    let lhs = crate::procedures::namespace_expr(&eq.lhs, &ns);
                    let rhs = crate::procedures::namespace_expr(&eq.rhs, &ns);
                    for var in lhs.variables().into_iter().chain(rhs.variables()) {
                        self.generated_names.entry(var.clone()).or_insert(var);
                    }
                    self.push(Equation::new(lhs, rhs, eq.source_text.clone()))?;
                }
                other => {
                    let kind = match other {
                        Statement::For { .. } => "a FOR block",
                        Statement::CallProc { .. } => "a CALL",
                        Statement::Symbolic(_) => "a SYMBOLIC declaration",
                        Statement::Eq(_) => unreachable!(),
                    };
                    return Err(parse_err(format!(
                        "MODULE {} body contains {kind}; only `=` equations can be \
                         grafted into the caller's system",
                        def.name
                    )));
                }
            }
        }

        // Output binding equations: outputVar = ns$outputParam.
        for (param, output) in def.outputs.iter().zip(outputs) {
            let output_expr = self.expand_expr(output, loop_vars)?;
            let Expr::Var(var_name) = &output_expr else {
                return Err(parse_err(format!(
                    "CALL output argument must resolve to a variable (in `CALL {}`)",
                    def.name
                )));
            };
            self.generated_names
                .entry(var_name.clone())
                .or_insert_with(|| var_name.clone());
            self.push(Equation::new(
                output_expr.clone(),
                Expr::Var(format!("{ns}{param}")),
                format!("MODULE {} output {param}", def.name),
            ))?;
        }
        Ok(())
    }

    /// Port of `EquationParser.flattenEigen` / `emitEigenvectors` (ledger item
    /// 34). `Eigenvalues(A : lambda)` is the real-spectrum form,
    /// `Eigenvalues(A : re, im)` carries a complex spectrum as its parts
    /// (mirroring how FFT carries complex data), and `Eigen(A : lambda, V)`
    /// adds the eigenvector matrix, eigenvectors as columns. The emitted
    /// equations carry the *symbolic* matrix entries in their argument list,
    /// so a matrix filled in by other equations orders the eigen block after
    /// them — fixture `eqsys-eigen-waits-for-matrix-entries-solved-elsewhere`
    /// is the witness.
    fn flatten_eigen(
        &mut self,
        want_vectors: bool,
        inputs: &[Expr],
        outputs: &[Expr],
        source: &str,
        loop_vars: &Scope,
    ) -> Result<()> {
        let complex_pair = !want_vectors && outputs.len() == 2;
        let count_ok = if want_vectors {
            outputs.len() == 2
        } else {
            outputs.len() == 1 || outputs.len() == 2
        };
        if inputs.len() != 1 || !count_ok {
            return Err(parse_err(if want_vectors {
                "Eigen expects 1 input matrix and 2 outputs (eigenvalue vector, \
                 eigenvector matrix), e.g. CALL Eigen(A[1:3,1:3] : lambda[1:3], V[1:3,1:3])"
            } else {
                "Eigenvalues expects 1 input matrix and 1 output vector (real spectra) \
                 or 2 output vectors (real and imaginary parts), e.g. \
                 CALL Eigenvalues(A[1:3,1:3] : lambda[1:3]) or \
                 CALL Eigenvalues(A[1:2,1:2] : re[1:2], im[1:2])"
            }));
        }
        let a = self.parse_matrix_info(&inputs[0], loop_vars)?;
        let lambda = self.parse_vector_info(&outputs[0], loop_vars)?;
        if a.rows != a.cols || lambda.size != a.rows {
            return Err(parse_err(
                "Eigenvalues requires a square matrix and an eigenvalue vector \
                 of matching size.",
            ));
        }
        let n = a.rows;
        self.reserve(
            n.saturating_add(if complex_pair { n } else { 0 })
                .saturating_add(if want_vectors { n.saturating_mul(n) } else { 0 }),
        )?;
        let entries = matrix_entries(&a);
        let real_prefix = if complex_pair {
            "eigen$re"
        } else {
            "eigen$val"
        };
        for k in 0..n {
            self.push(Equation::new(
                lambda.elements[k].clone(),
                Expr::Call {
                    function: format!("{real_prefix}${k}${n}"),
                    args: entries.clone(),
                },
                source,
            ))?;
        }
        if complex_pair {
            let imag = self.parse_vector_info(&outputs[1], loop_vars)?;
            if imag.size != n {
                return Err(parse_err(
                    "Eigenvalues requires the imaginary-part vector to match \
                     the matrix size.",
                ));
            }
            for k in 0..n {
                self.push(Equation::new(
                    imag.elements[k].clone(),
                    Expr::Call {
                        function: format!("eigen$im${k}${n}"),
                        args: entries.clone(),
                    },
                    source,
                ))?;
            }
        }
        if want_vectors {
            let v = self.parse_matrix_info(&outputs[1], loop_vars)?;
            if v.rows != n || v.cols != n {
                return Err(parse_err(
                    "Eigen requires an n x n eigenvector matrix (eigenvectors \
                     as columns).",
                ));
            }
            // Row-major over (component i, eigenpair k), exactly the Java's
            // `emitEigenvectors` loop order.
            for i in 0..n {
                for k in 0..n {
                    self.push(Equation::new(
                        v.elements[i][k].clone(),
                        Expr::Call {
                            function: format!("eigen$vec${i}${k}${n}"),
                            args: entries.clone(),
                        },
                        source,
                    ))?;
                }
            }
        }
        Ok(())
    }

    /// Port of `EquationParser.flattenFft`: the DFT (or its inverse) of the
    /// complex sequence carried as two equal-length real vectors. The four
    /// vectors — two in, two out — all have the same length.
    fn flatten_fft(
        &mut self,
        inverse: bool,
        inputs: &[Expr],
        outputs: &[Expr],
        source: &str,
        loop_vars: &Scope,
    ) -> Result<()> {
        let name = if inverse { "IFFT" } else { "FFT" };
        if inputs.len() != 2 || outputs.len() != 2 {
            return Err(parse_err(format!(
                "{name} expects 2 input vectors (real, imag) and 2 output vectors, \
                 e.g. CALL {name}(re[1:n], im[1:n] : outRe[1:n], outIm[1:n])"
            )));
        }
        let re = self.parse_vector_info(&inputs[0], loop_vars)?;
        let im = self.parse_vector_info(&inputs[1], loop_vars)?;
        let out_re = self.parse_vector_info(&outputs[0], loop_vars)?;
        let out_im = self.parse_vector_info(&outputs[1], loop_vars)?;
        let n = re.size;
        if im.size != n || out_re.size != n || out_im.size != n {
            return Err(parse_err(format!(
                "{name} requires all four vectors to have the same length."
            )));
        }
        self.reserve(n.saturating_mul(2))?;
        let mut entries = Vec::with_capacity(2 * n);
        entries.extend(re.elements);
        entries.extend(im.elements);
        let prefix = if inverse { "ifft" } else { "fft" };
        for k in 0..n {
            self.push(Equation::new(
                out_re.elements[k].clone(),
                Expr::Call {
                    function: format!("{prefix}$re${k}${n}"),
                    args: entries.clone(),
                },
                source,
            ))?;
            self.push(Equation::new(
                out_im.elements[k].clone(),
                Expr::Call {
                    function: format!("{prefix}$im${k}${n}"),
                    args: entries.clone(),
                },
                source,
            ))?;
        }
        Ok(())
    }

    // ── Phase 4.5 sensor kernels ────────────────────────────────────────────
    //
    // These have no Java counterpart — the Java `SignalProcessing` stops at the
    // DFT and the convolution. They follow the same flattening shape as
    // `flatten_fft`: one equation per output element, each carrying the whole
    // input series in its argument list, because a flattened document has no
    // vector values to pass.

    /// The string literal an option argument carries, lowercased.
    fn call_option(name: &str, inputs: &[Expr], idx: usize, allowed: &str) -> Result<String> {
        match inputs.get(idx) {
            Some(Expr::Str(text)) => Ok(text.to_ascii_lowercase()),
            _ => Err(parse_err(format!(
                "{name} option {} must be a quoted name, one of {allowed}.",
                idx + 1
            ))),
        }
    }

    /// Emit one equation per element of `out`, each calling
    /// `<prefix>$<k>$<suffix>` over the shared `entries`.
    fn emit_elementwise(
        &mut self,
        out: &VectorInfo,
        prefix: &str,
        suffix: &str,
        entries: &[Expr],
        source: &str,
    ) -> Result<()> {
        self.reserve(out.size)?;
        for k in 0..out.size {
            self.push(Equation::new(
                out.elements[k].clone(),
                Expr::Call {
                    function: format!("{prefix}${k}${suffix}"),
                    args: entries.to_vec(),
                },
                source,
            ))?;
        }
        Ok(())
    }

    /// `CALL Detrend(y : yd)` removes a least-squares straight line;
    /// `CALL Detrend(y, 'constant' : yd)` removes only the mean.
    fn flatten_detrend(
        &mut self,
        inputs: &[Expr],
        outputs: &[Expr],
        source: &str,
        loop_vars: &Scope,
    ) -> Result<()> {
        if !(1..=2).contains(&inputs.len()) || outputs.len() != 1 {
            return Err(parse_err(
                "Detrend expects 1 input vector, an optional 'linear'/'constant' mode, \
                 and 1 output vector, e.g. CALL Detrend(y[1:n] : yd[1:n])",
            ));
        }
        let mode = if inputs.len() == 2 {
            Self::call_option("Detrend", inputs, 1, "'linear' or 'constant'")?
        } else {
            "linear".to_string()
        };
        let linear = match mode.as_str() {
            "linear" => true,
            "constant" | "const" | "mean" => false,
            other => {
                return Err(parse_err(format!(
                    "Detrend mode must be 'linear' or 'constant', got '{other}'."
                )))
            }
        };
        let y = self.parse_vector_info(&inputs[0], loop_vars)?;
        let out = self.parse_vector_info(&outputs[0], loop_vars)?;
        if out.size != y.size {
            return Err(parse_err(
                "Detrend requires the output vector to match the input length.",
            ));
        }
        let n = y.size;
        let tag = if linear { "linear" } else { "const" };
        self.emit_elementwise(
            &out,
            &format!("detrend${tag}"),
            &n.to_string(),
            &y.elements,
            source,
        )
    }

    /// `CALL Smooth(x, k : y)` — centred moving average over an odd `k`.
    fn flatten_smooth(
        &mut self,
        inputs: &[Expr],
        outputs: &[Expr],
        source: &str,
        loop_vars: &Scope,
    ) -> Result<()> {
        if inputs.len() != 2 || outputs.len() != 1 {
            return Err(parse_err(
                "Smooth expects 1 input vector, an odd window length and 1 output \
                 vector, e.g. CALL Smooth(x[1:n], 5 : y[1:n])",
            ));
        }
        let x = self.parse_vector_info(&inputs[0], loop_vars)?;
        let width = self.in_scalar_int(inputs, 1, loop_vars)?;
        let out = self.parse_vector_info(&outputs[0], loop_vars)?;
        if out.size != x.size {
            return Err(parse_err(
                "Smooth requires the output vector to match the input length.",
            ));
        }
        if width < 1 || width % 2 == 0 {
            return Err(parse_err(format!(
                "Smooth requires an odd window length >= 1, got {width}."
            )));
        }
        if width as usize > x.size {
            return Err(parse_err(format!(
                "Smooth window ({width}) is longer than the series ({}).",
                x.size
            )));
        }
        let suffix = format!("{}", x.size);
        self.emit_elementwise(
            &out,
            &format!("smooth${width}"),
            &suffix,
            &x.elements,
            source,
        )
    }

    /// `CALL Window(x, 'hann' : y)` — the series tapered by a symmetric window.
    fn flatten_window(
        &mut self,
        inputs: &[Expr],
        outputs: &[Expr],
        source: &str,
        loop_vars: &Scope,
    ) -> Result<()> {
        if inputs.len() != 2 || outputs.len() != 1 {
            return Err(parse_err(
                "Window expects 1 input vector, a quoted window name and 1 output \
                 vector, e.g. CALL Window(x[1:n], 'hann' : y[1:n])",
            ));
        }
        let kind = Self::call_option(
            "Window",
            inputs,
            1,
            "'rect', 'hann', 'hamming', 'blackman' or 'bartlett'",
        )?;
        let Some(resolved) = crate::signal::Window::from_name(&kind) else {
            return Err(parse_err(format!(
                "Unknown window '{kind}'. Use 'rect', 'hann', 'hamming', \
                 'blackman' or 'bartlett'."
            )));
        };
        // Canonicalize the alias so the synthetic carries one spelling per kind.
        let tag = match resolved {
            crate::signal::Window::Rect => "rect",
            crate::signal::Window::Hann => "hann",
            crate::signal::Window::Hamming => "hamming",
            crate::signal::Window::Blackman => "blackman",
            crate::signal::Window::Bartlett => "bartlett",
        };
        let x = self.parse_vector_info(&inputs[0], loop_vars)?;
        let out = self.parse_vector_info(&outputs[0], loop_vars)?;
        if out.size != x.size {
            return Err(parse_err(
                "Window requires the output vector to match the input length.",
            ));
        }
        let suffix = format!("{}", x.size);
        self.emit_elementwise(&out, &format!("window${tag}"), &suffix, &x.elements, source)
    }

    /// `CALL Filter(b, a, x : y)` and `CALL FiltFilt(b, a, x : y)` — the causal
    /// difference equation and its zero-phase forward/backward pair.
    fn flatten_filter(
        &mut self,
        zero_phase: bool,
        inputs: &[Expr],
        outputs: &[Expr],
        source: &str,
        loop_vars: &Scope,
    ) -> Result<()> {
        let name = if zero_phase { "FiltFilt" } else { "Filter" };
        if inputs.len() != 3 || outputs.len() != 1 {
            return Err(parse_err(format!(
                "{name} expects numerator, denominator and signal vectors plus 1 \
                 output vector, e.g. CALL {name}(b[1:3], a[1:2], x[1:n] : y[1:n])"
            )));
        }
        let b = self.parse_vector_info(&inputs[0], loop_vars)?;
        let a = self.parse_vector_info(&inputs[1], loop_vars)?;
        let x = self.parse_vector_info(&inputs[2], loop_vars)?;
        let out = self.parse_vector_info(&outputs[0], loop_vars)?;
        if out.size != x.size {
            return Err(parse_err(format!(
                "{name} requires the output vector to match the signal length."
            )));
        }
        let mut entries = Vec::with_capacity(b.size + a.size + x.size);
        entries.extend(b.elements);
        entries.extend(a.elements);
        entries.extend(x.elements);
        let tag = if zero_phase { "zero" } else { "causal" };
        let suffix = format!("{}${}${}", b.size, a.size, x.size);
        self.emit_elementwise(&out, &format!("filter${tag}"), &suffix, &entries, source)
    }

    /// `CALL XCorr(a, b : c)` — full cross-correlation, `m + n - 1` long, with
    /// zero lag at the centre element `c[n]`.
    fn flatten_xcorr(
        &mut self,
        inputs: &[Expr],
        outputs: &[Expr],
        source: &str,
        loop_vars: &Scope,
    ) -> Result<()> {
        if inputs.len() != 2 || outputs.len() != 1 {
            return Err(parse_err(
                "XCorr expects 2 input vectors and 1 output vector, \
                 e.g. CALL XCorr(a[1:m], b[1:n] : c[1:m+n-1])",
            ));
        }
        let a = self.parse_vector_info(&inputs[0], loop_vars)?;
        let b = self.parse_vector_info(&inputs[1], loop_vars)?;
        let c = self.parse_vector_info(&outputs[0], loop_vars)?;
        let (m, n) = (a.size, b.size);
        if c.size != m + n - 1 {
            return Err(parse_err(format!(
                "XCorr requires the output length to be m + n - 1 = {}.",
                m + n - 1
            )));
        }
        let mut entries = Vec::with_capacity(m + n);
        entries.extend(a.elements);
        entries.extend(b.elements);
        self.emit_elementwise(&c, "xcorr", &format!("{m}${n}"), &entries, source)
    }

    /// `CALL Welch(x, fs, nperseg : f, pxx)` — the averaged-periodogram PSD.
    /// Both outputs are `nperseg/2 + 1` long; `fs` stays an expression, so a
    /// sample rate computed elsewhere in the document works.
    fn flatten_welch(
        &mut self,
        inputs: &[Expr],
        outputs: &[Expr],
        source: &str,
        loop_vars: &Scope,
    ) -> Result<()> {
        if inputs.len() != 3 || outputs.len() != 2 {
            return Err(parse_err(
                "Welch expects a signal vector, a sample rate and a segment length, \
                 plus frequency and PSD output vectors, e.g. \
                 CALL Welch(x[1:n], 1000, 256 : f[1:129], pxx[1:129])",
            ));
        }
        let x = self.parse_vector_info(&inputs[0], loop_vars)?;
        let nperseg = self.in_scalar_int(inputs, 2, loop_vars)?;
        if nperseg < 2 {
            return Err(parse_err(format!(
                "Welch requires a segment length of at least 2, got {nperseg}."
            )));
        }
        let nperseg = nperseg as usize;
        if nperseg > x.size {
            return Err(parse_err(format!(
                "Welch segment length ({nperseg}) exceeds the series length ({}).",
                x.size
            )));
        }
        let bins = nperseg / 2 + 1;
        let freq = self.parse_vector_info(&outputs[0], loop_vars)?;
        let pxx = self.parse_vector_info(&outputs[1], loop_vars)?;
        if freq.size != bins || pxx.size != bins {
            return Err(parse_err(format!(
                "Welch requires both output vectors to be nperseg/2 + 1 = {bins} long."
            )));
        }
        let mut entries = Vec::with_capacity(x.size + 1);
        entries.push(inputs[1].clone());
        entries.extend(x.elements);
        let suffix = format!("{nperseg}${}", x.size);
        self.emit_elementwise(&freq, "welch$f", &suffix, &entries, source)?;
        self.emit_elementwise(&pxx, "welch$pxx", &suffix, &entries, source)
    }

    /// Port of `EquationParser.flattenConvolve`: the linear convolution of two
    /// vectors, `m + n - 1` long.
    fn flatten_convolve(
        &mut self,
        inputs: &[Expr],
        outputs: &[Expr],
        source: &str,
        loop_vars: &Scope,
    ) -> Result<()> {
        if inputs.len() != 2 || outputs.len() != 1 {
            return Err(parse_err(
                "Convolve expects 2 input vectors and 1 output vector, \
                 e.g. CALL Convolve(a[1:m], b[1:n] : c[1:m+n-1])",
            ));
        }
        let a = self.parse_vector_info(&inputs[0], loop_vars)?;
        let b = self.parse_vector_info(&inputs[1], loop_vars)?;
        let c = self.parse_vector_info(&outputs[0], loop_vars)?;
        let (m, n) = (a.size, b.size);
        if c.size != m + n - 1 {
            return Err(parse_err(format!(
                "Convolve requires the output length to be m + n - 1 = {}.",
                m + n - 1
            )));
        }
        self.reserve(c.size)?;
        let mut entries = Vec::with_capacity(m + n);
        entries.extend(a.elements);
        entries.extend(b.elements);
        for k in 0..c.size {
            self.push(Equation::new(
                c.elements[k].clone(),
                Expr::Call {
                    function: format!("conv${k}${m}${n}"),
                    args: entries.clone(),
                },
                source,
            ))?;
        }
        Ok(())
    }

    /// Port of `EquationParser.flattenLinFit`: the ordinary-least-squares line
    /// through `(x, y)`, reported as three *scalar* outputs.
    fn flatten_lin_fit(
        &mut self,
        inputs: &[Expr],
        outputs: &[Expr],
        source: &str,
        loop_vars: &Scope,
    ) -> Result<()> {
        if inputs.len() != 2 || outputs.len() != 3 {
            return Err(parse_err(
                "LinFit expects 2 input vectors and 3 outputs (slope, intercept, r2), \
                 e.g. CALL LinFit(x[1:n], y[1:n] : slope, intercept, r2)",
            ));
        }
        let x = self.parse_vector_info(&inputs[0], loop_vars)?;
        let y = self.parse_vector_info(&inputs[1], loop_vars)?;
        let n = x.size;
        if y.size != n {
            return Err(parse_err("LinFit requires x and y of equal length."));
        }
        let mut entries = Vec::with_capacity(2 * n);
        entries.extend(x.elements);
        entries.extend(y.elements);
        for (k, part) in ["slope", "intercept", "r2"].iter().enumerate() {
            let out = self.expand_expr(&outputs[k], loop_vars)?;
            self.push(Equation::new(
                out,
                Expr::Call {
                    function: format!("linfit${part}${n}"),
                    args: entries.clone(),
                },
                source,
            ))?;
        }
        Ok(())
    }

    /// Port of `EquationParser.flattenPolyFit`: the least-squares polynomial
    /// coefficients in **ascending** powers, `degree + 1` of them.
    fn flatten_poly_fit(
        &mut self,
        inputs: &[Expr],
        outputs: &[Expr],
        source: &str,
        loop_vars: &Scope,
    ) -> Result<()> {
        if inputs.len() != 3 || outputs.len() != 1 {
            return Err(parse_err(
                "PolyFit expects 2 input vectors and a degree, plus 1 output coefficient \
                 vector, e.g. CALL PolyFit(x[1:n], y[1:n], 2 : c[1:3])",
            ));
        }
        let x = self.parse_vector_info(&inputs[0], loop_vars)?;
        let y = self.parse_vector_info(&inputs[1], loop_vars)?;
        let degree = self.in_scalar_int(inputs, 2, loop_vars)?;
        let n = x.size;
        if y.size != n {
            return Err(parse_err("PolyFit requires x and y of equal length."));
        }
        if degree < 0 {
            return Err(parse_err("PolyFit degree must be >= 0."));
        }
        // Saturating so an absurd degree cannot overflow before the length
        // check rejects it; a real coefficient vector is capped by
        // `MAX_RANGE_SPAN`, so `degree` is small once the check passes.
        let wanted = degree.saturating_add(1);
        let c = self.parse_vector_info(&outputs[0], loop_vars)?;
        if c.size as i64 != wanted {
            return Err(parse_err(format!(
                "PolyFit requires a coefficient vector of length degree + 1 = {wanted}."
            )));
        }
        let degree = degree as usize;
        self.reserve(degree + 1)?;
        let mut entries = Vec::with_capacity(2 * n);
        entries.extend(x.elements);
        entries.extend(y.elements);
        for k in 0..=degree {
            self.push(Equation::new(
                c.elements[k].clone(),
                Expr::Call {
                    function: format!("polyfit${k}${degree}${n}"),
                    args: entries.clone(),
                },
                source,
            ))?;
        }
        Ok(())
    }

    // ── Matrix expression compilation ───────────────────────────────────────

    /// Port of `EquationParser.compileMatrixExpr`.
    fn compile_matrix_expr(
        &mut self,
        e: &Expr,
        loop_vars: &Scope,
        shapes: &mut HashMap<String, Shape>,
    ) -> Result<Matrix> {
        match e {
            Expr::ArrayAccess { name, indices } => {
                self.matrix_from_array_access(e, name, indices, loop_vars)
            }
            Expr::ArrayLiteral(elements) => self.matrix_from_literal(elements, loop_vars),
            Expr::BinOp { op, left, right } => {
                self.compile_matrix_bin_op(*op, left, right, loop_vars, shapes)
            }
            Expr::Call { function, args } => {
                self.compile_matrix_call(function, args, loop_vars, shapes)
            }
            other => Ok(vec![vec![self.expand_expr(other, loop_vars)?]]),
        }
    }

    fn matrix_from_array_access(
        &self,
        e: &Expr,
        name: &str,
        indices: &[Expr],
        loop_vars: &Scope,
    ) -> Result<Matrix> {
        match indices.len() {
            2 => Ok(self.parse_matrix_info(e, loop_vars)?.elements),
            1 => {
                let v = self.parse_vector_info(e, loop_vars)?;
                Ok(v.elements.into_iter().map(|el| vec![el]).collect())
            }
            _ => Err(parse_err(format!(
                "Matrix/vector must have 1 or 2 dimensions: {name}"
            ))),
        }
    }

    /// Port of `matrixFromLiteral` / `matrixFromRowLiterals`: a literal whose
    /// first element is itself a literal is row-major (`[1 2; 3 4]`); a flat
    /// literal is a column vector.
    fn matrix_from_literal(&self, elements: &[Expr], loop_vars: &Scope) -> Result<Matrix> {
        if elements.is_empty() {
            return Err(parse_err("Matrix literal must not be empty."));
        }
        if matches!(elements[0], Expr::ArrayLiteral(_)) {
            let mut num_cols: Option<usize> = None;
            let mut matrix: Matrix = Vec::with_capacity(elements.len());
            for (i, row) in elements.iter().enumerate() {
                let Expr::ArrayLiteral(row_elements) = row else {
                    return Err(parse_err(format!(
                        "Heterogeneous matrix literal: row {} is not a row literal.",
                        i + 1
                    )));
                };
                if row_elements.is_empty() {
                    return Err(parse_err("Matrix literal rows must not be empty."));
                }
                match num_cols {
                    None => num_cols = Some(row_elements.len()),
                    Some(cols) if cols != row_elements.len() => {
                        return Err(parse_err(
                            "Matrix literal rows must have compatible column dimensions.",
                        ))
                    }
                    Some(_) => {}
                }
                let mut out_row = Vec::with_capacity(row_elements.len());
                for element in row_elements {
                    out_row.push(self.expand_expr(element, loop_vars)?);
                }
                matrix.push(out_row);
            }
            return Ok(matrix);
        }
        let mut matrix = Vec::with_capacity(elements.len());
        for element in elements {
            matrix.push(vec![self.expand_expr(element, loop_vars)?]);
        }
        Ok(matrix)
    }

    /// Port of `EquationParser.compileMatrixBinOp`.
    fn compile_matrix_bin_op(
        &mut self,
        op: BinOp,
        left: &Expr,
        right: &Expr,
        loop_vars: &Scope,
        shapes: &mut HashMap<String, Shape>,
    ) -> Result<Matrix> {
        match op {
            BinOp::Add | BinOp::Sub => self.matrix_add_sub(op, left, right, loop_vars, shapes),
            BinOp::Mul => self.matrix_multiply(left, right, loop_vars, shapes),
            BinOp::LeftDiv => self.matrix_backslash(left, right, loop_vars, shapes),
            _ if op.is_element_wise() => {
                self.compile_elementwise(op, left, right, loop_vars, shapes)
            }
            other => Err(parse_err(format!(
                "Unsupported binary matrix operator: {}",
                other.as_str()
            ))),
        }
    }

    /// Element-wise op (`.*`, `./`, `.\`, `.^`): the base scalar op applied to
    /// each pair of elements, with scalar broadcasting on either side.
    fn compile_elementwise(
        &mut self,
        op: BinOp,
        left: &Expr,
        right: &Expr,
        loop_vars: &Scope,
        shapes: &mut HashMap<String, Shape>,
    ) -> Result<Matrix> {
        let base = op.scalar_equivalent();
        let l_mat = self.compile_matrix_expr(left, loop_vars, shapes)?;
        let r_mat = self.compile_matrix_expr(right, loop_vars, shapes)?;
        let l_scalar = is_1x1(&l_mat);
        let r_scalar = is_1x1(&r_mat);
        let (rows, cols) = elementwise_dims(op, &l_mat, &r_mat, l_scalar, r_scalar)?;
        let mut result = Vec::with_capacity(rows);
        for i in 0..rows {
            let mut row = Vec::with_capacity(cols);
            for j in 0..cols {
                let a = if l_scalar { &l_mat[0][0] } else { &l_mat[i][j] };
                let b = if r_scalar { &r_mat[0][0] } else { &r_mat[i][j] };
                // Left divide A .\ B is element-wise B / A (the evaluator has
                // no scalar `\` op, so emit a division with swapped operands).
                row.push(if op == BinOp::ElemLeftDiv {
                    Expr::bin(BinOp::Div, b.clone(), a.clone())
                } else {
                    Expr::bin(base, a.clone(), b.clone())
                });
            }
            result.push(row);
        }
        Ok(result)
    }

    fn matrix_add_sub(
        &mut self,
        op: BinOp,
        left: &Expr,
        right: &Expr,
        loop_vars: &Scope,
        shapes: &mut HashMap<String, Shape>,
    ) -> Result<Matrix> {
        let l_mat = self.compile_matrix_expr(left, loop_vars, shapes)?;
        let r_mat = self.compile_matrix_expr(right, loop_vars, shapes)?;
        if is_1x1(&l_mat) {
            return Ok(broadcast_scalar(op, &l_mat[0][0], &r_mat, true));
        }
        if is_1x1(&r_mat) {
            return Ok(broadcast_scalar(op, &r_mat[0][0], &l_mat, false));
        }
        if l_mat.len() != r_mat.len() || l_mat[0].len() != r_mat[0].len() {
            return Err(parse_err(format!(
                "Matrix dimensions must agree for addition/subtraction: {}x{} vs {}x{}",
                l_mat.len(),
                l_mat[0].len(),
                r_mat.len(),
                r_mat[0].len()
            )));
        }
        Ok(l_mat
            .iter()
            .zip(&r_mat)
            .map(|(l_row, r_row)| {
                l_row
                    .iter()
                    .zip(r_row)
                    .map(|(l, r)| Expr::bin(op, l.clone(), r.clone()))
                    .collect()
            })
            .collect())
    }

    fn matrix_multiply(
        &mut self,
        left: &Expr,
        right: &Expr,
        loop_vars: &Scope,
        shapes: &mut HashMap<String, Shape>,
    ) -> Result<Matrix> {
        let l_mat = self.compile_matrix_expr(left, loop_vars, shapes)?;
        let r_mat = self.compile_matrix_expr(right, loop_vars, shapes)?;
        if is_1x1(&l_mat) {
            return Ok(broadcast_scalar(BinOp::Mul, &l_mat[0][0], &r_mat, true));
        }
        if is_1x1(&r_mat) {
            return Ok(broadcast_scalar(BinOp::Mul, &r_mat[0][0], &l_mat, false));
        }
        if l_mat[0].len() != r_mat.len() {
            return Err(parse_err(format!(
                "Inner matrix dimensions must agree: {}x{} vs {}x{}",
                l_mat.len(),
                l_mat[0].len(),
                r_mat.len(),
                r_mat[0].len()
            )));
        }
        Ok(mat_mul(&l_mat, &r_mat))
    }

    /// `A \ b`: introduces a fresh unknown vector and emits `A·x = b`. Port of
    /// `EquationParser.matrixBackslash`.
    fn matrix_backslash(
        &mut self,
        left: &Expr,
        right: &Expr,
        loop_vars: &Scope,
        shapes: &mut HashMap<String, Shape>,
    ) -> Result<Matrix> {
        let a_mat = self.compile_matrix_expr(left, loop_vars, shapes)?;
        let b_mat = self.compile_matrix_expr(right, loop_vars, shapes)?;
        if a_mat.len() != a_mat[0].len() {
            return Err(parse_err("Backslash solver requires square matrix A"));
        }
        if b_mat.len() != a_mat.len() || b_mat[0].len() != 1 {
            return Err(parse_err(format!(
                "Backslash solver dimensions mismatch: A is {}x{}, b is {}x{}",
                a_mat.len(),
                a_mat[0].len(),
                b_mat.len(),
                b_mat[0].len()
            )));
        }
        self.emit_linear_solve(&a_mat, &b_mat, "backslash_temp_", "Backslash solve")
    }

    /// Introduces a fresh unknown vector x and emits `A·x = b` row equations,
    /// returning x. The temp name embeds the equation count at emission time,
    /// exactly like the Java code, so names are deterministic.
    fn emit_linear_solve(
        &mut self,
        a_mat: &Matrix,
        b_mat: &Matrix,
        prefix: &str,
        label: &str,
    ) -> Result<Matrix> {
        let m = a_mat.len();
        let temp_vec_name = format!("{prefix}{}", self.out.len());
        let x_mat: Matrix = (1..=m)
            .map(|i| vec![Expr::Var(format!("{temp_vec_name}[{i}]"))])
            .collect();
        for i in 0..m {
            let mut term = Expr::bin(BinOp::Mul, a_mat[i][0].clone(), x_mat[0][0].clone());
            for k in 1..m {
                term = Expr::bin(
                    BinOp::Add,
                    term,
                    Expr::bin(BinOp::Mul, a_mat[i][k].clone(), x_mat[k][0].clone()),
                );
            }
            self.push(Equation::new(term, b_mat[i][0].clone(), label))?;
        }
        Ok(x_mat)
    }

    /// Port of `EquationParser.compileMatrixCall`.
    fn compile_matrix_call(
        &mut self,
        function: &str,
        args: &[Expr],
        loop_vars: &Scope,
        shapes: &mut HashMap<String, Shape>,
    ) -> Result<Matrix> {
        match function {
            "zeros" | "ones" | "eye" | "identity" | "diag" | "linspace" => {
                self.compile_matrix_generator(function, args, loop_vars, shapes)
            }
            "transpose" => {
                let mat =
                    self.compile_matrix_expr(arg_at(args, 0, "transpose")?, loop_vars, shapes)?;
                let rows = mat.len();
                let cols = mat[0].len();
                Ok((0..cols)
                    .map(|j| (0..rows).map(|i| mat[i][j].clone()).collect())
                    .collect())
            }
            "inverse" | "inv" => self.matrix_inverse(args, loop_vars, shapes),
            "axpy" => self.matrix_axpy(args, loop_vars, shapes),
            "scal" => self.matrix_scal(args, loop_vars, shapes),
            "gemv" => self.matrix_gemv(args, loop_vars, shapes),
            "gemm" => self.matrix_gemm(args, loop_vars, shapes),
            "ger" => self.matrix_ger(args, loop_vars, shapes),
            "copy" => {
                if args.len() != 1 {
                    return Err(parse_err("copy expects exactly 1 argument: copy(x)"));
                }
                self.compile_matrix_expr(&args[0], loop_vars, shapes)
            }
            "solvelinear" => self.matrix_solve_linear(args, loop_vars, shapes),
            other => Err(parse_err(format!("Unsupported matrix function: {other}"))),
        }
    }

    /// A matrix function used inside a larger expression: materialise the
    /// inverse as `inverse_temp_N` unknowns pinned by `A·A⁻¹ = I` equations.
    /// Port of `EquationParser.matrixInverse`.
    fn matrix_inverse(
        &mut self,
        args: &[Expr],
        loop_vars: &Scope,
        shapes: &mut HashMap<String, Shape>,
    ) -> Result<Matrix> {
        let a_mat = self.compile_matrix_expr(arg_at(args, 0, "inverse")?, loop_vars, shapes)?;
        if a_mat.len() != a_mat[0].len() {
            return Err(parse_err("Inverse requires a square matrix"));
        }
        let m = a_mat.len();
        let temp_mat_name = format!("inverse_temp_{}", self.out.len());
        let inv_mat: Matrix = (1..=m)
            .map(|i| {
                (1..=m)
                    .map(|j| Expr::Var(format!("{temp_mat_name}[{i},{j}]")))
                    .collect()
            })
            .collect();
        for i in 0..m {
            for j in 0..m {
                let mut term = Expr::bin(BinOp::Mul, a_mat[i][0].clone(), inv_mat[0][j].clone());
                for k in 1..m {
                    term = Expr::bin(
                        BinOp::Add,
                        term,
                        Expr::bin(BinOp::Mul, a_mat[i][k].clone(), inv_mat[k][j].clone()),
                    );
                }
                self.push(Equation::new(
                    term,
                    Expr::num(kronecker_delta(i, j)),
                    "Matrix inverse definition",
                ))?;
            }
        }
        Ok(inv_mat)
    }

    fn matrix_axpy(
        &mut self,
        args: &[Expr],
        loop_vars: &Scope,
        shapes: &mut HashMap<String, Shape>,
    ) -> Result<Matrix> {
        if args.len() != 3 {
            return Err(parse_err(
                "axpy expects exactly 3 arguments: axpy(alpha, x, y)",
            ));
        }
        let alpha = &args[0]; // used raw, as in the Java code
        let x_mat = self.compile_matrix_expr(&args[1], loop_vars, shapes)?;
        let y_mat = self.compile_matrix_expr(&args[2], loop_vars, shapes)?;
        if x_mat.len() != y_mat.len() || x_mat[0].len() != y_mat[0].len() {
            return Err(parse_err(format!(
                "axpy dimension mismatch: x is {}x{}, y is {}x{}",
                x_mat.len(),
                x_mat[0].len(),
                y_mat.len(),
                y_mat[0].len()
            )));
        }
        Ok(x_mat
            .iter()
            .zip(&y_mat)
            .map(|(x_row, y_row)| {
                x_row
                    .iter()
                    .zip(y_row)
                    .map(|(x, y)| {
                        Expr::bin(
                            BinOp::Add,
                            Expr::bin(BinOp::Mul, alpha.clone(), x.clone()),
                            y.clone(),
                        )
                    })
                    .collect()
            })
            .collect())
    }

    fn matrix_scal(
        &mut self,
        args: &[Expr],
        loop_vars: &Scope,
        shapes: &mut HashMap<String, Shape>,
    ) -> Result<Matrix> {
        if args.len() != 2 {
            return Err(parse_err(
                "scal expects exactly 2 arguments: scal(alpha, x)",
            ));
        }
        let alpha = &args[0];
        let x_mat = self.compile_matrix_expr(&args[1], loop_vars, shapes)?;
        Ok(broadcast_scalar(BinOp::Mul, alpha, &x_mat, true))
    }

    fn matrix_gemv(
        &mut self,
        args: &[Expr],
        loop_vars: &Scope,
        shapes: &mut HashMap<String, Shape>,
    ) -> Result<Matrix> {
        if args.len() != 5 {
            return Err(parse_err(
                "gemv expects exactly 5 arguments: gemv(alpha, A, x, beta, y)",
            ));
        }
        let alpha = &args[0];
        let a_mat = self.compile_matrix_expr(&args[1], loop_vars, shapes)?;
        let x_mat = self.compile_matrix_expr(&args[2], loop_vars, shapes)?;
        let beta = &args[3];
        let y_mat = self.compile_matrix_expr(&args[4], loop_vars, shapes)?;
        if x_mat[0].len() != 1 {
            return Err(parse_err(format!(
                "gemv: x must be a column vector (got {}x{})",
                x_mat.len(),
                x_mat[0].len()
            )));
        }
        if y_mat[0].len() != 1 {
            return Err(parse_err(format!(
                "gemv: y must be a column vector (got {}x{})",
                y_mat.len(),
                y_mat[0].len()
            )));
        }
        if a_mat[0].len() != x_mat.len() {
            return Err(parse_err(format!(
                "gemv inner dimension mismatch: A is {}x{}, x is {}x{}",
                a_mat.len(),
                a_mat[0].len(),
                x_mat.len(),
                x_mat[0].len()
            )));
        }
        if a_mat.len() != y_mat.len() {
            return Err(parse_err(format!(
                "gemv outer dimension mismatch: A is {}x{}, y is {}x{}",
                a_mat.len(),
                a_mat[0].len(),
                y_mat.len(),
                y_mat[0].len()
            )));
        }
        let m = a_mat.len();
        let n = a_mat[0].len();
        let mut result = Vec::with_capacity(m);
        for i in 0..m {
            let mut sum = Expr::bin(BinOp::Mul, a_mat[i][0].clone(), x_mat[0][0].clone());
            for k in 1..n {
                sum = Expr::bin(
                    BinOp::Add,
                    sum,
                    Expr::bin(BinOp::Mul, a_mat[i][k].clone(), x_mat[k][0].clone()),
                );
            }
            result.push(vec![Expr::bin(
                BinOp::Add,
                Expr::bin(BinOp::Mul, alpha.clone(), sum),
                Expr::bin(BinOp::Mul, beta.clone(), y_mat[i][0].clone()),
            )]);
        }
        Ok(result)
    }

    fn matrix_gemm(
        &mut self,
        args: &[Expr],
        loop_vars: &Scope,
        shapes: &mut HashMap<String, Shape>,
    ) -> Result<Matrix> {
        if args.len() != 5 {
            return Err(parse_err(
                "gemm expects exactly 5 arguments: gemm(alpha, A, B, beta, C)",
            ));
        }
        let alpha = &args[0];
        let a_mat = self.compile_matrix_expr(&args[1], loop_vars, shapes)?;
        let b_mat = self.compile_matrix_expr(&args[2], loop_vars, shapes)?;
        let beta = &args[3];
        let c_mat = self.compile_matrix_expr(&args[4], loop_vars, shapes)?;
        if a_mat[0].len() != b_mat.len() {
            return Err(parse_err(format!(
                "gemm inner dimension mismatch: A is {}x{}, B is {}x{}",
                a_mat.len(),
                a_mat[0].len(),
                b_mat.len(),
                b_mat[0].len()
            )));
        }
        if a_mat.len() != c_mat.len() || b_mat[0].len() != c_mat[0].len() {
            return Err(parse_err(format!(
                "gemm output dimension mismatch: C must be {}x{} (got {}x{})",
                a_mat.len(),
                b_mat[0].len(),
                c_mat.len(),
                c_mat[0].len()
            )));
        }
        let m = a_mat.len();
        let n = b_mat[0].len();
        let inner = a_mat[0].len();
        let mut result = Vec::with_capacity(m);
        for i in 0..m {
            let mut row = Vec::with_capacity(n);
            for j in 0..n {
                let mut sum = Expr::bin(BinOp::Mul, a_mat[i][0].clone(), b_mat[0][j].clone());
                for p in 1..inner {
                    sum = Expr::bin(
                        BinOp::Add,
                        sum,
                        Expr::bin(BinOp::Mul, a_mat[i][p].clone(), b_mat[p][j].clone()),
                    );
                }
                row.push(Expr::bin(
                    BinOp::Add,
                    Expr::bin(BinOp::Mul, alpha.clone(), sum),
                    Expr::bin(BinOp::Mul, beta.clone(), c_mat[i][j].clone()),
                ));
            }
            result.push(row);
        }
        Ok(result)
    }

    fn matrix_ger(
        &mut self,
        args: &[Expr],
        loop_vars: &Scope,
        shapes: &mut HashMap<String, Shape>,
    ) -> Result<Matrix> {
        if args.len() != 4 {
            return Err(parse_err(
                "ger expects exactly 4 arguments: ger(alpha, x, y, A)",
            ));
        }
        let alpha = &args[0];
        let x_mat = self.compile_matrix_expr(&args[1], loop_vars, shapes)?;
        let y_mat = self.compile_matrix_expr(&args[2], loop_vars, shapes)?;
        let a_mat = self.compile_matrix_expr(&args[3], loop_vars, shapes)?;
        if x_mat[0].len() != 1 {
            return Err(parse_err(format!(
                "ger: x must be a column vector (got {}x{})",
                x_mat.len(),
                x_mat[0].len()
            )));
        }
        if y_mat[0].len() != 1 {
            return Err(parse_err(format!(
                "ger: y must be a column vector (got {}x{})",
                y_mat.len(),
                y_mat[0].len()
            )));
        }
        if a_mat.len() != x_mat.len() || a_mat[0].len() != y_mat.len() {
            return Err(parse_err(format!(
                "ger dimension mismatch: A must be {}x{} (got {}x{})",
                x_mat.len(),
                y_mat.len(),
                a_mat.len(),
                a_mat[0].len()
            )));
        }
        let m = a_mat.len();
        let n = a_mat[0].len();
        let mut result = Vec::with_capacity(m);
        for i in 0..m {
            let mut row = Vec::with_capacity(n);
            for j in 0..n {
                row.push(Expr::bin(
                    BinOp::Add,
                    Expr::bin(
                        BinOp::Mul,
                        alpha.clone(),
                        Expr::bin(BinOp::Mul, x_mat[i][0].clone(), y_mat[j][0].clone()),
                    ),
                    a_mat[i][j].clone(),
                ));
            }
            result.push(row);
        }
        Ok(result)
    }

    /// `solvelinear(A, b)` inside a larger expression: like backslash but with
    /// its own temp prefix. Port of `EquationParser.matrixSolveLinear`.
    fn matrix_solve_linear(
        &mut self,
        args: &[Expr],
        loop_vars: &Scope,
        shapes: &mut HashMap<String, Shape>,
    ) -> Result<Matrix> {
        if args.len() != 2 {
            return Err(parse_err(
                "solvelinear expects exactly 2 arguments: solvelinear(A, b)",
            ));
        }
        let a_mat = self.compile_matrix_expr(&args[0], loop_vars, shapes)?;
        let b_mat = self.compile_matrix_expr(&args[1], loop_vars, shapes)?;
        if a_mat.len() != a_mat[0].len() {
            return Err(parse_err("solvelinear requires square matrix A"));
        }
        if b_mat.len() != a_mat.len() || b_mat[0].len() != 1 {
            return Err(parse_err(format!(
                "solvelinear dimensions mismatch: A is {}x{}, b is {}x{}",
                a_mat.len(),
                a_mat[0].len(),
                b_mat.len(),
                b_mat[0].len()
            )));
        }
        self.emit_linear_solve(&a_mat, &b_mat, "solvelinear_temp_", "SolveLinear solve")
    }

    // ── Generators ──────────────────────────────────────────────────────────

    /// Port of `EquationParser.compileMatrixGenerator` and the `gen*` helpers.
    fn compile_matrix_generator(
        &mut self,
        function: &str,
        args: &[Expr],
        loop_vars: &Scope,
        shapes: &mut HashMap<String, Shape>,
    ) -> Result<Matrix> {
        match function {
            "zeros" | "ones" => self.gen_constant(function, args, loop_vars),
            "eye" | "identity" => self.gen_identity(function, args, loop_vars),
            "linspace" => self.gen_linspace(function, args, loop_vars),
            _ => self.gen_diag(args, loop_vars, shapes), // diag
        }
    }

    /// A compile-time numeric generator argument.
    fn gen_num(&self, e: &Expr, loop_vars: &Scope) -> Result<f64> {
        let expanded = self.expand_expr(e, loop_vars)?;
        self.eval_index_expr(&expanded, loop_vars)
    }

    fn check_generator_size(&self, rows: i64, cols: i64, function: &str) -> Result<(usize, usize)> {
        if rows < 1 || cols < 1 {
            return Err(parse_err(format!("{function} dimensions must be >= 1")));
        }
        if (rows as i128) * (cols as i128) > MAX_RANGE_SPAN {
            return Err(parse_err(format!(
                "{function} matrix is too large (limit {MAX_RANGE_SPAN})."
            )));
        }
        Ok((rows as usize, cols as usize))
    }

    fn gen_constant(&self, function: &str, args: &[Expr], loop_vars: &Scope) -> Result<Matrix> {
        let r = java_round(self.gen_num(arg_at(args, 0, function)?, loop_vars)?);
        let c = if args.len() > 1 {
            java_round(self.gen_num(&args[1], loop_vars)?)
        } else {
            r
        };
        let (rows, cols) = self.check_generator_size(r, c, function)?;
        let fill = if function == "ones" { 1.0 } else { 0.0 };
        Ok(vec![vec![Expr::num(fill); cols]; rows])
    }

    fn gen_identity(&self, function: &str, args: &[Expr], loop_vars: &Scope) -> Result<Matrix> {
        let r = java_round(self.gen_num(arg_at(args, 0, function)?, loop_vars)?);
        let c = if args.len() > 1 {
            java_round(self.gen_num(&args[1], loop_vars)?)
        } else {
            r
        };
        let (rows, cols) = self.check_generator_size(r, c, function)?;
        Ok((0..rows)
            .map(|i| {
                (0..cols)
                    .map(|j| Expr::num(kronecker_delta(i, j)))
                    .collect()
            })
            .collect())
    }

    fn gen_linspace(&self, function: &str, args: &[Expr], loop_vars: &Scope) -> Result<Matrix> {
        let a = self.gen_num(arg_at(args, 0, function)?, loop_vars)?;
        let b = self.gen_num(arg_at(args, 1, function)?, loop_vars)?;
        let n = if args.len() > 2 {
            java_round(self.gen_num(&args[2], loop_vars)?)
        } else {
            100
        };
        let (n, _) = self.check_generator_size(n, 1, function)?;
        Ok((0..n)
            .map(|k| {
                let t = if n == 1 {
                    b
                } else {
                    a + (b - a) * (k as f64) / ((n - 1) as f64)
                };
                vec![Expr::num(t)]
            })
            .collect())
    }

    fn gen_diag(
        &mut self,
        args: &[Expr],
        loop_vars: &Scope,
        shapes: &mut HashMap<String, Shape>,
    ) -> Result<Matrix> {
        let v = self.compile_matrix_expr(arg_at(args, 0, "diag")?, loop_vars, shapes)?;
        if v.len() == 1 || v[0].len() == 1 {
            // vector -> diagonal matrix
            let n = v.len().max(v[0].len());
            return Ok((0..n)
                .map(|i| {
                    (0..n)
                        .map(|j| {
                            if i == j {
                                if v.len() == 1 {
                                    v[0][i].clone()
                                } else {
                                    v[i][0].clone()
                                }
                            } else {
                                Expr::num(0.0)
                            }
                        })
                        .collect()
                })
                .collect());
        }
        // matrix -> extract diagonal
        let n = v.len().min(v[0].len());
        Ok((0..n).map(|i| vec![v[i][i].clone()]).collect())
    }

    // ── Slice resolution ────────────────────────────────────────────────────

    /// Resolve `A[r1:r2, c1:c2]` to its element variables. Port of
    /// `EquationParser.parseMatrixInfo` (descending ranges allowed).
    fn parse_matrix_info(&self, expr: &Expr, loop_vars: &Scope) -> Result<MatrixInfo> {
        let Expr::ArrayAccess { name, indices } = expr else {
            return Err(parse_err("Expected matrix array access: e.g. A[1:3, 1:3]"));
        };
        if indices.len() != 2 {
            return Err(parse_err(format!(
                "Matrix must have exactly 2 dimensions: {name}"
            )));
        }
        let r0 = self.expand_expr(&indices[0], loop_vars)?;
        let r1 = self.expand_expr(&indices[1], loop_vars)?;
        let (
            Expr::Range {
                start: start0,
                end: end0,
            },
            Expr::Range {
                start: start1,
                end: end1,
            },
        ) = (&r0, &r1)
        else {
            return Err(parse_err(
                "Matrix indices must specify ranges: e.g. A[1:3, 1:3]",
            ));
        };
        let r_start = java_round(self.eval_index_expr(start0, loop_vars)?);
        let r_end = java_round(self.eval_index_expr(end0, loop_vars)?);
        let c_start = java_round(self.eval_index_expr(start1, loop_vars)?);
        let c_end = java_round(self.eval_index_expr(end1, loop_vars)?);

        let num_rows = (r_end as i128 - r_start as i128).abs() + 1;
        let num_cols = (c_end as i128 - c_start as i128).abs() + 1;
        if num_rows * num_cols > MAX_RANGE_SPAN {
            return Err(parse_err(format!(
                "Matrix '{name}[...]' is too large ({} elements; limit {MAX_RANGE_SPAN}). \
                 Reduce the index ranges.",
                num_rows * num_cols
            )));
        }
        let num_rows = num_rows as usize;
        let num_cols = num_cols as usize;
        let r_dir: i64 = if r_start <= r_end { 1 } else { -1 };
        let c_dir: i64 = if c_start <= c_end { 1 } else { -1 };
        let elements = (0..num_rows)
            .map(|i| {
                let row_idx = r_start + (i as i64) * r_dir;
                (0..num_cols)
                    .map(|j| {
                        let col_idx = c_start + (j as i64) * c_dir;
                        Expr::Var(format!("{name}[{row_idx},{col_idx}]"))
                    })
                    .collect()
            })
            .collect();
        Ok(MatrixInfo {
            rows: num_rows,
            cols: num_cols,
            elements,
        })
    }

    /// Resolve `v[a:b]` to its element variables. Port of
    /// `EquationParser.parseVectorInfo`.
    fn parse_vector_info(&self, expr: &Expr, loop_vars: &Scope) -> Result<VectorInfo> {
        let Expr::ArrayAccess { name, indices } = expr else {
            return Err(parse_err("Expected vector array access: e.g. v[1:3]"));
        };
        if indices.len() != 1 {
            return Err(parse_err(format!(
                "Vector must have exactly 1 dimension: {name}"
            )));
        }
        let r0 = self.expand_expr(&indices[0], loop_vars)?;
        let Expr::Range { start, end } = &r0 else {
            return Err(parse_err("Vector index must specify a range: e.g. v[1:3]"));
        };
        let v_start = java_round(self.eval_index_expr(start, loop_vars)?);
        let v_end = java_round(self.eval_index_expr(end, loop_vars)?);
        let span = (v_end as i128 - v_start as i128).abs() + 1;
        if span > MAX_RANGE_SPAN {
            return Err(parse_err(format!(
                "Vector '{name}[...]' is too large ({span} elements; limit {MAX_RANGE_SPAN}). \
                 Reduce the index range."
            )));
        }
        let size = span as usize;
        let dir: i64 = if v_start <= v_end { 1 } else { -1 };
        let elements = (0..size)
            .map(|i| {
                let idx = v_start + (i as i64) * dir;
                Expr::Var(format!("{name}[{idx}]"))
            })
            .collect();
        Ok(VectorInfo { size, elements })
    }

    // ── Scalar expression expansion ─────────────────────────────────────────

    /// Port of `EquationParser.expandExpr`: substitutes loop variables,
    /// flattens constant-indexed array accesses into scalar variables, and
    /// splices slice arguments of scalar calls into their elements. Purely
    /// scalar expressions are rebuilt identically (byte-for-byte).
    fn expand_expr(&self, e: &Expr, loop_vars: &Scope) -> Result<Expr> {
        match e {
            Expr::Num { .. } | Expr::Str(_) => Ok(e.clone()),
            Expr::Var(name) => match loop_vars.get(name) {
                Some(value) => Ok(Expr::num(*value)),
                None => Ok(e.clone()),
            },
            Expr::Neg(operand) => Ok(Expr::Neg(Box::new(self.expand_expr(operand, loop_vars)?))),
            Expr::BinOp { op, left, right } => Ok(Expr::BinOp {
                op: *op,
                left: Box::new(self.expand_expr(left, loop_vars)?),
                right: Box::new(self.expand_expr(right, loop_vars)?),
            }),
            Expr::Call { function, args } => {
                let mut expanded_args = Vec::with_capacity(args.len());
                for arg in args {
                    match arg {
                        Expr::ArrayAccess { name, indices }
                            if indices.iter().any(|ix| matches!(ix, Expr::Range { .. })) =>
                        {
                            expanded_args.extend(
                                self.expand_array_access_to_elements(name, indices, loop_vars)?,
                            );
                        }
                        other => expanded_args.push(self.expand_expr(other, loop_vars)?),
                    }
                }
                Ok(Expr::Call {
                    function: function.clone(),
                    args: expanded_args,
                })
            }
            Expr::ArrayAccess { name, indices } => {
                if indices.iter().any(|ix| matches!(ix, Expr::Range { .. })) {
                    return Err(parse_err(format!(
                        "Array range '{name}[...]' is only allowed on the LHS of assignments \
                         or as function arguments."
                    )));
                }
                let mut eval_indices = Vec::with_capacity(indices.len());
                for index in indices {
                    let expanded = self.expand_expr(index, loop_vars)?;
                    let value = self.eval_index_expr(&expanded, loop_vars)?;
                    eval_indices.push(java_round(value));
                }
                let joined = eval_indices
                    .iter()
                    .map(|i| i.to_string())
                    .collect::<Vec<_>>()
                    .join(",");
                Ok(Expr::Var(format!("{name}[{joined}]")))
            }
            Expr::Range { start, end } => Ok(Expr::Range {
                start: Box::new(self.expand_expr(start, loop_vars)?),
                end: Box::new(self.expand_expr(end, loop_vars)?),
            }),
            Expr::ArrayLiteral(elements) => Ok(Expr::ArrayLiteral(
                elements
                    .iter()
                    .map(|el| self.expand_expr(el, loop_vars))
                    .collect::<Result<Vec<_>>>()?,
            )),
            Expr::Compare { op, left, right } => Ok(Expr::Compare {
                op: *op,
                left: Box::new(self.expand_expr(left, loop_vars)?),
                right: Box::new(self.expand_expr(right, loop_vars)?),
            }),
            Expr::Logical { op, left, right } => Ok(Expr::Logical {
                op: *op,
                left: Box::new(self.expand_expr(left, loop_vars)?),
                right: Box::new(self.expand_expr(right, loop_vars)?),
            }),
            Expr::Not(operand) => Ok(Expr::Not(Box::new(self.expand_expr(operand, loop_vars)?))),
        }
    }

    /// Splice a slice argument (`v[1:3]`, `A[1:2,1:2]`) into its element
    /// variables, row-major. Port of
    /// `EquationParser.expandArrayAccessToElements`.
    fn expand_array_access_to_elements(
        &self,
        name: &str,
        indices: &[Expr],
        loop_vars: &Scope,
    ) -> Result<Vec<Expr>> {
        let mut index_possibilities: Vec<Vec<i64>> = Vec::with_capacity(indices.len());
        for index in indices {
            let expanded = self.expand_expr(index, loop_vars)?;
            if let Expr::Range { start, end } = &expanded {
                index_possibilities.push(self.expand_range_index(start, end, loop_vars)?);
            } else {
                let value = self.eval_index_expr(&expanded, loop_vars)?;
                index_possibilities.push(vec![java_round(value)]);
            }
        }
        let mut total: i128 = 1;
        for dim in &index_possibilities {
            total *= dim.len() as i128;
            if total > MAX_RANGE_SPAN {
                return Err(parse_err(format!(
                    "Array expansion of '{name}[...]' is too large ({total} elements; \
                     limit {MAX_RANGE_SPAN}). Reduce the index ranges."
                )));
            }
        }
        let mut combinations: Vec<Vec<i64>> = vec![Vec::new()];
        for list in &index_possibilities {
            let mut next = Vec::with_capacity(combinations.len() * list.len());
            for prefix in &combinations {
                for value in list {
                    let mut combo = prefix.clone();
                    combo.push(*value);
                    next.push(combo);
                }
            }
            combinations = next;
        }
        Ok(combinations
            .into_iter()
            .map(|combo| {
                let joined = combo
                    .iter()
                    .map(|i| i.to_string())
                    .collect::<Vec<_>>()
                    .join(",");
                Expr::Var(format!("{name}[{joined}]"))
            })
            .collect())
    }

    /// Materialise a `start:end` index range into the explicit indices it
    /// covers (ascending or descending). Port of
    /// `EquationParser.expandRangeIndex`.
    fn expand_range_index(
        &self,
        start_expr: &Expr,
        end_expr: &Expr,
        loop_vars: &Scope,
    ) -> Result<Vec<i64>> {
        let start = java_round(self.eval_index_expr(start_expr, loop_vars)?);
        let end = java_round(self.eval_index_expr(end_expr, loop_vars)?);
        let count = (end as i128 - start as i128).abs() + 1;
        if count > MAX_RANGE_SPAN {
            return Err(parse_err(format!(
                "Array range is too large ({count} elements; limit {MAX_RANGE_SPAN}). \
                 Reduce the index range."
            )));
        }
        let dir: i64 = if start <= end { 1 } else { -1 };
        Ok((0..count as i64).map(|k| start + k * dir).collect())
    }

    /// Evaluate a compile-time index/size expression against constants and
    /// loop variables. Port of `EquationParser.evalIndexExpr`.
    ///
    /// Safety note: `crate::eval` is the engine's own numeric evaluator over
    /// the parsed, typed `Expr` AST (a safe expression evaluator) — it never
    /// executes arbitrary code.
    fn eval_index_expr(&self, e: &Expr, loop_vars: &Scope) -> Result<f64> {
        let mut combined = self.constants.clone();
        for (name, value) in loop_vars {
            combined.insert(name.clone(), *value);
        }
        eval::eval_with(e, &combined, EvalContext::with_defs(self.defs))
            .map_err(|_| parse_err("Array index expression cannot be evaluated to a constant"))
    }

    /// Evaluate a compile-time integer. Port of `EquationParser.constIndex`.
    fn const_index(&self, e: &Expr, loop_vars: &Scope) -> Result<i64> {
        let expanded = self.expand_expr(e, loop_vars)?;
        Ok(java_round(self.eval_index_expr(&expanded, loop_vars)?))
    }
}

// ---------------------------------------------------------------------------
// The control-systems back-reference
// ---------------------------------------------------------------------------

/// The Java `ControlSystemsFlattener` holds an `EquationParser` and calls five
/// of its methods back (`parseMatrixInfo`, `parseVectorInfo`, `expandExpr`,
/// `constIndex`, `registerShape`) plus `ctx.out().add`. `control::flatten`
/// states that back-reference as the [`Shapes`]/[`Host`] trait pair; these two
/// adapters supply it over this pass's [`Flattener`].
///
/// Two adapters rather than one because `auto_size` runs *before* any
/// emission and needs only `&Flattener`, while `flatten` needs `&mut`. Both
/// delegate to the same three free functions, so there is one implementation
/// of each query.
struct ControlShapes<'a, 'b> {
    flattener: &'b Flattener<'a>,
    loop_vars: &'b Scope,
}

struct ControlHost<'a, 'b> {
    flattener: &'b mut Flattener<'a>,
    loop_vars: &'b Scope,
    shapes: &'b mut HashMap<String, Shape>,
}

/// `parseMatrixInfo` plus the base name the Java `MatrixInfo` carries and this
/// port's [`MatrixInfo`] does not (no other caller reads it).
fn control_matrix_info(
    flattener: &Flattener<'_>,
    loop_vars: &Scope,
    expr: &Expr,
) -> Result<control::flatten::MatrixRef> {
    let info = flattener.parse_matrix_info(expr, loop_vars)?;
    Ok(control::flatten::MatrixRef {
        name: array_access_name(expr),
        rows: info.rows,
        cols: info.cols,
        elements: info.elements,
    })
}

fn control_vector_info(
    flattener: &Flattener<'_>,
    loop_vars: &Scope,
    expr: &Expr,
) -> Result<control::flatten::VectorRef> {
    let info = flattener.parse_vector_info(expr, loop_vars)?;
    Ok(control::flatten::VectorRef {
        name: array_access_name(expr),
        size: info.size,
        elements: info.elements,
    })
}

/// Both `parse_*_info` calls above reject anything but an `ArrayAccess`, so
/// the fallback is unreachable through them; it exists so the helper is total.
fn array_access_name(expr: &Expr) -> String {
    match expr {
        Expr::ArrayAccess { name, .. } => name.clone(),
        _ => String::new(),
    }
}

impl control::flatten::Shapes for ControlShapes<'_, '_> {
    fn matrix_info(&self, expr: &Expr) -> Result<control::flatten::MatrixRef> {
        control_matrix_info(self.flattener, self.loop_vars, expr)
    }
    fn vector_info(&self, expr: &Expr) -> Result<control::flatten::VectorRef> {
        control_vector_info(self.flattener, self.loop_vars, expr)
    }
    fn expand(&self, expr: &Expr) -> Result<Expr> {
        self.flattener.expand_expr(expr, self.loop_vars)
    }
    fn const_index(&self, expr: &Expr) -> Result<i64> {
        self.flattener.const_index(expr, self.loop_vars)
    }
}

impl control::flatten::Shapes for ControlHost<'_, '_> {
    fn matrix_info(&self, expr: &Expr) -> Result<control::flatten::MatrixRef> {
        control_matrix_info(self.flattener, self.loop_vars, expr)
    }
    fn vector_info(&self, expr: &Expr) -> Result<control::flatten::VectorRef> {
        control_vector_info(self.flattener, self.loop_vars, expr)
    }
    fn expand(&self, expr: &Expr) -> Result<Expr> {
        self.flattener.expand_expr(expr, self.loop_vars)
    }
    fn const_index(&self, expr: &Expr) -> Result<i64> {
        self.flattener.const_index(expr, self.loop_vars)
    }
}

impl control::flatten::Host for ControlHost<'_, '_> {
    fn register_shape(&mut self, name: &str, rows: usize, cols: usize) {
        // The Java's three-argument `registerShape`, i.e. no declared
        // dimensionality — every control-systems output goes through that one.
        register_shape(self.shapes, name, rows, cols, None);
    }
    fn emit(&mut self, equation: Equation) -> Result<()> {
        self.flattener.push(equation)
    }
    fn reserve(&self, planned: usize) -> Result<()> {
        self.flattener.reserve(planned)
    }
}

// ---------------------------------------------------------------------------
// Shape registry and bare-name resolution
// ---------------------------------------------------------------------------

fn register_shape(
    shapes: &mut HashMap<String, Shape>,
    name: &str,
    rows: usize,
    cols: usize,
    dims: Option<usize>,
) {
    shapes.insert(name.to_ascii_lowercase(), Shape { rows, cols, dims });
}

/// Rewrites bare references to a registered matrix/vector variable into the
/// explicit `A[1:r,1:c]` form. Port of `EquationParser.resolveShapes` — note
/// it does **not** descend into array accesses, literals, comparisons or
/// `not`, exactly like the Java switch.
fn resolve_shapes(e: &Expr, shapes: &HashMap<String, Shape>) -> Expr {
    match e {
        Expr::Var(name) => match shapes.get(&name.to_ascii_lowercase()) {
            Some(shape) => range_access(name, *shape),
            None => e.clone(),
        },
        Expr::BinOp { op, left, right } => Expr::BinOp {
            op: *op,
            left: Box::new(resolve_shapes(left, shapes)),
            right: Box::new(resolve_shapes(right, shapes)),
        },
        Expr::Neg(operand) => Expr::Neg(Box::new(resolve_shapes(operand, shapes))),
        Expr::Call { function, args } => Expr::Call {
            function: function.clone(),
            args: args.iter().map(|a| resolve_shapes(a, shapes)).collect(),
        },
        _ => e.clone(),
    }
}

/// The explicit slice form of a registered shape. Port of
/// `EquationParser.rangeAccess`: declared dimensionality wins when known, the
/// legacy 1-row/1-column heuristic otherwise.
fn range_access(name: &str, shape: Shape) -> Expr {
    let vector = match shape.dims {
        Some(dims) => dims == 1,
        None => shape.rows == 1 || shape.cols == 1,
    };
    let indices = if vector {
        vec![range_one_to(shape.rows.max(shape.cols))]
    } else {
        vec![range_one_to(shape.rows), range_one_to(shape.cols)]
    };
    Expr::ArrayAccess {
        name: name.to_string(),
        indices,
    }
}

fn range_one_to(n: usize) -> Expr {
    Expr::Range {
        start: Box::new(Expr::num(1.0)),
        end: Box::new(Expr::num(n as f64)),
    }
}

/// If lhs is a bare name, register it as a vector of the given size and return
/// the explicit `v[1:size]` form. Port of `EquationParser.explicitVectorOutput`.
fn explicit_vector_output(lhs: &Expr, size: usize, shapes: &mut HashMap<String, Shape>) -> Expr {
    if let Expr::Var(name) = lhs {
        register_shape(shapes, name, size, 1, None);
        return Expr::ArrayAccess {
            name: name.clone(),
            indices: vec![range_one_to(size)],
        };
    }
    lhs.clone()
}

/// As [`explicit_vector_output`] but for a rows×cols matrix output. Port of
/// `EquationParser.explicitMatrixOutput`.
fn explicit_matrix_output(
    lhs: &Expr,
    rows: usize,
    cols: usize,
    shapes: &mut HashMap<String, Shape>,
) -> Expr {
    if let Expr::Var(name) = lhs {
        register_shape(shapes, name, rows, cols, None);
        return range_access(
            name,
            Shape {
                rows,
                cols,
                dims: None,
            },
        );
    }
    lhs.clone()
}

// ---------------------------------------------------------------------------
// Matrix-expression classification
// ---------------------------------------------------------------------------

/// Functions whose result is a matrix/vector (so an equation using one is a
/// matrix equation). Scalar-valued ones (det, dot, norm) are excluded. Port of
/// `EquationParser.MATRIX_FUNCTIONS`.
const MATRIX_FUNCTIONS: [&str; 16] = [
    "transpose",
    "inverse",
    "inv",
    "solvelinear",
    "axpy",
    "scal",
    "gemv",
    "gemm",
    "ger",
    "copy",
    "zeros",
    "ones",
    "eye",
    "identity",
    "diag",
    "linspace",
];

fn is_matrix_function(function: &str) -> bool {
    let lower = function.to_ascii_lowercase();
    MATRIX_FUNCTIONS.contains(&lower.as_str())
}

/// Port of `EquationParser.isMatrixExpr`.
fn is_matrix_expr(e: &Expr) -> bool {
    match e {
        Expr::ArrayAccess { indices, .. } => indices
            .iter()
            .any(|index| matches!(index, Expr::Range { .. }) || is_matrix_expr(index)),
        Expr::ArrayLiteral(elements) => elements
            .iter()
            .any(|element| matches!(element, Expr::ArrayLiteral(_)) || is_matrix_expr(element)),
        Expr::BinOp { op, left, right } => {
            op.is_element_wise()
                || (matches!(op, BinOp::Mul | BinOp::Add | BinOp::Sub | BinOp::LeftDiv)
                    && (is_matrix_expr(left) || is_matrix_expr(right)))
        }
        Expr::Call { function, .. } => is_matrix_function(function),
        _ => false,
    }
}

/// Port of `EquationParser.containsElementwiseOp`.
fn contains_elementwise_op(e: &Expr) -> bool {
    match e {
        Expr::BinOp { op, left, right } => {
            op.is_element_wise() || contains_elementwise_op(left) || contains_elementwise_op(right)
        }
        Expr::Neg(operand) => contains_elementwise_op(operand),
        Expr::Call { args, .. } => args.iter().any(contains_elementwise_op),
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Free matrix helpers
// ---------------------------------------------------------------------------

fn is_1x1(m: &Matrix) -> bool {
    m.len() == 1 && m[0].len() == 1
}

/// `scalar op mat` (`scalar_on_left`) or `mat op scalar`, broadcasting the
/// scalar. Port of `EquationParser.broadcastScalar`.
fn broadcast_scalar(op: BinOp, scalar: &Expr, mat: &Matrix, scalar_on_left: bool) -> Matrix {
    mat.iter()
        .map(|row| {
            row.iter()
                .map(|element| {
                    if scalar_on_left {
                        Expr::bin(op, scalar.clone(), element.clone())
                    } else {
                        Expr::bin(op, element.clone(), scalar.clone())
                    }
                })
                .collect()
        })
        .collect()
}

/// Port of `EquationParser.matMul` (left-folded sums, so residual float
/// evaluation order matches the Java engine).
fn mat_mul(l_mat: &Matrix, r_mat: &Matrix) -> Matrix {
    let rows = l_mat.len();
    let cols = r_mat[0].len();
    let inner = l_mat[0].len();
    (0..rows)
        .map(|i| {
            (0..cols)
                .map(|j| {
                    let mut term = Expr::bin(BinOp::Mul, l_mat[i][0].clone(), r_mat[0][j].clone());
                    for k in 1..inner {
                        term = Expr::bin(
                            BinOp::Add,
                            term,
                            Expr::bin(BinOp::Mul, l_mat[i][k].clone(), r_mat[k][j].clone()),
                        );
                    }
                    term
                })
                .collect()
        })
        .collect()
}

/// Broadcasts a scalar RHS to the LHS shape, transposes a flipped row/column
/// vector, or errors on a genuine dimension mismatch. Port of
/// `EquationParser.conformRhsToLhs`.
fn conform_rhs_to_lhs(lhs_mat: &Matrix, rhs_mat: Matrix) -> Result<Matrix> {
    if is_1x1(&rhs_mat) {
        let scalar = rhs_mat[0][0].clone();
        return Ok(lhs_mat
            .iter()
            .map(|row| row.iter().map(|_| scalar.clone()).collect())
            .collect());
    }
    if lhs_mat.len() == rhs_mat.len() && lhs_mat[0].len() == rhs_mat[0].len() {
        return Ok(rhs_mat);
    }
    if lhs_mat.len() == rhs_mat[0].len()
        && lhs_mat[0].len() == rhs_mat.len()
        && (lhs_mat.len() == 1 || lhs_mat[0].len() == 1)
    {
        return Ok((0..lhs_mat.len())
            .map(|i| {
                (0..lhs_mat[0].len())
                    .map(|j| rhs_mat[j][i].clone())
                    .collect()
            })
            .collect());
    }
    Err(parse_err(format!(
        "Matrix assignment dimension mismatch: LHS is {}x{}, but RHS is {}x{}",
        lhs_mat.len(),
        lhs_mat[0].len(),
        rhs_mat.len(),
        rhs_mat[0].len()
    )))
}

/// Closed-form cofactor expansion, only for n <= [`DET_CLOSED_FORM_MAX`].
/// Port of `EquationParser.expandDeterminant` / `subMatrix`.
fn expand_determinant(mat: &Matrix) -> Expr {
    let n = mat.len();
    if n == 1 {
        return mat[0][0].clone();
    }
    if n == 2 {
        return Expr::bin(
            BinOp::Sub,
            Expr::bin(BinOp::Mul, mat[0][0].clone(), mat[1][1].clone()),
            Expr::bin(BinOp::Mul, mat[0][1].clone(), mat[1][0].clone()),
        );
    }
    let mut sum: Option<Expr> = None;
    for j in 0..n {
        let sub: Matrix = (1..n)
            .map(|r| {
                (0..n)
                    .filter(|&c| c != j)
                    .map(|c| mat[r][c].clone())
                    .collect()
            })
            .collect();
        let sub_det = expand_determinant(&sub);
        let mut cofactor = Expr::bin(BinOp::Mul, mat[0][j].clone(), sub_det);
        if j % 2 == 1 {
            cofactor = Expr::Neg(Box::new(cofactor));
        }
        sum = Some(match sum {
            None => cofactor,
            Some(acc) => Expr::bin(BinOp::Add, acc, cofactor),
        });
    }
    sum.expect("n >= 1")
}

fn arg_at<'e>(args: &'e [Expr], index: usize, function: &str) -> Result<&'e Expr> {
    args.get(index).ok_or_else(|| {
        parse_err(format!(
            "{function} is missing argument {} (got {} argument(s))",
            index + 1,
            args.len()
        ))
    })
}

fn elementwise_dims(
    op: BinOp,
    l_mat: &Matrix,
    r_mat: &Matrix,
    l_scalar: bool,
    r_scalar: bool,
) -> Result<(usize, usize)> {
    if l_scalar {
        return Ok((r_mat.len(), r_mat[0].len()));
    }
    if r_scalar {
        return Ok((l_mat.len(), l_mat[0].len()));
    }
    if l_mat.len() != r_mat.len() || l_mat[0].len() != r_mat[0].len() {
        return Err(parse_err(format!(
            "Matrix dimensions must agree for element-wise '{}': {}x{} vs {}x{}",
            op.as_str(),
            l_mat.len(),
            l_mat[0].len(),
            r_mat.len(),
            r_mat[0].len()
        )));
    }
    Ok((l_mat.len(), l_mat[0].len()))
}

/// A resolved matrix's element expressions flattened row-major — the argument
/// packing every kernel synthetic expects. Port of
/// `EquationParser.matrixEntries`.
fn matrix_entries(m: &MatrixInfo) -> Vec<Expr> {
    let mut entries = Vec::with_capacity(m.rows * m.cols);
    for row in &m.elements {
        entries.extend(row.iter().cloned());
    }
    entries
}

fn set_vec(outputs: &mut [Expr], index: usize, size: usize) {
    if index < outputs.len() && size > 0 {
        if let Expr::Var(name) = &outputs[index] {
            outputs[index] = Expr::ArrayAccess {
                name: name.clone(),
                indices: vec![range_one_to(size)],
            };
        }
    }
}

fn set_mat(outputs: &mut [Expr], index: usize, rows: usize, cols: usize) {
    if index < outputs.len() && rows > 0 && cols > 0 {
        if let Expr::Var(name) = &outputs[index] {
            outputs[index] = Expr::ArrayAccess {
                name: name.clone(),
                indices: vec![range_one_to(rows), range_one_to(cols)],
            };
        }
    }
}

/// CALL intrinsic names the Java `flattenCallProc` dispatches that are not in
/// the matrix-expansion scope. Refused by name so a document using one fails
/// loudly instead of being reported as an unknown procedure.
///
/// What is left is the Euler decompose/rotate pair. The dense linear-algebra,
/// signal and statistics intrinsics (the Java `LIN_ALG_SIGNAL_STATS_CALLS`
/// set) are wired above, Phase 9 moved the whole control-systems suite out
/// of here into [`control::flatten`], and the eigen pair left when ledger
/// item 34 closed — all must stay out of this list, or `flatten_call_proc`
/// short-circuits before their flatteners.
/// `procedures::EXPANDED_CALL_TARGETS` is the matching stage-2 allowance.
const UNPORTED_CALL_INTRINSICS: [&str; 2] = ["eulerrotate", "eulerdecompose"];

/// The number of outputs a fixed-shape CALL intrinsic produces, used to pad
/// trailing omission. `-1` for user-defined calls and for intrinsics whose
/// output count must be stated explicitly. Port of
/// `EquationParser.expectedOutputCount` (kept in full so padding behaviour is
/// already Java-exact when the remaining intrinsics port).
fn expected_output_count(def_name: &str, inputs: &[Expr]) -> i32 {
    match def_name {
        "eigenvalues" | "eulerrotate" => 1,
        "eigen" | "ludecompose" | "qr" | "fft" | "ifft" | "welch" => 2,
        "eulerdecompose" | "ss2ss" | "svd" => 3,
        "ss2tf" | "ss2tfij" | "zp2tf" | "c2d" | "d2c" | "pade" | "pole" | "zero" | "bode"
        | "nyquist" | "nichols" => 2,
        "tf2ss" | "margin" | "stepinfo" => 4,
        "tf2zp" => 5,
        "series" | "parallel" | "feedback" => {
            if inputs.len() >= 8 {
                4
            } else {
                2
            }
        }
        "rlocus" | "errorconst" | "pidtune" | "balreal" | "linfit" => 3,
        _ => -1,
    }
}

// ---------------------------------------------------------------------------
// The `range` intrinsic (rangeAssign materialisation)
// ---------------------------------------------------------------------------

/// Rewrite `x = range(start, middle, stop, '<spacing>')` — the
/// [`crate::parser::toplevel`] lowering of `x = 0:10:100 [| Log]` — into the
/// explicit `x[1:N] = [v1, …]` assignment the Java `buildRangeAssign`
/// produces, so the ordinary matrix machinery materialises the elements.
fn desugar_range_intrinsic(lhs: &Expr, rhs: &Expr) -> Result<Option<(Expr, Expr)>> {
    let Expr::Var(name) = lhs else {
        return Ok(None);
    };
    let Expr::Call { function, args } = rhs else {
        return Ok(None);
    };
    if function != RANGE_INTRINSIC || args.len() != 4 {
        return Ok(None);
    }
    let (start, middle, stop, spacing) = match (&args[0], &args[1], &args[2], &args[3]) {
        (
            Expr::Num { value: start, .. },
            Expr::Num { value: middle, .. },
            Expr::Num { value: stop, .. },
            Expr::Str(spacing),
        ) => (*start, *middle, *stop, spacing.as_str()),
        _ => return Ok(None),
    };
    let values = match spacing {
        "linear" => linear_range(name, start, middle, stop)?,
        "log" => log_range(name, start, middle, stop)?,
        _ => return Ok(None), // not the lowering's shape; leave as written
    };
    let new_lhs = Expr::ArrayAccess {
        name: name.clone(),
        indices: vec![range_one_to(values.len())],
    };
    let new_rhs = Expr::ArrayLiteral(values.into_iter().map(Expr::num).collect());
    Ok(Some((new_lhs, new_rhs)))
}

/// Port of `AstBuilder.linearRange` (values; the same validation already ran
/// in `toplevel::linear_range_count` at parse time and is repeated with the
/// identical formulas so hand-built ASTs behave the same).
fn linear_range(var: &str, start: f64, step: f64, stop: f64) -> Result<Vec<f64>> {
    if step == 0.0 {
        return Err(parse_err(format!("Range step is zero in {var} = ...")));
    }
    if (stop - start) * step < 0.0 {
        return Err(parse_err(format!(
            "Range step points the wrong way in {var} = {start}:{step}:{stop}."
        )));
    }
    // Screened as a float first (see `toplevel::linear_range_count`): the Java
    // long cast saturates and wraps on `inf`, which must not be reproduced.
    let raw = libm::floor((stop - start) / step + 1e-9);
    if !raw.is_finite() || raw > MAX_RANGE_ELEMENTS as f64 {
        return Err(parse_err(format!(
            "Range {var} = ... would generate more than {MAX_RANGE_ELEMENTS} \
             elements. Use a larger step."
        )));
    }
    let count = raw as i64 + 1;
    if count > MAX_RANGE_ELEMENTS {
        return Err(parse_err(format!(
            "Range {var} = ... would generate {count} elements (max {MAX_RANGE_ELEMENTS}). \
             Use a larger step."
        )));
    }
    Ok((0..count).map(|k| start + (k as f64) * step).collect())
}

/// Port of `AstBuilder.logRange`: `middle` is the point count; spacing is
/// geometric with the final element pinned to `stop` exactly.
fn log_range(var: &str, start: f64, count_raw: f64, stop: f64) -> Result<Vec<f64>> {
    if start <= 0.0 || stop <= 0.0 {
        return Err(parse_err(format!(
            "A logarithmic range needs positive bounds in {var} = ..."
        )));
    }
    // `libm::round`, matching `toplevel::log_range_count`'s count exactly.
    let count = libm::round(count_raw) as i64;
    if count < 2 {
        return Err(parse_err(format!(
            "A logarithmic range needs a point count of at least 2 in {var} = ..."
        )));
    }
    if count > MAX_RANGE_ELEMENTS {
        return Err(parse_err(format!(
            "Range {var} = ... would generate {count} elements (max {MAX_RANGE_ELEMENTS})."
        )));
    }
    let ratio = libm::pow(stop / start, 1.0 / (count - 1) as f64);
    Ok((0..count)
        .map(|k| {
            if k == count - 1 {
                stop
            } else {
                start * libm::pow(ratio, k as f64)
            }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_document;

    fn expand(source: &str) -> Vec<Equation> {
        expand_document(&parse_document(source).expect("parse")).expect("expand")
    }

    fn expand_err(source: &str) -> String {
        match expand_document(&parse_document(source).expect("parse")) {
            Ok(eqs) => panic!(
                "expected expansion of {source:?} to fail, got {} eqs",
                eqs.len()
            ),
            Err(err) => err.to_string(),
        }
    }

    fn var(name: &str) -> Expr {
        Expr::Var(name.into())
    }
    fn num(value: f64) -> Expr {
        Expr::num(value)
    }
    fn mul(l: Expr, r: Expr) -> Expr {
        Expr::bin(BinOp::Mul, l, r)
    }
    fn add(l: Expr, r: Expr) -> Expr {
        Expr::bin(BinOp::Add, l, r)
    }
    fn sub(l: Expr, r: Expr) -> Expr {
        Expr::bin(BinOp::Sub, l, r)
    }

    /// `(lhs, rhs)` pairs, dropping source text.
    fn sides(eqs: &[Equation]) -> Vec<(Expr, Expr)> {
        eqs.iter()
            .map(|eq| (eq.lhs.clone(), eq.rhs.clone()))
            .collect()
    }

    /// `name → value` over the `Var = Num` equations.
    fn literal_values(eqs: &[Equation]) -> HashMap<String, f64> {
        eqs.iter()
            .filter_map(|eq| match (&eq.lhs, &eq.rhs) {
                (Expr::Var(name), Expr::Num { value, .. }) => Some((name.clone(), *value)),
                _ => None,
            })
            .collect()
    }

    /// Assert `solution` (plus every literal the expansion emitted) zeroes the
    /// residual of every expanded equation — i.e. the emitted system has
    /// exactly the oracle's solution.
    fn assert_satisfied(eqs: &[Equation], solution: &[(&str, f64)]) {
        let mut scope: Scope = literal_values(eqs).into_iter().collect();
        for (name, value) in solution {
            scope.insert((*name).to_string(), *value);
        }
        for eq in eqs {
            let l = eval::eval(&eq.lhs, &scope)
                .unwrap_or_else(|e| panic!("lhs of `{}` did not evaluate: {e:?}", eq.source_text));
            let r = eval::eval(&eq.rhs, &scope)
                .unwrap_or_else(|e| panic!("rhs of `{}` did not evaluate: {e:?}", eq.source_text));
            assert!(
                (l - r).abs() < 1e-9,
                "`{}` not satisfied: {l} vs {r}",
                eq.source_text
            );
        }
    }

    // ── scalar pass-through ─────────────────────────────────────────────────

    #[test]
    fn scalar_documents_pass_through_byte_identical() {
        let source = "x = 2\ny = x^2 + sin(x)\nz + y = min(3, x)\nGUESS z = 1";
        let doc = parse_document(source).unwrap();
        let expanded = expand_document(&doc).unwrap();
        let original: Vec<Equation> = doc.equations().into_iter().cloned().collect();
        assert_eq!(expanded, original);
    }

    #[test]
    fn constant_indexed_element_write_flattens_to_a_scalar_variable() {
        let eqs = expand("A[1,2] = 5");
        assert_eq!(sides(&eqs), vec![(var("a[1,2]"), num(5.0))]);
        assert_eq!(eqs[0].source_text, "A[1,2] = 5");
    }

    // ── literals (the solvesMatlab* oracle set) ─────────────────────────────

    #[test]
    fn bare_matrix_literal_expands_to_element_equations() {
        let eqs = expand("A = [2 0; 0 4]");
        assert_eq!(
            sides(&eqs),
            vec![
                (var("a[1,1]"), num(2.0)),
                (var("a[1,2]"), num(0.0)),
                (var("a[2,1]"), num(0.0)),
                (var("a[2,2]"), num(4.0)),
            ]
        );
    }

    #[test]
    fn column_vector_literal_uses_flat_indices() {
        let eqs = expand("b = [6; 8]");
        assert_eq!(
            sides(&eqs),
            vec![(var("b[1]"), num(6.0)), (var("b[2]"), num(8.0))]
        );
    }

    #[test]
    fn row_vector_literal_uses_flat_indices() {
        let eqs = expand("v = [1, 2, 3]");
        assert_eq!(
            sides(&eqs),
            vec![
                (var("v[1]"), num(1.0)),
                (var("v[2]"), num(2.0)),
                (var("v[3]"), num(3.0)),
            ]
        );
    }

    #[test]
    fn solves_matlab_style_bare_name_matrix() {
        // EquationSystemSolverTest.solvesMatlabStyleBareNameMatrix
        let eqs = expand("A = [2 0; 0 4]\nb = [6; 8]\nx = SolveLinear(A, b)");
        assert_eq!(eqs.len(), 8);
        // Row equations write x directly — no temp.
        assert_eq!(
            (&eqs[6].lhs, &eqs[6].rhs),
            (
                &add(
                    mul(var("a[1,1]"), var("x[1]")),
                    mul(var("a[1,2]"), var("x[2]"))
                ),
                &var("b[1]")
            )
        );
        assert_satisfied(&eqs, &[("x[1]", 3.0), ("x[2]", 2.0)]);
    }

    #[test]
    fn solves_matlab_style_bare_name_inverse() {
        // EquationSystemSolverTest.solvesMatlabStyleBareNameInverseAndMatVec (1st half)
        let eqs = expand("A = [4 0; 0 5]\nC = Inverse(A)");
        assert_eq!(eqs.len(), 8);
        // (A · C)[1,1] = 1 — the output is written directly, no temp.
        assert_eq!(
            (&eqs[4].lhs, &eqs[4].rhs),
            (
                &add(
                    mul(var("a[1,1]"), var("c[1,1]")),
                    mul(var("a[1,2]"), var("c[2,1]"))
                ),
                &num(1.0)
            )
        );
        assert!(eqs
            .iter()
            .all(|eq| !eq.variables().iter().any(|v| is_internal_temp(v))));
        assert_satisfied(
            &eqs,
            &[
                ("c[1,1]", 0.25),
                ("c[1,2]", 0.0),
                ("c[2,1]", 0.0),
                ("c[2,2]", 0.2),
            ],
        );
    }

    #[test]
    fn solves_matlab_style_bare_name_matvec() {
        // EquationSystemSolverTest.solvesMatlabStyleBareNameInverseAndMatVec (2nd half)
        let eqs = expand("A = [1 2; 3 4]\nx = [5; 6]\ny = A * x");
        assert_eq!(eqs.len(), 8);
        assert_eq!(
            (&eqs[6].lhs, &eqs[6].rhs),
            (
                &var("y[1]"),
                &add(
                    mul(var("a[1,1]"), var("x[1]")),
                    mul(var("a[1,2]"), var("x[2]"))
                )
            )
        );
        assert_satisfied(&eqs, &[("y[1]", 17.0), ("y[2]", 39.0)]);
    }

    #[test]
    fn solves_matlab_matrix_generators() {
        // EquationSystemSolverTest.solvesMatlabMatrixGenerators
        let eqs = expand(
            "I = eye(3)\nZ = zeros(2,2)\nu = ones(3,1)\nD = diag([2; 5; 7])\ng = linspace(0, 10, 5)",
        );
        assert_eq!(eqs.len(), 9 + 4 + 3 + 9 + 5);
        let values = literal_values(&eqs);
        assert_eq!(values["i[1,1]"], 1.0);
        assert_eq!(values["i[1,2]"], 0.0);
        assert_eq!(values["z[2,1]"], 0.0);
        assert_eq!(values["u[3]"], 1.0);
        assert_eq!(values["d[2,2]"], 5.0);
        assert_eq!(values["d[1,2]"], 0.0);
        assert_eq!(values["g[2]"], 2.5); // 0, 2.5, 5, 7.5, 10
        assert_eq!(values["g[5]"], 10.0);
    }

    #[test]
    fn solves_matlab_inv_det_aliases() {
        // EquationSystemSolverTest.solvesMatlabInvDetAliases
        let eqs = expand("A = [4 0; 0 5]\nC = inv(A)\nd = det(A)");
        assert_eq!(eqs.len(), 9);
        // Closed-form 2x2 determinant.
        assert_eq!(
            (&eqs[8].lhs, &eqs[8].rhs),
            (
                &var("d"),
                &sub(
                    mul(var("a[1,1]"), var("a[2,2]")),
                    mul(var("a[1,2]"), var("a[2,1]"))
                )
            )
        );
        assert_satisfied(
            &eqs,
            &[
                ("c[1,1]", 0.25),
                ("c[1,2]", 0.0),
                ("c[2,1]", 0.0),
                ("c[2,2]", 0.2),
                ("d", 20.0),
            ],
        );
    }

    // ── temps for matrix functions inside larger expressions ────────────────

    #[test]
    fn inverse_inside_expression_materializes_prefixed_temp() {
        let eqs = expand("A = [4 0; 0 5]\nb = [8; 10]\nx = Inverse(A) * b");
        assert_eq!(eqs.len(), 12);
        // 6 literals first, so the temp is named after equation count 6.
        assert_eq!(eqs[6].source_text, "Matrix inverse definition");
        let temp_vars: Vec<String> = eqs[6]
            .variables()
            .into_iter()
            .filter(|v| is_internal_temp(v))
            .collect();
        assert_eq!(
            temp_vars,
            vec!["inverse_temp_6[1,1]", "inverse_temp_6[2,1]"]
        );
        assert_satisfied(
            &eqs,
            &[
                ("inverse_temp_6[1,1]", 0.25),
                ("inverse_temp_6[1,2]", 0.0),
                ("inverse_temp_6[2,1]", 0.0),
                ("inverse_temp_6[2,2]", 0.2),
                ("x[1]", 2.0),
                ("x[2]", 2.0),
            ],
        );
    }

    #[test]
    fn backslash_materializes_prefixed_temp() {
        let eqs = expand("A = [2 0; 0 4]\nb = [6; 8]\nx = A \\ b");
        // 6 literals, 2 "Backslash solve" rows (temp named after count 6),
        // then x bound to the temp.
        assert_eq!(eqs.len(), 10);
        assert_eq!(eqs[6].source_text, "Backslash solve");
        assert!(eqs[6]
            .variables()
            .iter()
            .any(|v| v == "backslash_temp_6[1]"));
        assert_eq!(eqs[8].lhs, var("x[1]"));
        assert_eq!(eqs[8].rhs, var("backslash_temp_6[1]"));
        assert_satisfied(
            &eqs,
            &[
                ("backslash_temp_6[1]", 3.0),
                ("backslash_temp_6[2]", 2.0),
                ("x[1]", 3.0),
                ("x[2]", 2.0),
            ],
        );
    }

    #[test]
    fn nested_solvelinear_materializes_prefixed_temp() {
        let eqs = expand("A = [2 0; 0 4]\nb = [6; 8]\ny = 2 * SolveLinear(A, b)");
        // 6 literals, 2 "SolveLinear solve" rows, 2 bindings of y.
        assert_eq!(eqs.len(), 10);
        assert_eq!(eqs[6].source_text, "SolveLinear solve");
        assert_eq!(eqs[8].rhs, mul(num(2.0), var("solvelinear_temp_6[1]")));
        assert_satisfied(
            &eqs,
            &[
                ("solvelinear_temp_6[1]", 3.0),
                ("solvelinear_temp_6[2]", 2.0),
                ("y[1]", 6.0),
                ("y[2]", 4.0),
            ],
        );
    }

    // ── transpose, dot, norms, cross ────────────────────────────────────────

    #[test]
    fn postfix_transpose_writes_the_output_directly() {
        let eqs = expand("A = [1 2; 3 4]\nB = A'");
        assert_eq!(eqs.len(), 8);
        assert_eq!(
            sides(&eqs[4..]),
            vec![
                (var("b[1,1]"), var("a[1,1]")),
                (var("b[1,2]"), var("a[2,1]")),
                (var("b[2,1]"), var("a[1,2]")),
                (var("b[2,2]"), var("a[2,2]")),
            ]
        );
    }

    #[test]
    fn dot_product_expands_to_a_sum_of_products() {
        let eqs = expand("u = [1; 2; 3]\nv = [4; 5; 6]\nd = dot(u, v)");
        assert_eq!(
            (&eqs[6].lhs, &eqs[6].rhs),
            (
                &var("d"),
                &add(
                    add(mul(var("u[1]"), var("v[1]")), mul(var("u[2]"), var("v[2]"))),
                    mul(var("u[3]"), var("v[3]"))
                )
            )
        );
        assert_satisfied(&eqs, &[("d", 32.0)]);
    }

    #[test]
    fn norm_trace_fronorm_asum_expand_to_scalar_equations() {
        let eqs = expand(
            "A = [3 0; 0 4]\nv = [3; 4]\nn1 = norm(v)\nt = trace(A)\nf = MatrixNorm(A)\ns = asum(v)",
        );
        assert_satisfied(&eqs, &[("n1", 5.0), ("t", 7.0), ("f", 5.0), ("s", 7.0)]);
    }

    #[test]
    fn cross_product_expands_componentwise() {
        let eqs = expand("u = [1; 0; 0]\nv = [0; 1; 0]\nw = cross(u, v)");
        assert_eq!(eqs.len(), 9);
        assert_satisfied(&eqs, &[("w[1]", 0.0), ("w[2]", 0.0), ("w[3]", 1.0)]);
    }

    #[test]
    fn det_beyond_3x3_emits_a_runtime_lu_intrinsic() {
        let eqs = expand("A = eye(4)\nd = det(A)");
        let last = eqs.last().unwrap();
        assert_eq!(last.lhs, var("d"));
        match &last.rhs {
            Expr::Call { function, args } => {
                assert_eq!(function, "det$4");
                assert_eq!(args.len(), 16);
                assert_eq!(args[0], var("a[1,1]"));
                assert_eq!(args[15], var("a[4,4]"));
            }
            other => panic!("expected det$4 call, got {other:?}"),
        }
    }

    // ── element-wise operators and broadcasting ─────────────────────────────

    #[test]
    fn elementwise_operators_apply_per_element() {
        let eqs = expand(
            "A = [1 2; 3 4]\nB = [5 6; 7 8]\nC = A .* B\nD = A ./ B\nE = 2 .* A\nF = A .\\ B\nG = A .^ 2",
        );
        assert_satisfied(
            &eqs,
            &[
                ("c[1,1]", 5.0),
                ("c[1,2]", 12.0),
                ("c[2,1]", 21.0),
                ("c[2,2]", 32.0),
                ("d[1,1]", 0.2),
                ("d[1,2]", 2.0 / 6.0),
                ("d[2,1]", 3.0 / 7.0),
                ("d[2,2]", 0.5),
                ("e[1,1]", 2.0),
                ("e[2,2]", 8.0),
                ("e[1,2]", 4.0),
                ("e[2,1]", 6.0),
                ("f[1,1]", 5.0),
                ("f[1,2]", 3.0),
                ("f[2,1]", 7.0 / 3.0),
                ("f[2,2]", 2.0),
                ("g[1,1]", 1.0),
                ("g[1,2]", 4.0),
                ("g[2,1]", 9.0),
                ("g[2,2]", 16.0),
            ],
        );
    }

    #[test]
    fn matrix_add_sub_and_scalar_broadcast() {
        let eqs = expand("A = [1 2; 3 4]\nB = A + 1\nC = A - [1 1; 1 1]\nD = 3 * A");
        assert_satisfied(
            &eqs,
            &[
                ("b[1,1]", 2.0),
                ("b[1,2]", 3.0),
                ("b[2,1]", 4.0),
                ("b[2,2]", 5.0),
                ("c[1,1]", 0.0),
                ("c[1,2]", 1.0),
                ("c[2,1]", 2.0),
                ("c[2,2]", 3.0),
                ("d[1,1]", 3.0),
                ("d[1,2]", 6.0),
                ("d[2,1]", 9.0),
                ("d[2,2]", 12.0),
            ],
        );
    }

    // ── explicit slices ─────────────────────────────────────────────────────

    #[test]
    fn explicit_slice_conforms_a_flipped_row_vector() {
        // EquationSystemSolverTest.unitOnMatrixLiteralAndRangeAssign (2nd half)
        let eqs = expand("c[1:3] = [2, 3, 4]");
        let values = literal_values(&eqs);
        assert_eq!(values["c[1]"], 2.0);
        assert_eq!(values["c[3]"], 4.0);
    }

    #[test]
    fn explicit_slice_registers_the_shape_for_later_bare_use() {
        let eqs = expand("c[1:2] = [1, 2]\ns = dot(c, c)");
        assert_satisfied(&eqs, &[("s", 5.0)]);
    }

    #[test]
    fn descending_slice_keeps_the_written_direction() {
        let eqs = expand("v[3:1] = [1, 2, 3]");
        assert_eq!(
            sides(&eqs),
            vec![
                (var("v[3]"), num(1.0)),
                (var("v[2]"), num(2.0)),
                (var("v[1]"), num(3.0)),
            ]
        );
    }

    #[test]
    fn scalar_rhs_broadcasts_over_a_slice() {
        let eqs = expand("x[1:3] = 7");
        assert_eq!(
            sides(&eqs),
            vec![
                (var("x[1]"), num(7.0)),
                (var("x[2]"), num(7.0)),
                (var("x[3]"), num(7.0)),
            ]
        );
    }

    #[test]
    fn diag_of_a_matrix_extracts_the_diagonal() {
        let eqs = expand("A = [1 2; 3 4]\nd = diag(A)");
        assert_eq!(
            sides(&eqs[4..]),
            vec![(var("d[1]"), var("a[1,1]")), (var("d[2]"), var("a[2,2]"))]
        );
    }

    // ── rangeAssign ─────────────────────────────────────────────────────────

    #[test]
    fn range_assign_expands_to_element_equations() {
        let eqs = expand("speed = 0:10:100");
        assert_eq!(eqs.len(), 11);
        let values = literal_values(&eqs);
        assert_eq!(values["speed[1]"], 0.0);
        assert_eq!(values["speed[2]"], 10.0);
        assert_eq!(values["speed[11]"], 100.0);
        assert_eq!(eqs[0].source_text, "speed = 0:10:100");
    }

    #[test]
    fn range_assign_two_number_form_steps_by_one() {
        let eqs = expand("i = 1:5");
        assert_eq!(eqs.len(), 5);
        assert_eq!(literal_values(&eqs)["i[4]"], 4.0);
    }

    #[test]
    fn range_assign_log_spaces_geometrically_with_exact_endpoint() {
        let eqs = expand("f = 1:5:10000 | Log");
        assert_eq!(eqs.len(), 5);
        let values = literal_values(&eqs);
        assert!((values["f[1]"] - 1.0).abs() < 1e-12);
        assert!((values["f[2]"] - 10.0).abs() < 1e-9);
        assert!((values["f[3]"] - 100.0).abs() < 1e-8);
        assert_eq!(values["f[5]"], 10000.0); // pinned exactly
    }

    #[test]
    fn range_assign_registers_the_shape_for_later_bare_use() {
        let eqs = expand("x = 0:1:2\ny = dot(x, x)");
        assert_satisfied(&eqs, &[("y", 5.0)]);
    }

    // ── FOR loops ───────────────────────────────────────────────────────────

    #[test]
    fn for_loop_substitutes_the_loop_variable() {
        let eqs = expand("FOR i = 1 TO 3\n  x[i] = i^2\nEND");
        assert_eq!(
            sides(&eqs),
            vec![
                (var("x[1]"), Expr::bin(BinOp::Pow, num(1.0), num(2.0))),
                (var("x[2]"), Expr::bin(BinOp::Pow, num(2.0), num(2.0))),
                (var("x[3]"), Expr::bin(BinOp::Pow, num(3.0), num(2.0))),
            ]
        );
    }

    #[test]
    fn for_loop_iterates_descending_bounds() {
        let eqs = expand("FOR i = 3 TO 1\n  x[i] = i\nEND");
        assert_eq!(
            sides(&eqs),
            vec![
                (var("x[3]"), num(3.0)),
                (var("x[2]"), num(2.0)),
                (var("x[1]"), num(1.0)),
            ]
        );
    }

    #[test]
    fn for_loop_span_is_bounded() {
        let message = expand_err("FOR i = 1 TO 2000000\n  x[i] = i\nEND");
        assert!(message.contains("FOR loop range is too large"), "{message}");
    }

    #[test]
    fn equation_budget_is_enforced() {
        let message = expand_err("Z = zeros(200, 200)");
        assert!(
            message.contains("Too many equations generated"),
            "{message}"
        );
    }

    // ── shape inference and index expansion ─────────────────────────────────

    #[test]
    fn element_by_element_writes_infer_the_shape() {
        let eqs = expand("A[1,1] = 4\nA[1,2] = 0\nA[2,1] = 0\nA[2,2] = 5\nd = det(A)");
        assert_eq!(eqs.len(), 5);
        assert_satisfied(&eqs, &[("d", 20.0)]);
    }

    #[test]
    fn a_scalar_of_the_same_name_blocks_shape_inference() {
        // `k` is assigned as a scalar, so `K[1,1]` must not make `k` a matrix.
        let eqs = expand("k = 1000\nK[1,1] = 1\nd = k * 2");
        assert_eq!(eqs.len(), 3);
        assert_eq!(eqs[2].rhs, mul(var("k"), num(2.0)));
    }

    #[test]
    fn slice_arguments_of_scalar_calls_flatten_to_elements() {
        let eqs = expand("v = [4; 9]\ns = sqrt(v)");
        // `sqrt` is not a matrix function: the slice splices into scalar args,
        // exactly like the Java expandExpr.
        assert_eq!(
            eqs[2].rhs,
            Expr::Call {
                function: "sqrt".into(),
                args: vec![var("v[1]"), var("v[2]")],
            }
        );
    }

    #[test]
    fn matrix_in_an_unsupported_operator_position_is_refused() {
        let message = expand_err("v = [1; 2]\ny = v ^ 2");
        assert!(
            message.contains("only allowed on the LHS of assignments"),
            "{message}"
        );
    }

    // ── dimension errors ────────────────────────────────────────────────────

    #[test]
    fn inner_dimension_mismatch_is_reported() {
        let message = expand_err("A = [1 2; 3 4]\nb = [1; 2; 3]\ny = A * b");
        assert!(
            message.contains("Inner matrix dimensions must agree: 2x2 vs 3x1"),
            "{message}"
        );
    }

    #[test]
    fn assignment_dimension_mismatch_is_reported() {
        let message = expand_err("c[1:2] = [1, 2, 3]");
        assert!(
            message.contains("Matrix assignment dimension mismatch: LHS is 2x1, but RHS is 1x3"),
            "{message}"
        );
    }

    #[test]
    fn ragged_matrix_literal_is_reported() {
        let message = expand_err("A = [1 2; 3]");
        assert!(
            message.contains("Matrix literal rows must have compatible column dimensions."),
            "{message}"
        );
    }

    #[test]
    fn generator_dimensions_must_be_positive() {
        let message = expand_err("Z = zeros(0, 2)");
        assert!(
            message.contains("zeros dimensions must be >= 1"),
            "{message}"
        );
    }

    #[test]
    fn elementwise_dimension_mismatch_is_reported() {
        let message = expand_err("A = [1 2; 3 4]\nB = [1; 2]\nC = A .* B");
        assert!(
            message.contains("Matrix dimensions must agree for element-wise '.*'"),
            "{message}"
        );
    }

    // ── constants feed matrix sizes ─────────────────────────────────────────

    #[test]
    fn extracted_constants_size_generators() {
        let eqs = expand("n = 3\nI = eye(n)");
        assert_eq!(eqs.len(), 10); // n = 3 passes through + 9 elements
        assert_eq!(literal_values(&eqs)["i[3,3]"], 1.0);
    }

    // ── CALL LUDecompose ────────────────────────────────────────────────────

    #[test]
    fn lu_decompose_call_expands_to_pinned_triangular_equations() {
        let eqs = expand("A = [4 3; 6 3]\nCALL LUDecompose(A[1:2,1:2] : L[1:2,1:2], U[1:2,1:2])");
        // 4 literals + per (i,j): triangular pins (l[1,2]=0, l[1,1]=1, l[2,2]=1,
        // u[2,1]=0) and 4 product equations.
        assert_eq!(eqs.len(), 4 + 4 + 4);
        assert_satisfied(
            &eqs,
            &[
                ("l[1,1]", 1.0),
                ("l[1,2]", 0.0),
                ("l[2,1]", 1.5),
                ("l[2,2]", 1.0),
                ("u[1,1]", 4.0),
                ("u[1,2]", 3.0),
                ("u[2,1]", 0.0),
                ("u[2,2]", -1.5),
            ],
        );
    }

    #[test]
    fn lu_decompose_multi_assign_auto_sizes_bare_outputs() {
        let eqs = expand("A = [4 3; 6 3]\n[L, U] = LUDecompose(A)");
        assert_eq!(eqs.len(), 12);
        assert_satisfied(
            &eqs,
            &[
                ("l[1,1]", 1.0),
                ("l[1,2]", 0.0),
                ("l[2,1]", 1.5),
                ("l[2,2]", 1.0),
                ("u[1,1]", 4.0),
                ("u[1,2]", 3.0),
                ("u[2,1]", 0.0),
                ("u[2,2]", -1.5),
            ],
        );
    }

    #[test]
    fn lu_decompose_pads_an_omitted_trailing_output_with_a_sink() {
        let eqs = expand("A = [4 3; 6 3]\n[L] = LUDecompose(A)");
        assert_eq!(eqs.len(), 12);
        let all_vars: HashSet<String> = eqs.iter().flat_map(|eq| eq.variables()).collect();
        assert!(
            all_vars
                .iter()
                .any(|v| v.starts_with(IGNORED_OUTPUT_PREFIX) && v.contains("[2,1]")),
            "expected a padded sink matrix, got {all_vars:?}"
        );
    }

    // ── kernel-synthetic CALLs (qr / chol / expm / svd / fft / conv / fits) ──
    //
    // Values are the Java oracle's, captured by running each document through
    // `tools/golden-dumper` against the real engine; the same documents are
    // frozen in `fixtures/corpus` so the parity gate keeps them honest.

    /// The synthetic function name of every `out = kernel(…)` equation, in
    /// emission order.
    fn call_names(eqs: &[Equation]) -> Vec<String> {
        eqs.iter()
            .filter_map(|eq| match &eq.rhs {
                Expr::Call { function, .. } => Some(function.clone()),
                _ => None,
            })
            .collect()
    }

    /// Evaluate a kernel expansion: seed the literal element equations, then
    /// resolve every `out = kernel(known…)` equation in one pass (which is all
    /// these flatteners emit).
    fn kernel_values(eqs: &[Equation]) -> Scope {
        let mut scope: Scope = literal_values(eqs).into_iter().collect();
        for eq in eqs {
            if let (Expr::Var(name), Expr::Call { .. }) = (&eq.lhs, &eq.rhs) {
                let value = eval::eval(&eq.rhs, &scope)
                    .unwrap_or_else(|e| panic!("`{}` did not evaluate: {e:?}", eq.source_text));
                scope.insert(name.clone(), value);
            }
        }
        scope
    }

    fn assert_kernel_values(eqs: &[Equation], expected: &[(&str, f64)]) {
        let values = kernel_values(eqs);
        for (name, want) in expected {
            let got = values
                .get(*name)
                .unwrap_or_else(|| panic!("`{name}` not produced; have {values:?}"));
            assert!(
                (got - want).abs() < 1e-9,
                "`{name}` = {got} but the oracle got {want}"
            );
        }
    }

    #[test]
    fn qr_emits_q_then_r_elements_with_the_java_sign_convention() {
        let eqs = expand("A = [1 0; 0 1]\nCALL QR(A : Q, R)");
        assert_eq!(
            call_names(&eqs),
            vec![
                "qr$q$0$0$2$2",
                "qr$q$0$1$2$2",
                "qr$q$1$0$2$2",
                "qr$q$1$1$2$2",
                "qr$r$0$0$2$2",
                "qr$r$0$1$2$2",
                "qr$r$1$0$2$2",
                "qr$r$1$1$2$2",
            ]
        );
        // Householder QR of I is -I in Commons Math, not +I.
        assert_kernel_values(
            &eqs,
            &[
                ("q[1,1]", -1.0),
                ("q[2,2]", -1.0),
                ("r[1,1]", -1.0),
                ("r[2,2]", -1.0),
            ],
        );
    }

    #[test]
    fn qr_sizes_a_bare_q_to_m_by_m_and_r_to_m_by_n() {
        // 3x2 input: Q is 3x3 (9 equations), R is 3x2 (6).
        let eqs = expand("A = [1 2; 3 4; 5 6]\nCALL QR(A : Q, R)");
        assert_eq!(call_names(&eqs).len(), 9 + 6);
        assert_kernel_values(&eqs, &[("q[3,3]", 0.40824829046386274), ("r[3,2]", 0.0)]);
    }

    #[test]
    fn qr_rejects_a_q_that_is_not_square_in_m() {
        let message =
            expand_err("A = [1 2; 3 4; 5 6]\nCALL QR(A[1:3,1:2] : Q[1:2,1:2], R[1:3,1:2])");
        assert!(
            message.contains("QR requires Q to be 3x3 (m x m for an m x n input)."),
            "{message}"
        );
    }

    #[test]
    fn qr_rejects_an_r_that_does_not_match_the_input_shape() {
        let message =
            expand_err("A = [1 2; 3 4; 5 6]\nCALL QR(A[1:3,1:2] : Q[1:3,1:3], R[1:3,1:3])");
        assert!(
            message.contains("QR requires R to match the input shape (3x2)."),
            "{message}"
        );
    }

    #[test]
    fn cholesky_emits_the_full_l_matrix_including_its_zero_upper_half() {
        let eqs = expand("A = [4 0; 0 9]\nCALL Cholesky(A : L)");
        assert_eq!(
            call_names(&eqs),
            vec![
                "chol$l$0$0$2",
                "chol$l$0$1$2",
                "chol$l$1$0$2",
                "chol$l$1$1$2",
            ]
        );
        assert_kernel_values(
            &eqs,
            &[
                ("l[1,1]", 2.0),
                ("l[1,2]", 0.0),
                ("l[2,1]", 0.0),
                ("l[2,2]", 3.0),
            ],
        );
    }

    #[test]
    fn cholesky_rejects_mismatched_sizes() {
        let message = expand_err("A = [4 0; 0 9]\nCALL Cholesky(A[1:2,1:2] : L[1:1,1:1])");
        assert!(
            message.contains("Cholesky requires square matrices of identical size."),
            "{message}"
        );
    }

    #[test]
    fn matexp_of_the_zero_matrix_is_the_identity() {
        let eqs = expand("A = [0 0; 0 0]\nCALL MatExp(A : E)");
        assert_eq!(
            call_names(&eqs),
            vec!["expm$0$0$2", "expm$0$1$2", "expm$1$0$2", "expm$1$1$2"]
        );
        assert_kernel_values(
            &eqs,
            &[
                ("e[1,1]", 1.0),
                ("e[1,2]", 0.0),
                ("e[2,1]", 0.0),
                ("e[2,2]", 1.0),
            ],
        );
    }

    #[test]
    fn matexp_rejects_a_non_square_input() {
        let message = expand_err("A = [1 2; 3 4; 5 6]\nCALL MatExp(A[1:3,1:2] : E[1:3,1:2])");
        assert!(
            message.contains("MatExp requires square matrices of identical size."),
            "{message}"
        );
    }

    #[test]
    fn singular_values_are_reported_in_descending_order() {
        let eqs = expand("A = [2 0; 0 3]\nCALL SingularValues(A : s)");
        assert_eq!(call_names(&eqs), vec!["svd$s$0$2$2", "svd$s$1$2$2"]);
        // Descending, so the 3 from A[2,2] comes first.
        assert_kernel_values(&eqs, &[("s[1]", 3.0), ("s[2]", 2.0)]);
    }

    #[test]
    fn singular_values_size_a_bare_output_to_min_rows_cols() {
        let eqs = expand("A = [1 2; 3 4; 5 6]\nCALL SingularValues(A : s)");
        assert_eq!(call_names(&eqs), vec!["svd$s$0$3$2", "svd$s$1$3$2"]);
    }

    #[test]
    fn singular_values_rejects_a_wrongly_sized_output() {
        let message = expand_err("A = [1 2; 3 4; 5 6]\nCALL SingularValues(A[1:3,1:2] : s[1:3])");
        assert!(
            message.contains(
                "SingularValues requires an output vector of length min(rows, cols) = 2."
            ),
            "{message}"
        );
    }

    #[test]
    fn svd_emits_u_then_smat_then_v_with_the_thin_shapes() {
        // 3x2 -> p = 2: U is 3x2 (6), S is 2x2 (4), V is 2x2 (4).
        let eqs = expand("A = [1 2; 3 4; 5 6]\nCALL SVD(A : U, S, V)");
        let names = call_names(&eqs);
        assert_eq!(names.len(), 6 + 4 + 4);
        assert_eq!(names[0], "svd$u$0$0$3$2");
        assert_eq!(names[6], "svd$smat$0$0$3$2");
        assert_eq!(names[10], "svd$v$0$0$3$2");
        assert_kernel_values(
            &eqs,
            &[
                ("s[1,1]", 9.525518091565106),
                ("s[1,2]", 0.0),
                ("s[2,2]", 0.5143005806586448),
                ("u[1,1]", 0.22984769640007152),
            ],
        );
    }

    #[test]
    fn svd_rejects_outputs_that_are_not_the_thin_shapes() {
        let message = expand_err(
            "A = [1 2; 3 4; 5 6]\nCALL SVD(A[1:3,1:2] : U[1:3,1:3], S[1:2,1:2], V[1:2,1:2])",
        );
        assert!(
            message.contains("SVD of a 3x2 matrix requires outputs U (3x2), S (2x2), and V (2x2)."),
            "{message}"
        );
    }

    #[test]
    fn fft_interleaves_the_real_and_imaginary_outputs() {
        let eqs = expand("re = [1, 0, 0, 0]\nim = [0, 0, 0, 0]\nCALL FFT(re, im : fr, fi)");
        assert_eq!(
            call_names(&eqs),
            vec![
                "fft$re$0$4",
                "fft$im$0$4",
                "fft$re$1$4",
                "fft$im$1$4",
                "fft$re$2$4",
                "fft$im$2$4",
                "fft$re$3$4",
                "fft$im$3$4",
            ]
        );
        // The DFT of the unit impulse is flat.
        assert_kernel_values(
            &eqs,
            &[
                ("fr[1]", 1.0),
                ("fr[4]", 1.0),
                ("fi[1]", 0.0),
                ("fi[4]", 0.0),
            ],
        );
    }

    #[test]
    fn fft_packs_the_real_vector_before_the_imaginary_one() {
        let eqs = expand("re = [1, 2]\nim = [3, 4]\nCALL FFT(re, im : fr, fi)");
        let Expr::Call { args, .. } = &eqs[4].rhs else {
            panic!("expected a kernel call, got {:?}", eqs[4].rhs);
        };
        assert_eq!(
            args.to_vec(),
            vec![var("re[1]"), var("re[2]"), var("im[1]"), var("im[2]")]
        );
    }

    #[test]
    fn ifft_uses_the_inverse_prefix() {
        let eqs = expand("re = [1, 1]\nim = [0, 0]\nCALL IFFT(re, im : gr, gi)");
        assert_eq!(
            call_names(&eqs),
            vec!["ifft$re$0$2", "ifft$im$0$2", "ifft$re$1$2", "ifft$im$1$2"]
        );
        // IFFT of a constant spectrum is the (scaled) impulse.
        assert_kernel_values(&eqs, &[("gr[1]", 1.0), ("gr[2]", 0.0)]);
    }

    #[test]
    fn fft_needs_two_input_and_two_output_vectors() {
        let message = expand_err("re = [1, 0]\nCALL FFT(re[1:2] : fr[1:2], fi[1:2])");
        assert!(
            message.contains(
                "FFT expects 2 input vectors (real, imag) and 2 output vectors, \
                 e.g. CALL FFT(re[1:n], im[1:n] : outRe[1:n], outIm[1:n])"
            ),
            "{message}"
        );
        let inverse = expand_err("re = [1, 0]\nCALL IFFT(re[1:2] : fr[1:2], fi[1:2])");
        assert!(
            inverse.contains("IFFT expects 2 input vectors"),
            "{inverse}"
        );
    }

    #[test]
    fn fft_needs_all_four_vectors_the_same_length() {
        let message = expand_err(
            "re = [1, 0, 0]\nim = [0, 0]\nCALL FFT(re[1:3], im[1:2] : fr[1:3], fi[1:3])",
        );
        assert!(
            message.contains("FFT requires all four vectors to have the same length."),
            "{message}"
        );
    }

    #[test]
    fn convolve_emits_m_plus_n_minus_one_elements() {
        let eqs = expand("a = [1, 2]\nb = [1, 3]\nCALL Convolve(a[1:2], b[1:2] : c[1:3])");
        assert_eq!(
            call_names(&eqs),
            vec!["conv$0$2$2", "conv$1$2$2", "conv$2$2$2"]
        );
        assert_kernel_values(&eqs, &[("c[1]", 1.0), ("c[2]", 5.0), ("c[3]", 6.0)]);
    }

    #[test]
    fn convolve_sizes_a_bare_output_to_m_plus_n_minus_one() {
        let eqs = expand("a = [1, 2, 3]\nb = [4, 5]\nCALL Convolve(a, b : c)");
        assert_eq!(call_names(&eqs).len(), 4);
        assert_kernel_values(
            &eqs,
            &[
                ("c[1]", 4.0),
                ("c[2]", 13.0),
                ("c[3]", 22.0),
                ("c[4]", 15.0),
            ],
        );
    }

    #[test]
    fn convolve_rejects_a_wrongly_sized_output() {
        let message = expand_err("a = [1, 2]\nb = [1, 3]\nCALL Convolve(a[1:2], b[1:2] : c[1:2])");
        assert!(
            message.contains("Convolve requires the output length to be m + n - 1 = 3."),
            "{message}"
        );
    }

    #[test]
    fn linfit_emits_three_scalar_outputs_in_slope_intercept_r2_order() {
        let eqs = expand("x = [1, 2, 3]\ny = [2, 4, 6]\nCALL LinFit(x, y : m, b, r2)");
        assert_eq!(
            call_names(&eqs),
            vec!["linfit$slope$3", "linfit$intercept$3", "linfit$r2$3"]
        );
        assert_kernel_values(&eqs, &[("m", 2.0), ("b", 0.0), ("r2", 1.0)]);
    }

    #[test]
    fn linfit_needs_x_and_y_of_equal_length() {
        let message =
            expand_err("x = [1, 2, 3]\ny = [1, 2]\nCALL LinFit(x[1:3], y[1:2] : m, b, r2)");
        assert!(
            message.contains("LinFit requires x and y of equal length."),
            "{message}"
        );
    }

    #[test]
    fn linfit_rejects_more_than_three_outputs() {
        let message =
            expand_err("x = [1, 2]\ny = [1, 2]\nCALL LinFit(x[1:2], y[1:2] : m, b, r2, extra)");
        assert!(
            message.contains("LinFit expects 2 input vectors and 3 outputs (slope, intercept, r2)"),
            "{message}"
        );
    }

    /// Trailing omission works for the fixed-arity kernels (Java
    /// `padOmittedOutputs` / `expectedOutputCount`): the dropped slot is still
    /// computed, into a hidden sink.
    #[test]
    fn linfit_pads_an_omitted_trailing_output_with_a_sink() {
        let eqs = expand("x = [1, 2, 3]\ny = [2, 4, 6]\nCALL LinFit(x, y : m, b)");
        assert_eq!(
            call_names(&eqs),
            vec!["linfit$slope$3", "linfit$intercept$3", "linfit$r2$3"]
        );
        let all_vars: HashSet<String> = eqs.iter().flat_map(|eq| eq.variables()).collect();
        assert!(
            all_vars
                .iter()
                .any(|v| v.starts_with(IGNORED_OUTPUT_PREFIX)),
            "expected a padded sink, got {all_vars:?}"
        );
    }

    #[test]
    fn polyfit_coefficients_are_ascending_powers() {
        // y = 2x + 1 -> c[1] is the constant term, c[2] the slope.
        let eqs = expand("x = [0, 1, 2, 3]\ny = [1, 3, 5, 7]\nCALL PolyFit(x, y, 1 : c)");
        assert_eq!(call_names(&eqs), vec!["polyfit$0$1$4", "polyfit$1$1$4"]);
        assert_kernel_values(&eqs, &[("c[1]", 1.0), ("c[2]", 2.0)]);
    }

    #[test]
    fn polyfit_sizes_a_bare_output_to_degree_plus_one() {
        let eqs = expand("x = [1, 2, 3]\ny = [1, 4, 9]\nCALL PolyFit(x, y, 2 : c)");
        assert_eq!(call_names(&eqs).len(), 3);
        assert_kernel_values(&eqs, &[("c[3]", 1.0)]);
    }

    #[test]
    fn polyfit_rejects_a_short_coefficient_vector() {
        let message =
            expand_err("x = [1, 2, 3]\ny = [1, 4, 9]\nCALL PolyFit(x[1:3], y[1:3], 2 : c[1:2])");
        assert!(
            message.contains("PolyFit requires a coefficient vector of length degree + 1 = 3."),
            "{message}"
        );
    }

    #[test]
    fn polyfit_rejects_a_negative_degree() {
        let message =
            expand_err("x = [1, 2, 3]\ny = [1, 4, 9]\nCALL PolyFit(x[1:3], y[1:3], -1 : c[1:1])");
        assert!(
            message.contains("PolyFit degree must be >= 0."),
            "{message}"
        );
    }

    /// The kernel flatteners copy the input matrix into every equation, so the
    /// generation budget is asserted *before* the batch is built (Java's
    /// `BoundedEquationList.addAll`). Same error type and message as the
    /// per-insert check, but it costs O(1) instead of gigabytes.
    #[test]
    fn kernel_calls_over_the_equation_budget_are_refused_before_they_are_built() {
        // 130^2 (Q) + 130^2 (R) = 33 800 equations.
        let message = expand_err("CALL QR(A[1:130,1:130] : Q[1:130,1:130], R[1:130,1:130])");
        assert!(message.contains(TOO_MANY_EQUATIONS), "{message}");
        // Vector kernels are budgeted the same way: 2 * 13 000 outputs.
        let message = expand_err("CALL FFT(re[1:13000], im[1:13000] : fr[1:13000], fi[1:13000])");
        assert!(message.contains(TOO_MANY_EQUATIONS), "{message}");
        // …and a request that fits is untouched.
        let eqs = expand("CALL Cholesky(A[1:60,1:60] : L[1:60,1:60])");
        assert_eq!(eqs.len(), 3600);
    }

    #[test]
    fn kernel_call_arguments_are_packed_row_major() {
        let eqs = expand("A = [1 2; 3 4]\nCALL Cholesky(A : L)");
        let Expr::Call { args, .. } = &eqs[4].rhs else {
            panic!("expected a kernel call, got {:?}", eqs[4].rhs);
        };
        assert_eq!(
            args.to_vec(),
            vec![var("a[1,1]"), var("a[1,2]"), var("a[2,1]"), var("a[2,2]")]
        );
    }

    // ── CALL error surface ──────────────────────────────────────────────────

    #[test]
    fn unknown_call_names_the_java_error() {
        let message = expand_err("CALL Nope(x : y)");
        assert!(
            message.contains("Unknown PROCEDURE or MODULE: 'nope'"),
            "{message}"
        );
    }

    #[test]
    fn unported_call_intrinsics_are_refused_by_name() {
        let message = expand_err("CALL EulerDecompose(R[1:3,1:3] : phi, theta, psi)");
        assert!(
            message.contains("`CALL eulerdecompose` is not supported by the wasm engine yet"),
            "{message}"
        );
    }

    /// Every wired CALL name must be gone from the refusal list, or
    /// `flatten_call_proc` would short-circuit before its flattener. Covers
    /// both the Phase-4 kernel set and the Phase-9 control-systems suite.
    #[test]
    fn the_wired_call_intrinsics_left_the_unported_list() {
        let kernels = [
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
        ];
        for name in kernels
            .into_iter()
            .chain(control::flatten::CALL_NAMES)
            .chain(["mason"])
        {
            assert!(
                !UNPORTED_CALL_INTRINSICS.contains(&name),
                "`{name}` is wired but still listed as unported"
            );
        }
    }

    /// …and the two that are genuinely unported must still be refused, by
    /// name rather than as an unknown procedure. (The eigen pair left this
    /// list when ledger item 34 closed.)
    #[test]
    fn the_euler_decompositions_are_still_refused() {
        for name in ["eulerrotate", "eulerdecompose"] {
            assert!(
                UNPORTED_CALL_INTRINSICS.contains(&name),
                "`{name}` is not implemented and must remain refused"
            );
            assert!(
                !control::flatten::handles(name),
                "`{name}` must not be claimed by control::flatten"
            );
        }
        // The eigen pair must stay out of control::flatten too — they are
        // matrix-expansion intrinsics, dispatched in flatten_call_proc.
        for name in ["eigenvalues", "eigen"] {
            assert!(
                !control::flatten::handles(name),
                "`{name}` must not be claimed by control::flatten"
            );
        }
    }

    // ── SYMBOLIC ────────────────────────────────────────────────────────────

    /// Port of `flattenIdentity`: the identity is solved for its coefficients
    /// and each becomes a concrete `name = value` equation the ordinary solver
    /// reports. `a*s = 2*s` holds for all `s` exactly when `a = 2`.
    #[test]
    fn a_symbolic_identity_becomes_one_equation_per_coefficient() {
        let eqs = expand("SYMBOLIC s\na * s = 2 * s");
        assert_eq!(sides(&eqs), vec![(var("a"), num(2.0))]);
    }

    /// The `TransferFunction.expandCalls` pre-pass: an identity may be written
    /// with `tf(num, den)` on one side. `(s+3)/(s^2+3s+2) = A/(s+1) + B/(s+2)`
    /// has residues `A = 2`, `B = -1`.
    #[test]
    fn a_tf_call_in_an_identity_is_expanded_before_it_is_solved() {
        let eqs = expand("SYMBOLIC s\ntf([1, 3], [1, 3, 2]) = A/(s+1) + B/(s+2)");
        assert_eq!(
            sides(&eqs),
            vec![(var("a"), num(2.0)), (var("b"), num(-1.0))]
        );
    }

    /// A CAS failure surfaces as a parse error carrying the CAS's own words,
    /// which is `flattenIdentity`'s `catch (CasException) -> ParseException`.
    #[test]
    fn an_unsolvable_identity_reports_the_cas_message() {
        let message = expand_err("SYMBOLIC s\ns = s");
        assert!(!message.is_empty(), "{message}");
        assert!(
            message.contains("identity") || message.contains("coefficient"),
            "{message}"
        );
    }

    #[test]
    fn an_identity_may_involve_only_one_symbolic_variable() {
        let message = expand_err("SYMBOLIC s, t\ns = t");
        assert!(
            message.contains("An identity may involve only one SYMBOLIC variable"),
            "{message}"
        );
    }

    #[test]
    fn symbolic_declarations_alone_do_not_block_scalar_equations() {
        let eqs = expand("SYMBOLIC s\nx = 2");
        assert_eq!(sides(&eqs), vec![(var("x"), num(2.0))]);
    }

    // ── the internal-temp filter ────────────────────────────────────────────

    #[test]
    fn internal_temp_names_are_recognized() {
        assert!(is_internal_temp("inverse_temp_12[1,2]"));
        assert!(is_internal_temp("backslash_temp_0[1]"));
        assert!(is_internal_temp("solvelinear_temp_4[2]"));
        assert!(is_internal_temp("INVERSE_TEMP_3[1,1]"));
        assert!(!is_internal_temp("motor_temp_5"));
        assert!(!is_internal_temp("x[1]"));
    }

    // ── BLAS-style helpers ──────────────────────────────────────────────────

    #[test]
    fn gemv_expands_to_alpha_a_x_plus_beta_y() {
        let eqs = expand("A = [1 2; 3 4]\nx = [1; 1]\ny0 = [1; 1]\nz = gemv(2, A, x, 3, y0)");
        assert_satisfied(&eqs, &[("z[1]", 9.0), ("z[2]", 17.0)]);
    }

    #[test]
    fn axpy_and_scal_and_copy_expand() {
        let eqs =
            expand("x = [1; 2]\ny = [10; 20]\np = axpy(3, x, y)\nq = scal(2, x)\nr = copy(x)");
        assert_satisfied(
            &eqs,
            &[
                ("p[1]", 13.0),
                ("p[2]", 26.0),
                ("q[1]", 2.0),
                ("q[2]", 4.0),
                ("r[1]", 1.0),
                ("r[2]", 2.0),
            ],
        );
    }

    #[test]
    fn gemm_and_ger_expand() {
        let eqs = expand(
            "A = [1 2; 3 4]\nB = [1 0; 0 1]\nC0 = [1 1; 1 1]\nM = gemm(2, A, B, 1, C0)\nx = [1; 2]\ny = [3; 4]\nG = ger(1, x, y, C0)",
        );
        assert_satisfied(
            &eqs,
            &[
                ("m[1,1]", 3.0),
                ("m[1,2]", 5.0),
                ("m[2,1]", 7.0),
                ("m[2,2]", 9.0),
                ("g[1,1]", 4.0),
                ("g[1,2]", 5.0),
                ("g[2,1]", 7.0),
                ("g[2,2]", 9.0),
            ],
        );
    }
}
