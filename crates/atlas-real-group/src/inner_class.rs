use std::collections::{BTreeSet, VecDeque};
use std::sync::Arc;

use crate::cartan_class::ClassMembership;
use crate::grading::try_capacity;
use crate::twisted_involution::compose_matrices;
use crate::{
    pair, BasedRootDatum, Coweight, LatticeInvolution, RootId, RootInvolutionData, RootKind,
    RootSystem, StructureError, TwistedConjugacyClass, TwistedConjugacyPartition,
    TwistedInvolution, Weight, WeylAction, WeylElement,
};

/// Shared structural data at the beginning of an Atlas inner-class computation.
///
/// This is intentionally a partial implementation: it owns a validated based
/// root datum, its finite ordinary root system, and a distinguished root
/// involution. It can enumerate root-theoretic twisted-conjugacy orbits,
/// supplies the distinguished-involution context for
/// [`crate::CayleyCrossDecomposition`], and anchors provenance for
/// [`crate::RealFormLabels`], but does not yet build Atlas Cartan-class
/// fibers or own real-form data, nor does it contain the torus data
/// required to construct a KGB graph.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InnerClass {
    datum: BasedRootDatum,
    roots: RootSystem,
    distinguished_involution: RootInvolutionData,
}

/// Build the inner class defined by `involution` together with its Weyl
/// factor relative to the resulting distinguished involution.
pub fn inner_class_with_twisted_involution(
    datum: BasedRootDatum,
    involution: LatticeInvolution,
    root_budget: usize,
) -> Result<(WeylElement, InnerClass), StructureError> {
    let (inner_class, factor) =
        InnerClass::from_root_involution_with_factor(datum, involution, root_budget)?;
    Ok((factor, inner_class))
}

impl InnerClass {
    /// Build the shared root-theoretic state for an inner class.
    ///
    /// Root enumeration is deliberately caller-budgeted. A successful result
    /// proves that the distinguished lattice involution permutes this root
    /// system and transports its stored coroots, but is not yet a claim of
    /// Atlas real-form compatibility.
    pub fn new(
        datum: BasedRootDatum,
        distinguished_involution: LatticeInvolution,
        root_budget: usize,
    ) -> Result<Self, StructureError> {
        let roots = RootSystem::enumerate(&datum, root_budget)?;
        let distinguished_involution = RootInvolutionData::new(&roots, distinguished_involution)?;
        Self::with_roots(datum, roots, distinguished_involution)
    }

    /// Build the shared state from an arbitrary root-datum involution.
    ///
    /// This mirrors the upstream `inner_class(RootDatum,mat)` entry point
    /// (interpreter/atlas-types.w `check_involution`): any involution of the
    /// unbased root datum is accepted, validated to permute the root system
    /// and transport coroots, and then left-composed with the Weyl word that
    /// `wrt_distinguished` reads off the reflected simple-root images, which
    /// makes it an involution of the based datum. The Weyl word itself is
    /// forgotten, exactly as the upstream wrapper does.
    pub fn from_root_involution(
        datum: BasedRootDatum,
        involution: LatticeInvolution,
        root_budget: usize,
    ) -> Result<Self, StructureError> {
        Ok(Self::from_root_involution_and_word(datum, involution, root_budget)?.0)
    }

    /// Owned construction path shared by the Atlas `inner_class` and
    /// `twisted_involution` surfaces. Normalization produces the conjugating
    /// word, so the pair-returning surface must consume it here rather than
    /// clone the input and repeat `wrt_distinguished_word` afterward.
    fn from_root_involution_with_factor(
        datum: BasedRootDatum,
        involution: LatticeInvolution,
        root_budget: usize,
    ) -> Result<(Self, WeylElement), StructureError> {
        let (inner_class, word) =
            Self::from_root_involution_and_word(datum, involution, root_budget)?;
        let factor = inner_class.weyl_element_from_word(word)?;
        Ok((inner_class, factor))
    }

    fn from_root_involution_and_word(
        datum: BasedRootDatum,
        involution: LatticeInvolution,
        root_budget: usize,
    ) -> Result<(Self, Vec<usize>), StructureError> {
        let roots = RootSystem::enumerate(&datum, root_budget)?;
        let involution = RootInvolutionData::new(&roots, involution)?;
        let (distinguished, word) = wrt_distinguished_word(&datum, &roots, &involution)?;
        let distinguished = RootInvolutionData::new(&roots, distinguished)?;
        let inner_class = Self::with_roots(datum, roots, distinguished)?;
        Ok((inner_class, word))
    }

    fn with_roots(
        datum: BasedRootDatum,
        roots: RootSystem,
        distinguished_involution: RootInvolutionData,
    ) -> Result<Self, StructureError> {
        if !preserves_simple_system(&datum, &roots, &distinguished_involution)? {
            return Err(StructureError::InvalidBasedAutomorphism);
        }
        Ok(Self {
            datum,
            roots,
            distinguished_involution,
        })
    }

    pub fn datum(&self) -> &BasedRootDatum {
        &self.datum
    }

    pub fn root_system(&self) -> &RootSystem {
        &self.roots
    }

    pub fn distinguished_involution(&self) -> &RootInvolutionData {
        &self.distinguished_involution
    }

    /// Port of upstream `check_based_root_datum_involution`
    /// (interpreter/atlas-types.w:2787-2795): the involution must permute
    /// this class's root system and transport coroots (the
    /// [`RootInvolutionData`] gate), and additionally map every SIMPLE root
    /// to a simple root — upstream's "distinguished" rejection is
    /// [`StructureError::InvalidBasedAutomorphism`]. On success the induced
    /// simple-root permutation (upstream `rootdata::twist`) is returned.
    pub fn based_involution_twist(
        &self,
        involution: LatticeInvolution,
    ) -> Result<Vec<usize>, StructureError> {
        let data = RootInvolutionData::new(&self.roots, involution)?;
        let simple_ids = self.roots.simple_root_ids();
        let mut twist = Vec::with_capacity(simple_ids.len());
        for &simple_id in simple_ids {
            let image = data
                .image(simple_id)
                .ok_or(StructureError::InvalidRootAutomorphism)?;
            let position = simple_ids
                .iter()
                .position(|&candidate| candidate == image)
                .ok_or(StructureError::InvalidBasedAutomorphism)?;
            twist.push(position);
        }
        Ok(twist)
    }

    /// Port of upstream `twisted_from_involution`
    /// (interpreter/atlas-types.w:3844-3851): validate that `involution` — a
    /// root-datum involution, already checked square and involutive by the
    /// caller — lies in THIS inner class, and return the Weyl element `w`
    /// of its twisted-involution factorization `theta = w * delta` with
    /// `delta` distinguished. Upstream compares the reflected involution's
    /// simple-root twist against the class twist AND the reflected matrix
    /// against the distinguished one ("Involution not in this inner
    /// class"); the weight-matrix equality subsumes the twist comparison,
    /// and [`StructureError::InvalidBasedAutomorphism`] carries the
    /// rejection here.
    pub fn twisted_from_involution(
        &self,
        involution: LatticeInvolution,
    ) -> Result<WeylElement, StructureError> {
        let data = RootInvolutionData::new(&self.roots, involution)?;
        let (distinguished, word) = wrt_distinguished_word(&self.datum, &self.roots, &data)?;
        if distinguished.weight_matrix()
            != self.distinguished_involution.involution().weight_matrix()
        {
            return Err(StructureError::InvalidBasedAutomorphism);
        }
        self.weyl_element_from_word(word)
    }

    fn weyl_element_from_word(
        &self,
        word: impl IntoIterator<Item = usize>,
    ) -> Result<WeylElement, StructureError> {
        // Upstream `Weyl_group().element(ww)`: right-multiply the letters
        // left to right.
        let mut element = WeylElement::identity(&self.roots)?;
        for generator in word {
            let (next, _) = element.right_multiply_simple(&self.roots, generator)?;
            element = next;
        }
        Ok(element)
    }

    /// The distinguished involution's permutation of the simple generators
    /// (the `weyl::Twist` of upstream's `TwistedWeylGroup`): `twist[s]` is
    /// the generator whose simple root is the distinguished image of
    /// `alpha_s`.
    pub fn generator_twist(&self) -> Result<Vec<usize>, StructureError> {
        let simple_ids = self.roots.simple_root_ids();
        let mut twist = Vec::with_capacity(simple_ids.len());
        for &simple_id in simple_ids {
            let image = self
                .distinguished_involution
                .image(simple_id)
                .ok_or(StructureError::InvalidBasedAutomorphism)?;
            let position = simple_ids
                .iter()
                .position(|&candidate| candidate == image)
                .ok_or(StructureError::InvalidBasedAutomorphism)?;
            twist.push(position);
        }
        Ok(twist)
    }

    /// Canonicalize a twisted involution by the three-phase Atlas algorithm.
    ///
    /// This ports `InnerClass::canonicalize` from
    /// `sources/structure/innerclass.cpp:740-832`: first make the sums of the
    /// positive real and imaginary roots dominant, then restrict to simple
    /// generators orthogonal to both sums, and finally make the actual
    /// involution preserve positivity in the residual complex subsystem.
    ///
    /// The returned generators are in execution order. Repeatedly replacing
    /// `sigma` by `s * sigma * delta(s)` for each returned `s` transports the
    /// input to the returned canonical representative.
    pub fn canonicalize(
        &self,
        involution: TwistedInvolution,
    ) -> Result<(TwistedInvolution, Vec<usize>), StructureError> {
        let active = vec![true; self.datum.semisimple_rank()];
        self.canonicalize_with_generators(involution, &active)
    }

    /// `InnerClass::canonicalize` restricted to the simple generators in
    /// `active` (innerclass.cpp:740-832 with the `RankFlags gens` argument;
    /// `Rep_context::to_singular_canonical` uses this with the singular
    /// generators, repr.cpp:613-620). The residual subsystem of phase two is
    /// the intersection of `active` with the generators orthogonal to both
    /// dominant sums.
    pub fn canonicalize_with_generators(
        &self,
        involution: TwistedInvolution,
        active: &[bool],
    ) -> Result<(TwistedInvolution, Vec<usize>), StructureError> {
        self.validate_twisted_involution(&involution)?;
        let twist = self.generator_twist()?;
        let mut real_sum =
            positive_root_sum(&self.roots, involution.root_involution(), RootKind::Real)?;
        let mut imaginary_sum = positive_root_sum(
            &self.roots,
            involution.root_involution(),
            RootKind::Imaginary,
        )?;
        let mut action = match involution.retained_weyl_action() {
            Some(action) => action.clone(),
            // A table record's dropped Weyl factor: w = theta after delta
            // (delta is an involution), recovered exactly on both lattices.
            None => WeylAction::from_theta_factor(
                involution.root_involution().involution(),
                self.distinguished_involution.involution(),
            )?,
        };
        let positive_root_count = self.roots.roots().len() / 2;
        // Phase one decreases a lexicographic pair whose coordinates each
        // lie in 0..=positive_root_count. Phase three decreases involution
        // length and therefore takes at most positive_root_count steps.
        let phase_one_cap = positive_root_count
            .checked_add(1)
            .and_then(|bound| bound.checked_mul(bound))
            .ok_or(StructureError::ArithmeticOverflow)?;
        let phase_three_cap = positive_root_count;
        // The quadratic cap detects a broken termination invariant; it is not
        // a realistic output-size estimate. Keep eager allocation linear and
        // grow fallibly if a valid word exceeds it.
        let word_capacity = positive_root_count
            .checked_mul(2)
            .ok_or(StructureError::ArithmeticOverflow)?;
        let mut word = try_capacity(word_capacity)?;

        self.make_root_sums_dominant(
            &mut action,
            &twist,
            &mut real_sum,
            &mut imaginary_sum,
            &mut word,
            phase_one_cap,
            active,
        )?;
        let residual_generators = self.residual_generators(&real_sum, &imaginary_sum, active)?;
        self.make_residual_action_positive(
            &mut action,
            &twist,
            &residual_generators,
            &mut word,
            phase_three_cap,
        )?;

        let canonical = TwistedInvolution::new(
            &self.datum,
            &self.roots,
            self.distinguished_involution.involution(),
            action,
        )?;
        Ok((canonical, word))
    }

    // Phase one: lexicographically make the real sum dominant, then the
    // imaginary sum on the walls of the real sum. Only the generators
    // flagged in `active` participate (innerclass.cpp:770-795).
    #[allow(clippy::too_many_arguments)]
    fn make_root_sums_dominant(
        &self,
        action: &mut WeylAction,
        twist: &[usize],
        real_sum: &mut Weight,
        imaginary_sum: &mut Weight,
        word: &mut Vec<usize>,
        max_steps: usize,
        active: &[bool],
    ) -> Result<(), StructureError> {
        let mut steps = 0_usize;
        loop {
            let mut changed = false;
            for generator in 0..self.datum.semisimple_rank() {
                if !active
                    .get(generator)
                    .copied()
                    .ok_or(StructureError::IndexOutOfRange {
                        index: generator,
                        upper_bound: active.len(),
                    })?
                {
                    continue;
                }
                let real_pairing = pair(real_sum, &self.datum.simple_coroots()[generator])?;
                let should_reflect = real_pairing < 0
                    || (real_pairing == 0
                        && pair(imaginary_sum, &self.datum.simple_coroots()[generator])? < 0);
                if should_reflect {
                    if steps == max_steps {
                        return Err(StructureError::CartanClassificationInvariantViolation {
                            invariant: "canonicalize phase-one termination",
                        });
                    }
                    let next_real = self.datum.reflect_weight(generator, real_sum)?;
                    let next_imaginary = self.datum.reflect_weight(generator, imaginary_sum)?;
                    let next_action = self.twisted_conjugate_action(action, generator, twist)?;
                    *real_sum = next_real;
                    *imaginary_sum = next_imaginary;
                    *action = next_action;
                    word.try_reserve(1)
                        .map_err(|_| StructureError::AllocationFailed { requested: 1 })?;
                    word.push(generator);
                    steps += 1;
                    changed = true;
                    break;
                }
            }
            if !changed {
                break;
            }
        }
        Ok(())
    }

