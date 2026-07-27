use std::sync::Arc;

use crate::mod_two::ModTwoAmbientMap;
use crate::{
    pair, BasedRootDatum, CartanFiber, CartanFiberElement, Coweight, IntegerLatticeBudget,
    LatticeInvolution, ModTwoVector, RootInvolutionData, RootSystem, StructureError, Weight,
};

const ADJOINT_PERSISTENT_SQUARES: usize = 16;

/// Caller-owned resource bounds for an adjoint Cartan-fiber construction.
///
/// These limits bound one computation; they are not a mathematical rank cap.
/// The integral reduction receives its own explicit budget because it owns
/// Malachite intermediates. The two remaining limits cover retained structural
/// coordinates and the direct restriction work used to prove quotient descent.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdjointFiberBudget {
    integer_lattice: IntegerLatticeBudget,
    max_persistent_entries: usize,
    max_projection_operations: usize,
}

impl AdjointFiberBudget {
    pub const fn new(
        integer_lattice: IntegerLatticeBudget,
        max_persistent_entries: usize,
        max_projection_operations: usize,
    ) -> Self {
        Self {
            integer_lattice,
            max_persistent_entries,
            max_projection_operations,
        }
    }

    /// Bounds exact integral work while building the adjoint Cartan fiber.
    pub fn integer_lattice_budget(&self) -> &IntegerLatticeBudget {
        &self.integer_lattice
    }
}

/// The based root datum of the adjoint semisimple quotient.
///
/// Its character basis is the full simple-root basis of the source datum, and
/// its cocharacter basis is the corresponding fundamental-coweight basis.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdjointBasedRootDatum {
    datum: BasedRootDatum,
}

impl AdjointBasedRootDatum {
    fn from_source(source: &BasedRootDatum) -> Result<Self, StructureError> {
        Ok(Self {
            datum: BasedRootDatum::standard(clone_i32_matrix(source.cartan_matrix())?)?,
        })
    }

    pub fn as_based_root_datum(&self) -> &BasedRootDatum {
        &self.datum
    }

    pub fn rank(&self) -> usize {
        self.datum.lattice_rank()
    }
}

#[derive(Debug)]
struct AdjointProjectionModel {
    source_lattice_rank: usize,
    source_simple_roots: Vec<Weight>,
    target_datum: AdjointBasedRootDatum,
    max_projection_operations: usize,
}

/// A coweight bound to the source lattice of one [`AdjointProjection`].
///
/// A raw [`Coweight`] has coordinates but no datum provenance. Binding records
/// the caller's assertion that those coordinates use this source datum, then
/// prevents the resulting value from being reused through another projection.
#[derive(Clone, Debug)]
pub struct AmbientCoweight {
    model: Arc<AdjointProjectionModel>,
    coordinates: Coweight,
}

impl AmbientCoweight {
    pub fn as_coweight(&self) -> &Coweight {
        &self.coordinates
    }
}

impl PartialEq for AmbientCoweight {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.model, &other.model) && self.coordinates == other.coordinates
    }
}

impl Eq for AmbientCoweight {}

/// A coweight in the adjoint fundamental-coweight lattice.
#[derive(Clone, Debug)]
pub struct AdjointCoweight {
    model: Arc<AdjointProjectionModel>,
    coordinates: Coweight,
}

impl AdjointCoweight {
    pub fn datum(&self) -> &AdjointBasedRootDatum {
        &self.model.target_datum
    }

    pub fn as_coweight(&self) -> &Coweight {
        &self.coordinates
    }
}

impl PartialEq for AdjointCoweight {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.model, &other.model) && self.coordinates == other.coordinates
    }
}

impl Eq for AdjointCoweight {}

/// The restriction map from the ambient cocharacter lattice to the adjoint one.
///
/// For `y` in the source coweight lattice, its target coordinates are the
/// pairings with the source simple roots. This map can have a central kernel
/// and is not presented as an isomorphism.
#[derive(Clone, Debug)]
pub struct AdjointProjection {
    model: Arc<AdjointProjectionModel>,
}

impl AdjointProjection {
    fn from_source(
        source: &BasedRootDatum,
        max_projection_operations: usize,
    ) -> Result<Self, StructureError> {
        Ok(Self {
            model: Arc::new(AdjointProjectionModel {
                source_lattice_rank: source.lattice_rank(),
                source_simple_roots: clone_weights(source.simple_roots())?,
                target_datum: AdjointBasedRootDatum::from_source(source)?,
                max_projection_operations,
            }),
        })
    }

