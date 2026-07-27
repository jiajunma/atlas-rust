use crate::grading::try_capacity;
use crate::mod_two::ModTwoSubquotient;
use crate::weak_real_form::{walk_mask_orbits, MAX_MASK_BITS};
use crate::{
    AdjointFiberElement, CartanClassification, CartanId, ModTwoSubspace, ModTwoVector,
    StructureError, WeakRealFormId,
};

/// Stable identifier for one central square class of one Cartan's fiber.
///
/// The number IS the coset coordinate integer of the class in the crate's
/// echelon basis of (adjoint fiber group) / im(toAdjoint); it is
/// deterministic here but basis-convention-dependent, so Atlas's numbers
/// stay under the standing adapter deferral. The partition structure and
/// all sizes are convention-invariant.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SquareClassId(pub(crate) usize);

/// A strong real form representative: a `W_im` orbit in the fiber group of
/// one square class, lying over one weak real form.
///
/// The `fiber_orbit` NUMBER depends on the particular solution this crate's
/// elimination picks — upstream's convention differs — but the orbit's
/// class size does not: translation by `ker(toAdjoint)` commutes with the
/// action.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct StrongRealFormRep {
    fiber_orbit: usize,
    square_class: SquareClassId,
}

impl StrongRealFormRep {
    pub fn fiber_orbit(&self) -> usize {
        self.fiber_orbit
    }

    pub fn square_class(&self) -> SquareClassId {
        self.square_class
    }
}

/// The strong-real layer of one Cartan class: square classes, per-square
/// fiber-orbit sizes, and strong representatives.
#[derive(Clone, Debug)]
pub struct StrongRealData {
    central_square_classes: Vec<SquareClassId>,
    class_bases: Vec<AdjointFiberElement>,
    orbit_sizes: Vec<Vec<usize>>,
    strong_representatives: Vec<StrongRealFormRep>,
}

impl StrongRealData {
    pub fn square_class_count(&self) -> usize {
        self.class_bases.len()
    }

    /// Bounded by the Cartan's local weak class count.
    pub fn central_square_class(&self, local: WeakRealFormId) -> Option<SquareClassId> {
        self.central_square_classes.get(local.0).copied()
    }

    /// Bounded by the Cartan's local weak class count.
    pub fn strong_real_form(&self, local: WeakRealFormId) -> Option<StrongRealFormRep> {
        self.strong_representatives.get(local.0).copied()
    }

    /// The number of `W_im` orbits in one square class's fiber partition.
    pub fn fiber_orbit_count(&self, square: SquareClassId) -> Option<usize> {
        self.orbit_sizes.get(square.0).map(Vec::len)
    }

    /// The fiber size of one local weak class: the class size of its strong
    /// representative's orbit.
    pub fn fiber_size(&self, local: WeakRealFormId) -> Option<usize> {
        let representative = self.strong_real_form(local)?;
        self.orbit_sizes
            .get(representative.square_class.0)?
            .get(representative.fiber_orbit)
            .copied()
    }
}

/// The strong-real layer of a whole inner class, with the KGB sizes — the
/// first differential numbers comparable against published Atlas values.
#[derive(Clone, Debug)]
pub struct StrongRealClassification {
    per_cartan: Vec<StrongRealData>,
    local_of_form: Vec<Vec<Option<usize>>>,
    kgb_sizes: Vec<usize>,
    global_kgb_size: usize,
}

