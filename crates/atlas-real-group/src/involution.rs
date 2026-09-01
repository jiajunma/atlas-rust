use std::sync::Arc;

use crate::{BasedRootDatum, Coweight, StructureError, Weight};

/// A pairing-preserving involution of a root datum's dual lattices.
///
/// The character and cocharacter actions are stored separately and validated
/// together, so pairing preservation is an invariant instead of a convention
/// imposed on callers. This type deliberately does not claim that the action
/// preserves the finite root system; [`crate::RootInvolutionData`] establishes
/// that stronger property — a root permutation that also transports stored
/// coroots — against an enumerated root system.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LatticeInvolution {
    datum: Arc<BasedRootDatum>,
    weight_action: Vec<Vec<i32>>,
    coweight_action: Vec<Vec<i32>>,
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
        Ok(Self {
            datum: Arc::new(datum.clone()),
            weight_action,
            coweight_action,
        })
    }

    pub fn identity(datum: &BasedRootDatum) -> Result<Self, StructureError> {
        let rank = datum.lattice_rank();
        Ok(Self {
            datum: Arc::new(datum.clone()),
            weight_action: identity_matrix(rank)?,
            coweight_action: identity_matrix(rank)?,
        })
    }

    pub fn lattice_rank(&self) -> usize {
        self.weight_action.len()
    }

    /// `s_generator * theta * s_generator` on both lattices, by reflection
    /// sparsity (rank^2 per side per lattice, the [`crate::WeylAction`]
    /// simple-compose discipline).
    ///
    /// The involution table's cross edge `w |-> s w twist(s)` induces
    /// `theta |-> s theta s` — a PLAIN conjugation by the same generator,
    /// because `s_twist(s) * delta == delta * s` (the distinguished
    /// involution's simple-root permutation is itself an involution; see
    /// the phase-two comment in `InnerClass::involution_orbits`). Table
    /// records therefore transport theta across cross edges without
    /// materializing the Weyl factor's matrices at all.
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
        Ok(Self {
            datum: Arc::clone(&self.datum),
            weight_action: crate::weyl::reflection_right(
                root,
                coroot,
                &crate::weyl::reflection_left(root, coroot, &self.weight_action, rank)?,
                rank,
            )?,
            coweight_action: crate::weyl::reflection_right(
                coroot,
                root,
                &crate::weyl::reflection_left(coroot, root, &self.coweight_action, rank)?,
                rank,
            )?,
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

    pub fn coweight_matrix(&self) -> &[Vec<i32>] {
        &self.coweight_action
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
        apply_matrix(&self.weight_action, weight.as_slice()).map(Weight::new)
    }

    pub fn act_on_coweight(&self, coweight: &Coweight) -> Result<Coweight, StructureError> {
        apply_matrix(&self.coweight_action, coweight.as_slice()).map(Coweight::new)
    }

    /// `act_on_weight` into a caller-owned buffer, for bulk loops (the
    /// root-classification pass pays one vector per root otherwise).
    pub(crate) fn act_on_weight_into(
        &self,
        coordinates: &[i32],
        out: &mut Vec<i32>,
    ) -> Result<(), StructureError> {
        apply_matrix_into(&self.weight_action, coordinates, out)
    }

    /// `act_on_coweight` into a caller-owned buffer (see
    /// [`Self::act_on_weight_into`]).
    pub(crate) fn act_on_coweight_into(
        &self,
        coordinates: &[i32],
        out: &mut Vec<i32>,
    ) -> Result<(), StructureError> {
        apply_matrix_into(&self.coweight_action, coordinates, out)
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

fn apply_matrix(matrix: &[Vec<i32>], coordinates: &[i32]) -> Result<Vec<i32>, StructureError> {
    let mut out = Vec::with_capacity(coordinates.len());
    apply_matrix_into(matrix, coordinates, &mut out)?;
    Ok(out)
}

fn apply_matrix_into(
    matrix: &[Vec<i32>],
    coordinates: &[i32],
    out: &mut Vec<i32>,
) -> Result<(), StructureError> {
    if matrix.len() != coordinates.len() {
        return Err(StructureError::RankMismatch {
            expected: matrix.len(),
            actual: coordinates.len(),
        });
    }
    out.clear();
    for row in matrix {
        if row.len() != coordinates.len() {
            return Err(StructureError::InvalidInvolution);
        }
        out.push(
            i32::try_from(checked_sum(
                row.iter().copied().zip(coordinates.iter().copied()),
            )?)
            .map_err(|_| StructureError::ArithmeticOverflow)?,
        );
    }
    Ok(())
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
                delta.coweight_matrix(),
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
                    theta.coweight_matrix(),
                )
                .unwrap(),
                reflection.coweight_matrix(),
            )
            .unwrap();
            let transported = theta.conjugate_simple(generator).unwrap();
            assert_eq!(transported.weight_matrix(), &expected_weight);
            assert_eq!(transported.coweight_matrix(), &expected_coweight);
        }
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
