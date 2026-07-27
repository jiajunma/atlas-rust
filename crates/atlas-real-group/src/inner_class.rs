use std::collections::{BTreeMap, BTreeSet};

use crate::{
    BasedRootDatum, LatticeInvolution, RootId, RootInvolutionData, RootSystem, StructureError,
    TwistedConjugacyClass, TwistedConjugacyPartition, TwistedInvolution, WeylAction, WeylGroup,
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
}
