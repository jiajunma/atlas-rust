//! Matrix-level Weyl actions on the full character/cocharacter lattices.
//!
//! This is the provenance-bearing action representation: every
//! [`WeylAction`] carries its datum and acts on both lattices at once. The
//! word-level combinatorial substrate — elements with cached lengths,
//! descents, and reduced words on the root-permutation representation —
//! lives in `weyl_element.rs`; the two layers are mutually checkable
//! through [`RootSystem::action_permutation`].

use std::collections::{BTreeMap, VecDeque};

use crate::{BasedRootDatum, Coweight, RootId, RootSystem, StructureError, Weight};

/// A canonical Weyl-group action on the full character/cocharacter lattices.
///
/// Equality is equality of the matrix action, so equivalent generator words
/// have the same value without enumerating the Weyl group.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct WeylAction {
    datum: BasedRootDatum,
    weight_matrix: Vec<Vec<i32>>,
    coweight_matrix: Vec<Vec<i32>>,
}

impl WeylAction {
    pub fn identity(datum: &BasedRootDatum) -> Result<Self, StructureError> {
        let rank = datum.lattice_rank();
        Ok(Self {
            datum: datum.clone(),
            weight_matrix: identity_matrix(rank)?,
            coweight_matrix: identity_matrix(rank)?,
        })
    }

    pub fn simple_reflection(
        datum: &BasedRootDatum,
        generator: usize,
    ) -> Result<Self, StructureError> {
        let rank = datum.lattice_rank();
        let root = datum
            .simple_roots()
            .get(generator)
            .ok_or(StructureError::IndexOutOfRange {
                index: generator,
                upper_bound: datum.semisimple_rank(),
            })?;
        let coroot = &datum.simple_coroots()[generator];
        let weight_matrix = reflection_matrix(root.as_slice(), coroot.as_slice())?;
        let coweight_matrix = reflection_matrix(coroot.as_slice(), root.as_slice())?;
        if weight_matrix.len() != rank || coweight_matrix.len() != rank {
            return Err(StructureError::RankMismatch {
                expected: rank,
                actual: weight_matrix.len(),
            });
        }
        Ok(Self {
            datum: datum.clone(),
            weight_matrix,
            coweight_matrix,
        })
    }

    /// The reflection in an arbitrary enumerated root, acting on both
    /// lattices.
    ///
    /// [`Self::simple_reflection`] covers only simple roots; replaying a
    /// Cayley/cross decomposition needs reflections in arbitrary strongly
    /// orthogonal roots. Sign is irrelevant: a root and its negative define
    /// the same reflection.
    pub fn root_reflection(
        datum: &BasedRootDatum,
        root_system: &RootSystem,
        root: RootId,
    ) -> Result<Self, StructureError> {
        if root_system.datum() != datum {
            return Err(StructureError::DatumMismatch);
        }
        let weight = root_system
            .root(root)
            .ok_or(StructureError::IndexOutOfRange {
                index: root.0,
                upper_bound: root_system.roots().len(),
            })?;
        let coroot = root_system
            .coroot(root)
            .ok_or(StructureError::IndexOutOfRange {
                index: root.0,
                upper_bound: root_system.roots().len(),
            })?;
        Ok(Self {
            datum: datum.clone(),
            weight_matrix: reflection_matrix(weight.as_slice(), coroot.as_slice())?,
            coweight_matrix: reflection_matrix(coroot.as_slice(), weight.as_slice())?,
        })
    }

    /// Return the composite `self after right`.
    pub fn compose(&self, right: &Self) -> Result<Self, StructureError> {
        if self.datum != right.datum {
            return Err(StructureError::DatumMismatch);
        }
        let rank = self.rank();
        if right.rank() != rank {
            return Err(StructureError::RankMismatch {
                expected: rank,
                actual: right.rank(),
            });
        }
        Ok(Self {
            datum: self.datum.clone(),
            weight_matrix: compose_matrices(&self.weight_matrix, &right.weight_matrix)?,
            coweight_matrix: compose_matrices(&self.coweight_matrix, &right.coweight_matrix)?,
        })
    }

    pub fn act(&self, weight: &Weight) -> Result<Weight, StructureError> {
        apply_matrix(&self.weight_matrix, weight.as_slice()).map(Weight::new)
    }

    pub fn act_on_coweight(&self, coweight: &Coweight) -> Result<Coweight, StructureError> {
        apply_matrix(&self.coweight_matrix, coweight.as_slice()).map(Coweight::new)
    }

    pub fn rank(&self) -> usize {
        self.weight_matrix.len()
    }

    pub fn datum(&self) -> &BasedRootDatum {
        &self.datum
    }

    pub fn matrix(&self) -> &[Vec<i32>] {
        &self.weight_matrix
    }

    pub fn coweight_matrix(&self) -> &[Vec<i32>] {
        &self.coweight_matrix
    }
}

