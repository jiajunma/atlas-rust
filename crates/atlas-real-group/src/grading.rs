use crate::{
    AdjointCartanFiber, AdjointFiberElement, CartanFiberElement, ModTwoSubspace, ModTwoVector,
    RootId, RootInvolutionData, RootSystem, StructureError,
};

/// A compactness grading of the simple-imaginary roots of one Cartan class.
///
/// Bit `i` belongs to the `i`-th entry of the owning model's simple-imaginary
/// root list, itself a copy of
/// [`RootInvolutionData::imaginary_simple_roots`] in this crate's
/// deterministic root order. A set bit means NONCOMPACT
/// (`gradings.cpp:20-23`).
///
/// This is deliberately distinct from the mod-two ambient coweight
/// coordinates of [`crate::CartanFiber`] and [`AdjointCartanFiber`]. Those
/// index lattice coordinates or simple roots of the full datum; a grading
/// indexes simple-imaginary positions only. The three spaces have equal
/// dimension in common cases such as A2 with the identity involution, so a
/// dimension check cannot separate them and the type must. The derived `Ord`
/// is an arbitrary deterministic map-key order, not a mathematical one.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Grading(ModTwoVector);

impl Grading {
    /// Build a grading marking the listed imaginary indices noncompact.
    ///
    /// Indices outside `imaginary_rank` are `IndexOutOfRange`.
    pub fn from_noncompact(
        imaginary_rank: usize,
        noncompact: impl IntoIterator<Item = usize>,
    ) -> Result<Self, StructureError> {
        ModTwoVector::from_ones(imaginary_rank, noncompact).map(Self)
    }

    pub fn imaginary_rank(&self) -> usize {
        self.0.dimension()
    }

    /// `None` when `imaginary_index >= imaginary_rank()`; `Some(true)` means
    /// noncompact.
    pub fn is_noncompact(&self, imaginary_index: usize) -> Option<bool> {
        self.0.bit(imaginary_index)
    }

    pub fn noncompact_indices(&self) -> impl Iterator<Item = usize> + '_ {
        (0..self.0.dimension()).filter(|&index| self.0.bit(index) == Some(true))
    }
}

/// Validated grading tables for one Cartan involution: `m_alpha`, adjoint
/// `m_alpha`, the base grading, and the grading shifts.
///
/// The base point (adjoint fiber element zero) grades every simple-imaginary
/// root noncompact by the quasisplit normalization; every other element's
/// grading is the affine-linear evaluation against its canonical ambient
/// representative. Construction rejects a nonfaithful shift matrix, so the
/// grading-to-element inverse is unique. `W_im` orbits live in
/// [`crate::WeakRealFormPartition`] and real-form labels in
/// [`crate::RealFormLabels`]; strong real forms are a later layer.
#[derive(Clone, Debug)]
pub struct CartanGradingData {
    imaginary_simple_roots: Vec<RootId>,
    simple_mod_two: Vec<ModTwoVector>,
    m_alphas: Vec<CartanFiberElement>,
    adjoint_m_alphas: Vec<AdjointFiberElement>,
    base_grading: Grading,
    grading_shifts: Vec<Grading>,
    adjoint: AdjointCartanFiber,
}

