use crate::adjoint_fiber::AdjointFiberBudget;
use crate::grading::try_capacity;
use crate::integer_lattice::IntegerLatticeBudget;
use crate::{
    AdjointCartanFiber, CartanClass, CartanFiber, CartanGradingData, CayleyCrossDecomposition,
    InnerClass, RealFormLabels, RootKind, StructureError, TwistedConjugacyClass, TwistedInvolution,
    WeakRealFormId, WeakRealFormPartition, WeylAction,
};

/// Stable identifier for one Cartan class of one inner class.
///
/// Numbers follow the crate Cartan order: the fundamental class first, then
/// the remaining classes in the deterministic twisted-conjugacy enumeration
/// order. They are deterministic for this crate but not Atlas numbering; the
/// standing compatibility-adapter deferral covers them. The fundamental
/// class is always `CartanId(0)`.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CartanId(pub(crate) usize);

/// Owned budgets for building one full Cartan classification.
///
/// Each scalar threads to exactly one existing knob: the Weyl enumeration,
/// the per-Cartan fiber-element cap, and the decomposition peeling bound;
/// the nested budgets feed the fiber chains. No new limit kinds exist here.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CartanClassificationBudget {
    integer_lattice: IntegerLatticeBudget,
    adjoint_fiber: AdjointFiberBudget,
    weyl_budget: usize,
    max_fiber_elements: usize,
    max_peeling_steps: usize,
}

impl CartanClassificationBudget {
    pub const fn new(
        integer_lattice: IntegerLatticeBudget,
        adjoint_fiber: AdjointFiberBudget,
        weyl_budget: usize,
        max_fiber_elements: usize,
        max_peeling_steps: usize,
    ) -> Self {
        Self {
            integer_lattice,
            adjoint_fiber,
            weyl_budget,
            max_fiber_elements,
            max_peeling_steps,
        }
    }
}

/// The Cartan classes of one inner class with their aggregated facts:
/// per-form Cartan sets, most-split classes, the involution count, and the
/// strict Cartan partial order.
///
/// `is_below(a, b)` is true iff `a != b` and the identity component of `b`'s
/// fixed torus is Weyl-conjugate into `a`'s — `a` is the more-compact end of
/// a nonempty chain of single-root Cayley links into `b`. The fundamental
/// class is below every other class.
#[derive(Clone, Debug)]
pub struct CartanClassification {
    cartan_classes: Vec<CartanClass>,
    below: Vec<Vec<bool>>,
    cartan_sets: Vec<Vec<CartanId>>,
    most_split: Vec<CartanId>,
    twisted_involution_count: usize,
}

