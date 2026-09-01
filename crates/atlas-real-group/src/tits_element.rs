//! Torus parts and Tits-group operations (KGB stage c).
//!
//! A [`TitsElement`] is the pair `(involution, torus bits)` — upstream's
//! PERSISTED per-element shape, used here during construction too because
//! the stage-(b) [`InvolutionTable`] supplies O(1) cross links and the
//! Cayley edge, so no per-element Weyl data is ever carried. The word walks
//! of upstream `push_across`/`pull_across` are replaced by mod-2
//! matrix transport from the records' stored `WeylAction`s (the
//! sophistication upstream's own comment names, tits.cpp:425-432), and the
//! based cross action is one closed-form per-element map verified
//! step-for-step against the four sigma multiplications
//! (tits.cpp:469-503). Inverse Cayley uses the source involution's
//! mod-space to repair a compact reconstructed grading before reduction
//! at the target (tits.cpp:605-644).

use crate::grading::try_capacity;
use crate::{
    InnerClass, InvolutionId, InvolutionRecord, InvolutionTable, ModTwoVector, RootId, RootKind,
    RootSystem, StructureError, Weight, WeylAction,
};

/// One KGB-side Tits element: an involution number in one table's
/// numbering plus left-stored mod-2 torus bits.
///
/// Field order is load-bearing: `involution` first makes the derived `Ord`
/// group elements by involution. The derived relations compare RAW bits and
/// are semantically meaningful on REDUCED representatives (the normal-form
/// contract); [`TitsCoset::reduce`] produces them.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TitsElement {
    involution: InvolutionId,
    torus: ModTwoVector,
}

impl TitsElement {
    /// A gated raw constructor; it does NOT auto-reduce.
    pub fn new(
        table: &InvolutionTable,
        involution: InvolutionId,
        torus: ModTwoVector,
    ) -> Result<Self, StructureError> {
        if involution.0 >= table.involution_count() {
            return Err(StructureError::IndexOutOfRange {
                index: involution.0,
                upper_bound: table.involution_count(),
            });
        }
        let lattice_rank = table.root_system().lattice_rank();
        if torus.dimension() != lattice_rank {
            return Err(StructureError::RankMismatch {
                expected: lattice_rank,
                actual: torus.dimension(),
            });
        }
        Ok(Self { involution, torus })
    }

    pub fn involution(&self) -> InvolutionId {
        self.involution
    }

    pub fn torus_bits(&self) -> &ModTwoVector {
        &self.torus
    }
}

/// The based Tits group: upstream's `TitsGroup` mod-2 tables merged with
/// the `TitsCoset` grading offset.
///
/// Operations take `&InvolutionTable` per call (the substrate idiom);
/// provenance is FULL inner-class equality, since two inner classes over
/// one datum with different distinguished involutions would silently
/// corrupt every transport under a datum-only gate. The twist permutation
/// and simple-reflection root permutations are the coset's own derived
/// copies — a named duplication of the table's private caches.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TitsCoset {
    inner_class: InnerClass,
    grading_offset: Vec<bool>,
    twist: Vec<usize>,
    /// Simple coroots mod 2 (`X_*` side): the elements `m_alpha`.
    m_alpha: Vec<ModTwoVector>,
    /// Simple roots mod 2 (`X^*` side): the pairing vectors.
    dual_m_alpha: Vec<ModTwoVector>,
    /// Root permutations of the simple reflections, for the
    /// conjugate-to-simple grading loop.
    reflection_permutations: Vec<Vec<RootId>>,
}

impl TitsCoset {
    /// Build the tables once from the inner class; the grading offset is
    /// the caller's elected input (stage (d) derives it from the
    /// square-class cocharacter; the adjoint convention is
    /// `offset[s] = (twist(s) == s)`).
    pub fn new(
        inner_class: &InnerClass,
        grading_offset: Vec<bool>,
    ) -> Result<Self, StructureError> {
        let datum = inner_class.datum();
        let system = inner_class.root_system();
        let delta_data = inner_class.distinguished_involution();
        let semisimple_rank = datum.semisimple_rank();
        if grading_offset.len() != semisimple_rank {
            return Err(StructureError::RankMismatch {
                expected: semisimple_rank,
                actual: grading_offset.len(),
            });
        }
        let simple_ids = system.simple_root_ids();
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
        // A strong involution forces the offset to be twist-invariant
        // (tits.h:664-667); which square class it realizes stays the
        // caller's contract.
        for (generator, &target) in twist.iter().enumerate() {
            if grading_offset[generator] != grading_offset[target] {
                return Err(StructureError::TitsCosetInvariantViolation {
                    invariant: "offset twist invariance",
                });
            }
        }
        let mut m_alpha = try_capacity(semisimple_rank)?;
        for coroot in datum.simple_coroots() {
            m_alpha.push(parity_vector(coroot.as_slice())?);
        }
        let mut dual_m_alpha = try_capacity(semisimple_rank)?;
        for root in datum.simple_roots() {
            dual_m_alpha.push(parity_vector(root.as_slice())?);
        }
        let mut reflection_permutations = try_capacity(semisimple_rank)?;
        for generator in 0..semisimple_rank {
            let action = WeylAction::simple_reflection(datum, generator)?;
            reflection_permutations.push(system.action_permutation(&action)?);
        }
        Ok(Self {
            inner_class: inner_class.clone(),
            grading_offset,
            twist,
            m_alpha,
            dual_m_alpha,
            reflection_permutations,
        })
    }

