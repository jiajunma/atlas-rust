//! Per-block KL support data (gkmod/klsupport.{h,cpp}).
//!
//! `KlSupport` precomputes, for every block element, its descent set
//! ("tau-invariant") and its good-ascent set (klsupport.cpp:63-89), the
//! length-stop table (klsupport.cpp:46-62), and the primitive-index
//! tables that map a block element to its position in the list of
//! primitive elements for a given descent set (klsupport.cpp:109-144).
//!
//! The primitive notion is central to the KLV computation: `x` is
//! *primitive* for the descent set of `y` when no good ascent of `x` is
//! a descent of `y` (klsupport.h `is_primitive`). The KLV polynomial
//! `P_{x,y}` is stored at `prim_index(x, desc(y))` in column `y`
//! (kl.cpp:124-148).

use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};

use crate::block::BlockDescent;
use crate::{BlockTopology, StructureError};

/// Multiplicative hasher for the descent-set mask keys of the
/// primitive-index table. The keys are single `u32`s and the table is
/// consulted on every `kl_pol` call, where SipHash's rounds are pure
/// overhead.
#[derive(Default)]
struct MaskHasher(u64);

impl Hasher for MaskHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.0 = (self.0 ^ u64::from(byte)).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        }
    }

    fn write_u32(&mut self, value: u32) {
        self.0 = u64::from(value).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    }
}

/// A bitset over the simple generators, matching the `RankFlags` semantics
/// used by the KL algorithm (upstream `RankFlags`). The rank is ≤32.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RankFlags {
    bits: u32,
}

impl RankFlags {
    pub fn empty() -> Self {
        Self { bits: 0 }
    }

    pub fn set(&mut self, generator: usize) {
        self.bits |= 1 << generator;
    }

    pub fn contains(&self, other: &Self) -> bool {
        self.bits & other.bits == other.bits
    }

    /// `self & other` — generators in both.
    pub fn intersect(&self, other: &Self) -> Self {
        Self {
            bits: self.bits & other.bits,
        }
    }

    /// `self - other` — generators in self but not other.
    pub fn difference(&self, other: &Self) -> Self {
        Self {
            bits: self.bits & !other.bits,
        }
    }

    pub fn none(&self) -> bool {
        self.bits == 0
    }

    /// The first set generator (upstream `RankFlags::firstBit`), or `None`.
    pub fn first_bit(&self) -> Option<usize> {
        (0..32).find(|&generator| self.bits & (1 << generator) != 0)
    }

    pub fn is_set(&self, generator: usize) -> bool {
        self.bits & (1 << generator) != 0
    }
}

/// Per-block-element KL support data, mirroring `klsupport::KLSupport`
/// (klsupport.h).
pub struct KlSupport<B: BlockTopology> {
    block: B,
    descents: Vec<RankFlags>,
    good_ascents: Vec<RankFlags>,
    length_stop: Vec<usize>,
    /// Maps a descent-set mask to its slot in `prim_index`, so the hot
    /// recursion loops can resolve the record once per column fill.
    prim_slots: HashMap<u32, u32, BuildHasherDefault<MaskHasher>>,
    /// The primitive-index records, one per prepared descent set; slot
    /// numbers are stable for the life of the support.
    prim_index: Vec<PrimIndexRecord>,
}

/// The primitive-index record for one descent set (klsupport.h
/// `prim_index_tp`): `index[z]` is the position of `z`'s primitivisation
/// among the primitive elements for the descent set, or `range` when `z`
/// has no primitive element for it; `range` is the number of primitives.
#[derive(Clone, Debug)]
struct PrimIndexRecord {
    index: Vec<usize>,
    range: usize,
}

