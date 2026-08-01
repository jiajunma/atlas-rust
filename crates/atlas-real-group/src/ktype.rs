//! The `K_type` value and its `Rep_context` operations
//! (gkmod/K_repr.h, gkmod/K_repr.cpp).
//!
//! A [`KType`] is the K-restriction of a standard representation, or an
//! irreducible representation of K: the parameter data of
//! [`crate::StandardRepr`] without the `nu` part (K_repr.h:25-30). It
//! stores the ELECTED `lambda-rho` representative — normalization modulo
//! `(1-theta_x)X^*` happens once in [`KType::sr_k`]
//! (`Rep_context::sr_K`, K_repr.cpp:25-32) — so value equality is the
//! upstream strict component equality (K_repr.h:56-60), and the
//! `equivalent` relation is computed by moving to the canonical
//! involution (K_repr.cpp:159-171).

use crate::lattice::{checked_add_weights, checked_sub_weights, pair};
use crate::rep_context::RepContext;
use crate::{KgbId, KgbStatus, RootId, StructureError, Weight};

/// A K-type: upstream `K_repr::K_type` (K_repr.h:30-86). The stored
/// `lam_rho` is always the `lambda_unique` representative of its
/// `(1-theta_x)X^*`-coset, and `height` is precomputed at construction
/// (K_repr.h:36-44).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KType {
    x: KgbId,
    lam_rho: Weight,
    height: u32,
}

impl KType {
    /// The raw value constructor; normalization is [`KType::sr_k`]'s job,
    /// matching the upstream struct constructors (K_repr.h:39-44).
    pub(crate) fn new(x: KgbId, lam_rho: Weight, height: u32) -> Self {
        Self { x, lam_rho, height }
    }

    /// `Rep_context::sr_K(KGBElt, Weight)` (K_repr.cpp:25-32): normalize
    /// `lambda_rho` modulo `(1-theta_x)X^*` and store the height of
    /// `(1+theta_x)*lambda`.
    pub fn sr_k(rc: &RepContext, x: KgbId, lambda_rho: &Weight) -> Result<Self, StructureError> {
        let involution = rc.involution_of(x)?;
        let normalized = rc.lambda_unique(involution, lambda_rho)?;
        let theta = rc.theta_at(x)?;
        let th1_lambda = checked_add_weights(
            &checked_add_weights(&normalized, &theta.act_on_weight(&normalized)?)?,
            &rc.theta_plus_one_rho_at(x)?,
        )?;
        let height = rc.height(&th1_lambda)?;
        Ok(Self::new(x, normalized, height))
    }

    pub fn x(&self) -> KgbId {
        self.x
    }

