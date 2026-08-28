//! The synthetic real form's elected cocharacter and initial torus part.
//!
//! Two ports serve the custom-seed branch of upstream
//! `real_form_value::build` (interpreter/atlas-types.w:3534-3545):
//!
//! - [`elected_square_root`] is the `coch` output of `real_form_of`
//!   (innerclass.cpp:1318-1327): the `stable_log` of the strong datum's
//!   shifted square, computed from the projected torus factor. This is the
//!   `g_rho_check` the custom `RealReductiveGroup` constructor stores.
//! - [`minimal_torus_part`] is `realredgp::minimal_torus_part`
//!   (realredgp.cpp:212-309): descend the given strong representative to
//!   the fundamental fiber by inverse Cayley transforms and based twisted
//!   conjugations, then walk the fundamental imaginary grading orbit and
//!   elect the numerically minimal torus part whose datum-simple
//!   compactness pattern reproduces the target weak real form.
//!
//! The descent reuses the stage-(c) [`TitsCoset`] operations, which reduce
//! each intermediate torus part at its target involution. Upstream reduces
//! only at the end; the operations are class maps on the mod-two quotients
//! (the property the stage-(e) KGB enumeration is verified against), so the
//! per-step reductions cannot move the final reduced class.

use std::collections::BTreeSet;

use malachite::base::num::arithmetic::traits::DivisibleBy;
use malachite::base::num::basic::traits::{One, Zero};
use malachite::{Integer, Rational};

use crate::grading::try_capacity;
use crate::real_form_seed::stable_log;
use crate::{
    CartanClassification, CartanId, InnerClass, IntegerLatticeBudget, InvolutionTable,
    ModTwoVector, RootKind, StructureError, TitsCoset, TitsElement, WeakRealFormId, WeylAction,
    WeylElement, WeylInterface,
};

/// The `u64` encoding of the orbit walk's seen-set caps the lattice rank
/// (same discipline as the weak-form mask walk).
const MAX_MASK_BITS: usize = 63;

/// The elected square root of the strong datum's shifted square: upstream's
/// `coch = stable_log(square_shifted(a), xi.transposed())` inside
/// `real_form_of` (innerclass.cpp:1318-1327). `factor` is the caller's
/// PROJECTED theta-fixed torus factor (the wrapper's doubled,
/// centrality-checked, then halved value); `twisted` its Weyl part.
/// `square_shifted` is `t + w.delta_tr(t)` with the Weyl action taken on
/// the DUAL side (tits.cpp:127-134), so the transport below applies the
/// distinguished coweight matrix first and the element's coweight action
/// second. The output is exactly the input of [`stable_log`]'s port.
pub fn elected_square_root(
    inner_class: &InnerClass,
    twisted: &WeylElement,
    factor: &[Rational],
    budget: &IntegerLatticeBudget,
) -> Result<Vec<Rational>, StructureError> {
    let datum = inner_class.datum();
    let system = inner_class.root_system();
    let rank = datum.lattice_rank();
    if factor.len() != rank {
        return Err(StructureError::RankMismatch {
            expected: rank,
            actual: factor.len(),
        });
    }
    if twisted.image_permutation().len() != system.roots().len() {
        return Err(StructureError::DatumMismatch);
    }

    // Rebuild the matrix-level action from the element's reduced word to
    // reach its coweight matrix; the round-trip check pins the
    // word-composition direction (`w = s_word[0] * s_word[1] * ...`).
    let word = twisted.reduced_word(system)?;
    let mut action = WeylAction::identity(datum)?;
    for &generator in word.iter().rev() {
        action = WeylAction::simple_reflection(datum, generator)?.compose(&action)?;
    }
    if &WeylElement::from_action(system, &action)? != twisted {
        return Err(StructureError::SeedInvariantViolation {
            invariant: "Weyl action round trip",
        });
    }

    let delta = inner_class
        .distinguished_involution()
        .involution()
        .coweight_matrix();
    let after_delta = apply_integer_matrix(delta, factor)?;
    let after_w = apply_integer_matrix(action.coweight_matrix(), &after_delta)?;
    // stable_log sees the square through log_2pi, halving the exp_pi
    // representation; the port's own mod-1 reduction handles the rest.
    let mut half_square = try_capacity(rank)?;
    for (coordinate, image) in factor.iter().zip(&after_w) {
        half_square.push((coordinate + image) / Rational::from(2));
    }
    let mut plus_one = try_capacity(rank)?;
    for (row_index, row) in delta.iter().enumerate() {
        let mut entries = try_capacity(rank)?;
        for (column, &value) in row.iter().enumerate() {
            entries.push(if row_index == column {
                value + 1
            } else {
                value
            });
        }
        plus_one.push(entries);
    }
    stable_log(&half_square, &plus_one, budget)
}

