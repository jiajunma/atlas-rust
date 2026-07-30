//! Word-level Weyl elements on the root-permutation representation.
//!
//! This is stage (a) of the KGB map: the combinatorial substrate the Tits
//! sigma formulas consume — O(1) length and descent queries, multiplication,
//! inverses, twisted conjugation, and on-demand reduced words. Elements are
//! the construction currency of the involution table and Tits stages, not of
//! persistent KGB elements, so no budget knob exists at this layer.
//!
//! An element is represented by its permutation of the enumerated roots of
//! one ambient [`RootSystem`]. The only provenance check expressible per
//! operation is the root-count match; the single-ambient-system discipline
//! is the caller's contract, owned by the KGB stages. Antisymmetry of the
//! permutation is guaranteed by keeping the constructors the only entry
//! points.

use crate::grading::try_capacity;
use crate::{RootId, RootSystem, StructureError, WeylAction};

/// A Weyl-group element as a permutation of the enumerated roots.
///
/// Field order is load-bearing: `permutation` comes first so the derived
/// `Ord` is the documented lexicographic root-permutation order, and since
/// `inverse` and `length` are functions of `permutation` established by
/// every constructor, the derived `Eq`/`Hash` agree with permutation-only
/// equality.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WeylElement {
    permutation: Vec<RootId>,
    inverse: Vec<RootId>,
    length: usize,
}

impl WeylElement {
    pub fn identity(system: &RootSystem) -> Result<Self, StructureError> {
        let count = system.roots().len();
        let mut permutation = try_capacity(count)?;
        for index in 0..count {
            permutation.push(RootId(index));
        }
        Self::from_permutation(system, permutation)
    }

    pub fn simple_reflection(
        system: &RootSystem,
        generator: usize,
    ) -> Result<Self, StructureError> {
        let action = WeylAction::simple_reflection(system.datum(), generator)?;
        Self::from_action(system, &action)
    }

    /// Build the element realizing a matrix-level action, making the two
    /// Weyl layers mutually checkable.
    pub fn from_action(system: &RootSystem, action: &WeylAction) -> Result<Self, StructureError> {
        Self::from_permutation(system, system.action_permutation(action)?)
    }

    /// The single private entry point establishing the inverse (with a free
    /// bijectivity check) and the positivity-counted length.
    fn from_permutation(
        system: &RootSystem,
        permutation: Vec<RootId>,
    ) -> Result<Self, StructureError> {
        let count = system.roots().len();
        if permutation.len() != count {
            return Err(StructureError::WeylElementInvariantViolation {
                invariant: "provenance",
            });
        }
        const UNSET: usize = usize::MAX;
        let mut inverse: Vec<RootId> = try_capacity(count)?;
        inverse.resize(count, RootId(UNSET));
        for (index, image) in permutation.iter().enumerate() {
            let slot =
                inverse
                    .get_mut(image.0)
                    .ok_or(StructureError::WeylElementInvariantViolation {
                        invariant: "permutation range",
                    })?;
            if slot.0 != UNSET {
                return Err(StructureError::WeylElementInvariantViolation {
                    invariant: "permutation bijectivity",
                });
            }
            *slot = RootId(index);
        }
        let length = count_length(system, &permutation);
        Ok(Self {
            permutation,
            inverse,
            length,
        })
    }

    pub fn length(&self) -> usize {
        self.length
    }

    pub fn is_identity(&self) -> bool {
        self.permutation
            .iter()
            .enumerate()
            .all(|(index, image)| image.0 == index)
    }

    pub fn image(&self, root: RootId) -> Option<RootId> {
        self.permutation.get(root.0).copied()
    }

    pub fn image_permutation(&self) -> &[RootId] {
        &self.permutation
    }

    /// Whether `l(s w) < l(w)`: reads the INVERSE vector, since the
    /// condition is `w^{-1}(alpha_s) < 0`.
    pub fn has_left_descent(
        &self,
        system: &RootSystem,
        generator: usize,
    ) -> Result<bool, StructureError> {
        self.check_provenance(system)?;
        let alpha = simple_id(system, generator)?;
        Ok(!system.positivity()[self.inverse[alpha.0].0])
    }

