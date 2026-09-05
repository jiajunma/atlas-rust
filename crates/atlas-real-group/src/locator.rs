//! Canonical integral-datum interning and the Weyl-attitude locator of a
//! rational infinitesimal character.
//!
//! This is step 1 of the "nonidentity generator attitude" slice: a pure,
//! unwired port of upstream `InnerClass::int_item`
//! (structure/innerclass.cpp:1116-1182) together with the
//! `subsystem::integral_datum_entry`/`integral_datum_item` pair
//! (structure/subsystem.h:155-201, subsystem.cpp:178-224).  Nothing here is
//! called by `RepTable::lookup` yet; the existing exact-embedded-list
//! interner there is untouched.
//!
//! Upstream stores the interner on the inner class (`int_hash`/`int_table`,
//! innerclass.h:238).  The crate keeps inner-class state as owned values
//! (`InnerClass` owns its `RootSystem`; `InvolutionTable` is owned and
//! `Arc`-shared by `RepTableOwner`), so the analogue is an owned
//! [`IntegralDatumTable`] value that the future wiring slice will place
//! next to those.  Every method re-takes the `RootSystem`, matching the
//! provenance contract style of [`WeylElement`].
//!
//! One deliberate deviation in storage, none in semantics: upstream's
//! `RootNbr` order among positive roots is height, then reverse
//! lexicographic simple coordinates (`partial_block.rs`'s
//! `upstream_positive_root_order`); the crate's `RootId` order is ambient
//! lexicographic.  All locator-facing lists (`simp_int`, item simple roots,
//! interning keys) are therefore sorted with the upstream order so that
//! `simple_pi` values are directly comparable with the oracle.

use std::collections::{BTreeSet, HashMap};

use crate::alcove::{checked_dot, root_vertex_of_alcove};
use crate::partial_block::upstream_positive_root_order;
use crate::root_system::combine_roots;
use crate::{
    BasedRootDatum, RationalWeight, RootId, RootSystem, StructureError, Weight, WeylElement,
};

/// The `repr::locator` of a query (gkmod/repr.h:484-491).
///
/// `int_sys` is the canonical integral datum's id in the owning
/// [`IntegralDatumTable`].  `w` maps the canonical fundamental-alcove
/// integral subsystem onto the query's actual one, preserving integral
/// positivity.  `simp_int` holds the images of the integrally-simple roots
/// in increasing parent root order, and `simple_pi` sends each canonical
/// subsystem simple-generator index to its position within `simp_int`
/// (innerclass.cpp:1178-1179).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlockLocator {
    int_sys: u32,
    w: WeylElement,
    simp_int: Vec<RootId>,
    simple_pi: Vec<usize>,
}

impl BlockLocator {
    /// Sequence number of the canonical integral datum (`int_sys_nr`).
    pub fn int_sys(&self) -> u32 {
        self.int_sys
    }

    /// The Weyl element carrying the canonical subsystem to the query's
    /// attitude (`locator::w`).
    pub fn w(&self) -> &WeylElement {
        &self.w
    }

    /// Images of the integrally-simple roots, in upstream positive-root
    /// order (`locator::simp_int`).
    pub fn simp_int(&self) -> &[RootId] {
        &self.simp_int
    }

    /// Canonical simple-generator index to position in `simp_int`
    /// (`locator::simple_pi`).
    pub fn simple_pi(&self) -> &[usize] {
        &self.simple_pi
    }

    /// Aggregate constructor, mirroring how upstream's `locator` struct is
    /// filled field-by-field by `block_modifier` construction
    /// (repr.cpp:1403-1419) and by `Reduced_param::reduce` writing through
    /// the `locator&` base subobject (repr.cpp:110-125).  Crate-private:
    /// outside the slice, locators come from [`IntegralDatumTable::int_item`].
    pub(crate) fn from_parts(
        int_sys: u32,
        w: WeylElement,
        simp_int: Vec<RootId>,
        simple_pi: Vec<usize>,
    ) -> Self {
        Self {
            int_sys,
            w,
            simp_int,
            simple_pi,
        }
    }

    /// The locator part of `Rep_context::make_relative_to`
    /// (repr.cpp:343-345): post-multiply the attitude by the inverse of the
    /// base locator's attitude and right-compose `simple_pi` with the
    /// INVERSE of the base's (`Permutation(loc.simple_pi,-1)`):
    /// `simple_pi[j] = old_simple_pi[inv[j]]` with `inv` the inverse
    /// permutation (permutations.cpp:56-63's `compose(a,b): a[j] =
    /// a_old[b[j]]`).
    ///
    /// Both locators must reference the same canonical integral datum; the
    /// reduced-parameter key match upstream guarantees this (the key carries
    /// `int_sys_nr`, repr.cpp:119-124).
    pub(crate) fn make_relative_to(
        &mut self,
        system: &RootSystem,
        base: &BlockLocator,
    ) -> Result<(), StructureError> {
        if self.int_sys != base.int_sys {
            return Err(StructureError::RepInvariantViolation {
                invariant: "block modifier attitude shares the canonical integral datum",
            });
        }
        self.w = self.w.multiply(system, &base.w.inverse())?;
        let inverse = inverse_permutation(base.simple_pi())?;
        if inverse.len() != self.simple_pi.len() {
            return Err(StructureError::RepInvariantViolation {
                invariant: "block modifier simple_pi rank",
            });
        }
        let old = std::mem::take(&mut self.simple_pi);
        for &preimage in &inverse {
            self.simple_pi.push(old[preimage]);
        }
        Ok(())
    }
}

