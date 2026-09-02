//! The Kazhdan-Lusztig-Vogan polynomial table (gkmod/kl.{h,cpp}).
//!
//! `KlTable` computes and stores the KLV polynomials `P_{x,y}` for a
//! block of representations (kl.h:64-173). The storage layout follows
//! upstream: column `y` holds `d_KL[y]`, indexed by the primitive-index
//! position of `x` for the descent set of `y`; the μ-coefficients live in
//! `d_mu[y]`; the polynomial content is deduplicated in a shared pool
//! (kl.cpp:100-101 initialises `zero` at index 0 and `one` at index 1).
//!
//! The fill algorithm (kl.cpp `fill_KL_column` + `recursion_column` /
//! `new_recursion_column`) computes column `y` from the already-filled
//! columns of shorter elements. A column with a direct recursion (a
//! complex or real-type-I descent of `y`) uses `recursion_column`; the
//! generic case uses `new_recursion_column`, which distinguishes on `x`
//! via the "nice and real" and "endgame" cases of recursion.pdf.

use std::sync::Arc;

use crate::block::BlockDescent;
use crate::kl_polynomial::{KlHashTable, KlPol};
use crate::kl_support::{KlSupport, RankFlags};
use crate::{BlockGraph, BlockTopology, PartialBlock, StructureError};

pub type BlockElt = usize;
pub type KlIndex = usize;
pub type MuCoeff = i32;

/// One μ-pair: `(x, μ(x,y))` (kl.h:41-45 `Mu_pair`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MuPair {
    pub x: BlockElt,
    pub coef: MuCoeff,
}

/// Storage behind the source-compatible [`KlTable`] alias.
#[doc(hidden)]
pub struct KlTableHandle<B: BlockTopology> {
    support: KlSupport<B>,
    /// Columns yet to compute (kl.h:72 `d_holes`).
    holes: Vec<bool>,
    /// `d_KL[y]`: per primitive element of `y`, the pool index of
    /// `P_{x,y}` (kl.h:74-75).
    columns: Vec<Vec<KlIndex>>,
    /// `d_mu[y]`: nonzero μ-coefficients for column `y` (kl.h:77-78).
    mu_columns: Vec<Vec<MuPair>>,
    /// The polynomial pool, sharing `zero`/`one` slots.
    pool: KlHashTable,
}

/// The KLV polynomial table for one block.
///
/// The lifetime parameter is retained only in this compatibility alias so
/// existing `KlTable<'_>` annotations continue to mean a table borrowing a
/// [`BlockGraph`]. The storage itself is generic solely over its handle type.
pub type KlTable<'a, B = &'a BlockGraph> = KlTableHandle<B>;

/// A KL table that owns a shared partial/common block handle.
pub type SharedKlTable = KlTableHandle<Arc<PartialBlock>>;

impl<'a> KlTableHandle<&'a BlockGraph> {
    /// `KL_table::KL_table` (kl.cpp:94-104): allocate columns for the
    /// whole block, mark every column as a hole, and seed the pool.
    /// Every element's descent set gets its primitive-index table now,
    /// so the recursion can look up any shorter column.
    pub fn new(block: &'a BlockGraph) -> Result<Self, StructureError> {
        Self::from_handle(block)
    }
}

impl<B: BlockTopology> KlTableHandle<B> {
    /// Construct a KL table that stores an arbitrary borrowed or owned block
    /// handle. `new` remains the compatibility constructor for
    /// `&BlockGraph`; owned callers use this entry point.
    pub fn from_handle(block: B) -> Result<Self, StructureError> {
        let size = block.size();
        let mut support = KlSupport::new(block)?;
        let pool = KlHashTable::new();
        for y in 0..size {
            let desc_y = support.descent_set(y).clone();
            support.prepare_prim_index(&desc_y);
        }
        Ok(Self {
            support,
            holes: vec![true; size],
            columns: vec![Vec::new(); size],
            mu_columns: vec![Vec::new(); size],
            pool,
        })
    }

    pub fn support(&self) -> &KlSupport<B> {
        &self.support
    }

    /// The shared polynomial pool (kl.h:123 `pol_store`).
    pub fn pool(&self) -> &KlHashTable {
        &self.pool
    }

    /// The pool index of the KLV polynomial `P_{x,y}` (kl.cpp:124-148):
    /// primitivise `x` for the descent set of `y`, then read the column.
    pub fn kl_pol(&self, x: BlockElt, y: BlockElt) -> Result<KlIndex, StructureError> {
        let desc_y = self.support.descent_set(y);
        let prim = self.primitive_index_of(x, desc_y)?;
        let column = self.columns.get(y).ok_or(StructureError::IndexOutOfRange {
            index: y,
            upper_bound: self.columns.len(),
        })?;
        // kl.cpp:129-131: when `prim` is at or past the column end
        // (l(x) >= l(y), including a failed primitivisation), the answer
        // is the identity P_{y,y}=1 when `x` primitivises to `y` itself,
        // else the zero polynomial.
        if prim >= column.len() {
            return Ok(if prim == self.support.self_index(y) {
                1
            } else {
                0
            });
        }
        Ok(column[prim])
    }

    /// The μ-coefficient μ(x,y) (kl.cpp:150-161).
    pub fn mu(&self, x: BlockElt, y: BlockElt) -> Option<MuCoeff> {
        self.mu_columns
            .get(y)?
            .iter()
            .find(|pair| pair.x == x)
            .map(|pair| pair.coef)
    }

