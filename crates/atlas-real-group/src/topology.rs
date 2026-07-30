//! Connectedness of a real form: the dual component group of its most split
//! Cartan (`topology::dual_component_group_basis`, topology.cpp:165-192).
//!
//! With `B` the lattice basis whose columns are the simple coroots followed
//! by a basis of the radical of the coweight lattice, `i_sw = B^t theta
//! B^{-t}` is the involution transported to the simply-connected cover's
//! weight lattice (upstream `theta.transposed().on_basis(basis).transposed()`).
//! The dual component group is the kernel of the restriction map
//! `dualPi0(theta) -> dualPi0(i_sw)` induced by `B_z^t mod 2`, where `B_z`
//! is `B` with its radical columns zeroed; the real form is connected
//! exactly when that kernel vanishes, i.e. when the induced map is injective.

use malachite::base::num::arithmetic::traits::Floor;
use malachite::base::num::basic::traits::Zero;
use malachite::Rational;

use crate::integer_lattice::{
    reduce_basis_mod_two, saturated_kernel, IntegerLatticeBudget, IntegerMatrix,
};
use crate::mod_two::{ModTwoAmbientMap, ModTwoSubquotient, ModTwoSubspace, ModTwoVector};
use crate::real_form_seed::invert_rational;
use crate::{BasedRootDatum, StructureError};

/// The `tori::dualPi0` subquotient of an involution: the mod-two kernel of
/// `theta + 1` modulo the mod-two image of the saturated `+1` eigenlattice
/// (tori.cpp:162-176, `eigen_lattice(theta, 1)`).
fn dual_pi0(
    theta: &[Vec<i32>],
    budget: &IntegerLatticeBudget,
) -> Result<ModTwoSubquotient, StructureError> {
    let rank = theta.len();
    let mut forms = ModTwoSubspace::new(rank)?;
    for (row_index, row) in theta.iter().enumerate() {
        if row.len() != rank {
            return Err(StructureError::InvalidIntegerMatrixShape);
        }
        let ones: Vec<usize> = row
            .iter()
            .enumerate()
            .filter(|(column, &value)| {
                let diagonal = i32::from(*column == row_index);
                (value + diagonal) % 2 != 0
            })
            .map(|(column, _)| column)
            .collect();
        forms.insert(ModTwoVector::from_ones(rank, ones)?)?;
    }
    let numerator = forms.right_kernel()?;

    let mut minus_identity = Vec::with_capacity(rank);
    for (row_index, row) in theta.iter().enumerate() {
        let mut converted = Vec::with_capacity(rank);
        for (column, &value) in row.iter().enumerate() {
            converted.push(value - i32::from(column == row_index));
        }
        minus_identity.push(converted);
    }
    let eigen = saturated_kernel(
        &IntegerMatrix::from_i32_rows(&minus_identity, budget)?,
        budget,
    )?;
    let denominator = reduce_basis_mod_two(&eigen)?;
    ModTwoSubquotient::new(numerator, denominator)
}

/// The restriction map `x -> B_z^t x mod 2` on ambient mod-two spaces.
struct CorootRestriction {
    /// `B` with the radical columns zeroed; columns index the map output.
    zeroed: Vec<Vec<i32>>,
}

impl ModTwoAmbientMap for CorootRestriction {
    fn source_dimension(&self) -> usize {
        self.zeroed.len()
    }

    fn target_dimension(&self) -> usize {
        self.zeroed.len()
    }

    fn apply(&self, source: &ModTwoVector) -> Result<ModTwoVector, StructureError> {
        if source.dimension() != self.zeroed.len() {
            return Err(StructureError::RankMismatch {
                expected: self.zeroed.len(),
                actual: source.dimension(),
            });
        }
        let rank = self.zeroed.len();
        let mut ones = Vec::new();
        for column in 0..rank {
            let mut parity = false;
            for (row, source_row) in self.zeroed.iter().enumerate() {
                if source_row[column] % 2 != 0 && source.bit(row) == Some(true) {
                    parity = !parity;
                }
            }
            if parity {
                ones.push(column);
            }
        }
        ModTwoVector::from_ones(rank, ones)
    }
}

fn rational_matrix(matrix: &[Vec<i32>]) -> Vec<Vec<Rational>> {
    matrix
        .iter()
        .map(|row| row.iter().map(|&value| Rational::from(value)).collect())
        .collect()
}

fn rational_product(
    left: &[Vec<Rational>],
    right: &[Vec<Rational>],
) -> Result<Vec<Vec<Rational>>, StructureError> {
    let rank = left.len();
    let mut product = vec![vec![Rational::ZERO; rank]; rank];
    for (row, target_row) in product.iter_mut().enumerate() {
        for (column, entry) in target_row.iter_mut().enumerate() {
            let mut sum = Rational::ZERO;
            for middle in 0..rank {
                sum += left[row][middle].clone() * right[middle][column].clone();
            }
            *entry = sum;
        }
    }
    Ok(product)
}

fn integral_entry(value: &Rational) -> Result<i32, StructureError> {
    let floored = value.clone().floor();
    if *value != floored {
        return Err(StructureError::LayoutInvariantViolation {
            invariant: "simply-connected involution integrality",
        });
    }
    i32::try_from(&floored).map_err(|_| StructureError::ArithmeticOverflow)
}

