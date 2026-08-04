# Slice: common-block srm pool (`lookup_full_block`)

**Status**: planned (2026-08-04). **Goal**: make `block_Hasse`, `KL_block`,
`block_deform`, and the extended-block family match the oracle on
mid-block parameters (e.g. A2 x=3, C3/B3 mid params) — the last
architectural blocker of the port.

## Oracle references

- `gkmod/blocks.cpp:740-1030` — `Block::lookup_full_block`: the z_pool
  BFS and the info table (descents, lengths, x/y packet bookkeeping).
- `gkmod/repr.cpp:2660-2780` — `common_context`: constructor,
  `status(s,x)`, `cross`, `up_Cayley`, `down_Cayley`, `is_parity`.
- `gkmod/repr.h:300-360` — `StandardReprMod` (x, gamma_lambda mod X*),
  `gamma_lambda_rho`, `mod_reduce`/`build`.
- `sources/structure/rootdata.cpp:144-219` — `roots_at_level` (needed for
  `SubSystem`'s simple basis if we rebuild subsystem roots).

## Why the current fibred closure is wrong

The Rust `BlockGraph` is the full fibred product (z = (x_primal, x_dual)),
and `common_block_members` closes under the fibred cross/Cayley — but the
oracle's `lookup_full_block` works in the `common_context` of the
parameter: `sub = SubSystem(rd, integrality_simples(rd, gamma))`, and
every status/transform goes through `conj = sub.to_simple(s)` (a word in
the FULL system) first. Concretely: A2 x=3 (su(2,1)) — the oracle common
block is `{x=3}` only, while the fibred closure pulls in x=1, x=2. The
previous simplified srm layer (604cc83, reverted) over-approximated
because it used the primal KGB x + real-KGB status instead of the common
subsystem view; it also hit sr_gamma integrality failures because each
block element needs its OWN lambda_rho = gamma - gamma_lambda_rho(srm)
(repr.h:329-331), not the starting parameter's.

## Progress 2026-08-04 (A2 x=3 mechanism verified, reverted)

A working srm prototype was built and verified on A2 x=3:
- `integrality_simples` via the Cartan pairing `<gamma, alpha_s^vee> =
  sum_i gamma_i * C[i][s]` (the Rust simple coroots are the STANDARD basis
  vectors; pairing needs the Cartan columns).
- the srm closure selected by the primal KGB status (no conj pre-cross
  for the whole-datum case), with each block element's own
  `lambda_rho = gamma - gamma_lambda - rho` (repr.h:329-331).
- A2 x=3 (su(2,1)): members = {x=3}, byte-identical to the oracle.
- BUT A2 x=0 must be the whole 6-element block, and the prototype only
  reached {0,3,4,5}: the oracle's `lookup_full_block` passes a
  `block_modifier bm` whose integral subsystem (`bm.int_sys_nr`, the
  image of a subsystem under `bm.w`) — NOT simply `integrality_simples(gamma)`
  — governs which generators the z_pool may use (blocks.cpp:745,
  common_context(rc, bm)). Porting the bm/int_sys selection is the
  remaining sub-step before the prototype can replace the fibred closure.

## Performance slice: compact Weyl layer (2026-08-04, partial)

- New `weyl_transducer.rs`: the du Cloux/van Leeuwen parabolic-subquotient
  representation (Transducer build weyl.cpp:100-547, O(length)
  multiplication). Tests: group orders A2/B2/G2/A3/D4, inverse group law,
  compact-vs-matrix action equality, twisted-involution equivalence on
  A1xA1/A2 — all pass.
- `longest_action` now walks 2rho -> -2rho (O(36) for E6 instead of |W|).
- `WeylAction` datum is Arc; compose_matrices is i64-accumulated;
  enumerate_actions uses precomputed reflections. E6 W_graph probe:
  13.7s -> 6.5s (C++ -O3: 0.028s; the remaining gap is the
  twisted-involution classification, which still enumerates matrices).
- NOT WIRED: compact twisted-involution enumeration. Debugging found the
  partition (enumeration members) vs classification canonicalized
  representatives use DIFFERENT image-permutations, so
  `class_by_permutation` misses canonical keys. Next round: canonicalize
  each candidate in the partition (or look up by orbit membership) so
  `class_of` always hits. This unlocks E6 classification ~1.1s -> ~0.05s.

## Slice steps (verify each)

1. **`integrality_simples(rd, gamma)`** — the simple generators s with
   `<gamma, alpha_s^vee>` integral (rootdata.cpp `SubSystem::integral`).
   Verify: A2 gamma=[0,0]/1 → all simples; gamma=[1,0]/2 → subset.
2. **`SubSystem`** — wrap a subset of simple generators; `to_simple(s)`
   = the reflection word of the integral simple s in the full system;
   `simple(s)` = the full-system simple RootNbr. For the quasisplit
   gamma=0 case the subsystem is the whole datum (to_simple = identity) —
   the fixture cases below exercise exactly this, so step 1-2 can be a
   thin layer.
3. **`common_context::status(s, x)`** (repr.cpp:2676-2684) —
   `conj_x = kgb.cross(to_simple(s), x)`, then the status of
   `sub.simple(s)` at `conj_x`; the second bool = isDoubleCayleyImage for
   real / isDescent for complex / type-1 test for imaginary noncompact.
4. **StandardReprMod transforms** on the primal KGB x:
   - `cross` (repr.cpp:2694-2707): x' = cross(refl, x);
     gamma_lambda -= root_sum(pos_to_neg(refl) ∩ real_roots(x));
     simple_reflect(s, gamma_lambda); build (real_unique).
   - `down_Cayley` (repr.cpp:2724-2746) / `up_Cayley` (repr.cpp:2748-2775)
     with the real-flip corrections.
5. **z_pool BFS** (blocks.cpp:740-1030): the srm closure with the
   x/y packet bookkeeping; the info table's descents come from
   `common_context::status` (complex/imaginary) or `is_parity` (real);
   `srm_hash.match` identifies the block element (x, y) with the srm.
6. **Rewire `block_Hasse`** to the srm pool and give each member its own
   lambda_rho = gamma - gamma_lambda_rho(srm) (repr.h:329). Verify:
   A2 x=3 → `{x=3}` (1x1 hasse); C3/B3 mid params; then D5/D6 full
   blocks. Run the whole `block_hasse` fixture suite.
7. **Then unlock**: `KL_block`, `block_deform`, `partial_block` (full
   common semantics), `extended_block` family, `K_type_pol_extended`,
   `twisted_KL_sum_at_s` / `twisted_deform` / `twisted_full_deform`.

## Fixtures to add (freeze first, per the delivery loop)

- `domain/block_hasse` already covers A2 x=0/x=3, B2, C3, B3, D5, E6, A4.
  The srm pool should make the existing fixture's mid-block rows exact;
  extend with C3/B3 mid-block params and D6.
- `domain/kl_block.atlas` (Param → Mat of KL polys) — new.
- `domain/block_deform.atlas` (Param → KTypePol) — new.

## Deliverables

- `crates/atlas-real-group/src/common_context.rs` (SubSystem + status +
  StandardReprMod transforms) or an equivalent owned module.
- `crates/atlas-core/src/domain_builtins.rs`: `common_block_members_srm`
  (replace the fibred closure), per-member lambda_rho.
- `KL_block`, `block_deform` arms + typed.rs registration.
- Fixture + HPC differential (reference_capture + pipeline_swap_diff),
  meta upgrade to `verified_hpc`, ledger + HANDOFF update.
