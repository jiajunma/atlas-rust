# Axis core design: types, definitions, functions (language phase B)

Substrates (all cites live there, not duplicated here):
- docs/AXIS_LANGUAGE_TRACE.md — grammar (parser.y productions), set/
  set_type/forget semantics, overload-table rules, session output.
- docs/AXIS_CORE_TRACE.md — the type analyser (type_expr, specialise,
  convert_expr, the COMPLETE coercion table, balancing, overload call
  resolution, closures/frames, the runtime value model with exact
  printing, assignment typing) and the startup builtin inventory with
  basic.at's load-bearing minimum.

Reviewed 2026-07-28 by three independent fresh-context checks
(fidelity 10 / internals 9 / staging 9 findings); every correction is
folded below, [R] marking reshaped decisions.

## Goal and evidence

After phase A, 227 of 240 corpus scripts die at basic.at:3 (`set_type`)
and the rest die on their own `set` commands. basic.at (2249 lines)
exercises the whole definition/type core AND the math-layer value
types: Split at line 753, W_elt at 1298, Cartan_info at 1396, KType at
1675, Param at 1763, branch/block_deform at 2046/2064 — every one a
`set` command that must TYPE-CHECK for the file to load. Phase B is
the axis static type system and definition machinery, ported as
upstream structures it: parse -> type analysis producing a typed
executable -> evaluation. Call sites resolve overloads statically, so
evaluation cannot stay type-free.

[R] Consequence the reviews forced into the open: basic.at cannot load
on language features alone — it also needs the WeylElt/CartanClass/
Block/Split/KType/Param BUILTIN surfaces (trace §1j-1s). Phase B
delivers the language plus the non-math layers (Split is pure pairs);
the math-layer builtins land with their porting tasks, and the B7 gate
is stated honestly below.

## Implementation status (2026-07-29)

The B2 typed-pipeline swap is now live in `main` at `c6d5d6a`:
`session.rs` routes command execution through typed conversion and
evaluation, and the former `eval.rs` implementation has been removed.
The locally verified surface is the scalar/container/subscription/slice
pipeline plus the currently exposed RootDatum, matrix, Cartan-matrix,
and KGB constructor adapters. RootDatum inference checks exact Cartan
matches before simultaneous relabeling so rank-two B/C orientation is
preserved; list and conditional balancing follows upstream conflict
pruning, including nested void salvage.

This is not yet an Atlas-compatibility claim. The typed-swap differential
job has not run for this commit because the XMU SSH path was unavailable;
the checked-in HPC harness and oracle metadata remain ready for that gate.
InnerClass/RealForm/KGB rendering and external numbering, function
closures, definitions, loops, and the broader builtin inventory remain
staged work below.

## Architecture (mirrors upstream axis.w / axis-types.w)

New atlas-core modules; the existing eval.rs pipeline is REPLACED by
the typed pipeline at the end of B2 (a live migration with an explicit
test-migration list — see B2).

1. `types.rs` (landed, B1) — the type model:
   - `Type`: `Undetermined`, `Primitive(Prim)`, `Function`, `Row`,
     `Tuple`, `Union`, `Tabled(TypeNumber)`; void = empty tuple;
     length-1 collapse; specialise/can_specialise as traced; display
     byte-exact.
   - [R] `Prim` ships ALL TWENTY upstream primitives in B1 (int rat
     string bool vec mat ratvec LieType RootDatum WeylElt InnerClass
     RealForm CartanClass KGBElt Block Split KType KTypePol Param
     ParamPol) and the LEXER reserves all twenty PRIMTYPE names —
     positional lexing of the names is load-bearing even before their
     layers exist (`= WeylElt:` must not scan as an identifier).
   - [R] TABLING: only the bracketed `set_type [ ... ]` form creates
     `Tabled` entries (name-printed, discrimination-capable). Simple
     `set_type Id = T` is an eagerly-expanded parse-time alias whose
     later mentions print STRUCTURALLY; its field names exist only as
     projector/injector overload entries. Tagged discrimination
     requires a tabled union and errors otherwise.
