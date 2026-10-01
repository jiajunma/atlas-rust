# atlas-rust agent guide

Rust reimplementation of the [Atlas of Lie Groups](https://github.com/jeffreyadams/atlasofliegroups).
The compatibility target is the Atlas language and its observable behavior,
not a source-level C++ translation.

## Verified local Kimi delegation — 2026-10-01

The user explicitly authorized local verification of the Kimi integration.
Kimi is installed and logged in **on this local machine**; use that installation
and account. Do not move its credentials to HPC. This exception covers Kimi
invocation, task delivery and process lifecycle checks only. Atlas builds,
program tests, fixtures, mathematical verification and benchmarks remain HPC-only.

The verified route is **Codex coordinator -> local Kimi CLI subprocess ->
reviewed handoff**, not a Kimi model argument to Codex's native `spawn_agent`.
Actual checks passed with `/home/hoxide/.kimi-code/bin/kimi` **2.1.1**, model
alias `kimi-code/k3-256k`: response, Read/Edit programming task, exact-session
continuation, startup and in-flight cancellation, timeout, forced-kill fallback,
and intentional startup-error reporting. See
[`docs/slices/kimi_runtime_validation_2026-10-01.md`](docs/slices/kimi_runtime_validation_2026-10-01.md)
for raw evidence and limits. Recheck version/help after an upgrade; old Python
`kimi-cli` flags and YAML agent definitions are not this installed interface.

Use the verified Linux runner from the repository root (replace placeholders):

```sh
python3 tools/kimi_subagent.py \
  --model kimi-code/k3-256k \
  --agent-file .agents/kimi/editor.md \
  --prompt-file /absolute/path/to/frozen-task.txt \
  --work-dir /absolute/path/to/existing/workspace \
  --owned-file relative/path/to/existing-file \
  --output-dir /absolute/path/to/new/task-evidence \
  --timeout 180 --max-steps 12
```

- Give Kimi one bounded routine task, exact read/edit scope, necessary context,
  deliverable, exclusions and a no-local-program-test rule. It does not inherit
  this Codex conversation. Freeze the prompt before calling; the runner records
  the exact prompt/profile, hashes, model, owned files and deadline before the
  model-bearing launch. Capture failures as well as successes.
- `.agents/kimi/editor.md` permits only `Read` and `Edit`; it excludes Shell,
  builds, network tools and nested delegation. `.agents/kimi/probe.md` has no
  tools and can return proposals from supplied text. These are tool restrictions,
  **not filesystem sandboxes**. Keep file ownership explicit, preserve user
  changes and inspect all output/diffs. Do not grant mathematical acceptance,
  job submission, commit/push, secrets or destructive work to Kimi.
- The runner uses `-p` and `--output-format stream-json`. Never combine `-p`
  with `--plan`, `--yolo` or `--auto`: `-p` already uses automatic permissions.
  A plain unrestricted `kimi -p` can modify files and execute commands.
- Parse stdout as JSONL containing `meta`, `assistant` and `tool` messages.
  A trailing `meta` record is not the answer. The `session.resume_hint` record
  contains the actual session ID. To continue, use the same work directory and
  replace `--agent-file ...` with `--session <actual-ID>`; never pass both.
  Prefer the explicit ID to `--continue`, which may select another task.
- Leave the existing Kimi data root/authentication in place. `KIMI_CODE_HOME`
  relocates configuration and credentials as well as logs; an empty `/tmp`
  root is not a fix for storage errors. No credentials belong in task evidence.
  Use a new evidence directory per invocation; the runner refuses overwrite.
- The runner sets finite step/attempt limits, disables infinite request retry
  and auto-update, closes stdin, separates stdout/stderr and starts Kimi in its
  own process group. **Stop the recorded `wrapper_pid` in `process.json` with
  SIGINT or SIGTERM**. It sends SIGTERM to its own Kimi group, waits the grace
  period (default5s), escalates to SIGKILL if necessary, reaps the child and
  records remaining live group members. Never use broad `pkill kimi`.
- Runner exit codes: **0** CLI completed, **124** deadline, **130** SIGINT,
  **143** SIGTERM, **1** other failure. Inspect `result.json`, raw streams and
  actual files: exit0 alone is not task acceptance. Interruption does not roll
  back edits. Review partial files and the exact session before any retry.
- Do not SIGKILL the wrapper itself: no process can execute its cleanup after
  SIGKILL. The verified cleanup covers its process group, not independently
  detached daemons. Do not enable Shell/background delegation on this evidence.
  The coordinator owns the tool-session handle and must collect/reap completion.

Every actual invocation needs its model/session, prompt scope, files touched,
result, independent review, rejected suggestions and lesson in the task report
and handoff. This integration proof is not a mathematical or generated-program
test pass. Apply project changes only after review and run their required HPC
gates before committing/pushing them. No new worktree is authorized.

## Verified bidirectional Kimi interaction — 2026-10-01

For task discussion, clarification and follow-up, use the tested local ACP
client `tools/kimi_acp_session.py`. Bootstrap a new session with the earlier
CLI runner and `.agents/kimi/interactive.md`, then load that exact session ID
with the same work directory/model and a fresh evidence directory. This
profile allows **AskUserQuestion only**: Kimi proposes code as text; Codex
reviews/applies it and owns required HPC verification. For the separately
verified bounded Read/Edit route, keep using the CLI editor profile above.

The ACP client accepts JSONL `prompt`, `answer`, `cancel`, `close` commands on
stdin and emits `ready`, `update`, `question`, `turn_end`, `closed` events.
Wait for readiness before a task and for `turn_end` before another prompt.
Answer a real question by its `request_id` and required enum fields; never
invent an answer or send a competing prompt while it is waiting. Codex can
answer from known requirements; ask the user only for genuinely missing facts
or authorization. No parent Codex context is inherited automatically.

Actual checks verified question/answer, a code proposal, remembered follow-up,
busy-turn rejection, cancellation while waiting, recovery in the same session,
history reload in a new process, explicit close, EOF, deadline and SIGTERM.
All twelve Kimi process groups were clean afterward. The first interaction
attempt failed because of a historical `-p` auto-mode reminder; the final
client successfully sets ACP `default` mode and states the current mode in
each new prompt. Keep that correction. Do not assume global `--agent-file`
flags apply to the `kimi acp` subcommand.

`cancel` preserves the session: await `turn_end` with `cancelled`. The client
also settles pending question RPCs with `action: cancel`; keep this cleanup,
which eliminated the intermediate capture's shutdown warnings. `close`
disposes the live session without deleting saved history. Stop the recorded
client wrapper PID with SIGINT/SIGTERM for process-level cancellation; its
deadline/cleanup remain bounded. Exit0 means the client closed, not that all
turns succeeded. Inspect per-turn errors and the final process-group record.
The profile/client are not an OS sandbox; arbitrary sessions, command/file
tools and detached background jobs are outside this ACP proof.

See [the interaction guide and evidence](docs/slices/kimi_acp_interaction_2026-10-01.md)
for commands, state handling and limitations. ACP is not MCP: never register
plain `kimi acp` as an MCP server. An MCP tool wrapper is a documented future
option, not an installed capability. The verified route already works through
Codex terminal tools and the existing local Kimi account.

## Hard rules

1. **All testing and builds run on HPC compute nodes.** The user's latest
   mathematical-validation directive supersedes the older local-small-check
   convention. Use local execution for reading, editing, Git and hashing;
   use the login node for Git/source staging and dependency acquisition only.
   Submit Cargo builds/checks/tests, fixture/verifier execution, Atlas/CWEB
   differential jobs and benchmarks through SLURM. The original Atlas
   executable remains an HPC oracle. GitHub Actions is allowed only when its
   workflow is the requested verification environment.
2. **The original Atlas executable is the language oracle.** The upstream
   repository and its generated CWEB output define reference behavior. Do not
   infer undocumented semantics from what is convenient to implement.
3. **Differential tests precede implementation claims.** A feature is not
   supported until an HPC run compares it with the reference on accepted and
   rejected inputs and stores the result artifact.
4. **Generated files are disposable.** Do not hand-edit CWEB-generated C/C++
   or parser-generated files. Generate derived artifacts in HPC job folders.
5. **No unsafe Rust in the core.** FFI and platform-specific code must be
   isolated, reviewed, and justified.
6. **Preserve user changes.** Never reset unrelated work. Use `apply_patch` for
   hand edits and conventional commits (`feat:`, `test:`, `fix:`, `docs:`,
   `chore:`).
7. **Every Rust calculation error requires a regression test.** Whenever an
   incorrect Rust mathematical result is discovered, add a reproducing case
   to the test library (`tests/math` or the relevant `tests/fixtures` suite)
   before fixing it. Preserve the triggering input and an independently
   justified expected result; never use the faulty Rust output as the golden.
   Run the regression and original-Atlas comparison on HPC, retaining evidence
   that it fails before the fix and passes after it. If the fix is deferred,
   keep the case explicitly tracked as failing; do not remove or weaken it.

8. **At most 10 HPC jobs; increase rank only after smaller cases pass.** The
   user's 2026-09-30 directive supersedes earlier bulk-array, quota-sized
   shard and speculative dependency-chain submission practices. Submit no
   more than 10 jobs per batch, and keep this Atlas project's total outstanding
   jobs (pending/running/configuring/completing, including reviews/builds)
   at or below 10. Ten is a ceiling, not a target: prefer one focused job
   for the current smallest unverified mathematical gate. Count every array
   task as a job: `--array=0-99%2` is 100
   submitted jobs, NOT 2. Reconcile existing and uncertain submissions before
   sending more; a failed status query does not create free slots. Never
   prequeue later-rank batches behind dependencies. Start with small groups,
   inspect complete original-backed results, repair failures and verify the
   same regressions, then increase rank one step at a time. If a smaller-rank
   case fails, stop escalation until its regression passes against the oracle.
   A build/checker pass or inventory-only match does not qualify a higher mathematical
   operation for escalation. Keep the comprehensive catalog as a backlog,
   not a license to submit all cases. Do not cancel unrelated user jobs.
   New single-job launchers use `hpc/progressive_submit.py`: expanded queue
   count, shared submission lock, durable fail-closed intent, no arrays or
   dependencies. Its submission boundary rejects every legacy top-level stage;
   old stagers are read-only historical evidence, not executable launchers.
   Parent-seal job3872554 is FINAL and independently inspected. Its launcher is
   now closed. Ladder v2 job3872594 failed only its checker preflight and is
   immutable; corrected BEFORE v3 job3873400 is FINAL and accepted in its
   tests-first scope. Both ladder BEFORE/AFTER launchers are now disabled;
   AFTER v3 job3875239 is FINAL and accepted. The sole enabled launcher is the
   idempotent acceptance-index publication stager
   `stage_ladder_boundary_index.py`, bound to FINAL job3879103 and its exact
   existing stage. Reinvocation may only validate or recover that same durable
   receipt; it must not submit a duplicate or create a sibling.
   Every old direct
   `sbatch` and pre-campaign `submit_one` entry point must fail before parsing,
   copying, creating directories, writing pins or contacting SLURM.
   This does not retrofit historical bulk launchers. Every operation needs its
   own smaller-rank acceptance before escalation.
   Preserve explicit scheduler rejections separately from uncertain replies:
   Hodge R1's1CPU/8GiB request was rejected (4GiB allowed), then fresh queue
   and accounting checks proved no job existed. Its exact intent/error were
   archived before a fresh2CPU/8GiB stage was accepted. Never release an
   uncertain intent merely because a query timed out or the queue is empty.
9. **One bounded campaign tree; generated build trees are disposable.** Do
   not create a new top-level `/public/home/majj/atlas-*` directory for every
   probe, before/after gate, retry or profile. New work belongs below one dated
   `/public/home/majj/atlas-rust-campaign-YYYYMMDD/stages/<stage>` tree and all
   stages share its submission ledger. `YYYYMMDD` is the campaign creation
   date, not a daily rotation rule: reuse the same active campaign across
   midnight and retries instead of creating another top-level directory.
   The current validator pins the exact active root
   `atlas-rust-campaign-20260930`; opening a later campaign requires an
   explicit reviewed lifecycle transition after this campaign is sealed.
   A reconnect, retry or resubmission with the same input pin and manifest
   reuses the exact same logical stage path. An uncertain network or scheduler
   reply never creates a `-retry`, `-r2` or fresh stage. A new stage is allowed
   only when input bytes actually change, after recording its predecessor,
   changed digest/reason, retention class and retirement condition.
   Expanded source trees, Cargo `target`
   trees and profile build trees live in the exact job's node-local or
   auto-cleaned temporary workspace; they are never durable evidence. Retain
   the pin/report and checksum, fixtures, original/candidate raw streams,
   diagnostics, timing/RSS, exact patch, source manifest and only binaries or
   source archives explicitly required by a downstream gate. A downstream
   gate must be reconstructible from those durable inputs; do not make it
   depend on a historical extracted source tree or Cargo cache merely because
   an absolute path is convenient. Historical harnesses that still contain
   retired launchers may be durable only as opaque content-addressed archives;
   expand them solely inside an auto-cleaned compute workspace. Read-only mode
   is not an execution boundary because `python old_stager.py` remains
   runnable. Never remove historical directories in
   bulk: first inventory live jobs, unresolved submission intents, report
   references, uncommitted worktrees and unique commits. See
   `docs/slices/hpc_artifact_retention_2026-09-30.md`.
   Local `/tmp` is not an evidence store. Temporary clones, extracted source,
   transport packages and capture/result bundles must use context-managed
   directories and be removed before handoff. The only exception is one
   explicitly registered active handoff transport whose exact path, owner,
   file manifest, hashes and retirement event are recorded in `docs/HANDOFF.md`.
   Do not accumulate suffix-named revisions, or retain both a reconstructible
   Git checkout/extracted tree and its archive merely for convenience. Treat
   the existing `/tmp/atlas*` inventory as frozen legacy backlog, not as
   permission to create another path; every handoff lists each temporary path
   created during the task as removed or registered.
   Local Git worktrees follow the same lifecycle rule. Parallel agents default
   to the shared primary worktree with non-overlapping file ownership; they do
   not create worktrees as concurrency slots. No new local worktree is
   authorized under the current registry schema: every ACTIVE row is rejected,
   regardless of the LEGACY count, and the old `precreate` command is hard
   disabled before it reads registry, Git or filesystem state. A later
   transition must first
   implement and HPC-verify a durable receipt-bound creator, then obtain
   explicit user authorization for at most one ACTIVE task-scoped worktree
   repository-wide when checkout isolation is demonstrably required and no
   existing checkout can be reused. Record its exact path, branch, starting
   commit, owner, purpose and retirement condition before creation. A retry or
   follow-up reuses that worktree and never gets a new directory. The owning
   task is not complete until tracked, untracked and ignored state, commit
   reachability and active-use status are handed off.
   Passing those checks only makes the exact directory retirement-eligible;
   remove it only after explicit user authorization for that path. Worktree
   removal never authorizes branch or commit deletion. The machine-readable
   source of truth is `docs/worktree_registry.json`; prose is not a registry.
   The 24 audited LEGACY `(path, HEAD, branch)` identities are frozen exactly in
   `hpc/local_worktree_guard.py`. Until a separate receipt-bound retirement
   schema is implemented and HPC-verified, the registry may neither retire,
   add nor replace a LEGACY identity. Adding a matching registry row after a
   raw worktree creation, or deleting a row together with its directory, is not
   authorization and must fail.
   Before editing or delegation and again before handoff, run the read-only
   local Git preflight `python3 hpc/local_worktree_guard.py check`. Any extra,
   missing, detached, locked, prunable or drifted worktree is a stop condition;
   never auto-prune, repair or remove it. Normal checks sample registry, Git and
   the fixed `/home/hoxide/mycodes/atlas*` sibling namespace twice, require real
   no-follow directories with the expected `.git` type, and reject changes
   between samples. This remains detection rather than an atomic creation lock.
   `precreate` is disabled and produces no usable result. PRIMARY pins path and
   branch but deliberately not HEAD,
   because committing the registry advances that HEAD; LEGACY and ACTIVE rows
   pin exact HEADs. This is a cooperative fail-closed authorization and
   detection boundary, not an operating-system interception of raw
   `git worktree add`. Running raw worktree-creation commands is forbidden; an
   unregistered path still present in the managed namespace is detected at the
   next mandatory check, and all work
   stops without automatic cleanup.
10. **Mathematical acceptance is an append-only, independently reviewed
    decision.** The machine-readable ledger starts at
    `tests/reference/hpc/math_acceptance_index_2026_10_01.json`. Keep evidence
    maturity (`accepted` versus `review_pending`) separate from result class
    (`math_pass`, `inventory_only`, failures, rejections and timeouts). Only
    the latest unsuperseded state of a stable `claim_id` may release the next
    rank or operation gate, and only when it is `accepted + math_pass`. The
    broad `operation` groups independent scopes; it is never a supersession
    lineage. Same-operation claims may coexist, while `supersedes` may point
    only within one `claim_id` and must include that claim's latest entry. An
    entry of either maturity requires a registered `(claim_id, status)`
    contract and capture validator, so a pending row cannot relabel arbitrary
    evidence. `accepted` may describe a reviewed non-pass outcome, but it
    additionally requires a claim-specific independent-review validator. An
    execution report cannot review itself: acceptance requires a distinct
    machine-readable review that binds the exact source evidence, report hash,
    operation, selected case IDs, assertions and limitations. A FINAL status,
    exit zero, checker count, hash-only reference, compiled inventory or prose
    review is insufficient. Never edit, delete or reorder published ledger
    entries; append a hash-chained superseding entry. Every published suffix
    also appends its exact `(length, head)` to the checker's frozen prefix
    checkpoints without removing an older checkpoint; default validation must
    reject an uncheckpointed tail, which is a draft rather than a published
    decision. A bounded finite anchor
    never becomes generic associated-cycle, AV-ann, KLV, unitarity or Hodge
    acceptance by inference.
11. **A completed subtask ends in evidence, report, handoff, commit and push.**
    Do not call a subtask complete while its required HPC verification is
    missing or failing. After independent inspection, write a durable report
    or evidence record, update `docs/HANDOFF.md` and the relevant indexed
    status/design document, make a focused conventional commit, and push that
    exact branch to its configured remote. Never stage unrelated user changes,
    never push unverified code as transport, and record a failed push as an
    open handoff blocker rather than silently treating the task as delivered.
## Repository map

- `crates/atlas-core`: lexer, parser, AST, values, evaluator, domain traits,
  diagnostics, and compatible file primitives.
- `crates/atlas-cli`: batch and interactive command-line behavior.
- `tests/fixtures`: Atlas source programs, expected events, and negative cases.
- `tests/reference`: oracle metadata and checksums; large outputs stay on HPC.
- `hpc`: SLURM jobs and synchronization helpers.
- `docs`: compatibility contract, language matrix, design, migration gates,
  and HPC operations.

## Required workflow

1. Read `docs/COMPATIBILITY.md`, `docs/LANGUAGE.md`, and `docs/DESIGN.md`.
2. Add or update a fixture and reference expectation first.
3. Sync to HPC and run the smallest relevant differential job.
4. Implement the smallest module owning the behavior.
5. Run the stage's HPC test and inspect its report.
6. Commit source, fixtures, and report metadata; never commit unverified local
   output.

## Working conventions (user directives, 2026-08-04)

1. **Submit, do not wait.** Once an HPC job is submitted (`sbatch`), move on
   to the next task immediately. Never block the local loop on a pending job;
   results are collected later in batches (a periodic poll is fine, but the
   default is to keep producing work).
2. **Heavy fixtures run on HPC with a generous timeout.** E7 and similar
   Weyl-heavy work go to the `fat` partition with a large `--timeout`
   (e.g. `TIMEOUT=1200`); the `cpu` partition's per-task 8G limit OOMs on
   E7. `#SBATCH` lines do not expand env vars — override via sbatch CLI flags
   (`--partition=fat --mem=32G --export=ALL,TIMEOUT=1200`).
3. **Benchmark every differential comparison.** The drivers
   (`hpc/pipeline_swap_diff.py`, `hpc/reference_capture.py`) record wall
   time AND peak RSS per fixture for both the Rust CLI and the oracle: GNU
   `time -v` on Linux (exact), `getrusage` fallback on macOS (approximate,
   cumulative child peak). Fields: `seconds`, `maxrss_kb`,
   `maxrss_approximate`. Keep this benchmark data in every report.
4. **Keep iterating until the whole Atlas is ported to Rust.** Do not stop
   at one milestone; after a fixture/commit lands, immediately pick the next
   builtin or coverage extension (see `docs/REMAINING_BUILTINS.md`).
5. **Record conventions and puzzles.** Anything a future agent must know
   (blockers, root causes, disproven hypotheses, HPC quirks) goes into
   `docs/REMAINING_BUILTINS.md` and `docs/HANDOFF.md`; do not rely on
   session scratch files for project state.
6. **Prefer HPC-side Git synchronization.** The HPC login node can access
   GitHub. For published source, fetch there and materialize the exact pinned
   commit in a fresh job stage; never pull into the shared dirty development
   checkout. For an unpushed candidate, transfer a small checksummed patch
   against an already pinned HPC baseline and build its source archive on HPC.
   Verify resulting source hashes; do not upload whole source/target trees
   when the baseline is already available. Do not push unverified code merely
   as a transport shortcut. Builds and tests still belong on compute nodes.
7. **Evaluate parallel acceleration with controlled A/B tests.** The user's
   2026-09-28 extension requires investigating computational parallelism and
   selecting effective changes by HPC evidence. First require full output
   equality with the original and between serial/parallel candidates. Hold
   source, input, compiler, node and resource limits fixed; alternate arm order,
   repeat fresh processes and record wall time, CPU time, peak RSS and thread
   settings. Distinguish Rust serial-to-parallel scaling from Rust-versus-C++
   speedup. Failed or mathematically unequal runs yield no accepted speed ratio.
   Prefer60-600s workloads where possible; small tests are correctness controls.
   Existing math-suite observations explicitly force RAYON_NUM_THREADS=1;
   never present those timings as measurements of the multicore configuration.

## Verified repair guard

### Keep oracle availability separate from candidate integrity

- R2 function build3837402 passes units/CLI, but an original signal makes
  its driver exit before final source hashes. R4 build3837531 retains that
  invalid case AND finishes source/input rechecks, with explicit
  FOUNDATION_UNITS_PASS_ORACLE_UNAVAILABLE status and nonzero scheduler exit.
  Only its separately verified build/unit artifacts may be reused. Never
  classify a signal as rejection or silently drop it to turn a gate green.
- Staged checker scripts must be invoked from their pinned stage, not via
  relative discovery in an older source archive that lacks them. R3 was
  cancelled before this path defect ran; R4 tests the absolute script first.
- Balance discovery3837421 finds both wrong rejection and wrong acceptance.
  Preserve the positive and mixed-coercion negative, all recovery values and
  unchanged printer assertion. Full-core3837531 passes the new balance unit;
  a negative's matching message is not matching full stderr presentation.

### Binding constness depends on type scope and actual flag bits

- Binding reads need the DEFINING type-variable floor. Before3837952's new
  unit compiles then fails: an outer free global row is wrongly treated as
  inner rigid T (found[A] while[int] needed). After3837985 passes that exact
  assertion, a local fixed/free independence test and all432core checks, CLI
  and final integrity. Store the floor at every binding creation site; shift
  only free variables on an owned read copy, never mutate the stored scheme.
  All100old Rust streams remain unchanged, but actual any_type grammar is
  still unported; internal scope tests do not prove source-level abstractions.
- Original3837531 rejects five global/local polymorphic assignments while
  preserving concrete sibling mutation, atomic multiple-assignment failure,
  monomorphic shadowing and rebinding. global.w::definition_group and
  axis.w::thread_bindings use per-leaf schemes; fixed lexical variables are
  not free variables. Require both unchanged old before failures to pass
  explicitly after repair, never hide them by filtering/ignoring tests.
  Repair3837694 passes429/429core, both unchanged assertions separately, CLI
  and final integrity guards. Four mutation fixtures gain stdout equality and
  correct rejection; stderr envelopes remain different, so do not call them
  full diagnostic matches. The master still retains original's signal.
- Original3837531 rejects row-loop element/index mutation but ACCEPTS a
  counted-loop index assignment. layer::add takes unsigned-char flags; the
  counted call's true is1, not const-bit0x4, despite its stale comment.
  Follow executable flags plus original output, not the comment. Keep the
  loop fixture and its recovery; this repair is not in constness job3837694.
  Separate3837799 passes430core checks, CLI and final source guards; the loop
  fixture now matches stdout, but its stderr formatting still differs.

### A frozen source archive is not necessarily a complete Git tree

- Overload build3836297 stopped at the exact manifest guard before compiling:
  the parent archive omitted17 generic fixtures already tracked at the chosen
  Git base. A git diff therefore supplied no add-file hunks for those files,
  even though the new source manifest and include_str tests required them.
- Compare against the parent report's actual file set. Add checksummed
  /dev/null patches ONLY for missing files not already represented in the Git
  patch. Preserve submitted stages and keep the strict manifest guard. R2
  build3836325 passes staging/compilation and the three new overload units.
- Capture3836306 confirms field writes survive forgotten type names and reject
  generic/concrete metadata ambiguity. Copying a union's definition and tags
  can make tagged discrimination ambiguous, even for an explicitly named
  receiver. Search all retained definitions, not only active names or current
  projector functions; do not silently pick the receiver's own named slot.
- Builds3836325/3836333 pass the new overload/member units but expose the
  historical empty-subscription diagnostic assertion. Capture3836366 runs its
  exact source: +([][0],[][0]) in int context is ambiguous between(int,int) and
  (rat,int), with independent variables(A,B). In string context the FIRST
  result fails before ambiguity is considered. Preserve all recovery values;
  print the scoped argument body, not legacy independent-star placeholders.
- Full-core3836397 (416pass/3fail/2filtered) exposes a second migration seam:
  current global.w registers generic row/printer/error schemes as ordinary
  overloads. The historical hidden-special fallback wrongly selects a matrix
  for3#[] after exact inference is activated. Capture3836455 confirms ambiguity
  for3#[], ##([]) and both viable row extensions; []##[1,2] now succeeds.
  Preserve linked formal variables and original registration order. Do not
  restore old suffix priority or waive the full-core failures as stale tests.
- Capture3836455 runs the exact historical container-error sources: rational
  remainder with an unknown subscription reaches a runtime bounds error,
  while ambiguous integer arithmetic rejects statically. Capture3836507
  confirms concrete bool-row cardinality conflicts with the generic scheme,
  and a merely coercible ratvec overload does not suppress integer-row length.
- Function-value ambiguity may appear inside a rejected set command without
  a Type/Program header. Failed capture3836409 preserves that gap;3836435
  verifies the corrected classifier against the exact indented envelope and
  negative controls. Complete stream equality is still a separate gate.
- Discovery3836975 exits139 in original for the combined function-selection
  negative fixture. Preserve source/report and isolate the exact trigger;
  RESOURCE_OR_SIGNAL_FAILURE is not a type rejection or language contract.
  It is separate from row-build3836533's frozen88-case gate. The current
  function-detail probe also uses historical lowercase ascii, while latest
  global.w:4147-4148 registers uppercase ASCII. Keep the rejected discovery
  source and use a separately captured uppercase companion for runtime traces.
- Diagnostic subset3837092 isolates original exit139 to (string->int):succ
  alone. Nonfunction context and no-match among multiple ASCII signatures
  reject normally; corrected uppercase runtime probes also recover. Keep the
  five-case isolation catalog explicit, retain the failing single-command
  source, and do not silently shrink the master91-case contract corpus.
- Function-value build3837257 compiles and passes capture/result-selection
  and ambiguity units, but the original-backed captured-printer unit fails:
  a free context variable D is rejected as a non-row before [3,4] is analysed.
  Follow current axis.w::list_display's scoped row specialisation. Balancing
  must also unify polymorphic types (axis-types.w::join_to), not compare them
  with the old monomorphic broader_eq relation or final bare specialisation.
  Keep that unchanged failing unit and require the full core gate after repair.
- Scope discovery3837308 accepts outer polymorphic global/local imports inside
  rigid scopes and preserves outer fixed T across an inner S closure. Assigning
  inner S to outer T rejects. It also DISPROVES concrete casts surrounding an
  abstraction: (int->int):any_type T ((T x):x) rejects inside its rigid body.
  Preserve both concrete-cast errors and recovery; do not infer conventional
  instantiate-after-body semantics from the older prose. The full95-case probe
  retains the separate original signal and is not an accepted differential run.

### Generic declarations and angle tokens have nonstandard boundaries

- Original captures3835190/3835224 show that every comma-separated binding
  in an any_type block executes separately: a bad middle initializer does
  not undo the first binding or prevent the last from being installed.
  Ordinary SET keeps parallel analysis and installs neither sibling when
  the second refers to the not-yet-installed first. Preserve both fixtures.
- Adjacent >>> is one relation OPERATOR, not three constructor closers.
  Keep the rejected discovery source and the accepted spaced companion.
  lexer.w:720-768 leaves single '<'/'>' without newline suppression;
  maximal runs over <=> are operators and never fuse with ':='.
- Name-analysis diagnostics can lack a 'Name error' heading. Test the exact
  raw indentation/envelope: original Undefined identifier has two leading
  spaces. A trimmed display is not classifier input; loader failures must
  remain invalid. Full stdout/stderr equality is a separate gate.
- A stage manifest must describe the files actually copied from its frozen
  baseline. Capture3835154 stopped before execution because a local helper
  hash was mixed with the older staged helper. Correct in a fresh stage,
  preserving submitted stages and the failed guard.
- Constructor build3835225 compiles and passes its new grammar-action tests,
  but the older manually scoped bare-T test expects rejection at TYPE_VAR.
  Once TYPE_VAR participates in type syntax, bare T still rejects, now at the
  missing cast colon. Keep that rejection and a binding-position TYPE_VAR
  rejection/recovery check; token-classification tests must not prohibit valid
  type-prefix syntax while enforcing that variables are not value identifiers.
- Constructor R2 build3835267 reaches all expected structural values, but its
  new session test inspects Output instead of ReportLine. Typed prints emits
  ReportLine, including a newline. Keep the failed run and assert whole lines
  in the actual event channel; a log containing right values is not a passing
  full-core/CLI/capture gate. R3 build3835776 passes the full413-test inventory
  (411pass, two known failures separately executed) and62-case capture.
- Member capture3835786 confirms generated generic projectors participate in
  normal ambiguity: a concrete overload does not win over a generic match.
  Repeated injector variables reject mixed arguments; result constraints share
  the argument's substitution. Keep these negatives and their recovery output.
  The first field-instance probe accidentally binds reserved keyword fi;
  preserve its rejection and use the separately named valid companion.
- Capture3836211 verifies that field assignment uses matching type definitions,
  not the current projector closure. A generic same-name concrete overload and
  a monomorphic projector replaced by a function returning99 both still allow
  field writes in original; Rust rejects them. Follow executable axis.w:8824+
  and axis-types.w:1454 matching_bindings, not stale prose above that code.
  Search retained type slots and reject multiple field/tag candidates.
- The positional-union discovery uses invalid ():0; the zero-argument lambda
  literal is @:0 (parser.y:261). Preserve the rejected source and its valid
  companion. Syntax failures do not validate applied-union branch inference.
- Fixing that lambda does NOT make the old positional case syntax valid:
  capture3836223 rejects case/in/function branches in latest original.
  parser.y:426/489 uses case subject | (pattern): body | ... esac for untagged
  union discrimination; case/in is the integer-case grammar. Capture the
  current patterns and the isolated monomorphic legacy negative before changing
  historical casefor_b6 tests. R4 capture3836236 confirms original8/0 for the
  current pattern syntax, Rust rejection, and original rejection/Rust acceptance
  of the isolated legacy syntax. An old passing unit is not the latest contract.

### Type-variable scope is lexical, and command boundaries remain observable

- Capture3834951 accepts sibling `any_type T` scopes, while3834266 rejects
  nested reuse before the outer scope ends. Do not implement conventional
  same-name shadowing or reserve the name beyond its lexical group.
- Keep duplicate formal slots when assigning indices: lookup uses the first
  occurrence and an inner variable's index includes all outer slots, even
  unused duplicates. TYPE_VAR is not an ordinary value-binding IDENT.
- A generic recursive group's bare self-reference forwards the group's
  arguments (MathGenericList becomes MathGenericList<A>); it is not an
  unparameterized constructor call.
- Original rejects the constructor's mandatory bang on the next physical
  line after a completed type spec. Keep that disproven acceptance fixture;
  virtual nesting is not permission to suppress every declaration newline.
  Parser actions must affect subsequent token classification lazily, and
  LALRPOP lookahead timing needs its own tests rather than assumptions from Bison.

### Named definitions have identity, structure and a live binding

- Original-backed capture3834815 accepts simple named union discrimination
  and forgetting/reusing a type identifier; unchanged Rust3834754 rejects
  both. Historical simple-alias exclusion tests are not latest-oracle rules.
- Keep old table slots when redefining or forgetting a name: values and
  closures can still reference them. Update the active name binding separately.
  Distinct nonrecursive names may be structurally equal; distinct recursive
  identities must terminate comparison rather than expanding indefinitely.
- Preserve names while checking structural consumers (row operations, tuple
  patterns, function calls); do not change only their display. A copied alias
  also retains field metadata. Query locations belong to the live definition,
  even if an identical type slot is reused. Pass the actual lexer terminator
  into definition spans: original set_type's @$ includes the newline.
- Capture3834868 disproves two tempting assumptions. The current original
  retains an old generated projector after type redefinition (clean_out's
  kind()!=tabled early return bypasses its documented cleanup). Named-void
  global initializers retain their actual scalar/row/function values; coerce
  leaves the expression unchanged. Preserve these inputs and observable
  behavior rather than implementing the stale prose or intuitive discarding.
- Full-core review3834919 exposes a historical declaration assertion missed
  by filtered suites. Before changing it, capture3835038 runs its exact source:
  original reports Pair (not expanded(int,int)) and preserves all four field/
  mutation values. Keep that full-output fixture and failed review; never
  update a historical expected result just because the new Rust prints it.
- Full-core build3835058 then passes405 of407 tests; the two known global/
  local polymorphic assignment failures are each executed separately at their
  original assertions, not ignored. Use the exact inventory/summary gate when
  changing shared type infrastructure; filtered suites missed the stale unit.

### An oracle loader failure invalidates differential evidence

- Bridge3834754 passes136 unit checks and builds the Rust CLI, but its36
  oracle arms never start: the build-only batch environment omitted GCC's
  libstdc++ directory, producing GLIBCXX_3.4.26/29 loader errors.
- Capture jobs must derive the runtime library directory from the selected
  g++ and export LD_LIBRARY_PATH. Treat OTHER_FAILURE/timeouts/signals as
  invalid oracle execution, not type rejection or compatibility evidence.
- Preserve the failed capture and replay only the interpreters in a fresh
  stage, rehashing the original build/source/command logs and exact candidate
  binary. Do not rebuild or alter the already verified candidate just to repair
  the test environment. The replay helper has explicit negative checker tests.

### Type names need a live parsing and semantic environment

- Original-backed capture3834702 accepts named casts/parameters and rejects
  a wrong component at type analysis. Distinguish TYPE_ID from IDENT; type
  names must not become ordinary expression or parameter-binding tokens.
- Pass the live type table to both command parsing and redirect-body parsing.
  Semantic casts, parameters and recursive results must use resolve_in, not
  the diagnostic-only resolve helper that replaces unknown names by holes.
- Latest type definitions retain names and definition locations. Legacy Rust
  query/report text is not the latest oracle golden; preserve full-output
  mismatches while migrating the alias table and scheme-carrying analyzer.
- Read actual CWEB actions as well as prose: axis.w:4110 raises and lowers
  the existing inference context through type abstraction, despite nearby
  older prose claiming an unconstrained fresh context. Preserve substitutions.

### Type imports must carry pending assignments and their scope

- Matching foundation3834447 passes38 type tests/3 coercion tests and CLI
  check. Structural matching must expose the requested components while
  preserving repeated-variable constraints and recursive nominal identity.
  Direct function calls apply argument substitutions to the result; a failed
  two-sided trial restores both expression bodies and both assignment scopes.
- Source-backed scope foundation3834389 passes27 type tests (nine new),
  three coercion tests and CLI check. Import both the type body AND shifted
  pending substitutions; adding fresh empty slots alone discards constraints.
- Bake pending substitutions before raising the rigid-variable threshold.
  Pack tuple components with each component's own threshold, then freshen
  their free ranges separately. Reuse the overload argument's variable shift
  when substituting its result, and clear/restore failed candidate trials.
- Preserve unused declared constructor slots while the scope is clean.
  These are internal contracts, not generic-language acceptance: the active
  parser/analyzer migration and global/local wrong-assignment repairs remain.

### Parallel scaling is not removal of excess mathematical work

- E7 A/B3834321 and independent review3834377 verify all four whole-output
  comparisons: Rayon4 is2.958576x faster than Rayon1 with18.54% more peak RSS.
  Yet both Rust arms still consume about69-70 CPU seconds; Rust4 remains
  about17.7x slower than original. Keep algorithmic work reduction separate
  from serial/parallel scaling, and profile before attributing the CPU gap.
- Recheck full streams across rounds, not just within each pair. Bind the
  scheduler job/node/CPU count, affinity, binaries, source, GNU time metrics
  and alternating schedule. Reject incomplete/failing trials even if their
  reported summary looks fast; test the reviewer against those failures.
- Scoped four-thread selection does not justify a global thread default,
  D8 limit increase or claims about unitarity/Hodge/FPP/AV/cycles. See the
  indexed parallel A/B slice for the exact retained experiment and review.

### Structural specialisation must expose the receiver's components

- Foundation3834296's same test fails with the earlier Applied implementation
  and passes after: a read-only can_specialise check returned true but left the
  receiver Applied instead of the Row the caller needed to inspect.
- Follow axis-types.w:942-979: expand/refine the receiver on successful
  structural specialisation. Preserve the distinct-recursive-name termination
  guard and use disjoint before/after build targets. This internal fix is not
  generic-language support; the active analyzer still needs its scope migration.

### Type abstraction is not conventional lexical shadowing

- Generic capture3834266 accepts nested abstractions with distinct T/S names,
  but rejects an inner `any_type T` while outer T is still a TYPE_VAR token:
  the declaration grammar expects IDENT. Duplicate constructor formals in one
  not-yet-installed list are a different case and remain accepted.
- The same capture shows that polymorphic LOCAL empty rows are implicitly
  constant too (`axis.w::thread_bindings`), not just global bindings. Preserve
  both failing regressions, concrete [int] assignment controls and mutation of
  fixed-T locals; neither a global-only nor an empty-value-only fix is sound.
- Discovery fixtures must reach the intended semantics. Adjacent `>=` is one
  operator, and named struct specifications require at least two fields. Keep
  the rejected discovery inputs and add valid companions; do not interpret
  these syntax failures as evidence about unused type-parameter substitution.

### Missing upward Cayley images have a zero polynomial, not a missing descent

- E6 probe3833624 and build3833716's unchanged before test execute the
  imaginary-II `second image` panic. A partial block can contain only one
  upward Cayley image; original KL_y(UndefBlock) resolves to its zero slot.
- Preserve required downward links. Only optional upward imaginary-II
  contributions are zeroed. The same integer/half-scale test passes after;
  independent review3833739 verifies complete E6 KL output and retained
  F4 coefficient/history cases (14 mathematical matches,1 rejection).
  This does not establish general associated-cycle or E7 correctness.

### Discover polymorphic contracts instead of assuming conventional rules

- Current-original capture3833740 rejects an int call ambiguous between a
  generic identity and a concrete int overload; do not assume the concrete
  overload wins. It also ACCEPTS duplicate type formal names. Preserve the
  discovery inputs and investigate substitution before imposing uniqueness.
- Constructor-arity rejection lacks the literal `Type error` heading.
  Bind rejection expectations to the actual diagnostic and failing command;
  a Rust syntax error before the declaration is not a matching type rejection.
- The one-CPU capture submission rejected8GiB and reported a4GiB allowance.
  The same small capture completed with a2GiB CLI override. This does not
  establish a partition-wide4GiB cap: the two-CPU repair job used8GiB.
  Check the actual requested CPU/memory combination and scheduler response.

### Preserve the polymorphic versus concrete assignment distinction

- Capture3833758 shows original rejecting `set x=[]; x:=[1]` as constant
  while Rust accepts it. Probe3833788 executes the unchanged-runtime unit
  failure (`constant=false rejected=false`), after permitting concrete `[int]`.
- R3 capture3833787 confirms the concrete `[int]` control matches completely.
  Original global.w:992 derives implicit constness from the binding's type
  scheme. Preserve that type-level rule, including fixed versus polymorphic
  variables; do not make every empty list or every type variable constant.
  The regression remains explicitly failing until a real after-pass and
  full original-backed comparison exist. See the indexed generic-language slice.

### Compare archive paths in extracted-tree coordinates

- Review3833635 failed before examining mathematics: tar stored ./crates/...
  but file_manifest used crates/.... Bytes and archive pins had not changed.
- Normalize safe relative tar member names before exact file/hash comparison;
  reject traversal, absolute names, unsupported links and duplicate canonical
  files. Keep regression tests for both spelling forms and rejected aliases.
  Re-review existing frozen results with separately pinned checker code;
  never alter their source archives, reports or numerical comparison rules.

### Endgame search failure is not an absent cross

- F4 probe3833589 computes the same wrong P(4,334) in cold and history
  contexts. In first_endgame_pair, upstream kl.cpp:318-340 distinguishes
  an absent cross (valid pair with no t needed) from a defined cross with
  no suitable t (continue searching s). Rust had both branches wrong.
- Build3833612's unchanged coefficient unit fails before and passes after
  correcting only those branches. Review3833712 also verifies complete F4
  history output, including every coefficient, while retaining the separate
  E6 failure. Preserve both checks; one coefficient alone cannot establish
  correctness of the full table or downstream associated cycles.

### Keep patch backup files outside frozen source archives

- E6 probe3833616 was rejected before compilation because an offset GNU
  patch application created domain_builtins.rs.orig, changing the file set.
- Generate patches against the exact pinned base and use
  --no-backup-if-mismatch when staging into a fresh disposable tree. Keep
  the baseline archive as the recovery copy and retain exact file-set checks;
  never weaken the source guard to ignore accidental backup files.

### Revalidate old language exclusions after an oracle upgrade

- Latest original7e1b958c declares ANY_TYPE in parser.y and uses generic
  constructors and any_type scopes in basic.at. The old Rust lexer unit
  `oracle_non_keyword_is_scanned_as_an_identifier` asserts the opposite
  for any_type; its historical expectation does not define the new oracle.
- Keep the pinned old evidence, but use current original-backed positive
  AND negative fixtures to migrate lexer, parser, type-variable scope and
  overload/type-constructor semantics together. Do not preprocess away
  generics or replace upstream scripts merely to pass high-level math tests.

### Equal KL index matrices do not prove equal polynomials

- F4 history review3833539 and diagnosis3833564 find identical parameters
  and336x336 index matrices, yet pool entry102 at(4,334) differs:
  original[0,0,2,3,3,2], Rust[0,0,4,6,5,3]. The other175 pool entries match.
- Always compare complete coefficient vectors as well as parameter/index
  data. Internal cold/warm equality can hold while both answers are wrong.
  Preserve the full-output fixture and test the affected polynomial in both
  cold-full and partial/full history contexts; do not bless matching shapes.

### Fundamental-fiber coordinates are not grading-shift positions

- E6 probe3833407 reproduces the external-form constructor failure without
  full Weyl enumeration. The fundamental adjoint fiber basis is e1/e3, but
  e3 flips several imaginary-subsystem simple roots; its FIRST flipped root
  has ambient-simple coordinates[0,0,1,1,1,0], not a datum-simple root.
- Never infer a fiber basis coordinate from the first flipped grading root.
  For a distinguished diagram involution, exchanged pairs vanish in the
  fiber quotient and fixed fundamental-coweight coordinates survive. Verify
  the actual basis representatives and their ascending coordinate order.
- Original FormNumberMap uses the PARTITION overload of specialGrading,
  not its Fiber overload. Preserve the maximal-popcount/highest-index
  election and require unchanged-unit after-pass plus full original-backed
  E6 KGB/KL comparison before accepting an external-numbering repair.

### Read template defaults for root/coroot operations

- `InnerClass::int_item` calls `additive_closure` without a template argument;
  upstream `rootdata.h` defaults `for_coroots=true`. The Rust locator wrongly
  ported root sums. In B/C/F/G, root and coroot closures need not coincide.
- Probe3832726 executes two failing regressions: B2 closure gives4 instead
  of8 elements; F4 half-scale locator fails integral-image positivity.
  Preserve these tests, use exact coroot evaluations as an independent check,
  and require after-pass plus original-backed full-output comparison.
- `combine_roots`'s boolean selects subtraction, NOT root versus coroot.
  Do not flip that flag or remove positivity checks as a shortcut.

### Undefined links in a partial KL block are descent-specific

- F4 case80/job3832508 panics at `cross of extremal`. The unchanged-runtime
  probe3832534 isolates `x=263, y=278, s=2`, with descent `RealTypeII`.
- A real-II cross can leave a downward-closed partial block. Original
  `blocks.cpp` explicitly permits `UndefBlock` there, and `KL_pol` evaluates
  its contribution as zero. A complex descent or inverse Cayley descent,
  in contrast, must remain inside the interval.
- Do not eagerly unwrap every extremal's cross, silently zero every missing
  link, or require cross-closure of a partial block. Preserve the descent
  distinction and compare full KL parameters/matrices/pools, including
  partial/full/partial cache histories. Keep the original-backed regression
  and before-panic/after-pass evidence; no acceptance from avoiding a panic
  alone. See `docs/slices/math_failures_2026-09-28.md`.

### Isolate Cargo outputs for before/after archive builds

- HPC job `3832260` compiled the before source and failed the intended D4
  assertion, but the after command finished in 0.7 seconds and executed the
  same old word/binary despite the verified source containing the reversal.
  Both archive trees preserved timestamps and shared `CARGO_TARGET_DIR`.
- Never share Cargo target directories between the before and after trees in
  a regression proof. Use separate fresh `target-before` and `target-after`
  directories, and record their paths with each command. An exit code alone
  cannot establish which source was executed.
- Keep failed build artifacts. The isolated replacement `3832301` gives
  before-assertion-fail/after-pass. Differential review `3832333` confirms
  the D4 repair and E6, but E7 still times out; do not equate the unit pass
  with all-FPP acceptance. See the R2 submission/review JSONs in
  `tests/reference/hpc` and the separate E7 long-run receipt.

### Bare-core mathematical fixtures must use bare-core operations

- R10 jobs `3832308`/review `3832312` showed that equality of `[Param]` and
  complete KL tuples requires script overloads, even in the original.
  Loading latest `basic.at` would mask the independent kernel regression
  behind Rust's unrelated generic-language gap.
- Compare Param/vec/int entries explicitly, retain the full printed records,
  and keep the failed fixture artifacts. R11 review `3832340` then confirms
  four original-pass/Rust-fail regressions without changing either engine.

### Owned `LatticeInvolution` builders

- Root cause: a migration from borrowed to owned involution input left a
  read-only helper call unborrowed, which failed only at Rust type checking.
- Diagnostic: use the frozen `real_group_preflight.sbatch` package compile;
  job `3463647` reported the concrete mismatch before tests ran.
- Prevention: audit every helper call before the final move into the fiber
  model, then require a passing HPC preflight such as job `3463683`.

## Rustcox reuse

`rustcox` is a related but separate project. Its Coxeter, root-system, Bruhat,
Laurent-polynomial, W-graph, and canonical JSON modules may be adapted after
their APIs and semantics are wrapped by Atlas domain traits. Do not make the
Atlas language layer depend directly on rustcox internals.

## License

GPL-3.0-or-later. Reused code must have a compatible license and retain notices.
