//! First-order uncertainty propagation — the `val ± unc` engine.
//!
//! Port of the uncertainty mechanism that lives inside
//! `../frEES/backend/core/src/main/java/com/frees/backend/core/EquationSystemSolver.java`
//! (`UncertaintyContribution`, `UncPropagation`, `propagateUncertainty`,
//! `partitionVariables`, `computeNumericalJacobian`, `solveRssUncertainties`,
//! `splitJacobianColumns`, `selectNonZeroRows`, `computeDependentVariances`,
//! `setSourceUncertainties`, `evaluateSystemResiduals`, `mentionsUncertaintyOf`,
//! `extractUncertaintyEquations`, `getUncertaintyTarget`,
//! `applyUncertaintySpecs`, `injectUncertaintyValues`,
//! `resolveUncertaintySecondPass`) — it is not a single Java file, so the
//! module gathers the whole mechanism in one place.
//!
//! # What the engine does
//!
//! Every variable that carries a declared `uncertainty > 0` is an **uncertainty
//! source**; every other variable in the system is **dependent**. The system's
//! residual Jacobian is taken numerically (forward differences), split into its
//! dependent columns `Jy` and its source columns `Jx`, and for each source `i`
//! with stated uncertainty `u_i` the linear system
//!
//! ```text
//! Jy · dy = −Jx[:, i] · u_i
//! ```
//!
//! is solved in the least-squares sense (SVD pseudo-inverse — the system is
//! generally rectangular and may be rank-deficient). `dy[j]` is the signed
//! first-order response of dependent variable `j` to source `i`: the **tornado
//! contribution**. The variable's propagated uncertainty is the root sum of
//! squares over the sources, which is the independent-sources assumption the
//! Java engine makes explicit by summing `dy·dy` and taking a square root.
//!
//! # `UncertaintyOf(X)` — two spellings, two roles
//!
//! * **`UncertaintyOf(X) = <expr>`** as an equation's left-hand side is *not* an
//!   equation. [`extract_uncertainty_equations`] lifts it out of the system and
//!   remembers the RHS; [`apply_uncertainty_specs`] evaluates it at the solved
//!   state and pins the result into `X`'s spec as its stated uncertainty.
//! * **`UncertaintyOf(X)` inside an active equation** is a *query*. The
//!   evaluator resolves it by reading the scope entry `uncertaintyof$<x>`
//!   ([`UNCERTAINTY_OF_FN`]), which [`inject_uncertainty_values`] publishes
//!   after a propagation pass. Because the first pass solves before any
//!   uncertainty exists, a document containing such a query needs a **second
//!   solve pass** — [`resolve_second_pass`] — which feeds the first pass's
//!   uncertainties back in, re-solves warm-started, and re-propagates.
//!
//! # Wiring
//!
//! The functions here take an equation list, a value scope, specs and defs;
//! nothing in this module re-implements Newton (the module-level rule in
//! [`crate::analysis`]). [`analyze`] is the orchestrator and mirrors
//! `EquationSystemSolver.solve` lines 362–369 exactly; it takes the re-solve as
//! a closure so the caller supplies the engine's own block-and-solve.
//!
//! The `val ± unc` rendering itself is a boundary concern: `backend/web`'s
//! `ReplEvaluator` appends `" ± " + number(uncertainty)` and the frontend grids
//! suppress the suffix when the uncertainty is null or zero. Core publishes the
//! numbers; it does not format them.

// Two guards here are deliberately written `!(x > y)` rather than `x <= y`: the
// Java writes `sigma <= 0.0` and `singularValues[i] > tol`, and negating the
// *positive* form is what keeps a NaN on the reject side of the branch in both
// cases (`NaN <= 0.0` is false, so a literal transcription would emit a tornado
// breakdown for a NaN sigma). `neg_cmp_op_on_partial_ord` wants the readable
// form, which would silently change the NaN behaviour — the port's parity rule
// wins.
#![allow(clippy::neg_cmp_op_on_partial_ord)]

use std::collections::{BTreeMap, BTreeSet};

use super::distributions::Distribution;
use super::sampling::{Correlation, CorrelationEntries};
use crate::ast::{Equation, Expr};
use crate::diag::Result;
use crate::eval::{eval_with, EvalContext, Scope};
use crate::linalg::{Mat, SvdSolver};

/// Scope-key prefix under which a computed uncertainty is published so
/// `UncertaintyOf(X)` queries can read it. Port of
/// `EquationSystemSolver.UNCERTAINTY_OF_FN`; the evaluator's `uncertaintyof`
/// arm (`crate::eval`) reads exactly this key.
pub const UNCERTAINTY_OF_FN: &str = "uncertaintyof$";

/// A residual row is treated as *not* constraining the dependent block when
/// every `Jy` entry falls below this. Port of the `1e-12` literal in
/// `selectNonZeroRows`.
const JY_ROW_ZERO_THRESHOLD: f64 = 1e-12;

/// The `VariableSpec` fields this engine reads and writes.
///
/// The Java `VariableSpec` is `(name, guess, lower, upper, uncertainty)`; the
/// solver-side three are already modelled by `engine::VarSpec`, whose public
/// face [`crate::engine::VariableOverride`] deliberately has no `uncertainty`
/// field. This is the uncertainty-aware form the propagation engine,
/// `UncertaintyOf(X) = expr` and Monte Carlo all need, kept local to `analysis`
/// so the solver contract is untouched.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UncertaintySpec {
    pub guess: f64,
    pub lower: f64,
    pub upper: f64,
    /// Stated 1-sigma uncertainty. Only `> 0.0` makes the variable a source —
    /// the Java test is `spec != null && spec.uncertainty() > 0.0`.
    pub uncertainty: f64,
}

impl Default for UncertaintySpec {
    /// The Java `VariableSpec` defaults: guess 1.0, unbounded, no uncertainty.
    fn default() -> UncertaintySpec {
        UncertaintySpec {
            guess: crate::engine::DEFAULT_GUESS,
            lower: f64::NEG_INFINITY,
            upper: f64::INFINITY,
            uncertainty: 0.0,
        }
    }
}

impl UncertaintySpec {
    /// A spec that states nothing but an uncertainty — the common case when the
    /// Variable Information window supplies only a `±` column.
    pub fn with_uncertainty(uncertainty: f64) -> UncertaintySpec {
        UncertaintySpec {
            uncertainty,
            ..UncertaintySpec::default()
        }
    }
}

