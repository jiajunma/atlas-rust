//! The per-involution `(1-theta)X^*` image-basis pair (lift_mat, M_real).
//!
//! This is upstream's `InvolutionTable::record` pair (involutions.h:104-105).
//! `lift_mat` is the column-echelon basis of the image of `1-theta`
//! (rank `n x r`); `m_real` (rank `r x n`) expresses an image element in
//! that basis. The pair satisfies `lift_mat * m_real == 1 - theta`
//! (involutions.h:105).
//!
//! The basis is NOT uniquely determined by theta: upstream seeds it from
//! the echelon reduction of `1-theta` at the Cartan orbit's canonical
//! involution (`InvolutionTable::add_involution`, involutions.cpp:196-208)
//! and then transports it along the cross-action BFS (`add_cross`,
//! involutions.cpp:242-243). The seed's echelon reduction reproduces
//! upstream's `matreduc::column_echelon` (matreduc.h:129) and its gcd
//! sweep (matreduc.h:70) operation-for-operation, because the elected
//! `lambda-rho` representative and the `y_lift` signs depend on the exact
//! image basis.

use crate::{LatticeInvolution, StructureError};

/// The per-involution `(1-theta)X^*` image data: upstream's
/// `InvolutionTable::record` pair (involutions.h:104-105). Entries are
/// `i32`, matching upstream's `int_Matrix` (the echelon machinery works in
/// `i64` and converts with a checked narrowing at the boundary); this halves
/// the retained per-record payloads of the involution table.
///
/// Both matrices are stored as ONE flat row-major buffer each. The
/// involution table retains ~270k of these pairs on the unipotent
/// workload, and the retired per-row `Vec<i32>` layout cost a 24B header
/// plus a minimum-size malloc chunk per row — massif attributed ~112MB of
/// the unipotent heap peak to that overhead.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RealProjection {
    /// `lift_mat` entries, row-major: `rank` rows of `image_rank` columns.
    lift_mat: Vec<i32>,
    /// `m_real` entries, row-major: `image_rank` rows of `rank` columns.
    m_real: Vec<i32>,
    /// Ambient lattice rank `n` (`lift_mat` rows, `m_real` columns).
    rank: u32,
    /// Image rank `r` (`lift_mat` columns, `m_real` rows).
    image_rank: u32,
}

/// Row views of a flat row-major buffer. A zero stride iterates zero rows
/// (the rank-0 datum), where `chunks_exact(0)` would panic.
fn rows(flat: &[i32], stride: usize, count: usize) -> impl Iterator<Item = &[i32]> {
    debug_assert_eq!(flat.len(), stride * count);
    (0..count).map(move |row| &flat[row * stride..(row + 1) * stride])
}

/// Checked narrowing of an echelon-computed `i64` matrix into the stored
/// `i32` representation (upstream computes these in `int` throughout).
fn narrow_matrix(matrix: &[Vec<i64>]) -> Result<Vec<Vec<i32>>, StructureError> {
    let mut narrowed = Vec::new();
    narrowed
        .try_reserve_exact(matrix.len())
        .map_err(|_| StructureError::AllocationFailed {
            requested: matrix.len(),
        })?;
    for row in matrix {
        let mut narrowed_row = Vec::new();
        narrowed_row
            .try_reserve_exact(row.len())
            .map_err(|_| StructureError::AllocationFailed {
                requested: row.len(),
            })?;
        for &entry in row {
            narrowed_row.push(
                i32::try_from(entry).map_err(|_| StructureError::ArithmeticOverflow)?,
            );
        }
        narrowed.push(narrowed_row);
    }
    Ok(narrowed)
}

