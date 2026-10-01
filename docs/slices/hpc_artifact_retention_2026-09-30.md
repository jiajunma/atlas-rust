# HPC artifact retention and directory lifecycle — 2026-09-30

## Finding

The verification campaign isolated each experiment correctly but never closed
its lifecycle. At the audit point the HPC home contained317 top-level entries
whose names began `atlas-rust`, a subset of the1174 whose names began `atlas`;
these are overlapping counts, not additive sets. The local repository had25
Git worktrees, of which8 secondary worktrees were dirty. Two recent permanent
Cargo targets occupied about828MiB and416MiB, while each copied `hpc`+`tests`
input tree was only about2MiB. The primary waste is retained build output and
expanded sources, followed by unretired experimental worktrees—not fixtures.

The error had five causes:

1. “Frozen stage” was implemented as “permanent full filesystem snapshot.”
   Mathematical evidence requires immutable bytes and hashes, not a Cargo
   incremental cache.
2. Verifiers followed absolute paths into parent stages. That made disposable
   implementation state look like a durable dependency and created an
   ever-growing chain.
3. No stage had a declared retention class, self-contained dependency closure,
   or safe retirement condition.
4. Lifecycle enforcement was placed too late. Several launchers copied trees
   or wrote pins before the common submission helper rejected their top-level
   path, while older direct-`sbatch` helpers bypassed that boundary entirely.
   A guard that runs only at scheduler contact cannot prevent directory growth.
5. Parallel local experiments treated a Git worktree as a disposable
   concurrency slot, but task completion did not own the corresponding
   retirement step. Branches named for one-off agents and probes therefore
   survived after the experiment, including clean-but-unique commits and dirty
   evidence that cannot now be removed blindly.

The underlying reasoning error was local rather than mathematical: each new
gate optimized for immediate immutability and retry isolation without charging
the cost to the whole campaign. A fresh directory made one submission easy to
reason about, but repeated hundreds of times it duplicated source/build state
and converted absolute implementation paths into permanent dependencies. The
correct unit of isolation is an immutable pin plus content hashes; it is not a
new filesystem tree.

A follow-up live classification on `2026-10-01` resolves an apparent count
change: the HPC home currently has335 top-level `atlas-rust*` entries, but17
are regular `.bundle` files and318 are directories. The earlier317 count was
directories only; the sole additional directory is the one active campaign
root. No legacy top-level `atlas-rust*` directory has a modification time after
`2026-09-22`; all newer activity is contained below
`atlas-rust-campaign-20260930`. Thus stop-growth is working, but it does not
make the historical318-directory backlog safe to delete.

The same missing lifecycle also affected local temporary staging. A live
read-only inventory on `2026-10-01` found 141 top-level `/tmp/atlas*`
directories using 914324 KiB allocated (~893 MiB). The two largest account for
785448 KiB (~767 MiB): `atlas-upstream-math.37eKHYdJ` retains an extracted Git
checkout beside both source archives, and
`atlas-original-source-audit-7e1b958c` is another clean checkout at the pinned
original-Atlas revision. The other 139 paths are predominantly transport,
package, capture and result staging from earlier gates. They are not Git
worktrees and are separate from the 25-worktree count above. No deletion was
performed: apparent reconstructibility is not proof that no process, handoff
or unique unarchived evidence still uses a path.

New local temporary state is therefore turn-scoped and context-managed. It
must be removed before handoff. At most one exact active cross-turn transport
may survive, and only when `docs/HANDOFF.md` records its owner, manifest,
hashes and retirement event. A retry reuses or explicitly supersedes that
registered transport; it never creates an untracked suffix sibling. Existing
`/tmp/atlas*` paths are a frozen cleanup backlog and do not authorize further
growth.

## Required layout (parent-seal path accepted; generic metadata enforcement open)

All new work for the active campaign is below one
`/public/home/majj/atlas-rust-campaign-YYYYMMDD` root. Individual immutable
stages are children of `stages/`; the job cap ledger and lock live at the
campaign root. The date names when the campaign began; crossing midnight is
not a reason to create another root. Reuse the active campaign for retries and
the rest of the same optimization/correctness sequence.
A reconnect, retry or resubmission with identical pin/manifest bytes reuses
the same `stages/<logical-stage>` path. A timeout or uncertain scheduler reply
must not create a `-retry`, `-r2` or fresh stage. A new logical stage requires
changed input bytes plus a pre-recorded predecessor, changed digest/reason,
retention class and retirement condition.
For this campaign the accepted name is hard-pinned to
`atlas-rust-campaign-20260930`; a different date is rejected before staging or
scheduler access, so it cannot silently create an independent lock/ledger.
A future rollover is a reviewed lifecycle transition after sealing, not a
date-derived default.
`hpc/campaign_workspace.py` validates that layout and provides
an exact temporary job workspace. Expanded source, Cargo target and sampling
build trees are created there and removed on context exit. A hard-killed job
can leave at most one uniquely named hidden workspace inside its own result
folder (or node-local scratch), which is separately auditable.

