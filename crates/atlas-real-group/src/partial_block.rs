//! Bruhat intervals and partial common blocks on the integral subsystem.
//!
//! This module ports the machinery behind the Atlas `print_partial_block`
//! wrapper (interpreter/atlas-types.w:6700-6711):
//!
//! - [`StandardReprMod`]: the upstream `repr::StandardReprMod` — a KGB
//!   element with a `(1-theta)X^*`-reduced `gamma_lambda` (repr.cpp:52-67),
//!   built on top of [`RepContext::mod_reduce`] and [`RepContext::build_srm`].
//! - [`IntegralSubsystem`]: the simple-root data of the upstream
//!   `subsystem::SubSystem` (structure/subsystem.cpp:35-101) constructed from
//!   `integrality_simples` (structure/rootdata.cpp:1494-1500). Only the
//!   generator-indexed accessors that `common_context` uses
//!   (`parent_nr_simple`, `simple`, `to_simple`, `reflection`) are ported;
//!   the full subsystem root closure is not needed by the Bruhat generator.
//! - [`CommonContext`]: `repr::common_context` (repr.cpp:2666-2677) with its
//!   srm-level operations `status` (:2679-2692), `cross` (:2694-2708),
//!   `is_parity` (:2711-2722), `down_Cayley` (:2724-2742), and `up_Cayley`
//!   (:2744-2773).
//! - [`bruhat_below`]: `Rep_table::Bruhat_below` (repr.cpp:1565-1573) via
//!   `Bruhat_generator::block_below` (repr.cpp:1476-1563).
//! - [`PartialBlock`]: the partial `blocks::common_block` constructor
//!   (gkmod/blocks.cpp:1086-1248) with its final `(length, x, y)` sort
//!   (:1488-1517), plus `singular`/`survives` (blocks.cpp:701-708 and
//!   :323-330) for the print wrapper.
//!
//! Upstream `assert`s that guard caller contracts which the oracle itself
//! only checks under NDEBUG are omitted here with a comment at each site
//! (the commit f668589 precedent); genuine internal inconsistencies are
//! reported as [`StructureError`] instead of panicking.

use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};

use crate::{
    BlockDescent, Coweight, KgbId, KgbStatus, RationalWeight, RepContext, RootId, RootSystem,
    StandardRepr, StructureError, Weight,
};

/// Upstream `repr::StandardReprMod`: a standard parameter modulo `X^*` —
/// the KGB element `x` together with `gamma_lambda` made `real_unique` and
/// normalised by `StandardReprMod::build` (repr.cpp:61-67), so value
/// equality is the upstream hash-table equality.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct StandardReprMod {
    x: KgbId,
    gamma_lambda: RationalWeight,
}

impl StandardReprMod {
    /// `StandardReprMod::build` (repr.cpp:61-67): reduce `gamma_lambda`
    /// modulo the `(1-theta)X^*` image at `x` and normalise.
    pub fn build(
        rc: &RepContext<'_>,
        x: KgbId,
        gamma_lambda: &RationalWeight,
    ) -> Result<Self, StructureError> {
        Ok(Self {
            x,
            gamma_lambda: rc.build_srm(x, gamma_lambda)?,
        })
    }

    /// `StandardReprMod::mod_reduce` (repr.cpp:52-58): the parameter `z`
    /// read modulo `X^*` — the wrapper's seed computation
    /// (atlas-types.w:6704).
    pub fn mod_reduce(rc: &RepContext<'_>, z: &StandardRepr) -> Result<Self, StructureError> {
        let (x, gamma_lambda) = rc.mod_reduce(z)?;
        Ok(Self { x, gamma_lambda })
    }

    pub fn x(&self) -> KgbId {
        self.x
    }

    pub fn gamma_lambda(&self) -> &RationalWeight {
        &self.gamma_lambda
    }

    /// In-place replacement of the KGB element, as upstream's
    /// `Rep_context::transform` mutates `srm.x_part` (repr.cpp:717).
    /// Crate-private: used by the block-modifier Weyl transport only.
    pub(crate) fn set_x(&mut self, x: KgbId) {
        self.x = x;
    }

    /// In-place replacement of `gamlam` (repr.cpp:718,354,819).
    /// Crate-private: used by the block-modifier shift/transform only.
    pub(crate) fn set_gamma_lambda(&mut self, gamma_lambda: RationalWeight) {
        self.gamma_lambda = gamma_lambda;
    }

    /// `Rep_context::sr(const StandardReprMod&, const RatWeight&)`
    /// (repr.cpp:807-813): restore the full standard parameter at
    /// infinitesimal character `gamma`.
    pub fn to_standard(
        &self,
        rc: &RepContext<'_>,
        gamma: &RationalWeight,
    ) -> Result<StandardRepr, StructureError> {
        let gamma_lambda_rho = self.gamma_lambda.add(rc.rho())?;
        let coordinates = gamma.sub(&gamma_lambda_rho)?.integral_coordinates()?;
        let mut lambda_rho = Vec::new();
        lambda_rho
            .try_reserve_exact(coordinates.len())
            .map_err(|_| StructureError::AllocationFailed {
                requested: coordinates.len(),
            })?;
        for coordinate in coordinates {
            lambda_rho
                .push(i32::try_from(coordinate).map_err(|_| StructureError::ArithmeticOverflow)?);
        }
        rc.sr_gamma(self.x, &Weight::new(lambda_rho), gamma)
    }
}

/// The simple-root data of the upstream `subsystem::SubSystem`
/// (structure/subsystem.h:45-136) for the integral root system of a
/// rational weight, restricted to what `common_context` uses: per
/// subsystem simple root, the parent positive root (`parent_nr_simple`),
/// the parent simple root it is conjugate to (`sub_root[s].simple`), the
/// conjugating word (`to_simple`), and the palindromic reflection word
/// (`reflection`).
///
/// Word convention: as in upstream, `cross(word, x)` (kgb.cpp:106-111)
/// applies the LAST letter first; `to_simple(s)` conjugates the subsystem
/// simple root to the parent simple root `simple(s)` when applied in that
/// order, and `reflection(s) = to_simple ++ [simple] ++ reverse(to_simple)`
/// is the parent word of the reflection in the subsystem simple root.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegralSubsystem {
    /// `pos_map[s]`: the parent positive root of subsystem simple `s`.
    parent_root: Vec<RootId>,
    /// `sub_root[s].simple`: parent simple generator conjugate to `s`.
    conjugated_simple: Vec<usize>,
    /// `sub_root[s].to_simple`: parent word conjugating the root to simple.
    to_simple: Vec<Vec<usize>>,
    /// `sub_root[s].reflection`: parent word of the reflection in root `s`.
    reflection: Vec<Vec<usize>>,
}

impl IntegralSubsystem {
    /// The ambient root system viewed as its own integral subsystem.
    ///
    /// This avoids scanning every positive root for the common full-integral
    /// case while preserving the same generator order and transport words as
    /// [`Self::integral`] at an everywhere-integral weight.
    pub(crate) fn full(system: &RootSystem) -> Result<Self, StructureError> {
        Self::from_simple_roots(system, system.simple_root_ids())
    }

    /// `SubSystem::integral` (subsystem.cpp:97-110) via
    /// `integrality_simples` (rootdata.cpp:1494-1500): the subsystem whose
    /// simple roots are the simple basis of the positive roots whose
    /// coroot pairs integrally with `gamma`.
    pub fn integral(system: &RootSystem, gamma: &RationalWeight) -> Result<Self, StructureError> {
        let denominator = gamma.denominator();
        let mut integral_roots = BTreeSet::new();
        for (id, _, coroot) in system.entries() {
            if system.is_positive(id) != Some(true) {
                continue;
            }
            if pairing_numerator(gamma, coroot)? % denominator == 0 {
                integral_roots.insert(id);
            }
        }
        let mut simples = simple_basis(system, &integral_roots)?;
        // The subsystem generators are numbered by the parent's positive
        // root order (subsystem.cpp:43-48 pushes |sub_sys| in list order,
        // and |simpleBasis| iterates the RootNbrSet): upstream positive
        // roots are numbered by height, ties broken by reverse
        // lexicographic simple coordinates (rootdata.cpp:119-129,172-181
        // with the level-by-level generation). The crate's RootId order is
        // ambient-coordinate lexicographic, so re-sort explicitly.
        simples.sort_by(|&a, &b| upstream_positive_root_order(system, a, b));
        Self::from_simple_roots(system, &simples)
    }

    /// The `SubSystem` constructor's per-simple-root `root_info`
    /// computation (subsystem.cpp:66-101), restricted to the subsystem
    /// simple roots: reduce each root by parent simple descents until it
    /// is simple, recording the descent word.
    fn from_simple_roots(system: &RootSystem, simples: &[RootId]) -> Result<Self, StructureError> {
        let datum = system.datum();
        let mut parent_root = Vec::new();
        let mut conjugated_simple = Vec::new();
        let mut to_simple = Vec::new();
        let mut reflection = Vec::new();
        for &alpha in simples {
            // `while (alpha!=rd.simpleRootNbr(s=rd.find_descent(alpha)))`
            // (subsystem.cpp:79-83): the descent scan in generator order.
            let mut word = Vec::new(); // descents in application order
            let mut current = system
                .root(alpha)
                .ok_or(StructureError::IndexOutOfRange {
                    index: alpha.index(),
                    upper_bound: system.roots().len(),
                })?
                .clone();
            let simple = loop {
                let mut descent = None;
                for (t, coroot) in datum.simple_coroots().iter().enumerate() {
                    if crate::pair(&current, coroot)? > 0 {
                        descent = Some(t);
                        break;
                    }
                }
                let t = descent.ok_or(StructureError::RootSystemInvariantViolation {
                    invariant: "positive root has a simple descent",
                })?;
                if current == datum.simple_roots()[t] {
                    break t;
                }
                word.push(t);
                current = datum.reflect_weight(t, &current)?;
            };
            // to_simple is stored right-to-left; reflection is the
            // palindrome `word ++ [simple] ++ reverse(word)`
            // (subsystem.cpp:88-96).
            let mut reflect_word = word.clone();
            reflect_word.push(simple);
            reflect_word.extend(word.iter().rev());
            parent_root.push(alpha);
            conjugated_simple.push(simple);
            to_simple.push(word.iter().rev().copied().collect());
            reflection.push(reflect_word);
        }
        Ok(Self {
            parent_root,
            conjugated_simple,
            to_simple,
            reflection,
        })
    }

    /// The subsystem rank (number of integrally simple roots).
    pub fn rank(&self) -> usize {
        self.parent_root.len()
    }

    /// `parent_nr_simple(s)`: the parent positive root of subsystem
    /// simple `s` (subsystem.h:93).
    pub fn parent_root(&self, s: usize) -> Option<RootId> {
        self.parent_root.get(s).copied()
    }

    /// The ambient palindromic word for reflection in subsystem simple `s`
    /// (subsystem.h `reflection`). Kept crate-private because this is
    /// transport data for representation algorithms, not a language value.
    pub(crate) fn reflection_word(&self, s: usize) -> Option<&[usize]> {
        self.reflection.get(s).map(Vec::as_slice)
    }

    fn checked(&self, s: usize) -> Result<(), StructureError> {
        if s >= self.rank() {
            return Err(StructureError::IndexOutOfRange {
                index: s,
                upper_bound: self.rank(),
            });
        }
        Ok(())
    }
}

/// `<gamma, coroot>` at numerator level: `gamma.numerator().dot(coroot)`,
/// the un-divided pairing (upstream `RatWeight` numerator arithmetic).
fn pairing_numerator(gamma: &RationalWeight, coroot: &Coweight) -> Result<i64, StructureError> {
    let mut total = 0_i64;
    for (&numerator, &entry) in gamma.numerator().iter().zip(coroot.as_slice()) {
        total = total
            .checked_add(
                numerator
                    .checked_mul(i64::from(entry))
                    .ok_or(StructureError::ArithmeticOverflow)?,
            )
            .ok_or(StructureError::ArithmeticOverflow)?;
    }
    Ok(total)
}

/// `<gamma, coroot>` as an exact integer: valid for integrally simple
/// roots, where the pairing is guaranteed integral.
fn eval_on_coroot(gamma: &RationalWeight, coroot: &Coweight) -> Result<i64, StructureError> {
    let total = pairing_numerator(gamma, coroot)?;
    if total % gamma.denominator() != 0 {
        return Err(StructureError::RepInvariantViolation {
            invariant: "integral coroot evaluation",
        });
    }
    Ok(total / gamma.denominator())
}

/// `<weight, coroot>` as an integer.
fn pair_weight(weight: &Weight, coroot: &Coweight) -> Result<i64, StructureError> {
    let mut total = 0_i64;
    for (&coordinate, &entry) in weight.as_slice().iter().zip(coroot.as_slice()) {
        total = total
            .checked_add(
                i64::from(coordinate)
                    .checked_mul(i64::from(entry))
                    .ok_or(StructureError::ArithmeticOverflow)?,
            )
            .ok_or(StructureError::ArithmeticOverflow)?;
    }
    Ok(total)
}