    // Phase two: retain exactly the simple generators orthogonal to both
    // dominant sums, intersected with `active`. They generate the residual
    // complex subsystem (innerclass.cpp:798-803).
    fn residual_generators(
        &self,
        real_sum: &Weight,
        imaginary_sum: &Weight,
        active: &[bool],
    ) -> Result<Vec<bool>, StructureError> {
        let mut residual_generators = try_capacity(self.datum.semisimple_rank())?;
        for generator in 0..self.datum.semisimple_rank() {
            let real_pairing = pair(real_sum, &self.datum.simple_coroots()[generator])?;
            let kept = real_pairing <= 0
                && pair(imaginary_sum, &self.datum.simple_coroots()[generator])? <= 0
                && active
                    .get(generator)
                    .copied()
                    .ok_or(StructureError::IndexOutOfRange {
                        index: generator,
                        upper_bound: active.len(),
                    })?;
            residual_generators.push(kept);
        }
        Ok(residual_generators)
    }

    // Phase three: eliminate negative simple-root images in the residual
    // subsystem, restarting the ascending scan after every conjugation.
    fn make_residual_action_positive(
        &self,
        action: &mut WeylAction,
        twist: &[usize],
        residual_generators: &[bool],
        word: &mut Vec<usize>,
        max_steps: usize,
    ) -> Result<(), StructureError> {
        let mut steps = 0_usize;
        loop {
            let mut changed = false;
            for (generator, &active) in residual_generators.iter().enumerate() {
                if !active {
                    continue;
                }
                let twisted_generator =
                    *twist
                        .get(generator)
                        .ok_or(StructureError::IndexOutOfRange {
                            index: generator,
                            upper_bound: twist.len(),
                        })?;
                let twisted_simple = *self.roots.simple_root_ids().get(twisted_generator).ok_or(
                    StructureError::IndexOutOfRange {
                        index: twisted_generator,
                        upper_bound: self.roots.simple_root_ids().len(),
                    },
                )?;
                let root = self
                    .roots
                    .root(twisted_simple)
                    .ok_or(StructureError::InvalidRootAutomorphism)?;
                let image = action.act(root)?;
                let image = self
                    .roots
                    .id_of(&image)
                    .ok_or(StructureError::InvalidRootAutomorphism)?;
                let is_positive = self
                    .roots
                    .is_positive(image)
                    .ok_or(StructureError::InvalidRootAutomorphism)?;
                if !is_positive {
                    if steps == max_steps {
                        return Err(StructureError::CartanClassificationInvariantViolation {
                            invariant: "canonicalize phase-three termination",
                        });
                    }
                    *action = self.twisted_conjugate_action(action, generator, twist)?;
                    word.try_reserve(1)
                        .map_err(|_| StructureError::AllocationFailed { requested: 1 })?;
                    word.push(generator);
                    steps += 1;
                    changed = true;
                    break;
                }
            }
            if !changed {
                break;
            }
        }
        Ok(())
    }

    fn validate_twisted_involution(
        &self,
        involution: &TwistedInvolution,
    ) -> Result<(), StructureError> {
        if involution.root_involution().involution().datum() != &self.datum {
            return Err(StructureError::DatumMismatch);
        }
        // Table records drop the Weyl factor's matrices; their construction
        // gate is push_record's compact-element/theta consistency check.
        // Everything else still carries the action and gets the full
        // recomposition check.
        let Some(action) = involution.retained_weyl_action() else {
            return Ok(());
        };
        if action.datum() != &self.datum {
            return Err(StructureError::DatumMismatch);
        }
        let distinguished = self.distinguished_involution.involution();
        let stored = involution.root_involution().involution();
        if compose_matrices(action.matrix(), distinguished.weight_matrix())?
            != stored.weight_matrix()
            || compose_matrices(action.coweight_matrix(), &distinguished.coweight_matrix().to_vec())?
                != stored.coweight_matrix()
        {
            return Err(StructureError::DistinguishedInvolutionMismatch);
        }
        Ok(())
    }

    fn twisted_conjugate_action(
        &self,
        action: &WeylAction,
        generator: usize,
        twist: &[usize],
    ) -> Result<WeylAction, StructureError> {
        let twisted_generator = *twist
            .get(generator)
            .ok_or(StructureError::IndexOutOfRange {
                index: generator,
                upper_bound: twist.len(),
            })?;
        let left = WeylAction::simple_reflection(&self.datum, generator)?;
        let right = WeylAction::simple_reflection(&self.datum, twisted_generator)?;
        left.compose(action)?.compose(&right)
    }

    /// Port of upstream `TwistedWeylGroup::canonical_involution_expr`
    /// (weyl.cpp:1359-1385): the reduced twisted-involution expression of a
    /// twisted involution's Weyl part, lexicographically least in the
    /// EXTERNAL generator numbering, one signed entry per step — a plain
    /// entry `s` is a cross (left multiplication by `s`), a
    /// bitwise-complemented entry `!s` is twisted conjugation by `s`
    /// (upstream packs both into one `int`; prettyprint.cpp:219-232 decodes
    /// the same way).
    ///
    /// PRECONDITION, the caller's contract exactly as upstream: `weyl` is
    /// the Weyl part of a twisted involution of THIS inner class — the
    /// loop's termination relies on it (each step drops the twisted
    /// length).
    pub fn canonical_involution_expr(
        &self,
        weyl: &WeylElement,
    ) -> Result<Vec<i32>, StructureError> {
        let twist = self.generator_twist()?;
        let mut result = try_capacity(weyl.length())?;
        let mut current = weyl.clone();
        while !current.is_identity() {
            // The first descent, in ascending generator order (upstream's
            // external-least election, NOT the internal renumbering).
            let mut generator = 0;
            while !current.has_left_descent(&self.roots, generator)? {
                generator += 1;
            }
            // hasTwistedCommutation (weyl.cpp:1296-1312): right-multiply by
            // the TWISTED generator, then compare the length change against
            // the product's own left descent.
            let (transported, change) =
                current.right_multiply_simple(&self.roots, twist[generator])?;
            let signed =
                i32::try_from(generator).map_err(|_| StructureError::ArithmeticOverflow)?;
            if (change > 0) == transported.has_left_descent(&self.roots, generator)? {
                result.push(signed);
                current = current.left_multiply_simple(&self.roots, generator)?.0;
            } else {
                result.push(!signed);
                current = current.twisted_conjugate(&self.roots, generator, &twist)?;
            }
        }
        Ok(result)
    }

    /// Enumerate root involutions of the form `w after distinguished`.
    ///
    /// This is a stable list of twisted involutions, not yet the quotient into
    /// Cartan classes by twisted conjugacy or Cayley transforms. The list is
    /// generated by classwise cross-action closure (never by a full Weyl
    /// enumeration, which is unaffordable for E8: |W| = 696,729,600 while the
    /// twisted involutions number only 199,952).
    pub fn twisted_involutions(
        &self,
        weyl_budget: usize,
    ) -> Result<Vec<TwistedInvolution>, StructureError> {
        let mut involutions = Vec::new();
        let mut consume = |orbit: ClassOrbit| -> Result<(), StructureError> {
            involutions
                .try_reserve(orbit.member_count())
                .map_err(|_| StructureError::AllocationFailed {
                    requested: orbit.member_count(),
                })?;
            involutions.extend(orbit.materialize(self)?);
            Ok(())
        };
        let mut emit = |_: usize, _: &[u8], _: Option<u128>| -> Result<(), StructureError> {
            Ok(())
        };
        if self.datum.semisimple_rank() <= 8 {
            self.involution_orbits::<u64, _>(weyl_budget, &mut emit, &mut consume)?;
        } else {
            self.involution_orbits::<u128, _>(weyl_budget, &mut emit, &mut consume)?;
        }
        Ok(involutions)
    }

    /// Deterministic Weyl twisted-conjugacy orbits of twisted involutions.
    ///
    /// Each class's representative is the canonical twisted involution
    /// produced by [`Self::canonicalize`] during the Cayley BFS. This
    /// operation does not yet construct Cartan fibers, real forms, or the
    /// Cartan partial order.
    pub fn twisted_conjugacy_classes(
        &self,
        weyl_budget: usize,
    ) -> Result<Vec<TwistedConjugacyClass>, StructureError> {
        Ok(self
            .twisted_conjugacy_partition(weyl_budget)?
            .classes()
            .to_vec())
    }

    /// The full twisted-conjugacy partition with a membership lookup.
    ///
    /// This is the single orbit implementation;
    /// [`Self::twisted_conjugacy_classes`] is a thin wrapper over it.
    pub fn twisted_conjugacy_partition(
        &self,
        weyl_budget: usize,
    ) -> Result<TwistedConjugacyPartition, StructureError> {
        let simple_positions: Vec<u8> = self
            .roots
            .simple_root_ids()
            .iter()
            .map(|id| id.0 as u8)
            .collect();
        // Rank <= 12: the simple-root-image key needs at most 96 bits, so the
        // owning class index packs into the low 32 bits of the same u128 and
        // the membership index is a single flat sorted vector — 16 bytes per
        // member (E8: 199,952 members per side) instead of ~69 bytes of
        // hash-table entry overhead. Streamed assembly: each member is
        // indexed as the orbit closure discovers it, so no per-class flat
        // permutation buffer is retained (the E8 transient was ~48MB of
        // buffers plus Vec-doubling overshoot).
        let packed = simple_positions.len() <= 12;
        let mut classes = Vec::new();
        let mut entries: Vec<u128> = Vec::new();
        let mut fallback = PermutationKeyMap::default();
        // Set when the parallel driver left `entries` fully sorted (it merges
        // the workers' individually sorted runs), so the final sort is a
        // redundant no-op pass to skip.
        let mut entries_sorted = false;
        // Heavy packed inner classes (E7: 126 roots, E8: 240) run phase two
        // on two worker threads; everything else keeps the sequential
        // driver (thread setup is not worth it below ~100 roots). Keys fit
        // a u64 slot at semisimple rank <= 8 (E7/E8 included), halving the
        // per-worker membership table.
        if packed && self.roots.roots().len() >= 100 {
            if simple_positions.len() <= 8 {
                self.involution_orbits_parallel::<u64>(
                    weyl_budget,
                    &simple_positions,
                    &mut entries,
                    &mut classes,
                )?;
            } else {
                self.involution_orbits_parallel::<u128>(
                    weyl_budget,
                    &simple_positions,
                    &mut entries,
                    &mut classes,
                )?;
            }
            entries_sorted = true;
        } else {
            // The closure hands back each member's packed simple-image key
            // (computed for the membership probe anyway), so indexing a
            // member here is a shift/or and a push — no re-gather.
            let mut emit = |class_index: usize,
                            permutation: &[u8],
                            key: Option<u128>|
             -> Result<(), StructureError> {
                if packed {
                    let owner = u32::try_from(class_index)
                        .map_err(|_| StructureError::ArithmeticOverflow)?;
                    entries
                        .try_reserve(1)
                        .map_err(|_| StructureError::AllocationFailed { requested: 1 })?;
                    let key = key
                        .unwrap_or_else(|| pack_simple_images(permutation, &simple_positions));
                    entries.push((key << 32) | u128::from(owner));
                } else {
                    fallback.insert(
                        match key {
                            Some(key) => PermutationKey::Packed(key),
                            None => PermutationKey::pack(permutation, &simple_positions),
                        },
                        class_index,
                    );
                }
                Ok(())
            };
            let mut consume = |orbit: ClassOrbit| -> Result<(), StructureError> {
                let orbit_member_count = orbit.member_count();
                classes.push(TwistedConjugacyClass::new(
                    orbit.representative,
                    orbit_member_count,
                ));
                Ok(())
            };
            if simple_positions.len() <= 8 {
                self.involution_orbits::<u64, _>(weyl_budget, &mut emit, &mut consume)?;
            } else {
                self.involution_orbits::<u128, _>(weyl_budget, &mut emit, &mut consume)?;
            }
        }
        let membership = if packed {
            // Keys are unique (the packing is injective), so ordering by the
            // composite entry orders by key alone. The partition is retained
            // in the classification cache for the session's lifetime, so
            // drop the sequential driver's growth-doubling slack (the
            // parallel driver already reserve_exacts).
            if !entries_sorted {
                entries.sort_unstable();
            }
            entries.shrink_to_fit();
            ClassMembership::Packed { entries }
        } else {
            ClassMembership::Full(fallback)
        };
        Ok(TwistedConjugacyPartition::new(
            self.datum.clone(),
            self.distinguished_involution.clone(),
            classes,
            simple_positions,
            membership,
        ))
    }

