//! The implicit-DAE integrator: variable-order variable-step BDF with
//! consistent initialization, root finding, and a sparse CSC linear solve.
//!
//! Port of `../frEES/backend/core/src/main/java/com/frees/backend/core/dae/`
//! `IdaDaeSolver.java` (398) and `SparseSteadyKlu.java` (151).
//!
//! # Why the algorithm and not the library
//!
//! Those two Java files are *bindings*: `IdaDaeSolver` is JNA marshalling over
//! SUNDIALS IDA and `SparseSteadyKlu` is JNA marshalling over KLU. A wasm build
//! has no JNA and no `.so` to bind to, so what carries over is the **numerical
//! contract**, not the plumbing (`PLAN.md` §5, option (a)). This module is
//! therefore a direct Rust implementation of IDA's algorithm — the same
//! fixed-leading-coefficient BDF, the same `phi/psi/alpha/beta/sigma/gamma`
//! coefficient recurrence, the same predictor, the same Newton convergence test
//! and `cj`-ratio correction, the same WRMS error test and order/step
//! selection, the same `IDACalcIC` line-search, and the same `IDARootfind`
//! Illinois search. No new dependency is taken.
//!
//! Doing it this way also *keeps* the property the parent repo documents as a
//! foot-gun: the SUNDIALS v6-vs-v7 `SUNContext`/MPI ABI trap is a
//! dynamic-linking problem, and there is no longer any dynamic linking.
//!
//! # Verification
//!
//! `tools/dae-probe/run.sh` drives the **real** `IdaDaeSolver` (SUNDIALS IDA
//! 6.4.1 through JNA) over the analytic DAE problems the tests below replay,
//! and writes `fixtures/dae-oracle.json`. The oracle values embedded in the
//! tests came from that run; the agreement measured against them is recorded
//! there. Re-run the probe rather than trusting the numbers on faith.
//!
//! # Known deviations from the Java binding
//!
//! * **A residual error is recoverable.** The Java residual throws and the JNA
//!   callback maps that to IDA's recoverable return; this port returns `Err`
//!   and gets the same treatment (step cut by 1/4, retry, `MAXNCF` times).
//! * **No `hmax`.** `DynamicSolver.solveWithIda` never sets one, so the port
//!   defaults to unbounded like IDA does; [`IdaDaeSolver::set_max_step`] exists
//!   for callers that want it.
//! * **Root direction is not filtered here.** IDA is left monitoring both
//!   directions (the Java never calls `IDASetRootDirection` either) and
//!   [`Step::roots_found`] carries IDA's `±1` per root, so the caller applies
//!   the block's `direction` keyword itself.

// This module is a line-by-line transcription of SUNDIALS `ida.c`, whose
// vectors are indexed in lockstep (`phi[j][i]`, `yy[i]`, `ewt[i]`, `psi[j]`).
// Rewriting those sweeps as zipped iterators would make the port unreadable
// against its reference and un-reviewable against a future SUNDIALS revision,
// so the index form stays.
#![allow(clippy::needless_range_loop)]
// PARITY RULE: `!(best > 0.0)` is a NaN-rejecting guard — a NaN pivot must be
// treated as singular, which `best <= 0.0` would NOT do. The negation is
// load-bearing and stays.
#![allow(clippy::neg_cmp_op_on_partial_ord)]
// `IDACompleteStep`'s order decision is a `goto takeaction` ladder in which
// MAINTAIN is reached from two structurally different conditions (order already
// at the maximum vs. a step-size history too short to estimate order k+1).
// Collapsing them into one arm would hide which IDA branch a step took.
#![allow(clippy::if_same_then_else)]

use crate::dae::assembly::{DaeAssembly, DaeResidual, DaeRootFn};
use crate::dae::jacobian;
use crate::diag::{FreesError, Result};

// ---------------------------------------------------------------------------
// Constants — transcribed from SUNDIALS `ida.c` / `ida_impl.h`
// ---------------------------------------------------------------------------

/// Unit roundoff (`SUNDIALS UNIT_ROUNDOFF`).
const UROUND: f64 = f64::EPSILON;
/// Highest BDF order.
const MAXORD: usize = 5;
/// `MAXORD + 1` — the width of the `phi` history.
const MXORDP1: usize = MAXORD + 1;
/// `IDA_mem->ida_epcon`: the Newton convergence factor.
const EPCON: f64 = 0.33;
/// `MAXNI`: Newton iterations per step.
const MAXCOR: usize = 4;
/// `RATEMAX`: the divergence trip in the Newton convergence test.
const RATEMAX: f64 = 0.9;
/// `XRATE`: the `cj`-ratio band outside which the Jacobian is refreshed.
const XRATE: f64 = 0.25;
/// `MXNCF`: consecutive convergence failures allowed in one step.
const MAXNCF: usize = 10;
/// `MXNEF`: consecutive error-test failures allowed in one step.
const MAXNEF: usize = 10;
/// `MAXNJ`: Jacobian setups allowed inside `IDACalcIC`.
const MAXNJ: usize = 4;
/// `MAXNIT`: Newton iterations inside `IDACalcIC`.
const MAXNIT: usize = 10;
/// `MAXNH`: step-size retries inside `IDACalcIC`.
const MAXNH: usize = 5;
/// `ALPHALS`: the line-search alpha condition.
const ALPHALS: f64 = 1e-4;

/// `IDACalcIC` option: solve for the algebraic components of `y` and the
/// derivatives of the differential components. Needs
/// [`IdaDaeSolver::set_variable_id`].
pub const IDA_YA_YDP_INIT: i32 = 1;
/// `IDACalcIC` option: solve for all of `y`, holding `y'` fixed.
pub const IDA_Y_INIT: i32 = 2;

/// `IDASolve` returned normally at `tout`.
pub const IDA_SUCCESS: i32 = 0;
/// `IDASolve` stopped early because a root function changed sign.
pub const IDA_ROOT_RETURN: i32 = 2;

/// Above this dimension the frees IDA path switches from the dense linear
/// solver to the sparse one. Transcribed from `DynamicSolver.SPARSE_THRESHOLD`.
pub const SPARSE_THRESHOLD: usize = 24;

// ---------------------------------------------------------------------------
// Dense LU
// ---------------------------------------------------------------------------

/// An LU factorization with partial pivoting (`SUNLinSol_Dense`'s `denseGETRF`).
struct DenseLu {
    lu: Vec<Vec<f64>>,
    piv: Vec<usize>,
}

fn dense_lu_factor(mut a: Vec<Vec<f64>>) -> Option<DenseLu> {
    let n = a.len();
    let mut piv = vec![0usize; n];
    for k in 0..n {
        let mut p = k;
        let mut best = a[k][k].abs();
        for (r, row) in a.iter().enumerate().skip(k + 1) {
            if row[k].abs() > best {
                best = row[k].abs();
                p = r;
            }
        }
        if !(best > 0.0) {
            return None; // structurally or numerically singular
        }
        piv[k] = p;
        if p != k {
            a.swap(p, k);
        }
        let pivot = a[k][k];
        for i in (k + 1)..n {
            let f = a[i][k] / pivot;
            a[i][k] = f;
            if f != 0.0 {
                for j in (k + 1)..n {
                    let above = a[k][j];
                    a[i][j] -= f * above;
                }
            }
        }
    }
    Some(DenseLu { lu: a, piv })
}

fn dense_lu_solve(f: &DenseLu, b: &mut [f64]) {
    let n = b.len();
    // `dense_lu_factor` swaps whole rows, so the stored multipliers are already
    // in pivot order: `P·A = L·U`. Every interchange must therefore be applied
    // to `b` FIRST (LAPACK's `dlaswp` before `dtrsm`). Interleaving the swaps
    // with the forward substitution — correct only when `L` is stored
    // unpermuted — silently returns a wrong solution on any matrix that
    // actually pivots.
    for k in 0..n {
        b.swap(k, f.piv[k]);
    }
    for k in 0..n {
        for i in (k + 1)..n {
            b[i] -= f.lu[i][k] * b[k];
        }
    }
    for k in (0..n).rev() {
        b[k] /= f.lu[k][k];
        for i in 0..k {
            b[i] -= f.lu[i][k] * b[k];
        }
    }
}

// ---------------------------------------------------------------------------
// Sparse CSC + LU  (the SparseSteadyKlu replacement)
// ---------------------------------------------------------------------------

/// A square compressed-sparse-column matrix with a **fixed** pattern.
///
/// This is the Rust shape of the `SUNSparseMatrix(n, n, nnz, CSC_MAT)` that
/// `SparseSteadyKlu` and the IDA sparse path both allocate: the pattern is
/// written once (the block's structural incidence does not change) and only the
/// values are refilled per iteration, which is exactly why KLU can keep its
/// symbolic factorization.
#[derive(Debug, Clone, PartialEq)]
pub struct SparseCsc {
    n: usize,
    /// `col_ptr[c] .. col_ptr[c+1]` indexes column `c`'s entries.
    col_ptr: Vec<usize>,
    /// Row index per stored entry, ascending within each column.
    row_idx: Vec<usize>,
    /// Value per stored entry, in the same order.
    values: Vec<f64>,
}

impl SparseCsc {
    /// Builds the pattern from `column_rows[c] = ascending row list of column c`.
    ///
    /// Port of the pattern write in `SparseSteadyKlu`'s constructor, including
    /// its `require(nnz > 0, "empty pattern")` refusal.
    pub fn from_columns(column_rows: &[Vec<usize>]) -> Option<SparseCsc> {
        let n = column_rows.len();
        let mut col_ptr = Vec::with_capacity(n + 1);
        let mut row_idx = Vec::new();
        let mut pos = 0usize;
        for rows in column_rows {
            col_ptr.push(pos);
            for &i in rows {
                if i >= n {
                    return None;
                }
                row_idx.push(i);
                pos += 1;
            }
        }
        col_ptr.push(pos);
        if pos == 0 {
            return None;
        }
        Some(SparseCsc {
            n,
            col_ptr,
            row_idx,
            values: vec![0.0; pos],
        })
    }

