//! The twisted-involution table (KGB stage b).
//!
//! Per-Cartan contiguous orbits of twisted involutions, each record carrying
//! the word-level element, the matrix-level [`TwistedInvolution`] (theta plus
//! root classification), the mod-2 dedup subspace, `(1+theta)rho`, both
//! lengths, and the `(1-theta)X^*` image-basis pair (`lift_mat`, `M_real`).
//! Every record field is derived canonically from theta at entry EXCEPT the
//! image-basis pair: matching upstream's `InvolutionTable` record
//! (involutions.h:104-105), it is seeded by the echelon reduction of
//! `1-theta` at the orbit's canonical involution (involutions.cpp:196-208)
//! and transported along the cross-action BFS (involutions.cpp:242-243),
//! because the basis is path-dependent and `y_lift`'s signs depend on it.
//! The mod-2 dedup subspace is likewise seeded fresh and thereafter
//! transported along the cross edge (involutions.cpp:256,
//! [`transport_mod_space`]) — its basis is canonical RREF
//! ([`ModTwoSubspace::insert`]), hence path-INDEPENDENT, so the cheap
//! word-parallel transport reproduces the fresh
//! `negative_coweight_eigenspace` + `reduce_basis_mod_two` result exactly
//! while skipping that exact rational matrix work per record.
//! Numbering is the caller's Cartan add order (the documented discipline is
//! ascending [`CartanId`]) with an external-order BFS inside each orbit.

use std::collections::HashMap;
use std::sync::Arc;

use crate::grading::try_capacity;
use crate::inner_class::PermutationHasherBuilder;
use crate::integer_lattice::{negative_coweight_eigenspace, reduce_basis_mod_two};
use crate::real_projection::RealProjection;
use crate::weyl_transducer::{CompactWeyl, WeylElt};
use crate::{
    CartanClassification, CartanId, CayleyCrossDecomposition, InnerClass, IntegerLatticeBudget,
    LatticeInvolution, ModTwoSubspace, ModTwoVector, RootId, RootKind, RootSystem, StructureError,
    TwistedInvolution, Weight, WeylAction, WeylElement,
};

/// Stable identifier of one twisted involution in one table's numbering.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct InvolutionId(pub(crate) usize);

/// Owned budgets for one involution table: the entry-count cap plus the
/// nested integral budget threading to the per-entry eigenlattice reduction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvolutionTableBudget {
    max_involutions: usize,
    integer_lattice: IntegerLatticeBudget,
}

impl InvolutionTableBudget {
    pub const fn new(max_involutions: usize, integer_lattice: IntegerLatticeBudget) -> Self {
        Self {
            max_involutions,
            integer_lattice,
        }
    }
}

/// One twisted involution's record: canonical-from-theta data plus the
/// path-transported `(1-theta)X^*` image-basis pair and dedup subspace.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvolutionRecord {
    element: WeylElt,
    legacy_element: WeylElement,
    involution: TwistedInvolution,
    mod_space: ModTwoSubspace,
    theta_plus_one_rho: Weight,
    involution_length: usize,
    weyl_length: usize,
    projection: RealProjection,
}

impl InvolutionRecord {
    pub fn weyl_element(&self) -> &WeylElement {
        &self.legacy_element
    }

    pub(crate) fn compact_weyl_element(&self) -> WeylElt {
        self.element
    }

    pub fn twisted_involution(&self) -> &TwistedInvolution {
        &self.involution
    }

    /// Shallow convenience for the composed lattice involution theta.
    pub fn theta(&self) -> &LatticeInvolution {
        self.involution.root_involution().involution()
    }

    /// The X_* mod-2 dedup subspace; its ordered basis serves the
    /// inverse-Cayley repair of the Tits stage. Seeded fresh at the orbit's
    /// canonical involution, then transported along the cross edge that first
    /// reached the record ([`transport_mod_space`]) — value-identical to a
    /// fresh reduction because the basis is canonical RREF.
    pub fn mod_space(&self) -> &ModTwoSubspace {
        &self.mod_space
    }

    pub fn involution_length(&self) -> usize {
        self.involution_length
    }

    pub fn weyl_length(&self) -> usize {
        self.weyl_length
    }

    pub fn theta_plus_one_rho(&self) -> &Weight {
        &self.theta_plus_one_rho
    }

    /// The `(1-theta)X^*` image-basis pair (upstream `record`'s
    /// `M_real`/`lift_mat`): seeded from theta at the orbit's canonical
    /// involution, then transported along the cross-action BFS
    /// (involutions.cpp:242-243), so it carries the path-dependent signs
    /// the oracle's `y_lift` relies on.
    pub(crate) fn projection(&self) -> &RealProjection {
        &self.projection
    }
}

/// Dedup/lookup key of a table entry: the images of the SIMPLE roots only.
///
/// A Weyl element is fixed by its simple-root images, so the packed key is
/// injective — equality semantics are unchanged. For semisimple rank <= 16
/// with <= 256 roots the key packs into a u128 (the layout of
/// `inner_class::PermutationKey`), which keeps the E8 cross-action BFS's
/// ~1.6M probes to an integer hash and compare instead of chasing 240-entry
/// ordered-map keys. Larger tables fall back to the full permutation.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum DedupKey {
    Packed(u128),
    Full(Box<[RootId]>),
}

/// The table's dedup index, with its keying discipline. The map is probed
/// and inserted only — never iterated — so the hash order is unobservable.
#[derive(Clone, Debug)]
struct DedupIndex {
    map: HashMap<DedupKey, InvolutionId, PermutationHasherBuilder>,
    /// Simple roots' positions in the root enumeration, generator order
    /// (the packing layout of [`DedupKey::Packed`]).
    simple_positions: Vec<usize>,
    root_count: usize,
    packed: bool,
}

impl DedupIndex {
    fn new(root_system: &RootSystem) -> Self {
        let simple_positions: Vec<usize> = root_system
            .simple_root_ids()
            .iter()
            .map(|id| id.0)
            .collect();
        let root_count = root_system.roots().len();
        let packed = simple_positions.len() <= 16 && root_count <= usize::from(u8::MAX) + 1;
        Self {
            map: HashMap::default(),
            simple_positions,
            root_count,
            packed,
        }
    }

    /// The packed key of the permutation whose simple-root images are
    /// `image(0..rank)`: generator order, 8 bits per generator.
    fn pack(&self, image: impl Fn(usize) -> RootId) -> u128 {
        let mut key = 0_u128;
        for shift in 0..self.simple_positions.len() {
            key |= (image(shift).0 as u128) << (8 * shift);
        }
        key
    }

    fn key_of(&self, permutation: &[RootId]) -> DedupKey {
        // A foreign-length permutation keys as Full even in packed mode, so
        // it can never alias a stored Packed key (provenance contract).
        if self.packed && permutation.len() == self.root_count {
            DedupKey::Packed(self.pack(|simple| permutation[self.simple_positions[simple]]))
        } else {
            DedupKey::Full(permutation.into())
        }
    }

    fn get(&self, key: &DedupKey) -> Option<InvolutionId> {
        self.map.get(key).copied()
    }

    fn insert(&mut self, key: DedupKey, id: InvolutionId) {
        self.map.insert(key, id);
    }
}

