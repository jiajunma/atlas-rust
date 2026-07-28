# Axis core design: types, definitions, functions (language phase B)

Substrates (all cites live there, not duplicated here):
- docs/AXIS_LANGUAGE_TRACE.md — grammar (parser.y productions), set/
  set_type/forget semantics, overload-table rules, session output.
- docs/AXIS_CORE_TRACE.md — the type analyser (type_expr, specialise,
  convert_expr, the COMPLETE coercion table, balancing, overload call
  resolution, closures/frames, the runtime value model with exact
  printing, assignment typing) and the startup builtin inventory with
  the basic.at load-bearing minimum.

## Goal and evidence

After phase A, 227 of 240 corpus scripts die at basic.at:3 (`set_type`)
and the rest die on their own `set` commands. basic.at exercises the
whole definition/type core within its first 40 lines (union set_type,
function-definition sugar with typed bodies, operator overloading and
op-casts, lambdas, `+:=`, function types, `!` constants). Phase B is
therefore the axis STATIC TYPE SYSTEM and definition machinery, ported
as upstream structures it: parse -> type analysis producing a typed
executable -> evaluation. There is no dynamic-dispatch shortcut: call
sites resolve overloads STATICALLY from argument types, so expression
evaluation cannot stay type-free.

## Architecture (mirrors upstream axis.w / axis-types.w)

New atlas-core modules; the existing eval.rs pipeline is REPLACED by
the typed pipeline at the end of B2 (a live migration, not a parallel
stack — the language bridge's ~15 domain builtins re-register in the
new overload table with their upstream signatures).

1. `types.rs` — the type model:
   - `Type`: `Undetermined` (`*`), `Primitive(Prim)`,
     `Function(Box<(Type, Type)>)`, `Row(Box<Type>)`,
     `Tuple(Vec<Type>)`, `Union(Vec<Type>)`, `Tabled(TypeNumber)`.
     void = `Tuple(vec![])`; length-1 tuples/unions are unrepresentable
     by construction (constructors collapse them).
   - `Prim` staged: B ships `Int, Rat, String, Bool, Vec, Mat, RatVec`
     plus the five landed domain prims (`LieType, RootDatum,
     InnerClass, RealForm, KGBElt`); `WeylElt, CartanClass, Block,
     Split, KType, KTypePol, Param, ParamPol` join with their layers.
   - `specialise` / `can_specialise` exactly as traced (tag match +
     recursion, tabled equality by number, `*` narrowing; NOT
     commit-or-rollback — `can_specialise` guards where rollback
     matters).
   - The typedef table (`type_map`): `TypeBinding { name, type,
     fields }` — variant/field names live HERE, never in Type.
     Display of types matches upstream exactly (`(int,int->int)`,
     `(int|string)`, `[vec]`, `void`, tabled by name).
2. `typed.rs` — the typed executable and conversion:
   - `convert_expr(&Expr, &mut Type, &mut Analysis) ->
     Result<TypedExpr, Diagnostic>`: checking-and-synthesis in one
     pass; the in/out type pattern mutates only via specialise;
     `conform_types` = specialise-else-coerce-else-type-error.
   - The COERCION TABLE ported verbatim from the trace (all ~30
     entries incl. the mat/[vec]/[[int]] web and the domain coercions
     InnerClass->RootDatum etc.); coercion nodes evaluate exactly one
     conversion function; voiding is the analyser's job, with explicit
     `Voiding` nodes only where the trace lists them (void-typed
     assignment rhs, void components, void-arg calls).
   - BALANCING for if/case/list displays as traced: convert each
     branch against a copy, maintain the broadest common type under
     `broader_eq`, absorb nested balance errors, re-convert divergent
     branches at the final target.
   - OVERLOAD RESOLUTION as traced (the a-priori-type design): convert
     arguments once in undetermined context; one variant pass — exact
     match wins immediately, else first `is_close & 0x1` variant is
     the inexact candidate; generic operators (`#`, `##`, `print`,
     `prints`, `to_string`, `error`, `=` special-casing) sit between
     exact and inexact; inexact resolution re-converts or coerces
     per-argument with the "shielding" expression-form list. Local
     bindings shadow ALL overloads; Id_table function values apply
     only when no variants exist.
   - `is_close` with the 3-bit contract, ported with the coercion
     table (they must stay consistent — one source of truth for both).
