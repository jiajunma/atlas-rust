use std::collections::{BTreeMap, BTreeSet};

use crate::grading::try_capacity;
use crate::twisted_involution::compose_matrices;
use crate::{
    pair, BasedRootDatum, Coweight, LatticeInvolution, RootId, RootInvolutionData, RootKind,
    RootSystem, StructureError, TwistedConjugacyClass, TwistedConjugacyPartition,
    TwistedInvolution, Weight, WeylAction, WeylElement, WeylGroup,
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
        self.validate_twisted_involution(&involution)?;
        let twist = self.generator_twist()?;
        let mut real_sum =
            positive_root_sum(&self.roots, involution.root_involution(), RootKind::Real)?;
        let mut imaginary_sum = positive_root_sum(
            &self.roots,
            involution.root_involution(),
            RootKind::Imaginary,
        )?;
        let mut action = involution.weyl_action().clone();
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
        )?;
        let residual_generators = self.residual_generators(&real_sum, &imaginary_sum)?;
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
    // imaginary sum on the walls of the real sum.
    fn make_root_sums_dominant(
        &self,
        action: &mut WeylAction,
        twist: &[usize],
        real_sum: &mut Weight,
        imaginary_sum: &mut Weight,
        word: &mut Vec<usize>,
        max_steps: usize,
    ) -> Result<(), StructureError> {
        let mut steps = 0_usize;
        loop {
            let mut changed = false;
            for generator in 0..self.datum.semisimple_rank() {
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
    // dominant sums. They generate the residual complex subsystem.
    fn residual_generators(
        &self,
        real_sum: &Weight,
        imaginary_sum: &Weight,
    ) -> Result<Vec<bool>, StructureError> {
        let mut residual_generators = try_capacity(self.datum.semisimple_rank())?;
        for generator in 0..self.datum.semisimple_rank() {
            let real_pairing = pair(real_sum, &self.datum.simple_coroots()[generator])?;
            let active = real_pairing <= 0
                && pair(imaginary_sum, &self.datum.simple_coroots()[generator])? <= 0;
            residual_generators.push(active);
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
        if involution.weyl_action().datum() != &self.datum
            || involution.root_involution().involution().datum() != &self.datum
        {
            return Err(StructureError::DatumMismatch);
        }
        let distinguished = self.distinguished_involution.involution();
        let stored = involution.root_involution().involution();
        if compose_matrices(
            involution.weyl_action().matrix(),
            distinguished.weight_matrix(),
        )? != stored.weight_matrix()
            || compose_matrices(
                involution.weyl_action().coweight_matrix(),
                distinguished.coweight_matrix(),
            )? != stored.coweight_matrix()
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
    /// Cartan classes by twisted conjugacy or Cayley transforms.
    pub fn twisted_involutions(
        &self,
        weyl_budget: usize,
    ) -> Result<Vec<TwistedInvolution>, StructureError> {
        Ok(self.enumerated_twisted_involutions(weyl_budget)?.1)
    }

    /// Deterministic Weyl twisted-conjugacy orbits of twisted involutions.
    ///
    /// The representative is not Atlas-canonical. This operation does not yet
    /// construct Cartan fibers, real forms, or the Cartan partial order.
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
        let (weyl_actions, candidates) = self.enumerated_twisted_involutions(weyl_budget)?;
        let permutations = candidates
            .iter()
            .map(|candidate| {
                candidate
                    .root_involution()
                    .image_permutation()
                    .iter()
                    .map(|id| id.0)
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let candidate_by_permutation = permutations
            .iter()
            .enumerate()
            .map(|(index, permutation)| (permutation.clone(), index))
            .collect::<BTreeMap<_, _>>();
        let weyl_permutations = weyl_actions
            .iter()
            .map(|action| self.roots.action_permutation(action))
            .collect::<Result<Vec<_>, _>>()?;
        let mut visited = vec![false; candidates.len()];
        let mut classes = Vec::new();
        let mut class_by_permutation = BTreeMap::new();
        for (index, candidate) in candidates.iter().enumerate() {
            if visited[index] {
                continue;
            }
            let candidate_permutation = candidate.root_involution().image_permutation();
            let orbit =
                weyl_permutations
                    .iter()
                    .try_fold(BTreeSet::new(), |mut orbit, action| {
                        let inverse = inverse_permutation(action)?;
                        let conjugate = (0..action.len())
                            .map(|root| action[candidate_permutation[inverse[root]].0].0)
                            .collect::<Vec<_>>();
                        let conjugate_index = candidate_by_permutation
                            .get(&conjugate)
                            .copied()
                            .ok_or(StructureError::InvalidRootAutomorphism)?;
                        orbit.insert(conjugate_index);
                        Ok(orbit)
                    })?;
            for member in &orbit {
                visited[*member] = true;
                class_by_permutation.insert(permutations[*member].clone(), classes.len());
            }
            classes.push(TwistedConjugacyClass::new(candidate.clone(), orbit.len()));
        }
        Ok(TwistedConjugacyPartition::new(
            self.datum.clone(),
            self.distinguished_involution.clone(),
            classes,
            class_by_permutation,
        ))
    }

    fn enumerated_twisted_involutions(
        &self,
        weyl_budget: usize,
    ) -> Result<(Vec<WeylAction>, Vec<TwistedInvolution>), StructureError> {
        let actions = WeylGroup::new(self.datum.clone()).enumerate_actions(weyl_budget)?;
        let involutions = actions
            .iter()
            .cloned()
            .filter_map(|action| {
                match TwistedInvolution::new(
                    &self.datum,
                    &self.roots,
                    self.distinguished_involution.involution(),
                    action,
                ) {
                    Ok(involution) => Some(Ok(involution)),
                    Err(StructureError::InvalidInvolution) => None,
                    Err(error) => Some(Err(error)),
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok((actions, involutions))
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

fn inverse_permutation(permutation: &[RootId]) -> Result<Vec<usize>, StructureError> {
    let mut inverse = vec![None; permutation.len()];
    for (source, image) in permutation.iter().enumerate() {
        let target = inverse
            .get_mut(image.0)
            .ok_or(StructureError::InvalidRootAutomorphism)?;
        if target.replace(source).is_some() {
            return Err(StructureError::InvalidRootAutomorphism);
        }
    }
    inverse
        .into_iter()
        .collect::<Option<Vec<_>>>()
        .ok_or(StructureError::InvalidRootAutomorphism)
}

#[cfg(test)]
mod tests {
    use crate::{BasedRootDatum, LatticeInvolution, RootKind, StructureError};

    use super::*;

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
        let (_, involutions) = inner_class
            .enumerated_twisted_involutions(weyl_budget)
            .unwrap();
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
        assert_eq!(
            inner_class.twisted_involutions(5),
            Err(StructureError::ResourceLimitExceeded { limit: 5 })
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
        let (_, involutions) = inner_class.enumerated_twisted_involutions(24).unwrap();
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
