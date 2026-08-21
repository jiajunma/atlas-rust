//! Binary Kazhdan-Lusztig file formats (io/filekl.w).
//!
//! Upstream Atlas exchanges precomputed KL data with utility programs
//! through four binary file formats (filekl.w:21-48): the *block file*
//! (block size, rank, length stops, descent sets, and the ascent table used
//! for primitivisation), the *matrix file* (per-row bitmaps of nonzero KL
//! polynomials plus their pool indices), the *polynomial store* (the
//! deduplicated pool of KL polynomials), and the *progress file* (per-row
//! counts of newly seen polynomials).
//!
//! All multi-byte values are little-endian, following `basic_io`
//! (basic_io.cpp:267-279): `put_int` writes a 4-byte LE u32, and
//! `write_bytes(n, v)`/`read_bytes(n)` transfer the `n` least significant
//! bytes first for `n` in 1..=8. Streams are dense, without padding or
//! tags. Unlike the upstream readers, which silently read EOF bytes as
//! garbage, every reader here validates the input and reports truncation
//! or malformed data as an `io::Error` with `ErrorKind::InvalidData`.

use std::collections::HashMap;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::rc::Rc;

use crate::block::BlockDescent;
use crate::kl_polynomial::KlHashTable;
use crate::kl_table::KlTableHandle;
use crate::BlockTopology;

/// Upstream `UndefBlock` (filekl.w:147-157): the out-of-range `BlockElt`
/// written for a real nonparity ascent, which has no good-ascent image.
pub const UNDEF_BLOCK: u32 = 0xFFFF_FFFF;

/// Upstream `no_good_ascent` (filekl.w:155): flags an ascent-table slot
/// whose generator is a descent or an imaginary type II ascent — in both
/// cases no good ascent exists.
pub const NO_GOOD_ASCENT: u32 = 0xFFFF_FFFE;

/// Upstream `magic_code` (filekl.w:157): written over the first 4 bytes of
/// a new-format matrix file, in memory of Fokko du Cloux.
pub const MAGIC_CODE: u32 = 0x06AB_DCF0;

/// `basic_io::put_int` (basic_io.cpp:267): 4 bytes, least significant first.
fn put_int<W: Write + ?Sized>(value: u32, out: &mut W) -> io::Result<()> {
    out.write_all(&value.to_le_bytes())
}

/// `basic_io::write_bytes` (basic_io.cpp:268-279): the `n` least
/// significant bytes of `value`, least significant first, for `n` in 1..=8.
fn write_bytes<W: Write + ?Sized>(n: usize, value: u64, out: &mut W) -> io::Result<()> {
    debug_assert!((1..=8).contains(&n));
    out.write_all(&value.to_le_bytes()[..n])
}

/// `basic_io::read_bytes` (basic_io.cpp:250-265): `n` bytes into an
/// unsigned value, least significant first, for `n` in 1..=8.
fn read_bytes<R: Read + ?Sized>(n: usize, input: &mut R) -> io::Result<u64> {
    debug_assert!((1..=8).contains(&n));
    let mut buf = [0u8; 8];
    input.read_exact(&mut buf[..n])?;
    Ok(u64::from_le_bytes(buf))
}

fn read_u32<R: Read + ?Sized>(input: &mut R) -> io::Result<u32> {
    Ok(u32::try_from(read_bytes(4, input)?).expect("4 bytes fit u32"))
}

fn invalid_data(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

fn u32_exact(value: usize, what: &'static str) -> io::Result<u32> {
    u32::try_from(value).map_err(|_| invalid_data(format!("{what} does not fit in 32 bits")))
}

/// `write_block_file` (filekl.w:176-229): block size (4 bytes), rank (1
/// byte), maximal length (1 byte), then for each `l` in `0..max_length` the
/// number of elements of length `<= l` (4 bytes each), then the descent set
/// of every element as a 32-bit bitmap of weak descents, then the ascent
/// table, `x`-major with the simple generator `s` innermost:
/// `no_good_ascent` for descents (imaginary compact included) and
/// imaginary type II ascents, `UndefBlock` for real nonparity, the cross
/// image for complex ascents, and the first Cayley image for imaginary
/// type I ascents (filekl.w:216-227).
///
/// The exact byte count is `6 + 4*max_length + 4*size + 4*size*rank`
/// (filekl.w:162-167).
pub fn write_block_file<B: BlockTopology + ?Sized, W: Write + ?Sized>(
    block: &B,
    out: &mut W,
) -> io::Result<()> {
    let size = block.size();
    if size == 0 {
        return Err(invalid_data("cannot write an empty block"));
    }
    let rank = block.rank();
    put_int(u32_exact(size, "block size")?, out)?;
    out.write_all(&[u8::try_from(rank).map_err(|_| invalid_data("rank exceeds 255"))?])?;

    // Length data (filekl.w:183-197).
    let length_of = |z: usize| -> io::Result<usize> {
        block
            .length(z)
            .ok_or_else(|| invalid_data("block element has no length"))
    };
    let max_length = length_of(size - 1)?;
    out.write_all(
        &[u8::try_from(max_length).map_err(|_| invalid_data("max length exceeds 255"))?],
    )?;
    let mut z = 0;
    for l in 0..max_length {
        while length_of(z)? <= l {
            z += 1;
        }
        put_int(u32_exact(z, "length stop")?, out)?;
        // record that there are `z` elements of length <= l
    }

    // Descent sets (filekl.w:199-206): bit s set iff s is a weak descent.
    for y in 0..size {
        let mut descents = 0u32;
        for s in 0..rank {
            let value = block
                .descent(y, s)
                .ok_or_else(|| invalid_data("block element has no descent status"))?;
            if value.is_descent() {
                descents |= 1 << s;
            }
        }
        put_int(descents, out)?;
    }

    // Table of primitivisation successors (filekl.w:208-228).
    for x in 0..size {
        for s in 0..rank {
            let value = block
                .descent(x, s)
                .ok_or_else(|| invalid_data("block element has no descent status"))?;
            let entry = match value {
                BlockDescent::ComplexAscent => u32_exact(
                    block
                        .cross(x, s)
                        .ok_or_else(|| invalid_data("complex ascent has no cross image"))?,
                    "cross image",
                )?,
                BlockDescent::RealNonparity => UNDEF_BLOCK,
                BlockDescent::ImaginaryTypeI => u32_exact(
                    block
                        .cayley(x, s)
                        .ok_or_else(|| invalid_data("imaginary type I has no Cayley cell"))?
                        .0
                        .ok_or_else(|| {
                            invalid_data("imaginary type I has no first Cayley image")
                        })?,
                    "Cayley image",
                )?,
                // Every descent (imaginary compact included) and every
                // imaginary type II ascent has no good ascent
                // (filekl.w:217-219).
                _ => NO_GOOD_ASCENT,
            };
            put_int(entry, out)?;
        }
    }
    Ok(())
}

/// The reader half of the block format: `block_info` (filekl.w:245-266,
/// constructor at 321-354). Fields mirror the upstream struct; the ascent
/// table is stored row-major (`x` outer, generator `s` inner) as a flat
/// vector, and `start_length` is reconstructed to `max_length + 2` entries
/// with `start_length[0] == 0` and `start_length[max_length + 1] == size`
/// (filekl.w:330-334).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlockInfo {
    /// Number of block elements (`N`).
    pub size: u32,
    /// Number of simple generators; at most 32, as descent sets are u32.
    pub rank: u32,
    /// Maximal length of a block element.
    pub max_length: u32,
    /// Length stops: `start_length[l]` is the first element of length
    /// `>= l`; has `max_length + 2` entries.
    pub start_length: Vec<u32>,
    /// Weak-descent bitmap per block element (filekl.w:253).
    pub descent_sets: Vec<u32>,
    /// Ascent table, row-major by block element then generator
    /// (filekl.w:256): entries are cross/Cayley images, `NO_GOOD_ASCENT`,
    /// or `UNDEF_BLOCK`.
    pub ascent_table: Vec<u32>,
}