/// One uncertainty source's signed first-order contribution to a dependent
/// variable. Port of `EquationSystemSolver.UncertaintyContribution`; the
/// root-sum-square of a variable's contributions is its propagated
/// uncertainty, and the ranked list is the tornado chart's data
/// (`uncertaintyBreakdown` in `web/src/api.ts`).
#[derive(Debug, Clone, PartialEq)]
pub struct UncertaintyContribution {
    pub source: String,
    pub value: f64,
}

/// Propagated uncertainties plus, per dependent variable, the signed
/// contribution of each source. Port of `EquationSystemSolver.UncPropagation`.
///
/// `uncertainties` covers **every** variable of the system: a source maps to
/// its stated value, a dependent variable to its propagated sigma, and — when
/// the document declares no source at all — everything maps to `0.0`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct UncPropagation {
    pub uncertainties: BTreeMap<String, f64>,
    pub contributions: BTreeMap<String, Vec<UncertaintyContribution>>,
    /// Phase 4.3: the input shape each source declared, for the samplers that
    /// need more than a sigma. Empty when every source is the historical
    /// bare-`±` normal.
    pub distributions: BTreeMap<String, Distribution>,
    /// Phase 4.3: the resolved `Correlation(A, B)` coefficients, already
    /// validated against the source list.
    pub correlations: CorrelationEntries,
}

/// The equations that stay in the system, plus the `UncertaintyOf(X) = expr`
/// declarations lifted out of it. Port of
/// `EquationSystemSolver.ExtractedUncertainties`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ExtractedUncertainties {
    pub active_equations: Vec<Equation>,
    pub declarations: DeclarationSet,
}

/// The declaration families lifted out of the equation system: statements about
/// a variable rather than equations to solve, so counting them would unbalance
/// the document.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DeclarationSet {
    /// Lowercase variable name → the RHS expression stating its uncertainty.
    pub uncertainty_exprs: BTreeMap<String, Expr>,
    /// Phase 4.3: lowercase variable name → the `Name(args…)` shape declared by
    /// `DistributionOf(X) = Uniform(0.9, 1.1)`. The call is kept unevaluated
    /// because its arguments may reference solved values.
    pub distribution_exprs: BTreeMap<String, (String, Vec<Expr>)>,
    /// Phase 4.3: lowercase ordered name pair → the RHS of
    /// `Correlation(A, B) = 0.6`. The pair is stored as written; assembling and
    /// validating the matrix belongs to [`super::sampling::Correlation`].
    pub correlation_exprs: BTreeMap<(String, String), Expr>,
}

// ---------------------------------------------------------------------------
// Extraction of `UncertaintyOf(X) = expr` declarations
// ---------------------------------------------------------------------------

/// Splits `UncertaintyOf(X) = <expr>` declarations out of the equation list.
///
/// Port of `extractUncertaintyEquations`. A declaration is recognised by its
/// **left-hand side only**, and a later declaration for the same variable
/// overwrites an earlier one (the Java `Map.put`).
pub fn extract_uncertainty_equations(equations: &[Equation]) -> ExtractedUncertainties {
    let mut active = Vec::with_capacity(equations.len());
    let mut uncertainty_exprs = BTreeMap::new();
    let mut distribution_exprs = BTreeMap::new();
    let mut correlation_exprs = BTreeMap::new();
    for eq in equations {
        if let Some(var) = uncertainty_target(&eq.lhs) {
            uncertainty_exprs.insert(var.to_ascii_lowercase(), eq.rhs.clone());
            continue;
        }
        // Phase 4.3 declarations, lifted at the same position and for the same
        // reason: they state a *property* of a variable, so counting them as
        // equations would unbalance the system.
        if let Some(var) = declaration_target(&eq.lhs, "distributionof", 1) {
            if let Expr::Call { function, args } = &eq.rhs {
                distribution_exprs.insert(
                    var[0].to_ascii_lowercase(),
                    (function.clone(), args.clone()),
                );
                continue;
            }
            // A malformed right-hand side stays in the system so the solver
            // reports it against the user's own text rather than vanishing.
        } else if let Some(pair) = declaration_target(&eq.lhs, "correlation", 2) {
            correlation_exprs.insert(
                (pair[0].to_ascii_lowercase(), pair[1].to_ascii_lowercase()),
                eq.rhs.clone(),
            );
            continue;
        }
        active.push(eq.clone());
    }
    ExtractedUncertainties {
        active_equations: active,
        declarations: DeclarationSet {
            uncertainty_exprs,
            distribution_exprs,
            correlation_exprs,
        },
    }
}

/// The variable names a declaration call like `DistributionOf(X)` or
/// `Correlation(A, B)` addresses, when it has exactly `arity` arguments and
/// every one of them names a variable (or quotes one).
///
/// Same argument rule as [`uncertainty_target`]: a bare identifier or a string
/// literal, nothing else, so `Correlation(a + b, c)` is not mistaken for a
/// declaration and stays an ordinary equation.
fn declaration_target<'a>(expr: &'a Expr, name: &str, arity: usize) -> Option<Vec<&'a str>> {
    let Expr::Call { function, args } = expr else {
        return None;
    };
    if function != name || args.len() != arity {
        return None;
    }
    args.iter()
        .map(|arg| match arg {
            Expr::Var(name) => Some(name.as_str()),
            Expr::Str(value) => Some(value.as_str()),
            _ => None,
        })
        .collect()
}

/// The variable an `UncertaintyOf(...)` call names, or `None` for any other
/// expression. Port of `getUncertaintyTarget`: the single argument may be a
/// variable (`UncertaintyOf(T)`) or a string literal (`UncertaintyOf('T')`).
///
/// The AST already lowercases call and variable names
/// (`Expr::call` / `Expr::var`), so the `equalsIgnoreCase` of the Java is a
/// plain comparison here — but a *string* argument keeps its source case, which
/// is why the caller still lowercases.
fn uncertainty_target(expr: &Expr) -> Option<&str> {
    let Expr::Call { function, args } = expr else {
        return None;
    };
    if function != "uncertaintyof" || args.len() != 1 {
        return None;
    }
    match &args[0] {
        Expr::Var(name) => Some(name),
        Expr::Str(value) => Some(value),
        _ => None,
    }
}

/// True when any equation *queries* `UncertaintyOf(...)` anywhere in it — the
/// trigger for the second solve pass. Port of `mentionsUncertaintyOf`.
pub fn mentions_uncertainty_of(equations: &[Equation]) -> bool {
    equations
        .iter()
        .any(|eq| mentions_uncertainty_of_expr(&eq.lhs) || mentions_uncertainty_of_expr(&eq.rhs))
}

