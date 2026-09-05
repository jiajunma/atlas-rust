use std::num::{NonZeroI32, NonZeroU64};

use malachite::{
    base::num::{
        arithmetic::traits::{Abs, DivExact, DivMod, DivisibleBy, ExtendedGcd},
        basic::traits::{One, Zero},
        logic::traits::SignificantBits,
    },
    Integer,
};

use crate::{ModTwoSubspace, ModTwoVector, StructureError};

/// Caller-owned resource bounds for exact integral linear algebra.
///
/// These are computational budgets, not mathematical rank restrictions. They
/// bound each matrix dimension, the aggregate live working-entry count,
/// elementary operation count, and intermediate coefficient size for a
/// particular computation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegerLatticeBudget {
    max_rank: usize,
    max_entries: usize,
    max_steps: usize,
    max_coefficient_bits: u64,
}

impl IntegerLatticeBudget {
    pub const fn new(
        max_rank: usize,
        max_entries: usize,
        max_steps: usize,
        max_coefficient_bits: u64,
    ) -> Self {
        Self {
            max_rank,
            max_entries,
            max_steps,
            max_coefficient_bits,
        }
    }

    pub(crate) const fn max_rank(&self) -> usize {
        self.max_rank
    }
}

/// Coefficients for the unimodular transformation associated with two entries.
struct BezoutTransform {
    s: Integer,
    t: Integer,
    u: Integer,
    v: Integer,
}

impl BezoutTransform {
    fn for_entries(a: &Integer, b: &Integer) -> Self {
        let (gcd, s, t) = a.extended_gcd(b);
        let gcd = Integer::from(gcd);
        Self {
            s,
            t,
            u: a.div_exact(&gcd),
            v: b.div_exact(&gcd),
        }
    }
}

/// A rectangular matrix over Malachite integers in row-major storage.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegerMatrix {
    pub rows: usize,
    pub columns: usize,
    pub entries: Vec<Integer>,
}

impl IntegerMatrix {
    fn from_i32_entries(
        rows: usize,
        columns: usize,
        source: &[i32],
        budget: &IntegerLatticeBudget,
    ) -> Result<Self, StructureError> {
        let entry_count = checked_shape(rows, columns, budget)?;
        if source.len() != entry_count {
            return Err(StructureError::InvalidIntegerMatrixShape);
        }
        let mut entries = try_integer_vec(entry_count)?;
        entries.extend(source.iter().copied().map(Integer::from));
        let matrix = Self {
            rows,
            columns,
            entries,
        };
        matrix.check_coefficient_bits(budget)?;
        Ok(matrix)
    }

    pub(crate) fn from_i32_rows(
        rows: &[Vec<i32>],
        budget: &IntegerLatticeBudget,
    ) -> Result<Self, StructureError> {
        let row_count = rows.len();
        let column_count = rows.first().map_or(0, Vec::len);
        if rows.iter().any(|row| row.len() != column_count) {
            return Err(StructureError::InvalidIntegerMatrixShape);
        }
        let entry_count = checked_shape(row_count, column_count, budget)?;
        let mut entries = try_integer_vec(entry_count)?;
        for row in rows {
            entries.extend(row.iter().copied().map(Integer::from));
        }
        let matrix = Self {
            rows: row_count,
            columns: column_count,
            entries,
        };
        matrix.check_coefficient_bits(budget)?;
        Ok(matrix)
    }

    pub(crate) fn zero(
        rows: usize,
        columns: usize,
        budget: &IntegerLatticeBudget,
    ) -> Result<Self, StructureError> {
        let entry_count = checked_shape(rows, columns, budget)?;
        let mut entries = try_integer_vec(entry_count)?;
        entries.resize(entry_count, Integer::ZERO);
        Ok(Self {
            rows,
            columns,
            entries,
        })
    }

    fn identity(rank: usize, budget: &IntegerLatticeBudget) -> Result<Self, StructureError> {
        let mut matrix = Self::zero(rank, rank, budget)?;
        for index in 0..rank {
            matrix.set(index, index, Integer::ONE);
        }
        Ok(matrix)
    }

    fn try_clone(&self) -> Result<Self, StructureError> {
        let mut entries = try_integer_vec(self.entries.len())?;
        entries.extend(self.entries.iter().cloned());
        Ok(Self {
            rows: self.rows,
            columns: self.columns,
            entries,
        })
    }

    pub(crate) fn entry(&self, row: usize, column: usize) -> &Integer {
        &self.entries[self.index(row, column)]
    }

    fn set(&mut self, row: usize, column: usize, value: Integer) {
        let index = self.index(row, column);
        self.entries[index] = value;
    }

    fn index(&self, row: usize, column: usize) -> usize {
        row * self.columns + column
    }

    fn check_against(&self, budget: &IntegerLatticeBudget) -> Result<(), StructureError> {
        checked_shape(self.rows, self.columns, budget)?;
        self.check_coefficient_bits(budget)
    }

    fn check_coefficient_bits(&self, budget: &IntegerLatticeBudget) -> Result<(), StructureError> {
        if self
            .entries
            .iter()
            .any(|entry| entry.significant_bits() > budget.max_coefficient_bits)
        {
            return Err(resource_limit(
                "coefficient bits",
                budget.max_coefficient_bits,
            ));
        }
        Ok(())
    }

    fn swap_rows(&mut self, left: usize, right: usize) {
        if left == right {
            return;
        }
        for column in 0..self.columns {
            let left_index = self.index(left, column);
            let right_index = self.index(right, column);
            self.entries.swap(left_index, right_index);
        }
    }

    fn swap_columns(&mut self, left: usize, right: usize) {
        if left == right {
            return;
        }
        for row in 0..self.rows {
            let left_index = self.index(row, left);
            let right_index = self.index(row, right);
            self.entries.swap(left_index, right_index);
        }
    }

    fn negate_row(&mut self, row: usize) {
        for column in 0..self.columns {
            let index = self.index(row, column);
            self.entries[index] = -&self.entries[index];
        }
    }

    // The row/column operations below compute each combination and write it
    // in place in a single pass: every element is read before it is
    // overwritten and the source line is never a write target, so element
    // order cannot affect results. A budget failure can leave the target
    // line partially rewritten; that is unobservable because the error is a
    // fatal resource limit — every caller propagates it and drops the
    // matrix. (An intermediate two-pass version prevalidated all bounds
    // first to keep the matrix pristine on error, but doubling the per-entry
    // i64 conversions cost more than the saved Vec allocation.)

    fn add_row_multiple(
        &mut self,
        target: usize,
        source: usize,
        factor: &Integer,
        max_coefficient_bits: u64,
    ) -> Result<(), StructureError> {
        for column in 0..self.columns {
            let target_index = self.index(target, column);
            let source_index = self.index(source, column);
            let value = bounded_linear_combination(
                &Integer::ONE,
                &self.entries[target_index],
                factor,
                &self.entries[source_index],
                max_coefficient_bits,
            )?;
            self.entries[target_index] = value;
        }
        Ok(())
    }

    fn add_column_multiple(
        &mut self,
        target: usize,
        source: usize,
        factor: &Integer,
        max_coefficient_bits: u64,
    ) -> Result<(), StructureError> {
        for row in 0..self.rows {
            let target_index = self.index(row, target);
            let source_index = self.index(row, source);
            let value = bounded_linear_combination(
                &Integer::ONE,
                &self.entries[target_index],
                factor,
                &self.entries[source_index],
                max_coefficient_bits,
            )?;
            self.entries[target_index] = value;
        }
        Ok(())
    }

    /// Apply the unimodular two-row transformation
    /// `[[s, t], [-v, u]]`, where `a*u = b*v` and `s*a + t*b = gcd(a,b)`.
    fn bezout_rows(
        &mut self,
        top: usize,
        bottom: usize,
        transform: &BezoutTransform,
        max_coefficient_bits: u64,
    ) -> Result<(), StructureError> {
        let negative_v = -&transform.v;
        for column in 0..self.columns {
            let top_index = self.index(top, column);
            let bottom_index = self.index(bottom, column);
            let top_value = bounded_linear_combination(
                &transform.s,
                &self.entries[top_index],
                &transform.t,
                &self.entries[bottom_index],
                max_coefficient_bits,
            )?;
            let bottom_value = bounded_linear_combination(
                &negative_v,
                &self.entries[top_index],
                &transform.u,
                &self.entries[bottom_index],
                max_coefficient_bits,
            )?;
            self.entries[top_index] = top_value;
            self.entries[bottom_index] = bottom_value;
        }
        Ok(())
    }

    /// Apply the same unimodular transformation to two columns.
    fn bezout_columns(
        &mut self,
        left: usize,
        right: usize,
        transform: &BezoutTransform,
        max_coefficient_bits: u64,
    ) -> Result<(), StructureError> {
        let negative_v = -&transform.v;
        for row in 0..self.rows {
            let left_index = self.index(row, left);
            let right_index = self.index(row, right);
            let left_value = bounded_linear_combination(
                &transform.s,
                &self.entries[left_index],
                &transform.t,
                &self.entries[right_index],
                max_coefficient_bits,
            )?;
            let right_value = bounded_linear_combination(
                &negative_v,
                &self.entries[left_index],
                &transform.u,
                &self.entries[right_index],
                max_coefficient_bits,
            )?;
            self.entries[left_index] = left_value;
            self.entries[right_index] = right_value;
        }
        Ok(())
    }
}

/// A column basis in a free abelian group.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct IntegralBasis {
    ambient_rank: usize,
    columns: Vec<Vec<Integer>>,
}

impl IntegralBasis {
    /// The standard basis of the whole lattice: the radical of a
    /// rootless datum (Lie type T_k), whose kernel computation has no
    /// rows to reduce.
    pub(crate) fn full_space(ambient_rank: usize) -> Self {
        let mut columns = Vec::with_capacity(ambient_rank);
        for index in 0..ambient_rank {
            let mut column = vec![Integer::from(0u8); ambient_rank];
            column[index] = Integer::from(1u8);
            columns.push(column);
        }
        Self {
            ambient_rank,
            columns,
        }
    }

    fn kernel_columns(
        transformed_columns: &IntegerMatrix,
        start: usize,
        budget: &IntegerLatticeBudget,
    ) -> Result<Self, StructureError> {
        let ambient_rank = transformed_columns.rows;
        let kernel_rank = transformed_columns
            .columns
            .checked_sub(start)
            .ok_or(StructureError::IntegerLatticeInvariantViolation)?;
        let output_entries = ambient_rank
            .checked_mul(kernel_rank)
            .ok_or(StructureError::ArithmeticOverflow)?;
        check_entry_total(&[transformed_columns.entries.len(), output_entries], budget)?;
        let mut columns = Vec::new();
        columns
            .try_reserve_exact(kernel_rank)
            .map_err(|_| StructureError::AllocationFailed {
                requested: kernel_rank,
            })?;
        for column in start..transformed_columns.columns {
            let mut coordinates = try_integer_vec(ambient_rank)?;
            for row in 0..ambient_rank {
                coordinates.push(transformed_columns.entry(row, column).clone());
            }
            columns.push(coordinates);
        }
        Ok(Self {
            ambient_rank,
            columns,
        })
    }

    #[cfg(test)]
    pub(crate) fn ambient_rank(&self) -> usize {
        self.ambient_rank
    }

    pub(crate) fn rank(&self) -> usize {
        self.columns.len()
    }

    pub(crate) fn columns(&self) -> &[Vec<Integer>] {
        &self.columns
    }
}

/// Compute a basis of the saturated integer kernel of `matrix`.
///
/// The reduction keeps a unimodular right factor `V` while applying elementary
/// row and column transformations. Once the matrix is diagonal, the columns of
/// `V` belonging to zero diagonal entries form the full integral kernel. This
/// is deliberately not rational row reduction followed by clearing denominators.
pub(crate) fn saturated_kernel(
    matrix: &IntegerMatrix,
    budget: &IntegerLatticeBudget,
) -> Result<IntegralBasis, StructureError> {
    matrix.check_against(budget)?;
    // Machine-word twin first: identical reduction on i64 entries
    // (perf-unitary-3683465: the integer_lattice cluster is ~4.4% of the
    // heavy-unitary profile, all per-entry malachite traffic). `None`
    // means an intermediate left the i64 window — the generic path then
    // recomputes the identical result on bignums.
    if let Some(basis) = try_saturated_kernel_small(matrix, budget)? {
        return Ok(basis);
    }
    saturated_kernel_generic(matrix, budget)
}

fn saturated_kernel_generic(
    matrix: &IntegerMatrix,
    budget: &IntegerLatticeBudget,
) -> Result<IntegralBasis, StructureError> {
    let mut state = ReductionState::new(matrix, budget)?;
    let diagonal_limit = state.matrix.rows.min(state.matrix.columns);
    let mut diagonal_rank = 0;

    while diagonal_rank < diagonal_limit {
        let Some((pivot_row, pivot_column)) = state.first_nonzero(diagonal_rank) else {
            break;
        };
        state.swap_rows(diagonal_rank, pivot_row)?;
        state.swap_columns(diagonal_rank, pivot_column)?;

        loop {
            while let Some(row) = state.nonzero_below(diagonal_rank) {
                state.eliminate_below(diagonal_rank, row)?;
            }
            while let Some(column) = state.nonzero_right(diagonal_rank) {
                state.eliminate_right(diagonal_rank, column)?;
            }
            if state.nonzero_below(diagonal_rank).is_none()
                && state.nonzero_right(diagonal_rank).is_none()
            {
                break;
            }
        }

        if state.matrix.entry(diagonal_rank, diagonal_rank) < &Integer::ZERO {
            state.negate_row(diagonal_rank)?;
        }
        diagonal_rank += 1;
    }

    let ReductionState {
        matrix: working_matrix,
        right,
        ..
    } = state;
    drop(working_matrix);
    let kernel_rank = right
        .columns
        .checked_sub(diagonal_rank)
        .ok_or(StructureError::IntegerLatticeInvariantViolation)?;
    let output_entries = right
        .rows
        .checked_mul(kernel_rank)
        .ok_or(StructureError::ArithmeticOverflow)?;
    check_entry_total(
        &[matrix.entries.len(), right.entries.len(), output_entries],
        budget,
    )?;
    let basis = IntegralBasis::kernel_columns(&right, diagonal_rank, budget)?;
    drop(right);
    verify_annihilation(matrix, &basis, budget)?;
    Ok(basis)
}

