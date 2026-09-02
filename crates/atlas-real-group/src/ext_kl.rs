//! Extended (twisted) Kazhdan-Lusztig-Vogan polynomial tables
//! (upstream `gkmod/ext_kl.{h,cpp}`).
//!
//! This module mirrors `kl_table.rs` structurally, but the recursion is the
//! extended-block one: [`DescentTable`] (ext_kl.cpp:20-118) precomputes
//! descents/good ascents, the primitive-index tables *with primitivisation
//! sign flips* (`prim_flip`, ext_kl.cpp:79-80), and [`ExtKlTable`]
//! (ext_kl.cpp:120-841) stores one column of pool indices per block element
//! `y`, indexed by the primitive position of `x` for the descent set of
//! `y`; the primitivisation sign lives in the flip bitmap, so
//! [`ExtKlTable::kl_pol_index`] returns the `(KLIndex, bool)` pair that the
//! `raw_ext_KL` wrapper renders as `inx.second ? -inx.first : inx.first`
//! (interpreter/atlas-types.w:8713-8714).
//!
//! The polynomial pool is the shared [`KlHashTable`] of `kl_polynomial.rs`:
//! its entries are `KlPol` over `i32`, which is exactly upstream's
//! `IntPolEntry = Polynomial<int>`, so `KlHashTable` already *is* an
//! `ext_KL_hash_Table = HashTable<IntPolEntry, ext_kl::KLIndex>`
//! (Atlas.h:478) with `KLIndex = unsigned int` (Atlas.h:475). No sign bit
//! is packed into the index — upstream does not either; the sign is the
//! separate `prim_flip` bitmap consulted at lookup time.
//!
//! Deviations from upstream, all deliberate:
//!
//! - `KL_table` always owns its pool; the shared-pool pointer mode and
//!   `swallow` (ext_kl.cpp:854-894, partial-block migration) are deferred
//!   to the common-block slice that needs them.
//! - `fill_columns` propagates errors after clearing the offending column
//!   instead of upstream's `catch(...)` swallow (ext_kl.cpp:437-444); the
//!   column invariant (`column[y]` empty or complete) is preserved.
//! - The `#ifndef NDEBUG` helpers `get_M` (ext_kl.cpp:248-347) and the
//!   down-set defect check (ext_kl.cpp:618-637) are not ported; `get_Mp`
//!   is the production path.
//! - `check_polys` (ext_kl.cpp:917-936) is not ported: it needs the
//!   untwisted block, which `ExtBlock` does not retain.
//! - [`ext_kl_matrix`] (ext_kl.cpp:939-1020) is split at the `ExtBlock`
//!   boundary: building the extended block from a `StandardRepr` needs
//!   `StandardReprMod`/`common_block`, which is a later slice. What is
//!   ported here is everything after `eblock` exists: fill, expand,
//!   [`condense`], parity sign flips, and the pool-index matrix. The
//!   caller supplies `size` (upstream `eblock.element(entry_element+1)`)
//!   and `singular_orbits` (upstream `eblock.singular_orbits(B.singular
//!   (gamma))`).

use crate::ext_block::{DescValue, ExtBlock, SPol};
use crate::kl_polynomial::{KlHashTable, KlPol};
use crate::kl_support::RankFlags;
use crate::StructureError;

pub type BlockElt = usize;

/// Upstream `ext_kl::KLIndex` (Atlas.h:475): an index into the polynomial
/// pool. The sign of a stored polynomial is *not* packed here; it is the
/// `prim_flip` bitmap of [`DescentTable`].
pub type KLIndex = usize;

/// Indices of the constant polynomials 0 and 1 in the pool
/// (ext_kl.h:126-128).
const ZERO: KLIndex = 0;
const ONE: KLIndex = 1;

/// Upstream's `dead_end = -1` sentinel in the primitive-index tables
/// (ext_kl.cpp:44).
const DEAD_END: usize = usize::MAX;

/// Guard for the `1 << rank` primitive-index tables (ext_kl.cpp:22): the
/// allocation is exponential in the folded rank, like upstream, but we fail
/// rather than exhaust memory on a nonsensical block.
const MAX_FOLDED_RANK: usize = 12;

/// The bit mask of a `RankFlags` (upstream `RankFlags::to_ulong`).
fn mask_of(flags: &RankFlags) -> usize {
    let mut mask = 0usize;
    for generator in 0..32 {
        if flags.is_set(generator) {
            mask |= 1 << generator;
        }
    }
    mask
}

/// A bitmap over descent-set masks (upstream `BitMap` in
/// `descent_table::prim_flip`, ext_kl.h:40).
#[derive(Clone, Debug)]
struct BitMap {
    words: Vec<u64>,
}

impl BitMap {
    fn new(capacity: usize) -> Self {
        Self {
            words: vec![0; capacity.div_ceil(64)],
        }
    }
    fn set(&mut self, index: usize) {
        self.words[index / 64] |= 1 << (index % 64);
    }
    fn is_member(&self, index: usize) -> bool {
        self.words[index / 64] & (1 << (index % 64)) != 0
    }
}

/// Per-element descent data (upstream `descent_table::Elt_info`,
/// ext_kl.h:32-36).
#[derive(Clone, Debug)]
struct EltInfo {
    descents: RankFlags,
    good_ascents: RankFlags,
}

/// The descent table of an extended block (upstream `ext_kl::descent_table`,
/// ext_kl.h:30-97, constructor ext_kl.cpp:20-91).
pub struct DescentTable<'a> {
    info: Vec<EltInfo>,
    /// `prim_index[mask][x]`: position of the primitivisation of `x` among
    /// the primitive elements for the descent set `mask` (ext_kl.h:39).
    prim_index: Vec<Vec<usize>>,
    /// `prim_flip[x]`: the descent sets for which primitivising `x` picks
    /// up a sign (ext_kl.h:40).
    prim_flip: Vec<BitMap>,
    pub block: &'a ExtBlock,
}

impl<'a> DescentTable<'a> {
    /// Upstream `descent_table::descent_table` (ext_kl.cpp:20-91).
    pub fn new(block: &'a ExtBlock) -> Result<Self, StructureError> {
        let rank = block.rank();
        if rank > MAX_FOLDED_RANK {
            return Err(StructureError::ResourceLimitExceeded {
                limit: MAX_FOLDED_RANK,
            });
        }
        let size = block.size();
        let mut info = Vec::with_capacity(size);
        for x in 0..size {
            let mut descents = RankFlags::empty();
            let mut good_ascents = RankFlags::empty();
            for s in 0..rank {
                let value = block.descent_type(s, x);
                if value.is_descent() {
                    descents.set(s);
                } else if !value.has_double_image() {
                    good_ascents.set(s); // at most one upward neighbour
                }
            }
            info.push(EltInfo {
                descents,
                good_ascents,
            });
        }

        let n_masks = 1usize << rank;
        let mut prim_index = vec![vec![0usize; size]; n_masks];
        let mut prim_flip: Vec<BitMap> = (0..size).map(|_| BitMap::new(n_masks)).collect();
        for mask in 0..n_masks {
            // Take the row out so `prim_flip` stays independently mutable.
            let mut prindex_vec = std::mem::take(&mut prim_index[mask]);
            let mut count = 0usize;
            // Decreasing loop: primitivisation indices are first computed
            // as counts of *larger* primitives (ext_kl.cpp:48).
            for x in (0..size).rev() {
                // First good ascent of `x` inside `mask`, if any.
                let d = (0..rank).find(|&s| info[x].good_ascents.is_set(s) && (mask >> s) & 1 == 1);
                let Some(s) = d else {
                    prindex_vec[x] = count; // `x` is primitive for `mask`
                    count += 1;
                    continue;
                };
                if block.descent_type(s, x).is_like_nonparity() {
                    prindex_vec[x] = DEAD_END; // zero primitivisation
                    continue;
                }
                let Some(sx) = block.some_scent(s, x) else {
                    prindex_vec[x] = DEAD_END; // crossed a partial-block edge
                    continue;
                };
                debug_assert!(sx > x); // ascents go up in the block
                prindex_vec[x] = prindex_vec[sx];
                let base_flip = prim_flip[sx].is_member(mask);
                if (block.epsilon(s, x, sx) < 0) != base_flip {
                    prim_flip[x].set(mask);
                }
            }
            // Primitive lists are stored increasing: reverse the indices
            // (ext_kl.cpp:84-87). When `count == 0` every slot is DEAD_END.
            if count > 0 {
                let last = count - 1;
                for slot in prindex_vec.iter_mut() {
                    if *slot != DEAD_END {
                        *slot = last - *slot;
                    }
                }
            }
            prim_index[mask] = prindex_vec;
        }
        Ok(Self {
            info,
            prim_index,
            prim_flip,
            block,
        })
    }

    /// Upstream `descent_set` (ext_kl.h:49).
    pub fn descent_set(&self, y: BlockElt) -> &RankFlags {
        &self.info[y].descents
    }

