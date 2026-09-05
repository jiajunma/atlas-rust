//! Minimal polynomial engine for the KLV computation (gkmod/kl.cpp).
//!
//! The KLV polynomials `P_{x,y}` are polynomials over ℤ in the variable
//! `q`, with non-negative coefficients and leading coefficient 1. The
//! upstream stores them as `SafePoly<KLCoeff>` — a vector of coefficients
//! with the top coefficient always nonzero (so the zero polynomial is the
//! empty vector). This module mirrors that layout: `KlPol(Vec<i32>)`,
//! least-degree first, no trailing zeros except for the zero polynomial
//! (kl.cpp:79-83, polynomials.h).
//!
//! Only the operations the recursion and μ-correction need are provided:
//! `add`/`sub`, `shift` (multiply by `1+q`), scaling by a monomial
//! `q^d`, and evaluation at `q = -1` (kl.cpp deformation_terms loop).
//!
//! Each allocating operation has an in-place `*_assign` twin mirroring
//! upstream's `SafePoly::safeAdd`/`safeSubtract` (polynomials.h), which
//! mutate the receiver; the KLV fill loops use those to reuse the
//! accumulator buffers across the μ-correction iterations.

use crate::involution_table::MixingHasherBuilder;
use crate::StructureError;
use std::hash::{BuildHasher, Hash, Hasher};

/// A polynomial over ℤ in `q`, least-degree first. The zero polynomial is
/// the empty vector; otherwise the leading (top) coefficient is nonzero.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct KlPol(Vec<i32>);

impl KlPol {
    /// The zero polynomial (kl.cpp:79, `const KLPol Zero`).
    pub fn zero() -> Self {
        Self(Vec::new())
    }

