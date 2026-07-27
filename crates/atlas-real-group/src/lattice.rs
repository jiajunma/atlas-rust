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
    let value = left
        .iter()
        .zip(right)
        .try_fold(0_i128, |sum, (&left, &right)| {
            let product = i128::from(left)
                .checked_mul(i128::from(right))
                .ok_or(StructureError::ArithmeticOverflow)?;
            sum.checked_add(product)
                .ok_or(StructureError::ArithmeticOverflow)
        })?;
    i32::try_from(value).map_err(|_| StructureError::ArithmeticOverflow)
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

    pub(crate) fn coordinates(&self) -> &[Rational] {
        &self.coordinates
    }
}