    pub fn grading_offset(&self) -> &[bool] {
        &self.grading_offset
    }

    /// Noncompactness of `alpha_s` at this element:
    /// `offset[s] XOR <alpha_s mod 2, torus bits>` (true = noncompact,
    /// matching `Grading::is_noncompact`). The formula is evaluated
    /// blindly, as upstream does — it is meaningful only when `s` is
    /// IMAGINARY for the element's involution; callers guard via
    /// `InvolutionTable::simple_root_kind`.
    pub fn simple_grading(
        &self,
        table: &InvolutionTable,
        element: &TitsElement,
        generator: usize,
    ) -> Result<bool, StructureError> {
        self.gate(table)?;
        self.simple_grading_pregated(element, generator)
    }

    /// [`Self::simple_grading`] without the per-call inner-class gate, for
    /// callers that gated ONCE (the stage-(e) BFS issues millions of calls;
    /// the deep equality per call would dominate the whole build).
    pub(crate) fn simple_grading_pregated(
        &self,
        element: &TitsElement,
        generator: usize,
    ) -> Result<bool, StructureError> {
        self.check_generator(generator)?;
        Ok(self.grading_offset[generator] != self.dual_m_alpha[generator].dot(&element.torus)?)
    }

    /// The based cross action, reduced at the target: the closed-form map
    /// equal to `sigma_mult(s, .)` then `mult_sigma_inv(., twist(s))` plus
    /// the offset correction. The second correction pulls across the
    /// TARGET Weyl part — the matrix of `w'` as a group element.
    pub fn cross(
        &self,
        table: &InvolutionTable,
        generator: usize,
        element: &TitsElement,
    ) -> Result<TitsElement, StructureError> {
        self.gate(table)?;
        self.cross_pregated(table, generator, element)
    }

    /// [`Self::cross`] without the per-call inner-class gate (see
    /// [`Self::simple_grading_pregated`]).
    pub(crate) fn cross_pregated(
        &self,
        table: &InvolutionTable,
        generator: usize,
        element: &TitsElement,
    ) -> Result<TitsElement, StructureError> {
        self.check_generator(generator)?;
        let record = required_record(table, element.involution)?;
        let target_id = table.cross(generator, element.involution)?;
        let target = required_record(table, target_id)?;
        let mut bits = element.torus.clone();
        if self.dual_m_alpha[generator].dot(&bits)? {
            bits.xor_assign(&self.m_alpha[generator])?;
        }
        let left_descent = table.weyl_has_left_descent(element.involution, generator)?;
        if left_descent {
            bits.xor_assign(&self.m_alpha[generator])?;
        }
        let s_w_length = if left_descent {
            record
                .weyl_length()
                .checked_sub(1)
                .ok_or(StructureError::ArithmeticOverflow)?
        } else {
            record
                .weyl_length()
                .checked_add(1)
                .ok_or(StructureError::ArithmeticOverflow)?
        };
        if target.weyl_length() > s_w_length {
            let pulled = apply_matrix_mod_two(
                target.twisted_involution().weyl_action().coweight_matrix(),
                &self.m_alpha[self.twist[generator]],
            )?;
            bits.xor_assign(&pulled)?;
        }
        if self.grading_offset[generator] {
            bits.xor_assign(&self.m_alpha[generator])?;
        }
        let reduced = target.mod_space().quotient_representative(bits)?;
        Ok(TitsElement {
            involution: target_id,
            torus: reduced,
        })
    }

    /// The Cayley transform (a bare `sigma_mult`), reduced at the target's
    /// grown mod-space; `None` while the target Cartan class has not been
    /// added to the table.
    pub fn cayley(
        &self,
        table: &InvolutionTable,
        generator: usize,
        element: &TitsElement,
    ) -> Result<Option<TitsElement>, StructureError> {
        self.gate(table)?;
        self.cayley_pregated(table, generator, element)
    }

    /// [`Self::cayley`] without the per-call inner-class gate (see
    /// [`Self::simple_grading_pregated`]).
    pub(crate) fn cayley_pregated(
        &self,
        table: &InvolutionTable,
        generator: usize,
        element: &TitsElement,
    ) -> Result<Option<TitsElement>, StructureError> {
        self.check_generator(generator)?;
        let _record = required_record(table, element.involution)?;
        let Some(target_id) = table.cayley(generator, element.involution)? else {
            return Ok(None);
        };
        let target = required_record(table, target_id)?;
        let mut bits = element.torus.clone();
        if self.dual_m_alpha[generator].dot(&bits)? {
            bits.xor_assign(&self.m_alpha[generator])?;
        }
        if table.weyl_has_left_descent(element.involution, generator)? {
            bits.xor_assign(&self.m_alpha[generator])?;
        }
        let reduced = target.mod_space().quotient_representative(bits)?;
        Ok(Some(TitsElement {
            involution: target_id,
            torus: reduced,
        }))
    }

