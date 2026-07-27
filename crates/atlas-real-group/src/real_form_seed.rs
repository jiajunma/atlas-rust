//! The KGB seed `x0` (stage d) — mathematical substrate.
//!
//! This module lands in two steps per `SEED_X0_DESIGN.md`: the exact
//! rational machinery first (`stable_log`, the fundamental coweights), the
//! `RealFormSeed` builder next. The elections here are OBSERVABLE-BEARING:
//! `stable_log`'s adapted-basis representative fixes `g_rho_check`, hence
//! every downstream `torus_factor` rational.

use malachite::base::num::arithmetic::traits::Floor;
use malachite::base::num::basic::traits::{One, Zero};
use malachite::Rational;

use crate::grading::try_capacity;
use crate::integer_lattice::{adapted_basis, IntegerLatticeBudget};
use crate::{pair, BasedRootDatum, StructureError};

/// Elected `xi^T`-stable logarithm (upstream `stable_log`,
/// y_values.cpp:155-166): reduce the input per-coordinate mod 1
/// (nonnegative), take the first-`d` adapted-basis coordinates of `xi + 1`,
/// reduce THOSE mod 1 (i.e. modulo the full fixed lattice), convert back.
/// The output lies EXACTLY in the +1-eigenspace.
///
/// PRECONDITION, checked: the trailing adapted-basis coordinates of the
/// reduced input are integral — equivalently the input is congruent mod
/// `X_*` to an exactly-fixed vector. `some_coch` inputs satisfy this
/// structurally; general squares need not.
#[allow(dead_code)] // Stage-(d) substrate; consumed once the seed builder lands.
pub(crate) fn stable_log(
    coordinates: &[Rational],
    coweight_action_plus_one: &[Vec<i32>],
    budget: &IntegerLatticeBudget,
) -> Result<Vec<Rational>, StructureError> {
    let dimension = coordinates.len();
    if coweight_action_plus_one.len() != dimension {
        return Err(StructureError::RankMismatch {
            expected: dimension,
            actual: coweight_action_plus_one.len(),
        });
    }
    let adapted = adapted_basis(coweight_action_plus_one, budget)?;
    let fixed_rank = adapted.diagonal.len();

    let mut reduced = try_capacity(dimension)?;
    for coordinate in coordinates {
        reduced.push(fractional_part(coordinate));
    }

    // All adapted-basis coordinates of the reduced vector; the trailing ones
    // carry the integrality precondition.
    let mut stable = try_capacity(fixed_rank)?;
    for row in 0..dimension {
        let mut value = Rational::ZERO;
        for (column, coordinate) in reduced.iter().enumerate() {
            value += Rational::from(adapted.inverse.entry(row, column).clone()) * coordinate;
        }
        if row < fixed_rank {
            stable.push(fractional_part(&value));
        } else if fractional_part(&value) != Rational::ZERO {
            return Err(StructureError::SeedInvariantViolation {
                invariant: "stable-log integrality",
            });
        }
    }

    let mut result = try_capacity(dimension)?;
    for row in 0..dimension {
        let mut value = Rational::ZERO;
        for (column, coordinate) in stable.iter().enumerate() {
            value += Rational::from(adapted.basis.entry(row, column).clone()) * coordinate;
        }
        result.push(value);
    }
    Ok(result)
}

/// The fundamental coweights `varpi_i^vee = sum_j (C^{-1})_{ji}
/// alpha_j^vee` in full lattice-rank coordinates — the unique rational
/// solutions of `<alpha_s, varpi_i^vee> = delta_si` with ZERO radical
/// component (upstream rootdata.cpp:850-853, 1015-1016). Rank-bounded
/// rational elimination carries no budget knob, per the crate's recorded
/// discipline.
#[allow(dead_code)] // Stage-(d) substrate; consumed once the seed builder lands.
pub(crate) fn fundamental_coweights(
    datum: &BasedRootDatum,
) -> Result<Vec<Vec<Rational>>, StructureError> {
    let semisimple_rank = datum.semisimple_rank();
    let lattice_rank = datum.lattice_rank();
    let mut cartan = try_capacity(semisimple_rank)?;
    for root in datum.simple_roots() {
        let mut row = try_capacity(semisimple_rank)?;
        for coroot in datum.simple_coroots() {
            row.push(Rational::from(pair(root, coroot)?));
        }
        cartan.push(row);
    }
    let inverse_columns = invert_rational(&cartan)?;

    let mut coweights = try_capacity(semisimple_rank)?;
    for column in inverse_columns {
        let mut coordinates = try_capacity(lattice_rank)?;
        coordinates.resize(lattice_rank, Rational::ZERO);
        for (index, coefficient) in column.iter().enumerate() {
            for (slot, &entry) in coordinates
                .iter_mut()
                .zip(datum.simple_coroots()[index].as_slice())
            {
                *slot += coefficient.clone() * Rational::from(entry);
            }
        }
        coweights.push(coordinates);
    }
    Ok(coweights)
}

/// The nonnegative fractional part of a rational: `value - floor(value)`.
#[allow(dead_code)] // Stage-(d) substrate; consumed once the seed builder lands.
fn fractional_part(value: &Rational) -> Rational {
    value - Rational::from(value.floor())
}

