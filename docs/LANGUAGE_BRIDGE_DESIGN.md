# Language bridge design (task #8)

## Approved scope

Connect the interpreter crate (`atlas-core`) to the domain crate
(`atlas-real-group`) so Atlas-language code constructs and queries
real-group values, making the seven reserved `domain/*.atlas` fixtures
executable and the KGB observables scriptable — the first structurally
complete language-level differential surface. Phased: this design
covers PHASE 1 (the constructor/query set below); blocks, parameters,
and the full script library are later phases.

## Survey facts (both sides)

INTERPRETER (`atlas-core`, working tree):
- `Value` is `{Integer, Rational, Boolean, String, Tuple, List}`
  (value.rs:11-19) with the doc comment explicitly reserving domain
  handles; derives `Clone, Debug, Eq, PartialEq` — embedded domain
  payloads must satisfy those bounds.
- THE GRAMMAR HAS NO FUNCTION APPLICATION — `f(x)` is unparseable
  (grammar.lalrpop:121-135); builtins today are symbolic operators
  dispatched twice on the symbol string (`infer_operator_type`
  eval.rs:447-500, `eval_operator_call` eval.rs:1047-1149) over
  `ScalarType` (eval.rs:250-259).
- Session pipeline: stateful command-at-a-time lexing
  (session.rs:22-90), `Command::{Expression, Define, Declare}`,
  events `SessionEvent::{Value, Output, Diagnostic}`. The
  `events.json` serializer DOES NOT EXIST (verification is in-crate
  asserts; atlas-cli is a stub; the sbatch writes a lexer-stage
  report), and the reference corpus has a nested-vs-flat Value shape
  inconsistency to resolve.
- The interpreter is completely domain-free today (zero references);
  `atlas-real-group`'s crate doc declares itself the adapter boundary.
- UNCOMMITTED slice-syntax work (eval.rs, grammar.lalrpop, session.rs,
  syntax.rs): `a[l:u]` postfix slices via a `PostfixSuffix` enum —
  the bridge's application syntax must compose with it, not collide.

UPSTREAM (atlas-types.w @ 4d3e9449), the phase-1 surface with exact
semantics:
- `Lie_type(string)` with implicit string coercion; the string parser
  (atlas-types.w:222-247, 165-211): repeat { skip punctuation/space;
  one letter of "ABCDEFGT"; unsigned decimal }, per-letter rank
  bounds (E only 6-8, F only 4, G only 2, D >= 4, B/C >= 2), total
  rank <= 32 upstream, `Tr` rewritten to r copies of `T1`.
- `simply_connected(lt)` (script default prefers coroots),
  `adjoint(lt)`, `root_datum(mat, mat, bool)` (columns = simple
  (co)roots), `root_datum(lt, mat, bool)` (sublattice of the weight
  lattice).
- `inner_class(rd, mat)` is the BUILTIN (conjugates the involution to
  the distinguished one of its class); the STRING form is
  script-level sugar over `involution(lt, perm, "Ccesu"-string)`
  with letter normalization (C consumes two identical factors,
  e == c, s -> c when w0 = -1, u only for A/D/E6/T).
- `real_form(ic, n)` numbers forms in the EXTERNAL FormNumberMap
  order (output.cpp:71-140): ascending depth = size of a maximal
  orthogonal set of noncompact imaginary roots, tiebreak by the
  special grading bitset — compact is external 0, quasisplit last.
  `form_number`, `quasisplit_form`, `nr_of_real_forms`,
  `base_grading_vector`, `initial_torus_bits`.
- `KGB_size(rf)`, `KGB(rf, i)`, and per-element: `cross(i, x)`
  (simple for i < ss_rank, posroot beyond, negative = negated root),
  `Cayley(i, x)` — COMBINED forward-and-inverse (`any_Cayley`),
  undefined returns the ARGUMENT UNCHANGED, only the FIRST of a
  double inverse returned; `status(i, x)` coded
  0=C- 1=ic 2=r 3=nc 4=C+ (descent iff < 3, imaginary iff odd,
  Cayley defined iff == 3); `length`, `involution(x)`,
  `torus_bits`, `torus_factor`, `Cartan_class`.
