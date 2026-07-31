//! The KGB seed `x0` (stage d) — mathematical substrate.
//!
//! This module lands in two steps per `SEED_X0_DESIGN.md`: the exact
//! rational machinery first (`stable_log`, the fundamental coweights), the
//! `RealFormSeed` builder next. The elections here are OBSERVABLE-BEARING:
//! `stable_log`'s adapted-basis representative fixes `g_rho_check`, hence
//! every downstream `torus_factor` rational.

use malachite::base::num::arithmetic::traits::Floor;
use malachite::base::num::basic::traits::{One, Zero};
use malachite::Rational;

use crate::grading::try_capacity;
use crate::integer_lattice::{adapted_basis, IntegerLatticeBudget};
use crate::weak_real_form::walk_mask_orbits;
use crate::{
    pair, BasedRootDatum, CartanClassification, CartanId, InnerClass, InvolutionTable,
    ModTwoVector, RationalCoweight, StrongRealClassification, StructureError, TitsElement,
    WeakRealFormId, WeylAction, WeylElement,
};
use malachite::base::num::arithmetic::traits::DivisibleBy;
use malachite::Integer;

/// Elected `xi^T`-stable logarithm (upstream `stable_log`,
/// y_values.cpp:155-166): reduce the input per-coordinate mod 1
/// (nonnegative), take the first-`d` adapted-basis coordinates of `xi + 1`,
/// reduce THOSE mod 1 (i.e. modulo the full fixed lattice), convert back.
/// The output lies EXACTLY in the +1-eigenspace.
///
/// PRECONDITION, checked: the trailing adapted-basis coordinates of the
/// reduced input are integral — equivalently the input is congruent mod
/// `X_*` to an exactly-fixed vector. `some_coch` inputs satisfy this
/// structurally; general squares need not.
pub(crate) fn stable_log(
    coordinates: &[Rational],
    coweight_action_plus_one: &[Vec<i32>],
    budget: &IntegerLatticeBudget,
) -> Result<Vec<Rational>, StructureError> {
    let dimension = coordinates.len();
    if coweight_action_plus_one.len() != dimension {
        return Err(StructureError::RankMismatch {
            expected: dimension,
            actual: coweight_action_plus_one.len(),
        });
    }
    let adapted = adapted_basis(coweight_action_plus_one, budget)?;
    let fixed_rank = adapted.diagonal.len();

    let mut reduced = try_capacity(dimension)?;
    for coordinate in coordinates {
        reduced.push(fractional_part(coordinate));
    }

    // All adapted-basis coordinates of the reduced vector; the trailing ones
    // carry the integrality precondition.
    let mut stable = try_capacity(fixed_rank)?;
    for row in 0..dimension {
        let mut value = Rational::ZERO;
        for (column, coordinate) in reduced.iter().enumerate() {
            value += Rational::from(adapted.inverse.entry(row, column).clone()) * coordinate;
        }
        if row < fixed_rank {
            stable.push(fractional_part(&value));
        } else if fractional_part(&value) != Rational::ZERO {
            return Err(StructureError::SeedInvariantViolation {
                invariant: "stable-log integrality",
            });
        }
    }

    let mut result = try_capacity(dimension)?;
    for row in 0..dimension {
        let mut value = Rational::ZERO;
        for (column, coordinate) in stable.iter().enumerate() {
            value += Rational::from(adapted.basis.entry(row, column).clone()) * coordinate;
        }
        result.push(value);
    }
    Ok(result)
}

/// The fundamental coweights `varpi_i^vee = sum_j (C^{-1})_{ji}
/// alpha_j^vee` in full lattice-rank coordinates — the unique rational
/// solutions of `<alpha_s, varpi_i^vee> = delta_si` with ZERO radical
/// component (upstream rootdata.cpp:850-853, 1015-1016). Rank-bounded
/// rational elimination carries no budget knob, per the crate's recorded
/// discipline.
pub(crate) fn fundamental_coweights(
    datum: &BasedRootDatum,
) -> Result<Vec<Vec<Rational>>, StructureError> {
    let semisimple_rank = datum.semisimple_rank();
    let lattice_rank = datum.lattice_rank();
    let mut cartan = try_capacity(semisimple_rank)?;
    for root in datum.simple_roots() {
        let mut row = try_capacity(semisimple_rank)?;
        for coroot in datum.simple_coroots() {
            row.push(Rational::from(pair(root, coroot)?));
        }
        cartan.push(row);
    }
    let inverse_columns = invert_rational(&cartan)?;

    let mut coweights = try_capacity(semisimple_rank)?;
    for column in inverse_columns {
        let mut coordinates = try_capacity(lattice_rank)?;
        coordinates.resize(lattice_rank, Rational::ZERO);
        for (index, coefficient) in column.iter().enumerate() {
            for (slot, &entry) in coordinates
                .iter_mut()
                .zip(datum.simple_coroots()[index].as_slice())
            {
                *slot += coefficient.clone() * Rational::from(entry);
            }
        }
        coweights.push(coordinates);
    }
    Ok(coweights)
}

