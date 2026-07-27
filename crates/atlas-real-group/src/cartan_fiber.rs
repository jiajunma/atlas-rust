use std::sync::Arc;

use crate::mod_two::{ModTwoAmbientMap, ModTwoSubquotient};
use crate::{
    integer_lattice::{negative_coweight_eigenspace, reduce_basis_mod_two, IntegerLatticeBudget},
    Coweight, LatticeInvolution, ModTwoSubspace, ModTwoVector, StructureError,
};

/// The primary finite component group attached to a Cartan involution.
///
/// Atlas C++ source analysis describes a subquotient of `Y / 2Y`, where `Y` is
/// the cocharacter lattice. This structural layer uses that documented formula
/// as its current convention:
///
/// ```text
/// ker_F2(I + theta_Y) / red_2 ker_Z(I + theta_Y).
/// ```
///
/// The abstract quotient `Y^theta / (I + theta_Y)Y` is isomorphic, but it has
/// different natural coordinates.  Adjoint maps live in
/// [`crate::AdjointCartanFiber`] and [`crate::FiberToAdjoint`], gradings in
/// [`crate::CartanGradingData`], and the weak-real-form partition in
/// [`crate::WeakRealFormPartition`]; strong real forms and KGB data still
/// belong to later layers.
#[derive(Clone, Debug)]
pub struct CartanFiber {
    model: Arc<CartanFiberModel>,
}

#[derive(Debug)]
struct CartanFiberModel {
    involution: LatticeInvolution,
    quotient: ModTwoSubquotient,
}

/// An opaque element of one [`CartanFiber`].
///
/// Coordinates are retained in the fiber's canonical basis.  The shared model
/// prevents accidentally combining equal-rank elements that came from
/// different Cartan involutions.  Opacity means provenance, not secrecy:
/// coordinate views (`canonical_representative`, `coordinates`) are served
/// only by the owning fiber, against its documented basis convention.
#[derive(Clone, Debug)]
pub struct CartanFiberElement {
    model: Arc<CartanFiberModel>,
    coordinates: ModTwoVector,
}

impl PartialEq for CartanFiberElement {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.model, &other.model) && self.coordinates == other.coordinates
    }
}

impl Eq for CartanFiberElement {}

impl CartanFiber {
    /// Build this layer's low-pivot packed fiber group from a pairing-preserving
    /// dual-lattice involution. The exact integral reduction is bounded by
    /// `budget`.
    pub fn build(
        involution: &LatticeInvolution,
        budget: &IntegerLatticeBudget,
    ) -> Result<Self, StructureError> {
        Self::build_owned(involution.clone(), budget)
    }

    pub(crate) fn build_owned(
        involution: LatticeInvolution,
        budget: &IntegerLatticeBudget,
    ) -> Result<Self, StructureError> {
        // Compute the integral denominator first, so its caller-owned resource
        // budget is enforced before allocating the finite-field numerator.
        let negative_eigenspace =
            negative_coweight_eigenspace(involution.coweight_matrix(), budget)?;
        let denominator = reduce_basis_mod_two(&negative_eigenspace)?;
        let numerator = mod_two_i_plus_coweight_kernel(&involution)?;
        let quotient = ModTwoSubquotient::new(numerator, denominator)?;

        Ok(Self {
            model: Arc::new(CartanFiberModel {
                involution,
                quotient,
            }),
        })
    }

    pub fn lattice_rank(&self) -> usize {
        self.model.quotient.ambient_dimension()
    }

    /// Dimension over `F_2`; this never enumerates `2^dimension` elements.
    pub fn dimension(&self) -> usize {
        self.model.quotient.dimension()
    }

    pub fn involution(&self) -> &LatticeInvolution {
        &self.model.involution
    }

    pub fn identity(&self) -> Result<CartanFiberElement, StructureError> {
        Ok(CartanFiberElement {
            model: Arc::clone(&self.model),
            coordinates: ModTwoVector::zero(self.dimension())?,
        })
    }

    /// Interpret an ambient vector in `Y / 2Y` as a fiber element.
    ///
    /// The vector must lie in the numerator `ker_F2(I + theta_Y)`.
    pub fn element_from_ambient(
        &self,
        representative: ModTwoVector,
    ) -> Result<CartanFiberElement, StructureError> {
        Ok(CartanFiberElement {
            model: Arc::clone(&self.model),
            coordinates: self.model.quotient.to_coordinates(representative)?,
        })
    }

