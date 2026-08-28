//! The block of a real form and a dual real form (upstream `blocks::Block`).
//!
//! A block is the fibred product of the two one-sided parameter sets: the
//! real form's KGB graph and the dual real form's KGB graph, paired over
//! twisted involutions `w` and `dual_w` (upstream `Block::Block(kgb,
//! dual_kgb)`, gkmod/blocks.cpp:526-606). The pairing map is the
//! twisted-Weyl-group duality [`dual_involution`] (upstream
//! `blocks::dual_involution`, gkmod/blocks.cpp:1701-1711): on involution
//! matrices it is minus-transpose, on Weyl elements it is characterized by
//! `f(e) = w0` and `f(s.w) = f(w).dwist(s)`.
//!
//! The interpreter builds blocks from the FULL KGB sets of the two forms
//! (`Block::build(RealReductiveGroup&, RealReductiveGroup&)`,
//! gkmod/blocks.cpp:622-626, called from `Block_value` at
//! interpreter/atlas-types.w:4753-4758): the numbering of the extracted KGB
//! coordinates must stay the forms' own KGB numbering, which the
//! `common_Cartans`-restricted overload (blocks.cpp:610-619) would change
//! (the comment at atlas-types.w:4739-4745). The restriction to common
//! Cartans is then implicit: a primal involution whose dual is absent from
//! the dual form's KGB contributes an empty packet, exactly like upstream's
//! `KGB_base::tauPacket` returning `(0,0)` there (gkmod/kgb.cpp:131-140).

use std::collections::HashMap;

use crate::grading::try_capacity;
use crate::{
    InnerClass, InvolutionTable, KgbGraph, KgbId, KgbStatus, RootSystem, StructureError,
    WeylElement,
};

/// The per-generator descent status of a block element, in the upstream
/// `descents::DescentStatus::Value` order (gkmod/descents.h:40): the first
/// four values are ascents, the last four (bit `0x4` set) weak descents.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum BlockDescent {
    ComplexAscent,
    RealNonparity,
    ImaginaryTypeI,
    ImaginaryTypeII,
    ImaginaryCompact,
    ComplexDescent,
    RealTypeII,
    RealTypeI,
}

impl BlockDescent {
    const ALL: [BlockDescent; 8] = [
        BlockDescent::ComplexAscent,
        BlockDescent::RealNonparity,
        BlockDescent::ImaginaryTypeI,
        BlockDescent::ImaginaryTypeII,
        BlockDescent::ImaginaryCompact,
        BlockDescent::ComplexDescent,
        BlockDescent::RealTypeII,
        BlockDescent::RealTypeI,
    ];

    fn index(self) -> usize {
        Self::ALL
            .iter()
            .position(|&value| value == self)
            .expect("ALL covers every variant")
    }

    /// Upstream `DescentStatus::isDescent` (descents.h:71): bit `0x4` of the
    /// value index — the weak descents {ImaginaryCompact, ComplexDescent,
    /// RealTypeII, RealTypeI}.
    pub fn is_descent(self) -> bool {
        self.index() & 0x4 != 0
    }

    /// Upstream `DescentStatus::dual` (descents.h:74-79): the status of the
    /// corresponding generator in the dual block — ComplexAscent↔
    /// ComplexDescent, RealNonparity↔ImaginaryCompact, ImaginaryTypeI↔
    /// RealTypeII, ImaginaryTypeII↔RealTypeI.
    pub fn dual(self) -> BlockDescent {
        const DUAL: [BlockDescent; 8] = [
            BlockDescent::ComplexDescent,
            BlockDescent::ImaginaryCompact,
            BlockDescent::RealTypeII,
            BlockDescent::RealTypeI,
            BlockDescent::RealNonparity,
            BlockDescent::ComplexAscent,
            BlockDescent::ImaginaryTypeI,
            BlockDescent::ImaginaryTypeII,
        ];
        DUAL[self.index()]
    }

    /// The Atlas-language status code: the renumbering
    /// `tab = {4,5,6,7,1,0,3,2}` of `block_status_wrapper`
    /// (interpreter/atlas-types.w:4911-4913), i.e. 0=C-, 1=ic, 2=r1, 3=r2,
    /// 4=C+, 5=rn, 6=i1, 7=i2.
    pub fn language_code(self) -> u32 {
        const TAB: [u32; 8] = [4, 5, 6, 7, 1, 0, 3, 2];
        TAB[self.index()]
    }
}

/// The twisted involution dual to `w` (upstream `blocks::dual_involution`,
/// gkmod/blocks.cpp:1701-1711): start from the longest element of the dual
/// Weyl group and right-multiply by the dual-twisted letters of a reduced
/// word of `w` taken right-to-left. `word` carries EXTERNAL generator
/// numbers — the one numbering the primal and dual twisted Weyl groups
/// share; `dual_twist` is the dual inner class's distinguished generator
/// permutation (`TwistedWeylGroup::twisted` on the dual side).
pub fn dual_involution(
    word: &[usize],
    dual_system: &RootSystem,
    dual_twist: &[usize],
    dual_longest: &WeylElement,
) -> Result<WeylElement, StructureError> {
    let mut result = dual_longest.clone();
    for &generator in word.iter().rev() {
        let twisted = *dual_twist
            .get(generator)
            .ok_or(StructureError::IndexOutOfRange {
                index: generator,
                upper_bound: dual_twist.len(),
            })?;
        let (next, _) = result.right_multiply_simple(dual_system, twisted)?;
        result = next;
    }
    Ok(result)
}