    /// Invert a Cayley transform through a real simple root.
    ///
    /// The inverse first applies the bare `sigma_inv_mult`. Modular
    /// reduction at the real source can have forgotten the source-side
    /// grading, so a compact reconstructed root is repaired by the first
    /// source mod-space basis vector that pairs nontrivially with it. The
    /// result is then reduced at the imaginary target. `None` means either
    /// that the root is not real or that the downward Cartan class has not
    /// been added to the table.
    pub fn inverse_cayley(
        &self,
        table: &InvolutionTable,
        generator: usize,
        element: &TitsElement,
    ) -> Result<Option<TitsElement>, StructureError> {
        self.gate(table)?;
        self.check_generator(generator)?;
        let source = required_record(table, element.involution)?;
        if table.simple_root_kind(element.involution, generator) != Some(RootKind::Real) {
            return Ok(None);
        }
        let Some(target_id) = table.cayley(generator, element.involution)? else {
            return Ok(None);
        };
        let target = required_record(table, target_id)?;

        // Upstream `sigma_inv_mult`: reflect the left torus part, then add
        // m_alpha precisely when left multiplication increases Weyl length.
        let mut bits = element.torus.clone();
        if self.dual_m_alpha[generator].dot(&bits)? {
            bits.xor_assign(&self.m_alpha[generator])?;
        }
        if !table.weyl_has_left_descent(element.involution, generator)? {
            bits.xor_assign(&self.m_alpha[generator])?;
        }

        if self.grading_offset[generator] == self.dual_m_alpha[generator].dot(&bits)? {
            let mut repaired = false;
            for basis_vector in source.mod_space().basis_vectors() {
                if self.dual_m_alpha[generator].dot(basis_vector)? {
                    bits.xor_assign(basis_vector)?;
                    repaired = true;
                    break;
                }
            }
            if !repaired {
                return Err(StructureError::TitsCosetInvariantViolation {
                    invariant: "inverse Cayley grading repair",
                });
            }
        }

        let result = TitsElement {
            involution: target_id,
            torus: target.mod_space().quotient_representative(bits)?,
        };
        if !self.simple_grading_pregated(&result, generator)? {
            return Err(StructureError::TitsCosetInvariantViolation {
                invariant: "inverse Cayley grading repair",
            });
        }
        Ok(Some(result))
    }

    /// Noncompactness of an arbitrary IMAGINARY root, by upstream's
    /// conjugate-to-simple loop over based cross actions; `None` when the
    /// root is not imaginary at the element's involution.
    pub fn grading(
        &self,
        table: &InvolutionTable,
        element: &TitsElement,
        root: RootId,
    ) -> Result<Option<bool>, StructureError> {
        self.gate(table)?;
        let record = required_record(table, element.involution)?;
        let system = table.root_system();
        let kind = record
            .twisted_involution()
            .root_involution()
            .kind(root)
            .ok_or(StructureError::IndexOutOfRange {
                index: root.0,
                upper_bound: system.roots().len(),
            })?;
        if kind != RootKind::Imaginary {
            return Ok(None);
        }
        let mut alpha = if system.is_positive(root).unwrap_or(false) {
            root
        } else {
            negated_root(system, root)?
        };
        let mut current = element.clone();
        let simple_ids = system.simple_root_ids();
        let mut steps = 0_usize;
        loop {
            if let Some(generator) = simple_ids.iter().position(|&id| id == alpha) {
                return Ok(Some(self.simple_grading(table, &current, generator)?));
            }
            if steps == system.roots().len() {
                return Err(StructureError::TitsCosetInvariantViolation {
                    invariant: "grading conjugation",
                });
            }
            steps += 1;
            let generator = (0..simple_ids.len())
                .find(|&candidate| {
                    system
                        .bracket(alpha, simple_ids[candidate])
                        .map(|pairing| pairing > 0)
                        .unwrap_or(false)
                })
                .ok_or(StructureError::TitsCosetInvariantViolation {
                    invariant: "grading conjugation",
                })?;
            current = self.cross(table, generator, &current)?;
            alpha = self.reflection_permutations[generator][alpha.0];
        }
    }

    /// The table's normal form for this element's torus bits; idempotent.
    pub fn reduce(
        &self,
        table: &InvolutionTable,
        element: &TitsElement,
    ) -> Result<TitsElement, StructureError> {
        self.gate(table)?;
        let record = required_record(table, element.involution)?;
        Ok(TitsElement {
            involution: element.involution,
            torus: record
                .mod_space()
                .quotient_representative(element.torus.clone())?,
        })
    }

    /// Full inner-class equality, not datum equality alone: a same-datum
    /// inner class with a different delta has different twists and
    /// transports.
    fn gate(&self, table: &InvolutionTable) -> Result<(), StructureError> {
        if table.inner_class() != &self.inner_class {
            return Err(StructureError::DatumMismatch);
        }
        Ok(())
    }

    fn check_generator(&self, generator: usize) -> Result<(), StructureError> {
        if generator >= self.twist.len() {
            return Err(StructureError::IndexOutOfRange {
                index: generator,
                upper_bound: self.twist.len(),
            });
        }
        Ok(())
    }
}