/// Port of `mentionsUncertaintyOfExpr`. Note the Java arm for `Expr.Call`:
/// a call *named* `uncertaintyof` short-circuits to true without inspecting its
/// arguments, and the `ArrayAccess` arm walks only the indices.
fn mentions_uncertainty_of_expr(expr: &Expr) -> bool {
    match expr {
        Expr::Num { .. } | Expr::Str(_) | Expr::Var(_) => false,
        Expr::Neg(inner) | Expr::Not(inner) => mentions_uncertainty_of_expr(inner),
        Expr::BinOp { left, right, .. }
        | Expr::Compare { left, right, .. }
        | Expr::Logical { left, right, .. }
        | Expr::Range {
            start: left,
            end: right,
        } => mentions_uncertainty_of_expr(left) || mentions_uncertainty_of_expr(right),
        Expr::Call { function, args } => {
            function == "uncertaintyof" || args.iter().any(mentions_uncertainty_of_expr)
        }
        Expr::ArrayAccess { indices, .. } => indices.iter().any(mentions_uncertainty_of_expr),
        Expr::ArrayLiteral(elements) => elements.iter().any(mentions_uncertainty_of_expr),
    }
}

// ---------------------------------------------------------------------------
// Specs and the `uncertaintyof$` scope entries
// ---------------------------------------------------------------------------

/// Evaluates each `UncertaintyOf(X) = expr` at the solved state and pins the
/// result into `X`'s spec. Port of `applyUncertaintySpecs`.
///
/// An expression that cannot be evaluated at this state is **silently
/// skipped** — the Java `catch (Exception ignored)`; it leaves the spec as-is
/// so a later pass can try again.
pub fn apply_uncertainty_specs(
    uncertainty_exprs: &BTreeMap<String, Expr>,
    values: &Scope,
    specs: &mut BTreeMap<String, UncertaintySpec>,
    ctx: EvalContext<'_>,
) {
    for (name, expr) in uncertainty_exprs {
        let Ok(value) = eval_with(expr, values, ctx) else {
            continue;
        };
        let old = specs.get(name).copied().unwrap_or_default();
        specs.insert(
            name.clone(),
            UncertaintySpec {
                uncertainty: value,
                ..old
            },
        );
    }
}

/// Publishes computed uncertainties into the value scope as
/// `uncertaintyof$<var>` so `UncertaintyOf(X)` queries in active equations can
/// read them. Port of `injectUncertaintyValues`.
pub fn inject_uncertainty_values(values: &mut Scope, uncertainties: &BTreeMap<String, f64>) {
    for (name, unc) in uncertainties {
        values.insert(
            format!("{UNCERTAINTY_OF_FN}{}", name.to_ascii_lowercase()),
            *unc,
        );
    }
}

/// Copies the `uncertaintyof$…` entries of a warm-start scope into the value
/// map a solve is about to iterate.
///
/// **This is required for the second pass to mean anything, and the Rust
/// `engine::solve_equation_list` does not do it yet.** The Java
/// `EquationSystemSolver.solveEquationList` seeds `values` from `allVars` only
/// — and `uncertaintyof$area` is not a variable of the system, so it would be
/// dropped — and therefore carries the injected entries across explicitly:
///
/// ```text
/// if (mutableWarmStart != null) {
///     for (Map.Entry<String, Double> entry : mutableWarmStart.entrySet()) {
///         if (entry.getKey().startsWith(UNCERTAINTY_OF_FN)) {
///             values.put(entry.getKey(), entry.getValue());
///         }
///     }
/// }
/// ```
///
/// Without that block the evaluator's `uncertaintyof` arm falls back to `0.0`
/// and a document like `rel = UncertaintyOf(area) / area` silently solves to
/// zero instead of failing — which is exactly the kind of wrong answer this
/// port refuses to ship. Whoever wires [`resolve_second_pass`] into
/// `engine::solve_equation_list` must call this on the warm-start scope.
pub fn carry_uncertainty_entries(warm: &Scope, values: &mut Scope) {
    for (key, value) in warm {
        if key.starts_with(UNCERTAINTY_OF_FN) {
            values.insert(key.clone(), *value);
        }
    }
}

// ---------------------------------------------------------------------------
// Propagation
// ---------------------------------------------------------------------------

/// Variables split into the sources and the dependents, plus a zero-initialized
/// uncertainty map covering every variable. Port of `UncPartition`.
struct UncPartition {
    unc_vars: Vec<String>,
    dep_vars: Vec<String>,
    uncertainties: BTreeMap<String, f64>,
}

/// Every variable of the system, sorted — the Java `collectVariables`, which
/// returns a `TreeSet`.
fn collect_variables(equations: &[Equation]) -> Vec<String> {
    let mut out = BTreeSet::new();
    for eq in equations {
        out.extend(eq.variables());
    }
    out.into_iter().collect()
}

/// System-wide first-order propagation. Port of `propagateUncertainty`.
///
/// # Errors
///
/// Propagates whatever the evaluator raises on the **base** residual evaluation
/// (the Java `evaluateSystemResiduals` does not catch), or a linear-algebra
/// failure in the SVD.
pub fn propagate(
    equations: &[Equation],
    values: &Scope,
    specs: &BTreeMap<String, UncertaintySpec>,
    ctx: EvalContext<'_>,
    correlations: &CorrelationEntries,
) -> Result<UncPropagation> {
    let var_list = collect_variables(equations);
    let part = partition_variables(&var_list, specs);
    if part.unc_vars.is_empty() {
        if !correlations.is_empty() {
            return Err(crate::diag::FreesError::solver(
                "Correlation(...) was declared but the document has no uncertainty \
                 sources to correlate.",
            ));
        }
        return Ok(UncPropagation {
            uncertainties: part.uncertainties,
            ..UncPropagation::default()
        });
    }
    // Built here, not by the caller: the matrix is indexed by source order, and
    // this is the only place that order (`unc_vars`) is decided.
    let correlation = if correlations.is_empty() {
        None
    } else {
        Some(Correlation::build(&part.unc_vars, correlations)?)
    };
    let jacobian = numerical_jacobian(equations, values, ctx, &var_list)?;
    solve_rss_uncertainties(&jacobian, &var_list, part, specs, correlation.as_ref())
}