fn identity_matrix(rank: usize) -> Result<Vec<Vec<i32>>, StructureError> {
    let mut matrix = zero_matrix(rank)?;
    for (index, row) in matrix.iter_mut().enumerate() {
        row[index] = 1;
    }
    Ok(matrix)
}

fn zero_matrix(rank: usize) -> Result<Vec<Vec<i32>>, StructureError> {
    let mut matrix = Vec::new();
    matrix
        .try_reserve_exact(rank)
        .map_err(|_| StructureError::AllocationFailed { requested: rank })?;
    for _ in 0..rank {
        let mut row = Vec::new();
        row.try_reserve_exact(rank)
            .map_err(|_| StructureError::AllocationFailed { requested: rank })?;
        row.resize(rank, 0);
        matrix.push(row);
    }
    Ok(matrix)
}

fn reflection_matrix(
    reflected_vector: &[i32],
    pairing_vector: &[i32],
) -> Result<Vec<Vec<i32>>, StructureError> {
    let rank = reflected_vector.len();
    if pairing_vector.len() != rank {
        return Err(StructureError::RankMismatch {
            expected: rank,
            actual: pairing_vector.len(),
        });
    }
    let mut matrix = zero_matrix(rank)?;
    for (row, target_row) in matrix.iter_mut().enumerate() {
        for (column, entry) in target_row.iter_mut().enumerate() {
            let identity = i128::from(row == column);
            let correction = i128::from(reflected_vector[row])
                .checked_mul(i128::from(pairing_vector[column]))
                .ok_or(StructureError::ArithmeticOverflow)?;
            *entry = i32::try_from(
                identity
                    .checked_sub(correction)
                    .ok_or(StructureError::ArithmeticOverflow)?,
            )
            .map_err(|_| StructureError::ArithmeticOverflow)?;
        }
    }
    Ok(matrix)
}

fn compose_matrices(
    left: &[Vec<i32>],
    right: &[Vec<i32>],
) -> Result<Vec<Vec<i32>>, StructureError> {
    let rank = left.len();
    if right.len() != rank
        || left.iter().any(|row| row.len() != rank)
        || right.iter().any(|row| row.len() != rank)
    {
        return Err(StructureError::RankMismatch {
            expected: rank,
            actual: right.len(),
        });
    }
    let mut matrix = zero_matrix(rank)?;
    for (row, target_row) in matrix.iter_mut().enumerate() {
        for (column, entry) in target_row.iter_mut().enumerate() {
            *entry = checked_dot(&left[row], |middle| right[middle][column])?;
        }
    }
    Ok(matrix)
}

fn apply_matrix(matrix: &[Vec<i32>], coordinates: &[i32]) -> Result<Vec<i32>, StructureError> {
    let rank = matrix.len();
    if coordinates.len() != rank {
        return Err(StructureError::RankMismatch {
            expected: rank,
            actual: coordinates.len(),
        });
    }
    if matrix.iter().any(|row| row.len() != rank) {
        return Err(StructureError::InvalidRootAutomorphism);
    }
    let mut result = Vec::new();
    result
        .try_reserve_exact(rank)
        .map_err(|_| StructureError::AllocationFailed { requested: rank })?;
    for row in matrix {
        result.push(checked_dot(row, |column| coordinates[column])?);
    }
    Ok(result)
}

/// Lazy Weyl-group operations attached to one root datum.
///
/// The type creates generators and canonical actions on demand. It does not
/// enumerate all group elements or encode a global order limit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WeylGroup {
    datum: BasedRootDatum,
}

impl WeylGroup {
    pub fn new(datum: BasedRootDatum) -> Self {
        Self { datum }
    }

    pub fn datum(&self) -> &BasedRootDatum {
        &self.datum
    }

    pub fn identity(&self) -> Result<WeylAction, StructureError> {
        WeylAction::identity(&self.datum)
    }

    pub fn simple_reflection(&self, generator: usize) -> Result<WeylAction, StructureError> {
        WeylAction::simple_reflection(&self.datum, generator)
    }

    /// Enumerate canonical Weyl actions with an explicit cardinality budget.
    ///
    /// This is intentionally separate from construction. Results are ordered
    /// lexicographically by their character-lattice action matrix.
    pub fn enumerate_actions(&self, budget: usize) -> Result<Vec<WeylAction>, StructureError> {
        let mut actions = BTreeMap::new();
        let mut pending = VecDeque::new();
        insert_action(self.identity()?, budget, &mut actions, &mut pending)?;
        let generators = (0..self.datum.semisimple_rank())
            .map(|generator| self.simple_reflection(generator))
            .collect::<Result<Vec<_>, _>>()?;
        while let Some(action) = pending.pop_front() {
            for generator in &generators {
                insert_action(
                    generator.compose(&action)?,
                    budget,
                    &mut actions,
                    &mut pending,
                )?;
            }
        }
        Ok(actions.into_values().collect())
    }
}

