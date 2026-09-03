use std::collections::{HashMap, VecDeque};
use std::hash::{BuildHasherDefault, Hasher};
use std::sync::Arc;

use crate::lattice::{pair_coordinates, try_copy_coordinates};
use crate::{pair, BasedRootDatum, Coweight, StructureError, Weight, WeylAction};
use smallvec::SmallVec;

/// Stable identifier for one ordinary root in a deterministically ordered root
/// system. One ID indexes the root, its coroot, and its simple-root
/// coordinates simultaneously.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RootId(pub(crate) usize);

impl RootId {
    /// Construct a root index (crate-internal numbering).
    pub fn from_usize(index: usize) -> Self {
        Self(index)
    }

    /// The crate-internal root index.
    pub fn index(&self) -> usize {
        self.0
    }
}

/// Caller-owned resource bounds for one ordinary-root closure.
///
/// These are computational budgets for a single enumeration, not a
/// mathematical rank cap. `max_lattice_rank` bounds the full torus rank (and
/// with it the semisimple rank), `max_roots` bounds the accepted root
/// cardinality, `max_coordinate_entries` bounds peak live coordinate values,
/// and `max_reflection_steps` bounds dual-reflection visits. The entry and
/// step limits are consistency checks against the worst case implied by
/// `max_roots`; after they pass, only the cardinality limit can fire.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RootSystemBudget {
    max_lattice_rank: usize,
    max_roots: usize,
    max_coordinate_entries: usize,
    max_reflection_steps: usize,
}

impl RootSystemBudget {
    pub const fn new(
        max_lattice_rank: usize,
        max_roots: usize,
        max_coordinate_entries: usize,
        max_reflection_steps: usize,
    ) -> Self {
        Self {
            max_lattice_rank,
            max_roots,
            max_coordinate_entries,
            max_reflection_steps,
        }
    }

    /// The budget whose derived limits cannot bind before `max_roots` does.
    ///
    /// The entry and step limits are the saturated worst-case bounds for this
    /// datum at `max_roots`, so enumeration under this budget accepts and
    /// rejects exactly as the compatibility wrapper
    /// [`RootSystem::enumerate`] does.
    pub fn complete_for(datum: &BasedRootDatum, max_roots: usize) -> Self {
        Self {
            max_lattice_rank: datum.lattice_rank(),
            max_roots,
            max_coordinate_entries: saturated_to_usize(entry_bound(datum, max_roots)),
            max_reflection_steps: saturated_to_usize(step_bound(datum, max_roots)),
        }
    }
}

/// A set of root IDs stored as a bitset over the stable root order.
///
/// This is the crate's analogue of the upstream `RootNbrSet` bitmap: ladder
/// tables such as [`RootSystem::min_roots_for`] are precomputed per root and
/// shared by reference, so the set itself stays a read-only value.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RootSet {
    blocks: SmallVec<[u64; 4]>,
    len: usize,
}

struct RootSetIter<'a> {
    blocks: &'a [u64],
    next_block: usize,
    block_index: usize,
    pending: u64,
}

impl Iterator for RootSetIter<'_> {
    type Item = RootId;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.pending != 0 {
                let bit = self.pending.trailing_zeros() as usize;
                self.pending &= self.pending - 1;
                return Some(RootId(self.block_index * 64 + bit));
            }
            let block_index = self.next_block;
            self.pending = *self.blocks.get(block_index)?;
            self.next_block += 1;
            self.block_index = block_index;
        }
    }
}

impl RootSet {
    /// An empty set over a universe of `count` root IDs.
    fn with_capacity(count: usize) -> Result<Self, StructureError> {
        let block_count = count.div_ceil(64);
        let mut blocks = SmallVec::new();
        blocks
            .try_reserve_exact(block_count)
            .map_err(|_| StructureError::AllocationFailed {
                requested: block_count,
            })?;
        blocks.resize(block_count, 0);
        Ok(Self { blocks, len: 0 })
    }

    fn insert(&mut self, id: RootId) {
        let block = id.0 / 64;
        let bit = id.0 % 64;
        if self.blocks[block] & (1u64 << bit) == 0 {
            self.blocks[block] |= 1u64 << bit;
            self.len += 1;
        }
    }

    /// Whether `id` is a member.
    pub fn contains(&self, id: RootId) -> bool {
        self.blocks
            .get(id.0 / 64)
            .is_some_and(|block| block & (1u64 << (id.0 % 64)) != 0)
    }

    /// Number of members.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Whether the set is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Members in ascending stable root order.
    pub fn iter(&self) -> impl Iterator<Item = RootId> + '_ {
        self.iter_nonzero_bits()
    }

    fn iter_nonzero_bits(&self) -> RootSetIter<'_> {
        RootSetIter {
            blocks: &self.blocks,
            next_block: 0,
            block_index: 0,
            pending: 0,
        }
    }
}

/// The finite ordinary root system generated from a based root datum.
///
/// Roots, coroots, and simple-root coordinates are index-aligned under
/// [`RootId`], and `roots` stays ascending in lexicographic coordinate order
/// because [`RootSystem::id_of`] binary-searches it. Enumeration is an
/// explicit, caller-budgeted operation: the compatibility wrapper derives
/// only non-cardinality limits, so the caller's root cardinality stays
/// authoritative, and no process-wide limit exists. The budget that produced
/// a system is deliberately not stored in it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RootSystem {
    datum: BasedRootDatum,
    roots: Vec<Weight>,
    coroots: Vec<Coweight>,
    simple_coordinates: Vec<Vec<i32>>,
    /// Positivity per root, precomputed because `roots` sorts by AMBIENT
    /// coordinates: positivity lives in the simple-coordinate basis, so no
    /// half-split shortcut exists.
    positive: Vec<bool>,
    /// Stable IDs of positive roots in ambient root order. Consumers that
    /// only classify positive roots can skip the negative half of the system.
    positive_root_ids: Box<[RootId]>,
    /// Simple-coordinate heights aligned with `positive_root_ids`.
    positive_root_heights: Box<[i64]>,
    /// Stable IDs of the simple roots in generator order, so descent
    /// queries need no per-call binary search.
    simple_ids: Vec<RootId>,
    /// Ladder-bottom sets per root, precomputed like the upstream
    /// `RootSystem` constructor's `d_minRoots`/`d_minCoroots` tables
    /// (rootdata.h:154-157): `min_roots[alpha]` flags every root `beta`
    /// for which `beta - alpha` is not a root, `min_coroots[alpha]` the
    /// same relation on the paired coroots.
    min_roots: Vec<RootSet>,
    min_coroots: Vec<RootSet>,
    /// Negation table: `negatives[r]` is the id of `-r`.
    negatives: Arc<[RootId]>,
    /// Root permutation of each simple reflection, in generator order.
    /// Recomputing one per `WeylElement::simple_reflection` call (two
    /// rank×rank matrices, then a matvec plus binary search per root —
    /// 240 matvecs per W-word letter on E8) dominated word-heavy scripts,
    /// so the table is built once alongside the negation table, directly
    /// from the closure's reflection formula (the reflection matrix acts
    /// entrywise as `w - <w, coroot> * root`, so the permutations are the
    /// same ones `action_permutation` derives).
    simple_reflections: Vec<Vec<RootId>>,
}

impl RootSystem {
    /// Compatibility wrapper preserving the historic root-cardinality budget.
    ///
    /// It derives the remaining limits with [`RootSystemBudget::complete_for`]
    /// so only cardinality can reject, and maps that rejection back to the
    /// historic [`StructureError::ResourceLimitExceeded`].
    pub fn enumerate(datum: &BasedRootDatum, max_roots: usize) -> Result<Self, StructureError> {
        Self::enumerate_with_budget(datum, &RootSystemBudget::complete_for(datum, max_roots))
            .map_err(|error| match error {
                StructureError::RootSystemResourceLimit {
                    resource: "roots",
                    limit,
                } => StructureError::ResourceLimitExceeded { limit },
                other => other,
            })
    }

