//! Exact integer matrix reduction for the global.w batch-3 builtins, ported
//! operation-for-operation from the pinned upstream tree (rev 4d3e9449):
//! `utilities/matreduc.h` (`gcd` with recorder, `column_echelon`,
//! `echelon_solve`), `utilities/matreduc.cpp` (`diagonalise`,
//! `adapted_basis`, `Smith_basis`), `utilities/matrix.cpp:471-498`
//! (`inverse`), and `structure/lattice.cpp:133-160` (`kernel`,
//! `eigen_lattice`, `row_saturate`).
//!
//! Operation-level fidelity matters: the echelon recorder order, the
//! determinant-sign bookkeeping, and the chosen kernel bases are all
//! observable through the interpreter builtins. Arithmetic is wrapping
//! `i32`, mirroring upstream's C++ `int` exactly; only the `echelon_solve`
//! scale factor and the `inverse` denominator are arbitrary-precision
//! (upstream `arithmetic::big_int`). Storage is column-major, matching
//! `linear_values::Matrix`.

use malachite::Integer as BigInt;

use crate::linear_values::Matrix;

/// A mutable rectangular integer matrix in column-major storage (the
/// working form of `linear_values::Matrix` for the reduction algorithms).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PidMatrix {
    rows: usize,
    columns: usize,
    data: Vec<i32>,
}

impl PidMatrix {
    fn new(rows: usize, columns: usize) -> Self {
        Self {
            rows,
            columns,
            data: vec![0; rows * columns],
        }
    }

    fn identity(n: usize) -> Self {
        let mut result = Self::new(n, n);
        for i in 0..n {
            result.set(i, i, 1);
        }
        result
    }

    pub(crate) fn from_matrix(matrix: &Matrix) -> Self {
        let mut result = Self::new(matrix.rows(), matrix.cols());
        for j in 0..matrix.cols() {
            for i in 0..matrix.rows() {
                result.set(i, j, matrix.entry(i, j).expect("entry in range"));
            }
        }
        result
    }

    pub(crate) fn to_matrix(&self) -> Matrix {
        Matrix::from_columns(self.rows, self.columns, self.data.clone())
            .expect("pid matrix data matches dimensions")
    }

    fn get(&self, i: usize, j: usize) -> i32 {
        self.data[j * self.rows + i]
    }

    fn set(&mut self, i: usize, j: usize, value: i32) {
        self.data[j * self.rows + i] = value;
    }

    fn column(&self, j: usize) -> Vec<i32> {
        self.data[j * self.rows..(j + 1) * self.rows].to_vec()
    }

    fn set_column(&mut self, j: usize, column: Vec<i32>) {
        assert_eq!(column.len(), self.rows);
        self.data[j * self.rows..(j + 1) * self.rows].copy_from_slice(&column);
    }

    /// Row entries for columns `j0..j1` (`matrix::Matrix_base::partial_row`,
    /// matrix.h:237-238).
    fn partial_row(&self, i: usize, j0: usize, j1: usize) -> Vec<i32> {
        (j0..j1).map(|j| self.get(i, j)).collect()
    }

    /// Column entries for rows `i0..i1` (`partial_column`, matrix.h:239-240).
    fn partial_column(&self, j: usize, i0: usize, i1: usize) -> Vec<i32> {
        (i0..i1).map(|i| self.get(i, j)).collect()
    }

    /// In-place `v := M * v` (`matrix::Matrix::apply_to`, wrapping).
    fn apply_to(&self, v: &mut [i32]) {
        assert_eq!(v.len(), self.columns);
        let old = v.to_vec();
        for (i, slot) in v.iter_mut().enumerate() {
            let mut sum = 0i32;
            for (j, &x) in old.iter().enumerate() {
                sum = sum.wrapping_add(self.get(i, j).wrapping_mul(x));
            }
            *slot = sum;
        }
    }

    /// Row-vector product `w * M` (`matrix::Matrix::right_prod`, wrapping).
    fn right_prod(&self, w: &[i32]) -> Vec<i32> {
        assert_eq!(w.len(), self.rows);
        (0..self.columns)
            .map(|j| {
                let mut sum = 0i32;
                for (i, &x) in w.iter().enumerate() {
                    sum = sum.wrapping_add(x.wrapping_mul(self.get(i, j)));
                }
                sum
            })
            .collect()
    }

    fn transpose(&mut self) {
        assert_eq!(self.rows, self.columns);
        for i in 0..self.rows {
            for j in (i + 1)..self.columns {
                let a = self.get(i, j);
                let b = self.get(j, i);
                self.set(i, j, b);
                self.set(j, i, a);
            }
        }
    }

    pub(crate) fn transposed(&self) -> Self {
        let mut result = Self::new(self.columns, self.rows);
        for j in 0..self.columns {
            for i in 0..self.rows {
                result.set(j, i, self.get(i, j));
            }
        }
        result
    }

    /// Column `j` plus `c` times column `k` (`matrix::Matrix::columnOperation`).
    fn column_operation(&mut self, j: usize, k: usize, c: i32) {
        if c != 0 {
            for i in 0..self.rows {
                let value = self.get(i, j).wrapping_add(c.wrapping_mul(self.get(i, k)));
                self.set(i, j, value);
            }
        }
    }

    fn swap_columns(&mut self, i: usize, j: usize) {
        for r in 0..self.rows {
            let a = self.get(r, i);
            let b = self.get(r, j);
            self.set(r, i, b);
            self.set(r, j, a);
        }
    }

    fn row_multiply(&mut self, i: usize, c: i32) {
        for j in 0..self.columns {
            self.set(i, j, self.get(i, j).wrapping_mul(c));
        }
    }

    fn column_multiply(&mut self, j: usize, c: i32) {
        for i in 0..self.rows {
            self.set(i, j, self.get(i, j).wrapping_mul(c));
        }
    }

    fn erase_column(&mut self, j: usize) {
        self.data.drain(j * self.rows..(j + 1) * self.rows);
        self.columns -= 1;
    }

    /// The block at rows `i0..i1`, columns `j0..j1` (`PID_Matrix::block`,
    /// matrix.cpp:515-529).
    fn block(&self, i0: usize, j0: usize, i1: usize, j1: usize) -> Self {
        let mut result = Self::new(i1 - i0, j1 - j0);
        for j in j0..j1 {
            for i in i0..i1 {
                result.set(i - i0, j - j0, self.get(i, j));
            }
        }
        result
    }
}

/// Floor division with positive divisor: the unique `q` with
/// `a == q*b + r` and `0 <= r < b` (`arithmetic::divide`, arithmetic.h:249-253;
/// callers guarantee `b > 0`). Unlike Rust's truncating `/` for negative `a`.
fn divide(a: i32, b: i32) -> i32 {
    debug_assert!(b > 0);
    if a >= 0 {
        a / b
    } else {
        -1 - ((-1 - a) / b)
    }
}

/// Non-negative remainder of `a` modulo positive `b` (`arithmetic::remainder`,
/// arithmetic.h:274-280).
fn remainder(a: i32, b: i32) -> i32 {
    debug_assert!(b > 0);
    if a >= 0 {
        a % b
    } else {
        b - 1 - (-1 - a) % b
    }
}

/// `arithmetic::gcd` (arithmetic.h:284-291): non-negative gcd of signed `a`
/// with positive `b`. The `a < 0` path mirrors the upstream sign-extension
/// overflow regime for `i32::MIN` (cast of the wrapped negation to u64).
fn gcd_signed(a: i32, b: i32) -> i32 {
    debug_assert!(b > 0);
    let a = if a > 0 {
        u64::from(a as u32)
    } else if a == 0 {
        return b;
    } else {
        a.wrapping_neg() as i64 as u64
    };
    let mut x = a;
    let mut y = u64::from(b as u32);
    loop {
        x %= y;
        if x == 0 {
            return y as u32 as i32;
        }
        y %= x;
        if y == 0 {
            return x as u32 as i32;
        }
    }
}