impl CartanClassification {
    /// Build every Cartan class of the inner class and aggregate.
    pub fn build(
        inner_class: &InnerClass,
        budget: &CartanClassificationBudget,
    ) -> Result<Self, StructureError> {
        let datum = inner_class.datum();
        let root_system = inner_class.root_system();
        let delta = inner_class.distinguished_involution().involution();
        let partition = inner_class.twisted_conjugacy_partition(budget.weyl_budget)?;
        let class_count = partition.classes().len();

        // Crate Cartan order: fundamental first. The raw enumeration order is
        // matrix-lexicographic and generally puts the identity class last;
        // the class is found by MEMBERSHIP, since a multi-element class may
        // have been seeded by a non-identity candidate. Its representative is
        // then normalized to the identity twisted involution — a member of
        // the class — so the fundamental chain sits at the distinguished
        // involution, as the label gates require.
        let identity_twisted =
            TwistedInvolution::new(datum, root_system, delta, WeylAction::identity(datum)?)?;
        let fundamental_raw = partition.class_of(&identity_twisted)?;
        let mut order = try_capacity(class_count)?;
        order.push(fundamental_raw);
        for raw in 0..class_count {
            if raw != fundamental_raw {
                order.push(raw);
            }
        }
        let mut position_of_raw = try_capacity(class_count)?;
        position_of_raw.resize(class_count, 0_usize);
        for (position, &raw) in order.iter().enumerate() {
            position_of_raw[raw] = position;
        }
        let mut class_infos = try_capacity(class_count)?;
        for &raw in &order {
            if raw == fundamental_raw {
                class_infos.push(TwistedConjugacyClass::new(
                    identity_twisted.clone(),
                    partition.classes()[raw].twisted_involution_count(),
                ));
            } else {
                class_infos.push(partition.classes()[raw].clone());
            }
        }

        // Phase 1: the fiber chain and decomposition at every representative.
        struct PartialClass {
            grading: CartanGradingData,
            partition: WeakRealFormPartition,
            decomposition: CayleyCrossDecomposition,
        }
        let mut partial = try_capacity(class_count)?;
        for class_info in &class_infos {
            let twisted = class_info.representative();
            let data = twisted.root_involution();
            let source = CartanFiber::build(data.involution(), &budget.integer_lattice)?;
            let adjoint =
                AdjointCartanFiber::build(root_system, data, &source, &budget.adjoint_fiber)?;
            let grading = CartanGradingData::build(root_system, data, &adjoint)?;
            let weak = WeakRealFormPartition::build(&grading, budget.max_fiber_elements)?;
            let decomposition =
                CayleyCrossDecomposition::build(inner_class, twisted, budget.max_peeling_steps)?;
            partial.push(PartialClass {
                grading,
                partition: weak,
                decomposition,
            });
        }

        // Phase 2: labels against the fundamental class, including its own
        // identity map.
        let mut labels = try_capacity(class_count)?;
        for entry in &partial {
            labels.push(RealFormLabels::build(
                inner_class,
                &partial[0].grading,
                &partial[0].partition,
                &entry.grading,
                &entry.partition,
                &entry.decomposition,
            )?);
        }

        // Phase 3: assemble the owning values in crate Cartan order.
        let mut cartan_classes = try_capacity(class_count)?;
        for ((class_info, entry), label) in class_infos.into_iter().zip(partial).zip(labels) {
            cartan_classes.push(CartanClass::new(
                class_info,
                entry.decomposition,
                entry.grading,
                entry.partition,
                label,
            ));
        }

        // The Cayley-link relation, then an order-independent transitive
        // closure: the crate class order is not graded, so the incremental
        // upstream scheme would under-close.
        let mut below = try_capacity(class_count)?;
        for _ in 0..class_count {
            let mut row = try_capacity(class_count)?;
            row.resize(class_count, false);
            below.push(row);
        }
        for (source_position, cartan_class) in cartan_classes.iter().enumerate() {
            let twisted = cartan_class.representative();
            let data = twisted.root_involution();
            let mut imaginary = try_capacity(root_system.roots().len())?;
            for root in data.roots_of_kind(RootKind::Imaginary) {
                let coordinates = root_system.simple_coordinates(root).ok_or(
                    StructureError::IndexOutOfRange {
                        index: root.0,
                        upper_bound: root_system.roots().len(),
                    },
                )?;
                if coordinates.iter().all(|&coordinate| coordinate >= 0) {
                    imaginary.push(root);
                }
            }
            for root in imaginary {
                let reflection = WeylAction::root_reflection(datum, root_system, root)?;
                let action = reflection.compose(twisted.weyl_action())?;
                let successor =
                    TwistedInvolution::new(datum, root_system, delta, action).map_err(|error| {
                        match error {
                            StructureError::InvalidInvolution => {
                                StructureError::CartanClassificationInvariantViolation {
                                    invariant: "Cayley successor",
                                }
                            }
                            other => other,
                        }
                    })?;
                let target_position = position_of_raw[partition.class_of(&successor)?];
                if target_position == source_position {
                    return Err(StructureError::CartanClassificationInvariantViolation {
                        invariant: "Cayley successor",
                    });
                }
                below[target_position][source_position] = true;
            }
        }
        for k in 0..class_count {
            for t in 0..class_count {
                if below[t][k] {
                    for s in 0..class_count {
                        if below[k][s] {
                            below[t][s] = true;
                        }
                    }
                }
            }
        }
        for (index, row) in below.iter().enumerate() {
            if row[index] {
                return Err(StructureError::CartanClassificationInvariantViolation {
                    invariant: "strict Cartan order",
                });
            }
        }

        // Aggregates, precomputed so the invariants fire at construction.
        let form_count = cartan_classes[0].partition().class_count();
        let mut cartan_sets = try_capacity(form_count)?;
        let mut most_split = try_capacity(form_count)?;
        for form_index in 0..form_count {
            let form = WeakRealFormId(form_index);
            let mut cartan_set = Vec::new();
            let mut split_candidates = Vec::new();
            for (position, cartan_class) in cartan_classes.iter().enumerate() {
                let Some(local_index) = cartan_class
                    .labels()
                    .labels()
                    .iter()
                    .position(|&label| label == form)
                else {
                    continue;
                };
                cartan_set.push(CartanId(position));
                let local = WeakRealFormId(local_index);
                let representative = cartan_class.partition().class_representative(local).ok_or(
                    StructureError::IndexOutOfRange {
                        index: local_index,
                        upper_bound: cartan_class.partition().class_count(),
                    },
                )?;
                let grading = cartan_class.grading().grading(representative)?;
                if grading.noncompact_indices().count() == 0 {
                    split_candidates.push(CartanId(position));
                }
            }
            if split_candidates.len() != 1 {
                return Err(StructureError::CartanClassificationInvariantViolation {
                    invariant: "most-split uniqueness",
                });
            }
            cartan_sets.push(cartan_set);
            most_split.push(split_candidates[0]);
        }
        let mut twisted_involution_count = 0_usize;
        for cartan_class in &cartan_classes {
            twisted_involution_count = twisted_involution_count
                .checked_add(cartan_class.twisted_involution_count())
                .ok_or(StructureError::ArithmeticOverflow)?;
        }

        Ok(Self {
            cartan_classes,
            below,
            cartan_sets,
            most_split,
            twisted_involution_count,
        })
    }