/// One weak real form's KGB seed: the elected base-grading offset, the exact
/// square-class cocharacter, and the reduced seed element at the fundamental
/// involution — in ONE SHARED table's numbering (the caller keeps a single
/// append-only table per inner class).
///
/// Private fields: public construction could assemble a mismatched
/// offset/cocharacter/element triple. Construction invariant:
/// `grading_offset == grading_of_simples(cocharacter)`. Seed verification is
/// upstream's own x0-compacts assert (innerclass.cpp:1090-1092), which at
/// the identity Weyl part subsumes the element-square check.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RealFormSeed {
    form: WeakRealFormId,
    grading_offset: Vec<bool>,
    cocharacter: RationalCoweight,
    element: TitsElement,
}

impl RealFormSeed {
    /// Build the seed for one weak real form, per `SEED_X0_DESIGN.md`.
    #[allow(clippy::too_many_arguments)]
    pub fn build(
        inner_class: &InnerClass,
        classification: &CartanClassification,
        strong: &StrongRealClassification,
        table: &InvolutionTable,
        form: WeakRealFormId,
        integer_budget: &IntegerLatticeBudget,
        max_fiber_elements: usize,
    ) -> Result<Self, StructureError> {
        // Gate (i): the table's inner class (TitsCoset idiom).
        if table.inner_class() != inner_class {
            return Err(StructureError::DatumMismatch);
        }
        // Gate (ii): the classification's fundamental class is normalized to
        // the identity twisted involution over this datum and delta.
        let datum = inner_class.datum();
        let fundamental = classification.cartan_class(CartanId(0)).ok_or(
            StructureError::SeedInvariantViolation {
                invariant: "fundamental class",
            },
        )?;
        let representative = fundamental.representative();
        if representative.weyl_action().datum() != datum
            || representative.weyl_action() != &WeylAction::identity(datum)?
        {
            return Err(StructureError::SeedInvariantViolation {
                invariant: "fundamental class",
            });
        }
        let delta = inner_class.distinguished_involution().involution();
        let stored = representative.root_involution().involution();
        if stored.weight_matrix() != delta.weight_matrix()
            || stored.coweight_matrix() != delta.coweight_matrix()
        {
            return Err(StructureError::SeedInvariantViolation {
                invariant: "fundamental class",
            });
        }
        // Gates (iii)/(iv): the recorded count-consistency downgrade for the
        // strong layer, and the form id ranged through it. Global form ids
        // ARE the fundamental Cartan's local ids.
        let strong_data =
            strong
                .strong_real_data(CartanId(0))
                .ok_or(StructureError::SeedInvariantViolation {
                    invariant: "strong layer pairing",
                })?;
        let square =
            strong_data
                .central_square_class(form)
                .ok_or(StructureError::IndexOutOfRange {
                    index: form.0,
                    upper_bound: fundamental.partition().class_count(),
                })?;
        if strong.kgb_size(form).is_none() {
            return Err(StructureError::SeedInvariantViolation {
                invariant: "strong layer pairing",
            });
        }

        let grading = fundamental.grading();
        let weak = fundamental.partition();
        let adjoint = grading.adjoint_fiber();
        let ambient = adjoint.ambient_fiber();
        let fiber_map = adjoint.fiber_map();
        let system = inner_class.root_system();
        let simple_ids = system.simple_root_ids();
        let semisimple_rank = datum.semisimple_rank();
        let lattice_rank = datum.lattice_rank();
        let delta_data = inner_class.distinguished_involution();

        // The square class base: the LOWEST local form with this coset
        // coordinate (upstream classRep election).
        let mut base_local = None;
        for local in weak.classes() {
            if strong_data.central_square_class(local) == Some(square) {
                base_local = Some(local);
                break;
            }
        }
        let base_local = base_local.ok_or(StructureError::SeedInvariantViolation {
            invariant: "square class base",
        })?;
        let base_element = weak.class_representative(base_local).ok_or(
            StructureError::SeedInvariantViolation {
                invariant: "square class base",
            },
        )?;

        // Delta-fixed simple generators (upstream simple_roots_imaginary),
        // with their positions in the imaginary-subsystem basis.
        let mut fixed_simples = try_capacity(semisimple_rank)?;
        for (generator, &simple_id) in simple_ids.iter().enumerate() {
            if delta_data.image(simple_id) == Some(simple_id) {
                let position = grading
                    .imaginary_simple_roots()
                    .iter()
                    .position(|&candidate| candidate == simple_id)
                    .ok_or(StructureError::SeedInvariantViolation {
                        invariant: "fixed simple root",
                    })?;
                fixed_simples.push((generator, position));
            }
        }

        // some_coch: sum the fundamental coweights over the elected
        // representative's COMPACT delta-fixed simples, then stable_log.
        let base_grading_value = grading.grading(base_element)?;
        let coweights = fundamental_coweights(datum)?;
        let mut coch_rep = try_capacity(lattice_rank)?;
        coch_rep.resize(lattice_rank, Rational::ZERO);
        for &(generator, position) in &fixed_simples {
            let noncompact = base_grading_value.is_noncompact(position).ok_or(
                StructureError::SeedInvariantViolation {
                    invariant: "square class base",
                },
            )?;
            if !noncompact {
                for (slot, value) in coch_rep.iter_mut().zip(&coweights[generator]) {
                    *slot += value;
                }
            }
        }
        let mut xi_plus_one = try_capacity(lattice_rank)?;
        for (index, row) in delta.coweight_matrix().iter().enumerate() {
            let mut entries = try_capacity(lattice_rank)?;
            for (column, &value) in row.iter().enumerate() {
                entries.push(if index == column { value + 1 } else { value });
            }
            xi_plus_one.push(entries);
        }
        let cocharacter = stable_log(&coch_rep, &xi_plus_one, integer_budget)?;

        // Parities: the offset over ALL simples (even = set), the base
        // compacts over the delta-fixed ones (odd = compact), both under the
        // integral-pairing gate.
        let mut grading_offset = try_capacity(semisimple_rank)?;
        for root in datum.simple_roots() {
            let mut pairing = Rational::ZERO;
            for (value, &coordinate) in cocharacter.iter().zip(root.as_slice()) {
                pairing += value.clone() * Rational::from(coordinate);
            }
            if fractional_part(&pairing) != Rational::ZERO {
                return Err(StructureError::SeedInvariantViolation {
                    invariant: "integral simple pairing",
                });
            }
            let integral = Rational::from(pairing.clone().floor());
            debug_assert_eq!(integral, pairing);
            grading_offset.push(pairing.floor().divisible_by(&Integer::from(2)));
        }

        // rf_cpt and the grading-shift difference over the fixed simples.
        let form_element =
            weak.class_representative(form)
                .ok_or(StructureError::IndexOutOfRange {
                    index: form.0,
                    upper_bound: weak.class_count(),
                })?;
        let form_grading = grading.grading(form_element)?;
        let mut difference_bits = try_capacity(fixed_simples.len())?;
        for &(generator, position) in &fixed_simples {
            let base_compact = !grading_offset[generator];
            let form_compact = !form_grading.is_noncompact(position).ok_or(
                StructureError::SeedInvariantViolation {
                    invariant: "fixed simple root",
                },
            )?;
            difference_bits.push(base_compact != form_compact);
        }

        // Grading-shift solve: fiber-basis columns paired mod 2 against the
        // fixed simple roots; augmented elimination elects the same
        // particular solution as upstream's section().
        let fiber_dimension = ambient.dimension();
        let mut shift_columns = try_capacity(fiber_dimension)?;
        for basis in ambient.basis_representatives() {
            let mut column = try_capacity(fixed_simples.len())?;
            for &(generator, _) in &fixed_simples {
                column.push(parity_dot(
                    basis,
                    datum.simple_roots()[generator].as_slice(),
                )?);
            }
            shift_columns.push(bool_vector(&column)?);
        }
        let target = bool_vector(&difference_bits)?;
        let shift_mask = solve_mod_two(&shift_columns, &target)?.ok_or(
            StructureError::SeedInvariantViolation {
                invariant: "grading-shift exactness",
            },
        )?;
        let mut bits = ModTwoVector::zero(lattice_rank)?;
        for (index, basis) in ambient.basis_representatives().iter().enumerate() {
            if shift_mask & (1_u64 << index) != 0 {
                bits.xor_assign(basis)?;
            }
        }

        // The central fiber: re-walk the elected square class, solve
        // toAdjoint(y) = form_rep - class_base, and minimize the SHIFTED
        // value bits + c over the same-adjoint-image orbit members.
        let adjoint_dimension = adjoint.dimension();
        let mut image_elements = try_capacity(fiber_dimension)?;
        let mut image_coordinates: Vec<ModTwoVector> = try_capacity(fiber_dimension)?;
        for basis in ambient.basis_representatives() {
            let element = ambient.element_from_ambient(basis.clone())?;
            let image = fiber_map.apply(&element)?;
            image_coordinates.push(adjoint.coordinates(&image)?.clone());
            image_elements.push(image);
        }
        let imaginary_rank = grading.imaginary_rank();
        let ambient_base = grading.base_grading();
        let mut alpha_columns = try_capacity(imaginary_rank)?;
        alpha_columns.resize(imaginary_rank, 0_u64);
        for (basis_index, image) in image_elements.iter().enumerate() {
            let image_grading = grading.grading(image)?;
            for (imaginary_index, column) in alpha_columns.iter_mut().enumerate() {
                if image_grading.is_noncompact(imaginary_index)
                    != ambient_base.is_noncompact(imaginary_index)
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
            seed_limit_error,
        )?;

        // y: the strong-representative solve over the toAdjoint columns.
        let difference = adjoint.add(form_element, base_element)?;
        let difference_coordinates = adjoint.coordinates(&difference)?.clone();
        let mut adjoint_columns = try_capacity(fiber_dimension)?;
        for coordinates in &image_coordinates {
            adjoint_columns.push(coordinates.clone());
        }
        let y_mask = solve_mod_two(&adjoint_columns, &difference_coordinates)?.ok_or(
            StructureError::SeedInvariantViolation {
                invariant: "strong representative",
            },
        )?;
        let y_index = usize::try_from(y_mask).map_err(|_| StructureError::ArithmeticOverflow)?;
        let orbit = orbits.class_of_by_mask[y_index];

        let mut best: Option<ModTwoVector> = None;
        for (index, &class) in orbits.class_of_by_mask.iter().enumerate() {
            if class != orbit {
                continue;
            }
            let mask = u64::try_from(index).map_err(|_| StructureError::ArithmeticOverflow)?;
            let mut image = ModTwoVector::zero(adjoint_dimension)?;
            for (basis_index, coordinates) in image_coordinates.iter().enumerate() {
                if mask & (1_u64 << basis_index) != 0 {
                    image.xor_assign(coordinates)?;
                }
            }
            if image != difference_coordinates {
                continue;
            }
            let mut candidate = bits.clone();
            let shift = mask ^ y_mask;
            for (basis_index, basis) in ambient.basis_representatives().iter().enumerate() {
                if shift & (1_u64 << basis_index) != 0 {
                    candidate.xor_assign(basis)?;
                }
            }
            if best.as_ref().is_none_or(|current| candidate < *current) {
                best = Some(candidate);
            }
        }
        let final_bits = best.ok_or(StructureError::SeedInvariantViolation {
            invariant: "central fiber",
        })?;

        // Upstream's own seed verification: the compacts of t + bits equal
        // the form's compact set.
        for &(generator, position) in &fixed_simples {
            let base_compact = !grading_offset[generator];
            let flip = parity_dot(&final_bits, datum.simple_roots()[generator].as_slice())?;
            let form_compact = !form_grading.is_noncompact(position).ok_or(
                StructureError::SeedInvariantViolation {
                    invariant: "fixed simple root",
                },
            )?;
            if (base_compact != flip) != form_compact {
                return Err(StructureError::SeedInvariantViolation {
                    invariant: "x0 compacts",
                });
            }
        }

        // The seed element, located by lookup — never assumed InvolutionId(0)
        // — and reduced at the fundamental record's mod space.
        let identity = WeylElement::identity(system)?;
        let fundamental_involution =
            table
                .lookup(&identity)
                .ok_or(StructureError::SeedInvariantViolation {
                    invariant: "fundamental involution",
                })?;
        let record =
            table
                .record(fundamental_involution)
                .ok_or(StructureError::SeedInvariantViolation {
                    invariant: "fundamental involution",
                })?;
        let reduced = record.mod_space().quotient_representative(final_bits)?;
        let element = TitsElement::new(table, fundamental_involution, reduced)?;

        Ok(Self {
            form,
            grading_offset,
            cocharacter: RationalCoweight::from_coordinates(cocharacter),
            element,
        })
    }

    /// Build the seed from an EXPLICIT (cocharacter, torus part) pair — the
    /// custom branch of upstream `real_form_value::build`
    /// (atlas-types.w:3534-3545), whose `RealReductiveGroup` constructor
    /// stores the pair unchanged (realredgp.cpp:56-69). Gates: the table's
    /// inner class and the classification's fundamental class are
    /// normalized as in [`Self::build`], the cocharacter's simple pairings
    /// are integral (upstream asserts inside `RatCoweight::dot`), and the
    /// torus part reproduces the form's compact pattern — upstream's own
    /// x0-compacts assert (innerclass.cpp:1090-1092), the same
    /// verification [`Self::build`] performs on its elected output.
    pub fn custom(
        inner_class: &InnerClass,
        classification: &CartanClassification,
        table: &InvolutionTable,
        form: WeakRealFormId,
        cocharacter: &[Rational],
        torus_part: ModTwoVector,
    ) -> Result<Self, StructureError> {
        if table.inner_class() != inner_class {
            return Err(StructureError::DatumMismatch);
        }
        let datum = inner_class.datum();
        let lattice_rank = datum.lattice_rank();
        if cocharacter.len() != lattice_rank {
            return Err(StructureError::RankMismatch {
                expected: lattice_rank,
                actual: cocharacter.len(),
            });
        }
        if torus_part.dimension() != lattice_rank {
            return Err(StructureError::RankMismatch {
                expected: lattice_rank,
                actual: torus_part.dimension(),
            });
        }
        let fundamental = classification.cartan_class(CartanId(0)).ok_or(
            StructureError::SeedInvariantViolation {
                invariant: "fundamental class",
            },
        )?;
        let representative = fundamental.representative();
        if representative.weyl_action().datum() != datum
            || representative.weyl_action() != &WeylAction::identity(datum)?
        {
            return Err(StructureError::SeedInvariantViolation {
                invariant: "fundamental class",
            });
        }
        let delta = inner_class.distinguished_involution().involution();
        let stored = representative.root_involution().involution();
        if stored.weight_matrix() != delta.weight_matrix()
            || stored.coweight_matrix() != delta.coweight_matrix()
        {
            return Err(StructureError::SeedInvariantViolation {
                invariant: "fundamental class",
            });
        }

        // The offset over ALL simples (even = set), under the same
        // integral-pairing gate as the elected cocharacter's.
        let semisimple_rank = datum.semisimple_rank();
        let mut grading_offset = try_capacity(semisimple_rank)?;
        for root in datum.simple_roots() {
            let mut pairing = Rational::ZERO;
            for (value, &coordinate) in cocharacter.iter().zip(root.as_slice()) {
                pairing += value.clone() * Rational::from(coordinate);
            }
            if fractional_part(&pairing) != Rational::ZERO {
                return Err(StructureError::SeedInvariantViolation {
                    invariant: "integral simple pairing",
                });
            }
            let integral = Rational::from(pairing.clone().floor());
            debug_assert_eq!(integral, pairing);
            grading_offset.push(pairing.floor().divisible_by(&Integer::from(2)));
        }

        let system = inner_class.root_system();
        let simple_ids = system.simple_root_ids();
        let grading = fundamental.grading();
        let delta_data = inner_class.distinguished_involution();
        let mut fixed_simples = try_capacity(semisimple_rank)?;
        for (generator, &simple_id) in simple_ids.iter().enumerate() {
            if delta_data.image(simple_id) == Some(simple_id) {
                let position = grading
                    .imaginary_simple_roots()
                    .iter()
                    .position(|&candidate| candidate == simple_id)
                    .ok_or(StructureError::SeedInvariantViolation {
                        invariant: "fixed simple root",
                    })?;
                fixed_simples.push((generator, position));
            }
        }
        let weak = fundamental.partition();
        let form_element =
            weak.class_representative(form)
                .ok_or(StructureError::IndexOutOfRange {
                    index: form.0,
                    upper_bound: weak.class_count(),
                })?;
        let form_grading = grading.grading(form_element)?;

        // The seed element, located by lookup and reduced at the
        // fundamental record's mod space, exactly as in `build`.
        let identity = WeylElement::identity(system)?;
        let fundamental_involution =
            table
                .lookup(&identity)
                .ok_or(StructureError::SeedInvariantViolation {
                    invariant: "fundamental involution",
                })?;
        let record =
            table
                .record(fundamental_involution)
                .ok_or(StructureError::SeedInvariantViolation {
                    invariant: "fundamental involution",
                })?;
        let reduced = record.mod_space().quotient_representative(torus_part)?;

        // Upstream's own seed verification: the compacts of t + bits equal
        // the form's compact set (innerclass.cpp:1090-1092).
        for &(generator, position) in &fixed_simples {
            let base_compact = !grading_offset[generator];
            let flip = parity_dot(&reduced, datum.simple_roots()[generator].as_slice())?;
            let form_compact = !form_grading.is_noncompact(position).ok_or(
                StructureError::SeedInvariantViolation {
                    invariant: "fixed simple root",
                },
            )?;
            if (base_compact != flip) != form_compact {
                return Err(StructureError::SeedInvariantViolation {
                    invariant: "x0 compacts",
                });
            }
        }
        let element = TitsElement::new(table, fundamental_involution, reduced)?;

        Ok(Self {
            form,
            grading_offset,
            cocharacter: RationalCoweight::from_coordinates(cocharacter.to_vec()),
            element,
        })
    }

    pub fn form(&self) -> WeakRealFormId {
        self.form
    }

    pub fn grading_offset(&self) -> &[bool] {
        &self.grading_offset
    }

    pub fn square_class_cocharacter(&self) -> &RationalCoweight {
        &self.cocharacter
    }

    pub fn element(&self) -> &TitsElement {
        &self.element
    }
}

fn seed_limit_error(resource: &'static str, limit: usize) -> StructureError {
    StructureError::SeedResourceLimit { resource, limit }
}

/// Mod-2 pairing of a bit vector (over lattice coordinates) with an integer
/// weight.
fn parity_dot(vector: &ModTwoVector, weight: &[i32]) -> Result<bool, StructureError> {
    if vector.dimension() != weight.len() {
        return Err(StructureError::RankMismatch {
            expected: weight.len(),
            actual: vector.dimension(),
        });
    }
    let mut parity = false;
    for (index, &value) in weight.iter().enumerate() {
        if value % 2 != 0 && vector.bit(index) == Some(true) {
            parity = !parity;
        }
    }
    Ok(parity)
}

fn bool_vector(bits: &[bool]) -> Result<ModTwoVector, StructureError> {
    let mut ones = try_capacity(bits.len())?;
    for (index, &bit) in bits.iter().enumerate() {
        if bit {
            ones.push(index);
        }
    }
    ModTwoVector::from_ones(bits.len(), ones)
}

/// Solve `sum_j x_j * columns[j] = target` over `F_2` by the augmented
/// marker elimination, electing the same particular solution as upstream
/// `BinaryMap::section()` (greedy lowest-pivot in index order, solution
/// supported on pivot columns). Returns the solution as a column mask, or
/// `None` when the target is outside the span. This is the shared home the
/// design names; migrating the three older elimination copies onto it is
/// recorded follow-up.
pub(crate) fn solve_mod_two(
    columns: &[ModTwoVector],
    target: &ModTwoVector,
) -> Result<Option<u64>, StructureError> {
    let dimension = target.dimension();
    let augmented = dimension
        .checked_add(columns.len())
        .ok_or(StructureError::ArithmeticOverflow)?;
    let mut span = crate::ModTwoSubspace::new(augmented)?;
    for (index, column) in columns.iter().enumerate() {
        if column.dimension() != dimension {
            return Err(StructureError::RankMismatch {
                expected: dimension,
                actual: column.dimension(),
            });
        }
        let mut ones = try_capacity(augmented)?;
        for bit in 0..dimension {
            if column.bit(bit) == Some(true) {
                ones.push(bit);
            }
        }
        ones.push(dimension + index);
        span.insert(ModTwoVector::from_ones(augmented, ones)?)?;
    }
    let mut ones = try_capacity(augmented)?;
    for bit in 0..dimension {
        if target.bit(bit) == Some(true) {
            ones.push(bit);
        }
    }
    let remainder = span.quotient_representative(ModTwoVector::from_ones(augmented, ones)?)?;
    if (0..dimension).any(|bit| remainder.bit(bit) == Some(true)) {
        return Ok(None);
    }
    let mut mask = 0_u64;
    for (index, _) in columns.iter().enumerate() {
        if remainder.bit(dimension + index) == Some(true) {
            mask |= 1_u64 << index;
        }
    }
    Ok(Some(mask))
}

/// The nonnegative fractional part of a rational: `value - floor(value)`.
pub(crate) fn fractional_part(value: &Rational) -> Rational {
    value - Rational::from(value.floor())
}

/// Exact inverse of a square rational matrix by Gaussian elimination with
/// the first-nonzero pivot (deterministic; Cartan matrices of finite type
/// are invertible). Returns the inverse's COLUMNS.
pub(crate) fn invert_rational(
    matrix: &[Vec<Rational>],
) -> Result<Vec<Vec<Rational>>, StructureError> {
    let size = matrix.len();
    let mut work = try_capacity(size)?;
    for (index, row) in matrix.iter().enumerate() {
        if row.len() != size {
            return Err(StructureError::RankMismatch {
                expected: size,
                actual: row.len(),
            });
        }
        let mut augmented = try_capacity(2 * size)?;
        augmented.extend(row.iter().cloned());
        for column in 0..size {
            augmented.push(if column == index {
                Rational::ONE
            } else {
                Rational::ZERO
            });
        }
        work.push(augmented);
    }
    for pivot in 0..size {
        let source = (pivot..size)
            .find(|&row| work[row][pivot] != Rational::ZERO)
            .ok_or(StructureError::SeedInvariantViolation {
                invariant: "invertible Cartan matrix",
            })?;
        work.swap(pivot, source);
        let divisor = work[pivot][pivot].clone();
        for value in &mut work[pivot] {
            *value /= &divisor;
        }
        for row in 0..size {
            if row == pivot || work[row][pivot] == Rational::ZERO {
                continue;
            }
            let factor = work[row][pivot].clone();
            let pivot_row = work[pivot].clone();
            for (slot, pivot_value) in work[row].iter_mut().zip(&pivot_row) {
                let subtrahend = &factor * pivot_value;
                *slot -= subtrahend;
            }
        }
    }
    let mut columns = try_capacity(size)?;
    for column in 0..size {
        let mut values = try_capacity(size)?;
        for row in &work {
            values.push(row[size + column].clone());
        }
        columns.push(values);
    }
    Ok(columns)
}

#[cfg(test)]
mod tests {
    use crate::{Coweight, Weight};