    /// Generate ordinary roots and coroots together under an explicit budget.
    pub fn enumerate_with_budget(
        datum: &BasedRootDatum,
        budget: &RootSystemBudget,
    ) -> Result<Self, StructureError> {
        check_budget_consistency(datum, budget)?;
        let snapshot = datum.try_clone()?;
        let mut closure = Closure::new(budget.max_roots);
        for (simple_index, root) in snapshot.simple_roots().iter().enumerate() {
            let coroot = &snapshot.simple_coroots()[simple_index];
            let mut simple_coordinates = try_zero_coordinates(snapshot.semisimple_rank())?;
            simple_coordinates[simple_index] = 1;
            let negative_coordinates = try_negate(&simple_coordinates)?;
            closure.insert(
                Weight::new(try_copy_coordinates(root.as_slice())?),
                Coweight::new(try_copy_coordinates(coroot.as_slice())?),
                simple_coordinates,
            )?;
            closure.insert(
                Weight::new(try_negate(root.as_slice())?),
                Coweight::new(try_negate(coroot.as_slice())?),
                negative_coordinates,
            )?;
        }
        // Scratch buffers reused across all (record, generator) visits: the
        // reflection candidates are ephemeral — only genuinely new roots are
        // copied into the closure — so per-visit allocation dominated this
        // loop (240 roots x 8 generators x ~3 vectors on E8).
        let semisimple_rank = snapshot.semisimple_rank();
        let mut reflected_root = Vec::new();
        let mut reflected_coroot = Vec::new();
        let mut reflected_coordinates = Vec::new();
        let mut record_coroot = Vec::new();
        let mut record_simple_coordinates = Vec::new();
        while let Some(root) = closure.pending.pop_front() {
            {
                let record = closure.seen.get(root.as_ref()).ok_or(
                    StructureError::RootSystemInvariantViolation {
                        invariant: "pending root membership",
                    },
                )?;
                record_coroot.clear();
                record_coroot
                    .try_reserve_exact(record.coroot.len())
                    .map_err(|_| StructureError::AllocationFailed {
                        requested: record.coroot.len(),
                    })?;
                record_coroot.extend_from_slice(&record.coroot);
                record_simple_coordinates.clear();
                record_simple_coordinates
                    .try_reserve_exact(record.simple_coordinates.len())
                    .map_err(|_| StructureError::AllocationFailed {
                        requested: record.simple_coordinates.len(),
                    })?;
                record_simple_coordinates.extend_from_slice(&record.simple_coordinates);
            }
            for generator in 0..semisimple_rank {
                let simple_root = &snapshot.simple_roots()[generator];
                let simple_coroot = &snapshot.simple_coroots()[generator];
                let coefficient = pair_coordinates(root.as_ref(), simple_coroot.as_slice())?;
                let dual_coefficient =
                    pair_coordinates(simple_root.as_slice(), record_coroot.as_slice())?;
                // A generator orthogonal to the record on both sides reflects
                // root, coroot, and simple coordinates to themselves, so the
                // candidate is the already-stored record and no error path is
                // reachable; skipping it is observationally identical.
                if coefficient == 0 && dual_coefficient == 0 {
                    continue;
                }
                reflect_coordinates_into(
                    root.as_ref(),
                    simple_root.as_slice(),
                    coefficient,
                    &mut reflected_root,
                )?;
                reflect_coordinates_into(
                    record_coroot.as_slice(),
                    simple_coroot.as_slice(),
                    dual_coefficient,
                    &mut reflected_coroot,
                )?;
                reflected_coordinates.clear();
                reflected_coordinates
                    .try_reserve_exact(record_simple_coordinates.len())
                    .map_err(|_| StructureError::AllocationFailed {
                        requested: record_simple_coordinates.len(),
                    })?;
                reflected_coordinates.extend_from_slice(&record_simple_coordinates);
                reflected_coordinates[generator] = reflected_coordinates[generator]
                    .checked_sub(coefficient)
                    .ok_or(StructureError::ArithmeticOverflow)?;
                closure.insert_coordinates(
                    &reflected_root,
                    &reflected_coroot,
                    &reflected_coordinates,
                )?;
            }
        }
        Self::from_closure(snapshot, closure)
    }

    fn from_closure(datum: BasedRootDatum, closure: Closure) -> Result<Self, StructureError> {
        debug_assert!(closure.pending.is_empty());
        let count = closure.seen.len();
        let mut roots = Vec::new();
        roots
            .try_reserve_exact(count)
            .map_err(|_| StructureError::AllocationFailed { requested: count })?;
        let mut coroots = Vec::new();
        coroots
            .try_reserve_exact(count)
            .map_err(|_| StructureError::AllocationFailed { requested: count })?;
        let mut simple_coordinates = Vec::new();
        simple_coordinates
            .try_reserve_exact(count)
            .map_err(|_| StructureError::AllocationFailed { requested: count })?;
        for (coordinates, record) in closure.into_sorted_records()? {
            roots.push(Weight::new(coordinates));
            coroots.push(Coweight::new(record.coroot));
            simple_coordinates.push(record.simple_coordinates);
        }
        debug_assert_eq!(roots.len(), coroots.len());
        debug_assert_eq!(roots.len(), simple_coordinates.len());
        let mut positive = Vec::new();
        positive
            .try_reserve_exact(count)
            .map_err(|_| StructureError::AllocationFailed { requested: count })?;
        let mut positive_root_ids = Vec::new();
        positive_root_ids
            .try_reserve_exact(count / 2)
            .map_err(|_| StructureError::AllocationFailed {
                requested: count / 2,
            })?;
        let mut positive_root_heights = Vec::new();
        positive_root_heights
            .try_reserve_exact(count / 2)
            .map_err(|_| StructureError::AllocationFailed {
                requested: count / 2,
            })?;
        for (index, coordinates) in simple_coordinates.iter().enumerate() {
            let is_positive = coordinates.iter().any(|&value| value > 0);
            positive.push(is_positive);
            if is_positive {
                positive_root_ids.push(RootId(index));
                let height = coordinates.iter().try_fold(0_i64, |sum, &value| {
                    sum.checked_add(i64::from(value))
                        .ok_or(StructureError::ArithmeticOverflow)
                })?;
                positive_root_heights.push(height);
            }
        }
        let semisimple_rank = datum.semisimple_rank();
        let mut simple_ids = Vec::new();
        simple_ids.try_reserve_exact(semisimple_rank).map_err(|_| {
            StructureError::AllocationFailed {
                requested: semisimple_rank,
            }
        })?;
        for simple_root in datum.simple_roots() {
            let id = roots
                .binary_search_by(|candidate| candidate.as_slice().cmp(simple_root.as_slice()))
                .ok()
                .map(RootId)
                .ok_or(StructureError::RootSystemInvariantViolation {
                    invariant: "simple-root membership",
                })?;
            simple_ids.push(id);
        }
        let (min_roots, min_coroots) = build_ladder_bottoms(&roots, &coroots)?;
        // Negation table: `negatives[r]` is the id of `-r` (every root's
        // negative is a root). Precomputed once so involution classification
        // reads it instead of re-deriving the negative per root.
        let mut negatives = Vec::new();
        negatives
            .try_reserve_exact(count)
            .map_err(|_| StructureError::AllocationFailed { requested: count })?;
        // `negated` is reused across roots: only the binary search reads it.
        let mut negated = Vec::new();
        for root in &roots {
            negated.clear();
            negated
                .try_reserve_exact(root.as_slice().len())
                .map_err(|_| StructureError::AllocationFailed {
                    requested: root.as_slice().len(),
                })?;
            for &coordinate in root.as_slice() {
                negated.push(
                    coordinate
                        .checked_neg()
                        .ok_or(StructureError::ArithmeticOverflow)?,
                );
            }
            negatives.push(
                roots
                    .binary_search_by(|candidate| candidate.as_slice().cmp(negated.as_slice()))
                    .ok()
                    .map(RootId)
                    .ok_or(StructureError::RootSystemInvariantViolation {
                        invariant: "root negation closure",
                    })?,
            );
        }
        let mut system = Self {
            datum,
            roots,
            coroots,
            simple_coordinates,
            positive,
            positive_root_ids: positive_root_ids.into_boxed_slice(),
            positive_root_heights: positive_root_heights.into_boxed_slice(),
            simple_ids,
            min_roots,
            min_coroots,
            negatives: Arc::from(negatives.into_boxed_slice()),
            simple_reflections: Vec::new(),
        };
        // One-time fill straight from the reflection formula. The previous
        // matrix path (a datum clone and two rank×rank matrices per
        // generator, then one allocating matvec per root) cost more than
        // the enumeration itself on small data; `image` is reused across
        // all (generator, root) pairs.
        let mut simple_reflections = Vec::new();
        simple_reflections
            .try_reserve_exact(semisimple_rank)
            .map_err(|_| StructureError::AllocationFailed {
                requested: semisimple_rank,
            })?;
        let mut image = Vec::new();
        for generator in 0..semisimple_rank {
            let simple_root = system.datum.simple_roots()[generator].as_slice();
            let simple_coroot = system.datum.simple_coroots()[generator].as_slice();
            let mut permutation = Vec::new();
            permutation
                .try_reserve_exact(count)
                .map_err(|_| StructureError::AllocationFailed { requested: count })?;
            for root in &system.roots {
                let coefficient = pair_coordinates(root.as_slice(), simple_coroot)?;
                reflect_coordinates_into(root.as_slice(), simple_root, coefficient, &mut image)?;
                permutation.push(
                    system
                        .id_of_slice(&image)
                        .ok_or(StructureError::InvalidRootAutomorphism)?,
                );
            }
            simple_reflections.push(permutation);
        }
        system.simple_reflections = simple_reflections;
        Ok(system)
    }

