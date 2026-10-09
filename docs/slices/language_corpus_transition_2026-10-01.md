# Latest-original language corpus transition — 2026-10-01

## Scope and current boundary

The historical language corpus is not a blanket claim against current Atlas
master. There are 353 `.atlas` fixtures, but only 346 executable
fixture/event/metadata triplets. Seven `relations_*_probe` fixtures have no
event contract, and `tests/reference/eval/scalar_errors.meta.json` is one
aggregate record rather than a case. All 346 per-case metadata records bind
legacy original revision `4d3e9449062a07c1c85f4e6df215eb6ccc0eeae9`.

The frozen historical boundary is:

- directory counts: commands19, domain233, eval89, lex1, negative1, parse3;
- outcomes: 200 exit-zero and146 exit-one;
- event schemas:344 `atlas-eval-events-v0` and2
  `atlas-session-events-v0`;
- 6739 events:3737 Value,2161 ReportLine,362 Output,479 Diagnostic;
- 199 cases with both old stdout/stderr hashes and147 with neither;
- 264 cases with a legacy binary hash and82 without one;
- 339 cases with a reference job and7 without one;
- two successful cases intentionally containing diagnostics:
  `eval/fromfile_b10` and `negative/unterminated_string`.

Missing legacy raw hashes remain unavailable. Never infer byte equality, copy
a binary/job pin from another field, or rewrite old metadata to current master.

## Prepared input manifest

`hpc/language_corpus_manifest.py` builds
`atlas-language-corpus-manifest-v1`. Its only valid status is
`FROZEN_INPUT_INVENTORY_ONLY` with `compatibility_claim=false`. The module and
its synthetic/production contract tests are currently:

- manifest module SHA-256
  `48368e2acdc92fb03d41a37f8277ad279b2cfed0fed7c44486e40c3c98791877`;
- test module SHA-256
  `3cfa0f96a98c9df89b0bec27cfc1f606f14e4b16e5888fe67f9ec475ea3d97b5`.

It performs two complete independent snapshots and requires canonical
equality, rejects duplicate JSON keys, symlinks, unknown categories and
inventory/count drift, and binds each fixture/event/meta file by name, bytes
and SHA-256. It records the exact `append-quit-v1` effective input, legacy
provenance availability, event counts, dependencies, timeout policy and
filesystem isolation contract. It parses the old pipeline only as a static
inventory:337 plans,336 without a pending marker, one explicit partial and
nine event-backed omissions. It deliberately sets
`selector_semantics_validated=false` and does not execute or bless that runner.

No manifest JSON has been generated and neither module has run on an HPC
compute node. Two independent static reviews found no P0/P1 issue. The helper
must stay outside the frozen parent-seal transport and disabled until a
campaign-bound compute stage exists.

## Filesystem effects and isolation

An anchored scan finds exactly nine file-directive lines in four cases. The
remaining342 cases are ordinary no-write cases; three sensitive cases are also
no-write, so the complete boundary is345 no-write cases and one writer.

| Case | Required contract |
|---|---|
| `eval/fromfile_accepted_b10` | In one process, map helper SHA `be35fe524df41f56a0e1e0fd330757d385f4d7203cd2b4caa6269a6d93860ec0` read-only at the exact historical `/public/home/majj/atlas-rust/tests/fixtures/eval/include_b10_helper.at`; zero writable-tree delta. |
| `eval/fromfile_b10` | Keep `no_such_dir_b10/missing.at` absent before and after in the engine cwd and every configured include root; zero delta. |
| `eval/file_commands_b9` | Give each engine arm a fresh private `/tmp`; allow only creation of regular, non-symlink `/tmp/atlas_fc_b9.out` with exact bytes `Value: 3\nValue: 12\n`; reject every other delta. |
| `eval/file_commands_b9_rejected` | Use a fresh empty private cwd; require `x`, `no_such_dir_fc_b9`, and its `f.out` child absent before and after; zero delta. |

Execution must use the exact frozen fixture bytes. Do not rewrite the embedded
absolute path. A pinned namespace backend must provide fresh private writable
roots, read-only binaries/scripts/source, no network and complete pre/post
inventory. If the compute node cannot provide that boundary, fail with an
isolation-blocked result; there is no unsandboxed fallback.

## Provenance prerequisites

The current original pin is
`7e1b958c7aa9456769cc9cf09ac1542814b4800a`, with known oracle binary digest
`d4f0f3dc3a82102529aa2ec562db0601e99b368dee25d5539ca52dae2fd37a5a`
and264-script manifest digest
`62c20ed8d6ba83d09d2d7bbaa05091a034fcb95611cafadad15d3fc6bc2ee182`.
These are descriptor evidence today, not locally available replay CAS bytes.
The parent seal has not run. Ladder BEFORE, the production root-ladder repair,
ladder AFTER, its independent review and its accepted Rust source/binary refs
also do not exist yet. Therefore the346-case transition is not runnable.

UPDATE 2026-10-10: those prerequisites now exist — parent seal job3872554 and
ladder AFTER-v3 job3875239 are both FINAL and independently accepted
(BEFORE-v3 job3873400 and the repair chain precede them; the accepted
source/binary refs are recorded in the ladder entries).  The transition is
still not runnable today: it waits behind the G2-v1 capture and the HPC
tunnel, and the two-job sequential plan below still applies in full.

The execution stage must bind the accepted parent seal, accepted ladder-AFTER
Rust source and binary, explicit current-oracle build/source/scripts pins, the
exact manifest object, isolation backend/version/hash, harness bytes,
environment/resources and every raw CAS object. It may preserve the147 legacy
raw limitations; it may not describe them as byte matches. Full346-case legacy
byte equality would require separately sealing/replaying the old4d3 oracle.

## Sequential scheduler plan

Only after parent seal and ladder AFTER are FINAL and inspected:

1. Submit one non-array `fat`,2CPU/32GiB execution job under the existing
   campaign. Run each engine/case as a fresh process, alternate arm order,
   force Rust serial mode, use a1200-second heavy-case limit and retain full
   streams, exit/timeout/signal, CPU/wall/RSS and filesystem effects. Its only
   successful maturity is captured/unreviewed with no compatibility claim.
2. Inspect that report. Then, and only then, retire its launcher and submit one
   independent review job. The reviewer must rehash all inputs/raw objects and
   independently recompute event rendering, diagnostic/outcome classes,
   200/146,199/147 and side-effect decisions. It must not call execution-side
   classification functions.

Never prequeue the review, use an array/dependency, create another campaign
root, or count a capture report as acceptance. Even a reviewed finite-corpus
match is not a proof of general language or mathematical correctness.

## Retained limitations

The manifest builder's remaining P2 boundary assumes the controlled workflow:
it does not defend against a hostile same-UID process swapping a whole real
repository tree consistently across both snapshots; its compute-only check is
policy, not a scheduler security boundary; and publication still needs an
active-campaign/ledger/job-result wrapper with final inode/hash checks. Its
directive recognizer is deliberately exact for the current corpus, not a
generic replacement for the Atlas lexer. New file-command grammar forms must
extend both scanner and tests before a new manifest version is frozen.