- Fixture needs: root_coroot needs only the datum layer; the "33 A1"
  scenario EXCEEDS upstream RANK_MAX = 32 — Rust-only, never
  oracle-replayed.

## Port decisions (for review)

1. DEPENDENCY DIRECTION: `atlas-core` depends on `atlas-real-group`
   (the domain crate is the declared adapter boundary; no cycle —
   the domain crate never learns syntax). No third crate yet (YAGNI).
2. VALUE MODEL: one new variant `Value::Domain(DomainValue)` where
   `DomainValue` is an enum of Arc-backed handles:
   `LieType, RootDatum, InnerClass, RealForm, KgbElement` (phase 1).
   Handles bundle their CONTEXT: an interpreter-side
   `Arc<RealFormContext>` owns the pipeline (inner class,
   classification, strong layer, shared involution table, seed, and
   the lazily built `KgbGraph`), so `KgbElement` is
   `(Arc<RealFormContext>, KgbId)` and equality is Arc-identity plus
   id — the crate's Arc-provenance idiom lifted to the language
   layer. All bounds (`Eq`) hold. Construction budgets are
   interpreter-session constants (documented; revisited when the
   language exposes budget control).
3. SYNTAX: function application lands as a new `PostfixSuffix`
   variant `Call(Vec<Expr>)` — composing with the uncommitted slice
   work's enum rather than colliding with it — plus identifier-head
   application only (no first-class functions yet; the head must be
   a known builtin name, checked at eval). String literals already
   exist; the implicit string->LieType coercion is replicated at
   argument-coercion time.
4. BUILTIN REGISTRY: a new `atlas-core` module `domain_builtins.rs`
   with a single dispatch table `name -> (signature, handler)`;
   `ScalarType` gains one leaf `Domain(DomainKind)` so inference
   stays exhaustive. Overloads resolve on argument kinds (upstream
   parity: `root_datum` has two shapes).
5. EXTERNAL REAL-FORM ORDER: the bridge implements the FormNumberMap
   sort exactly (depth via maximal orthogonal sets of noncompact
   imaginary roots at the distinguished involution, gradings::max_orth
   semantics, special-grading tiebreak) over the strong/grading
   layers — the stage-(d) adapter obligation lands HERE, as the
   single hardest new piece; `real_form(ic, n)` speaks external
   numbers, internal ids stay crate-side.
6. UPSTREAM QUIRKS PRESERVED: `Cayley` returns its argument when
   undefined and only the first inverse; `status` uses the 0-4
   coding; `cross` accepts posroot and negative indices (phase 1 may
   gate to simple indices with a documented deferral if the posroot
   tables are not yet exposed — review call).
7. EVENTS ADAPTER: a serializer from `SessionEvent` to the
   `events.json` row shapes, resolving the nested-vs-flat
   inconsistency in favor of the FLAT shape (the newer fixtures),
   with domain values printing through their `Display` (stable,
   documented rendering — KGB elements as `KGB element #n of ...`)
   — this is what makes the differential harness executable and is
   IN SCOPE for phase 1.
8. FIXTURE ACTIVATION ORDER: root_coroot first (datum layer only),
   then kgb_generation (sizes/statuses — the headline observables),
   then the intermediate fixtures as their queries land; the 33-A1
   scenario is marked Rust-only in its meta.

## Tests and review plan

Phase-1 gate: `simply_connected("A1")` through
`KGB_size(real_form(inner_class(rd, M), n))` end to end in-crate,
with SL(2,R)/Sp(4,R)/SU(2,1) sizes and statuses matching the domain
tests, and the external form order verified against the oracle's
published numbering (compact = 0). Three fresh-context reviews before
implementation: (1) upstream semantics of the surface list (the
Cayley/status/external-order quirks, the Lie-type parser spec);
(2) interpreter internals (grammar composition with the slice work,
the dispatch/inference extensions, the events serializer); (3) API
and consumer fit (the context-bundle shape, budgets, Display
renderings, fixture activation). Findings fold here before source
edits.