    /// A shared zero polynomial, for borrow-based pool-lookup fallbacks.
    pub fn zero_ref() -> &'static Self {
        static ZERO: KlPol = KlPol(Vec::new());
        &ZERO
    }

    /// A shared `1` polynomial (`q^0`, pool index 1), for shifted scalar
    /// terms that would otherwise build a temporary monomial.
    pub fn one_ref() -> &'static Self {
        static ONE: std::sync::OnceLock<KlPol> = std::sync::OnceLock::new();
        ONE.get_or_init(|| KlPol::monomial(0))
    }

    /// The polynomial `q^d` (kl.cpp:83 `One` is `q^0`).
    pub fn monomial(d: usize) -> Self {
        let mut coefficients = vec![0_i32; d];
        coefficients.push(1);
        Self(coefficients)
    }

    pub fn is_zero(&self) -> bool {
        self.0.is_empty()
    }

    /// The degree of the polynomial; 0 for the zero polynomial
    /// (polynomials.h `Polynomial::degree`).
    pub fn degree(&self) -> usize {
        self.0.len().saturating_sub(1)
    }

    /// Coefficient of `q^degree`; panics if out of range.
    pub fn coefficient(&self, degree: usize) -> i32 {
        self.0.get(degree).copied().unwrap_or(0)
    }

    /// `self + other` (polynomials.h `Polynomial::add`).
    pub fn add(&self, other: &Self) -> Self {
        let mut result = self.clone();
        result.add_assign(other);
        result
    }

    /// `self += other`, reusing the receiver's buffer
    /// (polynomials.h `Polynomial::safeAdd`).
    pub fn add_assign(&mut self, other: &Self) {
        if self.0.len() < other.0.len() {
            self.0.resize(other.0.len(), 0);
        }
        for (target, &coefficient) in self.0.iter_mut().zip(other.0.iter()) {
            *target += coefficient;
        }
        self.trim_in_place();
    }

    /// `self - other` (polynomials.h `Polynomial::subtract`).
    pub fn sub(&self, other: &Self) -> Self {
        let mut result = self.clone();
        result.sub_assign(other);
        result
    }

    /// `self -= other`, reusing the receiver's buffer
    /// (polynomials.h `Polynomial::safeSubtract`).
    pub fn sub_assign(&mut self, other: &Self) {
        if self.0.len() < other.0.len() {
            self.0.resize(other.0.len(), 0);
        }
        for (target, &coefficient) in self.0.iter_mut().zip(other.0.iter()) {
            *target -= coefficient;
        }
        self.trim_in_place();
    }

    /// `self - q^d * other` — the μ-correction term (kl.cpp:504-512
    /// `safeSubtract(pol, d, mu)`).
    pub fn sub_shifted(&self, other: &Self, d: usize, multiplier: i32) -> Self {
        let mut result = self.clone();
        result.sub_shifted_assign(other, d, multiplier);
        result
    }

    /// `self -= q^d * multiplier * other`, in place (kl.cpp:504-512
    /// `safeSubtract(pol, d, mu)`).
    pub fn sub_shifted_assign(&mut self, other: &Self, d: usize, multiplier: i32) {
        let needed = other.0.len() + d;
        if self.0.len() < needed {
            self.0.resize(needed, 0);
        }
        for (index, &coefficient) in other.0.iter().enumerate() {
            self.0[index + d] -= coefficient * multiplier;
        }
        self.trim_in_place();
    }

    /// Multiply by `1 + q` (kl.cpp:409 `Pxy.safeAdd(Pxy,1)`,
    /// polynomials.h `Polynomial::safeAdd` with shift 1): the result is
    /// `P + q*P`.
    pub fn shift(&self) -> Self {
        let mut result = self.clone();
        result.shift_assign();
        result
    }

    /// `self *= 1 + q`, in place: the top-down pass keeps the
    /// not-yet-shifted coefficients intact.
    pub fn shift_assign(&mut self) {
        let len = self.0.len();
        self.0.resize(len + 1, 0);
        for index in (0..len).rev() {
            let coefficient = self.0[index];
            self.0[index + 1] += coefficient;
        }
        self.trim_in_place();
    }

    /// `self + q^d * other` — the complex-descent recursion term
    /// `P_{sx,sy} + q.P_{x,sy}` (kl.cpp:416 `safeAdd(KL_pol(x,sy),1)`).
    pub fn add_shifted(&self, other: &Self, d: usize) -> Self {
        let mut result = self.clone();
        result.add_shifted_assign(other, d);
        result
    }

    /// `self += q^d * other`, in place (polynomials.h `safeAdd(pol, d)`).
    pub fn add_shifted_assign(&mut self, other: &Self, d: usize) {
        let needed = other.0.len() + d;
        if self.0.len() < needed {
            self.0.resize(needed, 0);
        }
        for (index, &coefficient) in other.0.iter().enumerate() {
            self.0[index + d] += coefficient;
        }
        self.trim_in_place();
    }

    /// Evaluate at `q = -1`: the alternating sum of coefficients
    /// (repr.cpp:1953-1955, `eval = pol[d] - eval`).
    pub fn evaluate_at_minus_one(&self) -> i32 {
        let mut eval = 0_i32;
        for &coefficient in self.0.iter().rev() {
            eval = coefficient - eval;
        }
        eval
    }

    /// A scalar multiple of this polynomial.
    pub fn scaled(&self, factor: i32) -> Self {
        let mut result = self.clone();
        result.scale_assign(factor);
        result
    }

    /// `self *= factor`, in place.
    pub fn scale_assign(&mut self, factor: i32) {
        if factor == 1 {
            return;
        }
        for coefficient in &mut self.0 {
            *coefficient *= factor;
        }
        self.trim_in_place();
    }

    /// `self + q^d * multiplier * other` — the μ-sum contribution
    /// (kl.cpp:834-836 `safeAdd(Pxz, d, mu)`).
    pub fn add_shifted_scaled(&self, other: &Self, d: usize, multiplier: i32) -> Self {
        let mut result = self.clone();
        result.add_shifted_scaled_assign(other, d, multiplier);
        result
    }

    /// `self += q^d * multiplier * other`, in place (kl.cpp:834-836
    /// `safeAdd(Pxz, d, mu)`).
    pub fn add_shifted_scaled_assign(&mut self, other: &Self, d: usize, multiplier: i32) {
        let needed = other.0.len() + d;
        if self.0.len() < needed {
            self.0.resize(needed, 0);
        }
        for (index, &coefficient) in other.0.iter().enumerate() {
            self.0[index + d] += coefficient * multiplier;
        }
        self.trim_in_place();
    }

    /// Divide by 2, exact in the KLV context (kl.cpp:702 `safeDivide(2)`).
    pub fn divide_by_2(&self) -> Result<Self, StructureError> {
        let mut result = self.clone();
        result.divide_by_2_assign()?;
        Ok(result)
    }

    /// `self /= 2`, in place (kl.cpp:702 `safeDivide(2)`).
    pub fn divide_by_2_assign(&mut self) -> Result<(), StructureError> {
        if self.0.iter().any(|&c| c % 2 != 0) {
            return Err(StructureError::RepInvariantViolation {
                invariant: "KL polynomial parity",
            });
        }
        for coefficient in &mut self.0 {
            *coefficient /= 2;
        }
        self.trim_in_place();
        Ok(())
    }

    /// Divide by `1 + q` (kl.cpp:711 `safe_quotient_by_1_plus_q`). The
    /// input is `(1+q)P` for a polynomial `P` of degree at most the
    /// bound; synthetic division recovers `P` as the alternating partial
    /// sums of the input's coefficients, truncated to the bound.
    pub fn quotient_by_1_plus_q(&self, degree_bound: usize) -> Result<Self, StructureError> {
        let mut result = Vec::new();
        let mut accumulator = 0_i32;
        for &coefficient in self.0.iter() {
            accumulator = coefficient - accumulator;
            result.push(accumulator);
        }
        result.truncate(degree_bound + 1);
        Ok(Self::trim(result))
    }

    /// The raw coefficient vector.
    pub fn as_slice(&self) -> &[i32] {
        &self.0
    }

    /// Test-only: expose the coefficient vector for assertions.
    #[cfg(test)]
    pub fn coefficients(&self) -> &[i32] {
        &self.0
    }

    /// Trim trailing zeros; the empty vector is the zero polynomial.
    fn trim(mut coefficients: Vec<i32>) -> Self {
        while coefficients.last() == Some(&0) {
            coefficients.pop();
        }
        Self(coefficients)
    }

    /// In-place counterpart of [`Self::trim`].
    fn trim_in_place(&mut self) {
        while self.0.last() == Some(&0) {
            self.0.pop();
        }
    }

    /// Constructor from coefficients, trimming trailing zeros (the empty
    /// vector is the zero polynomial). `ext_kl` builds polynomials
    /// coefficient-by-coefficient (e.g. `extract_M`), which the shift/scale
    /// combinators cannot express.
    pub fn from_coefficients(coefficients: Vec<i32>) -> Self {
        Self::trim(coefficients)
    }
}

