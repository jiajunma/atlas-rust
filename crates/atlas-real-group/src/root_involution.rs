use std::sync::Arc;

use smallvec::SmallVec;

use crate::{LatticeInvolution, ModTwoVector, RootId, RootSystem, StructureError};

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

/// Row bitmasks of the involution's cocharacter action mod 2: row `i` has
/// bit `j` set exactly when the coweight entry `(i, j)` is odd — i.e. when
/// the retained character matrix's `(j, i)` entry is odd (the cocharacter
/// action is the transpose). Applying the mod-2 action to a bit vector is
/// then one AND + popcount per row instead of a strided per-entry scan of
/// the integer matrix — bit-identical to
/// [`crate::tits_element::apply_matrix_mod_two`] on the same operands
/// (`entry % 2 != 0` selects exactly the odd entries, negatives included).
/// 16 inline words cover every lattice rank <= 16 without a heap touch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CoweightParity {
    dimension: usize,
    words_per_row: usize,
    rows: SmallVec<[u64; 16]>,
}

impl CoweightParity {
    fn new(weight_action: &[Vec<i32>]) -> Result<Self, StructureError> {
        let dimension = weight_action.len();
        let words_per_row = dimension
            .checked_add(u64::BITS as usize - 1)
            .ok_or(StructureError::ArithmeticOverflow)?
            / u64::BITS as usize;
        let word_total = dimension
            .checked_mul(words_per_row)
            .ok_or(StructureError::ArithmeticOverflow)?;
        let mut rows = SmallVec::new();
        rows.try_reserve_exact(word_total)
            .map_err(|_| StructureError::AllocationFailed {
                requested: word_total,
            })?;
        rows.resize(word_total, 0_u64);
        for (column, stored_row) in weight_action.iter().enumerate() {
            if stored_row.len() != dimension {
                return Err(StructureError::RankMismatch {
                    expected: dimension,
                    actual: stored_row.len(),
                });
            }
            for (row, &entry) in stored_row.iter().enumerate() {
                if entry % 2 != 0 {
                    rows[row * words_per_row + column / u64::BITS as usize] |=
                        1_u64 << (column % u64::BITS as usize);
                }
            }
        }
        Ok(Self {
            dimension,
            words_per_row,
            rows,
        })
    }

