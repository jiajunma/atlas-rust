//! Extended blocks: the `delta`-fixed part of an ordinary block, with
//! generators folded into `delta`-orbits (upstream `gkmod/ext_block.{h,cpp}`).
//!
//! This module covers the structural slice of `ext_block`:
//!
//! - [`DescValue`], the 32-value extended descent classification
//!   (ext_block.h:38-74) with its twelve predicates and the
//!   `generator_length`/`link_count` counts (ext_block.cpp:42-206).
//! - [`extended_type`], the purely combinatorial local type recognition on
//!   the parent block (ext_block.cpp:343-503).
//! - [`ExtBlock::build`], the constructor for a full parent block with the
//!   TRIVIAL block modifier (ext_block.cpp:618-668 with `bm` the identity,
//!   so `transformed_twisted` :597-616 reduces to `kgb.twisted` on both KGB
//!   coordinates, `complete_construction` :696-856, and the induced
//!   permutation [`induced`] :670-693).
//! - [`ExtBlock::build_partial`], the same constructor over a
//!   [`PartialBlock`] parent (a common block on a proper integral
//!   subsystem, blocks.cpp:733-1081). The fixed-point test is the
//!   `x` + `gamma_lambda` form of [`transformed_twisted`] (ext_block.cpp:
//!   597-616) — a partial block's `y` is a synthetic subsystem count, not a
//!   dual-KGB element — and [`fold_orbits`] runs on the SUBSYSTEM Cartan
//!   matrix (common_block::fold_orbits, blocks.cpp:1288-1292 via
//!   rootdata.cpp:1553-1577). Only the identity generator attitude is
//!   ported: a non-identity `bm.simple_pi` fails loudly.
//! - [`ExtBlock::tune_signs`] (ext_block.cpp:1707-1876), ported generically
//!   over the [`StarOracle`] trait: the per-generator `star` computation and
//!   the `ext_param` values it compares belong to the later `ext_param`/`star`
//!   slice, so they are injected here. The debug verification gates
//!   `check_quadratic`/`check_braid` (ext_block.cpp:2140-2245) are ported as
//!   [`check_quadratic`]/[`check_braid`] and run inside `tune_signs` under
//!   `debug_assertions`, exactly like upstream's `#ifndef NDEBUG` block.
//!
//! Element numbering, cross/Cayley semantics and the `UndefBlock` encoding
//! (here `None`) follow `block.rs`, which itself mirrors `blocks::Block_base`.

use crate::block::{BlockDescent, BlockGraph};
use crate::block_modifier::BlockModifier;
use crate::dynkin::folded_cartan;
use crate::kl_support::RankFlags;
use crate::partial_block::{CommonContext, PartialBlock, StandardReprMod};
use crate::{
    IntegralSubsystem, InvolutionTable, KgbGraph, LatticeInvolution, RationalWeight, RepContext,
    RootSystem, StructureError,
};

/// The extended descent status of one orbit generator at one block element
/// (upstream `ext_block::DescValue`, ext_block.h:38-74). The discriminants
/// reproduce the upstream enumeration order exactly: every even/odd pair is
/// an ascent/descent pair, so `is_descent(v) == (v as usize) % 2 == 1`
/// (ext_block.h:77).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum DescValue {
    OneComplexAscent = 0,
    OneComplexDescent,
    /// Type 1, single Cayley image.
    OneImaginarySingle,
    /// Type 1, twist-fixed inverse Cayley images.
    OneRealPairFixed,
    /// Type 2, twist-fixed Cayley images.
    OneImaginaryPairFixed,
    /// Type 2, single inverse Cayley image.
    OneRealSingle,
    /// Ascent: type 1i2 with twist-switched images.
    OneImaginaryPairSwitched,
    /// Descent: type 1r1 with twist-switched images.
    OneRealPairSwitched,
    /// An ascent.
    OneRealNonparity,
    /// A descent.
    OneImaginaryCompact,
    /// Distinct commuting complex ascents.
    TwoComplexAscent,
    /// Distinct commuting complex descents.
    TwoComplexDescent,
    /// Identical complex ascents.
    TwoSemiImaginary,
    /// Identical complex descents.
    TwoSemiReal,
    /// Commuting single-valued Cayleys.
    TwoImaginarySingleSingle,
    /// Commuting double-valued inverse Cayleys (2-valued).
    TwoRealDoubleDouble,
    /// Single-valued Cayleys become double.
    TwoImaginarySingleDoubleFixed,
    /// Single-valued inverse Cayleys become double.
    TwoRealSingleDoubleFixed,
    /// Commuting double-valued Cayleys (2-valued).
    TwoImaginaryDoubleDouble,
    /// Commuting single-valued inverse Cayleys.
    TwoRealSingleSingle,
    /// Ascent: 2i12, twist-switched images.
    TwoImaginarySingleDoubleSwitched,
    /// Descent: 2r12, twist-switched images.
    TwoRealSingleDoubleSwitched,
    /// An ascent.
    TwoRealNonparity,
    /// A descent.
    TwoImaginaryCompact,
    /// Distinct non-commuting complex ascents.
    ThreeComplexAscent,
    /// Distinct non-commuting complex descents.
    ThreeComplexDescent,
    /// Non-commuting complex ascents become imaginary.
    ThreeSemiImaginary,
    /// Non-commuting single-valued inverse Cayleys get complex.
    ThreeRealSemi,
    /// Non-commuting single-valued Cayleys become complex.
    ThreeImaginarySemi,
    /// Non-commuting complex descents become real.
    ThreeSemiReal,
    /// An ascent.
    ThreeRealNonparity,
    /// A descent.
    ThreeImaginaryCompact,
}

impl DescValue {
    /// The upstream enumeration index (ext_block.h:38-74).
    fn index(self) -> usize {
        self as usize
    }

    /// Upstream `is_descent` (ext_block.h:77): odd enumeration values.
    pub fn is_descent(self) -> bool {
        !self.index().is_multiple_of(2)
    }

    /// Upstream `is_complex` (ext_block.cpp:42-50): types 1C+, 1C-, 2C+,
    /// 2C-, 3C+, 3C-.
    pub fn is_complex(self) -> bool {
        use DescValue::*;
        matches!(
            self,
            OneComplexAscent
                | OneComplexDescent
                | TwoComplexAscent
                | TwoComplexDescent
                | ThreeComplexAscent
                | ThreeComplexDescent
        )
    }

    /// Upstream `is_unique_image` (ext_block.cpp:52-63): types 1r1f, 1i2f,
    /// 2r11, 2i22, and defects; complex types also have unique images.
    pub fn is_unique_image(self) -> bool {
        use DescValue::*;
        matches!(
            self,
            OneRealPairFixed
                | OneImaginaryPairFixed
                | TwoSemiImaginary
                | TwoSemiReal
                | TwoRealDoubleDouble
                | TwoImaginaryDoubleDouble
                | ThreeSemiImaginary
                | ThreeRealSemi
                | ThreeImaginarySemi
                | ThreeSemiReal
        ) || self.is_complex()
    }

    /// Upstream `has_double_image` (ext_block.cpp:65-74): types 1r1f, 1i2f,
    /// 2r11, 2i22, and quads.
    pub fn has_double_image(self) -> bool {
        use DescValue::*;
        matches!(
            self,
            OneRealPairFixed
                | OneImaginaryPairFixed
                | TwoRealDoubleDouble
                | TwoImaginaryDoubleDouble
                | TwoImaginarySingleDoubleFixed
                | TwoRealSingleDoubleFixed
        )
    }

    /// Upstream `is_like_nonparity` (ext_block.cpp:77-85): ascents with
    /// zero links.
    pub fn is_like_nonparity(self) -> bool {
        use DescValue::*;
        matches!(
            self,
            OneRealNonparity
                | OneImaginaryPairSwitched
                | TwoRealNonparity
                | TwoImaginarySingleDoubleSwitched
                | ThreeRealNonparity
        )
    }

    /// Upstream `is_like_compact` (ext_block.cpp:87-95): descents with
    /// zero links.
    pub fn is_like_compact(self) -> bool {
        use DescValue::*;
        matches!(
            self,
            OneImaginaryCompact
                | OneRealPairSwitched
                | TwoImaginaryCompact
                | TwoRealSingleDoubleSwitched
                | ThreeImaginaryCompact
        )
    }

    /// Upstream `is_like_type_1` (ext_block.cpp:97-104): types 1i1, 1r1f,
    /// 2i11, 2r11.
    pub fn is_like_type_1(self) -> bool {
        use DescValue::*;
        matches!(
            self,
            OneImaginarySingle | OneRealPairFixed | TwoImaginarySingleSingle | TwoRealDoubleDouble
        )
    }

    /// Upstream `is_like_type_2` (ext_block.cpp:106-113): types 1i2f, 1r2,
    /// 2i22, 2r22.
    pub fn is_like_type_2(self) -> bool {
        use DescValue::*;
        matches!(
            self,
            OneImaginaryPairFixed | OneRealSingle | TwoImaginaryDoubleDouble | TwoRealSingleSingle
        )
    }

    /// Upstream `has_defect` (ext_block.cpp:115-123): types 2Ci, 2Cr, 3Ci,
    /// 3r, 3Cr, 3i.
    pub fn has_defect(self) -> bool {
        use DescValue::*;
        matches!(
            self,
            TwoSemiImaginary
                | TwoSemiReal
                | ThreeSemiImaginary
                | ThreeRealSemi
                | ThreeImaginarySemi
                | ThreeSemiReal
        )
    }

    /// Upstream `has_quadruple` (ext_block.cpp:125-132): types 2i12f and
    /// 2r21f.
    pub fn has_quadruple(self) -> bool {
        use DescValue::*;
        matches!(
            self,
            TwoImaginarySingleDoubleFixed | TwoRealSingleDoubleFixed
        )
    }

    /// Upstream `has_october_surprise` (ext_block.cpp:134-139): links with
    /// an even length difference, singled out since October 2016.
    pub fn has_october_surprise(self) -> bool {
        self.generator_length() == if self.has_defect() { 3 } else { 2 }
    }

    /// Upstream `is_proper_ascent` (ext_block.cpp:141-144): an ascent with
    /// at least one link.
    pub fn is_proper_ascent(self) -> bool {
        !(self.is_descent() || self.is_like_nonparity())
    }

    /// Upstream `might_be_uncertain` (ext_block.cpp:146-160): for these
    /// ascent types a link may remain undefined at the edge of a partial
    /// block, which might make the type itself uncertain.
    pub fn might_be_uncertain(self) -> bool {
        use DescValue::*;
        matches!(
            self,
            OneComplexAscent
                | TwoComplexAscent
                | ThreeComplexAscent
                | OneImaginaryPairFixed
                | TwoImaginarySingleDoubleFixed
                | TwoImaginaryDoubleDouble
                | ThreeImaginarySemi
                | ThreeSemiImaginary
        )
    }

    /// Upstream `generator_length` (ext_block.cpp:162-163): the length of
    /// the folded generator, 1, 2, or 3.
    pub fn generator_length(self) -> usize {
        if self.index() < DescValue::TwoComplexAscent.index() {
            1
        } else if self.index() < DescValue::ThreeComplexAscent.index() {
            2
        } else {
            3
        }
    }

    /// Upstream `link_count` (ext_block.cpp:165-206): the number of links
    /// recorded for this type.
    pub fn link_count(self) -> usize {
        use DescValue::*;
        match self {
            // Zero-valued Cayleys: nothing recorded (cross action trivial).
            OneRealNonparity
            | OneImaginaryCompact
            | OneImaginaryPairSwitched
            | OneRealPairSwitched
            | TwoRealNonparity
            | TwoImaginaryCompact
            | TwoImaginarySingleDoubleSwitched
            | TwoRealSingleDoubleSwitched
            | ThreeRealNonparity
            | ThreeImaginaryCompact => 0,
            // Complex cases (record only the cross action).
            OneComplexAscent | OneComplexDescent | TwoComplexAscent | TwoComplexDescent
            | ThreeComplexAscent | ThreeComplexDescent => 1,
            // Semi cases do not record their (trivial) cross action.
            TwoSemiImaginary | TwoSemiReal | ThreeSemiImaginary | ThreeRealSemi
            | ThreeImaginarySemi | ThreeSemiReal => 1,
            // Some single-valued extended Cayleys use the second link for
            // the cross action.
            OneImaginarySingle | OneRealSingle | TwoImaginarySingleSingle | TwoRealSingleSingle => {
                2
            }
            // Double-valued Cayleys also have unrecorded trivial cross
            // actions.
            OneRealPairFixed
            | OneImaginaryPairFixed
            | TwoRealDoubleDouble
            | TwoImaginaryDoubleDouble
            | TwoImaginarySingleDoubleFixed
            | TwoRealSingleDoubleFixed => 2,
        }
    }
}

/// A generator of the extended (folded) Weyl group: a `delta`-orbit of
/// simple generators of the parent block (upstream `ext_gen`,
/// structure/lietype.h:153-176). Orbits have size one or two; a two-element
/// orbit of commuting generators has kind [`ExtGenKind::Two`], a
/// non-commuting one [`ExtGenKind::Three`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExtGen {
    pub kind: ExtGenKind,
    pub s0: usize,
    /// The second orbit member; `usize::MAX` (upstream `~0`) for kind
    /// [`ExtGenKind::One`].
    pub s1: usize,
}

/// Upstream `ext_gen::type` (one/two/three).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExtGenKind {
    One,
    Two,
    Three,
}

impl ExtGen {
    /// Upstream `ext_gen::length`: also the size of `w_kappa`.
    pub fn length(&self) -> usize {
        match self.kind {
            ExtGenKind::One => 1,
            ExtGenKind::Two => 2,
            ExtGenKind::Three => 3,
        }
    }

    fn is_orbit_pair(&self) -> bool {
        self.kind != ExtGenKind::One
    }
}

