use crate::grading::try_capacity;
use crate::twisted_involution::compose_matrices;
use crate::{
    InnerClass, RootId, RootKind, RootSystem, StructureError, TwistedInvolution, Weight, WeylAction,
};

/// One peeled letter of an involution word, in peel order.
enum Letter {
    Cayley(usize),
    Cross(usize),
}

/// The Cayley/cross factorization of a twisted involution relative to the
/// distinguished involution of its inner class.
///
/// The invariant, verified at construction: starting from the distinguished
/// involution, twisted-conjugating by the cross word in order and then
/// left-multiplying by the reflection of each Cayley root reproduces the
/// input, with every Cayley root imaginary at its replay step. Neither part
/// is unique across ports — Atlas peels in its internal transducer order —
/// so differential comparison must use replay-invariant or label-level
/// results, never these raw parts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CayleyCrossDecomposition {
    cayley_roots: Vec<RootId>,
    cross_word: Vec<usize>,
    cross_action: WeylAction,
    twisted_involution: TwistedInvolution,
}

impl CayleyCrossDecomposition {
    /// Decompose a twisted involution of this inner class.
    ///
    /// The inner class supplies the distinguished involution, which is not
    /// recoverable from the twisted involution itself, and its based-
    /// automorphism guarantee, which grounds the generator-index cross word.
    /// `max_peeling_steps` is defensive: given the provenance gate, peeling
    /// strictly reduces the Weyl length by one or two per step.
    pub fn build(
        inner_class: &InnerClass,
        twisted: &TwistedInvolution,
        max_peeling_steps: usize,
    ) -> Result<Self, StructureError> {
        let datum = inner_class.datum();
        let root_system = inner_class.root_system();
        let delta_data = inner_class.distinguished_involution();
        let delta = delta_data.involution();
        if twisted.weyl_action().datum() != datum {
            return Err(StructureError::DatumMismatch);
        }
        // Factorization provenance: the stored composed involution must be
        // exactly `w after delta` for THIS delta. This also underwrites the
        // termination argument, since it forces `w^{-1} = delta w delta`.
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

        let semisimple_rank = datum.semisimple_rank();
        let mut simple_ids = try_capacity(semisimple_rank)?;
        for root in datum.simple_roots() {
            simple_ids.push(
                root_system
                    .id_of(root)
                    .ok_or(StructureError::InvalidRootAutomorphism)?,
            );
        }
        let mut twist = try_capacity(semisimple_rank)?;
        for &simple_id in &simple_ids {
            let image = delta_data
                .image(simple_id)
                .ok_or(StructureError::InvalidBasedAutomorphism)?;
            let position = simple_ids
                .iter()
                .position(|&candidate| candidate == image)
                .ok_or(StructureError::InvalidBasedAutomorphism)?;
            twist.push(position);
        }
        let identity = WeylAction::identity(datum)?;

        // Peel: lowest external descent first. A descent's kind decides the
        // letter: Real is a Cayley letter, Complex a cross letter, and an
        // imaginary root can never be a descent.
        let mut letters: Vec<Letter> = Vec::new();
        let mut current = twisted.clone();
        let mut steps = 0_usize;
        loop {
            let mut descent = None;
            for (generator, &simple_id) in simple_ids.iter().enumerate() {
                let theta_image = current
                    .root_involution()
                    .image(simple_id)
                    .ok_or(StructureError::InvalidRootAutomorphism)?;
                let image = delta_data
                    .image(theta_image)
                    .ok_or(StructureError::InvalidRootAutomorphism)?;
                let coordinates = root_system.simple_coordinates(image).ok_or(
                    StructureError::IndexOutOfRange {
                        index: image.0,
                        upper_bound: root_system.roots().len(),
                    },
                )?;
                if coordinates.iter().all(|&coordinate| coordinate <= 0) {
                    descent = Some(generator);
                    break;
                }
            }
            let Some(generator) = descent else {
                if current.weyl_action() != &identity {
                    return Err(StructureError::CayleyCrossInvariantViolation {
                        invariant: "peeling termination",
                    });
                }
                break;
            };
            if steps == max_peeling_steps {
                return Err(StructureError::CayleyCrossResourceLimit {
                    resource: "peeling steps",
                    limit: max_peeling_steps,
                });
            }
            steps = steps
                .checked_add(1)
                .ok_or(StructureError::ArithmeticOverflow)?;
            let simple_id = simple_ids[generator];
            let kind = current
                .root_involution()
                .kind(simple_id)
                .ok_or(StructureError::InvalidRootAutomorphism)?;
            let reflection = WeylAction::simple_reflection(datum, generator)?;
            let new_action = match kind {
                RootKind::Real => {
                    letters.push(Letter::Cayley(generator));
                    reflection.compose(current.weyl_action())?
                }
                RootKind::Complex => {
                    letters.push(Letter::Cross(generator));
                    reflection
                        .compose(current.weyl_action())?
                        .compose(&WeylAction::simple_reflection(datum, twist[generator])?)?
                }
                RootKind::Imaginary => {
                    return Err(StructureError::CayleyCrossInvariantViolation {
                        invariant: "descent kind",
                    });
                }
            };
            current = TwistedInvolution::new(datum, root_system, delta, new_action)?;
        }

        // Replay the word in reverse, growing the Cayley set; cross letters
        // reflect every root already collected.
        let mut cayley_roots: Vec<RootId> = Vec::new();
        let mut cross_word: Vec<usize> = Vec::new();
        for letter in letters.iter().rev() {
            match *letter {
                Letter::Cayley(generator) => cayley_roots.push(simple_ids[generator]),
                Letter::Cross(generator) => {
                    cross_word.push(generator);
                    for entry in cayley_roots.iter_mut() {
                        let weight =
                            root_system
                                .root(*entry)
                                .ok_or(StructureError::IndexOutOfRange {
                                    index: entry.0,
                                    upper_bound: root_system.roots().len(),
                                })?;
                        let reflected = datum.reflect_weight(generator, weight)?;
                        *entry = root_system
                            .id_of(&reflected)
                            .ok_or(StructureError::InvalidRootAutomorphism)?;
                    }
                }
            }
        }

        ensure_pairwise_orthogonal(root_system, &cayley_roots)?;
        let mut cayley_roots = long_orthogonalize(root_system, cayley_roots)?;
        for entry in cayley_roots.iter_mut() {
            *entry = positive_form(root_system, *entry)?;
        }
        cayley_roots.sort_unstable();

        let mut cross_action = identity.clone();
        for &generator in &cross_word {
            cross_action =
                cross_action.compose(&WeylAction::simple_reflection(datum, generator)?)?;
        }

        // Replay verification: cross letters in order, then ascending Cayley
        // reflections with per-step imaginarity, must reproduce the input.
        let mut replay = TwistedInvolution::new(datum, root_system, delta, identity)?;
        for &generator in &cross_word {
            let reflection = WeylAction::simple_reflection(datum, generator)?;
            let new_action = reflection
                .compose(replay.weyl_action())?
                .compose(&WeylAction::simple_reflection(datum, twist[generator])?)?;
            replay = TwistedInvolution::new(datum, root_system, delta, new_action)?;
        }
        for &root in &cayley_roots {
            if replay.root_involution().kind(root) != Some(RootKind::Imaginary) {
                return Err(StructureError::CayleyCrossInvariantViolation {
                    invariant: "Cayley root imaginary",
                });
            }
            let reflection = WeylAction::root_reflection(datum, root_system, root)?;
            let new_action = reflection.compose(replay.weyl_action())?;
            replay = TwistedInvolution::new(datum, root_system, delta, new_action)?;
        }
        if replay != *twisted {
            return Err(StructureError::CayleyCrossInvariantViolation {
                invariant: "replay equality",
            });
        }

        Ok(Self {
            cayley_roots,
            cross_word,
            cross_action,
            twisted_involution: twisted.clone(),
        })
    }

