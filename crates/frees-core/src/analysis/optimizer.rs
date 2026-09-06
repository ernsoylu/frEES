//! Calculate ▸ Min/Max — optimisation of an objective variable by manipulating
//! one or more independent variables inside given bounds.
//!
//! Port of `../frEES/backend/core/src/main/java/com/frees/backend/core/Optimizer.java`
//! (719 LOC), whose search strategies are Apache Commons Math 3.6.1
//! (`BrentOptimizer`, `SimplexOptimizer` + `NelderMeadSimplex`, `BOBYQAOptimizer`).
//! Since the Java delegates the *search* to Commons Math, a faithful port has to
//! carry those algorithms too — the answer alone is not the contract, the
//! sequence of points probed is, because every probe is a full system solve and
//! `evaluations` is a reported field.
//!
//! # What drives what
//!
//! Nothing here re-implements Newton. One "objective evaluation" is
//! [`crate::engine::solve_with`] over the document text with the decision
//! variables appended as `name = value` equations — exactly the Java
//! `solveWithDecisions`, which builds the same augmented text and calls
//! `EquationSystemSolver.solve`. The optimiser only chooses *where* to solve.
//!
//! # Modes (`Problem::method`)
//!
//! | value | Java | here |
//! |---|---|---|
//! | `None` / `"brent"` with one decision | `BrentOptimizer(1e-10, 1e-12)` | [`brent_optimize`] — a full port |
//! | anything else (`"simplex"`, `"nelder-mead"`, …) | `SimplexOptimizer(1e-10, 1e-12)` + `NelderMeadSimplex(n)` | [`nelder_mead`] — a full port |
//! | `"bobyqa"` | `BOBYQAOptimizer(2n+1)` | **not ported** — see below |
//!
//! ## The BOBYQA gap
//!
//! Powell's BOBYQA is ~1,300 lines of trust-region quadratic-model code in
//! Commons Math and is *not* ported. `"bobyqa"` runs the Nelder–Mead search
//! instead, with the Java's BOBYQA branch semantics preserved where they are
//! observable from outside: bounds are enforced by projection rather than by the
//! smooth out-of-box penalty ("BOBYQA enforces bounds natively and skips this"),
//! and the answer is clamped into the box before the final solve. Expect the
//! same optimum on smooth problems and a **different** `evaluations` count and
//! different trailing digits. This is the one deliberate divergence in this
//! module.
//!
//! # Exactness notes
//!
//! * `SearchInterval(lo, hi)` starts Brent at the **midpoint** `lo + 0.5*(hi-lo)`,
//!   not at the spec's guess — that is Commons Math, and it is why a 1-D run
//!   ignores `GUESS`.
//! * `AbstractSimplex(n)` builds a *triangular* start simplex (vertex `i+1` is
//!   the start point with the first `i+1` coordinates each stepped by `1.0`),
//!   not a set of unit vectors. [`build_simplex`] reproduces the exact
//!   `System.arraycopy` construction.
//! * `MaxEval` is enforced Apache-style: the counter increments *before* the
//!   function is called, so the budget is exactly `max` successful evaluations
//!   and the aborted call never reaches the solver.
//! * `Math.max`/`FastMath.max` propagate NaN; Rust's `f64::max` does not.
//!   [`jmax`]/[`jmin`] are used at every site the Java clamps or compares.
//!
//! # Parity rule
//!
//! Constants are transcribed from the Java as written (`PENALTY = 1e100`,
//! `BOUNDS_PENALTY_WEIGHT = 1e8`, …) and are deliberately not "cleaned up".

// The Java transcription keeps its own shape: several routines legitimately
// take the whole (problem, multipliers, penalties, warm start) tuple because
// that is one Java method signature, and bundling them into a struct would
// make the correspondence to `Optimizer.java` harder to audit, not easier.
#![allow(clippy::too_many_arguments)]

use std::cmp::Ordering;

use crate::diag::{FreesError, Result};
use crate::engine::{variable_override_spec, Solution, VariableOverride};
use crate::solver::SolverSettings;

// ---------------------------------------------------------------------------
// Constants — transcribed from Optimizer.java verbatim
// ---------------------------------------------------------------------------

/// The value a failed solve reports, so an infeasible probe is never chosen.
const PENALTY: f64 = 1e100;
/// `MaxEval` for the 1-D Brent search.
const MAX_EVALUATIONS: usize = 500;
/// Each evaluation is a full system solve; multivariate searches (and the
/// penalized sub-problems of constrained runs) need a larger budget than
/// 1-D Brent before they converge.
const MULTIVARIATE_MAX_EVALUATIONS: usize = 2000;
/// Weight of the quadratic out-of-bounds penalty used to keep the
/// bounds-unaware Nelder-Mead simplex inside the box.
const BOUNDS_PENALTY_WEIGHT: f64 = 1e8;
const CONSTRAINED_MAX_OUTER_ITERATIONS: usize = 15;
const BARRIER_MU_INITIAL: f64 = 1.0;
const BARRIER_MU_FACTOR: f64 = 0.1;
const BARRIER_MU_MIN: f64 = 1e-10;
const LAGRANGIAN_RHO_INITIAL: f64 = 1.0;
const LAGRANGIAN_RHO_FACTOR: f64 = 10.0;
const LAGRANGIAN_RHO_MAX: f64 = 1e6;
const CONSTRAINT_TOLERANCE: f64 = 1e-6;

/// `BrentOptimizer(1e-10, 1e-12)` — the relative threshold of the 1-D search.
const BRENT_RELATIVE_THRESHOLD: f64 = 1e-10;
/// `BrentOptimizer(1e-10, 1e-12)` — the absolute threshold of the 1-D search.
const BRENT_ABSOLUTE_THRESHOLD: f64 = 1e-12;
/// `SimplexOptimizer(1e-10, 1e-12)` — `SimpleValueChecker` relative threshold.
const SIMPLEX_RELATIVE_THRESHOLD: f64 = 1e-10;
/// `SimplexOptimizer(1e-10, 1e-12)` — `SimpleValueChecker` absolute threshold.
const SIMPLEX_ABSOLUTE_THRESHOLD: f64 = 1e-12;

// ---------------------------------------------------------------------------
// Problem / result records
// ---------------------------------------------------------------------------

/// One optimisation request — the Java `Optimizer.Problem` record.
///
/// `text`, `settings` and `overrides` are what the objective evaluation replays
/// through [`crate::engine::solve_with`]; `overrides` stands in for the Java
/// `Map<String, VariableSpec> specs` (both come from the Variable Information
/// window).
#[derive(Debug, Clone, PartialEq)]
pub struct Problem {
    /// The document, verbatim. Decision assignments are appended to it.
    pub text: String,
    pub settings: SolverSettings,
    /// Per-variable guesses/bounds — the Java `specs` map.
    pub overrides: Vec<VariableOverride>,
    /// Name of the variable being minimised/maximised.
    pub objective: String,
    /// The independent variables, in request order.
    pub decisions: Vec<String>,
    pub lowers: Vec<f64>,
    pub uppers: Vec<f64>,
    /// `"brent"`, `"bobyqa"`, or anything else (⇒ Nelder–Mead). `None` behaves
    /// exactly as `Some("brent")` — the Java `method == null` branch.
    pub method: Option<String>,
    pub maximize: bool,
    /// `expr <= value` / `expr >= value` / `expr = value`, RHS a numeric
    /// constant. Empty for an unconstrained run.
    pub constraints: Vec<String>,
}

impl Problem {
    /// The Java backwards-compatibility constructor for a 1-D problem: one
    /// decision, `"brent"`, no constraints.
    pub fn univariate(
        text: impl Into<String>,
        settings: SolverSettings,
        overrides: Vec<VariableOverride>,
        objective: impl Into<String>,
        decision: impl Into<String>,
        lower: f64,
        upper: f64,
        maximize: bool,
    ) -> Problem {
        Problem {
            text: text.into(),
            settings,
            overrides,
            objective: objective.into(),
            decisions: vec![decision.into()],
            lowers: vec![lower],
            uppers: vec![upper],
            method: Some("brent".to_string()),
            maximize,
            constraints: Vec::new(),
        }
    }

    /// The Java `Problem.decision()` — the first decision variable.
    pub fn decision(&self) -> Option<&str> {
        self.decisions.first().map(String::as_str)
    }

    /// The Java `Problem.hasConstraints()`.
    pub fn has_constraints(&self) -> bool {
        !self.constraints.is_empty()
    }

    fn is_bobyqa(&self) -> bool {
        self.method
            .as_deref()
            .is_some_and(|m| m.eq_ignore_ascii_case("bobyqa"))
    }

    fn is_brent(&self) -> bool {
        match self.method.as_deref() {
            None => true,
            Some(m) => m.eq_ignore_ascii_case("brent"),
        }
    }
}

/// What one optimisation produced — the Java `Optimizer.OptimizeResult`.
#[derive(Debug, Clone, PartialEq)]
pub struct OptimizeResult {
    /// The optimal decision vector, aligned with [`Problem::decisions`].
    pub decision_values: Vec<f64>,
    /// The objective read out of the final solve (not the search's internal
    /// penalised value).
    pub objective_value: f64,
    /// How many objective evaluations (full system solves) the search spent.
    pub evaluations: usize,
    /// The full solve at the optimum, so the caller can show every variable.
    pub solution: Box<Solution>,
    /// Set when a constraint is still violated at the returned point — the
    /// Java refuses to present a point that silently breaks a constraint as
    /// the optimum.
    pub warning: Option<String>,
}

impl OptimizeResult {
    /// The Java `OptimizeResult.decisionValue()` — the first decision's value.
    pub fn decision_value(&self) -> f64 {
        self.decision_values.first().copied().unwrap_or(0.0)
    }
}