/// Upstream `rootdata::fold_orbits(rd, delta)` (structure/rootdata.cpp:
/// 1532-1551), specialized to the simple generators of the parent block:
/// `twist` is the simple-root permutation induced by `delta` and `cartan`
/// the parent datum's Cartan matrix (`cartan[i][j] = <alpha_i, alpha_j^v>`,
/// the crate convention). Orbits are emitted in increasing `s0` order; a
/// non-involution `twist` is upstream's "Not a distinguished involution".
pub fn fold_orbits(cartan: &[Vec<i32>], twist: &[usize]) -> Result<Vec<ExtGen>, StructureError> {
    let rank = cartan.len();
    if twist.len() != rank {
        return Err(StructureError::RankMismatch {
            expected: rank,
            actual: twist.len(),
        });
    }
    let mut result = Vec::with_capacity(rank);
    for (s, &t) in twist.iter().enumerate() {
        if t >= rank || twist[t] != s {
            // Upstream throws when the image is not a simple root or the
            // permutation is not an involution ("Not a distinguished
            // involution").
            return Err(StructureError::InvalidRootDatumAutomorphism);
        }
        if t == s {
            result.push(ExtGen {
                kind: ExtGenKind::One,
                s0: s,
                s1: usize::MAX,
            });
        } else if t > s {
            // `ext_gen(commute, s, t)`: commuting pairs fold to length 2,
            // linked pairs to length 3.
            let orthogonal = cartan[s][t] == 0 && cartan[t][s] == 0;
            result.push(ExtGen {
                kind: if orthogonal {
                    ExtGenKind::Two
                } else {
                    ExtGenKind::Three
                },
                s0: s,
                s1: t,
            });
        }
    }
    Ok(result)
}

/// The parent-block surface that [`extended_type`] and
/// [`ExtBlock::complete_construction`] read (upstream `blocks::common_block`
/// as used by the `ext_block` constructor, ext_block.cpp:618-668): both a
/// full [`BlockGraph`] and a [`PartialBlock`] over an integral subsystem
/// qualify. Accessors are `(z, generator)` ordered like [`BlockGraph`]; the
/// Cayley pairs follow upstream `Block_base::cayley`/`inverseCayley`
/// (blocks.h:143-155): the shared slot is exposed only in its own direction
/// (`cayley` at non-descents, `inverse_cayley` at weak descents), masked to
/// `(None, None)` otherwise.
pub trait ParentBlock {
    /// Upstream `Block_base::size`.
    fn size(&self) -> usize;
    /// Upstream `Block_base::length`.
    fn length(&self, z: usize) -> Option<usize>;
    /// Upstream `Block_base::descentValue`.
    fn descent_value(&self, z: usize, generator: usize) -> Option<BlockDescent>;
    /// Upstream `Block_base::cross`; `None` is `UndefBlock`.
    fn cross(&self, z: usize, generator: usize) -> Option<usize>;
    /// Upstream `Block_base::cayley` (blocks.h:143-148): the direct Cayley
    /// pair, masked to `(None, None)` at weak descents.
    fn cayley(&self, z: usize, generator: usize) -> Option<(Option<usize>, Option<usize>)>;
    /// Upstream `Block_base::inverseCayley` (blocks.h:150-155): the inverse
    /// Cayley pair, masked to `(None, None)` at non-descents.
    fn inverse_cayley(&self, z: usize, generator: usize) -> Option<(Option<usize>, Option<usize>)>;
}

impl ParentBlock for BlockGraph {
    fn size(&self) -> usize {
        self.size()
    }

    fn length(&self, z: usize) -> Option<usize> {
        self.length(z)
    }

    fn descent_value(&self, z: usize, generator: usize) -> Option<BlockDescent> {
        self.descent_value(z, generator)
    }

    fn cross(&self, z: usize, generator: usize) -> Option<usize> {
        self.cross(z, generator)
    }

    fn cayley(&self, z: usize, generator: usize) -> Option<(Option<usize>, Option<usize>)> {
        self.cayley(z, generator)
    }

    fn inverse_cayley(&self, z: usize, generator: usize) -> Option<(Option<usize>, Option<usize>)> {
        self.inverse_cayley(z, generator)
    }
}

impl ParentBlock for PartialBlock {
    fn size(&self) -> usize {
        self.size()
    }

    fn length(&self, z: usize) -> Option<usize> {
        self.length(z)
    }

    fn descent_value(&self, z: usize, generator: usize) -> Option<BlockDescent> {
        self.descent(z, generator)
    }

    fn cross(&self, z: usize, generator: usize) -> Option<usize> {
        self.cross(generator, z)
    }

    fn cayley(&self, z: usize, generator: usize) -> Option<(Option<usize>, Option<usize>)> {
        // The partial block stores both directions in the one
        // `Cayley_image` slot, as upstream does; mask like Block_base.
        if self.descent(z, generator)?.is_descent() {
            return Some((None, None));
        }
        self.cayley(generator, z)
    }

    fn inverse_cayley(&self, z: usize, generator: usize) -> Option<(Option<usize>, Option<usize>)> {
        if !self.descent(z, generator)?.is_descent() {
            return Some((None, None));
        }
        self.cayley(generator, z)
    }
}

/// The local extended type of the `delta`-fixed parent element `z` for the
/// orbit `p`, together with its parent-block link (upstream
/// `extended_type`, ext_block.cpp:343-503). `None` links are upstream's
/// `UndefBlock`; they arise only at the edge of a partial parent block (the
/// "uncertain" cases) since a full `BlockGraph` has total cross/Cayley
/// tables. No linear algebra is used.
pub fn extended_type<P: ParentBlock>(
    parent: &P,
    z: usize,
    p: ExtGen,
    fixed_points: &[bool],
) -> (DescValue, Option<usize>) {
    let is_fixed =
        |element: Option<usize>| -> bool { matches!(element, Some(t) if fixed_points[t]) };
    let descent = |z: usize, s: usize| -> BlockDescent {
        parent
            .descent_value(z, s)
            .expect("extended_type: element/generator in range")
    };
    let cross = |z: usize, s: usize| -> Option<usize> { parent.cross(z, s) };
    let cayley = |z: usize, s: usize| -> (Option<usize>, Option<usize>) {
        parent
            .cayley(z, s)
            .expect("extended_type: element/generator in range")
    };
    let inverse_cayley = |z: usize, s: usize| -> (Option<usize>, Option<usize>) {
        parent
            .inverse_cayley(z, s)
            .expect("extended_type: element/generator in range")
    };

    let s0 = p.s0;
    let s1 = p.s1;
    match p.kind {
        ExtGenKind::One => match descent(z, s0) {
            BlockDescent::ComplexAscent => (DescValue::OneComplexAscent, cross(z, s0)),
            BlockDescent::ComplexDescent => (DescValue::OneComplexDescent, cross(z, s0)),
            BlockDescent::RealNonparity => (DescValue::OneRealNonparity, None),
            BlockDescent::ImaginaryCompact => (DescValue::OneImaginaryCompact, None),
            BlockDescent::ImaginaryTypeI => (DescValue::OneImaginarySingle, cayley(z, s0).0),
            BlockDescent::RealTypeII => (DescValue::OneRealSingle, inverse_cayley(z, s0).0),
            BlockDescent::ImaginaryTypeII => {
                let t = cayley(z, s0).0;
                // If |t| is undefined the type is uncertain; tentatively
                // return "fixed" (upstream comment at :365-366).
                if t.is_none() || is_fixed(t) {
                    (DescValue::OneImaginaryPairFixed, t)
                } else {
                    (DescValue::OneImaginaryPairSwitched, None)
                }
            }
            BlockDescent::RealTypeI => {
                let t = inverse_cayley(z, s0).0;
                if is_fixed(t) {
                    (DescValue::OneRealPairFixed, t)
                } else {
                    (DescValue::OneRealPairSwitched, None)
                }
            }
        },
        ExtGenKind::Two => match descent(z, s0) {
            BlockDescent::ComplexAscent => {
                let t = cross(z, s0);
                match t {
                    None => (DescValue::TwoComplexAscent, None), // uncertain
                    Some(t) if Some(t) == cross(z, s1) => (DescValue::TwoSemiImaginary, Some(t)),
                    Some(t) => (DescValue::TwoComplexAscent, cross(t, s1)),
                }
            }
            BlockDescent::ComplexDescent => {
                let t = cross(z, s0);
                if t == cross(z, s1) {
                    (DescValue::TwoSemiReal, t)
                } else {
                    (DescValue::TwoComplexDescent, t.and_then(|t| cross(t, s1)))
                }
            }
            BlockDescent::RealNonparity => (DescValue::TwoRealNonparity, None),
            BlockDescent::ImaginaryCompact => (DescValue::TwoImaginaryCompact, None),
            BlockDescent::ImaginaryTypeI => {
                let t = cayley(z, s0).0; // unique Cayley ascent
                match t {
                    None => (DescValue::TwoImaginarySingleDoubleFixed, None), // uncertain
                    Some(t) => {
                        if descent(t, s1) == BlockDescent::ImaginaryTypeI {
                            (DescValue::TwoImaginarySingleSingle, cayley(t, s1).0)
                        } else {
                            let link = cayley(t, s1).0; // uncertain when undefined
                            if link.is_none() || is_fixed(link) {
                                (DescValue::TwoImaginarySingleDoubleFixed, link)
                            } else {
                                (DescValue::TwoImaginarySingleDoubleSwitched, None)
                            }
                        }
                    }
                }
            }
            BlockDescent::RealTypeII => {
                let t = inverse_cayley(z, s0)
                    .0
                    .expect("extended_type: type II inverse Cayley defined");
                if descent(t, s1) == BlockDescent::RealTypeII {
                    (DescValue::TwoRealSingleSingle, inverse_cayley(t, s1).0)
                } else {
                    let link = inverse_cayley(t, s1).0;
                    if is_fixed(link) {
                        (DescValue::TwoRealSingleDoubleFixed, link)
                    } else {
                        (DescValue::TwoRealSingleDoubleSwitched, None)
                    }
                }
            }
            BlockDescent::ImaginaryTypeII => {
                let mut tmp = cayley(z, s0).0;
                if tmp.is_none() {
                    // First Cayley ascent crossed the edge of a partial
                    // block; both links lie beyond the edge too.
                    return (DescValue::TwoImaginaryDoubleDouble, None);
                }
                let mut pair = cayley(tmp.expect("checked"), s1);
                if pair.0.is_none() || (!is_fixed(pair.0) && pair.1.is_none()) {
                    // Try again with the other Cayley ascent by |s0| of |z|.
                    tmp = cayley(z, s0).1;
                    if tmp.is_none() {
                        return (DescValue::TwoImaginaryDoubleDouble, None);
                    }
                    pair = cayley(tmp.expect("checked"), s1);
                    if pair.0.is_none() {
                        return (DescValue::TwoImaginaryDoubleDouble, None);
                    }
                }
                let link = if is_fixed(pair.0) { pair.0 } else { pair.1 };
                debug_assert!(link.is_none() || is_fixed(link));
                (DescValue::TwoImaginaryDoubleDouble, link)
            }
            BlockDescent::RealTypeI => {
                let t = inverse_cayley(z, s0)
                    .0
                    .expect("extended_type: type I inverse Cayley defined");
                let mut link = inverse_cayley(t, s1).0;
                if !is_fixed(link) {
                    link = link.and_then(|l| cross(l, s0));
                    debug_assert!(is_fixed(link));
                }
                (DescValue::TwoRealDoubleDouble, link)
            }
        },
        ExtGenKind::Three => match descent(z, s0) {
            BlockDescent::RealNonparity => (DescValue::ThreeRealNonparity, None),
            BlockDescent::ImaginaryCompact => (DescValue::ThreeImaginaryCompact, None),
            BlockDescent::ComplexAscent => {
                let t = cross(z, s0);
                match t {
                    None => (DescValue::ThreeComplexAscent, None), // uncertain
                    Some(t) => {
                        if Some(t) == cross(t, s1) {
                            debug_assert!(descent(t, s1) == BlockDescent::ImaginaryTypeII);
                            let mut link = cayley(t, s1).0;
                            if link.is_some() && !is_fixed(link) {
                                // Choose the door without a goat.
                                link = cayley(t, s1).1;
                                debug_assert!(link.is_none() || is_fixed(link));
                            }
                            // Certain, but the link may be undefined.
                            (DescValue::ThreeSemiImaginary, link)
                        } else {
                            let mut link = cross(t, s1);
                            if link.is_some() {
                                // Continue up the third link.
                                link = link.and_then(|l| cross(l, s0));
                                debug_assert!(link.is_none() || is_fixed(link));
                            }
                            // Certain, but the link may be undefined.
                            (DescValue::ThreeComplexAscent, link)
                        }
                    }
                }
            }
            BlockDescent::ComplexDescent => {
                let t = cross(z, s0).expect("extended_type: complex descent cross");
                if Some(t) == cross(t, s1) {
                    debug_assert!(descent(t, s1) == BlockDescent::RealTypeI);
                    let mut link = inverse_cayley(t, s1).0;
                    if !is_fixed(link) {
                        link = link.and_then(|l| cross(l, s1));
                        debug_assert!(is_fixed(link));
                    }
                    (DescValue::ThreeSemiReal, link)
                } else {
                    let link = cross(t, s1).and_then(|l| cross(l, s0));
                    debug_assert!(is_fixed(link));
                    (DescValue::ThreeComplexDescent, link)
                }
            }
            BlockDescent::ImaginaryTypeI => {
                let t = cayley(z, s0).0;
                match t {
                    // Certain, but with unset link.
                    None => (DescValue::ThreeImaginarySemi, None),
                    Some(t) => {
                        let link = cross(t, s1); // may be undefined
                        debug_assert!(
                            link.is_none() || {
                                is_fixed(link)
                                    && link == cross(cayley(z, s1).0.expect("Cayley defined"), s0)
                            }
                        );
                        (DescValue::ThreeImaginarySemi, link)
                    }
                }
            }
            BlockDescent::RealTypeII => {
                let t = inverse_cayley(z, s0)
                    .0
                    .expect("extended_type: type II inverse Cayley defined");
                let link = cross(t, s1);
                debug_assert_eq!(link, cross(inverse_cayley(z, s1).0.expect("defined"), s0));
                debug_assert!(is_fixed(link));
                (DescValue::ThreeRealSemi, link)
            }
            // These cases should never occur (upstream :498-499).
            BlockDescent::ImaginaryTypeII | BlockDescent::RealTypeI => {
                unreachable!("extended_type: type-II imaginary / type-I real at a length-3 orbit")
            }
        },
    }
}

/// The subsystem Cartan matrix of an [`IntegralSubsystem`] (upstream
/// `RootSystem::Cartan_matrix(rb)`, rootdata.cpp:523-537, entered into
/// `SubSystem` at subsystem.cpp:31): `result[s][t] = <alpha_s, alpha_t^v>`
/// on the subsystem simple roots, in the crate's Cartan convention.
fn subsystem_cartan(
    system: &RootSystem,
    sub: &IntegralSubsystem,
) -> Result<Vec<Vec<i32>>, StructureError> {
    let rank = sub.rank();
    let mut cartan = vec![vec![0i32; rank]; rank];
    for s in 0..rank {
        let root = sub.parent_root(s).and_then(|id| system.root(id)).ok_or(
            StructureError::IndexOutOfRange {
                index: s,
                upper_bound: rank,
            },
        )?;
        for t in 0..rank {
            let coroot = sub.parent_root(t).and_then(|id| system.coroot(id)).ok_or(
                StructureError::IndexOutOfRange {
                    index: t,
                    upper_bound: rank,
                },
            )?;
            cartan[s][t] = crate::pair(root, coroot)?;
        }
    }
    Ok(cartan)
}

