# Atlas-Rust handoff - 2026-08-01 (handoff to next coding agent)

This is the continuation record for `/Users/hoxide/mycodes/atlas-rust`.
The goal is source-compatible Atlas language behavior, with the upstream Atlas
executable and CWEB sources as the behavior oracle. The core remains safe Rust.

## Checkpoint - 2026-08-23c (overload-resolution hot spot found + first fix; corpus diff snippets)

- Perf root cause for the residual per-script gap (median ~4.6s vs oracle
  ~0.15s; NOT size-correlated: 93KB basic.at 0.17s, 604B GKfast.at 8.7s):
  gdb sampling via `hpc/perf_sample.sbatch` (jobs 3617887/3617888,
  GKfast.at + generic_degrees.at) shows all samples in
  `coercions::same`/`is_close` <- `typed::merged_variants` <-
  `convert_overload_application`, plus malloc/free churn (deep type
  clones). First fix landed: `659df32` caches merged overload variants
  per name in OverloadState (agent-111). Re-measure via corpus before
  deeper surgery (is_close expansion caching / clone reduction may
  remain).
- Corpus driver enhanced (commit `3f6dc19`): each OUTPUT_DIFF entry now
  carries `output_diff` (first divergent line, both versions, differing
  line count) and the report has an `output_diff_histogram` bucketing
  first-diff cpp line shapes — triage buckets come straight from one run.
- Corpus 3617878 (0ab4baa): MATCH 93 / OUTPUT_DIFF 143 / EVAL_FAIL 2 /
  SKIPPED 2. EVAL_FAIL down to 2i12.at (Hecke decompose assert,
  agent-110) and gl4H.at ("parameter not in the common block", fixed by
  agent-109 in `305d3a9`: route full-integral twisted KL/deform blocks
  through Rep_table-style lookup). example.at pars[7] crash was already
  cured by the Levi fix `a0cbcd9`; `bad6338` matches upstream
  PreRootDatum equality for RootDatum `=`. quick_check 3617890 green.
- Large scripts: the 2 SKIPPED_LARGE (~3MB E8 cell data) run separately on
  fat: `sbatch --partition=fat --mem=32G --export=ALL,SIZE_CAP=4000000,MEM_CAP_GB=24,TIMEOUT=1200 hpc/script_corpus.sbatch <paths>` (job 3617912).
- Ledger rows live in docs/BENCHMARKS.md ("Script-corpus ledger").

## Checkpoint - 2026-08-23b (fundamental_(co)weight ambient coordinates; induction_sp4 unblocked)

- Corpus failure: `induction_sp4.at` died at
  `Runtime error at basic.at:25:58: Levi factor is not theta-stable` from
  `real_Levi(KGB(G,9))`, G=Sp(4,R). Probe `hpc/probe_levi_sp4.at` (run
  against both interpreters, cwd=atlas-scripts) bisected the chain
  involution/rho/from_dominant/zero_simple_coroots/cross — all matched —
  to the FIRST divergence: `Levi_coweight(rd,[0])` gave `[1,2]` vs the
  oracle's `[1,1]`.
- Root cause: the `fundamental_weight`/`fundamental_coweight` builtins in
  `crates/atlas-core/src/domain_builtins.rs` returned the (co)root-BASIS
  coordinates (bare `e_i`, resp. the Cramer solution of `C x = e_i`)
  instead of AMBIENT lattice coordinates. Upstream
  (rootdata.cpp:849-853, 1012-1016) multiplies by the simple (co)root
  matrix: `weight_numer=(root_mat*iC.transposed()).columns()`,
  `coweight_numer=(coroot_mat*iC).columns()`. Simply connected data have
  simple coroots = `e_i`, so the old code coincided with upstream there —
  which is why `tests/fixtures/domain/fundamental.atlas` (sc A2/B2/A3
  only) stayed green. Fix commit `a0cbcd9`: solve `C^T y = e_i` (weights)
  resp. `C x = e_i` (coweights) via Cramer, then combine the simple
  (co)roots with those coefficients via the new
  `ambient_rational_combination` helper (folds the det(C) sign into the
  numerators so the RatVec denominator stays positive).
- Oracle anchors (Sp(4,R)): fw=[1,0]/1,[1,1]/1; fcw=[1,0]/1,[1,1]/2.
  (SL(3,R)): fw=[1,0]/1,[1,1]/1; fcw=[2,-1]/3,[1,1]/3.
- Verification: probe diff vs oracle + quick_check job (see entry body
  when filled). Convention note: `rsync -az --delete .git/` WIPES
  HPC-side worktree metadata under `.git/worktrees/` — recreate the
  metadata dir (`commondir`=`../..`, `gitdir`=<wt>/.git, `HEAD`=full sha)
  or `git worktree repair` afterwards when keeping a persistent build
  worktree like `/public/home/majj/atlas-rust-levi`.
- Verification results (fix commit `a0cbcd9`, binary built in
  `/public/home/majj/atlas-rust-levi/target`): every `Value:` line of
  `probe_levi_sp4.at` is now IDENTICAL to the oracle, including
  `Levi_coweight(rd,[0]) = [1,1]`, `has_theta_stable_Levi = true`, and
  `real_Levi(x) = disconnected split real group with Lie algebra
  'sl(2,R).gl(1,R)'`. Full `induction_sp4.at` loads with exit 0; the only
  remaining diff vs the oracle is the pre-existing set_type echo spelling
  (`ratmat`/`orbit_data`/`Std_Levi_table` name vs expansion), unrelated
  to this fix. quick_check job **3617819**: `CHECK_DONE status=0`.

## Checkpoint - 2026-08-23a (TYPE_ID lexer classification; basic.at loads to line 1721)

- Corpus-driven clearing of `basic.at` load blockers continues. Jobs
  3614515 → 3614589 → 3614650 → 3614661 trace the progress:
  - `b781734`: user-defined type names now lex as **TYPE_ID**
    (lexer.w:419-448): `TypedContext` owns a shared
    `Rc<RefCell<BTreeSet<String>>>` that `execute_set_type` fills and
    `forget` drains; both `session.rs` and `session_frame.rs` lexers see
    it. Grammar accepts `type_identifier` in `TypeExprNode` (casts and
    result annotations), `SpecTypeNode`, equation names, field names,
    `forget`, and `whattype` (parser.y:142,163-167,185-186).
  - `020bbdc`: `set_type` injectors/projectors join the **overload table**
    (`OverloadState::add_user`) instead of overwriting a global
    (global.w:1398-1410 uses `overload_table::add`; basic.at defines
    `solution` for three unions). Same `is_close` conflict rules as `set`.
  - `b8b973c`: **type_defining lexer state** (lexer.w:473-476): between
    `set_type [` and the command end every identifier lexes as TYPE_ID, so
    re-included files (double `<basic.at`) re-parse. Lambda parameter
    annotations, casts, op-casts, and rec-lambda result annotations now
    `resolve_in(&types)` instead of `resolve()` against an empty table
    (the `(*->*)` bug: `(maybe_a_vec x)` parameters stayed Undetermined).
  - `18ad7b5`: **tabled-type canonicalisation** (axis-types.w:1024-1051):
    `TypeTable::equivalent` is a coinductive structural comparison;
    `execute_set_type` pass 3 merges an equivalent new definition into the
    earlier number (`merged_into`), so double-included `basic.at` types
    stay identical to first-include bindings. `canonicalise_references`
    rewrites merged numbers in stored expansions; `lookup` follows the
    chain. Also: parenthesised-expression selectors `v.(f(x))`
    (parser.y:321+366), and the lambda conversion no longer panics when
    the required type is a tabled function type (specialise only CHECKS
    the expansion; lazy_lists.at hit the `unreachable!`).
  - `af928ae`: for-in over **KTypePol/ParamPol** iterates by term
    (axis.w:5926-5936 `index_kind` retries): index type KType/Param,
    component Split; `eval_for_loop` now pairs an index VALUE with each
    component instead of assuming positional int indices.
- basic.at progression: 592 (TYPE_ID) → 593 (injector overloads) → 659
  (case over user union) → 1626 (paren selector) → double-include type
  identity (634) → 1721 (`for x@q in P` over KTypePol). Next corpus run
  (3614661) shows what follows.
- `eval/user_type_cast.atlas` fixture + reference capture **3614600**
  committed; differential pending.
- Benchmark snapshot (3614515, 9 MATCH scripts): E8_small_block… 73.9x
  slower (3.917s vs 0.053s — real compute, investigate after corpus is
  green), ellipticExceptional 50.6x but millisecond-scale (startup),
  other 7 within 0.7x–2.9x.
- Fat differential **3614405** still PENDING (fat partition busy).

## Checkpoint - 2026-08-22g (while-let value contexts; corpus gate in flight)

- Full script-corpus run **3614308** (240 `atlas-scripts/*.at`): 9 MATCH,
  2 SKIPPED_LARGE, **229 RUST_EVAL_FAIL with one shared root cause** —
  `basic.at:204` `set_bit_positions` uses
  `while let pow = AND_NOT(n,n-1) in !=pow do n-:=pow; bit_length(pow)-1 od`
  in a `vec` (value) context. Our parse-time desugar rewrites while-let to
  `while true do (let … in if guard then body else break)`, and our `break`
  converted as void, so the branch balance (typed.rs `balance`) picked void
  as the common type and the loop typed `[void]`: `found [void] while vec
  was needed.`
- Fix (commit `900c8fb`): `break` now converts in ANY context without
  touching the required type, exactly like the upstream breaker
  (axis.w:673-685) and our existing `die`; `while` additionally implements
  the upstream void context (body against void) and int context
  (yields_count, make_while_loop flag 0x8 — only COMPLETED iterations
  count, a breaking iteration does not). New fixture
  `eval/while_let_values.atlas` probes both (`vec` result and `int`
  count). Existing `eval/while_let` only covered the void/statement
  position, which is why the gap was invisible.
- `deform` in statement position silently skips per the upstream no-value
  gate (atlas-types.w:8085-8087); registered as Skip like `block_deform`
  (commit `fb09851`), fixture `domain/deform_no_value.atlas`, reference
  capture **3614307** PASS (`verified_hpc_reference`, meta committed in
  `3da0aef`).
- Benchmark convention reminder: every differential/corpus report carries
  `benchmark_summary` (rust_to_cpp_seconds, maxrss ratio); present the
  numbers to the user whenever a report lands.
- In flight at handoff: preflight **3614439**, reference capture
  **3614440** (`eval/while_let_values`), full corpus rerun **3614441**,
  fat fixture differential **3614405** (covers `deform_no_value`; queued
  before the break fix, so it uses the previous binary).
- NOTE: `crates/atlas-real-group/examples/fiber_probe.rs` is user-owned and
  must never be committed; it is now gitignored after an accidental
  `git add -A` sweep was reverted (`fe0ebf0`).

## Checkpoint - 2026-08-22b (generic operator casts HPC-verified)

- `op@type` now implements the upstream generic-special fallback for `print`,
  `prints`, `to_string`, `error`, `#`, and `##`, including nested wildcard
  capture names (`T`) and the existing exact-overload precedence. A repair
  pass now uses strict cast-shape predicates for `#`/`##`, rejecting wildcard
  row/scalar combinations the oracle rejects, and renders `##@[[int]]` as
  `{##@([[T]])}`. The RED/GREEN regression is
  `typed::tests::operator_casts_select_generic_special_instances`; local CLI
  output matches the upstream executable for both positive and rejected probes
  in `eval/op_cast_specials{,_rejected}`.
- Positive and rejected fixture references are HPC-captured by job
  **3613892** (`verified_hpc_reference`, exact binary pin, live submit-tree
  provenance); CPU differential **3613996** reports **349 fixtures,
  runnable PASS** and both new fixtures PASS exactly (whole-run PARTIAL only
  for the pre-existing declared harness cases). The
  `partial_kl_block_proper` differential **3612366** is complete and PASS for
  that fixture; its metadata now records the report SHA256.
- HPC preflight **3613988** passed on `cu023` with Rust 1.96 and exit 0;
  report SHA256 `6e802cd5667432348756a320bb6fe42e371788dcbb362fb5b9cef427f846c3c4`.
  Differential report SHA256 is
  `0d8da8638583cb25242c7e69af86ec18a81597ff850388dd280af709270217e6`.
  The existing untracked `crates/atlas-real-group/examples/fiber_probe.rs`
  remains user-owned.

## Checkpoint - 2026-08-22c (string slices HPC-verified)

- The upstream script corpus workload **3614053** ran six scripts with the
  fixed GCC 12.1 runtime: C++ exit 0 for all, and the first Rust blocker was
  `Cannot slice value of type string` while loading `basic.at`. This exposed
  the missing string slice family in the Rust typed evaluator.
- `Value::String` one-dimensional slices now support byte-length half-open
  ranges, from-end bounds, reverse ranges, and exact range/type diagnostics.
  Reference capture **3614066**, preflight **3614104**, and CPU differential
  **3614103** all pass; differential has 351 fixtures and 3 pre-existing
  declared pending cases, with both string-slice fixtures exact.
- The corpus harness itself now binds `/public/software/gcc/gcc-12.1` directly
  and records C++/Rust stderr and exit status (`a6a8549`), so missing
  `GLIBCXX_*` cannot be misclassified as a language mismatch.

## Checkpoint - 2026-08-22d (operator value aliases HPC-verified)

- Corpus rerun **3614111** removed the string-slice blocker and exposed the
  next parser gap: upstream `basic.at:21` uses `set ^ = !=@(bool,bool)`.
- Added the operator-value alias grammar/action (`parser.y:151-153`), which
  stores an existing function value directly in the operator overload table.
  Reference capture **3614131** and differential **3614146** PASS; the latter
  has 352 fixtures and 3 pre-existing declared pending cases, with the new
  `operator_alias` fixture exact. A clean-output HPC preflight **3614162**
  also passed (report SHA256
  `1dfde96bf955370bc3b8fa9cc4b7c6a7b19504335fdc584f16e5acf626dd36ac`).

## Checkpoint - 2026-08-22e (typed lambda result annotations HPC-verified)

- Corpus rerun **3614180** moved past operator aliases and exposed
  `(int i)bool: ...` result annotations in `basic.at`. The parser now lowers
  typed lambda results through the existing cast path.
- Reference capture **3614208**, preflight **3614209**, and differential
  **3614211** pass; the differential has 353 fixtures and 3 pre-existing
  declared pending cases, with `typed_lambda_result` exact.

## Checkpoint - 2026-08-22a (partial_KL_block LocatedBlock rewrite FIXED; differential 3612366 IN FLIGHT)

Branch = **main = `6c5a081`**, pushed + HPC-synced. The 2026-08-21j P0 is
done: `partial_KL_block(Param)` now goes through `RepTable::lookup`
(`LocatedBlock` partial block + shared KL table + modifier-aware survivor
parameters via `located_row_parameter`/`located_singular_flags`).

- **Root cause of the proper-case gap** (found via oracle capture 3612358
  of the new fixture `domain/partial_kl_block_proper`): BOTH the old
  full-block path and the first WIP used a cross/Cayley DESCENT closure
  for the partial subset, but upstream (atlas-types.w:7010-7014) takes the
  **Bruhat downward closure** of the start element (`subset.back_up` over
  the Hasse diagram). Second fidelity fix: the index matrix is
  identity-initialised with only the strict upper triangle grouped into a
  pool preseeded `{0, 1}` (atlas-types.w:6957-6971) — not a full-matrix
  first-seen grouping.
- Fixed arm lives at domain_builtins.rs:14784 ("partial_KL_block"); dead
  `block_finals_for` removed. Local gates: clippy/fmt clean; fixtures
  partial_kl_block, deform_proper, twisted_deform_proper(+_terms),
  full_deform_proper, twisted_full_deform_proper all PASS locally; the new
  fixture's events/meta generated from capture 3612358 and PASS locally.
- Formal verification: fat differential **3612366** submitted; on PASS set
  the meta's differential_job and record here.
- Old meta note to retire on PASS: partial_kl_block.meta.json still says
  the A2 x=3 proper case "needs the common-block srm descent statuses …
  removed from the fixture" — that limit is now covered by
  partial_kl_block_proper.

## Checkpoint - 2026-08-21i (filekl landed; formal differential 3608036 IN FLIGHT; main = `c19c755`)

## Checkpoint - 2026-08-21j (ordinary deform proper lookup verified)

- Ordinary `deform(Param)` no longer rebuilds a full dual `BlockGraph` for
  each final. It now uses `RepTable::lookup` and the existing
  `common_deformation_terms` partial-block algorithm, preserving proper
  subsystem descent, locator modifiers, and pooled KL state.
- Fixture `domain/deform_proper` covers B2 and A2 proper-subsystem parameters.
  Reference capture **3611271** and fat differential **3611540** pass; the
  latter reports 344 PASS plus the two pre-existing harness PARTIAL cases.
- The timed corpus driver `hpc/script_corpus_diff.py` now records C++/Rust
  wall time, peak RSS, timeout state, and ratios. A first seven-script run
  completed as measurement-only because those definitions-only scripts make
  the upstream process exit with its own load errors; use explicit call
  workloads for algorithm benchmarks.
- Next P0: replace `partial_KL_block(Param)`'s full-block/manual closure with
  `LocatedBlock` partial lookup, shared KL table, partial singular/final
  helpers, and modifier-aware survivor parameters. Then run the same
  fixture-first capture/differential loop.

Branch `codex/continue-atlas-port` = **main = `c19c755`**, pushed and
HPC-synced. Merged differential **3607276 (fat): 343 PASS + 2 declared
PARTIAL** (container_syntax_errors dangling paren, for_reversed_extra
trailing tilde — both harness artifacts registered as PendingCase).
Language matrix: **347/347 metas verified_hpc**; the ONLY remaining
LANGUAGE.md row was the KL binary file formats (filekl.w) — now implemented:

- `crates/atlas-real-group/src/filekl.rs` (c19c755): block/matrix/KL-store
  readers+writers plus read-only progress reader, byte-faithful to filekl.w
  (LE, magic 0x06ABDCF0, 5-byte store offsets, degree>=32 read rejection;
  upstream master kl.cpp:223-232 prim_map regression deliberately NOT
  replicated — canonical format keeps the y bit). 15 in-file unit tests,
  clippy/fmt clean.
- `crates/atlas-real-group/tests/filekl_dump.rs`: `#[ignore]`d dump driver
  (pub API only; quasisplit form via `ExternalFormOrder`, dual quasisplit —
  the interpreter's `full_block_of` recipe). Dumps A1(3)/A2(6)/B2(12)/G2(12)
  blocks + JSON expectations when `FILEKL_DUMP_DIR` is set.
- `hpc/filekl_diff.py` + `hpc/filekl_diff.sbatch`: KLread-oracle
  differential. KLread output format facts (KLread.cpp main loop): poly
  terms print DECREASING degree joined by " + ", `q^{n}` for degree>=10,
  trailing `.`; `value at q=1` only for non-constant polys; zero-polynomial
  queries (triangularity / real non-parity) go to STDERR as errors, so the
  driver counts "no output + zero expectation" as pass. `-q` suppresses the
  `rank=..., block size=...` header — do NOT pass it.
- **Rehearsal already PASS 4/4 (183 pairs)**: locally against a fresh
  KLread build (KLread.cpp is self-contained: `clang++ -std=c++17
  KLread.cpp`) and on HPC against
  `/public/home/majj/atlasofliegroups/sources/stand-alone/KLread` with
  locally-dumped files (LE format is cross-platform).
- Formal HPC artifact: job **3608036 COMPLETED exit 0** (cpu, 1m35s,
  `results/c19c755…/3608036/filekl_report.json`): 4/4 blocks PASS,
  183/183 polynomial pairs. LANGUAGE.md KL row promoted to `supported`
  (`7c8c319`). 取舍 recorded: KLread-only oracle (matstat/polstat do not
  link — missing rootdata/repr symbols — and are not needed).
- Goal-completion audit next: matrix zero pending rows (verified), all
  metas verified_hpc, `cargo test -p atlas-core --lib` + clippy/fmt clean.

## Checkpoint - 2026-08-21b (readline_completions slice IN FLIGHT; main = `60c248f`)

Branch `codex/continue-atlas-port` = **main = `60c248f`**. Regression
differential **3604363 @ 08e37c1 (fat): runnable_status PASS** (330 fixtures,
only the declared container_syntax_errors PARTIAL — harness-convention
artifacts, not language defects).

Correction to the 2026-08-21 registry-audit note below: `readline_completions`
is NOT TTY-only — it is an ordinary `string->[string]` builtin callable in
batch mode, so it is in scope for the language gate. Its semantics
(buffer.w:1175-1192): prefix match over `main_hash_table` insertion order —
keywords, primitive type names, builtins in upstream registration order (294
startup names), then session globals/user overloads in first-definition order;
`forget` removes, redefine-after-`forget` revives at the ORIGINAL position
(hash codes are never recycled). The audit also missed three startup SYSTEM
VARIABLES (not builtins): `input_path`/`prelude_log`/`back_trace`, all
`[string]`, defined at startup (main.w:408-435); `prelude_log` is const.

Slice state:
- Fixtures `eval/readline_completions{,_rejected}` committed (`6df50f5`);
  HPC capture **3604377** done; accepted events/meta generated and committed
  (`60c248f`, verified_hpc_reference). Startup name order captured verbatim in
  `/tmp/rlc_full_list.txt` (297 names; 294 static + 3 system variables).
- Implementation delegated (subagent): new `BuiltinImpl::Completions`,
  `STARTUP_COMPLETION_NAMES` const (294), session `completion_order` tracking
  in TypedContext (append-only, skip startup names), candidates refreshed into
  EvaluationContext at execute() top, system variables defined in
  TypedContext::new (prelude_log const), plus the const-override wording fix
  (` (constant)` suffix in define_variable's override report, oracle-verified).
- Remaining in this slice: rejected-fixture events/meta (generator
  `/tmp/gen_readline_events.py`, needs the implemented CLI for its
  oracle-vs-CLI diagnostic assertion), FIXTURE_PLANS registration, fat
  differential, then LANGUAGE.md row 39 → supported and REMAINING_BUILTINS
  notes cleanup.
- Follow-up slice (research delegated): `back_trace` runtime-error call-trace
  population ("In call of g@int at <span>, defined at <span>." + "{ x=2 }"
  frame dumps + builtin "built-in." lines, global.w:1100-1140) — no trace
  machinery exists in the Rust evaluator yet.
- Still deferred pending user decision: KL binary file formats (filekl.w,
  no language builtin touches them).

## Checkpoint - 2026-08-21 (locator attitude slice RESOLVED; main = `e10de93`)

Branch `codex/continue-atlas-port` = **main = `e10de93`** (the whole slice
landed on main after differential 3603961 PASS).

### Resolved this session

- Differential **3602066 @ 8f5151e (fat): 328 PASS + 1 declared PARTIAL
  (container_syntax_errors, its two pre-existing pending_features), 0
  FAIL**. `domain/polp_coercion` + `domain/torus_rank0` metas bumped to
  verified_hpc with differential_job 3602066 (commit `66ae893`).
- WIP `2717af2`: non-identity locator-attitude wiring (build_partial
  cofolding, lookup merge transport via `State::overlap_hits` +
  make_relative_to + shift_srm/transform_srm, 3 language gates removed).
- `f9dd1a4` ROOT CAUSE of the remaining print divergence: upstream
  `twisted_KL_column_at_s` (repr.cpp:2378-2382) and the
  `twisted_deformation` reducibility loop (repr.cpp:2605-2606) use the
  PARTIAL `Rep_table::lookup`, so deform pools the small Bruhat-interval
  block. `with_integral_block`'s ProperSubsystem arm used
  `lookup_full_block` (slice-plan choice), pooling a full block at a
  different attitude; later `print_partial_common_block` calls then hit a
  record the oracle never has. Switching the arm to `lookup`
  (domain_builtins.rs) fixed ALL divergences: probes lA-lE +
  probe_locator/probe_locator2 byte-identical to the oracle, covering
  deform-then-print in both orders and the mixed KL/full-deform battery.
  Note the oracle's printed gamma-lambda depends on which locator created
  the pooled record (shift-only print over attitude-stored rows,
  atlas-types.w:6726-6732) — pool shape is observable, not just values.
- New fixture `domain/locator_attitude` (registered in
  pipeline_swap_diff.py): B2 split form 2, twisted_deform then
  cross-attitude prints, twisted_KL_sum_at_s, twisted_full_deform both
  params, closing prints. Locally byte-identical to oracle.
  **HPC capture 3603952 IN FLIGHT** — then generate events/meta
  (template /tmp/gen_polp_coercion_events.py), run fat differential, bump
  to verified_hpc.
- Gates: atlas-real-group 479/479, atlas-core 329/329, clippy -D
  warnings, fmt; 29 twisted/partial/block fixtures locally byte-identical
  (3 timed_* rejected fixtures differ only in the known accepted
  diagnostic rendering — they PASS structurally on HPC).

### Next steps

1. ~~Collect capture 3603952~~ DONE: differential **3603961 @ 6384b05:
   329 PASS + 1 declared PARTIAL**, locator_attitude verified_hpc, and the
   whole slice pushed to **main = `e10de93`**.
2. Registry audit (agent-94, 2026-08-21): all 469 upstream `(name,args)`
   builtin pairs + 29 coercions ported; 0 missing overloads; no reachable
   NYI gates. Only deliberate exclusions remain: `readline_completions`
   (TTY-only) and KL binary file formats (no language builtin touches
   them) — both deferred pending a user decision. The language-level port
   is effectively COMPLETE; remaining work is semantic hardening (larger
   differential corpora, more groups/types) rather than missing features.

## Checkpoint - 2026-08-20 morning (differential 3591705 all PASS; corpus 315+)

- Fat differential **3591705 @ 722c05c: 325 PASS + 1 declared PARTIAL
  (container_syntax_errors), 0 FAIL**; report SHA256
  `c4285021b5799e373d8dc26c5f590982d77a2e9b5c43ea2e8e6bcd8e6de8b733`.
  Four metas bumped to verified_hpc: twisted_deform_proper,
  twisted_deform_proper_terms, twisted_deform_proper_rejected (capture
  3591165), twisted_full_deform_proper (capture 3586686).
- full_deform_proper events/meta frozen from capture 3586752 and verified by
  fat differential 3599345 (`bc94a31`).
- Wave A (non-integral common block work order) confirmed FULLY landed:
  `length(Param)` reroutes through `rep.lookup(&dominant)`
  (domain_builtins.rs:13970-13991), `dual_KL_block` via lookup_full_block
  + BareBlock dual (:14566+), `print_partial_common_block` shared-lookup
  with both headers (:10700+); fixtures length_dual_proper{,_a2},
  print_partial_common_block_seq, print_partial_block_proper all
  verified_hpc (3583557/3574934). The `common_block_rows` non-integral
  gate (:9834-9871) remains only as the rank-0/Singleton arm + loud NYI —
  workorder item 5 cleanup not done.
- print_gradings/print_real_Weyl/print_blockstabilizer wrappers are
  IMPLEMENTED (domain_builtins.rs:10270-10303, real_weyl.rs) — the old
  "RealWeyl 已移植但缺 wrapper" note is stale.
- agent-93 (read-only audit of genuinely-remaining items) in flight; use
  its report to re-plan the queue — the HANDOFF/REMAINING_BUILTINS queues
  are partially stale.

## Checkpoint - 2026-08-20 (`twisted_full_deform` slice 5 local)

## Checkpoint - 2026-08-20 (`full_deform` common-block recursion local)

- Ordinary `full_deform` reducibility recursion now uses `RepTable::lookup`
  and a partial/common-block deformation algorithm. The old full-block
  reconstruction retained rows above the lookup interval and produced two
  spurious B2 `[13]` terms.
- `common_deformation_terms` ports the upstream singular contributions,
  partial KL table, q=-1 accumulator, per-row `BlockModifier` transport,
  and orientation correction. The B2 anchor fixture is
  `tests/fixtures/domain/full_deform_proper.atlas`; local output is
  byte-identical to the oracle for integral, half-integral, and non-final
  parameters.
- Commits: `444e841` (fixture), `466e066` (algorithm), `b267f2a` (export).
  Local crate tests, clippy, fmt, and the focused deformation regressions
  pass. HPC capture 3586752 and differential 3599345 pass. Keep
  `fiber_probe.rs` untracked (user-owned).

- `twisted_full_deform` reducibility recursion now uses
  `RepTable::lookup` interval-below partial blocks for both Full and
  ProperSubsystem scopes, matching repr.cpp:2605. Rebuilding a full block
  at a Full-scope reducibility point caused the B2 anchor's two spurious
  `[13]` terms.
- `scaled_extended_finalise` now scales `RepContext::nu(sr)`, not `gamma`,
  while preserving `lambda_rho`; the old expression only worked when
  `lambda_rho == 0`.
- `tests/fixtures/domain/twisted_full_deform_proper.atlas` is byte-identical
  to the local oracle for x=5 at integral/half-integral nu and non-final x=10.
  Local gates: atlas-core 329/329, atlas-real-group 478/478, clippy and fmt
  clean. HPC reference capture, fixture registration, fat differential, and
  `verified_hpc` metadata remain pending.
- Oracle correction: x=10 is accepted by `twisted_full_deform`; do not create
  a rejected fixture for it.

## Checkpoint - 2026-08-19 late night (handoff mid-slice; UNCOMMITTED work in tree)

Branch `codex/continue-atlas-port` (push to main too). Pushed HEAD =
`c17b874`. The working tree carries TWO lines of uncommitted work —
do NOT mix them in one commit.

### A. vec/ratvec/mat subscription read/write (typed.rs + linear_values.rs) — MINE, nearly done

State: implementation COMPLETE and compiling (`cargo check`/`cargo build -p
atlas-cli` clean); the oracle comparison battery ran and **every message text
matches verbatim** — the only diff lines are the known diagnostic-frame
formatting divergence (Rust prints `Type error at <stdin>:L:C:` + underline,
the differential harness normalizes this). Battery script:
`/tmp/vecmat_battery.atlas` (rewrite from the git notes if /tmp was cleaned),
oracle output `/tmp/vecmat_oracle.txt`, rust `/tmp/vecmat_rust.txt`.

Still to do for this slice, in order:

1. Add the unit test after `typed.rs:11625` (helpers `convert_and_run_with`,
   `crate::frames::global_with`; values `Value::Vector(Vec32(vec![..]))`,
   `Value::RatVector(RatVec::new(vec![1,2],2).unwrap())`,
   `Value::Matrix(Matrix::from_columns(2,2,vec![1,3,2,4]).unwrap())` —
   column-major, so M=[[1,2],[3,4]] is data [1,3,2,4]). Matrix Display is a
   padded grid — assert via `matrix.entry(r,c)`, not to_string. Cover:
   reads `v[0]/v~[0]/rv[0]/rv~[1]/M[0]/M[0,1]=3/M[1,0]=2/M~[1,0]=3`;
   writes `v[0]:=7`, `v[0]+:=2`, `M[1]:=[9,9]`, `M[0,1]:=9`, `M[1,1]+:=10`;
   and these oracle-verified messages:
   - `index 5 out of range (0<= . <3) in subscription v[5]` (also rv; mat
     read column: `… in matrix column selection M[5]`; mat pair read:
     `initial/final index … in matrix subscription M[0,5]` — pair NO parens)
   - assignment: `in component assignment v[5]:=1`,
     `in matrix column assignment M[5]:=V[I]:[1,2]` (conversion tag prefix;
     `M[5]:=v` keeps plain `v`),
     `initial index 5 out of range (0<= . <2) in matrix entry assignment M[(5,0)]:=1` (pair WITH parens)
   - transform range checks fire on the synthetic READ: vec `in
     subscription v[5]`, mat column `in matrix column selection M[5]`, mat
     pair `in matrix subscription M[5,0]`
   - type errors: `Cannot subscript value of type ratvec with index of type
     int in assignment` (ratvec is READ-ONLY), `… mat … (int,string) in
     assignment`, `… mat … (string,int)`, `… vec … (int,int)`
   - `Cannot replace column of size 2 by one of size 1`;
     `M[0] +:= [1]` fails earlier with `Size mismatch 2:1` (existing, do
     not rewrite); `M[1] *:= [2,3]` → `found int while vec was needed.`
2. `cargo test -p atlas-core --lib` — NOTE: agent-91's WIP test
   `twisted_deform_proper_subsystems_match_oracle` FAILS in the tree; that
   is its normal intermediate state, do not "fix" it, only check your own
   tests. Then clippy `-D warnings` + fmt.
3. Update the `docs/REMAINING_BUILTINS.md` entry (~line 284) "vec/mat
   component assignment… shares the unimplemented vec/mat subscription gap"
   to FIXED.
4. Optional fixture pair (e.g. `tests/fixtures/eval/vec_mat_subscription{,_rejected}.atlas`)
   needs HPC capture; event-generator template `/tmp/gen_combined_twisted_events.py`.
5. Commit ONLY `crates/atlas-core/src/typed.rs
   crates/atlas-core/src/linear_values.rs docs/REMAINING_BUILTINS.md`
   (+ fixtures if made). NEVER `git add` domain_builtins.rs / deform.rs /
   the twisted_deform_proper* fixtures — those are agent-91's. NEVER commit
   `crates/atlas-real-group/examples/fiber_probe.rs` (user file).

### B. agent-91 twisted slice 4 (twisted_deform) — COMPLETE, pending commit + HPC

agent-91 finished; its uncommitted files are final:
`crates/atlas-core/src/domain_builtins.rs` (twisted_deform dispatch
~16647 drops the Full-or-NYI guard, passes `&parent` through to the
slice-3 `ProperSubsystem` arm of `with_integral_block`; dead
`proper_subsystem_diagnostic` removed), `crates/atlas-real-group/src/
deform.rs` (`twisted_deformation_terms` now takes `parent: &KlSumParent`;
per-row lambda_rho via `KlSumParent::sr` on Partial parents), and
untracked fixtures `tests/fixtures/domain/twisted_deform_proper.atlas`,
`_terms.atlas` (q2 = `param(KGB(rfb,10),[0,0],[1,1]/2)`, non-empty terms),
`_rejected.atlas`. Reported gates: 328+477 tests pass, clippy -D warnings
clean; spot-check confirmed
`twisted_deform_proper_subsystems_match_oracle` ok. All three fixtures
verified IDENTICAL to the local oracle (rejected differs only in location
wrappers).

 Harvest commands: full gates (`cargo test -p atlas-core --lib`,
`cargo test -p atlas-real-group --lib`, `cargo clippy --workspace
--all-targets -- -D warnings`, `cargo fmt --check`) → commit ONLY its
files → push → HPC sync (`rsync -az --delete .git/
ikkemhpc:/public/home/majj/atlas-rust/.git/ && git archive HEAD | ssh
ikkemhpc 'cd /public/home/majj/atlas-rust && tar -xf -'`) →
reference_capture the trio (oracle sha256 66f5d7d4…65c9, dirty=false) →
register → local run_fixture → fat differential (`sbatch --partition=fat
--time=01:00:00 --mem=32G --export=ALL,TIMEOUT=3600
hpc/pipeline_swap_diff.sbatch`) → verified_hpc + HANDOFF.

Record in REMAINING_BUILTINS.md (from agent-91's report):
1. **"alcove-wall closure overshoot → NDEBUG truncation" divergence**:
   gamma=[1,0]/2 on the top alcove wall in non-simply-laced data —
   `int_item`'s additive_closure overshoots to the full B2 datum
   (rootdata.cpp:685-707 does the same); upstream's `codec::internalise`
   assert (repr.cpp:104) is compiled out under the oracle's -DNDEBUG
   build and silently truncates 3/2→1, while the Rust IntegralCodec
   honestly rejects. Affects slice-3 paths identically; pre-existing, not
   a slice-4 gap.
2. q2 cannot share a fixture session with pb: same block record interned
   under different locators trips the locator gate
   `has_identity_generator_attitude` on the second lookup (oracle handles
   it via non-trivial `block_modifier`); hence the separate `_terms`
   fixture.

### Queued after that

- compatible_outer_twist coroot wording: `domain_builtins.rs:7763-7777`
  `based_involution_twist` mapping leaks
  `StructureError::SimpleCorootImageMismatch` into `other.to_string()`;
  mirror `twisted_involution_diagnostic` (:7691) via
  `atlas_root_number(handle,&image_root,span)` → "Matrix does not map
  simple coroot N to coroot M". Touch only when domain_builtins.rs is free.
- Twisted slice 5 (twisted_full_deform proper-subsystem recursion): recon
  landed at `docs/slices/twisted_full_deform_slice5_recon.md` (c17b874).
  Key trap: no swallow port needed; DeformParent enum + closure-side
  singular orbits; the anchor `param(KGB(rfb,5),[1,1],[1,0]/1)` currently
  does NOT hit the NYI and silently mis-computes four terms with `s` where
  the oracle yields two without.
- Remaining queue as listed in the previous checkpoint (next-wave A
  non-integral common block = largest item; B full_deform; C/E/F; locator
  step 5; `#:=` parser gap).

## Checkpoint - 2026-08-19 night (op:= + twisted slice 3 verified; alias declaration fixed)

- Frozen corpus now **311/311 verified_hpc** (was 307/307): the four new
  fixtures below all VERIFIED by fat differential **3585678** @ `ef395f3` —
  321 PASS + 1 declared PARTIAL (container_syntax_errors), 0 FAIL; report
  SHA256 `019e223c99f5293c6defdc65f7f4b5434aa72253b4f82231d90c545b82947b15`.
- op:= assignment family landed (`80518bd`): component/field assignment
  `a[i]:=v` (incl. `~[`) and `p.f:=v`, component/field transforms
  `a[i] op:= v` / `p.f op:= v`, and bare `x op:= e` desugared in the parser
  to `x := op(x,e)` (parser.y:263-278; axis.w:7736-8546 evaluation order).
  Grammar routes targets through identifier-anchored `Postfix` productions
  (upstream `assignable_subsn` prefix sharing). Fixtures
  `eval/combined_assignment{,_rejected}` (capture **3585649**), both metas
  `verified_hpc`. Known divergences documented in
  docs/REMAINING_BUILTINS.md (converted-call wording on out-of-range
  transforms, bison `expecting` list, two-index subscription, vec/mat
  component writes).
- Twisted slice 3 landed (`63e8118`): `twisted_KL_sum_at_s` both overloads
  on proper-subsystem gamma — `with_integral_block` gains the
  `ProperSubsystem` arm via `RepTable::lookup_full_block` + partial
  `ExtBlock`; `twisted_kl_sum`/`twisted_kl_column_at_s` generalised
  (atlas-types.w:8370-8382/8420-8431 → repr.cpp:2371-2423/2304-2350).
  Fixtures `domain/twisted_kl_proper{,_rejected}` (capture **3585649**),
  both metas `verified_hpc`.
- `set_type` alias declaration gap FIXED (`ff5c518`): `p: Pair` now
  declares — a bare-identifier `Command::Define` right side naming a known
  type is re-routed to the declaration path (mirrors parser.y TYPE_ID
  lexing). Residual divergence (`set Pair = 5` accepted, upstream says
  "unexpected TYPE_ID") recorded in docs/REMAINING_BUILTINS.md.
- In flight: agent-91 — twisted slice 4 (`twisted_deform` on
  proper-subsystem gamma; `twisted_deformation_terms` + partial-aware
  `singular_orbits_at`; workorder lines 127-129).
- Queue after slice 4: slice 5 (twisted_full_deform recursion, may force
  KL_table::swallow/partial merge) → next-wave A (non-integral common
  block, domain_builtins.rs:9431 gate) → C (KL_sum_at_s lambda-rho) → B
  (full_deform scope check) → E/F (Weyl_orbit size, integrality_points
  display) → locator step-5 (print_partial_common_block attitude +
  ext-block simple_pi).

## Checkpoint - 2026-08-19 late (twisted slice 2 verified; op:= + slice 3 in flight)

- Frozen corpus now **307/307 verified_hpc** (was 305/305 at `45acc32`).
- Twisted slice 2 landed and verified: `raw_ext_KL` +
  `partial_extended_KL_block` on proper integral subsystems
  (`ExtKlTable::fill_columns` + `ext_kl_matrix`/`condense` over the
  partial-parent ext block; `CommonContext::singular_flags` replaces the
  hand-rolled coroot loop). Implementation `d382014`; fixtures
  `domain/ext_kl_proper{,_rejected}` registered `89fe5a7` (capture
  **3585276**); VERIFIED by fat differential **3585343** @ `89fe5a7` —
  317 PASS + 1 declared PARTIAL (container_syntax_errors), 0 FAIL; report
  SHA256 `95b7eb23265d8c8924529169b7a10402ab4ab6bc0c741e93f322661539414edf`.
  Both metas `verified_hpc` (`59d4486`). Deferred (pre-existing, gate
  fidelity): `compatible_outer_twist` renders `SimpleCorootImageMismatch`
  via Display instead of upstream's "Matrix does not map simple coroot N to
  coroot M" wording.
- In flight: agent-89 — `op:=` OPERATOR_BECOMES compound assignment
  (lexer.w:507-516; parser.y:263-278 three productions:
  IDENT/assignable_subsn/field). Oracle ground truth probed by orchestrator:
  pure desugar `x := op(x,e)` with static-type equality (a[2] /:= 2 on
  [int] rejected "found rat while int was needed"); yields the NEW value;
  vec append `v #:= 3` and concat `v ##:= [4,5]` work; row selector `M#0`
  is NOT an assignment target (syntax error); no string special-casing.
  agent-90 — twisted slice 3 (`twisted_KL_sum_at_s` both overloads,
  `with_integral_block` ProperSubsystem arm; workorder lines 120-127).
- Queue after these: twisted slices 4 (twisted_deform) and 5
  (twisted_full_deform recursion, may force KL_table::swallow/partial merge);
  then next-wave A (non-integral common block, domain_builtins.rs:9431 gate)
  → C (KL_sum_at_s lambda-rho) → B (full_deform scope check) → E/F
  (Weyl_orbit size, integrality_points display) → locator step-5
  (print_partial_common_block attitude + ext-block simple_pi).

## Checkpoint - 2026-08-19 (locator anchors frozen; global.w batches in flight)

- Signature reconciliation (docs/REMAINING_BUILTINS.md 2026-08-18 entry,
  commit `387c3c4`): all 305 `atlas-types.w` signatures are registered in
  Rust; the only registry gap is `global.w` (89 signatures), plus the hard
  math gaps (generator-attitude gates, twisted/ext proper recursion,
  non-integral common blocks, cross-block partial merge).
- global.w batch 1 landed (commit `15a3292`): rat `floor`/`ceil`/`frac`,
  string `##`/`ascii` x2, `#` on string/vec/ratvec/mat (mat = column count),
  matrix `shape`/`row`/`column`/`rows`/`columns` (`rows`/`columns` return
  `[vec]`, not `int`). Reference frozen by capture **3574819**; VERIFIED by
  fat differential **3574838** @ `447fe44` — 289 PASS + 1 declared PARTIAL
  (container_syntax_errors) across 290 fixtures, both `global_batch1`
  fixtures exact; report SHA256
  `3006c3a8dcbd6339075274ca997c08410424e6cd6a698ee6de9125e584fcc58e`.
  Both metas are `verified_hpc`. (The first cpu-partition run 3574831 failed
  only `domain/kgb_hasse` on an environmental timeout — heavy full-suite
  differentials belong on `fat`, per the standing HPC note.)
- Locator slice anchors frozen (all intentionally UNREGISTERED from
  FixturePlan until the locator lands — current identity-attitude code
  diverges silently): `domain/common_block_locator` (A2 SL(3,R),
  `as transformed by <1>`, capture 3574723, commit `d93929a`),
  `domain/common_block_simple_pi` (A3 SL(4,R) rank-two,
  `as transformed by <0.2>, simple reflections permuted (0->1,1->0)`,
  capture 3574819, fixture commit `a732a27`), and
  `domain/common_block_rank0_locator` (A2 rank-zero, `<0.1.0>`, capture
  3574845, commit `3dd4b73`).
- Locator step 1 landed (commit `79b6b9d`): `BlockLocator`,
  `IntegralDatumTable`, and `int_item` (innerclass.cpp:1116-1182) as pure
  unwired functions in `atlas-real-group/src/locator.rs`, with
  `root_vertex_of_alcove` in alcove.rs. Key subtlety (step-1 report):
  `int_item` keys on the on-wall closure of the DOMINANT ALCOVED
  representative, so it is alcove-dependent — Weyl-conjugate gammas share
  the item, same-integral-system gammas on different alcove walls do NOT.
  461 lib tests pass, clippy/fmt clean (dev+release).
- Locator implementation route (do not reopen; full brief at
  docs/slices/locator_integration_brief.md): step 1 pure
  `InnerClass::int_item` canonicalization + `BlockLocator` interning
  (DONE, `79b6b9d`); step 2 `RepContext::transform` + `shift` +
  `make_relative_to` (repr.cpp:338-350) + `sr(srm,bm,gamma)`
  (repr.cpp:815-823) (DONE, `740f4d8`); step 3 canonical keys into
  RepTable::lookup/lookup_full_block with attitude gates on
  KL_column/KL_block/print_block(s)/kl_sum_at_s_terms FIRST (otherwise
  canonicalization lands silently wrong); step 4 transported consumers
  (`singular_flags(bm)`, `located_row_parameter` via `sr(bm)`), the
  `as transformed by`/`simple reflections permuted` headers, then gate
  release and all three locator fixture differentials.
- Non-integral common-block recon COMPLETE (agent-69); slice plan frozen at
  `docs/slices/nonintegral_common_block_workorder.md`. Headlines: upstream
  always builds the block of the integral subsystem directly (smaller
  blocks, subsystem-rank columns); three identity-attitude defects found —
  `length(Param)`, `dual_KL_block(Param)`, `print_partial_common_block`
  (first and third FIXED in `31064b1`; dual_KL_block still open). Four oracle-verified fixtures
  added: `domain/length_dual_proper`, `domain/length_dual_proper_a2`,
  `domain/print_partial_common_block_seq`, `domain/print_partial_block_proper`
  (all intentionally UNREGISTERED until fixed; the A2 one may stay gated on
  the known SL(3,R) identity-shift locator defect).
- Twisted/ext proper-subsystem recursion recon COMPLETE (agent-68); full
  slice plan frozen at `docs/slices/twisted_ext_proper_workorder.md`.
  IN FLIGHT: slice-1A (ExtBlock constructor over PartialBlock, pure
  atlas-real-group, agent-73); wiring phase waits for locator step-3.
- global.w batch 2 LANDED (commit `c5afd9c`, agent-65): int bit utilities
  (succ/pred, AND/OR/XOR/AND_NOT, bitwise_subset, nth_set_bit, bit_length,
  to_bitset), container relations/arithmetic, selectors/joins, matrix
  constructors, gcd(vec), elapsed_ms. Reference frozen by capture
  **3574906**; events/meta committed (`b9843aa`, plans registered); fat
  differential **3574922** in flight. Batch-3 linear algebra IN FLIGHT
  (agent-71, work order docs/slices/global_batch3_workorder.md).
- Locator step 2 LANDED (commit `740f4d8`, agent-66):
  `crates/atlas-real-group/src/block_modifier.rs` — BlockModifier +
  RepContext transform/shift/make_diff_integral_orthogonal/
  make_relative_to/sr_with_modifier, pure and unwired; A2 SL(3,R) anchor
  round-trip exact. Step-3 IN FLIGHT (agent-72: attitude gates FIRST on
  KL_column/KL_block/print_block(s)/kl_sum_at_s_terms, then canonical-key
  wiring of RepTable::lookup/lookup_full_block). NOTE: step-2 report flags
  that Reduced_param::reduce writes the QUERY's locator into bm; stored
  blocks stay in the generating query's attitude.
- Non-integral common-block slices 1-2 LANDED (commit `31064b1`,
  agent-70): `length(Param)` via make_dominant + shared lookup (with a
  value-exact lookup_full_block fallback around the commit_partial NYI),
  `print_partial_common_block` via shared Rep_table lookup + Subset/Elements
  headers. Byte-identical on print_partial_common_block_seq and
  print_partial_block_proper; length lines of length_dual_proper{,_a2}
  match. OPEN: slice 3 dual_KL_block(Param) (needs PartialBlock::dual);
  the A2 SL(3,R) gamma-lambda shift defect on rows 0/2 in the located
  full-block print path (oracle [-1,1]/2 vs Rust [-3,3]/2 — richer than
  the earlier [0,1] note) belongs to the locator slice.
- Six new anchors frozen (events+meta, `b9843aa`; ALL UNREGISTERED except
  the two batch-2 eval plans): domain/ext_block_proper (capture 3574900),
  domain/length_dual_proper{,_a2}, domain/print_partial_common_block_seq,
  domain/print_partial_block_proper (capture 3574902). UPDATE: regression
  differential **3574928** @ `665f2f5` passed 291 PASS + 1 declared PARTIAL
  with step-2 + non-integral slices 1-2 in; the two byte-identical anchors
  (print_partial_common_block_seq, print_partial_block_proper) were then
  REGISTERED (`852c0f6`) and VERIFIED by differential **3574934** (293 PASS
  + 1 declared PARTIAL; metas verified_hpc `7bdb30a`). Still
  unregistered: ext_block_proper (slice 1), length_dual_proper{,_a2}
  (dual_KL_block slice 3 + A2 locator shift defect).
- global.w batch 3 verified_hpc (commit `703a982`, agent-71): matreduc.rs
  op-for-op port — Bezout, echelon, linear_solve (union
  empty_set|affine_subspace), diagonalize, adapted_basis, kernel,
  eigen_lattice, row_saturate, Smith, invert. Capture 3574944, fat
  differential **3575810**: 295 PASS + 1 declared PARTIAL; metas
  verified_hpc (`e238dee`). Batch-4 GF(2) recon IN FLIGHT (agent-74).
- Cross-block partial merge recon COMPLETE (agent-75); work order +
  minimal port sketch at `docs/slices/partial_merge_workorder.md`
  (append_block_containing / pool-extension / union rebuild / retire —
  Hasse import NOT needed, block_access recomputes; KL swallow perf-only).
  Four oracle-verified anchors frozen (capture 3575819, `16339ba`,
  UNREGISTERED): partial_merge_{containment,union,chain,a2}. Implementation
  waits for locator step-3 (rep_table.rs collision).
- 2026-08-19 quota incident: agents 72/73/74 were killed by a provider
  403 (billing-cycle limit) mid-flight and RESUMED in place after the
  refresh; agent-73's partial ext_block.rs edits survived in the tree
  (compiles, uncommitted). If a subagent dies with 403, resume it — its
  context and tree edits persist.
- Twisted/ext slice order per the work order: (1) extended_block proper,
  (2) raw_ext_KL+partial_extended_KL_block proper, (3) twisted_KL_sum_at_s
  proper, (4) twisted_deform proper, (5) twisted_full_deform recursion at
  proper reducibility points (deepest; may force the cross-block
  partial-merge NYI early). Neighbor silent deviation flagged: ordinary
  full_deform's reducibility recursion has NO scope check at all
  (domain_builtins.rs:2282-2321).
- Known defect pinned by the probe: current Rust `print_common_block` on the
  A2 SL(3,R) family already differs from the oracle at identity attitude
  (gamma-lambda shifted by [0,1] on rows 0/2) — the identity-attitude shift
  handling itself is wrong for that family, not just the missing
  canonicalization.

## Checkpoint - 2026-08-17d (proper-integral partial block + KL sums verified)

- `partial_block(Param)` now walks the shared `RepTable` lookup: it rejects
  nonidentity generator attitudes loudly, takes the Bruhat downset of the
  start row through `block_bruhat_hasse`, and rebuilds rows with
  `located_row_parameter`. Both `KL_sum_at_s` and `KL_sum_at_s_to_height`
  share `kl_sum_at_s_terms` (domain_builtins.rs): upstream `contributions`
  expansion (repr.cpp:1861-1898) over the singular subsystem, Horner
  evaluation at q=s, parity sign, and the to-height filter on reconstructed
  final terms. The old dual-block approximation arm for
  `KL_sum_at_s_to_height` is deleted; both wrappers now run upstream's
  standard/final gates at no-value level (`domain_builtin_validate`).
- All 7 local dual-arm probes (`partial_block*`, `kl_sum_at_s*`,
  `print_partial_block`) are byte-identical with the pinned oracle, including
  the rejected standardness diagnostics. `cargo test -p atlas-core --lib`
  (299), clippy `-D warnings`, and fmt are clean. Commits `d388002`
  (HPC reference for 3 partial_block fixtures, capture job 3565274) and
  `1cedff5` (implementation), pushed to `codex/continue-atlas-port` and
  `main`.
- VERIFIED: differential **3573983** @ `1cedff5f` passed all runnable
  observations in 287 fixtures (286 PASS + the declared
  container_syntax_errors PARTIAL; report SHA256
  `564f53f9b80fdefde541420347dc3d1fcefe43d71485d4956e3442da17446e73`).
  Capture **3573984** froze `domain/kl_sum_at_s_param_proper` (B2 split x=5
  `[1,1]`/`[1,0]/2`, covering `KL_sum_at_s` + both `to_height` bounds);
  differential **3574581** @ `650fbccf` passed it (0.008s / 7312 KiB exact
  peak RSS) with 287 PASS + 1 declared PARTIAL across 288 fixtures (report
  SHA256 `e62b34a82ed59eb97541f30dfe92a86cbba65921a9770dda9a2387c95a2cad19`).
  All four metas are `verified_hpc` with their differential jobs recorded.
- Scope is still identity generator attitude; nonidentity locator
  canonicalization and `simple_pi` transport remain loud NYI.

## Checkpoint - 2026-08-17c (proper-integral Param block Hasse verified)

- `block_Hasse(Param)` now consumes the shared `RepTable` full common block,
  reconstructs every stored row through its relative locator shift, and runs
  the Bruhat Hasse recursion over `PartialBlock` through `BlockTopology`.
- Differential **3565080 @ 659646a1290d7a842766c2f5984cc6636211eab0**
  has runnable status PASS across 284 fixtures, with only the two declared
  parser-harness pending cases. The B2 proper-integral fixture took 0.006s /
  7256 KiB exact peak RSS; report SHA256
  `a128613557474a2d1f88fe415c62a6a826c64da4fff17370c65be3fc4eaabd4d`.
- The verified scope remains identity generator attitude. Nonidentity locator
  canonicalization and `simple_pi` transport still fail loudly.

## Checkpoint - 2026-08-17b (proper-integral Param W-graphs verified)

- `W_graph(Param)` and `W_cells(Param)` now use the subsystem-aware shared
  `RepTable` full block and KL table. The B2 `[3,1]/2` proper-integral anchor
  returns the oracle's three-row rank-one graph, start row, subsystem descent
  set, symmetric mu edges, and cell decomposition exactly.
- Differential **3564991 @ 3adbd42b89dbea029ed4fb0e9c53f47b3e46173e**
  has runnable status PASS across 283 fixtures, with only the two declared
  pending fixtures. The proper W-graph fixture took 0.009s / 7376 KiB exact
  peak RSS; report SHA256
  `1cdb3d5924a1cf76b6166d0b632eced4570ba112fd751af95a4c7babec786c8d`.
- This closes only the current identity generator attitude. Upstream locator
  canonicalization and nonidentity `simple_pi` transport are still absent, so
  the full Weyl-conjugate proper-integral domain is not yet claimed.

## Checkpoint - 2026-08-17 (timed twisted deformation verified)

- `twisted_full_deform(Param,int)->(void|KTypePol)` is implemented with a
  per-real-form completed-result cache separate from ordinary deformation,
  cooperative cancellation through recursive twisted deformation, and no
  publication of partial results.
- Upstream validation/timer order is preserved: timer narrowing precedes the
  standard-parameter gate, while `extended_finalise` setup precedes the timer
  start. Zero/negative fresh timers return `timed_out`; cached results return
  `done` even for a zero timer.
- Reference captures are jobs `3554983` and `3564221`. Differential **3564233
  @ 8851395** has runnable status PASS across 282 fixtures; the positive hunger
  contract, cache/timeout contract, and mixed-invalid validation-order fixture
  all pass exact. Rust took 0.006-0.007s and 7080-7276 KiB; report SHA256
  `1c24fcb33dc4d60755d0b1e0434fa5390e687b44d6731efa18e14029927ed107`.
- Proper nonempty integral subsystems in recursive twisted deformation remain
  a loud NYI. Param W-graphs now route through the subsystem-aware RepTable and
  are verified separately in differential 3564991 for identity generator
  attitude.

## Checkpoint - 2026-08-14d (proper integral common blocks activated)

- `PartialBlock::build_full` now drives its initial real-root orbit through
  subsystem generators and `CommonContext::cross`; the B2 `[3,1]/2` anchor
  materializes the expected rank-one, three-row block instead of feeding an
  ambient generator index into a rank-one subsystem.
- `RepTable` keys, row registration, and relative reconstruction now carry the
  interned integral-system identity and build their Smith codec from the
  subsystem parent coroots.  The A2 proper-system `KL_column` event on source
  line 27 locally matches frozen oracle event 26 exactly and is runnable in
  the differential plan.
- This closes the exact embedded/identity-attitude case only.  Upstream
  `int_item(gamma, locator)` canonicalization across Weyl-conjugate systems,
  `w`, `simple_pi`, and nontrivial block modifiers still need to be stored on
  block records before the full proper-system domain can be claimed.
- Differential **3550974 @ 8d03ba9** verified the selected typed pipeline with
  a clean source snapshot.  `domain/kl_column` now passes exact stdout,
  diagnostics, and exit status for its proper-system event; the run remains
  `PARTIAL` only for the two unrelated declared pending features.
- `print_block(Param)` now uses the shared `RepTable` full-block rows for a
  proper integral subsystem, while preserving the existing full-system path.
  New B2 `[3,1]/2` fixture `domain/print_common_block_proper` matches the
  pinned oracle byte-for-byte; differential **3551242 @ 62e32d3** passed it
  with Rust 0.006s / 7300 KiB.  Proper extended/twisted block consumers still
  need the same subsystem-aware treatment.

## Checkpoint - 2026-08-14c (block(Param) submitted; proper-system key foundation)

- **`block(Param)` landed as `6c4b6ff`**: the exact
  `Param -> ([Param],int)` registry signature, standardness-before-no-value
  validation, shared full-block lookup, singular-survivor filtering, shifted
  row reconstruction, and survivor-local start index are active.  Both P2
  fixtures PASS the local structured pipeline; local Atlas/Rust stdout and
  stderr are byte-identical for the accepted fixture.  Differential **3550585
  @ 6c4b6ff passed** (runnable PASS, 3 declared pending overall); P2 accepted
  and rejected took 0.005s and 7300/6804 KiB respectively.
- **The stale runner failure is cleared**: differential **3550540 @ 80e3eb4**
  completed with runnable status PASS.  `kl_column` is now PARTIAL by design at
  its one proper-subsystem event instead of failing the suite; the shared
  `rep_table_sequence(+-)` fixtures remain PASS.
- **Proper integral-system foundation is locally complete but does not yet
  change language behavior**: `RepTable::State` interns an exact embedded
  subsystem by its ordered parent-simple `RootId` list, reusing a stable
  `IntegralSystem::Interned` ID, while the identity ambient system remains
  `Full`.  `IntegralCodec` construction now accepts an `IntegralSubsystem` and
  builds its evaluation matrix from the subsystem parent coroots.  B2
  `[3,1]/2`, whose rank-one simple is a non-simple ambient root, is the test
  anchor.  Existing full lookup still rejects every non-`Full` system loudly.
- **Structural preflight submitted**: job **3550626 @ f5f33fc** was submitted
  on the fat partition after an exact detached-tree check; collect its report
  with the next HPC batch rather than blocking this loop.
- **Next**: port upstream `InnerClass::int_item(gamma, locator)` semantics:
  canonicalize Weyl-conjugate integral systems, retain `w`, ordered simply
  integral roots and `simple_pi`, then store that locator/subsystem metadata on
  each block record.  Only after `co_reduce`, relative modifier/shift, and row
  reconstruction use that metadata should the A2 `KL_column` pending event be
  enabled.  Exact embedding interning alone is not an observable-compatibility
  claim and must not be mistaken for the final locator.

## Checkpoint - 2026-08-14b (handoff: differential 3549756 analyzed; block(Param) half-done in tree)

**Repository state**: pushed through **`80e3eb4`** (includes
`fix: mark proper-subsystem KL case pending`, landed by a concurrent
agent during this session). Uncommitted in the working tree: the
half-done **block(Param)->([Param],int)** slice (see below) — it
compiles (`cargo check -p atlas-core` clean) but is otherwise
UNVERIFIED. `crates/atlas-real-group/examples/fiber_probe.rs` is the
user's file; preserve it.

**Differential 3549756 @ 5cb14f8**: 270 PASS / 3 PARTIAL / 1 FAIL.

- rep_table_sequence(±) PASS — the `ActiveKlCallback::drop` release
  repair (`fcc7026`) is confirmed on the release build; the
  nested-callback failure mode of 3547776 is gone.
- print_block_words(±) and prim_kl_order(±) PASS; all four metas now
  verified_hpc (print_block_words± via differential 3542976 by the
  concurrent agent; prim_kl_order± via 3549756).
- p2_block_graph_signatures(±) PARTIAL is BY DESIGN (pending
  block(Param) events) — cleared by finishing the in-tree slice below.
- container_syntax_errors PARTIAL is the permanent known item.
- kl_column FAIL is EXPLAINED, not a regression: the job ran at
  5cb14f8, whose pipeline_swap_diff.py lacked the kl_column line-27
  PendingCase (added later in `80e3eb4`). The runnable input therefore
  still contained `KL_column(q)` on the proper integral subsystem and
  hit the loud NYI. Re-running the differential at >= 80e3eb4 should
  clear it; no code change needed.

**Half-done slice: block(Param)->([Param],int)** (common_block_wrapper,
atlas-types.w:6748-6780, installed :7510). Already edited, compiles:

1. `crates/atlas-core/src/typed.rs` (~line 6212): second
   `domain_builtin_validate("block", Param -> ([Param],int), 0)`
   registration next to the (RealForm,RealForm) one.
2. `crates/atlas-core/src/domain_builtins.rs` validate arm "block"
   (~line 8489): arity-1 Param shape gates
   `test_standard(parameter, "Cannot generate block")` (upstream gates
   before the no_value check).
3. Same file, eval arm "block" (~line 12160): arity-1 Param path —
   test_standard, made_dominant, integral_block_scope (Singleton ->
   `([dominant], 0)`; ProperSubsystem -> `proper_subsystem_diagnostic`;
   Full -> `lookup_full_block`), then survivors via
   `CommonContext::integral` + `singular_flags(prepared_query().gamma())`
   + `block.survives`, params rebuilt with `located_row_parameter`,
   start_pos = survivor position of `located.raw_row()` else -1.
   Mirrors the KL_block arm (~line 13550) which is the verified pattern.

Remaining steps for this slice:

1. Local dual-arm probe: run the p2_block_graph_signatures fixture
   against the local oracle
   (`{ cat <fixture>; echo quit; } | (cd ~/mycodes/atlasofliegroups/atlas-scripts && ../atlas)`)
   and diff against ./target/debug/atlas-cli; event 14 (line 15
   `block(p)`) must match byte-exact.
2. Remove the two PendingCases (accepted line 15/event 14; rejected
   line 7/event 6) in hpc/pipeline_swap_diff.py, make every event
   runnable, then `python3 -m unittest hpc.test_pipeline_swap_diff` and
   a full local replay of both p2 fixtures through
   `expected_cli_observation` (stdout + diagnostics + exit status).
3. Gates: `cargo test -p atlas-core --lib`,
   `cargo clippy -p atlas-core --lib --tests --no-deps -- -D warnings`,
   `cargo fmt --all -- --check`.
4. Commit, push, bundle-sync to HPC (bundle base = HPC's current
   checkout), submit the differential
   (`ATLAS_COMMIT=<full sha> ATLAS_DIRTY_TREE=false sbatch
   --partition=fat --time=01:00:00 --mem=32G --export=ALL,TIMEOUT=3600
   hpc/pipeline_swap_diff.sbatch`). That same run also clears the
   stale-plan kl_column FAIL. On PASS, update the p2 fixture plans and
   record the job in HANDOFF.

**Then** (from checkpoint 2026-08-13i, still open): the real proper
integral subsystem RepTable/locator path (un-pends kl_column line 27),
and timed `full_deform(Param,int)`.

**Process note**: TWO agents worked this repo concurrently this
session; before editing, always `git log --oneline -3` and re-check
`git status` — file contents and HPC checkouts may have moved.

## Checkpoint - 2026-08-14a (rep_table release bug repaired; prim_KL/print_block sweep fixes)

- **`ActiveKlCallback::drop` repaired (`fcc7026`)**: the flag clear lived
  inside `debug_assert!` and vanished in release builds, leaving
  `ACTIVE_KL_CALLBACK=true` forever (root cause of the 3547776 nested-
  callback failures). The `replace(false)` now executes unconditionally;
  only the returned value is debug-asserted. Two regression tests added;
  the sequential-enter test is only meaningful under `--release`
  (`cargo test -p atlas-real-group --lib --release active_kl_callback`).
- **Coverage sweep found print_prim_KL divergences, fixed (`c7d09ee`)**:
  (1) primitive x indices emitted in descending prim_back_up walk order;
  upstream collects into a BitMap iterated ascending (kl.cpp:163-172,
  kl_io.cpp:117) — now reversed; (2) the P_{y,y} trailer missed the
  setw(width+tab) pad (kl_io.cpp:138-139). Invisible on the small
  kl_print blocks; surfaced on D4 (rf 4 x dual 1, 28-element block).
  print_KL_basis/print_KL_list/print_W_graph/print_W_cells re-probed
  byte-identical on the same block. Fixture domain/prim_kl_order(±).
- **print_block fixes (`7dea126`)** from the earlier sweep turn: `*`
  right-alignment (block_io.cpp:197,205) and the WeylGroup::word
  tie-break via `CompactWeyl::canonical_word` (weyl.cpp:944-958).
  Fixture domain/print_block_words(±).
- **HPC ledger**: captures 3549616 (print_block_words±, PASS) and
  3549730 (prim_kl_order±, PASS); references bumped to
  verified_hpc_reference (`84be139`, `5cb14f8`). Differential **3549756
  @ 5cb14f8 in flight** — resubmission of 3547776 with the rep_table
  repair plus the four new fixtures. On PASS: bump the four metas to
  verified_hpc with differential_job=3549756.
- Note: `crates/atlas-real-group/examples/fiber_probe.rs` is the user's
  file; preserve it.

## Checkpoint - 2026-08-13i (PAUSED: shared RepTable callers, release-only blocker — REPAIRED 2026-08-14a)

The user paused the autonomous port and is handing the repository to another
coding agent.  Do not resume the interrupted `block(Param)` or proper-integral-
subsystem explorations before repairing and re-running the failed differential
described below.

### Published state

- `main` is pushed through **`9730864`**.
- `75bf75b feat: route parameter KL through representation tables` routes
  `KL_column`, `KL_block`, and `print_common_block` through the per-real-form
  shared `RepTableOwner`; it also adds `LocatedBlock::prepared_query`, raw-row
  parameter reconstruction with the relative shift, shared KL fills, the
  `{zero,one}` condensed polynomial store, rank-zero singleton fallbacks, and
  the exact materialisation sequence test.
- `9730864 test: register representation table sequencing` adds
  `rep_table_sequence{,_rejected}` to `hpc/pipeline_swap_diff.py`.
- The preceding substrate commits are `108c463` (owning
  `RepTableOwner`), `ee2a631` (canonical real-form weak memo and shared owner),
  and `9ecc8d0` (one KL table per stable representation-block record).
- Source/spec and Rust reviews approved the full-integral identity-locator
  slice after fixing the `KL_column` raw range (`0..=raw_y`), preserving
  accumulated `finals_for` branches at compact descents, and making missing
  block lengths loud invariants.  These approvals predated the release-only
  failure below.
- The only local untracked file is the user's
  `crates/atlas-real-group/examples/fiber_probe.rs`; preserve it.

### HPC job 3547776: FAIL and exact first repair

Job **3547776** was submitted on the clean exact commit
`973086493d2fdfcaab0495649627ae5a0a07c4d1` (`fat`, 32 GiB,
`TIMEOUT=1200`).  Source-state verification passed, but the runnable
differential failed.  The report is:

```text
/public/home/majj/atlas-rust/results/
  973086493d2fdfcaab0495649627ae5a0a07c4d1/3547776/
  pipeline_swap/pipeline_swap_diff_report.json
```

There is a deterministic release-only RAII bug in
`crates/atlas-real-group/src/rep_table.rs`, `Drop for ActiveKlCallback`:

```rust
debug_assert!(active.replace(false));
```

In a debug local build the expression executes and clears the thread-local
flag.  In the release HPC build `debug_assert!` removes the entire expression,
so the first successful KL callback leaves `ACTIVE_KL_CALLBACK=true` forever.
Every later `with_kl_table` on that worker thread then fails with
`representation block KL table nested callback`.  This explains why local
debug replay of `rep_table_sequence.atlas` exits 0 while HPC release reports
five nested-callback errors, and why the larger `kl_column` fixture reports the
same error after its first callback.  The smallest repair is to execute the
state change unconditionally, then debug-assert only its returned value, for
example:

```rust
let was_active = active.replace(false);
debug_assert!(was_active);
```

Add a **release-relevant sequential callback regression** (two non-nested
`with_kl_table` calls on the same thread; preferably also two different
records), rebuild `atlas-cli --release`, replay `rep_table_sequence`, and
resubmit the differential.  Do not treat the existing debug-only focused tests
as sufficient evidence.

The same job also confirms the already declared independent gap at
`tests/fixtures/domain/kl_column.atlas:27`: the A2 parameter uses a proper
nonempty integral subsystem and now receives the loud diagnostic
`common block on a proper integral subsystem is not yet implemented`.  The
new shared path deliberately removed the old classic-full-block approximation;
do not restore that approximation.  Implement the real proper-subsystem
RepTable/locator path or mark that fixture line pending until it exists.

### Exact caller contracts already established

- `KL_column` validates standard and final before its no-value gate, uses
  partial `lookup`, fills through exclusive limit `raw_y + 1`, visits raw rows
  `0..=raw_y`, and emits `(raw_x, adapted Param, coefficients)` for nonzero
  polynomials.
- `KL_block` validates standard before no-value, uses `lookup_full_block`,
  fills the full shared KL table, retains singular survivors in raw order,
  condenses with `finals_for`, and exports an identity index matrix with the
  polynomial pool initially `[[],[1]]`.
- Both lookup functions prepare the wrapper-owned parameter by reference in
  upstream C++ (`normalise` for partial, `make_dominant` for full).  Therefore
  row reconstruction and singular flags must use
  `LocatedBlock::prepared_query().gamma()`, not the caller's pre-lookup gamma.
- `print_common_block(Param)` installs/reuses a full shared block;
  `print_block(Param)`, `print_partial_block`, and no-value `KL_block` do not
  warm the table.  The frozen sequence is standalone `KL_column` raw row 0,
  value `KL_block` then raw row 1, no-value `KL_block` then raw row 0,
  `print_common_block` then raw row 1, and direct printers then raw row 0.
- Ambient-rank-positive, integral-subsystem-rank-zero inputs retain exact
  singleton fallbacks.  Proper nonempty integral subsystems remain loud NYI.

### Next work after repairing 3547776

1. Repair `ActiveKlCallback::drop`, add sequential release regression, run
   bounded fmt/check/clippy/focused tests, then spec + Rust review.
2. Commit/push the repair, sync a clean exact checkout to HPC, and resubmit the
   pipeline differential.  Promote `rep_table_sequence` metadata only after
   its accepted and rejected entries pass with benchmark fields.
3. Implement the now-unblocked simple signature
   `block(Param)->([Param],int)`: upstream `common_block_wrapper` validates
   standardness before no-value, calls `lookup_full_block`, uses the prepared
   dominant gamma and modifier, filters `block.survives`, and returns the
   survivor-local start index or `-1`.  The frozen pending event is line 15 of
   `p2_block_graph_signatures.atlas`; its rejected companion also has one
   pending overload-diagnostic event in the runner.
4. Then implement genuine proper integral subsystems (canonical integral
   system/locator, reduced key, block modifier and row reconstruction) so the
   A2 `KL_column` case is accepted.  This is a prerequisite for claiming the
   KL builtins across their full upstream domain.
5. Timed `full_deform(Param,int)` is implemented and differential-verified by
   HPC `3551338`: exact signature/rejection, `0`/`-1` timeout, completed-result
   cache warming, no-value validation, and cooperative deadline checks all
   match the frozen `timed_full_deform_*` contracts.

### Distance to the stated goal

The registry was last audited at roughly 299/305 exact upstream signatures,
but signature count is not completion.  Remaining language incompatibility is
concentrated in proper integral subsystems/locators, recursive deformation and
its cache, extended/twisted KL branches, and timed cooperative cancellation.
Treat the project as having roughly the last 10--20% of engineering effort
left, but that remainder is the algorithmically hardest part; do not advertise
full Atlas C++ language compatibility yet.

## Checkpoint - 2026-08-13f (full common-block constructor foundation)

- `PartialBlock::build_full` now implements rank-zero singleton blocks and the
  full-integral, identity-locator common-block constructor from pinned
  `blocks.cpp:733-1081`.  The implementation preserves top-ascent restart,
  real-root `y` orbit generation, FIFO packets, Cayley completion, global `y`
  numbering, reversed lengths, sort/remap, and full-SRM lookup.
- Differential-shaped crate goldens cover A1 from `x=0/1/2` with exact init,
  and all 12 pinned B2 rows.  B2 explicitly fixes both `x=10` rows with their
  distinct `gamma_lambda`, the global `y` sequence, every status/cross/Cayley
  cell, and seed-independent construction.  Proper nonempty integral
  subsystems remain a loud NYI.
- The arbitrary-root `reflection_word` helper moved from `ext_param.rs` to a
  shared crate-private module without changing its wrapping arithmetic or word
  convention.  Both independent spec and Rust-quality reviews approved the
  final slice; the latter required decomposing the initial 300-line state
  machine into four auditable phases around a `FullBlockBuilder` state.
- Do not connect this constructor directly to `block(Param)`: the shared
  per-real-form `RepTable` pool, reduced-key row registration, locator/modifier,
  partial/full promotion, and sequence contracts are still missing.  The next
  integration target remains `rep_table_sequence{,_rejected}`.

## Checkpoint - 2026-08-13g (deformation alcove-center shrink)

- The real-group crate now owns `alcove_center(RepContext, StandardRepr)`.
  Ordinary full deformation centers every final helper input when its
  denominator exceeds `2^rank`; twisted deformation replaces the former loud
  NYI with the same preprocessing and leaves its flip bookkeeping unchanged.
  The language builtin delegates to this shared implementation.
- Review found two reusable arithmetic hazards and both are regression-tested:
  `checked_shl(63)` on `i64` produces `i64::MIN` rather than failing, so ranks
  63+ bypass the positive-denominator comparison; and a full-column-rank
  Gauss-Jordan solve must still reject residual `0 ... 0 | nonzero` rows in an
  overdetermined system.
- The job-3546215 positive/rejected oracle contracts are now closed by
  differential **3546956** at `cfd6643`: both are exact PASS (0.005s each,
  7144/6976 KiB).  The overall report is PARTIAL only because of the already
  declared project-wide pending items; runnable status is PASS.  Report SHA256
  is `6bf959ec7d880204564b862f767a1600ba878af44e3369a39a6376ff23c3972e`.
  This slice is shrink preprocessing only; it does not complete
  ordinary deformation recursion, proper-subsystem handling, RepTable memo,
  or timed cancellation.

## Checkpoint - 2026-08-13h (shared RepTable kernel)

- The real-group crate now has a crate-private, full-integral identity-locator
  `RepTable<'a>` kernel.  It is lifetime-bound to its `RepContext` owners and
  validates them by reference identity on every lookup; it cannot be reused
  across real forms or outlive the borrowed graph/table/inner class.
- Storage uses stable append-only block IDs, superseded tombstones, all-row
  reduced-key places, `Arc<PartialBlock>` records, and relative shifts.
  Partial/full materialisation happens outside the mutex; commit re-probes and
  either reuses a concurrent winner or updates state atomically.  Full
  promotion bulk-retires every overlapping partial and clears their places in
  one pass.
- Important row rule: a fresh partial lookup returns its exact seed row;
  reverse registration's smallest colliding row applies only to later key
  hits.  Also, pinned B2 rows 10/11 are not a collision: transported Smith
  residues are 0 and 2.  Both facts are fixed by tests.
- Nineteen focused tests cover A1 row 0→1 promotion, all B2 rows, relative
  shifts, stable IDs, no dangling places, context rejection, and deterministic
  full/partial/partial commit races.  Partial-partial merging is still a loud,
  failure-atomic NYI.  Next: make `RealFormContext` own the table, then route
  KL_column, KL_block, and print_common_block to close `rep_table_sequence`.

## Checkpoint - 2026-08-13e (signature audit + multi-assignment)

- The builtin completion target is now signature-level: 305 upstream
  `install_function` registrations, 277 exact Rust matches, 28 exact
  missing/mismatched signatures. The remaining work is classified in
  `REMAINING_BUILTINS.md`; clear the 23 simple signatures first.
- `set pattern := value` was a parser-only compatibility hole. Fixtures
  `eval/multi_assignment(±)` were captured by HPC job **3542977** and match
  the local frozen oracle byte-for-byte. The implementation distinguishes
  omitted tuple slots from explicit `()`, threads mixed local/global targets
  in child-before-whole postorder, evaluates the RHS once, commits only after
  success, returns the whole RHS, refines target types, and preserves exact
  Atlas diagnostics.
- Two review-found regressions were repaired before submission: RHS analysis
  may re-specialise a destination, and upstream deliberately ignores a later
  incompatible refine rather than panicking; case-discrimination `()` keeps
  its void payload constraint and uses the exact `Pattern () does not match
  type ... for variant ...` diagnostic. `atlas-core` has 251/251 passing
  tests; spec and Rust-quality reviews both approved the final diff.
- **multi-assignment CLOSED**: differential **3543144** @ 147b982 —
  245 fixtures, runnable status PASS, with only the permanent two-event
  `container_syntax_errors` EOF/quit exception. Both multi-assignment metas
  are `verified_hpc`; the fat-node job took 130 seconds and peaked at
  997100 KiB.
- HPC reference-capture arguments must be complete repository-relative
  `tests/fixtures/.../*.atlas` paths. Job 3542734 failed on bare names;
  corrected job **3542971** passed. Range bundles advertise `HEAD`; inspect
  `git bundle list-heads` instead of assuming a `main` ref.

## Checkpoint - 2026-08-13c (post-closure coverage sweep: 4 latent bugs found and fixed)

The 233/233 verified matrix was NOT the end: a sweep of live arms with
absent/thin fixture coverage found four real divergences, all fixed,
all dual-arm byte-verified locally, fixtures pinned:

1. **derived_info/mod_central_torus_info shared arm** (`cfe8420`):
   (a) the (RootDatum,mat) tuple flattened row-major data into
   `Matrix::from_columns` (column-major) — displayed the TRANSPOSE,
   invisible on single-column injectors (the only prior coverage was
   A1.T1 in coroot_queries); (b) the DerivedTag arm fed adapted_basis
   coroots-as-rows instead of upstream's coroots-as-columns
   (prerootdata.cpp:67-82), diverging on adjoint data; (c) the derived
   datum isogeny was hardcoded SimplyConnected — now classify_isogeny
   (adjoint B2 stays adjoint, G2 classifies Both). Fixtures
   domain/derived_info(±).
2. **integrality_points** (`add126c`): declared [ratvec] instead of
   upstream's [rat] (atlas-types.w:2268); fractions were not
   normalised/deduped (2/2 didn't fold into 1/1) nor value-ordered —
   upstream collects std::set<RatNum> (rootdata.cpp:1508-1527), now
   BTreeSet<BigRational>; the rank-length precheck
   (atlas-types.w:1808-1819) was missing entirely.
3. **dual_KL(Block)** (`37656ac`): the shared raw_KL arm returned the
   PRIMAL table; upstream raw_dual_KL_wrapper builds
   Block::build(dual_rf, rf) and maps entries through blocks::dual_map
   = dual_b.element(b.y(z), b.x(z)) (atlas-types.w:8640-8674,
   blocks.cpp:1715-1725). The dual_kl_block fixture only exercised the
   script-level dual_KL_block wrapper, masking this. Fixtures
   domain/dual_kl_raw(±).
4. **Identifier ascriptions accepted only primitive types** (`9c8c1ff`):
   `t : (int,int)` was a syntax error; parser.y:162 allows the full
   type grammar. Command::Declare now carries a TypeExpr. Named-type
   ascriptions (`x : MyType`) remain unsupported — the lexer has no
   type-table token (upstream lexes TYPE_ID contextually); no fixture
   or basic.at path needs it, recorded as a known limitation. Fixtures
   eval/declare_types(±).

Also pinned first dedicated coverage for index(Block,KGBElt,KGBElt) and
to_canonical_fiber(KType) (domain/block_ktype_extras(±)) — both were
already correct.

**Process lessons**:
- The differential REFUSES fixtures whose reference metadata is not
  verified_hpc_reference ("reference metadata is not HPC-verified" in
  configuration_errors; differential 3542470 failed exactly this way).
  Sequence is: capture PASS → bump reference_status → THEN differential.
- Probe harness gotcha: `printf fmt arg - <<<"quit"` replays the format
  on `-`, emitting ghost lines; always `{ printf '%s\n' ...; echo quit; }`.
- Audit pattern for matrix display: any `Matrix::from_columns` fed a
  row-major `.into_iter().flatten()` is a transpose bug; the remaining
  call sites (941, 2159, 10853) were checked and are correct.

**In flight**: capture 3542509 (dual_kl_raw±); after it passes, one
differential over the 8 new fixtures upgrades all metas to verified_hpc.

## Checkpoint - 2026-08-13d (sweep closed: print_block fixes + 3542511 all green)

- **Differential 3542511 @ 56ae86c: 240 PASS, 0 FAIL**
  (container_syntax_errors PARTIAL is the permanent known item). All 10
  new fixture metas upgraded to verified_hpc with
  differential_job=3542511, differential_commit=56ae86c(full sha):
  eval/declare_types(±), domain/derived_info(±),
  domain/integrality_points(±), domain/block_ktype_extras(±),
  domain/dual_kl_raw(±).
- **print_block sweep found two more latent divergences, fixed in
  `7dea126`**:
  1. `*` placeholders in the cross-action/Cartan columns were formatted
     `{:width$}` — Rust left-aligns chars by default, upstream setw
     right-aligns (block_io.cpp:197,205). Now `{:>width$}` in the
     print_block/print_blockd arms. Visible once column width > 1
     (e.g. `( *, *)` in D4 blocks).
  2. The Weyl word column used the greedy minimal left-descent word;
     upstream prints `WeylGroup::word` (weyl.cpp:944-958): per-piece
     unshift election, pieces appended in increasing order, d_out
     numbering. New `CompactWeyl::canonical_word` (weyl_transducer.rs,
     re-exported from atlas-real-group) reproduces it; wired into both
     the print_block arms and the Cartan_info fiber-word arm. B2
     (rf 2 x compact dual) pins w0 as `2,1,2,1` where greedy gives
     `1,2,1,2`; D4 (rf 4 x dual 1) pins a 28-element block.
  Fixtures domain/print_block_words(±); both arms byte-verified against
  the local oracle before submission, plus byte-exact local replay of
  the six pre-existing print_block/cartan fixtures to prove no
  regression from the canonical_word switch.
- **Reference capture CLOSED**: the original job 3542734 failed during fixture
  validation because it was submitted with bare names instead of complete
  repository-relative `tests/fixtures/.../*.atlas` paths. The corrected job
  **3542971** @ 7dea126 passed; both stdout/stderr pairs match the checked-in
  events byte-for-byte (including rejected exit status 1), with per-fixture
  wall time and peak RSS recorded. Both metas are now
  `verified_hpc_reference`; the next step is the differential run.

- **print_block_words CLOSED**: differential **3542976** @ 98c080f —
  runnable status PASS across 243 fixtures; only the permanent two-event
  `container_syntax_errors` EOF/quit exception remains PARTIAL. Both metas are
  `verified_hpc`. The fat-node job took 145 seconds and peaked at 957792 KiB.
- **Next compatibility gap activated**: `set pattern := value` multiple
  assignment was parser-only and still failed analysis despite assignment
  being marked supported. Positive/negative fixtures were pinned first;
  reference capture **3542977** @ 87c98eb passed and matched the local frozen
  oracle byte-for-byte. Implementation follows axis.w:6956-7500.
- **Bundle sync lesson**: a range bundle created as
  `git bundle create <file> <base>..HEAD` advertises the tip as `HEAD`, not
  necessarily `main`; inspect with `git bundle list-heads` and fetch the
  advertised ref (`git fetch <bundle> HEAD`) rather than assuming `main`.



- **print_partial_block CLOSED — the last contract**: differential
  **3542430** @ 516f8c6 — 228 PASS, 0 FAIL (container_syntax_errors
  PARTIAL is the permanent known item); meta verified_hpc (`989f2df`).
  Language arms (`6fb1c30`, agent-60): print_partial_block +
  print_partial_common_block via partial_block_rows helper on the
  f11f48a crate port; byte-exact replay. Notes from delivery: the
  "Subset {...}" header branch is unimplemented (requires a cross-call
  block-cache hit the fresh-build-per-call design never produces —
  documented in the arm); the brief's header text was wrong (upstream
  prints init_index, not init_index+1 — moot, no captured case emits a
  header); the partial path supports arbitrary gamma incl. rank>0
  non-integral subsystems, exceeding common_block_rows.
- **Meta scan: 231 verified_hpc, 0 pending.** LANGUAGE.md domain row
  moved to `supported`. The full upstream install_function surface
  (187 names) is live and differential-verified.
- **Remaining open items (none block the language matrix)**:
  1. readline completion (TTY-only interactive feature) — deferred
     outside the language-only gate, needs user decision;
  2. KL binary file formats (filekl.w; zero language builtins touch
     them) — deferred, needs user decision;
  3. Known non-blocking hazards on record: print_block(Block)/print_blockd
     `*` left-align padding (agent-53; no fixture triggers it);
     typed.rs:5216 KL_block dead skip registration order (harmless);
     W_graph non-integer gamma imaginary grading not ported (loud
     error, no fixture); timed twisted_full_deform runtime arm is a
     loud "not yet implemented" (registration needed for overload
     wording); proper-integral-subsystem common block is a loud error
     path (IntegralBlockScope::ProperSubsystem); block_modifier-based
     common_context ctor not ported (print_partial caveat);
     alcove_center not ported (denominator > 2^rank → loud error).
  4. Suggested crate refactor (not required): crate-owned
     `RepContext::is_fixed_normalised` to replace the language-side
     shim in domain_builtins.rs (agent-57 note).

## Checkpoint - 2026-08-13a (ext_finalise closed; E3 language layer landed; print_partial in flight)

- **ext_finalise(±) CLOSED**: differential **3542388** @ 638cfed — 223
  PASS, 0 FAIL; metas verified_hpc (`225216a`). E2 language layer
  (`f6efd3b`, agent-57): trio registered via domain_builtin_validate,
  upstream gate order, literal "|" typo kept in K_type_pol_extended
  descr. **Deviation worth knowing**: the brief's suggested crate
  `RepContext::is_fixed` was WRONG for this slice (raw gamma check);
  the wrappers need repr.cpp:669-675's normalising is_fixed —
  implemented language-side on public APIs (`z.normalised` + rebuild
  twisted via graph.twisted/sr_gamma, PartialEq compare). Suggested
  follow-up: crate-owned `RepContext::is_fixed_normalised`.
- **E3 language layer landed (`0cfba0b`, agent-59)**:
  twisted_deform/twisted_full_deform(+timed overload)/twisted_KL_sum_at_s
  (both arities)/block_deform. 238 atlas-core tests, replay 12/12
  byte-exact. Notable: timed `(Param,int)->|KTypePol` overload
  registered because the rejected fixture's multi-variant wording
  requires it (runtime arm fails loudly "timed twisted_full_deform is
  not yet implemented"); ProperSubsystem maps to loud "common block on
  a proper integral subsystem is not yet implemented". FixturePlans
  registered (`537aaf5`); differential **3542417** in flight.
- **In flight**: agent-60 (print_partial_block +
  print_partial_common_block language arms — the LAST never-registered
  builtin surface; crate machinery from f11f48a, renderer reuses
  pcb's render_common_block).
- **Queue**: (1) collect 3542417 → twisted_family/block_deform metas;
  (2) agent-60 delivery → print_partial FixturePlan + differential;
  (3) final matrix audit + user decision on the two documented
  exclusions (readline completion TTY-only; KL binary file formats).

## Checkpoint - 2026-08-12f (shift_flip landed + differential in flight; NDEBUG assert lesson)

- **shift_flip language layer landed (`46963fd`, agent-55)**:
  registration in typed.rs (Validate level matching atlas-types.w:7341-7362),
  shared gate helper (test_compatible → "Involution does not fix rational
  weight" → "Involution does not fix infinitesimal character", upstream
  order/strings), call arm via ExtRepContext::shifted_default_extension +
  is_default. 233 atlas-core tests, clippy/fmt clean, release AND debug
  replay byte-identical.
- **NDEBUG assert lesson (`f668589`)**: two debug_asserts in
  atlas-real-group ext_param.rs (shifted_default_extension's
  `(1+theta_x)*shift==0` and same_sign's `same_standard_reps`) encode
  upstream `assert`s that the oracle NEVER checks (upstream Makefile
  builds with -DNDEBUG). Both are reachable-via-shift_flip with
  violating inputs (nonzero shift at a compact Cartan) and panicked the
  debug CLI where the oracle returns a well-defined `false`. Rule of
  thumb: before porting any upstream `assert` as a Rust (debug_)assert,
  check whether the wrapper layer can reach it with violating inputs —
  if yes, omit it with a comment citing the NDEBUG parity.
- **shift_flip FixturePlan registered (`8f1043f`)**: 45 lines/45 events,
  alignment pre-analysed; harness unittest OK. Differential **3541888**
  @ 8f1043f in flight (fat partition, TIMEOUT=3600).
- **shift_flip(±) CLOSED**: differential **3541896** @ f4b1391 — 221
  PASS, 0 FAIL; both metas verified_hpc (`1bac17d`). (First submission
  3541888 failed on configuration only: events.json status was still
  pending_hpc_reference — fixed by HPC capture 3541893 + byte-exact
  round-trip + status flip `f4b1391`. Lesson: registering a FixturePlan
  requires BOTH meta.reference_status AND events.status
  verified_hpc_reference.)
- **LANGUAGE.md counters refreshed (`cd45130`)**: 224 of 231 fixture
  contracts verified_hpc; 7 pending (all domain, no rejected by design):
  ext_finalise(±), twisted_family(±), block_deform(±),
  print_partial_block.
- **E3 crate drivers landed (`4246006`, agent-56)**: new deform.rs —
  SplitInteger, integral_block_scope (Singleton/Full/ProperSubsystem),
  twisted_deformation_terms, twisted_kl_sum + twisted_kl_column_at_s,
  twisted_deformation (lookup closure for reducibility recursion),
  block_deformation_to_height. 387 crate tests (12 new replaying oracle
  jobs 3536421/3536583), clippy/fmt clean. Language-layer note: Singleton
  scope short-circuits (twisted_deform → empty pol, twisted_KL_sum_at_s
  → 1*p); ProperSubsystem must be a loud runtime error;
  alcove_center not ported (NotYetImplemented when
  gamma.denominator() > 2^rank — fixtures never trigger).
- **print_partial crate port landed (`f11f48a`, agent-58)**: new
  partial_block.rs — StandardReprMod, IntegralSubsystem (upstream
  generator-order re-sort fix for B2), CommonContext srm-level
  cross/is_parity/down_cayley/up_cayley, bruhat_below interval
  generator, PartialBlock::build+sort, singular_flags/survives. 392
  crate tests (5 new replaying print_partial_block oracle rows: x-sets,
  descents, cross/Cayley links, lengths, gamma_lambdas). Language call
  path documented in the module (mod_reduce → CommonContext::integral →
  bruhat_below → PartialBlock::build → singular_flags). Caveat: only
  the gamma-based common_context ctor ported; the block_modifier-based
  one (repr.cpp:2672-2677) is not — irrelevant for the current fixture.
- **In flight**: agent-57 (E2 language layer: scale_extended/
  K_type_pol_extended/finalize_extended, atlas-core).
- **Queue**: (1) agent-57 delivery → ext_finalise FixturePlan (snippet
  in queue doc) + differential; (2) E3 language layer (brief
  /tmp/slice_e3_brief.md) → twisted_family + block_deform
  differentials; (3) print_partial language arms (render reuses pcb's
  render_common_block; call path in partial_block.rs docs) →
  FixturePlan + differential; (4) final matrix audit + user decision on
  the two documented exclusions (readline completion TTY-only; KL
  binary file formats).

## Checkpoint - 2026-08-12e (dual_KL verified; pcb landed; E2 crate landed; skip-tail retracted)

- **dual_kl_block(±) closed**: differential **3541634** @ dafdc03 —
  219 fixtures, 218 PASS, 0 FAIL; metas verified_hpc (`0f1789a`).
- **print_common_block landed (`ab811fa`, agent-53)**:
  common_block_rows engine + byte-exact render (block_io.cpp:54-147,
  right-aligned `*` markers), print_block(Param) branch; replay
  byte-exact, 232 atlas-core tests. Registered in FixturePlan
  (`946a97a`); differential **3541690** in flight (cron 0beadea7).
  Agent-flagged pre-existing hazard (not touched): print_block(Block)/
  print_blockd `*` padding left-aligns — a future Block fixture with
  undefined Cayley at width>1 would misalign.
- **E2 crate drivers landed (`be0e16c`, agent-54)**:
  extended_restrict_to_k / extended_finalise / scaled_extended_finalise
  (ext_block.cpp:2435-2807), purely additive, 6 tests replaying pinned
  ext_finalise values, 375 crate tests green. E2 language layer
  (typed.rs wrappers, precondition order per /tmp/slice_e2_brief.md) is
  the remaining E2 work, queued behind shift_flip for atlas-core.
- **Builtin reconciliation corrected (`a8a314e`)**: 187 upstream
  install_function names; empirical probes on committed binaries show
  every "skip arm" in typed.rs is a dead registration shadowed by a
  live one (dual/inner_class/involution/twist/K_type/param/re_form
  conversions, `#`(Block), KL_block(Param), dual_KL(Block),
  KL_sum_at_s_to_height all evaluate correctly). The entire remaining
  builtin surface is exactly the 10 never-registered names: E2 trio +
  E3 four + shift_flip (in flight) + print_partial_block/
  print_partial_common_block (no fixture yet). Conversion arms are live
  but several lack dedicated fixtures — coverage gap, noted in
  REMAINING_BUILTINS.md.
- **Interpreter semantics pin (oracle-verified)**: implicit
  definition `x:=2` without prior `name : Type` ascription is REJECTED
  by the oracle ("Undefined identifier 'x' in assignment") — the Rust
  CLI matches. Probe files must ascribe types first (all fixtures do).
- **In flight**: agent-55 (shift_flip language layer, atlas-core,
  brief /tmp/slice_shift_flip_brief.md); agent-56 (E3 crate drivers:
  twisted_deformation_terms/twisted_KL_sum/twisted_deformation/
  block_deformation_to_height, atlas-real-group, brief
  /tmp/slice_e3_brief.md — rank-0 integral-subsystem path only,
  twisted_full_deform builds on be0e16c's extended_finalise).
- **Queue**: (1) collect 3541690 → pcb meta; (2) agent-55 delivery →
  shift_flip FixturePlan (snippet in queue doc) + differential;
  (3) E2 language layer → ext_finalise FixturePlan + differential;
  (4) agent-56 delivery → E3 language layer → twisted_family +
  block_deform differentials; (5) print_partial_* fixtures.

## Checkpoint - 2026-08-12d (print_x verified_hpc; crate bugs fixed; dual_KL differential in flight)

- **print_x(±) closed**: differential **3540739** @ 3908db4 — 217
  fixtures, 216 PASS, 0 FAIL (container_syntax_errors PARTIAL is the
  permanent known item). Both metas bumped to verified_hpc (`c3ba84a`).
- **agent-52 crate fixes verified + committed (`f399fc8`)**: kl_table
  RT2 arm uses inverse_Cayley first term; complete_primitives reads the
  in-progress column (kl.cpp:129-131/566-574); RealProjection is seeded
  at the canonical involution and transported along cross-action BFS
  (involutions.cpp:242-243) via new `real_projection.rs` — B2 x=4
  λ=[2,2] now correct. dual_KL replay (incl. B2) byte-identical;
  369 crate tests + 230 atlas-core tests + clippy/fmt all green.
- **dual_KL registered + differential in flight**: FixturePlan entries
  (33 lines/33 events) committed `dafdc03`, harness 10/10. Differential
  **3541634** submitted on a clean dafdc03 checkout.
- **HPC sync lesson — use git bundle, not fetch**: HPC→GitHub https
  fetch is flaky (worked once, then silent failures). Deterministic
  path: local `git bundle create /tmp/x.bundle <base-sha>..main` →
  scp → HPC `git fetch /tmp/x.bundle main && git checkout -f <sha>` →
  verify `git status --porcelain --untracked-files=all` empty (watch
  for stray `atlas-pipeline-swap-*.out` files making the tree dirty) →
  sbatch. Local sync uses a clean worktree `/tmp/atlas-hpc-sync`
  (`git worktree add --detach`) so agents' dirty trees never leak into
  the rsync.
- **In flight**: agent-53 (print_common_block language layer,
  atlas-core, resumed after a provider-quota kill); agent-54 (E2 crate
  drivers scaled_extended_finalise/extended_restrict_to_K/
  extended_finalise, atlas-real-group, purely additive constraint).
- **Queue**: (1) collect 3541634 → dual_KL metas; (2) agent-53 delivery
  → pcb FixturePlan registration (snippet ready in
  docs/slices/post_weyl_lang_queue.md:296-308) + differential; (3)
  agent-54 delivery → E2 language layer (typed.rs wrappers per
  /tmp/slice_e2_brief.md precondition order); (4) E3 twisted family +
  block_deform (/tmp/slice_e3_brief.md, depends on E2's
  extended_finalise).

## Checkpoint - 2026-08-12c (E1 crate + dual_KL + print_X landed; two crate bugs found; HPC offline)

- **E1 crate landed (`f12b27b`, agent-47)**: `ext_param.rs` (2329 lines,
  full length-3 `star` cases + ExtParamOracle), `matreduc.rs`, `ext_kl.rs`
  contributions, `rep_context.rs` `orientation_number`. 366
  atlas-real-group tests green. Key fix: malachite 0.10 `Rational`
  numerator is unsigned — `rational_coweight_dot` was dropping signs.
- **dual_KL_block language layer (`ced33b8`, agent-49)**: typed.rs
  registers `dual_KL_block: Param -> ([Param],int,mat,[vec])` in UPSTREAM
  order (the acceptance pitfall was avoided); domain_builtins.rs arm at
  :12202 + `common_block_srms` helper (:2680, faithful port of
  blocks.cpp:733-1076). Replay: A1/A2/rejected byte-identical; **B2 has
  2 diffs = crate bugs** (below).
- **print_X (`a2979ad`)**: ~25 lines — typed.rs
  `domain_printer_builtin("print_X", InnerClass)` + print_text arm +
  `print_x` helper (GlobalKgb::build + print_layout().render()). Replay
  print_x(±) all pass. **Reminder: `cargo build -p atlas-cli` before any
  replay** — the scripts call ./target/debug/atlas-cli, stale binaries
  report "Undefined identifier".
- **print_x registered in FixturePlan (`b0fc234`)**:
  hpc/pipeline_swap_diff.py runnable=(2,4,5,7,9,10,12,14,15),
  silent=(1,3,6,8,11,13); harness unit tests 10/10 green. All four
  commits pushed to origin/main (HEAD b0fc234).
- **Two crate bugs exposed by dual_KL B2 — agent-52 in flight fixing
  (exclusive atlas-real-group)**:
  - Bug 1: kl_table.rs RT2 arm first term must be
    `inverse_cayley(x,s).0`, not `cross` (kl.cpp:416-425).
  - Bug 2: rep_context.rs:633 RealProjection lift must NOT be recomputed
    from θ; per involutions.cpp:242-243 the lift_mat must be transported
    along the generation path via simple_reflect. Evidence: B2 x=4
    θ=[[-1,0],[2,1]] γ=[2,2], oracle lift column [2,-2]ᵀ vs crate
    [-2,2] → λ printed [0,4] should be [2,2].
- **print_common_block language layer — agent-53 in flight** (exclusive
  atlas-core, brief /tmp/slice_pcb_brief.md). Reminded that the lift bug
  may cause λ diffs: record precisely, do not work around.
- **HPC offline + differential 3540635 FAILED (12s)**: "declared
  Atlas-Rust source state does not match the submit checkout". Root
  cause: rsync excludes .git, so HPC HEAD sat at 4f363ef while the job
  declared DIRTY_TREE=false. Also: **HPC could not fetch — origin used
  git@github.com and the cluster has no deploy key; origin has been
  switched to https://github.com/jiajunma/atlas-rust.git (repo is public,
  https fetch verified working)**. Correct flow (hpc/README.md:42-57): push
  origin → on HPC `git fetch origin && git checkout <sha>` (**HPC git is
  old: `checkout --detach <sha>` fails with "does not take a path
  argument"; plain `git checkout <sha>` works**) → `git status --porcelain
  --untracked-files=all` → then sbatch. SSH to majj@10.26.14.64 timed out
  mid-fetch (was fine 40 min earlier; login node flapping). On recovery:
  fetch+checkout b0fc234, then resubmit
  `ATLAS_DIRTY_TREE=false sbatch --partition=fat --time=01:00:00
  --mem=32G --export=ALL,TIMEOUT=3600 hpc/pipeline_swap_diff.sbatch`
  (print_x already registered). If status shows untracked leftovers
  (e.g. fiber_probe.rs) making the tree detected-dirty, declare true to
  stay consistent.
- **Side findings logged, not touched**: typed.rs:5216 KL_block skip
  registration order is misaligned `([Param],mat,[vec],int)` vs upstream
  `([Param],int,mat,[vec])` — KL_block still skipped, harmless.
  common_block_gamma_lambdas/torus_part give wrong gamlam for this
  block's z=6,7,10 — deprecated.
- **Queue after these land**: (1) HPC recovery → resubmit differential →
  print_x meta → verified_hpc; (2) agent-52 delivery → three gates + B2
  replay → commit atlas-real-group → E2 crate drivers (brief
  /tmp/slice_e2_brief.md); (3) agent-53 delivery → three gates + pcb
  replay → commit atlas-core → dual_KL+pcb FixturePlan registration
  (snippet in docs/slices/post_weyl_lang_queue.md:242) → differential →
  metas; (4) E2 language layer → E3 twisted family + block_deform
  (brief /tmp/slice_e3_brief.md; E3 depends on E2's extended_finalise).
- Ownership discipline unchanged: one agent per crate at a time;
  examples/fiber_probe.rs is agent-47 debug residue — never commit,
  never delete.

## Checkpoint - 2026-08-12b (slice C closed; final queue is 6 fixture pairs)

- **Slice C closed (`f612507`)**: differential **3538976** (commit
  d19090a) — 215 fixtures, 0 FAIL (container_syntax_errors PARTIAL is
  the permanent known item). print_gradings/print_gradings_rejected/
  real_weyl_print/real_weyl_print_rejected bumped to verified_hpc.
  Note: print_blockstabilizer was folded into the print_gradings
  fixture; no separate fixture exists.
- **ext_finalise(+_rejected) reference verified**: HPC capture
  **3538977** byte-identical to local pinned-oracle captures
  (stdout+stderr, all 4 files); metas bumped to
  verified_hpc_reference. rust_status stays pending_hpc_differential
  until the E1 crate lands (scale_extended/K_type_pol_extended/
  finalize_extended need ext_param + star).
- **Remaining queue — every remaining fixture already has
  verified_hpc_reference; only rust implementation + differential
  left** (6 pairs):
  1. dual_kl_block (±rejected) — agent-49 in flight (atlas-core).
     Crate dual landed 1e7fcc4. Pitfall: typed.rs KL_block skip
     registration return order is misaligned with upstream; new arms
     must follow upstream `([Param],int,mat,[vec])` order.
  2. block_deform (±rejected) — E3, blocked on E1 crate.
  3. twisted_family (±rejected) — E3, blocked on E1 crate.
  4. ext_finalise (±rejected) — E2, blocked on E1 crate.
  5. print_x (±rejected) — global_KGB crate landed 64048ac
     (GlobalKgbPrint::render() byte-identical to the 3 references);
     only language-layer registration left (atlas-core).
  6. print_common_block — **recon done (agent-50, brief
     /tmp/slice_pcb_brief.md)**: the feared srm pool/Rep_table port is
     NOT needed — the pool is pure memoization, fresh-build-per-call is
     output-equivalent for dominant gamma (same precedent as
     partial_block/KL_block). Real work: ~180-260 lines
     domain_builtins.rs render helper + ~12 lines typed.rs (new
     print_common_block arm + print_block(Param) overload branch).
     Pitfalls: header N matched on (x,gamma-lambda) not x alone;
     content/stars gamma split (made-dominant for block, original gamma
     for `*`); print_block(Param) at fixture line 19 dedups silent.
     Depends on agent-49's uncommitted srm helpers → lands after
     dual_KL_block. No rejected fixture by documented design.
- **Completion-criterion audit (2026-08-12)**: LANGUAGE.md matrix rows —
  all language rows supported; "domain objects" partial = exactly the
  remaining queue below. Two rows are explicitly deferred OUTSIDE the
  language-only gate by the doc itself (LANGUAGE.md:66-68): readline
  completion (TTY) and KL binary file formats (filekl.w is used only by
  stand-alone utilities; zero interpreter references — no Atlas-language
  builtin reads/writes KL files). Both need a user decision at
  completion time; neither blocks the 7-pair queue.
- **Acceptance ordering with both crates dirty**: when agent-49
  delivers, commit ONLY crates/atlas-core paths (dual_KL_block slice);
  atlas-real-group stays dirty for agent-47. cargo test -p atlas-core
  needs atlas-real-group to compile — agent-47's checkpoint discipline
  (E1a/E1b/E1c each all-green) keeps it compiling; if it is mid-checkpoint
  red, wait for its next green checkpoint rather than touching its files.
- **In flight**: agent-47 (E1 ext_param+star crate, exclusive
  atlas-real-group, brief /tmp/slice_e_brief.md, checkpointed
  E1a/E1b/E1c); agent-49 (dual_KL_block language layer, exclusive
  atlas-core, brief /tmp/slice_d_brief.md). Both resumed at ~16:25
  with 2h timeouts; on timeout resume the same agent id.
- **shift_flip fixtures closed the last fixture gap** (`ca88ac1` +
  `5a2e324`): every remaining builtin now has a verified_hpc_reference
  fixture; only implementation + differential remain. shift_flip
  itself needs E1 (shifted_default_extension + is_default); accepted
  cases all return false — ~1300 oracle probes found no true case
  (noted in meta + queue doc §4). print_x FixturePlan line alignment
  pre-analysed into the queue doc snippet (`7a19b87`).
- Per-slice closure recipe unchanged: three gates → local replay
  byte-compare → commit → register FixturePlan (watch print-fixture
  line/event alignment, silent_lines pattern from d19090a) → rsync +
  archive + sbatch fat differential (ATLAS_DIRTY_TREE=false) → on
  0 FAIL bump metas.

## Checkpoint - 2026-08-12a (ext builtins landed; global_KGB landed; differential 3537192 in flight)

- **agent-36 ext three-builtin registration delivered and committed
  (`9ba51f0`)**: extended_block/raw_ext_KL/partial_extended_KL_block.
  Acceptance: local replay byte-identical stdout on both fixtures
  (accepted exit 0, rejected exit 1); rejected stderr differs only by
  CLI diagnostic framing (established convention) — 9/9 Diagnostic
  messages match reference events via parse_cli_diagnostics. Three
  gates clean (230 atlas-core lib tests, clippy -D warnings, fmt).
  Root causes fixed this run: RealTypeII parity gate in
  common_block_members (blocks.cpp:914), fiber-relative length
  normalization for raw_ext_KL stops, entry-fiber truncation for
  partial_extended_KL_block (ext_kl.cpp:962-963). Soft flags for the
  differential: parity gate uses entry lambda_rho (not per-element
  srm); in-fiber condense untested on singular-gamma + high-entry
  cases; partial KLV from full-block submatrix.
- **agent-37 global_KGB delivered and committed (`64048ac`)**:
  crates/atlas-real-group/src/global_kgb.rs (1370 lines) — upstream
  kgb::global_KGB + kgb_io::print_X layout; render() byte-identical to
  the three verified print_x reference outputs (SC A1/adj A1/SC B2).
  339 crate tests pass. Known gaps (deferred): second constructor from
  a GlobalTitsElement seed, lookup/compact/descent storage, non-id
  delta base-fiber basis order, semisimple-rank-0 edge (blocked by a
  weyl_transducer.rs:485 panic in shared infra).
- **HPC differential 3537192 submitted** (fat, TIMEOUT=3600,
  ATLAS_DIRTY_TREE=false) covering the two pre-registered ext_block
  FixturePlans. On 0 FAIL: bump ext_block metas to verified_hpc.
- **Slice briefs staged**: /tmp/slice_a_brief.md (coroot_queries +
  root_numbering, 14 items), /tmp/slice_b_brief.md (orbit_ws 4 +
  poly_surface 8; queue doc corrected — K_type_pol has only the
  (ParamPol->KTypePol) overload, first/last_term arms are complete and
  flip-ready), /tmp/slice_c_brief.md (print_gradings + print_real_Weyl
  + print_blockstabilizer; real_weyl.rs API sufficient).
- **Dispatched**: agent-40 slice A implementation (exclusive
  atlas-core); agent-41 atlas-real-group patch (RootSystem
  min_roots_for/min_coroots_for for slice B + public
  bourbaki_permutation for slice C — the two crate gaps the briefs
  identified).

## Checkpoint - 2026-08-11d (ext references verified; global_KGB crate dispatched)

- **ext_block(+_rejected) references verified**: fixtures + pending
  metadata committed `c342494`; HPC reference capture **3536831** PASS,
  stdout+stderr byte-identical to local pinned-oracle captures for both
  fixtures; metas/events upgraded to verified_hpc_reference (`9fb7eb8`).
  Quirk: first capture submission 3536828 FAILED at
  validating_declared_source_state — after `git archive HEAD | ssh tar -xf -`
  the HPC checkout is CLEAN even when the local tree has uncommitted
  agent edits, so always declare `ATLAS_DIRTY_TREE=false` for archive-laid
  trees (the sbatch compares declared vs detected and rejects mismatches).
- **All remaining-slice fixtures now verified_hpc_reference** (11 pairs):
  root_numbering, coroot_queries, orbit_ws, print_gradings, poly_surface,
  real_weyl_print, print_x, print_common_block, dual_kl_block,
  twisted_family, block_deform (+_rejected each). FixturePlan snippets sit
  at queue §5 head — register only with the implementing slice.
- In flight: agent-36 ext three-builtin registration (exclusive on
  atlas-core, third run — fixing a gamma_lambda propagation panic: got
  [-1,-1]/1, expected [-3,0]/2, missing rho_r correction semantics,
  anchors at queue §gamma_lambda/repr.cpp:890-995); agent-37 global_KGB
  crate slice for print_X (new module in atlas-real-group, upstream refs
  kgb.h:213-266 + kgb.cpp + kgb_io.cpp; fixture print_x.events.json gives
  byte-exact layout targets).
- Next after agent-36: acceptance gates (three gates + PROP-eprintln
  sweep + local CLI replay byte-diff vs /tmp/ext_ext_block*.stdout/stderr,
  sha fingerprints in /tmp/ext_fixture_sha.txt), then I commit +
  FixturePlan registration + HPC differential (fat, TIMEOUT=3600).

## Checkpoint - 2026-08-11c (alcove/FPP verified; 4 fixture pairs prepped)

- **Differential 3533851 PASS**: 201 fixtures, 200 PASS / 1 known PARTIAL
  (container_syntax_errors) / 0 FAIL; alcove_fpp(+_rejected) metas now
  verified_hpc (`7032dd9`).
- **Four fixture pairs prepped and committed** (oracle-validated under
  true harness conditions — NO basic.at preload, source+quit):
  root_numbering(+_rejected) and coroot_queries(+_rejected) (`e6829d8`),
  orbit_ws(+_rejected) (`6de7f27`), print_gradings(+_rejected)
  (`b8ee71a`). They wait for their implementation slices; events/meta/
  FixturePlan registration belong to those slices. Probe facts are in
  queue §5.7/§5.3/§5.2/§4. Gotcha recorded: two_rho_check's [int]/
  predicate overloads are basic.at script-level, unavailable in fixtures;
  adjoint(LieType-with-torus) errors "Sub-lattice matrix should have
  size 2x2" (Cartan_matrix(lt) is semisimple-sized); basic_orbit_ws
  convention is v[0..stab_rank]=stab walls + v[stab_rank]=final root.
- **Recon now covers every remaining family**: twisted/deform crate gaps
  = ext contributions (repr.cpp:1901-1931), scaled_extended_finalise
  (ext_block.cpp:2736-2807), extended_finalise, extended_restrict_to_K,
  twisted_KL_column_at_s (~2300-2424), block_deformation_to_height
  (repr.cpp:2027-2124); finalise three scoped (queue §5.5); language
  emulation template = full_deformation_terms (domain_builtins.rs:2050).
  Note: pinned atlas has NO ext_param/star builtins — "ext_param+star"
  names the C++ machinery slice = shift_flip + finalise three + twisted
  family.
- In flight: agent-35 RealWeyl crate slice (exclusive on
  atlas-real-group); agent-36 ext three-builtin registration
  (extended_block/raw_ext_KL/partial_extended_KL_block, exclusive on
  atlas-core). Next after agent-36: small sweep (8 near-flips) +
  root-numbering 6 (fixtures ready), then orbit/ladder (fixture ready),
  then print_gradings (fixture ready).

## Checkpoint - 2026-08-11b (alcove/FPP landed; ext slice in flight)

- **alcove/FPP slice landed (`53581d8`, agent-30)**: alcove_center,
  alcove_root_vertex, FPP_numers, FPP_w_shifts registered
  (typed.rs:4864-4898; helpers domain_builtins.rs ~3736-4730 +
  arms ~8680-9030). Three real bugs fixed by oracle comparison:
  additive_closure defaults for_coroots=true (rootdata.h:119-120);
  CenterClassifier shifts unslice maps bit *positions* not masks;
  has_descent is left descent (w^-1 action). Fixtures alcove_fpp
  (+_rejected) byte-identical to oracle locally; 230 lib tests,
  clippy/fmt clean. HPC differential **3533851** submitted (fat, 201
  fixtures) — on PASS upgrade the two metas to verified_hpc with
  differential_job=3533851.
- **root_index coordinate question RESOLVED** (queue §5.7, commit
  eae26a3): no bug — vec coordinates are in each datum's native lattice
  basis (adjoint: roots in simple-root coords, coroots in fundamental
  coweight coords = Cartan columns; simply_connected: roots in
  fundamental-weight coords = Cartan rows, coroots in simple-coroot
  coords). Miss sentinel = signed numPosRoots (B2: 4), no dimension
  check. Negative index -k = negative of posroot k-1. Fixtures
  root_numbering.atlas(+_rejected) drafted and oracle-validated
  (untracked, for the root-numbering implementation slice; 6 builtins:
  root_expression/coroot_expression/root_index/coroot_index/
  root_involution/root_permutation — root/coroot/is_long_root already
  live).
- **srm pool anchors** (queue §5.5, commit fd2b4c1): print_common_block
  family needs Rep_table/block_modifier semantics (repr.h:485-499,
  534+); Rust block/partial_block emulate lookup_full_block via
  common_block_members but produce no bm display data (w word,
  simple_pi, shift) — that is the real gap for the print trio.
- In flight: agent-35 RealWeyl crate slice (exclusive on
  atlas-real-group); agent-36 ext three-builtin registration
  (extended_block/raw_ext_KL/partial_extended_KL_block, exclusive on
  atlas-core). Next after agent-36: small sweep (8 near-flips) +
  root-numbering 6 (fixtures ready), one combined language slice.

## Checkpoint - 2026-08-11 (199/199 verified; ext_kl landed; builtin reconciliation)

- **HPC differential `3533446` PASS**: 199 fixtures, 198 PASS / 1 PARTIAL
  (the two permanent container_syntax_errors pendings) / 0 FAIL. The five
  metas (block_sizes, weyl_orbit, weyl_orbit_rejected, walls,
  walls_rejected) are upgraded to verified_hpc with
  differential_job=3533446 (`7a5eba5`). Every fixture in the harness is
  now verified_hpc.
- **ext_kl crate slice landed (`602fce6`, agent-33)**:
  crates/atlas-real-group/src/ext_kl.rs (1761 lines) — DescentTable
  (ext_kl.cpp:20-118, DEAD_END sentinel, prim_flip bitmap), ExtKlTable
  (KL_table :120-841 incl. do_new_recursion all seven tsx cases),
  condense (ext_block.cpp:2015-2048), ext_kl_matrix (:939-1020, survivors
  + parity sign flips). Sign convention: sign lives in
  descent_table::prim_flip, not the pool index; kl_pol_index returns
  (KLIndex, bool) mirroring upstream pair<KLIndex,bool>. Oracle anchors:
  A2 trivial-delta 6x6, A2 flip-delta (SL(3,R), transpose convention),
  Sp4 12-element non-degenerate (pool {0,1,q}, stops [0,4,7,10,12]);
  325 lib tests, clippy/fmt clean. One real bug found and fixed in
  review: get_mp/mu must read the in-progress working column during
  do_new_recursion (upstream loads column[y] before recursing,
  ext_kl.cpp:517). Frontend boundary deferred to the common-block slice:
  StandardRepr->extended-block entry, survivors->StandardRepr map,
  singular-orbit computation.
- **Builtin reconciliation (178 upstream vs 128 live, 50 missing)**:
  upstream atlas-types.w has 178 install_function names; typed.rs has
  128 live; 50 missing = 28 never registered + 22 skip-placeholder only.
  Note: several skip names (dual, inner_class, param, real_form,
  involution, twist, KL_block, dual_KL, K_type_pol, first_term,
  last_term, null_module, W_cells, two_rho_check, simple_coroots,
  poscoroots, coroot_radical, mod_central_torus_info, adjoint,
  KL_sum_at_s_to_height, truncate_above_height) have their MAIN overloads
  live — skip marks only partial signatures. Family breakdown of the 50:
  in-flight alcove/FPP (4, agent-30); Weyl remainder (affine_orbit_ws,
  basic_orbit_ws, root_ladder_bottoms, coroot_ladder_bottoms); root
  numbering family (6, oracle-root-numbering blocked); ext family (7 —
  crate side now ready: ext_block 28e6109 + ext_kl 602fce6);
  deform/twisted (8); print family (7); KType/Rep (6, mostly skip);
  small items (semisimple_rank etc).
- In flight: agent-30 alcove/FPP language slice (exclusive on
  atlas-core). Next language slice once atlas-core frees up:
  extended_block/raw_ext_KL/partial_extended_KL_block registration
  (wrappers atlas-types.w:7366-7431/8682-8728/7445-7468, pure format
  conversion over the now-complete crate side).

## Checkpoint - 2026-08-09 (Weyl builtins + B2 fiber fix landed)

- **Weyl layer (`9111b7d`, agent-30)**: walls/walls_attitude
  (alcoves.cpp:112-236), Weyl_orbit/Weyl_orbit_ws both orders
  (rootdata.cpp:1690-1876), from_dominant corrected (lattice_rank torus
  pass-through; real simple-root pairings; split error wordings).
  RootNumbering keys on coroot coordinates when the datum prefers
  coroots (rootdata.cpp:164-167). Fixtures frozen (`afce162`):
  weyl_orbit(+_rejected)/walls(+_rejected), events from the local pinned
  oracle via /tmp/stdout_to_events.py (converter: ReportLine/Value display
  folding/Diagnostic parsing), harness plans registered.
- **B2 block_sizes root cause fixed (`e83eea2`)**: fiberSize is the
  strong-real fiber orbit class size (innerclass.cpp:603-614), not the
  adjoint weak partition; `fiber_size` now delegates to
  `StrongRealClassification::fiber_size` (B2 rows restored in the
  fixture; oracle 4/5/12 reproduced). Local replay: 199 fixtures, 196
  PASS + the two known local FAILs (fromfile_accepted_b10, kgb_hasse
  30s timeout) + the known PARTIAL.
- **HPC differential `3533446`** submitted on fat (TIMEOUT=3600,
  --mem=32G) at dfb4366; on 0 FAIL upgrade the five metas
  (block_sizes + four weyl) from pending_hpc_differential to
  verified_hpc.
- **Known gap queued**: Weyl_orbit oversize-vector semantics
  (docs/slices/post_weyl_lang_queue.md §1.5); post-Weyl language queue
  in the same file (alcove/FPP anchors, ext_block registration, print
  family order).
- In flight: agent-33 ext_kl crate slice (ext_kl.cpp KL_table +
  descent_table + ext_KL_matrix); agent-30 alcove/FPP language slice.

- **ext_block core committed (`28e6109`)**: agent-32's slice — DescValue
  32-value classification + predicates, `extended_type`, `ExtBlock::build`
  (trivial block modifier), `induced`, `tune_signs` over an injected
  `StarOracle` seam, debug `check_quadratic`/`check_braid`; 322 lib tests,
  clippy/fmt clean. Both A2 anchors verified byte-for-byte against the live
  oracle: `extended_block(trivial(SU(2,1)), id)` (types
  [[2,2],[2,9],[9,2],[0,3],[3,0],[1,1]], links with UndefBlock=6) and the
  flip case (types |26|,|27|; fixed elements x=0,x=3). Oracle note: for
  SL(3,R) the distinguished involution prints as `[[1,0],[1,-1]]`
  (columns-as-images), and `extended_block` wants the rows-as-images
  transpose `[[1,1],[0,-1]]`; the raw diagram flip is rejected
  ("not distinguished").
- **B2 `block_sizes` root cause FOUND (was the "B2 fiber undercount"
  follow-up below)**: the oracle's `fiberSize` (innerclass.cpp:603-614) is
  the STRONG-real full-fiber orbit class size, but
  `domain_builtins.rs:4398` `fiber_size` counts ADJOINT-fiber elements of
  the weak partition. The crate's strong-real machinery is CORRECT: a
  probe (`crates/atlas-real-group/examples/fiber_probe.rs`, uncommitted)
  reproduces all nine oracle entries `| 0, 0, 1 | / | 0, 0, 4 | /
  | 1, 5, 12 |` for sc-B2 complex when fiber sizes come from
  `StrongRealClassification`, and per-Cartan square classes/orbits match
  upstream `print_strong_real` on BOTH the sc-B2 side and the adjoint-C2
  dual side, Cartan numbering included. Fix queued: switch `fiber_size`
  (and only it) to `context.strong.fiber_size(form, cartan)`, re-expand
  the block_sizes fixture with the B2 rows (old version at
  `git show 8097c05^:tests/fixtures/domain/block_sizes.atlas`),
  recapture + differential. Blocked only on agent-30's in-flight
  domain_builtins.rs edits (serial discipline).
- **False alarm retired**: the "adjoint B2 Cartan C1/C2 swap" suspicion was
  a probe artifact — the dual of sc-B2 is adjoint **C2**, not adjoint B2;
  against `adjoint(Lie_type("C2"),true)` the crate's numbering, involution
  matrices, occurrence counts and strong-real data all match upstream.
- Probe scripts (local oracle, run from `atlas-scripts/` with
  `< basic.at` + `< groups.at` preloaded): `/tmp/sr_b2.atlas`,
  `/tmp/sr_c2_adj.atlas`, `/tmp/eb_a2d.atlas`, `/tmp/eb_a2i.atlas`.
- agent-30 (Weyl builtins: walls/from_dominant/Weyl_orbit) still in
  flight on typed.rs/domain_builtins.rs; agent-32's ext_block follow-ups
  (ext_kl table, star/ext_param, twisted_deform) are next in the
  atlas-real-group lane.

## Checkpoint - 2026-08-06 (post-sweep repair)

HPC differential `3520281` (195 fixtures) reported 189 PASS / 5 FAIL /
1 PARTIAL. Root cause analysis and repair (`fe657cf`):

- `fundamental` / `cartan_matrix_type` / `integrality`: stale-tree config
  artifacts (the job ran mid-trim); all three PASS locally and need no
  code change.
- `block_sizes` / `simple_factors`: the `8097c05` trim left their
  events.json at `local_oracle_reference` status and block_sizes.meta.json
  with the pre-trim fixture sha — harness configuration errors, not
  behavior diffs. Repaired in place (status -> `verified_hpc_reference`,
  fixture sha refreshed); both fixtures byte-exact locally.
- The uncommitted `fiber_size` debug `eprintln` (B2 fiber-count
  investigation) was reverted; the WG3 timing leftovers in the W_graph
  builtin arm and the weyl_transducer test debug eprintln/clone-on-Copy
  were removed so clippy `-D warnings` and fmt are clean again (the
  `9b3d988` revert had also left typed.rs unformatted).
- Local replay: 192 PASS + the known `fromfile_accepted_b10` FAIL (HPC
  paths) + `kgb_hasse` local-timeout FAIL (E7 needs the HPC fat node,
  506s/12.4G in swap 3515688; harness default timeout is 30s). Gates:
  230 + 316 lib tests, clippy/fmt clean, harness 10/10.
- Repair differential submitted as job `3531606`; on zero FAIL the six
  3520214-3520219 fixtures (cofolded, block_sizes, fundamental,
  simple_factors, cartan_matrix_type, integrality) get `verified_hpc`.
  DONE: `3531606` (cpu partition) passed all six but FAILED `kgb_hasse`
  on the 30s harness timeout (E7 needs fat; CPU-time ~2571s with rayon);
  resubmitted on fat as `3531617` (`TIMEOUT=3600 sbatch --partition=fat
  --time=01:00:00 --mem=32G`) — **194 PASS / 1 PARTIAL / 0 FAIL**, and the
  six metas now carry `rust_status: verified_hpc` +
  `differential_job: 3531617`. All 197 reference metas are verified_hpc.
  Submission note: after a new local commit, re-archive the tree to HPC
  before sbatch or the dirty-tree guard aborts ("declared Atlas-Rust
  source state does not match the submit checkout").
- Open follow-up: the B2 fiber undercount that motivated the block_sizes
  trim (oracle `| 0, 0, 4 |`/`| 1, 5, 12 |` vs Rust `| 0, 0, 3 |`/
  `| 1, 3, 8 |`) is a REAL behavior gap parked by the A2-only trim —
  `fiber_size`/`CartanClass` label mapping undercounts weak forms in a
  fiber. Re-expand block_sizes to B2 once fixed.

## Checkpoint - 2026-07-31 (usage-limit handoff)

This checkpoint was committed while three slice agents were interrupted by a
provider 403 (usage limit). Everything in this section supersedes the queues
below until the slices land.

**In-flight WIP (committed as `chore: checkpoint ... WIP`, may not compile):**

- `crates/atlas-real-group/src/{error.rs,lattice.rs}` + new
  `ktype.rs`/`rep_context.rs`: agent-27's Rep_context crate milestone
  (`RepInvariantViolation` error variant, y_pack coset machinery). Direction
  reviewed, contents not fully audited.
- `crates/atlas-core/src/{syntax.rs,typed.rs}`: agent-29's L2 bison
  syntax-message slice (error-state probe tests). Partial.
- agent-28's L1 diagnostic-wording slice had no evaluator edits on disk yet
  when interrupted.

**Resuming the agents (if this session is alive):** `Agent(resume="agent-27"
/ "agent-28" / "agent-29", run_in_background=true)` — each retains full
context. A fresh agent can instead finish the slices by hand from the briefs.

**Persisted slice briefs:** `docs/slices/` holds all nine agent briefs
(`/tmp` is volatile; the originals were copied here):

- `agent_L1_prompt.md` — 4 diagnostic-wording contracts
  (`commands/assignment_errors`, `slice_errors`, `subscription_errors`,
  `eval/container_errors`). Upstream anchors: `axis.w:7092`
  (`' in ' << where << ' ' << e`), `axis.w:4289` (`e->print(o << " in slice
  ")`, `<=2` no space), `axis.w:4172/:4103/:8167`, `axis-types.w:3515`.
- `agent_L2_prompt.md` — 5 bison syntax-message contracts
  (`commands/{container_syntax_errors,invalid_token_continues,
  mismatched_delimiter_continues,nested_invalid_token_continues}`,
  `parse/negative_trailing_token`). Target messages like `syntax error,
  unexpected INT, expecting '\n'`; `parser.y:63` has `%define parse.error
  verbose`. The dangling `[` line of `container_syntax_errors` is excluded
  (oracle saw the capture-time appended `quit`).
- `agent_L3_prompt.md` — `set verbose` + `lex/basic`. Anchors:
  `parser.y:171-178` (SET IDENT option command, unknown option `'X' is not
  something one can set`), `main.w:495-516` and `:528-540` (the three trace
  lines `Expression before type analysis: `/`Type found: `/`Converted
  expression: `). Blocked on L2 releasing `lex.rs`.
- `agent_L4_prompt.md` — `negative/unterminated_string` recovery. Oracle:
  lexical warning `Closing string denotation.` + recovers the string +
  prints the Value + exit 0; needs a warning-level diagnostic that does not
  flip the exit code. Blocked on L2.
- `agent27_rep_context_prompt.md` + the four language briefs
  `agent_ktype_lang_prompt.md` / `agent_param_lang_prompt.md` /
  `agent_ktypepol_lang_prompt.md` / `agent_parampol_lang_prompt.md` — the
  six ktype/param-family contracts (`domain/ktype_basic{,_rejected}`,
  `ktypepol_basic`, `param_basic{,_rejected}`, `parampol_basic`), all gated
  on the Rep_context crate milestone. Serialization rule: only one
  language-layer agent at a time on `typed.rs`/`domain_builtins.rs`.

**Remaining work after these slices:** 17 frozen contracts total (all with
verified reference + events, fields checked): the 11 above plus the 6
ktype/param family. Then the final `docs/LANGUAGE.md` matrix refresh.
readline completion and KL file formats stay outside the language-only gate
(they need the Block/KL layer; `deform` is a later large item).

**Per-slice delivery loop (unchanged):** local three-piece gate
(`cargo test -p atlas-core --lib`, `cargo test -p atlas-real-group --lib`,
`cargo clippy -p atlas-core -p atlas-real-group --lib --tests -- -D
warnings`, `cargo fmt --all -- --check`) + verbatim fixture comparison +
full local pipeline replay (only `eval/fromfile_accepted_b10` may FAIL) +
`python3 hpc/test_pipeline_swap_diff.py` from inside `hpc/` (10 tests OK) →
wire into `hpc/pipeline_swap_diff.py` → sync HPC + submit differential →
report shows both fixtures PASS, zero FAIL → bump meta to
`rust_status: verified_hpc` + `differential_job` → record here → commit →
`rsync -az --delete .git/` to HPC.

## Language gate completed - 2026-08-01

The language gate is complete: **166 of 166 frozen fixtures carry
`verified_hpc`** (differential `3506798`: 165 PASS + the one known
PARTIAL `container_syntax_errors`, zero FAIL). HEAD is `c0f26b4`
(main). Working tree clean. The last contract, `domain/deform`, landed
VERBATIM in three sub-slices:

- `8b8bd14` — the KLV polynomial table (kl.cpp → kl_polynomial.rs +
  kl_support.rs + kl_table.rs), with the A2 quasisplit block's mu
  columns pinning the frozen deform sources exactly.
- `6e33e0d` — deformation_terms (repr.cpp:1933-2025, simplified for the
  contract: identity modifier, empty singular system, constant
  lambda_rho) and StandardRepr::deform_readjust (repr.cpp:622-654).
- `d9f1cb2` — the deform builtin (typed.rs + domain_builtins.rs): the
  evaluator runs finals_for_standard, builds the common block against
  the dual inner class's first real form, fills the KL table, and
  accumulates terms scaled by Split_integer(c,-c) = c(1-s).

The three fixture rows produce the frozen output: deform(x=3) reaches
x=2 and x=0, deform(x=4) reaches x=1 and x=0, each
`(1-1s)*parameter(x=N,lambda=[1,1]/1,nu=[0,0]/1) [4]`; deform(x=5,
gamma=0) prints "Empty sum of standard modules" (its final is x=0 of
length 0, so deformation_terms returns the null result). 229 atlas-core
+ 305 atlas-real-group tests pass; clippy and fmt clean.

The remaining porting work is no longer gated on language contracts:
`twisted_deform`/`block_deform`/`full_deform`/KL sums (the same KL
table, extended with the twisted variant), the Param `cross`/`Cayley`
transforms (need the integral SubSystem), the KL/file formats (filekl
adapter), and readline completion.

The per-slice differential
chain (3506234, 3506272, 3506287, 3506321, 3506358, 3506387, 3506433,
3506622, …) ran the entire plan with zero FAIL across every run (only
`container_syntax_errors` stayed PARTIAL for its two permanent pending
cases; its meta was upgraded at `53ebfba`).

The design is in `docs/DEFORM_DESIGN.md`; the three briefs in
`docs/slices/` (agent_deform_kl_core_prompt.md,
agent_deform_terms_prompt.md, agent_deform_lang_prompt.md) match the
three landed sub-slices.

## Live continuation - 2026-08-01 (Param predicates/transforms)

The Param predicate/transform surface is landed and HPC-verified. HEAD is
the param_transforms commit; differential `3506622` ran 165 fixtures with
**zero FAIL** (one PARTIAL: the two intentional `container_syntax_errors`
pending cases) and PASSES `domain/param_transforms` (reference captured
by `3506620`). Its meta carries `rust_status: verified_hpc` +
`differential_job: 3506622`.

Implementation: the Param surface now registers `is_dominant`/
`is_semifinal`, the `dominant`/`normal` transforms
(`StandardRepr::made_dominant`/`normalised`, repr.cpp:1507-1561), and
Param equivalence (`StandardRepr::equivalent`, repr.cpp:1563-1576) with
the real-form mismatch gate.

## Live continuation - 2026-08-01 (ParamPol/Param operations)

The ParamPol/Param operation surface is landed and HPC-verified. HEAD is
the param_pol_ops commit; differential `3506433` ran 164 fixtures with
**zero FAIL** (one PARTIAL: the two intentional `container_syntax_errors`
pending cases) and PASSES `domain/param_pol_ops` (reference captured by
`3506427`). Its meta carries `rust_status: verified_hpc` +
`differential_job: 3506433`.

Implementation: `K_type_pol(ParamPol)` restricts every term to K (`sr_K`)
and re-expands through `finals_for` (atlas-types.w:7717-7730);
`last_term(ParamPol)` mirrors first_term; `RepContext::scale`
(repr.cpp:701-709) replaces a parameter's infinitesimal character along
its nu direction for the `(Param,rat)` wrapper, and the `(ParamPol,rat)`
scaling re-expands every scaled term through `finals_for`
(repr.cpp:1161-1170).

## Live continuation - 2026-08-01 (branch: deform-family slice 3)

The branch surface is landed and HPC-verified. HEAD is the branch
commit; differential `3506410` ran 163 fixtures with **zero FAIL** (one
PARTIAL: the two intentional `container_syntax_errors` pending cases) and
PASSES `domain/branch` (reference captured by `3506405`). Its meta
carries `rust_status: verified_hpc` + `differential_job: 3506410`.

Implementation: the branch wrapper (atlas-types.w:6055-6070) iterates
`Rep_context::branch` (K_repr.cpp:592-622) — repeatedly promote the least
remainder term into the result and subtract its `K_type_formula` (scaled
by the lead coefficient) from the remainder; the formula's own lead term
cancels the remainder's copy (keeping the lead IN the remainder while
subtracting is what terminates the loop). Negative bounds report
`Maximum level in branch cannot be negative` before the no-value gate.

The `deform` contract is FROZEN (fixture + events + meta, reference job
`3506415`): `domain/deform.atlas` on A2 su(2,1) pins the nontrivial
deformation of `param(x3/[0,0]/[1,1]1)` and `param(x4/[0,0]/[1,1]1)`
(`(1-1s)*parameter(x=2,...)[4]` + `(1-1s)*parameter(x=0,...)[4]`, and
the x4 variant with x=1/x=0) plus the length-0 empty sum. The next
implementation slice is the block/KLV machinery:
`Rep_table::lookup` (partial common block via `block_modifier`),
`contributions(block, singular, y)`, `deformation_terms`
(repr.cpp:1933-2025: KLV polynomials evaluated at q=-1 with the
alternating-column signs, the `remainder`/`acc` inversion loop, and the
orientation-number phases), the `kl::KL_table` (kl.cpp), and the
`blocks::common_block` structure (blocks.cpp) — the largest remaining
port.

## Live continuation - 2026-08-01 (K_type_formula: deform-family slice 2)

The K-type formula surface is landed and HPC-verified. HEAD is the
ktype_formula commit; differential `3506400` ran 162 fixtures with
**zero FAIL** (one PARTIAL: the two intentional `container_syntax_errors`
pending cases) and PASSES `domain/ktype_formula` (reference captured by
`3506396`). Its meta carries `rust_status: verified_hpc` +
`differential_job: 3506400`.

Implementation: `RepContext::k_type_formula` (K_repr.cpp:549-591) on top
of new foundation pieces — `RationalWeight::scale`/`dot_coroot`,
`height_bound` (the dominant-cone orthogonal projection with projector
vectors), `root_status_at` (the descent conjugation of kgb.cpp:819-830
for arbitrary roots), and `monomial_shift` (lambda shift + re-elected
coset representative + recomputed height). The formula expands the KGP
set by the nilpotent `(1-X^alpha)` factors of the parabolic, prunes by
`height_bound`, and re-expands through `finals_for`; the wrapper gates on
`is_semifinal` and maps a negative bound to the unbounded level.

## Live continuation - 2026-08-01 (KGP_sum: first deform-family slice)

The first deform-family surface is landed and HPC-verified. HEAD is the
kgp_sum commit; differential `3506387` ran 161 fixtures with **zero
FAIL** (one PARTIAL: the two intentional `container_syntax_errors`
pending cases) and PASSES `domain/kgp_sum` (reference captured by
`3506383`). Its meta carries `rust_status: verified_hpc` +
`differential_job: 3506387`.

Implementation: `KType::kgp_set` (K_repr.cpp:398-464) makes the input
theta-stable, collects the real-simple Levi generators, and BFS-explores
inverse-Cayley splits and complex crosses in the upstream discovery
order; the `KGP_sum` wrapper (atlas-types.w:5995-6010) gates on
`is_semifinal` before its no-value point (`K-type has parity real roots
(so not semifinal)`) and returns the row of length-parity-signed
`(int, KType)` pairs.

## Live continuation - 2026-08-01 (KTypePol/ParamPol arithmetic surface)

The pol arithmetic surface is landed and HPC-verified. HEAD is
`ef109af` (main); differential `3506368` ran 160 fixtures with **zero
FAIL** (one PARTIAL: the two intentional `container_syntax_errors`
pending cases) and PASSES `domain/ktypepol_arithmetic` and
`domain/parampol_arithmetic` (reference captured by `3506344`). Their
meta files carry `rust_status: verified_hpc` + `differential_job:
3506368`. The domain layer is complete at 81 of 81 frozen contracts.

Implementation (commit `ef109af`):

- Binary `+`/`-` on (KTypePol,KTypePol) and (ParamPol,ParamPol) merge
  like terms in the upstream pol term order (mismatch wordings `adding
  two K_types` / `subtracting two K_types` / `adding two modules` /
  `subtracting two modules`).
- `+(KTypePol,(Split,KType))` (add_K_type_term_wrapper): the explicit
  Split coefficient scales each final expansion term.
- `*(Split,KTypePol)` / `*(Split,ParamPol)` (split_mult_*_wrapper):
  every coefficient is multiplied by the Split, with the zero-divisor
  filtering — a scalar multiple of 1-s drops terms whose e-f vanishes, a
  multiple of 1+s drops terms whose e+f vanishes.
- `truncate_above_height(Pol,int)`: terms with height <= bound survive; a
  negative bound keeps everything.
- Binary `=`/`!=` on the pols via structural equality.

Local gate: 293 atlas-real-group + 229 atlas-core tests pass; clippy and
fmt clean; the eight ktype/param-family fixtures VERBATIM; the wired local
pipeline reports 158 PASS + 1 PARTIAL + the known `fromfile_accepted_b10`
FAIL; harness 10/10.

## Live continuation - 2026-08-01 (non-final KTypePol/ParamPol expansion)

The non-final pol contracts are landed and HPC-verified. HEAD is
`f4d5798` (main); differential `3506331` ran 158 fixtures with **zero
FAIL** (one PARTIAL: the two intentional `container_syntax_errors`
pending cases) and PASSES `domain/ktypepol_nonfinal` and
`domain/parampol_nonfinal` (reference captured by `3506276`). Their meta
files carry `rust_status: verified_hpc` + `differential_job: 3506331`.

Implementation (commit `f4d5798`):

- `KType::finals_for` (K_repr.cpp:290-396) and
  `RepContext::finals_for_standard` + `expand_final`
  (repr.cpp:1205-1309): crosses, type-1/type-2 Cayley and inverse-Cayley
  splits, singular-compact drops, and parity-real wall projections, with
  the multiplicity signs. The language layer now expands non-final
  KTypes/Params in the pol `+`/`-` wrappers and merges like terms in the
  upstream term order (`K_type_pol`: height asc, x asc, lam_rho lex;
  `SR_poly`: height asc, x desc, y bits, gamma cross-multiplied).
- **Projection-sweep fixes (root cause of a hang + a sign bug):**
  `gcd_sweep` now reduces a LOCAL row copy like upstream's `gcd` (the old
  code read and wrote the working matrix directly, applying the pivot
  multiple twice — the A2 su(2,1) involutions made it spin forever), and
  the pivot NEGATION is applied only to that local copy: the oracle build
  does not record `col(mindex,mindex) = -1` in the column ops (release
  asserts are off), which fixes the elected lambda-rho sign for the
  singleton-negative-pivot-with-swap involution — `K_type(x4,[1,0])` keeps
  `[1,0]` (the un-negated basis `(2,-1),[1,0]`) instead of electing
  `[-1,1]`. Verified against 14 oracle `%K_type` probes on su(2,1) and the
  compiled upstream `matreduc` for all four involutions. The regression
  test `a2_su21_context_builds_all_involutions_and_pins_nonfinal_anchors`
  pins the elected representatives.

Local gate: 293 atlas-real-group + 229 atlas-core tests pass; clippy and
fmt clean; the six ktype/param-family fixtures VERBATIM; the wired local
pipeline reports 156 PASS + 1 PARTIAL + the known `fromfile_accepted_b10`
FAIL; harness 10/10.

## Live continuation - 2026-08-01 (L3/L4: set verbose + string recovery)

The last two frozen legacy contracts are landed and HPC-verified. HEAD is
`41b2dbe` (main); differential `3506272` ran 156 fixtures with **zero
FAIL** (one PARTIAL: the two intentional `container_syntax_errors`
pending cases) and PASSES both:
`lex/basic` (`set quiet`/`set verbose` + the verbose analysis trace) and
`negative/unterminated_string` (lexical recovery with exit 0). Their meta
files carry `rust_status: verified_hpc` + `differential_job: 3506272`.
**All 17 frozen contracts from the 2026-07-31 checkpoint are now
verified**; the language-only gate is complete.

Implementation (commit `41b2dbe`):

- Session verbosity lives in `TypedContext` (`verbosity: u8`, default 0):
  `Command::SetOption` handles `quiet` (0) and `verbose` (1) per
  parser.y:171-178; unknown options report `'X' is not something one can
  set` through the span-less diagnostic header convention. The grammar
  gained the `set IDENT` command production (before the binding forms, so
  `set f = ...` still parses as bindings).
- The verbose trace (main.w:495-516, 528-540) emits three `Output`
  events per accepted expression command: `Expression before type
  analysis:` (via `compact_expression`), then `Type found:` and
  `Converted expression:` (new `compact_typed_expression`: denotations
  print their value, identifiers their name, calls `name(args)`; other
  shapes fall back to `<expression>` and are not oracle-verified).
  `TypedCommandEvent::Output` was added and flows to `SessionEvent::Output`.
- Unterminated strings stay a lexical recovery with the `Closing string
  denotation.` message (lexer.w:311-320) but are now `Diagnostic::warning`
  (new `warning` flag + `Diagnostic::warning` constructor); the session
  frame reports the warning without setting `clean=false` or aborting an
  include, so the run exits 0 and continues evaluating the recovered
  string.
- The missing `:=` bison row was added (`bison_token_name` → `:=`,
  `bison_expecting` → `'='`) so `let x := 42 in ...` reports the frozen
  `syntax error, unexpected :=, expecting '='`.
- Harness: `validate_plan` now accepts runnable lines that produce
  several events (verbose trace = 3 stdout lines + Value for one source
  line; a lexical warning rides with its recovered Value), and the two
  fixtures are wired with explicit line/event selections (`set verbose`
  is silent).

Local gate: 229 atlas-core + 292 atlas-real-group tests pass; clippy and
fmt clean; both fixtures VERBATIM; the wired local pipeline reports
154 PASS + 1 PARTIAL + the known `fromfile_accepted_b10` FAIL (HPC
paths); `hpc/test_pipeline_swap_diff.py` 10/10.

## Live continuation - 2026-08-01 (ktype/param language slice)

The six K-type/standard-parameter contracts are now landed and
HPC-verified. HEAD is `dbf02fe` (main); differential `3506258` ran 154
fixtures with **zero FAIL** (one PARTIAL: the two intentional
`container_syntax_errors` pending cases) and PASSES all six:
`domain/ktype_basic{,_rejected}`, `domain/param_basic{,_rejected}`,
`domain/ktypepol_basic`, `domain/parampol_basic`. Their meta files carry
`rust_status: verified_hpc` + `differential_job: 3506258`. The domain
layer is COMPLETE (77 of 77 frozen domain contracts), and
`docs/LANGUAGE.md` reflects that.

Implementation (commit `dbf02fe`, no atlas-real-group changes):

- `DomainValue` gained `KType`/`KTypePol`/`Param`/`ParamPol` variants
  carrying the owning `Arc<RealFormContext>` plus the crate `KType` /
  `StandardRepr` (or the pol term lists). Structural equality reuses the
  real-form identity of the `RealForm` arm (`same_real_form`) plus strict
  crate component equality, matching `K_type_value`/`module_parameter_value`
  operator==.
- Display: the 6-way adjective chain (is_standard → is_dominant →
  is_nonzero → is_semifinal → is_normal → "final") then ` K-type` +
  print_K_type (` K_type(x=N, lambda=[..]/d]`, LEADING space) for KType,
  and the same chain + `parameter(x=N,lambda=[..]/d,nu=[..]/d]` for Param.
  The lambda/nu render through a no-inner-space rational-vector helper
  (`[1]/1`), distinct from the language RatVec display used by `%`.
  Pols use print_K_type_pol/print_SR_poly exactly: one `\n` per term,
  coefficient embellishment (`(e+fs)` only when both components occur
  across the terms), `*` + term text, ` [height]`; empty texts
  `Empty sum of K-types` / `Empty sum of standard modules`.
- Registrations follow the fixture-gated install subsets
  (atlas-types.w:6071-6088, 7472-7480, 6091-6117, 8542-8570):
  `K_type` (KGBElt,vec) + (Param), `param` (KGBElt,vec,ratvec) +
  (KType), `%` (KType)/(Param), `real_form` ×4, `height` ×2, predicates
  (5 for KType, 3 for Param), `equivalent`, `dominant`/`normal`/
  `theta_stable`/`to_canonical_fiber` (KType), `null_K_module`/
  `null_module`, `#` ×2, `+`/`-` (KTypePol,KType)/(ParamPol,Param),
  `first_term`/`last_term` ×2, and `*(int,KTypePol/ParamPol)` (skip →
  implemented, hunger 2). Constructors and `equivalent` and the pol
  add/subtract mismatch checks precede the no-value gates (validate);
  the rest run behind them (skip).
- Rank checks replicate the wrapper order and wording: `Rank mismatch:
  (r,size)` for K_type and `Rank mismatch: (r,l,n)` for param, evaluated
  BEFORE the crate call. `%` on Param returns gamma (not nu) as the third
  component. Real-form mismatch wordings and the empty-term errors match
  the upstream strings.
- Deferred by design: `+`/`-` on a NON-final KType/Param is rejected with
  a runtime "not implemented" diagnostic — `finals_for`/`expand_final`
  expansions for non-final values await the deformation layer. The other
  install-list entries (Split-scaled pol products, KTypePol/ParamPol
  binary equality, term-list forms, truncate/scale/deform families) stay
  unregistered per the slice boundaries.

Local gate: 229 atlas-core + 292 atlas-real-group tests pass; clippy and
fmt clean; the six fixtures VERBATIM via check_fixture; the wired local
pipeline reports 152 PASS + 1 PARTIAL + the known `fromfile_accepted_b10`
FAIL (HPC paths); `hpc/test_pipeline_swap_diff.py` 10/10.

## Live continuation - 2026-08-01 (L1/L2 + Rep_context milestone)

The three interrupted slices are now landed and HPC-verified; the tree is
clean at HEAD `16cb440` (main). What changed since the 2026-07-31
checkpoint:

- **L1 diagnostic wordings (agent-28) — DONE + verified.** The typed.rs
  edits that were already in the checkpoint make the four contracts
  verbatim; verified locally and by differential `3506234`:
  `commands/assignment_errors` (assignment source text appended),
  `commands/slice_errors` (`<=` no space, slice source appended, the
  dedicated `Cannot slice value of type` error), `commands/subscription_errors`
  (dedicated cannot-subscript error for a bool row index),
  `eval/container_errors` (`No common type found between components of
  list expression: { ... }`).
- **L2 bison syntax messages (agent-29) — DONE + verified.** `syntax_error`
  now emits `syntax error, unexpected X[, expecting Y]` via
  `bison_syntax_message`/`bison_expecting` (syntax.rs): token-name table
  (INT, ']', '\\n', ',', '$', $undefined, :=, ...) and an expecting suffix
  derived from the LALRPOP state's QUOTED terminal set — LALRPOP reports
  expected terminals WITH quotes (`","`, `"]"`, `"|"`), so the helper
  compares the quoted form. Lexer recovery now clears open nesting on an
  unsupported character (`(`` then `2` recovers like the oracle instead
  of swallowing the whole file — the nested_invalid_token_continues
  stdout bug). The agent-29 probe test (`panic!("probe")`) was removed.
  Five contracts verified: `parse/negative_trailing_token`,
  `commands/invalid_token_continues`, `commands/mismatched_delimiter_continues`,
  `commands/nested_invalid_token_continues`, `commands/container_syntax_errors`
  (the latter PARTIAL: the dangling `[` line whose oracle saw the
  capture-time `quit`, and the swallowed `4` line after it, are two
  PendingCases sharing reference_event 6 — see the pipeline note below).
- **agent-27 Rep_context crate milestone — DONE + tested.** The checkpoint
  files were NOT registered in `lib.rs` (so they never compiled), had a
  duplicated row sweep in `RealProjection::build` (upstream
  `matreduc::column_echelon` runs ONE sweep), and were missing three APIs.
  Now: `mod ktype`/`mod rep_context` registered + exported (`KType`,
  `RationalWeight`, `RepContext`, `StandardRepr`); duplicate sweep
  removed; `InnerClass::canonicalize_with_generators` (RankFlags gens,
  innerclass.cpp:740-832), `RepContext::root_involution_image_at`,
  `RepContext::weight_defect` added. Two in-crate tests pin the split-A1
  anchors (K_type(x,[0]) lam_rho=[0] height 0 all predicates true,
  K_type(x,[2]) collapsing mod (1-theta)X*=2X* and SR-equivalent,
  param(x,[0],[0]/1) gamma=[0]/1, K_type<->param round trip) — commit
  `f09a835`, 292 atlas-real-group tests pass.
- **Differential `3506234`** (HEAD `f09a835`, wired pipeline): 148
  fixtures, **zero FAIL**, one PARTIAL (`container_syntax_errors`, the two
  intentional pending cases). The 8 L1/L2 contracts PASS; their meta
  files now carry `rust_status: verified_hpc` +
  `differential_job: 3506234` (commit `16cb440`). The locally-FAILing
  `eval/fromfile_accepted_b10` passes on HPC (path permissions).
- **Pipeline wiring:** the nine L1/L2 contracts were added to
  `hpc/pipeline_swap_diff.py`. `validate_plan` was relaxed to accept
  pending cases that SHARE one reference event (a pending line whose
  oracle event was produced for a different source line — here the
  swallowed `4`); the runnable+pending event coverage comparison now
  dedupes with `set(...)`.
- **Remaining contracts frozen with `not_implemented`:** the six
  ktype/param-family contracts (`domain/ktype_basic{,_rejected}`,
  `ktypepol_basic`, `param_basic{,_rejected}`, `parampol_basic`), plus
  `set verbose` (`lex/basic`) and the unterminated-string recovery
  (`negative/unterminated_string`) from the L3/L4 queues. The crate math
  (RepContext/KType/StandardRepr) is now compilable, tested, and ready
  for the language layer.

## Live continuation - 2026-07-31

The current committed baseline is `HEAD` on `main` (implementation HEAD
`152f4b8`, wiring `1288e1e`). Differential job `3503356` ran 139 fixtures
with zero FAIL and verified the 21 legacy command/eval contracts
(`0898e81`): the pre-harness `command-stream`/`expression-evaluation`/
`evaluator`/`parser` contracts were regenerated verbatim from capture job
`3503334` (32 fixtures: declarations/assignments/let, containers,
subscriptions, slices, exact bignum numerics, name/type rejections, and
error recovery), the combined `eval/negative` metadata split into
`negative_type`/`negative_undefined`, and the superseded parser AST goldens
were removed. Eleven contracts remain frozen with `not_implemented`:
four diagnostic-wording slices (`assignment_errors`, `slice_errors`,
`subscription_errors`, `container_errors`), five bison syntax-message
slices (`invalid_token_continues`, `mismatched_delimiter_continues`,
`nested_invalid_token_continues`, `container_syntax_errors`,
`parse/negative_trailing_token`), `set verbose` (`lex/basic`), and the
unterminated-string recovery (`negative/unterminated_string`).
Operational note: after `git archive` overlays on HPC, files deleted in
the new HEAD must be removed explicitly or the submit tree reads dirty
(job `3503347` aborted on exactly that).

Differential job `3503322` ran 118 fixtures with zero FAIL and verified the primitive involution constructors:
`involution(LieType,[int],string)` and `involution(LieType,mat,string)`
(`152f4b8`: `checked_inner_class_letters` with the 's'/'u' collapse rules
per atlas-types.w:742, per-letter layout permutation tables per
lietype.cpp:507, and the based `on_basis` lattice transport per
matrix.cpp:289 with the integrality gate; both wrapper gate orders follow
atlas-types.w:860/:902). `PENDING_OVERLOADS` is now empty and the harness
runs 118 wired fixtures.

Differential job `3502731` ran 111 fixtures with zero FAIL and
verified the last FIVE strong-real contracts: the four `dual_order` probes
(RootDatum dual-order surface `cba10ec`: `posroots`/`poscoroots`/
`dual(RootDatum)` with flipped coroot preference and letterwise B<->C Lie
type, `dual(InnerClass)`) and the `full_kgb` probe — the KGB renumbering
sort's third key is the TwistedInvolution value compare (`WeylElt::operator<`
= parabolic-subquotient pieces by internal generator order, ported as
`ParabolicPieces`; the crate's root-permutation Ord coincided at A2 and
reversed at B2/C2). **The strong-real family is COMPLETE** (base contract
plus all thirteen probes verified). Differential `3502718` verified fourteen
contracts: the eval `split_basic{,_rejected}` pair (**the eval family is
COMPLETE**), the three weak-real probes `b2_descent` /
`central_coroot_rejected` / `validation_rejected`, and the first nine
strong-real contracts. Differential `3502969` verified the last TWO
weak-real probes (`a1_t1_central`, `a2_noncanonical`): the custom-seed
real_form path (`8135b89`) ports the elected square root cocharacter,
the involution-table extension, the full `minimal_torus_part` descent
(realredgp.cpp:212-309), and `real_form_value::build`'s default-vs-custom
branch — **the weak real form family is COMPLETE** (base pair plus all
five probes), and the C2 print_KGB probe is frozen and verified
(`3502734`/`3502736`).

Earlier verified stages this line: relations `3502506`; involution
decomposition `3502550`; base `weak_real_form{,_rejected}` `3502697`; the
torus-radical fix `646f897`; the Cartan numbering adapter `a63dc32`
(upstream BFS discovery order; B2 = [e, s1s0s1, s0s1s0, w0], orbit sizes
[1,2,2,1]; A1/A2 unchanged); the Block domain `3503231` (`4167249`:
fibred-product BlockGraph over both sides' full KGB, tW-level
dual_involution, renumbered descent status, undefined Cayleys return the
input index). The older snapshot below remains useful as a
historical ledger, but its `c0710a1` HEAD and implementation queue are no
longer current.
builtins are verified by differential job `3502506`; the involution
decomposition builtins and all 17 associated fixtures are verified by job
`3502550` (90/90 runnable fixtures PASS; suite PARTIAL only for the three
explicitly pending overloads). The base `weak_real_form{,_rejected}` contract
pair is verified by differential job `3502697` (92 fixtures, zero FAIL; the
three-argument `real_form(InnerClass,mat,ratvec)` classification path:
complex-cross DFS to the class representative, grading bits from
simple-imaginary pairings, gradingRep/adjoint-orbit lookup). The torus-radical
`inner_class` gap is fixed (`646f897`: `StrongRealClassification::build` now
sizes the toWeakReal representative from the ambient fiber lattice rank, not
the adjoint datum rank), so `central_coroot_rejected` compares VERBATIM.
Thirteen strong-real probes (B2/C2 Cartan enumerations in root/coroot
preference, dual-order invariance, full B2 KGB prints, four rejected
diagnostics) are frozen with reference metadata from capture job `3502700`
(`230a8d5`). The CARTAN NUMBERING ADAPTER has landed (`a63dc32`:
`CartanClassification::build` enumerates classes in the upstream BFS
discovery order — parents in discovery order, positive imaginary roots in
(height, revlex) RootNbr order, Cayley successors canonicalized before
dedup; B2 order is now [e, s1s0s1, s0s1s0, w0] with orbit sizes [1,2,2,1];
A1/A2 unchanged). With it the four B2/C2 Cartan enumeration probes, the
four rejected strong-real probes, the base `strong_real` contract, and the
`b2_descent`/`central_coroot_rejected`/`validation_rejected` weak-real
probes all compare VERBATIM locally — none of these is wired into the
pipeline yet, so no HPC differential covers them so far. Two follow-up
slices are identified and queued: the KGB element discovery order still
diverges (`strong_real_b2_full_kgb_probe`: Cayley link targets; upstream
kgb.cpp:489 extends each element by all cross actions in simple-root order
before Cayley transforms), and the RootDatum dual-order surface is missing
four builtins (`posroots`, `poscoroots`, `dual(RootDatum)`,
`dual(InnerClass)` — the four `dual_order` probes). The older snapshot
below remains useful as a historical ledger, but its `c0710a1` HEAD and
implementation queue are no longer current.

Still open on the weak real form surface (five oracle probes from jobs
`3502476`/`3502479`): `b2_descent`, `validation_rejected`, and
`central_coroot_rejected` compare VERBATIM locally but are not yet wired into
the pipeline; `a1_t1_central` matches the oracle through `form_number` and
first diverges at `base_grading_vector` (want `[ 0, 1 ]/2`, got `[ 0, 0 ]/1`);
`a2_noncanonical` classifies correctly but diverges on the seed-derived
outputs. Both remaining probes need the custom-seed real_form gap: upstream
builds a non-default `real_form_value` seed via `minimal_torus_part`
(realredgp.cpp:212-309; the `global_tits.rs` rational torus carrier, inverse
Cayley, and `InnerClass::canonicalize` groundwork for this route are
committed). Upgrade the base-pair claim to the full slice only when all five
probes pass an HPC differential at one clean commit.

Important correction to the older queue text: upstream
`realredgp::minimal_torus_part` does **not** call `central_fiber`. It transports
the supplied Tits element downward to the fundamental fiber using inverse
Cayley or based twisted conjugation, reduces there, walks the fundamental
imaginary grading orbit, filters by the target weak-form compact grading, and
selects the numerically least torus part. `central_fiber` is part of the
separate elected `x0_torus_part` construction.

## Live continuation - 2026-08-06 (overnight builtin sweep)

HEAD: `401a78a` (main). HPC differential `3520179` (fat, TIMEOUT=1800,
`cargo build --offline` after syncing the local cargo cache/index to the
HPC node; earlier submissions 3519983/3519989/3519995/3520003/3520154/
3520168 failed on crates.io access, fixed by pinning crossbeam-deque
0.8.6 + offline + full cache/index sync) re-verifies the whole fixture
set after ~100 more builtins were live-ized; all local gates green (230
atlas-core + 316 atlas-real-group tests, clippy 0 warnings, fmt clean).

Builtins landed this sweep (each VERBATIM against the local oracle on
A2/B2/G2/A3/A1A1 probes):

- `cofolded` (InnerClass->RootDatum): fold_orbits + cofold via
  `RootInvolutionData::image_permutation`; B2 identity, A2/G2/A3 split
  (A1.T1), and the orthogonal A1A1 two-type pair byte-identical.
- KType predicates: height/is_standard/is_dominant/is_zero/is_final/
  is_semifinal/dominant/to_canonical_fiber (live registrations; the
  dominant/normal/theta_stable/to_canonical_fiber transform arm already
  existed). Param predicates: same six on StandardRepr.
- dual_datum (InnerClass->RootDatum), quasisplit_form / dual_quasisplit_form
  (InnerClass->RealForm), dual overloads (RootDatum rd->dual via the now-pub
  `dual::dual_datum`, InnerClass G->dual, Block->Block), form_names /
  dual_form_names, form_number, distinguished_involution, root_datum
  InnerClass coercion, central_fiber (strong_real::central_fiber), KGB_size.
- cross (int, Param): repr.cpp:891-910 port (made_dominant + gamma_lambda
  - pos_neg real-root correction + simple reflection + sr_gamma).
- Cayley (int, Param): repr.cpp:943-1002 port (ImaginaryNoncompact raise
  with parity/rho_r corrections, real inverse-Cayley with parity gate;
  Cayley_error passes the input parameter back unchanged).
- length (Param): Rep_table::length via the partial-block representative
  height. Live registrations for rank (RootDatum/LieType), length
  (KGBElt), orientation_nr (Param) whose arms already existed.

Also: `RationalWeight::add/sub` made pub (lattice.rs) for the cross/Cayley
ports; `dual::dual_datum` made pub + exported.

Remaining (all recorded in docs/REMAINING_BUILTINS.md, mostly gated on the
common-block srm pool / global KGB / ext_block layers): extended_block,
finalize_extended, partial_extended_KL_block, dual_KL_block,
K_type_pol_extended, scale_extended, raw_ext_KL, shift_flip, block_deform,
twisted_deform, twisted_full_deform, KL_block, twisted_KL_sum_at_s,
print_X/print_gradings/print_real_Weyl/print_blockstabilizer/
print_common_block, Weyl_orbit family, alcove_center/alcove_root_vertex,
walls/walls_attitude, FPP_numers/FPP_w_shifts, root_expression/root_index/
root_permutation (oracle root numbering), root_ladder_bottoms/
coroot_ladder_bottoms.

## Start here (next agent)

HEAD at handoff: `34f05e7` (main). Working tree clean.

### Since the 8d9837d handoff (2026-08-02 overnight + user ktype/param layer)

The user completed the ktype/param language layer (KTypeValue /
ParamValue / KTypePolValue / ParamPolValue, Display, typed.rs
registration, on-demand RepContext evaluation) and the
simple_roots/simple_coroots/is_Cartan_matrix builtins; all 13 ktype/param
fixtures and domain/simple_roots are VERBATIM + HPC-verified.

The overnight sprint delivered eight more builtins, all VERBATIM and
HPC-verified:

- `39c46cb` Cartan_info (classify triple, Weyl word, orbit/fiber sizes
  with a real fiber_rank, make_simple_complex subsystem types) —
  `domain/cartan_info`, HPC `3507853`.
- `17dc5a0` orientation_nr (repr.cpp:455-493) — `domain/orientation_nr`,
  HPC `3507866`.
- `693dd96` block_Hasse (param list + Bruhat Hasse matrix; the full
  block is the param's form paired with the dual's **quasisplit** form)
  — `domain/block_hasse`, HPC `3507974`.
- `c1958c8` W_graph/W_cells over a Param (descent sets + bidirectional
  mu edges, strong-component cells) — `domain/w_graph_param`,
  HPC `3507974` (extended to B2, HPC `3508032`).
- `0df2942` raw_KL/dual_KL (KL index matrix, polynomial pool, length
  stops) — `domain/raw_kl`, HPC `3507974` (extended to B2 12-element
  and G2, HPC `3508004`).
- `f199803` KL_sum_at_s/_to_height (KL column at q=s by Horner) —
  `domain/kl_sum_at_s`, HPC `3507981` (extended to B2, HPC `3508004`).
- `719ed41` two_rho/two_rho_check — `domain/two_rho`, HPC `3507991`.

After the 01df48e handoff the overnight sprint continued:

- `fa8f325` KL_column — the KL column of a final standard parameter over
  its partial block (Bruhat_generator::block_below with complex and
  **parity** real type-I descents; Rep_context::is_parity ported) —
  fixture `domain/kl_column`, HPC `3508248` (181 fixtures, 0 FAIL).
- `3daca78` partial_KL_block — the condensed KL matrix over the
  partial-block survivors with Block_base::finals_for (blocks.cpp:
  335-368) and a zero-first polynomial store — fixture
  `domain/partial_kl_block`, HPC `3508277` (182 fixtures, 0 FAIL).
  First Batch 6 (extended blocks) name.
- `4bfc4a5` kgb_hasse extended to B2/A3 (HPC `3508458`, 182 fixtures 0 FAIL).
- `f77f73a` — simple_roots prints the **transposed** Cartan matrix (the
  oracle's rows are simple coroot coordinates; B2/G2/F4/D4/E6 all match,
  HPC `3508482`). is_Cartan_matrix handles F4/E6/C4.
- Fixture extensions across kgb_hasse/cartan_info/orientation_nr/
  simple_roots/two_rho/kl_print (B2/G2/F4/D4/A3/B3/C3) all HPC-verified
  (swaps `3508458`, `3508475`, `3508482`, `3508486`, `3508490`).
- **Batch 7 first name: full_deform** (`7a5c2a3`) — the full K-type
  deformation (atlas-types.w:8213-8227) via the freshly ported
  Rep_context::finals_for (repr.cpp:1205-1297, `0108799`) and
  Rep_context::reducibility_points (repr.cpp:825-925, `ebe40de`), on top
  of the existing scale/deform_readjust/deformation_terms. A1/A2/B2/G2/A3
  byte-identical; fixture `domain/full_deform`, HPC `3511044` (183
  fixtures, 0 FAIL).
- **Batch 7 second name: KL_block** (`32398d5`) — the condensed KL
  matrix over the parameter's common block (fibred closure with
  parity-filtered real type-I descents), singular-coroot survives
  (repr.cpp:526-534: coroot·gamma numerator == 0), finals_for
  condensation. A2 x=0 and A1 x=2 byte-identical; HPC `3511377`
  (184 fixtures, 0 FAIL).
- **Batch 6 third name: partial_block** (`domain/partial_block`) — the
  partial-block parameter list (KL descent closure + singular
  survivors); HPC `3511402` (185 fixtures, 0 FAIL). partial_KL_block
  was recaptured after dropping its A2 x=3 case (HPC `3511377`).
- Fixture extensions all HPC-verified: raw_kl/w_graph_param/kl_sum_at_s
  B3/C3 + kgb_hasse C3/D4 (swaps `3511421`/`3511424`/`3511428`),
  simple_roots/two_rho E6/E7/E8 (swap `3511489`), kl_print B3/C3
  (recaptured `3511504`, swap `3511505`).
- **More rank-4/exceptional coverage**: kl_print(G2),
  partial_block(F4), partial_kl_block(F4) — all byte-identical locally,
  captures submitted (3513227/3513240/3513252).
- **E6 column-echelon deep-dive** (5h, unresolved): proved that the
  incremental port is not equivalent to C++'s one-shot `column_apply`,
  that E6 involution 187 needs `ops(mindex,mindex)=-1` recorded, and
  that `col` inversion needs Euclidean row reduction. Left blocked on
  an A2-vs-E6 contradiction (same C++ code, different sign behavior;
  full notes in REMAINING_BUILTINS.md).
- **Batch 1 verification**: is_Cartan_matrix and dual_datum fixtures
  added (byte-identical locally). **Known limit recorded**: E6's
  `RealProjection::build` column-echelon port fails for involution 187
  (packet 74) — the E6 class-1 real form's KL/deform surface is
  unavailable until the echelon port is fixed (1-2h task, recorded in
  REMAINING_BUILTINS.md).
- **kl_sum_at_s now covers B4/C4/F4/D4** (all byte-identical) —
  the KL-sum surface is swept across every split form of ranks 1-4.
- **The rank-4 classical series now verified**: W_cells(C4/B4),
  raw_KL(C4/B4/D4), kl_column(D4), partial_kl_block(D4),
  kl_print(F4). The KL/print/deform surface now covers
  A1..A4/B2..B4/C3..C4/G2/F4/D4 — every series' split forms.
- **G2 and F4 now swept across the whole KL/deform surface** —
  raw_kl(A1/G2), kl_column(G2), partial_block(G2), deform(G2),
  full_deform(F4). The KL family (raw_kl, kl_column, kl_sum_at_s,
  w_graph_param, partial_kl_block, partial_block) and the deform pair
  now cover A1/A2/B2/G2/A3/B4/F4/D4 — the non-simply-laced and
  exceptional ranks are all byte-identical.
- **More coverage**: W_cells/W_graph/raw_KL/kl_sum_at_s extended to
  F4 (all byte-identical); W_cells(G2), kl_sum_at_s(G2), W_cells(A3),
  raw_KL(B4), default_extended twist-validity checks (test_compatible,
  `91b3762`). The A4 invalid-twist rejection is implemented but not
  frozen (the local capture has no stderr diagnostics).
- **More coverage**: W_cells(A3), raw_KL(B4), default_extended
  twist-validity checks (test_compatible, `91b3762`). The A4
  invalid-twist rejection is implemented but not frozen (the local
  capture has no stderr diagnostics).
- **Fixture coverage swept through A3 and E7/E8** — the KL family
  (raw_kl, kl_column, kl_sum_at_s, w_graph_param, partial_kl_block,
  full_deform, deform) all extended to A3; simple_roots/two_rho to
  E7/E8; cartan_info/orientation_nr to A3. All byte-identical locally;
  captures batched on HPC (3512429-3512455). The E7 KGB_Hasse swap
  runs on the fat partition (2TB, job 3512428) — the earlier OOM was
  the cpu partition's 8G per-task cap, not a code issue.
- **default_extended is now COMPLETE** (`fab1593` + `6855ca2`) — the
  generic twist is solved by matreduc::find_solution (an exact rational
  Gaussian elimination port in the workspace); A2 identity + A3
  non-identity byte-identical, HPC-verified (swap `3512392`, 0 FAIL).
  This unlocks the ext_block layer's parameter model.
- **extend(LieType) lands** (`9b0abbb`) — append a simple factor
  (add_simple_factor, atlas-types.w:280-289); A2+G2+D4 byte-identical,
  HPC-verified. **E7 KGB_Hasse was tried and dropped** (ec40b29): the
  2.9M-element Weyl-group enumeration OOMs on the HPC node; the E6
  fixture stays verified. The WEYL_BUDGET was raised to 4M for E7-scale
  inner classes when memory allows, and the HPC swap timeout is now
  driven by the TIMEOUT env (600s used for E7-scale).
- **default_extended lands** (`fab1593`, HPC `3511998`) — the first
  Batch 6 name. The 4-tuple (lambda, tau, l, t) via the srm
  gamma-lambda unique mod X* (StandardReprMod::mod_reduce with the new
  real_unique, `7fcbc49`) and ell = base_grading_vector -
  torus_factor (ext_block.cpp:215). A2 x=1/2/3 + B2 x=0 byte-identical
  for the identity twist; the generic twist needs matreduc::find_solution
  (recorded). The E6 KGB_Hasse fixture is now HPC-verified (`3511986`),
  so the local-timeout constraint is lifted by the HPC node.
- **Rep_context::real_unique lands** (`7fcbc49`) — the unique
  mod-X* representative (involutions.cpp:334-342). With it the srm
  common-block experiment makes A2 x=3's block_Hasse byte-identical,
  but the full common block still needs the srm chain's per-element
  lambda-rho (the pool elements differ from the fibred elements), so
  block_Hasse stays on the fibred closure; real_unique stays for the
  ext_block layer (default_extended's mod_reduce). involution_of is
  now public.- More fixture extensions verified: cartan_info +C3 (`3511528`),
  orientation_nr +C3 (`3511528`), kl_column +B3/C3 (`3511532`),
  full_deform +B3/C3 (`3511570`), simple_roots +E6/C3/B3 (`3511747`),
  two_rho +B3/C3/F4 (`3511750`), cartan_info +G2/F4 (`3511753`),
  w_graph_param +G2 (`3511855`), kl_print +D4 (`3511862`),
  kl_sum_at_s +D4 (`3511873`) — all 185 fixtures, 0 real FAIL.
  root_ladder_bottoms needs the root_perm/link tables (recorded as a
  known limit, rootdata.cpp:243-313).
- The gamma-lambda-mod-cocharacter-lattice common-block matching
  (`523e647`) was **reverted** (`97770c0`): it over-restricted the
  fibred closure (C3 x=0 has 9 elements; the filter kept 4) because the
  srm matching needs the z_pool gamma-lambda layer. Rep_context::
  gamma_lambda and torus_part stay for that layer. Known limits: A2 x=3
  and C3 x=0 common-block element sets; B2 block_Hasse element 11's
  lambda (srm pool gamma-lambda).
- The common-block experiment (block_Hasse over the srm closure) was
  reverted: the fibred-transform closure over-expands (A2 x=3 → 5
  elements vs the oracle's 1); matching needs the StandardReprMod
  gamma-lambda layer. block_Hasse still uses the whole fibred block.

Plus the earlier fixes:
- `fbed749` — **the A3 grading fix**: verified_generator_map demanded
  exactly one simple-imaginary position per adjoint-fiber bit, but the
  oracle's shifts are coroot·root parities (realredgp.cpp:277-280) and
  the A3 dual's single bit flips two. Taking the first flipped position
  unlocks every classical-rank>=3 dual real form: A3/B3/C3/D4/F4
  raw_KL, deform, W_graph/W_cells, KL_sum_at_s and the KL printers are
  all byte-identical to the oracle (fixtures extended, HPC swaps
  `3508109`, `3508132`, `3508138` — 0 FAIL). raw_kl covers
  A2/B2/G2/A3/D4; w_graph_param/kl_sum_at_s/kl_print cover A3.
- `dfd62ef` — print_W_cells (and W_cells) list each cell's vertices
  ascending (the oracle's Partition traversal).
- `f7bda08` — print_KL_list sorts by coefficient count then descending
  coefficients (polynomials::compare).
- `fbed749` also guards the KL printers against 0-element blocks.

And the earlier important fixes:

- `fbed749` — **the A3 grading fix**: verified_generator_map demanded
  exactly one simple-imaginary position per adjoint-fiber bit, but the
  oracle's shifts are coroot·root parities (realredgp.cpp:277-280) and
  the A3 dual's single bit flips two. Taking the first flipped position
  unlocks every classical-rank>=3 dual real form: A3/B3/C3/D4/F4
  raw_KL, deform, W_graph/W_cells, KL_sum_at_s and the KL printers are
  all byte-identical to the oracle (fixtures extended, HPC swaps
  `3508109`, `3508132`, `3508138` — 0 FAIL). raw_kl covers
  A2/B2/G2/A3/D4; w_graph_param/kl_sum_at_s/kl_print cover A3.
- `dfd62ef` — print_W_cells (and W_cells) list each cell's vertices
  ascending (the oracle's Partition traversal).
- `f7bda08` — print_KL_list sorts by coefficient count then descending
  coefficients (polynomials::compare).
- `fbed749` also guards the KL printers against 0-element blocks.

Three earlier important fixes:

- `24ba188` — **the KL-table Cayley/inverse-Cayley/cross argument order**
  (the accessors take (element, generator) but the KL code called them
  (s, x)); missing images outside the block now contribute the zero
  polynomial. This unlocked B2/G2 KL columns, raw_KL 12-element blocks,
  KL_sum_at_s B2, deform B2 and print_KL_basis B2 — all byte-identical
  to the oracle (fixtures extended, HPC `3508004`).
- `562f7e7` — deform pairs with the dual's **quasisplit** form (was
  form 0; wrong for B2).
- `ee73c17` — endgame mu-pairs require a nonzero polynomial
  (KlPol::degree() saturates to 0 for zero), fixing B2 W_graph/W_cells.

Known limits: the oracle's `lookup_full_block` is the parameter's own
common_block (a proper sub-block for e.g. the A1 x=2 / A2 x=3 principal
series) — the Rust block is the fibred product, so those parameters
differ; KL_column needs the partial-block `lookup`; KL_sum_at_s uses
the input parameter's lambda-rho for every block element (height-parity
mismatch for mid-block parameters); A3+ `dual_real_form` fails with
"real-form order single-bit grading shift" (a multi-bit grading shift in
CartanGradingData); the Weyl word is the greedy reduced word (not the
WeylGroup transducer); print_gradings / root_ladders / root_index need
the oracle root numbering; print_X needs the global KGB; print_real_Weyl
/ print_blockstabilizer need realweyl; the extended-block family and
shift_flip / twisted_KL_sum_at_s need the ext_block layer.

The language gate
is complete: **166 of 166 frozen fixtures carry `verified_hpc`** — the
last contract, `domain/deform`, passed the HPC differential `3506798`
(165 PASS + the one known PARTIAL `container_syntax_errors`, zero FAIL)
and its meta was upgraded at the deform-verify commit.

The domain layer is complete (86 of 86 frozen domain contracts). Every
frozen contract from the 2026-07-31 checkpoint plus the deform-family
slices (KGP_sum, K_type_formula, branch, ParamPol/Param operations,
Param predicates/transforms, deform), L3/L4 (set verbose + string
recovery), and the ktype/param language surface are landed.

Since the gate closed, the remaining-builtin port has made three
HPC-verified batches (48 remaining names → 44):

- `4857d2a` Batch 1: `simple_roots`, `simple_coroots`, `is_Cartan_matrix`,
  `dual_datum(InnerClass)` — fixture `domain/simple_roots`.
- `0894ccf` Batch 2: `print_KGB_order`, `print_KGB_graph` —
  `KgbGraph::bruhat_hasse` (kgb.cpp:848-893) + `n_bruhat_comparable`
  (poset.cpp:197-229) — fixture `domain/kgb_bruhat`.
- `843e24a` Batch 3 (partial): `root_coradical`, `coroot_radical` —
  `BasedRootDatum::coradical_basis/radical_basis` via
  `integer_lattice::saturated_kernel` — fixture `domain/radical`.
- `076a01b`/`8d9837d`: HPC reference capture (job 3506835) and the
  swap-diff differential (job 3506839: 168 fixtures, runnable PASS,
  0 FAIL, 2 known pending). All three new metas are `verified_hpc`.

The 44 remaining names are tracked in `docs/REMAINING_BUILTINS.md`
(batches 3 remainder → 8): ladder bottoms (need the full root-system
permutations, rootdata.cpp:243-313, not stored by the atlas-core
RootTable), the block/KL/print family, W-cells, extended blocks, the
twisted deform variants, and Cartan_info (whose first triple is
`classify_involution`, already ported). Each batch follows the per-slice
loop below: probe the local oracle at `/Users/hoxide/mycodes/atlasofliegroups/atlas`,
freeze a fixture, local gate, HPC reference capture, swap diff, meta
upgrade.

The remaining porting work is no longer gated on language contracts:
the twisted deform variants (`twisted_deform`, `block_deform`,
`full_deform`, KL sums, `KL_block` — the same KL table, extended with
the twisted variant), the Param `cross`/`Cayley` transforms (need the
integral SubSystem), the KL/file formats (filekl adapter), and readline
completion.

## The per-slice loop (follow exactly)

1. Pick the next contract from the queue below. Contracts are already
   frozen (events.json status `verified_hpc_reference`); do NOT redesign
   them unless an implementation proves the probe wrong — in that case
   re-probe the oracle, never guess.
2. Implement in the smallest owning module. Domain builtins register in
   `crates/atlas-core/src/typed.rs` `builtin_registry()` (pattern: the six
   `root_coroot` entries after `Cartan_matrix(RootDatum)`, commit
   `af6cd7b`) and evaluate in `crates/atlas-core/src/domain_builtins.rs`;
   crate-level math lives in `crates/atlas-real-group/` (safe Rust only).
   Add `FixturePlan(name="domain/<n>")` (and `_rejected`) to
   `hpc/pipeline_swap_diff.py`.
3. Local bounded checks, all must pass:
   `cargo test -p atlas-core --lib`, `cargo test -p atlas-real-group --lib`,
   `cargo clippy -p atlas-core -p atlas-real-group --lib --tests -- -D warnings`,
   `cargo fmt --all -- --check`, `cargo build -p atlas-cli`
   (use `export PATH="$HOME/.cargo/bin:$PATH"`).
4. Verbatim fixture check in a /tmp cwd: run
   `./target/debug/atlas-cli tests/fixtures/domain/<n>.atlas`, compare
   stdout/stderr/exit against events.json via
   `hpc/pipeline_swap_diff.py:expected_cli_observation`.
5. Full local regression (FAIL allowed only for
   `fromfile_accepted_b10`, which needs HPC paths):
   `cd /tmp && rm -rf R && mkdir -p R/workspace && cp -R <repo>/tests R/workspace/ && cd R && python3 <repo>/hpc/pipeline_swap_diff.py <repo>/target/debug/atlas-cli out --workspace-root workspace --fixture-root <repo>/tests/fixtures --reference-root <repo>/tests/reference --commit local --dirty-tree true --detected-commit local --detected-dirty-tree true --job-id local --source-snapshot-sha256 local`
   Delete `hpc/__pycache__` afterwards (it is gitignored but keep the tree
   tidy); delete any stray file `x` at repo root if a runner creates it.
   Also run `python3 hpc/test_pipeline_swap_diff.py` (10 tests).
6. Commit (conventional commits, no push without asking).
7. Sync and submit the HPC differential:
   `rsync -az --delete .git/ majj@10.26.14.64:/public/home/majj/atlas-rust/.git/ && git archive HEAD | ssh majj@10.26.14.64 'cd /public/home/majj/atlas-rust && tar -xf -'`
   then `ssh majj@10.26.14.64 "cd /public/home/majj/atlas-rust && ATLAS_COMMIT=$(git rev-parse HEAD) ATLAS_DIRTY_TREE=false sbatch hpc/pipeline_swap_diff.sbatch"`.
   This sync pattern is robust against dirty working trees (concurrent
   subagents); the remote checkout must equal HEAD exactly.
8. When the job finishes: fetch the report, confirm the target fixtures
   PASS and no regressions (suite PARTIAL is normal while pending
   overloads remain), then upgrade the fixture metas to
   `rust_status: verified_hpc` + `differential_job` and commit.

## Implementation queue (all contracts frozen, in suggested order)

Domain (contracts in `tests/fixtures/domain/`, events verified):
`kgb_operations` + `tits_operations` (agent-10, see above) → `grading`
(base_grading_vector/initial_torus_bits/torus_bits — upstream semantics
pinned: base_grading_vector(rf) = `rf->val.g_rho_check()` (atlas-types.w:3689,
the rational coweight whose simple-root pairings are the base grading, e.g.
compact SU(2) = [1]/2); initial_torus_bits(rf) = `rf->val.x0_torus_part()`
(atlas-types.w:3695, distinguished-seed torus bits as int_Vector);
torus_bits(x) = the element's torus-part bit vec, parallel to the existing
`torus_factor` adapter at domain_builtins.rs:1988; crate hooks in
crates/atlas-real-group/src/grading.rs and real_form_labels.rs) → `weyl_element`
(W_elt/word/length/=,!=/*//#/root_datum — upstream semantics pinned:
W_elt(rd,w) = check_Weyl_word(w, semisimple_rank) + W().element(w)
(atlas-types.w:2361; errors 'Illegal Weyl word entry N (should be <R)' and
'Negative integer where unsigned is required'); word(w) = W.word(w)
(atlas-types.w:2374) is the CANONICAL reduced word from the Weyl-group
Transducer (structure/weyl.{h,cpp}) — display `<0.1.0>` must match the
oracle's transducer choice exactly (B2 input [0,1,0,1] canonicalizes to
<1.0.1.0>, A2 [0,1,0] stays <0.1.0>); NOTE the crate-level weyl_element
dropped the transducer order (WEYL_ELEMENT_DESIGN.md deferral), so the
language layer must port the Transducer word canonicalization, not reuse
the crate's raw word; length(w) = W.length(w)) → `weak_real_form` (real_form(InnerClass,mat,ratvec) —
atlas-types.w:3851: size check 'Torus factor size mismatch';
twisted_from_involution(theta) ('Given transformation is not an involution');
doubled projection num += theta.right_prod(num), is_central parity test on
the DOUBLED factor (fail: 'Torus factor does not define a valid strong
involution' — NOT exercised by the frozen contract, only the first two
diagnostics are contract-gated), then halve; real_form_of(G,tw,factor,coch)
classifies the weak form and sets the cocharacter; minimal_torus_part chooses
the base TorusPart; the intervening chunk ensures the Cartan involution table
covers tw's class downward before minimal_torus_part; anchors: (ic,[[1]],0)
-> split form 1, (ic,[[-1]],0) -> split form 1 (form_number is already
registered, typed.rs:4142), (ic,[[1]],[1]/2) -> compact form 0 — i.e. zero
factor selects the QUASISPLIT form and the rho_check shift the compact one);
CRATE RECON 2026-07-31: twisted_from_involution + seed_torus_part landed
with seed_x0; CartanGradingData grading classification
(grading/element_from_grading, grading.rs:201/216) exists — the new work is
(a) the (tw,factor)->grading->weak-class assembly of real_form_of
(innerclass.cpp) and (b) the distinct `minimal_torus_part` descent/orbit
algorithm from realredgp.cpp:212-309. It uses inverse Cayley and based twisted
conjugation to reach the fundamental fiber, then minimizes within the grading
orbit; it does not use `central_fiber`. MEDIUM slice) →
`involution_decomposition` (distinguished_involution(ic) =
G.distinguished() as mat; twisted_involution(rd,M) =
inner_class_value::build(rd,M,&ww) then the PAIR (W_elt(rd,W.element(ww)),
ic) (atlas-types.w:3200) — ww is the conjugation word bringing M to
distinguished form, and the W_elt display reuses the weyl_element Transducer
canonicalization (anchors: A2 opposition -> (<0.1.0>, compact ic), identity
-> (<>, same ic)); classify_involution(M) (atlas-types.w:2697): non-square
-> 'Involution should be a {r}x{r} matrix; received a {a}x{b} matrix',
M^2!=I -> 'Given transformation is not an involution' (the contract-gated
diagnostic), then tori::classify (tori.cpp:189) = NO eigenspace work:
tau1=M+I; plus_rank = integer column-echelon rank of tau1; complex_rank =
mod-2 image rank of tau1; result (plus-complex, complex, r-plus-complex) —
anchors: I2 -> (2,0,0), A2 opposition -> (0,1,0); CRATE RECON 2026-07-30:
seed_x0 already landed InnerClass::twisted_from_involution with the
conjugation word exported via wrt_distinguished_word — twisted_involution
is a thin pair-assembly over it, classify_involution needs only integer
echelon + mod-2 rank (integer_lattice.rs/mod_two.rs exist); LIGHT slice) → `strong_real` (square_classes + B2
print_strong_real — square_classes(cc) (atlas-types.w:4230): per square
class csc, pi=fiber_partition(csc), row of rfi.out(rfl[toWeakReal(c,csc)])
per partition class c — NOTE rfi.out can COLLAPSE distinct internal forms to
one external number (B2 c0 anchor: [[2],[1,0,0]] has duplicate external 0);
square_classes is already registered+verified by cartan_aggregation, so this
slice is COVERAGE-ONLY if involution_table landed the full print_strong_real:
B2 c2 exercises the multi-class layout ('there are 2 real form classes:\n\n'
header, blank line after EVERY block including the last; squares
exp(2i\pi([0,1]/2)) and exp(2i\pi([0,0]/1))). NUMBERING ADAPTER — LANDED
2026-07-31: `CartanClassification::build` now enumerates classes in the
upstream BFS discovery order (innerclass.cpp:218-291 task 1; parents in
discovery order, positive imaginary roots in (height, revlex) RootNbr
order, Cayley successors canonicalized via `InnerClass::canonicalize`
before dedup). B2 order is now [e, s1s0s1, s0s1s0, w0] with orbit sizes
[1,2,2,1], verified against the oracle's Cartan_info and the frozen
B2/C2 Cartan probes; A1/A2 order unchanged. The KGB element discovery
order still diverges (full_kgb probe: Cayley link targets) and is a
separate queued slice → `split_basic` (eval/; Split operator family —
language-level primitive type `Split` (no crate math; s^2=1 pair arithmetic
(e1e2+f1f2, e1f2+f1e2)); upstream install list atlas-types.w:5136-5145 is
NINE entries: =(Split,Split->bool), !=(Split,Split->bool), unary =(Split->bool)
and !=(Split->bool) zero tests, +(Split,Split->Split), -(Split,Split->Split),
unary -(Split->Split), *(Split,Split->Split), %(Split->int,int) returning a
TUPLE (e,f); coercions int->Split ((a,0)) and (int,int)->Split; display is
'(' e ('+'|'-') |f| 's)' with sign folded (anchors: (3+2s), (5+0s), (-3-2s),
(-2+2s)); type name in declarations is `Split`; no division overload —
s/2 gives 'Failed to match '/' with argument type (Split,int)') →
`block_basic` (install list atlas-types.w:4994-5004
is TEN entries: block(RealForm,RealForm->Block) gated by
is_dual(rf.ic,df.ic) else 'Inner class mismatch between real form and dual
real form'; %(Block->RealForm,RealForm) = (rf,dual_rf); #(Block->int);
element(Block,int->KGBElt,KGBElt) bounds 'Block element {i} out of range
(<{size})' — the y component is rebuilt in rf.ic_ptr->dual() via
real_form_value::build(dic, dual_rf.realForm()); index(Block,KGBElt,KGBElt
->int); dual(Block->Block); status(int,Block,int->int) bounds 'Illegal
simple reflection: {s}' then element bounds, output renumbered
tab={4,5,6,7,1,0,3,2} from DescentStatus::Value order
{ComplexAscent,RealNonparity,ImaginaryTypeI,ImaginaryTypeII,
ImaginaryCompact,ComplexDescent,RealTypeII,RealTypeI} (descents.h:40) to
0=C-,1=ic,2=r1,3=r2,4=C+,5=rn,6=i1,7=i2 (anchors: status(0,B,0)=6,
status(0,B,2)=2); cross always defined; Cayley = cayley(s,i).first with
UndefBlock -> return INPUT i as undefined indicator (same for
inverse_Cayley, anchor inverse_Cayley(0,B,0)=0); needs crate Block::build
(blocks.cpp:610/622) — the heaviest piece in this queue; display
'Block of N elements'; dual_real_form(InnerClass,int) already registered
(typed.rs:4125). BLOCK CONSTRUCTION MAP (recon 2026-07-30):
Block::build = KGB(rf, common_Cartans(G_R,dG_R)) + dual KGB likewise
(blocks.cpp:610) then Block(kgb,dual_kgb) (blocks.cpp:527): per twisted
involution w, dual_w = dual_involution(w,tW,dual_tW) — the tW-LEVEL dual
map, NEW (cartan_aggregation's dual_cartan_correspondence is the
class-level analogue) — and elements = fibred product x in tauPacket(w)
times y in tauPacket(dual_w); descents(x,y,kgb,dual_kgb) per simple root;
cross(s,z) = element(kgb.cross, dual_kgb.cross); Cayley TypeI/II pairs
kgb.cayley with dual_kgb.inverseCayley .first/.second; element(x,y) via
first_z_of_x binary search. Fixture needs NONE of compute_supports/Bruhat.
Crate reuse: tauPacket/involution table/cross/cayley exist per form;
new = common-Cartans restricted KGB, tW dual map, fibred assembly,
block-level descent status) →
`ktype_basic` (KType install list atlas-types.w:6071-6088
is 16 entries: K_type(KGBElt,vec->KType) = Rep_context::sr_K normalizing
lambda-rho mod (1-theta_x)X*, rank check 'Rank mismatch: ({rank},{size})'
(atlas-types.w:5240); %(KType->KGBElt,vec) elected representative;
real_form(KType->RealForm); height(KType->int); =/!=(KType,KType) on
normalized forms (anchor: K_type(x,[0]) = K_type(x,[2]) for split A1 x=2
since (1-theta)X*=2X*); equivalent (SR-equivalence); is_standard
((1+theta)lambda imaginary-dominant)/is_dominant/is_zero (singular compact
simply-imaginary exists)/is_semifinal (no real parity roots)/is_final;
dominant/to_canonical_fiber/normal/theta_stable (KType->KType); display =
adjective chain non-standard/non-dominant/zero/non-final/non-normal/final +
' K-type' + print_K_type 'K_type(x=N, lambda=[..]/d)' (basic_io;
atlas-types.w:5210+5224); needs crate Rep_context/K_repr machinery
(repr.{h,cpp}, K_repr.h) — sr_K normalization and the predicate set are the
math core of this slice. REP RECON 2026-07-30: the gated Rep_context subset
is focused despite repr.cpp's 2839 lines (most is blocks/KL/branch/deform):
sr(x,lam,nu)=sr_gamma(x,lam,gamma(x,lam,nu)) (repr.h:242); sr_gamma
(repr.cpp:756) = StandardRepr(x, y_pack(i_x,lam_rho), gamma,
height((1+theta)gamma)); sr_K(x,lam_rho) with the mod-(1-theta)X*
normalization inside K_type's constructor (K_repr.cpp, 626 lines total);
~8 predicates are compact root-table computations; supporting pieces
mostly EXIST: InvolutionTable (involution_table.rs), Tits coset reduce
(seed_x0's quotient_representative ~ y_pack), kgb status, g_rho_check —
plan ktype_basic+param_basic as ONE crate milestone (Rep_context subset)
with two language slices; ktypepol/parampol are then thin) →
`ktypepol_basic` (KTypePol install list atlas-types.w:6091-6117:
null_K_module(RealForm->KTypePol) display 'Empty sum of K-types';
real_form; unary =/!= zero tests; =/!=(KTypePol,KTypePol); # = TERM count
(not coefficient sum; anchor: #R=1 for 2*K); +(KTypePol,KType) /
-(KTypePol,KType) merging like terms (anchor: Q+K doubles coefficient);
+(KTypePol,(Split,KType)) and +(KTypePol,[(Split,KType)]) term-list forms;
+(KTypePol,KTypePol) / -(KTypePol,KTypePol); *(int,KTypePol) /
*(Split,KTypePol); last_term/first_term(KTypePol->Split,KType) — the tuple
prints Split in FULL '(e+fs)' form and the KType WITH adjective prefix;
truncate_above_height(KTypePol,int); pol display per basic_io.cpp:165
print_K_type_pol: coefficient embellishment — full print_split only when
BOTH e and s components occur across terms, else bare e (or '{s}s'), then
'*' + ' K_type(x=N, lambda=rho+lam_rho)' (NO adjective) + ' [{height}]',
one '\\n' per term; empty -> 'Empty sum of K-types') →
`param_basic` (param(KGBElt,vec,ratvec->Param) =
Rep_context::sr(x,lam_rho,nu), rank check 'Rank mismatch:
({rank},{lam_size},{nu_size})' (atlas-types.w:6215); %(Param->KGBElt,vec,
ratvec) = (x, rc().lambda_rho(val), val.gamma()) — NOTE third component is
the INFO CHARACTER gamma, not input nu (atlas-types.w:6252; A1 x=2 anchor:
gamma=[0]/1 since lambda projects to 0 on the split Cartan); height stored
in StandardRepr (= K-type height); real_form; K_type(Param->KType) =
rc().sr_K(val) restrict; param(KType->Param) = rc().sr(K-type) with nu=0;
=/!= on StandardRepr; is_standard/is_final/is_zero predicates; display =
same 6-way adjective chain as KType + print_stdrep
'parameter(x=N,lambda=[..]/d,nu=[..]/d)' (basic_io); SLICE BOUNDARY:
register ONLY the fixture-gated set — the upstream install chunk continues
to equivalent/is_dominant/is_semifinal/dominant/normal/cross/Cayley/twist/
orientation_nr/reducibility_points/scale (atlas-types.w:7485-7495) but
those await their own contracts; needs crate StandardRepr/Rep_context
(repr.{h,cpp}), shared with ktype_basic) →
`parampol_basic` (ParamPol fixture-gated set: null_module(RealForm->ParamPol)
display 'Empty sum of standard modules'; #(ParamPol->int) TERM count;
+(ParamPol,Param) / -(ParamPol,Param) merging like terms (anchor: W-p
returns to the empty display); first_term(ParamPol->Split,Param) tuple with
Split in FULL '(e+fs)' form and Param WITH adjective; pol display per
basic_io.cpp:214 print_SR_poly: same coefficient embellishment as KTypePol
(full print_split only when both e and s occur, else bare e / '{s}s'), then
'*' + print_stdrep 'parameter(x=N,lambda=[..]/d,nu=[..]/d)' — NO leading
space, so terms render '1*parameter(...)' (contrast KTypePol's
'1* K_type(...)' whose print_K_type has a leading space) + ' [{height}]';
SLICE BOUNDARY: the install chunk's =/!=/K_type_pol/scaling/last_term/
truncate/scale-by-rat and deform/twisted_deform/block_deform
(atlas-types.w:8546-8570) await their own contracts — deform is the KL
deformation, a later-slice centerpiece) → `involution_primitive`
(involution(LieType,[int],string->mat) = basic_involution_wrapper
(atlas-types.w:860): Layout{type, checked_inner_class_type(symbols,type),
checked_permutation(perm)} then lietype::involution(lo) on the FUNDAMENTAL
WEIGHT basis of the simply connected group; checked_permutation wordings
'Permutation entry {e} too big' / 'Permutation has repeated entry {e}',
size check 'Permutation size {n} does not match rank {r} of Lie type';
involution(LieType,mat,string->mat) = based_involution_wrapper
(atlas-types.w:902): basis r x r check 'Basis should be given by {r}x{r}
matrix', then lietype::involution(type,class).on_basis(basis) with
InexactIntegerDivision relabelled 'Inner class is not compatible with
given lattice'; checked_inner_class_type (atlas-types.w:742): letters
"Ccesu" with punctuation skipped, 'Too many inner class symbols' / 'Too few
inner class symbols' / "Unknown inner class symbol `x'" / 'Complex inner
class needs two identical consecutive types', 'c'~'e' synonyms, and the
's'/'u' COLLAPSING rules (atlas-types.w:782+: 's' means the class of -1 —
where -1 lies in W (A1,B2,Cn,D2n,...) it collapses to 'c'; 'u' often
collapses to 's') — anchors: A1 "s" -> | 1 | (collapsed), A2 "s" -> flip,
A2 "u" -> flip, B2 "s" -> I2, A1.A1 "C" -> swap, A2 mat [[1,1],[0,1]] "s"
-> | 1, 1 | / | 0, -1 | via on_basis). CRATE RECON 2026-07-30:
InnerClassLayout exists (layout.rs:25 factors/letters/perm); the new work
is table-driven, no deep math: simple_involution (lietype.cpp:480) per
letter — complex = factor swap, unequal_rank = per-type tables (A
antidiagonal, D last-two swap, E6 0<->5+2<->4, T -1), compact/split =
identity under the layout permutation — plus the swap_sc collapsing
(lietype.cpp:~435: A1/B/C/D2n/E7/E8/F/G interchange c<->s, E6 and T map
u->s) and on_basis (topology.rs:184 already ports the integrality-checked
division); MEDIUM slice of exact tables.
`real_group`, `cartan_aggregation`, `seed_x0`, `involution_table`,
`adjoint_fiber`, `real_form_labels`, `overloads_ops_b8c{,_rejected}`,
`whattype_ops_b8d`, and `dont_b13{,_rejected}` are DONE (verified
`3501779` / `3502126` / `3502176` / `3502272` / `3502318` / `3502375` /
`3501643`).

Uncovered matrix items needing contract design first (probe the oracle,
then freeze): KL file formats and readline completion. For readline
completion the pty methodology is PROVEN (2026-07-30): python3 `pty.fork`
drives the oracle interactively — banner + `atlas> ` prompt captured, line
echo + value + next prompt read; harness must normalize CRLF (`\r\n`).
CAUTION the local macOS binary is an older build (Sep 10 2024, readline
DISABLED, axis 1.1) — fine for semantics probes (all matched HPC captures
byte-for-byte) but completion probing requires the readline-ENABLED frozen
binary, i.e. run the same pty script on the HPC login node against
`/public/home/majj/atlasofliegroups-4d3e9449/atlas`. `dont`, `showall`,
`quit`, and the basic interactive TTY banner/prompt are implemented; the
newly frozen language fixtures are covered by differential `3501643`. Deeper math
overloads (KL polynomials, `W_graph`, `deform`, extended blocks). The
relation-style datum constructors (`Smith_Cartan`/`filter_units`/`ann_mod`/
`replace_gen`/`quotient_basis`, atlas-types.w:937) are now FROZEN
(`domain/relations{,_rejected}`, capture `3502198`/`3502199`) and join the
implementation queue after `involution_primitive`; brief:
Smith_Cartan(LieType->mat,vec) = LieType::Smith_basis of the transposed
Cartan matrix + block invariant factors (torus factors: standard basis,
null factors); filter_units(mat,vec->mat,vec) drops factor-1 columns;
ann_mod(mat,int->mat) = annihilator_modulo; replace_gen((mat,vec),mat->mat)
substitutes non-unit columns ('Too many factors: {n} for {m} columns' /
'Column lengths do not match' / 'Not enough replacement columns' / 'Too
many replacement columns'); quotient_basis(LieType,[ratvec]->mat) =
replace_gen(S, C*ann_mod(M,d)) with per-generator validation against the
invariant factors ('Improper generator entry: {r} not a multiple of 1/{d}',
'Length mismatch for generator {j}: {a}:{b}') (atlas-types.w:639-677).
CRATE RECON 2026-07-30: LieType::Smith_basis (lietype.cpp:267) is per-block
matreduc::adapted_basis — which the crate ALREADY ports faithfully
(integer_lattice.rs:508, observable-bearing pivot strategy) — plus the
D-even columnOperation(r-2,r-1,1) tweak and torus identity blocks, so
Smith_Cartan is nearly free; the only genuinely new math is
annihilator_modulo (lattice.cpp, mod-d kernel, small); filter/replace/
quotient are language-level assembly; LIGHT-MEDIUM slice.

Legacy scaffolding triage (2026-07-30): the pre-v0-schema fixtures under
`tests/fixtures/commands/`, `lex/`, `parse/`, `negative/`, and the early
eval set (`containers`, `container_errors`, `context`, `exact_numerics`,
`scalars`, `slices`, `subscriptions`) use an older events schema that the
current harness cannot consume. Their behaviors are covered by the verified
B-slice corpus — including lexer-error batch recovery, confirmed working
today (`1 $ + 2` then `3` reports the syntax error, prints `Value: 3`,
exits 1). `eval/exact_numerics` and `eval/scalars` still pass verbatim.
They are NOT part of the compatibility gate; candidates for retirement in
a future cleanup pass rather than schema migration.

## dont/showall probe findings (2026-07-30)

- Bare top-level `let x = 3` is a SYNTAX ERROR in the oracle
  (`expecting IN or THEN or ','`); `let x = 3 in x` evaluates fine.
- `dont` is only valid where parser.y has `do_expr` (while bodies,
  do-if branches, case arms): `for` loop bodies are plain `expr` and
  reject it; `while true do dont od` also fails because after `DO` the
  `tertiary DO expr` rule wants `expr`. The do_expr `DONT` alternative
  (parser.y:442) makes `sequence(false, die)` — canonical usage is
  `if cond then dont else ... fi` inside `while` bodies (see
  atlas-scripts/test.at:43). A valid minimal probe was NOT yet found;
  try `while true; if false then dont fi od` shapes before writing the
  fixture.
- `showall` prints `Overloaded operators and functions:` then
  `name: (signature): {source}` per overload (huge); untested further.

## Environment facts

- Local: macOS, `export PATH="$HOME/.cargo/bin:$PATH"`; CLI at
  `./target/debug/atlas-cli`. Upstream C++ sources (read-only reference):
  `/Users/hoxide/mycodes/atlasofliegroups` (master `4d3e9449`).
- HPC: `ssh majj@10.26.14.64`, project `/public/home/majj/atlas-rust`,
  frozen oracle `/public/home/majj/atlasofliegroups-4d3e9449/atlas`
  (rev `4d3e9449062a07c1c85f4e6df215eb6ccc0eeae9`, binary sha256
  `66f5d7d47d560e616363392b38205166d1579985dc7337cc95ba4cae50be65c9`).
- Direct oracle probe (for designing new contracts; login node needs the
  gcc runtime):
  `ssh majj@10.26.14.64 'module load misc/gcc/12.1 >/dev/null 2>&1; gcc_lib="$(dirname "$(gcc -print-file-name=libstdc++.so.6)")"; export LD_LIBRARY_PATH="$gcc_lib:$LD_LIBRARY_PATH"; cd /public/home/majj/atlasofliegroups-4d3e9449/atlas-scripts && printf "<lines>\nquit\n" | /public/home/majj/atlasofliegroups-4d3e9449/atlas 2>&1'`
  A local oracle build at `/Users/hoxide/mycodes/atlasofliegroups/atlas`
  (built from the same frozen revision `4d3e9449`, different binary sha)
  runs the same probes without ssh — convenient for drafting; the HPC
  capture remains the verification of record either way.
- Reference capture: `ATLAS_BIN=... EXPECTED_ATLAS_BINARY_SHA256=66f5d7d... sbatch hpc/reference_capture.sbatch tests/fixtures/<sub>/<name>.atlas ...`
  (FULL paths with extension). Reports land in
  `results/<commit>/<jobid>/reference_capture/reference_capture_report.json`;
  per-fixture stdout/stderr text is embedded — verify verbatim against
  events.json before writing provenance.
- Meta provenance fields (order): fixture/oracle("atlas")/stage/
  reference_status/reference_atlas_revision/reference_binary_sha256/
  reference_job/source_archive_sha256/fixture_sha256/oracle_exit_status/
  oracle_stdout_sha256/oracle_stderr_sha256/capture_artifacts_sha256/
  rust_status/upstream_evidence/notes(/differential_job). The artifacts
  hash: on HPC in the capture dir,
  `shasum -a 256 "$PWD/x.stdout" "$PWD/x.stderr" > artifacts_x.sha256`,
  then take that file's own sha256. events.json status goes
  `pending_hpc_reference` → `verified_hpc_reference`; rust_status goes
  `not_implemented` → `verified_hpc` (with `differential_job`).
- Harness dirty detection ignores `atlas-*.out` everywhere
  (`b1afa5e`, `cbf538f`); `__pycache__/` is gitignored (`4843b9f`).
- Value/event encodings used in events.json: integers/booleans/strings
  plain; `{"type":"vec","display":"[ 1, 0 ]"}` (padded); rows unpadded
  `[0,1,0]`; `{"type":"ratvec","display":"[ 1, 0 ]/2"}`;
  `{"type":"matrix","display":"\n| 1, 0 |\n| 0, 1 |\n"}`; domain values
  `{"type":"domain","domain":"RealForm","display":"..."}`; KTypePol/
  ParamPol terms have a leading-newline display; any value may carry
  `display` verbatim (harness `render_value` short-circuits on it).

## Current state

**2026-08-11 checkpoint**（最新状态以此为准，下方旧段落仅作历史）：

- HEAD `d1a73b0`。最近 verified 切片：alcove/FPP
  （alcove_center/alcove_root_vertex/FPP_numers/FPP_w_shifts，实现
  `53581d8`，差分 **3533851** 全绿：201 fixtures，200 PASS / 1 已知
  PARTIAL container_syntax_errors / 0 FAIL；meta 升级 `7032dd9`）。
- 后续切片的 reference 已**全部** `verified_hpc_reference`（本地 pinned
  oracle 捕获与 HPC reference_capture 3535636/3535942/3536119/3536288/
  3536369/3536421/3536583 字节一致；`e045ec1`+`d1a73b0` 起）：
  `root_numbering`（根编号族 6 个）、`coroot_queries`（小件 sweep 8 个）、
  `orbit_ws`（orbit/ladder 4 个）、`print_gradings`、`poly_surface`
  （ParamPol/KTypePol skip 重载 ~10 个）、`real_weyl_print`
  （print_real_Weyl/print_blockstabilizer）、`print_x`（global KGB 表）、
  `print_common_block`、`dual_kl_block`、`twisted_family`
  （twisted_deform/twisted_full_deform/twisted_KL_sum_at_s）、
  `block_deform`。rust_status 均 `pending_hpc_differential`；
  **实现切片收尾时自行在 hpc/pipeline_swap_diff.py 注册 FixturePlan**
  （片段在 docs/slices/post_weyl_lang_queue.md §5 开头；不要提前注册，
  未实现会 FAIL 污染其他切片的差分）。预研事实见同文档 §4/§5。
- 在飞切片（串行纪律：atlas-core 归 ext agent，atlas-real-group 归
  RealWeyl agent）：extended_block/raw_ext_KL/partial_extended_KL_block
  三注册（语言层，实现已写完编译干净，收尾验收中）；RealWeyl crate
  切片**已交付** `51b9d83`（real_weyl.rs 1858 行，10 个字节级锚点；
  对偶侧精确 -θ fiber 链的坑见 REMAINING_BUILTINS.md）。
- 后续顺序：切片 A（coroot_queries 8 + root_numbering 6）→ 切片 B
  （orbit/ladder + poly 表层）→ 切片 C（print_gradings，等 RealWeyl）
  → dual_KL_block + KL_block 第二重载 → deform/twisted 族 +
  ext_param+star → print_X（global_KGB）+ print_common_block（两件套：print_block(Param)/print_common_block）。

- Branch: `main`.
- B3a non-recursive functions, B3b recursive functions / definition sugar,
  B3c parameter patterns, B3d selectors, B4 loops, B5 `set_type`, B6
  case / counted-for, B7 forget/die, B8 user overloads + `set`, B9
  redirect-body parsing, B10 file inclusion (accepted and missing-file),
  B11 precedence, and B12 subscription/runtime diagnostics are implemented
  and differentially verified. The exact commit is shown by
  `git log -1 --oneline`.
- InnerClass/RealForm values now print exactly as the oracle renders them
  (compact/split/quasisplit/disconnected variants, dual-form
  singular/plural), verified by differential `3501467`; the
  `pipeline_swap_domain_equality` fixture runs fully in the swap plan.
- Domain contracts frozen against the oracle: `root_coroot` + `kgb_generation`
  (implemented `af6cd7b`/`d7cef57`, verified `3501555`),
  `real_group` (verified `3501779`), `grading` (verified `3501915`) +
  `involution_primitive` (frozen `3501449`),
  `weyl_element` (verified `3502034`) + `kgb_operations` +
  `tits_operations` (verified `3501870`), `cartan_aggregation`
  (implemented `1989f62`, verified `3502126`) + `seed_x0`
  (implemented `babbefd`, verified `3502176`) + `involution_table`
  (implemented `72d42a8`, verified `3502272`) + `adjoint_fiber`
  (implemented `81eb98e`, verified `3502318`) + `real_form_labels`
  (implemented `fa90911`, verified `3502375`) +
  `weak_real_form` + `involution_decomposition` +
  `strong_real` (`3501500`), `split_basic` + `block_basic` (`3501519`),
  `ktype_basic` + `ktypepol_basic` + `param_basic` + `parampol_basic`
  (`3501537`) — all pending implementation except where noted.
- Eval contracts `overloads_ops_b8c{,_rejected}`, `whattype_ops_b8d`, and
  `dont_b13{,_rejected}` are implemented and verified by differential
  `3501643`.
- Harness: Slurm stdout files (`atlas-*.out`) no longer count as checkout
  dirt in either the bootstrap or the checked source-state helper
  (commits `b1afa5e`, `cbf538f`); `__pycache__/` is gitignored (`4843b9f`).
- No uncommitted repository changes should remain after the handoff commit.

The typed session pipeline is active: `session.rs` and `session_frame.rs`
convert/evaluate through `typed.rs`; the old dynamic `eval.rs` path is deleted.
The current typed surface includes scalar and linear values, subscriptions
(including string subscript with the oracle range wording), one-dimensional
slices, matrix/vector/ratvec crossings, RootDatum/Cartan constructors, the
exposed KGB constructor adapter, non-recursive functions: typed lambda
literals `(int n): body`, parameterless `@: body` closures with frame capture
(including escaped captures), `return` intercepted at the call boundary and
rejected at analysis outside a function body, identifier selector postfix
`receiver.name` lowered to `name(receiver)`, function-definition sugar
`f(params): body` in `let`/`set` declarations, `rec_fun` recursive functions
in declaration and expression form with explicit result types, binding and
parameter patterns (tuple destructuring, discard `type .`, const `!x`,
whole-value `(a, b): t`) compiled to a shared `SlotShape` frame layout,
operator/unit selectors (`2.-`, `2.3`) with operator selectors resolving
through the standard overload table, loops (`while`/`for` collecting each
iteration's body value into a row, `break` discarding the breaking iteration,
`for x@i` index binding, `;` sequencing), user overloads with merged
builtin/user dispatch (`Defined`/`Added definition [n]`/`Redefined` reports,
`whattype f ?` listings, shadow-on-exact-replace forget semantics), `set`
parallel bindings (all RHS analyzed, then evaluated, then bound), and
redirect bodies parsed as expressions before the sink opens. This is not a
claim of full Atlas compatibility: primitive `involution` constructors,
blocks, K-types, parameters, the KL layer, and the relation-style datum
constructors (`Smith_Cartan`, `filter_units`, `ann_mod`, `replace_gen`,
`quotient_basis` — atlas-types.w:937, not yet covered by any frozen
contract) remain pending differential evidence.

## Verified stage: real_form_labels matrices and block sizes (differential 3502375)

- `tests/fixtures/domain/real_form_labels{,_rejected}.atlas`:
  `occurrence_matrix`/`dual_occurrence_matrix` Cartan-membership bitmaps,
  `block_sizes`/`block_size` via the innerclass.cpp:1100 summation (orbit
  size × fiber size × dual-fiber size — no Block build), and `Cartan_order`
  over the poset relation. ZERO crate changes: the Cartan-ordering poset
  already existed as the `below` matrix with `is_below`
  (cartan_classification.rs, cartan_aggregation era) — the earlier recon
  note flagging it as the slice's main gap was outdated. A2 Cartan
  numbering confirmed consistent with upstream (the frozen occurrence and
  order matrices hit verbatim). Commit `fa90911`.
- Differential: `pipeline_swap_diff` job `3502375` at commit `fa90911`
  reports both fixtures PASS with zero regressions. Metadata carries
  `rust_status: verified_hpc` with `differential_job: 3502375`.

## Verified stage: adjoint_fiber central_fiber (differential 3502318)

- `tests/fixtures/domain/adjoint_fiber{,_rejected}.atlas`:
  `central_fiber(RealForm->[vec])` — the fundamental-fiber stabiliser of a
  real form's gradings (innerclass.cpp:1042/1020). The crate assembly
  reuses the strong-representative solve (`wrf_preimage_masks`, collected
  during the existing build loop) as the `toAdjoint` preimage, so no new
  solver was needed; `wrf_rep` = the fundamental partition's
  `class_representative`. Registered as `skip` (only conform-level
  diagnostics). The agent report records a theoretical caveat: list order
  follows the crate's augmented-span reduction, not upstream
  `BinaryMap::section` — observable only when `diff != 0`, which no frozen
  contract exercises (all three have `diff = 0`). Commit `81eb98e`.
- Differential: `pipeline_swap_diff` job `3502318` at commit `81eb98e`
  reports both fixtures PASS with zero regressions. Metadata carries
  `rust_status: verified_hpc` with `differential_job: 3502318`.

## Verified stage: involution_table printers (differential 3502272)

- `tests/fixtures/domain/involution_table{,_rejected}.atlas`: `print_KGB`
  in both upstream forms (full `kgbsize: N` + `Base grading: [..].` header
  and the selection form without the header) and `print_strong_real`
  (single- and multi-class layouts), ported column-for-column from
  kgb_io.cpp:60/output.cpp:490. Crate side: `InnerClass::canonical_involution_expr`
  (weyl.cpp:1359-1385) produces the `1^2x1^e` decoration words; the printer
  output drains through a new `EvaluationContext.printed` buffer into
  report events (`BuiltinImpl::DomainPrinter` prints at both levels and
  returns the empty tuple at single_value). The rejected contract's
  `Failed to match 'print_KGB' with argument type RootDatum` overload-miss
  wording required implementing the selection overload as upstream
  registers two. Commit `72d42a8`.
- The B2 Cartan/involution enumeration divergence this note recorded is
  RESOLVED: the numbering adapter (`CartanClassification::build` BFS
  discovery order with canonical representatives) landed after this stage;
  see the numbering-adapter entry in the live continuation.
- Differential: `pipeline_swap_diff` job `3502272` at commit `72d42a8`
  reports both fixtures PASS with zero regressions. Metadata carries
  `rust_status: verified_hpc` with `differential_job: 3502272`.

## Verified stage: seed_x0 synthetic KGB constructor (differential 3502176)

- `tests/fixtures/domain/seed_x0{,_rejected}.atlas`: `KGB_elt(RealForm, mat,
  ratvec)` — the atlas-types.w:4580 synthetic seed. Crate side:
  `InnerClass::twisted_from_involution` (root-permutation/coroot transport
  gate, left-conjugation to distinguished, weight-matrix comparison) and
  `KgbGraph::{lookup, seed_torus_part}` (kgb.cpp:716 lookup port; the
  `(v + θᵀv)/2 − g_rho_check` arithmetic with non-integral-coordinate coset
  rejection). Language side: shared `build_kgb_element` pipeline so call and
  validate emit identical diagnostics in the upstream wrapper order; the
  `(vec,int->ratvec)` division overload (`Denominator 0 in rational vector`,
  negative-denominator normalization) was added as a fixture precondition.
  Commit `babbefd`.
- Differential: `pipeline_swap_diff` job `3502176` at commit `babbefd`
  reports both fixtures PASS with zero regressions. Metadata carries
  `rust_status: verified_hpc` with `differential_job: 3502176`.

## Verified stage: cartan_aggregation domain surface (differential 3502126)

- `tests/fixtures/domain/cartan_aggregation{,_rejected}.atlas`: the
  CartanClass language surface — `Cartan_class(InnerClass,int)` /
  `Cartan_class(RealForm,int)` bound-checked constructors, `nr_of_Cartan_classes`,
  `most_split_Cartan`, `involution(CartanClass)`, `real_forms`,
  `dual_real_forms`, `square_classes`, `fiber_partition`, and the
  `Cartan class #N, occurring for X real form(s) and for Y dual real
  form(s)` display. Dual correspondence is computed at the crate as the
  negated covariant involution matrix matched by root-image permutation
  against the dual classification's twisted-conjugacy partition (upstream
  `innerclass.cpp:435-441` pairs `tw` with `tw·w0`, then canonicalizes,
  so matrix equality is unreliable — the permutation key is the same one
  `class_of` uses). Commit `1989f62`.
- Differential: `pipeline_swap_diff` job `3502126` at commit `1989f62`
  reports both fixtures PASS with zero regressions. Metadata carries
  `rust_status: verified_hpc` with `differential_job: 3502126`.

## Verified stage: B8/B9/B10/B12 + domain display (differential 3501467)

- `overloads_b8{,b,_rejected}`: user function overloads via `set f =
  (int x): x` — heterogeneous signatures accumulate (`Added definition
  [2] of f:`), same-signature replaces (`Redefined`), non-function values
  bind as variables coexisting with the overload table, `whattype f ?`
  lists instances, and merged dispatch inserts user variants among the
  builtins (commit `162216c`).
- `file_commands_b9{,_rejected}`: redirect bodies parse as expressions
  before the sink opens, so `> "x" set qfc = 10` fails with
  `syntax error, unexpected '='` and creates no file; the expression
  grammar accepts the parser.y:264 `set pattern := expr` form (analysis
  rejects it as not yet implemented) (commit `2a3eff6`).
- `fromfile_accepted_b10`: HPC-only include fixture, fixed by the B8
  `set` implementation.
- `runtime_errors_b12`: range errors carry the compact subscription
  source (`index N out of range (0<= . <L) in subscription EXPR`), tuple
  subscript is the axis.w:4101-4105 type error, and string subscript is
  legal with one-character results (commit `a3c2f8d`).
- `pipeline_swap_domain_equality` now runs fully: InnerClass/RealForm
  display matches the oracle (Dynkin classification, inner-class layout,
  dual counts, topology, real-form type naming, presentation bits;
  commit `b4c8dc6`).
- Differential: `pipeline_swap_diff` job `3501467` at commit `8feb364`
  reports all 31 fixtures PASS with zero failures (suite PARTIAL only for
  the three plan-level pending overloads: two `involution` constructors
  and the synthetic `real_form`). Metadata carries
  `rust_status: verified_hpc` with `differential_job: 3501467`.
- Harness fixes landed alongside: Slurm stdout ignored in dirty detection
  (`b1afa5e`, `cbf538f`), `__pycache__/` gitignored (`4843b9f`).

## Verified stage: B7 forget/die + B10 missing-file diagnostics

- `tests/fixtures/eval/commands_b7.atlas` (4 accepted events: `forget x`
  on an unknown name reports `Identifier 'x' not known`, `forget + @
  (int,int)` reports `Definition of '+@(int,int)' forgotten`, after which
  `1+2` resolves through int->rat coercion to `3/1`) and
  `tests/fixtures/eval/commands_b7_rejected.atlas` (`die` raises runtime
  `I die` and the batch continues; an undefined identifier is the name
  error `Undefined identifier 'x'`). Implementation: `Command::Forget` /
  `Command::ForgetOverload` / `Expr::Die`; overload removal is a
  per-context filter over the static builtin registry
  (`Analysis::forgotten`); the plain-identifier and assignment undefined
  wordings now match `axis.w:1431` (commit `f86fc68`).
- `tests/fixtures/eval/fromfile_b10.atlas` (2 io diagnostics for missing
  `<`/`<<` targets, batch continues, exit 0): span-less diagnostics render
  with the `<Kind> error:` header (commit `73c7d81`); the oracle prints
  the same lines bare, so the header is a harness-grammar surface, not an
  oracle wording change.
- Oracle captures: `3499657` (B7), `3500378` (B10).
- Differential: `pipeline_swap_diff` job `3500583` at commit `37e0f23`
  reports all three fixtures PASS; all previously verified fixtures PASS
  (regression clean). Metadata carries `rust_status: verified_hpc` with
  `differential_job: 3500583`.

## Verified stage: B6 case and counted for

- `tests/fixtures/eval/casefor_b6.atlas` (11 accepted events: integer case
  with 0-based in-range selection, remainder wrapping for out-of-range
  without else, else catching out-of-range, then catching negative,
  positional union case with function branches, counted `for i: n from m`,
  `downto`, anonymous `for : n`, and `e1 next e2` collecting e1) and
  `tests/fixtures/eval/casefor_b6_rejected.atlas` (2 rejected type errors:
  non-function union branch `found int while (int->*) was needed.`,
  disagreeing branch types `found string while int was needed.`).
  Implementation: `IntCase`/`UnionCase`/`CountedFor`/`Next` typed variants,
  `conform_types` wording aligned to the oracle `found {} while {} was
  needed.` format (commit `5f58160`).
- Oracle capture: `3499627` (commit `cfdd9cc`), PASS against the frozen
  oracle.
- Differential: `pipeline_swap_diff` job `3500495` at commit `6df6622`
  reports both fixtures PASS; all previously verified fixtures PASS
  (regression clean).
- Reference metadata: `tests/reference/eval/casefor_b6{,_rejected}.meta.json`
  carry `rust_status: verified_hpc` with `differential_job: 3500495`.

## Verified stage: B5 set_type

- `tests/fixtures/eval/settype_b5.atlas` (accepted: single-name `set_type`
  aliases with projector/injector overloads, bracketed `set_type [ ... ]`
  entering the tabled type map for case discrimination and recursion, union
  values displaying as `value.tag`, tabled types printing by name in
  `whattype`, `Defined type:`/`Type:` headers) and
  `tests/fixtures/eval/settype_b5_rejected.atlas` (rejected: `expr : type`
  ascription syntax error, case discrimination on a union named only by the
  single-name form, discrimination branches with disagreeing result types).
- Oracle capture: `3499601` (commit `559f363`), PASS against the frozen oracle.
- Differential: `pipeline_swap_diff` job `3500393` at commit `9bb95e3`
  reports both fixtures PASS (suite PARTIAL as long as
  `pipeline_swap_domain_equality` keeps its pending domain lines; B6-B12
  fixtures still FAIL until implemented).
- Reference metadata: `tests/reference/eval/settype_b5{,_rejected}.meta.json`
  carry `rust_status: verified_hpc` with `differential_job: 3500393`.
- Note: job `3500391` was invalidated by fixture-side file creation inside
  the frozen snapshot; commit `9bb95e3` moved fixture execution into an
  isolated per-run workspace directory.

## Verified stage: B4 loops

- `tests/fixtures/eval/loops_b4.atlas` (8 accepted lines: `while`/`for`
  collecting each iteration's body value into a row, `break` contributing
  nothing for the breaking iteration, condition-less `while do ... od`,
  `for x@i` index binding, `begin`-style `;` sequencing) and
  `tests/fixtures/eval/loops_b4_rejected.atlas` (4 rejected lines: top-level
  `break`, `break x` syntax error, iterating a non-row, non-boolean while
  condition). Implementation: `Sequence`/`While`/`For`/`Break` typed variants
  with analysis-time `loop_depth` legality and `Control::Break(usize)`
  evaluation (commit `5be00f9`).
- Oracle capture: `3498786` (commit `a5856a1`), PASS against the frozen oracle.
- Differential: `pipeline_swap_diff` job `3499732` at commit `152138ca`
  reports both fixtures PASS (suite PARTIAL as long as
  `pipeline_swap_domain_equality` keeps its pending domain lines).
- Reference metadata: `tests/reference/eval/loops_b4{,_rejected}.meta.json`
  carry `rust_status: verified_hpc` with `differential_job: 3499732`.

The bounded local checks for this stage:

- `cargo test -p atlas-core --lib`: 174 passed, 0 failed;
- `cargo clippy -p atlas-core --lib -- -D warnings`;
- `cargo fmt --all -- --check` and `python3 hpc/test_pipeline_swap_diff.py`.

## Verified stage: B3c parameter patterns and B3d selectors

- `tests/fixtures/eval/patterns_b3c.atlas` (5 accepted lines: tuple
  destructuring bindings, discard `type .` parameters, const `!x` bindings,
  whole-value `(a, b): t` patterns) and
  `tests/fixtures/eval/patterns_b3c_rejected.atlas` (3 rejected lines:
  const assignment, two pattern shape mismatches). Implementation: `Pattern`
  AST with `SlotShape` frame layout shared by let groups and call frames
  (commit `83debd3`).
- `tests/fixtures/eval/selectors_b3d.atlas` (3 accepted lines: unit selector
  `().f`, chained identifier selectors `2.f.g`, operator selector `2.-`) and
  `tests/fixtures/eval/selectors_b3d_rejected.atlas` (2 rejected lines:
  `2.+` without a unary-plus overload, `2.3` calling a non-function).
  Implementation: selector callee variants identifier/operator/unit-literal,
  operator selectors reusing `OperatorCall` overload resolution (commit
  `f6a5e5c`).
- Oracle captures: B3c `3498578`, B3d `3498619`, both PASS against the
  frozen oracle.
- Differential: `pipeline_swap_diff` job `3499673` at commit `a938573`
  reports all four fixtures PASS (the same run reports `loops_b4` FAIL,
  expected: its implementation was still in flight).
- Reference metadata: `tests/reference/eval/patterns_b3c{,_rejected}.meta.json`
  and `selectors_b3d{,_rejected}.meta.json` carry
  `rust_status: verified_hpc` with `differential_job: 3499673`.

The bounded local checks for these stages:

- `cargo test -p atlas-core --lib`: 169 passed, 0 failed;
- `cargo clippy -p atlas-core --lib -- -D warnings`;
- `cargo fmt --all -- --check` and `python3 hpc/test_pipeline_swap_diff.py`.

## Verified stage: B3a non-recursive functions

- `tests/fixtures/eval/functions_b3.atlas` (5 accepted lines) and
  `tests/fixtures/eval/functions_b3_rejected.atlas` (6 rejected lines: top-level
  return, argument type mismatch, wrong arity as void-vs-pattern, calling a
  non-function, missing-colon lambda syntax error, undefined selector target).
- Oracle capture: HPC jobs `3498312` (accepted) and `3498466` (rejected),
  both PASS against the frozen `/public/home/majj/atlasofliegroups-4d3e9449`
  checkout (revision `4d3e9449062a07c1c85f4e6df215eb6ccc0eeae9`, binary sha256
  `66f5d7d4...`, submitted with `ATLAS_BIN` and
  `EXPECTED_ATLAS_BINARY_SHA256=66f5d7d47d560e616363392b38205166d1579985dc7337cc95ba4cae50be65c9`).
- Differential: `pipeline_swap_diff` job `3498527` reports both fixtures PASS
  (stdout/exit/diagnostics exact; suite remains PARTIAL only for the known
  `pipeline_swap_domain_equality` pending cases).
- Reference metadata: `tests/reference/eval/functions_b3{,_rejected}.meta.json`
  carry the capture provenance and `rust_status: verified_hpc` with
  `differential_job: 3498527`.

The bounded local checks for this stage:

- `cargo test -p atlas-core --lib`: 154 passed, 0 failed;
- `cargo clippy -p atlas-core --lib -- -D warnings`;
- `cargo fmt --all -- --check`, JSON validation of the reference files, and
  `python3 hpc/test_pipeline_swap_diff.py`.

Only bounded local checks are appropriate here. The project policy puts full
workspace tests, Atlas/CWEB execution, differential jobs, and benchmarks on
XMU HPC.

## Verified stage: B3b recursive functions and definition sugar

- `tests/fixtures/eval/functions_b3b.atlas` (6 accepted lines: single- and
  multi-parameter definition sugar, `rec_fun` in declaration and expression
  form with explicit result types, parameterless sugar, recursive closures
  capturing their `let` scope) and
  `tests/fixtures/eval/functions_b3b_rejected.atlas` (3 rejected lines: body
  type error under sugar, recursive call with mismatched argument type,
  recursive declaration missing its result type).
- Oracle capture: HPC job `3498562`, PASS against the frozen oracle.
- Differential: `pipeline_swap_diff` job `3498653` at commit `f773695`
  reports both fixtures PASS.
- Reference metadata: `tests/reference/eval/functions_b3b{,_rejected}.meta.json`
  carry `rust_status: verified_hpc` with `differential_job: 3498653`.
- The bison expecting-list after `syntax error, unexpected IF` is not
  asserted; only the offending token is (see `docs/DESIGN.md` on diagnostic
  wording vs semantic equality).

The bounded local checks for this stage:

- `cargo test -p atlas-core --lib`: 160 passed, 0 failed;
- `cargo clippy -p atlas-core --lib -- -D warnings`;
- `cargo fmt --all -- --check`, JSON validation of the reference files, and
  `python3 hpc/test_pipeline_swap_diff.py`.

## HPC operations notes (verified this stage)

- The submit checkout must be clean at the declared commit. A previous job's
  root-level Slurm stdout (`atlas-*-<jobid>.out`) is untracked and makes the
  next submission dirty; move it away before resubmitting. The same applies to
  stale untracked sources (an old `eval.rs` leftover blocked one sync).
- The frozen oracle `/public/home/majj/atlasofliegroups-4d3e9449` is a git
  checkout at the pinned revision and must stay clean: job `3498017` failed
  because legacy `oracle-results/` and copied fixture files were left inside
  it (now in `/tmp/atlas-oracle-trash` on the login node). The unpinned
  `/public/home/majj/atlasofliegroups` tree is no longer a git repository and
  its binary differs from every pin; do not use it for captures.
- `reference_capture.sbatch` fails before the harness when declared and
  detected source state differ; the FAIL fallback report names the phase.
- After any commit that touches `crates/`, a subsequent rsync that excludes
  `crates/` (while a background agent holds uncommitted changes) leaves the
  remote checkout dirty against its HEAD, and a capture submitted in that
  window records `dirty_tree: true`. Repair with
  `git archive HEAD crates | ssh ... tar -x -C <remote>` before submitting,
  and re-capture anything taken in the dirty window (job `3499634` was
  re-taken as `3499638`).

## Next implementation slice (B7 misc commands in flight, then B8/B9/B10/B12)

In rough dependency order, each with its own fixture + HPC capture first:

1. B7 misc commands (capture `3499657`, commit `21ee423`): `forget` of
   unknown identifiers and of single overloads, `die` as a runtime
   diagnostic with batch continuation, coercion fallback after overload
   removal. The `whattype id_op ?` overload listing is deferred until the
   domain types appearing in builtin lists are ported.
2. B8 user overloads (captures `3499692`, `3499705`): `set f = <lambda>`
   accumulates overloads (`Defined f: T`, `Added definition [2] of f: T`,
   `Redefined f: T` for a repeated signature), `whattype f ?` lists user
   overloads in definition order, calls resolve by arity, and a variable can
   coexist with function definitions on one identifier; wrong-arity calls
   are analysis-time type errors.
3. B9 file commands (capture `3499747`; probe `3499729`, file evidence
   `3499737`): `> "f" expr` / `>> "f" expr` redirect only the
   `Value: ...` line (truncate/append), a failed open prints
   `Failed to open <name>` on stderr and continues, and `tofile` accepts
   only an expression (`set` there is a syntax error). The accepted lines
   already PASS as of job `3500393`; the rejected line needs parse failure
   before the output file is opened, and open failures must render through
   the `Io error:` diagnostic header.
4. B10 fromfile/quit (capture `3500378`): `< "f"` / `<< "f"` with a missing
   target print `failed to open input file '<name>'.` on stderr, batch
   continues, exit stays 0; `quit` mid-input terminates evaluation
   immediately, still prints `Bye.`, exit 0. Accepted-form inclusion
   semantics still need an HPC-absolute helper probe.
5. B12 runtime-error messages (capture `3500488`; differential `3500489`
   shows 2 of 5 already exact): row subscription out-of-range must append
   the space-free subscription source (`in subscription [1,2][5]`), tuple
   subscription with a non-constant index is a type error worded `Cannot
   subscript value of type (int,int) with index of type int`, and string
   subscription must exist as a runtime-checked operation.
6. Domain surface, smallest first: `pipeline_swap_domain_equality` lines
   3-14 (capture `3496440`). Gap analysis (2026-07-29, measured against the
   oracle): KGB `#0` numbering and all six equality/inequality events already
   match; the only blockers are two Display placeholders. (a) InnerClass
   print (`domain_builtins.rs:190-194`) needs: LieType reconstruction from
   the Cartan matrix (Dynkin classification + Bourbaki layout, no Rust
   module yet), inner-class type letters from the distinguished twist
   (`c`/`s`/`u`/`C`; `InnerClass::new` currently requires a distinguished
   involution and exposes no twist API), `numRealForms` (READY via
   `ExternalFormOrder::form_count`), and `numDualRealForms` (needs the dual
   root datum / dual weak-real-form partition — the largest sub-gap, no
   dual machinery in Rust yet). (b) RealForm print
   (`domain_builtins.rs:195-197`) needs: connected/compact/split/quasisplit
   flags (most-split Cartan involution export + dual component group —
   dual again) and the `printType` Lie-algebra naming module
   (`ExternalFormOrder` sorting is ported; per-form special gradings and
   the A/B/C/D/E/F/G/T naming branches are not). Upstream evidence:
   `atlas-types.w:3164-3172`, `3565-3575`; `output.cpp:751-782`. After
   those, the
   14 `tests/fixtures/domain/*.atlas` fixtures are blocked one level deeper:
   an Atlas-callable constructor/event adapter must exist before their
   oracle references can even be captured. Also uncovered: `showall`,
   `dont`, `quit` semantics, `whattype id_op ?` builtin listing, `fromfile`,
   KL/file formats, interactive input, and the primitive domain types
   (Split/Block/KType/KTypePol/Param/ParamPol).

Before continuing, run the smallest local parser/core check with the project
toolchain, then sync a clean committed tree to HPC and submit the relevant
SLURM job. Record the job id, reference revision, source commit, dirty state,
fixture manifest, exit code, and checksums in the reference metadata/report.

## Local environment

- `rustup` is installed through Homebrew.
- Stable toolchain: Rust 1.96.0; project `rust-toolchain.toml` selects stable
  and requires clippy/rustfmt.
- Rust 1.90.0 is also installed for the repository's earlier local gate.
- `rust-analyzer` is installed at `/opt/homebrew/bin/rust-analyzer`.
- `~/.cargo/bin` now precedes `/opt/local/bin` in `~/.zprofile`, so new shells
  use rustup's `rustc`, `cargo`, `clippy`, and `rustfmt` proxies. Restart the
  shell or source `~/.zprofile` before checking versions.

## Standing rules

- Read `docs/COMPATIBILITY.md`, `docs/LANGUAGE.md`, and `docs/DESIGN.md` before
  changing language behavior.
- Add/update fixture and reference metadata before implementation claims.
- Never hand-edit generated CWEB or parser output.
- Keep root-data and real-group invariants in their owned domain layer.
- Preserve unrelated user changes and do not commit unverified HPC output.## Remaining work after these slices:
- **D6 unlocked**: the column-echelon fix extends to rank 6 — D6
  kl_column, kl_sum_at_s, raw_KL, W_graph, partial_block all byte-identical
  (HPC captures 3516121-22, 3516175-77; final swap 3516180 in flight).
  D6 deform/full_deform/kl_print deferred (local oracle too slow);
  D6 block_Hasse differs under the fibred closure (needs the srm pool).
- **E6 coverage complete**: KL family + cartan_info/orientation_nr/
  simple_roots/two_rho (captures 3516083-84, 3516092-93).
- Next: the common-block srm pool (lookup_full_block z_pool) — the one
  remaining architectural blocker for KL_block/block_deform/extended_block
  and for mid-block block_Hasse. A simplified srm layer over-approximates
  (uses the real KGB x instead of the common_context subsystem view); a
  faithful port needs  (sub = integral subsystem of the
  dual), then z_pool BFS + srm_hash matching.

## Remaining work after these slices:
- **Full suite HPC-verified**: swap 3515917 — 189 fixtures, 0 FAIL, 1
  known PARTIAL (container_syntax_errors). E7 kgb_hasse, E6/D5 families all
  byte-identical. Meta ledger at verified_hpc/3515917.
- **Coverage now spans** A1-A4/B2-B4/C3-C4/D4-D5/E6/G2/F4 for the KL family
  (KL_column, KL_sum_at_s, raw_KL, kl_print, W_graph/W_cells,
  partial_block, partial_kl_block, full_deform, deform, block_hasse) plus
  cartan_info/orientation_nr/two_rho/simple_roots (A2-A4/B2-B4/C3/D4-D5/F4).
- Next big module: the common-block srm pool (lookup_full_block z_pool,
  needs the common_context subsystem view; C3 mid-block params still differ
  under the fibred closure). Then ext_block, twisted deform, print family.

## Remaining work after these slices:
- **COLUMN-ECHELON FIX (2026-08-04, 248aeb9)**: the E6/D5 "image basis
  factorization" failure and the A2 anchor mismatch shared one root cause —
  the incremental column-echelon port is not equivalent to C++'s one-shot
  `column_apply`. Fix = one-shot ops matrix with `ops(mindex,mindex)=-1`
  recorded + Euclidean row-reduction inverse + truncating division in
  `lambda_unique` (matches `arithmetic::divide`; the earlier A2/E6
  "contradiction" was a div_euclid artifact). A2, E6, D5 all pass; E7
  kgb_hasse verified on HPC fat (swap 3515688: 506s, 12.4G peak RSS).
- **Coverage sweep**: E6/D5 families (KL_column, KL_sum_at_s, raw_KL,
  kl_print, W_graph, partial_block, partial_kl_block, full_deform, deform)
  + A4/B4/C4/C3 extended; all byte-identical. print_KL_list enumerates the
  pool (empty blocks print the constant one). Final full swap 3515893 on
  fat (TIMEOUT=3600) in flight.

## Remaining work after these slices:
- **Batch coverage sweep (2026-08-04)**: A4/B4/C4 + C3 + D5 + G2/D4 KL
  family extended (KL_column, KL_sum_at_s, raw_KL, kl_print, W_graph/W_cells,
  partial_block, partial_kl_block, full_deform, deform, block_hasse,
  cartan_info, orientation_nr, two_rho). print_KL_list now enumerates the
  pool (empty blocks print the constant one). HPC captures 3515466-75,
  3515630-35, 3515698-99 verified; E7 kgb_hasse swap 3515688 RUNNING on fat
  (TIMEOUT=3600). New limit: D5 real forms hit the same column-echelon bug
  as E6 involution 187 (see REMAINING_BUILTINS.md).

## 2026-08-13 P0/P1 builtin continuation

- P0 oracle capture job `3543149` is pinned in the two
  `p0_simple_signatures` reference metas.  Mechanical type reconciliation and
  `(int,Param)` transforms are locally green and Rust-reviewed.  The B2
  proper-integral probe confirms the transform generator is an
  `IntegralSubsystem` generator, not an ambient simple-root index.
- One P0 differential remains intentionally open: `KL_block(p)` installs a
  full block in upstream's session `Rep_table`, so the following
  `KL_column(p)` returns raw row 1 instead of a fresh partial-block row 0.
  A cache keyed only by the exact seed was reviewed and rejected.  Port the
  `Reduced_param`/locator/shared block-pool semantics described at the top of
  `docs/REMAINING_BUILTINS.md`.
- P1 fixture contracts were committed at `b82adfe`; HPC reference capture job
  `3543697` was submitted for the accepted/rejected pair.  It covers Weyl
  left `#`, both `##` overloads, `Cartan_class(KGBElt)`, and the missing unary
  and term-addition KTypePol/ParamPol signatures.  Per standing rule, continue
  implementation without waiting for that job.
- P1 repair rule: polynomial terms use real-form owner identity, not structural
  equality. Canonical/default constructions share a logical owner; repeated
  genuinely custom constructions remain distinct even when they print and
  compare structurally equal. KType/Param `equivalent`, however, uses that
  structural equality. `Arc::ptr_eq` is not a substitute until canonical
  real-form values are actually interned. Signed generator conversion is exact
  `i32`; oracle probes reject positive `2147483648` as too large rather than
  wrapping it. Term-list addition must retain the sort-once/linear-coalesce
  path because upstream exposes this overload for large lists.
- P1 differential repair: job `3543756` ran 245 registered plans and reported
  243 PASS, one known PARTIAL, and one unrelated FAIL: the heavy `kgb_hasse`
  plan hit the default 30-second per-fixture timeout. The new P1 fixtures were
  not in `FIXTURE_PLANS`, so that job provided no P1 evidence despite their
  verified reference files. They are now explicitly registered; a resubmission
  must use a generous `TIMEOUT` (at least 120 seconds) so the existing E6/E7
  KGB sweep cannot mask the small P1 results. Before submitting, use
  `run_fixture` locally on the two P1 plans to confirm configuration, stdout,
  diagnostics, stderr parsing, and exit status all pass.
- P1 is now differential-verified by fat-partition job `3543762` at source
  `bcaa99a9c17f94ae4d630309f32e35a525703f5f`: both accepted and rejected
  fixtures PASS exact stdout/diagnostics/exit checks, with no FAIL fixture in
  the 247-plan run.  The overall report remains PARTIAL only for two older
  declared pending cases.  Oracle/Rust measurements are recorded in the P1
  reference metas; the Rust pair took 0.005s each at 7160/7068 KiB peak RSS.
  Operational lesson: adding a fixture and capturing its oracle is not enough;
  every new differential fixture must also be registered in `FIXTURE_PLANS`,
  and a full corpus submission must inherit the timeout/partition needs of the
  heaviest already-registered plan.
- P2 Block `W_graph`/`W_cells` is HPC-verified as an exact runnable subset by
  job `3543773` at source `9874ff6f6c6be99f792c2722396bff1f8c229404`:
  accepted and rejected plans both have all six runnable checks PASS, no
  fixture FAIL exists in the 249-plan report, and each plan carries one
  explicit `block(Param)`-dependent pending event.  Rust took 0.005s at
  7104/6760 KiB peak RSS.  Do not upgrade these metas beyond `partial_hpc`
  until the shared RepTable/ReducedParam pool restores the accepted overload
  and its candidate-set rejection wording.
- P3 Param twist implementation is locally complete pending its full HPC
  differential. Unary and explicit-matrix overloads deliberately follow
  different upstream paths. The edge case where the target KGB packet is
  absent is represented as the printable `UndefKGB` sentinel `4294967295`,
  with graph access guarded and undefined Param print weights transported and
  cached safely. Reference jobs: signatures `3543702`, nonstandard `3543783`,
  Param sentinel `3543792`, KGB sentinel `3543798`, and safe sentinel fields
  `3543906`. A blanket sentinel rejection was disproved: strict equality,
  `%`, `height(Param)`, and `real_form(Param)` are valid upstream and must stay
  on storage-only paths.
- P3 is differential-verified by fat job `3543916` at source `ee44bd0`: all
  six P3 plans PASS exact stdout/diagnostics/exit checks, and the full run has
  no FAIL fixture (overall PARTIAL only for previously declared pending
  cases). Rust wall times are 0.005-0.006s and peak RSS 6892-7252 KiB; report
  SHA256 `cd1618a82cd3e43dec23bf81376f04c7509c17f4262a48c744e61c1f95b1f065`.
- The next bounded repair is `full_deform` outer KTypePol accumulation. Oracle
  job `3543807` proves that two distinct KTypes with coefficient `1` both
  survive. Keep the claim narrow: the remaining deformation subsystem and
  timed overloads still require their architectural ports.
- The outer KTypePol accumulation repair is differential-verified by fat job
  `3543928` at source `5269fb6`: accepted/rejected both PASS exact, no FAIL in
  the corpus, 0.005s and 7056/6932 KiB peak RSS. Report SHA256
  `0b6346282fdac558b595f2953854a7d33a5f5463503d585b5da764578f576734`.
  This closes only the merge contract, not the remaining deformation engine.
- Param `W_graph`/`W_cells` static result contracts are pinned by oracle job
  `3543933`.  The accepted fixture deliberately inspects nested rows with the
  generic row-cardinality `#`; this exposed that core `axis.w` special
  operators are outside the 305-entry `install_function` inventory and need a
  separate compatibility ledger.  The Rust implementation and pipeline plans
  are locally green; record the swap job and benchmarks here after the clean
  committed differential completes.
- Hunger audit correction: the `install_function` hunger integer is an
  assignment pilfer/evaluation-order hint (`axis.w:1968-1984,7165-7235`), not
  a coercion mask.  Oracle fixtures now separate the three same-type
  assignment cases from already-runnable domain calls and the independently
  NYI timed deformation branch.  Reference capture job `3545163` was submitted
  from `cd1c9c9`; upgrade metadata only after its frozen report is inspected.
- HPC submission repair: jobs `3543992`, `3543998`, and `3545163` executed no
  fixtures because `ATLAS_COMMIT` was passed as a seven-character short SHA.
  Both capture scripts reject that value before installing their fallback
  report trap, so the jobs fail with exit `2:0` and produce no report or
  benchmark.  Always set `ATLAS_COMMIT="$(git rev-parse HEAD)"` on the remote
  checkout and assert its length is 40 before `sbatch`; corrected jobs are
  `3545169` (Param W-graph differential), `3545170` (root-transform oracle),
  and `3545171` (hunger oracle).  Never promote metadata from the failed jobs.
- Corrected Param W-graph job `3545169` is valid: both new static-contract
  fixtures PASS, the corpus has 256 PASS / 3 declared PARTIAL / 0 FAIL, and
  report SHA256 is `155f7a3da22fcdb48218a941dbdfc6d4dc78d1f9b53d138f4d78b4706bba33e2`.
  Corrected root-transform reference job `3545170` is also valid; report SHA
  `b2139d49c7f7a6f3d3bcf0137e8c24292f81b33e24e2738a8477c9207151f9cd`.
- Hunger job `3545171` is invalid evidence despite its capture report saying
  PASS: all four oracle invocations failed in the loader on `cu013` for missing
  `GLIBCXX_3.4.26/.29`.  A capture harness PASS only proves artifacts were
  written; always inspect oracle exit/stderr and plausible RSS/time before
  promoting reference metadata.  Job `3545182` re-runs on known-good `cu007`.
- `3545182` and the hunger-assignment capture `3545198` reproduced the same
  loader failure even on `cu007`.  The operational root cause is inherited
  compiler-module state combined with `module load misc/gcc/12.1 || true`:
  a module conflict was silently ignored and the script selected the system
  GCC 8 `libstdc++`.  A first repair that purged and reloaded the module still
  failed inside batch job `3545207` despite an equivalent diagnostic job
  loading it successfully. `reference_capture.sbatch` therefore binds the
  site GCC 12.1 installation directly (overridable with `ATLAS_GCC_ROOT`) and
  verifies `GLIBCXX_3.4.29` before running any oracle. Capture report PASS
  alone remains insufficient; inspect each raw oracle exit and stderr.
- Direct GCC-runtime binding fixed the capture environment. Job `3545219` on
  `35b783b46384edc9d453a13df299bc026ce28a9c` validly captured all four hunger
  contracts and both hunger-assignment contracts with exact local-oracle
  hashes and realistic 3.7-4.6 MiB RSS. Report SHA256:
  `d5d41520e3be0c947b93c0fcf9a6d6a77a4850b2073d98bcefe93867fe21cfcf`.
- Hunger execution is now implemented for the three observable same-result
  products.  The evaluator rewrites only a top-level builtin RHS of a simple
  assignment when the hungry operand is the exact destination binding.  It
  pilfers local/global slots, preserves hunger 1 right-to-left versus hunger 2
  left-to-right evaluation, leaves the destination unset after failure, and
  keeps aliases copy-on-write.  The five runnable hunger fixtures have
  `verified_hpc_reference` events and are registered in the swap runner;
  `hunger_contract_timed_nyi` deliberately remains outside it until timed
  deformation exists.  Fat differential `3545729` at `196dd7c` passed all
  five hunger fixtures exactly (full stage status remains PARTIAL only for the
  four declared project-wide pending features). Rust used 0.004-0.006s and
  5920-7316 KiB per hunger fixture; report SHA256 is
  `b0285ed87cf6898c245edbc1ea476d21b90468277c86c10e53f25a7f6b634bda`.
- Arbitrary-root Param transforms are implemented at `cc9e285`. Reference job
  `3545170` pins the A2 contract; job `3545520` pins successful three-step A3
  integral dominance and nonstandard-first rejection (report SHA
  `071d5589faf5f4dccd53a341ec8165de39dbd47f5c30a72ad7e0a7ad0dee6d7c`).
  The successful dominance word `[1,2,1]` is palindromic, so retain the direct
  root-first CWEB evidence for forward iteration and seek a non-palindromic
  fixture as a later coverage strengthening, not as an alternate algorithm.
- Initial implementation differential `3545555` executed no target Rust
  fixture: the four meta files had been upgraded after reference capture but
  their event files still said `pending_hpc_reference`, so the harness rejected
  configuration before spawning Rust. Keep event and meta reference statuses
  synchronized whenever a capture is promoted; empty Rust output at 0 seconds
  plus `configuration_valid=false` is configuration evidence, not a semantic
  mismatch.
- Corrected root-transform differential `3545623` at `a61a324` is valid:
  all four target fixtures PASS exact, and the full corpus has 260 PASS / 3
  declared PARTIAL / 0 FAIL. Rust took 0.005-0.006s at 6940-7252 KiB; report
  SHA256 `f27b6b6ebfada2aeaed23f240bb79aa698f340f8f7b2b9771ecf437ae9cb5d6b`.
- Shared RepTable sequence contracts are now frozen by oracle job `3545765`
  at `ce9034b`: standalone `KL_column` row 0; value `KL_block` and
  `print_common_block` install a full family and expose raw row 1; no-value
  `KL_block`, direct `print_block`, and `print_partial_block` do not install.
  Accepted/rejected used 0.015/0.009s and 4508/4360 KiB; report SHA256 is
  `b078c04a0fe0dd854deb7400fa491bd535e8fe1255532b605ba28504cc7d0ec9`.
- RepTable implementation has started at the lowest reusable boundary:
  `atlas-real-group/src/rep_table.rs` contains crate-private
  `IntegralSystem`, `ReducedParamKey`, and an `IntegralCodec` that reuses the
  existing Smith diagonaliser over the transported real-projection basis.
  Its tests pin negative Euclidean residues, multi-digit order, deliberate
  `u32` overflow, divisibility/shape rejection, theta-minus-one preimages,
  and key hashing.  Do not mistake this for a pool or `block(Param)` support;
  the next stage is the full `CommonBlock`/`BlockTopology` and all-row
  registration boundary.
- The KL half of that next boundary is implemented: sealed `BlockTopology`
  adapters for `BlockGraph` and `PartialBlock`, generic borrowed/`Arc` KL
  storage, and eager validation of rank/order/cells/link targets.  A B2 test
  drops the original `Arc<PartialBlock>` and still fills/queries its owned KL
  table, proving the future RepTable record needs no self-reference.  The full
  common-block packet constructor remains the next algorithmic step.
- Deformation alcove-shrink contracts are HPC-frozen by job `3546215` at
  `1cda0fe`: A1 denominator 3 makes `alcove_center` change `nu` from 1/3 to
  1/2 before both full deformation variants, while the rejected fixture pins
  the standard gate in no-value context. Accepted/rejected used 0.012/0.008s
  and 4368/4288 KiB; report SHA256 is
  `623e0650b86d18c795ba5d35b851f75cb681fb071b310cde3102b409759f9c2a`.
- Timed ordinary full-deformation oracle contracts are frozen by job
  `3547426` at `1a3e2e23`. Four separate processes pin the static overload and
  `.done` union, fresh `0`/`-1` millisecond `.timed_out` branches, and the
  cache/no-value ordering (discarded calls do not warm; unary calls do;
  bigint timer narrowing still diagnoses). They used 0.009--0.014 seconds and
  4344--4484 KiB RSS; report SHA256 is
  `97931b44e402672b0704a1caca595fcb4e5c91582d95325ab3ff82536fb75b04`.
  Rust commit `3b42183` uses a typed per-real-form completed-result cache and
  cooperative deadline checks inside ordinary deformation. Differential job
  `3551338` matches all four fixtures exactly (0.008s, 6972--7104 KiB); report
  SHA256 is
  `d59adb977b717ab1f43559f877ee8f64896d8b64a7e887da86b99341afaa31d0`.
  The lower-level RepTable still does not retain partial formula progress from
  a timed-out computation; that boundary is not exercised by the frozen
  fixtures and remains a compatibility/performance follow-up.
- Representation-table ownership direction: keep the mutable cache with the
  exact `InvolutionTable`/`KgbGraph` substrates in an `Arc<RepTableOwner>`.
  Construct short-lived `RepContext` views from that owner; never self-borrow a
  `RealFormContext`, use address tokens, or install a global table. Canonical
  real forms need a per-`InnerClassContext` weak interning table; custom forms
  always get fresh owners. The KL table itself must ultimately live in each
  shared block record, otherwise rebuilding `KlTable` in every language caller
  loses the cache effects observed by timed deformation.

## 2026-08-19 global.w sweep + locator/twisted-ext/partial-merge stage

- Reference ledger: 285+ fixtures `verified_hpc`. Remaining frozen anchors:
  3 locator + `ext_block_proper` + 2 `length_dual` + 4 `partial_merge`
  (all `not_implemented`, captured but unregistered).
- global.w ported in four batches. Batch 1 `15a3292` (differential 3574838),
  batch 2 `c5afd9c` (capture 3574906, differential 3574922: 291 PASS + 1
  declared PARTIAL), batch 3 `703a982` (matreduc.rs linear algebra, capture
  3574944, differential 3575810: 295 PASS). Batch 4 in flight (agent-76):
  `swiss_matrix_knife`, `mod2_section`, `subspace_normal` per
  `docs/slices/global_batch4_workorder.md`; the hidden "matrix slicer" /
  "transpose " signatures are parser-layer gaps (2-D slice syntax, commabarlist
  row display), recorded in REMAINING, deliberately not ported.
- Non-integral common-block slices 1-2 verified at `31064b1`:
  `length(Param)` via shared lookup, `print_partial_common_block` heads.
  Slice 3 remaining: `dual_KL_block(Param)` needs `PartialBlock::dual`
  (blocks.cpp:474-507, pure combinatorial reversal); see
  `docs/slices/nonintegral_common_block_workorder.md`.
- RepTable locator landed in steps: step-1 `79b6b9d` (BlockLocator/int_item),
  step-2 `740f4d8` (block_modifier arithmetic, 464 tests green). Step-3 in
  flight (agent-72): attitude gates on KL_column/KL_block/print_block(s)/
  kl_sum_at_s_terms + canonical keys into RepTable::lookup/lookup_full_block;
  brief at `docs/slices/locator_integration_brief.md`. Known defect pinned
  there: A2 SL(3,R) family identity-attitude shift is wrong (gamma-lambda rows
  0/2: oracle [-1,1]/2 vs Rust [-3,3]/2). Step-4 next: transport consumers,
  header, un-gate, register the three locator fixtures. Verified 2026-08-19:
  their events.json `Variable x: T` lines are CORRECT — both the oracle and
  the Rust CLI print `Variable rd: RootDatum` for `set x = ...`; the
  `Declaring identifier 'x': T` wording belongs only to the `x : T` / `x :=`
  declaration form (e.g. p0_simple_signatures). Do not regenerate them.
- Twisted/ext proper: workorder `docs/slices/twisted_ext_proper_workorder.md`.
  Slice order: 1 extended_block (1A constructor over PartialBlock in flight,
  agent-73; 1B wiring replaces the gate at domain_builtins.rs:14829),
  2 raw_ext_KL + partial_extended_KL_block, 3 twisted_KL_sum_at_s,
  4 twisted_deform, 5 twisted_full_deform recursion (hard-blocked on
  partial-merge NYI).
- Cross-block partial merge: workorder `docs/slices/partial_merge_workorder.md`
  (recon agent-75). `RepTable::commit_partial` merge minimal port: append /
  pool-extend / union-rebuild / retire; no Hasse move, block_access recomputed
  on demand. Two existing tests pin NYI behaviour
  (rep_table.rs:2057 `unsupported_partial_overlap_is_failure_atomic`,
  rep_table.rs:2246 concurrent overlap) and must be rewritten to pin merge
  results; the `length()` fallback arm (domain_builtins.rs:13478-13482) is
  deleted at merge time. Anchors F1-F4 frozen (capture 3575819, unregistered).
- Operational notes: subagents share the working tree; dispatch with strict
  file scopes, no subagent commits, `cargo fmt -p <crate>` scoping, and retry
  transient mid-edit compile errors after 60s. Quota 403s kill subagents —
  recover in place with `Agent(resume="agent-NN")`; context and tree edits
  survive. HPC full-corpus differentials must use the fat partition
  (`--partition=fat --time=01:00:00 --mem=32G --export=ALL,TIMEOUT=3600`);
  heavy fixtures OOM/timeout on cpu.
- All ten remaining anchors were pre-verified on 2026-08-19 by rebuilding
  expected stdout via `hpc.pipeline_swap_diff.expected_cli_observation` and
  diffing against the local Rust CLI: prefixes match exactly and every first
  divergence lands on its documented boundary (locator: A2 gamma-lambda
  defect + missing `as transformed by <...>` header; ext_block_proper: the
  `extended_block` NYI gate; length_dual_proper: `dual_KL_block` NYI;
  length_dual_proper_a2: the A2 defect; partial_merge_*: merge NYI — Rust
  prints unmerged partial rows where the oracle prints the merged
  `Subset {...} in the following common block`). Registration can proceed
  as each slice lands without regenerating events.
- Next-wave plan (beyond the current queue) is frozen at
  `docs/slices/next_wave_production_plan.md` (recon agent-77): every live
  NYI gate maps to the in-flight queue; the big remaining item is the
  non-integral common-block Param surface (`common_block_rows` gate,
  domain_builtins.rs:9431), then the parser pair (2-D slice, commabarlist),
  then KL_sum_at_s lambda-rho / full_deform scope, then small surface fixes.
  Concurrency rule: domain_builtins.rs items never run concurrently.
  Rank-0 non-integral ext builtins uniformly return a size-1 block upstream
  (plan §G) — verify at slice-1A review, else dispatch a small follow-up.

## 2026-08-19 evening: global.w closed, step-3 + slice-1A + partial merge landed

- global.w batch 4 (final global.w slice) is differential-verified:
  `swiss_matrix_knife`/`mod2_section`/`subspace_normal` at `68082cf`,
  capture 3576078, fat differential 3577111 (299 PASS + 1 declared
  PARTIAL, 0 FAIL; report SHA256
  `1a5ec2eba9ab4b555c9f32d202ef491bb24f2ab9bf61b8fbdc601ebb98d8ae64`).
  global.w is now fully dispositioned: 160+4 signatures ported or
  recorded as exclusions (parser-layer 2-D slice + commabarlist,
  readline_completions).
- Locator step-3 landed at `38a81f8`: canonical Reduced_param keying
  (`{x, int_sys, residue}` + IntegralDatumTable), lookup/lookup_full_block
  return LocatedBlock with the query-to-stored BlockModifier, six loud
  non-identity-attitude gates in domain_builtins.
- Slice-1A (ExtBlock over PartialBlock) landed at `1e36a3c`: ParentBlock
  trait, subsystem_cartan/subsystem_twist/transformed_twisted,
  build_partial, PartialBlockOracle; oracle-pinned B2/A2/C2 tests.
- A2 identity-attitude gamma-lambda defect FIXED at `c43e33c` (root cause
  by recon agent-78: located_common_block_rows applied bm.shift as a bare
  add; upstream re-normalises per element via Rep_context::shift /
  real_unique, repr.cpp:352-356). NOT a keying or rho-shift bug.
- Cross-block partial merge landed at `584717a` (agent-80): commit_partial
  merges overlapping partials (probe/build-outside-lock/re-verify/commit,
  retire = block_erase); attitude-mismatch merge stays loudly NYI; the
  length(Param) full-block fallback arm deleted at `6a0d867`. All four
  partial_merge_* anchors byte-match locally and are registered
  (`f291d59`); fat differential 3581761 in flight for promotion +
  regression of the whole corpus.
- In flight: agent-79 = locator step-4 (transported consumers, headers
  `as transformed by <...>`, singleton arms must route through the pool,
  gate release, registration of the three common_block_* anchors).
  After step-4: slice-1B (extended_block gate replacement + rank-0
  non-integral放行 + ext_block_proper registration), then dual_KL_block
  slice, then twisted slices 2-5 (slice 5 now unblocked by the merge).
  Next-wave plan beyond that: docs/slices/next_wave_production_plan.md
  (non-integral common block is the big one).

## 2026-08-19 late: dispatches after the merge landing

- partial_merge_{containment,union,chain,a2} promoted to verified_hpc by
  fat differential 3581761 (303 PASS + 1 declared PARTIAL, 0 FAIL;
  report SHA256 `b67e84026d63b8d2367d466418e03c4fbe2d167f2f5d5368c4d6b8194c35401a`).
- In flight (disjoint file scopes): agent-83 locator step-4
  (domain_builtins.rs: transported consumers, headers, singleton arms
  must route through the pool, gate release, register the three
  common_block_* anchors) — RE-DISPATCH of agent-79, which hung at the
  transport level (46 min, zero output bytes, zero edits; killed and
  replaced with a fresh instance on the same brief); agent-81 axis.w
  row operators ##/# (typed.rs only, new eval/row_operators fixtures);
  agent-82 PartialBlock::dual (partial_block.rs only,
  blocks.cpp:474-507, plus wiring instructions for the dual_KL_block
  domain slice).
- Serial rule remains: domain_builtins.rs is single-owner. After step-4
  lands, dispatch order is slice-1B (extended_block gate + rank-0
  non-integral放行), then dual_KL_block wiring, then twisted slices 2-5.

## 2026-08-19 evening: step-4 landed, row ops + dual landed, next wave dispatched

- agent-82 `PartialBlock::dual` landed (`d5419c3`): BareBlock + sealed
  BlockTopology, blocks.cpp:474-507 verbatim semantics, oracle-pinned B2
  proper/A2 split dual tests incl. dual KL matrices. dual_KL_block(Param)
  wiring instructions in the agent-82 report (10-step reroute through
  lookup_full_block + located params + singular_flags; needs lib.rs
  BareBlock export).
- agent-81 generic row operators landed (`a2c2737`): axis.w:1549-1579
  exact→generic→coercible order, hidden_special_variant shape matcher
  (2544-2595), four Row* ScalarOps. Fixtures row_operators{,_rejected}
  byte/payload-identical to oracle; captures 3582025/3582026; registered
  pending_hpc_differential (`f696a05`). `#:=` combined assignment recorded
  as a pre-existing parser gap.
- agent-83 locator step-4 landed (`5215c42`) — RE-DISPATCH of agent-79
  (hung 46 min, zero output; killed). print_common_block fully transported
  (print_c_block_wrapper headers `<w>` + `simple reflections permuted`,
  modifier-aware singular flags blocks.cpp:711-721, sr_with_modifier
  repr.cpp:815-823); ALL scopes route through lookup_full_block (canonical
  stored row order is oracle-visible); print_block proper arm now builds a
  FRESH block (closed a proven pre-existing init-index divergence). Nine
  attitude gates released (KL_column/KL_block/block/print_block/
  print_common_block/kl_sum_at_s/partial_block/W_graph+W_cells/
  block_Hasse); print_partial_common_block gate stays (step 5). Three
  locator anchors registered pending_hpc_differential; local run_fixture
  PASS ×3; 314 atlas-core tests.
- Fat differential **3582163 @ 5215c42 in flight** (locator 3 + row
  operators 2 + full-corpus regression). On 0 FAIL: bump the five metas
  to verified_hpc with differential_job=3582163.
- In flight: agent-84 parser pair 2-D slice + commabarlist (syntax.rs +
  typed.rs slice arm; swiss_matrix_knife engine already landed);
  agent-85 slice-1B extended_block on proper subsystems + rank-0
  non-integral (domain_builtins.rs, wiring per agent-73 notes + twisted
  workorder slice 1; registers ext_block_proper).
- Next after agent-85 frees domain_builtins.rs: dual_KL_block wiring
  (agent-82 instructions), then twisted slices 2-5, then next-wave A
  (non-integral common block, the largest remaining item).

## 2026-08-19 night: frozen corpus fully verified — 305/305 verified_hpc

- agent-85 slice-1B landed (`423445a`): extended_block on proper integral
  subsystems via extended_block_partial (slice-1A call pattern,
  distinguished delta atlas-types.w:7392); rank-0 non-integral falls out
  free (oracle: size-1 block, 1x0 matrices). raw_ext_KL /
  partial_extended_KL_block keep gates until slice 2.
- agent-84 parser pair landed (`3aaecc7`): 2-D slice M[r,c]
  (parser.y:658-705, SliceFlags column bits, swiss_matrix_knife caller)
  + commabarlist [a,b | c,d] (parser.y:370-410 via dedicated
  Expr::BarList — oracle probing showed the hidden "transpose " is
  overload-immune, so NOT the desugar). Known divergence recorded:
  `[ | 3]` expecting-token wording. Captures 3583469-72; registered
  `68a570f`.
- agent-86 dual_KL_block rewired (`f5e8aec`): lookup_full_block +
  KlTable<BareBlock> off PartialBlock::dual + located_singular_flags +
  located_row_parameter; the old uniform-lambda_rho non-integral
  divergence is gone. test_standard gate added (atlas-types.w:7055).
- Fat differential **3583557 @ 68a570f: 315 PASS + 1 declared PARTIAL
  (container_syntax_errors), 0 FAIL**; report SHA256
  `82d5d1d47ea5ce772e5080fdb4a4f6983e5f283f6e77d78dfad1cd927b9f07d1`.
  Seven anchors bumped verified_hpc (`45acc32`). **The frozen reference
  corpus is fully closed: 305 metas verified_hpc, 0 pending, 0
  not_implemented.**
- In flight: agent-87 twisted slice 2 (raw_ext_KL +
  partial_extended_KL_block on proper subsystems, ExtKlTable/condense over
  the partial-parent ext block + subsystem singular_flags; NEW fixtures
  ext_kl_proper{,_rejected} — oracle probing first, no events/meta/
  registration until HPC capture).
- Remaining queue (all need NEW fixtures + captures): twisted slices 3-5
  (twisted_KL_sum_at_s, twisted_deform, twisted_full_deform recursion —
  slice 5 may force the KL_table::swallow/partial-merge machinery);
  next-wave A non-integral common block (domain_builtins.rs:9431 gate,
  largest remaining item); B full_deform silent full-block approximation;
  C KL_sum_at_s per-element lambda-rho; E Weyl_orbit oversize semantics;
  F integrality_points display; locator step 5 (print_partial_common_block
  attitude + ext-block simple_pi induced); `#:=` parser gap.

## 2026-08-21: language-layer endgame — back_trace + tilde_opt + iffor_loop

- readline_completions slice fully landed (`138e7c5` impl incl. startup
  system variables input_path/prelude_log/back_trace and the const-override
  ` (constant)` wording fix; differential **3604405 PASS**, 333 fixtures;
  metas verified_hpc, LANGUAGE.md "interactive input and completion" row
  promoted to supported, REMAINING_BUILTINS stale conclusions corrected
  in `21efedd`). Registry audit: **469 builtins + 29 coercions all mapped**.
- back_trace semantics locked by oracle probing (main.w:651,
  global.w:1135-1148): runtime errors write the trace into the back_trace
  global ONLY when the trace is non-empty (sticky otherwise); line format
  `In call of g@int at <standard input>:3:0-4, defined at ...` with
  0-BASED columns (Rust SourceSpan is 1-based — subtract); def span starts
  at the function NAME (Rust function_binding currently starts at `(`);
  closures with params append a frame dump `{ x=2 }`, zero-param closures
  do not; builtin frames end `built-in.`; loop traces: `During iteration N
  of the [reversed ]for-loop` + frame dump, counted loops `During iteration
  N (i=V) of the counted [reversed ]for-loop` (no dump), anonymous counted
  has a DOUBLE SPACE in `of the  counted for-loop`; while has no iteration
  line. Recursive self-call traces (dynamic call line, self-binding dump
  with embedded multi-line closure print) DEFERRED to the let_rec patch.
- NEW grammar gap found: **tilde_opt on loops** (parser.y:319,523-571).
  `for x in L~ do e od` reverses input traversal (@index counts DOWN);
  `for x in L do e~ od` reverses output accumulation; both cancel.
  Counted for: tilde after count/bound expr reverses counting direction,
  tilde after body reverses accumulation; anonymous `for:n` allows only
  the body tilde; **DOWNTO has NO tilde_opt** (syntax error
  `unexpected '~', expecting OD`). `while c do e~ od` reverses the
  collected body-value list (while DOES collect body values into a list —
  verified: `while i<3 do begin i:=i+1; i end~ od` → [3,2,1]).
- NEW grammar gap found: **iffor_loop / quiet-if unit** (parser.y:365,
  506-521). `if c do e fi` is a general unit expression =
  `if c then [e] else [] fi` (returns a ROW; `if true do 42 fi` → [42]);
  `if c iffor fi` nests; every for_loop form accepts a do-less iffor_loop
  body wrapped in the `## ` drop-voids coercion (`for i:3 if i>1 do i fi od`
  → [2]); quiet-if takes NO else (`unexpected ELSE, expecting FI`).
  Rust rejects all of these today.
- Fixtures frozen: back_trace (`0df738a`, registered, capture 3604415),
  back_trace_let_rec (`f833bf1`, capture 3604440, unregistered),
  back_trace_loops (`a464e53`, capture 3604460, events/meta NOT generated),
  for_reversed (`fc37303`, capture 3604471, unregistered),
  for_reversed_extra (`46e52da`, capture 3604479, events/meta NOT
  generated), for_quiet_body (`aebfc7f`, capture 3604504 submitted).
  Local capture mirrors: /tmp/capture-3604471, /tmp/capture-3604479.
- In flight (parallel, isolated): **agent-98** implements back_trace
  call-stack tracing in the MAIN tree (evaluator + diagnostic + frames +
  syntax + typed + value); **agent-99** implements tilde_opt in the
  WORKTREE /Users/hoxide/mycodes/atlas-tilde-wt (branch codex/tilde-opt)
  to avoid evaluator collisions — merge back after agent-98 lands, then
  resume agent-99 for the iffor_loop/quiet-if extension (same ForTail
  productions). Generator template for events/meta: /tmp/gen_readline_events.py
  (accepted → rejected=False; error-line fixtures → rejected=True asserts
  oracle==CLI diagnostics, so the downto `~` diagnostic must land first).
- Known trap: /tmp/gen_back_trace_let_rec_events.py had a stale duplicate
  build() call from template adaptation (rindex truncation) — check any
  regenerated script has exactly one build().
- After these land, the ONLY non-supported LANGUAGE.md row left is the KL
  binary file format (filekl.w) — no language builtin touches it; deferred
  pending USER DECISION (exclude from the language gate vs port filekl).
  That decision gates goal completion.
  - Closure-printer probes (oracle, locked 2026-08-21): let-bound closures
    in frame dumps print multi-line too — `{ g=Function defined at
    <standard input>:1:17-29\n(y): %@(int,int)(y,0) }` (name-anchored def
    span; body = TYPED pretty-print with internal `op@type(args)` prefix
    form); the rec_fun self-binding is the same with a `Recursive ` prefix
    and `b = ` name line. Dynamic calls (through variables) trace as
    `In call of g at 1:33-37, defined at 1:17-29.` — no @type suffix but
    WITH defined-at taken from the closure value's span. Vec values dump
    as `[ 3 ]`. So the slice after counted-for tracing is: typed-expr
    pretty-printer + closure printer + let-frame dumps + dynamic-call
    defined-at (covers back_trace_let_rec.atlas).

## 2026-08-21b: counted-for tracing landed; tilde merge; two more slices in flight

- agent-100 landed (`6c25a29`): counted-for iteration trace lines
  (`(i=V)`, downto → `counted reversed`, anonymous keeps the double-space
  shared format, no frame dump) + group-transparent operator spans
  (parser.y:366 peels Expr::Group; fixes the 5:18-24 off-by-one).
  Differential for the combined tracing work pending the next full run.
- tilde_opt (agent-99, worktree branch codex/tilde-opt) landed there as
  `6401ca8` and was merged with main-tree tracing as `f123295`.
  Merge-resolution semantics (KEEP THESE): the for-in trace reports a
  TRAVERSAL-ORDER iteration counter, separate from the `@` index position
  (oracle: `[2,1,0]~` fails at `iteration 0` with dump `{ i=0 }`);
  `reversed` word in counted traces keys on descending (downto OR
  count-side tilde). back_trace_loops.atlas byte-exact vs capture 3604460
  incl. the reversed for-in line.
- NEW gap found: for-in over NON-ROW iterables (string→1-char strings,
  vec→ints, mat→columns as vecs, ratvec→rats; all reversal-compatible).
  Rust only accepted rows. Fixture eval/for_iterable_kinds frozen
  (`2457baf`, capture 3604537, events/meta generated, unregistered).
- In flight: agent-99 (resumed, worktree) implements iffor_loop/quiet-if
  + non-row iteration; agent-101 (main tree) implements the closure
  printer + let-frame dumps + dynamic-call defined-at
  (back_trace_let_rec.atlas, capture 3604440). After both land: merge
  worktree into main, register the six pending fixtures
  (back_trace_loops, back_trace_let_rec, for_reversed,
  for_reversed_extra, for_quiet_body, for_iterable_kinds), run the merged
  fat differential, bump metas, promote LANGUAGE.md rows.
  - Extra tilde diagnostics probe (oracle, 2026-08-21b): anonymous counted
    `for :3~ do 7 od` rejects with `unexpected '~', expecting IF or DO or
    FOR` (agent-99's current wording lacks the expecting suffix — fix at
    acceptance). NAMED plain counted `for i:3~ do i od` IS accepted and
    counts down ([2,1,0]); only the anonymous form lacks the count-side
    tilde.

## 2026-08-21c: unit-production audit — three more gaps frozen

- Systematic parser.y unit (339-386) vs Rust Atom audit found three more
  gaps, all probed and frozen:
  - **op_cast** (parser.y:381-383): `%@(int,int)`, `+@(int,int)`,
    `prints@string` select one overload as a value; rejection
    `No instance for mod@(int,int) found` (category type). Fixture
    eval/op_cast, capture 3604565.
  - **`$` last-value unit** (parser.y:343 make_dollar): value of the last
    evaluated expression, sticky across runtime errors. Fixture
    eval/last_value, capture 3604566.
  - **break N** (parser.y:385 BREAK INT): unwinds N+1 loop levels (Rust's
    Control::Break(levels) already unwinds — only the parser production
    and the analysis-time depth check are missing); rejection
    `Using 'break 2' requires 3 nested levels of loops`. Fixture
    eval/break_levels, capture 3604567.
  All three events/meta frozen (`9788171`), registration deferred.
- Dispatch plan: op_cast + `$` + the anonymous-counted tilde diagnostic
  wording (`expecting IF or DO or FOR`) go to agent-99's next resume
  (grammar area, same worktree); break N goes to the main tree after
  agent-101 (closure printer) frees typed.rs.
  - expr/tertiary level audit (parser.y:224-338 vs Rust, battery-diffed):
    CLEAN — OPERATOR_BECOMES (`x+:=3`), return, let-patterns, top-level
    multi-set `set (u,v)=(7,8)`, expression-level `set (p,q):=(4,5)`
    (incl. the Undefined-identifier-in-multiple-assignment wording) all
    match the oracle already. The only remaining grammar gaps are the
    frozen ones: tilde_opt (done in worktree), iffor_loop/quiet-if,
    non-row iteration (agent-99 in flight), op_cast, `$`, break N.

## 2026-08-21d: back_trace_let_rec landed; caselist dot-label gap found and frozen

- agent-101 landed (`507cdda`): let-frame trace dumps
  (`TypedExpr::LetGroup.names` + outlined `evaluate_let_frame`,
  typed.rs:11479 — the outline is required, inlining blew the test-thread
  stack on rec_fun depth 6), multi-line closure printer
  (`closure_trace_string`/`trace_value_string`, typed.rs:11530+),
  `compact_typed_expression` upgraded to `typed_expression_print`
  (typed.rs:645) with Conditional/elif/Next printing and
  `special_int_unary_print` (typed.rs:756, emulates the upstream
  special-builtin rewrite `x+1 -> succ@int(x)` at print time since this
  port deliberately skips that rewrite), dynamic call `defined at
  <closure span>`. Verified: back_trace_let_rec + back_trace stdout
  byte-identical to captures 3604440/3604415; 345 lib tests; clippy/fmt
  clean. Registered in harness (`4c90145`); differential job 3604616.
- back_trace_loops events/meta frozen (`b2c008f`, capture 3604460);
  registration deferred until agent-99's iffor/non-row iteration lands
  (the fixture's stdout is already produced correctly, but registration
  rides the merged differential).
- **caselist dot-label gap** (the last parser.y caselist production,
  419/426 `pattern '.' IDENT ':' expr`): tag AFTER the dot, binding
  pattern before — `(v).solution: #v`, `v.solution:`, `(a,b).pair:`,
  `(,).pair:` (throwaway slots) all accepted by the oracle; Rust rejects
  with `unexpected $undefined`. Real scripts use it
  (classical_W_classes_and_reps.at `(alpha,s).split_class:`,
  Gaussian_elim.at `(v,).affine_space:` — note trailing comma). Fixture
  eval/case_dot_label frozen (`5af4824`, capture job 3604622). Rejected
  wordings: `Branch has label bogus not associated to any variant of the
  union type mvv`; `Multiple branches with label solution` (both
  category type).
- **set_type bare-form quirk** (both sides already match, no work):
  `set_type name = (...)` WITHOUT the `[...]` list prints the definition
  message but does NOT register injector tags in type_map, so a later
  discrimination on that union fails with `Discrimination on expression
  of type (void|vec) requires using 'set_type' for this type, and naming
  injectors for it`. The list form `set_type [ name = (...) ]` registers
  tags. Rust already mirrors this exactly.
- break N dispatched to agent-102 (main tree; parser production
  BREAK INT + analysis-time depth check; Control::Break(levels) unwind
  already exists).

### op_cast / `$` extended probes (2026-08-21d, oracle)

- `IDENT '@' type` (parser.y:382) works on user overloads: `u@int`
  evaluates to the closure and prints MULTI-LINE at top level
  (`Function defined at <span>` + body line); `(u@int)(3)` applies.
  Rejection wording `No instance for u@string found` / `No instance for
  +@int found` (category type).
- Unary operator casts accepted: `-@int` -> `{-@int}`, `#@vec` ->
  `{#@vec}` (built-in closures print brace-wrapped name@type).
- `prints@string` displays as `{prints@T}` — the generic type variable
  leaks into the closure display even after a concrete cast.
- `$` (last value): sticky across runtime AND type errors; void-valued
  evaluations (`prints("x")`, `()`) do NOT update `$`; a bare `$` before
  any value evaluates to void (no Value line, no error). Bare `f` for an
  overload name is `Undefined identifier 'f'` (functions live in the
  overload table, not the identifier table) — Rust already mirrors this.
- op_cast/last_value fixtures extended, re-capture jobs 3604640/3604641;
  these probes define agent-99's resume batch scope.

### Counted-for tilde placement matrix (2026-08-21d, oracle probes)

- `for i:3 from 5~ do i od` -> [7,6,5]: from-side tilde reverses the
  counted range (starts at from+count-1, descends to from).
- `for i:3~ do i od` -> [2,1,0]: count-side tilde on NAMED counted
  accepted, implicit 0..n-1 reversed; but `for i:3~ from 5 do` rejects
  with `unexpected FROM` (no expecting suffix) — after count-side tilde
  no from/downto clause may follow.
- Anonymous counted is bare only: `for :3 from 0 do` rejects
  `unexpected FROM, expecting IF or DO or FOR`; `for :3~ do` rejects
  `unexpected '~', expecting IF or DO or FOR`.
- `for i:3 downto 0~ do` rejects `unexpected '~', expecting IF or DO or
  FOR` (no tilde after downto bound).
- `while c do e od~` rejects `unexpected '~', expecting '\n'` (trailing
  tilde after od).
- `for i@k in [7,8]~ do (k,i) od` -> [(1,8),(0,7)]: reversed for-in with
  @index iterates pairs in reverse with original indices.

### print/to_string/error variadic specials (2026-08-21e, frozen)

- Oracle probe + capture 3604701: `to_string` concatenates component
  displays with strings unquoted (`to_string(1,"a",[2,3],(4,5))` ->
  `"1a[2,3](4,5)"`, zero args -> `""`); `print` displays the argument
  TUPLE verbatim (strings stay quoted: `print("a",1)` prints
  `("a",1)`) and RETURNS it as the value (zero args prints `()`);
  `error` concatenates stripped text and raises it as a runtime error
  (zero args -> empty message). All three are shared_variadic_builtin
  specials (axis.w:2504, 8773+): never in the global overload table,
  never in startup completions. Rust currently reports Undefined
  identifier for all three; prints alone was done twice (agent-99
  worktree + agent-102 da7bae0 — dedupe at merge).
- Fixture eval/print_family frozen (events/meta, capture 3604701);
  implementation is the batch after agent-99's current one. to_string
  is used by 30+ atlas-scripts files, so this was a real coverage hole
  the builtin-registry audit missed (the specials are not in
  atlas-types.w's table).

### Special-operator sweep CLOSED (2026-08-21e)

axis.w:1806 is_special_operator is the complete list: `#` (size_of),
`##` (concatenate), protected `## `, print, prints, to_string, error.
Rust has `#`, `##`, protected `## `, prints; print/to_string/error are
the frozen print_family batch. No other hidden special operators exist
— after print_family lands this class is provably complete.

## 2026-08-21f: print_family verified; final merge staged

- print/to_string/error landed in main (`a9d078d`), registered
  (`a877560`), differential `3604733` PASS (336 PASS + 1 known
  container_syntax_errors PARTIAL), meta bumped to verified_hpc
  (`05625c5`). Special-operator class is closed 7/7.
- The 8 frozen fixtures (back_trace_loops, case_dot_label,
  for_iterable_kinds, for_quiet_body, for_reversed, for_reversed_extra,
  last_value, op_cast) are pre-registered in FIXTURE_PLANS (`0769fcd`)
  and pass every harness configuration check locally (validate_plan
  coverage, sha256, names, statuses, revision, exit status).
- FACT CORRECTION: agent-99 batch-1 `4c3b120` (quiet-if/iffor, non-row
  iteration, hidden prints) is NOT in main — `4024094` merged main
  INTO codex/tilde-opt, not the reverse. Local pre-check of the 4
  batch-1 fixtures against main HEAD fails as expected; they pass only
  after the codex/tilde-opt merge.
- Merge preview (scratch worktree, since aborted): only typed.rs
  conflicts, 9 hunks, all on known overlap surfaces — prints
  registration arm (dedupe), startup-completion exemption test
  (dedupe), eval_for_loop extracted version (keep main structure,
  absorb worktree features), agent-99's function outlining (do not
  revert). grammar.lalrpop and syntax.rs auto-merge cleanly.
- Post-PASS mechanics staged: `/tmp/bump_metas.py <job>` bumps the 8
  metas (indent=1 format verified against print_family.meta.json);
  `/tmp/language_md_promotion.txt` holds the LANGUAGE.md "Current
  Language Slice" replacement paragraph (fill in JOBID).
- After the merged fat differential PASSes, the only non-supported
  matrix row is KL binary file formats (deferred pending user
  decision) — that is the endgame question for the language gate.
- Also staged: `/tmp/postmerge_check.py` runs the 8 frozen fixtures
  through the harness against the main-tree debug binary in one command
  (exit 1 on any non-PASS). All three /tmp helpers are session scratch —
  rebuild them from this note if /tmp is cleared; the bump script logic
  is: load meta, set rust_status=verified_hpc, add differential_job,
  write back with indent=1 + trailing newline.

## 2026-08-21g: final grammar wave merged; differential 3607276 in flight

- agent-99's last four slices landed on codex/tilde-opt as `773030d`
  (op_cast exact-match casts, `$` analysis-time capture, caselist
  dot-label branches, tilde expecting-matrix wordings) plus a main-agent
  addition: failing `set` commands now append the oracle context line
  `Error in 'set' command at <loc>:` (global.w:1116-1130; the location
  spans the terminating newline because parser.y:140's `@$` includes the
  `'\n'` token).
- Main was merged INTO the worktree first (`060086e`): 9 conflict hunks
  resolved (syntax.rs break_shape union; typed.rs prints dedupe,
  variadic-special registry/selector arms, eval_for_loop keeps the
  extracted helper and absorbs break-N, startup-completion exemption for
  all four variadics, main's fuller tests kept, dead append_stripped
  dropped). The worktree then fast-forwarded into main with no further
  conflicts.
- Local gates on the merged main: 361 lib tests green, clippy
  `-D warnings` clean, fmt clean, 8-fixture gate 7 PASS + 1 declared
  PARTIAL (for_reversed_extra's trailing-tilde quit artifact, registered
  as PendingCase in `ccb7885`). Full 345-fixture local sweep: only
  fromfile_accepted_b10 (HPC-absolute include path) and kgb_hasse (60s
  local timeout) fail locally — both environmental, both previously PASS
  on HPC.
- Merged fat differential submitted as job `3607276` (all 345 fixtures).
  On PASS: run `/tmp/bump_metas.py 3607276`, apply
  `/tmp/language_md_promotion.txt` (LANGUAGE.md slice paragraph), commit
  and push. The only remaining matrix row is then KL binary file formats,
  deferred pending user decision.

## 2026-08-21h: merged differential 3607276 PASS — language matrix closed

- Job `3607276` (fat, 4m12s) on the merged tree `060086e`: 345 fixtures,
  **343 PASS + 2 declared PARTIAL, 0 FAIL**. The PARTIALs are the two
  registered harness artifacts (container_syntax_errors dangling bracket,
  for_reversed_extra trailing-tilde quit). All 8 frozen fixtures pass.
- The 8 metas are bumped to verified_hpc with differential_job=3607276;
  LANGUAGE.md's Current Language Slice paragraph now records every
  grammar-level row as supported. The meta census shows zero remaining
  pending_hpc_differential entries.
- ENDGAME DECISION POINT: the only non-supported matrix row is KL binary
  file formats (filekl.w, stand-alone utilities; no Atlas-language builtin
  reads or writes them). Deferred pending a user decision: exclude from
  the language gate vs port filekl.

## 2026-08-23b: corpus loop — for-loop required steering, recursive-type fix, term subscription

- `6feff9e` `convert_for_loop` now mirrors the upstream for_expr case
  (axis.w:5883-5924): the REQUIRED type steers the body — row context
  hands its component type to the body (so a `[(Split,KType)]`
  requirement narrows a body yielding `(int,KType)` per component; this
  unblocked basic.at:1721 `0*P + for x@q in P do (+%x,q) od`), a void
  context evaluates the body for side effects, and other required types
  go through a registered row coercion wrapping the loop.
- `6208ae0` iffor fix: the iffor body is a conditional producing ROWS
  (parser.y:509-522 wraps branches in list displays and the loop in a
  protected `## ` join), so its required component is itself a row of
  the loop's eventual component, and a void context never reaches an
  iffor body (the join is voided instead). Nested `for A for B if …`
  loops parse as outer-iffor over inner-for (iffor_loop includes
  for_loop), which the row-wrap handles. Same commit: corpus driver
  caps each child at MEM_CAP_GB (default 6) GiB via RLIMIT_AS so one
  diverging script cannot OOM-kill the SLURM job (job 3614680 was
  cgroup-OOMed by lazy_lists.at before).
- `544b316` tabled types compare by NUMBER in `coercions::same` /
  `broader_eq`. Root cause of the lazy_lists.at OOM/SIGSEGV: expansion-
  first structural comparison recursed forever on recursive types
  (`inf_list = (->inf_node)`) — diagnosed by gdb stack sampling on the
  live runaway process (all frames `coercions::same`). The table
  canonicalises, so number equality is type equality.
- `9f4d015` KTypePol[KType] / ParamPol[Param] term-coefficient
  subscription (analysis + eval), with the oracle's real-form mismatch,
  test_final (K-type reason chain: dominant/zero/semifinal/normal, else
  "not standard") and test_standard + made_dominant lookup; reversed
  subscription rejected at analysis. Unblocks basic.at:2046 `branch`.
- Corpus trajectory: basic.at blocker moved 1721 → 339 → 2046 → (runs
  to end; only the 5 subscription errors remained at 6208ae0).
- Known not-done: `P[t]:=s` coefficient ASSIGNMENT (upstream
  assign_coef, atlas-types.w:5655/7771) not yet implemented; watch the
  corpus for it.

## 2026-08-22: corpus loop — do_expr, tabled patterns, return cells, inner-class chain

- `075c5e8`/`2e1df2d` do_expr refactor + lalrpop borrow-after-move fix.
- `66982ed` `bind_pattern_leaves` expands Tabled types (fixes destructuring
  a tabled tuple like `ratmat(M,,d)`); whole-pattern binds keep the tabled
  type.
- `3d9a332` a local variable shadowing an overload name untables only when
  the local type is `Function` with undetermined list (fixes lazy_lists
  `denom`).
- `67a1395` empty root datum: `build_explicit_datum` accepts a 0x0 Lie
  type (atlas-types.w:1230); `as_matrix_rows_and_cols` preserves column
  count for 0xN matrices; lattice_rank<semisimple_rank and the Cartan
  check failures report the oracle wording "Matrices of (co)roots give
  invalid Cartan matrix".
- `f070407` return values convert against a per-function RESULT-TYPE CELL
  (`Analysis.return_type`): `convert_lambda_expression` builds a placeholder
  cell, seeds it with the annotated/tabled result type, specialises it with
  the converted body type, and `Expr::Return` converts against the cell —
  so `return` in a void context (for-loop body) is no longer voided
  (fixes `lex_lesseq`-style functions returning () and W_orbit's
  "(bool,WeylElt) while void was needed").
- `2ad918c` `extend` validates per-letter rank bounds
  (lwb[1,2,2,4,6,4,2,0]/upb, total-rank cap 32) with the oracle's exact
  wordings, and expands 'T' into `rank` separate T1 factors so rank 0
  adds nothing (atlas-types.w:205-207). This cleared 172 corpus scripts
  blocked on "Too few inner class symbols"
  (`inner_class(simply_connected("G2"),"e")` built a spurious T0 factor).
- `167e11d` every LINE of a multi-line report is prefixed by setw(2*depth)
  (global.cpp:2972-2984): the `set_type` echo's `with injectors:` line
  indents too. OUTPUT_DIFF dropped 34→10.
- `d16bb00` counted-for loops steer the body from the REQUIRED type like
  ordinary for loops (axis.w:6457-6464): row context hands the component
  type down, other types go through row_coercion wrapping the loop.
  Fixes polynomial.at:358 `for i:r do for j:c do [M[i,j]] od od`
  ([[vec]] target).
- `0e24226` component/field transforms only optimise to an in-place
  transform when the resolved call is a BUILTIN whose first argument is
  the unconverted selection (axis.w:8422-8455, 8572-8596); a user
  overload (e.g. `set # = #@([vec],vec)` storing a BuiltinFunction
  denotation, basic.at:468) or an implicit conversion falls back to an
  ordinary component/field assignment of the whole call. Killed the
  typed.rs:4290 `unreachable!` panic in 9 scripts (all ParamPol users
  via W_order.at's `new#:=j`). Known divergence: the fallback does not
  let-wrap a side-effecting index expression (upstream does); the index
  is re-evaluated inside the converted call's subscription.
- `a9eba08` real-form order tiebreak: adjoint fiber bit i maps to the
  i-th TWIST-FIXED simple generator ascending — upstream defines the key
  by `compacts.unslice(simple_roots_imaginary())` (cartanclass.cpp:929-948,
  innerclass.cpp:656-663). The previous shift-based "verification"
  (first flipped position of the grading shift) misassigned bits whenever
  a shift flipped more than one simple-imaginary position (A3 dual) and
  fired its own invariant on essentially every nontrivial inner class
  (180 corpus scripts blocked at basic.at:1380). Replaced by the direct
  unslice enumeration.
- `be68ac4` discrimination (`case`) branches convert against the SHARED
  context type, not an undetermined type merged afterwards
  (axis.w:5179-5189: no balancing; void context voids every branch) —
  fixes Gaussian_elim.at:86 `case pi | pivot: i+:=1 | else () esac` in a
  do-body. Same commit: the for-loop aggregate is untabled before the
  iterability check (axis-types.w:375-384 `kind()` untables), so a tabled
  row like `sparse_mat` iterates (sparse.at).
- grammar: `'@' cast` parses as a nullary lambda whose body is the cast
  (parser.y:226) — number_theory.at:102 `@bool: is_Fermat_prime(b,n)`.
- Corpus trajectory (240 scripts, HPC script_corpus.sbatch):
  3614728: MATCH 4-ish/OUTPUT_DIFF 21, blockers ratmat 84 + W_orbit 29;
  3614734 (f070407): MATCH 10, inner-class blocker 172 surfaced;
  3614781 (167e11d): MATCH 35, OUTPUT_DIFF 10, twist-fixed 171 surfaced;
  3614790 (0e24226): panics gone, polynomial gone, MATCH 35, 7
  PARSE_FAIL (`@bool:`) — since fixed.
- Watch: e8_gap.at matrix subscription out-of-range at
  `e8_gap_sgn_reflection[1,power_3[j]]` (suspect `^[...]` matrix-from-rows
  shape); corpus stderr cascades ("Abandoning reading of file") make later
  syntax errors noise — only the FIRST error (report category) matters.
- `9159a84` **E8 blocker fixed**: `twisted_conjugacy_partition` no longer
  enumerates the Weyl group. New `InnerClass::involution_orbits`: phase 1
  Cayley BFS from the identity twisted involution (innerclass.cpp:218-291,
  positive imaginary roots in upstream RootNbr order, successors
  canonicalized, deduped by root-image permutation), phase 2 cross-action
  closure `w ↦ s·w·twist(s)` per class (involutions.cpp:362-379, the
  `Cartan_orbit` constructor). Oracle-measured E8 numbers: 10 Cartan
  classes, 199,952 twisted involutions total (vs |W|=696,729,600, so the
  old filter-all-of-W approach could never fit the 4M budget). The budget
  now bounds the TOTAL twisted-involution count (upstream InvolutionTable
  size). Partition class order is now Cayley-BFS order (fundamental
  class first); `CartanClassification` still reorders via its own BFS, so
  consumers are order-insensitive. Removed now-dead
  `theta_generator_permutation`/`inverse_permutation` (inner_class.rs) and
  `piece_root_permutations`/`element_root_permutations` (weyl_transducer.rs).
  Corpus job 3614854 cancelled at 45/240 (31 E8-budget failures made its
  continuation useless); re-submitted as 3614863 on this fix.
- `33982ff` matrix pair subscription is (row, column), not (column, row):
  all three typed.rs sites (read, assign, transform) had treated pair[0]
  as the column. Oracle evidence: `mat:[[1,2],[3,4]]` builds columns
  [1,3],[2,4], and M[0,1]=3. The old unit tests had frozen the wrong
  semantics and were corrected. Root cause of e8_gap.at:575 out-of-range.
- `fda87a6` grammar: a `.` selector also accepts an if-expression as its
  unit (parser.y:321, `selector: unit`) — combinatorics.at:629
  `. if sign else +@(int,int) then -@(int,int) fi` (the else-first form
  already existed in IfTail). Source of 5 corpus PARSE_FAILs.
- `7581708` a tuple display in void context is voided componentwise
  instead of erroring "found (int,int) while void was needed"
  (combinatorics.at:826 `if (e:=S1[i],j:=i+1); while ...`, also
  `begin (1,2); 3 end`). Job 3614878's histogram was 43x this error —
  all already fixed before resubmit.
- `ace208c` lazy KGB pipeline: upstream real_form is a cheap handle; the
  KGB graph is built only when a builtin needs it. Eager construction
  cost ~90s per script loading groups.at for E8 forms never touched.
  RealFormContext now holds `seed: RealFormSeedPlan` +
  `kgb: Mutex<Option<Arc<KgbBundle>>>` (double-checked, build outside
  the lock); ~169 use sites go through kgb()/kgb_or_structure().
  NOTE: probing the oracle with `KGB_size` was a mis-measurement — it
  does NOT build the graph either.
- Residual E8 perf (post lazy-KGB): scripts that DO touch E8 KGB still
  pay rf1 19s / rf2 68s (KGB=67,110 / 320,206). Hot spots from gdb
  sampling: `ParabolicPieces` key() ~150K x 100us (mitigated by
  acea32d left_descend), plus KGB BFS `InvolutionTable::push_record`
  (TwistedInvolution::new / RootInvolutionData::new / saturated_kernel).
  perf(1) symbolisation fails on the HPC nodes; use gdb stack sampling.
- `433a21b` forget/forget-overload reports are NOT indented by include
  depth: upstream global_forget_identifier (global.w:1241-1248) and
  global_forget_overload (global.w:1253-1261) write to *output_stream
  directly; only definition reports emit setw(2*input_level)
  (global.w:937/1162/1388/1399). The centralised ReportLine indenter in
  session_frame.rs padded "Identifier 'save_s' forgotten" inside
  ratmat.at — the single OUTPUT_DIFF line in A1/center/central/
  Gaussian_elim_Jeff. New PlainReportLine event variant renders verbatim.
- untable read subscription: subscr_base::index_kind tests
  aggr.kind()==row_type and type_expr kind() untables transparently
  (axis-types.w:375-384), so a tabled row like Levi_subgroups.at's
  orbit_data=[orbit_entry] subscripts with int (47 corpus scripts
  blocked). The read path matched the raw Type; assignment already
  untabled. The not_so diagnostic still prints the original named type.
- overload twin check uses table-aware equality: upstream
  type_expr::operator== (axis-types.w:807-825) treats a tabled type as
  equal to its expansion, so synthetic.at:21 `set KGB_elt(KGBElt_gen x)`
  REPLACES the builtin KGB_elt@(InnerClass,mat,ratvec) (oracle:
  "Redefined KGB_elt: (KGBElt_gen->KGBElt)"). Structural == fell into
  the too-close error arm. New Type::equals(table) mirrors the operator.
- hpc/quick_check.sbatch: compiles HEAD in a throwaway git worktree with
  its own CARGO_TARGET_DIR — safe beside a running corpus job (which
  owns the main checkout target/). Never cargo-check in the main
  checkout while corpus runs, and never compile locally at all.
- OUTPUT_DIFF sweep (probe: double-run 32 scripts vs oracle, classify
  diff lines): every non-forget diff fell into three classes, all fixed:
  (a) set_type echo must print EVERY field position — holes contribute
  just their separator (global.w:1705-1725): `with projectors: , x, mu.`;
  (b) bracketed set_type echo AND whattype print a void function-arrow
  side as `void` (`(void->int)`), while definition reports print it
  empty (`(->int)`) — Type::display_in_set_type threads a void_arrow
  flag (single-name set_type keeps the plain spelling, global.w:1390);
  (c) a tabled FUNCTION type routes set bindings to the overload table
  (global.w:938 kind() untables) — inf_list = (->inf_node) reports
  `Defined Fibonacci: inf_list`, not `Variable ...`; add_user takes the
  arg type from the expansion but keeps the tabled type for the report.
- Corpus trajectory: 3614947 (ace208c, lazy KGB): MATCH 43,
  OUTPUT_DIFF 36, RUST_EVAL_FAIL 159 — histogram dominated by two
  already-fixed blockers (134x orbit_data subscript, 7x KGB_elt
  overload). 3615064 resubmitted at the full fix batch.
- quick_check.sbatch gotcha: cargo check needs --all-targets to compile
  test code (a new enum variant broke 5 exhaustive test matches only
  there); the worktree approach keeps it parallel-safe with corpus.
- Corpus 3615211 (6f8d821): MATCH 82, OUTPUT_DIFF 20, RUST_EVAL_FAIL 136
  (135x `No instance for maximal@KGPElt found` at parabolics.at:124).
  Benchmark: comparable 102, 95 over 5x slower; every script pays a
  constant ~4-5s — gdb sampling pins it on EAGER E8 inner-class
  construction (involution_orbits ~3.7s for 199,952 involutions).
  Upstream is ALSO eager (innerclass.cpp construct task 1) but does it
  in ms — this is a per-operation perf gap, not a laziness gap.
  Perf subagent task assigned (see below).
- `eeee72a` batch: (a) TypeTable::canonicalise_anonymous — anonymous
  sub-types equal to a named tabled type become references to it
  (add_typedefs reduces equivalence classes, axis-types.w:1024-1051);
  fixes the `(vec,int,orbit_data)` echo. (b) forget of a TYPE name
  reports "forgotten". (c) operator casts match instances with
  Type::equals (tabled == expansion): maximal@KGPElt.
- Residual known hole: forget removes a type name from the lexer set
  but not from TypeTable (no remove API); reusing the name would still
  resolve the old binding. No corpus evidence yet.
- gdb sampling recipe that works: `cd dir; bash -c "exec BIN < in > /dev/null 2>&1" & PID=$!; sleep N; gdb -p $PID -batch -ex "bt 22"`.
  `A && B &` backgrounds the WHOLE list as a subshell (you sample bash);
  perf symbolisation fails on the nodes; block-buffered stdout means a
  hung run leaves an empty file.
- InnerClass orbit-construction perf landed (`eeee72a` real-group part +
  `874a89f`): groups.at load 4.39s -> 0.85s (oracle 0.05s), E8
  involution_orbits 1.33s -> 0.24s, classification 1.76s -> 0.27s
  (199,952 involutions, 10 classes unchanged; E7 142->18ms, F4 9->1.7ms).
  Design: (a) `PermutationOrbits` runs the phase-1 Cayley BFS and
  canonicalize at the root-image-permutation level — decisions are
  provably identical to the matrix path (same pairings, same word), so
  representatives are rebuilt once per class by replaying the word with
  full `TwistedInvolution::new` validation; (b) phase-2 cross-closure
  probes use `PermutationKey`, the simple-root images packed into u128 —
  EXACT and injective because a root-datum involution is a linear map and
  the simple roots are a Z-basis of the root lattice (`Full(Vec<u8>)`
  fallback above rank 16); (c) `CartanClassification::build` reuses the
  partition's class order (it already IS the Atlas Cayley-BFS discovery
  order) instead of re-running the BFS; (d) CayleyCross peel/replay and
  subsystem_simple_roots run at permutation level. Gotchas for future
  agents: the u8-permutation encoding still caps at 255 roots (E8 has
  240); the permutation-level canonicalize MUST stay decision-identical
  to matrix-level `InnerClass::canonicalize` (still the public API for
  rep_context/ktype) — any change there needs both paths updated and an
  HPC differential gate.
- `ff8fb34` fix: `case` discrimination follows the merge chain AND a
  Tabled definition to the canonical union entry. Root cause: in a
  bracketed set_type group with structurally equal members
  (conjugate.at's maybe_a_mover/maybe_a_conjugator = (void|WeylElt)),
  pass-3 merge + canonicalise_anonymous leave the merged member's
  stored definition as a plain `Tabled` reference to the canonical
  entry; the discrimination check read `binding(n).definition`
  directly and saw the reference, not the Union. This single blocker
  accounted for 132/134 RUST_EVAL_FAIL in corpus 3615339 (MATCH 83).
  Minimal HPC probe that reproduced it:
  `set_type [a=(void x|int y), b=(void x|int y)]` then
  `set f(b v)=int: case v | y(v):v | x: 0 esac`.
- Remaining single-hit issue seen in 3615339: induction_sp4.at fails
  with "Runtime error at basic.at:25:58: Levi factor is not
  theta-stable" via `real_Levi(KGB(G,9))`/`real_Levi(KGB(G,7))` in
  parabolics.at — likely a real behavioral difference (upstream
  accepts that Levi). Investigate after the corpus rerun.
- Corpus 3616252 submitted at 6dff4ab (includes discrimination fix +
  the orbit-construction perf). Expect the 132 discrimination failures
  to clear and per-script times to drop ~4s; remaining gap to oracle
  (~0.05s/script vs ~0.8s) is other eager E8 work, not orbit
  construction.
- `29651e4` fix: `TypeTable::expansion` follows Tabled definition
  chains to the fully untabled structure (upstream expansion,
  axis-types.w:976-1000). One-level expansion returned another Tabled
  for a merged group member, so `equivalent()` never merged
  structurally equal group members (maybe_a_mover/maybe_a_conjugator),
  and overload add then rejected the second `set any(...)` with
  "Cannot overload" (132 corpus scripts, corpus 3616252). Oracle probe
  confirms: `set_type [a=(void x|int y), b=(void x|int y)]` echoes BOTH
  as "(void|int)" and `set f(b v)=...` reports "Defined f: (a->int)"
  (canonical name).
- Perf status after 3616252 (incl. `874a89f`): 174/238 scripts
  improved >0.5s, median 4.26s -> 3.67s per script, but still ~97
  scripts >5x slower than the oracle (~0.1s). The orbit-construction
  fix accounts for ~0.7s; the remaining ~2.8s per-script hot spot is
  unidentified — sample with the gdb recipe again once the corpus is
  green.
- `c13b06a` batch (after corpus 3617082: EVAL_FAIL 134->9, MATCH 87,
  OUTPUT_DIFF 142): (a) set_type single-name alias echo prints the type
  AS WRITTEN — a tabled RHS shows its name ("Parabolic defined as
  KGPElt"); ALL 14 sampled OUTPUT_DIFF scripts differed only in this
  echo + the Levi cast report line. (b) op-cast `f@T`: the resulting
  function type uses the WRITTEN T (axis.w:6761-6764
  type_expr(ctype.copy(), res_t.copy())), not the stored instance type.
  (c) `return` leaked its raw a-priori type when the declared result
  was desugared to a body cast (lambda_with_result): the shared return
  cell is now seeded from an outermost body cast when undetermined
  (upstream seeds from the declared result, axis.w:313). Minimal probes
  (HPC, both binaries): `set f(int n)=(mat,[int]): let rv=null(2,0)
  then ib=null(0) in if n>0 then return(rv,ib) fi; (rv,ib)` — oracle
  accepts (vec->[int] coercion at return), we rejected.
- Corpus 3617082 benchmark: comparable 229, 222 over 5x slower; the
  per-script gap is now ~3.7s median vs oracle ~0.1s — after the orbit
  fix (~0.7s of it), ~2.8s/脚本 unknown hot spot remains. Next perf
  step: gdb-sample a slow script (e.g. springer_table_E8.at) with the
  documented recipe.
- `0ab4baa` fix: bracketed set_type members with ALL fields unnamed
  parse as a plain type expression (Alias-shaped spec). (a) The echo
  must key on the set_type FORM (bracketed vs single-name), not the
  spec kind — `set_type [ t0 = (int,int) ]` echoes "(int,int)" while
  `set_type Parabolic = KGPElt` echoes "KGPElt". Keying on spec kind
  (c13b06a) regressed 42 previously-MATCH corpus scripts. (b) Pad
  binding.fields with None per component in pass 2, or whattype zips
  components against an empty list and prints "(  )".
- example.at corpus failure (3617285, at c13b06a): `set pars=theta_stable_parabolics(G)`
  for G=Sp(4,R) yielded 5 parabolics, so `pars[7]` died with
  "index 7 out of range (0<= . <5)". Root cause is the
  fundamental_(co)weight ambient-coordinate bug fixed by `a0cbcd9`
  (domain_builtins.rs): the builtins returned (co)root-basis
  coordinates; for Sp(4,R) the wrong Levi_coweight [1,2] vs [1,1]
  made has_theta_stable_Levi reject real_Levi(KGB(G,9)), halving the
  theta-stable list. No new code change needed. Verified by bisect job
  3617886 (clone /public/home/majj/atlas-rust-ex1): c13b06a release
  build reproduces the exact corpus error (exit=1), a0cbcd9 exits 0
  and `Variable P: KGPElt` binds. Per-S probe
  (/public/home/majj/ex1-probes/probe_ex1.at) now matches the oracle
  exactly: parabolics counts [11,6,4,1] for S=[], [0], [1], [0,1],
  theta-stable [4,3,2,1], total 10. NOTE: corpus 3617878 (at 0ab4baa)
  globbed only 76 scripts and did NOT include example.at — the next
  full corpus rerun should re-confirm it.
- 2i12.at corpus failure (`Runtime error at basic.at:23:61: assertion
  failed` inside `decompose(0,1,delta,B,g)`, 2i12.at:34): root cause was
  RootDatum `=` semantics, not Hecke math. Upstream compares root data as
  bare `PreRootDatum` (simple roots, simple coroots, `prefer_co`;
  prerootdata.cpp `operator==`), while our derived PartialEq on
  `RootDatumHandle` (domain_builtins.rs) also compared the lie_type and
  isogeny provenance fields. `integrality_datum` hardcodes
  `DatumIsogeny::SimplyConnected` when lattice_rank==semisimple_rank, but
  the same datum built via `root_datum(LieType, lattice)` gets
  `classify_isogeny` -> `Other`, so `w.root_datum=p.integrality_datum`
  was false and the single-argument assert in `cross@(WeylElt,Param)`
  (basic.at:1827) fired. Call chain: decompose -> generate_subspace ->
  neighbors -> ext_cross (type 2r21) -> ext_cross_special ->
  ext_cross_simple -> ext_cross_2 -> test_ext_cross -> cross(WeylElt,Param).
  Fix `bad6338`: manual `PartialEq for RootDatumHandle` comparing only
  `datum` + `prefers_coroots`. Debugging lesson: the default-message
  assert (`assert(b)` -> "assertion failed" at basic.at:23) is always one
  of the few single-argument assert sites (basic.at:1521/1524/1827/1830,
  tits.at:56/60, Wdelta.at:46, lattice.at:49/208); bisect by rebuilding
  the script's intermediate values with `prints` on both binaries. Verified
  on HPC clone /public/home/majj/atlas-rust-2i12: full 2i12.at exits 0,
  output matches oracle except the pre-existing WeylClassTable set_type
  echo formatting (`(int,int->int)` vs `((int,int)->int)`, unrelated).
  quick_check job 3617899: CHECK_DONE status=0.

## Checkpoint 2026-08-23d (corpus 3617910: one echo root cause for all 144 diffs)

- Corpus 3617910 (952b2c7, driver with output_diff snippets 3f6dc19):
  MATCH 93 / OUTPUT_DIFF 144 / RUST_EVAL_FAIL 1 (gl4H.at panicked in
  ext_block.rs — fixed by 7b6bb90) / SKIPPED_LARGE 2. Recorded in
  BENCHMARKS.md ledger (median rust/cpp 29.5x, slowest 76.8x — numbers
  predate the overload-cache perf commits 659df32/f1c5fc5).
- The output_diff_histogram collapsed all 144 OUTPUT_DIFFs to a single
  root cause: the bracketed `set_type` echo printed tuple/union arrow
  sides naked — rust `(int,int->int)` vs oracle `((int,int)->int)`.
  Mechanism: upstream `type_expr::add_typedefs` interns EVERY anonymous
  sub-type, so in the echo (`type.untabled()`, global.w:1647) a function
  arrow side has `raw_kind()==tabled`, not `tuple_type`, and keeps its
  parentheses (axis-types.w:1620-1635 prints naked only for DIRECT
  tuple/union). The single-name alias form echoes the checked type
  uninterned, so it stays naked (global.w:1390). Oracle probes on HPC:
  `set_type [ T = ((int,int->int) f) ]` echoes "(((int,int)->int))";
  same for union args, row-nested, and the result side;
  `set_type G = ((int,int->int) f, int x)` echoes naked.
- Fix 9a33da9: `write_arrow_side` in types.rs keeps parens for
  tuple/union sides when `void_arrow` (the bracketed-echo mode, used
  only by `display_in_set_type`). Unit tests added from the probes.
- Verification in flight: quick_check 3617952, corpus 3617953 at
  9a33da9 (also carries 659df32+f1c5fc5 overload-cache perf and
  7b6bb90 gl4H ext_block fix — corpus 3617953 doubles as the perf
  before/after measurement).
- Also this round: agent-111 landed OverloadState merged-variants cache
  (659df32, per-name) plus precise invalidation (f1c5fc5); its own
  HPC timing verification jobs were 3617924/3617925.
- Lesson: when every OUTPUT_DIFF shows the same first-divergence line,
  fix that before dispatching per-script buckets — the histogram's job
  is exactly to expose this.
- gl4H.at corpus failure (`Runtime error at hermitian.at:30:47: parameter
  not in the common block`, triggered by `twisted_KL_sum_at_s(p)` in the
  c-form formula for p=trivial(quasicompact_form(GL(8,R))) and
  trivial(SL(4,H))): TWO root causes, both fixed.
  (1) `KgbGraph::twisted_with_destination` (kgb_graph.rs) looked up the
  delta-twisted Tits element by RAW mod-2 torus bits, but upstream
  `KGB::lookup` first REDUCES the candidate against the renamed
  involution's mod space (`ic.involution_table().reduce(a)`,
  kgb.cpp:716-719) before comparing against the stored (reduced) fiber
  representatives. Where the twisted bits are only mod-space-equivalent to
  the stored ones we returned UndefKGB: `twist(KGB(G,2))` for
  G=quasicompact_form(GL(4,R)) gave #4294967295 vs oracle #2 (twisted
  bits 0101 reduce to the stored 1010 mod im(1+theta_2)). This poisoned
  every consumer: the ext_block fixed-point test
  (`transformed_twisted`/`ExtBlock::build`) dropped the delta-fixed seed,
  so the fixed fiber of trivial(GL(8,R))'s common block (25 elements,
  x=0,1,2,3,4,8,...) lacked the seed x=5. Fix `7b6bb90`: delegate to the
  already-verified reducing `KgbGraph::lookup`. Side effect also fixed:
  `extended_block(p,delta)` for the same p previously PANICKED
  (usize::MAX fiber index in the wrapper's `signed()`); now matches the
  oracle (25-element fiber). Debugging lesson: on panic, stdout buffered
  by the CLI is LOST — a probe printing progress before the failing call
  shows nothing; bisect with separate one-call probe files, and note the
  "parameter not in the common block" prose has three emit sites
  (domain_builtins.rs common-block seed search x2, formerly
  twisted_block_index x1) — the twisted one was a symptom, not the cause.
  (2) `with_integral_block` (domain_builtins.rs) rebuilt the FULL dual
  block for `IntegralBlockScope::Full` and searched it for the seed, but
  upstream `Rep_table::twisted_KL_column_at_s`/`twisted_deformation`
  always use `Rep_table::lookup` — the seed-rooted Bruhat interval-below
  common block — even at a full integral subsystem (repr.cpp:2378-2382,
  2605-2606); the full block's y-classes are propagated from its own
  generator, so a delta-fixed seed can sit on a non-fixed y-class there
  (and its `block.length` signs differ from the common block's). Fix
  `305d3a9`: Full now shares the lookup path with ProperSubsystem,
  matching the crate's own recursive twisted_deformation driver
  (deform.rs:1102); the homegrown `twisted_block_index` helper is gone.
  Verified on HPC clone /public/home/majj/atlas-rust-gl4h (release build
  of 7b6bb90): twisted_KL_sum_at_s matches the oracle line-for-line for
  trivial(quasicompact_form(GL(n,R))) n=2,4,6,8 (1,3,7,25 terms) and
  trivial(SL(4,H)) (130 lines), KL_sum_at_s(GL(8)) still matches (105
  lines), and gl4H.at lines 1-8 (`twisted_c_form_irreducible_as_sum_of_
  standards` for both forms, 105 terms each) match the oracle exactly
  (212 parameter lines). quick_check job 3617949 (compile gate for the
  pair of fixes). Probes live in /public/home/majj/gl4h-probes/ (NOT in
  atlas-scripts). NOTE: corpus rerun should re-confirm gl4H.at end to
  end; the extended_block fix may repair other scripts too.

## Checkpoint 2026-08-24a (corpus 238/238 individually green)

- Corpus 3617953 (9a33da9): MATCH 236/240, OUTPUT_DIFF 2, fat 3617912
  both large E8 cell scripts MATCH. Recorded in BENCHMARKS.md.
- exceptionalData.at: single-name set_type must say "redefined as" when
  the name was already a defined type (global.w:1384-1391; the bracketed
  form always prints "defined as", global.w:1635-1647). Fixed in b28664c;
  MATCH confirmed by targeted corpus 3621033.
- example.at was TWO printing bugs, both in the value-printing paths:
  1. Closures nested in containers printed the bare Display head
     "Function defined" instead of the multi-line closure_value::print
     (axis.w:3254-3271). Fixed by a recursive value_string in typed.rs
     used by render_value/trace_value_string AND by
     print/prints/to_string/error (b28664c + 5e66582; DomainValue leaves
     can't hold closures, baf6820).
  2. Projector applications printed as `((Function defined)(f))(mu)`
     because analysis inlines the set_type projector closure as a
     Denotation in a FunctionCall. Upstream keeps a projector_call node
     (axis.w:4495-4532) that prints postfix `argument.field` with the
     CALL-SITE name (build_call(name) — an aliased projector prints its
     alias). Fixed in typed_expression_print (bb8e539): Denotation of a
     closure whose body is TupleProject prints `arg.<trace-name part>`.
     MATCH confirmed by targeted corpus 3621110.
- Infrastructure: quick_check now also runs `cargo test --workspace`
  (bae6d65) with correct status capture and a kept full log (130e61b),
  and uses a PER-JOB worktree (5e66582) — job 3621023's cleanup deleted
  3621032's shared checkout mid-test ("couldn't read build.rs").
  Full cargo test currently red: 2 stale syntax shape assertions from
  075c5e8's do_expr reframing (break_accepts_an_integer_level,
  parses_loop_and_break_forms) and one REAL stack overflow in
  typed::tests::let_function_sugar_and_recursion_evaluate (SIGABRT,
  reproduced with per-job worktree in 3621075). Diagnosis job 3621118
  runs the two stale tests solo for exact shapes and the overflow test
  with RUST_MIN_STACK=32M to distinguish depth from runaway recursion.

## Recovery runbook 2026-08-24b (HPC login node outage)

- Login node 10.26.14.64 was unreachable (100% ping loss, ssh timeout)
  from ~01:54+08. Commits a0c733a/7fce0f4 were pushed to GitHub but NOT
  rsynced to HPC; quick_check for them never ran.
- On reconnect, in order:
  1. `rsync -az --delete .git/ ikkemhpc:/public/home/majj/atlas-rust/.git/`
     then `ssh ikkemhpc 'cd /public/home/majj/atlas-rust && git reset --hard HEAD && sbatch hpc/quick_check.sbatch'`
  2. Let agent-111 (overload perf) land its work; at a quiet point rerun
     the FULL corpus: `sbatch --export=ALL,TIMEOUT=300 hpc/script_corpus.sbatch`
     and confirm 240/240 MATCH on one HEAD.
  3. Benchmark workloads (self-contained, no includes) via the corpus
     driver — no new sbatch needed:
     `sbatch --export=ALL,TIMEOUT=600 hpc/script_corpus.sbatch '/public/home/majj/atlas-rust/hpc/workloads/workload_*.atlas'`
     Report lands in results/<commit>/<jobid>/script_corpus_report.json
     with seconds/maxrss for both binaries; record into BENCHMARKS.md.

## Perf work 2026-08-24c (overload resolution + Weyl sharing; agent-111)

- Root cause of the ~4.6s/script rust-vs-oracle gap (gdb + perf -F 999
  with force-frame-pointers on HPC; plain `-g` unwinding breaks through
  Rust's omitted frame pointers, use `RUSTFLAGS="-C
  force-frame-pointers=yes"` builds for profiling):
  `merged_variants` rebuilt the full ordered variant list of a name on
  EVERY call site (twice: the call-head dispatch emptiness check and
  convert_overload_application itself) — deep clones of all builtin
  variants plus `is_close` insert-position scans of every user variant
  against every builtin variant.
- Fixes (all local `cargo check`+focused tests green, HPC quick_check
  compile gate green):
  - `659df32` + `f1c5fc5`: cache merged variants per name in
    OverloadState; invalidate ONLY the mutated name (a corpus script
    interleaves hundreds of `set` commands with call sites of unrelated
    names, so full-cache clears kept every name cold). `add_user` must
    build its pre-mutation view with the uncached builder — the caching
    wrapper would re-populate the just-invalidated entry with the stale
    list (this broke 5 overload unit tests until fixed).
  - `ff23515`: (a) coercion_between/row_coercion now gate the 29-entry
    table scan by top-level shape (same() requires equal gates, so the
    filter only prunes certain non-matches; first-match order
    unchanged). (b) `weyl_datum_shared` caches
    (Arc<RootSystem>, Arc<WeylInterface>) per datum content —
    build_weyl_context and 12 direct RootSystem::enumerate sites
    re-enumerated per builtin call while scripts build Weyl elements per
    group element in load-time loops; upstream builds the WeylGroup once
    per inner class (innerclass.cpp). WeylEltContext fields are Arc now.
- HPC seconds (perf_build_time job 3621094 @ e347894; before = 3617892):
  GKfast 8.72→1.84, generic_degrees 8.77→1.85, test_braid 8.79→1.84,
  class_tables 5.69→1.24, basic.at 0.17→0.08. Corpus 3617953 (9a33da9):
  MATCH 236/240, median rust/cpp ratio 29.5x→12.97x.
- Remaining hot spots (fp profile of test_braid.at, pre-ff23515):
  CartanClassification::build 22% via build_inner_class (load-time
  distinct inner classes — classification_cached MISSES are legitimate,
  distinct fingerprints; the gap is algorithmic inside
  involution_orbits/twisted_conjugacy_partition, ~18% self),
  TypedContext::execute 10.8% self, memcmp 9% under execute_tokens
  (lexer/parser lane), malloc/free ~13%. The <1s/script target needs the
  atlas-real-group classification speedup next.
- KNOWN UNRELATED RED: quick_check's new `cargo test --workspace` phase
  fails on typed::tests::let_function_sugar_and_recursion_evaluate
  (stack overflow, SIGABRT) — reproduced locally; reverting typed.rs /
  coercions.rs / domain_builtins.rs to pre-perf commits does NOT cure
  it, so it belongs to the do_expr/evaluator lane (see 2026-08-24a
  checkpoint, diagnosis job 3621118), not to the perf commits.

## Perf finding 2026-08-24b (unipotent_representations_exceptional.at, job 3622755)

gdb sampling (20/20 samples, 3s interval) shows the 77s outlier is spent in
KGB graph construction, called per KGB print from the script:

  malachite Natural::mul <- integer_lattice::bounded_linear_combination
  <- IntegerMatrix::add_row_multiple <- saturated_kernel
  <- negative_coweight_eigenspace <- involution_table::push_record
  <- InvolutionTable::add_cartan <- KgbGraph::build
  <- RealFormContext::build_kgb <- RealFormContext::kgb
  <- call_with_printed (per-call!)

Two attack lines:
1. CACHE: RealFormContext::kgb appears to rebuild the KGB graph on every
   call; upstream caches per RealForm. A per-real-form KGB cache in the
   evaluator context should collapse repeated builds.
2. ARITHMETIC: saturated_kernel/add_row_multiple runs big-integer
   (malachite) multiplies in the inner loop; upstream lattice code keeps
   machine-int fast paths. Consider i64 fast path with overflow fallback.

## Perf finding 2026-08-24c (full-corpus per-script analysis, job 3622339 @ fc85095)

From `results/fc85095.../3622339/script_corpus_report.json` (238 MATCH):

- The mid-tier offenders cluster tightly: gl4H/test_non_distinguished/
  example/test_K/all/speh all sit at rust≈4.4-5.2s vs cpp≈0.38-0.42s.
  These scripts all build groups and compute K types, i.e. KGB-dense work.
- `E8_small_block_cell_parameter_numbers.at` is the worst ratio (76.7x,
  rust 3.91s vs cpp 0.05s): E8 KGB graph rebuilt per access, same root cause.
- Prediction: the per-real-form KGB cache (agent-114) should collapse this
  whole mid-tier cluster, not just the unipotent outlier. Verify on the next
  full-corpus rerun by checking the 4.4s cluster drops toward cpp+ε.
- SUPERSEDED by finding 2026-08-24d below: the per-real-form KGB cache
  already exists and works (probe 3622804: exactly 26 builds for 26 groups).
  The mid-tier cluster cost is one-time KGB builds + typed-evaluator
  overhead, not rebuilds.
- Residual check after the cache lands: if a fixed ~0.5-2s gap remains on
  tiny scripts (2i12.at was 2.19s vs 0.28s), profile interpreter/lib-load
  startup next; that is a different hotspot than KGB.

## Perf finding 2026-08-24d — KGB rebuild hypothesis DISPROVEN (probe job 3622804)

The `ATLAS_PROBE_KGB=1` build counter (7c325cf) on
unipotent_representations_exceptional.at prints exactly 26 `KGB_PROBE build`
lines for the script's 26 groups: every real form's KGB pipeline is built
ONCE. The per-real-form cache (RealFormContext::kgb Mutex + canonical_forms
Weak cache, domain_builtins.rs:283-305/2240-2301) already works; the repeated
`external/internal` pairs in the probe are distinct inner classes (E6 sc/ad,
E7 sc/ad share external ids). Do NOT build a per-real-form KGB cache — the
77s was the sum of 26 one-time builds, dominated by E7/E8 graph sizes.

What the 2026-08-24b stack really showed: one long serial
`InvolutionTable::add_cartan` phase (push_record per twisted involution,
each paying a malachite saturated_kernel) plus the parallel KGB BFS
(from_par_iter). Post-fast-path sampling (job 3622836) shows 7/25 samples
parked in rayon LockLatch (main thread waiting on the parallel BFS — normal,
NOT starvation: RAYON_NUM_THREADS=4 is unchanged at 66.2s, threads=1 is
WORSE at 81.0s; jobs 3622856/3622857), 3/25 in push_record, the rest
scattered across typed-evaluator script overhead and malloc.

Landed against this (all verified MATCH on the unipotent script):
- 460370e i64/i128 fast path in bounded_linear_combination: 68.0 -> 65.7s.
- 9dc4b37 HashMap intern index (dedup-only, never iterated) + ModTwoVector
  inline words (SmallVec<[u64;2]>, RANK_MAX=32 always fits): with f1a1c18
  and 6b5df6a in the same build, 65.7 -> 60.6s.

Residual gap to the oracle is 10.2x (60.6s vs 5.9s). Next levers, in
suspected impact order:
1. Serial add_cartan: per (record, generator) two WeylElement::multiply
   calls, each 2 Vec allocs + O(#roots) (E8: 240 roots x ~10^6 edges).
2. Parallel BFS per-element costs: cross_pregated/cayley_pregated +
   per-element `Vec::with_capacity(rank)` result buffers.
3. Per-inner-class shared InvolutionTable across real forms (upstream
   shares Cartan classes per ComplexReductiveGroup): ~3x table work for
   E8's 3 forms, but invasive (KgbBundle.table is Arc<InvolutionTable> and
   KgbGraph::build takes &mut table).

## Perf work 2026-08-24e (completion refresh + E8 classification wall; agent-115)

Baseline for this round: perf_build_time 3622331-ish numbers, GKfast /
generic_degrees / test_braid ~1.8s, class_tables 1.22s @ 24a0d1d
(pre-LTO). All jobs below run at the pushed HEAD named in the commit list.

- `b1daede`: execute() rebuilt the completion-candidate snapshot (294
  startup names + every session name, each String-cloned) at the top of
  EVERY command — O(commands x names) over long include chains. Now
  incremental: append-only push on new definitions, dirty-flag rebuild only
  after forget/revive/set_type. fp profile: TypedContext::execute self
  13.65% -> 0.58%. The candidates feed ONLY the readline_completions
  builtin (typed.rs:6108), so batch loads never read them.
- `ed215e0` + `5c424f1`: ATLAS_PROBE_CLASSIFICATION=1 (env-gated, keep it)
  prints hit/miss + rank/roots/classes/members/ms per
  classification_cached call. test_braid.at (probe 3622847): 12 miss / 2
  hit, ALL DISTINCT fingerprints — per-fingerprint InnerClassContext
  caching is useless here, and it would also break the white-box isolation
  contract of
  domain_builtins::tests::real_form_owner_caches_are_isolated_for_custom_and_distinct_inner_contexts
  (two identical inner_class calls must NOT share an Arc). The load-time
  wall is TWO split-E8 builds (primal+dual, rank 8, 240 roots, 199,952
  members each, ~223ms apiece) + 2xE7 (16ms) + 4xE6-size (~3ms).
- `f1a1c18`: ClassOrbit member permutations stored flat (one Vec<u8>,
  stride = root count) instead of one 240-byte heap Vec per member.
  test_braid 1.09 -> 1.03s (less than hoped: the storage was not the
  dominant cost inside the E8 cross-action closure; the seen-set probes
  and key-building loop are).
- `6b5df6a`: build_inner_class_context builds primal and dual
  Cartan classifications on two std::thread::scope threads (independent
  pure computations, same content-keyed cache; primal error precedence
  preserved). test_braid 1.03 -> 0.79s.
- HPC seconds (perf_build_time 3622928 @ 6b5df6a): GKfast 0.81,
  generic_degrees 0.80, test_braid 0.79, class_tables 0.66, basic.at 0.04
  (oracle ~0.12-0.18s; ratios now ~4.5-5.5x, all five under 1s).
  quick_check green at both stops: 3622871 (f1a1c18), 3622906 (6b5df6a).
  Full-corpus differential submitted as 3622952 @ 6b5df6a — compare its
  median ratio against 3622339's 10.25x when it lands.
- Remaining typed-side hot spots (perf-fp2 @ f1a1c18, test_braid):
  coercion_between 5.61% self (millions of tiny gated table scans from
  is_close + convert paths; no cheap key for memoization since types are
  not interned), convert_overload_application 5.64% cum,
  OverloadState::add_user 3.27% (inherent per-variant is_close scan),
  SourceText::position 1.95% self via lex::token (2 span positions per
  token; partition_point + per-line char count — a line-index or deferred
  spans would kill it, but the lexer lane has other agents active),
  parser __reduce 3.09%.
- Tooling notes: perf annotate came back EMPTY under the fat-LTO
  codegen-units=1 release profile (no debug info); add `debug = true` to
  [profile.release] locally if instruction-level attribution is needed.
  probe_classification.sbatch / perf_fp*.sbatch / perf_build_time.sbatch
  all recreate /public/home/majj/atlas-rust-perf — never run them
  concurrently; sequence: perf_build_time, then perf_fp2, then probes.


## Ops note 2026-08-24f — corpus glob paths

`script_corpus.sbatch <globs>` matches against the ORACLE's script dir
(`/public/home/majj/atlasofliegroups-4d3e9449/atlas-scripts/`), not
`/public/home/majj/atlas-scripts` (does not exist). A bad glob silently
yields `corpus: 0 scripts` (jobs 3623642, 3623661 wasted a build each).
Always use absolute paths under the oracle repo.

## Perf finding 2026-08-24g — E8_small_block regression @ 9b6f20f (job 3623687)

unipotent improved to 41.5s but E8_small_block_cell_parameter_numbers.at
regressed 3.91s -> 15.97s between 907dcd4 and 9b6f20f (commits in window:
1cca878 BFS result-row inlining + Tits hasher, 06b85d7 scratch-buffer root
classification, 9b6f20f pointer-equality gate in WeylAction::compose).
Output still MATCH; the slowdown is in the single big E8 build. Bisect by
re-running that script at each commit; likely candidate: 1cca878's inline
SmallVec rows copying on E8's wide BFS frontiers, or the pointer-eq gate
adding a branch that almost never hits for E8.

## Perf work 2026-08-24h — typed-side coercion machinery (agent-116)

Targets from perf-fp2 @ 6b5df6a (test_braid): coercion_between 5.59% self,
OverloadState::add_user 3.27%, SourceText::position 1.45%.

- `4ed8fe0` coercions.rs: (a) the 29-entry coercion table is now bucketed
  ONCE by (from-gate, to-gate) — 25 gate codes (20 prims + row/tuple/
  union/function/undetermined), registration order preserved inside each
  bucket, so the first-match result is provably the linear scan's; the
  None-gate (self-referential set_type placeholder) fallback still scans
  all. (b) `is_close` now follows upstream's control flow
  (axis-types.w:3258-3285): the table is consulted only when a primitive
  is involved; aggregate pairs go straight to the componentwise
  recursion. Equivalence argument (verified entry-by-entry against the
  fixed table): every row-to-row entry derives from a component coercion
  the recursion also finds ([int]->[rat] iff int->rat; [vec]<->[[int]]
  iff vec<->[int]; likewise the ratvec/[rat] entries), no tuple-to-tuple
  or function entries exist, and gate-mismatched pairs can never satisfy
  `same`. New unit tests pin the bucket index against a naive scan over a
  22-type cross product and pin is_close bits on aggregate pairs.
- `9e81bc3` source.rs: `SourceText::span` shares one line lookup and one
  prefix scan for both endpoints (lexer takes a span per token); columns
  count non-continuation bytes instead of decoding chars; same-line end
  columns extend the start column. Stored byte offsets and positions are
  unchanged (sweep test compares span() against position() pairs over a
  multiline Unicode text).
- Verification: quick_check 3623641 green (862 tests; the job's FAILED
  state was only the final `git worktree remove` — rsync of .git had
  wiped the HPC-side worktree registrations; use
  `rsync -a --exclude=/worktrees .git` from now on). Targeted corpus
  3623672 @ c6f453c: test_braid/class_tables/GKfast/generic_degrees/
  example/all all MATCH. Full corpus 3623704 @ 06b85d7: MATCH 238/238 +
  2 SKIPPED_LARGE, median rust/cpp 4.42x -> 4.09x, over_5x 34 -> 26,
  within_2x 41 -> 49 (benchmark rows in BENCHMARKS.md; that commit stack
  also carries agent-115's BFS/classification chain, which owns the
  unipotent 59.7 -> 41.5s win).
- Memoization considered and REJECTED for now: types are not interned,
  so a coercion memo needs structural digests + table-generation
  invalidation; with the gate bucket the lookup is already O(bucket) and
  deterministic — a hashmap probe would cost about the same while adding
  collision/invalidation risk.
- Remaining typed-side hot spots (unchanged, not mine or not clean):
  parser __reduce ~3% (generated lalrpop code — do not hand-edit, rule
  4), lexer's per-token span construction (1.45% -> ~halved by 9e81bc3;
  fully killing it needs deferred spans in the lexer lane).

## E8_small_block regression: correction and bisect (2026-08-24i)

- **The script has NO KGB build.** `E8_small_block_cell_parameter_numbers.at`
  is a single-line (~430KB) `set cells_small=[[...]]` giant nested-int
  literal assignment. agent-116's note "the cost is in the single E8 KGB
  build" is a wrong premise — the hot path is lex/parse/eval of a huge
  literal, i.e. the typed/lexer lane, not real-group.
- Corrected bisect window: 3.91s @ 907dcd4 (bisect job 3623712, cu054,
  re-measured) vs 15.97s @ 06b85d7 (corpus 3623704 report, run AT 06b85d7).
  So the culprit is <= 06b85d7: candidates {4ed8fe0, 9e81bc3, 1cca878}.
  9b6f20f is excluded (landed after the 15.97s measurement); 06b85d7
  itself only touches real-group code and is implausible for a
  parse-bound script.
- Prime suspect: `9e81bc3` SourceText::span — for a single-line file every
  per-token span still pays an O(line-prefix) column scan, i.e. O(n^2) over
  70k tokens; check whether the rewrite pessimized it (old code did two
  `chars().count()` prefix scans; new does one byte-filter prefix scan +
  short extension — should be faster, so if it regressed something else is
  going on, e.g. `adjusted()` or an accidental extra scan).
- Second suspect: `4ed8fe0` is_close restructure (per-element recursion on
  row/row pairs during literal type-check could have changed cost shape).
- Bisect jobs in flight: 3623976 (worktree atlas-rust-bisect; 1cca878,
  06b85d7, 9b6f20f) and 3623991 (worktree atlas-rust-bisect3; 4ed8fe0,
  9e81bc3). Scripts: `bisect2.sbatch` / `bisect3.sbatch` in those
  worktrees; output `bisect2-3623976.out` / `bisect3-3623991.out`.
  Each point does `git checkout -f + reset --hard + rev-parse HEAD guard`
  before building (a failed checkout previously benchmarked the wrong
  binary — see failed 3623959).
- Ops note: `rsync -a --delete .git` to HPC WIPES HPC-side worktree
  registrations (broke bisect 3623959 mid-run with "Could not read
  <sha>"). Always use `--exclude=/worktrees` (agent-116's workaround).
  If a worktree's gitdir link is broken, re-register or re-add it before
  submitting bisect jobs.

## Perf work 2026-08-24j (involution-table BFS + record classification; agent-117)

Lever 1+2 of the 2026-08-24d list, all verified MATCH on the 5-script KGB
corpus (quick_check 3623634/3623650/3623964 green; corpus 3623635, 3623687,
3623965):

- `907dcd4` allocation-free cross-edge probes: `InvolutionTable`'s dedup
  index moved from `BTreeMap<Vec<RootId>>` (240-entry ordered-map key chase
  per probe) to a hash map on an INJECTIVE packed key — simple-root images
  only, u128 for rank <= 16 and <= 256 roots (the inner_class
  `PermutationKey` layout), full permutation otherwise (foreign-length keys
  go to the Full variant, preserving the provenance test). The BFS dedup
  probe reads only the rank simple positions of `s*w*twist(s)`; the full
  neighbor permutation is composed (one buffer, no temporary WeylElement
  products) only for NEW involutions (1/rank of edges).
  `InvolutionTable::cayley` probes the packed key directly instead of
  materializing `s*w` per call (hot from the KGB BFS's cayley_pregated).
  unipotent 59.7 -> 52.7s.
- `1cca878` KGB BFS phase-1 rows as `SmallVec<[_; 8]>` (rank <= 8 stays
  inline) + FxHash-style hasher for the Tits intern map (dedup-only).
- `06b85d7` scratch-buffer root classification in `RootInvolutionData::new`
  (was 3 Vec allocs per root per record) + `RootSystem::id_of_slice`.
- `9b6f20f` Arc ptr-eq fast path in `WeylAction::compose`'s datum gate
  (compose chains share the same Arc; contents compare unchanged).
  unipotent 52.7 -> 41.5s (both stops).
- `9e74504` permutation-level record classification: post-907dcd4 sampling
  (3623688) showed 8/20 main-thread samples in `RootInvolutionData::new`
  (per record: 240 roots x 2 checked-i128 matrix applies + binary search).
  `push_record` now composes `w_perm[delta_perm[r]]` — equal to the
  composed theta matrix action by definition — via
  `TwistedInvolution::new_from_root_images` /
  `RootInvolutionData::from_images`, with the Real/Imaginary test reading a
  precomputed `RootSystem::negatives` table. The theta matrices and
  `LatticeInvolution` validation are unchanged; only the per-root
  re-derivation (guaranteed by the distinguished/Weyl factors) is skipped.
  `RootInvolutionData::new` keeps full validation for all other callers.
  unipotent 41.5 -> 22.8s (ratio 3.88x vs cpp 5.88s).

Post-9e74504 profile (gdb main-thread sample 3623966, 15 samples): 8/15
parked in rayon LockLatch (parallel KGB BFS worker wait — the workers' own
cost is invisible to main-thread sampling; use perf on workers next), the
serial add_cartan remnant scattered: subsystem_simple_roots 2/15 (O(P^2)
decomposability probes per record x2 kinds, already hash-set based),
from_images 2/15, push_record self 2/15, saturated_kernel /
verify_annihilation 2/15, malloc noise. No single dominant serial frame
left.

Remaining levers (updated):
1. Parallel KGB BFS worker-side cost (cross_pregated/cayley_pregated +
   phase-2 serial intern): sample a WORKER thread (gdb `thread apply all
   bt` or perf) before choosing.
2. Per-record memory: each InvolutionRecord owns a full BasedRootDatum
   clone inside LatticeInvolution (root_involution.involution.datum) plus
   two 240xusize WeylElement vectors; unipotent rust_maxrss 3.7GB vs cpp
   881MB. Arc-ing the datum in LatticeInvolution or shrinking RootId to
   u32 are both broad but mechanical; expect both RSS and allocator wins.
3. subsystem_simple_roots O(P^2) per record, if worker sampling clears.
4. Lever 3 (shared InvolutionTable per inner class) is DEFERRED with a
   concrete hazard: a shared table's orbit-slice order depends on which
   form builds first (form A {0,2,3} then form B adds 1 AFTER 3), so
   absolute InvolutionIds — and any TitsElement ordering that reads them —
   become build-order-dependent. Current per-form fresh tables match the
   oracle; a shared table must reproduce upstream's exact add order under
   lazy per-form demand, which is fragile. Not worth it while levers 1-2
   remain.

### Bisect conclusion (2026-08-24i, final)

- bisect2 3623976 final: 1cca878=15.96s, 06b85d7=15.98s, 9b6f20f=15.97s.
  bisect3 3623991: 4ed8fe0=3.91s. Since 9e81bc3 is the only commit between
  4ed8fe0 and 1cca878, the culprit is CONFIRMED: `9e81bc3`
  (bisect3's direct measurement of 9e81bc3 = 15.97s sealed it)
  (SourceText::span byte-filter column scan). Mechanism: per-token span on
  a single-line 430KB file is O(n^2) either way, but the old
  `chars().count()` prefix scan was auto-vectorized (~3.9s) while the new
  `iter().filter(b & 0xC0 != 0x80).count()` is scalar (~16s).
- Real-group trio (1cca878/06b85d7/9b6f20f) fully EXONERATED — all their
  numbers match the 15.97s they inherited from 9e81bc3.
- Fix dispatched to agent-116: incremental line/column cursor in the lexer
  (offset advances monotonically, so per-token span becomes amortized O(1);
  positions must stay byte-identical, pinned by the sweep test). Target:
  E8_small_block well under the 3.91s baseline, not just back to it.

### Startup-overhead hypothesis disproven (2026-08-24i)

Corpus 3623704 per-script numbers: the fastest scripts run in 5ms wall
(cyclotomic_field_bracket.at rust=0.005s vs cpp=0.007s), so the Rust CLI
has NO meaningful fixed startup/prelude cost. The corpus median (0.647s
rust, delta vs cpp ~0.5s) is uniform interpreter-throughput gap, not
startup. Implication for future perf work: do not chase startup; profile
mid-size scripts (e.g. class_tables.at 0.64s) for spread-out evaluator
cost (Rc<Value> traffic, overload resolution, parser reduce). The
per-script tail above 5x is otherwise all small scripts at 5-7x plus
E8_small_block (being fixed by the lexer cursor) and unipotent (agent-115
lane, 59.7 -> 22.8s at 7231e4c).