The generic helpers currently enforce the root shape and shared ledger but do
not themselves validate predecessor, changed digest/reason, retention class
and retirement condition for an arbitrary new sibling stage. The sole active
stager carries that contract explicitly. Do not introduce another active
stager until the metadata contract is centralized and covered by an HPC
checker; prose alone is not sufficient enforcement.

Durable evidence:

- input fixture/catalog and exact original golden;
- patch plus pre/post source manifests and source revision;
- pin, submission record, report and report checksum;
- complete stdout/stderr and exit/timeout status for differential cases;
- wall time, CPU time, peak RSS, compiler/node/thread settings;
- command logs needed to prove the reported checks;
- only a checksummed source archive or binary explicitly consumed later.
- generic evidence blobs, source archives and seals share the campaign object
  store at `objects/sha256/<prefix>/<sha>`; references carry schema, role,
  digest and size, never their historical source path.

Disposable state:

- Cargo `target`, incremental objects and dependency builds;
- extracted or copied source trees reconstructible from a pin+patch;
- profiler build trees after reports and needed samples are sealed;
- duplicate `hpc`/`tests` ancestors only after every referenced byte is in the
  verified CAS, a path-independent seal reproduces the legacy result, and a
  separate reachability dry run marks the exact directory eligible.

## Migration status

Phase 1, **stop growth**, is implemented and accepted in the bounded
directory-governance scope of compute job3875239. `campaign_workspace.py`
groups new stages under one dated
campaign and removes expanded job source/target trees. `campaign_source.py`
stores the accepted Rust source once as a deterministic path-free archive;
`campaign_blob.py` stores reports, streams, binaries and the seal in the same
CAS. Progressive submission records are written atomically and share one
campaign ledger. The common progressive submission boundary now rejects every
non-campaign top-level stage before any scheduler query or submission; legacy
stagers remain readable evidence only. A follow-up static inventory found 29
older direct-`sbatch` entry points, including the former 408/292-task arrays;
all 29 now exit as their first `main()` action. Campaign CAS writes also reject
every durable `atlas*` root except the already-created active campaign.
Another 31 pre-campaign `submit_one` stagers were also found to perform copies
or pin writes before the common boundary rejected them; they now exit at the
first `main()` statement too. At the phase-1 freeze,
`stage_weyl_parent_seal.py` was the sole enabled HPC stager pending its FINAL
migration report. That historical 69-stager snapshot had one active parent,
one pending ladder successor and 67 retired stagers. Following accepted
job3872554, the parent launcher is closed and only the seal-only ladder
successor is enabled; all 68 other entries fail before old preparation or
submission work. The active boundary opens locks,
ledger, pin, intent, receipt, scripts and numeric result directories without
following final-component symlinks, rechecks stable file identity, and rejects
unsafe state before a scheduler call. Source/blob references have exact
path-free key sets; an extra legacy `path` field is not tolerated.

Durability caveat: job3875239 accepted these helpers and guards only in the
bounded infrastructure/directory-governance scope. The commit containing this
slice must include the exact accepted helper/guard bytes and be pushed before a
fresh clean checkout inherits them; the HPC result alone is not source-control
durability. Do not call the stop-growth fix permanent from an unpushed tree.

The post-seal source gate also makes the launcher allowlist executable, not
merely documentary. The superseding exact inventory is70 `stage_*.py` files:
the parent and67 ordinary historical stagers are closed, ladder BEFORE is
disabled, and ladder AFTER is the sole active stager. The durable AFTER stage
contains only parent/BEFORE/AFTER; it does not copy the67 historical launchers.
The compute driver instead materializes the accepted parent source in its
auto-cleaned workspace, overlays those three current transition sources in
memory, then validates the exact70-name set and immediate unconditional
retirement of all67 ordinary historical `main` functions. The resulting
command log and GNU-time record are hashed like every other gate command.

The accepted parent transition added `hpc/test_stager_allowlist.py` and
`hpc/test_math_acceptance_index.py` to the ladder compute preflight before
Cargo, patching or math work. The allowlist's
five checks must pin the filename/role set, immediate retired exit, canonical
entry guards, both ladder switches/early guards, module-level side-effect AST
allowlist, and malicious mutation rejection. It validates two distinct policy
snapshots: historical parent-only bytes and current ladder-active bytes where
parent and all retired launchers are closed. Historical AFTER v1 additionally
ran20 synthetic `local_worktree_guard` checks against the exact25-row registry;
its unittest total was88, followed by the separate full70-stager inventory
command. V1 and v2 are now immutable harness failures. The AFTER v3 candidate
has22 guard methods and95 checker methods in total. Its reviewed candidate
29-input manifest is
`tests/reference/hpc/math_ladder_boundary_after_v3_overrides_2026_10_01.json`;
at the pre-creation checkpoint no immutable pin existed. The acceptance checker
currently has24 methods and requires its index plus full-deform source, cycle
capture and cycle review JSON evidence. Treat the checker, its four evidence
files and the allowlist as child-only: exclude them from both
`SEALED_SHARED_INPUTS` maps, or the child will incorrectly demand that the
frozen parent manifest contain post-seal inputs. The reviewed transition
closed the parent launcher before enabling ladder. Do not add any of these
files to the already frozen lifecycle-v6 parent transport; doing so would
invalidate the manifest for its14 payload files and the64-checker review. The
accepted BEFORE v3 total remains65; historical AFTER v1's88 checks and v2's93
checks executed before their distinct harness failures.