impl<B: BlockTopology> KlSupport<B> {
    pub fn new(block: B) -> Result<Self, StructureError> {
        validate_topology(&block)?;
        let rank = block.rank();
        let size = block.size();
        let mut descents = Vec::with_capacity(size);
        let mut good_ascents = Vec::with_capacity(size);
        for z in 0..size {
            let mut desc = RankFlags::empty();
            let mut good_asc = RankFlags::empty();
            for s in 0..rank {
                let value = block.descent(z, s).ok_or(StructureError::IndexOutOfRange {
                    index: z * rank + s,
                    upper_bound: size * rank,
                })?;
                if value.is_descent() {
                    desc.set(s);
                } else if value != BlockDescent::ImaginaryTypeII {
                    good_asc.set(s);
                }
            }
            descents.push(desc);
            good_ascents.push(good_asc);
        }

        // length_stop[l] = first element of length >= l, or size when none
        // (klsupport.cpp:46-62).
        let max_length = (0..size)
            .map(|z| block.length(z).unwrap_or(0))
            .max()
            .unwrap_or(0);
        let mut length_stop = Vec::new();
        for z in 0..size {
            let length = block.length(z).ok_or(StructureError::IndexOutOfRange {
                index: z,
                upper_bound: size,
            })?;
            while length_stop.len() <= length {
                length_stop.push(z);
            }
        }
        length_stop.push(size);

        let _ = max_length;
        Ok(Self {
            block,
            descents,
            good_ascents,
            length_stop,
            prim_slots: HashMap::default(),
            prim_index: Vec::new(),
        })
    }

    pub fn block(&self) -> &B {
        &self.block
    }

    pub fn size(&self) -> usize {
        self.block.size()
    }

    pub fn rank(&self) -> usize {
        self.block.rank()
    }

    pub fn length(&self, z: usize) -> usize {
        self.block.length(z).unwrap_or(0)
    }

    /// The number of block elements of length < `l` (klsupport.h
    /// `length_less`).
    pub fn length_less(&self, l: usize) -> usize {
        self.length_stop.get(l).copied().unwrap_or(self.size())
    }

    /// The first block element of length < `length(y)` (klsupport.h
    /// `length_floor`).
    pub fn length_floor(&self, y: usize) -> usize {
        self.length_less(self.length(y))
    }

    pub fn descent_set(&self, z: usize) -> &RankFlags {
        &self.descents[z]
    }

    pub fn good_ascent_set(&self, z: usize) -> &RankFlags {
        &self.good_ascents[z]
    }

    /// The first ascent of `x` that is a descent of `y` (klsupport.h
    /// `ascent_descent`), or `None`.
    pub fn ascent_descent(&self, x: usize, y: usize) -> Option<usize> {
        self.descent_set(y)
            .difference(self.descent_set(x))
            .first_bit()
    }

    /// Whether `x` is extremal for descent set `desc_y`: every descent of
    /// `y` is also a descent of `x` (klsupport.h `is_extremal`).
    pub fn is_extremal(&self, x: usize, desc_y: &RankFlags) -> bool {
        self.descent_set(x).contains(desc_y)
    }

    /// Whether `x` is primitive for descent set `desc_y`: no good ascent
    /// of `x` is a descent of `y` (klsupport.h `is_primitive`).
    pub fn is_primitive(&self, x: usize, desc_y: &RankFlags) -> bool {
        self.good_ascent_set(x).intersect(desc_y).none()
    }

    /// Walk `x` (inclusive) down to the previous primitive element for
    /// `desc_y`, returning `false` when none is found (klsupport.h
    /// `prim_back_up`).
    pub fn prim_back_up(&self, x: &mut usize, desc_y: &RankFlags) -> bool {
        while *x > 0 {
            *x -= 1;
            if self.is_primitive(*x, desc_y) {
                return true;
            }
        }
        false
    }

    /// The number of primitive elements for `descent_set(y)` of length
    /// less than `y` (klsupport.h `col_size`).
    pub fn col_size(&self, y: usize) -> usize {
        let mut x = self.length_floor(y);
        let desc_y = self.descent_set(y);
        if self.prim_back_up(&mut x, desc_y) {
            self.prim_index(x, desc_y) + 1
        } else {
            0
        }
    }