/// Reduce an integral basis modulo two, preserving only its span in `Y / 2Y`.
pub(crate) fn reduce_basis_mod_two(
    basis: &IntegralBasis,
) -> Result<ModTwoSubspace, StructureError> {
    let mut reduced = ModTwoSubspace::new(basis.ambient_rank)?;
    let two = Integer::from(2);
    for column in &basis.columns {
        if column.len() != basis.ambient_rank {
            return Err(StructureError::IntegerLatticeInvariantViolation);
        }
        let mut ones = Vec::new();
        ones.try_reserve_exact(basis.ambient_rank).map_err(|_| {
            StructureError::AllocationFailed {
                requested: basis.ambient_rank,
            }
        })?;
        for (index, coordinate) in column.iter().enumerate() {
            if !coordinate.divisible_by(&two) {
                ones.push(index);
            }
        }
        reduced.insert(ModTwoVector::from_ones(basis.ambient_rank, ones)?)?;
    }
    Ok(reduced)
}

/// Compute `ker_Z(I + theta_Y)` from the cocharacter action itself.
///
/// `LatticeInvolution::coweight_matrix()` already serves the dual action on
/// cocharacters (as the transposed view of the retained character action),
/// so this function intentionally does not transpose it again. Callers on
/// the cold per-class/per-orbit-seed paths materialize the view once.
pub(crate) fn negative_coweight_eigenspace(
    coweight_action: &[Vec<i32>],
    budget: &IntegerLatticeBudget,
) -> Result<IntegralBasis, StructureError> {
    let mut matrix = IntegerMatrix::from_i32_rows(coweight_action, budget)?;
    if matrix.rows != matrix.columns {
        return Err(StructureError::InvalidIntegerMatrixShape);
    }
    for index in 0..matrix.rows {
        let value = matrix.entry(index, index) + &Integer::ONE;
        matrix.set(index, index, value);
    }
    matrix.check_against(budget)?;
    saturated_kernel(&matrix, budget)
}

/// An exact integer matrix used by the public relation-lattice operations.
///
/// The representation stays opaque so language adapters cannot bypass the
/// resource checks or depend on the reduction engine's storage layout.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationMatrix(IntegerMatrix);

impl RelationMatrix {
    /// Validate a relation-matrix shape before an adapter computes its entry
    /// count or allocates/copies any entries.
    pub fn preflight_shape(
        rows: usize,
        columns: usize,
        budget: &IntegerLatticeBudget,
    ) -> Result<usize, RelationError> {
        checked_shape(rows, columns, budget).map_err(RelationError::Structure)
    }

    pub fn from_i32_entries(
        rows: usize,
        columns: usize,
        row_major_entries: &[i32],
        budget: &IntegerLatticeBudget,
    ) -> Result<Self, RelationError> {
        IntegerMatrix::from_i32_entries(rows, columns, row_major_entries, budget)
            .map(Self)
            .map_err(RelationError::Structure)
    }

    /// Construct from row-major entries, rejecting an oversized shape before
    /// the iterator is advanced or storage is reserved.
    pub fn from_i32_iter<I>(
        rows: usize,
        columns: usize,
        row_major_entries: I,
        budget: &IntegerLatticeBudget,
    ) -> Result<Self, RelationError>
    where
        I: IntoIterator<Item = i32>,
    {
        let entry_count = Self::preflight_shape(rows, columns, budget)?;
        let mut source = row_major_entries.into_iter();
        let mut entries = try_integer_vec(entry_count)?;
        for _ in 0..entry_count {
            let entry = source
                .next()
                .ok_or(StructureError::InvalidIntegerMatrixShape)?;
            entries.push(Integer::from(entry));
        }
        if source.next().is_some() {
            return Err(RelationError::Structure(
                StructureError::InvalidIntegerMatrixShape,
            ));
        }
        let matrix = IntegerMatrix {
            rows,
            columns,
            entries,
        };
        matrix.check_coefficient_bits(budget)?;
        Ok(Self(matrix))
    }

    pub fn from_i32_rows(
        rows: &[Vec<i32>],
        budget: &IntegerLatticeBudget,
    ) -> Result<Self, RelationError> {
        IntegerMatrix::from_i32_rows(rows, budget)
            .map(Self)
            .map_err(RelationError::Structure)
    }

    pub const fn rows(&self) -> usize {
        self.0.rows
    }

    pub const fn columns(&self) -> usize {
        self.0.columns
    }

    pub fn try_i32_rows(&self) -> Result<Vec<Vec<i32>>, RelationError> {
        let mut rows = Vec::new();
        rows.try_reserve_exact(self.0.rows).map_err(|_| {
            RelationError::Structure(StructureError::AllocationFailed {
                requested: self.0.rows,
            })
        })?;
        for row in 0..self.0.rows {
            let mut entries = Vec::new();
            entries.try_reserve_exact(self.0.columns).map_err(|_| {
                RelationError::Structure(StructureError::AllocationFailed {
                    requested: self.0.columns,
                })
            })?;
            for column in 0..self.0.columns {
                entries.push(
                    i32::try_from(self.0.entry(row, column))
                        .map_err(|_| RelationError::IntegerOutOfRange)?,
                );
            }
            rows.push(entries);
        }
        Ok(rows)
    }
}

/// An elected ambient basis and the corresponding positive invariant factors.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationBasis {
    basis: RelationMatrix,
    factors: Vec<i32>,
}

impl RelationBasis {
    pub const fn basis(&self) -> &RelationMatrix {
        &self.basis
    }

    pub fn factors(&self) -> &[i32] {
        &self.factors
    }

    pub fn into_parts(self) -> (RelationMatrix, Vec<i32>) {
        (self.basis, self.factors)
    }
}

/// A rational generator as stored by an Atlas `ratvec`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RelationGenerator<'a> {
    numerators: &'a [i64],
    denominator: NonZeroU64,
}

impl<'a> RelationGenerator<'a> {
    pub const fn new(numerators: &'a [i64], denominator: NonZeroU64) -> Self {
        Self {
            numerators,
            denominator,
        }
    }

    /// Collect borrowed generator descriptors only after their count and
    /// implied numerator matrix have passed the relation-lattice budget.
    pub fn try_collect<I>(
        ambient_rank: usize,
        generators: I,
        budget: &IntegerLatticeBudget,
    ) -> Result<Vec<Self>, RelationError>
    where
        I: ExactSizeIterator<Item = Self>,
    {
        let generator_count = generators.len();
        let numerator_entries = checked_shape(ambient_rank, generator_count, budget)?;
        check_entry_total(&[numerator_entries, generator_count], budget)?;
        let mut collected = Vec::new();
        collected.try_reserve_exact(generator_count).map_err(|_| {
            StructureError::AllocationFailed {
                requested: generator_count,
            }
        })?;
        collected.extend(generators);
        Ok(collected)
    }
}

/// Semantic or resource failure from a relation-lattice operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RelationError {
    Structure(StructureError),
    TooManyFactors {
        factors: usize,
        columns: usize,
    },
    ColumnLengthsDoNotMatch,
    NotEnoughReplacementColumns,
    TooManyReplacementColumns,
    GeneratorLengthMismatch {
        generator: usize,
        actual: usize,
        expected: usize,
    },
    ImproperGeneratorEntry {
        numerator: i64,
        denominator: u64,
        factor: i32,
    },
    IntegerOutOfRange,
}

impl From<StructureError> for RelationError {
    fn from(error: StructureError) -> Self {
        Self::Structure(error)
    }
}

/// Run the existing upstream-order adapted-basis reducer behind an opaque,
/// checked result boundary.
pub fn adapted_relation_basis(
    rows: &[Vec<i32>],
    budget: &IntegerLatticeBudget,
) -> Result<RelationBasis, RelationError> {
    let adapted = adapted_basis(rows, budget)?;
    let mut factors = Vec::new();
    factors
        .try_reserve_exact(adapted.diagonal.len())
        .map_err(|_| StructureError::AllocationFailed {
            requested: adapted.diagonal.len(),
        })?;
    for factor in adapted.diagonal {
        factors.push(i32::try_from(&factor).map_err(|_| RelationError::IntegerOutOfRange)?);
    }
    Ok(RelationBasis {
        basis: RelationMatrix(adapted.basis),
        factors,
    })
}

pub fn filter_relation_units(
    basis: &RelationMatrix,
    factors: &[i32],
    budget: &IntegerLatticeBudget,
) -> Result<RelationBasis, RelationError> {
    if factors.len() > basis.columns() {
        return Err(RelationError::TooManyFactors {
            factors: factors.len(),
            columns: basis.columns(),
        });
    }
    let kept_count = (0..basis.columns())
        .filter(|&column| factors.get(column).copied() != Some(1))
        .count();
    let mut kept = Vec::new();
    kept.try_reserve_exact(kept_count)
        .map_err(|_| StructureError::AllocationFailed {
            requested: kept_count,
        })?;
    kept.extend((0..basis.columns()).filter(|&column| factors.get(column).copied() != Some(1)));
    let selected = select_columns(&basis.0, &kept, budget)?;
    let mut selected_factors = Vec::new();
    selected_factors
        .try_reserve_exact(kept_count)
        .map_err(|_| StructureError::AllocationFailed {
            requested: kept_count,
        })?;
    selected_factors.extend(factors.iter().copied().filter(|&factor| factor != 1));
    Ok(RelationBasis {
        basis: RelationMatrix(selected),
        factors: selected_factors,
    })
}

pub fn replace_relation_generators(
    basis: &RelationMatrix,
    factors: &[i32],
    replacements: &RelationMatrix,
    budget: &IntegerLatticeBudget,
) -> Result<RelationMatrix, RelationError> {
    if factors.len() > basis.columns() {
        return Err(RelationError::TooManyFactors {
            factors: factors.len(),
            columns: basis.columns(),
        });
    }
    if replacements.rows() != basis.rows() {
        return Err(RelationError::ColumnLengthsDoNotMatch);
    }
    check_entry_total(
        &[
            basis.0.entries.len(),
            replacements.0.entries.len(),
            basis.0.entries.len(),
        ],
        budget,
    )?;
    let mut result = basis.0.try_clone()?;
    let mut replacement = 0;
    for column in 0..basis.columns() {
        if factors.get(column).copied() == Some(1) {
            continue;
        }
        if replacement >= replacements.columns() {
            return Err(RelationError::NotEnoughReplacementColumns);
        }
        for row in 0..basis.rows() {
            result.set(row, column, replacements.0.entry(row, replacement).clone());
        }
        replacement += 1;
    }
    if replacement < replacements.columns() {
        return Err(RelationError::TooManyReplacementColumns);
    }
    result.check_against(budget)?;
    Ok(RelationMatrix(result))
}

fn select_columns(
    matrix: &IntegerMatrix,
    columns: &[usize],
    budget: &IntegerLatticeBudget,
) -> Result<IntegerMatrix, StructureError> {
    if columns.iter().any(|&column| column >= matrix.columns) {
        return Err(StructureError::IntegerLatticeInvariantViolation);
    }
    let output_entries = matrix
        .rows
        .checked_mul(columns.len())
        .ok_or(StructureError::ArithmeticOverflow)?;
    check_entry_total(&[matrix.entries.len(), output_entries], budget)?;
    let mut result = IntegerMatrix::zero(matrix.rows, columns.len(), budget)?;
    for (target, &source) in columns.iter().enumerate() {
        for row in 0..matrix.rows {
            result.set(row, target, matrix.entry(row, source).clone());
        }
    }
    Ok(result)
}

/// Return the full lattice of vectors whose transpose sends `matrix` into the
/// nonzero signed modulus. The elected basis follows upstream `diagonalise`.
pub fn annihilator_modulo(
    matrix: &RelationMatrix,
    modulus: NonZeroI32,
    budget: &IntegerLatticeBudget,
) -> Result<RelationMatrix, RelationError> {
    let mut steps = 0;
    // atlas-types.w reads an `int`, then passes it to a function taking
    // `Denom_t` (`unsigned long long`). C++ therefore converts a negative
    // value modulo 2^64 before the unsigned gcd.
    let denominator = Integer::from(atlas_denom_from_i32(modulus.get()));
    annihilator_modulo_integer(
        &matrix.0,
        &denominator,
        budget,
        &mut steps,
        0,
        RelationMultiplierWidth::AtlasLatticeCoefficient,
    )
    .map(RelationMatrix)
    .map_err(RelationError::Structure)
}

#[derive(Clone, Copy)]
enum RelationMultiplierWidth {
    Exact,
    AtlasLatticeCoefficient,
}

fn atlas_denom_from_i32(value: i32) -> u64 {
    let magnitude = u64::from(value.unsigned_abs());
    if value.is_negative() {
        0_u64.wrapping_sub(magnitude)
    } else {
        magnitude
    }
}

fn atlas_lattice_coefficient(value: &Integer) -> Result<Integer, StructureError> {
    let unsigned =
        u64::try_from(value).map_err(|_| StructureError::IntegerLatticeInvariantViolation)?;
    let low_word = u32::try_from(unsigned & u64::from(u32::MAX))
        .map_err(|_| StructureError::IntegerLatticeInvariantViolation)?;
    // The pinned GCC build narrows the unsigned `div_gcd` result to Atlas's
    // 32-bit signed `LatticeCoeff` by retaining this low word. Reinterpret it
    // explicitly instead of relying on Rust casts or signed overflow.
    Ok(Integer::from(i32::from_ne_bytes(low_word.to_ne_bytes())))
}

struct RelationDiagonalState<'a, 's> {
    source_entries: usize,
    matrix: IntegerMatrix,
    left: IntegerMatrix,
    budget: &'a IntegerLatticeBudget,
    steps: &'s mut usize,
}