impl BlockInfo {
    /// Read a block file written by [`write_block_file`]
    /// (filekl.w:321-354), validating sizes and monotonicity of the length
    /// stops instead of upstream's silent EOF-tolerant reads.
    pub fn read<R: Read + ?Sized>(input: &mut R) -> io::Result<Self> {
        let size = read_u32(input)?;
        let rank = u32::try_from(read_bytes(1, input)?).expect("1 byte fits u32");
        if rank > 32 {
            return Err(invalid_data("block rank exceeds the 32-bit descent sets"));
        }
        let max_length = u32::try_from(read_bytes(1, input)?).expect("1 byte fits u32");

        // Length intervals (filekl.w:330-334).
        let mut start_length = Vec::with_capacity(max_length as usize + 2);
        start_length.push(0);
        for _ in 0..max_length {
            let stop = read_u32(input)?;
            if stop > size || stop < *start_length.last().expect("start_length nonempty") {
                return Err(invalid_data("block length stops are not nondecreasing"));
            }
            start_length.push(stop);
        }
        start_length.push(size);

        let mut descent_sets = Vec::new();
        descent_sets
            .try_reserve(size as usize)
            .map_err(|_| invalid_data("descent-set allocation failed"))?;
        for _ in 0..size {
            descent_sets.push(read_u32(input)?);
        }

        let entries = size as usize * rank as usize;
        let mut ascent_table = Vec::new();
        ascent_table
            .try_reserve(entries)
            .map_err(|_| invalid_data("ascent-table allocation failed"))?;
        for _ in 0..entries {
            ascent_table.push(read_u32(input)?);
        }

        Ok(Self {
            size,
            rank,
            max_length,
            start_length,
            descent_sets,
            ascent_table,
        })
    }

    /// The ascent-table entry for `(x, s)` (filekl.w:238-239).
    pub fn ascent(&self, x: u32, generator: u32) -> Option<u32> {
        let index = x as usize * self.rank as usize + generator as usize;
        self.ascent_table.get(index).copied()
    }

    /// `block_info::primitivize` (filekl.w:273-284): raise `x` through good
    /// ascents that are descents of `y` until none applies or `x >= y`;
    /// `UNDEF_BLOCK` and other large values return immediately.
    pub fn primitivize(&self, x: u32, y: u32) -> u32 {
        let descents = self.descent_sets[y as usize];
        let mut x = x;
        loop {
            if x >= y {
                return x;
            }
            let mut raised = false;
            for s in 0..self.rank {
                if descents & (1 << s) != 0 {
                    let ascent = self.ascent(x, s).expect("in-range ascent slot");
                    if ascent != NO_GOOD_ASCENT {
                        x = ascent; // this should raise x, now try another step
                        raised = true;
                        break;
                    }
                }
            }
            if !raised {
                return x;
            }
        }
    }

    /// `block_info::is_primitive` (filekl.w:289-298): no good ascent of `x`
    /// is a descent recorded in `descents`.
    fn is_primitive(&self, x: u32, descents: u32) -> bool {
        for s in 0..self.rank {
            if descents & (1 << s) != 0
                && self.ascent(x, s).expect("in-range ascent slot") != NO_GOOD_ASCENT
            {
                return false;
            }
        }
        true
    }

    /// The increasing list of weakly primitive elements for a descent-set
    /// bitmap (`prims_for_descents_of`, filekl.w:303-314, without the lazy
    /// cache — [`MatrixInfo`] keeps the cache).
    pub fn prims_for_descent_set(&self, descents: u32) -> Vec<u32> {
        (0..self.size)
            .filter(|&x| self.is_primitive(x, descents))
            .collect()
    }
}

/// Write one matrix row (`write_KL_row`, filekl.w:365-399) and return the
/// byte offset of its `n_prim` field (`start_row`).
///
/// The canonical row layout is: row number, `n_prim = kld.size() + 1` (the
/// number of weak primitives of `y` of length `< l(y)`, plus one slot for
/// `y` itself), the nonzero-polynomial bitmap over `n_prim` bits with the
/// final `y` bit always set (`P_{y,y} = 1`), then the pool indices of the
/// nonzero polynomials in increasing primitive order, then a literal `1`
/// for the unrecorded diagonal polynomial. Upstream master currently emits
/// `n_prim` one short due to a `prim_map` regression; historical files and
/// all readers use the canonical form written here, so the regression is
/// not replicated.
fn write_kl_row<B: BlockTopology, W: Write + Seek + ?Sized>(
    kl_table: &KlTableHandle<B>,
    y: usize,
    out: &mut W,
) -> io::Result<u64> {
    let support = kl_table.support();
    let desc_y = support.descent_set(y);
    let floor = support.length_floor(y);
    // The dense d_KL row (kl.cpp:369 `KL_data(y)`): pool indices of
    // P_{x,y} for the weak primitives x of y of length < l(y), in
    // increasing primitive order.
    let mut kld: Vec<u32> = Vec::new();
    for x in 0..floor {
        if support.is_primitive(x, desc_y) {
            let index = kl_table.kl_pol(x, y).map_err(io::Error::other)?;
            kld.push(u32_exact(index, "polynomial index")?);
        }
    }
    let n_prim = kld.len() + 1;

    // Row number for consistency check on reading (filekl.w:374).
    put_int(u32_exact(y, "row number")?, out)?;
    let start_row = out.stream_position()?;

    put_int(u32_exact(n_prim, "primitive count")?, out)?;

    // The bitmap as a sequence of 32-bit values (filekl.w:381-383).
    for word_start in (0..n_prim).step_by(32) {
        let mut word = 0u32;
        let width = (n_prim - word_start).min(32);
        for j in 0..width {
            let position = word_start + j;
            let nonzero = if position == n_prim - 1 {
                true // always P_{y,y} = 1 (filekl.w: prim_map's final insert)
            } else {
                kld[position] != 0
            };
            if nonzero {
                word |= 1 << j;
            }
        }
        put_int(word, out)?;
    }

    // The indices of the nonzero KL polynomials (filekl.w:385-391).
    for &index in kld.iter().filter(|&&index| index != 0) {
        put_int(index, out)?;
    }

    put_int(1, out)?; // unrecorded final polynomial 1 (filekl.w:393)
    Ok(start_row)
}

