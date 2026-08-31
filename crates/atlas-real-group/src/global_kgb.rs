//! The inner-class-wide KGB graph: a port of upstream `kgb::global_KGB`
//! (kgb.h:213-266, kgb.cpp:190-233 and 331-479) with the `kgb_io::print_X`
//! layout (io/kgb_io.cpp:57-159).
//!
//! One `GlobalKgb` enumerates the KGB elements of EVERY strong real form of
//! one inner class, grouped into tau packets over the twisted involutions.
//! Elements are deduplicated by the upstream `InvolutionTable::x_pack`
//! fingerprint (involutions.cpp:279-295): the torus part's `log_2pi`
//! projected onto the saturation of `theta + I`, reduced modulo the
//! denominator. The fingerprint — not the torus value — defines element
//! identity; the stored torus representative is the first-arrival one, whose
//! RAW arithmetic history is observable in the printout (negative numerators
//! such as `[0,-1]/2` survive because `simple_reflect` never re-reduces).
//!
//! The second upstream constructor (seeding from an arbitrary
//! `GlobalTitsElement`) and the Bruhat/Hasse layer are not ported.

use std::collections::HashMap;
use std::ops::Range;

use malachite::base::num::basic::traits::{One, Zero};
use malachite::{Integer, Rational};

use crate::grading::try_capacity;
use crate::integer_lattice::{
    adapted_basis, negative_coweight_eigenspace, reduce_basis_mod_two, IntegerLatticeBudget,
};
use crate::mod_two::{ModTwoSubquotient, ModTwoSubspace, ModTwoVector};
use crate::{
    BasedRootDatum, CartanClassification, CartanId, InnerClass, InvolutionId, InvolutionTable,
    KgbStatus, LatticeInvolution, RationalWeight, StructureError,
};

/// Upstream `ioutils::digits(n, 10)` (io/ioutils.cpp): `digits(0)` is 1.
fn digits(mut value: usize) -> usize {
    let mut count = 1;
    while value >= 10 {
        count += 1;
        value /= 10;
    }
    count
}

fn gcd_u64(mut left: u64, mut right: u64) -> u64 {
    while right != 0 {
        (left, right) = (right, left % right);
    }
    left
}

/// `y_values::TorusElement` (y_values.h:40-105, y_values.cpp:29-125): a
/// finite-order torus element stored as a rational vector `repr` with
/// `repr = numerator/denominator` representing `exp(i*pi*repr)`, coordinates
/// taken modulo `2Z^rank`. Construction entry points reduce numerators into
/// `[0, 2*denominator)`, but `simple_reflect` deliberately does NOT
/// re-reduce, so the stored numerator is an arithmetic history, not a
/// canonical form; every reader below replicates the upstream reduction
/// discipline exactly.
#[derive(Clone, Debug, Eq, PartialEq)]
struct GlobalTorusElement {
    numerator: Vec<i64>,
    denominator: i64,
}

impl GlobalTorusElement {
    /// The `(numer, denom) -> repr` reduction of the upstream constructor
    /// (y_values.cpp:29-37): numerators reduced with
    /// `arithmetic::remainder` (always non-negative) modulo `2*denom`.
    fn reduce_raw(mut numerator: Vec<i64>, denominator: i64) -> Self {
        let modulus = 2 * denominator;
        for entry in &mut numerator {
            *entry = entry.rem_euclid(modulus);
        }
        Self {
            numerator,
            denominator,
        }
    }

    /// `y_values::exp_pi` (y_values.h:108).
    fn exp_pi(numerator: Vec<i64>, denominator: i64) -> Self {
        Self::reduce_raw(numerator, denominator)
    }

    /// `y_values::exp_2pi` (y_values.h:109). The upstream `repr *= 2`
    /// (ratvec.cpp:104-117) cancels a factor 2 from an even denominator
    /// instead of doubling the numerator; no other normalization happens.
    fn exp_2pi(mut numerator: Vec<i64>, mut denominator: i64) -> Result<Self, StructureError> {
        if denominator % 2 == 0 {
            denominator /= 2;
        } else {
            for entry in &mut numerator {
                *entry = entry
                    .checked_mul(2)
                    .ok_or(StructureError::ArithmeticOverflow)?;
            }
        }
        Ok(Self::reduce_raw(numerator, denominator))
    }

    /// `TorusElement::operator+` (y_values.cpp:74-84): lcm combination,
    /// gcd normalization (the upstream `RationalVector::operator+=`
    /// normalizes, ratvec.cpp:67-79), then ONE conditional subtraction per
    /// coordinate — upstream only corrects a sum lying in `[2, 4)`, so
    /// non-canonical addends (post-reflection numerators) pass through
    /// unchanged.
    fn add(&self, right: &Self) -> Result<Self, StructureError> {
        let rank = self.numerator.len();
        let gcd = gcd_u64(self.denominator as u64, right.denominator as u64);
        let factor_self = i64::try_from(right.denominator as u64 / gcd)
            .map_err(|_| StructureError::ArithmeticOverflow)?;
        let factor_right = i64::try_from(self.denominator as u64 / gcd)
            .map_err(|_| StructureError::ArithmeticOverflow)?;
        let mut denominator = self
            .denominator
            .checked_mul(factor_self)
            .ok_or(StructureError::ArithmeticOverflow)?;
        let mut numerator = try_capacity(rank)?;
        for index in 0..rank {
            let left = self.numerator[index]
                .checked_mul(factor_self)
                .ok_or(StructureError::ArithmeticOverflow)?;
            let other = right.numerator[index]
                .checked_mul(factor_right)
                .ok_or(StructureError::ArithmeticOverflow)?;
            numerator.push(
                left.checked_add(other)
                    .ok_or(StructureError::ArithmeticOverflow)?,
            );
        }
        // RationalVector::normalize: divide by the gcd of the denominator
        // and every numerator entry.
        let mut divisor = denominator.unsigned_abs();
        for &entry in &numerator {
            divisor = gcd_u64(entry.unsigned_abs(), divisor);
            if divisor == 1 {
                break;
            }
        }
        if divisor > 1 {
            let divisor = i64::try_from(divisor).map_err(|_| StructureError::ArithmeticOverflow)?;
            denominator /= divisor;
            for entry in &mut numerator {
                *entry /= divisor;
            }
        }
        let modulus = 2 * denominator;
        for entry in &mut numerator {
            if *entry >= modulus {
                *entry -= modulus;
            }
        }
        Ok(Self {
            numerator,
            denominator,
        })
    }

    /// `TorusElement::operator+=(TorusPart)` (y_values.cpp:95-106): toggle
    /// each set coordinate by half the period.
    fn add_torus_part(&mut self, part: &ModTwoVector) -> Result<(), StructureError> {
        for index in 0..self.numerator.len() {
            if part.bit(index) == Some(true) {
                if self.numerator[index] < self.denominator {
                    self.numerator[index] += self.denominator;
                } else {
                    self.numerator[index] -= self.denominator;
                }
            }
        }
        Ok(())
    }