fn required_record(
    table: &InvolutionTable,
    id: InvolutionId,
) -> Result<&InvolutionRecord, StructureError> {
    table.record(id).ok_or(StructureError::IndexOutOfRange {
        index: id.0,
        upper_bound: table.involution_count(),
    })
}

/// The mod-2 reduction of an integer vector: bits at the odd coordinates.
pub(crate) fn parity_vector(coordinates: &[i32]) -> Result<ModTwoVector, StructureError> {
    let mut ones = try_capacity(coordinates.len())?;
    for (index, &value) in coordinates.iter().enumerate() {
        if value % 2 != 0 {
            ones.push(index);
        }
    }
    ModTwoVector::from_ones(coordinates.len(), ones)
}

/// Apply an integer matrix mod 2: output bit `i` is the parity of
/// `matrix[i][j]` over the set bits `j` of the input. This is the
/// production carrier for `pull_across` as matrix transport
/// (`ModTwoLinearMap` is deliberately test-only), shared with the KGB
/// twist (kgb_graph.rs).
pub(crate) fn apply_matrix_mod_two(
    matrix: &[Vec<i32>],
    vector: &ModTwoVector,
) -> Result<ModTwoVector, StructureError> {
    let dimension = matrix.len();
    if vector.dimension() != dimension {
        return Err(StructureError::RankMismatch {
            expected: dimension,
            actual: vector.dimension(),
        });
    }
    let mut ones = try_capacity(dimension)?;
    for (row_index, row) in matrix.iter().enumerate() {
        if row.len() != dimension {
            return Err(StructureError::RankMismatch {
                expected: dimension,
                actual: row.len(),
            });
        }
        let mut parity = false;
        for (column, &entry) in row.iter().enumerate() {
            if entry % 2 != 0 && vector.bit(column) == Some(true) {
                parity = !parity;
            }
        }
        if parity {
            ones.push(row_index);
        }
    }
    ModTwoVector::from_ones(dimension, ones)
}

fn negated_root(system: &RootSystem, root: RootId) -> Result<RootId, StructureError> {
    let weight = system.root(root).ok_or(StructureError::IndexOutOfRange {
        index: root.0,
        upper_bound: system.roots().len(),
    })?;
    let mut coordinates = try_capacity(weight.as_slice().len())?;
    for &value in weight.as_slice() {
        coordinates.push(
            value
                .checked_neg()
                .ok_or(StructureError::ArithmeticOverflow)?,
        );
    }
    system
        .id_of(&Weight::new(coordinates))
        .ok_or(StructureError::TitsCosetInvariantViolation {
            invariant: "root negation",
        })
}

#[cfg(test)]
mod tests {
    use crate::{
        AdjointFiberBudget, BasedRootDatum, CartanClassification, CartanClassificationBudget,
        CartanId, Coweight, IntegerLatticeBudget, InvolutionTableBudget, LatticeInvolution,
        WeylElement,
    };

    use super::*;

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

    fn filled_table(
        inner_class: &InnerClass,
        weyl: usize,
        max_involutions: usize,
    ) -> InvolutionTable {
        let classification = CartanClassification::build(inner_class, &class_budget(weyl)).unwrap();
        let mut table = InvolutionTable::new(
            inner_class,
            InvolutionTableBudget::new(
                max_involutions,
                IntegerLatticeBudget::new(64, 100_000, 100_000, 128),
            ),
        )
        .unwrap();
        for id in 0..classification.cartan_classes().len() {
            table.add_cartan(&classification, CartanId(id)).unwrap();
        }
        table
    }

    /// Simply connected A1: SL(2) realizations.
    fn sl2_inner_class() -> InnerClass {
        let datum = BasedRootDatum::from_simple_data(
            1,
            vec![vec![2]],
            vec![Weight::new(vec![2])],
            vec![Coweight::new(vec![1])],
        )
        .unwrap();
        let distinguished = LatticeInvolution::identity(&datum).unwrap();
        InnerClass::new(datum, distinguished, 2).unwrap()
    }

    /// Adjoint A1: PGL(2) realizations.
    fn pgl2_inner_class() -> InnerClass {
        let datum = BasedRootDatum::standard(vec![vec![2]]).unwrap();
        let distinguished = LatticeInvolution::identity(&datum).unwrap();
        InnerClass::new(datum, distinguished, 2).unwrap()
    }

    fn a2_inner_class() -> InnerClass {
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let distinguished = LatticeInvolution::identity(&datum).unwrap();
        InnerClass::new(datum, distinguished, 6).unwrap()
    }

    fn b2_inner_class() -> InnerClass {
        let datum = BasedRootDatum::standard(vec![vec![2, -2], vec![-1, 2]]).unwrap();
        let distinguished = LatticeInvolution::identity(&datum).unwrap();
        InnerClass::new(datum, distinguished, 8).unwrap()
    }

    fn bits(dimension: usize, ones: Vec<usize>) -> ModTwoVector {
        ModTwoVector::from_ones(dimension, ones).unwrap()
    }

