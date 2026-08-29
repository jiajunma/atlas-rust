//! The `ext_param`/`star` layer of extended blocks.
//!
//! This module ports the parameter layer of upstream `gkmod/ext_block.cpp`:
//! the [`ExtRepContext`] wrapper (`repr::Ext_rep_context`, repr.h:682-714 and
//! repr.cpp:2786-2836), the [`ExtParam`] value type (`ext_param`,
//! ext_block.h:293-364 and ext_block.cpp:2283-2420), the comparison and
//! alignment helpers `same_standard_reps`/`same_sign`/`z_align`/`level_a`
//! (ext_block.cpp:910-985), the conjugation word `fixed_conjugate_simple`
//! (ext_block.cpp:525-552), the cross action `complex_cross`
//! (ext_block.cpp:858-907), and the central [`star`] computation
//! (ext_block.cpp:990-1705).
//!
//! On top of that sit the three parameter-level finalisation drivers
//! [`extended_restrict_to_k`] (ext_block.cpp:2435-2547),
//! [`extended_finalise`] (ext_block.cpp:2598-2721), and
//! [`scaled_extended_finalise`] (ext_block.cpp:2736-2807), whose queue loops
//! replay folded-orbit reflections and `star` descents while tracking the
//! net flip against the default extension.
//!
//! Two porting conventions matter for fidelity:
//!
//! - All `Weight`/`Coweight`/`int` arithmetic uses two's-complement
//!   WRAPPING i32 operations, matching upstream `int` arithmetic (the same
//!   convention the `matreduc` port validated bit-for-bit against the
//!   compiled oracle). Rational-weight numerators stay i64, matching the
//!   crate's `RationalWeight` storage.
//! - Upstream `assert` conditions become `debug_assert` (or the
//!   debug-only [`validate`] port), exactly as upstream compiles them away
//!   under `NDEBUG`; genuine data-dependent failures surface as
//!   [`StructureError`].

use std::collections::{BTreeSet, VecDeque};

use malachite::{Integer, Rational};

use crate::ext_block::{fold_orbits, DescValue, ExtGen, ExtGenKind, StarOracle};
use crate::lattice::{checked_sub_weights, RationalWeight};
use crate::matreduc::{find_solution, has_solution, in_left_image, in_right_image, IntMatrix};
use crate::root_reflection::reflection_word;
#[cfg(debug_assertions)]
use crate::twisted_involution::compose_matrices;
use crate::{
    BlockGraph, Coweight, InvolutionId, InvolutionTable, KType, KgbId, LatticeInvolution,
    ModTwoVector, PartialBlock, RepContext, RootId, RootKind, RootSystem, StandardRepr,
    StructureError, Weight,
};

// ---------------------------------------------------------------------------
// Wrapping i32 coordinate arithmetic (upstream `int` semantics).
// ---------------------------------------------------------------------------

/// Coordinatewise wrapping sum.
fn vec_add(left: &[i32], right: &[i32]) -> Vec<i32> {
    debug_assert_eq!(left.len(), right.len());
    left.iter()
        .zip(right)
        .map(|(&a, &b)| a.wrapping_add(b))
        .collect()
}

/// Coordinatewise wrapping difference.
fn vec_sub(left: &[i32], right: &[i32]) -> Vec<i32> {
    debug_assert_eq!(left.len(), right.len());
    left.iter()
        .zip(right)
        .map(|(&a, &b)| a.wrapping_sub(b))
        .collect()
}

/// Wrapping scalar multiple.
fn vec_scaled(vector: &[i32], factor: i32) -> Vec<i32> {
    vector.iter().map(|&a| a.wrapping_mul(factor)).collect()
}

/// `acc += vector * factor`, wrapping.
fn vec_add_scaled(acc: &mut [i32], vector: &[i32], factor: i32) {
    debug_assert_eq!(acc.len(), vector.len());
    for (entry, &a) in acc.iter_mut().zip(vector) {
        *entry = entry.wrapping_add(a.wrapping_mul(factor));
    }
}

/// Wrapping dot product (upstream `Vector<int>::dot`).
fn vec_dot(left: &[i32], right: &[i32]) -> i32 {
    debug_assert_eq!(left.len(), right.len());
    left.iter()
        .zip(right)
        .fold(0_i32, |sum, (&a, &b)| sum.wrapping_add(a.wrapping_mul(b)))
}

/// The root vector of `id` as a plain coordinate slice.
fn root_coords(system: &RootSystem, id: RootId) -> Result<&[i32], StructureError> {
    Ok(system
        .root(id)
        .ok_or(StructureError::IndexOutOfRange {
            index: id.index(),
            upper_bound: system.roots().len(),
        })?
        .as_slice())
}

/// The coroot vector of `id` as a plain coordinate slice.
fn coroot_coords(system: &RootSystem, id: RootId) -> Result<&[i32], StructureError> {
    Ok(system
        .coroot(id)
        .ok_or(StructureError::IndexOutOfRange {
            index: id.index(),
            upper_bound: system.roots().len(),
        })?
        .as_slice())
}

/// The root number of `-root(id)` (upstream `RootSystem::rootMinus`).
fn root_minus(system: &RootSystem, id: RootId) -> Result<RootId, StructureError> {
    let negated = vec_scaled(root_coords(system, id)?, -1);
    system
        .id_of(&Weight::new(negated))
        .ok_or(StructureError::RepInvariantViolation {
            invariant: "root negation",
        })
}

/// The positive representative of a root (upstream `make_positive`).
fn make_positive(system: &RootSystem, id: RootId) -> Result<RootId, StructureError> {
    if system.is_positive(id) == Some(true) {
        Ok(id)
    } else {
        root_minus(system, id)
    }
}

/// The image of root `id` under the simple reflection `generator`
/// (upstream `RootSystem::simple_reflected_root`).
fn simple_reflect_root(
    system: &RootSystem,
    generator: usize,
    id: RootId,
) -> Result<RootId, StructureError> {
    let simple = system.simple_root_ids().get(generator).copied().ok_or(
        StructureError::IndexOutOfRange {
            index: generator,
            upper_bound: system.simple_root_ids().len(),
        },
    )?;
    let root = root_coords(system, id)?;
    let simple_root = root_coords(system, simple)?;
    let simple_coroot = coroot_coords(system, simple)?;
    let factor = vec_dot(root, simple_coroot);
    let mut image = root.to_vec();
    vec_add_scaled(&mut image, simple_root, -factor);
    system
        .id_of(&Weight::new(image))
        .ok_or(StructureError::RepInvariantViolation {
            invariant: "simple root reflection",
        })
}

/// Upstream `RootDatum::reflect` (rootdata.h:584-585): `v -= alpha *
/// <coroot(alpha), v>`, on raw coordinates.
fn reflect_coords(system: &RootSystem, beta: RootId, v: &mut [i32]) -> Result<(), StructureError> {
    let factor = vec_dot(coroot_coords(system, beta)?, v);
    let root = root_coords(system, beta)?.to_vec();
    vec_add_scaled(v, &root, -factor);
    Ok(())
}

/// Upstream `RootDatum::coreflect` (rootdata.h:597-598): `v -=
/// coroot(alpha) * (<root(alpha), v> + d)`.
fn coreflect_coords(
    system: &RootSystem,
    beta: RootId,
    v: &mut [i32],
    d: i32,
) -> Result<(), StructureError> {
    let factor = vec_dot(root_coords(system, beta)?, v).wrapping_add(d);
    let coroot = coroot_coords(system, beta)?.to_vec();
    vec_add_scaled(v, &coroot, -factor);
    Ok(())
}

/// Upstream `RatWeight::dot` (ratvec.h:160-165): the numerator pairing
/// divided by the denominator with TRUNCATING integer division, then
/// narrowed to `int`.
fn rat_dot(value: &RationalWeight, coweight: &[i32]) -> i32 {
    debug_assert_eq!(value.rank(), coweight.len());
    let numerator = value
        .numerator()
        .iter()
        .zip(coweight)
        .fold(0_i64, |sum, (&a, &b)| {
            sum.wrapping_add(a.wrapping_mul(i64::from(b)))
        });
    numerator.wrapping_div(value.denominator()) as i32
}

/// Upstream `RatWeight::integer_diff<int>`: the exact integral difference
/// of two rational weights, narrowed coordinatewise to `int`.
pub(crate) fn integer_diff(
    left: &RationalWeight,
    right: &RationalWeight,
) -> Result<Vec<i32>, StructureError> {
    let difference = left.sub(right)?;
    Ok(difference
        .integral_coordinates()?
        .iter()
        .map(|&entry| entry as i32)
        .collect())
}

/// Upstream `RatCoweight::dot(Weight)`: the exact rational pairing,
/// truncated to `int` (the invariants at the call sites make it integral).
fn rational_coweight_dot(value: &[Rational], weight: &[i32]) -> Result<i32, StructureError> {
    debug_assert_eq!(value.len(), weight.len());
    let mut total = Rational::from(0);
    for (coordinate, &entry) in value.iter().zip(weight) {
        total += coordinate * Rational::from(entry);
    }
    // Malachite's `Rational` stores an unsigned numerator with a separate
    // sign; reapply the sign before truncating (upstream `int` semantics).
    let negative = total < 0;
    let (numerator, denominator) = total.into_numerator_and_denominator();
    let magnitude = i64::try_from(&numerator).map_err(|_| StructureError::ArithmeticOverflow)?;
    let numerator = if negative { -magnitude } else { magnitude };
    let denominator =
        i64::try_from(&denominator).map_err(|_| StructureError::ArithmeticOverflow)?;
    Ok(numerator.wrapping_div(denominator) as i32)
}

/// `gamma_lambda` reflected in root `beta` at numerator level (upstream
/// `rd.reflect(beta, E.gamma_lambda.numerator())`, ext_block.cpp:880).
fn reflect_rational(
    system: &RootSystem,
    beta: RootId,
    value: &RationalWeight,
) -> Result<RationalWeight, StructureError> {
    let root = root_coords(system, beta)?;
    let coroot = coroot_coords(system, beta)?;
    let factor = value
        .numerator()
        .iter()
        .zip(coroot)
        .fold(0_i64, |sum, (&a, &b)| {
            sum.wrapping_add(a.wrapping_mul(i64::from(b)))
        });
    let numerator: Vec<i64> = value
        .numerator()
        .iter()
        .zip(root)
        .map(|(&a, &b)| a.wrapping_sub(factor.wrapping_mul(i64::from(b))))
        .collect();
    RationalWeight::new(numerator, value.denominator())
}

/// Upstream `RootSystem::pos_to_neg` (rootdata.cpp:1415-1439): the set of
/// positive roots made negative by left multiplication by the word, as
/// (positive) root numbers in ascending order.
fn pos_to_neg(system: &RootSystem, word: &[usize]) -> Result<Vec<RootId>, StructureError> {
    let mut current: BTreeSet<usize> = BTreeSet::new();
    for &generator in word {
        let simple = system.simple_root_ids()[generator];
        let mut tmp = BTreeSet::new();
        for &member in &current {
            let image = simple_reflect_root(system, generator, RootId::from_usize(member))?;
            // The simple root maps to its negative; upstream's permutation
            // of positive roots fixes it as a point, the flip below handles
            // its membership.
            let positive = make_positive(system, image)?;
            tmp.insert(positive.index());
        }
        if !tmp.remove(&simple.index()) {
            tmp.insert(simple.index());
        }
        current = tmp;
    }
    Ok(current.into_iter().map(RootId::from_usize).collect())
}

/// Upstream `RootSystem::sum_is_root` (rootdata.h:355-356): `alpha + beta`
/// is a root iff `beta` is not among the minimal roots for `-alpha`.
fn sum_is_root(system: &RootSystem, alpha: RootId, beta: RootId) -> bool {
    let Ok(minus) = root_minus(system, alpha) else {
        return false;
    };
    match system.min_roots_for(minus) {
        Some(minimal) => !minimal.contains(beta),
        None => false,
    }
}

/// Upstream `TwistedWeylGroup::prod(WeylWord, TwistedInvolution)`
/// (weyl.h:329-331 `left_multiply(w, ww)`): the product
/// `s_{w[0]} * ... * s_{w[k]} * tw`.
fn left_word_lookup(
    table: &InvolutionTable,
    involution: InvolutionId,
    word: &[usize],
) -> Result<InvolutionId, StructureError> {
    table
        .weyl_left_word_lookup(involution, word)?
        .ok_or(StructureError::RepInvariantViolation {
            invariant: "extended parameter Cartan lookup",
        })
}

fn reflected_involution(
    table: &InvolutionTable,
    involution: InvolutionId,
    words: &[Vec<usize>],
) -> Result<InvolutionId, StructureError> {
    let total_len = words.iter().map(Vec::len).sum();
    let mut word = Vec::new();
    word.try_reserve_exact(total_len)
        .map_err(|_| StructureError::AllocationFailed {
            requested: total_len,
        })?;
    for part in words {
        word.extend_from_slice(part);
    }
    left_word_lookup(table, involution, &word)
}

// ---------------------------------------------------------------------------
// ExtRepContext (upstream repr::Ext_rep_context).
// ---------------------------------------------------------------------------

/// A [`RepContext`] extended by the twisting involution `delta` (upstream
/// `repr::Ext_rep_context`, repr.h:682-714): `delta` as a root permutation
/// with its fixed-root set, and the induced simple-generator twist.
pub struct ExtRepContext<'a> {
    rc: &'a RepContext<'a>,
    delta: LatticeInvolution,
    /// `pi_delta`: delta's permutation of the enumerated roots.
    pi_delta: Vec<RootId>,
    /// `delta_fixed_roots`: per-root flag for `pi_delta[root] == root`.
    delta_fixed: Vec<bool>,
    /// The induced twist of the simple generators (repr.cpp:2795-2796).
    twist: Vec<usize>,
}

impl<'a> ExtRepContext<'a> {
    /// Build the extended context; `delta` must permute the root system
    /// and induce a permutation of the simple roots (upstream
    /// `rootPermutation(delta)` plus the `twist` loop).
    pub fn new(rc: &'a RepContext<'a>, delta: LatticeInvolution) -> Result<Self, StructureError> {
        let system = rc.root_system();
        let mut pi_delta = Vec::with_capacity(system.roots().len());
        let mut delta_fixed = Vec::with_capacity(system.roots().len());
        for (id, root, coroot) in system.entries() {
            let image_root = delta.act_on_weight(root)?;
            let image = system
                .id_of(&image_root)
                .ok_or(StructureError::InvalidRootAutomorphism)?;
            let image_coroot = system
                .coroot(image)
                .ok_or(StructureError::IndexOutOfRange {
                    index: image.index(),
                    upper_bound: system.roots().len(),
                })?;
            if delta.act_on_coweight(coroot)? != *image_coroot {
                return Err(StructureError::InvalidRootDatumAutomorphism);
            }
            delta_fixed.push(image == id);
            pi_delta.push(image);
        }
        let simple_ids = system.simple_root_ids();
        let mut twist = Vec::with_capacity(simple_ids.len());
        for &simple in simple_ids {
            let position = simple_ids
                .iter()
                .position(|&candidate| candidate == pi_delta[simple.index()])
                .ok_or(StructureError::InvalidBasedAutomorphism)?;
            twist.push(position);
        }
        Ok(Self {
            rc,
            delta,
            pi_delta,
            delta_fixed,
            twist,
        })
    }

    pub fn rc(&self) -> &'a RepContext<'a> {
        self.rc
    }

    pub fn delta(&self) -> &LatticeInvolution {
        &self.delta
    }

    /// Upstream `Ext_rep_context::delta_of`.
    pub fn delta_of(&self, alpha: RootId) -> RootId {
        self.pi_delta[alpha.index()]
    }

    /// Membership in upstream `delta_fixed_roots`.
    pub fn is_delta_fixed_root(&self, alpha: RootId) -> bool {
        self.delta_fixed[alpha.index()]
    }

    /// Upstream `Ext_rep_context::twisted`.
    pub fn twisted(&self, generator: usize) -> usize {
        self.twist[generator]
    }

    /// Upstream `Ext_rep_context::to_simple_shift` (repr.h:706-708).
    pub fn to_simple_shift(
        &self,
        theta: InvolutionId,
        theta_p: InvolutionId,
        roots: &[RootId],
    ) -> Result<Weight, StructureError> {
        self.rc.to_simple_shift(theta, theta_p, roots)
    }