The legacy verifier still names accepted reports, binaries, oracle scripts,
raw streams and command evidence by absolute paths. Consequently 3868782,
3868803, 3868832 and their reachable ancestors remain required in full. The
old prepared ladder BEFORE is not eligible for submission because it follows
that chain. Its absolute-parent contract was replaced by the seal-digest-only
`ladder-boundary-before-v2`; that immutable attempt consumed only the path-free
parent-seal object, uses dirfd-bounded immutable staging and binds the compute
job to the unique confirmed campaign ledger/intent before creating results.
Its custom submission record is now replaced locally with the shared pinned
`submit_one` schema plus existing-lock recovery, same-job partial-confirmation
repair and a final no-heal preflight. Its 65 checker tests and exact 521/630,
3-failure BEFORE gate were static-only at that snapshot. The activation
condition was satisfied by accepted job3872554, and the parent launcher is
closed. V2 later failed only its checker preflight as recorded below; at that
snapshot the corrected BEFORE v3 attempt was the current gate. It and bounded
AFTER v3 job3875239 are now independently accepted; publication in the
mathematical acceptance index remains a separate disposition.

Phase 2 has exactly one submitted compute attempt and is **accepted only in
its infrastructure-migration scope**.
After an empty-queue/accounting/exact-path reconciliation, the frozen stager
created only
`/public/home/majj/atlas-rust-campaign-20260930/stages/weyl-parent-seal-v1`
and submitted job3872554 with `queue_before=[]`. `sacct` records
`COMPLETED 0:0` on cu034, elapsed33m40s. The raw pin SHA is
`b52e91d827f39b7d290ab0447f82e251e979e46fdd440b61371fd5599973cf96`;
the campaign ledger, stage intent and receipt SHAs are
`97690b3555a0bc70050240a2499e7781b33b4f8e1c968356f861f24e10e1c706`,
`0c2a6b6b13404a7a4583c4f1f2c35c32d1ba07924a44b1cb12f4b05be83c5147`
and `6ced68fcf857ab4df3a5d12eb1f603e1a4257c18b4e23de11108dbb8bd7d6742`.
Do not create a sibling stage or resubmit this logical attempt. Its schema is
`atlas-weyl-context-parent-seal-v1` and its object role is
`weyl-parent-seal`. It runs the complete frozen legacy verifier twice under an
identical environment. The 341-file legacy harness and its external capture
pin are durable only as opaque CAS objects: the stager never installs a
runnable `legacy/hpc/stage_*.py` tree. The compute job expands that archive in
its auto-cleaned workspace, assigns stable logical keys to both temporary
harness reads and historical-home reads, records and hashes the complete open
set, stores the exact closure in CAS, rebuilds the
629/519 inventories from a
temporary source expansion, and independently cross-links fixed reports, raw
boundary streams, source, binaries, harness and scripts. The job is now FINAL
and independently inspected, so the ladder v2 stager and driver were reviewed
and activated against that exact seal. The ladder job must never read an old
stage path. Independent inspection recorded report SHA
`939482542c123a6a2a2c22f8dbe1e052af20dee3af4010da780fe958dc41bf79`,
seal object `db67234c0d67dbd6a6f0327386dd113b6093d25a46569710c33dfc8bc482cdcb`
and1558-file source object
`5133bb32da7e5a92775d5f56ea2035680c363b40f085a8af95eaeebfdbc4536b`.
All12 commands/64 checkers passed, both complete traces matched, inventories
were629/519, and the33143-file closure contained4355 unique objects. The
one-stage filesystem, hashes, submission state and removed ephemeral workspace
were checked; see `tests/reference/hpc/math_weyl_parent_seal_2026_10_01.json`.
This grants no mathematical, rank, repair, performance or parallel claim.

Bootstrap used a local package whose bytes had previously been frozen,
`/tmp/atlas-parent-seal-overrides.lifecycle-v6.tar` (225280 bytes, SHA
`8347c733555b1e54c9798a6006636beb6be22e2c03483c10172625f6875d9b6a`).
That tar and its unpacked `/tmp/atlas-parent-seal-overrides.lifecycle-v6/`
sibling were not registered with owner/retirement before use, violating the
new lifecycle procedure. Bootstrap provenance, lifecycle registration and
remote installation evidence were not pre-frozen; do not backdate them. The
local tar hash was checked; separately, the remote15-file allowlist and all
payload hashes were checked by the operator. Those remote observations are
not implied by the local package comparison and are not HPC acceptance. The
remote incoming directory was installed by same-filesystem rename and no
remote tar was retained. Both local paths are
`RETIREMENT_PENDING_EXPLICIT_AUTHORIZATION`. They are one active transport plus
one frozen legacy duplicate, not authority to create another path or to delete
either without the exact audit and user authorization.