    fn fundamental_id(table: &InvolutionTable) -> InvolutionId {
        (0..table.involution_count())
            .map(InvolutionId)
            .find(|&id| table.weyl_is_identity(id).unwrap())
            .unwrap()
    }

    fn reduced_torus_representatives(
        table: &InvolutionTable,
        coset: &TitsCoset,
        involution: InvolutionId,
    ) -> Vec<TitsElement> {
        let dimension = table.root_system().lattice_rank();
        let pattern_count = 1_usize.checked_shl(dimension as u32).unwrap();
        let mut representatives = Vec::new();
        for pattern in 0..pattern_count {
            let ones = (0..dimension)
                .filter(|&coordinate| pattern & (1 << coordinate) != 0)
                .collect();
            let raw = TitsElement::new(table, involution, bits(dimension, ones)).unwrap();
            let reduced = coset.reduce(table, &raw).unwrap();
            if !representatives.contains(&reduced) {
                representatives.push(reduced);
            }
        }
        representatives
    }

    fn assert_all_real_edges_inverse_round_trip(
        inner_class: &InnerClass,
        weyl: usize,
        max_involutions: usize,
        require_nontrivial_target: bool,
    ) {
        let table = filled_table(inner_class, weyl, max_involutions);
        let rank = inner_class.datum().semisimple_rank();
        let coset = TitsCoset::new(inner_class, vec![true; rank]).unwrap();
        let mut real_edges = 0_usize;
        let mut inverse_cases = 0_usize;
        let mut nontrivial_target_edges = 0_usize;

        for index in 0..table.involution_count() {
            let real_involution = InvolutionId(index);
            for generator in 0..rank {
                if table.simple_root_kind(real_involution, generator) != Some(RootKind::Real) {
                    continue;
                }
                real_edges += 1;
                let imaginary_involution =
                    table.cayley(generator, real_involution).unwrap().unwrap();
                assert_eq!(
                    table.simple_root_kind(imaginary_involution, generator),
                    Some(RootKind::Imaginary)
                );
                if table
                    .record(imaginary_involution)
                    .unwrap()
                    .mod_space()
                    .rank()
                    > 0
                {
                    nontrivial_target_edges += 1;
                }

                // Inverse Cayley is defined on valid strong involutions. Build
                // its exhaustive reduced source set as the forward images of
                // every noncompact representative at the imaginary endpoint.
                let mut real_sources = Vec::new();
                for preimage in reduced_torus_representatives(&table, &coset, imaginary_involution)
                {
                    if !coset.simple_grading(&table, &preimage, generator).unwrap() {
                        continue;
                    }
                    let source = coset.cayley(&table, generator, &preimage).unwrap().unwrap();
                    assert_eq!(source.involution(), real_involution);
                    if !real_sources.contains(&source) {
                        real_sources.push(source);
                    }
                }
                assert!(!real_sources.is_empty());

                for source in &real_sources {
                    assert_eq!(coset.reduce(&table, source).unwrap(), *source);
                    let inverse = coset
                        .inverse_cayley(&table, generator, source)
                        .unwrap()
                        .unwrap();
                    assert_eq!(coset.reduce(&table, &inverse).unwrap(), inverse);
                    assert_eq!(
                        table.simple_root_kind(inverse.involution(), generator),
                        Some(RootKind::Imaginary)
                    );
                    assert!(coset.simple_grading(&table, &inverse, generator).unwrap());
                    assert_eq!(
                        coset.cayley(&table, generator, &inverse).unwrap(),
                        Some(source.clone())
                    );
                    inverse_cases += 1;
                }
            }
        }

        assert!(real_edges > 0);
        assert!(inverse_cases >= real_edges);
        if require_nontrivial_target {
            assert!(nontrivial_target_edges > 0);
        }
    }

    #[test]
    fn sl2r_type_one_cross_swaps_the_fundamental_fiber_and_fixes_split() {
        let inner_class = sl2_inner_class();
        let table = filled_table(&inner_class, 2, 4);
        let coset = TitsCoset::new(&inner_class, vec![true]).unwrap();
        let fundamental = fundamental_id(&table);
        let zero = TitsElement::new(&table, fundamental, bits(1, vec![])).unwrap();
        let one = TitsElement::new(&table, fundamental, bits(1, vec![0])).unwrap();

        assert_eq!(coset.cross(&table, 0, &zero).unwrap(), one);
        assert_eq!(coset.cross(&table, 0, &one).unwrap(), zero);
        assert!(coset.simple_grading(&table, &zero, 0).unwrap());
        assert!(coset.simple_grading(&table, &one, 0).unwrap());

        let split_from_zero = coset.cayley(&table, 0, &zero).unwrap().unwrap();
        let split_from_one = coset.cayley(&table, 0, &one).unwrap().unwrap();
        assert_eq!(split_from_zero, split_from_one);
        assert_ne!(split_from_zero.involution(), fundamental);
        assert!(split_from_zero.torus_bits().is_zero());
        assert_eq!(
            coset.cross(&table, 0, &split_from_zero).unwrap(),
            split_from_zero
        );
        let states = [&zero, &one, &split_from_zero];
        for (left, right) in [(0, 1), (0, 2), (1, 2)] {
            assert_ne!(states[left], states[right]);
        }
    }

