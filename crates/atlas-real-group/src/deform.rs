//! Twisted and block deformation drivers (gkmod/repr.cpp):
//! [`twisted_deformation_terms`] (repr.cpp:2426-2520), the twisted KL sums
//! at `s` (repr.cpp:2304-2350 free function and repr.cpp:2371-2423 the
//! `Rep_table` variant), [`block_deformation_to_height`]
//! (repr.cpp:2027-2124), and the recursive [`twisted_deformation`]
//! (repr.cpp:2552-2653).
//!
//! Simplifications, in the shape of the frozen `domain/deform` contract
//! that [`RepContext::deformation_terms`] already documents:
//!
//! - The full block (upstream `lookup_full_block`) carries the trivial
//!   block modifier; `lambda_rho` is supplied once by the
//!   caller instead of per block element from the upstream
//!   `StandardReprMod` pool (`common_block::sr`, blocks.cpp:1260-1264).
//!   Per-element `lambda_rho` genuinely varies across a full block (the
//!   SL(2,R) block at `gamma = 2*rho` has `[1]` on the compact-Cartan
//!   element and `[0]` on the split ones), so callers must pass parameters
//!   whose deformation terms all share the supplied value — the language
//!   layer uses `rc.lambda_rho(p)` of the looked-up parameter, exactly like
//!   the verified `deform` arm. On a PROPER integral subsystem the parent
//!   is a [`PartialBlock`] (the `common_block` of `common_context`,
//!   repr.cpp:2666-2670) and each row's reconstruction uses its own stored
//!   `gamma_lambda` plus the lookup's block modifier
//!   (`RepContext::sr_with_modifier`, repr.cpp:815-823) exactly as
//!   upstream's `common_block::sr` does; see
//!   [`KlSumParent`]. The `twisted_KL_sum_at_s` drivers and
//!   [`twisted_deformation_terms`] accept both parent kinds (slices 3-4 of
//!   docs/slices/twisted_ext_proper_workorder.md).
//! - The rank-0 integral subsystem is detected by [`IntegralBlockScope`]:
//!   the common block is the singleton `{p}` of length 0, and the language
//!   layer takes that fast path.
//! - Upstream's `block_modifier`-indexed singular-orbit computations
//!   (repr.cpp:2380-2390 and 2617-2633, via `ext_block::reduce_to` and the
//!   inverse of `bm.simple_pi`) coincide with
//!   [`ExtBlock::singular_orbits`] of the plain simple-coroot singular set
//!   when `bm` is trivial (`bm.simp_int` is then the identity-indexed
//!   simple list and `bm.simple_pi` the identity permutation); see
//!   [`singular_orbits_at`]. The twisted KL sums take the caller-folded
//!   orbit flags, so a partial parent folds its subsystem-generator
//!   singular set (`common_context::singular`) the same way.
//! - The `weyl::alcove_center` shrink (repr.cpp:2556-2557) is applied before
//!   recursive twisted deformation when `gamma.denominator() > 2^rank`.
//! - `Rep_table` memoisation (`deformation_unit`/`alcove_hash`) is replaced
//!   by plain recomputation; without a shared pool the memo-hit flip
//!   adjustment (repr.cpp:2576-2584) is unnecessary, since every result is
//!   computed for the very parameter whose flip is being reported.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use crate::ext_block::ExtBlock;
use crate::ext_kl::{contributions, ExtKlTable};
use crate::ext_param::{extended_restrict_to_k, scaled_extended_finalise, ExtRepContext};
use crate::kl_polynomial::KlPol;
use crate::matreduc::{exp_i, inverse_upper_triangular, IntMatrix};
use crate::partial_block::PartialBlock;
use crate::rep_context::{RepContext, StandardRepr};
use crate::{
    BlockDescent, BlockGraph, BlockModifier, BlockTopology, CommonContext, KType, RankFlags,
    RationalWeight, StructureError, Weight,
};

// ---------------------------------------------------------------------------
// Split coefficients.
// ---------------------------------------------------------------------------

/// Upstream `arithmetic::Split_integer` (utilities/arithmetic.h:152-213),
/// stored as the coefficient pair `(a, b)` of `a + b*s` with `s*s = 1`
/// (upstream stores the evaluations at `1` and `-1`; the pair here is what
/// the language layer's `SplitValue` and the E2 extended-parameter drivers
/// use). All arithmetic wraps like upstream's `int`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SplitInteger {
    a: i32,
    b: i32,
}

impl SplitInteger {
    /// The value `a + b*s`.
    pub const fn new(a: i32, b: i32) -> Self {
        Self { a, b }
    }

    pub const fn zero() -> Self {
        Self::new(0, 0)
    }

    /// The coefficient of `1` (upstream `e`).
    pub const fn e(self) -> i32 {
        self.a
    }

    /// The coefficient of `s` (upstream `s`).
    pub const fn s(self) -> i32 {
        self.b
    }

    pub fn is_zero(self) -> bool {
        self.a == 0 && self.b == 0
    }

    pub fn negate(self) -> Self {
        Self::new(self.a.wrapping_neg(), self.b.wrapping_neg())
    }

    /// `*self += n` (arithmetic.h:173): add an integer to the `1`-part.
    pub fn add_int(self, n: i32) -> Self {
        Self::new(self.a.wrapping_add(n), self.b)
    }

    /// Multiplication by `s`: `(a + b*s)*s = b + a*s` (upstream `times_s`,
    /// arithmetic.h:197-199).
    pub fn times_s(self) -> Self {
        Self::new(self.b, self.a)
    }

    /// Multiplication by `1 - s` (upstream `times_1_s`, arithmetic.h:200-204):
    /// `(a + b*s)(1 - s) = (a - b) + (b - a)*s`.
    pub fn times_1_s(self) -> Self {
        let diff = self.a.wrapping_sub(self.b);
        Self::new(diff, diff.wrapping_neg())
    }

    pub fn mul_int(self, n: i32) -> Self {
        Self::new(self.a.wrapping_mul(n), self.b.wrapping_mul(n))
    }
}

impl std::ops::Add for SplitInteger {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new(self.a.wrapping_add(other.a), self.b.wrapping_add(other.b))
    }
}

impl std::ops::Mul for SplitInteger {
    type Output = Self;

    /// Split multiplication: `(a + b*s)(c + d*s) = (ac + bd) + (ad + bc)*s`
    /// (upstream `operator*`, componentwise on the evaluations).
    fn mul(self, other: Self) -> Self {
        Self::new(
            self.a
                .wrapping_mul(other.a)
                .wrapping_add(self.b.wrapping_mul(other.b)),
            self.a
                .wrapping_mul(other.b)
                .wrapping_add(self.b.wrapping_mul(other.a)),
        )
    }
}

impl From<(i32, i32)> for SplitInteger {
    fn from((a, b): (i32, i32)) -> Self {
        Self::new(a, b)
    }
}

impl From<SplitInteger> for (i32, i32) {
    fn from(value: SplitInteger) -> Self {
        (value.e(), value.s())
    }
}

// ---------------------------------------------------------------------------
// Integral-subsystem classification.
// ---------------------------------------------------------------------------

/// Which common block upstream would build for an infinitesimal character
/// (the block on the integral subsystem, `common_context`/`common_block`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntegralBlockScope {
    /// No root pairs integrally with `gamma`: the integral subsystem has
    /// rank 0 and the common block is the singleton `{p}` of length 0. The
    /// deformation terms of such a parameter are empty
    /// (`block.length(y) == 0`, repr.cpp:2435-2436) and its twisted KL sum
    /// at `s` is `1*p` (the `x == y == 0` column entry).
    Singleton,
    /// Every simple coroot (hence every coroot) pairs integrally: the
    /// common block is the full block. Drivers still take it from
    /// `Rep_table::lookup` (the interval-below partial block), as upstream
    /// does even at a full integral subsystem (repr.cpp:2378-2382): the
    /// full block's y-classes are propagated from its own generator, so a
    /// delta-fixed seed can be absent from the full block's extended
    /// block, while the seed-propagated interval block always contains it.
    Full,
    /// A proper integral subsystem: common-block support is not ported.
    /// Callers must fail loudly rather than silently computing on the full
    /// block (the A1 `nu = [1]/2` trap: the full block has size 3 where
    /// the integral block is a singleton).
    ProperSubsystem,
}

/// Classify `gamma` by its integral root system (blocks.cpp:701-709 reads
/// the same pairings for the singular set). A coroot pairs integrally when
/// `coroot . gamma.numerator()` is divisible by `gamma.denominator()`.
pub fn integral_block_scope(
    rc: &RepContext,
    gamma: &RationalWeight,
) -> Result<IntegralBlockScope, StructureError> {
    let datum = rc.datum();
    let numerator = gamma.numerator();
    let denominator = gamma.denominator();
    let mut integral_simples = 0_usize;
    for coroot in datum.simple_coroots() {
        let pairing = dot_numerator(numerator, coroot.as_slice())?;
        if pairing % denominator == 0 {
            integral_simples += 1;
        }
    }
    if integral_simples == datum.semisimple_rank() {
        return Ok(IntegralBlockScope::Full);
    }
    // Proper subsystem unless NO root pairs integrally (rank 0).
    let system = rc.root_system();
    for (id, _, coroot) in system.entries() {
        if system.is_positive(id) == Some(true)
            && dot_numerator(numerator, coroot.as_slice())? % denominator == 0
        {
            return Ok(IntegralBlockScope::ProperSubsystem);
        }
    }
    Ok(IntegralBlockScope::Singleton)
}