    /// `TorusElement::evaluate_at(Coweight)` (y_values.cpp:65-71): the
    /// pairing modulo 2 as `(remainder(dot, 2d), d)`, NOT normalized.
    fn evaluate_at(&self, weight: &[i32]) -> Result<(i64, i64), StructureError> {
        let mut dot = 0_i64;
        for (index, &coefficient) in weight.iter().enumerate() {
            let term = i64::from(coefficient)
                .checked_mul(self.numerator[index])
                .ok_or(StructureError::ArithmeticOverflow)?;
            dot = dot
                .checked_add(term)
                .ok_or(StructureError::ArithmeticOverflow)?;
        }
        Ok((dot.rem_euclid(2 * self.denominator), self.denominator))
    }

    /// `TorusElement::negative_at` (y_values.h:74-78): the pairing must be
    /// integral; the root is compact iff the integral value is odd.
    fn negative_at(&self, weight: &[i32]) -> Result<bool, StructureError> {
        let mut dot = 0_i64;
        for (index, &coefficient) in weight.iter().enumerate() {
            let term = i64::from(coefficient)
                .checked_mul(self.numerator[index])
                .ok_or(StructureError::ArithmeticOverflow)?;
            dot = dot
                .checked_add(term)
                .ok_or(StructureError::ArithmeticOverflow)?;
        }
        if dot % self.denominator != 0 {
            return Err(StructureError::KgbInvariantViolation {
                invariant: "integral root evaluation",
            });
        }
        Ok((dot / self.denominator) % 2 != 0)
    }

    /// `TorusElement::simple_reflect` (y_values.cpp:120-123) with the
    /// DUAL-side pre-root datum: `numer -= <numer, alpha_s> * coroot_s`.
    /// No reduction happens afterwards — this is what lets negative
    /// numerators reach the printout.
    fn simple_reflect(&mut self, root: &[i32], coroot: &[i32]) -> Result<(), StructureError> {
        let mut pairing = 0_i64;
        for (index, &coefficient) in root.iter().enumerate() {
            let term = i64::from(coefficient)
                .checked_mul(self.numerator[index])
                .ok_or(StructureError::ArithmeticOverflow)?;
            pairing = pairing
                .checked_add(term)
                .ok_or(StructureError::ArithmeticOverflow)?;
        }
        for (entry, &coroot_entry) in self.numerator.iter_mut().zip(coroot.iter()) {
            let shift = pairing
                .checked_mul(i64::from(coroot_entry))
                .ok_or(StructureError::ArithmeticOverflow)?;
            *entry = entry
                .checked_sub(shift)
                .ok_or(StructureError::ArithmeticOverflow)?;
        }
        Ok(())
    }

    /// `GlobalTitsGroup::imaginary_cross_act` (tits.cpp:167-174): reflect
    /// the torus part by the shift that fixes the rho_im evaluation, iff
    /// the root is noncompact on this element.
    fn imaginary_cross_act(&mut self, root: &[i32], coroot: &[i32]) -> Result<(), StructureError> {
        let (remainder, denominator) = self.evaluate_at(root)?;
        let r_numerator = remainder - denominator;
        if r_numerator != 0 {
            let mut shift = try_capacity(coroot.len())?;
            for &entry in coroot {
                shift.push(
                    i64::from(entry)
                        .checked_mul(-r_numerator)
                        .ok_or(StructureError::ArithmeticOverflow)?,
                );
            }
            let addend = Self::exp_2pi(shift, 2 * denominator)?;
            *self = self.add(&addend)?;
        }
        Ok(())
    }

    /// `TorusElement::log_2pi` (y_values.cpp:46-51): numerator over TWICE
    /// the denominator, gcd-normalized only — no reduction modulo 1.
    fn log_2pi(&self) -> Result<RationalWeight, StructureError> {
        RationalWeight::new(
            self.numerator.clone(),
            self.denominator
                .checked_mul(2)
                .ok_or(StructureError::ArithmeticOverflow)?,
        )
    }
}

/// The `x_pack` fingerprint (involutions.cpp:279-295): project the torus
/// part's `log_2pi` numerator onto the saturation of the image of
/// `theta + I`, reduce modulo the denominator. Upstream multiplies by the
/// full `row_saturate(theta^T + I)` matrix and keeps `rank` components; the
/// nonzero rows are exactly an adapted basis of the same saturated image,
/// and two bases of one saturation differ by a unimodular — hence
/// mod-d invertible — left factor, so keeping only the `diagonal.len()`
/// adapted-basis components is a lossless key.
fn fingerprint(
    theta: &LatticeInvolution,
    torus: &GlobalTorusElement,
    budget: &IntegerLatticeBudget,
) -> Result<RationalWeight, StructureError> {
    let log = torus.log_2pi()?;
    let rank = theta.lattice_rank();
    let mut theta_plus_one = try_capacity(rank)?;
    for (row_index, row) in theta.weight_matrix().iter().enumerate() {
        let mut shifted = row.clone();
        shifted[row_index] = shifted[row_index]
            .checked_add(1)
            .ok_or(StructureError::ArithmeticOverflow)?;
        theta_plus_one.push(shifted);
    }
    let adapted = adapted_basis(&theta_plus_one, budget)?;
    let components = adapted.diagonal.len();
    let mut projected = try_capacity(components)?;
    for column in 0..components {
        // Column `column` of the adapted basis, dotted with the numerator.
        // The integer-lattice budget caps entries at 64 bits, so the i128
        // accumulation below cannot overflow.
        let mut accumulator = 0_i128;
        for row in 0..rank {
            let entry = i64::try_from(adapted.basis.entry(row, column))
                .map_err(|_| StructureError::ArithmeticOverflow)?;
            accumulator += i128::from(entry) * i128::from(log.numerator()[row]);
        }
        let modulus = i128::from(log.denominator());
        projected.push(
            i64::try_from(accumulator.rem_euclid(modulus))
                .map_err(|_| StructureError::ArithmeticOverflow)?,
        );
    }
    RationalWeight::new(projected, log.denominator())
}

/// Cached adapted-basis projection for one twisted involution.
#[derive(Clone, Debug)]
struct FingerprintProjector {
    basis: crate::integer_lattice::IntegerMatrix,
    components: usize,
}

impl FingerprintProjector {
    fn new(
        theta: &LatticeInvolution,
        budget: &IntegerLatticeBudget,
    ) -> Result<Self, StructureError> {
        let rank = theta.lattice_rank();
        let mut theta_plus_one = try_capacity(rank)?;
        for (row_index, row) in theta.weight_matrix().iter().enumerate() {
            let mut shifted = row.clone();
            shifted[row_index] = shifted[row_index]
                .checked_add(1)
                .ok_or(StructureError::ArithmeticOverflow)?;
            theta_plus_one.push(shifted);
        }
        let adapted = adapted_basis(&theta_plus_one, budget)?;
        Ok(Self {
            components: adapted.diagonal.len(),
            basis: adapted.basis,
        })
    }

    fn apply(&self, torus: &GlobalTorusElement) -> Result<RationalWeight, StructureError> {
        let log = torus.log_2pi()?;
        let mut projected = try_capacity(self.components)?;
        for column in 0..self.components {
            // Column `column` of the adapted basis, dotted with the numerator.
            let mut accumulator = 0_i128;
            for row in 0..self.basis.rows {
                let entry = i64::try_from(self.basis.entry(row, column))
                    .map_err(|_| StructureError::ArithmeticOverflow)?;
                accumulator += i128::from(entry) * i128::from(log.numerator()[row]);
            }
            let modulus = i128::from(log.denominator());
            projected.push(
                i64::try_from(accumulator.rem_euclid(modulus))
                    .map_err(|_| StructureError::ArithmeticOverflow)?,
            );
        }
        RationalWeight::new(projected, log.denominator())
    }
}