    pub fn source_lattice_rank(&self) -> usize {
        self.model.source_lattice_rank
    }

    pub fn target_datum(&self) -> &AdjointBasedRootDatum {
        &self.model.target_datum
    }

    /// Bind source coordinates to this projection's source datum.
    ///
    /// The caller supplies the datum assertion at this raw-coordinate boundary;
    /// subsequent projection operations retain and validate the binding.
    pub fn source_coweight(
        &self,
        coordinates: Coweight,
    ) -> Result<AmbientCoweight, StructureError> {
        if coordinates.rank() != self.source_lattice_rank() {
            return Err(StructureError::RankMismatch {
                expected: self.source_lattice_rank(),
                actual: coordinates.rank(),
            });
        }
        Ok(AmbientCoweight {
            model: Arc::clone(&self.model),
            coordinates,
        })
    }

    /// Apply the integral restriction map `Y -> P^vee`.
    pub fn map_coweight(
        &self,
        source: &AmbientCoweight,
    ) -> Result<AdjointCoweight, StructureError> {
        self.ensure_source(source)?;
        self.check_projection_work(1)?;
        let target_rank = self.target_datum().rank();
        let mut coordinates = Vec::new();
        coordinates.try_reserve_exact(target_rank).map_err(|_| {
            StructureError::AllocationFailed {
                requested: target_rank,
            }
        })?;
        for root in &self.model.source_simple_roots {
            coordinates.push(pair(root, &source.coordinates)?);
        }
        Ok(AdjointCoweight {
            model: Arc::clone(&self.model),
            coordinates: Coweight::new(coordinates),
        })
    }

    fn ensure_source(&self, source: &AmbientCoweight) -> Result<(), StructureError> {
        if Arc::ptr_eq(&self.model, &source.model) {
            Ok(())
        } else {
            Err(StructureError::DatumMismatch)
        }
    }

    fn apply_mod_two(&self, source: &ModTwoVector) -> Result<ModTwoVector, StructureError> {
        if source.dimension() != self.source_lattice_rank() {
            return Err(StructureError::RankMismatch {
                expected: self.source_lattice_rank(),
                actual: source.dimension(),
            });
        }
        self.check_projection_work(1)?;
        let target_rank = self.target_datum().rank();
        let mut ones = Vec::new();
        ones.try_reserve_exact(target_rank)
            .map_err(|_| StructureError::AllocationFailed {
                requested: target_rank,
            })?;
        for (target_coordinate, root) in self.model.source_simple_roots.iter().enumerate() {
            let mut parity = false;
            for (source_coordinate, coefficient) in root.as_slice().iter().enumerate() {
                if *coefficient % 2 != 0 && source.bit(source_coordinate) == Some(true) {
                    parity = !parity;
                }
            }
            if parity {
                ones.push(target_coordinate);
            }
        }
        ModTwoVector::from_ones(target_rank, ones)
    }

    fn check_projection_work(&self, vector_count: usize) -> Result<(), StructureError> {
        let per_vector = checked_product(self.source_lattice_rank(), self.target_datum().rank())?;
        let operations = checked_product(per_vector, vector_count)?;
        if operations > self.model.max_projection_operations {
            return Err(StructureError::AdjointFiberResourceLimit {
                resource: "projection operations",
                limit: self.model.max_projection_operations,
            });
        }
        Ok(())
    }
}

impl ModTwoAmbientMap for AdjointProjection {
    fn source_dimension(&self) -> usize {
        self.source_lattice_rank()
    }

    fn target_dimension(&self) -> usize {
        self.target_datum().rank()
    }

    fn apply(&self, source: &ModTwoVector) -> Result<ModTwoVector, StructureError> {
        self.apply_mod_two(source)
    }
}

/// The finite adjoint Cartan fiber for one validated root-data involution.
#[derive(Clone, Debug)]
pub struct AdjointCartanFiber {
    source: CartanFiber,
    projection: AdjointProjection,
    fiber: CartanFiber,
}

/// An opaque element of an [`AdjointCartanFiber`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdjointFiberElement(CartanFiberElement);

