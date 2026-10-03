# Atlas-Rust handoff - 2026-08-01 (handoff to next coding agent)

## CURRENT: AFTER-v1 gate frozen locally, commit 342a0511 pushed; HPC submission BLOCKED on SecureLink tunnel — 2026-10-03

The changed-input `weyl-context-core-after-v1` gate is fully finalized,
locally verified and committed as `342a0511` on
`origin/codex/math-benchmark-suite` (12 files, +15190/-53).  It is NOT
submitted: the SecureLink tunnel (`tun0`) is down — the daemon and GUI
client processes are alive since Sep 18 but no tunnel interface exists,
`tailscale0` is logged out, and `majj@10.26.14.64:22` times out over the
ordinary Wi-Fi gateway.  The operator must reconnect SecureLink; the exact
launch procedure below is then immediately runnable.  No remote stage,
intent, ledger record or job was created; nothing is in an uncertain state.

Same-day documentation maintenance (all pushed): `d7bcb04d` advances the
Weyl KB source packet to this freeze state; `bd3b2352` completes the
initial `kb/` tree import (the vault was untracked transition-snapshot
content; `node_modules` and `.llmwiki` runtime stay ignored);
`5126d83d` refreshes the root-ladder packet and both curated pages to the
accepted AFTER-v3 state, citing acceptance-index entry
`0003-a1-torus-root-coroot-ladder-boundary` with its limitations.  The
compiler-generated pages remain `needs_refresh`; the provider
authorization blocker stands, so no `./kb/llmwiki compile` run happened.

Final frozen hashes: after stager `stage_weyl_context_core_after.py`
SHA-256 `8cb4b86d9986e66c1722c87d5704ab92ee387ce426ff0f3c194b60c5320904d6`,
after driver `math_weyl_context_core_after.py` SHA-256
`3d7d98903a838b3a3343482d165af4b7e9b3989cc1f30a5c9ccc49950700913b`,
after checker `test_math_weyl_context_core_after.py` (28 tests),
allowlist `test_stager_allowlist.py` (7 tests) with
`CURRENT_POLICY = "weyl-core-after-active"`.  Checker table:
32+17+18+21+28+7 = 123 (regression-contract gained the four
`classify_after` tests: 17 -> 21).  The capture/before pair is disabled
(`SUBMISSION_ENABLED = False` in both files); only the after pair is
enabled.  The repaired-manifest chain verifies offline: the frozen
before-v4 report's 1567-file source manifest has canonical SHA-256
`55f807cadb712377cbf6c0c79250b1d5e739e9757290a24209a086f89632850f`, and
applying `REPAIRED_SOURCE_HASHES` (domain_builtins `e6987e7c`, typed
`614975c5`) yields exactly `AFTER_SOURCE_MANIFEST_SHA256`
`3f8cf4753f29d33df8273086254a4f09170ada46f05e9c13acfdac2f5ab85c33`.

Latent transition bugs found and fixed this session by running the
synthetic suites locally under `umask 022` and by replaying every
evidence validator against the real frozen evidence in a 0444 mirror
tree (local file modes mask these in the default checkout):

1. `progressive_submit._validate_predecessor_state_descriptor` omitted the
   before-v3 scoped campaign creation files from `required_campaign_files`
   once the predecessor advanced to before-v4; the real 37-file campaign
   state would have been rejected at stage creation.  Added
   `before_v3_scoped_campaign_files`.
2. `test_campaign_stage_creation.predecessor_fixture` lacked the three
   before-v4 campaign-scoped creation files and its ledger re-read
   assertion kept the old length (`len(history) == 19` -> `== 20`).
3. `validate_before_v3_failure` still referenced bare
   `PREDECESSOR`/`PREDECESSOR_STATE`/`PREDECESSOR_STAGE` (correct in the
   before-v4 launcher where PREDECESSOR was before-v3, always-false in the
   after module where PREDECESSOR is before-v4).  The frozen before-v3
   identity now lives in `BEFORE_V3_PREDECESSOR*` constants transplanted
   verbatim from the capture stager, and the validator rebinds the three
   names locally so its body stays identical to the HPC-verified original;
   its `successor_stage` check is the historical literal
   `"weyl-context-core-before-v4"`, not the after stage name.
4. The before-v1/v2/v3 failure evidences froze `expected_test_counts_after`
   with the capture label and total 119; the before-v4 report froze
   checker total 119.  Validators now pin
   `BEFORE_FAILURE_PREDICTED_AFTER_COUNTS`/`BEFORE_V4_CHECKER_TOTAL`
   literals instead of reusing the current `EXPECTED_TEST_COUNTS`/
   `CHECKER_TESTS` (which legitimately moved to the `-after` label and
   123).
5. `test_running_job_recovers_exact_twentieth_attempt_and_missing_receipt`
   never patched `driver.validate_before_v4_result` or
   `driver.repaired_source_manifest`; both are now patched and asserted.

Local suite state under `umask 022` (reading/editing/hashing only; no
builds): campaign-stage-creation 32/32, progressive-submit 17/17,
contract 18/18, regression-contract 21/21, after checker 27/28,
allowlist 7/7.  The single after-checker error is the known
environmental class: it reads the repository's evidence files at their
checkout mode 0664 while the validator requires the stage-installed 0444;
the HPC-verified capture checker has the same local-only behaviour.  All
twelve evidence validators were additionally replayed directly against
the real frozen evidence bytes in a 0444 mirror tree and accept them.

Exact launch procedure once `ssh majj@10.26.14.64` answers again (one
job, progressive boundary only; the 10-job ceiling is untouched and the
queue was empty at before-v4 FINAL):

1. Read-only reconcile on the login node: `squeue`/`sacct` empty for this
   user, campaign ledger still 19 records with SHA-256
   `5381b3b3a8ffcf8ae1566eb63640719ddec13955f681dbeca5361bbf9321cd5a`,
   `stages/weyl-context-core-after-v1` absent, no accounting row for an
   after-v1 job.
2. Materialize commit `342a0511` in the HPC fetch checkout (fetch
   `origin/codex/math-benchmark-suite`; never pull into a dirty shared
   checkout), verify the after stager/driver hashes above.
3. Build the override payload exactly as the before-v4 launch did
   (changed inputs versus the campaign-pinned baseline), then run
   `python3 hpc/stage_weyl_context_core_after.py PAYLOAD_ROOT
   OVERRIDES_SHA256` on the login node.  It creates only
   `stages/weyl-context-core-after-v1` and submits exactly one job with
   `queue_before=[]`.
4. Record job/pin/intent/receipt here as SUBMITTED_NOT_VERIFIED; never
   resubmit an uncertain intent.

The gate proves only: both known regressions pass on the repaired source
with full golden-equal streams, the retained ladder control passes, the
632-test inventory is unchanged elsewhere, and integrity/ledger checks
hold.  It releases no mathematical, cache, performance, memory or rank
claim; the repaired production source (`domain_builtins.rs`,
`typed.rs`) remains uncommitted until the AFTER gate passes and is
independently inspected.

## SUPERSEDED: BEFORE-v4 FINAL COMPLETED — tests-first Weyl BEFORE retained — 2026-10-02

Job `3886748` is FINAL `COMPLETED 0:0` after 450 seconds on `cu081` (2 CPUs,
8 GiB requested, batch MaxRSS `1310532K`).  This is the retained tests-first
BEFORE evidence for the Weyl-owner semantic mismatch.  Independent inspection
(`tests/reference/hpc/math_weyl_context_core_before_v4_2026_10_02.json`)
verified: all six checker suites pass exactly 32+17+18+17+28+7=119 tests; the
release build (279.93 s) and the 632-test atlas-core inventory pass; the
focused selector run fails with exit 101 on exactly the two expected
regressions (`weyl_context_core_cold_dual_original`,
`weyl_context_core_prewarmed_dual_original`; 0 passed, 630 filtered); the
retained `root_ladder_coordinate_boundary_original` control passes; both
original fresh-process captures byte-match the four frozen v8 goldens
(cold stdout `7a614e47...`/empty stderr, prewarmed stdout
`5074aab3...`/stderr `ef4404d8...`); both Rust fresh processes exit 1 with
differing streams; the report hash chain is intact (report
`3d0c7c91f672e6bf41830296864a621ee11bbd260735e37c0441d6c499953bce`); source
and final integrity rechecks pass; the ephemeral workspace is absent; no legacy
path opens occurred; the queue is empty; the ledger remains at 19 records
(SHA-256 `5381b3b3a8ffcf8ae1566eb63640719ddec13955f681dbeca5361bbf9321cd5a`).

This retains ONLY the BEFORE proof: unchanged production, original goldens
matched, exactly the two known Rust regressions failing for the observed
semantic reasons.  It releases the smallest owning Weyl-owner semantic
implementation change followed by the same original-backed AFTER gate.  It is
not an AFTER pass, mathematical acceptance, cache acceptance, a
performance/memory claim, or a rank release.  A1 cross-dual products are only
`s0*s0`; G2/B2/C2, reverse operand orders, inner-class dual construction,
no-value relations, sole-WeylElt lifetime and work-count gates all remain
separate later steps.

UPDATE, same day: the released minimal repair is now implemented locally
(unverified).  `RootDatumHandle` gains an interned `Arc<DatumWeylIdentity>`
carrying a success-only lazy owner coordinate kernel (`RootSystem`) and the
abstract-group cell (`WeylInterface`); a process-global weak registry interns
handles by complete datum content plus preference, matching the original's
`root_datum_value` table.  `dual(RootDatum)` installs the source's abstract
group into a cold canonical target and never overwrites a prewarmed one.
Weyl `=`, `!=` and `*` now check the abstract-group `Arc` identity (the
relation check also fires at no-value level) and replay the right external
word in the left system instead of comparing foreign root permutations.
All eight handle construction sites route through the private `interned`
constructor; structural `RootDatum` `Eq`/`Debug` are preserved.  The
production patch `hpc/patches/weyl_context_core_repair.patch` (SHA-256
`246cd2d0dd48ee68387ed8f72a10a156e43b7d8c8c6dcb693474832450111c5c`,
29,430 bytes) was verified locally to apply to the accepted baseline
(domain_builtins `0359261f`, typed `7085d231`) and to reconstruct the exact
working-tree bytes (domain_builtins `e6987e7c`, typed `614975c5`).  No local
build or test ran (HPC-only rule); the two regression tests must now PASS in
the changed-input AFTER gate, which also needs the 630-other inventory with
the focused pair run serially or skipped in the parallel suite (global
interning makes the two A1 fixtures interfere if run concurrently in one
process).  The candidate is NOT committed and grants nothing until the AFTER
gate passes.

UPDATE, same day (AFTER preparation in progress, nothing verified or
committed yet).  The changed-input `weyl-context-core-after-v1` gate is being
prepared.  Done so far: `classify_after` plus the AFTER selector-state
validator are appended to `hpc/weyl_context_core_regression.py` (BEFORE logic
byte-identical; `_validate_observation_shape` is now parameterized by schema
and label); the AFTER sbatch `hpc/math_weyl_context_core_after.sbatch` (SHA-256
`b2f2ec5fb9b76fd6f9ee42052973d77f9d8e1dbb7f0446278ae7d359cb159082`) differs
from the BEFORE sbatch only in job name, output name and driver filename; and
`hpc/stage_weyl_context_core_after.py` (7121 lines, SHA-256
`805d8d5f67a7ab82e85ecbc97c03d8a6ba9b5d34d37232bfbf56042f66ed8ac1`) is a
verified faithful mirror of the BEFORE stager with exactly 28 changed/added
lines (docstring, the five identity constants, `SUBMISSION_ENABLED = False`,
the repair-patch and before-v4-record entries in `STAGE_INPUT_NAMES`, and the
SLURM output pattern).  The mirror still carries BEFORE semantics in its body
and must NOT be launched as-is.

Pinned AFTER constants already determined: repair patch
`hpc/patches/weyl_context_core_repair.patch` SHA-256
`246cd2d0dd48ee68387ed8f72a10a156e43b7d8c8c6dcb693474832450111c5c`;
post-repair file hashes domain_builtins `e6987e7cfc...` and typed
`614975c5e2...`; the 1567-file AFTER source manifest SHA-256
`3f8cf4753f29d33df8273086254a4f09170ada46f05e9c13acfdac2f5ab85c33`
(recomputed from the frozen before-v4 report manifest with exactly those two
entries replaced).  The direct predecessor is the before-v4 SUCCESS: job
3886748, stage device 3431958692 / inode 162130669722468804, 180 files / 18
directories / 4,151,738 bytes, pin `54221cf4...`, creation `c9197be0...`,
contract `2affae9a...`, transaction `4ae83ab4...`, prepared `eaa076df...`,
sealed `227bf9aa...`, published `0dc33624...`, intent `c94c6cc6...`, receipt
`f53f3766...`, report `3d0c7c91...`, out `977be4c8...`, overrides
`add7dfe0...`, and its three campaign creation events (prepared/sealed/
published).  The before-v4 independent-inspection record is
`tests/reference/hpc/math_weyl_context_core_before_v4_2026_10_02.json`
(SHA-256 `bf69999feb945b67f586d92d378cc20d238cb518ef84ad6ce11b54e3e74200d3`).

Remaining AFTER work, in order: (1) rebind the AFTER stager's PREDECESSOR to
that before-v4 success record, add a `validate_before_v4_result` check on the
inspection record, update LIFECYCLE, set `CURRENT_STAGER_PATH` to the AFTER
stager and extend `FROZEN_LAUNCHER_HASHES`/`FROZEN_LAUNCHER_STATES` so the
BEFORE pair is required disabled and only the AFTER pair enabled; (2) write
the AFTER driver `hpc/math_weyl_context_core_after.py` (apply the repair
patch after the regression patch, pin the post-repair hashes, call
`classify_after`, require both captures' full-stream/exit equality flags to
be true); (3) add AFTER tests to the regression-module checker and a new
driver checker suite; (4) migrate the allowlist, `progressive_submit`
ACTIVE_STAGE_NAME and the stage-creation tests to after-v1; (5) disable the
BEFORE pair in the working tree and re-pin the allowlist hashes; (6) freeze,
reconcile and launch exactly once.  No part of this is verified or committed.

UPDATE, same day: steps 1, 2, and the progressive_submit migration are now
implemented locally (unverified, uncommitted).  The AFTER stager
`hpc/stage_weyl_context_core_after.py` (SHA-256 currently
`aba2930c44a73d8c008e7c5acae479f1d0cab353ac1e12190cf728f08e6fc7e2`, still
`SUBMISSION_ENABLED = False` until the freeze) now binds the before-v4
success as its direct predecessor: the full 37-file campaign event set, the
180-file/18-directory/4,151,738-byte tree (canonical inventory SHA-256
`1fa45944f8f115cb6de516feb15a2d597b3167ba33050a135add020351bd1ee7`), the
19-record ledger `5381b3b3...`, and the new `validate_before_v4_result`
validator wired into both run paths.  `REPAIR_PATCH_HASHES`,
`REPAIRED_SOURCE_HASHES` (domain_builtins `e6987e7c...`, typed
`614975c5e2...`), `AFTER_SOURCE_MANIFEST_SHA256` (`3f8cf475...`) and the
58-entry `STAGE_INPUT_NAMES` (53 prior plus the repair patch, the before-v4
result record and the AFTER trio) are pinned.  The AFTER driver
`hpc/math_weyl_context_core_after.py` (SHA-256
`9ab7426a4e725fd9eee5f5ae31a968ffb60fb870a3b37f1cb68806e6e1626f35`) applies
the repair patch after the regression patch, requires the repaired manifest,
calls `classify_after`, and requires both captures' `full_stdout_equal`,
`full_stderr_equal` and `exit_status_equal` to be true.  `progressive_submit`
now has `ACTIVE_STAGE_NAME = "weyl-context-core-after-v1"`, the lineage gained
`("weyl-context-core-before-v4", "3886748")`, and the contract pin schema is
`atlas-weyl-context-core-after-pin-v1`.  The BEFORE pair is now disabled in
the working tree (stager `639b5c20...`, driver `31f5aef2...`).  Remaining:
the test-file migration (stage-creation, progressive-submit, allowlist,
regression AFTER tests, and a new AFTER driver checker suite) with the
updated `EXPECTED_TEST_COUNTS`, then freeze, reconcile and one launch.  No
local build or test ran; nothing here is verified or committed.

## SUPERSEDED launch record: BEFORE-v4 submitted exactly once as job 3886748 — 2026-10-02

The changed-input `weyl-context-core-before-v4` pair completed its remaining
launch steps this session.  Fresh read-only reconciliation at
2026-10-02T14:16:53Z found an empty expanded queue, predecessor job 3884903
FINAL `FAILED 1:0`, no before-v4 accounting row, the 18-record campaign ledger
at SHA-256 `fec706e87cd1a28c0700b55698a86b8c69553875b89d15274789065cf740e6b9`,
the before-v3 predecessor tree intact (141 files/18 directories/3,676,351
bytes), and no v4 stage, events, `.incoming` or transport.  The reconciliation
record is
`tests/reference/hpc/math_weyl_context_core_before_v4_reconciliation_2026_10_02.json`
(SHA-256 `640102e5ea5fd2ea6eb5741fb94fb7adc17848eb701692cd929a085d9b2b4190`).

The final whole-file hash pins were completed at 2026-10-02T14:07Z: both
`SUBMISSION_ENABLED` flags are `True`,
`FROZEN_LAUNCHER_HASHES["hpc/math_weyl_context_core_capture.py"]` is
`a8f784181cf4ec02d58ce00e5b64c0f10910ef975f65f7b53784be9d540e37ab`, and
`hpc/test_stager_allowlist.py` pins the current stager/driver bytes
(`c40520be...`/`a8f78418...`).  This session independently re-verified the
resulting state before launch: the 53-input hash delta against the v3 freeze
(exactly seven changed harness files plus the added v3 failure evidence), AST
parse and exact checker counts (32/17/18/17/28/7=119) of every changed Python
file, both flag states, every cross-pin, the restored historical
`ladder BEFORE v2 campaign policy changed` literal, and its mutation
regression.  A pre-launch stability recheck confirmed all 53 local hashes still
matched the frozen manifest.  The v4 freeze record is
`tests/reference/hpc/math_weyl_context_core_before_v4_freeze_2026_10_02.json`
(SHA-256 `e6e467318a757d33c478f85da5a8f832b2994a9b660c5d4e189c1d5c9a5b45d9`),
binding manifest
`add7dfe0a9d4f0efa1b7d73333149c934d9f367812b47e30a94f5fbe5ff12ebd`
(53 inputs, 1,805,958 bytes, 6,690 manifest bytes).

The complete 54-file payload (53 inputs plus `overrides.json`) was streamed to
the single registered transport
`/public/home/majj/.weyl-core-before-v4-payload`, verified remotely byte-for-byte
against the frozen manifest (all hashes, 0444/single-link modes, exact
topology), and the stager was invoked exactly once from that payload with
login Python 3.9.12.  It returned 0 and printed the receipt; the shared
campaign submission boundary accepted exactly job `3886748` with
`queue_before=[]`.  Pin SHA-256 is
`54221cf4545875ec768c09d92753ceba5c5140faeacd895cbf8af36327e18e78`, stage
creation receipt `c9197be0f005da73d110a3aac6d9b0531fe92728dd0f8db37d7c2a8b992307c7`,
contract `2affae9a40b11f15ca97ad80e38ca95534ff6294ff43e0c7823ede57202d919a`,
transaction `4ae83ab4446267af676897c4b1b1ec2ffcf1ead2db30bca431c0140960dff030`,
prepared/sealed/published events
`eaa076df...`/`227bf9aa...`/`0dc33624...`, submission intent
`c94c6cc6b8a28f795bb3f8412c15edd7c2ca24b3611378a6771ce93bc9d10ab2`, receipt
`f53f37662aa4607647b9055e15544fdd67e8bac9a328971a919afada93d18798`, and the
19-record ledger SHA-256
`5381b3b3a8ffcf8ae1566eb63640719ddec13955f681dbeca5361bbf9321cd5a`.
The job was RUNNING on `cu081` when observed.  Both transports (remote payload
and local `/tmp/weyl-v4-payload-wzm6fr5h`) are removed; the remote removal
first verified the exact single-child topology.  The submission record is
`tests/reference/hpc/math_weyl_context_core_before_v4_submission_2026_10_02.json`.

Status is `SUBMITTED_NOT_VERIFIED`: do not resubmit, create a sibling, accept
the repair or release any rank/performance/cache gate before FINAL independent
inspection.  All four BEFORE stages remain immutable.  No production repair is
authorized by this launch; the gate must still prove the two focused Rust
regression failures and exact original/golden agreement on unchanged
production bytes.

## SUPERSEDED: v8 is FINAL and confirms two Rust Weyl-owner regressions — 2026-10-02

Latest: BEFORE-v3 job `3884903` is FINAL `FAILED 1:0` after 56 seconds on
`cu001`. Again 112 prerequisite tests passed and the seven-test allowlist suite
reported five passes/two errors. Root's stage-version replacement accidentally
changed the expected campaign error text for the immutable historical ladder
BEFORE driver from v2 to v3. The historical script itself is unchanged and
correct; the current Weyl driver was not reached by that allowlist loop. This
was a coordinator migration/review error, not a new mathematical result.
Restore that one historical expectation, add a targeted mutation assertion,
and include filename/line/node kind in the generic rejection diagnostic.
BEFORE-v4 preparation is disabled pending exact review and reconciliation.
Independent static reviews of the v4 candidate are now complete; final whole-file
hash pins, fresh reconciliation and HPC execution remain pending. The capture
test candidate is `f0188e67fb421284ef1ed0e5e05016718ab59ad6b3df6d75c73f413fa31fc124`
(182,273 bytes, 28 test methods). Do not call this an executed checker pass.
All three BEFORE stages remain immutable; no source reconstruction, Cargo,
Atlas or mathematical regression ran in any of them. The v3 eight-file delta
used one in-memory transport; `/public/home/majj/.weyl-core-before-v3-payload`
was removed after its single submission. No local temporary path or worktree
was created. No production repair or performance claim is available.

Kimi's latest actual invocation completed in 109.531 seconds with CLI 2.1.1,
model `kimi-code/k3-256k`, session
`session_f338a507-d213-4a17-8ad7-fd3297da9d2c`, using the tool-free probe
profile. It reviewed only four supplied guard literals and the historical
version mutation assertion; owned/touched files were empty. Raw prompt,
streams, process/result records and independent review are retained under
`docs/evidence/kimi-weyl-allowlist-20261002/01-review/`. All supplied source
hashes still matched at review. Codex accepted the literal-match corroboration;
the optional extra assertion was not added because the existing positive
full-module loop already rejects a wrong historical expectation. This is
static review, not a test or mathematical acceptance. The runner exited 0,
was collected, and recorded no remaining live process-group members. No
temporary path, worktree or HPC job was created by this invocation. Lesson:
preserve historical driver versions and assess mutation coverage together
with the positive loop. Native Codex reviewers are separate from Kimi; earlier
Kimi ExitStack editing and the timed-out directory review have their own
records and must not be omitted from usage summaries.

Historical BEFORE-v2 job `3884880` is FINAL `FAILED 1:0` after 56 seconds on
`cu001`, batch MaxRSS `86800K`. The first five checker suites passed all 112
tests, including the corrected 28-test driver suite. The final allowlist suite
ran seven tests: five passed and two rejected non-allowlisted top-level
expressions (dictionary unpacking and a subscript). Cargo, Atlas and the Rust
regressions never ran. Fix the expressions using the existing allowed syntax;
do not broaden the safety policy. The immutable failed stage remains intact.
The changed-input BEFORE-v3 successor is being prepared with its launch guards
disabled. No mathematical BEFORE, production repair or performance claim exists.

Historical BEFORE-v1 job `3884862` is FINAL `FAILED 1:0` after 47 seconds on
`cu001`, batch MaxRSS `87272K`. Its first four checker suites passed
(32+17+18+17=84); the 28-test driver suite had 26 passes, one failure and one
error. The failure retained an obsolete predicted-output classification after
switching the synthetic original arm to true goldens. The error used a
format-sensitive source substring across a changed newline. Both are checker
defects, not the required mathematical BEFORE result. Cargo, Atlas and the
two new Rust regressions never ran. The failed stage and report are immutable;
no duplicate or same-input retry is authorized. The working tree is preparing
the changed-input `weyl-context-core-before-v2` successor. Its independent
static review passed and the sole pair is enabled, frozen at manifest
`0f66de83019ae828b258a06aa38afe312e166075bb5bc1a685c9752cc7b42542`
(51 inputs). Fresh reconciliation found an empty expanded queue, the exact
immutable failed predecessor, and no v2 stage/events/transport. The freeze
record is `tests/reference/hpc/math_weyl_context_core_before_v2_freeze_2026_10_02.json`.
BEFORE-v2 was submitted exactly once as job `3884880`, initially RUNNING.
Its pin is `bc98d9a033ab1fb0829456ef8a852b0db504a2eeffff6a6e6c377ddad045b750`
and creation receipt is `82b88fcc782c540055f11ef07e37244e73bec13fd1179c7d0b3c40747978dec9`.
The eight-file delta reused the verified BEFORE-v1 baseline. The sole remote
transport `/public/home/majj/.weyl-core-before-v2-payload` was removed after
exact-manifest verification; no local temporary directory or worktree was
created. Do not resubmit. Its failed result is summarized above.
Source/golden scope remains unchanged; no production repair is allowed yet.

Historical submission details for the frozen `weyl-context-core-before-v1`
snapshot follow. Its source
is the accepted 1,561-file baseline plus only the new session test block and
six fixture/golden files (1,567 files, manifest
`55f807cadb712377cbf6c0c79250b1d5e739e9757290a24209a086f89632850f`).
The single BEFORE job `3884862` failed in its checker preflight as recorded
above; no mathematical result was produced. Independent static reviews of the
launcher/driver/allowlist migration passed. The complete 50-input manifest is
`df73c17df5b1735692ec18410e97aef57ad7b395c7b95daae2e6132a773dc537`,
recorded in `tests/reference/hpc/math_weyl_context_core_before_v1_freeze_2026_10_02.json`.
Fresh scheduler, ledger and exact-stage reconciliation preceded its one
submission. The pin is `e713b56909d43d3f01d8782ef14d269c69d294b4418e56fba1f1929618ed8e12`
and creation receipt is `7a3c0d7f5f8efba29abcf399545fbadf85c92a01ba45f2430be001bab11ff51e`.
Seventeen changed inputs were transferred in memory from the local machine;
the other inputs came from the verified v8 stage. The sole remote transport
`/public/home/majj/.weyl-core-before-v1-payload` was removed after exact-manifest
verification. No local temporary directory or worktree was created. Do not
resubmit or invoke a mixed snapshot. This preparation has changed no Weyl
production implementation.

`weyl-context-core-capture-v8` was submitted exactly once as job `3884807` and
is immutable FINAL `COMPLETED 0:0`: 345 seconds on `cu001`, 2 CPUs, 8 GiB
requested and `1126772K` batch MaxRSS. All
32+17+18+28+7=102 checker tests, toolchain probes, source reconstruction and
the release build passed. The sealed 324,940-byte report has SHA-256
`e644017f6c040691c91ee56fdaeac09a3e8d834ec9956939c2ba2530c80c5c73`;
its immutable status remains `WEYL_CONTEXT_CORE_CAPTURE_RECORDED_UNREVIEWED`
because an execution report cannot review itself. The separate independent
inspection passed and classifies the result as
`CAPTURE_COMPLETE_RUST_SEMANTIC_MISMATCH_CONFIRMED_REGRESSION_REQUIRED`:
`tests/reference/hpc/math_weyl_context_core_capture_v8_inspection_2026_10_02.json`
has SHA-256
`b2c7f4ece709df3506c3224a57abad896f3ecfcd6625fea97731619f629e1da3`.

The complete fresh-process raw streams establish two distinct observable
errors. For cold canonical duals, original Atlas treats the owners as
compatible: `=` is true, `!=` is false and multiplication succeeds in each of
the two dual-construction directions (SC-to-adjoint and adjoint-to-SC); Rust
returns false/true and rejects both products as Weyl group mismatches. Reverse
operand orders remain pending coverage. For independently prewarmed
incompatible owners, original
Atlas rejects `=`, `!=` and multiplication before producing values; Rust
incorrectly returns false/true for the two relations and rejects only the
products. The frozen source-prediction strings were not exact goldens—they
missed original list-display spaces and some diagnostic shapes—but the four
sealed raw streams are complete and the independent review compared those
artifacts directly.

The original-backed regression library now retains all four byte-exact
goldens:
`weyl_context_core_cold_dual.oracle.stdout`
(`7a614e47b75469c441e774cbe46769dcd769c5cc0a2a31cf1868ff9b59a780dc`),
its empty stderr
(`e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`),
`weyl_context_core_prewarmed_dual.oracle.stdout`
(`5074aab3290d4ae404bb7abb0b24ccc99036f616abec518c6479e59ec2db021f`)
and stderr
(`ef4404d85f7252a611f9f4e4a5205fe6bbac2c66f48b7378a30a8053f986e157`).
The new `weyl_context_core_regression_catalog.json` is 5,208 bytes at SHA-256
`6a9f960753bcd2afb575cb5207d4310f21a5694505d02a83f4e4142211eaa634`.
Two focused Rust regression tests have been added to
`crates/atlas-core/src/session.rs` (current SHA-256
`969cdb27ccae61ca4fe3e36219801037d475517bee3614fead553024818307e7`),
and the tests-only production-relative patch is frozen at
`hpc/patches/weyl_context_core_regressions.patch` (SHA-256
`ccd3009892dbeae2f146dff9924efa6d6bede3dca910d026c8f3ab3e1b4c49cf`).
These tests are unexecuted. Before any production repair, one changed-input
tests-first successor must prove on HPC that the frozen original matches the
four exact goldens and the unchanged Rust binary fails the two focused tests
and complete comparisons for the observed semantic reasons. Only after that
retained BEFORE result may the smallest owning implementation change and run
the same original-backed AFTER gate.

The capture grants no Weyl-context mathematical acceptance, cache acceptance,
performance or memory improvement, or rank release. The 345-second job was
dominated by its 281.53-second release build; each one-shot mathematical
invocation took only 0.00–0.01 seconds, so no Rust/original speed ratio is
valid. The queue was empty after inspection and both exact v8 transports are
absent. Never invoke the v8 creator again or create a retry sibling. Production
repair, cache work, profiling and rank escalation remain blocked until the
tests-first BEFORE is retained.

## Historical v4 failure and v5 submission record — superseded by the current section above

Job `3884494` and stage `weyl-context-core-capture-v4` are immutable FINAL
failures.  The job ended `FAILED 1:0` after 24 seconds on `cu001`.  The creator
(32), progressive-submission (17) and contract (18) suites passed and were
recorded with exact GNU-time metrics.  The 27-test capture module then failed
to compile under HPC Python 3.9 before any test started: one `with` statement
had nineteen managers inside an outer two-manager `with`, exceeding CPython's
static block-stack limit.  The allowlist, source reconstruction, Cargo,
original Atlas, Rust Atlas, mathematical comparison and performance comparison
never ran.  The report recorded three commands, zero Atlas invocations and zero
captures.  Exact evidence is
`tests/reference/hpc/math_weyl_context_core_capture_v4_failure_2026_10_02.json`
(SHA-256 `051f5292f51dc2ed413a277932fb79556295732a58634c7cdff7cd27fe807517`).
This is a harness error, so no mathematical regression was triggered.

The direct v4 predecessor is sealed by its 95-file/18-directory/2,791,040-byte
tree (SHA-256
`7868842059b3bef7b6d53196f76a48c89105bd2a7d00a27b82fe5446b04f8d5b`),
eleven-record ledger (SHA-256
`00b7bc1cb917bfa15d729620f807cff92d2988c375a237d13fe5fdce9320a8ab`),
job, report, pin, intent, receipt and v1-v4 creation-event history.  A read-only
SSH attempt at `2026-10-01T19:56:18Z` timed out before authentication and was
discarded as uncertain; after connectivity returned, the complete r7
reconciliation at `19:59:53Z` found the queue empty, the same eleven-record
ledger, FINAL v4 accounting, two identical sealed-tree scans, no v5 target or
creation residual and no v5 transport.  It is frozen at
`tests/reference/hpc/weyl_context_core_precreation_reconciliation_r7_2026_10_02.json`
(SHA-256 `876bb13b3b34337e351e9a2a655f9e1b0ac607a91a876ec33e3e2001bf210dd8`).
Neither attempt created a remote path or job.  All v3 and v4 handoff transports
are removed.

`weyl-context-core-capture-v5` is the sole changed-input successor and was
created/submitted exactly once as job `3884727`.  The first durable observation
found it `RUNNING` on `cu001`, with exactly one outstanding job and a 12-record
ledger at SHA-256
`e5c1ed292453da24720c180dd0702cc031bd125716bf0ce053049e32bcc0d4bc`.
Pin/creation/intent/receipt SHA-256 values are respectively
`80d61ee985cbf6a920908d7b754e48654112baa7790611776f7a2cf715c47a13`,
`1e7d077e43d6d1cec6199e1e77e2842edf314742e4694c58c155179467a9a95f`,
`392747371545a3b78f51ee462eac4e41f392e4cad955b95acecd04d15a954ee4`
and `5651a5490ccd58b9c257d26f0b79f432a01a4acaf97543f583a72e2d5d11ca4c`.
The exact submission evidence is
`tests/reference/hpc/math_weyl_context_core_capture_v5_submission_2026_10_02.json`
(SHA-256 `fedccff3c50211e0b470651e95cea354782b5f3e63df6f566ab5489fdbb43f3d`).

Its only executable-test repair replaces the oversized manager list with
`contextlib.ExitStack`, preserving every patch target, alias, environment
value, entry order, reverse exit order and assertion.  The v5 lineage/allowlist
migration binds v4 directly and retains the five-suite 32+17+18+27+7=101
compute gate before build or capture.  Static AST parsing, method inventory,
whole-file hashes, all fifteen execution-node hashes and `git diff --check`
passed before submission; the current section records the later FINAL result.
Do not invoke the v5 creator again, submit a sibling or advance rank.

The v5 payload contained exactly 37 frozen inputs plus `overrides.json`; its
4,450-byte manifest SHA-256 was
`09d623a9663d2612787a6408422dc54074dd0ecd6ec6a105ea262fbf621c07ac`
and bound 1,391,776 input bytes.  One local transport and one remote transport
were created, one rsync and one creator invocation succeeded, and both exact
transport directories are now removed after the durable receipt.  The first
direct-`rm` cleanup command was rejected before process creation; bounded
no-follow Python cleanup then removed only the two registered directories.

Kimi performed only the bounded mechanical ExitStack edit with CLI 2.1.1,
model `kimi-code/k3-256k`, session
`session_86f6fd83-5814-4362-a875-8f67b49bfe4c`, and the Read/Edit-only profile.
It exited 0 after 113.362 seconds with no live group members.  Independent
review against the exact submitted v4 byte confirmed that only the import and
manager block changed and rejected no suggestion; Codex then applied the
separate v5 identity/evidence migration.  Its sole owned/touched file was
`hpc/test_math_weyl_context_core_capture.py`, changing from SHA-256
`0688b39813f80026ad0f25297f670d5fce97bb740b07dd7df1fc179cca8ecb0e`
to `b44b467883a1b2267d2ae9598bb576ce3100cbd8655c93a7d927785379e6cbd9`
for the bounded mechanical edit; later coordinator migration produced the v5
candidate hash.  The frozen prompt, raw streams, process/result records and
review are under
`docs/evidence/kimi-runtime-20261002/13-v5-exitstack-refactor/`.  No local
program test was run; the subsequent v5 HPC result is recorded in the current
section above.  Reusable lesson: use `ExitStack` for large ordered patch sets
and still compare the generated edit against the submitted predecessor
byte-for-byte.

Reconciliation r6 also records a coordinator error: a read-only v3 tree helper
was first run without `-B`, creating exactly two `.pyc` files and one
`__pycache__` directory.  Those three generated paths alone were removed; two
subsequent scans restored the exact sealed v3 tree.  Every later remote Python
probe uses both `python -B` and `PYTHONDONTWRITEBYTECODE=1`.  Evidence is
`tests/reference/hpc/weyl_context_core_precreation_reconciliation_r6_2026_10_02.json`
(SHA-256 `3fedbe71bdcaf8fa2ecd32b6c85201beabc13d3b5706040b13587be6032a7720`).
Do not reuse v4 or v5, create a retry sibling or advance rank.  No mathematical,
cache, performance or rank claim is released.

## Directory-growth guard is durable on GitHub — 2026-10-01

The bounded HPC campaign/worktree lifecycle correction is committed as
`7db322a313c3bcda1c0628e64b857ed30496e651` and is now pushed to
`origin/codex/math-benchmark-suite`; local and remote branch tips matched after
the push. This makes the already HPC-verified stop-growth controls available to
a fresh checkout. It does **not** authorize deletion of any of the 318
historical HPC directories, 24 secondary worktrees or 141 `/tmp/atlas*`
directories. Cleanup still requires a read-only reverse-reference/CAS audit,
an exact retirement receipt and explicit authorization for each exact path.
No directory was created or removed during this durability step.

## Historical fixed-child creator checkpoint, superseded above — 2026-10-01

This section preserves the pre-v1 checkpoint only; it is not current HPC state
or submission authority.  The v4/v5 record at the top of this file controls.

The next and only eligible mathematical action is still the core-only A1
Weyl-identity capture. It has **not** been submitted. HPC connectivity returned
and a fresh read-only four-way reconciliation completed: `squeue` is empty;
the campaign ledger has exactly its seven confirmed records and SHA-256
`b336fb831de28f72e9138284d2c6f07ecfb178f617ebe4bec78e1c82ffb8eaa7`;
`sacct` reports jobs 3872554/3873400/3875239/3879103 completed and the three
preserved harness failures 3872594/3873497/3874203 terminal; and the exact fixed
stage
`/public/home/majj/atlas-rust-campaign-20260930/stages/weyl-context-core-capture-v1`
is absent. No remote directory, transport, source tree or job was created.
The complete login-node observation is frozen in
`tests/reference/hpc/weyl_context_core_precreation_reconciliation_2026_10_01.json`,
SHA-256 `c876c133970328c2d53b7dd66be9be50a16f268b6afe050945325ce26c6726e9`.
That observation is now a historical checkpoint, not current authorization:
the new read-only `squeue` attempt at `2026-10-01T14:42:36Z` timed out before
authentication with no remote output. Queue, ledger, accounting, intent/receipt
inventory and target presence are therefore currently unknown. The failed
attempt is frozen in
`tests/reference/hpc/weyl_context_core_reconciliation_timeout_2026_10_01.json`,
SHA-256 `4f38e83298fd32946ed3f0f13c2b1db6521e1fea01fcb7d58d80d095afcb30cc`.
No directory, file or job was created. Repeat the complete four-way read-only
reconciliation after connectivity returns before invoking the creator.

Do not fill that absence with a manual `mkdir`. The formerly missing creator is
now implemented locally, but is not yet HPC-verified. It exposes no arbitrary
root or stage-name parameter and can create only this exact child through
`PREPARED -> hidden transaction -> SEALED -> RENAME_NOREPLACE -> PUBLISHED`.
The contract binds the exact campaign, predecessor-ledger bytes, input hashes,
submission script, pin schema, changed-input reasons, retention class and
retirement condition. A pre-created final, symlink, extra sibling, changed
contract, lock replacement or competing intent fails closed. The same contract
may recover only its own hidden transaction; it never creates a retry sibling.
The first atomic `submission-intent.json` is the uniqueness gate before
`sbatch`, and an already-running exact eighth ledger attempt can close only
against the current decimal `SLURM_JOB_ID`, without resubmission.

The frozen capture surface remains two fresh-process fixtures: cold canonical
dual sharing and prewarmed-target separation. The compute driver now runs five
checker suites before any build or capture: creator 21, progressive submission
17, contract 18, capture 27 and allowlist 7, exactly 90 methods, followed by
four tool/source commands for exactly nine command records. Current SHA-256
values are progressive submission `2481383117e0...`, fixed stager
`b165795e0ceb...`, driver `550ff8651e6a...`, creator test
`e0820335582b...`, progressive-submission test `2c4beac47a31...`, capture test
`7d51c65da190...` and allowlist `e1767281c5db...`. Static review found and
repaired an unconfirmed-ledger validation dead end, several directory-entry
durability/replay windows, derived count/order assumptions and the omission of
the 17-test submission suite from the frozen gate.
`git diff --check` and the 25-entry worktree guard pass. No local Python test,
AST execution, Cargo command, Atlas command or benchmark was run; these bytes
remain unverified until the first compute job executes them.

An independent audit found and repaired one evidence-test contradiction. The
old parent-only positive test rewrote the currently full-hash-frozen capture
stager/driver and then expected validation to pass, which was impossible. It is
now a fail-closed negative: merely changing the policy label is rejected, and
even a coordinated parent/capture role rewrite cannot synthesize historical
bytes. Production validation was not weakened and the test count remains
seven. A final exact-byte static review found no remaining P0/P1 in that
boundary; HPC execution is still required.

Source review fixes the semantic order before optimization. Original
`root_datum_value` weak-interns live equal data, owns a lazy strong WeylGroup,
and `dual()` shares that group only when the canonical target is cold. Binary
Weyl `=`, `!=` and `*` compare group addresses before the no-value gate. Current
Rust rebuilds `RootSystem+WeylInterface` per call and uses structural handle
relations, so this is potentially a correctness repair before a cache. The A1
stage is sufficient to capture that first history but cannot release a general
cache: its cross-dual products are only `s0*s0`, an alias still keeps the old
datum alive after rebind, and Atlas output cannot prove the intended internal
cache topology. After independent raw-stream review and any A1 regression/fix,
progressively add G2 interface-order, B2/C2, both operand orders, sole-WeylElt
lifetime, inner-class dual and no-value relation gates. Only then add
work-count tests and same-node time/CPU/RSS A/B.

The acyclic source-backed candidate has an owner-local lazy coordinate
`RootSystem` kernel and a separately weak-interned abstract Weyl-group identity
carrying `WeylInterface`; neither points back to a handle. `WeylEltContext`
holds the handle plus both `Arc`s. Cross-coordinate operations replay the right
external word in the left system. Original inner-class construction also calls
`srd->dual()` immediately and retains both datum owners, so changing only
explicit `dual(RootDatum)` would be incomplete. This is design evidence, not an
implemented or accepted cache. The existing compact `[u8;32]` transducer value
is a separate later representation target and must not be mixed into this
semantic repair.

The same findings are now indexed in
`kb/sources/weyl-context-identity-and-sharing.md` with the append-only reading
snapshot `2026-10-01-weyl-context-source-prediction.json`. A hold-all
llm-wiki-compiler run recognized it and the earlier ladder source but the Codex
provider exited before generation; review list remains empty and no state or
candidate file exists. An escalated retry was rejected because transmission of
the exact two source packets plus `kb/AGENTS.md` to the external configured
provider lacks separate payload/destination authorization. Do not route around
that decision. The manually written packet is useful indexed evidence but not
compiler-generated wiki content.

## Kimi bidirectional ACP interaction verified — 2026-10-01

The follow-up requested a workable Codex–Kimi interaction channel. Implemented
`tools/kimi_acp_session.py` and the question-only
`.agents/kimi/interactive.md`; Codex can send tasks/answers/follow-ups, receive
progress and questions, cancel a pending turn, reconnect and close through its
terminal tools. Full usage, official sources and observed limits are in
[`slices/kimi_acp_interaction_2026-10-01.md`](slices/kimi_acp_interaction_2026-10-01.md).
`tools/kimi_acp_probe.py` drives the reproducible synthetic local integration
capture; all exact requests, profiles, model/session IDs, streams and exits are
retained in `docs/evidence/kimi-acp-20261001/`. CLI2.1.1 / `kimi-code/k3-256k`
made fourteen model-bearing prompts across twelve processes and three sessions.

First attempt: rejected the model's unilateral duplicate-policy choice and
its stale claim that auto mode was still active. Corrected the client to state
the actual successful ACP manual-mode switch on each new task. The corrected
run proved a real AskUserQuestion reverse RPC, coordinator answer, source-text
proposal, remembered follow-up, overlap rejection, cancellation and recovery,
new-process history reload, close/EOF, deadline and SIGTERM. Final inspection
also found pending-question and redundant-cancel warnings; the final client
settles reverse RPCs with `action: cancel` and avoids cancelling an idle/closed
session. The full third capture passed with empty stderr. All process groups
were empty after cleanup. Independently inspected the proposed comprehension,
explicit loop and docstring; rejected the final variant's Python3.8+ claim
because the shown `list[str]` annotation needs3.9+ without postponed evaluation.
No proposal was applied to Atlas. No generated program, build, fixture or HPC
job was executed.

Reusable lessons: `kimi acp` does not forward CLI profile flags; bootstrap and
load a restricted explicit session. Historical `-p` auto reminders can affect
later model behavior. A question is answered on its original RPC, not by
starting another prompt. ACP cancel is a notification, close retains history,
and replayed messages are not new model output. Settle question RPCs separately
on cancellation and independently review version claims as well as code logic.
ACP and MCP need an adapter
between them; no MCP installation or global configuration change was made.

Temporary paths `/tmp/codex-kimi-acp-v5iolxif` and
`/tmp/codex-kimi-acp-v0r5jtwl` and `/tmp/codex-kimi-acp-3mu4vpvi` are REMOVED.
No worktree or transport was created.
Three synthetic sessions remain only in the normal local Kimi runtime store;
credentials and unrelated sessions were untouched. Root AGENTS.md and the
indexed status document now contain the tested interaction contract.

## Kimi runtime integration verified — 2026-10-01

The user explicitly authorized local Kimi invocation/delivery/exit checks while
keeping Atlas builds and program tests on HPC. Local Kimi2.1.1 with the existing
account and `kimi-code/k3-256k` passed real response, bounded Read/Edit coding,
exact-session continuation, early/in-flight interruption, timeout and forced
process-group cleanup; an intentionally invalid profile correctly returned1.
Eleven invocations have retained exact prompts, scopes, profiles, model/session
records, raw streams, exit/cleanup evidence and independent byte inspection in
[`slices/kimi_runtime_validation_2026-10-01.md`](slices/kimi_runtime_validation_2026-10-01.md).
All had zero remaining live group members. No generated program was executed,
no Kimi changes entered Atlas source and no mathematical acceptance is claimed.
No suggestions were rejected in the synthetic edit; no Kimi output is an oracle.

Use `tools/kimi_subagent.py` and `.agents/kimi/{probe,editor}.md`; root AGENTS.md
now records the verified operation and exit contract. Stop the recorded wrapper
PID with SIGINT/SIGTERM, not broad process-name matching or SIGKILL of the
wrapper. Review partial files after interruption. JSONL includes trailing
session metadata; resume by explicit session ID. Keep existing local credentials
in place. Lessons: local login works, Read/Edit suffices for bounded edits,
explicit resume retained context, and controlled group cleanup works. The
historical storage failure was not reproduced or independently diagnosed.

Task temporary paths, all REMOVED: `/tmp/codex-kimi-connect-7j8qhgvc`,
`/tmp/codex-kimi-edit-gi3ingyo`, `/tmp/codex-kimi-exit-lu4gr37h`,
`/tmp/codex-kimi-exit-42mq4jxf`, `/tmp/codex-kimi-exit-klprhrwx`,
`/tmp/codex-kimi-exit-ly157o0d`, `/tmp/codex-kimi-inflight-w8q51gia`,
`/tmp/codex-kimi-invalid-47uuwus2`, `/tmp/codex-kimi-final-ornpmvgw`, and
`/tmp/codex-kimi-final-syw69cg2`. No worktree, transport or HPC job was created.
Six synthetic diagnostic sessions remain in the normal local Kimi runtime
store, indexed by the report; unrelated sessions/credentials were untouched.

## Kimi subagent research — 2026-10-01

The current executable is Kimi Code **2.1.1**, superseding the historical
0.42.0 version observation.
Version/help inspection and release-pinned official documentation establish the
CLI integration contract; see
[`slices/kimi_subagent_workflow_2026-10-01.md`](slices/kimi_subagent_workflow_2026-10-01.md)
and its local capability record. Prefer a bounded external CLI worker with an
explicit Markdown tool profile, frozen task packet and captured JSONL result.
`-p` uses automatic permissions and cannot combine with `--plan`, `--yolo` or
`--auto`; legacy Python/YAML/`--print` recipes are not the installed interface.
The report includes a no-tools patch-proposer template, session/timeout rules,
and independent review/HPC handoff requirements. No model prompt, programming
delegation, account verification, source change or HPC job occurred. No
Kimi-derived coding lesson or acceptance is claimed. The prior storage failure
remains undiagnosed; `KIMI_CODE_HOME` moves credentials/configuration as well as
logs. No temporary paths, transports or worktrees were created by this research.

## Subtask closure, Kimi and wiki workflow — 2026-10-01

The user now requires every completed bounded subtask to finish with durable
evidence/report, an updated handoff, a focused conventional commit and a push of
the exact verified branch. A missing or failed HPC gate means the subtask stays
open; unverified code is never pushed merely to satisfy the closure checklist.
Do not stage unrelated dirty-tree changes. Record a push failure explicitly.

Kimi is available for bounded routine work only. Every actual invocation must
record CLI/model/session when available, exact prompt scope, owned files,
result, independent review/HPC verification, rejected suggestions and lessons.
It is not an oracle for mathematics or Atlas semantics. Two CLI process
attempts were made for a bounded routine review/scaffolding task during the
current root-ladder repair, using `/home/hoxide/.kimi-code/bin/kimi` version
0.42.0 from the repository root. The first used `kimi --plan -p <prompt>` and
failed in argument validation with `Cannot combine --prompt with --plan`; no
agent, model or session started. The second used `kimi -p <prompt>` and
produced the operator-observed error
`storage write failed: unrecognized I/O error` before any observed model
session. A sandbox-unwritable persistent session store is the leading
explanation, not a proven cause because no raw log was retained. No
Kimi-produced or adopted work product was observed. An escalated retry was not
executed: the current approval outcome rejected sending the proposed
repository-derived payload to the external Kimi service without authorization
for that exact payload/destination. Do not work around that decision.

The exact prompt bytes for those two failed process attempts were not frozen
before launch, so they cannot now be reported verbatim and must not be
reconstructed from memory. This is itself a logging failure. Before every
future Kimi process, write the exact bounded prompt and owned-file list to a
repository report, record its SHA-256 plus CLI version, and only then invoke
the CLI. Use either interactive `--plan` or noninteractive `-p`, never both;
confirm a writable session location without weakening sandbox boundaries.
There is no Kimi-derived mathematical, implementation or review conclusion in
the current repair. The per-attempt incident/task report is
`docs/slices/kimi_cli_attempts_2026-10-01.md`; unknown prompt, time, owned-file
and exit-status details are explicitly marked rather than reconstructed.

The required wiki uses the repository-pinned `./kb/llmwiki`, with `kb/` as the
same-repository project. Follow `kb/AGENTS.md`, keep hold-all review enabled and
update source packets/pages alongside verified mathematical or algorithmic
changes. Compiler output is editorial evidence only and cannot release a math
gate.

The first bounded compiler attempt on2026-10-01 used exactly
`./kb/llmwiki compile --review --instructions AGENTS.md`. It discovered the new
`root-ladder-overflow-repair.md` source, then exited1 because its Codex CLI
provider failed before candidate generation; child output was withheld by the
tool. `./kb/llmwiki review list` subsequently reported no pending candidates,
and no `.llmwiki/state.json` or candidate file exists. A retry outside the
sandbox received a current approval rejection because it could transmit the
repository-derived source packet and `kb/AGENTS.md` to an external provider
without explicit approval for that exact payload/destination. Do not retry,
switch providers or work around that decision. The manually authored source
packet and snapshot remain useful input but are not compiler output; another
external-provider generation attempt requires explicit user authorization to
transmit those named files.

## Knowledge base in the source repository — 2026-10-01

The user requires the KB to evolve inside this Git repository, primarily around
Rust mathematics, algorithms and design; C++ is the baseline during alignment.
`kb/` is the Markdown/Obsidian vault; start at `kb/index.md` and follow
`kb/AGENTS.md`. The user selected llm-wiki-compiler on2026-10-01. Version
1.4.0-rc.2 is installed locally under kb/node_modules with package.json,
pnpm-lock.yaml and pnpm-workspace.yaml pinning its dependencies and no lifecycle
scripts. Use `./kb/llmwiki`; the launcher sets kb/ as the project root and
defaults to codex-agent, Chinese output and embeddings off. Native config holds
all generated candidates for review and excludes source indexes/snapshots.
CLI version/help succeeded; no model generation, login validation, test suite
or new mathematical validation was performed. Existing four curated pages are
preserved, not automatically migrated into generated wiki/concepts/. Source
packets must be maintained with code; inspect each candidate and destination
before approval and the entire diff afterwards. See kb/AGENTS.md for exact
commands and the known source-freshness/manual-edit limitations.
Update affected mathematical, algorithmic and architectural explanations with
task-related source changes. Existing docs, fixtures and acceptance records
remain authoritative in place. The initial four topic pages are draft source
explanations with a hashed working-tree reading snapshot, not new mathematical
or performance acceptance. No extra Git repository, worktree, temporary path,
plugin installation or HPC job was created for this documentation work. The
compiler is a repository-local npm dependency, not a Codex plugin. No temporary
task paths were created; installed dependencies use the normal pnpm store.

## Active task-worktree registry

The authoritative registry is now `docs/worktree_registry.json`, schema
`atlas-local-worktree-registry-v1`, SHA-256
`2059e9c5a09ab03d0a10eb2af6b6cd9946a940c980ded1b5d8074d977f2790cb`.
It records one PRIMARY plus24 exact LEGACY worktrees and zero ACTIVE rows.
Legacy rows are cleanup backlog, not reusable concurrency slots. The read-only
`hpc/local_worktree_guard.py` compares that complete inventory through a fixed
`/usr/bin/git` and sanitized environment, rejects special/drifted/unregistered
entries. The current guard rejects every ACTIVE row regardless of LEGACY count,
and `precreate` fails before reading registry, Git or filesystem state. Only
after a new durable receipt-bound creator is HPC-verified and the user
separately authorizes it can a fully described single ACTIVE exception be
considered. PRIMARY
pins path/branch but not its self-advancing HEAD; LEGACY/ACTIVE pin exact HEADs.
Run `check` before editing/delegation and handoff; stop rather than auto-prune,
repair or remove on any mismatch. There is no usable `precreate` result under
the current schema.

The guard, registry and22 mocked synthetic tests passed on an HPC compute node
inside accepted AFTER v3 job3875239. Earlier adversarial review found one P1: a
new live worktree plus a matching new LEGACY row could previously pass. The
accepted fix freezes the exact24 audited `(path, HEAD, branch)` identities; until a
separate retirement-receipt schema exists, a missing, novel or changed LEGACY
identity is rejected before Git is queried. A read-only
local `check` with Git2.43 still finds all25 live records equal to the registry.
The completed gate closes that standard-CLI laundering path in this bounded
scope. The guard now double-samples registry, Git inventory and the fixed
`/home/hoxide/mycodes/atlas*` sibling namespace, uses no-follow directory opens,
checks primary-versus-linked `.git` type and rejects identity drift between
samples. It therefore detects persistent independent clone/copy/mkdir additions
inside that namespace. Still open are post-return and create-then-remove races,
paths outside the fixed namespace, subprocess buffering before the4MiB
validation cap, a real temporary-Git integration test, and durable creator and
retirement receipts. This remains a cooperative detector.

## Accepted parent-seal run and transport exception — 2026-10-01

The private HPC route recovered through `tun0`. Read-only reconciliation first
proved `squeue` empty, no parent-seal accounting record, and the exact campaign,
ledger and stage absent. The operator then created only
`/public/home/majj/atlas-rust-campaign-20260930/stages/weyl-parent-seal-v1`
under the one campaign root and ran the frozen `stage_weyl_parent_seal.py`.
It submitted exactly one job, `3872554`, with `queue_before=[]`. `sacct` now
records `COMPLETED 0:0` on `cu034`, elapsed33m40s. Independent inspection
accepts the parent seal in its infrastructure-only scope: report SHA
`939482542c123a6a2a2c22f8dbe1e052af20dee3af4010da780fe958dc41bf79`
has status `WEYL_CONTEXT_PARENT_SEALED`; all12 commands/64 checker tests pass,
the two complete traces are identical, and the629/519 inventories and
33143-file/4355-unique-object closure are sealed. The seal/source CAS objects
were independently rehashed as `db67234c0d67dbd6a6f0327386dd113b6093d25a46569710c33dfc8bc482cdcb`
and `5133bb32da7e5a92775d5f56ea2035680c363b40f085a8af95eaeebfdbc4536b`.
The exact evidence summary is
`tests/reference/hpc/math_weyl_parent_seal_2026_10_01.json`. Do not create a
retry/sibling stage or resubmit the accepted parent attempt. This acceptance
does not claim new mathematics, a root-ladder repair, speed or parallelism.

Historical BEFORE launcher transition: the parent stager exits as its first
action.
Seal-bound v2 submitted exactly job3872594 with `queue_before=[]`, but failed
in its checker preflight before Cargo, inventories or mathematical regressions.
Its immutable report SHA is
`4ee3b6cd0d09c2f152a9b5f7fa8a216db0197fd291077d9800add3755fb14e2c`;
63/65 checker outcomes passed. One contextmanager globals false positive and
one incomplete synthetic SBATCH fixture are harness failures, not Rust math.
Do not mutate or resubmit v2. At that point the sole enabled successor was the
BEFORE v3 attempt, which retained the structurally unchanged v2 protocol schema.
The child predecessor/reason/retention record is frozen before creation:
parent seal `db67234c0d67dbd6a6f0327386dd113b6093d25a46569710c33dfc8bc482cdcb`,
20-file override manifest
`1bbfd0027de76a812b2ee7378b16bbcb85c88614b92d0451b0ff739a443e0d82`,
expected pin `62be23da13326b28bd2f5fe0f7cd8ae2f6fa4533b21d1164e12c017f92077138`,
and class `ACTIVE_GATE_COMPACT`. Keep compact immutable evidence; its expanded
source/target is job-temporary. The stage is not retirement-eligible until an
accepted repair successor plus CAS-only reachability audit removes every
dependency and the user separately authorizes its exact path.
The BEFORE v3 predecessor is the v2 failure; its exact changed-input manifest is
`310a2b1793f7cc2f35ffcfcb54f0aab8cc779291a391fa582dc2d85b5dc7fe7e`
and expected pin is
`ca3e22561fe493b4fe1ea32751bf772e55b62e268e631091e2ee87c0f7c6c60a`.
It inherits `ACTIVE_GATE_COMPACT`. Fresh reconciliation proved an empty queue,
no BEFORE v3 accounting row, two confirmed ledger predecessors and an absent
BEFORE v3 path;
the stager then submitted exactly job3873400 with `queue_before=[]`. It was
FINAL `COMPLETED 0:0` on cu006 in5m36s. Independent inspection accepts the
exact BEFORE evidence:65 checkers,521/630 inventories, precisely2domain+1core
named failures,0 ignored, unchanged production, all command/input/CAS hashes
rechecked and no retained ephemeral workspace. Report SHA is
`51cc7a14a0dbb8188a4ea47705338e5689212bf1f707819bab8c4e5681e4052b`;
the local inspection record is
`tests/reference/hpc/math_ladder_boundary_before_v3_2026_10_01.json`.
This releases only the minimal overflow-membership repair; it is not an AFTER
pass, performance claim or rank release. Ledger/intent/receipt hashes are
`319981ee29a86e900ca6c9536ff8a3dbab5744d2866db1150228e07baf50160e`,
`985491fa2ecc744f5ec0dfb8c08c71a49a975b4d2155bc006e2f591de63f9505`
and `4fc0b646056a02f45b265ae53b8cdd8be8a0efaa63e14b583898434ea3d75ebd`.
Do not resubmit it; an AFTER/rank gate still requires a separately pinned and
inspected repair result.

The exact raw pin is
`b52e91d827f39b7d290ab0447f82e251e979e46fdd440b61371fd5599973cf96`.
The campaign ledger, stage intent and submission receipt hashes are respectively
`97690b3555a0bc70050240a2499e7781b33b4f8e1c968356f861f24e10e1c706`,
`0c2a6b6b13404a7a4583c4f1f2c35c32d1ba07924a44b1cb12f4b05be83c5147`
and `6ced68fcf857ab4df3a5d12eb1f603e1a4257c18b4e23de11108dbb8bd7d6742`.
The fixed capture report and pin remain
`/public/home/majj/atlas-weyl-context-capture-20260930.4p1Lbsj3/results/3868832/report.json`
(`bdee12811fe6ce4160ffbc2a25df18d9a9338a7b29f22d5fcd19352cf970f832`)
and `/public/home/majj/atlas-weyl-context-capture-20260930.4p1Lbsj3/weyl-context-capture-pin.json`
(`459a5147e8546bf7e61d865cdd71e74a414db7c74d3612ba1a5893723c3cdc34`).

The bootstrap has one explicit process deviation. The local transport bytes
were already frozen, but their provenance/lifecycle registration and remote
installation evidence were not pre-frozen. The transport
`/tmp/atlas-parent-seal-overrides.lifecycle-v6.tar` is 225280 bytes
with SHA
`8347c733555b1e54c9798a6006636beb6be22e2c03483c10172625f6875d9b6a`;
its manifest SHA is
`0aac6e735c0ae7c7e0da4773c46e0d8f116b92b68a73797b05e886ea342a2620`.
It contains14 payloads plus `overrides.json`, two directories and no symlink.
It and `/tmp/atlas-parent-seal-overrides.lifecycle-v6/` were not pre-registered
with an owner and retirement condition under hard rule9. Do not backdate that
registration: this is an operator-observed lifecycle/provenance gap, not a
known byte-integrity gap. The local source tar hash was checked; separately,
the operator checked the remote member allowlist and every payload hash.
`.transport-incoming` was atomically renamed on the same filesystem to
`overrides`, and no remote tar was retained. A post-submit listing confirms no
`.transport-incoming`, no
`legacy/` and no remote tar; all14 staged `hpc` files are single-link0444
regular files. The stage's `.incoming` is empty, its ephemeral compute
workspace is absent, and the durable result remains within the same stage;
none is a new top-level experiment tree.

Both local v6 paths are now explicitly
`RETIREMENT_PENDING_EXPLICIT_AUTHORIZATION`, owned by the parent-seal run.
The unpacked sibling is frozen legacy duplication, not a second active
transport. Do not delete either path—or any of the15 clean worktree-directory
candidates—until exact active-use/evidence checks and user authorization.
The FINAL report, seal, logs, ledger/intent/receipt cross-links and final
filesystem state have now been inspected; that does not authorize deleting
either local transport path. A reviewed bootstrap helper/receipt and the known
same-UID campaign-root replacement P2
belong to a later infrastructure change; never mutate the frozen v6 attempt.

## Append-only mathematical acceptance index — 2026-10-01

The first `atlas-math-acceptance-index-v1` ledger is now prepared at
`tests/reference/hpc/math_acceptance_index_2026_10_01.json`, with a matching
compute-node-only checker in `hpc/test_math_acceptance_index.py`. The two-entry
hash chain accepts only the finite rank1 AV-ann anchor and the separately
scoped point-cycle multiplicities1/2/3/5. Both bind capture3855999
(`a83efb1b...`), independent review3856006 (`5c366f5c...`) and the actually
executed full-deform-after source manifest `6f689116...`/binary `b9580f15...`;
generic AV-ann/associated-cycle,
higher rank and interpreter-produced general cycle output remain excluded.

The ledger separates a broad `operation` from a stable `claim_id`. Independent
scopes of the same operation coexist; only entries in one claim lineage may
supersede each other, and a repeat must supersede that claim's latest entry.
`review_pending` entries may record inventory, shared failures or other
non-acceptance states only after the exact `(claim_id, status)` has a registered
contract and capture validator; arbitrary report relabelling is rejected.
Every accepted claim, including a reviewed non-pass outcome, additionally
needs an explicitly registered claim-specific reviewer. Each published suffix must add
and retain a frozen `(prefix length, chain head)` checkpoint in the checker;
otherwise it remains a draft and cannot release a gate. Only the latest
unsuperseded `accepted + math_pass` state releases a gate.

The conservative seed is intentional: KGB3850878 and block3851489 are strong
machine-readable aggregate reviews, but each is also the only local aggregate
execution report, so neither may self-review in the ledger. KLV, unitarity,
Hodge/full_deform and FPP likewise need distinct immutable review receipts.
Only `accepted + math_pass` releases a mathematical/rank gate. Existing
reports are never rewritten; promotion appends a superseding hash-chained
entry. The checker verifies exact JSON shape, no duplicate keys, repository-
relative non-symlink evidence paths, file/source/entry hashes, review-to-
capture/source cross-links, exact selected IDs and the bounded mathematical
invariants, rejects duplicate result IDs and boolean sequence counters, and
contains mutation checks for claim lineages, evidence classification,
registered acceptance, uncheckpointed drafts, checkpoint replacement and
post-seed tail mutation/truncation. No test was executed locally. The accepted
parent seal remained immutable; the checker was later included as a child-only
input in ladder AFTER v2, where all24 methods ran on the HPC compute node.
The subsequently changed checker bytes require their own later child gate and
must never be backfilled into the frozen parent transport.

## Directory lifecycle correction and ladder error — 2026-09-30

User stopped the loop after noticing excessive `atlas-rust*` directories.
Read-only audit found 25 local worktrees (~1.93GiB total at that moment; 8
secondary dirty), plus317 HPC top-level entries whose names begin `atlas-rust`,
a subset of the1174 whose names begin `atlas`; the two HPC counts overlap and
must not be added.
Recent command-after/profile targets alone were~828MiB/~416MiB. Root cause:
one permanent top-level stage per probe plus permanent copied source/Cargo
target, compounded by absolute-path parent verification and no retention
closure. No directory was deleted: dirty/unique work and immutable report
dependencies require a separate exact audit.

Fresh read-only checks on2026-10-01 confirm stop-growth but not cleanup. HPC
has318 top-level `atlas-rust*` directories plus17 regular files; the only such
directory newer than2026-09-22 is the single active
`atlas-rust-campaign-20260930` root. Locally the25 registered linked worktrees
currently occupy2476340KiB allocated in total; the24 secondary worktrees use
1193148KiB. The registry check passes with one
PRIMARY,24 LEGACY and zero ACTIVE rows. Fifteen secondary directories are
strict clean retirement candidates, eight are dirty, and one Git-clean
worktree retains ignored evidence; none is deletion-authorized. The local
guard is cooperative authorization/detection rather than command interception:
raw `git worktree add` remains possible but forbidden and becomes a stop
condition at the next check. Compute-node verification passed in job3875239;
the exact governance/evidence bytes become durable for a fresh checkout only
when the focused commit containing this handoff is pushed.
  The campaign helpers
enforce the one root and shared ledger, but generic predecessor/change-reason/
retention metadata enforcement remains open; only the one reviewed active
stager may create a sibling until that contract is centralized and tested.
The guard now also freezes the exact24 audited LEGACY `(path, HEAD, branch)`
identities. Explicit user authorization is necessary but not sufficient to
shrink this registry: a new receipt-bound retirement schema must first be
implemented and HPC-verified. A newly created worktree cannot be accepted by
adding or replacing a LEGACY row. This tests-first correction passed all22
guard tests in accepted HPC job3875239.
Historical AFTER v1 pre-submission snapshot: that child was designed to close
the narrower regression gap. It
freezes all70 `stage_*.py` names, mutation-tests the retirement predicate, and
runs a separate timed/hash-recorded validation over the parent source in the
ephemeral job workspace with current parent/BEFORE/AFTER overlays. All67
ordinary historical stagers must exit unconditionally at the first `main`
statement; none is copied into the durable stage or executed. The same job now
included20 synthetic tests for the exact worktree registry/guard, bringing its
unittest total to88. Its later FINAL harness failure and its v2 successor are
recorded below; these counts are not the current candidate.

Pre-creation AFTER lifecycle record: exact stage
`/public/home/majj/atlas-rust-campaign-20260930/stages/ladder-boundary-after-v1`
supersedes accepted tests-first BEFORE v3 job3873400 only because the production
repair and AFTER/governance harness bytes changed. It remains bound to parent
seal `db67234c0d67dbd6a6f0327386dd113b6093d25a46569710c33dfc8bc482cdcb`
/12488095 bytes and BEFORE evidence `608e996a...fc37f`. The exact27-file
override manifest is
`tests/reference/hpc/math_ladder_boundary_after_v1_overrides_2026_10_01.json`,
SHA `2b4733b6151314ffa9ac4d67d694a65414a450f89a70617c97855e18f75de9be`.
Retention is `ACTIVE_GATE_COMPACT`: durable pin/submission/report/checksum,
patches/manifests, full streams, command timing/RSS/logs and later independent
review only; source/target/reconstructed launchers are disposable. Retirement
requires FINAL inspection, acceptance-ledger disposition, no live or uncertain
job and verified CAS closure, followed by separate deletion authorization. One
cpu2/8GiB/2h job, no array or dependency, is the only authorized submission.

Historical AFTER v1 submission snapshot: a fresh check found an empty queue, no accounting or
ledger match and no stage at the exact path. The27 manifest entries were
rehash-verified remotely before the shared stager accepted exactly job3873497,
`queue_before=[]`, pin
`200055d52c0d9559a2edf4b7e556296d0c8bd0df129030e4e855e4e4426df289`.
Post-submit inspection found the job RUNNING, all27 installed inputs0444 with
one link, empty `.incoming`, intent SHA `8a620953...fc29`, receipt SHA
`cf7ce26c...efb56e` and campaign-ledger SHA `d78c75c4...5f23d`. HPC still has
exactly318 top-level `atlas-rust*` directories: this created only the registered
child under the existing campaign. At that observation its status was
`SUBMITTED_NOT_VERIFIED`; no
duplicate submission, acceptance, cleanup or rank release follows.

FINAL correction: job3873497 later ended `FAILED|1:0` after1m20s on cu006.
All88 checker outcomes and `rustc`/`cargo -vV` passed; the first
`full-stager-inventory` command then raised `ValueError: full stager filename
inventory changed`. The sealed source object contains zero historical stagers,
so its three current overlays could never reconstruct the frozen70-name set.
Cargo, both patches and all mathematical commands were NOT_REACHED. Preserve
v1 unchanged as `HARNESS_FAILURE`; evidence is
`tests/reference/hpc/math_ladder_boundary_after_v1_failure_2026_10_01.json`.
The changed-input successor was `ladder-boundary-after-v2`, using one path-free
CAS bundle for the67 retired sources. The worktree baseline correction adds
two prepared guard tests, so
the planned v2 unittest total is93. No top-level directory was added.

V2 pre-creation record, 2026-10-01T07:35:16Z: the exact target was
`/public/home/majj/atlas-rust-campaign-20260930/stages/ladder-boundary-after-v2`
and was still absent after a fresh empty-queue/path/ledger check. It supersedes
only immutable v1 job3873497 because v1 assumed the accepted Rust source object
contained historical launchers; no mathematical input or production patch
changed. The changed inputs add the independently inspected v1 failure evidence,
read the exact67 retired stagers from campaign CAS, strengthen the local-worktree
baseline guard, and harden final report publication/toolchain-path checks. The
exact28-file override manifest is
`tests/reference/hpc/math_ladder_boundary_after_v2_overrides_2026_10_01.json`,
SHA-256 `b3e4fa16cdb9e86aba36d4bdd5723401a32b0c07ef21cdce29165dd938daa699`.
The retired bundle is now verified in the existing campaign object store as
role `retired-stager-bundle`, 311925 bytes, SHA-256
`550b1330ec8806d1343d50d6ceb8488f89eb03546f6bb37c538cb5cd460e9b09`,
mode0444/single-link; both remote upload scratch and its local `/tmp` source
were removed. Static independent reviews found no remaining v2 blocker; no
test was run locally. Retention remains `ACTIVE_GATE_COMPACT`: one non-array,
dependency-free cpu2/8GiB/2h job may retain only its pin/submission/report,
patches/manifests, complete command/differential evidence and later review;
expanded source, Cargo target and reconstructed launchers are disposable.
Retirement requires FINAL independent review, acceptance-index disposition,
zero live/uncertain ownership, verified CAS closure and separate user approval.
This record authorizes no second stage or duplicate submission.

V2 submission update: after one SSH attempt timed out before connecting, two
fresh read-only reconciliations proved that no v2 stage, ledger record or job
had been created. The exact child was then created under the existing campaign;
all28 override hashes were rechecked remotely and shared submission accepted
exactly job3874203 with `queue_before=[]`. Pin, intent, receipt and
campaign-ledger SHAs are
`3f0f3c8b79e7ab7688ec329e242d18a82ac1b113eb33fa3788cfe1ca3e31455d`,
`bf13ee5fa7f96b9846a2d000018a2951b1ef7dd6ae208e977fd2b3eb4e62c6a9`,
`6c1e342a1e59f67bdfaa611acebcb9913c581ff3a06e5c9b7b8488c3127d8672`
and `44624c765f57222be339cfaa62bbdaff82f6208a93e78d3fdc7c60e7aba1835d`.
All28 installed inputs are0444/single-link regular files and `.incoming` is
empty; the HPC top-level `atlas-rust*` directory count remains318.

V2 FINAL correction: job3874203 ended `FAILED|1:0` after4m55s on cu105. Its
immutable report SHA is
`256247b2be5fd7358ccb56e87ff262c6c0e87ca782e822e2f3c96419c4b4a694`.
All93 checker tests pass, including all22 local-worktree guard tests; the70/67
retired-stager reconstruction, domain inventory521, both focused boundary
tests and the complete521-test domain suite also pass. The suite's expected
`rep_table::tests::poisoned_kl_cache_returns_a_stable_error` catch-unwind test
emits its caught panic hook under `--nocapture`; `passing_log` incorrectly
rejected the otherwise successful log solely because it contained `panicked
at`. Core inventory/focused/full and final integrity were NOT_REACHED. This is
`HARNESS_FAILURE`, neither a mathematical failure nor acceptance. Independent
artifact evidence is
`tests/reference/hpc/math_ladder_boundary_after_v2_failure_2026_10_01.json`,
SHA `e757f56ce6da55f1b96978daaaa2156b82b1bc3f83b36c2971f7a7a66be1f8f1`.
Keep v2 immutable; do not resubmit it or create an unrecorded retry directory.
Independent review selected one narrow successor design: do not weaken the
strict parser or touch Rust math/tests/goldens; remove `--nocapture` only from
full-suite argv, explicitly scrub `RUST_TEST_NOCAPTURE`, add tests that focused
still uses nocapture and full never does, and bind/tamper-test the exact v2
failure evidence. Local AFTER v3 candidate bytes and a candidate29-input override
manifest now exist for review, but no immutable pin, HPC stage or submission
existed at that checkpoint. Any AFTER v3 must remain a recorded changed-input child below the same
campaign and rerun both crates plus final integrity.

AFTER v3 pre-creation lifecycle record, 2026-10-01T08:51:40Z: the exact target
is `/public/home/majj/atlas-rust-campaign-20260930/stages/ladder-boundary-after-v3`.
The direct predecessor is immutable v2 job3874203, evidence SHA
`e757f56ce6da55f1b96978daaaa2156b82b1bc3f83b36c2971f7a7a66be1f8f1`
and report SHA
`256247b2be5fd7358ccb56e87ff262c6c0e87ca782e822e2f3c96419c4b4a694`.
The reviewed29-input manifest is
`tests/reference/hpc/math_ladder_boundary_after_v3_overrides_2026_10_01.json`,
SHA `3fe6c8c5c0a0638ae8d128313ad3bfce02bc97e3d16a69f75670a864b28695c2`.
Compared with v2, seven existing inputs changed and the v2 failure evidence was
added. The reasons are exact: full suites capture output without nocapture and
scrub inherited `RUST_TEST_NOCAPTURE`; the local guard hard-disables
ACTIVE/precreate, freezes exact24 LEGACY and double-samples Git plus no-follow
sibling/admin identity; pin/receipt publication exact-validates confirmed
records, detaches nested JSON and pins the full-stager program. Parent seal
`db67234c...2cdcb`, retired bundle `550b1330...e9b09`/311925 and both math
patches remain unchanged. Retention is `ACTIVE_GATE_COMPACT`; retirement needs
FINAL inspection, acceptance-index disposition, no live/uncertain job owner,
verified CAS closure and separate exact-path deletion authorization. Static
review found no blocker. Remote read-only reconciliation found an empty queue,
no AFTER v3 accounting row, the target absent and the five-record ledger unchanged
at SHA `44624c76...1835d`. No immutable pin, stage, intent, receipt or job
existed at this checkpoint. The only launch permitted by that record was one
cpu job, 2CPU/8GiB/2h, with no array or dependency.

AFTER v3 submission update, 2026-10-01T09:08:55Z: final static review found no
blocker; all29 manifest entries and95 checker methods agreed. A fresh read-only
reconciliation again found an empty queue, no same-name accounting row, absent
target and the unchanged five-record ledger. The exact registered child was
created under the existing campaign and the shared stager accepted only
job3875239 with `queue_before=[]`. Pin, intent, receipt and new six-record
ledger SHAs are
`7af79aa20b37d0fcee9a60d8e3c6b4dbec05417cf6817747520b6ebe3ec41f88`,
`feff9e66a0317719258db7e210325a109e263d1447790b71d4a444c33f79a0ab`,
`ac1f6f52f99de9b86b7b7bbfe56e7478809f193f1938a0c07219bc6061ee2254`
and `a1776412e7dd6633c329743b52c5cf4b1615dde8a7a2f0b4fddd002b5615a618`.
All29 installed inputs and the retained override copy are0444/single-link.
The job was RUNNING on cu105 when observed. The top-level `atlas-rust*`
directory count remains exactly318 because this created only the pre-recorded
campaign child. Status is `SUBMITTED_NOT_VERIFIED`; do not duplicate, accept,
clean up or release a mathematical/rank gate before FINAL inspection.

AFTER v3 FINAL acceptance: `sacct` records job3875239 `COMPLETED|0:0` on
cu105 in10m18s. Independent inspection accepts this exact bounded gate. All95
checkers (`5+10+6+22+7+24+21`), full-stager70/67, domain521 inventory/2
focused/521 full and core630 inventory/1 focused/630 full pass; all16 commands
exit zero and all32 log/time artifacts rehash. Report SHA is
`771fc790dd4340f408235e50c3f6eee754ebe4c850cbad36d4af4f902a24627c`;
evidence
`tests/reference/hpc/math_ladder_boundary_after_v3_2026_10_01.json` SHA is
`a459fa08117ff8a721181d349467ebdd9267e798cd25b5bcb1380996c2d15e15`.
Final source integrity passes, zero legacy paths were opened, `.incoming` is
empty and the ephemeral source/target workspace is absent. Both installed and
retained override payloads match all29 hashes. The durable child occupies only
3220KiB allocated and the top-level `atlas-rust*` directory count remains318.
This accepts only the minimal root/coroot ladder repair and cooperative
directory guard; it does not release rank, performance, KLV, unitarity, Hodge,
associated-cycle or AV-ann coverage. Acceptance-index disposition, focused
commit/push and any exact-path retirement remain separate; no directory was
deleted.

Acceptance-index disposition is now complete. Job3879103 is FINAL
`COMPLETED|0:0` on cu075 in17s. Independent inspection accepts the exact
publication/durability gate: stage/index/allowlist suites pass20/34/7 without
skips; all24 installed inputs and the exact two-directory topology match the
frozen manifest; source integrity and the retired-stager CAS object recheck;
the ephemeral workspace is absent; driver-process audited legacy-path opens
are zero; and no Cargo or Atlas command ran. Full report
`tests/reference/hpc/math_ladder_boundary_index_v1_report_2026_10_01.json`
has SHA
`43cc79e1630b5ec21a24c5aa1a7c411eafffaa9a4c6911333c8752b0dc63cf9d`;
independent evidence
`tests/reference/hpc/math_ladder_boundary_index_v1_2026_10_01.json` has SHA
`d9448e5cd53ba4c25e6a0fd91a5fdb16e3b423d48fe58531a50d7d7457afb9ad`;
the canonical24-entry override manifest has SHA
`009c2ee6b05abc5508f667a9566e15eaacbefc6154eb545e83d213cb2e5b3a06`.
The queue is empty, the shared campaign ledger contains7 records, and the HPC
home still has exactly318 top-level `atlas-rust*` directories. This operation
created only the registered child
`atlas-rust-campaign-20260930/stages/ladder-boundary-index-v1`, not a new
top-level campaign. It publishes only the bounded A1-plus-central-torus
root/coroot-ladder claim; it grants no new mathematical computation, rank,
performance, KLV, unitarity, Hodge, associated-cycle, AV-ann or cleanup
release. Both ladder BEFORE/AFTER launchers are disabled; the index launcher
is bound idempotently to the existing receipt and cannot legitimately create a
duplicate or sibling.

A follow-up local `/tmp` audit found 141 top-level `atlas*` directories using
914324 KiB allocated (~893 MiB), separate from the Git-worktree inventory. Two
reconstructible-looking original-source trees account for ~767 MiB; the rest
are mostly historical transport/package/capture/result staging. None was
deleted, because active-use and unique-evidence checks are still missing. Hard
rule 9 now also forbids using `/tmp` as an evidence tier: temporary state is
removed before handoff, with at most one exact manifest-and-hash registered
active transport allowed to survive across turns. The existing 141 paths are
frozen legacy backlog, not reusable precedent.

Refreshed local inventory on2026-10-01 resolves the apparent directory count:
there is one primary checkout plus24 secondary Git worktrees sharing the same
`.git`, not25 independent repositories. Only three names literally match
`atlas-rust*`; the other 22 `atlas-*` paths are worktrees of this same repo.
They occupy2476340KiB (~2.362GiB) of allocated disk space in total;
the24 secondary worktrees use1193148KiB (~1.138GiB). Eight
secondary worktrees contain tracked or untracked changes and are immovable.
Sixteen have neither tracked nor untracked changes, but `atlas-rust-avopt`
retains ignored evidence, so
only 15 are strict clean-directory retirement candidates (currently 644.6MiB
apparent/724.5MiB allocated).
All 24 secondary HEADs are unmerged into the
current branch, have no upstream and are absent from remote refs; 18 are kept
reachable only by their own local branch. Consequently none authorizes branch
deletion, and even clean worktree-directory retirement still requires an
active-use check and explicit user approval. The creation pattern (`audit`,
`prototype`, `repair`, `parallel`, `reserve`, `reuse`) strongly indicates that
parallel experiments used worktrees as concurrency slots without assigning
retirement to task completion; this is a fleet-level inference, not a proven
history for every individual directory. New hard rule9 now forbids that
default: agents share the
primary tree with file ownership, and only the primary coordinator may request
explicit user authorization for one registered task worktree when a
demonstrated checkout-isolation need remains after receipt support exists.
The current guard covers worktrees reported by this repository's Git metadata
and persistent `atlas*` siblings in the fixed parent namespace; it cannot
intercept raw filesystem/Git commands, see paths created elsewhere, or close
the race after it returns. It temporarily requires all24 LEGACY entries exactly.
It does not persist inode identities across separate guard invocations or
enforce an exclusive OS owner/mode policy. A same-UID pre-check rebuild with
the same path/branch/HEAD can therefore evade the historical-identity claim;
this is stop-growth workflow enforcement, not a cross-user security boundary.
A future receipt-bound creator/retirer must bind durable identities and
ownership if that stronger guarantee is required.
Even an eligible, explicitly authorized directory cannot be removed until a
new receipt-bound retirement schema records the transition; deleting a path and
its registry row together remains outside the guard's proof boundary.

Historical pre-submission description, superseded by the current-run section
above: local infrastructure has a shared campaign CAS (`campaign_source.py` and
`campaign_blob.py`), auto-cleaned job workspaces, atomic campaign-wide
submission records, and resumable per-file frozen staging. Phase 2 is locally
implemented as stage `weyl-parent-seal-v1`, schema
`atlas-weyl-context-parent-seal-v1`, role `weyl-parent-seal`; it was not yet
submitted at this historical snapshot. The migration runs the complete legacy gate twice
under the same environment, hashes the exact observed read set, cross-links
fixed reports/raw streams/source/inventories/binaries/harness/scripts, and has
64 prepared checker tests. This migration is the sole next job under
`/public/home/majj/atlas-rust-campaign-20260930/stages/`; do not submit the old
ladder BEFORE because it still follows absolute parent paths. Its launcher now
exits before parsing that parent or contacting SLURM. The successor
`ladder-boundary-before-v2` is locally implemented with a seal-only boundary:
it consumes only the parent-seal digest and size through campaign CAS and uses
dirfd-bounded immutable installation. A later protocol audit replaced its
custom submission schema with shared pinned `submit_one`, existing-lock
recovery, whole-ledger validation, exact same-job partial-confirmation repair
and a final no-heal preflight. Both stager and driver remain disabled until the
parent seal is FINAL and inspected. Its prepared 65 checker tests, 521/630
inventories and 3 expected BEFORE failures are unexecuted, not acceptance. No historical
directory is deletion-eligible or deletion-authorized. Exact policy:
docs/slices/hpc_artifact_retention_2026-09-30.md.
The final independent ladder-v2 static review found no P0/P1 issue after the
allowlist mutations were changed to target the unique `main` guard plus its
exact `SystemExit`: the actual checker-method inventory is
`5+10+6+5+24+15=65`, both submission switches remain false, and nothing was
executed. The two retained P2 limits are caller-disciplined locked cleanup and
same-UID whole-directory replacement; neither is mathematical acceptance.

A second static sweep found 29 still-executable historical stagers that called
`sbatch` directly (including 408/292/28-task arrays). They now fail at the
first statement of `main()`. The CAS storage boundary now also refuses to
create or use any durable `atlas*` root other than the existing exact active
campaign. These are local/static changes only; no HPC job was run.

Everything below this newest section that contains an operational command is
historical evidence, not an executable procedure. This includes
`git worktree add`, local Cargo/build commands, `rsync`, direct `sbatch`, fresh
or retry top-level `atlas-*` stages, copied source/target trees, and changes to
the shared `/public/home/majj/atlas-rust` checkout. Hard rule 9, the active
registry above and the retention slice supersede it.

3868832 itself is FINAL, reportbdee1281. Original accepts all11 coordinate
cases/22rows; Rust rejects6 after10rows with arithmetic overflow. Goldens are
frozen;2root-system tests plus1complete session-stream test are added before
runtime repair. The other two context histories are invalid positive drafts:
bare `matrix` needs scripts, yielding original Program/Rust Name rejections.
Their exact frozen statuses are ORIGINAL_PROVISIONAL_INTENT_NOT_CONFIRMED and
ORIGINAL_HISTORY_INCOMPLETE; do not rewrite either historical classification.
No BEFORE job, runtime fix or Weyl cache implementation has yet been accepted.

On2026-10-01 all earlier14-payload transports through lifecycle-v4 became stale;
lifecycle-v5 was rejected before transport. The current post-hardening
lifecycle-v6 manifest SHA is
`0aac6e735c0ae7c7e0da4773c46e0d8f116b92b68a73797b05e886ea342a2620`
and its deterministic transport tar SHA is
`8347c733555b1e54c9798a6006636beb6be22e2c03483c10172625f6875d9b6a`.
All14 payload entries were rehashed and byte-compared with the workspace. At
this historical snapshot those hashes proved local packaging integrity only;
the separately operator-observed remote member/hash checks are neither implied
by that comparison nor HPC acceptance. Job3872554 subsequently supplied the
sole compute attempt and is accepted only by the independent evidence recorded
above.
At2026-09-30T23:41:23Z a final read-only audit reverified both hashes, exactly
14 payload manifest entries, and the exact expected tar member set: those14
payloads plus `overrides.json` (15 regular files total) and two directories,
with no members outside that set and no symlinks. It found zero differences
among the payload manifest, tar bytes, unpacked v6 tree and current workspace,
and a
closed local project-import set. It also recounted69 canonical stagers
(67retired,1disabled ladder,1active parent) and64 parent checker methods
(`5+10+6+17+16+10`). This is static readiness under the documented controlled
no-whole-directory-rename assumption, not compute acceptance.

Directory audit follow-up disabled another 31 old `submit_one` stagers which
could copy/write before late campaign rejection. Together with the 29 direct
`sbatch` launchers and earlier retired bulk launchers, the only enabled HPC
stager is now `stage_weyl_parent_seal.py`. Keep it that way until the seal job
is FINAL and inspected.
The active campaign is now exactly `atlas-rust-campaign-20260930`, so another
date cannot obtain a separate ledger. Temporary results must resolve below an
active campaign stage and `SLURM_TMPDIR` below HPC home is rejected. Submission
opens each script component with dirfd/`O_NOFOLLOW`, rejects option-shaped
names and nested `sbatch`, and passes the validated snapshot to `sbatch` on
stdin, eliminating path re-open races. The parent stager validates the parent,
all14 regular override files/hashes and any prepared state before creating its
lock, then repeats under the lock. Locks, campaign ledger, pin, intent,
receipt, scripts and result directories now use no-follow/stable-file checks;
source/blob refs reject unknown keys and therefore cannot smuggle a legacy
path into prepared state. The raw pin SHA is recorded in ledger and intent
before `sbatch`; the exact running compute job can recover the accepted-submit
receipt windows without resubmission, and the final preflight cannot self-heal.
Test method counts are5/10/6/17/16/10=64. Two independent static reviews found
no P0/P1 blocker; compute execution remains mandatory. The residual P2 is a
hostile same-UID whole-directory rename during a lock critical section because
state I/O remains pathname-based; the controlled campaign never performs such
a rename. A fresh read-only audit makes the concrete recreation path explicit:
`campaign_source.py` and `campaign_blob.py` redundantly call
`campaign.mkdir(parents=True, exist_ok=True)` after validating the existing
root. If another same-UID process renames that root in the interval, those
calls can recreate the top-level active-campaign name and split state. Do not
change the frozen lifecycle-v6 transport for this controlled-workflow P2;
after parent-seal acceptance, replace those root `mkdir` calls with child-only
creation relative to one pinned no-follow campaign directory descriptor and
rerun the infrastructure gate. Seven documentation-linked
legacy top-level/rank6 launchers now exit before parsing, copying or submitting,
and their documentation has explicit non-executable historical banners. A
complete static inventory counts69 `stage_*.py` files: only the parent-seal
stager remains enabled and all other68 fail before old work.
These stager guards, campaign helpers and lifecycle documents are still
modified/untracked local work pending compute-node acceptance; a fresh clean
checkout does not inherit them yet. After parent-seal acceptance they require
a reviewed infrastructure commit before the fix is durable.
The five-test AST allowlist and current24-test
`test_math_acceptance_index.py` checker are now prepared in ladder-v2. The
latter brings its index plus full-deform source, cycle capture and cycle-review
JSON evidence. All five acceptance files and the allowlist are child-only and
are excluded from both copies of `SEALED_SHARED_INPUTS`, so the child does not
require the frozen parent migration manifest to contain later files. The
allowlist distinguishes the current parent-only and future ladder-active
snapshots rather than requiring ladder's switch to remain false after the
intentional transition.
Current static inventory is1active+1pending+67retired; the accurate import
claim is no external/persistent side effects, not absolute zero state change.
After the parent FINAL is inspected, close its launcher, rehash/freeze this
exact65-test child snapshot, then enable both ladder switches. Do not alter the
frozen parent transport to add any of these files.
Historical connectivity record, superseded by the recovered-route/current-run
section above: six read-only SSH attempts to10.26.14.64 timed out before producing any remote
output; the latest was2026-09-30T20:11:09Z and is recorded in
`tests/reference/hpc/hpc_connectivity_2026_10_01.json`. Therefore the queue,
campaign directory and capture are UNKNOWN—not
empty or absent—and no remote stage/submission intent/job was created. On
recovery, inspect the exact existing
`atlas-rust-campaign-20260930/stages/weyl-parent-seal-v1` path and shared ledger
before uploading; reuse it if present and never create a retry directory.
A subsequent read-only route check showed10.26.14.64 using the ordinary Wi-Fi
gateway192.168.3.1 on `wlp0s20f3`, with no HPC private-tunnel route. This
explained the pre-authentication timeout; no retry was made until the route
changed. A fresh check on2026-10-01 returned the same old route; no SSH, remote
write, directory creation or submission was attempted. The exact
2026-09-30T23:36:06Z recheck again returned
`10.26.14.64 via 192.168.3.1 dev wlp0s20f3`; queue, campaign and stage state
therefore remain unknown and the existing logical stage must be reconciled
before any future upload or submission. `ssh -G` resolves only that direct
host/port and reports no `ProxyJump` or `ProxyCommand`; there is no configured
alternate path to try while the private route is absent.
An independent read-only GitHub ref check at2026-09-30T17:21:43Z still gives
original Atlas master `7e1b958c7aa9456769cc9cf09ac1542814b4800a`, exactly the frozen oracle pin;
there is no newer upstream commit to pull into this gate.
A fresh official GitHub check at2026-09-30T19:15:00Z again resolves HEAD and
master to the same full `7e1b958c7aa9456769cc9cf09ac1542814b4800a`; receipt
`tests/reference/hpc/upstream_head_2026_10_01.json`. Thus “update original”
currently requires no source change and must not invalidate the frozen oracle.
A second fresh read-only check at2026-09-30T22:47:13Z again returned that exact
HEAD/master pair. The same receipt records the recheck; no source, oracle binary
or mathematical expectation changed.
A third fresh read-only check at2026-10-01T01:46:33Z again returned the same
HEAD/master pair; it is appended to that receipt and still requires no pull or
oracle replacement.
However, latest-revision coverage is not retroactive. Exactly347 legacy
reference metadata files still bind `4d3e9449`; the official compare from that
pin to `7e1b958c` is494 commits/257 paths, including182 scripts,71 source paths,
the interpreter CWEB/parser inputs and the operation scripts used by
unitarity/Hodge/KLV/AV-ann. The source inventory is
`tests/reference/hpc/upstream_compare_2026_10_01.json`. Preserve the old
metadata; after the current parent/ladder/performance priority, replay both
accepted and rejected language streams on HPC and append new provenance rather
than relabelling historical captures.

The static replay audit narrows those347 metadata files to346 executable
fixture/event/metadata triplets; the extra record is the aggregate
`eval/scalar_errors` metadata. Their frozen oracle boundary is200 exit-zero and
146 exit-one. Two exit-zero controls, `eval/fromfile_b10` and
`negative/unterminated_string`, deliberately contain diagnostics, so replay
classification must bind status and ordered events rather than require empty
diagnostics. Only199 cases retain both exact old raw-stream hashes; the other147
cannot support a byte-level old-versus-current claim without recovering the
pinned old streams or recording that limitation. `pipeline_swap_diff.py`
covers337 cases and omits nine event-backed cases, while
`reference_capture.py` captures streams but does not validate their event/meta
contracts; the current harnesses therefore do not certify the full corpus.
Exact replay must also fail closed until `fromfile_accepted_b10`'s embedded
legacy absolute helper path and `file_commands_b9`'s fixed `/tmp` output are
isolated and their effects captured.

Post-ladder execution is exactly two sequential jobs after the parent seal and
ladder AFTER are FINAL and inspected: first one all-346 current-original/Rust
capture-differential job with raw streams, events, status, hashes, timing/RSS
and explicit limitations; inspect it, then run a distinct independent-review
job that rehashes and binds the report, case IDs,200/146 outcomes and diagnostic
controls. Do not prequeue the review, infer acceptance from the execution
report, or mutate old metadata; append only new `7e1b958c` provenance and the
separate review decision.

The local input-freezing helper is now specified in
`docs/slices/language_corpus_transition_2026-10-01.md`. Exact reviewed bytes:
`hpc/language_corpus_manifest.py`
`48368e2acdc92fb03d41a37f8277ad279b2cfed0fed7c44486e40c3c98791877` and
`hpc/test_language_corpus_manifest.py`
`3cfa0f96a98c9df89b0bec27cfc1f606f14e4b16e5888fe67f9ec475ea3d97b5`.
It freezes353 fixtures/346 executable triplets as342 ordinary cases,345 total
no-write contracts and one exact writer. Its only legal maturity is
`FROZEN_INPUT_INVENTORY_ONLY` with `compatibility_claim=false`; no manifest
JSON was generated and neither Python module nor test ran on HPC. Independent
static review found no P0/P1. Residual P2 includes same-UID whole-tree swap,
bind-alias/final-inode hardening and the lack of campaign/ledger binding, so
the helper stays disabled and outside the frozen parent transport until parent
seal and ladder AFTER are accepted and the isolated replay stage exists.

Historical rank1 unitarity now has a separately implemented review draft,
without capture-side parser imports. Exact bytes are
`hpc/math_unitarity_forms_rank1_review.py`
`8dfd50f556fd928bf6708ed4b9b453d229b8e755e808571b2221122052ec951f`
and `hpc/test_math_unitarity_forms_rank1_review.py`
`9c2f656b6a381febe778b7c565d9409743ef75077af429c9422e8a75471d74e4`.
The independent frozen scope is6 forms,30gamma rows,46parameters,
46constituents,6empty sets and36 raw artifacts, with source/binary/parent
provenance. Static review found no P0/P1. This remains unexecuted preparation,
not a review receipt or ledger acceptance: `REVIEW_ENABLED=False` exits before
any environment/path/read/write operation. Do not flip it. Its unreachable
body still has legacy absolute reads and unbound result writes; first replace
them with a reviewed campaign-bound parent-seal CAS/ledger adapter after the
parent seal is FINAL. It certifies only historical3856011 source even then,
not the current post-ladder candidate or a higher-rank release.

The Rust repository remote was also fetched without merging at
2026-09-30T23:38Z. `origin/main` remains
`05625c5d2b5f68ffc75c66c3f5f12302d25c7c30`, an ancestor of current
`eba9c7ea080e61de9d4b105fbf54589c44a10b87`; the current
`codex/math-benchmark-suite` branch has no upstream and is 26 commits ahead of
that base. `origin/codex/continue-atlas-port` is a separate, older divergent
line (633 commits after the same base, dated through 2026-09-05), not a
fast-forward source for this branch. With 1379 current status entries, no
merge/rebase/pull was attempted; doing so would mix an unrelated historical
line into unverified work. The original-Atlas freshness receipt above, rather
than that Rust branch, remains the oracle-update evidence.

Post-seal replay is now specified, not implemented/submitted: it must load the
seal with full archival verification from CAS only, deny old top-level Atlas
opens via an early audit hook, materialize/re-hash all1558 source files in an
ephemeral workspace, verify every unique reachable object and perform zero
deletes. Seal validation now rejects every unknown top-level field, including
reference-shaped dictionaries, so replay cannot silently omit a hidden
reference; unrelated unreferenced CAS objects are not closure failures. The
seal is an observed-open closure, not a complete directory or
reverse-reference inventory, so the first replay must mark every deletion
candidate ineligible. A separate CAS-stored retirement inventory plus fresh
scheduler/intent/Git checks is required before nonempty eligibility; explicit
user authorization remains separate.

Original-source review at frozen commit`7e1b958c` confirms the ladder bug's
boundary. Original `RootSystem` builds ladder bottoms in compressed abstract
simple-root coordinates and transports them by Weyl reflections; ambient
lattice roots/coroots are created only later, so a large torus coordinate
never participates in ladder subtraction. Rust instead subtracts every pair
of ambient `i32` roots/coroots. If that exact difference is not representable
as `i32`, it cannot equal any stored `i32` root, so membership is false rather
than a construction error. The minimal later repair belongs only in
`build_ladder_bottoms`; do not alter the already-wide `i128` reflection path,
use wrapping/saturating arithmetic, or fold the separate `combine_roots` risk
into this regression without new original evidence.

The same frozen source now gives an exact ownership model for the later Weyl
reuse candidate. `atlas-types.w:977-991,1083-1099` weak-interns equal root
data, `1021-1045,1140-1152` stores one strong lazy Weyl-group pointer per live
datum and fills an empty canonical-dual cache from it without overwriting an
already initialized dual, and `2459-2465` lets each Weyl element keep the datum
alive while borrowing its group. Current Rust
`domain_builtins.rs:146-179,379-397,9386-9400` instead rebuilds
`RootSystem+WeylInterface` per context and makes the full context own the
handle.

The former one-kernel proposal is superseded. `RootSystem` is datum-coordinate
specific, but `WeylInterface` carries the original abstract group's observable
canonical-word ordering. The future model therefore needs an owner-local lazy
`Arc<DatumWeylKernel { system }>` plus a separately weak-interned canonical
`WeylGroupCell` whose successful `Arc<AbstractWeylGroup>` owns the interface.
`dual()` shares that abstract identity only when the canonical target is still
cold; a prewarmed target remains distinct. `WeylEltContext` holds
`{handle, coordinate_kernel, abstract_group}` and no cached object points back
to the handle. Structural RootDatum Eq/Debug exclude both caches, but Weyl
compatibility is abstract-group `Arc` identity—not structural handle equality
or coordinate-kernel identity. Cross-coordinate equality/product must replay
the right external-generator word in the left root system; direct permutation
comparison/composition is unsafe. Binary `=`/`!=` must become fallible and
perform the mismatch check even at no-value evaluation. Full root-owner
interning remains a later optimization, but weak canonical identity-cell
interning is required for these semantics.

Memory is not unconditionally lower: sharing helps when multiple WeylElts stay
live, while the temporary-only elliptic loop may trade ephemeral rebuilds for
one longer-lived kernel and leave peak RSS flat or slightly higher. Require
live-owner/kernel counts plus same-node allocation/RSS evidence. No runtime
edit or speed/memory claim follows from this source audit.

The replacement A1 oracle inputs now exist as
`tests/math/generics/weyl_context_core_{cold_dual,prewarmed_dual}.atlas` with
`weyl_context_core_catalog.json`. They are core-only and cover `=`, `!=`, `*`,
same/alias/fresh/rebound owners, invalid words and recovery in two fresh
processes. The accepted-intent history uses both preference/isogeny pairs to
warm each source while its canonical target is still cold, then links the
target through `dual()`; the
rejected-intent history prewarms the canonical target while the source remains
cold, calls `dual()`, and warms the source only afterward. Their
status is only `source_predicted_not_captured`; source predicts
that current Rust prints booleans where original rejects, and rejects the cold
dual product that original accepts. Do not call this a confirmed calculation
error until the new original capture runs. If confirmed, preserve it as a
regression and complete the semantic repair before any cache work-count gate.
After the A1 semantic AFTER, expand progressively to G2 (interface-order
witness), B2/C2, both operand orders, inner-class dual construction and
no-value relations. Every one of those semantic gates must pass before the
cache work-count BEFORE or any production cache edit.
Fixture SHA256 values are cold`4eff8fa08f8490e242f07282125b83cde1daf24a875fe8df4dc7e73e75a0ae99`,
prewarm`f13d704175f702b966790bf82e56c837dd80a594f0dc2dcd0d6b81ba281799a2`
and catalog`7fb77fecda841962cb98bdeddbdba58df46d673e076f87f2087cd234e2f1a717`.
`hpc/weyl_context_core_contract.py` and its static test draft pin the actual
catalog/fixture bytes, exact payload values and order, complete normalized
oracle/Rust diagnostic blocks, fresh processes, finite exact metrics and raw
stream hashes. All statuses remain capture-only/unreviewed and all release
flags are false. They have not run on HPC and are not included in job3872554.
No launcher, compute driver, stage directory or submission was created; these
bytes must later enter the campaign only through a prerequisite-bound CAS pin.

The first cache implementation must also preserve call-local diagnostics.
Store only successfully completed immutable coordinate/group objects; never
cache a `Diagnostic`, `Result` or source span. Map any retryable
`StructureError` at the current `build_weyl_context` call so a failed first
construction neither
poisons the cell nor reuses the first caller's span. Existing `FallibleOnce`
has hard-coded lazy-real-form poison text, so do not reuse it unchanged: use a
small Weyl-specific cell or generalize its invariant label explicitly. Replace
all direct `RootDatumHandle` literals with one private constructor so clones
share the coordinate cell while every genuinely new dual/quotient/derived/
integral/folded/explicit datum gets a fresh coordinate cell. Canonical identity
cells are a separate weak registry keyed by exact pre-root data.

Separate KLV-storage audit: equal polynomials are already pooled once per
table in both versions; Rust pair entries are indices, not polynomial copies.
The remaining deltas are narrower but real: `KlHashTable` duplicates every
unique coefficient vector as a `HashMap<KlPol,usize>` key; Rust indices are
8-byte `usize` versus original checked 32-bit `KLIndex`; and Rust lacks the
original `Rep_table`-wide positive/signed pools shared across related blocks.
Read paths also deep-clone coefficient vectors, and `ExtKlMatrix` retains both
dense polynomials and a condensed index/pool representation. Do not put
`Arc<KlPol>` in every pair. After the current Weyl sequence, preserve the
order measurement probes -> borrowed reads -> checked-u32 columns plus
index-only hash storage -> direct packing/cache experiments. Only that last
stage may evaluate dropping the post-condensation dense matrix or an
owner-scoped shared pool.
Full design and source lines are in REMAINING_BUILTINS; no speed/RSS claim yet.

## FINAL post-cache profile; Weyl original discovery submitted — 2026-09-30

3868803 FINAL COMPLETED0:0in6m22cu315, local+remote report
math_overload_command_profile_2026_09_30.json SHA
1d2baaddbfcb2bf09801a995ed9a6702646dd3cf9a8f396b1ddfd6d22c693915.
23checkers/6unprofiled controls/4perf captures/source+artifact integrity PASS.
All4downloaded perf-report hashes match; files at
/tmp/atlas-overload-command-profile-results.K3sRdgFv/{load_unitarity_rank1,load_av_ann_rank1}/profile-{0,1}/perf-report.txt.
Samples240/241U,306/304AV,0lost. Root ladder SELF16.25/16.18%U and12.75/
14.14%AV; build_weyl_context accounts for14.58/14.52 and11.76/12.83percentage
points of those ladder samples (nested, DO NOT add). Type::equivalent SELF
2.92/1.66%U,1.96/3.29%AV, much lower than pre-cache. Allocation functions
remain prominent, but their ownership is not yet attributed. Samples are
short diagnostic runs, not new speedup measurements. Select repeated Weyl
construction next, consistent with prior timers and exact elliptic.at caller.

HISTORICAL PRE-FINAL STATE (superseded by FINAL3868832 above): Weyl
discovery3868832 was submitted from an empty queue in
/public/home/majj/atlas-weyl-context-capture-20260930.4p1Lbsj3,
pin459a5147e8546bf7e61d865cdd71e74a414db7c74d3612ba1a5893723c3cdc34.
33checker tests/3histories/6fresh processes on accepted7aa4350d binary; no
runtime edit. Histories cover same/aliased/fresh/dual/rebound data, both
numberings, saved elements, invalid words and existing11-call i32boundary.
Semisimple rank<=1; boundary includes totalrank2 A1+torus, explicitly not
rank2 semisimple expansion. Positive intents remain provisional; store exact
original rejections/errors without counting them as mathematical passes.
Package /tmp/atlas-weyl-context-capture-package.ASoREvbM/weyl-context-capture.tgz
SHA903954cc0523b25b93023a954a7bd4b76fcca0faa8146dfd6185c50a399a05a0.
Receipt math_weyl_context_capture_submission_2026_09_30.json authoritative.
New complete profile verifier reruns parent source/artifact/stream gates before
and after capture. Collect exact FINAL before any work-count regression or
runtime fix; if the boundary exposes a math error, save its original-backed
regression before fixing. No before/after cache-work tests implemented yet.

## HISTORICAL cross-command snapshot before the final reprofile — 2026-09-30

HISTORICAL PRE-FINAL SNAPSHOT, superseded by FINAL3868803 at the preceding
section: profile3868803 was RUNNING1m24cu315, profile-release phase after
23checker tests and the independent complete629/26/72/48source/artifact
verifier pass. Stage ZUDoR41Y, pin
2ea6f04d3831eda508591eeb278ce743c53714f6c5b577c9ceb80661d82d5012.
Submitted from empty expanded queue. Receipt
math_overload_command_profile_submission_2026_09_30.json SHA
7cdc083d5014c63f84266015f0fc64941a3922ae9e1da4f5c155a0ee4c85a8b2.
No new runtime/fixture change existed at this snapshot; sampling was not yet
available. Its old collect-next instruction and live scheduler wording are
retained only as history and must not drive current work.

3868782 COMPLETED0:0in16m25cu315. Full local report
tests/reference/hpc/math_overload_command_after_r2_2026_09_30.json SHA
fbc615d23c5d111aeea83ece1fecb6c67f7b37d6c0435c6af21e062a86d787fc
matches remote. OVERLOAD_COMMAND_RETAINED_RANK1_AB_IMPROVES, correctness/
source/artifact integrity all true.46checker,13focused and ALL629core actually
PASS,26complete original histories,72retained streams and48fresh serial A/B
processes pass all full-stream/frozen-source gates. Distinct rejected stderr
and3shared Hodge errors remain preserved, not72positive math passes.
Candidate binary7aa4350d3f4dac4b250608b1fa67fe111fb5442029e32db69906b5dbe8893cf6;
before7006dcdb. Stage kzNyGVyu/pin320dcf98. Runtime patchd9d43755 and all4
local owner hashes still exact; old Rust tests/goldens unchanged.

Four-repetition same-node median seconds:
Uload3.200306->2.488593 (22.24%less), original0.258588;
Ufull3.241245->2.538780 (21.67%less,1.27669x), original0.277168;
AVload4.236254->3.140196 (25.87%less), original0.405265;
AVfinite4.307805->3.208666 (25.52%less,1.34255x), original0.425888.
Every candidate repetition faster for all4inputs. Remaining full-input Rust/
original gap9.15971x/7.53406x. Full medianRSS43940->44118KiB (+0.41%),
53636->53912KiB (+0.51%). Loading-inclusive serial only, not kernel speed,
multicore,60-600s workloads or broad mathematical coverage. Do not multiply
old cross-node ratios. Earlier pending3868782 notes below historical.

After this exact FINAL inspection, profile package618d4f0d staged in
/public/home/majj/atlas-overload-command-profile-20260930.ZUDoR41Y.
Submission receipt once present is authoritative;23checker/2loads/6unprofiled
controls/4perf expected, same-source frame-pointer build. No new runtime,
rank expansion or root-ladder boundary execution. Continue with exact profile
result, then choose next measured target; weak-context expiry/cycle finding
below remains important for any Weyl cache design.

## Cross-command R2 full regressions passed; A/B running — 2026-09-30

Revalidated3868782 oncu315: ALL629core PASS in130.14s, all13focused,
26complete histories and72retained now pass; at14m20 A/B running. Partial
snapshot in /tmp/atlas-overload-command-after-r2-results.uUQ3d7cR has5of16
benchmark rows, NOT FINAL or accepted performance. Exact unchanged candidate
hashes rechecked. Do not promote before final integrity/summary.

New concrete caller evidence: current original elliptic.at128-135 computes
exceptional_elliptic_char_polys at load time. For each type it binds ONE
rd=types[i].adjoint outside the word loop, then evaluates
W_elt(rd,w).matrix.char_poly for each word. Word counts G2/F4/E6/E7/E8 are
3/9/5/12/30, exactly the old3868525 Weyl timer counts. Local original script
SHAac764741629173b475428d27e0d565ceff962e097e1122677eccd0bd1112bb17
matches the HPC oracle script. This is elliptic.at, NOT the older unused
ellipticExceptional.at. W_classes.at includes elliptic.at. Value/DomainValue/
RootDatumHandle cloning retains Arc datum identity; W_elt currently calls
build_weyl_context each time. This is stronger static reuse evidence, not a
new measured cache hit rate or post-candidate profile.

Important design consequence: this expression retains a characteristic-
polynomial vector, NOT the temporary WeylElt. A cache containing only a Weak
WeylEltContext can expire between consecutive calls even while rd lives.
Prefer investigating a strong handle-owned immutable coordinate kernel without
a back-reference to RootDatumHandle; strong caching of the whole existing
context would cycle (context owns a handle). This historical shorthand is
superseded by the two-layer coordinate-kernel/abstract-group design above.
Preserve structural RootDatum Eq/Debug/provenance, fallible first construction,
word-validation order and budgets. No cache edit followed from this note.

## Cross-command AFTER R1 harness failure; unchanged candidate R2 — 2026-09-30

This continuation verified3868782 RUNNING3m51/7m08cu315 (before-release then
inventory), no extra job/runtime edit. Cache mutation/clone/revision source
audit found no new omission; current hashes revalidated unchanged. Historical
pre-3868832 state, now superseded by the FINAL result at the top of this file:
new source evidence narrowed a future ladder boundary because reflection uses
i128 intermediates while all-pairs ladder differences use checked i32, and it
added an UNEXECUTED provisional
root_ladder_coordinate_boundary.atlas (11independent A1+torus/dual/numbering/
32bit-boundary calls and recovery), no golden or claimed original acceptance.
At that time capture was still required; no kernel edit had been made.
Extrema-guard equivalence and lazy-getter error-timing cautions are recorded
in the performance slice. This is not another mathematical coverage pass.
Latest live9m41cu315 reaches full-core after13focusedPASS; full629 is still
running, no fresh histories/A-B acceptance. New boundary fixture SHA
dc88d6606ae855b618dcf12589ecde82edcbe482873a94bcff1001163691ec65.
If original accepts a boundary Rust rejects, it needs an original-backed
before regression and repair, not an optimization preserving a known wrong
rejection. No runtime change before the relevant evidence.

CURRENT ONLY R23868782 SUBMITTED from empty queue in kzNyGVyu,
pin320dcf984e7340c130730fd34401af1cba0c73ec1d8a5942c76d3abac5c104de.
Receipt math_overload_command_after_r2_submission_2026_09_30.json SHA
8004f3db0707970402bf317079ec64d0393cee50f698f69232d87b40c7a6c9f8.
46checker/13focused+ALL629core/26histories/72retained/48serialAB. Inspect
its exact FINAL before reprofile or further runtime edits. No after acceptance.

Latest live2m58cu315: R2's10after-checker tests PASS (including both new
deferred-global/fault-injection guards), before-release phase, only queued job.
No new fresh histories/retained/A-B yet. Future profile package refreshed to
/tmp/atlas-overload-command-profile-r2-package.Uw6NJxMh/
overload-command-profile-r2.tgz SHA
618d4f0da6064ea8f98ae06a8c4ce60216bfdac7ef53307b447ab5a86d326448.
This is NOT uploaded or submitted:23checker (14existing+9verified),2loads/
6unprofiled controls/4perf samples. It uses corrected after driver ccf74bd4;
its new9th checker resolves deferred globals in verifier/profile modules.
It inherits test_math_overload_command_after.py9ffac94b from accepted R2,
which supplies the missing-global regression helper. Require exact R2 FINAL
before staging; the old3aea9ce7 package must never be used. All runtime hashes
remain7085d231/53264489/20d47c96/5ddcf1b4 and patchd9d43755 unchanged.

3868767 FINAL FAILED1:0 after12m14cu315, report
math_overload_command_after_r1_2026_09_30.json SHA
13116add53dc1121b2a455f915d407e4b315b17dbff0c5f9e173e285ac8694c2.
44checkers,13focused and ALL629core PASS (0failed/ignored/filtered), both
releases built. Full-core log e51e292e confirms actual execution130.05s.
Driver then hit NameError: observe is not defined BEFORE the first original
history; ZERO fresh histories/retained/A-B. HARNESS_FAILURE is correct, not
mathematical failure, correctness acceptance or performance evidence.

R2 imports canonical math_suite.observe and adds two checker regressions:
resolve deferred/nested LOAD_GLOBAL names before builds, and fault-inject
the absent observe binding.46checker total; all629Rust tests, runtime hashes,
goldens,26histories/72retained/48AB requirements unchanged. d9d43755 runtime
candidate is still unaccepted. New fresh stage
/public/home/majj/atlas-overload-command-after-20260930.kzNyGVyu;
submission receipt once present is authoritative. Empty queue before staging.
Package /tmp/atlas-overload-command-after-r2-package.Oc8iqA20/
overload-command-after-r2.tgz SHA
12108da843419c4f7851bb67b0aaa34ceb63cb7d734912072ad6b1c0a224f109.
Frozen failed stage remains untouched. Reprofile package3aea9ce7 is now stale
because it embeds the pre-import driver; refresh from R2 only after FINAL.
No profiling submission or root kernel change. Earlier running3868767 notes
below are historical; do not resubmit that stage or claim its speedup.

## Cross-command BEFORE FINAL; cache candidate prepared — 2026-09-30

Continuation revalidated unchanged candidate7085d231/53264489/d9d43755;
3868767 live6m34 then8m03cu315,44checkers and before-release passed,
candidate core inventory compiling. No new runtime edits/submissions.
Prepared next reprofile ONLY after exact FINAL inspection: new
math_overload_command_verified.py rechecks629actual units/13focused,
26whole histories including rejected diagnostics,72retained and48AB raw
artifacts/metrics/source. Eight checker regressions; shared profile driver
gets optional gate/schema/status/checker parameters with defaults unchanged.
Local math_overload_command_after.gates likewise accepts optional naming;
frozen3868767 stage untouched. New wrapper/stager/sbatch expected22checker/
2unchanged load inputs/6unprofiled controls/4perf captures, separate same-source
frame-pointer build. No diagnostic timings may enter accepted speed ratios.
Prepared package /tmp/atlas-overload-command-profile-package.UqB5RmgX/
overload-command-profile.tgz SHA
3aea9ce74b58a4dc4ddd0963afbe6897450f4bac3be069bea95261bcc4f6044e.
NOT uploaded/staged/submitted; source remains identical to pending candidate.

New source-only algorithm lead: upstream rootdata.cpp238-317 constructs
ladder bottoms for simple roots then transports bitsets by Weyl reflection;
Rust root_system.rs400-447 currently checks all ordered root pairs with
ambient coordinate subtraction+membership search. L(w alpha)=w L(alpha)
follows from preservation of the root set and linearity, likewise coroots.
Do not port upstream positive-root index arithmetic into Rust's ambient-
lexicographic RootIds. Reprofile before selection; validate all roots/coroots,
non-simply-laced orbits, reducible/tori, numberings, resource/overflow behavior
against unchanged complete originals and the direct predicate before any fix.
An avoided checked subtraction can alter overflow diagnostics: investigate
with original-backed fixtures, not a guessed acceptance expansion. No root
kernel edit yet; this is independent of uncertain per-handle cache hit rate.

CURRENT ONLY AFTER3868767 SUBMITTED from empty expanded queue in
/public/home/majj/atlas-overload-command-after-20260930.8BvdICJ0,
pin0927aee7ae4df9963cc22f1ca9f5550431f7c34fa07850f3ef823694269228ed.
Receipt math_overload_command_after_submission_2026_09_30.json. Candidate
d9d43755,44checker/13focused+ALL629core/26original histories/72retained/
48serialAB. No after acceptance until exact FINAL; do not submit more jobs.
Downloaded BEFORE log5275308e matches report, preserves both actual work
assertions and four controls. Before partial/pending notes below historical.
Latest live1m11cu315: all44checker tests PASS, before-release building;
no candidate compilation/unit/differential/A-B acceptance yet. Receipt SHA
ddf5e4ef879467adf62987f5dac03a8dcaeb791970b5995faa5b58da4720173a.

Source-only next-probe refinement: old post_view_cost Weyl key is
format!("{:?}",handle.datum), i.e. equal datum VALUE, not allocation identity.
The measured30E8rebuilds do NOT prove one RootDatumHandle/Arc is reused30times.
Before choosing handle-owned caching versus interning, distinguish equal datum
values from live allocation identity/caller paths in an isolated on/off probe.
Never infer per-handle cache hit rate from that structural-value counter.

Previous turn made progress: completion FINAL accepted and original command
capture3868735 inspected; six tests-only controls submitted3868749. This turn
revalidated that live job, then inspected its FINAL COMPLETED0:0in4m27cu315.
Local and remote report math_overload_command_before_2026_09_30.json SHA
3fd3aba357287b79fbb8c1ac88afbfab8e7fcdc1ee6c7377bbf99d243e780833.
22checker tests pass;622compiled inventory, exactly2actual work failures
(both3versus1 after correct values),4semantic/full-original controls pass;
production/source/artifact integrity true. Not a mathematical result error.

Only AFTER FINAL wrote cache candidate, runtime patch
d9d43755cbf8b5ab85f60a90d3973480fb4af9789e35e5413159379d91aa3807.
typed7085d231/types53264489; new types/revision_tests.rs20d47c96 and
typed/command_cache_tests.rs5ddcf1b4. Existing typed/types test sections and
session92e9e4f1 unchanged. Seven new lifecycle guards,629total not compiled yet.
Baseline snapshot /tmp/atlas-overload-command-runtime-before.eyLvckmS is
scratch provenance; durable runtime patch + BEFORE report pin exact sources.

Per-OverloadState immutable Rc<[MergedVariant]> cache holds no Value or mutable
inference state; Analysis's old borrowed-owner guards stay intact. Successful
user replacement (including early return), insertion and removal invalidate
only that name. TypeTable's owned Arc<()> identity detaches on add/update/
reused alias slot/successful forget, retains structural Eq/Debug and Send+Sync.
Constructor/recursive installation is already within add/update's mutation
boundary. Cache retains one revision only; old identity stays alive while
cached (no bare-pointer ABA). Transactional overload clones copy real state,
start with an independent empty optional cache. No unsafe/global cache.

AFTER harness prepared44checks/13focused+ALL629core/26whole original histories/
72retained/48balanced fresh serialAB; same-node before release rebuilt from
exact tests-only BEFORE source. Preserve distinct negative stderr and all3
shared Hodge failures. New packagec894d59ef3c615c8a4c429d46ca36d8d74e2d6eb56c8dc45570253a238727bfa.
Submission receipt, once present, is authoritative. Candidate NOT accepted;
do not quote a new speed ratio or resume rank expansion before FINAL.

## FINAL lazy completion accepted; next measured hotspot — 2026-09-30

CURRENT ONLY before3868749 SUBMITTED from empty expanded queue in
/public/home/majj/atlas-overload-command-before-20260930.E1WDSwZy,
pin71b172ae848bf75507280685e5c3528daa94d68365c6632b9c796d4e70097a79.
Receipt math_overload_command_before_submission_2026_09_30.json,
package9020c34e5ac9929764eee893ea57a8f2f880b25d873ac98f600763fa8d6ee97d.
22checker/622compiled inventory/2actual workFAIL+4controlPASS expected.
No runtime cache changed; inspect this exact FINAL before implementation.

LATEST discovery3868735 FINAL COMPLETED0:0in50s cu315; local+remote report
math_overload_command_capture_2026_09_30.json SHA
9d2b560dff1b6611d4d2bc3c06292d585bb4119556ced40a6269360d2ea13d49.
28checker tests pass,2histories/4processes unchanged production/final integrity.
Positive accepted by both with entire stdout/stderr equal. Negative entire
stdout equal, original Program versus Rust Name+Type; preserve BOTH causes:
Failed to match '+' with argument type(int,bool), Undefined oc3n_dispatch.
All4original goldens copied unchanged with verified hashes to tests/math/
generics/overload_command_{history,rejected}.oracle.{stdout,stderr}.
Stdout80897d6b/0bd63540; rejected stderr523fb4fc, positive empty.

Only then added6tests (622inventory, not yet compiled): typed4 new tests
expect2actual3-versus-1 work failures and2identity/clone controls; session2
complete original-stream controls preserve diagnostic kinds/causes. Refactored
the ONE existing OverloadState literal to Default+forgotten assignment without
changing assertions; all production bytes unchanged. Tests-only patch67d5fb38,
typed2c60b1a4/session92e9e4f1. Source before snapshot
/tmp/atlas-overload-command-before.HAFULWj5 is scratch only; durable patch
hpc/patches/overload_command_reuse_tests.patch. New22checker/622inventory/
2workFAIL+4controlPASS compute gate prepared in E1WDSwZy; receipt authoritative
once submitted. No runtime cache edit or new performance claim.

Historical discovery submission3868735 was from empty expanded queue,
stage nz47e1mI, pin d959b55fe4642ece5900a97e7bc97de128f05743257f3b820952fabab6158463.
Receipt math_overload_command_capture_submission_2026_09_30.json.
Receipt SHA94ab30e1; final result above supersedes its pending status.

3868661 COMPLETED0:0 in16m46 on cu315. Downloaded full report
`tests/reference/hpc/math_completion_after_2026_09_30.json` independently
matches remote SHA14a45971fc681d919ed757858844c418d56adaed623a6e251a786cd39aee7e3e.
Status COMPLETION_RETAINED_RANK1_AB_IMPROVES; correctness/source/artifact
integrity all true. Frozen stage ci1UtZtO/pin2ec5160e, binary5df50c0a.
53checker tests,11focused and ALL616core actually pass (129.49s,0ignored/
filtered/failed). Five completion histories,19old original histories and
72retained streams verified. Rejected type draft/recovery retain distinct
stderr;3shared Hodge errors remain failures, not72positive math passes.

48fresh A/B processes (4inputs x4repetitions x3arms), all four candidate
runs faster per input. Same-node serial medians: Uload3.984148->3.195138s;
Ufull4.022098->3.232297s vsoriginal0.278607s; AVload5.493665->4.240199s;
AVfinite5.541168->4.314198s vsoriginal0.429952s. Full ratios1.24435x/
1.28440x; elapsed reductions19.64%/22.14%. Remaining original gap11.6017x/
10.0341x. Median RSS43806->43940KiB and53570->53636KiB, essentially flat.
Not kernel/multicore,60-600s or higher-rank performance; do not multiply
historical ratios measured on different nodes. All older pending3868661
notes below are historical and superseded by this FINAL.

Next2case original discovery prepared after this inspection, no runtime
cache edit: overload_command_history/rejected, package
ddf5b650a1530bcfa82b4b75498496b970c4960bbfdbae0ee8e04d662fd213b5.
Stage /public/home/majj/atlas-overload-command-capture-20260930.nz47e1mI
created from empty expanded queue; submission receipt, once present, is
authoritative. New compute-only FINAL verifier rechecks all616/24histories/
72retained/48AB artifacts before and after capture.28checker tests/4fresh
original+Rust processes expected, no build or rank expansion. Capture only
means evidence retained: provisional positives need original acceptance,
rejections need full diagnostic review. Keep changed-result replacements,
saved old closures, alias identities, forgotten/revived variants and fresh
polymorphic scopes. No bare pointer or count-only TypeTable cache key.

## Continuation source revalidation and next probe preparation — 2026-09-30

LATEST PARTIAL3868661 at10m54cu315: ALL616core actually passed in129.49s,
0failed/0ignored/0filtered. Now candidate-release;21commands completed,
0fresh captures/retained/benchmarks yet. Only one queued job. Not FINAL or
accepted A/B; collect exact job next, do not rebuild/resubmit on this status.

PARTIAL3868661 at9m16cu315:53checkers/before release/616compiled inventory
and ALL11focused tests PASS (4unchanged original-backed regressions,2old
controls,5work/ownership tests). Full616core RUNNING; no fresh differential,
retained or A/B result yet. No FINAL performance/correctness acceptance.

Previous goal turn made concrete progress: FINAL before3868646 inspected,
lazy candidate implemented and after3868661 submitted. This continuation
rehashes all6candidate owners/patchcc397f42 unchanged and confirms scheduler
RUNNING4m56cu315 (only queued task),53checks pass, before-release phase.
No after acceptance yet. Prepared separate, UNEXECUTED
generics/overload_command_catalog.json with2provisional fixtures for the
remaining measured merged-view cost: cross-command result-type replacement,
saved closures, forgotten/revived variants, changed alias names with retained
old types, failed replacement and fresh polymorphic scopes. No guessed
goldens, runtime cache or new job. Capture these only after current candidate
FINAL is inspected; they are not mathematical coverage or accepted behavior.
Prepared fixture hashes: history336b906e, rejectedb5dc7310, catalogbd2b5543.
Cross-command design must also audit the one direct OverloadState literal in
typed::tests::overload_view_reuse_cloned_analysis_checks_type_table_identity:
if adding a cache field later, refactor its initialization in the BEFORE-only
test stage (preserve all assertions), not by weakening frozen AFTER tests.
No such refactor/cache field is applied yet.

## Completion BEFORE FINAL; lazy candidate prepared — 2026-09-30

CURRENT ONLY after3868661 SUBMITTED from empty expanded queue in
/public/home/majj/atlas-completion-after-20260930.ci1UtZtO,
pin2ec5160ebc951c12862b27504b222fb51c94c09fb974802e437b5cc52f4d7767.
Receipt math_completion_after_submission_2026_09_30.json, package79239f9d.
53checker/11focused+ALL616core/24original histories/72retained/48serialAB.
No FINAL/accepted speedup yet; inspect this exact candidate before further
runtime work. All original type-valid goldens downloaded and hash-matched.
PARTIAL1m50cu315: all53checkers passed, before-release compiling; no candidate
compilation, regression or A/B result yet. Receipt SHA255db2283f2e16ac5af8766c3b5f454b9ade7a3048dab6057cbf353736ebc307.
Latest metadata3m57cu315:3868661 still RUNNING, only queued task; same phase
before-release,53checks passed,0new captures/retained/benchmarks so far. Local
source/pinned candidate unchanged and no pending transfer/exec sessions.
Source-only next-target ownership/invalidation audit appended to performance
slice (OverloadState early replacement/TypeTable reused-slot mutations;
Weyl handle back-reference cycles, equality and dual-root coordinate limits).
No second runtime patch or submission from that audit.

3868646 COMPLETED0:0in5m14cu315, downloaded/remote hash matched:
631f71eadb5ea4a8ce371604c87e34ea656435af735a731e5806f604fa8fc810.
31checker PASS,611compiled inventory,4actual whole-stream assertion FAIL
with4ready markers and correct diagnostics,2old controls PASS, source and
final integrity true. before.log94978e39 downloaded and inspected. Corrected
constructor accepted by both, full original nonempty aliases/constructor/
fields differ from empty Rust; original stdouthashf142db1a, stderr empty.

Only AFTER inspecting FINAL implemented candidate in4existing owners plus
2new files: frames/completions.rs, typed/completion_tests.rs. Four old test
sections unchanged;5new work/ownership tests (616total) require zero eager
snapshot and stable reuse when only inactive names or values change. Session
lexers, including redirect/include boundary tokens, intern at consumption time;
no whole-file pretokenization. Rc<str> shares index/order names, OnceCell<Vec>
materializes only on query and is invalidated only on visibility changes.
Per-command and per-polymorphic-binding full refresh removed. Forget checks
surviving variable/type/overload and only56keyword/primitive names permanent.
Type publication reads actual tables on BOTH success/failure, preserving
grouped rollback versus simple-alias partial publication. No unsafe/global cache.

Candidate typed4b422d9e/frames8c4f4c56/sessiond2d008e4/frame19889fec;
newfiles5891eff8/76321095; runtimepatchcc397f429d1294ec59aa1ca04ae8de8f84bb114acb55f55746eee62046b5b4cd.
After gate53checker/11focused+ALL616core/5completion+19old original histories/
72retained/48fresh same-node serialAB prepared in ci1UtZtO. Package79239f9d;
submission receipt, once present, is authoritative. Candidate NOT accepted;
no new speed ratio. Rejected original type draft and distinct rejection stderr
remain preserved, not full diagnostic-equality passes. Before optional gate
naming added with unchanged defaults, no frozen parent edited. Local source
snapshot /tmp/atlas-completion-runtime-before.f6HPPQJY is for patch provenance,
not durable project state; patches/reports/goldens are retained in repository.

## Completion FINAL inspected; four before regressions prepared — 2026-09-30

CURRENT ONLY before3868646 SUBMITTED from empty expanded queue in6WMD97z2,
pin733ef33606def2ae441f5cd49ad37fd0ecfc6ff30f0a430498ec5dfdb9d57f26,
package237d6d44db5dce83429d524968b6be54fad182ae9ed98c9a38ad3cb660f42b7c.
Receipt math_completion_before_submission_2026_09_30.json;31checkers/
611inventory/expected4streamFAIL+2controls/one fresh2arm constructor probe.
Not FINAL; production unchanged. Initial staging command omitted PYTHONPATH
and failed importing stage_merged_math BEFORE any stage mutation or sbatch;
verified no pin/intent existed, then supplied frozen parent/hpc path and
submitted once. No checker/build ran on login, no frozen parent changed.

3868572 COMPLETED0:0in3m34cu315; downloaded report SHA
2bd59d1d559462e0185e7e0b5b77bef99007f121c9eaf56456f220f7e7269553,
23checks/4histories/8processes/integrity/unchanged production. Both accepted
history and visibility differ: forgotten succ remains listed; lexical local-
first ordering is replaced by definition order. Rejected-initializer revival
also orders incorrectly (preserve Program+Type versus Runtime+Type categories).
Types draft is rejected by BOTH for >= token/missing !; accepted survivors
still prove missing aliases/fields. Keep invalid input; valid constructor
companion uses `set_type cp3t_Row<T> = [T] !`, original acceptance pending.

All8original streams were individually hash-checked and copied unchanged to
tests/math/generics/completion_incremental_*.oracle.{stdout,stderr}. Four
whole-session tests added ONLY in session.rs cfg(test); session774b429a,
patchb39a7419. Production typedabf07725/framesc00afb1c unchanged. Before
gate prepared:31checkers,611compiled inventory,4actual stream failures after
verified setup,2existing controls,1fresh valid-constructor companion (2arms).
No implementation or new performance acceptance. Stage6WMD97z2 prepared;
submission receipt is authoritative once present. Frozen parents untouched.
Earlier RUNNING/pending3868572 notes below are superseded by this FINAL.

## FINAL targeted costs inspected; completion is next — 2026-09-30

CURRENT ONLY discovery3868572 SUBMITTED from empty expanded queue in
/public/home/majj/atlas-completion-capture-20260930.BKVqvaZu,
pinaae5e5972916a5e2cdfc296705ee77e5e7deffadfa21d9b8743f348c9f113dd7,
package227a20b70262618fd13358ead677fcdae896241b3552d3dd52d35e5b04e25a8f.
Receipt math_completion_capture_submission_2026_09_30.json.23checkers/
4histories/8original+Rust processes, no runtime changes. This is capture,
not yet original acceptance or a regression-before/after gate. Inspect exact
result; do not implement against guessed completion output or submit more jobs.
PARTIAL2m43cu315: exact3868572 remains RUNNING and is the only expanded
queued task. No FINAL capture inspected yet. Receipt SHA
cb87eb9ab0452e490dbc2cb2425e6811c9fb2bf5afb50919af9b1655567af9d7.
Core typedabf07725/framesc00afb1c/sessione519111c remain unchanged;
no runtime patch in this turn.

Transport recovered without rerunning the job. FINAL3868525 was downloaded
via rsync and its SHA0855b34b independently matched.21checks/4complete original
controls/16on-off observations/source+artifact integrity PASS. Diagnostic
binarybcbcd22e, unchanged accepted source plus isolated instrumentation only.
Loading U:3010refreshes/3,441,892names copied/0queries,0.7416-0.7436s.
Loading AV:3658refreshes/4,900,561names copied/0queries,1.1776-1.1778s.
Full U/AV refresh0.7470-0.7498s/1.1743-1.1795s, also0queries. New-name
tracking only4.7-7.1ms, so do NOT present its linear scan as a major bottleneck.
Merged views0.8352-0.8378s U/1.2742-1.2758s AV; Weyl0.386-0.388s includes
ladder work, not additive. Same E8datum reconstructed30times,~0.346s, in each
load. On/off overhead0.60-1.37%; no accepted speed ratio from these probes.
Chosen next repair: avoid unused full completion snapshots, with original
semantics first. Four-case23checker/8process capture is being staged after
this inspection; no runtime implementation edit or new acceptance yet.

3868525 is authoritatively COMPLETED0:0in6m33cu033, remote log reports
POST_VIEW_COST_PROBE_CAPTURED and final report SHA
0855b34bbeda9a1dce1f1ac408f1287fc8e0e075285e24c2aa5f85e12e4cf54c.
Historical transport failure: scp twice closed, legacy scp also failed, then
SSH/rsync timed out connecting to port22. Recovered rsync verified the same
FINAL; never resubmit3868525. No runtime source changed.

Prepared completion_incremental_catalog.json isolates4new provisional fixtures:
binding history, parallel/sequential visibility including lexical first-use,
type/member names and rejected-initializer recovery. New capture checker10
plus inherited protocol13,8fresh original/accepted-binary processes, no build
or runtime patch. math_post_view_cost_probe.gates adds optional naming args,
old defaults and all parent checks unchanged. New capture stager requires the
INSPECTED FINAL0855b34b; that prerequisite is now satisfied.
All original rejections/mismatches stay discovery, never category-only PASS.

Source audit identifies TWO probable pre-existing completion errors to capture:
original global.w168 present() includes type definitions, and global.w1504
publishes them. Rust execute_set_type does not note type OR field names and
refresh checks only globals/user overloads. Also original lexer.w468 interns
ALL identifier tokens; buffer.w1174 iterates that hash order. Rust records
first successful DEFINITION order. A local/failed first use followed by later
definitions can distinguish them. Do not design incremental completion around
the mistaken assumption that first definition is original order. Both new
histories are unexecuted; capture full original outputs before regression/fix.
See performance slice and tests/math/README for the prepared discovery index.

## FINAL post-view profile; isolated targeted timers prepared — 2026-09-30

HISTORICAL focused diagnostic3868525 SUBMITTED from empty expanded queue in
/public/home/majj/atlas-post-view-cost-probe-20260930.87kvHVtK,
pindd225daf4d3a27d5c569f1e82015d8cf60eb98d4f001fad9eb9efd225c43e020.
Receipt math_post_view_cost_probe_submission_2026_09_30.json, package
d51196dec7c8b1f7da14cc401a79ba361b9e9191d776373e794b5553e7acb8b2.
21checks/4unchanged inputs/8controls/16on-off processes; no result accepted.
Collect exact job before a new runtime change. Core typedabf07725 unchanged.
PARTIAL1m35:3868525 RUNNING and the only expanded queued task; isolated patch
applied to all7files. No probe results/FINAL inspected yet. Receipt SHA
0fec31076b21df7e6cb8ef19f5dfaf4f5c37de513ef1cc9eea94ed3f08d660b0.
Inherited profile stage lacked loading_cost_protocol.py: detected via source
inventory BEFORE submission, explicitly included its unchanged dependency;
all9overrides pinned. Generic probe runner now takes optional gate/source/
protocol/checker parameters, old defaults unchanged. Frozen parents untouched.

Source-only follow-up: original atlas-types.w977-1145 interns immutable root
data through weak references and owns a lazy W_ptr; Rust RootDatumHandle owns
Arc<BasedRootDatum>, but each build_weyl_context enumerates RootSystem anew.
Any future sharing must preserve handle provenance/numbering while reusing
only appropriate immutable kernels; datum-only keys do not encode lie-type,
isogeny or prefers_coroots metadata. Probe per-datum counts first. Completion
candidate design/semantic gate checklist appended to performance slice; no
completion/Weyl cache has been implemented or mathematically accepted.

LATEST3868496 FINAL24ab240db0b061e641a58d1fd502805c4d8eb0b261aa42feaa5e1d3203b67b62,
COMPLETED0:0in6m24cu033.22checks/6unprofiled controls/4perf captures/final
source+artifact integrity pass; same90accepted files, diagnostic84b2311d.
Four reports independently downloaded/rehashed. Self samples: Type::equivalent
11.25-12.73%, ladder bottoms7.25-10.55%, completion refresh7.25-9.60%.
Equivalent still comes from merged_variants; ladder mainly build_weyl_context,
NOT evidence selecting eager dual classification. libc allocations partly
unresolved; no measured case yet for prioritizing TypeScheme/Value clones.
Full findings/raw hashes/source boundaries in loading_performance_rank1 slice.

Historical preparation: queue confirmed empty after FINAL. Next independent post_view_cost_probe patch
e09e5fb0 prepared only:21checker tests,4unchanged load/full inputs,8original/
baseline controls,16balanced diagnostic on/off processes. Timers count copied
completion names, newly recorded names, completion queries, per-function
merged views, complete-datum Weyl builds and size-aggregated ladder pairs.
No accepted-runtime edits or new acceleration claims. All source builds/tests
remain compute-only; never sum nested timers. Submission receipt, once present,
supersedes this prepared note. Older pending3868496 notes below are historical.

## FINAL overload-view optimization; next same-source profile — 2026-09-30

CURRENT ONLY profile3868496 SUBMITTED from empty queue in
/public/home/majj/atlas-overload-view-profile-20260930.cKAa5VP7,
pin4da5e11b09f10152a4cb8a3231e531a9fbf6bb3bfcbfb051532b5824d08c8592.
Receipt math_overload_view_profile_submission_2026_09_30.json. Exact inspected
FINAL3868418/e6258384 parent, identical runtime source,22checker tests,
2loading inputs/6unprofiled controls/4perf captures. Diagnostic frame-pointer
build/times are NOT accepted speed ratios. No new profile result yet; inspect
this job before choosing a next runtime optimization or submitting anything.

PARTIAL3m34cu033 confirms same3868496 RUNNING:22checks/compiler version PASS,
diagnostic release compiling, no controls/samples yet. Source audit only:
original unary-minus folding runs AFTER selected builtin resolution and
defers folded runtime errors; never bypass overloads for negative literals.
Rust owned tuple/list/conversion temporaries still clone into FnOnce helpers;
distinguish these from necessary borrowed/global-value clones if sampling
points there. Detailed boundaries in performance slice, no runtime edits.

LATEST3868418 FINALe625838494340f48acc1dad3ed2b452aacc778d7b333bc47808bab3ed0304cb0,
COMPLETED0:0in17m21cu033, OVERLOAD_VIEW_RETAINED_RANK1_AB_IMPROVES.
All46checkers/6focused+ALL607core actualexecution/19original histories/
72retained streams/48ABprocesses/finalintegrity PASS. Local typedabf07725
and5Cartan owner hashes match their accepted candidates. Binaryf1add16c.
Same-node median fullU4.630245->4.013848s (1.15357x), finiteAV8.737303->
5.567626s (1.56931x); all4repetitions improve eachinput. RSS43818->43812KiB
and53478->53556KiB: essentially unchanged. Still13.9034x/12.1786x original
loading-inclusive walltime, not kernel/multicore/general math acceptance.
Retained56includes3shared Hodge errors; rejected category differences remain.
Full report is tests/reference/hpc/math_overload_view_after_r2_2026_09_30.json.

Fresh profile stage /public/home/majj/atlas-overload-view-profile-20260930.cKAa5VP7
created after inspected FINAL and empty queue; package79d9a252 being copied,
not submitted at this note. No runtime changes. It must inherit exact accepted
source/binary/report and run22checks/6unprofiled controls/4perf captures.
Do not use old3868400 failure or a partial3868418hash as its parent.
The pending/live notes below are historical, superseded by this FINAL.

CURRENT ONLY after R23868418 SUBMITTED from empty queue in
/public/home/majj/atlas-overload-view-after-20260930.tYeSNj3C,
pinde34e3d49cbece9d0e90669431e723931a923f0884bb22662488546184a92d1c.
Receipt math_overload_view_after_r2_submission_2026_09_30.json. Same full
gate46checkers/6focused+ALL607core/19fresh histories/72retained/48AB.
Source typedabf07725, runtimepatch55ce3afb; no after acceptance yet.
Collect this exact job, not failed3868400. Profile package79d9a252 remains
prepared ONLY; launch it only after inspecting THIS after FINAL and hash.

PARTIAL R23868418 at9m56cu033:46checkers/rustc/cargo, fresh before-release
(246.33s), candidate inventory compilation(197.91s) and six focused after
tests PASS; exact607inventory. Three work witnesses are each1 as required.
Full607core is RUNNING, not yet accepted; zero captures/retained/AB at this
snapshot. Lifetime compilation failure is resolved, but no FINAL math or
cache performance claim. Same only job, no extra submission.

Later PARTIAL10m47: ALL607core actually executed and passed (130.46s),
candidate-release RUNNING. Still no FINAL/captures/A-B at that snapshot.
Additional original trace in performance slice: global.w stores overload
func_type+combined degree once; axis.w matches it and clears trial assignments
per variant. Original matches skips zero displacement/zero degree shifts;
Rust rebuilds whole schemes per use and unconditionally shifts each trial.
These are concrete future probe boundaries, not yet measured after cache;
keep public validation and error ordering. No further runtime edit.

Index cleanup only: tests/math/README and progressive_validation now mark
3855559/3855673/3855719 as completed, and link FINAL3855872 for the repaired
four Hodge specialization assertions. Three shared original Hodge failures
remain explicitly non-passes. unitary-parameter slice links current loading
evidence rather than presenting old prepared controls as still unexecuted.
No new mathematical execution or coverage follows from this documentation.

Source audit while R2runs: eager dual classification is a possible later
target, NOT measured or selected. Original interpreter atlas-types.w3320
explicit dual() DOES call build(dual_datum,delta), potentially a second full
construction; innerclass.cpp403 DualTag copy constructor is Fokko-only.
Original metadata instead uses paired fibers/dual incidence inside its first
InnerClass. Rust domain_builtins.rs2003 eagerly classifies primal AND dual
before returning even metadata. Actual dual IDs/order are consumed downstream;
do not substitute counts/reverse numbering. New profile must establish its
remaining cost after Cartan/view changes. Full trace/boundaries are in
loading_performance_rank1 slice, "Further source audit"; no runtime edit/job.

UPDATE3868400 FAILED1:0in10m20cu033, before ANY after units/captures/A-B.
46checkers and fresh before-release PASS; candidate core compilation rejects
MultiAssignmentThreader::new at typed.rs5053. Cache makes Analysis<'tables>
invariant; the old threader tied its short expression/Analysis borrow to the
same table lifetime. Failure report391933d84105defd5b425828d00dfe4084d12a272accf64ac40e578420bbbf46,
inventory logfcd8b4b928d6122f8033c3b2ec920284f6651951da0d1212989360e956774a34.
No mathematical failure/result or new speedup. Frozen stage unchanged.
R2separates MultiAssignmentThreader<'a,'tables> and &'a Analysis<'tables>,
without changing algorithms/tests. Current typedabf07725/runtimepatch55ce3afb;
before tests56771ab3 remain unchanged. New after helper5c6671a8 only adds
optional gate args (same checks/counts/runtime flow); stager4cb6f1e4 unchanged.
R2package being staged, NOT submitted at this note. Do not run profile until
a new after FINAL passes; prepared profile package takes dynamic parent args.
Fresh R2stage /public/home/majj/atlas-overload-view-after-20260930.tYeSNj3C;
package /tmp/atlas-view-after-r2-package.8lyM4q3o/overload-view-after-r2.tgz
SHA cc96e6d17b0312d6c9e6772f62a4b10acdec00aacfc75b577511bc4ca2182f00.
It derives from FINAL before3868329 (4a9ad6f9), not the failed runtime tree;
all6test bytes,3original view fixtures,16Cartan histories and72retained
streams stay unchanged. All607core execution remains a REQUIRED after gate.

PARTIAL3868400 at7m23cu033:46checkers/rustc/cargo and before release PASS;
candidate core inventory compiling, no after units or mathematical observations
yet. Same live job, no extra submission. Prepared follow-up diagnostic only:
math_overload_view_profile.py b2ef03ec,8checks c74abd48, stagercd51fb03,
sbatch9471e33c. Current after helper5c6671a8 adds only optional gate arguments;
frozen3868400 still18d91d97. Reprofile only an inspected FINAL correctness/
integrity/measurement report:22checks,2unchanged load inputs,4original/baseline
timings +2unprofiled frame-pointer controls with BOTH complete streams equal,
then4perf captures (2perinput). Same source, no runtime patch or accepted
ratio from diagnostic timings. Nothing staged or submitted for it yet.
Prepared local profile package
/tmp/atlas-view-profile-package.aqU3Ag6p/overload-view-profile.tgz SHA
79d9a252af75653e38fee3a71cd52e5bb93bb9391475c2dafc7705f747f570eb;
stager takes a fresh atlas-overload-view-profile-20260930.XXXXXXXX directory,
exact after report path and INSPECTED FINAL SHA (not known yet). Do not
prequeue it, use a partial report hash or mutate submitted3868400.

CURRENT ONLY after3868400 SUBMITTED from empty queue in
/public/home/majj/atlas-overload-view-after-20260930.skFysMpK,
pincf4b3b00a4b5bccc0c0401795c605487b40ff05da985b55444bf6be3601a3fcb.
Receipt math_overload_view_after_submission_2026_09_30.json; package
17be6ee2a530da2e822ea90dcceb6d5bbb8b263c51316beafb81bdd3af22de3a.
46checkers/6focused thenALL607core/19fresh original histories/72retained
streams/48A-Bprocesses. No after units, mathematical or performance result
accepted yet. Collect exact job; no duplicate/extra job or rank escalation.
Main Cartan5owners match FINAL3868252; core typed19c36c95 is pending.
After FINAL acceptance, remeasure remaining loading costs on that exact
source before choosing further changes; old probes refer to pre-Cartan code.

UPDATE before R23868329 FINAL4a9ad6f9114096325f9c057457e6b21ffcaa2c2863d1fae57718a254b7a7291b,
COMPLETED0:0in4m39cu004.18checkerPASS/607compiled inventory, exact3work
FAIL(2/4/3builds)+3semanticPASS,3original histories and final source/artifact
integrity PASS. Positive complete stdout/stderr match; both rejected full
stdout/causes/survivors match, diagnostic envelopes/categories still differ.
Only after inspecting FINAL applied tests925c59fd unchanged to main and
implemented typed19c36c95 candidate (runtimepatchd99f50b6): Analysis-local
Rc<RefCell<...>> holds Rc<[MergedVariant]> unshifted views, guarded by both
borrowed TypeTable/OverloadState identities; all lexical child constructors
share it. Mutation/report paths still call merged_variants normally; fresh
schemes/inference/error priority remain unchanged. Tests retain56771ab3
baseline bytes. No unsafe or cross-command cache. NOT after-verified yet.
After harness18d91d97/6checks0fbae0de/stager4cb6f1e4 requires46checkers,
6focused thenALL607core,19fresh original histories/72retainedstreams/48AB.

HISTORICAL before R23868329 SUBMITTED from empty queue inbXdYQHLR,
pin72384af6ad749620273e53a8e80e7bde024857524db4cf111e3580265375fc95.
18checkers/3originalhistories/607inventory/6units expected3workFAIL3semantic
PASS, source unchanged. Receipt math_overload_view_before_r2_submission.
Next after helpers prepared, NOT submitted: require inspected R2FINAL;
46checkers, unchanged6first thenfull607core,19freshoriginal histories,
72retained streams and48same-node A/Bprocesses. Before helper now accepts
optional gate args for reuse; R2frozen97917106 untouched. No runtime cache.

UPDATE3868323 FAILED1:0in18scu004 after16checkerPASS, before any Rust tests
or runtime edits. Provisional history incorrectly assumed null([row]) is a
bare generic builtin; BOTH engines reject it at [int] and rigid[A], with
complete stdout equal (4fbca760), originalstderr7c6ab1a3. Failure report
2fdffdcd2d613beebd1c9d0a05cfca43e4b45c9682850eea87ba1e08f993e3db.
Exact old fixture8070021c retained as overload_view_reuse_row_null_rejected.
Valid positive now uses #([row]) across int/bool/string/nested-bool and
rigid/independent floors, as original startup global.w defines. Production
unchanged. New tests-onlypatch925c59fd -> prospective typed56771ab3; driver
97917106/protocole1cf65f9/stager06a4fbe8,18checkers/3originalhistories.
Fresh R2bXdYQHLR created after empty queue, package201aab516b806fc9581f6835eb06ec2c964c3d797b49538b5b909d14af0e6048;
not yet submitted at this note. No cache or claimed before-test result.

HISTORICAL only3868323 SUBMITTED from empty queue in
/public/home/majj/atlas-overload-view-before-20260930.Gwpfq3t3,
pin8ee9498670350038ef413b4b1f0fa0e991db14055933b7ba8f676e25137919ed.
Receipt math_overload_view_before_submission_2026_09_30.json. Tests-only
607inventory/6selected (expected3workFAIL/3semanticPASS),16checkers and
2fresh original histories. No runtime cache or accepted new result yet.
Collect this exact job; do not resubmit or mix changes before its FINAL gate.

LATEST3868252 COMPLETED0:0 in19m25cu004, CARTAN_PARTITION_RANK1_AB_IMPROVES.
Report tests/reference/hpc/math_cartan_partition_after_2026_09_30.json SHA
2702e8aef6b2671a58171027226ef2f57c65221aaaad5b793f6c1d5d003c66ff.
Binary43ae2fb62afa8f6129a4021148fe7ac67e87cc03e4bd294e868882f96871683f.
All42checkers,519compiled inventory (NOT519run),34distinct selected domain
units,16fresh Cartan histories,72complete retained streams and48ABprocesses
pass, final source/artifact integrity true.56retained regressions include
three shared Hodge failures, not56positive mathematical acceptances.
Same-node four-round median full U12.111728->4.956614s (2.44355x), full
finite AV16.789980->9.481124s (1.77089x), peak RSS266224->43820KiB and
266266->53480KiB. Original.311460/.493732s: Rust still15.914x/19.203x time.
Loads U12.348804->4.955048s, AV16.748178->9.403655s. These are serial,
loading-inclusive controls, not kernel or multicore performance, and not
general associated-cycle/mathematical acceptance. Prior pending notes below
are historical. Keep next language-view change separate until its before gate.

Prepared next gate: math_overload_view_before.py379a324c, stager7832a247,
protocolcbeadffa +8checksaf9b0b3c, before8checks24ab2054, tests-onlypatch
73c9ead3, prospective typedc1ca0159. Main typed30b24cae unchanged. New
math_cartan_partition_after.py0f5ba113 changes ONLY optional gate args,
not frozen3868252helper8081cd4f.16checker/2freshoriginalhistories/607inventory/
6selected gates require3workFAIL +3semanticPASS and unchanged production.
Packageeeb35ea54004e34644225d5a1d8f55e88069ec0ba42c7dada9f4da053ef4704c;
fresh stageGwpfq3t3 created, not yet submitted at this note. No runtime cache.

## Historical cost attribution and candidate progress

PARTIAL after3868252 at8m51 oncu004: all42checkers, before release,
519domain inventory, unchanged4rank1 units AND existing Cartan/dual/
real-Weyl/order selections PASS. Candidate release phase; no complete
CLI stream or A/B observation yet. Keep same job; not FINAL acceptance.

Follow-on tests-only refinement (still NOT applied to main or submitted):
view patch now73c9ead3, typed.tests.rs c1ca0159, superseding8e199487/e7ea4629.
The type-table identity control now changes a named forgotten-signature
expansion bool->int and uses succ@int (bypasses call-head presence), so
stale cached membership is observable. Merely changing a named argument's
expansion would not reliably detect this: unshifted cached signatures can
still expand dynamically at use time. Keep full original capture prerequisite.

Follow-on preparation only while3868252 builds: added provisional
tests/math/generics/overload_view_reuse_{history,rejected}.atlas and
hpc/patches/overload_view_reuse_tests.patch8e199487. No runtime cache or
main typed.rs edit (still30b24cae), no extra submission. Six prospective
units live only in the patch/temp typed.tests.rs e7ea4629: three work
assertions (nested calls, repeated negative literals, bare/cast/call), three
semantic controls (overload identity, type-table identity, polymorphic
trials/scopes/history). Expected607compiled/3workFAIL+3semanticPASS is
UNVERIFIED. Original must accept new positives and characterize singleton
overload rejection before any after-cache change; do not assume original
succ(bool) uses Rust's generic failed-match wording. Capture all survivors.
Temporary source /tmp/atlas-view-reuse.M1ICiGlr/typed.tests.rs. Patch and
fixtures are permanent; this does not change frozen Cartan after scope.

CURRENT after3868252 SUBMITTED as ONLY job from empty queue, stage
/public/home/majj/atlas-cartan-partition-after-20260930.vL6gPF4k,
pinba114c5d115eab984e84d5c4b76b5d5e5164b7ac44f561743227be449893364a.
Receipt math_cartan_partition_after_submission_2026_09_30.json. Collect
this exact candidate; do not submit a duplicate or combine an overload-cache
change before acceptance. Tests/goldens unchanged,42checkers/519inventory,
4rank1 units before existing related-unit controls,16Cartan histories,
72retained streams,48A/B processes. No after acceptance or speedup yet.
PARTIAL at2m02: first28checker tests PASS, remaining14ABchecker phase;
no Rust compilation, after units or math observation accepted yet. Later
collect same3868252. Secondary-source audit confirms AV unary-minus merges
~2.55s/5045calls and e8_gap.at8 analysis~1.91-1.93s. Analysis-local reuse
design is recorded in the performance slice, not implemented or mixed in.

UPDATE before R23868242 FINAL PASS, reporte57c69a60f0e12f35d4dbcf9867ef0e7a9727395a8bd124100c5130ed816a57a:
10checkers,519compiled domain inventory, exact2workFAIL/2semanticPASS and
source/artifact integrity. Original capture16streams rehashed and compared.
Only after this gate implemented a candidate in four runtime owners:
cartan_classification babb802e, cartan_class5ba8d82d, dual640b889b,
real_weyl1a2890a0. Tests remain byte-identical; inner_class1ee32f31 retains
test-only counter. Canonical BFS keys replace eager full generated members;
exact original |W|/(|W_im||W_re||W_cx|), using existing simple_complex,
preserves direct involution count budgets. Legacy full-W enumeration remains;
root-permutation width>256 fallback retains previous rejection behavior.
Membership keeps datum/delta gates and canonicalizes missing member queries.
Dual lookup reuses real-Weyl primal-word replay, with its helper extracted;
public correspondence's previously unused budget remains unused.

After harness prepared:42checkers, unchanged4rank1 tests then existing
Cartan/dual/real-Weyl/order units; fresh before and candidate release builds;
16fresh Cartan original histories,72complete retained streams and16A/Brows
(48processes). No new mathematical ranks. Runtime patch80796a12; package
d9833ab4818b1bfb7363f9c8e9e3b2af584b30f4b825227d20e6dbbe96aac199.
Not yet submitted or verified at this note. Main candidate NOT accepted.

CURRENT before R23868242 SUBMITTED as only job from empty queue, O6fhVGS1,
pin92049ff291bfd96ddb8803af14abfa68a4f310199caff6ab9217870d5b6d7419,
package9a6bb109ce625637269c345463a90add78a8add8a0bb93a7ded1cde0e528da87.
R13868238 FAILED1:0in9s before checker/build/runtime: serialized orbit
inventory uses JSON lists, recomputation uses tuples. All original hashes
are unchanged; full comparison falsely differs at metadata type equality.
Failure report13eb3fa7fdcc84c058c0372ba62e2d2f054e20bd305accc1ae822bf3b9825fee.
R2 JSON-round-trips comparison metadata only, adds two checker tests
(round-trip and altered/reordered/missing orbit rejection), now10checkers.
Does not normalize raw output, weaken goldens or change four Rust tests.
Main now contains exact tests-only patch94bf9210 (domain519), no runtime
rewrite yet. Collect R2; mathematical or performance acceptance is pending.

3868228 FINAL COMPLETED0:0 in10m21cu145, report
math_loading_cost_probe_r2_2026_09_30.json SHA
50c7fb3045b4b96ba6e2bc590ad8fe6d0177e7b97817e132299523b26def64f6.
12checkers,4fresh original/baseline controls,16probe observations and final
source/artifact integrity PASS. Two off/on repetitions per unchanged input;
enabled overhead medians1.01472/1.01426/1.01679/1.01387, not a speedup.
Both repetitions locate partition6.11-6.19s (generation2.89-2.94s,
orbit sweep2.37-2.40s, permutations~0.655s);424189members/3364708edges.
E8 groups.at251 evaluation6.71-6.79s includes this partition work. Each
E8 primal/dual datum has199952members but only10classes. Overload merge
costs~1.44s unitarity/~4.39s AV; AV e8_gap.at8 analysis~1.91-1.93s.
These are nested clocks: never add parents to children. Parser~0.11/0.17s
and cache lookup~0.00005s are not the main targets. No production speedup.

After inspecting FINAL probes and empty queue, submitted ONLY3868238:
stage /public/home/majj/atlas-cartan-partition-before-20260930.8RzY9Nkb,
pin8574c74fc91483d0e0114a01e59cae92cee0549a0581093e66801f596c2ff510,
package0d4ef714064854efa96ad49caef47b57ec11ed9c786f566a6566993fe97f6d28.
Tests-only4rank1 kernel gates after original capture3868167, expected
2workFAIL/2semanticPASS; NOT yet verified. This is unnecessary-work evidence,
not a mathematical-error claim. Next: inspect that gate, then replace the
direct classification's eager member enumeration with canonical Cartan BFS
and exact original orbit-size quotient; preserve legacy budgets, direct
count limits, numbering/poset, dual correspondence and provenance checks.
Use existing simple_complex helpers, not a second incompatible formula.
No higher-rank survey or speculative dependent jobs.

## User requests finer cost attribution; probe first — 2026-09-30

CURRENT3868228 R2 SUBMITTED as ONE job from empty queue, stage
/public/home/majj/atlas-loading-cost-probe-20260930.OHvh0QW6,
pine6e817a91e92dcd87e23b43e29504f97c1aadf1cbbefab83d9d1de06fc684e1d,
packageb7f8f52009e822ef710952619a2c21b96b7ebd22f99c351de9ef56aae2317536.
Receipt math_loading_cost_probe_r2_submission_2026_09_30.json. Collect this
same job; do not resubmit or infer successful compilation/measurements yet.
No production optimization was made in this probe iteration.
PARTIAL R2at6m30 oncu145: diagnostic release now compiled successfully;
first load_unitarity original/baseline control and probe-off full-output
check pass. One control/one probe observation recorded; no enabled probe
summary or FINAL integrity/A-B-overhead conclusion at this observation.
Continue same3868228. Source audit meanwhile confirms cache reuse must
respect Analysis's publicly rebindable types/overloads references and that
probe source-line keys use preprocessed, not necessarily physical lines.

UPDATE3868224 FAILED1:0 after3m01 oncu029 at isolated probe compilation:
cost_probe.rs73 JSON format string lacks one escaped closing brace.
12checkers passed; ZERO mathematical controls/probe observations ran.
This is a diagnostic-build failure, not a Rust mathematical failure.
Report math_loading_cost_probe_r1_failure_2026_09_30.json SHA
9872491f8534a421d6ec908558acca0d6d39e51eaa3eacb9b2b8a023f9e36417;
raw build log1cd892283ad8b2f5334e0e8d174ed28d97d7143755aab0cc594b4cc72584884d.
Frozen yEzcdKyy is untouched. R2 changes only that probe-format brace and
its pinned hashes: patch4481c3c4, helper74277e6b, driver2ff08571.
Fresh stage OHvh0QW6 being staged; collect a new receipt before inferring
submission. No probe result, optimization or production change yet.

Historical3868224 SUBMITTED as ONE job from empty queue in yEzcdKyy, pin
c6672e65649e4f8b6ccd6048218a45e9dc4572858a51fabc514ab4e4a5acb0b8.
Receipt tests/reference/hpc/math_loading_cost_probe_submission_2026_09_30.json.
No probe result yet. Do not submit the separate Cartan before-only job or
implement an algorithm based merely on the earlier self-sample percentages.

Latest user: carefully analyze or insert probes to locate excessive costs,
then optimize the measured bottleneck. Do NOT immediately implement the
Cartan rewrite. Probe package is prepared from exact correctness-accepted
3866086 source; main production files are untouched by instrumentation.
Small-rank load/full fixtures already import groups.at which initializes
G2/F4/E6/E7/E8 at lines217-254. That is a strong source explanation to
measure, NOT proof yet of its exact runtime share or rank1 kernel cost.

New isolated loading_cost_probe.patch SHA
8547697bf07c30ac6823ce26bab7410baa2e759f3c50b6d06b622a775d3e26b8:
timers/counters for command parse/execute by source+line, set analyze/eval/
bind, overload merge by name, cache hit/miss by complete datum/twist,
classification phases, and partition setup/generation/permutations/hash/
orbit sweep/representatives. Main-thread aggregation only; serial Rayon1
configuration. No per-reflection timers. Whole parent clocks are inclusive;
child phases partition only that parent, do not sum them with the parent.
Work counts are not RSS. Source identities and matrices are retained.
LOADING_COST_PROBE_ENABLED toggles the separate diagnostic binary. Driver
requires exact complete output after removing ONLY valid probe records,
fresh original/baseline controls, twice on/off each of4unchanged inputs,
12checker tests and final hashes. Instrumentation overhead is measured;
probe timing is NOT an accepted performance ratio. No optimization yet.

Package /tmp/atlas-cost-probe.SWSJZAVm/loading-cost-probe.tgz SHA
12ae89d64b011354dd12227d1b22183c62178402a08e19828678a7dc889b5c95;
fresh remote stage /public/home/majj/atlas-loading-cost-probe-20260930.yEzcdKyy
is being staged. Read its submission receipt before inferring a job exists.
math_cartan_class_capture.py current64e1797e adds optional gate arguments
only for reuse; frozen capture helper4b835415 stays unchanged on HPC.

Cartan3868167 FINAL COMPLETED0:0 in2m16cu029, report
tests/reference/hpc/math_cartan_class_capture_2026_09_30.json SHA
b8f60647a42644f3580e97ab755faa69d613b688b45c0645333fc3212a3bafb5.
All12checkers,8positive and8negative histories PASS; exact full math tails,
all classes/forms/dual incidence, four ordered bounds errors/recovery.
Loader-prefix stdout still differs. No runtime change or speedup.
Receipt9cdb124ead07a7ae0b4f42796b4b816c39f2806d4f7035a2b682fc4f3242e8a5.

Kernel before-only helpers math_cartan_partition_before.py/sbatch/stager/
checker and hpc/patches/cartan_partition_rank1_tests.patch94bf9210 are now
permanent, but NOT built/tested/submitted/applied to main. Expected519domain
inventory/4executed:2workFAIL/2semanticPASS remains a hypothesis. Original
capture prerequisite is met; finer probes now take priority. No speculative
dependency jobs, no new ranks. Full performance gap remains unresolved.

## Presence candidate correct but ineffective; Cartan target — 2026-09-30

3866086 FINAL COMPLETED0:0 in22m29cu009. Report
`tests/reference/hpc/math_overload_presence_after_2026_09_30.json`, SHA
95d23a47706b76e6548462c4835bf23f093e327a9b461bd789e8b5dbbd5095d2.
Status PRESENCE_RETAINED_RANK1_PERFORMANCE_NOT_ACCEPTED: all correctness,
source/artifact integrity gates PASS, but the A/B improvement gate fails.
38checkers;65selected Rust units PASS;601compiled/listed, NOT all run;
2fresh original presence histories and70complete retained streams unchanged;
16A/B rows,48fresh processes. Full U10.90781->10.86117s (~0.43% faster),
finite AV14.86341->14.90503s (~0.28% slower). Still45.24x/38.30x original
time including loading; ~260MiB unchanged. Do not count a third effective
performance repair or multiply ratios across nodes. Main typed30b24cae is
correctness-accepted in this bounded scope, NOT performance-accepted.
Candidate8d14698301e524c19f2da4d0a60e3518e8c47653f3f5e1da4a8a8479df95b110;
fresh before23751c90341ec28cc7f4464005360dbdc972ff633173704426ff2614cdc0b648.

Next Cartan capture helpers are prepared (not yet verified):16inputs from
8real/complex A1 presentations (SC/adjoint, both numberings), each with
complete positive histories and a four-error bounds/recovery companion.
12checker tests. They pin the inspected FINAL presence report and keep its
performance-not-accepted status visible. No builds/runtime change/rank
expansion. Package /tmp/atlas-cartan-class-rank1.eFepAtiK/cartan-class-capture.tgz,
SHA da73f0a141230832cabcb97d302cb2010cc745a49ebd81f39506034d59367213.
Remote fresh stage k51a7mP5 is being staged; do not infer submission until
its submission.json is recorded. See performance slice for scope.

Four isolated Cartan kernel tests plus a cfg(test) partition-build counter
are prepared under /tmp/atlas-cartan-class-rank1.eFepAtiK, NOT applied to
main, built or run. Expected2actual work failures/2semantic controls, to be
checked only after original Cartan capture: exact legacy/direct budgets,
all small involution lookups, wrong datum/twist rejection and dual incidence.
No Cartan production repair yet. Preserve all existing ordering guards.

## Post-repair profile FINAL; next presence-query before gate — 2026-09-30

NOW presence after3866086 SUBMITTED as ONE job from empty queue, stage
`/public/home/majj/atlas-presence-after-20260930.87MXIyFj`, pin
f411a031c56847d44e3e78ecc9c8ea72624fda3119208097a1dd0fea4bcdbf46.
Receipt tests/reference/hpc/math_overload_presence_after_submission_2026_09_30.json,
SHA d0b9c9f64c4bc532fd7d73edb8f1089c7f3654825b96a10d59f81202d0e62e5c;
package86e06e14e982c49aa98820356d100e5b8e0b844ca0ae7db978ead69c441411e6.
PARTIAL at54s oncu009:38checker tests/compiler metadata pass, release
test-inventory build running. No after units, output retention or A/B yet.
Source30b24cae stays fixed. Do not resubmit or mark performance accepted.
Full original goldens are now permanent overload_presence_*.oracle.* files;
positive stdout410c21f6, negative stdout9600a1df/stderr85caf759.

Parallel source audit (no Cartan runtime change): old ON_DEMAND design
contains superseded budget and numbering assumptions. Its new current note
records the actual dual_cartan_correspondence consumer, reusable dual_side
Weyl-word replay and simple_complex helpers. Important: cartan_rank1 fixture
tests matrix-type recognition, NOT the full Cartan-class surface. A future
partition rewrite needs exact per-class rank1 and dual-incidence gates;
12KGB/12block histories alone must not be described as that full gate.

UPDATE3864029 FINAL COMPLETED0:0 in3m19 oncu001; report
`tests/reference/hpc/math_overload_presence_before_2026_09_30.json`, SHA
9540131c43c901e7281914070ea3ef1a0006257b0fb73848846b0d44aaef4a46.
8checkers,601compiled inventory,2actual work assertions FAIL and1semantic
control PASS as required; both complete original captures/unchanged
production/final source and artifact integrity PASS. Call-head builds2
views instead of1; type conflict builds1instead of0. Negative original
PROGRAM vs Rust NAME+PROGRAM retained; exact3causes/survivors/full stdout
match. These are work failures, not incorrect mathematical results.

Only AFTER that FINAL gate, applied the unchanged3tests and implemented
has_active_variants, used at the two presence-only call sites. It checks
user entries then any non-forgotten builtin; no cache, no overload ordering
or TypeTable change. Candidate typed SHA
30b24caea8e9fac0f8d3584c5352c49b33b64875c050323e153d34fb13f06411,
runtime patchb248916bc049f245bba5c1579f65c6a7b63ab577010951ec4b9be52c6f41b50e.
After helpers prepared:38checkers/unchanged3tests+5coercion2session55type/
56rank1+2repair+12language full streams/2fresh original histories/48A-B
processes with fresh before/candidate releases. Not after-verified yet.
Do not call the601inventory a full test run or multiply cross-node timings.

Presence before3864029 is SUBMITTED as ONE job from an empty queue, stage
`/public/home/majj/atlas-presence-before-20260930.rFLDlK8B`, pin
7dbb74b2bd3975d732338cfd2e368cdc95fe04326ce54964ea8fc75756cf7e01.
Receipt tests/reference/hpc/math_overload_presence_before_submission_2026_09_30.json;
package5705b43790b7fc3dbb19d70bc906660b84466f4b5936f12f8c6161895b2497f3.
PARTIAL at49s oncu001:8checker tests pass, both original/Rust captures
match expected values/specific rejection causes and complete stdout;
negative categories remain originalREJECTED_PROGRAM versus Rust
REJECTED_NAME+PROGRAM, not identical diagnostics. Source/tests-only patch
applied and inventory compilation running. No601/before-work acceptance yet.
Collect the same job; no duplicate submission or runtime change before FINAL.

3861850 COMPLETED0:0 in5m20 oncu009, affinity[49,53], report
`tests/reference/hpc/math_overload_profile_2026_09_30.json`, SHA
7a2235c674eb882447fa4da65e5f6774b4171000fb7697e944877c47a4cf9f85.
22checkers, two unchanged loading streams, two separate exact-source
frame-pointer profiles and final integrity PASS. No new speedup claim.
U/AV self samples: Cartan generated partition23.64/18.00%, equivalent
6.57/14.95%, validation2.72/6.05%, is_close2.25/3.19%. Stacks confirm
merged_variants remains a caller. Do not sum nested percentages.

Next bounded experiment: remove full merged-view construction used only
to ask whether any active overload exists. Two call sites: named call-head
dispatch and grouped-type name collision. No cache or runtime edit yet.
Prepared math_overload_presence_before.py and eight checker tests, the
isolated3Rust-test patch and2provisional original fixtures. Expected601
compiled inventory (NOT full test run); expected2redundant-work assertion
failures plus1semantic control pass. Original full streams must establish
local-function/nonfunction shadowing, forget/rebind, captured closure and
callee-before-argument error priority before repair. Do not call work-count
failures mathematical errors or accept compilation/setup failures instead.
The before source includes only cfg(test) instrumentation/tests and the
explicit comment correction b147208c, not a presence helper.

## Two loading repairs FINAL; continue small-rank performance — 2026-09-30

LATEST3858574 FINAL COMPLETED0:0 in24m10, cu001 affinity[61,62]. Report
`tests/reference/hpc/math_overload_after_2026_09_30.json`, SHA
ac29df12a4d7286fe7cde4bd49d182edee9551fc088196c40053b5e6b5d13783.
Status OVERLOAD_REPAIR_RETAINED_RANK1_AB_IMPROVES, correctness/source/artifact
integrity true.38checkers,5coercion/2session/55type units, both full repaired
original histories,56retained streams (53positive/3shared original Hodge
failures),12language cases and16A-Brows/48processes PASS.598compiled/listed
inventory is NOT a full598test run. Every input faster in4/4repetitions.
Full unitarity12.71649->11.20919s (1.13447x), original0.247392s =>45.31x
original time; finite AV-ann19.43028->15.14635s (1.28284x), original0.402166s
=>37.66x original time. Loading-inclusive serial metrics, NOT kernel or
multicore ratios. RSS~260MiB unchanged. No cumulative cross-node multiplier.
Frozen candidate95cc29f2628fc9bf5010194653cd8a1bbdc58f180003a8cc93c8dc7ec662518a;
fresh before027a0191b31f7375c0d0903c7f9b2d7cb3574b66402f63f46e1761a2cbefb87d.
Both stay in jTo2AfSy/results/3858574; preserve full report/raw artifacts.

After collection, corrected ONLY the overly broad coercion comment locally:
coercionsb147208cfa48cf79ecc6768120202148296adf313de5998b709ecb25e5f4579c,
comment-only patchhpc/patches/overload_proximity_comment.patch SHA
7e3176e4cc13da7e90aa5d47062749a15158b64debfa905d84a1b1ba153253a9.
Verified/runtime profile source remains frozenb32c06a6; do not silently
substitute local comment/source hashes into that report. No functional edit.
Profile staging directory (submission details immediately below):
/public/home/majj/atlas-overload-profile-20260930.aH4uDHA0, prepared package
4913af29942caa8b13fc415354898012cf1909f4c8b625060994490e2c89fa96.
Next inspect profile hotspots; keep no-cache presence tests isolated until
chosen. No new group/rank work. All earlier pending3858574 entries below
are historical and superseded by this bounded FINAL result.

NOW3861850 SUBMITTED, ONE job from empty queue, no dependencies/arrays:
`/public/home/majj/atlas-overload-profile-20260930.aH4uDHA0`, pin
f1fccec7c0bcaa0f437012230575604fce9f8037a2adf57dffbc976df61a027c.
Receipt tests/reference/hpc/math_overload_profile_submission_2026_09_30.json.
22checker tests, two unchanged load inputs/four original-vs-accepted timed
processes, two separate frame-pointer profiles. Sampling binary must keep
exact frozen598source, not local comment correction/presence tests. No
new mathematics, runtime fix or speedup claim in this job. Do not resubmit.
Collect profile text and inspect stack quality before choosing presence-only
query, analysis-local reuse or Cartan initialization as the next experiment.
3861850 PARTIAL confirmed RUNNING oncu009 at2m18: all22checker tests and
profile-rustc pass; same-source diagnostic release build is running. No
profiles/results/final integrity yet. Receipt SHA
de7f61fb1fd6ea119909e2974356decaa79b446ec2360f6e3954efddb501803c.
No pending local tool session; do not confuse a metadata-query timeout with
a stopped job. Source-only presence patch/fixtures remain unrun/unapplied.

LATEST3856744 FINAL COMPLETED0:0 in9m05; report
`tests/reference/hpc/math_overload_before_2026_09_30.json`, SHA
33e8ffff875f0905892b0a8ee0cbd061ffa77118c3ae753c96fb7fc5a2b21e55.
6checker tests,596compiled inventory, exactly2actual nullary assertion
failures and final integrity PASS. Production unchanged. Recursive original
fully ACCEPTS; unchanged Rust exits139/signal11 in24.2638s at6207724KiB,
stdout empty. This is not a timeout or accepted speed ratio. Permanent full
golden4825c1d7293dfb8749d6b3e89a7fd8dd3ed6a8ebf94175563d9f833418740847
and two recursive tests were added before runtime editing. Only the two
nullary tests ran before; recursive before evidence is the CLI failure.

Local candidate changes ONLY is_close production logic: equality before
void/recursive boundaries; do not unfold unequal recursive names; query
coercions only with a primitive endpoint; stop tuple comparison at zero.
coercionsb32c06a65bf0d4aa4f5e5c2301e8eea09e69e36e6ec2a7f4ab45be7d7ead070e;
sessione519111cc78f3e6178224e1b3eb8ea11d31a727d8556bb1e10c0756d0a4eafb1.
Expected598compiled tests, not yet run. Runtime-only patch3f0c6485;
after harness preserves complete original goldens,56rank1/12language inputs,
fresh before/candidate release builds and48timed processes. No cache or
rank expansion, no second accepted speedup yet. Earlier pending observations
below are historical and superseded by this final before gate.

After3858574 SUBMITTED (not accepted), from an empty expanded queue as ONE
job, no arrays/dependencies. Stage
`/public/home/majj/atlas-overload-after-20260930.jTo2AfSy`, pin
d0de137da82017f2266ccd7ab69074c3bbb5830d490fada5b93c1240d87515a0.
Receipt `tests/reference/hpc/math_overload_after_submission_2026_09_30.json`;
package28bda321c8fd3d4393cd328cf647bbf46802d696ff0ae9637ec2ab875506b2fb.
38checkers/598inventory/4new regressions/2full repaired histories/56rank1/
12language/48A-Bprocesses. Preserve the stage; collect this job, do not
resubmit. Other work while pending is source audit, no new runtime candidate.
3858574 PARTIAL at3m38: all38Python checker tests and compiler metadata
commands exit0; release core inventory compilation is live. No after-unit,
repair-stream, rank1 or benchmark acceptance yet. No pending tool session.
Submission receipt SHA0174b11094010af055f7dc4b4dacd2b7fdf10b82cb5fabb18a8c8debe7ed8117.
Follow-on analysis-local overload-cache boundaries/work-counter test plan are
documented in the performance slice, not implemented. That slice also records
one explanatory comment to correct after collection (row-to-row conversions
exist; proximity handles them componentwise). Do not edit the frozen stage.
3858574 newer PARTIAL at8m53:598inventory and both fresh releases compile;
5coercion/2whole-session/55type tests PASS. Both fresh original/candidate
repair histories fully match (FRONTIER_COMPLETE_MATCH and
RECURSIVE_FULL_STREAM_MATCH).6/56retained rank1 streams completed; no
language/A-B/final-integrity acceptance yet. Local runtime unchangedb32c06a6.
Prepared, UNTESTED/UNSUBMITTED math_overload_profile.py and companions
(22checker tests) require an inspected FINAL after report/hash. They sample
the same two loads, use the accepted uninstrumented binary for comparisons,
and isolate a same-source frame-pointer build. No new discovery/rank/runtime
change. Do not prequeue it behind3858574 or claim its tests have passed.
Latest scheduler check3858574 RUNNING oncu001 at12m26;32/56retained
streams unchanged, no language/A-B yet. The original-backed nullary and
recursive repairs both run normally (about0.01s/~6MiB in this observation);
do not form a speedup ratio against the failed recursive before process.
Profiler-only package is ready, UNTESTED/UNSUBMITTED:
`/tmp/atlas-overload-profile.4heJyjzC/overload-profile.tgz`, SHA
4913af29942caa8b13fc415354898012cf1909f4c8b625060994490e2c89fa96.
Only after inspecting FINAL3858574, pass its report path/hash explicitly to
stage_overload_profile.py. No new source/runtime candidate was added this turn.
Further source audit finds a smaller no-cache experiment: identifier-call
dispatch materializes merged_variants just for is_empty, then ordinary-call
conversion immediately repeats it. Grouped-type collision detection also
needs only presence. Preserve nonempty user variants/type-aware forgotten
builtins and diagnostics; use a thread-local work counter plus full A/B.
Details precede the stateful cache design in the performance slice. This is
not implemented or measured; inspect the new profile before selecting it.
Prepared an ISOLATED tests-only patch for that possibility, not applied to
main typed.rs and not submitted: hpc/patches/overload_presence_tests.patch,
SHA9c223a7e61ff87cde0491cc49f6d6c4a7a0f4c1dc9101bc2a17c0e0bbf33237d.
It adds3tests plus cfg(test) thread-local merged-view counter. Expected
before2WORK-fail/1semantic-pass is unobserved, not mathematical failure.
Isolated sourcecda308aeda58f6707fa99ecad26554104f3cbaecf15c98bc1afdbdc0f2abf1c5
in /tmp/atlas-overload-presence.j6o9L27D/typed.tests.rs; main typed.rs stays
41b9771a3c73076c87682ca376c76fd01abb00cd059c20ff0a4b01b95db9f9d7.
No presence helper/cache runtime change. Do not mix these3tests into current
598inventory or the prepared same-source profiler. Latest3858574 at17m42
has all56rank1/12language streams retained and3/16A-B rows; still not FINAL.
Added provisional overload_presence_history.atlas and overload_presence_rejected.atlas
for original capture BEFORE any no-cache presence optimization. Not run or
in a submitted catalog. Source expectations derive from axis.w2794–2813,
not current Rust output; retain all intended errors, captures and recovery.

LATEST3856293 FINAL COMPLETED0:0 in8m52 oncu069. Final report
`tests/reference/hpc/math_loading_followup_2026_09_30.json`, SHA
f4c27c536339a3de4ed0b47ab1ece8ec12618d8c76e508fa82182952b38e6454.
14checkers/two unchanged complete loading streams/two profiles/integrity PASS.
AV self samples: equivalent27.09%, generated Cartan partition14.91%,
validation8.16%, is_close5.10%. Resolved application frames show
merged_variants -> is_close -> equivalent; some outer frames are unresolved.
Do not claim a complete inclusive percentage or profile-binary acceleration.
Original ACCEPTS zero-argument replacement/forget/capture history; Rust adds
duplicate overloads and reports2Type errors. Full golden049cddde2d6b5f2cdc1540228665f1c47994edee7ddcb1ec9da42ec0c0528d34 retained.

NEXT3856744 SUBMITTED/RUNNING from empty queue, ONE before-only job in
`/public/home/majj/atlas-overload-before-20260930.YyVCvrMg`, pin
f297701754def664f638d622cc04ea63c3473d0e76084904b641591b54a37e96.
Receipt math_overload_before_submission_2026_09_30.json.6checkers,
596expected compiled inventory and2actual failing assertions pending;
production unchanged. Also captures one previously UNCONFIRMED recursive-row
identity history with accepted CLI2645dea4,45s/6GiB; no new group coverage.
Tests-only source hashes:coercions3433ccc541dc8b3036c32567fab8529cb708fcc82464887b7e0a0a9d99bc0b1a,
sessionbf4d40e3638677b8a954a2c080bfc456d1026db65aa7432f66676f5e69c88e1f.
Do not repair runtime until before evidence; do not weaken either test/golden.
3856744 PARTIAL: all6checker tests PASS; core inventory compilation is live.
Recursive discovery original ACCEPTS every report/11,23,37/captured11 marker;
original stdout4825c1d7293dfb8749d6b3e89a7fd8dd3ed6a8ebf94175563d9f833418740847.
Rust exits139 after24.26s/RSS6207724KiB, GNUtime reports signal11 and
stdout is empty. Preserve termination_uncertain=true (the observer marks
signal/resource exits this way), not TIMEOUT or any accepted speed ratio.
This confirms a process failure on the original-accepted history, not yet
the exact call site: pre-expansion recursive-name comparison remains the
source hypothesis. No full before-unit/final-integrity gate yet, no runtime fix.

CURRENT R3job3856241 COMPLETED0:0 in30m09, atlas-type-equivalence-20260930.VBs8Hh3A,
pin3b3dc8de0b7c361b97c31a822657eed3db2371eec025ba90d01cb25f30d20e96.
FINAL report `tests/reference/hpc/math_type_equivalence_r3_2026_09_30.json`,
SHA4b234d258c6df5909f9f26a0b7a2e53d4ead9a18ec06ce9dd1a993e10128f877.
All16A/B rows/48processes accepted, faster4/4repetitions on every input,
final source/artifact integrity pass. Full unitarity23.4033->12.4552s
(1.879x); full finite AV-ann51.0667->19.0573s (2.680x). Original0.2409/
0.3886s: Rust STILL51.71x/49.04x slower end-to-end. No kernel/multicore
claim or meaningful RSS change (~260MiB). Continue performance repair first.
24checkers/before3PASS1workFAIL/after4PASS/55type/3coercion/both releases
pass;594is compiled inventory only.56unchanged streams comprise53positive
math cases and3shared original Hodge failures;12language cases retained.
Candidate binary2645dea42d7727c9860fa93fb936d785fad0f8972909f25e4f76d87433fcd874;
before4ce4e738fbcc937882d82f0bceaf68ae9994a1fb797be672bdea392c93921d3e.
Local crate source hashes match the candidate. No pending tool session.
Historical3856293 submission (now FINAL above), stage
`/public/home/majj/atlas-loading-followup-20260930.i1cxrpAU`, pin
922f780e87642d5a0aa3c979678b1cd28fb205cbf2b60e226b622ad73d63e8a7.
Receipt `tests/reference/hpc/math_loading_followup_submission_2026_09_30.json`,
SHAfdebe682c09ffbcacd123c5ab8e5b4fa16f7c6457499dc204e31ca1c740da98a.
One job only; all gates now final as summarized above.
Package831dc7005cdd8ec3c9102737103f86183f23987c262d28559d624e9059801e9d.
Do not resubmit or change this stage. Source/runtime unchanged, no rank growth.
Remote original HEAD rechecked via login-node Git at06:28:20Z, still7e1b958c;
see upstream_head_performance_2026_09_30.json. Do not swap oracle mid-job.

3856122 FAILED1:0 in15m46 at the eleventh language fixture, AFTER all56math
histories stayed unchanged (53positives,3shared original Hodge failures).
Pure rejection correctly emits only BEGIN/END/Bye, so the old nonempty
payload condition was wrong. All arms exit1; full Rust streams match.
R3 keeps exact fixtures/diagnostics and requires full ordered unique recovery,
specific rejection categories and unchanged whole streams; adds6checker
regressions and checkpoints failed rows before aborting. Source/goldens/test
patches unchanged. Preserve reporte267b48a in math_type_equivalence_r2.
Submitted followup profiling now takes a final report path/hash, not R2;
R3acceptance is now verified. Old temporary profile package obsolete.
Followup now has14checker tests and a separately labelled rank0 discovery
`overload_void_redefinition.atlas`: source audit finds original tests equality
before void in is_close, while Rust returns0for(void,void). Original capture
now confirms the bug; before-unit verification is pending. Preserve
full definition/forget/captured-function history. Original coercion lookup is
also linear, but original is_close structurally prunes lookups/tuple tails.
See performance slice for exact lines and analysis-cache identity boundaries.
To address broken old DWARF unwinds, this followup builds a separate
same-source frame-pointer/line-info binary for sampling ONLY. All timings
and rank0 discovery still use the accepted uninstrumented binary. Profile
stdout must match that binary exactly; isolate source/target/flags, retain
build/compiler/hash evidence, inspect actual stack quality. No new Rust source
change. Modified math_loading_rank1 is included in the six-file followup
override manifest (five helpers +fixture); preserve the frozen manifest.
14followup checker tests PASS (log eee0014a9fa51ea8453d029374766f569db348a8a9ece7aa70b2f9404548a45a).
Profile-only binary78ec2dd16ee32cd798b9a6988845b34678b0636d9e29d70bfb25d016e205f489 remains separate.
A separate source-suspicion fixture, now submitted in3856744,
`tests/math/generics/overload_recursive_row_identity.atlas` checks distinct
nominal self-row identities and overload/forget/capture history. Original
guards recursive names before expansion; Rust may recurse forever. This is
NOT observed or original-accepted yet. Capture it before repair; do not edit
3856293 or label source analysis as an executable failure.

Historical R2job3856122 was SUBMITTED in atlas-type-equivalence-20260930.84qj5jbF,
pin80e594ab3b4c75c16b61a2ad5c05f0a78805c92f3e3b4561fdba67774dc764d1.
One job from empty queue; now terminal harness failure above, no acceptance.

Live partial gates pass:594inventory compiled; expected BEFORE3semanticPASS/
1workFAIL (16770vs258), AFTER4PASS/all55type/3coercionPASS; both release CLIs
built. Rank1 full-stream replays later completed; do NOT claim final speedup.
Prepared unsubmitted loading_followup harness/eight checker tests profile
ONLY a final accepted repair binary on the same two load inputs, with fresh
original comparisons/full output and resource retention. Requires inspected
final report hash, no dependency prequeue/runtime edit/rank expansion. See
the performance slice for source-only Cartan/dual design leads and boundaries.

UPDATE3856060 FAILED1:0 in4s BEFORE any checker/build/math: missing generic
classifier import. Raw log0c811f5d retained. R2 carries that exact classifier,
a12-case targeted catalog and unchanged fixtures from pinned source;18checker
tests now include the fixture inventory. Runtime/tests/patches unchanged.
Do not reuse the failed stage; see newest submission receipt/performance slice.

3856033 COMPLETED0:0. Four repeats prove loading dominates the rank1 inputs;
all16pairs/10checker/integrity pass. Profile52.80%type validation/19.68%type
equality self samples; do not rely on incomplete unwind call chains.
Report0b84a41851fd5fb08202bc071cf23ce75ba57f28d2ddc385e73ea865d61a93b1.
Local types.rs now has an UNVERIFIED narrow candidate: structural fast
rejection plus once-per-tree validation, revalidating newly exposed aliases.
Four permanent tests (594core inventory pending), tests-before and runtime
patches pinned; candidate source732380b6. New single-job gate rebuilds both
arms on the same node/compiler, expects3pass/1workFAIL before and4pass after,
replays56existing rank1 streams including shared Hodge failures,12language
accepted/rejected inputs, and4inputs*4rounds*3arms. No rank expansion or
performance acceptance yet. See [performance slice](slices/loading_performance_rank1_2026-09-30.md).
Historical R1submission3856060 was in atlas-type-equivalence-20260930.U8z5KvFq, pin
56b7dcf5de87014d171ee9537060c638537dc556134e7f2f4a12159d3c82714d.
It has FAILED at import as recorded above; use the fresh R2 job. Local
tests/builds/checkers were NOT run.

## Latest priority: small-rank performance repair — 2026-09-30

User explicitly requests fixing the large performance gap FIRST, from small
rank. Pause new math/rank expansion. Existing rank1 full outputs remain
regression gates. Prepared loading experiment compares load-only/full
unitarity and AV-ann inputs with identical setup, four fresh-process rounds,
balanced order, fixed binaries/threads/node, full wall/CPU/RSS and output
retention. It is not a performance repair or a multicore claim. Profile
before choosing a runtime change; preserve full semantics and benchmark the
unchanged inputs before/after. Default one job, ceiling ten outstanding.
Diagnostic3856033 is now submitted (one job, initially empty queue), exact
verified binary reused. See [performance slice](slices/loading_performance_rank1_2026-09-30.md)
for frozen pin/protocol/source leads. No runtime optimization yet.

## Rank1 real/complex unitarity expansion FINAL — 2026-09-30

3856011 COMPLETED0:0,2m25; report
0c1147bfbccf50a05674b25e6c3dfe4a14d079ea7418d1bff33a07f2f53fe390.
All six full math tails MATCH, both engines exit0/empty stderr,15checker/
integrity PASS.30gamma sets contain46parameters/46final constituents. SIX
actual empty sets occur in SU2/PSU2; nonstandard/zero flags still never occur.
All coefficients/KL/unitarity/warm inventories preserved, complex rank1real
datum rank2explicit. No runtime edit, broader numbering or infinite coverage.
Rust22.631–22.842s vsoriginal0.229–0.263s full-process one-shots includeimports;
loader-prefix output differs. Source-only audit finds completion-vector
reconstruction/unfinished-type reparsing/transactional type-table cloning as
profiling leads, NOT measured bottlenecks or optimization claims.
See [form evidence](slices/unitarity_rank1_forms_2026-09-30.md) and linked
load-only protocol. This job is complete; no higher-rank job prequeued.

## Rank1 real/complex unitarity expansion submitted — 2026-09-30

3856011 SUBMITTED from empty queue, one job/no arrays/dependencies/rebuild.
Six original groups.at forms SL2R/SU2/PSL2R/PSU2/SL2C/PSL2C, five complete
fixed-gamma inventories each; exact old SL2R input retained. Complex rank1
has real root-datum rank2. Fifteen checker tests/full outputs/CPU-wall-RSS/
integrity required; no original acceptance yet. StageeLRD6alw, pin
d70b506b784d26352733a3807e06f5edf350e0a3ad9bfde352ccc2ced5e79e66.
See [form expansion](slices/unitarity_rank1_forms_2026-09-30.md) and receipt;
collect this same job, do not duplicate or release rank2 simple groups.

## Finite-cycle / AV-ann rank1 review FINAL — 2026-09-30

3856006 COMPLETED0:0 in11s; report
5c366f5ca96729e2d23075320a18aad48b011dcf3d8ae4a8b758f5ba8673e63f.
Both original3855999inputs now match complete mathematical tails;15checker/
integrity PASS, no Atlas rerun. Old ORACLE_INCOMPLETE_CONTROL was a marker
count bug, preserved in previous_status and the immutable capture report.
Four finite module dimensions1,2,3,5/two AV-ann routes/full characters and
four scaling controls pass; derived finite-point AC multiplicities are NOT
generic/nonzero-orbit cycle acceptance. Loader-prefix stdout still differs.
Both jobs complete, queue empty at final query; no rank2job submitted.
Source-audit boundary and report pins:
[cycle evidence](slices/associated_cycle_rank1_anchor_2026-09-30.md).
Prepared load-only controls are unsubmitted. Next expand remaining rank1forms
and nonzero-orbit multiplicity evidence, not bulk higher-rank dispatch.

## Rank1 finite AV-ann anchor FINAL; retained-control checker repair — 2026-09-30

3855999 COMPLETED0:0,2m35, reporta83efb1bec546be8635d0f9a5aad895b5a5d0d1946ed2f02f04dee573872f077.
Four finite modules match complete original math, dimensions1,2,3,5/both
AV-ann routes/full characters;12checker/final integrity pass. Old scaling
control is FALSELY incomplete because ORBIT(?!_ROOTS) counts ORBIT_COROOTS.
Both control engines exit0/empty stderr; raw artifacts are retained/rehashed.
New15checker and review-only harness prepared; no math rerun or runtime edit.
Review3856006 is SUBMITTED from empty queue in Zcp9H6GH, pin
f999f20621066b0415384d9ebfe8d68a918acb66a576048c2dda48fa2199ac95.
Only one review job, no math rerun. Inspect its FINAL before accepting control.
See [cycle evidence](slices/associated_cycle_rank1_anchor_2026-09-30.md).

## One rank1 finite-cycle / AV-ann capture submitted — 2026-09-30

3855999 SUBMITTED from empty expanded queue; only one job, no arrays or
rank escalation. StageY2kzRRx7, pin
0f00238b06d60387660b8e20b1bd2528016f6787946685c6c94096ae7f4d6cf1.
Reuses verified3855872binary;12checker tests/two inputs/independent finite
dimensions1,2,3,5/full character formulas/both AV-ann routes. The retained
SL2R scaling template comes from exact six-union. No production edit/build.
Original acceptance remains pending; submission is NOT mathematical evidence.
Receipt and source-audit boundary are indexed in
[rank1 cycle anchor](slices/associated_cycle_rank1_anchor_2026-09-30.md).
Original latest HEAD rechecked7e1b958c. Earlier unitarity3855929raw stdout
hashes2315328b/8e84224a verified locally; it includes both true/false results,
not only trivial positive controls. Do not duplicate either completed job.

## SL2R fixed-gamma unitarity capture FINAL — 2026-09-30

3855929 COMPLETED0:0,1m03; report
364387fb4bacedb0353fdf6e3a98157d0bdda6b860d8bc79a5a5425d82663d96.
Both full-parameter and retained trivial-scaling inputs are UNITARITY_MATCH,
exit0/empty stderr/complete math tails equal;9checker/final integrity PASS.
Five gamma sets contain3+2+2+4+4=15parameters and15final constituents; all
coefficients/order/KL/before-after-unitarity outputs retained. No nonstandard,
zero entries or empty sets actually occur here, despite synthetic guards.
Reused exact b9580f15candidate; no new runtime edit/build. Loader-prefix stdout
still differs; single-shot full-parameter wall0.278soriginal/23.643sRust includes
imports, not algorithm or multicore scaling. Details and scope are indexed in
[rank1 unitarity](slices/unitarity_rank1_parameters_2026-09-30.md).
No unitarity job remains pending; do not duplicate3855929. Finite-cycle anchor
is still only prepared, not original-accepted or submitted.

## Rank1 ordinary deformation repair FINAL — 2026-09-30

3855872 FINAL PASS within its bounded scope, report
beeb3e280738d528460e1935c00861c8540afe37e89c59f04773ffb2cd817226:
2unchanged regressions+2retained A1units,590compiled inventory,29checker tests,
release/integrity;45positive complete math matches,3shared original Hodge
failures retained (not48passes). All48tails equal; loader-prefix stdout and
error envelopes still differ. Both bare full_deform streams are fully equal;
elementary Hodge cold/warm checks alltrue with complete1-s/-s coefficients.
Binaryb9580f15dae817817963235962069b78deaf367be1961115ea5f9b7af0ee48b7.
No full590unit or higher-rank/general Hodge acceptance. Older pending3855872
notes below are historical. Next one-job rank1unitarity stageQh9dNMvS is being
submitted using this exact binary/report; inspect its receipt before retrying.
Detailed evidence and timing boundaries are in the ordinary deformation slice.
Unitary3855929 SUBMITTED from an empty queue in Qh9dNMvS, pin
0716191df1332366a4f27fcf9fe843c114a13624a5f56421aae59f7e9a983aee.
Exactly one job, two rank1 inputs,9checker tests, original versus verified
b9580f15binary; no rebuild, dependencies, arrays or rank escalation. Collect
this same job later; status is SUBMITTED_NOT_VERIFIED, not unitarity acceptance.

## Prepared rank1 AV-ann / finite-cycle multiplicity anchor — 2026-09-30

New `av_ann_rank1_finite_cycle.atlas`/expectation are NOT submitted or
original-accepted. Explicit SL2 Sym^m dimensions1,2,3,5 distinguish equal
zero-orbit support from different module-cycle multiplicities. The constant
good-filtration argument is recorded with a primary definition reference in
[rank1 cycle anchor](slices/associated_cycle_rank1_anchor_2026-09-30.md).
The fixture compares both original AV-ann routes and whole character formulas;
its cycle multiplicity is derived in this finite-dimensional case, not output
from a generic AC algorithm. Full nonzero-orbit/multiplicity coverage stays open.

## Ordinary full_deform before FINAL; recursive candidate — 2026-09-30

3855838 FINAL verifies1controlPASS/1actual split-streamFAIL,590inventory,
unchanged production and final integrity; reportd3288de2606076c5808eb06ba09c8ed1c7ac48c782efe251f1c36dadda568c17.
Only then ported the full recurrence F=L+sum c(1-s)F(child), not a factor-only
shortcut. Complete-only memoization/active-cycle guard/deadlines retained.
Candidate0359261f95641021206a20c18c0837339952c81b386a12d42e6fbf38b75aa258
is NOT after-verified. Fresh one-job rank1 gate prepared:2unchanged regressions,
2retained SL2cache/alcove tests,41foundation+7Hodge/deformation cases. No full
590unit sweep or rank2escalation. See indexed ordinary deformation slice and
the after submission receipt when available; never duplicate an uncertain job.
After3855872 SUBMITTED from empty queue, stageSBSbHIxR, pin
95ea63398928cc04c1ccef04560b777b0ce1b24af7e5907be2144abd8701aec5.
One2CPU/8GiB job, no arrays/dependencies. Require FINAL report and48case review;
submission/build alone is not acceptance. Do not mutate the frozen stage.
Next local fixture `tests/math/progressive/unitarity_rank1_parameters.atlas`
is prepared only, not submitted/original-accepted: all SL2R parameters at
gamma/rho0,1/3,1/2,1,3 and full final-constituent unitarity/KL histories.
It is an ordered shared-cache session, not a fresh process per constituent.
Keep its adjacent provisional expectation and old trivial-scaling controls.
The prepared `hpc/math_unitarity_rank1.py`/stager/checker now require9synthetic
checks and preserve5complete gamma inventories, all final coefficients,
nonstandard/zero/empty entries and ordered KL histories. The retained control's
MATH marker is inserted after imports/bindings, before UNIT_FINAL_INPUT, so
loader-prefix differences are not confused with mathematical mismatches.
This was prepared before submission; the newer3855929receipt above supersedes
the old no-submission notes below.
Prepared package SHA0f541d0418158b325dc739e0d432e12cae50d210c328f41de964af79816ed83f;
paths/source hashes are in the ordinary deformation slice. The reduced parent
omits the old SL2Rcontrol: stager imports its pinned six-union bytes explicitly.
Latest3855872live: RUNNING,2unchanged regressions PASS/release built/14of48case
files, NOT FINAL. No further jobs submitted, still one outstanding job.
Update:43of48case files at the later poll, still RUNNING/no FINAL. Next
unitarity package has been uploaded/extracted only (NO sbatch) at
`/public/home/majj/atlas-unitarity-rank1-20260930.Qh9dNMvS`.
Archive SHA0f541d0418158b325dc739e0d432e12cae50d210c328f41de964af79816ed83f
and driver/stager hashes match. Once3855872FINAL is inspected and accepted,
run the prepared stager there with its explicit report SHA, without creating
another stage or scheduling a dependency chain. No compute checks ran on login.

## Ordinary full_deform isolated; permanent before gate submitted — 2026-09-30

3855719 FINAL original accepts elementary Hodge reduction, Rust fails4cold/warm
v=s assertions. Full sides isolate c_form coefficients, not equality/order.
3855753 FINAL bare compactMATCH/splitMISMATCH, both fully accepted,8checker/
integrityPASS. Two complete original goldens +2session regressions added
(local590core). Before-only3855838 submitted from empty queue, no runtime fix.
Require actual1controlPASS/1streamFAIL before editing production. Exact source,
pins, goldens, scope and full recursive algorithm in
[ordinary full deformation](slices/ordinary_full_deform_rank1_2026-09-30.md).
Source defect: missing1-s AND recursive contribution; do not patch only factor.

## Rank1 foundation FINAL; Hodge reduced regression submitted — 2026-09-30

3855559 FINAL41/41complete mathematical matches on both six-union and Cartan
candidate:1Cartan+12KGB+12block inventories+16full KLV. Full20pairs include4zero.
Reportb31c94ddb73077f721396c968739b86d375fa11a694a89f0926237019916ddf1;
candidatebinary30eee954326380d9f140c2098ca93c185423d4c9086660f017c2303b68930e7e.
Both integrity gates PASS,15checker tests/CLI release PASS. Loaderprefixes
still differ; not588unit/highrank/Hodge/F4acceptance or performance ratios.

NewHodge3855719 SUBMITTED from EMPTY queue in
atlas-hodge-rank1-20260930.mmWkq7ae,
pin35ed22f801d616ea5301f043591d026b20df7d56e7d5f6ca458ad3813a27f9ed.
Retains4prior cases plus new complete elementary specialization reduction,
7checker tests; original+six-union only, no production edit. Keep rank escalation
paused for this newly exposed small-group discrepancy. Inspect full cold/warm
polynomial outputs and capture original golden before fixing anything.

## NEW rank1 Hodge mathematical discrepancy beneath shared error — 2026-09-30

Hodge3855673 FINAL, report13f50098469c1531c6faed253515a5ed46ad99cfce7ccb5d307f2770dea17cc3:
unpadded control MATCH; both engines reproduce trailing-zero multiplication
failure. Bound4complete raw trace proves original elementary[a,b,c,d]=alltrue,
Rust[true,false,true,false], BEFORE both hit the later padding out-of-range.
Do NOT dismiss the whole Hodge discrepancy as original-side failure. New
`tests/math/progressive/hodge_rank1_specialization.atlas` and `.expect.json`
track the two v=s identities with full left/right polynomials. Reduced capture
is PENDING, no runtime fix. Existing whole traces/errors/goldens stay unchanged.
Detailed evidence and next action in progressive validation slice.

## Rank1 Hodge diagnosis submitted separately — 2026-09-30

Hodge3855673 uses ORIGINAL and VERIFIED six-union, not pending Cartan repair.
Four small-rank cases: unpadded control, trailing-zero failures, full SL2R
branching traces at4/12. Stage8tixLg8l/pin61ddacc9d0bc308e3fec45727279fd557e16e7bc0d00fbc8450287615dd2f614.
Only3855559 was queued when this ONE job submitted: total2, both rank1.
First Hodge stagePOUF5rvs was explicitly rejected for1CPU/8GiB (4GiB allowed).
Fresh successful queue+accounting queries proved no Hodge job; exact intent
archived as REJECTED_CONFIRMED_NO_JOB before releasing it. R2only requests
2CPU/8GiB; do not edit failed/submitted stages. See progressive slice/receipts.
3855559 release compiled successfully;21/41case artifacts existed at last
poll, still RUNNING, no full mathematical acceptance or rank escalation yet.

## Progressive rank1 gate submitted — 2026-09-30

Exactly ONE job3855559 submitted from an empty queue, new serialized10-cap
guard. Stage/pin/scope: [progressive validation](slices/progressive_validation_2026-09-30.md).
Fresh Cartan candidate build then41rank1-only original/before/after cases;
no full unit/high-rank sweep or rank2dependency. NOT accepted until FINAL and
inspection. Keep all higher-rank regressions and F4consumers pending, unchanged.
Original latest GitHub HEAD checked locally:7e1b958c, same as pinned oracle.
Latest live poll:3855559 RUNNING,15synthetic guard/comparator tests PASS,
release compiling atlas-core/atlas-real-group; no final math report yet.
Rank2 Cartan and rank1 Hodge padding fixtures are only prepared locally,
not included in this frozen job. Do not duplicate3855559 or promote rank.

## USER OVERRIDE: maximum10jobs, small groups before larger ranks — 2026-09-30

Pause new submissions and inspect the live queue first. At most10jobs per
batch AND10outstanding Atlas jobs, counting array tasks, builds and reviews.
No speculative later-rank dependency chains. Smaller-group complete original
comparisons must pass before raising rank, one level at a time; failures must
be fixed and retested, not bypassed with a larger survey. Keep full catalogs
as pending backlogs. This supersedes old submit-all/quota100shard advice below
and is now a hard rule in AGENTS.md. Do not cancel unrelated users' work.

Current Cartan before3852364 FINAL1controlPASS/1relabelled streamFAIL,588test
inventory, unchanged production/integrity. ReportSHA
d3432a120b1f0d058bf2a164f29e058a758464330ac1467d7c8dbd6a86e36596.
Only AFTER that proof, local Cartan_matrix_type now uses existing
bourbaki_permutation instead of identity. Candidate sourceSHA
cc935c8fb10e414d8a04daebce4b3976ff6a16ef22f30f47b1cd039225f90633;
patch hpc/patches/cartan_type_permutation.patch. NOT after-verified; no after
job has been submitted. Preserve all unchanged goldens. Next verification
must follow the new small-group/rank-gated policy, not one large after sweep.

Full640KGB reviewer3850878 also FINAL640/640MATCH on pre-candidate six-union;
report math_rank6_kgb_review_2026_09_30.json SHA
f853ce0dc6221564ff853a835bc4a612774eb31aeb9d7b389df2687759ba3c22.
This does not verify the new local Cartan candidate or higher consumers.

## All640block inventories FINAL MATCH — 2026-09-30

3851489 FINAL640/640BLOCK_INVENTORY_MATCH on current six-union; all original
forms accepted and final integrity rechecked.2128dual pairs exported =1114
nonempty+1014zero; complete original coverage. ReportSHA
1e447dd5feab712b752b5bc6b6024fc996c746700452a698a9595d9652b1d146.
Pair manifest/pin and next per-pair scope in
[rank6block KLV](slices/rank6_block_klv_2026-09-29.md). This is inventory,
NOT full1114KLV or higher mathematical acceptance.640KGBreview3850878still
pending; no duplicate arrays. Cartan before3852364 still running at last poll.

## Full408review FINAL; Cartan before-regression submitted — 2026-09-30

3852258 FINAL:289MATH_MATCH/4MISMATCH/13Rustfail/3Rusttimeout/2Rustresource/
75originalfail/13originaltimeout/9rejection-category matches. ALL408retained;
this is old572source, NOT current six-union. Several FPP/GL2/G2/PSp failures
were already repaired later. Exact frontier and hashes in
[full408review](slices/full408_review_2026-09-30.md). No broad Hodge/cycle or
current correctness percentage. Review OOM was resolved by allocation only.

Cartan before3852364 SUBMITTED in atlas-cartan-before-20260930.EHIPbmct,
pineb16beb9e092f0281d2790766dba03318213da4eec717064dc513378457c0333.
Two complete original goldens, expected588inventory/1PASS+1FAIL, unchanged
production. Source/runtime repair waits for this gate.640reviews remain pending.

## FINAL24parameter matches; F4 class omission isolated — 2026-09-30

NEW Cartan capture3852252 FINAL:11canonical controlsMATCH,7relabelled
diagramsMISMATCH,all5673grid rowsoriginalACCEPT/RustMISMATCH,4invalid-call
recovery streams retained. Two original-backed permanent session tests added,
local588core (NOT yet verified), production unchanged. Before-only gate uses
exact six-union586 +2tests; require1controlPASS/1streamFAIL before repair.
Report/pins/golden hashes in parameter/F4 frontier. Existing domain helper
bourbaki_permutation already implements original tie choices and is imported
by domain_builtins; do not invent another arbitrary mapping algorithm.
Full408unchanged reviewer retry3852258 submitted32GiB/fat; no math reruns.

Review3851780 FINAL24PARAMETER_GRID_MATCH/3currentF4AV-ann RustFAIL/
1rejection discovery. Both original F4class traces3851976 ACCEPT; first
mathematical difference is C3Levi[1,2,3] elliptic representatives, with one
missing/one duplicate despite the same25class count. Kondo12lookup fails;
backtrace proves signature lookup, not linear_solve. Cartan_matrix_type's
identity-only permutation is a concrete source defect. Bare and full5673
rank<=6Cartan-permutation capture added; no runtime change before reduced
original-backed regression. Exact hashes and boundaries in
[parameter/F4 frontier](slices/rank6_parameter_frontier_2026-09-30.md).

Full408arrays have finished; old reviewer3848053 OOM at2GiB after63checks
passed. Retry reviewer ONLY with32GiB/fat, unchanged code/inputs408/runtime.
640KGB/640block reviews3850878/3851489still pending. Do not duplicate arrays.
Earlier pending individual/trace/408descriptions below are historical.

## Fixed-gamma discovery submitted; F4 AV-ann still fails — 2026-09-30

New28case array3851779/review3851780 after FINAL7-test checker3851750:
24full fixed-gamma inventories (6forms x rho/zero/half-rho/twice-rho), all
standard-parameter final constituents/coefficient/order/warm checks;3unchanged
F4 AV-ann replays and1rejection discovery. Stage/pins in
[parameter frontier](slices/rank6_parameter_frontier_2026-09-30.md).
Do not treat rejection discovery as a diagnostic compatibility pass.

First FINAL cases:3851782F4compact and3851783F4rankone originalACCEPT/RustFAIL
even on six-union. First compactA1/rho3851803matches. F4reduction3851976 now
submitted with full Levi/conjugacy/Kondo/signature/character/Springer trace in
both numberings; no runtime edit. Generic No solution found can arise from
binary_lookup(...).requisition, NOT necessarily a linear solve. Trace before
dependent undefined-name errors overwrite back_trace. Preserve legacy fixtures.
R1synthetic checker SyntaxError3851698 is archived; R2fixes only missing bracket.
640KGB/640block and full408fat jobs still live, no duplicates. Upstream HEAD
refresh timed out; pinned original remains7e1b958c, no fresh-update claim.

## ComplexA6 complete KLV FINAL; next controlled-performance candidate — 2026-09-30

3851324 FINAL12checker/complete block inventory/full raw+dual KLV MATCH,
5040rows,25,401,600slots per direction, full pools/order/warm histories and
final integrity PASS. Report math_rank6_klv_scale_3851324_2026_09_30.json SHA
1007c4c2655e57ac49b4188dc26404485641381f78b2a1dbc6ecd9efe303dfb1.
All4scale formsA3/A4/A5/A6now MATCH. Single-shot A6KLV original26.983s/
413876KiB,Rust69.806s/1403840KiB; NOT calibrated ratio,Rayon1,loader prefixes
still differ. Batch6m26s is NOT Rust execution time (includes harness review).
Next prepare a separately accepted full-output-equal bare fixture before
controlled A/B; source audit points to raw_KL per-Block caching/graph ownership
and matrix-conversion copies, not absent polynomial interning. No runtime
optimization implemented. Keep current correctness source frozen.

Full272inventory replay FINAL264MATCH/8original rejections; all48old Rust
failures recovered/no lost matches. CPU130full KGB FINAL130MATCH. All640KGB
and640block inventories are fully submitted, reviews3850878/3851489pending;
no remaining dispatch. Full408fat3848052/review3848053 still separate old572
survey. Collect these later in batches; develop next per-pair/full-parameter
coverage meanwhile. Never label these finite gates universal correctness.

## Full272inventory replay FINAL — 2026-09-30

QUEUE UPDATE: all640block metadata cases now submitted, final review3851489
after CPU3850907 and fat3851457. No remaining shards; do not resubmit. The
earlier600/40and other partial counts below are historical. Graph640review
3850878 and KLV complexA6job3851324 remain pending/running at last observation.
Source-only performance audit notes raw_KL's missing per-Block warm cache
versus original, not missing polynomial interning; no runtime edits/accepted
optimization. See rank6_block_klv slice before designing a controlled A/B.

3850467 FINAL264INVENTORY_MATCH/8original rejections. ALL48previous Rust
complex-rank5/6failures recovered, all216old matches retained; no new failure,
mismatch or timeout. Report SHA
3022a69e22eb9e996c6b7c131969fcc1d1e86a8ac131599eff3b9b5402b63031.
Still inventory-only, not higher-consumer acceptance. CPU130full KGB FINAL
ALL130MATCH remains valid. New complexA3/A4/A5full KLV scale jobs3851325,
3851337,3851348 FINAL MATCH with full streams/coefficient pools/history;
A6physical job3851324 still runs. Block metadata600/640submitted,40fat
indices and review remain; resume checked quota-aware sidecar, do not duplicate.
Newest exact hashes/receipts/frontiers: [dispatch](slices/rank6_dispatch_2026-09-30.md).

## Rank6 preflights FINAL; all640KGB submitted — 2026-09-30

NEW CPU130independent review3851146 FINAL: ALL130complete KGB graphs MATCH,
1000nodes/2904edges,15checker tests and full final integrity PASS. Report SHA
bf907f33fc686fb019803caa343ce47c6ce41c4eb0f679d34c8839797af7328e.
Fat510/all640review remain pending. Complete complexA3/A4/A5/A6KLV scale
array3851324 is submitted, not yet collected; exact scope/pin in dispatch
slice. Block inventory now534/640submitted;106fat indices+review remain.
Latest shard receipt supersedes the earlier163snapshot below.

3850437 FINAL13checker/10inventory matches; unchanged272arrays3850465/66
and review3850467 submitted. 3850436 FINAL15checker/10complete KGB matches
(5841nodes/67960edges). All640KGB submitted: oldCPU1303850496 adopted, fat
3850870/73/74/75/76/77 chained, final review3850878. 3850448 FINAL12checker,
10block inventories,12complete KLV and rejection/recovery PASS on six-union.
Block grid163/640submitted (CPU3850879/3850907,fat3850908);477fat remain.

Initial fat510submission hit normal QoS800submitted-jobs limit after CPU
succeeded. Never duplicate CPU or rerun the old launcher. New quota-aware
sidecar3850863 FINAL9tests PASS, including interrupted-submission fail-closed
guards. Resume remaining block shards with stage_rank6_shards.py; exact stages,
preflight hashes, receipts, scope and recovery instructions are in
[rank6 dispatch](slices/rank6_dispatch_2026-09-30.md). No full640acceptance yet.
Earlier pending-preflight descriptions below are historical. Full408fat3848052
and review3848053 remain separate old572jobs; do not duplicate.

## Six-repair union FINAL; three rank6 preflights submitted — 2026-09-30

3850248 FINAL SIX_REPAIR_UNION_GATES_PASS_BROAD_MATH_PENDING, report
math_rank_capacity_union_2026_09_30.json SHA
386924a81faa1114de1b06dec0c8f194b6825cf0005ff08dca638de756814dc0.
Exact586core/515domain inventories and all tests PASS;26complete mathematical
tails, all7unitarity companions, each parent's full positive/rejection gates,
273+11no-loss,4block histories,4complexFPP and final source/input/binary
integrity PASS. Binary3c912384a50f79e9a6d56d0d2f1337e5aaa626b89ebd2dbe8748d110b802f176.
All90local crates/* source files rehash identically to this verified source.
This is now the combined baseline; full rank6/408/performance remain open.

NEW preflights on exactly this candidate (NOT arrays/full acceptance yet):
-640KGB3850436: atlas-rank6-graphs-20260929.nvJ6nTIt,
 pin46986ae8c57e1dfdd75277549cea8923dbb31cce0450d3d9bc2174879a5a6559.
 Requires15checker tests and10complete graph matches, including complexA5/A6.
-272inventory3850437: atlas-rank6-replay-20260929.1FM6fd0x,
 pinf63a788052a40c0f642e106f6de35dabf94d874fb90e94d6a82a6a088714b1d1.
 Requires13checker tests and10complete inventories, then unchanged272array.
-640block-inventory3850448: atlas-rank6-block-grid-20260930.b2axzJak,
 pin8ec345bbe4ae26104f1bafe66f9d71d8b057a1856262464de0fcc2b81dae4e29.
 Requires12checker tests,10metadata smokes,12full KLV replays and rejection
 recovery on the new candidate, before all640dual-block inventories.

Block-grid checker3850433 FINAL12PASS, report SHA
167dbb4c5a30b7119666692595d1940425d5e6014960f20b14d3e1cefa0edef0,
stage atlas-rank6-block-grid-20260930.NgC943ix, pin
c30eb8df79fd91fdafba75307d6daadf0bacb09b0d99a4983cebf89b2f6552e8.
New driver independently reviews all640indices and raw outputs, emits every
original dual pair including zero sizes and Rust-failed cases. Original
failures/timeouts remain explicit missing-form entries and make the pair
manifest incomplete; never silently claim all-form KLV coverage from them.
Package atlas_rank6_block_grid_inputs_20260930.tgz SHA
7afcbbf96f1907dc98a1808c2a940d2fc53fd673166ec564790a3a729ce59c21,
scratch /tmp/atlas-rank6-block-grid.lMPbup7N/package. Templates unchanged
from FINAL3850366; capture only gains explicit optional time/memory limits.

Next collect these exact preflights and launch each array only on all gates.
Full408fat3848052/review3848053 still live on572; do not duplicate. All earlier
claims that the six-union is pending are historical and superseded here.

## Complete12block KLV smoke FINAL — 2026-09-30

3850366 FINAL BLOCK_KLV_SMOKE_CAPTURED_NOT_ALL_FORM_ACCEPTANCE,
report math_rank6_blocks_r3_2026_09_29.json SHA
56465af884d443f9139f4293a2d3e94b9c21db3bab602835753ddd50d166ae42.
Seven checker tests PASS,8complete block inventories MATCH, ALL12nonempty
blocks COMPLETE_KLV_MATCH, both rejected overloads recover. All positive
stderr empty/equal; full mathematical intervals equal through END/Bye.
All raw/dual matrix slots, coefficient pools, lengths, print_block and
interleaved cold/dual/warm histories retained.103total block rows and2811
matrix slots per raw/dual table;3zero-size pairs retained separately.
Final source/input/binary integrity rechecked. Diagnostic envelope wording
and loader-prefix differences stay explicit; no stable speed ratio.

Scope is exactly verifiedfive583/511 and the eight catalog forms, NOT640
forms, singular/fractional parameters or all higher consumers. Original
templates now accepted; previous tuple assertion failures remain archived.
Next use the FINALsix-union for640block-inventory extension and independent
per-pair arrays, after640KGB/272replay preflights. Six-union3850248 and
full408fat3848052/review3848053 are still live at latest poll; do not duplicate.
GraphR3package15checks and272replay13checks are ready after FINAL union.
See rank6_block_klv and rank6_form_graphs slices for exact packages/pins.

## Eight block inventories FINAL; complete KLV fixture correction — 2026-09-30

3850338 FINAL capture, report math_rank6_blocks_r2_2026_09_29.json SHA
e074914524c3b733dc4df28e71a997b1e3dc718965ce5a0c91011ecc2a075e0f.
All7checker tests and8complete block inventories MATCH;15dual pairs include
3zero-size and12nonempty blocks (103rows,2811matrix slots per raw/dual table).
All12KLV fixtures were ORIGINAL-rejected only at the two whole-tuple '='
assertions: basic.at has no equality on ((mat,[vec],vec),(mat,[vec],vec)).
Do NOT call these Rust math failures or accepted KLV tables. Complete
streams, errors and original12pair registry are retained. Negative companion
also exposed a classifier assumption: original overload analysis is a
Program-error envelope, while Rust uses Type error; messages name both calls.

R33850366 SUBMITTED in atlas-rank6-blocks-20260929.ociO8VP2, pin
2c4d250d0c41aee2177ca6e12746f4fe057de74438c761d4a9073e0484026127.
Replace ONLY invalid tuple assertions with matrix/vector '=' and [vec]'=='
(original basic.at119); retain full cold/dual/warm outputs. Rejection checker
now requires both exact failed overload messages in their observed per-engine
envelopes, never generic nonempty stderr. Exact diagnostic difference remains.
R3archive atlas_rank6_blocks_r3_inputs_20260929.tgz SHA
7b2c1862668ec3094ccbaa580933d6110d2dc5f10d07f50a3213342d92d977ba,
scratch /tmp/atlas-rank6-blocks.ElioNtLv/package-r3. Source remains FINALfive
583/511; six-union3850248 and full408fat remain live, no duplicate jobs.
Next collect R3and six-union; launch NEW640graph/272replay preflights only
after FINAL586/515. Use graphR3package15checks, not earlierR2package14.

## Complete KLV discovery submitted; graph final-failure guard verified — 2026-09-29

NEW KLV8form/all-nonempty-pair discovery3850338 SUBMITTED in
atlas-rank6-blocks-20260929.05nAxJ2Z, pin
58ddea0fac4332464d002e72cee55ee155a7f7c510d25533eb49e9b1cbf063ed.
Uses FINALfive583/511, not the live six-union. Seven checker tests precede
8complete dual-block inventories, then every ORIGINAL-positive-size pair's
full raw/dual index matrices, coefficient pools, lengths, print_block and
cold/dual/warm history, plus rejected-input recovery. R13850300 failed only
an overbroad checker '@' assertion before Atlas; R2recognizes valid drf@j
binders and rejects only actual @UPPERCASE_TOKEN@ placeholders. Templates/
driver/candidate unchanged. See indexed rank6_block_klv slice for pins/scope.

Graph R33850330 FINAL15checker tests PASS, report SHA
d3f88d45d8cae032fa5ef3fec3ed0ce45acffdc6ed9265a40e06fce89b1474bb.
Added a fault-injection regression: if final source verification raises after
capture, main must set HARNESS_FAILURE and exit1, not retain CAPTURED/exit0.
Independent review already rejected missing integrity, but the process status
also must be correct. No math code/template/candidate changed. New stage
atlas-rank6-graphs-20260929.ERLPriFN, pin
cf772c7b9934baf596598cc5fb8e38ad3288bb53138c419fbac71676fd2fb2e9.
Use R3package atlas_rank6_graphs_r3_inputs_20260929.tgz SHA
4a58c2cdb2b187786f3683ef4d28237121c2b65362a9cf795814b823e126166b,
scratch /tmp/atlas-rank6-graphs-r3.t182zGyM/package, for future640preflight.
Do NOT use superseded R2graph package for bulk launch.272replay's frozen
helper source remains independent; its own13checker tests still apply.

Six-union3850248 and full408fat3848052/review3848053 remain live. Collect
authoritative FINAL reports in batches; do not duplicate or call live logs
accepted. After six-union FINAL run NEW640graph and272replay preflights,
then their arrays. Older pending notes below are historical.

## Capacity FINAL; six-repair union submitted; all640 checker FINAL — 2026-09-29

272replay harness3850280 FINAL13checker tests PASS (no Atlas execution),
plus compute syntax checks of all4override Python files and before/after
input manifests. Stage atlas-rank6-replay-20260929.UG0dkAem, pin
80880e6c6ceb3bc3c1e92251e73d8f8c0046562ddd1228fe8062449e79060083.
Checker log SHA af89064c1ef329d4c53aedc406fcdc22208c30d0945260a5ec3e9871185a45d3;
package atlas_rank6_replay_inputs_20260929.tgz SHA
5b60b230caaf4ce21c080dc5d9f853db983fee64ea5e3e0e68a9ddc7407509dc,
scratch /tmp/atlas-rank6-replay.CgsVyFdc/package. No272replay array yet.
After FINAL union use stage_rank6_replay.py prepare with exact candidate
path/hash in a NEW stage;10inventory smokes must match before launch.
Every old index remains mandatory; review compares unchanged original raw
streams and reports recovered48/lost prior positives explicitly. It does
not hide original8rejections or assume every cap failure is already fixed.
The checker-only receipt records oldfivecandidate but runs NO Atlas; its
status cannot authorize array launch. Full replay verifies SIX586/515 only.

3849939 FINAL COMPLEX_RANK_CAPACITY_GATES_PASS_BROAD_MATH_PENDING.
Report math_complex_rank_build_r3_2026_09_29.json SHA
9592a2d1887822907edebbabafd49da6ccb16de912eeb4d214a16b1f218be8c5.
All3unchanged original regressions,575core/515domain, complete rank4/5/6
stdout/stderr,273+11no-loss,4script/4block/six accepted unitarity controls
and final source/input/binary integrity PASS. Wrong-size diagnostic wording
gap remains; isolated source still lacks the other five repairs. No speed claim.

Six-parent union3850248 SUBMITTED, NOT verified, stage
atlas-rank-capacity-union-20260929.5Fd0DALz, pin
b4187023dd7d514d33040ccb6e6f047eb8eff3417e364441612a0a0fa8d2a76b.
Requires exact586core/515domain union and all retained complete parent gates.
Package atlas_rank_capacity_union_inputs_20260929.tgz SHA
c286c539609bfc9771f5a3f9e175d0a977834539a524d293bce9165c51ab27b5.

All640 KGB runner/stager/reviewer now implemented. Checker R13850133 failed
at Python parsing (one extra closing parenthesis), before any math execution;
retain its log/receipt. R23850194 FINAL14checker tests PASS, report SHA
3d23d433a273a5b5822019cd2bc708ef85dbf89c156f9c1eacad0cff2343c87f,
stage atlas-rank6-graphs-20260929.VS6KNoL9, pin
f57af61c60a690d6b93625caa8c9d32b0936e0c2f7ab11e01f5f451348bdc4ae.
This is checker-only, NOT640graphs or Atlas acceptance. R2package
atlas_rank6_graphs_r2_inputs_20260929.tgz SHA
1d5117670f282066bb883851c761c178ed8fe10f6cf1ae46f7fdab438043d452,
scratch /tmp/atlas-rank6-graphs.S9Ebwehb/package-r2. Frozen old stages unchanged.

After FINAL3850248, prepare a NEW graph stage with its actual report hash;
ten complete graph smokes (old8 plus complexA5/A6) must match before640array
launch. CPU/fat arrays preserve every index, raw output and metrics; independent
review rejects missing/duplicate indices, changed artifacts and classifications.
Also replay272inventory on this new union; original8rejections remain visible.
Full408fat3848052/review3848053 remain live on572; do not duplicate.
Older pending descriptions below are historical and superseded here.

## Five-repair union FINAL; 640forms exported; capacity R3 live — 2026-09-29

3849627 FINAL FIVE_REPAIR_UNION_GATES_PASS_BROAD_MATH_PENDING,
report math_repair_union_2026_09_29.json SHA
3745881e68b6a99b9c75eb96dccbc1154ffd2544b6d274530be0666bee813d55.
Exact583core/511domain, six merge-checker tests, all specialized complete
streams/rejections,26script mathematical tails (including all7unitarity),
273+11no-loss/4blocks/4complex FPP and source/input/binary integrity PASS.
Binary5b87ab4a6f4b05395b0d913916353789590c3bbf2d0a3b9dcf2eb3d661df1a3c.
This is now a verified combined baseline, NOT full408/rank6 acceptance.

Capacity R2after3849865 FINAL FAIL, report9729b62d93c7e30a212fbd50bb4217903923d99a780a3d1f0c15611d0e85ffc5:
all3original regressions passed, but domain test compilation found a fourth
hardcoded `[0;8]` assertion at inner_class.rs2196. The earlier space-sensitive
search missed it. Replace with compact.identity(), preserving the assertion;
whitespace-tolerant search now leaves ONLY the intended rank<=8u64 branch.
Production/goldens/new4kernel tests unchanged from R2. Do not count R2 as PASS.

R3after3849939 SUBMITTED in atlas-complex-rank-build-20260929.766BZXsS,
pin5c4cf5fdc7b17d1fae7da14d05a0b14c746ea4168999bc75f6b3347e0c8c62e0.
Live logs show full515domain (including4wide tests) and575core PASS; no FINAL
consumer/integrity report yet. R3inner source4b1e1da0e019b9ea4834c26d84d4038ffac9cd7583d6c9a384636b63a9d96bc3.
Archive atlas_complex_rank_after_r3_inputs_20260929.tgz SHA
ec4bf632f4b5203385a811ffd039621252e50b090d8b12ed11925695c7822f74,
scratch /tmp/atlas-complex-rank-before.a1okDdNp/package-after-r3.

3850019 FINAL FORMS_EXTRACTED_KGB_SMOKE_REVIEWED_NOT_ALL_FORMS_VERIFIED,
report math_rank6_forms_preflight_2026_09_29.json SHA
5bb018f7b149c778303a45644b3a1122e3eba9d851d795eac73c89f1da7d41dd.
All264original-accepted inner-class inputs yield640form presentations,
including48from Rust-failed complex inventories;8original rejections retained.
Nine checker tests PASS. Eight complete KGB smoke cases on verified583union
allMATCH:81nodes/280simple-root edges, full matrices/torus/status/cross/Cayley,
recovery/end/Bye and integrity. This is NOT640graphs verified. Manifest remains
on HPC in atlas-rank6-forms-20260929.33P9yowF/results/3850019/forms.json,
and its SHA/path is pinned in the local report. See rank6_form_graphs slice.

Prepared, NOT submitted or verified: hpc/stage_rank_capacity_union.py and
capacity extension to math_repair_union.py. Require FINAL3849939 before
staging; it imports17-check additions without replacing verified408reader,
combines six exact parents, requires exact local586core/515domain source
hashes/test-set union and ALL retained gates plus complex-rank negatives.
Next collect R3, submit this exact union, then full272inventory/640KGB arrays
and per-form KLV/Hodge/unitarity/actual-cycle/AV-ann extensions. Do not submit
the640array on old cap8 source or call the finite tests a mathematical proof.
Full408fat3848052/review3848053 still live on572; no duplicate survey.
Older statuses below are superseded by this section.

## Complex-rank failure proof FINAL; width repair submitted — 2026-09-29

Before3849785 FINAL, report math_complex_rank_before_2026_09_29.json SHA
56751eef412080d68839a2cc79cdcabb31f9f35c7465bc74de3e53f034436b55.
Exact572 plus three complete original3849710 goldens =575core inventory;
rank4PASS, rank5/6 each one exact cap8Runtime after valid setup/recovery.
Production unchanged and all source/input hashes rechecked. No compile/setup
failures counted. Permanent session regressions now precede the runtime fix.

After3849865 SUBMITTED in atlas-complex-rank-build-20260929.RFCB0azs,
pin3dd373e38b3577eb612cc55e00a20ad5987ddae3b824ce7b44f8e5c356eb4ce7.
Candidate uses original width32, centralized identity initialization and the
same rank<=8 u64 path/count budgets. Four added domain tests cover width32/33,
all512A1^9elements, noncommuting rank10twist's exact48set in2304elements and
all4096rank12root permutations. Requires unchanged3core regressions,
575core/515domain,4positive/rejection-recovery probes,273+11no-loss,
4script/4block/six accepted unitary controls and final integrity. No after
acceptance yet. Wrong-size-involution diagnostic gap remains explicit.

This isolated575/515source EXCLUDES the five-repair union3849627. Local
combined source is586core/515domain and is NOT verified together. Frozen
union remains583/511 and its consumers/integrity are still pending; full408
fat3848052/review3848053 continue on exact572. Never overwrite these stages.
NEXT collect after3849865 and union3849627 in batches; after acceptance,
verify their exact union before systematic per-form representation coverage,
and replay all272inventory cases on the widened candidate. Keep failures,
original rejections and timeouts distinct. Do not duplicate pending jobs.

Scratch /tmp/atlas-complex-rank-before.a1okDdNp holds exact572session base,
isolated575session, parent Weyl/inner files and checksummed patches.
Before archive d5d8db23e23fc2a5e6eb1a9409c2f1f1467938f9a64e56ec3ad710680b60e06e;
after R2archive atlas_complex_rank_after_r2_inputs_20260929.tgz SHA
9ac88b58e1de2c98c1eca354e878285e9869fdf647a1814fa8f5ec332e58f2e1.
First after staging jsRs5G4C stopped BEFORE sbatch: direct-capture parent
had old six-check bridge and no integration helpers. R2 copies the three
exact helpers from FINAL PSp stage; no guard weakened or Rust source changed
between R1/R2. Failed archive8cf2b8ce and stage remain for evidence. See
indexed complex_rank_capacity slice. Older pending descriptions below are
historical and superseded by this section.

## Five-repair union submitted; complex rank>4 frontier — 2026-09-29

UPDATE rank6 full272 review3849376 FINAL, report
math_rank6_inventory_review_2026_09_29.json SHA
893c2e5ecf075221e5c8d1135f4eedcb02e5010602a0b3713264ea38272ba125:
216inventoryMATCH/48RustFAIL/8originalFAIL, no mismatches/timeouts. Real
156match+8canonicalD4/D6quotient-u rejections; complex60match+48cap8failures.
All48firstdiagnostics are the same fixed-Weyl-cap error. Exact572inventory
only, not higher-level representation acceptance; original rejects never pass.
All source/binary/raw/input integrity FINAL. No need to poll these arrays again.
Original reduced rank4/5/6goldens copied into tests/math/generics with hashes
in the complex_rank_capacity slice; NEXT add3core regressions and before
proof, then widen representation with count budgets unchanged.

UPDATE3849710 FINAL capture, report6e1c465626171975f8cf6f15bc8f289fc0fd59243495a3b53ac3d632fddd6bc4.
Original all3positive ACCEPT; rank4control fullstdout/stderrMATCH; rank5/6
each RustRuntimeFAIL exactly once with resource limit8, setup/recovery stay.
Wrong-size negative bothRuntime with equal whole stdout/recovery but different
wording (original10x10vs9x9, Rustnotaninvolution): retain separately, not a
full diagnostic match. NEXT freeze original goldens and before-only3core
regressions before representation widening. No Rust runtime edit yet.

Reduced four-case complex-rank discovery3849710 submitted in
atlas-merged-diagnostics-20260929.nHYbpwpo,
probe e1eb1ac301f3866fe02d39738840e24387c538766d8cd85ec7159e090d742457.
Archive atlas_complex_rank_inputs_20260929.tgz SHA
95ef022c8249fddc9f5603e9273c18ca5acd717a08742c3f037a100a4aa6cba3,
scratch /tmp/atlas-complex-rank.RmCP3Iws. A report has appeared with SHA
6e1c465626171975f8cf6f15bc8f289fc0fd59243495a3b53ac3d632fddd6bc4;
inspect categories before adding before-goldens or any runtime change.
See indexed complex_rank_capacity slice. Union3849627 six merge-checks
PASS; it has reached CLI check after full583core/511domain gates. Still
no FINAL consumer/integrity result; do not call the union accepted.

PSp3849212 FINAL PASS, reportb691a9b3f053570d2d5109583c63a484867d41329da442098ae94783efeb75e7:
unchanged3regressions/575core/511domain/all5complete histories/sixunitary/
273+11no-loss/4script4block/integrity. Exact five-parent union3849627 now
SUBMITTED in atlas-repair-union-20260929.YjHaB5YO,
pin c6920a2e6d7a3b764730f7ab3c97238e860fd2becf4ddbf6b79c181a286f7d25.
Requires exact four local hashes, all583core/511domain and every parent's
full-stream/rejection gates. New tests/source changes must NOT enter this
frozen union. See indexed five_repair_union slice and submission receipt.

Rank6 raw observations identify a NEW bounded-representation failure:
A5complex(real rank10), A6/B6complex(real rank12), etc reject Runtime
`enumeration exceeded its resource limit of 8` while original accepts.
weyl_transducer.rs uses WeylElt=[u8;8] and rejects rank>8 at line304;
original constants.h RANK_MAX=32, and WeylElt uses that bound. This is NOT
the existing generated-involution count budget; do not raise that budget
or delete guards. A5_sc_root_C raw job3849446 is a concrete retained witness.
The new four-case complex_rank_catalog adds bare A4control/A5/A6 and wrong-
involution-size recovery, before any runtime edit. Original acceptance of
these reduced fixtures is still pending. Generic checker17/master273/408
unchanged; frozen union uses checker16. Wider compact representation must
also audit all identity arrays, rank<=8 u64 enumeration, test prototypes,
u8root-permutation range and >32 guards. No rank-cap runtime change yet.

## Rank<=6 systematic inventory; G2after FINAL — 2026-09-29

R2stage atlas-rank6-inventory-20260929.oBwg4QCY, pin
f570cc4e18c2163f09a786d9610c641e8ce8dc7867b9931dc713cc4e96748a68.
Preflight3849361 FINAL PASS, report66dbbf110be4fd04e0f335208ed8cec4db0833dcbf6852f16e43d24f9fe94187:
nine checker tests/eight original-accepted smoke cases with complete equal
math tails (24form presentations)/integrity. Loader prefixes differ.
CPU64cases3849374/fat208cases3849375 SUBMITTED; full afterany review3849376.
See math_rank6_inventory_submission and math_rank6_inventory_preflight
metadata; not full272acceptance yet. Do not duplicate these arrays.

Latest user asks for comprehensive comparisons over groups of rank<=6.
Added separate272-case inventory:22simple types,54SC/adjoint/intermediate
central-subgroup presentations, both root numberings,164real inner classes
and108complex cases. Complex rank<=6 includes real root-datum rank12.
Uses original e/u symbolic classes (not only +/-I, which misses D-even u),
full form lists/Cartan metadata/KGB counts and full tails through recovery/
end/Bye. Original-rejected quotient/involution combinations are discoveries,
never passes. Products/diagonal quotients/tori and per-form representation
operations are NEXT scope, not already covered. See indexed
docs/slices/rank6_validation_2026-09-29.md and tests/math/rank6/catalog.json.

New drivers math_rank6.py/.sbatch, test_math_rank6.py, stage_rank6_inventory.py
reuse exact572build3847662 ONLY for discovery. Nine harness tests/eight oracle
smoke cases must pass before272array dispatch. Keep local583union acceptance
separate. Existing273/408inputs unchanged. Source/binary/archive/inputs rehashed
on compute; per-engine600s/6GiB CPU or1200s/24GiB fat32G, full outputs/time/RSS.

Current archive atlas_rank6_inventory_r2_inputs_20260929.tgz SHA
a8338ce6dacd879c95d5171e208b64da3d4f3e45d65e7cfad5a855220069e756;
scratch /tmp/atlas-rank6-inventory.ZJywqCjM. First BPAczNf9 stage STOPPED
before sbatch: Python import generated one parent bytecode file. Exact extra
file moved into failed stage; no source missing/changed; exact parent input
manifest restored. R2 disables bytecode BEFORE import and via environment.
Do not weaken checks or rerun first archive8338bead. See slice for hashes.

Collected G23848845 FINAL G2_DEFORM_CROSS_GATES_PASS_BROAD_MATH_PENDING,
report math_deform_cross_build_2026_09_29.json SHA
c28f26b28e56ebe78fcf955dcac5e837c99111405ee1fb17071304666090bdf9.
Unchanged bare original-stream regression,573core/511domain, full live
G2deform/unitarity trace, sixunitarycontrols,273+11no-loss,4script/4block
and finalintegrity allPASS. PSp and GL2fail on this isolated source by
design. PSp3849212 remains separate pending gate; exactunion with FINAL
ParamPol/FPP/GL2/G2 and any accepted PSp repair is still required.

## PSp before FINAL; overlap-shift candidate submitted — 2026-09-29

Before3848989 FINAL PSP4_HISTORY_FAILS_TWO_CONTROLS_PASS_BEFORE_REPAIR,
report math_psp4_cayley_before_2026_09_29.json SHA
29e175200bf14d56054d9b5ccac49de135d93d50ccfb8d067cd195409e7ee51b.
575inventory/production unchanged/final integrity; cold and triple-warm
controlsPASS, full bare KL-historyFAIL with exactly3Cayley Runtime messages
after successful setup/recovery. No compilation/setup failure substituted.
Only then changed RepTable overlap merge: ALWAYS compute make_relative_to
and apply shift then transform, even when stored/query locators are equal.
Original repr.cpp1749-1754/347-358 requires this. Vec<BlockModifier> replaces
the optional modifiers; canonical-key dedup, downward links, exact query
round-trips, locks/commit validation and all tests/goldens stay unchanged.

rep_table.rs before3b25d3120b3d69dcca456e1c9eddbaa63c54ccd12551a6cf5398f9f18437a57a,
after5c75d719137eab7aab4a47bf3a1b829381a937380e3e18558a5eec9702d1eed7.
After3849212 SUBMITTED in atlas-psp4-cayley-build-20260929.I3OP37I0,
pin92b42f8c74e142007be6709c6d80f73d6cb799d62f229a664713f9bbc9662f40.
Require unchanged3after/575core/511domain/CLI, all5complete PSp histories
(bare3fullstdout,2script full mathematical tails), six accepted unitary
companions,273+11no-loss/4script/4block histories/final integrity. GL2 stays
separate discovery on this source; G2cross/GL2/FPP/ParamPol NOT included.
No after acceptance yet. Receipt math_psp4_cayley_build_submission_2026_09_29.json.
Archive atlas_psp4_cayley_build_inputs_20260929.tgz
SHAf8581d33dc5f600cddf55009954ca2fd226387a891e44ca0ecbb8ebc391f7677;
scratch /tmp/atlas-psp4-before.7SqsOlen/build-package. Verified bridge/merge/
tail helpers explicitly included, frozenchecker16. Local583session SHA
3659f9cd4f29010c22b709325030e2ba675947f874c134fd9247223c37c65c45
contains other candidate tests and is NOT the isolated575gate inventory.

NEXT collect G23848845 and PSp3849212 in batches; full408fat/review still
live/pending. Then verify the exact union of FINAL ParamPol575/FPP574/
GL2574 plus whichever G2/PSp repair actually passes, retaining every parent's
full-stream gates before another broad survey. Do not claim local union
mathematical acceptance from separate candidate results. No local builds,
tests, commits or pushes in this turn; git diff --check passes.

Last live G23848845 check at20+minutes had progressed through descents and
all required unitary controls into273capture (no report.json yet). Its
unchanged bare regression and full573core already passed. Since the driver
reaches273capture only after full G2trace equality, that consumer gate is
past, but final no-loss/source integrity is still required. Do not duplicate.

## PSp R2 FINAL reproducer; before proof submitted — 2026-09-29

3848939 FINAL report29f190e06059e450af01c092fb3726d7e11658cefc86adb9ad62fae7a5abf11a:
all5originalACCEPT; two controls fully match; bare all-KL-term and isolated
unitary prewarm histories both fail3Cayley Runtime calls. Direct uncached
print_partial_block(target) remains correct after pooled calls fail. Shared
history is therefore necessary in the regression; do not substitute cold.

Three complete original goldens and session tests now frozen in before-only
3848989, atlas-psp4-cayley-before-20260929.T1pNGD8I,
pin8c748dadb5ff47e5f9c3f59b4d9570bcf406b1f1f00144cf5ffb07fa0edcf9f4.
Isolated575inventory, require2controlsPASS/1historyFAIL with EXACT3downward
Cayley Runtime messages after READY/setup/recovery. No runtime edits.
Sessionc746753c1dbcd6367fe5d179afecf543306c2099d48f21b5688889f969471473;
localunion now583, not a verified source. Archive
atlas_psp4_cayley_before_inputs_20260929.tgz
SHA285591846c5035bf084e4cf08c860265c84298fcf7e84f90813348748854762f;
scratch /tmp/atlas-psp4-before.7SqsOlen. Before receipt is in tests/reference/hpc.

Concrete source candidate: overlapping-block lookup skips modifier/shift if
record.locator==reduced.locator. Original ALWAYS calls make_relative_to;
same relative Weyl attitude still permits nonzero integral-orthogonal shift.
The existing cache-hit located() path already handles that. After before
proof, smallest candidate is compute every overlap modifier and transport
all rows via shift then transform, retaining canonical-key dedup/guards.
After harness math_psp4_cayley_build/stage is drafted but NOT submitted;
it requires575core/511domain/all5full PSp histories/sixaccepted companions/
273+11no-loss/4script/4block/integrity. Import verified bridge/merge/tail
helpers explicitly when packaging; direct-capture parents do not have them.

## PSp cold/triple-warm controls PASS; full history still fails — 2026-09-29

3848885 FINAL reportc9262e5749d7d49d3c96e143e8a2bd081f08a2ff34ec552c7824592992b7f0f2.
Bare cold and triple-reducibility warm programs BOTH accept and match whole
original stdout/stderr. This disproves a simple target-parameter or short
warm-prefix failure. Unchanged full script trace still fails and exactly
reproduces3848811raw hashes. No PSp runtime edit is justified yet.
New indexed slice docs/slices/psp4_cayley_history_2026-09-29.md records the
actual adjoint C2datum and two-row direct/three-row pooled control outputs.

R2five-case3848939 submitted in atlas-merged-diagnostics-20260929.KiDhhRum,
probe5d8d4f9b93f56c5f2d4690168d2bb6036688457182aea96f8967348b61ff35b1.
Retains all3sources and adds bare KL all-term unit/half/triple history and
isolated is_unitary(trivial) prewarming. Original acceptance pending.
Checker16/master273/full408 unchanged. Correct archive
atlas_psp4_cayley_r2_inputs_20260929.tgz SHA9d854119c9ae65dca3638f857dbabeda2b98804396d950122386903805c12ed2;
scratch /tmp/atlas-psp4-cayley.yM8oaTdn/package-r2.

## GL2 after FINAL PASS; PSp cold/warm reduction submitted — 2026-09-29

GL23848801 FINAL GL2_TWISTED_KL_GATES_PASS_BROAD_MATH_PENDING. Report
math_gl2_build_2026_09_29.json SHA
9c0dc173694b4ee168d1113a156b077b4ffffc0a0054023e408511a7fefa2eed.
Both unchanged regressions,574core/511domain/CLI release, all6GL2 streams
(including preserved old rejection inputs and both rank errors), ALL7full
unitarity tails now pass. GL2final companion changes RuntimeFAIL -> ACCEPT
and full math tail equal; six other groups remain equal.273+11no positive
loss,4script/4block histories and final source/binary/input integrity pass.
Loader-prefix stdout still differs. This is isolatedGL2574, NOT union with
separately verifiedParamPol575/FPP574 or pendingG2573. G23848845 live log
passes its unchanged full bare regression; full after gates remain pending.

PSp three-case3848885 SUBMITTED in atlas-merged-diagnostics-20260929.aWvwv5OY,
probefaa1706e2f4f328d512d0583e4c5f2a4c9bbd3acb80f3b033187094d5f063e94.
Separate psp4_cayley_catalog.json uses exact572release3847662. Bare cold and
triple-reducibility warm histories share groups.at's actual PSp datum:
rootsI2/coroot columns[[2,-2],[-1,2]], false numbering; x6lambda-rho[0,0],
nu[28,21]/6. Every deformation/direct and pooled partial block/KL output and
separate recovery stays. Unchanged full script trace is the third control.
Original acceptance is still provisional. No PSp runtime edit. Checker16,
master273/full408 unchanged; older frozen15/14 stages MUST NOT be modified.
Receipt math_psp4_cayley_submission_2026_09_29.json. Archive
atlas_psp4_cayley_inputs_20260929.tgz
SHA3c7bd51e3ed582a606e3746f4b6fd57a65840b8e27ee1087230ac32152391797;
scratch /tmp/atlas-psp4-cayley.yM8oaTdn.

## G2 before FINAL; one-call repair after gate submitted — 2026-09-29

Before3848829 FINAL G2_CROSS_REGRESSION_FAILS_BEFORE_REPAIR, report
math_deform_cross_before_2026_09_29.json SHA
c07a6902ad6db78b01e4b50b3d0538135e5ab3453e07b7e631611faa8e6866c5.
573inventory; one EXACT common-cross Runtime failure after all setup and
recovery markers; production unchanged and source integrity rechecked.
Only then changed common_deformation_terms to explicit
BlockTopology::cross(block,z,s), preserving the required-link invariant.
deform.rs beforecf33464ae2222ad2f0f40cd7241ea44282e68033683acb94130e543aac7ac712,
after3e5dc2c99fcde6f21b4a0dc341243dfe579250326d51c5ab553a0a1f51d91713.
No singular-modifier, Cayley, golden or test changes.

After3848845 SUBMITTED in atlas-deform-cross-build-20260929.DzCDNnd6,
pin5e090a6f848faa2e63a8575bf0b337bc88f98ae6eaa8a85e1230a1f79ec01218.
Require unchanged1after/573core/511domain/CLI, full bare G2 original stream
AND complete DEFORM_TRACE_INPUT tail including live recursive unitary trace;
PSp remains separate discovery, never an assumed repair. Keep six accepted
unitary companions,273+11no-loss/full4script/full4block histories and final
integrity. Local580union contains other independent fixes; gate uses ONLY
verified572 plus one G2test/cross-call change. No after acceptance yet.
Receipt math_deform_cross_build_submission_2026_09_29.json. Correct archive
atlas_deform_cross_build_r2_inputs_20260929.tgz
SHA27b2364968b9191598b3c89ebc5682a92b9f7fa63e6372b2aed8438580e5db00;
scratch /tmp/atlas-g2-cross-before.UBDQWnX5/build-package.

First staging ORWJAW9d stopped BEFORE sbatch: inherited direct-capture
harness has old bridge5f823851 and no math_verified_merge/strict-tail helper.
Keep stage immutable. R2 explicitly imports verified bridge861b1e48,
merge82e3c59c and tail93823672; checker15 unchanged. Also corrected G2trace
marker to DEFORM_TRACE_INPUT (bare uses DEFORM_CROSS_INPUT) before any job
was submitted. Do not resubmit the first archive. No canceled or duplicate job.

Next: collect3848845/GL23848801 and fullfat review in batches; reduce PSp
x6lambda[4,3]/2,nu[28,21]/6 into separately named bare cold/warm histories.
PSp uses groups.at's actual adjoint C2datum/root coordinates; do not silently
substitute a differently numbered Sp datum. Required downward closure stays.

## FPP four complex-group consumers FINAL PASS — 2026-09-29

R3job3848729 FINAL FPP_ORDER_GATES_PASS_BROAD_MATH_PENDING, report
math_fpp_build_r3_2026_09_29.json SHA
445e22c1d7ad89b82c299079e5fc971b3aa939a8ec1bebe770c329f7c4cbc0f5.
Two unchanged after regressions/574core/511domain/CLI release pass, all
4product/negative streams pass, all4complex FPP consumers (SL3C/PSL3C/
Sp4C/G2complex) change from exact572MISMATCH to full mathematical tail MATCH.
No273+11positive losses;4script/4block/six unitary companion tails retained;
GL2 still fails. Source/binary/input integrity FINAL. Loader-prefix stdout
still differs. IsolatedFPP574 only; never imply local580union/full408 or a
speedup. See indexed fpp_product_order slice. Older LIVE notes are superseded.

## G2/PSp deformation capture FINAL; G2 regression before gate — 2026-09-29

Capture3848811 FINAL: all three complete original programs ACCEPT, Rust
rejects all three at Runtime. Report math_deform_descents_2026_09_29.json
SHA9c8961c59c195a72d44bdb4786305dfe810180b1061033dd9543c708e4aa5d91.
G2bare fails ONLY at factor1/3, input x7lambda[1,1],nu[0,1]/2; earlier
factors1,7/9,5/9 and the later direct original parameter survive. G2trace
retains three cross errors; PSptrace retains a distinct required downward
Cayley error at x6lambda[4,3]/2,nu[28,21]/6 in the triple-scale history.
Do not infer PSp's cause from the G2 index-order defect candidate.

Untouched1794byte G2original golden copied into the test library, SHA
d220adfdfc8847bdc94d37586ca3d63249868e3253d5aa1e3469c67c007b61ea.
New ordinary_deform_g2_cross_original preserves the entire stream including
Value:[(),(),(),()], both recoveries and repeated direct output. Before
proof requires exact Runtime cross error after valid setup/recovery markers;
no setup/compile failure counts. Production deform.rs is still unchanged.
Before3848829 SUBMITTED in atlas-deform-cross-before-20260929.NE4iaBEc,
pin7db6750305675f2ce940175fa31b757a1787a04d1c800234724cb8c2466ec467.
Isolated verified572+1=573 session8054a36f32e918622ca963124337bd1ae7bf324e27036e774dbf5242e489c68e,
NOT the local580union. Receipt math_deform_cross_before_submission_2026_09_29.json.
Archive atlas_deform_cross_before_inputs_20260929.tgz
SHAb48493b88b64c1966e4e98949ae520793600c809e1aaa1e1249c891b0fea0e0b;
scratch /tmp/atlas-g2-cross-before.UBDQWnX5. Frozen checker15.

## CPU187 FINAL; GL2 repair submitted; ordinary-deform frontier — 2026-09-29

CPUreview3848054 FINAL on exact572release3847662. Report
math_merged_survey_cpu_review_2026_09_29.json SHA
b5bbd0faff56bf503c6d397b3e65d18e6727a65105a4ed6569716ab09351c867.
187cases=166math matches/6rejection-category matches/5Rust failures/
9original failures/1original timeout; reviewer63checks and artifacts rehashed.
All22form metadata/KGB/KLV/AV-ann cases match. On the SAME132inputs as old
CPUreview3846279,19new math matches and no losses:17AV-ann, PSp4R KLV,
Sp4R unitarity. This is not full408; fat221/review3848053 remain pending.
Unresolved Rust5 are G2_unitarity/G2split_form_unitarity (same complex-cross
invariant), SL3R/GL2Runitarity (Inexact halving), PSp4Runitarity (required
Cayley descent absent). D4unitarity original times out600s while Rust also
fails complex-cross; preserve both, not a successful speed ratio. Nine
original-rejected inputs remain unvalidated; never count them as RustPASS.

GL2before3848755 FINAL TWO_GL2_STREAM_REGRESSIONS_FAIL_BEFORE_REPAIR,
report math_gl2_before_2026_09_29.json SHA
5bc167022e310e8db0bea6b122eea6f52bb4c03e26491328b126caab169cad63.
Inventory574, exactly2stream assertion failures after READY/setup/two rank
diagnostics, production unchanged and final integrity. Only then runtime
changed: distinguished_twisted_kl_terms always RepTable.lookup, retains
prepared query/actual row/modifier, builds tuned partial extended block, flags
singular coroots in DIRECT bm.simp_int order using exact i128pairings. It
does NOT apply simple_pi twice. External-delta and twisted_deform paths,
all original goldens/tests and coefficient-halving assertions unchanged.
After3848801 SUBMITTED in atlas-gl2-build-20260929.2OcYoCM1, pin
a31ff47f22a7ddb700828ba999044525d06bff6d1662381857d32150386d02b1.
Receipt math_gl2_build_submission_2026_09_29.json. Isolated574after domain
8cf0268897c840307409301d26fc6c978d58469e548df1801787527f71de68ad,
not local579union3cae38fe953009bed4854d2cb2316480f770fc2965bdd725f379200462f0f2b1.
Require2after/574core/511domain/CLI,6GL2probes with FULL output and negatives,
ALL7complete unitary tails,273+11no loss/4script/4block histories and final
source integrity. No after acceptance yet. Archive atlas_gl2_build_inputs_20260929.tgz
SHA81021608bc7daf51a05dea6064f3f2778afd12bedf0d5cb495e3f656385e305a;
scratch /tmp/atlas-gl2-build.c8i8YSSa. Frozen checker14, not newer15.

FPP3848729 last confirmed LIVE after all4complex consumer gates and into
273capture. No duplicate, no final acceptance yet. Previous failed3848594
still retained; only the fresh reader/checker fixed the harness omission.

Next ordinary-deform source audit found a concrete index-order seam:
deform.rs common_deformation_terms calls concrete PartialBlock.cross(z,s),
but inherent cross expects(s,z); trait BlockTopology::cross expects(z,s).
Inherent method wins. That explains a plausible absent-link error whenever
row>=rank, but original-backed before/after proof is still required. Its
singular flags also ignore relative modifier attitude; investigate separately,
not by weakening required complex/Cayley descents. No ordinary-deform edit.
New separate3case deform_descents_catalog: bare G2x7lambda-rho[0,0],nu[0,3]/2,
plus complete G2/PSp4R oriented-KL terms/centers/reducibility and LIVE
d_verbose:=true traces. Unit/half/triple top-level recoveries retained.
Intent provisional until original capture; global checker15,273/408unchanged.
Capture3848811 SUBMITTED in atlas-merged-diagnostics-20260929.AQDKaZo9,
probe75bba0d2c01066ac155eda95f46032b27bbe6171febcec03ce28f12935efbb11.
Receipt math_deform_descents_submission_2026_09_29.json. Archive
atlas_deform_descents_inputs_20260929.tgz SHAed116dc14e0bc5274afe07ddd5e5bb74d318a92edffd5814d763b29c578fb5ec;
scratch /tmp/atlas-deform-descents.Lk8Q99qd.

## ParamPol FINAL; FPP harness rerun; bare GL2 regression captured — 2026-09-29

ParamPol3847661 FINAL PARAMPOL_ORDER_GATES_PASS_BROAD_MATH_PENDING,
report math_parampol_build_2026_09_29.json SHA
ee0281513cdde37713399ceb3194da57bfa4c789e6552b53e5061b32236d5f8f.
All3unchanged regressions/575core/511domain/CLI pass,273+11 no lost positives,
4script/4block tails preserved. Both complete valid order tails, all3cycle
traces and the PHI_ORBITS full discovery tail now match original with equal
stderr; full stdout still differs in loader declarations. Six unitarity
companions match; GL2R still fails. This is isolated575source, not localunion.

FPP3848594 FINAL FAIL, report math_fpp_build_2026_09_29.json SHA
afd4ab62d1c4ca9ac11b4ff73bcb1c08655d6e04ff24b455a16e9c4722577d36.
Two after regressions,574core/511domain/CLI and all4product stream gates pass;
then inherited OLD math_suite.py omits real_form_grids, expanding116not408.
No complex FPP consumers or later no-loss gates ran. This is a harness
failure, NOT a mathematical regression and NOT final FPP acceptance.
Fresh3848729 SUBMITTED in atlas-fpp-build-20260929.YEGACFZ3,
pin9d75cec267f92c89790c0cf619c372b3b079e70333249d51653dc8cd069264d0.
Receipt math_fpp_build_r3_submission_2026_09_29.json. Same574runtime/tests/
goldens. Imports readerb6ad5e37 and checkere74b6bd8 with exact survey catalog;
three catalog checks precede compilation. Keep failed stage immutable.
Archive atlas_fpp_build_r3_fixed_inputs_20260929.tgz SHAdcd3b7e4ca2cd8681a0b89d036b079731acef7a50a31beef477e0417833b0f75.
An earlier unsubmitted r3 archive has stale overrides after local node was
unavailable; NEVER submit it. Only the explicitly named fixed archive is valid.

GL2R3848639 FINAL: old bare probes fail setup at script-only -id_mat(2).
Retain both sources/error streams, not numeric goldens. Explicit matrix
companions in6case capture3848669 FINAL (dqSHcEKJ), probe
7472c24c41aee26ca1f79d6ddfbf1514ec2a303af9941e0e76008fbc0ec936b1,
report math_gl2_twisted_kl_r3_2026_09_29.json SHA
d3881958b0e1abf20858cfe8390d0d137887163a7eb557870ae90d6daa596f72.
Original accepts the entire bare positive; Rust accepts but loses singular
twisted1*p in BOTH numberings and cold/repeated histories. All other positive
lines match. Recovery has two intended Runtime rank errors in both engines,
then the same wrong singular sum. Original goldens copied unchanged:
positive6c60372b41f000318af411efe6c94168e856f5c993ce910dfd14c48409898e8e,
negative8577306f76f5ca98f198b0ad1811766cedd08430e547be73fb2a477163fe07ed.
Two session::tests::gl2_twisted_kl_* units now preserve complete streams,
both exact rank messages and recovery. No GL2 runtime edit. Before-only
package isolates verified572+2tests=574, session6fa4fdbd8e97186f5a8a0978b6b7b543b474bcbb2410bacfabaabe8a319e9e2e;
local union579 also has FPP/ParamPol and is not verified. GL2before3848755
SUBMITTED in atlas-gl2-before-20260929.PUCTg5Bf, pin
453fcb42858c47694f92e424d86b6da651589ecab4bcfa040f14d7170edc37a3.
Receipt math_gl2_before_submission_2026_09_29.json. Archive
atlas_gl2_before_inputs_20260929.tgz SHA169d8dbb15ac1a171a1cb82bfc73a486ea6fe97a12f78767ba6f2007828f21a2;
scratch /tmp/atlas-gl2-before.6nXimLoA. Must prove2actual assertions fail
after setup before repairing distinguished twisted-KL common-block lookup.
Full408 exact572 arrays3848051/3848052 and reviews3848053/3848054 remain live.

Local session.rs after both newGL2units SHA
73a5fed688ece28950c580e299ae79318967f97d496b493608feb8a8674ebcff;
domain runtime remains d3eedf7414c65e45725095b1dc61af20698e2ad3fde6990c45225561307c2e76.
No GL2algorithm edit, commits, pushes or local tests/builds in this turn.

Next GL2 adapter audit: original repr.cpp2499-2553 always normalises and
performs Rep_table.lookup, including full integral gamma. Rust's Full branch
still builds a dual-quasisplit BlockGraph; twisted_block_index reconstructs
using sr.x/lambda/gamma independently of candidate z, effectively first-x.
Keep distinguished overload separate from external-delta full common-block
and twisted_deform paths until their own evidence. Extended singular flags
must follow DIRECT bm.simp_int order (original explicitly warns against
block.singular). Existing located_singular_flags applies simple_pi and returns
PARENT-generator order, so it cannot be reused unchanged for cofolded ExtBlock.
These are source-backed candidate boundaries, not yet a verified root-cause fix.

The older entries below are chronological snapshots, not current status.

## Full408 launched; FPP after gate; singular GL2R reduced — 2026-09-29

LATEST live FPP3848594 after-fpp log has2PASS/0FAIL/572filtered, with the
unchanged full positive/negative-survivor goldens. Full574core/511domain,
release/consumer/no-loss gates remain LIVE; no final FPP acceptance yet.
ParamPol3847661 last confirmed LIVE at25minutes, already past273capture/
11diagnostics/4block gates and collecting unitarity; no duplicate/no waiver.

GL2R2companion capture3848639 SUBMITTED in
atlas-merged-diagnostics-20260929.kPVpndkB, probe
5c6e858619595f3f8da7091e614410dec0085e67b55655d261dbb9688540295a.
Receipt math_gl2_twisted_kl_r2_submission_2026_09_29.json. Catalog4 retains
the two original-accepted script probes and adds bare-core GL2 root/coroot
mat:[[1,-1]], split negativeidentity/quasisplit form, x1lambda_minus_rho[0,0],
four nu values and both numberings; separate rejected nu-size companion
retains complete recovery and a valid singular sum. These two new intents
are provisional pending original capture. No GL2runtime repair or unit
golden yet. Checker remains14, all older inputs retained, master273/408
unchanged. Archive6d9d10295de3f92959bf50ab6202de4d852fffbc8a5abcb18fd2f673644babe3
at /public/home/majj/atlas_gl2_kl_r2_inputs_20260929.tgz; scratch
/tmp/atlas-gl2-kl.7xTfUvqm/package-r2. This uses fresh merged572release,
not either unverified ordering candidate or local577union.

Fresh release3847662 FINAL, report math_merged_survey_build_2026_09_29.json
SHAe1cd61a05e0d76c4bddd2938841aaa719b9a89ab941bca32a00b063a7d97a423.
1499Rust/606original source files rehashed; Rust c18e20d4e67a88c878dcd79a1879777ddf2b97459d01019e99a82d64b198f156,
original d4f0f3dc3a82102529aa2ec562db0601e99b368dee25d5539ca52dae2fd37a5a.
Together with FINAL63-check preflight3847663, frozen launch helper submitted
all408 unchanged cases: CPU3848051(187cases,%4,600s/6GiB child),
fat3848052(221cases,%2,1200s/24GiB child), independent full review3848053
and CPU review3848054 afterany. Same mwbojUD8stage/suite9828d7c9; updated
math_merged_survey_submission receipt47c5c393c3448d7906750aa92cd59b708e2c0ad882a7540b330ccdb1a64315e3.
These are RUNNING/PENDING, not new mathematical acceptance. No duplicate.
All408 source remains exact572, excludes both newer ordering candidates.

FPPbefore3847713 FINAL TWO_FPP_STREAM_REGRESSIONS_FAIL_BEFORE_REPAIR,
reportd4167064b6df432e5a3961f0ee51fd7b7e12a5855ba6de655eb2e873f3bc4421.
574inventory; exact2assertion failures after READY/setup/4diagnostic checks;
no runtime changes; source integrity checked. Root component repair now
adds only largest-RootNbr ordering after unchanged union-find, based on
original rootdata.cpp1514-1537's append-on-touch algorithm. Tests unchanged.

After FPP3848594 SUBMITTED in atlas-fpp-build-20260929.b0qeDUZg, pin
93b827a0f94d55e0f2425fe5ee85e9283ec813fed617e2b7374a19ba26ad4d4b.
Receipt math_fpp_build_submission_2026_09_29.json. Isolated574source AFTER
domain file d08d68f65f115efe0c0ec360c6c0ce513042cef3ebd893a48fe21fb5c06dbf3f.
No ParamPol patch in that source. Local combined577domain file d3eedf7414c65e45725095b1dc61af20698e2ad3fde6990c45225561307c2e76
is NOT this candidate and NOT verified. Gates require2after/574core/511domain,
CLI,4FPPprobes including full negative recovery,4actual complex-group FPP
inputs from unchanged408,273+11no lost positives,4script/4block histories,
and no lost six valid unitarity tails. No FPP after acceptance yet.
Archive /public/home/majj/atlas_fpp_build_r2_inputs_20260929.tgz SHA
e51780ebc558b843302de565335e2966ec22575363a58226203ba60df47733fd;
scratch /tmp/atlas-fpp-build.glANSQge. First staging QzbyEM7z failed BEFORE
sbatch/pin because reduced ancestor lacks broad tests/math/catalog.json.
No job was created. Corrected stager explicitly imports unchanged catalog
and templates from survey909e6390 before freezing input manifest. Runtime,
tests and expected outputs unchanged. Preserve failed stage; do not run it.

GL2R2capture3848183 FINAL in atlas-merged-diagnostics-20260929.11HtgJj3,
probe226e1af866c2a688ce19ab7e6c2045ddaba24415fde3036eaab4e9b9b8b9e8bd,
report math_gl2_twisted_kl_2026_09_29.json SHA
317fec1c70a61f5c24563da8a8d76fce39c3c6c4bc462b2f4f054212c35ef5f0.
Both original programs ACCEPT; merged572Rust fails Inexact halving in both
ordinary-first and cold twisted-first histories. At scale0(x1,lambda[1,-1]/2,
nu0), ordinary KL is1*p in both, but twisted KL is1*p original versus EMPTY
Rust. Hence combined coefficient2 becomes1+s, whose halving remainder1+s
correctly triggers the assertion. Half/unit/triple scale stage outputs match
on inspection; complete program remains FAILED. This is no longer a vague
halving hypothesis: fix upstream twisted singular-column loss, not division.
Keep all coefficients/twist/orientation/recovery, and obtain bare-core original
regressions before runtime repair. Source with_integral_block uses common
partial blocks only for ProperSubsystem; Full still builds BlockGraph while
original Rep_table::twisted_KL_column_at_s always lookup(sr,bm). That is a
next boundary to audit, not yet the proven root cause. See unitarity slice.
Raw reviewed stage streams /tmp/atlas-gl2-kl.7xTfUvqm; source/harness14 are
separate from frozen FPP13, ParamPol12, foundation10 and408preflight63.

## Merged foundation FINAL; new consumer jobs — 2026-09-29

LATEST independent FPP before-only gate3847713 SUBMITTED:
`/public/home/majj/atlas-fpp-before-20260929.9zMPfKEd`, pin
8d08f2045a1238f564e3cb87c72ff4b8c75245b5c873ee7ad9468cc0945f36e8.
Receipt math_fpp_before_submission_2026_09_29.json. Two new session tests
preserve full original3847592positive89812-byte stdout and rejected400-byte
survivor stdout, all4Runtime messages, witnesses/multiplicities/recovery.
Original goldens6a5e2c92/781a3bd5 are copied unchanged into tests/math.
Source572+2tests=574 is isolated from ParamPol575; local union now577 but
has NO acceptance. session.rs test-only SHA
2d490f46b2eb51a8846cd8087fee876d767335a994ab94a2d73dd20d1cdaae16.
No FPP runtime patch; require two actual stream-assertion before failures
after READY markers and successful negative diagnostics. Archive7af70aa3
at /public/home/majj/atlas_fpp_before_inputs_20260929.tgz; scratch
/tmp/atlas-fpp-r2-review.3N7fVITP holds frozen package and raw review streams.

408preflight3847663 FINAL HARNESS_PASS_NOT_MATH_VERIFIED, unchanged inputs,
reportd45c7e1ec5560e2ea64feaeb8e374d3604431bc1962b4c170dedf960f397e7be.
Fresh build3847662 remains live at last check; do not duplicate it. The
frozen launch helper also requires its exact FINAL build report hash.
ParamPol3847661 live log has3after-order tests PASS; full gate still live,
so do not claim the comparator/consumer repair verified yet.

Merged R4job3847093 FINAL status MERGED_FOUNDATION_GATES_PASS_BROAD_MATH_PENDING.
Report math_verified_merge_r4_2026_09_29.json SHA
9a38426f9749f4ff7bf2b42443d80927f7a88bf6a0377ee59b7336b93e07124d.
All511domain/572core, CLI,4marker harness checks,273capture and11diagnostics
pass with no lost positives. All4script tails and4block histories match
the original from their declared markers through the entire remaining stream;
full stdout still differs in loader declarations.1499source files rehashed.
Six of seven final-constituent unitarity companions now match completely:
SL2R,SL2C,PSL2C,SU21,PSU21,Sp11. GL2R's trivial anchor is restored but the
whole program still rejects Inexact halving. No blanket unitarity acceptance.

Submitted independent fresh standard-release build3847662/preflight3847663:
`/public/home/majj/atlas-merged-math-20260929.mwbojUD8`, suite pin
9828d7c9c051724bebff5581c1524204ff7490fbc795432cbfd54b4e9ade7446.
Receipt math_merged_survey_submission_2026_09_29.json. All408inputs unchanged;
arrays are NOT submitted yet. Collect FINAL build/preflight hashes, then use
the frozen hpc/stage_merged_math.py launch. This is exact572foundation, NOT
the separate575ordering candidate. Do not wait on live jobs or duplicate them.

ParamPol after gate3847661 SUBMITTED, stage
`/public/home/majj/atlas-parampol-build-20260929.GsZiylne`, pin
69b546b03477fcd93ce4db31fdf8f88c5b967d9b4e8f4628eafd7dc810dcecc7.
Receipt math_parampol_build_submission_2026_09_29.json. It pins the accepted
foundation and verified3before failures;575core/511domain and full ordering/
cycle/prerequisite consumers remain required. No after acceptance yet.

FPP R2job3847592 FINAL in atlas-real-form-diagnostics-20260929.FRmnNF5f,
probe e1eabd1c01cf10f1cbca7f3418b61fda7fb05c2a09a43564a65654b129fdea30,
report math_fpp_product_r2_2026_09_29.json SHA
e18e46143afb4da6909a1d46d65eee5e3f7ade294f66c7a4d17640113a5cb000.
Checker13 passes. Valid8type/both-numbering/SC+ad product program ACCEPTS
wholly in both engines (empty stderr), full89812-byte stdout differs.
Leading T1.A2 isolation: SC accepts in both preferences; original adjoint
rejects Dependent lattice generators in BOTH, followed by undefined-name
errors and retained recovery. Rust accepts all four. This is distinct from
FPP ordering; do not add a blanket torus rejection. All four captured inputs
remain. No FPP runtime change yet; next freeze original-backed whole-stream
regressions and prove before failures. See indexed fpp_product_order slice.

Older entries below are chronological snapshots, superseded by this section.

## FPP original setup failure retained; split companions — 2026-09-29

FPP2capture3847534 FINAL report
07d9633a3d4e57713561899f679890d729fecb9a3e351a208c11c3caf9d552ae.
Original positive-intent compound loop rejects Dependent lattice generators
when constructing the T1.A2pair, after A2.T1/true finishes; Rust accepts.
This is NOT a full positive oracle or a confirmed FPP enumeration failure.
Negative companion preserves all four intended rank/non-alcove errors and
recovery in both engines (prose differs; complete surviving vectors differ).

Keep both exact old files. Added fpp_product_order_valid excludes only the
separately retained leading-torus constructor case (8types x2numberings
x2isogenies, includes A2.T1), and fpp_leading_torus separates all four
SC/adjoint true/false constructors into top-level commands. Catalog4,
checker13; newest R2capture receipt has exact pin. No FPP runtime change.
Ordinary ordering candidate still frozen12checker and uploaded archive
b90f883c; do not copy these later supplemental fixtures into it.

## Ordering before gate FINAL; comparator candidate prepared — 2026-09-29

Before-only3847509 FINAL, report
8607362847086c36d5ca30798f219b5c6bc704c145136d5923942a6ee63ab8ef.
Inventory575passes; all3expected order assertions fail after successful
setup, with no runtime changes and final source integrity. Not a compile
failure or failed test setup. Read math_parampol_before_2026_09_29.json.

Comparator runtime now changes ONLY sort_parampol_terms: numeric torsion
high-to-low, gamma descending, i128cross products. Three tests unchanged.
After-fileSHAc6a7207fe9853a9f2ab494dc7a081c298b39daee3db304181a206a9881c5667e;
regression-only before SHAeddcbf4e. Candidate archive prepared/uploaded at
`/public/home/majj/atlas_parampol_build_inputs_20260929.tgz`, SHA
b90f883cd9341de6db69479c2b1c3216ca1232049f8c0b974cb2f685ab1cad79.
Scratch `/tmp/atlas-parampol-build.EoebG0Ei`. NOT submitted at this entry:
stager requires FINAL merged3847093report hash first. It then requires
3afterpass/511domain/575core/CLI,273prerequisites/no lost positives,11diagnostics,
4block histories,4ordering probes (retained errors included),3complete cycle
traces; fullPhi and7unitarity results remain explicit discovery, not waived.
Frozen after harness has12generic tests from ordering3847089. Do NOT copy
later local13checker/FPPcatalog into that archive or live stage.

Merged3847093 LIVE at last check: full511domain/572core/CLI release complete,
273capture done,11diagnostics ongoing. No duplicate. Standard408stager also
prepared/uploaded as `/public/home/majj/atlas_stage_merged_math_20260929.py`,
SHA77f14b9de6d7746a07ffb0a8f794c0670b668e8574c7eda699c9f00a44c67a31;
must use accepted FINALfoundation, then fresh build/63-check preflight.

While those run, FPPproduct reductions are separate: old complex SL3C/
PSL3C/Sp4C/G2complex raw reports downloaded and hash checked at
`/tmp/atlas-fpp-products.NbQng82P`. SL3C's first two gamma outputs agree;
third differs by two product factors' order, including Weyl/shift witnesses.
Original components() appends each newly touched merged component; final
order is ascending MAX RootNbr. Rust union-find orders by first/min root.
No FPP runtime change. New fpp_product_catalog has one9type x2numbering
x2isogeny positive probe and one complete rejection/recovery companion;
provisional until original capture. Local checker13, master273/math408
unchanged. See fpp_product_order slice and newest capture receipt.
FPP2capture3847534 SUBMITTED in atlas-real-form-diagnostics-20260929.jAyC28Pj,
pin5b529b3a41c4fcba9360bb44712b9c7d41594ad0940de701847db68268d9bb90,
receipt math_fpp_product_submission_2026_09_29.json. Original acceptance
is provisional; no FPP runtime patch. Archive54e8cfd3ddacd715ce4add80e63133ab8610ea9179553c77aaa93de488f5d785.

## Current jobs: merged R4 and ParamPol before-regressions — 2026-09-29

CURRENT merged gate3847093 SUBMITTED, stage
`/public/home/majj/atlas-verified-merge-20260929.KolaCCOF`,
pin797acb3330c05ce114c0b2551c8606d17b15ce07021ccad1478048c9b7951555.
Receipt math_verified_merge_r4_submission_2026_09_29.json. At last check
four new marker harness tests pass; full gate is running. No runtime or
mathematical-fixture changes from R2. R3job3847086 exited at module import
because its archive omitted math_pos_neg_consumer.py; no report/tests/build.
Preserved raw log math_verified_merge_r3_import_failure_2026_09_29.log,
SHA5a40a9c883e95ed77aa72c7a6a9d314a0ef3510143e1be030bb63ed8fb3f461b.
Stager now inherits COMPLETE pinned R2harness, verifies helper93823672,
bridge861b1e48 and new checker e6b77bcf. R3is terminal, never poll it again.

Ordering discovery3847089 FINAL, report
4371cee054c6eba8c8e6b8338b38a1a9a8ac392b75b2801880232f02f4e9e200:
both explicit-Split positive probes wholly accepted in original/Rust,
both old integer-list rejections retained. Whole output confirms A2/T3
numeric torsion order and descending fractional gamma are wrong in Rust.
See new `slices/parampol_order_2026-09-29.md` for coefficients/raw hashes.

Three original-backed unit regressions added to domain_builtins tests only;
production prefix UNCHANGED. New local file SHA
eddcbf4e1582a4fbf221073c97cf2917d2eeaeb32ca9ff27155fdac8ec3c7ce2;
old production/test baseline00ebe48b. Core inventory becomes575, but frozen
mergeR4source remains572. `hpc/math_parampol_before.py` freezes exact R2
source plus these3tests, requires unchanged runtime and all3assertion
failures (not setup/compile failures),575inventory and final source rehash.
Read math_parampol_before submission receipt. No comparator runtime edit,
no after acceptance. Before-only source eligibility is not R2math acceptance.
Before-only job3847509 SUBMITTED in
`/public/home/majj/atlas-parampol-before-20260929.apavwghR`,
pinb60bb0680b828acbb4accf2ad64cb5dd680b273c0777c5bf5f4f6e6d3a5a6b34.
Scratch archive `/tmp/atlas-parampol-regression.FCYdp8YK/inputs.tgz`, SHA
83822ecb7b7fc45903acb0bd6e8e5225a88c9b3a492f48cecf999c58913d5f29.

Next: collect before-regressions; then change only sort_parampol_terms
(torsion high-to-low, gamma descending, i128cross products), retain exact
tests/goldens, run after/full-core/CLI/cycle consumers on a pinned source.
Separately collect FINAL3847093, prepare standard408build/preflight using
stage_merged_math.py; it now requires four merge-checker hashes too.
Do not launch408against a failed foundation or fold unverified new tests
into its frozen source. No local builds/tests, commits or pushes this turn.

## Merged R3 submitted after repeated-label harness failure — 2026-09-29

R2job3847039 FINAL FAIL, report
a97a3e261b0d10e09647448bc7aa710accd5a7a4b3e04d9a2395778c64da436e.
All511domain/572core/CLIbuild/273capture/11diagnostics pass their gates and
no positive/exception stream is lost. Script-tail helper then asserts one
SET_BITS label, but the unchanged original fixture prints10iterations;
FACTOR_WEYL_ORDER also repeats9times. No block/unitarity capture ran in R2.
Do not treat this harness failure as evidence of a mathematical regression.

Fresh R3job3847086 SUBMITTED, stage
`/public/home/majj/atlas-verified-merge-20260929.s7Iz06Bn`,
pin00b5c5da4095bcaef12e092dc54cec63169f95a9530b931ca5797619bffc5b6e.
Receipt math_verified_merge_r3_submission_2026_09_29.json. Same frozen10
generic checks, runtime/fixtures/goldens and full gates, plus4new dedicated
marker tests. `script_tail` requires exactly10/9labels, compares all bytes
from the FIRST occurrence through recovery/end/Bye, keeps unique markers
strict, and proves first/middle/last differences still fail equality.
Driver82e3c59c559412bbe94e462e175b7d1ab6a3a6685391b684eacc3d5e098edf58;
checker e6b77bcf1c7c1159705d99d6e950367769a7cba54a7479f91604802d5bfd81bc.
No full408array submission until FINAL accepted foundation, fresh release,
and63-check harness; prepared stage_merged_math.py must use R3notfailedR2.

Ordering discovery3847067 FINAL rejects both provisional whole programs
originally: list[(int,Param)] is NOT list[(Split,Param)], although the A2
prefix executes. Report0e32ad2e661aeae38e074fa5e0b5db918b21ca307d282555ce87f613d1031226.
Retain both original files as negative companions. Added _valid fixtures
with explicit Split:(i+1,0), catalog4/checker12; see newest submission receipt.
No runtime sorting changes yet; original complete acceptance still required.

## Cycle replay restores terms; ordering capture submitted — 2026-09-29

Cycle candidate replay3847046 FINAL, stage
`/public/home/majj/atlas-cycle-candidate-20260929.OakAOs94`,
pin12483b4d1f8ffcf5f72f28370a0e8c93ceae06aa55b3c634b101b9984e1c1975,
report39cb4e1301bc47d7f6ac1ddd1e50992288168b2f900cf9a3257c1cad3c4537c0.
All3complete programs ACCEPT on original/exact566/exact569/root-sign candidate.
Existing root-sign repair restores the missing A2terms and all three final
KTypePol outputs. Full induction/standardize tails still differ ONLY in two
ParamPol terms' order; identity controls match. Do not mark full-tail PASS,
and do not continue treating the restored Cayley term as an independent bug.
See associated_cycle_frontier slice for raw hashes/source comparator evidence.

Provisional two-case ordering capture3847067 SUBMITTED, stage
`/public/home/majj/atlas-real-form-diagnostics-20260929.hy7SC0ww`,
pinf2773d9b420c6bc42602fb23efca4e3249cbf9da2ff8b30e683b1a64ddffaf18.
Receipt math_parampol_order_submission_2026_09_29.json; exact566/original,
checker12. A2/T3torsion and T1signed fractional gamma: complete coefficients,
print/forward/reverse/rebuilt insertion/cancellation. Original basic.at258
defines reverse([T]); no runtime sorting edit yet. Capture acceptance before
goldens, add unchanged before/after regression gates before any repair.
Suspects: high-to-low numeric torsion comparison and descending gamma;
preserve KTypePol and global ModTwoVector ordering.

R2merge3847039 still LIVE at last check;511domain/572core already passed,
but no FINAL merged report yet. Do not duplicate. `hpc/stage_merged_math.py`
is PREPARED ONLY, not executed/submitted: after FINAL foundation acceptance,
pin its report hash, freshly build exact merged source, require unchanged
63-test harness, then launch all408 unchanged cases. Hodge/cycle/medium/large
use fat32G and1200s; CPU600s. Submit and move on; no local execution.

## Current completed candidates and next exact union — 2026-09-29

R2exact-union SUBMITTED3847039, stage
`/public/home/majj/atlas-verified-merge-20260929.8MBkttoJ`,
pin947e33f6c3fd91242c8fc5d9fb85cd48b848e8c055de939837cacbbc77b0f560.
Receipt math_verified_merge_r2_submission_2026_09_29.json. Same runtime,
same511domain/572core/273+11+4+7, frozen10checker; bridge now861b1e48.
No mathematical comparison ran in R1. Do not duplicate or copy later
11checker/3cycle-diagnostic catalog into this stage. Next collect R2, then
replay3cycle diagnostics on its exact binary (or verified root-sign consumer
3846760) before considering another runtime repair; run full408 afterwards.
No local builds/tests, commits or pushes. All report artifacts retained.

UPDATE merge3847021 FINAL FAIL after511domain/572core/check/release PASS:
capture inherited old bridge5f823851 (hardcoded6), while frozen checker
ran10allPASS. No fixture comparison ran. Report SHA
ec4381dd8ee01d3e85f2366140635d7c6b92ae5f67d6abf73f1a3ee3644910d0.
R2 stages existing verified configurable bridge861b1e48 explicitly and adds
a stager hash guard. Runtime and mathematical expectations are unchanged;
fresh full gate, not relaxed acceptance. Check R2 submission receipt.

Cycle induction2job3847027 FINAL accepts both original/Rust programs; full
A2tail differs already at INDUCE_PARAMPOL, before K-type conversion. Report
655fc61830b04f2825b7bc617d6c3c8c01cedf3f263b2237934bd55a4ac9b9c6.
Expanded3job3847030 FINAL also accepts the new original/Rust standardize
trace, report3ac52382f1deeb4ae82bd9d26992040a0ee03a185f86ccbd194f9151d3c72ec7.
First difference is STD_CONTINUED on the disconnected sl2R.gl1R cuspidal
Levi; inputs, dominant weights, cuspidal data, regular translate all match.
Original coherent polynomial has TWO positive x1terms with distinct lambda;
Rust keeps one. Check Cayley_sum's Cayley/cross pair next, and replay on the
verified root-sign candidate BEFORE adding a second runtime repair, because
integer cross/Cayley use CommonContext. All captures so far here use exact566.

Full real-form review3846278 is now FINAL:164match/21mismatch/47RustFailure/
57OracleFailure/3OracleTimeout across292new exact566cases. Report SHA
64bd45a571623d93ae781e86ae73e344b0c7b80b45156d8c4847981213bdd7f0.
All36metadata/36KGB match; KLV35/36. Indexed real_complex_forms slice has
the operation matrix and failure families. No pending old fat survey now.

Exact-union gate3847021 SUBMITTED in atlas-verified-merge-20260929.1y5mrReF,
pin6051b8955c288b2115f34d10eb5b88ce997fedf6d77f4a9c10524075dcea4e79.
Receipt math_verified_merge_submission_2026_09_29.json; frozen checker10.
Do not copy later checker11/cycle catalog into this live stage.
Cycle induction2discovery3847027 SUBMITTED separately in
atlas-real-form-diagnostics-20260929.TPIUfNjk,
pin8f5e0d17e7cc6ff6e84866f65ffa5f860527d44808be57a05149b9a4a1e6317f,
exact566/original, checker11. Receipt math_cycle_induction_submission_2026_09_29.json.
It splits raw p_G, induced ParamPol, each constituent's raw KType/final
KTypePol and full result; real/compact/complex identity controls. Original
acceptance remains provisional; inspect FINAL before expectations/fixes.

Torus R3 3846731 FINAL PASS, report SHA
0d458f16f3e15448d323cc0143b05f9244f6f0453b8300372f05c5f756933486:
508domain/572core/CLI/273capture/no lost positives, all three regressions
beforefail/afterpass, complete SL2R-half and torus AV-ann tails match.
PSp root-sign consumer3846760 FINAL PASS, report SHA
acfc900cb35cc4b872cfd33f787d0626502fdd005733376e071f63d0950a37c9:
four complete block histories match original; three PSp histories differ on
exact569before, SC Sp4 control unchanged. This binary is core569/domain511,
not torus572. Next gate hpc/math_verified_merge.py freezes their exact
disjoint union, requires511domain/572core/CLI/273+11+4 and separately captures
seven unitarity companions. Check its submission receipt; do not duplicate.

Unitary companions3846973 FINAL capture: all seven original ACCEPT, but
exact566 Rust SU21/PSU21/Sp11 still rejects at theta-stable parabolics;
GL2R fails trivial-unitarity and inexact-halving. SL2C/PSL2C/SL2R accepts,
not yet complete-tail-reviewed. See slices/unitarity_final_constituents_2026-09-29.md.
Report f57d677df8c7cb70fa33995fe4414af883696bfde49b2a80c1edda06a6bad39e.

A2 live Phi trace on verified569 first differs in the printed induced term:
twisted inducing KType, embedding/rho shifts match through that point;
theta_induce_standard(param(q),G).K_type_pol drops x3lambda[1,1]. Separate
the ParamPol induction output from K-type conversion before fixing anything.
See associated_cycle_frontier slice. No general cycle acceptance.

Older entries below are chronological evidence, superseded by these results.

## Latest kernel FINAL and focused release consumer — 2026-09-29

Kernel3846740 FINAL PASS, reporte8459baf79f4fdf1376535cd8ac2dffb9fd0e1158ca400407a0a2dc06ef41a9f,
3beforefail/3afterpass/full511domain/source integrity. Original PSp4R partial/
full cross links and independent short-word signs restored at domain level.
Focused release consumer3846760 SUBMITTED in atlas-pos-neg-consumer-20260929.iNxzU5Pc,
pin7e6903f05655583176f6aebc41ebad3ab9e57e72ad0525719fbbcb7fa4aec016.
Fresh release of EXACT kernel-after (core569), four full original histories,
extra exact569before. Full stream hashes kept, only loader preamble outside
fixed marker omitted from the complete mathematical-tail equality. Read
slices/positive_negative_roots_2026-09-29.md. Do not duplicate or conflate
with still-running torus572R3job3846731, which has passed full508domain/
572core/CLIbuild and is in273capture (not FINAL yet).

## Newest positive-to-negative-root kernel submission — 2026-09-29

Three independent/original-backed pos_neg regressions are frozen; candidate
only initializes the inversion set empty. Job3846740 SUBMITTED in
atlas-pos-neg-kernel-20260929.3HaWFrgg,
pin2b0deb8952113808e6b6b7ac6d1445206b67b8517b2489953a02c42149311c1a.
Require3beforefail/3afterpass/full511domain. This uses exact569parent, NOT
the still-running torus572R3job3846731. Next merge needs exact source guards,
511domain/572core/CLI/273prerequisites/11diagnostics/4block histories before
any broad consumer claim. See slices/positive_negative_roots_2026-09-29.md.
Local partial_block.rs includes the new tests and candidate; original preserved
in /tmp/atlas-pos-neg.0UhJQrCz. Do not duplicate either submission.

## Latest factor retry and PSp block diagnosis — 2026-09-29

R2torus3846692 FINAL FAIL: before3fail, after2pass/1full-output failure.
Mixed-torus RootDatum descriptions retain input torus positions rather than
appending radical factors as original RootDatum::type does. R3 adds only this
metadata repair, preserving all tests/goldens; job3846731 in
atlas-torus-bitset-build-20260929.sWNRDVnE, pina34986b0bb8c0504ca81111fe0edc8b03d4aecb4c1c5a6bf63dbb73d92958671.
See torus_factors_bitset slice. No full after acceptance yet.

PSp block4discovery3846724 FINAL, all4 original AND Rust programs accepted
but mathematical streams differ. Report0d33eff80bf8cb3ef2035e890eef2a8502256da99fd2d042311a702e28fb1b26,
stage atlas-real-form-diagnostics-20260929.LrhJZATc,
pinaabf6ecec71882a69e66bab9fb4bc04ff3dad8275545a41ed43b572c059bc942.
Direct print_partial_block already has Rust7/original8: NOT only pool/Hasse.
Full promotion restores12raw/8selected rows, but cross links and KL entries
remain wrong. Source root cause found: partial_block::pos_to_neg starts with
ALL positive roots; original rootdata.cpp1486 starts EMPTY. Thus identity
gets a nonempty correction, and a simple reflection gets the complement of
the required singleton. Next: independent root-image and original PSp link
regressions before a one-line initializer repair, then broad gates. Do not
claim solved from source inspection. Scratch /tmp/atlas-pos-neg.0UhJQrCz.

## Latest torus/bitset gate retry — 2026-09-29

R1 3846643 is FINAL FAIL at before-log recognition, after the required three
before failures. Current Rust includes `(198467)`-style thread IDs before
`panicked`; the old exact substring missed them. No after source or build ran.
Report math_torus_bitset_build_2026_09_29.json SHA
7715c9f8dc58c62816ae355ce68b6fc4f6896a83b631d1a58597c6847a6a5c03.
R2 changes only the recognizer to allow an optional numeric ID, retaining all
exact names/counts and unchanged regression/runtime patches and goldens.
SUBMITTED job3846692, stage atlas-torus-bitset-build-20260929.wYlJ6giM,
pin19b7d0875123f43d6950ed4a82c6f96b73320578e33ec8ab02178da461941084.
Receipt math_torus_bitset_build_r2_submission_2026_09_29.json. Do not duplicate;
full508domain/572core/273capture/11consumer gates remain required. Next useful
parallel investigation: PSp4R cold/warm raw common blocks versus Hasse downsets.

## Newest completed fundamental repair and factor discovery — 2026-09-29

Fundamental5693846487 is FINAL PASS, reportf56c0ee29d2c5a2a020b0454b5028c08fd24ad40efcc4ecc9b78b0b586e22036
local math_fundamental_build_2026_09_29.json. Three beforefail/afterpass,
508domain/569core/CLI/273capture/1493source hashes; no lost positives. Both
whole lattice/span/duality streams exact, all14errors and survivors, historical
A2/B2/A3streams exact. Reduced A1xA1four-orbit assertion restored; trace full
stdout still unequal, NOT general associated-cycle acceptance. This foundation
is the parent for the next torus/bitset repair. Do not conflate with566survey.
Supplemental catalog now11: original8retained + direct simple-factor torus,
negative and script Weyl-order controls. Reference3846570 is complete in
atlas-real-form-diagnostics-20260929.ffGS38Vy (pinc960a3228cce3e0746ca1555f190bbecb5883d858f0f31dc54cd5cbd1ad46e6a).
Inspect its FINAL report before production edits. Scratch
/tmp/atlas-torus-bitset.2qSUk30q preserves exact569domain/typed/session sources.
New hpc/math_torus_bitset_build gate is being prepared: require unchanged569
before regressions,572after,508domain/CLI/273capture and11diagnostic consumers
with an additional exact569before arm. It has not been submitted yet.

UPDATE:3846570 FINAL report194c74b9ef12aa0580b84779c2b844223f71998e9b3d71c207c4c6e207337594,
all9original positives accepted; factor negative has2errors (string conversion
is VALID). Original full goldens downloaded/hash matched. Candidate572 now
filters T and uses OR for duplicate bits;3session regressions are frozen.
Job3846643 SUBMITTED in atlas-torus-bitset-build-20260929.WeljoFuP, pin
fb1d11511d5d11a21319dc5fc6e403c295cbd44d123fad98e40ca7a72b346c27.
Receipt math_torus_bitset_build_submission_2026_09_29.json. Do not duplicate.
See slices/torus_factors_bitset_2026-09-29.md for gates/pins/caveats; no after
acceptance yet. Production snapshots match verified569exactly.
PSp4R cold partial_block ITSELF has8rows versus Rust7, before polynomials.
Missing cold row: x2lambda[4,3]/2,nu[1,0]/2; companion x2lambda[6,3]/2 exists.
Inspect bruhat_below/CommonContext/StandardReprMod generation and Hasse downset
versus warmed containing block. Do not fabricate rows or bypass downset checks.

## Newest CPU form review and fundamental569 gate — 2026-09-29

Real-form CPUreview3846279 FINAL132:100math matches,1PSp4R KLV mismatch,
22Rust failures,9original failures. Local math_real_forms_cpu_review_2026_09_29.json,
SHA544d4a966236b820fca771cedfae01bb4694b5803c92faecff6f2827b30449c2.
Fullreview3846278/fat3846277 remain pending. Indexed real_complex_forms slice
records failures and scope. Prioritize reduced AV-ann negative-set-bits
trace (SL2Rhalf and torus), separate actual-coordinate unitarity, adjoint KLV
and complex FPP; do not assume one coordinate repair fixes them all.

Reference2733846359 FINAL accepts both whole `_valid` fundamental companions
in original; unchanged566Rust rejects both. Report4b3169b6785ebfbda0e219882303e7595391e6841dd047d8663f2588a27d9814.
Original complete stdout goldens retained; R1failed setup fixtures unchanged.
Current local569candidate uses exact rational lattice multiplication and
validate-before-discard signed32indices; three new unchanged regressions.
Job3846487 is running in atlas-fundamental-build-20260929.rnj3xGh5, pin
9caa326f8e346313fab2a786441725ecd299ea794b05c3540c35fb5ba5fb05b7;
before3fail/after3pass live, full508domain/569core/CLI/273capture and historical
fixture/cycle/final integrity gates still required. Do not duplicate or claim
full math acceptance. Receipt math_fundamental_build_submission_2026_09_29.json.
Read slices/fundamental_lattice_2026-09-29.md for exact evidence and caveats.
Scratch /tmp/atlas-fundamental-fix.xUc5s1dV holds before sources, generated
regression/runtime patches and staging helpers. No local tests, commits or push.

Follow-up8diagnostics now SUBMITTED3846542 in
atlas-real-form-diagnostics-20260929.4iqDXKdR, pin15d16e289a2ed8bdcf046cc30d85ab0eff2abfb18f288d203d3a9596ade3a9be.
Separate real_form_diagnostic_catalog.json leaves master273unchanged. Full
AV-ann stage probes (SL2Rhalf/torus), cold/warm PSp4R8versus7partial-block
anchor, valid-coordinate FPP companion, bitset duplicates and negative checks.
Exact566release/original; source-to_bitset adds powers rather than union,
but do NOT assume this is AV-ann's root cause. Inspect original whole outputs
before fixing. hpc/math_generic_probe now has pinned optional supplemental
catalog and bounded per-case timeouts;8harness tests run on compute first.
First staging xuhAFqy7 failed sbatchCPU1core8G/QOS4G, no job. Fresh4G stage
submitted; keep scratch /tmp/atlas-real-form-diagnostics.fwyflH9u and receipt
math_real_form_diagnostics_submission_2026_09_29.json. Fundamental3846487
live logs now508domain/569core/CLIrelease pass;273capture/final gates pending.

UPDATE3846542 FINAL:8harness tests pass; original accepts all7whole positives,
negative full survivors retained. Report1ec1c43521caa37ba7eb4300f2f98860416a6e0514e098eaf07e2d2db5386985
local math_real_form_diagnostics_2026_09_29.json. Bitset duplicate bug now
proved (original[0,0]->1, Rust2). AV-ann fails first at character_table(T1),
not W_cells/integrality. Source root-cause candidate: Rust simple_factors
includes torus factors unlike atlas-types.w388; order_W_simple(T1) requests
highest roots and triggers the negative bitset assertion. NEXT: direct
original-backed factor/order_W regressions, then smallest semisimple-filter
fix, independent bitset union repair, unchanged-before/after full gates.
PSp4R cold7versus8 persists; warming triple first makes later unit count8,
so inspect actual common-block pool/parameter histories (not count only).
FPP rho companion accepts both; displayed SL3C/PSL3C differences are two
middle product terms/witnesses swapped, not yet proven missing mathematics.
No fixes to these runtime paths made; indexed real_complex_forms slice has
details. Master273 and candidate569 stage remain unchanged.

## User-requested real/complex form coverage — 2026-09-29

Catalog now408: original116 unchanged,36 named forms x8 separate operations,
plus4 all-real-form inventories D6/D8/E6/E7. Indexed design and exact group
list: slices/real_complex_forms_2026-09-29.md. No runtime edits in this task.
New templates/checker extension passed all63compute-node harness tests; no local tests.

R3deform3846153 is now FINAL PASS, report4eb3448b87966358759711f680fb9e30a04d199deca5cbca8ffe9bd5ed8848bd
archived as math_deform_build_r3_2026_09_29.json. All7regressions,508domain,
566core,265capture/no lost positives/final source integrity. Exact release
build3846235/preflight3846236 submitted in atlas-deform-release-20260929.X1FQvWkw;
source1487files, frozen historical116inputs. Do not duplicate or change it.
New survey stage atlas-real-forms-20260929.XgRFmEhr freezes408inputs, SHA
909e6390b93aa80dafa01ae72ebc2d80f3a232446eb15bad88df4f9a6b5ded33,
own63-test preflight3846269 FINAL PASS, report130be17b6d2da62893996fa7420d54e0eda143b1de0d979aac0ac76b39f4dfba.
Release3846235 FINAL built, report1a1608e04a689790b70b52fd4f63e67d5ae8c15a0761b3d65f37083eea6438d2.
Helper verified all1487source files equal3846153, source/binary/log hashes,
and old116catalog/templates unchanged. New292cases SUBMITTED CPU3846276,
fat3846277, fullreview3846278/CPUreview3846279. Do not duplicate. Local receipt
math_real_forms_submission_2026_09_29.json; arrays remain unreviewed.
Existing116math suites remain separate; this new run covers indices116..407.
Early raw observations, NOT reviewed: SL2Rmetadata/KGB/KLV/unitarity/cycle
and SU2cycle match. Original Hodge fails SL2R3846280 (component assignment
index3/length3) and SU2 3846286 (negative maximum branch level). Do not treat
those original failures as Rust passes or overwrite the submitted fixtures.

Original271capture3846184 is also FINAL, local
math_fundamental_reference_2026_09_29.json, SHAd8587795fafd609e68a0267859faf35fd2d23fba93dd71c62c3d0157453443dd.
Two new fundamental positives FAIL SETUP in original: bare-core ratvec*vec
overload unavailable. Do not use their matching partial stdout as a golden;
retain them and add companions using exact numerator pairing or loaded valid
overloads. Negative index diagnostics captured; live A2Phi trace accepts in
both engines but complete outputs differ. Earlier267 embedded-coweight
counterexample remains valid; no fundamental runtime fix yet.

## Newest live gates and fundamental-lattice root cause — 2026-09-29

R3deform3846153 live logs now pass all7unchanged regressions,508domain and
566core; CLI release completed. Full265three-arm capture/final source guards
remain required. Do not declare foundation acceptance until report.json is
FINAL PASS. Prepared helper (not executed):
/tmp/atlas-orientation-20260929.9f0huoTh/submit_deform_release.py,
SHAb4903905d731c3b65b181b4e9cc9339594ae3ba54e3f1b90e13f651d41375bf8.
It requires the final foundation SHA and packages exact566source for the
unchanged116math consumers; does not include later local diagnostics.

Cycle3846161 FINAL267 report39d312c98d78087ee2774085063e6d981ec9b48d5c775407d292dba38642c035:
original accepts BOTH whole new fixtures. Nilpotent trace isolates A1.A1
Levi[1] coweight[1,0]/2vsoriginal[0,1]/2 before missing distinguished H/orbit.
Core wrappers hard-code fundamental_weight=e_i and zero-pad C^-1for the
coweight; original multiplies by actual root/coroot columns. This is a
concrete embedding defect, not evidence of a sort/dedup defect. No repair yet.
See indexed slices/fundamental_lattice_2026-09-29.md; full raw trace stdout
verified and downloaded under scratch cycle-trace-orbits/cycle-trace-phi.

Master271now adds3fundamental positive/embedded/rejection cases plus live
Phi trace companion. Capture3846184 SUBMITTED in
atlas-fundamental-reference-20260929.ZGbYwWPn, pin
ca98694d09a48190a7d75f4a346ad1ab1a508b3a8bc00e44c49aa6d37af828af,
using exact559/original. Do not duplicate. Original Phi R1`set kn_verbose`
rebounds a new global, so numerical output differs but no intermediate trace
was enabled; new companion uses := without changing the retained R1.
R3deform still freezes265; never copy master271checker into that stage.

## Newest exact559CPU review and R2deform failure — 2026-09-29

CPUreview3845923 FINAL, local `math_matrix_axes_cpu_review_2026_09_29.json`,
SHA8130c40a345001b57e9803642fe38de662b3dbd99a02690741430f4f750cac37:
49mathematical+3language matches,7rejection matches,4mathematical mismatches,
4Rust failures,1independent-invariant failure and18original failures.
Versus552CPU, B2/C2/D4AV-ann newly match, with no lost prior matches.
A2/B2/C2/G2cycle jobs3845935/3845998/3846011/3846027 now exit0 with empty
stderr but unequal outputs. A1.A1cycle3846077 fails the independent product
checker: complex orbit count3versus original4. Full multiplicities/support/
cutoff completeness remain unproved. The fat3845921/fullreview3845922 are
separate and still unfinished; single observations are not speedup evidence.

Deform R2job3846124 FINAL FAIL, report
16b90f644ec12dc5d3b8a02fd9ebe17b363528bd16e356cfc9b8edd3799e74c3,
now local as `math_deform_build_r2_2026_09_29.json`. Before4deformfail and
orientation1pass/2fail; after4deformpass and orientation2pass/1fail. The
unchanged rejection test sees3errors instead of2: the wrapper incorrectly
executes discarded calls (BuildAndDrop), and exports internal invariant
prose instead of original make_dominant's Runtime message. Both positive
orientation whole streams match. No full508domain/566core/CLI/capture ran.
R3local repair changes only that registration to Skip and the exception
mapping, preserving all seven tests/goldens. Scratch9f0huoTh contains pinned
R2log, generated R3patch/archive and submit_deform_r3.py; do not mutate R2.
R3job3846153 now SUBMITTED in atlas-deform-build-r3-20260929.8pYfaONh,
pin4f1598fdf1f381204f459558f8e6df83c72bf1cfa7483b810a89b32d4a7ac7ff.
Same unchanged559before/566after/508domain/265capture; no test weakened.
Do not duplicate; collect its final report before exact-release116consumers.

Cycle reduced discovery3846161 SUBMITTED in atlas-cycle-trace-20260929.BIodHa08,
pinbc916d7987f5a8057ebd26a80587a8d02ed0fb9bd80a39f9a4cfe83d74bd518a.
Master267retains265prior cases; new nilpotent orbit/Levi/H trace across five
groups and A2Phi term trace run on exact559, not separate566candidate. No
cycle runtime changes. Source/raw payload findings and next gates are indexed
in slices/associated_cycle_frontier_2026-09-29.md. Do not duplicate capture.

## Latest deformation failure and exact559consumer submission — 2026-09-29

R2orientation3846059 FINAL265 report
e0aff2f24ddda148b9f09f69686a05c0e2b21ec8a5151e2002199a7dacf86aeb
confirms real numerical errors: G2x5orientation0vsoriginal1 at gamma[1,1]/2,
and several x6fractional rows. Historical anchors match; nonstandard compact
A1 also wrongly returns0 rather than rejecting (discarded call is valid).
Original full goldens are archived unchanged. Local session now566candidate:
seven new tests; runtime adds active shared orientation in RepContext and
removes duplicated core algorithm, atop actual-block deform lookup.
Driver math_deform_build.py now v2,508domain/566core/full265capture plus
unchanged-before proof. Source/pins listed in ordinary-deform slice.
R2build3846124 is SUBMITTED in atlas-deform-build-r2-20260929.zaNFiERf,
pin7069d5f014c0a2d04b8e502a6c1b4551a5df07999a1db8edcd61476f0ca7add1.
Do not duplicate it. Scratch9f0huoTh has generated
regression/runtime patches, archive and submit_deform_r2.py. No tests ran
locally; no verified orientation/deformation repair yet.

Orientation R1FINAL3846008 has whole historical A1/A2matching anchors but
the broad probe used script-only infinitesimal_character and fails before
math. Master265preserves it, adding a `%`-based companion and nonstandard/
discarded/type-error controls. R2job3846059 in NqlLJ3V4, pin
25993034a4ae3e92387108f7a2c1959afab354db3a90c6ed871521c572ef86f9,
uses exact559release. Report/receipts are local in tests/reference/hpc.
Do not use the rejected prefix as an orientation golden. Runtime orientation
is unchanged; updated local math_deform_build.py is preparatory for a
future566core/508domain/265capture candidate, not a submitted build.

R1deform3845869 is FINAL FAIL; local report
`tests/reference/hpc/math_deform_build_2026_09_29.json`, SHA
bd56c0905d43e0d911bfb0fe43c9351b45f8a958ab730c314757d00896a7e260.
All four before regressions failed on unchanged559production. After the
wrapper change A2/B2/C2 reach their full-output assertion, but the collector
omits the original top-level loop Value row; G2 hits exp_i's odd exponent.
No563core/CLI/full-capture acceptance. Local session helper now includes all
nonvoid Value events and retains all original bytes (SHA2be5e04913c42dd1472774cfcd34b79d434f799f95058b64398667cdec5ce2ed).
Requires corrected before/after rerun; do not weaken the G2 invariant.
Both orientation implementations currently follow inactive original code;
new master263orientation discovery includes complete partial-block parameter
counts and historical anchors. No orientation production edit yet.
Scratch: `/tmp/atlas-orientation-20260929.9f0huoTh`; failed decoded log remains
in `/tmp/atlas-matrix-axes-fix.dBt6x1Zj/deform-r1-after.log`.

Matrix559standard build3845871 PASS, SHA
4cc727bc413725b3a9cb90927e283c8e0159f6ffbfb4e5862fb5adfade8784fd;
61-test preflight3845872 PASS, SHA
143bf61eb7c18b79853e4674496ea03a33cbe7713368f7bdbd70786677e5878d.
Login staging verified1473source equality, binaries, logs, inputs and parent
proof, then submitted CPU3845920/fat3845921, fullreview3845922/CPUreview3845923.
Receipt: `math_matrix_axes_suite_submission_2026_09_29.json` in tests/reference/hpc.
Unchanged116inputs, CPU6GiB/600s/fat24GiB/1200s equally for both engines,
Rayon1. Stage pYus8R4f remains frozen; no563candidate mixed into this suite.
The older552fat3845472/fullreview3845473 remain separate and unfinished.

## Latest reviewed mathematics and matrix frontier — 2026-09-29

FINAL matrix3845829 report96269f8c3645ca40441d00bd258056f4f9cc9852a1357e74b4bb782cb6dd53a8:
4before failures with unchanged production;3new after passes plus corrected
historical unit in all559core;CLI/full257capture/final1473source integrity.
117whole positive matches/no parent or reference losses; all8negative errors
and survivor stdout match. Negative diagnostic envelope still differs.

NEXT JOBS ALREADY SUBMITTED (do not duplicate):

- Matrix exact559release3845871/preflight3845872 in
  `/public/home/majj/atlas-matrix-axes-release-20260929.pYus8R4f`.
  BaselineSHA3ac9cdfda4cfb2e41b5af8940afb99343ddc6c2c27c873e5f442418d57391c2a,
  suite-inputsSHA d4da1dd8da9de083d5e3d7129765609fab6b71d6c71990e9ef4c50387e620d25.
  Require build/preflight PASS and1473file equality to3845829 before all116
  arrays. Use existing CPU86/fat30indices,6/24GiB,600/1200s and independent
  reviews; don't transfer ongoing552suite results to this source.
- Ordinary-deform563candidate3845869 in
  `/public/home/majj/atlas-deform-build-20260929.ziS3OXJP`, pin
  c47d6807fff2b4c2643cc5f57edf5120b4459d1ba46c171c8c5fcb63f3262697.
  Four whole original-backed tests must fail on unchanged559runtime, then
  pass without assertion changes after wrapper lookup repair. Requires563core/
  CLI/frozen261capture/source integrity/no positive losses. Not verified yet.
  If after fails, keep the raw logs and diagnose; do not weaken invariants.

Local master remains261; session/domain files now include563candidate, while
matrix release uses only the frozen1473file559source. The two submission
receipts are in tests/reference/hpc. No commit or push yet.

Fresh HPC-login git ls-remote checks on2026-09-29 still show original7e1b958c
and Rustmain05625c5d. See math_remote_refs_2026_09_29.json; no stale upstream
substitution and no pull into dirty/frozen stages.

Matrix candidate3845829 is SUBMITTED, stage
`/public/home/majj/atlas-matrix-axes-build-20260929.JPbERXzU`, pin
b21cf0ccb525da19c58f2c1ec0fd3fdaddabaa429540022997087c3639f07f14.
R2original3845762 FINAL report94fadb74acfaa8f8a75995ec51eee30e9a8072642bbf1f2391d3f15b8b482bd7
accepts both new whole positives and provides all8negative messages/survivors.
Candidate corrects only row/column axes in reads/writes/transforms; keeps
storage/one-index columns. New tests3 plus corrected historical typed unit
must fail on unchanged556runtime; all559core/CLI/257capture must then pass
without weakening assertions. Source/patch snapshots:
`/tmp/atlas-matrix-axes-fix.dBt6x1Zj`. Do not alter its frozen stage.

Parent boundary3845705 is now FINAL556core/CLI/251capture,115whole positives
retained/no losses and all5syntax/stdout contracts fixed. Report
db3df68b46d7d4ae895f37052214ba2a4599f55e3f1bce6db6675ed476ab073a.
Independent unchanged552before R2report737aa425 proves1pass/3fail; full
negative stderr display and anonymous arrow printing remain separate.

Next mathematical diagnosis: ordinary deform still uses a historical full
BlockGraph and shared lambda-rho instead of the actual parameter's lookup.
See [ordinary deformation](slices/ordinary_deform_common_block_2026-09-29.md).
Master261adds4direct probes A2/B2/C2/G2 before any deform-runtime change.
Discovery3845837 is SUBMITTED in atlas-deform-reference-20260929.3XEsG0zH,
pin a968f40449d8a7f28c2fa39983e20d5fe83a6ad150e5d0977dfee92a0c4374bd.
UPDATE3845837 FINAL report1fa25ca34f62a1f82802a44793c0b640e38ab8c40a2df8e4e4896bc2d7acf7d0:
all4original positives accept, all4old552panic; full buffered Rust stdout is
empty, so the exact first command is not located by this capture alone.
Original stdout copied unchanged into four `.oracle.stdout` files. Four new
session units compare complete payloads; local ordinary-deform candidate
now uses RepTable lookup/common_deformation_terms while preserving finals
and all invariants. It is NOT verified. Driver math_deform_build.py requires
4unchanged559before failures,563core/CLI/261capture after and no positive loss.
Helpers submit_deform_build.py and submit_matrix_release.py in the matrix
scratch directory require a FINAL matrix559report SHA before submission.
Local session is now563candidate, not frozen559; source hash4761d1ad,
domain_builtins974484ae. Never copy it into the existing matrix stage.
Matrix3845829 has executed all3new before failures and reached after build;
its independent corrected historical-unit failure also gates that transition.
UPDATE: all3after units and all559core now PASS; CLI/capture/final report
still running. No complete matrix gate or downstream116acceptance yet.

CPU3845474 is FINAL exact552:49matches(46math+3language),7rejections,
12Rust failures,18original failures. Report SHA
5295fd3d23c53fdfce1b455448e91ab84f7aa2bad3cd4e34c11790107228b84c.
Compared with536CPU, nine new matches/no losses. Full116review3845473 is
still pending the live fat3845472 array; do not extrapolate CPU acceptance.
Matrix discovery3845716 FINAL255 report SHA
0f3f9f73f8ce549b47ad02a74f7efe84f9d23d8665d920e65c515ae5b102c11d.
R1 positive has a FIXTURE dependency error: assert is script-defined, not
a bare builtin. Preserve R1; master257 adds builtin conditional-error
companion and the exact historical2x2matrix unit sequence. All independent
numeric expectations remain unchanged. The negative fixture already proves
wrong accepted row2 reads/writes/transforms; retain all eight original errors
and unchanged-matrix survivors. No matrix runtime fix accepted yet.
R2discovery3845762 is FINAL in atlas-matrix-axes-r2-20260929.BQqxCtrn,
pin cd7cb39ce84f74772ff8523aa1f8afaeb6dba0d6587cd331bacaeb145a787c15.

## Current mathematical-validation frontier — 2026-09-28

Boundary556build3845705 is SUBMITTED in fN3It8ma afterany3845638, pin
8af7bf064fe9bd99c71c6033fc36ececfa13ebb3dcd42f7a6439364d0b95a56a.
It requires corrected before-proof report,4unchanged after tests, full556core,
CLI/251capture, five Syntax/stdout matches, no lost115positives, and unchanged
six exception streams. It is NOT the552source under116math validation.
No source may be changed inside submitted stages.

NEWEST: exact552build3844955 PASS (SHA924f7aede8ef76813f4ded358c6bf2db366da1c2891bc092c2e7a5c8f497c6a4)
and61-test harness3844956 PASS (SHA113b5ccb971e046e3581225f04493a25c2a7a40302cbf74a9653fdb99dec0f75).
All1459source files equal552foundation. Full116arrays are NOW submitted:
CPU3845471/fat3845472, fullreview3845473/CPUreview3845474, using6/24GiB
and600/1200s respectively, same limits for both engines. Current live A2
scriptKLV/unitarity/AV-ann and several other scriptKLV cases match; require
independent reviews before broad acceptance. New actual-math failures:
A2cycle matrix.at38, B2AV-ann combinatorics.at869 (swapped matrix axes),
and separate B2unitarity height-parity invariant. See matrix_index_axes slice.
No matrix runtime repair yet; master255adds two asymmetric matrix fixtures.

Original253arrow3845481 FINAL report SHAe45b7f0677a36b9456242fb4fb37530f4f7a18585dbbd407bf1a15c5fdaff93f:
arrow declarations accepted; bare formal/(int)/[*] group roots Syntax-reject,
but primitive void ACCEPTS. New full arrow golden retained; needs HPC byte
verification and exact552before failure before printer change. Earlier whattype
control remains matching. Boundary3845482 runs1pass/3intended failures but
report FAIL231ce1d790a061574f626b2a179f2766ff2809cde5a6e61e56c7f6c0c8e614c5
because Rust includes `(thread_id)` in panic headings. Corrected checker rerun
3845638 at CEQxT9N6 keeps identical tests/runtime, pin4fe8b71e8c1ebc689459fa4bd5b23e3e969a1d29f45881af56288a7c5b465b29.
Local556runtime candidate separates group grammar and adds virtual-spec
newline recovery; no anonymous-print or matrix changes. Candidate helper at
/tmp/atlas-recursive-boundary-fix.Bmy5lNJD/submit_boundary_build.py targets
atlas-recursive-boundary-build-20260929.fN3It8ma after3845638; inspect receipt
for submission status before acting. Never modify the frozen552math source.

LATEST2026-09-29: exact recursive552release build3844955/preflight3844956
SUBMITTED at /public/home/majj/atlas-recursive-group-release-20260929.MGjMmU4M.
Receipt math_recursive_group_release_submission_2026_09_29.json records
suite-inputs SHA5b1cec76171b5dc3b4d0ab326002781458b79c1714d3468773b07d1b936d310f.
Both confirmed RUNNING before subsequent SSH connection timeouts; do not
resubmit based on missing observations. Full116arrays NOT submitted yet.
Foundation3844656 is FINAL552core/CLI/1459sources/245capture,114whole positives,
four high-level imports load with empty stderr but stdout differences remain.
Require exact foundation source equality and preflight before consumer arrays.
Memory harness now accepts explicit limits: old6GiB was enforced even on
fat32G. New CPU6/fat24GiB (allocations8G/32G),600/1200s, BOTH engines equal.
New harness tests are in3844956, not locally executed; no speed claim.

Same552followup3844760 FINAL251 report SHA
d8f1d7fabcb30c80460e630b1c581fd05c80375f28f291d0e1038d77e6a87c27
is archived locally and matches HPC. No positive losses; boundary positive
ALREADY matches completely. Earlier whattype-based print-counterexample
hypothesis was wrong: it uses another printer. Five negatives do confirm
3wrong Program/2wrong arity errors instead of original Syntax, and alias
recovery differs. Four new session regressions are TEST-ONLY (runtime still
552), expected1pass/3fail, not executed yet. Master253 adds arrow-function
declarations and restricted-root discovery; no original result yet. See
generic_recursive_groups slice. Temporary sources/helpers:
/tmp/atlas-recursive-boundary-fix.Bmy5lNJD and
/tmp/atlas-recursive-group-build.ESfmC2u1. Older snapshots below are historical.

LIVE3844249 now passes five after regressions and full544core,46.16s;
CLI/release/frozen243/final integrity still open. R3original3844285 FINAL245
accepts the entire multi-group transform-scope positive; error companion
retained. Report43a5d13852a8069bf0e0388fd254803901cead2532d4433fa4b61c25708e20ad.
Compare these new245inputs against the eventual544binary separately; R3uses
old523Rust. Do not change submitted243stage. Still no high-level acceptance.

New polynomial-write544job3844249 RUNNING at LrSMV8Re, pin
c4dac3c8bd4feaa84505569f773a89f004d8017b7b73916b52eca0a8182ea405.
All five unchanged before regressions FAIL as intended; after compilation
still open. Frozen243catalog/report goldens already byte-checked against
original3844074. Master245adds separate transform-scope/error discovery.
Helpers at /tmp/atlas-poly-write-build.dleCDxeZ; the frozen before snapshots
there are verified539 (older lTSg9Fvu snapshots are538). No accepted544yet.

LATEST live-destination3844021 FINAL: the unchanged regression fails before
and passes after; all539core/CLI/final1442source and241capture gates pass.
107whole positive streams (one gain, no losses), parent atomicity and all
three subgroup exception streams retained. Report SHA
69ed01a390e698b8702f85a6ca5ede3603a01af3e9dfb7ead8474863a10eff21.
This exact source is the new polynomial-write baseline, not full math acceptance.
Polynomial writes now have an UNVERIFIED candidate plus five new tests:
replace/insert/delete, chain/effects, transforms, nine boundary diagnostics,
and atomic foreign-owner rejection. Before/after targets must be separate;
full544core and frozen243capture remain required. Read the polynomial slice.
Older pending/unimplemented statements below are historical snapshots.

LATEST FINAL transaction3843795: before0pass/2fail -> after2pass; full538core,
CLI/final1440source integrity and all238capture gates PASS,106whole positives
retained/no losses and two exact rollback stdout matches. Report
0284810cbf6801ed922d975b2c36840156947eb3a6f8735af3c12b26f7f5b0f4.
No generic parser/SCC implementation; high-level libraries remain blocked.

Discovery3843926 FINAL241, reportd27b157d37eacf753db568c842a827008c18593b5dce2fe73b2b781a78d4f40d:
original accepts full chained writes and row/vector live-destination controls.
Rust[7,2]vs original[7,7]is confirmed; RHS rebinding and index effects also lost.
Regression plus minimal live-cell repair3844021 is running at8zeLYvaS, pin
7fdb5793e7c7ebec36e73715520c1f4542bff502fd09215baa22d7dbba817194.
Before unit EXECUTES and fails; identical after unit and full539core now PASS
(0failed/ignored/filtered). CLI/241capture/final integrity still required.
See slices/component_assignment_live_target_2026-09-29.md. Polynomial writes
themselves are NOT implemented. Original foreign writes change key identity
(KKEY_RETAINEDfalse/PKEY_RETAINEDfalse), then reject reads by supplied keys;
do not use these corrupt-owner terms as mathematical goldens.

Master243adds transform/reverse/equal-owner and finality/zero/discard/type
controls; original discovery3844074 at yDDZbwy7, probe
ef3175fa1a175936aa780c8632c5582f3930c207f16a3e9a040af49dd6691e57.
NOW3844074 FINAL, reportf7a42716b0a6ce1daab2287c960edee485413a45058d9d07ca0043cadeae6239.
Complete transforms accepted: dynamic key precedes operand(ORDER21), simple
identifier key is reread after RHS changes it, reversed writes accepted but
reversed transforms reject. All9boundary messages inspected. See polynomial
slice before any transform lowering or final-key/zero/no-value implementation.
Valid coefficient-effects golden is local; verify its bytes on HPC before
using it in a regression. Frozen238/241stages are unchanged.

Exact536release3843772/preflight3843773 COMPLETE. Build SHA
f1dd49fac5411f0a800d22b3868d14fb6efb79b8369a8cd94906558cb9dce035;
preflight4db0c78774b1c337fb13f0b031b8d4ef6a48fa77f590d8672d17edee2adbcc9a.
All116unchanged consumers submitted CPU3843929/fat3843930, fullreview3843931
and CPUreview3843932. Keep600/1200s/Rayon1. Case2/3843935 is SCRIPT A2_klv
blocked at lazy_lists.at5, not a new bare-core FPP regression.
CPUreview3843932 FINAL:86cases,40whole matches(37math+3language),7rejections,
21Rust failures/18original failures. Report
fe3a528ffecb00d92bfb7798af1026ea619c34bbf68c5e0db04225bfccee40e0.
Full116review3843931 remains pending; these536results do not verify539.
HTTPS login-node refs still original7e1b958c/Rust05625c5d; Git SSH auth fails,
HTTPS works. No need to push unverified candidates. Helpers are in
/tmp/atlas-polynomial-write.lTSg9Fvu; receipts/source/report pins are durable.
All tests/builds stayed on HPC. No commits/push. Older pending snapshots below
retain history and must not override these current results.

LATEST guard3843259 FINAL:536core/CLI/release/final1436source integrity PASS;
all234three-arm streams retained,106complete positive matches/no parent
losses. Report68bec0c7c7750fb56a9d776378b8584958f50ca2e60ad5fe40b83ee1ed2600b4.
High-level imports advance to generic recursive groups(lazy_lists.at5) and
polynomial coefficient WRITES(finite_dimensional.at117); none is yet accepted.
Exact536release3843772 is confirmed RUNNING, preflight3843773 COMPLETED.
Its all116consumer arrays are not yet submitted; subgroup532full review3843198
is still PENDING. Keep version-specific results separate.

Grouped publication candidate3843795 is confirmed RUNNING at
atlas-group-transaction-build-20260929.xPR8phK1, pin
eca58a2393095bb79ba2471deb019369f2d018c0d6b2840717132d215fdbde2c.
Before EXECUTES both regressions:0pass/2intended assertion failures,0ignored.
Staged TypeTable/OverloadState/locations/events must publish atomically;
simple-alias semantics remain unchanged. Full538core/238capture still required.
Original late-member rollback3843432 report61ef95ce3d74830037b09a10fb98cda2a75547e974fd5f8d26a40965b1e8a6a9;
corrected recursive components3843356 reportbb08404a36d93381fa7415127a979b02a1ba3612969a43bc8ae26be5a00c1aa3.
Both complete original goldens are local. No generic parser/SCC repair yet.

Master241now adds coefficient-write effects, ordinary component live-target
controls and foreign-owner write/read diagnostics. No write runtime edit yet.
Source axis.w8497holds a live shared_value reference across RHS/index effects;
Rust currently clones the aggregate BEFORE them, a suspected lost-update
defect that must be original-captured before repair. See polynomial slice.
Frozen transaction238and while234stages remain unchanged. Older LIVE notes
below are historical, not current status.

LATEST guard3843259 RUNNING in WXQZc6QP (pin3d1b13fa...), not finalized.
Before source EXECUTES all4regressions:3intended assertion failures/1negative
control pass/0ignored. Independent after target now passes all4unchanged
regressions AND full536core (0failed/ignored/filtered). CLI/234capture/final
integrity remain required; no final integration acceptance from live logs.
Original3843183 accepts both complete mode/nested positives, six negatives
inspected; report cb256110021970f80bb65bd19b52e69f920e8bd12348f6aaf7e486ae2c287271.
Retained first nested syntax failure3843167; corrected companion only adds
`; dont` after break in the scoped DoExpr branch. Four new unit regressions
plus static/runtime guard-boundary repair are in local source. Fresh HPC
targets prove before/after before full536core/234capture. See while_guard slice.
Exact subgroup532release3843161/preflight3843162 PASS; ALL116unchanged cases
submitted CPU3843196/fat3843197, reviews3843198(full)/3843199(CPU).
CPUreview3843199 NOW COMPLETE:40whole matches(37math+3language),7rejection
matches,21Rust failures,18original failures. Report e8c09269e71d3ac904090c600a3cf129b576c06663878113870a21068826bb69.
Full116review3843198 still separate; do not transfer these results to guard536.
Build d4a8c982f018137fa5a1cce4d2f78a3b63bcaa41d46f01c0040cb6e6a04a5e12;
preflight a48fba97855fe879fa7ee572630c09bdf050192a90847c960114d2312292ed18.
While that work runs, master236adds recursive-group atomic rollback and
anonymous recursive-component discovery. No generic-group runtime edits.
Discovery3843306 submitted at Tl10Axe9, probe917299b7c1d220323cae54e1d1e1fb51b1ead8ae46ed36a144ee63b8bd15a74d.
NOW3843306 COMPLETE: report6c78c5f7297ce95fcb1f1cb9ab44391e4c82358b32d8fbb10c2da34f2713ebd4.
Nongeneric duplicate-field and function/type collisions are wrongly accepted
by old Rust, corrupting fresh values and subsequent names. Original rejects
both, preserving every survivor. Full expected report added. Component
positive fails originally on copied-tag alias ambiguity and adjacent >=;
retain it. Master237adds a companion with full child printing and > = spacing.
See generic_recursive_groups slice before implementation; no runtime group edits.
R2components237discovery3843356 submitted at BFMlNcea,
probe db5115a352fd46d3f209323f2da4de5f1b9a6cd099b42bac8d3c2b6a8f641697.
Scratch submission helpers are in /tmp/atlas-while-guard.Ok2lVcrH; all actual
source/input/report pins are retained in tests/reference/hpc receipts and HPC
stages. Local tests/builds were not run. No commits/push this turn.

CURRENT 2026-09-29: subgroup3842990 is COMPLETE, not running. All532core,
CLI and1430file final integrity pass; all228capture streams retained,
104whole original-positive matches/no losses. Entire minimal/broad subgroup
outputs equal the separately labeled corrected-original diagnostic; unmodified
original counterexamples and negative diagnostic-format differences remain.
Report dda80bef5fddb4ee1e155c5dcd7204de177191d11c7958254200add0c34122a2.
Exact-source standard release3843161/preflight3843162 are submitted in
atlas-weyl-subgroup-release-20260929.Anx6eiJt; all116consumers remain required.
Next COMMON frontier is W_orbit.at329: while guards are analyzed OUTSIDE
their own loop and evaluated OUTSIDE its Break handler. Three discovery
fixtures now extend master230to233. Read slices/while_guard_break_2026-09-29.md.
Generic recursive groups remain separately open after this common repair.
The paragraphs below retain earlier snapshots; they are not latest status.

LATEST: original-only diagnostic3842817 COMPLETE. Minimal empty-subgroup
and ENTIRE broad matrix/witness fixture pass after exactly two forwarding
fixes; all nine negatives retain identical stdout/stderr. Report
b72e92168b0c0ed087aa51aac309018f6fd97f69da28b8b78ab6d0be9907c8a6.
First3842703 is a retained patch-context failure before math, not a result.
Rust candidate NOW adds all four subgroup overloads with checked arithmetic,
exact pairing BFS, ambient witnesses and seven regression/property tests.
Full532core/228capture3842990 stage MKxzyGsd is RUNNING, NOT finalized.
Pin ccb0e16ec38226d8540c575f2acf637118c4b595887629567302257dd0409e28;
Live logs pass7new tests/all532core/CLIcheck/release. Complete228capture and
final integrity/report are still required; do not promote unit logs to a
finalized integration result.
Read the current top of slices/weyl_subgroup_orbits_2026-09-29.md before work.
No edits to official original or shared dirty HPC checkout; no commits/push.

Latest BEFORE tagged525 CPUreview3842500 COMPLETE:40whole matches (3language),
7rejection matches,21Rust loader failures at W_orbit.at127,18original failures.
Local/HPC report SHAe065b21100a347bd2141d955f4d2f47a21e9ec675c638b5afca656633ead03d9.
Full tagged116review3842499 and canonical3842129 still depend on live fat
arrays. Do not transfer these old-source results to the subgroup candidate.
Next independent frontier: generic mutually recursive set_type T [...] in
lazy_lists.at5 (AV-ann). Original230discovery3843104 is COMPLETE at ffZ34vsd,
pin d495ab93f06257c2981adf3ed303e0f368e9a5107320297c69ba957ddc961da6.
All228oldcases retained; original accepts all finite int/bool stream values,
rejects all7group errors and preserves old constructor/value bindings.
Report d6656ab92a51582e8ee88fccc934d3fb6f27aa9f96d83247dd4880817bb64b3e
matches local/HPC; complete report-line goldens added before runtime edits.
Read generic_recursive_groups
slice. Do not use master230/checker to mutate frozen subgroup228stage.

Older states below are the retained chronological record, not latest status.

PRIORITY: original mathematical counterexample in subgroup discovery3842475!
SC A2 g=[],v=[-1,-2]: original matrix orbit[2,1], but witness list identity.
rootdata.h669/678 ignores g in make_(co)dominant; witness functions pass g.
Do not port this bug or adopt the matrix as golden. Full retained report SHA
b499f884ce49a2825e32456c4ae02fe3e37387d716c15f9e84458792440cd7a9.
Master catalog228adds an independent empty-subgroup invariant. Isolated
original-only two-line causal diagnostic is being staged at ZwXZpKGH;
unchanged original remains the baseline, patched binary is NOT a new oracle.
Read slices/weyl_subgroup_orbits_2026-09-29.md before any runtime orbit port.
No Rust subgroup changes yet; full tagged116jobs3842497/3842498 remain live.

LATEST canonical CPUreview3842130 COMPLETE:86cases,40math/7reject matches,
21Rust failures at the old suffix loader/18original failures; no successful
math losses versus3841968. SHA65b8a7df836fe70d9dcc0284281bd094fc703e4d12b15b9240dca3138a250dc4.
Full fat3842128/review3842129 remain open. Reviewer writes review.json.
Tagged standard3842314 COMPLETE, exact1426source equality to3842065,
preflight3842315 PASS. ALL116cases now CPU3842497/fat3842498, independent
reviews3842499/3842500. Keep canonical523and tagged525acceptance separate.
Subgroup R2reference3842448 also rejects the positive at bare int-minus-vec;
R3job3842475 at3Mx8uCfk retains227inputs and uses unary -v(global.w5144).
No production subgroup edits yet; collect the entire original positive
before deriving goldens. All earlier fixture rejections remain in the corpus.

CURRENT subgroup discovery3842256 completes with nine original runtime
rejections, but its provisional positive uses unavailable bare adjoint(RootDatum).
Keep that failed source;226companion3842448 corrects only to
adjoint(LieType,bool), retaining all mathematical checks and225previous
entries. Stage C3d3A3qc/pinfc32efc010631b54d4b5dacd2ded776a9dfb208c354b5c5f9eccca1cef4ab897.
No subgroup production edits yet. Read weyl_subgroup_orbits slice for source
ordering, pre-no-value validation, ambient-owner and arithmetic boundaries.
Tagged standard release3842314 is live at rI7iffCH, preflight3842315 terminal;
all116arrays for that source still require finalized source-equality gates.

LATEST tagged3842065 COMPLETE525core/CLI/release/1426source integrity,
all223capture with104whole positives/no before or parent losses. Required
suffix/subject-stop/canonical full streams match; negative stdout repaired,
stderr/category aggregation still differs. Reportca62958f6c9889df9271852e54e1d583a1db8c6929bee8177f1757933f2edd97.
All four high-level imports reach W_orbit.at127's missing subgroup overloads;
AV-ann additionally hits lazy_lists.at5. New225original discovery3842256 at
atlas-weyl-subgroup-reference-20260929.Q0e6O40q is submitted before any runtime
orbit repair. Read slices/weyl_subgroup_orbits_2026-09-29.md. Tagged standard
release stage rI7iffCH is being submitted, not full116acceptance.

Canonical standard3842063 COMPLETE, exact1423source equality to3842014;
preflight3842064 PASS. All116arrays CPU3842127/fat3842128 submitted, reviews
3842129/3842130 follow. Source/inputs/timeouts unchanged; no shrinking scope.
GitHub refs freshly checked on login node, still05625c5d/7e1b958c.
Older LIVE/SUBMITTING statements below are historical.

FINAL canonical3842014 COMPLETE:508domain/523core/CLI/1423source integrity,
all217streams retained,102whole positives versus99/no losses. D4/D6/D8/E7
canonical mismatch is fixed on FULL expanded outputs; all D8/E7/E8 metadata
also exactly matches. Report4755f048a8e5342a6977e5ab30547f0ccc0bc107830399efeaf812fe4f4770c7.
Keep original signals (CANONICAL_GATES_PASS_ORACLE_UNAVAILABLE), full116and
parallel acceptance separate. Standard release stage DNq0OCvS freezes this
exact source for all116existing math inputs. Build3842063 is confirmed live;
preflight3842064 COMPLETE, reportfacc9789650c860cdd471d958c728f3184eaba40de168189b6ab26cb5795b1a4.
All116arrays require finalized build/source equality before submission.

NEXT tagged-case525candidate at atlas-tagged-case-build-20260929.pqwSIJ7O:
ordinary suffix grammar/span and source-ordered label/default coverage checks,
two new original-backed session units, full223capture. Starts from finalized
3842014; NOT yet verified. Original3842032 proves four wrong acceptances and
two diagnostic-priority cases. Local core source now includes this candidate;
do not call the entire current worktree verified by3842014. Job3842065 is
confirmed live; its logs pass525core/CLIcheck, but release/capture/final
integrity remain open. Pin31d51bcf075fcb00fc3b05812402f8debfc9470f47591b8c85728634fdfa1371.
See tagged slice and the two new submission receipts in tests/reference/hpc.

LIVE integration3842014 has passed508domain and523core in its logs; release
and full217capture/final integrity still pending. No finalized acceptance.
Suffix222isolation3842018 COMPLETE: original accepts the ENTIRE ordinary
expression companion; both bound prefix/suffix while versions signal, as does
the discard-pattern companion. Subject-stop suffix while matches completely.
Original ordinary payload retained in tagged_case_suffix_expressions.expected.
Next223discovery adds prefix-only coverage/default/duplicate-order negatives
in atlas-tagged-case-coverage-reference-20260929.yOCQGVuN (receipt follows).
No parser/typed runtime repair applied. See tagged_case_suffix for exact pins.

LATEST canonical kernel3841963 verifies3before failures/3controls;3841994
passes the unchanged6 plus7seed/8strong/12mod-two tests (exhaustive3x4maps).
Production candidate changes only mod_two.rs/real_form_seed.rs/strong_real.rs
from lazy523. Full508domain/523core/217capture integration3842014 RUNNING at
atlas-seed-section-integration-20260929.3eIMDyFQ, pin37cb27c95c43e415588d84c0ca0ead4ffb0201a7fb5f513d106869ee18ba36ee.
No complete metadata/full116acceptance yet. See canonical_seed_sections.
217original discovery3841964 additionally proves SC D4/D6 seed mismatches.

Corrected CPUreview3841968 COMPLETE, SHA4506687cca636c4ef64b854532dcac6e31c0e199fb31e453ad67a9f5c6266850:
86cases =40MATH_MATCH,7rejection matches,21Rust failures,18original failures.
Fullreview3841969 still waits on live fat3841831; no full116pass. This is
the pre-section lazy523source, not the newer kernel/integration candidate.

Suffix222isolation3842018 RUNNING:217's supposedly valid companion still
signals originally at while after printing every ordinary-expression value.
Five additive companions separate expressions, bound/prefix/discard while
and subject-stop control. No parser runtime edit yet. Preserve both original
signals. See tagged_case_suffix; current local catalog222does not change the
frozen217seed integration. All evidence/receipts live under tests/reference/hpc.

CURRENT priority: complete212capture3841698 has99whole positive matches
(98before/no losses), but D8/E7 initial_torus_bits/central_fiber differ;
only E8 metadata newly matches. Test-only before kernel3841963 and217original
discovery3841964 are submitted; no production seed repair yet. Read
[canonical sections](slices/canonical_seed_sections_2026-09-29.md) for exact
counterexamples, source hypothesis, immutable stages/pins and required gates.

Standard lazy523release3841793 COMPLETE, all1423source hashes equal finalized
3841689; all116unchanged cases submitted CPU3841830/fat3841831. CPU is terminal,
fat still live. CPU reviewer3841833 failed on the WRONG EXTERNAL MANIFEST PIN,
not a mathematical assertion. Cancelled unstarted wrong-pin fullreview3841832.
Corrected review-only jobs3841968/3841969 use existing outputs, in
atlas-lazy-release-review-r2-20260929.pvz1aQAS. See R2review receipt. No full
consumer acceptance yet; no interpreter rerun or reduced fixture scope.

Original suffix discovery3841829 COMPLETE. The provisional positive's
`((n)).suffix_int` is invalid original grammar and yields an original signal;
keep it unchanged, add tagged_case_suffix_values_valid rather than bless it.
217discovery3841964 captures that companion. All four high-level imports
still fail combinatorics.at944 in Rust; markers after abandonment are not
success. Older RUNNING/NOT_SUBMITTED statuses below are historical.

NEXT parser frontier observed in running212capture3841698: groups.at now
loads without diagnostics, but deform/Hodge stop at combinatorics.at944's
`(cycles).unsplit_class:`. Original succeeds. CaseBranch omits the suffix
production already present in DoCaseBranch; no repair applied yet. Local
discovery catalog expands212->215 with three positive/negative companions,
126accept intents. Stage atlas-tagged-suffix-reference-20260929.ohyTLwXv is
being prepared against exact523standard release3841793; NOT submitted yet.
See [suffix-tagged case](slices/tagged_case_suffix_2026-09-29.md). Markers after
an abandoned include do not imply loading success; require empty diagnostics.

LATEST R4lazy3841689 COMPLETE:523core/6newlazy/owner-control, CLI release,
final1423file integrity. Report9e14806e2efcaabefa88558cd9285a01a4005c5657a97b3ae6d70cff645b735f
verified local/HPC.212capture3841698 now RUNNING; no differential acceptance
yet. Standard release3841793 RUNNING at atlas-lazy-release-r4-20260929.09i46hhs;
preflight3841794 PASS(SHA0a313d811a3b09bfead7ea9bffb0f3f8de4b1f11a137d7ade8d54a51478845d5).
Next: collect build.json, require all1423source hashes equal finalized3841689,
then submit ALL116unchanged cases and independent full/CPU reviews. Copy
CPU/fat indices from prior direct suite receipt; pass SUITE_INPUTS_SHA256 to
BOTH arrays. Current new suite pin d0314d5086c138d3b3510822bbd23f28e5eb64cc7986bef72b4cffbec3d031b8.
Exact archive/baseline/harness pins in math_lazy_real_form_r4_release_submission
receipt; see lazy_real_form slice. No arrays for this release submitted yet.
GitHub refs rechecked via login-node Git: unchanged05625c5d/7e1b958c.

NEW exact517 E7/D8 KGB A/B independently VERIFIED3841670/3841690; complete
four-round outputs equal original and across thread counts. Paired medians
2.183940/2.303566; Rust4 still sloweroriginal4.39x/3.55x. Both below60s.
Local review SHA e049496c5559d67da40ab758b8ba58c2787934a99c4a0b3b214b008a863b1a1c
and eaef5768366ad7df363975fff569687f93d6239017acf1c0f10ea82b5ac2d776
match HPC. No global thread default or lazy523 acceptance. Source/stage pins
in updated math_direct_parallel_submission receipt; see parallel_ab slice.

LATEST lazy R4build3841689 is confirmed RUNNING on2026-09-29; downstream
212capture3841698 is submitted afterok in atlas-lazy-capture-r4-20260929.X1AvZnF2,
pin7f374d9a98700c820d4dd3d787fa92a0bd401c04f5b8aaf48916ff56c7a4f28f.
R3failed test SETUP (numeric id_mat called through domain-only dispatcher),
not a mathematical assertion; preserve its report and all metadata assertions.
R4uses the same identity Matrix directly. R3capture3841649 was cancelled
before execution after DependencyNeverSatisfied; R2capture was never submitted.
Full116eager517 arrays3841517/3841518 remain confirmed live, reviewers pending.
See lazy_real_form slice and R4submission receipts; older statuses below are
historical. No lazy or high-level mathematics acceptance yet.

CURRENT lazy R3job3841627 at atlas-lazy-real-form-r3-20260929.GPszxuyT
is SUBMITTED(523core), pin4930e53b399688ed52f53e572fd8d565f12ffcb57884687637b4599a3de65a71.
R2job3841592 FAILED only at the new test's nonexistent KgbId::from_usize;
report e2ee7d94af31a1b3a12187b592bf163a5cd64aa188ace96e4f0e281745451473
verified local/HPC. R3obtains a real id from a separate healthy A1 graph,
preserving the poisoned form and all assertions. No new lazy math acceptance.
Prepared capture R2stage gCm2C1xu was never submitted; R3uses fresh Q7hNECBi.
GitHub refs rechecked on HPC this turn, unchanged05625c5d/7e1b958c.

Lazy first3841545 FAILED at check --tests(E0425/E0277 in two renderer helpers);
report ca92f3e4f7d9984c9a5d975981dd6270ee008a4e99fcf84b12bbd6d072dbb41f
verified local/HPC. R2job3841592 at atlas-lazy-real-form-r2-20260929.ij3f77KY
is SUBMITTED with fallible renderer/span propagation and expanded error-span
coverage, still523tests. Pin b0aca725dde206c3a8b69060b72b97422c381227064b245fe8f70285df83c096.
No lazy runtime acceptance yet; source hash fd968063bcc66722db520f4bd83253f04a4e7bc323e2e52dd576d17aa784307b.
Collect this fresh stage; never overwrite the first failure. The separate
212capture driver is prepared locally as math_lazy_real_form_capture.sbatch.
Full116jobs3841517/3841518 still verify eager517, not lazy523.

NEW lazy-owner candidate3841545 at atlas-lazy-real-form-build-20260929.tCqyK5kr:
only domain_builtins.rs changes from517integration3841489, eager validated
seed plus separate fallible KGB/Rep_table initialization. Six new tests,
expected523full core. UNVERIFIED; local renderer/span corrections after
this first frozen submission are a separate next revision. Preserve failed
compiles/inputs, do not mutate tCqyK5kr. Source/ownership rationale and gates
are in [lazy real form](slices/lazy_real_form_2026-09-29.md).
Original212capture3841527 COMPLETE, report2b565b49ef2dbbf8466fb89c746fb70c270e5575d7596314d4a0a9a36e93776a
verified local/HPC: all three full metadata D8/E7/E8 cases succeed originally,
unchanged eager517Rust times out45s in each. This is before evidence, not
lazy acceptance. Full116jobs3841517/3841518 still test the eager517release.

CURRENT2026-09-29 integration3841489 COMPLETE:517core(0ignored),10Cartan,
all retained generator controls, CLI check/release and1423-file integrity.
Report0ace15ef3825df8daad56837a0b5dac152a3c6880f3cfe2e0bd9b3c84964d1ea
verified local/HPC. Interpreter budget/cache switch IS wired; historical
UNUSED notes below describe the earlier kernel. Three-arm209capture3841497
COMPLETE at atlas-direct-integration-capture-20260929.pjHD2LfH:
98whole positive matches,3gains/no losses; all full D8/E7/E8 Cartan outputs
now equal original. Report0fb1483b85a7c488a3e966b31cb846f13d9f6634b51f31fa992a139865f9c61b
verified local/HPC. groups.at/four library imports still45s timeout; original
signals retain aggregate CAPTURE_FAILED_ORACLE_EXECUTION. Not high-level
acceptance or repeated A/B. Large successful timings in direct-generation slice.
Standard release3841502 at atlas-direct-release-20260929.JvotR2Mh COMPLETE;
all1423source files equal3841489. Build SHA
f63f32d05e8bb41306019330d813d2bdce3bf68b4f6a2afd1817fd735c843090
verified local/HPC; preflight3841503 PASS. ALL116cases submitted under
CPU3841517/fat3841518; fullreview3841519 and CPUreview3841520 follow.
All prior inputs/timeouts/caps and Rayon1 retained; source/suite pins are in
math_direct_involution_suite_submission_2026_09_29.json.
Original209reference3841464 is COMPLETE, report606f3b6e1204cf213a07e3c8dd4cd89996f7d4c43dfd86e626b6130b8b434939;
all full D8/E7/E8 original inventories succeed and confirm17040/10208/199952.
Current local generic catalog212 adds provisional metadata-only D8/E7/E8
constructor contracts before any lazy-owner change.212capture3841527 at
atlas-real-form-metadata-reference-20260929.vCM0eLGq is submitted against
exact517release3841502; frozen209/116jobs are
unchanged. See direct-generation/library-context slices. Full high-level
math, general cycles, Hodge and controlled parallel A/B remain open.

NEW direct-generation prototype3841440 VERIFIED:32complete compact/lattice
sets through D6/E6,40prior partitions,23unchanged controls, D8/E7/E8 closure.
Report40c4bdd2d30b6fb897b4bf26e9718dcf2a5f2cf20ada2cb93b1f7727cb86b015
matches local/HPC. Candidate3841461 adds two UNUSED production APIs, not an
interpreter switch. Full partition/large-group kernel3841461 VERIFIED,
report9cc355680ad943d48c8f7734a4674dd7d408cfc149f6f3605a070fbd09805618
matches local/HPC. All32exact partitions pass; D8/E7/E8 construct16/10/10
classes with17040/10208/199952members. APIs remain UNUSED by the interpreter.
See [direct generation](slices/direct_twisted_generation_2026-09-29.md).
Original-reference corpus now209cases; initial3841457 fails at a nonexistent
release report.json before execution, corrected build.json R2stage dw4NDPzU
uses identical inputs/binaries. Collect its receipt/results before goldens.
R2job3841464 is confirmed RUNNING; do not restart while its handle is live.
Current516CPU review3840814 completes86cases with40math/7reject matches,
21Rust failures/18original failures;40includes3language/load cases. SHA
1cd09569dc299f60a0f49906f95c0c71536a57ba096f40da0b17c853ef3d598c
verified local/HPC. Full fat/review3840812/3840813 still separate and pending.

NEW terminal high-level barrier identified on exact516release: valid R2
A2_klv observation3840825 succeeds in original but Rust fails after91.23s
at groups.at251 E8_ic with the4000000 full-Weyl enumeration cap. Report
fda0c875e8cda340365856ddbe9f0ebc293dc150ff39fc18833019ec5a6a0ef2
verified local/HPC; both actual timeouts600s. No speed ratio from failure.
Earlier debug stack R2job3840859 identifies groups.at237 E6_s/external1,
rank6, eager KGB work; it is not the later terminal E8 barrier or evidence
of a reduction-loop bug. Reportbce3b6955ec17ac0fcd5c795f27b45efa58411f8ab881efe5645078489992114.
See [library context loading](slices/library_context_loading_2026-09-29.md)
for exact evidence, safe lazy-owner boundaries and the next source-backed
direct twisted-involution enumeration lead. The generator-orbit experiment
alone does NOT remove E8/D8's first full-Weyl traversal. Preserve all unmodified
upstream scripts, original failures and full high-level math requirements.

The first516suite arrays3840709/3840710 fail BEFORE Atlas at the strict
input guard: SUITE_INPUTS_SHA256 was exported only to preflight, not sibling
arrays. Preserve all raw HARNESS_FAILURE/empty-observation reports and
original reviewers3840711/3840712. Fresh suite R2stage80u4BXD7 reuses the
same516release build/inputs and explicitly exports the hash to both arrays;
no rebuild, no source/fixture changes, no mathematical conclusions from failure.
R2jobs:preflight3840810,CPU3840811,fat3840812,fullreview3840813,
separate CPU-only3840814. Inspect this stage80u4BXD7 for valid math runs;
original release stagezxhpXoLN retains the build and failed first submissions.
Generator experiment3840707 COMPLETE:4new tests/40full equal partitions,
missing-image rejection,23unchanged controls, final source guards. Report
e2cf3107ef4fe90530dfcceb85bd4e24234ddbcf5a78a4cccdd5e61bf9522d7f
verified local/HPC. Production remains unchanged; original-backed end-to-end
and controlled performance comparisons are still required before selection.
Exact-binary groups.at stack diagnosis3840788 at qcjtLejN stage is separate,
30s deliberate GDB interrupt of its own process only, not a math acceptance run.

Release3840661 COMPLETE, report9cf3bd131d9aa12550fa5013f052aa54dfc1233edf37a16919e646341a3ce185
verified local/HPC; all1423Rust source files exactly equal516foundation3840634.
ALL116cases submitted:preflight3840708,CPU3840709,fat3840710,fullreview3840711;
separate86case CPU-only review3840712. Preserve600/1200s actual timeouts,
Rayon1 and6GiB child address caps. These exclude the test-only orbit addition.
206loader discovery3840662 COMPLETE, report
086bee469851c525c0f10f025cee90a41fe43c73af72f4dbf616380a4488a948
verified local/HPC. All four newly added library imports are accepted by
original and hit45s Rust debug timeouts, as does groups.at. Preserve those
failures; no successful load, mathematical result or bottleneck location.
Test-only generator-orbit experiment3840707 (oytmODYw stage, receipt indexed
in twisted_orbit_search slice) compares40full same-input partitions plus a
missing-image rejection and23unchanged inner-class tests. Production prefix
must remain identical to516foundation. No runtime algorithm edit or speed
claim; future end-to-end original and alternating A/B gates remain mandatory.

FINAL516foundation3840634 verified local/HPC report
bb020272fb63cb71d3664cc3f4bb34b7445be7e93f5fd1875f98b4c07c1a8ed8.
All516core, CLI and final source/input integrity pass.202capture gives95
whole positive streams versus91/no losses: torus radical shapes, SC bases,
central alcove and Lie-type extension. Negative extension retains17exact
Runtime messages/recoveries in session; CLI stdout/category match but stderr
envelopes differ. Original failures remain, not waived. groups.at now reaches
a45s debug timeout with EMPTY stdout/stderr; source-level exceptional-group
eagerness is a lead, not a proven location. Do not invent a last executed line.
Release3840661 at atlas-lie-extend-release-20260929.zxhpXoLN has the exact
516source archive5c3ece3acab73e042a55ec0c666d446ea0071f20d8fd5887bbe882528d5786e4.
After completion require all1423source hashes equal foundation, then all116
cases under unchanged resource/time bounds and independent reviews.
Separate206case discovery3840662 at atlas-high-level-load-discovery-20260929.sgs3vbPb
pins44aecf84135011abafc20a6fc295919e660d94f8c3dae8c24ae2ed6b7e98fdb6;
it reuses the same eligible debug candidate and adds unmodified deform.at,
hodge_test.at, AV-ann and cycle-library imports.119accept/87reject intents;
loader success alone is not mathematical acceptance. Full receipts in
tests/reference/hpc/math_lie_extend_release_submission_2026_09_29.json and
math_high_level_load_discovery_submission_2026_09_29.json. Older pending notes
below are chronological and superseded only for these exact job scopes.

SUBMITTED combined516-core candidate3840634, stage
atlas-lie-extend-r2-build-20260929.uoMb985Z. Pin
762b50b8e4bcc11fa1b3e45ea20e2ee681c777c884dd16f1e3b441c082e91b15;
patch0be61a2fc00cfd48c699934b6778005eb3fb2c808aea92f819b8e3cc0edaa620.
Includes torus0xN/columns/adjoint repair, original3840609-backed extension
validation/Tn normalization, and original3840488-backed historical assertion
migration. Requires the exact registry unit separately, both extension units,
three torus units, all516core, CLI/full202capture/final integrity. No pending
or failed predecessor is counted as a completed foundation. The provisional
W7opH9De516 stage was never submitted; use only the R2job/pin above.
After success freeze its exact source for release/full116shared consumers;
collect current groups.at frontier before making high-level claims.
HPC GitHub ls-remote recheck2026-09-29: originalmaster remains7e1b958c7aa9456769cc9cf09ac1542814b4800a,
Rustmain remains05625c5d2b5f68ffc75c66c3f5f12302d25c7c30. No pull into dirty checkouts.

FULL514candidate3840560 FAILED513pass/1fail at the historical typed registry
assertion adjoint(A1.T1,false) rejects. Original3840488's exact28datum case
accepts it and prints complete matrices; current local assertion is migrated
to that captured RootDatum/value without removing any other registry checks.
Reportc978f7c1ead7cca7511234b7ee26e227f31a1c5430ff368e2da57d299419871f
verified local=remote. Three focused full torus goldens and kernel3840555
passed, but514full gate/final integrity did NOT pass; never reuse it as an
eligible completed foundation. Next combined516candidate contains the fix.

Discovery3840609 COMPLETE202cases. Original accepts all90torus extensions,
8simple factors/rank32/G2involution controls; unchanged511rejects zero-rank
G2involution and aggregate Tn normalization. All17negative sites reject
originally; Rust accepts15 and reports wrong Type category on the other2.
Report12ea2be7cec26eb1201600b35c9985fcbbade5edce43c717ad2bb024d9d517c2
verified local=remote. Full original goldens and2session tests preceded the
new add_simple_factor validation/normalization and signed32 wrapper repair.
See [Lie type extension](slices/lie_type_extend_2026-09-29.md).

Exact511CPU review3840622 COMPLETE:86cases40math/7reject/21Rustfail/
18originalfail, same id/status set as509review3840490. Report SHA
30622571198e3c6448cd268013a78d54068d6e88f57e00c40f32673763bb2301
verified local=remote. Full116review3840522 still awaits fat jobs; it must
not be replaced by the CPU-only result. No new high-level acceptance.

Kernel3840555 COMPLETE:2before failures,2after passes plus7root-datum/
2alcove controls, final source guards. Reportc32270f55d3d45aa96d4d7dbe7bd4cad6baafbbafeeb0ff09ce2f7e9b1d1be70
verified local=remote. Full514candidate3840560 remains separate pending gate.
New discovery3840609 at atlas-lie-extend-discovery-20260929.sIZ1S0l7 retains
all200previous inputs and adds extend positive/17negative sites (202total).
Pin58979c8353a62c7f325b02bc79d5886430d8ba8db69532fe32c4fed4777a04e1;
unchanged511-core parent3840381, not the radical candidate. Source upstream
atlas-types.w166-210 normalizes Tn to n T1 factors, so T0 appends nothing;
wrapper280 narrows signed32 before first-byte letter validation and enforces
validation even at no-value. No extension runtime edits until capture.

SUBMITTED torus repair3840560 (514core/200capture), exact pin
8367c2f98d845292d54d760672d296fb9f74e9f5a2b9a981d402efd66d21a721,
stage atlas-torus-radical-build-20260929.z5MKwZE0. Complete original goldens
and three session tests preceded runtime edits: retain0xN annihilator rank,
export root_coradical by columns, remove wrong adjoint torus rejection.
Kernel3840555 independently uses identical before/after root-free basis
tests on verified511parent, fresh targets; expected0pass2fail ->2pass plus
7root-datum/2alcove controls. Neither gate has completed yet.
The separate extend(T,0)/groups217 frontier remains unmodified.

Verified release3840501 build hash
0c3d97a6006b4f2d5c432d96db11b17b668718482d520c5360042a7e20b2ed7c;
all1413Rust source files equal511foundation3840381. All116math cases now
submitted under preflight3840519, CPU3840520, fat3840521, review3840522.
This immutable release excludes the torus repair. Keep all previous failed
original cases,1200s fat timeout and6GiB child cap; no new acceptance yet.

NEXT: torus R2 discovery3840498 COMPLETE on unchanged509-core. Original
accepts SC-only20data and all14pure-T1 central alcove controls; Rust reaches
all20then fails torus rank, and fails alcove with no unique solution.
B2 root_coradical confirmed transposed: original[[2,-1],[-2,2]] versus
Rust[[2,-2],[-1,2]] in both numberings. Report SHA
c54d6176dcc34680ff11d3233270c9cd1284b9a5317df20a2e28ca7f6f9df2d6
verified local=remote. Add complete captured goldens/direct empty-kernel unit
before repairing radical bases, exported columns and the adjoint torus guard.
See indexed torus_radical slice. No radical/adjoint/extend runtime edits yet.

SUBMITTED511-core release build3840501 at
atlas-empty-datum-release-20260929.qseNB51P. Archive
6f90eb13cd948dd55d060fecd91be442260b53feaae7ee856ed46367170b375c;
baseline6a2a3951b8cddd6252fd3206ca90fb0c72ce1592fed8eba467d7851032a77bff.
After completion verify source manifest exactly equals3840381, then submit
full116math cases using its own suite/review pins (see submission receipt).
Not a build of the as-yet-unrepaired torus/radical or G2 loader issues.
The new groups217 frontier calls extend(G2,"T",0), while add_simple_factor
currently unconditionally appendsT0; capture exact original extension and
involution boundaries before changing that separate owner.

FINAL empty-datum3840381:511core/CLI/final integrity PASS. Full Nx0 positive
matches stdout/stderr,91whole positive streams versus90/no losses. Matrix
negative stdout and seven exact session diagnostics pass (CLI envelope differs).
Report SHA ee1f1c78efad82834d15fc4744ffbd50084a7157ee7c8d99d99085ac33c82dcd
is verified local=remote.
groups.at now fails at217/basic.at1744 involution(extend(...)) with
`Too few inner class symbols`; not a fully loaded group library.
All earlier pending511 notes are superseded by this final report.

ACTIVE torus R2 discovery3840498 at
atlas-torus-radical-r2-discovery-20260929.hz3VkZ64,200cases, unchanged509-core;
pin6b941802e2893c677c034c039bbd708b6c0cf669d0b7d87969eb94b26a9a703d.
Adds separate SC nonsymmetric basis controls and pure-T1 alcove-center
central-character invariance; previous198cases retained. No radical/adjoint
fix yet. Frozen511capture remains197 and must NOT be enlarged in its stage.

Signed509-core CPU review3840490 FINAL:86cases,40math/7reject/21Rust failure/
18original failure, unchanged id/status set against506-core3840370.
Reportd039e0ef3c999e870f4cbe49d5981a4d6aea4d2ef187da05c3aa21ac82ccb4ef
verified local=remote. Full116review3840420 and older3840271 await fat jobs.

NEW confirmed torus3840488: original accepts28data/8554stdout bytes. Unchanged
509-core loses SC T1 (co)radical basis rank (0x0/1x0 instead of two1x1
identities), then rejects adjoint(T1) with bogus sub-lattice-size error.
Reportbeb56397b793b197edebf9e025a42294f815b92de858202d9dc83df4cca38d04
verified local=remote. See [torus radical frontier](slices/torus_radical_2026-09-29.md).
No radical/adjoint runtime changes yet. The loop abort masks later B2/G2
orientation controls; capture a separate simply-connected companion before
changing root_coradical's row/column convention. radical_basis feeds alcove.

ACTIVE empty-datum3840381 at atlas-empty-datum-build-20260929.bKJv2oNi:
both original-backed empty positive and seven-negative session units PASS;
full511core/CLI/197capture/integrity still pending. Pin
bb5e180986956a30b12a5823de4ad5e84c90a4621d95e8f5760b3753c213b81a.
Root cause and matrix-shape owner are indexed in
[empty explicit root data](slices/empty_root_datum_2026-09-29.md).
Original discovery3840368 positive Nx0 accepted/Rust rejected; original
negative conversion SIGNALS. R2job3840376 confirms seven matrix Runtime
rejections and isolates exit139 on empty vector-list conversion. Reports
7677aebd37812f7aec961876a6cf82b0a0eb005673153121da47ec2522ae2200 and
191a2e722a1f9baab4b78a4f29060de7665269df67bb8ca3dcb4aa69a2c3b0b7
verified local=remote. Keep both signal cases; they are not rejection goldens.

Signed-rational release3840371 BUILT;1407Rust files exactly equal509-core
3840284. Build08154093c59ded1e0b212a471b798529679cd95d5ca93da5e28902beefd676ac.
ALL116 math cases submitted: preflight3840391, cpu3840394, fat3840393,
review3840420 at atlas-signed-rational-release-20260929.d92XqWS0.
Separate CPU-only reviewer3840490 waits afterany3840394 for all86CPU cases;
keep its acceptance scope distinct from the full116review3840420.
This release excludes the new empty-datum fix. Older506-core CPU-only
review3840370 verifies86cases:40MATH_MATCH,7rejection matches,21Rust failures,
18original failures. SHA02264ccc92df7f2273ef45852e547e0141dfe37152d0c6b5f923a04f9f3ac5ca
verified local=remote. Full116review3840271 still waits for fat3840270;
do not substitute86for116 or compare different case counts as regressions.

SUBMITTED discovery3840488 (198cases) adds torus_radical_shapes on unchanged509-core, separate
from frozen511candidate197capture. Existing SC/adjoint torus and classical/
G2 product constructors expose possible empty kernel dimension loss and
root_coradical row/column orientation; prints all28data before final shape
assertion. Runtime radical code is NOT modified. Stage
atlas-torus-radical-discovery-20260929.VBmln3Ho, pin
47093f8468262ecaee304c4c4ff11f7078657f3d333e9698b2b896cd151138a0.

FINAL signed-rational3840284 passes509core/CLI/final source integrity and
both full original signed fixtures;90whole positive streams vs88, no losses.
Report59e3c64969a75e0b4671c5b50020d73e339bb91d46cad23021b42460000ce89a
verified local=remote. New release staging at
atlas-signed-rational-release-20260929.d92XqWS0 must use its exact source.

Next common high-level frontier:506-core release3840269 A2 KLV3840274,
A2 AV-ann3840280 and A1.A1 cycle product3840341 all fail at groups.at3 /
basic.at1180 root_datum(null(rank,0),null(rank,0),false). Capture195cases
3840368 on unchanged509-core at atlas-empty-datum-discovery-20260929.9bKYzbue;
pin04cc1253702fe62e6e4e7ce2220cd3509a2e3885349b0a08d1486bce4e6613a8.
No empty-datum repair yet. Preserve actual Matrix dimensions: as_matrix_rows
currently represents0xN as N empty rows for other validators, so blindly
removing build_explicit_datum's empty guard would wrongly treat it as Nx0.
Capture positive Nx0, negative0xN/mismatched shapes/Cartan, discarded calls
and empty-vector-list conversion separately before changing the constructor.

FINAL projection3840261:507core/CLI/final integrity PASS, full190capture
88whole positives vs87, gain signed projection/no losses. Complete original
T2/A1.T2 stdout/stderr equality. Report SHA
70231826547bff4eaf8d3068ca5241fd17cd9a85dcb90b6c9f7bc46d25d10914
verified local=remote. Kernel3840260 before-fail/after-pass proof retained.
Overall oracle-unavailable status preserves separate original failures.
This is the latest fully unit-verified candidate; local next509-core signed
rational changes are submitted under3840284 and not yet accepted. After
its gate, make a combined release and revalidate shared alcove/projection
consumers; currently running116math jobs use the EARLIER506-core source.

LATEST active signed-rational3840284 at
atlas-signed-rational-build-20260929.K802DBJt; pin
61ef13af8e7c70b706cb95230321e8614e249fa958adb16c1a3b82449b61d280,
patch21431227035462f899aaaa4a3062fed2463687482c2d1254f6c40980788372a5.
509core/192capture expected. Discovery3840264 COMPLETE, report
3b5e79939d6c13c24923d2ab67866a04a22c270e5069583e1f07aa42c13eee25
verifiedlocal=remote: original passes signed root reconstruction, Rustfails
A2negative coroot; original negative A1alcove centers stay negative, Rust
prints positive. Explicit regressions precede candidate; two exact signed
TryFrom<&Rational> conversions ONLY. See slices/signed_rational_2026-09-29.md.
Other numerator_ref sites remain audited separately. Candidate includes the
projection repair; broader alcove-consumer release revalidation still required.

HIGH-LEVEL revalidation is SUBMITTED: release3840262 exactly matches506-core
3840188(1402files), report6f32d288000d2f4a5381c12da41b61636e9ce3c0f6ff70db78ff4e3b1d43adc7.
All116cases under preflight3840268, cpu3840269(86indices), fat3840270(30),
independent reviewer3840271. Stage atlas-polynomial-math-release-20260929.vNRQbi8f.
CPU600s/fat1200s, unchanged6GiB child address cap, serial Rayon1. Separate
review submission JSON lists every index and pin. This source excludes BOTH
the newer projection and negative-coroot/center fixes; no high-level results yet.

K-type503-core review3840192 FINAL:45math/9rejection matches,7original failures,
2Rust failures/2timeouts, same65case statuses as previous3840164. Report
39651dc556d2b496b5f0df1bc8355b4e40d5d3ebd23ea6907cbb1bf575589311
verified local=remote. Original/E7allocation, sixHodgeerrors, D8capacity,
E7FPP/Cartan1200timeouts and cycle-product basic.at2437failure retained.

FINAL3840188:506core/CLI/final source guards PASS,189capture87whole positive
matches (gain polynomial_coefficient_values, no losses), report SHA
4284e529c879b5069448d7e513d4189a354e6c4cde5999d4251bfc592fed9bd3
verified local=remote. basic.at now loads COMPLETELY/no diagnostics, but
declaration indentation/type-variable names differ, so not a full-stream
pass. This opens actual high-level revalidation: release3840262 at
atlas-polynomial-math-release-20260929.vNRQbi8f is exact506-core source.
Archive128021d6ddc633547f9635da82e8235ae25d41880d0c581a16870c634eef92ac;
baseline3cefa924999181d46fb3f8a4ce08055fc89a253e780f6f2246be294194859564.
After build require source-manifest equality, then ALL116math cases (not
only old65kernel subset), including unitarity/Hodge/AV-ann/cycles. Failed
original executions and old regressions must remain. No projection repair
in this release, and no new high-level math pass is claimed yet.

Kernel3840260 FINAL passes required3before-fail/1before-pass,4after-pass,
9KType/10context, source guards; SHA567c684e1b00a38e296f997a6cebe5b2d970802612d800ac2a4248104eec3097.
Full projection build3840261 at atlas-projection-build-20260929.iPOx7YMI
has passed the full original signed session assertion, pending507core and
190capture. Pin362b90d05764c7a25711ec5c76d6718739779a01566445dc01fafac7aed35700.
Separate next discovery catalog192 adds signed root/coroot reconstruction
and negative A1 alcove centers; frozen3840261 MUST stay190. New discovery
stage atlas-signed-rational-discovery-20260929.VC9RGGAn uses verified506-core
and pin4051f6dcfa7fe6e4f3239636157ffedd6182fc238b1e93ecdacb851638080df4.
SUBMITTED192case discovery job3840264; no runtime repair to its new sign sites.
Release3840262 suite-inputs909808f6350a9ebef7f02ee212f0b24d995831f34b4cdd6ea5d49cc3eed50a0a,
review-code0cfa499494050110bc38dac907d570e3910a96a4eb6eb1fdb454af8023189c58,
input-list3599149dda44dce293b6e559c22a944d5e0465d74657ac1c2a888812d9a8475b.
Recheck these staged lists before downstream sbatch; baseline differs from
the older K-type release. All math observations currently impose6GiB child
address-space limit even on fat: distinguish this from scheduler memory
and retain allocation failures; any increased-cap experiment must be separate.

LATEST signed-arithmetic2026-09-29: polynomial3840181 FAILED unchanged
original positive output (nu sign), reportd251b7f640db857647f9060799b5c88249154c29cb225a4c3c3c10b5a09e0200.
rational_pair lost Malachite's separate sign. R2job3840188 at
atlas-polynomial-read-r2-build-20260929.olrcMVXb restores sign BEFORE i64
narrowing; signed/boundary helper plus both full original coefficient
positive/negative units pass.506core/full189capture/final gate pending.
Pin34968106bc97273db68be67f1a561f6adb011f082c8d79fe1c5b05be31f3a20d.

Projection discovery3840186 COMPLETE, report62c21127f656efbf0ca10a06db6c806ec0e9e130577b0add690b8073b4f2812b:
original/Rust both accept but whole T2/A1.T2 output differs; coset assertions
pass despite opposite canonical image-basis orientation. New four direct
tests and complete session golden retain original expectations. Candidate
changes only gcd_sweep quotient to div_euclid; see indexed
slices/signed_projection_2026-09-29.md. Kernel before/after3840260 at
atlas-projection-kernel-20260929.vAMLjuDd uses exact503-core3840172 parent,
new test bodies IDENTICAL before/after and independent targets. Pin
3aa472900351da64ffbe83e44181a1849a1a224c5164cf819c4ba5be4bbb2c0c.
Full projection candidate stage atlas-projection-build-20260929.iPOx7YMI is
being staged separately; do not treat the edit or kernel job as acceptance.

K-type release3840182 verified1399source files identical to3840172; broad
65case jobs preflight3840189, cpu3840190, fat3840191, reviewer3840192 are
live. No sign/read/projection changes in that release. Submission indexed
math_ktype_euclidean_release_review_submission_2026_09_29.json.
Corrected kernel3840183 COMPLETE:9KType/1B2anchor/10context and source guards
pass, report9d690dd9ac1fc66072af824f168cb4f7d91e03defbf6452daa7500028c6ede3b.

LATEST2026-09-29: verified K-type3840172 report SHA
a1a2fcc393d96b204a911ddb7bf8a0f74fab849e638d6f3b708efc6289efce9a
matches local/remote:503core/CLI/final integrity,85whole accepted language
matches versus84(no losses), including full raw/memo formula stream.
Separate189case replay3840179 SHA
f00a154d9f9cc8c40601cfd01be6319c823d7862de42cf7ecf83199dfe282120
adds the complete370107-byte coset match (86total,no losses). Original
and fixed Rust accept all five groups; old500-core3840085fails3840175,
report2de9547f6d8e50ae2f7599d7291648360536ae3c38579dd7f52c79252e92d686.
Retained original failures keep overall oracle-unavailable status.

Independent selector release review3840164 is COMPLETE:45MATH_MATCH,
9REJECTION_CATEGORY_MATCH,7ORACLE_FAILURE,2RUST_FAILURE,2RUST_TIMEOUT.
Reportc09b33955576c3baa3ade1625a492742d92df59920b35bb09f5ac363cf4c76c0
verified local=remote. All seven completed Cartan cases including F4 now
match; E7FPP/Cartan actual1200s timeouts remain, D8cap remains, original
Hodge failures and Rust cycle-product basic.at2437 loader failure retained.
This release3840096 PRECEDES the K-type Euclidean/cache change.

ACTIVE polynomial READ candidate3840181 at
atlas-polynomial-read-build-20260929.jiZfrYGi; pin
3236d7c21e0b3a2680c2b248091d8799cf4039e07d7a0b569d981cf96cca7c3f,
patch83a8d68400e6952d642c7e74bee6438524edd1a003f77c6e42e821fa226552af.
505core expected,189frozen capture. Includes verified K-type correction.
Read units retain complete original3840100 output and6Runtime/4Program
errors. Writes still unimplemented and cross-real-form behavior unresolved.
See slices/polynomial_coefficients_2026-09-29.md. Local discovery catalog
has advanced to190 separately, adding signed projection/T2/A1.T2 probe;
do not edit the already-submitted189case stage.

ACTIVE shared-kernel revalidation release build3840182 at
atlas-ktype-euclidean-release-20260929.okABIdkY, exact3840172source archive
46c98dbaef1ff01bb6a64d6f45ad1663328124d12dea76a1cf427ae48ecf2d0d;
baseline27f13b327f40b6e4eacdf69966d37b5e42ae24f80af87f98e2c7ace561440eaf.
After build verify source manifest equals foundation before submitting the
full65math cases (same controls/failures as3840164), not only K-type tests.
KernelR2 job3840183 at atlas-ktype-kernel-r2-20260929.fwgEjQ43 runs same
503-core source using script6d43796fd07b121868e3a18ec92b5d80d31b747b91daa00c035deffc984066b6.
Firstkernel3840173 FAIL retained, SHA
fc1a5e32153e2346fe564cbf26a68a296d6393dc2d2dbf6a8c8814a809bcb37d:
9KType/10context pass, nonexistent real_projection::tests yields zero.
R2 replaces only that invalid selection with actual B2 transported-lift unit.
No direct RealProjection test coverage is claimed. Its gcd_sweep still has
a false truncating-division comment; investigate before altering mathematics.
Both public Git pins rechecked unchanged7e1b958c/05625c5d. No commit/push.

LATEST ACTIVE: broader coset discovery3840175 at
atlas-ktype-coset-discovery-20260929.kGR2fit0
captures189cases against UNCHANGED3840085; pin
badc5ad43e50f12f8cf373327dd5b452813dda7cc38ec1955f295db5b845b90c.
208inputs verified. New fixture checks strict K-type equality under +/-
(1-theta) basis images on A1/A2/B2/C2/G2, all KGB elements, both numberings,
SC/adjoint and both identity signs. This is separate BEFORE evidence; rerun
it on an eligible fixed candidate before accepting the broader invariant.

K-type R2/Euclidean candidate3840172 at
atlas-ktype-euclidean-build-20260929.ScMf7WNk, pin
290118bbb650c53b32e87def1c3d53b8f8c7d97aad7009e1423f6eba87df63a7.
OBSERVED: both unchanged full-formula/signed32 session units now PASS, plus
the owner-isolation/high-cutoff Arc-reuse unit. Normal stack. Session log
f123cd98acc6ff448f755cfbfb9854124bc685583c996da438ffbdbc7522062f;
cache logb00adc5f72a487516e1bf71d2c95ddebb2b4b103ce0e70ebd501197004c4108a.
Full503core/CLI/capture/final source-integrity gates are still pending;
do not treat these three passing units as foundation or whole-math acceptance.
Patch9176a60dee998675fbbfa5ce182a00d4ee03ab1b3956a844e232710b3530a797;
Separate targeted kernel job3840173(afterany3840172) at
atlas-ktype-kernel-20260929.DJo8g2lJ runs all KType, real_projection and
rep_context unit filters against the same frozen source in a fresh target,
rehashing before/after. Scriptf581467dfff971074117ab94e47f6640b5f8fb2b737ef450e15c8de30acde0e4.
original184case capture and503core expectation retained.3840098failed both
new units: negative test counted internal void recovery Values; positive
proved A2 wrong lambda keys/term coalescing. Only observation guard and
RepContext::lambda_unique v.div_euclid(2) change in R2. Original mathematical
expected file unchanged. Failed report9aba8592de7f40c9c2e92f016715ea2b02c5705b1a16c4f6ca503b10fec20b6e retained.

Release3840096 COMPLETED;1396source files exactly match verified3840085.
Build reportbd0f4cc72dd9b0ef6aab46c0341e3900505ef61dee2197228a7a16f9ea0f2f01.
Preflight3840102;65case arrays3840103(cpu),3840104(fat),3840138(extra
partial-KL history103-106); reviewer3840164 waits afterany all. Corrected
review-code list4f3ab0e6790daa25ff7217f6bde59b84ad2029bd8d66dd345709579696b73d53
includes this stage's new baseline hash. Wrong review3840121 was cancelled
PENDING before execution (script hash/index syntax); computations untouched.
Source3840085 does NOT include latest K-type/Euclidean changes.

Polynomial discovery3840100 COMPLETE:188cases, unchanged3840085. Original
accepts coefficient read/write positives, Rust rejects types. Original
RECEIVER first ORDER12; Param read makes a standard parameter dominant.
Cross-real-form writes unexpectedly succeed in original, unlike reads;
preserve actual output and investigate, not a guessed rejection contract.
Reporta0481dbd77ebef303c7d76b8e3f0d7d9b1cbd33b8af7556f450b512a09d4e072
matches local/remote. No polynomial subscription implementation yet.

ACTIVE K-type candidate3840098 at atlas-ktype-formula-build-20260929.REiqgPmT;
pin2e024668fc1e7a53099b0198c7d6f15a647631b7f8fe2030ef9786874054698a.
203source and205remote inputs verified before sbatch;503core/184capture
expected. Full original positive report and six signed32 errors required.
Local discovery catalog now188 (adds four polynomial read/write probes);
that discovery is separate from the frozen184case candidate. No after-pass yet.

LATEST verified results2026-09-29: F4numbering3840083 passes497core, CLI,
final integrity, and the focused eight-variant full original output. SHA
bced548aa2bf4b579ff6e168613e9b386daabad6bf98a7e9514822a83ab4a612.
Selector3840085 passes500core(0ignored), CLI/final integrity; report SHA
acc0a32786b8609076c08f70c708da0ea52d50891345efe6187d194a27606ba4
matches local/remote.84whole accepted language matches vs83, no losses.
basic.at advances671->2437, now KTypePol[KType] unsupported. Both reports
retain FOUNDATION_UNITS_PASS_ORACLE_UNAVAILABLE, not all-language acceptance.

Release build3840096 at atlas-selector-math-release-20260929.Fge03hIX uses
the EXACT500core source. Source archive356a7ecc5595cef1339b5527c51ddbb91028a1eb7701bda99fc2ecec227afa84;
baseline2802d597655cab2a9ed90f103a9974431407bcbfec3ea1ab215fafd76f16e457.
56suite inputs verified; corrected Cartan fixture/timeouts retained. After
completion revalidate full8Cartan cases and wider math, not only the F4unit.
No subsequent K-type implementation is included in that source archive.

Independent corrective review3840071 COMPLETED and local/remote SHA
82cc3115a160223679e73a0052d5dab7d56b2b2c79a1c5af73142444c92a2b36
matches.6Cartan full matches(A2/B2/C2/D4/G2/E6),1F4 ordered-type mismatch,
2actual1200second timeouts(E7FPP and E7Cartan). Originals19.1909s and0.3400s;
Rust timeout is NOT a completed speed ratio or mathematical acceptance.
This review uses3839993release, BEFORE the F4fix; do not conflate candidates.

Discovery3840093 (184cases, unchanged3840019) is now preserved locally,
SHA2faef400cd12df3b85a5e6c39c89ba37c3f241fa742c57f727f37f6f2a6fc147.
Original accepts new raw/memo and stored-history positives; Rust missing names.
Six original signed32 bound errors expose memo WRONG ACCEPTANCE, also at
no-value level. See slices/ktype_formula_2026-09-29.md. Current local runtime
adds candidate raw registration, signed32 validation and RepTableOwner formula
memoization with cutoff truncation,503core expected. No after-pass yet.
Snapshot /tmp/atlas-ktype-before.fRvhKbMr is exact3840085implementation;
transport files /tmp/atlas-abstraction-build.S4ZJBAGw/ktype-*.
Public original/master and Rust/main freshly rechecked unchanged7e1b958c/05625.
No commits/pushes this wave. Prior goal turn yielded new terminal evidence;
this continuation fixes the demonstrated boundary while preserving full scope.

ACTIVE selector-unit build3840085 at
atlas-selector-unit-build-20260929.2sb4BPre; pin
ec16762a4863933c7a5c08c6ab8fe0031b39cd09b3ef232803a3ee4d15a6f6c6.
199local source/200remote input hashes verified;500core expected,180capture.
Current local runtime is frozen at this candidate (includes F4candidate).
UnitAtom is shared across atoms/dot selectors except bare empty tuple;
three tests pin full3840081output, four negative/recoveries and parser shape.
See slices/full_unit_selectors_2026-09-29.md. No after-pass claimed.

Separately3840083 HAS PASSED its unchanged F4numbering session regression
on normal stack (8variants,0ignored); full497core/CLI/capture/final integrity
still pending. After verified completion, create a release candidate and
rerun the full8Cartan cases, not merely the focused assertion. Preserve
corrective F4mismatch3840078 and before-assertion-failure3840081.
E7long array3840070(68/115) remains live at14:46; reviewer3840071pending.
Actual per-engine1200s should be read from final command records. Do not
restart either long case from an observation timeout. No commit/push.

Useful scratch: /tmp/atlas-abstraction-build.S4ZJBAGw contains
cartan-numbering and selector-unit candidate patches/pins/input manifests;
snapshots /tmp/atlas-cartan-before.izhOrwmN and
/tmp/atlas-selector-before.zNl6SSpt retain immediately preceding runtime files.
All durable results/receipts are in tests/reference/hpc; scratch is transport
only. Next collect3840083/3840085 reports and basic.at frontier, then continue
the next original-backed missing operation without dropping math failures.

ACTIVE F4Cartan numbering repair3840083 at
atlas-cartan-numbering-build-20260929.xsUfuGcZ; pin
44dc37d7422b9d71e2e006043701e6b02d824aca0548cbbe48ae0567f5bf471c.
199local changed-source and200remote input hashes verified.497core expected,
180capture cases, normal stack. subsystem_type_value now receives the full
InnerClassContext and sorts by RootNumbering(prefers_coroots); no global
LieType normalization or selector change. Require after-pass and full Cartan
release revalidation. Original3840081 accepts all8focused F4 variants while
UNCHANGED3840019fails the exact real-subsystem assertion. Full selector
positive also original-accepted/Rust-syntax-rejected; negative semantic phases
captured. Discovery SHA9211274d928f3e98aa3df22fe8c75506183c2285fbd6605ca531f7c509828a54
matches remote, retained master original failures keep overall capture invalid.

FINAL named-updateR2 3840019 passes496core(0ignored),CLI/final integrity.
82of177whole language streams match, up from81with no losses; the new
match is named_update_values. Negative stdout/recoveries match, diagnostic
presentation differs. Retained original UB effects mismatch remains visible.
Overall FOUNDATION_UNITS_PASS_ORACLE_UNAVAILABLE retains original failures.
Report SHA8fe4c46edff082088aea63c1b528af22e1755d3f0027fa47512217d0d889d78b
matches local/remote. Latest basic.at loads through402named updates and
now fails at671: (#alphabets).(rec_fun generate...). UnitSelector is limited
to literals instead of current parser.y selector:unit. New180case discovery
adds full-unit selectors positive/rejected and F4Cartan numbering assertion.
Both public Git refs freshly rechecked unchanged: original7e1b958c/main05625.

Corrective preflight3840068 passes59checker tests including actual sbatch
timeout forwarding/conflicts. Cartan F4 case113/job3840078 exposes four
whole-output mismatches: coroot numbering real subsystem reports C2 versus
original B2. Root numbering correctly remains C2. The helper sorts by first
nonzero ambient coordinate, not original RootNbr. Do NOT normalize all C2 to
B2: dynkin.cpp's rank-two classification explicitly preserves basis order.
This is an observable ordered-type difference, not proven nonisomorphic root
systems. Preserve full eight-variant fixture and add focused assertions.

ACTIVE corrective preflight3840068, cpu Cartan array3840069(108-114,600s)
and fat array3840070(68E7FPP/115E7Cartan,1200s,32G), independent review
3840071(afterany both arrays). Stage atlas-cartan-corrective-20260929.hTJtvukv;
56inputs verified, suite SHA37165becc1a3c2bdc1e07a4ab3c1a86502de37d3df30ea9a53b95ded039d77f1.
Uses exact3839993 build report/binaries; no Rust changes in this rerun.
Preflight tests actual sbatch timeout forwarding/default/conflicts plus CLI
1200s bound. Do not claim pending checker or Cartan passes. Per-engine6GiB
and Rayon1 remain; no failure speed ratio. Receipt records all dependencies.

FINAL independent review3840066 verifies all58 release reports and rehashes
606original/1393Rust source files,264upstream scripts and complete raw outputs.
Counts:38MATH_MATCH,9REJECTION_CATEGORY_MATCH,9ORACLE_FAILURE,1RUST_FAILURE,
1RUST_TIMEOUT. Local/remote report SHA
4c422d03b3f38264ecd6c7c0555e0d3ca82d823bde07ebf1142ad09f26cd200c.
E7 FPP actually ran300s, D8 hits4000000enumeration limit, E7KL has original
bad_alloc/Rust allocation abort under6GiB. Eight Cartan cases fail analysis
at int*mat on both sides, not mathematical mismatches. Their failed inputs
remain frozen in3839993stage. Corrective source uses conditional +/-identity.
IMPORTANT: previous submission receipt600/1200 described intent only;
math_suite.sbatch ignored TIMEOUT and defaulted300, Python capped600 too.
Both boundaries now have a local candidate and checker tests, pending HPC.
Fresh corrective stage atlas-cartan-corrective-20260929.hTJtvukv reuses exact
3839993 binaries/build, not the separate named-update candidate.
Named-update3840019 has496core all-pass but final CLI/capture/integrity pending.

FINAL release build3839993 succeeds, archive/source/harness checks retained.
Report math_integrality_release_build_2026_09_29.json SHA
e555c9370070fcb21c68a2a35ee3e1752247b77187f0506cd6e91248dce06bab
matches remote. The58case arrays3839995/3839994 are now RUNNING; do not
claim their outcomes before all stored reports and independent review.
The original public master was freshly rechecked via local git ls-remote
after an HPC-side Git refresh timed out; it is still7e1b958c7aa9456769cc9cf09ac1542814b4800a.
The timeout did not alter any frozen source or justify restarting a job.

ACTIVE named-update R2 build3840019 at
atlas-named-update-r2-20260929.8J60YKTJ, pin
024c864756db8b82278ecb9e0f6518f7ebc1cd1a8cb6685723a722327e86db06.
196local source and198remote input/pin hashes verified;496core expected,
177capture cases. Both positive/rejection session assertions remain unchanged.
R2 retains IDENT identity through scalar/component/field lowering and uses
existing ordinary call name-first resolution; no symbolic diagnostic rewrite.
No after-pass yet. Current local runtime is frozen at this R2 candidate;
math catalog/harness extension116is separately frozen in3839993release stage.
No commits/push; preserve all prior failures and original bounds evidence.

FINAL named-update3839981 compiles and passes its unchanged full positive
unit, but FAILS negative unit at first assertion: unknown_update yields
ErrorKind::Type rather than original undefined-name error. Retain failure;
report math_named_update_build_2026_09_29.json SHA
bcd94876fbc4a832e6c0e29bddd10b40f1c8d6e38a974f5e36ef16a20a7c7a97
matches remote. No full-core/CLI/capture acceptance, not reusable parent.
R2 local candidate carries named-operator identity through scalar/component/
field lowering into an ordinary identifier call, which resolves the name
before argument matching. Symbolic operator behavior stays unchanged.
The positive/rejection session assertions are unchanged. New R2 not yet
submitted; runtime/source differs from frozen3839981 and3839978 release.

ACTIVE release rebuild3839993 at
atlas-integrality-math-release-20260929.c8EEePtM, exact verified3839978
candidate (NOT subsequent named-update3839981). Baseline SHA
94f6b56e8aeb65fb5f2e7b03302e71b7ed95b4d6488692247b887a88ea3fe042;
56file suite manifest SHA
aac8ee2e731917b636ed705c9e8b964e7f8e811e5837169ae94c46c00a2ec246.
Dependent arrays SUBMITTED:3839995(cpu,51cases,%4,600s per engine) and
3839994(fat32G,7cases,%2,1200s). Total58full kernel/shared-helper cases;
all eight new Cartan cases108-115 included. Original108catalog cases stay
intact; unitarity/Hodge/AV/cycles remain open, not accepted by this subset.
Per-engine6GiB cap and Rayon1 remain unchanged, no performance ratio from
failures. Receipt contains all exact indices. After build/arrays terminate,
inspect full results and run a fresh independent math_suite_review on HPC;
rehash source/archive/outputs and compare release source manifest to3839978.

FINAL original diagnostic3839980 confirms the suspected bounds defect:
checked effects/global arms exit134 at std::vector bounds assertion;
GDB global backtrace sees index315 in field_assignment::assign axis.w8526
for a two-component tuple. Unchanged RELEASE isolated global also exits139.
Both local arms complete with identical UPDATE74/LOCAL(74,9) output. Report
math_field_update_diagnosis_2026_09_29.json SHA
aed39ee4413488e1aee23bd8e31d7235ae351bda6ab5bfc8ed5bf36172803e92
matches remote; all source/input/binary checks retained. Do NOT make the old
effects END(7,9) output a Rust golden; it arose from confirmed original UB.
Keep the complete mismatch visible. Source/scoped mathematical semantics
need independent justification, never a silently patched reference binary.

FINAL combined3839978 passes ALL493core tests (0ignored), CLI and final
source integrity on NORMAL stack. Full capture has81of177whole stream
matches, no losses versus3839502; the two new matches are complete
current_root_math_values and integrality_subsystem_boundary. Root-negative
and five wrong-rank cases now reject with correct Runtime messages/recovery,
but stderr presentation differs. Exact B2 before-fail/after-match evidence
is retained. Overall FOUNDATION_UNITS_PASS_ORACLE_UNAVAILABLE keeps three
original execution/classification failures; it is not all-math acceptance.
Report math_analysis_build_2026_09_29.json SHA
e864610c7544259ce07e02afc9b5197efa9998877787284cf283d7e43da1f373
matches remote. This is an eligible frozen parent for release revalidation.

ACTIVE named-update build3839981 at
atlas-named-update-build-20260929.50kIsQy4, pin
8a7effede54b6c54370ec18a626cb21601e8342ba8ac26892c78c05944c12c75,
198remote hashes verified,496core expected/177capture cases. The prior
combined candidate3839978 has passed all493core tests (0ignored), including
normal-stack root and B2 regressions; full capture/final integrity pending.
Separate math-release stageatlas-integrality-math-release-20260929.c8EEePtM
is UNSUBMITTED, with exact3839978 source archive SHA
02644451a91d97c41bd84399f0e8e598cdb360f71290631d8ee949b91a4e668e.
It preserves the prior108math cases and adds eight full Cartan subsystem
cases (indices108-115;116total) to cover the other shared simple_basis
consumer across both root numberings, SC/adjoint and +/- identity inner
classes. New cases are provisional pending original acceptance. Do not
confuse this116case math catalog with177case language capture.

ACTIVE original field diagnostic3839980 at
atlas-field-diagnostic-20260929.i0vWqsiZ, pin
6278dbb1d5ae95f8484de3655b5f367b2838ad3a3cad71791b6738f5ab8bfffc,
201remote hashes verified. Source-identical original checked build plus
unchanged release, three full fixtures and GDB traces; not math acceptance.
Static axis.w fallback passes tuple,selector,pos where constructor requires
tuple,pos,selector. Preserve FIELD74/END(7,9)12 and do not copy suspected UB.
Named grammar candidate adds3tests (496core expected) and preserves the
177case full master/effects mismatch; not yet submitted. Root/math3839978
has passed104session units including unchanged full root test on normal
stack; final full-core/CLI/capture/integrity still pending. Details indexed
in slices/named_update_2026-09-29.md.

ACTIVE combined math/analysis build3839978 at
atlas-math-analysis-build-20260929.GsGMnt5Y, pin
a6af7fcac4167624f474877930240f74074de9cc9e0c26e4ade03faf378c87fd.
All196local changed-source and198remote input/pin hashes matched before
submission.493core expected;177capture cases. convert_expr_context keeps
the original context-floor adjustment and dispatches to12 non-inlined
families; all40 former arm bodies are mechanically preserved. This combines
the pending B2/simple_basis/root-interface repair with the actual analysis
stack repair, not a larger-stack workaround. No after-pass yet. Do not use
the older unsubmitted math-only stage. While the frozen build runs, proceed
with named-update contracts captured by3839528 in a separate next candidate.
Source/report receipt: math_analysis_build_submission_2026_09_29.json.

FINAL diagnostic3839541 COMPLETED0:26 but is NOT a test pass. Exact unchanged
3839527 binary traps SIGSEGV in typed::convert_expr_context's stack probe,
not domain evaluation. Prologue reserves at least0x21000(135168)bytes per
recursive conversion. Backtrace goes through nested let/for/overload/tuple
analysis. With diagnostic-only16MiB, SAME binary reaches unchanged full-output
assertion and fails. call_with_printed also has a large0x9f3d8 frame, but is
NOT the faulting function; don't repair the wrong layer from that observation.
Normal log SHA50f3ce68bb692cc4ccee2275a4b7ca6b5e19d6f9632b33f96298ee9b7414be78;
large log SHA32694dc2c8aaefbe51a70ad81658b9b34880a4594f0adeb767a775c8dc4c5c5a;
batch log SHA019b05eaf766266d9b6aa6e74260222691a71fac187a18eca3eb943e415ff7e0.
All source/report/binary pins unchanged before/after. Metadata saved as
math_root_stack_diagnosis_2026_09_29.json. NEXT: mechanically partition the
large convert_expr_context match into non-inlined families, preserve analysis
contexts and every arm body, then rebuild combined B2/root candidate on NORMAL
stack with all493core/177capture gates. Do not submit the staged math-only
candidate without addressing this known analysis-stack failure. No active
jobs remain; no source commit/push. Named update grammar and stored math
interfaces remain open with accepted original discovery evidence.

ACTIVE exact-binary root-stack diagnostic3839541 at
atlas-root-stack-probe-20260929.5US5VCW2, script hash
3b898646b45c2558de49427a7990c4029a7584c5799ec0ece6d5e8586bc81a41.
Parent3839527 test binary SHA
19580eb34726eaaf16c45ed961c64c2dcb628eccbf92e547eee31f07780a9524.
Read normal-3839541.log, large-3839541.log and root-stack-3839541.log;
normal GDB trace plus diagnostic-only16MiB run, all before/after hashes.
Do not confuse a completed diagnostic job with a passing test. The integrality
repair stage below remains UNSUBMITTED awaiting stack cause/fix. No other
active jobs among3839502/3839518/3839527/3839528. No commit/push.
Indexed diagnosis: slices/integrality_subsystem_2026-09-29.md.

FINAL3839527 FAILS: compilation/type/previous targeted units pass, but
session::tests::current_root_math_values_match_original_classical_and_exceptional
overflows its test-thread stack; no full-core/CLI/capture acceptance. Report
math_current_root_build_2026_09_29.json SHA
09bb65dd31313928b9f5aa7f3e7733bc5503037f375d60740d3064716481f87d
matches remote. This is NOT the simple-basis assertion failure; preserve it
and diagnose the exact binary separately. Do not reuse it for capture replay.

FINAL3839528 retains CAPTURE_FAILED_ORACLE_EXECUTION; report
math_named_update_discovery_2026_09_29.json SHA
06d06b3e93e77a55732e614b44563dd9bc8aac1ddb685f818d950dbaac7c37e8
matches remote. Both named-update positives are accepted in original, syntax
rejected by Rust; negatives reach their intended original semantic phases.
Independent B2 integrality regression PROVES old wrong mathematics:
original rank2/A1.A1/Cartan diag(2,2)/dominantfalse, Rust rank1/A1.T1/[2]/true.
Both retained mathematical assertions fail in old Rust. All five wrong-rank
calls reject originally and are wrongly accepted by Rust, including no-value.

Local candidate repairs shared simple_basis via full reflection-positivity,
integrality RootNbr order/lattice classification and dimension checks; two
new session units retain complete B2 reports and all five rejection recoveries.
493core expected. Staged but NOT SUBMITTED at
atlas-integrality-repair-20260929.AqCnRVZK, pin
6b94f44e0e9110d1f32e12aca14a96fb40a26d67b9a89fd0a99e09beccf2dd8c,
candidate integrality-repair-candidate.patch in the shared /tmp staging folder.
Hold this stage pending exact3839527stack diagnosis; do not submit a known
unrepaired stack failure or enlarge test stacks for acceptance. All root/new
B2 assertions remain unchanged; named-update grammar is still unimplemented.

ACTIVE named-update/integrality discovery3839528 after195remote hashes:
atlas-named-update-discovery-20260929.ielK0IRc, pin
5c0bfb8888dfa3d5b9f1d66e8882b2534b39b605a975c0bc72c12785a4b25955.
177cases/99provisional positives on verified489core iffor3839502; not the
root-interface candidate. Root candidate3839527 remains independently active.
All core implementation files remain frozen at3839527 locally; only catalog,
discovery fixtures/checker and documentation advanced. Shared simple_basis
also feeds FPP and Cartan complex-factor construction; a repair needs those
regressions too, not only the new registration's positive unit. No commit/push.

ACTIVE root-interface build3839527 submitted after193remote hashes matched:
atlas-current-root-build-20260929.JtniKSQU, pin
fb86948a2aa2b3d2652cb1359b1766bd34193591ee37800639c88cc44ec22118.
191changed source/fixture files,491core expected,172capture cases. No after
pass yet. Further static audit finds a likely older mathematical defect:
simple_basis prunes by AMBIENT coordinate comparison, which need not identify
simples in an integral subsystem. Original3839518 B2 [1,2]/2 simples[1,3]
(coroot ordering) are a rank2 subsystem; the old helper may drop the second.
Current local177case next discovery includes an isolated pre-existing
rank/datum/dominance reproducer plus wrong-rank controls and three named
update contracts, using verified iffor3839502. Do not change3839527's submitted
candidate or weaken its newly frozen output test if this deeper error appears.

FINAL iffor3839502:489core ALLpass,0ignored, CLI/final integrity verified.
Report math_iffor_build_2026_09_29.json SHA
f1b08aa08d4d6a5c75059103166d6e7fd697b5d3264044e894ebd743152f36d2
matches remote.79of169whole language streams match; no losses versus3839432.
One shared-case gain is iffor_values; four newly included matches are valid
void consumers, actual generic hidden join, distinct polynomial keys and
current completion inventory. basic.at advances to402:41, n AND:= n-1;
named-function compound update is the next loader frontier. Overall retains
FOUNDATION_UNITS_PASS_ORACLE_UNAVAILABLE; not high-level math acceptance.

FINAL latest-builtin discovery3839518 retains CAPTURE_FAILED_ORACLE_EXECUTION;
local report math_current_builtin_discovery_2026_09_29.json SHA
d2cdb6bbdd04f18c31a2e098e11d7dbbf90a23a0cb2c1f654086b880783da945
matches remote. Both root and stored-cache positives are original-accepted,
Rust REJECTED_NAME. The root negative has five original runtime errors with
all recoveries. Original roots stdout SHA
a19618e000ad9d1870f20a24026395afd167ce87701baaf9224714d44b89e9cd;
235 report lines are frozen in current_root_math_values.expected. Independent
HPC comparison retains the enclosing loop value and complete stdout/stderr.
Current candidate adds W_refl and integrality_simples plus two session units;
491core expected, not yet verified. Stored interfaces/raw K-type remain open.
CompactA1 proves cold four caches none, warmed KL/full/twisted some; Q remains
none even after those operations. Do not implement stored queries by computing
missing results, and do not assume one warm cache populates another.

ACTIVE latest-builtin discovery3839518 submitted after190remote hashes
matched; stageatlas-current-math-discovery-20260929.sGXbVYM1, pin
9d5ad425fefcb01e6dd42cee6cec124679c67164fca843afd7af314d0240d3fa.
172cases/96provisional positives on verified3839432, independent of the
pending iffor build. Adds current_root_math_values/rejected and
current_stored_math_values. Original source validates wrong rank and W_refl
index narrowing before its no-value gate. Preserve signed root numbering
and exact simple-root order; do not infer output order from Rust RootId.

ACTIVE iffor3839502 atatlas-iffor-build-20260929.jAjObjyv, pin
d2873cb4bb8227b431bbcf92ebd9b0e61bb61d27a01cea5a14d30473874a8f61;
189remote hashes verified before submission.489core expected,169language
cases. Quiet-if/iffor uses hidden "## " (trailing space), preserving visible
generic override independence. Six new units also freeze exact309+3completion
names and actual distinct two-term ParamPol/KTypePol iteration. No final gate
yet; do not reuse until terminal core/CLI/final integrity evidence is inspected.

FINAL discoveries3839480/3839492 retain CAPTURE_FAILED_ORACLE_EXECUTION.
Reports math_next_boundary_discovery_2026_09_29.json SHA
c9d15e5960ade92e5d56dc20eb8cdd61d08f5fd63527e083dbef3dace77399a3
and math_hidden_completion_discovery_2026_09_29.json SHA
1256f0f88b25422794e470f49774148bed17231c44f9d4a5d9a0555479cd1b22
match remote. Valid void consumers and distinct compactA1 two-term fixtures
match full streams using3839432. Dynamic discarded-vector containers still
wrongly evaluate; let tuple patterns still wrongly accept. The actual generic
visible-join override is original-accepted, unlike the retained concrete probe.
Latest completion names expose seven unregistered mathematical interfaces
plus query/system; see AGENTS. New172case local discovery adds root-reflection/
integrality positive+negative and compactA1 stored-cache/raw-K-type probes;
these are provisional, not new mathematical acceptance. Runtime is unchanged
from frozen iffor3839502; no source commit/push.

NEXT discovery3839480 SUBMITTED after186remote hashes matched, at
/public/home/majj/atlas-next-boundary-discovery-20260929.ARh81VNj,
pin2493078553400fbaedc8e10d26af3d0396bbcd40185ac3dabcd7cd7ba7051737.
168language cases/93provisional positives on verified void/for3839432.
All176pinned candidate source files still match the local runtime; only
discovery fixtures/catalog/harness and docs advanced. Next actions: inspect
whole valid consumer outputs, dynamic no-value and historical let errors;
implement quiet-if/iffor using original3839396 contracts and hidden join
source semantics, keeping the failed visible concrete-override probe. No
new source commit/push. High-level math and parallel performance scope stay
unchanged. Do not rerun or modify the submitted stage merely for new inputs.

LATEST void/for3839432 TERMINAL FAILED11:17 with explicit
FOUNDATION_UNITS_PASS_ORACLE_UNAVAILABLE (retained original signal).
ALL483core tests pass,0ignored, CLI and FINAL source integrity pass. The
unchanged prior VOID6 failure plus all8new void/for tests pass.159language
cases have74whole stdout+stderr matches: no losses versus byte3839335,
one gain among its145cases (for_reversal_values), eight newly added matches.
Latest unmodified basic.at now fails at378:15 IF in a filtered for loop,
not at183. Eligible parent for further captures, NOT high-level mathematics
acceptance. Report math_void_boundary_build_2026_09_29.json SHA
4d15de87847a312c5283398660508e4c59d6c30fa0ee1bc179c3b07008756790
verified against remote. Source is still frozen locally; no commit/push.

Discovery3839452 TERMINAL FAILED1:05, retained original execution failures.
Report math_void_level_discovery_2026_09_29.json SHA
402a498966e1fb273c166d30434fe6dce193f8719bab362030d6e847a61619ce
verified against remote. Original ACCEPTS void_consumer_values_valid, old
byte3839335 also accepts but returns different values. let_pattern_void_rejected
has two original program errors, both wrongly accepted by old Rust. Preserve
its positive VALID1 and both recoveries. The vector literal no-value probe
is REJECTED_RUNTIME on six constant expressions, but the nonconstant EFFECTS
line succeeds with1 in original and fails in Rust. global.w5148 marks the
operator as folding: constant-folding validation is separate from runtime
no-value. Do not remove either existing integer or vector-literal rejection.
New dynamic-vector and exact historical let-unit companions are staged with
generic hidden-join replacement and distinct compact-A1 polynomial keys.
Local inventory168/93provisional positives; pending stage below uses verified
3839432, not old byte3839335. Follow up on full consumer outputs before claims.

Void build3839432: all3new void units and all5for units PASS, including the
unchanged former VOID6 assertion. Full report is still pending; not eligible
for reuse until core/CLI/final hashes finish. Discovery3839442 TERMINAL
FAILED1:02; report SHA44873d8e1bd908f1053b81f20320d6671e24956bbb8d4f042b2e3acfb943df55
verified locally and remotely. Consumer fixture reveals a pre-existing wrong
acceptance of nested-void let tuple patterns. Integer divisions validate even
when discarded. Both are recorded in slices/void_boundaries_2026-09-29.md.
NEXT discovery3839452 SUBMITTED with164cases/90provisional positives,
182remote hashes verified, atatlas-void-level-discovery-20260929.X5UB5puv.
Pin48770eb6b40a80375fed49f080eaa6e8ce814bbee0bafbfd2aa8452a07633665.
Three companions isolate accepted consumers, vector no-value behavior and
let-pattern rejection. Runtime still matches frozen3839432; no further edit.

ACTIVE discovery3839442 at
/public/home/majj/atlas-void-consumer-discovery-20260929.gvH11fZf,
pin4af14ac3dea67941cf70ee64179895fc0b5f3d2db8253ae9f6e7ac7ef40d205b.
All179remote hashes matched;161cases/88provisional positives on unchanged
verified3839335. Consumer and discarded-integer-division probes await original
results. Integers may still validate division at no-value; preserve rejections.

ACTIVE void-boundary candidate3839432 submitted at
/public/home/majj/atlas-void-boundary-build-20260929.YOetNKte with pin
07b74184f20118c7281a559e3abc1b37ecf2435e67d6bc924bd5a195bda6a242.
All178remote hashes verified.483core expected,159frozen language cases;
three new captured void units plus unchanged failed for-context assertion.
No after-pass yet. Local next-discovery inventory161/88adds consumer and
no-value evaluation fixtures; keep the submitted159stage immutable.
Source audit and explicit-consumer differences are indexed in
slices/void_boundaries_2026-09-29.md. All testing remains on HPC.

LATEST TERMINAL discovery3839396 FAILED0:31 with retained
CAPTURE_FAILED_ORACLE_EXECUTION. Original accepts BOTH void positives and
iffor_values with empty stderr. Local report math_iffor_void_discovery_2026_09_29.json
SHA2b98bfdf82a9e796908e945c9a1779192eca8b3fbe434f0eb48f7f0352560b9e,
matching remote. Original outputs SCALAR7, ROW[1,2], TUPLE(7,"x"),
CONDITIONAL7, SEQUENCE11, INFERRED_TUPLE(7,8), EXPLICIT_TUPLE((),8),
INFERRED_ROW[()], EXPLICIT_ROW[(),()], GLOBAL42, CALL6. Loop scope outputs
CAST_SEQUENCE6, CAST_LOOP6, COUNTED_SEQUENCE3, COUNTED_LOOP3 and both
body-row controls[(),()]. This proves eager coerce-to-void is too aggressive;
explicit consumers still must discard. Prior3839328 Rust already printed
VOID(), so the new for unit exposes a pre-existing defect, not a regression
introduced by this for patch. NEXT: add exact new units, implement explicit
discard boundaries per source audit, then rerun unchanged480+ core/CLI/full
streams on a fresh stage. No active jobs remain among3839386/3839388/3839396.

iffor_hidden_join was NOT original-accepted: visible concrete ##([[int]])
overload is forbidden as too close to existing [vec]. The original prints a
bare SET rejection envelope not yet classified (OTHER_FAILURE). Retain it;
prepare a valid different companion before claiming hidden-join isolation.
iffor_rejected reaches intended type/name checks; old Rust fails syntax first.
All166pinned candidate source files still match3839386 locally; only harness/
discovery inventory advanced to159. No source commit/push or math acceptance.

NEXT discovery3839396 SUBMITTED atatlas-iffor-discovery-20260929.GXMZk31E
after177remote hashes matched.159cases/86provisional positives, unchanged
verified byte3839335. Preserve for3839386's4pass/1fail unit result; the void
boundary source audit and required consumer checks are indexed in
slices/void_boundaries_2026-09-29.md. Do not reuse failed3839386 as a parent.

LATEST: for build3839386 TERMINAL FAILED5:07. Compilation,46type units and
coercions pass; the five new session units give4pass/1fail. Nonrow bytes,
polynomial keys, reversals and negatives pass, but the UNCHANGED context
fixture prints VOID() instead of original VOID6. Full core/CLI/capture were
not reached; do not reuse this build as an eligible capture parent. Local
report math_for_iteration_build_2026_09_29.json SHA
b20555e2144e23cae059b7f9c248bd9456c7616a416f54fd27cd5bed931adba9.
Source remains frozen at this failed candidate. Suspect eager structural-void
wrapping in conform_types; casts greedily contain the following sequence.
Audit actual discard boundaries before any global change; preserve the failure.

Detail3839388 TERMINAL FAILED1:15, CAPTURE_FAILED_ORACLE_EXECUTION. Report
math_for_detail_discovery_2026_09_29.json SHA
913d1c2b42d27d6f507f97cce52474cf3f44615e16d4dee81a0f7d1ad1c8ca3f.
Both report hashes match remote. New original positives all accept: signed64
boundary values, named-void loops and polynomial arithmetic. The supposed
multi-term A1 example COLLAPSES to one canonical term (x=2,lambda=[1]/1), so
it verifies coefficient addition/cancellation but NOT multi-term ordering.
Named outputs: NAMED_LOOP[0,1,2], NAMED_BODY[(),()], VOID_BRANCH3.
Count narrowing emits three bare "Integer value too big for conversion" plus
"Evaluation aborted." pairs; current classifier marks OTHER_FAILURE. The
fourth alleged overflow case emits NO diagnostic. Preserve it and isolate;
do not claim the oracle enforces that boundary from source prose alone.
Trace fixture rejects as runtime but does not print back_trace itself;
add a companion if validating actual trace headings/frame dumps.

NEXT LOCAL master159/86positives includes three iffor probes plus two void
boundary probes. Fresh stageatlas-iffor-discovery-20260929.GXMZk31E uses
verified byte3839335, not failed3839386. Pin
799ed7586428a3e80e501ab95745af71b7be68ed975f10d204d12633e1486ac5.
Earlier157/154/149 inventories refer to distinct frozen stages below.

LOCAL NEXT inventory157/84provisional positives adds three iffor probes;
not submitted yet. Runtime still matches frozen3839386. Do not accidentally
replace its149case or discovery3839388's154case inventories in place. Hidden
join isolation and exact parser.y references are in the indexed for slice.

FOR DETAIL discovery3839388 SUBMITTED at
/public/home/majj/atlas-for-detail-discovery-20260929.XcZTYXRY,
pinf3ce7fbae12ad13e0a5292fa505187aec096d446182cb1574b5c2fd85108b304.
Reuses verified3839335 (475core), NOT active3839386. Frozen154language cases/
82provisional positives. Five additions isolate signed64 count/bound narrowing
and overflow, accepted boundaries, named-void contexts, multi-term A1 polynomial
ordering/cancellation and runtime trace categories. All172remote hashes checked.
Inspect original acceptance before implementing range/message changes.

FOR BUILD3839386 SUBMITTED atatlas-for-build-20260929.wJ3UYyT2 after all168
remote input/pin hashes matched.480core expected, frozen149language master.
Continue independent original probes while this job runs; do not restart it.

LATEST TERMINAL byteR2 3839335 FAILED9:39 with explicit
FOUNDATION_UNITS_PASS_ORACLE_UNAVAILABLE (retained original signal). All475
core tests, CLI and final source integrity pass;65of145 complete language
streams match, no losses against return-stack3839234. Six gains: string slices,
numeric/raw byte values, nonrow slices, byte subscription and valid transport.
Local report math_byte_string_build_r2_2026_09_29.json SHA
38f64e930ef1954e6b686651cbc3c24b0e1bb956526b76e1bbb1eb10cc8e4e50,
verified against the remote original. basic.at now reaches line183, failing
the reverse counted loop in last_true. This is not added high-level math
acceptance. Previous pending/running notes below are historical.

CURRENT LOCAL source now adds general for-loop implementation after3839328:
seven receiver kinds, actual polynomial keys, two reversal flags, body-context
analysis before execution, no-value iteration and independent closure frames.
Five new whole-fixture units target480core total. Fresh candidate is staging
at atlas-for-build-20260929.wJ3UYyT2, pin
3fb5bd7dd715cdf6db70a90605a16d3dc96aa84819541f274c726dd4f6cb7e80,
patch ee89d544418ada99c522275135c5ad5e33838c22ddc53d55e4466fe60cab9a31.
No compile/pass claim yet. General quiet-if/iffor, counted signed64-bit
boundaries, richer polynomial order/cancellation and named-void loop contexts
remain separate explicit discovery targets. No source commit/push.

DISCOVERY3839328 FINAL FAILED1:13, CAPTURE_FAILED_ORACLE_EXECUTION from the
retained original signal. All3new for positives are accepted by original;
unchanged Rust rejects nonrow/ParamPol/KTypePol receivers and ~DO/~OD.
The negative reaches original vector/string binding constness; Rust fails
receiver typing first. CurrentASCII companion confirms original uses the
message "Integer value too big for conversion"; historical Rust "to big"
wording remains an OPEN migration, not repaired in byteR2.
At the last source audit all162byteR2 pinned source files still match locally;
the local149case catalog differs intentionally from its frozen145capture gate.

LATEST ACTIVE byteR2 job3839335 at
/public/home/majj/atlas-byte-string-r2-20260929.vBfroBpv,
pin1eb62274c12c1a8a3463f2999a8b2d7e48a1cbf87b937e132956345060066bad,
patcha374dde2b3452f2a61c27d984e2ce9a16b18e17dae4ad88b4a22835d98424a2a.
Only difference fromR1: two historical session test value-extractors now
handle ReportBytes/OutputBytes as non-values. Unchanged475tests expected,
normal stack and145frozen language cases. No pass yet. Current session hash
997bb9f0263676f3bcf38bc557d82672caca3e63734fea679f622721033621a1.
R1 job3839304 TERMINAL FAILED3:48, E0004 at session.rs1323/1468; no unit ran.
Local report math_byte_string_build_2026_09_29.json SHA
651e9433df2804033c51ae25c66af804e8fa417d2e17aa6a46de1fb666651f7b.
Do not weaken assertions or reuse/alter its submitted stage.

NEXT discovery3839328 submitted at
/public/home/majj/atlas-for-iteration-discovery-20260929.VtDqCWbP,
pinfadd53e4d54221fe06166c0c4d4dc021bf8bbfac1b85ec1abe507d1dbaf81242.
Reuses unchanged verified3839234, not either unverified byte build.149cases/
79provisional positives, adding nonrow and ParamPol/KTypePol iteration,
context/capture/reversal, negatives, currentASCII and corrected## companions.
Local source audit indexed in slices/for_iteration_2026-09-29.md. No loop
implementation yet, and this language inventory is not the108math catalog.

ACTIVE BYTE BUILD3839304 at
/public/home/majj/atlas-byte-string-build-20260929.szzp9hwY, pin
8156d8b17515866d18391818bbf5ffe4631f20b7d5702a65d9e7c0461211c401,
patch362682dceed2b524f8e9aaf78539f2c0420e493e00c983d12b85a10d3b3663e7.
Expected475core/145language cases, no skips. Actual checks pending. Local
runtime was frozen at submission; do not modify its stage or restart live job.
New APIs preserve raw bytes through Value, prints/to_string/showall, events,
stdout/redirect, diagnostics and back_trace. No unsafe, no source-input claim.
Same candidate migrates uppercaseASCII after3839257 verifies all seven exact
lowercase old unit sources reject; keeps old negative plus current companion.

DISCOVERY3839257 TERMINAL FAILED1:05, CAPTURE_FAILED_ORACLE_EXECUTION (old
original signal retained). Local report math_byte_transport_discovery_2026_09_29.json
SHAb13505010236b82e1384629ed279e8d0d36a0b4d334282d245f3867f52ea6950.
New raw domain/error stderr confirmed by hex: c3 in Lie_type/inner-class
messages, a9 in BYTE_ERROR. Original rejects string '+' in transport fixture;
the unmodified failed fixture stays, new transport_valid uses correct##.
These latest notes supersede older submitted/running notes below.

AUTHORITATIVE UPDATE2026-09-29: stack3839234 is terminal FAILED12:36 with
FOUNDATION_UNITS_PASS_ORACLE_UNAVAILABLE (retained original signal, NOT a
new Rust unit failure). All465core/CLI/final integrity pass;135language cases
give59whole stdout+stderr matches, no losses against cast3839029's55.
Gains: return_loop_values, return_inference_values, return_structural_values,
slice_reverse_asymmetric. Report math_return_stack_build_2026_09_29.json SHA
72cf45b4efe5ec3e523f18f3d17294c03c3fb9693f8f8635236633cae7ef3026.
This is not whole-language or high-level math acceptance. basic.at still
fails string slicing at86:10. Older RUNNING/SUBMITTED notes below are history.

Byte subscription3839240 is terminal FAILED0:46 from the same retained
original signal. Its isolated new fixture is ACCEPTED by BOTH with empty
stderr: original INDEX_LENGTHS1111, Rust INDEX_LENGTHS3333. Report
math_byte_index_discovery_2026_09_29.json SHA
912ff01256e57155c4e0e3e242772909306dec48b9fa9425c70dfa0264363f39.

CURRENT LOCAL candidate now diverges from3839234: safe AtlasString(Vec<u8>),
byte-preserving container/value/prints/to_string/showall output, report events,
CLI and redirect sink; exact runtime-message and frame back-trace bytes;
five slice receiver kinds, reverse coordinates and long_val evaluation order.
All edits are UNVERIFIED until the next compute-node build. The source/file
input decoder remains an explicit separate UTF-8 boundary, not solved here.
Discovery3839257 is SUBMITTED at
/public/home/majj/atlas-byte-transport-discovery-20260929.6WSqUaCC, pin
8111f351371d8a23d4a0f223e10f3353c9c935c0f4493666e7eb7806506d6c72.
It reuses unchanged verified3839234,143cases/75provisional positives; exact
legacy ascii unit sources must be captured before changing those assertions.
No source commit/push, no new mathematical or speed acceptance.

NEXT DISCOVERY3839240 is SUBMITTED at
/public/home/majj/atlas-byte-index-discovery-20260929.GmtPAhvs, pin
1bff4c8a9fc818147980824a979f9690b154b48bd9de01e6fe9b326b2f4b4907.
140cases/74provisional positives, unchanged cast3839029 binary. It isolates
byte-index length from missing uppercaseASCII and slice failures; inspect
generic_string_byte_subscription whole stdout before asserting acceptance.
Current runtime hashes match stack3839234 pin (typed3489d0a2, syntaxec763f3b).
No runtime/source commit or push. Submit/collect, do not restart live jobs.

LATEST3839234 atRUNNING10:11: ALL465core pass (0failed/ignored/filtered),
including unchanged return/stack/name/slice assertions. CLI is now compiling;
complete135case capture/final integrity still pending, no completed report yet.
Do not mark full language/math acceptance from this unit gate.
Current local140case discovery adds an isolated byte-index wrong-value fixture;
runtime remains exactly the frozen3839234 candidate, no byte repair implemented.

Byte3839236 inspected: allTHREEnew positives are original-accepted/empty
stderr; bounds negative is REJECTED_RUNTIME with all recoveries. Raw original
stdout contains isolatedc3/a9 and reverseda9c3 bytes, including row/value output.
Numeric BYTE_INDEX195169/BYTE_SPLIT19516911 and exact vec/ratvec/matrix slices
are captured. Rust rejects slices and uppercaseASCII; actual registry still
uses historical lowercaseascii (do not rely on earlier prose assuming mapped
uppercase registration). Indexed source/expectations: slices/string_bytes_2026-09-29.md.
Original master signal remains invalid, not a type rejection or math success.

Update3839234: all46type tests and allSIXreturn session regressions pass,
including the unchanged structural-positive previously aborting on the normal
stack. Full-core/CLI/full capture/final source integrity remain pending. This
is after-pass at the exact assertion, not full language/math acceptance.
Byte capture3839236 is terminal FAILED0:57 withCAPTURE_FAILED_ORACLE_EXECUTION;
report80ac71121e4a34bb9fabaf3555a82004bbdbd95d79fa15c7c9beaa079e4e5df7,
new fixture outputs still being inspected. Captured case folders have the
generic_ prefix; use paths from each report rather than bare catalog IDs.

Update: precedence/name-only3839232 is TERMINAL FAILED4:11. All46type
tests, the new parser/name units, slice unit and structural-return negative
pass; structural-positive still stack-overflows on the normal stack. This
isolates the remaining failure from precedence/name fixes without weakening
the test. Report245cd09c367b6985b39848492cc8c7017b06c4158c7166dbfaa087e06039b19a.
Combined3839234 remains active; current local runtime matches it. Byte-string
capture3839236 is SUBMITTED atLgv9WCvk, pin
4a9837971136a33f72823e56cdebe01a13ea4054a8ae4fc85c675bd2c38b824c,
139cases against unchanged verified cast3839029. Do not conflate its source
or inventory with the135case build. All original signal cases stay retained.

ACTIVE2026-09-29: precedence/name-only3839232 is SUBMITTED at22WEAC7O;
combined precedence/name/stack3839234 is SUBMITTED atwPFZUJAy. Both465core
expected, frozen135case master; only the latter matches the current local
runtime. Do not restart either. Stack candidate pin
9bad823ba28311f9e5ac59f99ccc1dccb85dc192e6c326b3a3da6cf9b5cfdcc0,
patch51e4f25636d9ba8ab277715f3926b75ff0f5b18278aee3c1b6ebac1880abe99d.
No test-stack increase:38existing evaluator arm bodies are repartitioned into
sixnon-inlined families. GDB3839222 proves the old evaluate frame is97320bytes;
with16MiB diagnostic stack the same binary reaches RECURSIVE120 then fails
the preserved parser assertions (LET0/ABSTRACT61 etc). Evidence and binary/log
hashes are in math_return_stack_diagnosis_2026_09_29.json.
Transfer failure3839224 is retained (candidate.patch truncated, guard rejects
before compilation); replacement stages assert remote hashes before sbatch.
The NEXT local discovery corpus is139cases/73provisional positives, adding
byte-string numeric/raw-output probes, non-row slice receivers and bounds.
These are not in either frozen135case build and have no expected-value claim
until the original capture is inspected. No commit/push.

LATEST TERMINAL EVIDENCE (2026-09-29): return3839117 FAILED12:58 with
FOUNDATION_UNITS_PASS_ORACLE_UNAVAILABLE. All459core, CLI and final integrity
pass, but54whole language matches versus cast's55: gains both return positives
and loses THREE named-function declaration streams. Their numeric values still
agree. Report669e82299f947f592d405cdd4b8f068fb9d99ca5dfcd31698a9679c8998cec40.
Return+slice3839163 FAILED5:05: asymmetric slice assertion passes, structural
negative sees only2of3required errors (wrong_explicit installed), and positive
structural-return test stack-overflows. No full-core/CLI/final integrity pass.
Reportfb911463e3cdbd6be439df0bd488a9b289f5c2e5f86d90d8462b7751610ffc03
is retained locally as math_return_slice_build_2026_09_29.json.

Current LOCAL UNVERIFIED candidate fixes RETURN precedence (RETURN tertiary,
not RETURN expr) and non-destructive named-function component matching. Three
new units supplement, not replace, all old assertions (465core expected).
Frozen patch19defa1b6a97dbdbb518d42c78191cfebdbdfc85e400032c958dac65f6acad36,
pin8e87bd73f33925f9250f558e19e83a788d0729c5505fb08cf73ba915177bf529,
stage atlas-return-precedence-build-20260929.jyZQVOPd is being staged.
Stack diagnostic3839217 on the exact3839163test binary traps in
TypedExpr::evaluate; fresh detail3839222 probes frame size and a larger stack
ONLY for diagnosis, not acceptance. Stage atlas-return-stack-probe-20260929.ipvCxcub.
No commit/push; string slicing still unported, no added high-level math acceptance.

Earlier chronological checkpoints follow; terminal evidence above supersedes
their RUNNING/SUBMITTED assertions.

LATEST LOCAL CANDIDATE adds two original3839128 structural-return session
units and one original3839145 asymmetric slice unit (462core expected).
Only additional runtime change after return3839117 is evaluate_slice's reverse
coordinate mapping n-upper..n-lower, plus clarified SliceFlags documentation.
All historical units stay unchanged. Patch
b7649187da5d46ebd6f576d56687ace3f1a4009d3704d2d72dc914da01a8747d,
pin e7c703cfc0cdf05fb3d81a08407bbc2ced27056ef444719b293e0c200b9f53df,
fresh stage atlas-return-slice-build-20260929.6vau3Quw. Full135case master;
string slicing is still UNIMPLEMENTED. No commit/push.

Return3839117 now passes ALL459core (0failed/ignored/filtered), with CLI
capture/final integrity still pending at RUNNING9:34. Slice discovery3839145
is terminal FAILED0:45 only for retained original signal; report
dfc23bd82c7776f00ee27190bcc7a7656a3b1dc84b8975b9f998917ede6012da.
Its new positive is accepted by both engines, but a~[0:2] is original[4,3]
versus Rust[1,0]; from-end/open intervals also differ. Complete outputs are
preserved. Upstream HEAD was rechecked via HPC GitHub and remains7e1b958c.

Return3839117 compiles,45type tests and ALL FOUR new return regressions pass;
full core/CLI/capture/final integrity are still pending. Do not confuse that
partial gate with final acceptance.
Structural discovery3839128 is terminal FAILED0:39, report
e190fdba51bb1e60ff295e38fe0427d35c44ab09a8f412f73ce36bcde7f23db6.
Original accepts the complete structural fixture (PAIR tuples, ROW[2,3][5],
TUPLE(31,37), LIST[41], LET43, GUARD47, FUNCTION53, ABSTRACT59, RECURSIVE120).
Pre-return Rust panics in the final recursion on an integer builtin receiving
void. Original rejects all three structural negatives; old Rust accepts all.
These cases must be replayed against the completed return candidate too.
String-slice original positive fully accepts FULLabcdef/LEFTabc/RIGHTcdef/
MIDDLEbcd/EMPTY!/PREFIXtruefalse. Bounds need integer analysis before receiver
slice-kind checking (axis.w:4645); original negative categories include PROGRAM
as well as TYPE. String slices are byte-based upstream, not Unicode characters.
Discovery3839145 at atlas-slice-direction-20260929.iFTSRlSp, pin
903e35cd1679586c2d214038ee41444beb2dd2e97f488202f5d84ca2a620da5b,
is submitted with135cases and the unchanged verified cast runtime.
The corpus adds an asymmetric reverse-row slice probe: slicing reverse
iterators is not reversing a forward-selected interval. Capture before repair;
the historical len4[1:3] test is symmetric and cannot distinguish these rules.

ACTIVE RETURN REPAIR3839117 (2026-09-29) at
/public/home/majj/atlas-return-build-20260929.BjBKYbKn, pin
f292f9768c4d9b1a4aee7b4a301ec7f1547e31120043e6c8304e9dd7d7e85f17,
patch bda189236b7030898a24305f84ce4888d0b9c6a919f31d032ccb78bddbdf4c0d.
Current local runtime is now this UNVERIFIED candidate: safe live ConversionType
requirements replace copied function-result contexts. Return operands share
the nearest closure result; casts publish constraints before bodies; balancing
copies the live target per branch, not once before all branches. Nested closure
scopes remain distinct, and no mutable RefCell guard crosses recursion.
Four new session units (459core expected) use exact3839034/3839086 fixtures;
130case frozen master plus full-core/CLI/final integrity required. No commit.

Inference3839086 is terminal FAILED0:28 only for the retained original signal;
report c2be4176e08f15714be780a88d23c1f4b72143fcd243dba81e9e94b52b573cd7.
Original accepts INFER79/RAT3/21/1/DECLARED1/1/NEXT1/1/NESTED17/RECOVER233.
It rejects ALL THREE incompatible-result functions, while pre-repair Rust
accepts all three with no diagnostics. Preserve the complete negative too.
Structural discovery3839128 is SUBMITTED at
/public/home/majj/atlas-return-structure-20260929.fATGPfbW, pin
746f2ce2434b3880d64c0bc630c2ceb8ca923c9d2174dea4ded81a933b9d192d.
Catalog134/69positive adds structural returns and ASCII string-slice loader
prerequisites; it is not the frozen130case repair job. Its unchanged runtime
is cast3839029, not the active unverified return implementation.

LATEST 2026-09-29: operator-cast3839029 is terminal FAILED9:54 with explicit
FOUNDATION_UNITS_PASS_ORACLE_UNAVAILABLE. All455core pass, CLI and final
source/input integrity pass;55whole-stream language matches. Report
2836a184c412027bdd4daaa98ca21557294c188144a4c461744323d7cdd949a7.
Both unchanged positive operator-pattern failures now pass. Latest basic.at
failure is STRING SLICING at86:10, with file abandoned at92, not the old
operator-binding syntax error. No high-level mathematical acceptance.

PRIORITY WRONG RESULTS: discovery3839034 (report
b0f8946d3c5026d1f3156218c07a22dee5133293fe0120f98c7f9faf002000bf)
accepts return_loop_values in BOTH engines with empty stderr, but original
BOOLfalsetrue/INDEX1-1/WHILE23 become Rust BOOL()true/INDEX()-1/WHILE().
return_loop_rejected also wrongly installs the first two functions in Rust;
the third error does not make the whole fixture a matching rejection.
Preserve both regressions. axis.w:743-774 shares the function-result type
with every return operand, independent of discarded loop/branch context.
Returns do not balance: inference order matters. Casts specialise the shared
context BEFORE converting their bodies (axis.w:7290-7332). A root-cast-only
patch or post-hoc result join is insufficient. No return fix exists yet.

Return-inference discovery3839086 is SUBMITTED at
/public/home/majj/atlas-return-inference-20260928.DIALhq43, pin
de5520058e1e81d02843a7aeb8831a00586480b7d8804fb79709f3318dbb1d69.
130language cases/67provisional positives, verified break3838716 parent.
Adds inferred/rat/next/nested-return positives and conflicting-return negatives;
keep expectations provisional until full original output is inspected.
The for_reversal_values fixture in3839034 is original-accepted but Rust
syntax-rejected; no reversal implementation has changed. Earlier SUBMITTED
notes below are historical, not current handles. Current local runtime is
cast3839029; no new commit/push.

OPERATOR PATTERN3838965 FAILED3:22 AFTER compilation/45type checks: both
nonfunction negatives pass, both positive fixtures fail at `@`. Report
ae2aa6c93aebe4a88be778871b2df442abdeca7813abfaa2c88d497e278b0561.
Do not weaken their assertions or replace selected functions with easier
unselected names. Concrete operator casts are absent as well as explicit
generic casts. Original3838987 confirms the full new positive (EXACT323/4,
APPEND integer/string rows, GLOBAL3,FIXED7seven,FALLBACK3/4,RECOVER167) and
all semantic/scope rejection recoveries;125language cases,64provisional positives.
Report0a52468d2df4fcb028ba150e8029f33a34455483ef1c85df2c1165d4ae906838
retains headerless cast negatives as OTHER_FAILURE plus old original signal.
New classifier controls cover the exact cast rejection envelope, pending HPC.

CURRENT local runtime additionally implements OperatorCast AST/grammar and
exact-first global selection with scoped generic fallback, whole-signature
substitution and captured-function traces; adds3session tests (455core expected).
Scratch operator-cast-candidate.patch, operator-cast-build-pin.json freeze it:
patch7386f48431f1859fae32d574ffe30f59805b56312dac43d3a338d1e25baaee61,
pin564e96dfca228b90c49cca91ad355c53b338f9c2f7511479bb3cc4263408c104.
Fresh stage /public/home/majj/atlas-operator-cast-build-20260928.fOeLIepD is
SUBMITTED as3839029; latest fully verified runtime remains break3838716/448core.
No commit; no new high-level mathematical acceptance. Earlier notes below
are chronological evidence, not the latest state when they say SUBMITTED.

CURRENT UNVERIFIED RUNTIME: operator pattern candidate3838965 is SUBMITTED at
/public/home/majj/atlas-operator-pattern-build-20260928.zvPKvFXp, pin
1103b209bffad4d7524b7c2701675b50ec3b55745994b15106e56c657e63521a,
patch0806f4cfde5d3b98ecf6954556911b298cdcd6496cde20fa7d6fb2c1d343909d.
Four new original-backed session regressions (452core expected),122case master;
operator flag on Pattern::Name, shared global/local nonfunction checks,
lexical operator dispatch using the existing direct function solver. No commit.
Latest VERIFIED runtime remains break3838716 below.
Discovery3838953 FAILED0:25 only for retained original captured-selection signal;
report777b1ffd4e1159195c68dabfeb2d57ccd5dcdeccb98e75e09a3411dc52f7e447.
Accepted bang scopes print LOCAL12,TUPLE(15,5),PARAM16,LOOP[18],GLOBAL199,
RECOVER137. Four local nonfunction errors and one global error retain ATOMIC19
and all five recovery values. Failed tilde-comma sources3838933 are preserved;
right parenthesis itself is NOT a special tilde lookahead in lexer.w:802+.

LATEST VERIFIED RUNTIME: break3838716 finished FAILED8:51 with explicit
FOUNDATION_UNITS_PASS_ORACLE_UNAVAILABLE. All448core pass, CLI and final
source/input integrity pass;50whole-stream language matches, no losses from
mode R2. Report0876bd107000158f7635080818f8353491032248bdbdbdc8177aa027bf965fa1.
basic.at take now analyses; loading advances to line91 bare operator binding
set ! = prints_lines@[T], rejected syntactically. High-level math unchanged.
Operator discovery3838739 preserves118cases, original acceptance of bare
symbol/function bindings and static nonfunction rejection. Its latter envelope
has no Type/Program header and was OTHER_FAILURE, alongside the retained
original signal. Report38539a7596b7506ad514cd2e7073dcb2edb50cfacb1f32fb8527258a7e111b00.
New120case discovery adds all-scope symbol patterns and atomic rejection;
original-backed classifier fix has positive/negative controls. Runtime is
still unchanged at the verified break candidate when this note is written.

Earlier checkpoint notes follow; their SUBMITTED/UNVERIFIED states are superseded
only by the explicit terminal evidence above.

CURRENT BREAK BUILD3838716 is SUBMITTED at
/public/home/majj/atlas-break-build-20260928.9r9ZO6Ju, pin
78e3dd1cfad548b20d2702b5fcbde6101ca70095d41a2198c10873f250207746,
patch05af528f30b4b66fbb213bc35b5f6558e08944c46f1b860a434e5248949b0e09.
Current local runtime is this UNVERIFIED candidate: break leaves its type
unconstrained, grammar repeats BREAK and rejects numeric syntax, depth
diagnostic repeats keywords. Raw compact AST remains numeric like original.
Two session regressions plus historical parser/depth assertions migrated
after exact source captures; old numeric fixture/goldens remain untouched.
448core expected, frozen116case capture. Scratch break-candidate.patch and
break-pin.json include all16new generic fixture add-file hunks exactly once.

MODE R2 VERIFIED:3838648 terminal FAILED11:17 with
FOUNDATION_UNITS_PASS_ORACLE_UNAVAILABLE;446core all-pass, CLI and final
integrity. Report65935ce0523e1e34f1c49a3527226267a03e4f716a973a9c1e5fff8f1c27834d.
While context positive matches complete streams;48full matches/no losses.
Negative bodies match stdout but not stderr envelope. basic.at count error
is gone; take[void]/[A] remains. This is the latest verified runtime, not
the current unverified break tree; no high-level mathematical acceptance.

Historical break3838675 terminal FAILED1:05, report
be744d85056bff9dfa3525133faca807eb80470708f5de7752d0467226b05eea.
Original accepts THREE[[[(0,0,0),(0,0,1)],[(0,1,0),(0,1,1)]]],
TWO[[(0,0),(0,1)]] and rejects numeric old inputs. Exact repeated depth
messages are captured. Original signal remains invalid throughout.

Local language inventory is now118/61provisional positives: the next two
operator_value_* fixtures probe basic.at's later bare symbol assignment,
including nonfunction rejection. No operator-binding runtime changes yet.

HISTORICAL BREAK DISCOVERY3838675 is SUBMITTED in
/public/home/majj/atlas-break-history-discovery-20260928.FMfdu2CL, pin
13dcb42e1d97df98bb121f85913322041927c51e979f06e3a779d9301157ed70.
It captures all old numeric evaluator sources and repeated two/three-level
companions before changing historical units.116language cases/60provisional
positives, verified scope3838545 runtime. Note upstream RAW AST still prints
break1 numerically in error expression context, while its depth diagnostic
and source grammar use repeated break; do not indiscriminately change
compact_expression/break_shape to repeated spelling.

BREAK R2 DISCOVERY VERIFIED:3838661 FAILED1:52/CAPTURE_FAILED_ORACLE_EXECUTION,
report0709cb49246cfb62917e25e70a4b4c90c2275cca22e1a13fb956aed0ad5ebd8c.
1320scope files/113cases rechecked. The repeated-BREAK positive is entirely
ACCEPTED in original, including PREFIX[7,7]["seven","seven"], TAGGED[7],
BREAK[1], NESTED[[(0,0),(0,1)]], RECOVER59. Rust rejects generic result types
and repeated syntax. Lexical errors recover61/67/71. Isolated numeric break1
is original REJECTED_SYNTAX but Rust ACCEPTED: retain that wrong acceptance.
Only invalid oracle remains the old captured-selection signal. Mode R2
3838648 has passed its unchanged context unit and45type checks; full gate
still pending. Local master now116/60positive: exact historical two/three
level sources and their current repeated counterparts are prepared for a
fresh discovery stage FMfdu2CL, pin13dcb42e1d97df98bb121f85913322041927c51e979f06e3a779d9301157ed70.
Break implementation and historical tests are still unchanged.

ACTIVE JOBS: mode R2 build3838648 (neoXSfc7, frozen108cases/expected446core)
and break discovery R2 job3838661 (VTutFn6J,113cases/59provisional positives).
Break R2 stage /public/home/majj/atlas-break-discovery-r2-20260928.VTutFn6J,
pin57ecf53ae49a8be98cfd206efcf5596096deb8545e71ab5e392d13e0086b2f4d.
Collect these exact handles; do not restart on a transient observation timeout.
Latest fully verified runtime is scope3838545 (443core/CLI/final integrity);
local runtime is UNVERIFIED mode R2, with no break repair. No commits this
iteration. Next: inspect mode tests/full capture, then valid break oracle
streams before adding its unchanged regression and minimal type/grammar fix.

CURRENT LOCAL LANGUAGE INVENTORY113/59provisional positives adds
break_polymorphic_valid (repeated BREAK), break_lexical_valid, and isolated
break_numeric_rejected. Retain the previous two rejected discovery fixtures.
Fresh R2 discovery pin57ecf53ae49a8be98cfd206efcf5596096deb8545e71ab5e392d13e0086b2f4d
is in scratch break-discovery-r2-pin.json; it reuses scope3838545, not mode R2.
Break runtime and historical tests remain unchanged. Mode R2 job3838648
remains the active candidate, expected446core and frozen108case capture.

LIVE MODE R2 JOB3838648 is SUBMITTED in the fresh neoXSfc7 stage below.
Break discovery3838627 is terminal FAILED1:06/CAPTURE_FAILED_ORACLE_EXECUTION,
report6201e722efce04357474fa81b5880349fe0decb43a2be557e95870a4b57b185d.
Original DOES accept generic break_prefix/break_gather and prints
PREFIX[7,7]["seven","seven"], TAGGED[7], BREAK[1], RECOVER59, while Rust
fails both definitions with[void]/[A]. BUT the combined provisional positive
also contains break1 numeric syntax, which latest original rejects at INT.
Therefore it is NOT an accepted whole fixture. Preserve it and capture an
isolated accepted companion plus current repeated-BREAK syntax before any
syntax/old-test migration. Negative fixture likewise combines lexical errors
with that syntax rejection; recovery61/67/71 alone does not distinguish them.

MODE R1 TERMINAL:3838593 FAILED4:47 before tests execute, compiler
E0507/E0596 at at_level's Fn closure (moving count/row and reversing row).
Report0e489c7d8abd60cce5e005ec257f1a79589395f43b682181778b0025d64d8067.
The helper calls the producer zero/one times, so current local typed.rs now
uses FnOnce and adds result_level_consumes_owned_values_only_when_requested.
R2 source expected446core; same108case mode capture, unchanged mode tests.
Scratch while-context-r2-candidate.patch/while-context-r2-pin.json freeze it;
pin6f2390d6c45b70339aa88d1ca3941384558873203a9e6bdd4968d31b4ee59dbe,
patche3d0e363b3894162803e5305a44d931d439b6986def6ce81a22127bf6601b0de.
Fresh stage atlas-while-context-r2-20260928.neoXSfc7; submitted as3838648.
This is current UNVERIFIED runtime. Break still unchanged. Preserve R1;
its compiler failure is not an after-pass or a reason to skip any test.

NEXT DISCOVERY3838627 is SUBMITTED at
atlas-break-discovery-20260928.ry6qD674, pin
4f322a4cc8471a108ff8a5670a988de8c111a1c21b740b09d8a15380b976a083.
Uses verified scope3838545, exact110case master/58provisional positives,
adding break_polymorphic_result and break_lexical_rejected. Break is still
unchanged; first collect original/full Rust streams, then keep regression
before any type repair. Active mode build3838593 was confirmed RUNNING4:05;
it has no after-pass yet. No lost/missing handle, no reason to restart.
All submissions, exact pins and terminal reports are in tests/reference/hpc.

SCOPE BUILD VERIFIED:3838545 is terminal FAILED8:44 with explicit
FOUNDATION_UNITS_PASS_ORACLE_UNAVAILABLE. All443core, CLI build and final
source/input integrity pass. Reportce77c127d5437c0c62880cfc8f02b53144f876d3b648eb52c3e0a95a815f72d9.
The new scope fixture matches COMPLETE stdout/stderr; missing-else matches
stdout and rejects correctly but stderr differs.47full matches, no losses
from R3. Original signal remains invalid, not a waived negative.
Unmodified basic.at now parses through that block, exposing TWO semantic
errors at lines53/67: take finds[void] instead of[A], count finds[void]
instead of int. Count is covered by active3838593. Source axis.w:699+
leaves break's type unconstrained; Rust incorrectly conforms it to void.
New local break_polymorphic_result/break_lexical_rejected fixtures extend
language inventory110/58positive; break runtime has NOT been changed.
Collect original evidence before altering break, retaining lexical-depth
checks and historical missing-else void/break tests. A separate discovery
pin in scratch reuses verified scope3838545, not the live mode candidate.

ACTIVE MODE CANDIDATE3838593 is SUBMITTED at
atlas-while-context-build-20260928.8FTAzVQW, pin
7b5de03761b43d179b9e6705298d5f2012f1840f0573c0f51bb142304ee553e5,
patch70923ebc196ccec3ba7ea4fbb44a0b11a3d352b81c852b08069e3ad9e58e272d.
This is CURRENT local runtime, expected445core and108language cases. It
keeps scope-build3838545 frozen at443core/106cases. Scope build has now
passed ALL443core, zero ignored/filtered, but was still running CLI/capture
at7:56. Collect both final reports; do not declare either complete from units.
No commit yet: preserve current dirty source and all failed reports.

CONTEXT DISCOVERY COMPLETE:3838562 FAILED1:01/CAPTURE_FAILED_ORACLE_EXECUTION,
report16360efc3e5cb0b58bc67d482ba42eb34e4f08cfdfaa1075fd5880a8731fd7c8.
All1318R3 source files rechecked; only invalid original remains the signal.
Original prints COUNT3, VOID2, REVERSE[2,1,0], GUARD[0,1], EFFECTS32,
NESTED[([],0),([],1)], RECOVER41. Its three dead-body/escaping-local/mixed
branch errors preserve RECOVER_A43/RECOVER_B47/RECOVER_C53. Current local
runtime now goes BEYOND scope job3838545: adds WhileMode::{Void,Count,Row},
required body context and row-coercion selection, reverse flag, and two new
original-backed session units. These mode changes are UNVERIFIED and frozen
in scratch while-context-candidate.patch/while-context-pin.json. The scope
candidate's two new session tests passed; full443core/CLI/capture still pending
at the last live poll6:27. Do not conflate these candidate source sets.

FOLLOW-UP: context discovery3838562 is SUBMITTED at
atlas-while-context-discovery-20260928.Agyw2Yhc, pin
ee5a250fe237cd4abfef8c624a4fb29848ecf14b09711c9392aac8704e12fca3.
It reuses verified R3 abstraction3838431, NOT the live while candidate.
Local language master is now108cases/57provisional positives, adding
while_do_contexts and while_do_invalid_bodies. This language inventory is
separate from the108case mathematical catalog. Scope-build3838545 is frozen
at106cases. Do not alter either submitted stage or mix the two checker hashes.
No context-mode implementation is present yet; collect the original count,
void, reverse, nested-stop and rejection/recovery output before adding units.

NEW: while discovery3838473 is terminal FAILED0:36/CAPTURE_FAILED_ORACLE_EXECUTION;
only invalid oracle is the retained captured-selection signal. Report
3f5c05114ad0140c40441c01f4bc524c4bd98f444a769c9a542d29272b1a0b24.
Original accepts CASE[0,1,2], LET[3,4], IF[0,1], INDEXED[10,20], STATE522,
RECOVER31; rejects missing else at FI and recovers37. Rust rejects the scope
fixture before execution. Candidate3838545 is SUBMITTED at
atlas-while-build-20260928.aGpeY4uX, pin
f160aabaa39221da8e6155fe090448a5015d00c80efd3ccd92ff4d8a9b9f6472,
patchd7ffe215e43e5b9d8000e480663dd55338bd1d5f49e46196026fd2f91af67469.
It adds the two session assertions, scope-preserving Do nodes, and let/if/
tagged/integer do-case grammar. Dont leaves the body type unconstrained.
Simple while shapes retain their earlier flat AST path.443core expected;
frozen capture106cases, all old assertions and final integrity remain required.
Current runtime includes this UNVERIFIED while candidate beyond verified R3.
Scratch while-candidate.patch/while-pin.json live beside R3 frozen patches.
Count/void/reversal and untagged do cases remain separate open requirements.

UPDATE: R3 job3838431 is terminal FAILED8:26 with explicit
FOUNDATION_UNITS_PASS_ORACLE_UNAVAILABLE, not a failed build. All441core
tests pass (zero ignored/skipped), including unchanged escaping recursion;
CLI build and final source/input integrity pass. Report3413c95f169afbf2b55a48af2406c2680f2b4aa9c729302cdcb7dbd60183e763.
Same104case comparison against discovery3838409 gains13full-stream matches
(33to46), loses none. Original captured-selection signal remains invalid.
Latest basic.at now reaches line73 and fails at DONT in the case-controlled
while, still no high-level mathematical acceptance. R3 runtime remains
uncommitted. New while discovery3838473 is SUBMITTED at
atlas-while-discovery-20260928.cnHMz8a9, pin
168dd50cfad66b61f212518d3c25b15612db5053415cc65073a9a3850e016841.
Its106case master/56provisional positives add while_do_scope and
while_do_rejected; runtime is verified pre-abstraction3837985, not R3.
Collect its full original streams before assigning expectations. Original
axis.w:5887-6110 keeps do condition/body in one expression below let/case
frames. It also has context-sensitive void/count/row modes; the existing
Rust always-row implementation does not establish those modes.

ACTIVE abstraction integration: verified TypeCell checkpoint is committed
c5437a55; current uncommitted runtime goes beyond it. R1 job3838237 compiles,
passes44type/3coercion/parser checks, then FAILS the new accepted identity
session assertion: `(A->*)` does not match free context `B`. The rigid-context
negative test passes. Preserve reportdb624058b53577c21ecbefe4d47dcf7e3469224a32f427f1df2778d379c71c42
and stage atlas-abstraction-build-20260928.qcrIB1Mu; no CLI/full-core acceptance.
Root cause: old lambda Type::specialise cannot assign free scoped variables.
R2 job3838315 FAILED5:20, stage atlas-abstraction-r2-20260928.88IPlIG9, pin
84d4b97f858e0177c09eccba7395e3c1bda4e90fb9d47a0a6b566971a92d2d27.
It retains that assertion, uses scoped structural matching and exports the
body into the SAME function context (including argument/result links).
The unchanged abstraction assertion,59session tests and CLI check pass. Full
core executes438pass/1fail/0ignored/0filtered: the old escaping-recursive-closure
assertion fails (int->int versus free context A). Recursive lambdas still use
bare specialisation. Reportb15f14e5531b12583897c102b8813b29e69f335a8b788a5c1932247233903f80.
No CLI build/capture/final integrity reached; do not claim any_type acceptance.
Its master103case input adds bang/result-cast, sequential runtime failure and
unmodified basic.at loading to the prior100; the original signal remains.
Discovery3838269 failed BEFORE interpreters: the intentional3case subset hit
the old checker's exact100case guard. Preserve its failed stage/report; use a
fresh full103case discovery stage with exact103/54positive counts. Do not
weaken the guard. Fresh discovery3838354 ran at
atlas-abstraction-discovery-r2-20260928.y2HedPZS, pin
62b9ff3a38487adb8ad3f35aba2747b0df4b9779adc3075db2a81b68fb761b59;
it uses the unchanged verified pre-abstraction3837985 binary for before
evidence. It finishes FAILED1:19/CAPTURE_FAILED_ORACLE_EXECUTION. Original
accepts bang/result-cast and basic.at fixtures; old Rust rejects them. Its
middle die preserves BEFORE_THROW5, KEPT[][] and recovery, but the old
classifier treats the unheaded SET runtime interruption as OTHER_FAILURE.
Local hpc/math_generic_probe.py and its6-test checker contain the exact-envelope
classification fix (not in frozen3838315). Full104case discovery3838409 at
atlas-abstraction-discovery-r3-20260928.gaOJ30GC, pin
5c0f25bad9f71985a274a6e8f0ea97dd9f47c56fd9798d546835a87ec46732a7.
It adds the exact historical escaping-recursive source plus generic recursion,
and checks the runtime classifier. It is terminal FAILED0:53, with original
printing ESCAPING7, RECURSIVE7seven and RECOVER23. Report
fb4d877c9800b7724f4dabdd4974d61e18727acbb17befc57f6d502174120231
rechecks1314candidate source files, all104cases and checker controls. Runtime
SET interruption now correctly classifies REJECTED_RUNTIME; the sole invalid
oracle is the preserved captured-selection signal. This is before discovery,
not any_type candidate acceptance.
R3 build3838431 is SUBMITTED at atlas-abstraction-r3-20260928.qrZzti8L, pin
5f7a08ab404480ee30304714dd5349c9a4dc32563545d2bb9c1b7dba8ecee942.
It ports recursive context matching/export, adds postfix bang and
original-backed ordered-runtime tests. Current runtime is that UNVERIFIED R3,
not frozen R2. Read its report when terminal; do not duplicate submitted jobs.
Scratch /tmp/atlas-abstraction-build.S4ZJBAGw holds cumulative candidates built
from frozen binding-after.patch plus git diff c5437a55, with four NEW fixture
add-file hunks appended only once. Source baseline is still canonical3833788.
R3's candidate patch SHA7a7a3360901610d0ee68bdcc47af8638ebc67bcac273383f7331d43a5d4a9cd5.
Do not commit current runtime without its completed core/CLI/integrity gates.

Preceding verified loop-constness checkpoint3837799 at
atlas-loop-constness-20260928.q5u8C75q, pin
e478c719ed408bff7750b2aec7154791bd4488973a32ecf056f55f1f30c015fc.
Includes implicit-constness repair below plus original3837531 loop regression:
row element/index immutable, counted index mutable. All430core pass, zero
skips/ignored, both prior assignment assertions separately pass; CLI and final
source/input hashes pass. Report
041bcb284df25afc75d7615ce364c471c2a1093470ca73e1e194a8b234fd8cc9.
On identical100inputs versus3837694, loop stdout now matches and none are lost;
whole matches remain33. Loop rejection messages are right but stderr envelopes
still differ. Scheduler FAILED7:20 because the full master retains original's
signal, with explicit FOUNDATION_UNITS_PASS_ORACLE_UNAVAILABLE status. This is
not full language/math acceptance. Frozen source is independent of3837694.

Checkpoint committed as9bedb6bc. Its typed.rs bytes match3837799 exactly;
the following TypeCell repair is a separate verified checkpoint. No push.
For future cumulative patches against the canonical3833788 archive, use the
frozen3837799 stage's candidate.patch PLUS git diff9bedb6bc for new runtime
changes; its added fixtures are already present in that cumulative patch.
Do not append the older row patch to a post9bedb6bc-only diff or re-add the
same /dev/null fixture hunks. Keep the exact manifest guard.

Verified TypeCell history: binding scope before3837952 FAILED2:30 at its intended assertion, stage
atlas-binding-scope-before-20260928.pYR0z0ne, pin
1cab87a46ab7ca84202abd8f34de53dfd4216bde0434d770cee13cea3d18ea40.
It adds ONLY a global-identifier import assertion to3837799 runtime. The
compiled test fails0pass/1fail/430filtered: found[A] while[int] was needed.
This proves the global free variable is incorrectly interpreted as inner fixed
T. Original3837308 already accepts the source-level scope contract.
Before report47569b8880473eb2f6e88fdcae682eb650d37548dd288fefe124e2e339814c01;
marker global_imported=false, exit101 after type44/coercion3 pass. Manifest
path set equals3837799 and only typed.rs hash changes. Preserve failed report
and stage; its normal driver reports FAIL and stops before the final rehash.
AFTER candidate3837985 FAILED7:25 at its explicit unavailable-oracle exit, stage
atlas-binding-scope-after-20260928.0fWlmRT7, pin
c22c09521ce46adf72bb4e0c41ac3f225b14ceb8912a1416b6adb6b23958ef6f.
It stores each TypeCell's defining floor and imports only free variables under
inner scopes without mutating the binding. Same global assertion plus local
fixed/free/escape test. Final report confirms unchanged global unit passes with
global_imported=true, plus all432core pass (0skips/ignored,12.89s). CLI and final
source/input hashes pass. Report
8f90fc2bb0194faa7c974ebfafa7561b66b1e452c5e25dcf83e95567f306e8e6.
All100inputs and all100Rust stdout/stderr hashes match3837799, preserving33whole
oracle matches. The sole original signal keeps full capture invalid. Current
typed.rs matches this after pin, with no further runtime edits. Both jobs are
terminal; do not duplicate them. NEXT is actual any_type expression/command
grammar, context raising/lowering and ordered sequential command events, as
specified in the indexed generic-language slice. basic.at remains blocked.

Implicit-constness repair3837694 FAILED7:08 with explicit unavailable-oracle
status (all build/unit/integrity gates pass), stage
atlas-constness-repair-20260928.FD2wMFAo, pin
357207caa13fc992c8e473385ca42c07b5b4f5afbd3d4f739b70283b7f187496.
Expected429 core tests ALL passing, zero skips/ignored, plus both unchanged
global/local regression assertions separately passing. The pin explicitly
sets polymorphic_assignment_repair=true; the checker tests reject missing or
wrong markers and non-boolean mode switches. Runtime derives implicit
constness per pattern leaf at its lexical fixed-variable floor. One new
internal floor test and one original-backed global/local mutation session test.
Final report confirms429pass/0fail/0ignored/0filtered (12.21s), and
both unchanged assertions separately pass: global constant=true rejected=true,
local rejected=true. CLI and final source/input rechecks PASS. Report SHA
ac328bd10b24c73fa7662199a6df4e216a14764ffc6ffe36dc04375e42e504c5.
Against R4 on identical100source hashes, four previously wrongly accepted
mutation fixtures now reject and match complete stdout; concrete assignment
controls remain whole-stream matches. Whole matches stay33, none lost; error
messages are correct but stderr envelope remains different. Only original signal
is generic_captured_selection_rejected. This is a scoped behavior repair, not
full diagnostic/language/math compatibility. Snapshot is frozen; see receipt.
The complete100-case master still includes original's signal. Do not infer
any_type/basic.at or mathematical acceptance from the build/unit gate.

Function R4 job3837531 FAILED7:41 only at the explicit unavailable-oracle exit:
status FOUNDATION_UNITS_PASS_ORACLE_UNAVAILABLE, full427 inventory425pass,
0ignored, both known assignment failures separately reproduced. CLI, all
checker/filtered checks, formatting artifacts and FINAL source/input hashes
pass. Stage atlas-function-build-r4-20260928.JljCv98L, pin
3bb8682fb3343d5c2b0d099f60d49cd4be319515ce23a74008806c9bb279aa36;
report512b0d51e559e6cf9405798069ff1cbff55bcedbaee9b78fc0f3ec27a848b787.
Master100 has33 whole-stream matches; retained generic_captured_selection_rejected
is the only original signal. R4 includes NO implicit-constness repair.
New balance acceptance matches fully; the mixed-coercion negative now rejects
and recovers but stderr envelope still differs. No debug timing ratios.

R4 original mutation contracts: globals reject3 assignments and preserve
UNCHANGED2[]/TUPLE([],1), then permit monomorphic rebinding REBOUND[5]. Locals
reject2 assignments, but accept MONO[1]/SHADOW[2]/LEAF(2,[])/RECOVER17. R4 Rust
wrongly accepts all5. Row-loop element/index assignments reject in original,
while counted-loop assignment succeeds COUNT[3,3]; Rust does the opposite.
The loop mismatch is separate from submitted3837694. In current axis.w,
thread_bindings forces flag0x4 for row loops, but counted-loop bind.add(...,true)
passes numeric1 to unsigned-char flags, despite its stale constant comment.
Follow executable flags plus captured behavior, not that comment.

R3 job3837495 is confirmed CANCELLED by32367 after1:09, not a test failure.
Its new checker path used relative discovery inside the old Rust archive,
which lacks the new script. R4 runs the absolute stage-pinned checker before
compilation. Preserve both frozen stages and the cancellation receipt.

FUNCTION-VALUE CANDIDATE3837257 FAILED2:27. Stage
atlas-function-build-20260928.vIRqFh8S, pin
74fc657b480387d39dfc487695dccc552b9d253ce7c556fd7c48371263ca76e0.
Adds opaque builtin function values, complete-signature overload capture,
shared direct-call argument/result inference and independent tuple-component
imports. Three original-backed session tests cover generic captures, builtins,
display/variadic packing and ambiguity recovery. Full core inventory expected426;
both known polymorphic-assignment failures remain separately required.
Type44/coercion3/syntax48 pass; session51pass1fail. Capture/ambiguity units
pass, but the unchanged captured-printer unit rejects [3,4] against free D.
Full-core/CLI/capture were NOT reached. Failed report SHA
860b557737794596989ad3fd95939a4c0449435b389f339d3348dab86d0d21f3.
The planned capture retains the complete91-case master, including the
original signal isolated by3837092. Therefore the differential acceptance gate
remains unavailable if that signal recurs; do not discard it or infer a passing
build from submitted source. Inspect command checks and per-case streams
separately. No high-level math acceptance or debug speed ratio. Receipt:
tests/reference/hpc/math_function_value_build_submission_2026_09_28.json.

Scope discovery3837308 FAILED1:30 due to the retained original signal, not a
new scope-case crash. Report5ba5290c9c95262571ae5ce6beb004f4f47b6cd057cd8403c9046f447264cb58.
Original accepts outer polymorphic empty-row imports under rigid scopes:
GLOBAL[][], LOCAL([],[]), FIXED(7,"a")(true,3/4). Outer fixed-T assignment
from inner fixed-S rejects (B while A needed). Direct higher-order calls
supply tuple argument context: CONTEXT8/RETURNED10. The concrete abstraction
context hypothesis is disproved: both int->int and rat->rat contexts reject
inside the rigid body, then RECOVER17. Current catalog95 has51accept/44reject;
frozen discovery preserves its initial52accept hypothesis and full signal case.
Do not change expected outputs to Rust or infer a successful differential gate.

R2 job3837402 FAILED8:19 at atlas-function-build-r2-20260928.CkhXMtK0, pin
bef0db4176d62bd7d0c69603fc49b693e05d4e75e40c9214023562d6ca239731.
It specialises list contexts to rows and replaces monomorphic-only balancing
with scoped joins, retaining independent first-pass contexts and monomorphic
coercion reconsideration. Same failed printer assertion plus original3837308
direct-call control. Full426 core inventory and all95 capture cases required.
Full426 inventory424pass, known2 separately fail,167filtered checks and CLI
pass. All95 inputs match scope-discovery3837308 hashes:32whole-stream matches
versus27, five gained and none lost (function capture/display/direct contexts).
Original signal prevents valid full capture; this older driver stopped before
final source rehash/format artifacts. R4 fixes that reporting/integrity path.
Reportbf6f140046c3ad314effd52c1a9037ee16e6972e64d24804b67fb78cdc58156a.

Balance discovery3837421 FAILED50s at atlas-balance-discovery-20260928.D9rfTLkB,
pin56ed08c40157bf5ab2bd41dbd8f774b2dc9ddf6904ba56882b2a20d0fd8c5528.
It uses unchanged verified3836533 and the full97-case master, adding the
current axis.w polymorphic tuple/row balancing example and its documented
mixed-coercion negative. Original accepts BALANCED[([],[3/4,5/1]),([2],[])]
and EMPTY[[],[]], but rejects the mixed rat/bool versus int/free-row list.
Before3836533 does the opposite on both. The overall failure is the retained
original signal, not either new case. Report
1eb50e3e60b1f46fa22debca4dd3a935c200c7569df8d08c5382898e3472d54d.
R2's already frozen95-case gate is unaffected. Keep the original signal in
both; neither submitting a job nor collecting normal per-case output proves
all-language acceptance. No local builds/tests/checkers were run.

CURRENT WAVE:3836397 FAILED5:59, full421-test inventory416pass/3fail/2filtered;
all163 filtered checks and CLI check pass. Report
b822046943c022d690d1f4df8acf34d8d5513cec8748625fc808d59bd3a8aaf5.
The three failures uncover stale empty-row/error-phase assertions AND the real
hidden-special registry defect: current original registers row/printer/error
schemes as ordinary overloads. Original captures3836455/3836507 confirm all
87/88 intents (48accepted), including3#[] ambiguity, mixed empty/concrete row
joins, and generic/concrete cardinality conflict. Failed source stages and all
receipts remain preserved alongside the verified replacement.
Replacement candidate stage atlas-row-build-20260928.cJPjhwE0, pin
0962e36bda2ea15fbd15b97a07a13e5888ee1b0e4f45687b3501e4162042792d,
completed as3836533 in14:01. Full423-test inventory:421pass/0ignored, both
known assignment failures separately executed;164filtered checks and CLI pass.
Report ad9cd5d815fae466ed5557a40575c192dd3be94065f4a4e0986ddaadf3764188.
All88 original intents confirmed;27full-stream matches versus19 on the same
corpus with verified3835776 (3836507), eight gained and none lost. Compared
with older release3833612, only3 whole matches; do not conflate these baselines.
The row registry/value/mixed cases and new projector/field positive controls
match completely. Rejected ambiguities now reject and retain recovery values,
but stderr differs (Program vs Type envelopes); they are NOT exact diagnostic
matches. Tagged union context values and binding types match; the remaining
stdout mismatch is bare whattype ni/ns: original reports identifier kind/type
and introduction location, Rust only prints Type. This is partial language
improvement, not basic.at acceptance.
It removes the hidden fallback and preserves linked variables/query order.
Keep all failed gates3836297/3836325/3836333/3836397. Function capture3836435
confirms all82 pre-row intents after the narrowly corrected classifier;
3836409's invalid report remains. Next after this registry gate: whole-signature
function-value capture/direct-call inference, then actual any_type/TypeCell
scope migration. Builtins need a real function value, not a fake user closure.
No new high-level mathematical acceptance or debug speed ratio. Runtime files
were rehashed after submission: all99 pinned changed/source fixture bytes match.

NEXT DISCOVERY3836975 FAILED30s: catalog91 (three added function-value details)
has90 ordinary oracle outcomes and one ORIGINAL exit139 in
captured_selection_rejected.atlas. Report
bf964d529142206a2f2b0ebf5ee862e9f2deb7db4a517c9ee6565fd665b87ee1.
Preserve its source and signal classification; do not turn it into a matching
rejection or silently remove it from counts. It must be isolated before using
this enlarged catalog as an acceptance gate; row-build3836533 still uses its
frozen88 cases. Builtin display succeeds: {succ@int}, {pred@int}, {prints@A};
captured printers retain tuple packing. Lowercase ascii in the runtime discovery
is undefined in latest original: global.w:4147-4148 spells it ASCII. Keep that
negative and capture a separate uppercase companion. No runtime changes made
for function values yet; all local runtime bytes still match3836533's pin.

Diagnostic subset3837092 FAILED12s isolates the ORIGINAL signal to the single
command (string->int):succ, without any preceding error or state. Nonfunction
int:succ and no-match (int->bool):ASCII reject normally. Uppercase captured
ASCII reaches the expected range error, and all three runtime controls recover.
The five-case selection_isolation_catalog.json retains the signal case and
the positive display control; it is explicitly separate from the91-case
master, not a reduced replacement acceptance suite. Preserve both failures.
Isolation report SHA
bc9e813893eb1df0b39cba255ac17ef342546dceab2f84a4824a84a02ee33955.

Current user objective now ALSO requires root-cause analysis and sequential
repairs of discovered errors, recording reusable lessons in AGENTS/indexed docs.
Do not stop at inventorying defects. All testing/builds remain HPC-only.

VERIFIED constructor R3 candidate3835776 COMPLETE8:34, stage
atlas-constructor-build-r3-20260928.WDBn6GSS, pin
b178d4d0449577858f0719957045c2a5a61bc7104fddd0a20da54d76e4803ebd.
Runtime remains R2; only the new structural unit now checks complete ReportLine
strings rather than the wrong Output channel. R2 build3835267 FAILED4:38,
43type/3coercion/48syntax pass, session40pass/1fail, report
cd6fbcb13feacf29d451b9c748f785303332e3ad2f88280534eab5bbaf0f62e9.
Its failure log contains all five correct values; no full-core/CLI/capture was
reached. Keep both failed constructor builds. R3 full413-test inventory confirms
411pass/0ignored and both known assignment failures execute separately at their
exact assertions.155filtered tests and CLI check/build pass; all62original
intents confirmed37accept25reject,18full-stream matches (previously15). Report
cdd254bf8cafe5b01490d722b894e200ae3f1816a9b7fe0e72bad09563959824.
Source hashes match the eight runtime files committed as adf40792. Arity rejection category
matches but stderr differs; generated projectors still fail unification.
This is not full generic/latest-basic.at acceptance; no debug speed ratios.

Member discovery3835786 COMPLETE29s with unchanged3835058 candidate confirms
66/67 provisional intents, report
19290257a122c50dac212cd92cfd0e34edaab7d28ac6cf783cf43340328a8456.
The failed intent binds reserved fi; keep its rejected source and add a separate
int_fields/rat_fields companion. Generic union context, repeated-argument
rejection/recovery, generic/concrete ambiguity and linked result rejection are
now captured. Replay3836211 COMPLETE36s proves original accepts generic field
writes and monomorphic writes even after a non-projector overload replaces the
projector; Rust rejects all four writes. Rust also returns99 for the ambiguous
concrete call, then fails only at rat recovery. Whole-stream comparison is
essential. Report36bfb7d7cbafeb3c2587eaa8621c8d656cd32745f01223364d87600b74f3f6b7.
Its positional case discovery has invalid ():0. Replay3836223 COMPLETE26s then
disproves acceptance even with (@:0): latest original requires case/bar/pattern
clauses for positional union discrimination, not case/in/function branches.
Report1bdd5f611420cdff30e7cee87bade9be0bf39995f15775bf53498ba7ec7b6b70.
Preserve both rejected discovery sources and the historically named _valid file.
Catalog73cases41accept32reject adds current pattern syntax and a monomorphic
legacy-syntax negative. Replay3836236 COMPLETE26s at
atlas-constructor-members-r4-20260928.9fKTqaYu, pin
ad85d47acaf9d5629424abd67db4782055bebb6d4438632ac17e59c860ba5a26,
using the exact verified3835776 binary confirms all73 original intents,18
full-stream matches,6capture+3bridge checker tests. Report
257b50d1d83d595dfca7cdbccbf105a5132cd68e418986a8e57710af2960d8f1.
Original current-pattern case gives POSITION8/EMPTY0; Rust rejects. Original
rejects the monomorphic legacy syntax, but both Rust arms accept and print
LEGACY4/RECOVER17. All jobs in this wave are terminal; no duplicate or replay
needed. Runtime is committedadf40792; proceed to active inference/semantics.
The indexed generic-language slice records executable field-binding lookup
versus stale projector-closure prose, and the linked overload inference port.
Next active integration must preserve linked argument/result substitutions and
ambiguity while migrating Analysis/TypeCell floors; any_type remains the latest
basic.at blocker. Field/tag matching must search retained type definitions.
Historical casefor_b6 syntax expectations need current-original migration,
not silent golden replacement. No new high-level mathematical acceptance.
Original7e1b958c and GitHub main05625c5d unchanged; HPC Git ls-remote verified.

PREVIOUS VERIFIED BUILD3835058 COMPLETE7:45 at atlas-lexical-core-build-20260928.QM7IzLdg,
pin550899263c93e43a298b1481e353ab936435e19013f6637e0b403e649cf18fa6.
Full407-test inventory verified:405 pass,0ignored, and both known assignment
failures execute separately at their exact assertions.150 filtered checks and
CLI check/build also pass. Core suite13.782s/155048KiB;1227 source files verified.
All56 original intents confirmed35accept/21reject,15 full-stream matches.
Report SHA0dc45ae377d879bc213e893b03ec6096d1619f403582950b8c64939195d6400f.
The retained FPP/F4/E6 mathematical units remain passing. This is not generic
language support or full high-level mathematical acceptance; no debug speed ratio.
Capture3835038 COMPLETE16s proves the exact failed unit's latest-original
declaration text retains Pair and values remain(3,4),3,14,(3,14). All56 intents
confirmed;15 full-stream matches. Only the report assertion/comment changed,
no runtime implementation or value check weakened. Report
86bddb98e26b3c246c56db0d594c19614d8aa22ba838d5c77b26f6315ac5df17.
All jobs in this wave are terminal. Do not duplicate3835058 or the failed review.
Next grammar-action/lookahead integration details are in the generic slice;
LALRPOP0.22.2 always fetches lookahead before reducing on it.

NEW SCOPE FOUNDATION build3834975 COMPLETE6:50 at
atlas-lexical-scope-build-20260928.BKCNlPyQ, pin
2c2184a056b01dddb3e928dd59ef441f88c81ddc39e5aa4d95b68949351da74d.
149 related tests and CLI check/build pass;55case capture confirms all current
34accept/21reject intents, retaining14 full-stream matches. Report
b6bc69edcb2f02ad4b2022390b9e7fa55eb43f456b76efb1ec3fc862d6c0c16e.
Moves name classification to lazy token consumption, adds shared lexical type
scope with six tests. Grammar actions/AST/inference are still unconnected;
this is not generic language acceptance. Capture3834951 COMPLETE29s, report
0f126fe0d88e2507ed32958a836ca3365c1196a4b5f513ef6e0e9b43f7ea32fc,
adds six boundary probes (catalog55): four accepted values, one TYPE_VAR binding
rejection, one disproved acceptance (constructor bang on the following line
is rejected). Keep the frozen discovery; current catalog34accept/21reject.
Read the generic slice for precise values, before streams and remaining work.

NAMED-TYPE MIGRATION: before capture3834815 COMPLETE22s, all45
original intents confirmed (26accept/19reject), exact3834754candidate retained.
Nine new cases cover lifetime/redefinition, structural equality, rows/functions,
copied fields, union tags, forget and nominal recursive rejection. Simple union
discrimination and type-name reuse after forget fail Rust, pass original.
Report23008c2c5d3173032330d63499fd432aeb6a5c641ff1f1b00d9d6250616a13ff.
Candidate build3834852 FAILED2:47 at atlas-named-build-20260928.Njj1ymrl,
pin0651483feef4ed9bf7287ba83c7507b28db1771764f977d8e4991e6411f44a11.
41type/3coercion/40syntax pass; session35pass3fail on missing newline in spans
and wrong event-kind test assertion. No CLI build/capture occurred. Edge
capture3834868 COMPLETE21s verifies all49 original intents and disproves the
assumed named-void discarding and old-projector cleanup. See generic slice.
R2 build3834895 COMPLETE6:30 at atlas-named-build-r2-20260928.xa3ImNjp,
pin a4ee2a14a2bab18d47e776dd38b0148aee6adc86e33e71a5035d785513af195a.
It passes143 related units (42type/3coercion/40syntax/39session/19session-frame),
CLI check/build and49case capture. Report SHA
386976d612e9438e0c1acd4d91c4331ac4f8e69440a6380013f686c86714c8ab.
Fourteen whole stdout/stderr matches versus two before; simple named union
discrimination and forget/reuse now pass. Six named negatives reject correctly
but diagnostic prose differs. Ordinary identifier queries and closure printing
still differ. Generic scopes and implicit polymorphic constness remain open.
No speed ratios from this debug candidate. Full-core review3834919 FAILED1:00
at atlas-named-core-review-20260928.LxyUVwBE. Pin
cf0b25c7f52684825b4b0c5debe601d925e658c8279de23e7414c758defad381.
The exact built test binary inventories401 tests:398pass/1fail/2filtered,
16.83s/154644KiB RSS. Failure is alias_name_declares_a_variable's historical
expanded(int,int) report expectation versus retained Pair. The exact source is
new named_declaration_fields fixture (catalog56), being captured against latest
original before changing the unit. Both known assignment failures' separate
runs were NOT reached in this failed review; do not claim them executed here.
Report6455ab36eb8d096f1a850fcae14146d459c79c168a3a59ea8744c3d77ac79c6b.
fat projected start onOctober1; same pending job moved to cpu/8GiB, actual4CPUs,
and ran without rebuild. scontrol memory takes integer MiB; NumCPUs does not
change CPUs/Task. The future review script defaults to cpu2/8GiB. No duplicates.

LANGUAGE BRIDGE BUILD:3834754 COMPLETE6:17 at
/public/home/majj/atlas-language-bridge-20260928.Zdtz3EAN, pin
c23be725d1153c89df51bf5b7d2015eec13c074ccea2819f2628449d59db1d97.
Candidate connects persistent type-name classification to session/redirect
parsing and resolves semantic cast/parameter/recursive-result annotations
against the live table. Five session/redirect tests added first; syntax,
session, session-frame, type/coercion suites and CLI build run on compute.
All136 related tests and CLI check/build pass. The36case capture is INVALID:
all original arms fail startup with GLIBCXX_3.4.26/29 missing, due to the
build-only batch environment lacking GCC's runtime-library path. Retain report
24929729a67f2ce0d28fcec03d0008903088b4b0ecb6499e3e3742e1600c964e.
Corrected replay3834785 COMPLETE52s at atlas-language-replay-20260928.sZ06vqxJ,
pin20b933551b59ad0e09e5c4378bb81d0c05f184ca3f2817537c803e5c10c8d05e.
It reuses the exact binary and rehashes1213candidate/606oracle/1212before-Rust
source files, logs/inputs/scripts; all36 oracle intents confirmed (19accept,
17reject),9 checker tests pass. Report SHA
bf738be8f6ab362be0cfe0b2988f2881b0611daec378b4d9a47be986f2a2db1e.
Named casts/parameters and recursive result calls now execute correct values;
bad components reach type rejection. TYPE_ID parameter binding was wrongly
accepted before and is rejected now. Complete outputs still differ in retained
type names/query reports and diagnostics; only two existing mutable controls
match fully. Bare TYPE_ID rejection has an end-of-file versus newline location
mismatch. Preserve those differences; do not claim general language acceptance.
Debug timings must NOT become a speed ratio. Existing
generic scopes, implicit constness, named-type retention/reporting remain open.
Both jobs terminal, no active jobs from this wave. See bridge/replay receipts.
Next: retained named-type table/definition locations, then scoped TYPE_VAR and
the scheme-carrying analyzer; source-audit boundaries are in the generic slice.

NEWEST LANGUAGE FOUNDATION: matching3834447 COMPLETE4:44 passes38 type tests,
3 coercion tests and CLI check. Eleven new tests cover structural matching,
two-sided rollback, direct-function argument/result substitution and differing
scopes. Source polymorphic.rs e8d63d0eed149db31c60306a669644148e1c3ec123e5d7fed198739911b18cd0;
report799066fe0f91e45622f9bbc9b48c18e6ce9c86e47c8995641dd4dde81637b2ec.
The global regression still executes/fails; active language integration remains.
Capture3834702 COMPLETE34s at atlas-generic-calls-20260928.f5mSx0Od confirms
all31 oracle intents (17accept/14reject),6 checker tests,606original/1212Rust
source files and264 scripts rehashed. Report
31901e9563db8128d51a4c61d60382984f000acbc7dbea3d8d37e42df3ddd271.
New named-type, direct/tuple-polymorphic-call and rigid-result raw streams
inspected. Rust still fails their syntax. No active jobs from this wave.
See matching/R7 submission receipts and the indexed generic-language slice.

PREVIOUS LANGUAGE FOUNDATION: owned `InferredType` scope wrapper and nine new
tests verified by3834389 COMPLETE4:15 at atlas-type-scope-20260928.VbWRGzcY. Pin
29c461ae28757dfe1b52fba2e953baeba432b8c326ca5dd5127e452bdcaf002d;
polymorphic.rs a45e0edf329baaf3ef0fbd6d5e59a0cf077d36604033fdb18a5036cc24864b02.
It carries pending assignments through disjoint tuple/overload imports,
bakes before raising floors, and keeps argument/result substitutions linked.
It does not change the active parser/analyzer or claim either constant-binding
regression repaired. All27type/3coercion tests and CLI check pass; the known
global unit still executes/fails as expected. Report SHA
40946d39c30790af6504e76a3a2130ac7cfc629b2ba263866e8044eff8ccacbe.
No active jobs remain from this wave. See generic-language slice and
math_type_scope_submission_2026_09_28.json; do not duplicate terminal jobs.

LATEST LANGUAGE IMPLEMENTATION: internal type foundation job3833858 COMPLETE
4:12;17 type tests,3 coercion tests and CLI cargo check pass. The prior global
assignment regression remains an executed failure, NOT repaired. New owned
TypeScheme/TypeAssignment and variable/applied-constructor representation are
not yet connected to the parser/analyzer. Exact report/source pins are in
tests/reference/hpc/math_type_foundation_2026_09_28.json and the indexed
generic-language slice; do not claim generic or latest-basic.at support.
R4 capture3834266 COMPLETE22cases discovers the analogous LOCAL empty-row
wrong acceptance; it also disproves same-name nested type shadowing. R5
capture3834282 COMPLETE24cases confirms the concrete local assignment control
matches fully. Preserve both discovery fixture mistakes (>= lexical trap,
single named struct field), and collect the separate valid two-field probe.
The new local unchanged-runtime unit probe3834285 COMPLETE3:32 at
/public/home/majj/atlas-math-polymorphic-local-probe-20260928.jPmq9lgO;
executes0pass/1fail after the concrete control succeeds. It adds only that unit to accepted
mathematical runtime3833716; it does NOT include the new type foundation or
the global unit. Neither global nor local constness is repaired yet.
R6 capture3834288 COMPLETE25cases/6checker tests; all oracle intents confirmed,
including the valid two-field constructor with an unused argument slot.
Foundation R2 job3834296 COMPLETE6:25 at
/public/home/majj/atlas-type-foundation-r2-20260928.GFARslae: same structural
Applied-specialisation unit executes/fails before and passes after among18
type tests;3 coercion tests and CLI check pass. Report SHA
adbc4b8f88ea06825ec2623eccf811aca4c7920a972a7ec7dd7f43c748906902.
Current types.rs46fab125.../polymorphic.rs07082292... match the tested source.
All language foundation/capture/local-probe jobs above are terminal.

USER GOAL EXTENSION: investigate computational parallelism and select speedups
using HPC A/B. Existing observer forced one Rayon thread, despite parallel
Weyl/inner-class/KGB code. The new1/4-thread E7 KGB experiment uses accepted
build3833716 (same binary both Rust arms), four balanced fresh-process rounds,
full oracle/stdout/stderr equality, wall/CPU/RSS/affinity capture and independent
review before any speed claim. See `slices/parallel_ab_2026-09-28.md` and the
parallel submission receipt. Do not conflate serial-to-parallel scaling with
Rust-versus-original performance. A/B job3834321 COMPLETE6:23 at
/public/home/majj/atlas-parallel-ab-20260928.15hQxOhx with4CPUs/8G.
Pin535e979fde936c98c02a04b2561ee17b7ec423e4c2bd6287413e19975aeba889.
Independent review3834377 COMPLETE1:01 at atlas-parallel-review-20260928.fL023YvY,
54 checker tests pass; all36 raw artifacts, source/binary/script pins and the
four-round schedule verified. Report SHA537d2a1a986e2dcfc9ac341994eef4b90577e152d0ddcf3007620de55bb008ea.
E7 KGB median original1.327984s/Rust1 69.595196s/Rust4 23.523671s. Paired
scaling2.958576x; peak-RSS ratio1.185404. Selected FOR THIS WORKLOAD only,
not a global setting or algorithm change. Rust4 remains about17.7x slower
than original and uses about41.9x its maximum RSS. Both Rust arms spend
about69-70 CPU seconds, so the excess work remains. See the indexed slice
and math_parallel_review_2026_09_28.json; both jobs terminal, do not duplicate.

LATEST COMPLETED WAVE: E6 imaginary-II Cayley repair at
/public/home/majj/atlas-math-cayley-repair-20260928.FFGcIkPi.
Preflight3833713 PASS56 checker tests; build3833716 COMPLETE10:21 and
15-case array3833718 terminal. Independent review3833739 COMPLETE16s,
56 checker tests pass:14 complete mathematical matches and1 expected rejection.
E6 core KL81 now matches every output byte; F4 history107 remains exact.
Build SHA62aa96181c87d0ae816928dcbab37b5b7b448d1d1d00632106e28214319b2e63;
review SHA4a7f3cc2b77832f553977336912ca6dcb2fc6889fa48454f73d40e8dfd6ca98e.
Do not duplicate these jobs. Input pin
545d1a3bf86d0d1a3871bdd41c6b041e5ee79727cae5d38088fb2eb2182a4c9a;
review-code pin10478f30b2b00cd296abf3085b3abae638d97de47b86a62db00a7d5d406bae04.
Before = endgame3833612 plus unchanged E6 regression; after changes ONLY
optional upward imaginary-II Cayley polynomial terms to zero. Core
4aa28dec...; KL beforef9226ab7.../aftera6d63837.... Same before regression
executes/fails, after passes at integer/half scale; all targeted unit suites
and release build pass. Runtime acceptance is limited to retained cases,
not all real forms, E7, high-level scripts or the full goal.
Before probe3833624 actually executes/fails at second image, logging absent
second images (e.g. x1282,s5,Some1344/None). Report SHA
afda003dfd27b6fd88b99afad00c173c5ba002501e66a996255f37e050c6f11c.
Probe3833616 instead failed staging due to an extra.orig file; preserve it
as a harness failure, not mathematical evidence.

F4 ENDGAME VERIFIED ON RETAINED CASES: build3833612 COMPLETE9:40, same
cold/history coefficient test fails before and passes after; previous targeted
units and release build pass. Build SHA
e2ce570826277bf24377b4e0b627f071f264c8ff8144ae789d04c9692e448b8f.
Array3833613 and corrected independent review3833712 COMPLETE:
13mathematical matches (including full F4 history case107),1rejection,
1E6 second-image failure. Review SHA
bddc64b66afb6e92edfc3037e8b52ca9ea3e39b81836c064aa69c84bd7dfd3bb.
All606 original/1212 Rust files,264 scripts and full output streams rehashed.
First review3833635 failed before mathematics because tar member names had
./ prefixes. Corrected reviewer ran55 tests, canonicalizes only safe relative
paths, rejects duplicate/unsafe members and rechecks UNCHANGED original
archives/builds/results. It lives in atlas-math-endgame-review-r2-20260928.PtUN9kig;
do not mutate the original endgame stage. No interpreter rerun was needed.

E6 coordinate review3833590 remains retained: all8 KGB cases match.
The deeper Cayley regression and full E6 core comparison now pass3833739.
FPP E7 resource/time limits and high-level generic-language gaps remain open.

Next language frontier source audit: latest parser.y has ANY_TYPE command
and expression productions, scoped TYPE_VAR and parameterized constructors;
basic.at starts with Pair/One_of/Maybe/Iterator and many polymorphic scopes.
Rust lex.rs still explicitly tests any_type as an ordinary identifier. The
generic gap is not just one missing keyword. Preserve current oracle cases
72-74; port scoping, type substitution and overload behavior with negative
fixtures rather than rewriting upstream scripts. See the newer foundation
status above; the active parser/analyzer is still unchanged.

LATEST LANGUAGE EVIDENCE: capture3833758 COMPLETE16cases/6checker tests,
reportc321581d1666d9856951701ba1ab0c83955643f5447ba20b9b77f381de3d29d6.
Duplicate<T,T> instantiated<int,rat> has TWO int projector results; later
duplicate slots remain unused. Explicit generic selection succeeds, ordinary
generic/concrete int application is ambiguous, occurs-check example rejected.
All8 original-positive cases fail in Rust; Rust wrongly ACCEPTS assignment
to an untyped empty row. R3 capture3833787 adds the concrete `[int]` control:
all17 oracle intents confirmed; that control matches full stdout/stderr.
R3 report9184eb016e262418882e8b2cf385512d3da16bafc20c1d71edeccf688785ef49.
Same-runtime unit probe3833788 COMPLETE2:18: executes0pass1fail with
`constant=false rejected=false`, after concrete assignment succeeds. Source
typed.rsa5c919d9... contains ONLY the new regression on parent3833716.
Report1ab2f23b5b520f0a29a88ef6469ab41bddbbce5012b6e0043a650b78d7c301c0;
log79f9970bbabf079f6e5fcbdcd5a6d23f2e2fff6bd4b56312958efca8a441f035.
All jobs in this paragraph are terminal; do not duplicate. The new unit is
intentionally still failing, not ignored. No language implementation yet.
Next implement the scheme/scope/constructor/overload boundaries described in
`slices/generic_language_2026-09-28.md`, preserving all17 probes and the
unchanged before unit. Current Rust has only independent `Undetermined`
holes; these cannot represent repeated linked type variables. In particular,
implicit constness comes from the scheme in global.w:992, not the value[].
Avoid a one-off list-constant patch that leaves type soundness and reports wrong.

Earlier generic discovery:

Generic contract capture3833740 COMPLETE27s,4 checker tests pass, at
/public/home/majj/atlas-math-generic-probe-20260928.7vz3U5lz.
Pin eafd8bf29b7593bcf0e366a5b8245a1c1f573f9a1d937cd4924cadeb355636dc;
report3333d195a2b86dd52a0d0a6623041e229e1918364855aa88f0ed81879b08ae01.
Eleven separate positive/negative probes use unchanged build3833612.
Original accepts nested pair, identity instances, result context and explicit
operator scheme, but generic+concrete int overload call is AMBIGUOUS.
Duplicate formal names are ACCEPTED, contrary to provisional fixture intent.
Arity errors are explicit analysis rejections lacking a `Type error` label;
the conservative capture calls them OTHER_FAILURE pending exact review.
All original successes fail in Rust. Scope escape has syntax errors in both,
but Rust fails the preceding declaration too; that is NOT equivalence.
Next review raw streams, freeze evidence-backed expectations and extend the
duplicate/substitution and overload cases before implementing type schemes.
Initial one-CPU8G submission created no job (reported allowance4G); overriding
to2G completed the capture. This is not a partition-wide limit; two-CPU
repair jobs used8G. Do not mutate the frozen capture. Receipt and raw paths
are in tests/reference/hpc/math_generic_*.

### Earlier wave snapshots (superseded by latest results above)

E6 coordinate repair stage
/public/home/majj/atlas-math-e6-form-repair-20260928.l7uK40QO;
preflight3833556 PASS45checks; build3833558 COMPLETE9:26, before E6 test
executes/fails, after passes; form-order6/KL6/locator10/F4-boundary1/FPP1 pass.
Build SHAe15d6dc305a21079cbc316f52463652e22eb1305a6b7ded85892b0f630850888.
Dependent array3833560 running (23cases,
including every existing KGB group, E6 core KL and F4 history). Only
real_form_order.rs differs from3833274 plus the unchanged E6 regression.
It checks actual ordered ambient unit-vector representatives, retaining
specialGrading's partition election. Pinned independent review3833590
submitted afterany3833560. Collect its result; do not duplicate or accept
the runtime on unit evidence alone.
Parent review3833539 is now COMPLETE, SHA
e2d648eaefd212e72863c64d034e5bd5c6f966e282ecc859330c0bcf8e4025d5:
12mathmatches,1rejection,1F4 history mismatch,1E6 failure,1original E7 failure.
F4 core outputs now match, but index107 exits0/empty stderr in both engines
with different complete output. This is a genuine comparison failure, not
an after-pass. Retain case107 and diagnose its sections; do not waive it.
E7 now also allocation-aborts Rust under6GiB (exit134/6250992KiB), while
original still std::bad_alloc. No completed ratio or mathematical verdict.

F4 stored-output diagnoses3833562/3833564 are COMPLETE without interpreter
reruns. R2 SHA9f3752fed4e751594560190f99c85556093ad26e17cf94825efe83509e72bd4b.
All half/zero/unit cold/warm partial outputs match. Only unit-scale FULL
pool102 at matrix(4,334) differs: original[0,0,2,3,3,2] versus
Rust[0,0,4,6,5,3]; all336x336 indices, parameters and175other polynomials
match. Current core adds a focused test of this coefficient in cold-full
and exact prior-history contexts; probe3833589 is submitted, not yet reviewed.
Static suspect: kl_table.rs::first_endgame_pair currently uses `cross?`
(wrongly returning None for an absent cross) and returns Some(s,None) when
an existing cross has no valid t (wrongly stopping the outer s search).
Original kl.cpp:318-340 does the opposite distinction: absent cross gives
Some(s,None); existing cross with no suitable t continues searching s.
Do not claim this explains P(4,334) until before/after and full outputs prove
it. Probe stage atlas-math-f4-full-kl-probe-20260928.pv3QKQOM is submitted
against3833274 runtime, not the E6 repair; only the new core test is added.

COMBINED REPAIR BUILD: checked coroot sums were staged together with the
real-II KL boundary repair, retaining all integer/half-scale regressions.
Stage /public/home/majj/atlas-math-kl-coroot-repair-20260928.vCEV2VU4;
preflight3833256 COMPLETED(44checks); build3833274 COMPLETED7:57.
Identical before tests execute cross panic and both coroot failures; after
F4 integer/half passes, coroot2 passes, KL-table6/locator10/FPP1 pass.
Build SHA364a6b02f6cc4fb3bcf2e5762c20b5b32aeede31609c98e0585cbce154a01215;
Rust binary SHAd1d648fb1eac8dd02c1059d3e5eb8939edb33e7c63341a79fc4faa101d0c3313.
Dependent array3833289 and independently pinned review3833539 are terminal;
see the residual history mismatch above. Do not duplicate this wave.
Indices75-82,89-91,103-107. Do not duplicate or mutate this frozen stage.
Receipt math_kl_coroot_repair_submission_2026_09_28.json records every source
and input pin. Build/unit and differential evidence are retained. Runtime
is NOT fully accepted or committed because F4 history remains unequal.
While HPC executes, analyze E6 real-form ordering: verified_generator_map
chooses the first flipped imaginary-subsystem root, which need not be a
datum-simple root. Original partition indices instead use the adjoint fiber
in fundamental-coweight coordinates. Prove the exact failure with a focused
unchanged-runtime regression before changing the map or tiebreak logic.
Focused E6 probe3833407 is COMPLETE in
/public/home/majj/atlas-math-e6-coordinate-probe-20260928.eoSKciRl.
It adds ONLY a real_form_order unit to3832345 runtime; no E6 repair yet.
The explicit diagram permutation fixes coordinates1/3; its fundamental fiber
can be checked without the full Weyl enumeration. The unit EXECUTES and fails:
basis e1/e3 is correct, but e3's first flipped imaginary-subsystem root is
[0,0,1,1,1,0], not datum-simple. Report SHA
603606c432068d75dc0bc2d16fb70f854bc1d0d08ad0d8f47fc2a9eb50739dc6;
log SHAeb481eb2e2e271cba3a13018c4c69b8a331de4b43c34de79bf901bd6fed4ed5c.
NEXT E6 repair: verify ambient unit-vector basis in increasing fixed-generator
order directly, retaining the partition specialGrading election. Keep this
same regression and require full original-backed KGB/KL differential after.

LATEST: FPP build3832301 COMPLETE, before exact-root test fails D4 and after
passes ten types/both numberings. Independent review3832333 is COMPLETE:
8mathmatches,8rejections,1Rust E7 timeout at300s. D4 wall is correct; E6
now matches in19.012s (original0.282s). Do not claim all FPP passes yet.
Build SHA3d0efc1f9fa119e3659a82131831224b35b4e7d0c7b2fbc4dcffc3f57a6b29cd;
review SHA50ab1506a23e9708eb17c2c970522d713a8f5df4ee4beb2f4229b7e4754b085b.
The exact FPP source core SHA is8a52ffb1..., NOT the current local core,
which now also contains the unverified partial-KL candidate.

R10/R11 now107cases. Indices103/104 A2/G2 partial/full/partial history,
105B2 containment,106compact A1 rejection. R10 review3832312 preserves
three fixture failures (bare [Param]/KL-tuple equality needs script overloads)
and the genuine compact A1 acceptance bug. R11 uses elementwise builtin
comparisons. Preflight3832334 and review3832340 are COMPLETE: original passes
all4; baseline Rust fails all4. Review SHA8995ea83b4de4c1f927b3e92fcd0236ce28dd792612874e68f7ba8420bd4a97b.

Partial-KL candidate stage:
/public/home/majj/atlas-math-partial-kl-repair-20260928.SZRaJr6Y.
Preflight3832344 COMPLETE (38checks); build3832345 COMPLETE in5:54.
Dependent array3832350 indices75-82,89-91,103-106, afterok3832345.
Do NOT duplicate. Build and latest original are freshly compiled in isolation.
Input manifest c293daeb15ad0a6dce4bdffd4223a339b39421520ff6048e374f3f7bc6578781;
review-code SHA594c8d56ff32ac37b0b034cbae15bd31a1fbfc0b6541e76594a74b88c3a7d5f0.
Build SHA6d7083abde0026076d8f872d6bd1c3e28e80d3373890a2210f243df099f32431;
Independent review3832517 is COMPLETE, SHA
65517a73057b4244c1a572f9a8ed7ebc671cd6c55fc6aec25c0dde2efb6616dd:
11mathmatches,1rejection match,2Rust failures,1original failure. All array jobs
are terminal; do not rerun unchanged. A2/B2/C2/D4/G2 core results, A2/G2
focused regressions and all4 new history/containment/rejection cases pass.
F4 now reaches a deeper panic in kl_table.rs:265 (`cross of extremal`);
E7 also hits that Rust panic, while original E7 fails allocation under6GiB.
E6 remains the pre-existing real-form constructor error. This partial-KL
candidate is NOT fully accepted and remains uncommitted.
Frozen core SHA4547f83fa036ef463ac8a0e732c0055b6c47099d550b3b90fe701b7aed423592.
Only core differs from main: FPP repair/test, replacement partial-KL arm,
removal of its obsolete classic finals helper. No unrelated code changed.
No acceptance/commit of the runtime changes yet. Patch/archive pins are in
results/math-partial-kl-repair-20260928 and the submission receipt.

F4 KL boundary repair is INCOMPLETE. Unchanged-runtime probe3832534
executes the new core regression and identifies x263,y278,s2,RealTypeII at
unit scale, then reproduces the panic. Report SHA
14d877bb00d5c4b4e4d25c7d9dacf7c58105c63ade97b905ea357c0390b7f616.
The real-II cross may legitimately leave the interval, as original blocks.cpp
states; its KL_pol(UndefBlock,sy) term is zero. Current kl_table.rs moves cross
lookup into only the branches needing it and zeroes only the absent real-II
subtraction term, retaining required complex/Cayley links. Runtime NOT accepted.
Core SHA now d7dcd40b43c53cde6a1aef86c5be19b867b99129ff6fcc2774d424d76bb6891c
(adds only the F4 unit to the3832345 core); KL SHA
13346677b299721bb5b623999b9f5c0bd51e6a42c0abc967b6fda2e319573ee2.
Repair stage /public/home/majj/atlas-math-kl-boundary-repair-20260928.qacZjoD2,
preflight3832612 PASS(41checks), isolated build3832615 is TERMINAL/FAILED4:24.
Before unit executes the cross panic. After unit completes the integer-scale
partial KL call/parameter comparison, then half-scale lookup fails integral
image positivity. No after-pass, no release build, no KL-table/FPP unit suite
from this job. Build SHA
1f9d64d78c695a1a292f9d1e1c31b1fe5103569aa351ccfcad2aaa352336409d,
before/after log SHA9d2ca542... /a42b3e8d... . Array3832623 was observed
PENDING/DependencyNeverSatisfied and then CANCELLED; no interpreter
executions and no independent differential review. Preserve the failed build.
Original source/scripts remain unchanged. Full-output differential still
required after BOTH issues are addressed (indices75-82,89-91,103-107).

Catalog108cases:107 adds F4 half/zero/unit partial/full/partial history.
R12 preflight3832551 passed39checks, but execution3832552 omitted
SUITE_INPUTS_SHA256 and did not run either interpreter. Review3832594 then
failed on the incomplete report. Preserve the harness-failure artifact;
it is not a mathematical observation. Identical-input corrected R12b stage
/public/home/majj/atlas-math-suite-r12b-20260928.Z4w6C7c8, execution3832609,
and review3832614 are COMPLETE. Review SHA
89813b870cae2d89336598b6d0eb7df22cb9eca48b02a86fe8b304ddee16ffbf:
original passes the full history, Rust fails first cold half-scale call with
`Weyl-element integral image positivity invariant was violated`. Raw Rust
stdout has no PARAMETER/COLD/FULL records (131bytes), so this is BEFORE the
full-lookup cache enlargement. Always export the manifest pin to every
execution job, not just preflight. Receipts identify the exact review.

NEXT additional root cause (static source evidence; not fixed): locator.rs
`additive_closure` explicitly implements ROOT sums via combine_roots(...,false),
whose bool means SUBTRACT, not root/coroot. Original rootdata.h:131 declares
`template<bool for_coroots=true>`; InnerClass::int_item uses that default, and
rootdata.cpp:690-704 closes COROOT sums. The Rust doc incorrectly says
additive_closure<false>. This matters for B/C/F/G, unlike A/D/E. A minimal
independent regression: in the existing standard B2 root datum, close the
roots [1,0] and [1,2] as coroots. Coroots [2,-1] and [0,1] add to [2,0],
the coroot of [1,1]; closure must recover all8 signed roots, while root sums
retain only4. Also test F4 gamma=[1,1,1,1]/2 in fundamental-weight coordinates:
mapped canonical positive roots must equal the exact set where gamma evaluates
integrally on coroots. Keep case107's full original-backed failure. Do not just
drop the positivity guard or flip combine_roots's subtract flag. locator.rs now
adds ONLY those two focused tests, no runtime repair yet; SHA
48faf441c52b37585cc13b946d463a34aefd38030c6ff507fe2472d2619e8b5d.
Unchanged-runtime coroot-probe stage
/public/home/majj/atlas-math-coroot-probe-20260928.ckgeagE9, job3832726 COMPLETE42s;
consult math_coroot_probe_submission_2026_09_28.json for the actual job before
starting more work. It uses the original3832345 candidate runtime, not the
KL-only repair, with only the locator unit tests added. The probe verifies
that source distinction. Both tests EXECUTED and FAILED: B2 closure gives4
instead of8 roots; F4 hits integral-image positivity. Report SHA
88aaa653d427f1705f36ef992a280a5db52acc3a662d2b4c329196cd2e455e43,
log SHAac43be69bfcff8c1d8c140d6c5f5684c979d83db726fcb6d32072fffb188a467.
No active jobs remain from this wave. NEXT: repair locator's coroot additive
closure with checked coroot-coordinate sums (a borrowed coroot-to-RootId map
avoids rescanning all roots). Preserve the same two tests and positivity guard,
then prove after-pass on HPC and rebuild the combined KL+locator candidate.
Do not weaken the core F4 unit that now exposes both integer and half-scale
failures. Full differential, including index107, remains mandatory.
The source archive SHA is4304a7fc9f39dea728a57cfc3efdda328efea801d0ee905b6d4ff8711ba79fdb;
probe pin SHA283d278151fd05ffa9118a91354d985057cd7d5fca0e6b76f2b14061d765d781.

Separate E7 FPP600s stage (same FPP-only build, not the partial-KL candidate):
/public/home/majj/atlas-math-fpp-e7-long-20260928.FEh5ubAb.
Preflight3832370 COMPLETE (38checks); array3832501 case68 is TERMINAL,
Rust timeout600.2182s/2588124KiB, original success24.1906s/981172KiB.
Review3832506 COMPLETE, SHA
4b405a26cf01655d6ac79bdfbcbf5928812099fb325d7ec6aa826e7d74ec31db,
retained in math_fpp_e7_long_review_2026_09_28.json. This is not full FPP
acceptance or a completed speed ratio; do not repeat unchanged at a longer
timeout. Profile/isolate the actual FPP stages before the next candidate.
Input SHA175dc2c9578a51db21b274dd0a473a3272fa9797c27a52b149355aa218399f12,
review-code SHA91326c438d07484062ee093d70d38953d893cb9da107decc1910df379acde7a4.
Its batch supports an explicit timeout; older frozen stages remain unchanged.
See the submission receipt for the exact build/limits; both jobs are terminal.

R9 catalog103cases: D6/D8 KGB at93/94, eight nontrivial Hodge probes at95-102.
Preflight3832208 passes36tests. Hodge array3832211/review3832222 are complete:
all8 original cases fail (A2 internal negative branch bound, G2 owner mismatch)
at half/zero scale and bounds4/20; Rust independently fails generic loading.
Repeated benchmark array3832212 is source-bound to current main. Actual jobs
E7=3832215, D6=3832216, D8=3832212; independent review3832228 is COMPLETE.
E7/D6 pass4 fresh-process, full-output-stable pairs. Median original/Rust:
E7 1.35766/69.4160s, D6 0.150402/0.822140s. D8 original completes3.757s;
Rust fails the4000000 enumeration cap; no ratio. All R9 jobs are terminal.
Stage /public/home/majj/atlas-math-suite-r9-20260928.L9clrgy7.
See math_suite_r9_submissions_2026_09_28.json; do not duplicate live jobs.

FPP repair candidate is IN PROGRESS, NOT ACCEPTED. Local core changes add an
exact root-reflection identity unit test over A2/B2/C2/D4/D6/D8/G2/F4/E6/E7,
both root numberings, and reverse the conjugating suffix of reflection_word.
The old/new archives are frozen from main05625 with only this core file
overlaid. The main baseline, original scripts and installed defaults remain
untouched. Before-unit must fail an executed assertion (not compile), after-unit
must pass, then the candidate must reproduce full FPP oracle output. Record
the before/after build and differential results before claiming acceptance.
Exact repair stage: /public/home/majj/atlas-math-fpp-repair-20260928.rraUQUz4.
Preflight3832250 passes36checker tests. Build3832260 is now TERMINAL/FAILED:
the before test compiled and failed the intended D4 assertion, but the after
command reused the old test binary from the shared Cargo target (0.7s, no
compile, same unreversed word), despite correct after-source hashes. Its result
is NOT evidence against the mathematical repair. Preserve the frozen stage.
Replacement stage /public/home/majj/atlas-math-fpp-repair-r2-20260928.49AOGABt
keeps identical source archives and isolates target-before/target-after.
Preflight3832300 completed; build3832301 is IN PROGRESS, not accepted.
Collect this exact job; do not duplicate. Input manifest
cc68314c624aedb60a99a2ebdbf2b7b4740adfd8b310677b9f39460b3de3cbd4;
review-code SHA d2f939fd84c581681057ef01d998d392320bae375108689f0eec67b39e9c5e40.
After successful build, run full FPP indices5,14,23,32,41,50,59,68,89 and
rank rejection indices8,17,26,35,44,53,62,71 in that frozen stage, then an
independent review pinned to the new build SHA. No runtime acceptance yet.
Source-only tar archives were generated on
HPC from the existing immutable baseline plus checksummed small patches.
Core SHA before9973a807... /after8a52ffb1... agrees exactly with local files.
The slow local full-tar upload was explicitly terminated; its partial file is
retained as abandoned-before-upload.partial. HPC GitHub connectivity was
verified with git ls-remote; origin/main remains05625c5d.
See AGENTS working convention6: prefer HPC fetch/pinned commits or small patches.

Next partial-KL repair can reuse the already-correct parameter plumbing of
`partial_block` and condensation/polynomial interning of `KL_block`; see the
new detailed source checklist in `slices/math_failures_2026-09-28.md`. Do not
replace actual-parameter lookup with a fresh dual-quasisplit block or merely
patch the expected counts. Preserve cache-history and singular cases.

NEW R7: broad array3832011 and independent review3832103 are COMPLETE.
The54-case review SHA is31c0e62d1a6b9b3227cfc2b55fc5ea213a29bb75f82a5f5cb28f1986feea1250,
stored in math_suite_broad_review_2026_09_28.json. Counts14mathmatches,
6rejections,18Rust failures,4oracle failures,9oracle timeouts,2Rust timeouts,
1mathmismatch. See MATH_VALIDATION for operation-by-group coverage, not a
blanket pass percentage. E7 KGB matches fully but single-shot times are
1.3032s original /69.8035s Rust; peakRSS41672/1459140KiB. E6/E7 FPP original
finishes0.2799/24.4600s; Rust hits300s. E6 KGB fails construction. Preserve
these cases as regression evidence. Do not infer stable ratios from one run.

R7 appends product cycle numerical multiplicities at index92 (93cases total).
It checks30 reconstructions, m=0/1/2 and bounds4/6, with full Phi sign binding,
exact rational kernel completeness and independent tensor expectations.
Latest scripts and both baseline binaries remain unchanged. No general cycle,
exceptional multiplicity, cutoff-completeness or representation-valued claim.
R7 preflight3832190 passed31checker tests; execution3832193 and independent
review3832194 confirm an old-fixture API mismatch (any on Maybe<vec>).
All R7 artifacts remain frozen. R8 uses succeeds for vector/solve results,
as in latest scripts. Stage /public/home/majj/atlas-math-suite-r8-20260928.raDj5B8V,
preflight3832199 passed31tests; execution3832200 and independent review3832201
are terminal/reviewed. Original passes all30 scoped reconstructions and complete
rational kernels (N4: Q14x41, rank14, kernel27; N6: Q22x97, rank22, kernel75).
All8 Phi identities bind the orbit signs; tensor multiplicities1/2/3 agree.
Rust fails latest basic.at before computing. Review SHA
30924499bb053599239cfc9095dfa666f0e3d3bd4d20b242be8ba84dad414656.
All known R2-R8 jobs are now terminal; do not rerun unchanged cases.
Next mandatory scope: exceptional/general cycle multiplicities and cutoff
justification, broader forms/isogenies and basic operations, nontrivial Hodge,
and correctness-accepted repeated minute-scale D6/D8/E7 measurements.

Earlier R6 checkpoint: the catalog had92cases. R4 probe array3832118 and independent
review3832122 are terminal/reviewed; preflight3832117 passed19tests. All six
Hodge probes (split A2/G2 bounds4/20; complex A2/G2 bounds15/50) still fail
in the original. Trace-backed causes are recorded in
`docs/slices/math_failures_2026-09-28.md`. Do not edit the latest scripts.
R4 core KLV initially failed on the script-only infinitesimal_character alias;
R5 replaces that helper with `%Param` destructuring, keeping R4 frozen.

R5 preflight3832134 passes20tests. Array3832137, indices75-82,89, and
independent review3832147 are complete and verified. A2/G2 core KL
outputs show original4/10 versus Rust1 parameters (half-scale2versus1).
Raw D4 FPP regression3832137 confirms negative integral-simple images;
the original succeeds. Three focused failing regressions now exist, not
just a blanket note: fpp_d4_wall.atlas and A2/G2 partial_kl_regression.atlas.
R5 counts:6complete core-KLV mismatches (A2/B2/C2/D4/G2/F4),2Rust failures
(E6 real-form constructor and D4 FPP invariant),1original failure (E7
std::bad_alloc under6GiB AS cap). No speed ratios from failures.
No runtime fix, alternate script or changed baseline has been used.
R5 stage `/public/home/majj/atlas-math-suite-r5-20260928.XoUEs2LH`;
R6 stage `/public/home/majj/atlas-math-suite-r6-20260928.wmMKYzQR` adds the
focused A2/G2 expected-size regressions at90/91. Preflight3832155 passes21tests;
array3832163 and independent review3832167 are terminal: both original cases
pass, both Rust cases fail the expected predecessor-count assertion. These
are verified failing regressions, NOT fixed bugs; do not rerun unchanged.
Full pins are in
`math_suite_probe_submissions_2026_09_28.json`. R3 broad3832011 is now reviewed.

These findings supersede any inference that latest-main failures are only
language-level. The older optimized branches and old-original measurements
still describe distinct builds. See MATH_VALIDATION and LANGUAGE warnings.

User objective: pull, update the original, establish a mathematical test AND
benchmark library spanning classical/exceptional groups, basic/KGB, unitarity,
Hodge filtration, FPP, associated cycles and annihilator varieties. **All testing
must run on HPC** (latest user instruction); local work is inspection/edit/sync.
Do not narrow this to performance or the currently passing subset.

Working branch `codex/math-benchmark-suite`, based on remote main `05625c5d`.
`git pull --ff-only` was already up to date; direct origin/main check agrees.
Remote upstream master checked directly at `7e1b958c7aa9456769cc9cf09ac1542814b4800a`.
`tests/math/baseline.json` pins both full source archives. The shared HPC
`/public/home/majj/atlas-rust` is a DIFFERENT dirty development checkout; do not
overwrite, pull, build from or silently substitute its mutable working tree.
The older optimized/PGO worktrees are also separate candidates, not main.

Library: `tests/math/catalog.json`, nine templates per group, 72 initial cases
across A2/B2/C2/D4/G2/F4/E6/E7. Root-data/KGB/FPP core calls are separated from
latest-script loading. High-level Hodge/unitarity/AV use the unmodified latest
scripts. Raw complete output, status, wall seconds and peak RSS are retained.
Three additional language prerequisites bring the catalog to75; all original
domain indices are preserved. Generic-pair, any_type and basic.at-loading cases
are diagnostics, not additional mathematical group coverage.
Cycle-foundation matrices are explicitly NOT associated-cycle multiplicities;
general cycles, broader Hodge bounds/parameters, isogenies/real forms, independent
identities and minute-scale D6/D8/E7 benchmarks remain mandatory open coverage.
No E8/fat resource unlock. See `tests/math/README.md` for acceptance rules.

HPC handles (do not duplicate or mutate frozen inputs):
- Build **3831844**, COMPLETED0:0 in8:58 on cu008. Stage
  `/public/home/majj/atlas-math-baseline-20260928.iMXyKqJ5`;
  report `results/3831844/build.json`, SHA
  `167cd7e783ee07b617c3a7ee027158343c2fecd2d90074f8ff5b0ab4c37b57cb`.
  Latest oracle + remote-main Rust,
  CPU2/8G/55min, compiler invocations and full source/script/binary pins recorded.
- R2 harness preflight **3831893**, COMPLETED0:0,4s/cu002: all **9** verifier
  tests pass. Report SHA `573f9d7b8b3391e414e13c2ea88cba655e3ebe45aa2ca3e67d953011375e451a`.
  This is framework correctness only, not Atlas/math acceptance.
- Initial differential pilot **3831897**, all18 A2/G2 cases terminal. Stage
  `/public/home/majj/atlas-math-suite-r2-20260928.rjGJpUN8`.
  Independent review3831996 rehashed both full sources (606/1212files),
  264scripts, binaries and all raw streams. Two root-data matches, two intended
  rejections, eight Rust script-loading failures, six original/fixture failures.
- R3 stage `/public/home/majj/atlas-math-suite-r3-20260928.CneUcC69`:
  preflight3831995 passes17checker tests; corrective array3831999 terminal;
  independent review3832014 confirms four A2/G2 KGB/FPP matches and three
  Rust generic-language failures. Corrected fixtures use builtin `0-mat` and
  an explicit all-ones vector instead of unloaded script helpers. Original
  R2 inputs remain immutable; latest upstream scripts are unchanged.
- Broader array3832011, `9-35,45-71%2`, covers54cases in B2/C2/D4/F4/E6/E7.
  All terminal, CPU2/8G/25min,300s/engine,child AS6G.
  Independent review3832103 COMPLETED0:0 and accepted all artifact hashes;
  it did not accept all mathematics. Do not resubmit these unchanged jobs.

Submission metadata: `tests/reference/hpc/math_suite_submission_2026_09_28.json`.
Verified reviews and new receipts are in `tests/reference/hpc/math_suite_*`;
current conclusions are in `docs/MATH_VALIDATION.md`. Next: collect the wider
survey/review, investigate Hodge original failures, extend actual associated
cycle multiplicity and nontrivial Hodge coverage, and retain every discovered
Rust calculation error as a regression (AGENTS hard rule7). Never infer
performance from failed math or a single short run. Goal remains ACTIVE.

Important source finding: old oracle `4d3e9449` -> latest `7e1b958c` changes257
files (20078insertions/13071deletions). Latest basic.at uses `Pair<S,T>` and
`any_type`; pulled main lacks their grammar and explicitly lexes any_type as
an identifier. Independent HPC reviews3831996/3832014 now confirm this major
upstream-compatibility gap. Do not rewrite upstream scripts to hide it.

Infrastructure repair:3831840/3831841 failed before test/build because sbatch
`--chdir` does NOT change `SLURM_SUBMIT_DIR`. Verified sacct/logs showed terminal
failure; corrected submissions first `cd` to the exact stage. No runtime change.
Old preflight3831843 passed8tests; R2 adds strict rejection validation and core
isolation. Preserve both histories. The abandoned slow source rsync was explicitly
terminated; archives were generated from immutable Git objects on HPC instead,
with SHA equality to local archives. No existing source checkout was edited.

This is the continuation record for `/Users/hoxide/mycodes/atlas-rust`.
The goal is source-compatible Atlas language behavior, with the upstream Atlas
executable and CWEB sources as the behavior oracle. The core remains safe Rust.

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
  `lambda_unique`. CORRECTION2026-09-29: the claimed match to
  arithmetic::divide was false; its signed negative-odd quotient is
  Euclidean. Original3840093/Rust3840098 expose A2 wrong canonical keys
  and coefficient coalescing; see the K-type slice and R2candidate3840172.
  Historical selected tests reported A2, E6, D5 passing; E7
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
# Latest continuation — finalized polynomial544, generic recursion next

FINAL recursive552 job3844656: report
672fd67b23067e3b42da4735995938b11a191bc85331161e18b8767c7e03e3fe,
552core/CLI/final1459source integrity/245capture;114whole positives/no losses.
ALL FOUR high-level libraries now load successfully (empty stderr), but full
stdout differs; do not label loading as mathematical or full-stream equality.
Exact552standard release/all116consumers are the next math gate. The same
binary251boundary capture3844760 is submitted in
/public/home/majj/atlas-recursive-group-followup-20260929.u6vSUOLP,
pin086c9c47a35f1f5ade9f767b892d4df8a63f0b0e5b85cc17d904416a7cef2f1c,
afterany3844656 (driver still requires full build/unit/source gates). Retain
the new grammar/parenthesis defects before R2; do not rewrite original goldens.

Original251discovery3844689 FINAL; report SHA
b8ee76d96b3dc6c1299f5bdf7fa808e9b3ceb7986d0bbff88a375763acd0a37f.
ENTIRE boundary positive accepted, full golden retained. Direct group-local
RHSs ALL Syntax-rejected (disproves the proposed runtime/assert path);
nongroup bare constructors also Syntax-rejected. Source-backed R1 defect:
anonymous tuple function arguments must retain parentheses, unlike R1's
provisional kernel expectation. See slice/AGENTS; a same552binary251capture
is being prepared to preserve actual before differences before R2 repair.

Latest3844656 logs: SAME3after regressions PASS; graph5controls PASS;
all552core PASS (0failed/ignored/filtered,46.91s). CLI/release/245capture and
final report remain open. Before0pass/3fail proved before runtime change.

3844656 has executed before0pass/3fail; after compile underway. New master251
discovery3844689 is separately frozen in
/public/home/majj/atlas-recursive-boundaries-reference-20260929.2uKQ2ieV,
probe65839e82d3f58b5284b38033b19725f9a057732f33b0f51ed53ec894329c799e.
Keep552candidate's245inputs untouched. Collect full original boundary output,
especially non-group bare constructor errors and direct-local-RHS signals,
before promoting those provisional fixtures to regression goldens.

Generic group candidate552 submitted as3844656 at
/public/home/majj/atlas-recursive-group-build-20260929.viHsFDuL,
pin261fc145bfb6ae2ebf022d0dd91919135a8a784864dfda693b21fbfc6f1a98ac.
Three same before/after session regressions, five graph controls,552core/CLI
and245full captures required. No after result yet. Source snapshots/patch
generation scratch: /tmp/atlas-recursive-group-build.ESfmC2u1; frozen stage is
authoritative, not subsequent local work. See generic_recursive_groups slice
for direct-RHS and non-group bare-constructor edge discovery requirements.

HPC3844249 FINAL report math_polynomial_write_build_2026_09_29.json SHA
6356f7bf478687dfbacfdfa9bef6efa3cb30886f45509c1f29445432e0d97749.
All five before failures/after passes,544core/CLI/source integrity,243capture
with110whole positives/no losses. Followup3844321 FINAL report SHA
f20d570c1d6b32ed6d19e2fb4ba5a76df6b4e3e36a376db7ae69a83da3524d29
retains111whole positives in245cases on exactly the same binary. Complete
local/captured/user-op fixture passes; all six negative messages and survivor
stdout pass, but diagnostic envelope/stderr differences remain explicit.
Four latest-library imports now share lazy_lists.at5 generic recursion as
their first error. Next work is graph-wide recursive type groups, not a
grammar-only acceptance. Three original-backed session regressions have been
added before implementation; see generic_recursive_groups slice. No full116
or new parallel A/B acceptance; old536consumer jobs remain source-distinct.
