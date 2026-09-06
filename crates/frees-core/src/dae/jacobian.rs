//! Finite-difference assembly of the combined DAE system matrix
//! `J = ∂F/∂y + cj · ∂F/∂y'`.
//!
//! Port of `../frEES/backend/core/src/main/java/com/frees/backend/core/dae/DaeJacobian.java`
//! (146 lines), whole.
//!
//! # The key identity
//!
//! Perturbing state `y[c]` by `ε` **and** `y'[c]` by `cj·ε` together makes the
//! residual change by `(∂F/∂y_c + cj·∂F/∂y'_c)·ε`, so a single forward
//! difference yields the whole combined column `c` — no separate `∂F/∂y` and
//! `∂F/∂y'` sweeps. That is the column the sparse pattern
//! ([`crate::dae::assembly::build_sparsity`], one column per `y` variable with
//! `der$X` folded onto state `X`'s column) lays out.
//!
//! # Two increment rules, deliberately
//!
//! [`perturbation`] is the frees rule (`1e-7·max(|v|, 1)`) that the Java feeds
//! IDA through `IDASetJacFn` on the **sparse** path. IDA's own dense
//! difference-quotient Jacobian uses a different increment, and the Java engine
//! leaves that path to IDA — so the port keeps both:
//! [`crate::dae::solver`] uses [`ida_dense_increment`] when it is running the
//! dense linear solver and this module's [`dense_colored`] when it is running
//! the sparse one, matching what each configuration of the oracle actually
//! does. The Jacobian only steers Newton's convergence rate, never the root it
//! converges to, but reproducing the oracle's iterate sequence is what keeps
//! the step/order history — and therefore the trajectory — identical.

// Transcription of `DaeJacobian.java`: the sweeps index `j[row][c]`, `y[c]`,
// `yp[c]` and `eps[c]` in lockstep against a *pattern* (`col_rows[c]`), so an
// iterator rewrite would obscure which array the pattern applies to. The Java
// index form stays, exactly as it does in `crate::eval`.
#![allow(clippy::needless_range_loop)]

use crate::dae::assembly::DaeResidual;
use crate::diag::Result;

/// The frees finite-difference increment: `1e-7 · max(|v|, 1)`.
///
/// Transcribed from `DaeJacobian.perturbation`; the constant stays as written
/// (parity rule) rather than being re-derived from `f64::EPSILON`.
pub fn perturbation(v: f64) -> f64 {
    1e-7 * v.abs().max(1.0)
}

/// IDA's own dense difference-quotient increment (`idaLsDenseDQJac`):
/// `inc = max(√u · max(|y_j|, |h·y'_j|), 1/ewt_j)`, signed to follow `h·y'_j`,
/// then snapped to a representable difference by `(y_j + inc) - y_j`.
///
/// `ewt_j` is the error weight `1/(rtol·|y_j| + atol)`, so `1/ewt_j` is the
/// absolute tolerance floor for that component.
pub fn ida_dense_increment(y_j: f64, yp_j: f64, h: f64, inv_ewt_j: f64) -> f64 {
    let srur = f64::EPSILON.sqrt();
    let mut inc = (srur * y_j.abs().max((h * yp_j).abs())).max(inv_ewt_j);
    if h * yp_j < 0.0 {
        inc = -inc;
    }
    (y_j + inc) - y_j
}

/// Computes combined column `c` of `J` into `out` (length `n`) by one forward
/// difference, reusing the already-evaluated base residual `f0`. Returns the
/// `ε` used.
///
/// Port of `DaeJacobian.column`. The argument list is the Java signature
/// unchanged — `f0` and `out` are caller-owned buffers precisely so a Jacobian
/// sweep allocates once, which is why they are not bundled into a struct.
#[allow(clippy::too_many_arguments)]
pub fn column(
    res: &dyn DaeResidual,
    t: f64,
    cj: f64,
    y: &[f64],
    yp: &[f64],
    c: usize,
    f0: &[f64],
    out: &mut [f64],
) -> Result<f64> {
    let n = y.len();
    let eps = perturbation(y[c]);
    let mut y_pert = y.to_vec();
    let mut yp_pert = yp.to_vec();
    y_pert[c] += eps;
    yp_pert[c] += cj * eps;
    let mut fp = vec![0.0; n];
    res.eval(t, &y_pert, &yp_pert, &mut fp)?;
    for i in 0..n {
        out[i] = (fp[i] - f0[i]) / eps;
    }
    Ok(eps)
}