    /// The permutation-level orbit machinery over this inner class.
    pub(crate) fn permutation_orbits(&self) -> Result<PermutationOrbits<'_>, StructureError> {
        PermutationOrbits::new(self)
    }

    /// The twisted-conjugacy classes, each as (canonical representative, full
    /// membership), generated WITHOUT enumerating the Weyl group.
    ///
    /// Phase one reproduces innerclass.cpp:218-291 (task 1): BFS from the
    /// identity twisted involution, Cayley-transforming each representative
    /// by its positive imaginary roots (in upstream RootNbr order) and
    /// canonicalizing the successors; deduplication is by the canonical
    /// representative's root-image permutation, so no membership oracle is
    /// needed. Phase two fills each class exactly like upstream's
    /// `Cartan_orbit` constructor (involutions.cpp:362-379): closure under
    /// the cross actions `w |-> s w twist(s)` for every simple generator `s`.
    /// `weyl_budget` bounds the TOTAL number of twisted involutions
    /// (upstream's InvolutionTable size), not the Weyl group order — this is
    /// what keeps the E8 inner class affordable (199,952 twisted involutions
    /// out of |W| = 696,729,600).
    ///
    /// Both phases run at the root-image-permutation level through
    /// [`PermutationOrbits`]; the owning [`TwistedInvolution`] (matrix
    /// provenance, full [`RootInvolutionData`] validation) is rebuilt only
    /// for freshly discovered class representatives, so the E8 construction
    /// pays the matrix cost once per CLASS instead of once per Cayley edge.
    ///
    /// Each phase-two orbit is streamed: every member's permutation is handed
    /// to `emit` (with the orbit's index, which equals its class index) in
    /// BFS discovery order as it is found, and the completed orbit is handed
    /// to `consume` as soon as its cross closure finishes. No per-class flat
    /// member buffer survives its own closure — accumulating every
    /// [`ClassOrbit`] buffer to end-of-build pinned ~48MB of E8 transients
    /// (plus Vec-doubling overshoot) against the process peak.
    fn involution_orbits<T, E>(
        &self,
        weyl_budget: usize,
        emit: &mut E,
        consume: &mut dyn FnMut(ClassOrbit) -> Result<(), StructureError>,
    ) -> Result<(), StructureError>
    where
        T: PackedSlot,
        E: FnMut(usize, &[u8], Option<u128>) -> Result<(), StructureError>,
    {
        let mut orbit_machine = PermutationOrbits::new(self)?;
        let (representatives, representative_permutations) =
            self.cayley_bfs_representatives(&mut orbit_machine)?;
        let rank = self.datum.semisimple_rank();

        // Phase two: per-class cross-action closure, AT THE PERMUTATION
        // LEVEL. For theta_w = w after distinguished, the cross action
        // w |-> s w twist(s) induces theta |-> r_s theta r_s (the twist cancels
        // against delta), a plain conjugation by the simple root reflection.
        // Materializing a TwistedInvolution per BFS EDGE would dominate the
        // runtime (E8: ~1.6M edges against 199,952 members), so members are
        // streamed to `emit` as root-image permutations plus retained parent
        // links; matrices are rebuilt only on demand (ClassOrbit::materialize,
        // for the test-only twisted_involutions listing).
        let mut total = 0_usize;
        let mut seen = PermutationKeySet::default();
        let mut packed_seen = PackedKeySet::<T>::new();
        let mut next: Vec<u8> = Vec::new();
        let packed = orbit_machine.simple_positions.len() <= 16;
        let stride = self.roots.roots().len();
        for (orbit_index, (representative, representative_permutation)) in representatives
            .into_iter()
            .zip(representative_permutations)
            .enumerate()
        {
            let orbit = Self::orbit_cross_closure(
                &orbit_machine,
                orbit_index,
                representative,
                representative_permutation,
                rank,
                stride,
                packed,
                &mut seen,
                &mut packed_seen,
                &mut next,
                emit,
            )?;
            total = total
                .checked_add(orbit.member_count())
                .ok_or(StructureError::ArithmeticOverflow)?;
            if total > weyl_budget {
                return Err(StructureError::ResourceLimitExceeded { limit: weyl_budget });
            }
            consume(orbit)?;
        }
        Ok(())
    }

    /// Phase one of [`Self::involution_orbits`], factored so the parallel
    /// phase-two driver can share it: canonical class representatives by
    /// Cayley BFS.
    ///
    /// The representative matrices are replayed from the recorded
    /// canonicalizing word only when the canonical permutation is new, so
    /// discovery order and representatives match the historic matrix-level
    /// loop exactly.
    fn cayley_bfs_representatives(
        &self,
        orbit_machine: &mut PermutationOrbits,
    ) -> Result<(Vec<TwistedInvolution>, Vec<Vec<u8>>), StructureError> {
        let delta = self.distinguished_involution.involution();
        let rank = self.datum.semisimple_rank();
        let identity = TwistedInvolution::new(
            &self.datum,
            &self.roots,
            delta,
            WeylAction::identity(&self.datum)?,
        )?;
        let active = vec![true; rank];
        let mut representatives: Vec<TwistedInvolution> = vec![identity];
        let mut representative_permutations = vec![involution_key(&representatives[0])];
        let mut seen_canonical = PermutationKeySet::default();
        seen_canonical.insert(PermutationKey::pack(
            &representative_permutations[0],
            orbit_machine.simple_positions(),
        ));
        let mut cursor = 0_usize;
        while cursor < representatives.len() {
            let permutation = representative_permutations[cursor].clone();
            // Positive imaginary roots in upstream RootNbr order. Imaginary
            // means fixed by the involution's root permutation.
            let mut imaginary = Vec::new();
            for (index, &image) in permutation.iter().enumerate() {
                if usize::from(image) != index {
                    continue;
                }
                let root = RootId(index);
                let coordinates = self.roots.simple_coordinates(root).ok_or(
                    StructureError::IndexOutOfRange {
                        index,
                        upper_bound: self.roots.roots().len(),
                    },
                )?;
                if coordinates.iter().all(|&coordinate| coordinate >= 0) {
                    imaginary.push((
                        crate::cartan_classification::upstream_positive_key(coordinates)?,
                        root,
                    ));
                }
            }
            imaginary.sort_by(|(left, _), (right, _)| left.cmp(right));
            for (_, root) in imaginary {
                let successor = orbit_machine.cayley_successor(&permutation, root)?;
                let (canonical_permutation, word) =
                    orbit_machine.canonicalize(&successor, &active)?;
                if seen_canonical.insert(PermutationKey::pack(
                    &canonical_permutation,
                    orbit_machine.simple_positions(),
                )) {
                    let representative = self.cayley_representative(
                        &representatives[cursor],
                        root,
                        &word,
                        &orbit_machine.twist,
                    )?;
                    debug_assert_eq!(
                        involution_key(&representative),
                        canonical_permutation,
                        "permutation-level canonicalize diverges from the matrix replay"
                    );
                    representatives.push(representative);
                    representative_permutations.push(canonical_permutation);
                }
            }
            cursor += 1;
        }
        Ok((representatives, representative_permutations))
    }

    /// Phase two of [`Self::involution_orbits`] on two worker threads, for
    /// heavy packed inner classes (E7/E8: the closure is ~85% of the
    /// small-script fixed cost, and the primal/dual classifications already
    /// run on two threads in `build_inner_class_context`, leaving two cores
    /// of the four-CPU corpus allocation idle).
    ///
    /// Each class's cross closure is independent — per-orbit membership set,
    /// chunk stream, and emitted keys — so workers pull orbit indices from
    /// an atomic cursor and write per-orbit result slots and per-thread
    /// entry buffers. Determinism is preserved by REPLAYING in orbit index
    /// order afterwards: the budget check and the class vector fill run in
    /// the same order as the sequential loop, the first error in orbit
    /// order wins (as sequential timing guarantees), and the packed
    /// membership entries are order-free because
    /// [`crate::TwistedConjugacyPartition`] sorts them. Each worker keeps its
    /// own scratch (`PackedKeySet`, chunk buffers), cleared per orbit exactly
    /// as the sequential driver does.
    fn involution_orbits_parallel<T: PackedSlot>(
        &self,
        weyl_budget: usize,
        simple_positions: &[u8],
        entries: &mut Vec<u128>,
        classes: &mut Vec<TwistedConjugacyClass>,
    ) -> Result<(), StructureError> {
        use std::sync::Mutex;
        use std::sync::atomic::{AtomicUsize, Ordering};

        let rank = self.datum.semisimple_rank();
        let stride = self.roots.roots().len();
        let mut orbit_machine = PermutationOrbits::new(self)?;
        let (representatives, representative_permutations) =
            self.cayley_bfs_representatives(&mut orbit_machine)?;
        let orbit_count = representatives.len();
        let work = Mutex::new(
            representatives
                .into_iter()
                .zip(representative_permutations)
                .map(Some)
                .collect::<Vec<_>>(),
        );
        let results: Vec<Mutex<Option<Result<ClassOrbit, StructureError>>>> =
            (0..orbit_count).map(|_| Mutex::new(None)).collect();
        let next_orbit = AtomicUsize::new(0);
        let orbit_machine = &orbit_machine;
        let thread_entries = std::thread::scope(|scope| {
            let mut handles = Vec::new();
            for _ in 0..2 {
                let work = &work;
                let results = &results;
                let next_orbit = &next_orbit;
                handles.push(scope.spawn(move || {
                    let mut seen = PermutationKeySet::default();
                    let mut packed_seen = PackedKeySet::<T>::new();
                    let mut next: Vec<u8> = Vec::new();
                    let mut local_entries: Vec<u128> = Vec::new();
                    loop {
                        let orbit_index = next_orbit.fetch_add(1, Ordering::Relaxed);
                        if orbit_index >= orbit_count {
                            break;
                        }
                        let (representative, representative_permutation) = work.lock()
                            .expect("orbit work lock poisoned")[orbit_index]
                            .take()
                            .expect("orbit work item taken twice");
                        let mut emit = |orbit_index: usize,
                                        permutation: &[u8],
                                        key: Option<u128>|
                         -> Result<(), StructureError> {
                            let owner = u32::try_from(orbit_index)
                                .map_err(|_| StructureError::ArithmeticOverflow)?;
                            let key = key.unwrap_or_else(|| {
                                pack_simple_images(permutation, simple_positions)
                            });
                            push_slim(&mut local_entries, (key << 32) | u128::from(owner))
                        };
                        let orbit = Self::orbit_cross_closure(
                            orbit_machine,
                            orbit_index,
                            representative,
                            representative_permutation,
                            rank,
                            stride,
                            true,
                            &mut seen,
                            &mut packed_seen,
                            &mut next,
                            &mut emit,
                        );
                        *results[orbit_index]
                            .lock()
                            .expect("orbit result lock poisoned") = Some(orbit);
                    }
                    // Sort in the worker (parallel) and drop the
                    // growth-doubling slack before the replay concatenation,
                    // so the peak no longer holds a third exact-size buffer
                    // next to both per-worker buffers.
                    local_entries.sort_unstable();
                    local_entries.shrink_to_fit();
                    local_entries
                }));
            }
            handles
                .into_iter()
                .map(|handle| handle.join().expect("orbit closure worker panicked"))
                .collect::<Vec<_>>()
        });
        // Replay in orbit index order: identical class vector, budget check,
        // and error precedence to the sequential driver.
        let mut total = 0_usize;
        for slot in &results {
            let orbit = slot
                .lock()
                .expect("orbit result lock poisoned")
                .take()
                .ok_or(StructureError::CartanClassificationInvariantViolation {
                    invariant: "orbit worker result",
                })??;
            total = total
                .checked_add(orbit.member_count())
                .ok_or(StructureError::ArithmeticOverflow)?;
            if total > weyl_budget {
                return Err(StructureError::ResourceLimitExceeded { limit: weyl_budget });
            }
            let orbit_member_count = orbit.member_count();
            classes.push(TwistedConjugacyClass::new(
                orbit.representative,
                orbit_member_count,
            ));
        }
        // Merge the two (individually sorted, exactly sized) worker buffers
        // into one sorted vector with a single O(n) two-way pass: keys are
        // unique (the packing is injective), so the merged order equals the
        // sorted concatenation exactly, and the caller skips its final
        // sort_unstable. Each worker buffer is dropped as soon as it drains.
        let entry_total = thread_entries.iter().try_fold(0_usize, |sum, local| {
            sum.checked_add(local.len())
                .ok_or(StructureError::ArithmeticOverflow)
        })?;
        entries
            .try_reserve_exact(entry_total)
            .map_err(|_| StructureError::AllocationFailed {
                requested: entry_total,
            })?;
        let mut locals = thread_entries.into_iter();
        match (locals.next(), locals.next()) {
            (Some(left), Some(right)) => {
                let (mut i, mut j) = (0_usize, 0_usize);
                // Within the exact reserve, pushes never reallocate.
                while i < left.len() && j < right.len() {
                    if left[i] <= right[j] {
                        entries.push(left[i]);
                        i += 1;
                    } else {
                        entries.push(right[j]);
                        j += 1;
                    }
                }
                entries.extend_from_slice(&left[i..]);
                entries.extend_from_slice(&right[j..]);
            }
            (Some(mut local), None) => entries.append(&mut local),
            (None, _) => {}
        }
        Ok(())
    }

    /// Profiling-visible extraction of the phase-two per-class BFS.
    ///
    /// Member permutations live in fixed-size chunks; a chunk is released as
    /// soon as the sequential BFS cursor advances past it, so the resident
    /// footprint is the frontier width rather than the class size (the E8
    /// 113,400-member class otherwise held a 27MB flat buffer to end-of-build
    /// and spiked to ~47MB at its final doubling). Every member is handed to
    /// `emit` in BFS discovery order (representative first) as it is found,
    /// together with its packed simple-image key when one exists (semisimple
    /// rank <= 16): the probe key of a successor IS that successor's own
    /// packed key (`next[p] = r(c(r(p)))` byte-for-byte), so the membership
    /// indexer reuses it instead of re-gathering from the permutation.
    /// `emit` is generic (not `dyn`) so the membership indexer inlines into
    /// the BFS loop: for E8 that is ~200k indirect calls plus a key
    /// re-gather that now disappear into the edge pipeline.
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn orbit_cross_closure<T: PackedSlot, E: FnMut(usize, &[u8], Option<u128>) -> Result<(), StructureError>>(
        orbit_machine: &PermutationOrbits,
        orbit_index: usize,
        representative: TwistedInvolution,
        representative_permutation: Vec<u8>,
        rank: usize,
        stride: usize,
        packed: bool,
        seen: &mut PermutationKeySet,
        packed_seen: &mut PackedKeySet<T>,
        next: &mut Vec<u8>,
        emit: &mut E,
    ) -> Result<ClassOrbit, StructureError> {
        // Chunk target ~256KB of member bytes: the resident footprint is the
        // BFS frontier width plus up to two chunks, so smaller chunks halve
        // the granularity overshoot (E8 stride 240: 262KB vs the historic
        // 1MB chunk pair at peak). Small strides keep the historic 4096
        // members (their chunks are already far below 256KB); the floor
        // bounds alloc churn for huge strides. The member count is rounded
        // DOWN to a power of two, so the per-member chunk locate is a
        // shift/mask instead of an integer division.
        let chunk_members = if stride == 0 {
            4096
        } else {
            let target = (262_144 / stride).clamp(64, 4096);
            1_usize << (usize::BITS - 1 - target.leading_zeros())
        };
        let chunk_shift = chunk_members.trailing_zeros();
        let chunk_mask = chunk_members - 1;
        let chunk_bytes = chunk_members
            .checked_mul(stride)
            .ok_or(StructureError::ArithmeticOverflow)?;

        // The representative's permutation seeds the first chunk.
        let mut chunks: VecDeque<Vec<u8>> = VecDeque::new();
        let mut first = try_capacity(chunk_bytes)?;
        first.extend_from_slice(&representative_permutation);
        chunks.push_back(first);
        // `base` = the number of members in already-released chunks; member
        // `i` lives in `chunks[(i - base) / chunk_members]`.
        let mut base = 0_usize;
        let mut parents: Vec<(u32, u8)> = vec![(u32::MAX, 0)];
        let mut member_count = 1_usize;
        // Padded fast path (packed keys and a root count that fits the u8
        // encoding — every rank <= 16 case in practice): the per-member
        // permutation and the simple-reflection tables are staged in
        // fixed-size [u8; 256] buffers, so every gather inside the probe and
        // the successor compose indexes a 256-entry array with a u8 and the
        // compiler emits NO bounds checks, and membership probes go to a
        // lean open-addressing u128 table instead of the general hash set.
        // Three further probe-level cuts: the member's slice is located
        // once per member (not per edge); edges whose conjugated key equals
        // the member's own key are skipped before the membership probe —
        // `r_s c r_s == c` iff the simple images agree (the packing is
        // injective), and every member's key is already in the set at
        // discovery time, so the skipped `insert` was a guaranteed no-op
        // duplicate probe; and the edge back to a member's BFS parent is
        // skipped outright, because the cross action by one generator is an
        // involution, so probing the child with the parent's generator
        // reproduces the parent — another guaranteed no-op probe.
        let padded = packed && stride <= 256;
        // `seed_key` is the representative's packed simple-image key, handed
        // to `emit` so the membership indexer does not re-gather it.
        let seed_key: Option<u128>;
        if padded {
            packed_seen.clear();
            let mut key = T::ZERO;
            for (shift, &position) in orbit_machine.simple_positions.iter().enumerate() {
                key = key.or_byte(representative_permutation[usize::from(position)], shift);
            }
            packed_seen.insert(key);
            seed_key = Some(key.to_u128());
        } else {
            seen.clear();
            let seed = PermutationKey::pack(
                &representative_permutation,
                orbit_machine.simple_positions(),
            );
            seed_key = match &seed {
                PermutationKey::Packed(key) => Some(*key),
                PermutationKey::Full(_) => None,
            };
            seen.insert(seed);
        }
        emit(orbit_index, &representative_permutation, seed_key)?;
        let mut padded_reflections: Vec<[u8; 256]> = Vec::new();
        // `inner_images[g][p] = r_g(alpha_p)`: the inner gather of the probe
        // key hoisted out of the edge loop, so probing one edge costs two
        // gathers per simple position instead of three.
        let mut inner_images: Vec<[u8; 16]> = Vec::new();
        if padded {
            padded_reflections = try_capacity(rank)?;
            for reflection in &orbit_machine.simple_reflections {
                let mut table = [0_u8; 256];
                table[..reflection.len()].copy_from_slice(reflection);
                padded_reflections.push(table);
            }
            inner_images = try_capacity(rank)?;
            for reflection in &padded_reflections {
                let mut inner = [0_u8; 16];
                for (slot, &position) in orbit_machine.simple_positions.iter().enumerate() {
                    inner[slot] = reflection[usize::from(position)];
                }
                inner_images.push(inner);
            }
        }
        let mut current_buf = [0_u8; 256];
        let mut next_buf = [0_u8; 256];
        let simple_count = orbit_machine.simple_positions.len();

        let mut cursor = 0_usize;
        while cursor < member_count {
            // Release chunks the sequential cursor has fully read.
            while cursor - base >= chunk_members {
                chunks.pop_front();
                base += chunk_members;
            }
            if padded {
                let chunk = &chunks[(cursor - base) >> chunk_shift];
                let offset = ((cursor - base) & chunk_mask) * stride;
                current_buf[..stride].copy_from_slice(&chunk[offset..offset + stride]);
                let mut current_key = T::ZERO;
                for (shift, &position) in orbit_machine.simple_positions.iter().enumerate() {
                    current_key = current_key.or_byte(current_buf[usize::from(position)], shift);
                }
                let (parent_member, parent_gen) = parents[cursor];
                let parent_generator = if parent_member == u32::MAX {
                    usize::MAX
                } else {
                    usize::from(parent_gen)
                };
                // next = reflection after current after reflection. Probe
                // on the simple-root images alone (an injective key), so
                // the full successor permutation is computed only for
                // genuine new members — about one edge in eight for E8.
                // The zip walks both per-generator tables without bounds
                // checks; the enumerate index doubles as the parent-edge
                // skip key.
                for (generator, (reflection, inner)) in padded_reflections
                    .iter()
                    .zip(inner_images.iter())
                    .enumerate()
                {
                    if generator == parent_generator {
                        // Cross action by one generator is an involution:
                        // this edge lands back on the BFS parent, already in
                        // the set, so the probe would be a no-op duplicate.
                        continue;
                    }
                    let mut key = T::ZERO;
                    for (shift, &first) in inner[..simple_count].iter().enumerate() {
                        let image = reflection[usize::from(current_buf[usize::from(first)])];
                        key = key.or_byte(image, shift);
                    }
                    // Self-loop (`r_s c r_s == c`) or an already-seen member:
                    // both are no-op inserts, skipped without touching the
                    // membership set in the self-loop case.
                    if key == current_key || !packed_seen.insert(key) {
                        continue;
                    }
                    for slot in 0..stride {
                        next_buf[slot] = reflection
                            [usize::from(current_buf[usize::from(reflection[slot])])];
                    }
                    let open = chunks.back_mut().ok_or(
                        StructureError::CartanClassificationInvariantViolation {
                            invariant: "orbit chunk",
                        },
                    )?;
                    if open.len() == chunk_bytes {
                        chunks.push_back(try_capacity(chunk_bytes)?);
                    }
                    let open = chunks.back_mut().ok_or(
                        StructureError::CartanClassificationInvariantViolation {
                            invariant: "orbit chunk",
                        },
                    )?;
                    open.extend_from_slice(&next_buf[..stride]);
                    push_slim(&mut parents, (cursor as u32, generator as u8))?;
                    member_count += 1;
                    // The probe key IS the successor's packed simple-image
                    // key (`next_buf[p] = r(c(r(p)))` byte-for-byte), so the
                    // membership indexer reuses it instead of re-gathering.
                    emit(orbit_index, &next_buf[..stride], Some(key.to_u128()))?;
                }
            } else {
                for generator in 0..rank {
                    // next = reflection after current after reflection. Probe
                    // on the simple-root images alone (an injective key), so
                    // the full successor permutation is computed only for
                    // genuine new members.
                    let reflection = &orbit_machine.simple_reflections[generator];
                    let mut packed_key = None;
                    if packed {
                        let mut key = 0_u128;
                        {
                            let chunk = &chunks[(cursor - base) >> chunk_shift];
                            let offset = ((cursor - base) & chunk_mask) * stride;
                            let current = &chunk[offset..offset + stride];
                            for (shift, &position) in
                                orbit_machine.simple_positions.iter().enumerate()
                            {
                                let image = reflection
                                    [current[usize::from(reflection[usize::from(position)])]
                                    as usize];
                                key |= u128::from(image) << (8 * shift);
                            }
                        }
                        if !seen.insert(PermutationKey::Packed(key)) {
                            continue;
                        }
                        packed_key = Some(key);
                    }
                    next.clear();
                    next.try_reserve(reflection.len())
                        .map_err(|_| StructureError::AllocationFailed {
                            requested: reflection.len(),
                        })?;
                    {
                        let chunk = &chunks[(cursor - base) >> chunk_shift];
                        let offset = ((cursor - base) & chunk_mask) * stride;
                        let current = &chunk[offset..offset + stride];
                        for &image in reflection.iter() {
                            next.push(reflection[current[usize::from(image)] as usize]);
                        }
                    }
                    if packed || seen.insert(PermutationKey::Full(next.clone())) {
                        let open = chunks.back_mut().ok_or(
                            StructureError::CartanClassificationInvariantViolation {
                                invariant: "orbit chunk",
                            },
                        )?;
                        if open.len() == chunk_bytes {
                            chunks.push_back(try_capacity(chunk_bytes)?);
                        }
                        let open = chunks.back_mut().ok_or(
                            StructureError::CartanClassificationInvariantViolation {
                                invariant: "orbit chunk",
                            },
                        )?;
                        open.extend_from_slice(&next);
                        push_slim(&mut parents, (cursor as u32, generator as u8))?;
                        member_count += 1;
                        emit(orbit_index, &next, packed_key)?;
                    }
                }
            }
            cursor += 1;
        }
        // The parallel driver retains every ClassOrbit until replay, so drop
        // the growth-doubling slack before handing the orbit back.
        parents.shrink_to_fit();
        Ok(ClassOrbit {
            representative,
            parents,
        })
    }

    /// Rebuild the canonical representative discovered by a Cayley step: the
    /// successor action `r_root after parent` conjugated by the recorded
    /// canonicalizing word, then fully validated as a [`TwistedInvolution`].
    /// The word comes from the permutation-level canonicalization, whose
    /// decisions are the same pairings the matrix-level
    /// [`Self::canonicalize`] computes, so this is the representative the
    /// historic loop would have stored.
    fn cayley_representative(
        &self,
        parent: &TwistedInvolution,
        root: RootId,
        word: &[usize],
        twist: &[usize],
    ) -> Result<TwistedInvolution, StructureError> {
        let delta = self.distinguished_involution.involution();
        let reflection = WeylAction::root_reflection(&self.datum, &self.roots, root)?;
        let mut action = reflection.compose(parent.weyl_action())?;
        for &generator in word {
            action = self.twisted_conjugate_action(&action, generator, twist)?;
        }
        TwistedInvolution::new(&self.datum, &self.roots, delta, action).map_err(|error| {
            match error {
                StructureError::InvalidInvolution => {
                    StructureError::CartanClassificationInvariantViolation {
                        invariant: "Cayley successor",
                    }
                }
                other => other,
            }
        })
    }
}

