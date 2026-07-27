use malachite::base::num::arithmetic::traits::DivisibleBy;
use malachite::{Integer, Rational};

use crate::grading::try_capacity;
use crate::root_system::combine_roots;
use crate::twisted_involution::compose_matrices;
use crate::{
    CartanGradingData, CayleyCrossDecomposition, InnerClass, ModTwoSubspace, ModTwoVector, RootId,
    RootInvolutionData, RootKind, RootSystem, StructureError, WeakRealFormId,
    WeakRealFormPartition,
};

/// Per-Cartan real-form labels: the map from a Cartan's weak-real classes to
/// the fundamental partition's classes, which are the inner class's global
/// real-form numbers.
///
/// Inputs to [`Self::label`] are the Cartan's LOCAL classes; outputs are
/// classes of the FUNDAMENTAL partition. The correlation is the grading-based
/// legacy mechanism: pull each local class representative's grading back
/// through the Cayley set, extend all-noncompact over it, transport the root
/// list by the cross action, and solve in the fundamental fiber.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RealFormLabels {
    labels: Vec<WeakRealFormId>,
}

impl RealFormLabels {
    /// Correlate one Cartan's weak-real classes with the fundamental ones.
    ///
    /// Provenance gates run before any computation: the fundamental grading
    /// must sit at this inner class's distinguished involution, the Cartan
    /// grading at the decomposition's composed involution, the decomposition
    /// must factor through this distinguished involution, and each partition
    /// must share its grading data's fiber.
    pub fn build(
        inner_class: &InnerClass,
        fundamental_grading: &CartanGradingData,
        fundamental_partition: &WeakRealFormPartition,
        cartan_grading: &CartanGradingData,
        cartan_partition: &WeakRealFormPartition,
        decomposition: &CayleyCrossDecomposition,
    ) -> Result<Self, StructureError> {
        let root_system = inner_class.root_system();
        let delta_data = inner_class.distinguished_involution();
        let twisted = decomposition.twisted_involution();
        if twisted.weyl_action().datum() != inner_class.datum() {
            return Err(StructureError::DatumMismatch);
        }
        if fundamental_grading
            .adjoint_fiber()
            .ambient_fiber()
            .involution()
            != delta_data.involution()
        {
            return Err(StructureError::CartanFiberInvolutionMismatch);
        }
        if cartan_grading.adjoint_fiber().ambient_fiber().involution()
            != twisted.root_involution().involution()
        {
            return Err(StructureError::CartanFiberInvolutionMismatch);
        }
        let stored = twisted.root_involution().involution();
        if compose_matrices(
            twisted.weyl_action().matrix(),
            delta_data.involution().weight_matrix(),
        )? != stored.weight_matrix()
            || compose_matrices(
                twisted.weyl_action().coweight_matrix(),
                delta_data.involution().coweight_matrix(),
            )? != stored.coweight_matrix()
        {
            return Err(StructureError::DistinguishedInvolutionMismatch);
        }
        // Partition-fiber identity probes: a foreign fiber fails through the
        // element-provenance path in constant time.
        fundamental_partition.class_of(&fundamental_grading.adjoint_fiber().identity()?)?;
        cartan_partition.class_of(&cartan_grading.adjoint_fiber().identity()?)?;

        let cartan_imaginary = cartan_grading.imaginary_simple_roots();
        let cayley = decomposition.cayley_roots();
        let local_rank = cartan_grading.imaginary_rank();
        let list_len = local_rank
            .checked_add(cayley.len())
            .ok_or(StructureError::ArithmeticOverflow)?;

        // Cayley pullback flips, computed at the Cartan side: bit i flips
        // when rl[i] + alpha is a root for an odd number of Cayley alphas.
        let mut flips = try_capacity(local_rank)?;
        for &root in cartan_imaginary {
            let mut flip = false;
            for &alpha in cayley {
                if combine_roots(root_system, root, alpha, false)?.is_some() {
                    flip = !flip;
                }
            }
            flips.push(flip);
        }

        // Transport the root list by the cross action; every image must be
        // imaginary for the distinguished involution.
        let mut transported = try_capacity(list_len)?;
        for &root in cartan_imaginary.iter().chain(cayley.iter()) {
            let weight = root_system
                .root(root)
                .ok_or(StructureError::IndexOutOfRange {
                    index: root.0,
                    upper_bound: root_system.roots().len(),
                })?;
            let image_weight = decomposition.cross_action().act(weight)?;
            let image = root_system
                .id_of(&image_weight)
                .ok_or(StructureError::InvalidRootAutomorphism)?;
            if delta_data.kind(image) != Some(RootKind::Imaginary) {
                return Err(StructureError::RealFormLabelInvariantViolation {
                    invariant: "fundamental imaginary",
                });
            }
            transported.push(image);
        }

        // The fundamental base grading extended by mod-2 linearity, and the
        // fundamental shift columns restricted to the transported list.
        let mut base_bits = try_capacity(list_len)?;
        for &image in &transported {
            base_bits.push(base_grading_extension(root_system, delta_data, image)?);
        }
        let fundamental_fiber = fundamental_grading.adjoint_fiber();
        let dimension = fundamental_fiber.dimension();
        let semisimple_rank = inner_class.datum().semisimple_rank();
        let augmented = list_len
            .checked_add(dimension)
            .ok_or(StructureError::ArithmeticOverflow)?;
        let mut span = ModTwoSubspace::new(augmented)?;
        for (adjoint_basis_index, representative) in
            fundamental_fiber.basis_representatives().iter().enumerate()
        {
            let mut ones = try_capacity(augmented)?;
            for (position, &image) in transported.iter().enumerate() {
                if simple_mod_two(root_system, image, semisimple_rank)?.dot(representative)? {
                    ones.push(position);
                }
            }
            ones.push(list_len + adjoint_basis_index);
            span.insert(ModTwoVector::from_ones(augmented, ones)?)?;
        }

        let mut labels = try_capacity(cartan_partition.class_count())?;
        for class in cartan_partition.classes() {
            let representative = cartan_partition.class_representative(class).ok_or(
                StructureError::IndexOutOfRange {
                    index: class.0,
                    upper_bound: cartan_partition.class_count(),
                },
            )?;
            let grading = cartan_grading.grading(representative)?;
            // Right-hand side: (pulled-back grading, then all-noncompact over
            // the Cayley block) XOR the extended base — explicitly, since the
            // base is not all-ones here.
            let mut difference = try_capacity(list_len)?;
            for position in 0..list_len {
                let bit = if position < local_rank {
                    grading
                        .is_noncompact(position)
                        .ok_or(StructureError::IndexOutOfRange {
                            index: position,
                            upper_bound: local_rank,
                        })?
                        ^ flips[position]
                } else {
                    true
                };
                if bit != base_bits[position] {
                    difference.push(position);
                }
            }
            let remainder =
                span.quotient_representative(ModTwoVector::from_ones(augmented, difference)?)?;
            if (0..list_len).any(|position| remainder.bit(position) == Some(true)) {
                return Err(StructureError::ImpossibleGrading);
            }
            let ambient_rank = fundamental_fiber.datum().rank();
            let mut ambient = ModTwoVector::zero(ambient_rank)?;
            for (adjoint_basis_index, basis_representative) in
                fundamental_fiber.basis_representatives().iter().enumerate()
            {
                if remainder.bit(list_len + adjoint_basis_index) == Some(true) {
                    ambient.xor_assign(basis_representative)?;
                }
            }
            let element = fundamental_fiber.element_from_ambient(ambient)?;
            labels.push(fundamental_partition.class_of(&element)?);
        }

        if labels.first() != Some(&fundamental_partition.quasisplit_class()) {
            return Err(StructureError::RealFormLabelInvariantViolation {
                invariant: "quasisplit anchor",
            });
        }
        Ok(Self { labels })
    }