/// Left-multiply rows `i..i+r` of `a` by the `r x r` matrix `ops`
/// (`matrix::row_apply`, matrix.h:479-494).
fn row_apply(a: &mut PidMatrix, ops: &PidMatrix, i: usize) {
    let r = ops.rows;
    assert_eq!(r, ops.columns);
    assert!(i + r <= a.rows);
    for j in 0..a.columns {
        let mut tmp: Vec<i32> = (0..r).map(|k| a.get(i + k, j)).collect();
        ops.apply_to(&mut tmp);
        for (k, &value) in tmp.iter().enumerate() {
            a.set(i + k, j, value);
        }
    }
}

/// Right-multiply columns `j..j+r` of `a` by the `r x r` matrix `ops`
/// (`matrix::column_apply`, matrix.h:496-510).
fn column_apply(a: &mut PidMatrix, ops: &PidMatrix, j: usize) {
    let r = ops.rows;
    assert_eq!(r, ops.columns);
    assert!(j + r <= a.columns);
    for i in 0..a.rows {
        let tmp: Vec<i32> = (0..r).map(|l| a.get(i, j + l)).collect();
        let tmp = ops.right_prod(&tmp);
        for (l, &value) in tmp.iter().enumerate() {
            a.set(i, j + l, value);
        }
    }
}

/// GCD of the entries of `row` by unimodular column operations, with the
/// recording matrix (`matreduc::gcd` with `col != nullptr`, matreduc.h:69-122).
/// Returns `(d, col)` where applying `col` moves `d` to entry `dest` and
/// zeroes the rest; `flip` accumulates the determinant sign of the recorded
/// operations. The active-entry scan order and the early exit on an all-zero
/// row (no `dest` swap) reproduce upstream exactly.
pub(crate) fn gcd_recorder(mut row: Vec<i32>, flip: &mut bool, dest: usize) -> (i32, PidMatrix) {
    let n = row.len();
    let mut col = PidMatrix::identity(n);
    let mut active_entries: Vec<usize> = Vec::new();
    let mut min: i32 = 0;
    let mut mindex: usize = 0;
    for (j, &entry) in row.iter().enumerate() {
        if entry != 0 {
            active_entries.push(j);
            if min == 0 || entry.wrapping_abs() < min {
                min = entry.wrapping_abs();
                mindex = j;
            }
        }
    }
    if active_entries.is_empty() {
        return (0, col);
    }
    if row[mindex] < 0 {
        row[mindex] = row[mindex].wrapping_neg();
        *flip = !*flip;
        col.set(mindex, mindex, -1);
    }

    while active_entries.len() > 1 {
        let cur_col = mindex;
        let d = row[mindex];
        let mut idx = 0;
        while idx < active_entries.len() {
            let j = active_entries[idx];
            if j == cur_col {
                idx += 1;
                continue;
            }
            let q = divide(row[j], d);
            col.column_operation(j, cur_col, q.wrapping_neg());
            row[j] = row[j].wrapping_sub(d.wrapping_mul(q));
            if row[j] == 0 {
                active_entries.remove(idx); // `idx` now points at the next entry
            } else {
                if row[j] < min {
                    min = row[j];
                    mindex = j;
                }
                idx += 1;
            }
        }
        debug_assert!(active_entries.len() == 1 || mindex != cur_col);
    }

    if mindex != dest {
        col.swap_columns(dest, mindex);
        *flip = !*flip;
    }

    (min, col)
}

/// Transform `m` to column echelon form in place and return
/// `(col, pivots, flip)` (`matreduc::column_echelon`, matreduc.h:128-161):
/// zero columns are erased from `m` (rank columns remain) and rotated to the
/// right end of the `n x n` recorder `col`, so the last `n - rank` columns of
/// `col` span the kernel. `pivots` lists the pivot rows ascending.
pub(crate) fn column_echelon(m: &mut PidMatrix) -> (PidMatrix, Vec<usize>, bool) {
    let n = m.columns;
    let mut col = PidMatrix::identity(n);
    let mut flip = false;
    let mut pivot_rows = vec![false; m.rows];
    let mut rank = 0usize;
    let mut l = n;
    for i in (0..m.rows).rev() {
        let partial = m.partial_row(i, 0, l);
        // `l - 1` wraps when `l == 0`, but then the row is empty and the
        // gcd early-returns 0 before `dest` is consulted (matreduc.h:83-84).
        let (d, ops) = gcd_recorder(partial, &mut flip, l.wrapping_sub(1));
        if d == 0 {
            continue; // partial row was already zero; skip the row
        }
        column_apply(m, &ops, 0);
        column_apply(&mut col, &ops, 0);
        pivot_rows[i] = true;
        rank += 1;
        l -= 1; // the new pivot sits in column `l`
        debug_assert_eq!(m.get(i, l), d);
    }

    while l > 0 {
        l -= 1;
        // Erase zero column `l` from `m`, and rotate it in `col` towards the
        // right end among the not-yet-removed columns (matreduc.h:150-158).
        m.erase_column(l);
        let cc = col.column(l);
        for j in l..m.columns {
            let next = col.column(j + 1);
            col.set_column(j, next);
        }
        col.set_column(m.columns, cc);
        flip ^= (m.columns - l) % 2 == 1;
    }
    debug_assert_eq!(rank, m.columns);

    let pivots = (0..m.rows).filter(|&i| pivot_rows[i]).collect();
    (col, pivots, flip)
}

/// Solve `E * x == b * f` with `f > 0` minimal, for echelon `E` with the
/// given pivot rows (`matreduc::echelon_solve`, matreduc.h:164-203).
/// Returns `(x, f)`; fails with the upstream message when a non-pivot row
/// has a nonzero right-hand side.
pub(crate) fn echelon_solve(
    e: &PidMatrix,
    pivots: &[usize],
    mut b: Vec<i32>,
) -> Result<(Vec<i32>, BigInt), &'static str> {
    assert_eq!(b.len(), e.rows);
    let mut f = BigInt::from(1);
    let mut result = vec![0i32; e.columns];
    let mut j = pivots.len();
    for i in (0..e.rows).rev() {
        if pivots.contains(&i) {
            j -= 1;
            let pivot = e.get(i, j);
            debug_assert!(pivot > 0);
            let d = gcd_signed(b[i], pivot);
            debug_assert!(d > 0);
            let m = b[i].wrapping_div(d); // C truncating division, as upstream
            if d < pivot {
                // Division is not exact: scale `b` up by `pivot / d`.
                let q = pivot / d;
                f *= BigInt::from(q);
                for entry in b.iter_mut().take(i + 1) {
                    *entry = entry.wrapping_mul(q);
                }
                for entry in result.iter_mut().skip(j + 1) {
                    *entry = entry.wrapping_mul(q);
                }
            }
            result[j] = m;
            for (k, entry) in b.iter_mut().enumerate().take(i + 1) {
                *entry = entry.wrapping_sub(e.get(k, j).wrapping_mul(m));
            }
            debug_assert_eq!(b[i], 0);
        } else if b[i] != 0 {
            return Err("Inconsistent linear system");
        }
    }
    debug_assert_eq!(j, 0);
    Ok((result, f))
}