2. `typed.rs` — conversion (unchanged from the first draft except):
   - The coercion table ported verbatim WITH REGISTRATION ORDER
     preserved — `coerce` and `row_coercion` are first-match linear
     scans (a mat-context list display must pick component `vec`).
   - [R] Voiding-node sites: void-typed assignment rhs, void
     components, void-arg calls, AND void-typed LOOP BODIES in row
     context (while/for-in/counted-for).
   - [R] Special generic operators are exactly the traced seven —
     `#`, `##`, `## ` (trailing space, targeted by the parser's
     loop-flattening/matrix desugars), `print`, `prints`,
     `to_string`, `error` — plus `not`. `=` is NOT generic-special;
     the only `=` special-casing is the void-context :=-suggestion
     error on an exact match. `print` is transparent (may re-convert
     its argument in the outer context).
   - [R] Local bindings shadow the overload table ONLY when the local
     binding has function type; a non-function local leaves overloads
     reachable.
   - [R] Overload variants carry the upstream `hunger` byte
     (coercion-tolerance mask; cross/Cayley install with 2).
3. `value.rs` extensions:
   - Payloads as traced (Vec32 i32 entries — landed; column-major
     Matrix — landed; RatVec i64/u64 — landed; Union; Closure;
     Builtin). Narrowing from Integer uses the EXACT upstream error
     text `Integer value to big for conversion` (typo included).
   - [R] Internal vec/mat/ratvec arithmetic WRAPS (`wrapping_*`) —
     upstream is unchecked machine-int C++ and wraps in practice;
     recorded as the deliberate mirror of that practice.
   - [R] `PartialEq` becomes MANUAL: structural for data variants,
     `Rc::ptr_eq` for closures, function-pointer identity for
     builtins (upstream never defines `=` on function values, so
     nothing observes closure structure). `SessionEvent`/`EvalEvent`
     equality keeps working through it. The derive breaks at compile
     time the moment the variants land, so this is B2 text.
   - [R] Rational printing is manual `numerator/denominator`
     INCLUDING `/1` (malachite's Display drops it); ratvec keeps its
     `/denom` tail after an empty `[ ]` too.
   - [R] Closure printing is the full upstream form: `Function
     defined <loc>` newline + printed lambda; recursive closures
     print `Recursive function defined <loc>` + `<name> = ...`.
4. `frames.rs` [R — reshaped by the internals review]:
   - `struct Frame { next: Option<Rc<Frame>>, slots:
     RefCell<Vec<SharedValue>> }` — interior mutability is REQUIRED
     (local assignment writes through the shared chain).
   - Context swaps (frame push, closure apply) go through a DROP-GUARD
     type so `?`-propagated `Control` cannot leave the environment
     corrupted (upstream relies on C++ RAII for exactly this).
   - Borrow discipline as invariants: reads clone and drop the borrow;
     writes happen only after the rhs fully evaluated, under a short
     borrow; no borrow held across a nested `evaluate` (closures
     re-enter shared tail frames).
   - Globals: `Id_table` entries hold `Rc<RefCell<Option<Value>>>`;
     [R] EVERY `set`/`:` definition allocates a FRESH cell
     unconditionally (same type or not) — converted code keeps the
     old cell; only `:=` writes through a captured cell.
5. Evaluation:
   - `Result<Eval, Control>` with `Break(depth)`, `Return(v)`,
     `RuntimeError`. [R] Interception contract pinned: loops consume
     `Break(0)` and rethrow decremented; closure apply consumes
     `Return` only; call sites map backtrace lines onto RuntimeError
     only; a Break/Return reaching top level is an internal error
     (analysis guarantees legality). Upstream's tuple-expression
     stack-cleanup catch has no analogue in a value-returning
     evaluator and is NOT ported.
   - [R] Builtins receive expanded args EXCEPT variadic ones
     (print/prints/to_string/error), which take a single value like
     closures.
6. Commands (`global.rs`): as before, with [R] the indentation rule
   corrected: 2 x include-depth applies ONLY to definition reports
   (set / `:` declarations / set_type); forget, whattype, and showall
   print UNINDENTED.
7. [R] OUTPUT-SURFACE OWNERSHIP (single formatter): `SessionEvent`
   extends ONCE with `ReportLine { text }`, and `Value` gains
   `is_void_type: bool` from the analyser. The session frame is the
   only formatter: it indents ReportLines by include depth, drops
   void-TYPED values (replacing phase A's value==() test — the typed
   pipeline makes the upstream static-type rule real), renders
   `Value: `. No formatting decided in two places.

## Grammar migration [R]

The expression grammar is replaced WHOLESALE with the upstream
parser.y shape at B2/B3 rather than grafted onto the current deviated
productions (upstream parser.y is verified LALR(1)-clean with zero
conflicts, so an exact production-level port is lalrpop-feasible; the
graft, not the grammar, is the conflict risk — e.g. the current
callable `()` atom must go: upstream excludes `'(' ')'` from comprim).
Token splits the port needs beyond the current lexer: dedicated
variants for `*`, `@`, `|` (lalrpop cannot match Operator payloads),
`$`, `!`, and the OPERATOR_BECOMES family. Casts land in B2 and are
PRIMTYPE-headed until typedefs exist — fine, stated explicitly.

Lexer feedback [R]: `type_ids` IS a legitimate between-commands
snapshot (typedefs land only at the `'\n'` reduction) — installed via
`Lexer::set_type_ids(...)` at each command boundary of the session
frame's `run_stream` (clone; no borrow interaction with the eval
context). `in_type_definition` is INTERNAL lexer state — toggled when
`set_type` is scanned with `[` lookahead, cleared at command end —
exactly the upstream state machine; a frame-supplied flag is
unimplementable.