impl CartanGradingData {
    /// Build the grading tables from a validated root system, involution
    /// data, and adjoint Cartan fiber.
    ///
    /// The ambient fiber is deliberately not a parameter: value equality
    /// cannot express fiber identity, so `m_alpha` is built against
    /// [`AdjointCartanFiber::ambient_fiber`], the exact source the adjoint
    /// descent was proved against.
    pub fn build(
        root_system: &RootSystem,
        root_involution: &RootInvolutionData,
        adjoint: &AdjointCartanFiber,
    ) -> Result<Self, StructureError> {
        if root_system.datum() != root_involution.involution().datum() {
            return Err(StructureError::DatumMismatch);
        }
        if adjoint.ambient_fiber().involution() != root_involution.involution() {
            return Err(StructureError::CartanFiberInvolutionMismatch);
        }

        let imaginary = root_involution.imaginary_simple_roots();
        let imaginary_rank = imaginary.len();
        let semisimple_rank = root_system.datum().semisimple_rank();

        let mut imaginary_simple_roots = try_capacity(imaginary_rank)?;
        let mut simple_mod_two = try_capacity(imaginary_rank)?;
        let mut m_alphas = try_capacity(imaginary_rank)?;
        let mut adjoint_m_alphas = try_capacity(imaginary_rank)?;
        for &root_id in imaginary {
            let coroot = root_system
                .coroot(root_id)
                .ok_or(StructureError::IndexOutOfRange {
                    index: root_id.0,
                    upper_bound: root_system.roots().len(),
                })?;
            m_alphas.push(
                adjoint
                    .ambient_fiber()
                    .element_from_coweight_mod_two(coroot)?,
            );
            // `Pi(y)_j = <alpha_j, y>` is exactly the bracket vector of
            // adjoint m_alpha, so the pairing keeps its single
            // implementation inside the projection.
            let bound = adjoint.projection().source_coweight(coroot.clone())?;
            let image = adjoint.projection().map_coweight(&bound)?;
            adjoint_m_alphas.push(adjoint.element_from_coweight_mod_two(&image)?);

            let coordinates =
                root_system
                    .simple_coordinates(root_id)
                    .ok_or(StructureError::IndexOutOfRange {
                        index: root_id.0,
                        upper_bound: root_system.roots().len(),
                    })?;
            let mut odd = try_capacity(semisimple_rank)?;
            for (index, coordinate) in coordinates.iter().enumerate() {
                if *coordinate % 2 != 0 {
                    odd.push(index);
                }
            }
            simple_mod_two.push(ModTwoVector::from_ones(semisimple_rank, odd)?);
            imaginary_simple_roots.push(root_id);
        }

        let base_grading = Grading::from_noncompact(imaginary_rank, 0..imaginary_rank)?;

        let mut grading_shifts = try_capacity(adjoint.dimension())?;
        for representative in adjoint.basis_representatives() {
            let mut noncompact = try_capacity(imaginary_rank)?;
            for (imaginary_index, simple_two) in simple_mod_two.iter().enumerate() {
                if simple_two.dot(representative)? {
                    noncompact.push(imaginary_index);
                }
            }
            grading_shifts.push(Grading::from_noncompact(imaginary_rank, noncompact)?);
        }
        ensure_faithful_shifts(imaginary_rank, &grading_shifts)?;

        Ok(Self {
            imaginary_simple_roots,
            simple_mod_two,
            m_alphas,
            adjoint_m_alphas,
            base_grading,
            grading_shifts,
            adjoint: adjoint.clone(),
        })
    }

    pub fn imaginary_rank(&self) -> usize {
        self.imaginary_simple_roots.len()
    }

    pub fn imaginary_simple_roots(&self) -> &[RootId] {
        &self.imaginary_simple_roots
    }

    /// Bounded by `imaginary_rank()`.
    pub fn imaginary_simple_root(&self, imaginary_index: usize) -> Option<RootId> {
        self.imaginary_simple_roots.get(imaginary_index).copied()
    }

    /// The ambient-fiber class of the coroot of the `imaginary_index`-th
    /// simple-imaginary root, reduced mod two. Bounded by `imaginary_rank()`.
    pub fn m_alpha(&self, imaginary_index: usize) -> Option<&CartanFiberElement> {
        self.m_alphas.get(imaginary_index)
    }

    /// The adjoint-fiber class of the same coroot in fundamental-coweight
    /// coordinates. Bounded by `imaginary_rank()`.
    pub fn adjoint_m_alpha(&self, imaginary_index: usize) -> Option<&AdjointFiberElement> {
        self.adjoint_m_alphas.get(imaginary_index)
    }

    /// All-ones: the base point grades every simple-imaginary root noncompact.
    pub fn base_grading(&self) -> &Grading {
        &self.base_grading
    }

    /// Bounded by `adjoint_fiber().dimension()`.
    pub fn grading_shift(&self, adjoint_basis_index: usize) -> Option<&Grading> {
        self.grading_shifts.get(adjoint_basis_index)
    }

