# HPC verification

For the current mathematical-validation goal, all builds and testing run
through SLURM on compute nodes, including small unit and verifier tests.
Local work is reading, editing, Git and hashing. Use the XMU login node for
Git/source staging and dependency acquisition, not builds or interpreter runs.
This supersedes the historical local-small-check/login-build instructions.

## Repository location and toolchain

The shared project directory `/public/home/majj/atlas-rust` on XMU contains
unrelated uncommitted work; never pull into, overwrite or build from it for
the current validation campaign. Use immutable children of the single campaign
tree, not fresh top-level directories. The login node
has GitHub access (HTTPS checked on2026-09-28) and can acquire dependencies; compute
nodes do not. The repository follows the installed stable toolchain, with
Rust 1.90 as the enforced minimum because Malachite 0.10 requires it. Install
or update a suitable stable toolchain on the login node before building:

```bash
rustup toolchain install stable --profile minimal --component clippy,rustfmt
```

Acquire dependencies on the login node; build binaries on compute nodes.
Every job records the commit, dirty-tree state, Rust toolchain, reference
Atlas revision, CWEB version, SLURM job/node, fixture manifest, exit status,
and report checksums.

Never put tokens, credentials, or large generated outputs in Git.

## Campaign layout and retention

New validation work uses one dated top-level campaign rather than one
top-level directory per probe or retry:

```text
/public/home/majj/atlas-rust-campaign-YYYYMMDD/
  .atlas-progressive-submit.json
  stages/<descriptive-stage>/
    hpc/ tests/ pin.json submission.json
    results/<job-id>/
```

The date is the campaign's creation date, not a daily rollover. Keep using the
same active campaign across midnight, retries and successive stages of the
same investigation.
The current active root is exactly
`/public/home/majj/atlas-rust-campaign-20260930`; the validator rejects every
other date so a second root cannot acquire an independent lock/ledger. A later
campaign rollover requires an explicit reviewed code/policy change after the
current campaign is sealed, not merely a new calendar date.
A reconnect, retry or resubmission with the same pin and manifest must reuse
the same `stages/<logical-stage>` path. An uncertain network or scheduler
reply never creates a `-retry`, `-r2` or fresh stage. A new logical stage is
permitted only when input bytes change and its predecessor, changed digest,
reason, retention class and retirement condition are recorded before creation.