/// `write_matrix_file` (filekl.w:408-426): all KL rows for the block, then
/// a trailer of `N` deltas where `delta[y] = (start_row(y) -
/// start_row(y-1)) / 4` with `start_row(-1) = 0` (so `delta[0] == 1`), and
/// finally the magic code overwriting the first 4 bytes to mark the new
/// format.
pub fn write_matrix_file<B: BlockTopology, W: Write + Seek>(
    kl_table: &KlTableHandle<B>,
    out: &mut W,
) -> io::Result<()> {
    let size = kl_table.support().size();
    let mut delta = Vec::with_capacity(size);
    let mut offset = 0u64;
    for y in 0..size {
        let new_offset = write_kl_row(kl_table, y, out)?;
        let step = (new_offset - offset) / 4;
        delta.push(
            u32::try_from(step).map_err(|_| invalid_data("row delta does not fit in 32 bits"))?,
        );
        offset = new_offset;
    }

    // The values allowing rapid location of the matrix rows (filekl.w:419-421).
    for value in delta {
        put_int(value, out)?;
    }

    // Sign the file as being in the new format (filekl.w:423-425).
    out.seek(SeekFrom::Start(0))?;
    put_int(MAGIC_CODE, out)?;
    Ok(())
}

/// The reader half of the matrix format: `matrix_info` (filekl.w:437-479).
/// Owns the decoded block information and the matrix stream; rows are
/// located through the delta trailer in new-format files (detected by
/// [`MAGIC_CODE`] at offset 0) or by a verifying linear scan for old-format
/// files (filekl.w:582-639).
#[derive(Debug)]
pub struct MatrixInfo<R> {
    block: BlockInfo,
    matrix: R,
    /// Byte offset of each row's `n_prim` field (filekl.w:443).
    row_pos: Vec<u64>,
    /// Lazy weak-primitive lists per descent-set bitmap (filekl.w:257).
    prim_cache: HashMap<u32, Rc<Vec<u32>>>,
    // Data for the currently selected row `y` (filekl.w:445-448).
    cur_y: Option<u32>,
    cur_strong_prims: Vec<u32>,
    cur_row_entries: u64,
    /// Set by [`MatrixInfo::find_pol_nr`] (filekl.w:455 `x_prim`).
    x_prim: u32,
}

impl<R: Read + Seek> MatrixInfo<R> {
    /// `matrix_info::matrix_info` (filekl.w:582-639): detect the format by
    /// the leading magic code, then build the row-position table from the
    /// delta trailer (new format) or a full scan that verifies row numbers
    /// (old format; misaligned rows report "Alignment problem", truncated
    /// ones "Premature end of file").
    pub fn open(block: BlockInfo, mut matrix: R) -> io::Result<Self> {
        let size = block.size as usize;
        let file_len = matrix.seek(SeekFrom::End(0))?;
        let mut row_pos = vec![0u64; size];

        matrix.seek(SeekFrom::Start(0))?;
        let first = read_u32(&mut matrix)
            .map_err(|_| invalid_data("matrix file is shorter than one row number"))?;
        if first == MAGIC_CODE {
            // New format (filekl.w:590-597): the last 4*N bytes are deltas.
            let trailer = 4u64
                .checked_mul(size as u64)
                .ok_or_else(|| invalid_data("block size overflow"))?;
            if file_len < trailer {
                return Err(invalid_data("matrix file too short for its delta trailer"));
            }
            matrix.seek(SeekFrom::Start(file_len - trailer))?;
            let mut cumul = 0u64;
            for slot in row_pos.iter_mut() {
                cumul += 4 * u64::from(read_u32(&mut matrix)?);
                *slot = cumul;
            }
        } else {
            // Old format (filekl.w:599-636): scan the whole file.
            matrix.seek(SeekFrom::Start(0))?;
            let premature = |error: io::Error| {
                if error.kind() == io::ErrorKind::UnexpectedEof {
                    invalid_data("Premature end of file")
                } else {
                    error
                }
            };
            for y in 0..size {
                let row_number = read_u32(&mut matrix).map_err(premature)?;
                if row_number != y as u32 && y != 0 {
                    return Err(invalid_data(format!("Alignment problem at row {y}")));
                }
                row_pos[y] = matrix.stream_position().map_err(premature)?;
                let n_prim = read_u32(&mut matrix).map_err(premature)? as usize;

                // Count the strong primitives while reading the bitmap
                // (filekl.w:614-625). Historical files were written on
                // LP64 hosts, so the bulk bitmap unit is the 8-byte
                // `unsigned long`.
                let mut n_strong_prim = 0u64;
                for _ in 0..n_prim / 64 {
                    n_strong_prim +=
                        u64::from(read_bytes(8, &mut matrix).map_err(premature)?.count_ones());
                }
                for _ in 0..(n_prim % 64).div_ceil(32) {
                    n_strong_prim +=
                        u64::from(read_u32(&mut matrix).map_err(premature)?.count_ones());
                }

                // Skip over the matrix entries (filekl.w:627-629).
                let skip = 4i64
                    .checked_mul(
                        i64::try_from(n_strong_prim)
                            .map_err(|_| invalid_data("strong-primitive count overflow"))?,
                    )
                    .ok_or_else(|| invalid_data("strong-primitive count overflow"))?;
                let position = matrix.seek(SeekFrom::Current(skip)).map_err(premature)?;
                if position > file_len {
                    return Err(invalid_data("Premature end of file"));
                }
            }
        }

        Ok(Self {
            block,
            matrix,
            row_pos,
            prim_cache: HashMap::new(),
            cur_y: None,
            cur_strong_prims: Vec::new(),
            cur_row_entries: 0,
            x_prim: 0,
        })
    }

    pub fn block_info(&self) -> &BlockInfo {
        &self.block
    }

    pub fn rank(&self) -> u32 {
        self.block.rank
    }

    pub fn block_size(&self) -> u32 {
        self.block.size
    }

    /// `matrix_info::length` (filekl.w:486-491): the length of `y` in the
    /// block, from the length-stop table.
    pub fn length(&self, y: u32) -> Option<usize> {
        if y >= self.block.size {
            return None;
        }
        Some(self.block.start_length.partition_point(|&stop| stop <= y) - 1)
    }

    pub fn descent_set(&self, y: u32) -> Option<u32> {
        self.block.descent_sets.get(y as usize).copied()
    }

    /// The byte offset of row `y`'s `n_prim` field (filekl.w:469).
    pub fn row_offset(&self, y: u32) -> Option<u64> {
        self.row_pos.get(y as usize).copied()
    }

    pub fn primitivize(&self, x: u32, y: u32) -> u32 {
        self.block.primitivize(x, y)
    }