/// Reusable per-involution fingerprint projectors used while generating KGB.
#[derive(Clone, Debug, Default)]
struct FingerprintCache {
    projectors: HashMap<InvolutionId, FingerprintProjector>,
}

impl FingerprintCache {
    fn new() -> Self {
        Self::default()
    }

    fn fingerprint(
        &mut self,
        id: InvolutionId,
        theta: &LatticeInvolution,
        torus: &GlobalTorusElement,
        budget: &IntegerLatticeBudget,
    ) -> Result<RationalWeight, StructureError> {
        if let Some(projector) = self.projectors.get(&id) {
            return projector.apply(torus);
        }
        let projector = FingerprintProjector::new(theta, budget)?;
        let fingerprint = projector.apply(torus)?;
        self.projectors.insert(id, projector);
        Ok(fingerprint)
    }
}

/// The fundamental fiber group `dualPi0(-delta^t)` (tori.cpp:163-175 via
/// cartanclass.cpp:209-212), recomputed with the same formula as
/// `CartanFiber::build_owned` (cartan_fiber.rs:69-87) so the basis order
/// matches the per-Cartan fiber layer.
fn fundamental_fiber(
    delta: &LatticeInvolution,
    budget: &IntegerLatticeBudget,
) -> Result<ModTwoSubquotient, StructureError> {
    let negative_eigenspace = negative_coweight_eigenspace(delta.coweight_matrix(), budget)?;
    let denominator = reduce_basis_mod_two(&negative_eigenspace)?;
    let rank = delta.lattice_rank();
    let mut equations = ModTwoSubspace::new(rank)?;
    for (row_index, row) in delta.coweight_matrix().iter().enumerate() {
        if row.len() != rank {
            return Err(StructureError::InvalidInvolution);
        }
        let mut ones = try_capacity(
            rank.checked_add(1)
                .ok_or(StructureError::ArithmeticOverflow)?,
        )?;
        for (column_index, &entry) in row.iter().enumerate() {
            if entry % 2 != 0 {
                ones.push(column_index);
            }
        }
        // `from_ones` xor-toggles duplicates, so pushing the diagonal index
        // implements the `+ I` term even when theta has an odd entry there.
        ones.push(row_index);
        equations.insert(ModTwoVector::from_ones(rank, ones)?)?;
    }
    let numerator = equations.right_kernel()?;
    ModTwoSubquotient::new(numerator, denominator)
}

/// The square-class generator simple roots (tits.cpp:318-356
/// `compute_square_classes`): evaluate the mod-2 `+1` eigenspace of
/// `delta_Y` at the delta-fixed simple roots; the non-pivot positions of
/// the mapped subspace, in ascending order, elect the generators.
fn square_class_generators(
    inner_class: &InnerClass,
    budget: &IntegerLatticeBudget,
) -> Result<Vec<usize>, StructureError> {
    let datum = inner_class.datum();
    let twist = inner_class.generator_twist()?;
    let mut fixed = try_capacity(twist.len())?;
    for (generator, &image) in twist.iter().enumerate() {
        if image == generator {
            fixed.push(generator);
        }
    }
    let delta = inner_class.distinguished_involution().involution();
    let mut negated = try_capacity(delta.coweight_matrix().len())?;
    for row in delta.coweight_matrix() {
        let mut negated_row = try_capacity(row.len())?;
        for &entry in row {
            negated_row.push(
                entry
                    .checked_neg()
                    .ok_or(StructureError::ArithmeticOverflow)?,
            );
        }
        negated.push(negated_row);
    }
    let positive_eigenspace = negative_coweight_eigenspace(&negated, budget)?;
    let v_plus = reduce_basis_mod_two(&positive_eigenspace)?;
    let mut mapped = ModTwoSubspace::new(fixed.len())?;
    for vector in v_plus.basis_vectors() {
        let mut ones = try_capacity(fixed.len())?;
        for (position, &generator) in fixed.iter().enumerate() {
            let root = datum.simple_roots()[generator].as_slice();
            let mut evaluation = 0_i64;
            for (index, &coefficient) in root.iter().enumerate() {
                if vector.bit(index) == Some(true) {
                    evaluation += i64::from(coefficient);
                }
            }
            if evaluation % 2 != 0 {
                ones.push(position);
            }
        }
        mapped.insert(ModTwoVector::from_ones(fixed.len(), ones)?)?;
    }
    let mut is_pivot = vec![false; fixed.len()];
    for (pivot, _) in mapped.pivot_rows() {
        is_pivot[pivot] = true;
    }
    let mut generators = try_capacity(fixed.len())?;
    for (position, &generator) in fixed.iter().enumerate() {
        if !is_pivot[position] {
            generators.push(generator);
        }
    }
    Ok(generators)
}

/// `RootDatum::fundamental_coweight` (rootdata.cpp:1015-1018): column `i`
/// of `det(C) * C^{-1}` combines the simple coroots, over `det(C)`. The
/// Cartan inverse is computed by exact rational Gaussian elimination.
fn fundamental_coweights(datum: &BasedRootDatum) -> Result<Vec<RationalWeight>, StructureError> {
    let cartan = datum.cartan_matrix();
    let rank = cartan.len();
    let lattice_rank = datum.lattice_rank();
    let mut result = try_capacity(rank)?;
    if rank == 0 {
        return Ok(result);
    }
    let mut matrix: Vec<Vec<Rational>> = Vec::with_capacity(rank);
    let mut inverse: Vec<Vec<Rational>> = Vec::with_capacity(rank);
    for (row_index, row) in cartan.iter().enumerate() {
        matrix.push(row.iter().map(|&entry| Rational::from(entry)).collect());
        inverse.push(
            (0..rank)
                .map(|column| {
                    if column == row_index {
                        Rational::ONE
                    } else {
                        Rational::ZERO
                    }
                })
                .collect(),
        );
    }
    for column in 0..rank {
        let pivot = (column..rank)
            .find(|&row| matrix[row][column] != Rational::ZERO)
            .ok_or(StructureError::InvalidCartanMatrix)?;
        matrix.swap(column, pivot);
        inverse.swap(column, pivot);
        let pivot_value = matrix[column][column].clone();
        for entry in 0..rank {
            matrix[column][entry] /= &pivot_value;
            inverse[column][entry] /= &pivot_value;
        }
        for row in 0..rank {
            if row != column && matrix[row][column] != Rational::ZERO {
                let factor = matrix[row][column].clone();
                for entry in 0..rank {
                    let subtract = &factor * &matrix[column][entry];
                    matrix[row][entry] -= subtract;
                    let subtract = &factor * &inverse[column][entry];
                    inverse[row][entry] -= subtract;
                }
            }
        }
    }
    let coroots = datum.simple_coroots();
    for generator in 0..rank {
        // coordinate j of omega-vee_i is sum_k inv[k][i] * coroot_k[j].
        let mut coordinates: Vec<Rational> = Vec::with_capacity(lattice_rank);
        for coordinate in 0..lattice_rank {
            let mut sum = Rational::ZERO;
            for (k, coroot) in coroots.iter().enumerate() {
                let coefficient = i64::from(coroot.as_slice()[coordinate]);
                if coefficient != 0 {
                    sum += &inverse[k][generator] * Rational::from(coefficient);
                }
            }
            coordinates.push(sum);
        }
        let mut denominator = 1_i64;
        for value in &coordinates {
            let next = i64::try_from(value.denominator_ref())
                .map_err(|_| StructureError::ArithmeticOverflow)?;
            denominator = denominator
                .checked_mul(next / gcd_u64(denominator as u64, next as u64) as i64)
                .ok_or(StructureError::ArithmeticOverflow)?;
        }
        let mut numerator = try_capacity(lattice_rank)?;
        for value in &coordinates {
            let scaled = value * Rational::from(denominator);
            let integer =
                Integer::try_from(&scaled).map_err(|_| StructureError::KgbInvariantViolation {
                    invariant: "fundamental coweight fraction",
                })?;
            numerator
                .push(i64::try_from(&integer).map_err(|_| StructureError::ArithmeticOverflow)?);
        }
        result.push(RationalWeight::new(numerator, denominator)?);
    }
    Ok(result)
}

