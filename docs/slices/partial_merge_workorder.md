# Work order: cross-block partial merge (`RepTable::commit_partial`)

Recon 2026-08-18. Upstream pinned at rev 4d3e9449 (`/Users/hoxide/mycodes/atlasofliegroups/sources`). All oracle outputs below verified against the local oracle the same day. No files modified.

## 1. Upstream behavior on overlapping partial blocks

### Cache key

The table cache key is `Reduced_param` (`gkmod/repr.h:154-180`):

- `x` — the KGB element **after transport to the canonical (fundamental-alcove) attitude** by `loc.w` (`Reduced_param::reduce`, `repr.cpp:110-125`: `rc.transform<true>(loc.w,srm)` before hashing);
- `int_sys_nr` — the **canonicalized** integral-system id from `InnerClass::int_item` (`structure/innerclass.cpp:1116-1182`): gamma is moved into the fundamental alcove, the canonical integral datum is interned by its positive-coroot set, and `loc.w`/`loc.simp_int`/`loc.simple_pi` record the attitude of the query relative to it;
- `evs_reduced` — mixed-radix packing of the integral-coroot evaluations of `gamma_lambda` modulo the Smith diagonal of the codec (`repr.cpp:118-124`, `127-138`; codec at `repr.cpp:73-95`).

So the key is `(attitude-normalized x, canonical integral datum, coroot-evaluation residue)`. Two params with the same key are isomorphic under a `block_modifier` (`repr.h:485-499`: `locator` = `(int_sys_nr, w, simp_int, simple_pi)` plus `shift`); `Rep_context::make_relative_to` (`repr.cpp:338-350`) computes the modifier relating a stored representative to the query.

The reverse index is `place: vector<pair<bl_it, BlockElt>>` parallel to the reduced pool (`repr.h:555`): one entry per distinct `Reduced_param`, pointing at (block list iterator, row).

### `Rep_table::lookup` (partial) — `repr.cpp:1796-1822`

1. `normalise(sr)`, `mod_reduce`, `Reduced_param::reduce` → key `h`.
2. **Hit** (`h` known): return the cached block at `place[h]`, with `bm` set by `make_relative_to`. No merge, no rebuild.
3. **Miss**: `add_block_below(srm, &subset, bm)` (`repr.cpp:1585-1645`):
   - Generate the Bruhat interval below the seed (`Bruhat_generator::block_below`, `repr.cpp:1476-1563`).
   - Scan interval elements through `append_block_containing` (`repr.cpp:1671-1693`): `co_reduce` each element with the *new* locator; if its key was known **before this call** (`h < place_limit`, fixed at entry) and its block is not yet recorded, push a `sub_triple{block, h, sub_to_new_modifier}` — the modifier computed by `make_relative_to` (`repr.cpp:1685-1690`). Each overlapping block recorded once.
   - **Extend the pool**: for every recorded sub-block, every row representative is `shift`+`transform<false>`-ed into the new attitude and `hash.match`-ed into the pool, appended *beyond* the interval `limit` marker (`repr.cpp:1601-1607`).
   - Construct the merged partial `common_block` from the **union** element list (`repr.cpp:1610-1618`).
   - `set_Bruhat` only for the `limit` interval rows from the generator's covering data (`repr.cpp:1620-1641`).
   - `swallow_blocks_and_append` (`repr.cpp:1695-1740`): for each sub-block, transport every row into the new attitude, `block.lookup` it to build the `embed` translation array, `block.swallow(...)` it (`blocks.cpp:1416-1470`), then `block_erase` the sub-block (`repr.cpp:1743-1770`). Finally splice the new block at the end of `block_list` and re-register **every** row's reduced key in `place`, iterating rows in reverse so the *least* row wins for duplicate keys (`repr.cpp:1727-1739`).
4. Back in `lookup`: `which = last(subset)` — the seed is the top of its interval, i.e. the last row of the interval bitmap in the merged block.

`lookup_full_block` (`repr.cpp:1773-1794`) → `add_block` (`repr.cpp:1648-1668`) is the same machinery with a full-block constructor seed; it swallows any overlapping partials identically.