/// `Permutation(pi, -1)` (permutations.cpp:35-38): the inverse permutation,
/// `result[pi[i]] = i`.
fn inverse_permutation(pi: &[usize]) -> Result<Vec<usize>, StructureError> {
    let mut inverse = Vec::new();
    inverse
        .try_reserve_exact(pi.len())
        .map_err(|_| StructureError::AllocationFailed {
            requested: pi.len(),
        })?;
    inverse.resize(pi.len(), usize::MAX);
    for (index, &image) in pi.iter().enumerate() {
        let slot = inverse
            .get_mut(image)
            .ok_or(StructureError::RepInvariantViolation {
                invariant: "simple_pi is a permutation",
            })?;
        if *slot != usize::MAX {
            return Err(StructureError::RepInvariantViolation {
                invariant: "simple_pi is a permutation",
            });
        }
        *slot = index;
    }
    Ok(inverse)
}

/// One interned canonical integral datum (`subsystem::integral_datum_item`,
/// subsystem.h:176-201): the subsystem of the ambient root system cut out
/// by a Weyl-orbit representative in the fundamental alcove.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegralDatumItem {
    /// Canonical positive roots, sorted in the upstream positive-root
    /// order; this is the interning key (`integral_datum_entry::posroots`).
    posroots: Vec<RootId>,
    /// Simple roots of the canonical subsystem as parent root ids, in
    /// upstream positive-root order (`SubSystem::simple_roots`).
    simple_roots: Vec<RootId>,
    /// Ambient coroot coordinates per simple root
    /// (`integral_datum_item::simple_coroots`, subsystem.cpp:182-188).
    simple_coroots: Vec<Vec<i32>>,
}

impl IntegralDatumItem {
    /// `integral_datum_item::integral_datum_item` (subsystem.cpp:178-189):
    /// build the canonical `SubSystem` from its positive roots and cache
    /// the simple-coroot matrix.
    fn new(system: &RootSystem, posroots: Vec<RootId>) -> Result<Self, StructureError> {
        let simple_roots = pos_simples(system, &posroots)?;
        let mut simple_coroots = Vec::new();
        simple_coroots
            .try_reserve_exact(simple_roots.len())
            .map_err(|_| StructureError::AllocationFailed {
                requested: simple_roots.len(),
            })?;
        for &alpha in &simple_roots {
            let coroot = system
                .coroot(alpha)
                .ok_or(StructureError::IndexOutOfRange {
                    index: alpha.index(),
                    upper_bound: system.roots().len(),
                })?;
            simple_coroots.push(coroot.as_slice().to_vec());
        }
        Ok(Self {
            posroots,
            simple_roots,
            simple_coroots,
        })
    }

    /// The canonical positive roots (the interning key).
    pub fn positive_roots(&self) -> &[RootId] {
        &self.posroots
    }

    /// Subsystem simple roots as parent root ids, in subsystem generator
    /// order (`SubSystem::parent_nr_simple`).
    pub fn simple_roots(&self) -> &[RootId] {
        &self.simple_roots
    }

    /// Ambient coroot coordinates per subsystem simple root.
    pub fn simple_coroots(&self) -> &[Vec<i32>] {
        &self.simple_coroots
    }

    /// `integral_datum_item::image_simples` (subsystem.cpp:192-206): images
    /// of the canonical simple roots under `w`, sorted into the upstream
    /// positive-root order.  `w` must map to integrally-dominant attitude;
    /// a non-positive image is the checked form of upstream's
    /// `assert(rd.is_posroot(image))`.
    pub fn image_simples(
        &self,
        system: &RootSystem,
        w: &WeylElement,
    ) -> Result<Vec<RootId>, StructureError> {
        let mut images = Vec::new();
        images
            .try_reserve_exact(self.simple_roots.len())
            .map_err(|_| StructureError::AllocationFailed {
                requested: self.simple_roots.len(),
            })?;
        for &alpha in &self.simple_roots {
            let image = w
                .image(alpha)
                .ok_or(StructureError::WeylElementInvariantViolation {
                    invariant: "provenance",
                })?;
            if system.is_positive(image) != Some(true) {
                return Err(StructureError::WeylElementInvariantViolation {
                    invariant: "integral image positivity",
                });
            }
            images.push(image);
        }
        images.sort_by(|&a, &b| upstream_positive_root_order(system, a, b));
        Ok(images)
    }

    /// `integral_datum_item::coroots_matrix` (subsystem.cpp:208-218): the
    /// ambient coroot coordinates of `image_simples(w)`, one row per root.
    pub fn coroots_matrix(
        &self,
        system: &RootSystem,
        w: &WeylElement,
    ) -> Result<Vec<Vec<i32>>, StructureError> {
        let mut rows = Vec::new();
        let simples = self.image_simples(system, w)?;
        rows.try_reserve_exact(simples.len())
            .map_err(|_| StructureError::AllocationFailed {
                requested: simples.len(),
            })?;
        for alpha in simples {
            let coroot = system
                .coroot(alpha)
                .ok_or(StructureError::IndexOutOfRange {
                    index: alpha.index(),
                    upper_bound: system.roots().len(),
                })?;
            rows.push(coroot.as_slice().to_vec());
        }
        Ok(rows)
    }
}

/// The canonical integral-datum interner: upstream's `int_hash`/`int_table`
/// pair (innerclass.h:238, innerclass.cpp:1150-1157), owned per inner-class
/// context.  Ids are append-only sequence numbers, matching upstream's
/// `int_sys_nr`.
#[derive(Clone, Debug, Default)]
pub struct IntegralDatumTable {
    items: Vec<IntegralDatumItem>,
    ids: HashMap<Vec<RootId>, u32>,
}

