//! Read-only block topology used by the KL implementation.
//!
//! Upstream `klsupport::KLSupport` consumes a `const Block_base&`; it does
//! not depend on the concrete classic or common-block representation.  This
//! trait is the corresponding Rust boundary.  Implementations return `None`
//! for an invalid element/generator, while an undefined Cayley link is
//! represented by the inner `None` values of the returned pair.

use std::collections::BTreeSet;
use std::sync::Arc;

use crate::{BlockDescent, BlockGraph, PartialBlock};

/// Private sealing boundary. The visibility permits crate-local invariant
/// tests without allowing downstream crates to implement `BlockTopology`.
pub(crate) mod sealed {
    pub trait Sealed {}
}

/// The minimal, read-only block surface required by `KlSupport` and
/// `KlTable`.
///
/// Implementations are sealed because the KL algorithm relies on structural
/// invariants beyond individual method signatures: `rank() <= 32`, elements
/// are ordered by nondecreasing length, every element/generator cell exists,
/// and every defined cross or Cayley target is smaller than `size()`. The KL
/// construction validates these invariants before recursion begins.
pub trait BlockTopology: sealed::Sealed {
    fn size(&self) -> usize;
    fn rank(&self) -> usize;
    fn length(&self, element: usize) -> Option<usize>;
    fn descent(&self, element: usize, generator: usize) -> Option<BlockDescent>;
    fn cross(&self, element: usize, generator: usize) -> Option<usize>;
    fn cayley(&self, element: usize, generator: usize) -> Option<(Option<usize>, Option<usize>)>;
    fn inverse_cayley(
        &self,
        element: usize,
        generator: usize,
    ) -> Option<(Option<usize>, Option<usize>)>;
}

/// The Bruhat Hasse diagram of a block topology (blocks.cpp:1576-1656).
/// Each row lists the immediate down-neighbours of the corresponding block
/// element. Elements must be in nondecreasing length order, as required by
/// [`BlockTopology`].
pub fn bruhat_hasse<B: BlockTopology + ?Sized>(block: &B) -> Vec<Vec<usize>> {
    let size = block.size();
    let rank = block.rank();
    let mut hasse: Vec<Vec<usize>> = Vec::with_capacity(size);
    for z in 0..size {
        let mut covered = BTreeSet::new();
        let strict_good = (0..rank).find(|&s| {
            matches!(
                block.descent(z, s),
                Some(BlockDescent::ComplexDescent | BlockDescent::RealTypeI)
            )
        });
        match strict_good {
            Some(s) => match block.descent(z, s) {
                Some(BlockDescent::ComplexDescent) => {
                    let sz = block.cross(z, s).expect("complex descent cross");
                    covered.insert(sz);
                    insert_ascents(block, &hasse[sz], s, &mut covered);
                }
                Some(BlockDescent::RealTypeI) => {
                    let (first, second) =
                        block.inverse_cayley(z, s).expect("type I inverse Cayley");
                    let first = first.expect("type I first image");
                    covered.insert(first);
                    if let Some(second) = second {
                        covered.insert(second);
                    }
                    insert_ascents(block, &hasse[first], s, &mut covered);
                }
                _ => unreachable!("strict good descent match"),
            },
            None => {
                for s in 0..rank {
                    if block.descent(z, s) == Some(BlockDescent::RealTypeII) {
                        if let Some(first) = block
                            .inverse_cayley(z, s)
                            .expect("type II inverse Cayley")
                            .0
                        {
                            covered.insert(first);
                        }
                    }
                }
            }
        }
        hasse.push(covered.into_iter().collect());
    }
    hasse
}

fn insert_ascents<B: BlockTopology + ?Sized>(
    block: &B,
    hr: &[usize],
    s: usize,
    hs: &mut BTreeSet<usize>,
) {
    for &z in hr {
        match block.descent(z, s) {
            Some(BlockDescent::ComplexAscent) => {
                if let Some(c) = block.cross(z, s) {
                    hs.insert(c);
                }
            }
            Some(BlockDescent::ImaginaryTypeI) => {
                if let Some(c) = block.cayley(z, s).expect("type I Cayley").0 {
                    hs.insert(c);
                }
            }
            Some(BlockDescent::ImaginaryTypeII) => {
                if let Some((a, b)) = block.cayley(z, s) {
                    if let Some(a) = a {
                        hs.insert(a);
                    }
                    if let Some(b) = b {
                        hs.insert(b);
                    }
                }
            }
            _ => {}
        }
    }
}