impl RealProjection {
    /// Flattening constructor: validates the nested shape (`lift_mat` is
    /// `n x r`, `m_real` is `r x n`) and packs both matrices row-major.
    pub(crate) fn from_nested(
        lift_mat: Vec<Vec<i32>>,
        m_real: Vec<Vec<i32>>,
    ) -> Result<Self, StructureError> {
        let rank = lift_mat.len();
        let image_rank = m_real.len();
        if lift_mat.iter().any(|row| row.len() != image_rank)
            || m_real.iter().any(|row| row.len() != rank)
        {
            return Err(StructureError::InvalidIntegerMatrixShape);
        }
        let mut flat_lift = Vec::new();
        flat_lift
            .try_reserve_exact(rank * image_rank)
            .map_err(|_| StructureError::AllocationFailed {
                requested: rank * image_rank,
            })?;
        for row in &lift_mat {
            flat_lift.extend_from_slice(row);
        }
        let mut flat_real = Vec::new();
        flat_real
            .try_reserve_exact(image_rank * rank)
            .map_err(|_| StructureError::AllocationFailed {
                requested: image_rank * rank,
            })?;
        for row in &m_real {
            flat_real.extend_from_slice(row);
        }
        Ok(Self {
            lift_mat: flat_lift,
            m_real: flat_real,
            rank: u32::try_from(rank).map_err(|_| StructureError::ArithmeticOverflow)?,
            image_rank: u32::try_from(image_rank)
                .map_err(|_| StructureError::ArithmeticOverflow)?,
        })
    }

    /// The ambient lattice rank `n`.
    pub(crate) fn rank(&self) -> usize {
        self.rank as usize
    }

    /// `lift_mat[row][column]` of the flat row-major storage.
    pub(crate) fn lift_entry(&self, row: usize, column: usize) -> i32 {
        self.lift_mat[row * self.image_rank() + column]
    }

    /// `m_real` rows, each of length `rank`.
    pub(crate) fn m_real_rows(&self) -> impl Iterator<Item = &[i32]> {
        rows(&self.m_real, self.rank(), self.image_rank())
    }

    /// Nested `lift_mat` view for value assertions in tests.
    #[cfg(test)]
    pub(crate) fn lift_mat_nested(&self) -> Vec<Vec<i32>> {
        rows(&self.lift_mat, self.image_rank(), self.rank())
            .map(|row| row.to_vec())
            .collect()
    }

    /// Nested `m_real` view for value assertions in tests.
    #[cfg(test)]
    pub(crate) fn m_real_nested(&self) -> Vec<Vec<i32>> {
        self.m_real_rows().map(|row| row.to_vec()).collect()
    }

