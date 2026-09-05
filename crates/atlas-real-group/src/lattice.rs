use malachite::Rational;

use crate::StructureError;

/// An element of the character lattice `X^*`.
///
/// Coordinates intentionally stay in checked fixed-width storage. Exact
/// interpreter values convert at the domain boundary rather than changing the
/// representation of every root-system matrix entry.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Weight(Vec<i32>);

impl Weight {
    pub fn new(coordinates: Vec<i32>) -> Self {
        Self(coordinates)
    }

    pub fn rank(&self) -> usize {
        self.0.len()
    }

    pub fn as_slice(&self) -> &[i32] {
        &self.0
    }
}

/// An element of the cocharacter lattice `X_*`.
///
/// This is deliberately distinct from [`Weight`]. The two lattices have a
/// perfect pairing but are not interchangeable, even when a chosen basis has
/// the same coordinate representation.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Coweight(Vec<i32>);

impl Coweight {
    pub fn new(coordinates: Vec<i32>) -> Self {
        Self(coordinates)
    }

    pub fn rank(&self) -> usize {
        self.0.len()
    }

    pub fn as_slice(&self) -> &[i32] {
        &self.0
    }
}

/// Evaluate the canonical character--cocharacter pairing.
pub fn pair(weight: &Weight, coweight: &Coweight) -> Result<i32, StructureError> {
    if weight.rank() != coweight.rank() {
        return Err(StructureError::RankMismatch {
            expected: weight.rank(),
            actual: coweight.rank(),
        });
    }

    pair_coordinates(weight.as_slice(), coweight.as_slice())
}

/// Fallibly copy fixed-width coordinates, mapping reservation failure to
/// [`StructureError::AllocationFailed`].
pub(crate) fn try_copy_coordinates(values: &[i32]) -> Result<Vec<i32>, StructureError> {
    let mut copy = Vec::new();
    copy.try_reserve_exact(values.len())
        .map_err(|_| StructureError::AllocationFailed {
            requested: values.len(),
        })?;
    copy.extend_from_slice(values);
    Ok(copy)
}

pub(crate) fn pair_coordinates(left: &[i32], right: &[i32]) -> Result<i32, StructureError> {
    let n = left.len().min(right.len());
    let mut total = 0_i64;
    for i in 0..n {
        // i32 * i32 always fits in i64.
        let product = i64::from(left[i]) * i64::from(right[i]);
        match total.checked_add(product) {
            Some(sum) => total = sum,
            None => return pair_coordinates_wide(left, right, n),
        }
    }
    i32::try_from(total).map_err(|_| StructureError::ArithmeticOverflow)
}

/// i128 accumulation for partial sums that leave the i64 window; with i32
/// inputs this stays exact for any feasible length.
#[cold]
fn pair_coordinates_wide(left: &[i32], right: &[i32], n: usize) -> Result<i32, StructureError> {
    let mut total = 0_i128;
    for i in 0..n {
        total += i128::from(left[i]) * i128::from(right[i]);
    }
    i32::try_from(total).map_err(|_| StructureError::ArithmeticOverflow)
}

/// Checked coordinatewise sum of two equal-rank weights.
pub(crate) fn checked_add_weights(left: &Weight, right: &Weight) -> Result<Weight, StructureError> {
    combine_weights(left, right, 1)
}

/// Checked coordinatewise difference of two equal-rank weights.
pub(crate) fn checked_sub_weights(left: &Weight, right: &Weight) -> Result<Weight, StructureError> {
    combine_weights(left, right, -1)
}