    pub fn lattice_rank(&self) -> usize {
        self.datum.lattice_rank()
    }

    pub fn datum(&self) -> &BasedRootDatum {
        &self.datum
    }

    pub fn roots(&self) -> &[Weight] {
        &self.roots
    }

    pub fn root(&self, id: RootId) -> Option<&Weight> {
        self.roots.get(id.0)
    }

    /// The coroot paired with a root during closure, in ambient coordinates.
    pub fn coroot(&self, id: RootId) -> Option<&Coweight> {
        self.coroots.get(id.0)
    }

    /// Index-aligned `(id, root, coroot)` triples in the stable root order.
    pub fn entries(&self) -> impl ExactSizeIterator<Item = (RootId, &Weight, &Coweight)> + '_ {
        self.roots
            .iter()
            .zip(&self.coroots)
            .enumerate()
            .map(|(index, (root, coroot))| (RootId(index), root, coroot))
    }

    /// `<root(root), coroot(coroot)>` in Atlas's root-left, coroot-right
    /// argument order.
    pub fn bracket(&self, root: RootId, coroot: RootId) -> Result<i32, StructureError> {
        let root_value = self.root(root).ok_or(StructureError::IndexOutOfRange {
            index: root.0,
            upper_bound: self.roots.len(),
        })?;
        let coroot_value = self.coroot(coroot).ok_or(StructureError::IndexOutOfRange {
            index: coroot.0,
            upper_bound: self.coroots.len(),
        })?;
        pair(root_value, coroot_value)
    }

    /// Coordinates of a root in the datum's simple-root basis.
    pub fn simple_coordinates(&self, id: RootId) -> Option<&[i32]> {
        self.simple_coordinates.get(id.0).map(Vec::as_slice)
    }

    /// Permutation induced by a Weyl action on these stable root IDs.
    ///
    /// Coroot transport is not re-checked here: every [`WeylAction`] is a
    /// word in simple reflections whose coweight generator is the same dual
    /// reflection the closure used, so transport is a consequence of the
    /// closure's coroot-agreement invariant.
    pub fn action_permutation(&self, action: &WeylAction) -> Result<Vec<RootId>, StructureError> {
        if action.datum() != self.datum() {
            return Err(StructureError::DatumMismatch);
        }
        self.roots
            .iter()
            .map(|root| {
                let image = action.act(root)?;
                self.id_of(&image)
                    .ok_or(StructureError::InvalidRootAutomorphism)
            })
            .collect()
    }

    /// Positivity per root, index-aligned under [`RootId`].
    pub(crate) fn positivity(&self) -> &[bool] {
        &self.positive
    }

    /// Stable IDs of positive roots in ambient root order.
    pub(crate) fn positive_root_ids(&self) -> &[RootId] {
        &self.positive_root_ids
    }

    /// Simple-coordinate heights aligned with [`Self::positive_root_ids`].
    pub(crate) fn positive_root_heights(&self) -> &[i64] {
        &self.positive_root_heights
    }

    /// Whether a root is positive in the simple-root basis.
    pub fn is_positive(&self, id: RootId) -> Option<bool> {
        self.positive.get(id.0).copied()
    }

    /// Stable IDs of the simple roots, in generator order.
    pub(crate) fn simple_root_ids(&self) -> &[RootId] {
        &self.simple_ids
    }

    pub fn id_of(&self, root: &Weight) -> Option<RootId> {
        self.id_of_slice(root.as_slice())
    }

    /// `id_of` on bare coordinates, for bulk classification loops that hold
    /// the image in a reusable buffer instead of a fresh `Weight`.
    pub(crate) fn id_of_slice(&self, root: &[i32]) -> Option<RootId> {
        self.roots
            .binary_search_by(|candidate| candidate.as_slice().cmp(root))
            .ok()
            .map(RootId)
    }

    /// The negation table: `negatives()[r]` is the id of `-r`.
    pub(crate) fn negatives(&self) -> &[RootId] {
        &self.negatives
    }

    /// Shared immutable negation storage for bulk involution records.
    pub(crate) fn negatives_arc(&self) -> &Arc<[RootId]> {
        &self.negatives
    }

    /// Root permutation of simple reflection `generator` (the table built
    /// at construction); `None` when `generator` is out of range.
    pub(crate) fn simple_reflection_permutation(&self, generator: usize) -> Option<&[RootId]> {
        self.simple_reflections.get(generator).map(Vec::as_slice)
    }

    /// Ladder-bottom roots for `alpha`: every root `beta` such that
    /// `beta - alpha` is not a root (`alpha` itself included), in the
    /// stable root order. This is the upstream `RootSystem::min_roots_for`
    /// relation (rootdata.h:270-271) behind `root_ladder_bottoms`.
    pub fn min_roots_for(&self, alpha: RootId) -> Option<&RootSet> {
        self.min_roots.get(alpha.0)
    }

    /// Ladder-bottom coroots for `alpha`, the [`RootSystem::min_roots_for`]
    /// relation transported to the paired coroots (upstream
    /// `min_coroots_for`, rootdata.h:272-273): `beta` is flagged exactly
    /// when `coroot(beta) - coroot(alpha)` is not a coroot.
    pub fn min_coroots_for(&self, alpha: RootId) -> Option<&RootSet> {
        self.min_coroots.get(alpha.0)
    }
}