/// The twist of the subsystem simple generators induced by `delta`
/// (upstream `rootdata::fold_orbits(rd, roots, delta)`, rootdata.cpp:
/// 1553-1577): `result[s]` is the subsystem generator whose parent root is
/// `delta * parent_root(s)`. A `delta` that does not preserve the integral
/// system is upstream's "Not a distinguished involution".
fn subsystem_twist(
    system: &RootSystem,
    sub: &IntegralSubsystem,
    delta: &LatticeInvolution,
) -> Result<Vec<usize>, StructureError> {
    let rank = sub.rank();
    let mut roots = Vec::with_capacity(rank);
    for s in 0..rank {
        let root = sub.parent_root(s).and_then(|id| system.root(id)).ok_or(
            StructureError::IndexOutOfRange {
                index: s,
                upper_bound: rank,
            },
        )?;
        roots.push(root.clone());
    }
    let mut twist = Vec::with_capacity(rank);
    for root in &roots {
        let image = delta.act_on_weight(root)?;
        let position = roots
            .iter()
            .position(|candidate| *candidate == image)
            .ok_or(StructureError::InvalidRootDatumAutomorphism)?;
        twist.push(position);
    }
    Ok(twist)
}

/// Upstream `transformed_twisted` (ext_block.cpp:597-616): the block
/// element that, after `bm`-transforming the block, corresponds to element
/// `z` twisted by `delta`. `Ok(None)` is upstream's `UndefBlock` lookup
/// miss (the twisted representative lies outside the block); an
/// upstream-`UndefKGB` twist target also lands here, since no stored
/// element matches it.
///
/// `block.context()` upstream is the plain `Rep_context`
/// (blocks.h:380), so the shift/transform steps are the ambient
/// [`RepContext::shift_srm`]/[`RepContext::transform_srm`]; with the
/// trivial modifier they renormalize to the identity.
pub fn transformed_twisted(
    parent: &PartialBlock,
    rc: &RepContext,
    bm: &BlockModifier,
    delta: &LatticeInvolution,
    twist: &[usize],
    z: usize,
) -> Result<Option<usize>, StructureError> {
    let mut rep = parent
        .element(z)
        .cloned()
        .ok_or(StructureError::IndexOutOfRange {
            index: z,
            upper_bound: parent.size(),
        })?;
    rc.shift_srm(bm.shift(), &mut rep)?;
    rc.transform_srm::<false>(bm.w(), &mut rep)?;

    let Some(x1) = rc.graph().twisted(rep.x(), rc.table(), delta, twist)? else {
        return Ok(None);
    };
    let new_gl = rep.gamma_lambda().apply_matrix(delta.weight_matrix())?;
    let mut rep = StandardReprMod::build(rc, x1, &new_gl)?;

    rc.transform_srm::<true>(bm.w(), &mut rep)?;
    let unshift = RationalWeight::zero(bm.shift().numerator().len())?.sub(bm.shift())?;
    rc.shift_srm(&unshift, &mut rep)?;
    Ok(parent.lookup(&rep))
}

/// The per-element, per-generator data of the extended block (upstream
/// `ext_block::block_fields`, ext_block.h:121-127).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BlockFields {
    pub kind: DescValue,
    /// First link in extended-block numbering (`None` = `UndefBlock`).
    pub link: Option<usize>,
    /// Second link, present only for `link_count(kind) == 2` types.
    pub second: Option<usize>,
}

/// The extended block of a `delta`-stable parent block (upstream
/// `ext_block::ext_block(common_block, bm, delta, pol_hash)` with the
/// trivial block modifier, ext_block.cpp:618-668). The parent is a full
/// [`BlockGraph`] for [`ExtBlock::build`] or a [`PartialBlock`] on an
/// integral subsystem for [`ExtBlock::build_partial`]. The sign flips start
/// cleared and are set by [`ExtBlock::tune_signs`].
#[derive(Clone, Debug)]
pub struct ExtBlock {
    /// `delta`-orbits of the parent generators (upstream `orbits`).
    orbits: Vec<ExtGen>,
    /// Parent element index per extended element (upstream `elt_info::z`).
    zs: Vec<usize>,
    /// Sign flips per extended element (upstream `elt_info::flips`, a pair
    /// of generator bitsets).
    flips: Vec<[u32; 2]>,
    /// `data[orbit][element]` (upstream `data`).
    data: Vec<Vec<BlockFields>>,
    /// Start indices of parent-length levels (upstream `l_start`).
    l_start: Vec<usize>,
    /// The folded Cartan matrix on the orbits (upstream `diagram`; this
    /// module keeps the integer matrix from [`folded_cartan`] instead of a
    /// `DynkinDiagram` object).
    folded: Vec<Vec<i32>>,
}

impl ExtBlock {
    /// Constructor for a full parent block with the trivial block modifier
    /// (ext_block.cpp:618-668 + 696-856). With `bm` trivial,
    /// `transformed_twisted` (ext_block.cpp:597-616) reduces to twisting
    /// both KGB coordinates: `z` is `delta`-fixed iff `kgb.twisted(x(z),
    /// delta) == x(z)` and `dual_kgb.twisted(y(z), delta^T) == y(z)` — the
    /// same criterion the old full-block constructor used
    /// (ext_block.cpp:508-515, 585-587). `cartan` is the parent datum's
    /// Cartan matrix (for [`fold_orbits`] and the folded diagram); `twist`
    /// and `dual_twist` are the simple-root permutations induced by `delta`
    /// and by `dual_delta` (its transpose on the dual datum), as validated
    /// by `InnerClass::based_involution_twist`.
    #[allow(clippy::too_many_arguments)]
    pub fn build(
        parent: &BlockGraph,
        graph: &KgbGraph,
        table: &InvolutionTable,
        dual_graph: &KgbGraph,
        dual_table: &InvolutionTable,
        delta: &LatticeInvolution,
        twist: &[usize],
        dual_delta: &LatticeInvolution,
        dual_twist: &[usize],
        cartan: &[Vec<i32>],
    ) -> Result<Self, StructureError> {
        let orbits = fold_orbits(cartan, twist)?;

        // The delta-fixed points of the block (ext_block.cpp:630-635).
        let mut fixed_points = vec![false; parent.size()];
        for z in 0..parent.size() {
            let x = parent.x(z).ok_or(StructureError::BlockInvariantViolation {
                invariant: "block element x coordinate",
            })?;
            let y = parent.y(z).ok_or(StructureError::BlockInvariantViolation {
                invariant: "block element y coordinate",
            })?;
            fixed_points[z] = graph.twisted(x, table, delta, twist)? == Some(x)
                && dual_graph.twisted(y, dual_table, dual_delta, dual_twist)? == Some(y);
        }

        let folded = folded_cartan(cartan, &orbits)?;
        Ok(Self::complete_construction(
            parent,
            orbits,
            &fixed_points,
            folded,
        ))
    }

    /// The same constructor over a [`PartialBlock`] parent: upstream
    /// `ext_block::ext_block(common_block, bm, delta, pol_hash)`
    /// (ext_block.cpp:618-668) with the block built on gamma's integral
    /// subsystem (blocks.cpp:733-1081). `pol_hash` is the crate's separate
    /// `ext_kl` concern and is not threaded through here.
    ///
    /// Differences from [`Self::build`], all forced by the partial parent:
    ///
    /// - The fixed-point test is [`transformed_twisted`] verbatim: the
    ///   `x` + `gamma_lambda` representative of each element is twisted and
    ///   looked up with [`PartialBlock::lookup`] (a partial block's `y` is a
    ///   synthetic subsystem y-count, NOT a dual-KGB element, so the dual
    ///   `kgb.twisted` test of the full-block path does not apply).
    /// - [`fold_orbits`] runs on the SUBSYSTEM Cartan matrix and the
    ///   subsystem generator twist (common_block::fold_orbits,
    ///   blocks.cpp:1288-1292, via rootdata.cpp:1553-1577), so orbit member
    ///   indices are subsystem generator numbers. `tune_signs` must
    ///   translate them back to parent root numbers through the locator's
    ///   `simp_int` list (the `simply_ints` argument): after the cofolding
    ///   permutation below, orbit members are `simp_int` POSITIONS.
    /// - The generator attitude is cofolded after `complete_construction`
    ///   (ext_block.cpp:636-663): diagram, orbits, and link tables are
    ///   permuted by [`induced`]`(orbits, bm.simple_pi)` and each orbit's
    ///   member numbers are rewritten through `bm.simple_pi`. The
    ///   `bm.shift`/`bm.w` transport inside [`transformed_twisted`] applies
    ///   likewise.
    ///
    /// Like [`Self::build`], the sign flips start cleared; callers run
    /// [`Self::tune_signs`] with a `PartialBlock`-backed
    /// [`StarOracle`](crate::ext_param::PartialBlockOracle).
    pub fn build_partial(
        parent: &PartialBlock,
        ctxt: &CommonContext<'_, '_>,
        bm: &BlockModifier,
        delta: &LatticeInvolution,
        twist: &[usize],
    ) -> Result<Self, StructureError> {
        let rc = ctxt.rep_context();
        let sub = ctxt.subsystem();
        let block_rank = sub.rank();
        if bm.simple_pi().len() != block_rank {
            return Err(StructureError::RepInvariantViolation {
                invariant: "generator attitude rank matches the integral subsystem",
            });
        }

        let system = rc.root_system();
        let cartan = subsystem_cartan(system, sub)?;
        let sub_twist = subsystem_twist(system, sub, delta)?;
        let orbits = fold_orbits(&cartan, &sub_twist)?;

        // The delta-fixed points of the block (ext_block.cpp:630-635).
        let mut fixed_points = vec![false; parent.size()];
        for z in 0..parent.size() {
            fixed_points[z] = transformed_twisted(parent, rc, bm, delta, twist, z)?
                .is_some_and(|image| image == z);
        }

        let folded = folded_cartan(&cartan, &orbits)?;
        let mut block = Self::complete_construction(parent, orbits, &fixed_points, folded);

        // The cofolded generator attitude (ext_block.cpp:636-663): permute
        // the folded diagram, the orbits, and the per-generator link tables
        // by the orbit permutation induced from `bm.simple_pi`, then rewrite
        // each orbit's member generator numbers through `bm.simple_pi`
        // itself (keeping pairs sorted).  Identity attitudes induce the
        // identity permutation, so this is a no-op on the existing paths.
        let opi = induced(&block.orbits, bm.simple_pi());
        if !opi.iter().enumerate().all(|(index, &image)| index == image) {
            // dynkin::permute (dynkin.cpp:339-349): forward push both axes.
            let diagram_rank = block.folded.len();
            let mut diagram = vec![vec![0_i32; diagram_rank]; diagram_rank];
            for (i, row) in block.folded.iter().enumerate() {
                for (j, &entry) in row.iter().enumerate() {
                    diagram[opi[i]][opi[j]] = entry;
                }
            }
            block.folded = diagram;
            // Permutation::permute (permutations_def.h:62-80): v'[pi[i]] = v[i].
            push_permute(&opi, &mut block.orbits);
            for orbit in &mut block.orbits {
                orbit.s0 = bm.simple_pi()[orbit.s0];
                if orbit.is_orbit_pair() {
                    orbit.s1 = bm.simple_pi()[orbit.s1];
                    if orbit.s0 > orbit.s1 {
                        std::mem::swap(&mut orbit.s0, &mut orbit.s1);
                    }
                }
            }
            push_permute(&opi, &mut block.data);
        }
        Ok(block)
    }

