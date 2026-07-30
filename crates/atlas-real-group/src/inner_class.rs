use std::collections::{BTreeMap, BTreeSet};

use crate::{
    BasedRootDatum, Coweight, LatticeInvolution, RootId, RootInvolutionData, RootSystem,
    StructureError, TwistedConjugacyClass, TwistedConjugacyPartition, TwistedInvolution, Weight,
    WeylAction, WeylGroup,
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
        let roots = RootSystem::enumerate(&datum, root_budget)?;
        let involution = RootInvolutionData::new(&roots, involution)?;
        let distinguished = wrt_distinguished(&datum, &roots, &involution)?;
        let distinguished = RootInvolutionData::new(&roots, distinguished)?;
        Self::with_roots(datum, roots, distinguished)
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
fn wrt_distinguished(
    datum: &BasedRootDatum,
    roots: &RootSystem,
    involution: &RootInvolutionData,
) -> Result<LatticeInvolution, StructureError> {
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
    for &generator in steps.iter().rev() {
        let simple = simple_index(images[generator])?;
        let (reflected_weight, reflected_coweight) =
            left_reflect(&datum, &weight_action, &coweight_action, simple)?;
        weight_action = reflected_weight;
        coweight_action = reflected_coweight;
    }
    LatticeInvolution::new(&datum, weight_action, coweight_action)
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
            Err(StructureError::InvalidRootDatumAutomorphism)
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
            Err(StructureError::InvalidRootAutomorphism)
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
        // A lattice involution that does not permute the roots fails the
        // earlier root-automorphism gate instead.
        let drifting = LatticeInvolution::new(
            &datum,
            vec![vec![1, 1], vec![0, -1]],
            vec![vec![1, 0], vec![1, -1]],
        )
        .unwrap();
        assert_eq!(
            inner_class.based_involution_twist(drifting),
            Err(StructureError::InvalidRootAutomorphism)
        );
    }
}
