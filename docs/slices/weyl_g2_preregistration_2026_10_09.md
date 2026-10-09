# G2 Weyl capture pre-registration: upstream source reading — 2026-10-09

Status: **source-reading analysis** of the pinned oracle (upstream commit
`7e1b958c7aa9456769cc9cf09ac1542814b4800a`, raw files fetched from GitHub
2026-10-09 into a temporary directory, removed after reading) cross-checked
against the landed Rust repair (`690c2b92`).  No Atlas/Cargo execution.  This
pin-level reading does NOT replace the HPC capture; it pre-registers exact
expected outcomes so the capture's interpretation is immediate, and it
identifies one miscalibrated prediction set and one vacuous test before they
cost analysis time.

Pin-validity follow-up (same day): upstream master moved to `5ae51193`
(10 commits ahead; receipt `tests/reference/hpc/upstream_head_2026_10_09.json`,
inventory `upstream_compare_2026_10_09.json`).  Hunk-by-hunk, the diff does
not touch any section cited below (the atlas-types.w hunks are
root_coradical/coroot_radical containers and KGB_cross/Cayley/status
integer-0 fixes; the rootdata.cpp hunk is a const qualifier), so every
citation here stands at the pin, and the cited lines are content-unchanged
at the new HEAD.  Existing goldens remain bound to `7e1b958c`; a pin refresh
is a separate HPC-replay transition.

## Upstream facts (file:line at the pin)

1. **Interning is by PreRootDatum content.** `root_datum_entry::hashCode`
   (atlas-types.w, "We have a simple hash function...") folds
   `prefer_coroots()` plus every simple-root and simple-coroot matrix entry;
   `root_datum_value::build` interns by that key with weak pointers.
2. **Interpreter `dual()` swaps and flips, never renumbers.**
   `root_datum_value::dual` (atlas-types.w, the definition right after `W()`):
   `PreRootDatum pre(val); pre.dualise(); build(...); if
   (result->W_ptr==nullptr) W(), result->W_ptr = W_ptr;`.
   `PreRootDatum::dualise` (prerootdata.h:101-102) is exactly
   `simple_roots.swap(simple_coroots); prefer_co = !prefer_co;`.
3. **The fresh RootDatum renumbers from the transposed Cartan.**
   `RootDatum::RootDatum(const PreRootDatum&)` (rootdata.cpp:820) builds
   `RootSystem(prd.Cartan_matrix(), prd.prefer_coroots())` — a fresh
   numbering from the *transposed* Cartan.  The comment above
   `RootDatum(RootDatum, DualTag)` (rootdata.cpp:867-873) states the
   metadata dual keeps the primal ordering and that this "is not the
   ordering that would have been used in a freshly constructed root datum" —
   i.e., the interpreter dual (fresh construction) and the DualTag metadata
   dual (Fokko-only, innerclass.cpp) have *different* root orderings.
4. **RootDatum `=` is interned-pointer equality.**
   `root_datum_eq_wrapper` (atlas-types.w:1370-1374): `rd0==rd1`, "compare
   pointers".  Via content interning: equality ⟺ identical
   (roots, coroots, preference) content among live values.
5. **Weyl compatibility is WeylGroup-pointer identity, checked before the
   no-value gate.** `W_elt_eq_wrapper`/`neq_wrapper`/`prod_wrapper`
   (atlas-types.w:2576-2613): `if (&w0->W!=&w1->W) throw
   runtime_error("Weyl group mismatch")` precedes `if
   (l==eval_level::no_value)`.  The product mutates and returns `w0` — the
   **left** operand's owner (`w0->W.mult(w0->val,w1->val); push_value(w0)`).
6. **Constructors**: `simply_connected(lt,pc)` ≡ `root_datum(lt,id_mat,pc)`;
   `adjoint(lt,pc)` ≡ `root_datum(lt,transpose_Cartan(lt),pc)`
   (atlas-types.w:1319-1360).  PreRootDatum(LieType) (prerootdata.cpp:47-62):
   simple-root columns are Cartan rows, simple coroots the identity basis.

## Consequence: the canonical dual is not directly constructible for G2

For G2 the fixed Bourbaki Cartan is `[[2,-1],[-3,2]]`; dualise produces
coroot matrix `[[2,-3],[-1,2]]` (the transpose).  No
`simply_connected`/`adjoint(Lie_type("G2"),·)` constructor produces the
transposed content, so the canonical dual `dual(SC(G2,true))` is a distinct
interned object from both `adjoint(G2,false)` and `adjoint(G2,true)`.

For B2/C2 the letter changes: C2's fixed Cartan *is* the transpose of B2's
(`[[2,-1],[-2,2]]` vs `[[2,-2],[-1,2]]`), so `dual(SC(B2,true))` has exactly
the content of `adjoint(C2,false)` — the canonical dual IS directly
constructible across the type boundary.

## Pre-registered expectations for the frozen g2-v1 fixtures