### `common_block::swallow` — `blocks.cpp:1416-1470`

- NDEBUG-only `check_sub_block` (`blocks.cpp:1378-1396`) pins the invariants: for every sub row `z`, `sub.length(z) == block.length(embed[z])`; `sub.descent(z).permuted(simple_pi) == block.descent(embed[z])`; defined cross links commute with `embed` after generator permutation.
- Moves the sub-block's Hasse rows into the parent, translating covered rows through `embed` (`blocks.cpp:1424-1430`). The parent's own rows are untouched.
- If the sub-block has a KL table, it is swallowed into the parent's table through `embed` (`blocks.cpp:1433-1439` → `kl::KL_table::swallow`, `kl.cpp:953`). The extended-block swallow is `#if 0`'d out upstream.

## 2. Invariants and canonical row order

**Row order is never preserved by insertion — it is re-derived.** Upstream never extends a cached block in place; `add_block_below` builds a brand-new `common_block` from the union element list, and the partial constructor (`blocks.cpp:1086-1248`) ends with `sort()` (`blocks.cpp:1488-1517`, comparator `elt_info_less` at `1483-1486`): rows sorted by `(length, x, y)`, where `y` numbers the distinct `gamma_lambda` values per involution, sorted ascending, offsets accumulated over **decreasing** involution numbers (`blocks.cpp:1106-1131`). The constructor computes links on the union set, and Hasse/KL data from swallowed blocks are translated through `embed` — never consulted for row order.

Consequences:

- A merged block's printed rows are a pure function of its element set; merge history is invisible in the output except through the (larger) element set itself.
- `check_sub_block` guarantees length and link agreement on the overlap, which holds because every cached partial block is a **downward-closed** subset of the full block (Bruhat intervals and unions thereof), so each swallowed element's entire downset already lay in its old block.
- After the merge, `place[h]` for *all* rows of the merged block (including rows that came only from swallowed blocks) points at the new block; old blocks are erased.

## 3. Rust gap map

| Upstream | Rust status |
|---|---|
| `Reduced_param` key | `ReducedParamKey{x, integral_system, residue}` (`rep_table.rs:37-52`); residue codec is `IntegralCodec` (`rep_table.rs:60-280`, mixed-radix at 200-214). **Deviation**: `x` is not attitude-transformed and `integral_system` interns the *exact embedded* simple-root list (`State::integral_system`, `rep_table.rs:376-412`), no Weyl-conjugacy canonicalization — see the comment at `rep_table.rs:431-435`. Equivalent only under identity attitude. |
| `place` / `block_list` | `State::places: HashMap<ReducedParamKey, Place>` + append-only `slots` with `BlockSlot::{Active, Superseded}` (`rep_table.rs:356-374`) — structurally equivalent, iterator-fixup of `block_erase` replaced by `retire_all` (`rep_table.rs:445-452`). |
| Least-row-wins re-registration (`repr.cpp:1727-1739`) | `State::reverse_register` (`rep_table.rs:454-464`) — `.rev()` insert order, matches. |
| `append_block_containing` (`repr.cpp:1671-1693`) | **Missing.** `commit_partial` (`rep_table.rs:466-499`) detects overlaps only to throw `NotYetImplemented` at 490-494. |
| Pool extension + union rebuild (`repr.cpp:1601-1618`) | **Missing.** `RepTable::lookup` (`rep_table.rs:689-719`) builds the block from the bare interval and commits it. |
| `swallow_blocks_and_append` for partials | **Missing** for the partial path; the *full* path already retires overlapping partials (`lookup_full_block`, `rep_table.rs:751-787`). |
| `common_block::swallow` Hasse import | **Not needed as data movement**: Rust never stores Hasse in `PartialBlock`; `bruhat_hasse` (`block_access.rs:46-94`, port of `blocks.cpp:1576-1656`) recomputes it from links on demand, which coincides with upstream's imported rows by downward-closedness. |
| `KL_table::swallow` (`kl.cpp:953`) | **Missing.** `BlockRecord.kl_table` (`rep_table.rs:302`) dies with the retired record and is recomputed lazily — observably identical, perf-only gap. The full-promotion test `promoted_partial_keeps_old_kl_and_full_gets_a_fresh_table` (`rep_table.rs:1789`+) already blesses fresh-table semantics. |
| `block_modifier` / `make_relative_to` | Ported but unwired: `locator.rs` (step 1), `block_modifier.rs` (step 2); `GeneratorAttitude::Identity` (`rep_table.rs:305-310`) is the gate. The merge port below stays inside identity attitude. |