/// A block: the fibred product of two KGB graphs over their involution
/// packets, with the block-level cross/Cayley tables and descent statuses.
/// Element numbering reproduces upstream's: packets in the primal KGB's
/// sorted involution order, `x` ascending within a packet, `y` ascending
/// within the paired dual packet (blocks.cpp:548-558).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlockGraph {
    rank: usize,
    /// Primal KGB coordinates per block element (full-KGB numbering).
    xs: Vec<KgbId>,
    /// Dual KGB coordinates per block element.
    ys: Vec<KgbId>,
    /// Flat `z * rank + s`.
    descent: Vec<BlockDescent>,
    /// Flat `s * size + z` (upstream `data[s][z]`).
    cross: Vec<usize>,
    /// Upstream `Cayley_image.first/.second`, shared by the direct Cayley
    /// (ascents) and the inverse Cayley (weak descents) accessors.
    cayley_first: Vec<Option<usize>>,
    cayley_second: Vec<Option<usize>>,
    /// The primal KGB length of each block element (blocks.cpp:557
    /// `kgb.length(x)`), used by the KL-table's length ordering.
    lengths: Vec<usize>,
    /// `first_z_of_x[x]` = the first block element with `x(z) >= x`
    /// (blocks.cpp:630-643); length `xrange + 1` with a size sentinel.
    first_z_of_x: Vec<usize>,
}

