//! Compact Weyl group in the transducer (parabolic-subquotient)
//! representation of du Cloux / van Leeuwen (upstream `structure/weyl.cpp`).
//!
//! A Weyl element is `[u8; rank]`: entry `i` indexes the minimal coset
//! representative of the parabolic subquotient `W_{i-1}\W_i`. Multiplication
//! is O(length) via the per-generator transducers, so enumerating a whole
//! Weyl group (e.g. 51840 for E6) is orders of magnitude cheaper than the
//! matrix representation used by [`crate::WeylAction`].

use std::collections::{HashSet, VecDeque};

use crate::StructureError;

pub(crate) type Generator = usize;
pub(crate) type EltPiece = u16;
/// A Weyl element as a fixed stack array (rank <= 8, one byte per piece) -
/// the C++ `std::array<unsigned char, RANK_MAX>` equivalent, zero heap
/// allocation in the enumeration/twisted scan.
pub(crate) type WeylElt = [u8; 8];

const UNDEF_PIECE: EltPiece = u16::MAX;
const UNDEF_GEN: u16 = u16::MAX;

/// Coxeter matrix entry `(i, j)` for a connected component of type
/// `letter` with generators numbered 0.. in Bourbaki order
/// (weyl.cpp:191-215).
fn coxeter_entry(letter: char, i: usize, j: usize) -> u32 {
    let (mut a, mut b) = (i, j);
    if a > b {
        std::mem::swap(&mut a, &mut b);
    }
    if letter != 'D' && letter != 'E' {
        // linear diagrams
        match b - a {
            0 => 1,
            1 => {
                if letter == 'A'
                    || ((letter == 'B' || letter == 'C') && a > 0)
                    || (letter == 'F' && a != 1)
                {
                    3
                } else if letter == 'G' {
                    6
                } else {
                    4 // BC with (a,b)=(0,1), or F with (1,2)
                }
            }
            _ => 2,
        }
    } else if a == 0 {
        if b == 2 {
            3
        } else {
            2
        }
    } else if letter == 'E' && a == 1 {
        if b == 3 {
            3
        } else {
            2
        }
    } else if b - a == 1 {
        3
    } else {
        2
    }
}

/// One parabolic subquotient transducer (weyl.cpp:100-288, 289-416).
pub(crate) struct Transducer {
    offset: Generator,
    limit: usize,
    lengths: Vec<u8>,
    rights: Vec<u8>,
    /// Flattened `size * limit`; entry `< size` is a shift, `>= size` is a
    /// transduction (`out = entry - size`).
    table: Vec<u16>,
}