Original (oracle) behavior, pin by pin; the landed Rust repair mirrors these
semantics (content+preference interning, cold-share, structural equality,
left-owner products), so **engines are expected to MATCH on every line**;
any DIFFERED is a genuine Rust defect signal:

- `WG_DUAL_OWNER` / `WG_REVERSE_OWNER`: **false** in both engines — the
  transposed dual equals neither adjoint numbering.  The frozen contract
  predicts `true` for both engines: a known miscalibration of the
  prediction, NOT a discrepancy; classify_capture records the
  prediction mismatch and the capture completes.
- `WG_DUAL_EQ`=true, `WG_DUAL_NEQ`=false, `WG_DUAL_MUL`=word `[]`;
  `WG_REVERSE_*` likewise; the product's owner is the left operand
  (`WG_REVERSE_MUL` root-datum check = true).  Cold duals share the primal's
  WeylGroup (W_ptr cold-share), so cross-owner relations are compatible
  even though the dual datum is the transposed object.
- Prewarmed fixture: the two numberings differ only in the preference flag
  (same matrices) → distinct interned objects → the 3 owner-side mismatches
  throw as predicted.  **But the dual-side triplet is vacuous by
  construction**: `wgn_target=adjoint(G2,false)` is NOT the canonical dual
  (Bourbaki ≠ transposed content), so prewarming it cannot fill the
  canonical dual's identity slot; `dual(wgn_true)` finds the slot empty and
  cold-shares the primal's group.  Expect `WGN_DUAL_PREWARM_EQ` to print
  **true** (no throw), `WGN_DUAL_PREWARM_NEQ` false, the product to succeed,
  and the AFTER markers to print `wgn_dual=wgn_target` → **false**.  Both
  engines, matching.  The frozen prediction of 6 mismatches is
  miscalibrated: 3 will fire.

## Follow-up gates this creates

- The genuine "independently prewarmed canonical dual rejects" witness for
  G2 requires prewarming the **transposed** content explicitly, e.g.
  `root_datum(id_mat(2), mat:[[2,-3],[-1,2]], false)` before
  `dual(SC(G2,true))`.  DRAFTED 2026-10-09 as
  `tests/math/generics/weyl_context_g2_prewarmed_transposed_dual.atlas`
  (sha `7ed86be0a46080a9d7954e1bd203a1f2c5dc1adfe28f1f93f805c7dae79a2730`;
  prefix `WGT_`, recovery 737): the explicit construction hits the canonical
  dual's exact content (roots I2, coroot columns = primal Cartan rows
  [2,-1] and [-3,2]), `WGT_SLOT` proves the hit (interned-pointer equality
  true), then the relation triplet must reject.  Provisional and unwired;
  it belongs to a later arc stage after the frozen g2-v1 capture confirms
  the vacuity prediction — the frozen payload stays unchanged, and its
  dual-side lines remain valid cold-share evidence either way.
- The B2/C2 drafts (`weyl_context_b2c2_*`) need no change: their
  both-numberings owner probes now carry the definite prediction
  `WB_DUAL_OWNER_FALSE`=true / `WB_DUAL_OWNER_TRUE`=false, and their
  prewarmed fixture is NOT vacuous (adjoint(C2,false) is the canonical
  dual of SC(B2,true)).
- The A1 v8 evidence remains consistent: at rank 1 the transpose is
  invisible (`[2]` = `[2]`), which is exactly why this semantics could not
  have been learned from A1.

## What the capture still decides

The reading predicts the oracle's outputs; the capture is the evidence.
If the oracle matches this reading everywhere and Rust matches the oracle,
the G2 semantic gate passes with the corrected predictions recorded.  Any
DIFFERED between engines — on any pin — is the Rust defect the arc exists
to catch; a miscalibrated-predictions note applies only to the three pins
named above.

## Part 2: W_elt retention and the inner-class eager dual (same day, later)

Two more upstream facts complete the source-level pre-registration for the
remaining drafted fixtures (same fetched file, atlas-types.w at the pin):

7. **A live W_elt keeps its datum alive.** `W_elt_value` (atlas-types.w:
   2459-2471) has members `shared_root_datum rd; const WeylGroup& W;
   WeylElt val;` — the `rd` strong pointer prevents interning expiry while
   any element lives, and `W` references `rd->W()`.  `root_datum(W_elt)`
   exists: `datum_from_W_elt_wrapper` (2552-2556) pushes `w->rd`.  So the
   sole-WeylElt lifetime fixture's expectations in the original are safe by
   construction: `WL_*_ROOT` round-trips print true, products and the cold
   dual through `dual(root_datum(w))` work, and a later equal construction
   reuses the same interned object because the store's weak slot is still
   alive.  Rust must match (its W_elt holds the interned handle); any
   expiry misbehavior is a genuine defect.