impl BlockGraph {
    /// Port of `Block::Block(kgb, dual_kgb)` (gkmod/blocks.cpp:526-606).
    /// `dual_inner_class` supplies the dual twisted Weyl group (its
    /// distinguished twist and longest element); `weyl_budget` bounds the
    /// enumeration locating that longest element. The primal/dual KGB
    /// graphs carry their own tables for the involution words.
    pub fn build(
        graph: &KgbGraph,
        table: &InvolutionTable,
        dual_graph: &KgbGraph,
        dual_table: &InvolutionTable,
        dual_inner_class: &InnerClass,
        weyl_budget: usize,
    ) -> Result<Self, StructureError> {
        let rank = graph.semisimple_rank();
        if dual_graph.semisimple_rank() != rank
            || dual_table.root_system() != dual_inner_class.root_system()
        {
            return Err(StructureError::DatumMismatch);
        }
        let dual_system = dual_table.root_system();
        // dual_tW: the dual distinguished twist and the dual longest element.
        let dual_twist = dual_inner_class.generator_twist()?;
        let longest = crate::dual::longest_action(dual_inner_class, weyl_budget)?;
        let dual_longest = WeylElement::from_action(dual_system, &longest)?;

        // The dual form's packets, keyed by their twisted involution.
        let mut dual_position: HashMap<WeylElement, usize> = HashMap::new();
        for position in 0..dual_graph.packet_count() {
            let id = dual_graph.packet_involution(position).ok_or(
                StructureError::BlockInvariantViolation {
                    invariant: "dual packet involution",
                },
            )?;
            let record = dual_table
                .record(id)
                .ok_or(StructureError::BlockInvariantViolation {
                    invariant: "dual packet record",
                })?;
            dual_position.insert(record.weyl_element().clone(), position);
        }

        // The bijection tW -> dual_tW tabulated over the primal packets, and
        // the fibred-product size (blocks.cpp:535-543): a missing dual packet
        // is upstream's empty `tauPacket`, contributing zero pairs.
        let mut dual_w: Vec<WeylElement> = try_capacity(graph.packet_count())?;
        let mut size = 0_usize;
        for position in 0..graph.packet_count() {
            let id = graph.packet_involution(position).ok_or(
                StructureError::BlockInvariantViolation {
                    invariant: "packet involution",
                },
            )?;
            let _record = table
                .record(id)
                .ok_or(StructureError::BlockInvariantViolation {
                    invariant: "packet record",
                })?;
            let word = table.weyl_word(id)?;
            let dual = dual_involution(&word, dual_system, &dual_twist, &dual_longest)?;
            let (_, x_count) =
                graph
                    .tau_packet(position)
                    .ok_or(StructureError::BlockInvariantViolation {
                        invariant: "tau packet",
                    })?;
            let y_count = match dual_position.get(&dual) {
                Some(&dual_pos) => {
                    dual_graph
                        .tau_packet(dual_pos)
                        .ok_or(StructureError::BlockInvariantViolation {
                            invariant: "dual tau packet",
                        })?
                        .1
                }
                None => 0,
            };
            size = size
                .checked_add(
                    x_count
                        .checked_mul(y_count)
                        .ok_or(StructureError::ArithmeticOverflow)?,
                )
                .ok_or(StructureError::ArithmeticOverflow)?;
            dual_w.push(dual);
        }

        // The fibred product (blocks.cpp:548-558): per primal involution, the
        // Cartesian product of the two packets, x outer, y inner.
        let mut xs: Vec<KgbId> = try_capacity(size)?;
        let mut ys: Vec<KgbId> = try_capacity(size)?;
        let mut descent: Vec<BlockDescent> = try_capacity(size * rank.max(1))?;
        let mut lengths: Vec<usize> = try_capacity(size)?;
        for (position, dual) in dual_w.iter().enumerate() {
            let Some(&dual_pos) = dual_position.get(dual) else {
                continue; // empty dual packet: upstream's (0,0) tauPacket
            };
            let (x_start, x_count) =
                graph
                    .tau_packet(position)
                    .ok_or(StructureError::BlockInvariantViolation {
                        invariant: "tau packet",
                    })?;
            let (y_start, y_count) =
                dual_graph
                    .tau_packet(dual_pos)
                    .ok_or(StructureError::BlockInvariantViolation {
                        invariant: "dual tau packet",
                    })?;
            for x_offset in 0..x_count {
                let x = KgbId(x_start.index() + x_offset);
                for y_offset in 0..y_count {
                    let y = KgbId(y_start.index() + y_offset);
                    xs.push(x);
                    ys.push(y);
                    lengths.push(graph.length(x).ok_or(
                        StructureError::BlockInvariantViolation {
                            invariant: "block element length",
                        },
                    )?);
                    for generator in 0..rank {
                        descent.push(descents(x, y, generator, graph, dual_graph)?);
                    }
                }
            }
        }
        if xs.len() != size {
            return Err(StructureError::BlockInvariantViolation {
                invariant: "block size",
            });
        }

        // compute_first_zs (blocks.cpp:630-643); x values weakly increase.
        let first_z_of_x = first_zs(&xs, graph.size());

        // Cross and Cayley tables (blocks.cpp:565-593).
        let mut cross: Vec<usize> = try_capacity(size * rank.max(1))?;
        cross.resize(size * rank.max(1), 0);
        let mut cayley_first: Vec<Option<usize>> = try_capacity(size * rank.max(1))?;
        cayley_first.resize(size * rank.max(1), None);
        let mut cayley_second: Vec<Option<usize>> = try_capacity(size * rank.max(1))?;
        cayley_second.resize(size * rank.max(1), None);
        for generator in 0..rank {
            for z in 0..size {
                let slot = generator * size + z;
                let (x, y) = (xs[z], ys[z]);
                let cross_x =
                    graph
                        .cross(x, generator)
                        .ok_or(StructureError::BlockInvariantViolation {
                            invariant: "cross link",
                        })?;
                let cross_y = dual_graph.cross(y, generator).ok_or(
                    StructureError::BlockInvariantViolation {
                        invariant: "dual cross link",
                    },
                )?;
                cross[slot] = element_at(&xs, &ys, &first_z_of_x, cross_x, cross_y)?;
                match descent[z * rank + generator] {
                    BlockDescent::ImaginaryTypeII => {
                        let cayley_x = graph.cayley(x, generator)?.ok_or(
                            StructureError::BlockInvariantViolation {
                                invariant: "Cayley link",
                            },
                        )?;
                        let (dual_first, dual_second) = dual_graph
                            .inverse_cayley(y, generator)?
                            .ok_or(StructureError::BlockInvariantViolation {
                                invariant: "dual inverse Cayley link",
                            })?;
                        let dual_second =
                            dual_second.ok_or(StructureError::BlockInvariantViolation {
                                invariant: "type II inverse Cayley pair",
                            })?;
                        // Double-valued direct Cayley (blocks.cpp:575-582).
                        let z1 = element_at(&xs, &ys, &first_z_of_x, cayley_x, dual_second)?;
                        cayley_second[generator * size + z] = Some(z1);
                        cayley_first[generator * size + z1] = Some(z);
                        // FALL THROUGH to the type I branch (blocks.cpp:583-590).
                        let z0 = element_at(&xs, &ys, &first_z_of_x, cayley_x, dual_first)?;
                        cayley_first[generator * size + z] = Some(z0);
                        first_free_slot(
                            &mut cayley_first[generator * size + z0],
                            &mut cayley_second[generator * size + z0],
                            z,
                        )?;
                    }
                    BlockDescent::ImaginaryTypeI => {
                        let cayley_x = graph.cayley(x, generator)?.ok_or(
                            StructureError::BlockInvariantViolation {
                                invariant: "Cayley link",
                            },
                        )?;
                        let (dual_first, _) = dual_graph.inverse_cayley(y, generator)?.ok_or(
                            StructureError::BlockInvariantViolation {
                                invariant: "dual inverse Cayley link",
                            },
                        )?;
                        let z0 = element_at(&xs, &ys, &first_z_of_x, cayley_x, dual_first)?;
                        cayley_first[generator * size + z] = Some(z0);
                        first_free_slot(
                            &mut cayley_first[generator * size + z0],
                            &mut cayley_second[generator * size + z0],
                            z,
                        )?;
                    }
                    _ => {}
                }
            }
        }

        Ok(Self {
            rank,
            xs,
            ys,
            descent,
            cross,
            cayley_first,
            cayley_second,
            lengths,
            first_z_of_x,
        })
    }