/// One row of the `print_X` layout (io/kgb_io.cpp:57-125 with
/// `traditional == false`, `G == nullptr`, `which == nullptr`).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GlobalKgbPrintRow {
    pub length: usize,
    pub statuses: Vec<KgbStatus>,
    pub cross: Vec<usize>,
    pub cayley: Vec<Option<usize>>,
    pub torus_label: RationalWeight,
    pub cartan: usize,
    pub involution_word: String,
}

/// The structured `kgb_io::print_X` output: the torus-element-offset header
/// line and one row per element. [`GlobalKgbPrint::render`] reproduces the
/// exact byte stream, including every `setw` pad.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GlobalKgbPrint {
    pub header: String,
    pub lattice_rank: usize,
    pub rows: Vec<GlobalKgbPrintRow>,
}

impl GlobalKgbPrint {
    /// The exact bytes upstream writes: header line, then one terminated
    /// line per element.
    pub fn render(&self) -> String {
        let size = self.rows.len();
        let width = digits(size.saturating_sub(1));
        let cwidth = self.rows.last().map_or(1, |row| digits(row.cartan));
        let lwidth = self.rows.last().map_or(1, |row| digits(row.length));
        let label_width = 3 * self.lattice_rank + 3;
        let mut output = String::new();
        output.push_str(&self.header);
        output.push('\n');
        for (index, row) in self.rows.iter().enumerate() {
            output.push_str(&format!("{index:>width$}:  "));
            output.push_str(&format!("{:>lwidth$}", row.length));
            output.push_str("  ");
            output.push('[');
            for (position, status) in row.statuses.iter().enumerate() {
                if position > 0 {
                    output.push(',');
                }
                output.push(match status {
                    KgbStatus::Complex => 'C',
                    KgbStatus::ImaginaryCompact => 'c',
                    KgbStatus::ImaginaryNoncompact => 'n',
                    KgbStatus::Real => 'r',
                });
            }
            output.push(']');
            output.push(' ');
            for &target in &row.cross {
                output.push_str(&format!("{target:>width$}", width = width + 2));
            }
            output.push_str("  ");
            for &target in &row.cayley {
                match target {
                    Some(element) => {
                        output.push_str(&format!("{element:>width$}", width = width + 2));
                    }
                    None => output.push_str(&format!("{:>width$}", '*', width = width + 2)),
                }
            }
            output.push_str("  ");
            output.push_str(&format!(
                "{:>label_width$}",
                format_rational_weight(&row.torus_label)
            ));
            output.push(' ');
            output.push_str(&format!("{:>cwidth$} ", row.cartan));
            output.push_str(&row.involution_word);
            output.push('\n');
        }
        output
    }
}

/// `ratvec::operator<<` (io/basic_io.cpp:138-144): the numerator as a
/// bracket-enclosed comma-separated list, then `/` and the denominator.
fn format_rational_weight(weight: &RationalWeight) -> String {
    let mut output = String::from("[");
    for (index, entry) in weight.numerator().iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&entry.to_string());
    }
    output.push(']');
    output.push('/');
    output.push_str(&weight.denominator().to_string());
    output
}

/// `prettyprint::printInvolution` (io/prettyprint.cpp): each canonical
/// expression entry `n >= 0` prints as the generator digit and `^`, an
/// entry `!n` as the digit and `x`, then a trailing `e`. Upstream renders
/// the generator as the CHARACTER `'1' + n`; the same byte is produced
/// here, quirk included.
fn format_involution_word(expression: &[i32]) -> String {
    let mut output = String::new();
    for &entry in expression {
        if entry >= 0 {
            output.push((b'1' + entry as u8) as char);
            output.push('^');
        } else {
            output.push((b'1' + (!entry) as u8) as char);
            output.push('x');
        }
    }
    output.push('e');
    output
}

/// The inner-class-wide KGB graph: upstream `kgb::global_KGB`.
///
/// Storage is flat with index `x * semisimple_rank + generator`, matching
/// upstream's `data[s][x]` layout. `first_of_tau` delimits the tau packets:
/// elements `first_of_tau[i]..first_of_tau[i+1]` lie over the i-th
/// involution in generation order (`involutions[i]`).
#[derive(Clone, Debug)]
pub struct GlobalKgb {
    semisimple_rank: usize,
    lattice_rank: usize,
    elements: Vec<GlobalTorusElement>,
    element_packet: Vec<usize>,
    involutions: Vec<InvolutionId>,
    first_of_tau: Vec<usize>,
    statuses: Vec<Option<KgbStatus>>,
    cross: Vec<usize>,
    cayley: Vec<Option<usize>>,
    inverse_cayley: Vec<Option<(usize, Option<usize>)>>,
    involution_lengths: Vec<usize>,
    involution_cartans: Vec<usize>,
    involution_words: Vec<String>,
    header_offset: RationalWeight,
}