    pub fn dimension(&self) -> usize {
        self.n
    }

    /// Number of stored entries; the caller sizes its value buffer with this.
    /// Port of `SparseSteadyKlu.nonzeros`.
    pub fn nonzeros(&self) -> usize {
        self.values.len()
    }

    /// Refills the values in the CSC order the pattern was declared in.
    pub fn set_values(&mut self, csc_values: &[f64]) -> Result<()> {
        if csc_values.len() != self.values.len() {
            return Err(FreesError::solver(format!(
                "sparse matrix expects {} values, got {}",
                self.values.len(),
                csc_values.len()
            )));
        }
        self.values.copy_from_slice(csc_values);
        Ok(())
    }

    pub fn col_ptr(&self) -> &[usize] {
        &self.col_ptr
    }

    pub fn row_idx(&self) -> &[usize] {
        &self.row_idx
    }

    pub fn values(&self) -> &[f64] {
        &self.values
    }

    pub fn values_mut(&mut self) -> &mut [f64] {
        &mut self.values
    }

    pub fn col_ptr_and_values_mut(&mut self) -> (&[usize], &mut [f64]) {
        (&self.col_ptr, &mut self.values)
    }
}

/// A sparse LU factorization with partial pivoting.
///
/// Left-looking Gilbert–Peierls: column `k` of `L`/`U` comes from a sparse
/// triangular solve `L x = A(:,k)` whose nonzero pattern is found by depth-first
/// reachability, then a pivot is chosen by magnitude among the not-yet-pivotal
/// rows. That is the numerical core of KLU minus its BTF block-triangular
/// reordering and AMD fill-reducing permutation — which affect speed and fill,
/// never the answer.
///
/// **Crossover.** Fill is not controlled here, so a matrix whose natural
/// ordering fills badly costs more than KLU would. For the C-R-C networks this
/// path exists for (banded, one storage state per cell) the natural ordering is
/// already near-optimal. If a future model shows heavy fill, the fix is an AMD
/// ordering in front of this factorization, not a different factorization.
struct SparseLu {
    n: usize,
    /// `L` in CSC with unit diagonal, rows already permuted.
    lp: Vec<usize>,
    li: Vec<usize>,
    lx: Vec<f64>,
    /// `U` in CSC.
    up: Vec<usize>,
    ui: Vec<usize>,
    ux: Vec<f64>,
    /// `pinv[row] = k` when `row` is the `k`-th pivot row.
    pinv: Vec<usize>,
}

impl SparseLu {
    fn factor(a: &SparseCsc) -> Option<SparseLu> {
        let n = a.n;
        let mut x = vec![0.0f64; n];
        let mut pinv = vec![usize::MAX; n];
        let mut lp = vec![0usize; n + 1];
        let mut up = vec![0usize; n + 1];
        let (mut li, mut lx): (Vec<usize>, Vec<f64>) = (Vec::new(), Vec::new());
        let (mut ui, mut ux): (Vec<usize>, Vec<f64>) = (Vec::new(), Vec::new());
        // DFS workspace. `mark[i] == k + 1` means "row i already reached while
        // building column k", which avoids clearing a visited set per column.
        let mut mark = vec![0usize; n];
        let mut stack: Vec<usize> = Vec::with_capacity(n);
        let mut pstack: Vec<usize> = Vec::with_capacity(n);
        let mut order: Vec<usize> = Vec::with_capacity(n);

        for k in 0..n {
            lp[k] = li.len();
            up[k] = ui.len();
            order.clear();
            // --- symbolic: reachability of A(:,k) through L (cs_dfs)
            for p in a.col_ptr[k]..a.col_ptr[k + 1] {
                let j = a.row_idx[p];
                if mark[j] == k + 1 {
                    continue;
                }
                stack.clear();
                pstack.clear();
                stack.push(j);
                pstack.push(usize::MAX);
                mark[j] = k + 1;
                while let Some(&node) = stack.last() {
                    let col = pinv[node];
                    // Only a pivotal row has a column of L to descend into; at
                    // this point every pivotal row has `pinv < k`, so `lp[col+1]`
                    // is already final.
                    let (lo, hi) = if col == usize::MAX {
                        (0, 0)
                    } else {
                        (lp[col], lp[col + 1])
                    };
                    let slot = pstack.last_mut().expect("stacks move together");
                    if *slot == usize::MAX {
                        *slot = lo;
                    }
                    let mut done = true;
                    while *slot < hi {
                        let i = li[*slot];
                        *slot += 1;
                        if mark[i] != k + 1 {
                            mark[i] = k + 1;
                            stack.push(i);
                            pstack.push(usize::MAX);
                            done = false;
                            break;
                        }
                    }
                    if done {
                        order.push(node);
                        stack.pop();
                        pstack.pop();
                    }
                }
            }
            // --- numeric: scatter, then solve in topological order
            for p in a.col_ptr[k]..a.col_ptr[k + 1] {
                x[a.row_idx[p]] = a.values[p];
            }
            for &node in order.iter().rev() {
                let col = pinv[node];
                if col == usize::MAX {
                    continue;
                }
                let xj = x[node];
                if xj == 0.0 {
                    continue;
                }
                for p in (lp[col] + 1)..lp[col + 1] {
                    x[li[p]] -= lx[p] * xj;
                }
            }
            // --- pivot among the rows that are not yet pivotal
            let mut ipiv = usize::MAX;
            let mut best = 0.0f64;
            for &i in &order {
                if pinv[i] == usize::MAX {
                    let t = x[i].abs();
                    if t > best {
                        best = t;
                        ipiv = i;
                    }
                } else {
                    ui.push(pinv[i]);
                    ux.push(x[i]);
                }
            }
            if ipiv == usize::MAX || !(best > 0.0) {
                return None;
            }
            let pivot = x[ipiv];
            ui.push(k);
            ux.push(pivot);
            pinv[ipiv] = k;
            li.push(ipiv);
            lx.push(1.0);
            for &i in &order {
                if pinv[i] == usize::MAX {
                    li.push(i);
                    lx.push(x[i] / pivot);
                }
                x[i] = 0.0;
            }
        }
        lp[n] = li.len();
        up[n] = ui.len();
        // Rewrite L's row indices in pivot order so the solve needs no lookup.
        for slot in li.iter_mut() {
            *slot = pinv[*slot];
        }
        Some(SparseLu {
            n,
            lp,
            li,
            lx,
            up,
            ui,
            ux,
            pinv,
        })
    }

    /// Solves `A x = b` from the factorization.
    fn solve(&self, b: &[f64]) -> Vec<f64> {
        let n = self.n;
        // Permute: x = P b.
        let mut x = vec![0.0f64; n];
        for i in 0..n {
            x[self.pinv[i]] = b[i];
        }
        // Forward substitution through unit-diagonal L.
        for c in 0..n {
            let xc = x[c];
            if xc != 0.0 {
                for p in (self.lp[c] + 1)..self.lp[c + 1] {
                    x[self.li[p]] -= self.lx[p] * xc;
                }
            }
        }
        // Back substitution through U (its last entry per column is the diagonal).
        for c in (0..n).rev() {
            let diag = self.ux[self.up[c + 1] - 1];
            x[c] /= diag;
            let xc = x[c];
            if xc != 0.0 {
                for p in self.up[c]..(self.up[c + 1] - 1) {
                    x[self.ui[p]] -= self.ux[p] * xc;
                }
            }
        }
        x
    }
}

/// Standalone CSC + sparse-LU linear solver for the **steady** Newton path.
///
/// Port of `SparseSteadyKlu.java`. One instance is built per solved block — the
/// pattern is fixed by the block's structural incidence — and each Newton
/// iteration refills the values and solves. It degrades exactly as the Java
/// does: [`SparseSteady::create`] returns `None` for a pattern it cannot serve,
/// and [`SparseSteady::solve`] returns `None` (never an error) on a singular
/// factorization, so the caller falls back to the dense/SVD path without
/// special-casing.
pub struct SparseSteady {
    matrix: SparseCsc,
}

impl SparseSteady {
    /// Builds a solver for the fixed CSC pattern `column -> ascending row list`.
    pub fn create(column_rows: &[Vec<usize>]) -> Option<SparseSteady> {
        SparseCsc::from_columns(column_rows).map(|matrix| SparseSteady { matrix })
    }

    /// Number of stored entries; the caller sizes its value buffer with this.
    pub fn nonzeros(&self) -> usize {
        self.matrix.nonzeros()
    }

    /// Refills the matrix values (same CSC order the pattern was declared in),
    /// factorizes and solves `A x = b`. Returns `None` when the factorization
    /// reports failure (a structurally or numerically singular matrix).
    pub fn solve(&mut self, csc_values: &[f64], b: &[f64]) -> Option<Vec<f64>> {
        if b.len() != self.matrix.n || self.matrix.set_values(csc_values).is_err() {
            return None;
        }
        SparseLu::factor(&self.matrix).map(|lu| lu.solve(b))
    }
}

// ---------------------------------------------------------------------------
// The integrator
// ---------------------------------------------------------------------------