/// The proven quotient-respecting map from an ambient Cartan fiber to its
/// adjoint fiber.
///
/// It keeps no dense mod-two matrix or cached quotient-coordinate images. The
/// constructor of [`AdjointCartanFiber`] has already proved descent, so each
/// application can project a deterministic source representative on demand.
#[derive(Clone, Debug)]
pub struct FiberToAdjoint {
    source: CartanFiber,
    target: CartanFiber,
    projection: AdjointProjection,
}

impl AdjointFiberElement {
    pub fn is_identity(&self) -> bool {
        self.0.is_identity()
    }
}

impl AdjointCartanFiber {
    /// Derive the adjoint action from a validated root-data involution and
    /// build its exact finite fiber. All structural and integral work is
    /// caller-bounded.
    pub fn build(
        root_system: &RootSystem,
        root_involution: &RootInvolutionData,
        source: &CartanFiber,
        budget: &AdjointFiberBudget,
    ) -> Result<Self, StructureError> {
        if root_system.datum() != root_involution.involution().datum() {
            return Err(StructureError::DatumMismatch);
        }
        if source.involution() != root_involution.involution() {
            return Err(StructureError::CartanFiberInvolutionMismatch);
        }
        validate_adjoint_build_budget(root_system.datum(), budget)?;

        let projection =
            AdjointProjection::from_source(root_system.datum(), budget.max_projection_operations)?;
        let root_action = root_basis_action(root_system, root_involution)?;
        let coweight_action = transpose_square(&root_action)?;
        // The dual action is inverse-transpose. `RootInvolutionData` has
        // already validated that the root action is involutive, so its
        // inverse-transpose is the displayed transpose.
        let adjoint_involution = LatticeInvolution::new(
            projection.target_datum().as_based_root_datum(),
            root_action,
            coweight_action,
        )?;
        let fiber = CartanFiber::build_owned(adjoint_involution, budget.integer_lattice_budget())?;
        source.validate_induced_map(&fiber, &projection)?;

        Ok(Self {
            source: source.clone(),
            projection,
            fiber,
        })
    }

    pub fn datum(&self) -> &AdjointBasedRootDatum {
        self.projection.target_datum()
    }

    /// The exact ambient source fiber this adjoint construction was proved
    /// against. Elements minted by this value are the ones the descent proof
    /// covers; value equality of separately built fibers cannot express that.
    pub fn ambient_fiber(&self) -> &CartanFiber {
        &self.source
    }

    pub fn projection(&self) -> &AdjointProjection {
        &self.projection
    }

    /// Dimension over `F_2`; this never enumerates fiber elements.
    pub fn dimension(&self) -> usize {
        self.fiber.dimension()
    }

    pub fn identity(&self) -> Result<AdjointFiberElement, StructureError> {
        self.fiber.identity().map(AdjointFiberElement)
    }

    /// Interpret an adjoint ambient class in `P^vee / 2P^vee` as a fiber element.
    pub fn element_from_ambient(
        &self,
        representative: ModTwoVector,
    ) -> Result<AdjointFiberElement, StructureError> {
        self.fiber
            .element_from_ambient(representative)
            .map(AdjointFiberElement)
    }

    pub fn element_from_coweight_mod_two(
        &self,
        coweight: &AdjointCoweight,
    ) -> Result<AdjointFiberElement, StructureError> {
        if !Arc::ptr_eq(&self.projection.model, &coweight.model) {
            return Err(StructureError::DatumMismatch);
        }
        self.fiber
            .element_from_coweight_mod_two(&coweight.coordinates)
            .map(AdjointFiberElement)
    }

    pub fn canonical_representative(
        &self,
        element: &AdjointFiberElement,
    ) -> Result<ModTwoVector, StructureError> {
        self.fiber.canonical_representative(&element.0)
    }

    pub fn add(
        &self,
        left: &AdjointFiberElement,
        right: &AdjointFiberElement,
    ) -> Result<AdjointFiberElement, StructureError> {
        self.fiber.add(&left.0, &right.0).map(AdjointFiberElement)
    }

    pub fn same_class(
        &self,
        left: &AdjointFiberElement,
        right: &AdjointFiberElement,
    ) -> Result<bool, StructureError> {
        self.fiber.same_class(&left.0, &right.0)
    }

    pub fn basis_representatives(&self) -> &[ModTwoVector] {
        self.fiber.basis_representatives()
    }

    /// Return the only ambient-to-adjoint map proven for this construction.
    pub fn fiber_map(&self) -> FiberToAdjoint {
        FiberToAdjoint {
            source: self.source.clone(),
            target: self.fiber.clone(),
            projection: self.projection.clone(),
        }
    }
}

