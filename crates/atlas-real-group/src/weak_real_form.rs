use crate::grading::try_capacity;
use crate::{
    AdjointCartanFiber, AdjointFiberElement, CartanGradingData, ModTwoVector, StructureError,
};

/// Sentinel for a mask the orbit walk has not classified yet.
const CLASS_SENTINEL: u32 = u32::MAX;

/// Masks are `u64`, so the adjoint fiber dimension must fit below the sign of
/// no bit at all — 63 keeps every `1 << dimension` shift in range.
const MAX_MASK_BITS: usize = 63;

/// Stable identifier for one weak real form of one Cartan involution.
///
/// Numbers are assigned by ascending minimal element of each `W_im` orbit in
/// canonical-coordinate integer order, so they are deterministic for this
/// crate but not Atlas numbering; the standing compatibility-adapter deferral
/// recorded in the root and grading designs covers them. Class 0 is the orbit
/// of the identity element, the quasisplit normalization. Ids minted by the
/// fundamental Cartan's partition additionally serve as the crate's global
/// real-form numbers, carried by [`crate::RealFormLabels`].
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WeakRealFormId(pub(crate) usize);

/// The partition of an adjoint Cartan fiber into `W_im` orbits.
///
/// Orbits correspond to the weak real forms of the inner class at this
/// Cartan involution. The partition owns its class table and one
/// deterministic representative element per class; real-form labels live in
/// [`crate::RealFormLabels`], while central square classes and strong real
/// forms are later layers.
#[derive(Clone, Debug)]
pub struct WeakRealFormPartition {
    adjoint: AdjointCartanFiber,
    class_of_by_mask: Vec<u32>,
    representatives: Vec<AdjointFiberElement>,
}

impl WeakRealFormPartition {
    /// Partition the adjoint fiber underlying validated grading tables.
    ///
    /// `max_elements` is the caller's bound on the enumerated fiber size
    /// `2^dimension`; the walk cost is bounded by that count times the
    /// imaginary rank, so no second knob exists.
    pub fn build(grading: &CartanGradingData, max_elements: usize) -> Result<Self, StructureError> {
        let adjoint = grading.adjoint_fiber();
        let dimension = adjoint.dimension();
        if dimension > MAX_MASK_BITS {
            return Err(StructureError::WeakRealFormResourceLimit {
                resource: "mask bits",
                limit: MAX_MASK_BITS,
            });
        }
        // The mask-bits gate above keeps this shift in range; the comparison
        // is deliberately unsaturated so `dimension == 64` cannot slip past
        // a `usize::MAX` limit on a 64-bit host.
        let required = 1_u128 << dimension;
        if required > widened(max_elements) {
            return Err(StructureError::WeakRealFormResourceLimit {
                resource: "fiber elements",
                limit: max_elements,
            });
        }
        let element_count =
            usize::try_from(required).map_err(|_| StructureError::ArithmeticOverflow)?;

        let imaginary_rank = grading.imaginary_rank();
        let mut base = try_capacity::<bool>(imaginary_rank)?;
        let mut alpha_columns = try_capacity::<u64>(imaginary_rank)?;
        let mut m_alpha_masks = try_capacity::<u64>(imaginary_rank)?;
        for imaginary_index in 0..imaginary_rank {
            base.push(
                grading
                    .base_grading()
                    .is_noncompact(imaginary_index)
                    .ok_or(StructureError::IndexOutOfRange {
                        index: imaginary_index,
                        upper_bound: imaginary_rank,
                    })?,
            );
            let mut column = 0_u64;
            for adjoint_basis_index in 0..dimension {
                let shift = grading.grading_shift(adjoint_basis_index).ok_or(
                    StructureError::IndexOutOfRange {
                        index: adjoint_basis_index,
                        upper_bound: dimension,
                    },
                )?;
                if shift.is_noncompact(imaginary_index) == Some(true) {
                    column |= 1_u64 << adjoint_basis_index;
                }
            }
            let element = grading.adjoint_m_alpha(imaginary_index).ok_or(
                StructureError::IndexOutOfRange {
                    index: imaginary_index,
                    upper_bound: imaginary_rank,
                },
            )?;
            let translation = mask_of(adjoint, element)?;
            // Involutivity of the generator is `<alpha_s, alpha_s_vee> = 2`
            // read mod two through validated grading tables; upstream only
            // ever assumes each generator permutes the fiber.
            debug_assert_eq!((translation & column).count_ones() % 2, 0);
            alpha_columns.push(column);
            m_alpha_masks.push(translation);
        }

        let mut class_of_by_mask = try_capacity::<u32>(element_count)?;
        class_of_by_mask.resize(element_count, CLASS_SENTINEL);
        let mut pending = try_capacity::<u64>(element_count)?;
        let mut class_count: usize = 0;
        for seed_index in 0..element_count {
            if class_of_by_mask[seed_index] != CLASS_SENTINEL {
                continue;
            }
            let class = seeded_class(class_count)?;
            class_of_by_mask[seed_index] = class;
            pending.push(mask_from_index(seed_index)?);
            while let Some(mask) = pending.pop() {
                for imaginary_index in 0..imaginary_rank {
                    let paired = (mask & alpha_columns[imaginary_index]).count_ones() & 1 == 1;
                    if base[imaginary_index] != paired {
                        // Noncompact at this root: translate by its m_alpha.
                        let image = mask ^ m_alpha_masks[imaginary_index];
                        let image_index = index_from_mask(image)?;
                        if class_of_by_mask[image_index] == CLASS_SENTINEL {
                            class_of_by_mask[image_index] = class;
                            pending.push(image);
                        }
                    }
                }
            }
            class_count = class_count
                .checked_add(1)
                .ok_or(StructureError::ArithmeticOverflow)?;
        }

        // Classes are numbered by ascending minimal mask, so the first
        // ascending occurrence of each class number is its representative.
        let mut representatives = try_capacity(class_count)?;
        for (index, &class) in class_of_by_mask.iter().enumerate() {
            let class = usize::try_from(class).map_err(|_| StructureError::ArithmeticOverflow)?;
            if class == representatives.len() {
                representatives.push(element_of(adjoint, mask_from_index(index)?)?);
            }
        }
        debug_assert_eq!(representatives.len(), class_count);

        Ok(Self {
            adjoint: adjoint.clone(),
            class_of_by_mask,
            representatives,
        })
    }