3. `value.rs` extensions (payloads match upstream EXACTLY —
   differential-critical):
   - `Vec32(Vec<i32>)` (upstream vec = machine 32-bit entries; checked
     conversions from Integer values, overflow = runtime error),
     `Matrix { rows, cols, data: Vec<i32> }` (column-major fill as
     upstream ctor), `RatVec { numerators: Vec<i64>, denominator: u64 }`
     (normalised on construction), `Union { tag, injector_name, value }`,
     `Closure`, `Builtin`. Existing Integer/Rational stay malachite
     (upstream is arbitrary precision).
   - Printing ported from the trace's exact rules: vec right-aligned
     `setw(w+1)` fields with `" ]"` tail, empty `[ ]`; ratvec numerator
     -vector then `/denom`; mat leading newline + `|`-framed per-column
     -width rows (or `The KxL matrix` for empty); row `[a,b,c]` no
     padding; tuple `(a,b,c)`; union `value.injectorname`; closure
     `Function defined <loc>`.
4. `frames.rs` — environments:
   - `EvaluationContext`: Rc-linked list of frames (`Vec<SharedValue>`)
     — closures keep tails alive, exactly the upstream sharing shape.
     Locals are `(depth, offset)` fixed at analysis; empty layers do
     not count toward depth.
   - Globals: `Id_table` entry holds `Rc<RefCell<Option<Value>>>`;
     converted code captures the CELL at analysis time (re-`set` with a
     new type rebinds a fresh cell and never affects old code; reading
     an unset cell is a runtime error).
   - `OverloadTable`: name -> variants sorted most-specific-first with
     the traced add/replace/reject rules (equal arg type replaces in
     place; mutually-convertible or undirected-close pairs rejected
     with the upstream two-line message).
5. Evaluation:
   - `TypedExpr::evaluate(&mut ctx, Level)` with the level protocol
     (`NoValue, SingleValue, MultiValue`) — a Rust value-returning
     shape: `Result<Eval, Control>` where `Eval` is `Value`/`Values`/
     `None` per level and `Control` carries `Break(n)`, `Return(v)`,
     and `RuntimeError(Diagnostic + backtrace)`; call sites append
     trace-back lines as upstream does. `back_trace: [string]` global
     variable filled when a runtime error reaches top level.
   - Strict call-by-value; builtins receive expanded args, closures a
     single value, per the traced argument policy.
6. Commands (`global.rs`): `set` (all declaration forms incl. function
   sugar and operator definitions), `IDENT : expr` / `IDENT : type`,
   `forget name` / `forget name@type`, `set_type` (simple + fields),
   `whattype`/`showall` — each with the BYTE-EXACT report lines and
   2 x include-depth indentation from the defs trace (the frame
   already carries the depth). Definition evaluation order is
   observable and ported: typecheck fully, evaluate rhs once, then
   per-identifier report lines in pattern order.

## Grammar migration

The lalrpop grammar grows in stage order (below). Two lexer notes:
- `TYPE_ID` needs lexer feedback (an identifier defined as a type
  scans as TYPE_ID; inside `set_type [ ... ]` ALL identifiers do).
  Port: the session frame already owns the command loop — it passes a
  `LexerHints { type_ids: <from Id_table>, in_type_definition: bool }`
  snapshot per command; the lexer consults it exactly where upstream
  consults global_id_table. No mid-command mutation (upstream defines
  types only between commands).
