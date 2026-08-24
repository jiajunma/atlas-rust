use std::collections::{BTreeMap, VecDeque};

use crate::lattice::{pair_coordinates, try_copy_coordinates};
use crate::{pair, BasedRootDatum, Coweight, StructureError, Weight, WeylAction};

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
    blocks: Vec<u64>,
    len: usize,
}

impl RootSet {
    /// An empty set over a universe of `count` root IDs.
    fn with_capacity(count: usize) -> Result<Self, StructureError> {
        let block_count = count.div_ceil(64);
        let mut blocks = Vec::new();
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
        self.blocks
            .iter()
            .enumerate()
            .flat_map(|(block_index, &block)| {
                (0..64).filter_map(move |bit| {
                    if block & (1u64 << bit) != 0 {
                        Some(RootId(block_index * 64 + bit))
                    } else {
                        None
                    }
                })
            })
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
    negatives: Vec<RootId>,
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
        while let Some(record) = closure.pending.pop_front() {
            for generator in 0..snapshot.semisimple_rank() {
                let coefficient = pair(&record.root, &snapshot.simple_coroots()[generator])?;
                let mut reflected_coordinates = try_copy_coordinates(&record.simple_coordinates)?;
                reflected_coordinates[generator] = reflected_coordinates[generator]
                    .checked_sub(coefficient)
                    .ok_or(StructureError::ArithmeticOverflow)?;
                closure.insert(
                    snapshot.reflect_weight(generator, &record.root)?,
                    snapshot.reflect_coweight(generator, &record.coroot)?,
                    reflected_coordinates,
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
        for (coordinates, record) in closure.seen {
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
        for coordinates in &simple_coordinates {
            positive.push(coordinates.iter().any(|&value| value > 0));
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
        for root in &roots {
            let mut negated = Vec::with_capacity(root.as_slice().len());
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
        Ok(Self {
            datum,
            roots,
            coroots,
            simple_coordinates,
            positive,
            simple_ids,
            min_roots,
            min_coroots,
            negatives,
        })
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
/// likewise on coroot coordinates. Root membership binary-searches the
/// sorted `roots`; coroots are unsorted (they follow the root order), so a
/// coordinate map is built once.
fn build_ladder_bottoms(
    roots: &[Weight],
    coroots: &[Coweight],
) -> Result<(Vec<RootSet>, Vec<RootSet>), StructureError> {
    let count = roots.len();
    let mut coroot_ids: BTreeMap<&[i32], usize> = BTreeMap::new();
    for (index, coroot) in coroots.iter().enumerate() {
        coroot_ids.insert(coroot.as_slice(), index);
    }
    let mut min_roots = Vec::new();
    min_roots
        .try_reserve_exact(count)
        .map_err(|_| StructureError::AllocationFailed { requested: count })?;
    let mut min_coroots = Vec::new();
    min_coroots
        .try_reserve_exact(count)
        .map_err(|_| StructureError::AllocationFailed { requested: count })?;
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
            if !coroot_ids.contains_key(difference.as_slice()) {
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

/// One closure record: a root key with its paired coroot and simple-root
/// coordinates.
struct ClosureRecord {
    coroot: Vec<i32>,
    simple_coordinates: Vec<i32>,
}

struct PendingRecord {
    root: Weight,
    coroot: Coweight,
    simple_coordinates: Vec<i32>,
}

/// Private closure state shared by production enumeration and the invariant
/// tests, which inject candidate records directly.
struct Closure {
    max_roots: usize,
    seen: BTreeMap<Vec<i32>, ClosureRecord>,
    pending: VecDeque<PendingRecord>,
}

impl Closure {
    fn new(max_roots: usize) -> Self {
        Self {
            max_roots,
            seen: BTreeMap::new(),
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
        if pair_coordinates(root.as_slice(), coroot.as_slice())? != 2 {
            return Err(StructureError::RootSystemInvariantViolation {
                invariant: "self pairing",
            });
        }
        if let Some(existing) = self.seen.get(root.as_slice()) {
            if existing.coroot != coroot.as_slice()
                || existing.simple_coordinates != simple_coordinates
            {
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
        let key = try_copy_coordinates(root.as_slice())?;
        let record = ClosureRecord {
            coroot: try_copy_coordinates(coroot.as_slice())?,
            simple_coordinates: try_copy_coordinates(&simple_coordinates)?,
        };
        self.seen.insert(key, record);
        self.pending
            .try_reserve(1)
            .map_err(|_| StructureError::AllocationFailed { requested: 1 })?;
        self.pending.push_back(PendingRecord {
            root,
            coroot,
            simple_coordinates,
        });
        Ok(())
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