/// `left after middle after right` as permutations: `left[middle[right[i]]]`.
/// The single composed buffer replaces the two temporary `WeylElement`
/// products (four heap buffers and a discarded inverse pass) the cross-edge
/// loop used to pay per edge.
fn compose_permutation(
    left: &[RootId],
    middle: &[RootId],
    right: &[RootId],
    count: usize,
) -> Result<Vec<RootId>, StructureError> {
    let mut composed = try_capacity(count)?;
    for index in 0..count {
        composed.push(left[middle[right[index].0].0]);
    }
    Ok(composed)
}

/// Compare the compact and compatibility representations without allocating
/// a word or a full root permutation. A Weyl element is determined by its
/// simple-root images; applying the elected word to one root runs in reverse
/// word order because the word is accumulated by right multiplication.
fn compact_matches_legacy(
    compact_weyl: &CompactWeyl,
    reflections: &[WeylElement],
    root_system: &RootSystem,
    compact: &WeylElt,
    legacy: &WeylElement,
) -> Result<bool, StructureError> {
    let simple_roots = root_system.simple_root_ids();
    if reflections.len() != simple_roots.len() || compact_weyl.d_out().len() != simple_roots.len() {
        return Err(StructureError::RankMismatch {
            expected: simple_roots.len(),
            actual: reflections.len(),
        });
    }
    if compact_weyl.length(compact) != legacy.length() {
        return Ok(false);
    }
    for &simple_root in simple_roots {
        let mut image = simple_root;
        for piece_index in (0..simple_roots.len()).rev() {
            for &local in compact_weyl
                .word_of_piece(piece_index, compact[piece_index])
                .iter()
                .rev()
            {
                let internal = compact_weyl.piece_offset(piece_index) + local;
                let external = *compact_weyl
                    .d_out()
                    .get(internal)
                    .ok_or(StructureError::InvalidRootAutomorphism)?;
                image = reflections
                    .get(external)
                    .and_then(|reflection| reflection.image(image))
                    .ok_or(StructureError::InvalidRootAutomorphism)?;
            }
        }
        if legacy.image(simple_root) != Some(image) {
            return Ok(false);
        }
    }
    Ok(true)
}

/// The involution table: per-Cartan contiguous orbit slices of twisted
/// involutions, shared across the real forms of one inner class.
#[derive(Clone, Debug)]
pub struct InvolutionTable {
    inner_class: Arc<InnerClass>,
    budget: InvolutionTableBudget,
    twist: Vec<usize>,
    compact_weyl: CompactWeyl,
    compact_index: HashMap<WeylElt, InvolutionId, PermutationHasherBuilder>,
    reflections: Vec<WeylElement>,
    reflection_actions: Vec<WeylAction>,
    /// Mod-2 reductions of the simple roots and coroots, per generator:
    /// the cross-edge transport of the dedup subspace applies
    /// `b |-> b XOR <b, alpha_s> * beta_s` (upstream
    /// `mod_space.apply(torus_simple_reflection[s])`, involutions.cpp:256).
    root_parity: Vec<ModTwoVector>,
    coroot_parity: Vec<ModTwoVector>,
    two_rho: Weight,
    records: Vec<InvolutionRecord>,
    index: DedupIndex,
    cross_links: Vec<Vec<InvolutionId>>,
    orbits: Vec<(CartanId, usize, usize)>,
}

impl InvolutionTable {
    /// An empty table for one inner class, filled by [`Self::add_cartan`].
    ///
    /// The inner class owns the datum, root system, and distinguished
    /// involution together, so no cross-input gate is needed. Derived once
    /// here: the validated simple twist, the rank simple-reflection
    /// elements (the BFS edge must NOT rebuild reflections per call), and
    /// `2rho` from the positivity slice.
    pub fn new(
        inner_class: &InnerClass,
        budget: InvolutionTableBudget,
    ) -> Result<Self, StructureError> {
        let datum = inner_class.datum();
        let root_system = inner_class.root_system();
        let delta_data = inner_class.distinguished_involution();
        let semisimple_rank = datum.semisimple_rank();
        let simple_ids = root_system.simple_root_ids();
        let cartan: Vec<Vec<i32>> = datum
            .cartan_matrix()
            .iter()
            .map(|row| row.to_vec())
            .collect();
        let compact_weyl = CompactWeyl::new(&cartan)?;

        let mut twist = try_capacity(semisimple_rank)?;
        for &simple_id in simple_ids {
            let image = delta_data
                .image(simple_id)
                .ok_or(StructureError::InvalidBasedAutomorphism)?;
            let position = simple_ids
                .iter()
                .position(|&candidate| candidate == image)
                .ok_or(StructureError::InvalidBasedAutomorphism)?;
            twist.push(position);
        }

        let mut reflections = try_capacity(semisimple_rank)?;
        let mut reflection_actions = try_capacity(semisimple_rank)?;
        for generator in 0..semisimple_rank {
            let action = WeylAction::simple_reflection(datum, generator)?;
            reflections.push(WeylElement::from_action(root_system, &action)?);
            reflection_actions.push(action);
        }

        let lattice_rank = datum.lattice_rank();
        let mut root_parity = try_capacity(semisimple_rank)?;
        let mut coroot_parity = try_capacity(semisimple_rank)?;
        for generator in 0..semisimple_rank {
            root_parity.push(crate::tits_element::parity_vector(
                datum.simple_roots()[generator].as_slice(),
            )?);
            coroot_parity.push(crate::tits_element::parity_vector(
                datum.simple_coroots()[generator].as_slice(),
            )?);
        }
        let mut two_rho = try_capacity(lattice_rank)?;
        two_rho.resize(lattice_rank, 0_i32);
        for (id, root, _) in root_system.entries() {
            if root_system.positivity()[id.0] {
                for (sum, &coordinate) in two_rho.iter_mut().zip(root.as_slice()) {
                    *sum = sum
                        .checked_add(coordinate)
                        .ok_or(StructureError::ArithmeticOverflow)?;
                }
            }
        }

        Ok(Self {
            inner_class: Arc::new(inner_class.clone()),
            budget,
            twist,
            compact_weyl,
            compact_index: HashMap::default(),
            reflections,
            reflection_actions,
            root_parity,
            coroot_parity,
            two_rho: Weight::new(two_rho),
            records: Vec::new(),
            index: DedupIndex::new(root_system),
            cross_links: Vec::new(),
            orbits: Vec::new(),
        })
    }

