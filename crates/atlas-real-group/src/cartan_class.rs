use std::collections::BTreeMap;

use crate::twisted_involution::compose_matrices;
use crate::{
    BasedRootDatum, CartanGradingData, CayleyCrossDecomposition, RealFormLabels,
    InnerClass, RootInvolutionData, StructureError, TwistedInvolution, WeakRealFormPartition,
};

/// One deterministic orbit under Weyl twisted conjugacy.
///
/// A class produced by [`TwistedConjugacyPartition`] carries the first
/// action in this crate's deterministic Weyl enumeration as its
/// representative; [`crate::CartanClassification`] rebuilds each class it
/// consumes with the Atlas-canonical representative instead
/// (`InnerClass::canonicalize`, applied to the Cayley successor before
/// numbering, innerclass.cpp:252-263). Either way the class decomposes
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
/// Explicitly enumerated partitions retain their deterministic raw order
/// (not Cartan numbering). Direct classifications instead store canonical
/// representatives in Atlas BFS order, with exact orbit sizes and on-demand
/// membership canonicalization. The lookup key
/// is the root-image permutation, which does not encode the datum, so
/// [`Self::class_of`] gates datum and distinguished-involution provenance
/// before the map hit; a miss after those gates is an invariant violation,
/// never a recoverable absence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TwistedConjugacyPartition {
    datum: BasedRootDatum,
    distinguished: RootInvolutionData,
    classes: Vec<TwistedConjugacyClass>,
    class_by_permutation: BTreeMap<Vec<u8>, usize>,
    // Representative-only partitions canonicalize membership queries on
    // demand; enumerated partitions retain their complete member map.
    canonicalizer: Option<InnerClass>,
}

impl TwistedConjugacyPartition {
    pub(crate) fn new(
        datum: BasedRootDatum,
        distinguished: RootInvolutionData,
        classes: Vec<TwistedConjugacyClass>,
        class_by_permutation: BTreeMap<Vec<u8>, usize>,
    ) -> Self {
        Self {
            datum,
            distinguished,
            classes,
            class_by_permutation,
            canonicalizer: None,
        }
    }

    pub(crate) fn from_canonical_classes(
        inner: &InnerClass,
        classes: Vec<TwistedConjugacyClass>,
    ) -> Result<Self, StructureError> {
        let mut membership = BTreeMap::new();
        for (index, class) in classes.iter().enumerate() {
            let key = class.representative().root_involution().image_permutation()
                .iter().map(|id| u8::try_from(id.0).map_err(|_| StructureError::RootSystemTooLarge))
                .collect::<Result<Vec<_>, _>>()?;
            if membership.insert(key, index).is_some() {
                return Err(StructureError::CartanClassificationInvariantViolation {
                    invariant: "distinct canonical Cartan representatives",
                });
            }
        }
        let mut result = Self::new(inner.datum().clone(),
            inner.distinguished_involution().clone(), classes, membership);
        result.canonicalizer = Some(inner.clone());
        Ok(result)
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
        let key: Vec<u8> = twisted
            .root_involution()
            .image_permutation()
            .iter()
            .map(|id| id.0 as u8)
            .collect();
        if let Some(&index) = self.class_by_permutation.get(&key) {
            return Ok(index);
        }
        if let Some(inner) = &self.canonicalizer {
            let (canonical, _) = inner.canonicalize(twisted.clone())?;
            let key = canonical.root_involution().image_permutation().iter()
                .map(|id| u8::try_from(id.0).map_err(|_| StructureError::RootSystemTooLarge))
                .collect::<Result<Vec<_>, _>>()?;
            if let Some(&index) = self.class_by_permutation.get(&key) {
                return Ok(index);
            }
        }
        Err(
            StructureError::CartanClassificationInvariantViolation {
                invariant: "enumerated class lookup",
            },
        )
    }
}

/// One Cartan class as an owning value: the twisted-conjugacy class plus the
/// full validated per-Cartan machinery built at its representative.
///
/// This is the crate's partial port of Atlas `CartanClass`: the dual fiber
/// and `simpleComplex` are later additions, while the strong-real layer
/// lives in the sibling [`crate::StrongRealClassification`]. All components
/// are built at the SAME representative, so every layer's provenance gates
/// hold by construction.
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
