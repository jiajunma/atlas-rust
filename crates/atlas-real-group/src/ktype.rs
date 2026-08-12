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
    pub(crate) fn theta_plus_1_lambda(&self, rc: &RepContext) -> Result<Weight, StructureError> {
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

    /// `Rep_context::finals_for(const K_repr::K_type&)`
    /// (K_repr.cpp:290-396): the final-K-type expansion of this (possibly
    /// non-final) K-type — a signed multiplicity list of final K-types in
    /// its equivalence class, produced by making `(1+theta_x)lambda`
    /// dominant through complex/noncompact-imaginary crosses and Cayley
    /// transforms, dropping singular compact factors, and splitting along
    /// parity real roots. The returned list is unordered; coefficient
    /// merging happens at the language layer in the polynomial's canonical
    /// term order. A final K-type yields exactly itself with multiplicity
    /// one (the loop terminates immediately).
    pub fn finals_for(&self, rc: &RepContext) -> Result<Vec<(KType, i32)>, StructureError> {
        let datum = rc.inner_class().datum();
        let mut result = Vec::new();
        let mut todo = vec![(self.clone(), 1_i32)];
        while let Some((ktype, mut coef)) = todo.pop() {
            let height = ktype.height();
            let mut x = ktype.x();
            let mut lr = ktype.lambda_rho().clone();
            let theta = rc.theta_at(x)?;
            // `im_wt = lr + theta*lr + theta_plus_1_rho`
            // (K_repr.cpp:306-311).
            let mut im_wt = checked_add_weights(
                &checked_add_weights(&lr, &theta.act_on_weight(&lr)?)?,
                &rc.theta_plus_one_rho_at(x)?,
            )?;
            let mut dropped = false;
            'restart: loop {
                for s in 0..datum.semisimple_rank() {
                    let eval = pair(&im_wt, &datum.simple_coroots()[s])?;
                    if eval > 0 {
                        continue;
                    }
                    match rc.kgb_status(x, s)? {
                        KgbStatus::ImaginaryCompact => {
                            if eval < 0 {
                                rc.simple_reflect(s, &mut im_wt, 0)?;
                                rc.simple_reflect(s, &mut lr, 1)?;
                                coef = -coef;
                                continue 'restart;
                            }
                            dropped = true;
                            break 'restart;
                        }
                        KgbStatus::ImaginaryNoncompact => {
                            if eval < 0 {
                                let sx = rc.cross_at(x, s)?;
                                let cx = rc.graph().cayley(x, s)?.ok_or(
                                    StructureError::RepInvariantViolation {
                                        invariant: "noncompact imaginary Cayley",
                                    },
                                )?;
                                todo.push((KType::sr_k(rc, cx, &lr)?, coef));
                                if sx == x {
                                    // Type-2 Cayley: the shifted-lambda term.
                                    let shifted =
                                        checked_add_weights(&lr, &datum.simple_roots()[s])?;
                                    todo.push((KType::sr_k(rc, cx, &shifted)?, coef));
                                }
                                x = sx;
                                rc.simple_reflect(s, &mut im_wt, 0)?;
                                rc.simple_reflect(s, &mut lr, 1)?;
                                coef = -coef;
                                continue 'restart;
                            }
                        }
                        KgbStatus::Complex => {
                            if eval < 0 || rc.is_complex_descent(x, s)? {
                                x = rc.cross_at(x, s)?;
                                rc.simple_reflect(s, &mut im_wt, 0)?;
                                rc.simple_reflect(s, &mut lr, 1)?;
                                continue 'restart;
                            }
                        }
                        KgbStatus::Real => {
                            let eval_lr = pair(&lr, &datum.simple_coroots()[s])?;
                            if eval_lr % 2 != 0 {
                                // Project to the wall for the parity real
                                // root, then split by inverse Cayley
                                // (K_repr.cpp:381-390).
                                let shift = (eval_lr + 1) / 2;
                                let mut projected = Vec::new();
                                projected.try_reserve_exact(lr.rank()).map_err(|_| {
                                    StructureError::AllocationFailed {
                                        requested: lr.rank(),
                                    }
                                })?;
                                for (&entry, &root_entry) in
                                    lr.as_slice().iter().zip(datum.simple_roots()[s].as_slice())
                                {
                                    projected.push(
                                        entry
                                            .checked_sub(
                                                root_entry
                                                    .checked_mul(shift)
                                                    .ok_or(StructureError::ArithmeticOverflow)?,
                                            )
                                            .ok_or(StructureError::ArithmeticOverflow)?,
                                    );
                                }
                                lr = Weight::new(projected);
                                let Some((first, second)) = rc.graph().inverse_cayley(x, s)? else {
                                    return Err(StructureError::RepInvariantViolation {
                                        invariant: "parity real inverse Cayley",
                                    });
                                };
                                if let Some(second) = second {
                                    todo.push((KType::sr_k(rc, second, &lr)?, coef));
                                }
                                todo.push((KType::sr_k(rc, first, &lr)?, coef));
                                dropped = true;
                                break 'restart;
                            }
                        }
                    }
                }
                break;
            }
            if dropped {
                continue;
            }
            // The loop terminated with a dominant weight: contribute the
            // normalized, now-final K-type. The height is carried from the
            // todo entry (invariant under the Weyl-conjugate moves,
            // K_repr.cpp:174-204 comment).
            let normalized = rc.lambda_unique(rc.involution_of(x)?, &lr)?;
            result.push((KType::new(x, normalized, height), coef));
        }
        Ok(result)
    }

    /// `Rep_context::KGP_set` (K_repr.cpp:398-464): the KGP set of this
    /// (final, or semifinal) K-type — the K-types reachable from the
    /// theta-stable representative through inverse-Cayley splits and
    /// complex crosses along the real-simple Levi generators, in the
    /// upstream BFS discovery order. The caller is responsible for the
    /// semifinal precondition (the wrapper checks it before calling).
    pub fn kgp_set(&self, rc: &RepContext) -> Result<Vec<KType>, StructureError> {
        let datum = rc.inner_class().datum();
        let system = rc.inner_class().root_system();
        let theta_stable = self.made_theta_stable(rc)?;
        let simple_ids = system.simple_root_ids();
        // The Levi generators: the real-simple simple roots at the
        // theta-stable element (K_repr.cpp:404-411).
        let real_simples = rc.real_simple_roots_at(theta_stable.x())?;
        let mut levi_generators = Vec::new();
        for &real_simple in &real_simples {
            if let Some(generator) = simple_ids
                .iter()
                .position(|&candidate| candidate == real_simple)
            {
                levi_generators.push(generator);
            }
        }

        let mut present = vec![false; rc.graph().size()];
        present[theta_stable.x().index()] = true;
        let mut result = vec![theta_stable.clone()];
        let mut queue = std::collections::VecDeque::new();
        queue.push_back((theta_stable.x(), theta_stable.lambda_rho().clone()));

        while let Some((x, lam_rho)) = queue.pop_front() {
            for &s in &levi_generators {
                match rc.kgb_status(x, s)? {
                    KgbStatus::Real => {
                        // Non-parity (the final/semifinal precondition
                        // guarantees an even evaluation): project to the
                        // wall and split by inverse Cayley
                        // (K_repr.cpp:434-453).
                        let eval = pair(&lam_rho, &datum.simple_coroots()[s])?;
                        let shift = eval / 2;
                        let mut new_lr = Vec::new();
                        new_lr.try_reserve_exact(lam_rho.rank()).map_err(|_| {
                            StructureError::AllocationFailed {
                                requested: lam_rho.rank(),
                            }
                        })?;
                        for (&entry, &root_entry) in lam_rho
                            .as_slice()
                            .iter()
                            .zip(datum.simple_roots()[s].as_slice())
                        {
                            new_lr.push(
                                entry
                                    .checked_sub(
                                        root_entry
                                            .checked_mul(shift)
                                            .ok_or(StructureError::ArithmeticOverflow)?,
                                    )
                                    .ok_or(StructureError::ArithmeticOverflow)?,
                            );
                        }
                        let new_lr = Weight::new(new_lr);
                        let Some((first, second)) = rc.graph().inverse_cayley(x, s)? else {
                            return Err(StructureError::RepInvariantViolation {
                                invariant: "KGP real inverse Cayley",
                            });
                        };
                        // With the first of the pair more likely to be
                        // inserted, try it last (K_repr.cpp:438-450).
                        if let Some(second) = second {
                            if !present[second.index()] {
                                present[second.index()] = true;
                                result.push(KType::sr_k(rc, second, &new_lr)?);
                                queue.push_back((second, new_lr.clone()));
                            }
                        }
                        if !present[first.index()] {
                            present[first.index()] = true;
                            result.push(KType::sr_k(rc, first, &new_lr)?);
                            queue.push_back((first, new_lr));
                        }
                    }
                    KgbStatus::Complex => {
                        let sx = rc.graph().cross(x, s).ok_or(
                            StructureError::RepInvariantViolation {
                                invariant: "KGP complex cross",
                            },
                        )?;
                        if !present[sx.index()] {
                            present[sx.index()] = true;
                            let mut reflected = lam_rho.clone();
                            rc.simple_reflect(s, &mut reflected, 0)?;
                            result.push(KType::sr_k(rc, sx, &reflected)?);
                            queue.push_back((sx, reflected));
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(result)
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

#[cfg(test)]
mod tests {
    use crate::{
        AdjointFiberBudget, BasedRootDatum, CartanClassification, CartanClassificationBudget,
        CartanId, Coweight, InnerClass, IntegerLatticeBudget, InvolutionTable,
        InvolutionTableBudget, LatticeInvolution, RealFormSeed, RepContext,
        StrongRealClassification, WeakRealFormId, Weight,
    };

    use super::*;

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

    /// Inline split-A1 context builder; the tests keep the graph alive in
    /// their own scope while the context borrows it.
    fn with_split_a1<T>(f: impl FnOnce(&RepContext<'_>, &crate::KgbGraph) -> T) -> T {
        let datum = BasedRootDatum::from_simple_data(
            1,
            vec![vec![2]],
            vec![Weight::new(vec![2])],
            vec![Coweight::new(vec![1])],
        )
        .unwrap();
        let involution = LatticeInvolution::identity(&datum).unwrap();
        let inner_class = InnerClass::new(datum, involution, 4).unwrap();
        let classification = CartanClassification::build(&inner_class, &class_budget(4)).unwrap();
        let strong = StrongRealClassification::build(&classification, 4_096).unwrap();
        let mut table = InvolutionTable::new(
            &inner_class,
            InvolutionTableBudget::new(64, IntegerLatticeBudget::new(64, 100_000, 100_000, 128)),
        )
        .unwrap();
        table.add_cartan(&classification, CartanId(0)).unwrap();
        let seed = RealFormSeed::build(
            &inner_class,
            &classification,
            &strong,
            &table,
            WeakRealFormId(0),
            &IntegerLatticeBudget::new(64, 100_000, 100_000, 128),
            4_096,
        )
        .unwrap();
        let graph =
            crate::KgbGraph::build(&inner_class, &classification, &strong, &mut table, &seed)
                .unwrap();
        let context = RepContext::new(&inner_class, &table, &graph).unwrap();
        f(&context, &graph)
    }

    /// Inline su(2,1) (quasisplit A2) context builder; the language layer
    /// uses the simply-connected datum (roots are Cartan rows, coroots the
    /// identity basis).
    fn with_su21<T>(f: impl FnOnce(&RepContext<'_>, &crate::KgbGraph) -> T) -> T {
        let datum = BasedRootDatum::from_simple_data(
            2,
            vec![vec![2, -1], vec![-1, 2]],
            vec![Weight::new(vec![2, -1]), Weight::new(vec![-1, 2])],
            vec![Coweight::new(vec![1, 0]), Coweight::new(vec![0, 1])],
        )
        .unwrap();
        let involution = LatticeInvolution::identity(&datum).unwrap();
        let inner_class = InnerClass::new(datum, involution, 6).unwrap();
        let classification = CartanClassification::build(&inner_class, &class_budget(6)).unwrap();
        let strong = StrongRealClassification::build(&classification, 4_096).unwrap();
        let mut table = InvolutionTable::new(
            &inner_class,
            InvolutionTableBudget::new(64, IntegerLatticeBudget::new(64, 100_000, 100_000, 128)),
        )
        .unwrap();
        table.add_cartan(&classification, CartanId(0)).unwrap();
        let seed = RealFormSeed::build(
            &inner_class,
            &classification,
            &strong,
            &table,
            WeakRealFormId(0),
            &IntegerLatticeBudget::new(64, 100_000, 100_000, 128),
            4_096,
        )
        .unwrap();
        let graph =
            crate::KgbGraph::build(&inner_class, &classification, &strong, &mut table, &seed)
                .unwrap();
        let context = RepContext::new(&inner_class, &table, &graph).unwrap();
        f(&context, &graph)
    }

    #[test]
    fn split_a1_k_type_anchors_match_the_frozen_contract() {
        with_split_a1(|rc, graph| {
            // The quasisplit (split) SL(2,R) KGB has three elements.
            assert_eq!(graph.size(), 3);
            let x = KgbId(2);
            let k = KType::sr_k(rc, x, &Weight::new(vec![0])).unwrap();

            // Stored lambda-rho is [0] (displayed lambda = rho + lam_rho =
            // [1]/1 for A1); the height of (1+theta)lambda is 0.
            assert_eq!(k.lambda_rho(), &Weight::new(vec![0]));
            assert_eq!(k.height(), 0);

            assert!(k.is_standard(rc).unwrap());
            assert!(k.is_dominant(rc).unwrap());
            assert!(k.is_nonzero(rc).unwrap());
            assert!(k.is_final(rc).unwrap());
            assert!(k.is_semifinal(rc).unwrap());
            assert!(k.is_normal(rc).unwrap());

            // dominant/normal/theta_stable all fix this final K-type.
            assert_eq!(k.made_dominant(rc).unwrap(), k);
            assert_eq!(k.normalised(rc).unwrap(), k);
            assert_eq!(k.made_theta_stable(rc).unwrap(), k);

            // K_type(x,[2]) normalizes mod (1-theta)X* = 2X* for this x, so
            // the two constructions are equal and SR-equivalent.
            let k2 = KType::sr_k(rc, x, &Weight::new(vec![2])).unwrap();
            assert_eq!(k2, k);
            assert!(k.equivalent(rc, &k2).unwrap());
        });
    }

    #[test]
    fn split_a1_finals_for_a_final_parameter_is_itself() {
        with_split_a1(|rc, graph| {
            // x=2 ([r] real) with gamma=[1]/2: strictly dominant, so the
            // parameter is already final and finals_for returns it once.
            let x = crate::KgbId(2);
            let lambda_rho = Weight::new(vec![1]);
            let gamma = crate::RationalWeight::new(vec![1], 2).unwrap();
            let sr = rc.sr_gamma(x, &lambda_rho, &gamma).unwrap();
            let finals = rc.finals_for(&sr).unwrap();
            assert_eq!(finals.len(), 1);
            assert_eq!(finals[0].1, 1);
            assert_eq!(finals[0].0.x(), x);
            assert_eq!(finals[0].0.gamma(), &gamma);
            let _ = graph;
        });
    }

    #[test]
    fn split_a1_finals_for_a_non_dominant_parameter_reflects() {
        with_split_a1(|rc, graph| {
            // x=1 ([n] noncompact) with gamma=[0]/1: the generator is
            // singular (eval 0), which is left alone, so the parameter
            // is already final.
            let x = crate::KgbId(1);
            let lambda_rho = Weight::new(vec![0]);
            let gamma = crate::RationalWeight::new(vec![0], 1).unwrap();
            let sr = rc.sr_gamma(x, &lambda_rho, &gamma).unwrap();
            let finals = rc.finals_for(&sr).unwrap();
            assert_eq!(finals.len(), 1);
            assert_eq!(finals[0].0.x(), x);
            let _ = graph;
        });
    }

    #[test]
    fn split_a1_finals_for_a_negative_parameter_descends() {
        with_split_a1(|rc, graph| {
            // x=1 ([n] noncompact) with gamma=[-1]/1: the generator is
            // non-dominant (eval -1), so finals_for crosses to x=0 and
            // reflects; the result must be final and integral.
            let x = crate::KgbId(1);
            let lambda_rho = Weight::new(vec![0]);
            let gamma = crate::RationalWeight::new(vec![-1], 1).unwrap();
            let sr = rc.sr_gamma(x, &lambda_rho, &gamma).unwrap();
            let finals = rc.finals_for(&sr).unwrap();
            // The noncompact imaginary descent produces two terms: the
            // Cayley image (x=2, gamma=[-1]/1) and the crossed+reflected
            // term (x=0, gamma=[1]/1 with coefficient -1).
            assert_eq!(finals.len(), 2);
            let mut by_x = std::collections::BTreeMap::new();
            for (final_sr, coef) in &finals {
                by_x.insert(final_sr.x().index(), (coef, final_sr.gamma().clone()));
            }
            assert_eq!(by_x[&0].0, &-1);
            assert_eq!(by_x[&0].1, crate::RationalWeight::new(vec![1], 1).unwrap());
            assert_eq!(by_x[&2].0, &1);
            let _ = graph;
        });
    }

    #[test]
    fn split_a1_reducibility_points_are_empty_for_gamma_half() {
        with_split_a1(|rc, graph| {
            // x=2 ([r] real) with gamma=[1]/2: the only positive real
            // coroot pairing is 1, whose odds-table lower bound stays 0,
            // so no fraction s/d with d*s<=1 exists: empty.
            let x = crate::KgbId(2);
            let lambda_rho = Weight::new(vec![1]);
            let gamma = crate::RationalWeight::new(vec![1], 2).unwrap();
            let sr = rc.sr_gamma(x, &lambda_rho, &gamma).unwrap();
            assert!(rc.reducibility_points(&sr).unwrap().is_empty());
            // x=1 ([n]) with gamma=[0]/1 has no real roots and no
            // complex roots: empty.
            let x1 = crate::KgbId(1);
            let gamma0 = crate::RationalWeight::new(vec![0], 1).unwrap();
            let sr1 = rc.sr_gamma(x1, &Weight::new(vec![0]), &gamma0).unwrap();
            assert!(rc.reducibility_points(&sr1).unwrap().is_empty());
            let _ = graph;
        });
    }

    #[test]
    fn split_a1_standard_repr_anchors_match_the_frozen_contract() {
        with_split_a1(|rc, _graph| {
            let x = KgbId(2);
            // param(x,[0],[0]/1): gamma projects to [0]/1 on the split
            // Cartan, and the third % component is that info character.
            let parameter = rc
                .sr(
                    x,
                    &Weight::new(vec![0]),
                    &crate::RationalWeight::new(vec![0], 1).unwrap(),
                )
                .unwrap();
            assert_eq!(parameter.x(), x);
            assert_eq!(
                parameter.gamma(),
                &crate::RationalWeight::new(vec![0], 1).unwrap()
            );
            assert_eq!(parameter.height(), 0);
            assert!(parameter.is_standard(rc).unwrap());
            assert!(parameter.is_final(rc).unwrap());
            assert!(parameter.is_nonzero(rc).unwrap());
            assert!(parameter.is_normal(rc).unwrap());

            // K_type(p) restricts back to the K-type above.
            let k = rc.sr_k_of_standard(&parameter).unwrap();
            assert_eq!(k.lambda_rho(), &Weight::new(vec![0]));

            // param(K_type(x,[0])) = p: extending with nu = 0 returns the
            // same standard module.
            let k2 = KType::sr_k(rc, x, &Weight::new(vec![0])).unwrap();
            let parameter2 = rc.sr_of_ktype(&k2).unwrap();
            assert!(parameter.equivalent(rc, &parameter2).unwrap());
        });
    }

    #[test]
    fn a2_su21_context_builds_all_involutions_and_pins_nonfinal_anchors() {
        // The su(2,1) KGB graph exercises every packet involution's
        // (1-theta) image basis, including the singleton-negative-pivot
        // sweep with a column swap. The oracle's lambda_unique does NOT
        // record that pivot negation in the column ops (release build), so
        // the elected basis for x=4 is (2,-1),[1,0] — the negation of what
        // recording it would give — and the sweep must terminate (the
        // local row copy keeps the Euclidean reductions off the working
        // matrix). These anchors pin the elected representatives:
        // K_type(x4,[1,0]) keeps [1,0], K_type(x5,[1,0]) keeps [1,0],
        // K_type(x5,[0,0]) keeps [0,0], and the non-final predicates hold.
        with_su21(|rc, graph| {
            assert_eq!(graph.size(), 6);
            let k4 = KType::sr_k(rc, KgbId(4), &Weight::new(vec![1, 0])).unwrap();
            assert_eq!(k4.lambda_rho(), &Weight::new(vec![1, 0]));
            assert!(!k4.is_final(rc).unwrap());
            let k5 = KType::sr_k(rc, KgbId(5), &Weight::new(vec![1, 0])).unwrap();
            assert_eq!(k5.lambda_rho(), &Weight::new(vec![1, 0]));
            assert!(!k5.is_dominant(rc).unwrap());
            let k = KType::sr_k(rc, KgbId(5), &Weight::new(vec![0, 0])).unwrap();
            assert_eq!(k.lambda_rho(), &Weight::new(vec![0, 0]));
            assert!(!k.is_final(rc).unwrap());
        });
    }
    #[test]
    fn su21_deform_parameter_shapes_match_the_frozen_contract() {
        // The frozen deform contract (job 3506415) feeds these three
        // parameters (KGB elements 3, 4, 5 of the quasisplit su(2,1)):
        //   deform(param(KGB(rf,3),[0,0],[1,1]/1)) -> x=2, x=0
        //   deform(param(KGB(rf,4),[0,0],[1,1]/1)) -> x=1, x=0
        //   deform(param(KGB(rf,5),[0,0],[0,0]/1)) -> empty
        // Pin the shapes the crate computes for the inputs and the
        // (post-deformation) outputs so the deform evaluator can reuse
        // them.
        with_su21(|rc, _graph| {
            let rho = rc.rho();
            eprintln!("rho = {:?}/{}", rho.numerator(), rho.denominator());
            for x in 0..6 {
                let nu = crate::RationalWeight::new(vec![1, 1], 1).unwrap();
                let lam_rho = crate::Weight::new(vec![0, 0]);
                let repr = rc.sr(KgbId(x), &lam_rho, &nu).unwrap();
                eprintln!(
                    "param(x={x}): lam_rho={:?} gamma={:?}/{} height={}",
                    rc.lambda_rho(&repr).unwrap().as_slice(),
                    repr.gamma().numerator(),
                    repr.gamma().denominator(),
                    repr.height(),
                );
            }
        });
    }

    #[test]
    fn su21_finals_for_singular_gamma_zero() {
        // The frozen deform contract's third row:
        //   deform(param(KGB(rf,5),[0,0],[0,0]/1)) -> Empty
        // gamma = lam_rho + nu = [0,0] is fully singular. The deform
        // wrapper runs finals_for first; pin what that produces.
        with_su21(|rc, _graph| {
            let gamma = crate::RationalWeight::new(vec![0, 0], 1).unwrap();
            let repr = rc
                .sr_gamma(KgbId(5), &Weight::new(vec![0, 0]), &gamma)
                .unwrap();
            eprintln!(
                "sr(x=5,gamma=0): height={} gamma={:?}/{}",
                repr.height(),
                repr.gamma().numerator(),
                repr.gamma().denominator()
            );
            let finals = rc.finals_for_standard(&repr).unwrap();
            eprintln!("finals_for(x=5,gamma=0) = {} terms", finals.len());
            for (term, coef) in &finals {
                eprintln!(
                    "  term x={} lam_rho={:?} gamma={:?}/{} coef={}",
                    term.x().index(),
                    rc.lambda_rho(term).unwrap().as_slice(),
                    term.gamma().numerator(),
                    term.gamma().denominator(),
                    coef
                );
            }
        });
    }
}
