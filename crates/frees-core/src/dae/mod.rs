//! Implicit DAE `F(t, y, y') = 0` and `LINEARIZE`.
//!
//! Port of `../frEES/backend/core/src/main/java/com/frees/backend/core/dae/`
//! (7 files, 1,026 LOC) plus `ast/LinearizeSystem.java` and the
//! `EquationSystemSolver` / `DynamicSolver` code that drives them.
//!
//! # The SUNDIALS question, answered
//!
//! The Java reaches SUNDIALS IDA and KLU through JNA. A wasm build cannot, and
//! `PLAN.md` §5 ranks the options: **(a) port the BDF/IDA algorithm directly in
//! Rust**, (b) take a dependency on `diffsol`. This module took **(a)**. No new
//! crate is on the dependency list; [`solver`] is a direct implementation of
//! IDA's algorithm — same fixed-leading-coefficient BDF, same coefficient
//! recurrence, same predictor, same Newton convergence test and `cj`-ratio
//! correction, same WRMS error test and order/step selection, same
//! `IDACalcIC` line search, same `IDARootfind` Illinois search — plus a
//! Gilbert–Peierls sparse LU standing in for KLU.
//!
//! Doing it this way also *keeps* the property the parent repo documents as a
//! foot-gun: the SUNDIALS v6-vs-v7 `SUNContext`/MPI ABI trap is a
//! dynamic-linking problem, and there is no longer any dynamic linking.
//!
//! # Ground truth
//!
//! `tools/dae-probe/run.sh` drives the **real** Java `IdaDaeSolver`,
//! `DaeJacobian` and `SparseSteadyKlu` (JNA → libsundials_ida 6.4.1, which this
//! machine has) over a set of analytic DAE problems and writes
//! `fixtures/dae-oracle.json`. Every oracle constant in the tests comes from
//! that run. It exists because the document-level oracle
//! (`tools/golden-dumper`) cannot reach the IDA path until the `DYNAMIC`
//! grammar lands: a `.frees` document has no other way to ask for it.
//!
//! # Layout
//!
//! * [`assembly`] — `DaeAssembly` / `DaeResidual` / `DaeRootFn`, the residual
//!   and sparsity built from a classified `DYNAMIC` block's equation template.
//! * [`jacobian`] — the combined `J = ∂F/∂y + cj·∂F/∂y'` finite difference,
//!   with the greedy column colouring that cuts it to `#colours` evaluations.
//! * [`solver`] — the integrator and the sparse steady linear solve.
//! * [`linearize`] — `LINEARIZE`'s `(A, B, C, D)`, which Phase 9's control
//!   suite consumes.
//!
//! # Invariant this module must preserve
//!
//! Classification belongs to the `DYNAMIC` block owner (`ode/dynamic.rs`);
//! everything here takes the *result* of classification. Nothing in `dae/`
//! decides what a state is.
pub mod assembly;
pub(crate) mod colamd;
pub mod jacobian;
pub mod linearize;
pub mod solver;