/// Precompute both ladder-bottom tables: `min_roots[alpha]` flags `beta`
/// when `roots[beta] - roots[alpha]` is not a root, and `min_coroots`
/// likewise on coroot coordinates. Both membership passes use a monotone
/// merge cursor: for fixed `alpha`, subtracting its coordinates preserves the
/// lexicographic order of the sorted candidate sequence. Coroot candidates
/// therefore run in `coroot_order`, while their original root IDs are inserted
/// into the result bitset.
#[inline]
fn merge_root_cursor_contains(roots: &[Weight], difference: &[i32], cursor: &mut usize) -> bool {
    while *cursor < roots.len() && roots[*cursor].as_slice() < difference {
        *cursor += 1;
    }
    *cursor < roots.len() && roots[*cursor].as_slice() == difference
}

#[inline]
fn merge_coroot_cursor_contains(
    coroot_order: &[usize],
    coroots: &[Coweight],
    difference: &[i32],
    cursor: &mut usize,
) -> bool {
    while *cursor < coroot_order.len() && coroots[coroot_order[*cursor]].as_slice() < difference {
        *cursor += 1;
    }
    *cursor < coroot_order.len() && coroots[coroot_order[*cursor]].as_slice() == difference
}

fn build_ladder_bottoms(
    roots: &[Weight],
    coroots: &[Coweight],
) -> Result<(Vec<RootSet>, Vec<RootSet>), StructureError> {
    let count = roots.len();
    let mut coroot_order = Vec::new();
    coroot_order
        .try_reserve_exact(count)
        .map_err(|_| StructureError::AllocationFailed { requested: count })?;
    coroot_order.extend(0..count);
    coroot_order.sort_by(|&left, &right| coroots[left].as_slice().cmp(coroots[right].as_slice()));
    let mut min_roots = Vec::new();
    min_roots
        .try_reserve_exact(count)
        .map_err(|_| StructureError::AllocationFailed { requested: count })?;
    let mut min_coroots = Vec::new();
    min_coroots
        .try_reserve_exact(count)
        .map_err(|_| StructureError::AllocationFailed { requested: count })?;
    // Keep root and coroot subtraction interleaved as in the historical loop
    // so checked-arithmetic errors retain their original precedence. Coroot
    // membership is scanned later in `coroot_order`, using these validated
    // differences as a flat beta-indexed scratch buffer.
    let ambient_rank = roots.first().map_or(0, Weight::rank);
    debug_assert!(coroots.iter().all(|coroot| coroot.rank() == ambient_rank));
    let scratch_entries = count
        .checked_mul(ambient_rank)
        .ok_or(StructureError::ArithmeticOverflow)?;
    let mut coroot_differences = Vec::new();
    coroot_differences
        .try_reserve_exact(scratch_entries)
        .map_err(|_| StructureError::AllocationFailed {
            requested: scratch_entries,
        })?;
    coroot_differences.resize(scratch_entries, 0);
    let mut difference = Vec::new();
    for alpha in 0..count {
        let mut root_bottoms = RootSet::with_capacity(count)?;
        let mut coroot_bottoms = RootSet::with_capacity(count)?;
        let mut root_cursor = 0;
        for beta in 0..count {
            subtract_coordinates(
                roots[beta].as_slice(),
                roots[alpha].as_slice(),
                &mut difference,
            )?;
            if !merge_root_cursor_contains(&roots, &difference, &mut root_cursor) {
                root_bottoms.insert(RootId(beta));
            }
            subtract_coordinates(
                coroots[beta].as_slice(),
                coroots[alpha].as_slice(),
                &mut difference,
            )?;
            let start = beta
                .checked_mul(ambient_rank)
                .ok_or(StructureError::ArithmeticOverflow)?;
            let end = start
                .checked_add(ambient_rank)
                .ok_or(StructureError::ArithmeticOverflow)?;
            coroot_differences[start..end].copy_from_slice(difference.as_slice());
        }
        let mut coroot_cursor = 0;
        for &beta in &coroot_order {
            let start = beta
                .checked_mul(ambient_rank)
                .ok_or(StructureError::ArithmeticOverflow)?;
            let end = start
                .checked_add(ambient_rank)
                .ok_or(StructureError::ArithmeticOverflow)?;
            if !merge_coroot_cursor_contains(
                &coroot_order,
                coroots,
                &coroot_differences[start..end],
                &mut coroot_cursor,
            ) {
                coroot_bottoms.insert(RootId(beta));
            }
        }
        min_roots.push(root_bottoms);
        min_coroots.push(coroot_bottoms);
    }
    Ok((min_roots, min_coroots))
}

/// `out = left - right` entrywise, with checked subtraction.
fn subtract_coordinates(
    left: &[i32],
    right: &[i32],
    out: &mut Vec<i32>,
) -> Result<(), StructureError> {
    debug_assert_eq!(left.len(), right.len());
    out.clear();
    out.try_reserve_exact(left.len())
        .map_err(|_| StructureError::AllocationFailed {
            requested: left.len(),
        })?;
    for (&a, &b) in left.iter().zip(right) {
        out.push(a.checked_sub(b).ok_or(StructureError::ArithmeticOverflow)?);
    }
    Ok(())
}

/// `out = values - coefficient * direction` entrywise, reusing `out`'s
/// allocation. The inputs are all `i32`; an `i64` intermediate therefore
/// covers the full product and subtraction range while retaining the same
/// `i32` conversion errors as the wider datum reflection path.
fn reflect_coordinates_into(
    values: &[i32],
    direction: &[i32],
    coefficient: i32,
    out: &mut Vec<i32>,
) -> Result<(), StructureError> {
    debug_assert_eq!(values.len(), direction.len());
    out.clear();
    out.try_reserve_exact(values.len())
        .map_err(|_| StructureError::AllocationFailed {
            requested: values.len(),
        })?;
    for (&coordinate, &direction_coordinate) in values.iter().zip(direction) {
        let correction = i64::from(coefficient) * i64::from(direction_coordinate);
        let value = i64::from(coordinate) - correction;
        out.push(i32::try_from(value).map_err(|_| StructureError::ArithmeticOverflow)?);
    }
    Ok(())
}

/// One closure record: a root key with its paired coroot and simple-root
/// coordinates.
struct ClosureRecord {
    coroot: Vec<i32>,
    simple_coordinates: Vec<i32>,
}

/// Fast deterministic hasher for short fixed-width root-coordinate keys.
/// Closure lookup is dedup-only, so a compact non-cryptographic hash avoids
/// the per-probe cost of `HashMap`'s randomized default while equality still
/// compares the complete coordinate vector.
#[derive(Clone, Default)]
struct RootCoordinateHasher(u64);

impl Hasher for RootCoordinateHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        const SEED: u64 = 0x51_7c_c1_b7_27_22_0a_95;
        let mut chunks = bytes.chunks_exact(8);
        for chunk in &mut chunks {
            let value = u64::from_le_bytes(chunk.try_into().unwrap_or([0; 8]));
            self.0 = (self.0.rotate_left(5) ^ value).wrapping_mul(SEED);
        }
        let remainder = chunks.remainder();
        if !remainder.is_empty() {
            let mut tail = 0_u64;
            for &byte in remainder {
                tail = (tail << 8) | u64::from(byte);
            }
            self.0 = (self.0.rotate_left(5) ^ tail).wrapping_mul(SEED);
        }
    }
}

type RootCoordinateHasherBuilder = BuildHasherDefault<RootCoordinateHasher>;