    /// Whether `l(w s) < l(w)`: reads the FORWARD permutation, since the
    /// condition is `w(alpha_s) < 0`.
    pub fn has_right_descent(
        &self,
        system: &RootSystem,
        generator: usize,
    ) -> Result<bool, StructureError> {
        self.check_provenance(system)?;
        let alpha = simple_id(system, generator)?;
        Ok(!system.positivity()[self.permutation[alpha.0].0])
    }

    /// The composite `self after right`, matching `WeylAction::compose`.
    ///
    /// The inverse is maintained by the dual composition in the same pass
    /// (`(uv)^{-1} = v^{-1} u^{-1}`); the length is recomputed from the
    /// positivity slice — operand lengths do not add. A general length
    /// change is cached-length subtraction at the call site.
    pub fn multiply(&self, system: &RootSystem, right: &Self) -> Result<Self, StructureError> {
        self.check_provenance(system)?;
        right.check_provenance(system)?;
        let count = self.permutation.len();
        let mut permutation = try_capacity(count)?;
        let mut inverse = try_capacity(count)?;
        for index in 0..count {
            permutation.push(self.permutation[right.permutation[index].0]);
            inverse.push(right.inverse[self.inverse[index].0]);
        }
        let length = count_length(system, &permutation);
        Ok(Self {
            permutation,
            inverse,
            length,
        })
    }

    /// `s * self` with its length change: `-1` means the length DECREASED —
    /// the branch on which `sigma_mult` adds `m_alpha` — and `+1` the
    /// `sigma_inv_mult` branch.
    pub fn left_multiply_simple(
        &self,
        system: &RootSystem,
        generator: usize,
    ) -> Result<(Self, isize), StructureError> {
        let reflection = Self::simple_reflection(system, generator)?;
        let product = reflection.multiply(system, self)?;
        let change = unit_length_change(self.length, product.length)?;
        Ok((product, change))
    }

    /// `self * s` with its length change, mirroring
    /// [`Self::left_multiply_simple`] for `mult_sigma`/`mult_sigma_inv`.
    pub fn right_multiply_simple(
        &self,
        system: &RootSystem,
        generator: usize,
    ) -> Result<(Self, isize), StructureError> {
        let reflection = Self::simple_reflection(system, generator)?;
        let product = self.multiply(system, &reflection)?;
        let change = unit_length_change(self.length, product.length)?;
        Ok((product, change))
    }

    pub fn inverse(&self) -> Self {
        Self {
            permutation: self.inverse.clone(),
            inverse: self.permutation.clone(),
            length: self.length,
        }
    }

    /// A reduced word by lowest-left-descent peeling, composing
    /// left-to-right: `w = s_{word[0]} * s_{word[1]} * ...`.
    pub fn reduced_word(&self, system: &RootSystem) -> Result<Vec<usize>, StructureError> {
        self.check_provenance(system)?;
        let mut word = try_capacity(self.length)?;
        let mut current = self.clone();
        for _ in 0..self.length {
            let generator = lowest_left_descent(&current, system)?;
            let (next, change) = current.left_multiply_simple(system, generator)?;
            if change != -1 {
                return Err(StructureError::WeylElementInvariantViolation {
                    invariant: "descent peeling",
                });
            }
            current = next;
            word.push(generator);
        }
        if !current.is_identity() {
            return Err(StructureError::WeylElementInvariantViolation {
                invariant: "descent peeling",
            });
        }
        Ok(word)
    }

    /// `s_generator * self * s_{twist(generator)}`, the Weyl shadow of
    /// Tits twisted conjugation.
    ///
    /// The twist must be an involutive permutation of the generators; that
    /// it agrees with the distinguished involution's simple-root action is
    /// the caller's contract. The length change stage (b) consumes
    /// (`d` in `{0, +-2}`, `d/2` the Cayley-length step) is cached-length
    /// subtraction on the result.
    pub fn twisted_conjugate(
        &self,
        system: &RootSystem,
        generator: usize,
        twist: &[usize],
    ) -> Result<Self, StructureError> {
        self.check_provenance(system)?;
        check_twist(twist, system.simple_root_ids().len())?;
        let twisted = *twist
            .get(generator)
            .ok_or(StructureError::IndexOutOfRange {
                index: generator,
                upper_bound: twist.len(),
            })?;
        let left = Self::simple_reflection(system, generator)?;
        let right = Self::simple_reflection(system, twisted)?;
        left.multiply(system, self)?.multiply(system, &right)
    }