The post-hardening lifecycle-v6 transport contains exactly14 payload files;
the tar contains15 regular files after adding `overrides.json`, plus two
directory entries and no symlinks.
Its `overrides.json` SHA-256 is
`0aac6e735c0ae7c7e0da4773c46e0d8f116b92b68a73797b05e886ea342a2620`;
the deterministic tar SHA-256 is
`8347c733555b1e54c9798a6006636beb6be22e2c03483c10172625f6875d9b6a`.
Each payload manifest entry was rehashed and compared byte-for-byte with the
current workspace. This proves local packaging integrity only; the separately
operator-observed remote allowlist/hash checks did not constitute HPC
acceptance. Separately, the64 checkers and two legacy traces completed in
job3872554, and its FINAL report plus independent inspection now accept only
the bounded parent-seal migration.

The raw migration-pin SHA is written to the campaign ledger and stage intent
before `sbatch`. If submission was accepted but the login stager exits before
finishing confirmed state or receipt publication, only that running compute
job's exact `SLURM_JOB_ID`, script and pin may complete the same attempt; it
never resubmits or creates a second stage. The final preflight is read-only and
rejects a state rollback. Two independent static reviews found no P0/P1
blocker. Residual P2: lock-file inodes are stable, but a hostile same-UID
process could replace a whole stage/campaign directory while pathname-based
state I/O is in the critical section. Controlled campaign procedures never
rename those directories; full hostile-concurrency hardening requires
dirfd-relative state reads and atomic writes. Concretely,
`campaign_source.py` and `campaign_blob.py` still repeat
`campaign.mkdir(parents=True, exist_ok=True)` after validating the active root;
a same-UID rename in that interval could recreate the top-level campaign name
and split state. Preserve lifecycle-v6 now; after its compute acceptance,
remove those root-creation calls and create only `objects/...` children through
the pinned campaign dirfd, then rerun the infrastructure gate.

## Safe migration and cleanup

Do not glob-delete historical `atlas*` directories. First produce a dry-run
inventory containing job state, unresolved submission intents, report/pin
references, retained artifact hashes and size by retention class. A directory
is eligible only if no live or uncertain job owns it and every referenced file
has been migrated to a content-addressed durable location whose hash and byte
size are independently verified. Keep reports immutable; future resolvers may
fall back from their historical absolute path to a hash-addressed artifact.

Local worktree retirement is separate. Archiving a dirty patch preserves
evidence but never makes that worktree deletion-eligible. Removal requires a
fully clean tracked/untracked/ignored-file audit, a named branch/commit kept
reachable from a protected ref, proof that no active task uses it, and
explicit user authorization for the exact path. Never remove a dirty worktree
or delete a unique commit as storage cleanup.

New parallel agents work in the shared primary tree with explicit,
non-overlapping file ownership. They must not create a worktree merely to gain
a concurrency slot. Creation is unconditionally frozen under the current
schema: every ACTIVE row is invalid and `precreate` fails before reading state.
A future transition must first implement and HPC-verify a durable receipt-bound
creator, then the primary coordinator may
request explicit user authorization for at most one ACTIVE task worktree
repository-wide, and only for a task that actually needs an independent
checkout.
Retries reuse that same directory, and task handoff must report tracked,
untracked and ignored state, commit reachability and whether the directory is
still in use.
Passing these checks establishes eligibility, not deletion authorization.

The prose rule is now backed by that strict host-local registry and the
read-only preflight `hpc/local_worktree_guard.py`. Its frozen inventory
is one PRIMARY,24 LEGACY and zero ACTIVE worktrees. PRIMARY records path and
branch but omits HEAD to avoid an impossible self-reference when the registry
commit advances the primary branch; every LEGACY entry pins HEAD exactly.
Normal `check` samples registry, Git inventory and the fixed
`/home/hoxide/mycodes/atlas*` sibling namespace twice. It requires exact
live/registered identities, no-follow real directories and the expected
primary-versus-linked `.git` type, and rejects detached, locked, prunable,
unknown, symlinked, extra, missing or sample-drifted records. `precreate` is
hard-disabled and every ACTIVE row is rejected. The helper never invokes Git
mutation commands or automatic cleanup. It is deliberately a cooperative
authorization/detection boundary,
not an operating-system or Git-command interceptor. A raw `git worktree add`
can create an unauthorized path that the next check detects only if it remains
in the managed namespace; that command is forbidden, and detection stops work
without adopting or deleting the path.

The 24 LEGACY `(path, HEAD, branch)` identities are now an exact frozen set in
the guard. Until a separate retirement-receipt schema is implemented and
HPC-verified, a missing, novel or changed identity is rejected before Git
inventory is consulted. This closes both "create first, then add a matching
LEGACY row" and unaudited "delete path and row together" transitions. It does
not prove historical monotonicity across arbitrary source rewrites or turn the
cooperative guard into an OS-level interceptor.

