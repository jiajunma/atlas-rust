//! Matrix-level Weyl actions on the full character/cocharacter lattices.
//!
//! This is the provenance-bearing action representation: every
//! [`WeylAction`] carries its datum and acts on both lattices at once. The
//! word-level combinatorial substrate — elements with cached lengths,
//! descents, and reduced words on the root-permutation representation —
//! lives in `weyl_element.rs`; the two layers are mutually checkable
//! through [`RootSystem::action_permutation`].

use std::collections::VecDeque;

use crate::{BasedRootDatum, Coweight, RootId, RootSystem, StructureError, Weight};

/// A canonical Weyl-group action on the full character/cocharacter lattices.
///
/// Equality is equality of the matrix action, so equivalent generator words
/// have the same value without enumerating the Weyl group.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct WeylAction {
    datum: std::sync::Arc<BasedRootDatum>,
    weight_matrix: Vec<Vec<i32>>,
    coweight_matrix: Vec<Vec<i32>>,
}

impl WeylAction {
    pub fn identity(datum: &BasedRootDatum) -> Result<Self, StructureError> {
        let rank = datum.lattice_rank();
        Ok(Self {
            datum: std::sync::Arc::new(datum.clone()),
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
            datum: std::sync::Arc::new(datum.clone()),
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
            datum: std::sync::Arc::new(datum.clone()),
            weight_matrix: reflection_matrix(weight.as_slice(), coroot.as_slice())?,
            coweight_matrix: reflection_matrix(coroot.as_slice(), weight.as_slice())?,
        })
    }