impl IntegralDatumTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// `InnerClass::int_item(int_sys_nr)` (innerclass.h inline): the
    /// interned item behind a canonical datum id.
    pub fn item(&self, id: u32) -> Result<&IntegralDatumItem, StructureError> {
        self.items
            .get(id as usize)
            .ok_or(StructureError::IndexOutOfRange {
                index: id as usize,
                upper_bound: self.items.len(),
            })
    }

    /// `InnerClass::int_item(gamma, loc)` (innerclass.cpp:1116-1182):
    /// canonicalize `gamma`'s integral root system across the Weyl group
    /// and return its interned id together with the attitude locator.
    pub fn int_item(
        &mut self,
        system: &RootSystem,
        gamma: &RationalWeight,
    ) -> Result<(u32, BlockLocator), StructureError> {
        if gamma.rank() != system.lattice_rank() {
            return Err(StructureError::RankMismatch {
                expected: system.lattice_rank(),
                actual: gamma.rank(),
            });
        }
        let datum = system.datum();
        let denominator = gamma.denominator();

        // (a) innerclass.cpp:1123: pull gamma into the Weyl orbit of the
        // fundamental alcove by subtracting its alcove's root-lattice
        // vertex (alcoves.cpp:414-428).
        let vertex = root_vertex_of_alcove(system, gamma)?;
        let mut numerator = Vec::new();
        numerator.try_reserve_exact(gamma.rank()).map_err(|_| {
            StructureError::AllocationFailed {
                requested: gamma.rank(),
            }
        })?;
        for (&entry, &vertex_coordinate) in gamma.numerator().iter().zip(vertex.as_slice()) {
            let shift = denominator
                .checked_mul(i64::from(vertex_coordinate))
                .ok_or(StructureError::ArithmeticOverflow)?;
            numerator.push(
                entry
                    .checked_sub(shift)
                    .ok_or(StructureError::ArithmeticOverflow)?,
            );
        }

        // (b) innerclass.cpp:1125: make the numerator dominant; the word
        // transforms the dominant value back to the original when applied
        // right-to-left (rootdata.cpp:1117-1135).
        let word = factor_dominant(datum, &mut numerator)?;

        // (c) innerclass.cpp:1129-1134: the fundamental-alcove walls on
        // which the now-dominant gamma lies: simple walls evaluate to 0,
        // the per-component negative highest-coroot wall to `-denominator`.
        let mut on_wall = BTreeSet::new();
        for alpha in fundamental_alcove_walls(system)? {
            let coroot = system
                .coroot(alpha)
                .ok_or(StructureError::IndexOutOfRange {
                    index: alpha.index(),
                    upper_bound: system.roots().len(),
                })?;
            let evaluation = checked_dot(&numerator, coroot.as_slice())?;
            let negative = system.is_positive(alpha) == Some(false);
            if evaluation == if negative { -denominator } else { 0 } {
                on_wall.insert(alpha);
            }
        }

        // (d) innerclass.cpp:1137-1145: traverse the dominance word
        // right-to-left; each letter whose current evaluation is
        // non-integral is left-multiplied into `w` and reflected out of
        // gamma, so `w(dominant gamma)` is the final attitude.
        let mut w = WeylElement::identity(system)?;
        for &s in word.iter().rev() {
            let dot = checked_dot(&numerator, datum.simple_coroots()[s].as_slice())?;
            if dot.rem_euclid(denominator) != 0 {
                let (product, _) = w.left_multiply_simple(system, s)?;
                w = product;
                reflect_numerator(datum, s, &mut numerator, dot)?;
            }
        }

        // (e) innerclass.cpp:1147-1157: the canonical key is the positive
        // part of the additive closure of the on-wall roots
        // (`additive_closure(rd, on_wall_subset) & rd.posroot_set()`).
        let closure = additive_closure(system, &on_wall)?;
        let mut posroots: Vec<RootId> = closure
            .into_iter()
            .filter(|&id| system.is_positive(id) == Some(true))
            .collect();
        posroots.sort_by(|&a, &b| upstream_positive_root_order(system, a, b));
        let int_sys = self.intern(system, posroots)?;

        // (f) innerclass.cpp:1161-1179: images of the canonical simple
        // roots under `w` (upstream applies `word(loc.w)` via
        // `permuted_root`; `WeylElement::image` is the same left action),
        // then `simp_int` sorted and `simple_pi` as positions.
        let item = self.item(int_sys)?;
        let mut images = Vec::new();
        images
            .try_reserve_exact(item.simple_roots.len())
            .map_err(|_| StructureError::AllocationFailed {
                requested: item.simple_roots.len(),
            })?;
        for &alpha in &item.simple_roots {
            let image = w
                .image(alpha)
                .ok_or(StructureError::WeylElementInvariantViolation {
                    invariant: "provenance",
                })?;
            // upstream `assert(rd.is_posroot(beta))`: `w` preserves
            // integral positivity.
            if system.is_positive(image) != Some(true) {
                return Err(StructureError::WeylElementInvariantViolation {
                    invariant: "integral image positivity",
                });
            }
            images.push(image);
        }
        let mut simp_int = images.clone();
        simp_int.sort_by(|&a, &b| upstream_positive_root_order(system, a, b));
        debug_assert_eq!(
            simp_int.iter().collect::<BTreeSet<_>>().len(),
            images.len(),
            "Weyl images of distinct roots are distinct"
        );
        let mut simple_pi = Vec::new();
        simple_pi.try_reserve_exact(images.len()).map_err(|_| {
            StructureError::AllocationFailed {
                requested: images.len(),
            }
        })?;
        for image in &images {
            let position = simp_int.iter().position(|entry| entry == image).ok_or(
                StructureError::RootSystemInvariantViolation {
                    invariant: "locator image membership",
                },
            )?;
            simple_pi.push(position);
        }
        Ok((
            int_sys,
            BlockLocator {
                int_sys,
                w,
                simp_int,
                simple_pi,
            },
        ))
    }

    /// `int_hash.match(e)` plus the `int_table.emplace_back` growth
    /// (innerclass.cpp:1150-1157).
    fn intern(
        &mut self,
        system: &RootSystem,
        posroots: Vec<RootId>,
    ) -> Result<u32, StructureError> {
        if let Some(&id) = self.ids.get(&posroots) {
            return Ok(id);
        }
        let item = IntegralDatumItem::new(system, posroots.clone())?;
        let id = u32::try_from(self.items.len()).map_err(|_| StructureError::ArithmeticOverflow)?;
        self.items
            .try_reserve_exact(1)
            .map_err(|_| StructureError::AllocationFailed { requested: 1 })?;
        self.items.push(item);
        self.ids.insert(posroots, id);
        Ok(id)
    }
}