    /// Strongly orthogonal, positive, ascending.
    pub fn cayley_roots(&self) -> &[RootId] {
        &self.cayley_roots
    }

    /// Simple generators in application order (first entry applied first).
    pub fn cross_word(&self) -> &[usize] {
        &self.cross_word
    }

    /// The replayed composite of the cross word.
    pub fn cross_action(&self) -> &WeylAction {
        &self.cross_action
    }

    /// The decomposed twisted involution, for zero-cost identity checks by
    /// consumers.
    pub fn twisted_involution(&self) -> &TwistedInvolution {
        &self.twisted_involution
    }
}

fn ensure_pairwise_orthogonal(
    root_system: &RootSystem,
    roots: &[RootId],
) -> Result<(), StructureError> {
    for (index, &left) in roots.iter().enumerate() {
        for &right in &roots[index + 1..] {
            if root_system.bracket(left, right)? != 0 || root_system.bracket(right, left)? != 0 {
                return Err(StructureError::CayleyCrossInvariantViolation {
                    invariant: "orthogonal Cayley set",
                });
            }
        }
    }
    Ok(())
}

/// Replace orthogonal short pairs whose sum is a root — B2 pairs — by the
/// long sum and difference. One ascending sweep per replacement suffices:
/// every replacement output is long, and long roots never re-pair, so the
/// long-root count strictly increases.
fn long_orthogonalize(
    root_system: &RootSystem,
    mut roots: Vec<RootId>,
) -> Result<Vec<RootId>, StructureError> {
    roots.sort_unstable();
    'restart: loop {
        for i in 0..roots.len() {
            for j in (i + 1)..roots.len() {
                if let Some(sum) = combine_roots(root_system, roots[i], roots[j], false)? {
                    let difference = combine_roots(root_system, roots[i], roots[j], true)?.ok_or(
                        StructureError::CayleyCrossInvariantViolation {
                            invariant: "B2 pair",
                        },
                    )?;
                    roots[i] = sum;
                    roots[j] = positive_form(root_system, difference)?;
                    roots.sort_unstable();
                    continue 'restart;
                }
            }
        }
        return Ok(roots);
    }
}

