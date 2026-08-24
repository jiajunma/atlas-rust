//! Word-level Weyl elements on the root-permutation representation.
//!
//! This is stage (a) of the KGB map: the combinatorial substrate the Tits
//! sigma formulas consume — O(1) length and descent queries, multiplication,
//! inverses, twisted conjugation, and on-demand reduced words. Elements are
//! the construction currency of the involution table and Tits stages, not of
//! persistent KGB elements, so no budget knob exists at this layer.
//!
//! An element is represented by its permutation of the enumerated roots of
//! one ambient [`RootSystem`], stored in fixed stack arrays for every system
//! the upstream transducer covers (at most E8's 240 roots — the fixed-size
//! `WeylElt` discipline of weyl.h:60-80) with a heap fallback for larger
//! closures, so multiplication on the inline tiers never allocates. The only
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

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};

use crate::grading::try_capacity;
use crate::{RootId, RootSystem, StructureError, WeylAction};

/// Root-count ceiling of the small inline tier: every system through
/// B4/C4 (32 roots), covering the low-rank braid/word-heavy scripts whose
/// per-letter operations would otherwise pay a 240-entry memset per step.
const SMALL_ROOTS: usize = 32;

/// Root-count ceiling of the inline tier: E8's 240 roots, so every
/// semisimple system of rank <= 8 multiplies without touching the heap.
const MAX_INLINE_ROOTS: usize = 240;

/// Permutation storage: fixed stack arrays below the inline ceilings, heap
/// vectors beyond. The array tail past the live prefix is zero-filled and
/// never read; equality, hashing, and ordering see only the prefix.
#[derive(Clone, Debug)]
enum Repr {
    Small {
        permutation: [RootId; SMALL_ROOTS],
        inverse: [RootId; SMALL_ROOTS],
    },
    Inline {
        permutation: [RootId; MAX_INLINE_ROOTS],
        inverse: [RootId; MAX_INLINE_ROOTS],
    },
    Heap {
        permutation: Vec<RootId>,
        inverse: Vec<RootId>,
    },
}

impl Repr {
    /// The live permutation prefix (the element's forward root action).
    fn permutation(&self, count: usize) -> &[RootId] {
        match self {
            Repr::Small { permutation, .. } => &permutation[..count],
            Repr::Inline { permutation, .. } => &permutation[..count],
            Repr::Heap { permutation, .. } => permutation,
        }
    }

    /// The live inverse-permutation prefix.
    fn inverse(&self, count: usize) -> &[RootId] {
        match self {
            Repr::Small { inverse, .. } => &inverse[..count],
            Repr::Inline { inverse, .. } => &inverse[..count],
            Repr::Heap { inverse, .. } => inverse,
        }
    }
}

/// The single representation-construction site: `fill` establishes the
/// live prefix of both buffers. The heap tier keeps the `try_capacity`
/// allocation-failure gate of the pre-inline implementation; the inline
/// tiers cannot fail to allocate.
fn build_repr(
    count: usize,
    fill: impl FnOnce(&mut [RootId], &mut [RootId]) -> Result<(), StructureError>,
) -> Result<Repr, StructureError> {
    if count <= SMALL_ROOTS {
        let mut permutation = [RootId(0); SMALL_ROOTS];
        let mut inverse = [RootId(0); SMALL_ROOTS];
        fill(&mut permutation[..count], &mut inverse[..count])?;
        Ok(Repr::Small {
            permutation,
            inverse,
        })
    } else if count <= MAX_INLINE_ROOTS {
        let mut permutation = [RootId(0); MAX_INLINE_ROOTS];
        let mut inverse = [RootId(0); MAX_INLINE_ROOTS];
        fill(&mut permutation[..count], &mut inverse[..count])?;
        Ok(Repr::Inline {
            permutation,
            inverse,
        })
    } else {
        let mut permutation = try_capacity(count)?;
        permutation.resize(count, RootId(0));
        let mut inverse = try_capacity(count)?;
        inverse.resize(count, RootId(0));
        fill(&mut permutation, &mut inverse)?;
        Ok(Repr::Heap {
            permutation,
            inverse,
        })
    }
}

/// A Weyl-group element as a permutation of the enumerated roots.
///
/// Equality, hashing, and ordering are the live permutation prefix — the
/// pre-inline derived-trait contract preserved by hand (slice ordering IS
/// the old `Vec` lexicographic order, prefix rule included), so the unused
/// array tail can never leak into comparisons. `inverse` and `length` are
/// functions of the permutation established by every constructor, hence
/// prefix equality implies full equality.
#[derive(Clone)]
pub struct WeylElement {
    repr: Repr,
    /// Live prefix length, i.e. the ambient root count: the provenance gate.
    count: usize,
    length: usize,
}

