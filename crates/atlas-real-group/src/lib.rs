//! Structural data for real reductive groups.
//!
//! The crate deliberately contains mathematical values, not Atlas syntax or
//! output policy.  It is an adapter boundary for the interpreter's future
//! domain values: all constructors validate the root-datum invariants that
//! would otherwise be implicit in the C++ implementation.
#![allow(dead_code, clippy::needless_range_loop, clippy::type_complexity)]

use std::collections::{HashSet, VecDeque};

mod adjoint_fiber;
mod block;
mod cartan_class;
mod cartan_classification;
mod cartan_fiber;
mod cayley_cross;
pub mod deform;
mod dual;
mod dynkin;
mod error;
pub mod ext_block;
pub mod ext_kl;
pub mod ext_param;
mod form_name;
mod global_kgb;
// Consumer is the synthetic-real-form builder; the carrier lands first so
// its rational transport can be reviewed independently.
#[allow(dead_code)]
mod global_tits;
mod grading;
mod inner_class;
mod integer_lattice;
mod involution;
mod involution_classification;
mod involution_table;
mod kgb_graph;
mod kl_polynomial;
mod kl_support;
mod kl_table;
mod ktype;
mod lattice;
mod layout;
mod matreduc;
mod minimal_torus;
mod mod_two;
mod partial_block;
mod presentation;
mod primitive_involution;
mod real_form_labels;
mod real_form_order;
mod real_form_seed;
mod real_projection;
pub mod real_weyl;
mod rep_context;
mod rep_table;
mod restricted_roots;
mod root_datum;
mod root_involution;
mod root_system;
mod strong_real;
mod tits_element;
mod topology;
pub use topology::dual_component_group_rank;
mod twisted_involution;
mod weak_real_form;
mod weyl;
mod weyl_element;
mod weyl_transducer;
// Consumer is the parked task #9 (docs/ON_DEMAND_PARTITION_DESIGN.md);
// the allow lifts when the on-demand partition lands.
#[allow(dead_code)]
mod weyl_size;

pub use adjoint_fiber::{
    AdjointBasedRootDatum, AdjointCartanFiber, AdjointCoweight, AdjointFiberBudget,
    AdjointFiberElement, AdjointProjection, AmbientCoweight, FiberToAdjoint,
};
pub use block::{dual_involution, BlockDescent, BlockGraph};
pub use cartan_class::{CartanClass, TwistedConjugacyClass, TwistedConjugacyPartition};
pub use cartan_classification::{CartanClassification, CartanClassificationBudget, CartanId};
pub use cartan_fiber::{CartanFiber, CartanFiberElement};
pub use cayley_cross::CayleyCrossDecomposition;
pub use deform::{
    block_deformation_to_height, integral_block_scope, singular_orbits_at, twisted_deformation,
    twisted_deformation_terms, twisted_kl_column_at_s, twisted_kl_sum, IntegralBlockScope,
    SplitInteger,
};
pub use dual::{
    dual_cartan_correspondence, dual_datum, dual_inner_class, dual_real_form_count, longest_action,
};
pub use dynkin::bourbaki_permutation;
pub use error::StructureError;
pub use form_name::form_type_name;
pub use global_kgb::{GlobalKgb, GlobalKgbPrint, GlobalKgbPrintRow};
pub use grading::{CartanGradingData, Grading};
pub use inner_class::{inner_class_with_twisted_involution, InnerClass};
pub use integer_lattice::{adapted_basis, AdaptedBasis, IntegerMatrix};
pub use integer_lattice::{
    adapted_relation_basis, annihilator_modulo, filter_relation_units, quotient_relation_basis,
    replace_relation_generators, IntegerLatticeBudget, RelationBasis, RelationError,
    RelationGenerator, RelationMatrix,
};
pub use involution::LatticeInvolution;
pub use involution_classification::{classify_involution, fiber_rank, InvolutionClassification};
pub use involution_table::{
    InvolutionId, InvolutionRecord, InvolutionTable, InvolutionTableBudget,
};
pub use kgb_graph::{KgbGraph, KgbId, KgbStatus};
pub use kl_polynomial::KlPol;
pub use kl_support::RankFlags;
pub use kl_table::{KlTable, MuPair};
pub use ktype::KType;
pub use lattice::{pair, Coweight, RationalCoweight, RationalWeight, Weight};
pub use layout::InnerClassLayout;
pub use minimal_torus::{elected_square_root, minimal_torus_part};
pub use mod_two::{ModTwoSubspace, ModTwoVector};
pub use partial_block::{
    bruhat_below, CommonContext, IntegralSubsystem, PartialBlock, StandardReprMod,
};
pub use presentation::{build_presentations, RealFormPresentation};
pub use primitive_involution::{
    checked_inner_class_letters, layout_involution, on_basis, InnerClassLetterError,
};
pub use real_form_labels::RealFormLabels;
pub use real_form_order::ExternalFormOrder;
pub use real_form_seed::RealFormSeed;
pub use rep_context::{RepContext, StandardRepr};
pub use restricted_roots::{RestrictedRoot, RestrictedRootSystem, RestrictedWeight};
pub use root_datum::BasedRootDatum;
pub use root_involution::{RootInvolutionData, RootKind};
pub use root_system::{RootId, RootSet, RootSystem, RootSystemBudget};
pub use strong_real::{
    central_fiber, strong_real_class_prints, SquareClassId, StrongRealClassPrint,
    StrongRealClassification, StrongRealData, StrongRealFormRep,
};
pub use tits_element::{TitsCoset, TitsElement};
pub use twisted_involution::TwistedInvolution;
pub use weak_real_form::{weak_real_form_at_representative, WeakRealFormId, WeakRealFormPartition};
pub use weyl::{WeylAction, WeylGroup};
pub use weyl_element::{ParabolicPieces, WeylElement, WeylInterface};
pub use weyl_transducer::CompactWeyl;