/// A hash table over the polynomial pool: dedupes polynomial content to
/// a `KlIndex` (upstream `KL_hash_Table`, hashtable.h / kl.cpp:100-101).
///
/// `zero` is always index 0 and `one` (the constant 1) index 1, exactly
/// like the upstream `KLStore{Zero, One}` initialisation (kl.cpp:100).
/// The index is a lean open-addressing table of pool indices (slot value =
/// pool index + 1, 0 = empty): the pool is the SINGLE owner of polynomial
/// storage, so a miss clones the polynomial once (the previous std
/// HashMap<KlPol, usize> kept a second copy of every key) and a hit is one
/// hash plus one pool compare. Hashing affects only bucket layout, never
/// interning order (`match_pol` returns the existing index on a hit and
/// appends on a miss), so pool indices — and the raw indices `filekl`
/// serializes — are layout-independent.
#[derive(Clone, Debug)]
pub struct KlHashTable {
    pool: Vec<KlPol>,
    /// Power-of-two open-addressing slots; value = pool index + 1.
    slots: Vec<u32>,
    mask: usize,
}

impl Default for KlHashTable {
    fn default() -> Self {
        Self::new()
    }
}

impl KlHashTable {
    pub fn new() -> Self {
        let mut table = KlHashTable {
            pool: Vec::new(),
            slots: vec![0; 16],
            mask: 15,
        };
        table.match_pol(&KlPol::zero());
        table.match_pol(&KlPol::monomial(0));
        table
    }

    /// The pool index of a polynomial, inserting it if new.
    pub fn match_pol(&mut self, polynomial: &KlPol) -> usize {
        let mut hasher = MixingHasherBuilder::default().build_hasher();
        polynomial.hash(&mut hasher);
        let hash = hasher.finish();
        let mut slot = (hash as usize) & self.mask;
        loop {
            let entry = self.slots[slot];
            if entry == 0 {
                let index = self.pool.len();
                debug_assert!(index < u32::MAX as usize);
                self.pool.push(polynomial.clone());
                self.slots[slot] = (index + 1) as u32;
                // Grow at 3/4 load, rehashing from the pool (the only
                // storage), so no key copies are ever reshuffled.
                if (self.pool.len() + 1) * 4 > self.slots.len() * 3 {
                    self.grow();
                }
                return index;
            }
            let index = usize::try_from(entry - 1).expect("slot fits usize");
            if self.pool[index] == *polynomial {
                return index;
            }
            slot = (slot + 1) & self.mask;
        }
    }

    fn grow(&mut self) {
        let new_len = self.slots.len() * 2;
        let mut slots = vec![0_u32; new_len];
        let mask = new_len - 1;
        for (index, polynomial) in self.pool.iter().enumerate() {
            // A fresh hasher per polynomial: Hasher state accumulates.
            let mut hasher = MixingHasherBuilder::default().build_hasher();
            polynomial.hash(&mut hasher);
            let mut slot = (hasher.finish() as usize) & mask;
            while slots[slot] != 0 {
                slot = (slot + 1) & mask;
            }
            slots[slot] = (index + 1) as u32;
        }
        self.slots = slots;
        self.mask = mask;
    }

    /// The polynomial at a pool index.
    pub fn get(&self, index: usize) -> Option<&KlPol> {
        self.pool.get(index)
    }