/// Distance-1 greedy colouring of the columns (Phase S2): two columns get
/// different colours iff they share a row (their structural supports overlap).
/// Columns of the *same* colour are structurally orthogonal, so they can be
/// perturbed together and recovered from a single residual evaluation — cutting
/// the FD Jacobian from `n` residual evaluations to `#colours` (≈ the bandwidth
/// for a banded C-R-C system).
///
/// Returns a 0-based colour per column; `sparsity_rows[i]` lists the columns
/// present in row `i`. Port of `DaeJacobian.colorColumns`, including its greedy
/// order (column 0 upward, lowest free colour wins), which is what makes the
/// result reproducible against the oracle rather than merely valid.
pub fn color_columns(sparsity_rows: &[Vec<usize>], n: usize) -> Vec<usize> {
    let mut adj: Vec<std::collections::BTreeSet<usize>> = vec![Default::default(); n];
    for row in sparsity_rows {
        for a in 0..row.len() {
            for b in (a + 1)..row.len() {
                adj[row[a]].insert(row[b]);
                adj[row[b]].insert(row[a]);
            }
        }
    }
    // `usize::MAX` is the Java `-1` sentinel for "not yet coloured".
    let mut color = vec![usize::MAX; n];
    for c in 0..n {
        let mut used = vec![false; n + 1];
        for &nb in &adj[c] {
            if color[nb] != usize::MAX {
                used[color[nb]] = true;
            }
        }
        let mut k = 0;
        while used[k] {
            k += 1;
        }
        color[c] = k;
    }
    color
}

/// Number of colours in a colouring (the count of residual evaluations
/// [`dense_colored`] will spend).
pub fn color_count(color: &[usize]) -> usize {
    color.iter().map(|&c| c + 1).max().unwrap_or(0)
}

/// Scratch buffers for finite-difference Jacobian evaluation.
/// Reused across calls to eliminate per-color and per-Jacobian allocations.
#[derive(Debug, Clone, Default)]
pub struct DaeJacobianScratch {
    pub f0: Vec<f64>,
    pub fp: Vec<f64>,
    pub y_pert: Vec<f64>,
    pub yp_pert: Vec<f64>,
    pub eps: Vec<f64>,
}

impl DaeJacobianScratch {
    pub fn new(n: usize) -> Self {
        Self {
            f0: vec![0.0; n],
            fp: vec![0.0; n],
            y_pert: vec![0.0; n],
            yp_pert: vec![0.0; n],
            eps: vec![0.0; n],
        }
    }

    pub fn ensure_capacity(&mut self, n: usize) {
        if self.f0.len() < n {
            self.f0.resize(n, 0.0);
            self.fp.resize(n, 0.0);
            self.y_pert.resize(n, 0.0);
            self.yp_pert.resize(n, 0.0);
            self.eps.resize(n, 0.0);
        }
    }
}