impl<'a, 's> RelationDiagonalState<'a, 's> {
    fn new(
        source: &IntegerMatrix,
        budget: &'a IntegerLatticeBudget,
        steps: &'s mut usize,
        external_entries: usize,
    ) -> Result<Self, StructureError> {
        source.check_against(budget)?;
        let left_entries = source
            .rows
            .checked_mul(source.rows)
            .ok_or(StructureError::ArithmeticOverflow)?;
        let temporary_entries = source.columns.max(source.rows);
        let diagonal_entries = source.rows.min(source.columns);
        check_entry_total(
            &[
                external_entries,
                source.entries.len(),
                source.entries.len(),
                left_entries,
                temporary_entries,
                diagonal_entries,
            ],
            budget,
        )?;
        Ok(Self {
            source_entries: external_entries
                .checked_add(source.entries.len())
                .and_then(|entries| entries.checked_add(diagonal_entries))
                .ok_or(StructureError::ArithmeticOverflow)?,
            matrix: source.try_clone()?,
            left: IntegerMatrix::identity(source.rows, budget)?,
            budget,
            steps,
        })
    }

    fn begin_step(&mut self) -> Result<(), StructureError> {
        count_step(self.steps, self.budget)?;
        Ok(())
    }

    fn check_coefficients(&self) -> Result<(), StructureError> {
        self.matrix.check_against(self.budget)?;
        self.left.check_against(self.budget)
    }

    fn check_temporary(&self, entries: usize) -> Result<(), StructureError> {
        check_entry_total(
            &[
                self.source_entries,
                self.matrix.entries.len(),
                self.left.entries.len(),
                entries,
            ],
            self.budget,
        )
    }

    fn negate_matrix_column(&mut self, column: usize) -> Result<(), StructureError> {
        self.begin_step()?;
        negate_column(&mut self.matrix, column);
        self.check_coefficients()
    }

    fn swap_matrix_columns(&mut self, left: usize, right: usize) -> Result<(), StructureError> {
        if left == right {
            return Ok(());
        }
        self.begin_step()?;
        self.matrix.swap_columns(left, right);
        self.check_coefficients()
    }

    fn add_matrix_column_multiple(
        &mut self,
        target: usize,
        source: usize,
        factor: &Integer,
    ) -> Result<(), StructureError> {
        self.check_temporary(self.matrix.rows)?;
        self.begin_step()?;
        self.matrix.add_column_multiple(
            target,
            source,
            factor,
            self.budget.max_coefficient_bits,
        )?;
        self.check_coefficients()
    }

    fn negate_rows(&mut self, row: usize) -> Result<(), StructureError> {
        self.begin_step()?;
        self.matrix.negate_row(row);
        self.left.negate_row(row);
        self.check_coefficients()
    }

    fn swap_rows(&mut self, left: usize, right: usize) -> Result<(), StructureError> {
        if left == right {
            return Ok(());
        }
        self.begin_step()?;
        self.matrix.swap_rows(left, right);
        self.left.swap_rows(left, right);
        self.check_coefficients()
    }

    fn add_row_multiple(
        &mut self,
        target: usize,
        source: usize,
        factor: &Integer,
    ) -> Result<(), StructureError> {
        self.check_temporary(self.matrix.columns.max(self.left.columns))?;
        self.begin_step()?;
        self.matrix
            .add_row_multiple(target, source, factor, self.budget.max_coefficient_bits)?;
        self.left
            .add_row_multiple(target, source, factor, self.budget.max_coefficient_bits)?;
        self.check_coefficients()
    }

    fn negate_left_first_row(&mut self) -> Result<(), StructureError> {
        self.begin_step()?;
        self.left.negate_row(0);
        self.check_coefficients()
    }
}

fn relation_gcd_row(
    state: &mut RelationDiagonalState<'_, '_>,
    row: usize,
    pivot: usize,
) -> Result<(Integer, bool), StructureError> {
    if pivot >= state.matrix.columns {
        return Ok((Integer::ZERO, false));
    }
    let capacity = state.matrix.columns - pivot;
    let mut active = try_usize_vec(capacity)?;
    let mut minimum = Integer::ZERO;
    let mut minimum_column = pivot;
    for column in pivot..state.matrix.columns {
        let entry = state.matrix.entry(row, column);
        if entry == &Integer::ZERO {
            continue;
        }
        active.push(column);
        let magnitude = entry.clone().abs();
        if minimum == Integer::ZERO || magnitude < minimum {
            minimum = magnitude;
            minimum_column = column;
        }
    }
    if active.is_empty() {
        return Ok((Integer::ZERO, false));
    }

    let mut flipped = false;
    if state.matrix.entry(row, minimum_column) < &Integer::ZERO {
        state.negate_matrix_column(minimum_column)?;
        flipped = !flipped;
    }
    while active.len() > 1 {
        let current = minimum_column;
        let divisor = state.matrix.entry(row, current).clone();
        let mut survivors = try_usize_vec(active.len())?;
        for column in active {
            if column == current {
                survivors.push(column);
                continue;
            }
            let (quotient, remainder) = state.matrix.entry(row, column).div_mod(&divisor);
            if quotient != Integer::ZERO {
                state.add_matrix_column_multiple(column, current, &-quotient)?;
            }
            if remainder == Integer::ZERO {
                continue;
            }
            if remainder < minimum {
                minimum = remainder;
                minimum_column = column;
            }
            survivors.push(column);
        }
        active = survivors;
    }
    if minimum_column != pivot {
        state.swap_matrix_columns(pivot, minimum_column)?;
        flipped = !flipped;
    }
    Ok((minimum, flipped))
}

fn relation_gcd_column(
    state: &mut RelationDiagonalState<'_, '_>,
    column: usize,
    pivot: usize,
) -> Result<(Integer, bool), StructureError> {
    if pivot >= state.matrix.rows {
        return Ok((Integer::ZERO, false));
    }
    let capacity = state.matrix.rows - pivot;
    let mut active = try_usize_vec(capacity)?;
    let mut minimum = Integer::ZERO;
    let mut minimum_row = pivot;
    for row in pivot..state.matrix.rows {
        let entry = state.matrix.entry(row, column);
        if entry == &Integer::ZERO {
            continue;
        }
        active.push(row);
        let magnitude = entry.clone().abs();
        if minimum == Integer::ZERO || magnitude < minimum {
            minimum = magnitude;
            minimum_row = row;
        }
    }
    if active.is_empty() {
        return Ok((Integer::ZERO, false));
    }

    let mut flipped = false;
    if state.matrix.entry(minimum_row, column) < &Integer::ZERO {
        state.negate_rows(minimum_row)?;
        flipped = !flipped;
    }
    while active.len() > 1 {
        let current = minimum_row;
        let divisor = state.matrix.entry(current, column).clone();
        let mut survivors = try_usize_vec(active.len())?;
        for row in active {
            if row == current {
                survivors.push(row);
                continue;
            }
            let (quotient, remainder) = state.matrix.entry(row, column).div_mod(&divisor);
            if quotient != Integer::ZERO {
                state.add_row_multiple(row, current, &-quotient)?;
            }
            if remainder == Integer::ZERO {
                continue;
            }
            if remainder < minimum {
                minimum = remainder;
                minimum_row = row;
            }
            survivors.push(row);
        }
        active = survivors;
    }
    if minimum_row != pivot {
        state.swap_rows(pivot, minimum_row)?;
        flipped = !flipped;
    }
    Ok((minimum, flipped))
}

fn relation_diagonalise(
    matrix: &IntegerMatrix,
    budget: &IntegerLatticeBudget,
    steps: &mut usize,
    external_entries: usize,
) -> Result<(Vec<Integer>, IntegerMatrix), StructureError> {
    let mut state = RelationDiagonalState::new(matrix, budget, steps, external_entries)?;
    let mut diagonal = Vec::new();
    diagonal
        .try_reserve_exact(matrix.rows.min(matrix.columns))
        .map_err(|_| StructureError::AllocationFailed {
            requested: matrix.rows.min(matrix.columns),
        })?;
    if matrix.rows == 0 || matrix.columns == 0 {
        return Ok((diagonal, state.left));
    }

    let mut row_minus = false;
    let mut pivot_row = 0;
    for column in 0..matrix.columns {
        let (mut factor, first_flip) = relation_gcd_column(&mut state, column, pivot_row)?;
        if factor == Integer::ZERO {
            continue;
        }
        row_minus = first_flip;
        let mut last_flip;
        loop {
            let old_factor = factor.clone();
            let (next_factor, flip) = relation_gcd_row(&mut state, pivot_row, column)?;
            factor = next_factor;
            last_flip = flip;
            if factor == old_factor {
                break;
            }

            let old_factor = factor.clone();
            let (next_factor, flip) = relation_gcd_column(&mut state, column, pivot_row)?;
            factor = next_factor;
            row_minus ^= flip;
            last_flip = flip;
            if factor >= old_factor {
                break;
            }
        }
        row_minus ^= last_flip;
        diagonal.push(factor);
        pivot_row += 1;
    }
    if row_minus {
        state.negate_left_first_row()?;
    }
    Ok((diagonal, state.left))
}

fn annihilator_modulo_integer(
    matrix: &IntegerMatrix,
    denominator: &Integer,
    budget: &IntegerLatticeBudget,
    steps: &mut usize,
    external_entries: usize,
    multiplier_width: RelationMultiplierWidth,
) -> Result<IntegerMatrix, StructureError> {
    if denominator == &Integer::ZERO {
        return Err(StructureError::IntegerLatticeInvariantViolation);
    }
    let (diagonal, mut left) = relation_diagonalise(matrix, budget, steps, external_entries)?;
    let row_external = external_entries
        .checked_add(matrix.entries.len())
        .and_then(|entries| entries.checked_add(diagonal.len()))
        .ok_or(StructureError::ArithmeticOverflow)?;
    for (row, factor) in diagonal.iter().enumerate() {
        let divisor = positive_integer_gcd(denominator, factor);
        let exact_multiplier = denominator.div_exact(&divisor);
        let multiplier = match multiplier_width {
            RelationMultiplierWidth::Exact => exact_multiplier,
            RelationMultiplierWidth::AtlasLatticeCoefficient => {
                atlas_lattice_coefficient(&exact_multiplier)?
            }
        };
        multiply_row_bounded(&mut left, row, &multiplier, row_external, budget, steps)?;
    }
    drop(diagonal);
    let live_external = external_entries
        .checked_add(matrix.entries.len())
        .ok_or(StructureError::ArithmeticOverflow)?;
    transpose_bounded(&left, live_external, budget)
}

fn positive_integer_gcd(left: &Integer, right: &Integer) -> Integer {
    let mut left = left.clone().abs();
    let mut right = right.clone().abs();
    while right != Integer::ZERO {
        let (_, remainder) = left.div_mod(&right);
        left = right;
        right = remainder;
    }
    left
}

fn multiply_row_bounded(
    matrix: &mut IntegerMatrix,
    row: usize,
    factor: &Integer,
    external_entries: usize,
    budget: &IntegerLatticeBudget,
    steps: &mut usize,
) -> Result<(), StructureError> {
    check_entry_total(
        &[external_entries, matrix.entries.len(), matrix.columns],
        budget,
    )?;
    count_step(steps, budget)?;
    let mut replacement = try_integer_vec(matrix.columns)?;
    for column in 0..matrix.columns {
        replacement.push(bounded_linear_combination(
            factor,
            matrix.entry(row, column),
            &Integer::ZERO,
            &Integer::ZERO,
            budget.max_coefficient_bits,
        )?);
    }
    for (column, value) in replacement.into_iter().enumerate() {
        matrix.set(row, column, value);
    }
    Ok(())
}

fn transpose_bounded(
    matrix: &IntegerMatrix,
    external_entries: usize,
    budget: &IntegerLatticeBudget,
) -> Result<IntegerMatrix, StructureError> {
    let output_entries = matrix
        .rows
        .checked_mul(matrix.columns)
        .ok_or(StructureError::ArithmeticOverflow)?;
    check_entry_total(
        &[external_entries, matrix.entries.len(), output_entries],
        budget,
    )?;
    let mut result = IntegerMatrix::zero(matrix.columns, matrix.rows, budget)?;
    for row in 0..matrix.rows {
        for column in 0..matrix.columns {
            result.set(column, row, matrix.entry(row, column).clone());
        }
    }
    Ok(result)
}