/// Legacy untyped coordinate vector used only by the A1 migration prototype.
///
/// New root-data APIs use [`Weight`] and [`Coweight`], which prevent mixing
/// the two dual lattices at compile time. This type remains temporarily while
/// its prototype consumers are migrated.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub(crate) struct LatticeVector(Vec<i32>);

impl LatticeVector {
    pub fn new(values: Vec<i32>) -> Self {
        Self(values)
    }
    pub fn rank(&self) -> usize {
        self.0.len()
    }
    pub fn as_slice(&self) -> &[i32] {
        &self.0
    }
}

/// A root datum in the simple-root basis.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RootDatum {
    cartan: Vec<Vec<i32>>,
    simple_roots: Vec<LatticeVector>,
    simple_coroots: Vec<LatticeVector>,
}

#[allow(dead_code)] // Crate-private A1 prototype pending replacement by BasedRootDatum consumers.
impl RootDatum {
    pub fn new(cartan: Vec<Vec<i32>>) -> Result<Self, StructureError> {
        let rank = cartan.len();
        if rank == 0 {
            return Err(StructureError::EmptyRootDatum);
        }
        BasedRootDatum::standard(cartan.clone())?;
        if cartan.iter().any(|row| row.len() != rank) {
            return Err(StructureError::NonSquareCartan);
        }
        if cartan
            .iter()
            .enumerate()
            .any(|(i, row)| row[i] != 2 || row.iter().enumerate().any(|(j, &a)| i != j && a > 0))
        {
            return Err(StructureError::InvalidCartanMatrix);
        }
        let simple_roots = (0..rank)
            .map(|i| {
                let mut v = vec![0; rank];
                v[i] = 1;
                LatticeVector::new(v)
            })
            .collect();
        // In simple-root coordinates, the j-th simple coroot is represented
        // by the j-th Cartan-matrix column, so its pairing with alpha_i is
        // the Cartan entry a_ij.
        let simple_coroots = (0..rank)
            .map(|j| LatticeVector::new(cartan.iter().map(|row| row[j]).collect()))
            .collect();
        Ok(Self {
            cartan,
            simple_roots,
            simple_coroots,
        })
    }

    pub fn from_basis(
        cartan: Vec<Vec<i32>>,
        simple_roots: Vec<LatticeVector>,
        simple_coroots: Vec<LatticeVector>,
    ) -> Result<Self, StructureError> {
        let datum = Self::new(cartan)?;
        let rank = datum.rank();
        if simple_roots.len() != rank {
            return Err(StructureError::RankMismatch {
                expected: rank,
                actual: simple_roots.len(),
            });
        }
        if simple_coroots.len() != rank {
            return Err(StructureError::RankMismatch {
                expected: rank,
                actual: simple_coroots.len(),
            });
        }
        if simple_roots.iter().any(|v| v.rank() != rank) {
            return Err(StructureError::RankMismatch {
                expected: rank,
                actual: simple_roots
                    .iter()
                    .find(|v| v.rank() != rank)
                    .unwrap()
                    .rank(),
            });
        }
        if simple_coroots.iter().any(|v| v.rank() != rank) {
            return Err(StructureError::RankMismatch {
                expected: rank,
                actual: simple_coroots
                    .iter()
                    .find(|v| v.rank() != rank)
                    .unwrap()
                    .rank(),
            });
        }
        for (i, root) in simple_roots.iter().enumerate() {
            for (j, coroot) in simple_coroots.iter().enumerate() {
                let actual = lattice::pair_coordinates(root.as_slice(), coroot.as_slice())?;
                if actual != datum.cartan[i][j] {
                    return Err(StructureError::RootPairingMismatch {
                        row: i,
                        column: j,
                        expected: datum.cartan[i][j],
                        actual,
                    });
                }
            }
        }
        Ok(Self {
            cartan: datum.cartan,
            simple_roots,
            simple_coroots,
        })
    }