/// Coloured finite-difference combined Jacobian written directly into CSC storage.
///
/// Uses `#colours` residual evaluations instead of `n`, and directly fills the
/// CSC value buffer `out_values` corresponding to `(col_ptr, row_idx)` without
/// allocating any `n × n` dense intermediate.
///
/// If `f0` is provided (`Some(f0)`), it reuses the base residual already evaluated
/// by the caller. Reuses `scratch` buffers for `fp`, `y_pert`, `yp_pert`, `eps`.
#[allow(clippy::too_many_arguments)]
pub fn csc_colored_into(
    res: &dyn DaeResidual,
    t: f64,
    cj: f64,
    y: &[f64],
    yp: &[f64],
    f0: Option<&[f64]>,
    col_rows: &[Vec<usize>],
    color: &[usize],
    col_ptr: &[usize],
    out_values: &mut [f64],
    scratch: &mut DaeJacobianScratch,
) -> Result<()> {
    let n = y.len();
    scratch.ensure_capacity(n);

    let base_f0 = match f0 {
        Some(f) => f,
        None => {
            res.eval(t, y, yp, &mut scratch.f0[..n])?;
            &scratch.f0[..n]
        }
    };

    for c in 0..n {
        scratch.eps[c] = perturbation(y[c]);
    }

    scratch.y_pert[..n].copy_from_slice(y);
    scratch.yp_pert[..n].copy_from_slice(yp);

    let ncolors = color_count(color);
    for g in 0..ncolors {
        for c in 0..n {
            if color[c] == g {
                scratch.y_pert[c] += scratch.eps[c];
                scratch.yp_pert[c] += cj * scratch.eps[c];
            }
        }

        res.eval(
            t,
            &scratch.y_pert[..n],
            &scratch.yp_pert[..n],
            &mut scratch.fp[..n],
        )?;

        for c in 0..n {
            if color[c] == g {
                let start = col_ptr[c];
                for (offset, &row) in col_rows[c].iter().enumerate() {
                    out_values[start + offset] = (scratch.fp[row] - base_f0[row]) / scratch.eps[c];
                }
                scratch.y_pert[c] = y[c];
                scratch.yp_pert[c] = yp[c];
            }
        }
    }

    Ok(())
}

/// Coloured finite-difference combined Jacobian into a newly allocated CSC value buffer.
#[allow(clippy::too_many_arguments)]
pub fn csc_colored(
    res: &dyn DaeResidual,
    t: f64,
    cj: f64,
    y: &[f64],
    yp: &[f64],
    col_rows: &[Vec<usize>],
    color: &[usize],
    col_ptr: &[usize],
    nnz: usize,
) -> Result<Vec<f64>> {
    let mut values = vec![0.0; nnz];
    let mut scratch = DaeJacobianScratch::new(y.len());
    csc_colored_into(
        res,
        t,
        cj,
        y,
        yp,
        None,
        col_rows,
        color,
        col_ptr,
        &mut values,
        &mut scratch,
    )?;
    Ok(values)
}

/// Coloured finite-difference combined Jacobian, returned dense — identical (to
/// FD precision) to [`dense`] but using `#colours` residual evaluations instead
/// of `n`. `col_rows[c]` are the rows present in column `c`; `color` is from
/// [`color_columns`].
///
/// Port of `DaeJacobian.denseColored`. Entries outside the declared pattern stay
/// zero, exactly as in the Java: the pattern is a promise the assembler makes.
pub fn dense_colored(
    res: &dyn DaeResidual,
    t: f64,
    cj: f64,
    y: &[f64],
    yp: &[f64],
    col_rows: &[Vec<usize>],
    color: &[usize],
) -> Result<Vec<Vec<f64>>> {
    let n = y.len();
    let mut f0 = vec![0.0; n];
    res.eval(t, y, yp, &mut f0)?;
    let mut j = vec![vec![0.0; n]; n];
    let ncolors = color_count(color);
    let eps: Vec<f64> = (0..n).map(|c| perturbation(y[c])).collect();
    let mut fp = vec![0.0; n];
    for g in 0..ncolors {
        let mut y_pert = y.to_vec();
        let mut yp_pert = yp.to_vec();
        for c in 0..n {
            if color[c] == g {
                y_pert[c] += eps[c];
                yp_pert[c] += cj * eps[c];
            }
        }
        res.eval(t, &y_pert, &yp_pert, &mut fp)?;
        for c in 0..n {
            if color[c] == g {
                for &row in &col_rows[c] {
                    j[row][c] = (fp[row] - f0[row]) / eps[c];
                }
            }
        }
    }
    Ok(j)
}

/// Full dense combined Jacobian `J[i][j]` (for the dense solver and tests).
///
/// Port of `DaeJacobian.dense`.
pub fn dense(
    res: &dyn DaeResidual,
    t: f64,
    cj: f64,
    y: &[f64],
    yp: &[f64],
) -> Result<Vec<Vec<f64>>> {
    let n = y.len();
    let mut f0 = vec![0.0; n];
    res.eval(t, y, yp, &mut f0)?;
    let mut j = vec![vec![0.0; n]; n];
    let mut col = vec![0.0; n];
    for c in 0..n {
        column(res, t, cj, y, yp, c, &f0, &mut col)?;
        for i in 0..n {
            j[i][c] = col[i];
        }
    }
    Ok(j)
}

