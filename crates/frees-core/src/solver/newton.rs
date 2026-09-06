//! Newton's method with an analytic-first Jacobian, step-halving and bounds.
//!
//! Port of `../frEES/backend/core/src/main/java/com/frees/backend/core/NewtonSolver.java`
//! (832 LOC) and `SolverSettings.java`.
//!
//! # The iteration
//!
//! Each Newton iteration
//!
//! 1. tests the residuals against the stop criteria (§ *Convergence* below) and
//!    returns as soon as they are met;
//! 2. builds the Jacobian — **analytically first** when the problem supplies
//!    one ([`NewtonProblem::analytic_jacobian`], the Java
//!    `NewtonSolver.computeJacobian` → `analyticalJacobian` path; a `None`
//!    from the source falls back to finite differences *for that iteration*,
//!    exactly as the Java returns `null` on an evaluation failure) —
//!    otherwise by **finite differences**, one column per unknown,
//!    with the parent engine's perturbation rule
//!    `h = jacobian_epsilon * max(|x_j|, 1)` — relative, with an absolute floor
//!    of one `jacobian_epsilon` so a variable sitting at zero is still probed.
//!    Probes are **range-aware**: a perturbation never leaves the variable's
//!    `[lo, hi]` box (the probe is clamped and the difference divided by the
//!    *actual* step taken; a direction pinned at its bound is skipped), so a
//!    property call is never probed out of range — the Java
//!    `computeJacobianColumn` §8.5 rule. A column whose forward probe lands in
//!    an invalid (non-finite) region is re-probed *backward*, and a column
//!    whose difference drowns in catastrophic cancellation is re-probed with a
//!    widened step (`× 1e4`, up to five times per direction), exactly as
//!    `NewtonSolver.computeJacobianColumn` does;
//! 3. solves `J·Δ = f` by **Gaussian elimination with partial pivoting** after
//!    column equilibration (each Jacobian column scaled to unit 2-norm, the step
//!    unscaled afterwards). Equilibration is the parent engine's §8.5 automatic
//!    scaling: it keeps a multidomain system mixing `P~1e5`, `ṁ~1`, `I~1e-3`
//!    conditioned well enough to factor, and is a no-op at the root. When a
//!    pivot collapses anyway, the **SVD pseudo-inverse of the equilibrated
//!    matrix** answers instead — the Java `solveLinear`'s
//!    `catch (SingularMatrixException)` arm (ledger item 40). Equilibration
//!    alone does not save a Jacobian whose columns differ by `1e12`: Commons
//!    Math's LU declares the equilibrated matrix singular at exactly the point
//!    this port's does, and the pseudo-inverse is what produces the step;
//! 4. **halves the step** until the residual norm decreases — `x - λ·Δ` with
//!    `λ = 1, ½, ¼, …`, up to [`SolverSettings::max_step_halvings`] halvings.
//!    **Every candidate is clamped into the per-variable `[lo, hi]` box**
//!    (Java `backtrackLineSearch`'s `Math.clamp`), so a bounded unknown never
//!    leaves its range at any point the residual is evaluated. This
//!    backtracking line search is what makes the engine robust against a
//!    Newton direction that overshoots (`atan`-like residuals, property calls
//!    that go invalid past the step);
//! 5. when neither the full step nor any halved step descends, falls back to a
//!    **damped (Levenberg–Marquardt) rescue**: solve
//!    `(JᵀJ + λ·diag(JᵀJ))·δ = Jᵀf`, escalating `λ` from `1e-3` to `1e12` until
//!    a norm-reducing candidate appears. Small `λ` recovers the Newton step;
//!    large `λ` gives a short, per-variable-scaled descent step. Damped
//!    candidates are clamped into the `[lo, hi]` box exactly like line-search
//!    ones (Java `dampedRescue`), and a candidate that cannot move because
//!    every variable is pinned ends the rescue. The rescue is also the last
//!    resort for a Jacobian the pseudo-inverse of step 3 could not turn into a
//!    descending step (the Java reaches its own `dampedRescue` the same way,
//!    through a non-descending `backtrackLineSearch`).
//!
//! # Convergence
//!
//! The Java stop criteria are *both* a residual test and a step test:
//!
//! * residual — `|r_i| / max(|lhs_i|, 1) <= relativeResiduals` for **every**
//!   equation, checked at the top of every iteration. A non-finite residual is
//!   explicitly *not* converged (`NaN > tol` is false and would otherwise be
//!   mistaken for a solution);
//! * step — when the largest change in any variable falls below
//!   `changeInVariables`, the iterate has frozen: that is a success if the
//!   residuals are within tolerance and a hard failure ("stalled") otherwise.
//!   A damped step is deliberately short, so its length never triggers this.
//!
//! This port sees only `f(x)`, never the two sides of each equation, so
//! `|lhs_i|` is unavailable. Its stand-in is the *row scale*
//! `s_i = max(1, Σ_j |J_ij · x_j|)` — the magnitude of the terms equation `i`
//! is built from, which is what `|lhs_i|` measures. An equation reading
//! `1e8·x = 2e8` therefore still converges at `1e-8`-sized residuals, exactly
//! as it does in Java, while a unit-scale equation is held to
//! `rel_tolerance`. Before the first Jacobian exists the scale is `1` (the
//! strictest choice).
//!
//! Failure to converge is an `Err`, mirroring the Java `SolverException`.

use crate::diag::{FreesError, Result};

/// Stop criteria, mirroring Options ▸ Preferences ▸ Stop Crit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SolverSettings {
    pub max_iterations: usize,
    /// Relative residual tolerance.
    pub rel_tolerance: f64,
    /// Absolute residual tolerance.
    pub abs_tolerance: f64,
    /// Maximum successive halvings of a Newton step before giving up.
    pub max_step_halvings: usize,
    /// Relative perturbation used to build the finite-difference Jacobian.
    pub jacobian_epsilon: f64,
    /// Complex mode (the Java `SolverSettings.complexMode`): the engine feeds
    /// this to [`crate::parser::complex::expand_complex`] before blocking.
    /// Newton itself never reads it — it rides here because the Java record
    /// carries it and the wire format will want the same knob.
    pub complex_mode: bool,
}

impl Default for SolverSettings {
    fn default() -> SolverSettings {
        // `SolverSettings.DEFAULTS` in the Java engine is
        // `(250, 1e-12, 1e-15, 3600.0)` — max iterations, relative residuals,
        // change in variables, elapsed-time budget. The first two are carried
        // over verbatim: a stop criterion of `1e-9` is the *verification*
        // tolerance the parity harness compares with (`PLAN.md` §"Numerical
        // tolerance policy"), and using it as the stop criterion too leaves the
        // solve no headroom — the canonical fixture then lands 1e-12 away from
        // the Java oracle instead of on it.
        //
        // `abs_tolerance` has no Java counterpart (the Java rule is purely
        // relative, with the row scale floored at 1). Keeping it strictly below
        // `rel_tolerance` makes it inert, which is the Java behaviour.
        //
        // `max_step_halvings` counts halvings, and the line search tries
        // `max_step_halvings + 1` values of λ, so 24 reproduces Java's
        // `MAX_HALVINGS = 25` trials (λ down to 2⁻²⁴).
        //
        // `jacobian_epsilon` deliberately does *not* match the Java
        // `JACOBIAN_EPS = sqrt(ulp(1.0)) ≈ 1.49e-8`: Java only reaches its
        // finite-difference path when `Differentiator` cannot differentiate a
        // residual, and this port has no symbolic path at all, so the larger
        // step is what keeps the widening heuristic informative here.
        SolverSettings {
            max_iterations: 250,
            rel_tolerance: 1e-12,
            abs_tolerance: 1e-15,
            max_step_halvings: 24,
            jacobian_epsilon: 1e-7,
            complex_mode: false,
        }
    }
}

/// The residual system one Newton solve iterates on, plus its optional
/// analytic Jacobian — the two callbacks `NewtonSolver.computeJacobian`
/// chooses between.
///
/// Plain closures `FnMut(&[f64], &mut [f64]) -> Result<()>` implement this
/// automatically (finite differences only), so simple callers and the
/// existing tests keep their shape.
pub trait NewtonProblem {
    /// Write `f(x)` into `out` (`out.len() == x.len()`).
    fn residual(&mut self, x: &[f64], out: &mut [f64]) -> Result<()>;

    /// The full Jacobian `J[i][j] = ∂f_i/∂x_j` evaluated at `x`, or `None` to
    /// fall back to finite differences **for this iteration**.
    ///
    /// This is the Java `NewtonSolver.analyticalJacobian` contract: the Java
    /// method returns `null` both when some residual cannot be symbolically
    /// differentiated at all (a permanent property — the engine then never
    /// supplies a source) and when a derivative expression fails to evaluate
    /// at the current point (a per-iteration property — the source returns
    /// `None` and the solver takes the numerical path just this once).
    fn analytic_jacobian(&mut self, _x: &[f64]) -> Option<Vec<Vec<f64>>> {
        None
    }

    /// Wave G3b: [`Self::analytic_jacobian`] writing into a caller-owned
    /// `n × n` buffer, so the hot path stops allocating a fresh matrix per
    /// iteration. Returns `false` to fall back to finite differences for this
    /// iteration, exactly like `analytic_jacobian` answering `None`.
    ///
    /// The default delegates to [`Self::analytic_jacobian`] — including the
    /// dimension check the solver used to apply to its result — so existing
    /// implementors keep their exact behaviour; the engine's hot implementor
    /// overrides this to evaluate straight into `out`.
    fn analytic_jacobian_into(&mut self, x: &[f64], out: &mut [Vec<f64>]) -> bool {
        let n = out.len();
        match self
            .analytic_jacobian(x)
            .filter(|j| j.len() == n && j.iter().all(|row| row.len() == n))
        {
            Some(j) => {
                for (slot, row) in out.iter_mut().zip(j) {
                    slot.copy_from_slice(&row);
                }
                true
            }
            None => false,
        }
    }

    /// For each unknown `j`, the equations that actually mention it — the Java
    /// `IterationContext.varToEquations`, built once per block.
    ///
    /// The finite-difference Jacobian uses this to touch only the rows a column
    /// can possibly be non-zero in. That is not an optimisation: a residual
    /// that is `NaN` for an unrelated reason (a property call at an invalid
    /// state point) would otherwise make *every* probe direction of *every*
    /// column read as "invalid region", abandon them all, and flatten the whole
    /// Jacobian to zero — at which point there is no descent direction left and
    /// the block stalls at iteration 0 instead of walking out of the bad
    /// region on the rows that are perfectly healthy.
    ///
    /// `None` means no structure is available and every row is used, which is
    /// the safe conservative answer for hand-built closures.
    fn row_dependencies(&self) -> Option<Vec<Vec<usize>>> {
        None
    }
}

impl<F> NewtonProblem for F
where
    F: FnMut(&[f64], &mut [f64]) -> Result<()>,
{
    fn residual(&mut self, x: &[f64], out: &mut [f64]) -> Result<()> {
        self(x, out)
    }
}

/// What one Newton solve did.
#[derive(Debug, Clone, PartialEq)]
pub struct NewtonReport {
    pub iterations: usize,
    /// Max absolute residual at the returned point.
    pub residual_norm: f64,
    pub converged: bool,
}

/// First damping tried when the undamped Newton step cannot descend.
const LM_LAMBDA_MIN: f64 = 1.0e-3;
/// Damping ceiling: beyond this the damped step is a vanishing nudge.
const LM_LAMBDA_MAX: f64 = 1.0e12;
/// Consecutive near-flat damped iterations before the mode gives up.
const CREEP_WINDOW: usize = 10;
/// An iteration is "creeping" when it reduces the norm by less than this.
const CREEP_REDUCTION: f64 = 1.0e-3;
/// `SolverSettings.changeInVariables` from the Java `DEFAULTS`: the iterate has
/// frozen once no variable moves by more than this.
const CHANGE_IN_VARIABLES: f64 = 1.0e-15;
/// A finite difference smaller than this many ulps of the residuals it came
/// from is catastrophic cancellation, not signal.
const CANCELLATION_ULPS: f64 = 1.0e5;
/// Widening applied to a perturbation that produced only cancellation.
const WIDEN_FACTOR: f64 = 1.0e4;
/// Widening attempts per probe direction.
const MAX_WIDENINGS: usize = 5;
/// A pivot below this fraction of the largest matrix entry means singular. It
/// matches the `1e-11` singularity threshold Apache Commons Math's
/// `LUDecomposition` applies to the equilibrated matrix in the Java original.
const SINGULARITY_RATIO: f64 = 1.0e-11;

/// Solve `residual(x) = 0` in place, starting from `x`.
///
/// The closure form: finite differences only. To supply an analytic Jacobian
/// source alongside the residual, implement [`NewtonProblem`] and call
/// [`newton_solve_problem`] — this function is that one behind the blanket
/// closure impl.
///
/// `bounds` are the per-variable boxes `[lo, hi]`, aligned with `x`; `None`
/// means unbounded everywhere. When bounds are given the iterate — and
/// **every point the residual is evaluated at**, line-search candidates,
/// damped-rescue candidates and finite-difference probes alike — stays inside
/// the box, reproducing the three `Math.clamp` sites of the Java
/// `NewtonSolver` (`backtrackLineSearch`, `dampedRescue`,
/// `computeJacobianColumn`).
///
/// On success `x` holds the solution. On failure `x` holds the last iterate
/// reached — the Java engine likewise leaves its partially updated values in
/// the variable map — which is what makes a stall report actionable.
///
/// # Errors
///
/// * the callback's own error, propagated unchanged and immediately;
/// * [`FreesError::Solver`] when the iteration limit is exhausted, when no step
///   (full, halved or damped) can reduce the residual, when a bounded solve
///   pins a variable at its bound with the residuals still out of tolerance
///   (the Java "Constrained solution" diagnosis), or when the settings or
///   bounds are unusable.
pub fn newton_solve<F>(
    residual: F,
    x: &mut [f64],
    settings: &SolverSettings,
    bounds: Option<&[(f64, f64)]>,
) -> Result<NewtonReport>
where
    F: FnMut(&[f64], &mut [f64]) -> Result<()>,
{
    // The direct `FnMut` bound (rather than `P: NewtonProblem`) is what lets
    // plain closures infer their higher-ranked argument lifetimes; the
    // blanket impl turns them into an FD-only problem.
    newton_solve_problem(residual, x, settings, bounds)
}