/// Exact inverse of a square rational matrix by Gaussian elimination with
/// the first-nonzero pivot (deterministic; Cartan matrices of finite type
/// are invertible). Returns the inverse's COLUMNS.
#[allow(dead_code)] // Stage-(d) substrate; consumed once the seed builder lands.
fn invert_rational(matrix: &[Vec<Rational>]) -> Result<Vec<Vec<Rational>>, StructureError> {
    let size = matrix.len();
    let mut work = try_capacity(size)?;
    for (index, row) in matrix.iter().enumerate() {
        if row.len() != size {
            return Err(StructureError::RankMismatch {
                expected: size,
                actual: row.len(),
            });
        }
        let mut augmented = try_capacity(2 * size)?;
        augmented.extend(row.iter().cloned());
        for column in 0..size {
            augmented.push(if column == index {
                Rational::ONE
            } else {
                Rational::ZERO
            });
        }
        work.push(augmented);
    }
    for pivot in 0..size {
        let source = (pivot..size)
            .find(|&row| work[row][pivot] != Rational::ZERO)
            .ok_or(StructureError::SeedInvariantViolation {
                invariant: "invertible Cartan matrix",
            })?;
        work.swap(pivot, source);
        let divisor = work[pivot][pivot].clone();
        for value in &mut work[pivot] {
            *value /= &divisor;
        }
        for row in 0..size {
            if row == pivot || work[row][pivot] == Rational::ZERO {
                continue;
            }
            let factor = work[row][pivot].clone();
            for column in 0..2 * size {
                let subtrahend = &factor * &work[pivot][column];
                work[row][column] -= subtrahend;
            }
        }
    }
    let mut columns = try_capacity(size)?;
    for column in 0..size {
        let mut values = try_capacity(size)?;
        for row in &work {
            values.push(row[size + column].clone());
        }
        columns.push(values);
    }
    Ok(columns)
}

#[cfg(test)]
mod tests {
    use crate::{Coweight, Weight};

    use super::*;

    fn budget() -> IntegerLatticeBudget {
        IntegerLatticeBudget::new(16, 256, 1_000, 128)
    }

    fn rational(numerator: i32, denominator: i32) -> Rational {
        Rational::from(numerator) / Rational::from(denominator)
    }

    #[test]
    fn stable_log_elects_the_fixed_representative_for_the_swap() {
        // xi = swap on Z^2, xi + 1 = [[1,1],[1,1]].
        let matrix = vec![vec![1, 1], vec![1, 1]];
        let half = vec![rational(1, 2), rational(1, 2)];
        assert_eq!(stable_log(&half, &matrix, &budget()).unwrap(), half);
        // Integral shifts wash out through the mod-1 reductions.
        let shifted = vec![rational(3, 2), rational(1, 2)];
        assert_eq!(stable_log(&shifted, &matrix, &budget()).unwrap(), half);
        // A non-stable input violates the trailing-integrality precondition.
        let unstable = vec![rational(1, 2), Rational::ZERO];
        assert_eq!(
            stable_log(&unstable, &matrix, &budget()),
            Err(StructureError::SeedInvariantViolation {
                invariant: "stable-log integrality",
            })
        );
    }

    #[test]
    fn stable_log_of_the_negative_identity_accepts_only_integral_logs() {
        // xi = -1: the fixed lattice is trivial. Integral logs elect zero.
        let matrix = vec![vec![0, 0], vec![0, 0]];
        let integral = vec![Rational::from(1), Rational::from(-2)];
        assert_eq!(
            stable_log(&integral, &matrix, &budget()).unwrap(),
            vec![Rational::ZERO, Rational::ZERO]
        );
        // A 2-torsion log is STABLE but not congruent mod X_* to an
        // exactly-fixed vector: upstream would silently drop it (the
        // review-documented hazard); the port's precondition rejects.
        let torsion = vec![rational(1, 2), rational(1, 2)];
        assert_eq!(
            stable_log(&torsion, &matrix, &budget()),
            Err(StructureError::SeedInvariantViolation {
                invariant: "stable-log integrality",
            })
        );
    }

    #[test]
    fn fundamental_coweights_pair_as_the_identity() {
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let coweights = fundamental_coweights(&datum).unwrap();
        assert_eq!(coweights.len(), 2);
        for (index, root) in datum.simple_roots().iter().enumerate() {
            for (which, coweight) in coweights.iter().enumerate() {
                let pairing = root
                    .as_slice()
                    .iter()
                    .zip(coweight)
                    .fold(Rational::ZERO, |sum, (&a, b)| sum + Rational::from(a) * b);
                let expected = if index == which {
                    Rational::ONE
                } else {
                    Rational::ZERO
                };
                assert_eq!(pairing, expected);
            }
        }
    }

    #[test]
    fn fundamental_coweights_stay_in_the_coroot_span() {
        // Non-semisimple A1 in a rank-2 lattice: the radical component is
        // elected zero, so the coweight is half the simple coroot.
        let datum = BasedRootDatum::from_simple_data(
            2,
            vec![vec![2]],
            vec![Weight::new(vec![1, 0])],
            vec![Coweight::new(vec![2, 0])],
        )
        .unwrap();
        let coweights = fundamental_coweights(&datum).unwrap();
        assert_eq!(coweights, vec![vec![Rational::ONE, Rational::ZERO]]);
    }
}
