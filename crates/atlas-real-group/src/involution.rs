use std::sync::Arc;

use crate::{BasedRootDatum, Coweight, StructureError, Weight};

/// A pairing-preserving involution of a root datum's dual lattices.
///
/// The character and cocharacter actions are validated together, so pairing
/// preservation is an invariant instead of a convention imposed on callers;
/// only the character action is RETAINED. Pairing preservation pins the
/// cocharacter action to the character action's transpose (`C = (W^T)^-1`,
/// and `W^2 = I` makes that `W^T`), matching upstream, where an involution
/// record keeps a single `WeightInvolution` int_Matrix and reads the
/// cocharacter direction off the same storage (a right product, e.g.
/// `ratvec::symmetrise`). [`Self::coweight_matrix`] therefore returns a
/// zero-copy transposed view instead of a second matrix. This type
/// deliberately does not claim that the action preserves the finite root
/// system; [`crate::RootInvolutionData`] establishes that stronger property —
/// a root permutation that also transports stored coroots — against an
/// enumerated root system.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LatticeInvolution {
    datum: Arc<BasedRootDatum>,
    weight_action: Vec<Vec<i32>>,
    /// Row-major flat copy of `weight_action`: the apply loops read rows
    /// from one allocation instead of chasing per-row pointers.
    weight_flat: Vec<i32>,
    /// Row-major flat copy of the transpose: the cocharacter apply reads
    /// the stored matrix down its columns (a strided pointer chase —
    /// perf-unitary-3683612: apply_matrix_transposed_into 3.90% self);
    /// with one flat transposed copy it becomes sequential dot products.
    weight_transposed_flat: Vec<i32>,
}

fn flatten(matrix: &[Vec<i32>]) -> Vec<i32> {
    let rank = matrix.len();
    let mut flat = Vec::with_capacity(rank * rank);
    for row in matrix {
        flat.extend_from_slice(row);
    }
    flat
}

fn flatten_transposed(matrix: &[Vec<i32>]) -> Vec<i32> {
    let rank = matrix.len();
    let mut flat = vec![0; rank * rank];
    for (row, entries) in matrix.iter().enumerate() {
        for (column, &entry) in entries.iter().enumerate() {
            flat[column * rank + row] = entry;
        }
    }
    flat
}

impl LatticeInvolution {
    pub fn new(
        datum: &BasedRootDatum,
        weight_action: Vec<Vec<i32>>,
        coweight_action: Vec<Vec<i32>>,
    ) -> Result<Self, StructureError> {
        let rank = datum.lattice_rank();
        if !is_square_of_rank(&weight_action, rank) || !is_square_of_rank(&coweight_action, rank) {
            return Err(StructureError::InvalidInvolution);
        }
        if !is_identity_product(&weight_action, &weight_action)?
            || !is_identity_product(&coweight_action, &coweight_action)?
        {
            return Err(StructureError::InvalidInvolution);
        }
        if !preserves_pairing(&weight_action, &coweight_action)? {
            return Err(StructureError::InvalidRootAutomorphism);
        }
        let weight_flat = flatten(&weight_action);
        let weight_transposed_flat = flatten_transposed(&weight_action);
        Ok(Self {
            datum: Arc::new(datum.clone()),
            weight_action,
            weight_flat,
            weight_transposed_flat,
        })
    }

    pub fn identity(datum: &BasedRootDatum) -> Result<Self, StructureError> {
        let rank = datum.lattice_rank();
        let weight_action = identity_matrix(rank)?;
        let weight_flat = flatten(&weight_action);
        let weight_transposed_flat = weight_flat.clone();
        Ok(Self {
            datum: Arc::new(datum.clone()),
            weight_action,
            weight_flat,
            weight_transposed_flat,
        })
    }

    pub fn lattice_rank(&self) -> usize {
        self.weight_action.len()
    }