/// Port of `partitionVariables`.
fn partition_variables(
    var_list: &[String],
    specs: &BTreeMap<String, UncertaintySpec>,
) -> UncPartition {
    let mut unc_vars = Vec::new();
    let mut dep_vars = Vec::new();
    let mut uncertainties = BTreeMap::new();
    for v in var_list {
        uncertainties.insert(v.clone(), 0.0);
        match specs.get(v) {
            // PARITY: the Java guard is `spec.uncertainty() > 0.0`, so a NaN
            // uncertainty is *not* a source. Transcribed as written.
            Some(spec) if spec.uncertainty > 0.0 => unc_vars.push(v.clone()),
            _ => dep_vars.push(v.clone()),
        }
    }
    UncPartition {
        unc_vars,
        dep_vars,
        uncertainties,
    }
}

/// Forward-difference Jacobian of the residual system w.r.t. every variable.
/// Port of `computeNumericalJacobian`, including its sparse
/// variable → dependent-equation index, its `sqrt(ulp(1))` step and its
/// swallow-and-keep-zero behaviour on a perturbed evaluation that throws.
fn numerical_jacobian(
    equations: &[Equation],
    values: &Scope,
    ctx: EvalContext<'_>,
    var_list: &[String],
) -> Result<Mat> {
    let m = equations.len();
    let n = var_list.len();
    let mut jacobian = vec![vec![0.0; n]; m];
    let base_residual = evaluate_system_residuals(equations, values, ctx)?;
    let mut perturbed = values.clone();
    // `Math.sqrt(Math.ulp(1.0))` — `Math.ulp(1.0)` is `f64::EPSILON` (2^-52),
    // so this is exactly 2^-26.
    let eps = f64::EPSILON.sqrt();

    // Precompute the sparse dependency map: variable j → equation indices.
    let eq_vars: Vec<BTreeSet<String>> = equations.iter().map(Equation::variables).collect();
    let var_to_eqs: Vec<Vec<usize>> = var_list
        .iter()
        .map(|name| {
            (0..m)
                .filter(|&i| eq_vars[i].contains(name))
                .collect::<Vec<_>>()
        })
        .collect();

    for (j, name) in var_list.iter().enumerate() {
        let deps = &var_to_eqs[j];
        if deps.is_empty() {
            // Variable appears in no equation; its column stays 0.0.
            continue;
        }
        let x = values.get(name).copied().unwrap_or(1.0);
        let h = eps * x.abs().max(1.0);
        perturbed.insert(name.clone(), x + h);

        for &i in deps {
            let eq = &equations[i];
            let lhs = eval_with(&eq.lhs, &perturbed, ctx);
            let rhs = eval_with(&eq.rhs, &perturbed, ctx);
            if let (Ok(lhs), Ok(rhs)) = (lhs, rhs) {
                jacobian[i][j] = (lhs - rhs - base_residual[i]) / h;
            }
            // else: keep jacobian[i][j] as 0.0 (the Java `catch (Exception ignored)`).
        }

        // Restores the original value — and, when `values` had no entry for the
        // variable, leaves the Java's `1.0` default behind, exactly as
        // `perturbedValues.put(varName, x)` does.
        perturbed.insert(name.clone(), x);
    }
    Ok(jacobian)
}

/// `lhs - rhs` of every equation at `values`. Port of
/// `evaluateSystemResiduals` — deliberately **not** exception-swallowing.
fn evaluate_system_residuals(
    equations: &[Equation],
    values: &Scope,
    ctx: EvalContext<'_>,
) -> Result<Vec<f64>> {
    equations
        .iter()
        .map(|eq| Ok(eval_with(&eq.lhs, values, ctx)? - eval_with(&eq.rhs, values, ctx)?))
        .collect()
}

/// Solves `Jy·dy = −Jx·u` for each source and combines the dependent-variable
/// responses in root-sum-square. Port of `solveRssUncertainties`.
fn solve_rss_uncertainties(
    jacobian: &Mat,
    var_list: &[String],
    part: UncPartition,
    specs: &BTreeMap<String, UncertaintySpec>,
    correlation: Option<&Correlation>,
) -> Result<UncPropagation> {
    let UncPartition {
        unc_vars,
        dep_vars,
        mut uncertainties,
    } = part;
    let m = jacobian.len();
    let p = unc_vars.len();
    let q = dep_vars.len();

    let (jy, jx) = split_jacobian_columns(jacobian, var_list, &dep_vars, &unc_vars, m);
    let non_zero_rows = select_non_zero_rows(&jy, m, q);
    if non_zero_rows.is_empty() {
        set_source_uncertainties(&mut uncertainties, &unc_vars, specs);
        return Ok(UncPropagation {
            uncertainties,
            ..UncPropagation::default()
        });
    }

    let jy_prime: Mat = non_zero_rows.iter().map(|&i| jy[i].clone()).collect();
    let jx_prime: Mat = non_zero_rows.iter().map(|&i| jx[i].clone()).collect();

    let variances = dependent_variances(&jy_prime, &jx_prime, &unc_vars, specs, q, p, correlation)?;
    let mut contributions = BTreeMap::new();
    for (j, dep) in dep_vars.iter().enumerate() {
        let sigma = variances.sum_sq[j].sqrt();
        uncertainties.insert(dep.clone(), sigma);
        if !(sigma > 0.0) {
            // PARITY: `if (sigma <= 0.0) continue;` — kept negated so a NaN
            // sigma also skips the breakdown rather than emitting one.
            continue;
        }
        let mut per_source: Vec<UncertaintyContribution> = (0..p)
            .filter_map(|i| {
                let dy = variances.per_source[j][i];
                (dy != 0.0 && dy.is_finite()).then(|| UncertaintyContribution {
                    source: unc_vars[i].clone(),
                    value: dy,
                })
            })
            .collect();
        if !per_source.is_empty() {
            // Java: `perSource.sort((a, b) -> Double.compare(abs(b), abs(a)))`
            // — a stable sort, so ties keep source order. `sort_by` is stable
            // too.
            per_source.sort_by(|a, b| b.value.abs().total_cmp(&a.value.abs()));
            contributions.insert(dep.clone(), per_source);
        }
    }
    set_source_uncertainties(&mut uncertainties, &unc_vars, specs);
    Ok(UncPropagation {
        uncertainties,
        contributions,
        ..UncPropagation::default()
    })
}