/// `matreduc::diagonalise` (matreduc.cpp:145-226), operation-faithful:
/// unimodular `row`, `col` with `row * m * col` diagonal; diagonal entries
/// positive except possibly the first; `det(row) == det(col) == 1`. Returns
/// `(row, col, diagonal)`.
pub(crate) fn diagonalise(m: &PidMatrix) -> (PidMatrix, PidMatrix, Vec<i32>) {
    let mut m = m.clone();
    let (n_rows, n_columns) = (m.rows, m.columns);

    let mut row = PidMatrix::identity(n_rows);
    let mut col = PidMatrix::identity(n_columns);
    let mut diagonal: Vec<i32> = Vec::new();
    if n_rows == 0 || n_columns == 0 {
        return (row, col, diagonal);
    }

    let mut row_minus = false; // whether det(row) == -1
    let mut col_minus = false; // whether det(col) == -1
    let mut pivot_columns = vec![false; n_columns];

    let mut k = 0usize; // current pivot row; `l` is the candidate pivot column
    for (l, pivot_column) in pivot_columns.iter_mut().enumerate() {
        let mut flip = false;
        let partial = m.partial_column(l, k, n_rows);
        let (mut d, mut ops) = gcd_recorder(partial, &mut flip, 0);
        if d == 0 {
            continue; // partial column was already zero; do not increment `k`
        }

        *pivot_column = true; // there will be a pivot at `M(k,l)`

        row_minus = flip;
        ops.transpose(); // recorded column operations applied as row ops
        row_apply(&mut m, &ops, k);
        row_apply(&mut row, &ops, k);
        debug_assert_eq!(m.get(k, l), d);

        loop {
            // exit when row and column from `M(k,l)` onwards are zero
            let old_d = d;
            flip = false;
            let partial = m.partial_row(k, l, n_columns);
            let (row_gcd, ops) = gcd_recorder(partial, &mut flip, 0);
            d = row_gcd;
            col_minus ^= flip;
            column_apply(&mut m, &ops, l);
            column_apply(&mut col, &ops, l);
            debug_assert_eq!(m.get(k, l), d);
            if d == old_d {
                break; // no improvement: row was cleared beyond `l`
            }

            let old_d = d;
            flip = false;
            let partial = m.partial_column(l, k, n_rows);
            let (column_gcd, mut ops) = gcd_recorder(partial, &mut flip, 0);
            d = column_gcd;
            row_minus ^= flip;
            ops.transpose(); // recorded column operations applied as row ops
            row_apply(&mut m, &ops, k);
            row_apply(&mut row, &ops, k);
            debug_assert_eq!(m.get(k, l), d);
            if d >= old_d {
                break;
            }
        }

        // The final `flip` folds into `row_minus` once more
        // (matreduc.cpp:201): on the `d == old_d` exit this is the ROW gcd
        // flip (also already in `col_minus`); on the `d >= old_d` exit it
        // re-applies the column gcd flip, cancelling line 193's fold.
        row_minus ^= flip;

        diagonal.push(d); // record the positive gcd that finally remains
        k += 1; // this row contains a pivot, so has been dealt with
    }

    // Adapt `col` by stable-sorting columns, moving non-pivot columns to the
    // end; the sign of the permutation feeds the determinant bookkeeping
    // (matreduc.cpp:207-216).
    {
        let n_piv = diagonal.len();
        let below = pivot_columns.iter().take(n_piv).filter(|&&b| b).count();
        if below < n_piv {
            let mut pi: Vec<usize> = (0..n_columns).filter(|&j| pivot_columns[j]).collect();
            pi.extend((0..n_columns).filter(|&j| !pivot_columns[j]));
            col = pull_back_columns(&col, &pi);
            col_minus ^= permutation_is_negative(&pi);
        }
    }

    if !diagonal.is_empty() && row_minus != col_minus {
        diagonal[0] = diagonal[0].wrapping_neg();
    }
    if row_minus {
        row.row_multiply(0, -1); // ensure determinant of `row` is 1
    }
    if col_minus {
        col.column_multiply(0, -1); // ensure determinant of `col` is 1
    }

    (row, col, diagonal)
}

/// New column `j` is old column `pi[j]` (`permutations::pull_back_columns`).
fn pull_back_columns(m: &PidMatrix, pi: &[usize]) -> PidMatrix {
    let mut result = PidMatrix::new(m.rows, m.columns);
    for (j, &source) in pi.iter().enumerate() {
        for i in 0..m.rows {
            result.set(i, j, m.get(i, source));
        }
    }
    result
}

/// Sign of the permutation `j -> pi[j]`, computed as inversion parity.
fn permutation_is_negative(pi: &[usize]) -> bool {
    let mut inversions = 0usize;
    for i in 0..pi.len() {
        for j in (i + 1)..pi.len() {
            if pi[i] > pi[j] {
                inversions += 1;
            }
        }
    }
    inversions % 2 == 1
}

/// Find `k` with `row[k] % row[0]` positive and minimal
/// (`matreduc.cpp:230-242` `find_small_remainder`); `row[0]` must be positive.
fn find_small_remainder(row: &[i32]) -> Option<usize> {
    let a = row[0];
    debug_assert!(a > 0);
    let mut min = a;
    let mut found = None;
    for (index, &entry) in row.iter().enumerate().skip(1) {
        let r = remainder(entry, a);
        if r > 0 && r < min {
            min = r;
            found = Some(index);
        }
    }
    found
}

/// `matreduc::adapted_basis` (matreduc.cpp:261-336), operation-faithful:
/// returns `(B, diagonal)` with `image(m)` spanned by `diagonal[i]` times
/// column `i` of `B`; all diagonal entries are positive. The recorder applies
/// inverse column operations where `diagonalise` applies row operations.
pub(crate) fn adapted_basis(m: &PidMatrix) -> (PidMatrix, Vec<i32>) {
    let mut m = m.clone();
    let (n_rows, n_columns) = (m.rows, m.columns);

    let mut result = PidMatrix::identity(n_rows);
    let mut flip = false; // fed to the gcd recorder, never read (as upstream)
    let mut diagonal: Vec<i32> = Vec::new();

    let mut kept_rows = vec![false; n_rows];
    let mut j = 0usize;
    for (i, kept_row) in kept_rows.iter_mut().enumerate() {
        let partial = m.partial_row(i, j, n_columns);
        let (mut d, ops) = gcd_recorder(partial, &mut flip, 0);
        if d == 0 {
            continue; // without advancing `j` or recording `i`
        }
        *kept_row = true;
        column_apply(&mut m, &ops, j);
        debug_assert_eq!(m.get(i, j), d);
        // While some entry below the pivot has a nonzero remainder, swap the
        // smallest-remainder row up and reduce (matreduc.cpp:283-307).
        while let Some(k_rel) = find_small_remainder(&m.partial_column(j, i, n_rows)) {
            let k = k_rel + i; // convert relative `k` to the actual row number
            let q = divide(m.get(k, j), m.get(i, j));
            for l in j..n_columns {
                let tmp = m.get(i, l);
                m.set(i, l, m.get(k, l).wrapping_sub(q.wrapping_mul(tmp)));
                m.set(k, l, tmp);
            }
            debug_assert!(m.get(i, j) > 0 && m.get(i, j) < m.get(k, j));

            for l in 0..n_rows {
                // apply the inverse to columns `i,k` of `result`
                let tmp = result.get(l, k);
                result.set(l, k, result.get(l, i).wrapping_add(q.wrapping_mul(tmp)));
                result.set(l, i, tmp);
            }

            let partial = m.partial_row(i, j, n_columns);
            let (new_d, ops) = gcd_recorder(partial, &mut flip, 0);
            d = new_d;
            column_apply(&mut m, &ops, j); // ensure remainder of row `i` is zero
            debug_assert_eq!(m.get(i, j), d);
        }
        // Once divisibility by the pivot is attained the remainder of column
        // `j` of `m` is ignored, but `result` must reflect the conceptual
        // row clearing (matreduc.cpp:308-317).
        for k in (i + 1)..n_rows {
            debug_assert_eq!(m.get(k, j).wrapping_rem(m.get(i, j)), 0);
            let q = m.get(k, j).wrapping_div(m.get(i, j)); // exact division
            result.column_operation(i, k, q); // inverse multiplication on the right
        }
        debug_assert_eq!(m.get(i, j), d);
        diagonal.push(d);
        j += 1;
    }

    // Rotate the recorder columns: kept rows first (in order), then dropped
    // (matreduc.cpp:323-333).
    if kept_rows.iter().any(|&kept| !kept) {
        let mut columns: Vec<Vec<i32>> = Vec::with_capacity(n_rows);
        let mut dropped: Vec<Vec<i32>> = Vec::new();
        for (i, &kept) in kept_rows.iter().enumerate() {
            if kept {
                columns.push(result.column(i));
            } else {
                dropped.push(result.column(i));
            }
        }
        columns.extend(dropped);
        for (i, column) in columns.into_iter().enumerate() {
            result.set_column(i, column);
        }
    }

    (result, diagonal)
}