fn combine_weights(left: &Weight, right: &Weight, sign: i32) -> Result<Weight, StructureError> {
    if left.rank() != right.rank() {
        return Err(StructureError::RankMismatch {
            expected: left.rank(),
            actual: right.rank(),
        });
    }
    let mut coordinates = Vec::new();
    coordinates
        .try_reserve_exact(left.rank())
        .map_err(|_| StructureError::AllocationFailed {
            requested: left.rank(),
        })?;
    for (&left_entry, &right_entry) in left.as_slice().iter().zip(right.as_slice()) {
        let scaled = right_entry
            .checked_mul(sign)
            .ok_or(StructureError::ArithmeticOverflow)?;
        coordinates.push(
            left_entry
                .checked_add(scaled)
                .ok_or(StructureError::ArithmeticOverflow)?,
        );
    }
    Ok(Weight::new(coordinates))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pairing_rejects_out_of_range_results() {
        assert_eq!(
            pair(
                &Weight::new(vec![i32::MAX, i32::MAX]),
                &Coweight::new(vec![2, 2]),
            ),
            Err(StructureError::ArithmeticOverflow)
        );
    }

    #[test]
    fn pairing_keeps_weight_and_coweight_distinct() {
        let weight = Weight::new(vec![2, -1]);
        let coweight = Coweight::new(vec![3, 4]);
        assert_eq!(pair(&weight, &coweight), Ok(2));
    }

    #[test]
    fn pairing_wide_fallback_recovers_when_partial_sums_leave_i64() {
        // Products: MAX^2, MAX^2 (2*MAX^2 still fits i64), MIN^2 (partial
        // sum 3*2^62 - 2^33 + 2 overshoots i64), then three MIN*MAX. The
        // wide path must return the exact total -2^31 + 2.
        let value = pair_coordinates(
            &[i32::MAX, i32::MAX, i32::MIN, i32::MIN, i32::MIN, i32::MIN],
            &[i32::MAX, i32::MAX, i32::MIN, i32::MAX, i32::MAX, i32::MAX],
        );
        assert_eq!(value, Ok(i32::MIN + 2));
        let out_of_range = pair_coordinates(
            &[i32::MAX, i32::MAX, i32::MIN, i32::MIN, i32::MIN, i32::MIN, i32::MIN],
            &[i32::MAX, i32::MAX, i32::MIN, i32::MAX, i32::MAX, i32::MAX, i32::MAX],
        );
        assert_eq!(out_of_range, Err(StructureError::ArithmeticOverflow));
    }
}

/// An exact rational character value: upstream `RatWeight`
/// (utilities/ratvec.h). Storage mirrors the C++ layout — one integer
/// numerator vector over a single common denominator — because the Atlas
/// display layer prints exactly that pair (`[ 5, 0 ]/2`), and gcd-normalized
/// common-denominator storage makes that printing convention, and value
/// equality, canonical. Denominators stay positive; construction normalizes
/// by the gcd of the denominator and every numerator entry (ratvec.cpp:172).
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RationalWeight {
    numerator: Vec<i64>,
    denominator: i64,
}

impl RationalWeight {
    /// Build and gcd-normalize; rejects a non-positive denominator, as
    /// upstream's `normalize` rejects zero (ratvec.cpp:172-175).
    pub fn new(numerator: Vec<i64>, denominator: i64) -> Result<Self, StructureError> {
        if denominator <= 0 {
            return Err(StructureError::RepInvariantViolation {
                invariant: "rational weight denominator",
            });
        }
        let mut gcd = denominator.unsigned_abs();
        for &entry in &numerator {
            gcd = gcd_u64(entry.unsigned_abs(), gcd);
            if gcd == 1 {
                break;
            }
        }
        let divisor = i64::try_from(gcd).map_err(|_| StructureError::ArithmeticOverflow)?;
        let mut normalized = Vec::new();
        normalized.try_reserve_exact(numerator.len()).map_err(|_| {
            StructureError::AllocationFailed {
                requested: numerator.len(),
            }
        })?;
        for entry in numerator {
            normalized.push(
                entry
                    .checked_div(divisor)
                    .ok_or(StructureError::ArithmeticOverflow)?,
            );
        }
        Ok(Self {
            numerator: normalized,
            denominator: denominator / divisor,
        })
    }