    /// The list of nonzero μ-pairs for column `y` (kl.h:128).
    pub fn mu_column(&self, y: BlockElt) -> &[MuPair] {
        self.mu_columns.get(y).map_or(&[], Vec::as_slice)
    }

    /// The nonzero-KL bitmap for column `y` (kl.cpp:223-232 `prim_map`).
    pub fn prim_map(&self, y: BlockElt) -> Vec<bool> {
        self.columns
            .get(y)
            .map(|column| column.iter().map(|&index| index != 0).collect::<Vec<_>>())
            .unwrap_or_default()
    }

    /// The primitive-index position of `x` for the descent set of `y`
    /// (klsupport.h `prim_index`), preparing the table if needed.
    fn primitive_index_of(&self, x: BlockElt, desc_y: &RankFlags) -> Result<usize, StructureError> {
        if x == self.support.size() {
            // UndefBlock: index is the range sentinel.
            return Ok(self.support.nr_of_primitives(desc_y));
        }
        Ok(self.support.prim_index(x, desc_y))
    }

    /// `KL_table::fill(limit)` (kl.cpp:188-221): compute columns up to
    /// (excluding) `limit`; `limit == 0` fills everything.
    pub fn fill(&mut self, limit: usize) -> Result<(), StructureError> {
        let limit = if limit == 0 {
            self.support.size()
        } else {
            limit
        };
        let mut working = Vec::with_capacity(self.support.size());
        for y in 0..limit.min(self.support.size()) {
            if !self.holes[y] {
                continue;
            }
            self.fill_kl_column(&mut working, y)?;
            self.holes[y] = false;
        }
        Ok(())
    }

    /// `KL_table::fill_KL_column` (kl.cpp:350-363): compute column `y`.
    fn fill_kl_column(
        &mut self,
        working: &mut Vec<KlPol>,
        y: BlockElt,
    ) -> Result<(), StructureError> {
        // Prepare the primitive index for y's descent set (kl.cpp:353).
        let desc_y = self.support.descent_set(y).clone();
        self.support.prepare_prim_index(&desc_y);
        let s = self.first_direct_recursion(y);
        if s < self.support.rank() {
            self.recursion_column(working, y, s)?;
            self.complete_primitives(working, y)?;
        } else {
            self.new_recursion_column(working, y)?;
        }
        Ok(())
    }

    /// `KL_table::first_direct_recursion` (kl.cpp:249-259): the first
    /// generator that is a complex descent or real type I descent of `y`;
    /// `rank()` when none.
    fn first_direct_recursion(&self, y: BlockElt) -> usize {
        for s in 0..self.support.rank() {
            let value = self.support.block().descent(y, s).expect("valid generator");
            if matches!(
                value,
                BlockDescent::ComplexDescent | BlockDescent::RealTypeI
            ) {
                return s;
            }
        }
        self.support.rank()
    }

    /// `KL_table::first_nice_and_real` (kl.cpp:270-286): the first real
    /// nonparity ascent for `y` that is complex ascent, imaginary type 2,
    /// or compact imaginary for `x`; `rank()` when none.
    fn first_nice_and_real(&self, x: BlockElt, y: BlockElt) -> usize {
        for s in 0..self.support.rank() {
            let dy = self.support.block().descent(y, s).expect("valid generator");
            if dy == BlockDescent::RealNonparity {
                let dx = self.support.block().descent(x, s).expect("valid generator");
                if matches!(
                    dx,
                    BlockDescent::ComplexAscent
                        | BlockDescent::ImaginaryTypeII
                        | BlockDescent::ImaginaryCompact
                ) {
                    return s;
                }
            }
        }
        self.support.rank()
    }