/// One integrator return: the state at `t`, the solver flag, and any roots that
/// fired. Port of the `IdaDaeSolver.Step` record.
#[derive(Debug, Clone, PartialEq)]
pub struct Step {
    pub t: f64,
    pub y: Vec<f64>,
    pub yp: Vec<f64>,
    pub flag: i32,
    /// IDA's `iroots`: `+1` where the root function was increasing through
    /// zero, `-1` decreasing, `0` where it did not fire.
    pub roots_found: Vec<i32>,
}

impl Step {
    pub fn root_return(&self) -> bool {
        self.flag == IDA_ROOT_RETURN
    }
}

/// Which linear solver the integrator runs.
enum LinearPath {
    /// `SUNDenseMatrix` + `SUNLinSol_Dense`, with IDA's own difference-quotient
    /// Jacobian.
    Dense {
        fp: Vec<f64>,
        y_pert: Vec<f64>,
        yp_pert: Vec<f64>,
    },
    /// `SUNSparseMatrix(CSC)` + KLU, with the frees `DaeJacobian` coloured FD.
    Sparse {
        col_rows: Vec<Vec<usize>>,
        color: Vec<usize>,
        matrix: SparseCsc,
        scratch: jacobian::DaeJacobianScratch,
    },
}

enum Factored {
    Dense(DenseLu),
    Sparse(SparseLu),
}

/// Internal step outcomes, mirroring `ida.c`'s `PREDICT_AGAIN` / flag protocol.
#[derive(Debug, Clone, Copy, PartialEq)]
enum NlFlag {
    Success,
    /// Recoverable: cut the step and predict again.
    Recoverable,
    /// The local error test failed.
    ErrorTest,
}

/// The frees-facing implicit-DAE integrator.
///
/// ```ignore
/// let mut s = IdaDaeSolver::new(n, &residual)?;
/// s.set_tolerances(1e-8, 1e-8);
/// s.set_variable_id(&id)?;          // 1 = differential, 0 = algebraic
/// s.set_roots(nroots, &root_fn);    // §4.8 Tier-2 structural events
/// s.init(t0, &y0, &yp0)?;
/// s.calc_consistent_ic(IDA_YA_YDP_INIT, t0 + 1e-3)?;
/// let out = s.step(tout)?;
/// ```
pub struct IdaDaeSolver<'a> {
    res: &'a dyn DaeResidual,
    n: usize,

    // user settings
    rtol: f64,
    atol: f64,
    max_steps: u64,
    hmax_inv: f64,
    variable_id: Option<Vec<f64>>,
    nroots: usize,
    root_fn: Option<&'a dyn DaeRootFn>,
    path: LinearPath,

    // BDF history and coefficients
    phi: Vec<Vec<f64>>,
    psi: [f64; MXORDP1],
    alpha: [f64; MXORDP1],
    beta: [f64; MXORDP1],
    sigma: [f64; MXORDP1],
    gamma: [f64; MXORDP1],

    tn: f64,
    hh: f64,
    hused: f64,
    kk: usize,
    kused: usize,
    knew: usize,
    phase: i32,
    ns: usize,
    nst: u64,
    rr: f64,
    tretlast: f64,

    cj: f64,
    cjlast: f64,
    cjold: f64,
    cjratio: f64,
    ss: f64,
    /// `IDA_mem->ida_oldnrm`, kept across the Newton loop's iterations so the
    /// convergence-rate estimate can be formed.
    oldnrm: f64,
    eps_newt: f64,
    toldel: f64,

    ewt: Vec<f64>,
    yy: Vec<f64>,
    yp: Vec<f64>,
    yypredict: Vec<f64>,
    yppredict: Vec<f64>,
    ee: Vec<f64>,
    savres: Vec<f64>,

    jac: Option<Factored>,
    jcur: bool,
    force_setup: bool,

    // root-finding state
    tlo: f64,
    thi: f64,
    trout: f64,
    ttol: f64,
    glo: Vec<f64>,
    ghi: Vec<f64>,
    grout: Vec<f64>,
    iroots: Vec<i32>,
    gactive: Vec<bool>,
    rootdir: Vec<i32>,
    irfnd: bool,

    initialized: bool,
}

impl<'a> IdaDaeSolver<'a> {
    pub fn new(n: usize, res: &'a dyn DaeResidual) -> Result<IdaDaeSolver<'a>> {
        if n < 1 {
            return Err(FreesError::solver("DAE dimension must be >= 1"));
        }
        Ok(IdaDaeSolver {
            res,
            n,
            rtol: 1e-6,
            atol: 1e-8,
            max_steps: 50_000,
            hmax_inv: 0.0,
            variable_id: None,
            nroots: 0,
            root_fn: None,
            path: LinearPath::Dense {
                fp: vec![0.0; n],
                y_pert: vec![0.0; n],
                yp_pert: vec![0.0; n],
            },
            phi: vec![vec![0.0; n]; MXORDP1],
            psi: [0.0; MXORDP1],
            alpha: [0.0; MXORDP1],
            beta: [0.0; MXORDP1],
            sigma: [0.0; MXORDP1],
            gamma: [0.0; MXORDP1],
            tn: 0.0,
            hh: 0.0,
            hused: 0.0,
            kk: 1,
            kused: 0,
            knew: 1,
            phase: 0,
            ns: 0,
            nst: 0,
            rr: 1.0,
            tretlast: 0.0,
            cj: 0.0,
            cjlast: 0.0,
            cjold: 0.0,
            cjratio: 1.0,
            ss: 20.0,
            oldnrm: 0.0,
            eps_newt: EPCON,
            toldel: 1e-4 * EPCON,
            ewt: vec![0.0; n],
            yy: vec![0.0; n],
            yp: vec![0.0; n],
            yypredict: vec![0.0; n],
            yppredict: vec![0.0; n],
            ee: vec![0.0; n],
            savres: vec![0.0; n],
            jac: None,
            jcur: false,
            force_setup: false,
            tlo: 0.0,
            thi: 0.0,
            trout: 0.0,
            ttol: 0.0,
            glo: Vec::new(),
            ghi: Vec::new(),
            grout: Vec::new(),
            iroots: Vec::new(),
            gactive: Vec::new(),
            rootdir: Vec::new(),
            irfnd: false,
            initialized: false,
        })
    }

    /// Builds a solver configured the way `DynamicSolver.solveWithIda` does:
    /// tolerances, variable id, roots, and the sparse linear solver above
    /// [`SPARSE_THRESHOLD`].
    pub fn for_assembly(
        dae: &'a DaeAssembly<'a>,
        rtol: f64,
        atol: f64,
    ) -> Result<IdaDaeSolver<'a>> {
        let mut s = IdaDaeSolver::new(dae.n, dae.residual.as_ref())?;
        s.set_tolerances(rtol, atol);
        s.set_variable_id(&dae.id)?;
        if dae.event_count() > 0 {
            if let Some(rf) = dae.root_fn.as_ref() {
                s.set_roots(dae.event_count(), rf.as_ref());
            }
        }
        if dae.n > SPARSE_THRESHOLD {
            s.set_sparsity(&dae.sparsity)?;
        }
        Ok(s)
    }

    pub fn set_tolerances(&mut self, rtol: f64, atol: f64) -> &mut Self {
        self.rtol = rtol;
        self.atol = atol;
        self
    }

    pub fn set_max_steps(&mut self, max_steps: u64) -> &mut Self {
        self.max_steps = max_steps;
        self
    }

    /// Caps `|h|`. IDA's default (and the frees IDA path's) is no cap.
    pub fn set_max_step(&mut self, hmax: f64) -> &mut Self {
        self.hmax_inv = if hmax > 0.0 { 1.0 / hmax } else { 0.0 };
        self
    }

    /// Marks each component differential (1) or algebraic (0); needed for
    /// [`IDA_YA_YDP_INIT`].
    pub fn set_variable_id(&mut self, id: &[f64]) -> Result<&mut Self> {
        if id.len() != self.n {
            return Err(FreesError::solver("id length must equal the DAE dimension"));
        }
        self.variable_id = Some(id.to_vec());
        Ok(self)
    }

    /// Registers `nroots` switching functions (§4.8 Tier-2 events). Call before
    /// [`IdaDaeSolver::init`].
    pub fn set_roots(&mut self, nroots: usize, root_fn: &'a dyn DaeRootFn) -> &mut Self {
        self.nroots = nroots;
        self.root_fn = Some(root_fn);
        self.glo = vec![0.0; nroots];
        self.ghi = vec![0.0; nroots];
        self.grout = vec![0.0; nroots];
        self.iroots = vec![0; nroots];
        self.gactive = vec![true; nroots];
        self.rootdir = vec![0; nroots];
        self
    }

    /// Selects the **sparse** linear solver with the given per-row column
    /// dependency pattern (as produced by the DAE assembler), instead of the
    /// dense default. The combined system matrix is filled by the coloured
    /// finite-difference Jacobian of [`crate::dae::jacobian`].
    ///
    /// Port of `IdaDaeSolver.setSparsity`, including its transpose into
    /// per-column row lists and its `DaeJacobian.colorColumns` call.
    pub fn set_sparsity(&mut self, sparsity_rows: &[Vec<usize>]) -> Result<&mut Self> {
        if sparsity_rows.len() != self.n {
            return Err(FreesError::solver(
                "sparsity must have one row per equation",
            ));
        }
        let col_rows = jacobian::transpose_pattern(sparsity_rows, self.n);
        let color = jacobian::color_columns(sparsity_rows, self.n);
        match SparseCsc::from_columns(&col_rows) {
            // An empty or malformed pattern degrades to dense, exactly as the
            // Java degrades when the native sparse libraries are absent.
            None => {
                self.path = LinearPath::Dense {
                    fp: vec![0.0; self.n],
                    y_pert: vec![0.0; self.n],
                    yp_pert: vec![0.0; self.n],
                };
            }
            Some(matrix) => {
                let scratch = jacobian::DaeJacobianScratch::new(self.n);
                self.path = LinearPath::Sparse {
                    col_rows,
                    color,
                    matrix,
                    scratch,
                };
            }
        }
        Ok(self)
    }