`hpc/campaign_workspace.py` enforces this shape. All campaign stages share one
queue ledger. A job expands its pinned source and writes Cargo/profile targets
inside an exact temporary workspace (prefer `SLURM_TMPDIR`, otherwise a hidden
directory below that job's result folder); normal or exceptional context exit
removes that workspace. Durable results contain the report and checksum,
command logs/timing, raw differential streams, source manifest, exact patch and
fixtures. Preserve only a checksummed binary/source archive when a later gate
actually consumes it. Never retain a Cargo target as evidence.
`hpc/progressive_submit.py` rejects a non-campaign stage before it contacts
SLURM. Existing top-level stagers are historical/recovery code only and must
not be used for a new submission. The active launcher reads locks, ledger,
pin, intent, receipt and scripts without following symlinks and verifies a
stable regular-file identity before accepting state.

For the one-time parent seal, the raw pin SHA is durable in both campaign
ledger and stage intent before `sbatch`. A compute job that was accepted while
the login stager crashed may complete only its own exact partial submission
state under the existing stage-then-campaign locks; it never resubmits or
creates another stage. The end-of-job preflight is validation-only. The frozen
lifecycle-v6 transport contains14 files; manifest SHA-256 is
`0aac6e735c0ae7c7e0da4773c46e0d8f116b92b68a73797b05e886ea342a2620`
and deterministic tar SHA-256 is
`8347c733555b1e54c9798a6006636beb6be22e2c03483c10172625f6875d9b6a`.
These are local transport hashes, not HPC acceptance. The lock model assumes
cooperating same-UID processes do not rename whole stage/campaign directories;
hostile directory replacement remains a documented P2 until all state I/O is
dirfd-relative.

The shared object store is `objects/sha256/<prefix>/<sha>`.
`hpc/campaign_source.py` stores deterministic source archives and
`hpc/campaign_blob.py` stores generic evidence and seals; refs contain only
schema, role, SHA-256 and byte count. Parent-seal job3872554 is FINAL and was
independently accepted in its infrastructure-only scope; its launcher is now
closed. The path-free seal is the sole parent dependency of
the ladder-boundary BEFORE gate. V2 job3872594 failed its Python checker
preflight at63/65 before Cargo or mathematical work and remains immutable.
BEFORE v3 job3873400 then completed on cu006 and its FINAL report was independently
inspected: all65 checkers pass, inventories are521/630, and unchanged
production has exactly2domain+1core named failures with0 ignored. Report SHA
is `51cc7a14a0dbb8188a4ea47705338e5689212bf1f707819bab8c4e5681e4052b`.
This is accepted tests-first BEFORE evidence only; do not resubmit BEFORE v3 or treat
it as a repaired mathematical result.

Historical successor `ladder-boundary-after-v1` kept all work under this
campaign root and ran88 unittest checks, but job3873497 is now FINAL
`FAILED|1:0`: all88 checker outcomes and both toolchain commands passed, then
the first `full-stager-inventory` command failed because the accepted source
object contained zero historical stagers. No Cargo, patch or mathematical
command ran. Report SHA-256 is
`fec90eb2f4cf0a4dff4820c4527c5263c151f8dff4aabf272007618646149e15`;
this is immutable `HARNESS_FAILURE`, not a mathematical result.

The changed-input successor was `ladder-boundary-after-v2`, under the same
campaign root. It replaced the
false source-tree assumption with one path-free CAS bundle of the exact67
retired stager sources, validated them in memory, and overlaid only the then-current
parent/BEFORE/AFTER launchers. At its pre-creation freeze, the local worktree
guard had22 synthetic tests and the v2 unittest total was93. Those exact bytes
later ran in job3874203 as described below.
Its pre-creation28-file override manifest is
`tests/reference/hpc/math_ladder_boundary_after_v2_overrides_2026_10_01.json`
(SHA-256 `b3e4fa16cdb9e86aba36d4bdd5723401a32b0c07ef21cdce29165dd938daa699`).
The exact67-source canonical bundle has been stored once in the existing
campaign CAS as role `retired-stager-bundle`, 311925 bytes, SHA-256
`550b1330ec8806d1343d50d6ceb8488f89eb03546f6bb37c538cb5cd460e9b09`;
its local and remote upload scratch were removed. After an empty-queue,
accounting, ledger and exact-path reconciliation, the child stage was created
and shared submission accepted exactly job3874203 with `queue_before=[]` and
pin SHA-256
`3f0f3c8b79e7ab7688ec329e242d18a82ac1b113eb33fa3788cfe1ca3e31455d`.
Post-submit inspection found all28 inputs0444/single-link, empty `.incoming`
and no change to the318 top-level `atlas-rust*` directory count. Job3874203 is
now FINAL `FAILED|1:0` after4m55s on cu105. All93 checkers, full-stager70/67,
domain inventory521, both focused tests and all521 domain tests pass, but the
driver records `HARNESS_FAILURE`: full-suite `--nocapture` exposed a panic
intentionally caught by a passing cache-poisoning test, and the log parser
rejected the `panicked at` text. Core and final-integrity gates were NOT_REACHED.
Report SHA is
`256247b2be5fd7358ccb56e87ff262c6c0e87ca782e822e2f3c96419c4b4a694`;
inspection evidence is
`tests/reference/hpc/math_ladder_boundary_after_v2_failure_2026_10_01.json`.
Do not modify, duplicate or resubmit v2, and do not treat it as mathematical
failure or acceptance.
The bounded AFTER v3 successor keeps strict panic-text rejection, removes
`--nocapture` from full suites and unsets `RUST_TEST_NOCAPTURE`; focused runs
retain nocapture. Local candidate bytes and a candidate29-input override
manifest existed at the pre-creation checkpoint, with no immutable pin, HPC child stage or submission. It
must bind the immutable v2 failure evidence and rerun domain and core gates
from scratch in a recorded changed-input child.
Its reviewed29-input manifest is
`tests/reference/hpc/math_ladder_boundary_after_v3_overrides_2026_10_01.json`
(SHA-256 `3fe6c8c5c0a0638ae8d128313ad3bfce02bc97e3d16a69f75670a864b28695c2`).
The exact target is the one child
`atlas-rust-campaign-20260930/stages/ladder-boundary-after-v3`; its direct
predecessor is v2 job3874203/evidence `e757f56c...be1f8f1`. Parent seal,
retired-stager CAS and math patches are unchanged. Retention is
`ACTIVE_GATE_COMPACT` with FINAL inspection, acceptance-index disposition,
zero live/uncertain ownership, CAS closure and separate deletion authorization
required for retirement. At the pre-creation reconciliation the queue was
empty, no AFTER v3 accounting row or target existed, and the five-record shared
ledger still hashed to `44624c76...1835d`; no pin/stage/submission existed.

Final static review found all29 manifest paths/hashes and95 checker methods
aligned. After another empty-queue/accounting/path/ledger reconciliation, only
the exact child above was created and the shared stager submitted job3875239
with `queue_before=[]`. Pin, intent, receipt and six-record ledger SHAs are
`7af79aa20b37d0fcee9a60d8e3c6b4dbec05417cf6817747520b6ebe3ec41f88`,
`feff9e66a0317719258db7e210325a109e263d1447790b71d4a444c33f79a0ab`,
`ac1f6f52f99de9b86b7b7bbfe56e7478809f193f1938a0c07219bc6061ee2254`
and `a1776412e7dd6633c329743b52c5cf4b1615dde8a7a2f0b4fddd002b5615a618`.
The job was RUNNING on cu105 when observed. All installed inputs are0444 and
single-link, and the top-level `atlas-rust*` directory count remains318.
This is `SUBMITTED_NOT_VERIFIED`, not repair or mathematical acceptance.

Job3875239 later completed `0:0` on cu105 in10m18s. Independent inspection
accepts this exact bounded AFTER gate: all95 checkers, full-stager70/67,
domain521 inventory/2 focused/521 full and core630 inventory/1 focused/630
full pass; all16 commands exit zero and all32 log/time artifacts rehash. Report
SHA is `771fc790dd4340f408235e50c3f6eee754ebe4c850cbad36d4af4f902a24627c`;
inspection evidence
`tests/reference/hpc/math_ladder_boundary_after_v3_2026_10_01.json` SHA is
`a459fa08117ff8a721181d349467ebdd9267e798cd25b5bcb1380996c2d15e15`.
The ephemeral build workspace is absent, and the durable child is only3220KiB
allocated. The top-level `atlas-rust*` count remains318. This is bounded
root-ladder/directory-governance acceptance, not a broader mathematical, rank,
performance or cleanup authorization.

The append-only acceptance-index publication then completed as job3879103
`COMPLETED|0:0` on cu075 in17s. Its independent evidence is
`tests/reference/hpc/math_ladder_boundary_index_v1_2026_10_01.json` (SHA
`d9448e5cd53ba4c25e6a0fd91a5fdb16e3b423d48fe58531a50d7d7457afb9ad`),
and the full report SHA is
`43cc79e1630b5ec21a24c5aa1a7c411eafffaa9a4c6911333c8752b0dc63cf9d`.
The stage/index/allowlist suites pass20/34/7; all24 installed inputs and exact
directory topology match; source/CAS integrity passes; no Cargo or Atlas
command ran; the ephemeral workspace is absent; and the top-level
`atlas-rust*` count remains318. The only new path is the registered child
`atlas-rust-campaign-20260930/stages/ladder-boundary-index-v1`. Both ladder
BEFORE/AFTER launchers are disabled; the still-enabled index launcher is
receipt-bound and may only recover or validate this exact stage/job, never
create a duplicate or sibling. This is publication/durability acceptance only.

Historical AFTER v1: after an empty-queue/accounting/ledger/exact-path
reconciliation, that stage was created under the existing campaign and shared
submission accepted job3873497 with `queue_before=[]`. Pin SHA-256 is
`200055d52c0d9559a2edf4b7e556296d0c8bd0df129030e4e855e4e4426df289`.
Post-submit inspection found27 staged0444/single-link inputs, an empty
`.incoming` directory and no change to the318 top-level `atlas-rust*`
directory count. The later FINAL failure above is preserved in place; do not
resubmit or mutate v1, and do not treat its checker success as mathematical
verification.

No currently inventoried local worktree or historical HPC stage is
deletion-authorized. Eligibility and authorization are separate decisions.
Many reports name
parent reports, source trees, binaries and raw streams by absolute path. Before
cleanup, build a reference inventory, reconcile live/uncertain jobs, and verify
that every retained report can resolve its complete dependency closure from
durable checksummed artifacts. Local worktrees also require dirty-state and
unique-commit review. The audit and migration rules are recorded in
`docs/slices/hpc_artifact_retention_2026-09-30.md`.

For local parallel work, the primary checkout is the default shared workspace;
agents coordinate non-overlapping files instead of allocating one worktree per
probe. New worktree creation is unconditionally frozen under the current
registry schema: every ACTIVE row is rejected regardless of the LEGACY count,
and the old `precreate` path fails before reading registry, Git or filesystem
state. Reopening creation requires first implementing and HPC-verifying a
durable receipt-bound creator, then obtaining explicit user
authorization for at most one task-scoped worktree when checkout isolation is
demonstrably necessary. Run
`python3 hpc/local_worktree_guard.py check` before delegation/editing and
handoff; any inventory mismatch is fail-closed and must never trigger automatic
prune, repair or removal. Normal checks sample registry, Git inventory and the
fixed `/home/hoxide/mycodes/atlas*` sibling namespace twice. Registered entries
must be real no-follow directories with the expected primary/linked `.git`
type, and the two Git/filesystem identity snapshots must agree. A retry reuses
an already authorized path. Passing
the tracked/untracked/ignored-file, commit-reachability and active-use checks
only makes the exact directory retirement-eligible; removal requires explicit
user authorization for that path. Branch deletion is a separate decision.
The guard is a cooperative authorization/detection boundary, not a Git syscall
interceptor: raw `git worktree add` is forbidden, and any still-present
unregistered result in the managed namespace stops subsequent work at the next
mandatory inventory check rather than being silently adopted or automatically
removed. It detects persistent additional clone/copy/mkdir names only inside
that fixed Atlas sibling namespace; paths elsewhere and create-then-remove
activity between samples remain invisible. It also does not persist inode
identities across separate invocations or enforce an exclusive owner/mode
policy, so a stable same-UID same-name rebuild completed before checking and
cross-user security are outside its guarantee. Until a receipt-bound retirement transaction is
implemented and HPC-verified, the exact24 LEGACY rows and directories cannot
shrink even after eligibility is established; a passing guard alone is not
deletion evidence.
The audited 24 LEGACY path/HEAD/branch identities are frozen in the guard.
The current schema cannot remove, add or replace those identities. A future
authorized retirement must use a new receipt-bound transition rather than
editing the current registry and directory together. This remains cooperative
detection, not an OS-level interceptor.
Likewise the campaign checks govern reviewed project launchers, not arbitrary
shell access: raw `mkdir`, direct `sbatch` and executable historical script
copies remain workflow violations that the helper cannot intercept at the OS
boundary.

For published source, prefer HPC-side Git. Resolve the exact reviewed full
commit SHA on the login node, fetch dependencies there, then have the campaign
stager create one content-addressed source object. No persistent source cache
is currently registered for this campaign, so do not create one. Use an
auto-cleaned incoming checkout below the existing campaign root and remove it
immediately after its source object and manifest are verified. Do not use
`mktemp` to create another top-level `atlas-rust-job.*` tree.

Verify HEAD equals the requested SHA and status is empty before submission.
A future persistent singleton cache requires a reviewed lifecycle transition
and its one exact path in the handoff registry. Even then, materialize the
exact commit only in the stage job's auto-cleaned workspace and retain the
verified CAS source object and manifest. Never use a moving branch as the
recorded version.
Unpushed candidates use a small checksummed patch against a pinned HPC
baseline, with an exact source-file manifest after application. Do not push
unverified code just to transport it, upload target trees, or overlay the
shared development checkout. Current math harnesses freeze archives plus
input manifests; see `docs/HANDOFF.md` for their active pins and jobs.

## Historical command appendix — non-executable

Do not copy or run any command in this appendix. These direct `sbatch`
examples bypass the campaign-wide ledger and may retain expanded source or
Cargo targets. They remain only to explain old reports. The accepted
`ladder-boundary-before-v3` must not be resubmitted. Its separately reviewed,
changed-input AFTER stage completed as accepted job3875239 under the same
guarded campaign and accepted parent seal; no duplicate submission is allowed,
and any successor requires a new reviewed changed-input lifecycle record. The
historical jobs ran from a fresh login-node checkout,
not the shared directory. A lexer preflight alone was not mathematical
verification:

```bash
ATLAS_COMMIT="$atlas_commit" ATLAS_DIRTY_TREE=false sbatch hpc/differential.sbatch
```

For the structural Rust layer, use the smaller preflight job first:

```bash
ATLAS_COMMIT="$atlas_commit" ATLAS_DIRTY_TREE=false sbatch hpc/real_group_preflight.sbatch
```

For the typed scalar operator stage, first capture the upstream oracle only.
The job writes raw output for each scalar fixture, per-stream checksums, exit
statuses, the Atlas revision, and a compact validation report. It deliberately
does not invoke Rust, so it is valid evidence for freezing the reference before
the implementation stage:

```bash
ATLAS_COMMIT="$atlas_commit" ATLAS_DIRTY_TREE=false sbatch hpc/scalar_reference.sbatch
```

The report is `results/<commit>/<job-id>/scalar_reference_report.json`; its
SHA-256 sidecar covers the manifest, while the manifest covers every captured
stdout/stderr artifact.

The scalar event expectations assert values and diagnostic text only. The
capture records the oracle exit status but does not infer an exit-code policy
from a diagnostic fixture unless that policy is explicitly added to the event
schema.

For the typed pipeline swap, compare the Rust CLI against the already frozen
Atlas event files with:

```bash
ATLAS_COMMIT="$atlas_commit" ATLAS_DIRTY_TREE=false sbatch hpc/pipeline_swap_diff.sbatch
```

The report is
`results/<commit>/<job-id>/pipeline_swap/pipeline_swap_diff_report.json`.
The constructors and linear-value fixtures run in full, including
`root_datum(LieType,mat,bool)`, `Cartan_matrix(RootDatum)`, and
`involution(KGBElt)`. The domain-equality fixture currently runs only its
RootDatum prefix: InnerClass/RealForm/KGB setup, full domain renderings, and
relation outputs are explicit pending cases until those renderers and stable
numbering are ported. The report also lists the three selected-stage upstream
overloads still outside the Rust type surface as `uncovered_overload` pending
cases: the two primitive `involution` constructors and synthetic
`real_form(InnerClass,mat,ratvec)`. This is a selected typed-pipeline scope,
not a claim that no later Atlas overloads exist. The suite remains `PARTIAL`
until these pending surfaces are ported. A mismatch in any runnable fixture
still fails the job.
The job records both the declared `ATLAS_COMMIT`/`ATLAS_DIRTY_TREE` and the
values detected from the submit checkout, and refuses a mismatch before and
immediately after creating the frozen source snapshot. A clean versioned
checkout is frozen with `git archive <detected-commit>`; a dirty or unversioned
checkout retains the live-tree snapshot but labels that state explicitly in
the report. Before it uses the dirty-tree helper, the job loads the clean-tree
helper from the detected Git object or freezes and hashes a dirty helper copy.
It also requires the Slurm spool script to match the copy in that snapshot
and, for a clean versioned checkout, the script blob in the detected commit. After
the safe numeric/token preflight and report-directory creation, an EXIT handler writes a
machine-readable FAIL fallback (and checksum) if build, harness, or snapshot
verification aborts before a normal report exists. Malformed source tokens or
an invalid Slurm job id are rejected before a safe report path can be
constructed.

To capture an upstream fixture before editing its checked-in expectation, use
the raw reference job. With no fixture argument it captures
`commands/subscription_context.atlas`:

```bash
ATLAS_COMMIT="$atlas_commit" ATLAS_DIRTY_TREE=false sbatch hpc/reference_capture.sbatch
```

Pass one or more paths below `tests/fixtures/` after the job script to capture
other fixtures. The job pins and verifies the Atlas Git revision, records the
binary checksum against the frozen pipeline-oracle binary by default, and
stores raw stdout/stderr plus a JSON manifest under
`results/<commit>/<job-id>/reference_capture/`. It neither consumes nor
modifies event expectations, and its `PASS` status means only that the oracle
capture itself is valid; it is not a Rust compatibility claim. Set
`EXPECTED_ATLAS_BINARY_SHA256` explicitly only when intentionally using a
separately audited rebuild of the pinned reference revision.

Fixture arguments must be repository-relative paths including the
`tests/fixtures/` prefix and `.atlas` extension, for example:

```bash
sbatch hpc/reference_capture.sbatch \
  tests/fixtures/domain/print_block_words.atlas \
  tests/fixtures/domain/print_block_words_rejected.atlas
```

The manifest also compares declared and detected submit-repository commit and
dirty state; either mismatch makes the capture FAIL. The batch job freezes a
clean versioned checkout from `git archive <detected-commit>` and labels a
dirty or unversioned live-tree snapshot explicitly; it binds the source-state
helper before use, rechecks state after freezing, and requires the Slurm spool
script to match both the frozen copy and, for a clean versioned checkout, the
committed script blob. Raw files retain their fixture-relative directory paths
to avoid artifact-name collisions. The capture rechecks the upstream revision, dirty
state, executable checksum, and `atlas-scripts` tree hash after the final
fixture; a runtime replacement during capture therefore cannot produce a PASS
report.

All differential jobs must use `sbatch`; do not run them on the login node.
Job scripts must fail on a mismatch and write a machine-readable report under
`results/<commit>/<job-id>/`. Pull only summaries and checksums back:

Copy only the exact job's reports/checksums (and targeted raw diagnostics when
needed) from its immutable stage. Do not synchronize an entire shared results
tree, and verify downloaded hashes against the remote report.