    /// The `x_prim` value set by the last [`MatrixInfo::find_pol_nr`] call
    /// (filekl.w:455).
    pub fn x_prim(&self) -> u32 {
        self.x_prim
    }

    /// The cached weak-primitive list for `y`'s descent set
    /// (`prims_for_descents_of` with its lazy cache, filekl.w:303-314).
    fn weak_prims(&mut self, y: u32) -> Rc<Vec<u32>> {
        let descents = self.block.descent_sets[y as usize];
        if let Some(prims) = self.prim_cache.get(&descents) {
            return Rc::clone(prims);
        }
        let prims = Rc::new(self.block.prims_for_descent_set(descents));
        self.prim_cache.insert(descents, Rc::clone(&prims));
        prims
    }

    /// `matrix_info::set_y` (filekl.w:496-538): decode row `y`'s bitmap
    /// into the strongly primitive list, replacing the final entry (the
    /// always-set diagonal slot) by `y` itself, and leave the stream at the
    /// row's polynomial entries.
    fn set_y(&mut self, y: u32) -> io::Result<()> {
        if y >= self.block.size {
            return Err(invalid_data(format!("matrix row {y} is outside the block")));
        }
        if self.cur_y == Some(y) {
            self.matrix.seek(SeekFrom::Start(self.cur_row_entries))?;
            return Ok(());
        }
        let weak_prims = self.weak_prims(y);
        let mut strong_prims = Vec::new();

        self.matrix
            .seek(SeekFrom::Start(self.row_pos[y as usize]))?;
        let n_prim = read_u32(&mut self.matrix)? as usize;
        let mut i = 0;
        while i < n_prim {
            let mut chunk = read_u32(&mut self.matrix)?;
            let mut j = 0;
            while chunk != 0 {
                if chunk & 1 != 0 {
                    let &prim = weak_prims.get(i + j).ok_or_else(|| {
                        invalid_data("bitmap bit set outside the weak primitive list")
                    })?;
                    strong_prims.push(prim);
                }
                chunk >>= 1;
                j += 1;
            }
            i += 32;
        }

        // Replace the diagonal slot by y itself (filekl.w:534).
        match strong_prims.last_mut() {
            Some(last) => *last = y,
            None => return Err(invalid_data("matrix row has an empty bitmap")),
        }

        self.cur_y = Some(y);
        self.cur_strong_prims = strong_prims;
        self.cur_row_entries = self.matrix.stream_position()?;
        Ok(())
    }

    /// The strongly primitive elements of row `y` (filekl.w:477-479).
    pub fn strongly_primitives(&mut self, y: u32) -> io::Result<&[u32]> {
        self.set_y(y)?;
        Ok(&self.cur_strong_prims)
    }

    /// `matrix_info::find_pol_nr` (filekl.w:543-556): primitivise `x` for
    /// the descent set of `y`; when the primitivisation reaches or passes
    /// `y` the answer is `1` for `x == y` and `0` otherwise, when it is not
    /// strongly primitive the answer is `0`, else the pool index stored at
    /// the primitive's position among the row's entries.
    pub fn find_pol_nr(&mut self, x: u32, y: u32) -> io::Result<u64> {
        self.set_y(y)?;
        self.x_prim = self.block.primitivize(x, y);
        if self.x_prim >= y {
            // Primitivisation copped out (filekl.w:547-548).
            return Ok(u64::from(self.x_prim == y));
        }
        let position = match self.cur_strong_prims.binary_search(&self.x_prim) {
            Ok(position) => position,
            Err(_) => return Ok(0), // not strong (filekl.w:551-552)
        };
        self.matrix
            .seek(SeekFrom::Start(self.cur_row_entries + 4 * position as u64))?;
        Ok(u64::from(read_u32(&mut self.matrix)?))
    }
}

/// `write_KL_store` (filekl.w:661-692): the number `N` of polynomials (4
/// bytes; index 0 is the zero polynomial, index 1 the constant one), then
/// `N + 1` five-byte little-endian indices where `index[i]` is the byte
/// offset of polynomial `i`'s constant coefficient in the coefficient area
/// (`index[0] == index[1] == 0` because the zero polynomial is empty) and
/// `index[N]` is the total coefficient byte count, then all coefficients,
/// constant term first, 4 bytes each. The degree of polynomial `i` is
/// `(index[i+1] - index[i]) / 4 - 1` (filekl.w:648-657).
pub fn write_kl_store<W: Write + ?Sized>(store: &KlHashTable, out: &mut W) -> io::Result<()> {
    const COEF_SIZE: u64 = 4; // dictated by basic_io::put_int (filekl.w:663)
    let n_pols = store.len();
    put_int(u32_exact(n_pols, "polynomial count")?, out)?;

    // The 5-byte indices, computed on the fly (filekl.w:667-682).
    let mut offset = 0u64;
    for i in 0..n_pols {
        let polynomial = store
            .get(i)
            .ok_or_else(|| invalid_data("polynomial store index gap"))?;
        put_int((offset & 0xFFFF_FFFF) as u32, out)?;
        write_bytes(1, offset >> 32, out)?;
        if !polynomial.is_zero() {
            offset += (polynomial.degree() as u64 + 1) * COEF_SIZE;
        }
    }
    put_int((offset & 0xFFFF_FFFF) as u32, out)?;
    write_bytes(1, offset >> 32, out)?;

    // The coefficients (filekl.w:684-691).
    for i in 0..n_pols {
        let polynomial = store
            .get(i)
            .ok_or_else(|| invalid_data("polynomial store index gap"))?;
        if !polynomial.is_zero() {
            for &coefficient in polynomial.as_slice() {
                put_int(coefficient as u32, out)?;
            }
        }
    }
    Ok(())
}

/// The reader half of the polynomial store: `polynomial_info`
/// (filekl.w:704-724, constructor at 730-738), with the eager degree
/// validation of `cached_pol_info` (filekl.w:808-827): any stored degree
/// of 32 or more is rejected at read time.
#[derive(Clone, Debug)]
pub struct PolynomialInfo {
    n_pols: u64,
    /// `n_pols + 1` byte offsets into the coefficient area.
    index: Vec<u64>,
    /// All coefficients, constant term first per polynomial.
    coefficients: Vec<u32>,
}