## Three independent design checks (returned; decisive corrections)

Full findings: the review archive (workflow wf_8d3d300d-f77 journal;
key excerpts below are normative). Fold status: recorded here, to be
applied during implementation.

1. BLOCKING, the external-order tiebreak spec (semantics): the
   tiebreak uses the PARTITION overload of specialGrading
   (cartanclass.cpp:929-948): scan adjoint-fiber indices ascending,
   keep the HIGHEST index attaining maximal popcount within the
   form's class (>= replacement, seeded with classRep); the grading
   is that index's COMPLEMENT within adjointFiberRank bits, unsliced
   onto the TWIST-FIXED simple generators, compared as unsigned with
   simple root 0 = LSB. Valid because the fundamental base grading is
   all-ones and shifts are the standard basis — the adapter must
   VERIFY that coordinate parity against the port's grading layer or
   the order silently diverges. Depth needs NEW machinery: the FULL
   noncompact imaginary root set per form (not just simple positions)
   plus the gradings.cpp:51-76 greedy (positive roots, lowest-index
   pick, orthogonality filter, short-root compactness FLIP when the
   sum is a root) — cardinality only, representative-invariant.
   upstream sorts with UNSTABLE std::sort: assert strict (depth,
   grading) ordering so any tie is a loud error.
2. BLOCKING, the context bundle (internals): "lazy KgbGraph in an
   Arc" is unimplementable (build needs &mut table; Value is
   immutable and cloned freely). Corrected: BUILD-THEN-FREEZE —
   real_form(ic, n) eagerly runs seed + KgbGraph::build on a
   PER-FORM CLONE of the pipeline, then wraps the finished six-piece
   bundle in Arc<RealFormContext>. DomainValue implements Eq BY HAND
   as Arc::ptr_eq + payload (the domain aggregates mostly do not
   derive PartialEq); upstream additionally MEMOIZES handles per
   constructor arguments (weak-pointer tables) so
   real_form(ic,0) = real_form(ic,0) holds across calls — the bridge
   memoizes per inner-class handle (structural equality =
   (inner-class Arc, internal form number)).
3. HIGH, exhaustive-match inventory (internals): Expr::Call also
   forces Expr::span, infer_scalar_type, eval_expr, and the test
   helper expression_shape; coerce_value_to_type NEEDS a Domain arm
   (lists coerce every element); assignment_compatible/common_type/
   the two operator matches need NO edits (equality arms + catch-alls
   cover them). Value's Display is where decision 7's renderings
   land.
4. HIGH, events corpus (internals): exactly 3 nested-shape files
   (eval/{scalars,context,exact_numerics}) regenerate to flat; the
   flat corpus must pick an integer encoding that survives big
   integers (decimal strings recommended) before regeneration.
5. HIGH, Display strings (semantics): byte-identical upstream forms —
   `KGB element #n` (NO suffix), `Lie type 'A1.T1'`,
   `[simply connected |adjoint ]root datum of Lie type '...'`, the
   multi-line inner-class print, and real-form prints REQUIRE the
   form-name machinery (deferred: phase 1 may print a documented
   stable placeholder, flagged adapter-sensitive).
6. MEDIUM quirks pinned (semantics): one shared index scheme for
   cross/Cayley/status with the SIGN DISCARDED (status(-1-i,x) ==
   status(i,x) — do not flip C+/C-); Ccesu: 'C' needs two identical
   CONSECUTIVE factors, 'u' legal for A_{n>=2}/any D/E6/T and
   rewritten to 's' except D_even; the string covers EVERY factor
   including expanded T1s; torus_factor SUBTRACTS (the .w prose
   claiming addition lies; the code subtracts); "first" of a double
   inverse Cayley = numerically smallest final KGB number (the
   port's (0, Some(1)) matches).