/// The upstream positive-root numbering order (RootNbr): increasing
/// height, ties broken by reverse lexicographic comparison of the simple
/// coordinates (`root_compare`, rootdata.cpp:119-129). Simple roots come
/// first in generator order.
pub(crate) fn upstream_positive_root_order(
    system: &RootSystem,
    a: RootId,
    b: RootId,
) -> std::cmp::Ordering {
    let coordinates_a = system.simple_coordinates(a).unwrap_or(&[]);
    let coordinates_b = system.simple_coordinates(b).unwrap_or(&[]);
    let height_a: i32 = coordinates_a.iter().sum();
    let height_b: i32 = coordinates_b.iter().sum();
    height_a.cmp(&height_b).then_with(|| {
        for i in (0..coordinates_a.len().max(coordinates_b.len())).rev() {
            let entry_a = coordinates_a.get(i).copied().unwrap_or(0);
            let entry_b = coordinates_b.get(i).copied().unwrap_or(0);
            match entry_a.cmp(&entry_b) {
                std::cmp::Ordering::Equal => continue,
                order => return order,
            }
        }
        std::cmp::Ordering::Equal
    })
}

/// `RootSystem::simpleBasis` (rootdata.cpp:621-652): the simple roots of
/// the subsystem generated by a set of positive roots. Upstream compares
/// root numbers where the comment says "positive dot product"; positive
/// root numbers increase with height (rootdata.cpp:172-181 generates
/// level by level), and `s_alpha(beta)` has strictly smaller height than
/// `beta` exactly when `<beta, alpha^vee> > 0`, so the pairing sign is the
/// faithful test.
fn simple_basis(
    system: &RootSystem,
    subset: &BTreeSet<RootId>,
) -> Result<Vec<RootId>, StructureError> {
    let mut candidates = subset.clone();
    'outer: for &alpha in subset {
        if !candidates.contains(&alpha) {
            continue; // pruned as a |beta| of an earlier iteration
        }
        let alpha_root = system.root(alpha).ok_or(StructureError::IndexOutOfRange {
            index: alpha.index(),
            upper_bound: system.roots().len(),
        })?;
        let alpha_coroot = system
            .coroot(alpha)
            .ok_or(StructureError::IndexOutOfRange {
                index: alpha.index(),
                upper_bound: system.roots().len(),
            })?;
        for &beta in subset {
            if alpha == beta {
                continue;
            }
            let beta_root = system.root(beta).ok_or(StructureError::IndexOutOfRange {
                index: beta.index(),
                upper_bound: system.roots().len(),
            })?;
            let pairing = crate::pair(beta_root, alpha_coroot)?;
            if pairing > 0 {
                // gamma = s_alpha(beta), a root with smaller height
                let mut gamma_coordinates = Vec::new();
                for (&b, &a) in beta_root.as_slice().iter().zip(alpha_root.as_slice()) {
                    gamma_coordinates.push(
                        b.checked_sub(
                            pairing
                                .checked_mul(a)
                                .ok_or(StructureError::ArithmeticOverflow)?,
                        )
                        .ok_or(StructureError::ArithmeticOverflow)?,
                    );
                }
                match system.id_of(&Weight::new(gamma_coordinates)) {
                    Some(gamma) if system.is_positive(gamma) == Some(true) => {
                        candidates.remove(&beta); // beta is not simple
                    }
                    _ => {
                        candidates.remove(&alpha); // alpha is not simple
                        continue 'outer;
                    }
                }
            }
        }
    }
    Ok(candidates.into_iter().collect())
}

/// `pos_to_neg` (rootdata.cpp:1413-1439): the positive roots made
/// negative by left multiplication with `word`, letters read left to
/// right: reflect the current set by the letter's simple reflection (the
/// simple root itself maps out) and toggle the simple root's membership.
fn pos_to_neg(system: &RootSystem, word: &[usize]) -> Result<Vec<RootId>, StructureError> {
    let datum = system.datum();
    let mut current: BTreeSet<RootId> = system
        .entries()
        .filter(|(id, _, _)| system.is_positive(*id) == Some(true))
        .map(|(id, _, _)| id)
        .collect();
    for &s in word {
        let simple =
            system
                .simple_root_ids()
                .get(s)
                .copied()
                .ok_or(StructureError::IndexOutOfRange {
                    index: s,
                    upper_bound: datum.semisimple_rank(),
                })?;
        let mut next = BTreeSet::new();
        for &beta in &current {
            if beta == simple {
                continue; // maps to its negative, out of the positive set
            }
            let image = datum.reflect_weight(
                s,
                system.root(beta).ok_or(StructureError::IndexOutOfRange {
                    index: beta.index(),
                    upper_bound: system.roots().len(),
                })?,
            )?;
            next.insert(system.id_of(&image).ok_or(
                StructureError::RootSystemInvariantViolation {
                    invariant: "reflected root is a root",
                },
            )?);
        }
        // the upstream `tmp.flip(s)`: add the simple root unless it was
        // just removed by the reflection
        if !current.contains(&simple) {
            next.insert(simple);
        }
        current = next;
    }
    Ok(current.into_iter().collect())
}

/// `root_sum` (rootdata.cpp:1647): the sum of the given root vectors.
fn root_sum(rc: &RepContext<'_>, roots: &[RootId]) -> Result<RationalWeight, StructureError> {
    RationalWeight::from_weight(&rc.two_rho_of(roots)?)
}

/// `repr::common_context` (repr.cpp:2664-2677): a [`RepContext`] together
/// with the integral root subsystem of the infinitesimal character,
/// against which the srm-level cross/Cayley operations act.
pub struct CommonContext<'r, 'a> {
    rc: &'r RepContext<'a>,
    sub: IntegralSubsystem,
}

impl<'r, 'a> CommonContext<'r, 'a> {
    pub(crate) fn full_if_integral(
        rc: &'r RepContext<'a>,
        gamma: &RationalWeight,
    ) -> Result<Option<Self>, StructureError> {
        let denominator = gamma.denominator();
        let full = rc
            .datum()
            .simple_coroots()
            .iter()
            .try_fold(true, |full, coroot| {
                Ok(full && pairing_numerator(gamma, coroot)? % denominator == 0)
            })?;
        if full {
            Ok(Some(Self {
                rc,
                sub: IntegralSubsystem::full(rc.root_system())?,
            }))
        } else {
            Ok(None)
        }
    }

    /// The gamma-based constructor (repr.cpp:2666-2670). The
    /// `print_partial_block` wrapper passes the seed srm's `gamma_lambda`
    /// (atlas-types.w:6705); any weight differing from the infinitesimal
    /// character by an integral weight gives the same subsystem.
    pub fn integral(
        rc: &'r RepContext<'a>,
        gamma: &RationalWeight,
    ) -> Result<Self, StructureError> {
        Ok(Self {
            rc,
            sub: IntegralSubsystem::integral(rc.root_system(), gamma)?,
        })
    }

    pub fn rep_context(&self) -> &RepContext<'a> {
        self.rc
    }

    pub fn subsystem(&self) -> &IntegralSubsystem {
        &self.sub
    }

    /// The subsystem rank (repr.cpp:1481's `ctxt.subsys().rank()`).
    pub fn rank(&self) -> usize {
        self.sub.rank()
    }

    /// `kgb().cross(ww, x)` (kgb.cpp:106-111): the word applied with its
    /// LAST letter first.
    fn cross_word(&self, word: &[usize], mut x: KgbId) -> Result<KgbId, StructureError> {
        for &s in word.iter().rev() {
            x = self.rc.cross_at(x, s)?;
        }
        Ok(x)
    }

    /// `kgb().cross(x, ww)` (kgb.cpp:113-118): the word applied with its
    /// FIRST letter first.
    fn cross_word_forward(&self, word: &[usize], mut x: KgbId) -> Result<KgbId, StructureError> {
        for &s in word {
            x = self.rc.cross_at(x, s)?;
        }
        Ok(x)
    }

    fn parent_coroot(&self, s: usize) -> Result<&Coweight, StructureError> {
        let root = self.sub.parent_root[s];
        self.rc
            .root_system()
            .coroot(root)
            .ok_or(StructureError::IndexOutOfRange {
                index: root.index(),
                upper_bound: self.rc.root_system().roots().len(),
            })
    }

    fn parent_root_weight(&self, s: usize) -> Result<&Weight, StructureError> {
        let root = self.sub.parent_root[s];
        self.rc
            .root_system()
            .root(root)
            .ok_or(StructureError::IndexOutOfRange {
                index: root.index(),
                upper_bound: self.rc.root_system().roots().len(),
            })
    }

    /// `common_context::status` (repr.cpp:2679-2692): the KGB status of
    /// the subsystem generator `s` at `x`, transported to the conjugated
    /// parent simple root, together with the type-1/descent flag —
    /// `isDoubleCayleyImage` for real, `isDescent` for complex, and
    /// cross-moves for noncompact imaginary (type 1 vs type 2).
    pub fn status(&self, s: usize, x: KgbId) -> Result<(KgbStatus, bool), StructureError> {
        self.sub.checked(s)?;
        let conj_x = self.cross_word(&self.sub.to_simple[s], x)?;
        let t = self.sub.conjugated_simple[s];
        let stat = self.rc.graph().status(conj_x, t).ok_or({
            StructureError::IndexOutOfRange {
                index: conj_x.index(),
                upper_bound: self.rc.graph().size(),
            }
        })?;
        let flag =
            match stat {
                KgbStatus::Real => matches!(
                    self.rc.graph().inverse_cayley(conj_x, t)?,
                    Some((_, Some(_)))
                ),
                KgbStatus::Complex => self.rc.graph().is_descent(conj_x, t).ok_or(
                    StructureError::IndexOutOfRange {
                        index: conj_x.index(),
                        upper_bound: self.rc.graph().size(),
                    },
                )?,
                KgbStatus::ImaginaryCompact | KgbStatus::ImaginaryNoncompact => {
                    conj_x != self.rc.cross_at(conj_x, t)?
                }
            };
        Ok((stat, flag))
    }

    /// `common_context::cross` (repr.cpp:2694-2708): cross action of the
    /// subsystem generator `s` on an srm — cross `x` by the reflection
    /// word and shift `gamma_lambda` by the `pos_to_neg` real-root
    /// correction, then reflect by the (integrally simple) parent root.
    pub fn cross(&self, s: usize, z: &StandardReprMod) -> Result<StandardReprMod, StructureError> {
        self.sub.checked(s)?;
        let reflection = &self.sub.reflection[s];
        let new_x = self.cross_word(reflection, z.x)?;
        let mut gamma_lambda = z.gamma_lambda.clone();
        let real_roots = self.rc.positive_real_roots_at(z.x)?;
        let correction: Vec<RootId> = pos_to_neg(self.rc.root_system(), reflection)?
            .into_iter()
            .filter(|id| real_roots.contains(id))
            .collect();
        gamma_lambda = gamma_lambda.sub(&root_sum(self.rc, &correction)?)?;
        // `subsys().simple_reflect(s, gamma_lambda.numerator())`
        // (repr.cpp:2706): numerator-level reflection; exact because the
        // pairing of an integrally simple coroot with `gamma_lambda` is
        // denominator-divisible, so `numerator - alpha * <coroot,
        // numerator>` reflects the rational value.
        let alpha = self.parent_root_weight(s)?;
        let coroot = self.parent_coroot(s)?;
        let eval = pairing_numerator(&gamma_lambda, coroot)?;
        let mut numerator = gamma_lambda.numerator().to_vec();
        for (entry, &a) in numerator.iter_mut().zip(alpha.as_slice()) {
            *entry = entry
                .checked_sub(
                    eval.checked_mul(i64::from(a))
                        .ok_or(StructureError::ArithmeticOverflow)?,
                )
                .ok_or(StructureError::ArithmeticOverflow)?;
        }
        let gamma_lambda = RationalWeight::new(numerator, gamma_lambda.denominator())?;
        StandardReprMod::build(self.rc, new_x, &gamma_lambda)
    }

    /// `common_context::is_parity` (repr.cpp:2711-2722): whether the
    /// subsystem generator `s`, supposed real at `z`, is a parity root.
    /// Upstream asserts that `parent_nr_simple(s)` is real at `z`; that
    /// caller contract is not re-checked here (NDEBUG-only in the oracle).
    pub fn is_parity(&self, s: usize, z: &StandardReprMod) -> Result<bool, StructureError> {
        self.sub.checked(s)?;
        let coroot = self.parent_coroot(s)?;
        let eval = eval_on_coroot(&z.gamma_lambda, coroot)?;
        let real_roots = self.rc.positive_real_roots_at(z.x)?;
        let two_rho_real = self.rc.two_rho_of(&real_roots)?;
        // `<coroot, 2*rho(real)>/2` is exact: the root of `s` lies in the
        // real subsystem, and `<2*rho_R, beta^vee> = 2` for every root
        // beta of a root subsystem R.
        let corr_twice = pair_weight(&two_rho_real, coroot)?;
        if corr_twice % 2 != 0 {
            return Err(StructureError::RepInvariantViolation {
                invariant: "rho_r parity correction",
            });
        }
        Ok((eval + corr_twice / 2) % 2 != 0)
    }

    /// `common_context::down_Cayley` (repr.cpp:2724-2742): the Cayley
    /// descent of `z` through the real parity generator `s`. Upstream
    /// asserts `is_parity(s, z)`; that caller contract is not re-checked
    /// here (NDEBUG-only in the oracle).
    pub fn down_cayley(
        &self,
        s: usize,
        z: &StandardReprMod,
    ) -> Result<StandardReprMod, StructureError> {
        self.sub.checked(s)?;
        let conj = &self.sub.to_simple[s];
        let t = self.sub.conjugated_simple[s];
        let conj_x = self.cross_word(conj, z.x)?;
        let (first, _) = self.rc.graph().inverse_cayley(conj_x, t)?.ok_or(
            StructureError::RepInvariantViolation {
                invariant: "down Cayley image",
            },
        )?;
        let new_x = self.cross_word_forward(conj, first)?;
        // posroots that change real status between z and its image and
        // map to negative under the conjugating word (repr.cpp:2736-2739).
        let real_down = self.rc.positive_real_roots_at(z.x)?;
        let real_up = self.rc.positive_real_roots_at(new_x)?;
        let correction: Vec<RootId> = pos_to_neg(self.rc.root_system(), conj)?
            .into_iter()
            .filter(|id| real_down.contains(id) != real_up.contains(id))
            .collect();
        let gamma_lambda = z.gamma_lambda.add(&root_sum(self.rc, &correction)?)?;
        StandardReprMod::build(self.rc, new_x, &gamma_lambda)
    }

    /// `common_context::up_Cayley` (repr.cpp:2744-2773): the Cayley ascent
    /// of `z` through the noncompact imaginary generator `s`, with the
    /// parity correction adding `alpha_s/2` when the raised `gamma_lambda`
    /// fails the parity condition. Upstream asserts the noncompact
    /// imaginary status; that caller contract is not re-checked here.
    pub fn up_cayley(
        &self,
        s: usize,
        z: &StandardReprMod,
    ) -> Result<StandardReprMod, StructureError> {
        self.sub.checked(s)?;
        let conj = &self.sub.to_simple[s];
        let t = self.sub.conjugated_simple[s];
        let conj_x = self.cross_word(conj, z.x)?;
        let cayley_image =
            self.rc
                .graph()
                .cayley(conj_x, t)?
                .ok_or(StructureError::RepInvariantViolation {
                    invariant: "up Cayley image",
                })?;
        let new_x = self.cross_word_forward(conj, cayley_image)?;
        let upstairs_real = self.rc.positive_real_roots_at(new_x)?;
        let real_down = self.rc.positive_real_roots_at(z.x)?;
        let correction: Vec<RootId> = pos_to_neg(self.rc.root_system(), conj)?
            .into_iter()
            .filter(|id| upstairs_real.contains(id) != real_down.contains(id))
            .collect();
        let mut gamma_lambda = z.gamma_lambda.add(&root_sum(self.rc, &correction)?)?;
        // parity correction against the UPSTAIRS real roots
        // (repr.cpp:2764-2770)
        let coroot = self.parent_coroot(s)?;
        let two_rho_real = self.rc.two_rho_of(&upstairs_real)?;
        let corr_twice = pair_weight(&two_rho_real, coroot)?;
        if corr_twice % 2 != 0 {
            return Err(StructureError::RepInvariantViolation {
                invariant: "up Cayley rho_r correction",
            });
        }
        let eval = eval_on_coroot(&gamma_lambda, coroot)?;
        if (eval + corr_twice / 2) % 2 == 0 {
            // add `RatWeight(simple_root(s), 2)` (repr.cpp:2770)
            let alpha = self.parent_root_weight(s)?;
            let rank = alpha.as_slice().len();
            let mut numerator = Vec::new();
            numerator
                .try_reserve_exact(rank)
                .map_err(|_| StructureError::AllocationFailed { requested: rank })?;
            for (&n, &a) in gamma_lambda.numerator().iter().zip(alpha.as_slice()) {
                numerator.push(
                    n.checked_mul(2)
                        .and_then(|n2| {
                            n2.checked_add(gamma_lambda.denominator().checked_mul(i64::from(a))?)
                        })
                        .ok_or(StructureError::ArithmeticOverflow)?,
                );
            }
            gamma_lambda = RationalWeight::new(
                numerator,
                gamma_lambda
                    .denominator()
                    .checked_mul(2)
                    .ok_or(StructureError::ArithmeticOverflow)?,
            )?;
        }
        StandardReprMod::build(self.rc, new_x, &gamma_lambda)
    }

    /// `common_block::singular` (blocks.cpp:701-708): per subsystem
    /// generator, whether its coroot vanishes on `gamma`'s numerator.
    pub fn singular_flags(&self, gamma: &RationalWeight) -> Result<Vec<bool>, StructureError> {
        let mut flags = Vec::new();
        flags
            .try_reserve_exact(self.rank())
            .map_err(|_| StructureError::AllocationFailed {
                requested: self.rank(),
            })?;
        for s in 0..self.rank() {
            flags.push(pairing_numerator(gamma, self.parent_coroot(s)?)? == 0);
        }
        Ok(flags)
    }
}

