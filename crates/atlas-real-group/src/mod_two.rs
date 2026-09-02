use smallvec::SmallVec;

use crate::StructureError;

/// A dynamically sized vector over `F_2`.
///
/// This represents finite mod-two quotient data used by Cartan fibers. It is
/// deliberately separate from the Malachite-backed integer layer: its words
/// encode elements of a finite vector space, not unbounded integer values.
/// Storage is inline up to two words (128 bits — every Atlas lattice rank,
/// RANK_MAX = 32, fits with room), so the millions of per-element clones in
/// the KGB BFS never touch the heap; larger dimensions spill to the heap.
///
/// The derived `Ord` is an arbitrary but deterministic total order for map
/// keys, not a mathematical order. The derives are sound because every
/// constructor zeroes the padding bits above `dimension` and keeps them zero.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ModTwoVector {
    dimension: usize,
    words: SmallVec<[u64; 2]>,
}

impl ModTwoVector {
    pub fn zero(dimension: usize) -> Result<Self, StructureError> {
        let word_count = word_count(dimension)?;
        let mut words = SmallVec::new();
        words
            .try_reserve_exact(word_count)
            .map_err(|_| StructureError::AllocationFailed {
                requested: word_count,
            })?;
        words.resize(word_count, 0);
        Ok(Self { dimension, words })
    }

    pub fn from_ones(
        dimension: usize,
        indices: impl IntoIterator<Item = usize>,
    ) -> Result<Self, StructureError> {
        let mut vector = Self::zero(dimension)?;
        for index in indices {
            vector.toggle(index)?;
        }
        Ok(vector)
    }

    pub fn dimension(&self) -> usize {
        self.dimension
    }

    pub fn bit(&self, index: usize) -> Option<bool> {
        (index < self.dimension).then(|| {
            self.words[index / u64::BITS as usize] & (1_u64 << (index % u64::BITS as usize)) != 0
        })
    }

    /// The packed words, for bulk parity arithmetic against precomputed
    /// masks (the padding bits above `dimension` are zero by contract).
    pub(crate) fn words(&self) -> &[u64] {
        &self.words
    }

    pub fn xor_assign(&mut self, right: &Self) -> Result<(), StructureError> {
        if self.dimension != right.dimension {
            return Err(StructureError::RankMismatch {
                expected: self.dimension,
                actual: right.dimension,
            });
        }
        for (left_word, right_word) in self.words.iter_mut().zip(&right.words) {
            *left_word ^= right_word;
        }
        Ok(())
    }

    pub fn is_zero(&self) -> bool {
        self.words.iter().all(|&word| word == 0)
    }

    /// The `F_2` pairing: parity of the coordinatewise product.
    pub(crate) fn dot(&self, right: &Self) -> Result<bool, StructureError> {
        if self.dimension != right.dimension {
            return Err(StructureError::RankMismatch {
                expected: self.dimension,
                actual: right.dimension,
            });
        }
        let parity = self
            .words
            .iter()
            .zip(&right.words)
            .fold(0, |parity, (left_word, right_word)| {
                parity ^ ((left_word & right_word).count_ones() & 1)
            });
        Ok(parity == 1)
    }

    fn toggle(&mut self, index: usize) -> Result<(), StructureError> {
        if index >= self.dimension {
            return Err(StructureError::IndexOutOfRange {
                index,
                upper_bound: self.dimension,
            });
        }
        self.words[index / u64::BITS as usize] ^= 1_u64 << (index % u64::BITS as usize);
        Ok(())
    }
}

/// An ambient map between dynamic mod-two spaces.
///
/// Domain layers can implement this directly when a dense matrix would be an
/// unnecessary allocation. The quotient layer needs only dimensions and the
/// ability to apply the map to its basis vectors.
pub(crate) trait ModTwoAmbientMap {
    fn source_dimension(&self) -> usize;

    fn target_dimension(&self) -> usize;

    fn apply(&self, source: &ModTwoVector) -> Result<ModTwoVector, StructureError>;
}

/// A dense test helper for quotient-descent checks.
///
/// Production domain maps implement [`ModTwoAmbientMap`] on demand, avoiding a
/// cached column for every source coordinate.
#[cfg(test)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ModTwoLinearMap {
    target_dimension: usize,
    columns: Vec<ModTwoVector>,
}

#[cfg(test)]
impl ModTwoLinearMap {
    pub(crate) fn new(
        target_dimension: usize,
        columns: Vec<ModTwoVector>,
    ) -> Result<Self, StructureError> {
        for column in &columns {
            if column.dimension != target_dimension {
                return Err(StructureError::RankMismatch {
                    expected: target_dimension,
                    actual: column.dimension,
                });
            }
        }
        Ok(Self {
            target_dimension,
            columns,
        })
    }
}