    use super::*;

    fn budget() -> IntegerLatticeBudget {
        IntegerLatticeBudget::new(16, 256, 1_000, 128)
    }

    fn rational(numerator: i32, denominator: i32) -> Rational {
        Rational::from(numerator) / Rational::from(denominator)
    }

    use crate::{
        AdjointFiberBudget, CartanClassification, CartanClassificationBudget, CartanId, InnerClass,
        InvolutionTable, InvolutionTableBudget, LatticeInvolution, StrongRealClassification,
        WeakRealFormId,
    };

    fn class_budget(weyl: usize) -> CartanClassificationBudget {
        CartanClassificationBudget::new(
            IntegerLatticeBudget::new(64, 100_000, 100_000, 128),
            AdjointFiberBudget::new(
                IntegerLatticeBudget::new(64, 100_000, 100_000, 128),
                50_000,
                100_000,
            ),
            weyl,
            64,
            64,
        )
    }

    fn seed_context(
        datum: BasedRootDatum,
        distinguished: Option<Vec<Vec<i32>>>,
        roots: usize,
        weyl: usize,
    ) -> (
        InnerClass,
        CartanClassification,
        StrongRealClassification,
        InvolutionTable,
    ) {
        let distinguished = match distinguished {
            Some(matrix) => LatticeInvolution::new(&datum, matrix.clone(), matrix).unwrap(),
            None => LatticeInvolution::identity(&datum).unwrap(),
        };
        let inner_class = InnerClass::new(datum, distinguished, roots).unwrap();
        let classification =
            CartanClassification::build(&inner_class, &class_budget(weyl)).unwrap();
        let strong = StrongRealClassification::build(&classification, 4_096).unwrap();
        let mut table = InvolutionTable::new(
            &inner_class,
            InvolutionTableBudget::new(64, IntegerLatticeBudget::new(64, 100_000, 100_000, 128)),
        )
        .unwrap();
        for id in 0..classification.cartan_classes().len() {
            table.add_cartan(&classification, CartanId(id)).unwrap();
        }
        (inner_class, classification, strong, table)
    }