## Staging [R — restated falsifiably]

Each stage: full workspace suite green (284 tests today; the number
moves as tests migrate — the gate is the suite, with the migration
list in the stage's commit message), clippy 1.90/1.96, fmt, commit.

- B1 (landed, amended): types.rs + linear_values.rs; [R] widen Prim
  and the lexer PRIMTYPE list to all twenty names.
- B2 THE PIPELINE SWAP. Scope = the ACTUAL current surface plus two
  named additions:
  - current: denotations, identifiers, operator formulas, tuples/
    lists, subscription + 1-D slices (incl. `~[`), builtin calls,
    pattern-less let, simple `:=` assignment, `IDENT : expr` /
    `IDENT : type` commands, `and/or/not` (desugared to conditionals
    as upstream);
  - new at B2: `if/then/elif/else/fi` (balancing needs a conditional
    to exist), casts `T:`, `begin/end` grouping.
  - value extensions + manual PartialEq; frames; level protocol;
    coercions; balancing; overload resolution.
  - [R] TEST MIGRATION IS PART OF THE STAGE, enumerated in the
    commit: session/domain gates rewritten to upstream call shapes
    (`simply_connected(Lie_type("A1"),true)` — the 1-arg form is a
    basic.at SCRIPT wrapper, not a builtin, and a deviant 1-arg
    registration would be replaced in place by basic.at:886 and
    poison its own body; `torus_factor` returns ratvec with ratvec
    display; `Lie_type` only at string/RootDatum; the string
    inner-class sugar is NOT a builtin and unregisters), eval.rs
    ScalarType/infer unit tests retired with the module, fixture
    expectations updated where displays change. Domain equality
    overloads (`= !=` at InnerClass/RealForm/KGBElt) register at B2
    or domain equality silently disappears. `prefers_coroots` bool
    threads through datum construction.
  - [R] a corpus rerun joins the B2 gate (protects the 2 MATCHes,
    which otherwise go unverified until B4).
- B3 functions: lambdas, closures, user calls, rec_fun, `return`,
  patterns, let-with-patterns; [R] the `.` selector application
  (`x.f` == `f(x)`, used at basic.at:35) belongs HERE with user
  calls.
- B4 definitions: tables, set family, forget, operator definitions,
  op-casts, `$`; assignment upgrades (multi, component with subscript
  kinds, field, op:= transform nodes). [R] Histogram expectation
  softened: deaths move off the `set` keyword into body features
  (loops, selectors), not to zero.
- B5 set_type: simple-alias form AND the bracketed tabled form with
  the type-defining lexer state, projectors/injectors, union case /
  discrimination. CORPUS RERUN: basic.at:3 falls.
- B6 control flow completion: full loop grammar, case variants,
  2-D slices, matrix display desugar (`[r1|r2]` -> transpose via
  `## `-style named builtins), `break n`, `die`, `next`.
- B7 builtin core: the non-math load-bearing inventory (generic
  `#`/`##` family, prints/to_string/error, int/rat/vec/mat/ratvec
  arithmetic + comparisons + structure + linear algebra incl.
  union-typed linear_solve and swiss_matrix_knife, `%`
  decompositions, ascii, back_trace, elapsed_ms, null/id_mat) PLUS
  the interpreter-local Split layer (pure (a,b) pairs).
  [R] GATE, honest form: basic.at loads THROUGH ITS SCALAR/VEC/MAT/
  ROOTDATUM PREFIX and the abandon point is the FIRST MISSING
  MATH-LAYER BUILTIN (recorded in the commit); corpus rerun
  histogram recorded. Full basic.at loading is the B8/#14 joint
  gate, not B7's.
- B8 (joint with task #14 and the KGB adapter work): the math-layer
  builtin surfaces (WeylElt words/products, CartanClass/Cartan_info,
  Block, KType/KTypePol, Param/ParamPol, raw_KL) over the landed
  real-group machinery. GATE: `<basic.at` loads END TO END; corpus
  MATCH count grows past 2.

[R] Performance check at B7 uses the corpus driver's per-script
timings (it already records both sides) or a language-level KGB
script through atlas-cli — the kgb_bench battery never touches
atlas-core and measures nothing about phase B.

## Explicit non-goals in phase B

- Interactive/prelude modes, quiet/verbose, readline.
- `whattype`/`showall` redirect forms.
- Extended/twisted machinery, deformation/KL computation BUILTINS
  beyond registration stubs agreed with task #14 (B8 scopes them).
- Performance work (task #9 stays parked).

## Review disposition

Fidelity 10: B7/prims scope contradiction resolved (all-20 prims at
B1, honest B7 gate, B8 added); fresh-cell-on-every-set; lexer-internal
type_defining state; simple-vs-bracketed set_type tabling split;
function-typed-only shadowing; loop-body voiding; the seven special
ops + `## ` + non-special `=` + variadic single-value + print
transparency; definition-only indentation; manual rational printing
incl. `/1`; full closure print. Internals 9: B2 cut restated to the
actual surface with an explicit test-migration list; builtin
re-registration reconciled with named test rewrites; manual
PartialEq; RefCell frames + drop-guard + borrow discipline; lexer
state split; wholesale grammar replacement + token splits; exact
narrowing message + wrapping arithmetic recorded; control-flow
interception contract pinned; test count corrected to the suite
(284 today). Staging 9: B7 gate honest + B8; signature mismatch list
(simply_connected/adjoint/Lie_type/torus_factor) with same-commit
test rewrites; B2 surface corrected both directions; `.` selector,
matrix display, begin/end staged; lexer state; single-formatter event
extension (ReportLine + is_void_type); corpus rerun at B2; hunger
byte + domain equality overloads; corpus timing check redirected.
No unresolved blocking items.