    /// Upstream `complete_construction` (ext_block.cpp:696-856): build the
    /// per-element tables from the fixed-point bitmap.
    fn complete_construction<P: ParentBlock>(
        parent: &P,
        orbits: Vec<ExtGen>,
        fixed_points: &[bool],
        folded: Vec<Vec<i32>>,
    ) -> Self {
        let folded_rank = orbits.len();
        // |child_nr| and |parent_nr| tables (ext_block.cpp:699-711).
        let mut child_nr = vec![usize::MAX; parent.size()];
        let mut parent_nr = Vec::with_capacity(parent.size());
        let max_length = parent
            .length(parent.size() - 1)
            .expect("complete_construction: nonempty block");
        let mut l_start = vec![0usize; max_length + 2];
        {
            let mut cur_len = 0usize;
            for (z, &fixed) in fixed_points.iter().enumerate() {
                if !fixed {
                    continue;
                }
                let x = parent_nr.len();
                parent_nr.push(z);
                child_nr[z] = x;
                let len = parent.length(z).expect("complete_construction: length");
                while cur_len < len {
                    // Mark |x| as first of length at least |cur_len + 1|.
                    cur_len += 1;
                    l_start[cur_len] = x;
                }
            }
            while cur_len + 1 < l_start.len() {
                cur_len += 1;
                l_start[cur_len] = parent_nr.len();
            }
        }

        let size = parent_nr.len();
        let mut data: Vec<Vec<BlockFields>> =
            (0..folded_rank).map(|_| Vec::with_capacity(size)).collect();

        for &z in parent_nr.iter() {
            for (oi, orbit) in orbits.iter().enumerate() {
                let s = orbit.s0;
                let t = orbit.s1;
                let (kind, parent_link) = extended_type(parent, z, *orbit, fixed_points);
                let mut link = parent_link;
                let mut second = None; // these index parent block elements

                if !kind.is_like_compact() && !kind.is_like_nonparity() {
                    // Now maybe set |second|, depending on case
                    // (ext_block.cpp:731-813).
                    match kind {
                        // Cases where the second link is the cross neighbour
                        // for |s|.
                        DescValue::OneImaginarySingle | DescValue::OneRealSingle => {
                            second = parent.cross(z, s);
                        }
                        // Cases where the second link is the second Cayley
                        // image, cross of |link|.
                        DescValue::OneRealPairFixed | DescValue::OneImaginaryPairFixed => {
                            if let Some(l) = link {
                                second = parent.cross(l, s);
                            }
                        }
                        // Cases where the second link is the double cross
                        // neighbour for |s| of |z|.
                        DescValue::TwoImaginarySingleSingle | DescValue::TwoRealSingleSingle => {
                            match parent.cross(z, s) {
                                Some(tmp) => {
                                    second = parent.cross(tmp, t);
                                    debug_assert!(
                                        parent.cross(z, t).is_none()
                                            || second
                                                == parent
                                                    .cross(z, t)
                                                    .and_then(|u| parent.cross(u, s))
                                    );
                                }
                                // Try the alternative route.
                                None => match parent.cross(z, t) {
                                    Some(tmp) => second = parent.cross(tmp, s),
                                    // Try to pass from above.
                                    None if kind == DescValue::TwoRealSingleSingle => {
                                        let tmp = parent
                                            .inverse_cayley(z, s)
                                            .expect("inverse Cayley in range")
                                            .0
                                            .and_then(|u| parent.cross(u, t));
                                        if let Some(tmp) = tmp {
                                            let pair =
                                                parent.cayley(tmp, s).expect("Cayley in range");
                                            second = match pair.0 {
                                                Some(first) if fixed_points[first] => pair.0,
                                                _ => pair.1,
                                            };
                                        }
                                    }
                                    // For |TwoImaginarySingleSingle| a nasty
                                    // case remains: leave |second| undefined.
                                    None => {}
                                },
                            }
                        }
                        // Pair-to-pair link cases; the second link is the
                        // second Cayley image, and sort.
                        DescValue::TwoImaginarySingleDoubleFixed
                        | DescValue::TwoRealSingleDoubleFixed => {
                            if let Some(l) = link {
                                second = parent.cross(l, s); // second Cayley is cross of first
                                debug_assert!(second == parent.cross(l, t));
                                if second.is_some() && Some(l) > second {
                                    // Order both by block number (for now).
                                    std::mem::swap(&mut link, &mut second);
                                }
                            }
                        }
                        // Cases where the second link is the second Cayley
                        // image, double cross of |link|.
                        DescValue::TwoImaginaryDoubleDouble | DescValue::TwoRealDoubleDouble => {
                            if let Some(l) = link {
                                match parent.cross(l, s) {
                                    Some(tmp) => {
                                        second = parent.cross(tmp, t);
                                        debug_assert!(
                                            parent.cross(l, t).is_none()
                                                || second
                                                    == parent
                                                        .cross(l, t)
                                                        .and_then(|u| parent.cross(u, s))
                                        );
                                    }
                                    None => match parent.cross(l, t) {
                                        Some(tmp) => second = parent.cross(tmp, s),
                                        None => {
                                            if let Some(tmp) =
                                                parent.cayley(z, s).expect("Cayley in range").1
                                            {
                                                // In the
                                                // |TwoImaginaryDoubleDouble|
                                                // case, try again from above.
                                                let pair =
                                                    parent.cayley(tmp, t).expect("Cayley in range");
                                                if let Some(first) = pair.0 {
                                                    second = if fixed_points[first] {
                                                        pair.0
                                                    } else {
                                                        pair.1
                                                    };
                                                }
                                            }
                                        }
                                    },
                                }
                                if second.is_some() && Some(l) > second {
                                    // Rank a single undefined link second.
                                    std::mem::swap(&mut link, &mut second);
                                }
                            }
                        }
                        _ => {}
                    }
                }

                // Enter the translations of |link| and |second| into child
                // block numbering (ext_block.cpp:815-820).
                let child = |element: Option<usize>| -> Option<usize> {
                    element.map(|e| {
                        let n = child_nr[e];
                        debug_assert!(n != usize::MAX, "link target must be fixed");
                        n
                    })
                };
                data[oi].push(BlockFields {
                    kind,
                    link: child(link),
                    second: child(second),
                });
            }
        }

        ExtBlock {
            orbits,
            zs: parent_nr,
            flips: vec![[0u32; 2]; size],
            data,
            l_start,
            folded,
        }
    }

    /// Upstream `ext_block::rank`: the number of orbits.
    pub fn rank(&self) -> usize {
        self.orbits.len()
    }

    /// Upstream `ext_block::size`.
    pub fn size(&self) -> usize {
        self.zs.len()
    }

    /// Upstream `ext_block::orbit(s)`.
    pub fn orbit(&self, s: usize) -> ExtGen {
        self.orbits[s]
    }

    /// Upstream `ext_block::folded_generators`.
    pub fn folded_generators(&self) -> &[ExtGen] {
        &self.orbits
    }

    /// The folded Cartan matrix on the orbits (upstream
    /// `ext_block::folded_diagram`, as an integer matrix).
    pub fn folded_cartan(&self) -> &[Vec<i32>] {
        &self.folded
    }

    /// Upstream `ext_block::z(n)`: the parent index of extended element
    /// `n`.
    pub fn z(&self, n: usize) -> usize {
        self.zs[n]
    }

    /// Upstream `ext_block::element` (ext_block.cpp:1877-1891): the
    /// smallest `n` with `z(n) >= zz`, or `size()` if none.
    pub fn element(&self, zz: usize) -> usize {
        self.zs.partition_point(|&z| z < zz)
    }

    /// Upstream `ext_block::is_present` (ext_block.h:181-182).
    pub fn is_present(&self, zz: usize) -> bool {
        let n = self.element(zz);
        n < self.size() && self.z(n) == zz
    }

    /// Upstream `ext_block::descent_type`.
    pub fn descent_type(&self, s: usize, n: usize) -> DescValue {
        self.data[s][n].kind
    }

    /// Upstream `ext_block::length` — same as `parent.length(z(n))`;
    /// recovered from `l_start` like ext_block.cpp:1894-1907.
    pub fn length(&self, n: usize) -> usize {
        self.l_start.partition_point(|&start| start <= n) - 1
    }

    /// Upstream `ext_block::l`.
    pub fn l(&self, y: usize, x: usize) -> usize {
        self.length(y) - self.length(x)
    }

    /// Upstream `ext_block::length_first`.
    pub fn length_first(&self, l: usize) -> usize {
        self.l_start[l]
    }

    /// Upstream `ext_block::cross` (ext_block.cpp:1909-1949).
    pub fn cross(&self, s: usize, n: usize) -> Option<usize> {
        use DescValue::*;
        match self.descent_type(s, n) {
            OneComplexAscent | OneComplexDescent | TwoComplexAscent | TwoComplexDescent
            | ThreeComplexAscent | ThreeComplexDescent => self.data[s][n].link,
            // Zero-valued Cayleys and double-valued Cayleys have trivial
            // cross actions; semi and single-double-fixed cases have
            // back-and-forth cross actions.
            OneRealNonparity
            | OneImaginaryCompact
            | OneImaginaryPairSwitched
            | OneRealPairSwitched
            | TwoRealNonparity
            | TwoImaginaryCompact
            | TwoImaginarySingleDoubleSwitched
            | TwoRealSingleDoubleSwitched
            | ThreeRealNonparity
            | ThreeImaginaryCompact
            | OneRealPairFixed
            | OneImaginaryPairFixed
            | TwoRealDoubleDouble
            | TwoImaginaryDoubleDouble
            | TwoSemiImaginary
            | TwoSemiReal
            | TwoImaginarySingleDoubleFixed
            | TwoRealSingleDoubleFixed
            | ThreeSemiImaginary
            | ThreeRealSemi
            | ThreeImaginarySemi
            | ThreeSemiReal => Some(n),
            // Some single-valued extended Cayleys use the second link for
            // the cross action.
            OneImaginarySingle | OneRealSingle | TwoImaginarySingleSingle | TwoRealSingleSingle => {
                self.data[s][n].second
            }
        }
    }

    /// Upstream `ext_block::Cayley` (ext_block.cpp:1951-1954): just one or
    /// none.
    pub fn cayley(&self, s: usize, n: usize) -> Option<usize> {
        if self.descent_type(s, n).is_complex() {
            None
        } else {
            self.data[s][n].link
        }
    }

    /// Upstream `ext_block::Cayleys` (ext_block.cpp:1956-1960): must be
    /// two.
    pub fn cayleys(&self, s: usize, n: usize) -> (Option<usize>, Option<usize>) {
        debug_assert!(self.descent_type(s, n).has_double_image());
        (self.data[s][n].link, self.data[s][n].second)
    }

    /// Upstream `ext_block::some_scent` (ext_block.h:197-198): an ascent or
    /// descent of `n`, assumed to exist.
    pub fn some_scent(&self, s: usize, n: usize) -> Option<usize> {
        self.data[s][n].link
    }

    /// Upstream `ext_block::epsilon` (ext_block.cpp:2255-2262): whether the
    /// link for `s` from `x` to `y` has a sign flip attached.
    pub fn epsilon(&self, s: usize, x: usize, y: usize) -> i32 {
        let fields = &self.data[s][x];
        let i = if fields.link == Some(y) {
            0
        } else {
            debug_assert!(fields.second == Some(y));
            1
        };
        if self.flips[x][i] & (1 << s) != 0 {
            -1
        } else {
            1
        }
    }

    /// Upstream `ext_block::flip_edge` (ext_block.cpp:2246-2252).
    pub fn flip_edge(&mut self, s: usize, x: usize, y: usize) {
        let fields = &self.data[s][x];
        let i = if fields.link == Some(y) {
            0
        } else if fields.second == Some(y) {
            1
        } else {
            panic!("flip_edge: {y} is not a link of ({s},{x})");
        };
        self.flips[x][i] ^= 1 << s;
    }

    /// Upstream `reduce_to` (ext_block.cpp:1963-1970) exposed as
    /// `ext_block::singular_orbits` (ext_block.h:203-205): flag the orbits
    /// whose members are flagged in `singular`.
    pub fn singular_orbits(&self, singular: &RankFlags) -> RankFlags {
        let mut result = RankFlags::empty();
        for (s, orbit) in self.orbits.iter().enumerate() {
            if singular.is_set(orbit.s0) {
                result.set(s);
            }
        }
        result
    }

    /// Upstream `ext_block::first_descent_among`
    /// (ext_block.cpp:1973-1981).
    pub fn first_descent_among(&self, singular_orbits: &RankFlags, y: usize) -> Option<usize> {
        (0..self.rank())
            .find(|&s| singular_orbits.is_set(s) && self.descent_type(s, y).is_descent())
    }

    /// Upstream `ext_block::down_set` (ext_block.cpp:2264-2278).
    pub fn down_set(&self, n: usize) -> Vec<usize> {
        let mut result = Vec::with_capacity(self.rank());
        for s in 0..self.rank() {
            let kind = self.descent_type(s, n);
            if kind.is_descent() && !kind.is_like_compact() {
                if let Some(first) = self.data[s][n].link {
                    result.push(first);
                }
                if kind.has_double_image() {
                    if let Some(second) = self.data[s][n].second {
                        result.push(second);
                    }
                }
            }
        }
        result
    }

    /// Upstream `ext_block::add_neighbours` (ext_block.cpp:2127-2138): all
    /// elements reached by a link are appended to `dst`, ascent/descent
    /// first; the result tells whether the edge of the block was hit (too
    /// few added).
    pub fn add_neighbours(&self, dst: &mut Vec<usize>, s: usize, n: usize) -> bool {
        let fields = &self.data[s][n];
        match fields.link {
            None => return 0 < fields.kind.link_count(),
            Some(first) => dst.push(first),
        }
        match fields.second {
            None => return 1 < fields.kind.link_count(),
            Some(second) => dst.push(second),
        }
        false
    }

    /// Upstream `ext_block::T_coef` (ext_block.cpp:2051-2105): the
    /// coefficient of neighbour `sx` of `x` in the action `(T_s+1)*a_x`.
    pub fn t_coef(&self, s: usize, sx: usize, x: usize) -> SPol {
        let v = self.descent_type(s, x);
        if !v.is_descent() {
            if x == sx {
                // Diagonal coefficient.
                if v.has_defect() {
                    return SPol::monomial(1, 1) + SPol::constant(1); // q+1
                }
                // 0, 1, or 2.
                return SPol::constant(if v.is_like_nonparity() {
                    0
                } else if v.has_double_image() {
                    2
                } else {
                    1
                });
            }
            if v.is_like_type_1() && Some(sx) == self.cross(s, x) {
                // Type 1 imaginary cross.
                let y = self.cayley(s, x).expect("t_coef: type 1 Cayley");
                let sign = self.epsilon(s, x, y) * self.epsilon(s, sx, y);
                return SPol::constant(i64::from(sign));
            }
            // Below-diagonal coefficient.
            let fields = &self.data[s][x];
            debug_assert!(fields.link == Some(sx) || fields.second == Some(sx));
            let sign = self.epsilon(s, x, sx);
            if v.has_defect() {
                // ±(q+1).
                return SPol::monomial(1, i64::from(sign)) + SPol::constant(i64::from(sign));
            }
            return SPol::constant(i64::from(sign)); // ±1
        }

        let k = self.orbit(s).length();
        let mut result = SPol::monomial(k, 1); // start with q^k
        if x == sx {
            // Diagonal coefficient.
            if v.has_double_image() {
                result.set(0, -1); // q^k - 1
            } else if v.is_like_compact() {
                result.set(0, 1); // q^k + 1
            } else if v.has_defect() {
                result.set(1, -1); // q^k - q
            }
            // Else leave q^k.
        } else if v.is_like_type_2() && Some(sx) == self.cross(s, x) {
            // Type 2 real cross.
            let y = self.cayley(s, x).expect("t_coef: type 2 Cayley");
            let sign = self.epsilon(s, y, x) * self.epsilon(s, y, sx);
            return SPol::constant(i64::from(-sign)); // forget q^k, return ∓1
        } else {
            // Remaining cases involve a descending edge (above-diagonal
            // coefficient).
            let fields = &self.data[s][x];
            debug_assert!(fields.link == Some(sx) || fields.second == Some(sx));
            if !v.is_complex() {
                result.set(if v.has_defect() { 1 } else { 0 }, -1); // q^k-1 or q^k-q
            }
            result = result * SPol::constant(i64::from(self.epsilon(s, x, sx)));
        }
        result
    }

