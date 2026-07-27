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
pub(crate) struct IntegerMatrix {
    rows: usize,
    columns: usize,
    entries: Vec<Integer>,
}

impl IntegerMatrix {
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

    fn add_row_multiple(
        &mut self,
        target: usize,
        source: usize,
        factor: &Integer,
        max_coefficient_bits: u64,
    ) -> Result<(), StructureError> {
        let mut replacement = try_integer_vec(self.columns)?;
        for column in 0..self.columns {
            replacement.push(bounded_linear_combination(
                &Integer::ONE,
                self.entry(target, column),
                factor,
                self.entry(source, column),
                max_coefficient_bits,
            )?);
        }
        for (column, value) in replacement.into_iter().enumerate() {
            self.set(target, column, value);
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
        let mut replacement = try_integer_vec(self.rows)?;
        for row in 0..self.rows {
            replacement.push(bounded_linear_combination(
                &Integer::ONE,
                self.entry(row, target),
                factor,
                self.entry(row, source),
                max_coefficient_bits,
            )?);
        }
        for (row, value) in replacement.into_iter().enumerate() {
            self.set(row, target, value);
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
        let mut top_replacement = try_integer_vec(self.columns)?;
        let mut bottom_replacement = try_integer_vec(self.columns)?;
        let negative_v = -&transform.v;
        for column in 0..self.columns {
            let top_value = self.entry(top, column);
            let bottom_value = self.entry(bottom, column);
            top_replacement.push(bounded_linear_combination(
                &transform.s,
                top_value,
                &transform.t,
                bottom_value,
                max_coefficient_bits,
            )?);
            bottom_replacement.push(bounded_linear_combination(
                &negative_v,
                top_value,
                &transform.u,
                bottom_value,
                max_coefficient_bits,
            )?);
        }
        for (column, (top_value, bottom_value)) in top_replacement
            .into_iter()
            .zip(bottom_replacement)
            .enumerate()
        {
            self.set(top, column, top_value);
            self.set(bottom, column, bottom_value);
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
        let mut left_replacement = try_integer_vec(self.rows)?;
        let mut right_replacement = try_integer_vec(self.rows)?;
        let negative_v = -&transform.v;
        for row in 0..self.rows {
            let left_value = self.entry(row, left);
            let right_value = self.entry(row, right);
            left_replacement.push(bounded_linear_combination(
                &transform.s,
                left_value,
                &transform.t,
                right_value,
                max_coefficient_bits,
            )?);
            right_replacement.push(bounded_linear_combination(
                &negative_v,
                left_value,
                &transform.u,
                right_value,
                max_coefficient_bits,
            )?);
        }
        for (row, (left_value, right_value)) in left_replacement
            .into_iter()
            .zip(right_replacement)
            .enumerate()
        {
            self.set(row, left, left_value);
            self.set(row, right, right_value);
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

    #[cfg(test)]
    pub(crate) fn rank(&self) -> usize {
        self.columns.len()
    }

    #[cfg(test)]
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
/// `LatticeInvolution::coweight_matrix()` already stores the dual action on
/// cocharacters, so this function intentionally does not transpose it again.
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

/// The result of [`adapted_basis`]: a unimodular basis `B` of the ambient
/// lattice, its inverse, and positive diagonal factors such that the input's
/// column span is exactly `span{ diagonal[t] * B.column(t) }` — so the FIRST
/// `diagonal.len()` columns of `B` span the SATURATION of that image.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AdaptedBasis {
    pub(crate) basis: IntegerMatrix,
    pub(crate) inverse: IntegerMatrix,
    pub(crate) diagonal: Vec<Integer>,
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
pub(crate) fn adapted_basis(
    rows: &[Vec<i32>],
    budget: &IntegerLatticeBudget,
) -> Result<AdaptedBasis, StructureError> {
    let mut matrix = IntegerMatrix::from_i32_rows(rows, budget)?;
    let height = matrix.rows;
    let width = matrix.columns;
    let mut basis = IntegerMatrix::identity(height, budget)?;
    let mut inverse = IntegerMatrix::identity(height, budget)?;
    let mut diagonal = Vec::new();
    diagonal.try_reserve_exact(width.min(height)).map_err(|_| {
        StructureError::AllocationFailed {
            requested: width.min(height),
        }
    })?;
    let mut kept = vec![false; height];
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
                let replacement = matrix.entry(other, column) - &quotient * &current;
                matrix.set(row, column, replacement);
                matrix.set(other, column, current);
            }
            for column in 0..height {
                let current = inverse.entry(row, column).clone();
                let replacement = inverse.entry(other, column) - &quotient * &current;
                inverse.set(row, column, replacement);
                inverse.set(other, column, current);
            }
            for target in 0..height {
                let current = basis.entry(target, other).clone();
                let replacement = basis.entry(target, row) + &quotient * &current;
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
                let update = inverse.entry(other, column) - &quotient * inverse.entry(row, column);
                inverse.set(other, column, update);
            }
            for target in 0..height {
                let update = basis.entry(target, row) + &quotient * basis.entry(target, other);
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
                for target in 0..matrix.rows {
                    let update =
                        matrix.entry(target, column) - &quotient * matrix.entry(target, current);
                    matrix.set(target, column, update);
                }
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
        let a = self.matrix.entry(pivot, pivot).clone();
        let b = self.matrix.entry(row, pivot).clone();
        let max_coefficient_bits = self.budget.max_coefficient_bits;
        if (&b).divisible_by(&a) {
            let factor = -b.div_exact(&a);
            self.check_live_entries(self.matrix.columns)?;
            self.matrix
                .add_row_multiple(row, pivot, &factor, max_coefficient_bits)?;
            return self.advance();
        }
        let transform = BezoutTransform::for_entries(&a, &b);
        self.check_live_entries(double_entries(self.matrix.columns)?)?;
        self.matrix
            .bezout_rows(pivot, row, &transform, max_coefficient_bits)?;
        self.advance()
    }

    fn eliminate_right(&mut self, pivot: usize, column: usize) -> Result<(), StructureError> {
        let a = self.matrix.entry(pivot, pivot).clone();
        let b = self.matrix.entry(pivot, column).clone();
        let max_coefficient_bits = self.budget.max_coefficient_bits;
        if (&b).divisible_by(&a) {
            let factor = -b.div_exact(&a);
            self.check_live_entries(self.matrix.rows.max(self.right.rows))?;
            self.matrix
                .add_column_multiple(column, pivot, &factor, max_coefficient_bits)?;
            self.right
                .add_column_multiple(column, pivot, &factor, max_coefficient_bits)?;
            return self.advance();
        }
        let transform = BezoutTransform::for_entries(&a, &b);
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
        self.check_bounds()
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

    fn matrix(rows: &[&[i32]]) -> IntegerMatrix {
        IntegerMatrix::from_i32_rows(
            &rows.iter().map(|row| row.to_vec()).collect::<Vec<_>>(),
            &budget(),
        )
        .unwrap()
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
}