    /// Generate one Cartan class's orbit as a contiguous slice.
    ///
    /// Idempotent: re-adding a class returns its existing slice. The seed
    /// and expected size both come from the classification's class, so they
    /// can never be a mismatched pair; the generated orbit must fill the
    /// expected size exactly.
    pub fn add_cartan(
        &mut self,
        classification: &CartanClassification,
        cartan: CartanId,
    ) -> Result<(InvolutionId, usize), StructureError> {
        if let Some(&(_, start, size)) = self.orbits.iter().find(|(id, _, _)| *id == cartan) {
            return Ok((InvolutionId(start), size));
        }
        let class = classification
            .cartan_class(cartan)
            .ok_or(StructureError::IndexOutOfRange {
                index: cartan.0,
                upper_bound: classification.cartan_classes().len(),
            })?;
        let representative = class.representative();
        self.gate_twisted(representative)?;
        let expected = class.twisted_involution_count();

        let start = self.records.len();
        self.records
            .try_reserve(expected)
            .map_err(|_| StructureError::AllocationFailed {
                requested: expected,
            })?;
        self.cross_links
            .try_reserve(expected)
            .map_err(|_| StructureError::AllocationFailed {
                requested: expected,
            })?;
        self.index
            .map
            .try_reserve(expected)
            .map_err(|_| StructureError::AllocationFailed {
                requested: expected,
            })?;
        self.compact_index
            .try_reserve(expected)
            .map_err(|_| StructureError::AllocationFailed {
                requested: expected,
            })?;

        // Seed: convert the matrix-level representative once, then apply the
        // (W_length + #Cayley)/2 formula. `CayleyCrossDecomposition` is a
        // per-class tool only — never per entry at scale.
        let seed_element =
            WeylElement::from_action(self.inner_class.root_system(), representative.weyl_action())?;
        let seed_compact = self.compact_weyl.encode_element(
            self.inner_class.datum(),
            self.inner_class.root_system(),
            &self.reflection_actions,
            &seed_element,
        )?;
        let seed_w_length = self.compact_weyl.length(&seed_compact);
        let decomposition = CayleyCrossDecomposition::build(
            &self.inner_class,
            representative,
            seed_w_length
                .checked_add(1)
                .ok_or(StructureError::ArithmeticOverflow)?,
        )?;
        let cayley_count = decomposition.cayley_roots().len();
        let length_sum = seed_w_length
            .checked_add(cayley_count)
            .ok_or(StructureError::ArithmeticOverflow)?;
        if length_sum % 2 != 0 {
            return Err(StructureError::InvolutionTableInvariantViolation {
                invariant: "length parity",
            });
        }
        push_record(
            &self.inner_class,
            &self.budget,
            &self.two_rho,
            &self.compact_weyl,
            &self.reflections,
            &mut self.records,
            &mut self.index,
            seed_compact,
            seed_element,
            seed_w_length,
            representative.weyl_action().clone(),
            length_sum / 2,
            None,
            None,
        )?;
        if let Some(existing) = self.compact_index.insert(seed_compact, InvolutionId(start)) {
            if existing != InvolutionId(start) {
                return Err(StructureError::InvolutionTableInvariantViolation {
                    invariant: "compact index uniqueness",
                });
            }
        }

        // External-order BFS. Cross links are filled at each node's visit,
        // so after the orbit closes every (generator, node) edge is an O(1)
        // stored link. The dedup probe is allocation-free on the hit path
        // (the majority: rank-1 of every rank edges): the neighbor's packed
        // simple-image key decides membership, and the full neighbor
        // permutation — one composed buffer, no temporary WeylElement
        // products — is built only for NEW involutions, matching the cost
        // profile of upstream's add_cross (involutions.cpp:228-258), which
        // pays one fixed-size twistedConjugate per edge.
        let semisimple_rank = self.twist.len();
        let root_count = self.inner_class.root_system().roots().len();
        let mut cursor = start;
        while cursor < self.records.len() {
            let mut links = try_capacity(semisimple_rank)?;
            for generator in 0..semisimple_rank {
                // The composed neighbor `s * w * twist(s)`, as permutations:
                // `composed[i] = left[current[right[i]]]`.
                let (left, current, right) = {
                    let current = self.records[cursor].legacy_element.image_permutation();
                    let left = self.reflections[generator].image_permutation();
                    let right = self.reflections[self.twist[generator]].image_permutation();
                    (left, current, right)
                };
                let theta = self.records[cursor]
                    .twisted_involution()
                    .root_involution()
                    .image_permutation();
                let delta = self
                    .inner_class
                    .distinguished_involution()
                    .image_permutation();
                if self.index.packed {
                    let probe = DedupKey::Packed(self.index.pack(|simple| {
                        let position = self.index.simple_positions[simple];
                        left[theta[delta[right[position].0].0].0]
                    }));
                    if let Some(existing) = self.index.get(&probe) {
                        links.push(existing);
                        continue;
                    }
                }
                // The compact transition is the hot-path result. Re-encoding
                // the materialized permutation is retained as a debug/test
                // invariant, but must not tax release KGB construction.
                let mut compact_neighbor = self.records[cursor].element;
                self.compact_weyl
                    .inner_mult(&mut compact_neighbor, self.twist[generator]);
                self.compact_weyl
                    .inner_left_mult(&mut compact_neighbor, generator);
                let neighbor = if self.index.packed {
                    WeylElement::from_twisted_composition(
                        self.inner_class.root_system(),
                        left,
                        theta,
                        delta,
                        right,
                    )?
                } else {
                    let composed = compose_permutation(left, current, right, root_count)?;
                    let probe = DedupKey::Full(composed.as_slice().into());
                    if let Some(existing) = self.index.get(&probe) {
                        links.push(existing);
                        continue;
                    }
                    WeylElement::from_permutation(self.inner_class.root_system(), composed)?
                };
                #[cfg(debug_assertions)]
                {
                    let encoded_neighbor = self.compact_weyl.encode_element(
                        self.inner_class.datum(),
                        self.inner_class.root_system(),
                        &self.reflection_actions,
                        &neighbor,
                    )?;
                    if compact_neighbor != encoded_neighbor {
                        return Err(StructureError::InvolutionTableInvariantViolation {
                            invariant: "compact cross action",
                        });
                    }
                }
                let neighbor_w_length = self.compact_weyl.length(&compact_neighbor);
                let new_length = stepped_length(
                    self.records[cursor].involution_length,
                    self.records[cursor].weyl_length,
                    neighbor_w_length,
                )?;
                // The same product `s_g * w * s_{twist(g)}` at the matrix
                // level, by reflection sparsity (rank^2 per compose instead
                // of rank^3; exact integer equality with the compose path).
                let new_action = self.records[cursor]
                    .involution
                    .weyl_action()
                    .left_compose_simple(generator)?
                    .right_compose_simple(self.twist[generator])?;
                // Transport the image basis across the cross edge
                // (involutions.cpp:242-243): the PLAIN generator s, not
                // twist(s) — delta is already incorporated in theta.
                let transported = self.records[cursor]
                    .projection()
                    .transported(self.reflection_actions[generator].matrix())?;
                // Transport the dedup subspace across the same edge
                // (involutions.cpp:256): canonical RREF, so the result equals
                // the fresh per-record `reduce_basis_mod_two` bit for bit.
                let transported_space = transport_mod_space(
                    self.records[cursor].mod_space(),
                    &self.root_parity[generator],
                    &self.coroot_parity[generator],
                )?;
                let id = push_record(
                    &self.inner_class,
                    &self.budget,
                    &self.two_rho,
                    &self.compact_weyl,
                    &self.reflections,
                    &mut self.records,
                    &mut self.index,
                    compact_neighbor,
                    neighbor,
                    neighbor_w_length,
                    new_action,
                    new_length,
                    Some(transported),
                    Some(transported_space),
                )?;
                if let Some(existing) = self.compact_index.insert(compact_neighbor, id) {
                    if existing != id {
                        return Err(StructureError::InvolutionTableInvariantViolation {
                            invariant: "compact index uniqueness",
                        });
                    }
                }
                links.push(id);
            }
            self.cross_links.push(links);
            cursor = cursor
                .checked_add(1)
                .ok_or(StructureError::ArithmeticOverflow)?;
        }

        let size = self.records.len() - start;
        if size != expected {
            return Err(StructureError::InvolutionTableInvariantViolation {
                invariant: "orbit size",
            });
        }
        self.orbits.push((cartan, start, size));
        Ok((InvolutionId(start), size))
    }