    /// Position `k` is the label of the Cartan's local class `k`, in
    /// `classes()` order.
    pub fn labels(&self) -> &[WeakRealFormId] {
        &self.labels
    }

    /// The fundamental class of one local class. Bounded by the Cartan's
    /// local class count.
    pub fn label(&self, local_class: WeakRealFormId) -> Option<WeakRealFormId> {
        self.labels.get(local_class.0).copied()
    }
}

/// The fundamental base grading at an arbitrary imaginary root: noncompact
/// iff the coefficient sum over the simple-imaginary basis is odd.
///
/// The coefficients solve the TRANSPOSED bracket-indexed subsystem Cartan
/// system (`row j` pairs every basis root against coroot `j`), exactly over
/// the rationals; the solution is integral for genuine imaginary roots.
/// Integrality is not an imaginarity test — a non-imaginary root's
/// projection can be integral — so the root kind is gated explicitly.
pub(crate) fn base_grading_extension(
    root_system: &RootSystem,
    involution_data: &RootInvolutionData,
    root: RootId,
) -> Result<bool, StructureError> {
    if root_system.datum() != involution_data.involution().datum() {
        return Err(StructureError::DatumMismatch);
    }
    if involution_data.kind(root) != Some(RootKind::Imaginary) {
        return Err(StructureError::RealFormLabelInvariantViolation {
            invariant: "imaginary subsystem solve",
        });
    }
    let basis = involution_data.imaginary_simple_roots();
    let size = basis.len();
    let mut rows: Vec<Vec<Rational>> = try_capacity(size)?;
    for &coroot_side in basis {
        let mut row = try_capacity(size + 1)?;
        for &basis_root in basis {
            row.push(Rational::from(
                root_system.bracket(basis_root, coroot_side)?,
            ));
        }
        row.push(Rational::from(root_system.bracket(root, coroot_side)?));
        rows.push(row);
    }
    for column in 0..size {
        let pivot_row = (column..size).find(|&row| rows[row][column] != 0).ok_or(
            StructureError::RealFormLabelInvariantViolation {
                invariant: "imaginary subsystem solve",
            },
        )?;
        rows.swap(column, pivot_row);
        let pivot = rows[column][column].clone();
        for entry in rows[column].iter_mut() {
            *entry = entry.clone() / pivot.clone();
        }
        let pivot_values = rows[column].clone();
        for (row, values) in rows.iter_mut().enumerate() {
            if row == column || values[column] == 0 {
                continue;
            }
            let factor = values[column].clone();
            for (entry, pivot_entry) in values.iter_mut().zip(&pivot_values) {
                *entry -= pivot_entry.clone() * factor.clone();
            }
        }
    }
    let two = Integer::from(2);
    let mut parity = false;
    for row in &rows {
        let value = Integer::try_from(row[size].clone()).map_err(|_| {
            StructureError::RealFormLabelInvariantViolation {
                invariant: "imaginary subsystem solve",
            }
        })?;
        if !(&value).divisible_by(&two) {
            parity = !parity;
        }
    }
    Ok(parity)
}