/// [`newton_solve`] for a full [`NewtonProblem`] — the entry the engine uses,
/// where the analytic-Jacobian source rides along with the residual.
pub fn newton_solve_problem<P>(
    problem: P,
    x: &mut [f64],
    settings: &SolverSettings,
    bounds: Option<&[(f64, f64)]>,
) -> Result<NewtonReport>
where
    P: NewtonProblem,
{
    // A fresh workspace per call is exactly the buffers this function
    // allocated itself before Wave I, so every existing caller is untouched.
    newton_solve_problem_with(
        problem,
        x,
        settings,
        bounds,
        &mut NewtonWorkspace::default(),
    )
}

/// [`newton_solve_problem`] with caller-owned cross-call buffers (Wave I).
///
/// The per-step transient path drives thousands of warm-started solves per
/// integration, and most converge at iteration 0–1 (the G3b finding): what
/// remained after per-iteration reuse was the four per-CALL vectors (`f`,
/// `scale`, `lo`, `hi`) plus the lazily created [`Scratch`], all rebuilt on
/// every call. This entry reuses them from `ws` across calls.
///
/// Soundness is the G3b rule extended across the call boundary: every buffer
/// is fully rewritten before it is read. `f` is NaN-refilled here and
/// NaN-poisoned again by [`eval_residual`], `scale` refilled with `1.0`,
/// `lo`/`hi` refilled from `bounds` under the same validation and errors as
/// before, and every [`Scratch`] use site zero-fills or overwrites before
/// reading — the same property that already made iteration-to-iteration
/// reuse sound. A workspace whose `Scratch` was sized for a different `n` is
/// dropped and lazily rebuilt, so the entry stays safe for any caller.
pub(crate) fn newton_solve_problem_with<P>(
    mut problem: P,
    x: &mut [f64],
    settings: &SolverSettings,
    bounds: Option<&[(f64, f64)]>,
    ws: &mut NewtonWorkspace,
) -> Result<NewtonReport>
where
    P: NewtonProblem,
{
    validate(settings)?;

    let n = x.len();
    if n == 0 {
        // A block with no unknowns is trivially satisfied: nothing to perturb
        // and nothing to factor.
        return Ok(NewtonReport {
            iterations: 0,
            residual_norm: 0.0,
            converged: true,
        });
    }

    let NewtonWorkspace {
        lo,
        hi,
        f,
        scale,
        scratch: scratch_store,
    } = ws;

    // Java IterationContext: lo/hi arrays from the specs, ±∞ where absent.
    unpack_bounds_into(bounds, n, lo, hi)?;
    let (lo, hi): (&[f64], &[f64]) = (lo, hi);

    f.clear();
    f.resize(n, f64::NAN);
    eval_residual(&mut problem, x, f)?;
    let mut norm = l2_norm(f);

    // Stand-in for the Java |lhs| of every equation; see the module docs.
    scale.clear();
    scale.resize(n, 1.0);
    // Damping carried between iterations; 0 = pure Newton.
    let mut lambda = 0.0f64;
    // Consecutive damped iterations with near-flat progress.
    let mut creep = 0usize;
    // Why the last linear stage failed, for the diagnostic.
    let mut linear_failure: Option<LinearFailure> = None;
    // G3b: every buffer below is reused across iterations and trials — and
    // created only when an iteration actually runs, so the warm-started
    // solves that pass `within_tolerance` at iteration 0 allocate nothing.
    // Wave I extends the reuse across calls when the caller keeps the
    // workspace; a stale size (one workspace shared across differently sized
    // blocks) is reset rather than trusted.
    if scratch_store.as_ref().is_some_and(|s| s.step.len() != n) {
        *scratch_store = None;
    }

    for iteration in 0..settings.max_iterations {
        if let Some(message) = crate::ode::deadline::strike() {
            return Err(FreesError::solver(message));
        }
        if within_tolerance(f, scale, settings) {
            return Ok(success(iteration, f));
        }
        let scratch = scratch_store.get_or_insert_with(|| Scratch::new(n));

        // Java NewtonSolver.computeJacobian: analytical first, numerical
        // fallback. A source that answers `false` (a derivative expression
        // failing to evaluate at this point) falls back for this iteration
        // only, exactly like the Java `analyticalJacobian` returning null
        // from its catch block. The dimension guard the solver used to apply
        // to the returned matrix lives in `analytic_jacobian_into`'s default.
        if !problem.analytic_jacobian_into(x, &mut scratch.jacobian) {
            scratch.ensure_numerical(n);
            let Scratch {
                jacobian,
                nj_probe,
                nj_out,
                ..
            } = &mut *scratch;
            numerical_jacobian_into(
                &mut problem,
                x,
                f,
                settings,
                lo,
                hi,
                jacobian,
                nj_probe,
                nj_out,
            )?;
        }

        let mut damped = false;
        let accepted: Accepted;

        if lambda > 0.0 {
            // Damped mode: keep taking damped steps, relaxing the damping while
            // they descend (the rescue escalates it again on its own when the
            // relaxed value fails). Once the damping bottoms out, hand the
            // iteration back to pure Newton. Grinding out only micro-descents
            // means the iterate sits at a local minimum of the residual norm —
            // no damping escapes that, so stop instead of burning iterations.
            if creep >= CREEP_WINDOW {
                return stalled(f, scale, settings, iteration, norm, linear_failure);
            }
            scratch.ensure_rescue(n);
            match damped_rescue(
                &mut problem,
                &scratch.jacobian,
                x,
                f,
                norm,
                lambda / 3.0,
                1.0,
                lo,
                hi,
                RescueBufs {
                    jtj: &mut scratch.jtj,
                    jtr: &mut scratch.jtr,
                    damped_a: &mut scratch.damped_a,
                    damped_b: &mut scratch.damped_b,
                    damped_y: &mut scratch.damped_y,
                    candidate: &mut scratch.candidate,
                    candidate_residual: &mut scratch.candidate_residual,
                },
            )? {
                None => return stalled(f, scale, settings, iteration, norm, linear_failure),
                Some(rescue) => {
                    lambda = if rescue.lambda <= LM_LAMBDA_MIN {
                        0.0
                    } else {
                        rescue.lambda
                    };
                    creep = if rescue.norm > norm * (1.0 - CREEP_REDUCTION) {
                        creep + 1
                    } else {
                        0
                    };
                    damped = true;
                    accepted = Accepted { norm: rescue.norm };
                }
            }
        } else {
            let mut taken: Option<Accepted> = None;
            let solved = {
                let Scratch {
                    jacobian,
                    lin_d,
                    lin_scaled,
                    lin_b,
                    lin_y,
                    step,
                    ..
                } = &mut *scratch;
                solve_linear_into(jacobian, f, lin_d, lin_scaled, lin_b, lin_y, step)
            };
            match solved {
                Ok(path) => {
                    // A pseudo-inverse direction is taken like any other, but
                    // it is remembered: if this iteration turns out to be the
                    // one that stalls, the stall message should still name the
                    // Jacobian that could not be factored.
                    linear_failure = match path {
                        LinearPath::Factored => None,
                        LinearPath::PseudoInverse => Some(LinearFailure::Singular),
                    };
                    let search = {
                        let Scratch {
                            step,
                            candidate,
                            candidate_residual,
                            ..
                        } = &mut *scratch;
                        backtrack_line_search(
                            &mut problem,
                            x,
                            step,
                            norm,
                            settings,
                            lo,
                            hi,
                            candidate,
                            candidate_residual,
                        )?
                    };
                    // Transcribed from the Java rescue condition
                    // `!isFinite(candidateNorm) || candidateNorm >= norm`. The
                    // `>=` is deliberate: it is false when `norm` is NaN, so a
                    // finite candidate is allowed to rescue a non-finite point.
                    let needs_rescue = !search.norm.is_finite() || search.norm >= norm;
                    if !needs_rescue {
                        taken = Some(search);
                    }
                }
                Err(kind) => linear_failure = Some(kind),
            }
            match taken {
                Some(step) => accepted = step,
                None => {
                    // The full and every halved Newton step fail to descend (or
                    // there was no usable Newton direction at all). Everything
                    // the damped mode does from here can only improve on the
                    // stall the undamped iteration already reached; a
                    // descending undamped iteration is never interfered with.
                    scratch.ensure_rescue(n);
                    match damped_rescue(
                        &mut problem,
                        &scratch.jacobian,
                        x,
                        f,
                        norm,
                        0.0,
                        1.0,
                        lo,
                        hi,
                        RescueBufs {
                            jtj: &mut scratch.jtj,
                            jtr: &mut scratch.jtr,
                            damped_a: &mut scratch.damped_a,
                            damped_b: &mut scratch.damped_b,
                            damped_y: &mut scratch.damped_y,
                            candidate: &mut scratch.candidate,
                            candidate_residual: &mut scratch.candidate_residual,
                        },
                    )? {
                        None => {
                            return stalled(f, scale, settings, iteration, norm, linear_failure)
                        }
                        Some(rescue) => {
                            lambda = rescue.lambda;
                            creep = 0;
                            damped = true;
                            accepted = Accepted { norm: rescue.norm };
                        }
                    }
                }
            }
        }

        // Every path that reaches here left the accepted trial point in
        // `scratch.candidate`/`candidate_residual` — the line search's
        // breaking trial, or the rescue's accepted one (a rescue that
        // overwrote a failed search's buffers is exactly the step taken).
        let mut max_change = 0.0f64;
        for (slot, &moved_to) in x.iter_mut().zip(scratch.candidate.iter()) {
            max_change = max_change.max((*slot - moved_to).abs());
            *slot = moved_to;
        }
        f.copy_from_slice(&scratch.candidate_residual);
        norm = accepted.norm;
        row_scale(&scratch.jacobian, x, scale);

        // Stop criterion: change in variables below the threshold. A damped step
        // is deliberately short, so its size says nothing about convergence —
        // only an undamped step may trigger this stop.
        if !damped && max_change < CHANGE_IN_VARIABLES {
            if within_tolerance(f, scale, settings) {
                return Ok(success(iteration + 1, f));
            }
            // Java NewtonSolver.handleConvergenceOrPinning: an iterate frozen
            // on a bound with the residuals still out of tolerance is a
            // *constrained* solution — the bounds, not the guesses, are what
            // the user must relax.
            if at_bound(x, lo, hi) {
                return Err(FreesError::solver(
                    "Constrained solution: a variable is pinned at its lower or upper bound \
                     and the residuals cannot be reduced further. \
                     Relax the bounds in the Variable Information window.",
                ));
            }
            return Err(FreesError::solver(format!(
                "Newton iteration stalled after {} iteration(s): the iterate stopped moving \
                 (largest change {:e}) with the residuals still outside the stop criteria \
                 (norm {:e}).{} Try different guess values.",
                iteration + 1,
                max_change,
                norm,
                failure_suffix(linear_failure),
            )));
        }
    }

    if within_tolerance(f, scale, settings) {
        return Ok(success(settings.max_iterations, f));
    }
    Err(FreesError::solver(format!(
        "Newton's method did not converge within {} iterations (residual norm {:e}). \
         Increase the iteration limit or adjust the guess values.",
        settings.max_iterations, norm,
    )))
}

/// The line search can make no further progress. If the residuals are already
/// within tolerance this is a converged solution sitting at its numerical floor
/// (common when a residual depends on a finite-difference-coupled term) —
/// accept it rather than fail.
fn stalled(
    f: &[f64],
    scale: &[f64],
    settings: &SolverSettings,
    iteration: usize,
    norm: f64,
    linear: Option<LinearFailure>,
) -> Result<NewtonReport> {
    if within_tolerance(f, scale, settings) {
        return Ok(success(iteration + 1, f));
    }
    Err(FreesError::solver(format!(
        "Newton iteration stalled after {} iteration(s): no full, halved or damped step \
         reduces the residual (norm {:e}).{} Try different guess values.",
        iteration,
        norm,
        failure_suffix(linear),
    )))
}

fn failure_suffix(linear: Option<LinearFailure>) -> String {
    match linear {
        None => String::new(),
        Some(kind) => format!(" {}", kind.describe()),
    }
}

fn success(iterations: usize, f: &[f64]) -> NewtonReport {
    NewtonReport {
        iterations,
        residual_norm: max_abs(f),
        converged: true,
    }
}

fn validate(settings: &SolverSettings) -> Result<()> {
    if settings.max_iterations < 1 {
        return Err(FreesError::solver(
            "Number of iterations must be at least 1.",
        ));
    }
    // `is_finite` first, so a NaN tolerance is rejected rather than silently
    // failing every comparison it takes part in.
    if !settings.rel_tolerance.is_finite() || settings.rel_tolerance <= 0.0 {
        return Err(FreesError::solver("Stop criteria values must be positive."));
    }
    if !settings.abs_tolerance.is_finite() || settings.abs_tolerance < 0.0 {
        return Err(FreesError::solver("Stop criteria values must be positive."));
    }
    if !settings.jacobian_epsilon.is_finite() || settings.jacobian_epsilon <= 0.0 {
        return Err(FreesError::solver(
            "The Jacobian perturbation must be a positive, finite fraction.",
        ));
    }
    Ok(())
}

/// Split the caller's bounds into the Java `IterationContext.lo`/`hi` arrays,
/// rejecting shapes that would poison every clamp: a length mismatch, a NaN
/// bound, or `lo > hi` (the Java `VariableSpec` constructor refuses those
/// upstream; this is the same guarantee at the solver's own door).
///
/// Wave I: fills the caller's buffers instead of allocating — same checks,
/// same errors, in the same order. On an error the buffers are left
/// part-filled, which is fine: every entry re-fills them from scratch.
fn unpack_bounds_into(
    bounds: Option<&[(f64, f64)]>,
    n: usize,
    lo: &mut Vec<f64>,
    hi: &mut Vec<f64>,
) -> Result<()> {
    lo.clear();
    hi.clear();
    match bounds {
        None => {
            lo.resize(n, f64::NEG_INFINITY);
            hi.resize(n, f64::INFINITY);
        }
        Some(pairs) => {
            if pairs.len() != n {
                return Err(FreesError::solver(format!(
                    "internal error: {} bounds for {} unknowns",
                    pairs.len(),
                    n
                )));
            }
            lo.reserve(n);
            hi.reserve(n);
            for &(l, h) in pairs {
                if l.is_nan() || h.is_nan() || l > h {
                    return Err(FreesError::solver(
                        "Lower bound exceeds upper bound for a variable.",
                    ));
                }
                lo.push(l);
                hi.push(h);
            }
        }
    }
    Ok(())
}