impl GlobalKgb {
    /// `global_KGB::global_KGB(InnerClass&)` (kgb.cpp:190-233): add every
    /// Cartan orbit to `table`, generate the involutions in length-interval
    /// BFS order, seed the fundamental fiber for all square classes, then
    /// run the cross/Cayley closure.
    ///
    /// `table` must be bound to `inner_class`; `classification` must be its
    /// Cartan classification. Upstream reserves the exact predicted size
    /// `inner_class::global_KGB_size`; this port grows dynamically instead
    /// (the prediction formula is an allocation hint, not observable).
    pub fn build(
        inner_class: &InnerClass,
        classification: &CartanClassification,
        table: &mut InvolutionTable,
        budget: &IntegerLatticeBudget,
    ) -> Result<Self, StructureError> {
        if table.inner_class() != inner_class {
            return Err(StructureError::DatumMismatch);
        }
        let datum = inner_class.datum();
        let system = inner_class.root_system();
        let semisimple_rank = datum.semisimple_rank();
        let lattice_rank = datum.lattice_rank();

        for cartan in 0..classification.cartan_classes().len() {
            table.add_cartan(classification, CartanId(cartan))?;
        }

        // generate_involutions (kgb.cpp:331-366): the identity twisted
        // involution seeds index 0; children are `s * w` on twisted
        // commutation and `s * w * delta(s)` otherwise.
        let identity_id = table
            .identity_id()
            .ok_or(StructureError::KgbInvariantViolation {
                invariant: "fundamental involution",
            })?;
        let total_involutions = table.involution_count();
        let mut involutions = try_capacity(total_involutions)?;
        involutions.push(identity_id);
        let mut involution_location = vec![usize::MAX; total_involutions];
        involution_location[identity_id.0] = 0;
        let mut end_length = 0_usize;
        while end_length < involutions.len() {
            let start_length = end_length;
            end_length = involutions.len();
            for generator in 0..semisimple_rank {
                for index in start_length..end_length {
                    let parent = involutions[index];
                    // hasTwistedCommutation (weyl.cpp:1296-1312), the same
                    // compact operation as the upstream `WeylElt` path.
                    let commutes = table.weyl_has_twisted_commutation(parent, generator)?;
                    let child = if commutes {
                        table.cayley(generator, parent)?.ok_or(
                            StructureError::KgbInvariantViolation {
                                invariant: "involution generation cross link",
                            },
                        )?
                    } else {
                        table.cross(generator, parent)?
                    };
                    if involution_location[child.0] == usize::MAX {
                        involution_location[child.0] = involutions.len();
                        involutions.push(child);
                    }
                }
            }
        }
        if involutions.len() != total_involutions {
            return Err(StructureError::KgbInvariantViolation {
                invariant: "involution generation count",
            });
        }

        // Per-packet derived data.
        let mut involution_lengths = try_capacity(total_involutions)?;
        let mut involution_cartans = try_capacity(total_involutions)?;
        let mut involution_words = try_capacity(total_involutions)?;
        for &id in &involutions {
            let record = table.record(id).ok_or(StructureError::IndexOutOfRange {
                index: id.0,
                upper_bound: total_involutions,
            })?;
            involution_lengths.push(record.involution_length());
            involution_cartans.push(
                table
                    .cartan_of(id)
                    .ok_or(StructureError::KgbInvariantViolation {
                        invariant: "involution Cartan class",
                    })?
                    .0,
            );
            involution_words.push(format_involution_word(
                &table.weyl_canonical_involution_expr(id)?,
            ));
        }

        // The fundamental fiber (kgb.cpp:209-231): square classes shift the
        // base torus element by fundamental coweights, the fiber group
        // contributes its `TorusPart` lifts.
        let delta = inner_class.distinguished_involution().involution().clone();
        let fiber = fundamental_fiber(&delta, budget)?;
        let fiber_basis = fiber.basis_representatives();
        let fiber_rank = fiber_basis.len();
        let fiber_size = 1_usize
            .checked_shl(u32::try_from(fiber_rank).map_err(|_| StructureError::ArithmeticOverflow)?)
            .ok_or(StructureError::ArithmeticOverflow)?;
        let generators = square_class_generators(inner_class, budget)?;
        let coweights = fundamental_coweights(datum)?;

        let mut store = ElementStore::new(semisimple_rank);
        let mut first_of_tau = try_capacity(total_involutions + 1)?;
        first_of_tau.push(0);

        let subset_count = 1_usize
            .checked_shl(
                u32::try_from(generators.len()).map_err(|_| StructureError::ArithmeticOverflow)?,
            )
            .ok_or(StructureError::ArithmeticOverflow)?;
        for subset in 0..subset_count {
            let mut rcw = RationalWeight::zero(lattice_rank)?;
            for (bit, &generator) in generators.iter().enumerate() {
                if subset & (1 << bit) != 0 {
                    rcw = rcw.add(&coweights[generator])?;
                }
            }
            for lift in 0..fiber_size {
                let mut torus =
                    GlobalTorusElement::exp_pi(rcw.numerator().to_vec(), rcw.denominator());
                // lift_from_fundamental_fiber (innerclass.h:431-434): the
                // basis combination selected by the lift's bits.
                let mut part = ModTwoVector::zero(lattice_rank)?;
                for (bit, representative) in fiber_basis.iter().enumerate() {
                    if lift & (1 << bit) != 0 {
                        part.xor_assign(representative)?;
                    }
                }
                torus.add_torus_part(&part)?;
                store.push(torus, 0)?;
            }
        }
        first_of_tau.push(store.len());

        // The dedup key: (involution, fingerprint), mirroring
        // KGB_elt_entry's `tw` and `fingerprint` fields (kgb.h:217-237,
        // kgb.cpp:73-84).
        let mut dedup: HashMap<(usize, RationalWeight), usize> = HashMap::new();
        let mut fingerprint_cache = FingerprintCache::new();
        for (index, torus) in store.elements.iter().enumerate() {
            let theta = table
                .record(identity_id)
                .ok_or(StructureError::IndexOutOfRange {
                    index: identity_id.0,
                    upper_bound: total_involutions,
                })?
                .theta();
            let key = (
                identity_id.0,
                fingerprint_cache.fingerprint(identity_id, theta, torus, budget)?,
            );
            if dedup.insert(key, index).is_some() {
                return Err(StructureError::KgbInvariantViolation {
                    invariant: "fundamental fiber distinctness",
                });
            }
        }

        // generate (kgb.cpp:370-479).
        let mut end_length = 0_usize;
        while end_length < first_of_tau.len() - 1 {
            let start_length = end_length;
            end_length = first_of_tau.len() - 1;
            for generator in 0..semisimple_rank {
                for index in start_length..end_length {
                    let tw_id = involutions[index];
                    let record = table.record(tw_id).ok_or(StructureError::IndexOutOfRange {
                        index: tw_id.0,
                        upper_bound: total_involutions,
                    })?;
                    let source_weyl_length = record.weyl_length();
                    let has_descent = table.weyl_has_left_descent(tw_id, generator)?;

                    let cross_target = table.cross(generator, tw_id)?;
                    let new_number = involution_location[cross_target.0];
                    let mut is_new = new_number >= first_of_tau.len() - 1;
                    if is_new && new_number != first_of_tau.len() - 1 {
                        return Err(StructureError::KgbInvariantViolation {
                            invariant: "involution generation order",
                        });
                    }
                    let imaginary = new_number == index && !has_descent;

                    let packet = first_of_tau[index]..first_of_tau[index + 1];
                    for x in packet.clone() {
                        let mut child = store.elements[x].clone();
                        let target_weyl_length = table
                            .record(cross_target)
                            .ok_or(StructureError::IndexOutOfRange {
                                index: cross_target.0,
                                upper_bound: total_involutions,
                            })?
                            .weyl_length();
                        let length_change = target_weyl_length as i64 - source_weyl_length as i64;
                        if length_change % 2 != 0 {
                            return Err(StructureError::KgbInvariantViolation {
                                invariant: "cross length parity",
                            });
                        }
                        let d = length_change / 2;
                        if d != 0 {
                            child.simple_reflect(
                                datum.simple_roots()[generator].as_slice(),
                                datum.simple_coroots()[generator].as_slice(),
                            )?;
                        } else if imaginary {
                            child.imaginary_cross_act(
                                datum.simple_roots()[generator].as_slice(),
                                datum.simple_coroots()[generator].as_slice(),
                            )?;
                        }
                        let theta = table
                            .record(cross_target)
                            .ok_or(StructureError::IndexOutOfRange {
                                index: cross_target.0,
                                upper_bound: total_involutions,
                            })?
                            .theta();
                        let key = (
                            cross_target.0,
                            fingerprint_cache.fingerprint(cross_target, theta, &child, budget)?,
                        );
                        let k = match dedup.get(&key) {
                            Some(&existing) => existing,
                            None => {
                                if !is_new {
                                    return Err(StructureError::KgbInvariantViolation {
                                        invariant: "cross image inside closed packet",
                                    });
                                }
                                if involution_cartans[new_number] != involution_cartans[index] {
                                    return Err(StructureError::KgbInvariantViolation {
                                        invariant: "cross Cartan class",
                                    });
                                }
                                let created = store.len();
                                dedup.insert(key, created);
                                store.push(child, new_number)?;
                                created
                            }
                        };
                        store.cross[x * semisimple_rank + generator] = k;
                        if d != 0 {
                            store.statuses[x * semisimple_rank + generator] =
                                Some(KgbStatus::Complex);
                        } else if imaginary {
                            let compact = store.elements[x]
                                .negative_at(datum.simple_roots()[generator].as_slice())?;
                            store.statuses[x * semisimple_rank + generator] = Some(if compact {
                                KgbStatus::ImaginaryCompact
                            } else {
                                KgbStatus::ImaginaryNoncompact
                            });
                        } else {
                            if k != x {
                                return Err(StructureError::KgbInvariantViolation {
                                    invariant: "real cross image",
                                });
                            }
                            store.statuses[x * semisimple_rank + generator] = Some(KgbStatus::Real);
                        }
                    }

                    if imaginary {
                        // Cayley links (kgb.cpp:443-466): `prod(s, tw)`,
                        // torus part unchanged, noncompact sources only.
                        let cayley_target = table.cayley(generator, tw_id)?.ok_or(
                            StructureError::KgbInvariantViolation {
                                invariant: "Cayley involution",
                            },
                        )?;
                        let new_number = involution_location[cayley_target.0];
                        is_new = new_number >= first_of_tau.len() - 1;
                        if is_new && new_number != first_of_tau.len() - 1 {
                            return Err(StructureError::KgbInvariantViolation {
                                invariant: "involution generation order",
                            });
                        }
                        for x in packet.clone() {
                            if store.statuses[x * semisimple_rank + generator]
                                == Some(KgbStatus::ImaginaryNoncompact)
                            {
                                let child = store.elements[x].clone();
                                let theta = table
                                    .record(cayley_target)
                                    .ok_or(StructureError::IndexOutOfRange {
                                        index: cayley_target.0,
                                        upper_bound: total_involutions,
                                    })?
                                    .theta();
                                let key = (
                                    cayley_target.0,
                                    fingerprint_cache.fingerprint(
                                        cayley_target,
                                        theta,
                                        &child,
                                        budget,
                                    )?,
                                );
                                let k = match dedup.get(&key) {
                                    Some(&existing) => existing,
                                    None => {
                                        let created = store.len();
                                        dedup.insert(key, created);
                                        store.push(child, new_number)?;
                                        created
                                    }
                                };
                                store.cayley[x * semisimple_rank + generator] = Some(k);
                                let slot =
                                    &mut store.inverse_cayley[k * semisimple_rank + generator];
                                *slot = match slot {
                                    None => Some((x, None)),
                                    Some((first, _)) => Some((*first, Some(x))),
                                };
                            }
                        }
                    }

                    if is_new {
                        first_of_tau.push(store.len());
                    }
                }
            }
        }

        // Every (element, generator) status and cross link is written when
        // the element's packet is visited; the loop above visits all.
        for status in &store.statuses {
            if status.is_none() {
                return Err(StructureError::KgbInvariantViolation {
                    invariant: "element status",
                });
            }
        }

        // The print_X header offset: `Tg.torus_element_offset().log_2pi()`
        // (kgb_io.cpp:152-156, tits.h:166-168): exp_2pi of
        // RatWeight(dual_twoRho, 4).
        let mut dual_two_rho = try_capacity(lattice_rank)?;
        dual_two_rho.resize(lattice_rank, 0_i64);
        for (id, _, coroot) in system.entries() {
            if system.positivity()[id.0] {
                for (sum, &coordinate) in dual_two_rho.iter_mut().zip(coroot.as_slice()) {
                    *sum = sum
                        .checked_add(i64::from(coordinate))
                        .ok_or(StructureError::ArithmeticOverflow)?;
                }
            }
        }
        let header_offset = GlobalTorusElement::exp_2pi(dual_two_rho, 4)?.log_2pi()?;

        Ok(Self {
            semisimple_rank,
            lattice_rank,
            elements: store.elements,
            element_packet: store.element_packet,
            involutions,
            first_of_tau,
            statuses: store.statuses,
            cross: store.cross,
            cayley: store.cayley,
            inverse_cayley: store.inverse_cayley,
            involution_lengths,
            involution_cartans,
            involution_words,
            header_offset,
        })
    }

