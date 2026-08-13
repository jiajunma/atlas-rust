//! Mathematical key/codec primitives shared by representation tables.

use crate::matreduc::IntMatrix;
use crate::real_projection::RealProjection;
use crate::{KgbId, RationalWeight, StructureError, Weight};

/// Identity of the integral root system used to reduce a parameter.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum IntegralSystem {
    /// The full root system, without allocating an integral-system table slot.
    Full,
    /// A non-full integral system already interned by its owning table.
    Interned(u32),
}

/// Hash-stable identity of a reduced parameter.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct ReducedParamKey {
    pub(crate) x: KgbId,
    pub(crate) integral_system: IntegralSystem,
    pub(crate) residue: u32,
}

impl ReducedParamKey {
    pub(crate) const fn new(x: KgbId, integral_system: IntegralSystem, residue: u32) -> Self {
        Self {
            x,
            integral_system,
            residue,
        }
    }
}

/// Smith-style codec for integral-coroot evaluations modulo
/// `(1-theta)X^*`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct IntegralCodec {
    coroots: IntMatrix,
    diagonal: Vec<i32>,
    input: IntMatrix,
    output: IntMatrix,
}

impl IntegralCodec {
    pub(crate) fn new(
        projection: &RealProjection,
        coroots: &IntMatrix,
    ) -> Result<Self, StructureError> {
        let ambient_rank = projection.lift_mat.len();
        let image_rank = projection.image_rank();
        if coroots.n_columns() != ambient_rank
            || projection
                .lift_mat
                .iter()
                .any(|row| row.len() != image_rank)
            || projection
                .m_real
                .iter()
                .any(|row| row.len() != ambient_rank)
        {
            return Err(StructureError::InvalidIntegerMatrixShape);
        }

        // Upstream codec::codec: A is the integral-coroot evaluation map
        // restricted to the transported basis of (1-theta)X^*.
        let mut image_evaluations = IntMatrix::new(coroots.n_rows(), image_rank);
        for row in 0..coroots.n_rows() {
            for column in 0..image_rank {
                let mut value = 0_i128;
                for index in 0..ambient_rank {
                    let term = i128::from(coroots.get(row, index))
                        .checked_mul(i128::from(projection.lift_mat[index][column]))
                        .ok_or(StructureError::ArithmeticOverflow)?;
                    value = value
                        .checked_add(term)
                        .ok_or(StructureError::ArithmeticOverflow)?;
                }
                image_evaluations.set(
                    row,
                    column,
                    i32::try_from(value).map_err(|_| StructureError::ArithmeticOverflow)?,
                );
            }
        }

        let (mut input, columns, mut diagonal) = crate::matreduc::diagonalise(&image_evaluations);
        if diagonal.first().is_some_and(|entry| *entry < 0) {
            diagonal[0] = diagonal[0]
                .checked_neg()
                .ok_or(StructureError::ArithmeticOverflow)?;
            for column in 0..input.n_columns() {
                input.set(
                    0,
                    column,
                    input
                        .get(0, column)
                        .checked_neg()
                        .ok_or(StructureError::ArithmeticOverflow)?,
                );
            }
        }

        // Keep only the columns corresponding to nonzero invariant factors.
        // As upstream notes, every use of `col` is immediately preceded by
        // multiplication with the transported image basis.
        let mut output = IntMatrix::new(ambient_rank, diagonal.len());
        for row in 0..ambient_rank {
            for column in 0..diagonal.len() {
                let mut value = 0_i128;
                for index in 0..image_rank {
                    let term = i128::from(projection.lift_mat[row][index])
                        .checked_mul(i128::from(columns.get(index, column)))
                        .ok_or(StructureError::ArithmeticOverflow)?;
                    value = value
                        .checked_add(term)
                        .ok_or(StructureError::ArithmeticOverflow)?;
                }
                output.set(
                    row,
                    column,
                    i32::try_from(value).map_err(|_| StructureError::ArithmeticOverflow)?,
                );
            }
        }

        Ok(Self {
            coroots: coroots.clone(),
            diagonal,
            input,
            output,
        })
    }

