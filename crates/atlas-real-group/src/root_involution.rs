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
        validate_simple_root_images(root_system, &involution)?;
        let root_count = root_system.roots().len();
        let mut image_by_root = Vec::with_capacity(root_count);
        let mut kind_by_root = Vec::with_capacity(root_count);
        // Scratch buffers: one matrix application per root (and per coroot
        // transport check) without a fresh vector per root.
        let mut image_buf = Vec::with_capacity(involution.lattice_rank());
        let mut coroot_buf = Vec::with_capacity(involution.lattice_rank());
        for (_, root, coroot) in root_system.entries() {
            involution.act_on_weight_into(root.as_slice(), &mut image_buf)?;
            let image_id = root_system
                .id_of_slice(&image_buf)
                .ok_or(StructureError::InvalidRootAutomorphism)?;
            let image_coroot =
                root_system
                    .coroot(image_id)
                    .ok_or(StructureError::IndexOutOfRange {
                        index: image_id.0,
                        upper_bound: root_count,
                    })?;
            involution.act_on_coweight_into(coroot.as_slice(), &mut coroot_buf)?;
            if coroot_buf.as_slice() != image_coroot.as_slice() {
                return Err(StructureError::InvalidRootDatumAutomorphism);
            }
            let kind = if image_buf.as_slice() == root.as_slice() {
                RootKind::Imaginary
            } else {
                let mut is_negative = true;
                for (&image_coordinate, &coordinate) in
                    image_buf.iter().zip(root.as_slice().iter())
                {
                    let negated = coordinate
                        .checked_neg()
                        .ok_or(StructureError::ArithmeticOverflow)?;
                    if image_coordinate != negated {
                        is_negative = false;
                        break;
                    }
                }
                if is_negative {
                    RootKind::Real
                } else {
                    RootKind::Complex
                }
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

    /// Classification from a KNOWN root permutation, for callers that
    /// already hold the involution's root action as a permutation (the
    /// involution table's cross-edge BFS composes `w after delta` at the
    /// permutation level, which equals the matrix action by definition of
    /// matrix composition). This skips the per-root matrix applications,
    /// root-id lookups, and coroot transport re-checks of [`Self::new`];
    /// the caller's contract is that `image_by_root` IS the involution's
    /// root action on this system, as the distinguished/Weyl factors of the
    /// table's records guarantee.
    pub(crate) fn from_images(
        root_system: &RootSystem,
        involution: LatticeInvolution,
        image_by_root: Vec<RootId>,
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
        if image_by_root.len() != root_system.roots().len() {
            return Err(StructureError::RootSystemInvariantViolation {
                invariant: "involution root action",
            });
        }
        let negatives = root_system.negatives();
        let mut kind_by_root = Vec::with_capacity(image_by_root.len());
        for (index, &image) in image_by_root.iter().enumerate() {
            kind_by_root.push(if image.0 == index {
                RootKind::Imaginary
            } else if image == negatives[index] {
                RootKind::Real
            } else {
                RootKind::Complex
            });
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

fn validate_simple_root_images(
    root_system: &RootSystem,
    involution: &LatticeInvolution,
) -> Result<(), StructureError> {
    let datum = root_system.datum();
    for (simple_root, (root, coroot)) in datum
        .simple_roots()
        .iter()
        .zip(datum.simple_coroots())
        .enumerate()
    {
        let image_root = involution.act_on_weight(root)?;
        let image_id = root_system
            .id_of(&image_root)
            .ok_or(StructureError::SimpleRootImageNotRoot { simple_root })?;
        let image_coroot = root_system
            .coroot(image_id)
            .ok_or(StructureError::IndexOutOfRange {
                index: image_id.0,
                upper_bound: root_system.roots().len(),
            })?;
        if involution.act_on_coweight(coroot)? != *image_coroot {
            return Err(StructureError::SimpleCorootImageMismatch {
                simple_root,
                image_root,
            });
        }
    }
    Ok(())
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
    // Membership by simple coordinates through a hash set of slices: the
    // decomposability probe below runs once per (candidate, summand) pair,
    // and the historic ordered-set-of-vectors with a fresh remainder
    // allocation per probe dominated RootInvolutionData::new (E8 identity
    // class: ~120 x 120 probes per call). Collisions compare exactly, so
    // semantics are unchanged.
    let positive_coordinates = positive
        .iter()
        .map(|&root| {
            root_system
                .simple_coordinates(root)
                .ok_or(StructureError::IndexOutOfRange {
                    index: root.0,
                    upper_bound: root_system.roots().len(),
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut members: std::collections::HashSet<&[i32], crate::inner_class::PermutationHasherBuilder> =
        std::collections::HashSet::default();
    members
        .try_reserve(positive_coordinates.len())
        .map_err(|_| StructureError::AllocationFailed {
            requested: positive_coordinates.len(),
        })?;
    for coordinates in &positive_coordinates {
        members.insert(coordinates);
    }
    let rank = root_system.datum().semisimple_rank();
    let mut remainder: Vec<i32> = Vec::new();
    remainder
        .try_reserve(rank)
        .map_err(|_| StructureError::AllocationFailed { requested: rank })?;
    let mut simple_roots = Vec::new();
    'candidate: for (candidate_index, &candidate) in positive.iter().enumerate() {
        let candidate_coordinates = positive_coordinates[candidate_index];
        for (summand_index, summand) in positive_coordinates.iter().enumerate() {
            if summand_index == candidate_index {
                continue;
            }
            remainder.clear();
            for (&candidate_coordinate, &summand_coordinate) in
                candidate_coordinates.iter().zip(*summand)
            {
                remainder.push(
                    candidate_coordinate
                        .checked_sub(summand_coordinate)
                        .ok_or(StructureError::ArithmeticOverflow)?,
                );
            }
            if members.contains(remainder.as_slice()) {
                continue 'candidate;
            }
        }
        simple_roots.push(candidate);
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
            vec![vec![-1, 0], vec![1, 1]],
            vec![vec![-1, 1], vec![0, 1]],
        )
        .unwrap();
        assert_eq!(
            RootInvolutionData::new(&roots, involution),
            Err(StructureError::SimpleRootImageNotRoot { simple_root: 0 })
        );
    }

    #[test]
    fn reports_a_positive_simple_root_with_the_wrong_image_coroot() {
        let datum = BasedRootDatum::from_simple_data(
            2,
            vec![vec![2]],
            vec![Weight::new(vec![1, 0])],
            vec![Coweight::new(vec![2, 0])],
        )
        .unwrap();
        let roots = RootSystem::enumerate(&datum, 2).unwrap();
        let involution = LatticeInvolution::new(
            &datum,
            vec![vec![1, 2], vec![0, -1]],
            vec![vec![1, 0], vec![2, -1]],
        )
        .unwrap();

        assert_eq!(
            RootInvolutionData::new(&roots, involution),
            Err(StructureError::SimpleCorootImageMismatch {
                simple_root: 0,
                image_root: Weight::new(vec![1, 0]),
            })
        );
    }

    #[test]
    fn reports_a_negative_simple_root_with_the_wrong_image_coroot() {
        let datum = BasedRootDatum::from_simple_data(
            2,
            vec![vec![2]],
            vec![Weight::new(vec![1, 0])],
            vec![Coweight::new(vec![2, 0])],
        )
        .unwrap();
        let roots = RootSystem::enumerate(&datum, 2).unwrap();
        let involution = LatticeInvolution::new(
            &datum,
            vec![vec![-1, 2], vec![0, 1]],
            vec![vec![-1, 0], vec![2, 1]],
        )
        .unwrap();

        assert_eq!(
            RootInvolutionData::new(&roots, involution),
            Err(StructureError::SimpleCorootImageMismatch {
                simple_root: 0,
                image_root: Weight::new(vec![-1, 0]),
            })
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