    /// Upstream `Ext_rep_context::is_very_complex` (repr.cpp:2804-2813):
    /// whether the positive root `alpha` has `theta(alpha)` different from
    /// both `alpha` and `delta(alpha)`, up to sign.
    pub fn is_very_complex(
        &self,
        theta: InvolutionId,
        alpha: RootId,
    ) -> Result<bool, StructureError> {
        let system = self.rc.root_system();
        debug_assert_eq!(system.is_positive(alpha), Some(true));
        let image = self.rc.root_involution_data(theta)?.image(alpha).ok_or(
            StructureError::IndexOutOfRange {
                index: alpha.index(),
                upper_bound: system.roots().len(),
            },
        )?;
        let positive_image = make_positive(system, image)?;
        Ok(positive_image != alpha && positive_image != self.delta_of(alpha))
    }

    /// Upstream `Ext_rep_context::shift_flip` (repr.cpp:2824-2836): whether
    /// delta acts by `-1` on the wedge over the positive-to-negative set
    /// `roots` that change very-complex status between the two involutions.
    pub fn shift_flip(
        &self,
        theta: InvolutionId,
        theta_p: InvolutionId,
        roots: &[RootId],
    ) -> Result<bool, StructureError> {
        let system = self.rc.root_system();
        let mut count = 0_u32;
        for &root in roots {
            if self.is_delta_fixed_root(root) {
                continue; // delta-fixed roots do not contribute
            }
            if self.is_very_complex(theta, root)? != self.is_very_complex(theta_p, root)?
                && !sum_is_root(system, root, self.delta_of(root))
            {
                count += 1;
            }
        }
        debug_assert!(count.is_multiple_of(2)); // pos_to_neg sets are delta-stable
        Ok(!count.is_multiple_of(4))
    }
}

// ---------------------------------------------------------------------------
// ExtParam (upstream ext_block::ext_param).
// ---------------------------------------------------------------------------

/// An extended parameter (upstream `ext_block::ext_param`,
/// ext_block.h:293-343): a twisted involution `tw` (defining `theta`), the
/// coweight `l` (with `tw` a global Tits element lifting `t`), the rational
/// `gamma_lambda`, a solution `tau` of `(1-theta)*tau = (1-delta)
/// gamma_lambda`, a solution `t` of `t*(1-theta) = l*(delta-1)`, and the
/// flipping-representation flag.
///
/// Unlike upstream the context is not stored; methods take the
/// [`ExtRepContext`] explicitly (the crate's borrow discipline).
#[derive(Clone, Debug)]
pub struct ExtParam {
    involution: InvolutionId,
    l: Coweight,
    gamma_lambda: RationalWeight,
    tau: Weight,
    t: Coweight,
    flipped: bool,
}

impl ExtParam {
    /// The raw constructor (ext_block.cpp:2283-2294); invariants are
    /// asserted through [`validate`] in debug builds.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        ctx: &ExtRepContext,
        involution: InvolutionId,
        gamma_lambda: RationalWeight,
        tau: Weight,
        l: Coweight,
        t: Coweight,
        flipped: bool,
    ) -> Self {
        let result = Self {
            involution,
            l,
            gamma_lambda,
            tau,
            t,
            flipped,
        };
        validate(ctx, &result);
        result
    }

    /// The default-extension constructor at KGB element `x`
    /// (ext_block.cpp:2297-2311): `tw` from `x`, `l = ell(kgb, x)`, `tau`
    /// and `t` elected solutions.
    pub fn at(
        ctx: &ExtRepContext,
        x: KgbId,
        gamma_lambda: RationalWeight,
        flipped: bool,
    ) -> Result<Self, StructureError> {
        let rc = ctx.rc();
        let rank = rc.rank();
        let involution = rc.involution_of(x)?;
        let record = rc
            .table()
            .record(involution)
            .ok_or(StructureError::IndexOutOfRange {
                index: involution.0,
                upper_bound: rc.table().involution_count(),
            })?;
        let theta = record.theta();

        // l = ell(kgb, x): g_rho_check - torus_factor(x), integral.
        let g_rho_check = rc.g_rho_check();
        let torus_factor = rc.graph().torus_factor(x, rc.table())?;
        let mut l_coordinates = Vec::with_capacity(rank);
        for (g, tf) in g_rho_check
            .coordinates()
            .iter()
            .zip(torus_factor.coordinates())
        {
            let difference = g - tf;
            let integer = Integer::try_from(&difference).map_err(|_| {
                StructureError::RepInvariantViolation {
                    invariant: "ext_param ell integrality",
                }
            })?;
            // Upstream narrows the (big-integer) numerator to `int`.
            let wide = i64::try_from(&integer).map_err(|_| StructureError::ArithmeticOverflow)?;
            l_coordinates.push(wide as i32);
        }
        let l = Coweight::new(l_coordinates);

        // tau: solution of (1-theta)*tau = (1-delta)*gamma_lambda.
        let delta_gamma = gamma_lambda.apply_matrix(ctx.delta().weight_matrix())?;
        let right = integer_diff(&gamma_lambda, &delta_gamma)?;
        let mut one_minus_theta = Vec::with_capacity(rank * rank);
        for (i, row) in theta.weight_matrix().iter().enumerate() {
            for (j, &entry) in row.iter().enumerate() {
                let diagonal: i32 = if i == j { 1 } else { 0 };
                one_minus_theta.push(diagonal.wrapping_sub(entry));
            }
        }
        let tau = find_solution(
            &IntMatrix::from_entries(rank, rank, one_minus_theta),
            &right,
        )
        .ok_or(StructureError::RepInvariantViolation {
            invariant: "ext_param tau solution",
        })?;

        // t: solution of t*(1-theta) = l*(delta-1), i.e.
        // (theta_coweight+I)*t = (delta_coweight-I)*l.
        let delta_l = ctx.delta().act_on_coweight(&l)?;
        let right = vec_sub(delta_l.as_slice(), l.as_slice());
        let mut theta_plus_one = Vec::with_capacity(rank * rank);
        for (i, row) in theta.coweight_matrix().iter().enumerate() {
            for (j, &entry) in row.iter().enumerate() {
                theta_plus_one.push(entry.wrapping_add(if i == j { 1 } else { 0 }));
            }
        }
        let t = find_solution(&IntMatrix::from_entries(rank, rank, theta_plus_one), &right).ok_or(
            StructureError::RepInvariantViolation {
                invariant: "ext_param t solution",
            },
        )?;

        Ok(Self::new(
            ctx,
            involution,
            gamma_lambda,
            Weight::new(tau),
            l,
            Coweight::new(t),
            flipped,
        ))
    }

    pub fn involution_id(&self) -> InvolutionId {
        self.involution
    }

    /// The lattice involution `theta` defined by `tw` (upstream
    /// `ext_param::theta`, ext_block.cpp:2395-2396).
    pub fn theta<'a>(
        &self,
        ctx: &ExtRepContext<'a>,
    ) -> Result<&'a LatticeInvolution, StructureError> {
        let id = self.involution;
        Ok(ctx
            .rc()
            .table()
            .record(id)
            .ok_or(StructureError::IndexOutOfRange {
                index: id.0,
                upper_bound: ctx.rc().table().involution_count(),
            })?
            .theta())
    }

    /// The involution-table number of `tw`.
    pub fn theta_id(&self, ctx: &ExtRepContext) -> Result<InvolutionId, StructureError> {
        ctx.rc()
            .table()
            .record(self.involution)
            .ok_or(StructureError::IndexOutOfRange {
                index: self.involution.0,
                upper_bound: ctx.rc().table().involution_count(),
            })?;
        Ok(self.involution)
    }

    pub fn l(&self) -> &Coweight {
        &self.l
    }

    pub fn gamma_lambda(&self) -> &RationalWeight {
        &self.gamma_lambda
    }

    pub fn tau(&self) -> &Weight {
        &self.tau
    }

    pub fn t(&self) -> &Coweight {
        &self.t
    }

    pub fn is_flipped(&self) -> bool {
        self.flipped
    }

    /// Upstream `ext_param::flip` (ext_block.h:332): toggle when `whether`.
    pub fn flip(&mut self, whether: bool) {
        self.flipped = whether != self.flipped;
    }

    /// Upstream `ext_param::x` (ext_block.cpp:2392-2393): reconstruct the
    /// KGB element from `(tw, l mod 2)`.
    pub fn x(&self, ctx: &ExtRepContext) -> Result<KgbId, StructureError> {
        let rc = ctx.rc();
        let involution = self.theta_id(ctx)?;
        let ones = self
            .l
            .as_slice()
            .iter()
            .enumerate()
            .filter_map(|(index, &coordinate)| (coordinate % 2 != 0).then_some(index));
        let torus = ModTwoVector::from_ones(self.l.rank(), ones)?;
        rc.graph().lookup(rc.table(), involution, torus)?.ok_or(
            StructureError::RepInvariantViolation {
                invariant: "ext_param x lookup",
            },
        )
    }

    /// Upstream `ext_param::restrict_mod` (ext_block.cpp:2399-2402): the
    /// parameter modulo `X^*`, as `(x, gamma_lambda real_unique)`.
    pub fn restrict_mod(
        &self,
        ctx: &ExtRepContext,
    ) -> Result<(KgbId, RationalWeight), StructureError> {
        let x = self.x(ctx)?;
        Ok((x, ctx.rc().build_srm(x, &self.gamma_lambda)?))
    }

    /// Upstream `ext_param::restrict` (ext_block.cpp:2405-2410): the
    /// standard parameter at infinitesimal character `gamma`.
    pub fn restrict(
        &self,
        ctx: &ExtRepContext,
        gamma: &RationalWeight,
    ) -> Result<StandardRepr, StructureError> {
        let rc = ctx.rc();
        let gamma_rho = gamma.sub(rc.rho())?;
        let lambda_rho = integer_diff(&gamma_rho, &self.gamma_lambda)?;
        rc.sr_gamma(self.x(ctx)?, &Weight::new(lambda_rho), gamma)
    }
}

/// Upstream `ext_block::default_extend` at a [`StandardRepr`]
/// (ext_block.cpp:2341-2345).
pub fn default_extend(ctx: &ExtRepContext, sr: &StandardRepr) -> Result<ExtParam, StructureError> {
    debug_assert!(ctx.rc().is_fixed(sr, ctx.delta()));
    let (x, gamma_lambda) = ctx.rc().mod_reduce(sr)?;
    ExtParam::at(ctx, x, gamma_lambda, false)
}

/// Upstream `ext_block::default_extend` at a `StandardReprMod` given as
/// `(x, gamma_lambda)` (ext_block.cpp:2348-2351); `gamma_lambda` must
/// already be `real_unique` at `x`.
pub fn default_extend_srm(
    ctx: &ExtRepContext,
    x: KgbId,
    gamma_lambda: RationalWeight,
) -> Result<ExtParam, StructureError> {
    ExtParam::at(ctx, x, gamma_lambda, false)
}

/// Upstream `shifted_default_extension` (ext_block.h:352-361).
pub fn shifted_default_extension(
    ctx: &ExtRepContext,
    sr: &StandardRepr,
    new_gamma: &RationalWeight,
) -> Result<ExtParam, StructureError> {
    let mut result = default_extend(ctx, sr)?;
    let shift = new_gamma.sub(sr.gamma())?;
    // Upstream keeps `assert((1+theta_x)*shift == 0)` here (ext_block.h:352-361),
    // but the oracle is compiled with -DNDEBUG (upstream Makefile) so it never
    // fires. The precondition is not implied for shift_flip-reachable calls
    // (theta_x = +1 at a compact Cartan makes any nonzero shift trip it), so a
    // debug_assert would panic where the reference returns a well-defined
    // result. Intentionally omitted to keep debug builds oracle-compatible.
    result.gamma_lambda = result.gamma_lambda.add(&shift)?;
    Ok(result)
}

/// Upstream `is_default` (ext_block.h:363-364).
pub fn is_default(ctx: &ExtRepContext, e: &ExtParam) -> Result<bool, StructureError> {
    let (x, gamma_lambda) = e.restrict_mod(ctx)?;
    let default = default_extend_srm(ctx, x, gamma_lambda)?;
    Ok(same_sign(ctx, e, &default))
}

/// Upstream `same_standard_reps` (ext_block.cpp:910-931), in the
/// same-context case: equal `theta`, `l` difference in the `(theta+1)`
/// right image, integral `gamma_lambda` difference in the `(theta-1)` left
/// image.
pub fn same_standard_reps(
    ctx: &ExtRepContext,
    e: &ExtParam,
    f: &ExtParam,
) -> Result<bool, StructureError> {
    if e.involution != f.involution {
        return Ok(false);
    }
    let theta = e.theta(ctx)?;
    let rank = ctx.rc().rank();
    // theta + 1 as an IntMatrix.
    let mut plus = Vec::with_capacity(rank * rank);
    let mut minus = Vec::with_capacity(rank * rank);
    for (i, row) in theta.weight_matrix().iter().enumerate() {
        for (j, &entry) in row.iter().enumerate() {
            let unit = if i == j { 1 } else { 0 };
            plus.push(entry.wrapping_add(unit));
            minus.push(entry.wrapping_sub(unit));
        }
    }
    let plus = IntMatrix::from_entries(rank, rank, plus);
    let minus = IntMatrix::from_entries(rank, rank, minus);
    let l_diff = vec_sub(e.l.as_slice(), f.l.as_slice());
    if !in_right_image(&plus, &l_diff) {
        return Ok(false);
    }
    let gamma_diff = integer_diff(&e.gamma_lambda, &f.gamma_lambda)?;
    Ok(in_left_image(&gamma_diff, &minus))
}

/// Upstream `same_sign` (ext_block.cpp:936-948): Proposition 16 of
/// "Parameters for twisted representations".
pub fn same_sign(ctx: &ExtRepContext, e: &ExtParam, f: &ExtParam) -> bool {
    // Upstream asserts same_standard_reps(E,F) here (ext_block.cpp:938), but
    // the oracle is built with -DNDEBUG and shift_flip reaches this via
    // is_default with a shifted parameter that can violate it (e.g. nonzero
    // shift at a compact Cartan). Omit the assert so debug builds stay
    // oracle-compatible; callers that rely on the precondition are unaffected.
    let delta = ctx.delta();
    let kappa_of = |tau: &Weight| -> Vec<i32> {
        let delta_tau = delta.act_on_weight(tau).expect("same_sign: delta on tau");
        vec_sub(tau.as_slice(), delta_tau.as_slice())
    };
    let kappa1 = kappa_of(&e.tau);
    let kappa2 = kappa_of(&f.tau);
    let i_exp = vec_dot(e.l.as_slice(), &kappa1).wrapping_sub(vec_dot(f.l.as_slice(), &kappa2));
    debug_assert!(i_exp % 2 == 0);
    let l_diff = vec_sub(f.l.as_slice(), e.l.as_slice());
    let gamma_diff = e
        .gamma_lambda
        .sub(&f.gamma_lambda)
        .expect("same_sign: gamma difference");
    let n1_exp =
        vec_dot(&l_diff, e.tau.as_slice()).wrapping_add(rat_dot(&gamma_diff, f.t.as_slice()));
    ((i_exp / 2).wrapping_add(n1_exp) % 2 == 0) != (e.flipped != f.flipped)
}

/// Upstream `z_align` (three-argument form, ext_block.cpp:965-971): set
/// `f`'s flip from `e`'s by the tau-alignment parity and `extra_flip`.
pub fn z_align(ctx: &ExtRepContext, e: &ExtParam, f: &mut ExtParam, extra_flip: bool) {
    debug_assert_eq!(e.t, f.t); // prepared upstairs by the caller
    let delta = ctx.delta();
    let aligned = |param: &ExtParam| -> i32 {
        let delta_tau = delta
            .act_on_weight(&param.tau)
            .expect("z_align: delta on tau");
        vec_dot(
            param.l.as_slice(),
            &vec_sub(delta_tau.as_slice(), param.tau.as_slice()),
        )
    };
    let d = aligned(e).wrapping_sub(aligned(f));
    debug_assert!(d % 2 == 0);
    f.flipped = e.flipped ^ (d.wrapping_rem(4) != 0) ^ extra_flip;
}

/// Upstream `z_align` (four-argument form, ext_block.cpp:983-985): fold
/// the parity of `t_mu` into the extra flip.
pub fn z_align_mu(
    ctx: &ExtRepContext,
    e: &ExtParam,
    f: &mut ExtParam,
    extra_flip: bool,
    t_mu: i32,
) {
    z_align(ctx, e, f, extra_flip ^ (t_mu % 2 != 0));
}