    pub(crate) fn internalise(
        &self,
        gamma_lambda: &RationalWeight,
    ) -> Result<Vec<i32>, StructureError> {
        if gamma_lambda.rank() != self.coroots.n_columns() {
            return Err(StructureError::RankMismatch {
                expected: self.coroots.n_columns(),
                actual: gamma_lambda.rank(),
            });
        }

        let denominator = i128::from(gamma_lambda.denominator());
        let mut evaluations = Vec::new();
        evaluations
            .try_reserve_exact(self.coroots.n_rows())
            .map_err(|_| StructureError::AllocationFailed {
                requested: self.coroots.n_rows(),
            })?;
        for row in 0..self.coroots.n_rows() {
            let mut numerator = 0_i128;
            for (column, &coordinate) in gamma_lambda.numerator().iter().enumerate() {
                let term = i128::from(self.coroots.get(row, column))
                    .checked_mul(i128::from(coordinate))
                    .ok_or(StructureError::ArithmeticOverflow)?;
                numerator = numerator
                    .checked_add(term)
                    .ok_or(StructureError::ArithmeticOverflow)?;
            }
            if numerator.rem_euclid(denominator) != 0 {
                return Err(StructureError::RepInvariantViolation {
                    invariant: "integral coroot evaluation",
                });
            }
            evaluations.push(
                i32::try_from(numerator / denominator)
                    .map_err(|_| StructureError::ArithmeticOverflow)?,
            );
        }
        self.input.apply_to(&mut evaluations);
        Ok(evaluations)
    }

    pub(crate) fn residue(&self, gamma_lambda: &RationalWeight) -> Result<u32, StructureError> {
        let evaluations = self.internalise(gamma_lambda)?;
        let mut packed = 0_u32;
        for (index, &modulus) in self.diagonal.iter().enumerate() {
            debug_assert!(modulus > 0);
            let radix =
                u32::try_from(modulus).map_err(|_| StructureError::RepInvariantViolation {
                    invariant: "positive codec diagonal",
                })?;
            let digit = u32::try_from(evaluations[index].rem_euclid(modulus))
                .map_err(|_| StructureError::ArithmeticOverflow)?;
            packed = packed.wrapping_mul(radix).wrapping_add(digit);
        }
        Ok(packed)
    }

    pub(crate) fn reduced_key(
        &self,
        x: KgbId,
        integral_system: IntegralSystem,
        gamma_lambda: &RationalWeight,
    ) -> Result<ReducedParamKey, StructureError> {
        Ok(ReducedParamKey::new(
            x,
            integral_system,
            self.residue(gamma_lambda)?,
        ))
    }

