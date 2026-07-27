# Weyl element substrate design

## Approved scope

This stage adds the word-level Weyl substrate the KGB construction needs
and the crate's matrix-based `WeylAction` deliberately does not provide:
elements with O(1) length and descent queries, multiplication reporting
length change, inverses, twisted conjugation, and on-demand reduced words.
It is stage (a) of the KGB map recorded in the strong-real era: the
InvolutionTable, Tits operations, seed, and generation are later stages.
No new mathematics enters — this is pure Coxeter combinatorics — but the
representation decision is numbering-relevant and made here.

## Atlas construction and the representation decision

Citations are to `~/mycodes/atlasofliegroups`, master `4d3e9449`.

Atlas represents Weyl elements through Fokko du Cloux's transducer tables
(weyl.cpp:495-527 builds an INTERNAL generator renumbering that reverses
B/C/D components), with `length_change` reported by the multiplication
primitives, `leftDescent` returning the lowest INTERNAL descent
(weyl.cpp:919-926), and the KGB involution sort tie-breaking on the
lexicographic transducer piece array (involutions.cpp:426-427,
weyl.h:133). The transducer is an implementation artifact: reproducing it
would wed the port to Fokko's renumbering forever.

This port instead represents a Weyl element by its ROOT PERMUTATION over
the crate's enumerated root system, plus a cached length:

- The permutation is canonical — two equal elements have equal
  representations, with no normal-form maintenance and no transducer.
- Length = the number of positive roots sent negative; cached at
  construction, recomputed in O(roots) at multiplication.
- `s` is a left descent of `w` iff `w^{-1}(alpha_s)` is negative — one
  permutation lookup on the inverse, which the element also caches, plus a
  `simple_coordinates` sign scan.
- Multiplication composes permutations in O(roots) and reports the length
  change by subtraction — exactly the signal the Tits sigma-multiplication
  formulas consume (tits.cpp:469-503).
- A reduced word is extracted on demand by descent peeling (O(length x
  rank) lookups), which `push_across`/`pull_across` will consume in the
  Tits stage; words are not stored.

Consequence for observables, recorded now: the KGB involution-sort
tie-break at equal involution and Weyl lengths will use a DOCUMENTED crate
order (lexicographic root-permutation compare), not Atlas's transducer
order. Sizes, lengths, packet sizes, and status multisets stay
directly differential-comparable; element numbering joins the standing
adapter deferral, exactly as the KGB trace's observables analysis
prescribes.

## Data layout and public boundary

```text
// weyl_element.rs
WeylElement {
    permutation: Vec<RootId>,           // image of each root, by RootId
    inverse: Vec<RootId>,
    length: usize,
}
WeylElement::identity(&RootSystem) -> Result<...>
WeylElement::simple_reflection(&RootSystem, generator) -> Result<...>
WeylElement::from_action(&RootSystem, &WeylAction) -> Result<...>

length() / is_identity()
has_left_descent(&RootSystem, generator) -> Result<bool, ...>
has_right_descent(&RootSystem, generator) -> Result<bool, ...>
multiply(&RootSystem, &WeylElement) -> Result<(WeylElement, i64), ...>
    // (product, length change); left_multiply_simple /
    // right_multiply_simple specializations report the +-1 change
inverse() -> WeylElement
reduced_word(&RootSystem) -> Result<Vec<usize>, ...>   // descent peeling
twisted_conjugate(&RootSystem, generator, twist: &[usize]) -> ...
```

The element does NOT own the root system: at KGB scale (hundreds of
thousands of elements) an owned or Arc'd system per element is waste, and
the crate already has a same-datum provenance idiom — operations take
`&RootSystem` and validate that the permutation length matches the
system's root count plus the deterministic-root-order equality that datum
equality already implies. Construction from a `WeylAction` goes through
the existing `action_permutation`, making the two Weyl layers mutually
checkable; a `to_action` inverse is deliberately deferred until a consumer
needs it. Simple-reflection permutations are built once per call from the
system; the KGB stage will cache the `rank` of them at its own level.

`multiply` allocates two permutation vectors per call via
`try_reserve_exact`; the length change is computed from cached lengths
with checked signed arithmetic. `reduced_word` peels the lowest left
descent repeatedly — the crate's deterministic order, not Atlas's
internal one — and must terminate in exactly `length()` steps, checked as
an invariant (`WeylElementInvariantViolation { invariant: "descent
peeling" }`). Degenerate systems (rank zero) yield the identity-only
group and empty words.

## Resource and arithmetic policy

Element operations are O(roots) with no enumeration and no unbounded
intermediates; no budget knob is warranted at this layer — the KGB stage
will bound ELEMENT COUNTS, not per-element arithmetic, per the trace's
budget analysis. Allocations use `try_reserve_exact`; length arithmetic is
checked; permutation indices are `RootId` values validated by
construction.

## Tests and fixture gate

- A2: all six elements enumerable by multiplication closure from the
  generators; lengths 0 through 3 with the longest element sending every
  positive root negative; `reduced_word` of the longest element has
  length 3 and multiplies back to it; left and right descents match the
  textbook values.
- B2: the eight elements; `s0 s1 s0 s1` equals `s1 s0 s1 s0` (the braid
  relation at the group level); length changes on both sides of every
  product agree with cached-length subtraction.
- Cross-validation: for every enumerated `WeylAction` of A2 and B2,
  `from_action` round-trips through `action_permutation`, lengths match
  inversion counts computed independently, and twisted conjugation with
  the identity twist matches conjugation.
- The A1 x A1 swap twist: twisted conjugation with the swap permutation
  matches the matrix layer's twisted involution set.
- Provenance: a permutation from a different same-rank system is rejected
  by the length/datum gates; `reduced_word` on the identity is empty.

`tests/fixtures/domain/weyl_element.atlas` is reserved; this substrate has
no direct language observable — its differential exposure arrives through
KGB numbering, already covered by the adapter deferral.

## Consequential updates

Landing this stage must update: `lib.rs` (module and exports); `weyl.rs`'s
module doc (the word-level substrate lives in `weyl_element.rs`; the
matrix layer remains the provenance-bearing action representation); and
`REAL_GROUP_DESIGN.md`'s progression paragraph (KGB stage (a) done; next
the involution table and Tits operations).

## Three independent design checks

Before implementation, this design must be reviewed in three fresh
subagent contexts: (1) Atlas source semantics — the length-change
contracts of the sigma formulas this substrate must serve, and the
observables consequences of dropping the transducer order; (2) Rust
internals — the permutation representation against `root_system.rs`, the
descent and multiplication mechanics, and the provenance gates; (3) public
API and consumer fit — naming, the borrowed-`&RootSystem` calling
convention at KGB scale, and what stages (b) and (c) will need. Their
findings and any corrections will be recorded here before source edits
begin.