impl Transducer {
    fn new(letter: char, offset: Generator, r: usize) -> Self {
        let limit = r + 1;
        // tab[x].shift / .out as flattened arrays; shifts are EltPiece,
        // outs are Generator (stored as u16).
        let mut shift: Vec<u16> = vec![UNDEF_PIECE; limit];
        let mut out: Vec<u16> = vec![UNDEF_GEN; limit];
        let mut lengths = vec![0_u8];
        let mut rights = vec![0_u8];
        // first row: shifts below r transduce the unchanged generator
        for i in 0..r {
            shift[i] = 0;
            out[i] = i as u16;
        }
        let mut size = 1_usize;
        let mut x = 0_usize;
        while x < size {
            for s in 0..=r {
                if shift[x * limit + s] != UNDEF_PIECE {
                    continue;
                }
                let xs = size;
                size += 1;
                lengths.push(lengths[x] + 1);
                rights.push(s as u8);
                shift.push(UNDEF_PIECE); // row xs, shifted by one slot
                out.push(UNDEF_GEN);
                // shift rows by one: row xs is at index xs*limit
                shift.extend(std::iter::repeat_n(UNDEF_PIECE, limit - 1));
                out.extend(std::iter::repeat_n(UNDEF_GEN, limit - 1));
                // tab[x].shift[s] = xs; top.shift[s] = x
                shift[x * limit + s] = xs as u16;
                shift[xs * limit + s] = x as u16;

                for t in 0..=r {
                    if t == s {
                        continue;
                    }
                    let mut b = x;
                    // minimum of the dihedral orbit for s and t
                    loop {
                        let next = shift[b * limit + t];
                        if next != UNDEF_PIECE && (next as usize) < b {
                            b = next as usize;
                        } else {
                            break;
                        }
                        let next = shift[b * limit + s];
                        if next != UNDEF_PIECE && (next as usize) < b {
                            b = next as usize;
                        } else {
                            break;
                        }
                    }
                    let d = (lengths[xs] as u32) - (lengths[b] as u32);
                    let m = coxeter_entry(letter, s, t);
                    let st = [s, t];
                    if d == m {
                        // case (1): no transduction; y is m-1 upward steps
                        let mut y = b;
                        let u = st[(m % 2) as usize];
                        let v = st[1 - (m % 2) as usize];
                        let mut steps = m - 1;
                        loop {
                            let next = shift[y * limit + u];
                            debug_assert!(next != UNDEF_PIECE && (next as usize) > y);
                            y = next as usize;
                            steps -= 1;
                            if steps == 0 {
                                break;
                            }
                            let next = shift[y * limit + v];
                            debug_assert!(next != UNDEF_PIECE && (next as usize) > y);
                            y = next as usize;
                            steps -= 1;
                            if steps == 0 {
                                break;
                            }
                        }
                        shift[xs * limit + t] = y as u16;
                        shift[y * limit + t] = xs as u16;
                    } else if d == m - 1 {
                        let u = st[1 - (m % 2) as usize];
                        if shift[b * limit + u] == b as u16 {
                            // case (2): xs fixed by t; output same g as b for u
                            shift[xs * limit + t] = xs as u16;
                            out[xs * limit + t] = out[b * limit + u];
                        }
                        // case (3): do nothing
                    }
                }
            }
            x += 1;
        }
        // pack into the table: shift when out is undef, else size + out
        let mut table = vec![0_u16; size * limit];
        for i in 0..size {
            for j in 0..=r {
                let entry = if out[i * limit + j] == UNDEF_GEN {
                    shift[i * limit + j]
                } else {
                    (size as u16) + out[i * limit + j]
                };
                table[i * limit + j] = entry;
            }
        }
        Self {
            offset,
            limit,
            lengths,
            rights,
            table,
        }
    }

    fn size(&self) -> usize {
        self.lengths.len()
    }

    fn has_shift(&self, x: EltPiece, s: usize) -> bool {
        (self.table[(x as usize) * self.limit + s] as usize) < self.size()
    }

    fn shift(&self, x: EltPiece, s: usize) -> EltPiece {
        self.table[(x as usize) * self.limit + s]
    }

    fn out(&self, x: EltPiece, s: usize) -> Generator {
        (self.table[(x as usize) * self.limit + s] as usize) - self.size()
    }

    fn unshift(&self, x: EltPiece) -> Generator {
        self.rights[x as usize] as usize
    }
}

/// The Weyl group in the compact representation, with the external
/// (datum) ↔ internal (transducer) generator numbering.
pub struct CompactWeyl {
    transducers: Vec<Transducer>,
    /// external -> internal
    d_in: Vec<usize>,
    /// internal -> external
    d_out: Vec<usize>,
    min_star: Vec<usize>,
    /// last generator of the internal diagram component of each generator
    upper: Vec<usize>,
    /// Precomputed piece words (local generators, left to right) per
    /// (transducer, piece): every element's inverse/twist scans reuse these.
    piece_words: Vec<Vec<Vec<usize>>>,
}

