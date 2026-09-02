//! Exact integer matrix reduction, ported operation-for-operation from the
//! upstream `utilities/matreduc.cpp` (`diagonalise`, `gcd`, `has_solution`,
//! `find_solution`).
//!
//! Operation-level fidelity matters here: the chosen solution of an
//! underdetermined system `A*x == b` is observable downstream (the parity of
//! the coordinates of `tau`/`t` enters `ext_block::same_sign`), so this port
//! reproduces the exact sequence of unimodular operations of the C++
//! algorithm, including its determinant-sign bookkeeping. Arithmetic is
//! wrapping `i32`, mirroring upstream's C++ `int` exactly (including its
//! overflow regime, which is observable in the chosen solutions).

use crate::StructureError;

/// A rectangular integer matrix in row-major storage.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct IntMatrix {
    rows: usize,
    columns: usize,
    data: Vec<i32>,
}

impl IntMatrix {
    /// Zero matrix of the given shape.
    pub(crate) fn new(rows: usize, columns: usize) -> Self {
        Self {
            rows,
            columns,
            data: vec![0; rows * columns],
        }
    }

    /// Identity matrix of the given size.
    pub(crate) fn identity(n: usize) -> Self {
        let mut result = Self::new(n, n);
        for i in 0..n {
            result.data[i * n + i] = 1;
        }
        result
    }

    /// Build from a row-major entry list; panics on shape mismatch.
    pub(crate) fn from_entries(rows: usize, columns: usize, data: Vec<i32>) -> Self {
        assert_eq!(rows * columns, data.len());
        Self {
            rows,
            columns,
            data,
        }
    }

    pub(crate) fn n_rows(&self) -> usize {
        self.rows
    }

    pub(crate) fn n_columns(&self) -> usize {
        self.columns
    }

    pub(crate) fn get(&self, i: usize, j: usize) -> i32 {
        self.data[i * self.columns + j]
    }

    pub(crate) fn set(&mut self, i: usize, j: usize, value: i32) {
        self.data[i * self.columns + j] = value;
    }