/// `Rep_table::Bruhat_generator` (repr.cpp:1459-1473): the recursive
/// interval state — the srm pool in creation order, its index, and the
/// predecessor lists per generated element.
struct BruhatGenerator<'c, 'r, 'a> {
    ctxt: &'c CommonContext<'r, 'a>,
    pool: Vec<StandardReprMod>,
    index: HashMap<StandardReprMod, usize>,
    predecessors: Vec<Vec<usize>>,
}

impl<'c, 'r, 'a> BruhatGenerator<'c, 'r, 'a> {
    /// `Bruhat_generator::block_below` (repr.cpp:1476-1563): insert `srm`
    /// and the whole Bruhat interval below it into the pool.
    fn block_below(&mut self, srm: &StandardReprMod) -> Result<(), StructureError> {
        if self.index.contains_key(srm) {
            return Ok(()); // seen earlier: nothing new
        }
        let rank = self.ctxt.rank();
        let mut pred: Vec<usize> = Vec::new();
        // find a complex or real type 1 descent, if one exists
        let mut descent = None;
        for s in 0..rank {
            let (stat, flag) = self.ctxt.status(s, srm.x())?;
            if !flag {
                continue; // imaginary, complex ascent or real type 2
            }
            if stat == KgbStatus::Complex {
                let sz = self.ctxt.cross(s, srm)?;
                self.block_below(&sz)?;
                pred.push(self.index[&sz]);
                descent = Some(s);
                break; // s-ascents of predecessors of |sz| are added below
            } else if stat == KgbStatus::Real && self.ctxt.is_parity(s, srm)? {
                // z has a type 1 real descent at s
                let sz0 = self.ctxt.down_cayley(s, srm)?;
                let sz1 = self.ctxt.cross(s, &sz0)?;
                self.block_below(&sz0)?;
                self.block_below(&sz1)?;
                pred.push(self.index[&sz0]);
                pred.push(self.index[&sz1]);
                descent = Some(s);
                break;
            }
        }
        match descent {
            None => {
                // the only descents are real type 2, if any
                // (repr.cpp:1512-1522, reversed loop)
                for s in (0..rank).rev() {
                    let (stat, _) = self.ctxt.status(s, srm.x())?;
                    if stat == KgbStatus::Real && self.ctxt.is_parity(s, srm)? {
                        let sz = self.ctxt.down_cayley(s, srm)?;
                        self.block_below(&sz)?;
                        pred.push(self.index[&sz]);
                    }
                }
            }
            Some(s) => {
                // add s-ascents for the elements covered by the descent
                // image (repr.cpp:1523-1558)
                let pred_sz = self.predecessors[pred[0]].clone();
                for p in pred_sz {
                    let zp = self.pool[p].clone();
                    let (stat, flag) = self.ctxt.status(s, zp.x())?;
                    match stat {
                        KgbStatus::Real | KgbStatus::ImaginaryCompact => {}
                        KgbStatus::Complex => {
                            if !flag {
                                // complex ascent
                                let szp = self.ctxt.cross(s, &zp)?;
                                self.block_below(&szp)?;
                                pred.push(self.index[&szp]);
                            }
                        }
                        KgbStatus::ImaginaryNoncompact => {
                            let szp = self.ctxt.up_cayley(s, &zp)?;
                            self.block_below(&szp)?;
                            pred.push(self.index[&szp]);
                            if !flag {
                                // nci type 2
                                let szp1 = self.ctxt.cross(s, &szp)?;
                                self.block_below(&szp1)?;
                                pred.push(self.index[&szp1]);
                            }
                        }
                    }
                }
            }
        }
        let h = self.pool.len();
        // upstream asserts h == predecessors.size() (repr.cpp:1561); true
        // by construction here since pool and predecessors grow together.
        debug_assert_eq!(h, self.predecessors.len());
        self.index.insert(srm.clone(), h);
        self.pool.push(srm.clone());
        self.predecessors.push(pred);
        Ok(())
    }
}

/// `Rep_table::Bruhat_below` (repr.cpp:1565-1573): the Bruhat interval
/// below `init` over the integral subsystem, as a list of srms in
/// generation order (descents precede). `init` must be a built/mod-reduced
/// srm, e.g. from [`StandardReprMod::mod_reduce`].
pub fn bruhat_below(
    ctxt: &CommonContext<'_, '_>,
    init: &StandardReprMod,
) -> Result<Vec<StandardReprMod>, StructureError> {
    let mut generator = BruhatGenerator {
        ctxt,
        pool: Vec::new(),
        index: HashMap::new(),
        predecessors: Vec::new(),
    };
    generator.block_below(init)?;
    Ok(generator.pool)
}

/// `RationalVector::operator<` (ratvec.cpp:46-60): component-wise
/// comparison on a common denominator (sizes first, though ranks always
/// agree here).
fn rational_weight_compare(left: &RationalWeight, right: &RationalWeight) -> std::cmp::Ordering {
    left.rank().cmp(&right.rank()).then_with(|| {
        for (&l, &r) in left.numerator().iter().zip(right.numerator()) {
            let diff = i128::from(l) * i128::from(right.denominator())
                - i128::from(r) * i128::from(left.denominator());
            match diff.cmp(&0) {
                std::cmp::Ordering::Equal => continue,
                order => return order,
            }
        }
        std::cmp::Ordering::Equal
    })
}

/// Ambient KGB cross by a Weyl word; `KGB::cross(word,x)` applies the last
/// letter first (`kgb.cpp:106-111`).
fn cross_kgb_word(
    rc: &RepContext<'_>,
    word: &[usize],
    mut x: KgbId,
) -> Result<KgbId, StructureError> {
    for &generator in word.iter().rev() {
        x = rc.cross_at(x, generator)?;
    }
    Ok(x)
}

/// The per-generator link fields of the partial block: upstream
/// `block_fields` (blocks.h) with `UndefBlock` as `None`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct BlockFields {
    cross_image: Option<usize>,
    cayley_image: (Option<usize>, Option<usize>),
}

/// The partial common block over a Bruhat interval: the upstream
/// `blocks::common_block(ctxt, elements)` constructor
/// (blocks.cpp:1086-1248). Elements are sorted by `(length, x, y)` at the
/// end of construction (blocks.cpp:1488-1517), so element numbers match
/// the oracle's printed row numbers.
#[derive(Clone, Debug)]
pub struct PartialBlock {
    rank: usize,
    /// `z_pool`, sorted along with `info`.
    elements: Vec<StandardReprMod>,
    lookup_table: HashMap<StandardReprMod, usize>,
    xs: Vec<KgbId>,
    ys: Vec<usize>,
    lengths: Vec<usize>,
    /// `descent[z * rank + s]`.
    descents: Vec<BlockDescent>,
    /// `data[s][z]`, generator-major as upstream.
    fields: Vec<Vec<BlockFields>>,
    highest_x: usize,
    highest_y: usize,
}

type FullPacket = Vec<Vec<StandardReprMod>>;

struct FullPacketWork {
    packet: FullPacket,
    reversed_length: usize,
}

/// Mutable state of `blocks.cpp:844-1064`: packets remain FIFO, `y_count`
/// is global across involutions, and `x_seen` decides whether a whole packet
/// must be materialised before its links are resolved.
struct FullBlockBuilder<'c, 'r, 'a> {
    ctxt: &'c CommonContext<'r, 'a>,
    block: PartialBlock,
    packets: VecDeque<FullPacketWork>,
    x_seen: BTreeSet<KgbId>,
    y_count: usize,
}

/// Step 2 (`blocks.cpp:758-784`): repeatedly take the first available
/// ascent and restart the generator scan at zero.
fn full_block_top_ascent(
    ctxt: &CommonContext<'_, '_>,
    seed: &StandardReprMod,
) -> Result<StandardReprMod, StructureError> {
    let mut top = seed.clone();
    'raise: loop {
        for generator in 0..ctxt.rank() {
            let (status, flag) = ctxt.status(generator, top.x())?;
            if status == KgbStatus::Complex && !flag {
                top = ctxt.cross(generator, &top)?;
                continue 'raise;
            }
            if status == KgbStatus::ImaginaryNoncompact {
                top = ctxt.up_cayley(generator, &top)?;
                continue 'raise;
            }
        }
        return Ok(top);
    }
}

/// Step 3 (`blocks.cpp:786-842`): close the top SRM under its real-root
/// reflection words and keep the row ordered by exact `gamma_lambda`.
fn full_block_initial_real_orbit(
    ctxt: &CommonContext<'_, '_>,
    top: StandardReprMod,
) -> Result<Vec<StandardReprMod>, StructureError> {
    // `CommonContext::cross` already transports each subsystem generator's
    // parent reflection word.  Using ambient root-reflection words here
    // would feed ambient generator indices back into a proper subsystem.
    let mut real_generators = Vec::new();
    for generator in 0..ctxt.rank() {
        let (status, _) = ctxt.status(generator, top.x())?;
        if status == KgbStatus::Real {
            real_generators.push(generator);
        }
    }
    let top_x = top.x();
    let mut orbit_queue = VecDeque::from([top]);
    let mut top_row = Vec::new();
    while let Some(value) = orbit_queue.pop_front() {
        match top_row.binary_search_by(|known: &StandardReprMod| {
            rational_weight_compare(known.gamma_lambda(), value.gamma_lambda())
        }) {
            Ok(position) if top_row[position] == value => continue,
            Ok(_) => {
                return Err(StructureError::RepInvariantViolation {
                    invariant: "top y orbit has equal gamma_lambda values",
                })
            }
            Err(position) => top_row.insert(position, value.clone()),
        }
        for &generator in &real_generators {
            let reflected = ctxt.cross(generator, &value)?;
            if reflected.x() != top_x {
                return Err(StructureError::RepInvariantViolation {
                    invariant: "real reflection preserves top x",
                });
            }
            orbit_queue.push_back(reflected);
        }
    }
    Ok(top_row)
}

