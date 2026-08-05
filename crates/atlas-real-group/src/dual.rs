//! The dual inner class of an inner class, and the number of dual real forms.
//!
//! Upstream the dual based involution is `rootdata::dualBasedInvolution`
//! (rootdata.cpp:1357-1365): with `w0 = rd.to_dominant(-rd.twoRho())` (the
//! longest Weyl group element, characterized by sending `2rho` to `-2rho`)
//! the result is `(q * rd.action_matrix(w0)).negative_transposed()`, where
//! `q` is the distinguished involution of the original inner class. The dual
//! based root datum transposes the Cartan matrix and swaps simple roots with
//! simple coroots (upstream `RootDatum(rd, tags::DualTag)`), and the number
//! of dual real forms is the size of the fundamental weak-real-form
//! partition of the dual inner class at its distinguished involution
//! (complexredgp.cpp `InnerClass::numDualRealForms`).

use crate::adjoint_fiber::{AdjointCartanFiber, AdjointFiberBudget};
use crate::cartan_fiber::CartanFiber;
use crate::grading::CartanGradingData;
use crate::integer_lattice::IntegerLatticeBudget;
use crate::twisted_involution::compose_matrices;
use crate::weak_real_form::WeakRealFormPartition;
use crate::{
    BasedRootDatum, CartanClassification, CartanId, Coweight, InnerClass, LatticeInvolution,
    StructureError, Weight, WeylAction, WeylGroup,
};

/// The based root datum dual to `datum`: transposed Cartan matrix, simple
/// roots and simple coroots interchanged.
fn dual_datum(datum: &BasedRootDatum) -> Result<BasedRootDatum, StructureError> {
    let semisimple_rank = datum.semisimple_rank();
    let cartan = datum.cartan_matrix();
    let transposed: Vec<Vec<i32>> = (0..semisimple_rank)
        .map(|row| cartan.iter().map(|cartan_row| cartan_row[row]).collect())
        .collect();
    let dual_roots: Vec<Weight> = datum
        .simple_coroots()
        .iter()
        .map(|coroot| Weight::new(coroot.as_slice().to_vec()))
        .collect();
    let dual_coroots: Vec<Coweight> = datum
        .simple_roots()
        .iter()
        .map(|root| Coweight::new(root.as_slice().to_vec()))
        .collect();
    BasedRootDatum::from_simple_data(datum.lattice_rank(), transposed, dual_roots, dual_coroots)
}

/// Twice the sum of the positive roots, in weight coordinates
/// (`RootDatum::twoRho`).
fn two_rho(inner_class: &InnerClass) -> Result<Weight, StructureError> {
    let root_system = inner_class.root_system();
    let mut sum = vec![0_i64; inner_class.datum().lattice_rank()];
    for (id, root, _) in root_system.entries() {
        if root_system.is_positive(id) != Some(true) {
            continue;
        }
        for (slot, &value) in sum.iter_mut().zip(root.as_slice()) {
            *slot = slot
                .checked_add(i64::from(value))
                .ok_or(StructureError::ArithmeticOverflow)?;
        }
    }
    let mut coordinates = Vec::with_capacity(sum.len());
    for value in sum {
        coordinates.push(i32::try_from(value).map_err(|_| StructureError::ArithmeticOverflow)?);
    }
    Ok(Weight::new(coordinates))
}