impl FiberToAdjoint {
    pub fn apply(
        &self,
        source: &CartanFiberElement,
    ) -> Result<AdjointFiberElement, StructureError> {
        let representative = self.source.canonical_representative(source)?;
        let projected = self.projection.apply_mod_two(&representative)?;
        self.target
            .element_from_ambient(projected)
            .map(AdjointFiberElement)
    }
}

fn validate_adjoint_build_budget(
    source: &BasedRootDatum,
    budget: &AdjointFiberBudget,
) -> Result<(), StructureError> {
    let semisimple_rank = source.semisimple_rank();
    if semisimple_rank > budget.integer_lattice.max_rank() {
        return Err(StructureError::AdjointFiberResourceLimit {
            resource: "semisimple rank",
            limit: budget.integer_lattice.max_rank(),
        });
    }

    let square = checked_product(semisimple_rank, semisimple_rank)?;
    let retained_square_entries = checked_product(square, ADJOINT_PERSISTENT_SQUARES)?;
    let copied_source_roots = checked_product(semisimple_rank, source.lattice_rank())?;
    let persistent_entries = checked_sum(retained_square_entries, copied_source_roots)?;
    if persistent_entries > budget.max_persistent_entries {
        return Err(StructureError::AdjointFiberResourceLimit {
            resource: "persistent entries",
            limit: budget.max_persistent_entries,
        });
    }

    // At most one numerator and one denominator basis vector occur per source
    // coordinate. Each direct projection examines every source coordinate for
    // each adjoint simple root.
    let source_square = checked_product(source.lattice_rank(), source.lattice_rank())?;
    let descent_vectors = checked_product(source_square, semisimple_rank)?;
    let descent_operations = checked_product(descent_vectors, 2)?;
    if descent_operations > budget.max_projection_operations {
        return Err(StructureError::AdjointFiberResourceLimit {
            resource: "projection operations",
            limit: budget.max_projection_operations,
        });
    }
    Ok(())
}

fn root_basis_action(
    root_system: &RootSystem,
    root_involution: &RootInvolutionData,
) -> Result<Vec<Vec<i32>>, StructureError> {
    let rank = root_system.datum().semisimple_rank();
    let mut action = zero_square_matrix(rank)?;
    for (column, simple_root) in root_system.datum().simple_roots().iter().enumerate() {
        let root = root_system
            .id_of(simple_root)
            .ok_or(StructureError::InvalidRootAutomorphism)?;
        let image = root_involution
            .image(root)
            .ok_or(StructureError::InvalidRootAutomorphism)?;
        let coordinates = root_system
            .simple_coordinates(image)
            .ok_or(StructureError::InvalidRootAutomorphism)?;
        if coordinates.len() != rank {
            return Err(StructureError::RankMismatch {
                expected: rank,
                actual: coordinates.len(),
            });
        }
        for (row, coordinate) in coordinates.iter().enumerate() {
            action[row][column] = *coordinate;
        }
    }
    Ok(action)
}

fn clone_i32_matrix(matrix: &[Vec<i32>]) -> Result<Vec<Vec<i32>>, StructureError> {
    let mut copy = Vec::new();
    copy.try_reserve_exact(matrix.len())
        .map_err(|_| StructureError::AllocationFailed {
            requested: matrix.len(),
        })?;
    for row in matrix {
        let mut row_copy = Vec::new();
        row_copy
            .try_reserve_exact(row.len())
            .map_err(|_| StructureError::AllocationFailed {
                requested: row.len(),
            })?;
        row_copy.extend_from_slice(row);
        copy.push(row_copy);
    }
    Ok(copy)
}

fn clone_weights(weights: &[Weight]) -> Result<Vec<Weight>, StructureError> {
    let mut copy = Vec::new();
    copy.try_reserve_exact(weights.len())
        .map_err(|_| StructureError::AllocationFailed {
            requested: weights.len(),
        })?;
    for weight in weights {
        let mut coordinates = Vec::new();
        coordinates.try_reserve_exact(weight.rank()).map_err(|_| {
            StructureError::AllocationFailed {
                requested: weight.rank(),
            }
        })?;
        coordinates.extend_from_slice(weight.as_slice());
        copy.push(Weight::new(coordinates));
    }
    Ok(copy)
}