    /// The unique ascent image of `z` through `s` (blocks.h
    /// `unique_ascent`): the cross image for a complex ascent, the first
    /// Cayley image for an imaginary type I ascent.
    pub fn unique_ascent(&self, s: usize, z: usize) -> Option<usize> {
        let value = self.block.descent(z, s)?;
        match value {
            BlockDescent::ComplexAscent => self.block.cross(z, s),
            BlockDescent::ImaginaryTypeI => self.block.cayley(z, s)?.0,
            _ => None,
        }
    }

    /// The slot of the primitive-index record for `desc_y`. Resolving it
    /// once per column fill lets the recursion loops use `prim_index_at`
    /// without repeating the mask lookup. `prepare_prim_index` must be
    /// called for the descent set first.
    pub fn prim_slot(&self, desc_y: &RankFlags) -> u32 {
        *self
            .prim_slots
            .get(&desc_y.bits)
            .expect("prepared primitive index")
    }

    /// `prim_index` through an already-resolved slot: two `Vec` indexes,
    /// no hashing (klsupport.h `prim_index` on a prepared descent set).
    pub fn prim_index_at(&self, slot: u32, x: usize) -> usize {
        self.prim_index[slot as usize].index[x]
    }

    /// The whole primitive-index row for an already-resolved slot, for
    /// per-column loops that borrow the row once instead of re-resolving
    /// the record per query.
    pub fn prim_index_row(&self, slot: u32) -> &[usize] {
        &self.prim_index[slot as usize].index
    }

    /// The position of `x`'s primitivisation among the primitive elements
    /// for `desc_y`, or `range` when `x` has no primitive element for it
    /// (klsupport.h `prim_index`). `prepare_prim_index` must be called for
    /// the descent set first.
    pub fn prim_index(&self, x: usize, desc_y: &RankFlags) -> usize {
        self.prim_index_at(self.prim_slot(desc_y), x)
    }

    /// The number of primitive elements for `desc_y` (klsupport.h
    /// `nr_of_primitives`).
    pub fn nr_of_primitives(&self, desc_y: &RankFlags) -> usize {
        self.nr_of_primitives_at(self.prim_slot(desc_y))
    }

    /// `nr_of_primitives` through an already-resolved slot.
    pub fn nr_of_primitives_at(&self, slot: u32) -> usize {
        self.prim_index[slot as usize].range
    }

    /// The position of `y` in its own primitive row (klsupport.h
    /// `self_index`).
    pub fn self_index(&self, y: usize) -> usize {
        self.prim_index(y, self.descent_set(y))
    }

    /// Prepare the primitive-index table for a descent set, if not already
    /// present (klsupport.cpp:109-144).
    pub fn prepare_prim_index(&mut self, desc_y: &RankFlags) {
        if self.prim_slots.contains_key(&desc_y.bits) {
            return;
        }
        let size = self.size();
        let mut index = vec![0_usize; size];
        let mut count = 0_usize;
        const DEAD_END: usize = usize::MAX;
        for x in (0..size).rev() {
            let good = self.good_ascent_set(x).intersect(desc_y);
            if good.none() {
                // x is primitive: record its index (count of larger
                // primitives seen so far).
                index[x] = count;
                count += 1;
                continue;
            }
            let s = good.first_bit().expect("nonempty good ascent");
            let value = self.block.descent(x, s).expect("valid generator");
            if value == BlockDescent::RealNonparity {
                index[x] = DEAD_END;
            } else {
                let ascent = self.unique_ascent(s, x);
                index[x] = match ascent {
                    Some(ascended) => index[ascended],
                    None => DEAD_END,
                };
            }
        }
        let range = count;
        for slot in index.iter_mut() {
            if *slot == DEAD_END {
                *slot = range;
            } else {
                *slot = range - 1 - *slot; // reverse indices
            }
        }
        let slot = self.prim_index.len() as u32;
        self.prim_index.push(PrimIndexRecord { index, range });
        self.prim_slots.insert(desc_y.bits, slot);
    }
}