    /// Port of `matreduc::column_echelon` (matreduc.h:129-161) applied to
    /// `1-theta`, tracking the column-operation matrix and its inverse
    /// incrementally; upstream's `InvolutionTable::add_involution`
    /// (involutions.cpp:196-208) then takes `M_real` as the first `r`
    /// rows of the inverse.
    pub(crate) fn build(theta: &LatticeInvolution) -> Result<Self, StructureError> {
        let matrix = theta.weight_matrix();
        let rank = matrix.len();
        // `a` starts as the integer matrix `1 - theta` (involutions.cpp:196).
        let mut a: Vec<Vec<i64>> = Vec::new();
        a.try_reserve_exact(rank)
            .map_err(|_| StructureError::AllocationFailed { requested: rank })?;
        for (row_index, row) in matrix.iter().enumerate() {
            let mut converted = Vec::new();
            converted
                .try_reserve_exact(rank)
                .map_err(|_| StructureError::AllocationFailed { requested: rank })?;
            for (column_index, &entry) in row.iter().enumerate() {
                let diagonal = i64::from(row_index == column_index);
                converted.push(
                    diagonal
                        .checked_sub(i64::from(entry))
                        .ok_or(StructureError::ArithmeticOverflow)?,
                );
            }
            a.push(converted);
        }
        let mut col = identity_matrix(rank)?;

        // Row sweep, bottom row first; the pivot of each processed row
        // lands at column `limit - 1` (matreduc.h:136-148). Each sweep
        // builds an `l x l` ops matrix (negation + column operations +
        // final swap) and applies it to `a` and `col` at once, exactly
        // like `column_echelon`'s `column_apply`.
        let mut limit = rank;
        for row in (0..rank).rev() {
            let pivot = gcd_sweep(&mut a, &mut col, row, limit)?;
            if pivot == 0 {
                continue; // partial row already zero: no pivot in this row
            }
            limit -= 1; // pivot now at column `limit`
        }

        // Erase the `limit` zero columns, rotating the corresponding
        // kernel columns of `col` towards the right end one at a time
        // (matreduc.h:150-158): columns already parked at the right are
        // not touched again.
        let zero_columns = limit;
        let mut erased = 0_usize;
        while limit > 0 {
            limit -= 1;
            for a_row in a.iter_mut() {
                a_row.remove(limit);
            }
            let m_columns = rank - erased - 1; // column count of `a` now
            let cc: Vec<i64> = (0..rank).map(|k| col[k][limit]).collect();
            for col_row in col.iter_mut() {
                for j in limit..m_columns {
                    col_row[j] = col_row[j + 1];
                }
            }
            for (k, col_row) in col.iter_mut().enumerate() {
                col_row[m_columns] = cc[k];
            }
            erased += 1;
        }
        debug_assert_eq!(erased, zero_columns);

        let image_rank = rank - zero_columns;
        // `M_real = col.inverse().block(0,0,image_rank,rank)`
        // (involutions.cpp:203): the integer inverse of the (unimodular)
        // column-operation matrix, computed by Euclidean row reduction.
        let col_inverse = invert_integer_matrix(&col)?;
        let projection = Self::from_nested(
            narrow_matrix(&a)?,
            narrow_matrix(&col_inverse[..image_rank])?,
        )?;
        projection.check_against(theta)?;
        Ok(projection)
    }

    /// Transport the image-basis pair across one simple reflection using
    /// its rank-one formula `s = I - alpha * beta^T`. This is equivalent to
    /// [`Self::transported`] for a matrix produced by
    /// `WeylAction::simple_reflection`, but avoids the `rank^2` matrix walk
    /// for each of the two products.
    pub(crate) fn transported_by_simple_reflection(
        &self,
        reflected_vector: &[i32],
        pairing_vector: &[i32],
    ) -> Result<Self, StructureError> {
        let rank = self.rank();
        if reflected_vector.len() != rank || pairing_vector.len() != rank {
            return Err(StructureError::InvalidIntegerMatrixShape);
        }
        let image_rank = self.image_rank();
        // lift_mat' = (I - alpha*beta^T) * lift_mat.
        let mut lift_mat = Vec::new();
        lift_mat
            .try_reserve_exact(self.lift_mat.len())
            .map_err(|_| StructureError::AllocationFailed {
                requested: self.lift_mat.len(),
            })?;
        lift_mat.extend_from_slice(&self.lift_mat);
        for column in 0..image_rank {
            let mut beta_lift = 0_i64;
            for (k, &coefficient) in pairing_vector.iter().enumerate() {
                if coefficient == 0 {
                    continue;
                }
                let product = i64::from(coefficient)
                    .checked_mul(i64::from(self.lift_entry(k, column)))
                    .ok_or(StructureError::ArithmeticOverflow)?;
                beta_lift = beta_lift
                    .checked_add(product)
                    .ok_or(StructureError::ArithmeticOverflow)?;
            }
            for row in 0..rank {
                let correction = i64::from(reflected_vector[row])
                    .checked_mul(beta_lift)
                    .ok_or(StructureError::ArithmeticOverflow)?;
                let entry = i64::from(lift_mat[row * image_rank + column])
                    .checked_sub(correction)
                    .ok_or(StructureError::ArithmeticOverflow)?;
                lift_mat[row * image_rank + column] =
                    i32::try_from(entry).map_err(|_| StructureError::ArithmeticOverflow)?;
            }
        }

        // m_real' = m_real * (I - alpha*beta^T).
        let mut m_real = Vec::new();
        m_real
            .try_reserve_exact(self.m_real.len())
            .map_err(|_| StructureError::AllocationFailed {
                requested: self.m_real.len(),
            })?;
        m_real.extend_from_slice(&self.m_real);
        for row in 0..image_rank {
            let source = &self.m_real[row * rank..(row + 1) * rank];
            let mut row_alpha = 0_i64;
            for (k, &coefficient) in source.iter().enumerate() {
                if coefficient == 0 {
                    continue;
                }
                let product = i64::from(coefficient)
                    .checked_mul(i64::from(reflected_vector[k]))
                    .ok_or(StructureError::ArithmeticOverflow)?;
                row_alpha = row_alpha
                    .checked_add(product)
                    .ok_or(StructureError::ArithmeticOverflow)?;
            }
            for target in 0..rank {
                let correction = row_alpha
                    .checked_mul(i64::from(pairing_vector[target]))
                    .ok_or(StructureError::ArithmeticOverflow)?;
                let entry = i64::from(m_real[row * rank + target])
                    .checked_sub(correction)
                    .ok_or(StructureError::ArithmeticOverflow)?;
                m_real[row * rank + target] =
                    i32::try_from(entry).map_err(|_| StructureError::ArithmeticOverflow)?;
            }
        }

        Ok(Self {
            lift_mat,
            m_real,
            rank: self.rank,
            image_rank: self.image_rank,
        })
    }