/// Port of `realredgp::minimal_torus_part` (realredgp.cpp:212-309): the
/// initial torus part for the KGB construction of the weak real form
/// `form`, seeded from the strong datum `(twisted, factor)` and its
/// elected `coch` (both from the caller; `factor` is the projected,
/// theta-fixed torus factor). `table` must already cover the Cartan class
/// of `twisted` and every class below it (the synthetic wrapper generates
/// exactly those, interpreter/atlas-types.w:3902-3907).
///
/// One deliberate translation: upstream pairs the target grading's slice
/// bits with the LEADING datum-simple entries of the fundamental
/// simple-imaginary basis, relying on its root numbering to place the
/// datum simples first in generator order. The crate's imaginary basis
/// follows its own deterministic root order, so the pairing below is made
/// per position: every datum-simple basis entry constrains its position
/// with the form's compactness at that generator. The two pairings agree
/// whenever upstream's leading-run invariant holds (in particular for a
/// trivial distinguished involution, where every entry is datum-simple).
pub fn minimal_torus_part(
    inner_class: &InnerClass,
    classification: &CartanClassification,
    table: &InvolutionTable,
    form: WeakRealFormId,
    coch: &[Rational],
    twisted: &WeylElement,
    factor: &[Rational],
) -> Result<ModTwoVector, StructureError> {
    if table.inner_class() != inner_class {
        return Err(StructureError::DatumMismatch);
    }
    let datum = inner_class.datum();
    let system = inner_class.root_system();
    let rank = datum.lattice_rank();
    if coch.len() != rank || factor.len() != rank {
        return Err(StructureError::RankMismatch {
            expected: rank,
            actual: coch.len().max(factor.len()),
        });
    }
    if twisted.image_permutation().len() != system.roots().len() {
        return Err(StructureError::DatumMismatch);
    }
    if rank > MAX_MASK_BITS {
        return Err(StructureError::SeedResourceLimit {
            resource: "mask bits",
            limit: MAX_MASK_BITS,
        });
    }

    // The torus part of the strong datum relative to the elected
    // cocharacter: `diff = (torus_factor-coch).normalize()`, integral by
    // the shared coset, and `TorusPart tp(diff.numerator())`.
    let mut ones = try_capacity(rank)?;
    for (index, (left, right)) in factor.iter().zip(coch).enumerate() {
        let difference = left - right;
        let integer =
            Integer::try_from(&difference).map_err(|_| StructureError::SeedInvariantViolation {
                invariant: "torus-part integrality",
            })?;
        if !integer.divisible_by(&Integer::from(2)) {
            ones.push(index);
        }
    }
    let mut torus_part = ModTwoVector::from_ones(rank, ones)?;

    // Move to the fundamental fiber: left descents of the Weyl part, real
    // ones through inverse Cayley, complex ones through based twisted
    // conjugation (the upstream loop at realredgp.cpp:230-243).
    let offset = grading_of_simples(inner_class, coch)?;
    let coset = TitsCoset::new(inner_class, offset)?;
    let involution = table
        .lookup(twisted)
        .ok_or(StructureError::SeedInvariantViolation {
            invariant: "synthetic involution coverage",
        })?;
    let mut element = TitsElement::new(table, involution, torus_part.clone())?;
    let interface = WeylInterface::new(datum.cartan_matrix())?;
    loop {
        let current = element.involution();
        if table
            .weyl_is_identity(current)
            .ok_or(StructureError::SeedInvariantViolation {
                invariant: "synthetic involution coverage",
            })?
        {
            break;
        }
        // Upstream `WeylGroup::leftDescent` (weyl.cpp:919-928): the first
        // nonzero parabolic piece, in external numbering. The first
        // internal left descent is the same generator: the piece list's
        // first nonzero level is exactly the smallest internal level at
        // which the element has a left descent.
        let generator = table
            .weyl_first_left_descent(current, interface.outward())?
            .ok_or(StructureError::SeedInvariantViolation {
                invariant: "left descent",
            })?;
        if table.simple_root_kind(current, generator) == Some(RootKind::Real) {
            element = coset.inverse_cayley(table, generator, &element)?.ok_or(
                StructureError::SeedInvariantViolation {
                    invariant: "inverse Cayley coverage",
                },
            )?;
        } else {
            element = coset.cross_pregated(table, generator, &element)?;
        }
    }
    // Upstream `i_tab.reduce(a)`: idempotent after a stepped descent,
    // decisive when `twisted` was already fundamental.
    torus_part = coset.reduce(table, &element)?.torus_bits().clone();

    // The grading at the fundamental fiber under `cowt = coch + lift(tp)`:
    // set means EVEN pairing (compact), as upstream's `start_grading`.
    let mut coweight = try_capacity(rank)?;
    for (index, coordinate) in coch.iter().enumerate() {
        let mut value = coordinate.clone();
        if torus_part.bit(index) == Some(true) {
            value += Rational::ONE;
        }
        coweight.push(value);
    }

    let fundamental =
        classification
            .cartan_class(CartanId(0))
            .ok_or(StructureError::SeedInvariantViolation {
                invariant: "fundamental class",
            })?;
    let grading = fundamental.grading();
    let basis = grading.imaginary_simple_roots();
    let imaginary_rank = basis.len();

    let form_element = fundamental.partition().class_representative(form).ok_or(
        StructureError::IndexOutOfRange {
            index: form.0,
            upper_bound: fundamental.partition().class_count(),
        },
    )?;
    let form_grading = grading.grading(form_element)?;
    let simple_ids = system.simple_root_ids();

    let mut start = try_capacity(imaginary_rank)?;
    let mut constrained = try_capacity(imaginary_rank)?;
    let mut target = try_capacity(imaginary_rank)?;
    let mut m_alpha = try_capacity(imaginary_rank)?;
    for (index, &root_id) in basis.iter().enumerate() {
        let root = system
            .root(root_id)
            .ok_or(StructureError::IndexOutOfRange {
                index: root_id.0,
                upper_bound: system.roots().len(),
            })?;
        start.push(even_pairing(root.as_slice(), &coweight)?);
        // `target = G.simple_roots_x0_compact(wrf).slice(...)^mask`: the
        // flip makes the stored bit the form's NONcompactness, which is
        // exactly the crate's `Grading` convention.
        let datum_simple = simple_ids.contains(&root_id);
        constrained.push(datum_simple);
        target.push(
            datum_simple
                && form_grading.is_noncompact(index).ok_or(
                    StructureError::SeedInvariantViolation {
                        invariant: "fundamental imaginary simple",
                    },
                )?,
        );
        let coroot = system
            .coroot(root_id)
            .ok_or(StructureError::IndexOutOfRange {
                index: root_id.0,
                upper_bound: system.roots().len(),
            })?;
        m_alpha.push(parity_vector(coroot.as_slice())?);
    }
    let mut grading_shift = try_capacity(imaginary_rank)?;
    for coroot_index in 0..imaginary_rank {
        let coroot = system
            .coroot(basis[coroot_index])
            .ok_or(StructureError::IndexOutOfRange {
                index: basis[coroot_index].0,
                upper_bound: system.roots().len(),
            })?;
        let mut shift = try_capacity(imaginary_rank)?;
        for &root_id in basis {
            let root = system
                .root(root_id)
                .ok_or(StructureError::IndexOutOfRange {
                    index: root_id.0,
                    upper_bound: system.roots().len(),
                })?;
            shift.push(crate::pair(root, coroot)? % 2 != 0);
        }
        grading_shift.push(shift);
    }

    // The upstream do-while walk (realredgp.cpp:271-299): a stack of
    // (torus part, grading) states, translating by `m_alpha[i]` at every
    // SET grading bit, recording states whose constrained positions match
    // the target. The seen-set bounds the walk by `2^rank`.
    let mut seen = BTreeSet::new();
    seen.insert(encode(&torus_part)?);
    let mut candidates = try_capacity(1)?;
    let mut stack = vec![(torus_part, start)];
    while let Some((current, grading_bits)) = stack.pop() {
        if (0..imaginary_rank)
            .all(|index| !constrained[index] || grading_bits[index] == target[index])
        {
            candidates.push(current.clone());
        }
        for index in 0..imaginary_rank {
            if !grading_bits[index] {
                continue;
            }
            let mut image = current.clone();
            image.xor_assign(&m_alpha[index])?;
            if seen.insert(encode(&image)?) {
                let mut shifted = grading_bits.clone();
                for (bit, &flip) in shifted.iter_mut().zip(&grading_shift[index]) {
                    *bit = *bit != flip;
                }
                stack.push((image, shifted));
            }
        }
    }
    // Upstream asserts the candidate list is nonempty and elects the
    // minimal torus part (integer order of the bit vector, which is the
    // crate's `ModTwoVector` order at equal dimension).
    candidates
        .into_iter()
        .min()
        .ok_or(StructureError::SeedInvariantViolation {
            invariant: "minimal torus part candidates",
        })
}

