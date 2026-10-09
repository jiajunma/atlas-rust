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
  2026-10-03 lesson: a no-tool probe can produce its complete assistant
  answer and still fail to exit before the deadline (exit 143 after SIGTERM).
  Always parse `stdout.jsonl` for the finished assistant record before
  discarding a timed-out probe run; the answer may be fully usable, with the
  process-group cleanup evidence in `result.json`.
  2026-10-09 lesson: the converse failure also occurs — a 16.5KB
  single-shot probe prompt (reference files embedded verbatim) produced
  ONLY the version banner in 180s (`stdout.jsonl` 58 bytes, nothing to
  salvage). Treat a zero-output timeout as retryable once with a longer
  deadline (the retry at 300s completed in 291s); budget >=300s for
  single-shot probes of that size on `kimi-code/k3-256k`, and fall back to
  a local review if the retry also yields nothing.  The probe profile is a
  good fit for structural fixture reviews: findings were limited to one
  dead binding and one non-ASCII header character, both independently
  verified by grep before applying.
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
   AFTER v3 job3875239 is FINAL and accepted. Published commit `23245b54`
   retains only the idempotent acceptance-index receipt-recovery pair, bound to
   FINAL job3879103 and its exact existing stage; it is not a Weyl-capture
   transport. That pair stayed disabled through the 2026-10-01 transition
   snapshot and is absent from the current launcher tree; re-enabling it
   requires a new reviewed binding, not a reference to commit `23245b54`.
   Tests-first `weyl-context-core-before-v1` job3884862 is FINAL
   `FAILED 1:0` in checker self-tests: four suites passed, then the driver
   suite found two stale test assumptions; Cargo and Atlas never ran. Its
   immutable failure is not mathematical BEFORE evidence. BEFORE-v4
   job3886748 is FINAL `COMPLETED 0:0` and independently inspected: all119
   checkers pass, the632-test inventory and retained ladder control pass,
   both original captures byte-match the frozen v8 goldens, and exactly the
   two known Rust regressions fail (0 passed,630 filtered). This retains the
   tests-first BEFORE proof only; it releases the smallest owning Weyl-owner
   semantic repair plus the same original-backed AFTER gate, nothing more.
   BEFORE-v3 job3884903 is FINAL
   `FAILED 1:0` because a coordinator version replacement changed the
   allowlist's historical ladder BEFORE-v2 error message to v3. The v4
   candidate restored the
   expected literal; never modify the frozen historical driver to match it.
   New rejection diagnostics bind filename/line/node and the exact mutation
   is retained in the existing allowlist test. BEFORE-v2 job3884880 is FINAL
   `FAILED 1:0`: 112 prerequisite tests passed, then two allowlist checks
   rejected dictionary unpacking and a subscript in top-level assignments.
   Cargo and Atlas again never ran. Preserve v2 and its 51-input freeze
   record; fix only the expressions, without weakening the allowlist.
   The v8 creator is closed: job3884807 was submitted exactly
   once and is FINAL `COMPLETED 0:0`. Independent inspection confirms the two
   Rust Weyl-owner semantic mismatches and requires an original-backed
   regression BEFORE. Both exact v8 transports are removed, the inspected
   queue was empty, and v8 grants no math/cache/performance/rank acceptance.
   Never invoke its creator again. No production repair may precede a
   changed-input tests-first successor that proves the unchanged Rust failures;
   never combine launcher, driver, sbatch or allowlist bytes from the published
   and transition snapshots.
   Reinvoking the published index pair may only validate or recover its same
   durable receipt; it must not submit a duplicate or create a sibling.
  Every old direct
   `sbatch` and pre-campaign `submit_one` entry point must fail before parsing,
   copying, creating directories, writing pins or contacting SLURM.
   This does not retrofit historical bulk launchers. See
   `docs/slices/progressive_validation_2026-09-30.md` for pinned rank gates;
   every operation needs its own smaller-rank acceptance before escalation.
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
12. **Kimi is a reviewed routine-work assistant, not a mathematical oracle.**
    Kimi may handle bounded mechanical analysis, scaffolding or documentation
    when its account/CLI is available. Do not delegate mathematical truth,
    original-Atlas interpretation, acceptance decisions, destructive actions,
    secrets or unbounded repository changes. Give it exact file ownership and
    a no-new-worktree/no-local-test boundary; inspect every byte or claim it
    returns and verify code on HPC. Each invocation must be recorded in the
    task report and handoff with CLI/model/session when available, prompt scope,
    files touched, result, independent checks, rejected suggestions and a
    reusable lesson. Freeze the exact prompt, owned-file list and their hashes
    before launching, and capture raw output/exit status even for launch
    failures; unknown historical fields stay explicitly unknown. (The
    historical Python kimi-cli 0.42.0 rejected `--plan` combined with `-p`;
    the verified current install is Kimi Code CLI 2.1.1 per the sections
    above — recheck version and `--help` after any upgrade instead of
    trusting either note.) An external-provider retry may
    transmit repository bytes, so it needs authorization for the exact
    payload/destination and must not be rerouted after rejection. No invocation
    means no fabricated Kimi lesson.
    KB-packet probe lessons (2026-10-06 arc): the working route is a
    full-bytes prompt — paste the complete source file into the prompt because
    the probe profile has no tools — followed by maintainer claim-by-claim
    verification of the returned packet against the actual `.rs` before
    committing. Scale the timeout to prompt size (roughly 24s per KiB of
    prompt; one 360s deadline on a 25.5KiB prompt was too tight), and grep
    `kb/sources/index.md` before drafting: six duplicate packets for
    already-covered files had to be merged away.
13. **The Atlas wiki evolves through the pinned compiler and reviewed sources.**
    Use repository launcher `./kb/llmwiki`, never an unpinned global substitute.
    For each mathematical/algorithmic implementation change, update the exact
    `kb/sources/` packet and relevant page, generate only review-held candidates,
    inspect them against current source and evidence, then include approved wiki
    changes in the same verified commit. Follow `kb/AGENTS.md`; generated prose
    is not mathematical acceptance and must distinguish proof, original-backed
    observation, implementation fact, proposal and benchmark evidence.

## Repository map

- `crates/atlas-core`: lexer, parser, AST, values, evaluator, domain traits,
  diagnostics, and compatible file primitives.
- `crates/atlas-cli`: batch and interactive command-line behavior.
- `tests/fixtures`: Atlas source programs, expected events, and negative cases.
- `tests/reference`: oracle metadata and checksums; large outputs stay on HPC.
- `hpc`: SLURM jobs and synchronization helpers.
- `docs`: compatibility contract, language matrix, design, migration gates,
  and HPC operations.
- `kb`: the in-repository Obsidian-compatible knowledge base for Rust
  mathematics, algorithms, implementation and design evolution. C++ is the
  baseline for alignment, not a mandatory comparison in every note. Start
  with `kb/index.md` and follow `kb/AGENTS.md` when editing it.

## Knowledge evolves with source

The user selected **llm-wiki-compiler** on 2026-10-01. Its project root is
`kb/`, with an exact dependency in `kb/package.json` and the dependency graph
in `kb/pnpm-lock.yaml`. Use `./kb/llmwiki` from this repository; do not replace
it with another knowledge engine or an unpinned global/npx installation.

For changes to mathematics, algorithms, observable behavior or architecture,
inspect related `kb/` pages and update affected explanations in the same task.
Include task-related KB changes with the corresponding reviewed source commit;
do not create a separate KB repository, worktree or acceptance ledger. A task
with no knowledge change needs no artificial note. Keep existing `docs/`,
fixtures and HPC records in place and link to their exact scope and versions.
KB editorial review never grants mathematical acceptance or releases an HPC
gate. Source discovery, builds and verification obey the hard rules above.

Maintain bounded Markdown source packets under `kb/sources/` with the actual
Rust source version, symbols, assumptions and evidence references. The compiler
does not detect changes in the `.rs` files behind those packets. Grep
`kb/sources/index.md` before drafting any packet: it is the coverage
authority, and re-drafting an already-covered file wastes the review cycle
(six duplicates had to be merged on 2026-10-06). Rust evolution
is the main narrative; add C++/CWEB/script comparisons when aligning with the
baseline. Follow `kb/AGENTS.md` for the native schema and existing-note boundary.

Generate candidates with
`./kb/llmwiki compile --review --instructions AGENTS.md`; the launcher changes
directory to `kb/`, so this explicitly supplies `kb/AGENTS.md` to the model.
Keep the native `review.hold: ["all"]` policy. Inspect each candidate against
its exact sources and current destination before approval; stop and regenerate
if either changed. Approval is editorial work within the authorized task, not
an additional user-confirmation requirement. Do not trust the compiler's
`fresh` flag as per-page evidence, bulk-approve unseen candidates, or use
unattended watch/quickstart/query-save paths to bypass this review.

Local installation of this documentation tool and document-only authoring are
authorized knowledge maintenance. This does not authorize local Atlas/CWEB
execution, examples, builds, tests, benchmarks or automated KB verifier suites;
their existing HPC rules and submission gates still apply.

## Required workflow

1. Run `python3 hpc/local_worktree_guard.py check`; this is a read-only local
   Git inventory preflight, not a build/test. Stop on any mismatch.
2. Read `docs/COMPATIBILITY.md`, `docs/LANGUAGE.md`, and `docs/DESIGN.md`.
3. Add or update a fixture and reference expectation first.
4. Sync to HPC and run the smallest relevant differential job.
5. Implement the smallest module owning the behavior.
6. Run the stage's HPC test and inspect its report.
7. Run the worktree preflight again, then commit source, fixtures, and report
   metadata; never commit unverified local
   output.

## Working conventions (user directives, 2026-08-04)

**Priority override, 2026-09-30:** Fix the large performance gap first,
starting with small rank. Pause new mathematical coverage/rank expansion;
retain existing complete original-backed results as regression gates.
Separate startup/library loading from mathematical computation, profile on
compute nodes, and accept an optimization only after controlled before/after
measurements and unchanged full results. A large timing gap is evidence to
investigate, not by itself proof of a mathematical error. Keep the 10-job
ceiling and default one focused job; do not trade correctness for speed.

1. **Submit, do not wait.** Once an HPC job is submitted (`sbatch`), move on
   to the next task immediately. Never block the local loop on a pending job;
   results are collected later in batches (a periodic poll is fine, but the
   default is to keep producing work). This means useful reading/editing and
   analysis, NOT more submissions beyond the 10-job cap or advancing rank
   before the preceding mathematical gate has been inspected and passed.
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
8. **Systematically expand correctness coverage through rank6.** The user's
   2026-09-29 directive asks for comprehensive comparisons across groups of
   rank <=6 after the smaller correctness gates. Enumerate simple types,
   all compatible inner classes/real forms and intermediate central quotients,
   not only split/compact or SC/adjoint endpoints. Include complex groups of
   complex rank <=6 and separately record their doubled real root-datum rank.
   Extend next to reducible types, diagonal quotients and reductive/tori cases;
   explicitly state which of these are still missing. Inventory equality is
   not KLV/unitarity/Hodge/associated-cycle/AV-ann acceptance. Keep full
   coefficients, multiplicities, ordering, singular/nonintegral inputs and
   history-dependent cases; infinite parameter spaces cannot be certified by
   a finite sample. Split jobs by form/operation, preserve original rejections
   and timeouts without counting them as passes, and retain time/RSS/raw output.
   Freeze a new catalog rather than modifying already submitted surveys.
   Execute it progressively: rank 1 small-group controls, then rank 2 cases
   (including non-simply-laced types), then ranks 3, 4, 5, 6 as each operation's
   smaller-rank gate passes. Separately report complex rank and doubled real
   root-datum rank. Preserve every unexecuted case as pending coverage; do
   not silently drop it or call the backlog tested. Historical 100/510-case
   dispatchers must not be reused without adapting and verifying the 10-job
   cap and inspected, rank-by-rank release gates.
9. **When the HPC tunnel is down, do documentation and small local-only jobs.**
   The user's 2026-10-06 directive: while HPC is unreachable, keep producing
   useful work that does not need it — KB/wiki source packets and page
   maintenance, documentation updates, code reading, hashing and JSON evidence
   records, the read-only worktree guard, and Kimi probe drafting. Prefer
   writing documents over writing code: do NOT queue speculative production
   edits without their HPC gate, and never run local Atlas builds/tests to
   substitute for them. Queue the blocked HPC work behind a retry cron with a
   complete self-contained runbook instead of polling, and resume it the
   moment the tunnel answers (reconcile first: queue, ledger, stage,
   accounting — this cluster's sacct 21.08.8 rejects `-h` and prints a
   166-line usage page; never count usage text as jobs).

## Verified repair guard

### Current frontier (read this first)

- Latest accepted mathematical gate: Weyl context-core AFTER-v5,
  job3900050, FINAL `COMPLETED 0:0` — bounded A1 Weyl owner/dual semantics
  only; every release flag stays FALSE. The validated tree is landed as
  production commit `690c2b92`. See the 2026-10-06 UPDATE entries below.
- Next gate: the G2 asymmetric-interface witness. Provisional fixtures
  `tests/math/generics/weyl_context_g2_cold_dual.atlas` and
  `tests/math/generics/weyl_context_g2_prewarmed_dual.atlas` (superseding the
  `a81db81c` draft) are NOT truth until the original's complete behavior is
  captured on HPC.  The capture pair was migrated in place from the closed
  before-v4 gate to `weyl-context-g2-v1` (see the 2026-10-09 migration UPDATE
  below): the direct PREDECESSOR is after-v5 job3900050 — the campaign ledger
  is strictly linear, so the chain head, not the case-set ancestor — and
  before-v4 job3886748 is bound historically through the ported
  `validate_before_v4_result`.  Predictions assume the landed repair
  generalizes; the pre-registered review in `docs/HANDOFF.md` maps a DIFFERED
  on exactly `WG_DUAL_OWNER`/`WG_REVERSE_OWNER` to the Cartan-numbering
  question and anything else to the fixture or a new discrepancy.
- G2-v1 submission state (2026-10-09): fully prepared and rehearsed — 129
  checker tests green locally under `umask 022` except the documented
  local-only 0444-env case; payload 65 frozen 0444 files, overrides manifest
  `bcc09dbc…`, 1569-file source chain `dff0e90d…`.  BLOCKED on the SSH tunnel
  (`majj@10.26.14.64`, connection timed out).  The complete resumption
  runbook (rebuild recipe, reconcile checks, transport, the exact one-shot
  invoke, post-checks) is `docs/HANDOFF.md` §"G2-V1 SUBMISSION — BLOCKED on
  the tunnel"; an hourly retry cron checks the tunnel and points at it.
  Never resubmit any prior stage; reconcile queue/ledger/stage/accounting
  first.
- KB lane (parallel, local-only): the source-packet library is complete —
  23 packets plus the regression-library map cover every module of all three
  crates, byte-exact snapshots at git base `964f0033`, implementation-stated
  versus HPC-verified claims separated; index audit fix landed in `5630a1f5`.
  The wiki COMPILATION step (`./kb/llmwiki compile --review`) is blocked on
  the user's interactive `codex login` (recorded in `kb/log.md`).
- After the G2 capture: B2/C2, reverse operand orders, inner-class-dual
  and no-value gates, each with its own original-backed capture; only after
  all of them pass comes the cache work-count BEFORE and any production
  cache edit.  Provisional B2/C2 fixture drafts already exist
  (`weyl_context_b2c2_cold_dual.atlas` sha `9809d0f2…`,
  `weyl_context_b2c2_prewarmed_dual.atlas` sha `e542d380…` — the cross-type
  dual witness), as do numbering-independent G2 reverse-operand drafts
  (`weyl_context_g2_reverse_operands.atlas` sha `652fbf81…`,
  `weyl_context_g2_reverse_prewarmed.atlas` sha `7768db8b…`), plus the
  G2 inner-class-dual routing draft
  (`weyl_context_g2_inner_class_dual.atlas` sha `83c71493…`), the G2
  no-value-relations draft (`weyl_context_g2_novalue_relations.atlas` sha
  `480c4be1…`) and the sole-WeylElt lifetime draft
  (`weyl_context_sole_weylelt_lifetime.atlas` sha `5fb7e7a9…`) — all seven
  witnesses from the 2026-10-03 design slice
  `docs/slices/weyl_semantic_next_captures_2026_10_03.md` are now drafted;
  none is wired into any contract/stager/catalog, predictions wait for the
  G2 numbering-convention answer; see `docs/HANDOFF.md`.  An eighth
  follow-up fixture
  (`weyl_context_g2_prewarmed_transposed_dual.atlas` sha `7ed86be0…`)
  implements the genuine prewarmed-rejection vehicle identified by the
  upstream reading (the frozen G2 prewarmed dual-side triplet is vacuous
  by construction); it belongs to a later arc stage.  A ninth draft
  (`weyl_context_novalue_dual_family.atlas` sha `ba5bf666…`) probes the
  no-value dual-family BuildAndDrop-vs-Skip candidate divergence found by
  the typed.rs registration audit (docs/slices Part 4); capture before any
  registration fix.
- Everything below the frontier entries is historical evidence carrying its
  own supersession markers. A historical "next target" or "reprofile" note
  never reactivates itself; only the latest unsuperseded state of a claim
  releases the next gate (hard rule 10).

### Predecessor transitions: rebind every historical validator before launch

- UPDATE G2 capture-stage migration2026-10-09: the capture pair moved from
  the closed before-v4 gate to `weyl-context-g2-v1` (G2 asymmetric-interface
  witness).  New transition lessons beyond the ones below: (1) a historical
  validator's `successor_stage` assertion must pin the literal stage name —
  `remediation.get("successor_stage") != STAGE_NAME` goes stale the moment
  the successor itself is superseded (caught in validate_before_v3_failure).
  (2) A new predecessor STATE uses the CURRENT descriptor schema
  (`atlas-stage-creation-predecessor-v14`), never the predecessor-era one;
  the frozen historical blocks keep their era versions.  (3) The campaign
  ledger is strictly linear: the direct PREDECESSOR must be the chain head
  (after-v5), not the case-set ancestor (before-v4); bind the ancestor
  through a ported `validate_before_v4_result` instead, and port the retired
  chain's failure validators with locally rebound era identities.
  (4) Stager constants must stay `pure_expression`s for the allowlist: no
  subscripts at module level (`G2_SOURCE` went literal; the validator
  cross-checks consistency instead).  (5) Retire the outgoing pair in the
  same migration (flip its `SUBMISSION_ENABLED`), extend the active pair's
  FROZEN_LAUNCHER trust set and STAGE_INPUT_NAMES to cover it, and swap the
  era-specific mutation directions in the allowlist's rejection battery.
  (6) Historical values for a new predecessor block come from the last
  HPC-validated submission record or a fresh read-only harvest — never from
  a truncated handoff note (three hand-completed sha tails were caught by
  the cross-check against the validated records before any launch).

- UPDATE Weyl AFTER-v3 failure2026-10-06: job3899303 FINAL `FAILED 1:0` at
  `release-build` (E0609) after all125 checkers passed.  The repair patch
  `hpc/patches/weyl_context_core_repair.patch` migrated session.rs call
  sites to `context.kernel.system` but omitted
  `crates/atlas-core/src/domain_builtins/weyl_subgroup.rs` — a file that
  exists only in the frozen campaign source archive, untracked in git, in
  no patch and no history.  The local untracked copy already had the
  correct two lines; it was never committed.  Failure evidence
  `tests/reference/hpc/math_weyl_context_core_after_v3_failure_2026_10_06.json`,
  report SHA `195cc4fca…`.  Lesson: a migration patch must enumerate its
  own blast radius — grep the whole source archive for the old field/API
  shape, including untracked and archive-only files, before freezing;
  a build gate one stage later is the only thing that catches it.
  The AFTER-v4 successor completes the migration; never resubmit3899303.

- UPDATE Weyl AFTER-v5 acceptance2026-10-06: job3900050 is FINAL
  `COMPLETED 0:0` on cu006, report status `WEYL_CONTEXT_AFTER_REGRESSIONS_PASS`.
  Independent inspection accepts the bounded A1 semantic gate: all13
  commands exit0 (127 checker tests, release-build, 632-test inventory,
  both regressions, ladder control); the source manifest is exactly
  `84a3fbfd…` (1567 files, proving the build/tests ran on the completed
  kernel-system migration); cold_dual fully byte-equal; prewarmed_dual
  matches on stdout/exit/ordered error summaries under the v5 classifier
  change.  Report SHA `3288480d…`; acceptance evidence
  `tests/reference/hpc/math_weyl_context_core_after_v5_acceptance_2026_10_06.json`.
  The report keeps every release flag FALSE and the review releases nothing
  either: no cache/performance/memory/rank/broader mathematical release.
  The gate chain from v8's confirmed Rust semantic mismatch to acceptance
  ran eight HPC jobs (v8 BEFORE + after-v1 harness constant bug + after-v2
  sbatch labels + after-v3 incomplete repair migration + after-v4 gate
  over-assertion + after-v5 pass).  Next: the focused production commit of
  the completed repair, then the progressive semantic AFTER gates (G2,
  B2/C2, reverse operands, inner-class-dual, no-value) before any cache
  work-count BEFORE or production cache edit.