/// Assemble the weight-lattice basis selected by rational centre generators.
pub fn quotient_relation_basis(
    smith_basis: &RelationMatrix,
    factors: &[i32],
    generators: &[RelationGenerator<'_>],
    budget: &IntegerLatticeBudget,
) -> Result<RelationMatrix, RelationError> {
    let filtered = filter_relation_units(smith_basis, factors, budget)?;
    let filtered_rank = filtered.factors.len();
    for (generator_index, generator) in generators.iter().enumerate() {
        if generator.numerators.len() != filtered_rank {
            return Err(RelationError::GeneratorLengthMismatch {
                generator: generator_index,
                actual: generator.numerators.len(),
                expected: filtered_rank,
            });
        }
    }

    let numerator_entries = filtered_rank
        .checked_mul(generators.len())
        .ok_or(StructureError::ArithmeticOverflow)?;
    check_entry_total(
        &[
            smith_basis.0.entries.len(),
            filtered.basis.0.entries.len(),
            numerator_entries,
            generators.len(),
            1,
            4,
        ],
        budget,
    )?;
    let mut numerators = IntegerMatrix::zero(filtered_rank, generators.len(), budget)?;
    let mut denominators = try_integer_vec(generators.len())?;
    let mut common_denominator = Integer::ONE;
    let mut steps = 0;

    for (column, generator) in generators.iter().enumerate() {
        let denominator = Integer::from(generator.denominator.get());
        count_step(&mut steps, budget)?;
        let divisor = positive_integer_gcd(&common_denominator, &denominator);
        let reduced = common_denominator.div_exact(&divisor);
        common_denominator = bounded_linear_combination(
            &reduced,
            &denominator,
            &Integer::ZERO,
            &Integer::ZERO,
            budget.max_coefficient_bits,
        )?;
        denominators.push(denominator.clone());

        for (row, (&numerator, &factor)) in generator
            .numerators
            .iter()
            .zip(&filtered.factors)
            .enumerate()
        {
            let numerator_integer = Integer::from(numerator);
            count_step(&mut steps, budget)?;
            let factored = bounded_linear_combination(
                &Integer::from(factor),
                &numerator_integer,
                &Integer::ZERO,
                &Integer::ZERO,
                budget.max_coefficient_bits,
            )?;
            if factored.div_mod(&denominator).1 != Integer::ZERO {
                return Err(RelationError::ImproperGeneratorEntry {
                    numerator,
                    denominator: generator.denominator.get(),
                    factor,
                });
            }
            numerators.set(row, column, numerator_integer.div_mod(&denominator).1);
        }
    }

    for (column, denominator) in denominators.iter().enumerate() {
        let multiplier = (&common_denominator).div_exact(denominator);
        if multiplier == Integer::ONE {
            continue;
        }
        check_entry_total(
            &[
                smith_basis.0.entries.len(),
                filtered.basis.0.entries.len(),
                numerators.entries.len(),
                denominators.len(),
                1,
                numerators.rows,
                2,
            ],
            budget,
        )?;
        count_step(&mut steps, budget)?;
        let mut replacement = try_integer_vec(numerators.rows)?;
        for row in 0..numerators.rows {
            replacement.push(bounded_linear_combination(
                &multiplier,
                numerators.entry(row, column),
                &Integer::ZERO,
                &Integer::ZERO,
                budget.max_coefficient_bits,
            )?);
        }
        for (row, value) in replacement.into_iter().enumerate() {
            numerators.set(row, column, value);
        }
    }

    let external_entries = smith_basis
        .0
        .entries
        .len()
        .checked_add(filtered.basis.0.entries.len())
        .and_then(|entries| entries.checked_add(denominators.len()))
        .and_then(|entries| entries.checked_add(1))
        .ok_or(StructureError::ArithmeticOverflow)?;
    let annihilator = annihilator_modulo_integer(
        &numerators,
        &common_denominator,
        budget,
        &mut steps,
        external_entries,
        RelationMultiplierWidth::Exact,
    )?;
    drop(numerators);
    drop(denominators);
    drop(common_denominator);

    let replacements = multiply_relation_matrices(
        &filtered.basis.0,
        &annihilator,
        smith_basis.0.entries.len(),
        budget,
        &mut steps,
    )?;
    drop(annihilator);
    drop(filtered);
    replace_relation_generators(smith_basis, factors, &RelationMatrix(replacements), budget)
}

fn multiply_relation_matrices(
    left: &IntegerMatrix,
    right: &IntegerMatrix,
    external_entries: usize,
    budget: &IntegerLatticeBudget,
    steps: &mut usize,
) -> Result<IntegerMatrix, StructureError> {
    if left.columns != right.rows {
        return Err(StructureError::InvalidIntegerMatrixShape);
    }
    let output_entries = left
        .rows
        .checked_mul(right.columns)
        .ok_or(StructureError::ArithmeticOverflow)?;
    check_entry_total(
        &[
            external_entries,
            left.entries.len(),
            right.entries.len(),
            output_entries,
            1,
        ],
        budget,
    )?;
    let mut result = IntegerMatrix::zero(left.rows, right.columns, budget)?;
    for row in 0..left.rows {
        for column in 0..right.columns {
            let mut value = Integer::ZERO;
            for inner in 0..left.columns {
                count_step(steps, budget)?;
                value = bounded_linear_combination(
                    &Integer::ONE,
                    &value,
                    left.entry(row, inner),
                    right.entry(inner, column),
                    budget.max_coefficient_bits,
                )?;
            }
            result.set(row, column, value);
        }
    }
    result.check_against(budget)?;
    Ok(result)
}

fn try_usize_vec(capacity: usize) -> Result<Vec<usize>, StructureError> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(capacity)
        .map_err(|_| StructureError::AllocationFailed {
            requested: capacity,
        })?;
    Ok(values)
}

/// The result of [`adapted_basis`]: a unimodular basis `B` of the ambient
/// lattice, its inverse, and positive diagonal factors such that the input's
/// column span is exactly `span{ diagonal[t] * B.column(t) }` — so the FIRST
/// `diagonal.len()` columns of `B` span the SATURATION of that image.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdaptedBasis {
    pub basis: IntegerMatrix,
    pub inverse: IntegerMatrix,
    pub diagonal: Vec<Integer>,
}

/// Faithful port of upstream `matreduc::adapted_basis` (matreduc.cpp:262-336
/// with its `gcd` helper, matreduc.h:70-122), tracking the LEFT transform's
/// inverse alongside instead of inverting afterwards.
///
/// The pivot strategy is replicated exactly — "minimal use of row
/// operations", the first-minimal gcd seed, `find_small_remainder`'s rotate
/// step, and the kept-rows-first reordering — because the elected basis is
/// OBSERVABLE-BEARING: it fixes the `stable_log` representative, hence
/// `g_rho_check` and every `torus_factor` rational downstream.
pub fn adapted_basis(
    rows: &[Vec<i32>],
    budget: &IntegerLatticeBudget,
) -> Result<AdaptedBasis, StructureError> {
    let mut matrix = IntegerMatrix::from_i32_rows(rows, budget)?;
    let height = matrix.rows;
    let width = matrix.columns;
    let square_entries = height
        .checked_mul(height)
        .ok_or(StructureError::ArithmeticOverflow)?;
    let diagonal_entries = height.min(width);
    // The working matrix, basis, and inverse are live throughout. A stable
    // permutation temporarily materialises one additional square matrix.
    check_entry_total(
        &[
            matrix.entries.len(),
            square_entries,
            square_entries,
            square_entries,
            diagonal_entries,
        ],
        budget,
    )?;
    let mut basis = IntegerMatrix::identity(height, budget)?;
    let mut inverse = IntegerMatrix::identity(height, budget)?;
    let mut diagonal = Vec::new();
    diagonal
        .try_reserve_exact(diagonal_entries)
        .map_err(|_| StructureError::AllocationFailed {
            requested: diagonal_entries,
        })?;
    let mut kept = Vec::new();
    kept.try_reserve_exact(height)
        .map_err(|_| StructureError::AllocationFailed { requested: height })?;
    kept.resize(height, false);
    let mut steps = 0_usize;
    let mut pivot_column = 0_usize;

    for (row, kept_flag) in kept.iter_mut().enumerate() {
        let mut d = gcd_row_to_pivot(&mut matrix, row, pivot_column, budget, &mut steps)?;
        if d == Integer::ZERO {
            continue;
        }
        *kept_flag = true;
        while let Some(other) = find_small_remainder(&matrix, row, pivot_column) {
            let (quotient, _) = matrix
                .entry(other, pivot_column)
                .div_mod(matrix.entry(row, pivot_column));
            count_step(&mut steps, budget)?;
            for column in pivot_column..width {
                let current = matrix.entry(row, column).clone();
                let replacement = bounded_linear_combination(
                    &Integer::ONE,
                    matrix.entry(other, column),
                    &-&quotient,
                    &current,
                    budget.max_coefficient_bits,
                )?;
                matrix.set(row, column, replacement);
                matrix.set(other, column, current);
            }
            for column in 0..height {
                let current = inverse.entry(row, column).clone();
                let replacement = bounded_linear_combination(
                    &Integer::ONE,
                    inverse.entry(other, column),
                    &-&quotient,
                    &current,
                    budget.max_coefficient_bits,
                )?;
                inverse.set(row, column, replacement);
                inverse.set(other, column, current);
            }
            for target in 0..height {
                let current = basis.entry(target, other).clone();
                let replacement = bounded_linear_combination(
                    &Integer::ONE,
                    basis.entry(target, row),
                    &quotient,
                    &current,
                    budget.max_coefficient_bits,
                )?;
                basis.set(target, other, replacement);
                basis.set(target, row, current);
            }
            d = gcd_row_to_pivot(&mut matrix, row, pivot_column, budget, &mut steps)?;
            if d == Integer::ZERO {
                return Err(StructureError::IntegerLatticeInvariantViolation);
            }
        }
        for other in (row + 1)..height {
            if *matrix.entry(other, pivot_column) == Integer::ZERO {
                continue;
            }
            if !matrix
                .entry(other, pivot_column)
                .divisible_by(matrix.entry(row, pivot_column))
            {
                return Err(StructureError::IntegerLatticeInvariantViolation);
            }
            let quotient = matrix
                .entry(other, pivot_column)
                .div_exact(matrix.entry(row, pivot_column));
            count_step(&mut steps, budget)?;
            // Conceptual row op `row_other -= q * row_row` on the matrix is
            // skipped (upstream ignores that column below the pivot), but
            // BOTH transforms must record it.
            for column in 0..height {
                let update = bounded_linear_combination(
                    &Integer::ONE,
                    inverse.entry(other, column),
                    &-&quotient,
                    inverse.entry(row, column),
                    budget.max_coefficient_bits,
                )?;
                inverse.set(other, column, update);
            }
            for target in 0..height {
                let update = bounded_linear_combination(
                    &Integer::ONE,
                    basis.entry(target, row),
                    &quotient,
                    basis.entry(target, other),
                    budget.max_coefficient_bits,
                )?;
                basis.set(target, row, update);
            }
        }
        diagonal.push(d);
        pivot_column += 1;
        matrix.check_against(budget)?;
        basis.check_against(budget)?;
        inverse.check_against(budget)?;
    }

    if kept.iter().any(|&flag| !flag) {
        let mut order = Vec::new();
        order
            .try_reserve_exact(height)
            .map_err(|_| StructureError::AllocationFailed { requested: height })?;
        order.extend((0..height).filter(|&index| kept[index]));
        order.extend((0..height).filter(|&index| !kept[index]));
        basis = permute_columns(&basis, &order, budget)?;
        inverse = permute_rows(&inverse, &order, budget)?;
    }

    Ok(AdaptedBasis {
        basis,
        inverse,
        diagonal,
    })
}

/// Upstream `matreduc::gcd` with `dest = 0`, operating directly on the
/// matrix's columns `pivot..` so no separate ops matrix is needed: makes the
/// entry at `(row, pivot)` the positive gcd of the row's tail and zeroes the
/// rest of that tail, by the exact upstream operation sequence.
fn gcd_row_to_pivot(
    matrix: &mut IntegerMatrix,
    row: usize,
    pivot: usize,
    budget: &IntegerLatticeBudget,
    steps: &mut usize,
) -> Result<Integer, StructureError> {
    let width = matrix.columns;
    let mut active = Vec::new();
    active
        .try_reserve_exact(width - pivot)
        .map_err(|_| StructureError::AllocationFailed {
            requested: width - pivot,
        })?;
    let mut minimum = Integer::ZERO;
    let mut minimum_column = pivot;
    for column in pivot..width {
        let value = matrix.entry(row, column);
        if *value != Integer::ZERO {
            active.push(column);
            let magnitude = value.clone().abs();
            if minimum == Integer::ZERO || magnitude < minimum {
                minimum = magnitude;
                minimum_column = column;
            }
        }
    }
    if active.is_empty() {
        return Ok(Integer::ZERO);
    }
    if *matrix.entry(row, minimum_column) < Integer::ZERO {
        count_step(steps, budget)?;
        negate_column(matrix, minimum_column);
    }
    while active.len() > 1 {
        let current = minimum_column;
        let divisor = matrix.entry(row, current).clone();
        let mut survivors = Vec::new();
        survivors.try_reserve_exact(active.len()).map_err(|_| {
            StructureError::AllocationFailed {
                requested: active.len(),
            }
        })?;
        for &column in &active {
            if column == current {
                survivors.push(column);
                continue;
            }
            let (quotient, remainder) = matrix.entry(row, column).div_mod(&divisor);
            if quotient != Integer::ZERO {
                count_step(steps, budget)?;
                matrix.add_column_multiple(
                    column,
                    current,
                    &-&quotient,
                    budget.max_coefficient_bits,
                )?;
            }
            if remainder == Integer::ZERO {
                continue;
            }
            if remainder < minimum {
                minimum = remainder;
                minimum_column = column;
            }
            survivors.push(column);
        }
        active = survivors;
    }
    if minimum_column != pivot {
        count_step(steps, budget)?;
        swap_matrix_columns(matrix, minimum_column, pivot);
    }
    Ok(minimum)
}

fn swap_matrix_columns(matrix: &mut IntegerMatrix, left: usize, right: usize) {
    for row in 0..matrix.rows {
        let left_value = matrix.entry(row, left).clone();
        let right_value = matrix.entry(row, right).clone();
        matrix.set(row, left, right_value);
        matrix.set(row, right, left_value);
    }
}

/// Upstream `find_small_remainder` over the pivot column below `row`: the
/// first row whose remainder by the pivot is the smallest POSITIVE one.
fn find_small_remainder(matrix: &IntegerMatrix, row: usize, pivot: usize) -> Option<usize> {
    let divisor = matrix.entry(row, pivot);
    let mut best: Option<(Integer, usize)> = None;
    for candidate in (row + 1)..matrix.rows {
        let (_, remainder) = matrix.entry(candidate, pivot).div_mod(divisor);
        if remainder == Integer::ZERO {
            continue;
        }
        match &best {
            Some((minimum, _)) if remainder >= *minimum => {}
            _ => best = Some((remainder, candidate)),
        }
    }
    best.map(|(_, candidate)| candidate)
}

fn count_step(steps: &mut usize, budget: &IntegerLatticeBudget) -> Result<(), StructureError> {
    if *steps == budget.max_steps {
        return Err(resource_limit("reduction steps", budget.max_steps as u64));
    }
    *steps += 1;
    Ok(())
}

fn negate_column(matrix: &mut IntegerMatrix, column: usize) {
    for row in 0..matrix.rows {
        let value = -matrix.entry(row, column).clone();
        matrix.set(row, column, value);
    }
}

fn permute_columns(
    matrix: &IntegerMatrix,
    order: &[usize],
    budget: &IntegerLatticeBudget,
) -> Result<IntegerMatrix, StructureError> {
    let mut result = IntegerMatrix::zero(matrix.rows, matrix.columns, budget)?;
    for (target, &source) in order.iter().enumerate() {
        for row in 0..matrix.rows {
            result.set(row, target, matrix.entry(row, source).clone());
        }
    }
    Ok(result)
}