    /// The transported pair across one cross edge by a simple reflection
    /// (upstream `InvolutionTable::add_cross`, involutions.cpp:242-243):
    /// `m_real := m_real * s` ("apply s before M_real") and
    /// `lift_mat := s * lift_mat` ("apply it after lift_mat"), where `s`
    /// is the reflection's weight-lattice matrix. The image basis is
    /// path-dependent — the transported basis differs from a fresh
    /// echelon reduction of `1-theta'` by column sign/order — which is
    /// exactly why upstream carries it in the record instead of
    /// recomputing. The factorization invariant is preserved:
    /// `(s*L)*(M*s) == s*(1-theta)*s == 1-theta'`.
    pub(crate) fn transported(&self, reflection: &[Vec<i32>]) -> Result<Self, StructureError> {
        let rank = reflection.len();
        if rank != self.rank() || reflection.iter().any(|row| row.len() != rank) {
            return Err(StructureError::InvalidIntegerMatrixShape);
        }
        let image_rank = self.image_rank();
        // lift_mat' = reflection * lift_mat (n x n times n x r).
        let mut lift_mat = Vec::new();
        lift_mat
            .try_reserve_exact(rank * image_rank)
            .map_err(|_| StructureError::AllocationFailed {
                requested: rank * image_rank,
            })?;
        for row in 0..rank {
            for column in 0..image_rank {
                let mut entry = 0_i64;
                for (k, &weight) in reflection[row].iter().enumerate() {
                    if weight == 0 {
                        continue;
                    }
                    let product = i64::from(weight)
                        .checked_mul(i64::from(self.lift_entry(k, column)))
                        .ok_or(StructureError::ArithmeticOverflow)?;
                    entry = entry
                        .checked_add(product)
                        .ok_or(StructureError::ArithmeticOverflow)?;
                }
                lift_mat.push(
                    i32::try_from(entry).map_err(|_| StructureError::ArithmeticOverflow)?,
                );
            }
        }
        // m_real' = m_real * reflection (r x n times n x n).
        let mut m_real = Vec::new();
        m_real
            .try_reserve_exact(image_rank * rank)
            .map_err(|_| StructureError::AllocationFailed {
                requested: image_rank * rank,
            })?;
        for row in self.m_real_rows() {
            for target in 0..rank {
                let mut entry = 0_i64;
                for (k, &coefficient) in row.iter().enumerate() {
                    if coefficient == 0 {
                        continue;
                    }
                    let product = i64::from(coefficient)
                        .checked_mul(i64::from(reflection[k][target]))
                        .ok_or(StructureError::ArithmeticOverflow)?;
                    entry = entry
                        .checked_add(product)
                        .ok_or(StructureError::ArithmeticOverflow)?;
                }
                m_real.push(
                    i32::try_from(entry).map_err(|_| StructureError::ArithmeticOverflow)?,
                );
            }
        }
        Ok(Self {
            lift_mat,
            m_real,
            rank: self.rank,
            image_rank: self.image_rank,
        })
    }