    /// Upstream `ext_block::tune_signs` (ext_block.cpp:1707-1876), for the
    /// trivial block modifier (so `bm.simple_pi` is the identity and
    /// `simply_ints[p.s0]` is the parent datum's root number of the orbit's
    /// first member: the simple-root numbers in increasing order for a full
    /// parent, [`IntegralSubsystem::parent_root`] values for a
    /// [`PartialBlock`] parent). The `star` computation and the `ext_param`
    /// values are injected through `oracle`; see [`StarOracle`]. Returns
    /// `false` where upstream returns `false` or throws (a type mismatch,
    /// or a quadratic/braid relation failure under `debug_assertions`).
    pub fn tune_signs<O: StarOracle>(&mut self, oracle: &mut O, simply_ints: &[usize]) -> bool {
        for n in 0..self.size() {
            let z = self.z(n); // element number in the parent block
            let e = oracle.def_ext(z);
            // With the trivial modifier, bm.simple_pi is the identity.
            for s in 0..self.rank() {
                let p = self.orbit(s);
                let n_alpha = simply_ints[p.s0];
                let (tp, links) = oracle.star(&e, p.length(), n_alpha);
                if self.descent_type(s, n).might_be_uncertain() && self.data[s][n].link.is_none() {
                    // Reset the uncertain type, leave possible links
                    // undefined; there are no link signs to tune.
                    self.data[s][n].kind = tp;
                    continue;
                } else if tp != self.descent_type(s, n) {
                    return false; // something is wrong
                }

                match tp {
                    // Cases with no links at all.
                    DescValue::OneImaginaryPairSwitched
                    | DescValue::OneRealPairSwitched
                    | DescValue::OneRealNonparity
                    | DescValue::OneImaginaryCompact
                    | DescValue::TwoImaginarySingleDoubleSwitched
                    | DescValue::TwoRealSingleDoubleSwitched
                    | DescValue::TwoRealNonparity
                    | DescValue::TwoImaginaryCompact
                    | DescValue::ThreeRealNonparity
                    | DescValue::ThreeImaginaryCompact => {
                        debug_assert!(links.is_empty());
                    }

                    DescValue::OneComplexAscent
                    | DescValue::OneComplexDescent
                    | DescValue::TwoComplexAscent
                    | DescValue::TwoComplexDescent
                    | DescValue::ThreeComplexAscent
                    | DescValue::ThreeComplexDescent => {
                        debug_assert_eq!(links.len(), 1);
                        let q = &links[0];
                        // Cross neighbour as bare element of |self|.
                        let Some(m) = self.cross(s, n) else {
                            continue; // don't fall off the edge of a partial block
                        };
                        let cz = self.z(m); // corresponding parent element
                        let f = oracle.def_ext(cz);
                        debug_assert!(oracle.same_standard_reps(q, &f));
                        if !oracle.same_sign(q, &f) {
                            self.flip_edge(s, n, m);
                        }
                    }

                    DescValue::OneImaginarySingle
                    | DescValue::OneRealSingle
                    | DescValue::TwoImaginarySingleSingle
                    | DescValue::TwoRealSingleSingle => {
                        debug_assert_eq!(links.len(), 2);
                        let q0 = &links[0];
                        let q1 = &links[1];
                        // The unique (inverse) Cayley.
                        if let Some(m) = self.some_scent(s, n) {
                            let cz = self.z(m);
                            let f = oracle.def_ext(cz);
                            debug_assert!(oracle.same_standard_reps(q0, &f));
                            if !oracle.same_sign(q0, &f) {
                                self.flip_edge(s, n, m);
                            }
                        }
                        // The cross link; don't fall off the edge.
                        if let Some(m) = self.cross(s, n) {
                            let cz = self.z(m);
                            let fc = oracle.def_ext(cz);
                            debug_assert!(oracle.same_standard_reps(q1, &fc));
                            if !oracle.same_sign(q1, &fc) {
                                self.flip_edge(s, n, m);
                            }
                        }
                    }

                    DescValue::TwoSemiImaginary
                    | DescValue::TwoSemiReal
                    | DescValue::ThreeSemiImaginary
                    | DescValue::ThreeRealSemi
                    | DescValue::ThreeImaginarySemi
                    | DescValue::ThreeSemiReal => {
                        debug_assert_eq!(links.len(), 1);
                        let q = &links[0];
                        // The unique (inverse) Cayley.
                        let Some(m) = self.some_scent(s, n) else {
                            continue; // don't fall off the edge
                        };
                        let cz = self.z(m);
                        let f = oracle.def_ext(cz);
                        debug_assert!(oracle.same_standard_reps(q, &f));
                        if !oracle.same_sign(q, &f) {
                            self.flip_edge(s, n, m);
                        }
                    }

                    DescValue::OneImaginaryPairFixed
                    | DescValue::OneRealPairFixed
                    | DescValue::TwoImaginaryDoubleDouble
                    | DescValue::TwoRealDoubleDouble
                    | DescValue::TwoImaginarySingleDoubleFixed
                    | DescValue::TwoRealSingleDoubleFixed => {
                        debug_assert_eq!(links.len(), 2);
                        let q0 = &links[0];
                        let q1 = &links[1];
                        let m = self.cayleys(s, n);

                        let Some(first) = m.0 else {
                            continue; // nothing to do if both are undefined
                        };

                        let cz = self.z(first);
                        let f0 = oracle.def_ext(cz);
                        let straight = oracle.same_standard_reps(q0, &f0);
                        let node0 = if straight { q0 } else { q1 };
                        debug_assert!(oracle.same_standard_reps(node0, &f0));
                        if !oracle.same_sign(node0, &f0) {
                            self.flip_edge(s, n, first);
                        }

                        let Some(second) = m.1 else {
                            continue;
                        };

                        let cz = self.z(second);
                        let f1 = oracle.def_ext(cz);
                        let node1 = if straight { q1 } else { q0 };
                        debug_assert!(oracle.same_standard_reps(node1, &f1));
                        if !oracle.same_sign(node1, &f1) {
                            self.flip_edge(s, n, second);
                        }
                    }
                }
            }
        }
        // When debugging, test the quadratic and braid relations for the
        // extended block (upstream's `#ifndef NDEBUG` block at
        // ext_block.cpp:1858-1871).
        #[cfg(debug_assertions)]
        {
            for x in 0..self.size() {
                for s in 0..self.rank() {
                    if !check_quadratic(self, s, x) {
                        return false;
                    }
                    for t in s + 1..self.rank() {
                        if !check_braid(self, s, t, x) {
                            return false;
                        }
                    }
                }
            }
        }
        true // report success if we get here
    }
}

/// The operations `tune_signs` needs from the `ext_param`/`star` slice
/// (ext_block.cpp:1707-1876): the default extension of each parent element
/// (`ext_param::def_ext`), the per-generator `star` computation
/// (ext_block.cpp:990-1705, returning the recomputed type and the adjacent
/// extended parameters), and the two comparisons `same_standard_reps`
/// (ext_block.cpp:918-931) and `same_sign` (ext_block.cpp:936-948). The
/// parameter type itself is opaque to this module.
pub trait StarOracle {
    /// The extended-parameter type (upstream `ext_block::ext_param`).
    type Param;
    /// Upstream `ext_param::def_ext(ctxt, bm, block.representative(z))`:
    /// the default extension at parent block element `z`.
    fn def_ext(&mut self, z: usize) -> Self::Param;
    /// Upstream `star(ctxt, E, orbit_length, n_alpha, links)`: recompute
    /// the extended type of `e` for the orbit root `n_alpha` (a root number
    /// of the parent datum) and export the adjacent parameters in the same
    /// order upstream pushes them (Cayley link(s) first, cross link last
    /// for the single-valued types).
    fn star(
        &mut self,
        e: &Self::Param,
        orbit_length: usize,
        n_alpha: usize,
    ) -> (DescValue, Vec<Self::Param>);
    /// Upstream `same_standard_reps` (ext_block.cpp:918-931).
    fn same_standard_reps(&self, a: &Self::Param, b: &Self::Param) -> bool;
    /// Upstream `same_sign` (ext_block.cpp:936-948).
    fn same_sign(&self, a: &Self::Param, b: &Self::Param) -> bool;
}

/// Upstream `ext_block::induced` (ext_block.cpp:670-693): the permutation
/// induced from the orbits to the `simple_pi`-image orbits. Standalone
/// because the trivial-modifier constructor never permutes generators; the
/// nontrivial-`bm` path will need it together with the generator
/// permutation at ext_block.cpp:639-663.
pub fn induced(orbits: &[ExtGen], simple_pi: &[usize]) -> Vec<usize> {
    let mut orbit_of = vec![usize::MAX; simple_pi.len()];
    for (s, orbit) in orbits.iter().enumerate() {
        orbit_of[orbit.s0] = s;
        if orbit.is_orbit_pair() {
            orbit_of[orbit.s1] = s;
        }
    }

    // The inverse permutation.
    let mut inv = vec![0usize; simple_pi.len()];
    for (i, &image) in simple_pi.iter().enumerate() {
        inv[image] = i;
    }

    let mut result = vec![0usize; orbits.len()];
    let mut seen = vec![false; simple_pi.len()];
    let mut count = 0usize;
    for i in 0..simple_pi.len() {
        if !seen[i] {
            let s = orbit_of[inv[i]];
            result[s] = count;
            count += 1;
            seen[simple_pi[orbits[s].s0]] = true;
            if orbits[s].is_orbit_pair() {
                seen[simple_pi[orbits[s].s1]] = true;
            }
            debug_assert!(seen[i]); // one of the two assignments ensured this
        }
    }
    result
}

/// Upstream `Permutation::permute` on a vector (permutations_def.h:62-80):
/// forward push, `v'[pi[i]] = v[i]`.
fn push_permute<T: Clone>(pi: &[usize], v: &mut [T]) {
    let old = v.to_owned();
    for (index, &image) in pi.iter().enumerate() {
        v[image] = old[index].clone();
    }
}

/// Upstream `check_quadratic` (ext_block.cpp:2140-2173): check the
/// quadratic relation for `s` at `x0`. Returns `true` when there is
/// nothing to check (edge of a partial block, or no links).
pub fn check_quadratic(block: &ExtBlock, s: usize, x0: usize) -> bool {
    let mut l = Vec::new();
    if block.add_neighbours(&mut l, s, x0) {
        return true;
    }
    if l.is_empty() {
        // Compact or nonparity cases: there is nothing to check.
        return true;
    }

    // Check symmetry of link signs.
    for &y in &l {
        if block.epsilon(s, x0, y) != block.epsilon(s, y, x0) {
            return false;
        }
    }

    let tp = block.descent_type(s, x0);
    if l.len() == 1 {
        // Cases without cycles; we're done.
        return true;
    }
    debug_assert_eq!(l.len(), 2);
    let y0 = l[0];
    let y1 = l[1];

    if tp.has_quadruple() {
        let mut l = Vec::new();
        if block.add_neighbours(&mut l, s, y0) {
            return true;
        }
        if x0 == l[0] {
            l.remove(0); // so that |l[0]| won't be |x0|
        }
        let x1 = l[0];
        debug_assert_ne!(x0, x1);
        // Negative product here!
        block.epsilon(s, x0, y0) * block.epsilon(s, x0, y1)
            != block.epsilon(s, y0, x1) * block.epsilon(s, y1, x1)
    } else {
        block.epsilon(s, x0, y0) * block.epsilon(s, x0, y1) == block.epsilon(s, y0, y1)
    }
}

/// Upstream `check_braid` (ext_block.cpp:2175-2244): check the braid
/// relation at `x` for the orbit generators `s` and `t`. Elements of the
/// connected `{s,t}`-cluster are pushed onto `cluster` when given.
pub fn check_braid(block: &ExtBlock, s: usize, t: usize, x: usize) -> bool {
    if s == t {
        return true;
    }
    const COX_ENTRY: [usize; 4] = [2, 3, 4, 6];
    let multiplicity = (block.folded[s][t] * block.folded[t][s]) as usize;
    let len = COX_ENTRY[multiplicity];

    // The connected cluster of |x|, in increasing order (upstream BitMap
    // iteration order).
    let mut used: Vec<usize> = Vec::new();
    let mut to_do = std::collections::VecDeque::from([x]);
    while let Some(z) = to_do.pop_front() {
        if used.contains(&z) {
            continue;
        }
        // Insert keeping |used| sorted.
        let position = used.partition_point(|&u| u < z);
        used.insert(position, z);
        let mut l = Vec::new();
        if block.add_neighbours(&mut l, s, z) || block.add_neighbours(&mut l, t, z) {
            return true;
        }
        for y in l {
            if !used.contains(&y) {
                to_do.push_back(y);
            }
        }
    }

    let n = used.len();
    let position_of = |z: usize| used.partition_point(|&u| u < z);
    let mut ts = vec![vec![SPol::zero(); n]; n];
    let mut tt = vec![vec![SPol::zero(); n]; n];

    for (j, &y) in used.iter().enumerate() {
        ts[j][j] = block.t_coef(s, y, y) - SPol::constant(1);
        tt[j][j] = block.t_coef(t, y, y) - SPol::constant(1);
        let mut l = Vec::new();
        if block.add_neighbours(&mut l, s, y) {
            return true;
        }
        for z in l {
            if used.contains(&z) {
                ts[position_of(z)][j] = block.t_coef(s, z, y);
            }
        }
        let mut l = Vec::new();
        if block.add_neighbours(&mut l, t, y) {
            return true;
        }
        for z in l {
            if used.contains(&z) {
                tt[position_of(z)][j] = block.t_coef(t, z, y);
            }
        }
    }

    let mut v = vec![SPol::zero(); n];
    v[position_of(x)] = SPol::constant(1);
    let mut w = v.clone();

    // Finally compute the braid relation.
    for i in 0..len {
        if i % 2 == 0 {
            v = apply(&ts, &v);
            w = apply(&tt, &w);
        } else {
            v = apply(&tt, &v);
            w = apply(&ts, &w);
        }
    }

    v == w
}

/// Matrix-times-vector over [`SPol`].
fn apply(matrix: &[Vec<SPol>], vector: &[SPol]) -> Vec<SPol> {
    let n = vector.len();
    let mut result = vec![SPol::zero(); n];
    for (i, row) in result.iter_mut().enumerate() {
        for (j, entry) in vector.iter().enumerate() {
            *row = std::mem::take(row) + matrix[i][j].clone() * entry.clone();
        }
    }
    result
}

/// A signed Laurent-free polynomial in `q` with integer coefficients
/// (upstream `ext_block::Pol = Polynomial<int>`), only as much as
/// `T_coef`/`check_braid` need: coefficient of `q^i` at index `i`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SPol(Vec<i64>);

impl SPol {
    pub fn zero() -> Self {
        SPol(Vec::new())
    }

    pub fn constant(c: i64) -> Self {
        if c == 0 {
            SPol::zero()
        } else {
            SPol(vec![c])
        }
    }