fn permute_rows(
    matrix: &IntegerMatrix,
    order: &[usize],
    budget: &IntegerLatticeBudget,
) -> Result<IntegerMatrix, StructureError> {
    let mut result = IntegerMatrix::zero(matrix.rows, matrix.columns, budget)?;
    for (target, &source) in order.iter().enumerate() {
        for column in 0..matrix.columns {
            result.set(target, column, matrix.entry(source, column).clone());
        }
    }
    Ok(result)
}

struct ReductionState<'a> {
    source_entries: usize,
    matrix: IntegerMatrix,
    right: IntegerMatrix,
    budget: &'a IntegerLatticeBudget,
    steps: usize,
}

impl<'a> ReductionState<'a> {
    fn new(
        matrix: &IntegerMatrix,
        budget: &'a IntegerLatticeBudget,
    ) -> Result<Self, StructureError> {
        let right_entries = matrix
            .columns
            .checked_mul(matrix.columns)
            .ok_or(StructureError::ArithmeticOverflow)?;
        check_entry_total(
            &[matrix.entries.len(), matrix.entries.len(), right_entries],
            budget,
        )?;
        let state = Self {
            source_entries: matrix.entries.len(),
            matrix: matrix.try_clone()?,
            right: IntegerMatrix::identity(matrix.columns, budget)?,
            budget,
            steps: 0,
        };
        state.check_bounds()?;
        Ok(state)
    }

    fn first_nonzero(&self, start: usize) -> Option<(usize, usize)> {
        for row in start..self.matrix.rows {
            for column in start..self.matrix.columns {
                if self.matrix.entry(row, column) != &Integer::ZERO {
                    return Some((row, column));
                }
            }
        }
        None
    }

    fn nonzero_below(&self, pivot: usize) -> Option<usize> {
        ((pivot + 1)..self.matrix.rows).find(|&row| self.matrix.entry(row, pivot) != &Integer::ZERO)
    }

    fn nonzero_right(&self, pivot: usize) -> Option<usize> {
        ((pivot + 1)..self.matrix.columns)
            .find(|&column| self.matrix.entry(pivot, column) != &Integer::ZERO)
    }

    fn swap_rows(&mut self, left: usize, right: usize) -> Result<(), StructureError> {
        if left == right {
            return Ok(());
        }
        self.matrix.swap_rows(left, right);
        self.advance()
    }

    fn swap_columns(&mut self, left: usize, right: usize) -> Result<(), StructureError> {
        if left == right {
            return Ok(());
        }
        self.matrix.swap_columns(left, right);
        self.right.swap_columns(left, right);
        self.advance()
    }

    fn negate_row(&mut self, row: usize) -> Result<(), StructureError> {
        self.matrix.negate_row(row);
        self.advance()
    }

    fn eliminate_below(&mut self, pivot: usize, row: usize) -> Result<(), StructureError> {
        let max_coefficient_bits = self.budget.max_coefficient_bits;
        if self
            .matrix
            .entry(row, pivot)
            .divisible_by(self.matrix.entry(pivot, pivot))
        {
            let factor = -self
                .matrix
                .entry(row, pivot)
                .div_exact(self.matrix.entry(pivot, pivot));
            self.check_live_entries(self.matrix.columns)?;
            self.matrix
                .add_row_multiple(row, pivot, &factor, max_coefficient_bits)?;
            return self.advance();
        }
        let transform = BezoutTransform::for_entries(
            self.matrix.entry(pivot, pivot),
            self.matrix.entry(row, pivot),
        );
        self.check_live_entries(double_entries(self.matrix.columns)?)?;
        self.matrix
            .bezout_rows(pivot, row, &transform, max_coefficient_bits)?;
        self.advance()
    }

    fn eliminate_right(&mut self, pivot: usize, column: usize) -> Result<(), StructureError> {
        let max_coefficient_bits = self.budget.max_coefficient_bits;
        if self
            .matrix
            .entry(pivot, column)
            .divisible_by(self.matrix.entry(pivot, pivot))
        {
            let factor = -self
                .matrix
                .entry(pivot, column)
                .div_exact(self.matrix.entry(pivot, pivot));
            self.check_live_entries(self.matrix.rows.max(self.right.rows))?;
            self.matrix
                .add_column_multiple(column, pivot, &factor, max_coefficient_bits)?;
            self.right
                .add_column_multiple(column, pivot, &factor, max_coefficient_bits)?;
            return self.advance();
        }
        let transform = BezoutTransform::for_entries(
            self.matrix.entry(pivot, pivot),
            self.matrix.entry(pivot, column),
        );
        self.check_live_entries(double_entries(self.matrix.rows.max(self.right.rows))?)?;
        self.matrix
            .bezout_columns(pivot, column, &transform, max_coefficient_bits)?;
        self.right
            .bezout_columns(pivot, column, &transform, max_coefficient_bits)?;
        self.advance()
    }

    fn advance(&mut self) -> Result<(), StructureError> {
        self.steps = self
            .steps
            .checked_add(1)
            .ok_or(StructureError::ArithmeticOverflow)?;
        if self.steps > self.budget.max_steps {
            return Err(resource_limit(
                "elementary operations",
                limit_as_u64(self.budget.max_steps)?,
            ));
        }
        // No full-matrix revalidation here: shapes and entry counts are fixed
        // after `new` checked them, every written entry passed the exact bit
        // bound inside `bounded_linear_combination`, and swaps/negations
        // preserve significant bits — so a per-step `check_bounds` scan can
        // never observe a violation the write path did not already reject.
        Ok(())
    }

    fn check_bounds(&self) -> Result<(), StructureError> {
        self.matrix.check_against(self.budget)?;
        self.right.check_against(self.budget)?;
        self.check_live_entries(0)
    }

    fn check_live_entries(&self, additional: usize) -> Result<(), StructureError> {
        check_entry_total(
            &[
                self.source_entries,
                self.matrix.entries.len(),
                self.right.entries.len(),
                additional,
            ],
            self.budget,
        )
    }
}

fn checked_shape(
    rows: usize,
    columns: usize,
    budget: &IntegerLatticeBudget,
) -> Result<usize, StructureError> {
    if rows > budget.max_rank || columns > budget.max_rank {
        return Err(resource_limit("rank", limit_as_u64(budget.max_rank)?));
    }
    let entries = rows
        .checked_mul(columns)
        .ok_or(StructureError::ArithmeticOverflow)?;
    if entries > budget.max_entries {
        return Err(resource_limit(
            "stored entries",
            limit_as_u64(budget.max_entries)?,
        ));
    }
    Ok(entries)
}

fn try_integer_vec(capacity: usize) -> Result<Vec<Integer>, StructureError> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(capacity)
        .map_err(|_| StructureError::AllocationFailed {
            requested: capacity,
        })?;
    Ok(values)
}

fn check_entry_total(parts: &[usize], budget: &IntegerLatticeBudget) -> Result<(), StructureError> {
    let entries = parts.iter().try_fold(0_usize, |total, &part| {
        total
            .checked_add(part)
            .ok_or(StructureError::ArithmeticOverflow)
    })?;
    if entries > budget.max_entries {
        return Err(resource_limit(
            "stored entries",
            limit_as_u64(budget.max_entries)?,
        ));
    }
    Ok(())
}

fn double_entries(entries: usize) -> Result<usize, StructureError> {
    entries
        .checked_mul(2)
        .ok_or(StructureError::ArithmeticOverflow)
}

fn resource_limit(resource: &'static str, limit: u64) -> StructureError {
    StructureError::IntegerLatticeResourceLimit { resource, limit }
}

fn limit_as_u64(limit: usize) -> Result<u64, StructureError> {
    u64::try_from(limit).map_err(|_| StructureError::ArithmeticOverflow)
}

fn bounded_linear_combination(
    left_factor: &Integer,
    left_value: &Integer,
    right_factor: &Integer,
    right_value: &Integer,
    max_coefficient_bits: u64,
) -> Result<Integer, StructureError> {
    // Fast path: all four operands fit an i64 (the common case — lattice
    // entries in involution-table work stay machine-sized, as upstream's
    // `long`-based lattice.cpp assumes). The bound check below is the exact
    // i64 image of the generic one, so both paths agree on acceptance; the
    // products fit i128 with room (|i64|^2 < 2^126), and only a result that
    // outgrows i64 falls through to the generic arithmetic.
    if let (Ok(left_factor), Ok(left_value), Ok(right_factor), Ok(right_value)) = (
        i64::try_from(left_factor),
        i64::try_from(left_value),
        i64::try_from(right_factor),
        i64::try_from(right_value),
    ) {
        let left_bits = product_bit_bound_i64(left_factor, left_value);
        let right_bits = product_bit_bound_i64(right_factor, right_value);
        let bound = match (left_bits, right_bits) {
            (0, bits) | (bits, 0) => bits,
            // Each side is at most 128, so this cannot overflow.
            (left, right) => left.max(right) + 1,
        };
        if bound > max_coefficient_bits {
            return Err(resource_limit("coefficient bits", max_coefficient_bits));
        }
        let value = (left_factor as i128) * (left_value as i128)
            + (right_factor as i128) * (right_value as i128);
        if let Ok(small) = i64::try_from(value) {
            return Ok(Integer::from(small));
        }
        // Rare: a machine-sized result window was exceeded. Fall through to
        // the generic path, which recomputes the same (already-checked) value.
    }
    let left_bits = product_bit_bound(left_factor, left_value)?;
    let right_bits = product_bit_bound(right_factor, right_value)?;
    let bound = match (left_bits, right_bits) {
        (0, bits) | (bits, 0) => bits,
        (left, right) => left
            .max(right)
            .checked_add(1)
            .ok_or_else(|| resource_limit("coefficient bits", max_coefficient_bits))?,
    };
    if bound > max_coefficient_bits {
        return Err(resource_limit("coefficient bits", max_coefficient_bits));
    }
    Ok(left_factor * left_value + right_factor * right_value)
}

fn product_bit_bound(left: &Integer, right: &Integer) -> Result<u64, StructureError> {
    if left == &Integer::ZERO || right == &Integer::ZERO {
        return Ok(0);
    }
    left.significant_bits()
        .checked_add(right.significant_bits())
        .ok_or_else(|| resource_limit("coefficient bits", u64::MAX))
}

/// `significant_bits` of a nonzero i64 (bits of its absolute value).
fn significant_bits_i64(value: i64) -> u64 {
    debug_assert_ne!(value, 0);
    64 - u64::from(value.unsigned_abs().leading_zeros())
}

/// The i64 image of [`product_bit_bound`]; each operand contributes at most
/// 64 bits, so the sum cannot overflow.
fn product_bit_bound_i64(left: i64, right: i64) -> u64 {
    if left == 0 || right == 0 {
        return 0;
    }
    significant_bits_i64(left) + significant_bits_i64(right)
}

fn verify_annihilation(
    matrix: &IntegerMatrix,
    basis: &IntegralBasis,
    budget: &IntegerLatticeBudget,
) -> Result<(), StructureError> {
    if basis.ambient_rank != matrix.columns {
        return Err(StructureError::RankMismatch {
            expected: matrix.columns,
            actual: basis.ambient_rank,
        });
    }
    for column in &basis.columns {
        if column.len() != basis.ambient_rank {
            return Err(StructureError::IntegerLatticeInvariantViolation);
        }
        for row in 0..matrix.rows {
            let mut value = Integer::ZERO;
            for (coefficient, coordinate) in
                (0..matrix.columns).map(|index| (matrix.entry(row, index), &column[index]))
            {
                value = bounded_linear_combination(
                    &Integer::ONE,
                    &value,
                    coefficient,
                    coordinate,
                    budget.max_coefficient_bits,
                )?;
            }
            if value != Integer::ZERO {
                return Err(StructureError::IntegerLatticeInvariantViolation);
            }
        }
    }
    Ok(())
}

// ---- machine-word twin of the saturated-kernel reduction ----
//
// Every function below is the i64 image of its generic counterpart above,
// in the same operation order and with the same budget checks, so both
// paths accept, reject, and compute identically. The twin speaks
// `Result<Option<T>, StructureError>`: `Err` is a real (budget/shape)
// failure to propagate, `Ok(None)` means a value left the i64 window and
// the caller restarts on the bignum path — the SmallRat pattern.

/// Row-major i64 matrix mirroring [`IntegerMatrix`].
struct SmallIntMatrix {
    rows: usize,
    columns: usize,
    entries: Vec<i64>,
}

impl SmallIntMatrix {
    /// The i64 image of `matrix`, or `None` when an entry does not fit.
    fn from_integer_matrix(
        matrix: &IntegerMatrix,
    ) -> Result<Option<Self>, StructureError> {
        let mut entries = Vec::new();
        entries
            .try_reserve_exact(matrix.entries.len())
            .map_err(|_| StructureError::AllocationFailed {
                requested: matrix.entries.len(),
            })?;
        for entry in &matrix.entries {
            match i64::try_from(entry) {
                Ok(value) => entries.push(value),
                Err(_) => return Ok(None),
            }
        }
        Ok(Some(Self {
            rows: matrix.rows,
            columns: matrix.columns,
            entries,
        }))
    }

    fn identity(rank: usize, budget: &IntegerLatticeBudget) -> Result<Self, StructureError> {
        let entry_count = checked_shape(rank, rank, budget)?;
        let mut entries = Vec::new();
        entries
            .try_reserve_exact(entry_count)
            .map_err(|_| StructureError::AllocationFailed {
                requested: entry_count,
            })?;
        entries.resize(entry_count, 0);
        for index in 0..rank {
            entries[index * rank + index] = 1;
        }
        Ok(Self {
            rows: rank,
            columns: rank,
            entries,
        })
    }

    fn entry(&self, row: usize, column: usize) -> i64 {
        self.entries[row * self.columns + column]
    }

    fn check_against(&self, budget: &IntegerLatticeBudget) -> Result<(), StructureError> {
        checked_shape(self.rows, self.columns, budget)?;
        if self
            .entries
            .iter()
            .any(|&entry| entry != 0 && significant_bits_i64(entry) > budget.max_coefficient_bits)
        {
            return Err(resource_limit(
                "coefficient bits",
                budget.max_coefficient_bits,
            ));
        }
        Ok(())
    }