    /// Upstream `good_ascent_set` (ext_kl.h:50).
    pub fn good_ascent_set(&self, y: BlockElt) -> &RankFlags {
        &self.info[y].good_ascents
    }

    /// Upstream `is_descent` (ext_kl.h:52-53).
    pub fn is_descent(&self, s: usize, y: BlockElt) -> bool {
        self.descent_set(y).is_set(s)
    }

    /// Upstream `very_easy_set` (ext_kl.h:55-56).
    pub fn very_easy_set(&self, x: BlockElt, y: BlockElt) -> RankFlags {
        self.info[x].good_ascents.intersect(&self.info[y].descents)
    }

    /// Upstream `easy_set` (ext_kl.h:58-59).
    pub fn easy_set(&self, x: BlockElt, y: BlockElt) -> RankFlags {
        self.info[y].descents.difference(&self.info[x].descents)
    }

    /// Upstream `x_index(x, desc)` (ext_kl.h:62-63): `desc` as a mask.
    fn x_index_mask(&self, x: BlockElt, desc: usize) -> usize {
        self.prim_index[desc][x]
    }

    /// Upstream `x_index(x, y)` (ext_kl.h:64-65).
    pub fn x_index(&self, x: BlockElt, y: BlockElt) -> usize {
        self.x_index_mask(x, mask_of(self.descent_set(y)))
    }

    /// Upstream `self_index` (ext_kl.h:66).
    pub fn self_index(&self, y: BlockElt) -> usize {
        self.x_index(y, y)
    }

    /// Upstream `flips` (ext_kl.h:67-68).
    pub fn flips(&self, x: BlockElt, y: BlockElt) -> bool {
        self.prim_flip[x].is_member(mask_of(self.descent_set(y)))
    }

    /// Upstream `length_floor` (ext_kl.h:70-71).
    pub fn length_floor(&self, y: BlockElt) -> usize {
        self.block.length_first(self.block.length(y))
    }

    /// Upstream `col_size` (ext_kl.cpp:94-100).
    pub fn col_size(&self, y: BlockElt) -> usize {
        let mut x = self.length_floor(y);
        if self.prim_back_up(&mut x, y) {
            self.x_index(x, y) + 1
        } else {
            0
        }
    }

    /// Upstream `is_extremal` (ext_kl.h:76-77).
    pub fn is_extremal(&self, x: BlockElt, descents_y: &RankFlags) -> bool {
        self.descent_set(x).contains(descents_y)
    }

    /// Upstream `is_primitive` (ext_kl.h:78-79).
    pub fn is_primitive(&self, x: BlockElt, descents_y: &RankFlags) -> bool {
        self.good_ascent_set(x).intersect(descents_y).none()
    }

    /// Upstream `extr_back_up(x, desc_y)` (ext_kl.h:82-85).
    pub fn extr_back_up_mask(&self, x: &mut BlockElt, desc_y: &RankFlags) -> bool {
        while *x > 0 {
            *x -= 1;
            if self.is_extremal(*x, desc_y) {
                return true;
            }
        }
        false
    }

    /// Upstream `prim_back_up(x, desc_y)` (ext_kl.h:87-91).
    pub fn prim_back_up_mask(&self, x: &mut BlockElt, desc_y: &RankFlags) -> bool {
        while *x > 0 {
            *x -= 1;
            if self.is_primitive(*x, desc_y) {
                return true;
            }
        }
        false
    }

    /// Upstream `prim_back_up(x, y)` (ext_kl.cpp:102-109).
    pub fn prim_back_up(&self, x: &mut BlockElt, y: BlockElt) -> bool {
        self.prim_back_up_mask(x, &self.info[y].descents.clone())
    }

    /// Upstream `extr_back_up(x, y)` (ext_kl.cpp:111-118).
    pub fn extr_back_up(&self, x: &mut BlockElt, y: BlockElt) -> bool {
        self.extr_back_up_mask(x, &self.info[y].descents.clone())
    }
}

/// Multiply a `KlPol` by a signed `SPol` (the `T_coef * P(...)` products of
/// ext_kl.cpp:209-212 and the `P(x,u)*M[u]` products elsewhere).
fn pol_mul_spol(coef: &SPol, pol: &KlPol) -> KlPol {
    let mut result = KlPol::zero();
    for (degree, &c) in coef.as_slice().iter().enumerate() {
        if c != 0 {
            result = result.add_shifted_scaled(
                pol,
                degree,
                i32::try_from(c).expect("T_coef coefficients are small"),
            );
        }
    }
    result
}

/// Ordinary polynomial multiplication (upstream `P(x,u)*M[u]`,
/// ext_kl.cpp:343, 559, 696).
fn pol_mul(a: &KlPol, b: &KlPol) -> KlPol {
    let mut result = KlPol::zero();
    for (degree, &c) in a.as_slice().iter().enumerate() {
        if c != 0 {
            result = result.add_shifted_scaled(b, degree, c);
        }
    }
    result
}

/// Upstream `Polynomial::up_remainder` with `c == 1`
/// (utilities/polynomials_def.h:208-217): the running remainder of division
/// by `1 + q` in degree `d`, assuming `degree() <= d`.
fn up_remainder_1(pol: &KlPol, d: usize) -> i32 {
    if pol.is_zero() {
        return 0;
    }
    debug_assert!(pol.as_slice().len() <= d + 1);
    let mut remainder = pol.coefficient(0);
    for i in 1..=d {
        remainder = pol.coefficient(i) - remainder;
    }
    remainder
}

/// Upstream `Polynomial::factor_by_1_plus_q`
/// (utilities/polynomials_def.h:220-238): replace `pol` by its quotient by
/// `1 + q` (computed upward to degree `d`), returning the degree-`d`
/// remainder.
fn factor_by_1_plus_q(pol: &KlPol, d: usize) -> (KlPol, i32) {
    if pol.is_zero() {
        return (KlPol::zero(), 0);
    }
    debug_assert!(pol.as_slice().len() <= d + 1);
    let mut data = pol.as_slice().to_vec();
    data.resize(d + 1, 0);
    let mut remainder = data[0];
    for i in 1..=d {
        data[i] -= remainder;
        remainder = data[i];
    }
    data[d] = 0; // kill the remainder in the top coefficient
    (KlPol::from_coefficients(data), remainder)
}

/// Upstream `Polynomial::factor_by_1_plus_q_to_the`
/// (utilities/polynomials_def.h:241-256): divide by `1 + q^k`, keeping the
/// terms up to degree `d - k`; the remainder is discarded (upstream returns
/// it, but the ext_kl call site ignores it).
fn factor_by_1_plus_q_to_the(pol: &KlPol, k: usize, d: usize) -> KlPol {
    if pol.is_zero() {
        return KlPol::zero();
    }
    debug_assert!(pol.as_slice().len() <= d + 1 && d >= k);
    let mut data = pol.as_slice().to_vec();
    data.resize(d + 1, 0);
    for i in k..=d {
        data[i] -= data[i - k];
    }
    for i in (0..k).rev() {
        data[d - i] = 0;
    }
    KlPol::from_coefficients(data)
}

/// Upstream `qk_plus_1` (ext_kl.cpp:178-184): `1 + q^k`.
pub fn qk_plus_1(k: usize) -> KlPol {
    debug_assert!((1..=3).contains(&k));
    KlPol::monomial(k).add(&KlPol::monomial(0))
}

/// Upstream `qk_minus_1` (ext_kl.cpp:186-192): `q^k - 1`.
pub fn qk_minus_1(k: usize) -> KlPol {
    debug_assert!((1..=3).contains(&k));
    KlPol::monomial(k).sub(&KlPol::monomial(0))
}

/// Upstream `qk_minus_q` (ext_kl.cpp:194-200): `q^k - q`.
pub fn qk_minus_q(k: usize) -> KlPol {
    debug_assert!((2..=3).contains(&k));
    KlPol::monomial(k).sub(&KlPol::monomial(1))
}

/// Upstream `m(a, b)` (ext_kl.cpp:226): the shifted symmetric Laurent
/// polynomial `a q^2 + b q + a` (or the constant `b` when `a == 0`).
fn m_poly(a: i32, b: i32) -> KlPol {
    if a == 0 {
        KlPol::monomial(0).scaled(b)
    } else {
        qk_plus_1(2).scaled(a).add(&KlPol::monomial(1).scaled(b))
    }
}

/// The twisted KLV table of one extended block (upstream `ext_kl::KL_table`,
/// ext_kl.h:114-190).
pub struct ExtKlTable<'a> {
    /// Upstream `aux` (ext_kl.h:117).
    pub aux: DescentTable<'a>,
    /// The polynomial pool; entries 0 and 1 are the constants 0 and 1.
    pool: KlHashTable,
    /// `column[y][i]`: pool index of `P_{x,y}` for the `i`-th primitive
    /// `x` of `y` (ext_kl.h:123-124). Signs live in `aux.flips`.
    column: Vec<Vec<KLIndex>>,
}