/// `arithmetic::lcm` for two positive `i32` viewed as `Denom_t` (u64)
/// (arithmetic.cpp `lcm`): returns `(lcm, gcd, mult_a)` truncated back to
/// the machine-int regime at the call site by the caller.
fn lcm_denom(a: i32, b: i32) -> (u64, u64, u64) {
    let mut c = u64::from(a as u32);
    let mut d = u64::from(b as u32);
    let mut m_c = c;
    let mut m_d = 0u64;
    loop {
        m_d += (d / c) * m_c;
        d %= c;
        if d == 0 {
            return (m_d, c, m_c);
        }
        m_c += (c / d) * m_d;
        c %= d;
        if c == 0 {
            let mult_a = if m_d == 0 { 0 } else { m_c - m_d };
            return (m_c, d, mult_a);
        }
    }
}

/// `matreduc::Smith_basis` (matreduc.cpp:359-385), operation-faithful:
/// `adapted_basis` followed by the bubble-sort divisibility correction;
/// returns `(B, inv_factors)` with positive factors and `d_i | d_{i+1}`.
pub(crate) fn smith_basis(m: &PidMatrix) -> (PidMatrix, Vec<i32>) {
    let (mut result, mut diagonal) = adapted_basis(m);
    let mut start = 0usize;
    let mut stop = diagonal.len().saturating_sub(1);
    let mut new_stop = stop;
    while start < stop {
        let mut new_start = new_stop; // if unchanged, this is the last pass
        for i in start..stop {
            if diagonal[i + 1].wrapping_rem(diagonal[i]) != 0 {
                // failure of the divisibility condition (matreduc.cpp:369-381)
                let (lcm, d, pa) = lcm_denom(diagonal[i], diagonal[i + 1]);
                diagonal[i + 1] = lcm as u32 as i32;
                diagonal[i] = d as u32 as i32;

                // Correct the basis for the row operations implicitly applied.
                result.column_operation(i + 1, i, -1);
                let coefficient = 1u64.wrapping_sub(pa / d) as u32 as i32;
                result.column_operation(i, i + 1, coefficient);

                if new_start == stop {
                    // only change `new_start` on the first swap
                    new_start = if i > 0 { i - 1 } else { 0 };
                }
                new_stop = i;
            }
        }
        start = new_start;
        stop = new_stop;
    }
    (result, diagonal)
}

/// `matrix::inverse` (matrix.cpp:471-498): `(N, d)` with `N/d == m^{-1}`,
/// `d > 0` the lcm of the per-column denominators; a singular square matrix
/// yields the zero matrix and `d == 0` WITHOUT an error. Callers guarantee a
/// square matrix. The final `(d/denom).int_val()` narrowing reproduces the
/// upstream overflow diagnostic payload.
pub(crate) fn inverse(m: &PidMatrix) -> Result<(PidMatrix, BigInt), &'static str> {
    assert_eq!(m.rows, m.columns);
    let n = m.rows;
    let result = PidMatrix::new(n, n);
    if n == 0 {
        return Ok((result, BigInt::from(1)));
    }

    let mut a = m.clone();
    let (col, pivots, _flip) = column_echelon(&mut a);
    if pivots.len() != n {
        return Ok((PidMatrix::new(n, n), BigInt::from(0)));
    }

    let mut denoms: Vec<BigInt> = Vec::with_capacity(n);
    let mut basis: Vec<Vec<i32>> = Vec::with_capacity(n);
    for j in 0..n {
        // from easy to harder, but the order is not vital (upstream comment)
        let mut unit = vec![0i32; n];
        unit[j] = 1;
        let (solution, factor) =
            echelon_solve(&a, &pivots, unit).expect("full-pivot echelon solves every unit vector");
        basis.push(solution);
        denoms.push(factor);
    }

    let mut d = denoms[0].clone();
    for denom in &denoms[1..] {
        d = bigint_lcm(d, denom.clone());
    }

    let mut result = PidMatrix::new(n, n);
    for j in 0..n {
        // `col * basis[j] * (d/denoms[j]).int_val()`
        let scale = &d / &denoms[j];
        let scale = i32::try_from(&scale).map_err(|_| "Integer value to big for conversion")?;
        let mut column = basis[j].clone();
        col.apply_to(&mut column);
        for entry in &mut column {
            *entry = entry.wrapping_mul(scale);
        }
        result.set_column(j, column);
    }
    Ok((result, d))
}

/// `arithmetic::lcm` for big integers (bigint.cpp:995-1002): `a*b/gcd`,
/// with the gcd divided out of the smaller side first; positive results for
/// positive inputs.
fn bigint_lcm(a: BigInt, b: BigInt) -> BigInt {
    let mut x = a.clone();
    let mut y = b.clone();
    while y != 0 {
        let r = x % &y;
        x = y;
        y = r;
    }
    let d = x; // positive gcd for positive inputs
    if d == 1 {
        a * b
    } else if a <= b {
        (a / &d) * b
    } else {
        a * (b / &d)
    }
}

/// `lattice::kernel` (lattice.cpp:133-140): an `m x (m - rank)` matrix whose
/// columns span `ker(M)` over the integers, taken from the echelon recorder.
pub(crate) fn kernel(m: &PidMatrix) -> PidMatrix {
    let width = m.columns;
    let mut reduced = m.clone();
    let (col, _pivots, _flip) = column_echelon(&mut reduced);
    col.block(0, reduced.columns, width, width)
}

/// Non-square `M * v` (`matrix::operator*` matrix-vector product): a fresh
/// vector of `m.rows` entries.
fn mat_vec_product(m: &PidMatrix, v: &[i32]) -> Vec<i32> {
    assert_eq!(v.len(), m.columns);
    (0..m.rows)
        .map(|i| {
            let mut sum = 0i32;
            for (j, &x) in v.iter().enumerate() {
                sum = sum.wrapping_add(m.get(i, j).wrapping_mul(x));
            }
            sum
        })
        .collect()
}

/// The `linear_solve` result (global.w:4891-4923): either no solution
/// (union tag 0, `empty_set`) or `(solution, factor, kernel)` with
/// `M * solution == factor * b` and the recorder's kernel block (union tag 1,
/// `affine_subspace`).
pub(crate) enum LinearSolution {
    Empty,
    Affine {
        solution: Vec<i32>,
        factor: BigInt,
        kernel: PidMatrix,
    },
}

/// `linear_solve` core (global.w:4905-4923): column echelon, then
/// `echelon_solve`; the solution is reconstructed through the recorder's
/// pivot block, the kernel from its rotated right block. Callers have
/// already checked `m.rows == b.len()` (the size-mismatch diagnostic fires
/// before the no-value gate upstream).
pub(crate) fn linear_solve(m: &PidMatrix, b: Vec<i32>) -> LinearSolution {
    let width = m.columns;
    let mut reduced = m.clone();
    let (col, pivots, _flip) = column_echelon(&mut reduced);
    let rank = reduced.columns;
    match echelon_solve(&reduced, &pivots, b) {
        Ok((initial, factor)) => {
            // Reconstruct through the recorder's pivot block: the solution
            // has `width` entries, the initial one only `rank`.
            let solution = mat_vec_product(&col.block(0, 0, width, rank), &initial);
            LinearSolution::Affine {
                solution,
                factor,
                kernel: col.block(0, rank, width, width),
            }
        }
        Err(_) => LinearSolution::Empty,
    }
}