This guard is intentionally outside the frozen parent-seal transport and does
not describe the HPC worker's checkout. All22 mocked synthetic tests passed in
accepted AFTER v3 job3875239. A follow-up adversarial review found the P1 LEGACY
registry-laundering path fixed above. The former advisory `precreate` token is
removed from the usable state machine; do not re-enable that old schema when a
receipt-bound creator is eventually designed. Retained limits include paths
outside the fixed sibling namespace, create-then-remove and post-return races,
stable same-name reconstruction completed before a check, lack of an exclusive
owner/mode policy or cross-user security boundary, subprocess output buffering
before the size check, and the pending real
temporary-repository integration test. This does not close the separate
durability gap below.

Compute-node verification passed in job3875239. Durability remains open until
the registry, guard, tests, campaign helpers and launcher closure are committed
together and pushed. They are currently modified/untracked working-tree bytes,
so a clean checkout does not yet inherit the stop-growth controls.

The refreshed read-only local audit found 15 clean candidate worktree
directories for possible later non-force retirement (660079KiB apparent/
741908KiB allocated),
with neither untracked nor ignored files. None is currently
deletion-authorized or fully deletion-eligible. Their branch refs and commits
must all remain: 14 contain commits
unique among the current local/remote/tag refs, while the remaining branch is
cheap to retain. Removing a worktree directory is distinct from deleting its
branch or commit. Eight secondary
worktrees are dirty and must not move; `atlas-rust-avopt` is Git-clean but has
50 ignored evidence files (~1.97MiB) and is therefore not clean for retirement.

Exact candidate snapshot at `2026-09-30T20:17:36Z` (directory retirement
only; preserve every branch and commit, and recheck before acting):

```text
/home/hoxide/mycodes/atlas-cycle-foundation-repair
/home/hoxide/mycodes/atlas-deform-bucket-audit
/home/hoxide/mycodes/atlas-deform-groups-parallel
/home/hoxide/mycodes/atlas-deform-primitive-groups
/home/hoxide/mycodes/atlas-deform-target-audit
/home/hoxide/mycodes/atlas-kl-exact-reserve
/home/hoxide/mycodes/atlas-kl-retained-memory
/home/hoxide/mycodes/atlas-mu-fiber-audit
/home/hoxide/mycodes/atlas-recursion-operands-audit
/home/hoxide/mycodes/atlas-recursion-operands-reuse
/home/hoxide/mycodes/atlas-rust-avopt-shardunit
/home/hoxide/mycodes/atlas-trace-lines-wt
/home/hoxide/mycodes/atlas-unitary-singular-cayley
/home/hoxide/mycodes/atlas-wcell-cost-audit
/home/hoxide/mycodes/atlas-weyl-value-compact
```

The main worktree has 53 modified and 1383 untracked entries at this snapshot.
Before any future transition for these15 clean directories, refresh remote
refs, check host-side open files/CWD, repeat status including ignored files and
freeze a receipt-bound retirement record. Only the later authorized
transaction may use non-force `git worktree remove` on exact paths. Removing
branches is a separate, currently unauthorized operation. The present exact24
schema intentionally blocks removal. No worktree was removed by this audit.
No historical HPC stage is currently approved for deletion.

A second read-only audit at `2026-10-01T06:39:02Z` rechecked all 15 exact
paths. Each still had zero tracked modifications, zero untracked files and
zero ignored files; its branch ref equalled its worktree HEAD; none reported
merge, rebase, cherry-pick, bisect, lock or prune state. Fourteen HEAD commits
were kept reachable only by their own local branch; the
`atlas-deform-primitive-groups` HEAD was also reachable from the
`atlas-deform-target-audit` branch. Exact-path scans found only the registry
and this historical candidate list, except that `atlas-weyl-value-compact` is
also deliberately frozen in `hpc/test_local_worktree_guard.py`. The visible
same-UID process CWD/fd scan and `lsof` found no owner of a candidate path.
This improves confidence but does not authorize removal: the task owner must
release each path, remote refs must be refreshed at action time, every branch
and commit must remain, and a new retirement-receipt schema plus the registry
and frozen guard tests must first be implemented, HPC-verified and reviewed.
Therefore all 15 remain
`CANDIDATE_PENDING_USER_AUTHORIZATION`, not `DELETION_AUTHORIZED`.

Historical AFTER v1 pre-creation record: the exact path is
`/public/home/majj/atlas-rust-campaign-20260930/stages/ladder-boundary-after-v1`.
Its predecessor is accepted tests-first BEFORE v3 job3873400 and evidence SHA
`608e996acac10ea1bacca177468fcd83f3ec39c3e579899440e11aab674fc37f`;
its durable parent remains seal object
`db67234c0d67dbd6a6f0327386dd113b6093d25a46569710c33dfc8bc482cdcb`
of12488095 bytes. Changed inputs are the tests-first plus production
root-ladder patches, the AFTER-only launcher/driver and its publication/signal
repairs, and the directory-governance gates covering the exact70 stagers and
25-worktree registry. The exact27-payload override manifest is
`tests/reference/hpc/math_ladder_boundary_after_v1_overrides_2026_10_01.json`,
SHA-256
`2b4733b6151314ffa9ac4d67d694a65414a450f89a70617c97855e18f75de9be`.
Retention class is `ACTIVE_GATE_COMPACT`: retain the immutable pin, submission
state, report/checksum, patches and manifests, complete original/candidate raw
streams, command logs/timing/RSS and independently reviewed acceptance record;
the expanded source, Cargo target and reconstructed historical stagers remain
ephemeral. The stage becomes retirement-eligible only after FINAL independent
review, acceptance-ledger disposition, zero live/uncertain scheduler ownership
and a verified CAS dependency closure; eligibility never authorizes deletion.
The planned launch was one non-array, dependency-free cpu job at2CPUs/8GiB/2h.

