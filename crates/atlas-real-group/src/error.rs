use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StructureError {
    EmptyRootDatum,
    NonSquareCartan,
    InvalidCartanMatrix,
    RankMismatch {
        expected: usize,
        actual: usize,
    },
    DatumMismatch,
    IndexOutOfRange {
        index: usize,
        upper_bound: usize,
    },
    InvalidInvolution,
    InvalidBasedAutomorphism,
    InvalidRootAutomorphism,
    InvalidRootDatumAutomorphism,
    RootSystemTooLarge,
    WeylGroupTooLarge,
    ResourceLimitExceeded {
        limit: usize,
    },
    AllocationFailed {
        requested: usize,
    },
    InvalidIntegerMatrixShape,
    IntegerLatticeInvariantViolation,
    IntegerLatticeResourceLimit {
        resource: &'static str,
        limit: u64,
    },
    RootSystemResourceLimit {
        resource: &'static str,
        limit: usize,
    },
    RootSystemInvariantViolation {
        invariant: &'static str,
    },
    GradingShiftsNotFaithful,
    ImpossibleGrading,
    WeakRealFormResourceLimit {
        resource: &'static str,
        limit: usize,
    },
    CayleyCrossResourceLimit {
        resource: &'static str,
        limit: usize,
    },
    CayleyCrossInvariantViolation {
        invariant: &'static str,
    },
    RealFormLabelInvariantViolation {
        invariant: &'static str,
    },
    CartanClassificationInvariantViolation {
        invariant: &'static str,
    },
    StrongRealResourceLimit {
        resource: &'static str,
        limit: usize,
    },
    StrongRealInvariantViolation {
        invariant: &'static str,
    },
    WeylElementInvariantViolation {
        invariant: &'static str,
    },
    DistinguishedInvolutionMismatch,
    ModTwoSubquotientInvariantViolation,
    NotInModTwoSubspace,
    CartanFiberMismatch,
    CartanFiberInvolutionMismatch,
    CartanFiberMapDoesNotDescend {
        relation: &'static str,
    },
    AdjointFiberResourceLimit {
        resource: &'static str,
        limit: usize,
    },
    ArithmeticOverflow,
    RootPairingMismatch {
        row: usize,
        column: usize,
        expected: i32,
        actual: i32,
    },
}

impl fmt::Display for StructureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyRootDatum => write!(f, "root datum must have positive rank"),
            Self::NonSquareCartan => write!(f, "Cartan matrix must be square"),
            Self::InvalidCartanMatrix => write!(f, "invalid generalized Cartan matrix"),
            Self::RankMismatch { expected, actual } => write!(
                f,
                "lattice rank mismatch: expected {expected}, got {actual}"
            ),
            Self::DatumMismatch => write!(f, "operations require the same based root datum"),
            Self::IndexOutOfRange { index, upper_bound } => {
                write!(f, "index {index} is outside the range 0..{upper_bound}")
            }
            Self::InvalidInvolution => write!(f, "Cartan involution is not an involution"),
            Self::InvalidBasedAutomorphism => {
                write!(
                    f,
                    "distinguished involution does not preserve the simple root/coroot system"
                )
            }
            Self::InvalidRootAutomorphism => {
                write!(f, "root automorphism does not preserve the Cartan pairing")
            }
            Self::InvalidRootDatumAutomorphism => {
                write!(f, "root permutation does not transport coroots to coroots")
            }
            Self::RootSystemTooLarge => {
                write!(f, "root-system closure exceeded the finite-system limit")
            }
            Self::WeylGroupTooLarge => {
                write!(f, "Weyl-group closure exceeded the finite-system limit")
            }
            Self::ResourceLimitExceeded { limit } => {
                write!(f, "enumeration exceeded its resource limit of {limit}")
            }
            Self::AllocationFailed { requested } => {
                write!(f, "could not reserve storage for {requested} elements")
            }
            Self::InvalidIntegerMatrixShape => {
                write!(f, "integer matrix has an invalid shape")
            }
            Self::IntegerLatticeInvariantViolation => {
                write!(f, "integer-lattice reduction invariant was violated")
            }
            Self::IntegerLatticeResourceLimit { resource, limit } => {
                write!(
                    f,
                    "integer-lattice {resource} exceeded its limit of {limit}"
                )
            }
            Self::RootSystemResourceLimit { resource, limit } => {
                write!(f, "root-system {resource} exceeded its limit of {limit}")
            }
            Self::RootSystemInvariantViolation { invariant } => {
                write!(f, "root-system {invariant} invariant was violated")
            }
            Self::GradingShiftsNotFaithful => {
                write!(f, "grading shifts do not determine a unique fiber element")
            }
            Self::ImpossibleGrading => {
                write!(f, "no fiber element has the requested grading")
            }
            Self::WeakRealFormResourceLimit { resource, limit } => {
                write!(f, "weak-real-form {resource} exceeded its limit of {limit}")
            }
            Self::CayleyCrossResourceLimit { resource, limit } => {
                write!(f, "Cayley-cross {resource} exceeded its limit of {limit}")
            }
            Self::CayleyCrossInvariantViolation { invariant } => {
                write!(f, "Cayley-cross {invariant} invariant was violated")
            }
            Self::RealFormLabelInvariantViolation { invariant } => {
                write!(f, "real-form-label {invariant} invariant was violated")
            }
            Self::CartanClassificationInvariantViolation { invariant } => {
                write!(
                    f,
                    "Cartan-classification {invariant} invariant was violated"
                )
            }
            Self::StrongRealResourceLimit { resource, limit } => {
                write!(f, "strong-real {resource} exceeded its limit of {limit}")
            }
            Self::StrongRealInvariantViolation { invariant } => {
                write!(f, "strong-real {invariant} invariant was violated")
            }
            Self::WeylElementInvariantViolation { invariant } => {
                write!(f, "Weyl-element {invariant} invariant was violated")
            }
            Self::DistinguishedInvolutionMismatch => {
                write!(
                    f,
                    "twisted involution does not factor through this inner class's distinguished involution"
                )
            }
            Self::ModTwoSubquotientInvariantViolation => {
                write!(f, "mod-two subquotient invariant was violated")
            }
            Self::NotInModTwoSubspace => {
                write!(f, "mod-two vector does not belong to the required subspace")
            }
            Self::CartanFiberMismatch => {
                write!(f, "Cartan-fiber elements belong to different fibers")
            }
            Self::CartanFiberInvolutionMismatch => {
                write!(f, "Cartan-fiber map requires the same Cartan involution")
            }
            Self::CartanFiberMapDoesNotDescend { relation } => {
                write!(
                    f,
                    "Cartan-fiber map does not preserve the {relation} relation"
                )
            }
            Self::AdjointFiberResourceLimit { resource, limit } => {
                write!(f, "adjoint-fiber {resource} exceeded its limit of {limit}")
            }
            Self::ArithmeticOverflow => write!(f, "root-system arithmetic overflow"),
            Self::RootPairingMismatch {
                row,
                column,
                expected,
                actual,
            } => write!(
                f,
                "root pairing ({row},{column}) is {actual}, expected {expected}"
            ),
        }
    }
}

impl std::error::Error for StructureError {}
