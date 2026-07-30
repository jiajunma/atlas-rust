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
    BasedRootDatum, Coweight, InnerClass, LatticeInvolution, StructureError, Weight, WeylGroup,
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

/// The distinguished involution of the dual inner class
/// (`dualBasedInvolution`): `-(q * W0)^t` on the dual weight lattice, with
/// `W0` the action matrix of the longest Weyl element.
fn dual_involution(
    inner_class: &InnerClass,
    weyl_budget: usize,
) -> Result<Vec<Vec<i32>>, StructureError> {
    let datum = inner_class.datum();
    let rho = two_rho(inner_class)?;
    let negative: Vec<i32> = rho
        .as_slice()
        .iter()
        .map(|value| {
            value
                .checked_neg()
                .ok_or(StructureError::ArithmeticOverflow)
        })
        .collect::<Result<_, _>>()?;
    let negative_rho = Weight::new(negative);

    let actions = WeylGroup::new(datum.clone()).enumerate_actions(weyl_budget)?;
    let longest = actions
        .iter()
        .find(|action| action.act(&rho).is_ok_and(|image| image == negative_rho))
        .ok_or(StructureError::LayoutInvariantViolation {
            invariant: "longest Weyl element",
        })?;

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
}
