# Work order: non-integral common blocks (print/length/dual_KL paths)

Recon completed 2026-08-18 (agent-69), oracle-verified locally against rev
4d3e9449. This document is the authoritative slice plan for the
"non-integral common blocks" registry gap.

## Upstream semantics

Upstream always builds the block of the integral subsystem directly — never
embeds in a full-rank block.

- `common_context` fixes the subsystem once per block:
  `simp_int = integrality_simples(rd, gamma)`, `sub = SubSystem(rd, simp_int)`
  (`gkmod/repr.cpp:2666-2670`).
- `common_block` full ctor: `Block_base(ctxt.subsys().rank())`
  (`blocks.cpp:737`), Dynkin diagram from the subsystem's transposed Cartan
  matrix (`blocks.cpp:753-754`), generation over subsystem generators with
  `int_sys.reflection_word(...)` transporting to parent words
  (`blocks.cpp:756-805`). Proper-subsystem gammas get genuinely smaller
  blocks (fewer generator columns, fewer rows).
- Full ctor (`blocks.cpp:733`, from one srm) vs partial ctor
  (`blocks.cpp:1086`, explicit srm list = Bruhat interval) — orthogonal to
  integrality.
- `print_common_block` header (`atlas-types.w:6676-6688`):
  `Parameter defines element N of the following common block,` +
  `as transformed by <w>` + optional `, simple reflections permuted (...)`.
  Row format (`block_io.cpp:54-110`, `common_block::print` 128-147):
  `z: length [descents] crosses (cayleys) *(x=N,gamma-lambda=v) word`;
  descent/cross/Cayley column count = subsystem rank; gamma-lambda field
  width uses the FULL datum semisimple rank (`3*rk+4`, `block_io.cpp:133`).
- `print_partial_common_block` headers (`atlas-types.w:6720-6732`):
  `Elements <= N of following block` when below-set is full and
  `init+1<size`; `Subset {...} in the following common block:` on a
  Rep_table cache hit; nothing otherwise. `print_partial_block` prints no
  header.

## Current Rust state

Substrate exists and is generic: `IntegralSubsystem::integral`
(`partial_block.rs:141`), `CommonContext::integral` (471),
`PartialBlock::build`/`build_full` (1383/1320),
`RepTable::lookup`/`lookup_full_block` (`rep_table.rs:673`/`706`) with
`integral_context` (799-812), `integral_block_scope` (`deform.rs:182-209`),
`KlTable::from_handle` over `Arc<PartialBlock>` (partial_block.rs:2234),
`LocatedBlock::{raw_row,relative_shift,prepared_query,with_kl_table}`.

Already fixture-pinned: `print_common_block`/`print_block(Param)` (integral,
rank-0 singleton, B2 proper rank-1 — differential 3551242);
`block`/`KL_block`/`KL_column`/`KL_sum_at_s` proper via the located path;
`partial_block`/`block_Hasse`/`W_graph`/`W_cells` at identity attitude;
`cross`/`Cayley` via `CommonContext::integral`.

## Broken / NYI (this work order)

1. **`length(Param)` silently wrong** (`domain_builtins.rs:13436-13457`):
   builds the FULL block and reads the full-block length of the first row
   with matching x. Upstream `Rep_table::length` (`repr.cpp:1435-1442`) does
   `make_dominant` + `lookup` (partial block on the integral subsystem).
   Wrong for ProperSubsystem AND rank-0 Singleton. SMALLEST FIX — all pieces
   exist: replace with `rep.lookup(&dominant)` then
   `block.length(located.raw_row())`.
2. **`dual_KL_block(Param)` silently wrong for non-integral gamma**
   (`domain_builtins.rs:14068-14086`, code comment admits the deferral).
   Upstream (`atlas-types.w:7060-7090`): `lookup_full_block` +
   `Bare_block::dual(block)` (`blocks.cpp:474-507` — purely combinatorial:
   swap x/y, complement length, dual descent codes, reverse cross/Cayley
   links). Needs a `PartialBlock::dual()` (or bare-block view) + dual-side
   KL table; survivor compression/`loc` logic at 14107+ can be kept.