impl PolynomialInfo {
    /// Read a polynomial store written by [`write_kl_store`]. The degree
    /// bound matches `cached_pol_info`'s `degree_mask` (filekl.w:795,
    /// 819-822): "Degree found too large (>=32)".
    pub fn read<R: Read + ?Sized>(input: &mut R) -> io::Result<Self> {
        let n_pols = u64::from(read_u32(input)?);
        let mut index = Vec::new();
        index
            .try_reserve(n_pols as usize + 1)
            .map_err(|_| invalid_data("polynomial index allocation failed"))?;
        for _ in 0..=n_pols {
            index.push(read_bytes(5, input)?);
        }
        for i in 0..n_pols as usize {
            let span = index[i + 1]
                .checked_sub(index[i])
                .ok_or_else(|| invalid_data("polynomial indices are not nondecreasing"))?;
            if span % 4 != 0 {
                return Err(invalid_data("polynomial index is not coefficient-aligned"));
            }
            let length = span / 4;
            if i >= 2 {
                if length == 0 {
                    return Err(invalid_data(
                        "nonzero polynomial stored without coefficients",
                    ));
                }
                if length > 32 {
                    return Err(invalid_data("Degree found too large (>=32)"));
                }
            }
        }
        let n_coef = index[n_pols as usize] / 4;
        let mut coefficients = Vec::new();
        coefficients
            .try_reserve(n_coef as usize)
            .map_err(|_| invalid_data("coefficient allocation failed"))?;
        for _ in 0..n_coef {
            coefficients.push(read_u32(input)?);
        }
        Ok(Self {
            n_pols,
            index,
            coefficients,
        })
    }

    pub fn n_polynomials(&self) -> u64 {
        self.n_pols
    }

    /// The number of stored coefficients (`n_coef`, filekl.w:710).
    pub fn n_coefficients(&self) -> u64 {
        self.index[self.n_pols as usize] / 4
    }

    /// `polynomial_info::degree` (filekl.w:745-753): the degree of
    /// polynomial `i`; `None` for the zero polynomial (index 0, which
    /// upstream lets wrap around), `Some(0)` for the constant one (index
    /// 1), and `None` when `i` is out of range.
    pub fn degree(&self, i: u64) -> Option<usize> {
        match i {
            0 => None,
            1 if self.n_pols > 1 => Some(0),
            _ if i < self.n_pols => {
                let length = (self.index[i as usize + 1] - self.index[i as usize]) / 4;
                Some(usize::try_from(length - 1).expect("validated nonzero"))
            }
            _ => None,
        }
    }

    /// `polynomial_info::coefficients` (filekl.w:758-771): the coefficients
    /// of polynomial `i`, constant term first (empty for the zero
    /// polynomial); `None` when `i` is out of range.
    pub fn coefficients(&self, i: u64) -> Option<&[u32]> {
        if i >= self.n_pols {
            return None;
        }
        let start = (self.index[i as usize] / 4) as usize;
        let end = (self.index[i as usize + 1] / 4) as usize;
        self.coefficients.get(start..end)
    }

    /// `polynomial_info::leading_coeff` (filekl.w:777-783): the last
    /// coefficient stored before the next polynomial; `i` itself for
    /// `i < 2`, so the zero polynomial's "leading coefficient" is 0.
    pub fn leading_coeff(&self, i: u64) -> Option<u32> {
        if i < 2 {
            return u32::try_from(i).ok().filter(|_| i < self.n_pols);
        }
        self.coefficients(i).and_then(|coefs| coefs.last().copied())
    }
}

/// The reader for the progress ("row") file: `progress_info`
/// (filekl.w:861-896). Each of the `N` records is 12 bytes; only the last
/// 4 bytes matter — the count of new distinct polynomials first appearing
/// in that row. `first_pol[y]` accumulates the counts of all rows before
/// `y`.
#[derive(Clone, Debug)]
pub struct ProgressInfo {
    first_pol: Vec<u64>,
}

impl ProgressInfo {
    /// `progress_info::progress_info` (filekl.w:878-891): the file size
    /// must be a multiple of 12 ("Row file size not a multiple of 12").
    pub fn read<R: Read + Seek>(input: &mut R) -> io::Result<Self> {
        let file_size = input.seek(SeekFrom::End(0))?;
        if file_size % 12 != 0 {
            return Err(invalid_data("Row file size not a multiple of 12"));
        }
        let size = file_size / 12;
        let mut first_pol = Vec::new();
        first_pol
            .try_reserve(size as usize + 1)
            .map_err(|_| invalid_data("progress-table allocation failed"))?;
        first_pol.push(0);
        input.seek(SeekFrom::Start(0))?;
        let mut record = [0u8; 12];
        for _ in 0..size {
            input.read_exact(&mut record)?;
            let count = u64::from(u32::from_le_bytes(
                record[8..12].try_into().expect("4 bytes"),
            ));
            let previous = *first_pol.last().expect("first_pol nonempty");
            first_pol.push(previous + count);
        }
        Ok(Self { first_pol })
    }

    pub fn block_size(&self) -> usize {
        self.first_pol.len() - 1
    }

    /// The number of distinct polynomials in rows before `y`
    /// (`first_new_in_row`, filekl.w:868-870); `y == block_size()` yields
    /// the grand total.
    pub fn first_new_in_row(&self, y: usize) -> Option<u64> {
        self.first_pol.get(y).copied()
    }