impl StrongRealClassification {
    /// Build the strong-real layer over a finished Cartan classification.
    pub fn build(
        classification: &CartanClassification,
        max_fiber_elements: usize,
    ) -> Result<Self, StructureError> {
        let mut per_cartan = try_capacity(classification.cartan_classes().len())?;
        let mut global_kgb_size = 0_usize;
        for cartan_class in classification.cartan_classes() {
            let grading = cartan_class.grading();
            let adjoint = grading.adjoint_fiber();
            let ambient = adjoint.ambient_fiber();
            // The accessor constructs the map, so obtain it once per Cartan.
            let fiber_map = adjoint.fiber_map();
            let weak = cartan_class.partition();
            let fiber_dimension = ambient.dimension();
            let adjoint_dimension = adjoint.dimension();
            if fiber_dimension > MAX_MASK_BITS {
                return Err(StructureError::StrongRealResourceLimit {
                    resource: "mask bits",
                    limit: MAX_MASK_BITS,
                });
            }

            // Ambient basis images and their adjoint coordinates.
            let mut image_elements = try_capacity(fiber_dimension)?;
            let mut image_coordinates: Vec<ModTwoVector> = try_capacity(fiber_dimension)?;
            for representative in ambient.basis_representatives() {
                let element = ambient.element_from_ambient(representative.clone())?;
                let image = fiber_map.apply(&element)?;
                image_coordinates.push(adjoint.coordinates(&image)?.clone());
                image_elements.push(image);
            }

            // The square quotient: full-space numerator over im(toAdjoint).
            let mut numerator = ModTwoSubspace::new(adjoint_dimension)?;
            for index in 0..adjoint_dimension {
                numerator.insert(ModTwoVector::from_ones(adjoint_dimension, [index])?)?;
            }
            let mut denominator = ModTwoSubspace::new(adjoint_dimension)?;
            for coordinates in &image_coordinates {
                denominator.insert(coordinates.clone())?;
            }
            let quotient = ModTwoSubquotient::new(numerator, denominator)?;
            let quotient_dimension = quotient.dimension();
            let square_class_count = 1_usize
                .checked_shl(
                    u32::try_from(quotient_dimension)
                        .map_err(|_| StructureError::ArithmeticOverflow)?,
                )
                .ok_or(StructureError::ArithmeticOverflow)?;

            // Central square class per local weak class, with surjectivity.
            let local_count = weak.class_count();
            let mut central_square_classes = try_capacity(local_count)?;
            for local in weak.classes() {
                let representative =
                    weak.class_representative(local)
                        .ok_or(StructureError::IndexOutOfRange {
                            index: local.0,
                            upper_bound: local_count,
                        })?;
                let coordinates = adjoint.coordinates(representative)?.clone();
                let quotient_coordinates = quotient.to_coordinates(coordinates)?;
                let mut value = 0_usize;
                for bit in 0..quotient_dimension {
                    if quotient_coordinates.bit(bit) == Some(true) {
                        value |= 1_usize << bit;
                    }
                }
                central_square_classes.push(SquareClassId(value));
            }
            let mut base_local = try_capacity(square_class_count)?;
            base_local.resize(square_class_count, None::<usize>);
            for (local_index, square) in central_square_classes.iter().enumerate() {
                if base_local[square.0].is_none() {
                    base_local[square.0] = Some(local_index);
                }
            }
            let mut class_bases = try_capacity(square_class_count)?;
            for slot in &base_local {
                let local_index = slot.ok_or(StructureError::StrongRealInvariantViolation {
                    invariant: "square class count",
                })?;
                class_bases.push(
                    weak.class_representative(WeakRealFormId(local_index))
                        .ok_or(StructureError::IndexOutOfRange {
                            index: local_index,
                            upper_bound: local_count,
                        })?
                        .clone(),
                );
            }

            // The square-class-independent fiber-side shift columns and the
            // ambient translation masks. Columns are measured against the
            // ambient all-noncompact base grading (the grading of the zero
            // adjoint element), never a per-class base.
            let imaginary_rank = grading.imaginary_rank();
            let base_grading = grading.base_grading();
            let mut alpha_columns = try_capacity(imaginary_rank)?;
            alpha_columns.resize(imaginary_rank, 0_u64);
            for (basis_index, image) in image_elements.iter().enumerate() {
                let image_grading = grading.grading(image)?;
                for (imaginary_index, column) in alpha_columns.iter_mut().enumerate() {
                    if image_grading.is_noncompact(imaginary_index)
                        != base_grading.is_noncompact(imaginary_index)
                    {
                        *column |= 1_u64 << basis_index;
                    }
                }
            }
            let mut m_alpha_masks = try_capacity(imaginary_rank)?;
            for imaginary_index in 0..imaginary_rank {
                let element =
                    grading
                        .m_alpha(imaginary_index)
                        .ok_or(StructureError::IndexOutOfRange {
                            index: imaginary_index,
                            upper_bound: imaginary_rank,
                        })?;
                let coordinates = ambient.coordinates(element)?;
                let mut mask = 0_u64;
                for bit in 0..fiber_dimension {
                    if coordinates.bit(bit) == Some(true) {
                        mask |= 1_u64 << bit;
                    }
                }
                m_alpha_masks.push(mask);
            }

            // One walk per square class; tables stay transient but live long
            // enough to classify the strong-representative solutions.
            let mut orbit_sizes = try_capacity(square_class_count)?;
            let mut tables = try_capacity(square_class_count)?;
            for base_element in &class_bases {
                let class_grading = grading.grading(base_element)?;
                let mut base_bits = try_capacity(imaginary_rank)?;
                for imaginary_index in 0..imaginary_rank {
                    base_bits.push(class_grading.is_noncompact(imaginary_index).ok_or(
                        StructureError::IndexOutOfRange {
                            index: imaginary_index,
                            upper_bound: imaginary_rank,
                        },
                    )?);
                }
                let orbits = walk_mask_orbits(
                    fiber_dimension,
                    max_fiber_elements,
                    &base_bits,
                    &alpha_columns,
                    &m_alpha_masks,
                    strong_limit_error,
                )?;
                let mut sizes = try_capacity(orbits.representative_masks.len())?;
                sizes.resize(orbits.representative_masks.len(), 0_usize);
                for &class in &orbits.class_of_by_mask {
                    let class =
                        usize::try_from(class).map_err(|_| StructureError::ArithmeticOverflow)?;
                    sizes[class] = sizes[class]
                        .checked_add(1)
                        .ok_or(StructureError::ArithmeticOverflow)?;
                }
                orbit_sizes.push(sizes);
                tables.push(orbits.class_of_by_mask);
            }

            // Strong representatives: solve toAdjoint(x) = rep XOR base per
            // local class, then classify the solution mask.
            let augmented = adjoint_dimension
                .checked_add(fiber_dimension)
                .ok_or(StructureError::ArithmeticOverflow)?;
            let mut span = ModTwoSubspace::new(augmented)?;
            for (basis_index, coordinates) in image_coordinates.iter().enumerate() {
                let mut ones = try_capacity(augmented)?;
                for bit in 0..adjoint_dimension {
                    if coordinates.bit(bit) == Some(true) {
                        ones.push(bit);
                    }
                }
                ones.push(adjoint_dimension + basis_index);
                span.insert(ModTwoVector::from_ones(augmented, ones)?)?;
            }
            let mut strong_representatives = try_capacity(local_count)?;
            for (local_index, square) in central_square_classes.iter().enumerate() {
                let representative = weak
                    .class_representative(WeakRealFormId(local_index))
                    .ok_or(StructureError::IndexOutOfRange {
                        index: local_index,
                        upper_bound: local_count,
                    })?;
                let difference = adjoint.add(representative, &class_bases[square.0])?;
                let coordinates = adjoint.coordinates(&difference)?;
                let mut ones = try_capacity(augmented)?;
                for bit in 0..adjoint_dimension {
                    if coordinates.bit(bit) == Some(true) {
                        ones.push(bit);
                    }
                }
                let remainder =
                    span.quotient_representative(ModTwoVector::from_ones(augmented, ones)?)?;
                if (0..adjoint_dimension).any(|bit| remainder.bit(bit) == Some(true)) {
                    return Err(StructureError::StrongRealInvariantViolation {
                        invariant: "strong representative",
                    });
                }
                let mut mask = 0_u64;
                for basis_index in 0..fiber_dimension {
                    if remainder.bit(adjoint_dimension + basis_index) == Some(true) {
                        mask |= 1_u64 << basis_index;
                    }
                }
                let mask_index =
                    usize::try_from(mask).map_err(|_| StructureError::ArithmeticOverflow)?;
                let fiber_orbit = usize::try_from(tables[square.0][mask_index])
                    .map_err(|_| StructureError::ArithmeticOverflow)?;
                strong_representatives.push(StrongRealFormRep {
                    fiber_orbit,
                    square_class: *square,
                });
            }

            // Global term: orbitSize * squareClassCount * 2^fiberRank.
            let fiber_size_total = 1_usize
                .checked_shl(
                    u32::try_from(fiber_dimension)
                        .map_err(|_| StructureError::ArithmeticOverflow)?,
                )
                .ok_or(StructureError::ArithmeticOverflow)?;
            let term = cartan_class
                .twisted_involution_count()
                .checked_mul(square_class_count)
                .and_then(|value| value.checked_mul(fiber_size_total))
                .ok_or(StructureError::ArithmeticOverflow)?;
            global_kgb_size = global_kgb_size
                .checked_add(term)
                .ok_or(StructureError::ArithmeticOverflow)?;

            per_cartan.push(StrongRealData {
                central_square_classes,
                class_bases,
                orbit_sizes,
                strong_representatives,
            });
        }

        // The global-form-to-local-class table, then the precomputed sizes.
        let form_count = classification.weak_real_form_count();
        let mut local_of_form = try_capacity(classification.cartan_classes().len())?;
        for cartan_class in classification.cartan_classes() {
            let mut row = try_capacity(form_count)?;
            for form_index in 0..form_count {
                row.push(
                    cartan_class
                        .labels()
                        .labels()
                        .iter()
                        .position(|&label| label == WeakRealFormId(form_index)),
                );
            }
            local_of_form.push(row);
        }
        let mut kgb_sizes = try_capacity(form_count)?;
        for form_index in 0..form_count {
            let form = WeakRealFormId(form_index);
            let cartan_set =
                classification
                    .cartan_set(form)
                    .ok_or(StructureError::IndexOutOfRange {
                        index: form_index,
                        upper_bound: form_count,
                    })?;
            let mut total = 0_usize;
            for &cartan in cartan_set {
                let cartan_class =
                    classification
                        .cartan_class(cartan)
                        .ok_or(StructureError::IndexOutOfRange {
                            index: cartan.0,
                            upper_bound: classification.cartan_classes().len(),
                        })?;
                let local_index = local_of_form[cartan.0][form_index].ok_or(
                    StructureError::StrongRealInvariantViolation {
                        invariant: "square class consistency",
                    },
                )?;
                let fiber_size = per_cartan[cartan.0]
                    .fiber_size(WeakRealFormId(local_index))
                    .ok_or(StructureError::StrongRealInvariantViolation {
                        invariant: "square class consistency",
                    })?;
                let term = cartan_class
                    .twisted_involution_count()
                    .checked_mul(fiber_size)
                    .ok_or(StructureError::ArithmeticOverflow)?;
                total = total
                    .checked_add(term)
                    .ok_or(StructureError::ArithmeticOverflow)?;
            }
            kgb_sizes.push(total);
        }

        Ok(Self {
            per_cartan,
            local_of_form,
            kgb_sizes,
            global_kgb_size,
        })
    }