/// `innerclass::grading_of_simples` (innerclass.cpp:1290-1303): per simple
/// root, whether the cocharacter's pairing is an EVEN integer. Upstream's
/// `RatCoweight::dot` asserts the integrality; the gate names it.
fn grading_of_simples(
    inner_class: &InnerClass,
    coch: &[Rational],
) -> Result<Vec<bool>, StructureError> {
    let datum = inner_class.datum();
    if coch.len() != datum.lattice_rank() {
        return Err(StructureError::RankMismatch {
            expected: datum.lattice_rank(),
            actual: coch.len(),
        });
    }
    let mut offset = try_capacity(datum.semisimple_rank())?;
    for root in datum.simple_roots() {
        offset.push(even_pairing(root.as_slice(), coch)?);
    }
    Ok(offset)
}

/// The parity of an assumed-integral root–coweight pairing: `true` when
/// even. Non-integral pairings are the upstream `RatCoweight::dot`
/// assertion, gated as a named invariant.
fn even_pairing(root: &[i32], coweight: &[Rational]) -> Result<bool, StructureError> {
    if root.len() != coweight.len() {
        return Err(StructureError::RankMismatch {
            expected: coweight.len(),
            actual: root.len(),
        });
    }
    let pairing = root
        .iter()
        .zip(coweight)
        .fold(Rational::ZERO, |sum, (&coordinate, value)| {
            sum + Rational::from(coordinate) * value
        });
    let integer =
        Integer::try_from(&pairing).map_err(|_| StructureError::SeedInvariantViolation {
            invariant: "integral simple pairing",
        })?;
    Ok(integer.divisible_by(&Integer::from(2)))
}

