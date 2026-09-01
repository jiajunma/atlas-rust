//! Exact rational torus transport for Atlas's global Tits cross action.
//!
//! Unlike [`crate::TitsCoset`], this carrier retains the full rational
//! cocharacter, including central coordinates.  It is the transport needed
//! while a synthetic strong involution is moved to a canonical Cartan
//! representative; reduction to a fiber's mod-two quotient happens later.

use malachite::base::num::arithmetic::traits::Floor;
use malachite::base::num::basic::traits::Zero;
use malachite::Rational;

use crate::grading::try_capacity;
use crate::twisted_involution::compose_matrices;
use crate::{
    InnerClass, RationalCoweight, RootKind, StructureError, TwistedInvolution, Weight, WeylAction,
};

/// A rational torus factor paired with its twisted involution.
///
/// The torus coordinates are canonical representatives in `[0, 2)`.  Fields
/// stay private so every value has passed the rank, datum, and distinguished-
/// involution provenance gates.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GlobalTitsElement {
    torus_factor: RationalCoweight,
    twisted_involution: TwistedInvolution,
}

impl GlobalTitsElement {
    pub(crate) fn new(
        inner_class: &InnerClass,
        torus_factor: RationalCoweight,
        twisted_involution: TwistedInvolution,
    ) -> Result<Self, StructureError> {
        let rank = inner_class.datum().lattice_rank();
        if torus_factor.dimension() != rank {
            return Err(StructureError::RankMismatch {
                expected: rank,
                actual: torus_factor.dimension(),
            });
        }
        validate_context(inner_class, &twisted_involution)?;
        Ok(Self {
            torus_factor: normalize_torus_factor(torus_factor.coordinates())?,
            twisted_involution,
        })
    }

    pub(crate) fn torus_factor(&self) -> &RationalCoweight {
        &self.torus_factor
    }

    pub(crate) fn twisted_involution(&self) -> &TwistedInvolution {
        &self.twisted_involution
    }

    /// Apply one simple cross action `s * (t,w) * delta(s)`.
    pub(crate) fn crossed_generator(
        &self,
        inner_class: &InnerClass,
        generator: usize,
    ) -> Result<Self, StructureError> {
        validate_context(inner_class, &self.twisted_involution)?;
        let semisimple_rank = inner_class.datum().semisimple_rank();
        if generator >= semisimple_rank {
            return Err(StructureError::IndexOutOfRange {
                index: generator,
                upper_bound: semisimple_rank,
            });
        }

        let roots = inner_class.root_system();
        let simple_root =
            *roots
                .simple_root_ids()
                .get(generator)
                .ok_or(StructureError::IndexOutOfRange {
                    index: generator,
                    upper_bound: roots.simple_root_ids().len(),
                })?;
        let kind = self
            .twisted_involution
            .root_involution()
            .kind(simple_root)
            .ok_or(StructureError::InvalidRootAutomorphism)?;

        let mut torus_coordinates = copy_rationals(self.torus_factor.coordinates())?;
        match kind {
            RootKind::Complex => {
                let pairing = rational_pair(
                    &inner_class.datum().simple_roots()[generator],
                    &torus_coordinates,
                )?;
                add_scaled_coroot(
                    &mut torus_coordinates,
                    &inner_class.datum().simple_coroots()[generator],
                    -pairing,
                )?;
            }
            RootKind::Imaginary => {
                let pairing = rational_pair(
                    &inner_class.datum().simple_roots()[generator],
                    &torus_coordinates,
                )?;
                if pairing != pairing.clone().floor() {
                    return Err(StructureError::InvalidStrongTorusFactor);
                }
                add_scaled_coroot(
                    &mut torus_coordinates,
                    &inner_class.datum().simple_coroots()[generator],
                    Rational::from(1) - pairing,
                )?;
            }
            RootKind::Real => {}
        }
        normalize_coordinates(&mut torus_coordinates);

        let twisted_generator = distinguished_generator_image(inner_class, generator)?;
        let left = WeylAction::simple_reflection(inner_class.datum(), generator)?;
        let right = WeylAction::simple_reflection(inner_class.datum(), twisted_generator)?;
        let action = left
            .compose(self.twisted_involution.weyl_action())?
            .compose(&right)?;
        let twisted_involution = TwistedInvolution::new(
            inner_class.datum(),
            roots,
            inner_class.distinguished_involution().involution(),
            action,
        )?;
        Ok(Self {
            torus_factor: RationalCoweight::from_coordinates(torus_coordinates),
            twisted_involution,
        })
    }

