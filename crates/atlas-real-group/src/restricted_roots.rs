use std::collections::BTreeMap;

use crate::{RootId, RootInvolutionData, RootSystem, StructureError, Weight};

/// An opaque element of `X^*/ker(1-theta)`.
///
/// Its private coordinates encode the class through `(1-theta)(weight)`. This
/// is an injective representation of the quotient into the image lattice, not
/// an ambient weight coordinate: in split A1, the class of `alpha` is encoded
/// by `2alpha` without being identified with the class of `2alpha`.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RestrictedWeight(Vec<i32>);

impl RestrictedWeight {
    pub fn restrict(
        weight: &Weight,
        involution: &crate::LatticeInvolution,
    ) -> Result<Self, StructureError> {
        let image = involution.act_on_weight(weight)?;
        weight
            .as_slice()
            .iter()
            .zip(image.as_slice())
            .map(|(&coordinate, &theta_coordinate)| {
                coordinate
                    .checked_sub(theta_coordinate)
                    .ok_or(StructureError::ArithmeticOverflow)
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Self)
    }

    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|&coordinate| coordinate == 0)
    }

    fn doubled(&self) -> Result<Self, StructureError> {
        self.0
            .iter()
            .map(|coordinate| {
                coordinate
                    .checked_mul(2)
                    .ok_or(StructureError::ArithmeticOverflow)
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Self)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RestrictedRoot {
    weight: RestrictedWeight,
    fiber: Vec<RootId>,
}

impl RestrictedRoot {
    pub fn weight(&self) -> &RestrictedWeight {
        &self.weight
    }

    /// Number of ordinary roots in this restriction fiber.
    pub fn multiplicity(&self) -> usize {
        self.fiber.len()
    }

    pub fn fiber(&self) -> &[RootId] {
        &self.fiber
    }
}

/// Restricted roots together with their ordinary-root fibers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RestrictedRootSystem {
    rank: usize,
    roots: Vec<RestrictedRoot>,
}

impl RestrictedRootSystem {
    pub fn build(
        root_system: &RootSystem,
        involution_data: &RootInvolutionData,
    ) -> Result<Self, StructureError> {
        let involution = involution_data.involution();
        if involution.datum() != root_system.datum() {
            return Err(StructureError::DatumMismatch);
        }
        if involution.lattice_rank() != root_system.lattice_rank() {
            return Err(StructureError::RankMismatch {
                expected: root_system.lattice_rank(),
                actual: involution.lattice_rank(),
            });
        }
        let mut fibers = BTreeMap::<RestrictedWeight, Vec<RootId>>::new();
        for (index, root) in root_system.roots().iter().enumerate() {
            let restricted = RestrictedWeight::restrict(root, involution)?;
            if !restricted.is_zero() {
                fibers.entry(restricted).or_default().push(RootId(index));
            }
        }
        Ok(Self {
            rank: involution.anti_invariant_rank()?,
            roots: fibers
                .into_iter()
                .map(|(weight, fiber)| RestrictedRoot { weight, fiber })
                .collect(),
        })
    }

    pub fn rank(&self) -> usize {
        self.rank
    }

    pub fn roots(&self) -> &[RestrictedRoot] {
        &self.roots
    }

    pub fn root(&self, weight: &RestrictedWeight) -> Option<&RestrictedRoot> {
        self.roots
            .binary_search_by(|root| root.weight.cmp(weight))
            .ok()
            .map(|index| &self.roots[index])
    }

    pub fn is_multipliable(&self, weight: &RestrictedWeight) -> Result<bool, StructureError> {
        Ok(self.root(&weight.doubled()?).is_some())
    }
}

#[cfg(test)]
mod tests {
    use crate::{BasedRootDatum, LatticeInvolution, RootInvolutionData, RootSystem, Weight};

    use super::*;

    #[test]
    fn split_a1_uses_the_quotient_class_not_a_doubled_ambient_root() {
        let datum = BasedRootDatum::standard(vec![vec![2]]).unwrap();
        let roots = RootSystem::enumerate(&datum, 2).unwrap();
        let involution = LatticeInvolution::new(&datum, vec![vec![-1]], vec![vec![-1]]).unwrap();
        let data = RootInvolutionData::new(&roots, involution).unwrap();
        let restricted = RestrictedRootSystem::build(&roots, &data).unwrap();
        let alpha = RestrictedWeight::restrict(&Weight::new(vec![1]), data.involution()).unwrap();
        assert_eq!(restricted.rank(), 1);
        assert_eq!(restricted.roots().len(), 2);
        assert_eq!(restricted.root(&alpha).unwrap().multiplicity(), 1);
        assert!(!restricted.is_multipliable(&alpha).unwrap());
    }

    #[test]
    fn compact_a1_has_no_restricted_roots() {
        let datum = BasedRootDatum::standard(vec![vec![2]]).unwrap();
        let roots = RootSystem::enumerate(&datum, 2).unwrap();
        let data =
            RootInvolutionData::new(&roots, LatticeInvolution::identity(&datum).unwrap()).unwrap();
        let restricted = RestrictedRootSystem::build(&roots, &data).unwrap();
        assert_eq!(restricted.rank(), 0);
        assert!(restricted.roots().is_empty());
    }

    #[test]
    fn a2_restriction_keeps_the_nonreduced_pair_and_fibers() {
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let roots = RootSystem::enumerate(&datum, 6).unwrap();
        let involution = LatticeInvolution::new(
            &datum,
            vec![vec![0, -1], vec![-1, 0]],
            vec![vec![0, -1], vec![-1, 0]],
        )
        .unwrap();
        let data = RootInvolutionData::new(&roots, involution).unwrap();
        let restricted = RestrictedRootSystem::build(&roots, &data).unwrap();
        let lambda =
            RestrictedWeight::restrict(&Weight::new(vec![1, 0]), data.involution()).unwrap();
        assert_eq!(restricted.rank(), 1);
        assert_eq!(restricted.root(&lambda).unwrap().multiplicity(), 2);
        assert!(restricted.is_multipliable(&lambda).unwrap());
    }
}