    /// The validated adjoint fiber these tables were computed against.
    pub fn adjoint_fiber(&self) -> &AdjointCartanFiber {
        &self.adjoint
    }

    /// The compactness grading of one adjoint fiber element.
    pub fn grading(&self, element: &AdjointFiberElement) -> Result<Grading, StructureError> {
        let representative = self.adjoint.canonical_representative(element)?;
        let mut noncompact = try_capacity(self.imaginary_rank())?;
        for (imaginary_index, simple_two) in self.simple_mod_two.iter().enumerate() {
            if !simple_two.dot(&representative)? {
                noncompact.push(imaginary_index);
            }
        }
        Grading::from_noncompact(self.imaginary_rank(), noncompact)
    }

    /// The unique adjoint fiber element with the requested grading.
    ///
    /// Uniqueness is the faithfulness invariant checked at construction; a
    /// grading outside the affine image is [`StructureError::ImpossibleGrading`].
    pub fn element_from_grading(
        &self,
        target: &Grading,
    ) -> Result<AdjointFiberElement, StructureError> {
        let imaginary_rank = self.imaginary_rank();
        if target.imaginary_rank() != imaginary_rank {
            return Err(StructureError::RankMismatch {
                expected: imaginary_rank,
                actual: target.imaginary_rank(),
            });
        }
        let dimension = self.grading_shifts.len();
        let augmented = imaginary_rank
            .checked_add(dimension)
            .ok_or(StructureError::ArithmeticOverflow)?;

        // Augmented elimination: each column carries its grading bits plus a
        // marker bit recording which shift it is, so reducing the right-hand
        // side also accumulates the solving combination.
        let mut span = ModTwoSubspace::new(augmented)?;
        for (adjoint_basis_index, shift) in self.grading_shifts.iter().enumerate() {
            let mut ones = try_capacity(augmented)?;
            ones.extend(shift.noncompact_indices());
            ones.push(imaginary_rank + adjoint_basis_index);
            span.insert(ModTwoVector::from_ones(augmented, ones)?)?;
        }

        // Right-hand side: target XOR base. The base is all ones, so the
        // difference marks the compact positions of the target.
        let mut difference = try_capacity(imaginary_rank)?;
        for imaginary_index in 0..imaginary_rank {
            if target.is_noncompact(imaginary_index) == Some(false) {
                difference.push(imaginary_index);
            }
        }
        let remainder =
            span.quotient_representative(ModTwoVector::from_ones(augmented, difference)?)?;
        if (0..imaginary_rank).any(|index| remainder.bit(index) == Some(true)) {
            return Err(StructureError::ImpossibleGrading);
        }

        let ambient_rank = self.adjoint.datum().rank();
        let mut representative = ModTwoVector::zero(ambient_rank)?;
        for (adjoint_basis_index, basis_representative) in
            self.adjoint.basis_representatives().iter().enumerate()
        {
            if remainder.bit(imaginary_rank + adjoint_basis_index) == Some(true) {
                representative.xor_assign(basis_representative)?;
            }
        }
        self.adjoint.element_from_ambient(representative)
    }
}

/// The faithfulness gate: shift columns must be linearly independent so
/// gradings determine adjoint fiber elements uniquely. Atlas asserts this
/// (`cartanclass.cpp:172`); here it is an unconditional rejection. No public
/// constructor is known to reach a dependent column, so the check is
/// defensive.
fn ensure_faithful_shifts(imaginary_rank: usize, shifts: &[Grading]) -> Result<(), StructureError> {
    let mut span = ModTwoSubspace::new(imaginary_rank)?;
    for shift in shifts {
        if !span.insert(shift.0.clone())? {
            return Err(StructureError::GradingShiftsNotFaithful);
        }
    }
    Ok(())
}

pub(crate) fn try_capacity<T>(capacity: usize) -> Result<Vec<T>, StructureError> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(capacity)
        .map_err(|_| StructureError::AllocationFailed {
            requested: capacity,
        })?;
    Ok(values)
}

#[cfg(test)]
mod tests {
    use crate::integer_lattice::IntegerLatticeBudget;
    use crate::{
        AdjointFiberBudget, BasedRootDatum, CartanFiber, Coweight, LatticeInvolution, Weight,
    };

