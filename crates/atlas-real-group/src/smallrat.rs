//! Machine-word scalar for the small dense Gauss-Jordan sweeps in
//! [`crate::alcove`]: i128 numerator/denominator, eagerly reduced with a
//! positive denominator (the same normalization the malachite operators
//! apply). Every operation is exact; `None` reports the first i128
//! overflow so the caller can restart the identical sweep on
//! [`Rational`] before any output is produced. Mirrors the `SmallRat`
//! fast path of atlas-core's `ratfast` module (that crate depends on this
//! one, so the helper is duplicated here rather than shared).

use malachite::{Integer, Rational};

/// Scalar operations shared by the alcove Gauss-Jordan solvers, so each
/// sweep can run on machine-word [`SmallRat`] first and restart on
/// [`Rational`] when an intermediate leaves the i128 window (identical
/// arithmetic — both paths normalize eagerly).
pub(crate) trait GjScalar: Clone {
    fn from_i64(value: i64) -> Self;
    fn is_nonzero(&self) -> bool;
    /// `*self /= pivot`; `Err(())` only from `SmallRat` overflow.
    fn div_assign(&mut self, pivot: &Self) -> Result<(), ()>;
    /// `*self -= pivot_entry * factor`; `Err(())` only from `SmallRat`
    /// overflow.
    fn mul_sub_assign(&mut self, pivot_entry: &Self, factor: &Self) -> Result<(), ()>;
}

#[derive(Clone, Copy)]
pub(crate) struct SmallRat {
    num: i128,
    // Invariant: den > 0 and gcd(|num|, den) == 1.
    den: i128,
}

impl SmallRat {
    /// The canonical rational equal to this value.
    pub(crate) fn to_rational(&self) -> Rational {
        Rational::from_integers(Integer::from(self.num), Integer::from(self.den))
    }
}

impl GjScalar for SmallRat {
    fn from_i64(value: i64) -> Self {
        Self {
            num: i128::from(value),
            den: 1,
        }
    }

    fn is_nonzero(&self) -> bool {
        self.num != 0
    }

    fn div_assign(&mut self, pivot: &Self) -> Result<(), ()> {
        // i64 fast path: every operand already fits an i64 in the common
        // case, so the products stay single-instruction; any overflow falls
        // through to the i128 computation, keeping the overflow contract
        // identical (only the first i128 overflow reports Err).
        if let (Ok(num_a), Ok(den_a), Ok(num_b), Ok(den_b)) = (
            i64::try_from(self.num),
            i64::try_from(self.den),
            i64::try_from(pivot.num),
            i64::try_from(pivot.den),
        ) {
            if let (Some(num), Some(den)) = (
                num_a.checked_mul(den_b),
                den_a.checked_mul(num_b),
            ) {
                if let Some((num, den)) = reduce_i64(num, den) {
                    *self = Self { num, den };
                    return Ok(());
                }
            }
        }
        let num = self.num.checked_mul(pivot.den).ok_or(())?;
        let den = self.den.checked_mul(pivot.num).ok_or(())?;
        let (num, den) = reduce_i128(num, den).ok_or(())?;
        *self = Self { num, den };
        Ok(())
    }

    fn mul_sub_assign(&mut self, pivot_entry: &Self, factor: &Self) -> Result<(), ()> {
        if let (Ok(sn), Ok(sd), Ok(pn), Ok(pd), Ok(fnum), Ok(fd)) = (
            i64::try_from(self.num),
            i64::try_from(self.den),
            i64::try_from(pivot_entry.num),
            i64::try_from(pivot_entry.den),
            i64::try_from(factor.num),
            i64::try_from(factor.den),
        ) {
            let attempt = || {
                let product_num = pn.checked_mul(fnum)?;
                let product_den = pd.checked_mul(fd)?;
                let num = sn
                    .checked_mul(product_den)?
                    .checked_sub(product_num.checked_mul(sd)?)?;
                let den = sd.checked_mul(product_den)?;
                reduce_i64(num, den)
            };
            if let Some((num, den)) = attempt() {
                *self = Self { num, den };
                return Ok(());
            }
        }
        let product_num = pivot_entry.num.checked_mul(factor.num).ok_or(())?;
        let product_den = pivot_entry.den.checked_mul(factor.den).ok_or(())?;
        let num = self
            .num
            .checked_mul(product_den)
            .and_then(|scaled| scaled.checked_sub(product_num.checked_mul(self.den)?))
            .ok_or(())?;
        let den = self.den.checked_mul(product_den).ok_or(())?;
        let (num, den) = reduce_i128(num, den).ok_or(())?;
        *self = Self { num, den };
        Ok(())
    }
}