/// `RootSystem::fundamental_alcove_walls` (rootdata.cpp:474-481): the
/// simple roots together with, per Dynkin component, the negative root
/// whose coroot is the component's lowest coroot — computed upstream as
/// the negative roots lying in `min_coroots_for` of every simple root.
fn fundamental_alcove_walls(system: &RootSystem) -> Result<Vec<RootId>, StructureError> {
    let mut lowest: Option<BTreeSet<RootId>> = None;
    for &simple in system.simple_root_ids() {
        let bottoms =
            system
                .min_coroots_for(simple)
                .ok_or(StructureError::RootSystemInvariantViolation {
                    invariant: "simple root has no coroot ladder table",
                })?;
        let set: BTreeSet<RootId> = bottoms.iter().collect();
        lowest = Some(match lowest {
            None => set,
            Some(current) => current.intersection(&set).copied().collect(),
        });
    }
    let mut walls = Vec::new();
    if let Some(lowest) = lowest {
        walls.extend(
            lowest
                .into_iter()
                .filter(|&id| system.is_positive(id) == Some(false)),
        );
    }
    walls.extend(system.simple_root_ids().iter().copied());
    Ok(walls)
}

/// `RootDatum::factor_dominant` (rootdata.cpp:1117-1135): greedily reflect
/// `v` by the lowest-index simple coroot on which it evaluates negatively,
/// until dominant.  The returned word is in application order; applied
/// right-to-left it transforms the dominant value back to the original.
fn factor_dominant(datum: &BasedRootDatum, v: &mut [i64]) -> Result<Vec<usize>, StructureError> {
    let mut word = Vec::new();
    loop {
        let mut reflected = false;
        for s in 0..datum.semisimple_rank() {
            let dot = checked_dot(v, datum.simple_coroots()[s].as_slice())?;
            if dot < 0 {
                word.push(s);
                reflect_numerator(datum, s, v, dot)?;
                reflected = true;
                break;
            }
        }
        if !reflected {
            return Ok(word);
        }
    }
}

/// `rd.simple_reflect(s, v)` on a rational-weight numerator: the integer
/// operation `v -= <v, alpha_s^vee> alpha_s`, with `dot` the precomputed
/// pairing.
fn reflect_numerator(
    datum: &BasedRootDatum,
    s: usize,
    v: &mut [i64],
    dot: i64,
) -> Result<(), StructureError> {
    let root = datum.simple_roots()[s].as_slice();
    for (entry, &coordinate) in v.iter_mut().zip(root) {
        let term = dot
            .checked_mul(i64::from(coordinate))
            .ok_or(StructureError::ArithmeticOverflow)?;
        *entry = entry
            .checked_sub(term)
            .ok_or(StructureError::ArithmeticOverflow)?;
    }
    Ok(())
}

/// `additive_closure<false>` (rootdata.cpp:685-707): close `generators`
/// under negation and root sums. Returns the members sorted by root id
/// (the upstream `RootList` set order).
fn additive_closure(
    system: &RootSystem,
    generators: &BTreeSet<RootId>,
) -> Result<Vec<RootId>, StructureError> {
    let num_roots = system.roots().len();
    // Membership is a dense bitset over root ids: the closure loop below
    // test-and-sets it O(members^2) times per `int_item` call, which made
    // BTreeSet::insert the top heavy-unitary leaf (perf-unitary-3681134).
    let mut bits = vec![0_u64; (num_roots + 63) / 64];
    let mut members: Vec<RootId> = Vec::new();
    members
        .try_reserve_exact(2 * generators.len())
        .map_err(|_| StructureError::AllocationFailed {
            requested: 2 * generators.len(),
        })?;
    for &id in generators {
        let negative = negate_root(system, id)?;
        for root in [id, negative] {
            let word = root.index() / 64;
            let mask = 1_u64 << (root.index() % 64);
            if bits[word] & mask == 0 {
                bits[word] |= mask;
                members.push(root);
            }
        }
    }
    // Work-queue fixpoint over insertion order, replacing full O(members^2)
    // rescan rounds. Processing `members[next]` tests its sums against
    // `members[0..next]` only: root addition is commutative and the pair
    // {i, j} with i < j is tested when j is processed, so every unordered
    // pair of distinct members is covered exactly once and the result
    // equals the rescan fixpoint.
    let mut next = 0;
    while next < members.len() {
        let left = members[next];
        for other in 0..next {
            if let Some(sum) = combine_roots(system, left, members[other], false)? {
                let word = sum.index() / 64;
                let mask = 1_u64 << (sum.index() % 64);
                if bits[word] & mask == 0 {
                    bits[word] |= mask;
                    members
                        .try_reserve(1)
                        .map_err(|_| StructureError::AllocationFailed { requested: 1 })?;
                    members.push(sum);
                }
            }
        }
        next += 1;
    }
    let mut closure = Vec::with_capacity(members.len());
    for (word, &bits_word) in bits.iter().enumerate() {
        let mut remaining = bits_word;
        while remaining != 0 {
            let bit = remaining.trailing_zeros() as usize;
            remaining &= remaining - 1;
            closure.push(RootId::from_usize(word * 64 + bit));
        }
    }
    Ok(closure)
}