/// `coroot . gamma.numerator()` as i64 (coroot coordinates are i32).
fn dot_numerator(numerator: &[i64], coroot: &[i32]) -> Result<i64, StructureError> {
    if numerator.len() != coroot.len() {
        return Err(StructureError::RankMismatch {
            expected: numerator.len(),
            actual: coroot.len(),
        });
    }
    Ok(numerator
        .iter()
        .zip(coroot.iter())
        .map(|(&n, &c)| n.wrapping_mul(i64::from(c)))
        .sum())
}

/// `common_block::singular(gamma)` with trivial block modifier
/// (blocks.cpp:701-709): the simple coroots vanishing on `gamma`.
pub fn simple_singular_flags(
    rc: &RepContext,
    gamma: &RationalWeight,
) -> Result<RankFlags, StructureError> {
    let datum = rc.datum();
    let mut result = RankFlags::empty();
    for (s, coroot) in datum.simple_coroots().iter().enumerate() {
        if dot_numerator(gamma.numerator(), coroot.as_slice())? == 0 {
            result.set(s);
        }
    }
    Ok(result)
}

/// The singular-orbit flags of an extended block at `gamma`
/// (repr.cpp:2380-2390 with trivial `bm`, or the wrapper's plain fold
/// atlas-types.w:8138-8141): `singular_orbits.set(s, singular[orbit(s).s0])`.
/// With a non-trivial block modifier upstream instead folds the transformed
/// simply-integral set through `ext_block::reduce_to`; that case is outside
/// the ported surface.
pub fn singular_orbits_at(
    rc: &RepContext,
    eblock: &ExtBlock,
    gamma: &RationalWeight,
) -> Result<RankFlags, StructureError> {
    Ok(eblock.singular_orbits(&simple_singular_flags(rc, gamma)?))
}

// ---------------------------------------------------------------------------
// Polynomial assembly helpers (upstream SR_poly/K_type_poly merging).
// ---------------------------------------------------------------------------

/// `SR_poly::add_term`: merge like parameters, dropping zero coefficients.
fn add_param_term(
    terms: &mut Vec<(StandardRepr, SplitInteger)>,
    sr: StandardRepr,
    coef: SplitInteger,
) {
    if coef.is_zero() {
        return;
    }
    if let Some(position) = terms.iter().position(|(existing, _)| *existing == sr) {
        let merged = terms[position].1 + coef;
        if merged.is_zero() {
            terms.remove(position);
        } else {
            terms[position].1 = merged;
        }
    } else {
        terms.push((sr, coef));
    }
}

/// `K_type_poly::add_term`: merge like K-types, dropping zero coefficients.
fn add_ktype_term(terms: &mut Vec<(KType, SplitInteger)>, ktype: KType, coef: SplitInteger) {
    if coef.is_zero() {
        return;
    }
    if let Some(position) = terms.iter().position(|(existing, _)| *existing == ktype) {
        let merged = terms[position].1 + coef;
        if merged.is_zero() {
            terms.remove(position);
        } else {
            terms[position].1 = merged;
        }
    } else {
        terms.push((ktype, coef));
    }
}

/// `K_type_poly::add_multiple`: `terms += coef * def`.
fn add_ktype_multiple(
    terms: &mut Vec<(KType, SplitInteger)>,
    def: &[(KType, SplitInteger)],
    coef: SplitInteger,
) {
    for (ktype, c) in def {
        add_ktype_term(terms, ktype.clone(), *c * coef);
    }
}

/// Horner evaluation of a KL polynomial at `q = s` with `times_s` steps
/// (repr.cpp:2322-2327 and 2412-2414).
fn evaluate_at_s(pol: &KlPol) -> SplitInteger {
    let mut eval = SplitInteger::zero();
    for &coefficient in pol.as_slice().iter().rev() {
        eval = eval.times_s().add_int(coefficient);
    }
    eval
}

/// The parent-block x-coordinate of `z`, or an invariant error.
fn block_x(block: &BlockGraph, z: usize) -> Result<crate::KgbId, StructureError> {
    block.x(z).ok_or(StructureError::BlockInvariantViolation {
        invariant: "block element x coordinate",
    })
}

/// The parent-block length of `z`, or an invariant error.
fn block_length(block: &BlockGraph, z: usize) -> Result<usize, StructureError> {
    block
        .length(z)
        .ok_or(StructureError::BlockInvariantViolation {
            invariant: "block element length",
        })
}

/// Common-block deformation terms for a partial block returned by
/// `RepTable::lookup` (repr.cpp:1933-2025).  Unlike the historical
/// `RepContext::deformation_terms` helper, this follows the upstream
/// `contributions(block, block.singular(bm,gamma), y)` path and reconstructs
/// every output row through its stored `StandardReprMod` and lookup modifier.
pub fn common_deformation_terms(
    rc: &RepContext,
    block: &PartialBlock,
    modifier: &BlockModifier,
    y: usize,
    gamma: &RationalWeight,
) -> Result<Vec<(StandardRepr, i32)>, StructureError> {
    let y_len = block
        .length(y)
        .ok_or(StructureError::BlockInvariantViolation {
            invariant: "common deformation y length",
        })?;
    if y_len == 0 {
        return Ok(Vec::new());
    }

    let seed = block
        .element(y)
        .ok_or(StructureError::BlockInvariantViolation {
            invariant: "common deformation y representative",
        })?;
    let ctxt = CommonContext::integral(rc, seed.gamma_lambda())?;
    let singular = ctxt.singular_flags(gamma)?;

    // `repr::contributions` expands each row to the final rows surviving the
    // singular system.  The first singular descent determines the branch.
    let mut contribution = vec![Vec::<(usize, i32)>::new(); y + 1];
    for z in 0..=y {
        let descent = (0..block.rank()).find_map(|s| {
            singular
                .get(s)
                .copied()
                .filter(|flag| *flag)
                .and_then(|_| block.descent(z, s).map(|d| (s, d)))
                .filter(|(_, d)| d.is_descent())
        });
        match descent {
            None => contribution[z].push((z, 1)),
            Some((s, BlockDescent::ComplexDescent)) => {
                let target = block
                    .cross(z, s)
                    .ok_or(StructureError::BlockInvariantViolation {
                        invariant: "common deformation complex cross",
                    })?;
                contribution[z] = contribution[target].clone();
            }
            Some((s, BlockDescent::RealTypeII)) => {
                let target = block.inverse_cayley(z, s).and_then(|pair| pair.0).ok_or(
                    StructureError::BlockInvariantViolation {
                        invariant: "common deformation type-II Cayley",
                    },
                )?;
                contribution[z] = contribution[target].clone();
            }
            Some((s, BlockDescent::RealTypeI)) => {
                let (first, second) =
                    block
                        .inverse_cayley(z, s)
                        .ok_or(StructureError::BlockInvariantViolation {
                            invariant: "common deformation type-I Cayley",
                        })?;
                let first = first.ok_or(StructureError::BlockInvariantViolation {
                    invariant: "common deformation type-I first Cayley",
                })?;
                let second = second.ok_or(StructureError::BlockInvariantViolation {
                    invariant: "common deformation type-I second Cayley",
                })?;
                let mut combined = contribution[first].clone();
                for (element, coefficient) in &contribution[second] {
                    if let Some((_, existing)) = combined.iter_mut().find(|(e, _)| e == element) {
                        *existing = existing.wrapping_add(*coefficient);
                    } else {
                        combined.push((*element, *coefficient));
                    }
                }
                contribution[z] = combined;
            }
            Some((_s, BlockDescent::ImaginaryCompact)) => {}
            Some((_s, _)) => {}
        }
    }

    let mut finals: Vec<usize> = contribution
        .iter()
        .enumerate()
        .filter_map(|(z, values)| {
            values
                .first()
                .is_some_and(|(first, _)| *first == z)
                .then_some(z)
        })
        .collect();
    finals.reverse();
    if finals.is_empty() || finals[0] != y {
        return Err(StructureError::RepInvariantViolation {
            invariant: "common deformation finals",
        });
    }

    let mut kl = crate::KlTable::from_handle(block)?;
    kl.fill(y + 1)?;
    let mut index = vec![usize::MAX; y + 1];
    for (position, &z) in finals.iter().enumerate() {
        index[z] = position;
    }
    let mut acc = vec![0_i32; finals.len()];
    let mut remainder = vec![0_i32; finals.len()];
    remainder[0] = 1;
    let y_parity = y_len % 2;
    for (position, &z) in finals.iter().enumerate() {
        let current = remainder[position];
        if current == 0 {
            continue;
        }
        let contribute = block.length(z).unwrap_or(0) % 2 != y_parity;
        for x in (0..=z).rev() {
            let pol = kl
                .pool()
                .get(kl.kl_pol(x, z)?)
                .cloned()
                .unwrap_or_else(KlPol::zero);
            let mut value = pol.evaluate_at_minus_one();
            if value == 0 {
                continue;
            }
            if !(block.length(z).unwrap_or(0) - block.length(x).unwrap_or(0)).is_multiple_of(2) {
                value = value.wrapping_neg();
            }
            for &(element, coefficient) in &contribution[x] {
                let target = index[element];
                if target == usize::MAX {
                    return Err(StructureError::RepInvariantViolation {
                        invariant: "common deformation contribution target",
                    });
                }
                let c = current.wrapping_mul(value).wrapping_mul(coefficient);
                remainder[target] = remainder[target].wrapping_sub(c);
                if contribute {
                    acc[target] = acc[target].wrapping_add(c);
                }
            }
        }
    }

    let sr_y = rc.sr_with_modifier(seed, modifier, gamma)?;
    let orientation_y = rc.orientation_number(&sr_y)? as i32;
    let mut result = Vec::new();
    for (position, &z) in finals.iter().enumerate() {
        if acc[position] == 0 {
            continue;
        }
        let srm = block
            .element(z)
            .ok_or(StructureError::BlockInvariantViolation {
                invariant: "common deformation result representative",
            })?;
        let sr = rc.sr_with_modifier(srm, modifier, gamma)?;
        let orientation = rc.orientation_number(&sr)? as i32;
        result.push((
            sr,
            acc[position].wrapping_mul(exp_i(orientation_y - orientation)),
        ));
    }
    Ok(result)
}

