//! Machine-word fast paths for malachite `Rational` arithmetic.
//!
//! E6-scale associated-variety workloads spend a large share of their
//! runtime in malachite rational arithmetic on machine-sized operands (gdb
//! sampling, jobs 3680277/3680278). Following the `bounded_linear_combination`
//! precedent (460370e), when the numerators and denominators of every
//! operand fit an i64 the helpers below compute in i128 with checked
//! arithmetic and rebuild the result through `Rational::from_signeds`, which
//! reduces to lowest terms with a positive denominator exactly like the
//! generic malachite operators — so the fast path yields the SAME normalized
//! value, and any overflow (or a reduced result beyond the i64 window)
//! yields `None`, sending the caller back to the generic path.
//!
//! On x86_64 every u128 `/` or `%` is a software libcall while u64 division
//! is a single hardware instruction, and the denominators in these workloads
//! almost always fit a u64 — so the gcd and reduction helpers take a u64
//! fast path whenever both operands fit, falling back to u128 otherwise.

use std::cmp::Ordering;

use malachite::{Integer as BigInt, Rational as BigRational};

/// The `(numerator, denominator)` of a rational when both fit an i64.
/// Malachite rationals are stored normalized, so the denominator is always
/// positive and the fraction is in lowest terms. The sign is stored
/// separately from the (non-negative) numerator — reattach it here.
pub(crate) fn small_parts(value: &BigRational) -> Option<(i64, i64)> {
    // u64, not i64: the magnitude of i64::MIN is 2^63, which overflows i64
    // but must still take the fast path (negated, it fits again).
    let magnitude = u64::try_from(value.numerator_ref()).ok()?;
    let numerator = if value < &BigRational::from(0) {
        i64::try_from(-i128::from(magnitude)).ok()?
    } else {
        i64::try_from(magnitude).ok()?
    };
    Some((numerator, i64::try_from(value.denominator_ref()).ok()?))
}