Submission update: fresh queue/accounting/ledger/path reconciliation passed,
then shared submission accepted exactly job3873497 with `queue_before=[]` and
pin `200055d52c0d9559a2edf4b7e556296d0c8bd0df129030e4e855e4e4426df289`.
The only new directory is that pre-recorded stage child; the HPC home still has
318 top-level `atlas-rust*` directories. Its27 durable stage inputs are0444,
single-link regular files, `.incoming` is empty, and intent/receipt/ledger SHAs
are `8a620953dc51f2f5410adf685b86ec807ccb486c43e77db3f46dd728f7a8fc29`,
`cf7ce26c8f765e30a908ae9732f32d88e5bf95df1db8ef55de6f4d9188efb56e`
and `d78c75c4d53a1336cb48d0a5bbcfa2c52d81391e466026b8cccc4d7832f5f23d`.
The job was RUNNING when observed and at that snapshot was
`SUBMITTED_NOT_VERIFIED`.

Final inspection supersedes that live observation: job3873497 ended
`FAILED|1:0` after1m20s on cu006. All88 checker outcomes and both toolchain
commands passed, but `full-stager-inventory` was the first failing command;
the sealed source object contains zero historical stagers, so three overlays
cannot satisfy the frozen70-name inventory. No Cargo, patch or mathematical
command ran. The immutable v1 stage is `HARNESS_FAILURE`, with report SHA
`fec90eb2f4cf0a4dff4820c4527c5263c151f8dff4aabf272007618646149e15`.
  Its changed-input v2 successor remained under the same campaign root. It used
  one path-free CAS bundle for the67 retired
sources; the two new LEGACY-baseline guard tests brought its planned unittest
total to93. This correction adds no top-level directory.

V2 pre-creation state was frozen at2026-10-01T07:35:16Z before making the
stage child. The exact changed-input target is
`atlas-rust-campaign-20260930/stages/ladder-boundary-after-v2`; its predecessor
is immutable failed v1 job3873497 and its sole semantic change is repairing the
harness provenance assumption, not changing the mathematical fixture or Rust
production patch. Its28-file override manifest is
`tests/reference/hpc/math_ladder_boundary_after_v2_overrides_2026_10_01.json`
with SHA-256
`b3e4fa16cdb9e86aba36d4bdd5723401a32b0c07ef21cdce29165dd938daa699`.
The exact67-source canonical bundle is stored once in the existing campaign
CAS as `retired-stager-bundle`, 311925 bytes, SHA-256
`550b1330ec8806d1343d50d6ceb8488f89eb03546f6bb37c538cb5cd460e9b09`;
the upload scratch and local `/tmp` transport were deleted after rehashing.
  The queue was empty, the target stage absent and the campaign ledger was unchanged
  at this checkpoint. Static reviewers found the v2 pin/mock/AST/inventory and
  93-checker contracts aligned; compute-node execution remained the acceptance
  boundary. Retention is `ACTIVE_GATE_COMPACT`, with the same one-job resource,
  durable-evidence and separately authorized retirement conditions as v1.

  V2 submission update: one initial connection timed out before remote state
  could change; two later reconciliations proved no stage/job/ledger record
  existed. The exact child was then created and all28 inputs were rehashed
  before shared submission accepted exactly job3874203 with
  `queue_before=[]`. Pin/intent/receipt/ledger SHAs are
  `3f0f3c8b79e7ab7688ec329e242d18a82ac1b113eb33fa3788cfe1ca3e31455d`,
  `bf13ee5fa7f96b9846a2d000018a2951b1ef7dd6ae208e977fd2b3eb4e62c6a9`,
  `6c1e342a1e59f67bdfaa611acebcb9913c581ff3a06e5c9b7b8488c3127d8672`
  and `44624c765f57222be339cfaa62bbdaff82f6208a93e78d3fdc7c60e7aba1835d`.
  All28 installed inputs are0444/single-link and `.incoming` is empty. The
  top-level `atlas-rust*` directory count remained exactly318.

  V2 final inspection supersedes the live observation: job3874203 ended
  `FAILED|1:0` after4m55s on cu105 with report
  `256247b2be5fd7358ccb56e87ff262c6c0e87ca782e822e2f3c96419c4b4a694`.
  All93 checker tests (including22 worktree-guard tests), full-stager70/67,
  domain inventory521, both focused regressions and all521 domain tests pass.
  The driver failed only because full-suite `--nocapture` surfaced the panic
  hook deliberately caught by the passing cache-poisoning test and its parser
  rejected every `panicked at` substring. Core and final integrity were
  NOT_REACHED. Evidence
  `tests/reference/hpc/math_ladder_boundary_after_v2_failure_2026_10_01.json`
  SHA `e757f56ce6da55f1b96978daaaa2156b82b1bc3f83b36c2971f7a7a66be1f8f1`.
  This is immutable `HARNESS_FAILURE`, not mathematical failure/acceptance;
  v2 may not be mutated, duplicated, resubmitted or used to release a gate.