    /// Number of a twisted involution, if its Cartan class has been added.
    /// Keyed by the forward root permutation, which stage (a) pinned as a
    /// complete equality key; a same-cardinality foreign system remains the
    /// caller's contract.
    pub fn lookup(&self, element: &WeylElement) -> Option<InvolutionId> {
        self.index
            .get(&self.index.key_of(element.image_permutation()))
    }

    /// Bounded by the involution count.
    pub fn record(&self, id: InvolutionId) -> Option<&InvolutionRecord> {
        self.records.get(id.0)
    }

    /// C++ `WeylElt::pieces` key stored alongside each involution record.
    /// This is the upstream tie-break representation used by KGB sorting.
    pub(crate) fn compact_key(&self, id: InvolutionId) -> Option<[u8; 8]> {
        self.records
            .get(id.0)
            .map(InvolutionRecord::compact_weyl_element)
    }

    /// Recover the Weyl factor from `theta = w after delta` without
    /// materializing a root permutation: `w(r) = theta(delta(r))` because
    /// the distinguished automorphism is an involution.
    pub(crate) fn weyl_image(&self, id: InvolutionId, root: RootId) -> Option<RootId> {
        let record = self.records.get(id.0)?;
        let delta_image = self
            .inner_class
            .distinguished_involution()
            .image_permutation()
            .get(root.0)
            .copied()?;
        record
            .twisted_involution()
            .root_involution()
            .image(delta_image)
    }

    pub(crate) fn weyl_is_identity(&self, id: InvolutionId) -> Option<bool> {
        self.records
            .get(id.0)
            .map(|record| record.element == self.compact_weyl.identity())
    }

    pub(crate) fn weyl_has_left_descent(
        &self,
        id: InvolutionId,
        generator: usize,
    ) -> Result<bool, StructureError> {
        if generator >= self.twist.len() {
            return Err(StructureError::IndexOutOfRange {
                index: generator,
                upper_bound: self.twist.len(),
            });
        }
        let mut element = self
            .records
            .get(id.0)
            .ok_or(StructureError::IndexOutOfRange {
                index: id.0,
                upper_bound: self.records.len(),
            })?
            .element;
        Ok(self.compact_weyl.inner_left_mult(&mut element, generator) < 0)
    }

    pub(crate) fn weyl_first_left_descent(
        &self,
        id: InvolutionId,
        generators: &[usize],
    ) -> Result<Option<usize>, StructureError> {
        let element = self
            .records
            .get(id.0)
            .ok_or(StructureError::IndexOutOfRange {
                index: id.0,
                upper_bound: self.records.len(),
            })?
            .element;
        for &generator in generators {
            if generator >= self.twist.len() {
                return Err(StructureError::IndexOutOfRange {
                    index: generator,
                    upper_bound: self.twist.len(),
                });
            }
            let mut candidate = element;
            if self.compact_weyl.inner_left_mult(&mut candidate, generator) < 0 {
                return Ok(Some(generator));
            }
        }
        Ok(None)
    }

    pub(crate) fn weyl_has_twisted_commutation(
        &self,
        id: InvolutionId,
        generator: usize,
    ) -> Result<bool, StructureError> {
        let mut element = self
            .records
            .get(id.0)
            .ok_or(StructureError::IndexOutOfRange {
                index: id.0,
                upper_bound: self.records.len(),
            })?
            .element;
        let twisted_generator =
            *self
                .twist
                .get(generator)
                .ok_or(StructureError::IndexOutOfRange {
                    index: generator,
                    upper_bound: self.twist.len(),
                })?;
        if twisted_generator >= self.twist.len() {
            return Err(StructureError::IndexOutOfRange {
                index: twisted_generator,
                upper_bound: self.twist.len(),
            });
        }
        let change = self
            .compact_weyl
            .inner_mult(&mut element, twisted_generator);
        let has_left_descent = self.compact_weyl.inner_left_mult(&mut element, generator) < 0;
        Ok((change > 0) == has_left_descent)
    }

    pub(crate) fn weyl_right_length_change(
        &self,
        id: InvolutionId,
        generator: usize,
    ) -> Result<i8, StructureError> {
        if generator >= self.twist.len() {
            return Err(StructureError::IndexOutOfRange {
                index: generator,
                upper_bound: self.twist.len(),
            });
        }
        let mut element = self
            .records
            .get(id.0)
            .ok_or(StructureError::IndexOutOfRange {
                index: id.0,
                upper_bound: self.records.len(),
            })?
            .element;
        Ok(self.compact_weyl.inner_mult(&mut element, generator))
    }

    pub(crate) fn weyl_word(&self, id: InvolutionId) -> Result<Vec<usize>, StructureError> {
        let element = &self
            .records
            .get(id.0)
            .ok_or(StructureError::IndexOutOfRange {
                index: id.0,
                upper_bound: self.records.len(),
            })?
            .element;
        Ok(self.compact_weyl.element_word(element))
    }

    /// Translate a stored Weyl factor by an external diagram permutation and
    /// resolve it through the compact table index. This is the hot-path
    /// counterpart of `WeylGroup::translation`: it performs no root
    /// permutation materialization and no per-letter `WeylElement`
    /// allocation.
    pub(crate) fn weyl_twisted_lookup(
        &self,
        id: InvolutionId,
        twist: &[usize],
    ) -> Result<Option<InvolutionId>, StructureError> {
        let element = &self
            .records
            .get(id.0)
            .ok_or(StructureError::IndexOutOfRange {
                index: id.0,
                upper_bound: self.records.len(),
            })?
            .element;
        let translated = self.compact_weyl.try_apply_twist(element, twist)?;
        Ok(self.compact_index.get(&translated).copied())
    }

    /// The stored cross-action link `s * w * twist(s)` — O(1) after build.
    pub fn cross(
        &self,
        generator: usize,
        id: InvolutionId,
    ) -> Result<InvolutionId, StructureError> {
        let links = self
            .cross_links
            .get(id.0)
            .ok_or(StructureError::IndexOutOfRange {
                index: id.0,
                upper_bound: self.cross_links.len(),
            })?;
        links
            .get(generator)
            .copied()
            .ok_or(StructureError::IndexOutOfRange {
                index: generator,
                upper_bound: links.len(),
            })
    }

    /// The Cayley neighbor `s * w`, or `None` while its Cartan class has not
    /// been added. The stage-(e) contract adds the form's upward-closed
    /// Cartan set first, after which `None` is the caller's invariant
    /// violation.
    ///
    /// The packed probe answers from the simple-root images of `s * w`
    /// alone — an injective key — so the hot path never materializes the
    /// product element.
    pub fn cayley(
        &self,
        generator: usize,
        id: InvolutionId,
    ) -> Result<Option<InvolutionId>, StructureError> {
        let record = self
            .records
            .get(id.0)
            .ok_or(StructureError::IndexOutOfRange {
                index: id.0,
                upper_bound: self.records.len(),
            })?;
        let reflection =
            self.reflections
                .get(generator)
                .ok_or(StructureError::IndexOutOfRange {
                    index: generator,
                    upper_bound: self.reflections.len(),
                })?;
        let current = record.legacy_element.image_permutation();
        let left = reflection.image_permutation();
        if self.index.packed {
            let probe = DedupKey::Packed(self.index.pack(|simple| {
                let position = self.index.simple_positions[simple];
                left[current[position].0]
            }));
            return Ok(self.index.get(&probe));
        }
        let product =
            reflection.multiply(self.inner_class.root_system(), &record.legacy_element)?;
        Ok(self
            .index
            .get(&self.index.key_of(product.image_permutation())))
    }

