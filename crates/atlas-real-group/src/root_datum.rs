use malachite::Rational;

use crate::lattice::try_copy_coordinates;
use crate::{pair, Coweight, StructureError, Weight};

/// Validated simple-root data inside a character/cocharacter lattice pair.
///
/// `lattice_rank` is the rank of the full reductive torus. `semisimple_rank`
/// is the number of simple roots. They coincide only when there is no central
/// torus. This type deliberately has no global rank cap.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct BasedRootDatum {
    lattice_rank: usize,
    cartan: Vec<Vec<i32>>,
    simple_roots: Vec<Weight>,
    simple_coroots: Vec<Coweight>,
}

impl BasedRootDatum {
    /// Construct the standard-coordinate datum determined by a Cartan matrix.
    pub fn standard(cartan: Vec<Vec<i32>>) -> Result<Self, StructureError> {
        let rank = validate_cartan(&cartan)?;
        let simple_roots = (0..rank)
            .map(|row| {
                let mut coordinates = vec![0; rank];
                coordinates[row] = 1;
                Weight::new(coordinates)
            })
            .collect();
        let simple_coroots = (0..rank)
            .map(|column| Coweight::new(cartan.iter().map(|row| row[column]).collect()))
            .collect();
        Self::from_simple_data(rank, cartan, simple_roots, simple_coroots)
    }

    /// Construct a datum with an explicit full lattice rank and simple data.
    pub fn from_simple_data(
        lattice_rank: usize,
        cartan: Vec<Vec<i32>>,
        simple_roots: Vec<Weight>,
        simple_coroots: Vec<Coweight>,
    ) -> Result<Self, StructureError> {
        let semisimple_rank = validate_cartan(&cartan)?;
        if lattice_rank < semisimple_rank {
            return Err(StructureError::RankMismatch {
                expected: semisimple_rank,
                actual: lattice_rank,
            });
        }
        if simple_roots.len() != semisimple_rank {
            return Err(StructureError::RankMismatch {
                expected: semisimple_rank,
                actual: simple_roots.len(),
            });
        }
        if simple_coroots.len() != semisimple_rank {
            return Err(StructureError::RankMismatch {
                expected: semisimple_rank,
                actual: simple_coroots.len(),
            });
        }
        for root in &simple_roots {
            ensure_lattice_rank(lattice_rank, root.rank())?;
        }
        for coroot in &simple_coroots {
            ensure_lattice_rank(lattice_rank, coroot.rank())?;
        }
        for (row, root) in simple_roots.iter().enumerate() {
            for (column, coroot) in simple_coroots.iter().enumerate() {
                let actual = pair(root, coroot)?;
                let expected = cartan[row][column];
                if actual != expected {
                    return Err(StructureError::RootPairingMismatch {
                        row,
                        column,
                        expected,
                        actual,
                    });
                }
            }
        }
        Ok(Self {
            lattice_rank,
            cartan,
            simple_roots,
            simple_coroots,
        })
    }

    pub fn lattice_rank(&self) -> usize {
        self.lattice_rank
    }

    pub fn semisimple_rank(&self) -> usize {
        self.cartan.len()
    }

    pub fn cartan_matrix(&self) -> &[Vec<i32>] {
        &self.cartan
    }

    pub fn simple_roots(&self) -> &[Weight] {
        &self.simple_roots
    }

    pub fn simple_coroots(&self) -> &[Coweight] {
        &self.simple_coroots
    }

    /// The coradical basis (rootdata.cpp:858 `lattice::perp`): a basis of
    /// the weights orthogonal to all simple coroots — the kernel of the
    /// matrix whose rows are the coroots. The `root_coradical` wrapper
    /// prints these rows after the simple roots.
    pub fn coradical_basis(&self) -> Result<Vec<Weight>, StructureError> {
        let basis = crate::integer_lattice::saturated_kernel(
            &crate::integer_lattice::IntegerMatrix::from_i32_rows(
                &self.coroot_rows(),
                &self.budget(),
            )?,
            &self.budget(),
        )?;
        let mut result = Vec::with_capacity(basis.rank());
        for column in basis.columns() {
            let mut coordinates = Vec::with_capacity(self.lattice_rank);
            for entry in column {
                coordinates
                    .push(i32::try_from(entry).map_err(|_| StructureError::ArithmeticOverflow)?);
            }
            result.push(Weight::new(coordinates));
        }
        Ok(result)
    }