impl<'a> ExtKlTable<'a> {
    /// Upstream `KL_table::KL_table` (ext_kl.cpp:120-135), always with an
    /// owned pool (the shared `ext_KL_hash_Table*` mode is deferred).
    pub fn new(block: &'a ExtBlock) -> Result<Self, StructureError> {
        let aux = DescentTable::new(block)?;
        let pool = KlHashTable::new(); // seeds zero at 0 and one at 1
        Ok(Self {
            column: vec![Vec::new(); block.size()],
            aux,
            pool,
        })
    }

    /// Upstream `rank` (ext_kl.h:133).
    pub fn rank(&self) -> usize {
        self.aux.block.rank()
    }

    /// Upstream `size` (ext_kl.h:134).
    pub fn size(&self) -> usize {
        self.column.len()
    }

    /// Upstream `descent_set` (ext_kl.h:136-137).
    pub fn descent_set(&self, y: BlockElt) -> &RankFlags {
        self.aux.descent_set(y)
    }

    /// Upstream `type` (ext_kl.h:139-140).
    pub fn descent_type(&self, s: usize, y: BlockElt) -> DescValue {
        self.aux.block.descent_type(s, y)
    }

    /// Upstream `l` (ext_kl.h:142). Returns 0 when `x` is not shorter than
    /// `y` (upstream callers never ask in that case).
    pub fn l(&self, y: BlockElt, x: BlockElt) -> usize {
        self.aux
            .block
            .length(y)
            .saturating_sub(self.aux.block.length(x))
    }

    /// The polynomial pool (upstream `polys`, ext_kl.h:144).
    pub fn polys(&self) -> &KlHashTable {
        &self.pool
    }

    /// Upstream `KL_pol_index` against an explicit column
    /// (ext_kl.cpp:137-147): the table's own column during a fill, or
    /// `self.column[y]` afterwards.
    fn kl_pol_index_in(&self, x: BlockElt, y: BlockElt, column: &[KLIndex]) -> (KLIndex, bool) {
        let inx = self.aux.x_index(x, y);
        if inx < column.len() {
            (column[inx], self.aux.flips(x, y))
        } else if inx == self.aux.self_index(y) {
            (ONE, self.aux.flips(x, y)) // diagonal entries are unrecorded
        } else {
            (ZERO, false) // out of bounds implies zero (also DEAD_END)
        }
    }

    /// Upstream `KL_pol_index` (ext_kl.cpp:137-147): the pool index and the
    /// primitivisation sign. This is the pair the `raw_ext_KL` wrapper
    /// renders as `inx.second ? -inx.first : inx.first`
    /// (atlas-types.w:8713-8714).
    pub fn kl_pol_index(&self, x: BlockElt, y: BlockElt) -> (KLIndex, bool) {
        self.kl_pol_index_in(x, y, &self.column[y])
    }

    /// Upstream `P` against an explicit column (ext_kl.cpp:149-154).
    fn p_in(&self, x: BlockElt, y: BlockElt, column: &[KLIndex]) -> KlPol {
        let (index, flip) = self.kl_pol_index_in(x, y, column);
        let pol = self.pool.get(index).cloned().unwrap_or_else(KlPol::zero);
        if flip {
            pol.scaled(-1)
        } else {
            pol
        }
    }

    /// The twisted KLV polynomial `P_{x,y}` (upstream `P`,
    /// ext_kl.cpp:149-154).
    pub fn p(&self, x: BlockElt, y: BlockElt) -> KlPol {
        self.p_in(x, y, &self.column[y])
    }

    /// Upstream `is_extremal` (ext_kl.h:150-151).
    pub fn is_extremal(&self, x: BlockElt, y: BlockElt) -> bool {
        self.aux.easy_set(x, y).none()
    }

    /// Upstream `is_primitive` (ext_kl.h:152-153).
    pub fn is_primitive(&self, x: BlockElt, y: BlockElt) -> bool {
        self.aux.very_easy_set(x, y).none()
    }

    /// Upstream `nonzero_column` (ext_kl.cpp:156-167): the elements `x`
    /// with `P_{x,y}` nonzero, decreasing from `y`.
    pub fn nonzero_column(&self, y: BlockElt) -> Vec<BlockElt> {
        let column = &self.column[y];
        let mut result = vec![y];
        for x in (0..self.aux.length_floor(y)).rev() {
            let inx = self.aux.x_index(x, y);
            let nonzero = if inx < column.len() {
                column[inx] != ZERO
            } else {
                inx == self.aux.self_index(y)
            };
            if nonzero {
                result.push(x);
            }
        }
        result
    }

    /// Upstream `mu` (ext_kl.cpp:169-176): the coefficient of
    /// `q^{(l(y/x)-i)/2}` in `P_{x,y}` (used with `i` = 1, 2, 3).
    pub fn mu(&self, i: usize, x: BlockElt, y: BlockElt) -> i32 {
        self.mu_in(i, x, y, &self.column[y])
    }

    /// `mu` against an explicit column: `do_new_recursion` must read the
    /// column it is currently writing (upstream stores it in `column[y]`
    /// before recursing, ext_kl.cpp:517).
    fn mu_in(&self, i: usize, x: BlockElt, y: BlockElt, column: &[KLIndex]) -> i32 {
        let d = self.l(y, x);
        if d < i || !(d - i).is_multiple_of(2) {
            return 0;
        }
        self.p_in(x, y, column).coefficient((d - i) / 2)
    }

    /// Upstream `product_comp` (ext_kl.cpp:204-213): the component of the
    /// basis element `a_x` in the product `(T_s + 1) C_{sy}`.
    fn product_comp(&self, x: BlockElt, s: usize, sy: BlockElt) -> KlPol {
        debug_assert!(self.descent_type(s, x).is_descent());
        let mut neighbours = Vec::new();
        self.aux.block.add_neighbours(&mut neighbours, s, x);
        let mut result = pol_mul_spol(&self.aux.block.t_coef(s, x, x), &self.p(x, sy));
        for sx in neighbours {
            result.add_assign(&pol_mul_spol(
                &self.aux.block.t_coef(s, x, sx),
                &self.p(sx, sy),
            ));
        }
        result
    }

    /// Upstream `get_Mp` (ext_kl.cpp:364-404): the new-recursion variant of
    /// the `m_s(x,y)` computation; terms involving `Cayley(s,x)` are
    /// omitted because `P_{x',y}` for `x' < x` is not yet known.
    fn get_mp(
        &self,
        s: usize,
        x: BlockElt,
        y: BlockElt,
        ms: &[KlPol],
        column: &[KLIndex],
    ) -> KlPol {
        let bl = self.aux.block;
        let k = bl.orbit(s).length();
        if k == 1 {
            return KlPol::monomial(0).scaled(if self.l(y, x).is_multiple_of(2) {
                0
            } else {
                self.mu_in(1, x, y, column)
            });
        }
        if k == 2 {
            if !self.l(y, x).is_multiple_of(2) {
                return qk_plus_1(1).scaled(self.mu_in(1, x, y, column));
            }
            let mut acc = self.mu_in(2, x, y, column);
            let mut l = bl.length(x) + 1;
            while l < bl.length(y) {
                for u in bl.length_first(l)..bl.length_first(l + 1) {
                    if self.aux.is_descent(s, u) && !ms[u].is_zero() {
                        debug_assert!(
                            ms[u].degree() == 1 && ms[u].coefficient(0) == ms[u].coefficient(1)
                        );
                        acc -= self.mu_in(1, x, u, column) * ms[u].coefficient(1);
                    }
                }
                l += 2;
            }
            return KlPol::monomial(0).scaled(acc);
        }

        debug_assert!(k == 3);
        if self.l(y, x).is_multiple_of(2) {
            // A multiple of `1 + q`.
            let mut acc = self.mu_in(2, x, y, column);
            let mut l = bl.length(x) + 1;
            while l < bl.length(y) {
                for u in bl.length_first(l)..bl.length_first(l + 1) {
                    if self.aux.is_descent(s, u) && ms[u].degree() == 2 {
                        acc -= self.mu_in(1, x, u, column) * ms[u].coefficient(2);
                    }
                }
                l += 2;
            }
            return qk_plus_1(1).scaled(acc);
        }

        // A polynomial of the form `a + bq + aq^2`.
        let a = self.mu_in(1, x, y, column);
        let mut b = self.mu_in(3, x, y, column);
        for u in bl.length_first(bl.length(x) + 1)..self.aux.length_floor(y) {
            if self.aux.is_descent(s, u) && ms[u].degree() == 2 - self.l(u, x) % 2 {
                let d = ms[u].degree();
                b -= self.mu_in(d, x, u, column) * ms[u].coefficient(d);
            }
        }
        m_poly(a, b)
    }

    /// Upstream `has_direct_recursion` (ext_kl.cpp:408-421): the first
    /// generator that is a unique-image descent of `y`, with the resulting
    /// descent `sy`.
    fn has_direct_recursion(&self, y: BlockElt) -> Option<(usize, BlockElt)> {
        for s in 0..self.rank() {
            let value = self.descent_type(s, y);
            if value.is_descent() && value.is_unique_image() {
                let sy = self
                    .aux
                    .block
                    .some_scent(s, y)
                    .expect("direct recursion: unique-image descent has a link");
                return Some((s, sy));
            }
        }
        None
    }