    /// One accessor covering upstream's three `is_*_simple` tests.
    pub fn simple_root_kind(&self, id: InvolutionId, generator: usize) -> Option<RootKind> {
        let record = self.records.get(id.0)?;
        let &simple_id = self
            .inner_class
            .root_system()
            .simple_root_ids()
            .get(generator)?;
        record.involution.root_involution().kind(simple_id)
    }

    /// The Cartan class whose orbit slice contains this involution.
    pub fn cartan_of(&self, id: InvolutionId) -> Option<CartanId> {
        self.orbits
            .iter()
            .find(|(_, start, size)| id.0 >= *start && id.0 < start + size)
            .map(|&(cartan, _, _)| cartan)
    }

    pub fn involution_count(&self) -> usize {
        self.records.len()
    }

    /// The contiguous orbit slice of an added Cartan class, with its typed
    /// starting number.
    pub fn orbit_slice(&self, cartan: CartanId) -> Option<(InvolutionId, &[InvolutionRecord])> {
        self.orbits
            .iter()
            .find(|(id, _, _)| *id == cartan)
            .map(|&(_, start, size)| (InvolutionId(start), &self.records[start..start + size]))
    }

    pub fn root_system(&self) -> &RootSystem {
        self.inner_class.root_system()
    }

    pub fn inner_class(&self) -> &InnerClass {
        self.inner_class.as_ref()
    }

    pub(crate) fn inner_class_shared(&self) -> &Arc<InnerClass> {
        &self.inner_class
    }

    /// Factorization provenance: the representative must be `w after delta`
    /// for THIS inner class's delta.
    fn gate_twisted(&self, twisted: &TwistedInvolution) -> Result<(), StructureError> {
        use crate::twisted_involution::compose_matrices;
        if twisted.weyl_action().datum() != self.inner_class.datum() {
            return Err(StructureError::DatumMismatch);
        }
        let delta = self.inner_class.distinguished_involution().involution();
        let stored = twisted.root_involution().involution();
        if compose_matrices(twisted.weyl_action().matrix(), delta.weight_matrix())?
            != stored.weight_matrix()
            || compose_matrices(
                twisted.weyl_action().coweight_matrix(),
                delta.coweight_matrix(),
            )? != stored.coweight_matrix()
        {
            return Err(StructureError::DistinguishedInvolutionMismatch);
        }
        Ok(())
    }
}

/// Involution length across one cross edge: the Weyl-length change must be
/// exactly `+-2` for a NEW involution (`0` means the edge fixes it, which the
/// dedup hit already consumed), and the involution length steps by half.
fn stepped_length(
    current_length: usize,
    current_w_length: usize,
    neighbor_w_length: usize,
) -> Result<usize, StructureError> {
    let before =
        isize::try_from(current_w_length).map_err(|_| StructureError::ArithmeticOverflow)?;
    let after =
        isize::try_from(neighbor_w_length).map_err(|_| StructureError::ArithmeticOverflow)?;
    let change = after
        .checked_sub(before)
        .ok_or(StructureError::ArithmeticOverflow)?;
    match change {
        2 => current_length
            .checked_add(1)
            .ok_or(StructureError::ArithmeticOverflow),
        -2 => current_length
            .checked_sub(1)
            .ok_or(StructureError::ArithmeticOverflow),
        _ => Err(StructureError::InvolutionTableInvariantViolation {
            invariant: "twisted length step",
        }),
    }
}

/// The cross-edge transport of the dedup subspace: the `-1` coweight
/// eigenspace of `s theta s` is `s(E)`, and [`ModTwoSubspace`] keeps a
/// canonical RREF basis (insertion-order independent), so re-inserting the
/// reflected basis vectors reproduces the fresh
/// `reduce_basis_mod_two(negative_coweight_eigenspace(..))` object exactly.
/// This is upstream's `mod_space.apply(torus_simple_reflection[s])`
/// (involutions.cpp:256): `b |-> b XOR <b, alpha_s> * beta_s` on coweight
/// parities, with the PLAIN generator s, matching the projection transport.
fn transport_mod_space(
    space: &ModTwoSubspace,
    root_parity: &ModTwoVector,
    coroot_parity: &ModTwoVector,
) -> Result<ModTwoSubspace, StructureError> {
    let mut transported = ModTwoSubspace::new(space.dimension())?;
    for basis in space.basis_vectors() {
        let mut image = basis.clone();
        if root_parity.dot(basis)? {
            image.xor_assign(coroot_parity)?;
        }
        transported.insert(image)?;
    }
    Ok(transported)
}

/// The single entry path: every record field except the image-basis pair and
/// the dedup subspace is derived fresh from theta; those two are seeded from
/// theta at the orbit's canonical involution and thereafter TRANSPORTED along
/// the cross edge that first reached the record (`Some`), matching upstream's
/// `add_involution` / `add_cross` split. The image basis is path-dependent,
/// so it is never re-derived from theta away from the seed; the dedup
/// subspace is path-INDEPENDENT (canonical RREF), but transporting it via
/// [`transport_mod_space`] is word-parallel cheap while the fresh
/// `negative_coweight_eigenspace` + `reduce_basis_mod_two` pair is exact
/// rational matrix work — the dominant serial cost of the BFS.
/// A free function over disjoint table fields so the BFS can hold the
/// reflection caches while inserting.
#[allow(clippy::too_many_arguments)]
fn push_record(
    inner_class: &InnerClass,
    budget: &InvolutionTableBudget,
    two_rho: &Weight,
    compact_weyl: &CompactWeyl,
    reflections: &[WeylElement],
    records: &mut Vec<InvolutionRecord>,
    index: &mut DedupIndex,
    element: WeylElt,
    legacy_element: WeylElement,
    weyl_length: usize,
    action: WeylAction,
    involution_length: usize,
    transported_projection: Option<RealProjection>,
    transported_mod_space: Option<ModTwoSubspace>,
) -> Result<InvolutionId, StructureError> {
    if records.len() == budget.max_involutions {
        return Err(StructureError::InvolutionTableResourceLimit {
            resource: "involutions",
            limit: budget.max_involutions,
        });
    }
    // The record's root action of theta = w after delta, composed at the
    // permutation level: `w_perm[delta_perm[r]]` equals the composed matrix
    // action, so classification needs no per-root matrix work.
    let delta_images = inner_class.distinguished_involution().image_permutation();
    if !compact_matches_legacy(
        compact_weyl,
        reflections,
        inner_class.root_system(),
        &element,
        &legacy_element,
    )? || legacy_element.length() != weyl_length
    {
        return Err(StructureError::InvolutionTableInvariantViolation {
            invariant: "compact Weyl element",
        });
    }
    let w_images = legacy_element.image_permutation();
    let mut root_images = try_capacity(delta_images.len())?;
    for delta_image in delta_images {
        root_images.push(w_images[delta_image.0]);
    }
    let involution = TwistedInvolution::new_from_root_images(
        inner_class.datum(),
        inner_class.root_system(),
        inner_class.distinguished_involution().involution(),
        action,
        root_images,
    )?;
    let theta = involution.root_involution().involution();
    let mod_space = match transported_mod_space {
        Some(space) => space,
        None => {
            let eigenlattice =
                negative_coweight_eigenspace(theta.coweight_matrix(), &budget.integer_lattice)?;
            reduce_basis_mod_two(&eigenlattice)?
        }
    };
    let theta_two_rho = theta.act_on_weight(two_rho)?;
    let mut coordinates = try_capacity(two_rho.as_slice().len())?;
    for (&plain, &reflected) in two_rho.as_slice().iter().zip(theta_two_rho.as_slice()) {
        let sum = plain
            .checked_add(reflected)
            .ok_or(StructureError::ArithmeticOverflow)?;
        if sum % 2 != 0 {
            return Err(StructureError::InvolutionTableInvariantViolation {
                invariant: "theta rho parity",
            });
        }
        coordinates.push(sum / 2);
    }
    let projection = match transported_projection {
        Some(projection) => {
            // The transport preserves lift_mat*m_real == 1-theta
            // algebraically; verify the edge math against this record's
            // freshly derived theta.
            projection.check_against(theta)?;
            projection
        }
        None => RealProjection::build(theta)?,
    };
    let id = InvolutionId(records.len());
    let key = index.key_of(legacy_element.image_permutation());
    index.insert(key, id);
    records.push(InvolutionRecord {
        element,
        legacy_element,
        involution,
        mod_space,
        theta_plus_one_rho: Weight::new(coordinates),
        involution_length,
        weyl_length,
        projection,
    });
    Ok(id)
}