AFTER v3 pre-creation and submission record: its exact target is
`atlas-rust-campaign-20260930/stages/ladder-boundary-after-v3`; its direct
predecessor is immutable v2 job3874203. The reviewed29-input manifest is
`tests/reference/hpc/math_ladder_boundary_after_v3_overrides_2026_10_01.json`,
SHA `3fe6c8c5c0a0638ae8d128313ad3bfce02bc97e3d16a69f75670a864b28695c2`.
Final static review found all29 hashes and95 checker methods aligned. A fresh
empty-queue/accounting/path/ledger reconciliation preceded creation of only
that registered campaign child. Shared submission accepted job3875239 with
`queue_before=[]`; pin/intent/receipt/ledger SHAs are
`7af79aa20b37d0fcee9a60d8e3c6b4dbec05417cf6817747520b6ebe3ec41f88`,
`feff9e66a0317719258db7e210325a109e263d1447790b71d4a444c33f79a0ab`,
`ac1f6f52f99de9b86b7b7bbfe56e7478809f193f1938a0c07219bc6061ee2254`
and `a1776412e7dd6633c329743b52c5cf4b1615dde8a7a2f0b4fddd002b5615a618`.
All installed inputs are0444/single-link. The job was RUNNING on cu105 when
observed, and the top-level `atlas-rust*` directory count remained318. This is
`SUBMITTED_NOT_VERIFIED`; FINAL review, acceptance-index disposition, zero
live/uncertain ownership, verified CAS closure and separate exact-path deletion
authorization remain mandatory before retirement.

Final inspection supersedes that live observation. Job3875239 completed
`0:0` on cu105 in10m18s. All95 checkers, full-stager70/67, domain521
inventory/2 focused/521 full and core630 inventory/1 focused/630 full pass;
all16 commands exit zero and all32 log/time artifacts rehash. Report SHA is
`771fc790dd4340f408235e50c3f6eee754ebe4c850cbad36d4af4f902a24627c`;
independent evidence
`tests/reference/hpc/math_ladder_boundary_after_v3_2026_10_01.json` SHA is
`a459fa08117ff8a721181d349467ebdd9267e798cd25b5bcb1380996c2d15e15`.
Final source integrity passes, the driver-process audit observed zero legacy
path opens, `.incoming` is empty and the ephemeral build workspace is absent.
Both installed and retained
override payloads match all29 hashes. The durable child is3220KiB allocated;
the HPC home remains at318 top-level `atlas-rust*` directories. This bounded
acceptance does not itself satisfy the separate commit/push or exact-path
retirement conditions, and authorizes no deletion. Its separate
acceptance-index publication condition was subsequently satisfied by the
compact gate below.

Acceptance-index publication completed in job3879103 (`COMPLETED|0:0`, 17s,
cu075). Stage/index/allowlist suites pass20/34/7 with no skips; all24 installed
inputs and the exact nested directory topology match the frozen manifest;
source and retired-stager CAS integrity recheck; no Cargo or Atlas command ran;
and the ephemeral workspace is absent. Full report
`tests/reference/hpc/math_ladder_boundary_index_v1_report_2026_10_01.json`
has SHA
`43cc79e1630b5ec21a24c5aa1a7c411eafffaa9a4c6911333c8752b0dc63cf9d`;
independent inspection
`tests/reference/hpc/math_ladder_boundary_index_v1_2026_10_01.json` has SHA
`d9448e5cd53ba4c25e6a0fd91a5fdb16e3b423d48fe58531a50d7d7457afb9ad`.
The latter explicitly retains the limitation that legacy-path auditing covers
the driver process only. The queue was empty after inspection, and the HPC home
remained at exactly318 top-level `atlas-rust*` directories. Only the registered
child `atlas-rust-campaign-20260930/stages/ladder-boundary-index-v1` was added;
no new top-level campaign was created. This is publication/durability evidence,
not a rank, performance, broader mathematical or deletion release.

The parent-seal gate is accepted. Its first child was the exact
`ladder-boundary-before-v2` stage below the same campaign; the parent launcher
was closed before those reviewed active-policy bytes were frozen.
Pre-creation lifecycle record: its predecessor is `weyl-parent-seal-v1` seal
`db67234c0d67dbd6a6f0327386dd113b6093d25a46569710c33dfc8bc482cdcb`;
the changed-input reason is the post-seal launcher transition plus the
original-accepted/Rust-overflow root-ladder regression. Its exact 20-payload
manifest is
`tests/reference/hpc/math_ladder_boundary_before_v2_overrides_2026_10_01.json`
(`1bbfd0027de76a812b2ee7378b16bbcb85c88614b92d0451b0ff739a443e0d82`),
with expected pin
`62be23da13326b28bd2f5fe0f7cd8ae2f6fa4533b21d1164e12c017f92077138`.
Retention class is `ACTIVE_GATE_COMPACT`: retain its immutable pin,
submission state, report/checksum, inventories, patch, failing command logs
and exact source manifest; remove only its expanded node-local source/target
workspace automatically. The durable stage becomes retirement-eligible only
after a verified repair/AFTER successor preserves the regression evidence, a
CAS-only replay/reachability audit finds no remaining reference or live job,
and the user separately authorizes that exact path. Reconnection and an
unchanged-input retry would reuse this one logical stage.