/// The longest Weyl group element, characterized by sending `2rho` to
/// `-2rho` (`rd.to_dominant(-rd.twoRho())`).
///
/// Unlike a full Weyl-group enumeration, the descent walk from `2rho` to
/// `-2rho` takes exactly the reduced length of the longest element
/// (upstream `WeylGroup::longest` is transducer O(rank); this is the
/// equivalent O(length) walk — each step reflects by a generator with
/// positive coroot pairing, which lowers the height of `2rho` by one).
pub fn longest_action(
    inner_class: &InnerClass,
    weyl_budget: usize,
) -> Result<WeylAction, StructureError> {
    let datum = inner_class.datum();
    let rank = datum.semisimple_rank();
    let two_rho_weight = two_rho(inner_class)?;
    let negative: Vec<i32> = two_rho_weight
        .as_slice()
        .iter()
        .map(|value| {
            value
                .checked_neg()
                .ok_or(StructureError::ArithmeticOverflow)
        })
        .collect::<Result<_, _>>()?;
    let negative_rho = Weight::new(negative);

    let group = WeylGroup::new(datum.clone());
    let mut action = WeylAction::identity(datum)?;
    let mut current = two_rho_weight;
    let mut steps = 0_usize;
    while current != negative_rho {
        let mut advanced = false;
        for s in 0..rank {
            let coroot = datum.simple_coroots()[s].as_slice();
            let mut pairing: i64 = 0;
            for (index, &coordinate) in coroot.iter().enumerate() {
                pairing += i64::from(coordinate) * i64::from(current.as_slice()[index]);
            }
            if pairing > 0 {
                let reflection = group.simple_reflection(s)?;
                action = reflection.compose(&action)?;
                current = reflection.act(&current)?;
                advanced = true;
                break;
            }
        }
        steps += 1;
        if steps > weyl_budget || !advanced {
            return Err(StructureError::LayoutInvariantViolation {
                invariant: "longest Weyl element",
            });
        }
    }
    Ok(action)
}

/// The distinguished involution of the dual inner class
/// (`dualBasedInvolution`): `-(q * W0)^t` on the dual weight lattice, with
/// `W0` the action matrix of the longest Weyl element.
fn dual_involution(
    inner_class: &InnerClass,
    weyl_budget: usize,
) -> Result<Vec<Vec<i32>>, StructureError> {
    let longest = longest_action(inner_class, weyl_budget)?;
    // M = q * W0; the dual involution is -M^t (the coweight action -M is
    // derived at construction, since M is an involution).
    let distinguished = inner_class.distinguished_involution().involution();
    compose_matrices(distinguished.weight_matrix(), longest.matrix())
}

/// Build the dual inner class of `inner_class`.
///
/// The Weyl budget bounds the enumeration locating the longest element;
/// `root_budget` bounds the dual root-system closure.
pub fn dual_inner_class(
    inner_class: &InnerClass,
    weyl_budget: usize,
    root_budget: usize,
) -> Result<InnerClass, StructureError> {
    let dual = dual_datum(inner_class.datum())?;
    let product = dual_involution(inner_class, weyl_budget)?;

    let rank = product.len();
    let mut dual_weight = vec![vec![0_i32; rank]; rank];
    let mut dual_coweight = vec![vec![0_i32; rank]; rank];
    for (row, product_row) in product.iter().enumerate() {
        for (column, &value) in product_row.iter().enumerate() {
            let negated = value
                .checked_neg()
                .ok_or(StructureError::ArithmeticOverflow)?;
            dual_weight[column][row] = negated;
            dual_coweight[row][column] = negated;
        }
    }
    let involution = LatticeInvolution::new(&dual, dual_weight, dual_coweight)?;
    InnerClass::new(dual, involution, root_budget)
}