    /// The radical basis (rootdata.cpp:859 `lattice::perp` of the
    /// coroots): a basis of the coweights orthogonal to all simple roots
    /// — the kernel of the matrix whose rows are the roots. The
    /// `coroot_radical` wrapper prints these rows after the simple
    /// coroots.
    pub fn radical_basis(&self) -> Result<Vec<Coweight>, StructureError> {
        let basis = crate::integer_lattice::saturated_kernel(
            &crate::integer_lattice::IntegerMatrix::from_i32_rows(
                &self.root_rows(),
                &self.budget(),
            )?,
            &self.budget(),
        )?;
        let mut result = Vec::with_capacity(basis.rank());
        for column in basis.columns() {
            let mut coordinates = Vec::with_capacity(self.lattice_rank);
            for entry in column {
                coordinates
                    .push(i32::try_from(entry).map_err(|_| StructureError::ArithmeticOverflow)?);
            }
            result.push(Coweight::new(coordinates));
        }
        Ok(result)
    }

    /// The simple-coroot rows (the coradical annihilator matrix).
    fn coroot_rows(&self) -> Vec<Vec<i32>> {
        self.simple_coroots
            .iter()
            .map(|coweight| coweight.as_slice().to_vec())
            .collect()
    }

    /// The simple-root rows (the radical annihilator matrix).
    fn root_rows(&self) -> Vec<Vec<i32>> {
        self.simple_roots
            .iter()
            .map(|weight| weight.as_slice().to_vec())
            .collect()
    }

    fn budget(&self) -> crate::IntegerLatticeBudget {
        crate::IntegerLatticeBudget::new(64, 100_000, 100_000, 128)
    }

    pub fn reflect_weight(
        &self,
        generator: usize,
        weight: &Weight,
    ) -> Result<Weight, StructureError> {
        if weight.rank() != self.lattice_rank {
            return Err(StructureError::RankMismatch {
                expected: self.lattice_rank,
                actual: weight.rank(),
            });
        }
        let root = self
            .simple_roots
            .get(generator)
            .ok_or(StructureError::IndexOutOfRange {
                index: generator,
                upper_bound: self.semisimple_rank(),
            })?;
        let coefficient = i128::from(pair(weight, &self.simple_coroots[generator])?);
        reflect_coordinates(weight.as_slice(), root.as_slice(), coefficient).map(Weight::new)
    }

    /// Dual simple reflection `y - <alpha_generator, y> alpha_generator_vee`
    /// on the cocharacter lattice, mirroring [`Self::reflect_weight`].
    pub fn reflect_coweight(
        &self,
        generator: usize,
        coweight: &Coweight,
    ) -> Result<Coweight, StructureError> {
        if coweight.rank() != self.lattice_rank {
            return Err(StructureError::RankMismatch {
                expected: self.lattice_rank,
                actual: coweight.rank(),
            });
        }
        let coroot = self
            .simple_coroots
            .get(generator)
            .ok_or(StructureError::IndexOutOfRange {
                index: generator,
                upper_bound: self.semisimple_rank(),
            })?;
        let coefficient = i128::from(pair(&self.simple_roots[generator], coweight)?);
        reflect_coordinates(coweight.as_slice(), coroot.as_slice(), coefficient).map(Coweight::new)
    }

    /// Fallible copy of the already-validated fields, avoiding both an
    /// unchecked `Clone` and a redundant `from_simple_data` re-validation.
    pub(crate) fn try_clone(&self) -> Result<Self, StructureError> {
        let mut cartan = Vec::new();
        cartan.try_reserve_exact(self.cartan.len()).map_err(|_| {
            StructureError::AllocationFailed {
                requested: self.cartan.len(),
            }
        })?;
        for row in &self.cartan {
            cartan.push(try_copy_coordinates(row)?);
        }
        let mut simple_roots = Vec::new();
        simple_roots
            .try_reserve_exact(self.simple_roots.len())
            .map_err(|_| StructureError::AllocationFailed {
                requested: self.simple_roots.len(),
            })?;
        for root in &self.simple_roots {
            simple_roots.push(Weight::new(try_copy_coordinates(root.as_slice())?));
        }
        let mut simple_coroots = Vec::new();
        simple_coroots
            .try_reserve_exact(self.simple_coroots.len())
            .map_err(|_| StructureError::AllocationFailed {
                requested: self.simple_coroots.len(),
            })?;
        for coroot in &self.simple_coroots {
            simple_coroots.push(Coweight::new(try_copy_coordinates(coroot.as_slice())?));
        }
        Ok(Self {
            lattice_rank: self.lattice_rank,
            cartan,
            simple_roots,
            simple_coroots,
        })
    }
}