    /// `s_generator * theta * s_generator`, by reflection sparsity (rank^2,
    /// the [`crate::WeylAction`] simple-compose discipline).
    ///
    /// The involution table's cross edge `w |-> s w twist(s)` induces
    /// `theta |-> s theta s` — a PLAIN conjugation by the same generator,
    /// because `s_twist(s) * delta == delta * s` (the distinguished
    /// involution's simple-root permutation is itself an involution; see
    /// the phase-two comment in `InnerClass::involution_orbits`). Table
    /// records therefore transport theta across cross edges without
    /// materializing the Weyl factor's matrices at all. Only the character
    /// side is computed: the cocharacter conjugate `s_c C s_c` with
    /// `s_c = s^T` and `C = W^T` is exactly `(s W s)^T`, i.e. the transposed
    /// view of the result.
    pub(crate) fn conjugate_simple(&self, generator: usize) -> Result<Self, StructureError> {
        let datum = &*self.datum;
        let rank = self.lattice_rank();
        if generator >= datum.semisimple_rank() {
            return Err(StructureError::IndexOutOfRange {
                index: generator,
                upper_bound: datum.semisimple_rank(),
            });
        }
        let root = datum.simple_roots()[generator].as_slice();
        let coroot = datum.simple_coroots()[generator].as_slice();
        let weight_action = crate::weyl::reflection_right(
            root,
            coroot,
            &crate::weyl::reflection_left(root, coroot, &self.weight_action, rank)?,
            rank,
        )?;
        Ok(Self {
            datum: Arc::clone(&self.datum),
            weight_flat: flatten(&weight_action),
            weight_transposed_flat: flatten_transposed(&weight_action),
            weight_action,
        })
    }

    pub fn datum(&self) -> &BasedRootDatum {
        &self.datum
    }

    /// The owned datum handle for trusted constructors that share immutable
    /// storage with a Weyl action.
    pub(crate) fn datum_arc(&self) -> &Arc<BasedRootDatum> {
        &self.datum
    }

    pub fn weight_matrix(&self) -> &[Vec<i32>] {
        &self.weight_action
    }

    /// The cocharacter action as a zero-copy transposed view of the retained
    /// character action (`C = W^T`, see the type-level comment); upstream
    /// reads the same entries through `int_Matrix::transposed()`/right
    /// products off its single stored matrix.
    pub fn coweight_matrix(&self) -> CoweightMatrixView<'_> {
        CoweightMatrixView {
            storage: &self.weight_action,
        }
    }

    /// Rank of `X^*/ker(1-theta)`, computed from the `-1` eigenspace of this
    /// involution. This avoids a floating-point rank calculation.
    pub fn anti_invariant_rank(&self) -> Result<usize, StructureError> {
        let trace =
            self.weight_action
                .iter()
                .enumerate()
                .try_fold(0_i128, |sum, (index, row)| {
                    sum.checked_add(i128::from(row[index]))
                        .ok_or(StructureError::ArithmeticOverflow)
                })?;
        let twice_rank = i128::try_from(self.lattice_rank())
            .map_err(|_| StructureError::ArithmeticOverflow)?
            .checked_sub(trace)
            .ok_or(StructureError::ArithmeticOverflow)?;
        if twice_rank < 0 || twice_rank % 2 != 0 {
            return Err(StructureError::InvalidInvolution);
        }
        usize::try_from(twice_rank / 2).map_err(|_| StructureError::ArithmeticOverflow)
    }

    pub fn act_on_weight(&self, weight: &Weight) -> Result<Weight, StructureError> {
        apply_flat(&self.weight_flat, self.lattice_rank(), weight.as_slice()).map(Weight::new)
    }

    pub fn act_on_coweight(&self, coweight: &Coweight) -> Result<Coweight, StructureError> {
        apply_flat(&self.weight_transposed_flat, self.lattice_rank(), coweight.as_slice())
            .map(Coweight::new)
    }

    /// `act_on_weight` into a caller-owned buffer, for bulk loops (the
    /// root-classification pass pays one vector per root otherwise).
    pub(crate) fn act_on_weight_into(
        &self,
        coordinates: &[i32],
        out: &mut Vec<i32>,
    ) -> Result<(), StructureError> {
        apply_flat_into(&self.weight_flat, self.lattice_rank(), coordinates, out)
    }

    /// `act_on_coweight` into a caller-owned buffer (see
    /// [`Self::act_on_weight_into`]).
    pub(crate) fn act_on_coweight_into(
        &self,
        coordinates: &[i32],
        out: &mut Vec<i32>,
    ) -> Result<(), StructureError> {
        apply_flat_into(
            &self.weight_transposed_flat,
            self.lattice_rank(),
            coordinates,
            out,
        )
    }
}