    /// Upstream `fill_columns` (ext_kl.cpp:429-448): compute all columns
    /// `y < limit`; `limit == 0` fills the whole block. On error the
    /// offending column is cleared (upstream's `catch(...)` at :441-444)
    /// and the error is propagated rather than swallowed.
    pub fn fill_columns(&mut self, limit: BlockElt) -> Result<(), StructureError> {
        let limit = if limit == 0 {
            self.aux.block.size()
        } else {
            limit
        };
        for y in self.aux.block.length_first(1)..limit {
            if self.column[y].len() != self.aux.col_size(y) {
                debug_assert!(self.column[y].is_empty());
                if let Err(error) = self.fill_column(y) {
                    self.column[y].clear();
                    return Err(error);
                }
            }
        }
        Ok(())
    }

    /// Upstream `extract_M` (ext_kl.cpp:457-512): clear the terms of
    /// `q` of degree `>= d/2` by subtracting `r^d m` for a symmetric
    /// Laurent polynomial `m` (and dividing by `1 + q` when `defect > 0`);
    /// returns `r^{deg m} m` as an ordinary polynomial.
    fn extract_m(&self, q: &mut KlPol, d: usize, defect: usize) -> KlPol {
        debug_assert!(q.as_slice().len() <= d / 2 + 2 && q.as_slice().len() <= d + 1);
        let m_deg = 2 * q.degree() as isize - d as isize; // used only when >= 0
        let mut m = KlPol::zero();

        if defect == 0 {
            if q.as_slice().len() <= d.div_ceil(2) {
                return m; // no correction needed: deg(Q) < d/2
            }
            let m_deg = m_deg as usize;
            debug_assert!(m_deg < 3);
            let top = q.coefficient(q.degree());
            m = KlPol::monomial(m_deg).scaled(top);
            if m_deg > 0 {
                m = m.add(&KlPol::monomial(0).scaled(top)); // symmetrise
                if m_deg == 2 {
                    // Sub-dominant coefficient (defect-0 case only).
                    m = m.add(&KlPol::monomial(1).scaled(q.coefficient(q.degree() - 1)));
                }
            }
            debug_assert!(q.degree() >= m_deg);
            let shift = q.degree() - m_deg;
            q.sub_shifted_assign(&m, shift, 1);
            return m;
        }

        // Defect 1: ensure `1 + q` divides `Q - q^{(d - deg m)/2} m`.
        if q.as_slice().len() > d / 2 + 1 {
            let m_deg = m_deg as usize;
            debug_assert!(m_deg != 0 && m_deg < 3);
            let top = q.coefficient(q.degree());
            m = KlPol::monomial(m_deg)
                .scaled(top)
                .add(&KlPol::monomial(0).scaled(top)); // symmetrise
            debug_assert!(q.degree() >= m_deg);
            let shift = q.degree() - m_deg;
            q.sub_shifted_assign(&m, shift, 1);
            debug_assert!(q.as_slice().len() <= d / 2 + 1);
        }
        let (quotient, c) = factor_by_1_plus_q(q, d / 2);
        *q = quotient;
        debug_assert!(c == 0 || d.is_multiple_of(2));
        if c == 0 {
            return m;
        }
        // Add the constant `c` to `m`, as `c r^d` had to be subtracted.
        if m.is_zero() {
            return KlPol::monomial(0).scaled(c);
        }
        debug_assert!(m_deg == 2);
        m.add(&KlPol::monomial(1).scaled(c)) // M[1] was zero here
    }

    /// Upstream `fill_column` (ext_kl.cpp:514-593): compute column `y`.
    /// `column[y]` is built locally (zero-initialised like upstream's
    /// `assign`, ext_kl.cpp:517) so `P(·, y)` lookups during the recursion
    /// see the entries already written (larger `x`, written from the back).
    fn fill_column(&mut self, y: BlockElt) -> Result<(), StructureError> {
        let mut column = vec![ZERO; self.aux.col_size(y)];

        let Some((s, sy)) = self.has_direct_recursion(y) else {
            self.do_new_recursion(y, &mut column)?;
            self.column[y] = column;
            return Ok(());
        };

        let defect = usize::from(self.descent_type(s, y).has_defect());
        let sign = self.aux.block.epsilon(s, sy, y);
        let floor_y = self.aux.length_floor(y);

        let mut cy = vec![KlPol::zero(); floor_y];
        let mut ms = vec![KlPol::zero(); floor_y];

        // Initial contributions from `(T_s + 1) * c_{sy}` at descents.
        for x in (0..floor_y).rev() {
            if self.aux.is_descent(s, x) {
                cy[x] = self.product_comp(x, s, sy);
            }
        }

        // Downward pass, refining coefficients to meet the degree bound.
        for u in (0..floor_y).rev() {
            if !self.aux.is_descent(s, u) {
                continue;
            }
            let d0 = self.aux.block.l(y, u) + defect;
            if cy[u].is_zero() {
                continue;
            }
            ms[u] = self.extract_m(&mut cy[u], d0, defect);
            if ms[u].is_zero() {
                continue;
            }
            let d = d0 - ms[u].degree();
            debug_assert!(d.is_multiple_of(2));
            let d = d / 2;
            for x in (0..self.aux.length_floor(u)).rev() {
                if self.aux.is_descent(s, x) {
                    // Subtract `q^d * M_u * P_{x,u}` (ext_kl.cpp:559).
                    let term = pol_mul(&self.p(x, u), &ms[u]);
                    cy[x].sub_shifted_assign(&term, d, 1);
                }
            }
        }

        // Copy the coefficients to the column, primitives in decreasing
        // order (upstream's reverse iterator, ext_kl.cpp:564-589).
        let mut slot = column.len();
        let mut x = floor_y;
        while self.aux.prim_back_up(&mut x, y) {
            slot -= 1;
            if self.aux.is_descent(s, x) {
                column[slot] = self.pool.match_pol(&cy[x].scaled(sign));
            } else {
                // `s` is a non-good (double-valued) ascent of `x`.
                debug_assert!(self.descent_type(s, x).has_double_image());
                let (first, second) = self.aux.block.cayleys(s, x);
                let Some(first) = first else {
                    column[slot] = ZERO; // edge of a partial block
                    continue;
                };
                let mut q = self.p_in(first, y, &column); // written above
                if self.aux.block.epsilon(s, x, first) < 0 {
                    q = q.scaled(-1);
                }
                if let Some(second) = second {
                    let p2 = self.p_in(second, y, &column);
                    if self.aux.block.epsilon(s, x, second) > 0 {
                        q.add_assign(&p2);
                    } else {
                        q.sub_assign(&p2);
                    }
                }
                column[slot] = self.pool.match_pol(&q);
            }
        }
        debug_assert!(slot == 0);
        self.column[y] = column;
        Ok(())
    }