/// Java `NewtonSolver.atBound`: any variable sitting exactly on its lower or
/// upper bound. The finiteness guard is ours: an unbounded variable that
/// diverged to ±∞ must not read as "pinned at its bound".
fn at_bound(x: &[f64], lo: &[f64], hi: &[f64]) -> bool {
    x.iter()
        .zip(lo.iter().zip(hi))
        .any(|(&v, (&l, &h))| v.is_finite() && (v == l || v == h))
}

/// Calls the user residual with a poisoned output buffer, so an entry the
/// callback forgets to write can never masquerade as a converged residual.
fn eval_residual<P>(problem: &mut P, x: &[f64], out: &mut [f64]) -> Result<()>
where
    P: NewtonProblem + ?Sized,
{
    for slot in out.iter_mut() {
        *slot = f64::NAN;
    }
    problem.residual(x, out)
}

/// `|r_i|` measured against the Java rule, with the row scale standing in for
/// `|lhs_i|`. Non-finite residuals are explicitly not converged.
fn within_tolerance(f: &[f64], scale: &[f64], settings: &SolverSettings) -> bool {
    for (i, &r) in f.iter().enumerate() {
        if !r.is_finite() {
            return false;
        }
        let s = scale.get(i).copied().unwrap_or(1.0);
        let tolerance = settings.abs_tolerance.max(settings.rel_tolerance * s);
        if r.abs() > tolerance {
            return false;
        }
    }
    true
}

/// `s_i = max(1, Σ_j |J_ij · x_j|)`: the magnitude of the terms equation `i` is
/// assembled from, standing in for the Java `|lhs_i|`.
fn row_scale(jacobian: &[Vec<f64>], x: &[f64], out: &mut [f64]) {
    for (i, row) in jacobian.iter().enumerate() {
        let mut s = 0.0f64;
        for (j, &v) in row.iter().enumerate() {
            let term = (v * x[j]).abs();
            if term.is_finite() {
                s += term;
            }
        }
        out[i] = if s.is_finite() { s.max(1.0) } else { 1.0 };
    }
}

fn l2_norm(v: &[f64]) -> f64 {
    let mut sum = 0.0f64;
    for &value in v {
        sum += value * value;
    }
    sum.sqrt()
}

fn max_abs(v: &[f64]) -> f64 {
    let mut m = 0.0f64;
    for &value in v {
        if value.is_nan() {
            return f64::NAN;
        }
        m = m.max(value.abs());
    }
    m
}

/// `Math.ulp`: the distance from `|v|` to the next representable double.
fn ulp(v: f64) -> f64 {
    let a = v.abs();
    if a.is_nan() || a == f64::INFINITY {
        return a;
    }
    if a == 0.0 {
        return f64::from_bits(1); // Double.MIN_VALUE
    }
    let bits = a.to_bits();
    let up = f64::from_bits(bits + 1) - a;
    if up.is_finite() && up > 0.0 {
        up
    } else {
        // `a` is f64::MAX: step downwards instead of overflowing to infinity.
        a - f64::from_bits(bits - 1)
    }
}

/// Wave I: the buffers one [`newton_solve_problem_with`] call needs beyond
/// [`Scratch`] — the per-call vectors that used to be freshly allocated at
/// every entry — bundled so a caller on the per-step transient path can keep
/// them alive across thousands of warm-started solves. See
/// [`newton_solve_problem_with`] for the reuse-soundness argument.
///
/// `Default` is the correct "fresh" state: every field is filled (or, for
/// `scratch`, lazily created) before use.
#[derive(Default)]
pub(crate) struct NewtonWorkspace {
    lo: Vec<f64>,
    hi: Vec<f64>,
    f: Vec<f64>,
    scale: Vec<f64>,
    scratch: Option<Scratch>,
}

/// A candidate point, its residuals and their 2-norm.
#[derive(Debug, Clone)]
/// Wave G3b: every buffer one Newton solve reuses across iterations, halvings
/// and damping trials. Before this, the callgrind profile of the per-step
/// hostile regime had ~20 % of all instructions in `Vec` allocation inside
/// `solve_block` — a fresh Jacobian per iteration, five vectors per linear
/// solve, an accepted-step pair per line search, and a `jtj.clone()` per
/// damping trial. Reuse changes no arithmetic: every buffer is fully
/// (re)written before it is read, and the fill orders are the old
/// constructors' orders.
struct Scratch {
    /// The iteration's Jacobian (`n × n`), analytic or finite-difference.
    jacobian: Vec<Vec<f64>>,
    /// `solve_linear`: column scales, the equilibrated copy, the gauss
    /// right-hand side, the unscaled step, and gauss's solution vector.
    lin_d: Vec<f64>,
    lin_scaled: Vec<Vec<f64>>,
    lin_b: Vec<f64>,
    lin_y: Vec<f64>,
    step: Vec<f64>,
    /// The line search's / rescue's current trial point and its residuals.
    /// On an accepted step the loop copies them into `x` and `f`.
    candidate: Vec<f64>,
    candidate_residual: Vec<f64>,
    /// `damped_rescue`: the normal equations and the per-trial damped copy.
    jtj: Vec<Vec<f64>>,
    jtr: Vec<f64>,
    damped_a: Vec<Vec<f64>>,
    damped_b: Vec<f64>,
    damped_y: Vec<f64>,
    /// `numerical_jacobian`: the probe point and its residuals.
    nj_probe: Vec<f64>,
    nj_out: Vec<f64>,
}

impl Scratch {
    /// The hot-path buffers only. The rescue and finite-difference sets stay
    /// empty until their paths run ([`Self::ensure_rescue`],
    /// [`Self::ensure_numerical`]): a warm-started per-step solve converges at
    /// iteration 0 or 1 and must not pay for buffers the failure paths own —
    /// eagerly allocating all fifteen was measured as a 2× *regression* on
    /// the stiff stand-in before this split.
    fn new(n: usize) -> Scratch {
        Scratch {
            jacobian: vec![vec![0.0; n]; n],
            lin_d: vec![0.0; n],
            lin_scaled: vec![vec![0.0; n]; n],
            lin_b: vec![0.0; n],
            lin_y: vec![0.0; n],
            step: vec![0.0; n],
            candidate: vec![0.0; n],
            candidate_residual: vec![0.0; n],
            jtj: Vec::new(),
            jtr: Vec::new(),
            damped_a: Vec::new(),
            damped_b: Vec::new(),
            damped_y: Vec::new(),
            nj_probe: Vec::new(),
            nj_out: Vec::new(),
        }
    }

    /// Size the damped-rescue buffers on first use.
    fn ensure_rescue(&mut self, n: usize) {
        if self.jtr.len() != n {
            self.jtj = vec![vec![0.0; n]; n];
            self.jtr = vec![0.0; n];
            self.damped_a = vec![vec![0.0; n]; n];
            self.damped_b = vec![0.0; n];
            self.damped_y = vec![0.0; n];
        }
    }

    /// Size the finite-difference buffers on first use.
    fn ensure_numerical(&mut self, n: usize) {
        if self.nj_probe.len() != n {
            self.nj_probe = vec![0.0; n];
            self.nj_out = vec![0.0; n];
        }
    }
}

impl Scratch {
    /// All of `damped_rescue`'s buffers at once — for tests that call the
    /// rescue directly (the solver loop builds `RescueBufs` field-by-field
    /// because it borrows `jacobian` from the same struct).
    #[cfg(test)]
    fn rescue_bufs(&mut self) -> RescueBufs<'_> {
        let n = self.step.len();
        self.ensure_rescue(n);
        RescueBufs {
            jtj: &mut self.jtj,
            jtr: &mut self.jtr,
            damped_a: &mut self.damped_a,
            damped_b: &mut self.damped_b,
            damped_y: &mut self.damped_y,
            candidate: &mut self.candidate,
            candidate_residual: &mut self.candidate_residual,
        }
    }
}

/// An accepted step's norm — its point and residuals sit in
/// [`Scratch::candidate`] / [`Scratch::candidate_residual`].
struct Accepted {
    norm: f64,
}

/// An accepted damped step and the damping value that produced it.
#[derive(Debug, Clone)]
struct DampedStep {
    norm: f64,
    lambda: f64,
}

/// The [`Scratch`] buffers `damped_rescue` borrows — a struct rather than
/// seven parameters, so the call sites stay readable and the split borrows
/// happen in one place.
struct RescueBufs<'a> {
    jtj: &'a mut [Vec<f64>],
    jtr: &'a mut [f64],
    damped_a: &'a mut [Vec<f64>],
    damped_b: &'a mut [f64],
    damped_y: &'a mut [f64],
    candidate: &'a mut [f64],
    candidate_residual: &'a mut [f64],
}

/// Why a linear stage could not produce a Newton direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LinearFailure {
    /// A pivot collapsed: the Jacobian is singular or too ill-conditioned to
    /// factor at this point.
    Singular,
    /// The Jacobian or the residual carries `NaN`/`Inf`.
    NonFinite,
}