impl GjScalar for Rational {
    fn from_i64(value: i64) -> Self {
        Rational::from(value)
    }

    fn is_nonzero(&self) -> bool {
        self != &Rational::from(0)
    }

    fn div_assign(&mut self, pivot: &Self) -> Result<(), ()> {
        *self /= pivot;
        Ok(())
    }

    fn mul_sub_assign(&mut self, pivot_entry: &Self, factor: &Self) -> Result<(), ()> {
        *self -= pivot_entry.clone() * factor;
        Ok(())
    }
}

/// Normalize the pivot row by the pivot and clear the pivot column above
/// and below — the shared body of the historical `Rational` sweeps, in
/// the same operation order.
pub(crate) fn gj_normalize_and_clear<T: GjScalar>(
    matrix: &mut [Vec<T>],
    rows: usize,
    pivot_row: usize,
    column: usize,
) -> Result<(), ()> {
    let pivot = matrix[pivot_row][column].clone();
    for entry in &mut matrix[pivot_row] {
        entry.div_assign(&pivot)?;
    }
    for row in 0..rows {
        if row == pivot_row || !matrix[row][column].is_nonzero() {
            continue;
        }
        let factor = matrix[row][column].clone();
        let (pivot_line, target) = if row < pivot_row {
            let (head, tail) = matrix.split_at_mut(pivot_row);
            (&tail[0], &mut head[row])
        } else {
            let (head, tail) = matrix.split_at_mut(row);
            (&head[pivot_row], &mut tail[0])
        };
        for (target_entry, pivot_entry) in target.iter_mut().zip(pivot_line.iter()) {
            target_entry.mul_sub_assign(pivot_entry, &factor)?;
        }
    }
    Ok(())
}

/// Build the scalar matrix the historical bodies built entry-by-entry.
pub(crate) fn gj_scalars<T: GjScalar>(ints: &[Vec<i64>]) -> Vec<Vec<T>> {
    ints.iter()
        .map(|row| row.iter().map(|&entry| T::from_i64(entry)).collect())
        .collect()
}

/// Reduce `num/den` (`den != 0`) to lowest terms with a positive
/// denominator; `None` only on i128 sign-flip overflow.
fn reduce_i128(num: i128, den: i128) -> Option<(i128, i128)> {
    debug_assert_ne!(den, 0);
    let (num, den) = if den < 0 {
        (num.checked_neg()?, den.checked_neg()?)
    } else {
        (num, den)
    };
    if num == 0 {
        return Some((0, 1));
    }
    let anum = num.unsigned_abs();
    let aden = den as u128;
    // u64 fast path: the overwhelming majority of sweep entries stay
    // small, and 128-bit modulo/division lower to __divti3 calls while the
    // 64-bit forms are single hardware instructions (perf-unitary-3683299:
    // gj_normalize_and_clear 9.0% self including the inlined reduction).
    if anum <= u128::from(u64::MAX) && aden <= u128::from(u64::MAX) {
        let (anum, aden) = (anum as u64, aden as u64);
        let divisor = gcd_u64(anum, aden);
        let sign = i128::from(num.signum());
        return Some((
            i128::from(anum / divisor) * sign,
            i128::from(aden / divisor),
        ));
    }
    // den > 0 here, so the gcd is positive and divides num; it fits an
    // i128 because it does not exceed den.
    let divisor = gcd_u128(anum, aden) as i128;
    Some((num / divisor, den / divisor))
}

/// i64 twin of [`reduce_i128`] for the fast paths; `None` only on an
/// i64 sign-flip overflow, which the callers treat as "retry in i128".
fn reduce_i64(num: i64, den: i64) -> Option<(i128, i128)> {
    debug_assert_ne!(den, 0);
    let (num, den) = if den < 0 {
        (num.checked_neg()?, den.checked_neg()?)
    } else {
        (num, den)
    };
    if num == 0 {
        return Some((0, 1));
    }
    let anum = num.unsigned_abs();
    let aden = den as u64;
    let divisor = gcd_u64(anum, aden);
    let sign = i128::from(num.signum());
    Some((i128::from(anum / divisor) * sign, i128::from(aden / divisor)))
}