    fn swap_rows(&mut self, left: usize, right: usize) {
        if left == right {
            return;
        }
        for column in 0..self.columns {
            self.entries
                .swap(left * self.columns + column, right * self.columns + column);
        }
    }

    fn swap_columns(&mut self, left: usize, right: usize) {
        if left == right {
            return;
        }
        for row in 0..self.rows {
            self.entries
                .swap(row * self.columns + left, row * self.columns + right);
        }
    }

    fn negate_row(&mut self, row: usize) -> Option<()> {
        for column in 0..self.columns {
            let index = row * self.columns + column;
            self.entries[index] = self.entries[index].checked_neg()?;
        }
        Some(())
    }

    /// `target += factor * source` per row, one entry at a time (see the
    /// generic add_row_multiple for why in-place order is safe).
    fn add_row_multiple(
        &mut self,
        target: usize,
        source: usize,
        factor: i64,
        max_coefficient_bits: u64,
    ) -> Result<Option<()>, StructureError> {
        for column in 0..self.columns {
            let target_index = target * self.columns + column;
            let source_index = source * self.columns + column;
            let Some(value) = small_bounded_combination(
                1,
                self.entries[target_index],
                factor,
                self.entries[source_index],
                max_coefficient_bits,
            )?
            else {
                return Ok(None);
            };
            self.entries[target_index] = value;
        }
        Ok(Some(()))
    }

    fn add_column_multiple(
        &mut self,
        target: usize,
        source: usize,
        factor: i64,
        max_coefficient_bits: u64,
    ) -> Result<Option<()>, StructureError> {
        for row in 0..self.rows {
            let target_index = row * self.columns + target;
            let source_index = row * self.columns + source;
            let Some(value) = small_bounded_combination(
                1,
                self.entries[target_index],
                factor,
                self.entries[source_index],
                max_coefficient_bits,
            )?
            else {
                return Ok(None);
            };
            self.entries[target_index] = value;
        }
        Ok(Some(()))
    }

    fn bezout_rows(
        &mut self,
        top: usize,
        bottom: usize,
        transform: &SmallBezout,
        max_coefficient_bits: u64,
    ) -> Result<Option<()>, StructureError> {
        let Some(negative_v) = transform.v.checked_neg() else {
            return Ok(None);
        };
        for column in 0..self.columns {
            let top_index = top * self.columns + column;
            let bottom_index = bottom * self.columns + column;
            let Some(top_value) = small_bounded_combination(
                transform.s,
                self.entries[top_index],
                transform.t,
                self.entries[bottom_index],
                max_coefficient_bits,
            )?
            else {
                return Ok(None);
            };
            let Some(bottom_value) = small_bounded_combination(
                negative_v,
                self.entries[top_index],
                transform.u,
                self.entries[bottom_index],
                max_coefficient_bits,
            )?
            else {
                return Ok(None);
            };
            self.entries[top_index] = top_value;
            self.entries[bottom_index] = bottom_value;
        }
        Ok(Some(()))
    }

    fn bezout_columns(
        &mut self,
        left: usize,
        right: usize,
        transform: &SmallBezout,
        max_coefficient_bits: u64,
    ) -> Result<Option<()>, StructureError> {
        let Some(negative_v) = transform.v.checked_neg() else {
            return Ok(None);
        };
        for row in 0..self.rows {
            let left_index = row * self.columns + left;
            let right_index = row * self.columns + right;
            let Some(left_value) = small_bounded_combination(
                transform.s,
                self.entries[left_index],
                transform.t,
                self.entries[right_index],
                max_coefficient_bits,
            )?
            else {
                return Ok(None);
            };
            let Some(right_value) = small_bounded_combination(
                negative_v,
                self.entries[left_index],
                transform.u,
                self.entries[right_index],
                max_coefficient_bits,
            )?
            else {
                return Ok(None);
            };
            self.entries[left_index] = left_value;
            self.entries[right_index] = right_value;
        }
        Ok(Some(()))
    }
}

/// The i64 image of [`BezoutTransform`].
struct SmallBezout {
    s: i64,
    t: i64,
    u: i64,
    v: i64,
}

/// Bézout coefficients of `a`, `b` with the exact malachite convention
/// (extended Euclid over the absolute values, cofactor signs flipped for
/// negative inputs — integer/arithmetic/extended_gcd.rs); `None` when a
/// coefficient leaves the i64 window.
fn small_bezout(a: i64, b: i64) -> Option<SmallBezout> {
    let (mut old_r, mut r) = (u128::from(a.unsigned_abs()), u128::from(b.unsigned_abs()));
    let (mut old_s, mut s) = (1_i128, 0_i128);
    let (mut old_t, mut t) = (0_i128, 1_i128);
    while r != 0 {
        let quotient = old_r / r;
        (old_r, r) = (r, old_r - quotient * r);
        let quotient = i128::try_from(quotient).ok()?;
        (old_s, s) = (s, old_s - quotient * s);
        (old_t, t) = (t, old_t - quotient * t);
    }
    let gcd = i128::from(u64::try_from(old_r).ok()?);
    let s = if a < 0 { -old_s } else { old_s };
    let t = if b < 0 { -old_t } else { old_t };
    Some(SmallBezout {
        s: i64::try_from(s).ok()?,
        t: i64::try_from(t).ok()?,
        u: i64::try_from(i128::from(a) / gcd).ok()?,
        v: i64::try_from(i128::from(b) / gcd).ok()?,
    })
}

/// The i64 image of the [`bounded_linear_combination`] fast path: same bit
/// bound, same budget error; `None` when the result leaves i64 (the generic
/// path would widen instead).
fn small_bounded_combination(
    left_factor: i64,
    left_value: i64,
    right_factor: i64,
    right_value: i64,
    max_coefficient_bits: u64,
) -> Result<Option<i64>, StructureError> {
    let left_bits = product_bit_bound_i64(left_factor, left_value);
    let right_bits = product_bit_bound_i64(right_factor, right_value);
    let bound = match (left_bits, right_bits) {
        (0, bits) | (bits, 0) => bits,
        (left, right) => left.max(right) + 1,
    };
    if bound > max_coefficient_bits {
        return Err(resource_limit("coefficient bits", max_coefficient_bits));
    }
    let value = i128::from(left_factor) * i128::from(left_value)
        + i128::from(right_factor) * i128::from(right_value);
    Ok(i64::try_from(value).ok())
}

/// The i64 image of [`ReductionState`].
struct SmallReductionState {
    source_entries: usize,
    matrix: SmallIntMatrix,
    right: SmallIntMatrix,
    steps: usize,
}

impl SmallReductionState {
    fn new(
        matrix: &IntegerMatrix,
        budget: &IntegerLatticeBudget,
    ) -> Result<Option<Self>, StructureError> {
        let right_entries = matrix
            .columns
            .checked_mul(matrix.columns)
            .ok_or(StructureError::ArithmeticOverflow)?;
        check_entry_total(
            &[matrix.entries.len(), matrix.entries.len(), right_entries],
            budget,
        )?;
        let Some(small_matrix) = SmallIntMatrix::from_integer_matrix(matrix)? else {
            return Ok(None);
        };
        let state = Self {
            source_entries: matrix.entries.len(),
            matrix: small_matrix,
            right: SmallIntMatrix::identity(matrix.columns, budget)?,
            steps: 0,
        };
        state.check_bounds(budget)?;
        Ok(Some(state))
    }

    fn first_nonzero(&self, start: usize) -> Option<(usize, usize)> {
        for row in start..self.matrix.rows {
            for column in start..self.matrix.columns {
                if self.matrix.entry(row, column) != 0 {
                    return Some((row, column));
                }
            }
        }
        None
    }

    fn nonzero_below(&self, pivot: usize) -> Option<usize> {
        ((pivot + 1)..self.matrix.rows).find(|&row| self.matrix.entry(row, pivot) != 0)
    }

    fn nonzero_right(&self, pivot: usize) -> Option<usize> {
        ((pivot + 1)..self.matrix.columns).find(|&column| self.matrix.entry(pivot, column) != 0)
    }

    fn swap_rows(
        &mut self,
        left: usize,
        right: usize,
        budget: &IntegerLatticeBudget,
    ) -> Result<(), StructureError> {
        if left == right {
            return Ok(());
        }
        self.matrix.swap_rows(left, right);
        self.advance(budget)
    }

    fn swap_columns(
        &mut self,
        left: usize,
        right: usize,
        budget: &IntegerLatticeBudget,
    ) -> Result<(), StructureError> {
        if left == right {
            return Ok(());
        }
        self.matrix.swap_columns(left, right);
        self.right.swap_columns(left, right);
        self.advance(budget)
    }

    fn negate_row(
        &mut self,
        row: usize,
        budget: &IntegerLatticeBudget,
    ) -> Result<Option<()>, StructureError> {
        if self.matrix.negate_row(row).is_none() {
            return Ok(None);
        }
        self.advance(budget)?;
        Ok(Some(()))
    }

    fn eliminate_below(
        &mut self,
        pivot: usize,
        row: usize,
        budget: &IntegerLatticeBudget,
    ) -> Result<Option<()>, StructureError> {
        let max_coefficient_bits = budget.max_coefficient_bits;
        let pivot_value = self.matrix.entry(pivot, pivot);
        let entry = self.matrix.entry(row, pivot);
        let Some(remainder) = entry.checked_rem(pivot_value) else {
            return Ok(None);
        };
        if remainder == 0 {
            let Some(factor) = entry
                .checked_div(pivot_value)
                .and_then(|quotient| quotient.checked_neg())
            else {
                return Ok(None);
            };
            self.check_live_entries(self.matrix.columns, budget)?;
            let Some(()) = self
                .matrix
                .add_row_multiple(row, pivot, factor, max_coefficient_bits)?
            else {
                return Ok(None);
            };
            self.advance(budget)?;
            return Ok(Some(()));
        }
        let Some(transform) = small_bezout(pivot_value, entry) else {
            return Ok(None);
        };
        self.check_live_entries(double_entries(self.matrix.columns)?, budget)?;
        let Some(()) = self
            .matrix
            .bezout_rows(pivot, row, &transform, max_coefficient_bits)?
        else {
            return Ok(None);
        };
        self.advance(budget)?;
        Ok(Some(()))
    }

    fn eliminate_right(
        &mut self,
        pivot: usize,
        column: usize,
        budget: &IntegerLatticeBudget,
    ) -> Result<Option<()>, StructureError> {
        let max_coefficient_bits = budget.max_coefficient_bits;
        let pivot_value = self.matrix.entry(pivot, pivot);
        let entry = self.matrix.entry(pivot, column);
        let Some(remainder) = entry.checked_rem(pivot_value) else {
            return Ok(None);
        };
        if remainder == 0 {
            let Some(factor) = entry
                .checked_div(pivot_value)
                .and_then(|quotient| quotient.checked_neg())
            else {
                return Ok(None);
            };
            self.check_live_entries(self.matrix.rows.max(self.right.rows), budget)?;
            let Some(()) = self
                .matrix
                .add_column_multiple(column, pivot, factor, max_coefficient_bits)?
            else {
                return Ok(None);
            };
            let Some(()) = self
                .right
                .add_column_multiple(column, pivot, factor, max_coefficient_bits)?
            else {
                return Ok(None);
            };
            self.advance(budget)?;
            return Ok(Some(()));
        }
        let Some(transform) = small_bezout(pivot_value, entry) else {
            return Ok(None);
        };
        self.check_live_entries(
            double_entries(self.matrix.rows.max(self.right.rows))?,
            budget,
        )?;
        let Some(()) = self
            .matrix
            .bezout_columns(pivot, column, &transform, max_coefficient_bits)?
        else {
            return Ok(None);
        };
        let Some(()) = self
            .right
            .bezout_columns(pivot, column, &transform, max_coefficient_bits)?
        else {
            return Ok(None);
        };
        self.advance(budget)?;
        Ok(Some(()))
    }

    fn advance(&mut self, budget: &IntegerLatticeBudget) -> Result<(), StructureError> {
        self.steps = self
            .steps
            .checked_add(1)
            .ok_or(StructureError::ArithmeticOverflow)?;
        if self.steps > budget.max_steps {
            return Err(resource_limit(
                "elementary operations",
                limit_as_u64(budget.max_steps)?,
            ));
        }
        Ok(())
    }

    fn check_bounds(&self, budget: &IntegerLatticeBudget) -> Result<(), StructureError> {
        self.matrix.check_against(budget)?;
        self.right.check_against(budget)?;
        self.check_live_entries(0, budget)
    }

    fn check_live_entries(
        &self,
        additional: usize,
        budget: &IntegerLatticeBudget,
    ) -> Result<(), StructureError> {
        check_entry_total(
            &[
                self.source_entries,
                self.matrix.entries.len(),
                self.right.entries.len(),
                additional,
            ],
            budget,
        )
    }
}