/// Upstream `level_a` (ext_block.cpp:958-962): `<gamma_lambda + shift,
/// coroot(alpha)>`, integral by the parity invariants.
pub fn level_a(
    ctx: &ExtRepContext,
    e: &ExtParam,
    shift: &Weight,
    alpha: RootId,
) -> Result<i32, StructureError> {
    let coroot = coroot_coords(ctx.rc().root_system(), alpha)?;
    let shifted = e
        .gamma_lambda
        .add(&RationalWeight::from_weight(&Weight::new(
            shift.as_slice().to_vec(),
        ))?)?;
    Ok(rat_dot(&shifted, coroot))
}

/// Upstream `validate` (ext_block.cpp:826-839), compiled in only under
/// `debug_assertions` exactly like upstream's `#ifndef NDEBUG`.
#[cfg(debug_assertions)]
fn validate(ctx: &ExtRepContext, e: &ExtParam) {
    let theta = e
        .theta(ctx)
        .expect("validate: twisted involution must be tabulated");
    let delta = ctx.delta();
    debug_assert_eq!(
        compose_matrices(delta.weight_matrix(), theta.weight_matrix()).expect("validate: compose"),
        compose_matrices(theta.weight_matrix(), delta.weight_matrix()).expect("validate: compose"),
        "validate: delta and theta must commute"
    );
    let delta_gamma = e
        .gamma_lambda
        .apply_matrix(delta.weight_matrix())
        .expect("validate: delta on gamma_lambda");
    let diff = integer_diff(&e.gamma_lambda, &delta_gamma).expect("validate: integer diff");
    let theta_tau = theta.act_on_weight(&e.tau).expect("validate: theta on tau");
    debug_assert_eq!(
        vec_sub(e.tau.as_slice(), theta_tau.as_slice()),
        diff,
        "validate: (1-theta)*tau"
    );
    let left = {
        let delta_l = delta.act_on_coweight(&e.l).expect("validate: delta on l");
        vec_sub(delta_l.as_slice(), e.l.as_slice())
    };
    let right = {
        let theta_t = theta.act_on_coweight(&e.t).expect("validate: theta on t");
        vec_add(theta_t.as_slice(), e.t.as_slice())
    };
    debug_assert_eq!(left, right, "validate: l*(delta-1) == t*(theta+1)");
    // ((g_rho_check - l) * (1-theta)) numerator vanishes.
    let g = ctx.rc().g_rho_check();
    let rank = ctx.rc().rank();
    for column in 0..rank {
        let mut total = Rational::from(0);
        for row in 0..rank {
            let entry = theta.weight_matrix()[row][column];
            let coefficient = if row == column { 1 - entry } else { -entry };
            if coefficient != 0 {
                let g_minus_l = &g.coordinates()[row] - Rational::from(e.l.as_slice()[row]);
                total += g_minus_l * Rational::from(coefficient);
            }
        }
        debug_assert_eq!(total, Rational::from(0), "validate: (g-l)(1-theta)");
    }
    let theta_gamma = e
        .gamma_lambda
        .apply_matrix(theta.weight_matrix())
        .expect("validate: theta on gamma_lambda");
    debug_assert!(
        e.gamma_lambda
            .add(&theta_gamma)
            .expect("validate: theta+1 on gamma_lambda")
            .numerator()
            .iter()
            .all(|&entry| entry == 0),
        "validate: (theta+1)*gamma_lambda"
    );
}

/// No-op in release builds, matching upstream's `#ifndef NDEBUG`.
#[cfg(not(debug_assertions))]
fn validate(_ctx: &ExtRepContext, _e: &ExtParam) {}

/// Upstream `fixed_conjugate_simple` (ext_block.cpp:525-552): conjugate
/// `alpha` (updated in place) to a simple root by a word in `W^delta`,
/// returning the left-conjugating word. May leave `alpha` non-simple at
/// the high root of a folded A2 (upstream's halfway `break`).
pub fn fixed_conjugate_simple(
    ctx: &ExtRepContext,
    alpha: &mut RootId,
) -> Result<Vec<usize>, StructureError> {
    let system = ctx.rc().root_system();
    let simple_ids = system.simple_root_ids();
    let cartan = ctx.rc().datum().cartan_matrix();
    let mut result = Vec::new();
    while !simple_ids.contains(alpha) {
        // descent_set(alpha) \ ascent_set(delta(alpha)), first bit.
        let delta_alpha = ctx.delta_of(*alpha);
        let mut generator = None;
        for (s, &simple) in simple_ids.iter().enumerate() {
            let coroot = coroot_coords(system, simple)?;
            if vec_dot(root_coords(system, *alpha)?, coroot) > 0
                && vec_dot(root_coords(system, delta_alpha)?, coroot) >= 0
            {
                generator = Some(s);
                break;
            }
        }
        let s = generator.ok_or(StructureError::RepInvariantViolation {
            invariant: "fixed_conjugate_simple descent",
        })?;
        let t = ctx.twisted(s);
        if simple_reflect_root(system, s, *alpha)? == simple_ids[t] {
            break; // alpha is the sum of the linked simple roots s, twist(s)
        }
        result.push(s);
        *alpha = simple_reflect_root(system, s, *alpha)?;
        if s != t {
            result.push(t);
            *alpha = simple_reflect_root(system, t, *alpha)?;
            if cartan[s][t] < 0 {
                // s and t do not commute: re-apply s to symmetrise.
                result.push(s);
                *alpha = simple_reflect_root(system, s, *alpha)?;
            }
        }
    }
    result.reverse();
    Ok(result)
}

// ---------------------------------------------------------------------------
// complex_cross (upstream ext_block.cpp:858-907).
// ---------------------------------------------------------------------------

/// Upstream `complex_cross`: the extended parameter across a complex link
/// for a folded generator of length 1, 2, or 3. `e` is taken by value and
/// modified, exactly as upstream's by-value parameter.
pub fn complex_cross(
    ctx: &ExtRepContext,
    length: usize,
    n_alpha: RootId,
    mut e: ExtParam,
) -> Result<ExtParam, StructureError> {
    let rc = ctx.rc();
    let system = rc.root_system();
    let table = rc.table();
    let theta = e.involution;

    // rho_r_shift = 2rho of the positive real roots at theta;
    // dual_rho_im_shift = 2rho_check of the positive imaginary roots.
    let positive_real: Vec<RootId> = rc
        .root_involution_data(theta)?
        .roots_of_kind(RootKind::Real)
        .filter(|&root| system.is_positive(root) == Some(true))
        .collect();
    let mut rho_r_shift = rc.two_rho_of(&positive_real)?.as_slice().to_vec();
    let positive_imaginary: Vec<RootId> = rc
        .root_involution_data(theta)?
        .roots_of_kind(RootKind::Imaginary)
        .filter(|&root| system.is_positive(root) == Some(true))
        .collect();
    let mut dual_rho_im_shift = coroot_sum(system, &positive_imaginary)?;

    // The kappa chain: [alpha]; length 2 prepends delta(alpha); length 3
    // prepends delta(alpha) and then alpha again (ext_block.cpp:872-878).
    let mut kappa = vec![n_alpha];
    if length > 1 {
        kappa.insert(0, ctx.delta_of(n_alpha));
        if length == 3 {
            kappa.insert(0, n_alpha);
        }
    }

    let g_rho_check = rc.g_rho_check().clone();
    for &beta in &kappa {
        for &generator in &reflection_word(rc, beta)? {
            e.involution = table.cross(generator, e.involution)?;
        }
        e.gamma_lambda = reflect_rational(system, beta, &e.gamma_lambda)?;
        reflect_coords(system, beta, &mut rho_r_shift)?;
        {
            let mut tau = e.tau.as_slice().to_vec();
            reflect_coords(system, beta, &mut tau)?;
            e.tau = Weight::new(tau);
        }
        {
            let d = -rational_coweight_dot(g_rho_check.coordinates(), root_coords(system, beta)?)?;
            let mut l = e.l.as_slice().to_vec();
            coreflect_coords(system, beta, &mut l, d)?;
            e.l = Coweight::new(l);
        }
        coreflect_coords(system, beta, &mut dual_rho_im_shift, 0)?;
        {
            let mut t = e.t.as_slice().to_vec();
            coreflect_coords(system, beta, &mut t, 0)?;
            e.t = Coweight::new(t);
        }
    }

    let new_theta = e.involution;
    let new_positive_real: Vec<RootId> = rc
        .root_involution_data(new_theta)?
        .roots_of_kind(RootKind::Real)
        .filter(|&root| system.is_positive(root) == Some(true))
        .collect();
    let new_real_sum = rc.two_rho_of(&new_positive_real)?;
    rho_r_shift = vec_sub(&rho_r_shift, new_real_sum.as_slice());
    // The difference is a sum of (real) roots: halve it.
    for entry in &mut rho_r_shift {
        *entry = entry.wrapping_div(2);
    }
    e.gamma_lambda = e
        .gamma_lambda
        .add(&RationalWeight::from_weight(&Weight::new(
            rho_r_shift.clone(),
        ))?)?;
    debug_assert_eq!(
        ctx.delta()
            .act_on_weight(&Weight::new(rho_r_shift.clone()))?,
        Weight::new(rho_r_shift.clone()),
        "complex_cross: rho_r_shift is delta-fixed"
    );

    let new_positive_imaginary: Vec<RootId> = rc
        .root_involution_data(new_theta)?
        .roots_of_kind(RootKind::Imaginary)
        .filter(|&root| system.is_positive(root) == Some(true))
        .collect();
    let new_imaginary_sum = coroot_sum(system, &new_positive_imaginary)?;
    dual_rho_im_shift = vec_sub(&dual_rho_im_shift, &new_imaginary_sum);
    for entry in &mut dual_rho_im_shift {
        *entry = entry.wrapping_div(2);
    }
    let l = vec_sub(e.l.as_slice(), &dual_rho_im_shift);
    e.l = Coweight::new(l);
    debug_assert_eq!(
        ctx.delta()
            .act_on_coweight(&Coweight::new(dual_rho_im_shift.clone()))?,
        Coweight::new(dual_rho_im_shift.clone()),
        "complex_cross: dual_rho_im_shift is delta-fixed"
    );

    validate(ctx, &e);

    let mut alpha_simple = n_alpha;
    let to_simple = fixed_conjugate_simple(ctx, &mut alpha_simple)?;
    // By symmetry, to_simple also conjugates delta(alpha) to simple.
    let s = pos_to_neg(system, &to_simple)?;
    e.flip(ctx.shift_flip(theta, new_theta, &s)?);

    e.flip(length == 2); // October surprise, parallelling the 2i/2r flips

    Ok(e)
}

/// Sum of the coroots of `roots` (upstream `coroot_sum`, rootdata.cpp:
/// 1654-1660).
fn coroot_sum(system: &RootSystem, roots: &[RootId]) -> Result<Vec<i32>, StructureError> {
    let mut sum = vec![0_i32; system.lattice_rank()];
    for &id in roots {
        vec_add_scaled(&mut sum, coroot_coords(system, id)?, 1);
    }
    Ok(sum)
}

// ---------------------------------------------------------------------------
// star (upstream ext_block.cpp:990-1705).
// ---------------------------------------------------------------------------

/// `g_rho_check - l` as exact rational coordinates.
fn g_minus_l(ctx: &ExtRepContext, l: &Coweight) -> Vec<Rational> {
    ctx.rc()
        .g_rho_check()
        .coordinates()
        .iter()
        .zip(l.as_slice())
        .map(|(g, &c)| g - Rational::from(c))
        .collect()
}

/// `rw + w` with `w` an integral weight in raw coordinates.
fn rw_add(rw: &RationalWeight, w: &[i32]) -> Result<RationalWeight, StructureError> {
    rw.add(&RationalWeight::from_weight(&Weight::new(w.to_vec()))?)
}

/// `rw - w` with `w` an integral weight in raw coordinates.
fn rw_sub(rw: &RationalWeight, w: &[i32]) -> Result<RationalWeight, StructureError> {
    rw.sub(&RationalWeight::from_weight(&Weight::new(w.to_vec()))?)
}

/// The weight matrix (or coweight matrix when `coweight`) of the
/// involution of `tw`, with `diagonal` added on the diagonal (upstream
/// `i_tab.matrix(tw) +/- 1` and its `.transposed()` variants).
fn shifted_involution_matrix(
    table: &InvolutionTable,
    id: InvolutionId,
    coweight: bool,
    diagonal: i32,
) -> Result<IntMatrix, StructureError> {
    let record = table.record(id).ok_or(StructureError::IndexOutOfRange {
        index: id.0,
        upper_bound: table.involution_count(),
    })?;
    let matrix = if coweight {
        record.theta().coweight_matrix()
    } else {
        record.theta().weight_matrix()
    };
    let rank = matrix.len();
    let mut data = Vec::with_capacity(rank * rank);
    for (i, row) in matrix.iter().enumerate() {
        for (j, &entry) in row.iter().enumerate() {
            data.push(entry.wrapping_add(if i == j { diagonal } else { 0 }));
        }
    }
    Ok(IntMatrix::from_entries(rank, rank, data))
}

/// Upstream `RootSystem::find_descent` (rootdata.h:299): the first
/// generator with `<root(alpha), coroot_s> > 0` (the sign-correct formula
/// for both positive and negative roots).
fn find_descent(system: &RootSystem, alpha: RootId) -> Result<usize, StructureError> {
    for (generator, &simple) in system.simple_root_ids().iter().enumerate() {
        if vec_dot(root_coords(system, alpha)?, coroot_coords(system, simple)?) > 0 {
            return Ok(generator);
        }
    }
    Err(StructureError::RepInvariantViolation {
        invariant: "find_descent",
    })
}

/// Upstream `RootDatum::reflected_root(alpha, r)`: reflect root `r` in
/// root `by`.
fn reflected_root(system: &RootSystem, by: RootId, r: RootId) -> Result<RootId, StructureError> {
    let factor = vec_dot(root_coords(system, r)?, coroot_coords(system, by)?);
    let mut image = root_coords(system, r)?.to_vec();
    vec_add_scaled(&mut image, root_coords(system, by)?, -factor);
    system
        .id_of(&Weight::new(image))
        .ok_or(StructureError::RepInvariantViolation {
            invariant: "reflected root",
        })
}

/// Upstream `RootDatum::permuted_root(r, ww)` (rootdata.h:320-324): apply
/// the word's reflections front to back.
fn permuted_root(
    system: &RootSystem,
    mut root: RootId,
    word: &[usize],
) -> Result<RootId, StructureError> {
    for &generator in word {
        root = simple_reflect_root(system, generator, root)?;
    }
    Ok(root)
}

/// Upstream `ext_block::scent_count` (ext_block.cpp:209-212).
fn scent_count(value: DescValue) -> usize {
    if value.has_double_image() {
        2
    } else if value.link_count() == 0 {
        0
    } else {
        1
    }
}