#[cfg(test)]
mod tests {
    use crate::{
        AdjointFiberBudget, BasedRootDatum, CartanClassificationBudget, Coweight,
        LatticeInvolution, ModTwoVector,
    };

    use super::*;

    fn table_budget(max_involutions: usize) -> InvolutionTableBudget {
        InvolutionTableBudget::new(
            max_involutions,
            IntegerLatticeBudget::new(64, 100_000, 100_000, 128),
        )
    }

    fn class_budget(weyl: usize) -> CartanClassificationBudget {
        CartanClassificationBudget::new(
            IntegerLatticeBudget::new(64, 100_000, 100_000, 128),
            AdjointFiberBudget::new(
                IntegerLatticeBudget::new(64, 100_000, 100_000, 128),
                50_000,
                100_000,
            ),
            weyl,
            64,
            64,
        )
    }

    fn context(
        cartan: Vec<Vec<i32>>,
        distinguished: Option<Vec<Vec<i32>>>,
        roots: usize,
        weyl: usize,
    ) -> (InnerClass, CartanClassification) {
        let datum = BasedRootDatum::standard(cartan).unwrap();
        let distinguished = match distinguished {
            Some(matrix) => LatticeInvolution::new(&datum, matrix.clone(), matrix).unwrap(),
            None => LatticeInvolution::identity(&datum).unwrap(),
        };
        let inner_class = InnerClass::new(datum, distinguished, roots).unwrap();
        let classification =
            CartanClassification::build(&inner_class, &class_budget(weyl)).unwrap();
        (inner_class, classification)
    }

    fn filled_table(
        inner_class: &InnerClass,
        classification: &CartanClassification,
        max_involutions: usize,
    ) -> InvolutionTable {
        let mut table = InvolutionTable::new(inner_class, table_budget(max_involutions)).unwrap();
        for id in 0..classification.cartan_classes().len() {
            table.add_cartan(classification, CartanId(id)).unwrap();
        }
        table
    }

    #[test]
    fn a1_split_has_two_singleton_orbits_and_is_idempotent() {
        let (inner_class, classification) = context(vec![vec![2]], None, 2, 2);
        let mut table = InvolutionTable::new(&inner_class, table_budget(4)).unwrap();
        let first = table.add_cartan(&classification, CartanId(0)).unwrap();
        let second = table.add_cartan(&classification, CartanId(1)).unwrap();
        assert_eq!(first, (InvolutionId(0), 1));
        assert_eq!(second, (InvolutionId(1), 1));
        assert_eq!(table.involution_count(), 2);
        assert_eq!(
            table.add_cartan(&classification, CartanId(0)).unwrap(),
            first
        );

        for id in [InvolutionId(0), InvolutionId(1)] {
            assert_eq!(table.cross(0, id).unwrap(), id);
        }
        let fundamental = table.record(InvolutionId(0)).unwrap();
        assert!(fundamental.weyl_element().is_identity());
        assert_eq!(fundamental.involution_length(), 0);
        assert_eq!(fundamental.theta_plus_one_rho(), &Weight::new(vec![1]));
        assert_eq!(fundamental.mod_space().rank(), 0);

        let split = table.record(InvolutionId(1)).unwrap();
        assert_eq!(split.weyl_length(), 1);
        assert_eq!(split.involution_length(), 1);
        assert_eq!(split.theta_plus_one_rho(), &Weight::new(vec![0]));
        assert_eq!(split.mod_space().rank(), 1);
        assert_eq!(
            split
                .mod_space()
                .quotient_representative(ModTwoVector::from_ones(1, vec![0]).unwrap())
                .unwrap(),
            ModTwoVector::zero(1).unwrap()
        );
        assert_eq!(table.cartan_of(InvolutionId(1)), Some(CartanId(1)));
        assert_eq!(
            table.simple_root_kind(InvolutionId(0), 0),
            Some(RootKind::Imaginary)
        );
        assert_eq!(
            table.simple_root_kind(InvolutionId(1), 0),
            Some(RootKind::Real)
        );
    }

    #[test]
    fn b2_split_orbits_match_the_classification_and_are_reproducible() {
        let (inner_class, classification) = context(vec![vec![2, -2], vec![-1, 2]], None, 8, 8);
        let table = filled_table(&inner_class, &classification, 8);
        assert_eq!(
            table.involution_count(),
            classification.twisted_involution_count()
        );
        for (index, class) in classification.cartan_classes().iter().enumerate() {
            let (_, slice) = table.orbit_slice(CartanId(index)).unwrap();
            assert_eq!(slice.len(), class.twisted_involution_count());
        }
        let again = filled_table(&inner_class, &classification, 8);
        for index in 0..classification.cartan_classes().len() {
            assert_eq!(
                table.orbit_slice(CartanId(index)).unwrap(),
                again.orbit_slice(CartanId(index)).unwrap()
            );
        }
    }