// ---------------------------------------------------------------------------
// Twisted deformation terms.
// ---------------------------------------------------------------------------

/// `Rep_table::twisted_deformation_terms` (repr.cpp:2426-2520) with trivial
/// block modifier: the deformation terms of the final, delta-fixed parent
/// block element `y` (PARENT numbering), as `(StandardRepr, int)` pairs in
/// reverse-accumulated finals order (the wrapper maps each coefficient `c`
/// to `Split(c, -c)` and sorts into `SR_poly` order).
///
/// `singular_orbits` flags the singular generators in EXTENDED-block
/// numbering (folded by the caller from the parent's own singular set; see
/// [`singular_orbits_at`] and [`KlSumParent`]); `gamma` is the common
/// infinitesimal character. Parent lengths and the `parent.sr`
/// reconstruction (repr.cpp:2504-2510, blocks.cpp:1260-1264) go through
/// [`KlSumParent`], so a proper-subsystem parent uses each row's own
/// `lambda_rho`.
pub fn twisted_deformation_terms(
    rc: &RepContext,
    parent: &KlSumParent,
    eblock: &ExtBlock,
    y: usize,
    singular_orbits: &RankFlags,
    gamma: &RationalWeight,
) -> Result<Vec<(StandardRepr, i32)>, StructureError> {
    if !eblock.is_present(y) {
        return Err(StructureError::RepInvariantViolation {
            invariant: "twisted deformation terms: parameter is not delta-fixed in its block",
        });
    }
    let y_index = eblock.element(y);
    let mut result = Vec::new();
    if parent.length(y)? == 0 {
        return Ok(result); // easy case, null result (repr.cpp:2435-2436)
    }

    let contrib = contributions(eblock, singular_orbits, y_index);
    // Finals in EXTENDED numbering, accumulated in reverse (push_front).
    let mut finals: Vec<usize> = Vec::new();
    for (z, entries) in contrib.iter().enumerate() {
        if entries.first().is_some_and(|(first, _)| *first == z) {
            finals.push(z);
        }
    }
    finals.reverse();

    let mut kl_tab = ExtKlTable::new(eblock)?;
    kl_tab.fill_columns(y_index + 1)?;

    // Evaluate the twisted KL pool at q = -1 (repr.cpp:2446-2457).
    let pool = kl_tab.polys();
    let mut pool_at_minus_1: Vec<i32> = Vec::with_capacity(pool.len());
    for index in 0..pool.len() {
        pool_at_minus_1.push(
            pool.get(index)
                .map_or(0, crate::kl_polynomial::KlPol::evaluate_at_minus_one),
        );
    }

    // Sparse map from final extended element to its position in `finals`.
    let mut index = vec![usize::MAX; eblock.size()];
    for (position, &z) in finals.iter().enumerate() {
        index[z] = position;
    }

    let mut acc = vec![0_i32; finals.len()];
    let mut remainder = vec![0_i32; finals.len()];
    remainder[0] = 1; // we initialised remainder = 1*sr_y
    let y_parity = parent.length(y)? % 2;

    for (position, &z) in finals.iter().enumerate() {
        let c_cur = remainder[position];
        if c_cur == 0 {
            continue;
        }
        let contribute = parent.length(eblock.z(z))? % 2 != y_parity;
        for x in kl_tab.nonzero_column(z) {
            let (pool_index, negate_p) = kl_tab.kl_pol_index(x, z);
            let pooled = pool_at_minus_1[pool_index];
            if pooled == 0 {
                continue; // polynomials with -1 as a root do not contribute
            }
            // XOR the stored sign with the PARENT length-difference parity
            // (repr.cpp:2486-2488).
            let length_difference_odd = !parent
                .length(eblock.z(x))?
                .wrapping_sub(parent.length(eblock.z(z))?)
                .is_multiple_of(2);
            let val_xz = if negate_p != length_difference_odd {
                pooled.wrapping_neg()
            } else {
                pooled
            };
            for &(element, coefficient) in &contrib[x] {
                let j = index[element];
                debug_assert!(j != usize::MAX && j >= position);
                let c = c_cur.wrapping_mul(val_xz).wrapping_mul(coefficient);
                remainder[j] = remainder[j].wrapping_sub(c);
                if contribute {
                    acc[j] = acc[j].wrapping_add(c);
                }
            }
        }
        debug_assert_eq!(remainder[position], 0);
    }

    // The orientation pass (repr.cpp:2501-2517).
    let sr_y = parent.sr(rc, y, gamma)?;
    let orient_y = rc.orientation_number(&sr_y)?;
    for (position, &f) in finals.iter().enumerate() {
        let c = acc[position];
        if c == 0 {
            continue;
        }
        let sr_z = parent.sr(rc, eblock.z(f), gamma)?;
        let diff = orient_y as i32 - rc.orientation_number(&sr_z)? as i32;
        result.push((sr_z, c.wrapping_mul(exp_i(diff))));
    }
    Ok(result)
}

// ---------------------------------------------------------------------------
// Twisted KL sums at s.
// ---------------------------------------------------------------------------

/// The parent common block a twisted KL sum or twisted deformation reads
/// (`parent` of repr.cpp:2304-2350, `block` of repr.cpp:2371-2423 and
/// 2426-2520): either the full block of the real form with the
/// caller-supplied constant `lambda_rho` (the module-level caveat), or a
/// proper-subsystem [`PartialBlock`], whose per-row `gamma_lambda`
/// reconstructs each term's own `lambda_rho` exactly as `common_block::sr`
/// does (blocks.cpp:1260-1264: `lambda_rho =
/// gamma.integer_diff(context().gamma_lambda_rho(z))`).
pub enum KlSumParent<'a> {
    /// The full block (`lookup_full_block`) plus the shared `lambda_rho`.
    Full {
        /// The parent block graph.
        block: &'a BlockGraph,
        /// The constant `lambda_rho` of the looked-up parameter.
        lambda_rho: &'a Weight,
    },
    /// A partial common block, optionally viewed through the lookup
    /// modifier that relates its stored representative to the query.
    Partial {
        block: &'a PartialBlock,
        modifier: Option<&'a crate::BlockModifier>,
    },
}

/// Owned parent returned by reducibility-point lookup during recursive
/// twisted deformation. The recursive driver must keep the selected parent
/// alive while [`twisted_deformation_terms`] borrows its block view.
pub enum DeformParent {
    Full {
        block: Box<BlockGraph>,
        lambda_rho: Weight,
    },
    Partial {
        block: std::sync::Arc<PartialBlock>,
        modifier: crate::BlockModifier,
    },
}

impl DeformParent {
    fn as_kl_sum_parent(&self) -> KlSumParent<'_> {
        match self {
            Self::Full { block, lambda_rho } => KlSumParent::Full {
                block: block.as_ref(),
                lambda_rho,
            },
            Self::Partial { block, modifier } => KlSumParent::Partial {
                block,
                modifier: Some(modifier),
            },
        }
    }
}

impl KlSumParent<'_> {
    /// The parent-block x-coordinate of `z`, or an invariant error.
    fn x(&self, z: usize) -> Result<crate::KgbId, StructureError> {
        let x = match self {
            Self::Full { block, .. } => block.x(z),
            Self::Partial { block, .. } => block.x(z),
        };
        x.ok_or(StructureError::BlockInvariantViolation {
            invariant: "block element x coordinate",
        })
    }

    /// The parent-block length of `z`, or an invariant error.
    fn length(&self, z: usize) -> Result<usize, StructureError> {
        let length = match self {
            Self::Full { block, .. } => block.length(z),
            Self::Partial { block, .. } => block.length(z),
        };
        length.ok_or(StructureError::BlockInvariantViolation {
            invariant: "block element length",
        })
    }

    /// `parent.sr(z, gamma)` (blocks.cpp:1260-1264): the parameter of
    /// parent row `z` at infinitesimal character `gamma`. The full-block
    /// variant uses the caller-supplied constant `lambda_rho`; the partial
    /// variant derives each row's own from its stored `gamma_lambda`
    /// (`integer_diff(gamma - rho, gamma_lambda(z))`, integral because the
    /// row lies in gamma's common block).
    fn sr(
        &self,
        rc: &RepContext,
        z: usize,
        gamma: &RationalWeight,
    ) -> Result<StandardRepr, StructureError> {
        match self {
            Self::Full { lambda_rho, .. } => rc.sr_gamma(self.x(z)?, lambda_rho, gamma),
            Self::Partial { block, modifier } => {
                let srm = block
                    .element(z)
                    .ok_or(StructureError::BlockInvariantViolation {
                        invariant: "block element representative",
                    })?;
                match modifier {
                    Some(modifier) => rc.sr_with_modifier(srm, modifier, gamma),
                    None => srm.to_standard(rc, gamma),
                }
            }
        }
    }
}