    /// Initializes the integrator at `(t0, y0, yp0)`.
    pub fn init(&mut self, t0: f64, y0: &[f64], yp0: &[f64]) -> Result<()> {
        if y0.len() != self.n || yp0.len() != self.n {
            return Err(FreesError::solver("y0/yp0 length must equal dimension"));
        }
        for row in self.phi.iter_mut() {
            row.iter_mut().for_each(|v| *v = 0.0);
        }
        self.phi[0].copy_from_slice(y0);
        self.phi[1].copy_from_slice(yp0);
        self.yy.copy_from_slice(y0);
        self.yp.copy_from_slice(yp0);
        self.tn = t0;
        self.tretlast = t0;
        self.nst = 0;
        self.kk = 1;
        self.kused = 0;
        self.knew = 1;
        self.phase = 0;
        self.ns = 0;
        self.hh = 0.0;
        self.hused = 0.0;
        self.cj = 0.0;
        self.cjold = 0.0;
        self.cjlast = 0.0;
        self.cjratio = 1.0;
        self.ss = 20.0;
        self.eps_newt = EPCON;
        self.toldel = 1e-4 * EPCON;
        self.jac = None;
        self.jcur = false;
        self.force_setup = false;
        self.irfnd = false;
        self.psi = [0.0; MXORDP1];
        let y = self.phi[0].clone();
        self.set_ewt(&y)?;
        self.initialized = true;
        Ok(())
    }

    /// Re-initializes at a new `(t0, y0, yp0)` keeping the same problem
    /// structure — the §4.8 mode-frozen restart after a structural switch.
    /// Roots and linear-solver choice persist.
    pub fn reinit(&mut self, t0: f64, y0: &[f64], yp0: &[f64]) -> Result<()> {
        self.require_init()?;
        self.init(t0, y0, yp0)
    }

    /// The `(y, y')` most recently handed out — the initial condition after
    /// [`Self::init`] / [`Self::calc_consistent_ic`], and the value at the
    /// returned time after a [`Self::step`].
    ///
    /// These read `yy`/`yp` rather than the history, because `phi[1]` is `h·y'`
    /// once integration starts (IDA scales it in place) and would silently hand
    /// back a scaled derivative.
    pub fn current_state(&self) -> Vec<f64> {
        self.yy.clone()
    }

    pub fn current_derivative(&self) -> Vec<f64> {
        self.yp.clone()
    }

    fn require_init(&self) -> Result<()> {
        if self.initialized {
            Ok(())
        } else {
            Err(FreesError::solver(
                "IdaDaeSolver::init(...) must be called first",
            ))
        }
    }

    // ── error weights and norms ──────────────────────────────────────────────

    fn set_ewt(&mut self, y: &[f64]) -> Result<()> {
        for i in 0..self.n {
            let w = self.rtol * y[i].abs() + self.atol;
            if !(w > 0.0) {
                return Err(FreesError::solver(
                    "the error weight vector has a non-positive component; \
                     check that rtol/atol are positive",
                ));
            }
            self.ewt[i] = 1.0 / w;
        }
        Ok(())
    }

    /// `N_VWrmsNorm`: `sqrt(Σ (v_i·w_i)² / n)`.
    fn wrms(&self, v: &[f64]) -> f64 {
        let mut sum = 0.0;
        for i in 0..self.n {
            let p = v[i] * self.ewt[i];
            sum += p * p;
        }
        (sum / self.n as f64).sqrt()
    }

    // ── linear system: setup and solve ───────────────────────────────────────

    /// `idaLsSetup`: build `J = ∂F/∂y + cj·∂F/∂y'` and factor it.
    fn lsetup(&mut self, y: &[f64], yp: &[f64], f0: &[f64]) -> Result<bool> {
        let factored = match &mut self.path {
            LinearPath::Dense {
                fp,
                y_pert,
                yp_pert,
            } => {
                let mut j = vec![vec![0.0; self.n]; self.n];
                if fp.len() < self.n {
                    fp.resize(self.n, 0.0);
                    y_pert.resize(self.n, 0.0);
                    yp_pert.resize(self.n, 0.0);
                }
                y_pert[..self.n].copy_from_slice(y);
                yp_pert[..self.n].copy_from_slice(yp);
                for c in 0..self.n {
                    let inc =
                        jacobian::ida_dense_increment(y[c], yp[c], self.hh, 1.0 / self.ewt[c]);
                    y_pert[c] += inc;
                    yp_pert[c] += self.cj * inc;
                    self.res.eval(
                        self.tn,
                        &y_pert[..self.n],
                        &yp_pert[..self.n],
                        &mut fp[..self.n],
                    )?;
                    let inv = 1.0 / inc;
                    for i in 0..self.n {
                        j[i][c] = inv * (fp[i] - f0[i]);
                    }
                    y_pert[c] = y[c];
                    yp_pert[c] = yp[c];
                }
                dense_lu_factor(j).map(Factored::Dense)
            }
            LinearPath::Sparse {
                col_rows,
                color,
                matrix,
                scratch,
            } => {
                // Directly write coloured finite differences into existing CSC storage.
                // Eliminates the n × n dense intermediate buffer, avoids recomputing f0,
                // and reuses perturbation and residual scratch vectors.
                let (col_ptr, values) = matrix.col_ptr_and_values_mut();
                jacobian::csc_colored_into(
                    self.res,
                    self.tn,
                    self.cj,
                    y,
                    yp,
                    Some(f0),
                    col_rows,
                    color,
                    col_ptr,
                    values,
                    scratch,
                )?;
                SparseLu::factor(matrix).map(Factored::Sparse)
            }
        };
        self.jac = factored;
        self.jcur = true;
        self.cjold = self.cj;
        self.cjratio = 1.0;
        self.ss = 20.0;
        self.force_setup = false;
        Ok(self.jac.is_some())
    }

    /// `idaLsSolve`: solve `J x = b`, then apply IDA's `cj`-ratio correction.
    fn lsolve(&self, b: &mut Vec<f64>) -> bool {
        match &self.jac {
            None => false,
            Some(Factored::Dense(lu)) => {
                dense_lu_solve(lu, b);
                self.apply_cj_correction(b);
                b.iter().all(|v| v.is_finite())
            }
            Some(Factored::Sparse(lu)) => {
                *b = lu.solve(b);
                self.apply_cj_correction(b);
                b.iter().all(|v| v.is_finite())
            }
        }
    }

    fn apply_cj_correction(&self, b: &mut [f64]) {
        if self.cjratio != 1.0 {
            let s = 2.0 / (1.0 + self.cjratio);
            b.iter_mut().for_each(|v| *v *= s);
        }
    }

    // ── BDF coefficients, prediction, correction ─────────────────────────────

    /// `IDASetCoeffs`: the fixed-leading-coefficient BDF coefficients for the
    /// current `(h, k)`, the leading coefficient `cj`, the error coefficient
    /// `ck`, the `phi → phi*` rescale, and the time advance.
    fn set_coeffs(&mut self) -> f64 {
        if self.hh != self.hused || self.kk != self.kused {
            self.ns = 0;
        }
        self.ns = (self.ns + 1).min(self.kused + 2);
        if self.kk + 1 >= self.ns {
            self.beta[0] = 1.0;
            self.alpha[0] = 1.0;
            let mut temp1 = self.hh;
            self.gamma[0] = 0.0;
            self.sigma[0] = 1.0;
            for i in 1..=self.kk {
                let temp2 = self.psi[i - 1];
                self.psi[i - 1] = temp1;
                self.beta[i] = self.beta[i - 1] * self.psi[i - 1] / temp2;
                temp1 = temp2 + self.hh;
                self.alpha[i] = self.hh / temp1;
                self.sigma[i] = i as f64 * self.sigma[i - 1] * self.alpha[i];
                self.gamma[i] = self.gamma[i - 1] + self.alpha[i - 1] / self.hh;
            }
            self.psi[self.kk] = temp1;
        }
        // SUNDIALS sums `alpha0` over `alpha[0..kk-1]` — from index 0, where
        // `alpha[0] = 1` — not over `alpha[1..kk]`. The first transcription
        // used the latter; at k = 5 with constant h that inflates `ck` (which
        // is `|alpha[kk] + alphas − alpha0|`) about fourfold, so the error
        // test rejected steps real IDA accepts, and a long smooth stretch
        // ended in a spurious rejection cascade (`pressure-cooker`, t ≈ 696).
        let mut alphas = 0.0;
        let mut alpha0 = 0.0;
        for i in 0..self.kk {
            alphas -= 1.0 / (i + 1) as f64;
            alpha0 -= self.alpha[i];
        }
        self.cjlast = self.cj;
        self.cj = -alphas / self.hh;
        let mut ck = (self.alpha[self.kk] + alphas - alpha0).abs();
        ck = ck.max(self.alpha[self.kk]);
        // phi -> phi*
        if self.ns <= self.kk {
            for i in self.ns..=self.kk {
                let b = self.beta[i];
                self.phi[i].iter_mut().for_each(|v| *v *= b);
            }
        }
        self.tn += self.hh;
        ck
    }