/// The partition/lookup key of a twisted involution: its root-image
/// permutation (see [`TwistedConjugacyPartition`]).
pub(crate) fn involution_key(involution: &TwistedInvolution) -> Vec<u8> {
    involution
        .root_involution()
        .image_permutation()
        .iter()
        .map(|id| id.0 as u8)
        .collect()
}

/// FxHash-style hasher for root-image permutation keys.
///
/// The orbit structures hash 240-byte permutation keys hundreds of thousands
/// of times per inner class (E8: ~1.6M cross-action edges plus one partition
/// entry per member), so the default SipHasher would dominate the closure.
/// Collisions still compare the full key, so the hash choice never affects
/// semantics, only probe speed.
#[derive(Clone, Default)]
pub(crate) struct PermutationHasher(u64);

impl std::hash::Hasher for PermutationHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        const SEED: u64 = 0x51_7c_c1_b7_27_22_0a_95;
        let mut chunks = bytes.chunks_exact(8);
        for chunk in &mut chunks {
            let value = u64::from_le_bytes(chunk.try_into().unwrap_or([0; 8]));
            self.0 = (self.0.rotate_left(5) ^ value).wrapping_mul(SEED);
        }
        let remainder = chunks.remainder();
        if !remainder.is_empty() {
            let mut tail = 0_u64;
            for &byte in remainder {
                tail = (tail << 8) | u64::from(byte);
            }
            self.0 = (self.0.rotate_left(5) ^ tail).wrapping_mul(SEED);
        }
    }
}

pub(crate) type PermutationHasherBuilder = std::hash::BuildHasherDefault<PermutationHasher>;

/// Dedup/lookup key of a root-image permutation: the images of the SIMPLE
/// roots only.
///
/// A root-datum involution is a linear map of the root lattice, and the
/// simple roots form a Z-basis of that lattice, so the simple-root images
/// determine the map — and therefore the full root permutation — EXACTLY.
/// This is an injective key, not a digest: equality semantics are unchanged.
/// For semisimple rank <= 16 the key packs into a u128, which keeps the E8
/// cross-action closure's ~1.6M probes to an integer hash and compare
/// instead of a 240-byte key chase. Rank > 16 with <= 255 roots (the u8
/// encoding's own ceiling) needs at least 17 disjoint A1 factors; that case
/// falls back to the full permutation.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) enum PermutationKey {
    Packed(u128),
    Full(Vec<u8>),
}

impl PermutationKey {
    /// The key of a full root-image permutation, given the simple roots'
    /// positions in the permutation (generator order).
    pub(crate) fn pack(permutation: &[u8], simple_positions: &[u8]) -> Self {
        if simple_positions.len() <= 16 {
            Self::Packed(pack_simple_images(permutation, simple_positions))
        } else {
            Self::Full(permutation.to_vec())
        }
    }
}

/// The u128 packing of a permutation's simple-root images — the
/// [`PermutationKey::Packed`] payload, for consumers that store the packed
/// key directly. Caller guarantees `simple_positions.len() <= 16`.
pub(crate) fn pack_simple_images(permutation: &[u8], simple_positions: &[u8]) -> u128 {
    let mut packed = 0_u128;
    for (shift, &position) in simple_positions.iter().enumerate() {
        packed |= u128::from(permutation[usize::from(position)]) << (8 * shift);
    }
    packed
}

pub(crate) type PermutationKeySet =
    std::collections::HashSet<PermutationKey, PermutationHasherBuilder>;
pub(crate) type PermutationKeyMap<V> =
    std::collections::HashMap<PermutationKey, V, PermutationHasherBuilder>;