/// Splits the full Jacobian into dependent-variable (`Jy`) and source (`Jx`)
/// columns. Port of `splitJacobianColumns`.
fn split_jacobian_columns(
    jacobian: &Mat,
    var_list: &[String],
    dep_vars: &[String],
    unc_vars: &[String],
    m: usize,
) -> (Mat, Mat) {
    let index: BTreeMap<&str, usize> = var_list
        .iter()
        .enumerate()
        .map(|(j, name)| (name.as_str(), j))
        .collect();
    let pick = |names: &[String]| -> Mat {
        let cols: Vec<usize> = names.iter().map(|name| index[name.as_str()]).collect();
        (0..m)
            .map(|i| cols.iter().map(|&c| jacobian[i][c]).collect())
            .collect()
    };
    (pick(dep_vars), pick(unc_vars))
}

/// Indices of residual rows that actually constrain a dependent variable.
/// Port of `selectNonZeroRows`.
fn select_non_zero_rows(jy: &Mat, m: usize, q: usize) -> Vec<usize> {
    (0..m)
        .filter(|&i| (0..q).any(|j| jy[i][j].abs() >= JY_ROW_ZERO_THRESHOLD))
        .collect()
}

/// Per-dependent-variable variances plus the signed per-source `dy` vectors the
/// RSS folds together. Port of the `DependentVariances` carrier.
struct DependentVariances {
    sum_sq: Vec<f64>,
    /// `per_source[dependent][source]`.
    per_source: Vec<Vec<f64>>,
}

/// Port of `computeDependentVariances`: one SVD of `Jy'`, then one
/// pseudo-inverse apply per source.
fn dependent_variances(
    jy_prime: &Mat,
    jx_prime: &Mat,
    unc_vars: &[String],
    specs: &BTreeMap<String, UncertaintySpec>,
    q: usize,
    p: usize,
    correlation: Option<&Correlation>,
) -> Result<DependentVariances> {
    let solver = SvdSolver::new(jy_prime)?;
    let m_prime = jy_prime.len();
    let mut sum_sq = vec![0.0; q];
    let mut per_source = vec![vec![0.0; p]; q];
    for i in 0..p {
        let u = specs[&unc_vars[i]].uncertainty;
        let b: Vec<f64> = (0..m_prime).map(|k| -jx_prime[k][i] * u).collect();
        let dy = solver.solve(&b);
        for j in 0..q {
            sum_sq[j] += dy[j] * dy[j];
            per_source[j][i] = dy[j];
        }
    }

    // Phase 4.3: with declared correlations the total is the full quadratic
    // form `Σᵢₖ dyᵢ · R[i][k] · dyₖ`, of which the RSS above is the `R = I`
    // case. The independent path is left computing its own sum in its own
    // order — the numbers it produced are pinned by 1,308 golden fixtures, and
    // re-deriving them through a matrix product would move their last bits.
    if let Some(r) = correlation.filter(|c| !c.is_identity()) {
        // The correlation matrix is indexed by the *source* order the caller
        // built it in, which is `unc_vars` — assert rather than assume.
        debug_assert_eq!(r.sources.len(), p);
        for (j, total) in sum_sq.iter_mut().enumerate() {
            let mut acc = 0.0;
            for i in 0..p {
                for k in 0..p {
                    acc += per_source[j][i] * r.matrix[i][k] * per_source[j][k];
                }
            }
            // A quadratic form on a positive semidefinite matrix cannot be
            // negative; rounding can still land a hair below zero.
            *total = acc.max(0.0);
        }
    }
    Ok(DependentVariances { sum_sq, per_source })
}

/// Port of `setSourceUncertainties`: a source reports its *stated* uncertainty,
/// never a propagated one.
fn set_source_uncertainties(
    uncertainties: &mut BTreeMap<String, f64>,
    unc_vars: &[String],
    specs: &BTreeMap<String, UncertaintySpec>,
) {
    for name in unc_vars {
        uncertainties.insert(name.clone(), specs[name].uncertainty);
    }
}

// ---------------------------------------------------------------------------
// The SVD least-squares solver
// ---------------------------------------------------------------------------
//
// Commons Math's `SingularValueDecomposition.getSolver()` lives in
// [`crate::linalg::SvdSolver`]. It used to be a private copy here; ledger item
// 40 gave `solver::newton` a second caller for exactly the same Commons Math
// semantics, and two transcriptions of one Java class is how they drift apart.

// ---------------------------------------------------------------------------
// The two-pass orchestration
// ---------------------------------------------------------------------------

/// The result of a second solve pass: the re-solved state and the
/// re-propagated uncertainties. Port of `EquationSystemSolver.UncertaintyPass`.
#[derive(Debug, Clone, PartialEq)]
pub struct UncertaintyPass {
    pub values: Scope,
    pub propagation: UncPropagation,
}

/// The second solve pass, run when active equations query `UncertaintyOf(X)`.
/// Port of `resolveUncertaintySecondPass`.
///
/// `values` is the **first pass's** state; the first pass's uncertainties are
/// injected into it and it is handed to `resolve` as the warm start. The
/// returned state carries the second pass's uncertainties injected in turn, so
/// a caller that publishes it shows the same `uncertaintyof$…` rows the Java
/// engine does.
///
/// `resolve` must block-and-solve `equations` warm-started from the scope it is
/// given — `EquationSystemSolver.solveEquationList(…, warmStart)`.
///
/// # Errors
///
/// Whatever `resolve` returns, or a propagation failure.
#[allow(clippy::too_many_arguments)]
pub fn resolve_second_pass<F>(
    equations: &[Equation],
    values: &Scope,
    first_pass: &UncPropagation,
    uncertainty_exprs: &BTreeMap<String, Expr>,
    specs: &mut BTreeMap<String, UncertaintySpec>,
    ctx: EvalContext<'_>,
    correlations: &CorrelationEntries,
    resolve: F,
) -> Result<UncertaintyPass>
where
    F: FnOnce(&[Equation], &Scope) -> Result<Scope>,
{
    let mut warm = values.clone();
    inject_uncertainty_values(&mut warm, &first_pass.uncertainties);
    let mut resolved = resolve(equations, &warm)?;
    apply_uncertainty_specs(uncertainty_exprs, &resolved, specs, ctx);
    let mut propagation = propagate(equations, &resolved, specs, ctx, correlations)?;
    propagation.correlations = correlations.clone();
    inject_uncertainty_values(&mut resolved, &propagation.uncertainties);
    Ok(UncertaintyPass {
        values: resolved,
        propagation,
    })
}