#[cfg(test)]
impl ModTwoAmbientMap for ModTwoLinearMap {
    fn source_dimension(&self) -> usize {
        self.columns.len()
    }

    fn target_dimension(&self) -> usize {
        self.target_dimension
    }

    fn apply(&self, source: &ModTwoVector) -> Result<ModTwoVector, StructureError> {
        if source.dimension != self.source_dimension() {
            return Err(StructureError::RankMismatch {
                expected: self.source_dimension(),
                actual: source.dimension,
            });
        }
        let mut target = ModTwoVector::zero(self.target_dimension)?;
        for (index, column) in self.columns.iter().enumerate() {
            if source.bit(index) == Some(true) {
                target.xor_assign(column)?;
            }
        }
        Ok(target)
    }
}

/// A canonical row-reduced subspace of a dynamic `F_2` vector space.
///
/// Pivots are the lowest nonzero coordinates, following the low-pivot
/// convention observed in Atlas `BitVector::firstBit()`. The basis is reduced
/// at every pivot, so it is independent of insertion order and supplies
/// deterministic coordinates for this structural subquotient layer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModTwoSubspace {
    dimension: usize,
    pivots: Vec<Option<ModTwoVector>>,
    rank: usize,
}

impl ModTwoSubspace {
    pub fn new(dimension: usize) -> Result<Self, StructureError> {
        let mut pivots = Vec::new();
        pivots
            .try_reserve_exact(dimension)
            .map_err(|_| StructureError::AllocationFailed {
                requested: dimension,
            })?;
        pivots.resize(dimension, None);
        Ok(Self {
            dimension,
            pivots,
            rank: 0,
        })
    }

    pub fn dimension(&self) -> usize {
        self.dimension
    }

    pub fn rank(&self) -> usize {
        self.rank
    }

    pub fn quotient_dimension(&self) -> usize {
        self.dimension - self.rank
    }

    /// Insert a vector, returning whether it increases the subspace rank.
    pub fn insert(&mut self, vector: ModTwoVector) -> Result<bool, StructureError> {
        let reduced = self.reduce(vector)?;
        let Some(pivot) = lowest_set_bit(&reduced) else {
            return Ok(false);
        };

        // `reduced` has no existing pivot. Clear its new pivot from the older
        // rows as well, turning the echelon basis into a canonical RREF basis.
        for existing in self.pivots.iter_mut().flatten() {
            if existing.bit(pivot) == Some(true) {
                existing.xor_assign(&reduced)?;
            }
        }
        self.pivots[pivot] = Some(reduced);
        self.rank = self
            .rank
            .checked_add(1)
            .ok_or(StructureError::ArithmeticOverflow)?;
        Ok(true)
    }

    pub fn contains(&self, vector: ModTwoVector) -> Result<bool, StructureError> {
        Ok(lowest_set_bit(&self.reduce(vector)?).is_none())
    }

    /// The deterministic row-echelon representative of a quotient class.
    pub fn quotient_representative(
        &self,
        vector: ModTwoVector,
    ) -> Result<ModTwoVector, StructureError> {
        self.reduce(vector)
    }

    pub fn same_coset(
        &self,
        left: ModTwoVector,
        right: ModTwoVector,
    ) -> Result<bool, StructureError> {
        if left.dimension != right.dimension {
            return Err(StructureError::RankMismatch {
                expected: left.dimension,
                actual: right.dimension,
            });
        }
        let mut difference = left;
        difference.xor_assign(&right)?;
        self.contains(difference)
    }

    fn reduce(&self, mut vector: ModTwoVector) -> Result<ModTwoVector, StructureError> {
        if vector.dimension != self.dimension {
            return Err(StructureError::RankMismatch {
                expected: self.dimension,
                actual: vector.dimension,
            });
        }
        // A missing early pivot does not imply that later pivots are absent.
        // The canonical basis is already reduced at every pivot, so scanning in
        // increasing order clears every coefficient exactly once, just as
        // Atlas `normalSpanAdd` does before it adds a new row.
        for (pivot, basis_vector) in self.pivots.iter().enumerate() {
            if vector.bit(pivot) == Some(true) {
                if let Some(basis_vector) = basis_vector {
                    vector.xor_assign(basis_vector)?;
                }
            }
        }
        Ok(vector)
    }

