use std::collections::BTreeSet;

use malachite::base::num::arithmetic::traits::DivisibleBy;
use malachite::{Integer, Rational};

use crate::adjoint_fiber::AdjointFiberBudget;
use crate::grading::try_capacity;
use crate::integer_lattice::IntegerLatticeBudget;
use crate::{
    AdjointCartanFiber, CartanClass, CartanFiber, CartanGradingData, CayleyCrossDecomposition,
    Grading, InnerClass, RealFormLabels, RootKind, RootSystem, StructureError,
    TwistedConjugacyClass, TwistedInvolution, WeakRealFormId, WeakRealFormPartition, Weight,
    WeylAction, WeylElement,
};

/// Stable identifier for one Cartan class of one inner class.
///
/// Numbers follow the Atlas Cartan order (innerclass.cpp:218-291, task 1):
/// the fundamental class is `CartanId(0)`, and every later class is numbered
/// in BFS discovery order — parents in ascending number, each parent's
/// positive imaginary roots in upstream `RootNbr` order (height, then
/// reverse-lexicographic simple coordinates), with the Cayley successor
/// canonicalized before it is compared and stored.
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

        // Atlas Cartan order (innerclass.cpp:218-291, task 1): BFS discovery.
        // Cartan[0] is the fundamental class at the identity twisted
        // involution (innerclass.cpp:145 seeds `Cartan(1,C_info(...))` with
        // the default TwistedInvolution). Walking parents in discovery order,
        // each parent's positive imaginary roots Cayley-transform the parent
        // representative; the successor is canonicalized and appended only if
        // its class has no number yet. Representatives are therefore the
        // canonical twisted involutions upstream compares and stores
        // (innerclass.cpp:252-263), never raw enumeration members.
        //
        // The upstream loop conjugates each root down to a simple one and
        // Cayley-transforms by that simple reflection; the crate transforms
        // by the root's own reflection directly. The two successors are
        // twisted-conjugate (the descent conjugator carries `s_alpha w` to
        // `s_k c(w)`), so they land on the same class and canonicalize to
        // the same representative.
        let identity_twisted =
            TwistedInvolution::new(datum, root_system, delta, WeylAction::identity(datum)?)?;
        let fundamental_raw = partition.class_of(&identity_twisted)?;
        let mut representatives = try_capacity(class_count)?;
        let mut raw_of_position = try_capacity(class_count)?;
        let mut position_of_raw = try_capacity(class_count)?;
        position_of_raw.resize(class_count, usize::MAX);
        representatives.push(identity_twisted);
        raw_of_position.push(fundamental_raw);
        position_of_raw[fundamental_raw] = 0;

        // The Cayley-link relation, filled as the BFS discovers it (upstream
        // `Cartan[ii].below.insert(i)`), then an order-independent transitive
        // closure: the BFS number of a more-split class need not exceed its
        // parent's, so the incremental upstream scheme would under-close.
        let mut below = try_capacity(class_count)?;
        for _ in 0..class_count {
            let mut row = try_capacity(class_count)?;
            row.resize(class_count, false);
            below.push(row);
        }
        let mut cursor = 0_usize;
        while cursor < representatives.len() {
            let twisted = representatives[cursor].clone();
            let data = twisted.root_involution();
            // Positive imaginary roots in upstream RootNbr order.
            let mut imaginary = try_capacity(root_system.roots().len())?;
            for root in data.roots_of_kind(RootKind::Imaginary) {
                let coordinates = root_system.simple_coordinates(root).ok_or(
                    StructureError::IndexOutOfRange {
                        index: root.0,
                        upper_bound: root_system.roots().len(),
                    },
                )?;
                if coordinates.iter().all(|&coordinate| coordinate >= 0) {
                    imaginary.push((upstream_positive_key(coordinates)?, root));
                }
            }
            imaginary.sort_by(|(left, _), (right, _)| left.cmp(right));
            for (_, root) in imaginary {
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
                let (canonical, _conjugator) = inner_class.canonicalize(successor)?;
                let raw = partition.class_of(&canonical)?;
                if position_of_raw[raw] == usize::MAX {
                    position_of_raw[raw] = representatives.len();
                    raw_of_position.push(raw);
                    representatives.push(canonical);
                }
                let target_position = position_of_raw[raw];
                if target_position == cursor {
                    return Err(StructureError::CartanClassificationInvariantViolation {
                        invariant: "Cayley successor",
                    });
                }
                below[target_position][cursor] = true;
            }
            cursor += 1;
        }
        if representatives.len() != class_count {
            return Err(StructureError::CartanClassificationInvariantViolation {
                invariant: "Cartan discovery completeness",
            });
        }
        let mut class_infos = try_capacity(class_count)?;
        for (representative, &raw) in representatives.into_iter().zip(&raw_of_position) {
            class_infos.push(TwistedConjugacyClass::new(
                representative,
                partition.classes()[raw].twisted_involution_count(),
            ));
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

        // Phase 3: assemble the owning values in Atlas Cartan order.
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

        // Transitive closure of the BFS-filled Cayley links, then the
        // strict-order irreflexivity check.
        for k in 0..class_count {
            for t in 0..class_count {
                if below[t][k] {
                    let source = below[k].clone();
                    for (slot, &flag) in below[t].iter_mut().zip(&source) {
                        if flag {
                            *slot = true;
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

    /// The class ids in ascending order, for external consumers that cannot
    /// construct [`CartanId`] values directly.
    pub fn cartan_ids(&self) -> impl ExactSizeIterator<Item = CartanId> {
        (0..self.cartan_classes.len()).map(CartanId)
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

    /// Port of upstream `real_form_of` (innerclass.cpp:1305-1355, sole
    /// caller the synthetic `real_form(InnerClass,mat,ratvec)` wrapper,
    /// interpreter/atlas-types.w:3851-3871): the weak real form claiming
    /// the strong-involution datum `(twisted, factor)`. `factor` is the
    /// caller-projected theta-fixed rational coweight (the wrapper's
    /// doubled, centrality-checked, then halved torus factor). Upstream's
    /// `coch` output feeds only `minimal_torus_part` and the
    /// default-seed comparison in `real_form_value::build`; the crate's
    /// [`crate::RealFormSeed`] recomputes the elected seed from the form
    /// id alone, so no cocharacter is returned here.
    ///
    /// Upstream canonicalizes `tw` and cross-acts the torus part along;
    /// the canonicalize conjugator uses only COMPLEX letters
    /// (innerclass.cpp:760-766 records why), so `complex_cross_act`
    /// (tits.h:191-192 — a simple reflection of the factor) is the only
    /// transport that ever fires inside `real_form_of`. The walk below
    /// therefore branches only on complex simple roots: an imaginary or
    /// real simple root fixes the Weyl part exactly (`s theta s = theta`
    /// when `theta(alpha_s) = ±alpha_s`), so complex steps alone exhaust
    /// the Weyl-part cross orbit. The first class representative met is
    /// the datum's Cartan class, and the grading measured there — bit set
    /// (noncompact) iff the factor's pairing with the simple-imaginary
    /// root is an EVEN integer, upstream `gr.set(i, not
    /// a.torus_part().negative_at(...))` — classifies through the class's
    /// own partition and labels. The pairing's integrality is asserted
    /// upstream inside `negative_at` and gated here as a named invariant.
    pub fn real_form_of(
        &self,
        inner_class: &InnerClass,
        twisted: &WeylElement,
        factor: &[Rational],
    ) -> Result<WeakRealFormId, StructureError> {
        Ok(self.real_form_of_detailed(inner_class, twisted, factor)?.0)
    }

    /// [`Self::real_form_of`] plus the Cartan class of `twisted` — the
    /// synthetic wrapper additionally needs the class to extend its
    /// involution table over every class below it before calling
    /// `minimal_torus_part` (interpreter/atlas-types.w:3902-3907). The
    /// classification logic is unchanged: the class is the one whose
    /// representative the canonicalizing walk meets.
    pub fn real_form_of_detailed(
        &self,
        inner_class: &InnerClass,
        twisted: &WeylElement,
        factor: &[Rational],
    ) -> Result<(WeakRealFormId, CartanId), StructureError> {
        let datum = inner_class.datum();
        let system = inner_class.root_system();
        let rank = datum.lattice_rank();
        if factor.len() != rank {
            return Err(StructureError::RankMismatch {
                expected: rank,
                actual: factor.len(),
            });
        }
        if twisted.image_permutation().len() != system.roots().len() {
            return Err(StructureError::DatumMismatch);
        }
        let distinguished = inner_class.distinguished_involution();
        let simple_ids = system.simple_root_ids();
        let simple_roots = datum.simple_roots();
        let simple_coroots = datum.simple_coroots();

        let mut twist = try_capacity(simple_ids.len())?;
        let mut opposite = try_capacity(simple_ids.len())?;
        for &simple_id in simple_ids {
            let image = distinguished.image(simple_id).ok_or(
                StructureError::CartanClassificationInvariantViolation {
                    invariant: "distinguished simple image",
                },
            )?;
            let position = simple_ids
                .iter()
                .position(|&candidate| candidate == image)
                .ok_or(StructureError::CartanClassificationInvariantViolation {
                    invariant: "distinguished simple image",
                })?;
            twist.push(position);
            let negated: Vec<i32> = system
                .root(simple_id)
                .ok_or(StructureError::CartanClassificationInvariantViolation {
                    invariant: "simple root",
                })?
                .as_slice()
                .iter()
                .map(|coordinate| -coordinate)
                .collect();
            opposite.push(system.id_of(&Weight::new(negated)).ok_or(
                StructureError::CartanClassificationInvariantViolation {
                    invariant: "opposite simple root",
                },
            )?);
        }
        let mut representatives = try_capacity(self.cartan_classes.len())?;
        for cartan_class in &self.cartan_classes {
            representatives.push(WeylElement::from_action(
                system,
                cartan_class.representative().weyl_action(),
            )?);
        }

        // Depth-first over the cross orbit; each element is pushed once,
        // so the class size bounds the pops, and the representative of the
        // datum's own class is always reached.
        let mut visited = BTreeSet::new();
        let mut stack = try_capacity(self.twisted_involution_count())?;
        visited.insert(twisted.clone());
        stack.push((twisted.clone(), factor.to_vec()));
        let mut pops = 0_usize;
        while let Some((element, transported)) = stack.pop() {
            pops = pops
                .checked_add(1)
                .ok_or(StructureError::ArithmeticOverflow)?;
            if pops > self.twisted_involution_count() {
                return Err(StructureError::CartanClassificationInvariantViolation {
                    invariant: "synthetic class search",
                });
            }
            if let Some(position) = representatives
                .iter()
                .position(|representative| representative == &element)
            {
                return self
                    .synthetic_form_at(position, system, &transported)
                    .map(|form| (form, CartanId(position)));
            }
            for generator in 0..simple_ids.len() {
                let delta_image = distinguished.image(simple_ids[generator]).ok_or(
                    StructureError::CartanClassificationInvariantViolation {
                        invariant: "distinguished simple image",
                    },
                )?;
                let theta_image = element.image(delta_image).ok_or(
                    StructureError::CartanClassificationInvariantViolation {
                        invariant: "twisted simple image",
                    },
                )?;
                if theta_image == simple_ids[generator] || theta_image == opposite[generator] {
                    // Imaginary or real: the Weyl part is fixed, and
                    // upstream's conjugator never uses these letters here.
                    continue;
                }
                let next = element.twisted_conjugate(system, generator, &twist)?;
                let mut next_factor = transported.clone();
                let pairing = rational_dot(simple_roots[generator].as_slice(), &next_factor)?;
                for (slot, &coordinate) in next_factor
                    .iter_mut()
                    .zip(simple_coroots[generator].as_slice())
                {
                    if coordinate != 0 {
                        *slot -= &pairing * Rational::from(coordinate);
                    }
                }
                if visited.insert(next.clone()) {
                    stack.push((next, next_factor));
                }
            }
        }
        Err(StructureError::CartanClassificationInvariantViolation {
            invariant: "synthetic class search",
        })
    }

    /// The grading classification at one class representative: measure the
    /// parities, solve the adjoint fiber element with that grading, and
    /// read its weak-real-form label off the class's labels (upstream
    /// `f.gradingRep(gr)` + `f.adjoint_orbit(rep)` +
    /// `G.realFormLabels(cn)[...]`).
    fn synthetic_form_at(
        &self,
        position: usize,
        system: &RootSystem,
        factor: &[Rational],
    ) -> Result<WeakRealFormId, StructureError> {
        let cartan_class = &self.cartan_classes[position];
        let grading = cartan_class.grading();
        let imaginary_rank = grading.imaginary_rank();
        let mut noncompact = try_capacity(imaginary_rank)?;
        for imaginary_index in 0..imaginary_rank {
            let root = grading.imaginary_simple_root(imaginary_index).ok_or(
                StructureError::CartanClassificationInvariantViolation {
                    invariant: "imaginary simple root",
                },
            )?;
            let coordinates = system.root(root).ok_or(
                StructureError::CartanClassificationInvariantViolation {
                    invariant: "imaginary simple root",
                },
            )?;
            let pairing = rational_dot(coordinates.as_slice(), factor)?;
            let integer = Integer::try_from(&pairing).map_err(|_| {
                StructureError::CartanClassificationInvariantViolation {
                    invariant: "synthetic grading integrality",
                }
            })?;
            if integer.divisible_by(&Integer::from(2)) {
                noncompact.push(imaginary_index);
            }
        }
        let target = Grading::from_noncompact(imaginary_rank, noncompact)?;
        let element = grading.element_from_grading(&target)?;
        let local = cartan_class.partition().class_of(&element)?;
        cartan_class.labels().labels().get(local.0).copied().ok_or(
            StructureError::CartanClassificationInvariantViolation {
                invariant: "synthetic form label",
            },
        )
    }
}

/// The exact pairing of an integer weight with a rational coweight factor.
fn rational_dot(coordinates: &[i32], factor: &[Rational]) -> Result<Rational, StructureError> {
    if coordinates.len() != factor.len() {
        return Err(StructureError::RankMismatch {
            expected: factor.len(),
            actual: coordinates.len(),
        });
    }
    let mut pairing = Rational::from(0);
    for (&coordinate, value) in coordinates.iter().zip(factor) {
        pairing += Rational::from(coordinate) * value;
    }
    Ok(pairing)
}

/// Upstream positive-`RootNbr` order key for one positive root's simple
/// coordinates: ascending height, then reverse-lexicographic coordinates.
///
/// The upstream `RootSystem` constructor (rootdata.cpp:131-220) appends
/// positive roots level by level and keeps each level in `root_compare`
/// order, which walks coordinates from the LAST index down; the crate's own
/// root order is ambient-lexicographic instead, so BFS consumers sort by
/// this key explicitly.
fn upstream_positive_key(coordinates: &[i32]) -> Result<(i32, Vec<i32>), StructureError> {
    let mut reversed = try_capacity(coordinates.len())?;
    let mut height = 0_i32;
    for &coordinate in coordinates.iter().rev() {
        reversed.push(coordinate);
        height = height
            .checked_add(coordinate)
            .ok_or(StructureError::ArithmeticOverflow)?;
    }
    Ok((height, reversed))
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
    fn a2_identity_seeds_the_bfs_with_the_fundamental_class() {
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
        // Upstream's task-2 invariant `assert(w0==Cartan.back().tw)`: the
        // longest class is discovered last.
        assert_eq!(top, CartanId(3));
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
    fn b2_identity_discovers_cartans_in_upstream_bfs_order() {
        // Upstream derivation (innerclass.cpp:218-291 with delta = 1), checked
        // against the oracle `Cartan_info` print for
        // `inner_class(simply_connected(Lie_type("B2")),"c")`: Cartan #0 is
        // the identity class; its positive imaginary roots in upstream
        // RootNbr order are (1,0), (0,1), (1,1), (1,2), so the long simple
        // root discovers the long-reflection class first (canonical
        // representative s1 s0 s1, oracle word "2,1,2") and the short simple
        // root discovers the short-reflection class second (canonical
        // s0 s1 s0, oracle "1,2,1"); the w0 class is discovered from #1 as
        // #3 (oracle "2,1,2,1"). Words below are 0-based, right-multiplied
        // left to right as upstream's `WeylGroup::element` does.
        let datum = BasedRootDatum::standard(vec![vec![2, -2], vec![-1, 2]]).unwrap();
        let inner_class = InnerClass::new(
            datum.clone(),
            LatticeInvolution::identity(&datum).unwrap(),
            8,
        )
        .unwrap();
        let classification = CartanClassification::build(&inner_class, &budget(8)).unwrap();

        assert_eq!(classification.cartan_classes().len(), 4);
        let expected_words: [&[usize]; 4] = [&[], &[1, 0, 1], &[0, 1, 0], &[1, 0, 1, 0]];
        for (position, word) in expected_words.iter().enumerate() {
            let mut expected = WeylElement::identity(inner_class.root_system()).unwrap();
            for &generator in *word {
                expected = expected
                    .right_multiply_simple(inner_class.root_system(), generator)
                    .unwrap()
                    .0;
            }
            let representative = classification.cartan_classes()[position].representative();
            let actual =
                WeylElement::from_action(inner_class.root_system(), representative.weyl_action())
                    .unwrap();
            assert_eq!(actual, expected, "Cartan #{position} representative");
        }
        let sizes: Vec<usize> = classification
            .cartan_classes()
            .iter()
            .map(CartanClass::twisted_involution_count)
            .collect();
        assert_eq!(sizes, vec![1, 2, 2, 1]);

        // The canonical representatives are class invariants: canonicalizing
        // ANY member (here the raw Cayley successors s0 and s1 that the BFS
        // transforms first) lands on the same numbered representative.
        for (generator, position) in [(0_usize, 1_usize), (1, 2)] {
            let simple = WeylAction::simple_reflection(&datum, generator).unwrap();
            let twisted = TwistedInvolution::new(
                &datum,
                inner_class.root_system(),
                inner_class.distinguished_involution().involution(),
                simple,
            )
            .unwrap();
            let (canonical, _) = inner_class.canonicalize(twisted).unwrap();
            let actual =
                WeylElement::from_action(inner_class.root_system(), canonical.weyl_action())
                    .unwrap();
            let stored = WeylElement::from_action(
                inner_class.root_system(),
                classification.cartan_classes()[position]
                    .representative()
                    .weyl_action(),
            )
            .unwrap();
            assert_eq!(actual, stored, "canonicalize(s{generator})");
        }
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

    #[test]
    fn real_form_of_matches_the_frozen_a1_anchors() {
        // The weak_real_form fixture's simply-connected compact A1: the
        // identity theta with the zero factor is the split form (internal
        // quasisplit 0, external LAST), the rho-shifted factor the compact
        // form (internal 1, external FIRST), and -1 = w0 with the zero
        // factor lands on the split form again.
        let datum = BasedRootDatum::from_simple_data(
            1,
            vec![vec![2]],
            vec![Weight::new(vec![2])],
            vec![Coweight::new(vec![1])],
        )
        .unwrap();
        let inner_class = InnerClass::new(
            datum.clone(),
            LatticeInvolution::identity(&datum).unwrap(),
            2,
        )
        .unwrap();
        let classification = CartanClassification::build(&inner_class, &budget(2)).unwrap();
        let identity = WeylElement::identity(inner_class.root_system()).unwrap();
        let longest = WeylElement::simple_reflection(inner_class.root_system(), 0).unwrap();
        let zero = Rational::from(0);
        let half = Rational::from(1) / Rational::from(2);

        assert_eq!(
            classification.real_form_of(&inner_class, &identity, std::slice::from_ref(&zero)),
            Ok(WeakRealFormId(0))
        );
        assert_eq!(
            classification.real_form_of(&inner_class, &identity, std::slice::from_ref(&half)),
            Ok(WeakRealFormId(1))
        );
        assert_eq!(
            classification.real_form_of(&inner_class, &longest, std::slice::from_ref(&zero)),
            Ok(WeakRealFormId(0))
        );

        // The form_number round trip: internal 0 (split) is external 1,
        // internal 1 (compact) external 0 — compact first, quasisplit last.
        let order = crate::ExternalFormOrder::build(&inner_class, &classification).unwrap();
        assert_eq!(order.external(WeakRealFormId(0)), Some(1));
        assert_eq!(order.external(WeakRealFormId(1)), Some(0));

        assert!(matches!(
            classification.real_form_of(&inner_class, &identity, &[zero.clone(), zero]),
            Err(StructureError::RankMismatch {
                expected: 1,
                actual: 2
            })
        ));
    }

    #[test]
    fn real_form_of_transports_along_complex_crosses_to_the_class_representative() {
        // Compact A2: the split Cartan class is the three-element cross
        // orbit {s0, s1, w0}, and it hosts the quasisplit form alone
        // (most-split uniqueness), so every one of its twisted involutions
        // with the zero factor must classify to form 0 — exercising the
        // complex-reflection transport from non-representative elements.
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let inner_class = InnerClass::new(
            datum.clone(),
            LatticeInvolution::identity(&datum).unwrap(),
            6,
        )
        .unwrap();
        let classification = CartanClassification::build(&inner_class, &budget(6)).unwrap();
        let zero = [Rational::from(0), Rational::from(0)];
        for word in [vec![0_usize], vec![1], vec![0, 1, 0]] {
            let mut element = WeylElement::identity(inner_class.root_system()).unwrap();
            for &generator in &word {
                element = element
                    .right_multiply_simple(inner_class.root_system(), generator)
                    .unwrap()
                    .0;
            }
            assert_eq!(
                classification.real_form_of(&inner_class, &element, &zero),
                Ok(WeakRealFormId(0)),
                "word {word:?}"
            );
        }
    }
}