impl LinearFailure {
    fn describe(self) -> &'static str {
        match self {
            LinearFailure::Singular => {
                "The Jacobian is singular or too ill-conditioned to factor at this point: \
                 two equations are locally indistinguishable, or an unknown has no local \
                 influence on any residual."
            }
            LinearFailure::NonFinite => {
                "The Jacobian or the residuals are not finite at this point (a function was \
                 evaluated outside its valid range)."
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Finite-difference Jacobian
// ---------------------------------------------------------------------------

/// `J[i][j] = ∂f_i/∂x_j` by finite differences: one full residual sweep per
/// unknown, plus retries for columns that come back invalid or cancelling.
#[cfg(test)]
fn numerical_jacobian<P>(
    problem: &mut P,
    x: &[f64],
    base: &[f64],
    settings: &SolverSettings,
    lo: &[f64],
    hi: &[f64],
) -> Result<Vec<Vec<f64>>>
where
    P: NewtonProblem + ?Sized,
{
    let mut scratch = Scratch::new(x.len());
    scratch.ensure_numerical(x.len());
    let Scratch {
        jacobian,
        nj_probe,
        nj_out,
        ..
    } = &mut scratch;
    numerical_jacobian_into(
        problem, x, base, settings, lo, hi, jacobian, nj_probe, nj_out,
    )?;
    Ok(scratch.jacobian)
}

/// See the test-facing wrapper above: the same sweep into reused buffers
/// (G3b). `jacobian` is zero-filled here, exactly as the fresh matrix was.
#[allow(clippy::too_many_arguments)]
fn numerical_jacobian_into<P>(
    problem: &mut P,
    x: &[f64],
    base: &[f64],
    settings: &SolverSettings,
    lo: &[f64],
    hi: &[f64],
    jacobian: &mut [Vec<f64>],
    probe: &mut [f64],
    out: &mut [f64],
) -> Result<()>
where
    P: NewtonProblem + ?Sized,
{
    let n = x.len();
    for row in jacobian.iter_mut() {
        row.fill(0.0);
    }
    probe.copy_from_slice(x);
    out.fill(f64::NAN);
    let dependents = problem.row_dependencies();
    for j in 0..n {
        jacobian_column(
            problem,
            x,
            base,
            jacobian,
            probe,
            out,
            j,
            settings,
            (lo[j], hi[j]),
            dependents.as_ref().map(|rows| rows[j].as_slice()),
        )?;
    }
    Ok(())
}

/// One finite-difference column, ported from `computeJacobianColumn`.
///
/// Probe forward first, then backward; within a direction widen a few times to
/// escape catastrophic cancellation — but abandon the direction as soon as a
/// probe lands in an invalid (non-finite) region, because growing the step only
/// marches further into it. Perturbation is **range-aware** (the Java §8.5
/// rule): a direction pinned at its bound is skipped outright, every probe is
/// clamped into `[lo, hi]` — so a property call is never evaluated out of
/// range — and the difference is divided by the *actual* step taken. If no
/// informative derivative exists in either direction the column is left finite
/// (zero: no local sensitivity) so the linear solve still produces a finite
/// step.
#[allow(clippy::too_many_arguments)]
fn jacobian_column<P>(
    problem: &mut P,
    x: &[f64],
    base: &[f64],
    jacobian: &mut [Vec<f64>],
    probe: &mut [f64],
    out: &mut [f64],
    j: usize,
    settings: &SolverSettings,
    (lo, hi): (f64, f64),
    dependents: Option<&[usize]>,
) -> Result<()>
where
    P: NewtonProblem + ?Sized,
{
    let n = x.len();

    // The rows this column can be non-zero in — the Java `deps` list. Rows
    // outside it are structural zeros, and reading them would let one invalid
    // residual elsewhere in the block condemn this column. See
    // [`NewtonProblem::row_dependencies`].
    let all_rows: Vec<usize>;
    let rows: &[usize] = match dependents {
        Some(rows) => rows,
        None => {
            all_rows = (0..n).collect();
            &all_rows
        }
    };
    if rows.is_empty() {
        return Ok(()); // the variable appears in no equation: its column is 0
    }

    let anchor = x[j];
    let magnitude = settings.jacobian_epsilon * anchor.abs().max(1.0);

    for direction in 0..2 {
        let sign = if direction == 0 { 1.0 } else { -1.0 };
        // Java computeJacobianColumn: a direction already pinned at its bound
        // has nowhere to probe.
        if sign > 0.0 && anchor >= hi {
            continue;
        }
        if sign < 0.0 && anchor <= lo {
            continue;
        }
        let mut h = magnitude;
        for _ in 0..MAX_WIDENINGS {
            let point = (anchor + sign * h).clamp(lo, hi);
            if !point.is_finite() {
                break;
            }
            let actual_step = point - anchor;
            if actual_step == 0.0 {
                // Clamped onto the bound (or the perturbation vanished in
                // rounding) — this direction is exhausted.
                break;
            }

            probe[j] = point;
            let call = eval_residual(problem, probe, out);
            probe[j] = anchor;
            call?;

            let mut finite = true;
            let mut any_change = false;
            let mut cancellation = false;
            for &i in rows {
                let perturbed = out[i];
                if !perturbed.is_finite() {
                    finite = false;
                    break;
                }
                let change = (perturbed - base[i]).abs();
                if change > 0.0 {
                    any_change = true;
                    let ulp_scale = ulp(perturbed.abs().max(base[i].abs()));
                    if change < CANCELLATION_ULPS * ulp_scale {
                        cancellation = true;
                    }
                }
            }
            if !finite {
                break; // invalid region — try the other direction
            }

            // Commit this finite estimate; a later, cleaner attempt overwrites it.
            for &i in rows {
                jacobian[i][j] = (out[i] - base[i]) / actual_step;
            }
            if any_change && !cancellation {
                return Ok(()); // clean, informative column
            }
            h *= WIDEN_FACTOR; // finite but noisy/cancelling — widen
        }
    }

    // No informative derivative in either direction. Guarantee a finite column.
    for &i in rows {
        if !jacobian[i][j].is_finite() {
            jacobian[i][j] = 0.0;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Linear algebra
// ---------------------------------------------------------------------------

/// Solves `J·Δ = f` for the Newton step `Δ` (the iterate then moves to
/// `x - λ·Δ`), with column equilibration around a partial-pivoting Gaussian
/// elimination — and, when a pivot collapses, around the SVD pseudo-inverse
/// the Java falls back to instead ([`svd_fallback`]).
#[cfg(test)]
fn solve_linear(
    jacobian: &[Vec<f64>],
    rhs: &[f64],
) -> std::result::Result<Vec<f64>, LinearFailure> {
    let n = rhs.len();
    let mut scratch = Scratch::new(n);
    let Scratch {
        lin_d,
        lin_scaled,
        lin_b,
        lin_y,
        step,
        ..
    } = &mut scratch;
    solve_linear_into(jacobian, rhs, lin_d, lin_scaled, lin_b, lin_y, step)?;
    Ok(scratch.step)
}

/// See [`solve_linear`] (the test-facing wrapper): the same computation into
/// the caller's reused buffers (G3b), each fully rewritten before use.
#[allow(clippy::too_many_arguments)]
fn solve_linear_into(
    jacobian: &[Vec<f64>],
    rhs: &[f64],
    d: &mut [f64],
    scaled: &mut [Vec<f64>],
    b: &mut [f64],
    y: &mut [f64],
    step: &mut [f64],
) -> std::result::Result<LinearPath, LinearFailure> {
    let n = rhs.len();
    // Column equilibration (§8.5 automatic scaling): scale each unknown's column
    // to unit norm, then unscale the step (Δx = D·y). A no-op at the root, but
    // it keeps the factorization accurate where the raw Jacobian mixes wildly
    // different magnitudes.
    d.fill(1.0);
    for (j, slot) in d.iter_mut().enumerate() {
        let mut c = 0.0f64;
        for row in jacobian.iter().take(n) {
            let v = row[j];
            c += v * v;
        }
        let c = c.sqrt();
        let inv = 1.0 / c;
        if c > 0.0 && c.is_finite() && inv.is_finite() {
            *slot = inv;
        }
    }
    for i in 0..n {
        for j in 0..n {
            scaled[i][j] = jacobian[i][j] * d[j];
        }
    }
    b.copy_from_slice(rhs);

    let path = match gauss_solve_into(scaled, b, y) {
        Ok(()) => LinearPath::Factored,
        // The Java `solveLinear` catches Commons Math's
        // `SingularMatrixException` and re-solves the *same equilibrated*
        // matrix through the SVD pseudo-inverse; only then does the caller see
        // a step. Transcribing that catch is ledger item 40 — equilibration is
        // not what rescues a Jacobian whose columns differ by 1e12, because
        // Commons Math's LU calls the equilibrated matrix singular too (its
        // pivot threshold is an absolute 1e-11, ours the same ratio of the
        // largest entry). The pseudo-inverse is what rescues it.
        Err(LinearFailure::Singular) => {
            svd_fallback(jacobian, rhs, d, y)?;
            LinearPath::PseudoInverse
        }
        Err(other) => return Err(other),
    };

    for j in 0..n {
        let v = d[j] * y[j];
        if !v.is_finite() {
            return Err(LinearFailure::NonFinite);
        }
        step[j] = v;
    }
    Ok(path)
}

/// How the linear stage produced its direction. The distinction is carried out
/// of [`solve_linear_into`] for the *diagnostic* only: a pseudo-inverse step is
/// used exactly like a factored one, but if the iteration later stalls the
/// message should still say the Jacobian could not be factored, which is the
/// one fact a user can act on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LinearPath {
    /// Partial-pivoting elimination on the equilibrated matrix.
    Factored,
    /// A pivot collapsed and the SVD pseudo-inverse answered instead.
    PseudoInverse,
}

/// The Java `solveLinear`'s `catch (SingularMatrixException)` arm: the SVD
/// pseudo-inverse of the equilibrated Jacobian, writing the scaled solution
/// into `y` for the caller to unscale.
///
/// `gauss_solve_into` eliminates in place, so the equilibrated matrix is
/// rebuilt here rather than kept in a second buffer: this arm runs only where
/// the block would otherwise have no Newton direction at all, and the G3b
/// buffer discipline exists for the hot path, not for the rescue.
///
/// A `Singular` verdict from `gauss_solve_into` guarantees every Jacobian and
/// right-hand-side entry was finite (it checks those first, and reports
/// `NonFinite` instead), so the decomposition is fed the same finite matrix
/// Commons Math would see.
fn svd_fallback(
    jacobian: &[Vec<f64>],
    rhs: &[f64],
    d: &[f64],
    y: &mut [f64],
) -> std::result::Result<(), LinearFailure> {
    let n = rhs.len();
    let equilibrated: crate::linalg::Mat = (0..n)
        .map(|i| (0..n).map(|j| jacobian[i][j] * d[j]).collect())
        .collect();
    let solver =
        crate::linalg::SvdSolver::new(&equilibrated).map_err(|_| LinearFailure::Singular)?;
    let solved = solver.solve(rhs);
    y.copy_from_slice(&solved);
    Ok(())
}

/// Gaussian elimination with partial pivoting. Consumes the matrix and the
/// right-hand side because both are overwritten in place. Kept with this
/// signature for the unit tests below; the solver's hot paths call
/// [`gauss_solve_into`] with reused buffers (G3b) — same arithmetic, no
/// allocation.
#[cfg(test)]
fn gauss_solve(
    mut a: Vec<Vec<f64>>,
    mut b: Vec<f64>,
) -> std::result::Result<Vec<f64>, LinearFailure> {
    let mut y = vec![0.0f64; b.len()];
    gauss_solve_into(&mut a, &mut b, &mut y)?;
    Ok(y)
}

/// See [`gauss_solve`]: the elimination itself, writing the solution into `y`.
fn gauss_solve_into(
    a: &mut [Vec<f64>],
    b: &mut [f64],
    y: &mut [f64],
) -> std::result::Result<(), LinearFailure> {
    let n = b.len();
    if n == 0 {
        return Ok(());
    }

    let mut largest = 0.0f64;
    for row in a.iter() {
        for &v in row {
            if !v.is_finite() {
                return Err(LinearFailure::NonFinite);
            }
            largest = largest.max(v.abs());
        }
    }
    for &v in b.iter() {
        if !v.is_finite() {
            return Err(LinearFailure::NonFinite);
        }
    }
    let threshold = SINGULARITY_RATIO * largest;

    for col in 0..n {
        let mut pivot = col;
        let mut best = a[col][col].abs();
        for (r, row) in a.iter().enumerate().skip(col + 1) {
            let candidate = row[col].abs();
            if candidate > best {
                best = candidate;
                pivot = r;
            }
        }
        // `best` is finite (every entry was checked above), so this also rejects
        // the all-zero column, where `threshold` itself is 0.
        if best <= threshold {
            return Err(LinearFailure::Singular);
        }
        if pivot != col {
            a.swap(pivot, col);
            b.swap(pivot, col);
        }

        let (upper, lower) = a.split_at_mut(col + 1);
        let pivot_row = &upper[col];
        let diagonal = pivot_row[col];
        for (r, row) in lower.iter_mut().enumerate() {
            let factor = row[col] / diagonal;
            if factor == 0.0 {
                continue;
            }
            row[col] = 0.0;
            for (c, &above) in pivot_row.iter().enumerate().skip(col + 1) {
                row[c] -= factor * above;
            }
            b[col + 1 + r] -= factor * b[col];
        }
    }

    for i in (0..n).rev() {
        let mut sum = b[i];
        for j in i + 1..n {
            sum -= a[i][j] * y[j];
        }
        let v = sum / a[i][i];
        if !v.is_finite() {
            return Err(LinearFailure::NonFinite);
        }
        y[i] = v;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Globalization: step halving and the damped rescue
// ---------------------------------------------------------------------------

/// Backtracking line search: try `x - λ·Δ` for `λ = 1, ½, ¼, …`, stopping at the
/// first candidate whose residual norm beats `norm`. Every candidate is
/// clamped into the `[lo, hi]` box first — the Java `backtrackLineSearch`
/// `Math.clamp` site — so a bounded unknown is never evaluated out of range.
/// Returns the last candidate tried even when none descended — the caller
/// decides what to do with a non-descending result, exactly as in the Java.
#[allow(clippy::too_many_arguments)]
fn backtrack_line_search<P>(
    problem: &mut P,
    x: &[f64],
    step: &[f64],
    norm: f64,
    settings: &SolverSettings,
    lo: &[f64],
    hi: &[f64],
    candidate: &mut [f64],
    candidate_residual: &mut [f64],
) -> Result<Accepted>
where
    P: NewtonProblem + ?Sized,
{
    let n = x.len();
    let mut lambda = 1.0f64;
    candidate_residual.fill(f64::NAN);
    let mut candidate_norm = f64::INFINITY;

    for _ in 0..=settings.max_step_halvings {
        let mut finite_point = true;
        for i in 0..n {
            candidate[i] = (x[i] - lambda * step[i]).clamp(lo[i], hi[i]);
            finite_point &= candidate[i].is_finite();
        }
        if finite_point {
            eval_residual(problem, candidate, candidate_residual)?;
            candidate_norm = l2_norm(candidate_residual);
            if candidate_norm.is_finite() && candidate_norm < norm {
                break;
            }
        } else {
            // Never hand the residual callback a non-finite point; an overflowed
            // candidate cannot be a solution anyway.
            candidate_residual.iter_mut().for_each(|v| *v = f64::NAN);
            candidate_norm = f64::INFINITY;
        }
        lambda /= 2.0;
    }

    Ok(Accepted {
        norm: candidate_norm,
    })
}

/// Damped-step rescue (Levenberg–Marquardt): when the pure Newton direction
/// cannot descend — a near-singular or badly scaled Jacobian, or a step that
/// overshoots past a function's valid region — solve the damped normal
/// equations `(JᵀJ + λ·diag(JᵀJ))·δ = Jᵀf` instead, escalating `λ` until a
/// norm-reducing candidate appears. Diagonal scaling makes the damping
/// invariant to variable units, the same job column equilibration does for the
/// undamped path. Candidates are clamped into the `[lo, hi]` box — the Java
/// `dampedRescue` `Math.clamp` site — and a candidate pinned so hard it cannot
/// move ends the rescue (`moved`), because shorter steps cannot unpin it.
/// A candidate is accepted when its norm drops below
/// `accept_factor · norm`. Returns `None` when no damping achieves that; the
/// caller then reports the stall.
#[allow(clippy::too_many_arguments)]
fn damped_rescue<P>(
    problem: &mut P,
    jacobian: &[Vec<f64>],
    x: &[f64],
    residual: &[f64],
    norm: f64,
    previous_lambda: f64,
    accept_factor: f64,
    lo: &[f64],
    hi: &[f64],
    bufs: RescueBufs<'_>,
) -> Result<Option<DampedStep>>
where
    P: NewtonProblem + ?Sized,
{
    let n = x.len();
    let RescueBufs {
        jtj,
        jtr,
        damped_a,
        damped_b,
        damped_y,
        candidate,
        candidate_residual,
    } = bufs;
    for row in jtj.iter_mut() {
        row.fill(0.0);
    }
    jtr.fill(0.0);
    let mut any_row = false;
    for i in 0..n {
        // Rows stuck in an invalid region (non-finite residual or derivative)
        // carry no direction — leave them out of the normal equations.
        if !residual[i].is_finite() || !jacobian[i].iter().all(|v| v.is_finite()) {
            continue;
        }
        any_row = true;
        let row = &jacobian[i];
        for j in 0..n {
            jtr[j] += row[j] * residual[i];
            for k in 0..n {
                jtj[j][k] += row[j] * row[k];
            }
        }
    }
    let mut max_diagonal = 0.0f64;
    for (j, row) in jtj.iter().enumerate() {
        max_diagonal = max_diagonal.max(row[j]);
    }
    if !any_row || max_diagonal <= 0.0 || !max_diagonal.is_finite() {
        return Ok(None); // no usable curvature information at this point
    }
    if jtr.iter().any(|v| !v.is_finite()) {
        return Ok(None);
    }
    let diagonal_floor = 1.0e-12 * max_diagonal; // keeps zero columns damped too

    candidate_residual.fill(f64::NAN);
    let mut lam = LM_LAMBDA_MIN.max(previous_lambda);
    while lam <= LM_LAMBDA_MAX {
        for (slot, row) in damped_a.iter_mut().zip(jtj.iter()) {
            slot.copy_from_slice(row);
        }
        for (j, row) in damped_a.iter_mut().enumerate() {
            row[j] += lam * jtj[j][j].max(diagonal_floor);
        }
        damped_b.copy_from_slice(jtr);
        let delta: &[f64] = match gauss_solve_into(damped_a, damped_b, damped_y) {
            Ok(()) => damped_y,
            Err(_) => {
                lam *= 10.0; // more damping regularizes further
                continue;
            }
        };

        let mut moved = false;
        let mut finite_point = true;
        for i in 0..n {
            // Java dampedRescue: Math.clamp(x[i] - delta[i], lo[i], hi[i]).
            candidate[i] = (x[i] - delta[i]).clamp(lo[i], hi[i]);
            moved |= candidate[i] != x[i];
            finite_point &= candidate[i].is_finite();
        }
        if !moved {
            return Ok(None); // pinned at bounds: shorter steps cannot unpin
        }
        if !finite_point {
            lam *= 10.0;
            continue;
        }

        eval_residual(problem, candidate, candidate_residual)?;
        let candidate_norm = l2_norm(candidate_residual);
        // A finite norm beats a non-finite one: a damped step that walks the
        // iterate back onto valid ground is progress.
        if candidate_norm.is_finite()
            && (!norm.is_finite() || candidate_norm < norm * accept_factor)
        {
            return Ok(Some(DampedStep {
                norm: candidate_norm,
                lambda: lam,
            }));
        }
        lam *= 10.0;
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diag::FreesError;

    fn settings() -> SolverSettings {
        SolverSettings::default()
    }

    /// Unbounded lo/hi arrays for direct calls into the internals.
    fn unbounded(n: usize) -> (Vec<f64>, Vec<f64>) {
        (vec![f64::NEG_INFINITY; n], vec![f64::INFINITY; n])
    }

    fn solve<F>(f: F, start: &[f64]) -> (Vec<f64>, Result<NewtonReport>)
    where
        F: FnMut(&[f64], &mut [f64]) -> Result<()>,
    {
        solve_with(f, start, &settings())
    }

    fn solve_with<F>(f: F, start: &[f64], s: &SolverSettings) -> (Vec<f64>, Result<NewtonReport>)
    where
        F: FnMut(&[f64], &mut [f64]) -> Result<()>,
    {
        let mut x = start.to_vec();
        let report = newton_solve(f, &mut x, s, None);
        (x, report)
    }

    // ---------------- scalar systems ----------------

    #[test]
    fn scalar_linear_converges_in_one_iteration() {
        let (x, report) = solve(
            |x, out| {
                out[0] = 2.0 * x[0] - 4.0;
                Ok(())
            },
            &[0.0],
        );
        let report = report.expect("linear scalar must solve");
        // One Newton step lands on the root; the reachable accuracy is set by
        // the finite-difference Jacobian, which with `jacobian_epsilon = 1e-7`
        // is good to ~1e-9 relative — so at the Java stop criterion (1e-12) one
        // polishing step follows before the top-of-loop test returns.
        assert!(
            report.iterations <= 2,
            "a linear system must not iterate: {}",
            report.iterations
        );
        assert!((x[0] - 2.0).abs() < 1e-12, "x = {}", x[0]);
        assert!(report.converged);
        assert!(report.residual_norm < 1e-11, "{}", report.residual_norm);
    }

    #[test]
    fn scalar_quadratic_finds_the_nearby_root() {
        let f = |x: &[f64], out: &mut [f64]| {
            out[0] = x[0] * x[0] - 4.0;
            Ok(())
        };
        let (x, report) = solve(f, &[1.0]);
        assert!(report.unwrap().converged);
        assert!((x[0] - 2.0).abs() < 1e-9, "x = {}", x[0]);

        let (x, report) = solve(f, &[-3.0]);
        assert!(report.unwrap().converged);
        assert!((x[0] + 2.0).abs() < 1e-9, "x = {}", x[0]);
    }

    #[test]
    fn scalar_cubic_matches_the_classical_root() {
        // x^3 - 2x - 5 = 0, Wallis' cubic; root 2.0945514815423265.
        let (x, report) = solve(
            |x, out| {
                out[0] = x[0] * x[0] * x[0] - 2.0 * x[0] - 5.0;
                Ok(())
            },
            &[2.0],
        );
        let report = report.expect("cubic must solve");
        assert!(report.converged);
        assert!(
            (x[0] - 2.094_551_481_542_326_5).abs() < 1e-9,
            "x = {:.17}",
            x[0]
        );
        assert!(report.iterations >= 1 && report.iterations < 10);
    }

    #[test]
    fn scalar_with_a_far_guess_still_converges() {
        // x^2 = 4 from a million: Newton roughly halves x each step, so this
        // needs many iterations but never a halving.
        let (x, report) = solve(
            |x, out| {
                out[0] = x[0] * x[0] - 4.0;
                Ok(())
            },
            &[1.0e6],
        );
        assert!(report.unwrap().converged);
        assert!((x[0] - 2.0).abs() < 1e-8, "x = {}", x[0]);
    }

    // ---------------- 2x2 and 3x3 systems ----------------

    #[test]
    fn two_by_two_linear_converges_in_one_iteration() {
        let (x, report) = solve(
            |x, out| {
                out[0] = 2.0 * x[0] + x[1] - 5.0;
                out[1] = x[0] + 3.0 * x[1] - 10.0;
                Ok(())
            },
            &[0.0, 0.0],
        );
        let report = report.expect("2x2 linear must solve");
        // A linear system is solved by the very first step; the extra iteration
        // is the finite-difference Jacobian's ~1e-9 error being polished out.
        assert!(
            report.iterations <= 2,
            "a linear system must not iterate: {}",
            report.iterations
        );
        assert!((x[0] - 1.0).abs() < 1e-8, "x = {x:?}");
        assert!((x[1] - 3.0).abs() < 1e-8, "x = {x:?}");
    }

    #[test]
    fn two_by_two_circle_line_intersection() {
        // x^2 + y^2 = 4 and y = x  =>  (sqrt 2, sqrt 2).
        let (x, report) = solve(
            |x, out| {
                out[0] = x[0] * x[0] + x[1] * x[1] - 4.0;
                out[1] = x[1] - x[0];
                Ok(())
            },
            &[1.5, 0.5],
        );
        let report = report.expect("circle/line must solve");
        assert!(report.converged);
        let root = std::f64::consts::SQRT_2;
        assert!((x[0] - root).abs() < 1e-8, "x = {x:?}");
        assert!((x[1] - root).abs() < 1e-8, "x = {x:?}");
    }

    #[test]
    fn three_by_three_nonlinear_system() {
        // x+y+z = 6, xy+z = 5, xyz = 6  =>  (1, 2, 3).
        let (x, report) = solve(
            |x, out| {
                out[0] = x[0] + x[1] + x[2] - 6.0;
                out[1] = x[0] * x[1] + x[2] - 5.0;
                out[2] = x[0] * x[1] * x[2] - 6.0;
                Ok(())
            },
            &[0.7, 1.7, 3.4],
        );
        let report = report.expect("3x3 must solve");
        assert!(report.converged);
        assert!((x[0] - 1.0).abs() < 1e-6, "x = {x:?}");
        assert!((x[1] - 2.0).abs() < 1e-6, "x = {x:?}");
        assert!((x[2] - 3.0).abs() < 1e-6, "x = {x:?}");
    }

    #[test]
    fn badly_scaled_system_survives_column_equilibration() {
        // Columns differing by 14 orders of magnitude: without equilibration the
        // factorization loses the small unknown entirely.
        let (x, report) = solve(
            |x, out| {
                out[0] = 1.0e8 * x[0] - 2.0e8;
                out[1] = 1.0e-6 * x[1] - 3.0e-6;
                Ok(())
            },
            &[0.0, 0.0],
        );
        let report = report.expect("badly scaled system must solve");
        assert!(report.converged);
        assert!((x[0] - 2.0).abs() < 1e-6, "x = {x:?}");
        assert!((x[1] - 3.0).abs() < 1e-6, "x = {x:?}");
    }

    // ---------------- step halving ----------------

    /// A full Newton step on `atan` overshoots for any |x| > 1.3917; the
    /// undamped iteration then diverges geometrically.
    fn naive_full_step_newton_on_atan(mut x: f64, steps: usize) -> f64 {
        for _ in 0..steps {
            let r = libm::atan(x);
            let derivative = 1.0 / (1.0 + x * x);
            x -= r / derivative;
        }
        x
    }

    #[test]
    fn step_halving_rescues_an_overshooting_newton_step() {
        // Naive full-step Newton blows up from x0 = 2 ...
        let naive = naive_full_step_newton_on_atan(2.0, 4);
        assert!(
            naive.abs() > 1.0e4,
            "full-step Newton should diverge, got {naive}"
        );

        // ... while the halving line search walks it back to the root.
        let (x, report) = solve(
            |x, out| {
                out[0] = libm::atan(x[0]);
                Ok(())
            },
            &[2.0],
        );
        let report = report.expect("halving must rescue atan");
        assert!(report.converged);
        assert!(x[0].abs() < 1e-8, "x = {}", x[0]);
        assert!(report.residual_norm <= 1e-9);
    }

    #[test]
    fn the_solve_itself_rejects_the_full_step_and_takes_the_half_step() {
        // Same atan problem, but watch every point the solver evaluates.
        let mut visited: Vec<f64> = Vec::new();
        let mut x = [2.0];
        let report = newton_solve(
            |x, out| {
                visited.push(x[0]);
                out[0] = libm::atan(x[0]);
                Ok(())
            },
            &mut x,
            &settings(),
            None,
        )
        .expect("must solve");
        assert!(report.converged);

        let newton_step = libm::atan(2.0) * 5.0; // r / f'(2)
        let full = 2.0 - newton_step;
        let half = 2.0 - 0.5 * newton_step;
        assert!(
            visited.iter().any(|v| (v - full).abs() < 1e-6),
            "the full step must have been tried: {visited:?}"
        );
        assert!(
            visited.iter().any(|v| (v - half).abs() < 1e-6),
            "the halved step must have been tried: {visited:?}"
        );
        // The full step was evaluated but never became an iterate: nothing
        // downstream of it is ever visited again.
        assert!(
            visited.iter().all(|v| v.abs() < 4.0),
            "the diverging branch must never be entered: {visited:?}"
        );
    }

    #[test]
    fn line_search_picks_the_first_descending_halving() {
        let mut f = |x: &[f64], out: &mut [f64]| {
            out[0] = libm::atan(x[0]);
            Ok(())
        };
        let x = [2.0];
        let base = libm::atan(2.0);
        // Newton direction: r / f'(x) with f'(x) = 1/(1+x^2).
        let step = [base * 5.0];
        let full = libm::atan(x[0] - step[0]).abs();
        assert!(full > base, "the full step must not descend");

        let (lo, hi) = unbounded(1);
        let mut candidate = [0.0f64];
        let mut residual = [0.0f64];
        let result = backtrack_line_search(
            &mut f,
            &x,
            &step,
            base.abs(),
            &settings(),
            &lo,
            &hi,
            &mut candidate,
            &mut residual,
        )
        .unwrap();
        assert!(result.norm < base.abs());
        // lambda = 1/2 is the first descending step.
        assert!((candidate[0] - (2.0 - 0.5 * step[0])).abs() < 1e-12);
    }

    /// `NewtonSolver.backtrackLineSearch` runs `for (halving = 0; halving <
    /// MAX_HALVINGS; halving++)` with `MAX_HALVINGS = 25`, i.e. 25 values of λ
    /// ending at 2⁻²⁴. Ours counts *halvings*, so `max_step_halvings = 24` is
    /// the same 25 trials. Pin the arithmetic — an off-by-one here quietly
    /// shortens the globalization on exactly the hard problems it exists for.
    #[test]
    fn the_line_search_tries_one_more_lambda_than_it_allows_halvings() {
        for allowed in [0usize, 1, 3, 24] {
            let mut s = settings();
            s.max_step_halvings = allowed;
            let mut lambdas: Vec<f64> = Vec::new();
            let mut f = |x: &[f64], out: &mut [f64]| {
                // x = 10 - λ*10, so λ is recoverable from the point probed.
                lambdas.push((10.0 - x[0]) / 10.0);
                out[0] = 1.0; // never descends: the search exhausts its budget
                Ok(())
            };
            let (lo, hi) = unbounded(1);
            let mut candidate = [0.0f64];
            let mut residual = [0.0f64];
            let result = backtrack_line_search(
                &mut f,
                &[10.0],
                &[10.0],
                0.5,
                &s,
                &lo,
                &hi,
                &mut candidate,
                &mut residual,
            )
            .expect("no callback error");
            assert_eq!(
                lambdas.len(),
                allowed + 1,
                "{allowed} halvings must mean {} trials, got {lambdas:?}",
                allowed + 1
            );
            assert_eq!(lambdas[0], 1.0, "the full step is always tried first");
            let smallest = 0.5f64.powi(allowed as i32);
            assert!(
                (lambdas[allowed] - smallest).abs() < 1e-15,
                "last lambda should be 2^-{allowed}: {lambdas:?}"
            );
            // A non-descending search still hands back its last candidate.
            assert!(result.norm.is_finite());
        }
    }

    #[test]
    fn damped_rescue_covers_a_solver_with_no_halvings_allowed() {
        // With halving disabled the overshooting step is rejected outright and
        // the Levenberg-Marquardt rescue must carry the iteration.
        let mut s = settings();
        s.max_step_halvings = 0;
        let (x, report) = solve_with(
            |x, out| {
                out[0] = libm::atan(x[0]);
                Ok(())
            },
            &[2.0],
            &s,
        );
        let report = report.expect("damped rescue must carry the solve");
        assert!(report.converged);
        assert!(x[0].abs() < 1e-8, "x = {}", x[0]);
    }

    #[test]
    fn halving_steps_over_an_invalid_region() {
        // f is undefined (NaN) for x < 0; the full Newton step from 0.5 lands at
        // 4.25 and increases the residual, so halving is what makes progress.
        let (x, report) = solve(
            |x, out| {
                out[0] = if x[0] < 0.0 {
                    f64::NAN
                } else {
                    x[0] * x[0] - 4.0
                };
                Ok(())
            },
            &[0.5],
        );
        assert!(report.expect("must solve").converged);
        assert!((x[0] - 2.0).abs() < 1e-8, "x = {}", x[0]);
    }

    // ---------------- degenerate systems ----------------

    #[test]
    fn zero_jacobian_is_a_clean_error() {
        let mut s = settings();
        s.max_iterations = 5;
        let (_, report) = solve_with(
            |_x, out| {
                out[0] = 1.0;
                out[1] = 1.0;
                Ok(())
            },
            &[0.0, 0.0],
            &s,
        );
        match report {
            Err(FreesError::Solver { message }) => {
                assert!(message.contains("stalled"), "message = {message}");
            }
            other => panic!("expected a solver error, got {other:?}"),
        }
    }

    #[test]
    fn inconsistent_singular_system_is_a_clean_error() {
        // x + y = 2 and x + y = 5: rank-1 Jacobian, no solution.
        let mut s = settings();
        s.max_iterations = 20;
        let (_, report) = solve_with(
            |x, out| {
                out[0] = x[0] + x[1] - 2.0;
                out[1] = x[0] + x[1] - 5.0;
                Ok(())
            },
            &[0.0, 0.0],
            &s,
        );
        assert!(
            matches!(report, Err(FreesError::Solver { .. })),
            "expected a solver error, got {report:?}"
        );
    }

    #[test]
    fn consistent_rank_deficient_system_is_regularized() {
        // x + y = 2 and 2x + 2y = 4: singular but solvable. The elimination
        // fails on the pivot; the SVD pseudo-inverse (Java `solveLinear`'s
        // `catch (SingularMatrixException)`) returns the minimum-norm
        // least-squares direction, which lands on the solution line.
        let (x, report) = solve(
            |x, out| {
                out[0] = x[0] + x[1] - 2.0;
                out[1] = 2.0 * x[0] + 2.0 * x[1] - 4.0;
                Ok(())
            },
            &[0.0, 0.0],
        );
        let report = report.expect("rank-deficient but consistent system must solve");
        assert!(report.converged);
        assert!((x[0] + x[1] - 2.0).abs() < 1e-8, "x = {x:?}");
    }

    #[test]
    fn non_finite_residuals_do_not_panic() {
        let mut s = settings();
        s.max_iterations = 5;
        let (_, report) = solve_with(
            |_x, out| {
                out[0] = f64::NAN;
                Ok(())
            },
            &[1.0],
            &s,
        );
        assert!(
            matches!(report, Err(FreesError::Solver { .. })),
            "expected a solver error, got {report:?}"
        );
    }

    #[test]
    fn infinite_residuals_do_not_panic() {
        let mut s = settings();
        s.max_iterations = 5;
        let (_, report) = solve_with(
            |x, out| {
                out[0] = if x[0] > 0.0 { f64::INFINITY } else { x[0] };
                Ok(())
            },
            &[1.0],
            &s,
        );
        assert!(
            matches!(report, Err(FreesError::Solver { .. })),
            "expected a solver error, got {report:?}"
        );
    }

    #[test]
    fn iteration_limit_is_reported() {
        let mut s = settings();
        s.max_iterations = 3;
        let (_, report) = solve_with(
            |x, out| {
                out[0] = x[0] * x[0] - 4.0;
                Ok(())
            },
            &[1.0e6],
            &s,
        );
        match report {
            Err(FreesError::Solver { message }) => {
                assert!(message.contains("did not converge"), "message = {message}");
                assert!(message.contains('3'), "message = {message}");
            }
            other => panic!("expected a solver error, got {other:?}"),
        }
    }

    // ---------------- callback errors ----------------

    #[test]
    fn residual_error_on_the_first_call_propagates() {
        let (_, report) = solve(
            |_x, _out| Err(FreesError::property("no state point at 300 K, 1 kPa")),
            &[1.0],
        );
        match report {
            Err(FreesError::Property { message }) => {
                assert_eq!(message, "no state point at 300 K, 1 kPa");
            }
            other => panic!("expected the callback error, got {other:?}"),
        }
    }

    #[test]
    fn residual_error_inside_the_jacobian_propagates() {
        let mut calls = 0usize;
        let (_, report) = solve(
            |x, out| {
                calls += 1;
                if calls == 2 {
                    // The first Jacobian probe fails.
                    return Err(FreesError::evaluation("ln of a negative number"));
                }
                out[0] = x[0] * x[0] - 4.0;
                Ok(())
            },
            &[1.0],
        );
        match report {
            Err(FreesError::Evaluation { message }) => {
                assert_eq!(message, "ln of a negative number");
            }
            other => panic!("expected the callback error, got {other:?}"),
        }
        assert_eq!(calls, 2, "the solve must abort at the failing call");
    }

    #[test]
    fn residual_error_inside_the_line_search_propagates() {
        let mut calls = 0usize;
        let (_, report) = solve(
            |x, out| {
                calls += 1;
                // call 1: base residual, call 2: Jacobian probe, call 3: line search.
                if calls == 3 {
                    return Err(FreesError::property("invalid state"));
                }
                out[0] = x[0] * x[0] - 4.0;
                Ok(())
            },
            &[1.0],
        );
        assert!(
            matches!(report, Err(FreesError::Property { .. })),
            "expected the callback error, got {report:?}"
        );
        assert_eq!(calls, 3);
    }

    // ---------------- report accuracy ----------------

    #[test]
    fn starting_at_the_root_costs_zero_iterations_and_one_evaluation() {
        let mut calls = 0usize;
        let mut x = [2.0];
        let report = newton_solve(
            |x, out| {
                calls += 1;
                out[0] = x[0] * x[0] - 4.0;
                Ok(())
            },
            &mut x,
            &settings(),
            None,
        )
        .expect("already solved");
        assert_eq!(report.iterations, 0);
        assert_eq!(report.residual_norm, 0.0);
        assert!(report.converged);
        assert_eq!(x[0], 2.0, "x must not be disturbed");
        assert_eq!(calls, 1, "only the initial residual is needed");
    }

    #[test]
    fn report_residual_norm_is_the_max_absolute_residual() {
        let mut x = [0.0, 0.0];
        let report = newton_solve(
            |x, out| {
                out[0] = 2.0 * x[0] + x[1] - 5.0;
                out[1] = x[0] + 3.0 * x[1] - 10.0;
                Ok(())
            },
            &mut x,
            &settings(),
            None,
        )
        .expect("must solve");

        // Recompute the residuals at the returned point and compare.
        let r0: f64 = 2.0 * x[0] + x[1] - 5.0;
        let r1: f64 = x[0] + 3.0 * x[1] - 10.0;
        let expected = r0.abs().max(r1.abs());
        assert!(
            (report.residual_norm - expected).abs() <= f64::EPSILON.max(expected * 1e-12),
            "report {} vs recomputed {}",
            report.residual_norm,
            expected
        );
    }

    #[test]
    fn iteration_count_grows_with_a_harder_start() {
        let f = |x: &[f64], out: &mut [f64]| {
            out[0] = x[0] * x[0] - 4.0;
            Ok(())
        };
        let near = solve(f, &[2.1]).1.unwrap().iterations;
        let far = solve(f, &[1.0e3]).1.unwrap().iterations;
        assert!(near < far, "near {near} should beat far {far}");
        assert!(near >= 1);
    }

    #[test]
    fn failed_solve_leaves_the_last_iterate_in_x() {
        let mut s = settings();
        s.max_iterations = 2;
        let mut x = [1.0e6];
        let report = newton_solve(
            |x, out| {
                out[0] = x[0] * x[0] - 4.0;
                Ok(())
            },
            &mut x,
            &s,
            None,
        );
        assert!(report.is_err());
        assert!(x[0] < 1.0e6, "the iterate must have advanced: {}", x[0]);
        assert!(x[0] > 2.0);
    }

    // ---------------- settings ----------------

    #[test]
    fn settings_are_validated() {
        let mut s = settings();
        s.max_iterations = 0;
        assert!(newton_solve(|_x, _o| Ok(()), &mut [1.0], &s, None).is_err());

        let mut s = settings();
        s.rel_tolerance = 0.0;
        assert!(newton_solve(|_x, _o| Ok(()), &mut [1.0], &s, None).is_err());

        let mut s = settings();
        s.abs_tolerance = -1.0;
        assert!(newton_solve(|_x, _o| Ok(()), &mut [1.0], &s, None).is_err());

        let mut s = settings();
        s.jacobian_epsilon = 0.0;
        assert!(newton_solve(|_x, _o| Ok(()), &mut [1.0], &s, None).is_err());

        let mut s = settings();
        s.jacobian_epsilon = f64::NAN;
        assert!(newton_solve(|_x, _o| Ok(()), &mut [1.0], &s, None).is_err());
    }

    /// The Java `SolverSettings.DEFAULTS` is `(250, 1e-12, 1e-15, 3600.0)`, and
    /// `NewtonSolver.MAX_HALVINGS` is 25 line-search trials. Anything looser
    /// here is a silent accuracy divergence from the oracle, so pin the numbers.
    #[test]
    fn default_settings_match_the_java_defaults() {
        let s = SolverSettings::default();
        assert_eq!(
            s.max_iterations, 250,
            "Java SolverSettings.DEFAULTS.maxIterations"
        );
        assert_eq!(
            s.rel_tolerance, 1e-12,
            "Java SolverSettings.DEFAULTS.relativeResiduals"
        );
        // 24 halvings == 25 trials of λ == Java's MAX_HALVINGS.
        assert_eq!(
            s.max_step_halvings, 24,
            "Java NewtonSolver.MAX_HALVINGS - 1"
        );
        // Java has no absolute residual criterion; ours must stay inert.
        assert!(
            s.abs_tolerance > 0.0 && s.abs_tolerance < s.rel_tolerance,
            "abs_tolerance {} must never override rel_tolerance {}",
            s.abs_tolerance,
            s.rel_tolerance
        );
        assert!(s.jacobian_epsilon > 0.0);
        // Java SolverSettings.DEFAULTS carries complexMode = false.
        assert!(!s.complex_mode, "complex mode is off by default");
    }

    /// Regression for the divergence the default used to carry: with a `1e-9`
    /// stop criterion the canonical two-by-two lands ~1e-11 away from the Java
    /// oracle, which is the same order as the parity harness's own comparison
    /// tolerance. At the Java criterion it lands *on* the oracle.
    #[test]
    fn the_default_stop_criterion_reproduces_the_java_oracle_exactly() {
        // fixtures/golden/canonical.json: x^2 + y^3 = 77, x/y = 1.23456.
        let residual = |v: &[f64], out: &mut [f64]| {
            out[0] = v[0] * v[0] + v[1] * v[1] * v[1] - 77.0;
            out[1] = v[0] / v[1] - 1.23456;
            Ok(())
        };
        let (oracle_x, oracle_y) = (4.694012391660914, 3.802174371161316);

        let mut x = [1.0, 1.0];
        newton_solve(residual, &mut x, &SolverSettings::default(), None).expect("must solve");
        assert!(
            (x[0] - oracle_x).abs() <= 1e-14 && (x[1] - oracle_y).abs() <= 1e-14,
            "default settings must reach the oracle: {x:?}"
        );

        // The old default demonstrably could not.
        let loose = SolverSettings {
            max_iterations: 100,
            rel_tolerance: 1e-9,
            abs_tolerance: 1e-10,
            max_step_halvings: 20,
            jacobian_epsilon: 1e-7,
            complex_mode: false,
        };
        let mut y = [1.0, 1.0];
        newton_solve(residual, &mut y, &loose, None).expect("must solve");
        assert!(
            (y[0] - oracle_x).abs() > 1e-13,
            "a 1e-9 stop criterion should stop short of the oracle: {y:?}"
        );
    }

    #[test]
    fn empty_system_is_trivially_converged() {
        let mut x: [f64; 0] = [];
        let report = newton_solve(|_x, _out| Ok(()), &mut x, &settings(), None).unwrap();
        assert_eq!(report.iterations, 0);
        assert!(report.converged);
        assert_eq!(report.residual_norm, 0.0);
    }

    // ---------------- unit tests of the internals ----------------

    #[test]
    fn gauss_solve_uses_partial_pivoting() {
        // A zero leading pivot: only row swapping can solve this.
        let a = vec![vec![0.0, 1.0], vec![1.0, 0.0]];
        let b = vec![1.0, 2.0];
        let y = gauss_solve(a, b).expect("pivoting must handle a zero leading entry");
        assert!((y[0] - 2.0).abs() < 1e-12);
        assert!((y[1] - 1.0).abs() < 1e-12);
    }

    #[test]
    fn gauss_solve_reports_singular_and_non_finite() {
        let singular = vec![vec![1.0, 2.0], vec![2.0, 4.0]];
        assert_eq!(
            gauss_solve(singular, vec![1.0, 2.0]),
            Err(LinearFailure::Singular)
        );

        let zero = vec![vec![0.0, 0.0], vec![0.0, 0.0]];
        assert_eq!(
            gauss_solve(zero, vec![1.0, 1.0]),
            Err(LinearFailure::Singular)
        );

        let nan = vec![vec![f64::NAN, 0.0], vec![0.0, 1.0]];
        assert_eq!(
            gauss_solve(nan, vec![1.0, 1.0]),
            Err(LinearFailure::NonFinite)
        );

        let identity = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
        assert_eq!(
            gauss_solve(identity, vec![f64::INFINITY, 1.0]),
            Err(LinearFailure::NonFinite)
        );
    }

    #[test]
    fn gauss_solve_handles_a_three_by_three_system() {
        let a = vec![
            vec![2.0, 1.0, -1.0],
            vec![-3.0, -1.0, 2.0],
            vec![-2.0, 1.0, 2.0],
        ];
        let b = vec![8.0, -11.0, -3.0];
        let y = gauss_solve(a, b).unwrap();
        assert!((y[0] - 2.0).abs() < 1e-12, "{y:?}");
        assert!((y[1] - 3.0).abs() < 1e-12, "{y:?}");
        assert!((y[2] + 1.0).abs() < 1e-12, "{y:?}");
    }

    #[test]
    fn solve_linear_returns_the_newton_direction() {
        let jacobian = vec![vec![2.0, 1.0], vec![1.0, 3.0]];
        let rhs = vec![5.0, 10.0];
        let step = solve_linear(&jacobian, &rhs).unwrap();
        // J * step must reproduce rhs.
        let p0 = 2.0 * step[0] + step[1];
        let p1 = step[0] + 3.0 * step[1];
        assert!((p0 - 5.0).abs() < 1e-9, "{step:?}");
        assert!((p1 - 10.0).abs() < 1e-9, "{step:?}");
    }

    #[test]
    fn solve_linear_scales_columns_that_differ_by_many_decades() {
        // Unknowns twenty decades apart. Raw, the 1e-10 pivot is indistinguishable
        // from zero next to the 1e10 one and the factorization refuses it ...
        let jacobian = vec![vec![1.0e10, 0.0], vec![0.0, 1.0e-10]];
        let rhs = vec![1.0e10, 1.0e-10];
        assert_eq!(
            gauss_solve(jacobian.clone(), rhs.clone()),
            Err(LinearFailure::Singular),
            "the unequilibrated matrix must look singular"
        );

        // ... while column equilibration turns it into the identity.
        let step = solve_linear(&jacobian, &rhs).unwrap();
        assert!((step[0] - 1.0).abs() < 1e-12, "{step:?}");
        assert!((step[1] - 1.0).abs() < 1e-12, "{step:?}");
    }

    /// The exact equilibrated 2×2 the acceptance document
    /// `solver-equilibration-coupled-block-…` produces at its second iteration
    /// (`big ≈ 3990`, `small ≈ 1.996e-6`), captured out of `solve_linear_into`.
    const TWELVE_DECADE_JACOBIAN: [[f64; 2]; 2] = [[1.0, 1.0e12], [-7.914585300241862e-12, 1.0]];
    const TWELVE_DECADE_RHS: [f64; 2] = [4.128878936171532e-5, 1.9328344781864674e-6];

    #[test]
    fn equilibration_alone_does_not_save_a_twelve_decade_block() {
        // Ledger item 40's real shape. Column equilibration is applied — and the
        // equilibrated matrix is *still* singular to a partial-pivoting
        // factorization, because the trailing pivot lands at 8.91e-12 against
        // the 1e-11 threshold. Commons Math agrees exactly: on these same
        // numbers `new LUDecomposition(jMat).getSolver().isNonSingular()` is
        // `false` and `getDeterminant()` reads 0.0, which is why the Java's
        // rescue is the SVD catch and not the scaling.
        let jacobian: Vec<Vec<f64>> = TWELVE_DECADE_JACOBIAN.iter().map(|r| r.to_vec()).collect();
        let equilibrated = vec![
            vec![jacobian[0][0], jacobian[0][1] * 1.0e-12],
            vec![jacobian[1][0], jacobian[1][1] * 1.0e-12],
        ];
        assert_eq!(
            gauss_solve(equilibrated, TWELVE_DECADE_RHS.to_vec()),
            Err(LinearFailure::Singular),
            "the equilibrated matrix must still refuse to factor"
        );
    }

    #[test]
    fn solve_linear_falls_back_to_the_svd_pseudo_inverse_when_the_pivot_collapses() {
        // The step the Java produces on exactly these numbers, measured by
        // running Commons Math 3.6.1 on the reference repo's own classpath:
        //     LU threw SingularMatrixException -> Java falls back to SVD
        //       singular values = [1.414213562373095, 6.303563717266935E-12]
        //       SVD step = [-216817.0939025889, 2.1681709394387769E-7]
        let jacobian: Vec<Vec<f64>> = TWELVE_DECADE_JACOBIAN.iter().map(|r| r.to_vec()).collect();
        let step = solve_linear(&jacobian, &TWELVE_DECADE_RHS)
            .expect("the SVD fallback must produce a step");
        assert!(
            (step[0] - -216_817.093_902_588_9).abs() <= 1e-9 * 216_817.0,
            "step = {step:?}"
        );
        assert!(
            (step[1] - 2.168_170_939_438_776_9e-7).abs() <= 1e-9 * 2.17e-7,
            "step = {step:?}"
        );
    }

    #[test]
    fn twelve_orders_of_magnitude_coupled_block_solves() {
        // The reference's `SolverEquilibrationTest
        // .coupledBlockWithTwelveOrdersOfMagnitudeScaleDisparitySolves`, as a
        // bare residual pair: `big = 2e6 - 1e12*small`, `small = 1e-6*sqrt(big/1e6)`.
        // Its assertions are the Java's own (`1.0` on `big`, `1e-12` on `small`).
        let (x, report) = solve(
            |x, out| {
                out[0] = x[0] - (2.0e6 - 1.0e12 * x[1]);
                out[1] = x[1] - 1.0e-6 * (x[0] / 1.0e6).sqrt();
                Ok(())
            },
            &[1.0, 1.0],
        );
        let report = report.expect("the twelve-decade block must solve");
        assert!(report.converged);
        assert!((x[0] - 1.0e6).abs() <= 1.0, "x = {x:?}");
        assert!((x[1] - 1.0e-6).abs() <= 1.0e-12, "x = {x:?}");
    }

    #[test]
    fn solve_linear_is_exact_for_the_scalar_case() {
        let jacobian = vec![vec![4.0]];
        let step = solve_linear(&jacobian, &[2.0]).unwrap();
        assert_eq!(step[0], 0.5);
    }

    #[test]
    fn numerical_jacobian_matches_the_analytic_one() {
        let mut f = |x: &[f64], out: &mut [f64]| {
            out[0] = x[0] * x[1];
            out[1] = x[0] + 3.0 * x[1];
            Ok(())
        };
        let x = [2.0, 3.0];
        let mut base = vec![0.0; 2];
        f(&x, &mut base).unwrap();
        let (lo, hi) = unbounded(x.len());
        let jacobian = numerical_jacobian(&mut f, &x, &base, &settings(), &lo, &hi).unwrap();
        assert!((jacobian[0][0] - 3.0).abs() < 1e-6, "{jacobian:?}");
        assert!((jacobian[0][1] - 2.0).abs() < 1e-6, "{jacobian:?}");
        assert!((jacobian[1][0] - 1.0).abs() < 1e-6, "{jacobian:?}");
        assert!((jacobian[1][1] - 3.0).abs() < 1e-6, "{jacobian:?}");
    }

    #[test]
    fn jacobian_perturbation_floors_at_the_absolute_step() {
        // At x = 0 a purely relative rule would produce a zero perturbation; the
        // max(|x|, 1) floor keeps the column informative.
        let mut probed = f64::NAN;
        let mut f = |x: &[f64], out: &mut [f64]| {
            if x[0] != 0.0 {
                probed = x[0];
            }
            out[0] = 3.0 * x[0];
            Ok(())
        };
        let x = [0.0];
        let base = vec![0.0];
        let s = settings();
        let (lo, hi) = unbounded(x.len());
        let jacobian = numerical_jacobian(&mut f, &x, &base, &s, &lo, &hi).unwrap();
        assert!((jacobian[0][0] - 3.0).abs() < 1e-9, "{jacobian:?}");
        assert!(
            (probed - s.jacobian_epsilon).abs() < 1e-18,
            "probe was {probed}, expected the absolute floor {}",
            s.jacobian_epsilon
        );
    }

    #[test]
    fn jacobian_perturbation_is_relative_for_large_values() {
        let mut probed = f64::NAN;
        let mut f = |x: &[f64], out: &mut [f64]| {
            probed = x[0];
            out[0] = 3.0 * x[0];
            Ok(())
        };
        let x = [1.0e6];
        let base = vec![3.0e6];
        let s = settings();
        let (lo, hi) = unbounded(x.len());
        numerical_jacobian(&mut f, &x, &base, &s, &lo, &hi).unwrap();
        let expected = 1.0e6 + s.jacobian_epsilon * 1.0e6;
        assert!(
            (probed - expected).abs() < 1e-6,
            "probe was {probed}, expected {expected}"
        );
    }

    #[test]
    fn jacobian_probes_backward_when_the_forward_probe_is_invalid() {
        // Valid only for x <= 1; the forward probe from x = 1 is NaN.
        let mut f = |x: &[f64], out: &mut [f64]| {
            out[0] = if x[0] > 1.0 { f64::NAN } else { x[0] - 0.5 };
            Ok(())
        };
        let x = [1.0];
        let base = vec![0.5];
        let (lo, hi) = unbounded(x.len());
        let jacobian = numerical_jacobian(&mut f, &x, &base, &settings(), &lo, &hi).unwrap();
        assert!((jacobian[0][0] - 1.0).abs() < 1e-6, "{jacobian:?}");
    }

    #[test]
    fn jacobian_column_of_an_unused_variable_is_finite_zero() {
        let mut f = |x: &[f64], out: &mut [f64]| {
            out[0] = x[0] - 1.0;
            out[1] = x[0] - 1.0; // x[1] influences nothing
            Ok(())
        };
        let x = [4.0, 7.0];
        let mut base = vec![0.0; 2];
        f(&x, &mut base).unwrap();
        let (lo, hi) = unbounded(x.len());
        let jacobian = numerical_jacobian(&mut f, &x, &base, &settings(), &lo, &hi).unwrap();
        assert_eq!(jacobian[0][1], 0.0);
        assert_eq!(jacobian[1][1], 0.0);
        assert!(jacobian[0][0] > 0.9 && jacobian[0][0] < 1.1);
    }

    /// **Divergence from `NewtonSolver.computeJacobian`.**
    ///
    /// The Java solver builds the Jacobian *symbolically* first
    /// (`analyticalJacobian` → `Differentiator.differentiate`) and only falls
    /// back to finite differences when some residual cannot be differentiated.
    /// This port is finite-difference-only, and finite differences have a blind
    /// spot the symbolic path does not: when a residual carries a large additive
    /// constant, the increment of a *nonlinear* term can land below one ulp of
    /// that constant and vanish, leaving an exactly-zero derivative.
    ///
    /// Worse, the "clean, informative column" test in `jacobian_column` is a
    /// whole-column decision (`any_change && !cancellation`), so a second row
    /// that *does* move cleanly stops the widening the dead row needs. The
    /// result is a silently rank-deficient Jacobian.
    ///
    /// `x + y = 1`, `x² − y² = 2e10` is an ordinary, non-singular system
    /// (true Jacobian `[[1, 1], [2x, −2y]]`, determinant −2(x+y) = −2 at the
    /// default guess) that Java solves from the same start.
    #[test]
    fn finite_differences_lose_a_nonlinear_row_behind_a_large_constant() {
        let mut f = |v: &[f64], out: &mut [f64]| {
            out[0] = v[0] + v[1] - 1.0;
            out[1] = v[0] * v[0] - v[1] * v[1] - 2.0e10;
            Ok(())
        };
        let x = [1.0, 1.0];
        let mut base = vec![0.0; 2];
        f(&x, &mut base).unwrap();

        let (lo, hi) = unbounded(x.len());
        let jacobian = numerical_jacobian(&mut f, &x, &base, &settings(), &lo, &hi).unwrap();
        // Row 0 is recovered exactly ...
        assert!((jacobian[0][0] - 1.0).abs() < 1e-6, "{jacobian:?}");
        assert!((jacobian[0][1] - 1.0).abs() < 1e-6, "{jacobian:?}");
        // ... while row 1, whose true entries are (2, -2), comes back all zero:
        // (1+h)^2 - 1 = 2e-7 is below ulp(2e10) = 3.8e-6 and disappears.
        assert_eq!(
            (jacobian[1][0], jacobian[1][1]),
            (0.0, 0.0),
            "the nonlinear row should have been lost; if this now holds real \
             derivatives the symbolic-Jacobian gap has been closed: {jacobian:?}"
        );

        // And the consequence: the block is refused as structurally singular.
        let mut start = [1.0, 1.0];
        let outcome = newton_solve(
            |v: &[f64], out: &mut [f64]| {
                out[0] = v[0] + v[1] - 1.0;
                out[1] = v[0] * v[0] - v[1] * v[1] - 2.0e10;
                Ok(())
            },
            &mut start,
            &settings(),
            None,
        );
        match outcome {
            Err(FreesError::Solver { message }) => {
                assert!(message.contains("singular"), "message = {message}");
            }
            other => {
                panic!("expected the singular-Jacobian stall this test documents, got {other:?}")
            }
        }
    }

    /// The same blind spot, isolated: a lone nonlinear residual behind a large
    /// constant is rescued only because the widening has nothing else to satisfy.
    /// The derivative it settles on is 12 where the truth is 2 — a factor of six.
    #[test]
    fn the_widening_escape_returns_a_badly_wrong_derivative() {
        let mut f = |v: &[f64], out: &mut [f64]| {
            out[0] = v[0] * v[0] - 2.0e10;
            Ok(())
        };
        let x = [1.0];
        let mut base = vec![0.0; 1];
        f(&x, &mut base).unwrap();
        let (lo, hi) = unbounded(x.len());
        let jacobian = numerical_jacobian(&mut f, &x, &base, &settings(), &lo, &hi).unwrap();
        assert_ne!(jacobian[0][0], 0.0, "the widening must find *some* slope");
        assert!(
            jacobian[0][0] > 10.0,
            "widening to h = 10 gives ((1+10)^2 - 1)/10 = 12, not 2: {jacobian:?}"
        );
    }

    #[test]
    fn jacobian_never_leaks_non_finite_entries() {
        // Invalid in both directions around the anchor.
        let mut f = |x: &[f64], out: &mut [f64]| {
            out[0] = if x[0] == 1.0 { 1.0 } else { f64::NAN };
            Ok(())
        };
        let x = [1.0];
        let base = vec![1.0];
        let (lo, hi) = unbounded(x.len());
        let jacobian = numerical_jacobian(&mut f, &x, &base, &settings(), &lo, &hi).unwrap();
        assert!(jacobian[0][0].is_finite());
        assert_eq!(jacobian[0][0], 0.0);
    }

    #[test]
    fn ulp_matches_the_java_definition() {
        assert_eq!(ulp(1.0), f64::EPSILON);
        assert_eq!(ulp(0.0), f64::from_bits(1));
        assert_eq!(ulp(-1.0), f64::EPSILON);
        assert!(ulp(f64::MAX).is_finite() && ulp(f64::MAX) > 0.0);
        assert!(ulp(f64::NAN).is_nan());
        assert_eq!(ulp(f64::INFINITY), f64::INFINITY);
    }

    #[test]
    fn within_tolerance_rejects_non_finite_residuals() {
        let s = settings();
        assert!(!within_tolerance(&[f64::NAN], &[1.0], &s));
        assert!(!within_tolerance(&[f64::INFINITY], &[1.0], &s));
        assert!(within_tolerance(&[0.0], &[1.0], &s));
        assert!(!within_tolerance(&[1.0e-3], &[1.0], &s));
    }

    #[test]
    fn within_tolerance_scales_with_the_row_magnitude() {
        let s = settings();
        // A residual of 1e-4 is far outside a unit-scale equation ...
        assert!(!within_tolerance(&[1.0e-4], &[1.0], &s));
        // ... but is twelve orders below an equation whose terms are 1e12.
        assert!(within_tolerance(&[1.0e-4], &[1.0e12], &s));
        // The knee sits exactly at rel_tolerance * scale.
        assert!(!within_tolerance(&[1.0e-4], &[1.0e6], &s));
    }

    #[test]
    fn row_scale_measures_the_term_magnitudes() {
        let jacobian = vec![vec![1.0e8, 0.0], vec![0.0, 1.0]];
        let x = [2.0, 3.0];
        let mut scale = vec![0.0; 2];
        row_scale(&jacobian, &x, &mut scale);
        assert_eq!(scale[0], 2.0e8);
        assert_eq!(scale[1], 3.0);

        // Non-finite entries never produce a non-finite (and thus useless) scale.
        let jacobian = vec![vec![f64::NAN]];
        let mut scale = vec![0.0; 1];
        row_scale(&jacobian, &[1.0], &mut scale);
        assert_eq!(scale[0], 1.0);
    }

    #[test]
    fn max_abs_and_l2_norm_agree_on_simple_vectors() {
        assert_eq!(max_abs(&[]), 0.0);
        assert_eq!(max_abs(&[-3.0, 2.0]), 3.0);
        assert!(max_abs(&[1.0, f64::NAN]).is_nan());
        assert!((l2_norm(&[3.0, 4.0]) - 5.0).abs() < 1e-15);
    }

    #[test]
    fn damped_rescue_declines_when_there_is_no_curvature() {
        let mut f = |_x: &[f64], out: &mut [f64]| {
            out[0] = 1.0;
            Ok(())
        };
        let jacobian = vec![vec![0.0]];
        let (lo, hi) = unbounded(1);
        let mut scratch = Scratch::new(1);
        let rescue = damped_rescue(
            &mut f,
            &jacobian,
            &[1.0],
            &[1.0],
            1.0,
            0.0,
            1.0,
            &lo,
            &hi,
            scratch.rescue_bufs(),
        )
        .unwrap();
        assert!(rescue.is_none());
    }

    #[test]
    fn damped_rescue_descends_where_the_newton_step_overshoots() {
        let mut f = |x: &[f64], out: &mut [f64]| {
            out[0] = libm::atan(x[0]);
            Ok(())
        };
        let x = [2.0];
        let base = libm::atan(2.0);
        let jacobian = vec![vec![1.0 / 5.0]];
        let (lo, hi) = unbounded(1);
        let mut scratch = Scratch::new(1);
        let rescue = damped_rescue(
            &mut f,
            &jacobian,
            &x,
            &[base],
            base.abs(),
            0.0,
            1.0,
            &lo,
            &hi,
            scratch.rescue_bufs(),
        )
        .unwrap()
        .expect("damping must find a descent");
        assert!(rescue.norm < base.abs());
        assert!(rescue.lambda >= LM_LAMBDA_MIN);
    }

    #[test]
    fn eval_poisons_entries_the_callback_forgets() {
        let mut f = |_x: &[f64], out: &mut [f64]| {
            out[0] = 1.0; // out[1] left untouched
            Ok(())
        };
        let mut out = vec![0.0, 0.0];
        eval_residual(&mut f, &[0.0, 0.0], &mut out).unwrap();
        assert_eq!(out[0], 1.0);
        assert!(out[1].is_nan(), "an unwritten residual must not read as 0");
    }

    #[test]
    fn transcendental_system_converges() {
        // A 2x2 with genuine transcendentals: e^x + y = 3, x + sin(y) = 1.
        // Assert the residuals rather than a hand-computed root.
        let (x, report) = solve(
            |x, out| {
                out[0] = libm::exp(x[0]) + x[1] - 3.0;
                out[1] = x[0] + libm::sin(x[1]) - 1.0;
                Ok(())
            },
            &[0.0, 1.0],
        );
        let report = report.expect("must solve");
        assert!(report.converged);
        assert!((libm::exp(x[0]) + x[1] - 3.0).abs() < 1e-8, "x = {x:?}");
        assert!((x[0] + libm::sin(x[1]) - 1.0).abs() < 1e-8, "x = {x:?}");
    }

    // ---------------- analytic Jacobian (NewtonSolver.computeJacobian) ------

    /// A [`NewtonProblem`] built from two closures — the shape the engine
    /// hands the solver once the residual and its symbolic derivatives exist.
    struct AnalyticProblem<R, J> {
        residual_fn: R,
        jacobian_fn: J,
    }

    impl<R, J> NewtonProblem for AnalyticProblem<R, J>
    where
        R: FnMut(&[f64], &mut [f64]) -> Result<()>,
        J: FnMut(&[f64]) -> Option<Vec<Vec<f64>>>,
    {
        fn residual(&mut self, x: &[f64], out: &mut [f64]) -> Result<()> {
            (self.residual_fn)(x, out)
        }
        fn analytic_jacobian(&mut self, x: &[f64]) -> Option<Vec<Vec<f64>>> {
            (self.jacobian_fn)(x)
        }
    }

    /// On a smooth system the analytic and finite-difference paths must land on
    /// the same root — the Java guarantee that switching `computeJacobian`
    /// paths never changes *what* is solved, only how the direction is built.
    #[test]
    fn analytic_and_fd_jacobians_agree_on_a_smooth_system() {
        let residual = |x: &[f64], out: &mut [f64]| {
            out[0] = x[0] * x[0] + x[1] * x[1] - 4.0;
            out[1] = x[1] - x[0];
            Ok(())
        };

        let (fd, fd_report) = solve(residual, &[1.5, 0.5]);
        fd_report.expect("FD path must solve");

        let analytic_calls = std::cell::Cell::new(0usize);
        let problem = AnalyticProblem {
            residual_fn: residual,
            jacobian_fn: |x: &[f64]| {
                analytic_calls.set(analytic_calls.get() + 1);
                Some(vec![vec![2.0 * x[0], 2.0 * x[1]], vec![-1.0, 1.0]])
            },
        };
        let mut x = [1.5, 0.5];
        let report = newton_solve_problem(problem, &mut x, &settings(), None);
        report.expect("analytic path must solve");
        assert!(
            analytic_calls.get() >= 1,
            "the analytic source was never consulted"
        );

        let root = std::f64::consts::SQRT_2;
        assert!(
            (x[0] - root).abs() < 1e-9 && (x[1] - root).abs() < 1e-9,
            "x = {x:?}"
        );
        assert!(
            (fd[0] - x[0]).abs() < 1e-8 && (fd[1] - x[1]).abs() < 1e-8,
            "fd {fd:?} vs analytic {x:?}"
        );
    }

    /// The steep exponential: `exp(1e8·(x−1)) + x = 2`, root at x ≈ 1. From
    /// the start `1 + 1e-7` the finite-difference probe `h ≈ 1e-7` *doubles*
    /// the exponent, so the measured slope comes back ~2000× too steep, every
    /// Newton step lands ~2000× short, and 250 iterations cannot walk the
    /// 1e-7 distance — pure FD stalls out. The exact symbolic derivative
    /// (what A1's `Differentiator` will feed the engine) converges in a
    /// handful of iterations. This is the pure-FD-stalls /
    /// symbolic-converges pair the Java analytic path exists for.
    #[test]
    fn analytic_jacobian_solves_a_steep_exponential_where_fd_stalls() {
        let residual = |v: &[f64], out: &mut [f64]| {
            out[0] = libm::exp(1.0e8 * (v[0] - 1.0)) + v[0] - 2.0;
            Ok(())
        };

        // Pure FD provably fails from this start.
        let (_, fd_report) = solve(residual, &[1.0 + 1.0e-7]);
        assert!(
            fd_report.is_err(),
            "if FD now solves the steep exponential, tighten this test: {fd_report:?}"
        );

        // The symbolic derivative 1e8·exp(1e8·(x−1)) + 1 is exact at every
        // iterate, so each step shrinks the exponent by ~1.
        let problem = AnalyticProblem {
            residual_fn: residual,
            jacobian_fn: |v: &[f64]| {
                Some(vec![vec![1.0e8 * libm::exp(1.0e8 * (v[0] - 1.0)) + 1.0]])
            },
        };
        let mut x = [1.0 + 1.0e-7];
        let report = newton_solve_problem(problem, &mut x, &settings(), None)
            .expect("the symbolic derivative must converge where FD stalls");
        assert!(report.converged);
        assert!(
            report.iterations < 100,
            "symbolic Newton should be quick here: {}",
            report.iterations
        );
        let r = libm::exp(1.0e8 * (x[0] - 1.0)) + x[0] - 2.0;
        // The stop criterion is relative to the equation's ~1e8 row scale
        // (Java |lhs| rule), so the residual floor here is ~1e-4, not 1e-12.
        assert!(r.abs() < 1e-3, "residual {r} at x = {:.12}", x[0]);
        assert!((x[0] - 1.0).abs() < 1e-6, "x = {:.12}", x[0]);
    }

    /// A source that answers `None` (the Java `analyticalJacobian` returning
    /// null when a derivative fails to evaluate) must fall back to finite
    /// differences for that iteration and still solve.
    #[test]
    fn analytic_none_falls_back_to_fd_per_iteration() {
        let served = std::cell::Cell::new(0usize);
        let problem = AnalyticProblem {
            residual_fn: |x: &[f64], out: &mut [f64]| {
                out[0] = x[0] * x[0] - 4.0;
                Ok(())
            },
            jacobian_fn: |x: &[f64]| {
                served.set(served.get() + 1);
                if served.get() == 1 {
                    None // first iteration: pretend the derivative failed to evaluate
                } else {
                    Some(vec![vec![2.0 * x[0]]])
                }
            },
        };
        let mut x = [1.0];
        let report = newton_solve_problem(problem, &mut x, &settings(), None)
            .expect("mixed analytic/FD iterations must solve");
        assert!(report.converged);
        assert!((x[0] - 2.0).abs() < 1e-9, "x = {}", x[0]);
        assert!(served.get() >= 2, "both paths must have been exercised");
    }

    /// A malformed analytic matrix (wrong dimensions) must be ignored in favour
    /// of FD, never fed to the elimination.
    #[test]
    fn analytic_jacobian_with_wrong_dimensions_is_ignored() {
        let problem = AnalyticProblem {
            residual_fn: |x: &[f64], out: &mut [f64]| {
                out[0] = x[0] * x[0] - 4.0;
                Ok(())
            },
            jacobian_fn: |_x: &[f64]| Some(vec![vec![1.0, 2.0]]), // 1×2 for a 1×1 system
        };
        let mut x = [1.0];
        let report = newton_solve_problem(problem, &mut x, &settings(), None)
            .expect("the malformed matrix must be discarded, not fatal");
        assert!(report.converged);
        assert!((x[0] - 2.0).abs() < 1e-9, "x = {}", x[0]);
    }

    // ---------------- bounds (the three Java Math.clamp sites) --------------

    /// Every point the solver evaluates — line-search candidates, damped
    /// candidates and finite-difference probes — must stay inside `[lo, hi]`.
    /// The residual closure is the instrument: it records any violation.
    #[test]
    fn bounds_confine_every_residual_probe() {
        let violations = std::cell::RefCell::new(Vec::new());
        let bounds = [(0.5f64, 4.0f64)];
        let mut x = [0.6];
        let report = newton_solve(
            |x: &[f64], out: &mut [f64]| {
                if !(0.5..=4.0).contains(&x[0]) {
                    violations.borrow_mut().push(x[0]);
                }
                out[0] = x[0] * x[0] - 4.0;
                Ok(())
            },
            &mut x,
            &settings(),
            Some(&bounds),
        )
        .expect("must solve inside the box");
        assert!(report.converged);
        assert!((x[0] - 2.0).abs() < 1e-9, "x = {}", x[0]);
        assert!(
            violations.borrow().is_empty(),
            "probes left [0.5, 4]: {:?}",
            violations.borrow()
        );
    }

    /// The atan overshoot: the full Newton step from 2.0 lands at −3.5, which
    /// bounds [−3, 3] must clip. The solve still converges to the root 0 and
    /// never evaluates outside the box.
    #[test]
    fn bounds_clip_an_overshooting_step_and_still_converge() {
        let violations = std::cell::RefCell::new(Vec::new());
        let bounds = [(-3.0f64, 3.0f64)];
        let mut x = [2.0];
        let report = newton_solve(
            |x: &[f64], out: &mut [f64]| {
                if !(-3.0..=3.0).contains(&x[0]) {
                    violations.borrow_mut().push(x[0]);
                }
                out[0] = libm::atan(x[0]);
                Ok(())
            },
            &mut x,
            &settings(),
            Some(&bounds),
        )
        .expect("must converge inside the box");
        assert!(report.converged);
        assert!(x[0].abs() < 1e-8, "x = {}", x[0]);
        assert!(
            violations.borrow().is_empty(),
            "probes left [-3, 3]: {:?}",
            violations.borrow()
        );
    }

    /// A root outside the box cannot be reached: the iterate walks to the
    /// bound, sticks there, and the solve fails with the iterate still inside
    /// `[lo, hi]` — never with the out-of-range "solution" the unbounded
    /// solver would return.
    #[test]
    fn a_root_outside_the_bounds_is_a_bounded_failure_not_an_escape() {
        let bounds = [(0.0f64, 1.0f64)];
        let mut x = [0.5];
        let report = newton_solve(
            |x: &[f64], out: &mut [f64]| {
                out[0] = x[0] - 5.0;
                Ok(())
            },
            &mut x,
            &settings(),
            Some(&bounds),
        );
        match report {
            Err(FreesError::Solver { message }) => {
                assert!(
                    message.contains("stalled") || message.contains("Constrained"),
                    "message = {message}"
                );
            }
            other => panic!("expected a bounded stall, got {other:?}"),
        }
        assert!(
            (0.0..=1.0).contains(&x[0]),
            "the iterate must end inside the box: {}",
            x[0]
        );
        assert_eq!(x[0], 1.0, "the iterate should sit on the blocking bound");
    }

    /// The FD Jacobian at a bound probes backward (range-aware perturbation,
    /// Java computeJacobianColumn): anchored exactly on `hi`, the forward
    /// direction is pinned and the derivative must come from the backward
    /// probe — and every probe stays in range.
    #[test]
    fn jacobian_probes_respect_the_bounds() {
        let mut probed: Vec<f64> = Vec::new();
        let mut f = |x: &[f64], out: &mut [f64]| {
            probed.push(x[0]);
            out[0] = 3.0 * x[0] - 1.0;
            Ok(())
        };
        let x = [1.0];
        let base = vec![2.0];
        let s = settings();
        let (lo, hi) = (vec![0.0], vec![1.0]); // anchored exactly on hi
        let jacobian = numerical_jacobian(&mut f, &x, &base, &s, &lo, &hi).unwrap();
        assert!((jacobian[0][0] - 3.0).abs() < 1e-6, "{jacobian:?}");
        assert!(
            probed.iter().all(|&p| (0.0..=1.0).contains(&p)),
            "probes left the box: {probed:?}"
        );
        assert!(
            probed.iter().any(|&p| p < 1.0),
            "the backward probe must have run: {probed:?}"
        );
    }

    /// Bounds validation: reversed or NaN bounds and a length mismatch are
    /// clean errors, not clamp panics.
    #[test]
    fn invalid_bounds_are_rejected() {
        let f = |x: &[f64], out: &mut [f64]| {
            out[0] = x[0];
            Ok(())
        };
        let reversed = [(2.0f64, 1.0f64)];
        assert!(newton_solve(f, &mut [1.0], &settings(), Some(&reversed)).is_err());
        let nan = [(f64::NAN, 1.0f64)];
        assert!(newton_solve(f, &mut [1.0], &settings(), Some(&nan)).is_err());
        let short: [(f64, f64); 1] = [(0.0, 1.0)];
        assert!(newton_solve(f, &mut [1.0, 2.0], &settings(), Some(&short)).is_err());
    }

    /// A solution sitting exactly on a bound is still a solution — pinning is
    /// only an error when the residuals cannot be met there.
    #[test]
    fn a_root_on_the_bound_itself_still_solves() {
        let bounds = [(0.0f64, 2.0f64)];
        let mut x = [0.5];
        let report = newton_solve(
            |x: &[f64], out: &mut [f64]| {
                out[0] = x[0] - 2.0;
                Ok(())
            },
            &mut x,
            &settings(),
            Some(&bounds),
        )
        .expect("a root on the bound must be accepted");
        assert!(report.converged);
        assert_eq!(x[0], 2.0);
    }
}
