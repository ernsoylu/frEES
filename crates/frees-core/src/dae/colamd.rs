//! Column AMD (COLAMD-lite) fill-reducing permutation for square CSC matrices.
//!
//! Approximate minimum degree on the column-intersection graph (`AᵀA`)
//! without supercolumn absorption. Two columns are adjacent iff they share a
//! row. The live column of minimum degree is eliminated at each step; ties
//! take the lowest index so the permutation is deterministic.
//!
//! Used in front of the Gilbert–Peierls sparse LU in [`crate::dae::solver`].
//! `O(n²)` is acceptable: `n` is a DAE dimension, tens to low hundreds.

#![allow(clippy::needless_range_loop)]

/// Identity permutation `0..n`.
pub fn identity(n: usize) -> Vec<usize> {
    (0..n).collect()
}

/// Scatter a vector from permuted (new) order into original order:
/// `out[perm[i]] = x[i]`. Silent no-op on a length mismatch.
pub fn apply_column_perm(perm: &[usize], x: &[f64], out: &mut [f64]) {
    let n = perm.len();
    if n != x.len() || n != out.len() {
        return;
    }
    for i in 0..n {
        let j = perm[i];
        if j < n {
            out[j] = x[i];
        }
    }
}

/// Gather a vector from original order into permuted order:
/// `out[i] = b[perm[i]]`. Silent no-op on a length mismatch.
#[allow(dead_code)]
pub fn permute_rhs(perm: &[usize], b: &[f64], out: &mut [f64]) {
    let n = perm.len();
    if n != b.len() || n != out.len() {
        return;
    }
    for i in 0..n {
        let j = perm[i];
        if j < n {
            out[i] = b[j];
        }
    }
}

/// Column AMD permutation of a square CSC pattern.
///
/// `perm[k]` is the original column that becomes the `k`-th column of `A P`.
/// Returns the identity on `n == 0` or a malformed pattern (`col_ptr` length
/// not `n + 1`, or `row_idx` shorter than `col_ptr[n]`). Row indices `>= n`
/// are ignored.
///
/// Not applied in [`crate::dae::solver`] yet: permuting before Gilbert–Peierls
/// moved an IDA golden off its 2.5e-9 band. Tests below lock the permutation.
#[allow(dead_code)]
pub fn order(n: usize, col_ptr: &[usize], row_idx: &[usize]) -> Vec<usize> {
    if n == 0 {
        return Vec::new();
    }
    if col_ptr.len() != n + 1 {
        return identity(n);
    }
    let nnz = col_ptr[n];
    if row_idx.len() < nnz {
        return identity(n);
    }

    // Columns that touch each row.
    let mut rows: Vec<Vec<usize>> = vec![Vec::new(); n];
    for c in 0..n {
        for p in col_ptr[c]..col_ptr[c + 1] {
            let r = row_idx[p];
            if r < n {
                rows[r].push(c);
            }
        }
    }

    // Column adjacency of AᵀA: each row's columns form a clique.
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut mark = vec![0usize; n];
    let mut stamp = 1usize;
    for r in 0..n {
        let cols = &rows[r];
        if cols.len() < 2 {
            continue;
        }
        for &c in cols.iter() {
            stamp = next_stamp(&mut mark, stamp);
            for &existing in adj[c].iter() {
                mark[existing] = stamp;
            }
            mark[c] = stamp;
            for &d in cols.iter() {
                if mark[d] != stamp {
                    adj[c].push(d);
                    mark[d] = stamp;
                }
            }
        }
    }

    let mut degree: Vec<usize> = adj.iter().map(Vec::len).collect();
    let mut live = vec![true; n];
    let mut perm = Vec::with_capacity(n);

    for _ in 0..n {
        let mut best = usize::MAX;
        let mut best_deg = usize::MAX;
        for j in 0..n {
            if live[j] && (degree[j] < best_deg || (degree[j] == best_deg && j < best)) {
                best_deg = degree[j];
                best = j;
            }
        }
        if best == usize::MAX {
            break;
        }
        perm.push(best);
        live[best] = false;

        let mut survivors = Vec::new();
        for &q in adj[best].iter() {
            if live[q] {
                survivors.push(q);
            }
        }
        for &p in survivors.iter() {
            stamp = next_stamp(&mut mark, stamp);
            let mut rebuilt = Vec::new();
            mark[p] = stamp;
            mark[best] = stamp;
            for &q in adj[p].iter() {
                if live[q] && mark[q] != stamp {
                    rebuilt.push(q);
                    mark[q] = stamp;
                }
            }
            for &q in survivors.iter() {
                if q != p && mark[q] != stamp {
                    rebuilt.push(q);
                    mark[q] = stamp;
                }
            }
            degree[p] = rebuilt.len();
            adj[p] = rebuilt;
        }
    }
    for j in 0..n {
        if live[j] {
            perm.push(j);
        }
    }
    perm
}