// ---------------------------------------------------------------------------
// Constraints
// ---------------------------------------------------------------------------

/// A parsed constraint `lhsExpr op rhsValue`, normalised to `g(x) <= 0`
/// (inequality) or `h(x) = 0` (equality) — the Java `ParsedConstraint`.
#[derive(Debug, Clone, PartialEq)]
struct ParsedConstraint {
    index: usize,
    lhs_expr: String,
    operator: ConstraintOp,
    rhs_value: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConstraintOp {
    Le,
    Ge,
    Eq,
}

impl ConstraintOp {
    fn as_str(self) -> &'static str {
        match self {
            ConstraintOp::Le => "<=",
            ConstraintOp::Ge => ">=",
            ConstraintOp::Eq => "=",
        }
    }
}

impl ParsedConstraint {
    /// `g(x)` in `g(x) <= 0` form for inequalities, `h(x)` in `h(x) = 0` form
    /// for equalities, given the evaluated LHS.
    fn normalised(&self, lhs_eval: f64) -> f64 {
        match self.operator {
            // lhs <= rhs  ⟹  lhs - rhs <= 0
            ConstraintOp::Le => lhs_eval - self.rhs_value,
            // lhs >= rhs  ⟹  rhs - lhs <= 0
            ConstraintOp::Ge => self.rhs_value - lhs_eval,
            // lhs = rhs   ⟹  lhs - rhs = 0
            ConstraintOp::Eq => lhs_eval - self.rhs_value,
        }
    }

    fn is_equality(&self) -> bool {
        self.operator == ConstraintOp::Eq
    }
}

/// The Java `CONSTRAINT_PATTERN` = `^([^<>=]+)(<=|>=|=)(.+)$`, hand-rolled
/// because the port takes no regex dependency.
///
/// `[^<>=]+` cannot cross `<`, `>` or `=`, so the operator is always at the
/// **first** occurrence of one of those three characters, and that occurrence
/// must be at index ≥ 1 (the LHS needs at least one character). `(.+)$` is
/// greedy and swallows the rest, so a second operator lands in the RHS and
/// fails the numeric parse — exactly as the Java does.
fn split_constraint(trimmed: &str) -> Option<(&str, ConstraintOp, &str)> {
    let at = trimmed.find(['<', '>', '='])?;
    if at == 0 {
        return None; // `[^<>=]+` needs at least one character
    }
    let rest = &trimmed[at..];
    let (operator, width) = if rest.starts_with("<=") {
        (ConstraintOp::Le, 2)
    } else if rest.starts_with(">=") {
        (ConstraintOp::Ge, 2)
    } else if rest.starts_with('=') {
        (ConstraintOp::Eq, 1)
    } else {
        return None; // a bare `<` or `>` matches no alternative
    };
    let rhs = &trimmed[at + width..];
    if rhs.is_empty() {
        return None; // `(.+)$` needs at least one character
    }
    Some((&trimmed[..at], operator, rhs))
}

/// The Java `Optimizer.parseConstraints`.
fn parse_constraints(raw: &[String]) -> Result<Vec<ParsedConstraint>> {
    let mut parsed = Vec::with_capacity(raw.len());
    for constraint in raw {
        let trimmed = constraint.trim();
        let Some((lhs, operator, rhs)) = split_constraint(trimmed) else {
            return Err(FreesError::solver(format!(
                "Cannot parse constraint: '{constraint}'. Expected format: \
                 'expr <= value', 'expr >= value', or 'expr = value'."
            )));
        };
        let rhs = rhs.trim();
        let Ok(rhs_value) = rhs.parse::<f64>() else {
            return Err(FreesError::solver(format!(
                "Constraint RHS must be a numeric constant, got: '{rhs}' \
                 in constraint '{constraint}'."
            )));
        };
        let index = parsed.len();
        parsed.push(ParsedConstraint {
            index,
            lhs_expr: lhs.trim().to_string(),
            operator,
            rhs_value,
        });
    }
    Ok(parsed)
}

/// Selects a temporary variable prefix for constraint LHS expressions that is
/// guaranteed not to collide with any identifier in the problem text.
fn choose_constraint_prefix(text: &str) -> String {
    let base = "zz_opt_con_";
    if !text.contains(base) {
        return base.to_string();
    }
    let mut i = 1;
    loop {
        let cand = format!("zz_opt_con_{i}_");
        if !text.contains(&cand) {
            return cand;
        }
        i += 1;
    }
}