/// Push with a 1.5x exact-growth policy instead of `Vec::push`'s amortized
/// doubling.
///
/// The orbit-build buffers grow to a size that is unknowable during the
/// streamed BFS (E8: ~2MB of membership entries and ~1MB of parent links
/// per closure worker), so doubling's up-to-100% capacity slack sits
/// directly on the process peak — the peak snapshot lands mid-closure,
/// before any end-of-build shrink_to_fit can run. Exact 1.5x steps cap the
/// transient slack at 50% and cost O(n/(r-1)) = 3n element copies against
/// doubling's 2n, negligible next to the closure work. Buffers that know
/// their size up front still use exact pre-sizing; buffers that are small
/// or short-lived still use plain push.
fn push_slim<T>(vec: &mut Vec<T>, value: T) -> Result<(), StructureError> {
    if vec.len() == vec.capacity() {
        let grow = vec.len() / 2 + 1;
        vec.try_reserve_exact(grow)
            .map_err(|_| StructureError::AllocationFailed { requested: grow })?;
    }
    vec.push(value);
    Ok(())
}

/// Slot element of [`PackedKeySet`]: `u64` when the semisimple rank is at
/// most 8 (the packed simple-image key is at most 64 bits), `u128`
/// otherwise. The all-ones value is the empty sentinel: a valid packed key
/// can never equal it, because that would need every simple-root image byte
/// to be 0xFF, while a permutation's images are distinct (and rank >= 2 has
/// at least two simple roots; rank 1 keys carry zero bytes above the first).
pub(crate) trait PackedSlot: Copy + Eq + std::fmt::Debug {
    const EMPTY: Self;
    const ZERO: Self;
    /// `self | (image << 8*shift)` — one packed simple-root image byte.
    fn or_byte(self, image: u8, shift: usize) -> Self;
    fn hash(self) -> u64;
    /// The same packed key as a `u128` (zero-extended for narrow slots), for
    /// consumers like the membership index that store one width for all
    /// ranks.
    fn to_u128(self) -> u128;
}

impl PackedSlot for u64 {
    const EMPTY: Self = u64::MAX;
    const ZERO: Self = 0;

    #[inline]
    fn or_byte(self, image: u8, shift: usize) -> Self {
        self | u64::from(image) << (8 * shift)
    }

    #[inline]
    fn hash(self) -> u64 {
        self.wrapping_mul(PackedKeySet::<Self>::SEED)
    }

    #[inline]
    fn to_u128(self) -> u128 {
        u128::from(self)
    }
}

impl PackedSlot for u128 {
    const EMPTY: Self = u128::MAX;
    const ZERO: Self = 0;

    #[inline]
    fn or_byte(self, image: u8, shift: usize) -> Self {
        self | u128::from(image) << (8 * shift)
    }

    #[inline]
    fn hash(self) -> u64 {
        let lo = self as u64;
        let hi = (self >> 64) as u64;
        (lo ^ hi.rotate_left(32)).wrapping_mul(PackedKeySet::<Self>::SEED)
    }

    #[inline]
    fn to_u128(self) -> u128 {
        self
    }
}

/// Open-addressing membership set for packed permutation keys.
///
/// The phase-two cross closure probes membership once per (member,
/// generator) edge — ~1.6M probes for E8 — and even with the FxHash-style
/// [`PermutationHasher`], hashbrown's group probing was about a quarter of
/// the closure's sampled stacks. This table stores one slot per key with
/// linear probing and a multiplicative hash, and reallocates only on growth
/// (capacity is kept across orbits; `clear` is a fill). `u64` slots (rank
/// <= 8, covering E8) halve the per-worker table of the parallel driver.
#[derive(Clone, Debug)]
pub(crate) struct PackedKeySet<T: PackedSlot> {
    slots: Vec<T>,
    len: usize,
    /// `64 - log2(capacity)`: probe indices come from the hash's top bits.
    shift: u32,
}

impl<T: PackedSlot> PackedKeySet<T> {
    const SEED: u64 = 0x9E37_79B9_7F4A_7C15;

    pub(crate) fn new() -> Self {
        Self {
            slots: vec![T::EMPTY; 64],
            len: 0,
            shift: 58,
        }
    }

    pub(crate) fn clear(&mut self) {
        self.slots.fill(T::EMPTY);
        self.len = 0;
    }

    /// Insert `key`; returns false when it was already present.
    ///
    /// Inlined into the orbit-closure edge loop: the probe is one multiply,
    /// one masked load, and (for the duplicate majority) one compare, so the
    /// call overhead and spill traffic of an outlined probe were a
    /// measurable slice of the closure.
    #[inline]
    pub(crate) fn insert(&mut self, key: T) -> bool {
        debug_assert_ne!(key, T::EMPTY, "sentinel collision");
        let mask = self.slots.len() - 1;
        let mut slot = (key.hash() >> self.shift) as usize;
        loop {
            let stored = self.slots[slot];
            if stored == T::EMPTY {
                // The load-factor check lives on the genuine-insertion path
                // only: duplicate probes are the large majority of calls
                // (~7 of 8 cross-action edges) and need no growth
                // arithmetic. Growing here instead of at entry cannot skip a
                // needed doubling — a fresh key still triggers `grow` before
                // the 75% load factor is exceeded — and it never grows on a
                // no-op duplicate the way the entry-time check could.
                if (self.len + 1) * 4 > self.slots.len() * 3 {
                    self.grow();
                    return self.insert(key);
                }
                self.slots[slot] = key;
                self.len += 1;
                return true;
            }
            if stored == key {
                return false;
            }
            slot = (slot + 1) & mask;
        }
    }

    fn grow(&mut self) {
        let old = std::mem::take(&mut self.slots);
        self.slots = vec![T::EMPTY; old.len() * 2];
        self.shift -= 1;
        let mask = self.slots.len() - 1;
        for key in old {
            if key == T::EMPTY {
                continue;
            }
            let mut slot = (key.hash() >> self.shift) as usize;
            while self.slots[slot] != T::EMPTY {
                slot = (slot + 1) & mask;
            }
            self.slots[slot] = key;
        }
    }
}

/// Permutation-level twisted-involution orbit machinery.
///
/// A twisted involution `w after delta` is represented by its root-image
/// permutation — the same key [`TwistedConjugacyPartition`] stores. Cayley
/// succession (`r_alpha after theta`), canonicalization, and cross-action
/// closure (`r_s theta r_s`) are all permutation compositions on the finite
/// root system, so the orbit construction never touches lattice matrices.
/// Every decision the matrix-level [`InnerClass::canonicalize`] makes is a
/// pairing of a root-sum or a positivity query of a permuted root, both of
/// which are replicated here exactly, so the canonicalizing words — and
/// hence the representatives replayed from them — are unchanged.
pub(crate) struct PermutationOrbits<'a> {
    inner_class: &'a InnerClass,
    /// Distinguished generator twist: `twist[s]` is the generator whose
    /// simple root is the distinguished image of `alpha_s`.
    twist: Vec<usize>,
    /// `minus[r]` is the root id of `-r` (every root's negative is a root).
    minus: Vec<u8>,
    /// Simple-root positions in the permutation, in generator order (the
    /// packing layout of [`PermutationKey`]).
    simple_positions: Vec<u8>,
    /// Root permutations of the simple reflections, per generator.
    simple_reflections: Vec<Vec<u8>>,
    /// Cached root permutations of arbitrary-root reflections (Cayley roots).
    reflection_cache: Vec<Option<Arc<[u8]>>>,
}

impl<'a> PermutationOrbits<'a> {
    fn new(inner_class: &'a InnerClass) -> Result<Self, StructureError> {
        let roots = inner_class.root_system();
        let datum = inner_class.datum();
        let count = roots.roots().len();
        let twist = inner_class.generator_twist()?;
        let simple_positions = roots
            .simple_root_ids()
            .iter()
            .map(|id| id.0 as u8)
            .collect();
        // The root system already caches the negation table and the simple
        // reflection permutations at construction; reuse them instead of
        // rebuilding through WeylAction matrices (an action_permutation is
        // one big-int matrix apply per root).
        let minus = roots
            .negatives()
            .iter()
            .map(|id| id.0 as u8)
            .collect();
        let mut simple_reflections = try_capacity(datum.semisimple_rank())?;
        for generator in 0..datum.semisimple_rank() {
            let permutation = roots.simple_reflection_permutation(generator).ok_or(
                StructureError::IndexOutOfRange {
                    index: generator,
                    upper_bound: datum.semisimple_rank(),
                },
            )?;
            simple_reflections.push(permutation.iter().map(|id| id.0 as u8).collect());
        }
        let mut reflection_cache = try_capacity(count)?;
        reflection_cache.resize_with(count, || None);
        Ok(Self {
            inner_class,
            twist,
            minus,
            simple_positions,
            simple_reflections,
            reflection_cache,
        })
    }

    /// The simple-root packing layout of [`PermutationKey`].
    pub(crate) fn simple_positions(&self) -> &[u8] {
        &self.simple_positions
    }

    /// The root permutation of the reflection in `root`, cached per root.
    fn reflection_permutation(&mut self, root: RootId) -> Result<&[u8], StructureError> {
        self.reflection_rc(root)?;
        match &self.reflection_cache[root.0] {
            Some(permutation) => Ok(permutation),
            None => Err(StructureError::CartanClassificationInvariantViolation {
                invariant: "reflection permutation cache",
            }),
        }
    }

    /// [`Self::reflection_permutation`]'s worker, returning a shared handle
    /// so the recursion does not fight the cache borrow.
    ///
    /// The permutation is built COMBINATORIALLY, never through the
    /// reflection's lattice matrix (an `action_permutation` is one big-int
    /// matrix apply per root, and the Cayley-link loops reflect hundreds of
    /// distinct roots per inner class): reflections in a root and its
    /// negative coincide, simple roots reuse the cached simple-reflection
    /// table, and any other positive root has a simple descent — a
    /// generator with `s(alpha)` of lower height — giving
    /// `r_{s(gamma)} = s r_gamma s` as a permutation conjugation. The map
    /// on roots is exactly the matrix reflection's, so cached values are
    /// unchanged.
    fn reflection_rc(&mut self, root: RootId) -> Result<Arc<[u8]>, StructureError> {
        if let Some(cached) = &self.reflection_cache[root.0] {
            return Ok(Arc::clone(cached));
        }
        let roots = self.inner_class.root_system();
        let stride = roots.roots().len();
        let positive = match roots.is_positive(root) {
            Some(true) => root,
            Some(false) => RootId(usize::from(self.minus[root.0])),
            None => {
                return Err(StructureError::IndexOutOfRange {
                    index: root.0,
                    upper_bound: stride,
                })
            }
        };
        if positive != root {
            let permutation = self.reflection_rc(positive)?;
            self.reflection_cache[root.0] = Some(Arc::clone(&permutation));
            return Ok(permutation);
        }
        if let Some(generator) = self
            .simple_positions
            .iter()
            .position(|&id| usize::from(id) == positive.0)
        {
            let permutation: Arc<[u8]> =
                Arc::from(self.simple_reflections[generator].clone().into_boxed_slice());
            self.reflection_cache[root.0] = Some(Arc::clone(&permutation));
            return Ok(permutation);
        }
        let height: i32 = roots
            .simple_coordinates(positive)
            .ok_or(StructureError::IndexOutOfRange {
                index: positive.0,
                upper_bound: stride,
            })?
            .iter()
            .sum();
        let mut descent = None;
        for (generator, simple) in self.simple_reflections.iter().enumerate() {
            let image = RootId(usize::from(simple[positive.0]));
            let image_height: i32 = roots
                .simple_coordinates(image)
                .ok_or(StructureError::IndexOutOfRange {
                    index: image.0,
                    upper_bound: stride,
                })?
                .iter()
                .sum();
            if image_height < height {
                descent = Some((generator, image));
                break;
            }
        }
        let (generator, lower) = descent.ok_or(
            StructureError::CartanClassificationInvariantViolation {
                invariant: "reflection descent",
            },
        )?;
        let inner = self.reflection_rc(lower)?;
        let simple = &self.simple_reflections[generator];
        // r_{s(gamma)} = s r_gamma s, as permutation arrays.
        let permutation: Vec<u8> = (0..stride)
            .map(|index| simple[usize::from(inner[usize::from(simple[index])])])
            .collect();
        let permutation: Arc<[u8]> = Arc::from(permutation.into_boxed_slice());
        self.reflection_cache[root.0] = Some(Arc::clone(&permutation));
        Ok(permutation)
    }

    /// The Cayley successor `r_root after theta` at the permutation level.
    pub(crate) fn cayley_successor(
        &mut self,
        permutation: &[u8],
        root: RootId,
    ) -> Result<Vec<u8>, StructureError> {
        let reflection = self.reflection_permutation(root)?;
        Ok(permutation
            .iter()
            .map(|&image| reflection[usize::from(image)])
            .collect())
    }