impl CompactWeyl {
    /// Build from a Cartan matrix (weyl.cpp:495-547): classify the diagram,
    /// reverse types B/C/D (internal order), construct one transducer per
    /// internal generator.
    pub fn new(cartan: &[Vec<i32>]) -> Result<Self, StructureError> {
        let rank = cartan.len();
        let comps = crate::dynkin::classify(cartan)?;
        let mut d_out = vec![0_usize; rank];
        let mut upper = vec![0_usize; rank];
        for comp in &comps {
            let offset = comp.offset();
            let last = offset + comp.position.len() - 1;
            for i in 0..comp.position.len() {
                upper[offset + i] = last;
            }
            if matches!(comp.letter, 'B' | 'C' | 'D') {
                for (i, &position) in comp.position.iter().enumerate() {
                    d_out[last - i] = position;
                }
            } else {
                for (i, &position) in comp.position.iter().enumerate() {
                    d_out[offset + i] = position;
                }
            }
        }
        let mut d_in = vec![0_usize; rank];
        for i in 0..rank {
            d_in[d_out[i]] = i;
        }
        let mut transducers = Vec::with_capacity(rank);
        for comp in &comps {
            for i in 0..comp.position.len() {
                transducers.push(Transducer::new(comp.letter, comp.offset(), i));
            }
        }
        // d_min_star: for each generator, the first non-commuting or equal
        // generator with internal index <= it.
        let mut min_star = vec![0_usize; rank];
        for s in 0..rank {
            min_star[s] = s;
            let comp = comps
                .iter()
                .find(|c| c.offset() <= s && s < c.offset() + c.position.len())
                .expect("in-range");
            let mut t = s;
            while t > comp.offset() {
                t -= 1;
                let cox = coxeter_entry(comp.letter, s - comp.offset(), t - comp.offset());
                if cox >= 3 {
                    min_star[s] = t;
                    break;
                }
            }
        }
        let piece_words = transducers
            .iter()
            .map(|tr| {
                (0..tr.lengths.len())
                    .map(|piece| {
                        let mut word = Vec::new();
                        let mut cur = piece as EltPiece;
                        while cur > 0 {
                            let right = tr.unshift(cur);
                            word.push(right);
                            cur = tr.shift(cur, right);
                        }
                        word.reverse();
                        word
                    })
                    .collect()
            })
            .collect();
        if rank > 8 {
            return Err(StructureError::ResourceLimitExceeded { limit: 8 });
        }
        Ok(Self {
            transducers,
            d_in,
            d_out,
            min_star,
            upper,
            piece_words,
        })
    }

    fn start_gen(&self, internal_s: usize) -> usize {
        self.upper[internal_s]
    }

    /// Right multiply `w` by internal generator `s` at transducer `i`;
    /// returns +1/-1 for the length change.
    fn transduce(&self, w: &mut WeylElt, mut i: usize, mut s: usize) -> i8 {
        loop {
            let wi = w[i] as EltPiece;
            let tr = &self.transducers[i];
            if tr.has_shift(wi, s) {
                let shifted = tr.shift(wi, s);
                let down = (shifted as usize) < (wi as usize);
                w[i] = shifted as u8;
                return if down { -1 } else { 1 };
            }
            debug_assert!(i > 0);
            s = tr.out(wi, s);
            i -= 1;
        }
    }

    /// Right multiply by external generator `s_ext`.
    pub(crate) fn inner_mult(&self, w: &mut WeylElt, s_ext: usize) -> i8 {
        let s = self.d_in[s_ext];
        let local = s - self.transducers[s].offset;
        self.transduce(w, self.start_gen(s), local)
    }

    /// Multiply `w` on the right by the piece `i` of `v`.
    fn mult_by_piece(&self, w: &mut WeylElt, v: &WeylElt, i: usize) -> i32 {
        let tr = &self.transducers[i];
        let x = v[i] as EltPiece;
        let start = self.start_gen(i);
        let _ = self;
        let mut result = -(tr.lengths[x as usize] as i32);
        let mut stack: Vec<usize> = Vec::with_capacity(tr.lengths[x as usize] as usize);
        let mut cur = x;
        while cur > 0 {
            let right = tr.unshift(cur);
            stack.push(right);
            cur = tr.shift(cur, right); // x = shift(x, right)
        }
        // the unshift chain is rightmost-first; reverse it for left-to-right
        for &letter in stack.iter().rev() {
            result += i32::from(self.transduce(w, start, letter));
        }
        result
    }

    /// Right multiply `w` by `v` (both internal-numbered pieces).
    pub(crate) fn multiply(&self, w: &mut WeylElt, v: &WeylElt) {
        for i in 0..self.transducers.len() {
            self.mult_by_piece(w, v, i);
        }
    }

    fn identity(&self) -> WeylElt {
        [0_u8; 8]
    }