fn apply_integer_matrix(
    matrix: &[Vec<i32>],
    vector: &[Rational],
) -> Result<Vec<Rational>, StructureError> {
    if matrix.len() != vector.len() || matrix.iter().any(|row| row.len() != vector.len()) {
        return Err(StructureError::InvalidIntegerMatrixShape);
    }
    let mut result = try_capacity(vector.len())?;
    for row in matrix {
        let mut value = Rational::ZERO;
        for (&entry, coordinate) in row.iter().zip(vector) {
            value += Rational::from(entry) * coordinate;
        }
        result.push(value);
    }
    Ok(result)
}

/// The mod-2 reduction of an integer vector: bits at the odd coordinates.
fn parity_vector(coordinates: &[i32]) -> Result<ModTwoVector, StructureError> {
    let mut ones = try_capacity(coordinates.len())?;
    for (index, &value) in coordinates.iter().enumerate() {
        if value % 2 != 0 {
            ones.push(index);
        }
    }
    ModTwoVector::from_ones(coordinates.len(), ones)
}

/// The seen-set key: the bit vector as an integer (bit 0 the lowest).
/// Callers gate the dimension to [`MAX_MASK_BITS`].
fn encode(vector: &ModTwoVector) -> Result<u64, StructureError> {
    let mut key = 0_u64;
    for index in 0..vector.dimension() {
        if vector.bit(index) == Some(true) {
            key |= 1_u64 << index;
        }
    }
    Ok(key)
}

