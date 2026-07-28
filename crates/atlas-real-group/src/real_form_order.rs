//! The external (interpreter-facing) numbering of weak real forms.
//!
//! Upstream Atlas renumbers real forms for output through `FormNumberMap`
//! (output.cpp:71-140): forms sort by ascending DEPTH — the size of a
//! maximal orthogonal set of noncompact imaginary roots at the distinguished
//! involution (gradings.cpp:51-76) — with ties broken by the PARTITION
//! overload of `specialGrading` (cartanclass.cpp:929-948) compared as an
//! unsigned bitset over the twist-fixed simple generators (simple root 0 =
//! least significant). The compact form (depth zero) is external 0 and the
//! quasisplit form external last. Upstream sorts with an UNSTABLE
//! `std::sort`, so this port ASSERTS strict `(depth, grading)` ordering and
//! reports any tie as a loud invariant violation instead of silently picking
//! an order.

use std::collections::BTreeSet;

use malachite::base::num::arithmetic::traits::Floor;
use malachite::base::num::basic::traits::Zero;
use malachite::{Integer, Rational};

use malachite::base::num::arithmetic::traits::DivisibleBy;

use crate::grading::try_capacity;
use crate::real_form_seed::invert_rational;
use crate::weak_real_form::MAX_MASK_BITS;
use crate::{
    AdjointFiberElement, CartanClassification, CartanGradingData, CartanId, InnerClass,
    ModTwoVector, RootId, RootKind, StructureError, WeakRealFormId, WeakRealFormPartition, Weight,
};

/// The most significant simple-generator position the tiebreak key can hold.
const MAX_KEY_GENERATORS: usize = 127;

/// The bijection between this crate's internal weak-real-form numbers and
/// the upstream external output numbering of one inner class.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExternalFormOrder {
    external_to_internal: Vec<WeakRealFormId>,
    internal_to_external: Vec<usize>,
}

impl ExternalFormOrder {
    /// Compute the external order for a classification of this inner class.
    pub fn build(
        inner_class: &InnerClass,
        classification: &CartanClassification,
    ) -> Result<Self, StructureError> {
        let fundamental = classification.cartan_class(CartanId(0)).ok_or(
            StructureError::RealFormOrderInvariantViolation {
                invariant: "fundamental class",
            },
        )?;
        if fundamental.representative().weyl_action().datum() != inner_class.datum() {
            return Err(StructureError::DatumMismatch);
        }
        let grading = fundamental.grading();
        let weak = fundamental.partition();
        let form_count = weak.class_count();
        if form_count != classification.weak_real_form_count() {
            return Err(StructureError::RealFormOrderInvariantViolation {
                invariant: "fundamental form count",
            });
        }

        let depth_tables = DepthTables::build(inner_class, grading)?;
        let generator_of_bit = verified_generator_map(inner_class, grading)?;

        let mut keys = try_capacity(form_count)?;
        for internal in weak.classes() {
            let representative = weak.class_representative(internal).ok_or(
                StructureError::RealFormOrderInvariantViolation {
                    invariant: "form representative",
                },
            )?;
            let depth = depth_tables.depth(inner_class, grading, representative)?;
            let tiebreak = special_grading_key(grading, weak, internal, &generator_of_bit)?;
            keys.push((depth, tiebreak, internal));
        }
        keys.sort_by_key(|left| (left.0, left.1));
        for window in keys.windows(2) {
            if (window[0].0, window[0].1) == (window[1].0, window[1].1) {
                return Err(StructureError::RealFormOrderInvariantViolation {
                    invariant: "strict (depth, grading) order",
                });
            }
        }
        if keys
            .last()
            .is_some_and(|&(_, _, internal)| internal != weak.quasisplit_class())
        {
            return Err(StructureError::RealFormOrderInvariantViolation {
                invariant: "quasisplit last",
            });
        }

        let mut external_to_internal = try_capacity(form_count)?;
        let mut internal_to_external = try_capacity(form_count)?;
        internal_to_external.resize(form_count, 0_usize);
        for (external, &(_, _, internal)) in keys.iter().enumerate() {
            external_to_internal.push(internal);
            internal_to_external[internal.0] = external;
        }
        Ok(Self {
            external_to_internal,
            internal_to_external,
        })
    }

    pub fn form_count(&self) -> usize {
        self.external_to_internal.len()
    }

    /// The internal weak-real-form id at one external number.
    pub fn internal(&self, external: usize) -> Option<WeakRealFormId> {
        self.external_to_internal.get(external).copied()
    }