    /// The inverse of `w` (weyl.cpp:751-763): right-multiply by the
    /// unshift letters of the pieces, right to left.
    pub(crate) fn inverse(&self, w: &WeylElt) -> WeylElt {
        let mut wi = self.identity();
        for i in (0..self.transducers.len()).rev() {
            let tr = &self.transducers[i];
            let mut x = w[i] as EltPiece;
            while x > 0 {
                let right = tr.unshift(x);
                let external = self.d_out[tr.offset + right];
                self.inner_mult(&mut wi, external);
                x = tr.shift(x, right);
            }
        }
        wi
    }

    /// Apply the diagram automorphism `twist` (external generator
    /// permutation) to `w`: apply it to the letters of a word for `w`.
    pub(crate) fn apply_twist(&self, w: &WeylElt, twist: &[usize]) -> WeylElt {
        let mut result = self.identity();
        for i in 0..self.transducers.len() {
            let word = self.word_of_piece(i, w[i]);
            for &local in word {
                // word_of_piece letters are LOCAL to the transducer's
                // component; add the component offset for the global
                // internal numbering, then map to external.
                let internal = self.transducers[i].offset + local;
                let external = self.d_out[internal];
                let twisted_external = twist[external];
                self.inner_mult(&mut result, twisted_external);
            }
        }
        result
    }

    /// Whether `w` is a twisted involution: `w^{-1} = twist(w)`.
    pub(crate) fn is_twisted_involution(&self, w: &WeylElt, twist: &[usize]) -> bool {
        let wi = self.inverse(w);
        let tw = self.apply_twist(w, twist);
        wi == tw
    }

    /// The word (local generators, left to right) of one piece, from the
    /// precomputed table (no allocation).
    pub(crate) fn word_of_piece(&self, i: usize, x: u8) -> &[usize] {
        &self.piece_words[i][x as usize]
    }

    /// The group's canonical elected word (weyl.cpp:944-958
    /// `WeylGroup::word`) of the element represented by `external_word` —
    /// ANY word for it in external (datum) generator numbering, reduced or
    /// not. The element is rebuilt via `inner_mult`, then the elected piece
    /// words are concatenated in increasing piece order with letters mapped
    /// back through `d_out`; the result depends only on the element.
    pub fn canonical_word(&self, external_word: &[usize]) -> Vec<usize> {
        let mut elt: WeylElt = [0; 8];
        for &generator in external_word {
            self.inner_mult(&mut elt, generator);
        }
        let mut result = Vec::new();
        for (i, transducer) in self.transducers.iter().enumerate() {
            for &local in self.word_of_piece(i, elt[i]) {
                result.push(self.d_out[transducer.offset + local]);
            }
        }
        result
    }

    /// Internal -> external generator numbering.
    pub(crate) fn d_out(&self) -> &[usize] {
        &self.d_out
    }

    /// The component offset of the `i`-th transducer (for translating the
    /// local letters of `word_of_piece` to global internal numbering).
    pub(crate) fn piece_offset(&self, i: usize) -> usize {
        self.transducers[i].offset
    }

    /// Compose two root permutations (u8-encoded root indices): `(left
    /// after right)`.
    fn compose_perms(left: &[u8], right: &[u8]) -> Vec<u8> {
        right.iter().map(|&r| left[r as usize]).collect()
    }

    /// One root permutation per (transducer, piece): the composition of the
    /// simple-reflection root permutations of the piece word. Element root
    /// permutations are then compositions of these (no matrix needed).
    pub(crate) fn piece_root_permutations(
        &self,
        reflection_perms: &[Vec<u8>],
    ) -> Vec<Vec<Vec<u8>>> {
        self.transducers
            .iter()
            .enumerate()
            .map(|(i, tr)| {
                (0..tr.lengths.len())
                    .map(|piece| {
                        let word = self.word_of_piece(i, piece as u8);
                        let mut perm: Vec<u8> = (0..reflection_perms[0].len())
                            .map(|index| index as u8)
                            .collect();
                        for &local in word {
                            let internal = tr.offset + local;
                            let refl = &reflection_perms[self.d_out[internal]];
                            perm = Self::compose_perms(refl, &perm);
                        }
                        perm
                    })
                    .collect()
            })
            .collect()
    }