    /// Apply generators from first to last, matching upstream
    /// `cross_act(GlobalTitsElement&, const WeylWord&)`.
    pub(crate) fn crossed_word(
        &self,
        inner_class: &InnerClass,
        word: &[usize],
    ) -> Result<Self, StructureError> {
        validate_context(inner_class, &self.twisted_involution)?;
        let mut current = self.clone();
        for &generator in word {
            current = current.crossed_generator(inner_class, generator)?;
        }
        Ok(current)
    }
}

fn validate_context(
    inner_class: &InnerClass,
    twisted: &TwistedInvolution,
) -> Result<(), StructureError> {
    if twisted.root_involution().involution().datum() != inner_class.datum() {
        return Err(StructureError::DatumMismatch);
    }
    // Table records drop the Weyl factor's matrices; the recomposition
    // check applies to the (always retained) actions of GlobalTits values.
    let Some(action) = twisted.retained_weyl_action() else {
        return Ok(());
    };
    if action.datum() != inner_class.datum() {
        return Err(StructureError::DatumMismatch);
    }
    let distinguished = inner_class.distinguished_involution().involution();
    let stored = twisted.root_involution().involution();
    if compose_matrices(action.matrix(), distinguished.weight_matrix())? != stored.weight_matrix()
        || compose_matrices(action.coweight_matrix(), &distinguished.coweight_matrix().to_vec())?
            != stored.coweight_matrix()
    {
        return Err(StructureError::DistinguishedInvolutionMismatch);
    }
    Ok(())
}

fn distinguished_generator_image(
    inner_class: &InnerClass,
    generator: usize,
) -> Result<usize, StructureError> {
    let simple_roots = inner_class.root_system().simple_root_ids();
    let simple_root = *simple_roots
        .get(generator)
        .ok_or(StructureError::IndexOutOfRange {
            index: generator,
            upper_bound: simple_roots.len(),
        })?;
    let image = inner_class
        .distinguished_involution()
        .image(simple_root)
        .ok_or(StructureError::InvalidBasedAutomorphism)?;
    simple_roots
        .iter()
        .position(|&candidate| candidate == image)
        .ok_or(StructureError::InvalidBasedAutomorphism)
}

fn rational_pair(weight: &Weight, coweight: &[Rational]) -> Result<Rational, StructureError> {
    if weight.rank() != coweight.len() {
        return Err(StructureError::RankMismatch {
            expected: weight.rank(),
            actual: coweight.len(),
        });
    }
    Ok(weight
        .as_slice()
        .iter()
        .zip(coweight)
        .fold(Rational::ZERO, |sum, (&coefficient, coordinate)| {
            sum + Rational::from(coefficient) * coordinate
        }))
}

fn add_scaled_coroot(
    coordinates: &mut [Rational],
    coroot: &crate::Coweight,
    scale: Rational,
) -> Result<(), StructureError> {
    if coordinates.len() != coroot.rank() {
        return Err(StructureError::RankMismatch {
            expected: coroot.rank(),
            actual: coordinates.len(),
        });
    }
    for (coordinate, &direction) in coordinates.iter_mut().zip(coroot.as_slice()) {
        *coordinate += Rational::from(direction) * &scale;
    }
    Ok(())
}

fn normalize_torus_factor(coordinates: &[Rational]) -> Result<RationalCoweight, StructureError> {
    let mut normalized = copy_rationals(coordinates)?;
    normalize_coordinates(&mut normalized);
    Ok(RationalCoweight::from_coordinates(normalized))
}

fn copy_rationals(coordinates: &[Rational]) -> Result<Vec<Rational>, StructureError> {
    let mut copied = try_capacity(coordinates.len())?;
    copied.extend(coordinates.iter().cloned());
    Ok(copied)
}