/// Transposes a per-row column pattern into the per-column row lists CSC needs.
///
/// Port of the transpose inside `IdaDaeSolver.setSparsity`. Row indices come out
/// ascending within each column, which is the order the CSC value buffer and
/// `SparseSteadyKlu`'s declared pattern both assume.
pub fn transpose_pattern(sparsity_rows: &[Vec<usize>], n: usize) -> Vec<Vec<usize>> {
    let mut cols: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (i, row) in sparsity_rows.iter().enumerate() {
        for &c in row {
            cols[c].push(i);
        }
    }
    cols
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dae::assembly::ClosureResidual;

    /// F0 = yp0 + 3*y0*y1 - 7
    /// F1 = y1² - y0 - 2*yp1
    /// F2 = y2 - y0*y1 + t
    fn probe_residual() -> ClosureResidual<'static> {
        ClosureResidual::new(|t, y, yp, r| {
            r[0] = yp[0] + 3.0 * y[0] * y[1] - 7.0;
            r[1] = y[1] * y[1] - y[0] - 2.0 * yp[1];
            r[2] = y[2] - y[0] * y[1] + t;
            Ok(())
        })
    }

    // Ground truth: tools/dae-probe run against the real Java DaeJacobian.
    const ORACLE_DENSE: [[f64; 3]; 3] = [
        [1.750000002687102, 4.499999999296733, 0.0],
        [-0.9999999998437186, -9.499999897855105, 0.0],
        [0.7499999998827889, -1.4999999997655777, 0.9999999998437186],
    ];

    #[test]
    fn dense_matches_the_java_oracle_bitwise() {
        let res = probe_residual();
        let y = [1.5, -0.75, 2.25];
        let yp = [0.5, 0.25, -1.0];
        let j = dense(&res, 0.3, 4.0, &y, &yp).unwrap();
        for i in 0..3 {
            for c in 0..3 {
                assert_eq!(
                    j[i][c], ORACLE_DENSE[i][c],
                    "J[{i}][{c}] = {} vs oracle {}",
                    j[i][c], ORACLE_DENSE[i][c]
                );
            }
        }
    }

    #[test]
    fn colored_matches_dense_and_the_oracle() {
        let res = probe_residual();
        let y = [1.5, -0.75, 2.25];
        let yp = [0.5, 0.25, -1.0];
        let rows = vec![vec![0, 1], vec![0, 1], vec![0, 1, 2]];
        let color = color_columns(&rows, 3);
        assert_eq!(color, vec![0, 1, 2], "greedy colouring order");
        let col_rows = transpose_pattern(&rows, 3);
        assert_eq!(col_rows, vec![vec![0, 1, 2], vec![0, 1, 2], vec![2]]);
        let j = dense_colored(&res, 0.3, 4.0, &y, &yp, &col_rows, &color).unwrap();
        for i in 0..3 {
            for c in 0..3 {
                assert_eq!(j[i][c], ORACLE_DENSE[i][c], "coloured J[{i}][{c}]");
            }
        }
    }

    #[test]
    fn csc_colored_matches_dense_colored_bitwise() {
        let res = probe_residual();
        let y = [1.5, -0.75, 2.25];
        let yp = [0.5, 0.25, -1.0];
        let rows = vec![vec![0, 1], vec![0, 1], vec![0, 1, 2]];
        let color = color_columns(&rows, 3);
        let col_rows = transpose_pattern(&rows, 3);
        let mut col_ptr = vec![0];
        for c in &col_rows {
            col_ptr.push(col_ptr.last().unwrap() + c.len());
        }
        let nnz = *col_ptr.last().unwrap();
        let csc_vals =
            csc_colored(&res, 0.3, 4.0, &y, &yp, &col_rows, &color, &col_ptr, nnz).unwrap();

        let j_dense = dense_colored(&res, 0.3, 4.0, &y, &yp, &col_rows, &color).unwrap();

        // Verify each CSC entry matches the corresponding dense entry bitwise.
        for c in 0..3 {
            let start = col_ptr[c];
            for (offset, &r) in col_rows[c].iter().enumerate() {
                assert_eq!(
                    csc_vals[start + offset],
                    j_dense[r][c],
                    "Mismatch at col={c}, row={r}"
                );
            }
        }
    }

    #[test]
    fn csc_colored_scratch_reuse_matches() {
        let res = probe_residual();
        let y = [1.5, -0.75, 2.25];
        let yp = [0.5, 0.25, -1.0];
        let rows = vec![vec![0, 1], vec![0, 1], vec![0, 1, 2]];
        let color = color_columns(&rows, 3);
        let col_rows = transpose_pattern(&rows, 3);
        let mut col_ptr = vec![0];
        for c in &col_rows {
            col_ptr.push(col_ptr.last().unwrap() + c.len());
        }
        let nnz = *col_ptr.last().unwrap();
        let mut values1 = vec![0.0; nnz];
        let mut values2 = vec![0.0; nnz];
        let mut scratch = DaeJacobianScratch::new(3);

        csc_colored_into(
            &res,
            0.3,
            4.0,
            &y,
            &yp,
            None,
            &col_rows,
            &color,
            &col_ptr,
            &mut values1,
            &mut scratch,
        )
        .unwrap();

        // Reusing scratch on a second call with different inputs
        let y2 = [2.0, -1.0, 3.0];
        csc_colored_into(
            &res,
            0.5,
            2.0,
            &y2,
            &yp,
            None,
            &col_rows,
            &color,
            &col_ptr,
            &mut values2,
            &mut scratch,
        )
        .unwrap();

        let j_dense2 = dense_colored(&res, 0.5, 2.0, &y2, &yp, &col_rows, &color).unwrap();
        for c in 0..3 {
            let start = col_ptr[c];
            for (offset, &r) in col_rows[c].iter().enumerate() {
                assert_eq!(values2[start + offset], j_dense2[r][c]);
            }
        }
    }

    #[test]
    fn tridiagonal_colouring_matches_the_oracle() {
        // The oracle's 8x8 tridiagonal pattern -> [0,1,2,0,1,2,0,1].
        let n = 8;
        let rows: Vec<Vec<usize>> = (0..n)
            .map(|i| {
                let mut c = Vec::new();
                if i > 0 {
                    c.push(i - 1);
                }
                c.push(i);
                if i + 1 < n {
                    c.push(i + 1);
                }
                c
            })
            .collect();
        assert_eq!(color_columns(&rows, n), vec![0, 1, 2, 0, 1, 2, 0, 1]);
        assert_eq!(color_count(&color_columns(&rows, n)), 3);
    }

    #[test]
    fn colouring_makes_columns_sharing_a_row_differ() {
        let rows = vec![vec![0, 3], vec![1, 3], vec![2, 3]];
        let color = color_columns(&rows, 4);
        assert_ne!(color[0], color[3]);
        assert_ne!(color[1], color[3]);
        assert_ne!(color[2], color[3]);
        // 0, 1 and 2 never share a row, so greedy gives them all colour 0.
        assert_eq!(color[0], 0);
        assert_eq!(color[1], 0);
        assert_eq!(color[2], 0);
    }

    #[test]
    fn perturbation_floors_at_one() {
        assert_eq!(perturbation(0.0), 1e-7);
        assert_eq!(perturbation(-0.5), 1e-7);
        assert_eq!(perturbation(30.0), 3e-6);
    }

    #[test]
    fn ida_increment_follows_the_sign_of_h_times_yp() {
        let inc = ida_dense_increment(1.0, -1.0, 0.5, 1e-8);
        assert!(inc < 0.0, "increment should follow h*yp < 0");
        let inc = ida_dense_increment(1.0, 1.0, 0.5, 1e-8);
        assert!(inc > 0.0);
        // A zero state with a zero derivative still gets the tolerance floor.
        assert!(ida_dense_increment(0.0, 0.0, 0.5, 1e-8).abs() > 0.0);
    }

    #[test]
    fn residual_failure_propagates_out_of_the_sweep() {
        let res = ClosureResidual::new(|_t, _y, _yp, _r: &mut [f64]| {
            Err(crate::diag::FreesError::property("out of table"))
        });
        assert!(dense(&res, 0.0, 1.0, &[1.0], &[0.0]).is_err());
    }
}