8. **`inner_class_value::build` eagerly builds and retains the canonical
   dual datum.**  Line 3306:
   `std::make_shared<inner_class_value>(srd, srd->dual(), tau, lo, ...)`;
   the class stores `shared_root_datum datum, dual_datum` (3234).  Per
   `dual()`'s cold-share, this also fills the canonical dual's W_ptr with
   the primal's group.  `dual(InnerClass)` (3322-3325) rebuilds via
   `build(dual_datum, dualDistinguished())`.  Expected original behavior for
   the WI fixture: `WI_ROUTE`=true, `WI_ROUTE_EQ`=true, `WI_ROUTE_MUL`=[],
   `WI_DUAL_OWNER_FALSE`=**false**, `WI_DUAL_OWNER_TRUE`=**false** (the
   canonical dual is the transposed object, per Part 1),
   `WI_IC_DUAL_ROUTE`=true (the inner-class-routed dual datum IS the
   explicit `dual(wi_rd)` interned object), `WI_IC_DUAL_EQ`=true,
   `WI_IC_DUAL_NEQ`=false, `WI_IC_DUAL_MUL`=[], `WI_IC_DUAL_WORD`=[0,1]|2.
   The gate's question for Rust is whether its inner-class path participates
   in the same interned dual-identity scheme; the original's answers are
   these.

With Parts 1-2, every drafted fixture now has source-derived oracle
expectations: G2 cold/prewarmed (Part 1), B2/C2 (Part 1 consequence),
reverse operands (fact 5: left-owner products), inner-class-dual (fact 8),
no-value (fact 5: guards throw before the no-value gate, so all five WV
discarded/void contexts throw in the original while `WV_NOVALUE_OK` prints
41), and sole-WeylElt lifetime (fact 7).

## Part 3: error-text verification (same day, later)

The prewarmed fixtures' expected error texts were checked against both the
pin and the frozen A1 goldens:

- `check_Weyl_word` (atlas-types.w:2493-2510) throws `Illegal Weyl word
  entry <v> (should be <rank>)`; the frozen A1 golden carries `Illegal Weyl
  word entry 1 (should be <1)`, so the rank-2 prediction `Illegal Weyl word
  entry 2 (should be <2)` matches the pattern.
- The same function reads each entry via `int_value::ulong_val()` BEFORE
  the rank check, and the A1 golden shows the negative-entry error is
  `Negative integer where unsigned is required` (thrown by the conversion,
  not by the word check).  Source inference alone would have predicted a
  huge unsigned value in the message (`ulong_val` wraps); the golden
  settles it — a reminder to trust captured goldens over plausible
  source readings when they differ.
- `Weyl group mismatch` verbatim at atlas-types.w:2581/2591/2605 (the
  eq/neq/prod wrappers), matching Part 1 fact 5.

## Part 4: no-value registration policy audit (same day, later)

The no-value side-effect question cuts both ways: not only must validations
fire at no-value (the WV fixture), builds that the original SKIPS must not
run either.  Auditing `typed.rs` registrations against the pin found a
candidate divergence family — comments claiming "so skip" on registrations
that actually BuildAndDrop (run the full build at no-value, dropping the
result and propagating its errors):

| builtin | typed.rs | Rust policy | upstream wrapper (pin) | correct policy |
|---|---|---|---|---|
| `dual(InnerClass->InnerClass)` | ~9861 | BuildAndDrop | pure skip (3424-3428) | Skip |
| `dual_real_form` | ~9897 | BuildAndDrop | index validation, then skip (3962-3975) | Validate |
| `dual_quasisplit_form` | ~9907 | BuildAndDrop | pure skip (3977-3983) | Skip |
| `central_fiber` | ~9918 | BuildAndDrop | pure skip (4083-4088) | Skip |

(`dual(RootDatum)` and `dual(Block)` are correctly `domain_builtin_skip`;
the upstream `dual` overload set is exactly RootDatum/InnerClass/Block —
there is no `dual(RealForm)` upstream, so the Rust dispatch's arm set is
right.)  The audit method (comment-scan for "so skip"/"precedes" followed
by a non-skip registration) also confirmed five Validate registrations are
consistent (`Weyl_orbit`, `scale_extended`, `involution`, `KGP_sum`,
`twisted_deform`).

Semantic observability of BuildAndDrop-vs-Skip requires the no-value build
to ERROR (e.g. a budget-exceeding `dual_inner_class`); otherwise the
difference is wasted work only (still relevant to the performance lane).
Per hard rules 3/7 this is a source-level CANDIDATE: no production edit
until an original-backed capture.  Probe fixture drafted:
`tests/math/generics/weyl_context_novalue_dual_family.atlas` (sha
`ba5bf6662a89f70a5af7cd4a73771e9a89d611c1afa2a0fd82d12d37ebef2f3e`, prefix
`XN_`, recovery 738) — discarded `dual(ic)` / `dual_quasisplit_form(ic)` /
`central_fiber` must print 41, the invalid-index `dual_real_form(ic,99)`
must still error (validation precedes the gate), and
`dual_real_form(ic,0)` must not.  Provisional and unwired; belongs to the
no-value arc stage, and if the capture confirms divergence the fix is the
one-word registration change (plus the regression).