    /// [`InnerClass::canonicalize_with_generators`] at the permutation
    /// level: the same three phases (dominant real sum, then dominant
    /// imaginary sum on its walls, then positivity in the residual complex
    /// subsystem) driven by the same pairings, so the returned word and
    /// canonical permutation are the matrix-level result transported through
    /// `theta |-> root-image permutation`.
    pub(crate) fn canonicalize(
        &self,
        permutation: &[u8],
        active: &[bool],
    ) -> Result<(Vec<u8>, Vec<usize>), StructureError> {
        let inner_class = self.inner_class;
        let datum = inner_class.datum();
        let roots = inner_class.root_system();
        let mut permutation = permutation.to_vec();
        let mut real_sum = self.kind_sum(&permutation, RootKind::Real)?;
        let mut imaginary_sum = self.kind_sum(&permutation, RootKind::Imaginary)?;
        let positive_root_count = roots.roots().len() / 2;
        // Same termination caps as the matrix-level canonicalize.
        let phase_one_cap = positive_root_count
            .checked_add(1)
            .and_then(|bound| bound.checked_mul(bound))
            .ok_or(StructureError::ArithmeticOverflow)?;
        let phase_three_cap = positive_root_count;
        let word_capacity = positive_root_count
            .checked_mul(2)
            .ok_or(StructureError::ArithmeticOverflow)?;
        let mut word = try_capacity(word_capacity)?;

        // Phase one (compare `make_root_sums_dominant`).
        let mut steps = 0_usize;
        loop {
            let mut changed = false;
            for generator in 0..datum.semisimple_rank() {
                if !active
                    .get(generator)
                    .copied()
                    .ok_or(StructureError::IndexOutOfRange {
                        index: generator,
                        upper_bound: active.len(),
                    })?
                {
                    continue;
                }
                let real_pairing = pair(&real_sum, &datum.simple_coroots()[generator])?;
                let should_reflect = real_pairing < 0
                    || (real_pairing == 0
                        && pair(&imaginary_sum, &datum.simple_coroots()[generator])? < 0);
                if should_reflect {
                    if steps == phase_one_cap {
                        return Err(StructureError::CartanClassificationInvariantViolation {
                            invariant: "canonicalize phase-one termination",
                        });
                    }
                    real_sum = datum.reflect_weight(generator, &real_sum)?;
                    imaginary_sum = datum.reflect_weight(generator, &imaginary_sum)?;
                    self.conjugate(&mut permutation, generator);
                    word.try_reserve(1)
                        .map_err(|_| StructureError::AllocationFailed { requested: 1 })?;
                    word.push(generator);
                    steps += 1;
                    changed = true;
                    break;
                }
            }
            if !changed {
                break;
            }
        }

        // Phase two: the residual generators (identical sum pairings).
        let residual_generators =
            inner_class.residual_generators(&real_sum, &imaginary_sum, active)?;

        // Phase three (compare `make_residual_action_positive`): the matrix
        // level tests `w(alpha_{twist(g)})`, which is `theta(alpha_g)` — the
        // permutation image of the g-th simple root.
        let mut steps = 0_usize;
        loop {
            let mut changed = false;
            for (generator, &residual) in residual_generators.iter().enumerate() {
                if !residual {
                    continue;
                }
                let simple = *roots.simple_root_ids().get(generator).ok_or(
                    StructureError::IndexOutOfRange {
                        index: generator,
                        upper_bound: roots.simple_root_ids().len(),
                    },
                )?;
                let image = *permutation.get(simple.0).ok_or(StructureError::IndexOutOfRange {
                    index: simple.0,
                    upper_bound: permutation.len(),
                })?;
                let is_positive = roots
                    .is_positive(RootId(usize::from(image)))
                    .ok_or(StructureError::InvalidRootAutomorphism)?;
                if !is_positive {
                    if steps == phase_three_cap {
                        return Err(StructureError::CartanClassificationInvariantViolation {
                            invariant: "canonicalize phase-three termination",
                        });
                    }
                    self.conjugate(&mut permutation, generator);
                    word.try_reserve(1)
                        .map_err(|_| StructureError::AllocationFailed { requested: 1 })?;
                    word.push(generator);
                    steps += 1;
                    changed = true;
                    break;
                }
            }
            if !changed {
                break;
            }
        }
        Ok((permutation, word))
    }

    /// Replace `p` by `s p s`, the permutation shadow of
    /// [`InnerClass::twisted_conjugate_action`]: `s_g w s_{twist(g)} delta =
    /// s_g theta s_g` because `s_{twist(g)} delta = delta s_g`.
    pub(crate) fn conjugate(&self, permutation: &mut Vec<u8>, generator: usize) {
        let reflection = &self.simple_reflections[generator];
        let previous = permutation.clone();
        for (slot, &reflected) in permutation.iter_mut().zip(reflection.iter()) {
            *slot = reflection[usize::from(previous[usize::from(reflected)])];
        }
    }

    /// Left-multiply by a simple reflection: `p |-> s_g p` (a Cayley step's
    /// `r_root after theta` when the root is simple).
    pub(crate) fn left_multiply_reflection(&self, permutation: &mut [u8], generator: usize) {
        let reflection = &self.simple_reflections[generator];
        for entry in permutation.iter_mut() {
            *entry = reflection[usize::from(*entry)];
        }
    }

    /// The kind of `root` under the involution with this root permutation:
    /// fixed is imaginary, negated is real, anything else is complex — the
    /// [`RootInvolutionData`] classification read off the permutation.
    pub(crate) fn kind(&self, permutation: &[u8], root: RootId) -> RootKind {
        let image = permutation[root.0];
        if usize::from(image) == root.0 {
            RootKind::Imaginary
        } else if image == self.minus[root.0] {
            RootKind::Real
        } else {
            RootKind::Complex
        }
    }

    /// Sum of the positive roots of one kind in ambient coordinates — the
    /// permutation-level shadow of `positive_root_sum`, with kinds read off
    /// the permutation (fixed is imaginary, negated is real).
    fn kind_sum(&self, permutation: &[u8], kind: RootKind) -> Result<Weight, StructureError> {
        let roots = self.inner_class.root_system();
        let mut sum = try_capacity(roots.lattice_rank())?;
        sum.resize(roots.lattice_rank(), 0_i32);
        for (index, &image) in permutation.iter().enumerate() {
            let matches = if usize::from(image) == index {
                kind == RootKind::Imaginary
            } else if image == self.minus[index] {
                kind == RootKind::Real
            } else {
                kind == RootKind::Complex
            };
            if !matches {
                continue;
            }
            let root_id = RootId(index);
            match roots.is_positive(root_id) {
                Some(true) => {}
                Some(false) => continue,
                None => return Err(StructureError::InvalidRootAutomorphism),
            }
            let root = roots
                .root(root_id)
                .ok_or(StructureError::InvalidRootAutomorphism)?;
            for (total, &coordinate) in sum.iter_mut().zip(root.as_slice()) {
                *total = total
                    .checked_add(coordinate)
                    .ok_or(StructureError::ArithmeticOverflow)?;
            }
        }
        Ok(Weight::new(sum))
    }
}

/// One twisted-conjugacy class as generated by
/// [`InnerClass::involution_orbits`]: the canonical representative plus
/// per-member parent links `(parent_index, generator)` (BFS discovery order,
/// member 0 is the representative) so member matrices can be replayed on
/// demand. The member permutations themselves are streamed to the caller
/// during the closure (see the `emit` callback) and are not retained.
struct ClassOrbit {
    representative: TwistedInvolution,
    parents: Vec<(u32, u8)>,
}

impl ClassOrbit {
    /// The number of class members (member 0 is the representative).
    fn member_count(&self) -> usize {
        self.parents.len()
    }

    /// Rebuild every member's `TwistedInvolution` from the parent links:
    /// member `i`'s Weyl action is `s after parent after twist(s)`, where
    /// `(parent, s)` is its parent link (BFS order guarantees the parent's
    /// action is already built).
    fn materialize(
        &self,
        inner_class: &InnerClass,
    ) -> Result<Vec<TwistedInvolution>, StructureError> {
        let datum = inner_class.datum();
        let roots = inner_class.root_system();
        let delta = inner_class.distinguished_involution().involution();
        let twist = inner_class.generator_twist()?;
        let rank = datum.semisimple_rank();
        let reflections = (0..rank)
            .map(|generator| WeylAction::simple_reflection(datum, generator))
            .collect::<Result<Vec<_>, _>>()?;
        let mut actions: Vec<WeylAction> = try_capacity(self.member_count())?;
        actions.push(self.representative.weyl_action().clone());
        for index in 1..self.member_count() {
            let (parent, generator) = self.parents[index];
            let action = reflections[usize::from(generator)]
                .compose(&actions[parent as usize])?
                .compose(&reflections[twist[usize::from(generator)]])?;
            actions.push(action);
        }
        actions
            .into_iter()
            .map(|action| TwistedInvolution::new(datum, roots, delta, action))
            .collect()
    }
}

fn positive_root_sum(
    roots: &RootSystem,
    involution: &RootInvolutionData,
    kind: RootKind,
) -> Result<Weight, StructureError> {
    if involution.involution().datum() != roots.datum() {
        return Err(StructureError::DatumMismatch);
    }
    let mut sum = try_capacity(roots.lattice_rank())?;
    sum.resize(roots.lattice_rank(), 0_i32);
    for root_id in involution.roots_of_kind(kind) {
        match roots.is_positive(root_id) {
            Some(true) => {}
            Some(false) => continue,
            None => return Err(StructureError::InvalidRootAutomorphism),
        }
        let root = roots
            .root(root_id)
            .ok_or(StructureError::InvalidRootAutomorphism)?;
        for (total, &coordinate) in sum.iter_mut().zip(root.as_slice()) {
            *total = total
                .checked_add(coordinate)
                .ok_or(StructureError::ArithmeticOverflow)?;
        }
    }
    Ok(Weight::new(sum))
}

fn preserves_simple_system(
    datum: &BasedRootDatum,
    root_system: &RootSystem,
    involution: &RootInvolutionData,
) -> Result<bool, StructureError> {
    let simple_root_ids = datum
        .simple_roots()
        .iter()
        .map(|root| {
            root_system
                .id_of(root)
                .ok_or(StructureError::InvalidRootAutomorphism)
        })
        .collect::<Result<BTreeSet<_>, _>>()?;
    let roots_preserved = datum
        .simple_roots()
        .iter()
        .try_fold(true, |preserves, root| {
            let root_id = root_system
                .id_of(root)
                .ok_or(StructureError::InvalidRootAutomorphism)?;
            let image = involution
                .image(root_id)
                .ok_or(StructureError::InvalidRootAutomorphism)?;
            Ok(preserves && simple_root_ids.contains(&image))
        })?;
    if !roots_preserved {
        return Ok(false);
    }
    let simple_coroots = datum
        .simple_coroots()
        .iter()
        .map(|coroot| coroot.as_slice().to_vec())
        .collect::<BTreeSet<_>>();
    datum
        .simple_coroots()
        .iter()
        .try_fold(true, |preserves, coroot| {
            let image = involution.involution().act_on_coweight(coroot)?;
            Ok(preserves && simple_coroots.contains(image.as_slice()))
        })
}

/// Port of upstream `to_positive_system` + `wrt_distinguished`
/// (structure/rootdata.cpp:1329-1387): reflect the simple-root images until
/// every one is positive, then read the conjugating Weyl word off the final
/// images and left-compose the involution with it. The composition preserves
/// the simple system; [`InnerClass::with_roots`] re-checks that invariant.
fn wrt_distinguished_word(
    datum: &BasedRootDatum,
    roots: &RootSystem,
    involution: &RootInvolutionData,
) -> Result<(LatticeInvolution, Vec<usize>), StructureError> {
    let mut images = datum
        .simple_roots()
        .iter()
        .map(|root| {
            let id = roots
                .id_of(root)
                .ok_or(StructureError::InvalidRootAutomorphism)?;
            involution
                .image(id)
                .ok_or(StructureError::InvalidRootAutomorphism)
        })
        .collect::<Result<Vec<_>, _>>()?;
    // Upstream `to_positive_system`: while some image is negative, reflect
    // every image in the root sitting at the first negative position.
    let mut steps = Vec::new();
    while let Some(generator) = images
        .iter()
        .position(|&image| roots.is_positive(image) == Some(false))
    {
        let mirror = images[generator];
        for image in &mut images {
            *image = reflect_root(roots, mirror, *image)?;
        }
        steps.push(generator);
    }
    // The images now form a positive simple system, necessarily the standard
    // one, so each is one of the datum's simple roots.
    let simple_index = |image: RootId| -> Result<usize, StructureError> {
        let coordinates = roots
            .simple_coordinates(image)
            .ok_or(StructureError::InvalidRootAutomorphism)?;
        let mut index = None;
        for (position, &coordinate) in coordinates.iter().enumerate() {
            match coordinate {
                0 => {}
                1 if index.is_none() => index = Some(position),
                _ => return Err(StructureError::InvalidBasedAutomorphism),
            }
        }
        index.ok_or(StructureError::InvalidBasedAutomorphism)
    };
    // Upstream `wrt_distinguished`: reverse the reflection steps, then twist
    // each by the final images to get the left-conjugating Weyl word. The
    // intermediate composites are not involutions, so the reflections act on
    // the bare matrices and only the final result is revalidated.
    let datum = involution.involution().datum().clone();
    let mut weight_action = involution.involution().weight_matrix().to_vec();
    let mut coweight_action = involution.involution().coweight_matrix().to_vec();
    let mut word = Vec::with_capacity(steps.len());
    for &generator in steps.iter().rev() {
        let simple = simple_index(images[generator])?;
        let (reflected_weight, reflected_coweight) =
            left_reflect(&datum, &weight_action, &coweight_action, simple)?;
        weight_action = reflected_weight;
        coweight_action = reflected_coweight;
        word.push(simple);
    }
    Ok((
        LatticeInvolution::new(&datum, weight_action, coweight_action)?,
        word,
    ))
}

/// The reflection of root `gamma` in the hyperplane orthogonal to root
/// `mirror`: `gamma - <gamma, mirror_vee> mirror`, resolved back to a root ID.
fn reflect_root(
    roots: &RootSystem,
    mirror: RootId,
    gamma: RootId,
) -> Result<RootId, StructureError> {
    let coefficient = i128::from(roots.bracket(gamma, mirror)?);
    let mirror_weight = roots
        .root(mirror)
        .ok_or(StructureError::InvalidRootAutomorphism)?;
    let gamma_weight = roots
        .root(gamma)
        .ok_or(StructureError::InvalidRootAutomorphism)?;
    let mut image = Vec::with_capacity(gamma_weight.as_slice().len());
    for (&coordinate, &mirror_coordinate) in
        gamma_weight.as_slice().iter().zip(mirror_weight.as_slice())
    {
        let value = i128::from(coordinate) - coefficient * i128::from(mirror_coordinate);
        image.push(i32::try_from(value).map_err(|_| StructureError::ArithmeticOverflow)?);
    }
    roots
        .id_of(&Weight::new(image))
        .ok_or(StructureError::InvalidRootAutomorphism)
}

/// A lattice action matrix applied by row-dot (`image = M * v`).
type LatticeAction = Vec<Vec<i32>>;