    pub(crate) fn theta_1_preimage(
        &self,
        difference: &RationalWeight,
    ) -> Result<Weight, StructureError> {
        let evaluations = self.internalise(difference)?;
        let mut coordinates = Vec::new();
        coordinates
            .try_reserve_exact(self.diagonal.len())
            .map_err(|_| StructureError::AllocationFailed {
                requested: self.diagonal.len(),
            })?;
        for (index, &modulus) in self.diagonal.iter().enumerate() {
            if evaluations[index].rem_euclid(modulus) != 0 {
                return Err(StructureError::RepInvariantViolation {
                    invariant: "theta-1 preimage divisibility",
                });
            }
            coordinates.push(evaluations[index] / modulus);
        }
        if evaluations[self.diagonal.len()..]
            .iter()
            .any(|&entry| entry != 0)
        {
            return Err(StructureError::RepInvariantViolation {
                invariant: "theta-1 preimage trailing evaluations",
            });
        }

        let mut weight = Vec::new();
        weight
            .try_reserve_exact(self.output.n_rows())
            .map_err(|_| StructureError::AllocationFailed {
                requested: self.output.n_rows(),
            })?;
        for row in 0..self.output.n_rows() {
            let mut value = 0_i64;
            for (column, &coordinate) in coordinates.iter().enumerate() {
                let term = i64::from(self.output.get(row, column))
                    .checked_mul(i64::from(coordinate))
                    .ok_or(StructureError::ArithmeticOverflow)?;
                value = value
                    .checked_add(term)
                    .ok_or(StructureError::ArithmeticOverflow)?;
            }
            weight.push(i32::try_from(value).map_err(|_| StructureError::ArithmeticOverflow)?);
        }
        Ok(Weight::new(weight))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use super::*;

    fn projection(lift_entries: &[i64]) -> RealProjection {
        let rank = lift_entries.len();
        let mut lift_mat = vec![vec![0_i64; rank]; rank];
        let mut m_real = vec![vec![0_i64; rank]; rank];
        for (index, &entry) in lift_entries.iter().enumerate() {
            lift_mat[index][index] = entry;
            m_real[index][index] = 1;
        }
        RealProjection { lift_mat, m_real }
    }

    fn diagonal_matrix(entries: &[i32]) -> IntMatrix {
        let rank = entries.len();
        let mut matrix = IntMatrix::new(rank, rank);
        for (index, &entry) in entries.iter().enumerate() {
            matrix.set(index, index, entry);
        }
        matrix
    }

    #[test]
    fn residue_uses_euclidean_remainder_for_negative_evaluation() {
        let codec = IntegralCodec::new(&projection(&[2]), &diagonal_matrix(&[1])).unwrap();
        let gamma_lambda = RationalWeight::new(vec![-1], 1).unwrap();

        assert_eq!(codec.residue(&gamma_lambda), Ok(1));
    }

    #[test]
    fn residue_packs_multiple_digits_in_upstream_order() {
        let codec = IntegralCodec::new(&projection(&[2, 3]), &diagonal_matrix(&[1, 1])).unwrap();
        let gamma_lambda = RationalWeight::new(vec![-1, 5], 1).unwrap();

        assert_eq!(codec.residue(&gamma_lambda), Ok(5));
    }

    #[test]
    fn residue_deliberately_wraps_u32_mixed_radix_overflow() {
        let codec = IntegralCodec::new(
            &projection(&[65_536, 65_536, 2]),
            &diagonal_matrix(&[1, 1, 1]),
        )
        .unwrap();
        let gamma_lambda = RationalWeight::new(vec![65_535, 65_535, 0], 1).unwrap();

        assert_eq!(codec.residue(&gamma_lambda), Ok(u32::MAX - 1));
    }

    #[test]
    fn internalise_rejects_nonintegral_coroot_evaluations() {
        let codec = IntegralCodec::new(&projection(&[2]), &diagonal_matrix(&[1])).unwrap();
        let gamma_lambda = RationalWeight::new(vec![1], 2).unwrap();

        assert_eq!(
            codec.internalise(&gamma_lambda),
            Err(StructureError::RepInvariantViolation {
                invariant: "integral coroot evaluation",
            })
        );
    }

    #[test]
    fn construction_rejects_incompatible_matrix_shapes() {
        let coroots = IntMatrix::from_entries(1, 2, vec![1, 0]);

        assert_eq!(
            IntegralCodec::new(&projection(&[2]), &coroots),
            Err(StructureError::InvalidIntegerMatrixShape)
        );
    }

    #[test]
    fn theta_1_preimage_recovers_an_image_weight() {
        let codec = IntegralCodec::new(&projection(&[2, 3]), &diagonal_matrix(&[1, 1])).unwrap();
        let difference = RationalWeight::new(vec![8, -6], 1).unwrap();

        assert_eq!(
            codec.theta_1_preimage(&difference),
            Ok(Weight::new(vec![8, -6]))
        );
    }

    #[test]
    fn reduced_keys_have_stable_equality_and_hash_identity() {
        let full = ReducedParamKey::new(KgbId(7), IntegralSystem::Full, 11);
        let same = ReducedParamKey::new(KgbId(7), IntegralSystem::Full, 11);
        let interned = ReducedParamKey::new(KgbId(7), IntegralSystem::Interned(0), 11);
        let other_residue = ReducedParamKey::new(KgbId(7), IntegralSystem::Full, 12);

        assert_eq!(full, same);
        assert_ne!(full, interned);
        assert_ne!(full, other_residue);

        let mut set = HashSet::new();
        assert!(set.insert(full));
        assert!(!set.insert(same));
        assert!(set.insert(interned));

        let mut map = HashMap::new();
        map.insert(full, "full");
        map.insert(other_residue, "other residue");
        assert_eq!(map[&same], "full");
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn reduced_key_combines_codec_residue_with_identity_domain() {
        let codec = IntegralCodec::new(&projection(&[2]), &diagonal_matrix(&[1])).unwrap();
        let gamma_lambda = RationalWeight::new(vec![-1], 1).unwrap();

        assert_eq!(
            codec.reduced_key(KgbId(3), IntegralSystem::Full, &gamma_lambda),
            Ok(ReducedParamKey::new(KgbId(3), IntegralSystem::Full, 1))
        );
    }
}