fn zero_square_matrix(rank: usize) -> Result<Vec<Vec<i32>>, StructureError> {
    let mut matrix = Vec::new();
    matrix
        .try_reserve_exact(rank)
        .map_err(|_| StructureError::AllocationFailed { requested: rank })?;
    for _ in 0..rank {
        let mut row = Vec::new();
        row.try_reserve_exact(rank)
            .map_err(|_| StructureError::AllocationFailed { requested: rank })?;
        row.resize(rank, 0);
        matrix.push(row);
    }
    Ok(matrix)
}

fn transpose_square(matrix: &[Vec<i32>]) -> Result<Vec<Vec<i32>>, StructureError> {
    let rank = matrix.len();
    if matrix.iter().any(|row| row.len() != rank) {
        return Err(StructureError::InvalidInvolution);
    }
    let mut transpose = zero_square_matrix(rank)?;
    for (row, entries) in matrix.iter().enumerate() {
        for (column, entry) in entries.iter().enumerate() {
            transpose[column][row] = *entry;
        }
    }
    Ok(transpose)
}

fn checked_product(left: usize, right: usize) -> Result<usize, StructureError> {
    left.checked_mul(right)
        .ok_or(StructureError::ArithmeticOverflow)
}

fn checked_sum(left: usize, right: usize) -> Result<usize, StructureError> {
    left.checked_add(right)
        .ok_or(StructureError::ArithmeticOverflow)
}

#[cfg(test)]
mod tests {
    use crate::{BasedRootDatum, RootInvolutionData, TwistedInvolution, WeylGroup};

    use super::*;

    fn integer_budget() -> IntegerLatticeBudget {
        IntegerLatticeBudget::new(64, 100_000, 100_000, 128)
    }

    fn adjoint_budget() -> AdjointFiberBudget {
        AdjointFiberBudget::new(integer_budget(), 50_000, 100_000)
    }

    fn ambient(rank: usize, indices: impl IntoIterator<Item = usize>) -> ModTwoVector {
        ModTwoVector::from_ones(rank, indices).unwrap()
    }

    #[test]
    fn identity_a1_projects_the_fiber_generator() {
        let datum = BasedRootDatum::standard(vec![vec![2]]).unwrap();
        let roots = RootSystem::enumerate(&datum, 2).unwrap();
        let involution = LatticeInvolution::identity(&datum).unwrap();
        let root_involution = RootInvolutionData::new(&roots, involution.clone()).unwrap();
        let lattice_budget = integer_budget();
        let source = CartanFiber::build(&involution, &lattice_budget).unwrap();
        let target =
            AdjointCartanFiber::build(&roots, &root_involution, &source, &adjoint_budget())
                .unwrap();

        assert_eq!(target.datum().rank(), 1);
        assert_eq!(target.dimension(), 1);
        let source_coweight = target
            .projection()
            .source_coweight(Coweight::new(vec![1]))
            .unwrap();
        let projected = target.projection().map_coweight(&source_coweight).unwrap();
        let expected = target.element_from_coweight_mod_two(&projected).unwrap();
        let map = target.fiber_map();
        let source_generator = source.element_from_ambient(ambient(1, [0])).unwrap();

        assert_eq!(map.apply(&source_generator).unwrap(), expected);
        assert_eq!(
            target.canonical_representative(&expected).unwrap(),
            ambient(1, [0])
        );
    }

    #[test]
    fn central_coweights_map_to_the_adjoint_fiber_kernel() {
        let datum = BasedRootDatum::from_simple_data(
            2,
            vec![vec![2]],
            vec![Weight::new(vec![1, 0])],
            vec![Coweight::new(vec![2, 0])],
        )
        .unwrap();
        let roots = RootSystem::enumerate(&datum, 2).unwrap();
        let involution = LatticeInvolution::identity(&datum).unwrap();
        let root_involution = RootInvolutionData::new(&roots, involution.clone()).unwrap();
        let lattice_budget = integer_budget();
        let source = CartanFiber::build(&involution, &lattice_budget).unwrap();
        let target =
            AdjointCartanFiber::build(&roots, &root_involution, &source, &adjoint_budget())
                .unwrap();
        let map = target.fiber_map();
        let root_direction = source.element_from_ambient(ambient(2, [0])).unwrap();
        let central_direction = source.element_from_ambient(ambient(2, [1])).unwrap();

        assert_eq!(source.dimension(), 2);
        assert_eq!(target.dimension(), 1);
        assert_eq!(
            target
                .canonical_representative(&map.apply(&root_direction).unwrap())
                .unwrap(),
            ambient(1, [0])
        );
        assert!(map.apply(&central_direction).unwrap().is_identity());
        let sum = source.add(&root_direction, &central_direction).unwrap();
        assert_eq!(
            map.apply(&sum).unwrap(),
            target
                .add(
                    &map.apply(&root_direction).unwrap(),
                    &map.apply(&central_direction).unwrap(),
                )
                .unwrap()
        );
    }