/// `RootSystem::pos_simples` (rootdata.cpp:655-681): the simple roots of
/// the positive subsystem `posroots`, which must be sorted in the upstream
/// positive-root order; the result keeps that order.
///
/// Upstream reads `gamma<beta` (reflection lowers the root number) as
/// "positive dot product"; positive root numbers increase with height and
/// `s_alpha(beta)` has strictly smaller height than `beta` exactly when
/// `<beta, alpha^vee> > 0`, so the pairing sign is the faithful test (the
/// same argument as `simpleBasis`, partial_block.rs).
fn pos_simples(system: &RootSystem, posroots: &[RootId]) -> Result<Vec<RootId>, StructureError> {
    let mut removed = vec![false; posroots.len()];
    let mut result = Vec::new();
    'outer: for (index, &alpha) in posroots.iter().enumerate() {
        if removed[index] {
            continue; // pruned as a |beta| of an earlier iteration
        }
        for later in (index + 1)..posroots.len() {
            if removed[later] {
                continue;
            }
            let beta = posroots[later];
            let pairing = system.bracket(beta, alpha)?;
            if pairing > 0 {
                let image = reflect_root(system, alpha, beta, pairing)?;
                if system.is_positive(image) == Some(true) {
                    removed[later] = true; // beta cannot be simple
                } else {
                    // s_alpha made a later positive root negative, so
                    // alpha is not simple.
                    continue 'outer;
                }
            }
        }
        result.push(alpha);
    }
    Ok(result)
}

/// The root id of `-root(id)`, read from the negation table built at
/// enumeration (which already proved every root's negative is a root).
fn negate_root(system: &RootSystem, id: RootId) -> Result<RootId, StructureError> {
    if system.root(id).is_none() {
        return Err(StructureError::IndexOutOfRange {
            index: id.index(),
            upper_bound: system.roots().len(),
        });
    }
    system
        .negatives()
        .get(id.index())
        .copied()
        .ok_or(StructureError::RootSystemInvariantViolation {
            invariant: "root negation",
        })
}