/// The id of `left + right` (or `left - right`), when that vector is a root.
fn combine_roots(
    root_system: &RootSystem,
    left: RootId,
    right: RootId,
    subtract: bool,
) -> Result<Option<RootId>, StructureError> {
    let left_weight = root_system
        .root(left)
        .ok_or(StructureError::IndexOutOfRange {
            index: left.0,
            upper_bound: root_system.roots().len(),
        })?;
    let right_weight = root_system
        .root(right)
        .ok_or(StructureError::IndexOutOfRange {
            index: right.0,
            upper_bound: root_system.roots().len(),
        })?;
    let mut coordinates = try_capacity(left_weight.as_slice().len())?;
    for (&a, &b) in left_weight.as_slice().iter().zip(right_weight.as_slice()) {
        let value = if subtract {
            a.checked_sub(b)
        } else {
            a.checked_add(b)
        }
        .ok_or(StructureError::ArithmeticOverflow)?;
        coordinates.push(value);
    }
    Ok(root_system.id_of(&Weight::new(coordinates)))
}

/// The positive root in `{root, -root}`.
fn positive_form(root_system: &RootSystem, root: RootId) -> Result<RootId, StructureError> {
    let coordinates =
        root_system
            .simple_coordinates(root)
            .ok_or(StructureError::IndexOutOfRange {
                index: root.0,
                upper_bound: root_system.roots().len(),
            })?;
    if coordinates.iter().all(|&coordinate| coordinate >= 0) {
        return Ok(root);
    }
    let weight = root_system
        .root(root)
        .ok_or(StructureError::IndexOutOfRange {
            index: root.0,
            upper_bound: root_system.roots().len(),
        })?;
    let mut negated = try_capacity(weight.as_slice().len())?;
    for &coordinate in weight.as_slice() {
        negated.push(
            coordinate
                .checked_neg()
                .ok_or(StructureError::ArithmeticOverflow)?,
        );
    }
    root_system
        .id_of(&Weight::new(negated))
        .ok_or(StructureError::InvalidRootAutomorphism)
}

#[cfg(test)]
mod tests {
    use crate::{BasedRootDatum, LatticeInvolution, WeylGroup};

    use super::*;

    fn inner(datum: &BasedRootDatum, involution: LatticeInvolution, budget: usize) -> InnerClass {
        InnerClass::new(datum.clone(), involution, budget).unwrap()
    }

    fn tw(inner_class: &InnerClass, action: WeylAction) -> TwistedInvolution {
        TwistedInvolution::new(
            inner_class.datum(),
            inner_class.root_system(),
            inner_class.distinguished_involution().involution(),
            action,
        )
        .unwrap()
    }