    pub fn monomial(degree: usize, coefficient: i64) -> Self {
        if coefficient == 0 {
            return SPol::zero();
        }
        let mut coefficients = vec![0; degree + 1];
        coefficients[degree] = coefficient;
        SPol(coefficients)
    }

    /// The raw coefficient vector (coefficient of `q^i` at index `i`);
    /// `ext_kl::product_comp` needs it to convert a `T_coef` result into
    /// the `KlPol` world (ext_kl.cpp:209-212).
    pub fn as_slice(&self) -> &[i64] {
        &self.0
    }

    /// Set the coefficient of `q^degree` (upstream `result[k]=c`).
    fn set(&mut self, degree: usize, coefficient: i64) {
        if self.0.len() <= degree {
            self.0.resize(degree + 1, 0);
        }
        self.0[degree] = coefficient;
    }

    fn normalized(mut self) -> Self {
        while self.0.last() == Some(&0) {
            self.0.pop();
        }
        self
    }
}

impl std::ops::Add for SPol {
    type Output = SPol;
    fn add(self, other: SPol) -> SPol {
        let mut coefficients = self.0;
        coefficients.resize(coefficients.len().max(other.0.len()), 0);
        for (i, c) in other.0.iter().enumerate() {
            coefficients[i] += c;
        }
        SPol(coefficients).normalized()
    }
}

impl std::ops::Sub for SPol {
    type Output = SPol;
    fn sub(self, other: SPol) -> SPol {
        self + (-other)
    }
}

impl std::ops::Neg for SPol {
    type Output = SPol;
    fn neg(self) -> SPol {
        SPol(self.0.into_iter().map(|c| -c).collect())
    }
}

impl std::ops::Mul for SPol {
    type Output = SPol;
    fn mul(self, other: SPol) -> SPol {
        if self.0.is_empty() || other.0.is_empty() {
            return SPol::zero();
        }
        let mut coefficients = vec![0i64; self.0.len() + other.0.len() - 1];
        for (i, &a) in self.0.iter().enumerate() {
            for (j, &b) in other.0.iter().enumerate() {
                coefficients[i + j] += a * b;
            }
        }
        SPol(coefficients).normalized()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AdjointFiberBudget, BasedRootDatum, BlockLocator, CartanClassification,
        CartanClassificationBudget, CartanId, Coweight, InnerClass, IntegerLatticeBudget,
        InvolutionTableBudget, KgbId, RealFormSeed, RepContext, RootId, StrongRealClassification,
        WeakRealFormId, Weight, WeylElement,
    };

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

    /// The KGB graph of the class's real form whose KGB size is `size`
    /// (same helper shape as block.rs/kl_table.rs tests).
    fn graph_with_size(
        inner_class: &InnerClass,
        classification: &CartanClassification,
        strong: &StrongRealClassification,
        table: &mut InvolutionTable,
        size: usize,
    ) -> (KgbGraph, InvolutionTable) {
        for form in 0..classification.weak_real_form_count() {
            if strong.kgb_size(WeakRealFormId(form)) != Some(size) {
                continue;
            }
            table.add_cartan(classification, CartanId(0)).unwrap();
            let seed = RealFormSeed::build(
                inner_class,
                classification,
                strong,
                table,
                WeakRealFormId(form),
                &IntegerLatticeBudget::new(64, 100_000, 100_000, 128),
                4_096,
            )
            .unwrap();
            let graph = KgbGraph::build(inner_class, classification, strong, table, &seed).unwrap();
            return (graph, table.clone());
        }
        panic!("no real form with KGB size {size}");
    }

    struct BlockFixture {
        primal_class: InnerClass,
        dual_class: InnerClass,
        graph: KgbGraph,
        table: InvolutionTable,
        dual_graph: KgbGraph,
        dual_table: InvolutionTable,
        block: BlockGraph,
    }

    /// Build the block of the primal form with KGB size `primal_size`
    /// against the dual-class form with KGB size `dual_size`.
    fn fixture(
        primal_class: InnerClass,
        primal_size: usize,
        dual_size: usize,
        weyl: usize,
    ) -> BlockFixture {
        let classification =
            CartanClassification::build(&primal_class, &class_budget(weyl)).unwrap();
        let strong = StrongRealClassification::build(&classification, 4_096).unwrap();
        let mut table = InvolutionTable::new(
            &primal_class,
            InvolutionTableBudget::new(64, IntegerLatticeBudget::new(64, 100_000, 100_000, 128)),
        )
        .unwrap();
        let (graph, primal_table) = graph_with_size(
            &primal_class,
            &classification,
            &strong,
            &mut table,
            primal_size,
        );

        let dual_class = crate::dual::dual_inner_class(&primal_class, weyl, 64).unwrap();
        let dual_classification =
            CartanClassification::build(&dual_class, &class_budget(weyl)).unwrap();
        let dual_strong = StrongRealClassification::build(&dual_classification, 4_096).unwrap();
        let mut dual_table = InvolutionTable::new(
            &dual_class,
            InvolutionTableBudget::new(64, IntegerLatticeBudget::new(64, 100_000, 100_000, 128)),
        )
        .unwrap();
        let (dual_graph, dual_table) = graph_with_size(
            &dual_class,
            &dual_classification,
            &dual_strong,
            &mut dual_table,
            dual_size,
        );
        let block = BlockGraph::build(
            &graph,
            &primal_table,
            &dual_graph,
            &dual_table,
            &dual_class,
            weyl,
        )
        .unwrap();
        BlockFixture {
            primal_class,
            dual_class,
            graph,
            table: primal_table,
            dual_graph,
            dual_table,
            block,
        }
    }

    /// The A1 block of the SL(2,R) side (KGB 3) against the PGL(2,R) side
    /// (KGB 2): 3 elements, as anchored in block.rs tests.
    fn a1_block() -> BlockFixture {
        let datum = BasedRootDatum::from_simple_data(
            1,
            vec![vec![2]],
            vec![Weight::new(vec![2])],
            vec![Coweight::new(vec![1])],
        )
        .unwrap();
        let involution = LatticeInvolution::identity(&datum).unwrap();
        fixture(InnerClass::new(datum, involution, 2).unwrap(), 3, 2, 2)
    }

    /// The A2 root-lattice datum used by the kl_table.rs su(2,1) anchors.
    fn a2_datum() -> BasedRootDatum {
        BasedRootDatum::from_simple_data(
            2,
            vec![vec![2, -1], vec![-1, 2]],
            vec![Weight::new(vec![2, -1]), Weight::new(vec![-1, 2])],
            vec![Coweight::new(vec![1, 0]), Coweight::new(vec![0, 1])],
        )
        .unwrap()
    }

    /// The equal-rank A2 inner class (distinguished = identity), su(2,1)
    /// primal KGB size 6, dual class form size 4: the kl_table.rs block.
    fn a2_equal_rank_block() -> BlockFixture {
        let datum = a2_datum();
        let involution = LatticeInvolution::identity(&datum).unwrap();
        fixture(InnerClass::new(datum, involution, 8).unwrap(), 6, 4, 8)
    }

    /// The flipped A2 inner class (distinguished = diagram flip): the
    /// quasisplit sl(3,R) form has KGB size 4; the dual class is the
    /// equal-rank one, whose quasisplit su(2,1) form has KGB size 6.
    fn a2_flipped_block() -> BlockFixture {
        let datum = a2_datum();
        let flip = LatticeInvolution::new(
            &datum,
            vec![vec![0, 1], vec![1, 0]],
            vec![vec![0, 1], vec![1, 0]],
        )
        .unwrap();
        fixture(InnerClass::new(datum, flip, 8).unwrap(), 4, 6, 8)
    }

    /// Identity twist data: delta = identity on both sides.
    fn identity_twists(
        fixture: &BlockFixture,
    ) -> (LatticeInvolution, Vec<usize>, LatticeInvolution, Vec<usize>) {
        let delta = LatticeInvolution::identity(fixture.primal_class.datum()).unwrap();
        let twist = fixture
            .primal_class
            .based_involution_twist(delta.clone())
            .unwrap();
        let dual_delta = LatticeInvolution::identity(fixture.dual_class.datum()).unwrap();
        let dual_twist = fixture
            .dual_class
            .based_involution_twist(dual_delta.clone())
            .unwrap();
        (delta, twist, dual_delta, dual_twist)
    }

    /// The diagram-flip twist data: delta = flip on both A2 sides (the
    /// transposed flip on the dual datum is again the swap matrix).
    fn flip_twists(
        fixture: &BlockFixture,
    ) -> (LatticeInvolution, Vec<usize>, LatticeInvolution, Vec<usize>) {
        let swap = vec![vec![0, 1], vec![1, 0]];
        let delta =
            LatticeInvolution::new(fixture.primal_class.datum(), swap.clone(), swap.clone())
                .unwrap();
        let twist = fixture
            .primal_class
            .based_involution_twist(delta.clone())
            .unwrap();
        let dual_delta =
            LatticeInvolution::new(fixture.dual_class.datum(), swap.clone(), swap).unwrap();
        let dual_twist = fixture
            .dual_class
            .based_involution_twist(dual_delta.clone())
            .unwrap();
        (delta, twist, dual_delta, dual_twist)
    }

    /// Replicate the `extended_block` wrapper's output tables
    /// (interpreter/atlas-types.w:7411-7428): per element and orbit, the
    /// type index and the two link columns, with `size` for `UndefBlock`
    /// and `-1-link` for a flipped sign.
    fn wrapper_tables(eb: &ExtBlock) -> (Vec<Vec<usize>>, Vec<Vec<isize>>, Vec<Vec<isize>>) {
        let size = eb.size();
        let signed = |eb: &ExtBlock, s: usize, n: usize, link: Option<usize>| -> isize {
            match link {
                None => size as isize,
                Some(m) => {
                    if eb.epsilon(s, n, m) < 0 {
                        -1 - m as isize
                    } else {
                        m as isize
                    }
                }
            }
        };
        let mut types = Vec::new();
        let mut links0 = Vec::new();
        let mut links1 = Vec::new();
        for n in 0..size {
            let mut type_row = Vec::new();
            let mut row0 = Vec::new();
            let mut row1 = Vec::new();
            for s in 0..eb.rank() {
                let kind = eb.descent_type(s, n);
                type_row.push(kind as usize);
                if kind.is_like_compact() || kind.is_like_nonparity() {
                    row0.push(size as isize);
                    row1.push(size as isize);
                } else {
                    let first = if kind.is_complex() {
                        eb.cross(s, n)
                    } else {
                        eb.cayley(s, n)
                    };
                    row0.push(signed(eb, s, n, first));
                    if kind.link_count() == 1 {
                        row1.push(size as isize);
                    } else {
                        let second = if kind.has_double_image() {
                            eb.cayleys(s, n).1
                        } else {
                            eb.cross(s, n)
                        };
                        row1.push(signed(eb, s, n, second));
                    }
                }
            }
            types.push(type_row);
            links0.push(row0);
            links1.push(row1);
        }
        (types, links0, links1)
    }

    /// A stub [`StarOracle`] for the structural `tune_signs` tests: the
    /// "extended parameter" is just the parent element number, `star`
    /// recomputes the stored type and returns the stored links' parent
    /// indices, and every pair has equal sign (so no flips are made). It
    /// exercises the `tune_signs` control flow and the debug quadratic /
    /// braid gates; the genuine sign computation belongs to the
    /// `ext_param`/`star` slice.
    struct StubOracle {
        block: ExtBlock,
    }

    impl StarOracle for StubOracle {
        type Param = usize;
        fn def_ext(&mut self, z: usize) -> usize {
            z
        }
        fn star(
            &mut self,
            e: &usize,
            _orbit_length: usize,
            n_alpha: usize,
        ) -> (DescValue, Vec<usize>) {
            let eb = &self.block;
            let n = eb.element(*e);
            assert!(n < eb.size() && eb.z(n) == *e);
            let s = eb
                .orbits
                .iter()
                .position(|orbit| orbit.s0 == n_alpha)
                .expect("orbit for generator");
            let kind = eb.descent_type(s, n);
            let parent = |link: Option<usize>| link.map(|m| eb.z(m));
            let links = match kind {
                // Zero-link cases.
                _ if kind.link_count() == 0 => Vec::new(),
                // Complex cases: the single cross link.
                _ if kind.is_complex() => vec![parent(eb.cross(s, n)).expect("cross link")],
                // Single-valued Cayleys: Cayley link, then cross link.
                DescValue::OneImaginarySingle
                | DescValue::OneRealSingle
                | DescValue::TwoImaginarySingleSingle
                | DescValue::TwoRealSingleSingle => vec![
                    parent(eb.data[s][n].link).expect("Cayley link"),
                    parent(eb.data[s][n].second).expect("cross link"),
                ],
                // Semi cases: the unique (inverse) Cayley.
                _ if kind.has_defect() => {
                    vec![parent(eb.data[s][n].link).expect("semi link")]
                }
                // Double-valued Cayleys: both images.
                _ if kind.has_double_image() => vec![
                    parent(eb.data[s][n].link).expect("first Cayley"),
                    parent(eb.data[s][n].second).expect("second Cayley"),
                ],
                _ => unreachable!("stub star: uncovered type {kind:?}"),
            };
            (kind, links)
        }
        fn same_standard_reps(&self, a: &usize, b: &usize) -> bool {
            a == b
        }
        fn same_sign(&self, _a: &usize, _b: &usize) -> bool {
            true
        }
    }

    #[test]
    fn a1_trivial_delta_ext_block_structure() {
        let fixture = a1_block();
        let (delta, twist, dual_delta, dual_twist) = identity_twists(&fixture);
        let eb = ExtBlock::build(
            &fixture.block,
            &fixture.graph,
            &fixture.table,
            &fixture.dual_graph,
            &fixture.dual_table,
            &delta,
            &twist,
            &dual_delta,
            &dual_twist,
            &[vec![2]],
        )
        .unwrap();

        // Trivial delta: every element is fixed, one orbit per generator.
        assert_eq!(eb.size(), 3);
        assert_eq!(eb.rank(), 1);
        assert_eq!(eb.orbit(0).kind, ExtGenKind::One);
        assert_eq!(eb.folded_cartan(), &[vec![2]]);
        assert_eq!((0..3).map(|n| eb.z(n)).collect::<Vec<_>>(), vec![0, 1, 2]);

        // Hand-computed from the block.rs anchors: z0,z1 are i1 with
        // Cayley z2 and cross each other; z2 is r1 with images (z0,z1).
        let (types, links0, links1) = wrapper_tables(&eb);
        assert_eq!(
            types,
            vec![
                vec![DescValue::OneImaginarySingle as usize],
                vec![DescValue::OneImaginarySingle as usize],
                vec![DescValue::OneRealPairFixed as usize],
            ]
        );
        assert_eq!(links0, vec![vec![2], vec![2], vec![0]]);
        assert_eq!(links1, vec![vec![1], vec![0], vec![1]]);

        // Lengths: the r1 element has parent length 1.
        assert_eq!(eb.length(0), 0);
        assert_eq!(eb.length(1), 0);
        assert_eq!(eb.length(2), 1);
        assert_eq!(eb.length_first(1), 2);

        // element/is_present translate back to parent numbering.
        assert_eq!(eb.element(2), 2);
        assert!(eb.is_present(1));
        assert!(!eb.is_present(3));
    }