/// `lattice::eigen_lattice` (lattice.cpp:142-145): `kernel(M - lambda)`; the
/// diagonal subtraction touches up to `min(rows, cols)` entries
/// (`Matrix::operator-=`, global.w:4235-4248), no square check.
pub(crate) fn eigen_lattice(m: &PidMatrix, lambda: i32) -> PidMatrix {
    let mut shifted = m.clone();
    for i in 0..shifted.rows.min(shifted.columns) {
        shifted.set(i, i, shifted.get(i, i).wrapping_sub(lambda));
    }
    kernel(&shifted)
}

/// `lattice::row_saturate` (lattice.cpp:147-160): the first `rank` rows of
/// `adapted_basis(M^T)^T`, i.e. row `i` of the result is column `i` of the
/// transposed adapted basis.
pub(crate) fn row_saturate(m: &PidMatrix) -> PidMatrix {
    let n = m.columns;
    let (basis, factor) = adapted_basis(&m.transposed());
    let rank = factor.len();
    let mut result = PidMatrix::new(rank, n);
    for i in 0..rank {
        for (j, &entry) in basis.column(i).iter().enumerate() {
            result.set(i, j, entry);
        }
    }
    result
}

/// `swiss_matrix_knife` (global.w:4675-4809): the flag-bitfield slicer.
/// `flags` is already reduced to its low 8 bits by the caller (upstream
/// `BitSet<8>` from `int_val()`, no range or negativity check): bit 0/3
/// reverse the output row/column order, bits 1/2 and 4/5 read the row and
/// column bounds FROM THE END (`lwb = dim - bound`), bit 6 transposes the
/// result (dimensions swapped before the copy), bit 7 negates every entry
/// (wrapping i32, as C++ `int` arithmetic). The bounds diagnostic uses the
/// RAW bounds — the from-end bits do not relax the check — and keeps the
/// verbatim upstream texts, including the "to big"-style absence of a space
/// after "are". The caller has already narrowed the four bounds via
/// `ulong_val()` (upstream pop order `l, j, k, i`).
pub(crate) fn swiss_matrix_knife(
    flags: u8,
    m: &PidMatrix,
    i: u64,
    k: u64,
    j: u64,
    l: u64,
) -> Result<PidMatrix, String> {
    let (rows, columns) = (m.rows as u64, m.columns as u64);
    // global.w:4747-4771: report exactly which raw bounds are out of range.
    let r = rows < i.max(k);
    let c = columns < j.max(l);
    if r || c {
        let mut message = String::from("Range exceeds bounds: ");
        if r {
            if rows < k {
                if rows < i {
                    message.push_str(&format!("both row bounds {i},{k}"));
                } else {
                    message.push_str(&format!("upper row bound {k}"));
                }
            } else {
                message.push_str(&format!("lower row bound {i}"));
            }
        }
        if r && c {
            message.push_str(" and ");
        }
        if c {
            if columns < l {
                if columns < j {
                    message.push_str(&format!("both column bounds {j},{l}"));
                } else {
                    message.push_str(&format!("upper column bound {l}"));
                }
            } else {
                message.push_str(&format!("lower column bound {j}"));
            }
        }
        // No space after "are", a space after the comma (global.w:4770).
        message.push_str(&format!(
            " out of range, actual limits are{rows}, {columns}"
        ));
        return Err(message);
    }
    // The check passed, so every bound fits the matrix dimensions.
    let (i, k, j, l) = (i as usize, k as usize, j as usize, l as usize);
    let (m_rows, n_columns) = (m.rows, m.columns);
    let lwb_r = if flags & 0x02 != 0 { m_rows - i } else { i };
    let mut upb_r = if flags & 0x04 != 0 { m_rows - k } else { k };
    let lwb_c = if flags & 0x10 != 0 { n_columns - j } else { j };
    let mut upb_c = if flags & 0x20 != 0 { n_columns - l } else { l };
    // global.w:4778-4781: inverted ranges clamp to empty, keeping the shape.
    if lwb_r > upb_r {
        upb_r = lwb_r;
    }
    if lwb_c > upb_c {
        upb_c = lwb_c;
    }
    let rows_out = upb_r - lwb_r;
    let columns_out = upb_c - lwb_c;
    let transpose = flags & 0x40 != 0;
    let negate = flags & 0x80 != 0;
    let (r_size, c_size) = if transpose {
        (columns_out, rows_out)
    } else {
        (rows_out, columns_out)
    };
    // transform_copy<transpose,negate> (global.w:4675-4702): the reversal
    // tests stay outside the loops, exactly as upstream's `rev_flags`.
    let mut result = PidMatrix::new(r_size, c_size);
    for out_i in 0..rows_out {
        let source_row = if flags & 0x01 != 0 {
            upb_r - 1 - out_i
        } else {
            lwb_r + out_i
        };
        for out_j in 0..columns_out {
            let source_column = if flags & 0x08 != 0 {
                upb_c - 1 - out_j
            } else {
                lwb_c + out_j
            };
            let value = m.get(source_row, source_column);
            let value = if negate { value.wrapping_neg() } else { value };
            if transpose {
                result.set(out_j, out_i, value);
            } else {
                result.set(out_i, out_j, value);
            }
        }
    }
    Ok(result)
}

/// `permutations::standardization` (permutations.cpp:257-282): `pi[l]` is
/// the number of values `< a[l]` plus the number of EARLIER values equal to
/// `a[l]` — the stable-sort destination permutation of `a`.
fn standardization(a: &[usize], bound: usize) -> Vec<usize> {
    let mut count = vec![0usize; bound];
    for &value in a {
        debug_assert!(value < bound);
        count[value] += 1;
    }
    let mut sum = 0usize;
    for cell in count.iter_mut() {
        let ci = *cell;
        *cell = sum;
        sum += ci;
    }
    // now `count[v]` holds the number of values less than `v` in `a`
    let mut result = vec![0usize; a.len()];
    for (index, &value) in a.iter().enumerate() {
        result[index] = count[value];
        count[value] += 1;
    }
    result
}

/// `BitMatrix<64>::section` (bitvector.cpp:346-405), behind `mod2_section`
/// (global.w:5043-5053): a GF(2) matrix `B` of TRANSPOSE shape
/// (n_columns x n_rows) with `ABA == A` and `BAB == B`. Entry conversion is
/// `(x & 1) != 0` (negative odd entries are 1; bitvector.cpp:145-154).
/// Upstream bounds-guards rows/columns by `assert`s only, compiled out
/// under NDEBUG: row bits >= 64 are MASKED on input here, reproducing the
/// silent drop observed on the pinned oracle build; basis bits >= 64
/// (columns beyond 64) are masked likewise. Both regimes are UB upstream —
/// keep >64 inputs out of fixtures.
pub(crate) fn mod2_section(m: &PidMatrix) -> PidMatrix {
    let (d_rows, d_columns) = (m.rows, m.columns);
    let mut column = vec![0u64; d_columns]; // copy of our matrix's columns
    for (j, slot) in column.iter_mut().enumerate() {
        let mut bits = 0u64;
        for i in 0..d_rows.min(64) {
            if m.get(i, j) & 1 != 0 {
                bits |= 1u64 << i;
            }
        }
        *slot = bits;
    }
    // square matrix, initialised to the `d_columns` identity (bits >= 64
    // masked, see the caveat above)
    let mut basis = vec![0u64; d_columns];
    for (i, slot) in basis.iter_mut().enumerate().take(64.min(d_columns)) {
        *slot = 1u64 << i;
    }
    let mut pivots = 0u64; // row r is set if some column has its pivot there
    let mut pivot_col = [0usize; 64]; // column number having pivot in row r
    for k in 0..d_columns {
        let col_k = column[k];
        if col_k == 0 {
            continue; // `k` is never stored in `pivot_col`; `basis[k]` stays
        }
        let cur_pivot = col_k.trailing_zeros() as usize; // row of the pivot
        pivots |= 1u64 << cur_pivot;
        pivot_col[cur_pivot] = k;
        let b_k = basis[k]; // basis vector to be added to some others
                            // clear `cur_pivot` out of existing pivot columns above ours
        let mut above = pivots & ((1u64 << cur_pivot) - 1);
        while above != 0 {
            let r = above.trailing_zeros() as usize;
            above &= above - 1;
            let j = pivot_col[r]; // column where that row has its pivot
            if column[j] >> cur_pivot & 1 == 1 {
                column[j] ^= col_k;
                basis[j] ^= b_k;
            }
        }
        // also clear row `cur_pivot` in the (yet) non-pivot columns beyond k
        for j in (k + 1)..d_columns {
            if column[j] >> cur_pivot & 1 == 1 {
                column[j] ^= col_k;
                basis[j] ^= b_k;
            }
        }
    }
    // transpose-shaped result: column r of B is the preimage of e_r
    let mut result = PidMatrix::new(d_columns, d_rows);
    let mut rest = pivots;
    while rest != 0 {
        let r = rest.trailing_zeros() as usize;
        rest &= rest - 1;
        let preimage = basis[pivot_col[r]];
        for i in 0..d_columns.min(64) {
            if preimage >> i & 1 == 1 {
                result.set(i, r, 1);
            }
        }
    }
    result
}