fn gcd_u64(mut left: u64, mut right: u64) -> u64 {
    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }
    left
}

fn gcd_u128(mut left: u128, mut right: u128) -> u128 {
    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }
    left
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rational(numerator: i64, denominator: i64) -> Rational {
        Rational::from_signeds(numerator, denominator)
    }

    /// A randomized-in-spirit sweep: run the same elimination on SmallRat
    /// and Rational and require identical reduced matrices.
    #[test]
    fn small_rat_sweep_matches_rational_sweep() {
        let ints: Vec<Vec<i64>> = vec![
            vec![2, 1, -1, 8],
            vec![-3, -1, 2, -11],
            vec![-2, 1, 2, -3],
        ];
        fn sweep<T: GjScalar>(matrix: &mut [Vec<T>]) {
            let mut pivot_row = 0;
            for column in 0..3 {
                let found = (pivot_row..3)
                    .find(|&row| matrix[row][column].is_nonzero())
                    .unwrap();
                matrix.swap(pivot_row, found);
                gj_normalize_and_clear(matrix, 3, pivot_row, column).unwrap();
                pivot_row += 1;
            }
        }
        let mut small = gj_scalars::<SmallRat>(&ints);
        sweep(&mut small);
        let mut big = gj_scalars::<Rational>(&ints);
        sweep(&mut big);
        for (small_row, big_row) in small.iter().zip(&big) {
            for (small, big) in small_row.iter().zip(big_row) {
                assert_eq!(&small.to_rational(), big);
            }
        }
    }

    #[test]
    fn reduce_fast_path_matches_wide_path() {
        // Values below and above the u64 boundary must reduce identically.
        let cases: [(i128, i128); 6] = [
            (6, 9),
            (-6, 9),
            (6, -9),
            (i64::MAX as i128 * 3, i64::MAX as i128 * 6),
            (-(i64::MAX as i128) * 5, i64::MAX as i128 * 10),
            (1 << 100, 3 << 100),
        ];
        for (num, den) in cases {
            let (reduced_num, reduced_den) = reduce_i128(num, den).unwrap();
            assert_eq!(
                Rational::from_integers(
                    Integer::from(reduced_num),
                    Integer::from(reduced_den)
                ),
                Rational::from_integers(Integer::from(num), Integer::from(den)),
                "{num}/{den}"
            );
            assert!(reduced_den > 0);
        }
    }

    #[test]
    fn narrow_ops_match_rational_ops() {
        // The i64 fast paths in div_assign/mul_sub_assign must produce the
        // same reduced values as the Rational operators, including negative
        // pivot numerators (which flip the intermediate denominator sign).
        let cases: [(i64, i64, i64, i64); 5] = [
            (3, 7, -5, 11),
            (-2, 3, 4, -9),
            (0, 1, 6, 35),
            (i64::MAX - 1, 1, 1, 1),
            (12, 18, 9, 6),
        ];
        for (a_num, a_den, b_num, b_den) in cases {
            let mut narrow = SmallRat {
                num: i128::from(a_num),
                den: i128::from(a_den),
            };
            let pivot = SmallRat {
                num: i128::from(b_num),
                den: i128::from(b_den),
            };
            let mut wide = rational(a_num, a_den);
            let wide_pivot = rational(b_num, b_den);
            narrow.div_assign(&pivot).unwrap();
            wide /= wide_pivot.clone();
            assert_eq!(narrow.to_rational(), wide, "div {a_num}/{a_den} by {b_num}/{b_den}");

            let mut narrow = SmallRat {
                num: i128::from(a_num),
                den: i128::from(a_den),
            };
            let mut wide = rational(a_num, a_den);
            narrow.mul_sub_assign(&pivot, &pivot).unwrap();
            wide -= wide_pivot.clone() * wide_pivot;
            assert_eq!(
                narrow.to_rational(),
                wide,
                "mul_sub {a_num}/{a_den} by {b_num}/{b_den}^2"
            );
        }
    }

    #[test]
    fn small_rat_reports_overflow_instead_of_wrapping() {
        // den products i64::MAX^2 still fit an i128; i64::MAX^3 does not.
        let mut value = SmallRat {
            num: 1,
            den: i128::from(i64::MAX),
        };
        let huge = SmallRat {
            num: 1,
            den: i128::from(i64::MAX),
        };
        assert!(value.mul_sub_assign(&huge, &huge).is_err());
    }
}