    /// `lift_mat * m_real == 1 - theta` (involutions.h:105).
    pub(crate) fn check_against(&self, theta: &LatticeInvolution) -> Result<(), StructureError> {
        let matrix = theta.weight_matrix();
        for (row_index, row) in matrix.iter().enumerate() {
            for (column_index, &entry) in row.iter().enumerate() {
                let mut product = 0_i64;
                for basis_index in 0..self.image_rank() {
                    product = product
                        .checked_add(
                            i64::from(self.lift_entry(row_index, basis_index))
                                .checked_mul(i64::from(
                                    self.m_real[basis_index * self.rank() + column_index],
                                ))
                                .ok_or(StructureError::ArithmeticOverflow)?,
                        )
                        .ok_or(StructureError::ArithmeticOverflow)?;
                }
                let expected = i64::from(row_index == column_index) - i64::from(entry);
                if product != expected {
                    return Err(StructureError::RepInvariantViolation {
                        invariant: "image basis factorization",
                    });
                }
            }
        }
        Ok(())
    }

    /// The image rank `r`: the number of `(1-theta)X^*` basis columns.
    pub(crate) fn image_rank(&self) -> usize {
        self.image_rank as usize
    }

    /// `(1-theta)*v` in image-basis coordinates: `M_real * v`
    /// (involutions.h:211).
    pub(crate) fn coordinates(&self, weight: &crate::Weight) -> Result<Vec<i64>, StructureError> {
        let mut result = Vec::new();
        result
            .try_reserve_exact(self.image_rank())
            .map_err(|_| StructureError::AllocationFailed {
                requested: self.image_rank(),
            })?;
        for row in self.m_real_rows() {
            let mut entry = 0_i64;
            for (&coefficient, &coordinate) in row.iter().zip(weight.as_slice()) {
                let product = i64::from(coefficient)
                    .checked_mul(i64::from(coordinate))
                    .ok_or(StructureError::ArithmeticOverflow)?;
                entry = entry
                    .checked_add(product)
                    .ok_or(StructureError::ArithmeticOverflow)?;
            }
            result.push(entry);
        }
        Ok(result)
    }

    /// `lift_mat * coordinates` back in `X^*` (involutions.cpp:346-356).
    pub(crate) fn lift(&self, coordinates: &[i64]) -> Result<Vec<i64>, StructureError> {
        let mut result = vec![0_i64; self.rank()];
        for (basis_index, &coordinate) in coordinates.iter().enumerate() {
            for (row, entry) in result.iter_mut().enumerate() {
                let product = i64::from(self.lift_entry(row, basis_index))
                    .checked_mul(coordinate)
                    .ok_or(StructureError::ArithmeticOverflow)?;
                *entry = entry
                    .checked_add(product)
                    .ok_or(StructureError::ArithmeticOverflow)?;
            }
        }
        Ok(result)
    }
}

fn identity_matrix(rank: usize) -> Result<Vec<Vec<i64>>, StructureError> {
    let mut matrix = Vec::new();
    matrix
        .try_reserve_exact(rank)
        .map_err(|_| StructureError::AllocationFailed { requested: rank })?;
    for row in 0..rank {
        let mut values = Vec::new();
        values
            .try_reserve_exact(rank)
            .map_err(|_| StructureError::AllocationFailed { requested: rank })?;
        values.resize(rank, 0);
        values[row] = 1;
        matrix.push(values);
    }
    Ok(matrix)
}