    /// Reduce a cocharacter modulo two before interpreting it as a fiber
    /// representative. This does not assert that the cocharacter is exactly
    /// theta-fixed; the packed numerator is the mod-two condition.
    pub fn element_from_coweight_mod_two(
        &self,
        coweight: &Coweight,
    ) -> Result<CartanFiberElement, StructureError> {
        if coweight.rank() != self.lattice_rank() {
            return Err(StructureError::RankMismatch {
                expected: self.lattice_rank(),
                actual: coweight.rank(),
            });
        }
        let mut odd_coordinates = Vec::new();
        odd_coordinates
            .try_reserve_exact(coweight.rank())
            .map_err(|_| StructureError::AllocationFailed {
                requested: coweight.rank(),
            })?;
        for (index, coordinate) in coweight.as_slice().iter().enumerate() {
            if *coordinate % 2 != 0 {
                odd_coordinates.push(index);
            }
        }
        self.element_from_ambient(ModTwoVector::from_ones(coweight.rank(), odd_coordinates)?)
    }

    /// Return this implementation's deterministic ambient representative in
    /// `Y / 2Y`, using the documented low-pivot reduction convention.
    pub fn canonical_representative(
        &self,
        element: &CartanFiberElement,
    ) -> Result<ModTwoVector, StructureError> {
        self.ensure_member(element)?;
        self.model
            .quotient
            .ambient_representative(&element.coordinates)
    }

    /// The element's coordinates on this fiber's canonical subquotient basis.
    ///
    /// Bit `j` selects `basis_representatives()[j]`; the XOR of the selected
    /// representatives is exactly [`Self::canonical_representative`]. This
    /// exposes the same documented low-pivot basis data that representative
    /// already determines — no new coordinate commitment.
    pub fn coordinates<'e>(
        &self,
        element: &'e CartanFiberElement,
    ) -> Result<&'e ModTwoVector, StructureError> {
        self.ensure_member(element)?;
        Ok(&element.coordinates)
    }

    /// Add two fiber elements in canonical coordinates.
    pub fn add(
        &self,
        left: &CartanFiberElement,
        right: &CartanFiberElement,
    ) -> Result<CartanFiberElement, StructureError> {
        self.ensure_member(left)?;
        self.ensure_member(right)?;
        let mut coordinates = left.coordinates.clone();
        coordinates.xor_assign(&right.coordinates)?;
        Ok(CartanFiberElement {
            model: Arc::clone(&self.model),
            coordinates,
        })
    }

    pub fn same_class(
        &self,
        left: &CartanFiberElement,
        right: &CartanFiberElement,
    ) -> Result<bool, StructureError> {
        self.ensure_member(left)?;
        self.ensure_member(right)?;
        Ok(left.coordinates == right.coordinates)
    }

    /// Deterministic ambient representatives of the quotient basis generators.
    pub fn basis_representatives(&self) -> &[ModTwoVector] {
        self.model.quotient.basis_representatives()
    }

    /// Verify that an ambient map respects both subquotient relations.
    ///
    /// Higher layers may apply such a map on demand after this proof, avoiding
    /// a cached dense map in quotient coordinates.
    pub(crate) fn validate_induced_map(
        &self,
        target: &Self,
        ambient_map: &impl ModTwoAmbientMap,
    ) -> Result<(), StructureError> {
        self.model
            .quotient
            .validate_induced_map_to(&target.model.quotient, ambient_map)?;
        Ok(())
    }

    fn ensure_member(&self, element: &CartanFiberElement) -> Result<(), StructureError> {
        if Arc::ptr_eq(&self.model, &element.model) {
            Ok(())
        } else {
            Err(StructureError::CartanFiberMismatch)
        }
    }
}

impl CartanFiberElement {
    pub fn is_identity(&self) -> bool {
        self.coordinates.is_zero()
    }
}