    #[test]
    fn b2_compact_keys_equal_legacy_parabolic_piece_keys() {
        let (inner_class, classification) = context(vec![vec![2, -2], vec![-1, 2]], None, 8, 8);
        let table = filled_table(&inner_class, &classification, 8);
        let interface = crate::WeylInterface::new(inner_class.datum().cartan_matrix()).unwrap();
        let pieces = crate::ParabolicPieces::build(table.root_system(), &interface).unwrap();

        for index in 0..table.involution_count() {
            let id = InvolutionId(index);
            let record = table.record(id).unwrap();
            let compact = record.compact_weyl_element();
            let legacy = pieces
                .key(table.root_system(), &interface, record.weyl_element())
                .unwrap();
            assert_eq!(table.compact_key(id), Some(compact));
            assert_eq!(table.compact_index.get(&compact), Some(&id));
            assert_eq!(
                &compact[..legacy.len()],
                legacy
                    .iter()
                    .map(|&piece| u8::try_from(piece).unwrap())
                    .collect::<Vec<_>>()
            );
        }
        assert_eq!(table.compact_index.len(), table.involution_count());
    }

    #[test]
    fn compact_legacy_gate_rejects_different_elements_of_the_same_length() {
        let (inner_class, _) = context(vec![vec![2, -2], vec![-1, 2]], None, 8, 8);
        let table = InvolutionTable::new(&inner_class, table_budget(8)).unwrap();
        let mut compact = table.compact_weyl.identity();
        table.compact_weyl.inner_mult(&mut compact, 0);
        let legacy = WeylElement::simple_reflection(table.root_system(), 1).unwrap();

        assert_eq!(table.compact_weyl.length(&compact), legacy.length());
        assert!(!compact_matches_legacy(
            &table.compact_weyl,
            &table.reflections,
            table.root_system(),
            &compact,
            &legacy,
        )
        .unwrap());
    }

    #[test]
    fn b3_record_compact_values_match_legacy_and_survive_clone() {
        let (inner_class, classification) = context(
            vec![vec![2, -1, 0], vec![-1, 2, -2], vec![0, -1, 2]],
            None,
            18,
            48,
        );
        let table = filled_table(&inner_class, &classification, 48);

        for index in 0..table.involution_count() {
            let id = InvolutionId(index);
            let record = table.record(id).unwrap();
            assert!(compact_matches_legacy(
                &table.compact_weyl,
                &table.reflections,
                table.root_system(),
                &record.compact_weyl_element(),
                record.weyl_element(),
            )
            .unwrap());
            assert_eq!(table.compact_index.get(&record.element), Some(&id));
            assert_eq!(&record.clone(), record);
        }
        assert_eq!(table.compact_index.len(), table.involution_count());
    }

    #[test]
    fn b2_twisted_root_images_recover_stored_weyl_factor() {
        let (inner_class, classification) = context(vec![vec![2, -2], vec![-1, 2]], None, 8, 8);
        let table = filled_table(&inner_class, &classification, 8);
        for index in 0..table.involution_count() {
            let id = InvolutionId(index);
            let record = table.record(id).unwrap();
            for root in 0..table.root_system().roots().len() {
                assert_eq!(
                    table.weyl_image(id, RootId(root)),
                    record.weyl_element().image(RootId(root))
                );
            }
        }
    }

    #[test]
    fn b2_compact_queries_equal_legacy_weyl_element_queries() {
        let (inner_class, classification) = context(vec![vec![2, -2], vec![-1, 2]], None, 8, 8);
        let table = filled_table(&inner_class, &classification, 8);
        for index in 0..table.involution_count() {
            let id = InvolutionId(index);
            let legacy = table.record(id).unwrap().weyl_element();
            assert_eq!(table.weyl_is_identity(id), Some(legacy.is_identity()));
            let word = table.weyl_word(id).unwrap();
            assert_eq!(word.len(), legacy.length());
            let mut rebuilt = WeylElement::identity(table.root_system()).unwrap();
            for generator in word {
                rebuilt = rebuilt
                    .right_multiply_simple(table.root_system(), generator)
                    .unwrap()
                    .0;
            }
            assert_eq!(&rebuilt, legacy);
            for generator in 0..table.root_system().simple_root_ids().len() {
                assert_eq!(
                    table.weyl_has_left_descent(id, generator).unwrap(),
                    legacy
                        .has_left_descent(table.root_system(), generator)
                        .unwrap()
                );
            }
        }
    }

    #[test]
    fn b2_compact_first_left_descent_respects_generator_order() {
        let (inner_class, classification) = context(vec![vec![2, -2], vec![-1, 2]], None, 8, 8);
        let table = filled_table(&inner_class, &classification, 8);
        let orders = [vec![0_usize, 1], vec![1_usize, 0]];

        for index in 0..table.involution_count() {
            let id = InvolutionId(index);
            let legacy = table.record(id).unwrap().weyl_element();
            for order in &orders {
                let expected = order.iter().copied().find(|&generator| {
                    legacy
                        .has_left_descent(table.root_system(), generator)
                        .unwrap()
                });
                assert_eq!(table.weyl_first_left_descent(id, order).unwrap(), expected);
            }
        }
    }

    #[test]
    fn a2_compact_twisted_commutation_matches_legacy_decisions() {
        let (inner_class, classification) = context(
            vec![vec![2, -1], vec![-1, 2]],
            Some(vec![vec![0, 1], vec![1, 0]]),
            6,
            6,
        );
        let table = filled_table(&inner_class, &classification, 6);
        let twist = inner_class.generator_twist().unwrap();

        for index in 0..table.involution_count() {
            let id = InvolutionId(index);
            let legacy = table.record(id).unwrap().weyl_element();
            for generator in 0..twist.len() {
                let (transported, change) = legacy
                    .right_multiply_simple(table.root_system(), twist[generator])
                    .unwrap();
                let expected = (change > 0)
                    == transported
                        .has_left_descent(table.root_system(), generator)
                        .unwrap();
                assert_eq!(
                    table.weyl_has_twisted_commutation(id, generator).unwrap(),
                    expected,
                    "involution {index}, generator {generator}"
                );
            }
        }

        assert!(matches!(
            table.weyl_has_twisted_commutation(InvolutionId(usize::MAX), usize::MAX),
            Err(StructureError::IndexOutOfRange { index, upper_bound })
                if index == usize::MAX && upper_bound == table.involution_count()
        ));
    }

    #[test]
    fn compact_twisted_lookup_matches_legacy_translation() {
        let (inner_class, classification) = context(vec![vec![2, -1], vec![-1, 2]], None, 6, 6);
        let table = filled_table(&inner_class, &classification, 6);
        let twist = [1_usize, 0];
        for index in 0..table.involution_count() {
            let id = InvolutionId(index);
            let compact_target = table.weyl_twisted_lookup(id, &twist).unwrap();
            let word = table.weyl_word(id).unwrap();
            let system = table.root_system();
            let mut translated = WeylElement::identity(system).unwrap();
            for generator in word {
                translated = translated
                    .right_multiply_simple(system, twist[generator])
                    .unwrap()
                    .0;
            }
            assert_eq!(compact_target, table.lookup(&translated));
        }
    }