impl<'c, 'r, 'a> FullBlockBuilder<'c, 'r, 'a> {
    fn from_top_row(
        ctxt: &'c CommonContext<'r, 'a>,
        top_row: Vec<StandardReprMod>,
    ) -> Result<Self, StructureError> {
        let top_x = top_row
            .first()
            .ok_or(StructureError::RepInvariantViolation {
                invariant: "nonempty top real-root orbit",
            })?
            .x();
        let rank = ctxt.rank();
        let mut block = PartialBlock {
            rank,
            elements: Vec::new(),
            lookup_table: HashMap::new(),
            xs: Vec::new(),
            ys: Vec::new(),
            lengths: Vec::new(),
            descents: Vec::new(),
            fields: vec![Vec::new(); rank],
            highest_x: top_x.index(),
            highest_y: 0,
        };
        for (y, value) in top_row.iter().enumerate() {
            block.push_full_element(value.clone(), y, 0)?;
        }
        let y_count = top_row.len();
        Ok(Self {
            ctxt,
            block,
            packets: VecDeque::from([FullPacketWork {
                packet: vec![top_row],
                reversed_length: 0,
            }]),
            x_seen: BTreeSet::from([top_x]),
            y_count,
        })
    }

    /// Step 4 driver (`blocks.cpp:844-1064`): process whole involution
    /// packets in discovery order; newly materialised packets are appended.
    fn expand_fifo_packets(&mut self) -> Result<(), StructureError> {
        while let Some(work) = self.packets.pop_front() {
            for generator in 0..self.block.rank {
                self.expand_generator(&work, generator)?;
            }
        }
        Ok(())
    }

    fn expand_generator(
        &mut self,
        work: &FullPacketWork,
        generator: usize,
    ) -> Result<(), StructureError> {
        let first = work.packet.first().and_then(|row| row.first()).ok_or(
            StructureError::RepInvariantViolation {
                invariant: "nonempty common-block packet",
            },
        )?;
        let crossed_first = self.ctxt.cross(generator, first)?;
        let cross_new = !self.x_seen.contains(&crossed_first.x());
        let (status, flag) = self.ctxt.status(generator, first.x())?;
        let is_real = status == KgbStatus::Real;
        let is_type_one = is_real && flag;
        let (crosses, cayleys, sample_x) =
            self.generator_models(&work.packet, generator, cross_new, is_real)?;

        if cross_new {
            self.materialize_cross_packet(work, generator, crosses)?;
        }
        self.resolve_crosses_and_descents(&work.packet, generator)?;
        if !cayleys.is_empty() {
            self.complete_cayley_packet(work, generator, cayleys, sample_x, is_type_one)?;
        }
        Ok(())
    }

    fn generator_models(
        &self,
        packet: &FullPacket,
        generator: usize,
        cross_new: bool,
        is_real: bool,
    ) -> Result<(Vec<StandardReprMod>, Vec<StandardReprMod>, Option<KgbId>), StructureError> {
        let mut crosses = Vec::new();
        let mut cayleys = Vec::new();
        let mut sample_x = None;
        if cross_new {
            for value in &packet[0] {
                crosses.push(self.ctxt.cross(generator, value)?);
            }
        } else if is_real {
            for value in &packet[0] {
                if self.ctxt.is_parity(generator, value)? {
                    let down = self.ctxt.down_cayley(generator, value)?;
                    sample_x = Some(down.x());
                    cayleys.push(down);
                }
            }
        }
        Ok((crosses, cayleys, sample_x))
    }

    fn materialize_cross_packet(
        &mut self,
        work: &FullPacketWork,
        generator: usize,
        mut crosses: Vec<StandardReprMod>,
    ) -> Result<(), StructureError> {
        crosses.sort_by(|left, right| {
            rational_weight_compare(left.gamma_lambda(), right.gamma_lambda())
        });
        let mut packet = Vec::new();
        for row in &work.packet {
            let target_x = self.ctxt.cross(generator, &row[0])?.x();
            self.x_seen.insert(target_x);
            packet.push(self.materialize_row(target_x, &crosses, work.reversed_length + 1)?);
        }
        self.y_count = self
            .y_count
            .checked_add(crosses.len())
            .ok_or(StructureError::ArithmeticOverflow)?;
        self.packets.push_back(FullPacketWork {
            packet,
            reversed_length: work.reversed_length + 1,
        });
        Ok(())
    }

    fn materialize_row(
        &mut self,
        x: KgbId,
        models: &[StandardReprMod],
        reversed_length: usize,
    ) -> Result<Vec<StandardReprMod>, StructureError> {
        let mut row = Vec::new();
        for (column, model) in models.iter().enumerate() {
            let value = StandardReprMod::build(self.ctxt.rep_context(), x, model.gamma_lambda())?;
            self.block
                .push_full_element(value.clone(), self.y_count + column, reversed_length)?;
            row.push(value);
        }
        Ok(row)
    }

    fn resolve_crosses_and_descents(
        &mut self,
        packet: &FullPacket,
        generator: usize,
    ) -> Result<(), StructureError> {
        for row in packet {
            for value in row {
                let source = self.registered(value, "packet SRM registered")?;
                let crossed = self.ctxt.cross(generator, value)?;
                let target = self.registered(&crossed, "full-block cross image registered")?;
                self.block.fields[generator][source].cross_image = Some(target);
                let (status, flag) = self.ctxt.status(generator, value.x())?;
                let descent = self.descent_value(generator, value, status, flag)?;
                self.block.descents[source * self.block.rank + generator] = descent;
            }
        }
        Ok(())
    }

    fn descent_value(
        &self,
        generator: usize,
        value: &StandardReprMod,
        status: KgbStatus,
        flag: bool,
    ) -> Result<BlockDescent, StructureError> {
        Ok(match status {
            KgbStatus::Complex if flag => BlockDescent::ComplexDescent,
            KgbStatus::Complex => BlockDescent::ComplexAscent,
            KgbStatus::ImaginaryCompact => BlockDescent::ImaginaryCompact,
            KgbStatus::ImaginaryNoncompact if flag => BlockDescent::ImaginaryTypeI,
            KgbStatus::ImaginaryNoncompact => BlockDescent::ImaginaryTypeII,
            KgbStatus::Real if !self.ctxt.is_parity(generator, value)? => {
                BlockDescent::RealNonparity
            }
            KgbStatus::Real if flag => BlockDescent::RealTypeI,
            KgbStatus::Real => BlockDescent::RealTypeII,
        })
    }

    fn complete_cayley_packet(
        &mut self,
        work: &FullPacketWork,
        generator: usize,
        mut cayleys: Vec<StandardReprMod>,
        sample_x: Option<KgbId>,
        is_type_one: bool,
    ) -> Result<(), StructureError> {
        cayleys.sort_by(|left, right| {
            rational_weight_compare(left.gamma_lambda(), right.gamma_lambda())
        });
        cayleys.dedup();
        let sample_x = sample_x.ok_or(StructureError::RepInvariantViolation {
            invariant: "Cayley packet sample x",
        })?;
        if !self.x_seen.contains(&sample_x) {
            let packet =
                self.materialize_cayley_fiber(sample_x, &cayleys, work.reversed_length + 1)?;
            self.y_count = self
                .y_count
                .checked_add(cayleys.len())
                .ok_or(StructureError::ArithmeticOverflow)?;
            self.packets.push_back(FullPacketWork {
                packet,
                reversed_length: work.reversed_length + 1,
            });
        }
        self.link_cayleys(&work.packet, generator, is_type_one)
    }

    fn materialize_cayley_fiber(
        &mut self,
        sample_x: KgbId,
        cayleys: &[StandardReprMod],
        reversed_length: usize,
    ) -> Result<FullPacket, StructureError> {
        let rc = self.ctxt.rep_context();
        let reflections = rc
            .imaginary_simple_roots_at(sample_x)?
            .into_iter()
            .map(|root| crate::root_reflection::reflection_word(rc, root))
            .collect::<Result<Vec<_>, _>>()?;
        let mut packet = Vec::new();
        let mut x_queue = VecDeque::from([sample_x]);
        while let Some(x) = x_queue.pop_front() {
            if self.x_seen.contains(&x) {
                continue;
            }
            self.x_seen.insert(x);
            packet.push(self.materialize_row(x, cayleys, reversed_length)?);
            for word in &reflections {
                x_queue.push_back(cross_kgb_word(rc, word, x)?);
            }
        }
        Ok(packet)
    }

    fn link_cayleys(
        &mut self,
        packet: &FullPacket,
        generator: usize,
        is_type_one: bool,
    ) -> Result<(), StructureError> {
        for row in packet {
            for value in row {
                if !self.ctxt.is_parity(generator, value)? {
                    continue;
                }
                let source = self.registered(value, "Cayley source registered")?;
                let mut down = self.ctxt.down_cayley(generator, value)?;
                let first = self.registered(&down, "first Cayley target registered")?;
                self.block.fields[generator][source].cayley_image.0 = Some(first);
                self.block.add_inverse_cayley(generator, first, source)?;
                if is_type_one {
                    down = self.ctxt.cross(generator, &down)?;
                    let second = self.registered(&down, "second Cayley target registered")?;
                    self.block.fields[generator][source].cayley_image.1 = Some(second);
                    self.block.add_inverse_cayley(generator, second, source)?;
                }
            }
        }
        Ok(())
    }

    fn registered(
        &self,
        value: &StandardReprMod,
        invariant: &'static str,
    ) -> Result<usize, StructureError> {
        self.block
            .lookup(value)
            .ok_or(StructureError::RepInvariantViolation { invariant })
    }

    /// Step 5 (`blocks.cpp:1066-1080`): reverse temporary lengths, sort and
    /// remap every link, then locate the original complete SRM.
    fn finish(mut self, seed: &StandardReprMod) -> Result<(PartialBlock, usize), StructureError> {
        self.block.highest_x = self
            .x_seen
            .iter()
            .map(|x| x.index())
            .max()
            .unwrap_or(seed.x().index());
        self.block.highest_y = self.y_count.saturating_sub(1);
        let max_reversed_length = self.block.lengths.last().copied().unwrap_or(0);
        for length in &mut self.block.lengths {
            *length = max_reversed_length.checked_sub(*length).ok_or(
                StructureError::RepInvariantViolation {
                    invariant: "common-block reversed length",
                },
            )?;
        }
        self.block.sort();
        let init = self
            .block
            .lookup(seed)
            .ok_or(StructureError::RepInvariantViolation {
                invariant: "seed belongs to generated full common block",
            })?;
        Ok((self.block, init))
    }
}

impl PartialBlock {
    /// The full common-block constructor (`blocks.cpp:733-1081`).
    ///
    /// The empty integral subsystem is a singleton.  Subsystem generators are
    /// transported through `CommonContext`, so the same packet construction
    /// handles both ambient-full and proper integral systems.
    pub fn build_full(
        ctxt: &CommonContext<'_, '_>,
        seed: &StandardReprMod,
    ) -> Result<(Self, usize), StructureError> {
        if ctxt.rank() == 0 {
            let block = Self::build(ctxt, std::slice::from_ref(seed))?;
            return Ok((block, 0));
        }

        let top = full_block_top_ascent(ctxt, seed)?;
        let top_row = full_block_initial_real_orbit(ctxt, top)?;
        let mut builder = FullBlockBuilder::from_top_row(ctxt, top_row)?;
        builder.expand_fifo_packets()?;
        builder.finish(seed)
    }

    fn push_full_element(
        &mut self,
        value: StandardReprMod,
        y: usize,
        reversed_length: usize,
    ) -> Result<usize, StructureError> {
        if self.lookup_table.contains_key(&value) {
            return Err(StructureError::RepInvariantViolation {
                invariant: "new full-block SRM",
            });
        }
        let index = self.elements.len();
        self.lookup_table.insert(value.clone(), index);
        self.xs.push(value.x());
        self.ys.push(y);
        self.lengths.push(reversed_length);
        self.descents
            .extend(std::iter::repeat_n(BlockDescent::ComplexAscent, self.rank));
        for table in &mut self.fields {
            table.push(BlockFields::default());
        }
        self.elements.push(value);
        Ok(index)
    }

    fn add_inverse_cayley(
        &mut self,
        generator: usize,
        target: usize,
        source: usize,
    ) -> Result<(), StructureError> {
        let slot = &mut self.fields[generator][target].cayley_image;
        if slot.0.is_none() {
            slot.0 = Some(source);
        } else if slot.1.is_none() {
            slot.1 = Some(source);
        } else {
            return Err(StructureError::RepInvariantViolation {
                invariant: "two inverse Cayley slots",
            });
        }
        Ok(())
    }