/// The free function `twisted_KL_sum` (repr.cpp:2304-2350): the alternating
/// twisted KL column sum at `q = s` of the EXTENDED-block element `y`,
/// with signs from the EXTENDED block's own length function
/// (`eblock.length`, repr.cpp:2339-2344). `singular_orbits` flags the
/// singular generators in extended-block orbit numbering (the caller folds
/// the parent's own singular set; see [`singular_orbits_at`] and
/// [`KlSumParent`]), and `parent`/`gamma` reconstruct the contribution
/// targets (`parent.sr`, repr.cpp:2346).
pub fn twisted_kl_sum(
    rc: &RepContext,
    eblock: &ExtBlock,
    y: usize,
    parent: &KlSumParent,
    gamma: &RationalWeight,
    singular_orbits: &RankFlags,
) -> Result<Vec<(StandardRepr, SplitInteger)>, StructureError> {
    let mut kl_tab = ExtKlTable::new(eblock)?;
    kl_tab.fill_columns(y + 1)?;

    // A copy of the pool evaluated at q = s (repr.cpp:2316-2328).
    let pool = kl_tab.polys();
    let mut pool_at_s: Vec<SplitInteger> = Vec::with_capacity(pool.len());
    for index in 0..pool.len() {
        pool_at_s.push(pool.get(index).map_or(SplitInteger::zero(), evaluate_at_s));
    }

    let contrib = contributions(eblock, singular_orbits, y);

    let mut result = Vec::new();
    let parity = eblock.length(y) % 2;
    for x in 0..=y {
        let (pool_index, flip) = kl_tab.kl_pol_index(x, y);
        let mut eval = pool_at_s[pool_index];
        if flip {
            eval = eval.negate();
        }
        if eblock.length(x) % 2 != parity {
            eval = eval.negate(); // flip sign at odd length difference
        }
        for &(element, coefficient) in &contrib[x] {
            let sr = parent.sr(rc, eblock.z(element), gamma)?;
            add_param_term(&mut result, sr, eval.mul_int(coefficient));
        }
    }
    Ok(result)
}

/// `Rep_table::twisted_KL_column_at_s` (repr.cpp:2371-2423): the
/// alternating twisted KL column sum at `q = s` of the parameter at
/// PARENT block element `y0`, with signs from the PARENT block's length
/// function (`parent.length(eblock.z(x))`, repr.cpp:2416) — unlike
/// [`twisted_kl_sum`], which uses extended-block lengths.
pub fn twisted_kl_column_at_s(
    rc: &RepContext,
    eblock: &ExtBlock,
    parent: &KlSumParent,
    y0: usize,
    gamma: &RationalWeight,
    singular_orbits: &RankFlags,
) -> Result<Vec<(StandardRepr, SplitInteger)>, StructureError> {
    if !eblock.is_present(y0) {
        return Err(StructureError::RepInvariantViolation {
            invariant: "twisted KL column: parameter is not delta-fixed in its block",
        });
    }
    let y = eblock.element(y0);
    let contrib = contributions(eblock, singular_orbits, y);

    let mut kl_tab = ExtKlTable::new(eblock)?;
    kl_tab.fill_columns(y + 1)?;

    let mut result = Vec::new();
    let y_length = parent.length(y0)?;
    for x in (0..=y).rev() {
        let pol = kl_tab.p(x, y); // flip sign included (ext_kl.cpp:149-154)
        if pol.is_zero() {
            continue;
        }
        let eval = evaluate_at_s(&pol);
        let eval = if !y_length
            .wrapping_sub(parent.length(eblock.z(x))?)
            .is_multiple_of(2)
        {
            eval.negate() // alternating sum of the KL column at s
        } else {
            eval
        };
        for &(element, coefficient) in &contrib[x] {
            let sr = parent.sr(rc, eblock.z(element), gamma)?;
            add_param_term(&mut result, sr, eval.mul_int(coefficient));
        }
    }
    Ok(result)
}

// ---------------------------------------------------------------------------
// Block deformation to a height bound.
// ---------------------------------------------------------------------------

/// `Rep_table::block_deformation_to_height` (repr.cpp:2027-2124) with
/// trivial block modifier: deform the terms of `p`'s full block whose
/// height does not exceed `height_bound` (the caller passes `u32::MAX` for
/// upstream's negative-bound "maximal level"). `block` is the full block
/// (`lookup_full_block`; the caller makes `p` dominant first),
/// `gamma`/`lambda_rho` are `p`'s data, and `accumulator` holds the
/// `ParamPol` terms the block's coefficients are drawn from.
///
/// Returns the deformed `(StandardRepr, SplitInteger)` terms in downward
/// (reversed) block order — the language layer then slides each down its
/// reducibility points (atlas-types.w:8192-8198) — and, parallel to
/// `accumulator`, the flags of the consumed terms (upstream mutates the
/// queue in place with `queue.erase`; Atlas values are immutable, so the
/// split is reported instead).
///
/// Upstream fills the dual block's KL table with `plug_hole` above the
/// height bound; the holes only skip work — polynomial values do not
/// depend on them — so this port fills the whole table
/// (`repr.cpp:2057` fills everything not plugged).
///
/// The filled dual-block table is lazily cached per block identity for the
/// session (`rep_table::with_dual_kl_table`): upstream rebuilds it per
/// call, so the cache only skips recomputation and the observable output
/// is unchanged.
pub fn block_deformation_to_height(
    rc: &RepContext,
    block: &BlockGraph,
    gamma: &RationalWeight,
    lambda_rho: &Weight,
    height_bound: u32,
    accumulator: &[(StandardRepr, SplitInteger)],
) -> Result<(Vec<(StandardRepr, SplitInteger)>, Vec<bool>), StructureError> {
    crate::rep_table::with_dual_kl_table(block, |kl_tab| {
        block_deformation_with_dual_kl(
            rc,
            block,
            gamma,
            lambda_rho,
            height_bound,
            accumulator,
            kl_tab,
        )
    })
}

/// A digest over the `StandardRepr::operator==` fields (repr.cpp:36-40:
/// `x`, the packed torsion part, `gamma`) for the accumulator buckets of
/// [`block_deformation_with_dual_kl`]; bucket hits are verified by full
/// equality, so collisions only cost a short scan.
fn param_digest(sr: &StandardRepr) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    sr.x().hash(&mut hasher);
    sr.y_bits().hash(&mut hasher);
    sr.gamma().hash(&mut hasher);
    hasher.finish()
}

/// The body of [`block_deformation_to_height`] with the dual block's
/// filled KL table in hand (see there for the upstream references).
fn block_deformation_with_dual_kl(
    rc: &RepContext,
    block: &BlockGraph,
    gamma: &RationalWeight,
    lambda_rho: &Weight,
    height_bound: u32,
    accumulator: &[(StandardRepr, SplitInteger)],
    kl_tab: &mut crate::KlTableHandle<std::sync::Arc<BlockGraph>>,
) -> Result<(Vec<(StandardRepr, SplitInteger)>, Vec<bool>), StructureError> {
    // The dual KL pool evaluated at q = -1 (repr.cpp:2059-2066).
    let pool = kl_tab.pool();
    let mut value_at_minus_1: Vec<i32> = Vec::with_capacity(pool.len());
    for index in 0..pool.len() {
        value_at_minus_1.push(
            pool.get(index)
                .map_or(0, crate::kl_polynomial::KlPol::evaluate_at_minus_one),
        );
    }

    // Retained elements (height at most the bound), ascending, with their
    // accumulator coefficients (repr.cpp:2039-2056). The digest buckets
    // index the accumulator so each retained element probes O(bucket)
    // instead of scanning the whole accumulator; a bucket holds positions
    // in ascending order, so the first unconsumed equal entry of a bucket
    // is exactly upstream's `queue.find`.
    let mut digest_buckets: HashMap<u64, Vec<usize>> = HashMap::new();
    for (position, (sr, _)) in accumulator.iter().enumerate() {
        digest_buckets
            .entry(param_digest(sr))
            .or_default()
            .push(position);
    }
    let mut consumed = vec![false; accumulator.len()];
    let mut entries: Vec<(usize, StandardRepr, SplitInteger)> = Vec::new();
    for z in 0..block.size() {
        let q = rc.sr_gamma(block_x(block, z)?, lambda_rho, gamma)?;
        if q.height() > height_bound {
            continue;
        }
        // `queue.find(q)` + `queue.erase`: only unconsumed terms match,
        // and each match consumes exactly one occurrence.
        let coef = match digest_buckets.get(&param_digest(&q)) {
            Some(bucket) => match bucket
                .iter()
                .find(|&&position| !consumed[position] && accumulator[position].0 == q)
            {
                Some(&position) => {
                    consumed[position] = true;
                    accumulator[position].1
                }
                None => SplitInteger::zero(),
            },
            None => SplitInteger::zero(),
        };
        entries.push((z, q, coef));
    }

    // Drop the elements killed by the translation functor
    // (`block.survives`, repr.cpp:2068-2077).
    let singular = simple_singular_flags(rc, gamma)?;
    let mut survivors: Vec<(usize, StandardRepr, SplitInteger)> = Vec::new();
    for (z, q, coef) in entries {
        let killed = (0..block.rank()).any(|s| {
            singular.is_set(s) && block.descent_value(z, s).is_some_and(|d| d.is_descent())
        });
        if killed {
            if !coef.is_zero() {
                return Err(StructureError::RepInvariantViolation {
                    invariant: "block deformation: accumulator holds a non-final block term",
                });
            }
        } else {
            survivors.push((z, q, coef));
        }
    }

    // The dual KL matrix is lower triangular from the block's viewpoint;
    // build its transpose on the survivors, evaluated at q = -1
    // (repr.cpp:2080-2087).
    let n = survivors.len();
    let top = block.size() - 1;
    let mut q_mat = IntMatrix::identity(n);
    for j in 1..n {
        for i in 0..j {
            let pool_index = kl_tab.kl_pol(top - survivors[j].0, top - survivors[i].0)?;
            q_mat.set(i, j, value_at_minus_1[pool_index]);
        }
    }
    let signed_p = inverse_upper_triangular(&q_mat)?;
    let mut odd_length: Vec<bool> = Vec::with_capacity(n);
    for (z, _, _) in &survivors {
        odd_length.push(block_length(block, *z)? % 2 != 0);
    }

    // The parity/orientation coefficient pass (repr.cpp:2096-2121),
    // indexed by ASCENDING survivor position; upstream walks the reversed
    // result list, which is the same order.
    //
    // Each survivor's orientation number is queried once per (position, j)
    // pair upstream; it depends only on the survivor, so compute all of
    // them once up front.
    let orientations: Vec<u32> = survivors
        .iter()
        .map(|(_, sr, _)| rc.orientation_number(sr))
        .collect::<Result<_, _>>()?;
    let mut coefs: Vec<SplitInteger> = survivors.iter().map(|(_, _, coef)| *coef).collect();
    for position in (0..n).rev() {
        let c_cur = coefs[position];
        if c_cur.is_zero() {
            continue;
        }
        // The product of the opposite-parity columns of `signed_P` with
        // column `position` of `q_mat` (repr.cpp:2100-2108).
        let mut coef = vec![SplitInteger::zero(); n];
        for j in 0..position {
            if odd_length[j] == odd_length[position] {
                continue;
            }
            for i in 0..=j {
                let contribution = signed_p.get(i, j).wrapping_mul(q_mat.get(j, position));
                coef[i] = coef[i].add_int(contribution);
            }
        }
        let our_orient = orientations[position] as i32;
        for j in (0..position).rev() {
            let mut cj = coef[j] * c_cur;
            let diff = our_orient - orientations[j] as i32;
            debug_assert_eq!(diff % 2, 0);
            cj = cj.times_1_s();
            if diff % 4 != 0 {
                cj = cj.times_s(); // equivalently negate on the (c, -c) form
            }
            coefs[j] = coefs[j] + cj;
        }
    }

    // Upstream returns the result reversed: downward block traversal.
    let result = survivors
        .into_iter()
        .zip(coefs)
        .rev()
        .map(|((_, q, _), coef)| (q, coef))
        .collect();
    Ok((result, consumed))
}

