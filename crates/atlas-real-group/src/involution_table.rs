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

use smallvec::SmallVec;

use crate::grading::try_capacity;
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

type CrossLinkRow = SmallVec<[InvolutionId; 8]>;

/// Rotate-xor-multiply accumulation with the murmur3 fmix64 avalanche
/// deferred to `finish`.
///
/// `PermutationHasher`'s bare rotate-xor-multiply rounds only diffuse key
/// entropy UPWARD within a round, while hashbrown selects buckets on the
/// hash's LOW bits: keys whose entropy sits in the high bytes (the compact
/// `WeylElt` pieces — for E8 the variation lives in bytes 3..7, and the
/// derived `Hash` spends the first round on the constant length prefix)
/// collapse onto a handful of buckets (measured ~20x probe blowup on the
/// E8 involution table, lane D profile job 3672155). Avalanching at EVERY
/// word (the previous fmix64-per-word round, 3 multiplies per 8 bytes)
/// fixed that but cost 2.3% on av_ann_e7 (profile 3680833); the same
/// avalanche once in `finish` keeps the low-bit diffusion while the
/// per-word round stays a single multiply. Collision behavior is
/// irrelevant to semantics: the maps are probed/inserted, never iterated.
#[derive(Clone, Default)]
pub(crate) struct MixingHasher(u64);

impl std::hash::Hasher for MixingHasher {
    fn finish(&self) -> u64 {
        // murmur3 fmix64: avalanche all 64 state bits into the low bits.
        let mut mixed = self.0;
        mixed ^= mixed >> 33;
        mixed = mixed.wrapping_mul(0xff51_afd7_ed55_8ccd);
        mixed ^= mixed >> 33;
        mixed = mixed.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
        mixed ^= mixed >> 33;
        mixed
    }

    fn write(&mut self, bytes: &[u8]) {
        let mut chunks = bytes.chunks_exact(8);
        for chunk in &mut chunks {
            self.write_u64(u64::from_le_bytes(chunk.try_into().unwrap_or([0; 8])));
        }
        let remainder = chunks.remainder();
        if !remainder.is_empty() {
            let mut tail = [0_u8; 8];
            tail[..remainder.len()].copy_from_slice(remainder);
            self.write_u64(u64::from_le_bytes(tail));
        }
    }

    fn write_u64(&mut self, value: u64) {
        self.0 = (self.0.rotate_left(5) ^ value).wrapping_mul(0x51_7c_c1_b7_27_22_0a_95);
    }

    fn write_usize(&mut self, value: usize) {
        self.write_u64(value as u64);
    }

    fn write_u128(&mut self, value: u128) {
        self.write_u64(value as u64);
        self.write_u64((value >> 64) as u64);
    }
}

pub(crate) type MixingHasherBuilder = std::hash::BuildHasherDefault<MixingHasher>;

fn cross_link_row(rank: usize) -> Result<CrossLinkRow, StructureError> {
    let mut links = CrossLinkRow::new();
    links
        .try_reserve_exact(rank)
        .map_err(|_| StructureError::AllocationFailed { requested: rank })?;
    Ok(links)
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
///
/// The two lengths are stored as `u32` (Weyl length is bounded by the
/// positive-root count, which the root-system width contract caps at
/// 2^16) and widened to `usize` at the accessors: at ~270k records on the
/// unipotent workload every inline byte is retained heap.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvolutionRecord {
    element: WeylElt,
    involution: TwistedInvolution,
    mod_space: ModTwoSubspace,
    theta_plus_one_rho: Weight,
    involution_length: u32,
    weyl_length: u32,
    projection: RealProjection,
}

