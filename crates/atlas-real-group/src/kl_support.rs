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

use std::collections::BTreeMap;

use crate::block::{BlockDescent, BlockGraph};
use crate::StructureError;

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
pub struct KlSupport<'a> {
    block: &'a BlockGraph,
    descents: Vec<RankFlags>,
    good_ascents: Vec<RankFlags>,
    length_stop: Vec<usize>,
    /// Maps a descent-set mask to the primitive-index record.
    prim_index: BTreeMap<u32, PrimIndexRecord>,
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

impl<'a> KlSupport<'a> {
    pub fn new(block: &'a BlockGraph) -> Result<Self, StructureError> {
        let rank = block.rank();
        let size = block.size();
        let mut descents = Vec::with_capacity(size);
        let mut good_ascents = Vec::with_capacity(size);
        for z in 0..size {
            let mut desc = RankFlags::empty();
            let mut good_asc = RankFlags::empty();
            for s in 0..rank {
                let value = block
                    .descent_value(z, s)
                    .ok_or(StructureError::IndexOutOfRange {
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
            prim_index: BTreeMap::new(),
        })
    }

    pub fn block(&self) -> &'a BlockGraph {
        self.block
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
        let value = self.block.descent_value(z, s)?;
        match value {
            BlockDescent::ComplexAscent => self.block.cross(z, s),
            BlockDescent::ImaginaryTypeI => self.block.cayley(z, s)?.0,
            _ => None,
        }
    }

    /// The position of `x`'s primitivisation among the primitive elements
    /// for `desc_y`, or `range` when `x` has no primitive element for it
    /// (klsupport.h `prim_index`). `prepare_prim_index` must be called for
    /// the descent set first.
    pub fn prim_index(&self, x: usize, desc_y: &RankFlags) -> usize {
        let record = &self.prim_index[&desc_y.bits];
        record.index[x]
    }

    /// The number of primitive elements for `desc_y` (klsupport.h
    /// `nr_of_primitives`).
    pub fn nr_of_primitives(&self, desc_y: &RankFlags) -> usize {
        self.prim_index[&desc_y.bits].range
    }

    /// The position of `y` in its own primitive row (klsupport.h
    /// `self_index`).
    pub fn self_index(&self, y: usize) -> usize {
        self.prim_index(y, self.descent_set(y))
    }

    /// Prepare the primitive-index table for a descent set, if not already
    /// present (klsupport.cpp:109-144).
    pub fn prepare_prim_index(&mut self, desc_y: &RankFlags) {
        if self.prim_index.contains_key(&desc_y.bits) {
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
            let value = self.block.descent_value(x, s).expect("valid generator");
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
        self.prim_index
            .insert(desc_y.bits, PrimIndexRecord { index, range });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