    /// `progress_info::first_row_for_pol` (filekl.w:893-896): the first row
    /// in which polynomial `i` has appeared.
    pub fn first_row_for_pol(&self, i: u64) -> usize {
        self.first_pol[1..].partition_point(|&count| count <= i)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::kl_polynomial::KlPol;
    use crate::{
        AdjointFiberBudget, BasedRootDatum, BlockGraph, CartanClassification,
        CartanClassificationBudget, CartanId, IntegerLatticeBudget, InvolutionTable,
        InvolutionTableBudget, KgbGraph, KlTable, LatticeInvolution, RealFormSeed,
        StrongRealClassification, WeakRealFormId,
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

    /// An A1 block: the primal form with KGB size `primal_kgb` of the
    /// inner class defined by `datum`, paired with the dual form with KGB
    /// size `dual_kgb`.
    fn build_a1_block(datum: BasedRootDatum, primal_kgb: usize, dual_kgb: usize) -> BlockGraph {
        let involution = LatticeInvolution::identity(&datum).unwrap();
        let inner_class = crate::InnerClass::new(datum, involution, 2).unwrap();
        build_a1_block_in_class(inner_class, primal_kgb, dual_kgb)
    }

    /// [`build_a1_block`] for an explicitly given inner class.
    fn build_a1_block_in_class(
        inner_class: crate::InnerClass,
        primal_kgb: usize,
        dual_kgb: usize,
    ) -> BlockGraph {
        let classification = CartanClassification::build(&inner_class, &class_budget(2)).unwrap();
        let strong = StrongRealClassification::build(&classification, 4_096).unwrap();
        let mut table = InvolutionTable::new(
            &inner_class,
            InvolutionTableBudget::new(64, IntegerLatticeBudget::new(64, 100_000, 100_000, 128)),
        )
        .unwrap();
        let (graph, primal_table) = graph_with_size(
            &inner_class,
            &classification,
            &strong,
            &mut table,
            primal_kgb,
        );
        let dual_class = crate::dual::dual_inner_class(&inner_class, 2, 64).unwrap();
        let dual_classification =
            CartanClassification::build(&dual_class, &class_budget(2)).unwrap();
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
            dual_kgb,
        );
        BlockGraph::build(
            &graph,
            &primal_table,
            &dual_graph,
            &dual_table,
            &dual_class,
            2,
        )
        .unwrap()
    }

    /// block(PGL(2,R), SU(2)): the 1-element A1 block; its single element
    /// (length 1) is real nonparity.
    fn a1_rn_block() -> BlockGraph {
        build_a1_block(BasedRootDatum::standard(vec![vec![2]]).unwrap(), 2, 1)
    }

    /// block(SL(2,R), PGL(2,R)): the 3-element A1 block with two imaginary
    /// type I elements and one real type I element (see
    /// block.rs `sl2r_pgl2r_block_matches_the_frozen_language_anchors`).
    fn a1_type_one_block() -> BlockGraph {
        let datum = BasedRootDatum::from_simple_data(
            1,
            vec![vec![2]],
            vec![crate::Weight::new(vec![2])],
            vec![crate::Coweight::new(vec![1])],
        )
        .unwrap();
        build_a1_block(datum, 3, 2)
    }

    /// block(PGL(2,R), SL(2,R)): the 3-element A1 block with an imaginary
    /// type II element and two real type II elements (see
    /// block.rs `pgl2r_sl2r_dual_block_exercises_the_type_two_links`).
    fn a1_type_two_block() -> BlockGraph {
        build_a1_block(BasedRootDatum::standard(vec![vec![2]]).unwrap(), 2, 3)
    }

    #[test]
    fn block_file_matches_hand_computed_a1_bytes() {
        let block = a1_type_one_block();
        // The structure these expected bytes encode (matching the frozen
        // language anchors of block.rs): z=0 = (x0, y1) and z=1 = (x1, y1)
        // have length 0 and are imaginary type I, crossing each other and
        // sharing the Cayley image z=2 = (x2, y0), which has length 1 and
        // is real type I.
        assert_eq!(block.size(), 3);
        assert_eq!(block.rank(), 1);
        assert_eq!(block.length(0), Some(0));
        assert_eq!(block.length(1), Some(0));
        assert_eq!(block.length(2), Some(1));
        assert_eq!(
            block.descent_value(0, 0),
            Some(BlockDescent::ImaginaryTypeI)
        );
        assert_eq!(
            block.descent_value(1, 0),
            Some(BlockDescent::ImaginaryTypeI)
        );
        assert_eq!(block.descent_value(2, 0), Some(BlockDescent::RealTypeI));
        assert_eq!(block.cayley(0, 0), Some((Some(2), None)));
        assert_eq!(block.cayley(1, 0), Some((Some(2), None)));

        let mut bytes = Vec::new();
        write_block_file(&block, &mut bytes).unwrap();
        let expected: &[u8] = &[
            0x03, 0x00, 0x00, 0x00, // size = 3
            0x01, // rank = 1
            0x01, // max_length = 1
            0x02, 0x00, 0x00, 0x00, // start_length[1] = 2: two elements of length <= 0
            0x00, 0x00, 0x00, 0x00, // descent set of z=0: i1, no weak descents
            0x00, 0x00, 0x00, 0x00, // descent set of z=1: i1, no weak descents
            0x01, 0x00, 0x00, 0x00, // descent set of z=2: r1, weak descent at s=0
            0x02, 0x00, 0x00, 0x00, // ascent(0, 0) = first Cayley image 2
            0x02, 0x00, 0x00, 0x00, // ascent(1, 0) = first Cayley image 2
            0xFE, 0xFF, 0xFF, 0xFF, // ascent(2, 0) = no_good_ascent (a descent)
        ];
        assert_eq!(bytes, expected);
        // The byte-count formula (filekl.w:162-167): 6 + 4L + 4N + 4Nr.
        let (l, n, r) = (1usize, 3, 1);
        assert_eq!(bytes.len(), 6 + 4 * l + 4 * n + 4 * n * r);
    }

    #[test]
    fn block_file_matches_hand_computed_rn_bytes() {
        let block = a1_rn_block();
        // A single element of length 1, real nonparity: no good ascent,
        // and no element of length <= 0.
        assert_eq!(block.size(), 1);
        assert_eq!(block.length(0), Some(1));
        assert_eq!(block.descent_value(0, 0), Some(BlockDescent::RealNonparity));

        let mut bytes = Vec::new();
        write_block_file(&block, &mut bytes).unwrap();
        let expected: &[u8] = &[
            0x01, 0x00, 0x00, 0x00, // size = 1
            0x01, // rank = 1
            0x01, // max_length = 1
            0x00, 0x00, 0x00, 0x00, // start_length[1] = 0: no element of length <= 0
            0x00, 0x00, 0x00, 0x00, // descent set of z=0: rn, no weak descents
            0xFF, 0xFF, 0xFF, 0xFF, // ascent(0, 0) = UndefBlock (real nonparity)
        ];
        assert_eq!(bytes, expected);
        let (l, n, r) = (1usize, 1, 1);
        assert_eq!(bytes.len(), 6 + 4 * l + 4 * n + 4 * n * r);
    }

    #[test]
    fn block_file_round_trip_restores_all_fields() {
        let block = a1_type_two_block();
        let mut bytes = Vec::new();
        write_block_file(&block, &mut bytes).unwrap();
        let size = block.size();
        let rank = block.rank();
        let max_length = block.length(size - 1).unwrap();
        assert_eq!(bytes.len(), 6 + 4 * max_length + 4 * size + 4 * size * rank);

        let info = BlockInfo::read(&mut bytes.as_slice()).unwrap();
        assert_eq!(info.size as usize, size);
        assert_eq!(info.rank as usize, rank);
        assert_eq!(info.max_length as usize, max_length);
        assert_eq!(info.start_length.len(), max_length + 2);
        assert_eq!(info.start_length[0], 0);
        assert_eq!(*info.start_length.last().unwrap() as usize, size);
        for l in 0..=max_length {
            let first = info.start_length[l] as usize;
            assert!(first == size || block.length(first).unwrap() >= l);
            assert!(first == 0 || block.length(first - 1).unwrap() < l);
        }
        for z in 0..size {
            let expected: u32 = (0..rank)
                .filter(|&s| block.descent_value(z, s).unwrap().is_descent())
                .map(|s| 1 << s)
                .sum();
            assert_eq!(info.descent_sets[z], expected, "descent set of {z}");
            for s in 0..rank {
                let expected = match block.descent_value(z, s).unwrap() {
                    value if value.is_descent() || value == BlockDescent::ImaginaryTypeII => {
                        NO_GOOD_ASCENT
                    }
                    BlockDescent::RealNonparity => UNDEF_BLOCK,
                    BlockDescent::ComplexAscent => block.cross(z, s).unwrap() as u32,
                    BlockDescent::ImaginaryTypeI => block.cayley(z, s).unwrap().0.unwrap() as u32,
                    value => panic!("unexpected ascent status {value:?}"),
                };
                assert_eq!(info.ascent(z as u32, s as u32), Some(expected));
            }
        }
    }

    #[test]
    fn block_file_reader_rejects_truncation() {
        let block = a1_type_one_block();
        let mut bytes = Vec::new();
        write_block_file(&block, &mut bytes).unwrap();
        let mut truncated = &bytes[..bytes.len() - 1];
        let error = BlockInfo::read(&mut truncated).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::UnexpectedEof);
    }