    /// The elected `lambda-rho` representative (K_repr.h:49).
    pub fn lambda_rho(&self) -> &Weight {
        &self.lam_rho
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    /// `Rep_context::theta_plus_1_lambda` (K_repr.cpp:16-23):
    /// `(1+theta_x)*lambda = lambda_rho + theta*lambda_rho + (1+theta)rho`.
    fn theta_plus_1_lambda(&self, rc: &RepContext) -> Result<Weight, StructureError> {
        let theta = rc.theta_at(self.x)?;
        checked_add_weights(
            &checked_add_weights(&self.lam_rho, &theta.act_on_weight(&self.lam_rho)?)?,
            &rc.theta_plus_one_rho_at(self.x)?,
        )
    }

    /// `Rep_context::theta_plus_1_eval` (K_repr.cpp:35-43):
    /// `<alpha^v + (theta alpha)^v, lambda>` as an integer, needed for
    /// its sign (and zero test) throughout the predicate set.
    fn theta_plus_1_eval(&self, rc: &RepContext, alpha: RootId) -> Result<i32, StructureError> {
        let system = rc.inner_class().root_system();
        let beta = rc.root_involution_image_at(self.x, alpha)?;
        let coroot = system
            .coroot(alpha)
            .ok_or(StructureError::IndexOutOfRange {
                index: alpha.0,
                upper_bound: system.roots().len(),
            })?;
        let beta_coroot = system.coroot(beta).ok_or(StructureError::IndexOutOfRange {
            index: beta.0,
            upper_bound: system.roots().len(),
        })?;
        let first = pair(&self.lam_rho, coroot)?
            .checked_add(rc.colevel(alpha)?)
            .ok_or(StructureError::ArithmeticOverflow)?;
        let second = pair(&self.lam_rho, beta_coroot)?
            .checked_add(rc.colevel(beta)?)
            .ok_or(StructureError::ArithmeticOverflow)?;
        first
            .checked_add(second)
            .ok_or(StructureError::ArithmeticOverflow)
    }

    /// `Rep_context::is_standard` (K_repr.cpp:46-57): `lambda` is weakly
    /// dominant on the simply-imaginary coroots.
    pub fn is_standard(&self, rc: &RepContext) -> Result<bool, StructureError> {
        let system = rc.inner_class().root_system();
        for alpha in rc.imaginary_simple_roots_at(self.x)? {
            let coroot = system
                .coroot(alpha)
                .ok_or(StructureError::IndexOutOfRange {
                    index: alpha.0,
                    upper_bound: system.roots().len(),
                })?;
            let eval = pair(&self.lam_rho, coroot)?
                .checked_add(rc.colevel(alpha)?)
                .ok_or(StructureError::ArithmeticOverflow)?;
            if eval < 0 {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// `Rep_context::is_dominant` (K_repr.cpp:59-69): `(1+theta_x)lambda`
    /// is weakly dominant on every simple coroot.
    pub fn is_dominant(&self, rc: &RepContext) -> Result<bool, StructureError> {
        let system = rc.inner_class().root_system();
        for (generator, &simple) in system.simple_root_ids().iter().enumerate() {
            let _ = generator;
            if self.theta_plus_1_eval(rc, simple)? < 0 {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// `Rep_context::is_nonzero` (K_repr.cpp:71-83): no singular compact
    /// simply-imaginary root. Assumes `is_standard`, as upstream does.
    pub fn is_nonzero(&self, rc: &RepContext) -> Result<bool, StructureError> {
        let system = rc.inner_class().root_system();
        for alpha in rc.imaginary_simple_roots_at(self.x)? {
            let coroot = system
                .coroot(alpha)
                .ok_or(StructureError::IndexOutOfRange {
                    index: alpha.0,
                    upper_bound: system.roots().len(),
                })?;
            let eval = pair(&self.lam_rho, coroot)?
                .checked_add(rc.colevel(alpha)?)
                .ok_or(StructureError::ArithmeticOverflow)?;
            if eval == 0 && !rc.simple_imaginary_grading(self.x, alpha)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// `Rep_context::is_semifinal` (K_repr.cpp:85-100): no really-simple
    /// root is odd on the test weight `2*(lambda-rho) + 2rho - 2rho_R`.
    pub fn is_semifinal(&self, rc: &RepContext) -> Result<bool, StructureError> {
        let system = rc.inner_class().root_system();
        let positive_real = rc.positive_real_roots_at(self.x)?;
        let mut doubled = Vec::new();
        doubled
            .try_reserve_exact(self.lam_rho.rank())
            .map_err(|_| StructureError::AllocationFailed {
                requested: self.lam_rho.rank(),
            })?;
        for &entry in self.lam_rho.as_slice() {
            doubled.push(
                entry
                    .checked_mul(2)
                    .ok_or(StructureError::ArithmeticOverflow)?,
            );
        }
        let test_weight = checked_add_weights(
            &Weight::new(doubled),
            &checked_sub_weights(rc.two_rho(), &rc.two_rho_of(&positive_real)?)?,
        )?;
        for alpha in rc.real_simple_roots_at(self.x)? {
            let coroot = system
                .coroot(alpha)
                .ok_or(StructureError::IndexOutOfRange {
                    index: alpha.0,
                    upper_bound: system.roots().len(),
                })?;
            if pair(&test_weight, coroot)? % 4 != 0 {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// `Rep_context::is_normal` (K_repr.cpp:102-123): no singular complex
    /// descent. Upstream asserts `is_standard && is_dominant &&
    /// is_nonzero && is_semifinal` and is only called in that context
    /// (the interpreter's adjective chain); the computation itself is
    /// total, so this port evaluates it without the precondition.
    pub fn is_normal(&self, rc: &RepContext) -> Result<bool, StructureError> {
        let system = rc.inner_class().root_system();
        for &simple in system.simple_root_ids() {
            if rc.is_complex_descent(self.x, simple_generator(rc, simple)?)?
                && self.theta_plus_1_eval(rc, simple)? == 0
            {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// `Rep_context::is_final` (K_repr.cpp:126-157): dominance of
    /// `(1+theta_x)lambda` and absence of any singular descent.
    pub fn is_final(&self, rc: &RepContext) -> Result<bool, StructureError> {
        let datum = rc.inner_class().datum();
        let system = rc.inner_class().root_system();
        for generator in 0..datum.semisimple_rank() {
            let eval = self.theta_plus_1_eval(rc, system.simple_root_ids()[generator])?;
            if eval < 0 {
                return Ok(false);
            }
            if eval == 0 {
                match rc.kgb_status(self.x, generator)? {
                    KgbStatus::ImaginaryCompact => return Ok(false),
                    KgbStatus::Real => {
                        if pair(&self.lam_rho, &datum.simple_coroots()[generator])? % 2 != 0 {
                            return Ok(false);
                        }
                    }
                    KgbStatus::Complex => {
                        if rc.is_complex_descent(self.x, generator)? {
                            return Ok(false);
                        }
                    }
                    KgbStatus::ImaginaryNoncompact => {}
                }
            }
        }
        Ok(true)
    }

    /// `Rep_context::equivalent` (K_repr.cpp:159-171): same Cartan class,
    /// then strict equality after moving both to the canonical
    /// involution of the class.
    pub fn equivalent(&self, rc: &RepContext, other: &KType) -> Result<bool, StructureError> {
        let self_cartan = rc
            .graph()
            .cartan_of(self.x)
            .ok_or(StructureError::IndexOutOfRange {
                index: self.x.index(),
                upper_bound: rc.graph().size(),
            })?;
        let other_cartan =
            rc.graph()
                .cartan_of(other.x)
                .ok_or(StructureError::IndexOutOfRange {
                    index: other.x.index(),
                    upper_bound: rc.graph().size(),
                })?;
        if self_cartan != other_cartan {
            return Ok(false);
        }
        let left = self.to_canonical_fiber(rc)?;
        let right = other.to_canonical_fiber(rc)?;
        Ok(left == right)
    }

    /// `Rep_context::make_dominant` (K_repr.cpp:174-204): cross through
    /// complex simple roots with negative `(1+theta)lambda` evaluation
    /// until dominant. Non-standard input fails, as upstream's "Non
    /// standard K-type in make_dominant". The stored height is invariant
    /// (the `(1+theta)lambda` weight moves by Weyl conjugates) and is
    /// carried unchanged, as upstream does.
    pub fn made_dominant(&self, rc: &RepContext) -> Result<KType, StructureError> {
        let datum = rc.inner_class().datum();
        let system = rc.inner_class().root_system();
        let mut z = self.clone();
        // The imaginary-dominance check of K_repr.cpp:184-189.
        if !z.is_standard(rc)? {
            return Err(StructureError::RepInvariantViolation {
                invariant: "standard K-type in make_dominant",
            });
        }
        let mut remaining_steps = rc.weight_defect(&z.theta_plus_1_lambda(rc)?)?;
        loop {
            let mut reflected = false;
            for generator in 0..datum.semisimple_rank() {
                if z.theta_plus_1_eval(rc, system.simple_root_ids()[generator])? < 0 {
                    z.x = rc.cross_at(z.x, generator)?;
                    rc.simple_reflect(generator, &mut z.lam_rho, 1)?;
                    reflected = true;
                    break;
                }
            }
            if !reflected {
                return Ok(z);
            }
            remaining_steps -= 1;
            if remaining_steps < 0 {
                return Err(StructureError::RepInvariantViolation {
                    invariant: "dominance termination",
                });
            }
            z.lam_rho = rc.lambda_unique(rc.involution_of(z.x)?, &z.lam_rho)?;
        }
    }

    /// `Rep_context::make_theta_stable` (K_repr.cpp:207-233): exhaust
    /// simple complex descents for the involution. Every cross is a
    /// descent, so the graph size is a generous termination cap.
    pub fn made_theta_stable(&self, rc: &RepContext) -> Result<KType, StructureError> {
        let datum = rc.inner_class().datum();
        let mut z = self.clone();
        for _ in 0..=rc.graph().size() {
            let mut reflected = false;
            for generator in 0..datum.semisimple_rank() {
                if rc.is_complex_descent(z.x, generator)? {
                    z.x = rc.cross_at(z.x, generator)?;
                    rc.simple_reflect(generator, &mut z.lam_rho, 1)?;
                    reflected = true;
                    break;
                }
            }
            if !reflected {
                z.lam_rho = rc.lambda_unique(rc.involution_of(z.x)?, &z.lam_rho)?;
                return Ok(z);
            }
        }
        Err(StructureError::RepInvariantViolation {
            invariant: "theta-stable termination",
        })
    }

    /// `Rep_context::to_canonical_involution` with the full generator set
    /// (K_repr.cpp:236-256): cross along the `InnerClass::canonicalize`
    /// word to the elected fiber of the Cartan class.
    pub fn to_canonical_fiber(&self, rc: &RepContext) -> Result<KType, StructureError> {
        let involution = rc.involution_of(self.x)?;
        let twisted = rc
            .table()
            .record(involution)
            .ok_or(StructureError::IndexOutOfRange {
                index: involution.0,
                upper_bound: rc.table().involution_count(),
            })?
            .twisted_involution()
            .clone();
        let (_, word) = rc.inner_class().canonicalize(twisted)?;
        let mut z = self.clone();
        for generator in word {
            if !rc.is_complex_simple(z.x, generator)? {
                return Err(StructureError::RepInvariantViolation {
                    invariant: "canonical fiber cross",
                });
            }
            z.x = rc.cross_at(z.x, generator)?;
            rc.simple_reflect(generator, &mut z.lam_rho, 1)?;
        }
        z.lam_rho = rc.lambda_unique(rc.involution_of(z.x)?, &z.lam_rho)?;
        Ok(z)
    }

    /// `Rep_context::normalise` (K_repr.cpp:262-289): move to the
    /// canonical involution, then exhaust singular complex descents (and
    /// negative complex evaluations) for a final class member when one
    /// exists.
    pub fn normalised(&self, rc: &RepContext) -> Result<KType, StructureError> {
        let datum = rc.inner_class().datum();
        let system = rc.inner_class().root_system();
        let mut z = self.to_canonical_fiber(rc)?;
        // The negative-evaluation crosses reduce the dominance defect of
        // the `(1+theta)lambda` weight; the singular descent crosses
        // reduce the involution length. Their sum bounds the loop.
        let mut remaining_steps = rc.weight_defect(&z.theta_plus_1_lambda(rc)?)?
            + i64::try_from(rc.graph().size()).map_err(|_| StructureError::ArithmeticOverflow)?
            + 1;
        loop {
            let mut reflected = false;
            for generator in 0..datum.semisimple_rank() {
                let eval = z.theta_plus_1_eval(rc, system.simple_root_ids()[generator])?;
                if rc.kgb_status(z.x, generator)? == KgbStatus::Complex
                    && (eval < 0 || (eval == 0 && rc.is_complex_descent(z.x, generator)?))
                {
                    z.x = rc.cross_at(z.x, generator)?;
                    rc.simple_reflect(generator, &mut z.lam_rho, 1)?;
                    reflected = true;
                    break;
                }
            }
            if !reflected {
                z.lam_rho = rc.lambda_unique(rc.involution_of(z.x)?, &z.lam_rho)?;
                return Ok(z);
            }
            remaining_steps -= 1;
            if remaining_steps < 0 {
                return Err(StructureError::RepInvariantViolation {
                    invariant: "normal form termination",
                });
            }
        }
    }
}

/// Recover a simple root's generator index from its [`RootId`].
fn simple_generator(rc: &RepContext, simple: RootId) -> Result<usize, StructureError> {
    rc.inner_class()
        .root_system()
        .simple_root_ids()
        .iter()
        .position(|&candidate| candidate == simple)
        .ok_or(StructureError::RepInvariantViolation {
            invariant: "simple root generator",
        })
}