    /// Return the composite `self after right`.
    pub fn compose(&self, right: &Self) -> Result<Self, StructureError> {
        // Compose chains clone the SAME Arc, so the pointer fast path skips
        // a deep datum compare per compose; distinct allocations still
        // compare contents (semantics unchanged).
        if !std::sync::Arc::ptr_eq(&self.datum, &right.datum) && self.datum != right.datum {
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

    /// Composition without shape checks, used by the enumeration hot loop
    /// (the caller guarantees matching ranks and square matrices).
    pub(crate) fn compose_fast(&self, right: &Self) -> Self {
        Self {
            datum: std::sync::Arc::clone(&self.datum),
            weight_matrix: compose_matrices_fast(&self.weight_matrix, &right.weight_matrix),
            coweight_matrix: compose_matrices_fast(&self.coweight_matrix, &right.coweight_matrix),
        }
    }

    /// Recover the Weyl factor `w` of a twisted involution from its composed
    /// involution `theta = w after delta` and the distinguished involution:
    /// `w = theta * delta` (delta is an involution), on each lattice in the
    /// stored compose order. Table records drop `w`'s matrices; callers that
    /// still need the action (canonicalization word threading) rehydrate it
    /// here — one rank^3 compose per lattice, only away from the BFS.
    pub(crate) fn from_theta_factor(
        theta: &crate::LatticeInvolution,
        distinguished: &crate::LatticeInvolution,
    ) -> Result<Self, StructureError> {
        if theta.datum() != distinguished.datum() {
            return Err(StructureError::DatumMismatch);
        }
        let rank = theta.lattice_rank();
        if distinguished.lattice_rank() != rank {
            return Err(StructureError::RankMismatch {
                expected: rank,
                actual: distinguished.lattice_rank(),
            });
        }
        Ok(Self {
            datum: theta.datum_arc().clone(),
            weight_matrix: compose_matrices(theta.weight_matrix(), distinguished.weight_matrix())?,
            coweight_matrix: compose_matrices(
                theta.coweight_matrix(),
                distinguished.coweight_matrix(),
            )?,
        })
    }

    /// `s_generator after self` (matrix `s * M`), by reflection sparsity:
    /// `s = I - alpha*beta^T` gives `(s*M)[i][j] = M[i][j] - alpha_i *
    /// (beta^T M)[j]`, rank^2 instead of rank^3 per lattice. Exact integer
    /// equality with
    /// `WeylAction::simple_reflection(datum, generator)?.compose(self)`.
    /// Production moved to [`crate::LatticeInvolution::conjugate_simple`]
    /// (theta transport without the Weyl factor); retained for the pinning
    /// test against the full compose path.
    #[cfg(test)]
    pub(crate) fn left_compose_simple(&self, generator: usize) -> Result<Self, StructureError> {
        let rank = self.rank();
        let datum = &*self.datum;
        if generator >= datum.semisimple_rank() {
            return Err(StructureError::IndexOutOfRange {
                index: generator,
                upper_bound: datum.semisimple_rank(),
            });
        }
        Ok(Self {
            datum: std::sync::Arc::clone(&self.datum),
            weight_matrix: reflection_left(
                datum.simple_roots()[generator].as_slice(),
                datum.simple_coroots()[generator].as_slice(),
                &self.weight_matrix,
                rank,
            )?,
            coweight_matrix: reflection_left(
                datum.simple_coroots()[generator].as_slice(),
                datum.simple_roots()[generator].as_slice(),
                &self.coweight_matrix,
                rank,
            )?,
        })
    }

    /// `self after s_generator` (matrix `M * s`), by reflection sparsity:
    /// `(M*s)[i][j] = M[i][j] - (M*alpha)[i] * beta_j`. Exact integer
    /// equality with `self.compose(&WeylAction::simple_reflection(datum,
    /// generator)?)` (see [`Self::left_compose_simple`]). Test-only, like
    /// [`Self::left_compose_simple`].
    #[cfg(test)]
    pub(crate) fn right_compose_simple(&self, generator: usize) -> Result<Self, StructureError> {
        let rank = self.rank();
        let datum = &*self.datum;
        if generator >= datum.semisimple_rank() {
            return Err(StructureError::IndexOutOfRange {
                index: generator,
                upper_bound: datum.semisimple_rank(),
            });
        }
        Ok(Self {
            datum: std::sync::Arc::clone(&self.datum),
            weight_matrix: reflection_right(
                datum.simple_roots()[generator].as_slice(),
                datum.simple_coroots()[generator].as_slice(),
                &self.weight_matrix,
                rank,
            )?,
            coweight_matrix: reflection_right(
                datum.simple_coroots()[generator].as_slice(),
                datum.simple_roots()[generator].as_slice(),
                &self.coweight_matrix,
                rank,
            )?,
        })
    }

    /// The shared datum behind this action (Arc refcount bump).
    pub fn datum_arc(&self) -> &std::sync::Arc<BasedRootDatum> {
        &self.datum
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
    // Weyl matrices have small entries (Cartan-bounded); accumulate in i64
    // with no per-entry overflow checks. `checked_dot` remains for the
    // non-Weyl paths that still need it.
    for (row, target_row) in matrix.iter_mut().enumerate() {
        for (column, entry) in target_row.iter_mut().enumerate() {
            let mut sum: i64 = 0;
            for (index, &left) in left[row].iter().enumerate() {
                sum += i64::from(left) * i64::from(right[index][column]);
            }
            *entry = sum as i32;
        }
    }
    Ok(matrix)
}

fn compose_matrices_fast(left: &[Vec<i32>], right: &[Vec<i32>]) -> Vec<Vec<i32>> {
    let rank = left.len();
    let mut matrix = vec![vec![0_i32; rank]; rank];
    for (row, target_row) in matrix.iter_mut().enumerate() {
        for (column, entry) in target_row.iter_mut().enumerate() {
            let mut sum: i64 = 0;
            for (index, &left_entry) in left[row].iter().enumerate() {
                sum += i64::from(left_entry) * i64::from(right[index][column]);
            }
            *entry = sum as i32;
        }
    }
    matrix
}

/// `(s * M)` for the reflection `s = I - alpha*beta^T`, rank^2 via the
/// shared row vector `beta^T M`. Same small-entry i64 accumulation
/// discipline as [`compose_matrices`].
pub(crate) fn reflection_left(
    alpha: &[i32],
    beta: &[i32],
    matrix: &[Vec<i32>],
    rank: usize,
) -> Result<Vec<Vec<i32>>, StructureError> {
    if matrix.len() != rank
        || alpha.len() != rank
        || beta.len() != rank
        || matrix.iter().any(|row| row.len() != rank)
    {
        return Err(StructureError::RankMismatch {
            expected: rank,
            actual: matrix.len(),
        });
    }
    let mut pairing_row = vec![0_i64; rank];
    for (column, entry) in pairing_row.iter_mut().enumerate() {
        let mut sum: i64 = 0;
        for (index, &beta_entry) in beta.iter().enumerate() {
            sum += i64::from(beta_entry) * i64::from(matrix[index][column]);
        }
        *entry = sum;
    }
    let mut result = zero_matrix(rank)?;
    for (row, target_row) in result.iter_mut().enumerate() {
        for (column, entry) in target_row.iter_mut().enumerate() {
            *entry = (i64::from(matrix[row][column])
                - i64::from(alpha[row]) * pairing_row[column]) as i32;
        }
    }
    Ok(result)
}

/// `(M * s)` for the reflection `s = I - alpha*beta^T`, rank^2 via the
/// shared column vector `M * alpha` (see [`reflection_left`]).
pub(crate) fn reflection_right(
    alpha: &[i32],
    beta: &[i32],
    matrix: &[Vec<i32>],
    rank: usize,
) -> Result<Vec<Vec<i32>>, StructureError> {
    if matrix.len() != rank
        || alpha.len() != rank
        || beta.len() != rank
        || matrix.iter().any(|row| row.len() != rank)
    {
        return Err(StructureError::RankMismatch {
            expected: rank,
            actual: matrix.len(),
        });
    }
    let mut result = zero_matrix(rank)?;
    for (row, target_row) in result.iter_mut().enumerate() {
        let mut image: i64 = 0;
        for (index, &entry) in matrix[row].iter().enumerate() {
            image += i64::from(entry) * i64::from(alpha[index]);
        }
        for (column, entry) in target_row.iter_mut().enumerate() {
            *entry = (i64::from(matrix[row][column]) - image * i64::from(beta[column])) as i32;
        }
    }
    Ok(result)
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
        // Enumerate in the compact (transducer) representation — orders of
        // magnitude cheaper than the matrix BFS (E6: ~50ms vs ~1.1s) — then
        // materialize the matrices in parallel.
        let cartan: Vec<Vec<i32>> = self
            .datum
            .cartan_matrix()
            .iter()
            .map(|row| row.to_vec())
            .collect();
        let compact = crate::weyl_transducer::CompactWeyl::new(&cartan)?;
        let elements = compact.enumerate(budget)?;
        let reflections = self.reflections()?;
        let piece_matrices = compact.piece_matrices(&reflections)?;
        use rayon::prelude::*;
        elements
            .par_iter()
            .map(|elt| {
                let mut action = WeylAction::identity(&self.datum)?;
                for piece_index in 0..self.datum.semisimple_rank() {
                    action = action
                        .compose_fast(&piece_matrices[piece_index][elt[piece_index] as usize]);
                }
                Ok(action)
            })
            .collect()
    }

    fn reflections(&self) -> Result<Vec<WeylAction>, StructureError> {
        (0..self.datum.semisimple_rank())
            .map(|generator| WeylAction::simple_reflection(&self.datum, generator))
            .collect()
    }
}

fn insert_action(
    action: WeylAction,
    budget: usize,
    actions: &mut std::collections::HashMap<Vec<i32>, WeylAction>,
    pending: &mut VecDeque<WeylAction>,
) -> Result<(), StructureError> {
    let key: Vec<i32> = action.matrix().iter().flatten().copied().collect();
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
    fn sparse_simple_composes_equal_the_full_compose_path() {
        let datum =
            BasedRootDatum::standard(vec![vec![2, -2], vec![-1, 2]]).expect("B2 datum");
        let group = WeylGroup::new(datum);
        let generators = [group.simple_reflection(0).unwrap(), group.simple_reflection(1).unwrap()];
        let element = generators[0]
            .compose(&generators[1])
            .unwrap()
            .compose(&generators[0])
            .unwrap();
        for (generator, simple) in generators.iter().enumerate() {
            assert_eq!(
                element.left_compose_simple(generator).unwrap(),
                simple.compose(&element).unwrap(),
            );
            assert_eq!(
                element.right_compose_simple(generator).unwrap(),
                element.compose(simple).unwrap(),
            );
        }
    }

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
