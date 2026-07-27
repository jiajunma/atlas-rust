# Weyl element substrate design

## Approved scope

This stage adds the word-level Weyl substrate the KGB construction needs
and the crate's matrix-based `WeylAction` deliberately does not provide:
elements with O(1) length and descent queries, multiplication, inverses,
twisted conjugation, and on-demand reduced words. It is stage (a) of the
KGB map recorded in the strong-real era: the InvolutionTable, Tits
operations, seed, and generation are later stages. No new mathematics
enters — this is pure Coxeter combinatorics — but the representation
decision is numbering-relevant and made here.

Scale, corrected by review: `WeylElement` is the CONSTRUCTION CURRENCY of
stages (b) and (c), not of KGB elements. Per kgb.h:63-70, 88-92, a KGB
element persists only an involution index and a torus part; persistent
`WeylElement` storage is therefore bounded by the twisted-involution count
(about 2.0e5 for split E8, always at most the 320,206 KGB size), and the
involution table may retain only derived involution data, keeping full
permutation elements transient during generation.

## Atlas construction and the representation decision

Citations are to `~/mycodes/atlasofliegroups`, master `4d3e9449`.

Atlas represents Weyl elements through Fokko du Cloux's transducer tables
(weyl.cpp:495-527 builds an INTERNAL generator renumbering by Dynkin
component and position in all types, additionally reversing B/C/D
components), with `length_change` reported by the multiplication
primitives, `leftDescent` returning the lowest INTERNAL descent
(weyl.cpp:919-926), and the KGB involution sort tie-breaking on the
lexicographic transducer piece array (involutions.cpp:426-427,
weyl.h:133). The transducer is an implementation artifact: reproducing it
would wed the port to Fokko's renumbering forever.

This port instead represents a Weyl element by its ROOT PERMUTATION over
the crate's enumerated root system, plus a cached inverse and length:

- The permutation is canonical — two equal elements have equal
  representations, with no normal-form maintenance and no transducer.
- Length = the number of positive roots sent negative, read off a
  positivity slice the `RootSystem` now precomputes; cached at
  construction, recomputed in O(roots) at multiplication.
- `s` is a left descent of `w` iff `w^{-1}(alpha_s)` is negative — read
  the INVERSE vector; `s` is a right descent iff `w(alpha_s)` is negative
  — read the FORWARD permutation. Both are O(1) against the precomputed
  positivity slice and simple-root IDs.
- Multiplication composes permutations in O(roots); the length change the
  Tits sigma formulas consume (tits.cpp:469-503) is cached-length
  subtraction, and the simple-multiplication specializations report it as
  a signed unit step.