    #[test]
    fn inverse_cayley_round_trips_the_type_one_forward_image() {
        let inner_class = sl2_inner_class();
        let table = filled_table(&inner_class, 2, 4);
        let coset = TitsCoset::new(&inner_class, vec![true]).unwrap();
        let fundamental = fundamental_id(&table);
        let zero = TitsElement::new(&table, fundamental, bits(1, vec![])).unwrap();
        let one = TitsElement::new(&table, fundamental, bits(1, vec![0])).unwrap();
        let split = coset.cayley(&table, 0, &zero).unwrap().unwrap();
        assert_eq!(coset.cayley(&table, 0, &one).unwrap(), Some(split.clone()));

        let restored = coset.inverse_cayley(&table, 0, &split).unwrap().unwrap();
        assert_eq!(restored, zero);
        assert!(coset.simple_grading(&table, &restored, 0).unwrap());
        assert_eq!(coset.cayley(&table, 0, &restored).unwrap(), Some(split));
    }

    #[test]
    fn compact_su2_grades_everything_compact() {
        let inner_class = sl2_inner_class();
        let table = filled_table(&inner_class, 2, 4);
        let coset = TitsCoset::new(&inner_class, vec![false]).unwrap();
        let fundamental = fundamental_id(&table);
        let zero = TitsElement::new(&table, fundamental, bits(1, vec![])).unwrap();
        assert!(!coset.simple_grading(&table, &zero, 0).unwrap());
        assert_eq!(
            coset.grading(&table, &zero, table.root_system().simple_root_ids()[0]),
            Ok(Some(false))
        );
    }

    #[test]
    fn pgl2r_type_two_cross_fixes_the_fundamental_fiber() {
        let inner_class = pgl2_inner_class();
        let table = filled_table(&inner_class, 2, 4);
        let coset = TitsCoset::new(&inner_class, vec![true]).unwrap();
        let fundamental = fundamental_id(&table);
        for pattern in [vec![], vec![0]] {
            let element = TitsElement::new(&table, fundamental, bits(1, pattern)).unwrap();
            assert_eq!(coset.cross(&table, 0, &element).unwrap(), element);
        }
    }

    #[test]
    fn inverse_cayley_repairs_a_compact_type_two_lift() {
        let inner_class = pgl2_inner_class();
        let table = filled_table(&inner_class, 2, 4);
        let coset = TitsCoset::new(&inner_class, vec![false]).unwrap();
        let fundamental = fundamental_id(&table);
        let compact = TitsElement::new(&table, fundamental, bits(1, vec![])).unwrap();
        let noncompact = TitsElement::new(&table, fundamental, bits(1, vec![0])).unwrap();
        assert!(!coset.simple_grading(&table, &compact, 0).unwrap());
        assert!(coset.simple_grading(&table, &noncompact, 0).unwrap());

        let split = coset.cayley(&table, 0, &noncompact).unwrap().unwrap();
        let restored = coset.inverse_cayley(&table, 0, &split).unwrap().unwrap();
        assert_eq!(restored, noncompact);
        assert_eq!(coset.cayley(&table, 0, &restored).unwrap(), Some(split));
    }

    #[test]
    fn inverse_cayley_declines_non_real_roots_and_rejects_bad_generators() {
        let inner_class = sl2_inner_class();
        let table = filled_table(&inner_class, 2, 4);
        let coset = TitsCoset::new(&inner_class, vec![true]).unwrap();
        let fundamental = fundamental_id(&table);
        let imaginary = TitsElement::new(&table, fundamental, bits(1, vec![])).unwrap();

        assert_eq!(coset.inverse_cayley(&table, 0, &imaginary), Ok(None));
        assert_eq!(
            coset.inverse_cayley(&table, 3, &imaginary),
            Err(StructureError::IndexOutOfRange {
                index: 3,
                upper_bound: 1,
            })
        );
    }

    #[test]
    fn inverse_cayley_round_trips_every_a2_real_edge_and_reduced_torus_part() {
        assert_all_real_edges_inverse_round_trip(&a2_inner_class(), 6, 8, false);
    }

    #[test]
    fn inverse_cayley_round_trips_every_b2_real_edge_and_reduced_torus_part() {
        assert_all_real_edges_inverse_round_trip(&b2_inner_class(), 8, 8, true);
    }

    #[test]
    fn inverse_cayley_returns_none_when_the_downward_cartan_is_not_loaded() {
        let inner_class = sl2_inner_class();
        let classification = CartanClassification::build(&inner_class, &class_budget(2)).unwrap();
        let coset = TitsCoset::new(&inner_class, vec![true]).unwrap();
        let mut exercised = false;

        'cartans: for cartan in 0..classification.cartan_classes().len() {
            let mut table = InvolutionTable::new(
                &inner_class,
                InvolutionTableBudget::new(4, IntegerLatticeBudget::new(64, 100_000, 100_000, 128)),
            )
            .unwrap();
            table.add_cartan(&classification, CartanId(cartan)).unwrap();
            for index in 0..table.involution_count() {
                let involution = InvolutionId(index);
                if table.simple_root_kind(involution, 0) == Some(RootKind::Real)
                    && table.cayley(0, involution).unwrap().is_none()
                {
                    let source = TitsElement::new(&table, involution, bits(1, vec![])).unwrap();
                    assert_eq!(coset.inverse_cayley(&table, 0, &source), Ok(None));
                    exercised = true;
                    break 'cartans;
                }
            }
        }