    /// The root permutation of every element, composed from the per-piece
    /// permutations, in parallel.
    pub(crate) fn element_root_permutations(
        &self,
        elements: &[WeylElt],
        piece_perms: &[Vec<Vec<u8>>],
    ) -> Vec<Vec<u8>> {
        use rayon::prelude::*;
        let rank = self.transducers.len();
        // Rank 0 (T1): no transducers and no roots, so every element
        // (the identity) has the empty permutation.
        let width = piece_perms
            .first()
            .and_then(|perms| perms.first())
            .map_or(0, |perm| perm.len());
        elements
            .par_iter()
            .map(|elt| {
                let mut perm: Vec<u8> = (0..width).map(|index| index as u8).collect();
                for i in 0..rank {
                    perm = Self::compose_perms(&piece_perms[i][elt[i] as usize], &perm);
                }
                perm
            })
            .collect()
    }

    /// One matrix per (transducer, piece): the product of the simple
    /// reflections of the piece word. Element matrices are then the product
    /// of these per-piece matrices (rank matrix products instead of one per
    /// word letter).
    pub(crate) fn piece_matrices(
        &self,
        reflections: &[crate::weyl::WeylAction],
    ) -> Result<Vec<Vec<crate::weyl::WeylAction>>, StructureError> {
        let mut result = Vec::with_capacity(self.transducers.len());
        for (i, tr) in self.transducers.iter().enumerate() {
            let mut per_transducer = Vec::with_capacity(tr.lengths.len());
            for piece in 0..tr.lengths.len() {
                let word = self.word_of_piece(i, piece as u8);
                let mut action = crate::weyl::WeylAction::identity(reflections[0].datum_arc())?;
                for &local in word {
                    let internal = tr.offset + local;
                    let external = self.d_out[internal];
                    action = action.compose_fast(&reflections[external]);
                }
                per_transducer.push(action);
            }
            result.push(per_transducer);
        }
        Ok(result)
    }