/// Mod-2 full-simple coordinates of a root.
fn simple_mod_two(
    root_system: &RootSystem,
    root: RootId,
    semisimple_rank: usize,
) -> Result<ModTwoVector, StructureError> {
    let coordinates =
        root_system
            .simple_coordinates(root)
            .ok_or(StructureError::IndexOutOfRange {
                index: root.0,
                upper_bound: root_system.roots().len(),
            })?;
    let mut odd = try_capacity(semisimple_rank)?;
    for (index, coordinate) in coordinates.iter().enumerate() {
        if *coordinate % 2 != 0 {
            odd.push(index);
        }
    }
    ModTwoVector::from_ones(semisimple_rank, odd)
}

#[cfg(test)]
mod tests {
    use crate::integer_lattice::IntegerLatticeBudget;
    use crate::{
        AdjointCartanFiber, AdjointFiberBudget, BasedRootDatum, CartanFiber, Coweight,
        LatticeInvolution, TwistedInvolution, Weight, WeylAction, WeylGroup,
    };

    use super::*;

    fn integer_budget() -> IntegerLatticeBudget {
        IntegerLatticeBudget::new(64, 100_000, 100_000, 128)
    }

    fn adjoint_budget() -> AdjointFiberBudget {
        AdjointFiberBudget::new(integer_budget(), 50_000, 100_000)
    }

    fn site(
        inner_class: &InnerClass,
        twisted: &TwistedInvolution,
    ) -> (
        CartanGradingData,
        WeakRealFormPartition,
        CayleyCrossDecomposition,
    ) {
        let root_system = inner_class.root_system();
        let data = twisted.root_involution();
        let involution = data.involution().clone();
        let source = CartanFiber::build(&involution, &integer_budget()).unwrap();
        let adjoint =
            AdjointCartanFiber::build(root_system, data, &source, &adjoint_budget()).unwrap();
        let grading = CartanGradingData::build(root_system, data, &adjoint).unwrap();
        let partition = WeakRealFormPartition::build(&grading, 1 << adjoint.dimension()).unwrap();
        let decomposition = CayleyCrossDecomposition::build(inner_class, twisted, 64).unwrap();
        (grading, partition, decomposition)
    }

