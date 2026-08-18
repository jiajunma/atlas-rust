# Work order: proper-subsystem twisted/ext recursion

Recon completed 2026-08-18 (agent-68). Upstream pinned at rev 4d3e9449
(`/Users/hoxide/mycodes/atlasofliegroups/sources`). This document is the
authoritative slice plan; do not re-derive from scratch.

## Current Rust gates

Gate infrastructure (shared):

- `IntegralBlockScope::{Singleton, Full, ProperSubsystem}` +
  `integral_block_scope()` — `crates/atlas-real-group/src/deform.rs:160-209`.
  Classifies gamma by integral coroot pairings.
- `proper_subsystem_diagnostic()` —
  `crates/atlas-core/src/domain_builtins.rs:7694-7704`. Error text:
  `common block on a proper integral subsystem is not yet implemented`.
- `with_integral_block()` — `domain_builtins.rs:7824-7846`. Singleton →
  short-circuit; ProperSubsystem → loud NYI at 7835; Full → full block +
  `build_ext_block` + `twisted_block_index`.

| Builtin | Dispatch site | Gate behavior |
|---|---|---|
| `twisted_deform` | `domain_builtins.rs:16159-16210` | Loud NYI on proper subsystem; Singleton → empty terms; Full via `twisted_deformation_terms` (deform.rs:349) |
| `twisted_KL_sum_at_s` (1- and 2-arg) | `domain_builtins.rs:16220-16285` | Loud NYI; Singleton → `1*p`; Full via `twisted_kl_column_at_s` (deform.rs:507) / `twisted_kl_sum` (deform.rs:462) |
| `twisted_full_deform` (both overloads) | `domain_builtins.rs:16294-16380` → `compute_twisted_full_deform` (2396-2430) | Loud NYI at each reducibility point with proper-subsystem gamma (deform.rs:809-813); lookup closure `twisted_reducibility_lookup` (7734-7816) rebuilds the full block every call |
| `extended_block` | `domain_builtins.rs:14777-14933` | `gamma_is_integral` gate at 14829-14836 rejects all non-integral gamma |
| `raw_ext_KL` | branch 14935-15017 | same gate |
| `partial_extended_KL_block` | branch 15019-… | same gate; fiber-submatrix comment at 15057-15063 documents the full-block approximation |

Boundary (other slices, do not touch here):

- Generator-attitude gates at `domain_builtins.rs:14227/14573/14713` —
  locator slice.
- `common_block_rows` NYI at `domain_builtins.rs:9431-9435` — the separate
  non-integral common-block gap.
- Cross-block partial merge NYI at `rep_table.rs:474-478`.
- Silent deviation (no gate): ordinary `full_deform` reducibility-point
  recursion computes on the full block with no scope check
  (`full_deformation_terms`, `domain_builtins.rs:2282-2321`).

## Upstream anchors (rev 4d3e9449)

The proper-subsystem path is never special-cased upstream: everything flows
through `common_context` on gamma's integral subsystem plus a
`block_modifier`.

- `common_context` — `gkmod/repr.h:647-674`, ctor `gkmod/repr.cpp:2666-2670`
  (`simp_int = integrality_simples(rd, gamma)`, `SubSystem sub(rd, simp_int)`);
  status/cross with transport words `repr.cpp:2679-2700+`.
- `locator` / `block_modifier` — `gkmod/repr.h:485-499`.
- `common_block` subsystem ctors — `gkmod/blocks.cpp:733-1081`, `1086-1248`;
  `singular(bm, gamma)` at `blocks.cpp:711-721`; `fold_orbits` at
  `blocks.cpp:1288`.
- `common_block::extended_block` — `blocks.cpp:1305-1310` (trivial-bm) and
  `1344-1358` (bm-aware, cached per `(w, shift)`).
- `ext_block::ext_block(common_block, bm, delta, pol_hash)` —
  `gkmod/ext_block.cpp:618-668`: fixed points via `transformed_twisted`
  (`ext_block.cpp:597-616`), orbit permutation by `bm.simple_pi` via
  `induced` and `tune_signs` (`ext_block.cpp:1707-1876`).
- Wrappers: `extended_block_wrapper` `atlas-types.w:7366-7431`;
  `raw_ext_KL_wrapper` `atlas-types.w:8682-8728`;
  `extended_KL_block_wrapper` `atlas-types.w:7445-7468` →
  `gkmod/ext_kl.cpp:939-1018`; `twisted_deform_wrapper`
  `atlas-types.w:8120-8150` → `Rep_table::twisted_deformation_terms`
  `repr.cpp:2426-2520`; `twisted_full_deform_wrapper` `atlas-types.w:8229-8251`
  → `repr.cpp:2552-2653` (per-point `lookup` 2606, `extended_block` 2614,
  singular-orbits fold 2615-2631); `twisted_KL_sum_at_s_wrapper`
  `atlas-types.w:8370-8382` → `repr.cpp:2371-2423`;
  `external_twisted_KL_sum_at_s_wrapper` `atlas-types.w:8420-8431` →
  `repr.cpp:2304-2350`.

## Reusable Rust machinery

- `IntegralSubsystem` (`crates/atlas-real-group/src/partial_block.rs:116-219`)
  — faithful `SubSystem` port; `integral()` at 141, `full()` at 133.