    /// The only provenance gate expressible at this layer: root-count
    /// agreement. Same-cardinality foreign systems are undetectable here.
    fn check_provenance(&self, system: &RootSystem) -> Result<(), StructureError> {
        if self.permutation.len() != system.roots().len() {
            return Err(StructureError::WeylElementInvariantViolation {
                invariant: "provenance",
            });
        }
        Ok(())
    }

    /// The canonical reduced word of the upstream transducer
    /// (`WeylGroup::word`, weyl.cpp:944-957), emitted in datum generator
    /// numbers. Upstream documents this expression as "minimal for
    /// ShortLex" (weyl.cpp:911-926): the lexicographically smallest
    /// reduced expression in the INTERNAL generator order carried by
    /// [`WeylInterface`]. The greedy form below is equivalent — the
    /// generators that can start a reduced expression are exactly the
    /// left descents, so peeling the smallest internal left descent
    /// minimizes each successive letter.
    pub fn canonical_word(
        &self,
        system: &RootSystem,
        interface: &WeylInterface,
    ) -> Result<Vec<usize>, StructureError> {
        self.check_provenance(system)?;
        if interface.outward.len() != system.simple_root_ids().len() {
            return Err(StructureError::WeylElementInvariantViolation {
                invariant: "interface provenance",
            });
        }
        let mut word = try_capacity(self.length)?;
        let mut current = self.clone();
        while !current.is_identity() {
            let mut peeled = false;
            for &generator in &interface.outward {
                if current.has_left_descent(system, generator)? {
                    let (next, change) = current.left_multiply_simple(system, generator)?;
                    if change != -1 {
                        return Err(StructureError::WeylElementInvariantViolation {
                            invariant: "descent peeling",
                        });
                    }
                    current = next;
                    word.push(generator);
                    peeled = true;
                    break;
                }
            }
            if !peeled {
                return Err(StructureError::WeylElementInvariantViolation {
                    invariant: "descent peeling",
                });
            }
        }
        Ok(word)
    }
}

/// The internal generator renumbering of the upstream `WeylGroup`
/// constructor (weyl.cpp:495-527): Dynkin components in classification
/// order, each component's Bourbaki `position` taken straight for types
/// A/E/F/G and reversed for types B/C/D. `outward` is upstream `d_out`:
/// internal index -> datum (external) generator.
///
/// Upstream needs the renumbering to keep its transducer tables small;
/// this port keeps only its observable effect, the canonical-word choice
/// of [`WeylElement::canonical_word`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WeylInterface {
    outward: Vec<usize>,
}

impl WeylInterface {
    pub fn new(cartan: &[Vec<i32>]) -> Result<Self, StructureError> {
        let permutation_violation = |_| StructureError::WeylElementInvariantViolation {
            invariant: "interface permutation",
        };
        let components = crate::dynkin::classify(cartan)?;
        let mut outward = try_capacity(cartan.len())?;
        outward.resize(cartan.len(), usize::MAX);
        let mut offset = 0;
        for component in &components {
            let size = component.position.len();
            let reverse = matches!(component.letter, 'B' | 'C' | 'D');
            for (index, &external) in component.position.iter().enumerate() {
                let internal = if reverse {
                    offset + size - 1 - index
                } else {
                    offset + index
                };
                if external >= cartan.len() {
                    return Err(permutation_violation(()));
                }
                let slot = outward
                    .get_mut(internal)
                    .ok_or(())
                    .map_err(permutation_violation)?;
                if *slot != usize::MAX {
                    return Err(permutation_violation(()));
                }
                *slot = external;
            }
            offset += size;
        }
        if outward.contains(&usize::MAX) {
            return Err(permutation_violation(()));
        }
        Ok(Self { outward })
    }