/// Private closure state shared by production enumeration and the invariant
/// tests, which inject candidate records directly.
struct Closure {
    max_roots: usize,
    seen: HashMap<Arc<[i32]>, ClosureRecord, RootCoordinateHasherBuilder>,
    pending: VecDeque<Arc<[i32]>>,
}

impl Closure {
    fn new(max_roots: usize) -> Self {
        Self {
            max_roots,
            seen: HashMap::with_hasher(RootCoordinateHasherBuilder::default()),
            pending: VecDeque::new(),
        }
    }

    /// Insert one candidate `(root, coroot, simple-coordinate)` record.
    ///
    /// The candidate arrives as data and is never recomputed here. A
    /// duplicate root key must agree exactly with the stored record, and
    /// every candidate must satisfy `<alpha, alpha_vee> = 2`; both checks are
    /// defensive, since no public datum constructor is known to reach them.
    fn insert(
        &mut self,
        root: Weight,
        coroot: Coweight,
        simple_coordinates: Vec<i32>,
    ) -> Result<(), StructureError> {
        self.insert_coordinates(root.as_slice(), coroot.as_slice(), &simple_coordinates)
    }

    /// Slice form of [`Closure::insert`] for enumeration candidates held in
    /// reused scratch buffers: only genuinely new roots are copied into the
    /// map and the pending queue.
    fn insert_coordinates(
        &mut self,
        root: &[i32],
        coroot: &[i32],
        simple_coordinates: &[i32],
    ) -> Result<(), StructureError> {
        if pair_coordinates(root, coroot)? != 2 {
            return Err(StructureError::RootSystemInvariantViolation {
                invariant: "self pairing",
            });
        }
        if let Some(existing) = self.seen.get(root) {
            if existing.coroot != coroot || existing.simple_coordinates != simple_coordinates {
                return Err(StructureError::RootSystemInvariantViolation {
                    invariant: "coroot agreement",
                });
            }
            return Ok(());
        }
        if self.seen.len() == self.max_roots {
            return Err(StructureError::RootSystemResourceLimit {
                resource: "roots",
                limit: self.max_roots,
            });
        }
        // Keep one owned coordinate allocation for both the dedup key and the
        // pending work item. The queue only needs a shared handle while the
        // map retains the paired coroot/simple-coordinate record.
        let key: Arc<[i32]> = Arc::try_from(try_copy_coordinates(root)?).map_err(|_| {
            StructureError::AllocationFailed {
                requested: root.len(),
            }
        })?;
        let record = ClosureRecord {
            coroot: try_copy_coordinates(coroot)?,
            simple_coordinates: try_copy_coordinates(simple_coordinates)?,
        };
        self.seen.insert(Arc::clone(&key), record);
        self.pending
            .try_reserve(1)
            .map_err(|_| StructureError::AllocationFailed { requested: 1 })?;
        self.pending.push_back(key);
        Ok(())
    }

    /// Consume the deduplication map in the deterministic root order exposed
    /// by [`RootSystem::roots`]. Hashing keeps insertion and lookup cheap;
    /// sorting once at the boundary preserves the historical B-tree order.
    fn into_sorted_records(self) -> Result<Vec<(Vec<i32>, ClosureRecord)>, StructureError> {
        let count = self.seen.len();
        let mut records = Vec::new();
        records
            .try_reserve_exact(count)
            .map_err(|_| StructureError::AllocationFailed { requested: count })?;
        records.extend(
            self.seen
                .into_iter()
                .map(|(coordinates, record)| (coordinates.to_vec(), record)),
        );
        records.sort_unstable_by(|(left, _), (right, _)| left.cmp(right));
        Ok(records)
    }
}

/// Check the budget against the datum in the documented fixed order:
/// lattice rank, roots, coordinate entries, reflection steps.
fn check_budget_consistency(
    datum: &BasedRootDatum,
    budget: &RootSystemBudget,
) -> Result<(), StructureError> {
    if datum.lattice_rank() > budget.max_lattice_rank {
        return Err(StructureError::RootSystemResourceLimit {
            resource: "lattice rank",
            limit: budget.max_lattice_rank,
        });
    }
    let semisimple_rank = datum.semisimple_rank();
    if semisimple_rank > 0 && widened(semisimple_rank).saturating_mul(2) > widened(budget.max_roots)
    {
        return Err(StructureError::RootSystemResourceLimit {
            resource: "roots",
            limit: budget.max_roots,
        });
    }
    if saturated_to_usize(entry_bound(datum, budget.max_roots)) > budget.max_coordinate_entries {
        return Err(StructureError::RootSystemResourceLimit {
            resource: "coordinate entries",
            limit: budget.max_coordinate_entries,
        });
    }
    if saturated_to_usize(step_bound(datum, budget.max_roots)) > budget.max_reflection_steps {
        return Err(StructureError::RootSystemResourceLimit {
            resource: "reflection steps",
            limit: budget.max_reflection_steps,
        });
    }
    Ok(())
}

/// Effective record count for the entry and step bounds: a pure torus seeds
/// nothing, so its closure allocates no records at all.
fn effective_roots(datum: &BasedRootDatum, max_roots: usize) -> u128 {
    if datum.semisimple_rank() == 0 {
        0
    } else {
        widened(max_roots)
    }
}

/// Worst-case live coordinate entries at `max_roots`:
/// `2 (r^2 + 2 r n) + (R_eff + 1)(4 n + 2 r)`, covering the borrowed caller
/// datum, the owned snapshot, the map and pending queue, and one in-flight
/// candidate record.
fn entry_bound(datum: &BasedRootDatum, max_roots: usize) -> u128 {
    let n = widened(datum.lattice_rank());
    let r = widened(datum.semisimple_rank());
    let datum_entries = r
        .saturating_mul(r)
        .saturating_add(2u128.saturating_mul(r).saturating_mul(n));
    let per_record = 4u128
        .saturating_mul(n)
        .saturating_add(2u128.saturating_mul(r));
    effective_roots(datum, max_roots)
        .saturating_add(1)
        .saturating_mul(per_record)
        .saturating_add(datum_entries.saturating_mul(2))
}

/// Worst-case dual-reflection visits: each accepted record is popped once and
/// visited by every generator, so the bound is `R_eff * r`.
fn step_bound(datum: &BasedRootDatum, max_roots: usize) -> u128 {
    effective_roots(datum, max_roots).saturating_mul(widened(datum.semisimple_rank()))
}

fn widened(value: usize) -> u128 {
    u128::try_from(value).unwrap_or(u128::MAX)
}

fn saturated_to_usize(value: u128) -> usize {
    usize::try_from(value).unwrap_or(usize::MAX)
}

/// The id of `left + right` (or `left - right`), when that vector is a root.
pub(crate) fn combine_roots(
    root_system: &RootSystem,
    left: RootId,
    right: RootId,
    subtract: bool,
) -> Result<Option<RootId>, StructureError> {
    let left_weight = root_system
        .root(left)
        .ok_or(StructureError::IndexOutOfRange {
            index: left.0,
            upper_bound: root_system.roots().len(),
        })?;
    let right_weight = root_system
        .root(right)
        .ok_or(StructureError::IndexOutOfRange {
            index: right.0,
            upper_bound: root_system.roots().len(),
        })?;
    let mut coordinates = Vec::new();
    coordinates
        .try_reserve_exact(left_weight.as_slice().len())
        .map_err(|_| StructureError::AllocationFailed {
            requested: left_weight.as_slice().len(),
        })?;
    for (&a, &b) in left_weight.as_slice().iter().zip(right_weight.as_slice()) {
        let value = if subtract {
            a.checked_sub(b)
        } else {
            a.checked_add(b)
        }
        .ok_or(StructureError::ArithmeticOverflow)?;
        coordinates.push(value);
    }
    Ok(root_system.id_of(&Weight::new(coordinates)))
}