fn mod_two_i_plus_coweight_kernel(
    involution: &LatticeInvolution,
) -> Result<ModTwoSubspace, StructureError> {
    let rank = involution.lattice_rank();
    let mut equations = ModTwoSubspace::new(rank)?;
    for (row_index, row) in involution.coweight_matrix().iter().enumerate() {
        if row.len() != rank {
            return Err(StructureError::InvalidInvolution);
        }
        let capacity = rank
            .checked_add(1)
            .ok_or(StructureError::ArithmeticOverflow)?;
        let mut ones = Vec::new();
        ones.try_reserve_exact(capacity)
            .map_err(|_| StructureError::AllocationFailed {
                requested: capacity,
            })?;
        for (column_index, entry) in row.iter().enumerate() {
            if *entry % 2 != 0 {
                ones.push(column_index);
            }
        }
        // `from_ones` xor-toggles duplicates, so adding this index implements
        // the diagonal `+ I` term even when theta already has an odd entry.
        ones.push(row_index);
        equations.insert(ModTwoVector::from_ones(rank, ones)?)?;
    }
    equations.right_kernel()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mod_two::ModTwoLinearMap;
    use crate::BasedRootDatum;

    fn budget() -> IntegerLatticeBudget {
        IntegerLatticeBudget::new(8, 128, 1_000, 128)
    }

    fn central_torus(rank: usize) -> BasedRootDatum {
        BasedRootDatum::from_simple_data(rank, vec![], vec![], vec![]).unwrap()
    }

    fn ambient(rank: usize, indices: impl IntoIterator<Item = usize>) -> ModTwoVector {
        ModTwoVector::from_ones(rank, indices).unwrap()
    }

    #[test]
    fn identity_involution_keeps_the_full_mod_two_torus() {
        let datum = central_torus(2);
        let fiber =
            CartanFiber::build(&LatticeInvolution::identity(&datum).unwrap(), &budget()).unwrap();
        let first = fiber.element_from_ambient(ambient(2, [0])).unwrap();
        let second = fiber.element_from_ambient(ambient(2, [1])).unwrap();
        let sum = fiber.add(&first, &second).unwrap();

        assert_eq!(fiber.dimension(), 2);
        assert!(!first.is_identity());
        assert_eq!(
            fiber.canonical_representative(&sum).unwrap(),
            ambient(2, [0, 1])
        );
        assert_eq!(
            fiber.basis_representatives(),
            &[ambient(2, [0]), ambient(2, [1])]
        );
    }

    #[test]
    fn negative_identity_has_a_trivial_fiber() {
        let datum = central_torus(1);
        let involution = LatticeInvolution::new(&datum, vec![vec![-1]], vec![vec![-1]]).unwrap();
        let fiber = CartanFiber::build(&involution, &budget()).unwrap();
        let element = fiber.element_from_ambient(ambient(1, [0])).unwrap();

        assert_eq!(fiber.dimension(), 0);
        assert!(element.is_identity());
        assert_eq!(
            fiber.canonical_representative(&element).unwrap(),
            ambient(1, [])
        );
    }

    #[test]
    fn swap_involution_has_no_component_group() {
        let datum = central_torus(2);
        let involution = LatticeInvolution::new(
            &datum,
            vec![vec![0, 1], vec![1, 0]],
            vec![vec![0, 1], vec![1, 0]],
        )
        .unwrap();
        let fiber = CartanFiber::build(&involution, &budget()).unwrap();

        assert_eq!(fiber.dimension(), 0);
        assert!(fiber
            .element_from_ambient(ambient(2, [0, 1]))
            .unwrap()
            .is_identity());
        assert_eq!(
            fiber.element_from_ambient(ambient(2, [0])),
            Err(StructureError::NotInModTwoSubspace)
        );
    }

    #[test]
    fn uses_the_stored_coweight_action_and_negative_eigenlattice() {
        let datum = central_torus(2);
        // The dual actions satisfy weight^T * coweight = I.  The nonzero
        // off-diagonal entry also makes red_2 ker(I+theta_Y) differ from the
        // raw mod-two image of I+theta_Y, which would give the wrong rank.
        let involution = LatticeInvolution::new(
            &datum,
            vec![vec![1, 0], vec![2, -1]],
            vec![vec![1, 2], vec![0, -1]],
        )
        .unwrap();
        let fiber = CartanFiber::build(&involution, &budget()).unwrap();
        let first = fiber.element_from_ambient(ambient(2, [0])).unwrap();
        let second = fiber
            .element_from_coweight_mod_two(&Coweight::new(vec![0, 1]))
            .unwrap();

        assert_eq!(fiber.dimension(), 1);
        assert_eq!(first, second);
        assert!(!second.is_identity());
        assert!(fiber.add(&first, &second).unwrap().is_identity());
        assert_eq!(fiber.basis_representatives(), &[ambient(2, [1])]);
    }

    #[test]
    fn uses_coweight_rows_for_the_mod_two_numerator_without_transposing() {
        let datum = central_torus(3);
        // theta_Y is non-symmetric modulo two. Its dual weight action is its
        // transpose, so a second transpose in the numerator would exchange
        // the e0 and e1 tests below.  Here ker_F2(I + theta_Y) is <e0, e2>,
        // while red_2 ker_Z(I + theta_Y) is <e0>.
        let involution = LatticeInvolution::new(
            &datum,
            vec![vec![1, 0, 0], vec![1, -1, 0], vec![0, 0, 1]],
            vec![vec![1, 1, 0], vec![0, -1, 0], vec![0, 0, 1]],
        )
        .unwrap();
        let fiber = CartanFiber::build(&involution, &budget()).unwrap();
        let denominator = fiber.element_from_ambient(ambient(3, [0])).unwrap();
        let generator = fiber.element_from_ambient(ambient(3, [2])).unwrap();

        assert_eq!(fiber.dimension(), 1);
        assert!(denominator.is_identity());
        assert!(!generator.is_identity());
        assert_eq!(fiber.basis_representatives(), &[ambient(3, [2])]);
        assert_eq!(
            fiber.element_from_ambient(ambient(3, [1])),
            Err(StructureError::NotInModTwoSubspace)
        );
    }

    #[test]
    fn rejects_elements_from_a_different_fiber() {
        let datum = central_torus(1);
        let first =
            CartanFiber::build(&LatticeInvolution::identity(&datum).unwrap(), &budget()).unwrap();
        let second =
            CartanFiber::build(&LatticeInvolution::identity(&datum).unwrap(), &budget()).unwrap();
        let left = first.element_from_ambient(ambient(1, [0])).unwrap();
        let right = second.element_from_ambient(ambient(1, [0])).unwrap();

        assert_eq!(
            first.add(&left, &right),
            Err(StructureError::CartanFiberMismatch)
        );
    }

    #[test]
    fn rejects_ambient_maps_that_do_not_descend_through_both_relations() {
        let datum = central_torus(2);
        let identity =
            CartanFiber::build(&LatticeInvolution::identity(&datum).unwrap(), &budget()).unwrap();
        let swap_involution = LatticeInvolution::new(
            &datum,
            vec![vec![0, 1], vec![1, 0]],
            vec![vec![0, 1], vec![1, 0]],
        )
        .unwrap();
        let swap = CartanFiber::build(&swap_involution, &budget()).unwrap();
        let identity_map = ModTwoLinearMap::new(2, vec![ambient(2, [0]), ambient(2, [1])]).unwrap();

        assert!(matches!(
            identity.validate_induced_map(&swap, &identity_map),
            Err(StructureError::CartanFiberMapDoesNotDescend {
                relation: "numerator"
            })
        ));
        assert!(matches!(
            swap.validate_induced_map(&identity, &identity_map),
            Err(StructureError::CartanFiberMapDoesNotDescend {
                relation: "denominator"
            })
        ));
    }

    #[test]
    fn validates_coweight_rank_before_allocating_its_mod_two_image() {
        let datum = central_torus(1);
        let fiber =
            CartanFiber::build(&LatticeInvolution::identity(&datum).unwrap(), &budget()).unwrap();

        assert_eq!(
            fiber.element_from_coweight_mod_two(&Coweight::new(vec![0, 0])),
            Err(StructureError::RankMismatch {
                expected: 1,
                actual: 2,
            })
        );
        assert_eq!(
            fiber.element_from_ambient(ambient(2, [0])),
            Err(StructureError::RankMismatch {
                expected: 1,
                actual: 2,
            })
        );
    }

    #[test]
    fn respects_the_caller_owned_exact_lattice_budget() {
        let datum = central_torus(2);
        assert!(matches!(
            CartanFiber::build(
                &LatticeInvolution::identity(&datum).unwrap(),
                &IntegerLatticeBudget::new(1, 4, 8, 8),
            ),
            Err(StructureError::IntegerLatticeResourceLimit {
                resource: "rank",
                limit: 1,
            })
        ));
    }
}