- New tokens: `$`, `@`, `|` in type position, `->` (exists), `!`
  pattern marks (exists as operator), OPERATOR_BECOMES family
  (`+:=` etc.) — scanned as one token with the traced
  space-tolerant rule.

## Staging (each stage: tests green + clippy 1.90/1.96 + commit;
corpus rerun at B4, B5, B7)

- B1 types.rs: model, display, specialise, typedef table; type
  -expression grammar (`(int,int->int)`, `[T]`, unions, `(->)`);
  unit tests against upstream-printed spellings.
- B2 typed pipeline for EXISTING forms: TypedExpr + convert_expr for
  denotations, identifiers, calls of builtins, arithmetic formulas,
  tuples/lists, if/let (current subset), casts `T:`; coercion table;
  balancing; new Value variants + printing; level protocol; the
  builtin registry rehomes the current ~15 domain builtins + core
  int/rat/bool/string arithmetic as PROPER overload-table entries.
  Gate: every existing session/domain test passes through the typed
  pipeline (Value displays unchanged for covered types).
- B3 functions: lambda `(id_specs)`/`@`, closures, user calls,
  rec_fun, `return`; patterns (tuple, `!`, holes, `(list):name`);
  let with patterns.
- B4 definitions: Id/overload tables, set family, `:` declarations,
  forget, operator definitions, op-casts `f@type` / `op@type`, casts;
  report lines; `$`; assignment family (:=, multi, component with
  subscript kinds, field, op:= desugar and transform nodes).
  CORPUS RERUN: scripts stop dying on `set` — the histogram shifts
  into set_type and missing builtins/features.
- B5 set_type: fields/projectors/injectors, bracketed recursive form
  with the type-defining lexer state, union `case`/discrimination.
  CORPUS RERUN: basic.at:3 falls; histogram = next real blockers.
- B6 control flow completion: full for/while grammar (`@` index,
  `downto`, reversal `~` flags, do-forms), `case` int variants with
  all six orderings, slices (1-D flags done; 2-D matrix slicer),
  `break n`, `die`, `next`.
- B7 builtin core: the basic.at load-bearing minimum from the
  inventory (generic `#`/`##` family, prints/to_string/error, the
  int/rat/vec/mat/ratvec arithmetic + comparison + structure set,
  `%`-decompositions, linear algebra incl. union-typed linear_solve,
  swiss_matrix_knife alias, back_trace, elapsed_ms as id-table
  entry). GATE: `<basic.at` loads end to end; corpus rerun histogram
  recorded; MATCH count grows past the current 2.

Math-layer builtins beyond basic.at's minimum (Block/KL, WeylElt,
Cartan classes, K-types, Param) stay with their porting tasks (#14,
KGB adapters) — the registry makes adding an entry a one-liner.

## Explicit non-goals in phase B

- Interactive/prelude modes, quiet/verbose, readline (session frame
  non-goals carry over).
- `whattype`/`showall` REDIRECT forms (with phase A's deferral).
- Extended/twisted machinery, ParamPol layers, deformation builtins.
- Performance work (task #9 stays parked; the typed pipeline may not
  regress the differential timings materially — checked at B7 with
  the kgb battery).

## Three independent design checks

(1) Upstream fidelity — the type model/specialise/coercions/balancing
/overload resolution/value payloads+printing/report lines against
axis-types.w, axis.w, global.w; especially the coercion-table
completeness, the a-priori-type resolution order, and the exact
overload add/reject rules. (2) Rust internals — the Rc frame chain vs
borrow rules, RefCell global cells, Control-flow-as-Result, i32/i64
payload conversions from malachite values, lexer-hints plumbing
through the session frame, migration risk of replacing eval.rs while
keeping 292 tests green. (3) API and consumer fit — the staged
migration's blast radius per stage (session gate tests, domain
builtins re-registration, kgb_differential unchanged, corpus driver),
grammar-conflict risk in lalrpop for the new productions, and whether
the B2 "existing forms" cut is truly self-contained. Corrections fold
here before implementation.