    use super::*;

    fn integer_budget() -> IntegerLatticeBudget {
        IntegerLatticeBudget::new(64, 100_000, 100_000, 128)
    }

    fn adjoint_budget() -> AdjointFiberBudget {
        AdjointFiberBudget::new(integer_budget(), 50_000, 100_000)
    }

    fn build_chain(
        datum: &BasedRootDatum,
        involution: LatticeInvolution,
        max_roots: usize,
    ) -> (RootSystem, RootInvolutionData, AdjointCartanFiber) {
        let roots = RootSystem::enumerate(datum, max_roots).unwrap();
        let data = RootInvolutionData::new(&roots, involution.clone()).unwrap();
        let source = CartanFiber::build(&involution, &integer_budget()).unwrap();
        let adjoint = AdjointCartanFiber::build(&roots, &data, &source, &adjoint_budget()).unwrap();
        (roots, data, adjoint)
    }

    fn ambient(rank: usize, indices: impl IntoIterator<Item = usize>) -> ModTwoVector {
        ModTwoVector::from_ones(rank, indices).unwrap()
    }

    #[test]
    fn simply_connected_a1_realizes_the_quasisplit_normalization() {
        let datum = BasedRootDatum::from_simple_data(
            1,
            vec![vec![2]],
            vec![Weight::new(vec![2])],
            vec![Coweight::new(vec![1])],
        )
        .unwrap();
        let involution = LatticeInvolution::identity(&datum).unwrap();
        let (roots, data, adjoint) = build_chain(&datum, involution, 2);
        let grading_data = CartanGradingData::build(&roots, &data, &adjoint).unwrap();

        assert_eq!(grading_data.imaginary_rank(), 1);
        assert_eq!(adjoint.dimension(), 1);
        assert!(!grading_data.m_alpha(0).unwrap().is_identity());
        assert!(grading_data.adjoint_m_alpha(0).unwrap().is_identity());
        assert_eq!(
            grading_data.grading_shift(0).unwrap(),
            &Grading::from_noncompact(1, [0]).unwrap()
        );

        let base_point = adjoint.identity().unwrap();
        let noncompact = grading_data.grading(&base_point).unwrap();
        assert_eq!(noncompact.is_noncompact(0), Some(true));
        let other = adjoint.element_from_ambient(ambient(1, [0])).unwrap();
        let compact = grading_data.grading(&other).unwrap();
        assert_eq!(compact.is_noncompact(0), Some(false));

        assert_eq!(
            grading_data.element_from_grading(&noncompact).unwrap(),
            base_point
        );
        assert_eq!(grading_data.element_from_grading(&compact).unwrap(), other);
    }

    #[test]
    fn a2_identity_gradings_are_a_bijection() {
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let involution = LatticeInvolution::identity(&datum).unwrap();
        let (roots, data, adjoint) = build_chain(&datum, involution, 6);
        let grading_data = CartanGradingData::build(&roots, &data, &adjoint).unwrap();

        assert_eq!(grading_data.imaginary_rank(), 2);
        assert_eq!(
            grading_data.imaginary_simple_roots(),
            &[
                roots.id_of(&Weight::new(vec![0, 1])).unwrap(),
                roots.id_of(&Weight::new(vec![1, 0])).unwrap(),
            ]
        );
        // Index 0 is alpha_2, whose coroot is (-1, 2); index 1 is alpha_1.
        assert_eq!(
            adjoint
                .canonical_representative(grading_data.adjoint_m_alpha(0).unwrap())
                .unwrap(),
            ambient(2, [0])
        );
        assert_eq!(
            adjoint
                .canonical_representative(grading_data.adjoint_m_alpha(1).unwrap())
                .unwrap(),
            ambient(2, [1])
        );
        // Shift columns form a permutation matrix in this crate's root order.
        assert_eq!(
            grading_data.grading_shift(0).unwrap(),
            &Grading::from_noncompact(2, [1]).unwrap()
        );
        assert_eq!(
            grading_data.grading_shift(1).unwrap(),
            &Grading::from_noncompact(2, [0]).unwrap()
        );

        let mut seen = Vec::new();
        for indices in [vec![], vec![0], vec![1], vec![0, 1]] {
            let element = adjoint.element_from_ambient(ambient(2, indices)).unwrap();
            let grading = grading_data.grading(&element).unwrap();
            assert_eq!(
                grading_data.element_from_grading(&grading).unwrap(),
                element
            );
            assert!(!seen.contains(&grading));
            seen.push(grading);
        }
    }