    /// In-place `v := M * v` (`matrix::Matrix::apply_to`).
    pub(crate) fn apply_to(&self, v: &mut [i32]) {
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

    /// Row-vector product `w * M` (`matrix::Matrix::right_prod`); equivalently
    /// `M^T * w` as a column vector.
    pub(crate) fn right_prod(&self, w: &[i32]) -> Vec<i32> {
        assert_eq!(w.len(), self.rows);
        let mut result = vec![0; self.columns];
        for (j, slot) in result.iter_mut().enumerate() {
            let mut sum = 0i32;
            for (i, &x) in w.iter().enumerate() {
                sum = sum.wrapping_add(x.wrapping_mul(self.get(i, j)));
            }
            *slot = sum;
        }
        result
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

    /// Row `i` plus `c` times row `k` (`matrix::Matrix::rowOperation`).
    fn row_operation(&mut self, i: usize, k: usize, c: i32) {
        if c != 0 {
            for j in 0..self.columns {
                let value = self.get(i, j).wrapping_add(c.wrapping_mul(self.get(k, j)));
                self.set(i, j, value);
            }
        }
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

    fn swap_rows(&mut self, i: usize, j: usize) {
        for c in 0..self.columns {
            let a = self.get(i, c);
            let b = self.get(j, c);
            self.set(i, c, b);
            self.set(j, c, a);
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
            let value = self.get(i, j).wrapping_mul(c);
            self.set(i, j, value);
        }
    }

    fn column_multiply(&mut self, j: usize, c: i32) {
        for i in 0..self.rows {
            let value = self.get(i, j).wrapping_mul(c);
            self.set(i, j, value);
        }
    }
}

/// Floor division with positive divisor: the unique `q` with
/// `a == q*b + r` and `0 <= r < b` (`arithmetic::divide`; callers guarantee
/// `b > 0`). This differs from Rust's truncating `/` for negative `a`.
fn divide(a: i32, b: i32) -> i32 {
    debug_assert!(b > 0);
    if a >= 0 {
        a / b
    } else {
        -1 - ((-1 - a) / b)
    }
}

/// Left-multiply rows `i..i+r` of `a` by the `r x r` matrix `ops`
/// (`matrix::row_apply`).
fn row_apply(a: &mut IntMatrix, ops: &IntMatrix, i: usize) {
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
/// (`matrix::column_apply`).
fn column_apply(a: &mut IntMatrix, ops: &IntMatrix, j: usize) {
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

/// GCD of the entries of `row` by unimodular column operations
/// (`matreduc::gcd` with recording matrix). Returns the (positive) GCD and
/// the recorded column operations; `flip` accumulates the determinant sign
/// of those operations. After applying `ops`, entry `dest` holds the GCD.
fn gcd(mut row: Vec<i32>, flip: &mut bool, dest: usize) -> (i32, IntMatrix) {
    let n = row.len();
    let mut col = IntMatrix::identity(n);
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
            col.column_operation(j, cur_col, -q);
            row[j] = row[j].wrapping_sub(d.wrapping_mul(q));
            if row[j] == 0 {
                active_entries.remove(idx); // |idx| now points to the next entry
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

/// New column `j` is old column `pi[j]` (`permutations::pull_back_columns`).
fn pull_back_columns(m: &IntMatrix, pi: &[usize]) -> IntMatrix {
    let mut result = IntMatrix::new(m.rows, m.columns);
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

/// Find unimodular `row`, `col` such that `row * m * col` is diagonal, and
/// return `(row, col, diagonal)`; the diagonal entries are positive except
/// possibly the first (`matreduc::diagonalise`, operation-faithful port,
/// including its exact sign bookkeeping).
pub(crate) fn diagonalise(m: &IntMatrix) -> (IntMatrix, IntMatrix, Vec<i32>) {
    let mut m = m.clone();
    let (n_rows, n_columns) = (m.rows, m.columns);

    let mut row = IntMatrix::identity(n_rows);
    let mut col = IntMatrix::identity(n_columns);
    let mut diagonal: Vec<i32> = Vec::new();
    if n_rows == 0 || n_columns == 0 {
        return (row, col, diagonal);
    }

    let mut row_minus = false; // whether det(row) == -1
    let mut col_minus = false; // whether det(col) == -1
    let mut pivot_columns = vec![false; n_columns];

    let mut k = 0usize; // current pivot row; |l| is the candidate pivot column
    for l in 0..n_columns {
        let mut flip = false;
        let partial: Vec<i32> = (k..n_rows).map(|i| m.get(i, l)).collect();
        let (column_gcd, mut ops) = gcd(partial, &mut flip, 0);
        let mut d = column_gcd;
        if d == 0 {
            continue; // partial column was already zero; do not increment |k|
        }

        pivot_columns[l] = true; // there will be a pivot at |M(k,l)|

        row_minus = flip;
        ops.transpose(); // recorded column operations applied as row ops
        row_apply(&mut m, &ops, k);
        row_apply(&mut row, &ops, k);
        debug_assert_eq!(m.get(k, l), d);

        let mut old_d; // used in the final condition of the next loop
        loop {
            // exit when row and column from |M(k,l)| onwards are zero
            old_d = d;
            flip = false;
            let partial: Vec<i32> = (l..n_columns).map(|j| m.get(k, j)).collect();
            let (row_gcd, ops) = gcd(partial, &mut flip, 0);
            d = row_gcd;
            col_minus ^= flip;
            column_apply(&mut m, &ops, l);
            column_apply(&mut col, &ops, l);
            debug_assert_eq!(m.get(k, l), d);
            if d == old_d {
                break; // no improvement: row was cleared beyond |l|
            }

            old_d = d;
            flip = false;
            let partial: Vec<i32> = (k..n_rows).map(|i| m.get(i, l)).collect();
            let (column_gcd, mut ops) = gcd(partial, &mut flip, 0);
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

        row_minus ^= flip;

        diagonal.push(d); // record positive gcd that finally remains
        k += 1; // this row contains a pivot, so has been dealt with
    }

    // Adapt |col| by stable-sorting columns, moving non-pivot columns to the
    // end; the sign of the permutation feeds the determinant bookkeeping.
    {
        let n_piv = diagonal.len();
        let below = pivot_columns.iter().take(n_piv).filter(|&&b| b).count();
        if below < n_piv {
            // pivot columns not left-adjusted
            let mut pi: Vec<usize> = (0..n_columns).filter(|&j| pivot_columns[j]).collect();
            pi.extend((0..n_columns).filter(|&j| !pivot_columns[j]));
            col = pull_back_columns(&col, &pi);
            col_minus ^= permutation_is_negative(&pi);
        }
    }

    if !diagonal.is_empty() && row_minus != col_minus {
        diagonal[0] = -diagonal[0];
    }
    if row_minus {
        row.row_multiply(0, -1); // ensure determinant of |row| is 1
    }
    if col_minus {
        col.column_multiply(0, -1); // ensure determinant of |col| is 1
    }

    (row, col, diagonal)
}

/// Whether the integral system `a * x == b` has a solution
/// (`matreduc::has_solution`).
pub(crate) fn has_solution(a: &IntMatrix, b: &[i32]) -> bool {
    let (row, _col, diagonal) = diagonalise(a);
    let mut b = b.to_vec();
    row.apply_to(&mut b); // left multiply equation by |row|, giving D*col^{-1}*x = row*b

    for i in (0..b.len()).rev() {
        if (if i < diagonal.len() {
            b[i].wrapping_rem(diagonal[i])
        } else {
            b[i]
        }) != 0
        {
            return false;
        }
    }
    true
}

/// A solution of `a * x == b` (`matreduc::find_solution`), or `None` when the
/// system has no integral solution (upstream throws in that case; callers
/// here are expected to have run `has_solution` first).
pub(crate) fn find_solution(a: &IntMatrix, b: &[i32]) -> Option<Vec<i32>> {
    let (row, col, diagonal) = diagonalise(a);
    let mut b = b.to_vec();
    row.apply_to(&mut b);

    // now solve for the value of |col^{-1} * x|
    for (i, &d) in diagonal.iter().enumerate() {
        if b[i].wrapping_rem(d) != 0 {
            return None;
        }
        b[i] = b[i].wrapping_div(d);
    }
    if b[diagonal.len()..].iter().any(|&entry| entry != 0) {
        return None;
    }

    b.resize(col.n_rows(), 0); // adapt size in opposite sense to |a|
    col.apply_to(&mut b); // finally reconstruct the value of |x|
    Some(b)
}

/// Whether `beta` lies in the image of the operator `a`
/// (`ext_block.cpp` `in_L_image`).
pub(crate) fn in_left_image(beta: &[i32], a: &IntMatrix) -> bool {
    let (left, _right, inv_fact) = diagonalise(a);
    let mut image = beta.to_vec();
    left.apply_to(&mut image);

    for (i, &entry) in image.iter().enumerate() {
        if i < inv_fact.len() {
            if entry.wrapping_rem(inv_fact[i]) != 0 {
                return false;
            }
        } else if entry != 0 {
            return false;
        }
    }
    true
}

/// Whether `b` lies in the image of the operator `a^T`
/// (`ext_block.cpp` `in_R_image`).
pub(crate) fn in_right_image(a: &IntMatrix, b: &[i32]) -> bool {
    let (_left, right, inv_fact) = diagonalise(a);
    let image = right.right_prod(b);

    for (i, &entry) in image.iter().enumerate() {
        if i < inv_fact.len() {
            if entry.wrapping_rem(inv_fact[i]) != 0 {
                return false;
            }
        } else if entry != 0 {
            return false;
        }
    }
    true
}

/// Exact inverse of a unit upper-triangular matrix
/// (`matrix::inverse_upper_triangular`, matrix.cpp:420-440): back
/// substitution with wrapping `i32` arithmetic. Errors where upstream
/// throws: non-square input, or a diagonal entry different from 1.
pub(crate) fn inverse_upper_triangular(m: &IntMatrix) -> Result<IntMatrix, StructureError> {
    let n = m.n_columns();
    if m.n_rows() != n {
        return Err(StructureError::RepInvariantViolation {
            invariant: "invert triangular: matrix is not square",
        });
    }
    let mut result = IntMatrix::new(n, n);
    // Row-major slices instead of `get`: the row of `m` is contiguous and
    // wrapping addition is associative, so the ascending-k accumulation is
    // bit-identical to upstream's descending one (matrix.cpp:432-438).
    for j in 0..n {
        if m.get(j, j) != 1 {
            return Err(StructureError::RepInvariantViolation {
                invariant: "invert triangular: not unitriangular",
            });
        }
        result.data[j * n + j] = 1;
        for i in (0..j).rev() {
            let row = &m.data[i * n + (i + 1)..i * n + (j + 1)];
            let mut sum = 0_i32;
            for (offset, &entry) in row.iter().enumerate() {
                if entry != 0 {
                    let k = i + 1 + offset;
                    sum = sum.wrapping_add(entry.wrapping_mul(result.data[k * n + j]));
                }
            }
            result.data[i * n + j] = sum.wrapping_neg();
        }
    }
    Ok(result)
}

/// `arithmetic::exp_i` (arithmetic.h:49-51): `i^n` as ±1 for even `n`;
/// the evenness precondition is an upstream `assert`.
pub(crate) fn exp_i(n: i32) -> i32 {
    debug_assert_eq!(n % 2, 0, "exp_i: odd exponent");
    if n % 4 == 0 {
        1
    } else {
        -1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn matrix(rows: usize, columns: usize, entries: &[i32]) -> IntMatrix {
        IntMatrix::from_entries(rows, columns, entries.to_vec())
    }

    fn mat_mul(a: &IntMatrix, b: &IntMatrix) -> IntMatrix {
        assert_eq!(a.columns, b.rows);
        let mut result = IntMatrix::new(a.rows, b.columns);
        for i in 0..a.rows {
            for j in 0..b.columns {
                let mut sum = 0i32;
                for k in 0..a.columns {
                    sum += a.get(i, k) * b.get(k, j);
                }
                result.set(i, j, sum);
            }
        }
        result
    }

    fn det(matrix: &IntMatrix) -> i64 {
        assert_eq!(matrix.rows, matrix.columns);
        let n = matrix.rows;
        // Leibniz over permutations; only used for tiny test matrices.
        let mut result = 0i64;
        let entry = |i: usize, p: usize| i64::from(matrix.get(i, p));
        let mut perm: Vec<usize> = (0..n).collect();
        loop {
            let mut term = 1i64;
            for (i, &p) in perm.iter().enumerate() {
                term *= entry(i, p);
            }
            result += if permutation_is_negative(&perm) {
                -term
            } else {
                term
            };
            // next permutation (lexicographic)
            let mut i = n;
            while i > 1 && perm[i - 2] >= perm[i - 1] {
                i -= 1;
            }
            if i == 1 {
                break;
            }
            let pivot = i - 2;
            let mut j = n - 1;
            while perm[j] <= perm[pivot] {
                j -= 1;
            }
            perm.swap(pivot, j);
            perm[(pivot + 1)..].reverse();
        }
        result
    }

    #[test]
    fn diagonalise_reconstructs() {
        let cases = [
            matrix(1, 1, &[0]),
            matrix(1, 1, &[7]),
            matrix(1, 1, &[-7]),
            matrix(2, 2, &[2, 0, 0, 4]),
            matrix(2, 2, &[1, 2, 3, 4]),
            matrix(2, 2, &[0, 2, 2, 0]),
            matrix(2, 2, &[4, 6, 3, 9]),
            matrix(3, 2, &[1, 2, 3, 4, 5, 6]),
            matrix(2, 3, &[1, 2, 3, 4, 5, 6]),
            matrix(3, 3, &[2, 4, 4, -6, 6, 12, 10, -4, -16]),
            matrix(3, 3, &[0, 0, 2, 0, 3, 0, 5, 0, 0]),
        ];
        for case in &cases {
            let (row, col, diagonal) = diagonalise(case);
            // unimodular transformation matrices (upstream only forces
            // det(col) == 1; det(row) can be -1, e.g. for [[0,2],[2,0]])
            assert_eq!(det(&row).abs(), 1, "row determinant for {case:?}");
            assert_eq!(det(&col), 1, "col determinant for {case:?}");
            let product = mat_mul(&mat_mul(&row, case), &col);
            for i in 0..product.rows {
                for j in 0..product.columns {
                    if i == j && i < diagonal.len() {
                        assert_eq!(product.get(i, i), diagonal[i], "diagonal for {case:?}");
                    } else {
                        assert_eq!(product.get(i, j), 0, "off-diagonal for {case:?}");
                    }
                }
            }
            for (i, &d) in diagonal.iter().enumerate() {
                if i > 0 {
                    assert!(d > 0, "only the first diagonal entry may be negative");
                }
            }
        }
    }

    #[test]
    fn find_solution_solves() {
        // a = [[2,0],[0,4]], b = [6,4] -> x = [3,1]
        let a = matrix(2, 2, &[2, 0, 0, 4]);
        assert!(has_solution(&a, &[6, 4]));
        assert!(!has_solution(&a, &[6, 3]));
        let x = find_solution(&a, &[6, 4]).expect("solvable");
        assert_eq!(x, vec![3, 1]);

        // rank-deficient: a = [[1,2],[2,4]], b = [3,6] -> some x with A*x==b
        let a = matrix(2, 2, &[1, 2, 2, 4]);
        assert!(has_solution(&a, &[3, 6]));
        assert!(!has_solution(&a, &[3, 5]));
        let x = find_solution(&a, &[3, 6]).expect("solvable");
        let mut check = vec![0i32; 2];
        for i in 0..2 {
            check[i] = a.get(i, 0) * x[0] + a.get(i, 1) * x[1];
        }
        assert_eq!(check, vec![3, 6]);

        // rectangular: a = [[1,2,3],[4,5,6]], b = [6,15]
        let a = matrix(2, 3, &[1, 2, 3, 4, 5, 6]);
        let x = find_solution(&a, &[6, 15]).expect("solvable");
        assert_eq!(x.len(), 3);
        for (i, &target) in [6i32, 15].iter().enumerate() {
            let sum: i32 = (0..3).map(|j| a.get(i, j) * x[j]).sum();
            assert_eq!(sum, target);
        }
    }

    #[test]
    fn image_membership() {
        // image of 2*I in 1 dimension is the even numbers
        let a = matrix(1, 1, &[2]);
        assert!(in_left_image(&[4], &a));
        assert!(!in_left_image(&[3], &a));
        assert!(in_right_image(&a, &[4]));
        assert!(!in_right_image(&a, &[3]));
    }

    /// Cases captured from the C++ oracle (`utilities/matreduc.cpp`):
    /// bit-exact `(row, col, diagonal)` and solutions, including the
    /// negative-first-diagonal and rank-deficient regimes.
    #[test]
    fn oracle_reference_cases() {
        // case 0: rank-deficient 2x2 with negative diagonal entry
        let a = matrix(2, 2, &[0, 5, 0, 0]);
        let (row, col, diagonal) = diagonalise(&a);
        assert_eq!(row, matrix(2, 2, &[1, 0, 0, 1]));
        assert_eq!(col, matrix(2, 2, &[0, 1, -1, 0]));
        assert_eq!(diagonal, vec![-5]);
        assert!(!has_solution(&a, &[-2, -6]));
        assert!(!has_solution(&a, &[-3, -4]));
        assert!(!has_solution(&a, &[1, 4]));
        assert!(has_solution(&a, &[10, 0]));
        assert_eq!(find_solution(&a, &[10, 0]), Some(vec![0, 2]));

        // case 2: negative scalar keeps its sign on the diagonal
        let a = matrix(1, 1, &[-4]);
        let (_row, _col, diagonal) = diagonalise(&a);
        assert_eq!(diagonal, vec![-4]);
        assert!(!has_solution(&a, &[6]));
        assert!(!has_solution(&a, &[5]));
        assert!(!has_solution(&a, &[-5]));
        assert!(has_solution(&a, &[12]));
        assert_eq!(find_solution(&a, &[12]), Some(vec![-3]));

        // case 9: rank-deficient 6x6 with non-left-adjusted pivot columns
        let a = matrix(
            6,
            6,
            &[
                2, 0, -5, 0, 0, 0, //
                2, -4, -4, -3, 5, 6, //
                0, -2, 0, -1, 0, 0, //
                1, 0, 0, 0, 0, 0, //
                0, 0, -3, 0, 0, 0, //
                0, 6, 2, -2, -2, -3,
            ],
        );
        let (row, col, diagonal) = diagonalise(&a);
        assert_eq!(
            row,
            matrix(
                6,
                6,
                &[
                    0, 0, 0, 1, 0, 0, //
                    0, 0, -1, 0, 0, 0, //
                    1, 0, -6, -2, 0, 3, //
                    -2, 1, -3, 2, 2, 0, //
                    -43, 22, -68, 42, 43, 1, //
                    3, 0, 0, -6, -5, 0,
                ],
            )
        );
        assert_eq!(
            col,
            matrix(
                6,
                6,
                &[
                    1, 0, 0, 0, 0, 0, //
                    0, 0, 0, -2, 0, 1, //
                    0, 0, 1, 66, -9, 0, //
                    0, 1, 0, 4, 0, -2, //
                    0, 0, 0, 1, 6, -22, //
                    0, 0, 0, 0, -5, 18,
                ],
            )
        );
        assert_eq!(diagonal, vec![1, 1, 1, 1, 3]);
        assert!(!has_solution(&a, &[2, 3, -1, -5, 1, -3]));
        assert!(!has_solution(&a, &[-5, 3, 0, 6, 5, 3]));
        assert!(!has_solution(&a, &[-1, 2, -5, 1, -2, 0]));
        let b = [-13, 12, 9, 6, -15, -25];
        assert!(has_solution(&a, &b));
        assert_eq!(find_solution(&a, &b), Some(vec![6, 14, 5, -37, -421, 345]));
    }

    #[test]
    fn inverse_upper_triangular_recovers_the_unit_inverse() {
        let m = matrix(3, 3, &[1, 2, 3, 0, 1, 4, 0, 0, 1]);
        assert_eq!(
            inverse_upper_triangular(&m).unwrap(),
            matrix(3, 3, &[1, -2, 5, 0, 1, -4, 0, 0, 1])
        );
        // Product check on a second example.
        let m = matrix(2, 2, &[1, -7, 0, 1]);
        let inv = inverse_upper_triangular(&m).unwrap();
        assert_eq!(inv, matrix(2, 2, &[1, 7, 0, 1]));

        // Non-square and non-unitriangular inputs are rejected.
        assert!(inverse_upper_triangular(&IntMatrix::new(2, 3)).is_err());
        assert!(inverse_upper_triangular(&matrix(2, 2, &[2, 1, 0, 1])).is_err());
    }

    #[test]
    fn inverse_upper_triangular_matches_naive_back_substitution() {
        // Pseudo-random unit upper-triangular matrices with entries large
        // enough to exercise wrapping i32 arithmetic; compare against a
        // straightforward descending-k back substitution.
        fn naive(m: &IntMatrix) -> IntMatrix {
            let n = m.n_columns();
            let mut result = IntMatrix::new(n, n);
            for j in 0..n {
                result.set(j, j, 1);
                for i in (0..j).rev() {
                    let mut sum = 0_i32;
                    for k in ((i + 1)..=j).rev() {
                        sum = sum.wrapping_add(m.get(i, k).wrapping_mul(result.get(k, j)));
                    }
                    result.set(i, j, sum.wrapping_neg());
                }
            }
            result
        }
        let mut state = 0x243F6A88_85A308D3_u64;
        let mut next = move || {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (state >> 33) as u32
        };
        for &n in &[1_usize, 2, 5, 17, 33] {
            let mut m = IntMatrix::new(n, n);
            for i in 0..n {
                m.set(i, i, 1);
                for j in (i + 1)..n {
                    // Mix small values with near-overflow magnitudes.
                    let value = match next() % 3 {
                        0 => (next() % 7) as i32 - 3,
                        _ => (next() as i32) >> (next() % 5),
                    };
                    m.set(i, j, value);
                }
            }
            assert_eq!(inverse_upper_triangular(&m).unwrap(), naive(&m), "n={n}");
        }
    }

    #[test]
    fn exp_i_is_the_fourth_root_of_unity() {
        assert_eq!(exp_i(0), 1);
        assert_eq!(exp_i(2), -1);
        assert_eq!(exp_i(4), 1);
        assert_eq!(exp_i(-2), -1);
        assert_eq!(exp_i(6), -1);
    }
}