/// Constraint tolerance relative to the magnitude of the RHS constant.
fn tolerance(c: &ParsedConstraint) -> f64 {
    CONSTRAINT_TOLERANCE * jmax(1.0, c.rhs_value.abs())
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Optimise `problem` — the Java `Optimizer.optimize`.
///
/// # Errors
///
/// * [`FreesError::Solver`] for a malformed request (blank objective, no
///   decisions, objective == a decision, missing/crossed/non-finite bounds) —
///   the Java `validate`;
/// * [`FreesError::Solver`] when the objective variable is not part of the
///   solution at the optimum;
/// * whatever the final solve raises (the search itself swallows failed probes
///   as [`PENALTY`], exactly as the Java `catch (ParseException | SolverException)`
///   does).
pub fn optimize(problem: &Problem) -> Result<OptimizeResult> {
    validate(problem)?;
    if problem.has_constraints() {
        constrained_optimize(problem)
    } else {
        unconstrained_optimize(problem)
    }
}

/// The Java `Optimizer.validate`.
fn validate(problem: &Problem) -> Result<()> {
    if problem.objective.trim().is_empty() || problem.decisions.is_empty() {
        return Err(FreesError::solver(
            "Choose both an objective variable and at least one independent variable.",
        ));
    }
    for dec in &problem.decisions {
        if dec.trim().is_empty() {
            return Err(FreesError::solver("Independent variables cannot be blank."));
        }
        if problem.objective.eq_ignore_ascii_case(dec) {
            return Err(FreesError::solver(
                "The objective and the independent variables must differ.",
            ));
        }
    }
    if problem.lowers.len() != problem.decisions.len()
        || problem.uppers.len() != problem.decisions.len()
    {
        return Err(FreesError::solver(
            "Each independent variable requires lower and upper bounds.",
        ));
    }
    for i in 0..problem.decisions.len() {
        let lo = problem.lowers[i];
        let hi = problem.uppers[i];
        // Transcribed as written: the Java guard is `!isFinite || lo >= hi`,
        // which rejects NaN through the finiteness test.
        if !lo.is_finite() || !hi.is_finite() || lo >= hi {
            return Err(FreesError::solver(format!(
                "Optimization requires finite bounds with lower < upper for variable {}",
                problem.decisions[i]
            )));
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Unconstrained
// ---------------------------------------------------------------------------

/// The Java `unconstrainedOptimize`.
fn unconstrained_optimize(problem: &Problem) -> Result<OptimizeResult> {
    if problem.decisions.len() == 1 && problem.is_brent() {
        let mut ctx = Ctx::new(problem, &[], "", MAX_EVALUATIONS);
        // `SearchInterval(lo, hi)` starts at the midpoint; Brent never sees
        // the spec's guess in the 1-D path.
        let point = brent_optimize(
            &mut ctx,
            problem.lowers[0],
            problem.uppers[0],
            !problem.maximize,
        )
        // Java lets both exceptions escape the 1-D path (there is no catch
        // around `brent.optimize`), so both become the caller's error.
        .map_err(|abort| match abort {
            EvalAbort::Fatal(error) => error,
            EvalAbort::Budget => FreesError::solver(format!(
                "Optimization exceeded the evaluation budget of {MAX_EVALUATIONS} solves."
            )),
        })?;

        let decision_values = vec![point];
        let solution = solve_with_decisions(problem, &decision_values)?;
        let objective_value = read_objective(&solution, &problem.objective)?;
        return Ok(OptimizeResult {
            decision_values,
            objective_value,
            evaluations: ctx.evaluations,
            solution: Box::new(solution),
            warning: None,
        });
    }
    multivariate_optimize(problem, &[], "", &[], 0.0, &[], 0.0, None, None)
}

// ---------------------------------------------------------------------------
// Constrained: log-barrier (inequalities) + augmented Lagrangian (equalities)
// ---------------------------------------------------------------------------

/// The Java `constrainedOptimize`: wrap the objective in penalty terms and
/// iteratively tighten the barrier parameter μ and the Lagrangian weight ρ
/// until every constraint holds within tolerance.
fn constrained_optimize(problem: &Problem) -> Result<OptimizeResult> {
    let con_var_prefix = choose_constraint_prefix(&problem.text);
    let all_constraints = parse_constraints(&problem.constraints)?;
    let inequalities: Vec<ParsedConstraint> = all_constraints
        .iter()
        .filter(|c| !c.is_equality())
        .cloned()
        .collect();
    let equalities: Vec<ParsedConstraint> = all_constraints
        .iter()
        .filter(|c| c.is_equality())
        .cloned()
        .collect();

    let mut lambda = vec![0.0f64; equalities.len()];
    let mut rho = if equalities.is_empty() {
        0.0
    } else {
        LAGRANGIAN_RHO_INITIAL
    };
    let mut mu = if inequalities.is_empty() {
        0.0
    } else {
        BARRIER_MU_INITIAL
    };

    let mut best_point = initial_guess(problem);
    let mut total_evaluations = 0usize;
    let mut last_inner: Option<OptimizeResult> = None;

    for _outer in 0..CONSTRAINED_MAX_OUTER_ITERATIONS {
        let inner = multivariate_optimize(
            problem,
            &all_constraints,
            &con_var_prefix,
            &inequalities,
            mu,
            &equalities,
            rho,
            Some(&lambda),
            Some(&best_point),
        )?;

        best_point = inner.decision_values.clone();
        total_evaluations += inner.evaluations;

        let all_satisfied = update_and_check_constraints(
            &inequalities,
            &equalities,
            &con_var_prefix,
            &inner.solution,
            &mut lambda,
            rho,
        );
        last_inner = Some(inner);
        if all_satisfied {
            break;
        }

        if !inequalities.is_empty() && mu > BARRIER_MU_MIN {
            mu *= BARRIER_MU_FACTOR;
        }
        if !equalities.is_empty() && rho < LAGRANGIAN_RHO_MAX {
            rho *= LAGRANGIAN_RHO_FACTOR;
        }
    }

    let mut final_result = last_inner.expect("at least one outer iteration executed");
    let warning =
        build_constraint_warning(&all_constraints, &con_var_prefix, &final_result.solution);
    strip_constraint_variables(&mut final_result.solution, &con_var_prefix);

    Ok(OptimizeResult {
        decision_values: final_result.decision_values,
        objective_value: final_result.objective_value,
        evaluations: total_evaluations,
        solution: final_result.solution,
        warning,
    })
}

/// The Java `updateAndCheckConstraints`: check every constraint at
/// `best_point`, updating the Lagrange multipliers for the equalities.
///
/// The inequality loop `break`s on the first violation (so it stops checking)
/// while the equality loop runs to completion because it also has to update
/// `lambda` — transcribed exactly, because the two loops differ on purpose.
fn update_and_check_constraints(
    inequalities: &[ParsedConstraint],
    equalities: &[ParsedConstraint],
    con_var_prefix: &str,
    solution: &Solution,
    lambda: &mut [f64],
    rho: f64,
) -> bool {
    let mut all_satisfied = true;
    for c in inequalities {
        let key = format!("{con_var_prefix}{}", c.index);
        let lhs_val = solution.values.get(&key).copied().unwrap_or(f64::NAN);
        let g = c.normalised(lhs_val);
        if g > tolerance(c) {
            all_satisfied = false;
            break;
        }
    }
    for (j, c) in equalities.iter().enumerate() {
        let key = format!("{con_var_prefix}{}", c.index);
        let lhs_val = solution.values.get(&key).copied().unwrap_or(f64::NAN);
        let h = c.normalised(lhs_val);
        if h.abs() > tolerance(c) {
            all_satisfied = false;
        }
        lambda[j] += rho * h; // λ_j += ρ · h_j(x)
    }
    all_satisfied
}

/// The Java `buildConstraintWarning`.
fn build_constraint_warning(
    all_constraints: &[ParsedConstraint],
    con_var_prefix: &str,
    solution: &Solution,
) -> Option<String> {
    let mut warning: Option<String> = None;
    for c in all_constraints {
        let key = format!("{con_var_prefix}{}", c.index);
        let lhs_val = solution.values.get(&key).copied().unwrap_or(f64::NAN);
        let v = if c.is_equality() {
            c.normalised(lhs_val).abs()
        } else {
            jmax(0.0, c.normalised(lhs_val))
        };
        if v > tolerance(c) {
            let text = warning.get_or_insert_with(|| {
                "Constraints not satisfied at the returned point: ".to_string()
            });
            if !text.ends_with(": ") {
                text.push_str("; ");
            }
            text.push_str(&format!(
                "'{} {} {}' is off by {}",
                c.lhs_expr,
                c.operator.as_str(),
                java_double_text(c.rhs_value),
                format_g4(v)
            ));
        }
    }
    warning
}

// ---------------------------------------------------------------------------
// Multivariate inner optimisation
// ---------------------------------------------------------------------------

/// The Java `multivariateOptimize`, both overloads (`inequalities`/`equalities`
/// empty ⇒ the plain unconstrained path).
fn multivariate_optimize(
    problem: &Problem,
    all_constraints: &[ParsedConstraint],
    con_var_prefix: &str,
    inequalities: &[ParsedConstraint],
    mu: f64,
    equalities: &[ParsedConstraint],
    rho: f64,
    lambda: Option<&[f64]>,
    warm_start: Option<&[f64]>,
) -> Result<OptimizeResult> {
    let n = problem.decisions.len();
    let mut ctx = Ctx::new(
        problem,
        all_constraints,
        con_var_prefix,
        MULTIVARIATE_MAX_EVALUATIONS,
    );
    ctx.penalty = if inequalities.is_empty() && equalities.is_empty() {
        None
    } else {
        Some(Penalty {
            inequalities: inequalities.to_vec(),
            mu,
            equalities: equalities.to_vec(),
            // `lambda != null ? lambda : new double[eq.size()]`
            lambda: lambda.map_or_else(|| vec![0.0; equalities.len()], <[f64]>::to_vec),
            rho,
        })
    };

    let mut guess = match warm_start {
        Some(warm) => warm.to_vec(),
        None => initial_guess(problem),
    };
    let mut lower_bounds = vec![0.0f64; n];
    let mut upper_bounds = vec![0.0f64; n];
    for i in 0..n {
        lower_bounds[i] = problem.lowers[i];
        upper_bounds[i] = problem.uppers[i];
        guess[i] = jmax(lower_bounds[i], jmin(upper_bounds[i], guess[i]));
    }

    // The Nelder-Mead simplex is bounds-unaware: evaluate out-of-box points at
    // their projection onto the box plus a smooth quadratic distance penalty.
    // BOBYQA enforces bounds natively and skips this.
    ctx.bounds_penalty = !problem.is_bobyqa();
    ctx.lower_bounds = lower_bounds.clone();
    ctx.upper_bounds = upper_bounds.clone();
    // See the module docs: the unported BOBYQA branch still respects its box,
    // by projection instead of by penalty.
    ctx.project_out_of_box = problem.is_bobyqa();

    // Track the best point seen so far: when the evaluation budget runs out
    // Commons Math throws `TooManyEvaluationsException` without returning its
    // best iterate, so the Java keeps its own copy and degrades gracefully.
    ctx.tracked_point = guess.clone();
    ctx.tracked_value = if problem.maximize {
        f64::NEG_INFINITY
    } else {
        f64::INFINITY
    };

    // Java: `catch (TooManyEvaluationsException e) { return trackedPoint.clone(); }`
    // — and *only* that exception; a `SolverException` propagates.
    let mut best_points = match nelder_mead(&mut ctx, &guess, !problem.maximize) {
        Ok(point) => point,
        Err(EvalAbort::Budget) => ctx.tracked_point.clone(),
        Err(EvalAbort::Fatal(error)) => return Err(error),
    };
    // The simplex is unaware of bounds and the tracked point may stem from an
    // out-of-bounds probe: clamp before the final solve.
    for i in 0..n {
        best_points[i] = jmax(lower_bounds[i], jmin(upper_bounds[i], best_points[i]));
    }

    let solution = solve_candidate(problem, &best_points, all_constraints, con_var_prefix)?;
    let objective_value = read_objective(&solution, &problem.objective)?;
    Ok(OptimizeResult {
        decision_values: best_points,
        objective_value,
        evaluations: ctx.evaluations,
        solution: Box::new(solution),
        warning: None,
    })
}

/// The barrier / augmented-Lagrangian terms of one inner sub-problem, owned so
/// the evaluation context can hold them without borrowing back into the
/// caller's stack frame.
#[derive(Debug, Clone)]
struct Penalty {
    inequalities: Vec<ParsedConstraint>,
    mu: f64,
    equalities: Vec<ParsedConstraint>,
    lambda: Vec<f64>,
    rho: f64,
}

// ---------------------------------------------------------------------------
// The evaluation context — the Apache/Java function-wrapper stack, flattened
// ---------------------------------------------------------------------------

/// Why an objective evaluation stopped early.
///
/// Java throws two different exceptions out of the evaluation stack and the
/// call sites treat them differently, so the port keeps them apart:
///
/// * [`EvalAbort::Budget`] — Apache's `TooManyEvaluationsException`, thrown by
///   `computeObjectiveValue` when `MaxEval` runs out. The simplex path catches
///   it and degrades to the tracked best iterate; the Brent path does not.
/// * [`EvalAbort::Fatal`] — the `SolverException` the Java
///   `evaluateObjectiveMultivariate` throws when a solve *succeeds* but its
///   result does not contain the objective variable. That is a broken request,
///   not a bad probe, and it aborts the whole optimisation rather than scoring
///   [`PENALTY`] — note it is raised *outside* the `try` that swallows solver
///   failures.
#[derive(Debug, Clone, PartialEq)]
enum EvalAbort {
    Budget,
    Fatal(FreesError),
}

type EvalResult = std::result::Result<f64, EvalAbort>;

/// The whole evaluation stack the Java composes with lambdas, in one struct:
///
/// ```text
/// Apache computeObjectiveValue   → `budgeted`      (MaxEval, increments first)
///   trackedFn                    → `tracked`       (best-iterate bookkeeping)
///     wrapWithBoundsPenalty      → `bounded`       (projection + quadratic)
///       evaluateWithPenalty      → `penalised`     (barrier + Lagrangian)
///         evaluateObjective*     → `raw_objective` (one full system solve)
/// ```
struct Ctx<'a> {
    problem: &'a Problem,
    all_constraints: &'a [ParsedConstraint],
    con_var_prefix: &'a str,
    /// The Java `AtomicInteger evaluations` — full system solves attempted.
    evaluations: usize,
    /// Apache's `Incrementor` count.
    used: usize,
    /// Apache's `MaxEval`.
    max: usize,
    penalty: Option<Penalty>,
    bounds_penalty: bool,
    project_out_of_box: bool,
    lower_bounds: Vec<f64>,
    upper_bounds: Vec<f64>,
    tracked_point: Vec<f64>,
    tracked_value: f64,
}

impl<'a> Ctx<'a> {
    fn new(
        problem: &'a Problem,
        all_constraints: &'a [ParsedConstraint],
        con_var_prefix: &'a str,
        max: usize,
    ) -> Ctx<'a> {
        Ctx {
            problem,
            all_constraints,
            con_var_prefix,
            evaluations: 0,
            used: 0,
            max,
            penalty: None,
            bounds_penalty: false,
            project_out_of_box: false,
            lower_bounds: Vec::new(),
            upper_bounds: Vec::new(),
            tracked_point: Vec::new(),
            tracked_value: f64::INFINITY,
        }
    }

    /// Apache `BaseOptimizer.computeObjectiveValue`: `incrementEvaluationCount()`
    /// *first* (which is what throws), then call the function. The aborted call
    /// therefore never reaches the solver.
    fn budgeted(&mut self, point: &[f64]) -> EvalResult {
        self.used += 1;
        if self.used > self.max {
            return Err(EvalAbort::Budget);
        }
        self.tracked(point)
    }

    /// The Java `trackedFn`.
    fn tracked(&mut self, point: &[f64]) -> EvalResult {
        let value = self.bounded(point)?;
        let better = if self.problem.maximize {
            value > self.tracked_value
        } else {
            value < self.tracked_value
        };
        if better {
            self.tracked_value = value;
            self.tracked_point.clear();
            self.tracked_point.extend_from_slice(point);
        }
        Ok(value)
    }

    /// The Java `wrapWithBoundsPenalty`: evaluate out-of-box points at their
    /// projection onto the box plus a smooth quadratic distance penalty, so the
    /// landscape stays continuous at the bounds without rewarding infeasible
    /// points.
    fn bounded(&mut self, point: &[f64]) -> EvalResult {
        if !self.bounds_penalty {
            if !self.project_out_of_box {
                return self.penalised(point);
            }
            // The BOBYQA stand-in: bounds respected by projection, no penalty.
            let projected: Vec<f64> = point
                .iter()
                .enumerate()
                .map(|(i, &x)| jmax(self.lower_bounds[i], jmin(self.upper_bounds[i], x)))
                .collect();
            return self.penalised(&projected);
        }

        let penalty_sign = if self.problem.maximize { -1.0 } else { 1.0 };
        let mut violation = 0.0f64;
        let mut projected = point.to_vec();
        for (i, (&x, slot)) in point.iter().zip(projected.iter_mut()).enumerate() {
            if x < self.lower_bounds[i] {
                let d = self.lower_bounds[i] - x;
                violation += d * d;
                *slot = self.lower_bounds[i];
            } else if x > self.upper_bounds[i] {
                let d = x - self.upper_bounds[i];
                violation += d * d;
                *slot = self.upper_bounds[i];
            }
        }
        let value = self.penalised(&projected)?;
        Ok(value + penalty_sign * BOUNDS_PENALTY_WEIGHT * violation)
    }

    /// The Java `evaluateWithPenalty` (or a bare objective when the problem is
    /// unconstrained).
    fn penalised(&mut self, point: &[f64]) -> EvalResult {
        self.evaluations += 1;
        let infeasible = if self.problem.maximize {
            -PENALTY
        } else {
            PENALTY
        };

        let Ok(solution) = solve_candidate(
            self.problem,
            point,
            self.all_constraints,
            self.con_var_prefix,
        ) else {
            return Ok(infeasible);
        };

        let raw_obj = solution
            .values
            .get(&self.problem.objective.to_ascii_lowercase())
            .copied()
            .ok_or_else(|| {
                EvalAbort::Fatal(FreesError::solver(format!(
                    "The objective variable '{}' is not part of the system.",
                    self.problem.objective
                )))
            })?;

        let Some(penalty) = self.penalty.as_ref() else {
            return Ok(raw_obj);
        };
        if raw_obj == PENALTY || raw_obj == -PENALTY {
            return Ok(raw_obj);
        }

        let sign = if self.problem.maximize { 1.0 } else { -1.0 };
        let mut obj = raw_obj;

        for c in &penalty.inequalities {
            let key = format!("{}{}", self.con_var_prefix, c.index);
            let Some(&lhs_val) = solution.values.get(&key).filter(|v| v.is_finite()) else {
                return Ok(infeasible);
            };
            let g = c.normalised(lhs_val);
            if g >= 0.0 {
                let weight = 1.0 / jmax(penalty.mu, BARRIER_MU_MIN);
                obj -= sign * weight * (g * g + g);
            } else {
                obj += sign * penalty.mu * (-g).ln();
            }
        }
        if obj.is_nan() {
            return Ok(infeasible);
        }

        for (j, c) in penalty.equalities.iter().enumerate() {
            let key = format!("{}{}", self.con_var_prefix, c.index);
            let Some(&lhs_val) = solution.values.get(&key).filter(|v| v.is_finite()) else {
                return Ok(infeasible);
            };
            let h = c.normalised(lhs_val);
            obj -= sign * (penalty.lambda[j] * h + (penalty.rho / 2.0) * h * h);
        }
        if obj.is_nan() {
            return Ok(infeasible);
        }

        Ok(obj)
    }
}

// ---------------------------------------------------------------------------
// Talking to the solver
// ---------------------------------------------------------------------------

/// Pin the decisions, add one `<prefix><i> = <c.lhs_expr>` equation per
/// constraint, and solve once.
fn solve_candidate(
    problem: &Problem,
    values: &[f64],
    constraints: &[ParsedConstraint],
    con_var_prefix: &str,
) -> Result<Solution> {
    let mut augmented =
        String::with_capacity(problem.text.len() + 32 * (values.len() + constraints.len()));
    augmented.push_str(&problem.text);
    for (name, value) in problem.decisions.iter().zip(values) {
        augmented.push('\n');
        augmented.push_str(name);
        augmented.push_str(" = ");
        augmented.push_str(&plain_string(*value));
    }
    for c in constraints {
        augmented.push('\n');
        augmented.push_str(con_var_prefix);
        augmented.push_str(&c.index.to_string());
        augmented.push_str(" = ");
        augmented.push_str(&c.lhs_expr);
    }
    crate::engine::solve_with(&augmented, &problem.settings, &problem.overrides)
        .map_err(|failure| failure.error)
}

/// The Java `solveWithDecisions`: append `decision = value` for every decision
/// and re-solve the whole document.
fn solve_with_decisions(problem: &Problem, values: &[f64]) -> Result<Solution> {
    solve_candidate(problem, values, &[], "")
}

/// Strips temporary constraint variables introduced during optimization from the final solution.
fn strip_constraint_variables(solution: &mut Solution, prefix: &str) {
    if prefix.is_empty() {
        return;
    }
    solution.values.retain(|k, _| !k.starts_with(prefix));
    solution.display_names.retain(|k, _| !k.starts_with(prefix));
    solution
        .inferred_units
        .retain(|k, _| !k.starts_with(prefix));
    solution
        .residuals
        .retain(|r| !r.equation.trim_start().starts_with(prefix));
    solution.blocks.retain_mut(|b| {
        b.variables.retain(|v| !v.starts_with(prefix));
        !b.variables.is_empty()
    });
    solution.block_equations.retain_mut(|eqs| {
        eqs.retain(|eq| !eq.trim_start().starts_with(prefix));
        !eqs.is_empty()
    });
}

/// The Java `result.variables().get(objective)`.
///
/// The Java result map is a `TreeMap<>(String.CASE_INSENSITIVE_ORDER)` keyed by
/// display spelling, so the lookup is case-insensitive; this port keys
/// [`Solution::values`] by the lowercase canonical name, which is the same
/// thing.
fn read_objective(solution: &Solution, objective: &str) -> Result<f64> {
    solution
        .values
        .get(&objective.to_ascii_lowercase())
        .copied()
        .ok_or_else(|| {
            FreesError::solver(format!(
                "The objective variable '{objective}' is not part of the solution."
            ))
        })
}

/// The Java `initialGuess`: the decision's own spec guess when the Variable
/// Information window supplied one, else the midpoint of the box, clamped.
///
/// The Java looks the spec up with `problem.specs().get(dec)` against a map
/// whose keys `VariableSpec`'s compact constructor lowercased, so a decision
/// spelled with capitals silently misses and falls back to the midpoint. That
/// quirk is reproduced here rather than "fixed".
fn initial_guess(problem: &Problem) -> Vec<f64> {
    let mut guess = Vec::with_capacity(problem.decisions.len());
    for (i, dec) in problem.decisions.iter().enumerate() {
        let lo = problem.lowers[i];
        let hi = problem.uppers[i];
        let spec_guess = problem
            .overrides
            .iter()
            .filter(|o| o.name.trim().to_ascii_lowercase() == *dec)
            .find_map(|o| variable_override_spec(o).ok())
            .map(|(_, spec_guess, _, _)| spec_guess);
        let value = spec_guess.unwrap_or((lo + hi) / 2.0);
        guess.push(jmax(lo, jmin(hi, value)));
    }
    guess
}

// ---------------------------------------------------------------------------
// Apache Commons Math 3.6.1 — BrentOptimizer
// ---------------------------------------------------------------------------

/// `0.5 * (3 - sqrt(5))` — Apache's `BrentOptimizer.GOLDEN_SECTION`.
const GOLDEN_SECTION: f64 = 0.381_966_011_250_105_15;

/// Apache `BrentOptimizer.doOptimize`, driven over `[lo, hi]` from the interval
/// midpoint (`SearchInterval(lo, hi)`'s default start value).
///
/// Returns the abscissa of the best point found. `minimize == false` mirrors
/// `GoalType.MAXIMIZE`, which Apache implements by negating the objective and
/// minimising — reproduced literally so the parabolic-fit arithmetic matches.
///
/// No user convergence checker is installed (`BrentOptimizer(rel, abs)` passes
/// `null`), so the loop terminates only on Brent's own criterion.
fn brent_optimize(ctx: &mut Ctx<'_>, lo: f64, hi: f64, minimize: bool) -> EvalResult {
    let mid = lo + 0.5 * (hi - lo);

    let (mut a, mut b) = if lo < hi { (lo, hi) } else { (hi, lo) };

    let mut x = mid;
    let mut v = x;
    let mut w = x;
    let mut d = 0.0f64;
    let mut e = 0.0f64;
    let mut fx = ctx.budgeted(&[x])?;
    if !minimize {
        fx = -fx;
    }
    let mut fv = fx;
    let mut fw = fx;

    // `best` tracks the best (point, value) pair in *goal* orientation, which
    // is what Apache returns.
    let mut best = (x, if minimize { fx } else { -fx });

    loop {
        let m = 0.5 * (a + b);
        let tol1 = BRENT_RELATIVE_THRESHOLD * x.abs() + BRENT_ABSOLUTE_THRESHOLD;
        let tol2 = 2.0 * tol1;

        let stop = (x - m).abs() <= tol2 - 0.5 * (b - a);
        if stop {
            let candidate = (x, if minimize { fx } else { -fx });
            return Ok(better_of(best, candidate, minimize).0);
        }

        let mut u;
        if e.abs() > tol1 {
            // Fit a parabola.
            let mut r = (x - w) * (fx - fv);
            let mut q = (x - v) * (fx - fw);
            let mut p = (x - v) * q - (x - w) * r;
            q = 2.0 * (q - r);

            if q > 0.0 {
                p = -p;
            } else {
                q = -q;
            }

            r = e;
            e = d;

            if p > q * (a - x) && p < q * (b - x) && p.abs() < (0.5 * q * r).abs() {
                // Parabolic interpolation step.
                d = p / q;
                u = x + d;
                // f must not be evaluated too close to a or b.
                if u - a < tol2 || b - u < tol2 {
                    d = if x <= m { tol1 } else { -tol1 };
                }
            } else {
                // Golden section step.
                e = if x < m { b - x } else { a - x };
                d = GOLDEN_SECTION * e;
            }
        } else {
            // Golden section step.
            e = if x < m { b - x } else { a - x };
            d = GOLDEN_SECTION * e;
        }

        // Update by at least "tol1".
        if d.abs() < tol1 {
            u = if d >= 0.0 { x + tol1 } else { x - tol1 };
        } else {
            u = x + d;
        }

        let mut fu = ctx.budgeted(&[u])?;
        if !minimize {
            fu = -fu;
        }

        let current = (u, if minimize { fu } else { -fu });
        best = better_of(best, current, minimize);

        // Update a, b, v, w and x.
        if fu <= fx {
            if u < x {
                b = x;
            } else {
                a = x;
            }
            v = w;
            fv = fw;
            w = x;
            fw = fx;
            x = u;
            fx = fu;
        } else {
            if u < x {
                a = u;
            } else {
                b = u;
            }
            if fu <= fw || precision_equals(w, x) {
                v = w;
                fv = fw;
                w = u;
                fw = fu;
            } else if fu <= fv || precision_equals(v, x) || precision_equals(v, w) {
                v = u;
                fv = fu;
            }
        }
    }
}

/// Apache `BrentOptimizer.best` for two non-null pairs.
fn better_of(a: (f64, f64), b: (f64, f64), minimize: bool) -> (f64, f64) {
    if minimize {
        if a.1 <= b.1 {
            a
        } else {
            b
        }
    } else if a.1 >= b.1 {
        a
    } else {
        b
    }
}

// ---------------------------------------------------------------------------
// Apache Commons Math 3.6.1 — SimplexOptimizer + NelderMeadSimplex
// ---------------------------------------------------------------------------

/// `NelderMeadSimplex(n)` defaults: reflection, expansion, contraction, shrink.
const NM_RHO: f64 = 1.0;
const NM_KHI: f64 = 2.0;
const NM_GAMMA: f64 = 0.5;
const NM_SIGMA: f64 = 0.5;

/// One simplex vertex — Apache's `PointValuePair`.
#[derive(Clone, Debug)]
struct Vertex {
    point: Vec<f64>,
    value: f64,
}

/// `AbstractSimplex(n)` → `AbstractSimplex(n, 1.0)`: the *triangular* start
/// configuration Apache builds with a nested `System.arraycopy`.
///
/// `startConfiguration[i][k] = steps[k]` for `k <= i` and `0` beyond, so with
/// unit steps vertex `i + 1` is the start point with its first `i + 1`
/// coordinates each raised by one. This is **not** a set of unit-vector
/// offsets, and getting it wrong changes every subsequent iterate.
fn build_simplex(start: &[f64]) -> Vec<Vertex> {
    let n = start.len();
    let mut simplex = Vec::with_capacity(n + 1);
    simplex.push(Vertex {
        point: start.to_vec(),
        value: f64::NAN,
    });
    for i in 0..n {
        let mut vertex = start.to_vec();
        // `startConfiguration[i][k] = steps[k] = 1.0` for every `k <= i`.
        for slot in vertex.iter_mut().take(i + 1) {
            *slot += 1.0;
        }
        simplex.push(Vertex {
            point: vertex,
            value: f64::NAN,
        });
    }
    simplex
}

/// Apache `AbstractSimplex.evaluate`: fill in every not-yet-evaluated vertex,
/// then stable-sort best to worst.
fn evaluate_simplex(
    ctx: &mut Ctx<'_>,
    simplex: &mut [Vertex],
    minimize: bool,
) -> std::result::Result<(), EvalAbort> {
    for vertex in simplex.iter_mut() {
        if vertex.value.is_nan() {
            let value = ctx.budgeted(&vertex.point)?;
            vertex.value = value;
        }
    }
    // `Arrays.sort` is a stable merge sort; so is `slice::sort_by`.
    simplex.sort_by(|a, b| compare_vertices(a, b, minimize));
    Ok(())
}

/// The Java comparator: `isMinim ? Double.compare(v1, v2) : Double.compare(v2, v1)`.
fn compare_vertices(a: &Vertex, b: &Vertex, minimize: bool) -> Ordering {
    if minimize {
        java_compare(a.value, b.value)
    } else {
        java_compare(b.value, a.value)
    }
}

/// Apache `AbstractSimplex.replaceWorstPoint`.
fn replace_worst_point(simplex: &mut [Vertex], mut candidate: Vertex, minimize: bool) {
    let dimension = simplex.len() - 1;
    for slot in simplex.iter_mut().take(dimension) {
        if compare_vertices(slot, &candidate, minimize) == Ordering::Greater {
            std::mem::swap(slot, &mut candidate);
        }
    }
    simplex[dimension] = candidate;
}

/// Apache `SimplexOptimizer.doOptimize` with `NelderMeadSimplex(n)` and a
/// `SimpleValueChecker(1e-10, 1e-12)`.
///
/// Returns the best vertex's point. [`EvalAbort::Budget`] is the Java's
/// `TooManyEvaluationsException`, which the caller turns into the tracked best
/// iterate; [`EvalAbort::Fatal`] propagates.
fn nelder_mead(
    ctx: &mut Ctx<'_>,
    start: &[f64],
    minimize: bool,
) -> std::result::Result<Vec<f64>, EvalAbort> {
    let mut simplex = build_simplex(start);
    evaluate_simplex(ctx, &mut simplex, minimize)?;

    let mut previous: Vec<Vertex> = Vec::new();
    let mut iteration = 0usize;
    loop {
        if iteration > 0 {
            let mut converged = true;
            for i in 0..simplex.len() {
                converged =
                    converged && simple_value_converged(previous[i].value, simplex[i].value);
                if !converged {
                    break; // short circuit: "converged" will stay false
                }
            }
            if converged {
                return Ok(simplex[0].point.clone());
            }
        }
        previous = simplex.clone();
        nelder_mead_iterate(ctx, &mut simplex, minimize)?;
        iteration += 1;
    }
}

/// Apache `SimpleValueChecker.converged`.
fn simple_value_converged(previous: f64, current: f64) -> bool {
    let difference = (previous - current).abs();
    let size = jmax(previous.abs(), current.abs());
    difference <= size * SIMPLEX_RELATIVE_THRESHOLD || difference <= SIMPLEX_ABSOLUTE_THRESHOLD
}

/// Apache `NelderMeadSimplex.iterate`.
fn nelder_mead_iterate(
    ctx: &mut Ctx<'_>,
    simplex: &mut [Vertex],
    minimize: bool,
) -> std::result::Result<(), EvalAbort> {
    // The simplex has n + 1 points when the dimension is n.
    let n = simplex.len() - 1;

    let best = simplex[0].clone();
    let second_best = simplex[n - 1].clone();
    let worst = simplex[n].clone();
    let x_worst = worst.point.clone();

    // Centroid of the best vertices (dismissing the worst point at index n).
    let mut centroid = vec![0.0f64; n];
    for vertex in simplex.iter().take(n) {
        for (slot, coordinate) in centroid.iter_mut().zip(&vertex.point) {
            *slot += *coordinate;
        }
    }
    let scaling = 1.0 / n as f64;
    for slot in centroid.iter_mut() {
        *slot *= scaling;
    }

    // Reflection point.
    let mut x_r = vec![0.0f64; n];
    for j in 0..n {
        x_r[j] = centroid[j] + NM_RHO * (centroid[j] - x_worst[j]);
    }
    let reflected = Vertex {
        value: ctx.budgeted(&x_r)?,
        point: x_r.clone(),
    };

    if compare_vertices(&best, &reflected, minimize) != Ordering::Greater
        && compare_vertices(&reflected, &second_best, minimize) == Ordering::Less
    {
        replace_worst_point(simplex, reflected, minimize);
        return Ok(());
    }

    if compare_vertices(&reflected, &best, minimize) == Ordering::Less {
        // Expansion point.
        let mut x_e = vec![0.0f64; n];
        for j in 0..n {
            x_e[j] = centroid[j] + NM_KHI * (x_r[j] - centroid[j]);
        }
        let expanded = Vertex {
            value: ctx.budgeted(&x_e)?,
            point: x_e,
        };
        if compare_vertices(&expanded, &reflected, minimize) == Ordering::Less {
            replace_worst_point(simplex, expanded, minimize);
        } else {
            replace_worst_point(simplex, reflected, minimize);
        }
        return Ok(());
    }

    if compare_vertices(&reflected, &worst, minimize) == Ordering::Less {
        // Outside contraction.
        let mut x_c = vec![0.0f64; n];
        for j in 0..n {
            x_c[j] = centroid[j] + NM_GAMMA * (x_r[j] - centroid[j]);
        }
        let out_contracted = Vertex {
            value: ctx.budgeted(&x_c)?,
            point: x_c,
        };
        if compare_vertices(&out_contracted, &reflected, minimize) != Ordering::Greater {
            replace_worst_point(simplex, out_contracted, minimize);
            return Ok(());
        }
    } else {
        // Inside contraction.
        let mut x_c = vec![0.0f64; n];
        for j in 0..n {
            x_c[j] = centroid[j] - NM_GAMMA * (x_r[j] - centroid[j]);
        }
        let in_contracted = Vertex {
            value: ctx.budgeted(&x_c)?,
            point: x_c,
        };
        if compare_vertices(&in_contracted, &worst, minimize) == Ordering::Less {
            replace_worst_point(simplex, in_contracted, minimize);
            return Ok(());
        }
    }

    // Shrink.
    let x_smallest = simplex[0].point.clone();
    for vertex in simplex.iter_mut().skip(1) {
        for (slot, smallest) in vertex.point.iter_mut().zip(&x_smallest) {
            *slot = smallest + NM_SIGMA * (*slot - smallest);
        }
        vertex.value = f64::NAN;
    }
    evaluate_simplex(ctx, simplex, minimize)
}

// ---------------------------------------------------------------------------
// Java / Apache numeric primitives shared with the rest of `analysis`
// ---------------------------------------------------------------------------

/// `Math.max` / `FastMath.max`: **NaN propagates**, and `max(-0.0, 0.0) == 0.0`.
///
/// Rust's `f64::max` returns the non-NaN operand instead, which silently
/// changes clamping and convergence tests. Every site the Java writes
/// `Math.max`/`FastMath.max` uses this.
pub(crate) fn jmax(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        return f64::NAN;
    }
    if a == 0.0 && b == 0.0 {
        // `Math.max(-0.0, 0.0) == 0.0`
        return if a.is_sign_negative() { b } else { a };
    }
    if a >= b {
        a
    } else {
        b
    }
}

/// `Math.min` / `FastMath.min`: **NaN propagates**, and `min(-0.0, 0.0) == -0.0`.
pub(crate) fn jmin(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        return f64::NAN;
    }
    if a == 0.0 && b == 0.0 {
        return if a.is_sign_negative() { a } else { b };
    }
    if a <= b {
        a
    } else {
        b
    }
}