    fn twisted(inner_class: &InnerClass, action: WeylAction) -> TwistedInvolution {
        TwistedInvolution::new(
            inner_class.datum(),
            inner_class.root_system(),
            inner_class.distinguished_involution().involution(),
            action,
        )
        .unwrap()
    }

    #[test]
    fn a2_reflection_cartan_labels_only_the_quasisplit_form() {
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let inner_class = InnerClass::new(
            datum.clone(),
            LatticeInvolution::identity(&datum).unwrap(),
            6,
        )
        .unwrap();
        let group = WeylGroup::new(datum);
        let fundamental = twisted(&inner_class, group.identity().unwrap());
        let (fund_grading, fund_partition, _) = site(&inner_class, &fundamental);
        let reflection = twisted(&inner_class, group.simple_reflection(0).unwrap());
        let (cartan_grading, cartan_partition, decomposition) = site(&inner_class, &reflection);

        let labels = RealFormLabels::build(
            &inner_class,
            &fund_grading,
            &fund_partition,
            &cartan_grading,
            &cartan_partition,
            &decomposition,
        )
        .unwrap();
        assert_eq!(fund_partition.class_count(), 2);
        assert_eq!(labels.labels(), &[fund_partition.quasisplit_class()]);
    }

    #[test]
    fn the_fundamental_cartan_labels_are_the_identity_map() {
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let inner_class = InnerClass::new(
            datum.clone(),
            LatticeInvolution::identity(&datum).unwrap(),
            6,
        )
        .unwrap();
        let fundamental = twisted(&inner_class, WeylGroup::new(datum).identity().unwrap());
        let (grading, partition, decomposition) = site(&inner_class, &fundamental);

        let labels = RealFormLabels::build(
            &inner_class,
            &grading,
            &partition,
            &grading,
            &partition,
            &decomposition,
        )
        .unwrap();
        let expected: Vec<WeakRealFormId> = partition.classes().collect();
        assert_eq!(labels.labels(), expected.as_slice());
    }

    #[test]
    fn simply_connected_a1_split_cartan_labels_quasisplit() {
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
        let group = WeylGroup::new(datum);
        let fundamental = twisted(&inner_class, group.identity().unwrap());
        let (fund_grading, fund_partition, _) = site(&inner_class, &fundamental);
        assert_eq!(fund_partition.class_count(), 2);
        let split = twisted(&inner_class, group.simple_reflection(0).unwrap());
        let (cartan_grading, cartan_partition, decomposition) = site(&inner_class, &split);

        let labels = RealFormLabels::build(
            &inner_class,
            &fund_grading,
            &fund_partition,
            &cartan_grading,
            &cartan_partition,
            &decomposition,
        )
        .unwrap();
        assert_eq!(labels.labels(), &[fund_partition.quasisplit_class()]);
    }

    #[test]
    fn every_cartan_of_b2_and_twisted_a2_correlates_with_the_anchor() {
        let cases = [
            (
                BasedRootDatum::standard(vec![vec![2, -2], vec![-1, 2]]).unwrap(),
                LatticeInvolution::identity(
                    &BasedRootDatum::standard(vec![vec![2, -2], vec![-1, 2]]).unwrap(),
                )
                .unwrap(),
                8,
            ),
            (
                BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap(),
                LatticeInvolution::new(
                    &BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap(),
                    vec![vec![0, 1], vec![1, 0]],
                    vec![vec![0, 1], vec![1, 0]],
                )
                .unwrap(),
                6,
            ),
        ];
        for (datum, distinguished, budget) in cases {
            let inner_class = InnerClass::new(datum.clone(), distinguished, budget).unwrap();
            let fundamental = twisted(
                &inner_class,
                WeylGroup::new(datum.clone()).identity().unwrap(),
            );
            let (fund_grading, fund_partition, _) = site(&inner_class, &fundamental);
            for candidate in inner_class.twisted_involutions(budget).unwrap() {
                let (cartan_grading, cartan_partition, decomposition) =
                    site(&inner_class, &candidate);
                let labels = RealFormLabels::build(
                    &inner_class,
                    &fund_grading,
                    &fund_partition,
                    &cartan_grading,
                    &cartan_partition,
                    &decomposition,
                )
                .unwrap();
                assert_eq!(labels.labels().len(), cartan_partition.class_count());
                assert_eq!(
                    labels.label(cartan_partition.quasisplit_class()),
                    Some(fund_partition.quasisplit_class())
                );
                for &label in labels.labels() {
                    assert!(label.0 < fund_partition.class_count());
                }
            }
        }
    }