fn reflect_coordinates(
    values: &[i32],
    direction: &[i32],
    coefficient: i128,
) -> Result<Vec<i32>, StructureError> {
    let mut reflected = Vec::new();
    reflected
        .try_reserve_exact(values.len())
        .map_err(|_| StructureError::AllocationFailed {
            requested: values.len(),
        })?;
    for (&coordinate, &direction_coordinate) in values.iter().zip(direction) {
        let correction = coefficient
            .checked_mul(i128::from(direction_coordinate))
            .ok_or(StructureError::ArithmeticOverflow)?;
        let value = i128::from(coordinate)
            .checked_sub(correction)
            .ok_or(StructureError::ArithmeticOverflow)?;
        reflected.push(i32::try_from(value).map_err(|_| StructureError::ArithmeticOverflow)?);
    }
    Ok(reflected)
}

fn ensure_lattice_rank(expected: usize, actual: usize) -> Result<(), StructureError> {
    if expected == actual {
        Ok(())
    } else {
        Err(StructureError::RankMismatch { expected, actual })
    }
}

fn validate_cartan(cartan: &[Vec<i32>]) -> Result<usize, StructureError> {
    let rank = cartan.len();
    if cartan.iter().any(|row| row.len() != rank) {
        return Err(StructureError::NonSquareCartan);
    }
    for (row, entries) in cartan.iter().enumerate() {
        if entries[row] != 2
            || entries
                .iter()
                .enumerate()
                .any(|(column, &entry)| row != column && entry > 0)
        {
            return Err(StructureError::InvalidCartanMatrix);
        }
    }
    for (row, entries) in cartan.iter().enumerate() {
        for (column, &entry) in entries.iter().enumerate() {
            if (entry == 0) != (cartan[column][row] == 0) {
                return Err(StructureError::InvalidCartanMatrix);
            }
        }
    }
    if !is_finite_type(cartan) {
        return Err(StructureError::InvalidCartanMatrix);
    }
    Ok(rank)
}