    pub fn size(&self) -> usize {
        self.elements.len()
    }

    pub fn semisimple_rank(&self) -> usize {
        self.semisimple_rank
    }

    pub fn lattice_rank(&self) -> usize {
        self.lattice_rank
    }

    /// The number of tau packets, one per twisted involution.
    pub fn packet_count(&self) -> usize {
        self.involutions.len()
    }

    /// The element range of tau packet `index` (`KGB_base::tauPacket`).
    pub fn tau_packet(&self, index: usize) -> Option<Range<usize>> {
        if index + 1 < self.first_of_tau.len() {
            Some(self.first_of_tau[index]..self.first_of_tau[index + 1])
        } else {
            None
        }
    }

    pub fn status(&self, element: usize, generator: usize) -> Option<KgbStatus> {
        *self
            .statuses
            .get(element * self.semisimple_rank + generator)?
    }

    /// `KGB_base::cross(s, x)`.
    pub fn cross(&self, generator: usize, element: usize) -> Option<usize> {
        self.cross
            .get(element * self.semisimple_rank + generator)
            .copied()
    }

    /// `KGB_base::cayley(s, x)`; `None` is upstream's `UndefKGB` (`*`).
    pub fn cayley(&self, generator: usize, element: usize) -> Option<usize> {
        self.cayley
            .get(element * self.semisimple_rank + generator)
            .copied()
            .flatten()
    }

    /// `KGB_base::inverseCayley(s, x)`: the first and, when the Cayley
    /// transform is two-valued, second preimage.
    pub fn inverse_cayley(
        &self,
        generator: usize,
        element: usize,
    ) -> Option<(usize, Option<usize>)> {
        self.inverse_cayley
            .get(element * self.semisimple_rank + generator)
            .copied()
            .flatten()
    }

    /// The involution length of the element's involution
    /// (`KGB_base::length`, kgb.h:140-141).
    pub fn length(&self, element: usize) -> Option<usize> {
        self.involution_lengths
            .get(*self.element_packet.get(element)?)
            .copied()
    }

    /// The Cartan class number of the element's involution
    /// (`KGB_base::Cartan_class`, kgb.cpp:154-158).
    pub fn cartan_of(&self, element: usize) -> Option<usize> {
        self.involution_cartans
            .get(*self.element_packet.get(element)?)
            .copied()
    }