    /// The partial block constructor (blocks.cpp:1086-1248). `interval`
    /// is the srm list produced by [`bruhat_below`]; it is consumed in
    /// `x`-sorted order so that descents precede when lengths are set.
    pub fn build(
        ctxt: &CommonContext<'_, '_>,
        interval: &[StandardReprMod],
    ) -> Result<Self, StructureError> {
        let rc = ctxt.rep_context();
        let rank = ctxt.rank();
        let size = interval.len();

        // Per involution, the sorted list of distinct gamma_lambda values;
        // y offsets accumulate over DECREASING involution numbers
        // (blocks.cpp:1106-1131).
        let mut y_lists: BTreeMap<crate::InvolutionId, Vec<RationalWeight>> = BTreeMap::new();
        let mut highest_x = 0_usize;
        for srm in interval {
            highest_x = highest_x.max(srm.x().index());
            let involution = rc.involution_of(srm.x())?;
            let list = y_lists.entry(involution).or_default();
            if list.iter().all(|g| g != srm.gamma_lambda()) {
                list.push(srm.gamma_lambda().clone());
                list.sort_by(rational_weight_compare);
            }
        }
        let mut offsets: HashMap<crate::InvolutionId, usize> = HashMap::new();
        let mut total_y = 0_usize;
        for (involution, list) in y_lists.iter().rev() {
            offsets.insert(*involution, total_y);
            total_y += list.len();
        }
        let highest_y = total_y.saturating_sub(1);

        // pre-sort by x (stable, like upstream's list merge sort) so
        // descents precede when setting lengths (blocks.cpp:1133-1135)
        let mut sorted: Vec<StandardReprMod> = interval.to_vec();
        sorted.sort_by_key(StandardReprMod::x);

        let mut xs = Vec::new();
        xs.try_reserve_exact(size)
            .map_err(|_| StructureError::AllocationFailed { requested: size })?;
        let mut ys = Vec::new();
        ys.try_reserve_exact(size)
            .map_err(|_| StructureError::AllocationFailed { requested: size })?;
        let mut lookup_table = HashMap::new();
        for (i, srm) in sorted.iter().enumerate() {
            let involution = rc.involution_of(srm.x())?;
            let list = &y_lists[&involution];
            let position = list.iter().position(|g| g == srm.gamma_lambda()).ok_or(
                StructureError::RepInvariantViolation {
                    invariant: "gamma_lambda registered in y table",
                },
            )?;
            xs.push(srm.x());
            ys.push(offsets[&involution] + position);
            lookup_table.insert(srm.clone(), i);
        }
        let lookup = |srm: &StandardReprMod| lookup_table.get(srm).copied();

        let mut lengths = vec![0_usize; size];
        let mut descents = vec![BlockDescent::ComplexAscent; size * rank];
        let mut fields = vec![vec![BlockFields::default(); size]; rank];

        for (i, srm) in sorted.iter().enumerate() {
            for s in 0..rank {
                let (stat, flag) = ctxt.status(s, srm.x())?;
                match stat {
                    KgbStatus::Complex => {
                        descents[i * rank + s] = if flag {
                            BlockDescent::ComplexDescent
                        } else {
                            BlockDescent::ComplexAscent
                        };
                        if flag {
                            // set links both ways when seeing the descent
                            // (blocks.cpp:1169-1180)
                            let sz = lookup(&ctxt.cross(s, srm)?).ok_or(
                                StructureError::RepInvariantViolation {
                                    invariant: "complex descent inside interval",
                                },
                            )?;
                            if lengths[i] == 0 {
                                lengths[i] = lengths[sz] + 1;
                            }
                            // upstream also asserts lengths agree and the
                            // partner is a ComplexAscent; both are internal
                            // invariants of a downward-closed interval and
                            // are covered by the fixture tests instead.
                            fields[s][i].cross_image = Some(sz);
                            fields[s][sz].cross_image = Some(i);
                        }
                    }
                    KgbStatus::Real => {
                        if ctxt.is_parity(s, srm)? {
                            let srm_sz = ctxt.down_cayley(s, srm)?;
                            let sz =
                                lookup(&srm_sz).ok_or(StructureError::RepInvariantViolation {
                                    invariant: "Cayley descent inside interval",
                                })?;
                            if lengths[i] == 0 {
                                lengths[i] = lengths[sz] + 1;
                            }
                            fields[s][i].cayley_image.0 = Some(sz);
                            if flag {
                                // real type 1 (blocks.cpp:1196-1206)
                                descents[i * rank + s] = BlockDescent::RealTypeI;
                                fields[s][i].cross_image = Some(i);
                                fields[s][sz].cayley_image.0 = Some(i);
                                let sz2 = lookup(&ctxt.cross(s, &srm_sz)?).ok_or(
                                    StructureError::RepInvariantViolation {
                                        invariant: "second Cayley descent inside interval",
                                    },
                                )?;
                                fields[s][i].cayley_image.1 = Some(sz2);
                                fields[s][sz2].cayley_image.0 = Some(i);
                            } else {
                                // real type 2 (blocks.cpp:1207-1213)
                                descents[i * rank + s] = BlockDescent::RealTypeII;
                                let slot = &mut fields[s][sz].cayley_image;
                                // `first_free_slot` (blocks.cpp:153)
                                if slot.0.is_none() {
                                    slot.0 = Some(i);
                                } else if slot.1.is_none() {
                                    slot.1 = Some(i);
                                } else {
                                    return Err(StructureError::RepInvariantViolation {
                                        invariant: "two Cayley ascent slots",
                                    });
                                }
                                // the cross image may leave the interval
                                fields[s][i].cross_image = lookup(&ctxt.cross(s, srm)?);
                            }
                        } else {
                            descents[i * rank + s] = BlockDescent::RealNonparity;
                            fields[s][i].cross_image = Some(i);
                        }
                    }
                    KgbStatus::ImaginaryCompact => {
                        descents[i * rank + s] = BlockDescent::ImaginaryCompact;
                        fields[s][i].cross_image = Some(i);
                    }
                    KgbStatus::ImaginaryNoncompact => {
                        if flag {
                            descents[i * rank + s] = BlockDescent::ImaginaryTypeI;
                            // the cross image may leave the interval
                            fields[s][i].cross_image = lookup(&ctxt.cross(s, srm)?);
                        } else {
                            descents[i * rank + s] = BlockDescent::ImaginaryTypeII;
                            fields[s][i].cross_image = Some(i);
                        }
                    }
                }
            }
        }

        let mut block = Self {
            rank,
            elements: sorted,
            lookup_table,
            xs,
            ys,
            lengths,
            descents,
            fields,
            highest_x,
            highest_y,
        };
        block.sort();
        Ok(block)
    }

    /// `common_block::sort` (blocks.cpp:1488-1517): reorder by increasing
    /// `(length, x, y)`, remapping the cross and Cayley links.
    fn sort(&mut self) {
        let size = self.elements.len();
        let mut order: Vec<usize> = (0..size).collect();
        order.sort_by(|&a, &b| {
            (self.lengths[a], self.xs[a], self.ys[a]).cmp(&(
                self.lengths[b],
                self.xs[b],
                self.ys[b],
            ))
        });
        let mut rank_of = vec![0_usize; size];
        for (new, &old) in order.iter().enumerate() {
            rank_of[old] = new;
        }
        let take = |values: &mut Vec<usize>, order: &[usize]| {
            let old = std::mem::take(values);
            *values = order.iter().map(|&i| old[i]).collect();
        };
        take(&mut self.lengths, &order);
        take(&mut self.ys, &order);
        let old_xs = std::mem::take(&mut self.xs);
        self.xs = order.iter().map(|&i| old_xs[i]).collect();
        let old_elements = std::mem::take(&mut self.elements);
        self.elements = order.iter().map(|&i| old_elements[i].clone()).collect();
        let old_descents = std::mem::take(&mut self.descents);
        let rank = self.rank;
        let mut descents = Vec::with_capacity(old_descents.len());
        for &old in &order {
            descents.extend_from_slice(&old_descents[old * rank..old * rank + rank]);
        }
        self.descents = descents;
        self.lookup_table = self
            .elements
            .iter()
            .enumerate()
            .map(|(i, srm)| (srm.clone(), i))
            .collect();
        let old_fields = std::mem::take(&mut self.fields);
        self.fields = old_fields
            .into_iter()
            .map(|tab_s| {
                order
                    .iter()
                    .map(|&old| {
                        let mut entry = tab_s[old];
                        entry.cross_image = entry.cross_image.map(|z| rank_of[z]);
                        entry.cayley_image.0 = entry.cayley_image.0.map(|z| rank_of[z]);
                        entry.cayley_image.1 = entry.cayley_image.1.map(|z| rank_of[z]);
                        entry
                    })
                    .collect()
            })
            .collect();
    }

    pub fn size(&self) -> usize {
        self.elements.len()
    }

    /// The integral-subsystem rank (the number of generator columns).
    pub fn rank(&self) -> usize {
        self.rank
    }

    /// `common_block::lookup` (blocks.cpp:1250+): the block element of an
    /// srm, `None` outside the interval (upstream `UndefBlock`).
    pub fn lookup(&self, srm: &StandardReprMod) -> Option<usize> {
        self.lookup_table.get(srm).copied()
    }

    pub fn element(&self, z: usize) -> Option<&StandardReprMod> {
        self.elements.get(z)
    }

    pub fn x(&self, z: usize) -> Option<KgbId> {
        self.xs.get(z).copied()
    }

    pub fn y(&self, z: usize) -> Option<usize> {
        self.ys.get(z).copied()
    }

    pub fn length(&self, z: usize) -> Option<usize> {
        self.lengths.get(z).copied()
    }

    pub fn gamma_lambda(&self, z: usize) -> Option<&RationalWeight> {
        self.elements.get(z).map(StandardReprMod::gamma_lambda)
    }

    /// The per-generator descent status of element `z`.
    pub fn descent(&self, z: usize, generator: usize) -> Option<BlockDescent> {
        if generator >= self.rank {
            return None;
        }
        self.descents.get(z * self.rank + generator).copied()
    }

    /// `cross(s, z)` — `None` is upstream `UndefBlock` (the link leaves
    /// the interval or is not set).
    pub fn cross(&self, generator: usize, z: usize) -> Option<usize> {
        self.fields.get(generator)?.get(z)?.cross_image
    }

    /// The Cayley image pair: the forward Cayley targets for imaginary
    /// ascents, the inverse Cayley descents for real descents (upstream
    /// `block_fields::Cayley_image`; the print layer selects via
    /// `isWeakDescent`).
    pub fn cayley(&self, generator: usize, z: usize) -> Option<(Option<usize>, Option<usize>)> {
        Some(self.fields.get(generator)?.get(z)?.cayley_image)
    }

    /// `max_x` for the print width (block_io.cpp:58): the largest KGB
    /// number occurring in the interval.
    pub fn highest_x(&self) -> usize {
        self.highest_x
    }

    /// `max_y` for the print width (block_io.cpp:59).
    pub fn highest_y(&self) -> usize {
        self.highest_y
    }

    /// `Block_base::survives` (blocks.cpp:323-330): no singular generator
    /// is a descent of `z`.
    pub fn survives(&self, z: usize, singular: &[bool]) -> bool {
        singular
            .iter()
            .enumerate()
            .take(self.rank)
            .filter(|(_, &flag)| flag)
            .all(|(s, _)| {
                self.descent(z, s)
                    .is_some_and(|descent| !descent.is_descent())
            })
    }

    /// Upstream `Bare_block::dual` (gkmod/blocks.cpp:474-507), the
    /// common-block analogue of [`BlockGraph::dual`]: a pure data transform
    /// swapping the primal and dual roles of every row. Element order
    /// reverses (`z' = size-1-z`), the `x`/`y` coordinates swap, lengths
    /// reflect as `max_len - length(z)` with `max_len = length(size-1)`,
    /// each descent status maps by [`BlockDescent::dual`], and every
    /// cross/Cayley link `c` maps to `size-1-c` (a Cayley second image is
    /// mapped only when the first is defined). Upstream dualizes only full
    /// blocks, where cross links are always defined; a partial block may
    /// have links leaving the interval (`UndefBlock`), and those stay
    /// undefined here. Note the KL recursion is only valid on the dual of a
    /// link-closed source (a full block): an outgoing complex ascent of a
    /// partial interval dualizes to a complex descent with an undefined
    /// cross, which [`crate::KlTable`] rejects. The result is a
    /// [`BareBlock`]: the dual block's rows are not standard-rep parameters
    /// of this block's context, so the parameter pool and lookup table have
    /// no counterpart.
    pub fn dual(&self) -> BareBlock {
        let rank = self.rank;
        let size = self.size();
        let max_len = self.lengths.last().copied().unwrap_or(0);

        let mut xs: Vec<usize> = Vec::with_capacity(size);
        let mut ys: Vec<usize> = Vec::with_capacity(size);
        let mut lengths: Vec<usize> = Vec::with_capacity(size);
        let mut descents: Vec<BlockDescent> = Vec::with_capacity(size * rank);
        for z in (0..size).rev() {
            xs.push(self.ys[z]);
            ys.push(self.xs[z].index());
            lengths.push(max_len - self.lengths[z]);
            for generator in 0..rank {
                descents.push(self.descents[z * rank + generator].dual());
            }
        }

        let mut fields = vec![Vec::with_capacity(size); rank];
        for generator in 0..rank {
            for z in (0..size).rev() {
                let source = self.fields[generator][z];
                let mut entry = BlockFields {
                    cross_image: source.cross_image.map(|c| size - 1 - c),
                    cayley_image: (None, None),
                };
                if let Some(first) = source.cayley_image.0 {
                    entry.cayley_image.0 = Some(size - 1 - first);
                    if let Some(second) = source.cayley_image.1 {
                        entry.cayley_image.1 = Some(size - 1 - second);
                    }
                }
                fields[generator].push(entry);
            }
        }

        BareBlock {
            rank,
            xs,
            ys,
            lengths,
            descents,
            fields,
            highest_x: self.highest_y,
            highest_y: self.highest_x,
        }
    }
}