/// Upstream `ext_block::star` (ext_block.cpp:990-1705): the type of the
/// `delta`-orbit of root number `n_alpha` (a root number of the parent
/// datum) for the extended parameter `e`, exporting the adjacent extended
/// parameters in upstream's push order.
pub fn star(
    ctx: &ExtRepContext,
    e: &ExtParam,
    length: usize,
    n_alpha: usize,
) -> Result<(DescValue, Vec<ExtParam>), StructureError> {
    let rc = ctx.rc();
    let system = rc.root_system();
    let table = rc.table();
    let simple_ids: Vec<RootId> = system.simple_root_ids().to_vec();
    let mut e0 = e.clone();
    let n_alpha = RootId::from_usize(n_alpha);
    let theta = e.involution;
    let theta_alpha =
        rc.root_involution_data(theta)?
            .image(n_alpha)
            .ok_or(StructureError::IndexOutOfRange {
                index: n_alpha.index(),
                upper_bound: system.roots().len(),
            })?;
    let result;
    let mut links: Vec<ExtParam> = Vec::new();

    match length {
        1 => {
            let alpha = root_coords(system, n_alpha)?.to_vec();
            let alpha_v = coroot_coords(system, n_alpha)?.to_vec();

            if theta_alpha == n_alpha {
                // Length 1 imaginary case.
                let gml = g_minus_l(ctx, &e.l);
                let tf_alpha = rational_coweight_dot(&gml, &alpha)?;
                if tf_alpha % 2 != 0 {
                    return Ok((DescValue::OneImaginaryCompact, links));
                }
                // Noncompact case.
                let new_involution =
                    left_word_lookup(table, e.involution, &reflection_word(rc, n_alpha)?)?;
                let th_1 = shifted_involution_matrix(table, new_involution, false, -1)?;
                let mut tau_coef = vec_dot(&alpha_v, e.tau.as_slice());

                // Try to make alpha simple by conjugating by W^delta.
                let mut alpha_simple = n_alpha;
                let ww = fixed_conjugate_simple(ctx, &mut alpha_simple)?;
                let theta_p = new_involution;
                let s = pos_to_neg(system, &ww)?;
                let rho_r_shift = ctx.to_simple_shift(theta, theta_p, &s)?;
                let flipped = ctx.shift_flip(theta, theta_p, &s)?;
                debug_assert_eq!(
                    ctx.delta().act_on_weight(&rho_r_shift)?,
                    rho_r_shift,
                    "star 1i: rho_r_shift is delta-fixed"
                );
                debug_assert_eq!(vec_dot(e.t.as_slice(), &alpha), 0);

                if has_solution(&th_1, &alpha) {
                    // Type 1: extended type 1i1.
                    result = DescValue::OneImaginarySingle;
                    debug_assert!(simple_ids.contains(&alpha_simple));
                    let diff = find_solution(&th_1, &alpha).ok_or(
                        StructureError::RepInvariantViolation {
                            invariant: "star 1i1 solution",
                        },
                    )?;
                    let mut tau = e0.tau.as_slice().to_vec();
                    vec_add_scaled(&mut tau, &diff, tau_coef);
                    let mut l = e.l.as_slice().to_vec();
                    vec_add_scaled(&mut l, &alpha_v, tf_alpha / 2);
                    let mut f = ExtParam::new(
                        ctx,
                        new_involution,
                        rw_sub(&e.gamma_lambda, rho_r_shift.as_slice())?,
                        Weight::new(tau),
                        Coweight::new(l),
                        e.t.clone(),
                        false,
                    );
                    e0.l = Coweight::new(if tf_alpha % 4 == 0 {
                        vec_add(f.l.as_slice(), &alpha_v)
                    } else {
                        f.l.as_slice().to_vec()
                    });
                    debug_assert!(!same_standard_reps(ctx, e, &e0)?);
                    z_align(ctx, e, &mut f, flipped);
                    z_align(ctx, &f, &mut e0, flipped);
                    links.push(f); // Cayley link
                    links.push(e0); // cross link
                } else {
                    // Imaginary type 2: distinguish 1i2f and 1i2s.
                    let mut new_gam_lam = e.gamma_lambda.clone();
                    let mut new_tau = e.tau.as_slice().to_vec();
                    if !simple_ids.contains(&alpha_simple) {
                        tau_coef -= 1; // parity change and decrease both matter
                        let s = find_descent(system, alpha_simple)?;
                        let first = permuted_root(system, simple_ids[s], &ww)?;
                        debug_assert_eq!(
                            vec_add(
                                root_coords(system, first)?,
                                ctx.delta()
                                    .act_on_weight(&Weight::new(
                                        root_coords(system, first)?.to_vec()
                                    ))?
                                    .as_slice(),
                            ),
                            alpha,
                        );
                        new_gam_lam = rw_sub(&new_gam_lam, root_coords(system, first)?)?;
                        vec_add_scaled(&mut new_tau, root_coords(system, first)?, -1);
                    }
                    if tau_coef % 2 != 0 {
                        return Ok((DescValue::OneImaginaryPairSwitched, links));
                    }
                    result = DescValue::OneImaginaryPairFixed;
                    let mut l = e.l.as_slice().to_vec();
                    vec_add_scaled(&mut l, &alpha_v, tf_alpha / 2);
                    let l = Coweight::new(l);
                    let mut f0 = ExtParam::new(
                        ctx,
                        new_involution,
                        rw_sub(&new_gam_lam, rho_r_shift.as_slice())?,
                        Weight::new({
                            let mut tau = new_tau;
                            vec_add_scaled(&mut tau, &alpha, -(tau_coef / 2));
                            tau
                        }),
                        l.clone(),
                        e.t.clone(),
                        false,
                    );
                    let mut f1 = ExtParam::new(
                        ctx,
                        new_involution,
                        rw_sub(f0.gamma_lambda(), &alpha)?,
                        f0.tau.clone(),
                        l,
                        e.t.clone(),
                        false,
                    );
                    let flipped = if simple_ids.contains(&alpha_simple) {
                        flipped
                    } else {
                        !flipped
                    };
                    z_align(ctx, e, &mut f0, flipped);
                    z_align(ctx, e, &mut f1, flipped);
                    links.push(f0); // Cayley link
                    links.push(f1); // Cayley link
                }
            } else if theta_alpha == root_minus(system, n_alpha)? {
                // Length 1 real case.
                let mut alpha_simple = n_alpha;
                let ww = fixed_conjugate_simple(ctx, &mut alpha_simple)?;
                let new_involution =
                    left_word_lookup(table, e.involution, &reflection_word(rc, n_alpha)?)?;
                let theta_p = new_involution;
                let s = pos_to_neg(system, &ww)?;
                let mut rho_r_shift = ctx.to_simple_shift(theta, theta_p, &s)?;
                let mut flipped = ctx.shift_flip(theta, theta_p, &s)?;
                debug_assert_eq!(
                    ctx.delta().act_on_weight(&rho_r_shift)?,
                    rho_r_shift,
                    "star 1r: rho_r_shift is delta-fixed"
                );

                let t_alpha = vec_dot(e.t.as_slice(), &alpha);
                let theta_1 = shifted_involution_matrix(table, e.involution, false, -1)?;
                if has_solution(&theta_1, &alpha) {
                    // Length 1 type 1 real case.
                    debug_assert!(simple_ids.contains(&alpha_simple));
                    let level = level_a(ctx, e, &rho_r_shift, n_alpha)?;
                    if level % 2 != 0 {
                        return Ok((DescValue::OneRealNonparity, links));
                    }
                    if t_alpha % 2 != 0 {
                        return Ok((DescValue::OneRealPairSwitched, links));
                    }
                    result = DescValue::OneRealPairFixed;
                    let new_gam_lam = rw_sub(
                        &rw_add(&e.gamma_lambda, rho_r_shift.as_slice())?,
                        &vec_scaled(&alpha, level / 2),
                    )?;
                    debug_assert_eq!(rat_dot(&new_gam_lam, &alpha_v), 0);
                    {
                        let mut t = e0.t.as_slice().to_vec();
                        vec_add_scaled(&mut t, &alpha_v, -(t_alpha / 2));
                        e0.t = Coweight::new(t);
                    }
                    debug_assert!(same_sign(ctx, e, &e0));
                    let mut f0 = ExtParam::new(
                        ctx,
                        new_involution,
                        new_gam_lam.clone(),
                        e.tau.clone(),
                        e.l.clone(),
                        e0.t.clone(),
                        false,
                    );
                    let mut f1 = ExtParam::new(
                        ctx,
                        new_involution,
                        new_gam_lam,
                        e.tau.clone(),
                        Coweight::new(vec_add(e.l.as_slice(), &alpha_v)),
                        e0.t.clone(),
                        false,
                    );
                    z_align(ctx, &e0, &mut f0, flipped);
                    z_align(ctx, &e0, &mut f1, flipped);
                    links.push(f0); // first Cayley
                    links.push(f1); // second Cayley
                } else {
                    // Length 1 type 2 real.
                    let mut level = level_a(ctx, e, &rho_r_shift, n_alpha)?;
                    let mut new_tau = e.tau.as_slice().to_vec();
                    if !simple_ids.contains(&alpha_simple) {
                        // Adapt to integrality-based change of lambda.
                        let first = permuted_root(
                            system,
                            simple_ids[find_descent(system, alpha_simple)?],
                            &ww,
                        )?;
                        debug_assert_eq!(
                            vec_add(
                                root_coords(system, first)?,
                                ctx.delta()
                                    .act_on_weight(&Weight::new(
                                        root_coords(system, first)?.to_vec()
                                    ))?
                                    .as_slice(),
                            ),
                            alpha,
                        );
                        rho_r_shift = Weight::new(vec_add(
                            rho_r_shift.as_slice(),
                            root_coords(system, first)?,
                        ));
                        level += 1;
                        vec_add_scaled(&mut new_tau, root_coords(system, first)?, 1);
                        flipped = !flipped;
                    }
                    if level % 2 != 0 {
                        return Ok((DescValue::OneRealNonparity, links));
                    }
                    result = DescValue::OneRealSingle;
                    let new_gam_lam = rw_sub(
                        &rw_add(&e.gamma_lambda, rho_r_shift.as_slice())?,
                        &vec_scaled(&alpha, level / 2),
                    )?;
                    debug_assert_eq!(rat_dot(&new_gam_lam, &alpha_v), 0);
                    let diff = find_solution(
                        &shifted_involution_matrix(table, new_involution, true, 1)?,
                        &alpha_v,
                    )
                    .ok_or(StructureError::RepInvariantViolation {
                        invariant: "star 1r2 solution",
                    })?;
                    {
                        let mut t = e0.t.as_slice().to_vec();
                        vec_add_scaled(&mut t, &diff, -t_alpha);
                        e0.t = Coweight::new(t);
                    }
                    debug_assert!(same_sign(ctx, e, &e0));
                    let mut e1 = e0.clone();
                    e1.gamma_lambda = rw_sub(&e1.gamma_lambda, &alpha)?;
                    debug_assert!(!same_standard_reps(ctx, &e0, &e1)?);
                    let mut f = ExtParam::new(
                        ctx,
                        new_involution,
                        new_gam_lam,
                        Weight::new(new_tau),
                        e.l.clone(),
                        e0.t.clone(),
                        false,
                    );
                    z_align(ctx, &e0, &mut f, flipped);
                    z_align(ctx, &f, &mut e1, flipped);
                    // z_align ignores gamma_lambda, so the flips must agree.
                    debug_assert_eq!(e0.flipped, e1.flipped);
                    links.push(f); // Cayley link
                    links.push(e1); // cross link
                }
            } else {
                // Length 1 complex case.
                result = if system.is_positive(theta_alpha) == Some(true) {
                    DescValue::OneComplexAscent
                } else {
                    DescValue::OneComplexDescent
                };
                links.push(complex_cross(ctx, 1, n_alpha, e.clone())?);
            }
        }
        2 => {
            let alpha = root_coords(system, n_alpha)?.to_vec();
            let alpha_v = coroot_coords(system, n_alpha)?.to_vec();
            let n_beta = ctx.delta_of(n_alpha);
            let beta = root_coords(system, n_beta)?.to_vec();
            let beta_v = coroot_coords(system, n_beta)?.to_vec();

            if theta_alpha == n_alpha {
                // Length 2 imaginary case.
                let gml = g_minus_l(ctx, &e.l);
                let tf_alpha = rational_coweight_dot(&gml, &alpha)?;
                let tf_beta = rational_coweight_dot(&gml, &beta)?;
                debug_assert_eq!((tf_alpha - tf_beta) % 2, 0);
                if tf_alpha % 2 != 0 {
                    return Ok((DescValue::TwoImaginaryCompact, links));
                }
                // Noncompact case.
                let new_involution = reflected_involution(
                    table,
                    e.involution,
                    &[reflection_word(rc, n_beta)?, reflection_word(rc, n_alpha)?],
                )?;
                let mut alpha_simple = n_alpha;
                let ww = fixed_conjugate_simple(ctx, &mut alpha_simple)?;
                let theta_p = new_involution;
                let s = pos_to_neg(system, &ww)?;
                let rho_r_shift = ctx.to_simple_shift(theta, theta_p, &s)?;
                let mut flipped = ctx.shift_flip(theta, theta_p, &s)?;
                debug_assert_eq!(
                    ctx.delta().act_on_weight(&rho_r_shift)?,
                    rho_r_shift,
                    "star 2i: rho_r_shift is delta-fixed"
                );
                debug_assert!(simple_ids.contains(&alpha_simple));
                // October surprise: wedge correction for the 2i/2r cases.
                flipped = !flipped;

                let at = vec_dot(&alpha_v, e.tau.as_slice());
                let bt = vec_dot(&beta_v, e.tau.as_slice());
                let th_1 = shifted_involution_matrix(table, new_involution, false, -1)?;

                let mut new_l = e.l.as_slice().to_vec();
                vec_add_scaled(&mut new_l, &alpha_v, tf_alpha / 2);
                vec_add_scaled(&mut new_l, &beta_v, tf_beta / 2);
                let new_l = Coweight::new(new_l);

                if has_solution(&th_1, &alpha) {
                    // Type 2i11.
                    result = DescValue::TwoImaginarySingleSingle;
                    let sigma = find_solution(
                        &th_1,
                        &vec_add(&vec_scaled(&alpha, at), &vec_scaled(&beta, bt)),
                    )
                    .ok_or(StructureError::RepInvariantViolation {
                        invariant: "star 2i11 solution",
                    })?;
                    let mut f = ExtParam::new(
                        ctx,
                        new_involution,
                        rw_sub(&e.gamma_lambda, rho_r_shift.as_slice())?,
                        Weight::new(vec_add(e.tau.as_slice(), &sigma)),
                        new_l,
                        e.t.clone(),
                        false,
                    );
                    e0.l = Coweight::new(vec_add(&vec_add(e0.l.as_slice(), &alpha_v), &beta_v));
                    z_align(ctx, e, &mut f, flipped); // gamma_lambda unchanged
                    z_align(ctx, &f, &mut e0, flipped);
                    links.push(f); // Cayley link
                    links.push(e0); // cross link
                } else if has_solution(&th_1, &vec_add(&alpha, &beta)) {
                    // Case 2i12.
                    if (at + bt) % 2 != 0 {
                        return Ok((DescValue::TwoImaginarySingleDoubleSwitched, links));
                    }
                    result = DescValue::TwoImaginarySingleDoubleFixed;
                    let m = at.rem_euclid(2);
                    let mm = 1 - m;
                    // One tau needs the upstairs solution for an odd-odd pair.
                    let sigma = find_solution(
                        &th_1,
                        &vec_add(&vec_scaled(&alpha, at + mm), &vec_scaled(&beta, bt - mm)),
                    )
                    .ok_or(StructureError::RepInvariantViolation {
                        invariant: "star 2i12 solution",
                    })?;
                    let mut new_tau0 = e.tau.as_slice().to_vec();
                    vec_add_scaled(&mut new_tau0, &alpha, -((at + m) / 2));
                    vec_add_scaled(&mut new_tau0, &beta, -((bt - m) / 2));
                    // F0 is the Cayley link that does not need sigma.
                    let mut f0 = ExtParam::new(
                        ctx,
                        new_involution,
                        rw_sub(
                            &rw_sub(&e.gamma_lambda, rho_r_shift.as_slice())?,
                            &vec_scaled(&alpha, m),
                        )?,
                        Weight::new(new_tau0),
                        new_l.clone(),
                        e.t.clone(),
                        false,
                    );
                    let mut f1 = ExtParam::new(
                        ctx,
                        new_involution,
                        rw_sub(
                            &rw_sub(&e.gamma_lambda, rho_r_shift.as_slice())?,
                            &vec_scaled(&alpha, mm),
                        )?,
                        Weight::new(vec_add(e.tau.as_slice(), &sigma)),
                        new_l,
                        e.t.clone(),
                        false,
                    );
                    let t_alpha = vec_dot(e.t.as_slice(), &alpha);
                    z_align_mu(ctx, e, &mut f0, flipped, m * t_alpha);
                    z_align_mu(ctx, e, &mut f1, flipped, mm * t_alpha);
                    links.push(f0); // first Cayley
                    links.push(f1); // second Cayley
                } else {
                    // Type 2i22.
                    result = DescValue::TwoImaginaryDoubleDouble;
                    debug_assert_eq!((at - bt) % 2, 0);
                    let m = at.rem_euclid(2);
                    let mut tau0 = e.tau.as_slice().to_vec();
                    vec_add_scaled(&mut tau0, &alpha, -((at + m) / 2));
                    vec_add_scaled(&mut tau0, &beta, -((bt - m) / 2));
                    let mut tau1 = e.tau.as_slice().to_vec();
                    vec_add_scaled(&mut tau1, &alpha, -((at - m) / 2));
                    vec_add_scaled(&mut tau1, &beta, -((bt + m) / 2));
                    let mut f0 = ExtParam::new(
                        ctx,
                        new_involution,
                        rw_sub(
                            &rw_sub(&e.gamma_lambda, rho_r_shift.as_slice())?,
                            &vec_scaled(&alpha, m),
                        )?,
                        Weight::new(tau0),
                        new_l.clone(),
                        e.t.clone(),
                        false,
                    );
                    let mut f1 = ExtParam::new(
                        ctx,
                        new_involution,
                        rw_sub(
                            &rw_sub(
                                &rw_sub(&e.gamma_lambda, rho_r_shift.as_slice())?,
                                &vec_scaled(&alpha, 1 - m),
                            )?,
                            &beta,
                        )?,
                        Weight::new(tau1),
                        new_l,
                        e.t.clone(),
                        false,
                    );
                    let ta = vec_dot(e.t.as_slice(), &alpha);
                    let tb = vec_dot(e.t.as_slice(), &beta);
                    z_align_mu(ctx, e, &mut f0, flipped, ta * m);
                    z_align_mu(ctx, e, &mut f1, flipped, ta * (1 - m) + tb);
                    links.push(f0); // first Cayley
                    links.push(f1); // second Cayley
                }
            } else if theta_alpha == root_minus(system, n_alpha)? {
                // Length 2 real case.
                let mut alpha_simple = n_alpha;
                let ww = fixed_conjugate_simple(ctx, &mut alpha_simple)?;
                debug_assert!(simple_ids.contains(&alpha_simple));
                let new_involution = reflected_involution(
                    table,
                    e.involution,
                    &[reflection_word(rc, n_beta)?, reflection_word(rc, n_alpha)?],
                )?;
                let theta_p = new_involution;
                let s = pos_to_neg(system, &ww)?;
                let rho_r_shift = ctx.to_simple_shift(theta, theta_p, &s)?;
                let mut flipped = ctx.shift_flip(theta, theta_p, &s)?;
                debug_assert_eq!(
                    ctx.delta().act_on_weight(&rho_r_shift)?,
                    rho_r_shift,
                    "star 2r: rho_r_shift is delta-fixed"
                );
                // October surprise: wedge correction for the 2i/2r cases.
                flipped = !flipped;

                let a_level = level_a(ctx, e, &rho_r_shift, n_alpha)?;
                if a_level % 2 != 0 {
                    return Ok((DescValue::TwoRealNonparity, links));
                }
                let b_level = level_a(ctx, e, &rho_r_shift, n_beta)?;
                debug_assert_eq!(b_level % 2, 0);

                let theta_1 = shifted_involution_matrix(table, e.involution, false, -1)?;
                let new_gam_lam = rw_sub(
                    &rw_sub(
                        &rw_add(&e.gamma_lambda, rho_r_shift.as_slice())?,
                        &vec_scaled(&alpha, a_level / 2),
                    )?,
                    &vec_scaled(&beta, b_level / 2),
                )?;

                let ta = vec_dot(e.t.as_slice(), &alpha);
                let tb = vec_dot(e.t.as_slice(), &beta);
                let mut e1 = e.clone();

                if has_solution(&theta_1, &alpha) {
                    // Type 2r11.
                    result = DescValue::TwoRealDoubleDouble;
                    debug_assert_eq!((ta - tb) % 2, 0);
                    let m = ta.rem_euclid(2);
                    {
                        let mut t = e0.t.as_slice().to_vec();
                        vec_add_scaled(&mut t, &alpha_v, -((ta + m) / 2));
                        vec_add_scaled(&mut t, &beta_v, -((tb - m) / 2));
                        e0.t = Coweight::new(t);
                    }
                    debug_assert!(same_sign(ctx, e, &e0));
                    debug_assert_eq!(vec_dot(e0.t.as_slice(), &alpha), -m);
                    debug_assert_eq!(vec_dot(e0.t.as_slice(), &beta), m);
                    {
                        let mut t = e1.t.as_slice().to_vec();
                        vec_add_scaled(&mut t, &alpha_v, -((ta - m) / 2));
                        vec_add_scaled(&mut t, &beta_v, -((tb + m) / 2));
                        e1.t = Coweight::new(t);
                    }
                    debug_assert!(same_sign(ctx, e, &e1));
                    let mut f0 = ExtParam::new(
                        ctx,
                        new_involution,
                        new_gam_lam.clone(),
                        e.tau.clone(),
                        Coweight::new({
                            let mut l = e.l.as_slice().to_vec();
                            vec_add_scaled(&mut l, &alpha_v, m);
                            l
                        }),
                        e0.t.clone(),
                        false,
                    );
                    let mut f1 = ExtParam::new(
                        ctx,
                        new_involution,
                        new_gam_lam,
                        e.tau.clone(),
                        Coweight::new({
                            let mut l = e.l.as_slice().to_vec();
                            vec_add_scaled(&mut l, &alpha_v, 1 - m);
                            vec_add_scaled(&mut l, &beta_v, 1);
                            l
                        }),
                        e1.t.clone(),
                        false,
                    );
                    z_align_mu(ctx, &e0, &mut f0, flipped, m * ((b_level - a_level) / 2));
                    z_align_mu(ctx, &e1, &mut f1, flipped, m * ((a_level - b_level) / 2));
                    links.push(f0);
                    links.push(f1);
                } else if has_solution(&theta_1, &vec_add(&alpha, &beta)) {
                    // Type 2r21.
                    if (ta + tb) % 2 != 0 {
                        return Ok((DescValue::TwoRealSingleDoubleSwitched, links));
                    }
                    result = DescValue::TwoRealSingleDoubleFixed;
                    let m = ta.rem_euclid(2);
                    let mm = 1 - m;
                    // One t needs the downstairs solution for an odd-odd pair.
                    let s = find_solution(
                        &shifted_involution_matrix(table, new_involution, true, 1)?,
                        &vec_add(
                            &vec_scaled(&alpha_v, ta + mm),
                            &vec_scaled(&beta_v, tb - mm),
                        ),
                    )
                    .ok_or(StructureError::RepInvariantViolation {
                        invariant: "star 2r21 solution",
                    })?;
                    // E0 is adapted to the Cayley transform not needing s.
                    {
                        let mut t = e0.t.as_slice().to_vec();
                        vec_add_scaled(&mut t, &alpha_v, -((ta + m) / 2));
                        vec_add_scaled(&mut t, &beta_v, -((tb - m) / 2));
                        e0.t = Coweight::new(t);
                    }
                    debug_assert!(same_sign(ctx, e, &e0));
                    debug_assert_eq!(vec_dot(e0.t.as_slice(), &alpha), -m);
                    debug_assert_eq!(vec_dot(e0.t.as_slice(), &beta), m);
                    e1.t = Coweight::new(vec_sub(e1.t.as_slice(), &s));
                    debug_assert!(same_sign(ctx, e, &e1));
                    debug_assert_eq!(vec_dot(e1.t.as_slice(), &alpha), -mm);
                    debug_assert_eq!(vec_dot(e1.t.as_slice(), &beta), mm);
                    let mut f0 = ExtParam::new(
                        ctx,
                        new_involution,
                        new_gam_lam.clone(),
                        e.tau.clone(),
                        Coweight::new({
                            let mut l = e.l.as_slice().to_vec();
                            vec_add_scaled(&mut l, &alpha_v, m);
                            l
                        }),
                        e0.t.clone(),
                        false,
                    );
                    let mut f1 = ExtParam::new(
                        ctx,
                        new_involution,
                        new_gam_lam,
                        e.tau.clone(),
                        Coweight::new({
                            let mut l = e.l.as_slice().to_vec();
                            vec_add_scaled(&mut l, &alpha_v, mm);
                            l
                        }),
                        e1.t.clone(),
                        false,
                    );
                    z_align_mu(ctx, &e0, &mut f0, flipped, m * ((b_level - a_level) / 2));
                    z_align_mu(ctx, &e1, &mut f1, flipped, mm * ((b_level - a_level) / 2));
                    links.push(f0);
                    links.push(f1);
                } else {
                    // Case 2r22.
                    result = DescValue::TwoRealSingleSingle;
                    let s = find_solution(
                        &shifted_involution_matrix(table, new_involution, true, 1)?,
                        &vec_add(&vec_scaled(&alpha_v, ta), &vec_scaled(&beta_v, tb)),
                    )
                    .ok_or(StructureError::RepInvariantViolation {
                        invariant: "star 2r22 solution",
                    })?;
                    e0.t = Coweight::new(vec_sub(e0.t.as_slice(), &s));
                    debug_assert!(same_sign(ctx, e, &e0));
                    debug_assert_eq!(vec_dot(e.t.as_slice(), &alpha), 0);
                    debug_assert_eq!(vec_dot(e.t.as_slice(), &beta), 0);
                    e1.gamma_lambda = rw_sub(&e1.gamma_lambda, &vec_add(&alpha, &beta))?;
                    e1.t = e0.t.clone(); // cross action keeps the adaptation
                    debug_assert!(!same_standard_reps(ctx, &e0, &e1)?);
                    let mut f = ExtParam::new(
                        ctx,
                        new_involution,
                        new_gam_lam,
                        e.tau.clone(),
                        e.l.clone(),
                        e0.t.clone(),
                        false,
                    );
                    z_align(ctx, &e0, &mut f, flipped); // E.t vanishes on alpha, beta
                    z_align(ctx, &f, &mut e1, flipped);
                    links.push(f); // Cayley link
                    links.push(e1); // cross link
                }
            } else {
                // Length 2 complex case.
                let ascent = system.is_positive(theta_alpha) == Some(true);
                if theta_alpha
                    != (if ascent {
                        n_beta
                    } else {
                        root_minus(system, n_beta)?
                    })
                {
                    // Non theta-stable plane: twisted non-commutation.
                    result = if ascent {
                        DescValue::TwoComplexAscent
                    } else {
                        DescValue::TwoComplexDescent
                    };
                    links.push(complex_cross(ctx, 2, n_alpha, e.clone())?);
                } else if ascent {
                    // Twisted commutation: 2Ci.
                    result = DescValue::TwoSemiImaginary;
                    let mut new_involution = e.involution;
                    for &generator in &reflection_word(rc, n_alpha)? {
                        new_involution = table.cross(generator, new_involution)?;
                    }
                    let mut alpha_simple = n_alpha;
                    let ww = fixed_conjugate_simple(ctx, &mut alpha_simple)?;
                    debug_assert!(simple_ids.contains(&alpha_simple));
                    let theta_p = new_involution;
                    let s = pos_to_neg(system, &ww)?;
                    let rho_r_shift = ctx.to_simple_shift(theta, theta_p, &s)?;
                    let flipped = ctx.shift_flip(theta, theta_p, &s)?;
                    debug_assert_eq!(
                        ctx.delta().act_on_weight(&rho_r_shift)?,
                        rho_r_shift,
                        "star 2Ci: rho_r_shift is delta-fixed"
                    );
                    // The downstairs cross by ww has only imaginary and
                    // complex steps, so alpha_v.(gamma-lambda_rho) is
                    // unchanged across ww.
                    let f = rat_dot(&e.gamma_lambda, &alpha_v);
                    let new_gam_lam = rw_sub(
                        &rw_sub(&e.gamma_lambda, &vec_scaled(&alpha, f))?,
                        rho_r_shift.as_slice(),
                    )?;
                    // Both gamma-lambda and tau lose f*alpha by the
                    // alpha-reflection; adapt tau for the 1-delta image.
                    let mut new_tau = e.tau.as_slice().to_vec();
                    reflect_coords(system, n_alpha, &mut new_tau)?;
                    vec_add_scaled(&mut new_tau, &alpha, f);
                    let dual_f = rational_coweight_dot(&g_minus_l(ctx, &e.l), &alpha)?;
                    let new_l = Coweight::new({
                        let mut l = e.l.as_slice().to_vec();
                        vec_add_scaled(&mut l, &alpha_v, dual_f);
                        l
                    });
                    let new_t = Coweight::new({
                        let mut t = e.t.as_slice().to_vec();
                        coreflect_coords(system, n_alpha, &mut t, 0)?;
                        vec_add_scaled(&mut t, &alpha_v, -dual_f);
                        t
                    });
                    let mut fp = ExtParam::new(
                        ctx,
                        new_involution,
                        new_gam_lam,
                        Weight::new(new_tau),
                        new_l,
                        new_t,
                        e.flipped != flipped,
                    );
                    // Extra conditional flip for the 2Ci case.
                    let ab_tau = vec_dot(&vec_add(&alpha_v, &beta_v), e.tau.as_slice());
                    debug_assert_eq!(ab_tau % 2, 0);
                    fp.flip(ab_tau.wrapping_mul(dual_f).wrapping_rem(4) != 0);
                    links.push(fp); // "Cayley" link
                } else {
                    // Twisted commutation, not ascent: 2Cr.
                    result = DescValue::TwoSemiReal;
                    let mut new_involution = e.involution;
                    for &generator in &reflection_word(rc, n_alpha)? {
                        new_involution = table.cross(generator, new_involution)?;
                    }
                    let mut alpha_simple = n_alpha;
                    let ww = fixed_conjugate_simple(ctx, &mut alpha_simple)?;
                    debug_assert!(simple_ids.contains(&alpha_simple));
                    let theta_p = new_involution;
                    let s = pos_to_neg(system, &ww)?;
                    let rho_r_shift = ctx.to_simple_shift(theta, theta_p, &s)?;
                    let flipped = ctx.shift_flip(theta, theta_p, &s)?;
                    debug_assert_eq!(
                        ctx.delta().act_on_weight(&rho_r_shift)?,
                        rho_r_shift,
                        "star 2Cr: rho_r_shift is delta-fixed"
                    );
                    let f = level_a(ctx, e, &rho_r_shift, n_alpha)?;
                    let new_gam_lam = rw_sub(
                        &rw_add(&e.gamma_lambda, rho_r_shift.as_slice())?,
                        &vec_scaled(&alpha, f),
                    )?;
                    let mut new_tau = e.tau.as_slice().to_vec();
                    reflect_coords(system, n_alpha, &mut new_tau)?;
                    vec_add_scaled(&mut new_tau, &alpha, -f);
                    let dual_f = rational_coweight_dot(&g_minus_l(ctx, &e.l), &alpha)?;
                    let new_l = Coweight::new({
                        let mut l = e.l.as_slice().to_vec();
                        vec_add_scaled(&mut l, &alpha_v, dual_f);
                        l
                    });
                    let new_t = Coweight::new({
                        let mut t = e.t.as_slice().to_vec();
                        coreflect_coords(system, n_alpha, &mut t, 0)?;
                        vec_add_scaled(&mut t, &alpha_v, dual_f);
                        t
                    });
                    let mut fp = ExtParam::new(
                        ctx,
                        new_involution,
                        new_gam_lam,
                        Weight::new(new_tau),
                        new_l,
                        new_t,
                        e.flipped != flipped,
                    );
                    // Extra conditional flip for the 2Cr case.
                    let t_ab = vec_dot(e.t.as_slice(), &vec_sub(&beta, &alpha));
                    debug_assert_eq!(t_ab % 2, 0);
                    fp.flip(
                        t_ab.wrapping_mul(f.wrapping_add(vec_dot(&alpha_v, e.tau.as_slice())))
                            .wrapping_rem(4)
                            != 0,
                    );
                    links.push(fp); // "Cayley" link
                }
            }
        }
        3 => {
            let alpha = root_coords(system, n_alpha)?.to_vec();
            let alpha_v = coroot_coords(system, n_alpha)?.to_vec();
            let n_beta = ctx.delta_of(n_alpha);
            let beta = root_coords(system, n_beta)?.to_vec();
            let beta_v = coroot_coords(system, n_beta)?.to_vec();
            let n_kappa = reflected_root(system, n_beta, n_alpha)?; // s_beta(alpha)
            let kappa = root_coords(system, n_kappa)?.to_vec();
            debug_assert_eq!(kappa, vec_add(&alpha, &beta));
            let kappa_v = coroot_coords(system, n_kappa)?.to_vec();
            debug_assert_eq!(kappa_v, vec_add(&alpha_v, &beta_v));
            let s_kappa = reflection_word(rc, n_kappa)?;
            let beta_alpha = vec_sub(&beta, &alpha);
            let new_involution = left_word_lookup(table, e.involution, &s_kappa)?; // when applicable

            if theta_alpha == n_alpha {
                // Length 3 imaginary case.
                let gml = g_minus_l(ctx, &e.l);
                let tf_alpha = rational_coweight_dot(&gml, &alpha)?;
                let tf_beta = rational_coweight_dot(&gml, &beta)?;
                debug_assert_eq!((tf_alpha - tf_beta) % 2, 0);
                if tf_alpha % 2 != 0 {
                    // Both alpha and beta are compact.
                    return Ok((DescValue::ThreeImaginaryCompact, links));
                }
                // Noncompact case.
                result = DescValue::ThreeImaginarySemi;
                let mut alpha_simple = n_alpha;
                let ww = fixed_conjugate_simple(ctx, &mut alpha_simple)?;
                let theta_p = new_involution; // upstairs
                let s = pos_to_neg(system, &ww)?;
                let rho_r_shift = ctx.to_simple_shift(theta, theta_p, &s)?;
                let mut flipped = ctx.shift_flip(theta, theta_p, &s)?;
                debug_assert_eq!(
                    ctx.delta().act_on_weight(&rho_r_shift)?,
                    rho_r_shift,
                    "star 3i: rho_r_shift is delta-fixed"
                );
                debug_assert!(simple_ids.contains(&alpha_simple)); // length 3

                {
                    // Make |kappa_v.dot(E.tau)==0|.
                    let mut tau = e0.tau.as_slice().to_vec();
                    vec_add_scaled(&mut tau, &alpha, -vec_dot(&kappa_v, e.tau.as_slice()));
                    e0.tau = Weight::new(tau);
                }
                {
                    let mut l = e0.l.as_slice().to_vec();
                    vec_add_scaled(&mut l, &alpha_v, tf_alpha + tf_beta);
                    e0.l = Coweight::new(l);
                }
                {
                    let mut t = e0.t.as_slice().to_vec();
                    vec_add_scaled(
                        &mut t,
                        &vec_sub(&beta_v, &alpha_v),
                        (tf_alpha + tf_beta) / 2,
                    );
                    e0.t = Coweight::new(t);
                }
                let mut f = ExtParam::new(
                    ctx,
                    new_involution,
                    rw_sub(&e0.gamma_lambda, rho_r_shift.as_slice())?,
                    e0.tau.clone(),
                    e0.l.clone(),
                    e0.t.clone(),
                    false,
                );
                // January unsurprise for 3i: delta acts by -1.
                flipped = !flipped;
                z_align(ctx, &e0, &mut f, flipped ^ !same_sign(ctx, e, &e0));
                links.push(f); // Cayley link
            } else if theta_alpha == root_minus(system, n_alpha)? {
                // Length 3 real case.
                let mut alpha_simple = n_alpha;
                let ww = fixed_conjugate_simple(ctx, &mut alpha_simple)?;
                debug_assert!(simple_ids.contains(&alpha_simple));
                let theta_p = new_involution;
                let s = pos_to_neg(system, &ww)?;
                let rho_r_shift = ctx.to_simple_shift(theta, theta_p, &s)?;
                let mut flipped = ctx.shift_flip(theta, theta_p, &s)?;
                debug_assert_eq!(
                    ctx.delta().act_on_weight(&rho_r_shift)?,
                    rho_r_shift,
                    "star 3r: rho_r_shift is delta-fixed"
                );

                let a_level = level_a(ctx, e, &rho_r_shift, n_alpha)?;
                if a_level % 2 != 0 {
                    // Nonparity.
                    return Ok((DescValue::ThreeRealNonparity, links));
                }
                // Parity case.
                result = DescValue::ThreeRealSemi;
                let b_level = level_a(ctx, e, &rho_r_shift, n_beta)?;
                debug_assert_eq!(b_level % 2, 0); // same parity as a_level

                // Make the level for |kappa| 0 (even multiple of alpha).
                let new_gam_lam = rw_sub(
                    &rw_add(&e.gamma_lambda, rho_r_shift.as_slice())?,
                    &vec_scaled(&alpha, a_level + b_level),
                )?;

                {
                    // Makes |E0.t.dot(kappa)==0|.
                    let mut t = e0.t.as_slice().to_vec();
                    vec_add_scaled(&mut t, &alpha_v, -vec_dot(&kappa, e.t.as_slice()));
                    e0.t = Coweight::new(t);
                }
                e0.gamma_lambda = rw_sub(&e0.gamma_lambda, &vec_scaled(&alpha, a_level + b_level))?;
                {
                    let mut tau = e0.tau.as_slice().to_vec();
                    vec_add_scaled(&mut tau, &beta_alpha, (a_level + b_level) / 2);
                    e0.tau = Weight::new(tau);
                }
                debug_assert!(same_sign(ctx, e, &e0));
                debug_assert_eq!(
                    rw_add(&e0.gamma_lambda, rho_r_shift.as_slice())?,
                    new_gam_lam
                );
                validate(ctx, &e0);

                // January unsurprise for 3r.
                flipped = !flipped;
                let mut f = ExtParam::new(
                    ctx,
                    new_involution,
                    new_gam_lam,
                    e0.tau.clone(),
                    e0.l.clone(),
                    e0.t.clone(),
                    false,
                );
                // No fourth argument since |E.t.dot(kappa)==0|.
                z_align(ctx, &e0, &mut f, flipped);
                links.push(f); // Cayley link
            } else {
                // Length 3 complex case: one of 3Ci, 3Cr, 3C+/-.
                let ascent = system.is_positive(theta_alpha) == Some(true);
                if theta_alpha
                    == (if ascent {
                        n_beta
                    } else {
                        root_minus(system, n_beta)?
                    })
                {
                    // Reflection by |alpha+beta| twisted commutes with |E.tw|:
                    // 3Ci or 3Cr.
                    result = if ascent {
                        DescValue::ThreeSemiImaginary
                    } else {
                        DescValue::ThreeSemiReal
                    };
                    let mut alpha_simple = n_alpha;
                    let ww = fixed_conjugate_simple(ctx, &mut alpha_simple)?;
                    debug_assert!(simple_ids.contains(&alpha_simple));
                    let theta_p = new_involution;
                    let s = pos_to_neg(system, &ww)?;
                    let simple_shift = ctx.to_simple_shift(theta, theta_p, &s)?;
                    let rho_r_shift = if ascent {
                        simple_shift
                    } else {
                        Weight::new(vec_scaled(simple_shift.as_slice(), -1))
                    };
                    let mut flipped = ctx.shift_flip(theta, theta_p, &s)?;
                    debug_assert_eq!(
                        ctx.delta().act_on_weight(&rho_r_shift)?,
                        rho_r_shift,
                        "star 3Ci/3Cr: rho_r_shift is delta-fixed"
                    );

                    let gml = g_minus_l(ctx, &e.l);
                    let tf_alpha = rational_coweight_dot(&gml, &alpha)?;
                    let dtf_alpha = rat_dot(&e.gamma_lambda, &alpha_v);
                    // For now.
                    let mut new_gam_lam = rw_sub(&e.gamma_lambda, rho_r_shift.as_slice())?;

                    if ascent {
                        // 3Ci.
                        if dtf_alpha % 2 != 0 {
                            new_gam_lam = rw_sub(&new_gam_lam, &beta_alpha)?;
                            e0.gamma_lambda = rw_sub(&e0.gamma_lambda, &beta_alpha)?;
                            let mut tau = e0.tau.as_slice().to_vec();
                            vec_add_scaled(&mut tau, &beta_alpha, -1);
                            e0.tau = Weight::new(tau);
                        }
                        {
                            let mut l = e0.l.as_slice().to_vec();
                            vec_add_scaled(&mut l, &kappa_v, tf_alpha);
                            e0.l = Coweight::new(l);
                        }
                        {
                            let mut tau = e0.tau.as_slice().to_vec();
                            vec_add_scaled(
                                &mut tau,
                                &kappa,
                                -(vec_dot(&kappa_v, e.tau.as_slice()) / 2),
                            );
                            e0.tau = Weight::new(tau);
                        }
                        validate(ctx, &e0);
                        debug_assert_eq!(vec_dot(e0.t.as_slice(), &kappa), 0);
                        let mut f = ExtParam::new(
                            ctx,
                            new_involution,
                            new_gam_lam,
                            e0.tau.clone(),
                            e0.l.clone(),
                            e0.t.clone(),
                            false,
                        );
                        // January unsurprise for 3Ci.
                        flipped = !flipped;
                        z_align(ctx, &e0, &mut f, flipped ^ !same_sign(ctx, e, &e0));
                        links.push(f); // Cayley link
                    } else {
                        // Descent, so 3Cr.
                        e0.gamma_lambda = rw_sub(&e0.gamma_lambda, &vec_scaled(&kappa, dtf_alpha))?;
                        new_gam_lam = rw_sub(&new_gam_lam, &vec_scaled(&kappa, dtf_alpha))?;
                        {
                            // Makes |E0.t.dot(kappa)==0|.
                            let mut t = e0.t.as_slice().to_vec();
                            vec_add_scaled(
                                &mut t,
                                &kappa_v,
                                -(vec_dot(&kappa, e.t.as_slice()) / 2),
                            );
                            e0.t = Coweight::new(t);
                        }
                        if tf_alpha % 2 != 0 {
                            // b_a == beta_v - alpha_v == kappa_v - alpha_v*2.
                            let b_a = vec_sub(&beta_v, &alpha_v);
                            let mut l = e0.l.as_slice().to_vec();
                            vec_add_scaled(&mut l, &b_a, 1);
                            e0.l = Coweight::new(l);
                            let mut t = e0.t.as_slice().to_vec();
                            vec_add_scaled(&mut t, &b_a, -1);
                            e0.t = Coweight::new(t);
                        }
                        let mut f = ExtParam::new(
                            ctx,
                            new_involution,
                            new_gam_lam,
                            e0.tau.clone(),
                            e0.l.clone(),
                            e0.t.clone(),
                            false,
                        );
                        // January unsurprise for 3Cr.
                        flipped = !flipped;
                        z_align(ctx, &e0, &mut f, flipped ^ !same_sign(ctx, e, &e0));
                        // No fourth argument since |E.t.dot(kappa)==0|.
                        links.push(f); // Cayley link
                    }
                } else {
                    // Twisted non-commutation: 3C+ or 3C-.
                    result = if ascent {
                        DescValue::ThreeComplexAscent
                    } else {
                        DescValue::ThreeComplexDescent
                    };
                    links.push(complex_cross(ctx, 3, n_alpha, e.clone())?);
                }
            }
        }
        _ => unreachable!("star length dispatch"),
    }

    // October surprise: flip links whose length difference is 2.
    if length - usize::from(result.has_defect()) == 2 {
        for link in links.iter_mut().take(scent_count(result)) {
            link.flip(true);
        }
    }

    Ok((result, links))
}