    /// Upstream `Bare_block::dual` (gkmod/blocks.cpp:474-509): the block of
    /// the same two real forms with their roles swapped, a pure data
    /// transform. Element order reverses (`z' = size-1-z`), the `x`/`y`
    /// coordinates swap, lengths reflect as `max_len - length(z)` with
    /// `max_len = length(size-1)`, each descent status maps by
    /// [`BlockDescent::dual`], and every cross/Cayley link `c` maps to
    /// `size-1-c` (a Cayley second image is mapped only when the first is
    /// defined). Upstream's `orbits`/`dd` fields (marked "probably not
    /// right" there) have no Rust counterpart and are not reproduced.
    pub fn dual(&self) -> BlockGraph {
        let rank = self.rank;
        let size = self.size();
        let max_len = self.lengths.last().copied().unwrap_or(0);

        let mut xs: Vec<KgbId> = Vec::with_capacity(size);
        let mut ys: Vec<KgbId> = Vec::with_capacity(size);
        let mut descent: Vec<BlockDescent> = Vec::with_capacity(size * rank);
        let mut lengths: Vec<usize> = Vec::with_capacity(size);
        for z in (0..size).rev() {
            xs.push(self.ys[z]);
            ys.push(self.xs[z]);
            lengths.push(max_len - self.lengths[z]);
            for generator in 0..rank {
                descent.push(self.descent[z * rank + generator].dual());
            }
        }

        // Cross and Cayley tables, kept in the flat `s * size + z` layout.
        let mut cross: Vec<usize> = vec![0; size * rank];
        let mut cayley_first: Vec<Option<usize>> = vec![None; size * rank];
        let mut cayley_second: Vec<Option<usize>> = vec![None; size * rank];
        for generator in 0..rank {
            for z in (0..size).rev() {
                let source = generator * size + z;
                let target = generator * size + (size - 1 - z);
                cross[target] = size - 1 - self.cross[source];
                if let Some(first) = self.cayley_first[source] {
                    cayley_first[target] = Some(size - 1 - first);
                    if let Some(second) = self.cayley_second[source] {
                        cayley_second[target] = Some(size - 1 - second);
                    }
                }
            }
        }

        // Upstream sizes the dual's x-range as `max_y + 1`.
        let xrange = xs.iter().map(|x| x.index() + 1).max().unwrap_or(0);
        let first_z_of_x = first_zs(&xs, xrange);

        BlockGraph {
            rank,
            xs,
            ys,
            descent,
            cross,
            cayley_first,
            cayley_second,
            lengths,
            first_z_of_x,
        }
    }

    pub fn size(&self) -> usize {
        self.xs.len()
    }

    pub fn rank(&self) -> usize {
        self.rank
    }

    /// The primal KGB coordinate of `z` (upstream `Block_base::x`).
    pub fn x(&self, z: usize) -> Option<KgbId> {
        self.xs.get(z).copied()
    }

    /// The dual KGB coordinate of `z` (upstream `Block_base::y`).
    pub fn y(&self, z: usize) -> Option<KgbId> {
        self.ys.get(z).copied()
    }

    /// The primal KGB length of `z` (blocks.cpp:557), the KL-table's
    /// length ordering.
    pub fn length(&self, z: usize) -> Option<usize> {
        self.lengths.get(z).copied()
    }

    /// Upstream `Block_base::descentValue`.
    pub fn descent_value(&self, z: usize, generator: usize) -> Option<BlockDescent> {
        if generator >= self.rank {
            return None;
        }
        self.descent.get(z * self.rank + generator).copied()
    }

    /// Upstream `Block_base::cross` — defined for every generator.
    pub fn cross(&self, z: usize, generator: usize) -> Option<usize> {
        if generator >= self.rank {
            return None;
        }
        self.cross.get(generator * self.xs.len() + z).copied()
    }

    /// Upstream `Block_base::cayley` (blocks.h:143-148): the direct Cayley
    /// pair, forced to undefined at weak descents. The inner `None` is
    /// upstream's `UndefBlock`.
    pub fn cayley(&self, z: usize, generator: usize) -> Option<(Option<usize>, Option<usize>)> {
        if generator >= self.rank {
            return None;
        }
        if self.descent_value(z, generator)?.is_descent() {
            return Some((None, None));
        }
        let slot = generator * self.xs.len() + z;
        Some((self.cayley_first[slot], self.cayley_second[slot]))
    }

    /// Upstream `Block_base::inverseCayley` (blocks.h:150-155): the inverse
    /// Cayley pair, defined exactly at weak descents.
    pub fn inverse_cayley(
        &self,
        z: usize,
        generator: usize,
    ) -> Option<(Option<usize>, Option<usize>)> {
        if generator >= self.rank {
            return None;
        }
        if !self.descent_value(z, generator)?.is_descent() {
            return Some((None, None));
        }
        let slot = generator * self.xs.len() + z;
        Some((self.cayley_first[slot], self.cayley_second[slot]))
    }

    /// Look up an element by its `(x, y)` coordinates (upstream
    /// `Block::element`, blocks.cpp:242-248): the `first_z_of_x` range for
    /// `x`, then the consecutive-`y` offset. The found coordinates are
    /// verified, the role of upstream's assert.
    pub fn element(&self, x: KgbId, y: KgbId) -> Result<usize, StructureError> {
        element_at(&self.xs, &self.ys, &self.first_z_of_x, x, y)
    }

    /// The Atlas status code of `z` at `generator`
    /// ([`BlockDescent::language_code`]).
    pub fn status_code(&self, z: usize, generator: usize) -> Option<u32> {
        Some(self.descent_value(z, generator)?.language_code())
    }

    /// The Bruhat Hasse diagram of the block (blocks.cpp:1603-1656
    /// `complete_Hasse_diagram`): for each element the immediate
    /// down-neighbours. The recursion follows the first strict good descent
    /// (complex or type-I real) like the KGB case; at a split principal
    /// series the predecessors are exactly the inverse Cayley transforms of
    /// the type-II real descents.
    pub fn bruhat_hasse(&self) -> Vec<Vec<usize>> {
        crate::block_access::bruhat_hasse(self)
    }