/// The combinatorial dual of a block: upstream `blocks::Bare_block`
/// (blocks.h), the block with primal and dual real forms interchanged.
///
/// Produced by [`PartialBlock::dual`] (and constructible only there); it
/// carries no per-element parameters, only the block combinatorics the KL
/// recursion consumes through [`crate::BlockTopology`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BareBlock {
    rank: usize,
    /// Dual-side `x` coordinate of each row (the source block's `y` value).
    xs: Vec<usize>,
    /// Dual-side `y` coordinate of each row (the source block's `x` index).
    ys: Vec<usize>,
    lengths: Vec<usize>,
    /// `descent[z * rank + s]`.
    descents: Vec<BlockDescent>,
    /// `data[s][z]`, generator-major as upstream.
    fields: Vec<Vec<BlockFields>>,
    highest_x: usize,
    highest_y: usize,
}

impl BareBlock {
    pub fn size(&self) -> usize {
        self.xs.len()
    }

    /// The block rank (the number of generator columns).
    pub fn rank(&self) -> usize {
        self.rank
    }

    /// The dual-side `x` coordinate of `z` (upstream `Block_base::x`).
    pub fn x(&self, z: usize) -> Option<usize> {
        self.xs.get(z).copied()
    }

    /// The dual-side `y` coordinate of `z` (upstream `Block_base::y`).
    pub fn y(&self, z: usize) -> Option<usize> {
        self.ys.get(z).copied()
    }

    pub fn length(&self, z: usize) -> Option<usize> {
        self.lengths.get(z).copied()
    }

    /// The per-generator descent status of element `z`.
    pub fn descent(&self, z: usize, generator: usize) -> Option<BlockDescent> {
        if generator >= self.rank {
            return None;
        }
        self.descents.get(z * self.rank + generator).copied()
    }

    /// `cross(s, z)` — `None` is upstream `UndefBlock`.
    pub fn cross(&self, generator: usize, z: usize) -> Option<usize> {
        self.fields.get(generator)?.get(z)?.cross_image
    }

    /// The raw Cayley image pair (upstream `block_fields::Cayley_image`):
    /// forward Cayley targets on ascent cells, inverse Cayley descents on
    /// descent cells.
    pub fn cayley(&self, generator: usize, z: usize) -> Option<(Option<usize>, Option<usize>)> {
        Some(self.fields.get(generator)?.get(z)?.cayley_image)
    }

    /// `max_x` of the dual block: the source block's `max_y`
    /// (blocks.cpp:479).
    pub fn highest_x(&self) -> usize {
        self.highest_x
    }

    /// `max_y` of the dual block: the source block's `max_x`.
    pub fn highest_y(&self) -> usize {
        self.highest_y
    }
}

impl crate::block_access::sealed::Sealed for BareBlock {}

impl crate::BlockTopology for BareBlock {
    fn size(&self) -> usize {
        BareBlock::size(self)
    }

    fn rank(&self) -> usize {
        BareBlock::rank(self)
    }

    fn length(&self, element: usize) -> Option<usize> {
        BareBlock::length(self, element)
    }

    fn descent(&self, element: usize, generator: usize) -> Option<BlockDescent> {
        BareBlock::descent(self, element, generator)
    }

    fn cross(&self, element: usize, generator: usize) -> Option<usize> {
        BareBlock::cross(self, generator, element)
    }

    fn cayley(&self, element: usize, generator: usize) -> Option<(Option<usize>, Option<usize>)> {
        if BareBlock::descent(self, element, generator)?.is_descent() {
            return Some((None, None));
        }
        BareBlock::cayley(self, generator, element)
    }

    fn inverse_cayley(
        &self,
        element: usize,
        generator: usize,
    ) -> Option<(Option<usize>, Option<usize>)> {
        if !BareBlock::descent(self, element, generator)?.is_descent() {
            return Some((None, None));
        }
        BareBlock::cayley(self, generator, element)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AdjointFiberBudget, BasedRootDatum, CartanClassification, CartanClassificationBudget,
        CartanId, InnerClass, IntegerLatticeBudget, InvolutionTable, InvolutionTableBudget,
        KgbGraph, LatticeInvolution, RealFormSeed, StrongRealClassification, WeakRealFormId,
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

    /// `simply_connected(Lie_type("A1"),true)` with its compact inner
    /// class; the split form sl(2,R) has KGB size 3.
    fn a1_fixture() -> ContextFixture {
        let datum = BasedRootDatum::from_simple_data(
            1,
            vec![vec![2]],
            vec![Weight::new(vec![2])],
            vec![Coweight::new(vec![1])],
        )
        .unwrap();
        let involution = LatticeInvolution::identity(&datum).unwrap();
        fixture(datum, involution, 2, 3)
    }

    /// `simply_connected(Lie_type("B2"),true)` with its compact inner
    /// class; the split form so(3,2) has KGB size 11.
    fn b2_fixture() -> ContextFixture {
        let datum = BasedRootDatum::from_simple_data(
            2,
            vec![vec![2, -2], vec![-1, 2]],
            vec![Weight::new(vec![2, -2]), Weight::new(vec![-1, 2])],
            vec![Coweight::new(vec![1, 0]), Coweight::new(vec![0, 1])],
        )
        .unwrap();
        let involution = LatticeInvolution::identity(&datum).unwrap();
        fixture(datum, involution, 8, 11)
    }

    /// The wrapper's seed path (atlas-types.w:6703-6705):
    /// `StandardReprMod::mod_reduce` of the parameter, then the
    /// `common_context` on the seed's `gamma_lambda`.
    fn seed_and_context<'r, 'a>(
        rc: &'r RepContext<'a>,
        x: usize,
        lambda_rho: &[i32],
        gamma: &RationalWeight,
    ) -> (StandardReprMod, CommonContext<'r, 'a>) {
        let z = rc
            .sr_gamma(KgbId(x), &Weight::new(lambda_rho.to_vec()), gamma)
            .unwrap();
        let seed = StandardReprMod::mod_reduce(rc, &z).unwrap();
        let ctxt = CommonContext::integral(rc, seed.gamma_lambda()).unwrap();
        (seed, ctxt)
    }

    #[test]
    fn standard_repr_mod_restores_the_full_parameter() {
        let fixture = a1_fixture();
        let rc = fixture.rc();
        let gamma = rw(&[1], 1);
        let original = rc
            .sr_gamma(KgbId(2), &Weight::new(vec![1]), &gamma)
            .unwrap();
        let reduced = StandardReprMod::mod_reduce(&rc, &original).unwrap();
        let restored = reduced.to_standard(&rc, &gamma).unwrap();
        assert_eq!(restored, original);
    }

    fn interval_xs(ctxt: &CommonContext<'_, '_>, seed: &StandardReprMod) -> Vec<usize> {
        let mut xs: Vec<usize> = bruhat_below(ctxt, seed)
            .unwrap()
            .iter()
            .map(|srm| srm.x().index())
            .collect();
        xs.sort_unstable();
        xs
    }

    fn rw(numerator: &[i64], denominator: i64) -> RationalWeight {
        RationalWeight::new(numerator.to_vec(), denominator).unwrap()
    }

    /// A1 p of tests/reference/domain/print_partial_block.events.json:
    /// `param(KGB(rf,2),[1],[1]/2)` — gamma-lambda [1]/2 is not integral,
    /// so the integral subsystem has rank 0 and the interval is the
    /// singleton `0:  0  []   *(x=2,gamma-lambda=  [1]/2)  1^e`.
    #[test]
    fn a1_half_integral_seed_gives_singleton_interval() {
        let a1 = a1_fixture();
        let rc = a1.rc();
        let gamma = rw(&[5], 2); // lambda [2]/1 + nu [1]/2
        let (seed, ctxt) = seed_and_context(&rc, 2, &[1], &gamma);
        assert_eq!(*seed.gamma_lambda(), rw(&[1], 2), "pinned gamma-lambda");
        assert_eq!(ctxt.rank(), 0, "no integrally simple coroot");
        let interval = bruhat_below(&ctxt, &seed).unwrap();
        assert_eq!(interval.len(), 1);
        assert_eq!(interval[0], seed, "the seed alone is the interval");
        let block = PartialBlock::build(&ctxt, &interval).unwrap();
        assert_eq!(block.size(), 1);
        assert_eq!(block.rank(), 0);
        assert_eq!(block.x(0), Some(KgbId(2)));
        assert_eq!(block.length(0), Some(0));
        assert_eq!(block.gamma_lambda(0), Some(&rw(&[1], 2)));
    }

    #[test]
    fn full_common_block_rank_zero_is_the_seed_singleton() {
        let a1 = a1_fixture();
        let rc = a1.rc();
        let seed = StandardReprMod::build(&rc, KgbId(2), &rw(&[1], 2)).unwrap();
        let ctxt = CommonContext::integral(&rc, seed.gamma_lambda()).unwrap();

        let (block, init) = PartialBlock::build_full(&ctxt, &seed).unwrap();

        assert_eq!(init, 0);
        assert_eq!(block.size(), 1);
        assert_eq!(block.rank(), 0);
        assert_eq!(block.element(0), Some(&seed));
        assert_eq!(block.length(0), Some(0));
    }

    #[test]
    fn a1_full_common_block_is_seed_independent_with_exact_init() {
        let a1 = a1_fixture();
        let rc = a1.rc();
        let expected_gamma_lambda = rw(&[0], 1);

        for seed_x in 0..3 {
            let seed = StandardReprMod::build(&rc, KgbId(seed_x), &expected_gamma_lambda).unwrap();
            let ctxt = CommonContext::integral(&rc, seed.gamma_lambda()).unwrap();
            let (block, init) = PartialBlock::build_full(&ctxt, &seed).unwrap();

            assert_eq!(block.size(), 3);
            assert_eq!(init, seed_x);
            assert_eq!(block.lookup(&seed), Some(seed_x));
            assert_eq!(
                (0..3).map(|z| block.x(z)).collect::<Vec<_>>(),
                vec![Some(KgbId(0)), Some(KgbId(1)), Some(KgbId(2))]
            );
            assert_eq!(
                (0..3).map(|z| block.length(z)).collect::<Vec<_>>(),
                vec![Some(0), Some(0), Some(1)]
            );
            assert_eq!(
                (0..3).map(|z| block.descent(z, 0)).collect::<Vec<_>>(),
                vec![
                    Some(BlockDescent::ImaginaryTypeI),
                    Some(BlockDescent::ImaginaryTypeI),
                    Some(BlockDescent::RealTypeI),
                ]
            );
            assert_eq!(
                (0..3).map(|z| block.cross(0, z)).collect::<Vec<_>>(),
                vec![Some(1), Some(0), Some(2)]
            );
            assert_eq!(
                (0..3).map(|z| block.cayley(0, z)).collect::<Vec<_>>(),
                vec![
                    Some((Some(2), None)),
                    Some((Some(2), None)),
                    Some((Some(0), Some(1))),
                ]
            );
        }
    }

    #[test]
    fn b2_full_common_block_matches_the_pinned_twelve_rows() {
        let b2 = b2_fixture();
        let rc = b2.rc();
        let zero = rw(&[0, 0], 1);
        let seed = StandardReprMod::build(&rc, KgbId(0), &zero).unwrap();
        let ctxt = CommonContext::integral(&rc, seed.gamma_lambda()).unwrap();

        let (block, init) = PartialBlock::build_full(&ctxt, &seed).unwrap();

        assert_eq!(init, 0);
        assert_eq!(block.size(), 12);
        assert_eq!(block.rank(), 2);
        assert_eq!(
            (0..12)
                .map(|z| block.x(z).unwrap().index())
                .collect::<Vec<_>>(),
            vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 10]
        );
        let shifted = rw(&[1, 0], 1);
        assert_eq!(
            (0..12)
                .map(|z| (
                    block.x(z).unwrap().index(),
                    block.gamma_lambda(z).unwrap().clone(),
                ))
                .collect::<Vec<_>>(),
            vec![
                (0, zero.clone()),
                (1, zero.clone()),
                (2, zero.clone()),
                (3, zero.clone()),
                (4, zero.clone()),
                (5, zero.clone()),
                (6, zero.clone()),
                (7, zero.clone()),
                (8, zero.clone()),
                (9, zero.clone()),
                (10, zero.clone()),
                (10, shifted.clone()),
            ]
        );
        assert_eq!(
            (0..12).map(|z| block.y(z).unwrap()).collect::<Vec<_>>(),
            vec![6, 6, 6, 6, 4, 4, 5, 3, 2, 2, 0, 1]
        );
        assert_eq!(block.highest_y(), 6);
        assert_eq!(
            (0..12)
                .map(|z| block.length(z).unwrap())
                .collect::<Vec<_>>(),
            vec![0, 0, 0, 0, 1, 1, 1, 2, 2, 2, 3, 3]
        );
        let expected_descents = [
            [BlockDescent::ImaginaryTypeI, BlockDescent::ImaginaryTypeI],
            [BlockDescent::ImaginaryTypeI, BlockDescent::ImaginaryCompact],
            [BlockDescent::ImaginaryTypeI, BlockDescent::ImaginaryTypeI],
            [BlockDescent::ImaginaryTypeI, BlockDescent::ImaginaryCompact],
            [BlockDescent::RealTypeI, BlockDescent::ComplexAscent],
            [BlockDescent::RealTypeI, BlockDescent::ComplexAscent],
            [BlockDescent::ComplexAscent, BlockDescent::RealTypeI],
            [BlockDescent::ComplexDescent, BlockDescent::ImaginaryTypeII],
            [BlockDescent::ImaginaryTypeI, BlockDescent::ComplexDescent],
            [BlockDescent::ImaginaryTypeI, BlockDescent::ComplexDescent],
            [BlockDescent::RealTypeI, BlockDescent::RealTypeII],
            [BlockDescent::RealNonparity, BlockDescent::RealTypeII],
        ];
        for (z, row) in expected_descents.iter().enumerate() {
            for (s, &expected) in row.iter().enumerate() {
                assert_eq!(block.descent(z, s), Some(expected), "descent[{z},{s}]");
            }
        }
        let expected_cross = [
            [1, 0, 3, 2, 4, 5, 7, 6, 9, 8, 10, 11],
            [2, 1, 0, 3, 8, 9, 6, 7, 4, 5, 11, 10],
        ];
        let expected_cayley = [
            [
                (Some(4), None),
                (Some(4), None),
                (Some(5), None),
                (Some(5), None),
                (Some(0), Some(1)),
                (Some(2), Some(3)),
                (None, None),
                (None, None),
                (Some(10), None),
                (Some(10), None),
                (Some(8), Some(9)),
                (None, None),
            ],
            [
                (Some(6), None),
                (None, None),
                (Some(6), None),
                (None, None),
                (None, None),
                (None, None),
                (Some(0), Some(2)),
                (Some(10), Some(11)),
                (None, None),
                (None, None),
                (Some(7), None),
                (Some(7), None),
            ],
        ];
        for s in 0..2 {
            for z in 0..12 {
                assert_eq!(
                    block.cross(s, z),
                    Some(expected_cross[s][z]),
                    "cross[{z},{s}]"
                );
                assert_eq!(
                    block.cayley(s, z),
                    Some(expected_cayley[s][z]),
                    "cayley[{z},{s}]"
                );
            }
        }

        let top_zero = StandardReprMod::build(&rc, KgbId(10), &zero).unwrap();
        let top_shifted = StandardReprMod::build(&rc, KgbId(10), &shifted).unwrap();
        assert_eq!(block.lookup(&top_zero), Some(10));
        assert_eq!(block.lookup(&top_shifted), Some(11));
        assert_ne!(block.element(10), block.element(11));

        // The same common block is recovered from the oracle's second B2
        // seed (`pb2`), with only the returned entry index changing.
        let middle_seed = StandardReprMod::build(&rc, KgbId(5), &zero).unwrap();
        let (from_middle, middle_init) = PartialBlock::build_full(&ctxt, &middle_seed).unwrap();
        assert_eq!(middle_init, 5);
        assert_eq!(from_middle.size(), block.size());
        for z in 0..block.size() {
            assert_eq!(from_middle.element(z), block.element(z), "element[{z}]");
            assert_eq!(from_middle.length(z), block.length(z), "length[{z}]");
            for s in 0..block.rank() {
                assert_eq!(from_middle.descent(z, s), block.descent(z, s));
                assert_eq!(from_middle.cross(s, z), block.cross(s, z));
                assert_eq!(from_middle.cayley(s, z), block.cayley(s, z));
            }
        }
    }