fn insert_action(
    action: WeylAction,
    budget: usize,
    actions: &mut BTreeMap<Vec<Vec<i32>>, WeylAction>,
    pending: &mut VecDeque<WeylAction>,
) -> Result<(), StructureError> {
    let key = action.matrix().to_vec();
    if actions.contains_key(&key) {
        return Ok(());
    }
    if actions.len() == budget {
        return Err(StructureError::ResourceLimitExceeded { limit: budget });
    }
    actions.insert(key, action.clone());
    pending.push_back(action);
    Ok(())
}

fn checked_dot(row: &[i32], mut right_at: impl FnMut(usize) -> i32) -> Result<i32, StructureError> {
    let value = row
        .iter()
        .enumerate()
        .try_fold(0_i128, |sum, (index, &left)| {
            let product = i128::from(left)
                .checked_mul(i128::from(right_at(index)))
                .ok_or(StructureError::ArithmeticOverflow)?;
            sum.checked_add(product)
                .ok_or(StructureError::ArithmeticOverflow)
        })?;
    i32::try_from(value).map_err(|_| StructureError::ArithmeticOverflow)
}

#[cfg(test)]
mod tests {
    use crate::pair;

    use super::*;

    #[test]
    fn simple_reflection_uses_the_full_lattice_action() {
        let datum = BasedRootDatum::from_simple_data(
            2,
            vec![vec![2]],
            vec![Weight::new(vec![1, 0])],
            vec![crate::Coweight::new(vec![2, 0])],
        )
        .expect("rank-two A1 datum");
        let reflection = WeylGroup::new(datum).simple_reflection(0).unwrap();
        assert_eq!(
            reflection.act(&Weight::new(vec![3, 5])),
            Ok(Weight::new(vec![-3, 5]))
        );
        assert_eq!(
            reflection.act_on_coweight(&crate::Coweight::new(vec![7, 11])),
            Ok(crate::Coweight::new(vec![-7, 11]))
        );
    }

    #[test]
    fn braid_related_words_have_the_same_canonical_action() {
        let group = WeylGroup::new(
            BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).expect("A2 datum"),
        );
        let first = group.simple_reflection(0).unwrap();
        let second = group.simple_reflection(1).unwrap();
        let left = first.compose(&second).unwrap().compose(&first).unwrap();
        let right = second.compose(&first).unwrap().compose(&second).unwrap();
        assert_eq!(left, right);
    }

    #[test]
    fn dual_actions_preserve_pairing_for_a_nonsymmetric_cartan_matrix() {
        let datum = BasedRootDatum::standard(vec![vec![2, -2], vec![-1, 2]]).unwrap();
        let action = WeylGroup::new(datum).simple_reflection(0).unwrap();
        let weight = Weight::new(vec![3, 5]);
        let coweight = Coweight::new(vec![7, 11]);

        assert_eq!(
            pair(
                &action.act(&weight).unwrap(),
                &action.act_on_coweight(&coweight).unwrap()
            ),
            pair(&weight, &coweight)
        );
    }

    #[test]
    fn generators_are_checked_against_semisimple_rank() {
        let group =
            WeylGroup::new(BasedRootDatum::from_simple_data(2, vec![], vec![], vec![]).unwrap());
        assert_eq!(
            group.simple_reflection(0),
            Err(StructureError::IndexOutOfRange {
                index: 0,
                upper_bound: 0,
            })
        );
    }

    #[test]
    fn enumerates_a2_actions_in_a_caller_supplied_budget() {
        let group =
            WeylGroup::new(BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap());
        let actions = group.enumerate_actions(6).unwrap();
        assert_eq!(actions.len(), 6);
        assert_eq!(
            group.enumerate_actions(5),
            Err(StructureError::ResourceLimitExceeded { limit: 5 })
        );
    }

    #[test]
    fn refuses_to_compose_actions_from_different_same_rank_data() {
        let a2 = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let b2 = BasedRootDatum::standard(vec![vec![2, -2], vec![-1, 2]]).unwrap();
        let a2_action = WeylGroup::new(a2).identity().unwrap();
        let b2_action = WeylGroup::new(b2).identity().unwrap();

        assert_eq!(
            a2_action.compose(&b2_action),
            Err(StructureError::DatumMismatch)
        );
    }

    #[test]
    fn reports_overflow_while_constructing_an_unrepresentable_lattice_action() {
        let datum = BasedRootDatum::from_simple_data(
            2,
            vec![vec![2]],
            vec![Weight::new(vec![i32::MAX, 1])],
            vec![Coweight::new(vec![1, 2 - i32::MAX])],
        )
        .unwrap();

        assert_eq!(
            WeylGroup::new(datum).simple_reflection(0),
            Err(StructureError::ArithmeticOverflow)
        );
    }
}