3. **`print_partial_common_block` diverges in sequence**
   (`domain_builtins.rs:10206-10211`): uses fresh `partial_block_rows` per
   call; upstream `print_pc_block_wrapper` uses shared `rt().lookup`
   (`atlas-types.w:6713-6735`). After `print_common_block(pb)` installs the
   full block, the oracle prints `Subset {1} in the following common block:`
   + all 3 rows; Rust prints the fresh singleton with no header. Fix:
   reroute to shared `rep.lookup(&normalised)` (mirroring `partial_block`
   at 14219), `below(raw_row)` via existing `block_bruhat_hasse` (14231),
   emit both upstream headers. `print_partial_block` is fine (upstream also
   builds fresh, `atlas-types.w:6700-6711`).
4. **Coverage hole**: `print_partial_block`/`print_partial_common_block` on
   a proper subsystem unpinned (path is subsystem-generic, likely correct —
   pin it).
5. Cleanup (after 1-3): the non-integral NYI branch in `common_block_rows`
   (9410-9446) is unreachable except the Singleton arm; can be collapsed.

Excluded (other slices): `extended_block`/`raw_ext_KL`/
`partial_extended_KL_block` gate at 14829-14836 and the twisted recursion
gates — see `docs/slices/twisted_ext_proper_workorder.md`. Non-identity
generator attitude — locator slice.

## Fixtures (oracle-verified locally by recon; capture on HPC)

- **A — `length`/`dual_KL_block` on B2 proper** (`length_dual_proper.atlas`):
  ```
  rb := simply_connected(Lie_type("B2"),true)
  ib := inner_class(rb,[[1,0],[0,1]])
  rfb := real_form(ib,2)
  pb := param(KGB(rfb,5),[1,1],[1,0]/2)   # gamma=[3,1]/2; rank-1 subsystem on NON-simple coroot [1,1]
  length(pb)                              # oracle: 0  (Rust today: 1)
  pd := Cayley(0,pb)                      # = final parameter(x=10,lambda=[1,2]/1,nu=[1,7]/2)
  length(pd)                              # oracle: 1  (Rust today: 3)
  dual_KL_block(pb)                       # survivors x=4,5,10; start 1; matrix |1 0 0;0 1 0;1 1 1|; polys [[],[1]]
  ```
- **B — A2 split (SL(3,R)), subsystem on the highest root**
  (`length_dual_proper_a2.atlas`): exercises `to_simple` conjugated parent
  words:
  ```
  ra := simply_connected(Lie_type("A2"),true)
  ia2 := inner_class(ra,[[0,1],[1,0]])
  rs := real_form(ia2,0)
  p3 := param(KGB(rs,3),[0,0],[1,1]/2)    # rank-1 subsystem on α1+α2
  print_common_block(p3)                  # 3 rows, ONE generator column, type-2 codes [i2]/[r2], init=1
  length(p3)                              # oracle: 1  (Rust today: 2)
  print_partial_block(p3)                 # 2-row interval; row 1 has undef cross '*'
  dual_KL_block(p3)                       # survivors x=0,3,3; start 1; matrix |1 0 0;1 1 0;1 0 1|; polys [[],[1]]
  ```
  Caution: compact-class A2 forms renormalize `nu=[1,1]/2` to integral —
  only the split-class x=3 keeps the half-integral nu.
- **C — sequence divergence** (`print_partial_common_block_seq.atlas`):
  ```
  … rfb, pb as in A …
  print_common_block(pb)          # installs the full 3-row block
  print_partial_common_block(pb)  # oracle: "Subset {1} in the following common block:" + 3 rows
  ```
- **D — proper partial print pin** (`print_partial_block_proper.atlas`):
  ```
  … rfb, pb, pd as in A …
  print_partial_block(pb)   # singleton row: 0: 0 [i1] * (*,*) *(x=5,gamma-lambda=[1,-1]/2) 1^e
  print_partial_block(pd)   # full 3-row block
  ```

## Cross-cutting caveat

Fixture B correctness depends on the identity-attitude shift being right for
the SL(3,R) family — the locator probe reports the identity-attitude
assumption is NOT shift-correct for a related A2 configuration
(gamma-lambda off by [0,1] on some rows). If `print_common_block(p3)`
diverges on rows 0/2, that is the locator slice's known issue, not this
substrate.

## Slicing

1. `length(Param)` reroute + fixtures A/B (length lines) — tiny.
2. `print_partial_common_block` shared-lookup reroute + fixtures C/D.
3. `dual_KL_block(Param)` via `PartialBlock::dual()` + fixtures A/B
   (dual_KL_block lines) — the only genuinely new math piece.
