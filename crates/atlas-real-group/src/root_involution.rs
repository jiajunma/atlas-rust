use std::collections::BTreeSet;

use crate::{LatticeInvolution, RootId, RootSystem, StructureError};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RootKind {
    Imaginary,
    Real,
    Complex,
}

/// An involution action on a generated ordinary root system.
///
/// Construction validates that the dual-lattice involution truly permutes the
/// root system and transports each stored coroot to the coroot of its image
/// root before classifying anything. The second condition is the root-datum
/// automorphism property: pairing preservation alone admits actions that fix
/// every root while moving a coroot's central-torus coordinates.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RootInvolutionData {
    involution: LatticeInvolution,
    image_by_root: Vec<RootId>,
    kind_by_root: Vec<RootKind>,
    imaginary_simple_roots: Vec<RootId>,
    real_simple_roots: Vec<RootId>,
}

impl RootInvolutionData {
    pub fn new(
        root_system: &RootSystem,
        involution: LatticeInvolution,
    ) -> Result<Self, StructureError> {
        if involution.datum() != root_system.datum() {
            return Err(StructureError::DatumMismatch);
        }
        if involution.lattice_rank() != root_system.lattice_rank() {
            return Err(StructureError::RankMismatch {
                expected: root_system.lattice_rank(),
                actual: involution.lattice_rank(),
            });
        }
        let mut image_by_root = Vec::with_capacity(root_system.roots().len());
        let mut kind_by_root = Vec::with_capacity(root_system.roots().len());
        for (_, root, coroot) in root_system.entries() {
            let image = involution.act_on_weight(root)?;
            let image_id = root_system
                .id_of(&image)
                .ok_or(StructureError::InvalidRootAutomorphism)?;
            let image_coroot =
                root_system
                    .coroot(image_id)
                    .ok_or(StructureError::IndexOutOfRange {
                        index: image_id.0,
                        upper_bound: root_system.roots().len(),
                    })?;
            if involution.act_on_coweight(coroot)? != *image_coroot {
                return Err(StructureError::InvalidRootDatumAutomorphism);
            }
            let negative = root
                .as_slice()
                .iter()
                .map(|coordinate| {
                    coordinate
                        .checked_neg()
                        .ok_or(StructureError::ArithmeticOverflow)
                })
                .collect::<Result<Vec<_>, _>>()?;
            let kind = if image == *root {
                RootKind::Imaginary
            } else if image.as_slice() == negative.as_slice() {
                RootKind::Real
            } else {
                RootKind::Complex
            };
            image_by_root.push(image_id);
            kind_by_root.push(kind);
        }
        let imaginary_simple_roots =
            subsystem_simple_roots(root_system, &kind_by_root, RootKind::Imaginary)?;
        let real_simple_roots = subsystem_simple_roots(root_system, &kind_by_root, RootKind::Real)?;
        Ok(Self {
            involution,
            image_by_root,
            kind_by_root,
            imaginary_simple_roots,
            real_simple_roots,
        })
    }

    pub fn involution(&self) -> &LatticeInvolution {
        &self.involution
    }

    pub fn image(&self, root: RootId) -> Option<RootId> {
        self.image_by_root.get(root.0).copied()
    }

    pub fn image_permutation(&self) -> &[RootId] {
        &self.image_by_root
    }

    pub fn kind(&self, root: RootId) -> Option<RootKind> {
        self.kind_by_root.get(root.0).copied()
    }

    pub fn roots_of_kind(&self, kind: RootKind) -> impl Iterator<Item = RootId> + '_ {
        self.kind_by_root
            .iter()
            .enumerate()
            .filter_map(move |(index, candidate)| (*candidate == kind).then_some(RootId(index)))
    }

    /// Simple roots of the imaginary-root subsystem in the inherited positive system.
    pub fn imaginary_simple_roots(&self) -> &[RootId] {
        &self.imaginary_simple_roots
    }

    /// Simple roots of the real-root subsystem in the inherited positive system.
    pub fn real_simple_roots(&self) -> &[RootId] {
        &self.real_simple_roots
    }
}