    pub fn cartan_classes(&self) -> &[CartanClass] {
        &self.cartan_classes
    }

    /// Bounded by the class count.
    pub fn cartan_class(&self, id: CartanId) -> Option<&CartanClass> {
        self.cartan_classes.get(id.0)
    }

    /// The number of weak real forms, from the fundamental partition.
    pub fn weak_real_form_count(&self) -> usize {
        self.cartan_sets.len()
    }

    /// The Cartan classes at which this real form lives, ascending.
    pub fn cartan_set(&self, form: WeakRealFormId) -> Option<&[CartanId]> {
        self.cartan_sets.get(form.0).map(Vec::as_slice)
    }

    /// The unique most-split Cartan class of this real form.
    pub fn most_split(&self, form: WeakRealFormId) -> Option<CartanId> {
        self.most_split.get(form.0).copied()
    }

    /// The total number of twisted involutions across all classes.
    pub fn twisted_involution_count(&self) -> usize {
        self.twisted_involution_count
    }

    /// Strict closed Cayley order: `a` more compact than `b`. Irreflexivity
    /// is a construction invariant, so `is_below(x, x)` is `Some(false)`.
    pub fn is_below(&self, a: CartanId, b: CartanId) -> Option<bool> {
        if a.0 >= self.below.len() {
            return None;
        }
        self.below.get(b.0).map(|row| row[a.0])
    }
}

#[cfg(test)]
mod tests {
    use crate::{BasedRootDatum, Coweight, LatticeInvolution, Weight};

    use super::*;