    /// The number of weak real forms at this Cartan involution.
    pub fn class_count(&self) -> usize {
        self.representatives.len()
    }

    /// The id-typed enumeration of `0..class_count()`.
    pub fn classes(&self) -> impl ExactSizeIterator<Item = WeakRealFormId> {
        (0..self.representatives.len()).map(WeakRealFormId)
    }

    /// The weak real form whose orbit contains this element.
    pub fn class_of(
        &self,
        element: &AdjointFiberElement,
    ) -> Result<WeakRealFormId, StructureError> {
        let mask = mask_of(&self.adjoint, element)?;
        let index = index_from_mask(mask)?;
        let class =
            self.class_of_by_mask
                .get(index)
                .copied()
                .ok_or(StructureError::IndexOutOfRange {
                    index,
                    upper_bound: self.class_of_by_mask.len(),
                })?;
        usize::try_from(class)
            .map_err(|_| StructureError::ArithmeticOverflow)
            .map(WeakRealFormId)
    }

    /// The minimal element of a class in canonical-coordinate integer order.
    /// Bounded by `class_count()`.
    pub fn class_representative(&self, class: WeakRealFormId) -> Option<&AdjointFiberElement> {
        self.representatives.get(class.0)
    }

    /// The class of the identity element: the quasisplit weak real form,
    /// which is class 0 under this crate's ascending-minimal numbering.
    pub fn quasisplit_class(&self) -> WeakRealFormId {
        WeakRealFormId(0)
    }

    /// The validated adjoint fiber this partition was computed against.
    pub fn adjoint_fiber(&self) -> &AdjointCartanFiber {
        &self.adjoint
    }
}

/// Convert the next class ordinal into a stored class number, rejecting the
/// sentinel. This guard is reachable in principle: a simply-connected product
/// has zero adjoint `m_alpha` vectors, so every mask is a singleton class.
fn seeded_class(class_count: usize) -> Result<u32, StructureError> {
    match u32::try_from(class_count) {
        Ok(class) if class != CLASS_SENTINEL => Ok(class),
        _ => Err(StructureError::WeakRealFormResourceLimit {
            resource: "classes",
            limit: CLASS_SENTINEL as usize,
        }),
    }
}

fn mask_of(
    fiber: &AdjointCartanFiber,
    element: &AdjointFiberElement,
) -> Result<u64, StructureError> {
    debug_assert!(fiber.dimension() <= MAX_MASK_BITS);
    let coordinates = fiber.coordinates(element)?;
    let mut mask = 0_u64;
    for index in 0..fiber.dimension() {
        if coordinates.bit(index) == Some(true) {
            mask |= 1_u64 << index;
        }
    }
    Ok(mask)
}