    fn build_seed(
        context: &(
            InnerClass,
            CartanClassification,
            StrongRealClassification,
            InvolutionTable,
        ),
        form: usize,
    ) -> RealFormSeed {
        RealFormSeed::build(
            &context.0,
            &context.1,
            &context.2,
            &context.3,
            WeakRealFormId(form),
            &budget(),
            4_096,
        )
        .unwrap()
    }

    #[test]
    fn sl2_seeds_split_and_compact_forms() {
        let datum = BasedRootDatum::from_simple_data(
            1,
            vec![vec![2]],
            vec![Weight::new(vec![2])],
            vec![Coweight::new(vec![1])],
        )
        .unwrap();
        let context = seed_context(datum, None, 2, 2);
        let split = build_seed(&context, 0);
        assert_eq!(split.form(), WeakRealFormId(0));
        assert_eq!(split.grading_offset(), &[true]);
        assert!(split.element().torus_bits().is_zero());
        assert_eq!(split.square_class_cocharacter().dimension(), 1);

        let compact = build_seed(&context, 1);
        assert_eq!(compact.grading_offset(), &[false]);
        assert!(compact.element().torus_bits().is_zero());

        // Determinism: two builds agree exactly.
        assert_eq!(split, build_seed(&context, 0));
        assert_ne!(split, compact);
    }