    /// `IDAPredict`: `yypredict = Σ phi[j]`, `yppredict = Σ gamma[j]·phi[j]`.
    fn predict(&mut self) {
        for i in 0..self.n {
            let mut yysum = 0.0;
            for j in 0..=self.kk {
                yysum += self.phi[j][i];
            }
            self.yypredict[i] = yysum;
            let mut ypsum = 0.0;
            for j in 1..=self.kk {
                ypsum += self.gamma[j] * self.phi[j][i];
            }
            self.yppredict[i] = ypsum;
        }
    }

    /// `IDARestore`: undo `set_coeffs` after a failed attempt.
    ///
    /// Both halves matter. `set_coeffs` overwrote `psi[j-1]` with
    /// `psi_old[j-1] + h` (the step's backward time differences) and scaled
    /// `phi` to `phi*`; a retry with a smaller `h` recomputes the coefficients
    /// from `psi`, so leaving `psi` advanced poisons every following attempt —
    /// the symptom is an error test that never passes however far `h` is cut.
    ///
    /// The `phi` half mirrors `set_coeffs` **exactly**: only columns
    /// `ns..=kk` were scaled to `phi*`, and only when `ns <= kk`, so only
    /// those may be scaled back (SUNDIALS `IDARestore`, verbatim). Dividing
    /// all of `1..=kk` — the first transcription of this function did — is
    /// wrong precisely after a stretch of constant `(h, k)` steps: `ns` sits
    /// at `kused + 2 > kk`, `set_coeffs` scaled *nothing*, and the
    /// unconditional un-scale divides the history by stale `beta`s. The
    /// symptom was `pressure-cooker` stalling at t ≈ 696 s: the first
    /// rejection after ~10 s of clean k = 5 stepping corrupted `phi`, and
    /// every retry then plateaued at the same weighted error however far `h`
    /// was cut.
    fn restore(&mut self, saved_t: f64) {
        self.tn = saved_t;
        for j in 1..=self.kk {
            self.psi[j - 1] = self.psi[j] - self.hh;
        }
        if self.ns <= self.kk {
            for j in self.ns..=self.kk {
                let inv = 1.0 / self.beta[j];
                self.phi[j].iter_mut().for_each(|v| *v *= inv);
            }
        }
    }

    /// `IDANls`: the Newton solve for the correction `ee`.
    fn nls(&mut self) -> Result<NlFlag> {
        let mut call_lsetup = false;
        if self.nst == 0 {
            self.cjold = self.cj;
            self.ss = 20.0;
            call_lsetup = true;
        }
        self.cjratio = self.cj / self.cjold;
        let temp1 = (1.0 - XRATE) / (1.0 + XRATE);
        let temp2 = 1.0 / temp1;
        if self.cjratio < temp1 || self.cjratio > temp2 {
            call_lsetup = true;
        }
        if self.force_setup {
            call_lsetup = true;
        }
        if self.cj != self.cjlast {
            self.ss = 100.0;
        }

        let mut jbad_retry = true;
        loop {
            self.ee.iter_mut().for_each(|v| *v = 0.0);
            // yy/yp at the predictor, and the residual there.
            self.yy.copy_from_slice(&self.yypredict);
            self.yp.copy_from_slice(&self.yppredict);
            let mut delta = vec![0.0; self.n];
            if self
                .res
                .eval(self.tn, &self.yy, &self.yp, &mut delta)
                .is_err()
            {
                return Ok(NlFlag::Recoverable);
            }
            self.savres.copy_from_slice(&delta);

            if call_lsetup {
                let y = self.yy.clone();
                let yp = self.yp.clone();
                let f0 = self.savres.clone();
                if !self.lsetup(&y, &yp, &f0)? {
                    return Ok(NlFlag::Recoverable);
                }
            }

            let mut converged = false;
            let mut recoverable = false;
            let mut m = 0usize;
            loop {
                // rhs = -F
                delta.iter_mut().for_each(|v| *v = -*v);
                if !self.lsolve(&mut delta) {
                    recoverable = true;
                    break;
                }
                for i in 0..self.n {
                    self.ee[i] += delta[i];
                    self.yy[i] = self.yypredict[i] + self.ee[i];
                    self.yp[i] = self.yppredict[i] + self.cj * self.ee[i];
                }
                // `IDANlsConvTest`
                let delnrm = self.wrms(&delta);
                if m == 0 {
                    self.oldnrm = delnrm;
                    if delnrm <= self.toldel {
                        converged = true;
                        break;
                    }
                } else {
                    let rate = (delnrm / self.oldnrm).powf(1.0 / m as f64);
                    if rate > RATEMAX {
                        recoverable = true;
                        break;
                    }
                    self.ss = rate / (1.0 - rate);
                }
                if self.ss * delnrm <= self.eps_newt {
                    converged = true;
                    break;
                }
                m += 1;
                if m >= MAXCOR {
                    recoverable = true;
                    break;
                }
                if self
                    .res
                    .eval(self.tn, &self.yy, &self.yp, &mut delta)
                    .is_err()
                {
                    recoverable = true;
                    break;
                }
                self.savres.copy_from_slice(&delta);
            }

            if converged {
                self.jcur = false;
                return Ok(NlFlag::Success);
            }
            // A recoverable failure with a stale Jacobian: refresh it and retry
            // once, exactly as SUNNonlinSol_Newton's outer loop does.
            if recoverable && !self.jcur && jbad_retry {
                jbad_retry = false;
                call_lsetup = true;
                continue;
            }
            return Ok(NlFlag::Recoverable);
        }
    }

    /// `IDATestError`: the local error test, and the tentative order for the
    /// next step.
    fn test_error(&mut self, ck: f64) -> (NlFlag, f64, f64) {
        let enorm_k = self.wrms(&self.ee);
        let err_k = self.sigma[self.kk] * enorm_k;
        let terr_k = (self.kk + 1) as f64 * err_k;
        self.knew = self.kk;
        let mut err_km1 = 0.0;
        if self.kk > 1 {
            let mut delta = vec![0.0; self.n];
            for i in 0..self.n {
                delta[i] = self.phi[self.kk][i] + self.ee[i];
            }
            let enorm_km1 = self.wrms(&delta);
            err_km1 = self.sigma[self.kk - 1] * enorm_km1;
            let terr_km1 = self.kk as f64 * err_km1;
            if self.kk > 2 {
                for i in 0..self.n {
                    delta[i] += self.phi[self.kk - 1][i];
                }
                let enorm_km2 = self.wrms(&delta);
                let err_km2 = self.sigma[self.kk - 2] * enorm_km2;
                let terr_km2 = (self.kk - 1) as f64 * err_km2;
                if terr_km1.max(terr_km2) <= terr_k {
                    self.knew = self.kk - 1;
                }
            } else if terr_km1 <= 0.5 * terr_k {
                self.knew = self.kk - 1;
            }
        }
        let flag = if ck * enorm_k > 1.0 {
            NlFlag::ErrorTest
        } else {
            NlFlag::Success
        };
        (flag, err_k, err_km1)
    }

    /// `IDAHandleNFlag`. Returns `true` to predict again, `false` to give up.
    fn handle_nflag(
        &mut self,
        flag: NlFlag,
        err_k: f64,
        err_km1: f64,
        ncf: &mut usize,
        nef: &mut usize,
    ) -> bool {
        self.phase = 1;
        if flag != NlFlag::ErrorTest {
            *ncf += 1;
            if *ncf >= MAXNCF {
                return false;
            }
            self.rr = 0.25;
            self.hh *= self.rr;
            return true;
        }
        *nef += 1;
        if *nef == 1 {
            let err_knew = if self.kk == self.knew { err_k } else { err_km1 };
            self.kk = self.knew;
            self.rr = 0.9 * (2.0 * err_knew + 1e-4).powf(-1.0 / (self.kk + 1) as f64);
            self.rr = self.rr.clamp(0.25, 0.9);
            self.hh *= self.rr;
            true
        } else if *nef == 2 {
            self.kk = self.knew;
            self.rr = 0.25;
            self.hh *= self.rr;
            true
        } else if *nef < MAXNEF {
            self.kk = 1;
            self.rr = 0.25;
            self.hh *= self.rr;
            true
        } else {
            false
        }
    }