    /// `KL_table::recursion_column` (kl.cpp:381-450): the direct-recursion
    /// formula for column `y` with descent `s`, for all extremal `x`.
    fn recursion_column(
        &mut self,
        working: &mut Vec<KlPol>,
        y: BlockElt,
        s: usize,
    ) -> Result<(), StructureError> {
        working.clear();
        let desc_y = self.support.descent_set(y).clone();
        let sy = match self.support.block().descent(y, s).expect("valid generator") {
            BlockDescent::ComplexDescent => self
                .support
                .block()
                .cross(y, s)
                .expect("complex descent cross"),
            _ => self
                .support
                .block()
                .inverse_cayley(y, s)
                .expect("real type I inverse Cayley")
                .0
                .expect("real type I has first image"),
        };

        // Increasing list of all extremal elements shorter than y.
        let floor = self.support.length_floor(y);
        let mut extremals = Vec::new();
        for x in 0..floor {
            if self.support.is_extremal(x, &desc_y) {
                extremals.push(x);
            }
        }

        for &x in &extremals {
            let sx = self.support.block().cross(x, s).expect("cross of extremal");
            let value = self.support.block().descent(x, s).expect("valid generator");
            let pxy = match value {
                BlockDescent::ImaginaryCompact => {
                    // (q+1)P_{x,sy}
                    self.kl_pol_pool(x, sy)?.shift()
                }
                BlockDescent::ComplexDescent => {
                    // P_{sx,sy} + q.P_{x,sy}
                    let first = self.kl_pol_pool(sx, sy)?;
                    let second = self.kl_pol_pool(x, sy)?;
                    first.add_shifted(second, 1)
                }
                BlockDescent::RealTypeI => {
                    // P_{sx.first,sy} + P_{sx.second,sy} + (q-1)P_{x,sy}
                    let pair = self.support.block().inverse_cayley(x, s).ok_or(
                        StructureError::BlockInvariantViolation {
                            invariant: "real type I inverse Cayley slot",
                        },
                    )?;
                    let Some(first_image) = pair.0 else {
                        return Err(StructureError::BlockInvariantViolation {
                            invariant: "real type I inverse Cayley first slot",
                        });
                    };
                    let mut result = self.kl_pol_pool(first_image, sy)?.clone();
                    if let Some(second) = pair.1 {
                        let second_pol = self.kl_pol_pool(second, sy)?;
                        result.add_assign(second_pol);
                    }
                    let xsypol = self.kl_pol_pool(x, sy)?;
                    result.add_shifted_assign(xsypol, 1);
                    result.sub_assign(xsypol);
                    result
                }
                BlockDescent::RealTypeII => {
                    // P_{sx.first,sy} + qP_{x,sy} - P_{s.x,sy}
                    // (kl.cpp:416-425): the first term is the inverse
                    // Cayley image of x, NOT the cross image; `cross` is
                    // only used for the subtraction term.
                    let pair = self.support.block().inverse_cayley(x, s).ok_or(
                        StructureError::BlockInvariantViolation {
                            invariant: "real type II inverse Cayley slot",
                        },
                    )?;
                    let Some(first_image) = pair.0 else {
                        return Err(StructureError::BlockInvariantViolation {
                            invariant: "real type II inverse Cayley first slot",
                        });
                    };
                    let first = self.kl_pol_pool(first_image, sy)?;
                    let second = self.kl_pol_pool(x, sy)?;
                    let third = self.kl_pol_pool(sx, sy)?;
                    let mut result = first.add_shifted(second, 1);
                    result.sub_assign(third);
                    result
                }
                _ => {
                    return Err(StructureError::RepInvariantViolation {
                        invariant: "recursion descent status",
                    })
                }
            };
            working.push(pxy);
        }
        // μ-correction (kl.cpp:447-448).
        self.mu_correction(&extremals, &desc_y, sy, s, working)?;
        Ok(())
    }

    /// `KL_table::mu_correction` (kl.cpp:480-525): subtract the μ-terms
    /// from every polynomial in `working`.
    fn mu_correction(
        &self,
        extremals: &[BlockElt],
        desc_y: &RankFlags,
        sy: BlockElt,
        s: usize,
        working: &mut [KlPol],
    ) -> Result<(), StructureError> {
        let ly = self.support.length(sy) + 1;
        // Iterate decreasing without cloning the column.
        for &MuPair { x: z, coef: mu } in self.mu_columns[sy].iter().rev() {
            let sz = self.support.block().descent(z, s).expect("valid generator");
            if !sz.is_descent() {
                continue;
            }
            let lz = self.support.length(z);
            let d = (ly - lz) / 2;
            for (position, &x) in extremals.iter().enumerate() {
                if self.support.length(x) >= lz {
                    break;
                }
                let pol = self.kl_pol_pool(x, z)?;
                working[position].sub_shifted_assign(pol, d, mu);
            }
            // The final term x == z (when extremal for y).
            if self.support.is_extremal(z, desc_y) {
                if let Some(position) = extremals.iter().position(|&x| x == z) {
                    let term = KlPol::monomial(d).scaled(mu);
                    working[position].sub_assign(&term);
                }
            }
        }
        Ok(())
    }

    /// `KL_table::complete_primitives` (kl.cpp:544-589): transfer the
    /// completed row to `d_KL[y]` and `d_mu[y]`, inserting polynomials for
    /// primitive non-extremal elements.
    fn complete_primitives(
        &mut self,
        working: &[KlPol],
        y: BlockElt,
    ) -> Result<(), StructureError> {
        let desc_y = self.support.descent_set(y).clone();
        let ly = self.support.length(y);
        // Write each slot at its primitive index (kl.cpp:547 KL.resize +
        // the backward write): the primitive non-extremal case reads the
        // Cayley images' slots of THIS column, which the backward pass
        // has already written ("in current row, above", kl.cpp:567).
        let mut column: Vec<KlIndex> = vec![0; self.support.col_size(y)];
        let mut mu_pairs: Vec<MuPair> = Vec::new();

        // Traverse primitives of y with length < ly backwards.
        let mut x = self.support.length_floor(y);
        let mut work_index = working.len(); // we read `working` backwards
        while self.support.prim_back_up(&mut x, &desc_y) {
            let position = self.support.prim_index(x, &desc_y);
            if self.support.is_extremal(x, &desc_y) {
                work_index -= 1;
                let pxy = &working[work_index];
                let index = self.pool.match_pol(pxy);
                column[position] = index;
                let lx = self.support.length(x);
                if !pxy.is_zero() && ly == lx + 2 * pxy.degree() + 1 {
                    mu_pairs.push(MuPair {
                        x,
                        coef: pxy.coefficient(pxy.degree()),
                    });
                }
            } else {
                // Primitive non-extremal: sum of the two cayley images'
                // polynomials (kl.cpp:566-574), looked up in the current
                // column above the traversal point.
                let s = self.support.ascent_descent(x, y).ok_or(
                    StructureError::RepInvariantViolation {
                        invariant: "primitive non-extremal ascent",
                    },
                )?;
                let mut pxy = KlPol::zero();
                if let Some((Some(first_image), second)) = self.support.block().cayley(x, s) {
                    pxy = self
                        .current_column_pol(&column, &desc_y, first_image, y)
                        .clone();
                    if let Some(second_image) = second {
                        let second_pol = self.current_column_pol(&column, &desc_y, second_image, y);
                        pxy.add_assign(second_pol);
                    }
                }
                column[position] = self.pool.match_pol(&pxy);
            }
        }

        // Add the down_set of y with μ=1 (kl.cpp:578-585).
        let downs: Vec<MuPair> = self
            .down_set(y)?
            .into_iter()
            .map(|z| MuPair { x: z, coef: 1 })
            .collect();
        let mut merged = downs;
        merged.extend(mu_pairs);
        merged.sort_by_key(|pair| pair.x);
        merged.dedup_by_key(|pair| pair.x);

        self.columns[y] = column;
        self.mu_columns[y] = merged;
        Ok(())
    }