    #[test]
    fn distinguished_involutions_decompose_to_empty_parts() {
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let identity_class = inner(&datum, LatticeInvolution::identity(&datum).unwrap(), 6);
        let identity_action = WeylGroup::new(datum.clone()).identity().unwrap();
        let decomposition = CayleyCrossDecomposition::build(
            &identity_class,
            &tw(&identity_class, identity_action.clone()),
            8,
        )
        .unwrap();
        assert!(decomposition.cayley_roots().is_empty());
        assert!(decomposition.cross_word().is_empty());

        let twist = LatticeInvolution::new(
            &datum,
            vec![vec![0, 1], vec![1, 0]],
            vec![vec![0, 1], vec![1, 0]],
        )
        .unwrap();
        let twist_class = inner(&datum, twist, 6);
        let decomposition =
            CayleyCrossDecomposition::build(&twist_class, &tw(&twist_class, identity_action), 8)
                .unwrap();
        assert!(decomposition.cayley_roots().is_empty());
        assert!(decomposition.cross_word().is_empty());
    }

    #[test]
    fn a1_x_a1_identity_gives_one_cayley_and_swap_gives_one_cross() {
        let datum = BasedRootDatum::standard(vec![vec![2, 0], vec![0, 2]]).unwrap();
        let group = WeylGroup::new(datum.clone());

        let identity_class = inner(&datum, LatticeInvolution::identity(&datum).unwrap(), 4);
        let s0 = group.simple_reflection(0).unwrap();
        let decomposition =
            CayleyCrossDecomposition::build(&identity_class, &tw(&identity_class, s0.clone()), 8)
                .unwrap();
        assert_eq!(
            decomposition.cayley_roots(),
            &[identity_class
                .root_system()
                .id_of(&Weight::new(vec![1, 0]))
                .unwrap()]
        );
        assert!(decomposition.cross_word().is_empty());

        let swap = LatticeInvolution::new(
            &datum,
            vec![vec![0, 1], vec![1, 0]],
            vec![vec![0, 1], vec![1, 0]],
        )
        .unwrap();
        let swap_class = inner(&datum, swap, 4);
        let product = s0.compose(&group.simple_reflection(1).unwrap()).unwrap();
        let decomposition =
            CayleyCrossDecomposition::build(&swap_class, &tw(&swap_class, product), 8).unwrap();
        assert!(decomposition.cayley_roots().is_empty());
        assert_eq!(decomposition.cross_word().len(), 1);
    }

    #[test]
    fn a2_identity_exercises_both_letter_kinds() {
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let identity_class = inner(&datum, LatticeInvolution::identity(&datum).unwrap(), 6);
        let group = WeylGroup::new(datum.clone());
        let root_system = identity_class.root_system();

        let s0 = group.simple_reflection(0).unwrap();
        let single =
            CayleyCrossDecomposition::build(&identity_class, &tw(&identity_class, s0.clone()), 8)
                .unwrap();
        assert_eq!(
            single.cayley_roots(),
            &[root_system.id_of(&Weight::new(vec![1, 0])).unwrap()]
        );
        assert!(single.cross_word().is_empty());

        let s1 = group.simple_reflection(1).unwrap();
        let longest = s0.compose(&s1).unwrap().compose(&s0).unwrap();
        let mixed =
            CayleyCrossDecomposition::build(&identity_class, &tw(&identity_class, longest), 8)
                .unwrap();
        assert_eq!(
            mixed.cayley_roots(),
            &[root_system.id_of(&Weight::new(vec![1, 1])).unwrap()]
        );
        assert_eq!(mixed.cross_word(), &[0]);
        assert_eq!(
            mixed.cross_action(),
            &WeylGroup::new(datum.clone()).simple_reflection(0).unwrap()
        );
    }