    /// The external number of one internal weak-real-form id.
    pub fn external(&self, internal: WeakRealFormId) -> Option<usize> {
        self.internal_to_external.get(internal.0).copied()
    }

    /// The external number of the quasisplit form (always last).
    pub fn quasisplit_external(&self) -> usize {
        self.form_count().saturating_sub(1)
    }
}

/// Precomputed per-inner-class data for the depth computation: the positive
/// imaginary roots at the distinguished involution, their datum-simple
/// coordinate parities, and their coefficient-sum parities on the
/// imaginary-simple basis (the linear extension of the all-ones base
/// grading, upstream `makeBaseGrading`).
struct DepthTables {
    positive_imaginary: Vec<RootId>,
    base_noncompact_parity: Vec<bool>,
}

impl DepthTables {
    fn build(
        inner_class: &InnerClass,
        grading: &CartanGradingData,
    ) -> Result<Self, StructureError> {
        let root_system = inner_class.root_system();
        let delta = inner_class.distinguished_involution();
        let imaginary_simples = grading.imaginary_simple_roots();
        let imaginary_rank = imaginary_simples.len();

        // `M = C^T` of the imaginary subsystem: `<beta, alpha_j_vee>` over
        // the imaginary-simple coordinates `c` reads `p = M c`.
        let mut transposed = try_capacity(imaginary_rank)?;
        for &row_root in imaginary_simples {
            let mut row = try_capacity(imaginary_rank)?;
            for &column_root in imaginary_simples {
                row.push(Rational::from(root_system.bracket(column_root, row_root)?));
            }
            transposed.push(row);
        }
        let inverse_columns = invert_rational(&transposed)?;

        let mut positive_imaginary = Vec::new();
        let mut base_noncompact_parity = Vec::new();
        for root in delta.roots_of_kind(RootKind::Imaginary) {
            if root_system.is_positive(root) != Some(true) {
                continue;
            }
            let mut coefficient_sum = Rational::ZERO;
            for (index, column) in inverse_columns.iter().enumerate() {
                let pairing = Rational::from(root_system.bracket(root, imaginary_simples[index])?);
                for (slot, value) in column.iter().enumerate() {
                    let _ = slot;
                    let _ = value;
                }
                // c_i = sum_j inverse[i][j] * p_j; only the total sum is
                // needed, so accumulate `p_index * sum_i inverse[i][index]`.
                let mut column_sum = Rational::ZERO;
                for value in column {
                    column_sum += value;
                }
                coefficient_sum += pairing * column_sum;
            }
            let floored = Rational::from(coefficient_sum.clone().floor());
            if floored != coefficient_sum {
                return Err(StructureError::RealFormOrderInvariantViolation {
                    invariant: "integral imaginary-simple coordinates",
                });
            }
            let parity = !coefficient_sum.floor().divisible_by(&Integer::from(2));
            positive_imaginary.push(root);
            base_noncompact_parity.push(parity);
        }
        Ok(Self {
            positive_imaginary,
            base_noncompact_parity,
        })
    }

    /// The depth of one form: the `gradings.cpp` greedy maximal orthogonal
    /// set over the form's full noncompact positive-imaginary root set.
    fn depth(
        &self,
        inner_class: &InnerClass,
        grading: &CartanGradingData,
        representative: &AdjointFiberElement,
    ) -> Result<usize, StructureError> {
        let root_system = inner_class.root_system();
        let ambient = grading
            .adjoint_fiber()
            .canonical_representative(representative)?;
        let mut noncompact = BTreeSet::new();
        for (index, &root) in self.positive_imaginary.iter().enumerate() {
            let shifted = self.base_noncompact_parity[index]
                ^ parity_dot(&ambient, simple_coordinates(root_system, root)?)?;
            if shifted {
                noncompact.insert(root);
            }
        }

        let mut working = self.positive_imaginary.clone();
        let mut count = 0_usize;
        while let Some(pick) = working
            .iter()
            .copied()
            .find(|candidate| noncompact.contains(candidate))
        {
            count = count
                .checked_add(1)
                .ok_or(StructureError::ArithmeticOverflow)?;
            let pick_weight = root_system
                .root(pick)
                .ok_or(StructureError::RealFormOrderInvariantViolation {
                    invariant: "root lookup",
                })?
                .clone();
            let mut retained = Vec::new();
            for candidate in working {
                if candidate == pick {
                    continue;
                }
                if root_system.bracket(candidate, pick)? != 0 {
                    // Not orthogonal to the pick: dropped entirely.
                    noncompact.remove(&candidate);
                    continue;
                }
                // Orthogonal but summing to a root (short-root pairs in the
                // non-simply-laced types): the pick FLIPS its compactness.
                let candidate_weight = root_system.root(candidate).ok_or(
                    StructureError::RealFormOrderInvariantViolation {
                        invariant: "root lookup",
                    },
                )?;
                if let Some(sum) = weight_sum(candidate_weight, &pick_weight)? {
                    if root_system.id_of(&sum).is_some() {
                        if noncompact.contains(&candidate) {
                            noncompact.remove(&candidate);
                        } else {
                            noncompact.insert(candidate);
                        }
                    }
                }
                retained.push(candidate);
            }
            working = retained;
        }
        Ok(count)
    }
}