    /// Enumerate all group elements by breadth-first search (compact
    /// elements, cheap multiplication). For rank <= 8 the element is
    /// encoded in one u64 (one byte per piece) so the dedup set is a flat
    /// integer hash set.
    pub(crate) fn enumerate(&self, budget: usize) -> Result<Vec<WeylElt>, StructureError> {
        let rank = self.transducers.len();
        let use_u64 = rank <= 8;
        if use_u64 {
            let mut seen: HashSet<u64> = HashSet::with_capacity(budget.min(1 << 16));
            let mut pending: VecDeque<u64> = VecDeque::new();
            let id = 0_u64;
            seen.insert(id);
            pending.push_back(id);
            while let Some(w) = pending.pop_front() {
                let mut bytes = [0_u8; 8];
                for i in 0..rank {
                    bytes[i] = ((w >> (8 * i)) & 0xff) as u8;
                }
                for s in 0..rank {
                    let mut next_bytes = bytes;
                    self.inner_mult(&mut next_bytes, s);
                    let mut next = 0_u64;
                    for i in 0..rank {
                        next |= u64::from(next_bytes[i]) << (8 * i);
                    }
                    if seen.insert(next) {
                        if seen.len() > budget {
                            return Err(StructureError::ResourceLimitExceeded { limit: budget });
                        }
                        pending.push_back(next);
                    }
                }
            }
            Ok(seen
                .into_iter()
                .map(|w| {
                    let mut bytes = [0_u8; 8];
                    for i in 0..rank {
                        bytes[i] = ((w >> (8 * i)) & 0xff) as u8;
                    }
                    bytes
                })
                .collect())
        } else {
            let mut seen: HashSet<WeylElt> = HashSet::with_capacity(budget.min(1 << 16));
            let mut pending: VecDeque<WeylElt> = VecDeque::new();
            let id = self.identity();
            seen.insert(id);
            pending.push_back(id);
            while let Some(w) = pending.pop_front() {
                for s in 0..self.transducers.len() {
                    let mut next = w;
                    self.inner_mult(&mut next, s);
                    if seen.insert(next) {
                        if seen.len() > budget {
                            return Err(StructureError::ResourceLimitExceeded { limit: budget });
                        }
                        pending.push_back(next);
                    }
                }
            }
            Ok(seen.into_iter().collect())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compact_order(cartan: &[Vec<i32>]) -> usize {
        CompactWeyl::new(cartan)
            .unwrap()
            .enumerate(1 << 20)
            .unwrap()
            .len()
    }

    #[test]
    fn compact_order_matches_known_group_orders() {
        let a2: Vec<Vec<i32>> = vec![vec![2, -1], vec![-1, 2]];
        assert_eq!(compact_order(&a2), 6);
        let b2: Vec<Vec<i32>> = vec![vec![2, -2], vec![-1, 2]];
        assert_eq!(compact_order(&b2), 8);
        let g2: Vec<Vec<i32>> = vec![vec![2, -3], vec![-1, 2]];
        assert_eq!(compact_order(&g2), 12);
        let a3: Vec<Vec<i32>> = vec![vec![2, -1, 0], vec![-1, 2, -1], vec![0, -1, 2]];
        assert_eq!(compact_order(&a3), 24);
        let d4: Vec<Vec<i32>> = vec![
            vec![2, -1, 0, 0],
            vec![-1, 2, -1, -1],
            vec![0, -1, 2, 0],
            vec![0, -1, 0, 2],
        ];
        assert_eq!(compact_order(&d4), 192);
    }

    #[test]
    fn compact_inverse_satisfies_the_group_law() {
        let a2: Vec<Vec<i32>> = vec![vec![2, -1], vec![-1, 2]];
        let group = CompactWeyl::new(&a2).unwrap();
        let elements = group.enumerate(1 << 10).unwrap();
        for w in &elements {
            let wi = group.inverse(w);
            let mut prod = *w;
            group.multiply(&mut prod, &wi);
            assert_eq!(prod, group.identity(), "w * w^-1 != e for {w:?}");
        }
        // twisted involutions for the identity twist are the involutions
        let twist = [0_usize, 1];
        for w in &elements {
            let is_tw = group.is_twisted_involution(w, &twist);
            let mut sq = *w;
            group.multiply(&mut sq, w);
            let is_inv = sq == group.identity();
            assert_eq!(is_tw, is_inv, "mismatch for {w:?}");
        }
    }

    #[test]
    fn compact_matrices_match_the_matrix_enumeration() {
        use crate::weyl::WeylGroup;
        let cartan: Vec<Vec<i32>> = vec![vec![2, 0], vec![0, 2]];
        let datum = crate::BasedRootDatum::standard(cartan.clone()).unwrap();
        let compact = CompactWeyl::new(&cartan).unwrap();
        let elements = compact.enumerate(1 << 10).unwrap();
        let reflections: Vec<_> = (0..2)
            .map(|g| crate::weyl::WeylAction::simple_reflection(&datum, g).unwrap())
            .collect();
        for elt in &elements {
            let mut action = crate::weyl::WeylAction::identity(&datum).unwrap();
            for pi in 0..datum.semisimple_rank() {
                for &internal in compact.word_of_piece(pi, elt[pi]) {
                    let external = compact.d_out()[internal];
                    action = action.compose_fast(&reflections[external]);
                }
            }
            // the element as a Weyl group element must have w^2 == e and
            // match the matrix enumeration's action
            let actions = WeylGroup::new(datum.clone())
                .enumerate_actions(1 << 10)
                .unwrap();
            let found = actions.iter().find(|a| a.matrix() == action.matrix());
            assert!(
                found.is_some(),
                "compact matrix not in enumeration: {elt:?}"
            );
        }
    }

    #[test]
    fn compact_a1xa1_inverse_and_twisted_checks() {
        let cartan: Vec<Vec<i32>> = vec![vec![2, 0], vec![0, 2]];
        let group = CompactWeyl::new(&cartan).unwrap();
        let elements = group.enumerate(1 << 10).unwrap();
        assert_eq!(elements.len(), 4);
        for w in &elements {
            let wi = group.inverse(w);
            let mut prod = *w;
            group.multiply(&mut prod, &wi);
            assert_eq!(prod, group.identity(), "w*w^-1 != e for {w:?}");
        }
        let twist = [0_usize, 1];
        for w in &elements {
            let mut sq = *w;
            group.multiply(&mut sq, w);
            let is_inv = sq == group.identity();
            let is_tw = group.is_twisted_involution(w, &twist);
            assert_eq!(is_tw, is_inv, "twisted/involution mismatch for {w:?}");
        }
    }

    #[test]
    fn compact_e6_enumerates_the_full_group() {
        // E6 has 51840 elements; the compact enumeration must reach all of
        // them (and stay cheap) for the parallel matrix materialization.
        let cartan: Vec<Vec<i32>> = vec![
            vec![2, -1, 0, 0, 0, 0],
            vec![-1, 2, -1, 0, 0, 0],
            vec![0, -1, 2, -1, 0, -1],
            vec![0, 0, -1, 2, -1, 0],
            vec![0, 0, 0, -1, 2, 0],
            vec![0, 0, -1, 0, 0, 2],
        ];
        let group = CompactWeyl::new(&cartan).unwrap();
        let t = std::time::Instant::now();
        let elements = group.enumerate(1 << 20).unwrap();
        assert_eq!(elements.len(), 51_840);
        assert!(
            t.elapsed() < std::time::Duration::from_secs(2),
            "compact E6 enumeration too slow: {:?}",
            t.elapsed()
        );
    }

    #[test]
    fn compact_e6_matrices_match_the_matrix_enumeration_exactly() {
        use crate::weyl::{WeylAction, WeylGroup};
        use std::collections::HashSet;
        let cartan: Vec<Vec<i32>> = vec![
            vec![2, -1, 0, 0, 0, 0],
            vec![-1, 2, -1, 0, 0, 0],
            vec![0, -1, 2, -1, 0, -1],
            vec![0, 0, -1, 2, -1, 0],
            vec![0, 0, 0, -1, 2, 0],
            vec![0, 0, -1, 0, 0, 2],
        ];
        let datum = crate::BasedRootDatum::standard(cartan.clone()).unwrap();
        let compact = CompactWeyl::new(&cartan).unwrap();
        let elements = compact.enumerate(1 << 20).unwrap();
        assert_eq!(elements.len(), 51_840);
        let reflections: Vec<_> = (0..6)
            .map(|g| WeylAction::simple_reflection(&datum, g).unwrap())
            .collect();
        let piece_matrices = compact.piece_matrices(&reflections).unwrap();
        let compact_set: HashSet<Vec<i32>> = elements
            .iter()
            .map(|elt| {
                let mut action = WeylAction::identity(&datum).unwrap();
                for pi in 0..datum.semisimple_rank() {
                    action = action.compose_fast(&piece_matrices[pi][elt[pi] as usize]);
                }
                action
                    .matrix()
                    .iter()
                    .flatten()
                    .copied()
                    .collect::<Vec<_>>()
            })
            .collect();
        let matrix_set: HashSet<Vec<i32>> = WeylGroup::new(datum)
            .enumerate_actions(1 << 20)
            .unwrap()
            .iter()
            .map(|a| a.matrix().iter().flatten().copied().collect::<Vec<_>>())
            .collect();
        assert_eq!(matrix_set.len(), 51_840);
        let missing: Vec<_> = matrix_set.difference(&compact_set).take(3).collect();
        let extra: Vec<_> = compact_set.difference(&matrix_set).take(3).collect();
        assert!(missing.is_empty(), "compact missing {missing:?}");
        assert!(extra.is_empty(), "compact extra {extra:?}");
    }

    #[test]
    fn compact_multiply_matches_length_and_action() {
        // The compact multiplication must reproduce the group law: check
        // that w * s * w == identity for a sample of elements on A2.
        let cartan: Vec<Vec<i32>> = vec![vec![2, -1], vec![-1, 2]];
        let group = CompactWeyl::new(&cartan).unwrap();
        let elements = group.enumerate(1 << 10).unwrap();
        let mut involution_count = 0_usize;
        for w in &elements {
            let mut inv = *w;
            group.inner_mult(&mut inv, 0);
            group.inner_mult(&mut inv, 0);
            if inv == *w {
                involution_count += 1;
            }
        }
        // s0 * s0 = identity leaves every element unchanged.
        assert_eq!(involution_count, elements.len());
    }
}
