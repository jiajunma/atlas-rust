use crate::grading::try_capacity;
use crate::integer_lattice::{saturated_kernel, IntegerMatrix};
use crate::{IntegerLatticeBudget, ModTwoSubspace, ModTwoVector, StructureError};

/// The compact, Complex, and split factor counts of an integral involution.
///
/// These are the uniquely determined ranks in the integral decomposition
/// into identity, exchanged-pair, and negated factors. The decomposition
/// itself is deliberately not chosen or stored.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvolutionClassification {
    compact: usize,
    complex: usize,
    split: usize,
}

impl InvolutionClassification {
    pub const fn compact(&self) -> usize {
        self.compact
    }

    pub const fn complex(&self) -> usize {
        self.complex
    }

    pub const fn split(&self) -> usize {
        self.split
    }

    pub const fn as_tuple(self) -> (usize, usize, usize) {
        (self.compact, self.complex, self.split)
    }
}

pub fn classify_involution(
    matrix: &[Vec<i32>],
    budget: &IntegerLatticeBudget,
) -> Result<InvolutionClassification, StructureError> {
    let rank = matrix.len();
    if matrix.iter().any(|row| row.len() != rank) {
        return Err(StructureError::InvalidIntegerMatrixShape);
    }
    // Enforce rank, storage, and coefficient budgets before the cubic square
    // check. The temporary exact matrix is dropped before `theta + I` is
    // allocated, so it does not inflate the live-entry accounting below.
    drop(IntegerMatrix::from_i32_rows(matrix, budget)?);
    if !is_involution(matrix)? {
        return Err(StructureError::InvalidInvolution);
    }

    let mut plus_identity = try_capacity(rank)?;
    for (row_index, row) in matrix.iter().enumerate() {
        let mut adjusted_row = try_capacity(rank)?;
        for (column_index, &entry) in row.iter().enumerate() {
            adjusted_row.push(if row_index == column_index {
                entry
                    .checked_add(1)
                    .ok_or(StructureError::ArithmeticOverflow)?
            } else {
                entry
            });
        }
        plus_identity.push(adjusted_row);
    }
    classify_plus_identity(&plus_identity, budget)
}

/// Classify a known involution from its `theta + I` matrix.
///
/// This is crate-visible so the central-torus quotient calculation can reuse
/// the exact same rank kernel after it has formed `theta + I` in Smith-basis
/// coordinates. Callers are responsible for the involution precondition.
pub(crate) fn classify_plus_identity(
    plus_identity: &[Vec<i32>],
    budget: &IntegerLatticeBudget,
) -> Result<InvolutionClassification, StructureError> {
    let rank = plus_identity.len();
    if plus_identity.iter().any(|row| row.len() != rank) {
        return Err(StructureError::InvalidIntegerMatrixShape);
    }

    let matrix = IntegerMatrix::from_i32_rows(plus_identity, budget)?;
    let kernel_rank = saturated_kernel(&matrix, budget)?.rank();
    let plus_rank = rank
        .checked_sub(kernel_rank)
        .ok_or(StructureError::IntegerLatticeInvariantViolation)?;

    let mut image = ModTwoSubspace::new(rank)?;
    for row in plus_identity {
        let odd_coordinates = row
            .iter()
            .enumerate()
            .filter_map(|(column, entry)| (entry % 2 != 0).then_some(column));
        image.insert(ModTwoVector::from_ones(rank, odd_coordinates)?)?;
    }
    let complex = image.rank();
    let compact = plus_rank
        .checked_sub(complex)
        .ok_or(StructureError::IntegerLatticeInvariantViolation)?;
    let split = rank
        .checked_sub(
            plus_rank
                .checked_add(complex)
                .ok_or(StructureError::ArithmeticOverflow)?,
        )
        .ok_or(StructureError::IntegerLatticeInvariantViolation)?;

    Ok(InvolutionClassification {
        compact,
        complex,
        split,
    })
}

fn is_involution(matrix: &[Vec<i32>]) -> Result<bool, StructureError> {
    let rank = matrix.len();
    for row in 0..rank {
        for column in 0..rank {
            let product = (0..rank).try_fold(0_i128, |sum, middle| {
                let term = i128::from(matrix[row][middle])
                    .checked_mul(i128::from(matrix[middle][column]))
                    .ok_or(StructureError::ArithmeticOverflow)?;
                sum.checked_add(term)
                    .ok_or(StructureError::ArithmeticOverflow)
            })?;
            if product != i128::from(row == column) {
                return Ok(false);
            }
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn budget() -> IntegerLatticeBudget {
        IntegerLatticeBudget::new(16, 1_024, 10_000, 128)
    }

    #[test]
    fn identity_is_fully_compact() {
        let classification = classify_involution(&[vec![1, 0], vec![0, 1]], &budget()).unwrap();

        assert_eq!(classification.as_tuple(), (2, 0, 0));
        assert_eq!(classification.compact(), 2);
        assert_eq!(classification.complex(), 0);
        assert_eq!(classification.split(), 0);
    }

    #[test]
    fn a2_opposition_is_one_complex_factor() {
        let classification = classify_involution(&[vec![0, -1], vec![-1, 0]], &budget()).unwrap();

        assert_eq!(classification.as_tuple(), (0, 1, 0));
    }

    #[test]
    fn parity_distinguishes_complex_from_compact_and_split_factors() {
        let complex = classify_involution(&[vec![1, 1], vec![0, -1]], &budget()).unwrap();
        let compact_split = classify_involution(&[vec![1, 2], vec![0, -1]], &budget()).unwrap();

        assert_eq!(complex.as_tuple(), (0, 1, 0));
        assert_eq!(compact_split.as_tuple(), (1, 0, 1));
    }

    #[test]
    fn rejects_a_square_matrix_that_is_not_an_involution() {
        assert_eq!(
            classify_involution(&[vec![2, 0], vec![0, 2]], &budget()),
            Err(StructureError::InvalidInvolution)
        );
    }

    #[test]
    fn rejects_a_ragged_matrix_before_reduction() {
        assert_eq!(
            classify_involution(&[vec![1, 0], vec![0]], &budget()),
            Err(StructureError::InvalidIntegerMatrixShape)
        );
    }

    #[test]
    fn enforces_the_exact_lattice_budget_before_classification() {
        let rank_one_budget = IntegerLatticeBudget::new(1, 16, 100, 128);

        assert_eq!(
            classify_involution(&[vec![1, 0], vec![0, 1]], &rank_one_budget),
            Err(StructureError::IntegerLatticeResourceLimit {
                resource: "rank",
                limit: 1,
            })
        );
    }
}