    /// `IDACompleteStep`: commit the step, choose the next order and step size,
    /// and roll the `phi` history forward.
    fn complete_step(&mut self, err_k: f64, err_km1: f64) {
        self.nst += 1;
        let kdiff = self.kk as i64 - self.kused as i64;
        self.kused = self.kk;
        self.hused = self.hh;

        if self.knew + 1 == self.kk || self.kk == MAXORD {
            self.phase = 1;
        }

        if self.phase == 0 {
            if self.nst > 1 {
                self.kk += 1;
                let mut hnew = 2.0 * self.hh;
                let tmp = hnew.abs() * self.hmax_inv;
                if tmp > 1.0 {
                    hnew /= tmp;
                }
                self.hh = hnew;
            }
        } else {
            // LOWER = -1, MAINTAIN = 0, RAISE = 1
            let action;
            let mut err_knew = err_k;
            if self.knew + 1 == self.kk {
                action = -1;
            } else if self.kk == MAXORD {
                action = 0;
            } else if self.kk + 1 >= self.ns || kdiff == 1 {
                action = 0;
            } else {
                let mut tempv = vec![0.0; self.n];
                for i in 0..self.n {
                    tempv[i] = self.ee[i] - self.phi[self.kk + 1][i];
                }
                let enorm = self.wrms(&tempv);
                let err_kp1 = enorm / (self.kk + 2) as f64;
                let terr_k = (self.kk + 1) as f64 * err_k;
                let terr_kp1 = (self.kk + 2) as f64 * err_kp1;
                if self.kk == 1 {
                    if terr_kp1 >= 0.5 * terr_k {
                        action = 0;
                    } else {
                        action = 1;
                        err_knew = err_kp1;
                    }
                } else {
                    let terr_km1 = self.kk as f64 * err_km1;
                    if terr_km1 <= terr_k.min(terr_kp1) {
                        action = -1;
                    } else if terr_kp1 >= terr_k {
                        action = 0;
                    } else {
                        action = 1;
                        err_knew = err_kp1;
                    }
                }
            }
            match action {
                1 => self.kk += 1,
                -1 => {
                    self.kk -= 1;
                    err_knew = err_km1;
                }
                _ => {}
            }
            self.rr = (2.0 * err_knew + 1e-4).powf(-1.0 / (self.kk + 1) as f64);
            if self.rr >= 2.0 {
                let mut hnew = 2.0 * self.hh;
                let tmp = hnew.abs() * self.hmax_inv;
                if tmp > 1.0 {
                    hnew /= tmp;
                }
                self.hh = hnew;
            } else if self.rr <= 1.0 {
                // `SUNMAX(HALF, SUNMIN(PT9, rr))`. `clamp` — not `.min().max()`
                // — is the faithful form: both it and the C macros propagate a
                // NaN `rr`, whereas `f64::min`/`f64::max` would silently swallow
                // it and return 0.9.
                self.rr = self.rr.clamp(0.5, 0.9);
                self.hh *= self.rr;
            }
        }

        for i in 0..self.n {
            self.yy[i] = self.yypredict[i] + self.ee[i];
            self.yp[i] = self.yppredict[i] + self.cj * self.ee[i];
        }

        // `phi[kused+1] = ee` is the column that lets the NEXT step estimate the
        // order-(k+1) error (`ee - phi[kk+1]`) and be raised into it. Omitting
        // it does not look wrong — the integrator still runs — it just never
        // raises the order and grinds the step size down to nothing.
        if self.kused < MAXORD {
            let ee = self.ee.clone();
            self.phi[self.kused + 1].copy_from_slice(&ee);
        }
        // phi[kused] += ee; then phi[j] += phi[j+1] downward.
        for i in 0..self.n {
            self.phi[self.kused][i] += self.ee[i];
        }
        for j in (0..self.kused).rev() {
            for i in 0..self.n {
                self.phi[j][i] += self.phi[j + 1][i];
            }
        }
    }

    /// `IDAStep`: one internal step with its retry ladder.
    fn ida_step(&mut self) -> Result<()> {
        let saved_t = self.tn;
        let mut ncf = 0usize;
        let mut nef = 0usize;
        if self.nst == 0 {
            self.kk = 1;
            self.kused = 0;
            self.hused = 0.0;
            self.psi[0] = self.hh;
            self.cj = 1.0 / self.hh;
            self.phase = 0;
            self.ns = 0;
        }
        loop {
            let ck = self.set_coeffs();
            self.predict();
            let mut flag = self.nls()?;
            let mut err_k = 0.0;
            let mut err_km1 = 0.0;
            if flag == NlFlag::Success {
                let (f, a, b) = self.test_error(ck);
                flag = f;
                err_k = a;
                err_km1 = b;
            }
            if flag != NlFlag::Success {
                self.restore(saved_t);
                if !self.handle_nflag(flag, err_k, err_km1, &mut ncf, &mut nef) {
                    return Err(FreesError::solver(format!(
                        "the DAE integrator could not take a step at t = {} \
                         (h = {:e}, order {}): {}",
                        self.tn,
                        self.hh,
                        self.kk,
                        if flag == NlFlag::ErrorTest {
                            "the local error test kept failing"
                        } else {
                            "the corrector would not converge"
                        }
                    )));
                }
                if self.nst == 0 {
                    // `IDAReset`
                    self.psi[0] = self.hh;
                    let rr = self.rr;
                    self.phi[1].iter_mut().for_each(|v| *v *= rr);
                }
                continue;
            }
            self.complete_step(err_k, err_km1);
            return Ok(());
        }
    }

    /// `IDAGetSolution`: interpolate `(y, y')` at `t` from the `phi` history.
    fn get_solution(&self, t: f64) -> (Vec<f64>, Vec<f64>) {
        let kord = if self.kused == 0 { 1 } else { self.kused };
        let delt = t - self.tn;
        let mut c = 1.0;
        let mut d = 0.0;
        let mut gam = delt / self.psi[0];
        let mut cvals = [0.0f64; MXORDP1];
        let mut dvals = [0.0f64; MXORDP1];
        cvals[0] = c;
        for j in 1..=kord {
            d = d * gam + c / self.psi[j - 1];
            c *= gam;
            gam = (delt + self.psi[j - 1]) / self.psi[j];
            cvals[j] = c;
            dvals[j - 1] = d;
        }
        let mut y = vec![0.0; self.n];
        let mut yp = vec![0.0; self.n];
        for i in 0..self.n {
            let mut sy = 0.0;
            for j in 0..=kord {
                sy += cvals[j] * self.phi[j][i];
            }
            y[i] = sy;
            let mut sp = 0.0;
            for j in 0..kord {
                sp += dvals[j] * self.phi[j + 1][i];
            }
            yp[i] = sp;
        }
        (y, yp)
    }

    // ── the public step ──────────────────────────────────────────────────────

    /// Integrates to `tout` in `IDA_NORMAL` mode and returns the state (and any
    /// roots).
    pub fn step(&mut self, tout: f64) -> Result<Step> {
        self.require_init()?;
        let root_fn_present = self.nroots > 0 && self.root_fn.is_some();

        if self.nst > 0 {
            if root_fn_present {
                let troundoff = 100.0 * UROUND * (self.tn.abs() + self.hh.abs());
                if self.rcheck2()? {
                    self.irfnd = true;
                    return Ok(self.root_step());
                }
                if (self.tn - self.tretlast).abs() > troundoff {
                    if self.rcheck3(tout)? {
                        self.irfnd = true;
                        return Ok(self.root_step());
                    }
                    self.irfnd = false;
                }
            }
            if (self.tn - tout) * self.hh >= 0.0 {
                return Ok(self.solution_step(tout));
            }
        } else {
            self.first_call_setup(tout)?;
            if root_fn_present {
                self.rcheck1()?;
            }
        }

        let mut nstloc = 0u64;
        loop {
            if self.max_steps > 0 && nstloc >= self.max_steps {
                return Err(FreesError::solver(format!(
                    "the DAE integrator took {} steps without reaching t = {tout}; \
                     the model is either much stiffer than the tolerances admit or \
                     has no solution on this interval",
                    self.max_steps
                )));
            }
            // The boundary-installed wall-clock budget (Wave C1) — the same
            // check `ode/integrator.rs::guard` makes on the explicit path.
            // Native callers install nothing and can never strike here.
            if let Some(message) = crate::ode::deadline::strike() {
                return Err(FreesError::solver(message));
            }
            if self.nst > 0 {
                let phi0 = self.phi[0].clone();
                self.set_ewt(&phi0)?;
            }
            if self.tn + self.hh == self.tn {
                return Err(FreesError::solver(format!(
                    "the DAE integrator's step size underflowed at t = {} \
                     (h = {:e}); the residual is probably discontinuous there",
                    self.tn, self.hh
                )));
            }
            self.ida_step()?;
            nstloc += 1;

            if root_fn_present && self.rcheck3(tout)? {
                self.irfnd = true;
                return Ok(self.root_step());
            }
            if (self.tn - tout) * self.hh >= 0.0 {
                return Ok(self.solution_step(tout));
            }
        }
    }

    /// The first-call block of `IDASolve`: the initial step size, the
    /// convergence constants, and the `phi[1] ← h·y'` scaling.
    fn first_call_setup(&mut self, tout: f64) -> Result<()> {
        let tdist = (tout - self.tn).abs();
        if tdist == 0.0 {
            return Err(FreesError::solver(
                "the end time equals the start time; a transient needs a non-empty interval",
            ));
        }
        let troundoff = 2.0 * UROUND * (self.tn.abs() + tout.abs());
        if tdist < troundoff {
            return Err(FreesError::solver(
                "the end time is within roundoff of the start time",
            ));
        }
        self.hh = 0.001 * tdist;
        let yp1 = self.phi[1].clone();
        let ypnorm = self.wrms(&yp1);
        if ypnorm > 0.5 / self.hh {
            self.hh = 0.5 / ypnorm;
        }
        if tout < self.tn {
            self.hh = -self.hh;
        }
        let rh = self.hh.abs() * self.hmax_inv;
        if rh > 1.0 {
            self.hh /= rh;
        }
        self.tretlast = self.tn;
        self.eps_newt = EPCON;
        self.toldel = 1e-4 * self.eps_newt;
        let h = self.hh;
        self.phi[1].iter_mut().for_each(|v| *v *= h);
        Ok(())
    }

    /// The state to return at a requested output time. `yy`/`yp` are refreshed
    /// so [`Self::current_state`] agrees with what was handed out, matching the
    /// Java binding (IDA writes `yret`/`ypret` into the same vectors).
    fn solution_step(&mut self, tout: f64) -> Step {
        let (y, yp) = self.get_solution(tout);
        self.tretlast = tout;
        self.yy.copy_from_slice(&y);
        self.yp.copy_from_slice(&yp);
        Step {
            t: tout,
            y,
            yp,
            flag: IDA_SUCCESS,
            roots_found: vec![0; self.nroots],
        }
    }