/// The machine-word twin of [`saturated_kernel_generic`]; see the module
/// note above for the `Ok(None)` fallback convention.
fn try_saturated_kernel_small(
    matrix: &IntegerMatrix,
    budget: &IntegerLatticeBudget,
) -> Result<Option<IntegralBasis>, StructureError> {
    let Some(mut state) = SmallReductionState::new(matrix, budget)? else {
        return Ok(None);
    };
    let diagonal_limit = state.matrix.rows.min(state.matrix.columns);
    let mut diagonal_rank = 0;

    while diagonal_rank < diagonal_limit {
        let Some((pivot_row, pivot_column)) = state.first_nonzero(diagonal_rank) else {
            break;
        };
        state.swap_rows(diagonal_rank, pivot_row, budget)?;
        state.swap_columns(diagonal_rank, pivot_column, budget)?;

        loop {
            while let Some(row) = state.nonzero_below(diagonal_rank) {
                let Some(()) = state.eliminate_below(diagonal_rank, row, budget)? else {
                    return Ok(None);
                };
            }
            while let Some(column) = state.nonzero_right(diagonal_rank) {
                let Some(()) = state.eliminate_right(diagonal_rank, column, budget)? else {
                    return Ok(None);
                };
            }
            if state.nonzero_below(diagonal_rank).is_none()
                && state.nonzero_right(diagonal_rank).is_none()
            {
                break;
            }
        }

        if state.matrix.entry(diagonal_rank, diagonal_rank) < 0 {
            let Some(()) = state.negate_row(diagonal_rank, budget)? else {
                return Ok(None);
            };
        }
        diagonal_rank += 1;
    }

    let SmallReductionState {
        matrix: working_matrix,
        right,
        ..
    } = state;
    drop(working_matrix);
    let kernel_rank = right
        .columns
        .checked_sub(diagonal_rank)
        .ok_or(StructureError::IntegerLatticeInvariantViolation)?;
    let output_entries = right
        .rows
        .checked_mul(kernel_rank)
        .ok_or(StructureError::ArithmeticOverflow)?;
    check_entry_total(
        &[matrix.entries.len(), right.entries.len(), output_entries],
        budget,
    )?;
    // `IntegralBasis::kernel_columns`, converting once at the end.
    let ambient_rank = right.rows;
    check_entry_total(&[right.entries.len(), output_entries], budget)?;
    let mut columns = Vec::new();
    columns
        .try_reserve_exact(kernel_rank)
        .map_err(|_| StructureError::AllocationFailed {
            requested: kernel_rank,
        })?;
    for column in diagonal_rank..right.columns {
        let mut coordinates = try_integer_vec(ambient_rank)?;
        for row in 0..ambient_rank {
            coordinates.push(Integer::from(right.entry(row, column)));
        }
        columns.push(coordinates);
    }
    let basis = IntegralBasis {
        ambient_rank,
        columns,
    };
    drop(right);
    verify_annihilation_small(matrix, &basis, budget)?;
    Ok(Some(basis))
}

