use std::collections::BTreeMap;

use crate::twisted_involution::compose_matrices;
use crate::{
    BasedRootDatum, CartanGradingData, CayleyCrossDecomposition, RealFormLabels,
    RootInvolutionData, StructureError, TwistedInvolution, WeakRealFormPartition,
};

/// One deterministic orbit under Weyl twisted conjugacy.
///
/// The representative is the first action in this crate's deterministic Weyl
/// enumeration, not Atlas's `canonicalize` representative; it decomposes
/// through [`crate::CayleyCrossDecomposition`], and real-form labels
/// correlate through [`crate::RealFormLabels`] at that same representative.
/// Fiber groups, real-form attribution, and real Cartan component data live
/// in [`crate::CartanClass`], which owns a value of this type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TwistedConjugacyClass {
    representative: TwistedInvolution,
    twisted_involution_count: usize,
}

impl TwistedConjugacyClass {
    pub(crate) fn new(representative: TwistedInvolution, twisted_involution_count: usize) -> Self {
        Self {
            representative,
            twisted_involution_count,
        }
    }

    pub fn representative(&self) -> &TwistedInvolution {
        &self.representative
    }

    pub fn twisted_involution_count(&self) -> usize {
        self.twisted_involution_count
    }
}

/// The full twisted-conjugacy partition of one inner class's twisted
/// involutions, with a membership lookup.
///
/// Classes are in the deterministic enumeration order (matrix-lexicographic;
/// NOT graded — the identity class generally sorts last). The lookup key is
/// the root-image permutation, which does not encode the datum, so
/// [`Self::class_of`] gates datum and distinguished-involution provenance
/// before the map hit; a miss after those gates is an invariant violation,
/// never a recoverable absence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TwistedConjugacyPartition {
    datum: BasedRootDatum,
    distinguished: RootInvolutionData,
    classes: Vec<TwistedConjugacyClass>,
    class_by_permutation: BTreeMap<Vec<usize>, usize>,
}

impl TwistedConjugacyPartition {
    pub(crate) fn new(
        datum: BasedRootDatum,
        distinguished: RootInvolutionData,
        classes: Vec<TwistedConjugacyClass>,
        class_by_permutation: BTreeMap<Vec<usize>, usize>,
    ) -> Self {
        Self {
            datum,
            distinguished,
            classes,
            class_by_permutation,
        }
    }

    pub fn classes(&self) -> &[TwistedConjugacyClass] {
        &self.classes
    }

    /// The index of the class containing this twisted involution.
    pub fn class_of(&self, twisted: &TwistedInvolution) -> Result<usize, StructureError> {
        if twisted.weyl_action().datum() != &self.datum {
            return Err(StructureError::DatumMismatch);
        }
        let delta = self.distinguished.involution();
        let stored = twisted.root_involution().involution();
        if compose_matrices(twisted.weyl_action().matrix(), delta.weight_matrix())?
            != stored.weight_matrix()
            || compose_matrices(
                twisted.weyl_action().coweight_matrix(),
                delta.coweight_matrix(),
            )? != stored.coweight_matrix()
        {
            return Err(StructureError::DistinguishedInvolutionMismatch);
        }
        let key: Vec<usize> = twisted
            .root_involution()
            .image_permutation()
            .iter()
            .map(|id| id.0)
            .collect();
        self.class_by_permutation.get(&key).copied().ok_or(
            StructureError::CartanClassificationInvariantViolation {
                invariant: "enumerated class lookup",
            },
        )
    }
}

/// One Cartan class as an owning value: the twisted-conjugacy class plus the
/// full validated per-Cartan machinery built at its representative.
///
/// This is the crate's partial port of Atlas `CartanClass`: the dual fiber,
/// `simpleComplex`, and the strong-real layer are later additions. All
/// components are built at the SAME representative, so every layer's
/// provenance gates hold by construction.
#[derive(Clone, Debug)]
pub struct CartanClass {
    class_info: TwistedConjugacyClass,
    decomposition: CayleyCrossDecomposition,
    grading: CartanGradingData,
    partition: WeakRealFormPartition,
    labels: RealFormLabels,
}

impl CartanClass {
    pub(crate) fn new(
        class_info: TwistedConjugacyClass,
        decomposition: CayleyCrossDecomposition,
        grading: CartanGradingData,
        partition: WeakRealFormPartition,
        labels: RealFormLabels,
    ) -> Self {
        Self {
            class_info,
            decomposition,
            grading,
            partition,
            labels,
        }
    }

    pub fn representative(&self) -> &TwistedInvolution {
        self.class_info.representative()
    }

    pub fn twisted_involution_count(&self) -> usize {
        self.class_info.twisted_involution_count()
    }

    pub fn decomposition(&self) -> &CayleyCrossDecomposition {
        &self.decomposition
    }

    pub fn grading(&self) -> &CartanGradingData {
        &self.grading
    }

    pub fn partition(&self) -> &WeakRealFormPartition {
        &self.partition
    }

    pub fn labels(&self) -> &RealFormLabels {
        &self.labels
    }
}