    #[test]
    fn derives_the_adjoint_coweight_action_instead_of_reusing_root_rows() {
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let roots = RootSystem::enumerate(&datum, 6).unwrap();
        let twisted = TwistedInvolution::new(
            &datum,
            &roots,
            &LatticeInvolution::identity(&datum).unwrap(),
            WeylGroup::new(datum.clone()).simple_reflection(0).unwrap(),
        )
        .unwrap();
        let lattice_budget = integer_budget();
        let source =
            CartanFiber::build(twisted.root_involution().involution(), &lattice_budget).unwrap();
        let target = AdjointCartanFiber::build(
            &roots,
            twisted.root_involution(),
            &source,
            &adjoint_budget(),
        )
        .unwrap();

        assert_eq!(
            target.fiber.involution().weight_matrix(),
            &[vec![-1, 1], vec![0, 1]]
        );
        assert_eq!(
            target.fiber.involution().coweight_matrix(),
            &[vec![-1, 0], vec![1, 1]]
        );
    }

    #[test]
    fn requires_the_source_fiber_for_the_actual_involution_at_construction() {
        let datum = BasedRootDatum::standard(vec![vec![2]]).unwrap();
        let roots = RootSystem::enumerate(&datum, 2).unwrap();
        let identity = LatticeInvolution::identity(&datum).unwrap();
        let root_involution = RootInvolutionData::new(&roots, identity).unwrap();
        let different = LatticeInvolution::new(&datum, vec![vec![-1]], vec![vec![-1]]).unwrap();
        let lattice_budget = integer_budget();
        let source = CartanFiber::build(&different, &lattice_budget).unwrap();

        assert!(matches!(
            AdjointCartanFiber::build(&roots, &root_involution, &source, &adjoint_budget()),
            Err(StructureError::CartanFiberInvolutionMismatch)
        ));
    }

    #[test]
    fn rejects_source_coordinates_bound_to_another_projection() {
        let datum = BasedRootDatum::standard(vec![vec![2]]).unwrap();
        let roots = RootSystem::enumerate(&datum, 2).unwrap();
        let involution = LatticeInvolution::identity(&datum).unwrap();
        let root_involution = RootInvolutionData::new(&roots, involution.clone()).unwrap();
        let lattice_budget = integer_budget();
        let first_source = CartanFiber::build(&involution, &lattice_budget).unwrap();
        let second_source = CartanFiber::build(&involution, &lattice_budget).unwrap();
        let first =
            AdjointCartanFiber::build(&roots, &root_involution, &first_source, &adjoint_budget())
                .unwrap();
        let second =
            AdjointCartanFiber::build(&roots, &root_involution, &second_source, &adjoint_budget())
                .unwrap();
        let source_coweight = first
            .projection()
            .source_coweight(Coweight::new(vec![1]))
            .unwrap();

        assert_eq!(
            second.projection().map_coweight(&source_coweight),
            Err(StructureError::DatumMismatch)
        );
    }