    fn budget(weyl: usize) -> CartanClassificationBudget {
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

    fn classification(
        datum: &BasedRootDatum,
        distinguished: LatticeInvolution,
        weyl: usize,
    ) -> CartanClassification {
        let inner_class = InnerClass::new(datum.clone(), distinguished, 2 * weyl.max(4)).unwrap();
        CartanClassification::build(&inner_class, &budget(weyl)).unwrap()
    }

    #[test]
    fn simply_connected_a1_has_two_cartans_with_the_expected_attribution() {
        let datum = BasedRootDatum::from_simple_data(
            1,
            vec![vec![2]],
            vec![Weight::new(vec![2])],
            vec![Coweight::new(vec![1])],
        )
        .unwrap();
        let classification =
            classification(&datum, LatticeInvolution::identity(&datum).unwrap(), 2);

        assert_eq!(classification.cartan_classes().len(), 2);
        assert_eq!(classification.twisted_involution_count(), 2);
        assert_eq!(classification.weak_real_form_count(), 2);
        let quasisplit = WeakRealFormId(0);
        let compact = WeakRealFormId(1);
        assert_eq!(
            classification.cartan_set(quasisplit),
            Some(&[CartanId(0), CartanId(1)][..])
        );
        assert_eq!(classification.cartan_set(compact), Some(&[CartanId(0)][..]));
        assert_eq!(classification.most_split(quasisplit), Some(CartanId(1)));
        assert_eq!(classification.most_split(compact), Some(CartanId(0)));
        assert_eq!(
            classification.is_below(CartanId(0), CartanId(1)),
            Some(true)
        );
        assert_eq!(
            classification.is_below(CartanId(1), CartanId(0)),
            Some(false)
        );
    }

    #[test]
    fn a2_identity_reorders_the_fundamental_class_first() {
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let classification =
            classification(&datum, LatticeInvolution::identity(&datum).unwrap(), 6);

        assert_eq!(classification.cartan_classes().len(), 2);
        let sizes: Vec<usize> = classification
            .cartan_classes()
            .iter()
            .map(CartanClass::twisted_involution_count)
            .collect();
        assert_eq!(sizes, vec![1, 3]);
        assert!(classification.cartan_classes()[0]
            .representative()
            .weyl_action()
            .matrix()
            .iter()
            .enumerate()
            .all(|(row, values)| values
                .iter()
                .enumerate()
                .all(|(column, &value)| value == i32::from(row == column))));
        assert_eq!(classification.twisted_involution_count(), 4);
        assert_eq!(
            classification.cartan_set(WeakRealFormId(0)),
            Some(&[CartanId(0), CartanId(1)][..])
        );
        assert_eq!(
            classification.cartan_set(WeakRealFormId(1)),
            Some(&[CartanId(0)][..])
        );
        assert_eq!(
            classification.is_below(CartanId(0), CartanId(1)),
            Some(true)
        );
        assert_eq!(
            classification.is_below(CartanId(1), CartanId(0)),
            Some(false)
        );
    }

    #[test]
    fn b2_identity_has_the_graded_five_pair_poset() {
        let datum = BasedRootDatum::standard(vec![vec![2, -2], vec![-1, 2]]).unwrap();
        let classification =
            classification(&datum, LatticeInvolution::identity(&datum).unwrap(), 8);

        assert_eq!(classification.cartan_classes().len(), 4);
        assert_eq!(classification.twisted_involution_count(), 6);
        let mut sizes: Vec<usize> = classification
            .cartan_classes()
            .iter()
            .map(CartanClass::twisted_involution_count)
            .collect();
        assert_eq!(sizes[0], 1);
        sizes.sort_unstable();
        assert_eq!(sizes, vec![1, 1, 2, 2]);

        // The unique maximal class is the central longest involution.
        let ids = [CartanId(0), CartanId(1), CartanId(2), CartanId(3)];
        let maximal: Vec<CartanId> = ids
            .into_iter()
            .filter(|&candidate| {
                ids.iter()
                    .all(|&other| classification.is_below(candidate, other) != Some(true))
            })
            .collect();
        assert_eq!(maximal.len(), 1);
        let top = maximal[0];
        assert_eq!(
            classification
                .cartan_class(top)
                .unwrap()
                .twisted_involution_count(),
            1
        );
        let mut true_pairs = 0;
        for &a in &ids {
            for &b in &ids {
                if classification.is_below(a, b) == Some(true) {
                    true_pairs += 1;
                    assert!(a == CartanId(0) || b == top);
                }
            }
        }
        assert_eq!(true_pairs, 5);

        assert_eq!(classification.weak_real_form_count(), 3);
        let quasisplit = WeakRealFormId(0);
        assert_eq!(classification.cartan_set(quasisplit).unwrap().len(), 4);
        assert_eq!(classification.most_split(quasisplit), Some(top));
        // The compact form so(5) is the all-compact-grading fundamental class.
        assert_eq!(
            classification.cartan_set(WeakRealFormId(2)),
            Some(&[CartanId(0)][..])
        );
        assert_eq!(
            classification.most_split(WeakRealFormId(2)),
            Some(CartanId(0))
        );
    }

    #[test]
    fn a1_x_a1_identity_and_swap_have_the_expected_shapes() {
        let datum = BasedRootDatum::standard(vec![vec![2, 0], vec![0, 2]]).unwrap();
        let classification_identity =
            classification(&datum, LatticeInvolution::identity(&datum).unwrap(), 4);
        assert_eq!(classification_identity.cartan_classes().len(), 4);
        assert!(classification_identity
            .cartan_classes()
            .iter()
            .all(|class| class.twisted_involution_count() == 1));
        assert_eq!(classification_identity.twisted_involution_count(), 4);

        let swap = LatticeInvolution::new(
            &datum,
            vec![vec![0, 1], vec![1, 0]],
            vec![vec![0, 1], vec![1, 0]],
        )
        .unwrap();
        let classification_swap = classification(&datum, swap, 4);
        assert_eq!(classification_swap.cartan_classes().len(), 1);
        assert_eq!(
            classification_swap.cartan_classes()[0].twisted_involution_count(),
            2
        );
        assert_eq!(classification_swap.twisted_involution_count(), 2);
        assert_eq!(
            classification_swap.is_below(CartanId(0), CartanId(0)),
            Some(false)
        );
    }

    #[test]
    fn the_partition_lookup_round_trips_and_rejects_foreign_provenance() {
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let inner_class = InnerClass::new(
            datum.clone(),
            LatticeInvolution::identity(&datum).unwrap(),
            6,
        )
        .unwrap();
        let partition = inner_class.twisted_conjugacy_partition(6).unwrap();
        for twisted in inner_class.twisted_involutions(6).unwrap() {
            let class_index = partition.class_of(&twisted).unwrap();
            assert!(class_index < partition.classes().len());
        }

        let b2 = BasedRootDatum::standard(vec![vec![2, -2], vec![-1, 2]]).unwrap();
        let b2_class =
            InnerClass::new(b2.clone(), LatticeInvolution::identity(&b2).unwrap(), 8).unwrap();
        let foreign = b2_class.twisted_involutions(8).unwrap().remove(0);
        assert!(matches!(
            partition.class_of(&foreign),
            Err(StructureError::DatumMismatch)
        ));

        let twist = LatticeInvolution::new(
            &datum,
            vec![vec![0, 1], vec![1, 0]],
            vec![vec![0, 1], vec![1, 0]],
        )
        .unwrap();
        let twist_backed = TwistedInvolution::new(
            &datum,
            inner_class.root_system(),
            &twist,
            crate::WeylGroup::new(datum.clone()).identity().unwrap(),
        )
        .unwrap();
        assert!(matches!(
            partition.class_of(&twist_backed),
            Err(StructureError::DistinguishedInvolutionMismatch)
        ));
    }
}