/// Validate every invariant that the KL recursion later treats as trusted.
/// This keeps malformed topology at the fallible construction boundary
/// instead of allowing an indexing or `expect` panic deep in a column fill.
fn validate_topology(block: &impl BlockTopology) -> Result<(), StructureError> {
    const RANK_FLAGS_CAPACITY: usize = u32::BITS as usize;

    if block.rank() > RANK_FLAGS_CAPACITY {
        return Err(StructureError::BlockInvariantViolation {
            invariant: "KL topology rank exceeds RankFlags capacity",
        });
    }

    let size = block.size();
    let mut previous_length = None;
    for element in 0..size {
        let length = block
            .length(element)
            .ok_or(StructureError::BlockInvariantViolation {
                invariant: "KL topology element has no length",
            })?;
        if previous_length.is_some_and(|previous| length < previous) {
            return Err(StructureError::BlockInvariantViolation {
                invariant: "KL topology lengths are not nondecreasing",
            });
        }
        previous_length = Some(length);

        for generator in 0..block.rank() {
            block
                .descent(element, generator)
                .ok_or(StructureError::BlockInvariantViolation {
                    invariant: "KL topology element has no descent status",
                })?;
            validate_target(block.cross(element, generator), size)?;
            let cayley = block.cayley(element, generator).ok_or(
                StructureError::BlockInvariantViolation {
                    invariant: "KL topology element has no Cayley cell",
                },
            )?;
            validate_target(cayley.0, size)?;
            validate_target(cayley.1, size)?;
            let inverse = block.inverse_cayley(element, generator).ok_or(
                StructureError::BlockInvariantViolation {
                    invariant: "KL topology element has no inverse-Cayley cell",
                },
            )?;
            validate_target(inverse.0, size)?;
            validate_target(inverse.1, size)?;
        }
    }
    Ok(())
}