    #[test]
    fn a2_twist_rejects_the_all_compact_grading() {
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let twist = LatticeInvolution::new(
            &datum,
            vec![vec![0, 1], vec![1, 0]],
            vec![vec![0, 1], vec![1, 0]],
        )
        .unwrap();
        let (roots, data, adjoint) = build_chain(&datum, twist, 6);
        let grading_data = CartanGradingData::build(&roots, &data, &adjoint).unwrap();

        assert_eq!(grading_data.imaginary_rank(), 1);
        assert_eq!(
            grading_data.imaginary_simple_root(0),
            roots.id_of(&Weight::new(vec![1, 1]))
        );
        assert_eq!(adjoint.dimension(), 0);
        assert!(grading_data.grading_shift(0).is_none());
        assert!(grading_data.m_alpha(0).unwrap().is_identity());

        let base_point = adjoint.identity().unwrap();
        let noncompact = grading_data.grading(&base_point).unwrap();
        assert_eq!(noncompact, *grading_data.base_grading());
        assert_eq!(
            grading_data.element_from_grading(&noncompact).unwrap(),
            base_point
        );
        assert_eq!(
            grading_data.element_from_grading(&Grading::from_noncompact(1, []).unwrap()),
            Err(StructureError::ImpossibleGrading)
        );
    }

    #[test]
    fn a1_x_a1_swap_has_imaginary_rank_zero() {
        let datum = BasedRootDatum::standard(vec![vec![2, 0], vec![0, 2]]).unwrap();
        let swap = LatticeInvolution::new(
            &datum,
            vec![vec![0, 1], vec![1, 0]],
            vec![vec![0, 1], vec![1, 0]],
        )
        .unwrap();
        let (roots, data, adjoint) = build_chain(&datum, swap, 4);
        let grading_data = CartanGradingData::build(&roots, &data, &adjoint).unwrap();

        assert_eq!(grading_data.imaginary_rank(), 0);
        assert!(grading_data.imaginary_simple_root(0).is_none());
        assert!(grading_data.m_alpha(0).is_none());
        assert_eq!(grading_data.base_grading().imaginary_rank(), 0);

        let identity = adjoint.identity().unwrap();
        let empty = grading_data.grading(&identity).unwrap();
        assert_eq!(empty.imaginary_rank(), 0);
        assert_eq!(grading_data.element_from_grading(&empty).unwrap(), identity);
    }

    #[test]
    fn a_central_coweight_coordinate_distinguishes_m_alpha_from_its_adjoint_image() {
        let datum = BasedRootDatum::from_simple_data(
            2,
            vec![vec![2]],
            vec![Weight::new(vec![1, 0])],
            vec![Coweight::new(vec![2, 1])],
        )
        .unwrap();
        let involution = LatticeInvolution::identity(&datum).unwrap();
        let (roots, data, adjoint) = build_chain(&datum, involution, 2);
        let grading_data = CartanGradingData::build(&roots, &data, &adjoint).unwrap();

        let m_alpha = grading_data.m_alpha(0).unwrap();
        assert!(!m_alpha.is_identity());
        assert_eq!(
            adjoint
                .ambient_fiber()
                .canonical_representative(m_alpha)
                .unwrap(),
            ambient(2, [1])
        );
        assert!(grading_data.adjoint_m_alpha(0).unwrap().is_identity());
    }