impl InvolutionRecord {
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
        self.involution_length as usize
    }

    pub fn weyl_length(&self) -> usize {
        self.weyl_length as usize
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

// Dedup discipline: the record's compact Weyl factor alone is the dedup
// key. Within one table (fixed distinguished delta) `theta = w after
// delta` makes the Weyl factor determine the whole record, so
// `compact_index` ([`WeylElt`] = 8 bytes, one integer hash round) is an
// injective key; the retired `DedupIndex` re-derived the same key from the
// simple-root images at a per-edge packing cost. The map is probed and
// inserted only - never iterated - so the hash order is unobservable.

/// Compute the compact Weyl value for the cross action `s*w*twist(s)`.
fn compact_cross_neighbor(
    compact_weyl: &CompactWeyl,
    mut current: WeylElt,
    generator: usize,
    twisted_generator: usize,
) -> WeylElt {
    compact_weyl.inner_mult(&mut current, twisted_generator);
    compact_weyl.inner_left_mult(&mut current, generator);
    current
}

/// Compare the compact and permutation-level representations without
/// allocating a word or a full root permutation. A Weyl element is
/// determined by its simple-root images; applying the elected word to one
/// root runs in reverse word order because the word is accumulated by right
/// multiplication. `image_of` is the expected root action of the element.
fn compact_matches_images(
    compact_weyl: &CompactWeyl,
    reflections: &[WeylElement],
    root_system: &RootSystem,
    compact: &WeylElt,
    image_of: impl Fn(RootId) -> Option<RootId>,
) -> Result<bool, StructureError> {
    let simple_roots = root_system.simple_root_ids();
    if reflections.len() != simple_roots.len() || compact_weyl.d_out().len() != simple_roots.len() {
        return Err(StructureError::RankMismatch {
            expected: simple_roots.len(),
            actual: reflections.len(),
        });
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
        if image_of(simple_root) != Some(image) {
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
    compact_index: HashMap<WeylElt, InvolutionId, MixingHasherBuilder>,
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
    /// Cached left-descent bits per record: bit `g` of `descent_bits[id]`
    /// is the record's Weyl-factor left descent at generator `g`. A fixed
    /// property of the record's Weyl factor, computed once at push time
    /// instead of per KGB edge (the stage-(e) BFS asks it millions of
    /// times; each answer was a fresh `inner_left_mult`).
    descent_bits: Vec<u32>,
    /// Memoized Cayley links `s * w` per (record, generator), flat
    /// record-major, `u32::MAX` = absent. Valid only while
    /// `cayley_links_valid_len == records.len()`: a record's Cayley target
    /// can belong to a Cartan class added after the record, so
    /// [`Self::ensure_cayley_links`] rebuilds from scratch and any later
    /// `add_cartan` silently returns the accessors to the compact probe.
    cayley_links: Vec<u32>,
    cayley_links_valid_len: usize,
    /// The stored cross-action links, one record-major row of
    /// `twist.len()` generators per record, flattened into a single
    /// allocation: per-record `Vec` headers and per-row malloc rounding
    /// were ~100B of retained overhead per record (E8 unipotent: ~28MB).
    cross_links: Vec<InvolutionId>,
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
            descent_bits: Vec::new(),
            cayley_links: Vec::new(),
            cayley_links_valid_len: 0,
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
        let expected_links = expected
            .checked_mul(self.twist.len())
            .ok_or(StructureError::ArithmeticOverflow)?;
        self.cross_links
            .try_reserve(expected_links)
            .map_err(|_| StructureError::AllocationFailed {
                requested: expected_links,
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
            &mut self.descent_bits,
            seed_compact,
            representative
                .root_involution()
                .image_permutation()
                .to_vec(),
            seed_w_length,
            representative.root_involution().involution().clone(),
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
        // stored link. The dedup probe is the compact Weyl factor itself
        // (8-byte key, one integer hash round): within one table
        // `theta = w after delta` makes the factor an injective record key,
        // so the retired simple-root-image packing decided exactly the same
        // hits. The neighbor's theta root images — one buffer, no
        // WeylElement materialization — are built by transport only for NEW
        // involutions, matching the cost profile of upstream's add_cross
        // (involutions.cpp:228-258), which pays one fixed-size
        // twistedConjugate per edge.
        let semisimple_rank = self.twist.len();
        let delta = self
            .inner_class
            .distinguished_involution()
            .image_permutation();
        let mut cursor = start;
        while cursor < self.records.len() {
            let mut links = cross_link_row(semisimple_rank)?;
            let current_element = self.records[cursor].element;
            for generator in 0..semisimple_rank {
                // The cached reflection permutations serve the theta-image
                // transport on a dedup miss.
                let (left, right) = {
                    let left = self.reflections[generator].image_permutation();
                    let right = self.reflections[self.twist[generator]].image_permutation();
                    (left, right)
                };
                let theta = self.records[cursor]
                    .twisted_involution()
                    .root_involution()
                    .image_permutation();
                let compact_neighbor = compact_cross_neighbor(
                    &self.compact_weyl,
                    current_element,
                    generator,
                    self.twist[generator],
                );
                if let Some(existing) = self.compact_index.get(&compact_neighbor) {
                    links.push(*existing);
                    continue;
                }
                // Transport the theta root images across the cross edge
                // instead of materializing the neighbor's Weyl factor:
                // theta' = w' after delta with w' = s*w*twist(s) composes as
                // left[theta[delta[right[delta[r]]]]] — the index order the
                // retired WeylElement::from_twisted_composition materialized
                // (pinned by cross_edge_theta_transport_reproduces_* tests).
                let mut neighbor_images = try_capacity(theta.len())?;
                for root in 0..theta.len() {
                    neighbor_images.push(left[theta.at(delta.at(right[delta.at(root).0].0).0).0]);
                }
                let neighbor_w_length = self.compact_weyl.length(&compact_neighbor);
                let new_length = stepped_length(
                    self.records[cursor].involution_length as usize,
                    self.records[cursor].weyl_length as usize,
                    neighbor_w_length,
                )?;
                // Transport theta's MATRICES across the cross edge instead
                // of the Weyl factor's: `w |-> s_g w s_{twist(g)}` induces
                // `theta |-> s_g theta s_g` (plain conjugation — the twist
                // cancels against delta, the same identity the permutation
                // transport above relies on), at rank^2 per lattice per side.
                // Records therefore never retain the WeylAction pair.
                let new_theta = self.records[cursor]
                    .involution
                    .root_involution()
                    .involution()
                    .conjugate_simple(generator)?;
                // Transport the image basis across the cross edge
                // (involutions.cpp:242-243): the PLAIN generator s, not
                // twist(s) — delta is already incorporated in theta.
                let transported = self.records[cursor]
                    .projection()
                    .transported_by_simple_reflection(
                        self.inner_class.datum().simple_roots()[generator].as_slice(),
                        self.inner_class.datum().simple_coroots()[generator].as_slice(),
                    )?;
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
                    &mut self.descent_bits,
                    compact_neighbor,
                    neighbor_images,
                    neighbor_w_length,
                    new_theta,
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
            debug_assert_eq!(links.len(), semisimple_rank);
            self.cross_links.extend_from_slice(&links);
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
    /// Resolved through the compact factor: the legacy element is encoded
    /// into the table's compact numbering (an exact, verified boundary), so
    /// the hit set is the retired simple-root-image probe's — a Weyl element
    /// is fixed by its simple-root images, and both keys are injective.
    pub fn lookup(&self, element: &WeylElement) -> Option<InvolutionId> {
        let compact = self
            .compact_weyl
            .encode_element(
                self.inner_class.datum(),
                self.inner_class.root_system(),
                &self.reflection_actions,
                element,
            )
            .ok()?;
        self.compact_index.get(&compact).copied()
    }

    /// Bounded by the involution count.
    pub fn record(&self, id: InvolutionId) -> Option<&InvolutionRecord> {
        self.records.get(id.0)
    }

    /// Materialize one record's compact Weyl factor as a full root action.
    ///
    /// This is an explicit compatibility boundary: records retain only the
    /// allocation-free compact value, while callers that need the legacy
    /// permutation representation opt into its allocation at the call site.
    /// Invalid IDs are rejected before any action materialization.
    pub fn materialize_weyl_element(
        &self,
        id: InvolutionId,
    ) -> Result<WeylElement, StructureError> {
        let element = self
            .records
            .get(id.0)
            .ok_or(StructureError::IndexOutOfRange {
                index: id.0,
                upper_bound: self.records.len(),
            })?
            .element;
        let action = self.compact_weyl.materialize_action(
            self.inner_class.datum(),
            &self.reflection_actions,
            &element,
        )?;
        WeylElement::from_action(self.root_system(), &action)
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
            .get(root.0)?;
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

    /// Resolve the table entry whose compact Weyl factor is the identity.
    pub(crate) fn identity_id(&self) -> Option<InvolutionId> {
        self.compact_index
            .get(&self.compact_weyl.identity())
            .copied()
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
        // Bounds check mirrors the historic records.get; the answer itself
        // is the bit cached at push time (same `inner_left_mult` sign).
        if id.0 >= self.records.len() {
            return Err(StructureError::IndexOutOfRange {
                index: id.0,
                upper_bound: self.records.len(),
            });
        }
        Ok(self.descent_bits[id.0] & (1_u32 << generator) != 0)
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
        let element = self
            .records
            .get(id.0)
            .ok_or(StructureError::IndexOutOfRange {
                index: id.0,
                upper_bound: self.records.len(),
            })?
            .element;
        self.compact_has_twisted_commutation(element, generator)
    }

    fn compact_has_twisted_commutation(
        &self,
        mut element: WeylElt,
        generator: usize,
    ) -> Result<bool, StructureError> {
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

    /// Compact counterpart of upstream
    /// `TwistedWeylGroup::canonical_involution_expr` (weyl.cpp:1359-1385).
    pub fn weyl_canonical_involution_expr(
        &self,
        id: InvolutionId,
    ) -> Result<Vec<i32>, StructureError> {
        let mut current = self
            .records
            .get(id.0)
            .ok_or(StructureError::IndexOutOfRange {
                index: id.0,
                upper_bound: self.records.len(),
            })?
            .element;
        let mut result = try_capacity(self.compact_weyl.length(&current))?;
        while current != self.compact_weyl.identity() {
            let mut descent = None;
            for generator in 0..self.twist.len() {
                let mut reduced = current;
                if self.compact_weyl.inner_left_mult(&mut reduced, generator) < 0 {
                    descent = Some((generator, reduced));
                    break;
                }
            }
            let (generator, reduced) =
                descent.ok_or(StructureError::InvolutionTableInvariantViolation {
                    invariant: "canonical involution left descent",
                })?;
            let signed =
                i32::try_from(generator).map_err(|_| StructureError::ArithmeticOverflow)?;
            if self.compact_has_twisted_commutation(current, generator)? {
                result.push(signed);
                current = reduced;
            } else {
                result.push(!signed);
                current = reduced;
                self.compact_weyl
                    .inner_mult(&mut current, self.twist[generator]);
            }
        }
        Ok(result)
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

    pub fn weyl_word(&self, id: InvolutionId) -> Result<Vec<usize>, StructureError> {
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

    /// Resolve the dual of one source-table involution as an id in this
    /// table, without materializing either Weyl element as a root
    /// permutation.
    pub fn dual_involution_id(
        &self,
        source_table: &InvolutionTable,
        source_id: InvolutionId,
    ) -> Result<Option<InvolutionId>, StructureError> {
        let word = source_table.weyl_word(source_id)?;
        self.weyl_dual_lookup(&word, &self.twist)
    }

    /// Resolve the dual of an involution word without materializing a legacy
    /// root permutation. The duality starts at the dual longest element and
    /// replays the external word backwards with the dual distinguished twist.
    pub(crate) fn weyl_dual_lookup(
        &self,
        word: &[usize],
        dual_twist: &[usize],
    ) -> Result<Option<InvolutionId>, StructureError> {
        let mut current = self.compact_weyl.longest();
        for &generator in word.iter().rev() {
            let twisted = *dual_twist
                .get(generator)
                .ok_or(StructureError::IndexOutOfRange {
                    index: generator,
                    upper_bound: dual_twist.len(),
                })?;
            if twisted >= self.twist.len() {
                return Err(StructureError::IndexOutOfRange {
                    index: twisted,
                    upper_bound: self.twist.len(),
                });
            }
            self.compact_weyl.inner_mult(&mut current, twisted);
        }
        Ok(self.compact_index.get(&current).copied())
    }

    /// Left-multiply a stored compact Weyl factor by an external word and
    /// resolve only the final product through the involution-table index.
    /// Intermediate products need not themselves be twisted involutions.
    pub(crate) fn weyl_left_word_lookup(
        &self,
        id: InvolutionId,
        word: &[usize],
    ) -> Result<Option<InvolutionId>, StructureError> {
        let mut current = self
            .records
            .get(id.0)
            .ok_or(StructureError::IndexOutOfRange {
                index: id.0,
                upper_bound: self.records.len(),
            })?
            .element;
        for &generator in word.iter().rev() {
            if generator >= self.twist.len() {
                return Err(StructureError::IndexOutOfRange {
                    index: generator,
                    upper_bound: self.twist.len(),
                });
            }
            self.compact_weyl.inner_left_mult(&mut current, generator);
        }
        Ok(self.compact_index.get(&current).copied())
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
        let rank = self.twist.len();
        let record_count = if rank == 0 {
            0
        } else {
            self.cross_links.len().checked_div(rank).unwrap_or(0)
        };
        if id.0 >= record_count {
            return Err(StructureError::IndexOutOfRange {
                index: id.0,
                upper_bound: record_count,
            });
        }
        if generator >= rank {
            return Err(StructureError::IndexOutOfRange {
                index: generator,
                upper_bound: rank,
            });
        }
        Ok(self.cross_links[id.0 * rank + generator])
    }

    /// The Cayley neighbor `s * w`, or `None` while its Cartan class has not
    /// been added. The stage-(e) contract adds the form's upward-closed
    /// Cartan set first, after which `None` is the caller's invariant
    /// violation.
    ///
    /// The compact probe applies the simple reflection directly to the
    /// record-owned `WeylElt`, so the hot path never materializes a root
    /// permutation.
    pub fn cayley(
        &self,
        generator: usize,
        id: InvolutionId,
    ) -> Result<Option<InvolutionId>, StructureError> {
        self.compact_cayley_lookup(generator, id)
    }

    /// Resolve the Cayley neighbor from the record-owned compact Weyl value.
    /// This keeps the hot path independent of the compatibility permutation.
    /// When [`Self::ensure_cayley_links`] has run (and no Cartan has been
    /// added since), the memoized link row answers instead of the probe.
    pub(crate) fn compact_cayley_lookup(
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
        if generator >= self.twist.len() {
            return Err(StructureError::IndexOutOfRange {
                index: generator,
                upper_bound: self.twist.len(),
            });
        }
        if self.cayley_links_valid_len == self.records.len()
            && self.cayley_links.len() == self.records.len() * self.twist.len()
        {
            let packed = self.cayley_links[id.0 * self.twist.len() + generator];
            return Ok((packed != u32::MAX).then_some(InvolutionId(packed as usize)));
        }
        let mut product = record.element;
        self.compact_weyl.inner_left_mult(&mut product, generator);
        Ok(self.compact_index.get(&product).copied())
    }

    /// Materialize the memoized Cayley link rows for every record. Called by
    /// the KGB build after its Cartan-add phase: a record's Cayley target
    /// can live in a Cartan class absent when the record was created, so the
    /// rebuild covers ALL records, and the next `add_cartan` invalidates the
    /// whole memo (the length guard in [`Self::compact_cayley_lookup`]).
    /// Tables too large for the u32 packing keep the probe path.
    pub(crate) fn ensure_cayley_links(&mut self) -> Result<(), StructureError> {
        let rank = self.twist.len();
        let Some(total) = self.records.len().checked_mul(rank) else {
            self.cayley_links.clear();
            self.cayley_links_valid_len = 0;
            return Ok(());
        };
        if self.records.len() >= u32::MAX as usize {
            self.cayley_links.clear();
            self.cayley_links_valid_len = 0;
            return Ok(());
        }
        let mut links = try_capacity(total)?;
        for id in 0..self.records.len() {
            for generator in 0..rank {
                let target = self.compact_cayley_lookup(generator, InvolutionId(id))?;
                links.push(match target {
                    Some(target) => u32::try_from(target.0)
                        .map_err(|_| StructureError::ArithmeticOverflow)?,
                    None => u32::MAX,
                });
            }
        }
        self.cayley_links = links;
        self.cayley_links_valid_len = self.records.len();
        Ok(())
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

    /// The canonical representative of an added Cartan class.
    ///
    /// Each orbit is seeded from the classification representative before
    /// its cross-action closure is generated, so the slice's first id is the
    /// canonical representative used by upstream Cartan numbering.
    pub fn cartan_representative_id(&self, cartan: CartanId) -> Option<InvolutionId> {
        self.orbit_slice(cartan).map(|(start, _)| start)
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
                &delta.coweight_matrix().to_vec(),
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
    descent_bits: &mut Vec<u32>,
    element: WeylElt,
    root_images: Vec<RootId>,
    weyl_length: usize,
    theta: LatticeInvolution,
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
    // `root_images` is the record's root action of theta = w after delta:
    // seeded from the class representative's root involution, thereafter
    // transported along the cross edge by the caller. Composition at the
    // permutation level (`w_perm[delta_perm[r]]`) equals the composed matrix
    // action, so classification needs no per-root matrix work.
    let delta_images = inner_class.distinguished_involution().image_permutation();
    if compact_weyl.length(&element) != weyl_length
        || !compact_matches_images(
            compact_weyl,
            reflections,
            inner_class.root_system(),
            &element,
            |root| {
                delta_images
                    .get(root.0)
                    .and_then(|delta_image| root_images.get(delta_image.0).copied())
            },
        )?
    {
        return Err(StructureError::InvolutionTableInvariantViolation {
            invariant: "compact Weyl element",
        });
    }
    let involution =
        TwistedInvolution::record_from_theta(inner_class.root_system(), theta, root_images)?;
    let theta = involution.root_involution().involution();
    let mod_space = match transported_mod_space {
        Some(space) => space,
        None => {
            let eigenlattice = negative_coweight_eigenspace(
                &theta.coweight_matrix().to_vec(),
                &budget.integer_lattice,
            )?;
            reduce_basis_mod_two(&eigenlattice)?
        }
    };
    let coordinates = theta_plus_one_rho_coordinates(theta, two_rho)?;
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
    // The record's left-descent bits: a fixed property of the Weyl factor,
    // paid once here instead of per KGB edge.
    let mut descents = 0_u32;
    for generator in 0..compact_weyl.d_out().len() {
        let mut probe = element;
        if compact_weyl.inner_left_mult(&mut probe, generator) < 0 {
            descents |= 1_u32 << generator;
        }
    }
    descent_bits.push(descents);
    records.push(InvolutionRecord {
        element,
        involution,
        mod_space,
        theta_plus_one_rho: Weight::new(coordinates),
        involution_length: u32::try_from(involution_length)
            .map_err(|_| StructureError::ArithmeticOverflow)?,
        weyl_length: u32::try_from(weyl_length)
            .map_err(|_| StructureError::ArithmeticOverflow)?,
        projection,
    });
    Ok(id)
}

fn theta_plus_one_rho_coordinates(
    theta: &LatticeInvolution,
    two_rho: &Weight,
) -> Result<Vec<i32>, StructureError> {
    let mut coordinates = try_capacity(two_rho.as_slice().len())?;
    theta.act_on_weight_into(two_rho.as_slice(), &mut coordinates)?;
    for (plain, reflected) in two_rho.as_slice().iter().zip(coordinates.iter_mut()) {
        let sum = plain
            .checked_add(*reflected)
            .ok_or(StructureError::ArithmeticOverflow)?;
        if sum % 2 != 0 {
            return Err(StructureError::InvolutionTableInvariantViolation {
                invariant: "theta rho parity",
            });
        }
        *reflected = sum / 2;
    }
    Ok(coordinates)
}

#[cfg(test)]
mod tests {
    use crate::{
        AdjointFiberBudget, BasedRootDatum, CartanClassificationBudget, Coweight,
        LatticeInvolution, ModTwoVector, dual_inner_class, dual_involution,
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

    #[test]
    fn rank_eight_cross_link_rows_stay_inline() {
        let links = cross_link_row(8).unwrap();
        assert_eq!(links.len(), 0);
        assert_eq!(links.capacity(), 8);
    }

    #[test]
    fn theta_rho_coordinates_are_materialized_without_a_weight_temporary() {
        let datum = BasedRootDatum::standard(vec![vec![2]]).unwrap();
        let theta = LatticeInvolution::identity(&datum).unwrap();
        let coordinates = theta_plus_one_rho_coordinates(&theta, &Weight::new(vec![1])).unwrap();
        assert_eq!(coordinates, vec![1]);
    }

    #[test]
    fn records_share_the_theta_datum_arc_within_each_orbit() {
        let (inner_class, classification) = context(vec![vec![2, -1], vec![-1, 2]], None, 6, 6);
        let mut table = InvolutionTable::new(
            &inner_class,
            table_budget(classification.twisted_involution_count()),
        )
        .unwrap();

        for cartan in classification.cartan_ids() {
            let (start, size) = table.add_cartan(&classification, cartan).unwrap();
            let datum_arc = table.record(start).unwrap().theta().datum_arc().clone();
            for offset in 0..size {
                let id = InvolutionId(start.0 + offset);
                let record = table.record(id).unwrap();
                assert!(Arc::ptr_eq(&datum_arc, record.theta().datum_arc()));
                assert_eq!(record.theta().datum(), inner_class.datum());
            }
        }
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
        let fundamental_id = InvolutionId(0);
        let fundamental = table.record(fundamental_id).unwrap();
        assert!(
            table
                .materialize_weyl_element(fundamental_id)
                .unwrap()
                .is_identity()
        );
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
            let materialized = table.materialize_weyl_element(id).unwrap();
            let legacy = pieces
                .key(table.root_system(), &interface, &materialized)
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
        assert!(
            !compact_matches_images(
                &table.compact_weyl,
                &table.reflections,
                table.root_system(),
                &compact,
                |root| legacy.image(root),
            )
            .unwrap()
        );
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
            let materialized = table.materialize_weyl_element(id).unwrap();
            assert!(
                compact_matches_images(
                    &table.compact_weyl,
                    &table.reflections,
                    table.root_system(),
                    &record.compact_weyl_element(),
                    |root| materialized.image(root),
                )
                .unwrap()
            );
            assert_eq!(table.compact_index.get(&record.element), Some(&id));
            assert_eq!(&record.clone(), record);
        }
        assert_eq!(table.compact_index.len(), table.involution_count());
    }

    #[test]
    fn materialize_weyl_element_matches_compact_records_for_a2_b2_d4() {
        let cases = [
            (vec![vec![2, -1], vec![-1, 2]], 6, 6),
            (vec![vec![2, -2], vec![-1, 2]], 8, 8),
            (
                vec![
                    vec![2, -1, 0, 0],
                    vec![-1, 2, -1, -1],
                    vec![0, -1, 2, 0],
                    vec![0, -1, 0, 2],
                ],
                24,
                192,
            ),
        ];

        for (cartan, root_count, weyl_order) in cases {
            let (inner_class, classification) = context(cartan, None, root_count, weyl_order);
            let table = filled_table(
                &inner_class,
                &classification,
                classification.twisted_involution_count(),
            );
            for index in 0..table.involution_count() {
                let id = InvolutionId(index);
                let materialized = table.materialize_weyl_element(id).unwrap();
                let record = table.record(id).unwrap();
                assert_eq!(materialized.length(), record.weyl_length());
                assert!(
                    compact_matches_images(
                        &table.compact_weyl,
                        &table.reflections,
                        table.root_system(),
                        &record.compact_weyl_element(),
                        |root| materialized.image(root),
                    )
                    .unwrap()
                );
            }
        }
    }

    #[test]
    fn materialize_weyl_element_rejects_unknown_ids_before_materialization() {
        let (inner_class, classification) = context(vec![vec![2, -1], vec![-1, 2]], None, 6, 6);
        let table = filled_table(&inner_class, &classification, 6);
        assert!(matches!(
            table.materialize_weyl_element(InvolutionId(usize::MAX)),
            Err(StructureError::IndexOutOfRange {
                index: usize::MAX,
                upper_bound
            }) if upper_bound == table.involution_count()
        ));
    }

    #[test]
    fn b2_twisted_root_images_recover_stored_weyl_factor() {
        let (inner_class, classification) = context(vec![vec![2, -2], vec![-1, 2]], None, 8, 8);
        let table = filled_table(&inner_class, &classification, 8);
        for index in 0..table.involution_count() {
            let id = InvolutionId(index);
            for root in 0..table.root_system().roots().len() {
                assert_eq!(
                    table.weyl_image(id, RootId(root)),
                    table
                        .materialize_weyl_element(id)
                        .unwrap()
                        .image(RootId(root))
                );
            }
        }
    }

    #[test]
    fn cross_edge_theta_transport_reproduces_stored_root_images() {
        // Pin the cross-edge transport index order: theta' = w' after delta
        // with w' = s*w*twist(s) composes as
        // left[theta[delta[right[delta[r]]]]] — the order the retired
        // WeylElement::from_twisted_composition materialized. The stored
        // neighbor images are the production values this transport must
        // reproduce, and the materialized Weyl factor of the neighbor is an
        // independent oracle (theta' = w' after delta).
        // The twisted A2 case (Some distinguished) exercises a nontrivial
        // delta inside the composition.
        let cases = [
            (vec![vec![2, -1], vec![-1, 2]], None, 6, 6),
            (
                vec![vec![2, -1], vec![-1, 2]],
                Some(vec![vec![0, 1], vec![1, 0]]),
                6,
                6,
            ),
            (vec![vec![2, -2], vec![-1, 2]], None, 8, 8),
            (
                vec![
                    vec![2, -1, 0, 0],
                    vec![-1, 2, -1, -1],
                    vec![0, -1, 2, 0],
                    vec![0, -1, 0, 2],
                ],
                None,
                24,
                192,
            ),
        ];
        for (cartan, distinguished, root_count, weyl_order) in cases {
            let (inner_class, classification) =
                context(cartan, distinguished, root_count, weyl_order);
            let table = filled_table(
                &inner_class,
                &classification,
                classification.twisted_involution_count(),
            );
            let delta = inner_class.distinguished_involution().image_permutation();
            for index in 0..table.involution_count() {
                let id = InvolutionId(index);
                let theta = table
                    .record(id)
                    .unwrap()
                    .twisted_involution()
                    .root_involution()
                    .image_permutation();
                for generator in 0..table.twist.len() {
                    let left = table.reflections[generator].image_permutation();
                    let right = table.reflections[table.twist[generator]].image_permutation();
                    let transported: Vec<RootId> = (0..theta.len())
                        .map(|root| left[theta.at(delta.at(right[delta.at(root).0].0).0).0])
                        .collect();
                    let cross_id = table.cross(generator, id).unwrap();
                    let stored = table
                        .record(cross_id)
                        .unwrap()
                        .twisted_involution()
                        .root_involution()
                        .image_permutation();
                    assert_eq!(transported, stored.to_vec());
                    let materialized = table.materialize_weyl_element(cross_id).unwrap();
                    for (root, &image) in transported.iter().enumerate() {
                        assert_eq!(materialized.image(delta.at(root)), Some(image));
                    }
                }
            }
        }
    }

    #[test]
    fn b2_compact_queries_equal_legacy_weyl_element_queries() {
        let (inner_class, classification) = context(vec![vec![2, -2], vec![-1, 2]], None, 8, 8);
        let table = filled_table(&inner_class, &classification, 8);
        for index in 0..table.involution_count() {
            let id = InvolutionId(index);
            let legacy = table.materialize_weyl_element(id).unwrap();
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
            assert_eq!(rebuilt, legacy);
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
    fn compact_identity_lookup_matches_legacy_lookup() {
        let (inner_class, classification) = context(vec![vec![2, -2], vec![-1, 2]], None, 8, 8);
        let empty = InvolutionTable::new(&inner_class, table_budget(8)).unwrap();
        assert_eq!(empty.identity_id(), None);
        let table = filled_table(&inner_class, &classification, 8);
        let identity = WeylElement::identity(table.root_system()).unwrap();
        assert_eq!(table.identity_id(), table.lookup(&identity));
        assert_eq!(table.identity_id(), Some(InvolutionId(0)));
    }

    #[test]
    fn b2_c2_cartan_representative_ids_are_canonical_orbit_starts() {
        for cartan_matrix in [
            vec![vec![2, -2], vec![-1, 2]],
            vec![vec![2, -1], vec![-2, 2]],
        ] {
            let (inner_class, classification) = context(cartan_matrix, None, 8, 8);
            let mut table = InvolutionTable::new(&inner_class, table_budget(8)).unwrap();
            let invalid = CartanId(classification.cartan_classes().len());

            assert_eq!(table.cartan_representative_id(invalid), None);
            for cartan in classification.cartan_ids() {
                assert_eq!(table.cartan_representative_id(cartan), None);

                let (start, size) = table.add_cartan(&classification, cartan).unwrap();
                assert!(size > 0);
                assert_eq!(table.cartan_representative_id(cartan), Some(start));
                assert_eq!(
                    table
                        .orbit_slice(cartan)
                        .map(|(orbit_start, _)| orbit_start),
                    Some(start)
                );

                let representative = classification
                    .cartan_class(cartan)
                    .unwrap()
                    .representative();
                let legacy =
                    WeylElement::from_action(table.root_system(), representative.weyl_action())
                        .unwrap();
                assert_eq!(table.lookup(&legacy), Some(start));
            }

            assert_eq!(
                table.cartan_representative_id(CartanId(0)),
                table.identity_id()
            );
            assert_eq!(table.cartan_representative_id(invalid), None);
        }
    }

    #[test]
    fn compact_dual_lookup_matches_legacy_dual_involution() {
        for distinguished in [None, Some(vec![vec![0, 1], vec![1, 0]])] {
            let (inner_class, classification) =
                context(vec![vec![2, -1], vec![-1, 2]], distinguished, 6, 6);
            let table = filled_table(&inner_class, &classification, 6);
            let longest = table.compact_weyl.longest();
            let mut legacy_longest = WeylElement::identity(table.root_system()).unwrap();
            for generator in table.compact_weyl.element_word(&longest) {
                legacy_longest = legacy_longest
                    .right_multiply_simple(table.root_system(), generator)
                    .unwrap()
                    .0;
            }

            for index in 0..table.involution_count() {
                let id = InvolutionId(index);
                let word = table.weyl_word(id).unwrap();
                let expected =
                    dual_involution(&word, table.root_system(), &table.twist, &legacy_longest)
                        .unwrap();
                let expected_id =
                    (0..table.involution_count())
                        .map(InvolutionId)
                        .find(|&candidate| {
                            table.materialize_weyl_element(candidate).unwrap() == expected
                        });
                assert_eq!(
                    table.weyl_dual_lookup(&word, &table.twist).unwrap(),
                    expected_id
                );
            }

            assert!(matches!(
                table.weyl_dual_lookup(&[table.twist.len()], &table.twist),
                Err(StructureError::IndexOutOfRange { index, upper_bound })
                    if index == table.twist.len() && upper_bound == table.twist.len()
            ));
            let mut invalid_twist = table.twist.clone();
            invalid_twist[0] = table.twist.len();
            assert!(matches!(
                table.weyl_dual_lookup(&[0], &invalid_twist),
                Err(StructureError::IndexOutOfRange { index, upper_bound })
                    if index == table.twist.len() && upper_bound == table.twist.len()
            ));
        }
    }

    #[test]
    fn dual_involution_ids_match_legacy_lookup_in_the_target_table() {
        fn assert_all_source_records(
            label: &str,
            source: &InvolutionTable,
            target: &InvolutionTable,
        ) {
            let longest = target.compact_weyl.longest();
            let mut legacy_longest = WeylElement::identity(target.root_system()).unwrap();
            for generator in target.compact_weyl.element_word(&longest) {
                legacy_longest = legacy_longest
                    .right_multiply_simple(target.root_system(), generator)
                    .unwrap()
                    .0;
            }

            let mut target_numbering_differs = false;
            for index in 0..source.involution_count() {
                let source_id = InvolutionId(index);
                let source_element = source.materialize_weyl_element(source_id).unwrap();
                let source_word = source_element.reduced_word(source.root_system()).unwrap();
                let expected = dual_involution(
                    &source_word,
                    target.root_system(),
                    &target.twist,
                    &legacy_longest,
                )
                .unwrap();
                let expected_id = target
                    .lookup(&expected)
                    .expect("a full target table contains every dual involution");
                assert_eq!(
                    target.dual_involution_id(source, source_id).unwrap(),
                    Some(expected_id),
                    "{label}: source_id={source_id:?}"
                );
                target_numbering_differs |= source.lookup(&expected) != Some(expected_id);
            }
            assert!(
                target_numbering_differs,
                "{label}: target IDs must exercise target-table numbering"
            );
        }

        let (a2_inner_class, a2_classification) = context(
            vec![vec![2, -1], vec![-1, 2]],
            Some(vec![vec![0, 1], vec![1, 0]]),
            6,
            6,
        );
        let a2_source = filled_table(&a2_inner_class, &a2_classification, 6);
        let a2_dual = dual_inner_class(&a2_inner_class, 6, 6).unwrap();
        let a2_dual_classification =
            CartanClassification::build(&a2_dual, &class_budget(6)).unwrap();
        let a2_target = filled_table(&a2_dual, &a2_dual_classification, 6);
        assert_all_source_records("A2 node swap", &a2_source, &a2_target);

        let (b2_inner_class, b2_classification) =
            context(vec![vec![2, -2], vec![-1, 2]], None, 8, 8);
        let b2_source = filled_table(&b2_inner_class, &b2_classification, 8);
        let b2_dual = dual_inner_class(&b2_inner_class, 8, 8).unwrap();
        let b2_dual_classification =
            CartanClassification::build(&b2_dual, &class_budget(8)).unwrap();
        let b2_target = filled_table(&b2_dual, &b2_dual_classification, 8);
        assert_all_source_records("B2 identity twist", &b2_source, &b2_target);

        let (a1_inner_class, a1_classification) = context(vec![vec![2]], None, 2, 2);
        let incompatible_target = filled_table(&a1_inner_class, &a1_classification, 2);
        let invalid_source_id = InvolutionId(a2_source.involution_count());
        assert!(matches!(
            incompatible_target.dual_involution_id(&a2_source, invalid_source_id),
            Err(StructureError::IndexOutOfRange { index, upper_bound })
                if index == invalid_source_id.0 && upper_bound == a2_source.involution_count()
        ));
    }

    #[test]
    fn b2_compact_left_word_lookup_matches_legacy_products() {
        let (inner_class, classification) = context(vec![vec![2, -2], vec![-1, 2]], None, 8, 8);
        let table = filled_table(&inner_class, &classification, 8);
        let words = [
            vec![],
            vec![0],
            vec![1],
            vec![0, 1, 0],
            vec![1, 0, 1],
            vec![0, 1, 0, 1],
            vec![1, 0, 1, 0],
        ];

        for index in 0..table.involution_count() {
            let id = InvolutionId(index);
            for word in &words {
                let mut expected = table.materialize_weyl_element(id).unwrap();
                for &generator in word.iter().rev() {
                    let reflection =
                        WeylElement::simple_reflection(table.root_system(), generator).unwrap();
                    expected = reflection.multiply(table.root_system(), &expected).unwrap();
                }
                assert_eq!(
                    table.weyl_left_word_lookup(id, word).unwrap(),
                    table.lookup(&expected),
                    "id={id:?}, word={word:?}"
                );
            }
        }

        assert!(matches!(
            table.weyl_left_word_lookup(InvolutionId(usize::MAX), &[table.twist.len()]),
            Err(StructureError::IndexOutOfRange { index, upper_bound })
                if index == usize::MAX && upper_bound == table.involution_count()
        ));
        assert!(matches!(
            table.weyl_left_word_lookup(InvolutionId(0), &[table.twist.len()]),
            Err(StructureError::IndexOutOfRange { index, upper_bound })
                if index == table.twist.len() && upper_bound == table.twist.len()
        ));
    }

    #[test]
    fn b2_compact_cayley_matches_legacy_for_all_records() {
        let (inner_class, classification) = context(vec![vec![2, -2], vec![-1, 2]], None, 8, 8);
        let mut partial = InvolutionTable::new(&inner_class, table_budget(8)).unwrap();
        partial.add_cartan(&classification, CartanId(0)).unwrap();
        let partial_id = InvolutionId(0);
        let partial_element = partial.materialize_weyl_element(partial_id).unwrap();
        let partial_product = partial.reflections[0]
            .multiply(partial.root_system(), &partial_element)
            .unwrap();
        assert_eq!(
            partial.compact_cayley_lookup(0, partial_id).unwrap(),
            partial.lookup(&partial_product),
            "unadded Cayley targets must remain None"
        );

        let table = filled_table(&inner_class, &classification, 8);

        for index in 0..table.involution_count() {
            let id = InvolutionId(index);
            for generator in 0..table.twist.len() {
                let reflection = &table.reflections[generator];
                let record_element = table.materialize_weyl_element(id).unwrap();
                let product = reflection
                    .multiply(table.root_system(), &record_element)
                    .unwrap();
                assert_eq!(
                    table.compact_cayley_lookup(generator, id).unwrap(),
                    table.lookup(&product),
                    "id={id:?}, generator={generator}"
                );
            }
        }

        assert!(matches!(
            table.compact_cayley_lookup(InvolutionId(usize::MAX).0, InvolutionId(usize::MAX)),
            Err(StructureError::IndexOutOfRange { index, upper_bound })
                if index == usize::MAX && upper_bound == table.involution_count()
        ));
        assert!(matches!(
            table.compact_cayley_lookup(table.twist.len(), InvolutionId(0)),
            Err(StructureError::IndexOutOfRange { index, upper_bound })
                if index == table.twist.len() && upper_bound == table.twist.len()
        ));
    }

    #[test]
    fn bfs_compact_neighbors_match_the_legacy_word_products() {
        let (inner_class, classification) = context(vec![vec![2, -2], vec![-1, 2]], None, 8, 8);
        let mut table = InvolutionTable::new(&inner_class, table_budget(8)).unwrap();
        for cartan in 0..classification.cartan_classes().len() {
            table.add_cartan(&classification, CartanId(cartan)).unwrap();
        }

        for index in 0..table.involution_count() {
            let record = table.record(InvolutionId(index)).unwrap();
            for generator in 0..table.twist.len() {
                let neighbor = compact_cross_neighbor(
                    &table.compact_weyl,
                    record.element,
                    generator,
                    table.twist[generator],
                );
                let record_element = table.materialize_weyl_element(InvolutionId(index)).unwrap();
                let expected = table.reflections[generator]
                    .multiply(table.root_system(), &record_element)
                    .and_then(|left| {
                        left.multiply(
                            table.root_system(),
                            &table.reflections[table.twist[generator]],
                        )
                    })
                    .unwrap();
                let expected_id = table.lookup(&expected);
                assert_eq!(table.compact_index.get(&neighbor).copied(), expected_id);
                assert_eq!(
                    Some(table.cross(generator, InvolutionId(index)).unwrap()),
                    expected_id
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
            let legacy = table.materialize_weyl_element(id).unwrap();
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
            let legacy = table.materialize_weyl_element(id).unwrap();
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
    fn compact_canonical_involution_expr_matches_legacy_words() {
        let cases = [
            (vec![vec![2, -2], vec![-1, 2]], None, 8, 8),
            (
                vec![vec![2, -1], vec![-1, 2]],
                Some(vec![vec![0, 1], vec![1, 0]]),
                6,
                6,
            ),
        ];

        for (cartan, distinguished, roots, weyl) in cases {
            let (inner_class, classification) = context(cartan, distinguished, roots, weyl);
            let table = filled_table(&inner_class, &classification, weyl);
            for index in 0..table.involution_count() {
                let id = InvolutionId(index);
                let materialized = table.materialize_weyl_element(id).unwrap();
                let expected = inner_class
                    .canonical_involution_expr(&materialized)
                    .unwrap();
                assert_eq!(table.weyl_canonical_involution_expr(id).unwrap(), expected);
            }

            assert!(matches!(
                table.weyl_canonical_involution_expr(InvolutionId(usize::MAX)),
                Err(StructureError::IndexOutOfRange { index, upper_bound })
                    if index == usize::MAX && upper_bound == table.involution_count()
            ));
        }
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
            let id = InvolutionId(index);
            let record = table.record(id).unwrap();
            let materialized = table.materialize_weyl_element(id).unwrap();
            assert_eq!(record.weyl_length(), materialized.length());
            for (root, _, _) in inner_class.root_system().entries() {
                let delta_image = delta_data.image(root).unwrap();
                assert_eq!(
                    record.twisted_involution().root_involution().image(root),
                    materialized.image(delta_image)
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
        assert!(
            table
                .materialize_weyl_element(fundamental)
                .unwrap()
                .is_identity()
        );
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
        assert_eq!(record.projection().lift_mat_nested(), vec![vec![2], vec![-2]]);
        assert_eq!(record.projection().m_real_nested(), vec![vec![1, 0]]);
        // The fresh echelon build lands on the opposite sign — the value
        // the parameter layer used to compute on the spot.
        let recomputed = RealProjection::build(record.theta()).unwrap();
        assert_eq!(recomputed.lift_mat_nested(), vec![vec![-2], vec![2]]);
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
                negative_coweight_eigenspace(&record.theta().coweight_matrix().to_vec(), &budget)
                    .unwrap();
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