fn subsystem_simple_roots(
    root_system: &RootSystem,
    kinds: &[RootKind],
    kind: RootKind,
) -> Result<Vec<RootId>, StructureError> {
    let positive = kinds
        .iter()
        .enumerate()
        .filter_map(|(index, candidate_kind)| (*candidate_kind == kind).then_some(RootId(index)))
        .filter(|&root| {
            root_system
                .simple_coordinates(root)
                .is_some_and(|coordinates| coordinates.iter().all(|&coordinate| coordinate >= 0))
        })
        .collect::<Vec<_>>();
    let positive_coordinates = positive
        .iter()
        .map(|&root| {
            root_system
                .simple_coordinates(root)
                .map(|coordinates| coordinates.to_vec())
                .ok_or(StructureError::IndexOutOfRange {
                    index: root.0,
                    upper_bound: root_system.roots().len(),
                })
        })
        .collect::<Result<BTreeSet<_>, _>>()?;
    let mut simple_roots = Vec::new();
    for candidate in positive {
        let candidate_coordinates =
            root_system
                .simple_coordinates(candidate)
                .ok_or(StructureError::IndexOutOfRange {
                    index: candidate.0,
                    upper_bound: root_system.roots().len(),
                })?;
        let mut decomposable = false;
        for summand in &positive_coordinates {
            let remainder = candidate_coordinates
                .iter()
                .zip(summand)
                .map(|(&candidate_coordinate, &summand_coordinate)| {
                    candidate_coordinate
                        .checked_sub(summand_coordinate)
                        .ok_or(StructureError::ArithmeticOverflow)
                })
                .collect::<Result<Vec<_>, _>>()?;
            if positive_coordinates.contains(&remainder) {
                decomposable = true;
                break;
            }
        }
        if !decomposable {
            simple_roots.push(candidate);
        }
    }
    Ok(simple_roots)
}

#[cfg(test)]
mod tests {
    use crate::{BasedRootDatum, Coweight, LatticeInvolution, RootSystem, Weight};

    use super::*;

    #[test]
    fn classifies_real_and_complex_a2_roots() {
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let roots = RootSystem::enumerate(&datum, 6).unwrap();
        let data = RootInvolutionData::new(
            &roots,
            LatticeInvolution::new(
                &datum,
                vec![vec![0, -1], vec![-1, 0]],
                vec![vec![0, -1], vec![-1, 0]],
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(data.roots_of_kind(RootKind::Real).count(), 2);
        assert_eq!(data.roots_of_kind(RootKind::Complex).count(), 4);
        assert_eq!(data.roots_of_kind(RootKind::Imaginary).count(), 0);
        assert_eq!(
            data.real_simple_roots(),
            &[roots.id_of(&Weight::new(vec![1, 1])).unwrap()]
        );
    }

    #[test]
    fn rejects_pairing_preserving_actions_that_do_not_permute_roots() {
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let roots = RootSystem::enumerate(&datum, 6).unwrap();
        let involution = LatticeInvolution::new(
            &datum,
            vec![vec![1, 1], vec![0, -1]],
            vec![vec![1, 0], vec![1, -1]],
        )
        .unwrap();
        assert_eq!(
            RootInvolutionData::new(&roots, involution),
            Err(StructureError::InvalidRootAutomorphism)
        );
    }

    #[test]
    fn rejects_a_root_system_from_a_different_same_rank_datum() {
        let a2 = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let product_a1 = BasedRootDatum::standard(vec![vec![2, 0], vec![0, 2]]).unwrap();
        let roots = RootSystem::enumerate(&product_a1, 4).unwrap();

        assert_eq!(
            RootInvolutionData::new(&roots, LatticeInvolution::identity(&a2).unwrap()),
            Err(StructureError::DatumMismatch)
        );
    }

    #[test]
    fn classifies_identity_as_imaginary() {
        let datum = BasedRootDatum::from_simple_data(
            2,
            vec![vec![2]],
            vec![Weight::new(vec![1, 0])],
            vec![Coweight::new(vec![2, 0])],
        )
        .unwrap();
        let roots = RootSystem::enumerate(&datum, 2).unwrap();
        let data =
            RootInvolutionData::new(&roots, LatticeInvolution::identity(&datum).unwrap()).unwrap();
        assert_eq!(data.roots_of_kind(RootKind::Imaginary).count(), 2);
    }

    #[test]
    fn finds_the_inherited_simple_basis_of_the_imaginary_subsystem() {
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let roots = RootSystem::enumerate(&datum, 6).unwrap();
        let data =
            RootInvolutionData::new(&roots, LatticeInvolution::identity(&datum).unwrap()).unwrap();

        assert_eq!(
            data.imaginary_simple_roots(),
            &[
                roots.id_of(&Weight::new(vec![0, 1])).unwrap(),
                roots.id_of(&Weight::new(vec![1, 0])).unwrap(),
            ]
        );
    }
}