/// Verify the port-side coordinate parity the complement trick rides on:
/// every adjoint-fiber basis bit must flip exactly ONE simple-imaginary
/// position whose root is a twist-fixed SIMPLE generator of the datum, the
/// induced map must be injective, and it must cover every twist-fixed
/// generator. Returns the generator index per adjoint bit.
fn verified_generator_map(
    inner_class: &InnerClass,
    grading: &CartanGradingData,
) -> Result<Vec<usize>, StructureError> {
    let root_system = inner_class.root_system();
    let delta = inner_class.distinguished_involution();
    let datum = inner_class.datum();
    let semisimple_rank = datum.semisimple_rank();
    let mut fixed_generators = BTreeSet::new();
    for generator in 0..semisimple_rank {
        let id = root_system
            .id_of(&datum.simple_roots()[generator].clone())
            .ok_or(StructureError::RealFormOrderInvariantViolation {
                invariant: "simple-root membership",
            })?;
        if delta.image(id) == Some(id) {
            fixed_generators.insert(generator);
        }
    }
    let dimension = grading.adjoint_fiber().dimension();
    if dimension != fixed_generators.len() {
        return Err(StructureError::RealFormOrderInvariantViolation {
            invariant: "twist-fixed coordinate count",
        });
    }

    let mut generator_of_bit = try_capacity(dimension)?;
    let mut seen = BTreeSet::new();
    for bit in 0..dimension {
        let shift =
            grading
                .grading_shift(bit)
                .ok_or(StructureError::RealFormOrderInvariantViolation {
                    invariant: "grading shift",
                })?;
        let mut flipped = shift.noncompact_indices();
        let position = flipped
            .next()
            .ok_or(StructureError::RealFormOrderInvariantViolation {
                invariant: "single-bit grading shift",
            })?;
        if flipped.next().is_some() {
            return Err(StructureError::RealFormOrderInvariantViolation {
                invariant: "single-bit grading shift",
            });
        }
        let root = grading.imaginary_simple_root(position).ok_or(
            StructureError::RealFormOrderInvariantViolation {
                invariant: "grading shift",
            },
        )?;
        let generator = (0..semisimple_rank)
            .find(|&candidate| {
                root_system
                    .root(root)
                    .is_some_and(|weight| weight == &datum.simple_roots()[candidate])
            })
            .ok_or(StructureError::RealFormOrderInvariantViolation {
                invariant: "twist-fixed generator coordinate",
            })?;
        if !fixed_generators.contains(&generator) || !seen.insert(generator) {
            return Err(StructureError::RealFormOrderInvariantViolation {
                invariant: "twist-fixed generator coordinate",
            });
        }
        if generator >= MAX_KEY_GENERATORS {
            return Err(StructureError::RealFormOrderInvariantViolation {
                invariant: "tiebreak key width",
            });
        }
        generator_of_bit.push(generator);
    }
    Ok(generator_of_bit)
}

/// The `specialGrading` PARTITION-overload tiebreak: elect the HIGHEST
/// adjoint-fiber index attaining maximal popcount within the form's class
/// (ascending scan, `>=` replacement, seeded by the class representative),
/// complement it within the fiber rank, and unslice the complement onto the
/// twist-fixed simple generators as an unsigned key (generator 0 = LSB).
fn special_grading_key(
    grading: &CartanGradingData,
    weak: &WeakRealFormPartition,
    form: WeakRealFormId,
    generator_of_bit: &[usize],
) -> Result<u128, StructureError> {
    let adjoint = grading.adjoint_fiber();
    let dimension = adjoint.dimension();
    if dimension > MAX_MASK_BITS {
        return Err(StructureError::RealFormOrderInvariantViolation {
            invariant: "tiebreak fiber width",
        });
    }
    let representative =
        weak.class_representative(form)
            .ok_or(StructureError::RealFormOrderInvariantViolation {
                invariant: "form representative",
            })?;
    let mut best = mask_of(grading, representative)?;
    let mut best_popcount = best.count_ones();
    for index in 0..(1_u64 << dimension) {
        let element = element_of_mask(grading, index)?;
        if weak.class_of(&element)? != form {
            continue;
        }
        if index.count_ones() >= best_popcount {
            best = index;
            best_popcount = index.count_ones();
        }
    }
    let complement = !best & mask_width(dimension);
    let mut key = 0_u128;
    for (bit, &generator) in generator_of_bit.iter().enumerate() {
        if complement & (1_u64 << bit) != 0 {
            key |= 1_u128 << generator;
        }
    }
    Ok(key)
}