    /// Bounded by the Cartan class count.
    pub fn strong_real_data(&self, cartan: CartanId) -> Option<&StrongRealData> {
        self.per_cartan.get(cartan.0)
    }

    /// The fiber size of a GLOBAL form at one Cartan. `None` only for an
    /// out-of-range id; a valid pair where the form does not live at that
    /// Cartan is `Some(0)`, so sums over all Cartans stay correct.
    pub fn fiber_size(&self, form: WeakRealFormId, cartan: CartanId) -> Option<usize> {
        let data = self.per_cartan.get(cartan.0)?;
        match *self.local_of_form.get(cartan.0)?.get(form.0)? {
            Some(local) => data.fiber_size(WeakRealFormId(local)),
            None => Some(0),
        }
    }

    /// The precomputed KGB size of one real form. Bounded by the form count.
    pub fn kgb_size(&self, form: WeakRealFormId) -> Option<usize> {
        self.kgb_sizes.get(form.0).copied()
    }

    /// The total over all strong real forms.
    pub fn global_kgb_size(&self) -> usize {
        self.global_kgb_size
    }
}

fn strong_limit_error(resource: &'static str, limit: usize) -> StructureError {
    StructureError::StrongRealResourceLimit { resource, limit }
}

#[cfg(test)]
mod tests {
    use crate::adjoint_fiber::AdjointFiberBudget;
    use crate::integer_lattice::IntegerLatticeBudget;
    use crate::{
        BasedRootDatum, CartanClassificationBudget, Coweight, InnerClass, LatticeInvolution, Weight,
    };