    /// The down-set of `y` (blocks.cpp:204-229 `down_set`): the elements
    /// reached from `y` through its weak descents — a complex descent is
    /// the cross image, a real type I the two inverse-Cayley images, a
    /// real type II the first inverse-Cayley image; imaginary compact
    /// contributes nothing. Used for the μ-pairs of length one less than
    /// `y` (kl.cpp:578-585, 650-653).
    fn down_set(&self, y: BlockElt) -> Result<Vec<BlockElt>, StructureError> {
        let block = self.support.block();
        let mut result = Vec::new();
        for s in 0..self.support.rank() {
            match block.descent(y, s).expect("valid generator") {
                BlockDescent::ComplexDescent => {
                    let z = block.cross(y, s).expect("complex descent cross");
                    result.push(z);
                }
                BlockDescent::RealTypeI => {
                    let pair = block
                        .inverse_cayley(y, s)
                        .expect("real type I inverse Cayley");
                    if let Some(z) = pair.0 {
                        result.push(z);
                    }
                    if let Some(z) = pair.1 {
                        result.push(z);
                    }
                }
                BlockDescent::RealTypeII => {
                    let pair = block
                        .inverse_cayley(y, s)
                        .expect("real type II inverse Cayley");
                    if let Some(z) = pair.0 {
                        result.push(z);
                    }
                }
                _ => {}
            }
        }
        result.sort_unstable();
        result.dedup();
        Ok(result)
    }

    /// `KL_table::new_recursion_column` (kl.cpp:637-791): compute column
    /// `y` when no direct recursion exists.
    fn new_recursion_column(
        &mut self,
        working: &mut Vec<KlPol>,
        y: BlockElt,
    ) -> Result<(), StructureError> {
        let l_y = self.support.length(y);
        let desc_y = self.support.descent_set(y).clone();
        // The column holds one slot per primitive element (klsupport.h
        // `nr_of_primitives`), indexed by `prim_index`; `col_size` counts
        // only primitives of strictly smaller length, which is too small
        // when y itself is primitive.
        let height = self.support.nr_of_primitives(&desc_y);
        working.clear();
        working.resize(height + 1, KlPol::zero());
        working[self.support.self_index(y)] = KlPol::monomial(0); // P_{y,y} = 1

        // The lambda accessor: prim_index(x) locates KL_y(x) in `working`;
        // the loops below borrow the slot instead of cloning it.
        let kl_index = |x: BlockElt| self.support.prim_index(x, &desc_y);

        let mut mu_pairs: Vec<MuPair> = self
            .down_set(y)?
            .into_iter()
            .map(|z| MuPair { x: z, coef: 1 })
            .collect();
        let downs_len = mu_pairs.len();

        // Reverse loop through primitive elements.
        let mut x = self.support.length_less(l_y);
        while self.support.prim_back_up(&mut x, &desc_y) {
            let prim_pos = self.support.prim_index(x, &desc_y);
            let mut pxy = KlPol::zero();
            if let Some(s) = self.support.ascent_descent(x, y) {
                // Primitive but not extremal (kl.cpp:665-673). A missing
                // Cayley image (outside the block) contributes zero, like
                // KL_pol's UndefBlock handling (kl.cpp:127).
                if let Some((Some(first_image), second)) = self.support.block().cayley(x, s) {
                    pxy = working[kl_index(first_image)].clone();
                    if let Some(second_image) = second {
                        pxy.add_assign(&working[kl_index(second_image)]);
                    }
                }
                working[prim_pos] = pxy;
                continue;
            }

            // Now x is extremal for y.
            let l_x = self.support.length(x);
            let s = self.first_nice_and_real(x, y);
            if s < self.support.rank() {
                pxy = self.mu_new_formula(x, y, s, &mu_pairs)?;
                match self.support.block().descent(x, s).expect("valid generator") {
                    BlockDescent::ComplexAscent => {
                        let cross = self.support.block().cross(x, s).expect("cross");
                        pxy.sub_shifted_assign(&working[kl_index(cross)], 1, 1);
                    }
                    BlockDescent::ImaginaryTypeII => {
                        let pair = self.support.block().cayley(x, s).expect("cayley");
                        let mut sum = working[kl_index(pair.0.expect("first image"))].clone();
                        sum.add_assign(&working[kl_index(pair.1.expect("second image"))]);
                        pxy.add_assign(&sum);
                        pxy.sub_shifted_assign(&sum, 1, 1);
                        pxy.divide_by_2_assign()?;
                    }
                    BlockDescent::ImaginaryCompact => {
                        pxy = pxy.quotient_by_1_plus_q(l_y - l_x)?;
                    }
                    _ => {
                        return Err(StructureError::RepInvariantViolation {
                            invariant: "nice-and-real descent status",
                        })
                    }
                }
                if !pxy.is_zero() && l_y == l_x + 2 * pxy.degree() + 1 {
                    mu_pairs.push(MuPair {
                        x,
                        coef: pxy.coefficient(pxy.degree()),
                    });
                }
            } else {
                // No nice-and-real generator: endgame or zero.
                let st = self.first_endgame_pair(x, y);
                if let Some((s, t)) = st {
                    pxy = self.mu_new_formula(x, y, s, &mu_pairs)?;
                    let pair = self.support.block().cayley(x, s).expect("endgame cayley");
                    let p_xprime = kl_index(pair.0.expect("first image"));
                    pxy.add_assign(&working[p_xprime]);
                    pxy.sub_shifted_assign(&working[p_xprime], 1, 1);
                    if let Some(t) = t {
                        let sx = self.support.block().cross(x, s).expect("endgame cross");
                        let up = self.support.block().cayley(sx, t).expect("endgame up");
                        if let Some(first) = up.0 {
                            let first = kl_index(first);
                            pxy.sub_assign(&working[first]);
                        }
                        if let Some(second) = up.1 {
                            let second = kl_index(second);
                            pxy.sub_assign(&working[second]);
                        }
                    }
                    if !pxy.is_zero() && l_y == l_x + 2 * pxy.degree() + 1 {
                        mu_pairs.push(MuPair {
                            x,
                            coef: pxy.coefficient(pxy.degree()),
                        });
                    }
                }
                // else: P_{x,y} = 0, pxy stays zero.
            }
            working[prim_pos] = pxy;
        }

        // Transcribe to d_KL[y].
        let mut column = Vec::with_capacity(height);
        for polynomial in working.iter().take(height) {
            column.push(self.pool.match_pol(polynomial));
        }
        self.columns[y] = column;

        // Shuffle mu_pairs: initial part (downs) is increasing, remainder
        // decreasing; make everything increasing by x.
        let mut final_pairs: Vec<MuPair> = mu_pairs[downs_len..].to_vec();
        final_pairs.reverse();
        let mut downs = mu_pairs[..downs_len].to_vec();
        downs.extend(final_pairs);
        downs.sort_by_key(|pair| pair.x);
        downs.dedup_by_key(|pair| pair.x);
        self.mu_columns[y] = downs;
        Ok(())
    }

