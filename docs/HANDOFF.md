# Session handoff — 2026-07-28 evening

State record for the next agent. This handoff is current through
`2921da5` on `main` (git@github.com:jiajunma/atlas-rust), and the
working tree is clean. Scalar reference capture is verified by HPC
job `3496383`; Rust structural preflight is verified by HPC job
`3496382`.

## Standing directives from the user (in force)

1. GOAL: "做出 atlas 语言兼容的 rust 版本" — an Atlas-LANGUAGE-
   compatible Rust interpreter; existing .at script code must be
   reusable as-is. The yardstick is the 240-script upstream corpus
   differential (`hpc/script_corpus_diff.py`), byte-exact stdout.
2. PRIORITY: "先把这些东西都 port 了，然后再考虑优化" — porting
   BREADTH before performance. Task #9 (the E6 2.1s Cartan-discovery
   optimization) is PARKED with its fully reviewed design committed;
   do not resume it until the language core and math layers are done.
3. HPC testing is expected ("你可以在 HPC 上面跑测试"): majj@10.26.14.64,
   partition cpu, account yushilingroup. Preflights must pass under
   rustc 1.96's newer clippy as well as local 1.90.
4. Commit and push after every verified stage; the user reads progress
   from per-stage artifacts (design doc + reviews + tests + commits),
   not from conversation.

## Cadence convention (strict, has caught real errors every stage)

For every substantive stage: oracle trace (background agents read
/Users/hoxide/mycodes/atlasofliegroups, master 4d3e9449, READ-ONLY)
-> design doc in docs/ -> THREE parallel fresh-context reviews
(fidelity / Rust internals / API+staging lenses) -> fold corrections
into the design -> implement in small verified commits -> full suite
+ clippy 1.90 AND 1.96 + fmt -> commit + push. The three-review step
found ~20-28 real corrections per design; do not skip it.

## Where the port stands

### Done and validated
- Math core (atlas-real-group): root datum -> involutions -> Cartan
  classes -> real forms -> KGB, validated 17/17 groups against the
  C++ oracle on HPC (sizes, length multisets, external form order).
  175 tests. See docs/KGB_STAGE_MAP.md and the per-stage designs.
- Language phase A (session frame): `<file`/`<<file` includes with
  upstream bookkeeping, `>file`/`>>file` redirects, Value:/void/Bye.
  output surface, abandon cascade, clean flag, --path, registry-based
  diagnostics. Corpus gate passed: 0 include failures, both prior
  MATCHes retained (now 2), 227/236 failures collapse to basic.at:3
  (`set_type`) — the designed phase-B blocker. Design:
  docs/AXIS_SESSION_FRAME_DESIGN.md (22 review findings folded).
- Language phase B foundations (ALL reviewed via
  docs/AXIS_CORE_DESIGN.md, 28 findings folded — READ THAT DOC FIRST;
  its staging section B1-B8 is the plan of record):
  - B1 types.rs: the 20-primitive Type model, specialise, typedef
    table, byte-exact display. linear_values.rs: Vec32 (i32), Matrix
    (column-major), RatVec (i64/u64) with exact upstream printing.
  - B2a lex.rs: OperatorBecomes (`+:=`), `$`/`|`/`@` punctuation,
    all-20 PRIMTYPE names + `void`.
  - B2b grammar.lalrpop: the upstream TYPE grammar (conflict-free) +
    `type: expr` casts at top expression level; `*`/`|`/`->`/`void`
    as dedicated parser tokens.
  - B2c coercions.rs: all 29 coercions in registration order,
    is_close (3-bit), broader_eq. frames.rs: Rc frame chain, RefCell
    slots, scope-function restore, fresh-per-definition global cells.
    value.rs: Vector/Matrix/RatVector/Union variants.
  - B2c typed.rs (THE ACTIVE FILE): convert_expr for denotations,
    groups, casts, tuple/list displays (with row_coercion: mat
    context types display elements as vec), identifiers (Id_table
    with captured cells), operator calls via a-priori-type overload
    resolution (registry has int + - * / and unary -), conditionals
    with BALANCING, and/or/not desugared to conditionals. Conversion
    functions implemented: QI V[I] Qv[Q] Qv[I] QvV M[V] M[[I]] [I]V
    [Q][I], with the exact upstream narrowing error text.
  - Typed scalar registry is implemented and locally/HPC verified:
    integer/rational arithmetic, divmod, powers, complement, relations,
    string concatenation, hunger levels, and exact current-oracle errors.
    Scalar fixtures and capture harness are in `317e1a8`; typed
    implementation is `5fdfa7d`.
  - Domain adapter foundation is in `2921da5`: RootDatum provenance and
    `prefers_coroots`, Matrix/RatVec crossings, and square-matrix checks.
    The pre-typed dynamic evaluator retains one-argument constructor and
    nested-list compatibility until the pipeline swap.

### EXACTLY where work stopped

Mid-B2. The typed pipeline (typed.rs) exists IN PARALLEL with the old
dynamic evaluator (eval.rs) — nothing user-visible runs through it
yet. The next concrete steps, in order (all from the reviewed
AXIS_CORE_DESIGN.md staging section):

1. B2 remainder into typed.rs: pattern-less `let`, simple `:=`
   assignment, subscription + 1-D slices (incl. `~[`), `begin/end`
   grouping, `IDENT : expr` / `IDENT : type` commands. (All of these
   already parse and run in the DYNAMIC evaluator; they need typed
   conversion + evaluation arms.)