    #[test]
    fn a1_tune_signs_stub_and_flip_edge_gate() {
        let fixture = a1_block();
        let (delta, twist, dual_delta, dual_twist) = identity_twists(&fixture);
        let mut eb = ExtBlock::build(
            &fixture.block,
            &fixture.graph,
            &fixture.table,
            &fixture.dual_graph,
            &fixture.dual_table,
            &delta,
            &twist,
            &dual_delta,
            &dual_twist,
            &[vec![2]],
        )
        .unwrap();

        // The stub finds equal signs everywhere; tune_signs then runs the
        // debug quadratic gate over the unflipped block.
        let simply_ints = vec![0usize];
        let mut oracle = StubOracle { block: eb.clone() };
        assert!(eb.tune_signs(&mut oracle, &simply_ints));
        assert_eq!(eb.epsilon(0, 0, 2), 1);

        // A manually asymmetric flip is caught by check_quadratic.
        eb.flip_edge(0, 0, 2);
        assert_eq!(eb.epsilon(0, 0, 2), -1);
        assert_eq!(eb.epsilon(0, 2, 0), 1);
        assert!(!check_quadratic(&eb, 0, 0));
        eb.flip_edge(0, 0, 2);
        assert!(check_quadratic(&eb, 0, 0));
    }

    #[test]
    fn a2_trivial_delta_matches_oracle() {
        // Oracle: extended_block(trivial(SU(2,1)), id); distinguished of
        // the equal-rank class is the identity, so this is the full block
        // with one orbit per generator (probe at atlas-scripts/groups.at).
        let fixture = a2_equal_rank_block();
        assert_eq!(fixture.block.size(), 6);
        // The block elements are numbered by their primal KGB coordinate.
        for z in 0..6 {
            assert_eq!(fixture.block.x(z).unwrap().index(), z);
        }

        let (delta, twist, dual_delta, dual_twist) = identity_twists(&fixture);
        let cartan = vec![vec![2, -1], vec![-1, 2]];
        let mut eb = ExtBlock::build(
            &fixture.block,
            &fixture.graph,
            &fixture.table,
            &fixture.dual_graph,
            &fixture.dual_table,
            &delta,
            &twist,
            &dual_delta,
            &dual_twist,
            &cartan,
        )
        .unwrap();
        assert_eq!(eb.size(), 6);
        assert_eq!(eb.rank(), 2);
        assert_eq!(eb.folded_cartan(), &cartan);

        let (types, links0, links1) = wrapper_tables(&eb);
        let u = 6_isize; // UndefBlock in the wrapper output
        assert_eq!(
            types,
            vec![
                vec![2, 2], // OneImaginarySingle, OneImaginarySingle
                vec![2, 9], // OneImaginarySingle, OneImaginaryCompact
                vec![9, 2],
                vec![0, 3], // OneComplexAscent, OneRealPairFixed
                vec![3, 0],
                vec![1, 1], // OneComplexDescent, OneComplexDescent
            ]
        );
        assert_eq!(
            links0,
            vec![
                vec![4, 3],
                vec![4, u],
                vec![u, 3],
                vec![5, 0],
                vec![0, 5],
                vec![3, 4],
            ]
        );
        assert_eq!(
            links1,
            vec![
                vec![1, 2],
                vec![0, u],
                vec![u, 0],
                vec![u, 2],
                vec![1, u],
                vec![u, u],
            ]
        );

        // No sign flips for the trivial delta (all wrapper entries
        // positive above); the stub tuner confirms and runs the debug
        // quadratic AND braid gates (folded rank 2, multiplicity 1).
        let simply_ints = vec![0usize, 1];
        let mut oracle = StubOracle { block: eb.clone() };
        assert!(eb.tune_signs(&mut oracle, &simply_ints));
        for n in 0..eb.size() {
            for s in 0..eb.rank() {
                assert!(check_quadratic(&eb, s, n));
            }
        }
    }

    #[test]
    fn a2_flip_delta_matches_oracle() {
        // Oracle: extended_block(trivial(SL(3,R)), flip): the flipped
        // class's distinguished involution folds the two simple roots into
        // one orbit of length 3; the extended block has 2 elements, of
        // types ThreeSemiImaginary and ThreeRealSemi, linked to each
        // other. The parent block is the 6-element block of print_block
        // (trivial(SL(3,R))), whose fixed elements are z=0 (x=0) and z=3
        // (x=3).
        let fixture = a2_flipped_block();
        assert_eq!(fixture.block.size(), 6);

        let (delta, twist, dual_delta, dual_twist) = flip_twists(&fixture);
        assert_eq!(twist, vec![1, 0]);
        let cartan = vec![vec![2, -1], vec![-1, 2]];
        let mut eb = ExtBlock::build(
            &fixture.block,
            &fixture.graph,
            &fixture.table,
            &fixture.dual_graph,
            &fixture.dual_table,
            &delta,
            &twist,
            &dual_delta,
            &dual_twist,
            &cartan,
        )
        .unwrap();

        assert_eq!(eb.rank(), 1);
        let orbit = eb.orbit(0);
        assert_eq!(orbit.kind, ExtGenKind::Three);
        assert_eq!((orbit.s0, orbit.s1), (0, 1));
        assert_eq!(eb.folded_cartan(), &[vec![2]]);

        // Fixed elements: z=0 (x=0) and z=3 (x=3), in parent order.
        assert_eq!(eb.size(), 2);
        assert_eq!(fixture.block.x(eb.z(0)).unwrap().index(), 0);
        assert_eq!(fixture.block.x(eb.z(1)).unwrap().index(), 3);
        assert_eq!(eb.length(0), 0);
        assert_eq!(eb.length(1), 2);

        let (types, links0, links1) = wrapper_tables(&eb);
        let u = 2_isize;
        assert_eq!(
            types,
            vec![
                vec![DescValue::ThreeSemiImaginary as usize], // 26
                vec![DescValue::ThreeRealSemi as usize],      // 27
            ]
        );
        assert_eq!(links0, vec![vec![1], vec![0]]);
        assert_eq!(links1, vec![vec![u], vec![u]]);

        // The stub tuner: single folded generator, so only the quadratic
        // gate runs (both types have a single link; symmetry holds with
        // no flips).
        let simply_ints = vec![0usize, 1];
        let mut oracle = StubOracle { block: eb.clone() };
        assert!(eb.tune_signs(&mut oracle, &simply_ints));
    }

    #[test]
    fn fold_orbits_and_folded_cartan() {
        // A2 flip: one non-commuting pair, folded Cartan [[2]].
        let a2 = vec![vec![2, -1], vec![-1, 2]];
        let orbits = fold_orbits(&a2, &[1, 0]).unwrap();
        assert_eq!(orbits.len(), 1);
        assert_eq!(orbits[0].kind, ExtGenKind::Three);
        assert_eq!(folded_cartan(&a2, &orbits).unwrap(), vec![vec![2]]);

        // A3 reversal: a commuting pair and a singleton fold to B2.
        let a3 = vec![vec![2, -1, 0], vec![-1, 2, -1], vec![0, -1, 2]];
        let orbits = fold_orbits(&a3, &[2, 1, 0]).unwrap();
        assert_eq!(
            orbits,
            vec![
                ExtGen {
                    kind: ExtGenKind::Two,
                    s0: 0,
                    s1: 2
                },
                ExtGen {
                    kind: ExtGenKind::One,
                    s0: 1,
                    s1: usize::MAX
                },
            ]
        );
        assert_eq!(
            folded_cartan(&a3, &orbits).unwrap(),
            vec![vec![2, -1], vec![-2, 2]]
        );

        // A non-involution twist is rejected.
        assert!(fold_orbits(&a3, &[1, 2, 0]).is_err());
    }

    #[test]
    fn induced_permutation_tracks_orbit_images() {
        // A4 reversal folds to orbits {0,3} (commuting) and {1,2}
        // (linked); the generator permutation pi = [1,0,3,2] swaps the two
        // orbits, so the induced orbit permutation is [1,0].
        let orbits = vec![
            ExtGen {
                kind: ExtGenKind::Two,
                s0: 0,
                s1: 3,
            },
            ExtGen {
                kind: ExtGenKind::Three,
                s0: 1,
                s1: 2,
            },
        ];
        assert_eq!(induced(&orbits, &[1, 0, 3, 2]), vec![1, 0]);
        // The identity permutation induces the identity.
        assert_eq!(induced(&orbits, &[0, 1, 2, 3]), vec![0, 1]);
    }

    /// Per-element coherent `gamma_lambda` family at `gamma = rho` (the
    /// trivial representation's infinitesimal character), replicating the
    /// language layer's `common_block_gamma_lambdas`
    /// (domain_builtins.rs): torsion part from the dual element's torus
    /// bits, `gamma_lambda(x, y_bits, gamma)`, then `real_unique`.
    fn coherent_gamma_lambdas(
        fixture: &BlockFixture,
        rc: &RepContext,
    ) -> Vec<crate::RationalWeight> {
        let gamma = rc.rho().clone();
        let mut result = Vec::new();
        for z in 0..fixture.block.size() {
            let x = fixture.block.x(z).unwrap();
            let y = fixture.block.y(z).unwrap();
            let dual_bits = fixture.dual_graph.element(y).unwrap().torus_bits().clone();
            let y_bits = rc.torus_part(x, &dual_bits).unwrap();
            let mut value = rc.gamma_lambda(x, &y_bits, &gamma).unwrap();
            let involution = rc.involution_of(x).unwrap();
            rc.real_unique(involution, &mut value).unwrap();
            result.push(value);
        }
        result
    }

    /// Run `tune_signs` with the genuine ext_param/star oracle and assert
    /// success (which already includes the per-`(n, s)` type comparison
    /// and the debug quadratic/braid gates).
    fn tune_with_real_oracle(fixture: &BlockFixture, delta: &LatticeInvolution, eb: &mut ExtBlock) {
        let rc = RepContext::new(&fixture.primal_class, &fixture.table, &fixture.graph).unwrap();
        let ctx = crate::ext_param::ExtRepContext::new(&rc, delta.clone()).unwrap();
        let gamma_lambdas = coherent_gamma_lambdas(fixture, &rc);
        let simply_ints: Vec<usize> = rc
            .root_system()
            .simple_root_ids()
            .iter()
            .map(|id| id.index())
            .collect();
        let mut oracle =
            crate::ext_param::ExtParamOracle::new(&ctx, &fixture.block, &gamma_lambdas);
        assert!(eb.tune_signs(&mut oracle, &simply_ints));
    }

    #[test]
    fn a1_tune_signs_with_ext_param_oracle() {
        let fixture = a1_block();
        let (delta, twist, dual_delta, dual_twist) = identity_twists(&fixture);
        let mut eb = ExtBlock::build(
            &fixture.block,
            &fixture.graph,
            &fixture.table,
            &fixture.dual_graph,
            &fixture.dual_table,
            &delta,
            &twist,
            &dual_delta,
            &dual_twist,
            &[vec![2]],
        )
        .unwrap();
        tune_with_real_oracle(&fixture, &delta, &mut eb);
        // Trivial delta at gamma = rho: no edge is flipped.
        let (_, links0, links1) = wrapper_tables(&eb);
        for row in links0.iter().chain(&links1) {
            assert!(row.iter().all(|&link| link >= 0));
        }
    }

    #[test]
    fn a2_equal_rank_tune_signs_with_ext_param_oracle() {
        let fixture = a2_equal_rank_block();
        let (delta, twist, dual_delta, dual_twist) = identity_twists(&fixture);
        let cartan = vec![vec![2, -1], vec![-1, 2]];
        let mut eb = ExtBlock::build(
            &fixture.block,
            &fixture.graph,
            &fixture.table,
            &fixture.dual_graph,
            &fixture.dual_table,
            &delta,
            &twist,
            &dual_delta,
            &dual_twist,
            &cartan,
        )
        .unwrap();
        tune_with_real_oracle(&fixture, &delta, &mut eb);
        // Trivial delta at gamma = rho: no edge is flipped.
        let (_, links0, links1) = wrapper_tables(&eb);
        for row in links0.iter().chain(&links1) {
            assert!(row.iter().all(|&link| link >= 0));
        }
    }

    #[test]
    fn a2_flip_tune_signs_with_ext_param_oracle() {
        // Exercises the star length-3 cases (3Ci at element 0, 3r at
        // element 1) through the genuine oracle.
        let fixture = a2_flipped_block();
        let (delta, twist, dual_delta, dual_twist) = flip_twists(&fixture);
        let cartan = vec![vec![2, -1], vec![-1, 2]];
        let mut eb = ExtBlock::build(
            &fixture.block,
            &fixture.graph,
            &fixture.table,
            &fixture.dual_graph,
            &fixture.dual_table,
            &delta,
            &twist,
            &dual_delta,
            &dual_twist,
            &cartan,
        )
        .unwrap();
        tune_with_real_oracle(&fixture, &delta, &mut eb);
    }

    // -------------------------------------------------------------------
    // Partial-parent (integral subsystem) anchors: the frozen fixture
    // tests/fixtures/domain/ext_block_proper.atlas, verified against the
    // local oracle on 2026-08-18 (distinguished = identity involution for
    // these identity inner classes, so `extended_block(p, id)` builds over
    // delta = identity).
    // -------------------------------------------------------------------

    /// Owns the values a `RepContext` borrows, for the partial-parent
    /// fixtures (same shape as partial_block.rs's ContextFixture).
    struct PartialFixture {
        inner_class: InnerClass,
        table: InvolutionTable,
        graph: KgbGraph,
    }