### Minimal port sketch (identity attitude only)

Restructure `RepTable::lookup` (`rep_table.rs:689-719`) to mirror `add_block_below`:

1. Generate the interval (unchanged) and compute each interval element's `ReducedParamKey` directly (via `integral_key`, no need to build the block first).
2. Collect the set of active records hit by those keys — with the `place_limit` semantics (only pre-existing places). If empty: current fast path.
3. If non-empty: extend the pool with every element of every overlapping record's block (identity attitude ⇒ `shift=0`, `w=id`: the stored `StandardReprMod`s are inserted as-is, dedup by value equality — upstream's `hash.match`). Rebuild `PartialBlock::build` on the union, recompute `exact_seed_row = block.lookup(seed)` and `row_keys_for` over the merged block.
4. Commit: `retire_all(overlap_ids)` (exists), `insert_record(block, false)` (exists), `reverse_register` (exists). The retired records' `Arc`s keep old `LocatedBlock` handles alive; their KL caches are dropped (lazy recompute; port `KL_table::swallow` later as a perf slice).
5. Concurrency: upstream is single-threaded; Rust re-probes under the lock. Simplest correct loop: probe overlap set → build outside the lock → under the lock re-verify the overlap set is unchanged, else discard and rebuild (union is monotonic, so this terminates). The existing gates/tests `unsupported_partial_overlap_is_failure_atomic` (`rep_table.rs:2057`) and `concurrent_overlapping_partials_leave_the_first_commit_unchanged` (`rep_table.rs:2246`) pin the NYI and must be rewritten to pin merge outcomes.
6. Remove the now-dead fallback arm in `length(Param)` (`domain_builtins.rs:13478-13482`).

`lookup_full_block` needs no change: it already retires partials; merged partials retire identically.

## 4. Observable behavior: NYI vs fixed