That v2 attempt submitted exactly job3872594 with an empty queue and failed in
11 seconds before Cargo, inventories or the three mathematical BEFORE
regressions. Report
`4ee3b6cd0d09c2f152a9b5f7fa8a216db0197fd291077d9800add3755fb14e2c`
records 63/65 checker outcomes passing: one Python-contextmanager global-scope
false positive and one incomplete synthetic one-file SBATCH fixture. Preserve
the immutable v2 pin/result; it is neither a mathematical failure nor a reason
to mutate or resubmit that stage. The inspection record is
`tests/reference/hpc/math_ladder_boundary_before_v2_2026_10_01.json`.

At that historical point, the next and only eligible stage was
`ladder-boundary-before-v3` in the same campaign. Its predecessor was the
failed v2 report above; its changed-input
reason is exactly the two checker repairs plus attempt-specific stage/job
names. It keeps the unchanged v2 pin/report schema because their shape did not
change. The exact 20-payload manifest is
`tests/reference/hpc/math_ladder_boundary_before_v3_overrides_2026_10_01.json`
(`310a2b1793f7cc2f35ffcfcb54f0aab8cc779291a391fa582dc2d85b5dc7fe7e`),
with expected pin
`ca3e22561fe493b4fe1ea32751bf772e55b62e268e631091e2ee87c0f7c6c60a`.
It inherits `ACTIVE_GATE_COMPACT` retention and the same retirement condition.
After a fresh empty-queue/accounting/path/ledger reconciliation, the frozen
stager submitted exactly job3873400 with `queue_before=[]`; it was observed
running and then completed `0:0` on cu006 in5m36s. Independent inspection
accepts the exact tests-first BEFORE evidence:65 checker tests,521/630
inventories, exactly2domain+1core named failures,0 ignored, unchanged
production, rehashed command/input/CAS artifacts and an absent ephemeral
workspace. Report SHA is
`51cc7a14a0dbb8188a4ea47705338e5689212bf1f707819bab8c4e5681e4052b`;
the inspection record is
`tests/reference/hpc/math_ladder_boundary_before_v3_2026_10_01.json`.
`report.source_file_count=1558` is the sealed parent count while its final
`source_files` manifest has1561 entries after exactly three fixture additions;
this is not an integrity mismatch. Ledger/intent/receipt hashes are
`319981ee29a86e900ca6c9536ff8a3dbab5744d2866db1150228e07baf50160e`,
`985491fa2ecc744f5ec0dfb8c08c71a49a975b4d2155bc006e2f591de63f9505`
and `4fc0b646056a02f45b265ae53b8cdd8be8a0efaa63e14b583898434ea3d75ebd`.
Do not create another BEFORE stage or resubmit BEFORE v3 job3873400. This evidence releases only
the minimal production repair and its separately pinned AFTER gate.
Historical stages remain untouched until the migration is accepted and a
separate CAS-only reachability/replay dry run proves each exact deletion safe.
Eligibility still is not deletion authorization.

## Required post-seal replay contract

The independent replay must consume only the parent-seal object reference and
campaign CAS. It installs a filesystem-open audit hook before importing the
project helpers, rejects every old top-level `/public/home/majj/atlas-*` read
(allowing only the active campaign), calls
`load_parent_seal(..., verify_archival=True)`, verifies every unique reachable
blob, and materializes the1558-file Rust source into an auto-cleaned workspace
to recompute its exact manifest. It launches no subprocess and performs no
delete. Its report records verified unique-object count/bytes, zero missing
objects, zero legacy-path open attempts, exact source reconstruction and
`delete_actions: 0`.

The parent-seal validator must require the schema's exact top-level key set.
Unknown scalar fields and unknown reference-shaped dictionaries are rejected;
otherwise a future producer could hide a blob reference outside the verified
reachability traversal. Unreferenced ordinary objects already present in the
campaign CAS are neither errors nor part of the seal closure.

The seal contains the complete regular-file open set observed by the frozen
gate; it is not a complete directory inventory or a global reverse-reference
graph. Therefore the first CAS-only replay must mark every historical directory
`eligible: false`, with at least `DIRECTORY_INVENTORY_MISSING`,
`GLOBAL_REVERSE_REFERENCES_UNPROVEN` and
`LIVE_OR_UNCERTAIN_OWNERSHIP_UNPROVEN`. Nonempty eligibility requires a
separate read-only `atlas-retirement-inventory-v1` object covering exact
directory contents, all report/pin references, scheduler state, unresolved
intents, dirty Git state and unique commits. Scheduler ownership is rechecked
again immediately before any later user-authorized exact-path removal.