fn try_zero_coordinates(rank: usize) -> Result<Vec<i32>, StructureError> {
    let mut coordinates = Vec::new();
    coordinates
        .try_reserve_exact(rank)
        .map_err(|_| StructureError::AllocationFailed { requested: rank })?;
    coordinates.resize(rank, 0);
    Ok(coordinates)
}

fn try_negate(values: &[i32]) -> Result<Vec<i32>, StructureError> {
    let mut negated = Vec::new();
    negated
        .try_reserve_exact(values.len())
        .map_err(|_| StructureError::AllocationFailed {
            requested: values.len(),
        })?;
    for &value in values {
        negated.push(
            value
                .checked_neg()
                .ok_or(StructureError::ArithmeticOverflow)?,
        );
    }
    Ok(negated)
}

#[cfg(test)]
mod tests {
    use crate::WeylGroup;

    use super::*;

    fn a2() -> BasedRootDatum {
        BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap()
    }

    #[test]
    fn enumerates_a2_in_deterministic_coordinate_order() {
        let datum = a2();
        let roots = RootSystem::enumerate(&datum, 6).unwrap();
        assert_eq!(
            roots.roots(),
            &[
                Weight::new(vec![-1, -1]),
                Weight::new(vec![-1, 0]),
                Weight::new(vec![0, -1]),
                Weight::new(vec![0, 1]),
                Weight::new(vec![1, 0]),
                Weight::new(vec![1, 1]),
            ]
        );
        assert_eq!(
            roots.simple_coordinates(roots.id_of(&Weight::new(vec![1, 1])).unwrap()),
            Some(&[1, 1][..])
        );
    }

    #[test]
    fn positivity_and_simple_ids_are_precomputed() {
        let roots = RootSystem::enumerate(&a2(), 6).unwrap();
        let positive_count = roots.positivity().iter().filter(|&&flag| flag).count();
        assert_eq!(positive_count, 3);
        assert_eq!(roots.simple_root_ids().len(), 2);
        for (index, id) in roots.simple_root_ids().iter().enumerate() {
            let mut expected = [0; 2];
            expected[index] = 1;
            assert_eq!(roots.simple_coordinates(*id), Some(&expected[..]));
            assert_eq!(roots.is_positive(*id), Some(true));
        }
        for (id, _, _) in roots.entries() {
            let coordinates = roots.simple_coordinates(id).unwrap();
            assert_eq!(
                roots.is_positive(id),
                Some(coordinates.iter().any(|&value| value > 0))
            );
        }
    }

    #[test]
    fn positive_root_index_preserves_ambient_order_and_heights() {
        let roots = RootSystem::enumerate(&a2(), 6).unwrap();
        assert_eq!(roots.positive_root_ids(), &[RootId(3), RootId(4), RootId(5)]);
        assert_eq!(roots.positive_root_heights(), &[1, 1, 2]);
    }

    #[test]
    fn budget_is_an_explicit_resource_error() {
        let datum = BasedRootDatum::standard(vec![vec![2]]).unwrap();
        assert_eq!(
            RootSystem::enumerate(&datum, 1),
            Err(StructureError::ResourceLimitExceeded { limit: 1 })
        );
    }

    #[test]
    fn pairs_a2_coroots_with_roots_in_stable_order() {
        let roots = RootSystem::enumerate(&a2(), 6).unwrap();
        let expected = [
            (vec![-1, -1], vec![-1, -1]),
            (vec![-1, 0], vec![-2, 1]),
            (vec![0, -1], vec![1, -2]),
            (vec![0, 1], vec![-1, 2]),
            (vec![1, 0], vec![2, -1]),
            (vec![1, 1], vec![1, 1]),
        ];
        assert_eq!(roots.entries().len(), expected.len());
        for ((id, root, coroot), (expected_root, expected_coroot)) in roots.entries().zip(&expected)
        {
            assert_eq!(root.as_slice(), expected_root.as_slice());
            assert_eq!(coroot.as_slice(), expected_coroot.as_slice());
            assert_eq!(roots.coroot(id).unwrap().as_slice(), coroot.as_slice());
        }
    }

    #[test]
    fn pairs_non_simply_laced_b2_coroots() {
        let datum = BasedRootDatum::standard(vec![vec![2, -2], vec![-1, 2]]).unwrap();
        let roots = RootSystem::enumerate(&datum, 8).unwrap();
        let long_short = roots.id_of(&Weight::new(vec![1, 1])).unwrap();
        assert_eq!(
            roots.coroot(long_short).unwrap(),
            &Coweight::new(vec![2, 0])
        );
        let short_double = roots.id_of(&Weight::new(vec![1, 2])).unwrap();
        assert_eq!(
            roots.coroot(short_double).unwrap(),
            &Coweight::new(vec![0, 1])
        );
    }

    #[test]
    fn keeps_a_central_coweight_coordinate_for_both_signs() {
        let datum = BasedRootDatum::from_simple_data(
            2,
            vec![vec![2]],
            vec![Weight::new(vec![1, 0])],
            vec![Coweight::new(vec![2, 1])],
        )
        .unwrap();
        let roots = RootSystem::enumerate(&datum, 2).unwrap();
        let positive = roots.id_of(&Weight::new(vec![1, 0])).unwrap();
        let negative = roots.id_of(&Weight::new(vec![-1, 0])).unwrap();
        assert_eq!(roots.coroot(positive).unwrap(), &Coweight::new(vec![2, 1]));
        assert_eq!(
            roots.coroot(negative).unwrap(),
            &Coweight::new(vec![-2, -1])
        );
    }

    /// Members of a ladder-bottom set as root coordinate vectors, sorted.
    fn ladder_member_vectors(roots: &RootSystem, set: &RootSet) -> Vec<Vec<i32>> {
        let mut members: Vec<Vec<i32>> = set
            .iter()
            .map(|id| roots.root(id).unwrap().as_slice().to_vec())
            .collect();
        members.sort();
        members
    }

    // The expected sets below pin the upstream `root_ladder_bottoms` probe
    // values (slice-B fixtures, HPC capture 3535636). Atlas signed root
    // numbers are translated through the upstream positive-root order,
    // which for `simply_connected(B2, prefer_coroots=true)` follows the
    // dual system's generation order: B2 positives are
    // ri = [1,0], [0,1], [1,2], [1,1] and signed -k is -ri[k-1].
    #[test]
    fn b2_min_roots_match_the_oracle_ladder_bottoms() {
        let datum = BasedRootDatum::standard(vec![vec![2, -2], vec![-1, 2]]).unwrap();
        let roots = RootSystem::enumerate(&datum, 8).unwrap();
        // Atlas root 0 = ri[0] = [1,0]: oracle `[-4,-3,-1,0,1,2]`, i.e.
        // {ri0, ri1, ri2, -ri0, -ri2, -ri3}.
        let alpha = roots.id_of(&Weight::new(vec![1, 0])).unwrap();
        let expected = [
            vec![-1, -2],
            vec![-1, -1],
            vec![-1, 0],
            vec![0, 1],
            vec![1, 0],
            vec![1, 2],
        ];
        assert_eq!(
            ladder_member_vectors(&roots, roots.min_roots_for(alpha).unwrap()),
            expected
        );
    }