        assert!(exercised);
    }

    /// Upstream's exact four-step sigma recipe at the word level, used as
    /// the definitional oracle for the closed-form cross action.
    fn recipe_cross(
        system: &RootSystem,
        coset: &TitsCoset,
        w: &WeylElement,
        torus: &ModTwoVector,
        generator: usize,
    ) -> (WeylElement, ModTwoVector) {
        let mut t = torus.clone();
        // sigma_mult(s, a): reflect, left-multiply, m_s on decrease.
        if coset.dual_m_alpha[generator].dot(&t).unwrap() {
            t.xor_assign(&coset.m_alpha[generator]).unwrap();
        }
        let (after_left, left_change) = w.left_multiply_simple(system, generator).unwrap();
        if left_change < 0 {
            t.xor_assign(&coset.m_alpha[generator]).unwrap();
        }
        // mult_sigma_inv(a, twist(s)): right-multiply FIRST, then on
        // increase right_add = pull_across the updated Weyl part.
        let twisted = coset.twist[generator];
        let (after_right, right_change) =
            after_left.right_multiply_simple(system, twisted).unwrap();
        if right_change > 0 {
            let mut pulled = coset.m_alpha[twisted].clone();
            let word = after_right.reduced_word(system).unwrap();
            for &letter in word.iter().rev() {
                if coset.dual_m_alpha[letter].dot(&pulled).unwrap() {
                    pulled.xor_assign(&coset.m_alpha[letter]).unwrap();
                }
            }
            t.xor_assign(&pulled).unwrap();
        }
        // Offset correction.
        if coset.grading_offset[generator] {
            t.xor_assign(&coset.m_alpha[generator]).unwrap();
        }
        (after_right, t)
    }

    #[test]
    fn sp4r_closed_form_cross_matches_the_word_level_sigma_recipe() {
        let datum = BasedRootDatum::standard(vec![vec![2, -2], vec![-1, 2]]).unwrap();
        let inner_class = InnerClass::new(
            datum.clone(),
            LatticeInvolution::identity(&datum).unwrap(),
            8,
        )
        .unwrap();
        let table = filled_table(&inner_class, 8, 8);
        let coset = TitsCoset::new(&inner_class, vec![true, true]).unwrap();
        let system = table.root_system();
        assert_eq!(table.involution_count(), 6);
        for index in 0..table.involution_count() {
            let id = InvolutionId(index);
            let source_element = table.materialize_weyl_element(id).unwrap();
            for pattern in [vec![], vec![0], vec![1], vec![0, 1]] {
                let element = TitsElement::new(&table, id, bits(2, pattern)).unwrap();
                for generator in 0..2 {
                    let (recipe_w, recipe_bits) = recipe_cross(
                        system,
                        &coset,
                        &source_element,
                        element.torus_bits(),
                        generator,
                    );
                    let target_id = table.lookup(&recipe_w).unwrap();
                    assert_eq!(table.cross(generator, id).unwrap(), target_id);

                    // The closed form, replayed pre-reduction: reflect,
                    // descent correction, target pull, offset.
                    let mut closed = element.torus_bits().clone();
                    if coset.dual_m_alpha[generator].dot(&closed).unwrap() {
                        closed.xor_assign(&coset.m_alpha[generator]).unwrap();
                    }
                    let left_descent = source_element.has_left_descent(system, generator).unwrap();
                    if left_descent {
                        closed.xor_assign(&coset.m_alpha[generator]).unwrap();
                    }
                    let source_length = source_element.length();
                    let s_w_length = if left_descent {
                        source_length - 1
                    } else {
                        source_length + 1
                    };
                    let target_record = table.record(target_id).unwrap();
                    if target_record.weyl_length() > s_w_length {
                        let pulled = apply_matrix_mod_two(
                            target_record
                                .twisted_involution()
                                .weyl_action()
                                .coweight_matrix(),
                            &coset.m_alpha[coset.twist[generator]],
                        )
                        .unwrap();
                        closed.xor_assign(&pulled).unwrap();
                    }
                    if coset.grading_offset[generator] {
                        closed.xor_assign(&coset.m_alpha[generator]).unwrap();
                    }
                    // Bit-for-bit BEFORE reduction.
                    assert_eq!(closed, recipe_bits);

                    // And through the public operation, after reduction.
                    let expected = target_record
                        .mod_space()
                        .quotient_representative(recipe_bits.clone())
                        .unwrap();
                    let actual = coset.cross(&table, generator, &element).unwrap();
                    assert_eq!(actual.involution(), target_id);
                    assert_eq!(actual.torus_bits(), &expected);
                }
            }
        }
        // Degenerate check at the fundamental involution (all roots
        // imaginary): the general grading of a SIMPLE root equals the
        // simple_grading formula.
        let fundamental = fundamental_id(&table);
        let element = TitsElement::new(&table, fundamental, bits(2, vec![1])).unwrap();
        for generator in 0..2 {
            assert_eq!(
                coset
                    .grading(&table, &element, system.simple_root_ids()[generator])
                    .unwrap(),
                Some(coset.simple_grading(&table, &element, generator).unwrap())
            );
        }
    }

    #[test]
    fn su21_grading_conjugates_the_nonsimple_imaginary_root_to_simple() {
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let flip = LatticeInvolution::new(
            &datum,
            vec![vec![0, 1], vec![1, 0]],
            vec![vec![0, 1], vec![1, 0]],
        )
        .unwrap();
        let inner_class = InnerClass::new(datum, flip, 6).unwrap();
        let table = filled_table(&inner_class, 6, 4);
        let coset = TitsCoset::new(&inner_class, vec![false, false]).unwrap();
        let system = table.root_system();
        let fundamental = fundamental_id(&table);
        let element = TitsElement::new(&table, fundamental, bits(2, vec![])).unwrap();

        // The simple roots are COMPLEX at the fundamental involution of
        // the flipped class: the general grading declines them.
        assert_eq!(
            coset
                .grading(&table, &element, system.simple_root_ids()[0])
                .unwrap(),
            None
        );

        // The simple-imaginary root alpha_1 + alpha_2 is NON-simple, so
        // the loop performs a genuine conjugation step; the result equals
        // the loop unrolled by hand, and the negative root agrees.
        let highest = system.id_of(&Weight::new(vec![1, 1])).unwrap();
        let graded = coset.grading(&table, &element, highest).unwrap();
        let crossed = coset.cross(&table, 0, &element).unwrap();
        let unrolled = coset.simple_grading(&table, &crossed, 1).unwrap();
        assert_eq!(graded, Some(unrolled));
        let negative = system.id_of(&Weight::new(vec![-1, -1])).unwrap();
        assert_eq!(coset.grading(&table, &element, negative).unwrap(), graded);
    }

    #[test]
    fn offset_and_provenance_gates_hold() {
        let sl2 = sl2_inner_class();
        assert_eq!(
            TitsCoset::new(&sl2, vec![true, false]),
            Err(StructureError::RankMismatch {
                expected: 1,
                actual: 2,
            })
        );

        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let flip = LatticeInvolution::new(
            &datum,
            vec![vec![0, 1], vec![1, 0]],
            vec![vec![0, 1], vec![1, 0]],
        )
        .unwrap();
        let flip_class = InnerClass::new(datum.clone(), flip, 6).unwrap();
        assert_eq!(
            TitsCoset::new(&flip_class, vec![true, false]),
            Err(StructureError::TitsCosetInvariantViolation {
                invariant: "offset twist invariance",
            })
        );

        // Same datum, different delta: rejected by full inner-class
        // equality, which a datum-only gate would miss.
        let identity_class = InnerClass::new(
            datum.clone(),
            LatticeInvolution::identity(&datum).unwrap(),
            6,
        )
        .unwrap();
        let identity_table = filled_table(&identity_class, 6, 8);
        let flip_coset = TitsCoset::new(&flip_class, vec![false, false]).unwrap();
        let foreign_fundamental = fundamental_id(&identity_table);
        let foreign_element =
            TitsElement::new(&identity_table, foreign_fundamental, bits(2, vec![])).unwrap();
        assert_eq!(
            flip_coset.cross(&identity_table, 0, &foreign_element),
            Err(StructureError::DatumMismatch)
        );

        let sl2_table = filled_table(&sl2, 2, 4);
        let sl2_coset = TitsCoset::new(&sl2, vec![true]).unwrap();
        let fundamental = fundamental_id(&sl2_table);
        let element = TitsElement::new(&sl2_table, fundamental, bits(1, vec![])).unwrap();
        assert_eq!(
            sl2_coset.cross(&sl2_table, 3, &element),
            Err(StructureError::IndexOutOfRange {
                index: 3,
                upper_bound: 1,
            })
        );
        assert_eq!(
            TitsElement::new(&sl2_table, InvolutionId(99), bits(1, vec![])),
            Err(StructureError::IndexOutOfRange {
                index: 99,
                upper_bound: 2,
            })
        );
        assert_eq!(
            TitsElement::new(&sl2_table, fundamental, bits(3, vec![])),
            Err(StructureError::RankMismatch {
                expected: 1,
                actual: 3,
            })
        );

        // Public reduce: normalizes raw bits and is idempotent.
        let split = (0..sl2_table.involution_count())
            .map(InvolutionId)
            .find(|&id| !sl2_table.weyl_is_identity(id).unwrap())
            .unwrap();
        let raw = TitsElement::new(&sl2_table, split, bits(1, vec![0])).unwrap();
        let reduced = sl2_coset.reduce(&sl2_table, &raw).unwrap();
        assert!(reduced.torus_bits().is_zero());
        assert_eq!(sl2_coset.reduce(&sl2_table, &reduced).unwrap(), reduced);
    }
}