    pub(crate) fn right_kernel(&self) -> Result<Self, StructureError> {
        let mut kernel = Self::new(self.dimension)?;
        for free_coordinate in 0..self.dimension {
            if self.pivots[free_coordinate].is_some() {
                continue;
            }
            let capacity = self
                .rank
                .checked_add(1)
                .ok_or(StructureError::ArithmeticOverflow)?;
            let mut ones = Vec::new();
            ones.try_reserve_exact(capacity)
                .map_err(|_| StructureError::AllocationFailed {
                    requested: capacity,
                })?;
            ones.push(free_coordinate);
            for (pivot, equation) in self.pivots.iter().enumerate() {
                if equation
                    .as_ref()
                    .is_some_and(|row| row.bit(free_coordinate) == Some(true))
                {
                    ones.push(pivot);
                }
            }
            kernel.insert(ModTwoVector::from_ones(self.dimension, ones)?)?;
        }
        Ok(kernel)
    }

    /// Pivot rows with their pivot coordinates, in ascending pivot order.
    ///
    /// This is exactly the sorted canonical basis upstream `Gauss_Jordan`
    /// commits (bitvector.cpp:673-697); the R-group kernel construction of
    /// `real_weyl.rs` reads generator bits off these rows, where
    /// [`Self::right_kernel`] would re-reduce them into a fresh subspace.
    pub(crate) fn pivot_rows(&self) -> impl Iterator<Item = (usize, &ModTwoVector)> {
        self.pivots
            .iter()
            .enumerate()
            .filter_map(|(pivot, row)| row.as_ref().map(|row| (pivot, row)))
    }

    pub(crate) fn basis_vectors(&self) -> impl Iterator<Item = &ModTwoVector> {
        self.pivots.iter().filter_map(Option::as_ref)
    }
}

/// A canonical quotient of two `F_2` subspaces in a common ambient space.
///
/// This is deliberately crate-private: it owns the low-pivot packed coordinates
/// used by the current Cartan-fiber structural layer, while the public API is
/// the mathematical `CartanFiber` wrapper rather than a serialization format.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ModTwoSubquotient {
    numerator: ModTwoSubspace,
    denominator: ModTwoSubspace,
    complement_basis: Vec<ModTwoVector>,
}

impl ModTwoSubquotient {
    pub(crate) fn new(
        numerator: ModTwoSubspace,
        denominator: ModTwoSubspace,
    ) -> Result<Self, StructureError> {
        if numerator.dimension != denominator.dimension {
            return Err(StructureError::RankMismatch {
                expected: numerator.dimension,
                actual: denominator.dimension,
            });
        }
        if denominator.rank > numerator.rank {
            return Err(StructureError::ModTwoSubquotientInvariantViolation);
        }
        for vector in denominator.basis_vectors() {
            if !numerator.contains(vector.clone())? {
                return Err(StructureError::ModTwoSubquotientInvariantViolation);
            }
        }

        let complement_count = numerator
            .rank
            .checked_sub(denominator.rank)
            .ok_or(StructureError::ModTwoSubquotientInvariantViolation)?;
        let mut complement_basis = Vec::new();
        complement_basis
            .try_reserve_exact(complement_count)
            .map_err(|_| StructureError::AllocationFailed {
                requested: complement_count,
            })?;
        for (pivot, vector) in numerator.pivots.iter().enumerate() {
            if denominator.pivots[pivot].is_none() {
                if let Some(vector) = vector {
                    complement_basis.push(vector.clone());
                }
            }
        }
        if complement_basis.len() != complement_count {
            return Err(StructureError::ModTwoSubquotientInvariantViolation);
        }

        Ok(Self {
            numerator,
            denominator,
            complement_basis,
        })
    }

    pub(crate) fn ambient_dimension(&self) -> usize {
        self.numerator.dimension
    }

    pub(crate) fn dimension(&self) -> usize {
        self.complement_basis.len()
    }

    pub(crate) fn canonical_representative(
        &self,
        vector: ModTwoVector,
    ) -> Result<ModTwoVector, StructureError> {
        if vector.dimension != self.numerator.dimension {
            return Err(StructureError::RankMismatch {
                expected: self.numerator.dimension,
                actual: vector.dimension,
            });
        }
        if !self.numerator.contains(vector.clone())? {
            return Err(StructureError::NotInModTwoSubspace);
        }
        self.denominator.quotient_representative(vector)
    }

    pub(crate) fn to_coordinates(
        &self,
        vector: ModTwoVector,
    ) -> Result<ModTwoVector, StructureError> {
        let representative = self.canonical_representative(vector)?;
        let mut indices = Vec::new();
        indices.try_reserve_exact(self.dimension()).map_err(|_| {
            StructureError::AllocationFailed {
                requested: self.dimension(),
            }
        })?;
        for (index, vector) in self.complement_basis.iter().enumerate() {
            let pivot = lowest_set_bit(vector)
                .ok_or(StructureError::ModTwoSubquotientInvariantViolation)?;
            if representative.bit(pivot) == Some(true) {
                indices.push(index);
            }
        }
        ModTwoVector::from_ones(self.dimension(), indices)
    }