- A reduced word is extracted on demand by descent peeling (the crate's
  lowest-generator order, not Atlas's internal one); words are not
  stored. `push_across`/`pull_across` depend only on the group element —
  their defining equation is in `w` itself and `reflect` is conjugation
  by `sigma_s` in an honest mod-2 W-representation (tits.cpp:425-432,
  tits.h:516-526) — so the crate's word, though different from Atlas's
  transducer word, is valid for them.

Consequence for observables, recorded now and SCOPED by review: on the
full-KGB construction path, the involution-sort tie-break at equal
involution and Weyl lengths (involutions.cpp:427 via weyl.h:133) is the
only transducer-order leak into numbering, and this port replaces it with
a DOCUMENTED crate order (lexicographic root-permutation compare, the
derived `Ord`). The partial-KGB and global-KGB seeding paths additionally
leak `leftDescent`'s internal-lowest preference through `involution_expr`
(tits.cpp:815, kgb.cpp:253) — both paths sit on the KGB map's deferred
list. Sizes, lengths, and status multisets stay directly
differential-comparable; tau-packet sizes compare as multisets keyed by
involution (or grouped by involution length and Weyl length), not as a
raw sequence, since the tie-break reorders packets within equal-length
groups; element numbering joins the standing adapter deferral.

## Data layout and public boundary

```text
// weyl_element.rs
#[derive(Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
WeylElement {
    permutation: Vec<RootId>,   // FIRST field: derived Ord IS the documented order
    inverse: Vec<RootId>,
    length: usize,
}
WeylElement::identity(&RootSystem) -> Result<...>
WeylElement::simple_reflection(&RootSystem, generator) -> Result<...>
WeylElement::from_action(&RootSystem, &WeylAction) -> Result<...>

length() / is_identity()
image(root: RootId) -> Option<RootId>
image_permutation() -> &[RootId]
has_left_descent(&RootSystem, generator) -> Result<bool>   // inverse vector
has_right_descent(&RootSystem, generator) -> Result<bool>  // forward vector
multiply(&RootSystem, &WeylElement) -> Result<WeylElement>       // self after right
left_multiply_simple(&RootSystem, generator)
    -> Result<(WeylElement, isize)>                        // s * w, change is +-1
right_multiply_simple(&RootSystem, generator)
    -> Result<(WeylElement, isize)>                        // w * s, change is +-1
inverse() -> WeylElement
reduced_word(&RootSystem) -> Result<Vec<usize>>
    // w = s_{word[0]} * s_{word[1]} * ... left-to-right
twisted_conjugate(&RootSystem, generator, twist: &[usize]) -> Result<...>
```

Field order is load-bearing: `permutation` first makes the derived `Ord`
the documented lexicographic permutation order, and since `inverse` and
`length` are functions of `permutation` established by every constructor,
derived `Eq`/`Hash` agree with permutation-only equality. The full derive
set is required by consumers: stage (b) hashes twisted involutions, stage
(e) dedups elements, and the involution sort uses the tie-break `Ord`.

The element does NOT own the root system: operations take `&RootSystem`.
The only per-operation provenance check EXPRESSIBLE at this layer is that
the permutation length matches the system's root count
(`WeylElementInvariantViolation { invariant: "provenance" }`); a foreign
system with the same root cardinality is undetectable here, so the
single-ambient-system discipline is the CALLER'S CONTRACT, owned by the
KGB stages exactly as the fiber layers own their `Arc` provenance.
Antisymmetry (`p(-alpha) = -p(alpha)`) is guaranteed by keeping the
constructors the only entry points — there is no raw-permutation
constructor. Construction from a `WeylAction` goes through the existing
`action_permutation` (which validates datum match and root images), adds
a sentinel-fill inverse that rejects duplicate images as a free
bijectivity check, and counts length; `simple_reflection` reuses it. A
`to_action` inverse is deliberately deferred until a consumer needs it.

`RootSystem` gains two precomputed slices at `from_closure` time, both
O(roots x rank) once and negligible next to enumeration: a positivity
slice (`positive: Vec<bool>`, exposed crate-internally, keyed by the
sign of the simple-coordinate vector — the half-split shortcut is NOT
valid because `roots` sorts by ambient coordinates) and the simple
roots' `RootId`s (today every consumer rebuilds them by binary search).
Both are deterministic derived data; their participation in the derived
`RootSystem` equality is harmless.

`multiply` composes `self` after `right` — matching `WeylAction::compose`
— as `product.permutation[r] = self.permutation[right.permutation[r]]`,
and maintains the inverse by the DUAL composition in the same O(roots)
pass (`product.inverse[r] = right.inverse[self.inverse[r]]`, the
`(uv)^{-1} = v^{-1} u^{-1}` order — a classic bug site the invariant
tests cover). The product's length is recomputed from the positivity
slice, NEVER derived from operand lengths (lengths do not add). The
simple specializations report the +-1 change with checked `isize`
arithmetic and this pinned sign convention: -1 means the length
DECREASED, the branch on which `sigma_mult`/`mult_sigma` add `m_alpha`;
+1 means it increased, the `sigma_inv_mult`/`mult_sigma_inv` branch
(tits.cpp:469-503). Atlas's general element-by-element `mult` is void
(weyl.h:310-313) — the general `multiply` here returns only the product,
and callers who need a general change subtract cached lengths.

`reduced_word` peels the lowest left descent repeatedly, so
`w = s_{word[0]} * s_{word[1]} * ...` composes left-to-right; it must
terminate in exactly `length()` steps, checked as
`WeylElementInvariantViolation { invariant: "descent peeling" }`.
`twisted_conjugate(s, twist)` forms `s * w * twist(s)` (the W-shadow of
tits.h:598-599); the twist stays a bare `&[usize]` because
`cayley_cross.rs` already manufactures exactly that shape, but it is
validated per call in O(rank): correct length, entries in range, and
involutive (`twist[twist[i]] == i`). That `alpha_{twist(i)} =
delta(alpha_i)` for the actual distinguished involution is the caller's
contract, enforced at stage (b) where delta lives. The twisted-conjugacy
LENGTH CHANGE is load-bearing there — upstream consumes `d` with
`length += d/2; W_length += d` (involutions.cpp:231, 251-252) — and is
derivable by subtracting cached lengths (`d` is 0 or +-2), so the method
returns only the element. Generator indices out of range are
`IndexOutOfRange` against the semisimple rank. Degenerate systems (rank
zero) yield the identity-only group and empty words.

## Resource and arithmetic policy

Element operations are O(roots) with no enumeration and no unbounded
intermediates; no budget knob is warranted at this layer — the KGB stage
will bound ELEMENT COUNTS, not per-element arithmetic, per the trace's
budget analysis, and `reduced_word` is a priori bounded by `length()`.
Allocations use the shared `try_capacity`; length arithmetic is checked;
permutation indices are `RootId` values validated by construction. No
resource-limit error variant is added — only
`WeylElementInvariantViolation { invariant }` with the crate's standard
display shape.

## Tests and fixture gate

- A2: all six elements enumerable by multiplication closure from the
  generators; lengths 0 through 3 with the longest element sending every
  positive root negative; `reduced_word` of the longest element has
  length 3 and multiplies back to it; left and right descents match the
  textbook values via the inverse-vector/forward-vector split.
- B2: the eight elements; `s0 s1 s0 s1` equals `s1 s0 s1 s0` (the braid
  relation at the group level); for every product of enumerated
  elements, the cached length equals the positivity recount, and
  `(uv)^{-1} = v^{-1} u^{-1}` holds (the inverse-maintenance
  regression); the simple multiplications report exactly +-1 agreeing
  with cached-length subtraction.
- Cross-validation: for every enumerated `WeylAction` of A2 and B2,
  `from_action` round-trips through `action_permutation`, lengths match
  inversion counts computed independently through the action's weight
  images, and twisted conjugation with the identity twist matches
  conjugation.
- The A1 x A1 swap twist: mapping reduced words through the swap
  rebuilds `delta(w)`, and the twisted-involution set
  `{ w : w * delta(w) = e }` is exactly `{ e, s0 s1 }`.
- Provenance: an A2 element used against the B2 system (different root
  counts) is rejected by the expressible gate; the same-cardinality
  limitation is documented as caller contract, not tested as detection.
  `reduced_word` on the identity is empty; a pure torus yields the
  identity-only group.

`tests/fixtures/domain/weyl_element.atlas` is reserved; this substrate
has no direct language observable — its differential exposure arrives
through KGB numbering, already covered by the adapter deferral.

## Consequential updates

Landing this stage must update: `lib.rs` (rename the legacy A1 prototype
`WeylElement` to `PrototypeWeylElement` — it collides with the new
public type — then add the module and `pub use`); `weyl.rs` (it has NO
module doc today — add one stating the matrix layer is the
provenance-bearing action representation and the word-level substrate
lives in `weyl_element.rs`); `root_system.rs` (positivity slice and
simple-root IDs); `KGB_STAGE_MAP.md` (stage (a) landed; the observables
scoping above); and `REAL_GROUP_DESIGN.md`'s progression paragraph (next:
the involution table).