    #[test]
    fn b2_records_are_canonical_from_theta() {
        let (inner_class, classification) = context(vec![vec![2, -2], vec![-1, 2]], None, 8, 8);
        let table = filled_table(&inner_class, &classification, 8);
        let delta_data = inner_class.distinguished_involution();
        for index in 0..table.involution_count() {
            let record = table.record(InvolutionId(index)).unwrap();
            assert_eq!(record.weyl_length(), record.weyl_element().length());
            for (root, _, _) in inner_class.root_system().entries() {
                let delta_image = delta_data.image(root).unwrap();
                assert_eq!(
                    record.twisted_involution().root_involution().image(root),
                    record.weyl_element().image(delta_image)
                );
            }
            let decomposition = CayleyCrossDecomposition::build(
                &inner_class,
                record.twisted_involution(),
                record.weyl_length() + 1,
            )
            .unwrap();
            assert_eq!(
                record.involution_length(),
                (record.weyl_length() + decomposition.cayley_roots().len()) / 2
            );
            let two_rho = Weight::new(vec![3, 4]);
            let reflected = record.theta().act_on_weight(&two_rho).unwrap();
            let expected: Vec<i32> = two_rho
                .as_slice()
                .iter()
                .zip(reflected.as_slice())
                .map(|(&a, &b)| (a + b) / 2)
                .collect();
            assert_eq!(record.theta_plus_one_rho(), &Weight::new(expected));
            for generator in 0..2 {
                let simple_id = inner_class.root_system().simple_root_ids()[generator];
                assert_eq!(
                    table.simple_root_kind(InvolutionId(index), generator),
                    record
                        .twisted_involution()
                        .root_involution()
                        .kind(simple_id)
                );
            }
        }
    }

    #[test]
    fn b2_cayley_edge_appears_only_once_its_cartan_is_added() {
        let (inner_class, classification) = context(vec![vec![2, -2], vec![-1, 2]], None, 8, 8);
        let mut table = InvolutionTable::new(&inner_class, table_budget(8)).unwrap();
        let (fundamental, size) = table.add_cartan(&classification, CartanId(0)).unwrap();
        assert_eq!(size, 1);
        assert!(table
            .record(fundamental)
            .unwrap()
            .weyl_element()
            .is_identity());
        assert_eq!(table.cayley(0, fundamental).unwrap(), None);

        for id in 1..classification.cartan_classes().len() {
            table.add_cartan(&classification, CartanId(id)).unwrap();
        }
        let target = table.cayley(0, fundamental).unwrap().unwrap();
        let record = table.record(target).unwrap();
        assert_eq!(record.weyl_length(), 1);
        assert_eq!(record.involution_length(), 1);
        for generator in 0..2 {
            assert_eq!(table.cross(generator, fundamental).unwrap(), fundamental);
        }
    }

    #[test]
    fn twisted_a2_orbits_match_the_classification() {
        let (inner_class, classification) = context(
            vec![vec![2, -1], vec![-1, 2]],
            Some(vec![vec![0, 1], vec![1, 0]]),
            6,
            6,
        );
        let table = filled_table(&inner_class, &classification, 4);
        assert_eq!(
            table.involution_count(),
            classification.twisted_involution_count()
        );
        let mut sizes: Vec<usize> = (0..classification.cartan_classes().len())
            .map(|index| table.orbit_slice(CartanId(index)).unwrap().1.len())
            .collect();
        sizes.sort_unstable();
        assert_eq!(sizes, vec![1, 3]);
    }

    /// The B2 x=4 oracle anchor (pinned arm64 oracle run): the involution
    /// theta=[[-1,0],[2,1]] is reached from its Cartan orbit's seed along
    /// cross edges, and the oracle's TRANSPORTED lift_mat column is
    /// [2,-2] — while a fresh echelon reduction of 1-theta gives [-2,2].
    /// The record must carry the transported basis (involutions.cpp:
    /// 242-243), since y_lift's sign depends on it. The datum is the
    /// simply-connected B2 of the dual_KL_block fixture (fundamental-
    /// weight lattice basis).
    #[test]
    fn b2_projection_is_transported_not_recomputed() {
        let datum = BasedRootDatum::from_simple_data(
            2,
            vec![vec![2, -2], vec![-1, 2]],
            vec![Weight::new(vec![2, -2]), Weight::new(vec![-1, 2])],
            vec![Coweight::new(vec![1, 0]), Coweight::new(vec![0, 1])],
        )
        .unwrap();
        let involution = LatticeInvolution::identity(&datum).unwrap();
        let inner_class = InnerClass::new(datum, involution, 8).unwrap();
        let classification = CartanClassification::build(&inner_class, &class_budget(8)).unwrap();
        let table = filled_table(&inner_class, &classification, 8);
        let target = vec![vec![-1, 0], vec![2, 1]];
        let record = (0..table.involution_count())
            .map(InvolutionId)
            .map(|id| table.record(id).unwrap())
            .find(|record| record.theta().weight_matrix() == target.as_slice())
            .expect("B2 table holds theta=[[-1,0],[2,1]]");
        assert_eq!(record.projection().lift_mat, vec![vec![2], vec![-2]]);
        assert_eq!(record.projection().m_real, vec![vec![1, 0]]);
        // The fresh echelon build lands on the opposite sign — the value
        // the parameter layer used to compute on the spot.
        let recomputed = RealProjection::build(record.theta()).unwrap();
        assert_eq!(recomputed.lift_mat, vec![vec![-2], vec![2]]);
        assert_ne!(recomputed, *record.projection());
    }

    /// The cross-edge `mod_space` transport must reproduce the fresh
    /// `negative_coweight_eigenspace` + `reduce_basis_mod_two` reduction on
    /// EVERY record: the `-1` eigenspace is reflection-equivariant and the
    /// RREF basis is canonical, so the equality is exact, not approximate.
    #[test]
    fn b2_mod_space_transport_matches_fresh_reduction() {
        let (inner_class, classification) = context(vec![vec![2, -2], vec![-1, 2]], None, 8, 8);
        let table = filled_table(&inner_class, &classification, 8);
        let budget = IntegerLatticeBudget::new(64, 100_000, 100_000, 128);
        for index in 0..table.involution_count() {
            let record = table.record(InvolutionId(index)).unwrap();
            let eigenlattice =
                negative_coweight_eigenspace(record.theta().coweight_matrix(), &budget).unwrap();
            let fresh = reduce_basis_mod_two(&eigenlattice).unwrap();
            assert_eq!(
                record.mod_space(),
                &fresh,
                "transported mod_space differs from the fresh reduction at record {index}"
            );
        }
    }

    #[test]
    fn budget_and_out_of_range_inputs_are_guarded() {
        let (inner_class, classification) = context(vec![vec![2]], None, 2, 2);
        let mut small = InvolutionTable::new(&inner_class, table_budget(1)).unwrap();
        small.add_cartan(&classification, CartanId(0)).unwrap();
        assert_eq!(
            small.add_cartan(&classification, CartanId(1)),
            Err(StructureError::InvolutionTableResourceLimit {
                resource: "involutions",
                limit: 1,
            })
        );
        assert_eq!(
            small.add_cartan(&classification, CartanId(9)),
            Err(StructureError::IndexOutOfRange {
                index: 9,
                upper_bound: 2,
            })
        );
        assert_eq!(small.record(InvolutionId(99)), None);
        assert_eq!(
            small.cross(5, InvolutionId(0)),
            Err(StructureError::IndexOutOfRange {
                index: 5,
                upper_bound: 1,
            })
        );

        let (foreign_inner, _) = context(vec![vec![2, -1], vec![-1, 2]], None, 6, 6);
        let foreign_identity = WeylElement::identity(foreign_inner.root_system()).unwrap();
        assert_eq!(small.lookup(&foreign_identity), None);
    }
}