    use super::*;

    fn classification(
        datum: &BasedRootDatum,
        distinguished: LatticeInvolution,
        weyl: usize,
    ) -> CartanClassification {
        let inner_class = InnerClass::new(datum.clone(), distinguished, 2 * weyl.max(4)).unwrap();
        let budget = CartanClassificationBudget::new(
            IntegerLatticeBudget::new(64, 100_000, 100_000, 128),
            AdjointFiberBudget::new(
                IntegerLatticeBudget::new(64, 100_000, 100_000, 128),
                50_000,
                100_000,
            ),
            weyl,
            64,
            64,
        );
        CartanClassification::build(&inner_class, &budget).unwrap()
    }

    #[test]
    fn sl2_matches_the_published_kgb_sizes() {
        let datum = BasedRootDatum::from_simple_data(
            1,
            vec![vec![2]],
            vec![Weight::new(vec![2])],
            vec![Coweight::new(vec![1])],
        )
        .unwrap();
        let classification =
            classification(&datum, LatticeInvolution::identity(&datum).unwrap(), 2);
        let strong = StrongRealClassification::build(&classification, 64).unwrap();

        let quasisplit = WeakRealFormId(0);
        let compact = WeakRealFormId(1);
        assert_eq!(strong.kgb_size(quasisplit), Some(3));
        assert_eq!(strong.kgb_size(compact), Some(1));
        assert_eq!(strong.global_kgb_size(), 5);
        let fundamental = strong.strong_real_data(CartanId(0)).unwrap();
        assert_eq!(fundamental.square_class_count(), 2);
        assert_eq!(strong.fiber_size(quasisplit, CartanId(0)), Some(2));
        assert_eq!(strong.fiber_size(quasisplit, CartanId(1)), Some(1));
        assert_eq!(strong.fiber_size(compact, CartanId(1)), Some(0));
    }

