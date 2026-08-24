use crate::{
    BasedRootDatum, LatticeInvolution, RestrictedRootSystem, RootInvolutionData, RootSystem,
    StructureError, WeylAction,
};

/// A Weyl translate of the distinguished involution that is again involutive.
///
/// Atlas Cartan classes are ultimately indexed by canonical representatives of
/// their twisted-conjugacy orbits. This value only establishes the
/// root-theoretic condition `w theta` squared equals one; the Cayley/cross
/// decomposition relative to the distinguished involution lives in
/// [`crate::CayleyCrossDecomposition`], and Atlas canonicalization lives in
/// [`crate::InnerClass::canonicalize`], which [`crate::CartanClassification`]
/// applies when numbering Cartan classes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TwistedInvolution {
    weyl_action: WeylAction,
    root_involution: RootInvolutionData,
}

impl TwistedInvolution {
    /// Construct `w after theta` and verify that it is a root involution.
    pub fn new(
        datum: &BasedRootDatum,
        root_system: &RootSystem,
        distinguished: &LatticeInvolution,
        weyl_action: WeylAction,
    ) -> Result<Self, StructureError> {
        if root_system.datum() != datum
            || distinguished.datum() != datum
            || weyl_action.datum() != datum
        {
            return Err(StructureError::DatumMismatch);
        }
        let rank = datum.lattice_rank();
        if root_system.lattice_rank() != rank {
            return Err(StructureError::RankMismatch {
                expected: rank,
                actual: root_system.lattice_rank(),
            });
        }
        if distinguished.lattice_rank() != rank {
            return Err(StructureError::RankMismatch {
                expected: rank,
                actual: distinguished.lattice_rank(),
            });
        }
        if weyl_action.rank() != rank {
            return Err(StructureError::RankMismatch {
                expected: rank,
                actual: weyl_action.rank(),
            });
        }
        let involution = LatticeInvolution::new(
            datum,
            compose_matrices(weyl_action.matrix(), distinguished.weight_matrix())?,
            compose_matrices(
                weyl_action.coweight_matrix(),
                distinguished.coweight_matrix(),
            )?,
        )?;
        let root_involution = RootInvolutionData::new(root_system, involution)?;
        Ok(Self {
            weyl_action,
            root_involution,
        })
    }

    /// [`Self::new`] with the root action supplied as a permutation: the
    /// involution table's records compose `w after delta` at the
    /// permutation level (`w_perm[delta_perm[r]]`, equal to the composed
    /// matrix action), so the per-root matrix classification of
    /// [`RootInvolutionData::new`] collapses to array reads. Same datum and
    /// shape gates, same matrix-level validation of the composed
    /// involution; `root_images` must be that matrix action on this system.
    pub(crate) fn new_from_root_images(
        datum: &BasedRootDatum,
        root_system: &RootSystem,
        distinguished: &LatticeInvolution,
        weyl_action: WeylAction,
        root_images: Vec<crate::RootId>,
    ) -> Result<Self, StructureError> {
        if root_system.datum() != datum
            || distinguished.datum() != datum
            || weyl_action.datum() != datum
        {
            return Err(StructureError::DatumMismatch);
        }
        let rank = datum.lattice_rank();
        if root_system.lattice_rank() != rank {
            return Err(StructureError::RankMismatch {
                expected: rank,
                actual: root_system.lattice_rank(),
            });
        }
        if distinguished.lattice_rank() != rank {
            return Err(StructureError::RankMismatch {
                expected: rank,
                actual: distinguished.lattice_rank(),
            });
        }
        if weyl_action.rank() != rank {
            return Err(StructureError::RankMismatch {
                expected: rank,
                actual: weyl_action.rank(),
            });
        }
        let involution = LatticeInvolution::new_trusted(
            datum,
            compose_matrices(weyl_action.matrix(), distinguished.weight_matrix())?,
            compose_matrices(
                weyl_action.coweight_matrix(),
                distinguished.coweight_matrix(),
            )?,
        )?;
        let root_involution = RootInvolutionData::from_images(root_system, involution, root_images)?;
        Ok(Self {
            weyl_action,
            root_involution,
        })
    }