fn is_finite_type(cartan: &[Vec<i32>]) -> bool {
    let rank = cartan.len();
    let mut symmetrizer = vec![None; rank];
    for seed in 0..rank {
        if symmetrizer[seed].is_some() {
            continue;
        }
        symmetrizer[seed] = Some(Rational::from(1));
        let mut pending = vec![seed];
        while let Some(row) = pending.pop() {
            let row_scale = symmetrizer[row]
                .as_ref()
                .expect("pending Cartan vertex has a scale")
                .clone();
            for column in 0..rank {
                let forward = cartan[row][column];
                if forward == 0 {
                    continue;
                }
                let expected = row_scale.clone() * Rational::from(forward)
                    / Rational::from(cartan[column][row]);
                match &symmetrizer[column] {
                    Some(actual) if actual != &expected => return false,
                    Some(_) => {}
                    None => {
                        symmetrizer[column] = Some(expected);
                        pending.push(column);
                    }
                }
            }
        }
    }

    let scales = symmetrizer
        .into_iter()
        .map(|scale| scale.expect("every Cartan vertex receives a scale"))
        .collect::<Vec<_>>();
    let mut lower = vec![vec![Rational::from(0); rank]; rank];
    let mut diagonal = vec![Rational::from(0); rank];
    for column in 0..rank {
        let mut pivot = scales[column].clone() * Rational::from(cartan[column][column]);
        for (factor, diag) in lower[column][..column].iter().zip(&diagonal[..column]) {
            pivot -= factor.clone() * factor.clone() * diag.clone();
        }
        if pivot <= 0 {
            return false;
        }
        diagonal[column] = pivot;
        lower[column][column] = Rational::from(1);
        for row in (column + 1)..rank {
            let mut entry = scales[row].clone() * Rational::from(cartan[row][column]);
            for ((row_factor, column_factor), diag) in lower[row][..column]
                .iter()
                .zip(&lower[column][..column])
                .zip(&diagonal[..column])
            {
                entry -= row_factor.clone() * column_factor.clone() * diag.clone();
            }
            lower[row][column] = entry / diagonal[column].clone();
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_total_and_semisimple_rank_separate() {
        let datum = BasedRootDatum::from_simple_data(
            3,
            vec![vec![2]],
            vec![Weight::new(vec![1, 0, 0])],
            vec![Coweight::new(vec![2, 0, 0])],
        )
        .expect("A1 with a rank-two central torus is valid");
        assert_eq!(datum.lattice_rank(), 3);
        assert_eq!(datum.semisimple_rank(), 1);
    }

    #[test]
    fn accepts_rank_above_atlas_compatibility_limit() {
        let rank = 33;
        let mut cartan = vec![vec![0; rank]; rank];
        for row in 0..rank {
            cartan[row][row] = 2;
            if row + 1 < rank {
                cartan[row][row + 1] = -1;
                cartan[row + 1][row] = -1;
            }
        }
        let datum = BasedRootDatum::standard(cartan).expect("A33 is a valid dynamic datum");
        assert_eq!(datum.lattice_rank(), rank);
        assert_eq!(datum.semisimple_rank(), rank);
    }

    #[test]
    fn accepts_non_simply_laced_finite_type() {
        let datum =
            BasedRootDatum::standard(vec![vec![2, -2], vec![-1, 2]]).expect("B2 is finite type");
        assert_eq!(datum.semisimple_rank(), 2);
    }

    #[test]
    fn rejects_non_finite_cartan_matrices_and_insufficient_lattice_rank() {
        let affine = vec![vec![2, -2], vec![-2, 2]];
        assert_eq!(
            BasedRootDatum::standard(affine),
            Err(StructureError::InvalidCartanMatrix)
        );
        let indefinite = vec![vec![2, -3], vec![-3, 2]];
        assert_eq!(
            BasedRootDatum::standard(indefinite),
            Err(StructureError::InvalidCartanMatrix)
        );
        assert_eq!(
            BasedRootDatum::from_simple_data(
                1,
                vec![vec![2], vec![0, 2]],
                vec![Weight::new(vec![1]), Weight::new(vec![0])],
                vec![Coweight::new(vec![2]), Coweight::new(vec![0])],
            ),
            Err(StructureError::NonSquareCartan)
        );
        assert_eq!(
            BasedRootDatum::from_simple_data(
                0,
                vec![vec![2]],
                vec![Weight::new(vec![1])],
                vec![Coweight::new(vec![2])],
            ),
            Err(StructureError::RankMismatch {
                expected: 1,
                actual: 0,
            })
        );
    }

    #[test]
    fn reflects_coweights_dually_and_rejects_narrowing_overflow() {
        let datum = BasedRootDatum::from_simple_data(
            2,
            vec![vec![2]],
            vec![Weight::new(vec![1, 0])],
            vec![Coweight::new(vec![2, 0])],
        )
        .unwrap();
        assert_eq!(
            datum.reflect_coweight(0, &Coweight::new(vec![7, 11])),
            Ok(Coweight::new(vec![-7, 11]))
        );
        assert_eq!(
            datum.reflect_coweight(0, &Coweight::new(vec![i32::MIN, 5])),
            Err(StructureError::ArithmeticOverflow)
        );
        assert_eq!(
            datum.reflect_coweight(1, &Coweight::new(vec![0, 0])),
            Err(StructureError::IndexOutOfRange {
                index: 1,
                upper_bound: 1,
            })
        );
    }

    #[test]
    fn fallible_clone_reproduces_the_datum() {
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        assert_eq!(datum.try_clone().unwrap(), datum);
    }

    #[test]
    fn supports_a_central_torus_without_roots() {
        let datum = BasedRootDatum::from_simple_data(3, vec![], vec![], vec![])
            .expect("a torus has no roots but has positive lattice rank");
        assert_eq!(datum.lattice_rank(), 3);
        assert_eq!(datum.semisimple_rank(), 0);
    }
}