/// Read-only transposed view of a retained character action: entry
/// `(row, column)` reads `storage[column][row]`.
///
/// A pairing-preserving involution's cocharacter action is the character
/// action's transpose, so the second matrix is never materialized; this is
/// the accessor half of that change (upstream serves the same direction with
/// right products off its single `int_Matrix`). Row iteration is a strided
/// read across the stored rows — at lattice ranks the whole matrix is a
/// handful of cache lines, so this is as cheap as the retired copy.
#[derive(Clone, Copy, Debug)]
pub struct CoweightMatrixView<'a> {
    storage: &'a [Vec<i32>],
}

/// One row of a [`CoweightMatrixView`] (a COLUMN of the stored matrix).
#[derive(Clone, Copy, Debug)]
pub struct CoweightRowView<'a> {
    storage: &'a [Vec<i32>],
    row: usize,
}

impl<'a> CoweightMatrixView<'a> {
    pub fn len(&self) -> usize {
        self.storage.len()
    }

    pub fn is_empty(&self) -> bool {
        self.storage.is_empty()
    }

    /// Entry `(row, column)`, widening the historic `matrix[row][column]`
    /// indexing of the retired stored matrix; panics out of range the same
    /// way.
    pub fn at(&self, row: usize, column: usize) -> i32 {
        self.storage[column][row]
    }

    pub fn iter(&self) -> CoweightMatrixIter<'a> {
        CoweightMatrixIter {
            storage: self.storage,
            next: 0,
        }
    }

    /// The transposed action as an owned matrix, for callers that transport
    /// it into new storage or a `&[Vec<i32>]`-typed helper.
    pub fn to_vec(&self) -> Vec<Vec<i32>> {
        self.iter().map(|row| row.to_vec()).collect()
    }
}

/// Row iterator of a [`CoweightMatrixView`].
#[derive(Clone, Debug)]
pub struct CoweightMatrixIter<'a> {
    storage: &'a [Vec<i32>],
    next: usize,
}

impl<'a> Iterator for CoweightMatrixIter<'a> {
    type Item = CoweightRowView<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let row = self.next;
        if row >= self.storage.len() {
            return None;
        }
        self.next += 1;
        Some(CoweightRowView {
            storage: self.storage,
            row,
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.storage.len() - self.next;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for CoweightMatrixIter<'_> {}

impl<'a> IntoIterator for CoweightMatrixView<'a> {
    type Item = CoweightRowView<'a>;
    type IntoIter = CoweightMatrixIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl PartialEq for CoweightMatrixView<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len()
            && self
                .iter()
                .zip(other.iter())
                .all(|(left, right)| left.iter().eq(right.iter()))
    }
}

impl Eq for CoweightMatrixView<'_> {}

impl PartialEq<Vec<Vec<i32>>> for CoweightMatrixView<'_> {
    fn eq(&self, other: &Vec<Vec<i32>>) -> bool {
        self.len() == other.len()
            && self
                .iter()
                .zip(other.iter())
                .all(|(row, entries)| row.iter().eq(entries.iter()))
    }
}

impl PartialEq<CoweightMatrixView<'_>> for Vec<Vec<i32>> {
    fn eq(&self, other: &CoweightMatrixView<'_>) -> bool {
        other == self
    }
}