#[cfg(test)]
mod tests {
    use crate::{
        AdjointFiberBudget, BasedRootDatum, CartanClassificationBudget, Coweight,
        InvolutionTableBudget, LatticeInvolution, Weight,
    };

    use super::*;

    fn integer_budget() -> IntegerLatticeBudget {
        IntegerLatticeBudget::new(64, 100_000, 100_000, 128)
    }

    fn class_budget(weyl: usize) -> CartanClassificationBudget {
        CartanClassificationBudget::new(
            integer_budget(),
            AdjointFiberBudget::new(integer_budget(), 50_000, 100_000),
            weyl,
            64,
            64,
        )
    }

    fn rational(numerator: i32, denominator: i32) -> Rational {
        Rational::from(numerator) / Rational::from(denominator)
    }

    fn compact_inner(datum: BasedRootDatum, roots: usize) -> InnerClass {
        let distinguished = LatticeInvolution::identity(&datum).unwrap();
        InnerClass::new(datum, distinguished, roots).unwrap()
    }

    fn full_table(
        inner_class: &InnerClass,
        classification: &CartanClassification,
    ) -> InvolutionTable {
        let mut table = InvolutionTable::new(
            inner_class,
            InvolutionTableBudget::new(64, integer_budget()),
        )
        .unwrap();
        for id in classification.cartan_ids() {
            table.add_cartan(classification, id).unwrap();
        }
        table
    }

    fn a1_t1_datum() -> BasedRootDatum {
        BasedRootDatum::from_simple_data(
            2,
            vec![vec![2]],
            vec![Weight::new(vec![2, 0])],
            vec![Coweight::new(vec![1, 0])],
        )
        .unwrap()
    }

    fn sc_a2_datum() -> BasedRootDatum {
        // Simply-connected A2: roots are the Cartan rows, coroots the basis.
        BasedRootDatum::from_simple_data(
            2,
            vec![vec![2, -1], vec![-1, 2]],
            vec![Weight::new(vec![2, -1]), Weight::new(vec![-1, 2])],
            vec![Coweight::new(vec![1, 0]), Coweight::new(vec![0, 1])],
        )
        .unwrap()
    }

    #[test]
    fn a1_t1_central_factor_keeps_its_cocharacter_and_zero_torus_part() {
        // The frozen `weak_real_form_a1_t1_central_probe` anchor: the
        // involution negates the semisimple coordinate, the central
        // coordinate carries the whole torus factor.
        let inner_class = compact_inner(a1_t1_datum(), 2);
        let classification = CartanClassification::build(&inner_class, &class_budget(2)).unwrap();
        let table = full_table(&inner_class, &classification);
        let reflection = WeylElement::simple_reflection(inner_class.root_system(), 0).unwrap();
        let factor = vec![Rational::ZERO, rational(1, 2)];

        let coch =
            elected_square_root(&inner_class, &reflection, &factor, &integer_budget()).unwrap();
        assert_eq!(coch, factor);

        let torus_part = minimal_torus_part(
            &inner_class,
            &classification,
            &table,
            WeakRealFormId(0),
            &coch,
            &reflection,
            &factor,
        )
        .unwrap();
        assert!(torus_part.is_zero());
    }