    /// `KL_table::mu_new_formula` (kl.cpp:813-841): the μ-sum in a new
    /// K-L recursion.
    fn mu_new_formula(
        &self,
        x: BlockElt,
        y: BlockElt,
        s: usize,
        mu_y: &[MuPair],
    ) -> Result<KlPol, StructureError> {
        let mut pol = KlPol::zero();
        let lx = self.support.length(x);
        let ly = self.support.length(y);
        for &MuPair { x: z, coef: mu } in mu_y {
            let lz = self.support.length(z);
            if lz <= lx {
                break;
            }
            let sz = self.support.block().descent(z, s).expect("valid generator");
            if !sz.is_descent() {
                continue;
            }
            let d = (ly - lz).div_ceil(2);
            let p_xz = self.kl_pol_pool(x, z)?;
            pol.add_shifted_scaled_assign(p_xz, d, mu);
        }
        Ok(pol)
    }

    /// `KL_table::first_endgame_pair` (kl.cpp:318-340): the endgame pair
    /// `(s, t)`, or `None`.
    fn first_endgame_pair(&self, x: BlockElt, y: BlockElt) -> Option<(usize, Option<usize>)> {
        let r = self.support.rank();
        for s in 0..r {
            let dy_s = self.support.block().descent(y, s)?;
            let dx_s = self.support.block().descent(x, s)?;
            if dy_s == BlockDescent::RealNonparity && dx_s == BlockDescent::ImaginaryTypeI {
                let sx = self.support.block().cross(x, s)?;
                let dsx = self.support.block().descent(sx, s)?;
                let _ = dsx;
                for t in 0..r {
                    let dy_t = self.support.block().descent(y, t)?;
                    if dy_t != BlockDescent::RealTypeII {
                        continue;
                    }
                    let dsx_t = self.support.block().descent(sx, t)?;
                    if matches!(
                        dsx_t,
                        BlockDescent::ImaginaryTypeI | BlockDescent::ImaginaryTypeII
                    ) {
                        return Some((s, Some(t)));
                    }
                }
                return Some((s, None));
            }
        }
        None
    }

    /// `KL_pol` for a pool lookup during recursion — the primitive index
    /// of `x` for the descent set of `y`, reading the already-filled
    /// column (kl.cpp:124-132 with the kl.cpp:129-131 out-of-range case).
    /// Returns a borrow into the pool; the recursion loops combine the
    /// polynomials with the in-place `KlPol` operations, so no
    /// coefficient vector is cloned per lookup.
    fn kl_pol_pool(&self, x: BlockElt, y: BlockElt) -> Result<&KlPol, StructureError> {
        let desc_y = self.support.descent_set(y);
        let prim = self.support.prim_index(x, desc_y);
        let column = self.columns.get(y).ok_or(StructureError::IndexOutOfRange {
            index: y,
            upper_bound: self.columns.len(),
        })?;
        let index = if prim >= column.len() {
            if prim == self.support.self_index(y) {
                1 // P_{y,y} = 1
            } else {
                0 // zero polynomial
            }
        } else {
            column[prim]
        };
        match self.pool.get(index) {
            Some(polynomial) => Ok(polynomial),
            None => Ok(KlPol::zero_ref()),
        }
    }