/// `s_alpha(beta)` as a root id, with `pairing = <beta, alpha^vee>`
/// precomputed.
fn reflect_root(
    system: &RootSystem,
    alpha: RootId,
    beta: RootId,
    pairing: i32,
) -> Result<RootId, StructureError> {
    let alpha_root = system.root(alpha).ok_or(StructureError::IndexOutOfRange {
        index: alpha.index(),
        upper_bound: system.roots().len(),
    })?;
    let beta_root = system.root(beta).ok_or(StructureError::IndexOutOfRange {
        index: beta.index(),
        upper_bound: system.roots().len(),
    })?;
    let mut coordinates = Vec::new();
    coordinates
        .try_reserve_exact(beta_root.as_slice().len())
        .map_err(|_| StructureError::AllocationFailed {
            requested: beta_root.as_slice().len(),
        })?;
    for (&b, &a) in beta_root.as_slice().iter().zip(alpha_root.as_slice()) {
        let term = pairing
            .checked_mul(a)
            .ok_or(StructureError::ArithmeticOverflow)?;
        coordinates.push(
            b.checked_sub(term)
                .ok_or(StructureError::ArithmeticOverflow)?,
        );
    }
    system
        .id_of(&Weight::new(coordinates))
        .ok_or(StructureError::RootSystemInvariantViolation {
            invariant: "reflected root is a root",
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a2() -> RootSystem {
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        RootSystem::enumerate(&datum, 6).unwrap()
    }

    fn b2() -> RootSystem {
        let datum = BasedRootDatum::standard(vec![vec![2, -2], vec![-1, 2]]).unwrap();
        RootSystem::enumerate(&datum, 8).unwrap()
    }

    fn g2() -> RootSystem {
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-3, 2]]).unwrap();
        RootSystem::enumerate(&datum, 12).unwrap()
    }

    /// The pre-work-queue `additive_closure`: full O(members^2) rescan
    /// rounds to a fixpoint. Kept here as the reference for the optimized
    /// implementation's set equality.
    fn reference_additive_closure(
        system: &RootSystem,
        generators: &BTreeSet<RootId>,
    ) -> Result<BTreeSet<RootId>, StructureError> {
        let mut closure = generators.clone();
        for &id in generators {
            closure.insert(negate_root(system, id)?);
        }
        loop {
            let members: Vec<RootId> = closure.iter().copied().collect();
            let mut grew = false;
            for left in 0..members.len() {
                for right in (left + 1)..members.len() {
                    if let Some(sum) = combine_roots(system, members[left], members[right], false)?
                    {
                        grew |= closure.insert(sum);
                    }
                }
            }
            if !grew {
                return Ok(closure);
            }
        }
    }

    #[test]
    fn additive_closure_matches_rescan_reference() {
        for system in [a2(), b2(), g2()] {
            let all: Vec<RootId> = system.entries().map(|(id, _, _)| id).collect();
            let subsets: Vec<BTreeSet<RootId>> = vec![
                BTreeSet::new(),
                BTreeSet::from([all[0]]),
                BTreeSet::from([all[all.len() / 2]]),
                BTreeSet::from([all[1], all[all.len() - 1]]),
                all[..all.len() / 2].iter().copied().collect(),
                all.iter().copied().collect(),
            ];
            for generators in subsets {
                let reference: Vec<RootId> = reference_additive_closure(&system, &generators)
                    .unwrap()
                    .into_iter()
                    .collect();
                assert_eq!(
                    additive_closure(&system, &generators).unwrap(),
                    reference,
                    "generators={generators:?}"
                );
            }
        }
    }

    fn gamma(numerator: &[i64], denominator: i64) -> RationalWeight {
        RationalWeight::new(numerator.to_vec(), denominator).unwrap()
    }

    fn id(index: usize) -> RootId {
        RootId::from_usize(index)
    }

    // Conventions for the hand computations below.
    //
    // The design brief quotes gammas in the fundamental-weight basis (the
    // Atlas `vec` convention for `simply_connected(Lie_type("A2"),true)`);
    // the crate's standard datum uses ambient coordinates with simple roots
    // e_i and simple coroots the Cartan columns, so omega_i is row i of the
    // inverse Cartan matrix.  For A2, C^{-1} = (1/3)[[2,1],[1,2]].
    //
    // A2 RootIds (ambient lexicographic): [-1,-1]=0, [-1,0]=1, [0,-1]=2,
    // [0,1]=3 (alpha2), [1,0]=4 (alpha1), [1,1]=5 (theta).  The upstream
    // positive-root order is alpha1, alpha2, theta, i.e. ids 4, 3, 5.
    // `int_item` depends only on the root datum and Weyl group, so a plain
    // RootSystem stands in for the compact inner class of the brief.

    #[test]
    fn a2_first_slice_gamma_interns_theta_wall_a1() {
        let system = a2();
        let mut table = IntegralDatumTable::new();

        // Brief's gamma [2,5]/2 fundamental = [3,4]/2 ambient.  Derivation
        // of the canonical item (innerclass.cpp:1116-1182):
        // (a) wall set of gamma is the fundamental one {-theta, alpha1,
        //     alpha2} with evaluation floors (-4, 1, 2); the coroot relation
        //     is (1,1,1), the chosen coefficient-1 wall is -theta, and the
        //     transposed sub-Cartan inverse is [[2,1],[1,2]]/3.  The
        //     unshifted vertex numerator is [4,5]/3 (not integral); the
        //     first label-1 shift gives [6,6]/3 = [2,2], so the vertex is
        //     2*alpha1 + 2*alpha2 = [2,2] and gamma' = [-1,0]/2.
        // (b) factor_dominant([-1,0]) = word [0,1], dominant numerator
        //     [1,1], so the dominant representative is [1,1]/2.
        // (c) evaluations of [1,1]/2: alpha1^vee and alpha2^vee give 1,
        //     (-theta)^vee gives -2 == -denominator: on_wall = {-theta}.
        // (e) additive closure {theta, -theta}; canonical key [theta].
        // (d) filter of word [0,1] read right-to-left: s=1 has evaluation
        //     1/2 (non-integral) so w = s1 and the numerator becomes [1,0];
        //     s=0 then evaluates to 2 == 0 mod 2 and is skipped.  w = s1.
        // (f) s1(theta) = alpha1, so simp_int = [alpha1], simple_pi = [0].
        //
        // Note: gamma's own integral roots are {+-alpha1}
        // (<gamma,alpha1^vee> = 1, alpha2^vee gives 5/2), but the canonical
        // fundamental-alcove subsystem is the theta-wall one {+-theta}; the
        // brief's "same item for both A2 gammas" sketch is not what the
        // upstream algorithm computes (see the next test).
        let (item_id, loc) = table.int_item(&system, &gamma(&[3, 4], 2)).unwrap();
        assert_eq!(item_id, 0);
        assert_eq!(loc.int_sys(), 0);
        assert_eq!(loc.w().reduced_word(&system).unwrap(), vec![1]);
        assert_eq!(loc.simp_int(), &[id(4)]);
        assert_eq!(loc.simple_pi(), &[0]);
        let item = table.item(0).unwrap();
        assert_eq!(item.positive_roots(), &[id(5)]);
        assert_eq!(item.simple_roots(), &[id(5)]);
        assert_eq!(item.simple_coroots(), &[vec![1, 1]]);
        // subsystem.cpp helpers: s1(theta) = alpha1 with coroot [2,-1].
        assert_eq!(item.image_simples(&system, loc.w()).unwrap(), vec![id(4)]);
        assert_eq!(
            item.coroots_matrix(&system, loc.w()).unwrap(),
            vec![vec![2, -1]]
        );
    }

    #[test]
    fn a2_second_slice_gamma_interns_a_different_a1() {
        let system = a2();
        let mut table = IntegralDatumTable::new();
        let (first, _) = table.int_item(&system, &gamma(&[3, 4], 2)).unwrap();

        // Brief's gamma [4,3]/4 fundamental = [11,10]/12 ambient.
        // Derivation:
        // (a) same wall set {-theta, alpha1, alpha2} with floors (-2, 1, 0);
        //     base vertex numerator [2,1]/3 not integral, first label-1
        //     shift [4,2]/3 not integral, second [3,3]/3 = [1,1] integral:
        //     vertex = alpha1 + alpha2 = [1,1], gamma' = [-1,-2]/12.
        // (b) factor_dominant([-1,-2]) = word [1,0], dominant [2,1]/12.
        // (c) evaluations of [2,1]/12: alpha2^vee gives 0 (on wall),
        //     alpha1^vee gives 3, (-theta)^vee gives -3 != -12:
        //     on_wall = {alpha2}.
        // (e) closure {+-alpha2}; canonical key [alpha2] -- a DIFFERENT item
        //     from the theta-wall A1 above, although the query's integral
        //     roots are {+-alpha1} in both cases: the canonical datum
        //     depends on gamma's alcove, not only on its integral system.
        // (d) filter of [1,0] read right-to-left: s=0 evaluates to 3/12
        //     (non-integral), w = s0, numerator [-1,1]; s=1 evaluates to
        //     3/12, w = s1*s0, numerator back to [-1,-2].
        // (f) w(alpha2) = s1(s0(alpha2)) = s1(theta) = alpha1.
        let (item_id, loc) = table.int_item(&system, &gamma(&[11, 10], 12)).unwrap();
        assert_eq!(item_id, 1);
        assert_ne!(item_id, first);
        assert_eq!(loc.w().reduced_word(&system).unwrap(), vec![1, 0]);
        assert_eq!(loc.simp_int(), &[id(4)]);
        assert_eq!(loc.simple_pi(), &[0]);
        let item = table.item(1).unwrap();
        assert_eq!(item.positive_roots(), &[id(3)]);
        assert_eq!(item.simple_roots(), &[id(3)]);
        assert_eq!(item.simple_coroots(), &[vec![-1, 2]]);
    }

    #[test]
    fn a2_weyl_conjugate_gammas_share_the_canonical_item() {
        let system = a2();
        let mut table = IntegralDatumTable::new();
        let (first, _) = table.int_item(&system, &gamma(&[3, 4], 2)).unwrap();

        // s1([3,4]/2) = [3,-1]/2 ambient.  The Weyl orbit of the alcove is
        // unchanged, so the dominant representative [1,1]/2 and the on-wall
        // set {-theta} agree with the unconjugated query: same item.
        // Derivation of the locator: vertex [2,0], gamma' = [-1,-1]/2,
        // factor_dominant word [0,1,0]; filtering right-to-left: s=0
        // evaluates to 1/2 -> w = s0, numerator [0,1]; s=1 evaluates to 1
        // (integral, skipped); s=0 evaluates to -1/2 -> w = s0*s0 = id.
        // The reduced attitude is the dominant [1,1]/2 itself, whose
        // integrally-simple root is theta.
        let (item_id, loc) = table.int_item(&system, &gamma(&[3, -1], 2)).unwrap();
        assert_eq!(item_id, first);
        assert!(loc.w().is_identity());
        assert_eq!(loc.simp_int(), &[id(5)]);
        assert_eq!(loc.simple_pi(), &[0]);
        // Interning is idempotent: the conjugate query adds no new item.
        assert_eq!(table.len(), 1);
    }

    #[test]
    fn integral_gamma_interns_the_full_system_with_identity_attitude() {
        let system = a2();
        let mut table = IntegralDatumTable::new();

        // gamma = 0 (root lattice).  Its wall set is the fundamental alcove
        // walls {-theta, alpha1, alpha2} with all floors 0, so the vertex is
        // 0; it is already dominant, and the on-wall set is {alpha1, alpha2}
        // (the -theta wall evaluates to 0, not -1).  The additive closure is
        // the full root system, so the canonical item is the full A2 system
        // with simple roots [alpha1, alpha2] in upstream order, w = id, and
        // simp_int = [alpha1, alpha2] with simple_pi identity.
        let (item_id, loc) = table.int_item(&system, &gamma(&[0, 0], 1)).unwrap();
        assert_eq!(item_id, 0);
        assert!(loc.w().is_identity());
        assert_eq!(loc.simp_int(), &[id(4), id(3)]);
        assert_eq!(loc.simple_pi(), &[0, 1]);
        let item = table.item(0).unwrap();
        assert_eq!(item.positive_roots(), &[id(4), id(3), id(5)]);
        assert_eq!(item.simple_roots(), &[id(4), id(3)]);
        assert_eq!(item.simple_coroots(), &[vec![2, -1], vec![-1, 2]]);
    }

    #[test]
    fn b2_long_and_short_a1_intern_different_items() {
        let system = b2();
        let mut table = IntegralDatumTable::new();

        // B2 with Cartan [[2,-2],[-1,2]]: alpha1 = [1,0] (long, RootId 5),
        // alpha2 = [0,1] (short, RootId 4); the remaining positive roots are
        // [1,1] (id 6, coroot [2,0]) and [1,2] (id 7, coroot [0,1]).  The
        // fundamental alcove walls are {alpha1, alpha2, -[1,1]}.  In
        // fundamental-weight coordinates (p,q) = (<gamma,alpha1^vee>,
        // <gamma,alpha2^vee>) the four positive-coroot evaluations are
        // p, q, 2p+q, p+q, and the inverse Cartan is (1/2)[[2,2],[1,2]].

        // Long A1: (p,q) = (0,1/2), i.e. omega2/2 = [1,2]/4 ambient.
        // Derivation: the wall set is the fundamental one with floors
        // (-1,0,0); the component coroot relation is (1,2,1), the chosen
        // wall is -[1,1], and the transposed generator Cartan inverse is
        // [[2,1],[2,2]]/2, giving the integral vertex 0.  The numerator is
        // dominant (evaluations 0 and 2), so the dominance word is empty.
        // On-wall: alpha1^vee evaluates to 0; alpha2^vee to 2; the -[1,1]
        // wall to -2 != -4.  Closure {+-alpha1}: the long A1 item.
        let (long_id, long_loc) = table.int_item(&system, &gamma(&[1, 2], 4)).unwrap();
        assert_eq!(long_id, 0);
        assert!(long_loc.w().is_identity());
        assert_eq!(long_loc.simp_int(), &[id(5)]);
        assert_eq!(long_loc.simple_pi(), &[0]);
        let long_item = table.item(long_id).unwrap();
        assert_eq!(long_item.positive_roots(), &[id(5)]);
        assert_eq!(long_item.simple_roots(), &[id(5)]);
        assert_eq!(long_item.simple_coroots(), &[vec![2, -1]]);

        // Short A1: (p,q) = (1/3,0), i.e. omega1/3 = [1,1]/3 ambient.  Same
        // walls with floors (-1,0,0) and vertex 0; dominant already
        // (evaluations 1 and 0).  On-wall: only alpha2.  Closure
        // {+-alpha2}: the short A1 item, which must NOT merge with the
        // long one.
        let (short_id, short_loc) = table.int_item(&system, &gamma(&[1, 1], 3)).unwrap();
        assert_eq!(short_id, 1);
        assert_ne!(short_id, long_id);
        assert!(short_loc.w().is_identity());
        assert_eq!(short_loc.simp_int(), &[id(4)]);
        assert_eq!(short_loc.simple_pi(), &[0]);
        let short_item = table.item(short_id).unwrap();
        assert_eq!(short_item.positive_roots(), &[id(4)]);
        assert_eq!(short_item.simple_roots(), &[id(4)]);
        assert_eq!(short_item.simple_coroots(), &[vec![-2, 2]]);

        // Re-querying the long representative reuses the first item.
        let (again, _) = table.int_item(&system, &gamma(&[1, 2], 4)).unwrap();
        assert_eq!(again, long_id);
        assert_eq!(table.len(), 2);
    }

    #[test]
    fn root_vertex_of_alcove_matches_the_hand_computed_vertices() {
        let system = a2();
        // Vertices derived in the tests above; gamma = 0 lies at the
        // fundamental-alcove vertex itself.
        assert_eq!(
            crate::alcove::root_vertex_of_alcove(&system, &gamma(&[3, 4], 2)).unwrap(),
            Weight::new(vec![2, 2])
        );
        assert_eq!(
            crate::alcove::root_vertex_of_alcove(&system, &gamma(&[11, 10], 12)).unwrap(),
            Weight::new(vec![1, 1])
        );
        assert_eq!(
            crate::alcove::root_vertex_of_alcove(&system, &gamma(&[0, 0], 1)).unwrap(),
            Weight::new(vec![0, 0])
        );
    }

    #[test]
    fn make_relative_to_composes_simple_pi_with_the_inverse_permutation() {
        let system = a2();
        let identity = WeylElement::identity(&system).unwrap();

        // Upstream repr.cpp:345: compose(bm.simple_pi, Permutation(
        // loc.simple_pi,-1)) with compose(a,b): a[j] = a_old[b[j]]
        // (permutations.cpp:56-63), i.e. the composed permutation is
        // bm.pi o loc.pi^{-1} when permutations are read as maps
        // i -> pi[i].
        //
        // Rank-3 hand check: bm.pi = [2,1,0], loc.pi = [1,2,0].
        // loc.pi as a map sends 0->1, 1->2, 2->0, so its inverse is
        // inv = [2,0,1] (inv[1]=0, inv[2]=1, inv[0]=2).  Then
        // composed[j] = bm.pi[inv[j]] = [bm.pi[2], bm.pi[0], bm.pi[1]]
        // = [0,2,1].  Cross-check as a composite map:
        // (bm o loc^{-1})(0) = bm(2) = 0, (1) = bm(0) = 2,
        // (2) = bm(1) = 1.
        let mut bm = BlockLocator::from_parts(7, identity.clone(), vec![], vec![2, 1, 0]);
        let loc = BlockLocator::from_parts(7, identity.clone(), vec![], vec![1, 2, 0]);
        bm.make_relative_to(&system, &loc).unwrap();
        assert_eq!(bm.simple_pi(), &[0, 2, 1]);

        // An identity base permutation leaves bm's permutation unchanged.
        let mut bm = BlockLocator::from_parts(7, identity.clone(), vec![], vec![2, 1, 0]);
        let loc = BlockLocator::from_parts(7, identity.clone(), vec![], vec![0, 1, 2]);
        bm.make_relative_to(&system, &loc).unwrap();
        assert_eq!(bm.simple_pi(), &[2, 1, 0]);

        // The w part is right-multiplication by the inverse attitude
        // (repr.cpp:343: W.mult(bm.w, W.inverse(loc.w))): s0 * s1^{-1} =
        // s0*s1, whose canonical lowest-left-descent word is [0,1].
        let s0 = WeylElement::simple_reflection(&system, 0).unwrap();
        let s1 = WeylElement::simple_reflection(&system, 1).unwrap();
        let mut bm = BlockLocator::from_parts(7, s0, vec![], vec![0]);
        let loc = BlockLocator::from_parts(7, s1, vec![], vec![0]);
        bm.make_relative_to(&system, &loc).unwrap();
        assert_eq!(bm.w().reduced_word(&system).unwrap(), vec![0, 1]);

        // Different canonical data are rejected: the reduced-parameter
        // key match upstream guarantees equality (repr.cpp:119-124).
        let mut bm = BlockLocator::from_parts(0, identity.clone(), vec![], vec![0]);
        let loc = BlockLocator::from_parts(1, identity, vec![], vec![0]);
        assert!(matches!(
            bm.make_relative_to(&system, &loc),
            Err(StructureError::RepInvariantViolation { .. })
        ));
    }

    #[test]
    fn int_item_rejects_a_wrong_rank_gamma() {
        let system = a2();
        let mut table = IntegralDatumTable::new();
        assert_eq!(
            table.int_item(&system, &gamma(&[1, 0, 0], 2)),
            Err(StructureError::RankMismatch {
                expected: 2,
                actual: 3,
            })
        );
    }
}