    #[test]
    fn b2_split_builds_all_three_forms_deterministically() {
        let datum = BasedRootDatum::standard(vec![vec![2, -2], vec![-1, 2]]).unwrap();
        let context = seed_context(datum, None, 8, 8);
        let quasisplit = build_seed(&context, 0);
        assert_eq!(quasisplit.grading_offset(), &[true, true]);
        let middle = build_seed(&context, 1);
        let compact = build_seed(&context, 2);
        assert_ne!(quasisplit, middle);
        assert_ne!(middle, compact);
        for form in 0..3 {
            assert_eq!(build_seed(&context, form), build_seed(&context, form));
        }
    }

    #[test]
    fn su21_class_has_an_empty_fixed_simple_system() {
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let context = seed_context(datum, Some(vec![vec![0, 1], vec![1, 0]]), 6, 6);
        let quasisplit = build_seed(&context, 0);
        // Twist-invariance of the offset holds by construction on the
        // swapped pair.
        assert_eq!(
            quasisplit.grading_offset()[0],
            quasisplit.grading_offset()[1]
        );
        assert_eq!(quasisplit, build_seed(&context, 0));
    }

    #[test]
    fn stable_log_elects_the_fixed_representative_for_the_swap() {
        // xi = swap on Z^2, xi + 1 = [[1,1],[1,1]].
        let matrix = vec![vec![1, 1], vec![1, 1]];
        let half = vec![rational(1, 2), rational(1, 2)];
        assert_eq!(stable_log(&half, &matrix, &budget()).unwrap(), half);
        // Integral shifts wash out through the mod-1 reductions.
        let shifted = vec![rational(3, 2), rational(1, 2)];
        assert_eq!(stable_log(&shifted, &matrix, &budget()).unwrap(), half);
        // A non-stable input violates the trailing-integrality precondition.
        let unstable = vec![rational(1, 2), Rational::ZERO];
        assert_eq!(
            stable_log(&unstable, &matrix, &budget()),
            Err(StructureError::SeedInvariantViolation {
                invariant: "stable-log integrality",
            })
        );
    }