2. THE PIPELINE SWAP (the delicate step — the design's B2 section
   has the exact contract): run_source/execute_tokens route through
   convert_expr + TypedExpr::evaluate; delete eval.rs's inference
   machinery; re-register the ~15 domain builtins (domain_builtins.rs)
   in the typed overload registry WITH UPSTREAM SIGNATURES — this
   REWRITES the session gate tests in the same commit
   (`simply_connected(Lie_type("A1"),true)`; torus_factor returns
   ratvec with ratvec display; Lie_type only at string/RootDatum; the
   string inner-class sugar unregisters; domain `= !=` overloads must
   register or domain equality disappears). Add a corpus rerun to the
   gate (protect the 2 MATCHes). SessionEvent extends ONCE:
   `ReportLine { text }` + `is_void_type` on Value, frame = single
   formatter (see design "OUTPUT-SURFACE OWNERSHIP").
3. B3 functions: lambdas/closures (Closure/Builtin Value variants —
   Value's derived Eq BREAKS then; the design mandates manual
   PartialEq: structural data, Rc::ptr_eq closures, fn-pointer
   builtins), user calls, rec_fun, `return`, patterns, `.` selector
   (`x.f` == `f(x)`).
4. B4 definitions: set family + tables + report lines (byte-exact
   texts and indentation rules are in docs/AXIS_LANGUAGE_TRACE.md §5
   of the defs section; indentation ONLY on definition reports).
   CORPUS RERUN — deaths move off `set` into body features.
5. B5 set_type (simple = expanded alias, bracketed = tabled with the
   lexer's type_defining state — lexer-INTERNAL, see design) ->
   basic.at:3 falls. B6 loops/case/slices-2D. B7 builtin core (the
   inventory is in docs/AXIS_CORE_TRACE.md with basic.at's
   load-bearing minimum; B7's honest gate = basic.at through its
   scalar/RootDatum prefix). B8 = math-layer builtins joint with task
   #14 -> `<basic.at` loads end to end.

## Key documents map

- docs/AXIS_CORE_DESIGN.md — THE plan of record for phase B (staging
  B1-B8, all review corrections folded). Read first.
- docs/AXIS_CORE_TRACE.md — type analyser semantics + full builtin
  inventory (469 builtins, signatures, basic.at minimum).
- docs/AXIS_LANGUAGE_TRACE.md — grammar (parser.y productions),
  set/set_type/forget semantics, byte-exact session output surface.
- docs/AXIS_SESSION_FRAME_DESIGN.md — phase A as built.
- docs/KL_CHAIN_TRACE.md — blocks + KL recursion + dual side (task
  #14 substrate; block = (x,y) pairs with theta_y = -theta_x^t,
  fibred product over matched involution packets).
- docs/ON_DEMAND_PARTITION_DESIGN.md — task #9, PARKED, fully
  reviewed; resume only after porting breadth.
- AGENTS.md (repo root) — the older stage log and HPC workflow notes.

## Operational notes (learned the hard way)

- Toolchain: MacPorts shadows rustup. Use
  `TC=$HOME/.rustup/toolchains/1.90.0-aarch64-apple-darwin/bin;
  RUSTC=$TC/rustc PATH="$TC:$PATH" $TC/cargo <cmd>`.
- PIPE TRAP: `cargo clippy ... | grep/tail` loses the exit code (zsh
  pipeline status = last command). Two commits this session landed
  with clippy errors because of `... | tail && git commit`. Count
  errors (`grep -cE "^error"`) and READ the number before committing.
- HPC sync: `git archive HEAD | tar -x -C /tmp/atlas-sync && rsync -aq
  --exclude results/ --exclude target/ /tmp/atlas-sync/
  majj@10.26.14.64:~/atlas-rust/ && ssh ... sbatch hpc/<job>.sbatch`.
  Corpus job: hpc/script_corpus.sbatch; kgb battery:
  hpc/kgb_differential.sbatch; preflight: hpc/real_group_preflight
  .sbatch. Job logs land in ~/atlas-rust/atlas-*-<jobid>.out.
- Corpus run locally: build release atlas-cli, then
  `python3 hpc/script_corpus_diff.py /Users/hoxide/mycodes/
  atlasofliegroups/atlas target/release/atlas-cli` (driver runs the
  Rust CLI with cwd=atlas-scripts; upstream binary needs nothing
  locally). The HPC upstream checkout is a DIFFERENT snapshot (241
  scripts, no git) — its 2 OUTPUT_DIFFs are version artifacts, not
  regressions; the local oracle at master 4d3e9449 is authoritative.
- The Workflow tool (three-review pattern, traces) stalls
  occasionally: check the run's journal.jsonl if no notification
  arrives (one silent death this session); resume with
  {scriptPath, resumeFromRunId} — nothing is lost.
- lalrpop: the upstream parser.y ports conflict-free (verified by a
  reviewer under bison and empirically here for types/casts/if). When
  grafting more productions, follow the upstream shapes exactly; the
  current grammar's deviations (callable `()` atom, Group-vs-tuple)
  are scheduled for replacement at the swap, not incremental patching.
- Session gate tests to watch at the swap: session.rs
  `kgb_pipeline_is_scriptable_end_to_end` and
  `sp4r_kgb_sizes_match_the_oracle_through_the_language` (they call
  `simply_connected("A1")`-style 1-arg forms that MUST be rewritten
  to upstream signatures in the same commit as the re-registration).

## Task list state

- #8 in_progress — the axis language core (this handoff's subject).
- #14 in_progress — KL chain; trace done (docs/KL_CHAIN_TRACE.md),
  design not started; scheduled after the language core (B8 joins it).
- #9 pending/PARKED — on-demand Cartan discovery; design reviewed and
  committed; resume after breadth or when large-rank blocks need it.