    /// The element's printed torus label: its stored representative's
    /// `log_2pi` (`global_KGB::print`, kgb.cpp:318-322).
    pub fn torus_label(&self, element: usize) -> Option<RationalWeight> {
        self.elements.get(element)?.log_2pi().ok()
    }

    /// The printed involution word of a tau packet
    /// (`prettyprint::printInvolution`).
    pub fn involution_word(&self, packet: usize) -> Option<&str> {
        self.involution_words.get(packet).map(String::as_str)
    }

    /// The structured `kgb_io::print_X` output.
    pub fn print_layout(&self) -> Result<GlobalKgbPrint, StructureError> {
        let header = format!(
            "\\exp(i\\pi\\check\\rho) = \\exp(2i\\pi({}))",
            format_rational_weight(&self.header_offset)
        );
        let mut rows = try_capacity(self.size())?;
        for element in 0..self.size() {
            let packet = self.element_packet[element];
            let mut statuses = try_capacity(self.semisimple_rank)?;
            let mut cross = try_capacity(self.semisimple_rank)?;
            let mut cayley = try_capacity(self.semisimple_rank)?;
            for generator in 0..self.semisimple_rank {
                statuses.push(self.status(element, generator).ok_or(
                    StructureError::KgbInvariantViolation {
                        invariant: "element status",
                    },
                )?);
                cross.push(self.cross(generator, element).ok_or(
                    StructureError::IndexOutOfRange {
                        index: element,
                        upper_bound: self.size(),
                    },
                )?);
                cayley.push(self.cayley(generator, element));
            }
            rows.push(GlobalKgbPrintRow {
                length: self.involution_lengths[packet],
                statuses,
                cross,
                cayley,
                torus_label: self.elements[element].log_2pi()?,
                cartan: self.involution_cartans[packet],
                involution_word: self.involution_words[packet].clone(),
            });
        }
        Ok(GlobalKgbPrint {
            header,
            lattice_rank: self.lattice_rank,
            rows,
        })
    }
}

/// The element store under construction: the parallel vectors of
/// `GlobalKgb`, bundled so `push` keeps them in lockstep.
struct ElementStore {
    semisimple_rank: usize,
    elements: Vec<GlobalTorusElement>,
    element_packet: Vec<usize>,
    statuses: Vec<Option<KgbStatus>>,
    cross: Vec<usize>,
    cayley: Vec<Option<usize>>,
    inverse_cayley: Vec<Option<(usize, Option<usize>)>>,
}

impl ElementStore {
    fn new(semisimple_rank: usize) -> Self {
        Self {
            semisimple_rank,
            elements: Vec::new(),
            element_packet: Vec::new(),
            statuses: Vec::new(),
            cross: Vec::new(),
            cayley: Vec::new(),
            inverse_cayley: Vec::new(),
        }
    }

    fn len(&self) -> usize {
        self.elements.len()
    }