/// One elementary column operation `column_j += c * column_k` on `a` and
/// `col`, mirrored by the inverse row operation `row_k -= c * row_j` on
/// `col_inverse`, keeping `col * col_inverse == id` throughout.
fn gcd_sweep(
    a: &mut [Vec<i64>],
    col: &mut [Vec<i64>],
    row: usize,
    limit: usize,
) -> Result<i64, StructureError> {
    let dest = limit
        .checked_sub(1)
        .ok_or(StructureError::RepInvariantViolation {
            invariant: "echelon pivot column",
        })?;
    // Upstream `gcd` (matreduc.h:70-122) reduces a COPY of the partial row
    // and records the elementary operations in an `l x l` ops matrix that
    // `column_echelon` then applies to M and col at once (matreduc.h:143-144).
    // The oracle negates the LOCAL pivot and records `ops(mindex,mindex)=-1`;
    // together with the column operations and the final `swapColumns` this
    // lands on the pivot column that `column_apply` produces. The E6
    // involution-187 factorization only holds with that recorded sign.
    let mut local_row = a[row][..limit].to_vec();
    let mut active: Vec<usize> = Vec::new();
    let mut min = 0_i64;
    let mut mindex = 0_usize;
    for (column, &entry) in local_row.iter().enumerate() {
        if entry != 0 {
            active.push(column);
            let magnitude = entry.abs();
            if min == 0 || magnitude < min {
                min = magnitude;
                mindex = column;
            }
        }
    }
    if active.is_empty() {
        return Ok(0);
    }
    let mut ops = identity_matrix(limit)?;
    if local_row[mindex] < 0 {
        local_row[mindex] = -local_row[mindex];
        ops[mindex][mindex] = -1;
    }

    while active.len() > 1 {
        let current = mindex;
        let pivot = local_row[current];
        let mut survivors = Vec::new();
        survivors.try_reserve_exact(active.len()).map_err(|_| {
            StructureError::AllocationFailed {
                requested: active.len(),
            }
        })?;
        for &j in &active {
            if j == current {
                survivors.push(j);
                continue;
            }
            // C++ `arithmetic::divide` truncates toward zero.
            let quotient = local_row[j] / pivot;
            // ops: column j -= q * column current.
            for r in 0..limit {
                ops[r][j] = ops[r][j]
                    .checked_sub(
                        quotient
                            .checked_mul(ops[r][current])
                            .ok_or(StructureError::ArithmeticOverflow)?,
                    )
                    .ok_or(StructureError::ArithmeticOverflow)?;
            }
            local_row[j] = local_row[j]
                .checked_sub(
                    pivot
                        .checked_mul(quotient)
                        .ok_or(StructureError::ArithmeticOverflow)?,
                )
                .ok_or(StructureError::ArithmeticOverflow)?;
            if local_row[j] != 0 {
                survivors.push(j);
                if local_row[j] < min {
                    min = local_row[j];
                    mindex = j;
                }
            }
        }
        active = survivors;
    }

    if mindex != dest {
        for r in 0..limit {
            ops[r].swap(dest, mindex);
        }
    }

    // column_apply(M, ops, 0): M' = M * ops on the first `limit` columns.
    apply_column_ops(a, &ops, limit)?;
    apply_column_ops(col, &ops, limit)?;
    Ok(min)
}

/// `M' = M * ops` on the first `limit` columns (matrix.h:496-510).
fn apply_column_ops(
    matrix: &mut [Vec<i64>],
    ops: &[Vec<i64>],
    limit: usize,
) -> Result<(), StructureError> {
    let rows = matrix.len();
    let mut fresh = vec![vec![0_i64; limit]; rows];
    for c in 0..limit {
        for k in 0..limit {
            let weight = ops[k][c];
            if weight == 0 {
                continue;
            }
            for r in 0..rows {
                fresh[r][c] = fresh[r][c]
                    .checked_add(
                        weight
                            .checked_mul(matrix[r][k])
                            .ok_or(StructureError::ArithmeticOverflow)?,
                    )
                    .ok_or(StructureError::ArithmeticOverflow)?;
            }
        }
    }
    for r in 0..rows {
        for c in 0..limit {
            matrix[r][c] = fresh[r][c];
        }
    }
    Ok(())
}