    /// An integral weight viewed as a rational weight with denominator 1.
    pub fn from_weight(weight: &Weight) -> Result<Self, StructureError> {
        let mut numerator = Vec::new();
        numerator
            .try_reserve_exact(weight.as_slice().len())
            .map_err(|_| StructureError::AllocationFailed {
                requested: weight.as_slice().len(),
            })?;
        for &coordinate in weight.as_slice() {
            numerator.push(i64::from(coordinate));
        }
        Self::new(numerator, 1)
    }

    pub fn zero(rank: usize) -> Result<Self, StructureError> {
        Self::new(vec![0; rank], 1)
    }

    pub fn rank(&self) -> usize {
        self.numerator.len()
    }

    pub fn numerator(&self) -> &[i64] {
        &self.numerator
    }

    pub fn denominator(&self) -> i64 {
        self.denominator
    }

    pub fn is_zero(&self) -> bool {
        self.numerator.iter().all(|&entry| entry == 0)
    }

    pub fn add(&self, right: &Self) -> Result<Self, StructureError> {
        self.combine(right, 1)
    }

    pub fn sub(&self, right: &Self) -> Result<Self, StructureError> {
        self.combine(right, -1)
    }

    /// Exact sum/difference on a common denominator, then normalize.
    fn combine(&self, right: &Self, sign: i64) -> Result<Self, StructureError> {
        if self.rank() != right.rank() {
            return Err(StructureError::RankMismatch {
                expected: self.rank(),
                actual: right.rank(),
            });
        }
        let denominator = self
            .denominator
            .checked_mul(right.denominator)
            .ok_or(StructureError::ArithmeticOverflow)?;
        let mut numerator = Vec::new();
        numerator
            .try_reserve_exact(self.rank())
            .map_err(|_| StructureError::AllocationFailed {
                requested: self.rank(),
            })?;
        for (&left, &right_entry) in self.numerator.iter().zip(&right.numerator) {
            let scaled_left = left
                .checked_mul(right.denominator)
                .ok_or(StructureError::ArithmeticOverflow)?;
            let scaled_right = right_entry
                .checked_mul(self.denominator)
                .and_then(|value| value.checked_mul(sign))
                .ok_or(StructureError::ArithmeticOverflow)?;
            numerator.push(
                scaled_left
                    .checked_add(scaled_right)
                    .ok_or(StructureError::ArithmeticOverflow)?,
            );
        }
        Self::new(numerator, denominator)
    }

    /// Apply an integer matrix (an involution's weight action) to the
    /// numerator, keeping the denominator (ratvec.cpp:190).
    pub(crate) fn apply_matrix(&self, matrix: &[Vec<i32>]) -> Result<Self, StructureError> {
        if matrix.len() != self.rank() || matrix.iter().any(|row| row.len() != self.rank()) {
            return Err(StructureError::RankMismatch {
                expected: self.rank(),
                actual: matrix.len(),
            });
        }
        let mut numerator = Vec::new();
        numerator
            .try_reserve_exact(self.rank())
            .map_err(|_| StructureError::AllocationFailed {
                requested: self.rank(),
            })?;
        for row in matrix {
            let mut entry = 0_i64;
            for (&coefficient, &coordinate) in row.iter().zip(&self.numerator) {
                let product = i64::from(coefficient)
                    .checked_mul(coordinate)
                    .ok_or(StructureError::ArithmeticOverflow)?;
                entry = entry
                    .checked_add(product)
                    .ok_or(StructureError::ArithmeticOverflow)?;
            }
            numerator.push(entry);
        }
        Self::new(numerator, self.denominator)
    }

    /// Halve the value by doubling the denominator, without normalizing;
    /// callers normalize at the point upstream does.
    pub(crate) fn halve(&self) -> Result<Self, StructureError> {
        Ok(Self {
            numerator: self.numerator.clone(),
            denominator: self
                .denominator
                .checked_mul(2)
                .ok_or(StructureError::ArithmeticOverflow)?,
        })
    }

    /// Re-run the constructor's gcd normalization (ratvec.cpp:172).
    pub(crate) fn normalized(&self) -> Result<Self, StructureError> {
        Self::new(self.numerator.clone(), self.denominator)
    }