    pub(crate) fn ambient_representative(
        &self,
        coordinates: &ModTwoVector,
    ) -> Result<ModTwoVector, StructureError> {
        if coordinates.dimension != self.dimension() {
            return Err(StructureError::RankMismatch {
                expected: self.dimension(),
                actual: coordinates.dimension,
            });
        }
        let mut representative = ModTwoVector::zero(self.ambient_dimension())?;
        for (index, vector) in self.complement_basis.iter().enumerate() {
            if coordinates.bit(index) == Some(true) {
                representative.xor_assign(vector)?;
            }
        }
        Ok(representative)
    }

    pub(crate) fn basis_representatives(&self) -> &[ModTwoVector] {
        &self.complement_basis
    }

    /// Verify that an ambient map descends through both quotient relations.
    ///
    /// Checking only quotient-basis representatives would make a normal-form
    /// choice appear to define a map even when a source denominator vector has
    /// a nonzero target class.
    pub(crate) fn validate_induced_map_to(
        &self,
        target: &Self,
        map: &impl ModTwoAmbientMap,
    ) -> Result<(), StructureError> {
        if map.source_dimension() != self.ambient_dimension() {
            return Err(StructureError::RankMismatch {
                expected: self.ambient_dimension(),
                actual: map.source_dimension(),
            });
        }
        if map.target_dimension() != target.ambient_dimension() {
            return Err(StructureError::RankMismatch {
                expected: target.ambient_dimension(),
                actual: map.target_dimension(),
            });
        }
        for vector in self.numerator.basis_vectors() {
            if !target.numerator.contains(map.apply(vector)?)? {
                return Err(StructureError::CartanFiberMapDoesNotDescend {
                    relation: "numerator",
                });
            }
        }
        for vector in self.denominator.basis_vectors() {
            if !target.denominator.contains(map.apply(vector)?)? {
                return Err(StructureError::CartanFiberMapDoesNotDescend {
                    relation: "denominator",
                });
            }
        }
        Ok(())
    }
}

fn word_count(dimension: usize) -> Result<usize, StructureError> {
    dimension
        .checked_add(u64::BITS as usize - 1)
        .ok_or(StructureError::ArithmeticOverflow)
        .map(|rounded| rounded / u64::BITS as usize)
}