    #[test]
    fn b2_reduces_negative_coroot_coordinates_with_sign_safe_parity() {
        let datum = BasedRootDatum::standard(vec![vec![2, -2], vec![-1, 2]]).unwrap();
        let involution = LatticeInvolution::identity(&datum).unwrap();
        let (roots, data, adjoint) = build_chain(&datum, involution, 8);
        let grading_data = CartanGradingData::build(&roots, &data, &adjoint).unwrap();

        // Imaginary index 1 is alpha_1 = (1, 0), whose coroot (2, -1) must
        // reduce to (0, 1): the negative coordinate is odd.
        assert_eq!(
            grading_data.imaginary_simple_root(1),
            roots.id_of(&Weight::new(vec![1, 0]))
        );
        assert_eq!(
            adjoint
                .ambient_fiber()
                .canonical_representative(grading_data.m_alpha(1).unwrap())
                .unwrap(),
            ambient(2, [1])
        );
        assert_eq!(
            adjoint
                .canonical_representative(grading_data.adjoint_m_alpha(1).unwrap())
                .unwrap(),
            ambient(2, [1])
        );
    }

    #[test]
    fn rejects_foreign_inputs_before_computing_tables() {
        let a2 = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let b2 = BasedRootDatum::standard(vec![vec![2, -2], vec![-1, 2]]).unwrap();
        let identity = LatticeInvolution::identity(&a2).unwrap();
        let (a2_roots, a2_data, a2_adjoint) = build_chain(&a2, identity, 6);
        let (b2_roots, _, _) = build_chain(&b2, LatticeInvolution::identity(&b2).unwrap(), 8);

        assert!(matches!(
            CartanGradingData::build(&b2_roots, &a2_data, &a2_adjoint),
            Err(StructureError::DatumMismatch)
        ));

        let twist = LatticeInvolution::new(
            &a2,
            vec![vec![0, 1], vec![1, 0]],
            vec![vec![0, 1], vec![1, 0]],
        )
        .unwrap();
        let twist_data = RootInvolutionData::new(&a2_roots, twist).unwrap();
        assert!(matches!(
            CartanGradingData::build(&a2_roots, &twist_data, &a2_adjoint),
            Err(StructureError::CartanFiberInvolutionMismatch)
        ));

        let grading_data = CartanGradingData::build(&a2_roots, &a2_data, &a2_adjoint).unwrap();
        let (_, _, foreign_adjoint) =
            build_chain(&a2, LatticeInvolution::identity(&a2).unwrap(), 6);
        let foreign_element = foreign_adjoint.identity().unwrap();
        assert_eq!(
            grading_data.grading(&foreign_element),
            Err(StructureError::CartanFiberMismatch)
        );
    }

    #[test]
    fn an_injected_dependent_shift_column_is_rejected() {
        let duplicated = Grading::from_noncompact(2, [0]).unwrap();
        assert_eq!(
            ensure_faithful_shifts(2, &[duplicated.clone(), duplicated]),
            Err(StructureError::GradingShiftsNotFaithful)
        );
        assert_eq!(
            ensure_faithful_shifts(1, &[Grading::from_noncompact(1, []).unwrap()]),
            Err(StructureError::GradingShiftsNotFaithful)
        );
    }

    #[test]
    fn thirty_three_a1_factors_stay_dynamic() {
        let rank = 33;
        let mut cartan = vec![vec![0; rank]; rank];
        for (index, row) in cartan.iter_mut().enumerate() {
            row[index] = 2;
        }
        let datum = BasedRootDatum::standard(cartan).unwrap();
        let involution = LatticeInvolution::identity(&datum).unwrap();
        let (roots, data, adjoint) = build_chain(&datum, involution, 2 * rank);
        let grading_data = CartanGradingData::build(&roots, &data, &adjoint).unwrap();

        assert_eq!(grading_data.imaginary_rank(), rank);
        assert_eq!(adjoint.dimension(), rank);
        let base_point = adjoint.identity().unwrap();
        let grading = grading_data.grading(&base_point).unwrap();
        assert_eq!(grading.noncompact_indices().count(), rank);
        assert_eq!(
            grading_data.element_from_grading(&grading).unwrap(),
            base_point
        );
        let flipped = adjoint.element_from_ambient(ambient(rank, [7])).unwrap();
        let flipped_grading = grading_data.grading(&flipped).unwrap();
        assert_eq!(
            grading_data.element_from_grading(&flipped_grading).unwrap(),
            flipped
        );
    }
}
