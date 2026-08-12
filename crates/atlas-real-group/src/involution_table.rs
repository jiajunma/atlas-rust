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
//! Numbering is the caller's Cartan add order (the documented discipline is
//! ascending [`CartanId`]) with an external-order BFS inside each orbit.

use std::collections::BTreeMap;

use crate::grading::try_capacity;
use crate::integer_lattice::{negative_coweight_eigenspace, reduce_basis_mod_two};
use crate::real_projection::RealProjection;
use crate::{
    CartanClassification, CartanId, CayleyCrossDecomposition, InnerClass, IntegerLatticeBudget,
    LatticeInvolution, ModTwoSubspace, RootId, RootKind, RootSystem, StructureError,
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
/// path-transported `(1-theta)X^*` image-basis pair.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvolutionRecord {
    element: WeylElement,
    involution: TwistedInvolution,
    mod_space: ModTwoSubspace,
    theta_plus_one_rho: Weight,
    involution_length: usize,
    weyl_length: usize,
    projection: RealProjection,
}

impl InvolutionRecord {
    pub fn weyl_element(&self) -> &WeylElement {
        &self.element
    }

    pub fn twisted_involution(&self) -> &TwistedInvolution {
        &self.involution
    }

    /// Shallow convenience for the composed lattice involution theta.
    pub fn theta(&self) -> &LatticeInvolution {
        self.involution.root_involution().involution()
    }

    /// The X_* mod-2 dedup subspace; its ordered basis serves the
    /// inverse-Cayley repair of the Tits stage.
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

/// The involution table: per-Cartan contiguous orbit slices of twisted
/// involutions, shared across the real forms of one inner class.
#[derive(Clone, Debug)]
pub struct InvolutionTable {
    inner_class: InnerClass,
    budget: InvolutionTableBudget,
    twist: Vec<usize>,
    reflections: Vec<WeylElement>,
    reflection_actions: Vec<WeylAction>,
    two_rho: Weight,
    records: Vec<InvolutionRecord>,
    index_by_permutation: BTreeMap<Vec<RootId>, InvolutionId>,
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
            inner_class: inner_class.clone(),
            budget,
            twist,
            reflections,
            reflection_actions,
            two_rho: Weight::new(two_rho),
            records: Vec::new(),
            index_by_permutation: BTreeMap::new(),
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

        // Seed: convert the matrix-level representative once, then apply the
        // (W_length + #Cayley)/2 formula. `CayleyCrossDecomposition` is a
        // per-class tool only — never per entry at scale.
        let seed_element =
            WeylElement::from_action(self.inner_class.root_system(), representative.weyl_action())?;
        let seed_w_length = seed_element.length();
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
            &mut self.records,
            &mut self.index_by_permutation,
            seed_element,
            representative.weyl_action().clone(),
            length_sum / 2,
            None,
        )?;

        // External-order BFS. Cross links are filled at each node's visit,
        // so after the orbit closes every (generator, node) edge is an O(1)
        // stored link.
        let semisimple_rank = self.twist.len();
        let mut cursor = start;
        while cursor < self.records.len() {
            let current_element = self.records[cursor].element.clone();
            let current_action = self.records[cursor].involution.weyl_action().clone();
            let current_length = self.records[cursor].involution_length;
            let current_projection = self.records[cursor].projection().clone();
            let mut links = try_capacity(semisimple_rank)?;
            for generator in 0..semisimple_rank {
                let neighbor = self.reflections[generator]
                    .multiply(self.inner_class.root_system(), &current_element)?
                    .multiply(
                        self.inner_class.root_system(),
                        &self.reflections[self.twist[generator]],
                    )?;
                if let Some(&existing) = self.index_by_permutation.get(neighbor.image_permutation())
                {
                    links.push(existing);
                    continue;
                }
                let new_length =
                    stepped_length(current_length, current_element.length(), neighbor.length())?;
                let new_action = self.reflection_actions[generator]
                    .compose(&current_action)?
                    .compose(&self.reflection_actions[self.twist[generator]])?;
                // Transport the image basis across the cross edge
                // (involutions.cpp:242-243): the PLAIN generator s, not
                // twist(s) — delta is already incorporated in theta.
                let transported =
                    current_projection.transported(self.reflection_actions[generator].matrix())?;
                let id = push_record(
                    &self.inner_class,
                    &self.budget,
                    &self.two_rho,
                    &mut self.records,
                    &mut self.index_by_permutation,
                    neighbor,
                    new_action,
                    new_length,
                    Some(transported),
                )?;
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
        self.index_by_permutation
            .get(element.image_permutation())
            .copied()
    }

    /// Bounded by the involution count.
    pub fn record(&self, id: InvolutionId) -> Option<&InvolutionRecord> {
        self.records.get(id.0)
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
        let product = reflection.multiply(self.inner_class.root_system(), &record.element)?;
        Ok(self
            .index_by_permutation
            .get(product.image_permutation())
            .copied())
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

/// The single entry path: every record field except the image-basis pair is
/// derived fresh from theta; the pair is seeded from theta at the orbit's
/// canonical involution and thereafter TRANSPORTED along the cross edge that
/// first reached the record (`Some`), matching upstream's `add_involution` /
/// `add_cross` split — the basis is path-dependent, so it is never
/// re-derived from theta away from the seed.
/// A free function over disjoint table fields so the BFS can hold the
/// reflection caches while inserting.
#[allow(clippy::too_many_arguments)]
fn push_record(
    inner_class: &InnerClass,
    budget: &InvolutionTableBudget,
    two_rho: &Weight,
    records: &mut Vec<InvolutionRecord>,
    index_by_permutation: &mut BTreeMap<Vec<RootId>, InvolutionId>,
    element: WeylElement,
    action: WeylAction,
    involution_length: usize,
    transported_projection: Option<RealProjection>,
) -> Result<InvolutionId, StructureError> {
    if records.len() == budget.max_involutions {
        return Err(StructureError::InvolutionTableResourceLimit {
            resource: "involutions",
            limit: budget.max_involutions,
        });
    }
    let involution = TwistedInvolution::new(
        inner_class.datum(),
        inner_class.root_system(),
        inner_class.distinguished_involution().involution(),
        action,
    )?;
    let theta = involution.root_involution().involution();
    let eigenlattice =
        negative_coweight_eigenspace(theta.coweight_matrix(), &budget.integer_lattice)?;
    let mod_space = reduce_basis_mod_two(&eigenlattice)?;
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
    let weyl_length = element.length();
    let id = InvolutionId(records.len());
    let mut key = try_capacity(element.image_permutation().len())?;
    key.extend_from_slice(element.image_permutation());
    index_by_permutation.insert(key, id);
    records.push(InvolutionRecord {
        element,
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