    /// Upstream `do_new_recursion` (ext_kl.cpp:608-831): compute column `y`
    /// when `y` has no direct recursion, so some generator is real
    /// nonparity for `y`.
    fn do_new_recursion(
        &mut self,
        y: BlockElt,
        column: &mut [KLIndex],
    ) -> Result<(), StructureError> {
        let floor_y = self.aux.length_floor(y);

        // One `M` vector per real-nonparity generator of `y`
        // (ext_kl.cpp:612-616).
        let mut rn_for_y: Vec<(usize, Vec<KlPol>)> = Vec::new();
        for s in 0..self.rank() {
            if self.descent_type(s, y).is_like_nonparity() {
                rn_for_y.push((s, vec![KlPol::zero(); floor_y]));
            }
        }
        // Upstream's `#ifndef NDEBUG` down-set defect check
        // (ext_kl.cpp:618-637) is not ported; it has no production effect.

        let mut slot = column.len(); // upstream's reverse output iterator
        for x in (0..floor_y).rev() {
            if self.is_primitive(x, y) {
                // Compute `P_{x,y}` and store at the next slot from the back.
                if !self.is_extremal(x, y) {
                    // Primitive though not extremal: combine the (at most)
                    // two Cayley ascents by hand (ext_kl.cpp:645-666).
                    let s = self
                        .aux
                        .easy_set(x, y)
                        .first_bit()
                        .expect("non-extremal primitive has an easy ascent");
                    debug_assert!(!self.aux.is_descent(s, x));
                    debug_assert!(self.descent_type(s, x).has_double_image());
                    let (first, second) = self.aux.block.cayleys(s, x);
                    let mut pxy = KlPol::zero();
                    if let Some(first) = first {
                        // `P(x', y)` for `x' > x` is already computed.
                        pxy = self.p_in(first, y, column);
                        if self.aux.block.epsilon(s, x, first) < 0 {
                            pxy = pxy.scaled(-1);
                        }
                        if let Some(second) = second {
                            let p2 = self.p_in(second, y, column);
                            if self.aux.block.epsilon(s, x, second) > 0 {
                                pxy.add_assign(&p2);
                            } else {
                                pxy.sub_assign(&p2);
                            }
                        }
                    }
                    slot -= 1;
                    column[slot] = self.pool.match_pol(&pxy);
                } else {
                    // `x` is extremal: seek a proper generator `s` that is
                    // real nonparity for `y` (ext_kl.cpp:668-683).
                    let mut selected: Option<(usize, DescValue)> = None;
                    for (idx, &(s, _)) in rn_for_y.iter().enumerate() {
                        let tsx = self.descent_type(s, x);
                        if tsx.is_proper_ascent() {
                            if !tsx.is_like_type_1() {
                                selected = Some((idx, tsx));
                                break;
                            }
                            // Type 1: the cross neighbour might help.
                            if let Some(csx) = self.aux.block.cross(s, x) {
                                if !self.is_extremal(csx, y) {
                                    selected = Some((idx, tsx));
                                    break;
                                }
                            }
                        } else if tsx.is_like_compact() {
                            selected = Some((idx, tsx));
                            break;
                        }
                    }

                    let mut q = KlPol::zero(); // default: no good `s` found
                    if let Some((idx, tsx)) = selected {
                        let s = rn_for_y[idx].0;
                        let m_vec = &rn_for_y[idx].1;
                        let k = self.aux.block.orbit(s).length();
                        let last_u = self.aux.block.length_first(self.aux.block.length(x) + 1);

                        // `Q = sum_{x<u<y, s in tau(u)} r^{l(y/u)+k} P_{x,u} m_s(u,y)`
                        // (ext_kl.cpp:693-696).
                        for u in (last_u..floor_y).rev() {
                            if self.descent_type(s, u).is_descent() && !m_vec[u].is_zero() {
                                let shift = (self.aux.block.l(y, u) + k - m_vec[u].degree()) / 2;
                                let term = pol_mul(&self.p(x, u), &m_vec[u]);
                                q.add_shifted_assign(&term, shift);
                            }
                        }

                        // Subtract the ascent terms of `x` and divide by
                        // the diagonal coefficient (ext_kl.cpp:699-802).
                        self.new_recursion_finish(&mut q, tsx, s, x, y, k, floor_y, column)?;
                    }
                    slot -= 1;
                    column[slot] = self.pool.match_pol(&q);
                }
            }

            // The remainder of the loop body runs for every `x`
            // (ext_kl.cpp:810-827): defect-ascent `mu(1,x,y)` updates...
            if self.aux.block.l(y, x) == 1 + 2 * self.p_in(x, y, column).degree() {
                for (s, m_vec) in rn_for_y.iter_mut() {
                    let tsx = self.descent_type(*s, x);
                    if !tsx.is_descent() && tsx.has_defect() {
                        let sx = self
                            .aux
                            .block
                            .cayley(*s, x)
                            .expect("defect ascent has a Cayley link");
                        debug_assert!(sx < floor_y);
                        let mu = self
                            .p_in(x, y, column)
                            .coefficient(self.aux.block.l(y, x) / 2)
                            * self.aux.block.epsilon(*s, x, sx);
                        let d = usize::from(m_vec[sx].degree() == 2);
                        m_vec[sx].add_assign(&KlPol::monomial(d).scaled(mu));
                    }
                }
            }
            // ...and the `M[x]` entries themselves.
            for idx in 0..rn_for_y.len() {
                let s = rn_for_y[idx].0;
                if self.descent_type(s, x).is_descent() {
                    let mp = self.get_mp(s, x, y, &rn_for_y[idx].1, column);
                    rn_for_y[idx].1[x] = mp;
                }
            }
        }
        debug_assert!(slot == 0);
        Ok(())
    }

    /// The `switch (tsx)` of `do_new_recursion` (ext_kl.cpp:699-802):
    /// subtract the contributions of the ascent(s) of `x` from `q` and
    /// divide by the diagonal coefficient of `(T_s + 1)` at `x`.
    #[allow(clippy::too_many_arguments)]
    fn new_recursion_finish(
        &self,
        q: &mut KlPol,
        tsx: DescValue,
        s: usize,
        x: BlockElt,
        y: BlockElt,
        k: usize,
        floor_y: BlockElt,
        column: &[KLIndex],
    ) -> Result<(), StructureError> {
        use DescValue::*;
        let block = self.aux.block;
        match tsx {
            // `is_complex(tsx)`: coefficient is `+- q^k` (ext_kl.cpp:701-710).
            OneComplexAscent | TwoComplexAscent | ThreeComplexAscent => {
                if let Some(sx) = block.cross(s, x) {
                    if sx < floor_y {
                        let coef = block.t_coef(s, x, sx);
                        q.sub_assign(&pol_mul_spol(&coef, &self.p_in(sx, y, column)));
                    }
                }
                // Implicit division by `T_coef(s,x,x) == 1`.
            }
            // `has_defect(tsx)`: coefficient `+-(q^k - q)`, then divide by
            // `1 + q` (ext_kl.cpp:711-731).
            TwoSemiImaginary | ThreeSemiImaginary | ThreeImaginarySemi => {
                if let Some(sx) = block.cayley(s, x) {
                    if sx < floor_y {
                        let coef = block.t_coef(s, x, sx);
                        q.sub_assign(&pol_mul_spol(&coef, &self.p_in(sx, y, column)));
                    }
                }
                let (quotient, _remainder) =
                    factor_by_1_plus_q(q, self.aux.block.l(y, x).div_ceil(2));
                *q = quotient;
            }
            // `is_like_type_2(tsx)`: two images, divide by 2
            // (ext_kl.cpp:732-744).
            OneImaginaryPairFixed | TwoImaginaryDoubleDouble => {
                let (first, second) = block.cayleys(s, x);
                for sx in [first, second].into_iter().flatten() {
                    if sx < floor_y {
                        let coef = block.t_coef(s, x, sx);
                        q.sub_assign(&pol_mul_spol(&coef, &self.p_in(sx, y, column)));
                    }
                }
                *q = q.divide_by_2()?;
            }
            // `is_like_type_1(tsx)`, the former endgame case
            // (ext_kl.cpp:745-779).
            OneImaginarySingle | TwoImaginarySingleSingle => {
                if let Some(x_prime) = block.cayley(s, x) {
                    if x_prime < floor_y {
                        let coef = block.t_coef(s, x, x_prime);
                        q.sub_assign(&pol_mul_spol(&coef, &self.p_in(x_prime, y, column)));
                    }
                }
                // Implicit division by `T_coef(s,x,x) == 1`; then subtract
                // `P_{s x, y}`, computed on the fly.
                let s_cross_x = block.cross(s, x).expect("type 1 ascent has a cross link");
                debug_assert!(!self.is_extremal(s_cross_x, y));
                let t = self
                    .aux
                    .easy_set(s_cross_x, y)
                    .first_bit()
                    .expect("non-extremal element has an easy ascent");
                let ttscx = self.descent_type(t, s_cross_x);
                let eps_s = block.epsilon(s, x, s_cross_x);
                if ttscx.has_double_image() {
                    let (first, second) = block.cayleys(t, s_cross_x);
                    for sx in [first, second].into_iter().flatten() {
                        if sx < floor_y {
                            let sign = eps_s * block.epsilon(t, s_cross_x, sx);
                            q.sub_assign(&self.p_in(sx, y, column).scaled(sign));
                        }
                    }
                } else if let Some(sx) = block.cayley(t, s_cross_x) {
                    if sx < floor_y {
                        let sign = eps_s * block.epsilon(t, s_cross_x, sx);
                        q.sub_assign(&self.p_in(sx, y, column).scaled(sign));
                    }
                }
            }
            // The quadruple case: two images, divide by 2
            // (ext_kl.cpp:780-790).
            TwoImaginarySingleDoubleFixed => {
                let (first, second) = block.cayleys(s, x);
                for sx in [first, second].into_iter().flatten() {
                    if sx < floor_y {
                        let coef = block.t_coef(s, x, sx);
                        q.sub_assign(&pol_mul_spol(&coef, &self.p_in(sx, y, column)));
                    }
                }
                *q = q.divide_by_2()?;
            }
            // The compact and switched cases: nothing to subtract, divide
            // by `1 + q^k` (ext_kl.cpp:791-800).
            OneImaginaryCompact
            | OneRealPairSwitched
            | TwoImaginaryCompact
            | TwoRealSingleDoubleSwitched
            | ThreeImaginaryCompact => {
                *q = factor_by_1_plus_q_to_the(q, k, (self.aux.block.l(y, x) - 1) / 2 + k);
                debug_assert!(q.as_slice().len() <= self.aux.block.l(y, x).div_ceil(2));
            }
            _ => unreachable!("do_new_recursion: incompatible type selected"),
        }
        Ok(())
    }
}