    pub fn rank(&self) -> usize {
        self.cartan.len()
    }
    pub fn cartan_matrix(&self) -> &[Vec<i32>] {
        &self.cartan
    }
    pub fn simple_roots(&self) -> &[LatticeVector] {
        &self.simple_roots
    }
    pub fn simple_coroots(&self) -> &[LatticeVector] {
        &self.simple_coroots
    }
    pub fn pairing(
        &self,
        root: &LatticeVector,
        coroot: &LatticeVector,
    ) -> Result<i32, StructureError> {
        if root.rank() != self.rank() {
            return Err(StructureError::RankMismatch {
                expected: self.rank(),
                actual: root.rank(),
            });
        }
        if coroot.rank() != self.rank() {
            return Err(StructureError::RankMismatch {
                expected: self.rank(),
                actual: coroot.rank(),
            });
        }
        lattice::pair_coordinates(root.as_slice(), coroot.as_slice())
    }

    /// Enumerate the finite root system generated by the simple roots.
    ///
    /// The closure is intentionally bounded: an indefinite Cartan matrix does
    /// not define a finite root system, and must not make an interpreter call
    /// diverge indefinitely.
    pub fn roots(&self) -> Result<Vec<LatticeVector>, StructureError> {
        const LIMIT: usize = 4096;
        let mut seen = HashSet::new();
        let mut queue = VecDeque::new();
        for root in &self.simple_roots {
            let values = root.as_slice().to_vec();
            if seen.insert(values.clone()) {
                queue.push_back(LatticeVector::new(values.clone()));
            }
            let negative: Vec<i32> = values
                .into_iter()
                .map(|x| x.checked_neg().ok_or(StructureError::ArithmeticOverflow))
                .collect::<Result<_, _>>()?;
            if seen.insert(negative.clone()) {
                queue.push_back(LatticeVector::new(negative));
            }
        }
        while let Some(root) = queue.pop_front() {
            for i in 0..self.rank() {
                let coefficient = i128::from(lattice::pair_coordinates(
                    root.as_slice(),
                    self.simple_coroots[i].as_slice(),
                )?);
                let reflected: Vec<i32> = root
                    .as_slice()
                    .iter()
                    .zip(self.simple_roots[i].as_slice())
                    .map(|(a, simple)| {
                        i32::try_from(i128::from(*a) - coefficient * i128::from(*simple))
                            .map_err(|_| StructureError::ArithmeticOverflow)
                    })
                    .collect::<Result<_, _>>()?;
                if seen.insert(reflected.clone()) {
                    if seen.len() > LIMIT {
                        return Err(StructureError::RootSystemTooLarge);
                    }
                    queue.push_back(LatticeVector::new(reflected));
                }
            }
        }
        let mut roots: Vec<_> = seen.into_iter().map(LatticeVector::new).collect();
        roots.sort_by(|a, b| a.as_slice().cmp(b.as_slice()));
        Ok(roots)
    }
}

/// A Weyl group element represented by a word in the simple reflections.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub(crate) struct PrototypeWeylElement {
    word: Vec<usize>,
}

/// The finite Weyl group generated by the simple reflections of a root datum.
#[derive(Clone, Debug)]
pub(crate) struct PrototypeWeylGroup {
    datum: RootDatum,
    elements: Vec<PrototypeWeylElement>,
}

#[allow(dead_code)] // Crate-private A1 prototype pending canonical Weyl actions.
impl PrototypeWeylGroup {
    pub fn new(datum: RootDatum) -> Result<Self, StructureError> {
        datum.roots()?;
        let mut elements = vec![PrototypeWeylElement::identity()];
        let mut seen = HashSet::new();
        seen.insert(Self::element_key(&datum, &elements[0])?);
        let mut cursor = 0;
        while cursor < elements.len() {
            let current = elements[cursor].clone();
            for generator in 0..datum.rank() {
                let candidate = current.multiply(&PrototypeWeylElement {
                    word: vec![generator],
                });
                if seen.insert(Self::element_key(&datum, &candidate)?) {
                    if elements.len() >= 65_536 {
                        return Err(StructureError::WeylGroupTooLarge);
                    }
                    elements.push(candidate);
                }
            }
            cursor += 1;
        }
        Ok(Self { datum, elements })
    }