fn element_of(
    fiber: &AdjointCartanFiber,
    mask: u64,
) -> Result<AdjointFiberElement, StructureError> {
    let mut representative = ModTwoVector::zero(fiber.datum().rank())?;
    for (index, basis) in fiber.basis_representatives().iter().enumerate() {
        if mask & (1_u64 << index) != 0 {
            representative.xor_assign(basis)?;
        }
    }
    fiber.element_from_ambient(representative)
}

fn mask_from_index(index: usize) -> Result<u64, StructureError> {
    u64::try_from(index).map_err(|_| StructureError::ArithmeticOverflow)
}

fn index_from_mask(mask: u64) -> Result<usize, StructureError> {
    usize::try_from(mask).map_err(|_| StructureError::ArithmeticOverflow)
}

fn widened(value: usize) -> u128 {
    u128::try_from(value).unwrap_or(u128::MAX)
}

#[cfg(test)]
mod tests {
    use crate::integer_lattice::IntegerLatticeBudget;
    use crate::{
        AdjointFiberBudget, BasedRootDatum, CartanFiber, Coweight, LatticeInvolution,
        RootInvolutionData, RootSystem, Weight,
    };

    use super::*;

    fn integer_budget() -> IntegerLatticeBudget {
        IntegerLatticeBudget::new(64, 100_000, 100_000, 128)
    }

    fn adjoint_budget() -> AdjointFiberBudget {
        AdjointFiberBudget::new(integer_budget(), 50_000, 100_000)
    }

    fn grading_chain(
        datum: &BasedRootDatum,
        involution: LatticeInvolution,
        max_roots: usize,
    ) -> (AdjointCartanFiber, CartanGradingData) {
        let roots = RootSystem::enumerate(datum, max_roots).unwrap();
        let data = RootInvolutionData::new(&roots, involution.clone()).unwrap();
        let source = CartanFiber::build(&involution, &integer_budget()).unwrap();
        let adjoint = AdjointCartanFiber::build(&roots, &data, &source, &adjoint_budget()).unwrap();
        let grading = CartanGradingData::build(&roots, &data, &adjoint).unwrap();
        (adjoint, grading)
    }

    fn ambient(rank: usize, indices: impl IntoIterator<Item = usize>) -> ModTwoVector {
        ModTwoVector::from_ones(rank, indices).unwrap()
    }

    #[test]
    fn a2_identity_has_two_weak_real_forms() {
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let involution = LatticeInvolution::identity(&datum).unwrap();
        let (adjoint, grading) = grading_chain(&datum, involution, 6);
        let partition = WeakRealFormPartition::build(&grading, 4).unwrap();

        assert_eq!(partition.class_count(), 2);
        assert_eq!(partition.classes().len(), 2);
        let identity = adjoint.identity().unwrap();
        assert_eq!(
            partition.class_of(&identity).unwrap(),
            partition.quasisplit_class()
        );
        // The quasisplit orbit picks up both basis directions; the singleton
        // is their sum.
        for indices in [vec![0], vec![1]] {
            let element = adjoint.element_from_ambient(ambient(2, indices)).unwrap();
            assert_eq!(
                partition.class_of(&element).unwrap(),
                partition.quasisplit_class()
            );
        }
        let compact = adjoint.element_from_ambient(ambient(2, [0, 1])).unwrap();
        assert_eq!(partition.class_of(&compact).unwrap(), WeakRealFormId(1));
        for class in partition.classes() {
            let representative = partition.class_representative(class).unwrap();
            assert_eq!(partition.class_of(representative).unwrap(), class);
        }
    }

    #[test]
    fn b2_identity_has_three_weak_real_forms_in_the_pinned_order() {
        let datum = BasedRootDatum::standard(vec![vec![2, -2], vec![-1, 2]]).unwrap();
        let involution = LatticeInvolution::identity(&datum).unwrap();
        let (adjoint, grading) = grading_chain(&datum, involution, 8);
        let partition = WeakRealFormPartition::build(&grading, 4).unwrap();

        assert_eq!(partition.class_count(), 3);
        let expected = [ambient(2, []), ambient(2, [0]), ambient(2, [0, 1])];
        for (class, representative) in partition.classes().zip(&expected) {
            assert_eq!(
                &adjoint
                    .canonical_representative(partition.class_representative(class).unwrap())
                    .unwrap(),
                representative
            );
        }
        assert_eq!(
            partition.class_of(&adjoint.identity().unwrap()).unwrap(),
            partition.quasisplit_class()
        );
    }