/// `Double.compare`: total order with `-0.0 < 0.0` and NaN greater than
/// everything, including `+Infinity`.
///
/// Rust's `f64::total_cmp` agrees except on negative NaN, which Java's
/// `doubleToLongBits` canonicalises to positive quiet NaN — so a `-NaN` sorts
/// *last* in Java and *first* under `total_cmp`.
pub(crate) fn java_compare(a: f64, b: f64) -> Ordering {
    if a < b {
        return Ordering::Less;
    }
    if a > b {
        return Ordering::Greater;
    }
    // `Double.doubleToLongBits` canonicalises every NaN to one bit pattern.
    let bits = |v: f64| -> i64 {
        if v.is_nan() {
            0x7ff8_0000_0000_0000u64 as i64
        } else {
            v.to_bits() as i64
        }
    };
    bits(a).cmp(&bits(b))
}

/// Apache `Precision.equals(x, y)` — equality within one ULP, sign-aware.
///
/// Used verbatim by `BrentOptimizer` (`Precision.equals(w, x)`) and by
/// `BrentSolver` (`Precision.equals(fb, 0)`), where it decides which of the
/// three stored abscissae gets replaced. A naive `==` changes the search.
pub(crate) fn precision_equals(x: f64, y: f64) -> bool {
    precision_equals_ulps(x, y, 1)
}