    /// Push one element with its per-generator slot placeholders.
    fn push(&mut self, torus: GlobalTorusElement, packet: usize) -> Result<(), StructureError> {
        self.elements.push(torus);
        self.element_packet.push(packet);
        self.statuses
            .try_reserve(self.semisimple_rank)
            .map_err(|_| StructureError::AllocationFailed {
                requested: self.semisimple_rank,
            })?;
        self.cross.try_reserve(self.semisimple_rank).map_err(|_| {
            StructureError::AllocationFailed {
                requested: self.semisimple_rank,
            }
        })?;
        self.cayley.try_reserve(self.semisimple_rank).map_err(|_| {
            StructureError::AllocationFailed {
                requested: self.semisimple_rank,
            }
        })?;
        self.inverse_cayley
            .try_reserve(self.semisimple_rank)
            .map_err(|_| StructureError::AllocationFailed {
                requested: self.semisimple_rank,
            })?;
        for _ in 0..self.semisimple_rank {
            self.statuses.push(None);
            self.cross.push(usize::MAX);
            self.cayley.push(None);
            self.inverse_cayley.push(None);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AdjointFiberBudget, BasedRootDatum, CartanClassificationBudget, Coweight,
        InvolutionTableBudget, Weight,
    };

    fn class_budget(weyl: usize) -> CartanClassificationBudget {
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

    fn lattice_budget() -> IntegerLatticeBudget {
        IntegerLatticeBudget::new(64, 100_000, 100_000, 128)
    }

    fn build_global_kgb(datum: BasedRootDatum, roots: usize, weyl: usize) -> GlobalKgb {
        let distinguished = LatticeInvolution::identity(&datum).unwrap();
        let inner_class = InnerClass::new(datum, distinguished, roots).unwrap();
        let classification =
            CartanClassification::build(&inner_class, &class_budget(weyl)).unwrap();
        let mut table = InvolutionTable::new(
            &inner_class,
            InvolutionTableBudget::new(64, lattice_budget()),
        )
        .unwrap();
        GlobalKgb::build(&inner_class, &classification, &mut table, &lattice_budget()).unwrap()
    }

    /// Simply connected A1: `simply_connected(Lie_type("A1"),true)` of
    /// tests/fixtures/domain/print_x.atlas.
    fn simply_connected_a1() -> BasedRootDatum {
        BasedRootDatum::from_simple_data(
            1,
            vec![vec![2]],
            vec![Weight::new(vec![2])],
            vec![Coweight::new(vec![1])],
        )
        .unwrap()
    }

    /// Adjoint A1: `adjoint(Lie_type("A1"),false)` of the fixture.
    fn adjoint_a1() -> BasedRootDatum {
        BasedRootDatum::from_simple_data(
            1,
            vec![vec![2]],
            vec![Weight::new(vec![1])],
            vec![Coweight::new(vec![2])],
        )
        .unwrap()
    }

    /// Simply connected B2: `simply_connected(Lie_type("B2"),true)` of the
    /// fixture; simple coroots are the coordinate basis, so the simple
    /// roots are the Cartan rows. B2 has 8 roots and a Weyl group of
    /// order 8.
    fn simply_connected_b2() -> BasedRootDatum {
        BasedRootDatum::from_simple_data(
            2,
            vec![vec![2, -2], vec![-1, 2]],
            vec![Weight::new(vec![2, -2]), Weight::new(vec![-1, 2])],
            vec![Coweight::new(vec![1, 0]), Coweight::new(vec![0, 1])],
        )
        .unwrap()
    }

    /// Simply connected A2 with the simple coroots as the coordinate basis.
    fn simply_connected_a2() -> BasedRootDatum {
        BasedRootDatum::from_simple_data(
            2,
            vec![vec![2, -1], vec![-1, 2]],
            vec![Weight::new(vec![2, -1]), Weight::new(vec![-1, 2])],
            vec![Coweight::new(vec![1, 0]), Coweight::new(vec![0, 1])],
        )
        .unwrap()
    }

    fn assert_cached_fingerprints_match(datum: BasedRootDatum, roots: usize, weyl: usize) {
        let distinguished = LatticeInvolution::identity(&datum).unwrap();
        let inner_class = InnerClass::new(datum, distinguished, roots).unwrap();
        let classification =
            CartanClassification::build(&inner_class, &class_budget(weyl)).unwrap();
        let mut table = InvolutionTable::new(
            &inner_class,
            InvolutionTableBudget::new(64, lattice_budget()),
        )
        .unwrap();
        let kgb =
            GlobalKgb::build(&inner_class, &classification, &mut table, &lattice_budget()).unwrap();
        let mut cache = FingerprintCache::new();
        for element in 0..kgb.size() {
            let packet = kgb.element_packet[element];
            let id = kgb.involutions[packet];
            let theta = table.record(id).unwrap().theta();
            let direct = fingerprint(theta, &kgb.elements[element], &lattice_budget()).unwrap();
            let cached = cache
                .fingerprint(id, theta, &kgb.elements[element], &lattice_budget())
                .unwrap();
            assert_eq!(cached, direct, "element {element}");
            for generator in 0..kgb.semisimple_rank() {
                let target = kgb.cross(generator, element).unwrap();
                let target_packet = kgb.element_packet[target];
                let target_id = kgb.involutions[target_packet];
                let target_theta = table.record(target_id).unwrap().theta();
                let direct =
                    fingerprint(target_theta, &kgb.elements[target], &lattice_budget()).unwrap();
                let cached = cache
                    .fingerprint(
                        target_id,
                        target_theta,
                        &kgb.elements[target],
                        &lattice_budget(),
                    )
                    .unwrap();
                assert_eq!(cached, direct, "edge {element}->{target} via {generator}");
            }
        }
    }

    #[test]
    fn fingerprint_cache_matches_direct_projection_a1() {
        assert_cached_fingerprints_match(simply_connected_a1(), 2, 2);
    }

    #[test]
    fn fingerprint_cache_matches_direct_projection_a2() {
        assert_cached_fingerprints_match(simply_connected_a2(), 6, 6);
    }

    #[test]
    fn fingerprint_cache_matches_direct_projection_b2() {
        assert_cached_fingerprints_match(simply_connected_b2(), 8, 8);
    }

    /// The exact `print_X` bytes of the HPC-verified reference
    /// (tests/reference/domain/print_x.events.json, first print_X block).
    #[test]
    fn simply_connected_a1_print_matches_hpc_reference() {
        let kgb = build_global_kgb(simply_connected_a1(), 2, 2);
        let expected = "\\exp(i\\pi\\check\\rho) = \\exp(2i\\pi([1]/4))\n\
            0:  0  [n]   1    4   [0]/1 0 e\n\
            1:  0  [n]   0    4   [1]/2 0 e\n\
            2:  0  [c]   2    *   [1]/4 0 e\n\
            3:  0  [c]   3    *   [3]/4 0 e\n\
            4:  1  [r]   4    *   [0]/1 1 1^e\n";
        assert_eq!(kgb.print_layout().unwrap().render(), expected);
    }

    /// The fixture's second print_X block (adjoint A1).
    #[test]
    fn adjoint_a1_print_matches_hpc_reference() {
        let kgb = build_global_kgb(adjoint_a1(), 2, 2);
        let expected = "\\exp(i\\pi\\check\\rho) = \\exp(2i\\pi([1]/2))\n\
            0:  0  [n]   0    2   [0]/1 0 e\n\
            1:  0  [c]   1    *   [1]/2 0 e\n\
            2:  1  [r]   2    *   [0]/1 1 1^e\n";
        assert_eq!(kgb.print_layout().unwrap().render(), expected);
    }

    /// The fixture's third print_X block (simply connected B2), including
    /// the negative-numerator label `[0,-1]/2` of element 15 that pins the
    /// unreduced post-reflection numerator discipline.
    #[test]
    fn simply_connected_b2_print_matches_hpc_reference() {
        let kgb = build_global_kgb(simply_connected_b2(), 8, 8);
        let expected = "\\exp(i\\pi\\check\\rho) = \\exp(2i\\pi([0,3]/4))\n\
            \x200:  0  [n,n]    1   2     8  10    [0,0]/1 0 e\n\
            \x201:  0  [n,c]    0   1     8   *    [1,0]/2 0 e\n\
            \x202:  0  [n,n]    3   0     9  10    [0,1]/2 0 e\n\
            \x203:  0  [n,c]    2   3     9   *    [1,1]/2 0 e\n\
            \x204:  0  [c,n]    4   6     *  11    [2,1]/4 0 e\n\
            \x205:  0  [c,c]    5   5     *   *    [0,1]/4 0 e\n\
            \x206:  0  [c,n]    6   4     *  11    [2,3]/4 0 e\n\
            \x207:  0  [c,c]    7   7     *   *    [0,3]/4 0 e\n\
            \x208:  1  [r,C]    8  14     *   *    [0,0]/1 1 1^e\n\
            \x209:  1  [r,C]    9  15     *   *    [0,1]/2 1 1^e\n\
            10:  1  [C,r]   12  10     *   *    [0,0]/1 2 2^e\n\
            11:  1  [C,r]   13  11     *   *    [2,1]/4 2 2^e\n\
            12:  2  [C,n]   10  12     *  16    [0,0]/1 2 1x2^e\n\
            13:  2  [C,c]   11  13     *   *    [0,1]/4 2 1x2^e\n\
            14:  2  [n,C]   15   8    16   *    [0,0]/1 1 2x1^e\n\
            15:  2  [n,C]   14   9    16   *   [0,-1]/2 1 2x1^e\n\
            16:  3  [r,r]   16  16     *   *    [0,0]/1 3 1^2x1^e\n";
        assert_eq!(kgb.print_layout().unwrap().render(), expected);
    }

    /// Structural invariants of the B2 graph that do not depend on the
    /// printed bytes: packet sizes, involution words, cross involutivity,
    /// and the Cayley/inverse-Cayley pairing.
    #[test]
    fn b2_packet_structure_and_link_invariants() {
        let kgb = build_global_kgb(simply_connected_b2(), 8, 8);
        assert_eq!(kgb.size(), 17);
        assert_eq!(kgb.packet_count(), 6);
        let packet_sizes: Vec<usize> = (0..kgb.packet_count())
            .map(|packet| kgb.tau_packet(packet).unwrap().len())
            .collect();
        assert_eq!(packet_sizes, vec![8, 2, 2, 2, 2, 1]);
        let words: Vec<&str> = (0..kgb.packet_count())
            .map(|packet| kgb.involution_word(packet).unwrap())
            .collect();
        assert_eq!(words, vec!["e", "1^e", "2^e", "1x2^e", "2x1^e", "1^2x1^e"]);
        // Cross actions are involutions on the element set.
        for element in 0..kgb.size() {
            for generator in 0..kgb.semisimple_rank() {
                let image = kgb.cross(generator, element).unwrap();
                assert_eq!(kgb.cross(generator, image), Some(element));
            }
        }
        // Cayley and inverse-Cayley agree.
        for element in 0..kgb.size() {
            for generator in 0..kgb.semisimple_rank() {
                if let Some(image) = kgb.cayley(generator, element) {
                    let pair = kgb.inverse_cayley(generator, image).unwrap();
                    assert!(pair.0 == element || pair.1 == Some(element));
                }
            }
        }
    }

    // NOTE: semisimple-rank-0 boundary cases (the trivial group and the
    // one-dimensional torus) are intentionally untested: the shared
    // inner-class machinery panics on an empty generator set
    // (weyl_transducer.rs:485, index out of bounds inside
    // `InnerClass::new`), which is outside this module's scope to fix.
}
