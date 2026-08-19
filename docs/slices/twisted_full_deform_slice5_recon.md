# Slice 5 recon: `twisted_full_deform` recursion at proper-subsystem reducibility points

Recon by agent-92, 2026-08-19, working tree @ de2ee7b (slices 1-3 landed,
slice 4 in flight). Companion to docs/slices/twisted_ext_proper_workorder.md
slice 5; this document is the implementation brief.

## Current Rust state

- `twisted_deformation_with_cancel` — `crates/atlas-real-group/src/deform.rs:817-936`;
  the loud NYI is at **887-891** (`ProperSubsystem` arm →
  `StructureError::NotYetImplemented`). `Singleton` contributes nothing;
  `Full` (892-929) calls the lookup closure, then `singular_orbits_at` +
  `twisted_deformation_terms`, recursing per term with flip-parity sign
  (repr.cpp:2637-2646).
- Lookup closure type today (deform.rs:805,820):
  `FnMut(&StandardRepr) -> Result<(BlockGraph, ExtBlock, usize), StructureError>`.
- `twisted_reducibility_lookup` — `crates/atlas-core/src/domain_builtins.rs:8111-8193`:
  rebuilds the entire dual pipeline per call, full block only, no caching;
  invoked only from the `Full` arm.
- NOTE (orchestrator probe, 2026-08-19): on the anchor
  `param(KGB(rfb,5),[1,1],[1,0]/1)`, current Rust does NOT hit the NYI —
  it silently prints four `(1±s)` terms including [13]-block rows where the
  oracle prints exactly `1* K_type(x=2, lambda=[0,4]/1) [12]` and
  `1* K_type(x=3, ...) [12]`. So the classifier routes this input to the
  Full arm today; slice 5 must re-check where the proper-subsystem gamma
  classification is bypassed in the recursion path (the recursion terms
  live at [1,0]/2 = pb's rank-1 proper-subsystem gamma).

## What slice 3 put in place (reusable)

`with_integral_block`'s `ProperSubsystem` arm (domain_builtins.rs:8226-8272):
`RepTable::lookup_full_block` → `LocatedBlock`; identity-attitude gate
(8238-8243); `CommonContext::integral`; `tuned_partial_ext_block`
(3915-3942); singular fold `ctxt.singular_flags(gamma)` →
`eblock.singular_orbits`; delivery as `KlSumParent::Partial(&block)`
(deform.rs:471-532; per-row `sr` reconstructs lambda_rho from gamma_lambda,
= blocks.cpp:1260-1264).

Slice 5 additionally needs:

- **Partial `lookup`, not `lookup_full_block`** — upstream
  `twisted_deformation` uses `Rep_table::lookup` (repr.cpp:2605 →
  1796-1822, interval-below partial block). All block access in
  `twisted_deformation_terms` is at rows ≤ y, so interval-below suffices.
  `RepTable::lookup` is pub at rep_table.rs:1245.
- **Owned parent sum type** — `DeformParent { Full { block: BlockGraph,
  lambda_rho: Weight }, Partial(PartialBlock) }` with a
  `as_kl_sum_parent()` view (KlSumParent borrows; the closure returns
  owned data).
- `twisted_deformation_terms` generalized to `KlSumParent` — slice 4 is
  landing exactly this; slice 5 consumes, does not redo.

## Upstream recursion contract

- Wrapper `twisted_full_deform_wrapper` (atlas-types.w:8229-8251): gates
  `test_standard` + distinguished-delta-fix only (no `test_final`);
  `extended_finalise` expands to finals with per-final flips. Rust mirrors
  in `compute_twisted_full_deform` (domain_builtins.rs:2396-2448).
- `Rep_table::twisted_deformation` (repr.cpp:2552-2653): reducibility
  points rp (ported at rep_context.rs:1718); shrink-wrap to rp.back()
  (2562-2572); alcove-pool formula memoization (2574-2585) NOT ported
  (cost only); per point in reverse: `zi = scaled_extended_finalise`, then
  partial `lookup` with `block_modifier`, `extended_block(bm)` cached per
  (w, shift), singular-orbits fold over `bm.simp_int` (2617-2633; trivial
  bm reduces to `simple_singular_flags` + `eblock.singular_orbits` =
  slice-3 arm), `twisted_deformation_terms` (repr.cpp:2425-2520) building
  terms via `block.sr(eblock.z(f), bm, gamma)`, recursing with combined
  flip parity.

## Merge/swallow verdict (open question b resolved)

- Same-attitude partial merge IS ported: `State::commit_partial`
  (rep_table.rs:491-522), `RepTable::lookup` (748-846) with
  add_block_below/append/swallow-as-rebuild. `common_block::swallow`
  Hasse/KL pilfering (blocks.cpp:1416-1470, ext transfer `#if 0`'d
  upstream) deliberately replaced by on-demand recompute. **Slice 5 does
  not port `swallow`.**
- Surviving NYI is only relative-attitude merge (rep_table.rs:780-787).
- For the anchor fixture: γ=[1,0]/1 integral; rp ∋ 1/2; recursion gammas
  [1,0]/2, [1,0]/4, … share the same rank-1 integral system and (both
  dominant, trivial word) the same locator ⇒ merges, if any, are the
  ported same-attitude union rebuild. `ReducedParamKey` embeds `int_sys`
  so cross-subsystem overlap never happens. Caveats: (a) a recursion term
  landing in a different alcove with nontrivial dominant word would hit
  the identity-attitude gate or merge NYI — loud by design, acceptable;
  (b) formula memoization stays unported (cost only; whole-result
  `DeformationCache` at domain_builtins.rs:232 exists).

## Minimal change set (in order)

1. (Prereq, slice 4) `twisted_deformation_terms` → `parent: &KlSumParent`.
2. deform.rs: owned `DeformParent` enum + `as_kl_sum_parent`.
3. deform.rs `twisted_deformation_with_cancel`: closure →
   `FnMut(&StandardRepr) -> Result<(DeformParent, ExtBlock, usize, RankFlags),
   StructureError>` (singular orbits computed closure-side — only the
   closure has the `CommonContext` in the partial case); `ProperSubsystem`
   arm runs the Full body instead of erroring. Update recursion tests at
   1266-1300 (currently assert the NYI).
4. domain_builtins.rs: split `twisted_reducibility_lookup` into a
   dispatcher on `integral_block_scope`: Full → current body wrapped in
   `DeformParent::Full` + `singular_orbits_at`; ProperSubsystem →
   `context.rep.lookup(zi)` (partial, repr.cpp:2605) → identity-attitude
   gate (reuse 8238-8243 wording) → `CommonContext::integral` →
   `tuned_partial_ext_block` → singular fold. Factor the shared prologue
   of with_integral_block's arm (8233-8264) into one helper parameterized
   by the RepTable entry point.
5. `compute_twisted_full_deform` (2416-2417): closure becomes the
   dispatcher — signature plumbing only.

Slice-4 collision surface: the `twisted_deform` dispatch closure at
domain_builtins.rs:16661-16694 currently rejects `KlSumParent::Partial`
(16669-16673); slice 4 fills it, forcing step 1 and likely a partial-aware
`singular_orbits` helper. If slice 4 lands a different shape, only the
step-3 closure return type is affected.

## Fixture plan

Accepted — `tests/fixtures/domain/twisted_full_deform_proper.atlas`:

- `p := param(KGB(rfb,5),[1,1],[1,0]/1)`; `twisted_full_deform(p)` — the
  anchor. ORACLE-PROBED 2026-08-19: accepted, prints exactly
  `1* K_type(x=2, lambda=[0,4]/1) [12]` and `1* K_type(x=3, lambda=[0,4]/1) [12]`.
- `twisted_full_deform(pb)` — pb (γ=[1,0]/2) itself: recursion entirely in
  partial blocks.
- `twisted_full_deform(param(KGB(rfb,4),[1,1],[1,0]/1))` — probe first.
- A2 identity-class line only if the oracle shows a proper-subsystem
  reducibility point for an integral-γ analog of pa (probe).

Rejected — `twisted_full_deform_proper_rejected.atlas`:

- `twisted_full_deform(c10)` with `c10 := param(KGB(rfb,10),[1,1],[1,0]/2)`
  (expect "Parameter not fixed by inner class involution"; probe wording).
- A non-standard param → "Cannot compute full twisted deformation" (probe).
- Arity/type errors only if not pinned by
  timed_twisted_full_deform_validation_order_rejected.atlas.

Ambiguity: work order says "partial common block + row"; slices 3-4 use
`lookup_full_block`. Recon recommends partial `RepTable::lookup` (matches
repr.cpp:2605, smaller blocks, memoized reuse); reusing the full shape is
acceptable if slice 4 standardizes on it.