/// The Cartan-class correspondence across duality, for every Cartan class of
/// `classification` in crate Cartan order.
///
/// Upstream builds the dual inner class's Cartan list from the original one
/// in reverse order, pairing the Cartan with representative twisted
/// involution `tw` against the dual Cartan of twisted involution `tw * w0`
/// (innerclass.cpp:435-441, the dual `InnerClass` constructor). The dual
/// distinguished involution is `-(delta * w0)^t` (`dualBasedInvolution`),
/// and transposition is contragredient, so the paired dual Cartan involution
/// on the dual weight lattice — the original coweight lattice — simplifies:
/// `(w * w0)|co ∘ (-(delta * w0)^t)` = `-(w * delta)|co`, the negative of
/// the original Cartan involution read on the coweight side. Upstream then
/// canonicalizes the dual twisted involution, so the dual class's stored
/// representative is generally a CONJUGATE of `tw * w0` and matrix
/// comparison is unsound; the class is instead located by the root-image
/// permutation that lattice map induces on the dual roots, exactly the key
/// of [`crate::TwistedConjugacyPartition::class_of`]. Each entry carries
/// the dual CartanId with the dual class's weak-real-form count (upstream
/// `CartanClass::numDualRealForms`, the dual fiber's weak-real partition
/// size). A permutation miss is an invariant violation, never a hole: every
/// twisted involution is in the full-W enumeration. `weyl_budget` bounds
/// that enumeration of the dual side.
pub fn dual_cartan_correspondence(
    inner_class: &InnerClass,
    classification: &CartanClassification,
    dual: &InnerClass,
    dual_classification: &CartanClassification,
    _weyl_budget: usize,
) -> Result<Vec<(CartanId, usize)>, StructureError> {
    let original_fundamental = classification.cartan_ids().next().ok_or(
        StructureError::CartanClassificationInvariantViolation {
            invariant: "dual Cartan correspondence",
        },
    )?;
    if classification
        .cartan_class(original_fundamental)
        .expect("cartan_ids yields in-range ids")
        .representative()
        .weyl_action()
        .datum()
        != inner_class.datum()
    {
        return Err(StructureError::DatumMismatch);
    }
    let dual_roots = dual.root_system();
    let dual_fundamental = dual_classification.cartan_ids().next().ok_or(
        StructureError::CartanClassificationInvariantViolation {
            invariant: "dual Cartan correspondence",
        },
    )?;
    if dual_classification
        .cartan_class(dual_fundamental)
        .expect("cartan_ids yields in-range ids")
        .representative()
        .weyl_action()
        .datum()
        != dual.datum()
    {
        return Err(StructureError::DatumMismatch);
    }

    // The dual partition's member-permutation map, and the raw class index of
    // each classification class (its representative's permutation is a member
    // key; the fundamental class's normalized identity is the identity
    // permutation, also a member key).
    let partition = std::sync::Arc::clone(dual_classification.twisted_partition());
    let permutation_of = |dual_class: &crate::CartanClass| {
        dual_class
            .representative()
            .root_involution()
            .image_permutation()
            .iter()
            .map(|id| id.0 as u8)
            .collect::<Vec<_>>()
    };
    let mut cartan_of_raw = vec![None; partition.classes().len()];
    for id in dual_classification.cartan_ids() {
        let dual_class = dual_classification
            .cartan_class(id)
            .expect("cartan_ids yields in-range ids");
        let raw = partition
            .class_index_of_permutation(&permutation_of(dual_class))
            .ok_or(StructureError::CartanClassificationInvariantViolation {
                invariant: "dual Cartan correspondence",
            })?;
        cartan_of_raw[raw] = Some(id);
    }

    let mut correspondence = Vec::with_capacity(classification.cartan_classes().len());
    for cartan_class in classification.cartan_classes() {
        // theta_dual = -(theta on the coweight lattice): the permutation that
        // map induces on the dual roots.
        let coweight = cartan_class
            .representative()
            .root_involution()
            .involution()
            .coweight_matrix();
        let mut permutation = Vec::with_capacity(dual_roots.roots().len());
        for root in dual_roots.roots() {
            let mut image = Vec::with_capacity(coweight.len());
            for entries in coweight.iter() {
                let mut value = 0_i64;
                for (column, &entry) in root.as_slice().iter().enumerate() {
                    let term = i64::from(entries[column])
                        .checked_mul(i64::from(entry))
                        .ok_or(StructureError::ArithmeticOverflow)?;
                    value = value
                        .checked_add(term)
                        .ok_or(StructureError::ArithmeticOverflow)?;
                }
                let negated = value
                    .checked_neg()
                    .ok_or(StructureError::ArithmeticOverflow)?;
                image.push(i32::try_from(negated).map_err(|_| StructureError::ArithmeticOverflow)?);
            }
            permutation.push(dual_roots.id_of(&Weight::new(image)).ok_or(
                StructureError::CartanClassificationInvariantViolation {
                    invariant: "dual Cartan correspondence",
                },
            )?);
        }
        let permutation: Vec<u8> = permutation.iter().map(|id| id.0 as u8).collect();
        let raw = partition.class_index_of_permutation(&permutation).ok_or(
            StructureError::CartanClassificationInvariantViolation {
                invariant: "dual Cartan correspondence",
            },
        )?;
        let dual_id =
            cartan_of_raw[raw].ok_or(StructureError::CartanClassificationInvariantViolation {
                invariant: "dual Cartan correspondence",
            })?;
        let form_count = dual_classification
            .cartan_class(dual_id)
            .expect("cartan_ids yields in-range ids")
            .partition()
            .class_count();
        correspondence.push((dual_id, form_count));
    }
    Ok(correspondence)
}

