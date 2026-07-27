use std::collections::{BTreeMap, VecDeque};

use crate::lattice::{pair_coordinates, try_copy_coordinates};
use crate::{pair, BasedRootDatum, Coweight, StructureError, Weight, WeylAction};

/// Stable identifier for one ordinary root in a deterministically ordered root
/// system. One ID indexes the root, its coroot, and its simple-root
/// coordinates simultaneously.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RootId(pub(crate) usize);

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
        Ok(Self {
            datum,
            roots,
            coroots,
            simple_coordinates,
            positive,
            simple_ids,
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
        self.roots
            .binary_search_by(|candidate| candidate.as_slice().cmp(root.as_slice()))
            .ok()
            .map(RootId)
    }
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
