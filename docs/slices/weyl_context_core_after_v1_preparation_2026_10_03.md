# Weyl context-core AFTER-v1 gate preparation and freeze — 2026-10-03

Status: gate harness frozen, committed (`342a0511`, `f838de69` on
`origin/codex/math-benchmark-suite`) and pushed.  HPC submission is
**blocked**: the SecureLink tunnel (`tun0`) is down (client processes alive
but disconnected; `tailscale0` logged out; `majj@10.26.14.64:22` times out
over the ordinary gateway).  No remote stage, intent, ledger record or job
was created; no uncertain remote state exists.  The exact launch procedure
is in `docs/HANDOFF.md` ("CURRENT: AFTER-v1 gate frozen locally …").

## What the gate is

The changed-input AFTER successor of tests-first BEFORE-v4 job 3886748
(FINAL COMPLETED 0:0, evidence
`tests/reference/hpc/math_weyl_context_core_before_v4_2026_10_02.json`).
It applies the reviewed Weyl-owner repair patch
(`hpc/patches/weyl_context_core_repair.patch`, SHA-256 `246cd2d0…`,
29,430 bytes) on top of the regression-patched accepted source and requires
both known regressions to pass with the repaired Rust byte-matching all four
frozen original goldens (cold stdout `7a614e47…`/empty stderr, prewarmed
stdout `5074aab3…`/stderr `ef4404d8…`), the retained root-ladder control
passing, the 632-test atlas-core inventory otherwise unchanged, and all
ledger/integrity checks.  It releases no mathematical, cache, performance,
memory or rank claim; the repaired production files stay uncommitted until
the gate passes and is independently inspected.

Frozen identities:

- `hpc/stage_weyl_context_core_after.py` SHA-256
  `8cb4b86d9986e66c1722c87d5704ab92ee387ce426ff0f3c194b60c5320904d6`
  (`SUBMISSION_ENABLED = True`)
- `hpc/math_weyl_context_core_after.py` SHA-256
  `3d7d98903a838b3a3343482d165af4b7e9b3989cc1f30a5c9ccc49950700913b`
- `hpc/test_math_weyl_context_core_after.py` (28 tests)
- `hpc/test_stager_allowlist.py` (7 tests),
  `CURRENT_POLICY = "weyl-core-after-active"`; capture pair frozen and
  disabled (`SUBMISSION_ENABLED = False`)
- checker table 32+17+18+21+28+7 = 123; the regression-contract suite gained
  the four `classify_after` tests (17 → 21)
- regression source manifest (frozen before-v4 report, 1567 files)
  canonical SHA-256
  `55f807cadb712377cbf6c0c79250b1d5e739e9757290a24209a086f89632850f`;
  after applying `REPAIRED_SOURCE_HASHES` (domain_builtins `e6987e7c…`,
  typed `614975c5…`) the repaired manifest is exactly
  `3f8cf4753f29d33df8273086254a4f09170ada46f05e9c13acfdac2f5ab85c33`
  (verified offline)

## Transition defects found by local rehearsal (fixed before launch)

The default local checkout (umask 002, evidence files 0664) masks these
inside environment-sensitive tests; rerunning under `umask 022` and
replaying every validator against the real frozen evidence in a 0444 mirror
tree exposed them:

1. `progressive_submit._validate_predecessor_state_descriptor` missed the
   before-v3 scoped campaign creation files once the direct predecessor
   advanced to before-v4 — the real 37-file campaign state would have been
   rejected at stage creation.  Added `before_v3_scoped_campaign_files`.
2. `test_campaign_stage_creation.predecessor_fixture` lacked the three
   before-v4 campaign-scoped files; its ledger re-read assertion kept the
   old length (`== 19` → `== 20`).
3. `validate_before_v3_failure` still referenced bare
   `PREDECESSOR`/`PREDECESSOR_STATE`/`PREDECESSOR_STAGE` — correct in the
   before-v4 launcher (where PREDECESSOR was before-v3), never satisfiable
   in the after module (PREDECESSOR is before-v4).  The frozen before-v3
   identity now lives in transplanted `BEFORE_V3_PREDECESSOR*` constants;
   the validator rebinds the three names locally, keeping the body
   byte-identical to the HPC-verified original except the pinned
   predicted-counts constant, and its `successor_stage` check is the
   historical literal `"weyl-context-core-before-v4"`.
4. The before-v1/v2/v3 failure evidences froze `expected_test_counts_after`
   with the capture checker label and total 119; the before-v4 report froze
   checker total 119.  Validators now pin
   `BEFORE_FAILURE_PREDICTED_AFTER_COUNTS`/`BEFORE_V4_CHECKER_TOTAL`
   literals instead of reusing current mutable constants.  Historical
   bytes are never relabelled to match the successor.
5. The recovery test never patched `driver.validate_before_v4_result` or
   `driver.repaired_source_manifest`; both are now patched and asserted.

## Local verification state (no builds; reading/editing/hashing only)

Under `umask 022`: campaign-stage-creation 32/32, progressive-submit 17/17,
contract 18/18, regression-contract 21/21, after checker 27/28, allowlist
7/7.  The one after-checker error is the known environmental class: the
test reads repository evidence at checkout mode 0664 while the validator
requires the stage-installed 0444 (the HPC-verified capture checker shows
the same local-only behaviour).  All twelve evidence validators
(capture v1–v7, before v1–v3 failures, before-v4 result, prior creation
failure) were replayed directly against the real frozen evidence bytes in a
0444 mirror tree and accept them.

## Next

Reconnect SecureLink, then follow the four-step launch procedure in
`docs/HANDOFF.md`: reconcile (empty queue, 19-record ledger
`5381b3b3…`, stage path absent), materialize `342a0511` on the login node,
run the after stager once to create only
`stages/weyl-context-core-after-v1` and submit exactly one job with
`queue_before=[]`, and record job/pin/intent/receipt as
SUBMITTED_NOT_VERIFIED.  On FINAL, independently inspect the report and
write the AFTER acceptance record before any production commit of the
repair.
