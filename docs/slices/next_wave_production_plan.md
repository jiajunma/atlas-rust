# Next-wave production plan — recon report

## 1. NYI-gate cross-check (all gates accounted for)

Every live gate in the Rust sources maps to the in-flight queue **except the ones listed in §2**:

| Gate | Location | Status |
|---|---|---|
| non-identity attitude (partial_block/block/W_graph/block_Hasse) | `domain_builtins.rs:10206, 14259, 14605, 14745` | locator slice — in flight |
| `commit_partial` overlap merge | `rep_table.rs:464` | cross-block partial merge — in flight |
| ext block non-identity attitude | `ext_block.rs:962` | locator/ext_block_proper — in flight |
| ext family non-integral gamma | `domain_builtins.rs:14861-14867` | twisted-ext slices 1–2 — in flight |
| `twisted_full_deform` proper-subsystem rp | `deform.rs:810` | twisted-ext slice 5 — in flight |
| `with_integral_block` ProperSubsystem | `domain_builtins.rs:7835` | twisted-ext slices 3–5 — in flight |

Meta sweep: only 10 `rust_status: not_implemented` metas exist (3 locator, ext_block_proper, 2 length_dual_proper, 4 partial_merge) — all queued. **Everything below needs new fixtures + oracle capture; nothing is pre-frozen.**

## 2. Genuinely un-queued items

**A. Non-integral common block (classic Param surface)** — the big one.
- Gate: `common_block_rows`, `crates/atlas-core/src/domain_builtins.rs:9431-9435` ("common block at a non-integral infinitesimal character…"), fires when gamma is non-integral with ≥1 integral coroot (proper integral subsystem of rank ≥ 1). Consumers: `print_common_block`/`print_partial_common_block` (`:9923, :10130`), and via the located path the whole Param surface (`partial_block`, `block_Hasse`, `W_graph`/`W_cells`, `KL_sum_at_s`, `KL_block`, `length`).
- Upstream anchor: `common_context` ctor `gkmod/repr.cpp:2666-2670` + `common_block` subsystem ctors `gkmod/blocks.cpp:733-1248`. No special-casing upstream.
- Explicitly declared "separate gap, do not touch" in `docs/slices/twisted_ext_proper_workorder.md:34`; also listed in `docs/LANGUAGE.md:80` remaining set. `LANGUAGE.md` is the authoritative "remaining" list and this is its last un-claimed entry.
- Dependencies: locator slice must land first (the new `located_common_block_rows` path at `domain_builtins.rs:9527` is the wiring point); `RepTable::lookup_full_block` already builds proper-subsystem full common blocks (identity attitude), so the slice is mostly **wiring + fixtures**, not new math.
- Fixture/oracle: none. Probe B2 form 2 (`pb = param(KGB(rfb,5),[1,1],[1,0]/2)`, already used by `block_hasse_param_proper`) + A2 su(2,1) fractional-nu cases.

**B. Ordinary `full_deform` silent full-block approximation** (no gate — silent divergence).
- `full_deformation_terms`, `domain_builtins.rs:2282-2321`: reducibility-point recursion rebuilds the **full** block with no `integral_block_scope` check; upstream routes through `common_context`. Diverges for non-integral reducibility points. Flagged as boundary in the twisted workorder (line 37-39) but not claimed by any slice.
- Dependency rank: after A (needs the proper-subsystem common block it will consume).

**C. `KL_sum_at_s` per-element lambda-rho** — known deviation (REMAINING line 660): uses the input parameter's lambda-rho for every block element; height-parity mismatch for mid-block parameters (fixtures dodge by using the lowest element). Fix rides on A's located block members (per-element gamma-lambda already computed there). Small once A lands.

**D. Generic axis.w row operators `##`/`#` on `([*],*)` / `(*,[*])`** — unimplemented; bare `[1,2]##[3,4]` silently coerces to the vec overload and prints spaced vs the oracle's compact row form (REMAINING batch-2 note, lines 159-163). Lives in the typed operator layer (`typed.rs:3402` region handles unary row `#`; the binary/join generics are absent). Language-layer, no real-group deps.