pub(crate) fn precision_equals_ulps(x: f64, y: f64, max_ulps: i64) -> bool {
    const SGN_MASK: i64 = i64::MIN; // 0x8000_0000_0000_0000
    let x_int = x.to_bits() as i64; // `doubleToRawLongBits`
    let y_int = y.to_bits() as i64;
    let is_equal = if ((x_int ^ y_int) & SGN_MASK) == 0 {
        // Same sign: no overflow risk.
        (x_int - y_int).abs() <= max_ulps
    } else {
        // Opposite signs: measure each side's distance from its own zero.
        let (delta_plus, delta_minus) = if x_int < y_int {
            (y_int, x_int.wrapping_sub(i64::MIN))
        } else {
            (x_int, y_int.wrapping_sub(i64::MIN))
        };
        if delta_plus > max_ulps {
            false
        } else {
            delta_minus <= max_ulps - delta_plus
        }
    };
    is_equal && !x.is_nan() && !y.is_nan()
}

/// `BigDecimal.valueOf(value).toPlainString()` — the text the Java appends when
/// it pins a decision variable.
///
/// Rust's `Display` for `f64` is already the shortest round-tripping decimal
/// **and never uses exponent notation**, so it is a plain string by
/// construction. The digits can differ from Java's (`0.0000001` vs
/// `0.00000010`) but the parsed `f64` is bit-identical, which is the only thing
/// the re-solve can observe. Non-finite values are rendered as-is; the Java
/// throws `NumberFormatException` there, which this port surfaces as the
/// re-solve's own parse failure (⇒ [`PENALTY`]).
pub(crate) fn plain_string(value: f64) -> String {
    format!("{value}")
}