/// Evaluate each `DistributionOf(X) = Name(args…)` at the solved state, pin the
/// distribution's own standard deviation as `X`'s uncertainty, and hand the
/// shapes back for the samplers.
///
/// Unlike [`apply_uncertainty_specs`], a declaration that will not evaluate is
/// an **error** here rather than a silent skip. That asymmetry is deliberate:
/// the skip exists for Java parity on a construct that predates this engine, and
/// carrying it into new syntax would mean a typo'd distribution quietly reverts
/// to a normal.
///
/// # Errors
///
/// [`FreesError::Solver`] when an argument will not evaluate, the shape is
/// unknown or ill-parameterized, or the same variable also carries an
/// `UncertaintyOf` declaration — two different statements of one variable's
/// spread, with no defensible precedence between them.
fn apply_distribution_specs(
    declarations: &DeclarationSet,
    values: &Scope,
    specs: &mut BTreeMap<String, UncertaintySpec>,
    ctx: EvalContext<'_>,
) -> Result<BTreeMap<String, Distribution>> {
    let mut out = BTreeMap::new();
    for (name, (function, args)) in &declarations.distribution_exprs {
        if declarations.uncertainty_exprs.contains_key(name.as_str()) {
            return Err(crate::diag::FreesError::solver(format!(
                "`{name}` declares both UncertaintyOf and DistributionOf. State the \
                 spread once: a distribution already carries its own standard deviation."
            )));
        }
        let mut evaluated = Vec::with_capacity(args.len());
        for arg in args {
            evaluated.push(eval_with(arg, values, ctx).map_err(|e| {
                crate::diag::FreesError::solver(format!(
                    "DistributionOf({name}): {}",
                    e.to_string_message()
                ))
            })?);
        }
        let distribution = Distribution::from_call(function.as_str(), &evaluated)?;
        let entry = specs.entry(name.clone()).or_default();
        entry.uncertainty = distribution.std_dev();
        out.insert(name.clone(), distribution);
    }
    Ok(out)
}

/// Evaluate each `Correlation(A, B) = expr` at the solved state.
///
/// # Errors
///
/// [`FreesError::Solver`] when a coefficient will not evaluate. Range and
/// definiteness are checked later, by [`Correlation::build`], which is the only
/// place that knows the full source set.
fn evaluate_correlations(
    declarations: &DeclarationSet,
    values: &Scope,
    ctx: EvalContext<'_>,
) -> Result<CorrelationEntries> {
    let mut out = CorrelationEntries::new();
    for ((a, b), expr) in &declarations.correlation_exprs {
        let rho = eval_with(expr, values, ctx).map_err(|e| {
            crate::diag::FreesError::solver(format!(
                "Correlation({a}, {b}): {}",
                e.to_string_message()
            ))
        })?;
        out.insert((a.clone(), b.clone()), rho);
    }
    Ok(out)
}