    #[test]
    fn rejects_interleaved_and_foreign_inputs() {
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let inner_class = InnerClass::new(
            datum.clone(),
            LatticeInvolution::identity(&datum).unwrap(),
            6,
        )
        .unwrap();
        let group = WeylGroup::new(datum.clone());
        let fundamental = twisted(&inner_class, group.identity().unwrap());
        let (fund_grading, fund_partition, fund_decomposition) = site(&inner_class, &fundamental);
        let reflection = twisted(&inner_class, group.simple_reflection(0).unwrap());
        let (cartan_grading, cartan_partition, decomposition) = site(&inner_class, &reflection);

        // Cartan grading paired with the fundamental decomposition: involution
        // mismatch.
        assert!(matches!(
            RealFormLabels::build(
                &inner_class,
                &fund_grading,
                &fund_partition,
                &cartan_grading,
                &cartan_partition,
                &fund_decomposition,
            ),
            Err(StructureError::CartanFiberInvolutionMismatch)
        ));

        // Interleaved partition from a separately built chain: foreign fiber.
        let (_, foreign_partition, _) = site(&inner_class, &reflection);
        assert!(matches!(
            RealFormLabels::build(
                &inner_class,
                &fund_grading,
                &fund_partition,
                &cartan_grading,
                &foreign_partition,
                &decomposition,
            ),
            Err(StructureError::CartanFiberMismatch)
        ));

        // Decomposition peeled against a different distinguished involution.
        let twist = LatticeInvolution::new(
            &datum,
            vec![vec![0, 1], vec![1, 0]],
            vec![vec![0, 1], vec![1, 0]],
        )
        .unwrap();
        let twist_class = InnerClass::new(datum.clone(), twist, 6).unwrap();
        let twist_fundamental = twisted(
            &twist_class,
            WeylGroup::new(datum.clone()).identity().unwrap(),
        );
        let (_, _, twist_decomposition) = site(&twist_class, &twist_fundamental);
        assert!(matches!(
            RealFormLabels::build(
                &inner_class,
                &fund_grading,
                &fund_partition,
                &cartan_grading,
                &cartan_partition,
                &twist_decomposition,
            ),
            Err(StructureError::CartanFiberInvolutionMismatch)
        ));
    }

    #[test]
    fn the_base_extension_matches_hand_computed_parities() {
        let a2 = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let a2_class =
            InnerClass::new(a2.clone(), LatticeInvolution::identity(&a2).unwrap(), 6).unwrap();
        let a2_roots = a2_class.root_system();
        let delta = a2_class.distinguished_involution();
        let highest = a2_roots.id_of(&Weight::new(vec![1, 1])).unwrap();
        assert_eq!(base_grading_extension(a2_roots, delta, highest), Ok(false));
        let simple = a2_roots.id_of(&Weight::new(vec![1, 0])).unwrap();
        assert_eq!(base_grading_extension(a2_roots, delta, simple), Ok(true));

        let b2 = BasedRootDatum::standard(vec![vec![2, -2], vec![-1, 2]]).unwrap();
        let b2_class =
            InnerClass::new(b2.clone(), LatticeInvolution::identity(&b2).unwrap(), 8).unwrap();
        let b2_roots = b2_class.root_system();
        let b2_delta = b2_class.distinguished_involution();
        let long_short = b2_roots.id_of(&Weight::new(vec![1, 1])).unwrap();
        assert_eq!(
            base_grading_extension(b2_roots, b2_delta, long_short),
            Ok(false)
        );
        let doubled = b2_roots.id_of(&Weight::new(vec![1, 2])).unwrap();
        assert_eq!(
            base_grading_extension(b2_roots, b2_delta, doubled),
            Ok(true)
        );

        // A real root is rejected, not silently graded.
        let s0_class = twisted(
            &a2_class,
            WeylGroup::new(a2.clone()).simple_reflection(0).unwrap(),
        );
        assert_eq!(
            base_grading_extension(
                a2_roots,
                s0_class.root_involution(),
                a2_roots.id_of(&Weight::new(vec![1, 0])).unwrap()
            ),
            Err(StructureError::RealFormLabelInvariantViolation {
                invariant: "imaginary subsystem solve",
            })
        );
    }
}