    #[test]
    fn full_common_block_supports_proper_nonempty_integral_subsystem() {
        let b2 = b2_fixture();
        let rc = b2.rc();
        let seed = StandardReprMod::build(&rc, KgbId(5), &rw(&[3, 1], 2)).unwrap();
        let ctxt = CommonContext::integral(&rc, seed.gamma_lambda()).unwrap();
        assert_eq!(ctxt.rank(), 1);

        let (block, init) = PartialBlock::build_full(&ctxt, &seed).unwrap();
        assert_eq!(init, 1);
        assert_eq!(block.size(), 3);
        assert_eq!(
            (0..block.size())
                .map(|z| block.x(z).unwrap().index())
                .collect::<Vec<_>>(),
            vec![4, 5, 10]
        );
        assert_eq!(
            (0..block.size())
                .map(|z| block.length(z).unwrap())
                .collect::<Vec<_>>(),
            vec![0, 0, 1]
        );
    }

    /// A1 q3 of the reference: `param(KGB(rf,2),[1],[0]/1)` — the 3-row
    /// interval x=0,1,2 with the rows
    /// `0:  0  [i1]  1   (2,*)  *(x=0,...)` / `1: ... (x=1,...)` /
    /// `2:  1  [r1]  2   (0,1)   (x=2,...)  1^e`.
    #[test]
    fn a1_integral_seed_gives_three_row_interval() {
        let a1 = a1_fixture();
        let rc = a1.rc();
        let gamma = rw(&[2], 1); // lambda [2]/1 + nu [0]/1
        let (seed, ctxt) = seed_and_context(&rc, 2, &[1], &gamma);
        assert_eq!(*seed.gamma_lambda(), rw(&[0], 1));
        assert_eq!(ctxt.rank(), 1, "the full rank-1 integral system");
        assert_eq!(interval_xs(&ctxt, &seed), vec![0, 1, 2]);
        let interval = bruhat_below(&ctxt, &seed).unwrap();
        assert_eq!(interval.last(), Some(&seed), "the seed generates last");
        let block = PartialBlock::build(&ctxt, &interval).unwrap();
        assert_eq!(block.size(), 3);
        for z in 0..3 {
            assert_eq!(block.x(z), Some(KgbId(z)), "rows are x=0,1,2");
            assert_eq!(block.gamma_lambda(z), Some(&rw(&[0], 1)));
        }
        assert_eq!(block.length(0), Some(0));
        assert_eq!(block.length(1), Some(0));
        assert_eq!(block.length(2), Some(1));
        assert_eq!(block.descent(0, 0), Some(BlockDescent::ImaginaryTypeI));
        assert_eq!(block.descent(1, 0), Some(BlockDescent::ImaginaryTypeI));
        assert_eq!(block.descent(2, 0), Some(BlockDescent::RealTypeI));
        // cross column `1 0 2`
        assert_eq!(block.cross(0, 0), Some(1));
        assert_eq!(block.cross(0, 1), Some(0));
        assert_eq!(block.cross(0, 2), Some(2));
        // Cayley column `(2,*) (2,*) (0,1)`
        assert_eq!(block.cayley(0, 0), Some((Some(2), None)));
        assert_eq!(block.cayley(0, 1), Some((Some(2), None)));
        assert_eq!(block.cayley(0, 2), Some((Some(0), Some(1))));
        // singular(gamma) on the full system: <[2],[1]> != 0, so every
        // row survives; the `*` flag of rows 0,1 vs ` ` of row 2 comes
        // from `survives` — all true here
        let singular = ctxt.singular_flags(&gamma).unwrap();
        assert_eq!(singular, vec![false]);
        assert!((0..3).all(|z| block.survives(z, &singular)));
    }

    /// B2 pb of the reference: `param(KGB(rfb,0),[1,1],[0,0]/1)` — the
    /// most compact element has only imaginary ascents, so the interval is
    /// the singleton `0:  0  [i1,i1]  *  *   (*,*)  (*,*)  *(x=0,...)  e`.
    #[test]
    fn b2_most_compact_seed_gives_singleton_interval() {
        let b2 = b2_fixture();
        let rc = b2.rc();
        let gamma = rw(&[2, 2], 1);
        let (seed, ctxt) = seed_and_context(&rc, 0, &[1, 1], &gamma);
        assert_eq!(*seed.gamma_lambda(), rw(&[0, 0], 1));
        assert_eq!(ctxt.rank(), 2, "the full rank-2 integral system");
        assert_eq!(interval_xs(&ctxt, &seed), vec![0]);
        let interval = bruhat_below(&ctxt, &seed).unwrap();
        let block = PartialBlock::build(&ctxt, &interval).unwrap();
        assert_eq!(block.size(), 1);
        assert_eq!(block.rank(), 2);
        assert_eq!(block.descent(0, 0), Some(BlockDescent::ImaginaryTypeI));
        assert_eq!(block.descent(0, 1), Some(BlockDescent::ImaginaryTypeI));
        // both cross links leave the interval: printed `*`
        assert_eq!(block.cross(0, 0), None);
        assert_eq!(block.cross(1, 0), None);
        assert_eq!(block.cayley(0, 0), Some((None, None)));
        assert_eq!(block.cayley(1, 0), Some((None, None)));
    }

    /// B2 pb2 of the reference: `param(KGB(rfb,5),[1,1],[0,0]/1)` — the
    /// 3-row interval x=2,3,5 with rows
    /// `0:  0  [i1,i1]  1  *   (2,*)  (*,*)  *(x=2,...)` /
    /// `1:  0  [i1,ic]  0  1   (2,*)  (*,*)  *(x=3,...)` /
    /// `2:  1  [r1,C+]  2  *   (0,1)  (*,*)   (x=5,...)  1^e`.
    #[test]
    fn b2_split_seed_gives_three_row_interval() {
        let b2 = b2_fixture();
        let rc = b2.rc();
        let gamma = rw(&[2, 2], 1);
        let (seed, ctxt) = seed_and_context(&rc, 5, &[1, 1], &gamma);
        assert_eq!(*seed.gamma_lambda(), rw(&[0, 0], 1));
        assert_eq!(interval_xs(&ctxt, &seed), vec![2, 3, 5]);
        let interval = bruhat_below(&ctxt, &seed).unwrap();
        assert_eq!(interval.last(), Some(&seed), "the seed generates last");
        let block = PartialBlock::build(&ctxt, &interval).unwrap();
        assert_eq!(block.size(), 3);
        assert_eq!(block.x(0), Some(KgbId(2)));
        assert_eq!(block.x(1), Some(KgbId(3)));
        assert_eq!(block.x(2), Some(KgbId(5)));
        for z in 0..3 {
            assert_eq!(block.gamma_lambda(z), Some(&rw(&[0, 0], 1)));
        }
        assert_eq!(block.length(0), Some(0));
        assert_eq!(block.length(1), Some(0));
        assert_eq!(block.length(2), Some(1));
        // descent columns [i1,i1] / [i1,ic] / [r1,C+]
        assert_eq!(block.descent(0, 0), Some(BlockDescent::ImaginaryTypeI));
        assert_eq!(block.descent(0, 1), Some(BlockDescent::ImaginaryTypeI));
        assert_eq!(block.descent(1, 0), Some(BlockDescent::ImaginaryTypeI));
        assert_eq!(block.descent(1, 1), Some(BlockDescent::ImaginaryCompact));
        assert_eq!(block.descent(2, 0), Some(BlockDescent::RealTypeI));
        assert_eq!(block.descent(2, 1), Some(BlockDescent::ComplexAscent));
        // cross columns `1 * / 0 1 / 2 *`
        assert_eq!(block.cross(0, 0), Some(1));
        assert_eq!(block.cross(1, 0), None);
        assert_eq!(block.cross(0, 1), Some(0));
        assert_eq!(block.cross(1, 1), Some(1));
        assert_eq!(block.cross(0, 2), Some(2));
        assert_eq!(block.cross(1, 2), None);
        // Cayley columns `(2,*) (*,*) / (2,*) (*,*) / (0,1) (*,*)`
        assert_eq!(block.cayley(0, 0), Some((Some(2), None)));
        assert_eq!(block.cayley(1, 0), Some((None, None)));
        assert_eq!(block.cayley(0, 1), Some((Some(2), None)));
        assert_eq!(block.cayley(1, 1), Some((None, None)));
        assert_eq!(block.cayley(0, 2), Some((Some(0), Some(1))));
        assert_eq!(block.cayley(1, 2), Some((None, None)));
        assert_eq!(block.highest_x(), 5);
        // singular(gamma): both coroots pair to 2 with [2,2], so no
        // singular generator and every row survives (the `*`/` ` flag)
        let singular = ctxt.singular_flags(&gamma).unwrap();
        assert_eq!(singular, vec![false, false]);
        assert!((0..3).all(|z| block.survives(z, &singular)));
    }

    #[test]
    fn kl_table_can_own_an_arc_partial_block() {
        let b2 = b2_fixture();
        let rc = b2.rc();
        let gamma = rw(&[2, 2], 1);
        let (seed, ctxt) = seed_and_context(&rc, 5, &[1, 1], &gamma);
        let interval = bruhat_below(&ctxt, &seed).unwrap();
        let block = std::sync::Arc::new(PartialBlock::build(&ctxt, &interval).unwrap());

        let mut kl = crate::KlTable::from_handle(std::sync::Arc::clone(&block)).unwrap();
        drop(block);

        assert_eq!(kl.support().size(), 3);
        kl.fill(0).unwrap();
        assert_eq!(kl.kl_pol(2, 2).unwrap(), 1);
    }

    /// The seed of a proper half-integral subsystem: B2 with gamma
    /// `[3,1]/2` is integral only for the coroots pairing to even values,
    /// exercising `IntegralSubsystem` beyond the full/empty extremes.
    #[test]
    fn integral_subsystem_selects_integral_coroots() {
        let b2 = b2_fixture();
        let rc = b2.rc();
        let system = rc.root_system();
        // coroots are the coordinate basis; [3,1]/2 pairs integrally with
        // coroots whose first coordinate is even
        let sub = IntegralSubsystem::integral(system, &rw(&[3, 1], 2)).unwrap();
        // positive coroots [1,0], [0,1], [2,1], [1,1]: pairings 3, 1, 7,
        // 4 — only [1,1] is integral, a rank-1 subsystem on the NON-simple
        // parent root [0,2] = alpha_0 + 2*alpha_1
        assert_eq!(sub.rank(), 1);
        let root = sub.parent_root(0).unwrap();
        assert_eq!(system.root(root).unwrap().as_slice(), &[0, 2]);
    }

    #[test]
    fn full_integral_subsystem_uses_ambient_simple_generator_order() {
        let b2 = b2_fixture();
        let rc = b2.rc();
        let system = rc.root_system();
        let subsystem = IntegralSubsystem::full(system).unwrap();

        assert_eq!(subsystem.rank(), system.simple_root_ids().len());
        for (generator, &root) in system.simple_root_ids().iter().enumerate() {
            assert_eq!(subsystem.parent_root(generator), Some(root));
        }
    }

