# Locator slice — step 2/3/4 integration brief (2026-08-19)

Step 1 (in flight, agent-64) delivers `InnerClass::int_item` canonicalization
(upstream `innerclass.cpp:1116-1182`) and a `BlockLocator { int_sys, w,
simp_int, simple_pi }` value with canonical interning (upstream `locator`,
`repr.h:484-491`). This brief fixes the follow-up integration order; do not
reopen the design.

## Frozen anchors (differential gates for step 4)

- `tests/fixtures/domain/common_block_locator.atlas` — A2 split SL(3,R):
  `param(KGB(rf,3),[0,0],[2,1]/2)` installs a rank-one block;
  `param(KGB(rf,0),[0,0],[-2,-1]/2)` collides via a Weyl-conjugate integral
  subsystem → oracle prints `as transformed by <1>` and transported rows for
  `print_common_block`/`partial_block`/`block_Hasse`/`W_graph`/`KL_sum_at_s`.
  Capture 3574723, meta `rust_status=not_implemented`, NOT registered in
  `FIXTURE_PLANS`.
- `tests/fixtures/domain/common_block_simple_pi.atlas` — A3 SL(4,R) rank-two:
  `[0,1,1]/2` installs, `[1,1,0]/2` collides with permuted simples → oracle
  prints `as transformed by <0.2>, simple reflections permuted (0->1,1->0)`.
  Capture 3574819, same unregistered/pending state.
- Register both plans ONLY in the step-4 commit that makes them pass.

## Step 2 — block_modifier arithmetic (atlas-real-group, RepContext)

Port, in this order, against `repr.cpp`:

- `Rep_context::transform<forwards>(w, srm)` — Weyl action on
  StandardReprMod (both directions; upstream template at repr.cpp, used by
  `make_relative_to` and `sr`).
- `Rep_context::shift(amount, srm)` — repr.cpp:347-351: `gamlam += amount`
  then `involution_table().real_unique(kgb().inv_nr(x), gamlam)`.
- `Rep_context::make_relative_to(loc, srm0, bm, srm1)` — repr.cpp:338-350:
  `bm.w *= W.inverse(loc.w)`; `compose(bm.simple_pi, Permutation(loc.simple_pi,-1))`;
  `transform<true>(bm.w, srm1)`; `bm.shift =
  make_diff_integral_orthogonal(srm1.gamma_lambda(), srm0)`.
- `Rep_context::sr(srm, bm, gamma)` — repr.cpp:815-823: apply `bm.shift`
  first, then `transform<false>(bm.w, srm)`, then `sr_gamma(x_part,
  gamma.integer_diff(gamma_lambda_rho(srm)), gamma)`.
- `block_modifier` = locator + `RatWeight shift` (repr.h:493-499), with
  `clear(block_rank, datum_rank)` and the trivial-from-block constructor.

Unit tests: identity locator ⇒ `sr(srm,bm,gamma) == sr(srm,gamma)`; the A2
anchor pair must satisfy `make_relative_to` round-trip (shift then transform
srm0 lands on srm1 up to root translation); `simple_pi` compose order matches
upstream (`Permutation(loc.simple_pi,-1)` = inverse convention).

## Step 3 — canonical keys + attitude gates FIRST

Wire canonical `ReducedParamKey`s into `RepTable::lookup` /
`lookup_full_block` (co_reduce-row registration as already documented).
BEFORE any consumer may observe a stored block, add loud attitude gates
(reject nonidentity `block_modifier`) to every consumer that currently
assumes identity attitude:

- `KL_column` (domain_builtins.rs ~13833), `KL_block` (~13914),
- `print_block` / `print_common_block` / `block(Param)`,
- `kl_sum_at_s_terms` (KL_sum_at_s / KL_sum_at_s_to_height).

Rationale: without the gates, canonicalization lands silently wrong results.

## Step 4 — transported consumers + headers + gate release

- `singular_flags(bm)` and `located_row_parameter` go through `sr(srm,bm,·)`.
- `print_common_block` prints `as transformed by <w>` and, when nontrivial,
  `simple reflections permuted (i->j,...)` (upstream print format in
  repr.cpp `common_block` printing).
- Release the step-3 gates consumer by consumer as each is transported.
- Register both locator fixtures in `FIXTURE_PLANS`, run the differential,
  upgrade both metas to `verified_hpc`.

## Step 5 (later, not this slice)

Cross-block partial swallow and ext-block induced (`simple_pi` on extended
blocks).

## Known defect folded into this slice

Current Rust `print_common_block` on the A2 SL(3,R) family already differs
from the oracle at IDENTITY attitude (gamma-lambda shifted by [0,1] on rows
0/2 of the locator fixture): the identity-attitude shift handling is the
`bm.shift` field of `block_modifier`, i.e. exactly step 2's
`make_diff_integral_orthogonal`. No separate fix; verify the defect closes
when step 4 transports `print_common_block`.

## Current-Rust divergence map on the extended simple_pi anchor (2026-08-19)

Against the extended `common_block_simple_pi` fixture (capture 3574854),
current Rust at identity attitude:

- `print_common_block(q)`: wrong header (`<>` instead of
  `<0.2>, simple reflections permuted (0->1,1->0)`); gamma-lambda rows
  shifted (e.g. row 0 `[-3,0,3]/4` vs oracle `[-1,0,1]/4`).
- `partial_block(q)`: same parameter SET, wrong row ORDER (x=10/x=2 rows
  swapped at positions 2-3; the two x=12 rows swapped).
- `W_graph(q)`: same cell decomposition, wrong cell LISTING order
  (`[1]`/`[0]` cells swapped at positions 2-3).
- `block_Hasse(q)` matrix and `KL_sum_at_s(q)` value already match at
  identity attitude (order-insensitive outputs).

Implication for step 4: transported consumers must reproduce the CANONICAL
row ordering of the stored block, not a freshly built block's ordering —
the row order itself is oracle-visible.