    #[test]
    fn projects_a_nonsymmetric_actual_action_and_intertwines_the_dual_actions() {
        let datum =
            BasedRootDatum::standard(vec![vec![2, 0, 0], vec![0, 2, -1], vec![0, -1, 2]]).unwrap();
        let roots = RootSystem::enumerate(&datum, 8).unwrap();
        let twisted = TwistedInvolution::new(
            &datum,
            &roots,
            &LatticeInvolution::identity(&datum).unwrap(),
            WeylGroup::new(datum.clone()).simple_reflection(1).unwrap(),
        )
        .unwrap();
        let lattice_budget = integer_budget();
        let source =
            CartanFiber::build(twisted.root_involution().involution(), &lattice_budget).unwrap();
        let target = AdjointCartanFiber::build(
            &roots,
            twisted.root_involution(),
            &source,
            &adjoint_budget(),
        )
        .unwrap();

        assert_eq!(source.dimension(), 1);
        assert_eq!(target.dimension(), 1);
        assert_eq!(
            target.fiber.involution().weight_matrix(),
            &[vec![1, 0, 0], vec![0, -1, 1], vec![0, 0, 1]]
        );
        assert_eq!(
            target.fiber.involution().coweight_matrix(),
            &[vec![1, 0, 0], vec![0, -1, 0], vec![0, 1, 1]]
        );

        for coordinate in 0..datum.lattice_rank() {
            let mut basis_coordinates = vec![0; datum.lattice_rank()];
            basis_coordinates[coordinate] = 1;
            let source_basis = Coweight::new(basis_coordinates);
            let theta_source = twisted
                .root_involution()
                .involution()
                .act_on_coweight(&source_basis)
                .unwrap();
            let bound_source = target.projection().source_coweight(source_basis).unwrap();
            let bound_theta_source = target.projection().source_coweight(theta_source).unwrap();
            let lhs = target
                .projection()
                .map_coweight(&bound_theta_source)
                .unwrap();
            let projected = target.projection().map_coweight(&bound_source).unwrap();
            let rhs = target
                .fiber
                .involution()
                .act_on_coweight(projected.as_coweight())
                .unwrap();

            assert_eq!(lhs.as_coweight(), &rhs);
        }

        let source_generator = source.element_from_ambient(ambient(3, [0])).unwrap();
        let image = target.fiber_map().apply(&source_generator).unwrap();
        assert_eq!(
            target.canonical_representative(&image).unwrap(),
            ambient(3, [0])
        );
    }

    #[test]
    fn supports_dynamic_adjoint_rank_above_the_legacy_packed_limit() {
        let rank = 33;
        let mut cartan = vec![vec![0; rank]; rank];
        for (index, row) in cartan.iter_mut().enumerate() {
            row[index] = 2;
        }
        let datum = BasedRootDatum::standard(cartan).unwrap();
        let roots = RootSystem::enumerate(&datum, 66).unwrap();
        let involution = LatticeInvolution::identity(&datum).unwrap();
        let root_involution = RootInvolutionData::new(&roots, involution.clone()).unwrap();
        let lattice_budget = integer_budget();
        let source = CartanFiber::build(&involution, &lattice_budget).unwrap();
        let target =
            AdjointCartanFiber::build(&roots, &root_involution, &source, &adjoint_budget())
                .unwrap();
        let source_generator = source.element_from_ambient(ambient(rank, [32])).unwrap();

        assert_eq!(target.datum().rank(), rank);
        assert_eq!(target.dimension(), rank);
        assert_eq!(
            target
                .canonical_representative(&target.fiber_map().apply(&source_generator).unwrap())
                .unwrap(),
            ambient(rank, [32])
        );
    }

    #[test]
    fn rejects_adjoint_storage_before_allocating_target_matrices() {
        let datum = BasedRootDatum::standard(vec![vec![2]]).unwrap();
        let roots = RootSystem::enumerate(&datum, 2).unwrap();
        let involution = LatticeInvolution::identity(&datum).unwrap();
        let root_involution = RootInvolutionData::new(&roots, involution.clone()).unwrap();
        let lattice_budget = integer_budget();
        let source = CartanFiber::build(&involution, &lattice_budget).unwrap();
        let constrained = AdjointFiberBudget::new(integer_budget(), 1, 100);

        assert!(matches!(
            AdjointCartanFiber::build(&roots, &root_involution, &source, &constrained),
            Err(StructureError::AdjointFiberResourceLimit {
                resource: "persistent entries",
                limit: 1
            })
        ));
    }

    #[test]
    fn rejects_insufficient_projection_work_before_building_the_target() {
        let datum = BasedRootDatum::standard(vec![vec![2]]).unwrap();
        let roots = RootSystem::enumerate(&datum, 2).unwrap();
        let involution = LatticeInvolution::identity(&datum).unwrap();
        let root_involution = RootInvolutionData::new(&roots, involution.clone()).unwrap();
        let lattice_budget = integer_budget();
        let source = CartanFiber::build(&involution, &lattice_budget).unwrap();
        let constrained = AdjointFiberBudget::new(integer_budget(), 100, 0);

        assert!(matches!(
            AdjointCartanFiber::build(&roots, &root_involution, &source, &constrained),
            Err(StructureError::AdjointFiberResourceLimit {
                resource: "projection operations",
                limit: 0
            })
        ));
    }
}