/// Upstream `ext_block::condense` (ext_block.cpp:2015-2048, instantiated
/// with `Pol` at :2809), as a free function over the `KlPol` row matrix:
/// push every row `y` with a singular descent down to its descent rows
/// (sign `-epsilon`, ext_kl.cpp:2031 "surprise!"), and return the surviving
/// row numbers in increasing order. Rows of non-survivors are left dirty;
/// callers ignore them, exactly like upstream.
pub fn condense(eblock: &ExtBlock, m: &mut [Vec<KlPol>], sing_orbs: &RankFlags) -> Vec<BlockElt> {
    let mut survivors = Vec::new();
    for y in (0..m.len()).rev() {
        // The reverse loop is essential (ext_kl.cpp:2021).
        let Some(s) = eblock.first_descent_among(sing_orbs, y) else {
            survivors.push(y);
            continue;
        };
        let kind = eblock.descent_type(s, y);
        if kind.is_like_compact() {
            continue; // no descents: `y` represents zero
        }
        if kind.has_double_image() {
            let (first, second) = eblock.cayleys(s, y);
            let first = first.expect("condense: double image has a first link");
            let second = second.expect("condense: double image has a second link");
            row_operation(m, first, y, -eblock.epsilon(s, first, y));
            row_operation(m, second, y, -eblock.epsilon(s, second, y));
        } else {
            let x = eblock
                .some_scent(s, y)
                .expect("condense: descent has a link");
            row_operation(m, x, y, -eblock.epsilon(s, x, y));
        }
    }
    survivors.reverse(); // pushed in decreasing order
    survivors
}

/// `M.rowOperation(target, source, c)`: `row[target] += c * row[source]`.
fn row_operation(m: &mut [Vec<KlPol>], target: usize, source: usize, c: i32) {
    debug_assert!(target != source);
    let (target_row, source_row) = if target < source {
        let (low, high) = m.split_at_mut(source);
        (&mut low[target], &high[0])
    } else {
        let (low, high) = m.split_at_mut(target);
        (&mut high[0], &low[source])
    };
    for (entry, contribution) in target_row.iter_mut().zip(source_row.iter()) {
        if !contribution.is_zero() {
            entry.add_assign(&contribution.scaled(c));
        }
    }
}

/// The output of [`ext_kl_matrix`]: the condensed, sign-adjusted KLV
/// matrix and its pool encoding (upstream `P_mat`, `polys`, and
/// `P_index_mat` of ext_kl.cpp:939-1020), plus the surviving extended
/// block elements (upstream's `survivors`, which the wrapper maps to
/// `StandardRepr`s through `B.representative`/`rc.sr` — that mapping needs
/// the common-block slice and is not part of this port).
pub struct ExtKlMatrix {
    /// Extended block element numbers of the survivors, increasing.
    pub survivors: Vec<BlockElt>,
    /// The condensed square matrix of polynomials (after the odd-length
    /// parity sign flips of ext_kl.cpp:994-1003).
    pub matrix: Vec<Vec<KlPol>>,
    /// Pool indices of the matrix entries (ext_kl.cpp:1010-1016).
    pub index_matrix: Vec<Vec<KLIndex>>,
    /// The pool of the condensed matrix; entries 0 and 1 are 0 and 1.
    pub pool: KlHashTable,
}

/// Upstream `ext_KL_matrix` (ext_kl.cpp:939-1020), the part after the
/// extended block exists. `size` is upstream's `eblock.element
/// (entry_element+1)` (columns are filled up to and including the input
/// parameter); `singular_orbits` is upstream's `eblock.singular_orbits
/// (B.singular(gamma))`. Building the extended block from a `StandardRepr`
/// (mod-reduce, common context, common block) belongs to the common-block
/// slice.
pub fn ext_kl_matrix(
    eblock: &ExtBlock,
    size: BlockElt,
    singular_orbits: &RankFlags,
) -> Result<ExtKlMatrix, StructureError> {
    let mut table = ExtKlTable::new(eblock)?;
    table.fill_columns(size)?;

    // The expanded KLV matrix, upper triangular (ext_kl.cpp:967-970).
    let mut p_mat = vec![vec![KlPol::zero(); size]; size];
    for (x, row) in p_mat.iter_mut().enumerate() {
        for (y, entry) in row.iter_mut().enumerate().skip(x + 1) {
            *entry = table.p(x, y);
        }
    }

    // Push singular rows down to their descents (ext_kl.cpp:972-980).
    let survivors = condense(eblock, &mut p_mat, singular_orbits);

    // Compress to the surviving rows and columns (ext_kl.cpp:982-992).
    let n = survivors.len();
    let mut matrix = vec![vec![KlPol::zero(); n]; n];
    for (i, &si) in survivors.iter().enumerate() {
        for (j, &sj) in survivors.iter().enumerate().skip(i) {
            matrix[i][j] = p_mat[si][sj].clone();
        }
    }

    // Flip signs for odd length distance (ext_kl.cpp:994-1003).
    for j in 0..n {
        let parity = eblock.length(survivors[j]) % 2;
        for i in 0..j {
            if eblock.length(survivors[i]) % 2 != parity {
                matrix[i][j] = matrix[i][j].scaled(-1);
            }
        }
    }

    // Pool-encode the matrix (ext_kl.cpp:1009-1016).
    let mut pool = KlHashTable::new();
    let mut index_matrix = vec![vec![ZERO; n]; n];
    for j in 1..n {
        for i in 0..j {
            index_matrix[i][j] = pool.match_pol(&matrix[i][j]);
        }
    }

    Ok(ExtKlMatrix {
        survivors,
        matrix,
        index_matrix,
        pool,
    })
}

/// Upstream `flip` (repr.cpp:1849-1853): multiply every coefficient by
/// `sign` (a no-op for `sign == 1`), preserving the ascending order.
fn flip(sign: i32, list: Vec<(BlockElt, i32)>) -> Vec<(BlockElt, i32)> {
    if sign == 1 {
        list
    } else {
        list.into_iter()
            .map(|(z, coefficient)| (z, coefficient.wrapping_mul(sign)))
            .collect()
    }
}

/// Upstream `combine` (repr.cpp:1833-1847): merge two ascending-by-element
/// lists, accumulating the coefficients of like terms (which stay adjacent
/// after the merge; zero sums are *not* dropped, matching upstream).
fn combine(a: Vec<(BlockElt, i32)>, b: Vec<(BlockElt, i32)>) -> Vec<(BlockElt, i32)> {
    let mut merged = Vec::with_capacity(a.len() + b.len());
    let (mut i, mut j) = (0, 0);
    while i < a.len() && j < b.len() {
        if a[i].0 <= b[j].0 {
            merged.push(a[i]);
            i += 1;
        } else {
            merged.push(b[j]);
            j += 1;
        }
    }
    merged.extend_from_slice(&a[i..]);
    merged.extend_from_slice(&b[j..]);

    let mut result: Vec<(BlockElt, i32)> = Vec::with_capacity(merged.len());
    for (z, coefficient) in merged {
        match result.last_mut() {
            Some((last_z, last_c)) if *last_z == z => {
                *last_c = last_c.wrapping_add(coefficient);
            }
            _ => result.push((z, coefficient)),
        }
    }
    result
}

