# Next Weyl-owner semantic captures: design drafts — 2026-10-03

Status: **design drafts only**.  Nothing here is a live fixture, golden or
HPC input.  Every sketch must first go through an original-backed discovery
capture on HPC before any expectation is frozen.  The A1 line's own gate
(AFTER-v1, commit `342a0511`) is frozen but not yet submitted — the
SecureLink tunnel is down; see `docs/HANDOFF.md` 2026-10-03.

## Why A1 is not enough

The accepted v8 evidence and the BEFORE-v4 tests-first record cover A1 only.
The v8 inspection already lists the gaps; this slice turns them into concrete
capture plans.  A1 cannot prove:

1. **Nontrivial cross-coordinate products.**  Its only cross-dual product is
   `s0*s0 = 1`; a wrong generator renumbering or a direct composition of
   foreign root permutations is invisible at rank 1.
2. **Interface-order asymmetry.**  G2's canonical dual exchanges long and
   short roots; whether the dual's generator *order* follows the primal
   numbering is exactly what must be observed, not assumed.
3. **Dual pairs whose type letter changes** (B2 ↔ C2).
4. **Reverse operand order** of a cross-owner product (`w_dual * w_primal`,
   not only `w_primal * w_dual`).
5. **Inner-class construction**: the original's `inner_class_value::build`
   eagerly evaluates the canonical `dual()` and retains both datum owners;
   the current Rust `InnerClassContext` retains only the primal handle.
   Fixing explicit `dual(RootDatum)` alone does not cover this path.
6. **`no_value` relations**: the original's WeylGroup-identity check fires
   even when the relation's value is discarded.
7. **Sole-WeylElt lifetime**: a datum referenced only through a live
   `WeylElt` must stay alive (weak interning must not drop it).

## Draft case sketches (provisional Atlas source, no goldens)

Idioms follow the frozen A1 fixtures: core-only (no script loading), marker
lines via `prints`, an error followed by a recovery marker after every
expected rejection, both engines as fresh processes with complete
stdout/stderr/exit capture.

### G2 asymmetric interface-order witness

```
set g_sc=simply_connected(Lie_type("G2"),true)
set g_a=g_sc dual        { placeholder: exact dual() call form per contract }
set w01=W_elt(g_sc,[0,1])
set w10=W_elt(g_sc,[1,0])
prints("G2_ORDER|",word(w01*w10),"|",word(w10*w01))   { must differ }
set w01d=W_elt(g_a,[0,1])
prints("G2_CROSS|",word(w01*w01d),"|",word(w01d*w01))  { both orders }
prints("G2_CROSS_EQ|",w01*w01d = w01d*w01)
{ invalid words and recovery markers as in the A1 prewarmed fixture }
```

Discovery questions: does the original accept both cross-owner products on
cold duals; which generator ordering does the canonical dual present; do the
root-permutation results agree with replay-in-left-system semantics.

### B2/C2 letter-changing dual

```
set b2=simply_connected(Lie_type("B2"),true)
set c2=dual(b2)
prints("B2_DUAL_TYPE|",c2=adjoint(Lie_type("C2"),false))  { expectation TBD }
set wb=W_elt(b2,[0,1])  set wc=W_elt(c2,[0,1])
prints("BC_CROSS|",word(wb*wc),"|",word(wc*wb))
```

### Reverse operand orders (A1 control extension)

Extend the A1 cold/prewarmed histories with `w_dual * w_primal` (only the
forward order is covered today) so both multiplication directions are on
record for a pair whose `=` is true and for a pair whose `*` rejects.

### Inner-class dual retention

Construct a real form (inner class) over a small datum, then compare Weyl
elements built from the form's datum and from an explicit `dual` of it, in
both cold and prewarmed histories.  The original retains both owners from
`inner_class_value::build`; observe whether relations/products across them
match the explicit-`dual` behavior.

### no_value relation check

Evaluate an incompatible `=`/`*` in a discarded position (statement context
or a dead branch) and confirm the original still raises the Weyl-group
mismatch rather than skipping the check at no-value level.

### Sole-WeylElt lifetime

```
set w=W_elt(simply_connected(Lie_type("A1"),true),[0])   { owner never separately bound }
prints("LIFE|",word(w),"|",root_permutation(w))
prints("LIFE_EQ|",root_datum(w)=simply_connected(Lie_type("A1"),true))
```

## Ordering and gates

Each case needs its own original-backed capture, its own tests-first BEFORE
(unchanged Rust must fail the new regressions where the semantic fix is
required), then the AFTER on the repaired source.  None of this is submitted
while AFTER-v1 is pending; the queue discipline and the 10-job ceiling are
unchanged.  Work-count and cache A/B designs stay out of scope until every
semantic gate above has its own accepted AFTER.