    #[test]
    fn b2_short_pinning_long_orthogonalizes_and_long_pinning_does_not() {
        // alpha_0 short: the longest involution's raw Cayley pair is short
        // and gets replaced by the long sum and difference.
        let short_first = BasedRootDatum::standard(vec![vec![2, -1], vec![-2, 2]]).unwrap();
        let short_class = inner(
            &short_first,
            LatticeInvolution::identity(&short_first).unwrap(),
            8,
        );
        let group = WeylGroup::new(short_first.clone());
        let s0 = group.simple_reflection(0).unwrap();
        let s1 = group.simple_reflection(1).unwrap();
        let longest = s0
            .compose(&s1)
            .unwrap()
            .compose(&s0)
            .unwrap()
            .compose(&s1)
            .unwrap();
        let decomposition =
            CayleyCrossDecomposition::build(&short_class, &tw(&short_class, longest), 16).unwrap();
        let root_system = short_class.root_system();
        assert_eq!(
            decomposition.cayley_roots(),
            &[
                root_system.id_of(&Weight::new(vec![0, 1])).unwrap(),
                root_system.id_of(&Weight::new(vec![2, 1])).unwrap(),
            ]
        );
        assert_eq!(decomposition.cross_word(), &[1]);

        // alpha_0 long: the raw set is already long; the pass is a no-op.
        let long_first = BasedRootDatum::standard(vec![vec![2, -2], vec![-1, 2]]).unwrap();
        let long_class = inner(
            &long_first,
            LatticeInvolution::identity(&long_first).unwrap(),
            8,
        );
        let group = WeylGroup::new(long_first.clone());
        let s0 = group.simple_reflection(0).unwrap();
        let s1 = group.simple_reflection(1).unwrap();
        let longest = s0
            .compose(&s1)
            .unwrap()
            .compose(&s0)
            .unwrap()
            .compose(&s1)
            .unwrap();
        let decomposition =
            CayleyCrossDecomposition::build(&long_class, &tw(&long_class, longest), 16).unwrap();
        let root_system = long_class.root_system();
        assert_eq!(
            decomposition.cayley_roots(),
            &[
                root_system.id_of(&Weight::new(vec![1, 0])).unwrap(),
                root_system.id_of(&Weight::new(vec![1, 2])).unwrap(),
            ]
        );
        assert_eq!(decomposition.cross_word(), &[1]);
    }

    #[test]
    fn every_enumerated_twisted_involution_decomposes_and_replays() {
        let cases = [
            (
                BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap(),
                true,
                6,
            ),
            (
                BasedRootDatum::standard(vec![vec![2, -2], vec![-1, 2]]).unwrap(),
                false,
                8,
            ),
        ];
        for (datum, with_twist, budget) in cases {
            let involutions: Vec<LatticeInvolution> = if with_twist {
                vec![
                    LatticeInvolution::identity(&datum).unwrap(),
                    LatticeInvolution::new(
                        &datum,
                        vec![vec![0, 1], vec![1, 0]],
                        vec![vec![0, 1], vec![1, 0]],
                    )
                    .unwrap(),
                ]
            } else {
                vec![LatticeInvolution::identity(&datum).unwrap()]
            };
            for distinguished in involutions {
                let inner_class = inner(&datum, distinguished, budget);
                for twisted in inner_class.twisted_involutions(budget).unwrap() {
                    let decomposition =
                        CayleyCrossDecomposition::build(&inner_class, &twisted, 64).unwrap();
                    for (index, &left) in decomposition.cayley_roots().iter().enumerate() {
                        for &right in &decomposition.cayley_roots()[index + 1..] {
                            assert_eq!(inner_class.root_system().bracket(left, right).unwrap(), 0);
                        }
                    }
                    assert_eq!(decomposition.twisted_involution(), &twisted);
                }
            }
        }
    }

    #[test]
    fn rejects_budgets_and_foreign_provenance() {
        let datum = BasedRootDatum::standard(vec![vec![2, -1], vec![-1, 2]]).unwrap();
        let identity_class = inner(&datum, LatticeInvolution::identity(&datum).unwrap(), 6);
        let s0 = WeylGroup::new(datum.clone()).simple_reflection(0).unwrap();
        assert!(matches!(
            CayleyCrossDecomposition::build(&identity_class, &tw(&identity_class, s0), 0),
            Err(StructureError::CayleyCrossResourceLimit {
                resource: "peeling steps",
                limit: 0,
            })
        ));

        let b2 = BasedRootDatum::standard(vec![vec![2, -2], vec![-1, 2]]).unwrap();
        let b2_class = inner(&b2, LatticeInvolution::identity(&b2).unwrap(), 8);
        let foreign = tw(&b2_class, WeylGroup::new(b2.clone()).identity().unwrap());
        assert!(matches!(
            CayleyCrossDecomposition::build(&identity_class, &foreign, 8),
            Err(StructureError::DatumMismatch)
        ));

        let twist = LatticeInvolution::new(
            &datum,
            vec![vec![0, 1], vec![1, 0]],
            vec![vec![0, 1], vec![1, 0]],
        )
        .unwrap();
        let twist_backed = TwistedInvolution::new(
            &datum,
            identity_class.root_system(),
            &twist,
            WeylGroup::new(datum.clone()).identity().unwrap(),
        )
        .unwrap();
        assert!(matches!(
            CayleyCrossDecomposition::build(&identity_class, &twist_backed, 8),
            Err(StructureError::DistinguishedInvolutionMismatch)
        ));
    }
}