    /// The number of Bruhat-comparable pairs of the block poset, including
    /// each element with itself (poset.cpp:197-229 `n_comparable_from_Hasse`).
    pub fn n_bruhat_comparable(hasse: &[Vec<usize>]) -> usize {
        let mut closure: Vec<std::collections::BTreeSet<usize>> = Vec::with_capacity(hasse.len());
        let mut count = 0_usize;
        for row in hasse {
            let mut cl = std::collections::BTreeSet::new();
            cl.insert(closure.len());
            for &j in row {
                cl.extend(closure[j].iter().copied());
            }
            count += cl.len();
            closure.push(cl);
        }
        count
    }
}

/// The descent status of one generator at `(x, y)` (upstream `descents`,
/// gkmod/blocks.cpp:1541-1568).
fn descents(
    x: KgbId,
    y: KgbId,
    generator: usize,
    graph: &KgbGraph,
    dual_graph: &KgbGraph,
) -> Result<BlockDescent, StructureError> {
    let status = graph
        .status(x, generator)
        .ok_or(StructureError::BlockInvariantViolation {
            invariant: "descent status",
        })?;
    match status {
        KgbStatus::Complex => {
            if graph
                .is_descent(x, generator)
                .ok_or(StructureError::BlockInvariantViolation {
                    invariant: "descent status",
                })?
            {
                Ok(BlockDescent::ComplexDescent)
            } else {
                Ok(BlockDescent::ComplexAscent)
            }
        }
        KgbStatus::ImaginaryNoncompact => {
            let crossed =
                graph
                    .cross(x, generator)
                    .ok_or(StructureError::BlockInvariantViolation {
                        invariant: "descent cross link",
                    })?;
            Ok(if crossed != x {
                BlockDescent::ImaginaryTypeI
            } else {
                BlockDescent::ImaginaryTypeII
            })
        }
        KgbStatus::Real | KgbStatus::ImaginaryCompact => {
            let dual_status =
                dual_graph
                    .status(y, generator)
                    .ok_or(StructureError::BlockInvariantViolation {
                        invariant: "dual descent status",
                    })?;
            if dual_status == KgbStatus::ImaginaryNoncompact {
                let dual_crossed = dual_graph.cross(y, generator).ok_or(
                    StructureError::BlockInvariantViolation {
                        invariant: "dual descent cross link",
                    },
                )?;
                Ok(if dual_crossed != y {
                    BlockDescent::RealTypeII
                } else {
                    BlockDescent::RealTypeI
                })
            } else if status == KgbStatus::Real {
                Ok(BlockDescent::RealNonparity)
            } else {
                Ok(BlockDescent::ImaginaryCompact)
            }
        }
    }
}

/// Upstream `compute_first_zs` (blocks.cpp:630-643): for each `x` the first
/// block element with `x(z) >= x`, with a size sentinel; `xs` values weakly
/// increase. The result has length `xrange + 1`.
fn first_zs(xs: &[KgbId], xrange: usize) -> Vec<usize> {
    let size = xs.len();
    let mut first_z_of_x = vec![0; xrange + 1];
    let mut xx = 0_usize;
    for (z, &x) in xs.iter().enumerate() {
        while xx < x.index() {
            xx += 1;
            first_z_of_x[xx] = z;
        }
    }
    while xx < xrange {
        xx += 1;
        first_z_of_x[xx] = size;
    }
    first_z_of_x
}

/// Upstream `Block::element` (blocks.cpp:242-248) on bare tables.
fn element_at(
    xs: &[KgbId],
    ys: &[KgbId],
    first_z_of_x: &[usize],
    x: KgbId,
    y: KgbId,
) -> Result<usize, StructureError> {
    let first = *first_z_of_x
        .get(x.index())
        .ok_or(StructureError::IndexOutOfRange {
            index: x.index(),
            upper_bound: first_z_of_x.len().saturating_sub(1),
        })?;
    let base_y = ys
        .get(first)
        .ok_or(StructureError::BlockInvariantViolation {
            invariant: "element lookup",
        })?;
    let z = first
        .checked_add(y.index().checked_sub(base_y.index()).ok_or(
            StructureError::BlockInvariantViolation {
                invariant: "element fiber",
            },
        )?)
        .ok_or(StructureError::ArithmeticOverflow)?;
    if xs.get(z) != Some(&x) || ys.get(z) != Some(&y) {
        return Err(StructureError::BlockInvariantViolation {
            invariant: "element fiber",
        });
    }
    Ok(z)
}

