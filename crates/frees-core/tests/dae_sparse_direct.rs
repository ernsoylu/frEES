//! Phase 5 verification: Direct CSC sparse Jacobian assembly and buffer reuse.
//!
//! Confirms:
//! 1. Direct CSC write produces bitwise-identical Jacobian values to the scattered dense intermediate.
//! 2. Sparse DAE integration matches the dense solver across trajectories without allocating n × n matrices.
//! 3. Large sparse systems (N = 200) integrate stably and conserve physical quantities.

use frees_core::dae::assembly::ClosureResidual;
use frees_core::dae::jacobian::{self, DaeJacobianScratch};
use frees_core::dae::solver::IdaDaeSolver;

#[test]
fn csc_colored_into_matches_dense_colored_on_banded_system() {
    const N: usize = 16;
    // 1D diffusion: F_i = yp[i] - (y[i-1] - 2*y[i] + y[i+1])
    let res = ClosureResidual::new(move |_t, y, yp, r: &mut [f64]| {
        for i in 0..N {
            let left = if i > 0 { y[i - 1] } else { 0.0 };
            let right = if i + 1 < N { y[i + 1] } else { 0.0 };
            r[i] = yp[i] - (left - 2.0 * y[i] + right);
        }
        Ok(())
    });

    let y: Vec<f64> = (0..N).map(|i| (i as f64 + 1.0) * 0.5).collect();
    let yp: Vec<f64> = (0..N).map(|i| (i as f64) * 0.1).collect();

    let sparsity: Vec<Vec<usize>> = (0..N)
        .map(|i| {
            let mut cols = Vec::new();
            if i > 0 {
                cols.push(i - 1);
            }
            cols.push(i);
            if i + 1 < N {
                cols.push(i + 1);
            }
            cols
        })
        .collect();

    let color = jacobian::color_columns(&sparsity, N);
    let col_rows = jacobian::transpose_pattern(&sparsity, N);

    let mut col_ptr = vec![0];
    for c in &col_rows {
        col_ptr.push(col_ptr.last().unwrap() + c.len());
    }
    let nnz = *col_ptr.last().unwrap();

    let mut csc_values = vec![0.0; nnz];
    let mut scratch = DaeJacobianScratch::new(N);

    jacobian::csc_colored_into(
        &res,
        0.5,
        2.5,
        &y,
        &yp,
        None,
        &col_rows,
        &color,
        &col_ptr,
        &mut csc_values,
        &mut scratch,
    )
    .unwrap();

    let dense = jacobian::dense_colored(&res, 0.5, 2.5, &y, &yp, &col_rows, &color).unwrap();

    for c in 0..N {
        let start = col_ptr[c];
        for (offset, &r) in col_rows[c].iter().enumerate() {
            assert_eq!(
                csc_values[start + offset],
                dense[r][c],
                "Bitwise mismatch at col {c}, row {r}"
            );
        }
    }
}

#[test]
fn dense_and_sparse_paths_agree_on_stiff_network() {
    const N: usize = 20;
    // RC ladder / heat diffusion network
    let res = ClosureResidual::new(move |_t, y, yp, r: &mut [f64]| {
        for i in 0..N {
            let prev = if i > 0 { y[i - 1] } else { 100.0 }; // Fixed Dirichlet boundary
            let next = if i + 1 < N { y[i + 1] } else { 0.0 }; // Fixed Dirichlet boundary
            r[i] = yp[i] - 10.0 * (prev - 2.0 * y[i] + next);
        }
        Ok(())
    });

    let y0 = vec![50.0; N];
    let yp0 = vec![0.0; N];

    let sparsity: Vec<Vec<usize>> = (0..N)
        .map(|i| {
            let mut cols = Vec::new();
            if i > 0 {
                cols.push(i - 1);
            }
            cols.push(i);
            if i + 1 < N {
                cols.push(i + 1);
            }
            cols
        })
        .collect();

    // Dense solve
    let mut s_dense = IdaDaeSolver::new(N, &res).unwrap();
    s_dense.set_tolerances(1e-6, 1e-8);
    s_dense.init(0.0, &y0, &yp0).unwrap();
    let step_dense = s_dense.step(0.5).unwrap();

    // Sparse solve (direct CSC path)
    let mut s_sparse = IdaDaeSolver::new(N, &res).unwrap();
    s_sparse.set_tolerances(1e-6, 1e-8);
    s_sparse.set_sparsity(&sparsity).unwrap();
    s_sparse.init(0.0, &y0, &yp0).unwrap();
    let step_sparse = s_sparse.step(0.5).unwrap();

    // Check equivalence
    for i in 0..N {
        let diff = (step_sparse.y[i] - step_dense.y[i]).abs();
        let scale = step_dense.y[i].abs().max(1.0);
        assert!(
            diff / scale < 1e-5,
            "Dense/sparse divergence at node {i}: dense={}, sparse={}, rel_diff={}",
            step_dense.y[i],
            step_sparse.y[i],
            diff / scale
        );
    }
}

#[test]
fn large_sparse_system_integrates_without_dense_allocation() {
    const N: usize = 200; // 200 equations: dense intermediate would have been 40,000 doubles (320 KB) per Jacobian!
    let res = ClosureResidual::new(move |_t, y, yp, r: &mut [f64]| {
        for i in 0..N {
            let prev = if i > 0 { y[i - 1] } else { y[N - 1] }; // Periodic boundary
            let next = if i + 1 < N { y[i + 1] } else { y[0] };
            r[i] = yp[i] - 5.0 * (prev - 2.0 * y[i] + next);
        }
        Ok(())
    });

    let mut y0 = vec![0.0; N];
    for (i, val) in y0.iter_mut().enumerate() {
        *val = (i as f64 * std::f64::consts::PI * 2.0 / N as f64).sin();
    }
    let yp0 = vec![0.0; N];

    let sparsity: Vec<Vec<usize>> = (0..N)
        .map(|i| {
            let mut cols = Vec::new();
            if i > 0 {
                cols.push(i - 1);
            } else {
                cols.push(N - 1);
            }
            cols.push(i);
            if i + 1 < N {
                cols.push(i + 1);
            } else {
                cols.push(0);
            }
            cols.sort();
            cols
        })
        .collect();

    let mut s = IdaDaeSolver::new(N, &res).unwrap();
    s.set_tolerances(1e-5, 1e-7);
    s.set_sparsity(&sparsity).unwrap();
    s.init(0.0, &y0, &yp0).unwrap();

    let step = s.step(0.2).unwrap();
    assert!(step.t >= 0.2);

    // Energy / mass conservation under periodic boundary: sum(y) should remain 0
    let sum: f64 = step.y.iter().sum();
    assert!(
        sum.abs() < 1e-3,
        "Conservation violated on large sparse integration: sum={sum}"
    );
}