/// `String.valueOf(double)` / `Double.toString(double)` — the rendering the
/// Java `StringBuilder.append(double)` produces inside the constraint warning.
///
/// Verified against the oracle: `'a <= -50.0' is off by 50.00`, i.e. the RHS is
/// rendered with Java's mandatory `.0` and its switch to computerised
/// scientific notation outside `[1e-3, 1e7)`. Rust's `Display` would print
/// `-50`, which is a visible text difference in a user-facing sentence.
fn java_double_text(value: f64) -> String {
    if value.is_nan() {
        return "NaN".to_string();
    }
    if value.is_infinite() {
        return if value > 0.0 { "Infinity" } else { "-Infinity" }.to_string();
    }
    let magnitude = value.abs();
    if value == 0.0 || (1e-3..1e7).contains(&magnitude) {
        let plain = format!("{value}");
        if plain.contains('.') {
            plain
        } else {
            format!("{plain}.0")
        }
    } else {
        let scientific = format!("{value:e}");
        let (mantissa, exponent) = scientific
            .split_once('e')
            .expect("`{:e}` always emits an exponent");
        if mantissa.contains('.') {
            format!("{mantissa}E{exponent}")
        } else {
            format!("{mantissa}.0E{exponent}")
        }
    }
}

/// `String.format("%.4g", v)` — four significant digits, as the Java warning
/// uses. Java's `%g` never emits a bare integer and switches to scientific
/// notation outside `[1e-4, 1e5)`.
fn format_g4(v: f64) -> String {
    if v == 0.0 {
        return "0.00000".to_string();
    }
    if !v.is_finite() {
        return format!("{v}");
    }
    let exponent = v.abs().log10().floor() as i32;
    if (-5..4).contains(&exponent) {
        let decimals = (3 - exponent).max(0) as usize;
        format!("{v:.decimals$}")
    } else {
        format!("{v:.3e}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings() -> SolverSettings {
        SolverSettings::default()
    }

    fn problem(text: &str, objective: &str, decisions: &[&str], lo: &[f64], hi: &[f64]) -> Problem {
        Problem {
            text: text.to_string(),
            settings: settings(),
            overrides: Vec::new(),
            objective: objective.to_string(),
            decisions: decisions.iter().map(|s| (*s).to_string()).collect(),
            lowers: lo.to_vec(),
            uppers: hi.to_vec(),
            method: None,
            maximize: false,
            constraints: Vec::new(),
        }
    }

    // ── numeric primitives ────────────────────────────────────────────────

    #[test]
    fn jmax_and_jmin_propagate_nan_like_java() {
        assert!(jmax(f64::NAN, 1.0).is_nan());
        assert!(jmax(1.0, f64::NAN).is_nan());
        assert!(jmin(f64::NAN, 1.0).is_nan());
        assert!(jmin(1.0, f64::NAN).is_nan());
        // Rust's own max/min would answer 1.0 here — that is the divergence.
        assert_eq!(f64::NAN.max(1.0), 1.0);
        assert_eq!(jmax(-0.0, 0.0), 0.0);
        assert!(jmax(-0.0, 0.0).is_sign_positive());
        assert!(jmin(-0.0, 0.0).is_sign_negative());
        assert_eq!(jmax(3.0, 2.0), 3.0);
        assert_eq!(jmin(3.0, 2.0), 2.0);
    }

    #[test]
    fn java_compare_orders_nan_last_and_signed_zero() {
        assert_eq!(java_compare(1.0, 2.0), Ordering::Less);
        assert_eq!(java_compare(2.0, 1.0), Ordering::Greater);
        assert_eq!(java_compare(1.0, 1.0), Ordering::Equal);
        assert_eq!(java_compare(-0.0, 0.0), Ordering::Less);
        assert_eq!(java_compare(f64::INFINITY, f64::NAN), Ordering::Less);
        assert_eq!(java_compare(f64::NAN, f64::NAN), Ordering::Equal);
        // Java canonicalises -NaN, so it also sorts last.
        assert_eq!(java_compare(-f64::NAN, f64::INFINITY), Ordering::Greater);
    }

    #[test]
    fn precision_equals_is_one_ulp() {
        assert!(precision_equals(1.0, 1.0));
        assert!(precision_equals(1.0, f64::from_bits(1.0f64.to_bits() + 1)));
        assert!(!precision_equals(1.0, f64::from_bits(1.0f64.to_bits() + 2)));
        assert!(precision_equals(0.0, -0.0));
        assert!(!precision_equals(f64::NAN, f64::NAN));
        assert!(!precision_equals(1.0, 2.0));
    }

    #[test]
    fn plain_string_round_trips() {
        for value in [0.0, 1.0, -2.5, 1e-7, 1e21, 95.0, 1.0 / 3.0] {
            let text = plain_string(value);
            assert!(!text.contains('e'), "{text} should be plain");
            assert_eq!(text.parse::<f64>().unwrap(), value);
        }
    }

    // ── constraint parsing ────────────────────────────────────────────────

    #[test]
    fn parses_the_three_operators() {
        let raw = vec![
            "x <= 5".to_string(),
            "a + b >= -2.5".to_string(),
            "c = 3".to_string(),
        ];
        let parsed = parse_constraints(&raw).unwrap();
        assert_eq!(parsed[0].lhs_expr, "x");
        assert_eq!(parsed[0].operator, ConstraintOp::Le);
        assert_eq!(parsed[0].rhs_value, 5.0);
        assert_eq!(parsed[1].lhs_expr, "a + b");
        assert_eq!(parsed[1].operator, ConstraintOp::Ge);
        assert_eq!(parsed[1].rhs_value, -2.5);
        assert_eq!(parsed[2].operator, ConstraintOp::Eq);
        assert!(parsed[2].is_equality());
    }

    #[test]
    fn rejects_unparseable_constraints() {
        // A bare `<` matches no alternative of `(<=|>=|=)`.
        assert!(parse_constraints(&["x < 5".to_string()]).is_err());
        // `[^<>=]+` needs at least one leading character.
        assert!(parse_constraints(&["<= 5".to_string()]).is_err());
        // `(.+)$` needs at least one trailing character.
        assert!(parse_constraints(&["x <=".to_string()]).is_err());
        // The RHS must be a numeric constant.
        assert!(parse_constraints(&["x <= y".to_string()]).is_err());
    }

    #[test]
    fn normalisation_matches_the_java_table() {
        let le = ParsedConstraint {
            index: 0,
            lhs_expr: "x".into(),
            operator: ConstraintOp::Le,
            rhs_value: 5.0,
        };
        let ge = ParsedConstraint {
            index: 0,
            lhs_expr: "x".into(),
            operator: ConstraintOp::Ge,
            rhs_value: 5.0,
        };
        assert_eq!(le.normalised(7.0), 2.0);
        assert_eq!(ge.normalised(7.0), -2.0);
        assert_eq!(tolerance(&le), 1e-6 * 5.0);
    }

    // ── the Apache start simplex ──────────────────────────────────────────

    #[test]
    fn start_simplex_is_triangular_not_unit_vectors() {
        let simplex = build_simplex(&[0.0, 0.0, 0.0]);
        assert_eq!(simplex.len(), 4);
        assert_eq!(simplex[0].point, vec![0.0, 0.0, 0.0]);
        assert_eq!(simplex[1].point, vec![1.0, 0.0, 0.0]);
        assert_eq!(simplex[2].point, vec![1.0, 1.0, 0.0]);
        assert_eq!(simplex[3].point, vec![1.0, 1.0, 1.0]);
    }

    // ── validation ────────────────────────────────────────────────────────

    #[test]
    fn validation_rejects_bad_requests() {
        let mut p = problem("y = x^2", "y", &["x"], &[-1.0], &[1.0]);
        p.objective = String::new();
        assert!(optimize(&p).is_err());

        let mut p = problem("y = x^2", "y", &["x"], &[-1.0], &[1.0]);
        p.decisions.clear();
        assert!(optimize(&p).is_err());

        let p = problem("y = x^2", "x", &["x"], &[-1.0], &[1.0]);
        assert!(optimize(&p).is_err(), "objective must differ from decision");

        let p = problem("y = x^2", "y", &["x"], &[1.0], &[1.0]);
        assert!(optimize(&p).is_err(), "lower must be strictly below upper");

        let p = problem("y = x^2", "y", &["x"], &[f64::NEG_INFINITY], &[1.0]);
        assert!(optimize(&p).is_err(), "bounds must be finite");

        let mut p = problem("y = x^2", "y", &["x"], &[-1.0], &[1.0]);
        p.uppers.clear();
        assert!(optimize(&p).is_err(), "bounds must match the decisions");
    }

    // ── 1-D Brent ─────────────────────────────────────────────────────────

    #[test]
    fn brent_finds_a_parabola_minimum() {
        let p = problem("y = (x - 2)^2 + 3", "y", &["x"], &[-10.0], &[10.0]);
        let result = optimize(&p).unwrap();
        assert!(
            (result.decision_value() - 2.0).abs() < 1e-6,
            "x = {}",
            result.decision_value()
        );
        assert!((result.objective_value - 3.0).abs() < 1e-9);
        assert!(result.evaluations > 0 && result.evaluations <= MAX_EVALUATIONS);
        // The full solve at the optimum rides along.
        assert!(result.solution.values.contains_key("x"));
    }

    #[test]
    fn brent_maximises_when_asked() {
        let mut p = problem("y = -(x - 1.5)^2 + 7", "y", &["x"], &[-10.0], &[10.0]);
        p.maximize = true;
        let result = optimize(&p).unwrap();
        assert!((result.decision_value() - 1.5).abs() < 1e-6);
        assert!((result.objective_value - 7.0).abs() < 1e-9);
    }

    #[test]
    fn brent_respects_the_bracket_when_the_optimum_is_outside() {
        // The unconstrained minimum is at x = 5; the box stops at 1.
        let p = problem("y = (x - 5)^2", "y", &["x"], &[-1.0], &[1.0]);
        let result = optimize(&p).unwrap();
        assert!(
            result.decision_value() <= 1.0 + 1e-9 && result.decision_value() >= 0.99,
            "x = {}",
            result.decision_value()
        );
    }

    #[test]
    fn a_failing_probe_is_penalised_but_a_failing_final_solve_is_not() {
        // ln(x) is a domain error for x <= 0, so probes in the left half of the
        // box score PENALTY instead of aborting — but the *final* solve at the
        // chosen point is deliberately unguarded in the Java, so a search that
        // settles on the infeasible plateau surfaces the block failure. Oracle
        // (`Optimizer` on this exact problem): SolverException "Block 1 did not
        // converge within 250 iterations".
        let p = problem("y = (ln(x) - 1)^2", "y", &["x"], &[-5.0], &[5.0]);
        let err = optimize(&p).unwrap_err();
        assert!(
            err.to_string_message().contains("did not converge"),
            "{err}"
        );

        // Restricted to the valid half, the same objective optimises cleanly at
        // x = e, which proves the penalty plateau is what stopped it above.
        let p = problem("y = (ln(x) - 1)^2", "y", &["x"], &[0.1], &[10.0]);
        let result = optimize(&p).unwrap();
        assert!(
            (result.decision_value() - std::f64::consts::E).abs() < 1e-5,
            "x = {}",
            result.decision_value()
        );
    }

    // ── multivariate Nelder-Mead ──────────────────────────────────────────

    #[test]
    fn nelder_mead_finds_a_two_dimensional_minimum() {
        let mut p = problem(
            "z = (a - 1)^2 + (b + 2)^2",
            "z",
            &["a", "b"],
            &[-10.0, -10.0],
            &[10.0, 10.0],
        );
        p.method = Some("nelder-mead".to_string());
        let result = optimize(&p).unwrap();
        assert!(
            (result.decision_values[0] - 1.0).abs() < 1e-4,
            "a = {}",
            result.decision_values[0]
        );
        assert!(
            (result.decision_values[1] + 2.0).abs() < 1e-4,
            "b = {}",
            result.decision_values[1]
        );
        assert!(result.objective_value < 1e-7);
    }

    #[test]
    fn nelder_mead_stays_inside_the_box() {
        // The free minimum is at (5, 5); the box caps both at 1.
        let mut p = problem(
            "z = (a - 5)^2 + (b - 5)^2",
            "z",
            &["a", "b"],
            &[-1.0, -1.0],
            &[1.0, 1.0],
        );
        p.method = Some("simplex".to_string());
        let result = optimize(&p).unwrap();
        for value in &result.decision_values {
            assert!(*value <= 1.0 + 1e-12 && *value >= -1.0 - 1e-12, "{value}");
        }
        assert!(result.decision_values[0] > 0.9);
        assert!(result.decision_values[1] > 0.9);
    }

    #[test]
    fn a_single_decision_with_an_explicit_simplex_method_takes_the_nd_path() {
        let mut p = problem("y = (x - 2)^2 + 3", "y", &["x"], &[-10.0], &[10.0]);
        p.method = Some("nelder-mead".to_string());
        let result = optimize(&p).unwrap();
        assert!((result.decision_value() - 2.0).abs() < 1e-5);
    }

    #[test]
    fn bobyqa_falls_back_to_the_projected_simplex() {
        // Documented divergence: BOBYQA itself is not ported, but the request
        // must still produce the in-box optimum rather than an error.
        let mut p = problem(
            "z = (a - 3)^2 + (b - 3)^2",
            "z",
            &["a", "b"],
            &[0.0, 0.0],
            &[1.0, 1.0],
        );
        p.method = Some("BOBYQA".to_string());
        let result = optimize(&p).unwrap();
        for value in &result.decision_values {
            assert!(*value <= 1.0 + 1e-12 && *value >= -1e-12, "{value}");
        }
        // The Java oracle (real BOBYQA) answers x=[1.0, 1.0] obj=8.0 in 25
        // evaluations. The *answer* is reproduced; the evaluation count is not,
        // and cannot be without porting BOBYQA itself.
        assert_eq!(result.decision_values, vec![1.0, 1.0]);
        assert_eq!(result.objective_value, 8.0);
        assert_ne!(
            result.evaluations, 25,
            "if this ever matches, BOBYQA was ported and the doc note is stale"
        );
    }

    // ── constrained ───────────────────────────────────────────────────────

    #[test]
    fn an_inequality_constraint_moves_the_optimum() {
        // Unconstrained min of (a-5)^2 is a = 5; the constraint caps a at 2.
        let mut p = problem("y = (a - 5)^2", "y", &["a"], &[0.0], &[10.0]);
        p.method = Some("nelder-mead".to_string());
        p.constraints = vec!["a <= 2".to_string()];
        let result = optimize(&p).unwrap();
        assert!(
            result.decision_values[0] <= 2.0 + 1e-3,
            "a = {}",
            result.decision_values[0]
        );
        assert!(
            result.decision_values[0] > 1.0,
            "a = {}",
            result.decision_values[0]
        );
        assert!(result.warning.is_none(), "{:?}", result.warning);
        assert!(!result
            .solution
            .values
            .keys()
            .any(|k| k.starts_with("zz_opt_con_")));
    }

    #[test]
    fn an_equality_constraint_is_honoured() {
        // Minimise a^2 + b^2 subject to a + b = 2 → (1, 1).
        let mut p = problem(
            "z = a^2 + b^2\ns = a + b",
            "z",
            &["a", "b"],
            &[-5.0, -5.0],
            &[5.0, 5.0],
        );
        p.method = Some("nelder-mead".to_string());
        p.constraints = vec!["a + b = 2".to_string()];
        let result = optimize(&p).unwrap();
        let sum = result.decision_values[0] + result.decision_values[1];
        assert!((sum - 2.0).abs() < 1e-2, "a + b = {sum}");
        assert!(!result
            .solution
            .values
            .keys()
            .any(|k| k.starts_with("zz_opt_con_")));
    }

    #[test]
    fn an_impossible_constraint_reports_a_warning_instead_of_lying() {
        // The box forbids a <= -50, so the run cannot satisfy the constraint.
        let mut p = problem("y = a^2", "y", &["a"], &[0.0], &[10.0]);
        p.method = Some("nelder-mead".to_string());
        p.constraints = vec!["a <= -50".to_string()];
        let result = optimize(&p).unwrap();
        let warning = result.warning.expect("an unsatisfiable constraint warns");
        assert!(warning.starts_with("Constraints not satisfied at the returned point: "));
        assert!(warning.contains("a <= -50"), "{warning}");
        assert!(warning.contains("is off by"), "{warning}");
    }

    // ── parity with the Java oracle ───────────────────────────────────────
    //
    // Every expectation below was produced by running the real
    // `com.frees.backend.core.Optimizer` over the same document, bounds and
    // method (`tools/golden-dumper/classpath.sh` for the classpath). The
    // `evaluations` counts are the sharpest signal available: they pin the
    // whole probe *sequence*, not just where it stopped, so a wrong start
    // simplex or a wrong Brent branch shows up immediately.

    #[test]
    fn oracle_brent_evaluation_counts() {
        // Java: x=[1.9999999999999998] obj=3.0 evals=34
        let p = problem("y = (x - 2)^2 + 3", "y", &["x"], &[-10.0], &[10.0]);
        let r = optimize(&p).unwrap();
        assert_eq!(r.evaluations, 34, "Brent probe sequence diverged");
        assert!((r.decision_value() - 2.0).abs() < 1e-12);

        // Java: x=[1.5000000000000002] obj=7.0 evals=35
        let mut p = problem("y = -(x - 1.5)^2 + 7", "y", &["x"], &[-10.0], &[10.0]);
        p.maximize = true;
        let r = optimize(&p).unwrap();
        assert_eq!(r.evaluations, 35);
        assert!((r.decision_value() - 1.5).abs() < 1e-12);

        // Java: x=[0.9999999998574609] obj=16.000000001140315 evals=49
        let p = problem("y = (x - 5)^2", "y", &["x"], &[-1.0], &[1.0]);
        let r = optimize(&p).unwrap();
        assert_eq!(r.evaluations, 49);
        assert!((r.decision_value() - 0.999_999_999_857_460_9).abs() < 1e-12);
    }

    #[test]
    fn oracle_nelder_mead_evaluation_counts() {
        // Java: x=[0.9999997362153239, -1.999999699107613] evals=95
        let mut p = problem(
            "z = (a - 1)^2 + (b + 2)^2",
            "z",
            &["a", "b"],
            &[-10.0, -10.0],
            &[10.0, 10.0],
        );
        p.method = Some("nelder-mead".to_string());
        let r = optimize(&p).unwrap();
        assert_eq!(r.evaluations, 95, "simplex probe sequence diverged");
        assert!((r.decision_values[0] - 0.999_999_736_215_323_9).abs() < 1e-9);
        assert!((r.decision_values[1] + 1.999_999_699_107_613).abs() < 1e-9);

        // Java: x=[1.0, 1.0] obj=32.0 evals=183 — the bounds-penalty path.
        let mut p = problem(
            "z = (a - 5)^2 + (b - 5)^2",
            "z",
            &["a", "b"],
            &[-1.0, -1.0],
            &[1.0, 1.0],
        );
        p.method = Some("simplex".to_string());
        let r = optimize(&p).unwrap();
        assert_eq!(r.evaluations, 183);
        assert_eq!(r.decision_values, vec![1.0, 1.0]);
        assert_eq!(r.objective_value, 32.0);

        // Java: x=[2.0] obj=3.0 evals=38 — one decision on the N-D path.
        let mut p = problem("y = (x - 2)^2 + 3", "y", &["x"], &[-10.0], &[10.0]);
        p.method = Some("nelder-mead".to_string());
        let r = optimize(&p).unwrap();
        assert_eq!(r.evaluations, 38);
        assert_eq!(r.decision_values, vec![2.0]);
    }

    #[test]
    fn oracle_constrained_runs() {
        // Java: x=[2.0] obj=9.0 evals=110
        let mut p = problem("y = (a - 5)^2", "y", &["a"], &[0.0], &[10.0]);
        p.method = Some("nelder-mead".to_string());
        p.constraints = vec!["a <= 2".to_string()];
        let r = optimize(&p).unwrap();
        assert_eq!(r.evaluations, 110, "outer/inner penalty loop diverged");
        assert!((r.decision_values[0] - 2.0).abs() < 1e-9);
        assert!(r.warning.is_none());

        // Java: x=[0.9999991764004255, 0.9999996515548462] evals=309
        let mut p = problem(
            "z = a^2 + b^2\ns = a + b",
            "z",
            &["a", "b"],
            &[-5.0, -5.0],
            &[5.0, 5.0],
        );
        p.method = Some("nelder-mead".to_string());
        p.constraints = vec!["a + b = 2".to_string()];
        let r = optimize(&p).unwrap();
        assert_eq!(r.evaluations, 309);
        assert!((r.decision_values[0] - 0.999_999_176_400_425_5).abs() < 1e-8);
        assert!((r.decision_values[1] - 0.999_999_651_554_846_2).abs() < 1e-8);
    }

    #[test]
    fn oracle_unsatisfiable_constraint_warning_is_word_for_word() {
        // Java: x=[0.0] obj=0.0 evals=492
        //       warning "Constraints not satisfied at the returned point:
        //                'a <= -50.0' is off by 50.00"
        let mut p = problem("y = a^2", "y", &["a"], &[0.0], &[10.0]);
        p.method = Some("nelder-mead".to_string());
        p.constraints = vec!["a <= -50".to_string()];
        let r = optimize(&p).unwrap();
        assert_eq!(r.evaluations, 492);
        assert_eq!(r.decision_values, vec![0.0]);
        assert_eq!(
            r.warning.as_deref(),
            Some(
                "Constraints not satisfied at the returned point: \
                 'a <= -50.0' is off by 50.00"
            )
        );
    }

    #[test]
    fn oracle_a_missing_objective_aborts_the_search_immediately() {
        // Java (both methods): SolverException "The objective variable 'q' is
        // not part of the system." — raised *outside* the try that swallows
        // solver failures, so it is not a PENALTY-scored probe.
        for method in ["brent", "nelder-mead"] {
            let mut p = problem("y = (x - 2)^2 + 3", "q", &["x"], &[-10.0], &[10.0]);
            p.method = Some(method.to_string());
            let err = optimize(&p).unwrap_err();
            assert_eq!(
                err.to_string_message(),
                "The objective variable 'q' is not part of the system.",
                "method = {method}"
            );
        }
    }

    #[test]
    fn java_double_text_matches_double_to_string() {
        assert_eq!(java_double_text(-50.0), "-50.0");
        assert_eq!(java_double_text(0.0), "0.0");
        assert_eq!(java_double_text(2.5), "2.5");
        assert_eq!(java_double_text(1e-4), "1.0E-4");
        assert_eq!(java_double_text(1e8), "1.0E8");
        assert_eq!(java_double_text(f64::NAN), "NaN");
        assert_eq!(java_double_text(f64::INFINITY), "Infinity");
    }

    #[test]
    fn format_g4_matches_java_percent_g() {
        assert_eq!(format_g4(50.0), "50.00");
        assert_eq!(format_g4(0.5), "0.5000");
        assert_eq!(format_g4(1234.5), "1234");
        assert_eq!(format_g4(0.0), "0.00000");
    }
}