    #[test]
    fn b2_min_coroots_match_the_oracle_ladder_bottoms() {
        let datum = BasedRootDatum::standard(vec![vec![2, -2], vec![-1, 2]]).unwrap();
        let roots = RootSystem::enumerate(&datum, 8).unwrap();
        // Atlas coroot 0 (paired with ri[0] = [1,0]): oracle `[-4,-1,0,1]`,
        // i.e. {ri0, ri1, -ri0, -ri3}; members are identified by their roots.
        let alpha = roots.id_of(&Weight::new(vec![1, 0])).unwrap();
        let expected = [vec![-1, -1], vec![-1, 0], vec![0, 1], vec![1, 0]];
        assert_eq!(
            ladder_member_vectors(&roots, roots.min_coroots_for(alpha).unwrap()),
            expected
        );
    }

    // For `adjoint(G2, prefer_coroots=false)` the upstream positive-root
    // order is plain generation order on the transposed lietype Cartan:
    // ri = [1,0], [0,1], [1,1], [2,1], [3,1], [3,2].
    #[test]
    fn g2_min_roots_match_the_oracle_ladder_bottoms() {
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-3, 2]]).unwrap();
        let roots = RootSystem::enumerate(&datum, 12).unwrap();
        // Atlas root 5 = ri[5] = [3,2] (highest root): oracle
        // `[-6,-5,-4,-3,-2,-1,0,5]`, i.e. ri0 and ri5 plus every negative.
        let alpha = roots.id_of(&Weight::new(vec![3, 2])).unwrap();
        let expected = [
            vec![-3, -2],
            vec![-3, -1],
            vec![-2, -1],
            vec![-1, -1],
            vec![-1, 0],
            vec![0, -1],
            vec![1, 0],
            vec![3, 2],
        ];
        assert_eq!(
            ladder_member_vectors(&roots, roots.min_roots_for(alpha).unwrap()),
            expected
        );
    }

    #[test]
    fn ladder_bottoms_match_brute_force_subtraction() {
        for datum in [
            BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap(),
            BasedRootDatum::standard(vec![vec![2, -2], vec![-1, 2]]).unwrap(),
            BasedRootDatum::standard(vec![vec![2, -1], vec![-3, 2]]).unwrap(),
        ] {
            let roots = RootSystem::enumerate(&datum, 12).unwrap();
            for (alpha, _, _) in roots.entries() {
                let min_roots = roots.min_roots_for(alpha).unwrap();
                let min_coroots = roots.min_coroots_for(alpha).unwrap();
                assert_eq!(min_roots.len(), min_roots.iter().count());
                for (beta, _, _) in roots.entries() {
                    let root_expected =
                        !matches!(combine_roots(&roots, beta, alpha, true), Ok(Some(_)));
                    assert_eq!(min_roots.contains(beta), root_expected);
                    let difference: Vec<i32> = roots
                        .coroot(beta)
                        .unwrap()
                        .as_slice()
                        .iter()
                        .zip(roots.coroot(alpha).unwrap().as_slice())
                        .map(|(&a, &b)| a - b)
                        .collect();
                    let coroot_expected = !roots
                        .entries()
                        .any(|(_, _, coroot)| coroot.as_slice() == difference);
                    assert_eq!(min_coroots.contains(beta), coroot_expected);
                }
            }
        }
    }

    #[test]
    fn ladder_bottoms_reject_an_out_of_range_id() {
        let roots = RootSystem::enumerate(&a2(), 6).unwrap();
        assert_eq!(roots.min_roots_for(RootId(6)), None);
        assert_eq!(roots.min_coroots_for(RootId(6)), None);
    }

    #[test]
    fn a_pure_torus_has_empty_ladder_tables() {
        let datum = BasedRootDatum::from_simple_data(3, vec![], vec![], vec![]).unwrap();
        let roots = RootSystem::enumerate(&datum, 0).unwrap();
        assert_eq!(roots.min_roots_for(RootId(0)), None);
    }

    #[test]
    fn root_set_keeps_e8_ladder_bitset_inline() {
        let roots = RootSet::with_capacity(240).unwrap();
        assert!(!roots.blocks.spilled());
        assert!(RootSet::with_capacity(257).unwrap().blocks.spilled());
    }

    #[test]
    fn root_set_sparse_iteration_preserves_stable_order_across_blocks() {
        let mut roots = RootSet::with_capacity(257).unwrap();
        for id in [RootId(256), RootId(64), RootId(0), RootId(129), RootId(63)] {
            roots.insert(id);
        }
        assert_eq!(
            roots.iter().collect::<Vec<_>>(),
            vec![RootId(0), RootId(63), RootId(64), RootId(129), RootId(256)]
        );
    }

    #[test]
    fn a_pure_torus_has_no_pairs() {
        let datum = BasedRootDatum::from_simple_data(3, vec![], vec![], vec![]).unwrap();
        let roots = RootSystem::enumerate(&datum, 0).unwrap();
        assert_eq!(roots.entries().len(), 0);
        assert_eq!(roots.roots(), &[]);
    }

    #[test]
    fn every_stored_pair_has_self_bracket_two() {
        for cartan in [
            vec![vec![2, -1], vec![-1, 2]],
            vec![vec![2, -2], vec![-1, 2]],
        ] {
            let datum = BasedRootDatum::standard(cartan).unwrap();
            let roots = RootSystem::enumerate(&datum, 8).unwrap();
            for (id, _, _) in roots.entries() {
                assert_eq!(roots.bracket(id, id), Ok(2));
            }
        }
    }

    #[test]
    fn bracket_rejects_an_out_of_range_id() {
        let roots = RootSystem::enumerate(&a2(), 6).unwrap();
        assert_eq!(
            roots.bracket(RootId(6), RootId(0)),
            Err(StructureError::IndexOutOfRange {
                index: 6,
                upper_bound: 6,
            })
        );
    }

    #[test]
    fn enumerated_weyl_actions_transport_roots_and_coroots_together() {
        let datum = a2();
        let roots = RootSystem::enumerate(&datum, 6).unwrap();
        for action in WeylGroup::new(datum).enumerate_actions(6).unwrap() {
            let permutation = roots.action_permutation(&action).unwrap();
            for (id, _, coroot) in roots.entries() {
                assert_eq!(
                    action.act_on_coweight(coroot).unwrap(),
                    *roots.coroot(permutation[id.0]).unwrap()
                );
            }
        }
    }

    #[test]
    fn rejects_an_injected_coroot_conflict() {
        let mut closure = Closure::new(4);
        closure
            .insert(Weight::new(vec![1, 0]), Coweight::new(vec![2, 0]), vec![1])
            .unwrap();
        assert_eq!(
            closure.insert(Weight::new(vec![1, 0]), Coweight::new(vec![2, 2]), vec![1],),
            Err(StructureError::RootSystemInvariantViolation {
                invariant: "coroot agreement",
            })
        );
    }

    #[test]
    fn pending_roots_share_coordinate_storage_with_seen_keys() {
        let mut closure = Closure::new(4);
        closure
            .insert(Weight::new(vec![1, 0]), Coweight::new(vec![2, 0]), vec![1])
            .unwrap();
        let pending = closure.pending.front().unwrap();
        let key = closure.seen.keys().next().unwrap();
        assert_eq!(Arc::as_ptr(pending), Arc::as_ptr(key));
    }

    #[test]
    fn closure_records_are_sorted_by_root_coordinates() {
        let mut closure = Closure::new(4);
        for coordinate in [[1, 0], [0, 1], [-1, 0]] {
            closure
                .insert(
                    Weight::new(coordinate.to_vec()),
                    Coweight::new(coordinate.iter().map(|&value| 2 * value).collect()),
                    coordinate.to_vec(),
                )
                .unwrap();
        }
        let records = closure.into_sorted_records().unwrap();
        let roots: Vec<Vec<i32>> = records
            .iter()
            .map(|(coordinates, _)| coordinates.clone())
            .collect();
        assert_eq!(roots, vec![vec![-1, 0], vec![0, 1], vec![1, 0]]);
    }

    #[test]
    fn monotone_cursor_membership_matches_binary_search() {
        let ordered = [
            Weight::new(vec![-2, 0]),
            Weight::new(vec![-1, 0]),
            Weight::new(vec![0, 0]),
            Weight::new(vec![1, 0]),
        ];
        let differences = [vec![-3, 0], vec![-1, 0], vec![-1, 0], vec![2, 0]];
        let mut cursor = 0;
        for difference in &differences {
            let expected = ordered
                .binary_search_by(|candidate| candidate.as_slice().cmp(difference))
                .is_ok();
            assert_eq!(
                merge_root_cursor_contains(&ordered, difference, &mut cursor),
                expected
            );
        }

        let coroots = [
            Coweight::new(vec![0, 0]),
            Coweight::new(vec![1, 0]),
            Coweight::new(vec![1, 0]),
            Coweight::new(vec![2, 0]),
        ];
        let coroot_order = [0, 1, 2, 3];
        let mut coroot_cursor = 0;
        for difference in [vec![0, 0], vec![1, 0], vec![1, 0], vec![3, 0]] {
            let expected = coroot_order
                .binary_search_by(|&index| coroots[index].as_slice().cmp(&difference))
                .is_ok();
            assert_eq!(
                merge_coroot_cursor_contains(
                    &coroot_order,
                    &coroots,
                    &difference,
                    &mut coroot_cursor,
                ),
                expected
            );
        }
    }

    fn binary_search_ladder_bottoms_reference(
        roots: &[Weight],
        coroots: &[Coweight],
    ) -> Result<(Vec<RootSet>, Vec<RootSet>), StructureError> {
        let count = roots.len();
        let mut coroot_order: Vec<usize> = (0..count).collect();
        coroot_order
            .sort_by(|&left, &right| coroots[left].as_slice().cmp(coroots[right].as_slice()));
        let mut min_roots = Vec::with_capacity(count);
        let mut min_coroots = Vec::with_capacity(count);
        let mut difference = Vec::new();
        for alpha in 0..count {
            let mut root_bottoms = RootSet::with_capacity(count)?;
            let mut coroot_bottoms = RootSet::with_capacity(count)?;
            for beta in 0..count {
                subtract_coordinates(
                    roots[beta].as_slice(),
                    roots[alpha].as_slice(),
                    &mut difference,
                )?;
                if roots
                    .binary_search_by(|candidate| candidate.as_slice().cmp(difference.as_slice()))
                    .is_err()
                {
                    root_bottoms.insert(RootId(beta));
                }
                subtract_coordinates(
                    coroots[beta].as_slice(),
                    coroots[alpha].as_slice(),
                    &mut difference,
                )?;
                if coroot_order
                    .binary_search_by(|&index| coroots[index].as_slice().cmp(difference.as_slice()))
                    .is_err()
                {
                    coroot_bottoms.insert(RootId(beta));
                }
            }
            min_roots.push(root_bottoms);
            min_coroots.push(coroot_bottoms);
        }
        Ok((min_roots, min_coroots))
    }

    #[test]
    fn ladder_bottom_merge_matches_binary_search_reference() {
        for datum in [
            a2(),
            BasedRootDatum::standard(vec![vec![2, -2], vec![-1, 2]]).unwrap(),
            BasedRootDatum::standard(vec![vec![2, -1], vec![-3, 2]]).unwrap(),
        ] {
            let roots = RootSystem::enumerate(&datum, 12).unwrap();
            let expected =
                binary_search_ladder_bottoms_reference(roots.roots(), &roots.coroots).unwrap();
            let actual = build_ladder_bottoms(roots.roots(), &roots.coroots).unwrap();
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn rejects_a_candidate_without_self_pairing_two() {
        let mut closure = Closure::new(4);
        assert_eq!(
            closure.insert(Weight::new(vec![1, 0]), Coweight::new(vec![1, 0]), vec![1],),
            Err(StructureError::RootSystemInvariantViolation {
                invariant: "self pairing",
            })
        );
    }

    #[test]
    fn each_budget_limit_rejects_with_its_named_resource() {
        let datum = a2();
        assert_eq!(
            RootSystem::enumerate_with_budget(
                &datum,
                &RootSystemBudget::new(1, 6, usize::MAX, usize::MAX)
            ),
            Err(StructureError::RootSystemResourceLimit {
                resource: "lattice rank",
                limit: 1,
            })
        );
        assert_eq!(
            RootSystem::enumerate_with_budget(
                &datum,
                &RootSystemBudget::new(2, 2, usize::MAX, usize::MAX)
            ),
            Err(StructureError::RootSystemResourceLimit {
                resource: "roots",
                limit: 2,
            })
        );
        assert_eq!(
            RootSystem::enumerate_with_budget(&datum, &RootSystemBudget::new(2, 6, 1, usize::MAX)),
            Err(StructureError::RootSystemResourceLimit {
                resource: "coordinate entries",
                limit: 1,
            })
        );
        assert_eq!(
            RootSystem::enumerate_with_budget(&datum, &RootSystemBudget::new(2, 6, 108, 1)),
            Err(StructureError::RootSystemResourceLimit {
                resource: "reflection steps",
                limit: 1,
            })
        );
    }

    #[test]
    fn discovery_beyond_the_root_limit_is_the_named_runtime_rejection() {
        let datum = a2();
        assert_eq!(
            RootSystem::enumerate_with_budget(
                &datum,
                &RootSystemBudget::new(2, 4, usize::MAX, usize::MAX)
            ),
            Err(StructureError::RootSystemResourceLimit {
                resource: "roots",
                limit: 4,
            })
        );
    }

    #[test]
    fn wrapper_behavior_is_preserved() {
        let a1 = BasedRootDatum::standard(vec![vec![2]]).unwrap();
        assert_eq!(
            RootSystem::enumerate(&a1, usize::MAX)
                .unwrap()
                .roots()
                .len(),
            2
        );
        let datum = a2();
        assert_eq!(
            RootSystem::enumerate_with_budget(&datum, &RootSystemBudget::complete_for(&datum, 6))
                .unwrap(),
            RootSystem::enumerate(&datum, 6).unwrap()
        );
    }

    #[test]
    fn cloned_root_systems_share_the_negation_table() {
        let roots = RootSystem::enumerate(&a2(), 6).unwrap();
        let clone = roots.clone();

        assert!(Arc::ptr_eq(&roots.negatives, &clone.negatives));
        assert_eq!(roots.negatives(), clone.negatives());
    }

    #[test]
    fn thirty_three_a1_factors_stay_dynamic_under_a_complete_budget() {
        let rank = 33;
        let mut cartan = vec![vec![0; rank]; rank];
        for (index, row) in cartan.iter_mut().enumerate() {
            row[index] = 2;
        }
        let datum = BasedRootDatum::standard(cartan).unwrap();
        let budget = RootSystemBudget::complete_for(&datum, 2 * rank);
        let roots = RootSystem::enumerate_with_budget(&datum, &budget).unwrap();
        assert_eq!(roots.roots().len(), 2 * rank);
        for (id, _, _) in roots.entries() {
            assert_eq!(roots.bracket(id, id), Ok(2));
        }
    }

    #[test]
    fn a_seed_coroot_at_the_negation_boundary_is_a_checked_overflow() {
        let datum = BasedRootDatum::from_simple_data(
            2,
            vec![vec![2]],
            vec![Weight::new(vec![1, 0])],
            vec![Coweight::new(vec![2, i32::MIN])],
        )
        .unwrap();
        assert_eq!(
            RootSystem::enumerate(&datum, 2),
            Err(StructureError::ArithmeticOverflow)
        );
    }
}