SLURM opens `#SBATCH --output` before the script body runs. The checked-in jobs
therefore use root-level output filenames and exclude only the current job's
exact untracked stdout path from the submit-tree dirty check. Other Slurm logs,
untracked files, and tracked changes still make the checkout dirty. Exercise
that rule with `bash hpc/test_source_state.sh` inside an HPC compute job.

Compute-node jobs should also set `PATH="$HOME/.cargo/bin:$PATH"` explicitly;
the login-node shell environment is not guaranteed to be inherited.

Both checked-in jobs follow the `rustcox` cluster convention: the submit
directory is made explicit, the Rustup toolchain is set explicitly, and every
run writes a JSON report plus a SHA-256 sidecar under
`results/<commit>/<job-id>/`. `differential.sbatch` is still a lexer-stage
preflight, not differential evidence. `real_group_preflight.sbatch` runs the
Rust structural format check, Clippy with warnings denied, and the unit suite,
also without an Atlas-compatibility claim. A real-group differential job
requires an exposed Atlas constructor fixture, a reference event adapter, and a
Rust domain event adapter; until then
`tests/reference/domain/real_group.meta.json` records only an HPC preflight,
not domain compatibility.

`real_group_preflight.sbatch` freezes the complete submit directory before
Cargo runs, excluding only `.git`, build targets, prior `results`, and the
Slurm stdout file. It hashes that frozen tree, executes from it with targets
and reports outside it, then rejects the job if the hash changes. Each domain
fixture entry in its report is explicitly marked as a declared future
differential fixture rather than an executed one.

This rule was verified by HPC job `3462432`: reporting a hash from the mutable
submit directory in an exit trap could otherwise describe different inputs than
the Cargo commands consumed. Always derive `ATLAS_DIRTY_TREE` from
`git status --porcelain --untracked-files=all`, because an untracked source
module is part of a synchronized submit tree too.

## Verified repair notes

### Owned-involution builder compilation

- Root cause: `CartanFiber::build_owned` took ownership of its involution, but
  a read-only numerator helper was called without borrowing it.
- Diagnostic: frozen HPC job `3463647` reached Clippy compilation and reported
  the exact `expected &LatticeInvolution, found LatticeInvolution` error.
- Prevention: after changing a builder from borrowed to owned input, audit
  every pre-move helper call and rerun the package preflight; job `3463683`
  verified the repair with format, Clippy, and 77 tests.