/// Upstream `first_free_slot` (blocks.cpp:153-162): fill the first empty
/// slot of a Cayley pair.
fn first_free_slot(
    first: &mut Option<usize>,
    second: &mut Option<usize>,
    z: usize,
) -> Result<(), StructureError> {
    if first.is_none() {
        *first = Some(z);
        return Ok(());
    }
    if second.is_none() {
        *second = Some(z);
        return Ok(());
    }
    Err(StructureError::BlockInvariantViolation {
        invariant: "Cayley pair slots",
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        AdjointFiberBudget, BasedRootDatum, CartanClassification, CartanClassificationBudget,
        CartanId, Coweight, IntegerLatticeBudget, InvolutionTable, InvolutionTableBudget,
        LatticeInvolution, RealFormSeed, StrongRealClassification, WeakRealFormId, Weight,
    };

    use super::*;

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

    struct Pipeline {
        inner_class: InnerClass,
        classification: CartanClassification,
        strong: StrongRealClassification,
        table: InvolutionTable,
    }

    fn pipeline(datum: BasedRootDatum, roots: usize, weyl: usize) -> Pipeline {
        let involution = LatticeInvolution::identity(&datum).unwrap();
        let inner_class = InnerClass::new(datum, involution, roots).unwrap();
        pipeline_with_class(inner_class, weyl)
    }

    fn pipeline_with_class(inner_class: InnerClass, weyl: usize) -> Pipeline {
        let classification =
            CartanClassification::build(&inner_class, &class_budget(weyl)).unwrap();
        let strong = StrongRealClassification::build(&classification, 4_096).unwrap();
        let table = InvolutionTable::new(
            &inner_class,
            InvolutionTableBudget::new(64, IntegerLatticeBudget::new(64, 100_000, 100_000, 128)),
        )
        .unwrap();
        Pipeline {
            inner_class,
            classification,
            strong,
            table,
        }
    }

    /// The KGB graph of the pipeline's form whose expected size is `size`,
    /// with the involution table the graph was built against.
    fn graph_with_size(pipeline: &mut Pipeline, size: usize) -> (KgbGraph, InvolutionTable) {
        for form in 0..pipeline.classification.weak_real_form_count() {
            if pipeline.strong.kgb_size(WeakRealFormId(form)) != Some(size) {
                continue;
            }
            pipeline
                .table
                .add_cartan(&pipeline.classification, CartanId(0))
                .unwrap();
            let seed = RealFormSeed::build(
                &pipeline.inner_class,
                &pipeline.classification,
                &pipeline.strong,
                &pipeline.table,
                WeakRealFormId(form),
                &IntegerLatticeBudget::new(64, 100_000, 100_000, 128),
                4_096,
            )
            .unwrap();
            let graph = KgbGraph::build(
                &pipeline.inner_class,
                &pipeline.classification,
                &pipeline.strong,
                &mut pipeline.table,
                &seed,
            )
            .unwrap();
            return (graph, pipeline.table.clone());
        }
        panic!("no real form with KGB size {size}");
    }

    fn sc_a1_datum() -> BasedRootDatum {
        BasedRootDatum::from_simple_data(
            1,
            vec![vec![2]],
            vec![Weight::new(vec![2])],
            vec![Coweight::new(vec![1])],
        )
        .unwrap()
    }

    #[test]
    fn a1_dual_involution_swaps_identity_and_reflection() {
        // For A1 both twists are trivial and w0 = s: f(e) = s and
        // f(s) = s.dwist(s) = e, the minus-transpose bijection.
        let primal = pipeline(sc_a1_datum(), 2, 2);
        let dual_class = crate::dual::dual_inner_class(&primal.inner_class, 2, 64).unwrap();
        let dual_system = dual_class.root_system();
        let dual_twist = dual_class.generator_twist().unwrap();
        let longest = crate::dual::longest_action(&dual_class, 2).unwrap();
        let dual_longest = WeylElement::from_action(dual_system, &longest).unwrap();
        let identity = WeylElement::identity(dual_system).unwrap();
        let reflection = WeylElement::simple_reflection(dual_system, 0).unwrap();
        assert_eq!(
            dual_involution(&[], dual_system, &dual_twist, &dual_longest).unwrap(),
            reflection
        );
        assert_eq!(
            dual_involution(&[0], dual_system, &dual_twist, &dual_longest).unwrap(),
            identity
        );
    }

    #[test]
    fn sl2r_pgl2r_block_matches_the_frozen_language_anchors() {
        // The domain/block_basic fixture (capture 3501519): block(SL(2,R),
        // PGL(2,R)) has 3 elements, element(B,0) = (KGB#0, KGB#1),
        // cross(0,B,0) = 1, status(0,B,0) = 6 (i1), status(0,B,2) = 2 (r1),
        // and Cayley(0,B,2)/inverse_Cayley(0,B,0) are undefined (the wrapper
        // returns the input).
        let mut primal = pipeline(sc_a1_datum(), 2, 2);
        let (graph, table) = graph_with_size(&mut primal, 3);
        let dual_class = crate::dual::dual_inner_class(&primal.inner_class, 2, 64).unwrap();
        let mut dual = pipeline_with_class(dual_class.clone(), 2);
        let (dual_graph, dual_table) = graph_with_size(&mut dual, 2);
        let block = BlockGraph::build(
            &graph,
            &table,
            &dual_graph,
            &dual_table,
            &dual.inner_class,
            2,
        )
        .unwrap();

        assert_eq!(block.size(), 3);
        // Packet order: the compact fiber (w = e) pairs with the dual split
        // fiber (w' = s), the split fiber with the dual compact one.
        assert_eq!(block.x(0).unwrap().index(), 0);
        assert_eq!(block.y(0).unwrap().index(), 1);
        assert_eq!(block.x(1).unwrap().index(), 1);
        assert_eq!(block.y(1).unwrap().index(), 1);
        assert_eq!(block.x(2).unwrap().index(), 2);
        assert_eq!(block.y(2).unwrap().index(), 0);
        // element lookup inverts the coordinates.
        assert_eq!(block.element(KgbId(0), KgbId(1)).unwrap(), 0);
        assert_eq!(block.element(KgbId(1), KgbId(1)).unwrap(), 1);
        assert_eq!(block.element(KgbId(2), KgbId(0)).unwrap(), 2);
        // Descent statuses and their language codes.
        assert_eq!(
            block.descent_value(0, 0),
            Some(BlockDescent::ImaginaryTypeI)
        );
        assert_eq!(
            block.descent_value(1, 0),
            Some(BlockDescent::ImaginaryTypeI)
        );
        assert_eq!(block.descent_value(2, 0), Some(BlockDescent::RealTypeI));
        assert_eq!(block.status_code(0, 0), Some(6));
        assert_eq!(block.status_code(2, 0), Some(2));
        // Cross links.
        assert_eq!(block.cross(0, 0), Some(1));
        assert_eq!(block.cross(1, 0), Some(0));
        assert_eq!(block.cross(2, 0), Some(2));
        // Cayley links: the two i1 elements share the single r1 image.
        assert_eq!(block.cayley(0, 0), Some((Some(2), None)));
        assert_eq!(block.cayley(1, 0), Some((Some(2), None)));
        assert_eq!(block.inverse_cayley(2, 0), Some((Some(0), Some(1))));
        // The undefined cases the wrapper maps back to the input index.
        assert_eq!(block.cayley(2, 0), Some((None, None)));
        assert_eq!(block.inverse_cayley(0, 0), Some((None, None)));
    }

    #[test]
    fn pgl2r_sl2r_dual_block_exercises_the_type_two_links() {
        // The dual block swaps the roles: now the primal side carries the
        // type-II imaginary fiber and the dual side the type-I real one.
        let adjoint = BasedRootDatum::standard(vec![vec![2]]).unwrap();
        let mut primal = pipeline(adjoint, 2, 2);
        let (graph, table) = graph_with_size(&mut primal, 2);
        let dual_class = crate::dual::dual_inner_class(&primal.inner_class, 2, 64).unwrap();
        let mut dual = pipeline_with_class(dual_class.clone(), 2);
        let (dual_graph, dual_table) = graph_with_size(&mut dual, 3);
        let block = BlockGraph::build(
            &graph,
            &table,
            &dual_graph,
            &dual_table,
            &dual.inner_class,
            2,
        )
        .unwrap();

        assert_eq!(block.size(), 3);
        // z=0: (0, 2) i2; z=1: (1, 0) r2; z=2: (1, 1) r2.
        assert_eq!(block.x(0).unwrap().index(), 0);
        assert_eq!(block.y(0).unwrap().index(), 2);
        assert_eq!(block.x(1).unwrap().index(), 1);
        assert_eq!(block.y(1).unwrap().index(), 0);
        assert_eq!(block.x(2).unwrap().index(), 1);
        assert_eq!(block.y(2).unwrap().index(), 1);
        assert_eq!(
            block.descent_value(0, 0),
            Some(BlockDescent::ImaginaryTypeII)
        );
        assert_eq!(block.descent_value(1, 0), Some(BlockDescent::RealTypeII));
        assert_eq!(block.descent_value(2, 0), Some(BlockDescent::RealTypeII));
        assert_eq!(block.status_code(0, 0), Some(7));
        assert_eq!(block.status_code(1, 0), Some(3));
        // The type-II Cayley is double-valued, inverse single-valued.
        assert_eq!(block.cayley(0, 0), Some((Some(1), Some(2))));
        assert_eq!(block.inverse_cayley(1, 0), Some((Some(0), None)));
        assert_eq!(block.inverse_cayley(2, 0), Some((Some(0), None)));
        assert_eq!(block.cross(0, 0), Some(0));
    }

    /// The two A1 blocks of the fixtures above: block(SL(2,R), PGL(2,R)) and
    /// block(PGL(2,R), SL(2,R)).
    fn a1_blocks() -> (BlockGraph, BlockGraph) {
        let mut primal = pipeline(sc_a1_datum(), 2, 2);
        let (graph, table) = graph_with_size(&mut primal, 3);
        let dual_class = crate::dual::dual_inner_class(&primal.inner_class, 2, 64).unwrap();
        let mut dual = pipeline_with_class(dual_class.clone(), 2);
        let (dual_graph, dual_table) = graph_with_size(&mut dual, 2);
        let block = BlockGraph::build(
            &graph,
            &table,
            &dual_graph,
            &dual_table,
            &dual.inner_class,
            2,
        )
        .unwrap();

        let adjoint = BasedRootDatum::standard(vec![vec![2]]).unwrap();
        let mut dual_primal = pipeline(adjoint, 2, 2);
        let (graph, table) = graph_with_size(&mut dual_primal, 2);
        let dual_class = crate::dual::dual_inner_class(&dual_primal.inner_class, 2, 64).unwrap();
        let mut dual = pipeline_with_class(dual_class.clone(), 2);
        let (dual_graph, dual_table) = graph_with_size(&mut dual, 3);
        let dual_block = BlockGraph::build(
            &graph,
            &table,
            &dual_graph,
            &dual_table,
            &dual.inner_class,
            2,
        )
        .unwrap();
        (block, dual_block)
    }

    #[test]
    fn a1_bruhat_hasse_uses_the_shared_topology_algorithm() {
        let (block, _) = a1_blocks();
        assert_eq!(block.bruhat_hasse(), vec![vec![], vec![], vec![0, 1]]);
        assert_eq!(
            crate::block_access::bruhat_hasse(&block),
            block.bruhat_hasse()
        );
    }

    #[test]
    fn block_descent_dual_matches_the_upstream_static_table() {
        // The d[] table of DescentStatus::dual (descents.h:74-79), indexed by
        // the Value order that BlockDescent::ALL reproduces.
        use BlockDescent::*;
        let expected = [
            (ComplexAscent, ComplexDescent),
            (RealNonparity, ImaginaryCompact),
            (ImaginaryTypeI, RealTypeII),
            (ImaginaryTypeII, RealTypeI),
            (ImaginaryCompact, RealNonparity),
            (ComplexDescent, ComplexAscent),
            (RealTypeII, ImaginaryTypeI),
            (RealTypeI, ImaginaryTypeII),
        ];
        for (value, dual) in expected {
            assert_eq!(value.dual(), dual);
            assert_eq!(dual.dual(), value, "dual is an involution");
        }
    }

    #[test]
    fn block_dual_reverses_swaps_and_maps_the_links() {
        // Bare_block::dual (blocks.cpp:474-509) on block(SL(2,R), PGL(2,R)).
        let (block, _) = a1_blocks();
        let dual = block.dual();
        let size = block.size();
        assert_eq!(dual.size(), size);
        assert_eq!(dual.rank(), block.rank());
        let max_len = block.length(size - 1).unwrap();
        for z in 0..size {
            let zd = size - 1 - z;
            // Element order reverses and the x/y coordinates swap.
            assert_eq!(dual.x(zd), block.y(z));
            assert_eq!(dual.y(zd), block.x(z));
            assert_eq!(dual.length(zd), Some(max_len - block.length(z).unwrap()));
            for generator in 0..block.rank() {
                // Descent statuses map pointwise by DescentStatus::dual.
                assert_eq!(
                    dual.descent_value(zd, generator),
                    block.descent_value(z, generator).map(BlockDescent::dual)
                );
                // Cross links map to size-1 minus the original image.
                assert_eq!(
                    dual.cross(zd, generator),
                    block.cross(z, generator).map(|c| size - 1 - c)
                );
                // The raw Cayley slots map pointwise; the second image only
                // when the first is defined (blocks.cpp:497-501).
                let slot = generator * size + z;
                let dual_slot = generator * size + zd;
                match block.cayley_first[slot] {
                    Some(first) => {
                        assert_eq!(dual.cayley_first[dual_slot], Some(size - 1 - first));
                        assert_eq!(
                            dual.cayley_second[dual_slot],
                            block.cayley_second[slot].map(|second| size - 1 - second)
                        );
                    }
                    None => {
                        assert_eq!(dual.cayley_first[dual_slot], None);
                        assert_eq!(dual.cayley_second[dual_slot], None);
                    }
                }
            }
        }
    }

    #[test]
    fn block_dual_agrees_with_the_natively_built_dual_block() {
        // block(SL(2,R), PGL(2,R)).dual() against the natively built
        // block(PGL(2,R), SL(2,R)). The dual transform reverses the element
        // order while the native build numbers by its own packets, so the
        // comparison goes through `element` lookup; upstream's dual does
        // not reorder double-valued Cayley pairs either, so those compare
        // as unordered sets.
        let (block, native) = a1_blocks();
        let dual = block.dual();
        assert_eq!(dual.size(), native.size());
        let native_z = |dual_z: usize| {
            native
                .element(dual.x(dual_z).unwrap(), dual.y(dual_z).unwrap())
                .unwrap()
        };
        // Unordered pair, mapped into the native numbering before sorting
        // (the renumbering is not order-preserving).
        let native_pair = |pair: (Option<usize>, Option<usize>)| {
            let mut values: Vec<usize> = [pair.0, pair.1]
                .into_iter()
                .flatten()
                .map(native_z)
                .collect();
            values.sort_unstable();
            values
        };
        let sorted_pair = |pair: (Option<usize>, Option<usize>)| {
            let mut values: Vec<usize> = [pair.0, pair.1].into_iter().flatten().collect();
            values.sort_unstable();
            values
        };
        for z in 0..dual.size() {
            let nz = native_z(z);
            assert_eq!(dual.length(z), native.length(nz));
            for generator in 0..dual.rank() {
                assert_eq!(
                    dual.descent_value(z, generator),
                    native.descent_value(nz, generator)
                );
                // The cross link commutes with the renumbering.
                assert_eq!(
                    native_z(dual.cross(z, generator).unwrap()),
                    native.cross(nz, generator).unwrap()
                );
                // Cayley and inverse Cayley pairs as unordered sets: the
                // dual block's images map into the native numbering, the
                // native block's only need sorting.
                assert_eq!(
                    native_pair(dual.cayley(z, generator).unwrap()),
                    sorted_pair(native.cayley(nz, generator).unwrap())
                );
                assert_eq!(
                    native_pair(dual.inverse_cayley(z, generator).unwrap()),
                    sorted_pair(native.inverse_cayley(nz, generator).unwrap())
                );
            }
        }
        // Both blocks cover their full KGB ranges, so dual is a literal
        // involution here — including the first_z_of_x tables.
        assert_eq!(block.dual().dual(), block);
        assert_eq!(native.dual().dual(), native);
    }
}