    #[test]
    fn integral_subsystem_exposes_exact_parent_words_to_rep_context() {
        let b2 = b2_fixture();
        let rc = b2.rc();
        let system = rc.root_system();
        let sub = IntegralSubsystem::integral(system, &rw(&[3, 1], 2)).unwrap();
        assert_eq!(sub.rank(), 1);
        let parent = sub.parent_root(0).unwrap();
        assert_eq!(system.root(parent).unwrap().as_slice(), &[0, 2]);
        let reflection = sub.reflection_word(0).unwrap();
        assert_eq!(reflection.len() % 2, 1);
        assert_eq!(
            reflection.iter().rev().copied().collect::<Vec<_>>(),
            reflection
        );
        assert!(sub.reflection_word(1).is_none());
    }

    /// `simply_connected(Lie_type("A2"),true)` with the quasi-split inner
    /// class `[[0,1],[1,0]]`; the split form sl(3,R) has KGB size 4.
    fn a2_quasisplit_fixture() -> ContextFixture {
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let involution = LatticeInvolution::new(
            &datum,
            vec![vec![0, 1], vec![1, 0]],
            vec![vec![0, 1], vec![1, 0]],
        )
        .unwrap();
        fixture(datum, involution, 6, 4)
    }

    /// Assert the KL pool indices of `dual` match an oracle
    /// `dual_KL_block` matrix: `expected[i][j]` (lower unitriangular,
    /// `j <= i`) is the pool index of `P_{last-i, last-j}`, with the pool
    /// seeded `{0: zero, 1: one}` (atlas-types.w:7113-7133).
    fn assert_dual_kl_matrix(dual: &BareBlock, expected: &[&[usize]]) {
        let mut kl = crate::KlTable::from_handle(dual.clone()).unwrap();
        kl.fill(0).unwrap();
        let last = dual.size() - 1;
        for (i, row) in expected.iter().enumerate() {
            for (j, &want) in row.iter().enumerate() {
                assert_eq!(kl.kl_pol(last - i, last - j).unwrap(), want, "M[{i}][{j}]");
            }
        }
    }

    /// Oracle (local capture, rev 4d3e9449): the B2 proper rank-1 fixture of
    /// `length_dual_proper.atlas` — `param(KGB(rfb,5),[1,1],[1,0]/2)` with
    /// `rfb = real_form(inner_class(rb,[[1,0],[0,1]]),2)`:
    /// ```text
    /// Parameter defines element 1 of the following common block,
    /// 0:  0  [i1]  1   (2,*)  *(x= 4,gamma-lambda=  [1,-1]/2)  1^e
    /// 1:  0  [i1]  0   (2,*)  *(x= 5,gamma-lambda=  [1,-1]/2)  1^e
    /// 2:  1  [r1]  2   (0,1)  *(x=10,gamma-lambda=   [3,3]/2)  1^2x1^e
    /// dual_KL_block: survivors x=4,5,10; start 1; polys [[],[1]]; matrix
    /// |1 0 0;0 1 0;1 1 1| (every survivor pair is the constant polynomial).
    /// ```
    #[test]
    fn b2_proper_full_block_dual_matches_the_oracle() {
        let b2 = b2_fixture();
        let rc = b2.rc();
        let gamma = rw(&[3, 1], 2);
        // param lambda [1,1] minus rho [1,1].
        let (seed, ctxt) = seed_and_context(&rc, 5, &[0, 0], &gamma);
        assert_eq!(ctxt.rank(), 1, "rank-1 subsystem on coroot [1,1]");

        let (block, init) = PartialBlock::build_full(&ctxt, &seed).unwrap();
        assert_eq!(init, 1);
        assert_eq!(block.size(), 3);
        assert_eq!(
            (0..3)
                .map(|z| block.x(z).unwrap().index())
                .collect::<Vec<_>>(),
            vec![4, 5, 10]
        );
        assert_eq!(
            (0..3)
                .map(|z| block.gamma_lambda(z).unwrap().clone())
                .collect::<Vec<_>>(),
            vec![rw(&[1, -1], 2), rw(&[1, -1], 2), rw(&[3, 3], 2)]
        );
        assert_eq!(
            (0..3).map(|z| block.length(z).unwrap()).collect::<Vec<_>>(),
            vec![0, 0, 1]
        );
        assert_eq!(
            (0..3)
                .map(|z| block.descent(z, 0).unwrap())
                .collect::<Vec<_>>(),
            vec![
                BlockDescent::ImaginaryTypeI,
                BlockDescent::ImaginaryTypeI,
                BlockDescent::RealTypeI,
            ]
        );
        assert_eq!(
            (0..3).map(|z| block.cross(0, z)).collect::<Vec<_>>(),
            vec![Some(1), Some(0), Some(2)]
        );
        assert_eq!(
            (0..3).map(|z| block.cayley(0, z)).collect::<Vec<_>>(),
            vec![
                Some((Some(2), None)),
                Some((Some(2), None)),
                Some((Some(0), Some(1))),
            ]
        );

        let dual = block.dual();
        assert_eq!(dual.size(), 3);
        assert_eq!(dual.rank(), 1);
        // Reversed rows: lengths reflect about max_len = 1, descents dualize
        // (r1->i2, i1->r2), links complement as size-1-c.
        assert_eq!(
            (0..3).map(|z| dual.length(z).unwrap()).collect::<Vec<_>>(),
            vec![0, 1, 1]
        );
        assert_eq!(
            (0..3)
                .map(|z| dual.descent(z, 0).unwrap())
                .collect::<Vec<_>>(),
            vec![
                BlockDescent::ImaginaryTypeII,
                BlockDescent::RealTypeII,
                BlockDescent::RealTypeII,
            ]
        );
        assert_eq!(
            (0..3).map(|z| dual.cross(0, z)).collect::<Vec<_>>(),
            vec![Some(0), Some(2), Some(1)]
        );
        assert_eq!(
            (0..3).map(|z| dual.cayley(0, z)).collect::<Vec<_>>(),
            vec![
                Some((Some(2), Some(1))),
                Some((Some(0), None)),
                Some((Some(0), None)),
            ]
        );
        // The x/y coordinates swap per row.
        for z in 0..3 {
            assert_eq!(dual.x(2 - z), block.y(z));
            assert_eq!(dual.y(2 - z), Some(block.x(z).unwrap().index()));
        }
        assert_eq!(dual.highest_x(), block.highest_y());
        assert_eq!(dual.highest_y(), block.highest_x());

        // No singular generator (gamma pairs to 2 with coroot [1,1]), so
        // every row survives. M[1][0] = P_{1,2} = 0: dual rows 1 and 2
        // share length 1.
        let singular = ctxt.singular_flags(&gamma).unwrap();
        assert_eq!(singular, vec![false]);
        assert!((0..3).all(|z| block.survives(z, &singular)));
        assert_dual_kl_matrix(&dual, &[&[1], &[0, 1], &[1, 1, 1]]);
    }

    /// Oracle (local capture, rev 4d3e9449): the A2 split fixture of
    /// `length_dual_proper_a2.atlas` — `param(KGB(rs,3),[0,0],[1,1]/2)` with
    /// `rs = real_form(inner_class(ra,[[0,1],[1,0]]),0)`:
    /// ```text
    /// Parameter defines element 1 of the following common block,
    /// 0:  0  [i2]  0   (1,2)  *(x=0,gamma-lambda=  [-1,1]/2)  e
    /// 1:  1  [r2]  2   (0,*)  *(x=3,gamma-lambda=   [3,3]/2)  1^2xe
    /// 2:  1  [r2]  1   (0,*)  *(x=3,gamma-lambda=   [1,1]/2)  1^2xe
    /// dual_KL_block: survivors x=0,3,3; start 1; polys [[],[1]]; matrix
    /// |1 0 0;1 1 0;1 0 1|.
    /// ```
    /// The gamma-lambda representatives are NOT pinned: the identity-attitude
    /// shift for this SL(3,R) family is the locator slice's known issue
    /// (nonintegral_common_block_workorder.md, cross-cutting caveat).
    #[test]
    fn a2_split_full_block_dual_matches_the_oracle() {
        let a2 = a2_quasisplit_fixture();
        let rc = a2.rc();
        let gamma = rw(&[1, 1], 2);
        // param lambda [0,0] minus rho [1,1].
        let (seed, ctxt) = seed_and_context(&rc, 3, &[-1, -1], &gamma);
        assert_eq!(ctxt.rank(), 1, "rank-1 subsystem on the highest root");

        let (block, init) = PartialBlock::build_full(&ctxt, &seed).unwrap();
        assert_eq!(init, 1);
        assert_eq!(block.size(), 3);
        assert_eq!(
            (0..3)
                .map(|z| block.x(z).unwrap().index())
                .collect::<Vec<_>>(),
            vec![0, 3, 3]
        );
        assert_eq!(
            (0..3).map(|z| block.length(z).unwrap()).collect::<Vec<_>>(),
            vec![0, 1, 1]
        );
        assert_eq!(
            (0..3)
                .map(|z| block.descent(z, 0).unwrap())
                .collect::<Vec<_>>(),
            vec![
                BlockDescent::ImaginaryTypeII,
                BlockDescent::RealTypeII,
                BlockDescent::RealTypeII,
            ]
        );
        assert_eq!(
            (0..3).map(|z| block.cross(0, z)).collect::<Vec<_>>(),
            vec![Some(0), Some(2), Some(1)]
        );
        assert_eq!(
            (0..3).map(|z| block.cayley(0, z)).collect::<Vec<_>>(),
            vec![
                Some((Some(1), Some(2))),
                Some((Some(0), None)),
                Some((Some(0), None)),
            ]
        );

        let dual = block.dual();
        assert_eq!(dual.size(), 3);
        assert_eq!(dual.rank(), 1);
        assert_eq!(
            (0..3).map(|z| dual.length(z).unwrap()).collect::<Vec<_>>(),
            vec![0, 0, 1]
        );
        assert_eq!(
            (0..3)
                .map(|z| dual.descent(z, 0).unwrap())
                .collect::<Vec<_>>(),
            vec![
                BlockDescent::ImaginaryTypeI,
                BlockDescent::ImaginaryTypeI,
                BlockDescent::RealTypeI,
            ]
        );
        assert_eq!(
            (0..3).map(|z| dual.cross(0, z)).collect::<Vec<_>>(),
            vec![Some(1), Some(0), Some(2)]
        );
        assert_eq!(
            (0..3).map(|z| dual.cayley(0, z)).collect::<Vec<_>>(),
            vec![
                Some((Some(2), None)),
                Some((Some(2), None)),
                Some((Some(1), Some(0))),
            ]
        );
        for z in 0..3 {
            assert_eq!(dual.x(2 - z), block.y(z));
            assert_eq!(dual.y(2 - z), Some(block.x(z).unwrap().index()));
        }

        let singular = ctxt.singular_flags(&gamma).unwrap();
        assert_eq!(singular, vec![false]);
        assert!((0..3).all(|z| block.survives(z, &singular)));
        // M[2][1] = P_{0,1} = 0: dual rows 0 and 1 share length 0.
        assert_dual_kl_matrix(&dual, &[&[1], &[1, 1], &[1, 0, 1]]);
    }

    /// The dual transform on the rank-2 three-row interval of
    /// `b2_split_seed_gives_three_row_interval`: every dual row is the exact
    /// reversal of the primal row, per-generator, including undefined links
    /// (a partial block's links may leave the interval, unlike the full
    /// blocks upstream dualizes).
    #[test]
    fn dual_of_partial_interval_reverses_every_generator_column() {
        let b2 = b2_fixture();
        let rc = b2.rc();
        let gamma = rw(&[2, 2], 1);
        let (seed, ctxt) = seed_and_context(&rc, 5, &[1, 1], &gamma);
        let interval = bruhat_below(&ctxt, &seed).unwrap();
        let block = PartialBlock::build(&ctxt, &interval).unwrap();
        let size = block.size();
        let rank = block.rank();
        assert_eq!((size, rank), (3, 2));
        let max_len = block.length(size - 1).unwrap();

        let dual = block.dual();
        assert_eq!(dual.size(), size);
        assert_eq!(dual.rank(), rank);
        for z in 0..size {
            let zd = size - 1 - z;
            assert_eq!(dual.length(zd), Some(max_len - block.length(z).unwrap()));
            assert_eq!(dual.x(zd), block.y(z));
            assert_eq!(dual.y(zd), Some(block.x(z).unwrap().index()));
            for s in 0..rank {
                assert_eq!(
                    dual.descent(zd, s),
                    block.descent(z, s).map(BlockDescent::dual)
                );
                assert_eq!(dual.cross(s, zd), block.cross(s, z).map(|c| size - 1 - c));
                let expected = match block.cayley(s, z).unwrap() {
                    (Some(first), second) => (Some(size - 1 - first), second.map(|c| size - 1 - c)),
                    (None, _) => (None, None),
                };
                assert_eq!(dual.cayley(s, zd), Some(expected));
            }
        }
        // The interval's undefined links (cross(1,0), cross(1,2)) stay
        // undefined in the dual. The dual of a NON-closed interval is not a
        // valid KL input (its primal complex ascents leaving the interval
        // become complex descents with an undefined cross), so no KL table
        // is filled here; the full-block tests above exercise that path.
        assert_eq!(dual.cross(1, size - 1), None);
        assert_eq!(dual.cross(1, 0), None);
    }
}