    #[test]
    fn a2_noncanonical_seeds_match_the_frozen_probe() {
        // The frozen `weak_real_form_a2_noncanonical_probe` anchors: two
        // strong data of the quasisplit SU(2,1) with cocharacters that are
        // fundamental coweights, electing the two single-bit torus parts.
        let inner_class = compact_inner(sc_a2_datum(), 6);
        let classification = CartanClassification::build(&inner_class, &class_budget(6)).unwrap();
        let table = full_table(&inner_class, &classification);
        let first = WeylElement::simple_reflection(inner_class.root_system(), 0).unwrap();
        let second = WeylElement::simple_reflection(inner_class.root_system(), 1).unwrap();

        let first_factor = vec![rational(1, 3), rational(2, 3)];
        let first_coch =
            elected_square_root(&inner_class, &first, &first_factor, &integer_budget()).unwrap();
        assert_eq!(first_coch, first_factor);
        let first_part = minimal_torus_part(
            &inner_class,
            &classification,
            &table,
            WeakRealFormId(0),
            &first_coch,
            &first,
            &first_factor,
        )
        .unwrap();
        assert_eq!(first_part, ModTwoVector::from_ones(2, [0]).unwrap());

        let second_factor = vec![rational(2, 3), rational(1, 3)];
        let second_coch =
            elected_square_root(&inner_class, &second, &second_factor, &integer_budget()).unwrap();
        assert_eq!(second_coch, second_factor);
        let second_part = minimal_torus_part(
            &inner_class,
            &classification,
            &table,
            WeakRealFormId(0),
            &second_coch,
            &second,
            &second_factor,
        )
        .unwrap();
        assert_eq!(second_part, ModTwoVector::from_ones(2, [1]).unwrap());
    }

    #[test]
    fn a2_equivalent_factor_elects_the_same_seed() {
        // The probe's `rp`: [4,2]/3 projects to the same factor as [1,2]/3,
        // so the whole seed — and hence the real form value — coincides.
        let inner_class = compact_inner(sc_a2_datum(), 6);
        let classification = CartanClassification::build(&inner_class, &class_budget(6)).unwrap();
        let table = full_table(&inner_class, &classification);
        let first = WeylElement::simple_reflection(inner_class.root_system(), 0).unwrap();
        // `[4,2]/3` symmetrized and halved is exactly `[1,2]/3` (the
        // language layer's projection), so this repeats the r0 anchor.
        let factor = vec![rational(1, 3), rational(2, 3)];
        let coch = elected_square_root(&inner_class, &first, &factor, &integer_budget()).unwrap();
        let torus_part = minimal_torus_part(
            &inner_class,
            &classification,
            &table,
            WeakRealFormId(0),
            &coch,
            &first,
            &factor,
        )
        .unwrap();
        assert_eq!(coch, factor);
        assert_eq!(torus_part, ModTwoVector::from_ones(2, [0]).unwrap());
    }

    #[test]
    fn gates_reject_rank_and_coverage_mismatches() {
        let inner_class = compact_inner(sc_a2_datum(), 6);
        let classification = CartanClassification::build(&inner_class, &class_budget(6)).unwrap();
        let table = full_table(&inner_class, &classification);
        let first = WeylElement::simple_reflection(inner_class.root_system(), 0).unwrap();
        let factor = vec![rational(1, 3), rational(2, 3)];
        let coch = factor.clone();

        assert!(matches!(
            minimal_torus_part(
                &inner_class,
                &classification,
                &table,
                WeakRealFormId(0),
                &coch,
                &first,
                &[Rational::ZERO],
            ),
            Err(StructureError::RankMismatch { .. })
        ));
        // A table without the involution's Cartan class is rejected by
        // name, not by a silent miss.
        let empty = InvolutionTable::new(
            &inner_class,
            InvolutionTableBudget::new(64, integer_budget()),
        )
        .unwrap();
        assert_eq!(
            minimal_torus_part(
                &inner_class,
                &classification,
                &empty,
                WeakRealFormId(0),
                &coch,
                &first,
                &factor,
            ),
            Err(StructureError::SeedInvariantViolation {
                invariant: "synthetic involution coverage",
            })
        );
        assert_eq!(
            elected_square_root(&inner_class, &first, &[Rational::ZERO], &integer_budget()),
            Err(StructureError::RankMismatch {
                expected: 2,
                actual: 1,
            })
        );
    }
}