/// Upstream `contributions` for an extended block (repr.cpp:1901-1931):
/// expand the block elements `0..=y` into the extended-final ones for the
/// `singular_orbits` system, with the twisted signs: the October-surprise
/// sign from the link length change and each link's tuned `epsilon`.
/// Every returned list is sorted ascending by element.
pub fn contributions(
    eblock: &ExtBlock,
    singular_orbits: &RankFlags,
    y: BlockElt,
) -> Vec<Vec<(BlockElt, i32)>> {
    let mut result: Vec<Vec<(BlockElt, i32)>> = vec![Vec::new(); y + 1];
    for z in 0..=y {
        let Some(s) = eblock.first_descent_among(singular_orbits, z) else {
            // Extended final element: unit contribution to ourselves.
            result[z].push((z, 1));
            continue;
        };
        let kind = eblock.descent_type(s, z);
        if kind.is_like_compact() {
            continue; // no descents, |z| represents zero
        }
        let scent = eblock.some_scent(s, z).expect("descent has a scent link");
        // True link length change; the 2-case is the October surprise.
        let sign: i32 = if eblock.l(z, scent) == 2 { -1 } else { 1 };
        if kind.has_double_image() {
            // 1r1f, 2r11.
            let (first, second) = eblock.cayleys(s, z);
            let first = first.expect("double image has a first Cayley");
            let second = second.expect("double image has a second Cayley");
            result[z] = combine(
                flip(
                    sign.wrapping_mul(eblock.epsilon(s, first, z)),
                    result[first].clone(),
                ),
                flip(
                    sign.wrapping_mul(eblock.epsilon(s, second, z)),
                    result[second].clone(),
                ),
            );
        } else {
            result[z] = flip(
                sign.wrapping_mul(eblock.epsilon(s, scent, z)),
                result[scent].clone(),
            );
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::BlockGraph;
    use crate::{
        AdjointFiberBudget, BasedRootDatum, CartanClassification, CartanClassificationBudget,
        CartanId, Coweight, InnerClass, IntegerLatticeBudget, InvolutionTable,
        InvolutionTableBudget, KgbGraph, LatticeInvolution, RealFormSeed, StrongRealClassification,
        WeakRealFormId, Weight,
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

    fn lattice_budget() -> IntegerLatticeBudget {
        IntegerLatticeBudget::new(64, 100_000, 100_000, 128)
    }

    /// The KGB graph of the class's real form whose KGB size is `size`
    /// (same helper shape as ext_block.rs tests).
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
                &lattice_budget(),
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
            InvolutionTableBudget::new(64, lattice_budget()),
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
            InvolutionTableBudget::new(64, lattice_budget()),
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
    /// (KGB 2): 3 elements, as anchored in block.rs/ext_block.rs tests.
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

    /// The A2 root-lattice datum used by the ext_block.rs anchors.
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
    /// primal KGB size 6, dual class form size 4.
    fn a2_equal_rank_block() -> BlockFixture {
        let datum = a2_datum();
        let involution = LatticeInvolution::identity(&datum).unwrap();
        fixture(InnerClass::new(datum, involution, 8).unwrap(), 6, 4, 8)
    }

    /// The flipped A2 inner class (distinguished = diagram flip):
    /// quasisplit sl(3,R) primal KGB size 4, dual su(2,1) form size 6.
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

    /// The diagram-flip twist data: delta = flip on both A2 sides.
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

    /// Build the extended block of a fixture with the given twist data.
    fn ext_block_of(
        fixture: &BlockFixture,
        twists: &(LatticeInvolution, Vec<usize>, LatticeInvolution, Vec<usize>),
        cartan: &[Vec<i32>],
    ) -> ExtBlock {
        ExtBlock::build(
            &fixture.block,
            &fixture.graph,
            &fixture.table,
            &fixture.dual_graph,
            &fixture.dual_table,
            &twists.0,
            &twists.1,
            &twists.2,
            &twists.3,
            cartan,
        )
        .unwrap()
    }

    /// The pool polynomials as a set of coefficient vectors, for
    /// order-independent comparison with the oracle's `[vec]` pool output.
    fn pool_contents(pool: &KlHashTable) -> Vec<Vec<i32>> {
        let mut contents: Vec<Vec<i32>> = (0..pool.len())
            .map(|index| pool.get(index).expect("pool index").as_slice().to_vec())
            .collect();
        contents.sort();
        contents
    }

    /// Assert the twisted KLV matrix against the oracle's `raw_ext_KL`
    /// output. `expected[y][x]` for `x < y` is encoded as: 0 = zero
    /// polynomial, 1 = the constant 1, 2 = `q`. (No flipped entries occur
    /// in any of the anchor blocks: all oracle matrix entries are >= 0.)
    fn assert_raw_matrix(table: &ExtKlTable, expected: &[Vec<i32>]) {
        let size = table.size();
        for y in 0..size {
            assert_eq!(table.p(y, y).as_slice(), &[1], "P_{{{y},{y}}} should be 1");
            for x in 0..y {
                let actual = table.p(x, y);
                let want = match expected[y][x] {
                    0 => KlPol::zero(),
                    1 => KlPol::monomial(0),
                    2 => KlPol::monomial(1),
                    other => panic!("unexpected anchor code {other}"),
                };
                assert_eq!(actual.as_slice(), want.as_slice(), "P_{{{x},{y}}} mismatch");
            }
        }
    }

    /// Assert a condensed matrix against the oracle's
    /// `partial_extended_KL_block` output. `expected` holds pool indices;
    /// `pool_entries` maps pool index to the polynomial (0 = zero, 1 =
    /// constant 1, -1 = constant -1).
    fn assert_condensed(
        result: &ExtKlMatrix,
        expected: &[Vec<i32>],
        survivors: &[usize],
        pool_entries: &[i32],
    ) {
        assert_eq!(result.survivors, survivors, "survivors");
        let n = survivors.len();
        for y in 0..n {
            for x in 0..y {
                let want = KlPol::monomial(0).scaled(pool_entries[expected[y][x] as usize]);
                assert_eq!(
                    result.matrix[x][y].as_slice(),
                    want.as_slice(),
                    "condensed ({x},{y}) mismatch"
                );
                assert_eq!(
                    result
                        .pool
                        .get(result.index_matrix[x][y])
                        .expect("pool index")
                        .as_slice(),
                    want.as_slice(),
                    "pool encoding of ({x},{y}) mismatch"
                );
            }
        }
    }

    #[test]
    fn a2_trivial_delta_ext_kl_matches_oracle() {
        // Oracle: raw_ext_KL(trivial(SU(2,1)), id) gives a 6x6 matrix over
        // the pool [[ ],[ 1 ]] with length stops [ 0, 3, 5, 6 ]
        // (probe ekl_a2id_probe.at).
        let fixture = a2_equal_rank_block();
        let twists = identity_twists(&fixture);
        let cartan = vec![vec![2, -1], vec![-1, 2]];
        let eb = ext_block_of(&fixture, &twists, &cartan);
        assert_eq!(eb.size(), 6);

        let mut table = ExtKlTable::new(&eb).unwrap();
        table.fill_columns(0).unwrap();
        assert_eq!(table.polys().len(), 2, "pool holds only 0 and 1");

        #[rustfmt::skip]
        let expected: Vec<Vec<i32>> = vec![
            vec![],
            vec![0],
            vec![0, 0],
            vec![1, 0, 1],
            vec![1, 1, 0, 0],
            vec![1, 1, 1, 1, 1],
        ];
        assert_raw_matrix(&table, &expected);

        // Length stops [ 0, 3, 5, 6 ].
        assert_eq!(eb.length_first(0), 0);
        assert_eq!(eb.length_first(1), 3);
        assert_eq!(eb.length_first(2), 5);
        assert_eq!(eb.length_first(3), 6);

        // nonzero_column(5): every x <= 5 has nonzero P_{x,5}.
        assert_eq!(table.nonzero_column(5), vec![5, 4, 3, 2, 1, 0]);

        // Oracle: partial_extended_KL_block(trivial(SU(2,1)), id) keeps all
        // six elements (no singular roots), flips the odd-length-distance
        // signs, and uses the pool [[ ],[ 1 ],[ -1 ]].
        let result = ext_kl_matrix(&eb, 6, &RankFlags::empty()).unwrap();
        assert_eq!(pool_contents(&result.pool), vec![vec![], vec![-1], vec![1]]);
        #[rustfmt::skip]
        let expected_pm: Vec<Vec<i32>> = vec![
            vec![],
            vec![0],
            vec![0, 0],
            vec![2, 0, 2],
            vec![2, 2, 0, 0],
            vec![1, 1, 1, 2, 2],
        ];
        assert_condensed(&result, &expected_pm, &[0, 1, 2, 3, 4, 5], &[0, 1, -1]);
    }

    #[test]
    fn a2_flip_delta_ext_kl_matches_oracle() {
        // Oracle: raw_ext_KL(trivial(SL(3,R)), [[1,1],[0,-1]]) gives the
        // 2x2 matrix [[1,1],[0,1]] over pool [[ ],[ 1 ]] with length stops
        // [ 0, 1, 1, 2 ] (probe ekl_a2flip_probe.at). The extended block
        // has a single length-3 orbit; element 1 is ThreeRealSemi, so the
        // column is computed by the defect direct recursion.
        let fixture = a2_flipped_block();
        let twists = flip_twists(&fixture);
        let cartan = vec![vec![2, -1], vec![-1, 2]];
        let eb = ext_block_of(&fixture, &twists, &cartan);
        assert_eq!(eb.size(), 2);
        assert_eq!(eb.orbit(0).length(), 3);

        let mut table = ExtKlTable::new(&eb).unwrap();
        table.fill_columns(0).unwrap();
        assert_eq!(table.polys().len(), 2);
        #[rustfmt::skip]
        let expected: Vec<Vec<i32>> = vec![
            vec![],
            vec![1],
        ];
        assert_raw_matrix(&table, &expected);

        // Length stops [ 0, 1, 1, 2 ]: element 1 has parent length 2.
        assert_eq!(eb.length_first(0), 0);
        assert_eq!(eb.length_first(1), 1);
        assert_eq!(eb.length_first(2), 1);
        assert_eq!(eb.length_first(3), 2);

        // Oracle: partial_extended_KL_block(trivial(SL(3,R)), flip) keeps
        // both elements; same-parity lengths give no sign flips.
        let result = ext_kl_matrix(&eb, 2, &RankFlags::empty()).unwrap();
        assert_eq!(pool_contents(&result.pool), vec![vec![], vec![1]]);
        #[rustfmt::skip]
        let expected_pm: Vec<Vec<i32>> = vec![
            vec![],
            vec![1],
        ];
        assert_condensed(&result, &expected_pm, &[0, 1], &[0, 1]);
    }

    /// The C2 root-lattice datum (Bourbaki: alpha_0 short, alpha_1 long).
    fn c2_datum() -> BasedRootDatum {
        BasedRootDatum::from_simple_data(
            2,
            vec![vec![2, -1], vec![-2, 2]],
            vec![Weight::new(vec![2, -1]), Weight::new(vec![-2, 2])],
            vec![Coweight::new(vec![1, 0]), Coweight::new(vec![0, 1])],
        )
        .unwrap()
    }

    /// The Sp(4,R) block of the oracle's `print_block(trivial(Sp(4,R)))`:
    /// 12 elements over the primal KGB of size 11, with the dual-class
    /// form chosen so the block size is 12.
    fn sp4_block() -> BlockFixture {
        let datum = c2_datum();
        let involution = LatticeInvolution::identity(&datum).unwrap();
        let primal_class = InnerClass::new(datum, involution, 8).unwrap();
        let weyl = 8;
        let classification =
            CartanClassification::build(&primal_class, &class_budget(weyl)).unwrap();
        let strong = StrongRealClassification::build(&classification, 4_096).unwrap();
        let mut table = InvolutionTable::new(
            &primal_class,
            InvolutionTableBudget::new(64, lattice_budget()),
        )
        .unwrap();
        let (graph, primal_table) =
            graph_with_size(&primal_class, &classification, &strong, &mut table, 11);

        let dual_class = crate::dual::dual_inner_class(&primal_class, weyl, 64).unwrap();
        let dual_classification =
            CartanClassification::build(&dual_class, &class_budget(weyl)).unwrap();
        let dual_strong = StrongRealClassification::build(&dual_classification, 4_096).unwrap();
        let dual_sizes: Vec<usize> = (0..dual_classification.weak_real_form_count())
            .filter_map(|form| dual_strong.kgb_size(WeakRealFormId(form)))
            .collect();

        // Try each dual form KGB size until the block has the oracle's 12
        // elements with x-coordinates [0..=10, 10].
        let mut found = None;
        for &dual_size in &dual_sizes {
            let mut dual_table = InvolutionTable::new(
                &dual_class,
                InvolutionTableBudget::new(64, lattice_budget()),
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
            let xs: Vec<usize> = (0..block.size())
                .map(|z| block.x(z).unwrap().index())
                .collect();
            if block.size() == 12 && xs == (0..=10).chain(std::iter::once(10)).collect::<Vec<_>>() {
                found = Some((dual_graph, dual_table, block));
                break;
            }
        }
        let (dual_graph, dual_table, block) =
            found.unwrap_or_else(|| panic!("no dual form gives the oracle block: {dual_sizes:?}"));
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

    #[test]
    fn sp4_trivial_delta_ext_kl_matches_oracle() {
        // Oracle: raw_ext_KL(trivial(Sp(4,R)), id) gives a 12x12 matrix
        // over the pool [[ ],[ 1 ],[ 0, 1 ]] (so index 2 is `q`) with
        // length stops [ 0, 4, 7, 10, 12 ] (probe ekl_sp4_probe.at).
        let fixture = sp4_block();
        let twists = identity_twists(&fixture);
        let cartan = vec![vec![2, -1], vec![-2, 2]];
        let eb = ext_block_of(&fixture, &twists, &cartan);
        assert_eq!(eb.size(), 12);

        // The oracle's extended_block types (probe output rows), as
        // DescValue indices; this also pins the fixture's element order.
        #[rustfmt::skip]
        let expected_types: Vec<Vec<usize>> = vec![
            vec![2, 2], vec![2, 2], vec![9, 2], vec![9, 2],
            vec![3, 0], vec![0, 3], vec![0, 3], vec![1, 2],
            vec![1, 2], vec![4, 1], vec![5, 3], vec![5, 8],
        ];
        for n in 0..12 {
            for s in 0..2 {
                assert_eq!(
                    eb.descent_type(s, n) as usize,
                    expected_types[n][s],
                    "type of ({s},{n})"
                );
            }
        }

        // Length stops [ 0, 4, 7, 10, 12 ].
        assert_eq!(eb.length_first(0), 0);
        assert_eq!(eb.length_first(1), 4);
        assert_eq!(eb.length_first(2), 7);
        assert_eq!(eb.length_first(3), 10);
        assert_eq!(eb.length_first(4), 12);

        let mut table = ExtKlTable::new(&eb).unwrap();
        table.fill_columns(0).unwrap();
        assert_eq!(
            pool_contents(table.polys()),
            vec![vec![], vec![0, 1], vec![1]],
            "pool holds 0, 1, and q"
        );

        #[rustfmt::skip]
        let expected: Vec<Vec<i32>> = vec![
            vec![],
            vec![0],
            vec![0, 0],
            vec![0, 0, 0],
            vec![1, 1, 0, 0],
            vec![1, 0, 1, 0, 0],
            vec![0, 1, 0, 1, 0, 0],
            vec![1, 1, 1, 0, 1, 1, 0],
            vec![1, 1, 0, 1, 1, 0, 1, 0],
            vec![1, 1, 1, 1, 1, 1, 1, 0, 0],
            vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            vec![0, 0, 2, 2, 0, 0, 0, 0, 0, 1, 0],
        ];
        assert_raw_matrix(&table, &expected);

        // Oracle: partial_extended_KL_block(trivial(Sp(4,R)), id) covers
        // only the elements up to and including the input parameter
        // (element 10), so element 11 is out of range; there are no
        // singular roots, and the pool is [[ ],[ 1 ],[ -1 ]]
        // (probe ekl_sp4_partial_probe.at).
        let result = ext_kl_matrix(&eb, 11, &RankFlags::empty()).unwrap();
        assert_eq!(pool_contents(&result.pool), vec![vec![], vec![-1], vec![1]]);
        #[rustfmt::skip]
        let expected_pm: Vec<Vec<i32>> = vec![
            vec![],
            vec![0],
            vec![0, 0],
            vec![0, 0, 0],
            vec![2, 2, 0, 0],
            vec![2, 0, 2, 0, 0],
            vec![0, 2, 0, 2, 0, 0],
            vec![1, 1, 1, 0, 2, 2, 0],
            vec![1, 1, 0, 1, 2, 0, 2, 0],
            vec![1, 1, 1, 1, 2, 2, 2, 0, 0],
            vec![2, 2, 2, 2, 1, 1, 1, 2, 2, 2],
        ];
        let survivors: Vec<usize> = (0..=10).collect();
        assert_condensed(&result, &expected_pm, &survivors, &[0, 1, -1]);
    }

    #[test]
    fn contributions_a1_trivial_delta() {
        // The A1 extended block (trivial delta): z0, z1 are 1i1 (final for
        // the singular generator), z2 is 1r1f with Cayleys (z0, z1), both
        // length distance 1 and unflipped, so z2 expands to z0 + z1.
        let fixture = a1_block();
        let eb = ext_block_of(&fixture, &identity_twists(&fixture), &[vec![2]]);
        let mut singular = RankFlags::empty();
        singular.set(0);
        assert_eq!(
            contributions(&eb, &singular, 2),
            vec![vec![(0, 1)], vec![(1, 1)], vec![(0, 1), (1, 1)],]
        );
        // With no singular orbits every element is final.
        assert_eq!(
            contributions(&eb, &RankFlags::empty(), 2),
            vec![vec![(0, 1)], vec![(1, 1)], vec![(2, 1)]]
        );
    }

    #[test]
    fn contributions_a2_equal_rank_trivial_delta() {
        // Types per (element, generator) from the ext_block.rs anchors:
        // z0 [1i1,1i1], z1 [1i1,1iC], z2 [1iC,1i1], z3 [1C+,1r1f],
        // z4 [1r1f,1C+], z5 [1C-,1C-]. For the all-singular system: z0 is
        // final; z1, z2 are like-compact (zero); z3 and z4 expand to z0
        // via their 1r1f pair (the other image is a zero element); z5
        // expands through its 1C- cross to z3, hence to z0.
        let fixture = a2_equal_rank_block();
        let cartan = vec![vec![2, -1], vec![-1, 2]];
        let eb = ext_block_of(&fixture, &identity_twists(&fixture), &cartan);
        let mut singular = RankFlags::empty();
        singular.set(0);
        singular.set(1);
        assert_eq!(
            contributions(&eb, &singular, 5),
            vec![
                vec![(0, 1)],
                vec![],
                vec![],
                vec![(0, 1)],
                vec![(0, 1)],
                vec![(0, 1)],
            ]
        );
    }

    #[test]
    fn contributions_a2_flip_october_sign() {
        // The folded A2 block: element 0 is 3Ci (final), element 1 is 3r
        // with its link at length distance 2 (the October surprise), so
        // the untuned expansion is z1 -> -z0.
        let fixture = a2_flipped_block();
        let cartan = vec![vec![2, -1], vec![-1, 2]];
        let eb = ext_block_of(&fixture, &flip_twists(&fixture), &cartan);
        let mut singular = RankFlags::empty();
        singular.set(0);
        assert_eq!(
            contributions(&eb, &singular, 1),
            vec![vec![(0, 1)], vec![(0, -1)]]
        );
    }
}