/// The number of dual real forms of `inner_class`: the fundamental
/// weak-real-form partition size of the dual inner class
/// (`InnerClass::numDualRealForms`).
pub fn dual_real_form_count(
    inner_class: &InnerClass,
    weyl_budget: usize,
    integer_budget: &IntegerLatticeBudget,
    adjoint_budget: &AdjointFiberBudget,
    fiber_budget: usize,
    root_budget: usize,
) -> Result<usize, StructureError> {
    let dual = dual_inner_class(inner_class, weyl_budget, root_budget)?;
    let fundamental =
        CartanFiber::build(dual.distinguished_involution().involution(), integer_budget)?;
    let adjoint = AdjointCartanFiber::build(
        dual.root_system(),
        dual.distinguished_involution(),
        &fundamental,
        adjoint_budget,
    )?;
    let grading = CartanGradingData::build(
        dual.root_system(),
        dual.distinguished_involution(),
        &adjoint,
    )?;
    let partition = WeakRealFormPartition::build(&grading, fiber_budget)?;
    Ok(partition.class_count())
}

#[cfg(test)]
mod tests {
    use crate::{BasedRootDatum, Coweight, InnerClass, LatticeInvolution, Weight};

    use super::*;

    fn integer_budget() -> IntegerLatticeBudget {
        IntegerLatticeBudget::new(64, 100_000, 100_000, 128)
    }

    fn adjoint_budget() -> AdjointFiberBudget {
        AdjointFiberBudget::new(integer_budget(), 50_000, 100_000)
    }

    fn dual_count(datum: &BasedRootDatum, involution: LatticeInvolution) -> usize {
        let inner_class = InnerClass::new(datum.clone(), involution, 256).unwrap();
        dual_real_form_count(
            &inner_class,
            256,
            &integer_budget(),
            &adjoint_budget(),
            256,
            256,
        )
        .unwrap()
    }

    #[test]
    fn simply_connected_a1_has_two_dual_real_forms() {
        // The dual datum of sc A1 is adjoint A1; its two dual real forms are
        // the compact and split forms of the adjoint inner class.
        let datum = BasedRootDatum::from_simple_data(
            1,
            vec![vec![2]],
            vec![Weight::new(vec![2])],
            vec![Coweight::new(vec![1])],
        )
        .unwrap();
        let involution = LatticeInvolution::identity(&datum).unwrap();
        assert_eq!(dual_count(&datum, involution), 2);
    }

    #[test]
    fn adjoint_a1_has_two_dual_real_forms() {
        // The dual datum of adjoint A1 is sc A1.
        let datum = BasedRootDatum::standard(vec![vec![2]]).unwrap();
        let involution = LatticeInvolution::identity(&datum).unwrap();
        assert_eq!(dual_count(&datum, involution), 2);
    }

    #[test]
    fn equal_rank_a2_has_a_single_dual_real_form() {
        // Oracle: sc A2 compact inner class prints "2 real forms and 1 dual
        // real form": the dual is the quasi-split adjoint A2, which has the
        // single form PSL(3,R).
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let involution = LatticeInvolution::identity(&datum).unwrap();
        assert_eq!(dual_count(&datum, involution), 1);
    }

    #[test]
    fn twisted_a2_dual_is_again_twisted_with_two_forms() {
        // The twist swaps the A2 simple roots; the dual inner class of the
        // quasi-split A2 is again quasi-split, with two dual real forms
        // (SU(3)/SU(2,1) of the adjoint dual datum... counted as classes).
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let twist = LatticeInvolution::new(
            &datum,
            vec![vec![0, 1], vec![1, 0]],
            vec![vec![0, 1], vec![1, 0]],
        )
        .unwrap();
        assert_eq!(dual_count(&datum, twist), 2);
    }

    #[test]
    fn compact_b2_has_three_dual_real_forms() {
        // Oracle: sc B2 compact inner class prints "3 real forms and 3 dual
        // real forms" (for B2, w0 = -1, so the compact and split inner
        // classes coincide on both sides).
        let datum = BasedRootDatum::from_simple_data(
            2,
            vec![vec![2, -2], vec![-1, 2]],
            vec![Weight::new(vec![2, -2]), Weight::new(vec![-1, 2])],
            vec![Coweight::new(vec![1, 0]), Coweight::new(vec![0, 1])],
        )
        .unwrap();
        let involution = LatticeInvolution::identity(&datum).unwrap();
        assert_eq!(dual_count(&datum, involution), 3);
    }