    /// External generator numbers in increasing internal order.
    pub fn outward(&self) -> &[usize] {
        &self.outward
    }
}

fn count_length(system: &RootSystem, permutation: &[RootId]) -> usize {
    let positive = system.positivity();
    permutation
        .iter()
        .enumerate()
        .filter(|(index, image)| positive[*index] && !positive[image.0])
        .count()
}

fn simple_id(system: &RootSystem, generator: usize) -> Result<RootId, StructureError> {
    system
        .simple_root_ids()
        .get(generator)
        .copied()
        .ok_or(StructureError::IndexOutOfRange {
            index: generator,
            upper_bound: system.simple_root_ids().len(),
        })
}

fn lowest_left_descent(
    element: &WeylElement,
    system: &RootSystem,
) -> Result<usize, StructureError> {
    for generator in 0..system.simple_root_ids().len() {
        if element.has_left_descent(system, generator)? {
            return Ok(generator);
        }
    }
    Err(StructureError::WeylElementInvariantViolation {
        invariant: "descent peeling",
    })
}

fn unit_length_change(before: usize, after: usize) -> Result<isize, StructureError> {
    let before = isize::try_from(before).map_err(|_| StructureError::ArithmeticOverflow)?;
    let after = isize::try_from(after).map_err(|_| StructureError::ArithmeticOverflow)?;
    let change = after
        .checked_sub(before)
        .ok_or(StructureError::ArithmeticOverflow)?;
    if change != 1 && change != -1 {
        return Err(StructureError::WeylElementInvariantViolation {
            invariant: "simple length step",
        });
    }
    Ok(change)
}