    /// `KL_pol(x, y)` against the column being written by
    /// `complete_primitives` (kl.cpp:566-570 reads the in-progress
    /// `d_KL[y]`): slots above the backward traversal point are final,
    /// and the out-of-range cases are the identity at `y` and zero —
    /// exactly the kl.cpp:129-131 logic over the partial column.
    fn current_column_pol(
        &self,
        column: &[KlIndex],
        desc_y: &RankFlags,
        x: BlockElt,
        y: BlockElt,
    ) -> &KlPol {
        let prim = self.support.prim_index(x, desc_y);
        let index = if prim >= column.len() {
            if prim == self.support.self_index(y) {
                1 // P_{y,y} = 1
            } else {
                0 // zero polynomial
            }
        } else {
            column[prim]
        };
        match self.pool.get(index) {
            Some(polynomial) => polynomial,
            None => KlPol::zero_ref(),
        }
    }
}
#[cfg(test)]
mod tests {
    use crate::block::BlockGraph;
    use crate::{
        AdjointFiberBudget, BasedRootDatum, CartanClassification, CartanClassificationBudget,
        CartanId, Coweight, IntegerLatticeBudget, InvolutionTable, InvolutionTableBudget, KgbGraph,
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

    /// The KGB graph of `inner_class`'s form whose expected size is
    /// `size`, with the involution table the graph was built against.
    fn graph_with_size(
        inner_class: &crate::InnerClass,
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

    /// The A2 compact inner class, quasisplit su(2,1) block: primal KGB
    /// size 6, dual (compact su(3)) size 1 → 6-element block.
    fn a2_block() -> (BlockGraph, InvolutionTable) {
        let datum = BasedRootDatum::from_simple_data(
            2,
            vec![vec![2, -1], vec![-1, 2]],
            vec![Weight::new(vec![2, -1]), Weight::new(vec![-1, 2])],
            vec![Coweight::new(vec![1, 0]), Coweight::new(vec![0, 1])],
        )
        .unwrap();
        let involution = LatticeInvolution::identity(&datum).unwrap();
        let inner_class = crate::InnerClass::new(datum, involution, 8).unwrap();
        let classification = CartanClassification::build(&inner_class, &class_budget(8)).unwrap();
        let strong = StrongRealClassification::build(&classification, 4_096).unwrap();
        let mut table = InvolutionTable::new(
            &inner_class,
            InvolutionTableBudget::new(64, IntegerLatticeBudget::new(64, 100_000, 100_000, 128)),
        )
        .unwrap();
        let (graph, primal_table) =
            graph_with_size(&inner_class, &classification, &strong, &mut table, 6);
        let dual_class = crate::dual::dual_inner_class(&inner_class, 8, 64).unwrap();
        let dual_classification =
            CartanClassification::build(&dual_class, &class_budget(8)).unwrap();
        let dual_strong = StrongRealClassification::build(&dual_classification, 4_096).unwrap();
        let dual_sizes: Vec<usize> = (0..dual_classification.weak_real_form_count())
            .filter_map(|form| dual_strong.kgb_size(WeakRealFormId(form)))
            .collect();
        // The dual real form for the deform block: its KGB has four
        // elements (the unique real form of the dual A2 inner class).
        assert_eq!(dual_sizes, vec![4], "dual KGB sizes: {dual_sizes:?}");
        let mut dual_table = InvolutionTable::new(
            &dual_class,
            InvolutionTableBudget::new(64, IntegerLatticeBudget::new(64, 100_000, 100_000, 128)),
        )
        .unwrap();
        let (dual_graph, _) = graph_with_size(
            &dual_class,
            &dual_classification,
            &dual_strong,
            &mut dual_table,
            4,
        );
        let block = BlockGraph::build(
            &graph,
            &primal_table,
            &dual_graph,
            &dual_table,
            &dual_class,
            8,
        )
        .unwrap();
        eprintln!("--- A2 block ({}) ---", block.size());
        for z in 0..block.size() {
            let desc: Vec<String> = (0..2)
                .map(|s| format!("{:?}", block.descent_value(z, s).unwrap()))
                .collect();
            eprintln!(
                "z={z} x={} y={} len={} d=[{}]",
                block.x(z).unwrap().index(),
                block.y(z).unwrap().index(),
                block.length(z).unwrap(),
                desc.join(", ")
            );
        }
        (block, dual_table)
    }

    #[test]
    fn rank_flags_arithmetic() {
        let mut flags = RankFlags::empty();
        flags.set(0);
        flags.set(2);
        assert!(flags.is_set(0));
        assert!(flags.is_set(2));
        assert_eq!(flags.first_bit(), Some(0));
        let other = RankFlags::empty();
        assert!(flags.contains(&other));
        assert!(!flags.none());
    }

    #[test]
    fn a2_quasisplit_block_fills_and_diagonal_is_one() {
        let (block, _table) = a2_block();
        assert_eq!(block.size(), 6);
        assert_eq!(block.length(0), Some(0));
        let mut kl = KlTable::new(&block).unwrap();
        kl.fill(0).unwrap();
        // P_{y,y} = 1 (pool index 1) for every element.
        for y in 0..block.size() {
            let index = kl.kl_pol(y, y).unwrap();
            let pol = kl.pool.get(index).cloned().unwrap();
            assert_eq!(pol.as_slice(), &[1], "P_{{{y},{y}}} should be 1");
        }
    }

    #[test]
    fn a2_quasisplit_block_mu_columns_are_sane() {
        let (block, _table) = a2_block();
        let mut kl = KlTable::new(&block).unwrap();
        kl.fill(0).unwrap();
        for y in 0..block.size() {
            // mu(x,y) is nonzero only for x of length one less than y
            // (triangularity), and mu(y,y) = 0 by definition.
            assert_eq!(kl.mu(y, y), None);
            for pair in kl.mu_column(y) {
                assert!(pair.x < y);
                assert!(pair.coef != 0);
                assert!(block.length(pair.x).unwrap() < block.length(y).unwrap());
            }
        }
    }

    #[test]
    fn a2_quasisplit_klv_mu_columns_reach_the_deform_sources() {
        // The frozen deform contract (job 3506415) states:
        //   deform(param(x=3,...))  ->  terms at x=2 and x=0
        //   deform(param(x=4,...))  ->  terms at x=1 and x=0
        //   deform(param(x=5,...))  ->  empty
        // with height 4 coefficients. The block elements z 0..5 map
        // 1:1 to KGB x 0..5. The mu columns of the block drive the
        // deformation loop's triangular recursion.
        let (block, _table) = a2_block();
        let mut kl = KlTable::new(&block).unwrap();
        kl.fill(0).unwrap();
        let mu3: Vec<(usize, i32)> = kl.mu_column(3).iter().map(|p| (p.x, p.coef)).collect();
        let mu4: Vec<(usize, i32)> = kl.mu_column(4).iter().map(|p| (p.x, p.coef)).collect();
        let mu5: Vec<(usize, i32)> = kl.mu_column(5).iter().map(|p| (p.x, p.coef)).collect();
        // The frozen deform contract (job 3506415) deformations:
        //   deform(param(x=3,...)) -> terms at x=2 and x=0
        //   deform(param(x=4,...)) -> terms at x=1 and x=0
        // The mu columns must reach exactly those elements.
        assert_eq!(mu3, vec![(0, 1), (2, 1)], "mu3");
        assert_eq!(mu4, vec![(0, 1), (1, 1)], "mu4");
        assert_eq!(mu5, vec![(3, 1), (4, 1)], "mu5");
    }

    #[test]
    fn a2_deformation_terms_reach_the_frozen_deform_outputs() {
        // deform(param(KGB(rf,3),[0,0],[1,1]/1)) -> x=2, x=0 (height 4)
        // deform(param(KGB(rf,4),[0,0],[1,1]/1)) -> x=1, x=0 (height 4)
        // with the same gamma=[1,1]/1 and lambda_rho=[0,0] on every term.
        // Rebuild the quasisplit RepContext (like ktype::with_su21), the
        // dual KGB, the block, and the KL table, then run the simplified
        // deformation_terms for y=3 and y=4.
        let datum = BasedRootDatum::from_simple_data(
            2,
            vec![vec![2, -1], vec![-1, 2]],
            vec![Weight::new(vec![2, -1]), Weight::new(vec![-1, 2])],
            vec![Coweight::new(vec![1, 0]), Coweight::new(vec![0, 1])],
        )
        .unwrap();
        let involution = LatticeInvolution::identity(&datum).unwrap();
        let inner_class = crate::InnerClass::new(datum, involution, 8).unwrap();
        let classification = CartanClassification::build(&inner_class, &class_budget(8)).unwrap();
        let strong = StrongRealClassification::build(&classification, 4_096).unwrap();
        let mut table = InvolutionTable::new(
            &inner_class,
            InvolutionTableBudget::new(64, IntegerLatticeBudget::new(64, 100_000, 100_000, 128)),
        )
        .unwrap();
        let (graph, primal_table) =
            graph_with_size(&inner_class, &classification, &strong, &mut table, 6);
        let rc = crate::RepContext::new(&inner_class, &primal_table, &graph).unwrap();
        let dual_class = crate::dual::dual_inner_class(&inner_class, 8, 64).unwrap();
        let dual_classification =
            CartanClassification::build(&dual_class, &class_budget(8)).unwrap();
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
            4,
        );
        let block = BlockGraph::build(
            &graph,
            &primal_table,
            &dual_graph,
            &dual_table,
            &dual_class,
            8,
        )
        .unwrap();
        assert_eq!(block.size(), 6);
        let mut kl = KlTable::new(&block).unwrap();
        kl.fill(0).unwrap();
        let gamma = crate::RationalWeight::new(vec![1, 1], 1).unwrap();
        let lam_rho = Weight::new(vec![0, 0]);

        for &y in &[3usize, 4usize] {
            let terms = rc
                .deformation_terms(&block, y, &gamma, &lam_rho, &kl)
                .unwrap();
            let rendered: Vec<(usize, i32, u32)> = terms
                .iter()
                .map(|(sr, c)| {
                    let lam = rc.lambda(sr).unwrap();
                    let nu = rc.nu(sr).unwrap();
                    eprintln!(
                        "y={y}: x={} lambda={:?}/{} nu={:?}/{} c={} height={}",
                        sr.x().index(),
                        lam.numerator(),
                        lam.denominator(),
                        nu.numerator(),
                        nu.denominator(),
                        c,
                        sr.height(),
                    );
                    (sr.x().index(), *c, sr.height())
                })
                .collect();
            assert!(!rendered.is_empty(), "y={y} should have terms");
            for &(x, _, height) in &rendered {
                assert_eq!(height, 4, "y={y} term x={x} height");
            }
        }
        // The exact sources: y=3 reaches x=2 and x=0, y=4 reaches x=1
        // and x=0 (the frozen contract). Coefficients are both nonzero.
        let terms3 = rc
            .deformation_terms(&block, 3, &gamma, &lam_rho, &kl)
            .unwrap();
        let mut xs3: Vec<usize> = terms3.iter().map(|(sr, _)| sr.x().index()).collect();
        xs3.sort_unstable();
        assert_eq!(xs3, vec![0, 2], "deform(x=3) sources, terms={terms3:?}");
        let terms4 = rc
            .deformation_terms(&block, 4, &gamma, &lam_rho, &kl)
            .unwrap();
        let mut xs4: Vec<usize> = terms4.iter().map(|(sr, _)| sr.x().index()).collect();
        xs4.sort_unstable();
        assert_eq!(xs4, vec![0, 1], "deform(x=4) sources, terms={terms4:?}");
    }

    /// The B2 `dual_KL_block` oracle anchor (tests/reference/domain/
    /// dual_kl_block.events.json, verified_hpc_reference): the block of the
    /// split form (KGB 11) against the dual class's quasisplit form
    /// (KGB 7) has 12 elements; the KL table of its DUAL block reproduces
    /// the oracle's polynomial pool `[[ ], [1], [0,1], [2]]` and the full
    /// 12x12 index matrix. The pinned values P(1,5) = 2, P(2,5) =
    /// P(3,5) = 1 exercise the RealTypeII arm of `recursion_column`
    /// (kl.cpp:416-425), whose first term is the inverse-Cayley image of
    /// x — taking the cross image there instead zeroes exactly these
    /// polynomials.
    #[test]
    fn b2_dual_block_klv_matches_the_oracle_pool_and_matrix() {
        let datum = BasedRootDatum::from_simple_data(
            2,
            vec![vec![2, -2], vec![-1, 2]],
            vec![Weight::new(vec![2, -2]), Weight::new(vec![-1, 2])],
            vec![Coweight::new(vec![1, 0]), Coweight::new(vec![0, 1])],
        )
        .unwrap();
        let involution = LatticeInvolution::identity(&datum).unwrap();
        let inner_class = crate::InnerClass::new(datum, involution, 8).unwrap();
        let classification = CartanClassification::build(&inner_class, &class_budget(8)).unwrap();
        let strong = StrongRealClassification::build(&classification, 4_096).unwrap();
        let mut table = InvolutionTable::new(
            &inner_class,
            InvolutionTableBudget::new(64, IntegerLatticeBudget::new(64, 100_000, 100_000, 128)),
        )
        .unwrap();
        let (graph, primal_table) =
            graph_with_size(&inner_class, &classification, &strong, &mut table, 11);
        let dual_class = crate::dual::dual_inner_class(&inner_class, 8, 64).unwrap();
        let dual_classification =
            CartanClassification::build(&dual_class, &class_budget(8)).unwrap();
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
            7,
        );
        let block = BlockGraph::build(
            &graph,
            &primal_table,
            &dual_graph,
            &dual_table,
            &dual_class,
            8,
        )
        .unwrap();
        assert_eq!(block.size(), 12, "B2 split x dual quasisplit block");
        let dual_block = block.dual();
        assert_eq!(dual_block.size(), 12);
        let mut kl = KlTable::new(&dual_block).unwrap();
        kl.fill(0).unwrap();

        // The focused anchors the bug report pins on the dual block.
        let pol_at = |x: usize, y: usize| {
            kl.pool
                .get(kl.kl_pol(x, y).unwrap())
                .cloned()
                .unwrap()
                .as_slice()
                .to_vec()
        };
        assert_eq!(pol_at(2, 5), vec![1], "P(2,5)");
        assert_eq!(pol_at(3, 5), vec![1], "P(3,5)");
        assert_eq!(pol_at(1, 5), vec![2], "P(1,5)");

        // The full oracle comparison: deduplicate column-major over all
        // twelve elements (pool seeded with 0 and 1), exactly like the
        // dual_KL_block builtin (atlas-types.w:7053-7133).
        let last = dual_block.size() - 1;
        let mut polys: Vec<Vec<i32>> = vec![vec![], vec![1]];
        let mut index_of: std::collections::HashMap<Vec<i32>, usize> =
            std::collections::HashMap::new();
        index_of.insert(vec![], 0);
        index_of.insert(vec![1], 1);
        let mut index_matrix = vec![vec![0_usize; 12]; 12];
        for j in 0..12 {
            for i in j..12 {
                let coefficients = pol_at(last - i, last - j);
                let index = *index_of.entry(coefficients.clone()).or_insert_with(|| {
                    polys.push(coefficients);
                    polys.len() - 1
                });
                index_matrix[i][j] = index;
            }
        }
        assert_eq!(
            polys,
            vec![vec![], vec![1], vec![0, 1], vec![2]],
            "oracle polynomial pool"
        );
        #[rustfmt::skip]
        let expected: Vec<Vec<usize>> = vec![
            vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            vec![0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            vec![0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            vec![0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0],
            vec![1, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0],
            vec![0, 0, 1, 1, 0, 1, 0, 0, 0, 0, 0, 0],
            vec![1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0],
            vec![1, 0, 1, 0, 1, 1, 1, 1, 0, 0, 0, 0],
            vec![1, 0, 0, 0, 1, 0, 1, 0, 1, 0, 0, 0],
            vec![0, 0, 1, 0, 0, 1, 1, 0, 0, 1, 0, 0],
            vec![1, 0, 1, 0, 1, 1, 3, 1, 1, 1, 1, 0],
            vec![1, 2, 1, 2, 1, 1, 1, 1, 0, 0, 0, 1],
        ];
        assert_eq!(index_matrix, expected, "oracle KL index matrix");
    }
}