    /// The state to return when a root fired.
    fn root_step(&mut self) -> Step {
        let (y, yp) = self.get_solution(self.trout);
        self.tretlast = self.trout;
        self.yy.copy_from_slice(&y);
        self.yp.copy_from_slice(&yp);
        Step {
            t: self.trout,
            y,
            yp,
            flag: IDA_ROOT_RETURN,
            roots_found: self.iroots.clone(),
        }
    }

    // ── root finding ─────────────────────────────────────────────────────────

    fn gfun(&self, t: f64, y: &[f64], yp: &[f64], gout: &mut [f64]) -> Result<()> {
        match self.root_fn {
            Some(f) => f.eval(t, y, yp, gout),
            None => Ok(()),
        }
    }

    /// `IDARcheck1`: check for zeros of `g` at and near `t0`.
    ///
    /// IDA passes `phi[1]` — which at this point is `h·y'`, not `y'` — as the
    /// derivative argument. The quirk is reproduced deliberately: frees'
    /// switching functions are `lhs − rhs` over the reified `der$X`, so a root
    /// that reads a derivative would see the same scaled value in both engines.
    fn rcheck1(&mut self) -> Result<()> {
        self.iroots.iter_mut().for_each(|v| *v = 0);
        self.tlo = self.tn;
        self.ttol = (self.tn.abs() + self.hh.abs()) * UROUND * 100.0;
        let (y, yp) = (self.phi[0].clone(), self.phi[1].clone());
        let mut glo = vec![0.0; self.nroots];
        self.gfun(self.tlo, &y, &yp, &mut glo)?;
        self.glo = glo;
        let mut zroot = false;
        for i in 0..self.nroots {
            if self.glo[i] == 0.0 {
                zroot = true;
                self.gactive[i] = false;
            }
        }
        if !zroot {
            return Ok(());
        }
        let hratio = (self.ttol / self.hh.abs()).max(0.1);
        let smallh = hratio * self.hh;
        let tplus = self.tlo + smallh;
        let ytmp: Vec<f64> = (0..self.n).map(|i| y[i] + smallh * yp[i]).collect();
        let mut ghi = vec![0.0; self.nroots];
        self.gfun(tplus, &ytmp, &yp, &mut ghi)?;
        for i in 0..self.nroots {
            if !self.gactive[i] && ghi[i] != 0.0 {
                self.gactive[i] = true;
                self.glo[i] = ghi[i];
            }
        }
        self.ghi = ghi;
        Ok(())
    }

    /// `IDARcheck2`: a root at `tlo` on re-entry after a root return.
    fn rcheck2(&mut self) -> Result<bool> {
        if !self.irfnd {
            return Ok(false);
        }
        let (y, yp) = self.get_solution(self.tlo);
        let mut glo = vec![0.0; self.nroots];
        self.gfun(self.tlo, &y, &yp, &mut glo)?;
        self.glo = glo;
        self.iroots.iter_mut().for_each(|v| *v = 0);
        let mut zroot = false;
        for i in 0..self.nroots {
            if self.gactive[i] && self.glo[i] == 0.0 {
                zroot = true;
                self.iroots[i] = 1;
            }
        }
        if !zroot {
            return Ok(false);
        }
        self.ttol = (self.tn.abs() + self.hh.abs()) * UROUND * 100.0;
        let smallh = if self.hh > 0.0 { self.ttol } else { -self.ttol };
        let tplus = self.tlo + smallh;
        let (ytmp, yptmp) = if (tplus - self.tn) * self.hh >= 0.0 {
            let hratio = smallh / self.hh;
            (
                (0..self.n)
                    .map(|i| y[i] + hratio * self.phi[1][i])
                    .collect::<Vec<f64>>(),
                yp.clone(),
            )
        } else {
            self.get_solution(tplus)
        };
        let mut ghi = vec![0.0; self.nroots];
        self.gfun(tplus, &ytmp, &yptmp, &mut ghi)?;
        let mut zroot2 = false;
        for i in 0..self.nroots {
            if !self.gactive[i] {
                continue;
            }
            if ghi[i] == 0.0 {
                if self.iroots[i] == 1 {
                    return Err(FreesError::solver(
                        "two roots of the same event are closer together than the \
                         integrator can separate; widen the switching condition",
                    ));
                }
                zroot2 = true;
                self.iroots[i] = 1;
            } else if self.iroots[i] == 1 {
                self.glo[i] = ghi[i];
            }
        }
        if zroot2 {
            self.trout = self.tlo;
            return Ok(true);
        }
        Ok(false)
    }

    /// `IDARcheck3`: search `(tlo, thi]` for a root, where `thi` is `min(tn,
    /// tout)` in the direction of integration.
    fn rcheck3(&mut self, tout: f64) -> Result<bool> {
        self.thi = if (tout - self.tn) * self.hh >= 0.0 {
            self.tn
        } else {
            tout
        };
        let (y, yp) = self.get_solution(self.thi);
        let mut ghi = vec![0.0; self.nroots];
        self.gfun(self.thi, &y, &yp, &mut ghi)?;
        self.ghi = ghi;
        self.ttol = (self.tn.abs() + self.hh.abs()) * UROUND * 100.0;
        let found = self.rootfind()?;
        for i in 0..self.nroots {
            if !self.gactive[i] && self.grout[i] != 0.0 {
                self.gactive[i] = true;
            }
        }
        self.tlo = self.trout;
        self.glo = self.grout.clone();
        Ok(found)
    }

    /// `IDARootfind`: the modified-Illinois search for the nearest root.
    fn rootfind(&mut self) -> Result<bool> {
        let mut imax = 0usize;
        let (mut zroot, mut sgnchg) = self.scan_ghi(&mut imax);

        if !sgnchg {
            self.trout = self.thi;
            self.grout = self.ghi.clone();
            if !zroot {
                return Ok(false);
            }
            for i in 0..self.nroots {
                self.iroots[i] = 0;
                if self.gactive[i]
                    && self.ghi[i] == 0.0
                    && self.rootdir[i] as f64 * self.glo[i] <= 0.0
                {
                    self.iroots[i] = if self.glo[i] > 0.0 { -1 } else { 1 };
                }
            }
            return Ok(true);
        }

        let mut alph = 1.0f64;
        let mut side = 0i32;
        let mut sideprev = -1i32;
        loop {
            if (self.thi - self.tlo).abs() <= self.ttol {
                break;
            }
            alph = if sideprev == side {
                if side == 2 {
                    alph * 2.0
                } else {
                    alph * 0.5
                }
            } else {
                1.0
            };
            let mut tmid = self.thi
                - (self.thi - self.tlo) * self.ghi[imax] / (self.ghi[imax] - alph * self.glo[imax]);
            if (tmid - self.tlo).abs() < 0.5 * self.ttol {
                let fracint = (self.thi - self.tlo).abs() / self.ttol;
                let fracsub = if fracint > 5.0 { 0.1 } else { 0.5 / fracint };
                tmid = self.tlo + fracsub * (self.thi - self.tlo);
            }
            if (self.thi - tmid).abs() < 0.5 * self.ttol {
                let fracint = (self.thi - self.tlo).abs() / self.ttol;
                let fracsub = if fracint > 5.0 { 0.1 } else { 0.5 / fracint };
                tmid = self.thi - fracsub * (self.thi - self.tlo);
            }
            let (y, yp) = self.get_solution(tmid);
            let mut grout = vec![0.0; self.nroots];
            self.gfun(tmid, &y, &yp, &mut grout)?;
            self.grout = grout;

            sideprev = side;
            let (z, s) = self.scan_grout(&mut imax);
            zroot = z;
            sgnchg = s;
            if sgnchg {
                self.thi = tmid;
                self.ghi = self.grout.clone();
                side = 1;
                if (self.thi - self.tlo).abs() <= self.ttol {
                    break;
                }
                continue;
            }
            if zroot {
                self.thi = tmid;
                self.ghi = self.grout.clone();
                break;
            }
            self.tlo = tmid;
            self.glo = self.grout.clone();
            side = 2;
            if (self.thi - self.tlo).abs() <= self.ttol {
                break;
            }
        }

        self.trout = self.thi;
        for i in 0..self.nroots {
            self.grout[i] = self.ghi[i];
            self.iroots[i] = 0;
            if !self.gactive[i] {
                continue;
            }
            let dir_ok = self.rootdir[i] as f64 * self.glo[i] <= 0.0;
            if dir_ok && (self.ghi[i] == 0.0 || self.glo[i] * self.ghi[i] < 0.0) {
                self.iroots[i] = if self.glo[i] > 0.0 { -1 } else { 1 };
            }
        }
        Ok(true)
    }

    fn scan_ghi(&self, imax: &mut usize) -> (bool, bool) {
        Self::scan(&self.glo, &self.ghi, &self.gactive, &self.rootdir, imax)
    }

    fn scan_grout(&self, imax: &mut usize) -> (bool, bool) {
        Self::scan(&self.glo, &self.grout, &self.gactive, &self.rootdir, imax)
    }

    /// The sign-change scan shared by `IDARootfind`'s two passes: returns
    /// `(a component is exactly zero, a component changed sign)` and sets
    /// `imax` to the component whose bracketed fraction is largest.
    fn scan(
        glo: &[f64],
        ghi: &[f64],
        gactive: &[bool],
        rootdir: &[i32],
        imax: &mut usize,
    ) -> (bool, bool) {
        let mut maxfrac = 0.0;
        let mut zroot = false;
        let mut sgnchg = false;
        for i in 0..glo.len() {
            if !gactive[i] {
                continue;
            }
            if ghi[i] == 0.0 {
                if rootdir[i] as f64 * glo[i] <= 0.0 {
                    zroot = true;
                }
            } else if glo[i] * ghi[i] < 0.0 && rootdir[i] as f64 * glo[i] <= 0.0 {
                let gfrac = (ghi[i] / (ghi[i] - glo[i])).abs();
                if gfrac > maxfrac {
                    sgnchg = true;
                    maxfrac = gfrac;
                    *imax = i;
                }
            }
        }
        (zroot, sgnchg)
    }