fn check_twist(twist: &[usize], rank: usize) -> Result<(), StructureError> {
    if twist.len() != rank || twist.iter().any(|&target| target >= rank) {
        return Err(StructureError::WeylElementInvariantViolation {
            invariant: "twist permutation",
        });
    }
    for (index, &target) in twist.iter().enumerate() {
        if twist[target] != index {
            return Err(StructureError::WeylElementInvariantViolation {
                invariant: "twist permutation",
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BasedRootDatum, WeylGroup};

    fn enumerate(cartan: Vec<Vec<i32>>, max_roots: usize) -> RootSystem {
        let datum = BasedRootDatum::standard(cartan).unwrap();
        RootSystem::enumerate(&datum, max_roots).unwrap()
    }

    fn a2() -> RootSystem {
        enumerate(vec![vec![2, -1], vec![-1, 2]], 6)
    }

    fn b2() -> RootSystem {
        enumerate(vec![vec![2, -2], vec![-1, 2]], 8)
    }

    fn closure(system: &RootSystem) -> Vec<WeylElement> {
        let rank = system.simple_root_ids().len();
        let mut elements = vec![WeylElement::identity(system).unwrap()];
        let mut cursor = 0;
        while cursor < elements.len() {
            for generator in 0..rank {
                let (candidate, _) = elements[cursor]
                    .right_multiply_simple(system, generator)
                    .unwrap();
                if !elements.contains(&candidate) {
                    elements.push(candidate);
                }
            }
            cursor += 1;
        }
        elements
    }

    #[test]
    fn a2_closure_has_six_elements_with_textbook_lengths() {
        let system = a2();
        let elements = closure(&system);
        assert_eq!(elements.len(), 6);
        let mut lengths: Vec<usize> = elements.iter().map(WeylElement::length).collect();
        lengths.sort_unstable();
        assert_eq!(lengths, [0, 1, 1, 2, 2, 3]);
        let longest = elements.iter().max_by_key(|e| e.length()).unwrap();
        for index in 0..system.roots().len() {
            if system.positivity()[index] {
                let image = longest.image(RootId(index)).unwrap();
                assert!(!system.positivity()[image.0]);
            }
        }
    }

    #[test]
    fn longest_element_reduced_word_multiplies_back() {
        let system = a2();
        let elements = closure(&system);
        let longest = elements.iter().max_by_key(|e| e.length()).unwrap();
        let word = longest.reduced_word(&system).unwrap();
        assert_eq!(word.len(), 3);
        let mut rebuilt = WeylElement::identity(&system).unwrap();
        for &generator in &word {
            let reflection = WeylElement::simple_reflection(&system, generator).unwrap();
            rebuilt = rebuilt.multiply(&system, &reflection).unwrap();
        }
        assert_eq!(&rebuilt, longest);
    }

    #[test]
    fn descents_split_between_inverse_and_forward_vectors() {
        let system = a2();
        let s0 = WeylElement::simple_reflection(&system, 0).unwrap();
        let s1 = WeylElement::simple_reflection(&system, 1).unwrap();
        let s0s1 = s0.multiply(&system, &s1).unwrap();
        assert!(s0s1.has_left_descent(&system, 0).unwrap());
        assert!(!s0s1.has_left_descent(&system, 1).unwrap());
        assert!(s0s1.has_right_descent(&system, 1).unwrap());
        assert!(!s0s1.has_right_descent(&system, 0).unwrap());
    }

    #[test]
    fn b2_braid_relation_holds_at_the_group_level() {
        let system = b2();
        let s0 = WeylElement::simple_reflection(&system, 0).unwrap();
        let s1 = WeylElement::simple_reflection(&system, 1).unwrap();
        let left = s0
            .multiply(&system, &s1)
            .unwrap()
            .multiply(&system, &s0)
            .unwrap()
            .multiply(&system, &s1)
            .unwrap();
        let right = s1
            .multiply(&system, &s0)
            .unwrap()
            .multiply(&system, &s1)
            .unwrap()
            .multiply(&system, &s0)
            .unwrap();
        assert_eq!(left, right);
        assert_eq!(left.length(), 4);
    }

    #[test]
    fn b2_products_maintain_inverses_and_unit_length_steps() {
        let system = b2();
        let elements = closure(&system);
        assert_eq!(elements.len(), 8);
        for u in &elements {
            for v in &elements {
                let product = u.multiply(&system, v).unwrap();
                let dual = v.inverse().multiply(&system, &u.inverse()).unwrap();
                assert_eq!(product.inverse(), dual);
            }
            for generator in 0..2 {
                let (left, left_change) = u.left_multiply_simple(&system, generator).unwrap();
                assert_eq!(
                    left_change,
                    isize::try_from(left.length()).unwrap() - isize::try_from(u.length()).unwrap()
                );
                let (right, right_change) = u.right_multiply_simple(&system, generator).unwrap();
                assert_eq!(
                    right_change,
                    isize::try_from(right.length()).unwrap() - isize::try_from(u.length()).unwrap()
                );
                assert_eq!(
                    u.has_left_descent(&system, generator).unwrap(),
                    left_change == -1
                );
                assert_eq!(
                    u.has_right_descent(&system, generator).unwrap(),
                    right_change == -1
                );
            }
        }
    }

    #[test]
    fn from_action_round_trips_and_lengths_match_inversion_counts() {
        for system in [a2(), b2()] {
            let group = WeylGroup::new(system.datum().clone());
            for action in group.enumerate_actions(8).unwrap() {
                let element = WeylElement::from_action(&system, &action).unwrap();
                let permutation = system.action_permutation(&action).unwrap();
                assert_eq!(element.image_permutation(), &permutation[..]);
                let mut inversions = 0;
                for (id, root, _) in system.entries() {
                    if system.is_positive(id).unwrap() {
                        let image = action.act(root).unwrap();
                        let image_id = system.id_of(&image).unwrap();
                        if !system.is_positive(image_id).unwrap() {
                            inversions += 1;
                        }
                    }
                }
                assert_eq!(element.length(), inversions);
            }
        }
    }

    #[test]
    fn identity_twist_twisted_conjugation_matches_conjugation() {
        let system = a2();
        let identity_twist = [0usize, 1];
        for element in closure(&system) {
            for generator in 0..2 {
                let twisted = element
                    .twisted_conjugate(&system, generator, &identity_twist)
                    .unwrap();
                let s = WeylElement::simple_reflection(&system, generator).unwrap();
                let conjugated = s
                    .multiply(&system, &element)
                    .unwrap()
                    .multiply(&system, &s)
                    .unwrap();
                assert_eq!(twisted, conjugated);
            }
        }
    }

    #[test]
    fn a1_a1_swap_twist_has_exactly_two_twisted_involutions() {
        let system = enumerate(vec![vec![2, 0], vec![0, 2]], 4);
        let swap = [1usize, 0];
        let elements = closure(&system);
        assert_eq!(elements.len(), 4);
        let mut twisted_involutions = 0;
        for element in &elements {
            let word = element.reduced_word(&system).unwrap();
            let mut delta_image = WeylElement::identity(&system).unwrap();
            for &generator in &word {
                let reflection = WeylElement::simple_reflection(&system, swap[generator]).unwrap();
                delta_image = delta_image.multiply(&system, &reflection).unwrap();
            }
            if element
                .multiply(&system, &delta_image)
                .unwrap()
                .is_identity()
            {
                twisted_involutions += 1;
                assert!(element.is_identity() || element.length() == 2);
            }
        }
        assert_eq!(twisted_involutions, 2);
    }

    #[test]
    fn provenance_twist_and_degenerate_rank_are_guarded() {
        let a2 = a2();
        let b2 = b2();
        let element = WeylElement::identity(&a2).unwrap();
        assert_eq!(
            element.multiply(&b2, &WeylElement::identity(&b2).unwrap()),
            Err(StructureError::WeylElementInvariantViolation {
                invariant: "provenance",
            })
        );
        assert_eq!(element.reduced_word(&a2).unwrap(), Vec::<usize>::new());
        assert_eq!(
            element.twisted_conjugate(&a2, 0, &[1, 1]),
            Err(StructureError::WeylElementInvariantViolation {
                invariant: "twist permutation",
            })
        );
        assert_eq!(
            WeylElement::simple_reflection(&a2, 2),
            Err(StructureError::IndexOutOfRange {
                index: 2,
                upper_bound: 2,
            })
        );
        let torus = BasedRootDatum::from_simple_data(2, vec![], vec![], vec![]).unwrap();
        let torus_system = RootSystem::enumerate(&torus, 0).unwrap();
        let identity = WeylElement::identity(&torus_system).unwrap();
        assert!(identity.is_identity());
        assert_eq!(identity.length(), 0);
        assert_eq!(
            identity.reduced_word(&torus_system).unwrap(),
            Vec::<usize>::new()
        );
    }

    fn from_word(system: &RootSystem, word: &[usize]) -> WeylElement {
        let mut element = WeylElement::identity(system).unwrap();
        for &generator in word {
            let (next, _) = element.right_multiply_simple(system, generator).unwrap();
            element = next;
        }
        element
    }

    /// Every reduced expression of `element`, by right-descent recursion.
    fn all_reduced_words(system: &RootSystem, element: &WeylElement) -> Vec<Vec<usize>> {
        if element.is_identity() {
            return vec![Vec::new()];
        }
        let mut all = Vec::new();
        for generator in 0..system.simple_root_ids().len() {
            if element.has_right_descent(system, generator).unwrap() {
                let (rest, _) = element.right_multiply_simple(system, generator).unwrap();
                for mut word in all_reduced_words(system, &rest) {
                    word.push(generator);
                    all.push(word);
                }
            }
        }
        all
    }

    /// The ShortLex-minimal reduced expression in the internal order:
    /// all reduced expressions share the element's length, so the order
    /// is plain lexicographic on internal indices.
    fn shortlex_min(system: &RootSystem, element: &WeylElement) -> Vec<usize> {
        let interface = WeylInterface::new(system.datum().cartan_matrix()).unwrap();
        let internal = |generator: usize| {
            interface
                .outward()
                .iter()
                .position(|&external| external == generator)
                .unwrap()
        };
        all_reduced_words(system, element)
            .into_iter()
            .min_by_key(|word| {
                word.iter()
                    .map(|&generator| internal(generator))
                    .collect::<Vec<_>>()
            })
            .unwrap()
    }

    #[test]
    fn weyl_interface_renumbers_like_the_upstream_constructor() {
        // Types A keep the Bourbaki (here: given) order straight.
        let a2 = WeylInterface::new(&[vec![2, -1], vec![-1, 2]]).unwrap();
        assert_eq!(a2.outward(), &[0, 1]);
        // Types B and C reverse their component (weyl.cpp:517-521).
        let b2 = WeylInterface::new(&[vec![2, -2], vec![-1, 2]]).unwrap();
        assert_eq!(b2.outward(), &[1, 0]);
        let c2 = WeylInterface::new(&[vec![2, -1], vec![-2, 2]]).unwrap();
        assert_eq!(c2.outward(), &[1, 0]);
        // Type G is not reversed, but the Dynkin classifier already
        // swapped the component to put the short root first.
        let g2 = WeylInterface::new(&[vec![2, -3], vec![-1, 2]]).unwrap();
        assert_eq!(g2.outward(), &[1, 0]);
        let g2_ordered = WeylInterface::new(&[vec![2, -1], vec![-3, 2]]).unwrap();
        assert_eq!(g2_ordered.outward(), &[0, 1]);
        // Type D reverses; disconnected components stay in
        // classification order.
        let d4 = WeylInterface::new(&[
            vec![2, -1, 0, 0],
            vec![-1, 2, -1, -1],
            vec![0, -1, 2, 0],
            vec![0, -1, 0, 2],
        ])
        .unwrap();
        assert_eq!(d4.outward(), &[3, 2, 1, 0]);
        let a1a1 = WeylInterface::new(&[vec![2, 0], vec![0, 2]]).unwrap();
        assert_eq!(a1a1.outward(), &[0, 1]);
        let torus = WeylInterface::new(&[]).unwrap();
        assert_eq!(torus.outward(), &[] as &[usize]);
    }

    #[test]
    fn canonical_word_matches_oracle_anchors() {
        let a2 = a2();
        let a2_interface = WeylInterface::new(a2.datum().cartan_matrix()).unwrap();
        // A2: both braid forms of the longest element print <0.1.0>.
        for word in [&[0, 1, 0][..], &[1, 0, 1][..]] {
            let element = from_word(&a2, word);
            assert_eq!(
                element.canonical_word(&a2, &a2_interface).unwrap(),
                [0, 1, 0]
            );
        }
        let identity = WeylElement::identity(&a2).unwrap();
        assert_eq!(
            identity.canonical_word(&a2, &a2_interface).unwrap(),
            Vec::<usize>::new()
        );
        // w0 # 1 = s1 s0 prints <1.0>.
        let longest = from_word(&a2, &[0, 1, 0]);
        let (product, _) = longest.right_multiply_simple(&a2, 1).unwrap();
        assert_eq!(product.canonical_word(&a2, &a2_interface).unwrap(), [1, 0]);
        // The longest element of A2 is an involution.
        let inverse = longest.inverse();
        assert_eq!(
            inverse.canonical_word(&a2, &a2_interface).unwrap(),
            [0, 1, 0]
        );

        // B2: the internal order is reversed, so both braid forms of the
        // longest element print <1.0.1.0>.
        let b2 = b2();
        let b2_interface = WeylInterface::new(b2.datum().cartan_matrix()).unwrap();
        for word in [&[0, 1, 0, 1][..], &[1, 0, 1, 0][..]] {
            let element = from_word(&b2, word);
            assert_eq!(
                element.canonical_word(&b2, &b2_interface).unwrap(),
                [1, 0, 1, 0]
            );
        }
    }

    #[test]
    fn canonical_word_is_shortlex_minimal_in_the_internal_order() {
        let g2 = enumerate(vec![vec![2, -3], vec![-1, 2]], 12);
        let c2 = enumerate(vec![vec![2, -1], vec![-2, 2]], 8);
        let a3 = enumerate(vec![vec![2, -1, 0], vec![-1, 2, -1], vec![0, -1, 2]], 12);
        for system in [a2(), b2(), c2, g2, a3] {
            let interface = WeylInterface::new(system.datum().cartan_matrix()).unwrap();
            for element in closure(&system) {
                assert_eq!(
                    element.canonical_word(&system, &interface).unwrap(),
                    shortlex_min(&system, &element),
                );
            }
        }
    }
}