/// Whether the dual component group of a Cartan involution is trivial
/// (the `IsConnected` status bit of realredgp.cpp:73-75).
///
/// `theta` is the weight-lattice matrix of the Cartan involution of the
/// form's most split Cartan; `budget` bounds the integer-lattice reductions.
pub(crate) fn dual_component_group_trivial(
    theta: &[Vec<i32>],
    datum: &BasedRootDatum,
    budget: &IntegerLatticeBudget,
) -> Result<bool, StructureError> {
    let rank = datum.lattice_rank();
    let semisimple_rank = datum.semisimple_rank();
    if theta.len() != rank || theta.iter().any(|row| row.len() != rank) {
        return Err(StructureError::InvalidIntegerMatrixShape);
    }

    // The radical of the coweight lattice: kernel of the simple roots.
    let root_rows: Vec<Vec<i32>> = datum
        .simple_roots()
        .iter()
        .map(|root| root.as_slice().to_vec())
        .collect();
    let radical = saturated_kernel(&IntegerMatrix::from_i32_rows(&root_rows, budget)?, budget)?;
    if radical.rank() != rank - semisimple_rank {
        return Err(StructureError::LayoutInvariantViolation {
            invariant: "radical rank",
        });
    }

    // B: simple coroot columns, then radical columns.
    let mut basis = vec![vec![0_i32; rank]; rank];
    for (column, coroot) in datum.simple_coroots().iter().enumerate() {
        for (row, &value) in coroot.as_slice().iter().enumerate() {
            basis[row][column] = value;
        }
    }
    for (offset, column) in radical.columns().iter().enumerate() {
        for (row, value) in column.iter().enumerate() {
            basis[row][semisimple_rank + offset] =
                i32::try_from(value).map_err(|_| StructureError::ArithmeticOverflow)?;
        }
    }

    // i_sw = B^t theta B^{-t}, computed over the rationals with an
    // integrality check (upstream `on_basis` divides exactly).
    let transposed: Vec<Vec<i32>> = (0..rank)
        .map(|row| (0..rank).map(|column| basis[column][row]).collect())
        .collect();
    let transposed_rational = rational_matrix(&transposed);
    let inverse = invert_rational(&transposed_rational)?;
    let theta_rational = rational_matrix(theta);
    let product = rational_product(
        &rational_product(&transposed_rational, &theta_rational)?,
        &inverse,
    )?;
    let mut transported = Vec::with_capacity(rank);
    for row in &product {
        let mut converted = Vec::with_capacity(rank);
        for value in row {
            converted.push(integral_entry(value)?);
        }
        transported.push(converted);
    }

    let source = dual_pi0(theta, budget)?;
    let target = dual_pi0(&transported, budget)?;

    // The induced map between the subquotients must descend; its
    // injectivity is exactly the triviality of the dual component group.
    let mut zeroed = basis;
    for row in zeroed.iter_mut() {
        for entry in row.iter_mut().take(rank).skip(semisimple_rank) {
            *entry = 0;
        }
    }
    let map = CorootRestriction { zeroed };
    source.validate_induced_map_to(&target, &map)?;

    let mut image = ModTwoSubspace::new(target.dimension())?;
    for representative in source.basis_representatives() {
        let mapped = map.apply(representative)?;
        let coordinates = target.to_coordinates(mapped)?;
        image.insert(coordinates)?;
    }
    Ok(image.rank() == source.dimension())
}

#[cfg(test)]
mod tests {
    use crate::integer_lattice::IntegerLatticeBudget;
    use crate::{BasedRootDatum, Coweight, Weight};

    use super::*;

    fn budget() -> IntegerLatticeBudget {
        IntegerLatticeBudget::new(64, 100_000, 100_000, 128)
    }

    #[test]
    fn split_simply_connected_a1_is_connected() {
        // sc A1: coroot basis B = [[1]]; the restriction is the identity
        // mod 2, hence injective.
        let datum = BasedRootDatum::from_simple_data(
            1,
            vec![vec![2]],
            vec![Weight::new(vec![2])],
            vec![Coweight::new(vec![1])],
        )
        .unwrap();
        assert!(dual_component_group_trivial(&[vec![-1]], &datum, &budget()).unwrap());
    }

    #[test]
    fn split_adjoint_a1_is_disconnected() {
        // Adjoint A1 (PSL(2,R)): B = [[2]] vanishes mod 2, so the map is
        // zero on a rank-one subquotient: PGL(2,R) has two components.
        let datum = BasedRootDatum::standard(vec![vec![2]]).unwrap();
        assert!(!dual_component_group_trivial(&[vec![-1]], &datum, &budget()).unwrap());
    }

    #[test]
    fn compact_simply_connected_a1_is_connected() {
        // Compact SU(2): theta = +1. The numerator ker_F2(theta+1) is zero,
        // so the subquotient is trivial and the map injective.
        let datum = BasedRootDatum::from_simple_data(
            1,
            vec![vec![2]],
            vec![Weight::new(vec![2])],
            vec![Coweight::new(vec![1])],
        )
        .unwrap();
        assert!(dual_component_group_trivial(&[vec![1]], &datum, &budget()).unwrap());
    }

    #[test]
    fn split_gl2_is_disconnected() {
        // GL(2,R)-like datum: A1 plus a split central T1, theta = -1. The
        // coroot-basis restriction kills the torus coordinate, leaving a
        // rank-one dual component group: GL(2,R) has two components.
        let datum = BasedRootDatum::from_simple_data(
            2,
            vec![vec![2]],
            vec![Weight::new(vec![2, 0])],
            vec![Coweight::new(vec![1, 0])],
        )
        .unwrap();
        let theta = vec![vec![-1, 0], vec![0, -1]];
        assert!(!dual_component_group_trivial(&theta, &datum, &budget()).unwrap());
    }
}