    // ── consistent initialization ────────────────────────────────────────────

    /// `IDACalcIC`: correct `(y, y')` so the algebraic constraints hold at
    /// `t0`, then copy the result back into the history.
    ///
    /// `icopt` is [`IDA_YA_YDP_INIT`] (needs [`IdaDaeSolver::set_variable_id`])
    /// or [`IDA_Y_INIT`]. `tout1` is a point in the direction of integration —
    /// only its position relative to `t0` matters.
    pub fn calc_consistent_ic(&mut self, icopt: i32, tout1: f64) -> Result<()> {
        self.require_init()?;
        if icopt != IDA_YA_YDP_INIT && icopt != IDA_Y_INIT {
            return Err(FreesError::solver("unknown consistent-IC option"));
        }
        if icopt == IDA_YA_YDP_INIT && self.variable_id.is_none() {
            return Err(FreesError::solver(
                "consistent initialization of the algebraic components needs the \
                 differential/algebraic marker (set_variable_id)",
            ));
        }
        let tdist = (tout1 - self.tn).abs();
        let troundoff = 2.0 * UROUND * (self.tn.abs() + tout1.abs());
        if tdist < troundoff {
            return Err(FreesError::solver(
                "the consistent-IC probe time is within roundoff of t0",
            ));
        }

        let mut yy0 = self.phi[0].clone();
        let mut yp0 = self.phi[1].clone();
        let t0 = self.tn;

        // `sysindex` 0 means "no algebraic components", which rescales the IC
        // convergence norm by `tscale·|cj|`.
        let mut sysindex = 1;
        let tscale = tdist;
        if icopt == IDA_YA_YDP_INIT {
            let id = self.variable_id.as_ref().unwrap();
            let minid = id.iter().copied().fold(f64::INFINITY, f64::min);
            if minid < 0.0 {
                return Err(FreesError::solver(
                    "the differential/algebraic marker has a negative component",
                ));
            }
            if minid > 0.5 {
                sysindex = 0;
            }
        }

        let mut hic = 0.001 * tdist;
        let ypnorm = self.wrms(&yp0);
        if ypnorm > 0.5 / hic {
            hic = 0.5 / ypnorm;
        }
        if tout1 < self.tn {
            hic = -hic;
        }
        self.hh = hic;
        let mxnh = if icopt == IDA_YA_YDP_INIT {
            self.cj = 1.0 / hic;
            MAXNH
        } else {
            self.cj = 0.0;
            1
        };

        let mut last: Result<()> = Ok(());
        for _nwt in 0..2 {
            let mut ok = false;
            for nh in 0..mxnh {
                match self.nls_ic(icopt, sysindex, tscale, t0, &mut yy0, &mut yp0) {
                    Ok(()) => {
                        ok = true;
                        break;
                    }
                    Err(e) => {
                        last = Err(e);
                        if nh + 1 == mxnh {
                            break;
                        }
                        yy0.copy_from_slice(&self.phi[0]);
                        yp0.copy_from_slice(&self.phi[1]);
                        hic *= 0.1;
                        self.cj = 1.0 / hic;
                        self.hh = hic;
                    }
                }
            }
            if !ok {
                return last;
            }
            self.set_ewt(&yy0)?;
            self.phi[0].copy_from_slice(&yy0);
            self.phi[1].copy_from_slice(&yp0);
            self.yy.copy_from_slice(&yy0);
            self.yp.copy_from_slice(&yp0);
        }
        // The Jacobian was built at the IC's `cj`; force a fresh one for the
        // first real step.
        self.jac = None;
        self.jcur = false;
        self.force_setup = true;
        Ok(())
    }

    /// `IDANlsIC` + `IDANewtonIC` + `IDALineSrch`: the damped Newton that makes
    /// the initial condition consistent.
    fn nls_ic(
        &mut self,
        icopt: i32,
        sysindex: i32,
        tscale: f64,
        t0: f64,
        yy0: &mut Vec<f64>,
        yp0: &mut Vec<f64>,
    ) -> Result<()> {
        let steptol = UROUND.powf(2.0 / 3.0);
        let mut delta = vec![0.0; self.n];

        for _nj in 0..MAXNJ {
            // The residual has to be re-evaluated at the CURRENT (yy0, yp0)
            // before every Jacobian setup. On the retry pass `delta` holds the
            // previous Newton *step*, not a residual, and a difference-quotient
            // Jacobian built against that base is silently garbage — the
            // symptom is a second pass whose line search stalls immediately
            // because its "Newton direction" is not a descent direction.
            self.res.eval(t0, yy0, yp0, &mut delta)?;
            let f0 = delta.clone();
            let (y, yp) = (yy0.clone(), yp0.clone());
            let saved_tn = self.tn;
            self.tn = t0;
            let ok = self.lsetup(&y, &yp, &f0)?;
            self.tn = saved_tn;
            if !ok {
                return Err(FreesError::solver(
                    "the initial-condition Jacobian is singular; the algebraic \
                     constraints do not determine the algebraic unknowns",
                ));
            }
            match self.newton_ic(icopt, sysindex, tscale, t0, yy0, yp0, &mut delta, steptol) {
                Ok(()) => return Ok(()),
                Err(SlowOrFail::Slow) => continue,
                Err(SlowOrFail::Fail(e)) => return Err(e),
            }
        }
        Err(FreesError::solver(
            "the initial condition could not be made consistent: the algebraic \
             constraints are not satisfied at t0 and Newton did not converge",
        ))
    }

    #[allow(clippy::too_many_arguments)]
    fn newton_ic(
        &mut self,
        icopt: i32,
        sysindex: i32,
        tscale: f64,
        t0: f64,
        yy0: &mut Vec<f64>,
        yp0: &mut Vec<f64>,
        delta: &mut Vec<f64>,
        steptol: f64,
    ) -> std::result::Result<(), SlowOrFail> {
        let scale = |v: f64, s: &Self| {
            if sysindex == 0 {
                v * tscale * s.cj.abs()
            } else {
                v
            }
        };
        if !self.lsolve(delta) {
            return Err(SlowOrFail::Fail(FreesError::solver(
                "the initial-condition linear solve failed",
            )));
        }
        let mut fnorm = scale(self.wrms(delta), self);
        if fnorm <= self.eps_newt {
            return Ok(());
        }

        for mnewt in 0..MAXNIT {
            // `IDALineSrch`
            let f1norm = 0.5 * fnorm * fnorm;
            let slpi = -2.0 * f1norm;
            let delnorm = self.wrms(delta);
            let minlam = steptol / delnorm;
            let mut lambda = 1.0f64;
            let mut delnew = vec![0.0; self.n];
            loop {
                if lambda < minlam {
                    return Err(SlowOrFail::Fail(FreesError::solver(
                        "the initial-condition line search stalled; the state at t0 \
                         is too far from any consistent point",
                    )));
                }
                let (ynew, ypnew) = self.new_yyp(icopt, yy0, yp0, delta, lambda);
                if self.res.eval(t0, &ynew, &ypnew, &mut delnew).is_err() {
                    lambda /= 2.0;
                    continue;
                }
                if !self.lsolve(&mut delnew) {
                    return Err(SlowOrFail::Fail(FreesError::solver(
                        "the initial-condition linear solve failed",
                    )));
                }
                let fnormp = scale(self.wrms(&delnew), self);
                let f1normp = 0.5 * fnormp * fnormp;
                if f1normp <= f1norm + ALPHALS * slpi * lambda {
                    *yy0 = ynew;
                    *yp0 = ypnew;
                    fnorm = fnormp;
                    break;
                }
                lambda /= 2.0;
            }

            if fnorm <= self.eps_newt {
                return Ok(());
            }
            // (`IDANewtonIC` also forms `rate = fnorm/oldfnrm` here and then
            // never reads it; the dead statement is not carried over.)
            if mnewt + 1 == MAXNIT {
                return Err(SlowOrFail::Slow);
            }
            delta.copy_from_slice(&delnew);
        }
        Err(SlowOrFail::Fail(FreesError::solver(
            "the initial condition could not be made consistent",
        )))
    }

    /// `IDANewyyp`: apply the damped step to the components the option allows
    /// to move.
    fn new_yyp(
        &self,
        icopt: i32,
        yy0: &[f64],
        yp0: &[f64],
        delta: &[f64],
        lambda: f64,
    ) -> (Vec<f64>, Vec<f64>) {
        let mut ynew = yy0.to_vec();
        let mut ypnew = yp0.to_vec();
        if icopt == IDA_YA_YDP_INIT {
            let id = self
                .variable_id
                .as_ref()
                .expect("checked by calc_consistent_ic");
            for i in 0..self.n {
                let d_diff = id[i] * delta[i];
                ypnew[i] = yp0[i] - self.cj * lambda * d_diff;
                ynew[i] = yy0[i] - lambda * (delta[i] - d_diff);
            }
        } else {
            for i in 0..self.n {
                ynew[i] = yy0[i] - lambda * delta[i];
            }
        }
        (ynew, ypnew)
    }
}

/// Distinguishes `IDA_SLOW_CONVRG` (retry with a fresh Jacobian) from a hard
/// failure inside the consistent-IC Newton.
enum SlowOrFail {
    Slow,
    Fail(FreesError),
}

#[cfg(test)]
#[path = "solver_tests.rs"]
mod tests;