/// Left-compose involution actions with the simple reflection of `generator`,
/// on weights and coweights alike (upstream
/// `RootDatum::simple_reflect(generator, delta)`).
fn left_reflect(
    datum: &BasedRootDatum,
    weight_action: &[Vec<i32>],
    coweight_action: &[Vec<i32>],
    generator: usize,
) -> Result<(LatticeAction, LatticeAction), StructureError> {
    let rank = datum.lattice_rank();
    let mut reflected_weight = vec![vec![0; rank]; rank];
    let mut reflected_coweight = vec![vec![0; rank]; rank];
    for column in 0..rank {
        let image: Vec<i32> = weight_action.iter().map(|row| row[column]).collect();
        let image = datum.reflect_weight(generator, &Weight::new(image))?;
        for (row, &entry) in image.as_slice().iter().enumerate() {
            reflected_weight[row][column] = entry;
        }
        let coimage: Vec<i32> = coweight_action.iter().map(|row| row[column]).collect();
        let coimage = datum.reflect_coweight(generator, &Coweight::new(coimage))?;
        for (row, &entry) in coimage.as_slice().iter().enumerate() {
            reflected_coweight[row][column] = entry;
        }
    }
    Ok((reflected_weight, reflected_coweight))
}

#[cfg(test)]
mod tests {
    use crate::{BasedRootDatum, LatticeInvolution, RootKind, StructureError};

    use super::*;

    #[test]
    fn packed_key_set_dedups_across_growth() {
        // 10,000 distinct keys force several doublings from capacity 64,
        // for both slot widths.
        let mut wide = PackedKeySet::<u128>::new();
        for key in 0..10_000_u128 {
            assert!(wide.insert(key * 0x1_0001 + 7));
        }
        for key in 0..10_000_u128 {
            assert!(!wide.insert(key * 0x1_0001 + 7));
        }
        assert!(wide.insert(u128::from(u64::MAX)));
        assert!(!wide.insert(u128::from(u64::MAX)));
        wide.clear();
        assert!(wide.insert(42));
        let mut narrow = PackedKeySet::<u64>::new();
        for key in 0..10_000_u64 {
            assert!(narrow.insert(key * 0x1_0001 + 7));
        }
        for key in 0..10_000_u64 {
            assert!(!narrow.insert(key * 0x1_0001 + 7));
        }
        narrow.clear();
        assert!(narrow.insert(42));
    }

    fn compact_a2_inner_class() -> InnerClass {
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        InnerClass::new(
            datum.clone(),
            LatticeInvolution::identity(&datum).unwrap(),
            6,
        )
        .unwrap()
    }

    fn twisted_from_action(inner_class: &InnerClass, action: WeylAction) -> TwistedInvolution {
        TwistedInvolution::new(
            inner_class.datum(),
            inner_class.root_system(),
            inner_class.distinguished_involution().involution(),
            action,
        )
        .unwrap()
    }

    fn replay_twisted_conjugations(
        inner_class: &InnerClass,
        mut involution: TwistedInvolution,
        word: &[usize],
    ) -> TwistedInvolution {
        let twist = inner_class.generator_twist().unwrap();
        for &generator in word {
            let left = WeylAction::simple_reflection(inner_class.datum(), generator).unwrap();
            let right =
                WeylAction::simple_reflection(inner_class.datum(), twist[generator]).unwrap();
            let action = left
                .compose(involution.weyl_action())
                .unwrap()
                .compose(&right)
                .unwrap();
            involution = twisted_from_action(inner_class, action);
        }
        involution
    }

    fn assert_canonicalize_is_constant_on_simple_conjugacy(
        inner_class: &InnerClass,
        weyl_budget: usize,
    ) {
        let involutions = inner_class.twisted_involutions(weyl_budget).unwrap();
        assert!(!involutions.is_empty());
        for involution in involutions {
            let (canonical, word) = inner_class.canonicalize(involution.clone()).unwrap();
            assert_eq!(
                replay_twisted_conjugations(inner_class, involution.clone(), &word),
                canonical
            );
            let (canonical_again, idempotent_word) =
                inner_class.canonicalize(canonical.clone()).unwrap();
            assert_eq!(canonical_again, canonical);
            assert!(idempotent_word.is_empty());

            for generator in 0..inner_class.datum().semisimple_rank() {
                let conjugate = replay_twisted_conjugations(
                    inner_class,
                    involution.clone(),
                    std::slice::from_ref(&generator),
                );
                let (conjugate_canonical, _) = inner_class.canonicalize(conjugate).unwrap();
                assert_eq!(conjugate_canonical, canonical);
            }
        }
    }

    #[test]
    fn builds_shared_state_and_derives_split_a1_from_a_weyl_translate() {
        let datum = BasedRootDatum::standard(vec![vec![2]]).unwrap();
        let inner_class = InnerClass::new(
            datum.clone(),
            LatticeInvolution::identity(&datum).unwrap(),
            2,
        )
        .unwrap();

        assert_eq!(inner_class.datum().semisimple_rank(), 1);
        assert_eq!(inner_class.root_system().roots().len(), 2);
        assert_eq!(
            inner_class
                .distinguished_involution()
                .roots_of_kind(RootKind::Imaginary)
                .count(),
            2
        );
        let split = inner_class
            .twisted_involutions(2)
            .unwrap()
            .into_iter()
            .find(|candidate| {
                candidate
                    .root_involution()
                    .roots_of_kind(RootKind::Real)
                    .count()
                    == 2
            })
            .unwrap();
        assert_eq!(
            split
                .root_involution()
                .roots_of_kind(RootKind::Real)
                .count(),
            2
        );
        assert_eq!(
            split
                .restricted_roots(inner_class.root_system())
                .unwrap()
                .rank(),
            1
        );
    }

    #[test]
    fn preserves_root_enumeration_as_a_caller_visible_limit() {
        let datum = BasedRootDatum::standard(vec![vec![2]]).unwrap();
        let involution = LatticeInvolution::identity(&datum).unwrap();
        assert_eq!(
            InnerClass::new(datum, involution, 1),
            Err(StructureError::ResourceLimitExceeded { limit: 1 })
        );
    }

    #[test]
    fn enumerates_twisted_involutions_without_claiming_cartan_classes() {
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let inner_class = InnerClass::new(
            datum.clone(),
            LatticeInvolution::identity(&datum).unwrap(),
            6,
        )
        .unwrap();

        assert_eq!(inner_class.twisted_involutions(6).unwrap().len(), 4);
        // The budget bounds the TOTAL twisted-involution count: the two
        // classes have sizes 1 and 3, so a budget of 3 trips on the second.
        assert_eq!(
            inner_class.twisted_involutions(3),
            Err(StructureError::ResourceLimitExceeded { limit: 3 })
        );
    }

    #[test]
    fn groups_a2_twisted_involutions_into_deterministic_twisted_conjugacy_classes() {
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let inner_class = InnerClass::new(
            datum.clone(),
            LatticeInvolution::identity(&datum).unwrap(),
            6,
        )
        .unwrap();

        let mut orbit_sizes = inner_class
            .twisted_conjugacy_classes(6)
            .unwrap()
            .iter()
            .map(|class| class.twisted_involution_count())
            .collect::<Vec<_>>();
        orbit_sizes.sort_unstable();
        assert_eq!(orbit_sizes, vec![1, 3]);
    }

    #[test]
    fn uses_twisted_not_ordinary_conjugacy_for_an_a2_diagram_twist() {
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let diagram_twist = LatticeInvolution::new(
            &datum,
            vec![vec![0, 1], vec![1, 0]],
            vec![vec![0, 1], vec![1, 0]],
        )
        .unwrap();
        let inner_class = InnerClass::new(datum, diagram_twist, 6).unwrap();

        let mut orbit_sizes = inner_class
            .twisted_conjugacy_classes(6)
            .unwrap()
            .iter()
            .map(|class| class.twisted_involution_count())
            .collect::<Vec<_>>();
        orbit_sizes.sort_unstable();
        assert_eq!(orbit_sizes, vec![1, 3]);
    }

    #[test]
    fn rejects_a_distinguished_action_that_does_not_preserve_simple_roots() {
        let datum = BasedRootDatum::standard(vec![vec![2]]).unwrap();
        let negative_identity =
            LatticeInvolution::new(&datum, vec![vec![-1]], vec![vec![-1]]).unwrap();
        assert_eq!(
            InnerClass::new(datum, negative_identity, 2),
            Err(StructureError::InvalidBasedAutomorphism)
        );
    }

    #[test]
    fn rejects_a_simple_coroot_shift_into_the_central_torus() {
        let datum = BasedRootDatum::from_simple_data(
            2,
            vec![vec![2]],
            vec![crate::Weight::new(vec![1, 0])],
            vec![crate::Coweight::new(vec![2, 0])],
        )
        .unwrap();
        let action = LatticeInvolution::new(
            &datum,
            vec![vec![1, 2], vec![0, -1]],
            vec![vec![1, 0], vec![2, -1]],
        )
        .unwrap();

        // The coroot-transport check inside `RootInvolutionData::new` now
        // rejects this action before the simple-system check can run.
        assert_eq!(
            InnerClass::new(datum, action, 2),
            Err(StructureError::SimpleCorootImageMismatch {
                simple_root: 0,
                image_root: crate::Weight::new(vec![1, 0]),
            })
        );
    }

    #[test]
    fn conjugates_an_unbased_a2_involution_to_the_distinguished_identity() {
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        // The negated diagram swap maps each simple root to minus the other
        // one: a root-datum involution that is not based.
        let negated_swap = LatticeInvolution::new(
            &datum,
            vec![vec![0, -1], vec![-1, 0]],
            vec![vec![0, -1], vec![-1, 0]],
        )
        .unwrap();
        let inner_class = InnerClass::from_root_involution(datum.clone(), negated_swap, 6).unwrap();
        let expected = InnerClass::new(
            datum.clone(),
            LatticeInvolution::identity(&datum).unwrap(),
            6,
        )
        .unwrap();
        assert_eq!(inner_class, expected);
    }

    #[test]
    fn accepts_a_based_involution_unchanged_through_the_general_entry() {
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let diagram_twist = LatticeInvolution::new(
            &datum,
            vec![vec![0, 1], vec![1, 0]],
            vec![vec![0, 1], vec![1, 0]],
        )
        .unwrap();
        let general =
            InnerClass::from_root_involution(datum.clone(), diagram_twist.clone(), 6).unwrap();
        let strict = InnerClass::new(datum, diagram_twist, 6).unwrap();
        assert_eq!(general, strict);
    }

    #[test]
    fn general_entry_still_rejects_actions_that_do_not_permute_roots() {
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let involution = LatticeInvolution::new(
            &datum,
            vec![vec![1, 1], vec![0, -1]],
            vec![vec![1, 0], vec![1, -1]],
        )
        .unwrap();
        assert_eq!(
            InnerClass::from_root_involution(datum, involution, 6),
            Err(StructureError::SimpleCorootImageMismatch {
                simple_root: 0,
                image_root: crate::Weight::new(vec![1, 0]),
            })
        );
    }

    #[test]
    fn based_involution_twist_reads_the_simple_root_permutation() {
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let inner_class = InnerClass::new(
            datum.clone(),
            LatticeInvolution::identity(&datum).unwrap(),
            6,
        )
        .unwrap();
        let flip = LatticeInvolution::new(
            &datum,
            vec![vec![0, 1], vec![1, 0]],
            vec![vec![0, 1], vec![1, 0]],
        )
        .unwrap();
        assert_eq!(inner_class.based_involution_twist(flip), Ok(vec![1, 0]));
        assert_eq!(
            inner_class.based_involution_twist(LatticeInvolution::identity(&datum).unwrap()),
            Ok(vec![0, 1])
        );
        // The negated flip maps each simple root to minus the other one:
        // a root-datum involution, but not one of the BASED datum.
        let negated_flip = LatticeInvolution::new(
            &datum,
            vec![vec![0, -1], vec![-1, 0]],
            vec![vec![0, -1], vec![-1, 0]],
        )
        .unwrap();
        assert_eq!(
            inner_class.based_involution_twist(negated_flip),
            Err(StructureError::InvalidBasedAutomorphism)
        );
        // A lattice involution that fails simple-coroot transport is rejected
        // by the structured preflight before the full root permutation scan.
        let drifting = LatticeInvolution::new(
            &datum,
            vec![vec![1, 1], vec![0, -1]],
            vec![vec![1, 0], vec![1, -1]],
        )
        .unwrap();
        assert_eq!(
            inner_class.based_involution_twist(drifting),
            Err(StructureError::SimpleCorootImageMismatch {
                simple_root: 0,
                image_root: crate::Weight::new(vec![1, 0]),
            })
        );
    }

    #[test]
    fn twisted_from_involution_factors_unbased_and_rejects_foreign() {
        // A1 anchor (the seed_x0 fixture's matrices): the compact class
        // factors [[1]] as e and [[-1]] as the simple reflection.
        let datum = BasedRootDatum::standard(vec![vec![2]]).unwrap();
        let inner_class = InnerClass::new(
            datum.clone(),
            LatticeInvolution::identity(&datum).unwrap(),
            2,
        )
        .unwrap();
        let identity = inner_class
            .twisted_from_involution(LatticeInvolution::identity(&datum).unwrap())
            .unwrap();
        assert!(identity.is_identity());
        let negated = LatticeInvolution::new(&datum, vec![vec![-1]], vec![vec![-1]]).unwrap();
        let simple = inner_class.twisted_from_involution(negated).unwrap();
        assert_eq!(simple.length(), 1);
        assert_eq!(
            simple.reduced_word(inner_class.root_system()).unwrap(),
            vec![0]
        );

        // B2 anchor: -1 = w0 is central, so the compact class admits it and
        // factors it as the longest element.
        let datum = BasedRootDatum::standard(vec![vec![2, -2], vec![-1, 2]]).unwrap();
        let inner_class = InnerClass::new(
            datum.clone(),
            LatticeInvolution::identity(&datum).unwrap(),
            8,
        )
        .unwrap();
        let negated = LatticeInvolution::new(
            &datum,
            vec![vec![-1, 0], vec![0, -1]],
            vec![vec![-1, 0], vec![0, -1]],
        )
        .unwrap();
        let longest = inner_class.twisted_from_involution(negated).unwrap();
        assert_eq!(longest.length(), 4);

        // A2 anchor: the based diagram flip is an involution of the based
        // datum but not of the COMPACT inner class — upstream's
        // "Involution not in this inner class".
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let inner_class = InnerClass::new(
            datum.clone(),
            LatticeInvolution::identity(&datum).unwrap(),
            6,
        )
        .unwrap();
        let flip = LatticeInvolution::new(
            &datum,
            vec![vec![0, 1], vec![1, 0]],
            vec![vec![0, 1], vec![1, 0]],
        )
        .unwrap();
        assert_eq!(
            inner_class.twisted_from_involution(flip),
            Err(StructureError::InvalidBasedAutomorphism)
        );
    }