impl PartialEq for WeylElement {
    fn eq(&self, other: &Self) -> bool {
        self.permutation_slice() == other.permutation_slice()
    }
}

impl Eq for WeylElement {}

impl Hash for WeylElement {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.permutation_slice().hash(state);
    }
}

impl PartialOrd for WeylElement {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for WeylElement {
    fn cmp(&self, other: &Self) -> Ordering {
        self.permutation_slice().cmp(other.permutation_slice())
    }
}

impl std::fmt::Debug for WeylElement {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WeylElement")
            .field("permutation", &self.permutation_slice())
            .field("length", &self.length)
            .finish()
    }
}

impl WeylElement {
    pub fn identity(system: &RootSystem) -> Result<Self, StructureError> {
        let count = system.roots().len();
        let repr = build_repr(count, |permutation, inverse| {
            for index in 0..count {
                permutation[index] = RootId(index);
                inverse[index] = RootId(index);
            }
            Ok(())
        })?;
        Ok(Self {
            repr,
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
        let repr = build_repr(count, |permutation, inverse| {
            permutation.copy_from_slice(cached);
            inverse.copy_from_slice(cached);
            Ok(())
        })?;
        Ok(Self {
            repr,
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
        let repr = build_repr(count, |perm_buf, inv_buf| {
            perm_buf.copy_from_slice(&permutation);
            for slot in inv_buf.iter_mut() {
                *slot = RootId(UNSET);
            }
            for (index, image) in permutation.iter().enumerate() {
                let slot =
                    inv_buf
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
            Ok(())
        })?;
        let length = count_length(system, repr.permutation(count));
        Ok(Self {
            repr,
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
        let repr = build_repr(count, |permutation, inverse| {
            for index in 0..count {
                permutation[index] = left_permutation[right_permutation[index].0];
                inverse[index] = right_inverse[left_inverse[index].0];
            }
            Ok(())
        })?;
        let length = count_length(system, repr.permutation(count));
        Ok(Self {
            repr,
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
        let repr = build_repr(count, |permutation, inverse| {
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
                repr,
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
        let repr = build_repr(count, |permutation, inverse| {
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
                repr,
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
        // Infallible fill; the heap tier's allocation failure aborts, as the
        // pre-inline `Vec::with_capacity` construction did.
        let repr = build_repr(count, |permutation, inverse| {
            for index in 0..count {
                permutation[index] = reflection[current_permutation[index].0];
                // (s w)^{-1} = w^{-1} s.
                inverse[index] = current_inverse[reflection[index].0];
            }
            Ok(())
        })
        .expect("left_descend fill is infallible");
        Self {
            repr,
            count,
            length: self.length - 1,
        }
    }

    pub fn inverse(&self) -> Self {
        let count = self.count;
        let current_permutation = self.permutation_slice();
        let current_inverse = self.inverse_slice();
        let repr = build_repr(count, |permutation, inverse| {
            permutation.copy_from_slice(current_inverse);
            inverse.copy_from_slice(current_permutation);
            Ok(())
        })
        .expect("inverse fill is infallible");
        Self {
            repr,
            count,
            length: self.length,
        }
    }

    /// A reduced word by lowest-left-descent peeling, composing
    /// left-to-right: `w = s_{word[0]} * s_{word[1]} * ...`.
    ///
    /// Peels in place on scratch buffers: the forward permutation composes
    /// per-slot and the inverse goes through a double buffer, so no
    /// per-letter element (or heap pair, pre-inline) is constructed.
    pub fn reduced_word(&self, system: &RootSystem) -> Result<Vec<usize>, StructureError> {
        self.check_provenance(system)?;
        let mut word = try_capacity(self.length)?;
        let count = self.count;
        let mut buffers = PeelBuffers::new(count, self.permutation_slice(), self.inverse_slice());
        let simple_ids = system.simple_root_ids();
        let positivity = system.positivity();
        for _ in 0..self.length {
            // The lowest left descent: `w^{-1}(alpha_s) < 0` reads the
            // inverse. A non-identity element always has one; the error is
            // the same dead branch the pre-inline loop carried.
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
    /// pre-inline form paid two reflection elements and two general
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
        let repr = build_repr(count, |permutation, inverse| {
            for index in 0..count {
                // (s_g w s_t)(i) = s_g(w(s_t(i))).
                permutation[index] = left[current_permutation[right[index].0].0];
                // (s_g w s_t)^{-1} = s_t w^{-1} s_g, the reflections being
                // involutions.
                inverse[index] = right[current_inverse[left[index].0].0];
            }
            Ok(())
        })?;
        let length = count_length(system, repr.permutation(count));
        Ok(Self {
            repr,
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

    /// The live permutation prefix.
    fn permutation_slice(&self) -> &[RootId] {
        self.repr.permutation(self.count)
    }

    /// The live inverse-permutation prefix.
    fn inverse_slice(&self) -> &[RootId] {
        self.repr.inverse(self.count)
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
        let mut buffers = PeelBuffers::new(count, self.permutation_slice(), self.inverse_slice());
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

/// Scratch buffers for in-place word peeling: the forward permutation
/// composes in place (`out[i] = reflection[perm[i]]` touches only slot
/// `i`), while the inverse (`out[i] = inverse[reflection[i]]` reads
/// arbitrary slots) goes through the scratch double buffer.
enum PeelBuffers {
    Small {
        permutation: [RootId; SMALL_ROOTS],
        inverse: [RootId; SMALL_ROOTS],
        scratch: [RootId; SMALL_ROOTS],
    },
    Inline {
        permutation: [RootId; MAX_INLINE_ROOTS],
        inverse: [RootId; MAX_INLINE_ROOTS],
        scratch: [RootId; MAX_INLINE_ROOTS],
    },
    Heap {
        permutation: Vec<RootId>,
        inverse: Vec<RootId>,
        scratch: Vec<RootId>,
    },
}

impl PeelBuffers {
    fn new(count: usize, permutation: &[RootId], inverse: &[RootId]) -> Self {
        if count <= SMALL_ROOTS {
            let mut permutation_buf = [RootId(0); SMALL_ROOTS];
            permutation_buf[..count].copy_from_slice(permutation);
            let mut inverse_buf = [RootId(0); SMALL_ROOTS];
            inverse_buf[..count].copy_from_slice(inverse);
            Self::Small {
                permutation: permutation_buf,
                inverse: inverse_buf,
                scratch: [RootId(0); SMALL_ROOTS],
            }
        } else if count <= MAX_INLINE_ROOTS {
            let mut permutation_buf = [RootId(0); MAX_INLINE_ROOTS];
            permutation_buf[..count].copy_from_slice(permutation);
            let mut inverse_buf = [RootId(0); MAX_INLINE_ROOTS];
            inverse_buf[..count].copy_from_slice(inverse);
            Self::Inline {
                permutation: permutation_buf,
                inverse: inverse_buf,
                scratch: [RootId(0); MAX_INLINE_ROOTS],
            }
        } else {
            Self::Heap {
                permutation: permutation.to_vec(),
                inverse: inverse.to_vec(),
                scratch: inverse.to_vec(),
            }
        }
    }

    /// The live inverse prefix (the left-descent read).
    fn inverse(&self) -> &[RootId] {
        match self {
            Self::Small { inverse, .. } => inverse,
            Self::Inline { inverse, .. } => inverse,
            Self::Heap { inverse, .. } => inverse,
        }
    }

    /// Whether the live permutation prefix is the identity.
    fn is_identity(&self, count: usize) -> bool {
        let permutation: &[RootId] = match self {
            Self::Small { permutation, .. } => permutation,
            Self::Inline { permutation, .. } => permutation,
            Self::Heap { permutation, .. } => permutation,
        };
        permutation[..count]
            .iter()
            .enumerate()
            .all(|(index, image)| image.0 == index)
    }

    /// `reflection * current` in place, one left-descent step.
    fn peel(&mut self, count: usize, reflection: &[RootId]) {
        let (permutation, inverse, scratch): (&mut [RootId], &mut [RootId], &mut [RootId]) =
            match self {
                Self::Small {
                    permutation,
                    inverse,
                    scratch,
                } => (permutation, inverse, scratch),
                Self::Inline {
                    permutation,
                    inverse,
                    scratch,
                } => (permutation, inverse, scratch),
                Self::Heap {
                    permutation,
                    inverse,
                    scratch,
                } => (permutation, inverse, scratch),
            };
        for slot in permutation[..count].iter_mut() {
            *slot = reflection[slot.0];
        }
        for index in 0..count {
            scratch[index] = inverse[reflection[index].0];
        }
        inverse[..count].copy_from_slice(&scratch[..count]);
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
        Ok(Self { levels, reflections })
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
}