    pub fn len(&self) -> usize {
        self.pool.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pool.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_and_one_are_the_first_pool_entries() {
        let table = KlHashTable::new();
        assert_eq!(table.len(), 2);
        assert!(table.get(0).expect("zero").is_zero());
        assert_eq!(table.get(1).expect("one").as_slice(), &[1]);
    }

    #[test]
    fn match_pol_interns_in_first_seen_order_across_growth() {
        let mut table = KlHashTable::new();
        assert_eq!(table.match_pol(&KlPol::zero()), 0);
        assert_eq!(table.match_pol(&KlPol::monomial(0)), 1);
        // Enough distinct polynomials to force several slot-table growth
        // rounds (16 initial slots, 3/4 load factor).
        let polys: Vec<KlPol> = (1..40)
            .map(|d| {
                KlPol::from_coefficients(vec![
                    d, d + 1, d + 2, d + 3, d + 4, d + 5, d + 6, d + 7, d + 8, d + 9,
                ])
            })
            .collect();
        for (offset, polynomial) in polys.iter().enumerate() {
            assert_eq!(table.match_pol(polynomial), offset + 2);
        }
        // Re-queries after growth return the original indices.
        for (offset, polynomial) in polys.iter().enumerate() {
            assert_eq!(table.match_pol(polynomial), offset + 2);
        }
        assert_eq!(table.len(), 41);
    }

    #[test]
    fn shift_multiplies_by_one_plus_q() {
        let p = KlPol::from_coefficients(vec![1, 2]);
        // (1 + 2q)(1 + q) = 1 + 3q + 2q²
        assert_eq!(p.shift().as_slice(), &[1, 3, 2]);
    }

    #[test]
    fn evaluate_at_minus_one_alternates() {
        // 1 - q + 2q² at q=-1: 1 + 1 + 2 = 4
        let p = KlPol::from_coefficients(vec![1, -1, 2]);
        assert_eq!(p.evaluate_at_minus_one(), 4);
        // (1+q)(1+q) = 1+2q+q² at q=-1: 0
        let square = KlPol::from_coefficients(vec![1, 1]).shift();
        assert_eq!(square.evaluate_at_minus_one(), 0);
    }

    #[test]
    fn sub_shifted_applies_monomial_multiple() {
        // (1 + q) - q^1 * (1) = 1 + q - q = 1
        let p = KlPol::from_coefficients(vec![1, 1]);
        assert_eq!(p.sub_shifted(&KlPol::monomial(0), 1, 1).as_slice(), &[1]);
    }

    #[test]
    fn in_place_ops_match_the_allocating_variants() {
        let samples = [
            KlPol::zero(),
            KlPol::monomial(0),
            KlPol::monomial(3),
            KlPol::from_coefficients(vec![1, 1]),
            KlPol::from_coefficients(vec![3, -2, 0, 5]),
            KlPol::from_coefficients(vec![2, 4, 6]),
        ];
        for a in &samples {
            for b in &samples {
                let mut actual = a.clone();
                actual.add_assign(b);
                assert_eq!(actual, a.add(b), "{a:?} += {b:?}");
                let mut actual = a.clone();
                actual.sub_assign(b);
                assert_eq!(actual, a.sub(b), "{a:?} -= {b:?}");
                for &d in &[0_usize, 1, 3] {
                    let mut actual = a.clone();
                    actual.add_shifted_assign(b, d);
                    assert_eq!(actual, a.add_shifted(b, d), "{a:?} += q^{d}*{b:?}");
                    for &m in &[1, -2] {
                        let mut actual = a.clone();
                        actual.sub_shifted_assign(b, d, m);
                        assert_eq!(actual, a.sub_shifted(b, d, m), "{a:?} -= {m}q^{d}*{b:?}");
                        let mut actual = a.clone();
                        actual.add_shifted_scaled_assign(b, d, m);
                        assert_eq!(
                            actual,
                            a.add_shifted_scaled(b, d, m),
                            "{a:?} += {m}q^{d}*{b:?}"
                        );
                    }
                }
            }
            let mut actual = a.clone();
            actual.shift_assign();
            assert_eq!(actual, a.shift(), "{a:?} *= 1+q");
            for &factor in &[0, 1, -3] {
                let mut actual = a.clone();
                actual.scale_assign(factor);
                assert_eq!(actual, a.scaled(factor), "{a:?} *= {factor}");
            }
        }
    }

    #[test]
    fn divide_by_2_assign_matches_the_allocating_variant() {
        let p = KlPol::from_coefficients(vec![2, 4, 6]);
        let mut actual = p.clone();
        actual.divide_by_2_assign().unwrap();
        assert_eq!(actual, p.divide_by_2().unwrap());
        let mut odd = KlPol::from_coefficients(vec![1, 2]);
        assert!(odd.divide_by_2_assign().is_err());
    }
}