    /// The KL table of the 3-element A1 block, fully filled.
    fn a1_type_two_kl() -> (BlockGraph, KlTable<'static, BlockGraph>) {
        let block = a1_type_two_block();
        let mut kl = KlTableHandle::from_handle(block.clone()).unwrap();
        kl.fill(0).unwrap();
        (block, kl)
    }

    #[test]
    fn matrix_file_has_magic_and_consistent_delta_trailer() {
        let block = a1_type_two_block();
        let mut kl = KlTable::new(&block).unwrap();
        kl.fill(0).unwrap();

        let mut cursor = Cursor::new(Vec::new());
        write_matrix_file(&kl, &mut cursor).unwrap();
        let bytes = cursor.into_inner();

        // New-format signature at offset 0 (filekl.w:423-425).
        assert_eq!(&bytes[0..4], &MAGIC_CODE.to_le_bytes());

        // The delta trailer occupies the last 4*N bytes; the cumulative
        // deltas telescope to each row's n_prim offset (filekl.w:414-417).
        let size = block.size();
        let trailer_start = bytes.len() - 4 * size;
        let deltas: Vec<u32> = (0..size)
            .map(|y| u32::from_le_bytes(bytes[trailer_start + 4 * y..][..4].try_into().unwrap()))
            .collect();
        assert_eq!(deltas[0], 1, "start_row(0) is one u32 into the file");
        let mut position = 0u64;
        for (y, &delta) in deltas.iter().enumerate() {
            position += 4 * u64::from(delta);
            // Row y's row-number field sits 4 bytes before start_row(y);
            // row 0's field carries the magic code instead (filekl.w:424).
            let row_number =
                u32::from_le_bytes(bytes[position as usize - 4..][..4].try_into().unwrap());
            let expected = if y == 0 { MAGIC_CODE } else { y as u32 };
            assert_eq!(row_number, expected, "row {y} alignment");
        }
        // The final row's content and the trailer fill the rest of the file.
        let last = size - 1;
        let n_prim =
            u32::from_le_bytes(bytes[position as usize..][..4].try_into().unwrap()) as usize;
        let words = n_prim.div_ceil(32);
        let bitmap: Vec<u32> = (0..words)
            .map(|w| {
                u32::from_le_bytes(
                    bytes[position as usize + 4 + 4 * w..][..4]
                        .try_into()
                        .unwrap(),
                )
            })
            .collect();
        let n_strong: u32 = bitmap.iter().map(|w| w.count_ones()).sum();
        let row_end = position as usize + 4 + 4 * words + 4 * n_strong as usize;
        assert_eq!(row_end, trailer_start, "row {last} runs into the trailer");
    }

    #[test]
    fn matrix_file_find_pol_nr_agrees_with_kl_table() {
        // The type-II block exercises bitmap entries; the type-I block
        // exercises primitivisation through Cayley ascent images.
        for block in [a1_type_two_block(), a1_type_one_block()] {
            let mut kl = KlTable::new(&block).unwrap();
            kl.fill(0).unwrap();

            let mut matrix_bytes = Cursor::new(Vec::new());
            write_matrix_file(&kl, &mut matrix_bytes).unwrap();

            let mut block_bytes = Vec::new();
            write_block_file(&block, &mut block_bytes).unwrap();
            let info = BlockInfo::read(&mut block_bytes.as_slice()).unwrap();

            let mut matrix =
                MatrixInfo::open(info, Cursor::new(matrix_bytes.into_inner())).unwrap();
            assert_eq!(matrix.block_size() as usize, block.size());
            assert_eq!(matrix.rank() as usize, block.rank());
            for y in 0..block.size() {
                assert_eq!(matrix.length(y as u32), block.length(y));
                assert_eq!(
                    matrix.strongly_primitives(y as u32).unwrap().last(),
                    Some(&(y as u32)),
                    "strong primitives of {y} end with y itself"
                );
                for x in 0..block.size() {
                    let from_file = matrix.find_pol_nr(x as u32, y as u32).unwrap();
                    let from_table = kl.kl_pol(x, y).unwrap() as u64;
                    assert_eq!(
                        from_file, from_table,
                        "P_{{{x},{y}}}: file {from_file} vs table {from_table}"
                    );
                }
            }
        }
    }

    #[test]
    fn matrix_file_find_pol_nr_agrees_for_the_owned_handle() {
        // The same round trip through a table owning a detached topology,
        // exercising the generic BlockTopology path of the writer.
        let (block, kl) = a1_type_two_kl();
        let mut matrix_bytes = Cursor::new(Vec::new());
        write_matrix_file(&kl, &mut matrix_bytes).unwrap();
        let mut block_bytes = Vec::new();
        write_block_file(&block, &mut block_bytes).unwrap();
        let info = BlockInfo::read(&mut block_bytes.as_slice()).unwrap();
        let mut matrix = MatrixInfo::open(info, Cursor::new(matrix_bytes.into_inner())).unwrap();
        for y in 0..block.size() {
            for x in 0..block.size() {
                assert_eq!(
                    matrix.find_pol_nr(x as u32, y as u32).unwrap(),
                    kl.kl_pol(x, y).unwrap() as u64,
                    "P_{{{x},{y}}}"
                );
            }
        }
    }

    #[test]
    fn matrix_file_old_format_scan_verifies_row_numbers() {
        let block = a1_type_two_block();
        let mut kl = KlTable::new(&block).unwrap();
        kl.fill(0).unwrap();
        let mut cursor = Cursor::new(Vec::new());
        write_matrix_file(&kl, &mut cursor).unwrap();
        let mut bytes = cursor.into_inner();

        // Undo the new-format signature: drop the delta trailer and
        // restore the row-0 row number over the magic code. The result is
        // exactly what an old writer produced row-wise (bitmap chunks were
        // already 4-byte `put_int` words), so the old-format scan must
        // reproduce the same row positions and polynomial indices.
        let size = block.size();
        bytes.truncate(bytes.len() - 4 * size);
        bytes[0..4].copy_from_slice(&0u32.to_le_bytes());

        let mut block_bytes = Vec::new();
        write_block_file(&block, &mut block_bytes).unwrap();
        let info = BlockInfo::read(&mut block_bytes.as_slice()).unwrap();
        let mut matrix = MatrixInfo::open(info, Cursor::new(bytes)).unwrap();
        for y in 0..block.size() {
            for x in 0..block.size() {
                assert_eq!(
                    matrix.find_pol_nr(x as u32, y as u32).unwrap(),
                    kl.kl_pol(x, y).unwrap() as u64,
                    "P_{{{x},{y}}} via old-format scan"
                );
            }
        }
    }