fn lowest_set_bit(vector: &ModTwoVector) -> Option<usize> {
    vector
        .words
        .iter()
        .enumerate()
        .find_map(|(word_index, &word)| {
            (word != 0).then(|| word_index * u64::BITS as usize + word.trailing_zeros() as usize)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dynamically_tracks_a_subspace_above_the_atlas_packed_rank_limit() {
        let mut subspace = ModTwoSubspace::new(130).unwrap();
        for index in [0, 32, 64, 129] {
            assert!(subspace
                .insert(ModTwoVector::from_ones(130, [index]).unwrap())
                .unwrap());
        }
        assert_eq!(subspace.rank(), 4);
        assert!(subspace
            .contains(ModTwoVector::from_ones(130, [0, 64, 129]).unwrap())
            .unwrap());
        assert!(!subspace
            .contains(ModTwoVector::from_ones(130, [1]).unwrap())
            .unwrap());
    }

    #[test]
    fn reduction_detects_dependent_vectors_and_dimension_mismatches() {
        let mut subspace = ModTwoSubspace::new(3).unwrap();
        assert!(subspace
            .insert(ModTwoVector::from_ones(3, [0, 2]).unwrap())
            .unwrap());
        assert!(!subspace
            .insert(ModTwoVector::from_ones(3, [0, 2]).unwrap())
            .unwrap());
        assert_eq!(
            subspace.insert(ModTwoVector::zero(2).unwrap()),
            Err(StructureError::RankMismatch {
                expected: 3,
                actual: 2,
            })
        );
    }

    #[test]
    fn exposes_deterministic_quotient_representatives_and_cosets() {
        let mut subspace = ModTwoSubspace::new(4).unwrap();
        subspace
            .insert(ModTwoVector::from_ones(4, [1, 3]).unwrap())
            .unwrap();
        let left = ModTwoVector::from_ones(4, [0, 1, 3]).unwrap();
        let right = ModTwoVector::from_ones(4, [0]).unwrap();

        assert_eq!(subspace.quotient_dimension(), 3);
        assert_eq!(
            subspace.quotient_representative(left.clone()),
            Ok(right.clone())
        );
        assert!(subspace.same_coset(left, right).unwrap());
    }

    #[test]
    fn handles_zero_dimension_and_word_boundaries_with_finite_field_parity() {
        let vector = ModTwoVector::from_ones(129, [0, 63, 64, 127, 128, 63]).unwrap();
        assert_eq!(vector.bit(0), Some(true));
        assert_eq!(vector.bit(63), Some(false));
        assert_eq!(vector.bit(64), Some(true));
        assert_eq!(vector.bit(127), Some(true));
        assert_eq!(vector.bit(128), Some(true));
        assert_eq!(vector.bit(129), None);

        let mut zero_subspace = ModTwoSubspace::new(0).unwrap();
        assert!(!zero_subspace
            .insert(ModTwoVector::zero(0).unwrap())
            .unwrap());
        assert!(zero_subspace
            .contains(ModTwoVector::zero(0).unwrap())
            .unwrap());
    }

    #[test]
    fn eliminates_dependent_vectors_across_multiple_words() {
        let mut subspace = ModTwoSubspace::new(129).unwrap();
        assert!(subspace
            .insert(ModTwoVector::from_ones(129, [0, 64]).unwrap())
            .unwrap());
        assert!(subspace
            .insert(ModTwoVector::from_ones(129, [64, 128]).unwrap())
            .unwrap());
        assert!(!subspace
            .insert(ModTwoVector::from_ones(129, [0, 128]).unwrap())
            .unwrap());
    }

    #[test]
    fn insertion_keeps_the_basis_fully_reduced() {
        let mut subspace = ModTwoSubspace::new(3).unwrap();
        subspace
            .insert(ModTwoVector::from_ones(3, [1]).unwrap())
            .unwrap();
        subspace
            .insert(ModTwoVector::from_ones(3, [0, 1]).unwrap())
            .unwrap();

        assert_eq!(
            subspace.pivots[0],
            Some(ModTwoVector::from_ones(3, [0]).unwrap())
        );
        assert_eq!(
            subspace.pivots[1],
            Some(ModTwoVector::from_ones(3, [1]).unwrap())
        );
    }

    #[test]
    fn computes_right_kernels_from_a_canonical_row_space() {
        let mut equations = ModTwoSubspace::new(3).unwrap();
        equations
            .insert(ModTwoVector::from_ones(3, [1, 2]).unwrap())
            .unwrap();
        equations
            .insert(ModTwoVector::from_ones(3, [0, 1]).unwrap())
            .unwrap();

        let kernel = equations.right_kernel().unwrap();
        assert_eq!(kernel.rank(), 1);
        assert!(kernel
            .contains(ModTwoVector::from_ones(3, [0, 1, 2]).unwrap())
            .unwrap());
        assert!(!kernel
            .contains(ModTwoVector::from_ones(3, [1, 2]).unwrap())
            .unwrap());
    }

    #[test]
    fn subquotient_uses_low_pivot_deterministic_coordinates() {
        let mut numerator = ModTwoSubspace::new(2).unwrap();
        numerator
            .insert(ModTwoVector::from_ones(2, [0]).unwrap())
            .unwrap();
        numerator
            .insert(ModTwoVector::from_ones(2, [1]).unwrap())
            .unwrap();
        let mut denominator = ModTwoSubspace::new(2).unwrap();
        denominator
            .insert(ModTwoVector::from_ones(2, [0, 1]).unwrap())
            .unwrap();

        let quotient = ModTwoSubquotient::new(numerator, denominator).unwrap();
        let first = ModTwoVector::from_ones(2, [0]).unwrap();
        let second = ModTwoVector::from_ones(2, [1]).unwrap();

        assert_eq!(quotient.dimension(), 1);
        assert_eq!(
            quotient.canonical_representative(first.clone()).unwrap(),
            second
        );
        assert_eq!(
            quotient.to_coordinates(second).unwrap(),
            ModTwoVector::from_ones(1, [0]).unwrap()
        );
        assert_eq!(
            quotient
                .ambient_representative(&ModTwoVector::from_ones(1, [0]).unwrap())
                .unwrap(),
            ModTwoVector::from_ones(2, [1]).unwrap()
        );
    }

    #[test]
    fn rejects_oversized_dimensions_without_attempting_an_infallible_allocation() {
        assert_eq!(
            ModTwoVector::zero(usize::MAX),
            Err(StructureError::ArithmeticOverflow)
        );
        assert!(matches!(
            ModTwoSubspace::new(usize::MAX),
            Err(StructureError::AllocationFailed { .. })
        ));
    }
}