fn mask_width(dimension: usize) -> u64 {
    if dimension == 0 {
        0
    } else {
        u64::MAX >> (64 - dimension)
    }
}

fn mask_of(
    grading: &CartanGradingData,
    element: &AdjointFiberElement,
) -> Result<u64, StructureError> {
    let adjoint = grading.adjoint_fiber();
    let coordinates = adjoint.coordinates(element)?;
    let mut mask = 0_u64;
    for index in 0..adjoint.dimension() {
        if coordinates.bit(index) == Some(true) {
            mask |= 1_u64 << index;
        }
    }
    Ok(mask)
}

fn element_of_mask(
    grading: &CartanGradingData,
    mask: u64,
) -> Result<AdjointFiberElement, StructureError> {
    let adjoint = grading.adjoint_fiber();
    let mut representative = ModTwoVector::zero(adjoint.datum().rank())?;
    for (index, basis) in adjoint.basis_representatives().iter().enumerate() {
        if mask & (1_u64 << index) != 0 {
            representative.xor_assign(basis)?;
        }
    }
    adjoint.element_from_ambient(representative)
}

fn simple_coordinates(
    root_system: &crate::RootSystem,
    root: RootId,
) -> Result<&[i32], StructureError> {
    root_system
        .simple_coordinates(root)
        .ok_or(StructureError::RealFormOrderInvariantViolation {
            invariant: "root lookup",
        })
}

/// Mod-two pairing of ambient fiber coordinates with a root's datum-simple
/// coordinates (the linear extension of the crate's grading shifts).
fn parity_dot(ambient: &ModTwoVector, coordinates: &[i32]) -> Result<bool, StructureError> {
    if ambient.dimension() != coordinates.len() {
        return Err(StructureError::RankMismatch {
            expected: coordinates.len(),
            actual: ambient.dimension(),
        });
    }
    let mut parity = false;
    for (index, &value) in coordinates.iter().enumerate() {
        if value % 2 != 0 && ambient.bit(index) == Some(true) {
            parity = !parity;
        }
    }
    Ok(parity)
}

fn weight_sum(left: &Weight, right: &Weight) -> Result<Option<Weight>, StructureError> {
    if left.rank() != right.rank() {
        return Err(StructureError::RankMismatch {
            expected: left.rank(),
            actual: right.rank(),
        });
    }
    let mut coordinates = try_capacity(left.rank())?;
    for (&a, &b) in left.as_slice().iter().zip(right.as_slice()) {
        coordinates.push(a.checked_add(b).ok_or(StructureError::ArithmeticOverflow)?);
    }
    Ok(Some(Weight::new(coordinates)))
}

#[cfg(test)]
mod tests {
    use crate::adjoint_fiber::AdjointFiberBudget;
    use crate::integer_lattice::IntegerLatticeBudget;
    use crate::{
        BasedRootDatum, CartanClassificationBudget, Coweight, LatticeInvolution,
        StrongRealClassification, Weight,
    };

    use super::*;

    fn classification(
        datum: &BasedRootDatum,
        distinguished: Option<Vec<Vec<i32>>>,
        roots: usize,
        weyl: usize,
    ) -> (InnerClass, CartanClassification) {
        let distinguished = match distinguished {
            Some(matrix) => LatticeInvolution::new(datum, matrix.clone(), matrix).unwrap(),
            None => LatticeInvolution::identity(datum).unwrap(),
        };
        let inner_class = InnerClass::new(datum.clone(), distinguished, roots).unwrap();
        let budget = CartanClassificationBudget::new(
            IntegerLatticeBudget::new(64, 100_000, 100_000, 128),
            AdjointFiberBudget::new(
                IntegerLatticeBudget::new(64, 100_000, 100_000, 128),
                50_000,
                100_000,
            ),
            weyl,
            64,
            64,
        );
        let classification = CartanClassification::build(&inner_class, &budget).unwrap();
        (inner_class, classification)
    }

