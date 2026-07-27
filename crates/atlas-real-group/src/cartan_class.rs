use crate::TwistedInvolution;

/// One deterministic orbit under Weyl twisted conjugacy.
///
/// The representative is the first action in this crate's deterministic Weyl
/// enumeration, not Atlas's `canonicalize` representative. Fiber groups,
/// weak/strong real forms, and real Cartan component data are later additions.
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