// ---------------------------------------------------------------------------
// StarOracle bridge (upstream tune_signs' use of ext_param/star,
// ext_block.cpp:1707-1876).
// ---------------------------------------------------------------------------

/// A genuine [`StarOracle`] for [`crate::ext_block::ExtBlock::tune_signs`]:
/// the default extension of each parent block element (upstream
/// `ext_param::def_ext`, ext_block.cpp:2283-2310) is rebuilt from the
/// element's `x` and its stored `gamma_lambda` (the `z_pool` values,
/// already `real_unique` at `x`), and `star` recomputes the type and the
/// adjacent extended parameters.
pub struct ExtParamOracle<'a> {
    ctx: &'a ExtRepContext<'a>,
    parent: &'a BlockGraph,
    gamma_lambdas: &'a [RationalWeight],
}

impl<'a> ExtParamOracle<'a> {
    pub fn new(
        ctx: &'a ExtRepContext<'a>,
        parent: &'a BlockGraph,
        gamma_lambdas: &'a [RationalWeight],
    ) -> Self {
        debug_assert_eq!(parent.size(), gamma_lambdas.len());
        Self {
            ctx,
            parent,
            gamma_lambdas,
        }
    }
}

impl StarOracle for ExtParamOracle<'_> {
    type Param = ExtParam;

    fn def_ext(&mut self, z: usize) -> ExtParam {
        let x = self.parent.x(z).expect("in-range parent block element");
        default_extend_srm(self.ctx, x, self.gamma_lambdas[z].clone())
            .expect("default extension of a fixed block element")
    }

    fn star(
        &mut self,
        e: &ExtParam,
        orbit_length: usize,
        n_alpha: usize,
    ) -> (DescValue, Vec<ExtParam>) {
        star(self.ctx, e, orbit_length, n_alpha).expect("star on a tuned block element")
    }

    fn same_standard_reps(&self, a: &ExtParam, b: &ExtParam) -> bool {
        same_standard_reps(self.ctx, a, b).expect("same-context comparison")
    }

    fn same_sign(&self, a: &ExtParam, b: &ExtParam) -> bool {
        same_sign(self.ctx, a, b)
    }
}