/// `subspace_normal` (global.w:5062-5174): the GF(2) reduced column-echelon
/// normal form of a possibly dependent generator set, with combination and
/// relation tracking. The caller has already validated `dim <= 64` and
/// `n_gens <= 64` (those diagnostics fire BEFORE the no-value gate).
/// Returns `(basis_m, combin_m, relations_m, pivots)`: the dim x rank basis
/// with columns in ASCENDING pivot order (via `standardization`, NOT the
/// loop order), the n_gens x rank expressions of the basis vectors in the
/// original generators, the n_gens x (n_gens - rank) relations for the
/// excluded generators (column order = generator order minus pivoters,
/// `d = j - l`), and the ascending pivot rows.
pub(crate) fn subspace_normal(
    generators: &PidMatrix,
) -> (PidMatrix, PidMatrix, PidMatrix, Vec<usize>) {
    let dim = generators.rows;
    let n_gens = generators.columns;
    debug_assert!(dim <= 64 && n_gens <= 64);
    // `basis[l]` has pivot row `pivot[l]` and came from generator
    // `pivoter[l]`; `combination[j]` expresses the (virtual) basis element
    // from generator j in the original generators (initBasis: identity).
    let mut basis: Vec<u64> = Vec::new();
    let mut combination: Vec<u64> = (0..n_gens).map(|j| 1u64 << j).collect();
    let mut pivot: Vec<usize> = Vec::new();
    let mut pivoter: Vec<usize> = Vec::new();
    // the generators reduced modulo 2 (negative odd entries are 1)
    let generator_bits: Vec<u64> = (0..n_gens)
        .map(|j| {
            let mut bits = 0u64;
            for i in 0..dim {
                if generators.get(i, j) & 1 != 0 {
                    bits |= 1u64 << i;
                }
            }
            bits
        })
        .collect();
    for (j, &bits) in generator_bits.iter().enumerate() {
        let mut v = bits;
        for (l, &piv) in pivot.iter().enumerate() {
            if v >> piv & 1 == 1 {
                v ^= basis[l];
                combination[j] ^= combination[pivoter[l]];
            }
        }
        if v != 0 {
            let piv = v.trailing_zeros() as usize; // new pivot
            for l in 0..basis.len() {
                if basis[l] >> piv & 1 == 1 {
                    basis[l] ^= v;
                    combination[pivoter[l]] ^= combination[j];
                }
            }
            basis.push(v);
            pivoter.push(j);
            pivot.push(piv);
        }
    }
    let pi = standardization(&pivot, dim); // relative positions of pivots
    let rank = basis.len();
    let mut basis_m = PidMatrix::new(dim, rank);
    let mut combin_m = PidMatrix::new(n_gens, rank);
    let mut relations_m = PidMatrix::new(n_gens, n_gens - rank);
    let mut pivot_r = vec![0usize; rank];
    let mut l = 0usize; // basis vectors copied so far, current index
    for (j, &comb_j) in combination.iter().enumerate() {
        if l < rank && j == pivoter[l] {
            let d = pi[l]; // destination position
            let mut bits = basis[l];
            while bits != 0 {
                let i = bits.trailing_zeros() as usize;
                bits &= bits - 1;
                basis_m.set(i, d, 1);
            }
            let mut bits = comb_j;
            while bits != 0 {
                let i = bits.trailing_zeros() as usize;
                bits &= bits - 1;
                combin_m.set(i, d, 1);
            }
            pivot_r[d] = pivot[l];
            l += 1;
        } else {
            let d = j - l;
            let mut bits = comb_j;
            while bits != 0 {
                let i = bits.trailing_zeros() as usize;
                bits &= bits - 1;
                relations_m.set(i, d, 1);
            }
        }
    }
    debug_assert_eq!(l, rank);
    (basis_m, combin_m, relations_m, pivot_r)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Row-major literal constructor for tests.
    fn matrix(rows: usize, columns: usize, entries: &[i32]) -> PidMatrix {
        assert_eq!(entries.len(), rows * columns);
        let mut result = PidMatrix::new(rows, columns);
        for (index, &entry) in entries.iter().enumerate() {
            result.set(index / columns, index % columns, entry);
        }
        result
    }

    fn mat_mul(a: &PidMatrix, b: &PidMatrix) -> PidMatrix {
        assert_eq!(a.columns, b.rows);
        let mut result = PidMatrix::new(a.rows, b.columns);
        for j in 0..b.columns {
            for i in 0..a.rows {
                let mut sum = 0i32;
                for k in 0..a.columns {
                    sum = sum.wrapping_add(a.get(i, k).wrapping_mul(b.get(k, j)));
                }
                result.set(i, j, sum);
            }
        }
        result
    }

    /// Oracle-pinned cases from `tests/fixtures/eval/global_batch3.atlas`
    /// (the byte-identical fixture diff against the C++ oracle is the
    /// authoritative check; these pin the same values at the module level).
    #[test]
    fn oracle_anchored_results() {
        // echelon([[1,3],[2,4]]) (row-major; the fixture's
        // `mat: [[1,2],[3,4]]`) = (E, C, [0,1], -1) with zero columns kept.
        let mut m = matrix(2, 2, &[1, 3, 2, 4]);
        let (col, pivots, flip) = column_echelon(&mut m);
        assert_eq!(m, matrix(2, 2, &[1, 1, 0, 2]));
        assert_eq!(col, matrix(2, 2, &[-2, 1, 1, 0]));
        assert_eq!(pivots, vec![0, 1]);
        assert!(flip);

        // Rank-deficient: the zero column is erased from E and rotated to
        // the right of C, where it spans the kernel.
        let mut m = matrix(2, 2, &[2, 4, 4, 8]);
        let (col, pivots, flip) = column_echelon(&mut m);
        assert_eq!(m, matrix(2, 1, &[2, 4]));
        assert_eq!(col, matrix(2, 2, &[1, -2, 0, 1]));
        assert_eq!(pivots, vec![1]);
        assert!(!flip);
        assert_eq!(kernel(&matrix(2, 2, &[1, 2, 2, 4])), matrix(2, 1, &[-2, 1]));
        assert_eq!(kernel(&PidMatrix::new(0, 3)), PidMatrix::identity(3));

        // invert: (N, d) with N/d = M^-1; singular square -> zero matrix + 0.
        let (n, d) = inverse(&matrix(2, 2, &[1, 2, 3, 4])).expect("invertible");
        assert_eq!(n, matrix(2, 2, &[-4, 2, 3, -1]));
        assert_eq!(d, BigInt::from(2));
        let (n, d) = inverse(&matrix(2, 2, &[1, 2, 2, 4])).expect("singular is not an error");
        assert_eq!(n, PidMatrix::new(2, 2));
        assert_eq!(d, BigInt::from(0));
        let (n, d) = inverse(&PidMatrix::new(0, 0)).expect("empty inverts");
        assert_eq!(n, PidMatrix::new(0, 0));
        assert_eq!(d, BigInt::from(1));

        // diagonalize([[0,2],[2,0]]) = ([2,2], swap, identity).
        let (row, col, diagonal) = diagonalise(&matrix(2, 2, &[0, 2, 2, 0]));
        assert_eq!(diagonal, vec![2, 2]);
        assert_eq!(row, matrix(2, 2, &[0, 1, 1, 0]));
        assert_eq!(col, PidMatrix::identity(2));

        // First diagonal entry may carry the sign: diagonalize([[-2]]).
        let (_row, _col, diagonal) = diagonalise(&matrix(1, 1, &[-2]));
        assert_eq!(diagonal, vec![-2]);

        // Smith([[2,0],[0,3]]) = ([[4,-3],[-1,1]]... oracle: B = |4,-1|,|-3,1|
        // with factors [1,6] (the correction loop fired).
        let (basis, factors) = smith_basis(&matrix(2, 2, &[2, 0, 0, 3]));
        assert_eq!(factors, vec![1, 6]);
        assert_eq!(basis, matrix(2, 2, &[4, -1, -3, 1]));

        // Bezout([6,10,15]) = (1, C) with v*C == [1,0,0].
        let (d, recorder) = gcd_recorder(vec![6, 10, 15], &mut false, 0);
        assert_eq!(d, 1);
        assert_eq!(recorder, matrix(3, 3, &[1, 5, -5, 1, 0, -3, -1, -2, 4]));
    }

    #[test]
    fn diagonalise_reconstructs_and_smith_orders() {
        let cases = [
            matrix(2, 2, &[1, 2, 3, 4]),
            matrix(2, 2, &[0, 2, 2, 0]),
            matrix(2, 2, &[4, 6, 3, 9]),
            matrix(3, 2, &[1, 2, 3, 4, 5, 6]),
            matrix(2, 3, &[1, 2, 3, 4, 5, 6]),
            matrix(3, 3, &[2, 4, 4, -6, 6, 12, 10, -4, -16]),
        ];
        for case in &cases {
            let (row, col, diagonal) = diagonalise(case);
            let product = mat_mul(&mat_mul(&row, case), &col);
            for i in 0..product.rows {
                for j in 0..product.columns {
                    let expected = if i == j && i < diagonal.len() {
                        diagonal[i]
                    } else {
                        0
                    };
                    assert_eq!(product.get(i, j), expected, "diagonalise {case:?}");
                }
            }
            for &d in diagonal.iter().skip(1) {
                assert!(d > 0, "only the first diagonal entry may be negative");
            }

            // Smith factors are positive and divisibility-ordered.
            let (_basis, factors) = smith_basis(case);
            for i in 0..factors.len() {
                assert!(factors[i] > 0);
                if i + 1 < factors.len() {
                    assert_eq!(factors[i + 1] % factors[i], 0, "Smith order {case:?}");
                }
            }
        }
    }

    #[test]
    fn echelon_solve_scales_and_rejects() {
        // diag(2,4) against [6,3]: factor 4, solution [12,3].
        let mut m = matrix(2, 2, &[2, 0, 0, 4]);
        let (_col, pivots, _flip) = column_echelon(&mut m);
        let (solution, factor) = echelon_solve(&m, &pivots, vec![6, 3]).expect("scaled solution");
        assert_eq!(solution, vec![12, 3]); // E * x == b * f: diag(2,4)*[12,3] == 4*[6,3]
        assert_eq!(factor, BigInt::from(4));

        // Inconsistent non-pivot row rejects with the upstream payload.
        let mut m = matrix(2, 2, &[1, 2, 2, 4]);
        let (_col, pivots, _flip) = column_echelon(&mut m);
        assert_eq!(
            echelon_solve(&m, &pivots, vec![3, 5]),
            Err("Inconsistent linear system")
        );
    }

    /// Oracle-pinned cases from `tests/fixtures/eval/global_batch4.atlas`
    /// (the byte-identical fixture diff against the C++ oracle is the
    /// authoritative check). Matrices below are row-major literals, so the
    /// fixture's `mat: [[1,2],[3,4],[5,6]]` (COLUMN literals) appears here
    /// as the 2x3 matrix `[1,3,5 ; 2,4,6]`.
    #[test]
    fn swiss_matrix_knife_oracle_cases() {
        let m = matrix(2, 3, &[1, 3, 5, 2, 4, 6]);
        let slice =
            |flags, i, k, j, l| swiss_matrix_knife(flags, &m, i, k, j, l).expect("in-range slice");
        assert_eq!(slice(0, 0, 2, 0, 3), m); // identity
                                             // bit 6 transposes (dimensions swapped BEFORE the copy).
        assert_eq!(slice(64, 0, 2, 0, 3), matrix(3, 2, &[1, 2, 3, 4, 5, 6]));
        // bit 7 negates; combined with bit 0 the row reversal comes first.
        assert_eq!(
            slice(128, 0, 2, 0, 3),
            matrix(2, 3, &[-1, -3, -5, -2, -4, -6])
        );
        assert_eq!(
            slice(129, 0, 2, 0, 3),
            matrix(2, 3, &[-2, -4, -6, -1, -3, -5])
        );
        assert_eq!(slice(1, 0, 2, 0, 3), matrix(2, 3, &[2, 4, 6, 1, 3, 5]));
        assert_eq!(slice(8, 0, 2, 0, 3), matrix(2, 3, &[5, 3, 1, 6, 4, 2]));
        // from-end bound bits: lwb_r = m - i, upb_r = m - k, and columns.
        assert_eq!(slice(2, 1, 2, 0, 3), matrix(1, 3, &[2, 4, 6]));
        assert_eq!(slice(4, 0, 1, 0, 3), matrix(1, 3, &[1, 3, 5]));
        assert_eq!(slice(16, 0, 2, 1, 3), matrix(2, 1, &[5, 6]));
        assert_eq!(slice(32, 0, 2, 0, 1), matrix(2, 2, &[1, 3, 2, 4]));
        assert_eq!(slice(192, 0, 2, 1, 3), matrix(2, 2, &[-3, -4, -5, -6]));
        // inverted ranges clamp to empty, keeping the (swapped) shape.
        assert_eq!(slice(0, 2, 0, 0, 1), PidMatrix::new(0, 1));
        assert_eq!(slice(64, 2, 0, 0, 1), PidMatrix::new(1, 0));
        // all-bits flags on in-range zero bounds: from-end bits send both
        // row bounds to m and the clamp fires.
        assert_eq!(
            swiss_matrix_knife(255, &PidMatrix::new(2, 3), 0, 0, 0, 0).expect("clamped"),
            PidMatrix::new(0, 0)
        );
        // bit 7 negate wraps i32 (C++ int arithmetic): -(i32::MIN) is itself.
        assert_eq!(
            swiss_matrix_knife(128, &matrix(1, 1, &[i32::MIN]), 0, 1, 0, 1).expect("wrap"),
            matrix(1, 1, &[i32::MIN])
        );
    }

    #[test]
    fn swiss_matrix_knife_bounds_diagnostics() {
        let m = matrix(2, 2, &[1, 3, 2, 4]); // the fixture's mat: [[1,2],[3,4]]
        let message = |i, k, j, l| swiss_matrix_knife(0, &m, i, k, j, l).expect_err("out of range");
        // Verbatim upstream texts: NO space after "are", a space after ",".
        assert_eq!(
            message(0, 3, 0, 2),
            "Range exceeds bounds: upper row bound 3 out of range, actual limits are2, 2"
        );
        assert_eq!(
            message(5, 1, 0, 1),
            "Range exceeds bounds: lower row bound 5 out of range, actual limits are2, 2"
        );
        assert_eq!(
            message(0, 1, 0, 5),
            "Range exceeds bounds: upper column bound 5 out of range, actual limits are2, 2"
        );
        assert_eq!(
            message(0, 1, 5, 1),
            "Range exceeds bounds: lower column bound 5 out of range, actual limits are2, 2"
        );
        assert_eq!(
            message(5, 9, 3, 7),
            "Range exceeds bounds: both row bounds 5,9 and both column bounds 3,7 out of range, actual limits are2, 2"
        );
        assert_eq!(
            message(5, 9, 0, 7),
            "Range exceeds bounds: both row bounds 5,9 and upper column bound 7 out of range, actual limits are2, 2"
        );
        assert_eq!(
            message(0, 9, 1, 8),
            "Range exceeds bounds: upper row bound 9 and upper column bound 8 out of range, actual limits are2, 2"
        );
        // The from-end bits do NOT relax the raw-bound check: flags 2 makes
        // lwb_r = m - 5 underflow conceptually, but the check fires first.
        assert_eq!(
            swiss_matrix_knife(2, &m, 5, 1, 0, 1).expect_err("raw bounds"),
            "Range exceeds bounds: lower row bound 5 out of range, actual limits are2, 2"
        );
    }

    /// `mod2_section` (bitvector.cpp:346-405): GF(2) section with
    /// transpose-shaped output; fixtures pin these against the oracle.
    #[test]
    fn mod2_section_oracle_cases() {
        // Identity section.
        assert_eq!(
            mod2_section(&matrix(2, 2, &[1, 0, 0, 1])),
            PidMatrix::identity(2)
        );
        // mat: [[1,1],[0,1],[1,0]] (2x3, full row rank) -> a right inverse.
        assert_eq!(
            mod2_section(&matrix(2, 3, &[1, 0, 1, 1, 1, 0])),
            matrix(3, 2, &[1, 0, 1, 1, 0, 0])
        );
        // All-even entries reduce to zero mod 2.
        assert_eq!(
            mod2_section(&matrix(2, 2, &[2, 6, 4, 8])),
            PidMatrix::new(2, 2)
        );
        // Negative odd entries are 1 mod 2.
        assert_eq!(mod2_section(&matrix(1, 1, &[-1])), PidMatrix::identity(1));
        // mat: [[1,0,1],[0,1,1]] (3 rows, 2 columns): transpose-shaped 2x3.
        assert_eq!(
            mod2_section(&matrix(3, 2, &[1, 0, 0, 1, 1, 1])),
            matrix(2, 3, &[1, 0, 0, 0, 1, 0])
        );
        assert_eq!(mod2_section(&PidMatrix::new(0, 0)), PidMatrix::new(0, 0));
        assert_eq!(mod2_section(&PidMatrix::new(2, 3)), PidMatrix::new(3, 2));
        // Row bits >= 64 are masked on input (upstream NDEBUG UB regime):
        // null(65,1) silently drops row 64's (zero) bits -> zero 1x65.
        assert_eq!(mod2_section(&PidMatrix::new(65, 1)), PidMatrix::new(1, 65));
        // ABA == A and BAB == B over GF(2) on a rank-deficient matrix.
        let a = matrix(3, 3, &[1, 1, 0, 0, 1, 1, 1, 0, 1]);
        let b = mod2_section(&a);
        let gf2 = |m: &PidMatrix| {
            let mut r = m.clone();
            for entry in &mut r.data {
                *entry &= 1;
            }
            r
        };
        assert_eq!(gf2(&mat_mul(&mat_mul(&a, &b), &a)), gf2(&a));
        assert_eq!(gf2(&mat_mul(&mat_mul(&b, &a), &b)), b);
    }

    /// `subspace_normal` (global.w:5062-5174): reduced column-echelon over
    /// GF(2) with combination/relation tracking and PIVOT-ASCENDING output.
    #[test]
    fn subspace_normal_oracle_cases() {
        let run = |m: &PidMatrix| {
            let (b, c, r, p) = subspace_normal(m);
            (b, c, r, p)
        };
        // Independent generators: basis and combination are the identity.
        assert_eq!(
            run(&matrix(2, 2, &[1, 0, 0, 1])),
            (
                PidMatrix::identity(2),
                PidMatrix::identity(2),
                PidMatrix::new(2, 0),
                vec![0, 1]
            )
        );
        // mat: [[1,1],[1,0],[0,1]] (dim 2, 3 generators, third dependent).
        assert_eq!(
            run(&matrix(2, 3, &[1, 1, 0, 1, 0, 1])),
            (
                PidMatrix::identity(2),
                matrix(3, 2, &[0, 1, 1, 1, 0, 0]),
                matrix(3, 1, &[1, 1, 1]),
                vec![0, 1]
            )
        );
        // mat: [[1,1,2],[2,0,2]] (dim 3, 2 gens, second == 0 mod 2).
        assert_eq!(
            run(&matrix(3, 2, &[1, 2, 1, 0, 2, 2])),
            (
                matrix(3, 1, &[1, 1, 0]),
                matrix(2, 1, &[1, 0]),
                matrix(2, 1, &[0, 1]),
                vec![0]
            )
        );
        // mat: [[1,0],[1,0],[0,0]] (duplicate + zero column).
        assert_eq!(
            run(&matrix(2, 3, &[1, 1, 0, 0, 0, 0])),
            (
                matrix(2, 1, &[1, 0]),
                matrix(3, 1, &[1, 0, 0]),
                matrix(3, 2, &[1, 0, 1, 0, 0, 1]),
                vec![0]
            )
        );
        // mat: [[0,1],[0,1]]: the sole pivot is row 1.
        assert_eq!(
            run(&matrix(2, 2, &[0, 0, 1, 1])),
            (
                matrix(2, 1, &[0, 1]),
                matrix(2, 1, &[1, 0]),
                matrix(2, 1, &[1, 1]),
                vec![1]
            )
        );
        // Negative odd entries count as 1; even negative entries as 0.
        assert_eq!(
            run(&matrix(2, 2, &[-1, 0, 0, -3])),
            (
                PidMatrix::identity(2),
                PidMatrix::identity(2),
                PidMatrix::new(2, 0),
                vec![0, 1]
            )
        );
        // All-odd rank-1 generator set.
        assert_eq!(
            run(&matrix(2, 2, &[3, 5, 7, 9])),
            (
                matrix(2, 1, &[1, 1]),
                matrix(2, 1, &[1, 0]),
                matrix(2, 1, &[1, 1]),
                vec![0]
            )
        );
        // Degenerate shapes.
        assert_eq!(
            run(&PidMatrix::new(0, 0)),
            (
                PidMatrix::new(0, 0),
                PidMatrix::new(0, 0),
                PidMatrix::new(0, 0),
                vec![]
            )
        );
        assert_eq!(
            run(&PidMatrix::new(3, 0)),
            (
                PidMatrix::new(3, 0),
                PidMatrix::new(0, 0),
                PidMatrix::new(0, 0),
                vec![]
            )
        );
        // No generators are retained: relations are the identity.
        assert_eq!(
            run(&PidMatrix::new(0, 2)),
            (
                PidMatrix::new(0, 0),
                PidMatrix::new(2, 0),
                PidMatrix::identity(2),
                vec![]
            )
        );
        // standardization reorders basis columns by ASCENDING pivot, not by
        // loop order: generators [[0,1],[1,0]] pivot at rows 1 then 0.
        assert_eq!(
            run(&matrix(2, 2, &[0, 1, 1, 0])),
            (
                PidMatrix::identity(2),
                matrix(2, 2, &[0, 1, 1, 0]),
                PidMatrix::new(2, 0),
                vec![0, 1]
            )
        );
    }
}