- `CommonContext::integral` (`partial_block.rs:471-479`) with transported
  `status`/`cross`/`up_cayley`/`down_cayley` (539-726) — the
  `repr.cpp:2666-2700` surface. `singular_flags` at `partial_block.rs:727`.
- `PartialBlock` (`partial_block.rs:903+`): `build_full` (1320), partial
  `build` (1383), block surface (1608-1666). Caveat: `y(z)` (1631) is a
  synthetic subsystem y-count, NOT a dual-KGB element — the ext-block
  fixed-point test must port `transformed_twisted`'s x + gamma_lambda form
  (`PartialBlock::lookup`, 1619, supports it).
- `RepTable::lookup_full_block` (`rep_table.rs:706-772`, wrapper 1053)
  already materializes proper-subsystem full common blocks with identity
  `generator_attitude`. Call-pattern precedent: `print_block(Param)` at
  `domain_builtins.rs:9904-9921`.
- `ExtBlock` internals reusable unchanged once a partial-parent ctor exists:
  `complete_construction` (`ext_block.rs:714`), `fold_orbits` (361),
  `induced` (1386), `tune_signs` (1190) via the `StarOracle` trait — a
  PartialBlock-backed oracle variant of `ExtParamOracle`
  (`ext_param.rs:2278-2324`); `star` at `ext_param.rs:1210` already takes
  arbitrary root IDs.
- `ExtKlTable`/`condense` (ext_kl.rs) operate on any `ExtBlock`.
- deform.rs algorithm bodies are typed on `&BlockGraph`; the proper case
  needs a parent-block abstraction or a PartialBlock twin, plus a
  partial-aware `singular_orbits_at` (248-254, currently trivial-bm).

**Single shared prerequisite**: an `ExtBlock` constructor over `PartialBlock`
(identity-attitude `transformed_twisted` + subsystem-Cartan `fold_orbits` +
partial `StarOracle` for `tune_signs`). Every slice needs it.

## Slice plan (in order, each independently landable)

1. **`extended_block` on proper integral subsystems** — smallest: no KL
   table, distinguished delta, no recursion, identity attitude. Replaces the
   14829 gate for the proper case.
   Fixtures: B2 `real_form(ic_identity, 2)`,
   `pb := param(KGB(rfb,5),[1,1],[1,0]/2)` (already pinned for `block_Hasse`
   in `tests/fixtures/domain/block_hasse_param_proper.atlas`); A2 identity
   inner class form 1 `param(KGB(rf,0),[0,0],[1,0]/2)`; C2 split form from
   `ext_block.atlas:58-64` with fractional nu. Keep existing integral cases
   in `ext_block.atlas` as regression.
2. **`raw_ext_KL` + `partial_extended_KL_block` on proper subsystems** —
   adds `ExtKlTable` + `condense` over the partial-parent ext block; the
   partial branch also needs `B.singular(gamma)` over subsystem generators
   (`CommonContext::singular_flags`, `partial_block.rs:727`) replacing the
   hand-rolled loop at `domain_builtins.rs:15024-15035`. Same fixtures, all
   three builtins.
3. **`twisted_KL_sum_at_s` (both overloads) on proper-subsystem input
   gamma** — new `ProperSubsystem` arm in `with_integral_block` (7835) via
   `RepTable::lookup_full_block` + the partial `ExtBlock`; generalize
   `twisted_kl_sum`/`twisted_kl_column_at_s` to the partial parent.
   Fixtures: final delta-fixed parameters with proper-subsystem gamma —
   B2 form 2 `pb`, A2 identity-class `nu=[1,0]/2` variants, plus the
   distinguished-vs-external delta pair from `twisted_family.atlas:28-29`.
4. **`twisted_deform` on proper-subsystem input gamma** — same arm, plus
   `twisted_deformation_terms` and a partial-aware `singular_orbits_at`
   (identity-attitude `simp_int` fold). Fixtures as in 3.
5. **`twisted_full_deform` recursion at proper-subsystem reducibility
   points** (`deform.rs:809-813`) — deepest: `twisted_reducibility_lookup`
   (7734-7816) becomes RepTable-backed returning a partial common block +
   row. Fixture: B2 form 2 `param(KGB(rfb,5),[1,1],[1,0]/1)` under
   `twisted_full_deform` (its 1/2-scaled reducibility point is exactly `pb`'s
   gamma); verify acceptance against the oracle first.

Dependency graph: 1 → 2, 1 → 3 → 4 → 5. All slices stay inside identity
generator attitude; non-identity `simple_pi`/`w` remains the locator slice's
domain and keeps the loud NYI.

## Open questions (flagged by recon, decide when slicing)

- (a) Rank-0 non-integral `extended_block`/`raw_ext_KL`: upstream handles it
  uniformly via common_context (size-1 block); the registry lists
  "non-integral common blocks" as a separate gap, so it was scoped out of
  slice 1. Could ride along if cheap.
- (b) Slice 5 intersects the RepTable memoization/`swallow` machinery
  (`blocks.cpp:1379-1470`), only partially ported — may force the
  cross-block partial-merge NYI (`rep_table.rs:474-478`) to surface early.