    #[test]
    fn matrix_file_old_format_reports_misalignment() {
        let block = a1_type_two_block();
        let mut kl = KlTable::new(&block).unwrap();
        kl.fill(0).unwrap();
        let mut cursor = Cursor::new(Vec::new());
        write_matrix_file(&kl, &mut cursor).unwrap();
        let mut bytes = cursor.into_inner();
        let size = block.size();
        bytes.truncate(bytes.len() - 4 * size);
        bytes[0..4].copy_from_slice(&0u32.to_le_bytes());
        // Corrupt the row number of row 1: offset 16 = row 0 (4 u32s).
        bytes[16..20].copy_from_slice(&7u32.to_le_bytes());

        let mut block_bytes = Vec::new();
        write_block_file(&block, &mut block_bytes).unwrap();
        let info = BlockInfo::read(&mut block_bytes.as_slice()).unwrap();
        let error = MatrixInfo::open(info, Cursor::new(bytes)).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert!(error.to_string().contains("Alignment problem"), "{error}");
    }

    #[test]
    fn matrix_file_old_format_reports_premature_end() {
        let block = a1_type_two_block();
        let mut kl = KlTable::new(&block).unwrap();
        kl.fill(0).unwrap();
        let mut cursor = Cursor::new(Vec::new());
        write_matrix_file(&kl, &mut cursor).unwrap();
        let mut bytes = cursor.into_inner();
        let size = block.size();
        bytes.truncate(bytes.len() - 4 * size);
        bytes[0..4].copy_from_slice(&0u32.to_le_bytes());
        bytes.truncate(bytes.len() - 3); // cut the final row short

        let mut block_bytes = Vec::new();
        write_block_file(&block, &mut block_bytes).unwrap();
        let info = BlockInfo::read(&mut block_bytes.as_slice()).unwrap();
        let error = MatrixInfo::open(info, Cursor::new(bytes)).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert!(
            error.to_string().contains("Premature end of file"),
            "{error}"
        );
    }

    #[test]
    fn kl_store_round_trip_degrees_and_coefficients() {
        let mut pool = KlHashTable::new();
        let p1 = pool.match_pol(&KlPol::from_coefficients(vec![1, 1]));
        let p2 = pool.match_pol(&KlPol::from_coefficients(vec![1, 2, 1]));
        let p3 = pool.match_pol(&KlPol::from_coefficients(vec![2, 0, 3, 1]));
        assert_eq!((p1, p2, p3), (2, 3, 4));

        let mut bytes = Vec::new();
        write_kl_store(&pool, &mut bytes).unwrap();
        // Layout: 4 + 5*(N+1) header bytes, then the coefficient area;
        // stored coefficients: One plus p1..p3, i.e. 1+2+3+4 = 10.
        let stored = 10usize;
        assert_eq!(bytes.len(), 4 + 5 * 6 + 4 * stored);

        let info = PolynomialInfo::read(&mut bytes.as_slice()).unwrap();
        assert_eq!(info.n_polynomials(), 5);
        assert_eq!(info.index[0], 0);
        assert_eq!(info.index[1], 0, "the zero polynomial is empty");
        assert_eq!(info.n_coefficients(), stored as u64);
        assert_eq!(info.degree(0), None, "the zero polynomial has no degree");
        assert_eq!(info.degree(1), Some(0));
        assert_eq!(info.degree(p1 as u64), Some(1));
        assert_eq!(info.degree(p2 as u64), Some(2));
        assert_eq!(info.degree(p3 as u64), Some(3));
        assert_eq!(info.coefficients(0), Some([].as_slice()));
        assert_eq!(info.coefficients(1), Some([1].as_slice()));
        assert_eq!(info.coefficients(p1 as u64), Some([1, 1].as_slice()));
        assert_eq!(info.coefficients(p2 as u64), Some([1, 2, 1].as_slice()));
        assert_eq!(info.coefficients(p3 as u64), Some([2, 0, 3, 1].as_slice()));
        assert_eq!(info.leading_coeff(0), Some(0));
        assert_eq!(info.leading_coeff(p3 as u64), Some(1));
    }

    #[test]
    fn kl_store_round_trip_through_a_real_kl_pool() {
        let block = a1_type_two_block();
        let mut kl = KlTable::new(&block).unwrap();
        kl.fill(0).unwrap();

        let mut bytes = Vec::new();
        write_kl_store(kl.pool(), &mut bytes).unwrap();
        let info = PolynomialInfo::read(&mut bytes.as_slice()).unwrap();
        assert_eq!(info.n_polynomials() as usize, kl.pool().len());
        for i in 0..kl.pool().len() {
            let expected: Vec<u32> = kl
                .pool()
                .get(i)
                .unwrap()
                .as_slice()
                .iter()
                .map(|&c| c as u32)
                .collect();
            assert_eq!(
                info.coefficients(i as u64),
                Some(expected.as_slice()),
                "pol {i}"
            );
        }
    }

    #[test]
    fn kl_store_rejects_degrees_of_32_or_more() {
        // A hand-built store: zero, one, and a fake 33-coefficient
        // polynomial (degree 32) whose index span is 132 bytes.
        let mut bytes = Vec::new();
        put_int(3, &mut bytes).unwrap(); // n_pols
        for value in [0u64, 0, 0, 132] {
            put_int((value & 0xFFFF_FFFF) as u32, &mut bytes).unwrap();
            write_bytes(1, value >> 32, &mut bytes).unwrap();
        }
        bytes.resize(bytes.len() + 132, 0); // coefficient area
        let error = PolynomialInfo::read(&mut bytes.as_slice()).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert!(
            error.to_string().contains("Degree found too large (>=32)"),
            "{error}"
        );
    }

    #[test]
    fn progress_file_prefix_sums_and_row_lookup() {
        // Three records with new-polynomial counts 2, 0, 5; the leading 8
        // bytes of each record are ignored.
        let mut bytes = Vec::new();
        for (marker, count) in [(0xDEADu32, 2u32), (0xBEEF, 0), (0x1234, 5)] {
            put_int(marker, &mut bytes).unwrap();
            put_int(marker, &mut bytes).unwrap();
            put_int(count, &mut bytes).unwrap();
        }
        let info = ProgressInfo::read(&mut Cursor::new(bytes)).unwrap();
        assert_eq!(info.block_size(), 3);
        assert_eq!(info.first_new_in_row(0), Some(0));
        assert_eq!(info.first_new_in_row(1), Some(2));
        assert_eq!(info.first_new_in_row(2), Some(2));
        assert_eq!(info.first_new_in_row(3), Some(7));
        assert_eq!(info.first_row_for_pol(0), 0);
        assert_eq!(info.first_row_for_pol(1), 0);
        assert_eq!(info.first_row_for_pol(2), 2);
        assert_eq!(info.first_row_for_pol(6), 2);
        assert_eq!(info.first_row_for_pol(7), 3);
    }

    #[test]
    fn progress_file_rejects_sizes_not_multiple_of_12() {
        let bytes = vec![0u8; 13];
        let error = ProgressInfo::read(&mut Cursor::new(bytes)).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert!(
            error
                .to_string()
                .contains("Row file size not a multiple of 12"),
            "{error}"
        );
    }
}