/// The i64 image of [`verify_annihilation`]: the same per-step bit bounds
/// against `max_coefficient_bits`, accumulating in i128 (the generic path's
/// wide accumulation agrees on every value that fits). Overflow of the
/// i128 accumulator is unreachable for budget-bounded inputs.
fn verify_annihilation_small(
    matrix: &IntegerMatrix,
    basis: &IntegralBasis,
    budget: &IntegerLatticeBudget,
) -> Result<(), StructureError> {
    if basis.ambient_rank != matrix.columns {
        return Err(StructureError::RankMismatch {
            expected: matrix.columns,
            actual: basis.ambient_rank,
        });
    }
    for column in &basis.columns {
        if column.len() != basis.ambient_rank {
            return Err(StructureError::IntegerLatticeInvariantViolation);
        }
        for row in 0..matrix.rows {
            let mut value = 0_i128;
            for index in 0..matrix.columns {
                let coefficient = i64::try_from(matrix.entry(row, index))
                    .map_err(|_| StructureError::IntegerLatticeInvariantViolation)?;
                let coordinate = i64::try_from(&column[index])
                    .map_err(|_| StructureError::IntegerLatticeInvariantViolation)?;
                let value_bits = if value == 0 {
                    0
                } else {
                    128 - u32::from(value.unsigned_abs().leading_zeros()) as u64
                };
                let left_bits = value_bits; // 1 * value
                let right_bits = product_bit_bound_i64(coefficient, coordinate);
                let bound = match (left_bits, right_bits) {
                    (0, bits) | (bits, 0) => bits,
                    (left, right) => left.max(right) + 1,
                };
                if bound > budget.max_coefficient_bits {
                    return Err(resource_limit("coefficient bits", budget.max_coefficient_bits));
                }
                value += i128::from(coefficient) * i128::from(coordinate);
            }
            if value != 0 {
                return Err(StructureError::IntegerLatticeInvariantViolation);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn budget() -> IntegerLatticeBudget {
        IntegerLatticeBudget::new(16, 256, 1_000, 128)
    }

    fn multiply(left: &IntegerMatrix, right: &IntegerMatrix) -> Vec<Vec<Integer>> {
        (0..left.rows)
            .map(|row| {
                (0..right.columns)
                    .map(|column| {
                        (0..left.columns).fold(Integer::ZERO, |sum, middle| {
                            sum + left.entry(row, middle) * right.entry(middle, column)
                        })
                    })
                    .collect()
            })
            .collect()
    }

    fn assert_identity(product: &[Vec<Integer>]) {
        for (row_index, row) in product.iter().enumerate() {
            for (column_index, value) in row.iter().enumerate() {
                let expected = if row_index == column_index {
                    Integer::ONE
                } else {
                    Integer::ZERO
                };
                assert_eq!(*value, expected, "at ({row_index},{column_index})");
            }
        }
    }

    #[test]
    fn adapted_basis_of_the_swap_involution_spans_the_fixed_lattice() {
        // xi = swap, so xi + 1 = [[1,1],[1,1]]: image Z(1,1), saturation Z(1,1).
        let budget = IntegerLatticeBudget::new(16, 256, 1_000, 128);
        let adapted = adapted_basis(&[vec![1, 1], vec![1, 1]], &budget).unwrap();
        assert_eq!(adapted.diagonal, vec![Integer::ONE]);
        assert_eq!(*adapted.basis.entry(0, 0), Integer::ONE);
        assert_eq!(*adapted.basis.entry(1, 0), Integer::ONE);
        assert_identity(&multiply(&adapted.basis, &adapted.inverse));
    }

    #[test]
    fn adapted_basis_handles_doubling_and_zero_maps() {
        let budget = IntegerLatticeBudget::new(16, 256, 1_000, 128);
        // xi = identity: xi + 1 = 2I — full rank, diagonal all 2.
        let doubling = adapted_basis(&[vec![2, 0], vec![0, 2]], &budget).unwrap();
        assert_eq!(doubling.diagonal, vec![Integer::from(2), Integer::from(2)]);
        assert_identity(&multiply(&doubling.basis, &doubling.inverse));
        // xi = -identity: xi + 1 = 0 — empty diagonal, identity transforms.
        let zero = adapted_basis(&[vec![0, 0], vec![0, 0]], &budget).unwrap();
        assert!(zero.diagonal.is_empty());
        assert_identity(&multiply(&zero.basis, &zero.inverse));
    }

    #[test]
    fn adapted_basis_reproduces_the_image_span_for_a_mixed_matrix() {
        let budget = IntegerLatticeBudget::new(16, 256, 1_000, 128);
        let rows = vec![vec![2, 4, 0], vec![0, 6, 0], vec![2, 10, 0]];
        let adapted = adapted_basis(&rows, &budget).unwrap();
        assert_identity(&multiply(&adapted.basis, &adapted.inverse));
        // B^{-1} * M must have row t divisible by diagonal[t] for t < d and
        // zero rows below — the exact image-span certificate.
        let original = matrix(&[&[2, 4, 0], &[0, 6, 0], &[2, 10, 0]]);
        let transformed = multiply(&adapted.inverse, &original);
        let rank = adapted.diagonal.len();
        for (row_index, row) in transformed.iter().enumerate() {
            for value in row {
                if row_index < rank {
                    assert!(value.divisible_by(&adapted.diagonal[row_index]));
                } else {
                    assert_eq!(*value, Integer::ZERO);
                }
            }
        }
    }

    #[test]
    fn adapted_basis_respects_the_step_budget() {
        let starved = IntegerLatticeBudget::new(16, 256, 1, 128);
        assert_eq!(
            adapted_basis(&[vec![3, 5], vec![7, 11]], &starved),
            Err(StructureError::IntegerLatticeResourceLimit {
                resource: "reduction steps",
                limit: 1,
            })
        );
    }

    #[test]
    fn relation_adapted_basis_matches_the_observable_a2_election() {
        let adapted = adapted_relation_basis(&[vec![2, -1], vec![-1, 2]], &budget()).unwrap();
        assert_eq!(
            adapted.basis().try_i32_rows().unwrap(),
            vec![vec![1, 0], vec![-2, 1]]
        );
        assert_eq!(adapted.factors(), &[1, 3]);
    }

    #[test]
    fn relation_annihilator_matches_the_frozen_basis_and_annihilates_modulo_d() {
        let modulus = std::num::NonZeroI32::new(2).unwrap();
        let input = vec![vec![2, 6], vec![4, 3]];
        let input_matrix = RelationMatrix::from_i32_rows(&input, &budget()).unwrap();
        let annihilator = annihilator_modulo(&input_matrix, modulus, &budget()).unwrap();
        let rows = annihilator.try_i32_rows().unwrap();
        assert_eq!(rows, vec![vec![-1, 4], vec![0, -2]]);

        for column in 0..rows.first().map_or(0, Vec::len) {
            for input_column in 0..input.first().map_or(0, Vec::len) {
                let pairing = rows
                    .iter()
                    .zip(&input)
                    .map(|(row, input_row)| {
                        i64::from(row[column]) * i64::from(input_row[input_column])
                    })
                    .sum::<i64>();
                assert_eq!(pairing.rem_euclid(i64::from(modulus.get())), 0);
            }
        }
    }

    #[test]
    fn relation_annihilator_preserves_negative_modulus_semantics() {
        for (entry, modulus, expected) in [(1, -2, -2), (3, -3, -3)] {
            let input = RelationMatrix::from_i32_rows(&[vec![entry]], &budget()).unwrap();
            let result = annihilator_modulo(
                &input,
                std::num::NonZeroI32::new(modulus).unwrap(),
                &budget(),
            )
            .unwrap();
            assert_eq!(result.try_i32_rows().unwrap(), vec![vec![expected]]);
        }
    }

    #[test]
    fn relation_inputs_preflight_before_iterating_or_copying() {
        use std::cell::Cell;

        let matrix_visited = Cell::new(false);
        let over_rank = IntegerLatticeBudget::new(1, 16, 16, 32);
        let matrix = RelationMatrix::from_i32_iter(
            2,
            1,
            (0..2).map(|_| {
                matrix_visited.set(true);
                0
            }),
            &over_rank,
        );
        assert_eq!(
            matrix,
            Err(RelationError::Structure(
                StructureError::IntegerLatticeResourceLimit {
                    resource: "rank",
                    limit: 1,
                }
            ))
        );
        assert!(!matrix_visited.get());

        let generator_visited = Cell::new(false);
        let generators = RelationGenerator::try_collect(
            1,
            (0..3).map(|_| {
                generator_visited.set(true);
                RelationGenerator::new(&[], NonZeroU64::new(1).unwrap())
            }),
            &IntegerLatticeBudget::new(2, 16, 16, 32),
        );
        assert_eq!(
            generators,
            Err(RelationError::Structure(
                StructureError::IntegerLatticeResourceLimit {
                    resource: "rank",
                    limit: 2,
                }
            ))
        );
        assert!(!generator_visited.get());

        let stored_entry_visited = Cell::new(false);
        let generators = RelationGenerator::try_collect(
            2,
            (0..1).map(|_| {
                stored_entry_visited.set(true);
                RelationGenerator::new(&[], NonZeroU64::new(1).unwrap())
            }),
            &IntegerLatticeBudget::new(2, 2, 16, 32),
        );
        assert_eq!(
            generators,
            Err(RelationError::Structure(
                StructureError::IntegerLatticeResourceLimit {
                    resource: "stored entries",
                    limit: 2,
                }
            ))
        );
        assert!(!stored_entry_visited.get());
    }

    #[test]
    fn relation_reduction_enforces_step_and_coefficient_budgets() {
        let step_budget = IntegerLatticeBudget::new(4, 128, 0, 128);
        let step_input =
            RelationMatrix::from_i32_rows(&[vec![2, 6], vec![4, 3]], &step_budget).unwrap();
        assert_eq!(
            annihilator_modulo(
                &step_input,
                std::num::NonZeroI32::new(2).unwrap(),
                &step_budget,
            ),
            Err(RelationError::Structure(
                StructureError::IntegerLatticeResourceLimit {
                    resource: "reduction steps",
                    limit: 0,
                }
            ))
        );
        let coefficient_budget = IntegerLatticeBudget::new(1, 8, 16, 8);
        let coefficient_input =
            RelationMatrix::from_i32_rows(&[vec![1]], &coefficient_budget).unwrap();
        assert_eq!(
            annihilator_modulo(
                &coefficient_input,
                std::num::NonZeroI32::new(i32::MAX).unwrap(),
                &coefficient_budget,
            ),
            Err(RelationError::Structure(
                StructureError::IntegerLatticeResourceLimit {
                    resource: "coefficient bits",
                    limit: 8,
                }
            ))
        );

        let entry_budget = IntegerLatticeBudget::new(2, 15, 32, 128);
        let entry_input =
            RelationMatrix::from_i32_rows(&[vec![2, 6], vec![4, 3]], &entry_budget).unwrap();
        assert_eq!(
            annihilator_modulo(
                &entry_input,
                std::num::NonZeroI32::new(2).unwrap(),
                &entry_budget,
            ),
            Err(RelationError::Structure(
                StructureError::IntegerLatticeResourceLimit {
                    resource: "stored entries",
                    limit: 15,
                }
            ))
        );
    }

    #[test]
    fn relation_adapted_basis_counts_all_live_exact_matrices() {
        assert_eq!(
            adapted_relation_basis(
                &[vec![2, -1], vec![-1, 2]],
                &IntegerLatticeBudget::new(2, 11, 32, 32),
            ),
            Err(RelationError::Structure(
                StructureError::IntegerLatticeResourceLimit {
                    resource: "stored entries",
                    limit: 11,
                }
            ))
        );
    }

    #[test]
    fn quotient_relation_uses_nonnegative_residues_for_negative_generators() {
        let budget = budget();
        let smith = RelationMatrix::from_i32_rows(&[vec![1]], &budget).unwrap();
        let numerator = [-1_i64];
        let generators = [RelationGenerator::new(
            &numerator,
            NonZeroU64::new(4).unwrap(),
        )];
        let quotient = quotient_relation_basis(&smith, &[4], &generators, &budget).unwrap();
        assert_eq!(quotient.try_i32_rows().unwrap(), vec![vec![4]]);

        let remainder = Integer::from(-1).div_mod(&Integer::from(4)).1;
        assert_eq!(remainder, Integer::from(3));
    }

    #[test]
    fn relation_matrix_validates_shape_and_rank_before_reduction() {
        assert_eq!(
            RelationMatrix::from_i32_entries(2, 2, &[1, 2, 3], &budget()),
            Err(RelationError::Structure(
                StructureError::InvalidIntegerMatrixShape
            ))
        );
        assert_eq!(
            RelationMatrix::from_i32_entries(2, 1, &[1, 2], &IntegerLatticeBudget::new(1, 8, 8, 8),),
            Err(RelationError::Structure(
                StructureError::IntegerLatticeResourceLimit {
                    resource: "rank",
                    limit: 1,
                }
            ))
        );
    }

    fn matrix(rows: &[&[i32]]) -> IntegerMatrix {
        IntegerMatrix::from_i32_rows(
            &rows.iter().map(|row| row.to_vec()).collect::<Vec<_>>(),
            &budget(),
        )
        .unwrap()
    }

    #[test]
    fn linear_combination_fast_path_matches_generic_arithmetic() {
        let cases: &[(i64, i64, i64, i64)] = &[
            (1, 7, -3, 5),
            (-1, i64::MIN, 0, 0),
            (2, -9, 2, 9),
            (0, 0, 1, -1),
            (i64::MAX, 1, 1, -4),
        ];
        for &(lf, lv, rf, rv) in cases {
            let expected = Integer::from(lf) * Integer::from(lv)
                + Integer::from(rf) * Integer::from(rv);
            assert_eq!(
                bounded_linear_combination(
                    &Integer::from(lf),
                    &Integer::from(lv),
                    &Integer::from(rf),
                    &Integer::from(rv),
                    256,
                )
                .unwrap(),
                expected,
            );
        }
    }

    #[test]
    fn linear_combination_fast_path_keeps_the_coefficient_budget() {
        // significant_bits(16) = 5 per side, so the bound is 6 > 4: the fast
        // path must reject exactly like the generic one.
        let sixteen = Integer::from(16);
        assert_eq!(
            bounded_linear_combination(&sixteen, &sixteen, &Integer::ZERO, &Integer::ZERO, 4),
            Err(StructureError::IntegerLatticeResourceLimit {
                resource: "coefficient bits",
                limit: 4,
            })
        );
        // A result beyond the i64 window (2^40 * 2^40 = 2^80, bound 82)
        // falls through to the generic arithmetic with the same value.
        let base = Integer::from(1_i64 << 40);
        let expected = &base * &base;
        assert_eq!(
            bounded_linear_combination(&base, &base, &Integer::ZERO, &Integer::ZERO, 128).unwrap(),
            expected,
        );
    }

    fn integer_column(values: &[i32]) -> Vec<Integer> {
        values.iter().copied().map(Integer::from).collect()
    }

    #[test]
    fn zero_matrix_has_the_full_standard_kernel() {
        let kernel = saturated_kernel(&matrix(&[&[0, 0, 0], &[0, 0, 0]]), &budget()).unwrap();
        assert_eq!(kernel.ambient_rank(), 3);
        assert_eq!(kernel.rank(), 3);
        assert_eq!(
            kernel.columns(),
            &[
                integer_column(&[1, 0, 0]),
                integer_column(&[0, 1, 0]),
                integer_column(&[0, 0, 1]),
            ]
        );
    }

    #[test]
    fn full_rank_diagonal_matrix_has_zero_kernel() {
        let kernel = saturated_kernel(&matrix(&[&[2, 0], &[0, -3]]), &budget()).unwrap();
        assert_eq!(kernel.ambient_rank(), 2);
        assert_eq!(kernel.rank(), 0);
    }

    #[test]
    fn kernel_uses_a_primitive_integral_generator() {
        let kernel = saturated_kernel(&matrix(&[&[2, 4]]), &budget()).unwrap();
        assert_eq!(kernel.columns(), &[integer_column(&[-2, 1])]);
    }

    #[test]
    fn bezout_column_reduction_keeps_the_kernel_primitive() {
        let kernel = saturated_kernel(&matrix(&[&[2, 3]]), &budget()).unwrap();
        assert_eq!(kernel.columns(), &[integer_column(&[-3, 2])]);
    }

    #[test]
    fn alternating_row_and_column_reductions_preserve_every_kernel_column() {
        let kernel = saturated_kernel(&matrix(&[&[2, 3, 0], &[4, 6, 0]]), &budget()).unwrap();
        assert_eq!(
            kernel.columns(),
            &[integer_column(&[-3, 2, 0]), integer_column(&[0, 0, 1])]
        );
    }

    #[test]
    fn swap_involution_has_the_expected_integral_and_mod_two_denominator() {
        let kernel = negative_coweight_eigenspace(&[vec![0, 1], vec![1, 0]], &budget()).unwrap();
        assert_eq!(kernel.columns(), &[integer_column(&[-1, 1])]);
        let parity = reduce_basis_mod_two(&kernel).unwrap();
        assert_eq!(parity.dimension(), 2);
        assert_eq!(parity.rank(), 1);
        assert!(parity
            .contains(ModTwoVector::from_ones(2, [0, 1]).unwrap())
            .unwrap());
    }

    #[test]
    fn plus_or_minus_identity_give_the_expected_negative_eigenspaces() {
        let negative =
            negative_coweight_eigenspace(&[vec![-1, 0], vec![0, -1]], &budget()).unwrap();
        assert_eq!(negative.rank(), 2);
        assert_eq!(reduce_basis_mod_two(&negative).unwrap().rank(), 2);

        let positive = negative_coweight_eigenspace(&[vec![1, 0], vec![0, 1]], &budget()).unwrap();
        assert_eq!(positive.rank(), 0);
    }

    #[test]
    fn resource_budgets_fail_before_unbounded_reduction() {
        assert_eq!(
            IntegerMatrix::from_i32_rows(
                &[vec![1, 0], vec![0, 1]],
                &IntegerLatticeBudget::new(1, 4, 8, 8),
            ),
            Err(StructureError::IntegerLatticeResourceLimit {
                resource: "rank",
                limit: 1,
            })
        );
        assert_eq!(
            IntegerMatrix::from_i32_rows(&[vec![2]], &IntegerLatticeBudget::new(1, 1, 8, 1),),
            Err(StructureError::IntegerLatticeResourceLimit {
                resource: "coefficient bits",
                limit: 1,
            })
        );
        assert_eq!(
            // source (2) + working copy (2) + V (4) + the largest column
            // replacement (2) must fit before the zero-step boundary runs.
            saturated_kernel(&matrix(&[&[2, 4]]), &IntegerLatticeBudget::new(2, 10, 0, 8),),
            Err(StructureError::IntegerLatticeResourceLimit {
                resource: "elementary operations",
                limit: 0,
            })
        );
        assert_eq!(
            saturated_kernel(&matrix(&[&[2, 4]]), &IntegerLatticeBudget::new(2, 5, 8, 8),),
            Err(StructureError::IntegerLatticeResourceLimit {
                resource: "stored entries",
                limit: 5,
            })
        );
    }

    #[test]
    fn aggregate_entry_budget_counts_source_temporaries_and_output_basis() {
        assert!(matches!(
            saturated_kernel(&matrix(&[&[0]]), &IntegerLatticeBudget::new(1, 2, 8, 8),),
            Err(StructureError::IntegerLatticeResourceLimit {
                resource: "stored entries",
                limit: 2,
            })
        ));
        assert!(matches!(
            saturated_kernel(
                &matrix(&[&[2, 4, 0]]),
                &IntegerLatticeBudget::new(3, 17, 8, 8),
            ),
            Err(StructureError::IntegerLatticeResourceLimit {
                resource: "stored entries",
                limit: 17,
            })
        ));
        // The zero matrix performs no row or column reduction. This boundary
        // therefore isolates the later point where the source matrix, V, and
        // the materialized kernel basis coexist.
        assert!(matches!(
            saturated_kernel(
                &matrix(&[&[0, 0, 0], &[0, 0, 0]]),
                &IntegerLatticeBudget::new(3, 22, 8, 8),
            ),
            Err(StructureError::IntegerLatticeResourceLimit {
                resource: "stored entries",
                limit: 22,
            })
        ));
    }

    #[test]
    fn row_column_operations_match_build_then_swap_reference() {
        let base = matrix(&[&[2, 4, -6], &[1, 0, 3], &[-5, 2, 1]]);
        let factor = Integer::from(-3);
        for (target, source) in [(0_usize, 1_usize), (2, 0), (1, 2)] {
            let mut in_place = base.clone();
            in_place
                .add_row_multiple(target, source, &factor, 256)
                .unwrap();
            let mut reference = base.clone();
            for column in 0..reference.columns {
                let value = bounded_linear_combination(
                    &Integer::ONE,
                    reference.entry(target, column),
                    &factor,
                    reference.entry(source, column),
                    256,
                )
                .unwrap();
                reference.set(target, column, value);
            }
            assert_eq!(in_place, reference, "add_row_multiple {target} {source}");

            let mut in_place = base.clone();
            in_place
                .add_column_multiple(target, source, &factor, 256)
                .unwrap();
            let mut reference = base.clone();
            for row in 0..reference.rows {
                let value = bounded_linear_combination(
                    &Integer::ONE,
                    reference.entry(row, target),
                    &factor,
                    reference.entry(row, source),
                    256,
                )
                .unwrap();
                reference.set(row, target, value);
            }
            assert_eq!(in_place, reference, "add_column_multiple {target} {source}");
        }
    }

    #[test]
    fn bezout_operations_match_build_then_swap_reference() {
        let base = matrix(&[&[6, 4, -6], &[9, 0, 3], &[-15, 2, 1]]);
        for (first, second) in [(0_usize, 1_usize), (1, 2), (0, 2)] {
            let transform = BezoutTransform::for_entries(
                base.entry(first, first),
                base.entry(second, first),
            );
            let negative_v = -&transform.v;

            let mut in_place = base.clone();
            in_place
                .bezout_rows(first, second, &transform, 256)
                .unwrap();
            let mut reference = base.clone();
            for column in 0..reference.columns {
                let top = bounded_linear_combination(
                    &transform.s,
                    reference.entry(first, column),
                    &transform.t,
                    reference.entry(second, column),
                    256,
                )
                .unwrap();
                let bottom = bounded_linear_combination(
                    &negative_v,
                    reference.entry(first, column),
                    &transform.u,
                    reference.entry(second, column),
                    256,
                )
                .unwrap();
                reference.set(first, column, top);
                reference.set(second, column, bottom);
            }
            assert_eq!(in_place, reference, "bezout_rows {first} {second}");
        }
    }

    #[test]
    fn failed_row_operation_reports_the_budget_error() {
        // 16 * 16 has bit bound 11 > 4: the operation must fail. The matrix
        // may be left partially rewritten — the resource-limit error is
        // fatal to every caller, which drops the matrix.
        let mut work = matrix(&[&[16, 1], &[3, 16]]);
        assert_eq!(
            work.add_row_multiple(1, 0, &Integer::from(16), 4),
            Err(StructureError::IntegerLatticeResourceLimit {
                resource: "coefficient bits",
                limit: 4,
            })
        );
    }

    #[test]
    fn small_bezout_matches_malachite_convention() {
        let cases: [(i64, i64); 14] = [
            (6, 9),
            (6, -9),
            (-6, 9),
            (-6, -9),
            (240, 46),
            (-111, 300),
            (3, 5),
            (5, 5),
            (10, 5),
            (5, 10),
            (1, 1),
            (-1, -1),
            (i64::MAX - 1, i64::MAX - 2),
            (i64::MIN + 1, 3),
        ];
        for (a, b) in cases {
            let small = small_bezout(a, b).expect("all cases fit i64");
            let reference = BezoutTransform::for_entries(&Integer::from(a), &Integer::from(b));
            assert_eq!(small.s, i64::try_from(&reference.s).unwrap(), "s {a} {b}");
            assert_eq!(small.t, i64::try_from(&reference.t).unwrap(), "t {a} {b}");
            assert_eq!(small.u, i64::try_from(&reference.u).unwrap(), "u {a} {b}");
            assert_eq!(small.v, i64::try_from(&reference.v).unwrap(), "v {a} {b}");
        }
    }

    #[test]
    fn small_kernel_matches_generic_kernel() {
        let fixed: Vec<Vec<Vec<i32>>> = vec![
            vec![vec![1, 0], vec![0, 1]],
            vec![vec![0, 0], vec![0, 0]],
            vec![vec![2, 0], vec![0, 3]],
            vec![vec![1, 1], vec![1, -1]],
            vec![vec![6, 9], vec![-4, 7]],
            vec![vec![240, 46, 0], vec![-111, 300, 5]],
            vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]],
            vec![vec![-2, 1, 2, -3], vec![-3, -1, 2, -11]],
            vec![vec![5], vec![-7]],
            vec![vec![5, -7]],
        ];
        let mut matrices: Vec<IntegerMatrix> =
            fixed.iter().map(|rows| matrix(&rows.iter().map(Vec::as_slice).collect::<Vec<_>>())).collect();
        // Deterministic LCG batch for volume: mixed-sign 4x4 and 3x5.
        let mut state = 0x1234_5678_9abc_def0_u64;
        let mut next = move || {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            ((state >> 33) % 21) as i32 - 10
        };
        for _ in 0..40 {
            let rows: Vec<Vec<i32>> = (0..4).map(|_| (0..4).map(|_| next()).collect()).collect();
            matrices.push(matrix(&rows.iter().map(Vec::as_slice).collect::<Vec<_>>()));
            let rows: Vec<Vec<i32>> = (0..3).map(|_| (0..5).map(|_| next()).collect()).collect();
            matrices.push(matrix(&rows.iter().map(Vec::as_slice).collect::<Vec<_>>()));
        }
        for work in &matrices {
            let small = try_saturated_kernel_small(work, &budget())
                .unwrap()
                .expect("small entries stay in the i64 window");
            let reference = saturated_kernel_generic(work, &budget()).unwrap();
            assert_eq!(small, reference, "kernel mismatch for {work:?}");
        }
    }
}