/// Integer inverse of a unimodular matrix by Euclidean row reduction
/// (no scaling division: the pivots are reduced by row swaps and row
/// subtractions, which keeps every intermediate entry integral).
fn invert_integer_matrix(matrix: &[Vec<i64>]) -> Result<Vec<Vec<i64>>, StructureError> {
    let rank = matrix.len();
    if matrix.iter().any(|row| row.len() != rank) {
        return Err(StructureError::InvalidIntegerMatrixShape);
    }
    let mut augmented: Vec<Vec<i64>> = Vec::new();
    augmented
        .try_reserve_exact(rank)
        .map_err(|_| StructureError::AllocationFailed { requested: rank })?;
    for (row_index, row) in matrix.iter().enumerate() {
        let mut extended = row.to_vec();
        extended
            .try_reserve_exact(rank)
            .map_err(|_| StructureError::AllocationFailed { requested: rank })?;
        for column_index in 0..rank {
            extended.push(i64::from(row_index == column_index));
        }
        augmented.push(extended);
    }
    for pivot in 0..rank {
        let Some(best) = (pivot..rank)
            .filter(|&r| augmented[r][pivot] != 0)
            .min_by_key(|&r| augmented[r][pivot].abs())
        else {
            return Err(StructureError::RepInvariantViolation {
                invariant: "non-singular column operations matrix",
            });
        };
        augmented.swap(pivot, best);
        let mut changed = true;
        while changed {
            changed = false;
            for r in 0..rank {
                if r == pivot || augmented[r][pivot] == 0 {
                    continue;
                }
                if augmented[r][pivot].abs() >= augmented[pivot][pivot].abs() {
                    let quotient = augmented[r][pivot] / augmented[pivot][pivot];
                    if quotient != 0 {
                        for c in 0..2 * rank {
                            augmented[r][c] -= quotient * augmented[pivot][c];
                        }
                        changed = true;
                    }
                } else if augmented[pivot][pivot].abs() > 1 {
                    augmented.swap(pivot, r);
                    changed = true;
                }
            }
        }
    }
    for pivot in 0..rank {
        let diagonal = augmented[pivot][pivot];
        if diagonal == -1 {
            for entry in augmented[pivot].iter_mut() {
                *entry = -*entry;
            }
        } else if diagonal != 1 {
            return Err(StructureError::RepInvariantViolation {
                invariant: "unimodular column operations matrix",
            });
        }
    }
    let inverse: Vec<Vec<i64>> = augmented
        .into_iter()
        .map(|row| row[rank..].to_vec())
        .collect();
    for r in 0..rank {
        for c in 0..rank {
            let mut product = 0_i64;
            for k in 0..rank {
                product = product
                    .checked_add(
                        matrix[r][k]
                            .checked_mul(inverse[k][c])
                            .ok_or(StructureError::ArithmeticOverflow)?,
                    )
                    .ok_or(StructureError::ArithmeticOverflow)?;
            }
            if product != i64::from(r == c) {
                return Err(StructureError::RepInvariantViolation {
                    invariant: "integer matrix inverse",
                });
            }
        }
    }
    Ok(inverse)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_reflection_transport_matches_dense_matrix_transport() {
        let projection = RealProjection::from_nested(
            vec![vec![2, -1], vec![1, 3], vec![-2, 0]],
            vec![vec![1, 0, -1], vec![2, 1, 3]],
        )
        .unwrap();
        let alpha = [1, -1, 0];
        let beta = [0, 1, -1];
        let reflection = vec![
            vec![1, -1, 1],
            vec![0, 2, -1],
            vec![0, 0, 1],
        ];

        let expected = projection.transported(&reflection).unwrap();
        let actual = projection
            .transported_by_simple_reflection(&alpha, &beta)
            .unwrap();
        assert_eq!(actual, expected);
    }
}