    #[test]
    fn stable_log_of_the_negative_identity_accepts_only_integral_logs() {
        // xi = -1: the fixed lattice is trivial. Integral logs elect zero.
        let matrix = vec![vec![0, 0], vec![0, 0]];
        let integral = vec![Rational::from(1), Rational::from(-2)];
        assert_eq!(
            stable_log(&integral, &matrix, &budget()).unwrap(),
            vec![Rational::ZERO, Rational::ZERO]
        );
        // A 2-torsion log is STABLE but not congruent mod X_* to an
        // exactly-fixed vector: upstream would silently drop it (the
        // review-documented hazard); the port's precondition rejects.
        let torsion = vec![rational(1, 2), rational(1, 2)];
        assert_eq!(
            stable_log(&torsion, &matrix, &budget()),
            Err(StructureError::SeedInvariantViolation {
                invariant: "stable-log integrality",
            })
        );
    }

    #[test]
    fn fundamental_coweights_pair_as_the_identity() {
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let coweights = fundamental_coweights(&datum).unwrap();
        assert_eq!(coweights.len(), 2);
        for (index, root) in datum.simple_roots().iter().enumerate() {
            for (which, coweight) in coweights.iter().enumerate() {
                let pairing = root
                    .as_slice()
                    .iter()
                    .zip(coweight)
                    .fold(Rational::ZERO, |sum, (&a, b)| sum + Rational::from(a) * b);
                let expected = if index == which {
                    Rational::ONE
                } else {
                    Rational::ZERO
                };
                assert_eq!(pairing, expected);
            }
        }
    }

    #[test]
    fn fundamental_coweights_stay_in_the_coroot_span() {
        // Non-semisimple A1 in a rank-2 lattice: the radical component is
        // elected zero, so the coweight is half the simple coroot.
        let datum = BasedRootDatum::from_simple_data(
            2,
            vec![vec![2]],
            vec![Weight::new(vec![1, 0])],
            vec![Coweight::new(vec![2, 0])],
        )
        .unwrap();
        let coweights = fundamental_coweights(&datum).unwrap();
        assert_eq!(coweights, vec![vec![Rational::ONE, Rational::ZERO]]);
    }
}