fn gcd_u64(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

fn gcd_u128(mut a: u128, mut b: u128) -> u128 {
    // Hardware u64 division avoids the 128-bit division libcall.
    if a <= u128::from(u64::MAX) && b <= u128::from(u64::MAX) {
        return u128::from(gcd_u64(a as u64, b as u64));
    }
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

/// The canonical rational equal to `num/den` (`den != 0`), or `None` when
/// the REDUCED numerator or denominator outgrows an i64. Reduction and sign
/// placement are `from_signeds`' own, identical to the normalization every
/// malachite rational operator applies.
fn from_i128_parts(num: i128, den: i128) -> Option<BigRational> {
    debug_assert_ne!(den, 0);
    let (num, den) = if den < 0 {
        (num.checked_neg()?, den.checked_neg()?)
    } else {
        (num, den)
    };
    // den > 0 here, so the casts are lossless; the gcd divides num.
    if den <= i128::from(u64::MAX) && num.unsigned_abs() <= u128::from(u64::MAX) {
        // Hardware u64 division avoids the 128-bit division libcall; the
        // guard above makes every cast lossless.
        let divisor = gcd_u64(num.unsigned_abs() as u64, den as u64);
        let magnitude = i128::from(num.unsigned_abs() as u64 / divisor);
        let num = i64::try_from(if num < 0 { -magnitude } else { magnitude }).ok()?;
        let den = i64::try_from(i128::from(den as u64 / divisor)).ok()?;
        return Some(BigRational::from_signeds(num, den));
    }
    let divisor = gcd_u128(num.unsigned_abs(), den as u128) as i128;
    let num = i64::try_from(num / divisor).ok()?;
    let den = i64::try_from(den / divisor).ok()?;
    Some(BigRational::from_signeds(num, den))
}

/// `first + second` on machine-sized operands; `None` on any overflow.
pub(crate) fn add(first: &BigRational, second: &BigRational) -> Option<BigRational> {
    let (left_num, left_den) = small_parts(first)?;
    let (right_num, right_den) = small_parts(second)?;
    let num = (left_num as i128)
        .checked_mul(right_den as i128)?
        .checked_add((right_num as i128).checked_mul(left_den as i128)?)?;
    // Each factor fits an i64, so the product fits an i128 with room.
    let den = (left_den as i128) * (right_den as i128);
    from_i128_parts(num, den)
}

/// `first - second` on machine-sized operands; `None` on any overflow.
pub(crate) fn sub(first: &BigRational, second: &BigRational) -> Option<BigRational> {
    let (left_num, left_den) = small_parts(first)?;
    let (right_num, right_den) = small_parts(second)?;
    let num = (left_num as i128)
        .checked_mul(right_den as i128)?
        .checked_sub((right_num as i128).checked_mul(left_den as i128)?)?;
    let den = (left_den as i128) * (right_den as i128);
    from_i128_parts(num, den)
}

/// `first * second` on machine-sized operands; `None` when the reduced
/// product outgrows an i64 (the i128 products themselves cannot overflow:
/// |i64|^2 < 2^126).
pub(crate) fn mul(first: &BigRational, second: &BigRational) -> Option<BigRational> {
    let (left_num, left_den) = small_parts(first)?;
    let (right_num, right_den) = small_parts(second)?;
    let num = (left_num as i128) * (right_num as i128);
    let den = (left_den as i128) * (right_den as i128);
    from_i128_parts(num, den)
}

/// `first / second` on machine-sized operands; the caller rejects a zero
/// divisor. `None` when the reduced quotient outgrows an i64.
pub(crate) fn div(first: &BigRational, second: &BigRational) -> Option<BigRational> {
    let (left_num, left_den) = small_parts(first)?;
    let (right_num, right_den) = small_parts(second)?;
    if right_num == 0 {
        return None;
    }
    let num = (left_num as i128) * (right_den as i128);
    let den = (left_den as i128) * (right_num as i128);
    from_i128_parts(num, den)
}

/// `value + integer` on machine-sized operands; `None` on any overflow.
pub(crate) fn add_int(value: &BigRational, integer: &BigInt) -> Option<BigRational> {
    let (num, den) = small_parts(value)?;
    let integer = i64::try_from(integer).ok()?;
    let num = (num as i128).checked_add((integer as i128) * (den as i128))?;
    from_i128_parts(num, den as i128)
}

/// `value - integer` on machine-sized operands; `None` on any overflow.
pub(crate) fn sub_int(value: &BigRational, integer: &BigInt) -> Option<BigRational> {
    let (num, den) = small_parts(value)?;
    let integer = i64::try_from(integer).ok()?;
    let num = (num as i128).checked_sub((integer as i128) * (den as i128))?;
    from_i128_parts(num, den as i128)
}

/// `value * integer` on machine-sized operands; `None` when the reduced
/// product outgrows an i64.
pub(crate) fn mul_int(value: &BigRational, integer: &BigInt) -> Option<BigRational> {
    let (num, den) = small_parts(value)?;
    let integer = i64::try_from(integer).ok()?;
    from_i128_parts((num as i128) * (integer as i128), den as i128)
}

/// `value / integer` on machine-sized operands (`integer != 0`); `None`
/// when the reduced quotient outgrows an i64.
pub(crate) fn div_int(value: &BigRational, integer: &BigInt) -> Option<BigRational> {
    let (num, den) = small_parts(value)?;
    let integer = i64::try_from(integer).ok()?;
    if integer == 0 {
        return None;
    }
    from_i128_parts(num as i128, (den as i128) * (integer as i128))
}

/// The ordering of two machine-sized rationals: with positive denominators
/// the cross products order exactly like the values, and each fits an i128.
pub(crate) fn cmp(first: &BigRational, second: &BigRational) -> Option<Ordering> {
    let (left_num, left_den) = small_parts(first)?;
    let (right_num, right_den) = small_parts(second)?;
    Some(
        ((left_num as i128) * (right_den as i128)).cmp(&((right_num as i128) * (left_den as i128))),
    )
}

/// `value * numerator / denominator` (`denominator != 0`) on a
/// machine-sized value; `None` when the reduced result outgrows an i64.
pub(crate) fn scaled(
    value: &BigRational,
    numerator: i64,
    denominator: i64,
) -> Option<BigRational> {
    let (num, den) = small_parts(value)?;
    let num = (num as i128) * (numerator as i128);
    let den = (den as i128) * (denominator as i128);
    if den == 0 {
        return None;
    }
    from_i128_parts(num, den)
}

/// An exact sum of rational terms, reduced to lowest terms at every step to
/// keep intermediates minimal. Any i128 overflow invalidates the whole sum
/// (`None`), so the caller can restart on the generic path.
pub(crate) struct FastSum {
    num: i128,
    // Invariant: den > 0 and gcd(|num|, den) == 1.
    den: i128,
}

impl FastSum {
    pub(crate) fn zero() -> Self {
        Self { num: 0, den: 1 }
    }

    /// Add the term `num/den` (`den != 0`) to the running sum.
    pub(crate) fn add_term(&mut self, num: i128, den: i128) -> Option<()> {
        let (num, den) = if den < 0 {
            (num.checked_neg()?, den.checked_neg()?)
        } else {
            (num, den)
        };
        let (left, right) = if self.den >= 0
            && den >= 0
            && self.den <= i128::from(u64::MAX)
            && den <= i128::from(u64::MAX)
        {
            // Hardware u64 division avoids the 128-bit division libcall; the
            // guard makes the casts lossless and gcd_u64 of two positive
            // denominators is nonzero.
            let divisor = gcd_u64(self.den as u64, den as u64);
            (
                i128::from(self.den as u64 / divisor),
                i128::from(den as u64 / divisor),
            )
        } else {
            let divisor = gcd_u128(self.den as u128, den as u128) as i128;
            (self.den / divisor, den / divisor)
        };
        let num = self
            .num
            .checked_mul(right)?
            .checked_add(num.checked_mul(left)?)?;
        let den = left.checked_mul(den)?;
        let reduced = gcd_u128(num.unsigned_abs(), den as u128) as i128;
        self.num = num / reduced;
        self.den = den / reduced;
        Some(())
    }

    /// The canonical sum, or `None` when its reduced parts outgrow an i64.
    pub(crate) fn finish(&self) -> Option<BigRational> {
        from_i128_parts(self.num, self.den)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rational(numerator: i64, denominator: i64) -> BigRational {
        BigRational::from_signeds(numerator, denominator)
    }

    /// The fast path must equal the generic result whenever it returns a
    /// value; `None` is legitimate only when the reduced result no longer
    /// fits the i64 window (the caller then takes the generic path).
    fn check_against_generic(fast: Option<BigRational>, generic: BigRational) {
        match fast {
            Some(value) => assert_eq!(value, generic),
            None => assert!(
                small_parts(&generic).is_none(),
                "unexpected fallback: {generic} fits the i64 window"
            ),
        }
    }

    #[test]
    fn fast_binops_match_generic_arithmetic() {
        let pairs: &[((i64, i64), (i64, i64))] = &[
            ((1, 2), (1, 3)),
            ((-5, 6), (7, 4)),
            ((0, 1), (-3, 7)),
            ((i64::MAX, 2), (1, 2)),
            // i64::MIN itself is in the window; subtracting from it is not.
            ((i64::MIN, 1), (1, 1)),
            ((6, 9), (4, 10)),
        ];
        for &((an, ad), (bn, bd)) in pairs {
            let a = rational(an, ad);
            let b = rational(bn, bd);
            check_against_generic(add(&a, &b), &a + &b);
            check_against_generic(sub(&a, &b), &a - &b);
            check_against_generic(mul(&a, &b), &a * &b);
            if b != 0 {
                check_against_generic(div(&a, &b), &a / &b);
            }
            assert_eq!(cmp(&a, &b), Some(a.cmp(&b)));
        }
    }

    #[test]
    fn fast_int_ops_match_generic_arithmetic() {
        let cases: &[((i64, i64), i64)] = &[
            ((1, 2), 3),
            ((-7, 5), -4),
            ((i64::MAX, 2), 0),
            ((i64::MIN, 1), 1),
        ];
        for &((an, ad), k) in cases {
            let a = rational(an, ad);
            let integer = BigInt::from(k);
            let as_rational = BigRational::from(integer.clone());
            check_against_generic(add_int(&a, &integer), &a + &as_rational);
            check_against_generic(sub_int(&a, &integer), &a - &as_rational);
            check_against_generic(mul_int(&a, &integer), &a * &as_rational);
            if k != 0 {
                check_against_generic(div_int(&a, &integer), &a / &as_rational);
            }
        }
    }

    #[test]
    fn results_beyond_the_i64_window_fall_back() {
        // 3 * i64::MAX does not fit an i64 even after reduction.
        assert_eq!(mul(&rational(i64::MAX, 1), &rational(3, 1)), None);
        // i64::MAX + i64::MAX reduces to a numerator of 2^64 - 2.
        assert_eq!(add(&rational(i64::MAX, 1), &rational(i64::MAX, 1)), None);
        // A divisor of zero is rejected, never a panic.
        assert_eq!(div(&rational(1, 2), &rational(0, 1)), None);
        assert_eq!(div_int(&rational(1, 2), &BigInt::from(0)), None);
    }

    #[test]
    fn fast_sum_matches_generic_arithmetic() {
        let terms: &[(i64, i64)] = &[(1, 2), (1, 3), (5, 6), (-7, 4), (0, 1)];
        let mut sum = FastSum::zero();
        let mut generic = BigRational::from(0u32);
        for &(num, den) in terms {
            sum.add_term(i128::from(num), i128::from(den)).unwrap();
            generic += rational(num, den);
        }
        assert_eq!(sum.finish().as_ref(), Some(&generic));
        assert_eq!(FastSum::zero().finish().as_ref(), Some(&BigRational::from(0u32)));
    }

    #[test]
    fn fast_sum_beyond_u64_denominators_falls_back() {
        // Coprime denominators just below i64::MAX: after the second term
        // the running denominator p1*p2 exceeds u64::MAX, forcing the u128
        // fallback paths while the final sum reduces back into the i64
        // window: 1/p1 + 1/p2 - 1/p1 + 2/p2 = 3/p2 (3 does not divide p2).
        let p1 = i64::MAX; // 2^63 - 1
        let p2 = 9223372036854775783; // 2^63 - 25
        let terms: &[(i64, i64)] = &[(1, p1), (1, p2), (-1, p1), (2, p2)];
        let mut sum = FastSum::zero();
        let mut generic = BigRational::from(0u32);
        for &(num, den) in terms {
            sum.add_term(i128::from(num), i128::from(den)).unwrap();
            generic += rational(num, den);
        }
        assert_eq!(sum.finish().as_ref(), Some(&generic));
        assert_eq!(sum.finish().as_ref(), Some(&rational(3, p2)));
    }

    #[test]
    fn wide_products_reduce_back_into_the_i64_window() {
        // The denominator product p1*p2 exceeds u64::MAX, so from_i128_parts
        // takes its u128 fallback, yet cancellation brings the reduced
        // result 6/p2 back into the i64 window.
        let p1 = i64::MAX; // 2^63 - 1
        let p2 = 9223372036854775783; // 2^63 - 25
        let a = rational(6, p1);
        let b = rational(p1, p2);
        assert_eq!(mul(&a, &b).as_ref(), Some(&(&a * &b)));
        assert_eq!(mul(&a, &b).as_ref(), Some(&rational(6, p2)));
        // Same wide reduction reached through div: (6/p1) / (p2/p1).
        let c = rational(p2, p1);
        assert_eq!(div(&a, &c).as_ref(), Some(&(&a / &c)));
        assert_eq!(div(&a, &c).as_ref(), Some(&rational(6, p2)));
    }
}