/// A [`StarOracle`] backed by a [`PartialBlock`] parent (a common block on
/// a proper integral subsystem, blocks.cpp:733-1081), for
/// [`crate::ext_block::ExtBlock::tune_signs`] after
/// [`crate::ext_block::ExtBlock::build_partial`]. Unlike
/// [`ExtParamOracle`], no separate `gamma_lambdas` table is needed:
/// `def_ext` reads each element's stored `real_unique`-normalised
/// `gamma_lambda` (upstream `ext_param::def_ext(ctxt, bm,
/// block.representative(z))` with the trivial modifier,
/// ext_block.cpp:2283-2310). `star` takes the PARENT datum's root number of
/// the orbit's first member — the caller passes
/// `IntegralSubsystem::parent_root(s)` values as `tune_signs`'
/// `simply_ints`.
pub struct PartialBlockOracle<'a> {
    ctx: &'a ExtRepContext<'a>,
    parent: &'a PartialBlock,
    modifier: Option<&'a crate::BlockModifier>,
}

impl<'a> PartialBlockOracle<'a> {
    pub fn new(ctx: &'a ExtRepContext<'a>, parent: &'a PartialBlock) -> Self {
        Self {
            ctx,
            parent,
            modifier: None,
        }
    }

    pub fn with_modifier(
        ctx: &'a ExtRepContext<'a>,
        parent: &'a PartialBlock,
        modifier: &'a crate::BlockModifier,
    ) -> Self {
        Self {
            ctx,
            parent,
            modifier: Some(modifier),
        }
    }
}

impl StarOracle for PartialBlockOracle<'_> {
    type Param = ExtParam;

    fn def_ext(&mut self, z: usize) -> ExtParam {
        let mut element = self
            .parent
            .element(z)
            .expect("in-range parent block element")
            .clone();
        if let Some(modifier) = self.modifier {
            let shifted = element
                .gamma_lambda()
                .add(modifier.shift())
                .expect("lookup modifier shift matches the parent rank");
            element.set_gamma_lambda(shifted);
            self.ctx
                .rc()
                .transform_srm::<false>(modifier.w(), &mut element)
                .expect("lookup modifier transports a stored block row");
        }
        default_extend_srm(self.ctx, element.x(), element.gamma_lambda().clone())
            .expect("default extension of a fixed block element")
    }

    fn star(
        &mut self,
        e: &ExtParam,
        orbit_length: usize,
        n_alpha: usize,
    ) -> (DescValue, Vec<ExtParam>) {
        star(self.ctx, e, orbit_length, n_alpha).expect("star on a tuned block element")
    }

    fn same_standard_reps(&self, a: &ExtParam, b: &ExtParam) -> bool {
        same_standard_reps(self.ctx, a, b).expect("same-context comparison")
    }

    fn same_sign(&self, a: &ExtParam, b: &ExtParam) -> bool {
        same_sign(self.ctx, a, b)
    }
}

// ---------------------------------------------------------------------------
// Parameter-level finalisation drivers (ext_block.cpp:2435-2807).
// ---------------------------------------------------------------------------

/// The Weyl word of a folded orbit's `kappa` (upstream `ext_gen::w_kappa`,
/// lietype.h:159-167): `[s]`, `[s,t]` for commuting pairs, `[s,t,s]` for
/// linked pairs.
fn kappa_word(orbit: &ExtGen) -> Vec<usize> {
    match orbit.kind {
        ExtGenKind::One => vec![orbit.s0],
        ExtGenKind::Two => vec![orbit.s0, orbit.s1],
        ExtGenKind::Three => vec![orbit.s0, orbit.s1, orbit.s0],
    }
}

/// The enumerated root id of a simple root (upstream `simpleRootNbr`).
fn simple_root_id(system: &RootSystem, generator: usize) -> Result<RootId, StructureError> {
    system
        .simple_root_ids()
        .get(generator)
        .copied()
        .ok_or(StructureError::IndexOutOfRange {
            index: generator,
            upper_bound: system.simple_root_ids().len(),
        })
}

/// The kind of a simple root under an involution (upstream
/// `InvolutionTable::{is_complex_simple,is_real_simple,is_imaginary_simple}`).
fn simple_kind(
    table: &InvolutionTable,
    theta: InvolutionId,
    generator: usize,
) -> Result<RootKind, StructureError> {
    table
        .simple_root_kind(theta, generator)
        .ok_or(StructureError::IndexOutOfRange {
            index: generator,
            upper_bound: table.root_system().simple_root_ids().len(),
        })
}

/// Upstream `InvolutionTable::complex_is_descent` (involutions.cpp:273-276)
/// at a simple (positive) root: whether the involution negates it.
fn simple_complex_is_descent(
    rc: &RepContext,
    theta: InvolutionId,
    generator: usize,
) -> Result<bool, StructureError> {
    let system = rc.root_system();
    let simple = simple_root_id(system, generator)?;
    let image =
        rc.root_involution_data(theta)?
            .image(simple)
            .ok_or(StructureError::IndexOutOfRange {
                index: simple.index(),
                upper_bound: system.roots().len(),
            })?;
    Ok(system.is_positive(image) == Some(false))
}