    pub fn weyl_action(&self) -> &WeylAction {
        &self.weyl_action
    }

    pub fn root_involution(&self) -> &RootInvolutionData {
        &self.root_involution
    }

    /// Restricted roots for this actual Cartan involution.
    pub fn restricted_roots(
        &self,
        root_system: &RootSystem,
    ) -> Result<RestrictedRootSystem, StructureError> {
        RestrictedRootSystem::build(root_system, &self.root_involution)
    }
}

pub(crate) fn compose_matrices(
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
    left.iter()
        .map(|row| {
            (0..rank)
                .map(|column| {
                    let value =
                        row.iter()
                            .enumerate()
                            .try_fold(0_i128, |sum, (middle, &entry)| {
                                let product = i128::from(entry)
                                    .checked_mul(i128::from(right[middle][column]))
                                    .ok_or(StructureError::ArithmeticOverflow)?;
                                sum.checked_add(product)
                                    .ok_or(StructureError::ArithmeticOverflow)
                            })?;
                    i32::try_from(value).map_err(|_| StructureError::ArithmeticOverflow)
                })
                .collect()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::{BasedRootDatum, LatticeInvolution, RootKind, RootSystem, WeylGroup};

    use super::*;

    #[test]
    fn validates_a_weyl_translate_of_the_distinguished_involution() {
        let datum = BasedRootDatum::standard(vec![vec![2]]).unwrap();
        let roots = RootSystem::enumerate(&datum, 2).unwrap();
        let weyl = WeylGroup::new(datum.clone()).simple_reflection(0).unwrap();
        let twisted = TwistedInvolution::new(
            &datum,
            &roots,
            &LatticeInvolution::identity(&datum).unwrap(),
            weyl,
        )
        .unwrap();

        assert_eq!(
            twisted
                .root_involution()
                .roots_of_kind(RootKind::Real)
                .count(),
            2
        );
    }

    #[test]
    fn rejects_weyl_translates_that_are_not_involutive() {
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let roots = RootSystem::enumerate(&datum, 6).unwrap();
        let group = WeylGroup::new(datum.clone());
        let order_three = group
            .simple_reflection(0)
            .unwrap()
            .compose(&group.simple_reflection(1).unwrap())
            .unwrap();
        assert_eq!(
            TwistedInvolution::new(
                &datum,
                &roots,
                &LatticeInvolution::identity(&datum).unwrap(),
                order_three,
            ),
            Err(StructureError::InvalidInvolution)
        );
    }

    #[test]
    fn rejects_a_weyl_action_owned_by_a_different_same_rank_datum() {
        let a2 = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let b2 = BasedRootDatum::standard(vec![vec![2, -2], vec![-1, 2]]).unwrap();
        let roots = RootSystem::enumerate(&a2, 6).unwrap();
        let foreign_action = WeylGroup::new(b2).identity().unwrap();

        assert_eq!(
            TwistedInvolution::new(
                &a2,
                &roots,
                &LatticeInvolution::identity(&a2).unwrap(),
                foreign_action,
            ),
            Err(StructureError::DatumMismatch)
        );
    }

    #[test]
    fn composes_a_nontrivial_central_torus_involution_with_a_weyl_action() {
        let datum = BasedRootDatum::from_simple_data(
            2,
            vec![vec![2]],
            vec![crate::Weight::new(vec![1, 0])],
            vec![crate::Coweight::new(vec![2, 0])],
        )
        .unwrap();
        let roots = RootSystem::enumerate(&datum, 2).unwrap();
        let distinguished = LatticeInvolution::new(
            &datum,
            vec![vec![1, 0], vec![0, -1]],
            vec![vec![1, 0], vec![0, -1]],
        )
        .unwrap();
        let weyl = WeylGroup::new(datum.clone()).simple_reflection(0).unwrap();
        let twisted = TwistedInvolution::new(&datum, &roots, &distinguished, weyl).unwrap();

        assert_eq!(
            twisted
                .root_involution()
                .roots_of_kind(RootKind::Real)
                .count(),
            2
        );
    }
}