fn validate_target(target: Option<usize>, size: usize) -> Result<(), StructureError> {
    if target.is_some_and(|target| target >= size) {
        return Err(StructureError::BlockInvariantViolation {
            invariant: "KL topology link target is outside the block",
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeTopology {
        rank: usize,
        lengths: Vec<usize>,
        cross_target: usize,
    }

    impl crate::block_access::sealed::Sealed for FakeTopology {}

    impl crate::BlockTopology for FakeTopology {
        fn size(&self) -> usize {
            self.lengths.len()
        }

        fn rank(&self) -> usize {
            self.rank
        }

        fn length(&self, element: usize) -> Option<usize> {
            self.lengths.get(element).copied()
        }

        fn descent(&self, element: usize, generator: usize) -> Option<BlockDescent> {
            (element < self.size() && generator < self.rank).then_some(BlockDescent::ComplexAscent)
        }

        fn cross(&self, element: usize, generator: usize) -> Option<usize> {
            (element < self.size() && generator < self.rank).then_some(self.cross_target)
        }

        fn cayley(
            &self,
            element: usize,
            generator: usize,
        ) -> Option<(Option<usize>, Option<usize>)> {
            (element < self.size() && generator < self.rank).then_some((None, None))
        }

        fn inverse_cayley(
            &self,
            element: usize,
            generator: usize,
        ) -> Option<(Option<usize>, Option<usize>)> {
            (element < self.size() && generator < self.rank).then_some((None, None))
        }
    }

    #[test]
    fn rank_flags_set_and_query() {
        let mut flags = RankFlags::empty();
        assert!(flags.none());
        flags.set(2);
        assert!(flags.is_set(2));
        assert_eq!(flags.first_bit(), Some(2));
        let other = RankFlags::empty();
        assert!(flags.contains(&other)); // the empty set is contained
    }

    #[test]
    fn rejects_topology_rank_above_rank_flags_capacity() {
        let result = KlSupport::new(FakeTopology {
            rank: 33,
            lengths: vec![0],
            cross_target: 0,
        });
        assert!(matches!(
            result,
            Err(StructureError::BlockInvariantViolation {
                invariant: "KL topology rank exceeds RankFlags capacity"
            })
        ));
    }

    #[test]
    fn rejects_topology_not_sorted_by_length() {
        let result = KlSupport::new(FakeTopology {
            rank: 1,
            lengths: vec![1, 0],
            cross_target: 0,
        });
        assert!(matches!(
            result,
            Err(StructureError::BlockInvariantViolation {
                invariant: "KL topology lengths are not nondecreasing"
            })
        ));
    }

    #[test]
    fn rejects_topology_link_target_outside_the_block() {
        let result = KlSupport::new(FakeTopology {
            rank: 1,
            lengths: vec![0],
            cross_target: 1,
        });
        assert!(matches!(
            result,
            Err(StructureError::BlockInvariantViolation {
                invariant: "KL topology link target is outside the block"
            })
        ));
    }

    /// A four-element chain: generator 0 is a complex ascent chaining
    /// 0 -> 1 -> 2 -> 3 and an imaginary compact descent at 3; generator
    /// 1 is a complex descent everywhere. Element 3 is primitive for
    /// every descent set.
    struct ChainTopology;

    impl crate::block_access::sealed::Sealed for ChainTopology {}

    impl crate::BlockTopology for ChainTopology {
        fn size(&self) -> usize {
            4
        }

        fn rank(&self) -> usize {
            2
        }

        fn length(&self, element: usize) -> Option<usize> {
            [0, 1, 1, 2].get(element).copied()
        }

        fn descent(&self, element: usize, generator: usize) -> Option<BlockDescent> {
            if element >= self.size() || generator >= self.rank() {
                return None;
            }
            Some(match generator {
                0 if element == 3 => BlockDescent::ImaginaryCompact,
                0 => BlockDescent::ComplexAscent,
                _ => BlockDescent::ComplexDescent,
            })
        }

        fn cross(&self, element: usize, generator: usize) -> Option<usize> {
            if element >= self.size() || generator >= self.rank() {
                return None;
            }
            Some(if generator == 0 && element < 3 {
                element + 1
            } else {
                0
            })
        }

        fn cayley(
            &self,
            element: usize,
            generator: usize,
        ) -> Option<(Option<usize>, Option<usize>)> {
            (element < self.size() && generator < self.rank()).then_some((None, None))
        }

        fn inverse_cayley(
            &self,
            element: usize,
            generator: usize,
        ) -> Option<(Option<usize>, Option<usize>)> {
            (element < self.size() && generator < self.rank()).then_some((None, None))
        }
    }

    #[test]
    fn prim_slot_accessors_match_prim_index() {
        let mut support = KlSupport::new(ChainTopology).unwrap();
        let mut only_zero = RankFlags::empty();
        only_zero.set(0);
        let mut only_one = RankFlags::empty();
        only_one.set(1);
        let sets = [RankFlags::empty(), only_zero, only_one];
        for set in &sets {
            support.prepare_prim_index(set);
        }

        // Each descent set gets its own slot, and a repeated prepare is
        // idempotent (same slot, no duplicate record).
        let mut slots: Vec<u32> = sets.iter().map(|set| support.prim_slot(set)).collect();
        slots.sort_unstable();
        slots.dedup();
        assert_eq!(slots.len(), sets.len(), "each descent set gets a slot");
        for set in &sets {
            let slot = support.prim_slot(set);
            support.prepare_prim_index(set);
            assert_eq!(support.prim_slot(set), slot, "idempotent prepare");
        }

        // The slot accessors agree with the by-mask accessor everywhere.
        for set in &sets {
            let slot = support.prim_slot(set);
            for x in 0..support.size() {
                assert_eq!(support.prim_index_at(slot, x), support.prim_index(x, set));
            }
        }

        // Generator 1 is a descent of every element, so all four elements
        // are primitive for {1} and the index is the identity; for {0}
        // only element 3 is primitive and every x primitivises to it.
        let one_slot = support.prim_slot(&sets[2]);
        for x in 0..support.size() {
            assert_eq!(support.prim_index_at(one_slot, x), x);
        }
        assert_eq!(support.nr_of_primitives(&sets[2]), 4);
        assert_eq!(support.nr_of_primitives(&sets[1]), 1);
        let zero_slot = support.prim_slot(&sets[1]);
        for x in 0..support.size() {
            assert_eq!(support.prim_index_at(zero_slot, x), 0);
        }
    }
}