impl<const N: usize> PartialEq<&[Vec<i32>; N]> for CoweightMatrixView<'_> {
    fn eq(&self, other: &&[Vec<i32>; N]) -> bool {
        self.len() == other.len()
            && self
                .iter()
                .zip(other.iter())
                .all(|(row, entries)| row.iter().eq(entries.iter()))
    }
}

impl<'a> CoweightRowView<'a> {
    pub fn len(&self) -> usize {
        self.storage.len()
    }

    pub fn is_empty(&self) -> bool {
        self.storage.is_empty()
    }

    pub fn iter(&self) -> CoweightRowIter<'a> {
        CoweightRowIter {
            storage: self.storage,
            row: self.row,
            next: 0,
        }
    }

    pub fn to_vec(&self) -> Vec<i32> {
        self.iter().copied().collect()
    }
}

/// Entry iterator of a [`CoweightRowView`].
#[derive(Clone, Debug)]
pub struct CoweightRowIter<'a> {
    storage: &'a [Vec<i32>],
    row: usize,
    next: usize,
}

impl<'a> Iterator for CoweightRowIter<'a> {
    type Item = &'a i32;

    fn next(&mut self) -> Option<Self::Item> {
        let column = self.next;
        if column >= self.storage.len() {
            return None;
        }
        self.next += 1;
        Some(&self.storage[column][self.row])
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.storage.len() - self.next;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for CoweightRowIter<'_> {}

impl<'a> IntoIterator for CoweightRowView<'a> {
    type Item = &'a i32;
    type IntoIter = CoweightRowIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl std::ops::Index<usize> for CoweightRowView<'_> {
    type Output = i32;

    fn index(&self, column: usize) -> &Self::Output {
        &self.storage[column][self.row]
    }
}

fn identity_matrix(rank: usize) -> Result<Vec<Vec<i32>>, StructureError> {
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

fn is_square_of_rank(matrix: &[Vec<i32>], rank: usize) -> bool {
    matrix.len() == rank && matrix.iter().all(|row| row.len() == rank)
}

fn is_identity_product(left: &[Vec<i32>], right: &[Vec<i32>]) -> Result<bool, StructureError> {
    for (row, _) in left.iter().enumerate() {
        for (column, _) in right.iter().enumerate() {
            let value = checked_sum(
                (0..left.len()).map(|middle| (left[row][middle], right[middle][column])),
            )?;
            if value != i128::from(row == column) {
                return Ok(false);
            }
        }
    }
    Ok(true)
}

fn preserves_pairing(
    weight_action: &[Vec<i32>],
    coweight_action: &[Vec<i32>],
) -> Result<bool, StructureError> {
    for (weight_coordinate, _) in weight_action.iter().enumerate() {
        for (coweight_coordinate, _) in coweight_action.iter().enumerate() {
            let value = checked_sum((0..weight_action.len()).map(|row| {
                (
                    weight_action[row][weight_coordinate],
                    coweight_action[row][coweight_coordinate],
                )
            }))?;
            if value != i128::from(weight_coordinate == coweight_coordinate) {
                return Ok(false);
            }
        }
    }
    Ok(true)
}

/// Apply a rank×rank flat row-major matrix: `out[i] = sum_j flat[i*rank+j]
/// * coordinates[j]`. The transposed direction applies the flat TRANSPOSE
/// copy, so both directions are sequential row reads. Accumulation uses
/// `checked_row_sum`'s i64 fast path with exact i128 fallback.
fn apply_flat(
    flat: &[i32],
    rank: usize,
    coordinates: &[i32],
) -> Result<Vec<i32>, StructureError> {
    let mut out = Vec::with_capacity(rank);
    apply_flat_into(flat, rank, coordinates, &mut out)?;
    Ok(out)
}

fn apply_flat_into(
    flat: &[i32],
    rank: usize,
    coordinates: &[i32],
    out: &mut Vec<i32>,
) -> Result<(), StructureError> {
    debug_assert_eq!(flat.len(), rank * rank);
    if coordinates.len() != rank {
        return Err(StructureError::RankMismatch {
            expected: rank,
            actual: coordinates.len(),
        });
    }
    out.clear();
    // `max(1)`: a rank-0 matrix is empty, so any chunk width yields no rows.
    for row in flat.chunks(rank.max(1)) {
        out.push(
            i32::try_from(checked_row_sum(row, coordinates)?)
                .map_err(|_| StructureError::ArithmeticOverflow)?,
        );
    }
    Ok(())
}

/// Row dot product: i64 fast path (an i32×i32 product never overflows
/// i64; only the accumulation can), falling back to the exact i128
/// accumulation on overflow so the overflow contract is unchanged.
fn checked_row_sum(row: &[i32], coordinates: &[i32]) -> Result<i128, StructureError> {
    let mut sum = 0_i64;
    for (&left, &right) in row.iter().zip(coordinates) {
        let product = i64::from(left) * i64::from(right);
        match sum.checked_add(product) {
            Some(next) => sum = next,
            None => {
                return checked_sum(row.iter().copied().zip(coordinates.iter().copied()));
            }
        }
    }
    Ok(i128::from(sum))
}

fn checked_sum(mut pairs: impl Iterator<Item = (i32, i32)>) -> Result<i128, StructureError> {
    pairs.try_fold(0_i128, |sum, (left, right)| {
        let product = i128::from(left)
            .checked_mul(i128::from(right))
            .ok_or(StructureError::ArithmeticOverflow)?;
        sum.checked_add(product)
            .ok_or(StructureError::ArithmeticOverflow)
    })
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::pair;

    #[test]
    fn conjugate_simple_equals_full_reflection_compose() {
        // B2 theta with a nontrivial Weyl factor: theta = s0*s1*s0 after -1
        // (the split distinguished involution), conjugated by each simple
        // reflection — the involution table's cross-edge transport — must
        // equal the dense compose s * theta * s on both lattices.
        let datum = BasedRootDatum::standard(vec![vec![2, -2], vec![-1, 2]]).unwrap();
        let group = crate::WeylGroup::new(datum.clone());
        let w = group
            .simple_reflection(0)
            .unwrap()
            .compose(&group.simple_reflection(1).unwrap())
            .unwrap()
            .compose(&group.simple_reflection(0).unwrap())
            .unwrap();
        let minus_one = vec![vec![-1, 0], vec![0, -1]];
        let delta = LatticeInvolution::new(&datum, minus_one.clone(), minus_one).unwrap();
        let theta = LatticeInvolution::new(
            &datum,
            crate::twisted_involution::compose_matrices(w.matrix(), delta.weight_matrix()).unwrap(),
            crate::twisted_involution::compose_matrices(
                w.coweight_matrix(),
                &delta.coweight_matrix().to_vec(),
            )
            .unwrap(),
        )
        .unwrap();
        for generator in 0..2 {
            let reflection = group.simple_reflection(generator).unwrap();
            let expected_weight = crate::twisted_involution::compose_matrices(
                &crate::twisted_involution::compose_matrices(
                    reflection.matrix(),
                    theta.weight_matrix(),
                )
                .unwrap(),
                reflection.matrix(),
            )
            .unwrap();
            let expected_coweight = crate::twisted_involution::compose_matrices(
                &crate::twisted_involution::compose_matrices(
                    reflection.coweight_matrix(),
                    &theta.coweight_matrix().to_vec(),
                )
                .unwrap(),
                reflection.coweight_matrix(),
            )
            .unwrap();
            let transported = theta.conjugate_simple(generator).unwrap();
            assert_eq!(transported.weight_matrix(), &expected_weight);
            assert_eq!(transported.coweight_matrix(), expected_coweight);
        }
    }

    #[test]
    fn coweight_view_reads_the_retained_matrix_transposed() {
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let weight_action = vec![vec![-1, 0], vec![1, 1]];
        let coweight_action = vec![vec![-1, 1], vec![0, 1]];
        let involution =
            LatticeInvolution::new(&datum, weight_action.clone(), coweight_action.clone()).unwrap();

        let view = involution.coweight_matrix();
        assert_eq!(view.len(), 2);
        assert!(!view.is_empty());
        assert_eq!(view.at(0, 1), 1);
        assert_eq!(view.at(1, 0), 0);
        assert_eq!(view.to_vec(), coweight_action);
        assert_eq!(
            view.iter().map(|row| row.to_vec()).collect::<Vec<_>>(),
            coweight_action
        );
        assert_eq!(view, coweight_action);
        assert_eq!(coweight_action, view);
        assert_eq!(view, &[vec![-1, 1], vec![0, 1]]);
        assert_eq!(view.iter().len(), 2);
        let row = view.iter().next().unwrap();
        assert_eq!(row.len(), 2);
        assert_eq!(row[1], 1);
        assert_eq!(row.iter().copied().collect::<Vec<_>>(), vec![-1, 1]);

        // The transposed apply matches the retired stored coweight matrix:
        // C * [7, -11] = [(-1)*7 + 1*(-11), 0*7 + 1*(-11)] = [-18, -11].
        let coweight = Coweight::new(vec![7, -11]);
        let applied = involution.act_on_coweight(&coweight).unwrap();
        assert_eq!(applied.as_slice(), &[-18, -11]);
        let mut out = Vec::new();
        involution
            .act_on_coweight_into(coweight.as_slice(), &mut out)
            .unwrap();
        assert_eq!(out, applied.as_slice());
    }

    #[test]
    fn dual_actions_preserve_pairing() {
        let datum = BasedRootDatum::from_simple_data(
            2,
            vec![vec![2]],
            vec![Weight::new(vec![1, 0])],
            vec![Coweight::new(vec![2, 0])],
        )
        .expect("rank-two A1 datum");
        let involution = LatticeInvolution::new(
            &datum,
            vec![vec![-1, 0], vec![0, 1]],
            vec![vec![-1, 0], vec![0, 1]],
        )
        .expect("dual sign involution");
        let weight = involution.act_on_weight(&Weight::new(vec![3, 5])).unwrap();
        let coweight = involution
            .act_on_coweight(&Coweight::new(vec![7, -11]))
            .unwrap();
        assert_eq!(pair(&weight, &coweight), Ok(-34));
    }

    #[test]
    fn rejects_actions_that_do_not_preserve_the_pairing() {
        let datum = BasedRootDatum::standard(vec![vec![2]]).unwrap();
        assert_eq!(
            LatticeInvolution::new(&datum, vec![vec![-1]], vec![vec![1]]),
            Err(StructureError::InvalidRootAutomorphism)
        );
    }

    #[test]
    fn rejects_non_involutive_actions() {
        let datum = BasedRootDatum::standard(vec![vec![2]]).unwrap();
        assert_eq!(
            LatticeInvolution::new(&datum, vec![vec![2]], vec![vec![0]]),
            Err(StructureError::InvalidInvolution)
        );
    }

    #[test]
    fn datum_arc_backing_preserves_value_equality() {
        let datum = BasedRootDatum::standard(vec![vec![2]]).unwrap();
        let left = LatticeInvolution::identity(&datum).unwrap();
        let right = LatticeInvolution::identity(&datum.clone()).unwrap();

        assert_eq!(left, right);
        assert!(!Arc::ptr_eq(left.datum_arc(), right.datum_arc()));
    }
}
