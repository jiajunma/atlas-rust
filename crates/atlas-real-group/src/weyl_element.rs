//! Word-level Weyl elements on the root-permutation representation.
//!
//! This is stage (a) of the KGB map: the combinatorial substrate the Tits
//! sigma formulas consume — O(1) length and descent queries, multiplication,
//! inverses, twisted conjugation, and on-demand reduced words. Elements are
//! the construction currency of the involution table and Tits stages, not of
//! persistent KGB elements, so no budget knob exists at this layer.
//!
//! An element is represented by its permutation of the enumerated roots of
//! one ambient [`RootSystem`], stored with its inverse in ONE exact-size
//! flat buffer (`data[..count]` forward, `data[count..]` inverse): one
//! allocation per element instead of two, and a pointer-sized struct, so
//! the element stays cheap to store per involution/Tits record. The only
//! provenance check expressible per operation is the root-count match; the
//! single-ambient-system discipline is the caller's contract, owned by the
//! KGB stages. Antisymmetry of the permutation is guaranteed by keeping the
//! constructors the only entry points.
//!
//! The [`ParabolicPieces`] table reproduces the upstream transducer's
//! `EltPiece` indexing (weyl.cpp:289-416): the parabolic-subquotient piece
//! list is the tie-break of the upstream involution ordering
//! (`Cartan_orbits::comparer`, involutions.cpp:420-428, which compares the
//! `WeylElt::pieces` arrays lexicographically), so the KGB renumbering
//! consumes it verbatim.

use std::collections::BTreeMap;

use crate::grading::try_capacity;
use crate::{RootId, RootSystem, StructureError, WeylAction};

/// The single construction site: one exact-size buffer holding the forward
/// permutation in `[..count]` and its inverse in `[count..]`, filled by
/// `fill` through the split views. Keeps the `try_capacity`
/// allocation-failure gate of the two-`Vec` implementation.
fn build_data(
    count: usize,
    fill: impl FnOnce(&mut [RootId], &mut [RootId]) -> Result<(), StructureError>,
) -> Result<Box<[RootId]>, StructureError> {
    let mut data = try_capacity(
        count
            .checked_add(count)
            .ok_or(StructureError::ArithmeticOverflow)?,
    )?;
    data.resize(2 * count, RootId(0));
    let (permutation, inverse) = data.split_at_mut(count);
    fill(permutation, inverse)?;
    debug_assert_eq!(data.len(), data.capacity());
    Ok(data.into_boxed_slice())
}

/// A Weyl-group element as a permutation of the enumerated roots.
///
/// Field order is load-bearing: `data` comes first so the derived `Ord` is
/// the documented lexicographic root-permutation order (the inverse half is
/// a function of the forward half, so it can only refine ties between
/// foreign elements, which no single-system consumer ever compares), and
/// since the inverse and `length` are functions of the permutation
/// established by every constructor, the derived `Eq`/`Hash` agree with
/// permutation-only equality.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WeylElement {
    /// Forward permutation in `[..count]`, its inverse in `[count..]`.
    data: Box<[RootId]>,
    /// The ambient root count: the provenance gate.
    count: usize,
    length: usize,
}

impl WeylElement {
    pub fn identity(system: &RootSystem) -> Result<Self, StructureError> {
        let count = system.roots().len();
        let data = build_data(count, |permutation, inverse| {
            for index in 0..count {
                permutation[index] = RootId(index);
                inverse[index] = RootId(index);
            }
            Ok(())
        })?;
        Ok(Self {
            data,
            count,
            length: 0,
        })
    }

    pub fn simple_reflection(
        system: &RootSystem,
        generator: usize,
    ) -> Result<Self, StructureError> {
        // A simple reflection is an involution of length exactly 1, so the
        // cached permutation IS its own inverse and no length scan is
        // needed; the matrix path stays with `from_action`'s general
        // callers. The range error matches `WeylAction::simple_reflection`.
        let cached = reflection_permutation(system, generator)?;
        let count = cached.len();
        let data = build_data(count, |permutation, inverse| {
            permutation.copy_from_slice(cached);
            inverse.copy_from_slice(cached);
            Ok(())
        })?;
        Ok(Self {
            data,
            count,
            length: 1,
        })
    }

    /// Build the element realizing a matrix-level action, making the two
    /// Weyl layers mutually checkable.
    pub fn from_action(system: &RootSystem, action: &WeylAction) -> Result<Self, StructureError> {
        Self::from_permutation(system, system.action_permutation(action)?)
    }