    #[test]
    fn a2_matches_the_published_kgb_sizes() {
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let classification =
            classification(&datum, LatticeInvolution::identity(&datum).unwrap(), 6);
        let strong = StrongRealClassification::build(&classification, 64).unwrap();

        assert_eq!(strong.kgb_size(WeakRealFormId(0)), Some(6));
        assert_eq!(strong.kgb_size(WeakRealFormId(1)), Some(1));
        assert_eq!(strong.global_kgb_size(), 7);
        let fundamental = strong.strong_real_data(CartanId(0)).unwrap();
        assert_eq!(fundamental.square_class_count(), 1);
    }

    #[test]
    fn b2_matches_the_published_kgb_sizes() {
        // Simply connected Spin(5) = Sp(4): weight-lattice basis, roots are
        // Cartan rows, coroots the dual standard basis. The adjoint-isogeny
        // standard datum would give different (smaller) KGB sizes.
        let datum = BasedRootDatum::from_simple_data(
            2,
            vec![vec![2, -2], vec![-1, 2]],
            vec![Weight::new(vec![2, -2]), Weight::new(vec![-1, 2])],
            vec![Coweight::new(vec![1, 0]), Coweight::new(vec![0, 1])],
        )
        .unwrap();
        let classification =
            classification(&datum, LatticeInvolution::identity(&datum).unwrap(), 8);
        let strong = StrongRealClassification::build(&classification, 64).unwrap();

        assert_eq!(strong.kgb_size(WeakRealFormId(0)), Some(11));
        assert_eq!(strong.kgb_size(WeakRealFormId(1)), Some(4));
        assert_eq!(strong.kgb_size(WeakRealFormId(2)), Some(1));
        assert_eq!(strong.global_kgb_size(), 17);

        // The so(5) square class has three orbits: two strong forms over the
        // one weak compact form.
        let fundamental = strong.strong_real_data(CartanId(0)).unwrap();
        assert_eq!(fundamental.square_class_count(), 2);
        let compact_local = WeakRealFormId(2);
        let square = fundamental.central_square_class(compact_local).unwrap();
        assert_eq!(fundamental.fiber_orbit_count(square), Some(3));
        assert_eq!(fundamental.fiber_size(compact_local), Some(1));
    }

    #[test]
    fn a1_x_a1_is_multiplicative() {
        // Simply connected SL(2) x SL(2) in the weight-lattice basis.
        let datum = BasedRootDatum::from_simple_data(
            2,
            vec![vec![2, 0], vec![0, 2]],
            vec![Weight::new(vec![2, 0]), Weight::new(vec![0, 2])],
            vec![Coweight::new(vec![1, 0]), Coweight::new(vec![0, 1])],
        )
        .unwrap();
        let classification =
            classification(&datum, LatticeInvolution::identity(&datum).unwrap(), 4);
        let strong = StrongRealClassification::build(&classification, 64).unwrap();

        assert_eq!(strong.kgb_size(WeakRealFormId(0)), Some(9));
        assert_eq!(strong.global_kgb_size(), 25);
    }

    #[test]
    fn undersized_budgets_reject_with_the_strong_named_error() {
        let datum = BasedRootDatum::from_simple_data(
            1,
            vec![vec![2]],
            vec![Weight::new(vec![2])],
            vec![Coweight::new(vec![1])],
        )
        .unwrap();
        let classification =
            classification(&datum, LatticeInvolution::identity(&datum).unwrap(), 2);
        assert!(matches!(
            StrongRealClassification::build(&classification, 1),
            Err(StructureError::StrongRealResourceLimit {
                resource: "fiber elements",
                limit: 1,
            })
        ));
    }
}
