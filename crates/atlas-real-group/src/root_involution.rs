use std::sync::Arc;

use crate::{LatticeInvolution, RootId, RootSystem, StructureError};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RootKind {
    Imaginary,
    Real,
    Complex,
}

/// Read-only random-access view of an involution's root action.
///
/// Root images are retained as `u16` — the width of upstream's `RootNbr`
/// (`unsigned short`) — so the involution table's per-record storage is a
/// quarter of the retired `Vec<RootId>`; this view widens values back to
/// `RootId` at the accessor boundary.
#[derive(Clone, Copy, Debug)]
pub struct ImagePermutation<'a> {
    images: &'a [u16],
}

impl<'a> ImagePermutation<'a> {
    pub fn len(&self) -> usize {
        self.images.len()
    }

    pub fn is_empty(&self) -> bool {
        self.images.is_empty()
    }

    /// Image of the root at `index`, widening the stored `u16`.
    ///
    /// Panics on out-of-range indexes, matching the historic slice indexing
    /// of the retired `&[RootId]` return.
    pub fn at(&self, index: usize) -> RootId {
        RootId(usize::from(self.images[index]))
    }

    pub fn get(&self, index: usize) -> Option<RootId> {
        self.images
            .get(index)
            .map(|&image| RootId(usize::from(image)))
    }

    pub fn iter(&self) -> impl Iterator<Item = RootId> + '_ {
        self.images.iter().map(|&image| RootId(usize::from(image)))
    }

    /// The widened permutation as an owned vector, for callers that
    /// transport the action into new record storage.
    pub fn to_vec(&self) -> Vec<RootId> {
        self.iter().collect()
    }
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
    image_by_root: Box<[u16]>,
    negatives: Arc<[RootId]>,
    /// Boxed, not `Vec`: these are immutable after construction and the
    /// involution table retains one record per twisted involution, so the
    /// 8B capacity field is pure per-record overhead.
    imaginary_simple_roots: Box<[RootId]>,
    real_simple_roots: Box<[RootId]>,
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
            image_by_root.push(compact_image(image_id)?);
        }
        let negatives = Arc::clone(root_system.negatives_arc());
        let imaginary_simple_roots =
            subsystem_simple_roots(root_system, &image_by_root, &negatives, RootKind::Imaginary)?;
        let real_simple_roots =
            subsystem_simple_roots(root_system, &image_by_root, &negatives, RootKind::Real)?;
        Ok(Self {
            involution,
            image_by_root: image_by_root.into_boxed_slice(),
            negatives,
            imaginary_simple_roots: imaginary_simple_roots.into_boxed_slice(),
            real_simple_roots: real_simple_roots.into_boxed_slice(),
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
        let compact: Vec<u16> = image_by_root
            .iter()
            .map(|&image| compact_image(image))
            .collect::<Result<_, _>>()?;
        let negatives = Arc::clone(root_system.negatives_arc());
        let imaginary_simple_roots =
            subsystem_simple_roots(root_system, &compact, &negatives, RootKind::Imaginary)?;
        let real_simple_roots =
            subsystem_simple_roots(root_system, &compact, &negatives, RootKind::Real)?;
        Ok(Self {
            involution,
            image_by_root: compact.into_boxed_slice(),
            negatives,
            imaginary_simple_roots: imaginary_simple_roots.into_boxed_slice(),
            real_simple_roots: real_simple_roots.into_boxed_slice(),
        })
    }

    pub fn image(&self, root: RootId) -> Option<RootId> {
        self.image_by_root
            .get(root.0)
            .map(|&image| RootId(usize::from(image)))
    }

    pub fn image_permutation(&self) -> ImagePermutation<'_> {
        ImagePermutation {
            images: &self.image_by_root,
        }
    }

    pub fn kind(&self, root: RootId) -> Option<RootKind> {
        let image = self.image_by_root.get(root.0).copied()?;
        let negative = self.negatives.get(root.0).copied()?;
        Some(classify_root(root.0, RootId(usize::from(image)), negative))
    }

    pub fn roots_of_kind(&self, kind: RootKind) -> impl Iterator<Item = RootId> + '_ {
        self.image_by_root
            .iter()
            .map(|&image| RootId(usize::from(image)))
            .zip(self.negatives.iter().copied())
            .enumerate()
            .filter_map(move |(index, (image, negative))| {
                (classify_root(index, image, negative) == kind).then_some(RootId(index))
            })
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

#[inline]
fn classify_root(index: usize, image: RootId, negative: RootId) -> RootKind {
    if image.0 == index {
        RootKind::Imaginary
    } else if image == negative {
        RootKind::Real
    } else {
        RootKind::Complex
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

/// Narrow one root image into the compact storage width (upstream
/// `RootNbr` is `unsigned short`); root systems beyond 2^16 roots are
/// rejected rather than truncated.
fn compact_image(image: RootId) -> Result<u16, StructureError> {
    u16::try_from(image.0).map_err(|_| StructureError::RootSystemInvariantViolation {
        invariant: "involution root action width",
    })
}

fn subsystem_simple_roots(
    root_system: &RootSystem,
    image_by_root: &[u16],
    negatives: &[RootId],
    kind: RootKind,
) -> Result<Vec<RootId>, StructureError> {
    debug_assert_eq!(image_by_root.len(), negatives.len());
    let positive = image_by_root
        .iter()
        .map(|&image| RootId(usize::from(image)))
        .zip(negatives.iter().copied())
        .enumerate()
        .filter_map(|(index, (image, negative))| {
            (classify_root(index, image, negative) == kind).then_some(RootId(index))
        })
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
    let mut members: std::collections::HashSet<
        &[i32],
        crate::inner_class::PermutationHasherBuilder,
    > = std::collections::HashSet::default();
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
    fn derives_each_a2_kind_from_the_image_and_shared_negation_table() {
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

        let mut expected = [Vec::new(), Vec::new(), Vec::new()];
        for (root, _, _) in roots.entries() {
            let image = data.image(root).unwrap();
            let (kind, bucket) = if image == root {
                (RootKind::Imaginary, 0)
            } else if image == roots.negatives()[root.0] {
                (RootKind::Real, 1)
            } else {
                (RootKind::Complex, 2)
            };
            assert_eq!(data.kind(root), Some(kind));
            expected[bucket].push(root);
        }
        assert_eq!(
            data.roots_of_kind(RootKind::Imaginary).collect::<Vec<_>>(),
            expected[0]
        );
        assert_eq!(
            data.roots_of_kind(RootKind::Real).collect::<Vec<_>>(),
            expected[1]
        );
        assert_eq!(
            data.roots_of_kind(RootKind::Complex).collect::<Vec<_>>(),
            expected[2]
        );
        assert_eq!(data.kind(RootId(roots.roots().len())), None);
    }

    #[test]
    fn cloned_root_involutions_share_negatives_and_remain_value_equal() {
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let roots = RootSystem::enumerate(&datum, 6).unwrap();
        let data =
            RootInvolutionData::new(&roots, LatticeInvolution::identity(&datum).unwrap()).unwrap();
        let clone = data.clone();

        assert_eq!(data, clone);
        assert!(Arc::ptr_eq(&data.negatives, &clone.negatives));
        assert!(Arc::ptr_eq(&data.negatives, roots.negatives_arc()));
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

    #[test]
    fn shares_root_system_negatives_storage() {
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let roots = RootSystem::enumerate(&datum, 6).unwrap();
        let data =
            RootInvolutionData::new(&roots, LatticeInvolution::identity(&datum).unwrap()).unwrap();

        assert!(Arc::ptr_eq(&data.negatives, roots.negatives_arc()));
    }
}