    /// The mod-2 cocharacter action on a bit vector: output bit `i` is the
    /// parity of row `i` over the set bits of the input.
    fn apply(&self, vector: &ModTwoVector) -> Result<ModTwoVector, StructureError> {
        if vector.dimension() != self.dimension {
            return Err(StructureError::RankMismatch {
                expected: self.dimension,
                actual: vector.dimension(),
            });
        }
        let words = vector.words();
        let mut ones = Vec::new();
        for row in 0..self.dimension {
            let base = row * self.words_per_row;
            let mut parity = 0_u32;
            for (word_index, &mask) in
                self.rows[base..base + self.words_per_row].iter().enumerate()
            {
                parity ^= (mask & words[word_index]).count_ones() & 1;
            }
            if parity == 1 {
                ones.push(row);
            }
        }
        ModTwoVector::from_ones(self.dimension, ones)
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
    /// Mod-2 row masks of the cocharacter action, derived from the same
    /// matrix at construction; serves the KGB cross-pull transport.
    coweight_parity: CoweightParity,
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
        let coweight_parity = CoweightParity::new(involution.weight_matrix())?;
        Ok(Self {
            involution,
            image_by_root: image_by_root.into_boxed_slice(),
            negatives,
            imaginary_simple_roots: imaginary_simple_roots.into_boxed_slice(),
            real_simple_roots: real_simple_roots.into_boxed_slice(),
            coweight_parity,
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
        let coweight_parity = CoweightParity::new(involution.weight_matrix())?;
        Ok(Self {
            involution,
            image_by_root: compact.into_boxed_slice(),
            negatives,
            imaginary_simple_roots: imaginary_simple_roots.into_boxed_slice(),
            real_simple_roots: real_simple_roots.into_boxed_slice(),
            coweight_parity,
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

    /// The mod-2 cocharacter action on a bit vector, served from the cached
    /// parity rows — bit-identical to
    /// [`crate::tits_element::apply_matrix_mod_two`] on the integer matrix.
    pub(crate) fn apply_coweight_mod_two(
        &self,
        vector: &ModTwoVector,
    ) -> Result<ModTwoVector, StructureError> {
        self.coweight_parity.apply(vector)
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

/// Injective packing of a nonnegative simple-coordinate vector: 8 bits per
/// coordinate, coordinate `i` at bit offset `8i`. Only called when the
/// caller has checked the vector length is <= 8 and every coordinate fits
/// in a byte.
fn pack_coordinates(coordinates: &[i32]) -> u64 {
    debug_assert!(coordinates.len() <= 8);
    let mut packed = 0_u64;
    for (index, &coordinate) in coordinates.iter().enumerate() {
        debug_assert!((0..=u8::MAX as i32).contains(&coordinate));
        packed |= (coordinate as u64) << (8 * index);
    }
    packed
}

/// The subsystem-membership probe of [`subsystem_simple_roots`]: one u64
/// hash round when the coordinates pack, exact slice hashing otherwise.
/// Both variants decide membership by exact equality, so the classification
/// result is identical either way.
enum Membership<'a> {
    Packed(std::collections::HashSet<u64, crate::involution_table::MixingHasherBuilder>),
    Slices(std::collections::HashSet<&'a [i32], crate::involution_table::MixingHasherBuilder>),
}

impl Membership<'_> {
    fn contains(&self, coordinates: &[i32]) -> bool {
        match self {
            // Probed vectors are differences of two packed members that
            // survived the dominance check, hence nonnegative and bounded
            // by the member maximum — the pack preconditions still hold.
            Self::Packed(packed) => packed.contains(&pack_coordinates(coordinates)),
            Self::Slices(slices) => slices.contains(coordinates),
        }
    }
}

/// Simple roots of one kind's subsystem in the inherited positive system.
///
/// The decomposability probe iterates only the already-confirmed SIMPLES,
/// not every member: for a positive member `c`, some subsystem simple `s`
/// has `<c, s∨> > 0` (else `<c, c∨> = Σ m_i <c, s_i∨>` could not be 2 with
/// `m_i >= 0`), the root string then makes `c - s` a root, it is positive
/// (the simple coordinates stay nonnegative) and closed under the kind
/// (theta is additive on roots), so `c` is decomposable IFF `c - s` is a
/// member for an already-confirmed simple `s`. Inherited-simple-coordinate
/// height is additive over the decomposition, so processing candidates in
/// increasing height confirms every simple before any candidate that could
/// decompose over it. The emitted ORDER is the historic enumeration order
/// of `positive`, so the returned vector is unchanged bit for bit.
fn subsystem_simple_roots(
    root_system: &RootSystem,
    image_by_root: &[u16],
    negatives: &[RootId],
    kind: RootKind,
) -> Result<Vec<RootId>, StructureError> {
    debug_assert_eq!(image_by_root.len(), negatives.len());
    let mut positive = Vec::new();
    let mut positive_coordinates: Vec<&[i32]> = Vec::new();
    for (index, (&image, &negative)) in image_by_root.iter().zip(negatives.iter()).enumerate() {
        if classify_root(index, RootId(usize::from(image)), negative) != kind {
            continue;
        }
        let Some(coordinates) = root_system.simple_coordinates(RootId(index)) else {
            continue;
        };
        if coordinates.iter().all(|&coordinate| coordinate >= 0) {
            positive.push(RootId(index));
            positive_coordinates.push(coordinates);
        }
    }
    // Membership by simple coordinates. With semisimple rank <= 8 (the
    // compact-Weyl ceiling of `WeylElt`) every NONNEGATIVE coordinate
    // vector packs injectively into one u64 (8 bits per coordinate; root
    // coordinates stay <= 6 in every finite type), so the probe becomes a
    // single integer hash round instead of one hash round per slice
    // element; collisions still compare exactly. Larger ranks fall back to
    // hashing the coordinate slices themselves.
    let rank = root_system.datum().semisimple_rank();
    let packable = rank <= 8
        && positive_coordinates
            .iter()
            .all(|coordinates| coordinates.iter().all(|&coordinate| coordinate <= u8::MAX as i32));
    let members: Membership = if packable {
        let mut packed: std::collections::HashSet<
            u64,
            crate::involution_table::MixingHasherBuilder,
        > = std::collections::HashSet::default();
        packed
            .try_reserve(positive_coordinates.len())
            .map_err(|_| StructureError::AllocationFailed {
                requested: positive_coordinates.len(),
            })?;
        for coordinates in &positive_coordinates {
            packed.insert(pack_coordinates(coordinates));
        }
        Membership::Packed(packed)
    } else {
        let mut slices: std::collections::HashSet<
            &[i32],
            crate::involution_table::MixingHasherBuilder,
        > = std::collections::HashSet::default();
        slices
            .try_reserve(positive_coordinates.len())
            .map_err(|_| StructureError::AllocationFailed {
                requested: positive_coordinates.len(),
            })?;
        for coordinates in &positive_coordinates {
            slices.insert(coordinates);
        }
        Membership::Slices(slices)
    };
    let mut remainder: Vec<i32> = Vec::new();
    remainder
        .try_reserve(rank)
        .map_err(|_| StructureError::AllocationFailed { requested: rank })?;
    // Heights order the scan; i64 sums cannot overflow on root coordinates.
    let heights: Vec<i64> = positive_coordinates
        .iter()
        .map(|coordinates| coordinates.iter().map(|&c| i64::from(c)).sum())
        .collect();
    let mut scan_order: Vec<usize> = (0..positive.len()).collect();
    scan_order.sort_by_key(|&index| heights[index]);
    let mut is_simple = vec![false; positive.len()];
    let mut confirmed_simples: Vec<usize> = Vec::new();
    'candidate: for candidate_index in scan_order {
        let candidate_coordinates = positive_coordinates[candidate_index];
        for &simple_index in &confirmed_simples {
            remainder.clear();
            let mut dominated = false;
            for (&candidate_coordinate, &summand_coordinate) in candidate_coordinates
                .iter()
                .zip(positive_coordinates[simple_index])
            {
                let difference = candidate_coordinate
                    .checked_sub(summand_coordinate)
                    .ok_or(StructureError::ArithmeticOverflow)?;
                if difference < 0 {
                    // Members all have nonnegative coordinates, so this
                    // remainder can never be a member; skip the hash probe.
                    dominated = true;
                    break;
                }
                remainder.push(difference);
            }
            if !dominated && members.contains(&remainder) {
                continue 'candidate;
            }
        }
        is_simple[candidate_index] = true;
        confirmed_simples.push(candidate_index);
    }
    let mut simple_roots = Vec::new();
    for (index, &root) in positive.iter().enumerate() {
        if is_simple[index] {
            simple_roots.push(root);
        }
    }
    Ok(simple_roots)
}

#[cfg(test)]
mod tests {
    use crate::{BasedRootDatum, Coweight, LatticeInvolution, RootSystem, Weight, WeylAction};

    use super::*;

    /// The retired all-pairs decomposability probe, kept as the definitional
    /// oracle for the height-ordered production algorithm.
    fn naive_subsystem_simple_roots(
        root_system: &RootSystem,
        data: &RootInvolutionData,
        kind: RootKind,
    ) -> Vec<RootId> {
        let positive: Vec<RootId> = data
            .roots_of_kind(kind)
            .filter(|&root| {
                root_system
                    .simple_coordinates(root)
                    .is_some_and(|coordinates| {
                        coordinates.iter().all(|&coordinate| coordinate >= 0)
                    })
            })
            .collect();
        let coordinates: Vec<&[i32]> = positive
            .iter()
            .map(|&root| root_system.simple_coordinates(root).unwrap())
            .collect();
        let mut simple_roots = Vec::new();
        'candidate: for (candidate_index, &candidate) in positive.iter().enumerate() {
            for (summand_index, summand) in coordinates.iter().enumerate() {
                if summand_index == candidate_index {
                    continue;
                }
                let remainder: Vec<i32> = coordinates[candidate_index]
                    .iter()
                    .zip(*summand)
                    .map(|(&c, &s)| c - s)
                    .collect();
                if remainder.iter().all(|&c| c >= 0)
                    && coordinates
                        .iter()
                        .any(|&member| member == remainder.as_slice())
                {
                    continue 'candidate;
                }
            }
            simple_roots.push(candidate);
        }
        simple_roots
    }

    #[test]
    fn height_ordered_simple_roots_match_the_all_pairs_probe() {
        let cartans = [
            vec![vec![2]],
            vec![vec![2, -1], vec![-1, 2]],
            vec![vec![2, -2], vec![-1, 2]],
            vec![vec![2, 0], vec![0, 2]],
            vec![vec![2, -1, 0], vec![-1, 2, -1], vec![0, -1, 2]],
            vec![vec![2, -1, 0], vec![-1, 2, -2], vec![0, -1, 2]],
            vec![
                vec![2, -1, 0, 0],
                vec![-1, 2, -1, -1],
                vec![0, -1, 2, 0],
                vec![0, -1, 0, 2],
            ],
        ];
        for cartan in &cartans {
            let datum = BasedRootDatum::standard(cartan.clone()).unwrap();
            let roots = RootSystem::enumerate(&datum, 1 << 12).unwrap();
            let rank = datum.semisimple_rank();
            let minus_identity: Vec<Vec<i32>> = (0..rank)
                .map(|i| {
                    (0..rank)
                        .map(|j| if i == j { -1 } else { 0 })
                        .collect::<Vec<i32>>()
                })
                .collect();
            let mut involutions = vec![
                LatticeInvolution::identity(&datum).unwrap(),
                LatticeInvolution::new(&datum, minus_identity.clone(), minus_identity).unwrap(),
            ];
            for generator in 0..rank {
                let reflection = WeylAction::simple_reflection(&datum, generator).unwrap();
                involutions.push(
                    LatticeInvolution::new(
                        &datum,
                        reflection.matrix().to_vec(),
                        reflection.coweight_matrix().to_vec(),
                    )
                    .unwrap(),
                );
            }
            for involution in &involutions {
                let data = RootInvolutionData::new(&roots, involution.clone()).unwrap();
                for kind in [RootKind::Imaginary, RootKind::Real, RootKind::Complex] {
                    let expected = naive_subsystem_simple_roots(&roots, &data, kind);
                    let actual = match kind {
                        RootKind::Imaginary => data.imaginary_simple_roots(),
                        RootKind::Real => data.real_simple_roots(),
                        RootKind::Complex => continue,
                    };
                    assert_eq!(actual, expected.as_slice());
                }
            }
        }
    }

    #[test]
    fn cached_coweight_parity_matches_the_integer_matrix_scan() {
        let cartans = [
            vec![vec![2]],
            vec![vec![2, -1], vec![-1, 2]],
            vec![vec![2, -2], vec![-1, 2]],
            vec![vec![2, -1, 0], vec![-1, 2, -2], vec![0, -1, 2]],
        ];
        for cartan in &cartans {
            let datum = BasedRootDatum::standard(cartan.clone()).unwrap();
            let roots = RootSystem::enumerate(&datum, 1 << 12).unwrap();
            let rank = datum.lattice_rank();
            let mut involutions = vec![LatticeInvolution::identity(&datum).unwrap()];
            for generator in 0..datum.semisimple_rank() {
                let reflection = WeylAction::simple_reflection(&datum, generator).unwrap();
                involutions.push(
                    LatticeInvolution::new(
                        &datum,
                        reflection.matrix().to_vec(),
                        reflection.coweight_matrix().to_vec(),
                    )
                    .unwrap(),
                );
            }
            for involution in &involutions {
                let data = RootInvolutionData::new(&roots, involution.clone()).unwrap();
                for pattern in 0..(1_u32 << rank) {
                    let ones: Vec<usize> = (0..rank)
                        .filter(|&bit| pattern & (1 << bit) != 0)
                        .collect();
                    let vector = ModTwoVector::from_ones(rank, ones).unwrap();
                    let expected = crate::tits_element::apply_matrix_mod_two(
                        involution.coweight_matrix(),
                        &vector,
                    )
                    .unwrap();
                    assert_eq!(data.apply_coweight_mod_two(&vector), Ok(expected));
                }
            }
        }
    }

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