fn next_stamp(mark: &mut [usize], stamp: usize) -> usize {
    let next = stamp.wrapping_add(1);
    if next == 0 {
        mark.fill(0);
        1
    } else {
        next
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_perm(p: &[usize], n: usize) -> bool {
        if p.len() != n {
            return false;
        }
        let mut seen = vec![false; n];
        for &i in p {
            if i >= n || seen[i] {
                return false;
            }
            seen[i] = true;
        }
        true
    }

    #[test]
    fn identity_on_empty_and_malformed() {
        assert!(order(0, &[0], &[]).is_empty());
        assert_eq!(order(3, &[2, 4], &[]), vec![0, 1, 2]);
        // row_idx shorter than col_ptr[n]
        assert_eq!(order(3, &[0, 1, 2, 5], &[0, 1, 2, 4]), vec![0, 1, 2]);
    }

    #[test]
    fn diagonal_is_identity() {
        let col_ptr = [0, 1, 2, 3, 4];
        let row_idx = [0, 1, 2, 3];
        assert_eq!(order(4, &col_ptr, &row_idx), vec![0, 1, 2, 3]);
    }

    #[test]
    fn dense_square_is_a_permutation() {
        // Every column contains rows 0..5.
        let n = 5;
        let mut col_ptr = vec![0];
        let mut row_idx = Vec::new();
        for _ in 0..n {
            for r in 0..n {
                row_idx.push(r);
            }
            col_ptr.push(row_idx.len());
        }
        let perm = order(n, &col_ptr, &row_idx);
        assert!(is_perm(&perm, n), "{perm:?}");
    }

    #[test]
    fn arrowhead_eliminates_the_dense_column_last() {
        // Columns 0..3 are diagonal; column 4 touches every row.
        let n = 5;
        let mut col_ptr = vec![0];
        let mut row_idx = Vec::new();
        for c in 0..n - 1 {
            row_idx.push(c);
            col_ptr.push(row_idx.len());
        }
        for r in 0..n {
            row_idx.push(r);
        }
        col_ptr.push(row_idx.len());
        let perm = order(n, &col_ptr, &row_idx);
        assert!(is_perm(&perm, n), "{perm:?}");
        assert_eq!(perm[n - 1], n - 1, "dense column must be last: {perm:?}");
    }

    #[test]
    fn permute_helpers_round_trip() {
        let perm = vec![2, 0, 1];
        let x = [10.0, 20.0, 30.0];
        let mut original = [0.0; 3];
        apply_column_perm(&perm, &x, &mut original);
        assert_eq!(original, [20.0, 30.0, 10.0]);
        let mut back = [0.0; 3];
        permute_rhs(&perm, &original, &mut back);
        assert_eq!(back, x);
    }

    #[test]
    fn tridiagonal_is_a_permutation() {
        let n = 6;
        let mut col_ptr = vec![0];
        let mut row_idx = Vec::new();
        for c in 0..n {
            if c > 0 {
                row_idx.push(c - 1);
            }
            row_idx.push(c);
            if c + 1 < n {
                row_idx.push(c + 1);
            }
            col_ptr.push(row_idx.len());
        }
        let perm = order(n, &col_ptr, &row_idx);
        assert!(is_perm(&perm, n), "{perm:?}");
    }
}