    impl PartialFixture {
        fn rc(&self) -> RepContext<'_> {
            RepContext::new(&self.inner_class, &self.table, &self.graph).unwrap()
        }
    }

    /// The identity inner class of `datum`, with the KGB graph of its real
    /// form whose KGB size is `kgb_size`.
    fn partial_fixture(datum: BasedRootDatum, weyl: usize, kgb_size: usize) -> PartialFixture {
        let involution = LatticeInvolution::identity(&datum).unwrap();
        let inner_class = InnerClass::new(datum, involution, weyl).unwrap();
        let classification =
            CartanClassification::build(&inner_class, &class_budget(weyl)).unwrap();
        let strong = StrongRealClassification::build(&classification, 4_096).unwrap();
        let mut table = InvolutionTable::new(
            &inner_class,
            InvolutionTableBudget::new(64, IntegerLatticeBudget::new(64, 100_000, 100_000, 128)),
        )
        .unwrap();
        let (graph, table) =
            graph_with_size(&inner_class, &classification, &strong, &mut table, kgb_size);
        PartialFixture {
            inner_class,
            table,
            graph,
        }
    }

    /// `simply_connected(Lie_type("B2"),true)` (as in partial_block.rs);
    /// the split form so(3,2) has KGB size 11.
    fn b2_datum() -> BasedRootDatum {
        BasedRootDatum::from_simple_data(
            2,
            vec![vec![2, -2], vec![-1, 2]],
            vec![Weight::new(vec![2, -2]), Weight::new(vec![-1, 2])],
            vec![Coweight::new(vec![1, 0]), Coweight::new(vec![0, 1])],
        )
        .unwrap()
    }

    /// `simply_connected(Lie_type("C2"),true)`; the split form sp(4,R) has
    /// KGB size 11.
    fn c2_datum() -> BasedRootDatum {
        BasedRootDatum::from_simple_data(
            2,
            vec![vec![2, -1], vec![-2, 2]],
            vec![Weight::new(vec![2, -1]), Weight::new(vec![-2, 2])],
            vec![Coweight::new(vec![1, 0]), Coweight::new(vec![0, 1])],
        )
        .unwrap()
    }

    /// The wrapper's seed path plus the partial-parent construction:
    /// mod-reduce the parameter, build the full common block on its
    /// integral subsystem, then [`ExtBlock::build_partial`] with the
    /// identity involution and the trivial block modifier. Returns the
    /// parent block, the extended block, and the `simply_ints` (parent
    /// root numbers of the subsystem simple roots) that `tune_signs`
    /// needs.
    fn partial_ext_block(
        fixture: &PartialFixture,
        x: usize,
        lambda_rho: &[i32],
        gamma: &RationalWeight,
    ) -> (PartialBlock, ExtBlock, Vec<usize>) {
        let rc = fixture.rc();
        let z = rc
            .sr_gamma(KgbId(x), &Weight::new(lambda_rho.to_vec()), gamma)
            .unwrap();
        let seed = StandardReprMod::mod_reduce(&rc, &z).unwrap();
        let ctxt = CommonContext::integral(&rc, seed.gamma_lambda()).unwrap();
        let (block, _) = PartialBlock::build_full(&ctxt, &seed).unwrap();
        let delta = LatticeInvolution::identity(rc.datum()).unwrap();
        let twist = fixture
            .inner_class
            .based_involution_twist(delta.clone())
            .unwrap();
        let simp_int: Vec<RootId> = (0..ctxt.subsystem().rank())
            .map(|s| ctxt.subsystem().parent_root(s).unwrap())
            .collect();
        let bm = BlockModifier::trivial(rc.root_system(), simp_int.clone()).unwrap();
        let eb = ExtBlock::build_partial(&block, &ctxt, &bm, &delta, &twist).unwrap();
        let simply_ints = simp_int.iter().map(|id| id.index()).collect();
        (block, eb, simply_ints)
    }

    /// Run `tune_signs` with the `PartialBlock`-backed ext_param/star
    /// oracle and assert success (which already includes the per-`(n, s)`
    /// type comparison and the debug quadratic/braid gates).
    fn tune_partial(
        fixture: &PartialFixture,
        block: &PartialBlock,
        simply_ints: &[usize],
        eb: &mut ExtBlock,
    ) {
        let rc = fixture.rc();
        let delta = LatticeInvolution::identity(rc.datum()).unwrap();
        let ctx = crate::ext_param::ExtRepContext::new(&rc, delta).unwrap();
        let mut oracle = crate::ext_param::PartialBlockOracle::new(&ctx, block);
        assert!(eb.tune_signs(&mut oracle, simply_ints));
    }

    /// Assert that no edge carries a sign flip (the oracle prints no
    /// negative link entries for these anchors).
    fn assert_no_flips(eb: &ExtBlock) {
        let (_, links0, links1) = wrapper_tables(eb);
        for row in links0.iter().chain(&links1) {
            assert!(row.iter().all(|&link| link >= 0));
        }
    }

    #[test]
    fn b2_partial_ext_block_proper_subsystem_matches_oracle() {
        // Oracle anchor: pb := param(KGB(rfb,5),[1,1],[1,0]/2) over split
        // so(3,2); gamma(pb) = [5,3]/2 has a rank-1 integral subsystem
        // (generated by the long root [0,2] with coroot [1,1]). The parent
        // common block is the 3-element print_block(pb) with x = 4, 5, 10,
        // and the extended block is the A1-shaped fiber the oracle prints:
        // types [2,2,3], first links [2,2,0], second links [1,0,1].
        let fixture = partial_fixture(b2_datum(), 8, 11);
        let gamma = RationalWeight::new(vec![5, 3], 2).unwrap();
        let (block, mut eb, simply_ints) = partial_ext_block(&fixture, 5, &[1, 1], &gamma);

        assert_eq!(block.rank(), 1);
        assert_eq!(block.size(), 3);
        assert_eq!(
            (0..3)
                .map(|z| block.x(z).unwrap().index())
                .collect::<Vec<_>>(),
            vec![4, 5, 10]
        );

        // The subsystem Cartan matrix is [[2]]; the identity delta fixes
        // its single generator, so the one orbit is a singleton.
        assert_eq!(eb.rank(), 1);
        assert_eq!(eb.orbit(0).kind, ExtGenKind::One);
        assert_eq!(eb.orbit(0).s0, 0);
        assert_eq!(eb.folded_cartan(), &[vec![2]]);

        // All three elements are delta-fixed, in parent order.
        assert_eq!(eb.size(), 3);
        assert_eq!((0..3).map(|n| eb.z(n)).collect::<Vec<_>>(), vec![0, 1, 2]);
        assert_eq!(
            (0..3)
                .map(|n| block.x(eb.z(n)).unwrap().index())
                .collect::<Vec<_>>(),
            vec![4, 5, 10]
        );
        assert_eq!(eb.length(0), 0);
        assert_eq!(eb.length(1), 0);
        assert_eq!(eb.length(2), 1);

        let (types, links0, links1) = wrapper_tables(&eb);
        assert_eq!(
            types,
            vec![
                vec![DescValue::OneImaginarySingle as usize],
                vec![DescValue::OneImaginarySingle as usize],
                vec![DescValue::OneRealPairFixed as usize],
            ]
        );
        assert_eq!(links0, vec![vec![2], vec![2], vec![0]]);
        assert_eq!(links1, vec![vec![1], vec![0], vec![1]]);

        tune_partial(&fixture, &block, &simply_ints, &mut eb);
        assert_no_flips(&eb);
    }

    #[test]
    fn a2_partial_ext_block_full_subsystem_matches_oracle() {
        // Oracle anchor: pa := param(KGB(rfa,0),[0,0],[1,0]/2) over
        // su(2,1); gamma(pa) = rho is integral, so the integral subsystem
        // is the full A2 system and the partial parent is the 6-element
        // full common block. The tables equal those of
        // a2_trivial_delta_matches_oracle (same oracle output).
        let fixture = partial_fixture(a2_datum(), 8, 6);
        let gamma = RationalWeight::new(vec![1, 1], 1).unwrap();
        let (block, mut eb, simply_ints) = partial_ext_block(&fixture, 0, &[0, 0], &gamma);

        assert_eq!(block.rank(), 2);
        assert_eq!(block.size(), 6);
        assert_eq!(eb.size(), 6);
        assert_eq!(eb.rank(), 2);
        assert_eq!(eb.folded_cartan(), &[vec![2, -1], vec![-1, 2]]);

        let (types, links0, links1) = wrapper_tables(&eb);
        let u = 6_isize; // UndefBlock in the wrapper output
        assert_eq!(
            types,
            vec![
                vec![2, 2], // OneImaginarySingle, OneImaginarySingle
                vec![2, 9], // OneImaginarySingle, OneImaginaryCompact
                vec![9, 2],
                vec![0, 3], // OneComplexAscent, OneRealPairFixed
                vec![3, 0],
                vec![1, 1], // OneComplexDescent, OneComplexDescent
            ]
        );
        assert_eq!(
            links0,
            vec![
                vec![4, 3],
                vec![4, u],
                vec![u, 3],
                vec![5, 0],
                vec![0, 5],
                vec![3, 4],
            ]
        );
        assert_eq!(
            links1,
            vec![
                vec![1, 2],
                vec![0, u],
                vec![u, 0],
                vec![u, 2],
                vec![1, u],
                vec![u, u],
            ]
        );

        // The full-path constructor over the equal-rank block agrees
        // element-for-element with the partial-parent construction.
        let full = a2_equal_rank_block();
        let (delta, twist, dual_delta, dual_twist) = identity_twists(&full);
        let cartan = vec![vec![2, -1], vec![-1, 2]];
        let full_eb = ExtBlock::build(
            &full.block,
            &full.graph,
            &full.table,
            &full.dual_graph,
            &full.dual_table,
            &delta,
            &twist,
            &dual_delta,
            &dual_twist,
            &cartan,
        )
        .unwrap();
        assert_eq!(wrapper_tables(&eb), wrapper_tables(&full_eb));

        tune_partial(&fixture, &block, &simply_ints, &mut eb);
        assert_no_flips(&eb);
    }

    #[test]
    fn c2_partial_ext_block_full_subsystem_matches_oracle() {
        // Oracle anchor: ps := param(KGB(fs,0),[0,0],[1,0]/2) over split
        // sp(4,R); gamma(ps) = rho is integral (full subsystem). The
        // extended block has 12 elements over the 12-element common block
        // (two elements share x = 10), with the tables below.
        let fixture = partial_fixture(c2_datum(), 8, 11);
        let gamma = RationalWeight::new(vec![1, 1], 1).unwrap();
        let (block, mut eb, simply_ints) = partial_ext_block(&fixture, 0, &[0, 0], &gamma);

        assert_eq!(block.size(), 12);
        assert_eq!(eb.size(), 12);
        assert_eq!(eb.rank(), 2);
        // folded_cartan stores the transposed (upstream DynkinDiagram)
        // convention: folded[i][j] = <folded root j, folded coroot i>.
        assert_eq!(eb.folded_cartan(), &[vec![2, -2], vec![-1, 2]]);
        assert_eq!(
            (0..12)
                .map(|n| block.x(eb.z(n)).unwrap().index())
                .collect::<Vec<_>>(),
            vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 10]
        );

        let (types, links0, links1) = wrapper_tables(&eb);
        let u = 12_isize; // UndefBlock in the wrapper output
        assert_eq!(
            types,
            vec![
                vec![2, 2], // 1i1, 1i1
                vec![2, 2],
                vec![9, 2], // 1ic, 1i1
                vec![9, 2],
                vec![3, 0], // 1r1fixed, 1C+
                vec![0, 3],
                vec![0, 3],
                vec![1, 2], // 1C-, 1i1
                vec![1, 2],
                vec![4, 1], // 1i2fixed, 1C-
                vec![5, 3], // 1r2single, 1r1fixed
                vec![5, 8], // 1r2single, 1rn
            ]
        );
        assert_eq!(
            links0,
            vec![
                vec![4, 5],
                vec![4, 6],
                vec![u, 5],
                vec![u, 6],
                vec![0, 9],
                vec![7, 0],
                vec![8, 1],
                vec![5, 10],
                vec![6, 10],
                vec![10, 4],
                vec![9, 7],
                vec![9, u],
            ]
        );
        assert_eq!(
            links1,
            vec![
                vec![1, 2],
                vec![0, 3],
                vec![u, 0],
                vec![u, 1],
                vec![1, u],
                vec![u, 2],
                vec![u, 3],
                vec![u, 8],
                vec![u, 7],
                vec![11, u],
                vec![11, 8],
                vec![10, u],
            ]
        );

        tune_partial(&fixture, &block, &simply_ints, &mut eb);
        assert_no_flips(&eb);
    }

    #[test]
    fn partial_ext_block_cofolds_non_identity_attitude() {
        // The generator swap simple_pi=[1,0] on A2 (ext_block.cpp:636-663):
        // induced() yields the orbit swap, so the per-generator link tables
        // swap places while the (symmetric) folded diagram and the rewritten
        // orbits come out unchanged.
        let fixture = partial_fixture(a2_datum(), 8, 6);
        let rc = fixture.rc();
        let gamma = RationalWeight::new(vec![1, 1], 1).unwrap();
        let z = rc
            .sr_gamma(KgbId(0), &Weight::new(vec![0, 0]), &gamma)
            .unwrap();
        let seed = StandardReprMod::mod_reduce(&rc, &z).unwrap();
        let ctxt = CommonContext::integral(&rc, seed.gamma_lambda()).unwrap();
        let (block, _) = PartialBlock::build_full(&ctxt, &seed).unwrap();
        let delta = LatticeInvolution::identity(rc.datum()).unwrap();
        let twist = fixture
            .inner_class
            .based_involution_twist(delta.clone())
            .unwrap();
        let simp_int: Vec<RootId> = (0..ctxt.subsystem().rank())
            .map(|s| ctxt.subsystem().parent_root(s).unwrap())
            .collect();
        let identity_bm = BlockModifier::trivial(rc.root_system(), simp_int.clone()).unwrap();
        let reference =
            ExtBlock::build_partial(&block, &ctxt, &identity_bm, &delta, &twist).unwrap();
        let locator = BlockLocator::from_parts(
            u32::MAX,
            WeylElement::identity(rc.root_system()).unwrap(),
            simp_int,
            vec![1, 0], // the generator swap: not the identity
        );
        let bm = BlockModifier::from_locator(locator, RationalWeight::zero(2).unwrap());
        let swapped = ExtBlock::build_partial(&block, &ctxt, &bm, &delta, &twist).unwrap();
        assert_eq!(swapped.orbits, reference.orbits);
        assert_eq!(swapped.folded, reference.folded);
        assert_eq!(swapped.data[0], reference.data[1]);
        assert_eq!(swapped.data[1], reference.data[0]);
    }
}