    #[test]
    fn inner_class_pair_exposes_the_fixture_weyl_factor() {
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let opposition = LatticeInvolution::new(
            &datum,
            vec![vec![0, -1], vec![-1, 0]],
            vec![vec![0, -1], vec![-1, 0]],
        )
        .unwrap();

        let (factor, inner_class) =
            inner_class_with_twisted_involution(datum.clone(), opposition, 6).unwrap();

        assert_eq!(factor.length(), 3);
        assert_eq!(
            factor.reduced_word(inner_class.root_system()).unwrap(),
            vec![0, 1, 0]
        );
        assert_eq!(
            inner_class
                .distinguished_involution()
                .involution()
                .weight_matrix(),
            &[vec![1, 0], vec![0, 1]]
        );

        let (identity, same_class) = inner_class_with_twisted_involution(
            datum.clone(),
            LatticeInvolution::identity(&datum).unwrap(),
            6,
        )
        .unwrap();
        assert!(identity.is_identity());
        assert_eq!(same_class, inner_class);
    }

    #[test]
    fn owned_inner_class_constructor_returns_factor_and_honors_root_budget() {
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        assert_eq!(
            InnerClass::from_root_involution_with_factor(
                datum.clone(),
                LatticeInvolution::identity(&datum).unwrap(),
                3,
            ),
            Err(StructureError::ResourceLimitExceeded { limit: 3 })
        );

        let opposition = LatticeInvolution::new(
            &datum,
            vec![vec![0, -1], vec![-1, 0]],
            vec![vec![0, -1], vec![-1, 0]],
        )
        .unwrap();

        let (inner_class, factor) =
            InnerClass::from_root_involution_with_factor(datum, opposition, 6).unwrap();
        assert_eq!(factor.length(), 3);
        assert_eq!(
            factor.reduced_word(inner_class.root_system()).unwrap(),
            vec![0, 1, 0]
        );
    }

    #[test]
    fn canonical_involution_expr_matches_the_b2_kgb_table_words() {
        // A1 anchor: the split Cartan's involution is the simple reflection,
        // printed `1^e` by the oracle's print_KGB (cross, not conjugation).
        let datum = BasedRootDatum::standard(vec![vec![2]]).unwrap();
        let inner_class = InnerClass::new(
            datum.clone(),
            LatticeInvolution::identity(&datum).unwrap(),
            2,
        )
        .unwrap();
        let simple = WeylElement::simple_reflection(inner_class.root_system(), 0).unwrap();
        assert_eq!(inner_class.canonical_involution_expr(&simple), Ok(vec![0]));
        let identity = WeylElement::identity(inner_class.root_system()).unwrap();
        assert_eq!(
            inner_class.canonical_involution_expr(&identity),
            Ok(Vec::new())
        );

        // B2 split inner class (identity distinguished): the words below
        // are the oracle's print_KGB involution column for the quasisplit
        // form — `1^2x1^e` for w0, `1x2^e` for s0.s1.s0, `2x1^e` for
        // s1.s0.s1 (bitwise-complemented entries print with `x`).
        let datum = BasedRootDatum::standard(vec![vec![2, -2], vec![-1, 2]]).unwrap();
        let inner_class = InnerClass::new(
            datum.clone(),
            LatticeInvolution::identity(&datum).unwrap(),
            8,
        )
        .unwrap();
        let system = inner_class.root_system();
        let word = |letters: &[usize]| {
            let mut element = WeylElement::identity(system).unwrap();
            for &letter in letters {
                element = element.right_multiply_simple(system, letter).unwrap().0;
            }
            element
        };
        assert_eq!(
            inner_class.canonical_involution_expr(&word(&[0, 1, 0, 1])),
            Ok(vec![0, !1, 0])
        );
        assert_eq!(
            inner_class.canonical_involution_expr(&word(&[0, 1, 0])),
            Ok(vec![!0, 1])
        );
        assert_eq!(
            inner_class.canonical_involution_expr(&word(&[1, 0, 1])),
            Ok(vec![!1, 0])
        );
    }

    #[test]
    fn canonicalize_fixes_the_identity() {
        let inner_class = compact_a2_inner_class();
        let identity = twisted_from_action(
            &inner_class,
            WeylAction::identity(inner_class.datum()).unwrap(),
        );

        let (canonical, word) = inner_class.canonicalize(identity.clone()).unwrap();

        assert_eq!(canonical, identity);
        assert!(word.is_empty());
    }

    #[test]
    fn canonicalize_matches_the_a2_noncanonical_probe_representatives() {
        let inner_class = compact_a2_inner_class();
        let first = WeylAction::simple_reflection(inner_class.datum(), 0).unwrap();
        let second = WeylAction::simple_reflection(inner_class.datum(), 1).unwrap();
        assert_eq!(first.matrix(), &[vec![-1, 1], vec![0, 1]]);
        assert_eq!(second.matrix(), &[vec![1, 0], vec![1, -1]]);
        let expected = twisted_from_action(
            &inner_class,
            second.compose(&first).unwrap().compose(&second).unwrap(),
        );

        let first = twisted_from_action(&inner_class, first);
        let second = twisted_from_action(&inner_class, second);
        let (first_canonical, first_word) = inner_class.canonicalize(first.clone()).unwrap();
        let (second_canonical, second_word) = inner_class.canonicalize(second.clone()).unwrap();

        assert_eq!(first_canonical, expected);
        assert_eq!(second_canonical, expected);
        assert_eq!(first_word, vec![1]);
        assert_eq!(second_word, vec![0]);
        assert_eq!(
            replay_twisted_conjugations(&inner_class, first, &first_word),
            first_canonical
        );
        assert_eq!(
            replay_twisted_conjugations(&inner_class, second, &second_word),
            second_canonical
        );
    }

    #[test]
    fn canonicalize_is_idempotent() {
        let inner_class = compact_a2_inner_class();
        let first = twisted_from_action(
            &inner_class,
            WeylAction::simple_reflection(inner_class.datum(), 0).unwrap(),
        );
        let (canonical, _) = inner_class.canonicalize(first).unwrap();

        let (canonical_again, word) = inner_class.canonicalize(canonical.clone()).unwrap();

        assert_eq!(canonical_again, canonical);
        assert!(word.is_empty());
    }

    #[test]
    fn canonicalize_word_replays_forward_for_a_noncommuting_multi_step_case() {
        let datum = BasedRootDatum::standard(vec![vec![2, -1, 0], vec![-1, 2, -1], vec![0, -1, 2]])
            .unwrap();
        let inner_class = InnerClass::new(
            datum.clone(),
            LatticeInvolution::identity(&datum).unwrap(),
            12,
        )
        .unwrap();
        let involutions = inner_class.twisted_involutions(24).unwrap();
        let mut witnessed = false;

        for involution in involutions {
            let (canonical, word) = inner_class.canonicalize(involution.clone()).unwrap();
            if word.len() < 2 || !word.windows(2).any(|pair| pair[0] != pair[1]) {
                continue;
            }
            let reversed = word.iter().copied().rev().collect::<Vec<_>>();
            if replay_twisted_conjugations(&inner_class, involution.clone(), &reversed) == canonical
            {
                continue;
            }
            assert_eq!(
                replay_twisted_conjugations(&inner_class, involution, &word),
                canonical
            );
            witnessed = true;
            break;
        }

        assert!(witnessed, "A3 must expose a noncommuting multi-step word");
    }

    #[test]
    fn canonicalize_is_class_constant_on_b2_and_twisted_a2() {
        let b2_datum = BasedRootDatum::standard(vec![vec![2, -2], vec![-1, 2]]).unwrap();
        let b2 = InnerClass::new(
            b2_datum.clone(),
            LatticeInvolution::identity(&b2_datum).unwrap(),
            8,
        )
        .unwrap();
        assert_canonicalize_is_constant_on_simple_conjugacy(&b2, 8);

        let a2_datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let diagram_swap = LatticeInvolution::new(
            &a2_datum,
            vec![vec![0, 1], vec![1, 0]],
            vec![vec![0, 1], vec![1, 0]],
        )
        .unwrap();
        let twisted_a2 = InnerClass::new(a2_datum, diagram_swap, 6).unwrap();
        assert_canonicalize_is_constant_on_simple_conjugacy(&twisted_a2, 6);
    }

    #[test]
    fn canonicalize_eliminates_negative_images_in_the_residual_complex_subsystem() {
        let datum = BasedRootDatum::standard(vec![vec![2, 0], vec![0, 2]]).unwrap();
        let swap = LatticeInvolution::new(
            &datum,
            vec![vec![0, 1], vec![1, 0]],
            vec![vec![0, 1], vec![1, 0]],
        )
        .unwrap();
        let inner_class = InnerClass::new(datum.clone(), swap, 4).unwrap();
        let first = WeylAction::simple_reflection(&datum, 0).unwrap();
        let second = WeylAction::simple_reflection(&datum, 1).unwrap();
        let noncanonical = twisted_from_action(&inner_class, first.compose(&second).unwrap());
        let expected = twisted_from_action(&inner_class, WeylAction::identity(&datum).unwrap());

        let (canonical, word) = inner_class.canonicalize(noncanonical.clone()).unwrap();

        assert_eq!(canonical, expected);
        assert_eq!(word, vec![0]);
        assert_eq!(
            replay_twisted_conjugations(&inner_class, noncanonical, &word),
            canonical
        );
    }

    #[test]
    fn canonicalize_rejects_a_different_distinguished_involution_on_the_same_datum() {
        let compact = compact_a2_inner_class();
        let datum = compact.datum().clone();
        let diagram_swap = LatticeInvolution::new(
            &datum,
            vec![vec![0, 1], vec![1, 0]],
            vec![vec![0, 1], vec![1, 0]],
        )
        .unwrap();
        let diagram_class = InnerClass::new(datum.clone(), diagram_swap, 6).unwrap();
        let foreign = twisted_from_action(&diagram_class, WeylAction::identity(&datum).unwrap());

        assert_eq!(
            compact.canonicalize(foreign),
            Err(StructureError::DistinguishedInvolutionMismatch)
        );
    }

    #[test]
    fn rank_zero_partition_keeps_one_empty_permutation_member() {
        let datum = BasedRootDatum::from_simple_data(0, vec![], vec![], vec![]).unwrap();
        let identity = LatticeInvolution::identity(&datum).unwrap();
        let inner_class = InnerClass::from_root_involution(datum, identity, 1).unwrap();
        let partition = inner_class.twisted_conjugacy_partition(1).unwrap();

        assert_eq!(partition.classes().len(), 1);
        assert_eq!(partition.classes()[0].twisted_involution_count(), 1);
        assert_eq!(partition.class_index_of_permutation(&[]), Some(0));
    }

    #[test]
    fn builds_an_inner_class_for_a_datum_with_a_central_torus() {
        // A1.T1: lattice rank 2, semisimple rank 1 (the language fixture
        // `root_datum([[2,0]],[[1,0]],true)`; oracle job 3502476 builds the
        // identity inner class on it and prints inner class type 'cc').
        let datum = BasedRootDatum::from_simple_data(
            2,
            vec![vec![2]],
            vec![crate::Weight::new(vec![2, 0])],
            vec![crate::Coweight::new(vec![1, 0])],
        )
        .unwrap();
        let identity = LatticeInvolution::identity(&datum).unwrap();
        let inner_class =
            InnerClass::from_root_involution(datum.clone(), identity.clone(), 2).unwrap();

        assert_eq!(inner_class.datum().lattice_rank(), 2);
        assert_eq!(inner_class.datum().semisimple_rank(), 1);
        assert_eq!(inner_class.root_system().roots().len(), 2);
        // The distinguished involution acts on the FULL lattice, central
        // direction included: both action matrices are rank-2 identities.
        let distinguished = inner_class.distinguished_involution().involution();
        assert_eq!(distinguished.lattice_rank(), 2);
        assert_eq!(distinguished.weight_matrix(), identity.weight_matrix());
        assert_eq!(distinguished.coweight_matrix(), identity.coweight_matrix());
        // Every root is imaginary for the identity, matching upstream's 'cc'.
        assert_eq!(
            inner_class
                .distinguished_involution()
                .roots_of_kind(RootKind::Imaginary)
                .count(),
            2
        );
        assert_eq!(
            inner_class
                .distinguished_involution()
                .roots_of_kind(RootKind::Real)
                .count(),
            0
        );
        // The strict entry accepts the same already-based involution.
        assert_eq!(
            InnerClass::new(datum.clone(), identity, 2).unwrap(),
            inner_class
        );
        // The central reflection is a valid involution of the unbased datum:
        // it negates the central direction while fixing the root, so it is a
        // second distinguished involution on this datum (upstream's other
        // inner class of A1.T1).
        let central_reflection = LatticeInvolution::new(
            &datum,
            vec![vec![1, 0], vec![0, -1]],
            vec![vec![1, 0], vec![0, -1]],
        )
        .unwrap();
        let twisted =
            InnerClass::from_root_involution(datum, central_reflection.clone(), 2).unwrap();
        assert_eq!(
            twisted
                .distinguished_involution()
                .involution()
                .weight_matrix(),
            central_reflection.weight_matrix()
        );
    }
}