/// `rd.simple_reflect(s, numerator, offset)` (rootdata.h:617-618) on a
/// rational weight's i64 numerator: `v -= (<v, alpha_s^v> + offset) * alpha_s`.
fn reflect_numerator(
    system: &RootSystem,
    generator: usize,
    numerator: &mut [i64],
    offset: i64,
) -> Result<(), StructureError> {
    let simple = simple_root_id(system, generator)?;
    let coroot = coroot_coords(system, simple)?;
    let mut pairing = offset;
    for (&entry, &coroot_entry) in numerator.iter().zip(coroot) {
        let product = entry
            .checked_mul(i64::from(coroot_entry))
            .ok_or(StructureError::ArithmeticOverflow)?;
        pairing = pairing
            .checked_add(product)
            .ok_or(StructureError::ArithmeticOverflow)?;
    }
    let root = root_coords(system, simple)?.to_vec();
    for (entry, &root_entry) in numerator.iter_mut().zip(&root) {
        let shift = pairing
            .checked_mul(i64::from(root_entry))
            .ok_or(StructureError::ArithmeticOverflow)?;
        *entry = entry
            .checked_sub(shift)
            .ok_or(StructureError::ArithmeticOverflow)?;
    }
    Ok(())
}

/// Upstream `RootDatum::act(ww, gamma)` for a rational weight
/// (rootdata.h:665 + rootdata.cpp:1179-1180): the numerator is reflected
/// letter by letter in reverse word order, the denominator is kept.
fn act_word_rational(
    system: &RootSystem,
    kappa: &[usize],
    value: &RationalWeight,
) -> Result<RationalWeight, StructureError> {
    let mut numerator = value.numerator().to_vec();
    for &generator in kappa.iter().rev() {
        reflect_numerator(system, generator, &mut numerator, 0)?;
    }
    RationalWeight::new(numerator, value.denominator())
}

/// Upstream `RootDatum::act(ww, v)` (rootdata.h:651-657) on an integral
/// weight: simple reflections in reverse word order.
fn act_word_weight(
    system: &RootSystem,
    kappa: &[usize],
    value: &mut [i32],
) -> Result<(), StructureError> {
    for &generator in kappa.iter().rev() {
        reflect_coords(system, simple_root_id(system, generator)?, value)?;
    }
    Ok(())
}

/// The complex-orbit kappa reflection of an extended parameter (the
/// `eval<0` branch shared by all three drivers, ext_block.cpp:2480-2490,
/// 2632-2644, 2769-2781): twisted conjugation of `tw` (reverse word order,
/// weyl.h:549-555), `rd.act` on `gamma_lambda` and `tau`, the
/// rho-check-shifted simple coreflections of `l`, `dual_act` (forward word
/// order, rootdata.h:697-700) on `t`, and the flip recording every
/// two-generator (2C) orbit reflection.
fn kappa_reflect(
    ctx: &ExtRepContext,
    e: &mut ExtParam,
    kappa: &[usize],
) -> Result<(), StructureError> {
    let rc = ctx.rc();
    let system = rc.root_system();
    let table = rc.table();
    for &generator in kappa.iter().rev() {
        e.involution = table.cross(generator, e.involution)?;
    }
    e.gamma_lambda = act_word_rational(system, kappa, &e.gamma_lambda)?;
    let mut tau = e.tau.as_slice().to_vec();
    act_word_weight(system, kappa, &mut tau)?;
    e.tau = Weight::new(tau);
    let mut l = e.l.as_slice().to_vec();
    for &generator in kappa {
        let simple = simple_root_id(system, generator)?;
        let shift =
            rational_coweight_dot(rc.g_rho_check().coordinates(), root_coords(system, simple)?)?
                .wrapping_neg();
        coreflect_coords(system, simple, &mut l, shift)?;
    }
    e.l = Coweight::new(l);
    let mut t = e.t.as_slice().to_vec();
    for &generator in kappa {
        coreflect_coords(system, simple_root_id(system, generator)?, &mut t, 0)?;
    }
    e.t = Coweight::new(t);
    e.flip(kappa.len() == 2); // record flip for every 2C+/2C- done
    Ok(())
}

/// The singular complex descent step shared by all three drivers
/// (ext_block.cpp:2493-2504, 2650-2661, 2787-2798): `star` at the orbit of
/// the simple root produces the unique cross/Cayley link, and the extra
/// flip `star` records on October surprises is undone.
fn singular_complex_descent(
    ctx: &ExtRepContext,
    e: &mut ExtParam,
    orbit: &ExtGen,
) -> Result<(), StructureError> {
    let n_alpha = simple_root_id(ctx.rc().root_system(), orbit.s0)?.index();
    let (dtype, links) = star(ctx, e, orbit.length(), n_alpha)?;
    debug_assert!(
        dtype.is_complex() || matches!(dtype, DescValue::TwoSemiReal | DescValue::ThreeSemiReal)
    );
    debug_assert_eq!(links.len(), 1); // just one cross or Cayley link
    let mut next = links
        .into_iter()
        .next()
        .ok_or(StructureError::RepInvariantViolation {
            invariant: "extended finalise complex descent link",
        })?;
    next.flip(dtype.has_october_surprise()); // undo the extra flip in |star|
    *e = next;
    Ok(())
}

/// Upstream `ext_param::restrict_K` (ext_block.cpp:2413-2420): the K-type
/// `sr_K(x, (gamma2 - 2*rho)/2 - gamma_lambda)` at twice-gamma `gamma2`.
fn restrict_k(ctx: &ExtRepContext, e: &ExtParam, gamma2: &Weight) -> Result<KType, StructureError> {
    let rc = ctx.rc();
    let doubled = checked_sub_weights(gamma2, rc.two_rho())?; // now 2*(gamma-rho)
    let numerator = doubled.as_slice().iter().map(|&c| i64::from(c)).collect();
    let gamma_rho = RationalWeight::new(numerator, 2)?;
    let lambda_rho = integer_diff(&gamma_rho, &e.gamma_lambda)?;
    KType::sr_k(rc, e.x(ctx)?, &Weight::new(lambda_rho))
}

/// Scalar multiple of a rational weight by `num/den` (upstream
/// `RatWeight * RatNum`).
fn scale_rational_weight(
    value: &RationalWeight,
    num: i64,
    den: i64,
) -> Result<RationalWeight, StructureError> {
    let mut numerator = Vec::new();
    numerator
        .try_reserve_exact(value.rank())
        .map_err(|_| StructureError::AllocationFailed {
            requested: value.rank(),
        })?;
    for &entry in value.numerator() {
        numerator.push(
            entry
                .checked_mul(num)
                .ok_or(StructureError::ArithmeticOverflow)?,
        );
    }
    let denominator = value
        .denominator()
        .checked_mul(den)
        .ok_or(StructureError::ArithmeticOverflow)?;
    RationalWeight::new(numerator, denominator)
}

/// Upstream `ext_block::extended_restrict_to_K` (ext_block.cpp:2435-2547):
/// restrict the extended parameter at `sr` to `K`, pushing `nu` to zero and
/// finalising the resulting twice-gamma `gamma2 = (1+theta)*lambda` through
/// folded-orbit reflections and descents, tracking the inherited extension
/// data to detect the net flip against the default extension. `sr` must be
/// standard and fixed by `ctx`'s `delta` (both language-layer checked).
///
/// Returns the surviving K-types with their `Split` coefficients as
/// `(e, f)` pairs (`(1,0)` for a default-aligned survivor, `(0,1)` for a
/// flipped one), like terms merged as upstream's `K_type_pol::add_term`
/// does.
pub fn extended_restrict_to_k(
    ctx: &ExtRepContext,
    sr: &StandardRepr,
) -> Result<Vec<(KType, (i32, i32))>, StructureError> {
    let rc = ctx.rc();
    debug_assert!(matches!(sr.is_standard(rc), Ok(true)));
    debug_assert!(rc.is_fixed(sr, ctx.delta())); // delta-fixed
    let system = rc.root_system();
    let table = rc.table();
    let orbits = fold_orbits(rc.datum().cartan_matrix(), &ctx.twist)?;

    // The K-type of `sr` at nu==0, and twice its gamma (theta-fixed).
    let restricted_sr = rc.sr_k_of_standard(sr)?;
    let gamma2_start = restricted_sr.theta_plus_1_lambda(rc)?;

    let e0 = shifted_default_extension(
        ctx,
        sr,
        &RationalWeight::new(
            gamma2_start
                .as_slice()
                .iter()
                .map(|&c| i64::from(c))
                .collect(),
            2,
        )?,
    )?;

    let mut result: Vec<(KType, (i32, i32))> = Vec::new();
    let mut to_do: VecDeque<(ExtParam, Weight)> = VecDeque::new();
    to_do.push_back((e0, gamma2_start));
    while let Some((mut e, mut gamma2)) = to_do.pop_front() {
        let mut i_theta = e.theta_id(ctx)?;
        let mut dropped = false;
        // Upstream's `restart:` label: any state change re-scans all orbits.
        'restart: loop {
            for orbit in &orbits {
                let s = orbit.s0;
                match simple_kind(table, i_theta, s)? {
                    RootKind::Complex => {
                        let eval =
                            vec_dot(gamma2.as_slice(), rc.datum().simple_coroots()[s].as_slice());
                        if eval < 0 {
                            // Complex reflections: anti-dominant to dominant.
                            let kappa = kappa_word(orbit);
                            let mut coordinates = gamma2.as_slice().to_vec();
                            act_word_weight(system, &kappa, &mut coordinates)?;
                            gamma2 = Weight::new(coordinates);
                            kappa_reflect(ctx, &mut e, &kappa)?;
                            i_theta = e.theta_id(ctx)?;
                            continue 'restart;
                        } else if eval == 0 && simple_complex_is_descent(rc, i_theta, s)? {
                            singular_complex_descent(ctx, &mut e, orbit)?;
                            i_theta = e.theta_id(ctx)?;
                            continue 'restart;
                        }
                        // else a complex ascent; skip this orbit
                    }
                    RootKind::Real => {
                        debug_assert_eq!(
                            vec_dot(gamma2.as_slice(), rc.datum().simple_coroots()[s].as_slice()),
                            0
                        ); // so gamma2 is unchanged
                        let simple = simple_root_id(system, s)?;
                        if rat_dot(&e.gamma_lambda, coroot_coords(system, simple)?) % 2 != 0 {
                            continue; // nonparity: an ascent; skip this orbit
                        }
                        let (dtype, links) = star(ctx, &e, orbit.length(), simple.index())?;
                        if dtype.is_like_compact() {
                            // Real parity switched has zero descents.
                            dropped = true;
                            break 'restart;
                        }
                        debug_assert!(!links.is_empty());
                        let flip = dtype.has_october_surprise();
                        let mut links = links.into_iter();
                        let mut next =
                            links.next().ok_or(StructureError::RepInvariantViolation {
                                invariant: "extended restrict_to_K real descent link",
                            })?;
                        i_theta = next.theta_id(ctx)?;
                        next.flip(flip); // undo the extra flip in |star|
                        e = next;
                        if dtype.has_double_image() {
                            // Queue the second image with the same undo.
                            let mut second =
                                links.next().ok_or(StructureError::RepInvariantViolation {
                                    invariant: "extended restrict_to_K double image link",
                                })?;
                            second.flip(flip);
                            to_do.push_back((second, gamma2.clone()));
                        }
                        continue 'restart;
                    }
                    RootKind::Imaginary => {
                        let eval =
                            vec_dot(gamma2.as_slice(), rc.datum().simple_coroots()[s].as_slice());
                        debug_assert!(eval >= 0); // the K-type remains standard
                        if eval == 0 {
                            // Singular: drop when compact.
                            let simple = simple_root_id(system, s)?;
                            let gml = g_minus_l(ctx, &e.l);
                            if rational_coweight_dot(&gml, root_coords(system, simple)?)? % 2 != 0 {
                                dropped = true;
                                break 'restart;
                            }
                        }
                    }
                }
            }
            break 'restart;
        }
        if dropped {
            continue; // upstream's `drop:` label
        }
        // Contribute with coefficient s^(not is_default(E)).
        let ktype = restrict_k(ctx, &e, &gamma2)?;
        let coefficient = if is_default(ctx, &e)? { (1, 0) } else { (0, 1) };
        if let Some(term) = result.iter_mut().find(|(k, _)| *k == ktype) {
            term.1 .0 = term.1 .0.wrapping_add(coefficient.0);
            term.1 .1 = term.1 .1.wrapping_add(coefficient.1);
        } else {
            result.push((ktype, coefficient));
        }
    }
    Ok(result)
}

/// Upstream `ext_block::extended_finalise` (ext_block.cpp:2598-2721): make
/// `gamma` dominant and the parameter final through folded-orbit
/// reflections and descents, keeping `nu` (real anti-dominance is fixed by
/// `-rho`-centered reflections of `gamma_lambda`, marked experimental
/// upstream), and return each survivor paired with its net flip against the
/// default extension (`not is_default(E)`). `sr` must be standard and fixed
/// by `ctx`'s `delta` (both language-layer checked).
///
/// The result is in upstream queue order, NOT merged or sorted; the
/// language layer merges like terms and sorts with the `SR_poly`
/// comparator (repr.cpp:41-54).
pub fn extended_finalise(
    ctx: &ExtRepContext,
    sr: &StandardRepr,
) -> Result<Vec<(StandardRepr, bool)>, StructureError> {
    let rc = ctx.rc();
    debug_assert!(matches!(sr.is_standard(rc), Ok(true)));
    debug_assert!(rc.is_fixed(sr, ctx.delta())); // delta-fixed
    let system = rc.root_system();
    let table = rc.table();
    let orbits = fold_orbits(rc.datum().cartan_matrix(), &ctx.twist)?;

    let e0 = default_extend(ctx, sr)?; // start extension at |sr|

    let mut result: Vec<(StandardRepr, bool)> = Vec::new();
    let mut to_do: VecDeque<(ExtParam, RationalWeight)> = VecDeque::new();
    to_do.push_back((e0, sr.gamma().clone()));
    while let Some((mut e, mut gamma)) = to_do.pop_front() {
        let mut i_theta = e.theta_id(ctx)?;
        let mut dropped = false;
        // Upstream's `restart:` label: any state change re-scans all orbits.
        'restart: loop {
            for orbit in &orbits {
                let s = orbit.s0;
                match simple_kind(table, i_theta, s)? {
                    RootKind::Complex => {
                        let eval = rc.simple_coroot_numerator_pairing(s, gamma.numerator())?;
                        if eval < 0 {
                            // Complex reflections: anti-dominant to dominant.
                            let kappa = kappa_word(orbit);
                            gamma = act_word_rational(system, &kappa, &gamma)?;
                            kappa_reflect(ctx, &mut e, &kappa)?;
                            i_theta = e.theta_id(ctx)?;
                            continue 'restart;
                        } else if eval == 0 && simple_complex_is_descent(rc, i_theta, s)? {
                            // The reflections fix gamma; only E descends.
                            singular_complex_descent(ctx, &mut e, orbit)?;
                            i_theta = e.theta_id(ctx)?;
                            continue 'restart;
                        }
                        // else a complex ascent; skip this orbit
                    }
                    RootKind::Real => {
                        let eval = rc.simple_coroot_numerator_pairing(s, gamma.numerator())?;
                        if eval < 0 {
                            let kappa = kappa_word(orbit);
                            gamma = act_word_rational(system, &kappa, &gamma)?;
                            // The $-\rho$-centered reflections on
                            // gamma_lambda (per letter, forward order).
                            let mut numerator = e.gamma_lambda.numerator().to_vec();
                            let denominator = e.gamma_lambda.denominator();
                            for &generator in &kappa {
                                reflect_numerator(system, generator, &mut numerator, denominator)?;
                            }
                            e.gamma_lambda = RationalWeight::new(numerator, denominator)?;
                            let mut tau = e.tau.as_slice().to_vec();
                            act_word_weight(system, &kappa, &mut tau)?;
                            e.tau = Weight::new(tau);
                            e.flip(kappa.len() == 2); // one flip per "two real" reflection
                            continue 'restart;
                        } else if eval == 0 {
                            let simple = simple_root_id(system, s)?;
                            if rat_dot(&e.gamma_lambda, coroot_coords(system, simple)?) % 2 != 0 {
                                continue; // nonparity: an ascent; skip this orbit
                            }
                            let (dtype, links) = star(ctx, &e, orbit.length(), simple.index())?;
                            if dtype.is_like_compact() {
                                // Real parity switched has zero descents.
                                dropped = true;
                                break 'restart;
                            }
                            debug_assert!(!links.is_empty());
                            let flip = dtype.has_october_surprise();
                            let mut links = links.into_iter();
                            let mut next =
                                links.next().ok_or(StructureError::RepInvariantViolation {
                                    invariant: "extended finalise real descent link",
                                })?;
                            i_theta = next.theta_id(ctx)?;
                            next.flip(flip); // undo the extra flip in |star|
                            e = next;
                            if dtype.has_double_image() {
                                // Queue the second image with the same undo.
                                let mut second =
                                    links.next().ok_or(StructureError::RepInvariantViolation {
                                        invariant: "extended finalise double image link",
                                    })?;
                                second.flip(flip);
                                to_do.push_back((second, gamma.clone()));
                            }
                            continue 'restart;
                        }
                        // else a real ascent; skip this orbit
                    }
                    RootKind::Imaginary => {
                        let eval = rc.simple_coroot_numerator_pairing(s, gamma.numerator())?;
                        debug_assert!(eval >= 0); // parameters remain standard here
                        if eval == 0 {
                            // Singular: drop when compact.
                            let simple = simple_root_id(system, s)?;
                            let gml = g_minus_l(ctx, &e.l);
                            if rational_coweight_dot(&gml, root_coords(system, simple)?)? % 2 != 0 {
                                dropped = true;
                                break 'restart;
                            }
                        }
                    }
                }
            }
            break 'restart;
        }
        if dropped {
            continue; // upstream's `drop:` label
        }
        // Contribute E here with Boolean |not is_default(E)|.
        let flipped = !is_default(ctx, &e)?;
        result.push((e.restrict(ctx, &gamma)?, flipped));
    }
    Ok(result)
}