- UPDATE Weyl production commit2026-10-06: the validated state is landed
  as production commit `690c2b92` (41 files).  Landing rule: the whole
  tree was hashed against the v5 gate's source manifest `84a3fbfd…` —
  every `.rs` file matched (0 missing, 0 mismatched, 0 extra), so all
  `crates/**` changes were committed as ONE build-consistent unit rather
  than cherry-picking the 5 repair files (the repair interacts with the
  rest of the tree; the gate validated the tree as a unit).  Docs/meta
  edits (AGENTS.md, README, docs/*, .gitignore) were deliberately left
  uncommitted as the owner's separate work.  Lesson: when a gate validates
  a whole-tree manifest, the faithful production landing is the whole
  validated tree after a byte-level manifest comparison — never a subset
  that was not itself build-validated.
- UPDATE Weyl AFTER-v1 preparation2026-10-03: advancing a campaign stager's
  direct `PREDECESSOR` silently breaks every inherited validator that still
  compares a frozen historical evidence record against bare
  `PREDECESSOR`/`PREDECESSOR_STATE`/`PREDECESSOR_STAGE` (in the after stager
  the before-v3 validator would never match) or against the mutable current
  `EXPECTED_TEST_COUNTS`/`CHECKER_TESTS`/`STAGE_NAME` (frozen
  `expected_test_counts_after` predictions and predecessor checker totals keep
  their historical values; never relabel historical bytes to match the
  successor).  Transplant the old predecessor identity into versioned
  `BEFORE_Vn_PREDECESSOR*` constants and rebind locally, keeping the
  validator body byte-identical to the HPC-verified original.  Campaign
  scoped-creation-file sets in `progressive_submit` must name every retired
  stage explicitly once it is no longer the direct predecessor
  (before-v3 was missed when the predecessor advanced to before-v4); the
  synthetic fixture must add the new predecessor's campaign files and bump
  its ledger-length assertion.  The default local checkout (umask002,
  evidence files0664) masks these failures inside the environment-sensitive
  tests: rerun the suites under `umask 022` and replay every
  `validate_*_failure`/`validate_*_result` against the real frozen evidence
  bytes copied into a 0444 mirror tempdir before any launch.  This local
  static rehearsal caught four genuine transition defects that the
  HPC-blocked period would otherwise have spent a job on.  Local checker
  passes never substitute for the HPC gate itself.

- UPDATE Weyl AFTER-v2/v3 transition2026-10-06: when a stage version bumps,
  the sbatch `--job-name`/`--output` labels are versioned content too.
  AFTER-v2 job3890580 FAILED in3s before any gate because the stager's
  SLURM_OUTPUT_PATTERN moved to v2 while the sbatch labels still said v1:
  sbatch wrote `weyl-context-core-after-v1-3890580.out` into the after-v2
  stage and `validate_stage_topology` rejected the unexpected durable file.
  No report.json exists for such failures; the successor's PREDECESSOR omits
  `report_sha256`/`report_bytes` entirely rather than inventing one, and the
  failure validator asserts `report is None` in the evidence record.  The
  checker now pins both sbatch labels to suffixes derived from the active
  `STAGE_NAME`, so a version bump without an sbatch migration fails locally.
  The same transition also showed the predecessor-rebind lesson generalises:
  `validate_after_v1_failure` referenced the bare module-level
  `PREDECESSOR`/`PREDECESSOR_STAGE`, so advancing the predecessor to after-v2
  required transplanting the old identity into `AFTER_V1_PREDECESSOR*` and
  rebinding locally inside the validator — exactly the
  `BEFORE_V4_PREDECESSOR*` pattern; audit EVERY historical validator for
  bare-predecessor references before each launch.  The predecessor-state
  schema string stopped bumping once it stabilised
  (`atlas-stage-creation-predecessor-v12` is pinned by
  `progressive_submit.STAGE_CREATION_PREDECESSOR_SCHEMA` for all current
  predecessors); do not increment it per stage.

### Attribute small-rank loading before optimizing; retain complete outputs

The ledger below preserves historical measurements and queued follow-ups.
Parent-seal acceptance does not reactivate any historical “next target” or
“reprofile” wording: it remains backlog unless a later current-state entry
explicitly selects it. Ladder AFTER v1 and v2 are immutable harness failures;
changed-input AFTER v3 job3875239 is FINAL and independently accepted in its
bounded root-ladder/directory-governance scope. Do not duplicate it or infer a
rank, performance or broader mathematical release. The append-only index
publication gate job3879103 is also FINAL and independently accepted; it makes
only that bounded claim durable and does not broaden its scope.

- UPDATE ladder acceptance-index publication2026-10-01: job3879103 is FINAL
  `COMPLETED|0:0` after17s on cu075. Independent inspection accepts the exact
  publication/durability gate: stage/index/allowlist suites pass20/34/7 with
  no skips; all24 installed inputs and exact directory topology match the
  frozen manifest; source and retired-stager CAS integrity recheck; the
  ephemeral workspace is absent; and no Cargo or Atlas command ran. The full
  report SHA is
  `43cc79e1630b5ec21a24c5aa1a7c411eafffaa9a4c6911333c8752b0dc63cf9d`;
  independent evidence
  `tests/reference/hpc/math_ladder_boundary_index_v1_2026_10_01.json` has SHA
  `d9448e5cd53ba4c25e6a0fd91a5fdb16e3b423d48fe58531a50d7d7457afb9ad`,
  and the canonical24-entry override manifest has SHA
  `009c2ee6b05abc5508f667a9566e15eaacbefc6154eb545e83d213cb2e5b3a06`.
  The campaign ledger has7 records, the queue is empty, driver-process audited
  legacy-path opens are zero, and the HPC home remains at exactly318 top-level
  `atlas-rust*`
  directories: only the registered child
  `atlas-rust-campaign-20260930/stages/ladder-boundary-index-v1` was added.
  This publishes the bounded A1-plus-central-torus ladder claim only; it is no
  new computation, rank release, performance claim, cleanup authorization or
  KLV/unitarity/Hodge/associated-cycle/AV-ann acceptance.

- HISTORICAL pre-submission storage audit2026-09-30, superseded by the current
  parent-seal submission entry below: local25Git worktrees (~1.93GiB total at
  audit time;8secondary worktrees dirty) and317 HPC top-level entries whose
  names begin `atlas-rust`—a subset of the1174 whose names begin `atlas`—
  exposed a missing lifecycle policy. Recent retained Cargo targets alone
  consumed~828MiB and~416MiB. Do not bulk-delete:
  historical reports use absolute parent paths and dirty/unique worktrees may
  contain user work. New `campaign_workspace.py` requires one dated campaign,
  one shared submission ledger and exact auto-cleaned per-job workspaces;
  `campaign_source.py` stores the accepted Rust source as one path-free
  content-addressed archive; `campaign_blob.py` provides the shared generic
  CAS. Phase-2 `weyl-parent-seal-v1` is locally implemented with resumable
  staging, atomic submission records, two identical-environment legacy traces,
  exact read-set hashes and report/source/inventory/raw-stream cross-links.
  At this historical snapshot it was NOT submitted or HPC-accepted. No
  historical stage becomes cleanup-eligible until its
  FINAL seal plus a separate reachability/replay dry run pass. This is
  infrastructure correction, not mathematical or speed acceptance.

- UPDATE local-worktree guard2026-10-01: the authoritative
  `docs/worktree_registry.json` freezes one PRIMARY/24LEGACY/zero ACTIVE paths;
  SHA `2059e9c5a09ab03d0a10eb2af6b6cd9946a940c980ded1b5d8074d977f2790cb`.
  `hpc/local_worktree_guard.py` uses only fixed `/usr/bin/git` read commands in
  a sanitized environment and rejects an unregistered, missing, special or
  drifted live entry. It also samples the fixed local Atlas sibling namespace
  twice without following links, rejects persistent extra sibling names and
  replacement or inode drift occurring inside the sampling window, and
  requires the exact frozen24 LEGACY identities until a
  retirement-receipt schema exists. PRIMARY intentionally omits HEAD to avoid a commit
  self-reference; all LEGACY/ACTIVE rows pin it. The actual local `check`
  matches all25 entries. All22 synthetic tests passed inside accepted AFTER v3
  job3875239. The exact accepted governance bytes are committed as
  `7db322a313c3bcda1c0628e64b857ed30496e651` and pushed to
  `origin/codex/math-benchmark-suite`, so a fresh checkout of that branch now
  inherits them. `precreate` is now hard-disabled
  before any state read, and every ACTIVE row is rejected regardless of the
  LEGACY count; Git output remains buffered before its4MiB validation cap.
  A raw Git command can still create one before detection; it is a hard
  workflow violation, not a supported bypass or an automatically reversible
  operation.

- UPDATE parent-seal submission2026-10-01: the HPC private route recovered.
  A fresh queue/accounting/path reconciliation proved an empty queue, no prior
  parent-seal attempt and no existing campaign/ledger/stage. Exactly one stage,
  `/public/home/majj/atlas-rust-campaign-20260930/stages/weyl-parent-seal-v1`,
  was created and the frozen stager submitted exactly job3872554 with
  `queue_before=[]`. The raw pin SHA is
  `b52e91d827f39b7d290ab0447f82e251e979e46fdd440b61371fd5599973cf96`;
  ledger/intent/receipt SHAs are respectively
  `97690b3555a0bc70050240a2499e7781b33b4f8e1c968356f861f24e10e1c706`,
  `0c2a6b6b13404a7a4583c4f1f2c35c32d1ba07924a44b1cb12f4b05be83c5147`
  and `6ced68fcf857ab4df3a5d12eb1f603e1a4257c18b4e23de11108dbb8bd7d6742`.
  Historical submission-time observation had it RUNNING on cu034. Do not
  create a sibling stage or resubmit that accepted logical attempt.
  Post-submit inspection found the14 immutable staged `hpc`
  payload files as single-link0444 regular files and no `legacy/`, remote tar or
  `.transport-incoming`; retain the exact running result workspace until FINAL.
  The bootstrap package bytes were frozen locally, but bootstrap provenance,
  ownership/retirement registration and remote installation evidence were not
  pre-frozen: the local source tar
  `/tmp/atlas-parent-seal-overrides.lifecycle-v6.tar` (225280 bytes, SHA
  `8347c733555b1e54c9798a6006636beb6be22e2c03483c10172625f6875d9b6a`)
  and unpacked sibling were not pre-registered with owner/retirement under hard
  rule9. This is a lifecycle/provenance deviation, not a known byte-integrity
  failure: the local tar was hashed; the remote15-file member allowlist and all
  payload hashes were revalidated before same-filesystem atomic installation.
  Record both local paths as `RETIREMENT_PENDING_EXPLICIT_AUTHORIZATION`; do
  not call the unpacked copy another active transport and do not delete either
  without the exact safety audit and user authorization.

- UPDATE parent-seal acceptance2026-10-01: `sacct` records job3872554
  `COMPLETED 0:0` on cu034 in33m40s. Independent inspection accepts the
  infrastructure migration only: report
  `939482542c123a6a2a2c22f8dbe1e052af20dee3af4010da780fe958dc41bf79`
  has status `WEYL_CONTEXT_PARENT_SEALED`; all12 commands and64 checker tests
  pass, both complete legacy traces are identical, the inventories are629/519,
  and the33143-file closure has4355 unique objects. Seal object
  `db67234c0d67dbd6a6f0327386dd113b6093d25a46569710c33dfc8bc482cdcb`
  and1558-file source object
  `5133bb32da7e5a92775d5f56ea2035680c363b40f085a8af95eaeebfdbc4536b`
  were rehashed in CAS; the report, command artifacts, ledger/intent/receipt,
  one-stage filesystem and absence of the ephemeral workspace were inspected.
  Evidence metadata is
  `tests/reference/hpc/math_weyl_parent_seal_2026_10_01.json`. This is not a
  mathematical, rank, root-ladder-repair, performance or parallel acceptance.
  The next gate is the exact seal-only ladder-boundary BEFORE v2 child snapshot:
  the parent launcher is closed and only that child stager/driver is enabled.
  Run65 checkers, require521/630 inventories and exactly the2domain+1core
  expected failures before accepting it.

- UPDATE parent-seal transport2026-10-01: all earlier14-payload transports
  through lifecycle-v4 are STALE, and lifecycle-v5 was rejected by static
  review before transport. The replacement lifecycle-v6 exact manifest is
  `0aac6e735c0ae7c7e0da4773c46e0d8f116b92b68a73797b05e886ea342a2620`
  and its deterministic tar is
  `8347c733555b1e54c9798a6006636beb6be22e2c03483c10172625f6875d9b6a`.
  All14 payload manifest entries were rehashed and byte-compared with the
  workspace; this proves local packaging integrity only, not the separately
  operator-observed remote member/hash checks, HPC checker execution or
  migration acceptance.
  Unknown scalar or reference-shaped seal fields are now rejected, so a later
  reachability claim cannot omit a hidden reference; the parent-seal unit-test
  method count is16 and the full migration checker count is64. The active
  root is hard-pinned to campaign20260930; result workspaces must belong to it,
  and home-resident SLURM scratch is rejected. `submit_one` opens every script
  path component with dirfd+NOFOLLOW, rejects nested `sbatch`/option-shaped
  names, then submits the same checked snapshot on stdin. The parent stager
  fully validates parent, overrides and any prepared state before its lock and
  repeats under lock. Lock, ledger, pin, intent, receipt and result-directory
  handling now use no-follow, stable-file checks; CAS references require exact
  path-free schemas. The exact raw pin SHA is durable in ledger and intent
  before `sbatch`; at submission time the unique running compute job could
  finish only its own exact uncertain/partial receipt state without a
  resubmission, while final preflight was validation-only. Job3872554 and the
  independent inspection now satisfy that bounded execution gate. Residual P2
  threat model: lock-file
  inodes are pinned, but state I/O still uses pathnames, so a hostile same-UID
  process that renames an entire stage/campaign directory during the critical
  section could split the lock and data domains. The controlled workflow never
  renames those directories; a future hostile-concurrency hardening must use
  dirfd-relative state I/O throughout.
  Historical note, superseded by the submission update above: six read-only
  SSH attempts timed out before authentication/remote output, so at that time
  queue capacity, campaign existence and capture availability were UNKNOWN.
  The sixth attempt was at2026-09-30T20:11:09Z and is recorded in
  `tests/reference/hpc/hpc_connectivity_2026_10_01.json`. No remote directory,
  submission intent or job was created. Rebuild or reuse
  these exact bytes, first reconcile the remote queue and same campaign stage,
  and never infer a free slot or create a second stage from the timeout.
  A historical route check then showed10.26.14.64 going through ordinary
  Wi-Fi gateway192.168.3.1 on `wlp0s20f3`, with no HPC private-tunnel route;
  the later recovered `tun0` route and exact reconciliation supersede that
  connectivity state, not any frozen evidence.
  The legacy absolute-parent ladder-BEFORE contract is retired. A seal-only
  replacement is locally implemented and independently static-reviewed. The
  accepted parent transition closed the parent launcher. V2 job3872594 then
  failed at63/65 checker outcomes before Cargo/inventories/math because of two
  harness defects; report `4ee3b6cd...` is retained and grants no math result.
  The repaired launcher/driver targeted `ladder-boundary-before-v3` while keeping
  the structurally unchanged v2 schemas. Its
  sole durable parent reference is the parent-seal digest and byte count; its
  driver binds `SLURM_JOB_ID` to the unique confirmed campaign ledger/intent
  before creating a result directory. BEFORE v3 job3873400 is now FINAL
  `COMPLETED 0:0` on cu006 in5m36s. Independent inspection accepts its exact
  tests-first BEFORE evidence: all65 checker tests (`5+10+6+5+24+15`) pass,
  the inventories are521/630, and the unchanged Rust production source has
  exactly2domain+1core named failures with0 ignored. All12 command-artifact
  hashes,20 pinned inputs, parent/source CAS objects and final integrity were
  rechecked; the ephemeral source/target workspace is absent. Report SHA is
  `51cc7a14a0dbb8188a4ea47705338e5689212bf1f707819bab8c4e5681e4052b`;
  evidence is
  `tests/reference/hpc/math_ladder_boundary_before_v3_2026_10_01.json`.
  This releases only the minimal overflow-membership repair, not an AFTER,
  mathematical pass, performance result, parallel result or rank escalation.
  Do not resubmit BEFORE v3 job3873400. The five-test
  two-snapshot allowlist,24-test mathematical-
  acceptance checker and its index/full-deform/cycle evidence are already
  ladder-only inputs and are excluded from both `SEALED_SHARED_INPUTS` maps.
  Final independent static review and execution found no blocking harness
  issue. Keep this exact child stage immutable; do not change its bytes, open
  a sibling/retry stage or add child files to the frozen lifecycle-v6 parent
  transport.

- HISTORICAL UPDATE directory-governance AFTER v1 candidate2026-10-01: the prepared
  ladder AFTER v1 gate now carries the exact registry plus
  `local_worktree_guard.py` and its20 synthetic tests as child-only inputs.
  The transition allowlist remains5 tests, but freezes all70 current
  `stage_*.py` names and mutation-rejects addition, removal, reactivation and
  duplicate `main`. The compute driver materializes the accepted parent source
  only in its auto-cleaned workspace, overlays current parent/BEFORE/AFTER
  sources in memory, and runs a separate timed/hash-recorded check that all67
  ordinary historical stagers have an immediate unconditional `SystemExit`.
  It does not install those67 launchers in the durable stage. The manifest
  scans fixed `docs`/`hpc`/`tests` roots, and the isolated inventory process is
  exactly `python -I -S -B -c`, preventing environment imports and pycache
  drift. AFTER now has88 unittest checks plus that full-inventory command.
  These are prepared/static facts only until the exact one-job HPC gate and
  independent review pass.

- UPDATE ladder AFTER submission2026-10-01: fresh reconciliation found an
  empty queue, no accounting/ledger match and absent exact stage. The reviewed
  27-file manifest `2b4733b6...de9be` was installed only at
  `atlas-rust-campaign-20260930/stages/ladder-boundary-after-v1`; the top-level
  `atlas-rust*` directory count remains318. Shared submission accepted exactly
  job3873497 with `queue_before=[]` and pin
  `200055d52c0d9559a2edf4b7e556296d0c8bd0df129030e4e855e4e4426df289`.
  Post-submit state has27 staged0444/single-link inputs, empty `.incoming`,
  intent `8a620953...fc29`, receipt `cf7ce26c...efb56e` and campaign ledger
  `d78c75c4...5f23d`. The job was RUNNING when observed. At that snapshot its
  status was `SUBMITTED_NOT_VERIFIED`: do not duplicate, resubmit, accept the repair,
  delete the stage or release any rank/operation gate before FINAL independent
  inspection.

- UPDATE ladder AFTER v1 final/directory guard2026-10-01: job3873497 is FINAL
  `FAILED|1:0` with immutable report
  `fec90eb2f4cf0a4dff4820c4527c5263c151f8dff4aabf272007618646149e15`.
  All88 v1 checker outcomes and two toolchain commands passed, but the first
  full-stager inventory failed because the accepted source CAS contains zero
  historical stagers; Cargo, patches and math were NOT_REACHED. Preserve v1 as
  `HARNESS_FAILURE`, not a mathematical failure or pass. The changed-input v2
  successor remained inside the same active campaign. Its
  path-free retired-stager CAS design removes that false dependency. The local
  guard now freezes the exact24 audited LEGACY identities and has22 prepared
  synthetic tests; planned v2 checker total is93. This blocks a novel worktree
  being laundered as LEGACY, but remains a cooperative detector. A future
  ACTIVE worktree is forbidden until durable creation-receipt semantics are
  implemented and HPC-verified. No new top-level `atlas-rust*` directory was
  created by this correction.

- UPDATE ladder AFTER v2 submission2026-10-01: static review aligns the exact
  93-checker driver/stager/test boundary. The28-file override manifest is
  `b3e4fa16cdb9e86aba36d4bdd5723401a32b0c07ef21cdce29165dd938daa699`;
  the existing campaign CAS now contains the exact67-source retired-stager
  bundle `550b1330...e9b09`/311925 bytes as0444/single-link, with both upload
  scratch files removed. After a fresh empty-queue/accounting/ledger/path
  reconciliation, the exact child
  `atlas-rust-campaign-20260930/stages/ladder-boundary-after-v2` was created and
  shared submission accepted exactly job3874203 with `queue_before=[]`. Pin,
  intent, receipt and campaign-ledger SHAs are respectively
  `3f0f3c8b79e7ab7688ec329e242d18a82ac1b113eb33fa3788cfe1ca3e31455d`,
  `bf13ee5fa7f96b9846a2d000018a2951b1ef7dd6ae208e977fd2b3eb4e62c6a9`,
  `6c1e342a1e59f67bdfaa611acebcb9913c581ff3a06e5c9b7b8488c3127d8672`
  and `44624c765f57222be339cfaa62bbdaff82f6208a93e78d3fdc7c60e7aba1835d`.
  All28 stage inputs are0444/single-link, `.incoming` is empty, and the
  top-level `atlas-rust*` directory count remains318. Job3874203 is now FINAL
  `FAILED|1:0` after4m55s on cu105, with immutable report
  `256247b2be5fd7358ccb56e87ff262c6c0e87ca782e822e2f3c96419c4b4a694`.
  All93 checker tests, the70/67 full-stager inventory, domain inventory521,
  both focused boundary tests and the full521-test domain suite passed. The
  driver nevertheless returned `HARNESS_FAILURE`: `--nocapture` exposed the
  expected panic hook from the passing `poisoned_kl_cache_returns_a_stable_error`
  catch-unwind test, and `passing_log` incorrectly rejected any `panicked at`
  substring. Core inventory/regression/full suite and final integrity were
  NOT_REACHED. Evidence
  `tests/reference/hpc/math_ladder_boundary_after_v2_failure_2026_10_01.json`
  SHA `e757f56ce6da55f1b96978daaaa2156b82b1bc3f83b36c2971f7a7a66be1f8f1`.
  This is not a mathematical failure or acceptance. Do not modify/resubmit v2,
  create an unrecorded sibling/top-level directory, or release any gate.
  The only admissible successor is a reviewed changed-input AFTER v3 child: retain
  strict panic rejection, remove `--nocapture` only from full-suite commands,
  scrub `RUST_TEST_NOCAPTURE`, bind the exact v2 failure evidence and rerun
  domain+core inventories/focused/full/final integrity from scratch. At this
  v2-final snapshot it was not prepared, created or submitted.

- UPDATE ladder AFTER v3 pre-creation2026-10-01T08:51:40Z: the exact candidate
  target is
  `/public/home/majj/atlas-rust-campaign-20260930/stages/ladder-boundary-after-v3`.
  Its direct predecessor is immutable v2 job3874203, evidence
  `e757f56ce6da55f1b96978daaaa2156b82b1bc3f83b36c2971f7a7a66be1f8f1`
  and report
  `256247b2be5fd7358ccb56e87ff262c6c0e87ca782e822e2f3c96419c4b4a694`.
  The candidate29-input manifest is
  `tests/reference/hpc/math_ladder_boundary_after_v3_overrides_2026_10_01.json`,
  SHA `3fe6c8c5c0a0638ae8d128313ad3bfce02bc97e3d16a69f75670a864b28695c2`.
  Seven v2 inputs changed and the v2 failure evidence was added: full suites
  capture output without nocapture and scrub inherited `RUST_TEST_NOCAPTURE`;
  the local guard hard-disables ACTIVE/precreate, freezes exact24 LEGACY and
  double-samples Git/no-follow sibling/admin identity; pin/receipt publication
  now exact-validates confirmed records, detaches nested JSON and pins the
  full-stager program. Parent seal `db67234c...2cdcb`, retired bundle
  `550b1330...e9b09`/311925 and both math patches are unchanged. Retention is
  `ACTIVE_GATE_COMPACT`; retirement requires FINAL inspection, acceptance-index
  disposition, no live/uncertain ownership, verified CAS closure and separate
  exact-path deletion authorization. Read-only remote reconciliation found an
  empty queue, no AFTER v3 accounting row, the exact path absent and the existing
  five-record ledger unchanged at SHA `44624c76...1835d`. No AFTER v3 pin, stage,
  intent, receipt or job existed at that checkpoint. The sole launch authorized
  by that record was one non-array, no-dependency cpu job (2CPU/8GiB/2h) after
  final static review.

- UPDATE ladder AFTER v3 submission2026-10-01T09:08:55Z: final static review
  found no blocker; all29 manifest paths/hashes and95 checker methods agreed.
  A fresh remote reconciliation again found an empty queue, no AFTER v3
  accounting row, absent exact child and the unchanged five-record ledger.
  Only the registered child above was created. The shared stager accepted
  exactly job3875239 with `queue_before=[]`, pin
  `7af79aa20b37d0fcee9a60d8e3c6b4dbec05417cf6817747520b6ebe3ec41f88`,
  intent `feff9e66a0317719258db7e210325a109e263d1447790b71d4a444c33f79a0ab`,
  receipt `ac1f6f52f99de9b86b7b7bbfe56e7478809f193f1938a0c07219bc6061ee2254`
  and six-record ledger
  `a1776412e7dd6633c329743b52c5cf4b1615dde8a7a2f0b4fddd002b5615a618`.
  All29 installed inputs and the retained override copy are0444/single-link;
  the manifest is0444/single-link. Job3875239 was RUNNING on cu105 when
  observed. The HPC home still has exactly318 top-level `atlas-rust*`
  directories: this operation added only the pre-recorded stage child under
  the existing campaign. Status is `SUBMITTED_NOT_VERIFIED`; do not resubmit,
  create a sibling, claim the repair accepted or release any rank/operation
  gate before FINAL independent inspection.

- UPDATE ladder AFTER v3 acceptance2026-10-01: job3875239 is FINAL
  `COMPLETED|0:0` after10m18s on cu105. Independent inspection accepts the
  exact bounded gate: all95 checkers (`5+10+6+22+7+24+21`), full-stager70/67,
  domain inventory/focused/full521/2/521 and core inventory/focused/full
  630/1/630 pass; all16 commands exit zero and all32 log/time artifacts rehash.
  Final report SHA is
  `771fc790dd4340f408235e50c3f6eee754ebe4c850cbad36d4af4f902a24627c`;
  independent evidence
  `tests/reference/hpc/math_ladder_boundary_after_v3_2026_10_01.json` has SHA
  `a459fa08117ff8a721181d349467ebdd9267e798cd25b5bcb1380996c2d15e15`.
  Source/final integrity pass, zero legacy paths were opened, `.incoming` is
  empty and the ephemeral source/Cargo workspace is absent. All29 installed
  and retained override payloads match the frozen manifest. The durable stage
  is only3220KiB allocated and the HPC home still has exactly318 top-level
  `atlas-rust*` directories. This validates the minimal root/coroot ladder
  repair and cooperative directory guard only; acceptance-index disposition,
  a focused commit/push and any exact-path retirement authorization remain
  separate. No historical directory was deleted and no rank/performance/KLV/
  unitarity/Hodge/associated-cycle/AV-ann gate is released.

- UPDATE upstream refresh2026-10-01: a fresh official GitHub `ls-remote`
  through2026-10-01T01:46:33Z still resolves both HEAD and master to
  `7e1b958c7aa9456769cc9cf09ac1542814b4800a`. The existing oracle pin is
  therefore current; no source or oracle binary was changed. Receipt:
  `tests/reference/hpc/upstream_head_2026_10_01.json`. This does not refresh
  the347 legacy metadata records still pinned to `4d3e9449`: the official
  comparison is494 commits/257 paths, including interpreter sources and the
  unitarity/Hodge/KLV/AV-ann scripts. Preserve those historical pins and later
  append a complete HPC replay against current master; never relabel them in
  place. Inventory: `tests/reference/hpc/upstream_compare_2026_10_01.json`.
  UPDATE upstream moved2026-10-09: a fresh `ls-remote` at
  2026-10-09T13:14:20Z resolves HEAD/master to
  `5ae51193cbb8faa022847701195bba81702b1605`, ten commits ahead of the oracle
  pin (`7e1b958c`).  The compare inventory (20 files: atlas-types.w 19+/12-,
  rootdata.cpp 1+/1- const-only, repr.cpp 85+/35- integer-0 wrapper fixes,
  matrix.cpp/h empty-vector UB robustness, new cell_graph.at, kgb.cpp/h,
  gradings, basic.at, readline/macOS main.w) is
  `tests/reference/hpc/upstream_compare_2026_10_09.json`; receipt
  `tests/reference/hpc/upstream_head_2026_10_09.json`.  Hunk-by-hunk check:
  none of the sections cited by the G2 pre-registration slice
  (`docs/slices/weyl_g2_preregistration_2026_10_09.md`) is in the diff, so
  the pin-level reading stands.  The pin stays `7e1b958c`; a refresh is a
  separate reviewed transition requiring a complete HPC replay against
  current master — never relabel historical pins.  The KGB_cross/Cayley/
  status integer-0 fixes mean a future refresh CAN change edge-case oracle
  outputs: existing goldens stay bound to their pin.

- UPDATE language-corpus manifest2026-10-01: the frozen transition inventory
  contains353 `.atlas` fixtures but exactly346 executable fixture/event/meta
  triplets;342 are ordinary no-write cases, three additional path-sensitive
  cases must also produce zero writable-tree delta, and only
  `eval/file_commands_b9` may create its one exact private-`/tmp` output.
  `hpc/language_corpus_manifest.py` SHA
  `48368e2acdc92fb03d41a37f8277ad279b2cfed0fed7c44486e40c3c98791877`
  and `hpc/test_language_corpus_manifest.py` SHA
  `3cfa0f96a98c9df89b0bec27cfc1f606f14e4b16e5888fe67f9ec475ea3d97b5`
  are static-reviewed with no P0/P1 finding, but have not run on HPC. Their
  sole status is `FROZEN_INPUT_INVENTORY_ONLY`; no manifest JSON exists and
  no compatibility claim follows. Keep the helper disabled and outside the
  frozen parent transport until parent seal and ladder AFTER are accepted and
  a campaign-bound, fail-closed filesystem-isolation stage exists. Exact
  boundary and two-job replay plan:
  `docs/slices/language_corpus_transition_2026-10-01.md`.

- UPDATE rank1-unitarity independent-review draft2026-10-01:
  `hpc/math_unitarity_forms_rank1_review.py` SHA
  `8dfd50f556fd928bf6708ed4b9b453d229b8e755e808571b2221122052ec951f`
  and its20-test synthetic checker SHA
  `9c2f656b6a381febe778b7c565d9409743ef75077af429c9422e8a75471d74e4`
  independently reconstruct/parse the frozen3856011 six-form history without
  importing either capture classifier. The bounded contract is6 forms,
  30gamma inventories,46parameters,46final constituents,6empty sets,
  0nonstandard and0zero rows, all36 raw artifacts plus source/binary/parent
  provenance. This is prepared code only: `REVIEW_ENABLED=False` is the first
  `main()` gate, no test/review has run, and no ledger entry exists. It binds
  historical source only, not the current candidate or a rank gate. Keep it
  disabled until the parent seal is FINAL and a reviewed campaign/CAS/ledger
  adapter replaces every legacy absolute-path read and unsafe result write.

- UPDATE discovery3868832 FINALbdee1281: original accepts all11 A1+torus
  coordinate-boundary cases and emits22 ladder rows; Rust emits10 then rejects
  six cases with `root-system arithmetic overflow`. This is a real calculation
  error: representable opposite roots can have a nonrepresentable i32
  difference, but such a difference is necessarily absent from the stored
  i32-root set and must not abort membership testing. Full original stdout/
  stderr are frozen and two kernel plus one full-stream regression are added,
  still unverified until the HPC BEFORE gate proves the three expected
  failures. The other two Weyl-context drafts used script-only `matrix` without
  loading scripts; their frozen statuses are respectively
  ORIGINAL_PROVISIONAL_INTENT_NOT_CONFIRMED and ORIGINAL_HISTORY_INCOMPLETE.
  Original Program and Rust Name rejection categories are discovery evidence
  only, never positive cache validation. Original source review at frozen
  `7e1b958c` confirms its RootSystem constructs ladder bottoms in compressed
  abstract simple-root coordinates before ambient lattice roots/coroots exist;
  torus coordinates never enter this subtraction. Repair only
  `build_ladder_bottoms`: an unrepresentable exact difference cannot equal any
  stored `i32` root and therefore means non-membership, not failure. Preserve
  allocation errors and the `i128` reflection path; do not use wrapping or
  saturating arithmetic, and keep `combine_roots` out until separately covered.

- UPDATE profile3868803 FINAL1d2baadd confirms post-cache root-ladder SELF
  16.18-16.25%U/12.75-14.14%AV, mainly build_weyl_context; Type::equivalent
  now1.66-3.29%.23checks/6controls/4hash-verified samples,0lost. Select Weyl
  rebuilds next, but percentages are short diagnostic samples, not speedup.
  Historical profile-time rule: the then-pending coordinate-boundary
  discovery had to obtain original acceptance before any cache/math repair;
  FINAL3868832 has since supplied that acceptance, while the unchanged Rust
  regressions still await their HPC BEFORE gate.

- UPDATE command AFTER R23868782 FINALfbc615d2 accepted,46checker/
  13focused+ALL629core/26complete histories/72retained/48serialAB/final
  integrity. Ufull3.241245->2.538780s(-21.67%), AVfinite4.307805->3.208666s
  (-25.52%); still9.15971x/7.53406xoriginal. MedianRSS+0.41%/+0.51%.
  Runtime d9d43755/typed7085d231/types53264489; not kernel/parallel/general
  mathematical acceptance. Keep3shared Hodge failures/rejected diagnostics.
  Reprofile this accepted source before selecting another optimization.

- Weyl reuse caller: original elliptic.at128-135 binds one adjoint datum then
  computes W_elt(rd,w).matrix.char_poly repeatedly (3/9/5/12/30 exceptional
  words). It retains vectors, not WeylElts; a Weak-only context cache can
  expire between calls. Exact original atlas-types.w977-1152 weak-interns equal
  root data, retains one strong lazy WeylGroup identity per live datum, and
  fills the canonical dual's identity only when still empty. The former single
  `WeylKernel {RootSystem,WeylInterface}` plan is superseded: RootSystem is an
  owner-local coordinate cache, while WeylInterface belongs to a separately
  weak-interned abstract group identity. Cold dual shares that identity;
  prewarmed dual does not. Weyl compatibility is abstract-group Arc identity,
  never structural handle or coordinate-cache identity. Cross-coordinate
  equality/product must replay external-generator words in the left system;
  never directly compare/compose foreign root permutations. Binary `=`/`!=`
  need a fallible relation path and must check compatibility even at no-value
  evaluation. Preserve structural RootDatum Eq/Debug and W_elt validation
  order; cache only successful complete objects with current-call diagnostics.
  No production edit may precede the tests-first semantic BEFORE described
  below. UPDATE2026-10-02: BEFORE-v4 job3886748 retained that BEFORE proof, and
  the minimal repair is now implemented locally as an UNVERIFIED candidate
  (patch `hpc/patches/weyl_context_core_repair.patch` SHA
  `246cd2d0dd48ee68387ed8f72a10a156e43b7d8c8c6dcb693474832450111c5c`, verified
  only to reconstruct the working-tree bytes from the accepted baseline
  `0359261f`/`7085d231`). The candidate interns every RootDatumHandle by full
  content plus preference into a weak registry; each identity lazily owns one
  coordinate kernel and one abstract group; `dual` shares the group only into
  a cold target; Weyl `=`/`!=`/`*` check abstract-group `Arc` identity before
  the no-value gate and replay the right word in the left system. It is NOT
  committed and grants nothing until its changed-input AFTER gate passes. The
  AFTER full-suite run must keep the two A1 fixture tests out of one shared
  parallel process (run them with the serial selector, or `--skip` them in the
  parallel suite), because process-global interning makes them interfere.
  SUPERSEDED 2026-10-06: the candidate passed its changed-input AFTER gate
  (after-v5 job3900050, FINAL `COMPLETED 0:0`, both regressions pass) and the
  validated tree is landed as production commit `690c2b92`; see the frontier
  and the AFTER-v5/production-commit UPDATEs under "Predecessor transitions".
  Capture stages v1 through v7 are immutable harness failures, not
  mathematical evidence. The latest is v7
  job3884780, FINAL `FAILED 1:0`: the 32-test stage-creation suite passed, then
  four of the 17 progressive-submit tests errored because their synthetic
  creation receipt omitted `stage_device` and `stage_inode`. The real
  descriptor-bound creator receipt contained both fields and submitted
  successfully. Contract/capture/allowlist suites, build and both Atlas
  executables were not reached; zero Atlas invocations/captures mean no
  mathematical regression is required. The independently reconciled v7 tree
  is SHA
  `32a44f0679e3220a47ed5ed0dcad7d8a750de3610cd375ec4b05b3677131c8d8`;
  its 14-record ledger is SHA
  `11c0b626ad22e56c68f1e7d7c0f4d7336574b1ca3afb1374d6d33de06ec55b5e`.
  Both exact v7 transports are removed and no transport is active.

  V8/job3884807 was submitted exactly once and is FINAL `COMPLETED 0:0`; all
  102 checker tests and the release build passed. The independent inspection
  is PASS and classifies the complete fresh-process evidence as
  `CAPTURE_COMPLETE_RUST_SEMANTIC_MISMATCH_CONFIRMED_REGRESSION_REQUIRED`.
  Inspection SHA is
  `b2c7f4ece709df3506c3224a57abad896f3ecfcd6625fea97731619f629e1da3`.
  Cold canonical duals are compatible in the original in both
  dual-construction directions (SC-to-adjoint and adjoint-to-SC): equality is
  true, inequality false and identity products succeed; Rust returns
  false/true and rejects both products. Reverse operand orders remain pending.
  Independently prewarmed owners are incompatible: original equality,
  inequality and multiplication all reject before values, whereas Rust
  incorrectly returns false/true for the relations and rejects only products.

  Preserve the four byte-exact original goldens. Their hashes are cold stdout
  `7a614e47b75469c441e774cbe46769dcd769c5cc0a2a31cf1868ff9b59a780dc`,
  cold stderr
  `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`,
  prewarmed stdout
  `5074aab3290d4ae404bb7abb0b24ccc99036f616abec518c6479e59ec2db021f`
  and prewarmed stderr
  `ef4404d85f7252a611f9f4e4a5205fe6bbac2c66f48b7378a30a8053f986e157`.
  The regression catalog SHA is
  `6a9f960753bcd2afb575cb5207d4310f21a5694505d02a83f4e4142211eaa634`;
  two focused tests in `crates/atlas-core/src/session.rs` SHA
  `969cdb27ccae61ca4fe3e36219801037d475517bee3614fead553024818307e7`
  and the tests-only patch SHA
  `ccd3009892dbeae2f146dff9924efa6d6bede3dca910d026c8f3ab3e1b4c49cf`
  are frozen but unexecuted. A changed-input HPC BEFORE must prove exact
  original/golden agreement and the unchanged Rust failures before any
  production repair. The same regressions must pass AFTER. V8 grants no
  math/cache/performance/memory/rank acceptance; its 345 seconds were dominated
  by a 281.53-second build and its one-shot math invocations cannot yield a
  speed ratio. The inspected queue is empty, both exact transports are absent,
  and v8 must never be invoked again or cloned as a retry sibling.

  After the A1 semantic AFTER passes, release G2
  (the asymmetric interface-order witness), B2/C2, reverse operand orders,
  inner-class-dual and no-value cases progressively; all must pass their own
  original-backed semantic AFTER gates before the cache work-count BEFORE or
  any production cache edit. A1 is not a cross-coordinate proof: its dual
  products are only `s0*s0`, an alias still keeps the old datum alive after
  rebind, and language output cannot prove internal cache-cell topology. Add
  noncommuting word/action checks, a sole-WeylElt lifetime case and HPC-only
  Weak/work-count guards later; do not mutate the frozen A1 capture.
  Original inner-class build eagerly calls canonical `dual()` and retains both
  datum owners, while Rust currently retains only the primal handle and
  reconstructs a dual handle. Route inner-class construction through the same
  future dual-identity path; fixing explicit `dual(RootDatum)` alone is
  insufficient. The existing Rust `CompactWeyl` `[u8;32]` representation is a
  separate later element-layout target, not part of this semantic/cache patch.
  Static source reuse is not measured cache acceptance or a speed/memory claim.

- Historical ladder-boundary rule (now discharged by discovery3868832):
  root_datum reflection uses i128 intermediates, but root_system ladder
  membership subtracts i32 coordinates. Representable opposite roots can have
  an unrepresentable difference. The fixture was provisional and could not be
  treated as truth until complete original behavior was captured. 3868832 now
  proves original acceptance, and the failing regression must pass before any
  repair is accepted. Lazy tables also change constructor-time error behavior
  and derived traits; do not infer that dropping eager work is automatically
  semantics-preserving.

- UPDATE command AFTER3868767 FINAL13116add is HARNESS_FAILURE after all
  629core/13focused pass: observe was not imported, so ZERO fresh histories/
  retained/A-B ran. Keep the failed report; unit success is not differential
  or performance acceptance. R2 imports math_suite.observe and adds deferred
  global-resolution plus missing-import fault-injection checks (46checker),
  with unchanged runtime/tests/goldens and all26/72/48 gates in a fresh stage.
  Never edit the frozen failed stage or reuse the stale3aea9ce7profile package.

- UPDATE command BEFORE3868749 FINAL3fd3aba3:22checks/622compiled inventory,
  exactly2work assertionFAIL (3views versus1),4semantic/original controlsPASS,
  unchanged production and final integrity. Only then implemented candidate
  d9d43755: per-OverloadState Rc signature views, name-local invalidation on
  every successful add/replacement/remove, owned Arc type revision guarding
  cross-command reuse. No Value/inference state cache. TypeTable revision
  detaches on add/update/reused simple slot/successful forget; constructor/
  recursive paths go through add/update. Preserve structural Eq, Debug and
  Send+Sync; transactional overload clones have independent empty caches.
  Existing tests unchanged,7new lifecycle guards;44checker/629core/
  26original histories/72retained/48AB AFTER required before acceptance.

- UPDATE command discovery3868735 FINAL9d2b560d:28checks/2histories,
  original accepted positive full stdout/stderr match. Negative full stdout
  matches, but original Program versus Rust Name+Type envelopes stay distinct;
  exactly failed '+'(int,bool) and undefined oc3n_dispatch must be retained.
  Four original raw goldens preserved. Six tests-only additions (622inventory)
  now target two 3-versus-1 cross-command view work failures plus four semantic
  controls; no cross-command runtime cache implemented. The old direct
  OverloadState literal is replaced with Default+forgotten assignment ONLY
  in cfg(test), preserving all old assertions before freezing the before gate.

- UPDATE completion AFTER3868661 FINAL14a45971 accepted on cu315:53checker,
  11focused+ALL616core actually PASS,5completion/19old histories/72retained/
  48fresh serial AB and final source/artifact integrity. Lazy snapshot patch
  cc397f42 is now accepted in this bounded scope. Same-node Ufull4.022098->
  3.232297s (1.24435x), AVfinite5.541168->4.314198s (1.28440x); still
  11.6017x/10.0341x original end-to-end time. RSS essentially unchanged.
  Do not multiply ratios from earlier nodes or call these kernel/multicore/
  60-600s results. Three shared Hodge errors and distinct negative diagnostics
  remain retained failures, not mathematical acceptance. Next measured target
  is cross-command merged overload views (~0.84/1.27s in3868525); obtain the
  two new original histories before a cache, preserve fresh polymorphic scopes
  and invalidate on result-type replacement and same-length TypeTable changes.

- UPDATE completion BEFORE3868646 FINAL631f71ea proves all4new complete-
  stream assertions FAIL after verified setup,2old controls PASS,611compiled
  inventory,31checkers and final source/artifact integrity. Valid constructor
  companion accepted by both, original nonempty type/field names versus Rust
  empty. Only then wrote candidate: session-owned lexical-order index plus
  OnceCell<Vec<String>>, Rc<str> shared names; invalidate only active membership
  changes, never eagerly refresh per command. Four old test sections unchanged,
  five new work/ownership tests. Patchcc397f42, typed4b422d9e, not after-verified.
  Preserve sequential vs parallel binding and simple-alias partial publication;
  grouped type publication remains transactional. New53checker/616core/
  5completion+19old histories/72retained/48AB gate must pass before a speed claim.

- UPDATE completion3868572 FINAL2bd59d1d:23checks/4histories/8processes,
  unchanged production and final integrity. History/visibility are original-
  accepted but differ: forgotten builtin succ remains visible in Rust, and
  local-first lexical order is lost. Failed-name revival has the same order
  defect; retain original Program+Type versus Rust Runtime+Type envelopes.
  Type aliases/fields are missing in Rust. The original types draft is
  REJECTED_SYNTAX (>= token and missing !), not a positive pass: preserve it
  and its accepted survivors; separately capture `set_type Row<T> = [T] !`.
  All8original raw streams are saved unchanged; four session regressions
  added before any runtime fix. Pending before gate must show four actual
  stream assertion failures after setup plus two old controls. Do not keep
  all startup builtins permanently visible: only pre-first_identifier names
  are unconditional in original buffer.w. See performance slice/HANDOFF.

- UPDATE targeted timers3868525 FINAL0855b34b inspected:21checks/4original
  controls/16on-off/source+artifact integrity PASS. Uload3010refreshes copies
  3.44million names in~0.742s; AVload3658copies4.90million in~1.178s; BOTH
  have ZERO completion queries. Completion.note is only~5-7ms, not dominant.
  Next selected target is unused completion snapshots, after original history
  capture. Merged views still~0.84/1.27s; E8Weyl context repeats30times~0.346s.
  Preserve these independent targets; Weyl/ladder timers overlap, not additive.
  Probe overhead~0.60-1.37%, not a new speedup. Transport recovered without
  rerunning the completed job. See performance slice for full counts/bounds.

- Completion order is original hash/first-LEXICAL-use order, not necessarily
  first successful definition: lexer.w468 interns every identifier, and
  buffer.w1174 traverses that table, filtering by active binding/overload.
  global.w168 present() also includes defined types. Current Rust completion
  order tracks successful variable/overload definitions and misses set_type
  publication paths. New4completion_incremental histories are prepared only;
  obtain full original outputs before claiming discrepancies or changing this.
  Preserve local/failed-name revival, type/member names and sequential versus
  parallel visibility. Don't optimize a mistaken first-definition contract.

- UPDATE post-view3868496 FINAL24ab240d:22checks/6unprofiled controls/4perf
  captures/source+artifact integrity PASS. Same90accepted files. Self samples
  still show merged-view Type::equivalent11.25-12.73%, root ladders7.25-10.55%,
  completion refresh7.25-9.60%. Ladder caller is mainly build_weyl_context;
  do not prioritize eager dual classification or TypeScheme/Value copies based
  on source speculation alone. Rust rebuilds completion lists per command;
  original traverses tables on queries. Preserve completion history/order and
  visibility, measure isolated on/off timer overhead before changing it.
  Full source attribution and four raw hashes are in loading-performance slice.
  Pending3868496 notes below are historical. No rank escalation or new speedup
  follows from diagnostic timings; never add inclusive Weyl and ladder clocks.

- UPDATE view3868418 FINALe6258384, cu03317m21:46checkers,6focused+ALL607core
  actually executed,19original histories,72retained streams,48ABprocesses and
  final source/artifact integrity PASS. Local typedabf07725 is now accepted
  in this bounded scope. Same-node fullU4.630245->4.013848s (1.15357x),
  finiteAV8.737303->5.567626s (1.56931x), all4repetitions faster each input.
  RSS43818->43812KiB/53478->53556KiB essentially unchanged. Still13.9034x/
  12.1786x original end-to-end time. No multiplied cross-node ratios, kernel
  or multicore claim. Retained56includes3shared original Hodge errors;
  Program-vs-Name/Type negative diagnostic differences remain. Reprofile
  exact accepted source before choosing further changes; old pending status
  and R1compile failure are historical, not current failures. No rank expansion.
  Same-source profile3868496 submitted alone incKAa5VP7/pin4da5e11b,
  22checks/6unprofiled controls/4perf captures; pending is not new attribution.

- UPDATE Cartan3868252 FINAL2702e8ae, cu00419m25: correctness AND controlled
  serial A/B accepted. All42checkers,519compiled inventory/34distinct selected
  domain units,16fresh original Cartan histories,72retained complete streams,
  48A/B processes and final source/artifact integrity pass. Retained56includes
  three shared Hodge failures, not56positive mathematical passes. Full rank1
  unitarity12.11173->4.95661s (2.44355x), finite AV-ann16.78998->9.48112s
  (1.77089x); median peak RSS266224->43820KiB and266266->53480KiB.
  Still15.914x/19.203x original end-to-end time, not faster than C++.
  Loading-inclusive small controls are not60-600s kernels or multicore tests.
  Representative-only candidate is now accepted in this bounded scope;
  do not retain earlier pending status as current or multiply cross-node ratios.
  Next measured target is immutable overload-view reconstruction (~1.44s U,
  ~4.39s AV in isolated probes). Capture new accepted/rejected histories and
  actual work failures before runtime caching; cloned Analysis can rebind
  either public types or overloads reference, so both identities must guard
  reuse. Cache unshifted signatures only; preserve fresh polymorphic trials.
  View-before3868323 stopped before Rust tests: provisional null([row])
  controls are rejected by BOTH engines (original global.w null takes int
  or(int,int), not arbitrary rows). Keep exact8070021c draft as separate
  row_null_rejected fixture; use startup-polymorphic #([row]) for the valid
  freshness control. Preserve full survivors/causes and failure2fdffdcd;
  do not label the invalid probe a mathematical error or silently drop it.
  R23868329 FINAL4a9ad6f9:18checkers/607compiled inventory,3actualworkFAIL
  (2/4/3views) and3semanticPASS,3original histories/finalintegrity PASS.
  Positive full streams match; rejected categories/envelopes still differ.
  Only then applied unchanged tests925c59fd and typed19c36c95 view candidate;
  no after acceptance yet. Require six focused thenALL607core,19fresh original
  histories,72retained streams and controlled48process A/B before speed claims.
  After3868400 SUBMITTED alone in skFysMpK/pincf4b3b00; pending not accepted.
  Retain exact Cartan parent2702e8ae separately; no rank escalation, no
  duplicate submission and no speed claim from compilation/work counts alone.
  UPDATE3868400 FAILED before tests at candidate compilation, report391933d8.
  Rc<RefCell<...borrowed owners...>> makes Analysis invariant; an auxiliary
  &'a Analysis<'a> wrongly coupled its short expression borrow and table life.
  Separate MultiAssignmentThreader<'a,'tables>, not unsafe casts or weakening
  owner guards. R2typedabf07725/patch55ce3afb preserves all tests. Compilation
  failure is not a math discrepancy; no after or speed acceptance yet.
  After R23868418 SUBMITTED alone in tYeSNj3C/pinde34e3d4, same full gates;
  collect exact R2, not failed3868400. Profile follow-up is prepared only and
  must await inspected FINAL; keep current job count at one by default.

- Do not conflate original dual metadata with explicit interpreter dual().
  atlas-types.w3320 calls build(dual_datum,delta), potentially constructing
  another full InnerClass; innerclass.cpp403 DualTag constructor is Fokko-only.
  Paired fibers/dual incidence inside the first original InnerClass avoid a
  second classification for metadata, unlike Rust's eager context builder.
  This source difference is an unmeasured lead after the Cartan repair, not
  a selected optimization. Preserve actual dual IDs/order and error budgets;
  neither reverse numbering nor count-only metadata can replace the existing
  block-size/dual-real-forms/dual-Cartan-order consumers without evidence.

- User2026-09-30 explicitly requires careful attribution/probes before
  targeted optimization. Existing rank1 loading fixtures import groups.at,
  whose lines217-254 eagerly initialize G2/F4/E6/E7/E8; do not attribute
  their loading cost to rank1 mathematical kernels. Separate command
  parse/analysis/evaluation, overload merging, classification cache and
  partition phases, with full datum/twist identity and work counts.
  Instrument only isolated diagnostic sources; compare on/off outputs and
  overhead, never use probe clocks as accepted speedup or sum nested times.
  Probe3868224 failed at diagnostic compilation (format brace), after12
  checker passes and before ANY math/probe run. Preserve failure9872491f;
  R2 fixes only isolated instrumentation, never count it as a math repair.
  UPDATE R23868228 FINAL50c7fb30:12checkers/4controls/16probe observations/
  final integrity PASS. Partition6.11-6.19s is largest, overload merges
  ~1.44s U/~4.39s AV second; probe overhead1.4-1.7%. E8 initialization
  is measured loading work, not rank1 mathematics. No production speedup.
  Before-work R23868242 FINALe57c69a6 proves2workFAIL/2semanticPASS on519
  compiled domain inventory,10checkers/integrity. Only then implemented
  representative-only candidate80796a12, not yet after-accepted. Preserve
  direct exact involution count budgets vs legacy full-W budgets, canonical
  order, root-width boundary, full dual lookup and all original histories.
  After3868252 SUBMITTED alone in vL6gPF4k/pinba114c5d:42checkers,
  519inventory/unchanged rank1 and related units,16Cartan histories,
  72retained streams/48A-Bprocesses/final integrity; pending is not speedup.
  R13868238 falsely rejected JSON tuple/list orbit metadata before builds.
  Normalize comparison metadata through JSON, never raw output; two new
  checker cases retain altered/reordered/missing-orbit rejection.
  preserve budgets, canonical order, dual lookup and complete histories.
  Historical R23868228 SUBMITTED as one job inOHvh0QW6, pine6e817a9; no probe
  measurement accepted yet. Collect it before selecting the next rewrite.
- Cartan-class3868167 FINAL:12checkers,8positive+8bounds companions match
  complete mathematical histories and four specific errors/recovery;
  reportb8f60647a42644f3580e97ab755faa69d613b688b45c0645333fc3212a3bafb5.
  Loader-prefix stdout differences remain. This is rank1class/dual-incidence
  coverage, not higher-rank or broad representation acceptance. New kernel
  before helpers/test-only patch94bf9210 are prepared, NOT submitted/run;
  attribution probes take priority over an unmeasured partition rewrite.
- UPDATE presence3866086 FINAL COMPLETED0:0,22m29cu009; report
  95d23a47706b76e6548462c4835bf23f093e327a9b461bd789e8b5dbbd5095d2.
  Correctness/integrity PASS (38checkers,65selected units,601compiled
  inventory,2fresh original histories,70retained streams,48A/Bprocesses),
  but PERFORMANCE_NOT_ACCEPTED: full U10.90781->10.86117s, finite
  AV14.86341->14.90503s. Work-count reductions are not useful speedups.
  Still45.24x/38.30x original full-workload time; no meaningful RSS change.
  Keep correctness-accepted versus performance-accepted source distinct.
  Next measured target is eager Cartan partition; new rank1class capture
  must preserve full dual incidence/order/budgets before production edits.
- UPDATE presence3864029 FINAL:8checkers/601compiled inventory/2actual
  redundant-work assertion failures+1semantic control/2complete original
  captures/unchanged production/final integrity. Report9540131c43c901e7281914070ea3ef1a0006257b0fb73848846b0d44aaef4a46.
  Only then applied unchanged tests and presence-only helper at two sites;
  typed candidate30b24cae, runtime patchb248916b. No cache or changed
  overload ordering. After gates prepared but not accepted; require retained
  56rank1+2old repairs+12language, both new histories and same-node A/B.
  Work-count failures are not math failures; negative diagnostic category
  differences remain. No higher-rank expansion or new speedup claim.
  After3866086 SUBMITTED as one job in87MXIyFj, pinf411a031c56847d44e3e78ecc9c8ea72624fda3119208097a1dd0fea4bcdbf46;
  partial38checkers pass, release inventory compiling. No after acceptance.
  Cartan source audit stays separate: old ON_DEMAND plan's budget and
  numbering assumptions are superseded. cartan_rank1 is matrix-type
  recognition, not full Cartan-class/dual-incidence acceptance.
- UPDATE profile3861850 FINAL COMPLETED0:0 in5m20, cu009 affinity49/53;
  report7a2235c674eb882447fa4da65e5f6774b4171000fb7697e944877c47a4cf9f85.
  22checkers/two unchanged loading streams/separate exact-source profiles/
  integrity PASS. Self samples U/AV: generated Cartan partition23.64/18.00%,
  equivalent6.57/14.95%, validation2.72/6.05%, is_close2.25/3.19%.
  Resolved stacks still show merged_variants -> is_close -> equivalent;
  no inclusive-total or cross-node speedup inference. Next smallest change
  is a presence-only query at two sites, with no cache/state invalidation.
  Eight checker tests/three Rust tests/two original-capture fixtures are
  prepared, not yet run. Expected601compiled inventory,2work failures and
  1semantic control pass BEFORE repair; a work-count failure is not a
  mathematical error. Keep all forgotten-type/shadowing/error-priority rules.
  Before3864029 SUBMITTED inrFLDlK8B, pin7dbb74b2bd3975d732338cfd2e368cdc95fe04326ce54964ea8fc75756cf7e01,
  one job/empty queue. Partial captures match; compilation/before assertions
  still pending. Original negative PROGRAM vs Rust NAME+PROGRAM remains
  explicit despite same three causes/survivors. No runtime repair yet.
- UPDATE overload3858574 FINAL COMPLETED0:0 in24m10, cu001 affinity61/62.
  Reportac29df12a4d7286fe7cde4bd49d182edee9551fc088196c40053b5e6b5d13783.
  Both full repaired original histories/38checkers/5coercion/2session/55type
  units/56retained streams/12language/48A-Bprocesses/final integrity PASS.
  598is compiled inventory, NOT full598testPASS; retained53positive cases
  plus3shared original Hodge failures are not56mathematical passes.
  Fresh same-node medians: unitarity12.7165->11.2092s (1.134x), finite
  AV-ann19.4303->15.1464s (1.283x); original0.24739/0.40217s means Rust
  STILL45.31x/37.66x original end-to-end time. All4/4repetitions faster on
  every input. Memory~260MiB unchanged; no kernel/multicore/overall-port claim.
  Frozen runtimeb32c06a6, binary95cc29f2. Localb147208c differs ONLY by a
  corrected explanatory comment (row-to-row coercions exist and are handled
  componentwise); comment-only patch7e3176e4 is separate from verified source.
  Keep performance priority/small rank. Reprofile before choosing the next
  change; ordinary call-head presence currently builds the merged view twice.
  Presence tests patch9c223a7e and two provisional fixtures are isolated,
  unrun and NOT part of598. No presence helper or cache has been implemented.
  Post-repair profile3861850 SUBMITTED from an empty queue as ONE job in
  aH4uDHA0, pinf1fccec7c0bcaa0f437012230575604fce9f8037a2adf57dffbc976df61a027c.
  Requires22checker tests, accepted95cc binary for two unchanged load
  comparisons, separate SAME FROZEN source/frame-pointer build for sampling.
  No new discovery/rank/runtime change; collect before choosing another fix.
- UPDATE R3type-equivalence3856241 FINAL accepted inVBs8Hh3A,
  report4b234d258c6df5909f9f26a0b7a2e53d4ead9a18ec06ce9dd1a993e10128f877.
  Same-node four-round serial A/B: full rank1 unitarity23.4033->12.4552s
  (1.879x), finite AV-ann51.0667->19.0573s (2.680x); STILL51.71x/49.04x
  original time. Loading-inclusive, not kernel or multicore speedup. RSS
  remains about260MiB, no meaningful memory saving. All56retained streams
  unchanged (53positive math/3shared original Hodge failures),12language
  cases,24checkers and final integrity pass.594is compiled inventory,
  not a full test run:3semanticPASS/1workFAIL before,4PASS/55type/3coercion
  after. Candidate binary2645dea4/runtime732380b6; local crate hashes match.
  The16770-to-258node-visit reduction is NOT the measured process speedup.
  Continue small-rank profiling; no higher-rank expansion or broad acceptance.
- Followup3856293 FINAL in i1cxrpAU, reportf4c27c536339a3de4ed0b47ab1ece8ec12618d8c76e508fa82182952b38e6454.
  14checkers/two unchanged rank1 loads/two separate frame-pointer profiles/
  integrity PASS. AV self samples: equivalent27.09%, generated Cartan
  partition14.91%, validation8.16%, is_close5.10%; resolved stacks show
  merged_variants -> is_close -> equivalent. Some outer stacks remain
  incomplete: no invented inclusive total or instrumentation speed ratio.
  Original now CONFIRMS zero-argument redefinition replaces the old overload;
  Rust adds duplicates and emits2Type ambiguities. Preserve full golden
  049cddde and two new tests BEFORE fixing. Before3856744 FINAL in
  YyVCvrMg, pinf297701754def664f638d622cc04ea63c3473d0e76084904b641591b54a37e96;
  596compiled inventory/2actual assertion failures, production unchanged;
  report33e8ffff875f0905892b0a8ee0cbd061ffa77118c3ae753c96fb7fc5a2b21e55,
  final integrity PASS. Original recursive-row history ACCEPTS; Rust exits139/
  signal11,24.26s/RSS6207724KiB/empty stdout. Preserve signal/resource status,
  not timeout or an accepted speed ratio; exact call-site attribution remains
  source-based. Two recursive tests/full original golden4825c1d7 were added
  before the runtime edit (598local inventory; only two nullary units ran
  before, not four). Candidate is_close b32c06a6 ports equality-first,
  recursive-name guards, primitive-endpoint lookup and tuple early termination.
  No after acceptance yet; require full repaired histories, retained56/12
  streams and controlled A/B. Do not weaken goldens or add an overload cache
  before this correctness gate is resolved. No higher-rank expansion.
  After3858574 SUBMITTED as ONE job from an empty queue in jTo2AfSy,
  pind0de137da82017f2266ccd7ab69074c3bbb5830d490fada5b93c1240d87515a0;
  38checkers/598inventory/fresh before-and-after builds/48timed processes.
  Submitted is not accepted; collect this job rather than resubmitting.
  PARTIAL3858574:598compiled inventory,5coercion/2session/55type units,
  both full original repair histories pass;56/56rank1and12language streams
  retained at17m42,3/16A-Brows, final integrity pending. Post-repair profile helpers
  are prepared but UNTESTED/UNSUBMITTED; require FINAL after source and
  explicit report hash. Do not reuse594inventory acceptance for598source.
- R23856122 FAILED after56unchanged rank1 histories: the eleventh language
  input is a pure rejection with an EMPTY payload and complete BEGIN/END/Bye.
  All three exit1; Rust full streams match. Negative-language checks must
  retain exact ordered recovery/specific diagnostics/nonzero exits, not
  demand successful values. Accepted inputs still require successful output.
  R3adds6checker regressions (24total), checkpoints failures before raising,
  and preserves all sources/fixtures/goldens. Failed reporte267b48a retained;
  no R2benchmark/final integrity and no mathematical error inferred.
- Original coercion lookup is also linear; do not invent a conversion-matrix
  advantage. Its is_close prunes by structure and checks equality BEFORE void.
  Before Rust(void,void) returned0;3856293confirms the observable error.
  Preserve overload_void_redefinition input/full original golden and require
  unchanged before/after regression evidence. An analysis-local overload
  cache must account for BOTH public/rebindable TypeTable and OverloadState
  references and keep fresh inference trials/captured values independent.

- Type-equivalence3856060 FAILED before any checker/build/math at missing
  math_generic_probe import. A reduced direct-capture parent also lacks the
  generic master catalog. R2 must stage the classifier, a targeted12-case
  catalog AND unchanged pinned-source fixtures; require18Python checks.
  Runtime/tests/patches stay unchanged; preserve failure log0c811f5d.

- 3856033 FINAL: all16load/full pairs match; ten checker/integrity PASS.
  Rust medians22.63/22.76s for load/full unitarity and49.37/49.69s for
  load/full AV-ann. Resolved self samples: validate_applications52.80%,
  equivalent19.68%; do not infer inclusive caller shares from bad unwinds.
- Type-equivalence candidate is now verified in the bounded R3scope above.
  Preserve malformed unused
  constructor checks, recursive identities and validation when alias expansion
  exposes new nodes. Four unchanged regressions must show3semantic passes/
  1quadratic-work failure before,4passes after, then all retained rank1
  outputs and same-node fresh-build A/B for future changes. A work assertion is not a discovered
  mathematical calculation error. See loading_performance_rank1 slice.
- Historical candidate gate3856060 in U8z5KvFq, pin56b7dcf5de87014d171ee9537060c638537dc556134e7f2f4a12159d3c82714d,
  FAILED at import; no test inventory compiled in that failed job. Adopt R2.

### Rank1 real/complex unitarity coverage is finite and presentation-specific

- 3856011 FINAL matches six complete original math tails: SL_R(2),SU(2),
  PSL_R(2),PSU(2),SL_C(2),PSL_C(2), five fixed gamma/rho values each.
  Preserve all46parameters/46final constituents and SIXactual empty sets;
  nonstandard/zero flags still never occur. Complex rank1 has real datum
  rank2. These are exact groups.at presentations (PGL_R is a PSL_R alias),
  not every numbering, connectedness convention or infinite parameter.
  Report0c1147bfbccf50a05674b25e6c3dfe4a14d079ea7418d1bff33a07f2f53fe390.
  See `docs/slices/unitarity_rank1_forms_2026-09-30.md`; loader prefixes and
  large full-process timing gaps remain, no multicore speedup is established.

### AV support markers and finite-cycle multiplicity boundaries

- UPDATE review3856006 FINAL:15checker/integrity PASS and both complete
  capture3855999math tails match. No Atlas rerun; old false incomplete
  classification/report retained. Loader-prefix stdout still differs.
  Report5c366f5ca96729e2d23075320a18aad48b011dcf3d8ae4a8b758f5ba8673e63f.
- Rank1capture3855999 matches all four finite SL2modules (dimensions1,2,3,5),
  both AV-ann routes and full character formulas. AC(M)=dim(M)[{0}] here is
  an independent finite-module deduction, not a generic cycle implementation
  or the cycle of U(g)/Ann(M). Keep nonzero-orbit multiplicities open.
- Its retained control is falsely classified incomplete by the R1checker:
  `ORBIT(?!_ROOTS)` also matches `ORBIT_COROOTS`. Use distinct ordered
  PARAMETER/ORBIT/ORBIT_ROOTS/ORBIT_COROOTS inventories, preserve the original
  report/raw outputs, and review them on compute with regression checks.
  Never rerun mathematics or weaken complete-output equality to hide a
  checker mistake. See `docs/slices/associated_cycle_rank1_anchor_2026-09-30.md`.

### Ordinary full deformation needs both Split factors and recursion

- UPDATE3855872 FINAL: both unchanged regressions and2retained A1units pass;
  45positive complete math comparisons match,3shared original Hodge failures
  stay NOT passes. All48tails agree, but loader/diagnostic differences remain.
  Reportbeeb3e280738d528460e1935c00861c8540afe37e89c59f04773ffb2cd817226.
  590is compiled inventory, not a full unit run; no higher-rank/general Hodge
  or cycle acceptance. The original-backed before failures remain preserved.
- Before3855838 FINAL proves compact A1 PASS and split A1 full-stream FAIL
  after valid setup/recovery, with590compiled tests and unchanged production.
  Bare original goldens precede repair. Original repr.cpp computes
  D(z)=sum c*(L(t)+2D(t)), F(z)=L(z)+(1-s)D(z); equivalently recurse on the
  complete F(t) with c*(1-s). Do not merely add1-s while retaining Rust's
  erroneous conversion of each child into a single K-type.
- Preserve all scale-zero finals, reducibility points, common-block row and
  modifier, canonical ordering and deadlines. Never hold a cache lock during
  recursion or cache an incomplete result; detect cycles explicitly. The
  rank1candidate is not higher-rank/general Hodge acceptance. Retain shared
  original padding errors separately and compare complete cold/warm
  specialization polynomials. See the indexed ordinary_full_deform slice.

### Shared oracle failures can hide earlier Rust mathematical differences

- Hodge3855673 proves both engines fail on trailing-zero polynomial padding,
  but complete SL2R bound4/12backtraces expose earlier elementary v=s checks:
  original b,d=true; Rust b,d=false. Preserve this as a distinct Rust math
  regression. New hodge_rank1_specialization fixture/expectation is pending
  reduced original capture; no runtime fix before the unchanged regression.
  Inspect full successful intermediates and frame values, not just the final
  shared error. Do not hide the bug by stripping old inputs, dropping assertions,
  lowering bounds or silently patching the original scripts. See progressive
  validation slice for full hashes, cold/warm controls and scope boundaries.

### Preserve parameter and F4 diagnostic boundaries

- FINAL3851780 verifies all24fixed-gamma grids, not merely the firstA1case.
  All three F4AV-ann forms still FAIL on six-union. FINAL3851976 locates a
  missing C3Levi class plus a duplicate despite25totalclasses; Kondo12's
  signature lookup fails. Cartan_matrix_type must return the original's
  Bourbaki-to-input map, not unconditional identity or an arbitrary
  isomorphism permutation. Freeze reduced original goldens before repair,
  retain full class/character/Springer/AV-ann consumers afterwards.
- Old full408review3848053 OOM at2GiB after63checker tests passed. Retry
  only the unchanged reviewer with more compute memory, retaining all408
  raw cases and original failure evidence; do not resubmit mathematical arrays.

- Fixed-gamma inventories must keep every parameter returned by original
  all_parameters_gamma, all standard final constituents and coefficients,
  nonstandard entries and empty fractional sets; do not substitute a few
  trivial(G) scales. New rank6 parameters catalog is28cases with7HPC checks
  in3851750, not broad mathematical acceptance.
- Original basic.at requisition emits the same No solution found message
  for absent Maybe values such as binary_lookup failures. F4AV-ann still
  fails on six-union in3851782/3851783. Do not label it a linear-solver bug
  without tracing the caller. Capture back_trace immediately after failure,
  before a later undefined-name error can overwrite it.
- K_Nilpotent.at av/av_from_av_ann returns associated-variety orbit support.
  Do not relabel support or arbitrary quotient-matrix coefficients as an
  independently verified associated cycle with multiplicities.

### Rank6 inventory must include unequal D-even classes and real rank12

- FINAL3851489 verifies640/640complete block inventories on six-union and
  exports all1114nonempty+1014zero dual pairs without original missing forms.
  Do not resubmit metadata or call this full1114KLV acceptance. Preserve
  every pair and budget full quadratic KLV outputs before bulk dispatch;
  dimensional/pool-only equality cannot replace complete coefficients.

- Same-input272replay3850467 FINAL on six-union recovers ALL48complex
  rank5/6failures and preserves216old matches:264MATCH/8original rejections.
  Do not continue describing the repaired fixed-width cap as a current
  inventory failure. Keep the original before-regressions and the8canonical
  quotient/u rejections; inventory success does not certify full KLV or
  associated-cycle/AV-ann correctness. CPU130complete KGB review3851146
  is separately130MATCH, not full640acceptance.

- Do not assume a multi-array submission is atomic. KGB CPU3850496 was
  accepted while fat510 was rejected by normal QoS MaxSubmitJobsPU=800.
  Adopt successful job IDs; never blindly rerun the launcher. The checked
  stage_rank6_shards dispatcher preserves all indices, chains same-tier
  shards at original concurrency, and leaves uncertain submission intents
  fail-closed. Nine compute tests3850863 cover this. Scheduler limits are
  not mathematical failures; frozen fixtures must not be trimmed to fit them.

- UPDATE six-union3850248 FINAL586core/515domain/all retained gates PASS;
  local90crate source hashes match. New640KGB/272replay/640block-inventory
  preflights use this exact source. Full block-pair export must retain
  zero-size pairs and original-accepted pairs even when Rust failed; original
  failures/timeouts must make coverage explicitly incomplete. Checker3850433
  proves retention and tamper detection in12tests, not mathematical coverage.

- Original3850338 rejects '=' on two (mat,[vec],vec) triples. Compare fields
  with matrix/vector '=' and [vec]'==' (basic.at119), retaining full outputs;
  never assume structural equality for arbitrary Atlas tuples. Its rejected
  raw_KL(42)/dual_KL(42) overloads use Program-error analysis envelopes,
  whereas Rust uses Type errors. Require both specific failed-call messages,
  keep diagnostic wording/category distinctions, and retain failed fixtures.

- After a capture sets a provisional success tag, a final integrity failure
  must overwrite it with HARNESS_FAILURE and return nonzero. Graph R3checker
  3850330 proves this with fault injection (15tests); independent downstream
  review is not an excuse for a misleading successful process exit.
- Template checks must distinguish @UPPERCASE_TOKEN@ from Atlas's legal
  index binders such as drf@j. Block checker3850300 stopped before Atlas
  because it rejected every '@'. R2corrects only the synthetic assertion,
  explicitly retains index binders, and leaves math templates unchanged.

- UPDATE capacity3849939 FINAL575core/515domain/all retained gates PASS;
  exact six-parent586/515union3850248 is submitted, not verified together.
  All640graph checker3850194 passes14tests, not mathematical execution.
  Require FINAL union and10complete graph smokes before bulk arrays; preserve
  all640indices/raw artifacts and reclassify them in an independent review.
  R13850133 was a harness SyntaxError before math, not a calculation failure.

- Audit fixed-width initializers with whitespace-tolerant searches. R2after
  3849865 passed all3core originals but failed domain test compilation because
  `assert_eq!(generated[0],[0;8])` was missed by a `[0; 8]` search. R3changes
  this assertion to compact.identity(); do not weaken or remove it. Full
  515domain/575core pass in live R3logs; FINAL consumer/integrity still needed.
  A focused core test does not compile the domain crate's cfg(test) module.

- Per-form manifests must use ORIGINAL acceptance, not INVENTORY_MATCH.
  3850019 FINAL extracts640form presentations from264original-accepted inputs,
  including48from old Rust-failed complex inventories. Retain8original
  rejections separately. All8full KGB smoke graphs match on583union, but
  640graph coverage remains pending. Compare every node/edge payload and
  order, not only row counts; check contiguous nodes and the full Cartesian
  node/simple-generator index set, recovery/END/Bye and rejection/timeout
  handling. Actual associated cycles and AV-ann are still separate gates.

- Before3849785 FINAL proves rank4PASS/rank5+6FAIL on575core with exactly
  two cap8Runtime messages, valid setup/recovery and unchanged production.
  Width32 candidate3849865 is submitted, NOT accepted. Keep the original
  three complete goldens unchanged. Four kernel tests now cover width32/33,
  exact A1^9enumeration, noncommuting high-index twist/multiplication/inverse
  and every A1^12root permutation. Count budgets and the distinct rank<=8
  u64 path must remain unchanged. Root-permutation u8 limits are separate:
  generated partition uses checked conversion; width32 is not a blanket
  claim that all rank32 root systems or downstream operations are supported.
  The local586core/515domain union needs separate validation from isolated
  575/515capacity and frozen583/511five-repair gates.

- Direct-capture stages may still carry an old six-check language bridge and
  lack integration helpers. Capacity after R1jsRs5G4C stopped before sbatch
  on this hash mismatch. R2 explicitly imports the exact catalog-aware bridge,
  verified_merge and pos_neg_consumer helpers from FINAL PSp; retain full
  manifest/hash checks and do not weaken checker counts to accommodate a
  stale helper. Never edit the failed or already submitted stage in place.

- Reduced3849710 FINAL original accepts bare A4/A5/A6complex metadata.
  A4fullstdout/stderr matches; A5/A6each fail one cap8Runtime after setup.
  Wrong-size-involution recovery output matches, but the original10x10/9x9
  diagnostic differs from Rust's generic not-an-involution error. Keep this
  separately visible. Original complete goldens and before regression tests
  are required before changing the capacity representation.

- Raw rank6 A5complex/B6complex observations reveal an artificial
  WeylElt=[u8;8] constructor cap, not an actual generated-element budget
  exhaustion. Original constants.h uses RANK_MAX=32. Preserve the failing
  original-backed input before changing representation; audit all zero
  arrays, the separate rank<=8 u64 fast path, test-only generators, root
  permutation widths and the genuine >32 boundary. Do not just delete the
  check, raise enumeration counts or turn the failure into an empty result.

- Upstream generate_groups.at names e/u for A(rank>1), D and E6. In
  particular D4/D6 unequal classes are not covered by only +/-identity.
  Use original symbolic inner_class conversion and retain incompatible
  outer-involution/central-quotient pairs as discovery failures, not passes.
- The new272-case inventory includes22simple types,54central-subgroup
  presentations, both numberings,164real inner-class and108complex cases.
  These are constructor/inventory cases, not272distinct groups or complete
  representation tests. Complex rank6 needs real root-datum rank12.
  See rank6_validation slice; the existing408survey is unchanged.
- Disable bytecode BEFORE importing helpers from frozen parents. First rank6
  staging stopped before sbatch because Python wrote exactly one new
  stage_merged_math.cpython-39.pyc in the parent. The generated file was moved
  into the failed stage for evidence; the exact original file manifest was
  restored without changing source. Set PYTHONDONTWRITEBYTECODE=1 and make
  standalone stagers set sys.dont_write_bytecode before helper imports.

### Preserve canonical ParamPol order after restoring missing terms

- UPDATE3847661 FINAL PASS within its bounded575core/511domain scope,
  reportee0281513cdde37713399ceb3194da57bfa4c789e6552b53e5061b32236d5f8f.
  Three unchanged regressions pass; both full positive ordering tails, all
  three cycle traces AND the PHI_ORBITS discovery tail now match original.
  Loader-prefix stdout remains different. Six unitarity companions retained;
  GL2R still fails. This is not full408 or general cycle completeness.
- UPDATE after gate3847661 SUBMITTED in atlas-parampol-build-20260929.GsZiylne,
  pin69b546b03477fcd93ce4db31fdf8f88c5b967d9b4e8f4628eafd7dc810dcecc7.
  It uses FINAL merged3847093 and before3847509. Still NOT after-verified;
  preserve unchanged tests and complete cycle/order outputs. Frozen checker12
  is separate from the newer FPPchecker13.
- UPDATE before3847509 FINAL:575inventory, EXACT3ordering assertion
  failures after valid setup, runtime unchanged and source rehashed. Report
  8607362847086c36d5ca30798f219b5c6bc704c145136d5923942a6ee63ab8ef.
  Comparator candidate now scans torsion high-to-low, compares gamma
  descending and widens i64cross products to i128. Tests unchanged;
  after-sourcec6a7207fe9853a9f2ab494dc7a081c298b39daee3db304181a206a9881c5667e.
  NOT after-verified. The after gate must keep full order/cycle tails and
  no lost prerequisite positives; see newest receipt/HANDOFF.
- UPDATE original3847089 accepts both explicit-Split valid probes and
  retains both old untyped-list rejections. Complete output confirms BOTH
  ordering bugs: A2/T3packed torsion and T1signed fractional gamma. Three
  original-backed unit regressions are added WITHOUT a runtime change;
  see parampol_order slice and math_parampol_before submission receipt.
  Before-only job3847509 freezes exact R2source plus3tests; require3actual
  order assertion failures, not compilation/setup failures. No fix yet.
  Current local core has575tests (three new/unverified); frozen mergeR4
  still has572. Never claim the newer comparator/tests are in that binary.
- Cycle replay3847046 FINAL on exact root-sign511domain/core569 restores
  all missing A2coherent/induced terms and all three final KTypePol outputs.
  Complete induction/standardize tails STILL differ: two ParamPol terms are
  swapped. Identity controls match. Keep the full-tail failures; do not sort
  output or call the entire cycle accepted. Report39cb4e1301bc47d7f6ac1ddd1e50992288168b2f900cf9a3257c1cad3c4537c0.
- This supersedes the earlier missing-Cayley-pair next action. The existing
  root-sign repair already restores that pair; do not add a duplicate fix.
- Original repr.cpp1161 orders height ascending, x descending, numeric packed
  torsion ascending, then gamma DESCENDING via cross multiplication. Current
  Rust scans torsion low-bit-first and gamma ascending. Check both with full
  original-backed regressions before repairing; do not change global
  ModTwoVector map-key ordering or KTypePol's different comparator.
- Two provisional ordering fixtures submitted3847067 (checker12), separate
  from master273/full408; inspect original acceptance first. See indexed
  associated_cycle_frontier slice for exact pins and preserved streams.

### Keep bridge and direct-capture harness versions explicit

- UPDATE R4job3847093 FINAL foundation PASS, report
  9a38426f9749f4ff7bf2b42443d80927f7a88bf6a0377ee59b7336b93e07124d.
  All511domain/572core,4marker checks,273capture/11diagnostics without lost
  positives,4script mathematical tails and4block histories pass. Six of seven
  unitarity companion tails match; GL2R retains Inexact halving, although its
  trivial-unitarity anchor is restored. This does NOT validate all408cases.
  Fresh all408 release build3847662/preflight3847663 submitted in
  atlas-merged-math-20260929.mwbojUD8; arrays await both gates. Read newest
  math_merged_survey submission receipt; do not rebuild or reuse exact566.
- CURRENT R4job3847093 SUBMITTED in atlas-verified-merge-20260929.KolaCCOF,
  pin797acb3330c05ce114c0b2551c8606d17b15ce07021ccad1478048c9b7951555.
  R3exited at import before any tests/build/report: its archive omitted
  math_pos_neg_consumer.py. Preserve raw log5a40a9c883e95ed77aa72c7a6a9d314a0ef3510143e1be030bb63ed8fb3f461b.
  R4inherits the COMPLETE pinned R2harness and explicitly guards the strict
  tail helper/bridge/checker hashes. Never rebuild a derived stage from an
  older ancestor without carrying all transitive helper dependencies.
- UPDATE R3job3847086 SUBMITTED in atlas-verified-merge-20260929.s7Iz06Bn,
  pin00b5c5da4095bcaef12e092dc54cec63169f95a9530b931ca5797619bffc5b6e.
  R2passes511domain/572core/273capture/11diagnostics with no positive losses,
  then fails a HARNESS assertion: SET_BITS appears10times, not once;
  FACTOR_WEYL_ORDER likewise appears9times. Require those exact inventories
  and compare EVERY byte from the first iteration through recovery/end/Bye.
  Four dedicated harness regressions keep first/middle/end differences and
  malformed inventories visible; all other markers stay unique. R3changes
  no runtime, fixture, golden, or math gate. Preserve R2report
  a97a3e261b0d10e09647448bc7aa710accd5a7a4b3e04d9a2395778c64da436e.
- Corrected exact-union R2job3847039 SUBMITTED, pin947e33f6c3fd91242c8fc5d9fb85cd48b848e8c055de939837cacbbc77b0f560.
  Same runtime/goldens/counts. Read HANDOFF before collecting or replaying.
- Merge3847021 passes511domain/572core/CLI release, then falsely fails
  the capture checker: inherited generic-probe stage carries the OLD bridge
  hardcoding six tests, although its checker now runs ten, all passing.
  Direct capture never called that bridge, so its success did not validate
  bridge compatibility. Preserve FAIL report ec4381dd8ee01d3e85f2366140635d7c6b92ae5f67d6abf73f1a3ee3644910d0.
- Explicitly stage the already-verified configurable bridge861b1e48 and
  pin checker_tests=10; keep all runtime/test/golden sources unchanged.
  New stages require that bridge hash before sbatch. No capture ran in R1;
  passing units/builds are not merged mathematical acceptance.

### Positive-to-negative root sets start empty

- UPDATE consumer3846760 FINAL PASS, report acfc900cb35cc4b872cfd33f787d0626502fdd005733376e071f63d0950a37c9:
  complete cold/full-first/triple-first PSp4R mathematical histories now
  match original; all three differ on exact569before. SC Sp4R control matches
  before and after. Full stdout still differs in loader declarations; retain
  that fact. Merge with separately verified torus572 only in a new stage.
- UPDATE kernel3846740 FINAL PASS, report e8459baf79f4fdf1376535cd8ac2dffb9fd0e1158ca400407a0a2dc06ef41a9f:
  same3beforefail/3afterpass/full511domain/integrity. Focused release consumer
  3846760 requires four full block histories plus exact569before. Still no
  merged core/CLI/broad mathematical acceptance; retain separate candidates.
- PSp4R capture3846724 proves missing DIRECT partial-block rows and wrong
  full-block cross links. Original rootdata.cpp1486 starts pos_to_neg EMPTY;
  Rust started ALL positives, computing the complement, even for identity.
- Three regression gates cover identity/simple/square, direct root-image
  checks for all words through length6, and original PSp partial/full links.
  Kernel3846740 (exact569parent, full511domain) is verified.
  Preserve all goldens and require downstream consumers after merge with
  independent torus572candidate. See indexed positive_negative_roots slice.

### Semisimple factors exclude tori; bitsets are sets

- UPDATE R3job3846731 FINAL PASS, report 0d458f16f3e15448d323cc0143b05f9244f6f0453b8300372f05c5f756933486:
  three beforefail/afterpass,508domain/572core/CLI/273capture with no lost
  positives; full factor/bitset goldens and complete Weyl-order/SL2R-half/
  torus AV-ann mathematical tails match. Not broad408 acceptance. Root-sign
  kernel is a separate candidate; verify the exact union before consumers.
- R2torus3846692 passes two regressions but full factor stdout retains six
  wrong mixed-torus RootDatum names. Original RootDatum::type appends tori
  after semisimple components; build_datum already uses that matrix order.
  Keep input LieType order intact, canonicalize the datum metadata, and keep
  the unchanged full golden. R3job3846731 submitted, not verified.
- Original11capture3846570 accepts full factor/order_W and bitset positives;
  Rust fails. atlas-types.w388 skips T, but Rust included T, sending
  character_table(T1) through order_W_simple(T1)/nonexistent highest roots.
  Filter factors, do not weaken basic.at's negative-bitset assertion.
- Duplicate bit indices use OR, not +=. Original[0,0]->1 and[0,2,0,2]->5;
  old Rust2/10. Keep limb-boundary duplicate regressions.
- Original simple_factors("T1") ACCEPTS implicit conversion and prints
  Value:[]. The negative has TWO errors, not three; retain full survivors.
- Candidate5723846692 freezes verified569parent, unchanged-before3fail/
  after3pass gates,508domain/572core/CLI/273capture and11diagnostics with
  exact569before. Submitted, not verified. Read indexed
  docs/slices/torus_factors_bitset_2026-09-29.md for pins and scope.
- First gate3846643 stopped after the correct three before failures because
  Rust now prints a numeric thread ID between the panic name and `panicked`.
  Preserve that failure artifact. R2 only permits this optional numeric ID;
  all three exact test names, counts, tests, goldens and runtime patches stay
  unchanged. A log-parser failure is not evidence about after-fix behavior.

### Different real forms are not covered by a split-form label

- UPDATE PSp3849212 FINAL PASS, report
  b691a9b3f053570d2d5109583c63a484867d41329da442098ae94783efeb75e7.
  Same3regressions/575core/511domain/all5complete history streams/sixunitary/
  273+11no-loss/4script4block/finalintegrity. Exact five-repair union3849627
  is submitted, not yet accepted. Preserve isolated versus merged scope.

- PSpbefore3848989 FINAL verifies2controlsPASS/1historyFAIL with exact3
  Runtime Cayley errors,575inventory and unchanged production/integrity.
  Only then removed equal-locator/no-shift shortcut; candidate3849212 in
  I3OP37I0 is pending. Always compute relative shift from representatives,
  even if the Weyl locator is identical. Keep canonical-key dedup, all link/
  modifier/concurrency checks, every original golden and all5history gates.
  No acceptance by clearing the cache; no merged583claim from isolated575.
- PSp R23848939 FINAL: bare ordinary-KL all-term history AND isolated
  unitary prewarm reproduce three Cayley Runtime errors; uncached direct
  partial block still correct. Three whole-stream regressions-before3848989
  require2controlPASS/1historyFAIL on575inventory before production edits.
  Source suspect: equal block locators do NOT imply zero integral-orthogonal
  shift; original append_block_containing always computes make_relative_to,
  whereas Rust overlap merge skips it for equality. Keep every invariant
  and prove the unchanged histories after any repair; do not clear caches.
- PSp3848885 FINAL disproves a cold-target/short-warm failure: BOTH bare
  cold and full triple-reducibility histories match original stdout/stderr.
  Original direct block2rows versus pooled block3rows can be correct; keep
  the subset/KL witnesses. Unchanged full script still fails. R2capture
  3848939 separates all ordinary-KL term histories from is_unitary prewarm.
  No PSp runtime edit yet. See indexed psp4_cayley_history slice.
- GL2after3848801 FINAL PASS, report9c0dc173694b4ee168d1113a156b077b4ffffc0a0054023e408511a7fefa2eed.
  Two unchanged tests/574core/511domain/6GL2streams/ALL7complete unitary
  tails/273+11no loss/4script/4block/final integrity pass. GL2 companion is
  restored; six controls retained. Loader-prefix stdout still differs.
  This is isolatedGL2574, not the separate FPP/ParamPol/G2 union.
- PSp3848885 captures separate bare cold/warm cases with actual groups.at
  root datumI2/coroot columns[[2,-2],[-1,2]], false numbering, and unchanged
  live script trace. Its accepted intent is provisional; preserve every
  required Cayley link, block/KL result and recovery. Checker16 is separate
  from G2frozen15/GL2frozen14. No PSp runtime edit before original capture.
- FPP R3job3848729 FINAL PASS, report445e22c1d7ad89b82c299079e5fc971b3aa939a8ec1bebe770c329f7c4cbc0f5.
  Two unchanged regressions/574core/511domain/CLI and full4product streams
  pass; SL3C/PSL3C/Sp4C/G2complex full FPP mathematical tails now match,
  whereas exact572before still mismatches all4.273+11no losses,4script/
  4block/six unitary controls retained, final integrity. Loader declarations
  still differ; this isolated574repair is not merged local580/full408.
- UPDATE exact572 CPUreview3848054 FINAL:187cases,166MATH_MATCH,
  6REJECTION_CATEGORY_MATCH,5RUST_FAILURE,9ORACLE_FAILURE,1ORACLE_TIMEOUT.
  Reportb5bbd0faff56bf503c6d397b3e65d18e6727a65105a4ed6569716ab09351c867.
  The same132form inputs gain19matches without losing old matches:17AV-ann,
  PSp4R KLV and Sp4R unitarity. Each of22metadata/KGB/KLV/AV-ann cases matches.
  Fat221/full408 review remains pending; no Hodge/cycle/full-rank inference.
- GL2before3848755 FINAL proves2unchanged stream assertion failures after
  successful setup/rank diagnostics; report5bc167022e310e8db0bea6b122eea6f52bb4c03e26491328b126caab169cad63.
  Distinguished-only lookup repair submitted3848801 in2OcYoCM1. It uses
  exact common-block row/modifier at all integral ranks and DIRECT simp_int
  order for folded singular flags, not parent-order located_singular_flags.
  External-delta/twisted_deform/halving guards stay unchanged.574candidate
  excludes FPP/ParamPol; require full7unitarity tails, not only2units.
- G2unitarity3848441/3848708 reaches `common deformation complex cross`.
  Source audit: PartialBlock's inherent cross takes(generator,row), while
  BlockTopology::cross takes(row,generator); the concrete call in
  common_deformation_terms currently selects the inherent method with swapped
  arguments. Capture bare G2x7 and full oriented-KL/reducibility traces BEFORE
  repairing. Keep PSp4R's separate downward-Cayley interval failure intact;
  never change required downward links into zero contributions.
- GL2Rbare capture3848669 accepts the complete explicit-negativeidentity
  positive; both numberings lose ONLY the singular twisted diagonal in Rust.
  Its recovery companion has exactly two rank errors (2,2,1)/(2,2,3), then
  the same wrong empty sum. Preserve full original goldens and two new tests
  before runtime repair. Older3848639 bare inputs fail at script-only unary
  matrix minus; retain them but never count them as nu-size coverage.
- FPP3848594 passes2regressions/574core/511domain/CLI/full4product gates but
  fails its HARNESS inventory: inherited math_suite.py omits real_form_grids
  and expands only116cases. Reportafd4ab62d1c4ca9ac11b4ff73bcb1c08655d6e04ff24b455a16e9c4722577d36.
  Stage catalog AND its verified reader/checker together. Fresh3848729 in
  YEGACFZ3 pins unchanged runtime/tests/goldens and runs three actual catalog
  checker tests before compilation. Never waive408 or count this as mathPASS.
- UPDATE full408exact572 arrays are CPU3848051/fat3848052, reviews3848053/
  3848054 after verified fresh build3847662/preflight3847663. Never report
  these pending results as a percentage of mathematical correctness.
- GL2Rcapture3848183 proves singular zero-scale twisted_KL_sum_at_s loses
  its1*p diagonal term (EMPTY Rust), whereas ordinary KL remains1*p. The
  resulting fixed coefficient1+s cannot be halved. Keep the parity guard;
  repair the missing twisted contribution upstream, after regression capture.
  Both cold twisted-first and ordinary-first histories fail. See unitarity
  slice for exact pins and the unproven Full-vs-common-block hypothesis.
- UPDATE FPPbefore3847713 FINAL proves2actual stream failures,574inventory
  and unchanged runtime. Root-component max-RootNbr sort candidate is now
  submitted3848594; no after acceptance. Preserve source separation from
  ParamPol575/localunion577. Reduced-capture stages do not contain the broad
  survey catalog/templates: explicitly import their pinned bytes before
  freezing a derived consumer stage; first QzbyEM7z failed before sbatch.
- FPPbefore3847713 freezes verified572 plus two complete original-backed
  session regressions only (574tests). Local union577 also contains the
  separate unverified ParamPol candidate; never use its count as a verified
  inventory. No FPP production edit precedes actual stream assertion failures.
  Keep the full positive and negative-survivor goldens, all4Runtime messages,
  Weyl/shift witnesses, multiplicities and returned void rows. See HANDOFF.
- UPDATE FPP4capture3847592 FINAL, report
  e18e46143afb4da6909a1d46d65eee5e3f7ade294f66c7a4d17640113a5cb000.
  Eight-type valid product probe is wholly accepted by both engines but
  complete stdout differs. Leading T1.A2 isolation proves both adjoint
  numbering choices fail in original; SC succeeds. Keep all old failure and
  recovery streams; do not generalize this to trailing A2.T1 or all tori.
  No FPP runtime repair yet. See indexed fpp_product_order slice.
- FPPproduct ordering is SEPARATE from ParamPol sorting. Original
  rootdata.cpp1514components moves each newly touched component to the
  back, so final order follows ascending MAX RootNbr; Rust union-find
  currently emits ascending MIN RootNbr. This is a source-backed candidate
  cause of complex-product permutations, not yet a verified repair.
  New separate fpp_product_catalog has9types x2numberings x2isogenies,
  complete FPP vectors AND Weyl/shift witnesses, plus rejected inputs and
  recovery. Capture original acceptance first; never sort results to waive
  ordering or claim set equality without an independent complete check.
- Full form review3846278 FINAL292:164matches,21output mismatches,47Rust
  failures,57original failures,3original timeouts on exact566. Metadata36/
  KGB36all pass; KLV35/36. Broad high-level failures remain; original Hodge
  failures are not acceptance. Read the indexed real_complex_forms table.
  New merged511domain/572core gate3847021 is distinct and not verified yet.
- Final-constituent companions3846973: all seven original programs ACCEPT.
  Exact566 Rust accepts SL2C/PSL2C/SL2R (equality still requires full-tail
  review), rejects SU21/PSU21/Sp11 at theta-stable parabolics and GL2R at
  trivial-unitarity/inexact-halving checks. Preserve raw invalid cases too.
  Iterate EVERY term of finalize(raw), with coefficients, then normalize
  dominant parameters and assert finality before is_unitary. Never pick one
  constituent or assign unitarity to a virtual sum. See unitarity slice.
- PSp diagnostic4capture3846724 proves direct generation7versus8, not just
  pooled Hasse filtering. Full block restores row counts but cross links and
  KL entries remain wrong. Inspect pos_to_neg's initial set: original EMPTY,
  Rust ALL positive roots (complement); identity and single-reflection
  independent regressions must precede repair. Read HANDOFF for exact pins.
- User2026-09-29 requests real AND complex group testing. Catalog408 keeps
  original116 indices and appends36 named forms x8 operations plus4 D6/D8/
  E6/E7 inventories. Read docs/slices/real_complex_forms_2026-09-29.md.
- Original groups.at models a complex group as a real group with doubled
  root data and swapped factors. Use actual lattice rank for involution
  matrices and semisimple rank for simple-root loops; check central tori
  separately. Record actual form names/numbers, not just intended labels.
- Keep metadata/KGB/KLV/unitarity/Hodge/AV-ann/cycle/FPP as separate cases.
  FPP depends on root datum, not real form; same-datum repeats are controls.
  Bounded Phi matrices do not certify general cycle multiplicities or cutoff
  completeness. Original setup failures are not matching mathematical results.
- New survey uses exact566 source via release3846235 and its own63-test
  preflight3846269, both complete. Arrays3846276/3846277, reviews3846278/
  3846279 remain separate from older559 results. Never mutate a submitted stage.
- CPU review3846279 is FINAL:132cases,100math matches,1KLV mismatch
  (PSp4R),22Rust failures and9original failures. Report SHA
  544d4a966236b820fca771cedfae01bb4694b5803c92faecff6f2827b30449c2.
  AV-ann negative-bit-count failures include rank-zero integral subsystems
  and tori; reduce the cause before changing it. Original-rejected unitarity
  parameters need separately named valid companions, not changed old fixtures.
- Focused supplemental8case diagnostics3846542 preserve full cold/warm
  PSp4R partial blocks (original8rows versus Rust7at unit scale), SL2Rhalf/
  torus AV-ann stages, actual-lattice FPP rho companions and bitset duplicate
  controls. A bitset must be idempotent: current to_bitset adds powers, so
  duplicated indices are a separate source-level suspect; capture before fix.
- Diagnostic3846542 FINAL original accepts all7positives: duplicate-bitset
  idempotence now disproves Rust; AV-ann first fails character_table(T1).
  Original simple_factors skips T factors; current Rust includes them, causing
  order_W to attempt order_W_simple(T1)/highest roots. Freeze direct factor/
  order_W regressions before fixing; never relax the negative-bitset guard.
  PSp4R cold7/warm8rows proves history sensitivity: require both histories.
  Complex FPP displayed middle terms swap order; distinguish ordering from
  missing-point errors and retain exact-output coverage.
- CPU one-core diagnostic submission requesting8G was rejected by the current
  QOS limit of4G. Retain failed stage xuhAFqy7 (no job); fresh4G stage4iqDXKdR
  submitted successfully. Do not infer memory limits from old partition notes.

### Fundamental coordinates belong to the actual root/coroot spans

- Original267capture3846161 proves A1.A1Levi[1], root[0,2]/coroot[0,1],
  needs fundamental_coweight[0,1]/2, not Rust[1,0]/2. This loses the H[0,1]
  orbit; defining and optimized distinguished_Hs both inherit the bad input.
- Current fundamental_weight=e_i and zero-padded C^-1coweights assume a
  special coordinate basis. Original rootdata.cpp845-849uses R*(C^-1)^T
  and V*C^-1; preserve the actual lattice embedding and root/coroot spans.
  Pairing-only checks miss arbitrary central components. Use exact rational
  intermediates, not unchecked determinant products, and checked narrowing.
- Master273/reference3846359 accepts both complete corrected companions
  through D8/E8 and both SC/adjoint numberings; unchanged566Rust rejects both.
  R1ratvec*vec failed setups remain retained. Use builtin unfraction and
  integer numerator pairings; do not adopt partial stdout as a golden.
- Candidate569job3846487 uses exact rational inversion and actual root/
  coroot columns, plus ValidateThenDrop signed32index checking. Its three
  unchanged-before tests fail and three after tests pass in live logs; full
  gates remain pending. Original overflow text is "Integer value too big for
  conversion". Read docs/slices/fundamental_lattice_2026-09-29.md for pins.
- UPDATE3846487 FINAL PASS:508domain/569core/CLI/273capture/1493source guard,
  no lost positives; all lattice/error/historical gates passed. A1xA1four-orbit
  assertion is restored, but complete cycle trace stdout is still unequal.
  Reportf56c0ee29d2c5a2a020b0454b5028c08fd24ad40efcc4ecc9b78b0b586e22036.
- To enable traces read by already-compiled script functions, assign to the
  live flag (`kn_verbose:=true`); `set kn_verbose=true` creates a new binding.
  Retain accepted R1cycle_phi_a2_trace (it reproduces unequal Phi but has no
  intermediates) and capture its separately named live-assignment companion.

### Ordinary deform must use the parameter's actual common block

- UPDATE3848845 FINAL PASS, report
  c28f26b28e56ebe78fcf955dcac5e837c99111405ee1fb17071304666090bdf9.
  Unchanged bare original-stream regression,573core/511domain, complete
  live G2deform/unitarity trace, six unitarity controls,273+11no-loss,
  4script/4block histories and final integrity pass. Isolated G2only;
  PSp/GL2 failures remain on this source and require their own repairs.

- G2before3848829 FINAL proves1actual Runtime failure after valid markers,
 573inventory and unchanged production; reportc07a6902ad6db78b01e4b50b3d0538135e5ab3453e07b7e631611faa8e6866c5.
  Explicit trait-cross-only repair submitted3848845 inDzCDNnd6. Unchanged
  full golden,573core/511domain/full G2trace and downstream no-loss gates
  required; NOT after-verified. First ORWJAW9d staging correctly stopped
  before sbatch because direct captures had an old bridge/missing helpers.
  Stage complete verified bridge/merge/tail dependencies and use the trace's
  actual DEFORM_TRACE_INPUT marker, distinct from bare DEFORM_CROSS_INPUT.
- G2/PSp capture3848811 FINAL: all3originalACCEPT/RustRuntimeFAIL. Bare
  G2fails at1/3 (x7lambda[1,1],nu[0,1]/2), not the earlier factors1,7/9,5/9.
  Full original golden d220adfd is now a session regression; before3848829
  isolates573tests and unchanged production. Require its exact Runtime
  failure after READY/setup/recovery before editing cross. PartialBlock's
  inherent cross(generator,row) shadows trait cross(row,generator); prefer
  an explicit trait call, never a global argument swap. PSp's missing
  downward Cayley at x6lambda[4,3]/2,nu[28,21]/6 is a separate frontier.
- R2build3846124 FINAL FAIL (report16b90f644ec12dc5d3b8a02fd9ebe17b363528bd16e356cfc9b8edd3799e74c3)
  passes all4deform and2orientation positive regressions; the unchanged
  negative fails with3errors instead of2. orientation_nr was registered
  BuildAndDrop, not Skip: the completion-name list is NOT a no-value policy
  list. Read domain_builtin/domain_builtin_skip and the wrapper's actual
  gate (atlas-types.w6806). Preserve discarded-call recovery and map the
  stable make_dominant invariant to its original Runtime message.
  R3job3846153 keeps all seven tests/goldens unchanged; full acceptance
  remains open (stage8pYfaONh, pin4f1598fdf1f381204f459558f8e6df83c72bf1cfa7483b810a89b32d4a7ac7ff).
- Exact559CPUreview3845923 is FINAL:49math+3language matches,7rejections,
  4math mismatches,4Rust failures,1independent-invariant failure,18original
  failures. B2/C2/D4AV-ann newly match with no lost prior matches. Cycles
  A2/B2/C2/G2 now execute but outputs differ; A1.A1 has3complex orbits versus
  original4. These are retained failures, not downstream cycle acceptance.
- Distinguish two cycle failure boundaries: A1.A1 omits complex H[0,1],
  while A2retains orbit lists but loses Phi's x3lambda[1,1] term and changes
  T/Y/Q/PX. Trace Levi/H enumeration separately from Phi's twist/shifts/
  induction. Do not fabricate terms or weaken independent orbit/rank checks.
  Master267discovery is separate from frozen265deform gate; see indexed
  docs/slices/associated_cycle_frontier_2026-09-29.md.
- Original R2orientation3846059 accepts the complete265capture companion:
  G2x5 at gamma[1,1]/2 has orientation1, old Rust0; additional G2x6
  fractional rows are also low by1. A1/A2anchors still match. Nonstandard
  compact A1 rejects at value level but skips computation when discarded.
  Both the core wrapper and domain method need the SAME active algorithm;
  retain complex-pair evenness and use Euclidean FLOOR for real pairings.
  Current566candidate centralizes orientation in RepContext; R2job3846124
  in zaNFiERf (pin7069d5f014c0a2d04b8e502a6c1b4551a5df07999a1db8edcd61476f0ca7add1)
  requires seven regressions before/after,508domain/566core/265capture. It
  is unverified; no high-level unitarity acceptance follows from submission.
- The orientation negative has underlying Runtime and Type errors; original
  additionally prints a Program envelope. Assert BOTH individual messages
  and full survivor stdout, and preserve the full-stderr difference explicitly.
  Do not alter the classifier or claim aggregate categories/full diagnostics
  match when that envelope is still different.
- Orientation3846008 matches the whole historical A1/A2anchors, but the
  broad positive rejects script-only infinitesimal_character before math.
  Retain it and use builtin `%` to extract the same gamma in a companion.
  Master265/R2job3846059 also covers nonstandard value-producing and
  discarded calls. Rejected setup output must not become a numeric golden.
- R1job3845869 is FINAL FAIL (report bd56c0905d43e0d911bfb0fe43c9351b45f8a958ab730c314757d00896a7e260):
  four unchanged559before failures, then four after failures. A2/B2/C2
  reach the complete-output assertion, whose reports-only collector omits
  the original loop's `Value: [(),...]` line. Fix the collector to include
  nonvoid Value events, NEVER delete that line from the original golden.
  G2 instead fails exp_i's even-exponent assertion; preserve this invariant.
- Both the core orientation_nr wrapper and RepContext::orientation_number
  duplicate upstream's INACTIVE #if0 branch and overlook negative complex
  images. The comment forbidding the active branch is not proof. Master263
  adds all partial-block orientations for A2/B2/C2/G2 at singular/fractional/
  integral scales and reducibility points, plus historical A1/A2 anchors.
  Capture the current original before editing either implementation; do not
  assume source inspection alone identifies the G2 failing parameter.
- Matrix exact559build3845871/preflight3845872 PASS,1473source files equal
  foundation. Unchanged116consumers are submitted as CPU3845920/fat3845921,
  full/CPUreviews3845922/3845923. These do not contain the563deform candidate.
- Candidate563job3845869 is SUBMITTED in ziS3OXJP, pin c47d6807fff2b4c2643cc5f57edf5120b4459d1ba46c171c8c5fcb63f3262697.
  It depends on FINAL559matrix3845829, preserves4full original stdout
  goldens and requires4unchanged-before failures/4after passes,563core/CLI/
  frozen261capture without positive losses. No verified deformation repair
  or high-level unitarity acceptance yet; all invariants remain enforced.
- Original261capture3845837 ACCEPTS all4full deformation probes; old552
  panics in all4, including A2. Its stdout is buffered/empty, so do not
  infer the exact first failing command from the missing printed prefix.
  Four new whole-stream units now precede an unverified local wrapper
  repair using actual RepTable lookup;563before/after gate is being staged.
  Keep the unchanged `.oracle.stdout` artifacts and all invariant checks.
- When testing unchanged assertions in domain_builtins.rs, identify the
  final `#[cfg(test)] mod tests` module explicitly. Earlier cfg(test)
  instrumentation precedes production code; splitting at the FIRST such
  attribute falsely treats almost all runtime code as a tests-only tail.
- CPU3845474 B2/C2/G2unitarity failures are not grounds to remove height,
  weight-integrality or lambda-rho invariants. The ordinary deform wrapper
  still uses historical full-BlockGraph/first-matching-x/constant-lambda
  approximations. Original atlas-types.w8345 uses RepTable lookup with exact
  row and block modifier. See the indexed ordinary_deform_common_block slice.
- New master261 probes A2/B2/C2/G2 builtin deformation across singular,
  fractional/integral parameters, reducibility points and cold/warm history.
  Obtain full original capture before changing the wrapper; existing common
  deformation code used elsewhere is not blanket correctness evidence.

### Two-index matrix access is row/column, not column/row

- FINAL3845829 passes4unchanged-before failures,3newafter units plus
  corrected historical unit in all559core,CLI/257capture/final1473source
  integrity.117whole positives/no losses,8negative range errors and complete
  survivor stdout. Negative stderr presentation still differs. Standard
  release3845871/preflight3845872 now submitted for full116math consumers;
  no downstream cycle/AV-ann acceptance or speedup follows from units alone.
- R2original3845762 accepts BOTH full rectangular and historical2x2positive
  companions; old552fails both. Correct the historical unit's transposed
  expectations with this evidence, keeping unchanged-before failures.
  Candidate3845829 is SUBMITTED in JPbERXzU:3new+1corrected tests before,
  production-only axis patch,559core/CLI/frozen257capture after. Master261
  deformation probes and full116math are distinct; no accepted repair yet.
  Live3845829 now passes all3after units and all559core; CLI/capture/final
  source integrity still required before accepting or packaging the repair.
- CPU3845474 is now independently reviewed:46math+3language matches,
  7rejections,12Rust failures,18original failures; no full116acceptance.
- Matrix3845716 original capture proves row2 was wrongly accepted and can
  mutate a2x3matrix in Rust. Preserve all eight negative diagnostics and
  unchanged-matrix survivors, not only the aggregate Runtime label.
  Positive R1 failed because bare `assert` is SCRIPT-defined. Keep R1 and
  use the builtin conditional-error companion; do not accept partial stdout
  as a successful golden. Capture the historical2x2unit sequence before
  changing that old unit's transposed expectations (master257).
- Exact552full consumers now reach real calculations: A2cycle3845490 and
  B2AV-ann3845508 fail rectangular matrix bounds after library loading.
  Original axis.w4754/8647 checks (row,column); Rust's read/write/transform
  branches all reverse these axes. Preserve asymmetric2x3 fixtures before
  fixing, keeping ONE-index column selection unchanged. See indexed
  docs/slices/matrix_index_axes_2026-09-29.md. Master255discovery is separate
  from frozen251boundary and116math inputs; no repair accepted yet.
- B2unitarity3845496 reaches a distinct height-parity invariant in deform.
  Original repr.cpp asserts parity too; never remove the check as a fix.
- Rust panic logs may contain a numeric thread id between the quoted test
  name and `panicked at`. Boundary3845482 executed1pass/3intended failures,
  but its literal log guard failed. Preserve the report; corrected proof
  reruns identical tests in3845638, allowing ONLY the optional numeric id.

### Generic recursive groups after the polynomial544 gate

- Boundary3845705 FINAL: identical4regressions afterPASS,556core/CLI/251capture,
  all5new Syntax/stdout contracts and115whole positives retained. Report
  db3df68b46d7d4ae895f37052214ba2a4599f55e3f1bce6db6675ed476ab073a.
  Negative stderr presentation still differs; anonymous arrow printer remains
  unfixed. BeforeR2report737aa425 proves1pass/3fail on unchanged552runtime.
- Exact552standard3844955 and61-test harness3844956 PASS; full1459source
  identity verified. All116 consumers now CPU3845471/fat3845472, independent
  full/CPUreviews3845473/3845474. Early numerical matches are not a finalized
  review. Memory limits CPU6/fat24GiB are equal between engines.
- Original253arrow3845481 ACCEPTS full recursive function declarations:
  anonymous tuple/union arrow sides retain parentheses, raw tuple sides do
  not. Restricted root rejects bare formal, singleton parentheses and[*],
  but ACCEPTS primitive void. Never infer each error from the aggregate label.
  Same552capture of these new cases remains separate; no printer fix yet.
- FINAL same552followup3844760 captures251cases with no lost positives.
  The ENTIRE new boundary positive already MATCHES: its whattype injector
  signature does not exercise write_arrow_side. Do not cite it as a failing
  anonymous-function-printer regression. New arrow declaration and restricted
  root fixtures extend master to253; original discovery remains required.
  Five negative cases confirm wrong rejection phases (3Program/2arity versus
  original Syntax), with bare-alias recovery also different. Preserve all.
- Exact552standard build3844955 and harness preflight3844956 are submitted
  in atlas-recursive-group-release-20260929.MGjMmU4M. Do not submit116arrays
  until both reports and1459file equality pass. The old observer hardcoded a
  6GiB RLIMIT_AS even on fat32G; new explicit memory-limit argument needs
  HPC preflight, CPU6GiB/fat24GiB equally for both engines. Changed resource
  limits are capacity repair, not controlled speedup evidence.
- FINAL552job3844656 passes core/CLI/source/245capture with114whole positives
  and no losses. All four latest high-level imports now finish without
  diagnostics, but full stdout differs and numerical acceptance is separate.
  Run exact552all116math consumers; do not transfer536suite or older A/B data.
- Original251discovery3844689 disproves an assert-path hypothesis: direct
  group-local RHSs are rejected by parser.y974's RESTRICTED top-level
  typedef_type, before dissect_to. Do not implement these as Program errors
  or expand shared nongroup grammar to accommodate nested group references.
  Ordinary bare constructors remain Syntax errors. Preserve frozen251inputs.
- Anonymous recursive tuples in FUNCTION argument/result positions retain
  parentheses: axis-types.w2039 suppresses them only for RAW tuple/union
  kinds, not for anonymous tabled nodes. The full accepted boundary fixture
  prints `(GroupOld<A>,GroupOuter<A>)->GroupOuter<A>`. R1's unit expectation
  with naked anonymous arguments is provisional and WRONG; replace it only
  with this original-backed regression and keep the before evidence.
- FINAL3844249/3844321 pass544core/CLI/source integrity and full245capture
  with111whole positives/no losses. This does NOT mean high-level math passes:
  unitarity/Hodge/AV-ann/cycle all still fail lazy_lists.at5 generic recursion.
- Port the entire RHS type-expression graph, including anonymous tuple/row
  nodes on cycles; a named-equation-only graph loses nominal identities.
  Forward ALL group formals, mark actual recursion rather than every named
  alias, and reject existing recursive constructor instances only when they
  lie on a new cycle. Preserve staged type/member publication on every error.
- Original3843104 full streams/seven errors and3843356 anonymous-component
  positive precede the three new session regressions. See the indexed
  generic_recursive_groups slice before editing syntax or type resolution.

### While guards belong to the current loop's break boundary

- FINAL3843259 passes536core/CLI/final1436source integrity and234capture,
  retaining106whole positives/no losses. High-level imports now hit generic
  recursive groups and coefficient WRITES, not this repaired guard. Exact
  source116consumer and parallel gates remain separate; older LIVE notes below
  preserve history. Transaction3843795 has executed two before failures but
  is not yet verified. Never transfer frozen234/238/241catalog results.
- Original3843183 accepts COMPLETE mode/nested positives. R1nested fixture
  failed original grammar because a DoExpr branch ended at bare break; keep
  it, and use the companion with `break; dont`, not its accepted prefix.
  The independent INNER[[],[],[]] expectation fails as INNER[[]] before.
  Candidate3843259 uses distinct before/after targets,4identical regressions,
  full536core/234capture; before3fail/1pass and after4pass now executed.
  All536core also pass. CLI/capture/final report remains open; unit tests
  are not full mathematical acceptance.
  Master237is a different catalog.
- Subgroup3842990 FINAL passes532core/CLI/source and retains104whole positive
  matches in228capture. Four high-level imports still fail W_orbit.at329:
  flat while guards use outer Analysis and run outside the Break handler.
  Original axis.w5948 introduces the loop layer around the ENTIRE do-tree,
  and6134-6220 catches breaks from both condition and body.
- Repair both static depth and runtime unwinding. An inner guard's break
  must not exit the enclosing for, deeper breaks decrement exactly once,
  and interrupted iterations yield no row element/count. Preserve void,
  reverse, side effects, return propagation and lambda lexical boundaries.
  Do not move for iterables inside their own loop boundary by analogy.
  Read docs/slices/while_guard_break_2026-09-29.md for before/after evidence.
- Frozen subgroup228, recursive230and guard233catalogs are different inputs.
  Never update a submitted stage to the latest master checker/catalog.

### Existing builtin names can hide missing mathematical overloads

- FINAL transaction3843795 passes538core/CLI/final1440source integrity and
  238capture, both rollback streams and all106parent positives. Stage types,
  overloads, locations and reports together; simple aliases retain a distinct
  contract. Generic SCC/parser work remains open. Report
  0284810cbf6801ed922d975b2c36840156947eb3a6f8735af3c12b26f7f5b0f4.
- Nongeneric group3843306 exposes actual wrong acceptance independently of
  generic syntax: duplicate fields replace AtomicKeep/install AtomicOther,
  and a function identifier becomes an inaccessible type. Original rejects
  both atomically. Keep full saved/fresh/survivor output and source-backed
  diagnostics. Transactional publication must cover type names AND members.
- Existing recursive aliases copy tag metadata:3843306's unqualified case
  becomes ambiguous between ComponentTree and ComponentAlias. Preserve this
  original rejection, not a fabricated recursive-constructor failure. A
  separate companion prints the complete child and spaces constructor > =.

- Frozen subgroup532/228job3842990 and later recursive-group230discovery
  3843104 have different catalogs. Never copy a newer master checker into
  an active old stage. Generic recursive groups need argument forwarding,
  recursive SCC identities and transactional failure, not grammar alone;
  see docs/slices/generic_recursive_groups_2026-09-29.md before implementing.
- Original3843104 accepts complete generic int/bool streams and rejects all7
  grouped-definition errors while preserving old constructors/values. Its
  aggregate REJECTED_TYPE_ARITY label is not proof of seven checks: assert
  each diagnostic and full survivor output. Original stages projector/
  injector overloads in two passes; publishing the type table before field
  validation can leave a partially updated context. Keep failure atomic.

- Causal diagnostic3842817 verifies that ONLY forwarding g in the two header
  wrappers repairs the minimal invariant AND entire broad orbit/witness
  fixture; invalid-generator streams are unchanged. First3842703 failed
  before math because patch context omitted the space in `to_codominant (`.
  Keep both reports; patch success or a modified original alone does not
  establish Rust acceptance. See the indexed subgroup slice.
- The new subgroup candidate uses exact pairing-coordinate coset BFS:
  positive rescaling on the active subsystem preserves signs/edges, and
  finite Cartan invertibility preserves equality on orbit differences.
  Check full order, independent reflection closure and ambient witnesses;
  do not accept matching cardinalities alone. Keep checked coordinate
  narrowing and distinguish safe rejection of original undefined dimensions.
  Seven new controls require full532core/228capture; candidate is unverified.

- R3original3842475 gives a mathematical counterexample: A2 generators[],
  input[-1,-2], matrix orbit[2,1] but identity witnesses. The empty subgroup
  must fix its input. rootdata.h669/678 ignores g when forwarding
  make_(co)dominant; witness functions pass g correctly. Never port this
  wrong matrix as a golden. Preserve the assertion and compare an explicitly
  diagnostic two-line original patch separately, not as a replacement oracle.
  A math-correct divergence needs explicit scoped evidence; do not claim
  complete original compatibility while this discrepancy remains.
- Original3842256 rejects a provisional positive at adjoint(RootDatum)
  BEFORE orbit mathematics. The bare constructor is adjoint(LieType,bool).
  Preserve the rejected fixture; add a companion changing only the constructor
  and retain all group/lattice/witness checks. Nine original invalid-generator
  runtime controls do execute, but they cannot validate positive orbit values.
- Tagged3842065 passes525core/CLI/source and complete suffix positives but
  high-level imports still fail at W_orbit.at127. Current original exports
  four additional subgroup Weyl_orbit/Weyl_orbit_ws signatures; completion
  names alone cannot detect this gap. AV-ann also has an independent
  lazy_lists.at5 recursive-type grammar failure. Preserve both frontiers.
- Subgroup generators are arbitrary signed root numbers with valid Cartan
  pairings. Original validates them before no-value, follows user order for
  dominance/extensions, but INTERNAL RootNbr order within stabilizer BFS.
  Do not use the alcove sorted-layer BFS or return subsystem-owned witnesses.
  Capture full orbit columns AND ambient Weyl words. See indexed
  docs/slices/weyl_subgroup_orbits_2026-09-29.md before implementation.

### Canonical sections are not arbitrary solutions

- Final3842014 passes508domain/523core/CLI/source gates and complete expanded
  canonical plus D8/E7/E8 metadata equality:102whole positives versus99/no
  losses in217capture. Keep all116consumer and refreshed parallel gates
  separate; original signals mean CANONICAL_GATES_PASS_ORACLE_UNAVAILABLE,
  never blanket mathematical acceptance.

### Tagged-case coverage is checked in source order

- Original223discovery3842032 rejects missing/duplicate labels and
  multiple/spurious defaults; old Rust evaluates the first four bad cases
  and prints1. A later error cannot hide those wrong acceptances. Reject a
  duplicate BEFORE analyzing its body, but preserve an earlier body's error
  before any later duplicate. Original axis.w5711/5812/5828 is authoritative.
- Original222discovery3842018 accepts the whole isolated ordinary suffix
  fixture. Compound fixtures signal in while for BOTH prefix and suffix
  spellings; retain them as invalid oracle execution, not rejection goldens.
  The independent subject-stop while control matches both engines completely.
  See slices/tagged_case_suffix_2026-09-29.md for candidates and required gates.

- Kernel3841963 executes the three wrong-election assertions on unchanged
  runtime;3841994 passes the identical tests plus exhaustive3x4binary maps,
  source-mask64boundary and dynamic target130controls. One shared canonical
  section now serves seed/strong-real paths, preserving column priority and
  ordered fiber output. Full508domain/523core/217capture3842014 is separate:
  kernel success is not complete metadata or downstream math acceptance.
- Expanded original3841964 also exposes SC D4/D6 mismatches. Small split
  forms and zero preimages hid these elections; cover all real forms and
  both numberings, not just larger split examples. Keep full output streams.

- Full212capture3841698 exposes D8/E7 initial_torus_bits and ordered
  central_fiber differences despite successful execution and523passing core
  units. E8 metadata alone newly matches. Preserve the complete streams;
  same gradings/cardinalities do not prove canonical representative equality.
- BinaryMap::section discards dependent IMAGE columns as pivots. Reducing
  augmented[image;marker]vectors in the full space can introduce kernel
  pivots and elect another preimage (columns[1,1],target1: mask2 versus1).
  Capture before failures, repair the seed and strong-real elections at the
  shared boundary, and revalidate full consumers. Do not sort central_fiber
  to conceal a wrong section. See slices/canonical_seed_sections_2026-09-29.md.
- Review3841833 failed BEFORE classifying mathematical outputs because
  REVIEW_INPUTS_SHA256 received input_list_sha256 (inputs.sha256) instead of
  suite_inputs_sha256 (suite-inputs.json). Bind each hash to its actual file;
  preserved raw observations need a corrected independent review, not
  interpreter reruns. Cancelled unstarted wrong-pin review3841832; corrected
  reviews3841968/3841969 use the unchanged binaries and all116inputs.

### Lazy access must propagate through printers as well as computations

- First lazy candidate3841545 fails cargo check --tests with E0425/E0277:
  print_kgb and involution_expression still have infallible String signatures
  after their graph/table accesses become fallible. Preserve this compile
  report; it is not a mathematical failure or a test pass.
- Carry Result and the requesting SourceSpan through renderer callers,
  including common-block output. Test the error boundary at actual printers
  and representation access, not only graph(). Existing-value Display must
  not trigger new mathematical initialization. R2needs full523core/CLI plus
  original-backed212capture and full116consumers; see lazy_real_form slice.
- R2job3841592 catches a test-only E0599: KgbId is opaque and has no public
  from_usize. Obtain a valid id from an independent healthy graph when
  testing failure on an uninitialized graph; do not fabricate a sentinel or
  broaden the production id API. Keep both compile failures and all assertions.
- R3job3841627 compiles and passes5of6newlazy tests; metadata setup fails
  because numeric id_mat is not in domain_builtins::call. Domain-only units
  must construct the same Matrix value directly or use the full session
  layer. Keep all metadata assertions and the failing report; a setup error
  is neither an incorrect mathematical output nor proof of those assertions.
- R4job3841689 passes all523core tests (including all6lazy regressions),
  CLI release and final source integrity. This closes compilation/owner-unit
  gates only: require the original-backed212metadata/import capture and
  source-identical116mathematical consumers before accepting the migration.
  Existing eager517 E7/D8 parallel results cannot validate lazy523 by proxy.

### Direct generation, explicit budgets and lazy-owner boundaries

- Integration3841489 verifies517core/10Cartan units and all retained kernel
  controls, CLI check/release and1423-file integrity. The interpreter now
  selects generated twisted involutions; historical UNUSED API notes refer
  to the earlier3841461 kernel only.209capture3841497 verifies complete
  D8/E7/E8 Cartan outputs(98whole positive matches,3gains/no losses), while
  high-level imports still time out.116math and parallel A/B gates remain
  separate. See slices/direct_twisted_generation_2026-09-29.md.
- Keep the legacy Weyl budget distinct from the optional generated-involution
  limit, including identity and0/3/4boundary tests; include mode AND limit in
  classification cache keys so a warm larger result cannot bypass a guard.
- Original innerclass.cpp218-293 discovers canonical Cartan representatives
  and cartanclass.cpp1041-1057 computes orbit sizes by exact Weyl-order
  quotients. Local weyl_size.rs is currently unwired; comments mentioning an
  on-demand path do not prove it exists. Do not claim unmeasured savings.
- Before lazy RealFormContext migration, retain full metadata fixtures for
  D8/E7/E8 alongside full KGB/high-level cases. Logical-owner comparison must
  not initialize KGB. Preserve canonical weak caching, concurrent winner,
  custom-owner separation and fallible source-span diagnostics; no panic
  Deref or fabricated empty graph. See library_context_loading slice.

### Zero-rank torus extension is the identity, with validation before discard

- Combined3840634 completes516core/CLI/final integrity and all three full
  torus streams plus extension positive. Negative CLI stdout/category match,
  stderr envelope differs. groups.at now times out45s with NO output; do not
  infer a successful load or last executed group from that absence. Require
  exact-source release/shared-consumer validation after shared math repairs.
- Original3840609 accepts the complete extension/G2involution fixture;
  unchanged511-core keeps a T0 factor and fails Too few inner class symbols.
  Tn must be stored as n T1 factors, not one Tn factor. Preserve all90torus
  controls and full matrices, not merely rank sums or groups.at loading.
- All17invalid extensions reject originally; old Rust accepts illegal
  letters/ranks and discarded dynamic calls. Port letter/per-factor/total
  bounds and signed32 narrowing before unsigned32 rank validation, including
  -1 ->4294967295. Keep exact diagnostics/recoveries and BuildAndDrop policy.
  See indexed slices/lie_type_extend_2026-09-29.md; full repair gates pending.

### Root-free kernels and exported bases must retain rank and orientation

- Full514-core3840560 catches the historical A1.T1 adjoint-rejection assertion
  (513pass/1fail). Original3840488 explicitly accepts the same exact input
  in its complete28datum fixture. Retain the failure and migrate to that
  captured type/value, not a skipped test or a Rust-derived expectation.
  All three focused goldens passed but did not replace the full-core gate.
- Kernel3840555 proves identical root-free tests fail before and pass after
  an explicit0xN annihilator, with7root-datum/2alcove controls and separate
  targets. Do not substitute this for full golden session/shared-consumer
  validation of the accompanying column/adjoint fixes (candidate3840560).
- Discovery3840488 proves SC T1 loses both radical/coradical basis rank and
  adjoint(T1) is wrongly rejected. An empty annihilator is0xN, not0x0; keep
  lattice rank before passing it to saturated_kernel. The historical blanket
  adjoint torus exclusion is contradicted by the current original.
- Independent3840498 reaches B2: original root_coradical is[[2,-1],[-2,2]],
  Rust gives its transpose. Current atlas-types.w1679-1703 exports BOTH bases
  by columns; the Rust comment claiming rows is false. Symmetric A2 and
  shape-only checks hide this error. Preserve full matrix coefficients.
- Pure T1 alcove_center should retain all central characters; original's14
  controls pass, Rust reports no unique solution because radical rows were
  lost. Revalidate this consumer after any kernel repair. See indexed
  docs/slices/torus_radical_2026-09-29.md. These repairs remain pending.

### Empty explicit root data retain matrix dimensions

- Original3840368 accepts Nx0 simple-root/coroot matrices, including0x0;
  unchanged509-core rejects them and blocks groups.at initialization.
  Preserve the full output and rank/shape/duality assertions before repair.
- as_matrix_rows represents0xN as N empty rows for legacy shape validators.
  Do not remove an empty guard after that lossy conversion: use the actual
  Matrix dimensions at the explicit constructor, leaving unrelated adapters
  unchanged. Original validates dimensions/Cartan even for discarded calls.
- Preserve empty dimensions on export too: transposing an empty simple-root
  vector list loses lattice rank. The dimension-aware columns_matrix_value
  retains Nx0 for both simple_roots/simple_coroots.3840381 completes511core,
  CLI, final integrity and full original positive equality. groups.at still
  stops217at a later involution-symbol error, not mathematical acceptance.
- The combined negative signals in original at explicit empty vector-list
  conversion. Keep it as invalid oracle execution, and isolate matrix-only
  rejection and signal companions. Never use a crash as a rejection golden.
  See indexed docs/slices/empty_root_datum_2026-09-29.md.

### Signed arithmetic requires independent canonical-output checks

- Projection3840261 completes507core/CLI/source guards and full original
  T2/A1.T2 output,88whole positive matches/no losses. Pair this with kernel
  3840260's unchanged before failures/after passes; neither replaces broader
  revalidation of shared projection/alcove consumers on the combined release.

- Discovery3840264 proves two further unsigned-magnitude errors: A2 negative
  coroot expression[1,1] cannot reconstruct coroot[-1,-1], and alcove_center
  maps negative A1 nu to positive centers. Preserve all712 root reconstruction
  checks and full signed-center output before repair. For integral rationals,
  use signed TryFrom<&Rational>, not numerator_ref. See the indexed
  slices/signed_rational_2026-09-29.md; other conversions remain separate.

- Polynomial/sign3840188 passes506core/CLI/integrity and complete coefficient
  output; basic.at now loads without diagnostics, but declaration text still
  differs. Loading a prerequisite is NOT unitary/Hodge/AV/cycle acceptance.
  Revalidate the full math catalog on its exact release source. Projection
  kernel3840260 separately proves3before-fail/1before-pass ->4after-pass plus
  19controls; require the complete original parameter-output gate too.

- Original3840186 and Rust both accept skew T2/A1.T2 and pass coset equality,
  but lambda[8,5] versus[4,3] exposes a canonical-lift mismatch. RealProjection
  gcd_sweep's truncating quotient leaves a negative remainder/pivot; upstream
  matreduc.h uses Euclidean arithmetic::divide. Exact factorization still
  holds with the wrong basis orientation. Keep complete basis/KType/Param
  outputs and direct signed-column tests; see slices/signed_projection_2026-09-29.md.
- Polynomial3840181 fails unchanged original output because rational_pair
  treats Malachite numerator_ref's unsigned magnitude as a signed numerator.
  Restore sign BEFORE narrowing to preserve i64::MIN. Preserve the failing
  full coefficient regression, signed/boundary test and full after gates.
  Other numerator_ref call sites need independent sign-reachability evidence;
  do not assume every unsigned numerator conversion is erroneous.

### Polynomial coefficient reads are not positional subscription or writes

- Live-destination3844021 is FINAL539core/CLI/1442sources/241capture with
 107whole positives/no losses (report69ed01a390e698b8702f85a6ca5ede3603a01af3e9dfb7ead8474863a10eff21).
  New coefficient-write candidate544 is unverified. Preserve FULL operator
  calls: factoring away the left operand loses coercions/read-before-RHS.
  Dynamic keys need the hidden outer initializer plus a reanalysed deeper
  assignment; bare identifiers must still be reread on write. Validate final
  keys even for zero/discard and replace coefficients, never accumulate them.
  Foreign-owner rejection is explicitly mathematical divergence from proven
  original key corruption, not blanket compatibility. See polynomial slice.
- Discovery3843926 CONFIRMS lost nested updates: original[7,7]vs Rust[7,2],
  including local/captured/vector controls and RHS rebinding. Keep the full
  component_assignment_live_target fixture/golden. Candidate3844021 checks
  initialization cheaply before effects and rereads afterward; before test
  fails, same after test/full539core now pass; CLI/241/final still open. See
  slices/component_assignment_live_target_2026-09-29.md.
- Foreign writes originally give KKEY_RETAINEDfalse/PKEY_RETAINEDfalse,
  then reject reads by supplied keys. Do not port owner corruption as a golden.
  Polynomial transforms have a distinct fallback/hidden-index-let path
  (axis.w9168+), not optimized row RHS-before-index. Capture index/identifier
  mutation before implementation;243probe3844074 is separate from241build.
- FINAL3844074 accepts full transform effects: ordinary assignment ORDER12
  but dynamic polynomial transforms ORDER21; a bare identifier key is reread
  after RHS mutation. Reversed writes work while reversed transforms reject
  through the synthetic read. All9boundary errors retain finality validation
  for zero/discard. Port the actual fallback, not one generic update order.
- Chained Hodge writes require a LIVE assignment destination. Original
  axis.w8497holds the cell reference across RHS and index evaluation, then
  uniquifies; Rust currently clones before those effects. Capture ordinary
  row/vector and polynomial nested-write/rebinding controls before repair.
  Foreign-form writes may reinterpret keys under the target's owner; retain
  enumeration/equality/read-back diagnostics, never bless them as goldens.
- Original3840100 proves receiver-before-key evaluation, exact KType/Param
  indices, equal-real-form compatibility, final KType versus standard Param
  validation, and dominant-copy Param lookup. Validate discarded reads too.
  Reversed/wrong-key expressions give Program, not type-unification errors.
  Keep complete coefficient output and all ten rejected calls/recoveries.
- Original cross-real-form coefficient WRITES succeed despite analogous
  reads rejecting. Preserve that surprising output and investigate its
  mathematical safety; do not turn provisional intent into a rejection rule.
  See indexed `docs/slices/polynomial_coefficients_2026-09-29.md`.

### K-type formula bounds and memoization have separate contracts

- Euclidean candidate3840172 passes503core, CLI and final source integrity;
  full formula output now matches original. Broader replay3840179 also
  matches the entire370107-byte coset output on A1/A2/B2/C2/G2 (both
  numberings/lattices/inner signs), where unchanged3840085 failed3840175.
  This is scoped correctness evidence, not high-level mathematics acceptance.
- Kernel3840173 correctly FAILS its zero-test guard: real_projection.rs has
  no tests module. Nine KType and ten context tests did pass. A fresh rerun
  uses the actual original-backed B2 transported-lift anchor; do not report
  nonexistent direct projection tests. Separately inspect the remaining
  false truncation comment in RealProjection::gcd_sweep via signed-echelon
  and skew-torus/product original-backed discovery BEFORE any repair.

- Candidate3840098 executes the full original-backed formula assertion and
  fails: A2 x=1 produces lambda[1,0] instead of[0,1], and x=3 splits an
  original coefficient-2 into two noncanonical-1terms. Both raw and memo
  agree with each other while being wrong. lambda_unique used v/2 because
  its comment falsely called arithmetic::divide truncating. Read the actual
  arithmetic.h249-253: divide(-1,2)=-1, so use div_euclid(2). Preserve the
  complete before-failure and rerun every original coefficient unchanged.
- Recovery prints produce internal void Value events suppressed by the CLI.
  The signed32 rejection unit must forbid NON-VOID results, not all events;
  keep all six Runtime errors and exact recovery reports. This test-envelope
  correction does not justify altering the separate mathematical expectation.

- Original3840093 rejects signed32 overflow before the no-value gate. Old
  Rust returns memo polynomials at2147483648/-2147483649 and continues after
  a discarded invalid call. Keep all six rejected calls and recovery markers;
  a missing raw interface's Name error must not hide the memo wrong acceptance.
- Memoization may reuse a higher cutoff, but export must truncate to the
  requested height. Preserve strict K-type identity and real-form ownership,
  negative/unbounded controls, cold/increasing/decreasing histories, and full
  coefficients. Pointer sharing is not performance evidence. See indexed
  `docs/slices/ktype_formula_2026-09-29.md` for source and regression pins.
- Stored KL/Q presence queries may materialize a block through lookup; they
  must not fill missing polynomial columns. Deformation caches use canonical
  units/completion flags, not merely the exact top-level parameter key.

### Dot selectors accept full units and evaluate the callee first

- Original3840081 accepts grouped recursive lambdas, begin/if expressions,
  chained calls and generic all_words through receiver.unit. parser.y's
  selector:unit is not limited to scalar literals. Share unit grammar while
  keeping the bare empty tuple separate, and preserve identifier/operator
  resolution. See indexed full_unit_selectors_2026-09-29.md.
- Original ORDER821 records selector effects before receiver effects.
  Keep ordinary application order and the complete positive stream plus four
  name/type negatives and recoveries. Never modify basic.at or special-case
  its mathematical function to get past the missing syntax.

### Rank-two subsystem labels preserve root-number order

- Corrective F4Cartan3840078 matches ranks/words/involutions but reports
  realC2 where original coroot numbering givesB2. Root numbering correctly
  givesC2. dynkin.cpp94-116 intentionally lets ordered Cartan entries choose
  B2/C2; sorting by first ambient nonzero coordinate is not RootNbr ordering.
  Carry the datum's prefers_coroots choice and preserve both controls. Never
  rename C2globally or call isomorphic rank2 labels nonisomorphic systems.

### Verify executed timeouts and bare-core fixture operations

- Suite3840709/3840710 fails the input guard BEFORE either interpreter:
  SUITE_INPUTS_SHA256 was exported to preflight but not to sibling arrays.
  sbatch --export for one job does not populate later submissions. Pass the
  manifest hash explicitly to EVERY array, alongside timeout/build pins.
  Preserve failed reports (empty observations) and replay in a fresh suite
  stage with the SAME verified binaries/inputs; no rebuild or weaker guard.
- Independent review3840066 retains all58 reports:38math matches,9rejection
  matches, E7 FPP timeout, D8 capacity failure and9original failures. Eight
  Cartan inputs fail before mathematics because int*mat is script-defined;
  use conditional id_mat versus0-id_mat, keeping every Cartan/numbering/
  lattice variant and the original failed source/report in its frozen stage.
- The old math_suite.sbatch read MATH_CASE_TIMEOUT, not TIMEOUT; its Python
  driver also rejected values above600. The prior receipt's intended600/1200
  settings therefore did not describe execution: all58 used300seconds.
  Read observation.command and timeout_seconds, not just sbatch exports.
  Preserve the original300s failures, test environment forwarding/conflicts
  and the1200s CLI boundary on compute nodes, then rerun in a fresh stage.

### An accepted original stream can still expose an original defect

- Named-update3839981 passes the full positive but fails its unchanged
  rejection unit: an absent IDENT was lowered as a symbolic call and got a
  Type error. Preserve named-callee identity through component/field lowering
  and resolve it before arguments, as ordinary identifier applications do.
  Do not weaken the undefined-name assertion or silently change all symbolic
  operator diagnostics without separate original-backed evidence.

- Original3839528 named_update_effects prints FIELD74 then END(7,9)12,
  unlike its mutating builtin field update. Current axis.w global fallback
  passes selector and position in the opposite order from the constructor;
  assign then indexes the tuple unchecked. Treat this as suspected original
  undefined behavior pending checked-build/GDB evidence, not a no-write rule.
- Preserve the full effects fixture and isolated global/local controls.
  Diagnostic3839980 uses source-identical -D_GLIBCXX_ASSERTIONS and an unchanged
  release binary. Never manufacture a Rust golden from an out-of-bounds write
  or remove a mismatch to make the gate green. See the indexed named-update
  slice; checked-original completion is not mathematical acceptance.
- Diagnostic3839980 CONFIRMS the bounds defect: checked original aborts on
  the retained effects and isolated global fixtures; GDB sees index315 into
  a two-field tuple at field_assignment::assign. Release also segfaults on
  the isolated global case. Both builds agree on local UPDATE74/LOCAL(74,9).
  The old release's unchanged global tuple is undefined behavior, never a
  Rust mathematical golden. Preserve the mismatch and label this exception.

### Integral subsystem simples are not ambient-coordinate minima

- Original3839528 B2 gamma[1,2]/2 has integrality rank2 and datum A1.A1;
  Rust3839502 returns rank1/A1.T1. At gamma[-1,0]/2 original dominance is
  false but Rust returns true. Both explicit mathematical assertions fail
  in unchanged Rust; preserve integrality_subsystem_boundary.atlas.
- simple_basis incorrectly removes a root whenever another subsystem root
  has smaller AMBIENT simple coordinates. Use the reflection criterion:
  a subsystem simple reflects every OTHER positive subsystem root positively.
  This helper also feeds FPP and Cartan complex factors; revalidate them.
- Preserve upstream RootNbr order and classify a subsystem's actual lattices:
  full rank does not imply simply connected. All integrality wrappers validate
  vector dimension before their no-value gate;3839528 captures five wrong
  acceptances separately. Candidate repair remains unverified.
- Root-interface3839527 compiles, but its full classical/exceptional session
  fixture aborts with stack overflow before an assertion. Do not weaken the
  fixture or raise the default stack as a fix. Keep the independent mathematical
  before evidence and diagnose the exact binary's stack on HPC separately.
- Exact-binary GDB3839541 identifies convert_expr_context's recursive analysis
  frame (at least135168bytes), not evaluation. A diagnostic16MiB stack reaches
  the unchanged output assertion, which fails. Partition analysis families
  mechanically while preserving the same contexts and normal-stack tests;
  do not misattribute the fault to the separately large domain dispatcher.
- Combined3839978 verifies493core/CLI/final integrity on normal stack and
  complete original/Rust root/B2 positive equality. Retain the exact before
  failures and all235root lines. The shared helper's broader FPP/Cartan/KL
  release revalidation remains mandatory; one repaired counterexample is
  not whole-mathematics acceptance.

### Completion inventory is not implementation coverage

- Root discovery3839518 accepts the full seven-group/two-numbering fixture.
  W_refl narrows to signed32 before root-index checks; both it and
  integrality_simples validate at no-value level. Export simple roots in
  upstream signed RootNbr order, not enumeration order. Preserve all235
  report lines and full differential output, not just rank/involution checks.
- CompactA1 stored queries start with none; warming KL, full deformation or
  twisted deformation yields some for that cache but leaves Q absent. A
  stored query must not perform a missing computation or infer cache presence
  from another cached object. Keep cold/warm histories as distinct evidence.

- Original3839492 captures309 startup names plus three system variables.
  The historical294-name list is stale. Preserve exact order and the trailing
  space in hidden "## "; visible generic ## replacement must not affect iffor
  flattening (original3839480 gives visible[], hidden[0,2]).
- Nine latest names still lack registrations: query, system,
  integrality_simples, W_refl, K_type_formula_raw, stored_full_deform,
  stored_twisted_full_deform, stored_KL_sum_at_s, stored_KL_Q_polynomials.
  Do not claim mathematical support merely because completion lists a name.
  Historical hidden transpose/matrix-slicer copies are a separate parser case.
- axis.w2815-2945 catches constant-folding errors and installs a frozen_error
  expression; it throws when evaluated, including at no-value level, not by
  immediately aborting analysis. Preserve dead-code versus evaluated-constant
  distinctions when repairing container no-value propagation.

### General for loops carry mathematical keys and two independent orders

-3839452 distinguishes constant-folded and dynamic vector division: literal
  `(vec:[1])/0` rejects even in a discarded position, while an operand with
  mutation gives EFFECTS1 without a denominator error. global.w5148 registers
  folding, and the runtime wrapper gates result construction. A no-value
  regression needs nonconstant operands plus retained constant-error controls;
  never infer runtime validation policy from a folded expression alone.
- Combined3839432 passes the unchanged VOID6 assertion, all483core tests,
  CLI and final integrity;74of159full language streams match, no old losses.
  This advances basic.at only to378's filtered for. Keep high-level math
  gates separate and use the pinned report before reusing this candidate.

- Void-consumer discovery3839442 rejects a comma let whose RHS is(int,int)
  but pattern is((),k); unchanged Rust wrongly accepts. For let, axis.w3170
  supplies void context only for a WHOLE empty-tuple pattern, then checks
  ordinary patterns against the inferred RHS. Do not generalise assignment's
  prepared tuple-component coercion rule to binding declarations. Keep the
  rejected discovery and add a valid companion rather than weakening it.
- No-value does not mean skipping all validation. Original3839442 rejects
  discarded integer division by zero; divide_wrapper validates BEFORE its
  level gate. Tuple/list displays pass no-value to their elements, while
  individual operations retain their own validation policy. Distinguish
  overall-void lists from collected-void components, and component assignment
  from field assignment; see slices/void_boundaries_2026-09-29.md.

- Original3839396 confirms coerce-to-void can retain values (SCALAR7,
  GLOBAL42, CALL6), while prepared void tuple components and collected void
  row/loop bodies discard them. Follow axis-types.w5082 plus each explicit
  consumer; do not treat all static-void expressions as unit-valued runtime
  nodes. The prior3839328 Rust already showed VOID(), so retain this as a
  pre-existing wrong-value regression. See slices/void_boundaries_2026-09-29.md.

- Candidate3839386 passes four new iteration fixtures but fails the unchanged
  context fixture: original VOID6 versus Rust VOID(). Do not change the
  expectation or patch a loop's last value. Audit coerce-to-void separately
  from explicit discard consumers; casts can include a following sequence.
  Keep normal-stack/full-core gates after any shared evaluator change.
- Discovery3839388's two A1 inputs canonicalize to the same Param/KType.
  Verify the actual number and identity of nonzero terms before calling a
  fixture multi-term coverage. Retain the useful coefficient-cancellation
  control and add a distinct-key companion rather than rewriting evidence.

- Original3839328 accepts seven receiver kinds, including ParamPol and
  KTypePol. Their @ binding is the actual Param/KType key and the component
  is its Split coefficient, never an integer position. Preserve canonical
  nonzero term order and the owning real form; one-term examples do not
  establish cancellation or ordering correctness.
- Input reversal preserves original storage indices; output reversal changes
  only gathering order, including the completed prefix after break. Error
  iteration numbers count traversal steps. Each iteration owns a fresh frame
  and key/component handles so closures cannot observe the last iteration.
- Establish the loop body's row-component, void or row-coercion context
  before analysis. A for loop has no while-style integer-counting mode.
  Evaluate discarded bodies at no-value level and avoid constructing result
  rows. Empty binding layers behave as anonymous counted loops.
- ByteR2 3839335 passes475core/CLI/integrity and moves latest basic.at loading
  to reversed counted for at183, but is not high-level mathematical acceptance.
  General-for candidate3839386 is separate and must pass its five captured
  fixture units plus full core/CLI/stream gates. Counted signed64 boundaries,
  named-void contexts and multi-term polynomials require separate discovery.

### Byte strings require a complete value-to-output boundary

- Byte build3839304 failed at compilation because two historical session
  test value-extractors omitted the new ReportBytes/OutputBytes variants.
  Audit test-only exhaustive matches as well as production event plumbing;
  keep the failed compile report and rerun the full inventory without
  weakening any assertion. A library API migration is not complete when
  only ordinary source consumers have been updated.

- When migrating Atlas strings, preserve arbitrary bytes through values,
  nested printers, to_string, events, CLI/redirect sinks and runtime messages/
  back_trace; a safe Vec<u8> wrapper needs no unsafe code. Display is only a
  Unicode editor/debug preview, never the authority for interpreter bytes.
  Domain parsers must consume bytes or explicitly validate text, not insert
  replacement characters. Raw stderr3839257 contains isolatedc3/a9 too.
-3839257 rejects every exact historical lowercase ascii unit source as an
  undefined identifier. Current global.w4147-4148 spells BOTH overloads ASCII.
  Preserve the old sources; migrate spelling together with completion names,
  and retain a lowercase rejection test. Do not keep an unverified alias.
  The same discovery rejects string '+'; concatenation is '##'. Keep that
  rejected transport fixture and a separate valid companion.
- One-dimensional slice bounds undergo long_val narrowing separately in the
  order upper, lower, then receiver evaluation. Range/empty checks precede
  conversion to storage indices. Strings use bytes, matrices use columns and
  retain row count when empty, rational vectors normalize after selection.

### A large evaluator match can overflow a shallow recursive call

- Exact-binary GDB3839217 traps in TypedExpr::evaluate's stack probe.
  Detail3839222 measures0x17c28(97320)bytes per debug invocation. With a
  diagnostic-only16MiB stack, the SAME test evaluates RECURSIVE120 and reaches
  its separate parser-induced assertion failures. This is not infinite
  factorial recursion and not a reason to change mathematical expectations.
- Reduce unrelated temporaries carried by every call: partition evaluator
  arms into small, non-inlined families while preserving their bodies and
  evaluation order. Require the unchanged test to pass on the normal stack,
  plus full-core/CLI/full-stream comparison. Enlarging RUST_MIN_STACK is not
  a repair or an accepted gate; no performance claim from debug timings.

- Transfer timeouts can leave a partial candidate.patch (job3839224 stops
  at its input guard before compilation). Keep the failed stage; compress
  transfer artifacts and ASSERT remote checksums before sbatch, not merely
  print them. Never repair the bytes in an already submitted stage.

### RETURN belongs at assignment level, above sequencing

- Original3839128 rejects int:begin return 3/2;0 end. Candidate3839163
  still accepts it: its grammar used RETURN expr, swallowing the semicolon
  and returning the later integer instead. Current parser.y:341 uses
  RETURN tertiary. Keep grouped sequencing distinct, and cover NEXT and
  assignment operands as well as the unchanged full structural fixture.
- A shared return-type repair cannot compensate for a wrong parse tree.
  Preserve the before failure and all original-backed assertions. The same
  candidate also aborts in the positive structural-return test; investigate
  that separately, never weaken the assertion or hide it with a skipped test.

### Function-component matching must preserve a named context

- Candidate3839117 passes459core but loses three previously matching named
  function declaration streams (numeric results still agree). Its
  matches_function_component overwrote the whole context with expanded_top.
  Use a temporary expanded view to locate argument/result components and
  retain the named/applied body for substitution/export. Do not patch display
  strings to conceal the structural loss.
- Assert declaration names AND values for named_function_value,
  named_function_application and constructor_structural_uses_spaced. Test
  that generic parameter/result substitutions remain linked after matching.

### Reverse slices select in reverse-iterator coordinates

- Original3839145 and Rust both accept the complete asymmetric slice fixture
  without diagnostics, but a=[0,1,2,3,4]; a~[0:2] gives original[4,3] versus
  Rust[1,0]. The old len4[1:3] unit is symmetric and cannot expose this error.
  Keep it and add asymmetric bounds, from-end flags, open and empty intervals.
- axis.w::row_slice uses r.rbegin()+lower through r.rbegin()+upper. Translate
  to forward storage using n-upper..n-lower BEFORE reversing. Retain existing
  range checks and empty-interval handling before narrowing indices; do not
  weaken a regression to the old symmetric case.
- Source inspection also finds five slice receiver kinds upstream, not rows
  alone; bounds are analysed before receiver-kind rejection. String slices
  operate on bytes, not Unicode characters. Capture those separate contracts;
  the reverse-row repair does not imply string/vector/matrix slice acceptance.

- Byte discovery3839236 confirms isolatedc3/a9 bytes and reverseda9c3 in
  original stdout, including nested row and ordinary value formatting. Rust
  string subscription already indexes bytes, but from_utf8_lossy replaces
  the selected invalid byte with a three-byte replacement character. This
  changes values/length, not just display. Keep raw byte goldens and the
  isolated string_byte_subscription fixture; an ASCII-only slice patch or
  output-only replacement is not a faithful repair. See the indexed
  docs/slices/string_bytes_2026-09-29.md for the full value/output boundary.

### Function returns use one live result context, not their local context

- Original3839034 and Rust both accept return_loop_values without diagnostics,
  but Rust prints void instead of false/index1/integer23. Preserve the complete
  positive and the negative that wrongly installs two functions; one later
  type error must not hide earlier wrong acceptances in the same fixture.
- axis.w:743-774 shares a live result type across the body and return operands.
  A discarded branch/loop has void context, but its return operand does not.
  Return values do not participate in branch balancing; the first explicit
  result constraint is observable. Casts specialise the shared context BEFORE
  analysing the body (7290-7332), and next establishes its first expression's
  result before analysing the discarded second expression. Nested lambdas own
  separate return contexts. Do not repair this with root-cast extraction or a
  final join of return types. Use safe shared ownership without holding a
  mutable RefCell borrow across recursive conversion.

### Operator binding patterns are typed symbols, not ordinary identifiers

- Build3838965 compiles and passes both nonfunction-binding regressions, but
  both positive binding tests still fail at the later `@`: ordinary operator
  casts were not implemented. Keep those exact failing units; fixing the first
  rejected token is not whole-expression support. Original3838987 now captures
  exact-first selection, generic fallback, outer fixed variables, GLOBAL3 under
  local shadowing, no-coercion/ambiguity/no-instance errors and lexical formals.
- Operator casts query GLOBAL overloads even under local shadowing, unlike
  operator application. `f@ T (T)` introduces free selection variables, not a
  rigid TypeAbstraction node. Keep parameter/result substitutions linked over
  the whole signature and restore every failed generic trial (axis.w:7364+).

- Original3838739 accepts bare `set ! = function@type`, needed by basic.at:91,
  and rejects a nonfunction before evaluation. Its global rejection has no
  Type/Program heading; pin the full SET envelope and keep negative classifier
  controls. Matching values do not imply identical diagnostic presentation.
- Original3838953 verifies local/tuple/parameter/loop symbol bindings and
  global tuple atomicity. Preserve the parser's independent operator flag
  (0x8), reject nonfunctions in every scope, and make lexical operator functions
  shadow global overloads just like named lexical functions (axis.w:2741+).
- Preserve failed discovery3838933: lexer.w:802+ makes tilde before comma,
  colon, right bracket or do/od/if/for a reversal token. Right parenthesis is
  NOT in that list. Bang companions isolate symbol patterns without changing
  the rejected tilde-comma inputs. basic.at loading is not math acceptance.

### A zero/one-use result producer should accept FnOnce

- While mode3838593 fails compilation with E0507/E0596: its owned count/row
  cannot be moved/reversed inside at_level's unnecessarily strict Fn closure.
  The helper invokes the producer zero times at NoValue and once otherwise;
  use FnOnce as at_builtin_level already does, with an owned-value/NoValue
  regression. Retain the failed build and rerun all core/CLI/integrity gates.

### A while guard and its body share the enclosing lexical frame

- Discovery3838473 accepts while controls under let, tagged case, if and
  integer case; old Rust rejects them. Keep CASE[0,1,2], LET[3,4], IF[0,1],
  INDEXED[10,20] and STATE522. Original rejects the missing-else form at FI
  and still prints recovery37. Equal syntax categories alone do not prove
  the right rejection point or full diagnostic equality.
- Original axis.w:5887-6110 keeps do condition/body below the same scoped
  node. Do not extract them into separate evaluations of a discriminator or
  let initializer. Its dont is false do die: it imposes no body type, rather
  than forcing void and conflicting with a value-producing sibling branch.
- R3 abstraction3838431 passes441core/CLI/integrity and gains13full-stream
  matches with no losses, but basic.at still fails at this while/DONT gap.
  Unit counts are prerequisites, not high-level mathematical acceptance.
- Context discovery3838562 verifies COUNT3, VOID2, REVERSE[2,1,0], GUARD[0,1],
  EFFECTS32 and nested-empty-loop isolation. The required context selects
  counting/void/row evaluation; int is NOT a row-to-int coercion. Convert
  count/void bodies in void context before balancing branches, and never
  accumulate discarded row values. Still analyse dead bodies; the same
  discovery retains undefined-name, escaping-local and mixed-branch errors.
- Break discovery3838627 accepts the generic definitions/value checks but
  rejects numeric break1 later in the SAME provisional positive fixture.
  Current parser.y:461-462 repeats BREAK tokens (break break for two levels).
  Keep the rejected discovery; capture separate accepted and isolated numeric
  rejection companions before migrating historical numeric-break tests. Partial
  successful output is not a whole-fixture pass. Break also leaves the required
  type untouched upstream; forcing void corrupts basic.at take's row type.
- Follow-up3838661 accepts the entire repeated-break generic fixture and
  proves numeric break1 is wrongly accepted by Rust. Historical3838675 then
  captures the exact two/three-level old unit sources and current companions:
  keep the old values, migrate spelling/messages, and retain numeric negatives
  plus historical reference files. Raw parsetree error text STILL prints the
  depth numerically; only source syntax/depth diagnostics repeat BREAK.

### Raised inference contexts must reach the scoped lambda solver

- Abstraction integration3838237 compiles but its original-backed identity
  test fails: the old bare-Type lambda specialisation cannot assign a free
  context variable. Follow axis.w:3508's unify_specialise and body export into
  the SAME context scheme; keep argument/result links and rigid parameters.
  R2 repair3838315 passes that assertion but full core finds438pass/1fail in
  the old escaping-recursive-closure unit: rec_lambda needs the same migration.
  Retain BOTH unchanged failures, concrete-context negatives and full-core gate.
- Capture3838269's explicit3case discovery subset failed the master checker's
  exact100case guard before running interpreters. Preserve it; a fresh full103
  case stage updates the exact count and keeps every prior case. Do not weaken
  inventory guards or mistake a failed harness for language evidence.
- Original3838354's SET runtime die has an interruption envelope but no
  Runtime/Program heading. Preserve its before-print and adjacent bindings.
  Classifier/checker3838409 verifies the exact anchored envelope and negative
  controls; signals still remain invalid. A matching rejection category does
  not mean matching full diagnostics or execution order.

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

- Direct prototype3841440 verifies32complete compact/lattice candidate sets
  through D6/E6,40prior exact orbit partitions and23unchanged controls.
  D8/E7/E8 closure gives17040/10208/199952candidates, not original-backed
  end-to-end acceptance. Original weyl.cpp1312's reversible descent supplies
  the completeness argument; matching counts/closure alone does not.
- New generator APIs under3841461 use an EXPLICIT twisted-involution budget
  and materialize lattice data only for class representatives. They are not
  wired into CartanClassification/interpreter yet. Preserve legacy A2 Weyl
  budget5 rejection, central-lattice actions and every exact partition member.
  A future budget mode must enter cache keys; weyl_budget() also feeds the
  real-Weyl longest-action consumer. See indexed
  slices/direct_twisted_generation_2026-09-29.md before integration claims.
- Kernel3841461 passes32exact full partitions and large D8/E7/E8 partitions,
  with final source guards. This verifies the unused APIs, not high-level
  loading or mathematics; original209case inventory3841464 and subsequent
  caller/cache/consumer gates remain mandatory before integration claims.
- Release builds write build.json, unlike foundation/probe report.json.
  Reference3841457 stops before either interpreter on the wrong path. Keep
  the failure; resolve and hash the real file before submitting a fresh
  same-input stage. Never treat an empty capture as a mathematical result.
- Release A2_klv3840825 now reaches groups.at251 E8_ic and fails the4000000
  Weyl cap while original succeeds. Debug R2stack3840859 separately locates
  eager E6_s KGB at groups.at237. Do not conflate the sample with the terminal
  barrier. Orbit-generator closure alone removes only the SECOND full-Weyl
  sweep; direct twisted-involution generation and lazy real-form ownership
  have distinct correctness/budget/identity gates. Never omit E8 from the
  upstream library or increase the cap as a claimed mathematical repair.
  See indexed slices/library_context_loading_2026-09-29.md.
- Exact debug groups.at stack3840788 samples eager KGB construction through
  add_cartan/negative_coweight_eigenspace/saturated_kernel; external form1,
  Rayon worker idle. It does NOT implicate the orbit sweep in that sample or
  identify E7/E8. Keep generator optimization separate from lazy real-form
  ownership and use actual phase/rank evidence before attributing a timeout.
- Source audit2026-09-29 identifies TWO whole-Weyl traversals: candidate
  enumeration and the conjugacy sweep. Original weyl.cpp1256 uses generator
  orbit search for the latter. Replacing only that sweep does not bypass
  D8's enumeration cap. CompactWeyl returns HashSet order, so raw partition
  IDs are not stable external Cartan numbers; compare complete canonical/
  external outputs, never sort away observable differences. See indexed
  slices/twisted_orbit_search_2026-09-29.md before any optimization/A-B claim.
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