    #[test]
    fn sl2_orders_compact_zero_and_split_last() {
        let datum = BasedRootDatum::from_simple_data(
            1,
            vec![vec![2]],
            vec![Weight::new(vec![2])],
            vec![Coweight::new(vec![1])],
        )
        .unwrap();
        let (inner_class, classification) = classification(&datum, None, 2, 2);
        let order = ExternalFormOrder::build(&inner_class, &classification).unwrap();

        assert_eq!(order.form_count(), 2);
        assert_eq!(order.internal(0), Some(WeakRealFormId(1)));
        assert_eq!(order.internal(1), Some(WeakRealFormId(0)));
        assert_eq!(order.external(WeakRealFormId(0)), Some(1));
        assert_eq!(order.quasisplit_external(), 1);

        // The external KGB sizes read 1 then 3 — SU(2) then SL(2,R).
        let strong = StrongRealClassification::build(&classification, 4_096).unwrap();
        assert_eq!(strong.kgb_size(order.internal(0).unwrap()), Some(1));
        assert_eq!(strong.kgb_size(order.internal(1).unwrap()), Some(3));
    }

    #[test]
    fn spin5_orders_sizes_one_four_eleven() {
        // Simply connected B2 = Spin(5) = Sp(4): weight-lattice basis.
        let datum = BasedRootDatum::from_simple_data(
            2,
            vec![vec![2, -2], vec![-1, 2]],
            vec![Weight::new(vec![2, -2]), Weight::new(vec![-1, 2])],
            vec![Coweight::new(vec![1, 0]), Coweight::new(vec![0, 1])],
        )
        .unwrap();
        let (inner_class, classification) = classification(&datum, None, 8, 8);
        let order = ExternalFormOrder::build(&inner_class, &classification).unwrap();

        assert_eq!(order.form_count(), 3);
        assert_eq!(order.internal(0), Some(WeakRealFormId(2)));
        assert_eq!(order.internal(1), Some(WeakRealFormId(1)));
        assert_eq!(order.internal(2), Some(WeakRealFormId(0)));

        let strong = StrongRealClassification::build(&classification, 4_096).unwrap();
        let sizes: Vec<_> = (0..3)
            .map(|external| strong.kgb_size(order.internal(external).unwrap()).unwrap())
            .collect();
        assert_eq!(sizes, vec![1, 4, 11]);
    }

    #[test]
    fn equal_rank_a2_orders_su3_before_su21() {
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let (inner_class, classification) = classification(&datum, None, 6, 6);
        let order = ExternalFormOrder::build(&inner_class, &classification).unwrap();

        assert_eq!(order.form_count(), 2);
        assert_eq!(order.internal(0), Some(WeakRealFormId(1)));
        assert_eq!(order.internal(1), Some(WeakRealFormId(0)));
    }

    #[test]
    fn twisted_a2_has_the_single_quasisplit_form() {
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let (inner_class, classification) =
            classification(&datum, Some(vec![vec![0, 1], vec![1, 0]]), 6, 6);
        let order = ExternalFormOrder::build(&inner_class, &classification).unwrap();

        assert_eq!(order.form_count(), 1);
        assert_eq!(order.internal(0), Some(WeakRealFormId(0)));
        assert_eq!(order.quasisplit_external(), 0);
    }

    #[test]
    fn spin8_ties_break_strictly_with_compact_first_and_split_last() {
        // Simply connected D4 (Spin(8)) exercises the grading tiebreak: the
        // two so*(8) forms share a depth and must still order strictly.
        let cartan = vec![
            vec![2, -1, 0, 0],
            vec![-1, 2, -1, -1],
            vec![0, -1, 2, 0],
            vec![0, -1, 0, 2],
        ];
        let roots: Vec<Weight> = cartan.iter().cloned().map(Weight::new).collect();
        let coroots: Vec<Coweight> = (0..4)
            .map(|index| {
                let mut coordinates = vec![0; 4];
                coordinates[index] = 1;
                Coweight::new(coordinates)
            })
            .collect();
        let datum = BasedRootDatum::from_simple_data(4, cartan, roots, coroots).unwrap();
        let (inner_class, classification) = classification(&datum, None, 48, 192);
        let order = ExternalFormOrder::build(&inner_class, &classification).unwrap();

        let count = order.form_count();
        assert!(count >= 3);
        assert_eq!(order.internal(count - 1), Some(WeakRealFormId(0)));
        let strong = StrongRealClassification::build(&classification, 4_096).unwrap();
        assert_eq!(strong.kgb_size(order.internal(0).unwrap()), Some(1));
    }
}