/// Upstream `ext_block::scaled_extended_finalise` (ext_block.cpp:2736-2807):
/// scale `nu` by the strictly positive `factor_num/factor_den` and rewrite
/// as a final parameter at the extended-parameter level, returning the net
/// flip of the default extension choice. Since the factor is positive, no
/// regular real root becomes singular, so only complex orbits are treated
/// (for dominance and for finality). `sr` must be final and fixed by
/// `ctx`'s `delta` (both language-layer checked).
pub fn scaled_extended_finalise(
    ctx: &ExtRepContext,
    sr: &StandardRepr,
    factor_num: i64,
    factor_den: i64,
) -> Result<(StandardRepr, bool), StructureError> {
    let rc = ctx.rc();
    debug_assert!(matches!(sr.is_final(rc), Ok(true)));
    debug_assert!(factor_num > 0 && factor_den > 0); // no real root becomes singular
    debug_assert!(rc.is_fixed(sr, ctx.delta())); // delta-fixed
    let system = rc.root_system();
    let table = rc.table();
    let orbits = fold_orbits(rc.datum().cartan_matrix(), &ctx.twist)?;

    // First approximation to the result is the scaled input; importantly,
    // lambda (equivalently lambda_rho) is held fixed here.
    let scaled_nu = scale_rational_weight(&rc.nu(sr)?, factor_num, factor_den)?;
    let scaled_sr = rc.sr(sr.x(), &rc.lambda_rho(sr)?, &scaled_nu)?;
    let mut gamma = scaled_sr.gamma().clone();

    let mut e = default_extend(ctx, sr)?; // start extension at |sr|
                                          // Shift gamma_lambda by the nu change.
    e.gamma_lambda = e.gamma_lambda.add(&gamma.sub(sr.gamma())?)?;

    // Only complex coroots need treatment (both for dominance and for
    // finality); upstream's `restart:` label re-scans all orbits on change.
    let mut i_theta = e.theta_id(ctx)?;
    'restart: loop {
        for orbit in &orbits {
            let s = orbit.s0;
            if simple_kind(table, i_theta, s)? != RootKind::Complex {
                continue; // no |else| upstream: real/imaginary need no work
            }
            let eval = rc.simple_coroot_numerator_pairing(s, gamma.numerator())?;
            if eval < 0 {
                // Complex reflections: anti-dominant to dominant.
                let kappa = kappa_word(orbit);
                gamma = act_word_rational(system, &kappa, &gamma)?;
                kappa_reflect(ctx, &mut e, &kappa)?;
                i_theta = e.theta_id(ctx)?;
                continue 'restart;
            } else if eval == 0 && simple_complex_is_descent(rc, i_theta, s)? {
                // The reflections fix gamma; only E descends.
                singular_complex_descent(ctx, &mut e, orbit)?;
                i_theta = e.theta_id(ctx)?;
                continue 'restart;
            }
            // else a complex ascent; skip this orbit
        }
        break 'restart;
    }
    let flipped = !is_default(ctx, &e)?;
    Ok((e.restrict(ctx, &gamma)?, flipped))
}

// ---------------------------------------------------------------------------
// Tests.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AdjointFiberBudget, BasedRootDatum, CartanClassification, CartanClassificationBudget,
        CartanId, InnerClass, IntegerLatticeBudget, InvolutionTableBudget, KgbGraph, RealFormSeed,
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

    /// The split sl(2,R) context (compact inner class, KGB size 3).
    fn a1_fixture() -> ContextFixture {
        let datum = a1_datum();
        let involution = LatticeInvolution::identity(&datum).unwrap();
        fixture(datum, involution, 2, 3)
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

    /// The quasisplit su(2,1) context (compact inner class, KGB size 6).
    fn a2_compact_fixture() -> ContextFixture {
        let datum = a2_datum();
        let involution = LatticeInvolution::identity(&datum).unwrap();
        fixture(datum, involution, 8, 6)
    }

    /// The split sl(3,R) context (split inner class, KGB size 4).
    fn a2_split_fixture() -> ContextFixture {
        let datum = a2_datum();
        let swap = LatticeInvolution::new(
            &datum,
            vec![vec![0, 1], vec![1, 0]],
            vec![vec![0, 1], vec![1, 0]],
        )
        .unwrap();
        fixture(datum, swap, 8, 4)
    }

    /// The simply connected B2 datum (roots are Cartan rows).
    fn b2_datum() -> BasedRootDatum {
        BasedRootDatum::from_simple_data(
            2,
            vec![vec![2, -2], vec![-1, 2]],
            vec![Weight::new(vec![2, -2]), Weight::new(vec![-1, 2])],
            vec![Coweight::new(vec![1, 0]), Coweight::new(vec![0, 1])],
        )
        .unwrap()
    }

    /// The split so(3,2) context (compact inner class, KGB size 11).
    fn b2_fixture() -> ContextFixture {
        let datum = b2_datum();
        let involution = LatticeInvolution::identity(&datum).unwrap();
        fixture(datum, involution, 8, 11)
    }

    fn identity_delta(rc: &RepContext) -> LatticeInvolution {
        LatticeInvolution::identity(rc.datum()).unwrap()
    }

    fn swap_delta(rc: &RepContext) -> LatticeInvolution {
        LatticeInvolution::new(
            rc.datum(),
            vec![vec![0, 1], vec![1, 0]],
            vec![vec![0, 1], vec![1, 0]],
        )
        .unwrap()
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

    /// Sort `(StandardRepr, flip)` terms by descending `x` (the `SR_poly`
    /// display order) for order-independent comparison.
    fn sorted_params(terms: &[(StandardRepr, bool)]) -> Vec<(usize, StandardRepr, bool)> {
        let mut keyed: Vec<(usize, StandardRepr, bool)> = terms
            .iter()
            .map(|(sr, flip)| (sr.x().0, sr.clone(), *flip))
            .collect();
        keyed.sort_by_key(|a| std::cmp::Reverse(a.0));
        keyed
    }

    /// Sort `(KType, coef)` terms by ascending `x` (the `K_type_pol` display
    /// order) for order-independent comparison.
    fn sorted_ktypes(terms: &[(KType, (i32, i32))]) -> Vec<(usize, KType, (i32, i32))> {
        let mut keyed: Vec<(usize, KType, (i32, i32))> = terms
            .iter()
            .map(|(k, c)| (k.x().0, k.clone(), *c))
            .collect();
        keyed.sort_by_key(|a| a.0);
        keyed
    }

    // Oracle-pinned values from the verified fixture `domain/ext_finalise`
    // (reference capture job 3538977, oracle rev 4d3e944).

    #[test]
    fn default_extensions_expose_their_source_involution_ids() {
        let fixtures: Vec<(ContextFixture, fn(&RepContext) -> LatticeInvolution)> = vec![
            (a1_fixture(), identity_delta),
            (a2_split_fixture(), swap_delta),
            (b2_fixture(), identity_delta),
        ];

        for (fixture, delta) in fixtures {
            let rc = fixture.rc();
            let ctx = ExtRepContext::new(&rc, delta(&rc)).unwrap();
            for x in 0..rc.graph().size() {
                let x = KgbId(x);
                let id = rc.involution_of(x).unwrap();
                let extension =
                    default_extend_srm(&ctx, x, RationalWeight::zero(rc.rank()).unwrap()).unwrap();

                assert_eq!(extension.involution_id(), id);
                assert_eq!(extension.theta_id(&ctx).unwrap(), id);
                assert_eq!(
                    extension.theta(&ctx).unwrap(),
                    rc.table().record(id).unwrap().theta()
                );
                assert_eq!(rc.involution_of(extension.x(&ctx).unwrap()).unwrap(), id);
            }
        }
    }

    #[test]
    fn scaled_a1_compact_cartan_is_unchanged() {
        // scale_extended(pa,[[1]],2/1) = (pa,false), pa = (x=0,[0],[0]/1).
        let fixture = a1_fixture();
        let rc = fixture.rc();
        let pa = param(&rc, 0, &[0], &[0], 1);
        assert!(matches!(pa.is_final(&rc), Ok(true)));
        let ctx = ExtRepContext::new(&rc, identity_delta(&rc)).unwrap();
        let (result, flip) = scaled_extended_finalise(&ctx, &pa, 2, 1).unwrap();
        assert_eq!(result, pa);
        assert!(!flip);
    }

    #[test]
    fn scaled_a1_split_cartan_scales_nu() {
        // scale_extended(pa2,[[1]],3/2) = (param(x=2,nu=[3]/2),false).
        let fixture = a1_fixture();
        let rc = fixture.rc();
        let pa2 = param(&rc, 2, &[0], &[1], 1);
        assert!(matches!(pa2.is_final(&rc), Ok(true)));
        let ctx = ExtRepContext::new(&rc, identity_delta(&rc)).unwrap();
        let (result, flip) = scaled_extended_finalise(&ctx, &pa2, 3, 2).unwrap();
        assert_eq!(result, param(&rc, 2, &[0], &[3], 2));
        assert!(!flip);
    }

    #[test]
    fn scaled_b2_parameter_keeps_lambda_rho_and_scales_only_nu() {
        let fixture = b2_fixture();
        let rc = fixture.rc();
        let parameter = param(&rc, 5, &[1, 1], &[1, -1], 1);
        assert!(matches!(parameter.is_final(&rc), Ok(true)));
        let ctx = ExtRepContext::new(&rc, identity_delta(&rc)).unwrap();

        let (scaled, flip) = scaled_extended_finalise(&ctx, &parameter, 1, 2).unwrap();

        assert_eq!(rc.lambda_rho(&scaled).unwrap(), weight(&[1, 1]));
        assert_eq!(rc.nu(&scaled).unwrap(), rational(&[1, -1], 2));
        assert_eq!(scaled, param(&rc, 5, &[1, 1], &[1, -1], 2));
        assert!(!flip);
    }

    #[test]
    fn a2_quasisplit_trivial_drivers() {
        // qc = (x=0,[0,0],[0,0]/1) in su(2,1), delta identity:
        // scale 2 -> (qc,false); finalize -> 1*qc; K-pol -> 1*K_type(x=0).
        let fixture = a2_compact_fixture();
        let rc = fixture.rc();
        let qc = param(&rc, 0, &[0, 0], &[0, 0], 1);
        let ctx = ExtRepContext::new(&rc, identity_delta(&rc)).unwrap();

        let (scaled, flip) = scaled_extended_finalise(&ctx, &qc, 2, 1).unwrap();
        assert_eq!(scaled, qc);
        assert!(!flip);

        let terms = extended_finalise(&ctx, &qc).unwrap();
        assert_eq!(terms, vec![(qc.clone(), false)]);

        let kterms = extended_restrict_to_k(&ctx, &qc).unwrap();
        let expected = KType::sr_k(&rc, KgbId(0), &weight(&[0, 0])).unwrap();
        assert_eq!(kterms, vec![(expected, (1, 0))]);
    }

    #[test]
    fn a2_split_trivial_drivers() {
        // q = (x=0,[0,0],[0,0]/1) in sl(3,R), delta the diagram swap:
        // scale 2 -> (q,false); finalize -> 1*q; K-pol -> 1*K_type(x=0).
        let fixture = a2_split_fixture();
        let rc = fixture.rc();
        let q = param(&rc, 0, &[0, 0], &[0, 0], 1);
        let ctx = ExtRepContext::new(&rc, swap_delta(&rc)).unwrap();

        let (scaled, flip) = scaled_extended_finalise(&ctx, &q, 2, 1).unwrap();
        assert_eq!(scaled, q);
        assert!(!flip);

        let terms = extended_finalise(&ctx, &q).unwrap();
        assert_eq!(terms, vec![(q.clone(), false)]);

        let kterms = extended_restrict_to_k(&ctx, &q).unwrap();
        let expected = KType::sr_k(&rc, KgbId(0), &weight(&[0, 0])).unwrap();
        assert_eq!(kterms, vec![(expected, (1, 0))]);
    }

    #[test]
    fn a2_split_nonfinal_flips_to_compact_term() {
        // p = (x=3,[1,1],[0,0]/1) in sl(3,R), delta the swap:
        // finalize -> 1s*param(x=0,lambda=[0,0]) and
        // K-pol -> 1s*K_type(x=0,lambda=[0,0]) (lambda=[0,0] displays;
        // lambda_rho = [0,0]-rho = [-1,-1]).
        let fixture = a2_split_fixture();
        let rc = fixture.rc();
        let p = param(&rc, 3, &[1, 1], &[0, 0], 1);
        assert!(matches!(p.is_standard(&rc), Ok(true)));
        let ctx = ExtRepContext::new(&rc, swap_delta(&rc)).unwrap();

        let terms = extended_finalise(&ctx, &p).unwrap();
        let expected = param(&rc, 0, &[-1, -1], &[0, 0], 1);
        assert_eq!(terms, vec![(expected, true)]);

        let kterms = extended_restrict_to_k(&ctx, &p).unwrap();
        let expected_k = KType::sr_k(&rc, KgbId(0), &weight(&[-1, -1])).unwrap();
        assert_eq!(kterms, vec![(expected_k, (0, 1))]);
    }

    #[test]
    fn b2_split_nondominant_two_term_expansion() {
        // pb = (x=8,[0,0],[0,0]/1) in so(3,2), delta identity:
        // finalize -> 1*param(x=1,lambda=[0,1]) + 1*param(x=0,lambda=[0,1])
        // (ParamPol display order: height asc, then x DESC) and
        // K-pol -> 1*K_type(x=0) + 1*K_type(x=1) (x ASC);
        // lambda=[0,1] means lambda_rho = [0,1]-rho = [-1,0].
        let fixture = b2_fixture();
        let rc = fixture.rc();
        let pb = param(&rc, 8, &[0, 0], &[0, 0], 1);
        assert!(matches!(pb.is_standard(&rc), Ok(true)));
        assert!(matches!(pb.is_dominant(&rc), Ok(false)));
        let ctx = ExtRepContext::new(&rc, identity_delta(&rc)).unwrap();

        let terms = extended_finalise(&ctx, &pb).unwrap();
        let expected: Vec<(usize, StandardRepr, bool)> = vec![
            (1, param(&rc, 1, &[-1, 0], &[0, 0], 1), false),
            (0, param(&rc, 0, &[-1, 0], &[0, 0], 1), false),
        ];
        assert_eq!(sorted_params(&terms), expected);

        let kterms = extended_restrict_to_k(&ctx, &pb).unwrap();
        let expected_k: Vec<(usize, KType, (i32, i32))> = vec![
            (
                0,
                KType::sr_k(&rc, KgbId(0), &weight(&[-1, 0])).unwrap(),
                (1, 0),
            ),
            (
                1,
                KType::sr_k(&rc, KgbId(1), &weight(&[-1, 0])).unwrap(),
                (1, 0),
            ),
        ];
        assert_eq!(sorted_ktypes(&kterms), expected_k);
    }
}