    fn element_key(
        datum: &RootDatum,
        element: &PrototypeWeylElement,
    ) -> Result<Vec<Vec<i32>>, StructureError> {
        datum
            .simple_roots()
            .iter()
            .map(|root| element.act_on_root(datum, root).map(|image| image.0))
            .collect()
    }

    pub fn rank(&self) -> usize {
        self.datum.rank()
    }
    pub fn order(&self) -> usize {
        self.elements.len()
    }
    pub fn elements(&self) -> &[PrototypeWeylElement] {
        &self.elements
    }
    pub fn datum(&self) -> &RootDatum {
        &self.datum
    }
}

#[allow(dead_code)] // Crate-private A1 prototype pending canonical Weyl actions.
impl PrototypeWeylElement {
    pub fn identity() -> Self {
        Self { word: Vec::new() }
    }
    pub fn from_word(word: Vec<usize>, rank: usize) -> Result<Self, StructureError> {
        if let Some(&i) = word.iter().find(|&&i| i >= rank) {
            return Err(StructureError::RankMismatch {
                expected: rank,
                actual: i.saturating_add(1),
            });
        }
        Ok(Self { word })
    }
    pub fn word(&self) -> &[usize] {
        &self.word
    }
    pub fn multiply(&self, other: &Self) -> Self {
        let mut word = self.word.clone();
        word.extend_from_slice(&other.word);
        Self { word }
    }
    pub fn act_on_root(
        &self,
        datum: &RootDatum,
        root: &LatticeVector,
    ) -> Result<LatticeVector, StructureError> {
        if root.rank() != datum.rank() {
            return Err(StructureError::RankMismatch {
                expected: datum.rank(),
                actual: root.rank(),
            });
        }
        let mut result = root.as_slice().to_vec();
        for &i in self.word.iter().rev() {
            if i >= datum.rank() {
                return Err(StructureError::RankMismatch {
                    expected: datum.rank(),
                    actual: i.saturating_add(1),
                });
            }
            let coefficient = i128::from(lattice::pair_coordinates(
                &result,
                datum.simple_coroots[i].as_slice(),
            )?);
            for (entry, simple) in result.iter_mut().zip(datum.simple_roots[i].as_slice()) {
                *entry = i32::try_from(i128::from(*entry) - coefficient * i128::from(*simple))
                    .map_err(|_| StructureError::ArithmeticOverflow)?;
            }
        }
        Ok(LatticeVector::new(result))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RootType {
    CompactImaginary,
    NoncompactImaginary,
    Real,
    Complex,
}

/// A Cartan involution on the root lattice, with compactness data for fixed
/// (imaginary) simple roots kept separately from the root action.  The matrix
/// form is needed for diagram automorphisms that are not permutations.
///
/// The `compact_imaginary` flags here are an unvalidated caller assertion,
/// superseded by [`Grading`], which derives compactness from a validated
/// adjoint-fiber element; they are not the crate's compactness semantics.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CartanInvolution {
    matrix: Vec<Vec<i32>>,
    compact_imaginary: Vec<bool>,
}

#[allow(dead_code)] // Crate-private A1 prototype pending dual-lattice involutions.
impl CartanInvolution {
    pub fn new(
        matrix: Vec<Vec<i32>>,
        compact_imaginary: Vec<bool>,
        datum: &RootDatum,
    ) -> Result<Self, StructureError> {
        let rank = datum.rank();
        if matrix.len() != rank
            || matrix.iter().any(|row| row.len() != rank)
            || compact_imaginary.len() != rank
        {
            return Err(StructureError::InvalidInvolution);
        }
        for i in 0..rank {
            for j in 0..rank {
                let column = matrix.iter().map(|row| row[j]).collect::<Vec<_>>();
                let square = lattice::pair_coordinates(&matrix[i], &column)?;
                if square != if i == j { 1 } else { 0 } {
                    return Err(StructureError::InvalidInvolution);
                }
            }
        }
        let candidate = Self {
            matrix,
            compact_imaginary,
        };
        let roots: HashSet<Vec<i32>> = datum.roots()?.into_iter().map(|r| r.0).collect();
        for (i, root) in datum.simple_roots().iter().enumerate() {
            let image = candidate.act(root)?;
            if !roots.contains(image.as_slice()) {
                return Err(StructureError::InvalidRootAutomorphism);
            }
            if candidate.compact_imaginary[i] && image != *root {
                return Err(StructureError::InvalidInvolution);
            }
        }
        Ok(candidate)
    }
    pub fn identity(datum: &RootDatum) -> Self {
        Self {
            matrix: (0..datum.rank())
                .map(|i| {
                    (0..datum.rank())
                        .map(|j| if i == j { 1 } else { 0 })
                        .collect()
                })
                .collect(),
            compact_imaginary: vec![true; datum.rank()],
        }
    }
    pub fn rank(&self) -> usize {
        self.matrix.len()
    }
    pub fn matrix(&self) -> &[Vec<i32>] {
        &self.matrix
    }
    pub fn compact_imaginary(&self) -> &[bool] {
        &self.compact_imaginary
    }
    pub fn act(&self, root: &LatticeVector) -> Result<LatticeVector, StructureError> {
        if root.rank() != self.rank() {
            return Err(StructureError::RankMismatch {
                expected: self.rank(),
                actual: root.rank(),
            });
        }
        let mut result = vec![0; self.rank()];
        for (i, row) in self.matrix.iter().enumerate() {
            result[i] = lattice::pair_coordinates(row, root.as_slice())?;
        }
        Ok(LatticeVector::new(result))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RealReductiveGroup {
    root_datum: RootDatum,
    involution: CartanInvolution,
}

#[allow(dead_code)] // Crate-private A1 prototype pending real-form data.
impl RealReductiveGroup {
    pub fn new(
        root_datum: RootDatum,
        involution: CartanInvolution,
    ) -> Result<Self, StructureError> {
        if root_datum.rank() != involution.rank() {
            return Err(StructureError::RankMismatch {
                expected: root_datum.rank(),
                actual: involution.rank(),
            });
        }
        Ok(Self {
            root_datum,
            involution,
        })
    }
    pub fn root_datum(&self) -> &RootDatum {
        &self.root_datum
    }
    pub fn involution(&self) -> &CartanInvolution {
        &self.involution
    }
    /// Number of simple roots sent to their negatives. This is deliberately
    /// narrower than the real rank, which requires the full restricted root
    /// system and is not inferred from simple-root data.
    pub fn simple_real_rank(&self) -> usize {
        (0..self.root_datum.rank())
            .filter(|&i| {
                self.involution
                    .act(&self.root_datum.simple_roots()[i])
                    .map(|r| {
                        r.as_slice()
                            .iter()
                            .zip(self.root_datum.simple_roots()[i].as_slice())
                            .all(|(a, b)| *a == -*b)
                    })
                    .unwrap_or(false)
            })
            .count()
    }

    pub fn classify_simple_root(&self, index: usize) -> Result<RootType, StructureError> {
        if index >= self.root_datum.rank() {
            return Err(StructureError::RankMismatch {
                expected: self.root_datum.rank(),
                actual: index.saturating_add(1),
            });
        }
        let root = &self.root_datum.simple_roots()[index];
        let image = self.involution.act(root)?;
        if image == *root {
            return Ok(if self.involution.compact_imaginary[index] {
                RootType::CompactImaginary
            } else {
                RootType::NoncompactImaginary
            });
        }
        if image
            .as_slice()
            .iter()
            .zip(root.as_slice())
            .all(|(a, b)| *a == -*b)
        {
            Ok(RootType::Real)
        } else {
            Ok(RootType::Complex)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a1_simple_reflection_negates_the_root() {
        let datum = RootDatum::new(vec![vec![2]]).unwrap();
        let reflection = PrototypeWeylElement::from_word(vec![0], 1).unwrap();
        assert_eq!(
            reflection
                .act_on_root(&datum, &LatticeVector::new(vec![1]))
                .unwrap(),
            LatticeVector::new(vec![-1])
        );
    }

    #[test]
    fn fixed_root_can_be_noncompact_imaginary() {
        let datum = RootDatum::new(vec![vec![2]]).unwrap();
        let involution = CartanInvolution::new(vec![vec![1]], vec![false], &datum).unwrap();
        let group = RealReductiveGroup::new(datum, involution).unwrap();
        assert_eq!(
            group.classify_simple_root(0).unwrap(),
            RootType::NoncompactImaginary
        );
    }

    #[test]
    fn legacy_pairing_reports_overflow_instead_of_wrapping() {
        let error = RootDatum::from_basis(
            vec![vec![2, -1, 0], vec![-1, 2, -1], vec![0, -1, 2]],
            vec![LatticeVector::new(vec![i32::MAX; 3]); 3],
            vec![LatticeVector::new(vec![i32::MAX; 3]); 3],
        )
        .expect_err("large legacy coordinates overflow their fixed-width pairing");
        assert_eq!(error, StructureError::ArithmeticOverflow);
    }
}