    #[test]
    fn dual_involution_of_identity_is_minus_w0_transposed() {
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let identity = LatticeInvolution::identity(&datum).unwrap();
        let inner_class = InnerClass::new(datum, identity, 6).unwrap();
        // W0 for A2 sends e1 -> -e2, e2 -> -e1, so -W0^t = [[0,1],[1,0]].
        let product = dual_involution(&inner_class, 6).unwrap();
        assert_eq!(product, vec![vec![0, -1], vec![-1, 0]]);
    }

    #[test]
    fn simply_connected_a1_cartans_pair_with_the_dual_cartans_in_reverse() {
        // Oracle (capture 3501500, A1 compact inner class): Cartan #0 occurs
        // for 1 dual real form, Cartan #1 for 2. The dual inner class is the
        // adjoint A1, whose split Cartan (#1) carries only the quasisplit
        // dual form while its fundamental Cartan (#0) carries both.
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
        let budget =
            crate::CartanClassificationBudget::new(integer_budget(), adjoint_budget(), 2, 64, 64);
        let classification = CartanClassification::build(&inner_class, &budget).unwrap();
        let dual = dual_inner_class(&inner_class, 2, 64).unwrap();
        let dual_classification = CartanClassification::build(&dual, &budget).unwrap();
        let correspondence = dual_cartan_correspondence(
            &inner_class,
            &classification,
            &dual,
            &dual_classification,
            2,
        )
        .unwrap();
        assert_eq!(correspondence, vec![(CartanId(1), 1), (CartanId(0), 2)],);
    }

    #[test]
    fn compact_b2_correspondence_is_a_bijection_onto_the_dual_cartans() {
        // B2 has w0 = -1 central, so the compact inner class is its own dual
        // type. The fundamental Cartan pairs with the dual `w0` class — only
        // the quasisplit dual form has a split Cartan — and the original
        // `w0` class (involution -1 on the whole lattice) pairs with the
        // dual fundamental, which every one of the three dual real forms
        // contains.
        let datum = BasedRootDatum::from_simple_data(
            2,
            vec![vec![2, -2], vec![-1, 2]],
            vec![Weight::new(vec![2, -2]), Weight::new(vec![-1, 2])],
            vec![Coweight::new(vec![1, 0]), Coweight::new(vec![0, 1])],
        )
        .unwrap();
        let inner_class = InnerClass::new(
            datum.clone(),
            LatticeInvolution::identity(&datum).unwrap(),
            8,
        )
        .unwrap();
        let budget =
            crate::CartanClassificationBudget::new(integer_budget(), adjoint_budget(), 8, 64, 64);
        let classification = CartanClassification::build(&inner_class, &budget).unwrap();
        assert_eq!(classification.cartan_classes().len(), 4);
        let dual = dual_inner_class(&inner_class, 8, 64).unwrap();
        let dual_classification = CartanClassification::build(&dual, &budget).unwrap();
        let correspondence = dual_cartan_correspondence(
            &inner_class,
            &classification,
            &dual,
            &dual_classification,
            8,
        )
        .unwrap();
        assert_eq!(correspondence.len(), 4);
        assert_eq!(correspondence[0].1, 1);
        let longest_index = classification
            .cartan_classes()
            .iter()
            .position(|class| {
                class
                    .representative()
                    .root_involution()
                    .involution()
                    .weight_matrix()
                    .iter()
                    .enumerate()
                    .all(|(row, entries)| {
                        entries
                            .iter()
                            .enumerate()
                            .all(|(column, &entry)| entry == -i32::from(row == column))
                    })
            })
            .expect("the w0 class is enumerated");
        assert_eq!(correspondence[longest_index].1, 3);
        assert!(correspondence.iter().all(|&(_, count)| count >= 1));
        let mut ids: Vec<usize> = correspondence
            .iter()
            .map(|&(id, _)| {
                dual_classification
                    .cartan_ids()
                    .position(|other| other == id)
                    .unwrap()
            })
            .collect();
        ids.sort_unstable();
        assert_eq!(ids, vec![0, 1, 2, 3]);
    }
}