    /// The single entry point establishing the inverse (with a free
    /// bijectivity check) and the positivity-counted length. `pub(crate)`
    /// for the involution table's cross-edge BFS, which composes the
    /// neighbor permutation directly instead of paying two temporary
    /// `multiply` products per edge.
    pub(crate) fn from_permutation(
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
        let data = build_data(count, |perm_buf, inv_buf| {
            perm_buf.copy_from_slice(&permutation);
            for slot in inv_buf.iter_mut() {
                *slot = RootId(UNSET);
            }
            for (index, image) in permutation.iter().enumerate() {
                let slot = inv_buf.get_mut(image.0).ok_or(
                    StructureError::WeylElementInvariantViolation {
                        invariant: "permutation range",
                    },
                )?;
                if slot.0 != UNSET {
                    return Err(StructureError::WeylElementInvariantViolation {
                        invariant: "permutation bijectivity",
                    });
                }
                *slot = RootId(index);
            }
            Ok(())
        })?;
        let length = count_length(system, &data[..count]);
        Ok(Self {
            data,
            count,
            length,
        })
    }

    /// Build `left after middle after right` directly in the element's final
    /// flat storage. This avoids the intermediate composed permutation used
    /// by the involution-table cross-edge miss path.
    pub(crate) fn from_composition(
        system: &RootSystem,
        left: &[RootId],
        middle: &[RootId],
        right: &[RootId],
    ) -> Result<Self, StructureError> {
        let count = system.roots().len();
        if left.len() != count || middle.len() != count || right.len() != count {
            return Err(StructureError::WeylElementInvariantViolation {
                invariant: "provenance",
            });
        }
        const UNSET: usize = usize::MAX;
        let data = build_data(count, |permutation, inverse| {
            inverse.fill(RootId(UNSET));
            for index in 0..count {
                let right_image = right.get(index).filter(|image| image.0 < count).ok_or(
                    StructureError::WeylElementInvariantViolation {
                        invariant: "permutation range",
                    },
                )?;
                let middle_image = middle
                    .get(right_image.0)
                    .filter(|image| image.0 < count)
                    .ok_or(StructureError::WeylElementInvariantViolation {
                        invariant: "permutation range",
                    })?;
                let image = *left
                    .get(middle_image.0)
                    .filter(|image| image.0 < count)
                    .ok_or(StructureError::WeylElementInvariantViolation {
                        invariant: "permutation range",
                    })?;
                permutation[index] = image;
                let inverse_slot = &mut inverse[image.0];
                if inverse_slot.0 != UNSET {
                    return Err(StructureError::WeylElementInvariantViolation {
                        invariant: "permutation bijectivity",
                    });
                }
                *inverse_slot = RootId(index);
            }
            Ok(())
        })?;
        let length = count_length(system, &data[..count]);
        Ok(Self {
            data,
            count,
            length,
        })
    }

    /// Build `left after w after right` when only the twisted root action
    /// `theta = w after delta` is stored. Since `delta` is involutive,
    /// `w(x) = theta(delta(x))`; composition therefore writes
    /// `left[theta[delta[right[i]]]]` directly into final storage.
    ///
    /// Test-only: production cross-edge transport writes the fused index
    /// order directly (involution_table.rs `push_record`).
    #[cfg(test)]
    pub(crate) fn from_twisted_composition(
        system: &RootSystem,
        left: &[RootId],
        theta: &[RootId],
        delta: &[RootId],
        right: &[RootId],
    ) -> Result<Self, StructureError> {
        let count = system.roots().len();
        if left.len() != count
            || theta.len() != count
            || delta.len() != count
            || right.len() != count
        {
            return Err(StructureError::WeylElementInvariantViolation {
                invariant: "provenance",
            });
        }
        const UNSET: usize = usize::MAX;
        let data = build_data(count, |permutation, inverse| {
            inverse.fill(RootId(UNSET));
            for index in 0..count {
                let right_image = right[index].0;
                let delta_image = delta
                    .get(right_image)
                    .filter(|image| image.0 < count)
                    .ok_or(StructureError::WeylElementInvariantViolation {
                        invariant: "permutation range",
                    })?;
                let theta_image = theta
                    .get(delta_image.0)
                    .filter(|image| image.0 < count)
                    .ok_or(StructureError::WeylElementInvariantViolation {
                        invariant: "permutation range",
                    })?;
                let image = *left
                    .get(theta_image.0)
                    .filter(|image| image.0 < count)
                    .ok_or(StructureError::WeylElementInvariantViolation {
                        invariant: "permutation range",
                    })?;
                permutation[index] = image;
                let inverse_slot = &mut inverse[image.0];
                if inverse_slot.0 != UNSET {
                    return Err(StructureError::WeylElementInvariantViolation {
                        invariant: "permutation bijectivity",
                    });
                }
                *inverse_slot = RootId(index);
            }
            Ok(())
        })?;
        let length = count_length(system, &data[..count]);
        Ok(Self {
            data,
            count,
            length,
        })
    }