/// The whole mechanism, in the Java `EquationSystemSolver.solve` order.
///
/// Given the **active** equations (the declarations already lifted out by
/// [`extract_uncertainty_equations`]) and the first solve's values, this
///
/// 1. pins each `UncertaintyOf(X) = expr` into `specs`
///    ([`apply_uncertainty_specs`]) and each `DistributionOf(X) = …` alongside
///    it ([`apply_distribution_specs`]),
/// 2. propagates ([`propagate`]) — with the declared correlations folded into
///    the quadratic form when there are any,
/// 3. and, when an active equation *queries* `UncertaintyOf(...)`, runs the
///    second pass ([`resolve_second_pass`]) and returns its state and
///    propagation instead.
///
/// `values` is updated in place to whatever state the caller should publish.
///
/// # Errors
///
/// Whatever `resolve` returns, a malformed declaration, or a propagation
/// failure.
pub fn analyze<F>(
    equations: &[Equation],
    values: &mut Scope,
    specs: &mut BTreeMap<String, UncertaintySpec>,
    declarations: &DeclarationSet,
    ctx: EvalContext<'_>,
    resolve: F,
) -> Result<UncPropagation>
where
    F: FnOnce(&[Equation], &Scope) -> Result<Scope>,
{
    apply_uncertainty_specs(&declarations.uncertainty_exprs, values, specs, ctx);
    let distributions = apply_distribution_specs(declarations, values, specs, ctx)?;
    let correlations = evaluate_correlations(declarations, values, ctx)?;
    let mut propagation = propagate(equations, values, specs, ctx, &correlations)?;
    if !mentions_uncertainty_of(equations) {
        propagation.distributions = distributions;
        propagation.correlations = correlations;
        return Ok(propagation);
    }
    let pass = resolve_second_pass(
        equations,
        values,
        &propagation,
        &declarations.uncertainty_exprs,
        specs,
        ctx,
        &correlations,
        resolve,
    )?;
    *values = pass.values;
    let mut propagation = pass.propagation;
    propagation.distributions = distributions;
    propagation.correlations = correlations;
    Ok(propagation)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::defs::Definitions;
    use crate::parser::parse_document;

    /// Parse a document and split it the way `EquationSystemSolver.solve` does.
    fn split(source: &str) -> (ExtractedUncertainties, Definitions) {
        let doc = parse_document(source).expect("parse");
        let equations = crate::parser::expand::expand_document(&doc).expect("expand");
        let equations = crate::integral::hoist_nested(equations);
        (extract_uncertainty_equations(&equations), doc.defs)
    }

    /// Solve the active half through the real engine, then hand the state to the
    /// uncertainty engine. The engine cannot be asked to solve the *original*
    /// text: `UncertaintyOf(x) = 0.1` is not an equation, and the extraction
    /// that removes it is what this module provides.
    fn solved_values(active: &[Equation]) -> Scope {
        let source = active
            .iter()
            .map(|eq| eq.source_text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        let solution = crate::engine::solve(&source, &crate::solver::SolverSettings::default())
            .unwrap_or_else(|e| panic!("solve failed for {source:?}: {e}"));
        solution.values.into_iter().collect()
    }

    fn close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() <= 1e-12 * expected.abs().max(1.0),
            "expected {expected}, got {actual}"
        );
    }

    fn sources(list: &[UncertaintyContribution]) -> Vec<&str> {
        list.iter().map(|c| c.source.as_str()).collect()
    }

    // -- extraction ------------------------------------------------------

    #[test]
    fn uncertainty_declarations_leave_the_system() {
        let (ext, _) = split("UncertaintyOf(x) = 0.1\nx = 2\ny = x^2\n");
        assert_eq!(ext.active_equations.len(), 2);
        assert_eq!(ext.declarations.uncertainty_exprs.len(), 1);
        assert!(ext.declarations.uncertainty_exprs.contains_key("x"));
        assert!(!mentions_uncertainty_of(&ext.active_equations));
    }

    #[test]
    fn a_query_inside_an_active_equation_stays_and_is_detected() {
        let (ext, _) = split("UncertaintyOf(r) = 0.01\nr = 2.5\nrel = UncertaintyOf(r) / r\n");
        assert_eq!(ext.active_equations.len(), 2);
        assert!(mentions_uncertainty_of(&ext.active_equations));
    }

    #[test]
    fn a_string_argument_names_the_same_variable() {
        let e = Expr::call("UncertaintyOf", vec![Expr::Str("Temp".into())]);
        assert_eq!(uncertainty_target(&e), Some("Temp"));
        let e = Expr::call("UncertaintyOf", vec![Expr::var("Temp")]);
        assert_eq!(uncertainty_target(&e), Some("temp"));
        // Wrong arity is not a declaration.
        assert_eq!(
            uncertainty_target(&Expr::call("uncertaintyof", vec![])),
            None
        );
    }

    // -- propagation, against the Java oracle ----------------------------
    //
    // Every expected number below came from running the reference engine
    // (`tools/golden-dumper/classpath.sh` + a probe calling
    // `EquationSystemSolver.solve`) on the same document. They are quoted at
    // full precision, forward-difference truncation error included — the
    // `0.40000000298023225` below is *not* 0.4 by accident.

    #[test]
    fn oracle_square_law() {
        // UncertaintyOf(x) = 0.1 / x = 2 / y = x^2
        //   unc x = 0.1, unc y = 0.40000000298023225
        //   con y : x = 0.40000000298023225
        let (ext, defs) = split("UncertaintyOf(x) = 0.1\nx = 2\ny = x^2\n");
        let values = solved_values(&ext.active_equations);
        let mut specs = BTreeMap::new();
        apply_uncertainty_specs(
            &ext.declarations.uncertainty_exprs,
            &values,
            &mut specs,
            EvalContext::with_defs(&defs),
        );
        assert_eq!(specs["x"].uncertainty, 0.1);

        let unc = propagate(
            &ext.active_equations,
            &values,
            &specs,
            EvalContext::with_defs(&defs),
            &CorrelationEntries::new(),
        )
        .expect("propagate");
        assert_eq!(unc.uncertainties["x"], 0.1);
        close(unc.uncertainties["y"], 0.40000000298023225);
        let con = &unc.contributions["y"];
        assert_eq!(con.len(), 1);
        assert_eq!(con[0].source, "x");
        close(con[0].value, 0.40000000298023225);
        assert!(!unc.contributions.contains_key("x"));
    }

    #[test]
    fn oracle_two_sources_combine_in_root_sum_square() {
        // a=3 (±0.05), b=4 (±0.20), c = a*b, d = a + 2b
        //   unc c = 0.632455532033676   con c : b=0.6000000000000001, a=0.2
        //   unc d = 0.4031128874149275  con d : b=0.4, a=0.05
        let (ext, defs) = split(
            "UncertaintyOf(a) = 0.05\nUncertaintyOf(b) = 0.20\na = 3\nb = 4\nc = a * b\nd = a + 2 * b\n",
        );
        let values = solved_values(&ext.active_equations);
        let mut specs = BTreeMap::new();
        apply_uncertainty_specs(
            &ext.declarations.uncertainty_exprs,
            &values,
            &mut specs,
            EvalContext::with_defs(&defs),
        );
        let unc = propagate(
            &ext.active_equations,
            &values,
            &specs,
            EvalContext::with_defs(&defs),
            &CorrelationEntries::new(),
        )
        .expect("propagate");

        close(unc.uncertainties["c"], 0.632455532033676);
        close(unc.uncertainties["d"], 0.4031128874149275);
        assert_eq!(unc.uncertainties["a"], 0.05);
        assert_eq!(unc.uncertainties["b"], 0.20);

        // Ranked by |contribution|, descending — the tornado order.
        let c = &unc.contributions["c"];
        assert_eq!(sources(c), ["b", "a"]);
        close(c[0].value, 0.6000000000000001);
        close(c[1].value, 0.2);
        let d = &unc.contributions["d"];
        assert_eq!(sources(d), ["b", "a"]);
        close(d[0].value, 0.4);
        close(d[1].value, 0.05);
    }

    #[test]
    fn oracle_kinetic_energy() {
        // m=1.2 (±0.002), v=25 (±0.5), ke = 0.5 m v^2, p = m v
        //   unc ke = 15.013015263058197  con ke : v=15.000000076293945, m=0.625
        //   unc p  = 0.6020797289396148  con p  : v=0.6, m=0.05
        let (ext, defs) = split(
            "UncertaintyOf(m) = 0.002\nUncertaintyOf(v) = 0.5\nm = 1.2\nv = 25\nke = 0.5 * m * v^2\np = m * v\n",
        );
        let values = solved_values(&ext.active_equations);
        let mut specs = BTreeMap::new();
        apply_uncertainty_specs(
            &ext.declarations.uncertainty_exprs,
            &values,
            &mut specs,
            EvalContext::with_defs(&defs),
        );
        let unc = propagate(
            &ext.active_equations,
            &values,
            &specs,
            EvalContext::with_defs(&defs),
            &CorrelationEntries::new(),
        )
        .expect("propagate");

        close(unc.uncertainties["ke"], 15.013015263058197);
        close(unc.uncertainties["p"], 0.6020797289396148);
        let ke = &unc.contributions["ke"];
        assert_eq!(sources(ke), ["v", "m"]);
        close(ke[0].value, 15.000000076293945);
        close(ke[1].value, 0.625);
        let p = &unc.contributions["p"];
        close(p[0].value, 0.6);
        close(p[1].value, 0.05);
    }

    #[test]
    fn oracle_linear_gain() {
        // x = 2 (±0.1), y = 3x. The reference engine's Monte Carlo probe reports
        // this document's `base.uncertainties()` as y = 0.30000000000000004 —
        // the forward-difference Jacobian's answer, not the exact 0.3.
        let (ext, defs) = split("x = 2\ny = 3 * x\n");
        let values = solved_values(&ext.active_equations);
        let specs = BTreeMap::from([("x".to_string(), UncertaintySpec::with_uncertainty(0.1))]);
        let unc = propagate(
            &ext.active_equations,
            &values,
            &specs,
            EvalContext::with_defs(&defs),
            &CorrelationEntries::new(),
        )
        .expect("propagate");
        assert_eq!(unc.uncertainties["x"], 0.1);
        assert_eq!(unc.uncertainties["y"], 0.30000000000000004);
    }

    #[test]
    fn oracle_no_source_means_every_variable_reports_zero() {
        // x = 1 / y = 2x with no declared uncertainty: unc x = 0, unc y = 0.
        let (ext, defs) = split("x = 1\ny = 2 * x\n");
        let values = solved_values(&ext.active_equations);
        let unc = propagate(
            &ext.active_equations,
            &values,
            &BTreeMap::new(),
            EvalContext::with_defs(&defs),
            &CorrelationEntries::new(),
        )
        .expect("propagate");
        assert_eq!(unc.uncertainties["x"], 0.0);
        assert_eq!(unc.uncertainties["y"], 0.0);
        assert!(unc.contributions.is_empty());
    }

    #[test]
    fn oracle_second_pass_resolves_an_uncertaintyof_query() {
        // UncertaintyOf(r) = 0.01 / r = 2.5 / area = pi r^2
        //   rel = UncertaintyOf(area) / area
        // Java:
        //   area = 19.634954084936187   unc area = 0.15707963554051246
        //   rel  = 0.008000000145710704 unc rel  = 6.400000141154621E-5
        //   con rel : r = -6.400000141154621E-5
        //   and the value map gains uncertaintyof$area / uncertaintyof$r /
        //   uncertaintyof$rel rows.
        let (ext, defs) = split(
            "UncertaintyOf(r) = 0.01\nr = 2.5\narea = 3.14159265358979 * r^2\nrel = UncertaintyOf(area) / area\n",
        );
        assert!(mentions_uncertainty_of(&ext.active_equations));

        // First pass: `UncertaintyOf(area)` reads 0.0 (nothing injected yet), so
        // `rel` solves to 0 — exactly the state the Java first pass reaches.
        let mut values = solved_values(&ext.active_equations);
        let mut specs = BTreeMap::new();
        let unc = analyze(
            &ext.active_equations,
            &mut values,
            &mut specs,
            &ext.declarations,
            EvalContext::with_defs(&defs),
            // The second-pass re-solve. `engine::solve_equation_list` goes here
            // once this is wired into `engine::solve_with` — but it is private
            // *and* it is missing the `uncertaintyof$…` carry-over that
            // [`carry_uncertainty_entries`] documents, so the test supplies the
            // stand-in below.
            //
            // The stand-in is a forward sweep, not a solver: for this document
            // Tarjan produces three singleton blocks in source order
            // (`r`, then `area`, then `rel`), each an explicit `var = expr`,
            // and Newton on an explicit assignment lands on the RHS in one
            // step. So the sweep reaches the same fixed point the engine would
            // — with the injected scope visible, which is the whole point.
            |equations, warm| {
                let mut values = warm.clone();
                carry_uncertainty_entries(warm, &mut values);
                for eq in equations {
                    let Expr::Var(name) = &eq.lhs else {
                        panic!("stand-in only handles explicit assignments");
                    };
                    let value = eval_with(&eq.rhs, &values, EvalContext::with_defs(&defs))?;
                    values.insert(name.clone(), value);
                }
                Ok(values)
            },
        )
        .expect("analyze");

        close(unc.uncertainties["area"], 0.15707963554051246);
        close(unc.uncertainties["rel"], 6.400000141154621e-5);
        assert_eq!(unc.uncertainties["r"], 0.01);
        close(values["rel"], 0.008000000145710704);
        close(values["uncertaintyof$area"], 0.15707963554051246);
        assert_eq!(values["uncertaintyof$r"], 0.01);
        let rel = &unc.contributions["rel"];
        assert_eq!(rel.len(), 1);
        close(rel[0].value, -6.400000141154621e-5);
    }

    // -- unit-level checks on the pieces ---------------------------------

    #[test]
    fn the_svd_solver_is_a_pseudo_inverse_not_a_singularity_error() {
        // A rank-1 2x2: the Gauss-Newton system is singular, and Commons Math's
        // SVD solver answers with the minimum-norm least-squares solution
        // rather than failing.
        let a = vec![vec![1.0, 2.0], vec![2.0, 4.0]];
        let solver = SvdSolver::new(&a).expect("svd");
        let x = solver.solve(&[5.0, 10.0]);
        // A·x must reproduce b, and x must lie in the row space (x = t·[1,2]).
        close(a[0][0] * x[0] + a[0][1] * x[1], 5.0);
        close(a[1][0] * x[0] + a[1][1] * x[1], 10.0);
        close(x[1] / x[0], 2.0);
    }

    #[test]
    fn rows_that_do_not_constrain_a_dependent_variable_are_dropped() {
        let jy = vec![vec![0.0, 5e-13], vec![1.0, 0.0], vec![0.0, 0.0]];
        assert_eq!(select_non_zero_rows(&jy, 3, 2), vec![1]);
    }

    #[test]
    fn injected_keys_use_the_java_prefix_and_lowercase() {
        let mut scope = Scope::default();
        inject_uncertainty_values(&mut scope, &BTreeMap::from([("Temp".to_string(), 0.5)]));
        assert_eq!(scope.get("uncertaintyof$temp"), Some(&0.5));
    }

    #[test]
    fn an_unevaluatable_declaration_is_skipped_not_fatal() {
        let exprs = BTreeMap::from([("x".to_string(), Expr::var("nothing_here"))]);
        let mut specs = BTreeMap::new();
        apply_uncertainty_specs(
            &exprs,
            &Scope::default(),
            &mut specs,
            EvalContext::with_defs(&Definitions::default()),
        );
        assert!(specs.is_empty());
    }

    #[test]
    fn a_variable_in_no_equation_keeps_a_zero_jacobian_column() {
        // `x` is constrained by the one equation; `z` appears nowhere.
        let equations = vec![Equation::new(Expr::var("x"), Expr::num(2.0), "x = 2")];
        let values: Scope = [("x".to_string(), 2.0)].into_iter().collect();
        let jac = numerical_jacobian(
            &equations,
            &values,
            EvalContext::with_defs(&Definitions::default()),
            &["x".to_string(), "z".to_string()],
        )
        .expect("jacobian");
        close(jac[0][0], 1.0);
        assert_eq!(jac[0][1], 0.0);
    }
}