fn normalize_coordinates(coordinates: &mut [Rational]) {
    for coordinate in coordinates {
        let quotient = (coordinate.clone() / Rational::from(2)).floor();
        *coordinate -= Rational::from(quotient) * Rational::from(2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BasedRootDatum, Coweight, LatticeInvolution, RootSystem, WeylGroup};

    fn rational(numerator: i32, denominator: i32) -> Rational {
        Rational::from(numerator) / Rational::from(denominator)
    }

    fn compact_inner(datum: BasedRootDatum, root_budget: usize) -> InnerClass {
        let distinguished = LatticeInvolution::identity(&datum).unwrap();
        InnerClass::new(datum, distinguished, root_budget).unwrap()
    }

    fn twisted(inner_class: &InnerClass, action: WeylAction) -> TwistedInvolution {
        TwistedInvolution::new(
            inner_class.datum(),
            inner_class.root_system(),
            inner_class.distinguished_involution().involution(),
            action,
        )
        .unwrap()
    }

    fn element(
        inner_class: &InnerClass,
        coordinates: Vec<Rational>,
        action: WeylAction,
    ) -> GlobalTitsElement {
        GlobalTitsElement::new(
            inner_class,
            RationalCoweight::from_coordinates(coordinates),
            twisted(inner_class, action),
        )
        .unwrap()
    }

    fn adjoint_a1() -> BasedRootDatum {
        BasedRootDatum::from_simple_data(
            1,
            vec![vec![2]],
            vec![Weight::new(vec![2])],
            vec![Coweight::new(vec![1])],
        )
        .unwrap()
    }

    #[test]
    fn normalizes_every_coordinate_modulo_two() {
        let datum = BasedRootDatum::from_simple_data(
            2,
            vec![vec![2]],
            vec![Weight::new(vec![2, 0])],
            vec![Coweight::new(vec![1, 0])],
        )
        .unwrap();
        let inner_class = compact_inner(datum.clone(), 2);
        let value = element(
            &inner_class,
            vec![rational(-1, 2), rational(9, 2)],
            WeylGroup::new(datum).identity().unwrap(),
        );

        assert_eq!(
            value.torus_factor().coordinates(),
            &[rational(3, 2), rational(1, 2)]
        );
    }

    #[test]
    fn rank_zero_and_an_empty_word_are_identity_transport() {
        let datum = BasedRootDatum::from_simple_data(0, vec![], vec![], vec![]).unwrap();
        let inner_class = compact_inner(datum.clone(), 0);
        let value = element(
            &inner_class,
            vec![],
            WeylGroup::new(datum).identity().unwrap(),
        );

        assert_eq!(value.crossed_word(&inner_class, &[]).unwrap(), value);
    }

    #[test]
    fn a1_imaginary_cross_distinguishes_compact_and_noncompact_factors() {
        let datum = adjoint_a1();
        let inner_class = compact_inner(datum.clone(), 2);
        let identity = WeylGroup::new(datum).identity().unwrap();
        let compact = element(&inner_class, vec![rational(1, 2)], identity.clone());
        let noncompact = element(&inner_class, vec![Rational::ZERO], identity);

        assert_eq!(
            compact
                .crossed_generator(&inner_class, 0)
                .unwrap()
                .torus_factor()
                .coordinates(),
            &[rational(1, 2)]
        );
        assert_eq!(
            noncompact
                .crossed_generator(&inner_class, 0)
                .unwrap()
                .torus_factor()
                .coordinates(),
            &[Rational::from(1)]
        );
    }

    #[test]
    fn a1_imaginary_cross_requires_an_integral_root_pairing() {
        let datum = adjoint_a1();
        let inner_class = compact_inner(datum.clone(), 2);
        let value = element(
            &inner_class,
            vec![rational(1, 4)],
            WeylGroup::new(datum).identity().unwrap(),
        );

        assert_eq!(
            value.crossed_generator(&inner_class, 0),
            Err(StructureError::InvalidStrongTorusFactor)
        );
    }

    #[test]
    fn a1_real_cross_leaves_both_components_unchanged() {
        let datum = adjoint_a1();
        let inner_class = compact_inner(datum.clone(), 2);
        let reflection = WeylGroup::new(datum).simple_reflection(0).unwrap();
        let value = element(&inner_class, vec![rational(3, 4)], reflection);

        assert_eq!(value.crossed_generator(&inner_class, 0).unwrap(), value);
    }

    #[test]
    fn a2_complex_cross_reflects_the_rational_coweight() {
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let inner_class = compact_inner(datum.clone(), 6);
        let group = WeylGroup::new(datum.clone());
        let first = group.simple_reflection(0).unwrap();
        let value = element(
            &inner_class,
            vec![rational(1, 3), rational(1, 2)],
            first.clone(),
        );
        let crossed = value.crossed_generator(&inner_class, 1).unwrap();
        let second = group.simple_reflection(1).unwrap();
        let expected_action = second.compose(&first).unwrap().compose(&second).unwrap();

        assert_eq!(
            crossed.torus_factor().coordinates(),
            &[rational(5, 6), rational(3, 2)]
        );
        assert_eq!(
            crossed.twisted_involution(),
            &twisted(&inner_class, expected_action)
        );
    }

    #[test]
    fn a2_word_execution_is_forward_and_noncommuting() {
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let inner_class = compact_inner(datum.clone(), 6);
        let value = element(
            &inner_class,
            vec![Rational::ZERO, Rational::ZERO],
            WeylGroup::new(datum).identity().unwrap(),
        );

        let forward = value.crossed_word(&inner_class, &[0, 1]).unwrap();
        let explicit = value
            .crossed_generator(&inner_class, 0)
            .unwrap()
            .crossed_generator(&inner_class, 1)
            .unwrap();
        let reverse = value.crossed_word(&inner_class, &[1, 0]).unwrap();

        assert_eq!(forward, explicit);
        assert_eq!(
            forward.torus_factor().coordinates(),
            &[Rational::ZERO, Rational::from(1)]
        );
        assert_eq!(
            reverse.torus_factor().coordinates(),
            &[Rational::from(1), Rational::ZERO]
        );
        assert_ne!(forward, reverse);
    }

    #[test]
    fn b2_complex_cross_uses_the_coroot_not_the_root_direction() {
        let datum = BasedRootDatum::standard(vec![vec![2, -2], vec![-1, 2]]).unwrap();
        let inner_class = compact_inner(datum.clone(), 8);
        let group = WeylGroup::new(datum);
        let value = element(
            &inner_class,
            vec![Rational::ZERO, rational(1, 2)],
            group.simple_reflection(0).unwrap(),
        );

        assert_eq!(
            value
                .crossed_generator(&inner_class, 1)
                .unwrap()
                .torus_factor()
                .coordinates(),
            &[Rational::from(1), rational(3, 2)]
        );
    }

    #[test]
    fn a1_with_central_torus_preserves_the_central_coordinate() {
        let datum = BasedRootDatum::from_simple_data(
            2,
            vec![vec![2]],
            vec![Weight::new(vec![2, 0])],
            vec![Coweight::new(vec![1, 0])],
        )
        .unwrap();
        let inner_class = compact_inner(datum.clone(), 2);
        let value = element(
            &inner_class,
            vec![Rational::ZERO, rational(7, 3)],
            WeylGroup::new(datum).identity().unwrap(),
        );

        assert_eq!(
            value
                .crossed_generator(&inner_class, 0)
                .unwrap()
                .torus_factor()
                .coordinates(),
            &[Rational::from(1), rational(1, 3)]
        );
    }

    #[test]
    fn rejects_rank_generator_datum_and_distinguished_mismatches() {
        let datum = adjoint_a1();
        let inner_class = compact_inner(datum.clone(), 2);
        let identity = WeylGroup::new(datum.clone()).identity().unwrap();
        assert_eq!(
            GlobalTitsElement::new(
                &inner_class,
                RationalCoweight::from_coordinates(vec![]),
                twisted(&inner_class, identity.clone()),
            ),
            Err(StructureError::RankMismatch {
                expected: 1,
                actual: 0,
            })
        );
        let value = element(&inner_class, vec![Rational::ZERO], identity);
        assert_eq!(
            value.crossed_generator(&inner_class, 1),
            Err(StructureError::IndexOutOfRange {
                index: 1,
                upper_bound: 1,
            })
        );

        let foreign_datum = BasedRootDatum::standard(vec![vec![2]]).unwrap();
        let foreign_inner = compact_inner(foreign_datum.clone(), 2);
        let foreign_twisted = twisted(
            &foreign_inner,
            WeylGroup::new(foreign_datum).identity().unwrap(),
        );
        assert_eq!(
            GlobalTitsElement::new(
                &inner_class,
                RationalCoweight::from_coordinates(vec![Rational::ZERO]),
                foreign_twisted,
            ),
            Err(StructureError::DatumMismatch)
        );

        let a2 = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let compact_a2 = compact_inner(a2.clone(), 6);
        let roots = RootSystem::enumerate(&a2, 6).unwrap();
        let foreign_distinguished = LatticeInvolution::new(
            &a2,
            vec![vec![0, 1], vec![1, 0]],
            vec![vec![0, 1], vec![1, 0]],
        )
        .unwrap();
        let foreign_twisted = TwistedInvolution::new(
            &a2,
            &roots,
            &foreign_distinguished,
            WeylGroup::new(a2.clone()).identity().unwrap(),
        )
        .unwrap();
        assert_eq!(
            GlobalTitsElement::new(
                &compact_a2,
                RationalCoweight::from_coordinates(vec![Rational::ZERO, Rational::ZERO]),
                foreign_twisted,
            ),
            Err(StructureError::DistinguishedInvolutionMismatch)
        );
    }
}