    #[test]
    fn simply_connected_a1_has_a_trivial_action_and_singleton_classes() {
        let datum = BasedRootDatum::from_simple_data(
            1,
            vec![vec![2]],
            vec![Weight::new(vec![2])],
            vec![Coweight::new(vec![1])],
        )
        .unwrap();
        let involution = LatticeInvolution::identity(&datum).unwrap();
        let (adjoint, grading) = grading_chain(&datum, involution, 2);
        let partition = WeakRealFormPartition::build(&grading, 2).unwrap();

        assert_eq!(partition.class_count(), 2);
        let other = adjoint.element_from_ambient(ambient(1, [0])).unwrap();
        assert_eq!(partition.class_of(&other).unwrap(), WeakRealFormId(1));
    }

    #[test]
    fn the_a2_diagram_twist_has_one_weak_real_form() {
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let twist = LatticeInvolution::new(
            &datum,
            vec![vec![0, 1], vec![1, 0]],
            vec![vec![0, 1], vec![1, 0]],
        )
        .unwrap();
        let (adjoint, grading) = grading_chain(&datum, twist, 6);
        let partition = WeakRealFormPartition::build(&grading, 1).unwrap();

        assert_eq!(partition.class_count(), 1);
        let identity = adjoint.identity().unwrap();
        assert_eq!(
            partition.class_representative(partition.quasisplit_class()),
            Some(&identity)
        );
        assert!(partition.class_representative(WeakRealFormId(1)).is_none());
    }

    #[test]
    fn rejects_foreign_elements_and_undersized_budgets() {
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let involution = LatticeInvolution::identity(&datum).unwrap();
        let (_, grading) = grading_chain(&datum, involution.clone(), 6);
        let partition = WeakRealFormPartition::build(&grading, 4).unwrap();

        let (foreign_adjoint, _) = grading_chain(&datum, involution, 6);
        let foreign = foreign_adjoint.identity().unwrap();
        assert_eq!(
            partition.class_of(&foreign),
            Err(StructureError::CartanFiberMismatch)
        );

        assert!(matches!(
            WeakRealFormPartition::build(&grading, 3),
            Err(StructureError::WeakRealFormResourceLimit {
                resource: "fiber elements",
                limit: 3,
            })
        ));
    }

    #[test]
    fn large_products_are_rejected_by_honest_budgets_not_rank_caps() {
        let rank = 33;
        let mut cartan = vec![vec![0; rank]; rank];
        for (index, row) in cartan.iter_mut().enumerate() {
            row[index] = 2;
        }
        let datum = BasedRootDatum::standard(cartan).unwrap();
        let involution = LatticeInvolution::identity(&datum).unwrap();
        let (_, grading) = grading_chain(&datum, involution, 2 * rank);
        assert!(matches!(
            WeakRealFormPartition::build(&grading, 1_000_000),
            Err(StructureError::WeakRealFormResourceLimit {
                resource: "fiber elements",
                limit: 1_000_000,
            })
        ));

        let rank = 64;
        let mut cartan = vec![vec![0; rank]; rank];
        for (index, row) in cartan.iter_mut().enumerate() {
            row[index] = 2;
        }
        let datum = BasedRootDatum::standard(cartan).unwrap();
        let involution = LatticeInvolution::identity(&datum).unwrap();
        // The upstream fiber budgets must be generous enough that the
        // mask-bits gate is genuinely what rejects this rank.
        let roots = RootSystem::enumerate(&datum, 2 * rank).unwrap();
        let data = RootInvolutionData::new(&roots, involution.clone()).unwrap();
        let source = CartanFiber::build(&involution, &integer_budget()).unwrap();
        let adjoint = AdjointCartanFiber::build(
            &roots,
            &data,
            &source,
            &AdjointFiberBudget::new(integer_budget(), 200_000, 1_000_000),
        )
        .unwrap();
        let grading = CartanGradingData::build(&roots, &data, &adjoint).unwrap();
        assert!(matches!(
            WeakRealFormPartition::build(&grading, usize::MAX),
            Err(StructureError::WeakRealFormResourceLimit {
                resource: "mask bits",
                limit: MAX_MASK_BITS,
            })
        ));
    }

    #[test]
    fn the_class_number_guard_rejects_the_sentinel_ordinal() {
        assert_eq!(seeded_class(5), Ok(5));
        assert_eq!(
            seeded_class(usize::try_from(u32::MAX).unwrap()),
            Err(StructureError::WeakRealFormResourceLimit {
                resource: "classes",
                limit: usize::try_from(u32::MAX).unwrap(),
            })
        );
    }
}