- **`print_partial_common_block` on an overlap sequence** (`domain_builtins.rs:10186-10208`): today the NYI surfaces as a diagnostic ("not yet implemented: merging overlapping partial representation blocks"); the oracle prints the merged block with `Subset {...} in the following common block:` / `Elements <= N of following block` headers (`print_pc_block_wrapper`, `atlas-types.w:6713-6735`). Header text, row count, row numbering, and link columns all change.
- **Frozen anchor `tests/fixtures/domain/print_partial_common_block_seq.atlas`**: full-then-partial; the partial lookup's seed key is a hit in the full block, so the merge path is never reached — stays green, pins the *promotion* half of the machinery. Unaffected.
- **`tests/fixtures/domain/length_dual_proper.atlas`**: `length(pb)` caches the singleton interval below `pb`; `length(Cayley(0,pb))` overlaps it (the Cayley image's interval contains an srm with the same reduced key). Today: NYI → full-block fallback → correct value. After the fix: merge path, identical output. Becomes a regression fixture for the merge instead of the fallback.
- **`twisted_full_deform` slice 5** (`docs/slices/twisted_ext_proper_workorder.md:130-149`): the reducibility-point recursion looks up many related params on one integral system; overlapping partials are the norm, not the exception. Without the merge the options are an error or a full-block build per reducibility point — the exact cost upstream's memoize-and-swallow design exists to avoid (fatal for large groups). The merge is a hard prerequisite for a faithful slice 5.

## 5. Fixture proposals (oracle-verified, rev 4d3e9449)

All four currently fail in Rust with the NYI diagnostic on the second `print_partial_common_block` (inferred from `commit_partial` + the 10198-10202 error mapping; not re-run through the Rust CLI per the heavy-build rule).

### F1 `partial_merge_containment.atlas` — B2 split form 2, containment merge + header flip on re-query

```
rb : RootDatum
rb := simply_connected(Lie_type("B2"),true)
ib : InnerClass
ib := inner_class(rb,[[1,0],[0,1]])
rfb : RealForm
rfb := real_form(ib,2)
pb : Param
pb := param(KGB(rfb,5),[1,1],[1,0]/2)
print_partial_common_block(pb)
pd : Param
pd := Cayley(0,pb)
print_partial_common_block(pd)
print_partial_common_block(pb)
```

Oracle output (payload lines):

```
Value: final parameter(x=5,lambda=[2,2]/1,nu=[1,-1]/2)
0:  0  [i1]  *   (*,*)  *(x=5,gamma-lambda=  [1,-1]/2)  1^e
Value: final parameter(x=10,lambda=[1,2]/1,nu=[1,7]/2)
0:  0  [i1]  1   (2,*)  *(x= 4,gamma-lambda=  [1,-1]/2)  1^e
1:  0  [i1]  0   (2,*)  *(x= 5,gamma-lambda=  [1,-1]/2)  1^e
2:  1  [r1]  2   (0,1)  *(x=10,gamma-lambda=   [3,3]/2)  1^2x1^e
Subset {1} in the following common block:
0:  0  [i1]  1   (2,*)  *(x= 4,gamma-lambda=  [1,-1]/2)  1^e
1:  0  [i1]  0   (2,*)  *(x= 5,gamma-lambda=  [1,-1]/2)  1^e
2:  1  [r1]  2   (0,1)  *(x=10,gamma-lambda=   [3,3]/2)  1^2x1^e
```

Pins: singleton block swallowed into the 3-element interval; re-query of `pb` hits the merged block at row 1 and prints the `Subset {1}` header (vs. no header on first call).

### F2 `partial_merge_union.atlas` — B2 at gamma=rho, incomparable intervals, row-count-changing merge

```
rb : RootDatum
rb := simply_connected(Lie_type("B2"),true)
ib : InnerClass
ib := inner_class(rb,[[1,0],[0,1]])
rfb : RealForm
rfb := real_form(ib,2)
p4 : Param
p4 := param(KGB(rfb,4),[0,0],[1,1]/1)
print_partial_common_block(p4)
p6 : Param
p6 := param(KGB(rfb,6),[0,0],[1,1]/1)
print_partial_common_block(p6)
print_partial_common_block(p4)
```

Oracle output (payload):

```
Value: final parameter(x=4,lambda=[1,1]/1,nu=[1,-1]/1)
0:  0  [i1,i1]  1  *   (2,*)  (*,*)  *(x=0,gamma-lambda=   [0,0]/1)  e
1:  0  [i1,ic]  0  1   (2,*)  (*,*)  *(x=1,gamma-lambda=   [0,0]/1)  e
2:  1  [r1,C+]  2  *   (0,1)  (*,*)  *(x=4,gamma-lambda=   [0,0]/1)  1^e
Value: final parameter(x=6,lambda=[1,1]/1,nu=[-1,2]/2)
Subset {0,2,4} in the following common block:
0:  0  [i1,i1]  1  2   (3,*)  (4,*)  *(x=0,gamma-lambda=   [0,0]/1)  e
1:  0  [i1,ic]  0  1   (3,*)  (*,*)  *(x=1,gamma-lambda=   [0,0]/1)  e
2:  0  [i1,i1]  *  0   (*,*)  (4,*)  *(x=2,gamma-lambda=   [0,0]/1)  e
3:  1  [r1,C+]  3  *   (0,1)  (*,*)  *(x=4,gamma-lambda=   [0,0]/1)  1^e
4:  1  [C+,r1]  *  4   (*,*)  (0,2)  *(x=6,gamma-lambda=   [0,0]/1)  2^e
Subset {0,1,3} in the following common block:
0:  0  [i1,i1]  1  2   (3,*)  (4,*)  *(x=0,gamma-lambda=   [0,0]/1)  e
1:  0  [i1,ic]  0  1   (3,*)  (*,*)  *(x=1,gamma-lambda=   [0,0]/1)  e
2:  0  [i1,i1]  *  0   (*,*)  (4,*)  *(x=2,gamma-lambda=   [0,0]/1)  e
3:  1  [r1,C+]  3  *   (0,1)  (*,*)  *(x=4,gamma-lambda=   [0,0]/1)  1^e
4:  1  [C+,r1]  *  4   (*,*)  (0,2)  *(x=6,gamma-lambda=   [0,0]/1)  2^e
```

Pins: intervals below p4 (`{x0,x1,x4}`) and p6 (`{x0,x2,x6}`) overlap at x=0 only; merged block has 5 rows (a fresh lookup of p6 alone would print 3); canonical `(length,x,y)` re-sort (length-0 rows ordered x=0,1,2); links recomputed on the union (row 0 gen-1 cross becomes defined, Cayley targets renumbered).

### F3 `partial_merge_chain.atlas` — chain merge, lengths, and final full promotion

F2's prelude plus:

```
p10 : Param
p10 := param(KGB(rfb,10),[0,0],[1,1]/1)
print_partial_common_block(p10)
length(p4)
length(p6)
length(p10)
print_common_block(p6)
```

Oracle output after F2's payload:

```
Value: final parameter(x=10,lambda=[1,1]/1,nu=[1,1]/1)
 0:  0  [i1,i1]   1   2   ( 4, *)  ( 6, *)  *(x= 0,gamma-lambda=   [0,0]/1)  e
 1:  0  [i1,ic]   0   1   ( 4, *)  ( *, *)  *(x= 1,gamma-lambda=   [0,0]/1)  e
 2:  0  [i1,i1]   3   0   ( 5, *)  ( 6, *)  *(x= 2,gamma-lambda=   [0,0]/1)  e
 3:  0  [i1,ic]   2   3   ( 5, *)  ( *, *)  *(x= 3,gamma-lambda=   [0,0]/1)  e
 4:  1  [r1,C+]   4   8   ( 0, 1)  ( *, *)  *(x= 4,gamma-lambda=   [0,0]/1)  1^e
 5:  1  [r1,C+]   5   9   ( 2, 3)  ( *, *)  *(x= 5,gamma-lambda=   [0,0]/1)  1^e
 6:  1  [C+,r1]   7   6   ( *, *)  ( 0, 2)  *(x= 6,gamma-lambda=   [0,0]/1)  2^e
 7:  2  [C-,i2]   6   7   ( *, *)  (10, *)  *(x= 7,gamma-lambda=   [0,0]/1)  1x2^e
 8:  2  [i1,C-]   9   4   (10, *)  ( *, *)  *(x= 8,gamma-lambda=   [0,0]/1)  2x1^e
 9:  2  [i1,C-]   8   5   (10, *)  ( *, *)  *(x= 9,gamma-lambda=   [0,0]/1)  2x1^e
10:  3  [r1,r2]  10   *   ( 8, 9)  ( 7, *)  *(x=10,gamma-lambda=   [0,0]/1)  1^2x1^e
Value: 1
Value: 1
Value: 3
Parameter defines element 6 of the following common block,
as transformed by <>:
 0:  0  [i1,i1]   1   2   ( 4, *)  ( 6, *)  *(x= 0,gamma-lambda=   [0,0]/1)  e
 1:  0  [i1,ic]   0   1   ( 4, *)  ( *, *)  *(x= 1,gamma-lambda=   [0,0]/1)  e
 2:  0  [i1,i1]   3   0   ( 5, *)  ( 6, *)  *(x= 2,gamma-lambda=   [0,0]/1)  e
 3:  0  [i1,ic]   2   3   ( 5, *)  ( *, *)  *(x= 3,gamma-lambda=   [0,0]/1)  e
 4:  1  [r1,C+]   4   8   ( 0, 1)  ( *, *)  *(x= 4,gamma-lambda=   [0,0]/1)  1^e
 5:  1  [r1,C+]   5   9   ( 2, 3)  ( *, *)  *(x= 5,gamma-lambda=   [0,0]/1)  1^e
 6:  1  [C+,r1]   7   6   ( *, *)  ( 0, 2)  *(x= 6,gamma-lambda=   [0,0]/1)  2^e
 7:  2  [C-,i2]   6   7   ( *, *)  (10,11)  *(x= 7,gamma-lambda=   [0,0]/1)  1x2^e
 8:  2  [i1,C-]   9   4   (10, *)  ( *, *)  *(x= 8,gamma-lambda=   [0,0]/1)  2x1^e
 9:  2  [i1,C-]   8   5   (10, *)  ( *, *)  *(x= 9,gamma-lambda=   [0,0]/1)  2x1^e
10:  3  [r1,r2]  10  11   ( 8, 9)  ( 7, *)  *(x=10,gamma-lambda=   [0,0]/1)  1^2x1^e
11:  3  [rn,r2]  11  10   ( *, *)  ( 7, *)  *(x=10,gamma-lambda=   [1,0]/1)  1^2x1^e
```

Pins: second-order merge (11-row union block, row 7's Cayley shows `(10,*)` since row 11 is absent — partial links differ from the full block); lengths unchanged by merge history; `print_common_block` afterwards promotes through the merged partial cache to the 12-row full block.

### F4 `partial_merge_a2.atlas` — A2 su(2,1), symmetric two-generator overlap

```
ra : RootDatum
ra := simply_connected(Lie_type("A2"),true)
ia : InnerClass
ia := inner_class(ra,[[1,0],[0,1]])
rfa : RealForm
rfa := real_form(ia,1)
q3 : Param
q3 := param(KGB(rfa,3),[0,0],[1,1]/1)
print_partial_common_block(q3)
q4 : Param
q4 := param(KGB(rfa,4),[0,0],[1,1]/1)
print_partial_common_block(q4)
print_partial_common_block(q3)
length(q3)
length(q4)
```

Oracle output (payload):

```
Value: final parameter(x=3,lambda=[1,1]/1,nu=[-1,2]/2)
0:  0  [i1,i1]  *  1   (*,*)  (2,*)  *(x=0,gamma-lambda=   [0,0]/1)  e
1:  0  [ic,i1]  1  0   (*,*)  (2,*)  *(x=2,gamma-lambda=   [0,0]/1)  e
2:  1  [C+,r1]  *  2   (*,*)  (0,1)  *(x=3,gamma-lambda=   [0,0]/1)  2^e
Value: final parameter(x=4,lambda=[1,1]/1,nu=[2,-1]/2)
Subset {0,1,4} in the following common block:
0:  0  [i1,i1]  1  2   (4,*)  (3,*)  *(x=0,gamma-lambda=   [0,0]/1)  e
1:  0  [i1,ic]  0  1   (4,*)  (*,*)  *(x=1,gamma-lambda=   [0,0]/1)  e
2:  0  [ic,i1]  2  0   (*,*)  (3,*)  *(x=2,gamma-lambda=   [0,0]/1)  e
3:  1  [C+,r1]  *  3   (*,*)  (0,2)  *(x=3,gamma-lambda=   [0,0]/1)  2^e
4:  1  [r1,C+]  4  *   (0,1)  (*,*)  *(x=4,gamma-lambda=   [0,0]/1)  1^e
Subset {0,2,3} in the following common block:
(same 5-row block as above)
Value: 1
Value: 1
```

Pins the merge on a second root datum / Weyl group, with the overlap element at x=0 reached through generator 1 (F2 reaches it through both).

---

**Caveats**: I did not run the Rust CLI on F1–F4 (heavy-build rule); the current-failure claim rests on `commit_partial`'s NYI at `rep_table.rs:490-494` and the error mapping at `domain_builtins.rs:10198-10202`. The KL-swallow skip is justified by determinism of lazy recomputation, but if slice 5 benchmarks show KL recompute cost, port `kl::KL_table::swallow` (`kl.cpp:953`) with the `embed` translation as a follow-up.