    pub fn length(&self) -> usize {
        self.length
    }

    pub fn is_identity(&self) -> bool {
        self.permutation_slice()
            .iter()
            .enumerate()
            .all(|(index, image)| image.0 == index)
    }

    pub fn image(&self, root: RootId) -> Option<RootId> {
        self.permutation_slice().get(root.0).copied()
    }

    pub fn image_permutation(&self) -> &[RootId] {
        self.permutation_slice()
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
        Ok(!system.positivity()[self.inverse_slice()[alpha.0].0])
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
        Ok(!system.positivity()[self.permutation_slice()[alpha.0].0])
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
        let count = self.count;
        let left_permutation = self.permutation_slice();
        let left_inverse = self.inverse_slice();
        let right_permutation = right.permutation_slice();
        let right_inverse = right.inverse_slice();
        let data = build_data(count, |permutation, inverse| {
            for index in 0..count {
                permutation[index] = left_permutation[right_permutation[index].0];
                inverse[index] = right_inverse[left_inverse[index].0];
            }
            Ok(())
        })?;
        let length = count_length(system, &data[..count]);
        Ok(Self {
            data,
            count,
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
        let reflection = reflection_permutation(system, generator)?;
        self.check_provenance(system)?;
        // l(s w) = l(w) ± 1, signed by the left descent: no reflection
        // matrix, no length recount (the descent read is O(1)).
        let change: isize = if self.has_left_descent(system, generator)? {
            -1
        } else {
            1
        };
        let count = self.count;
        let current_permutation = self.permutation_slice();
        let current_inverse = self.inverse_slice();
        let data = build_data(count, |permutation, inverse| {
            for index in 0..count {
                permutation[index] = reflection[current_permutation[index].0];
                // (s w)^{-1} = w^{-1} s, and the reflection is an involution.
                inverse[index] = current_inverse[reflection[index].0];
            }
            Ok(())
        })?;
        let length = if change < 0 {
            self.length - 1
        } else {
            self.length + 1
        };
        Ok((
            Self {
                data,
                count,
                length,
            },
            change,
        ))
    }

    /// `self * s` with its length change, mirroring
    /// [`Self::left_multiply_simple`] for `mult_sigma`/`mult_sigma_inv`.
    pub fn right_multiply_simple(
        &self,
        system: &RootSystem,
        generator: usize,
    ) -> Result<(Self, isize), StructureError> {
        let reflection = reflection_permutation(system, generator)?;
        self.check_provenance(system)?;
        let change: isize = if self.has_right_descent(system, generator)? {
            -1
        } else {
            1
        };
        let count = self.count;
        let current_permutation = self.permutation_slice();
        let current_inverse = self.inverse_slice();
        let data = build_data(count, |permutation, inverse| {
            for index in 0..count {
                permutation[index] = current_permutation[reflection[index].0];
                // (w s)^{-1} = s w^{-1}, and the reflection is an involution.
                inverse[index] = reflection[current_inverse[index].0];
            }
            Ok(())
        })?;
        let length = if change < 0 {
            self.length - 1
        } else {
            self.length + 1
        };
        Ok((
            Self {
                data,
                count,
                length,
            },
            change,
        ))
    }

    /// `s * self` for a KNOWN left descent, given the simple reflection's
    /// root permutation: no length recount (a left descent drops the length
    /// by exactly one). For hot peeling loops such as
    /// [`ParabolicPieces::key`].
    fn left_descend(&self, reflection: &[RootId]) -> Self {
        debug_assert!(self.length > 0);
        let count = self.count;
        let current_permutation = self.permutation_slice();
        let current_inverse = self.inverse_slice();
        // Infallible fill; an allocation failure aborts through `expect`,
        // as the pre-flatbuffer `Vec::with_capacity` construction did.
        let data = build_data(count, |permutation, inverse| {
            for index in 0..count {
                permutation[index] = reflection[current_permutation[index].0];
                // (s w)^{-1} = w^{-1} s.
                inverse[index] = current_inverse[reflection[index].0];
            }
            Ok(())
        })
        .expect("left_descend fill is infallible");
        Self {
            data,
            count,
            length: self.length - 1,
        }
    }

    pub fn inverse(&self) -> Self {
        let count = self.count;
        let current_permutation = self.permutation_slice();
        let current_inverse = self.inverse_slice();
        let data = build_data(count, |permutation, inverse| {
            permutation.copy_from_slice(current_inverse);
            inverse.copy_from_slice(current_permutation);
            Ok(())
        })
        .expect("inverse fill is infallible");
        Self {
            data,
            count,
            length: self.length,
        }
    }

    /// A reduced word by lowest-left-descent peeling, composing
    /// left-to-right: `w = s_{word[0]} * s_{word[1]} * ...`.
    ///
    /// Peels in place on one scratch buffer: the forward permutation
    /// composes per-slot and the inverse goes through a scratch third, so
    /// no per-letter element (or heap pair, pre-flatbuffer) is constructed.
    pub fn reduced_word(&self, system: &RootSystem) -> Result<Vec<usize>, StructureError> {
        self.check_provenance(system)?;
        let mut word = try_capacity(self.length)?;
        let count = self.count;
        let mut buffers = PeelBuffers::new(count, self.permutation_slice(), self.inverse_slice())?;
        let simple_ids = system.simple_root_ids();
        let positivity = system.positivity();
        for _ in 0..self.length {
            // The lowest left descent: `w^{-1}(alpha_s) < 0` reads the
            // inverse. A non-identity element always has one; the error is
            // the same dead branch the pre-flatbuffer loop carried.
            let mut generator = None;
            for candidate in 0..simple_ids.len() {
                if !positivity[buffers.inverse()[simple_ids[candidate].0].0] {
                    generator = Some(candidate);
                    break;
                }
            }
            let Some(generator) = generator else {
                return Err(StructureError::WeylElementInvariantViolation {
                    invariant: "descent peeling",
                });
            };
            // In range by the scan bound, as in `left_multiply_simple`.
            let reflection = reflection_permutation(system, generator)?;
            buffers.peel(count, reflection);
            word.push(generator);
        }
        if !buffers.is_identity(count) {
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
    ///
    /// Composed in a single pass against the cached simple-reflection
    /// permutations — one length recount, no intermediate elements (the
    /// pre-flatbuffer form paid two reflection elements and two general
    /// `multiply` recounts). The error sequence is unchanged: provenance,
    /// twist permutation, the `twist` range read, then the generator range
    /// read (via the reflection cache, as `simple_reflection` did).
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
        let left = reflection_permutation(system, generator)?;
        let right = reflection_permutation(system, twisted)?;
        let count = self.count;
        let current_permutation = self.permutation_slice();
        let current_inverse = self.inverse_slice();
        let data = build_data(count, |permutation, inverse| {
            for index in 0..count {
                // (s_g w s_t)(i) = s_g(w(s_t(i))).
                permutation[index] = left[current_permutation[right[index].0].0];
                // (s_g w s_t)^{-1} = s_t w^{-1} s_g, the reflections being
                // involutions.
                inverse[index] = right[current_inverse[left[index].0].0];
            }
            Ok(())
        })?;
        let length = count_length(system, &data[..count]);
        Ok(Self {
            data,
            count,
            length,
        })
    }

    /// The only provenance gate expressible at this layer: root-count
    /// agreement. Same-cardinality foreign systems are undetectable here.
    fn check_provenance(&self, system: &RootSystem) -> Result<(), StructureError> {
        if self.count != system.roots().len() {
            return Err(StructureError::WeylElementInvariantViolation {
                invariant: "provenance",
            });
        }
        Ok(())
    }

    /// The live forward-permutation half.
    fn permutation_slice(&self) -> &[RootId] {
        &self.data[..self.count]
    }

    /// The live inverse-permutation half.
    fn inverse_slice(&self) -> &[RootId] {
        &self.data[self.count..]
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
        let count = self.count;
        let mut buffers = PeelBuffers::new(count, self.permutation_slice(), self.inverse_slice())?;
        let simple_ids = system.simple_root_ids();
        let positivity = system.positivity();
        while !buffers.is_identity(count) {
            let mut peeled = false;
            for &generator in &interface.outward {
                // `outward` is a permutation of `0..rank` by construction,
                // so this index cannot leave range.
                if !positivity[buffers.inverse()[simple_ids[generator].0].0] {
                    let reflection = reflection_permutation(system, generator)?;
                    buffers.peel(count, reflection);
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

/// Scratch for in-place word peeling, one flat buffer of `3 * count`
/// slots: the forward permutation (`[..count]`) composes in place
/// (`out[i] = reflection[perm[i]]` touches only slot `i`), while the
/// inverse (`[count..2*count]`, `out[i] = inverse[reflection[i]]` reads
/// arbitrary slots) goes through the scratch third (`[2*count..]`).
struct PeelBuffers {
    data: Vec<RootId>,
    count: usize,
}

impl PeelBuffers {
    fn new(
        count: usize,
        permutation: &[RootId],
        inverse: &[RootId],
    ) -> Result<Self, StructureError> {
        let mut data = try_capacity(
            count
                .checked_add(count)
                .and_then(|double| double.checked_add(count))
                .ok_or(StructureError::ArithmeticOverflow)?,
        )?;
        data.extend_from_slice(permutation);
        data.extend_from_slice(inverse);
        data.resize(3 * count, RootId(0));
        Ok(Self { data, count })
    }

    /// The live inverse third (the left-descent read).
    fn inverse(&self) -> &[RootId] {
        &self.data[self.count..2 * self.count]
    }

    /// Whether the live forward permutation is the identity.
    fn is_identity(&self, count: usize) -> bool {
        self.data[..count]
            .iter()
            .enumerate()
            .all(|(index, image)| image.0 == index)
    }

    /// `reflection * current` in place, one left-descent step.
    fn peel(&mut self, count: usize, reflection: &[RootId]) {
        let (permutation, rest) = self.data.split_at_mut(count);
        let (inverse, scratch) = rest.split_at_mut(count);
        for slot in permutation.iter_mut() {
            *slot = reflection[slot.0];
        }
        for index in 0..count {
            scratch[index] = inverse[reflection[index].0];
        }
        inverse.copy_from_slice(scratch);
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
/// of [`WeylElement::canonical_word`], and the internal-order piece
/// indexing of [`ParabolicPieces`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WeylInterface {
    outward: Vec<usize>,
    /// Per internal index, the first internal index of its Dynkin
    /// component — upstream `comp.offset` of the owning component.
    component_offset: Vec<usize>,
}

impl WeylInterface {
    pub fn new(cartan: &[Vec<i32>]) -> Result<Self, StructureError> {
        let permutation_violation = |_| StructureError::WeylElementInvariantViolation {
            invariant: "interface permutation",
        };
        let components = crate::dynkin::classify(cartan)?;
        let mut outward = try_capacity(cartan.len())?;
        outward.resize(cartan.len(), usize::MAX);
        let mut component_offset = try_capacity(cartan.len())?;
        component_offset.resize(cartan.len(), usize::MAX);
        let mut offset = 0;
        for component in &components {
            let size = component.position.len();
            let reverse = matches!(component.letter, 'B' | 'C' | 'D');
            for slot in component_offset.iter_mut().take(offset + size).skip(offset) {
                *slot = offset;
            }
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
        Ok(Self {
            outward,
            component_offset,
        })
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

/// The system's cached simple-reflection permutation, with the range
/// error the matrix path produced (`WeylAction::simple_reflection`).
fn reflection_permutation(
    system: &RootSystem,
    generator: usize,
) -> Result<&[RootId], StructureError> {
    system
        .simple_reflection_permutation(generator)
        .ok_or(StructureError::IndexOutOfRange {
            index: generator,
            upper_bound: system.datum().semisimple_rank(),
        })
}

/// The per-level `EltPiece` indexing of the upstream transducer
/// (`WeylGroup::Transducer::Transducer`, weyl.cpp:289-416): for each
/// internal generator `i`, the minimal coset representatives of
/// `W_{i-1}\W_i` enumerated in the transducer's creation order — BFS from
/// the identity piece, right-multiplying by the Dynkin component's
/// internal generators in ascending order and keeping only products that
/// stay coset-minimal (no left descents below the level).
///
/// The piece list of an element is the tie-break of the upstream
/// involution ordering: `Cartan_orbits::comparer`
/// (involutions.cpp:420-428) compares `WeylElt::pieces` arrays, upstream's
/// `WeylElt::operator<` (weyl.h:133). Building the table once per KGB
/// construction keeps [`Self::key`] lookups out of the transducer
/// enumeration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParabolicPieces {
    /// Per internal level: minimal coset representative -> piece index.
    levels: Vec<BTreeMap<WeylElement, usize>>,
    /// Per EXTERNAL generator: the simple reflection's root permutation.
    /// Cached once at build so [`Self::key`]'s descent peeling never
    /// materializes a reflection matrix.
    reflections: Vec<Vec<RootId>>,
}

impl ParabolicPieces {
    pub fn build(system: &RootSystem, interface: &WeylInterface) -> Result<Self, StructureError> {
        let rank = system.simple_root_ids().len();
        if interface.outward.len() != rank {
            return Err(StructureError::WeylElementInvariantViolation {
                invariant: "interface provenance",
            });
        }
        let mut reflections = try_capacity(rank)?;
        for generator in 0..rank {
            let action = WeylAction::simple_reflection(system.datum(), generator)?;
            reflections.push(system.action_permutation(&action)?);
        }
        let mut levels = try_capacity(rank)?;
        for level in 0..rank {
            let offset = interface.component_offset[level];
            let identity = WeylElement::identity(system)?;
            let mut reps = Vec::new();
            let mut index = BTreeMap::new();
            reps.push(identity.clone());
            index.insert(identity, 0_usize);
            let mut cursor = 0;
            while cursor < reps.len() {
                let current = reps[cursor].clone();
                for internal in offset..=level {
                    let (candidate, _) =
                        current.right_multiply_simple(system, interface.outward[internal])?;
                    let mut coset_minimal = true;
                    for lower in offset..level {
                        if candidate.has_left_descent(system, interface.outward[lower])? {
                            coset_minimal = false;
                            break;
                        }
                    }
                    if coset_minimal && !index.contains_key(&candidate) {
                        index.insert(candidate.clone(), reps.len());
                        reps.push(candidate);
                    }
                }
                cursor += 1;
            }
            levels.push(index);
        }
        Ok(Self {
            levels,
            reflections,
        })
    }

    /// The element's piece list in internal-level order: the unique
    /// factorization `w = w_1...w_n` with `w_i` the minimal representative
    /// of its right coset `W_{i-1}.w`, returned as the per-level piece
    /// indices. Lexicographic comparison of these lists is upstream's
    /// `WeylElt::operator<`.
    pub fn key(
        &self,
        system: &RootSystem,
        interface: &WeylInterface,
        element: &WeylElement,
    ) -> Result<Vec<usize>, StructureError> {
        element.check_provenance(system)?;
        let rank = system.simple_root_ids().len();
        if interface.outward.len() != rank || self.levels.len() != rank {
            return Err(StructureError::WeylElementInvariantViolation {
                invariant: "interface provenance",
            });
        }
        let mut pieces = try_capacity(rank)?;
        pieces.resize(rank, 0_usize);
        let mut tail = element.clone();
        for level in (0..rank).rev() {
            // Peel the coset `W_{level}.tail` down to its minimal
            // representative: any remaining left descent below the level
            // shortens the element inside the same coset.
            let mut minimal = tail.clone();
            loop {
                let mut descended = false;
                for internal in 0..level {
                    let generator = interface.outward[internal];
                    if minimal.has_left_descent(system, generator)? {
                        // A left descent shortens by exactly one: descend via
                        // the cached reflection permutation, no recount.
                        minimal = minimal.left_descend(&self.reflections[generator]);
                        descended = true;
                        break;
                    }
                }
                if !descended {
                    break;
                }
            }
            pieces[level] = *self.levels[level].get(&minimal).ok_or(
                StructureError::WeylElementInvariantViolation {
                    invariant: "parabolic piece",
                },
            )?;
            tail = tail.multiply(system, &minimal.inverse())?;
        }
        if !tail.is_identity() {
            return Err(StructureError::WeylElementInvariantViolation {
                invariant: "parabolic factorization",
            });
        }
        Ok(pieces)
    }

    /// Allocation-free form of [`Self::key`] for Atlas's rank-at-most-eight
    /// compact Weyl representation. Trailing entries are zero, so ordinary
    /// array comparison is the same lexicographic order as the rank-sized
    /// legacy vector within one root datum.
    pub(crate) fn fixed_key(
        &self,
        system: &RootSystem,
        interface: &WeylInterface,
        element: &WeylElement,
    ) -> Result<[u16; 8], StructureError> {
        element.check_provenance(system)?;
        let rank = system.simple_root_ids().len();
        if rank > 8 || interface.outward.len() != rank || self.levels.len() != rank {
            return Err(StructureError::WeylElementInvariantViolation {
                invariant: "interface provenance",
            });
        }
        let mut pieces = [0_u16; 8];
        let mut tail = element.clone();
        for level in (0..rank).rev() {
            let mut minimal = tail.clone();
            loop {
                let mut descended = false;
                for internal in 0..level {
                    let generator = interface.outward[internal];
                    if minimal.has_left_descent(system, generator)? {
                        minimal = minimal.left_descend(&self.reflections[generator]);
                        descended = true;
                        break;
                    }
                }
                if !descended {
                    break;
                }
            }
            let piece = *self.levels[level].get(&minimal).ok_or(
                StructureError::WeylElementInvariantViolation {
                    invariant: "parabolic piece",
                },
            )?;
            pieces[level] = u16::try_from(piece).map_err(|_| StructureError::ArithmeticOverflow)?;
            tail = tail.multiply(system, &minimal.inverse())?;
        }
        if !tail.is_identity() {
            return Err(StructureError::WeylElementInvariantViolation {
                invariant: "parabolic factorization",
            });
        }
        Ok(pieces)
    }
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
    fn fused_composition_matches_materialized_composition() {
        let system = b2();
        let left = WeylElement::simple_reflection(&system, 0).unwrap();
        let middle = WeylElement::simple_reflection(&system, 1).unwrap();
        let right = WeylElement::simple_reflection(&system, 0).unwrap();
        let expected = left
            .multiply(&system, &middle)
            .unwrap()
            .multiply(&system, &right)
            .unwrap();

        let actual = WeylElement::from_composition(
            &system,
            left.image_permutation(),
            middle.image_permutation(),
            right.image_permutation(),
        )
        .unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn twisted_factor_composition_matches_explicit_weyl_factor() {
        let system = b2();
        let left = WeylElement::simple_reflection(&system, 0).unwrap();
        let middle = WeylElement::simple_reflection(&system, 1).unwrap();
        let right = WeylElement::simple_reflection(&system, 0).unwrap();
        let delta: Vec<RootId> = (0..system.roots().len()).map(RootId).collect();
        let expected = WeylElement::from_composition(
            &system,
            left.image_permutation(),
            middle.image_permutation(),
            right.image_permutation(),
        )
        .unwrap();

        let actual = WeylElement::from_twisted_composition(
            &system,
            left.image_permutation(),
            middle.image_permutation(),
            &delta,
            right.image_permutation(),
        )
        .unwrap();

        assert_eq!(actual, expected);
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

    #[test]
    fn parabolic_pieces_match_the_oracle_b2_anchors() {
        // B2's internal order is reversed: internal t_0 = datum s_1,
        // t_1 = datum s_0. The level-1 minimal coset reps enumerate as
        // [e, t_1, t_1 t_0, t_1 t_0 t_1] (weyl.cpp:320-405), so the piece
        // lists of the (involution length, Weyl length) ties are:
        let b2 = b2();
        let interface = WeylInterface::new(b2.datum().cartan_matrix()).unwrap();
        let pieces = ParabolicPieces::build(&b2, &interface).unwrap();
        let key = |word: &[usize]| pieces.key(&b2, &interface, &from_word(&b2, word)).unwrap();
        assert_eq!(key(&[]), [0, 0]);
        // s_0 = t_1 factors (e, t_1); s_1 = t_0 factors (t_0, e).
        assert_eq!(key(&[0]), [0, 1]);
        assert_eq!(key(&[1]), [1, 0]);
        // s_0 s_1 s_0 = t_1 t_0 t_1 vs s_1 s_0 s_1 = t_0 (t_1 t_0).
        assert_eq!(key(&[0, 1, 0]), [0, 3]);
        assert_eq!(key(&[1, 0, 1]), [1, 2]);
        assert_eq!(key(&[0, 1, 0, 1]), [1, 3]);
        assert_eq!(key(&[1, 0, 1, 0]), [1, 3]);
        // The upstream involution ordering therefore places the s_0 packet
        // before the s_1 packet and the s_0 s_1 s_0 packet before the
        // s_1 s_0 s_1 packet — the frozen B2 full-KGB probe's numbering.
        assert!(key(&[0]) < key(&[1]));
        assert!(key(&[0, 1, 0]) < key(&[1, 0, 1]));
    }

    #[test]
    fn parabolic_pieces_preserve_the_a1_a2_ordering() {
        // A1 has a single generator: no equal-(length, Weyl length) ties
        // exist, so no numbering can change.
        let a1 = enumerate(vec![vec![2]], 2);
        let a1_interface = WeylInterface::new(a1.datum().cartan_matrix()).unwrap();
        let a1_pieces = ParabolicPieces::build(&a1, &a1_interface).unwrap();
        assert_eq!(
            a1_pieces
                .key(&a1, &a1_interface, &from_word(&a1, &[0]))
                .unwrap(),
            [1]
        );
        // A2's internal order is straight; the (1,1) tie compares
        // s_1 = [0,1] before s_0 = [1,0] — the SAME order the derived
        // root-permutation Ord already gave, so A2 numbering is
        // invariant under the tie-break switch.
        let a2 = a2();
        let interface = WeylInterface::new(a2.datum().cartan_matrix()).unwrap();
        let pieces = ParabolicPieces::build(&a2, &interface).unwrap();
        let s0 = from_word(&a2, &[0]);
        let s1 = from_word(&a2, &[1]);
        assert_eq!(pieces.key(&a2, &interface, &s1).unwrap(), [0, 1]);
        assert_eq!(pieces.key(&a2, &interface, &s0).unwrap(), [1, 0]);
        assert!(s1 < s0, "derived Ord already placed s_1 before s_0");
    }

    #[test]
    fn parabolic_pieces_key_is_a_total_order_on_group_elements() {
        let g2 = enumerate(vec![vec![2, -3], vec![-1, 2]], 12);
        let c2 = enumerate(vec![vec![2, -1], vec![-2, 2]], 8);
        let a3 = enumerate(vec![vec![2, -1, 0], vec![-1, 2, -1], vec![0, -1, 2]], 12);
        for system in [a2(), b2(), c2, g2, a3] {
            let interface = WeylInterface::new(system.datum().cartan_matrix()).unwrap();
            let pieces = ParabolicPieces::build(&system, &interface).unwrap();
            let mut keys = Vec::new();
            for element in closure(&system) {
                keys.push(pieces.key(&system, &interface, &element).unwrap());
            }
            let mut sorted = keys.clone();
            sorted.sort_unstable();
            sorted.dedup();
            assert_eq!(keys.len(), sorted.len(), "piece lists collide");
        }
        // The rank-zero edge: one empty piece list.
        let torus = BasedRootDatum::from_simple_data(2, vec![], vec![], vec![]).unwrap();
        let torus_system = RootSystem::enumerate(&torus, 0).unwrap();
        let torus_interface = WeylInterface::new(&[]).unwrap();
        let torus_pieces = ParabolicPieces::build(&torus_system, &torus_interface).unwrap();
        assert_eq!(
            torus_pieces
                .key(
                    &torus_system,
                    &torus_interface,
                    &WeylElement::identity(&torus_system).unwrap(),
                )
                .unwrap(),
            Vec::<usize>::new()
        );
    }

    #[test]
    fn fixed_parabolic_key_preserves_values_and_order() {
        for system in [a2(), b2()] {
            let interface = WeylInterface::new(system.datum().cartan_matrix()).unwrap();
            let pieces = ParabolicPieces::build(&system, &interface).unwrap();
            let elements = closure(&system);
            let legacy: Vec<Vec<usize>> = elements
                .iter()
                .map(|element| pieces.key(&system, &interface, element).unwrap())
                .collect();
            let fixed: Vec<[u16; 8]> = elements
                .iter()
                .map(|element| pieces.fixed_key(&system, &interface, element).unwrap())
                .collect();
            let rank = system.simple_root_ids().len();

            for index in 0..elements.len() {
                assert_eq!(
                    fixed[index][..rank],
                    legacy[index]
                        .iter()
                        .map(|&piece| u16::try_from(piece).unwrap())
                        .collect::<Vec<_>>()
                );
                for other in 0..elements.len() {
                    assert_eq!(
                        fixed[index].cmp(&fixed[other]),
                        legacy[index].cmp(&legacy[other])
                    );
                }
            }
        }
    }
}
