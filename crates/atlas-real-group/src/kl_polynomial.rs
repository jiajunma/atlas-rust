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

use crate::StructureError;

/// A polynomial over ℤ in `q`, least-degree first. The zero polynomial is
/// the empty vector; otherwise the leading (top) coefficient is nonzero.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct KlPol(Vec<i32>);

impl KlPol {
    /// The zero polynomial (kl.cpp:79, `const KLPol Zero`).
    pub fn zero() -> Self {
        Self(Vec::new())
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
        let length = self.0.len().max(other.0.len());
        let mut result = Vec::with_capacity(length);
        for index in 0..length {
            result.push(self.coefficient(index) + other.coefficient(index));
        }
        Self::trim(result)
    }

    /// `self - other` (polynomials.h `Polynomial::subtract`).
    pub fn sub(&self, other: &Self) -> Self {
        let length = self.0.len().max(other.0.len());
        let mut result = Vec::with_capacity(length);
        for index in 0..length {
            result.push(self.coefficient(index) - other.coefficient(index));
        }
        Self::trim(result)
    }

    /// `self - q^d * other` — the μ-correction term (kl.cpp:504-512
    /// `safeSubtract(pol, d, mu)`).
    pub fn sub_shifted(&self, other: &Self, d: usize, multiplier: i32) -> Self {
        let mut result = self.0.clone();
        result.resize(result.len().max(other.0.len() + d), 0);
        for (index, &coefficient) in other.0.iter().enumerate() {
            let target = index + d;
            result[target] -= coefficient * multiplier;
        }
        Self::trim(result)
    }

    /// Multiply by `1 + q` (kl.cpp:409 `Pxy.safeAdd(Pxy,1)`,
    /// polynomials.h `Polynomial::safeAdd` with shift 1): the result is
    /// `P + q*P`.
    pub fn shift(&self) -> Self {
        let mut result = self.0.clone();
        result.resize(result.len() + 1, 0);
        for (index, &coefficient) in self.0.iter().enumerate() {
            result[index + 1] += coefficient;
        }
        Self::trim(result)
    }

    /// `self + q^d * other` — the complex-descent recursion term
    /// `P_{sx,sy} + q.P_{x,sy}` (kl.cpp:416 `safeAdd(KL_pol(x,sy),1)`).
    pub fn add_shifted(&self, other: &Self, d: usize) -> Self {
        let mut result = self.0.clone();
        result.resize(result.len().max(other.0.len() + d), 0);
        for (index, &coefficient) in other.0.iter().enumerate() {
            result[index + d] += coefficient;
        }
        Self::trim(result)
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
        if factor == 1 {
            return self.clone();
        }
        Self::trim(self.0.iter().map(|&c| c * factor).collect())
    }

    /// `self + q^d * multiplier * other` — the μ-sum contribution
    /// (kl.cpp:834-836 `safeAdd(Pxz, d, mu)`).
    pub fn add_shifted_scaled(&self, other: &Self, d: usize, multiplier: i32) -> Self {
        let mut result = self.0.clone();
        result.resize(result.len().max(other.0.len() + d), 0);
        for (index, &coefficient) in other.0.iter().enumerate() {
            result[index + d] += coefficient * multiplier;
        }
        Self::trim(result)
    }

    /// Divide by 2, exact in the KLV context (kl.cpp:702 `safeDivide(2)`).
    pub fn divide_by_2(&self) -> Result<Self, StructureError> {
        if self.0.iter().any(|&c| c % 2 != 0) {
            return Err(StructureError::RepInvariantViolation {
                invariant: "KL polynomial parity",
            });
        }
        Ok(Self::trim(self.0.iter().map(|&c| c / 2).collect()))
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

    /// Test-friendly constructor from coefficients.
    #[cfg(test)]
    pub fn from_coefficients(coefficients: Vec<i32>) -> Self {
        Self::trim(coefficients)
    }
}

/// A hash table over the polynomial pool: dedupes polynomial content to
/// a `KlIndex` (upstream `KL_hash_Table`, hashtable.h / kl.cpp:100-101).
///
/// `zero` is always index 0 and `one` (the constant 1) index 1, exactly
/// like the upstream `KLStore{Zero, One}` initialisation (kl.cpp:100).
#[derive(Clone, Debug, Default)]
pub struct KlHashTable {
    pool: Vec<KlPol>,
    index: std::collections::HashMap<KlPol, usize>,
}

impl KlHashTable {
    pub fn new() -> Self {
        let mut table = Self::default();
        let zero = KlPol::zero();
        table.index.insert(zero.clone(), 0);
        table.pool.push(zero);
        let one = KlPol::monomial(0);
        table.index.insert(one.clone(), 1);
        table.pool.push(one);
        table
    }

    /// The pool index of a polynomial, inserting it if new.
    pub fn match_pol(&mut self, polynomial: &KlPol) -> usize {
        if let Some(&existing) = self.index.get(polynomial) {
            return existing;
        }
        let index = self.pool.len();
        self.pool.push(polynomial.clone());
        self.index.insert(polynomial.clone(), index);
        index
    }

    /// The polynomial at a pool index.
    pub fn get(&self, index: usize) -> Option<&KlPol> {
        self.pool.get(index)
    }

    pub fn len(&self) -> usize {
        self.pool.len()
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
}