    /// The numerator divided by the denominator, requiring integrality —
    /// the checked form of upstream's `assert(entry%denominator==0)`
    /// divisions (repr.cpp:194-199, 768-774).
    pub(crate) fn integral_coordinates(&self) -> Result<Vec<i64>, StructureError> {
        let mut coordinates = Vec::new();
        coordinates.try_reserve_exact(self.rank()).map_err(|_| {
            StructureError::AllocationFailed {
                requested: self.rank(),
            }
        })?;
        for &entry in &self.numerator {
            if entry % self.denominator != 0 {
                return Err(StructureError::RepInvariantViolation {
                    invariant: "rational weight integrality",
                });
            }
            coordinates.push(entry / self.denominator);
        }
        Ok(coordinates)
    }

    /// Scalar multiplication by the rational `numerator/denominator`,
    /// normalized (the RatWeight scalar products of K_repr.cpp
    /// height_bound/monomial arithmetic).
    pub(crate) fn scale(&self, numerator: i64, denominator: i64) -> Result<Self, StructureError> {
        if denominator <= 0 {
            return Err(StructureError::RepInvariantViolation {
                invariant: "rational weight scale denominator",
            });
        }
        let mut scaled = Vec::new();
        scaled
            .try_reserve_exact(self.rank())
            .map_err(|_| StructureError::AllocationFailed {
                requested: self.rank(),
            })?;
        for &entry in &self.numerator {
            scaled.push(
                entry
                    .checked_mul(numerator)
                    .ok_or(StructureError::ArithmeticOverflow)?,
            );
        }
        Self::new(
            scaled,
            self.denominator
                .checked_mul(denominator)
                .ok_or(StructureError::ArithmeticOverflow)?,
        )
    }

    /// `<lambda, coroot>` as the reduced rational `(numerator,
    /// denominator)` (the `dot_Q` of a rational weight with a coroot,
    /// ratvec.h:167-175).
    pub(crate) fn dot_coroot(&self, coroot: &Coweight) -> Result<(i64, i64), StructureError> {
        if self.rank() != coroot.rank() {
            return Err(StructureError::RankMismatch {
                expected: coroot.rank(),
                actual: self.rank(),
            });
        }
        let mut numerator = 0_i64;
        for (&entry, &coordinate) in self.numerator.iter().zip(coroot.as_slice()) {
            numerator = numerator
                .checked_add(
                    entry
                        .checked_mul(i64::from(coordinate))
                        .ok_or(StructureError::ArithmeticOverflow)?,
                )
                .ok_or(StructureError::ArithmeticOverflow)?;
        }
        let mut divisor = self.denominator.unsigned_abs();
        divisor = gcd_u64(numerator.unsigned_abs(), divisor);
        let divisor = i64::try_from(divisor).map_err(|_| StructureError::ArithmeticOverflow)?;
        Ok((numerator / divisor, self.denominator / divisor))
    }
}

fn gcd_u64(mut left: u64, mut right: u64) -> u64 {
    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }
    left.max(1)
}

/// An exact rational cocharacter value (stage (d)'s `some_coch` output).
///
/// Storage stays private malachite data: third-party types stay out of the
/// public API; the language adapter reads derived views when it arrives, and
/// in-crate consumers (`torus_factor`) use the exact coordinates.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RationalCoweight {
    coordinates: Vec<Rational>,
}

impl RationalCoweight {
    pub(crate) fn from_coordinates(coordinates: Vec<Rational>) -> Self {
        Self { coordinates }
    }

    pub fn dimension(&self) -> usize {
        self.coordinates.len()
    }

    /// Exact coordinate view for workspace consumers (the interpreter's
    /// value layer already speaks malachite rationals publicly).
    pub fn to_rationals(&self) -> Vec<Rational> {
        self.coordinates.clone()
    }

    pub(crate) fn coordinates(&self) -> &[Rational] {
        &self.coordinates
    }
}