// ---------------------------------------------------------------------------
// Recursive twisted deformation.
// ---------------------------------------------------------------------------

/// `Rep_table::twisted_deformation` (repr.cpp:2552-2653), without the
/// `Rep_table` memoisation (see the module docs): the full recursive
/// twisted deformation of the final, delta-fixed parameter `z` over
/// `ctx`'s delta, as `(KType, SplitInteger)` terms, plus the net flip
/// recorded by the shrink-wrap to the last reducibility point
/// (`flip == false` when no shrink-wrap happened, repr.cpp:2561).
///
/// `lookup` plays the role of `rt.lookup(zi, index, bm)` +
/// `block.extended_block(bm, ...)` at a reducibility-point parameter with
/// INTEGRAL gamma: it returns the parameter's interval-below common block,
/// the extended block over `ctx.delta()`, the parent row, and the singular
/// extended-generator orbits. This applies to both full and proper integral
/// subsystems; the rank-0 singleton contributes no deformation terms and
/// does not call `lookup`. A non-trivial block modifier rides in
/// [`DeformParent::Partial`] and is applied through
/// `RepContext::sr_with_modifier` (repr.cpp:815-823).
pub fn twisted_deformation(
    ctx: &ExtRepContext,
    z: &StandardRepr,
    lookup: &mut dyn FnMut(
        &StandardRepr,
    )
        -> Result<(DeformParent, ExtBlock, usize, RankFlags), StructureError>,
) -> Result<(Vec<(KType, SplitInteger)>, bool), StructureError> {
    let mut never_cancel = || false;
    match twisted_deformation_with_cancel(ctx, z, lookup, &mut never_cancel)? {
        Some(result) => Ok(result),
        None => unreachable!("the non-cancellable twisted deformation cannot be cancelled"),
    }
}

/// Cancellable form of [`twisted_deformation`]. The probe is checked between
/// each recursive or block-level operation; cancellation returns `Ok(None)`
/// without publishing a partial polynomial.
pub fn twisted_deformation_with_cancel(
    ctx: &ExtRepContext,
    z: &StandardRepr,
    lookup: &mut dyn FnMut(
        &StandardRepr,
    )
        -> Result<(DeformParent, ExtBlock, usize, RankFlags), StructureError>,
    cancelled: &mut dyn FnMut() -> bool,
) -> Result<Option<(Vec<(KType, SplitInteger)>, bool)>, StructureError> {
    if cancelled() {
        return Ok(None);
    }
    let rc = ctx.rc();
    debug_assert!(matches!(z.is_final(rc), Ok(true)));
    debug_assert!(rc.is_fixed(z, ctx.delta()));
    let mut z = if crate::denominator_exceeds_alcove_bound(rc.rank(), z.gamma().denominator()) {
        crate::alcove_center(rc, z)?
    } else {
        z.clone()
    };
    if cancelled() {
        return Ok(None);
    }

    let mut rp = rc.reducibility_points(&z)?;
    if cancelled() {
        return Ok(None);
    }
    let mut flip = false; // no flip recorded when shrink wrapping is not done
    if let Some(&back) = rp.last() {
        if !rat_eq(back, (1, 1)) {
            // Shrink wrap toward nu = 0 to the last reducibility point.
            let (shrunk, shrink_flip) = scaled_extended_finalise(ctx, &z, back.0, back.1)?;
            z = shrunk;
            flip = shrink_flip;
            for point in &mut rp {
                *point = rat_div(*point, back);
            }
            debug_assert!(rp.last().is_some_and(|&last| rat_eq(last, (1, 1))));
        }
    }
    if cancelled() {
        return Ok(None);
    }

    // Initialise to the restriction of z expanded to finals
    // (repr.cpp:2589-2595).
    let mut result: Vec<(KType, SplitInteger)> = Vec::new();
    for (ktype, coefficient) in extended_restrict_to_k(ctx, &z)? {
        add_ktype_term(&mut result, ktype, SplitInteger::from(coefficient));
        if cancelled() {
            return Ok(None);
        }
    }

    // The deformation terms at all reducibility points (repr.cpp:2597-2647).
    for i in (0..rp.len()).rev() {
        if cancelled() {
            return Ok(None);
        }
        let (zi, flip_p) = scaled_extended_finalise(ctx, &z, rp[i].0, rp[i].1)?;
        if cancelled() {
            return Ok(None);
        }
        let scope = integral_block_scope(rc, zi.gamma())?;
        if cancelled() {
            return Ok(None);
        }
        match scope {
            IntegralBlockScope::Singleton => {
                // The rank-0 integral block is the length-0 singleton; its
                // deformation terms are empty (repr.cpp:2435-2436).
            }
            IntegralBlockScope::ProperSubsystem | IntegralBlockScope::Full => {
                let (parent, eblock, index, singular_orbits) = lookup(&zi)?;
                if cancelled() {
                    return Ok(None);
                }
                let terms = twisted_deformation_terms(
                    rc,
                    &parent.as_kl_sum_parent(),
                    &eblock,
                    index,
                    &singular_orbits,
                    zi.gamma(),
                )?;
                if cancelled() {
                    return Ok(None);
                }
                for (term, coefficient) in terms {
                    let Some((def, flip_def)) =
                        twisted_deformation_with_cancel(ctx, &term, &mut *lookup, &mut *cancelled)?
                    else {
                        return Ok(None);
                    };
                    // $(\mp c, \pm c)$ by the combined flip parity
                    // (repr.cpp:2641-2645).
                    let coef = if flip_p != flip_def {
                        SplitInteger::new(coefficient.wrapping_neg(), coefficient)
                    } else {
                        SplitInteger::new(coefficient, coefficient.wrapping_neg())
                    };
                    add_ktype_multiple(&mut result, &def, coef);
                    if cancelled() {
                        return Ok(None);
                    }
                }
            }
        }
    }
    if cancelled() {
        return Ok(None);
    }
    Ok(Some((result, flip)))
}

/// Rational equality for the `(numerator, denominator)` pairs of
/// [`RepContext::reducibility_points`], which are not stored reduced.
fn rat_eq((n1, d1): (i64, i64), (n2, d2): (i64, i64)) -> bool {
    n1.wrapping_mul(d2) == n2.wrapping_mul(d1)
}

/// `(n/d) / (fn/fd)`, reduced (repr.cpp:2568-2569 `a /= f`).
fn rat_div((n, d): (i64, i64), (fn_, fd): (i64, i64)) -> (i64, i64) {
    let mut num = n.wrapping_mul(fd);
    let mut den = d.wrapping_mul(fn_);
    let mut common = gcd(num.unsigned_abs(), den.unsigned_abs());
    if common == 0 {
        common = 1;
    }
    num /= common as i64;
    den /= common as i64;
    if den < 0 {
        num = -num;
        den = -den;
    }
    (num, den)
}

fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    a
}

// ---------------------------------------------------------------------------
// Tests.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ext_param::extended_finalise;
    use crate::{
        AdjointFiberBudget, BasedRootDatum, CartanClassification, CartanClassificationBudget,
        CartanId, Coweight, InnerClass, IntegerLatticeBudget, InvolutionTable,
        InvolutionTableBudget, KgbGraph, KgbId, LatticeInvolution, RealFormSeed,
        StrongRealClassification, WeakRealFormId,
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

    fn graph_with_size(
        inner_class: &InnerClass,
        classification: &CartanClassification,
        strong: &StrongRealClassification,
        table: &mut InvolutionTable,
        size: usize,
    ) -> KgbGraph {
        for form in 0..classification.weak_real_form_count() {
            if strong.kgb_size(WeakRealFormId(form)) != Some(size) {
                continue;
            }
            table.add_cartan(classification, CartanId(0)).unwrap();
            let seed = RealFormSeed::build(
                inner_class,
                classification,
                strong,
                table,
                WeakRealFormId(form),
                &IntegerLatticeBudget::new(64, 100_000, 100_000, 128),
                4_096,
            )
            .unwrap();
            return KgbGraph::build(inner_class, classification, strong, table, &seed).unwrap();
        }
        panic!("no real form with KGB size {size}");
    }

    /// Owns the values a `RepContext` borrows, for fixture construction.
    struct ContextFixture {
        inner_class: InnerClass,
        table: InvolutionTable,
        graph: KgbGraph,
    }

    impl ContextFixture {
        fn rc(&self) -> RepContext<'_> {
            RepContext::new(&self.inner_class, &self.table, &self.graph).unwrap()
        }
    }

    fn fixture(
        datum: BasedRootDatum,
        involution: LatticeInvolution,
        weyl: usize,
        kgb_size: usize,
    ) -> ContextFixture {
        let inner_class = InnerClass::new(datum, involution, weyl).unwrap();
        let classification =
            CartanClassification::build(&inner_class, &class_budget(weyl)).unwrap();
        let strong = StrongRealClassification::build(&classification, 4_096).unwrap();
        let mut table = InvolutionTable::new(
            &inner_class,
            InvolutionTableBudget::new(64, IntegerLatticeBudget::new(64, 100_000, 100_000, 128)),
        )
        .unwrap();
        let graph = graph_with_size(&inner_class, &classification, &strong, &mut table, kgb_size);
        ContextFixture {
            inner_class,
            table,
            graph,
        }
    }

    /// The simply connected A1 datum (root = twice the fundamental weight).
    fn a1_datum() -> BasedRootDatum {
        BasedRootDatum::from_simple_data(
            1,
            vec![vec![2]],
            vec![Weight::new(vec![2])],
            vec![Coweight::new(vec![1])],
        )
        .unwrap()
    }

    /// The simply connected A2 datum (roots in the weight lattice).
    fn a2_datum() -> BasedRootDatum {
        BasedRootDatum::from_simple_data(
            2,
            vec![vec![2, -1], vec![-1, 2]],
            vec![Weight::new(vec![2, -1]), Weight::new(vec![-1, 2])],
            vec![Coweight::new(vec![1, 0]), Coweight::new(vec![0, 1])],
        )
        .unwrap()
    }

    /// The split sl(2,R) context (compact inner class, KGB size 3).
    fn a1_fixture() -> ContextFixture {
        let datum = a1_datum();
        let involution = LatticeInvolution::identity(&datum).unwrap();
        fixture(datum, involution, 2, 3)
    }

    /// The quasisplit su(2,1) context (compact inner class, KGB size 6).
    fn a2_compact_fixture() -> ContextFixture {
        let datum = a2_datum();
        let involution = LatticeInvolution::identity(&datum).unwrap();
        fixture(datum, involution, 8, 6)
    }

    /// Owns a full block together with the primal/dual substrate it was
    /// built from (the deform-arm shape: primal form against the dual
    /// class's quasisplit form, matching `lookup_full_block`).
    struct BlockCtx {
        primal: ContextFixture,
        dual_class: InnerClass,
        dual_graph: KgbGraph,
        dual_table: InvolutionTable,
        block: BlockGraph,
    }

    fn block_fixture(
        datum: BasedRootDatum,
        involution: LatticeInvolution,
        weyl: usize,
        primal_size: usize,
        dual_size: usize,
    ) -> BlockCtx {
        let primal = fixture(datum, involution, weyl, primal_size);
        let dual_class = crate::dual::dual_inner_class(&primal.inner_class, weyl, 64).unwrap();
        let dual_classification =
            CartanClassification::build(&dual_class, &class_budget(weyl)).unwrap();
        let dual_strong = StrongRealClassification::build(&dual_classification, 4_096).unwrap();
        let mut dual_table = InvolutionTable::new(
            &dual_class,
            InvolutionTableBudget::new(64, IntegerLatticeBudget::new(64, 100_000, 100_000, 128)),
        )
        .unwrap();
        let dual_graph = graph_with_size(
            &dual_class,
            &dual_classification,
            &dual_strong,
            &mut dual_table,
            dual_size,
        );
        let block = BlockGraph::build(
            &primal.graph,
            &primal.table,
            &dual_graph,
            &dual_table,
            &dual_class,
            weyl,
        )
        .unwrap();
        BlockCtx {
            primal,
            dual_class,
            dual_graph,
            dual_table,
            block,
        }
    }

    /// The extended block over the identity delta of a compact inner class.
    fn identity_ext_block(ctx: &BlockCtx) -> ExtBlock {
        let delta = LatticeInvolution::identity(ctx.primal.inner_class.datum()).unwrap();
        let twist = ctx
            .primal
            .inner_class
            .based_involution_twist(delta.clone())
            .unwrap();
        let dual_delta = LatticeInvolution::identity(ctx.dual_class.datum()).unwrap();
        let dual_twist = ctx
            .dual_class
            .based_involution_twist(dual_delta.clone())
            .unwrap();
        ExtBlock::build(
            &ctx.block,
            &ctx.primal.graph,
            &ctx.primal.table,
            &ctx.dual_graph,
            &ctx.dual_table,
            &delta,
            &twist,
            &dual_delta,
            &dual_twist,
            ctx.primal.inner_class.datum().cartan_matrix(),
        )
        .unwrap()
    }

    /// The sl(2,R) full block against the dual (adjoint A1) quasisplit
    /// form, whose KGB has size 2.
    fn a1_block() -> BlockCtx {
        let datum = a1_datum();
        let involution = LatticeInvolution::identity(&datum).unwrap();
        block_fixture(datum, involution, 2, 3, 2)
    }

    /// The su(2,1) full block against the dual class's quasisplit form
    /// (KGB size 4), the kl_table.rs block.
    fn a2_block() -> BlockCtx {
        let datum = a2_datum();
        let involution = LatticeInvolution::identity(&datum).unwrap();
        block_fixture(datum, involution, 8, 6, 4)
    }

    fn weight(coordinates: &[i32]) -> Weight {
        Weight::new(coordinates.to_vec())
    }

    fn rational(coordinates: &[i64], denominator: i64) -> RationalWeight {
        RationalWeight::new(coordinates.to_vec(), denominator).unwrap()
    }

    fn param(rc: &RepContext, x: usize, lambda_rho: &[i32], nu: &[i64], den: i64) -> StandardRepr {
        rc.sr(KgbId(x), &weight(lambda_rho), &rational(nu, den))
            .unwrap()
    }

    /// The block element representing `p`: an `x`-coordinate match whose
    /// reconstruction at `p`'s data equals `p` and that is delta-fixed
    /// (present in `eblock`).
    fn block_element_of(
        rc: &RepContext,
        ctx: &BlockCtx,
        eblock: &ExtBlock,
        p: &StandardRepr,
        lambda_rho: &Weight,
    ) -> usize {
        (0..ctx.block.size())
            .find(|&z| {
                ctx.block.x(z) == Some(p.x())
                    && eblock.is_present(z)
                    && rc.sr_gamma(p.x(), lambda_rho, p.gamma()).ok().as_ref() == Some(p)
            })
            .expect("parameter not found in its full block")
    }

    /// A `lookup` closure for [`twisted_deformation`] over the identity
    /// delta, rebuilding the block of `zi` from scratch. Only called for
    /// parameters with reducibility points; the fixtures have none.
    fn identity_lookup(
        _zi: &StandardRepr,
    ) -> Result<(DeformParent, ExtBlock, usize, RankFlags), StructureError> {
        unreachable!("the twisted_family fixture parameters have no reducibility points")
    }

    // --- SplitInteger arithmetic (utilities/arithmetic.h:152-213) --------

    #[test]
    fn split_integer_arithmetic() {
        let v = SplitInteger::new(3, 2); // 3 + 2s
        assert_eq!(v.times_s(), SplitInteger::new(2, 3));
        assert_eq!(v.times_1_s(), SplitInteger::new(1, -1)); // (3-2)(1-s)
        assert_eq!(v.negate(), SplitInteger::new(-3, -2));
        // (3+2s)(1-s) = 1 - s; (a+bs)(c+ds) = (ac+bd)+(ad+bc)s.
        assert_eq!(v * SplitInteger::new(1, -1), SplitInteger::new(1, -1));
        assert_eq!(v.mul_int(-2), SplitInteger::new(-6, -4));
        assert_eq!(v.add_int(1), SplitInteger::new(4, 2));
        // The (1-s) annihilation: (1-s)(1+s) = 0.
        assert!((SplitInteger::new(1, -1) * SplitInteger::new(1, 1)).is_zero());
        let tuple: (i32, i32) = v.into();
        assert_eq!(tuple, (3, 2));
        assert_eq!(SplitInteger::from((0, 1)), SplitInteger::new(0, 1));
    }

    // --- Integral-subsystem classification --------------------------------

    #[test]
    fn a1_nu_half_has_singleton_integral_block() {
        // twisted_family.atlas: p = param(KGB(rf,2),[0],[1]/2) in sl(2,R);
        // gamma = nu = [1]/2 pairs non-integrally with the only coroot.
        let fixture = a1_fixture();
        let rc = fixture.rc();
        let p = param(&rc, 2, &[0], &[1], 2);
        assert_eq!(
            integral_block_scope(&rc, p.gamma()).unwrap(),
            IntegralBlockScope::Singleton
        );
    }

    #[test]
    fn integral_gamma_has_full_block_scope() {
        // q = param(KGB(rf,0),[1],[0]/1) in sl(2,R): gamma = [2].
        let fixture = a1_fixture();
        let rc = fixture.rc();
        let q = param(&rc, 0, &[1], &[0], 1);
        assert_eq!(
            integral_block_scope(&rc, q.gamma()).unwrap(),
            IntegralBlockScope::Full
        );
        // q2 = param(KGB(rfb,0),[0,0],[0,0]/1) in su(2,1): gamma = rho.
        let fixture = a2_compact_fixture();
        let rc = fixture.rc();
        let q2 = param(&rc, 0, &[0, 0], &[0, 0], 1);
        assert_eq!(
            integral_block_scope(&rc, q2.gamma()).unwrap(),
            IntegralBlockScope::Full
        );
    }

    #[test]
    fn a2_half_integral_gamma_is_a_proper_subsystem() {
        // gamma = [1,1]/2: both simple coroot pairings are 1/2, but the
        // highest coroot [1,1] pairs integrally (value 1) — a proper A1
        // subsystem, which is not ported and must be rejected loudly.
        let fixture = a2_compact_fixture();
        let rc = fixture.rc();
        let gamma = rational(&[1, 1], 2);
        assert_eq!(
            integral_block_scope(&rc, &gamma).unwrap(),
            IntegralBlockScope::ProperSubsystem
        );
    }

    // --- twisted_deform / twisted_KL_sum_at_s (twisted_family.atlas) -------

    #[test]
    fn a1_twisted_deform_discrete_series_is_empty() {
        // Oracle (job 3536421): twisted_deform(q) = "Empty sum of standard
        // modules" for q = param(x=0,lambda=[2]/1,nu=[0]/1) — the block
        // element has length 0 (repr.cpp:2435-2436).
        let ctx = a1_block();
        let rc = ctx.primal.rc();
        let q = param(&rc, 0, &[1], &[0], 1);
        let lambda_rho = weight(&[1]);
        let eblock = identity_ext_block(&ctx);
        let y0 = block_element_of(&rc, &ctx, &eblock, &q, &lambda_rho);
        assert_eq!(ctx.block.length(y0), Some(0));
        let singular_orbits = singular_orbits_at(&rc, &eblock, q.gamma()).unwrap();
        let parent = KlSumParent::Full {
            block: &ctx.block,
            lambda_rho: &lambda_rho,
        };
        let terms =
            twisted_deformation_terms(&rc, &parent, &eblock, y0, &singular_orbits, q.gamma())
                .unwrap();
        assert!(terms.is_empty());
    }

    #[test]
    fn a1_twisted_kl_sum_at_s_discrete_series() {
        // Oracle (job 3536421): twisted_KL_sum_at_s(q) =
        // "1*parameter(x=0,lambda=[2]/1,nu=[0]/1) [2]" — both the
        // distinguished (Param) and the external-delta (Param,mat) paths.
        let ctx = a1_block();
        let rc = ctx.primal.rc();
        let q = param(&rc, 0, &[1], &[0], 1);
        let lambda_rho = weight(&[1]);
        let eblock = identity_ext_block(&ctx);
        let y0 = block_element_of(&rc, &ctx, &eblock, &q, &lambda_rho);
        let expected = vec![(q.clone(), SplitInteger::new(1, 0))];
        let parent = KlSumParent::Full {
            block: &ctx.block,
            lambda_rho: &lambda_rho,
        };
        let singular_orbits = singular_orbits_at(&rc, &eblock, q.gamma()).unwrap();
        // Rep_table variant (distinguished involution, parent lengths).
        let column =
            twisted_kl_column_at_s(&rc, &eblock, &parent, y0, q.gamma(), &singular_orbits).unwrap();
        assert_eq!(column, expected);
        // Free-function variant (external delta, extended lengths).
        let ext_y = eblock.element(y0);
        let sum =
            twisted_kl_sum(&rc, &eblock, ext_y, &parent, q.gamma(), &singular_orbits).unwrap();
        assert_eq!(sum, expected);
    }

    #[test]
    fn a2_twisted_deform_trivial_rep_is_empty() {
        // Oracle (job 3536421): twisted_deform(q2) = "Empty sum of standard
        // modules" for q2 = param(x=0,lambda=[1,1]/1,nu=[0,0]/1) in su(2,1).
        let ctx = a2_block();
        let rc = ctx.primal.rc();
        let q2 = param(&rc, 0, &[0, 0], &[0, 0], 1);
        let lambda_rho = weight(&[0, 0]);
        let eblock = identity_ext_block(&ctx);
        let y0 = block_element_of(&rc, &ctx, &eblock, &q2, &lambda_rho);
        assert_eq!(ctx.block.length(y0), Some(0));
        let singular_orbits = singular_orbits_at(&rc, &eblock, q2.gamma()).unwrap();
        let parent = KlSumParent::Full {
            block: &ctx.block,
            lambda_rho: &lambda_rho,
        };
        let terms =
            twisted_deformation_terms(&rc, &parent, &eblock, y0, &singular_orbits, q2.gamma())
                .unwrap();
        assert!(terms.is_empty());
    }

    #[test]
    fn a2_twisted_kl_sum_at_s_trivial_rep() {
        // Oracle (job 3536421): twisted_KL_sum_at_s(q2) =
        // twisted_KL_sum_at_s(q2,[[1,0],[0,1]]) =
        // "1*parameter(x=0,lambda=[1,1]/1,nu=[0,0]/1) [4]".
        let ctx = a2_block();
        let rc = ctx.primal.rc();
        let q2 = param(&rc, 0, &[0, 0], &[0, 0], 1);
        let lambda_rho = weight(&[0, 0]);
        let eblock = identity_ext_block(&ctx);
        let y0 = block_element_of(&rc, &ctx, &eblock, &q2, &lambda_rho);
        let expected = vec![(q2.clone(), SplitInteger::new(1, 0))];
        let parent = KlSumParent::Full {
            block: &ctx.block,
            lambda_rho: &lambda_rho,
        };
        let singular_orbits = singular_orbits_at(&rc, &eblock, q2.gamma()).unwrap();
        let column =
            twisted_kl_column_at_s(&rc, &eblock, &parent, y0, q2.gamma(), &singular_orbits)
                .unwrap();
        assert_eq!(column, expected);
        let ext_y = eblock.element(y0);
        let sum =
            twisted_kl_sum(&rc, &eblock, ext_y, &parent, q2.gamma(), &singular_orbits).unwrap();
        assert_eq!(sum, expected);
    }

    // --- twisted_full_deform (twisted_family.atlas) ------------------------

    /// The twisted_full_deform wrapper loop (atlas-types.w:8237-8247):
    /// finalise, then add each twisted deformation with `Split(0,1)` when
    /// the flips differ and `Split(1,0)` when they agree.
    fn twisted_full_deform(ctx: &ExtRepContext, p: &StandardRepr) -> Vec<(KType, SplitInteger)> {
        let mut result: Vec<(KType, SplitInteger)> = Vec::new();
        for (sr, finalise_flip) in extended_finalise(ctx, p).unwrap() {
            let (def, flip) = twisted_deformation(ctx, &sr, &mut identity_lookup).unwrap();
            let coef = if flip != finalise_flip {
                SplitInteger::new(0, 1)
            } else {
                SplitInteger::new(1, 0)
            };
            add_ktype_multiple(&mut result, &def, coef);
        }
        result
    }

    #[test]
    fn a1_twisted_full_deform_discrete_series() {
        // Oracle (job 3536421): twisted_full_deform(q) =
        // "1* K_type(x=0, lambda=[2]/1) [2]".
        let fixture = a1_fixture();
        let rc = fixture.rc();
        let q = param(&rc, 0, &[1], &[0], 1);
        assert!(matches!(q.is_final(&rc), Ok(true)));
        assert!(rc.is_delta_fixed(&q));
        assert!(rc.reducibility_points(&q).unwrap().is_empty());
        let ctx =
            ExtRepContext::new(&rc, LatticeInvolution::identity(rc.datum()).unwrap()).unwrap();
        let terms = twisted_full_deform(&ctx, &q);
        let expected = KType::sr_k(&rc, KgbId(0), &weight(&[1])).unwrap();
        assert_eq!(expected.height(), 2); // the oracle's [2]
        assert_eq!(terms, vec![(expected, SplitInteger::new(1, 0))]);
    }

    #[test]
    fn cancellable_twisted_deformation_drops_partial_work() {
        let fixture = a1_fixture();
        let rc = fixture.rc();
        let q = param(&rc, 0, &[1], &[0], 1);
        let ctx =
            ExtRepContext::new(&rc, LatticeInvolution::identity(rc.datum()).unwrap()).unwrap();
        let mut probes = 0;
        let mut cancel_after_entry = || {
            probes += 1;
            probes > 1
        };
        let result = twisted_deformation_with_cancel(
            &ctx,
            &q,
            &mut identity_lookup,
            &mut cancel_after_entry,
        )
        .expect("cancellation is not a structural error");
        assert_eq!(result, None);
    }

    #[test]
    fn a2_twisted_full_deform_trivial_rep() {
        // Oracle (job 3536421): twisted_full_deform(q2) =
        // "1* K_type(x=0, lambda=[1,1]/1) [4]".
        let fixture = a2_compact_fixture();
        let rc = fixture.rc();
        let q2 = param(&rc, 0, &[0, 0], &[0, 0], 1);
        assert!(matches!(q2.is_final(&rc), Ok(true)));
        assert!(rc.is_delta_fixed(&q2));
        assert!(rc.reducibility_points(&q2).unwrap().is_empty());
        let ctx =
            ExtRepContext::new(&rc, LatticeInvolution::identity(rc.datum()).unwrap()).unwrap();
        let terms = twisted_full_deform(&ctx, &q2);
        let expected = KType::sr_k(&rc, KgbId(0), &weight(&[0, 0])).unwrap();
        assert_eq!(expected.height(), 4); // the oracle's [4]
        assert_eq!(terms, vec![(expected, SplitInteger::new(1, 0))]);
    }

    #[test]
    fn a2_twisted_drivers_above_length_zero_smoke() {
        // No oracle value pins a multi-term twisted computation; this runs
        // the drivers at the highest delta-fixed block element so the
        // ported upstream invariants (the remainder/accumulator inverse
        // relation, repr.cpp:2499, and the finals triangularity) fire.
        let ctx = a2_block();
        let rc = ctx.primal.rc();
        let eblock = identity_ext_block(&ctx);
        let gamma = rational(&[1, 1], 1); // rho, regular
        let lambda_rho = weight(&[0, 0]);
        let y0 = (0..ctx.block.size())
            .rev()
            .find(|&z| eblock.is_present(z))
            .expect("some delta-fixed block element");
        assert!(ctx.block.length(y0).unwrap() > 0);
        let singular_orbits = singular_orbits_at(&rc, &eblock, &gamma).unwrap();
        let parent = KlSumParent::Full {
            block: &ctx.block,
            lambda_rho: &lambda_rho,
        };
        let terms =
            twisted_deformation_terms(&rc, &parent, &eblock, y0, &singular_orbits, &gamma).unwrap();
        for (sr, _) in &terms {
            assert_eq!(sr.gamma(), &gamma);
        }
        // The column sum contains y itself with coefficient 1 (P_{y,y}=1).
        let sr_y = rc
            .sr_gamma(ctx.block.x(y0).unwrap(), &lambda_rho, &gamma)
            .unwrap();
        let sum =
            twisted_kl_column_at_s(&rc, &eblock, &parent, y0, &gamma, &singular_orbits).unwrap();
        assert!(sum.contains(&(sr_y.clone(), SplitInteger::new(1, 0))));
        let ext_y = eblock.element(y0);
        let sum_free =
            twisted_kl_sum(&rc, &eblock, ext_y, &parent, &gamma, &singular_orbits).unwrap();
        assert!(sum_free.contains(&(sr_y, SplitInteger::new(1, 0))));
    }

    // --- block_deform (block_deform.atlas) ----------------------------------
    /// The `deform(p)` accumulator of block_deform.atlas, replayed from the
    /// oracle (job 3536583): d = (1-s)*param(x=2,nu=0) + (1-s)*param(x=0,
    /// nu=0) at lambda=[1,1], both of height 4.
    fn block_deform_accumulator(rc: &RepContext) -> Vec<(StandardRepr, SplitInteger)> {
        vec![
            (param(rc, 2, &[0, 0], &[0, 0], 1), SplitInteger::new(1, -1)),
            (param(rc, 0, &[0, 0], &[0, 0], 1), SplitInteger::new(1, -1)),
        ]
    }

    #[test]
    fn a2_block_deform_height_bounds() {
        // Oracle (job 3536583) for p = param(x=3,lambda=[1,1],nu=[-1,2]/2)
        // in su(2,1): bounds 0 and 3 keep the height-4 terms in the
        // accumulator ("(Empty, d)"); bounds 4 and 5 (and the negative
        // bound's maximal level) move both to the deformed component
        // ("(d, Empty)").
        let ctx = a2_block();
        let rc = ctx.primal.rc();
        let p = param(&rc, 3, &[0, 0], &[1, 1], 1);
        let p = p.made_dominant(&rc).unwrap();
        assert_eq!(p, param(&rc, 3, &[0, 0], &[-1, 2], 2)); // oracle display
        let lambda_rho = rc.lambda_rho(&p).unwrap();
        assert_eq!(lambda_rho, weight(&[0, 0]));
        let gamma = p.gamma().clone();
        assert_eq!(gamma, rational(&[1, 1], 1)); // rho, regular

        let accumulator = block_deform_accumulator(&rc);
        for (sr, _) in &accumulator {
            assert_eq!(sr.height(), 4); // the oracle's [4]
        }

        for bound in [0, 3] {
            let (deformed, consumed) = block_deformation_to_height(
                &rc,
                &ctx.block,
                &gamma,
                &lambda_rho,
                bound,
                &accumulator,
            )
            .unwrap();
            assert_eq!(consumed, vec![false, false], "bound {bound}");
            assert!(
                deformed.iter().all(|(_, coef)| coef.is_zero()),
                "bound {bound}: {deformed:?}"
            );
        }

        for bound in [4, 5, u32::MAX] {
            let (deformed, consumed) = block_deformation_to_height(
                &rc,
                &ctx.block,
                &gamma,
                &lambda_rho,
                bound,
                &accumulator,
            )
            .unwrap();
            assert_eq!(consumed, vec![true, true], "bound {bound}");
            let mut moved: Vec<(StandardRepr, SplitInteger)> = deformed
                .into_iter()
                .filter(|(_, coef)| !coef.is_zero())
                .collect();
            let mut expected = accumulator.clone();
            let by_x = |a: &(StandardRepr, _), b: &(StandardRepr, _)| a.0.x().cmp(&b.0.x());
            moved.sort_by(by_x);
            expected.sort_by(by_x);
            assert_eq!(moved, expected, "bound {bound}");
        }
    }

    #[test]
    fn block_deform_consumes_first_occurrence_only() {
        // Upstream `queue.find(q)` + `queue.erase`: each matching block
        // element consumes exactly one accumulator occurrence, the FIRST
        // unconsumed one. With `occurrences` matches in the block and
        // `occurrences + 1` duplicate terms, every copy but the last is
        // consumed — in order.
        let ctx = a2_block();
        let rc = ctx.primal.rc();
        let p = param(&rc, 3, &[0, 0], &[1, 1], 1);
        let p = p.made_dominant(&rc).unwrap();
        let lambda_rho = rc.lambda_rho(&p).unwrap();
        let gamma = p.gamma().clone();
        let term = param(&rc, 2, &[0, 0], &[0, 0], 1);

        let occurrences = (0..ctx.block.size())
            .map(|z| {
                rc.sr_gamma(ctx.block.x(z).unwrap(), &lambda_rho, &gamma)
                    .unwrap()
            })
            .filter(|q| *q == term)
            .count();
        assert!(occurrences >= 1);

        let accumulator: Vec<(StandardRepr, SplitInteger)> = (1..=(occurrences + 1) as i32)
            .map(|c| (term.clone(), SplitInteger::new(c, -c)))
            .collect();
        let (_, consumed) = block_deformation_to_height(
            &rc,
            &ctx.block,
            &gamma,
            &lambda_rho,
            u32::MAX,
            &accumulator,
        )
        .unwrap();
        let expected: Vec<bool> = (0..=occurrences).map(|i| i < occurrences).collect();
        assert_eq!(consumed, expected);
    }

    #[test]
    fn dual_kl_table_is_cached_per_block_identity() {
        // Two calls on the same block share the cached table; a different
        // block gets its own (the a1 block must not see the a2 table).
        let a2 = a2_block();
        let first = crate::rep_table::with_dual_kl_table(&a2.block, |kl| {
            Ok(kl as *mut _ as usize)
        })
        .unwrap();
        let second = crate::rep_table::with_dual_kl_table(&a2.block, |kl| {
            Ok(kl as *mut _ as usize)
        })
        .unwrap();
        assert_eq!(first, second);

        let a1 = a1_block();
        let other = crate::rep_table::with_dual_kl_table(&a1.block, |kl| {
            Ok(kl as *mut _ as usize)
        })
        .unwrap();
        assert_ne!(first, other);
    }

    #[test]
    fn dual_kl_table_callback_rejects_same_thread_nesting() {
        // The ActiveKlCallback contract of with_kl_table applies here too.
        let ctx = a2_block();
        let result = crate::rep_table::with_dual_kl_table(&ctx.block, |_| {
            crate::rep_table::with_dual_kl_table(&ctx.block, |_| Ok(()))
        });
        assert_eq!(
            result,
            Err(StructureError::RepInvariantViolation {
                invariant: "representation block KL table nested callback",
            })
        );
        // The guard clears on drop: a sequential call succeeds.
        assert!(crate::rep_table::with_dual_kl_table(&ctx.block, |_| Ok(())).is_ok());
    }
}