## Three independent design checks

All three reviews returned before implementation; corrections folded
above:

1. Atlas semantics: VERIFIED the sigma sign conventions (m_alpha on
   left-multiplication decrease for `sigma_mult`, increase for
   `sigma_inv_mult`, right-side mirrors), the push/pull word directions
   (left-to-right / right-to-left), and the test anchors (A1 x A1
   twisted involutions `{e, s0 s1}`). CORRECTED: the word-independence
   justification (no literal braid comment upstream — it follows from
   the defining equation plus `reflect` being an honest
   W-representation); the "only leak" claim scoped to the full-KGB path;
   packet sizes weakened to multiset comparison; the antisymmetry gap
   closed by constructor-only entry; the twisted-conjugation length
   change named as stage (b)'s consumed signal.
2. Rust internals: CORRECTED the scale story (bounded by involution
   count, not element count); ADDED the `RootSystem` positivity slice
   and simple-root IDs (with the invalid half-split shortcut
   documented); PINNED multiply conventions (self-after-right, dual
   inverse composition, length by recount, subtraction for the change);
   settled derives (`Eq`/`Hash` sound because non-key fields are
   functions of the first; field order load-bearing for `Ord`); kept
   the bare-slice twist with per-call validation.
3. Public API and consumer fit: CAUGHT the `lib.rs` name collision with
   the legacy A1 prototype; REWROTE the provenance gate as caller
   contract (only root-count expressible); required the full derive set
   and the `image`/`image_permutation` accessors; dropped the tuple
   from general `multiply` (void upstream; subtraction suffices); fixed
   the missing `weyl.rs` module doc and added `KGB_STAGE_MAP.md` to the
   consequential updates.