**E. `Weyl_orbit`/`Weyl_orbit_ws` oversize-vector semantics** — wrapper does no size check upstream; `v.size()!=rank` output diverges (`domain_builtins.rs:11906-11952`; detail in `docs/slices/post_weyl_lang_queue.md` §1.5). Small: probe oracle, remove/adjust the check.

**F. `integrality_points` display** — Rust prints RatVec list `[[ 1 ]/1]` vs oracle RatNum list `[1/1]` (recorded in `integrality_points.meta.json` notes; fixture currently dodges). Small display-layer fix, `domain_builtins.rs:12406` + value display.

**G. Rank-0 non-integral ext builtins** — `domain_builtins.rs:14861` rejects *all* non-integral gamma for `extended_block`/`raw_ext_KL`, including rank-0 where upstream uniformly returns a size-1 block. Open question (a) in the twisted workorder; may ride along with ext_block_proper — flag to that subagent, else a tiny follow-up.

**H. Perf-only / deferred (do not dispatch as compat work):** `KL_table::swallow` port (`kl.cpp:953`, perf follow-up to partial merge); readline completion (TTY-only, deferred by user decision); timed-deform partial-formula retention (unprobed boundary).

## 3. Parser-layer gaps (from `docs/slices/global_batch4_workorder.md` §2.4/§3)

Both upstream anchors are in `sources/interpreter/parser.y` (not a separate axis-parser.y):

- **2-D slice `M[rlo:rhi, clo:chi]`** — `parser.y:658-705`: desugared to a call of hidden `"matrix slicer"` with a packed flag int (bits 0x2/0x4/0x10/0x20 = from-end bits; absent upper bound compiles as `0` *with* the bit ⇒ `upb=dim`). Plain `M[i,j]` (no colons) is **not** part of this gap — it's a tuple subscription (`parser.y:585-598`). Rust side: `PostfixSuffix::Slice` holds a single lower/upper pair (`syntax.rs:548-553`); `M[0:2, 0:3]` is a parse error today. Scope: extend `SliceFlags` (2 more bits) + `PostfixSuffix::Slice`/`Expr::Slice` to two bound pairs in `syntax.rs`, then hook eval to the `swiss_matrix_knife` engine batch-4 lands (do **not** register the hidden name). Medium-small; touches `syntax.rs` + typed/eval slice arm.
- **commabarlist `[a,b | c,d]`** — `parser.y:370-376` + `commabarlist` at `:402-410`: desugars to `transpose (mat: [[a,b],[c,d]])` via hidden `"transpose "`. Rust: `Bar` is lexed (`syntax.rs:1051`) but no expression production consumes it — parse error today. Scope: one new parser production building the row-list, resolved to a matrix **directly in the parser/typer** (batch-2 `stack_rows`/transpose machinery exists; per the workorder, never register the hidden name). Small; mostly `syntax.rs` + one typed/display arm. Beware: `;` inside a display is statement sequencing, not a row separator.

## 4. Prioritized dispatch list (after current queue drains)

1. **A — non-integral common block.** Files: `domain_builtins.rs` (`common_block_rows`, `located_common_block_rows`, consumers), possibly `rep_table.rs` (read-only wiring). Needs the locator slice merged first. Largest compat item left; unlocks B and C. Dispatch alone (it owns `domain_builtins.rs`).
2. **Parser pair: 2-D slice + commabarlist.** Files: `syntax.rs` (+ a small typed/eval arm). Self-contained, no real-group deps; 2-D slice needs batch-4's `swiss_matrix_knife` engine merged. One subagent, both gaps (same file).
3. **C + B — `KL_sum_at_s` lambda-rho fix, then `full_deform` scope fix.** Files: `domain_builtins.rs` — must be **sequenced after A** (same file, same machinery), not concurrent with it.
4. **D, E, F — small surface fixes.** D: `typed.rs` operator layer (collides with item 2's typed arm if concurrent — sequence or split by line range). E and F: `domain_builtins.rs` display/wrapper — fold into the item-3 subagent's tail or run after it.

Concurrency rule for the orchestrator: items 1, 3, E, F all touch `domain_builtins.rs` — never run two concurrently. Item 2 (`syntax.rs`) and D (`typed.rs`) are the only clean parallel pair, provided D's typed.rs edits avoid item 2's slice-eval arm.