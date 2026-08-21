# filekl format adapter design

Status: implementation in progress (2026-08-21). This is the last
LANGUAGE.md matrix row ("KL and file formats", previously `planned`).

## Scope

Byte-exact Rust port of `sources/io/filekl.w` (896 lines CWEB): the
binary block file, KL matrix file, KL polynomial store file, and the
read-only progress ("row") file. The interpreter never references these
formats (zero matches in `sources/interpreter/*.w`; `blockwrite` /
`klwrite` live in `sources/interface/`, which the interpreter does not
link), so this lands as a library module
`crates/atlas-real-group/src/filekl.rs` — no CLI surface, no builtins.

## Format summary (from filekl.w; endianness always little-endian unsigned)

Shared primitives (basic_io): `put_int` = u32 LE; `read/write_bytes(n)`
for n in 1..=8, LSB first; dense streams, no padding or tags.
Constants: `UndefBlock = 0xFFFFFFFF`, `no_good_ascent = 0xFFFFFFFE`,
`magic_code = 0x06ABDCF0`.

- **Block file** (filekl.w:176-229, reader 321-354): header `size` u32,
  `rank` u8, `max_length` u8, `start_length[1..=L]` (L × u32); then N
  descent-set words (u32 bitmask of weak descents), then the N×r ascent
  table (u32; descent/ImaginaryTypeII → `no_good_ascent`, RealNonparity →
  `UndefBlock`, ComplexAscent → cross, ImaginaryTypeI → Cayley first).
  Total size `6 + 4L + 4N + 4Nr`.
- **Matrix file** (filekl.w:365-426, reader 496-639): per row y: row
  number u32, `n_prim` u32 (= weak primitives strictly below y, plus 1
  for y itself), bitmap ⌈n_prim/32⌉ × u32 (bit per primitive with nonzero
  KL polynomial; the y bit always set), (popcount−1) × u32 polynomial
  indices (zeros skipped), trailing diagonal u32 = 1. Trailer: N × u32
  deltas of row start offsets; first 4 bytes then overwritten with
  `magic_code`. ⚠️ Canonical format only: current upstream master's
  `prim_map` regression (kl.cpp:223-232) would emit `n_prim` one short
  without the y bit — do not replicate; all readers and historical files
  use the canonical form.
- **KL store** (filekl.w:646-692, reader 730-783): `n_pols` u32, then
  (N+1) × 5-byte LE offsets into the coefficient area, then coefficients
  constant-first as u32 LE. Zero = empty (index[0]=index[1]=0), One = one
  coefficient (so index[2]=4 self-describes the coefficient width).
  Degrees ≥ 32 are rejected on read (cached_pol_info).
- **Progress file** (reader only, filekl.w:861-896): N × 12-byte records;
  bytes 8-11 = u32 count of new distinct polynomials per row; file size
  must be divisible by 12.

## Module API

- LE helpers mirroring `basic_io` (`read_le<N>` / `write_le`).
- `write_block_file(&BlockGraph, impl Write+Seek)` /
  `BlockFile::read(impl Read)` (with validation; upstream silently reads
  EOF as 0xFF — we reject truncation instead).
- `write_matrix_file(block_size, rows, impl Write+Seek)` /
  `MatrixFile::read(impl Read+Seek, block_size)` with `find_pol_nr(x,y)`
  and the old-format (no magic) linear-scan fallback with upstream's
  "Alignment problem" / "Premature end of file" errors.
- `write_kl_store(&KlHashTable, impl Write)` /
  `PolynomialInfo::read(impl Read+Seek)`.
- `ProgressFile::read(impl Read+Seek)` (reader only).

## Oracle strategy (HPC)

No interpreter path exists; the oracle is the stand-alone utilities in
`sources/stand-alone/` (KLread, matstat, polstat — Makefile targets,
built on demand). Differential flow:

1. A Rust driver builds blocks + KL tables for small groups (A1, A2
   quasisplit, B2, G2) through the existing domain pipeline and writes
   the three files per block.
2. Upstream KLread/matstat/polstat read those files on HPC; their
   printed statistics (block size, primitive counts, polynomial count,
   max degree) must match the values our own tables report.
3. Round-trip: our reader parses our writer's output byte-identically
   (unit tests) plus any upstream-produced files if found.

Verification artifacts land under `results/<sha>/<job>/` like every
other differential; LANGUAGE.md's KL row flips to `supported` only
after that report passes.