impl sealed::Sealed for BlockGraph {}

impl BlockTopology for BlockGraph {
    fn size(&self) -> usize {
        BlockGraph::size(self)
    }

    fn rank(&self) -> usize {
        BlockGraph::rank(self)
    }

    fn length(&self, element: usize) -> Option<usize> {
        BlockGraph::length(self, element)
    }

    fn descent(&self, element: usize, generator: usize) -> Option<BlockDescent> {
        BlockGraph::descent_value(self, element, generator)
    }

    fn cross(&self, element: usize, generator: usize) -> Option<usize> {
        BlockGraph::cross(self, element, generator)
    }

    fn cayley(&self, element: usize, generator: usize) -> Option<(Option<usize>, Option<usize>)> {
        BlockGraph::cayley(self, element, generator)
    }

    fn inverse_cayley(
        &self,
        element: usize,
        generator: usize,
    ) -> Option<(Option<usize>, Option<usize>)> {
        BlockGraph::inverse_cayley(self, element, generator)
    }
}

impl sealed::Sealed for PartialBlock {}

impl BlockTopology for PartialBlock {
    fn size(&self) -> usize {
        PartialBlock::size(self)
    }

    fn rank(&self) -> usize {
        PartialBlock::rank(self)
    }

    fn length(&self, element: usize) -> Option<usize> {
        PartialBlock::length(self, element)
    }

    fn descent(&self, element: usize, generator: usize) -> Option<BlockDescent> {
        PartialBlock::descent(self, element, generator)
    }

    fn cross(&self, element: usize, generator: usize) -> Option<usize> {
        PartialBlock::cross(self, generator, element)
    }

    fn cayley(&self, element: usize, generator: usize) -> Option<(Option<usize>, Option<usize>)> {
        if PartialBlock::descent(self, element, generator)?.is_descent() {
            return Some((None, None));
        }
        PartialBlock::cayley(self, generator, element)
    }

    fn inverse_cayley(
        &self,
        element: usize,
        generator: usize,
    ) -> Option<(Option<usize>, Option<usize>)> {
        if !PartialBlock::descent(self, element, generator)?.is_descent() {
            return Some((None, None));
        }
        PartialBlock::cayley(self, generator, element)
    }
}

impl<T: BlockTopology + ?Sized> sealed::Sealed for &T {}

impl<T: BlockTopology + ?Sized> BlockTopology for &T {
    fn size(&self) -> usize {
        T::size(self)
    }

    fn rank(&self) -> usize {
        T::rank(self)
    }

    fn length(&self, element: usize) -> Option<usize> {
        T::length(self, element)
    }

    fn descent(&self, element: usize, generator: usize) -> Option<BlockDescent> {
        T::descent(self, element, generator)
    }

    fn cross(&self, element: usize, generator: usize) -> Option<usize> {
        T::cross(self, element, generator)
    }

    fn cayley(&self, element: usize, generator: usize) -> Option<(Option<usize>, Option<usize>)> {
        T::cayley(self, element, generator)
    }

    fn inverse_cayley(
        &self,
        element: usize,
        generator: usize,
    ) -> Option<(Option<usize>, Option<usize>)> {
        T::inverse_cayley(self, element, generator)
    }
}

impl<T: BlockTopology + ?Sized> sealed::Sealed for Arc<T> {}

impl<T: BlockTopology + ?Sized> BlockTopology for Arc<T> {
    fn size(&self) -> usize {
        T::size(self)
    }

    fn rank(&self) -> usize {
        T::rank(self)
    }

    fn length(&self, element: usize) -> Option<usize> {
        T::length(self, element)
    }

    fn descent(&self, element: usize, generator: usize) -> Option<BlockDescent> {
        T::descent(self, element, generator)
    }

    fn cross(&self, element: usize, generator: usize) -> Option<usize> {
        T::cross(self, element, generator)
    }

    fn cayley(&self, element: usize, generator: usize) -> Option<(Option<usize>, Option<usize>)> {
        T::cayley(self, element, generator)
    }

    fn inverse_cayley(
        &self,
        element: usize,
        generator: usize,
    ) -> Option<(Option<usize>, Option<usize>)> {
        T::inverse_cayley(self, element, generator)
    }
}
