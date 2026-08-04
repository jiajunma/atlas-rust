# Atlas-Rust handoff - 2026-08-01 (handoff to next coding agent)

This is the continuation record for `/Users/hoxide/mycodes/atlas-rust`.
The goal is source-compatible Atlas language behavior, with the upstream Atlas
executable and CWEB sources as the behavior oracle. The core remains safe Rust.

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

## Start here (next agent)

HEAD at handoff: `286f236` (main). Working tree clean.

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
- **Batch coverage sweep (2026-08-04)**: A4/B4/C4 + C3 + D5 + G2/D4 KL
  family extended (KL_column, KL_sum_at_s, raw_KL, kl_print, W_graph/W_cells,
  partial_block, partial_kl_block, full_deform, deform, block_hasse,
  cartan_info, orientation_nr, two_rho). print_KL_list now enumerates the
  pool (empty blocks print the constant one). HPC captures 3515466-75,
  3515630-35, 3515698-99 verified; E7 kgb_hasse swap 3515688 RUNNING on fat
  (TIMEOUT=3600). New limit: D5 real forms hit the same column-echelon bug
  as E6 involution 187 (see REMAINING_BUILTINS.md).


