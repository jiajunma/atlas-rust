# Remaining builtin coverage (post-language-gate)

LATEST 2026-10-10 registry recount: `builtin_registry()` in `typed.rs`
currently holds **479 entries over 240 distinct names** (per constructor
family: scalar 167/59, domain 154/119, domain_validate 86/58,
domain_skip 29/19, domain_printer 21/19, domain_relation 22/2), counted
exactly over the `vec![…]` registry body; the file is byte-identical at git
base `964f0033` and HEAD.  This corrects the older "321 entries / 170 names"
figure that counted only the scalar_builtin + domain_builtin families.  The
gap-tracking entries below remain historical records of their dates.

LATEST 2026-10-02: BEFORE-v3 job3884903 failed only the allowlist after 112
prerequisite passes. Root's version migration accidentally changed the expected
message for the immutable historical ladder BEFORE-v2 driver to v3. Restore
that expectation, retain the historical source, and assert this exact mutation
is rejected with filename/line/node diagnostics. Disabled BEFORE-v4 preparation
changes no Rust production bytes, fixtures or goldens. All failed stages remain
immutable; mathematical BEFORE and semantic repair are still pending.
V4's independent static reviews are complete, but final whole-file pins and
HPC execution are not. A real tool-free Kimi review completed in 109.531 seconds
(CLI2.1.1, `kimi-code/k3-256k`, session `session_f338a507-d213-4a17-8ad7-fd3297da9d2c`);
it changed no files and grants no test acceptance. Exact prompt, result and
coordinator review are in
`docs/evidence/kimi-weyl-allowlist-20261002/01-review/` and `docs/HANDOFF.md`.

Historical tests-first BEFORE-v2 job `3884880` is FINAL `FAILED 1:0`
after 56 seconds. All 112 tests in its first five checker suites passed. The
last allowlist suite had five passes and two errors: the new stager used
dictionary unpacking and a subscript where only bounded pure expressions are
allowed. BEFORE-v3 preparation changes these expressions, not the safety
policy, Rust production source or original goldens. Its guards remain disabled
pending review. No Cargo/Atlas/regression execution occurred in v2; it is not
mathematical BEFORE evidence. The failed stages remain immutable.

Historical tests-first BEFORE-v1 job `3884862` is FINAL `FAILED 1:0`
after 47 seconds. The 84 prerequisite checker tests passed; the driver suite
reported 26 passes, one stale prediction-classification failure and one
format-sensitive source-substring error. Cargo and Atlas never ran. Fixing
these checker tests requires one reviewed changed-input BEFORE-v2 successor,
not rerunning v1 or treating the harness failure as a mathematical regression.
The original goldens, two Rust regression tests and production source are
unchanged. Semantic repair, cache work and rank escalation remain blocked.

CURRENT 2026-10-02 Weyl identity gate: `weyl-context-core-capture-v8` job
`3884807` is immutable FINAL `COMPLETED 0:0` after 345 seconds on `cu001`.
All 102 checker tests, source reconstruction and release build passed. The
separate independent audit passed and classifies the complete fresh-process
capture as `CAPTURE_COMPLETE_RUST_SEMANTIC_MISMATCH_CONFIRMED_REGRESSION_REQUIRED`.
Its 11,902-byte evidence record is
`math_weyl_context_core_capture_v8_inspection_2026_10_02.json` at SHA-256
`b2c7f4ece709df3506c3224a57abad896f3ecfcd6625fea97731619f629e1da3`.

The capture confirms two Rust errors. Cold canonical duals are compatible in
original Atlas: equality is true, inequality false and products succeed in
both dual-construction directions (SC-to-adjoint and adjoint-to-SC); Rust
returns false/true and rejects both products. Reverse operand orders remain
pending coverage. When the canonical target was independently prewarmed,
original Atlas rejects equality, inequality and multiplication before a value;
Rust incorrectly returns false/true for the relations and rejects only the
products. Source-prediction formatting was inexact, but the independent audit
used the complete sealed raw streams.

Four byte-exact original goldens are now retained. Their SHA-256 values are
cold stdout `7a614e47...80dc`, empty cold stderr `e3b0c442...b855`, prewarmed
stdout `5074aab3...021f` and prewarmed stderr `ef4404d8...e157`. The new
5,208-byte `weyl_context_core_regression_catalog.json` has SHA-256
`6a9f960753bcd2afb575cb5207d4310f21a5694505d02a83f4e4142211eaa634`.
Two focused tests in `crates/atlas-core/src/session.rs` (`969cdb27...07e7`)
and the tests-only patch `hpc/patches/weyl_context_core_regressions.patch`
(`ccd30098...49cf`) are frozen but unexecuted. The next permitted HPC stage is
tests-first BEFORE: the frozen original must match all four goldens while
unchanged Rust fails for the observed reasons. No production repair may begin
until that failure evidence is retained; the same regressions must pass AFTER.

This capture grants no mathematical or cache acceptance, performance or memory
claim, or rank release. The 345-second total is mostly the 281.53-second build;
one-shot math invocations of 0.00–0.01 seconds cannot support a speed ratio.
The queue was empty after inspection and both exact transports are absent.
Never invoke v8 again, create a retry sibling or advance rank before the
regression BEFORE and subsequent original-backed repair pass.

HISTORICAL/SUPERSEDED 2026-10-02 Weyl identity gate: `weyl-context-core-capture-v4` job
`3884494` is an immutable FINAL `FAILED 1:0` harness result.  Its first three
suites passed (32+17+18), but HPC Python 3.9 rejected the 27-test module before
test discovery because an oversized multi-manager `with` exceeded the static
block limit.  Zero Atlas invocations, builds, captures, mathematical comparisons
or performance comparisons ran; no mathematical regression or acceptance is
implied.  The exact v4 failure evidence SHA-256 is
`051f5292f51dc2ed413a277932fb79556295732a58634c7cdff7cd27fe807517`.

The only changed-input successor, v5, was submitted exactly once as job
`3884727`; its first durable observation was `RUNNING` on `cu001`, the only
outstanding job.  The ledger now has exactly 12 records at SHA-256
`e5c1ed292453da24720c180dd0702cc031bd125716bf0ce053049e32bcc0d4bc`.
It uses `ExitStack` for that exact patch set, binds the immutable v4 tree and
eleven-record predecessor ledger, and retains the 32+17+18+27+7=101 checker
gate before build/capture.  Kimi made only the bounded mechanical refactor;
Codex verified the exact predecessor diff and owns the v5 migration and HPC
gate.  Static hash/AST/method checks pass but are not execution evidence.
Submission evidence SHA-256 is
`fedccff3c50211e0b470651e95cea354782b5f3e63df6f566ab5489fdbb43f3d`;
both exact transports are removed.  Do not invoke the creator again, resubmit
v4/v5, create a retry sibling or advance rank.  See the top of
`docs/HANDOFF.md` for exact tree, Kimi and r6/r7 evidence.

HISTORICAL/SUPERSEDED 2026-10-01 Weyl identity checkpoint: the core-only A1 capture package was
statically ready but unsubmitted. Fresh HPC reconciliation shows an empty
queue, the exact seven-record confirmed campaign ledger at SHA-256
`b336fb831de28f72e9138284d2c6f07ecfb178f617ebe4bec78e1c82ffb8eaa7`,
matching terminal `sacct` states for all seven job IDs, and no
`weyl-context-core-capture-v1` child. No remote directory or job was created.
This is now only the last successful checkpoint: a newer read-only queue query
at `2026-10-01T14:42:36Z` timed out before authentication, so current queue,
ledger, accounting and target state are unknown. Timeout evidence SHA is
`4f38e83298fd329c...`; no mutation occurred. Repeat all four observations after
connectivity returns before creating or submitting anything.
Manual `mkdir` remains forbidden. A receipt-bound creator for that one fixed
child is now implemented locally with PREPARED/SEALED/PUBLISHED markers,
no-replace publication, atomic intent uniqueness, exact-contract recovery and
running-job receipt closure without resubmission. It has no arbitrary stage or
campaign parameter and rejects raw finals, siblings, symlinks, lock swaps,
mode changes and changed manifests. The package remains capture-only. Its five
compute-node checker suites contain 21+17+18+27+7=90 methods and the driver
expects exactly nine command records. Current progressive/stager/driver/
creator-test/progressive-test/capture-test/allowlist hashes are
`2481383117e0...`/`b165795e0ceb...`/`550ff8651e6a...`/`e0820335582b...`/
`2c4beac47a31...`/`7d51c65da190...`/`e1767281c5db...`. Static review repaired
the post-intent validation dead end, directory-entry durability/replay windows,
derived count/order assumptions and the omitted 17-test submission suite, but
no HPC, Cargo, Atlas or local Python test has executed these bytes; the first
compute job must verify all 90 checks before any capture or mathematical claim.

Do not implement the Weyl cache after A1 capture alone. A1 constrains cold and
prewarmed dual identity histories but only multiplies the rank-one generator
with itself; it cannot validate cross-coordinate permutation replay or
interface ordering. It also leaves a root-datum alias live after rebind and
cannot prove internal owner/cell topology. Independent review must first freeze
complete raw streams and any confirmed mismatch as a regression, then semantic
AFTER must pass. Progress next through G2, B2/C2, both multiplication orders,
sole-WeylElt lifetime, inner-class dual and no-value relations. Only then run a
work-count BEFORE/cache/AFTER and controlled same-node time/CPU/RSS A/B.

The candidate ownership design is two-layer and acyclic: datum-local lazy
`RootSystem`, separately weak-interned abstract group identity with
`WeylInterface`, and a context holding the handle plus both immutable `Arc`s.
Original inner-class construction eagerly obtains and retains the canonical
dual datum, so the same dual-identity path must cover it; explicit datum dual
alone is insufficient. The internal compact `[u8;32]` transducer suggests a
later language-WeylElt representation experiment, but combining it with this
semantic repair would make attribution and review invalid. These findings are
indexed in the new KB source packet; llm-wiki-compiler produced no candidate
because its provider failed, and external retry requires explicit authorization
for the named source payload/destination.

Kimi ACP interaction2026-10-01: actual two-way local tasks, question/answer,
remembered follow-up, cancellation/recovery, new-process session reload and
close/EOF/deadline/SIGTERM are now verified. The first stale auto-mode failure
and the corrected capture are both retained. See the
[interaction guide](slices/kimi_acp_interaction_2026-10-01.md) and root AGENTS.md.
The ACP profile proposes code as text with AskUserQuestion only; Codex owns
review and application. No Atlas code/program test/math gate changed. MCP
wrapping is an explicit future design, not a currently installed tool.

Kimi runtime verification2026-10-01: actual local Codex-to-Kimi delegation,
Read/Edit coding, exact-session continuation and bounded exit paths now have
eleven recorded invocations under the user's explicit local-integration
exception. Root AGENTS.md records the verified runner/profiles and limitations;
see [runtime report](slices/kimi_runtime_validation_2026-10-01.md). No generated
program test, Atlas source change, mathematical acceptance or HPC gate occurred.

Kimi workflow research2026-10-01: the installed CLI is now2.1.1. The
[version-pinned integration guide](slices/kimi_subagent_workflow_2026-10-01.md)
documents external CLI workers, restricted Markdown profiles, task packets,
session capture and coordinator-owned HPC verification. Version/help were
inspected; model execution and coding delegation remain unverified. This is
routine-work tooling guidance, not mathematical coverage or a gate release.

Knowledge maintenance2026-10-01: `kb/` is the user's same-repository knowledge
base, focused on Rust evolution with C++ as an alignment baseline. Update its
relevant topic pages alongside future mathematical, algorithm and implementation
changes; see `kb/index.md` and `kb/AGENTS.md`. The user has selected
llm-wiki-compiler (pinned1.4.0-rc.2); use `./kb/llmwiki` with the required
instructions and candidate-review workflow. The initial
pages are draft explanations, not new coverage or release gates. Existing
fixtures, reports and the mathematical acceptance ledger retain their roles.

LATEST discovery3868832 FINALbdee1281: original accepts all11 A1+torus
coordinate-boundary cases/22rows; Rust returns10rows and six Runtime overflow
errors. Original goldens plus2kernel/1full-stream regressions now exist, but
the tests-only BEFORE is not submitted or accepted. Two Weyl cache drafts are
invalid positives because bare `matrix(WeylElt)` is script-defined: retain the
Program-vs-Name rejections and replace them only in a new capture. Before the
next mathematical job, storage migration is mandatory: one dated campaign
root, shared CAS and auto-cleaned source/target workspace. Exactly one
`weyl-parent-seal-v1` attempt, job3872554, is FINAL `COMPLETED 0:0` and
independently accepted in its infrastructure-only scope from the sole campaign
stage. Report SHA is `939482542c123a6a2a2c22f8dbe1e052af20dee3af4010da780fe958dc41bf79`;
the seal/source CAS hashes are `db67234c...`/`5133bb32...`. It must not be
resubmitted. Seal-only `ladder-boundary-before-v2` submitted job3872594 but
failed its checker preflight at63/65 before Cargo or the three expected math
failures; report `4ee3b6cd...` is retained and is not math evidence. Its two
harness defects are repaired only in the separately pinned v3 attempt, which
keeps the unchanged v2 protocol schema. The parent launcher is closed and only
`ladder-boundary-before-v3` ran exactly once as job3873400 and is FINAL
`COMPLETED 0:0`. Independent inspection accepts its tests-first BEFORE result:
65 checkers pass, inventories are521/630, and unchanged production has exactly
2domain+1core named failures with0 ignored. Report SHA is `51cc7a14...`; the
inspection record is
`tests/reference/hpc/math_ladder_boundary_before_v3_2026_10_01.json`. Do not
resubmit it. This releases only the minimal overflow-membership repair; no
AFTER, performance or rank gate is accepted. No
bulk cleanup of historical evidence; see hpc_artifact_retention slice.

STORAGE audit: one primary checkout plus24 linked secondary worktrees
(8secondary dirty), and317 HPC top-level entries whose names begin
`atlas-rust`, a subset of the1174 whose names begin `atlas` at observation
time; the HPC counts overlap and must not be added. Recent retained
targets~828/416MiB. New campaign
workspace/shared-ledger guards and the content-addressed source/blob helpers
are now accepted by parent-seal job3872554 in their bounded infrastructure
scope:64 checker tests, two identical traces and629/519 inventories. Legacy
paths were read only inside that migration and are now replaced downstream by
the path-free seal. No old stage is yet cleanup-eligible or deletion-authorized;
a separate CAS-only replay/reachability dry run is still required. This changes
infrastructure only, not math or performance acceptance.
The active root is now hard-pinned to campaign20260930; a second date, an
out-of-campaign result workspace, a home-resident scratch tree, a symlinked or
option-shaped job script, and a nested `sbatch` all fail before scheduler
contact. The checked job-script snapshot is submitted on stdin. The prominent
old rank6/loading launchers and their documentation are explicitly disabled;
historical evidence remains readable.

All later `RUNNING`, `SUBMITTED` and “next target” statements are historical
performance-ledger entries. They do not override the parent-seal storage
barrier and do not describe the current scheduler state.

KLV storage source audit2026-10-01: the user's equal-polynomial hypothesis is
correct, but pair-level deduplication is already present. Original
`kl.h:47-86,113-125` stores compact polynomial indices in columns and one
content-deduplicated pool; zero/one are indices0/1 and many entries are
implicit. Rust `kl_polynomial.rs:187-229` and `kl_table.rs:35-120` likewise
store one `KlPol` per distinct coefficient vector per table. The verified F4
full table has336x336 language-facing entries but only176 pooled polynomials.
Do not add an `Arc<KlPol>` to every pair: that would enlarge each pair entry
and add reference-count traffic.

There are still several source-backed storage gaps worth measuring after the
current Weyl/ladder priority. Rust's read paths clone complete coefficient
vectors where original `KL_pol` returns `const KLPol&` (`kl_table.rs:699-740`,
`rep_context.rs:2041-2047`, `deform.rs:466-472`). Rust's
`HashMap<KlPol,usize>` also owns a second full coefficient-vector copy of every
unique polynomial as its key, whereas original `HashTable` stores only pool
indices in its open-address table and compares through the pool
(`hashtable.h:75-105`, `hashtable_def.h:54-114`). Rust column/pool indices are
`usize`; original `KLIndex` is explicitly `unsigned int` because fewer than
2^32 distinct E8 polynomials are expected (`Atlas.h:460-475`). Each Rust block
table owns a pool, while original `Rep_table` offers owner-wide positive/signed
polynomial hashes shared among blocks (`repr.h:585-605`,
`repr.cpp:2031,2116`). Finally `ExtKlMatrix` currently retains a dense
`Vec<Vec<KlPol>>` together with its index matrix and pool even though production
consumers need only survivors/indices/pool.

The required bounded order is measurement probes -> borrowed/zero-copy reads
-> checked-u32 columns plus an index-only hash -> direct packing/cache
experiments. The last stage includes dropping the extended dense matrix after
local condensation and considering owner-scoped shared pools only if measured
cross-block overlap is substantial. An `Arc<KlPol>` per pair is
not the answer; if Arc backing is tested at all, one unique polynomial's pool
entry and hash key may share it. Preserve pool insertion order, zero/one
indices, exact collision comparison, signed/unsigned pool separation,
partial/full retirement and complete KLV output. A shared owner pool may retain
polynomials from retired blocks and add lock/refcount cost, so it can increase
peak memory or reduce speed. Instrument matrix-entry count, coefficient lengths,
read-clone bytes, per-table versus owner-global unique count, duplicate-key
bytes, pool hit rate, retirement retention, lock time, wall/CPU and peak RSS
before any production edit. ParamPol is a separate virtual-representation sum,
not a KLV polynomial pool; profile its term clones independently and do not
globally intern it. These are source leads, not accepted savings or authority
to interrupt the parent-seal/ladder sequence.

LATEST profile3868803 FINAL1d2baadd:23checks/6controls/4perf/integrity PASS.
Root ladder SELF16.18-16.25%U/12.75-14.14%AV, mostly build_weyl_context;
Type::equivalent now1.66-3.29%. Short samples, not new speedup. Next measured
target repeated Weyl construction. HISTORICAL PRE-FINAL STATE, superseded by
the FINAL3868832 result at the top of this file: discovery was submitted from
an empty queue,4p1Lbsj3/pin459a5147,33checks/3histories/6fresh processes on the
unchanged accepted binary. It included semisimple-rank1 reuse/rejection and the
A1+torus i32 boundary; no claim was permitted until that now-inspected FINAL.

CURRENT ONLY profile3868803 RUNNING1m24cu315/profile-release after23checks
and independent complete FINAL-parent verification. ZUDoR41Y/pin2ea6f04d,
submitted from empty queue; no sampling yet. Exact receipt indexed in HANDOFF.

LATEST3868782 FINALfbc615d2 accepted:46checker/13focused+ALL629core/
26histories/72retained/48serialAB and final integrity. Ufull3.241245->2.538780s
(-21.67%,still9.15971xoriginal); AVfinite4.307805->3.208666s(-25.52%,still
7.53406xoriginal). MedianRSS+0.41%/+0.51%. All4reps faster on all4inputs.
Bounded loading-inclusive serial scope, known failures retained. Profile
stage ZUDoR41Y prepared after inspected FINAL,23checker/6controls/4perf;
newest receipt authoritative. No root kernel edit or boundary capture yet.
All older pending3868782 notes superseded; see HANDOFF for exact hashes.

LATEST R23868782: all629core/13focused/26whole histories/72retained pass;
A/B RUNNING (5of16rows in latest snapshot), no FINAL/speedup acceptance.
New source caller: oracle elliptic.at128-135 binds one adjoint datum pertype
and eagerly computes W_elt(...).matrix.char_poly for3/9/5/12/30words, matching
G2/F4/E6/E7/E8old probe counts. Script SHAac764741 matches HPC. Weak-only
context caching may lose reuse because temporary WeylElt is immediately
discarded; whole strong context in handle cycles. Investigate strong acyclic
kernel ownership after R2FINAL/reprofile. See HANDOFF; no runtime edit yet.

The former per-owner `WeylKernel {RootSystem,WeylInterface}` plan is
superseded. `RootSystem` is an owner-local coordinate cache; `WeylInterface`
belongs to a separately weak-interned abstract group identity whose pointer is
the original compatibility token. `dual()` shares that identity only when the
canonical target is cold; a prewarmed target remains distinct. Cross-coordinate
equality/product must replay words in the left system, never compare/compose
foreign root permutations. Binary `=`/`!=` therefore need a fallible relation
path, including no-value checks. Cache state stays out of structural RootDatum
`Eq`/`Debug`; failed construction is retryable and errors use the current span.
The two core-only A1 fixtures have now run in v8. Independent review confirms
the cold-dual compatibility and prewarmed-owner rejection mismatches described
at the top of this file. Four byte-exact original goldens, the regression
catalog (`6a9f9607...a634`), two focused Rust tests and a tests-only patch are
frozen; all regression code remains unexecuted. The next stage must establish
the unchanged-Rust BEFORE failures before any semantic repair. After the same
regressions pass AFTER, G2/B2/C2, reverse operand orders, inner-class dual and
no-value semantics must pass progressive original-backed gates before the
cache work-count BEFORE.
Only after that does the elliptic 59-to-at-most5 build hypothesis receive
same-node time/RSS A/B; it is not an accepted speedup or memory saving.

Historical pre-3868832 source audit, superseded by the original acceptance at
the top of this file, added the then-PROVISIONAL/UNEXECUTED
root_ladder_coordinate_boundary.atlas: A1+torus, dual, both numberings,
11separate calls around checked i32 subtraction boundaries. Reflection uses
i128 intermediates; ladder pair differences do not. Original capture was
required before claiming a bug or changing runtime, and FINAL3868832 has now
satisfied that requirement. Detailed rationale is in the performance slice;
no additional rank escalation follows from it.

Post-capture source audit fixes the repair boundary. At frozen `7e1b958c`,
the original constructs ladder tables in small abstract simple-root
coordinates and Weyl-transports them before `RootDatum` maps roots into the
ambient lattice. For the A1 cases it directly records both roots as singleton
bottoms and performs no subtraction; the large torus coordinate is never
involved. Rust instead subtracts ambient stored `i32` roots/coroots. If the
exact difference has a coordinate outside `i32`, it cannot equal a stored
root/coroot, so this membership query is false rather than a datum-construction
error. The later minimal repair catches `ArithmeticOverflow` only around the
two `subtract_coordinates` calls in `build_ladder_bottoms`, inserts the
corresponding bottom, and propagates allocation and all other errors. Do not
change the helper globally, reflection, `combine_roots`, storage width, or use
wrapping/saturating arithmetic. Execute and inspect the frozen BEFORE gate
before applying that production patch.

Latest3868782 live9m41cu315:13focusedPASS, ALL629core running; no fresh
history/retained/A-B acceptance. Source boundary fixture SHA dc88d660.

CURRENT ONLY R23868782 SUBMITTED from empty queue, kzNyGVyu/pin320dcf98;
46checker/13focused+ALL629core/26histories/72retained/48AB. Exact FINAL
required before acceptance/reprofile. Runtime unchanged from R1 d9d43755.
Live2m58cu315:10after-checker testsPASS, before-release. Future profile
package618d4f0d replaces stale3aea9ce7,23checks/6controls/4perf; NOT submitted.
Inspect R2 FINAL first; exact package paths and source hashes in HANDOFF.

LATEST3868767 FINAL HARNESS_FAILURE13116add:44checkers/13focused/ALL629core
PASS, but missing observe import stops before ANY fresh history/retained/A-B.
No mathematical failure or speedup evidence. R2 imports canonical observer,
adds2deferred-global/fault-injection tests (46total), preserves all runtime/
Rust tests/goldens and26/72/48full gates. Fresh stage kzNyGVyu; newest receipt
authoritative. Pending3868767 notes below superseded; profile package3aea9ce7
must be refreshed after R2 FINAL, not submitted as-is. See newest HANDOFF.

CURRENT ONLY AFTER3868767 SUBMITTED,8BvdICJ0/pin0927aee7, from empty queue.
44checker/13focused+ALL629core/26original histories/72retained/48AB; no FINAL
or accepted speedup. Candidate runtime patchd9d43755, inspect exact job next.
Live1m11cu315:44checkers PASS, before-release building; no candidate checks yet.
Weyl follow-up caveat: old timer groups equal datum VALUES, not shared Arc
identity.30E8rebuilds alone do not prove handle-local cache hit rate; next
on/off probe should distinguish lifetime-safe identity and caller paths.
Later live8m03cu315: before-release passed, candidate inventory compiling.
Prepared ONLY reprofile package3aea9ce7 (22checker/2loads/6controls/4perf),
not submitted until FINAL; full629/26histories/72retained/48AB verifier included.
Source-only alternative to handle caching: original rootdata.cpp238-317
transports simple-root ladder bitsets by Weyl action; Rust brute-force checks
all root pairs. Preserve ambient RootId order, both lengths/coroots and
overflow/budget rejection behavior; no kernel edit or measured benefit yet.

LATEST BEFORE3868749 FINAL3fd3aba3:22checks/622compiled/2actual3-versus-1
workFAIL+4semantic/full-originalPASS, production and final integrity unchanged.
Only then implemented runtime cache candidate d9d43755: name-local invalidation,
owned TypeTable revision identity, immutable signatures only, no stale Values/
inference.7new lifecycle tests,629total, not after-verified.44checker/26original
histories/72retained/48AB AFTER prepared; newest receipt/HANDOFF authoritative.

Historical before3868749 SUBMITTED from empty queue, E1WDSwZy/pin71b172ae;
22checker/622inventory/2workFAIL+4controlPASS expected, production unchanged.
Inspect exact FINAL next; do not resubmit or claim cross-command cache acceptance.

LATEST command discovery3868735 FINAL9d2b560d:28checks/2histories/4processes
and final integrity, positive full stdout/stderr equality, negative full stdout
equality but Program vs Name+Type preserved with2exact causes. Four original
goldens retained. Six tests-only additions now622inventory (not compiled):
2expected work-countFAIL+4semantic/originalPASS. Patch67d5fb38, production
unchanged, no cache. New22checker before stage E1WDSwZy prepared; receipt
authoritative once submitted. Prior pending capture notes superseded.

Historical cross-command discovery3868735 SUBMITTED, nz47e1mI/pind959b55f,
28checker/2histories/4fresh processes, unchanged production. Empty queue
before submission; inspect exact FINAL before runtime/cache work.

LATEST FINAL3868661 accepted, report14a45971:53checks/11focused+ALL616core/
5completion+19old histories/72retained/48serialAB/source+artifact integrity.
Same-node Ufull4.022098->3.232297s (19.64% less), AVfinite5.541168->4.314198s
(22.14% less); still11.6017x/10.0341x original end-to-end time. RSS effectively
flat. No kernel/multicore/higher-rank inference;3shared Hodge failures and
distinct rejected diagnostics remain explicit. Next2case cross-command
overload discovery prepared (not a cache implementation), stage nz47e1mI;
submission receipt is authoritative when present. Full FINAL details in
HANDOFF/performance slice. All older pending3868661 notes are historical.

LATEST PARTIAL3868661 at10m54: ALL616core actually PASS in129.49s (none
ignored/filtered), candidate-release phase.53checks/11focused already PASS;
fresh original/retained/48AB still pending. No FINAL speedup acceptance.

PARTIAL3868661 at9m16:53checks/616inventory/ALL11focused PASS, including
all4unchanged original-backed completion regressions. Full616core RUNNING;
fresh differential/retained/A-B still pending, not FINAL or speedup evidence.

Continuation revalidates unchanged lazy candidate;3868661 RUNNING4m56,
53checkers passed, no candidate/AB acceptance. New2-case
overload_command_catalog.json is PROVISIONAL/UNEXECUTED preparation for the
remaining measured cross-command merged-view cost; no runtime/cache change
or extra job. Capture original histories after current FINAL only. Indexed
in HANDOFF/performance slice/test-library README.

CURRENT ONLY after3868661 SUBMITTED in ci1UtZtO/pin2ec5160e from empty
expanded queue.53checks/11focused+ALL616core/24original histories/72retained/
48serialAB, typed4b422d9e. Candidate not after-verified; no new speed claim.
Collect exact report before another runtime change; see newest HANDOFF.

LATEST completion before3868646 FINAL631f71ea:31checks/611compiled inventory/
4actual streamFAIL after correct setup/2controls/integrity. Valid constructor
is original-accepted and exposes missing type names too. Only then implemented
lazy candidate typed4b422d9e/patchcc397f42;4old test sections unchanged,5new
work/ownership tests. After53checks/616core/24original histories/72retained/
48serialAB prepared in ci1UtZtO, NOT accepted. See newest HANDOFF/performance
slice; previous before pending notes are historical. No rank escalation.

CURRENT ONLY before3868646 SUBMITTED from empty queue in6WMD97z2,
pin733ef336.31checkers/611inventory/expected4streamFAIL+2controls/fresh
valid-constructor companion; no runtime change or acceptance yet. Inspect
FINAL before fixing completion. Receipt indexed in HANDOFF.

LATEST completion3868572 FINAL2bd59d1d:23checks/4histories/integrity,
original accepted history+visibility differ; forgotten succ, lexical order,
type/member publication and failed-name ordering need repair. Types draft
is original-rejected >=/missing-! syntax, not positive acceptance. All8raw
goldens saved;4unchanged-runtime session regressions added. Before stage
6WMD97z2 being staged (31checks/611inventory/expected4FAIL+2controls/valid
constructor companion); inspect its FINAL before runtime changes. Existing
startup names are not all permanently active. See newest HANDOFF/performance
slice. Earlier pending3868572 notes below are historical.

CURRENT ONLY completion discovery3868572 submitted inBKVqvaZu/pinaae5e597,
empty queue before;23checks/4histories/8processes, no runtime change. Collect
exact report before original-golden regressions or completion implementation.

LATEST3868525 FINAL inspected after transport recovery, COMPLETED0:0in6m33cu033,
remoteFINAL0855b34bbeda9a1dce1f1ac408f1287fc8e0e075285e24c2aa5f85e12e4cf54c.
21checks/4controls/16on-off/source+artifact integrity PASS. U3010refreshes/
3.44million copied names/0queries cost0.742s; AV3658/4.90million/0queries
cost1.178s. Note-name scans only~5-7ms, not a major bottleneck. Next selected
target is unused completion snapshots; no runtime change.4completion_incremental
fixtures and23checker/8process discovery being staged. Source shows type/member registration and
first-LEXICAL-use versus first-definition ordering risks; both need original
capture. See newest HANDOFF; earlier running3868525 notes are historical.

CURRENT ONLY3868525 isolated post-view timers SUBMITTED in87kvHVtK,
pindd225daf, package d51196de, empty queue before.21checks/4inputs/8controls/
16on-off processes. No accepted result or runtime change; inspect exact job.
RootDatum source comparison and completion ownership/visibility checklist are
in loading-performance slice. All pending3868496 notes below are superseded.

LATEST profile3868496 FINAL24ab240d:22checks/6controls/4perf/finalintegrity
pass, exact accepted90source files. Four hash-matched samples isolate remaining
Type::equivalent11.25-12.73%, ladder bottoms7.25-10.55%, completion refresh
7.25-9.60% SELF cost. Main ladder caller is build_weyl_context, not eager-dual;
allocation attribution to TypeScheme/Value copies remains unproven. Completion
currently copies the full list before every command, unlike original query-time
table traversal. Next isolated timer patch e09e5fb0 prepared:21checks/4unchanged
inputs/8controls/16on-off processes. No production edit/next speedup or rank
expansion. See newest HANDOFF/performance slice; pending notes are historical.

CURRENT ONLY profile3868496 SUBMITTED from empty queue incKAa5VP7,
pin4da5e11b. Same accepted runtime as FINAL3868418;22checker tests,
6unprofiled controls/4perf captures. No accepted profile/bottleneck yet;
collect exact job before further runtime edits or new jobs.
Confirmed3m34cu033 RUNNING,22checks passed/release compiling. Additional
source-only allocation/folding boundaries are indexed in performance slice;
they are not measured bottlenecks or selected patches. Runtime unchanged.

LATEST3868418 FINALe6258384: overload-view reuse accepted.46checkers,
6focused+ALL607core,19original histories/72retained/48AB/finalintegrity PASS.
FullU4.630245->4.013848s (1.15357x), finiteAV8.737303->5.567626s (1.56931x);
RSS essentially unchanged. Still13.9034x/12.1786x original small end-to-end
time. Three shared Hodge errors retained, not mathematical passes. Next
same-source profile is being staged incKAa5VP7, not yet submitted at this
note. See newest HANDOFF/performance slice; older pending notes are historical.

CURRENT ONLY after R23868418 submitted from empty queue, tYeSNj3C/pinde34e3d4.
Same46checkers/6focused+ALL607core/19original histories/72retained/48AB.
No after acceptance yet; collect exact R2 before profile or further changes.
Partial9m56cu033:46checkers/compiled607inventory/six focused after tests PASS;
full607core RUNNING, no original histories/retained/A-B yet. Not FINAL.
Later10m47: full607core actually PASS (130.46s), release compiling; still
not FINAL. Original stored signature/degree and zero-shift matching paths
are traced in performance slice as unmeasured post-cache probe boundaries.

New source lead only: eager primal+dual classification in Rust inner-class
metadata versus original paired fibers/incidence. Explicit original dual()
still calls build; its Fokko-only DualTag constructor is NOT the interpreter
path. Await post-view sampling before selecting this work; retain actual
dual IDs/form ordering and budget/error histories. See loading-performance
slice "Further source audit"; no runtime change or extra job from this audit.

UPDATE3868400 FAILED before after tests at core compilation: the new shared
cache makes Analysis invariant; old MultiAssignmentThreader ties expression
borrow and table lifetime together. Report391933d8/logfcd8b4b9 retained.
R2only separates those lifetimes; typedabf07725/patch55ce3afb, all tests and
gates unchanged. Not a mathematical error; no cache speedup accepted. Fresh
after R2being staged, no profile submission before its FINAL acceptance.

HISTORICAL ONLY after3868400 submitted in skFysMpK, pincf4b3b00.
Candidate typed19c36c95;46checkers/6focused+ALL607core/19original histories/
72retained/48AB. Not verified yet; inspect exact job before further changes.
Partial7m23cu033:46checkers/before release PASS, core inventory compiling.
Same-source profile follow-up prepared, not submitted; requires FINAL first,
then22checks/2loading inputs/6unprofiled controls/4perf captures. See HANDOFF.

LATEST before R23868329 FINAL4a9ad6f9,18checks/607compiled/3actualworkFAIL
+3semanticPASS/3original histories/integrity. Only then implemented view
candidate typed19c36c95/patchd99f50b6, tests unchanged. Analysis-lifetime
read-only Rc views, both table identities guarded, fresh polymorphic trials
and mutation paths unchanged. After harness46checks/ALL607core/19histories/
72retainedstreams/48ABprepared, not yet submitted/accepted at this note.

HISTORICAL before R23868329 submitted alone inbXdYQHLR/pin72384af6.18checks/
3originalhistories/607inventory/6units; expected3workFAIL3semanticPASS is
not yet observed. Runtime unchanged; next after harness prepared separately.

UPDATE3868323 failed before Rust tests because the provisional null([row])
probe is rejected by both engines; report2fdffdcd. Exact draft retained as
third rejection fixture. Valid polymorphic controls now use #([row]); new
18checker/3history/6unit before R2prepared, no runtime change or math error.

HISTORICAL only3868323 SUBMITTED, tests-only overload-view gate inGwpfq3t3,
pin8ee94986.16checkers/2fresh original histories/607inventory/6selected
(expected3workFAIL+3semanticPASS), no runtime cache, not yet verified.

LATEST after3868252 FINAL2702e8ae: Cartan representative-only optimization
accepted in bounded rank1 retained scope.42checkers/34distinct selected domain
units/16freshCartan/72retainedstreams/48ABprocesses/finalintegrity PASS.
Full U12.11173->4.95661s (2.44355x), finite AV16.78998->9.48112s (1.77089x),
RSS~260MiB->43/52MiB. Still15.914x/19.203x original loading-inclusive time.
Next target repeated overload-view construction; tests-only6unit/2original
history gate prepared, no runtime cache yet. See newest HANDOFF/performance
slice; earlier pending notes below are historical, not current status.

HISTORICAL: after3868252 is the only submitted job, vL6gPF4k/pinba114c5d.
Probe-targeted Cartan candidate is NOT accepted until unchanged rank1 tests,
16Cartan histories/72retained streams/controlled48processes/final hashes.
Do not mix the separate overload-merge target into this frozen candidate.

LATEST before3868242 FINALe57c69a6:10checkers/519inventory/2workFAIL+
2semanticPASS/integrity. Targeted representative-only Cartan candidate now
implemented, patch80796a12; unchanged tests, exact orbit-count budgets and
dual provenance retained. After harness/package prepared, not yet submitted
or accepted. See newest HANDOFF. No production speedup claim yet.

CURRENT before R23868242 SUBMITTED inO6fhVGS1, pin92049ff2,10checkers.
R13868238 failed before build at tuple/list JSON metadata equality, not
mathematics. Original hashes/outputs and four Rust tests remain unchanged.

LATEST3868228 FINAL probe50c7fb30:12checkers/4controls/16observations/integrity
PASS. Partition6.11-6.19s dominates; repeated overload merges~1.44s U/~4.39s
AV are second. Probe overhead1.4-1.7%, no production speedup. groups.at E8
initialization is measured, not rank1 kernel work. ONLY before-work3868238
submitted from empty queue in8RzY9Nkb, pin8574c74f; tests-only4rank1 gates,
expected2workFAIL/2semanticPASS NOT yet observed. See newest HANDOFF.

LATEST presence3864029 FINAL, report9540131c43c901e7281914070ea3ef1a0006257b0fb73848846b0d44aaef4a46.
8checkers/601compiled inventory/2actual work failures+1semantic PASS,
two original histories and final integrity PASS. Then implemented ONLY
presence query at two sites; unchanged tests, typed30b24cae/patchb248916b.
After verification pending: no new speedup claim, no cache or rank expansion.
CURRENT: user requests precise attribution/probes before more optimization.
R2probe3868228 SUBMITTED as the only job inOHvh0QW6, pin
e6e817a91e92dcd87e23b43e29504f97c1aadf1cbbefab83d9d1de06fc684e1d.
R13868224 failed at a diagnostic JSON-format brace before any math/probe
run; failure9872491f retained. R2only fixes instrumentation, not mathematics.
Prepared isolated loading-cost probe splits script parse/type/evaluation,
per-name overload merges and per-datum Cartan/cache/partition phases.
groups.at initializes E8 etc even in existing rank1 fixtures; measure this
instead of calling the whole loading gap rank1 mathematics. Two on/off
repetitions and exact stream checks required; no accepted probe speedup.
Cartan3868167 FINAL12checkers/8positive+8bounds companions PASS, report
b8f60647a42644f3580e97ab755faa69d613b688b45c0645333fc3212a3bafb5;
loader prefixes still differ. Kernel tests/driver prepared, NOT run or
submitted. Probe first; see latest HANDOFF. No Cartan production rewrite.

UPDATE3866086 FINAL: correctness/integrity PASS but PERFORMANCE_NOT_ACCEPTED,
report95d23a47706b76e6548462c4835bf23f093e327a9b461bd789e8b5dbbd5095d2.
38checkers/65selected units/2fresh original histories/70retained streams/
48A-Bprocesses pass;601compiled inventory is not601executed tests.
Full U10.90781->10.86117s, finite AV14.86341->14.90503s: no effective
third performance repair. Still45.24x/38.30x original end-to-end time.
Next16input/12checker rank1Cartan-class capture prepared, not verified;
no Cartan runtime change or rank expansion. See latest HANDOFF/performance slice.

Historical: After3866086 SUBMITTED in87MXIyFj, pinf411a031c56847d44e3e78ecc9c8ea72624fda3119208097a1dd0fea4bcdbf46,
one job/empty queue; partial38checkers pass, release inventory compiling.
Cartan eager partition remains measured next target; old ON_DEMAND design
contains superseded budget/numbering/local-testing directives (see its new
current-status note), not a ready-to-apply specification.

LATEST profile3861850 FINAL, report7a2235c674eb882447fa4da65e5f6774b4171000fb7697e944877c47a4cf9f85:
22checkers/two unchanged loading streams/two exact-source frame-pointer
profiles/integrity PASS. Cartan partition23.64/18.00% and type equivalent
6.57/14.95% self samples (U/AV); merged-variant stacks remain. Next
presence-only query experiment is prepared, not implemented: three isolated
Rust tests, two original histories, eight checker tests. Preserve all
forget/shadow/rebind/error-priority contracts; no cache, rank expansion or
new accepted speedup. See loading_performance_rank1 slice.
Before3864029 SUBMITTED as one job inrFLDlK8B, pin7dbb74b2bd3975d732338cfd2e368cdc95fe04326ce54964ea8fc75756cf7e01.
Partial8checkers/two original captures pass; inventory/work assertions
pending. Keep diagnostic-category differences; no presence runtime change.

LATEST3858574 FINAL, reportac29df12a4d7286fe7cde4bd49d182edee9551fc088196c40053b5e6b5d13783.
Both full original repairs/38checkers/5coercion2session55type units/56retained
streams/12language/48timed processes/final integrity PASS.598is compiled
inventory, not598tests run. Same-node four-round medians: full unitarity
12.7165->11.2092s (1.134x), finite AV-ann19.4303->15.1464s (1.283x).
Still45.31x/37.66x original end-to-end time, ~260MiB unchanged; performance
remains the priority. Do not multiply speedups across different job nodes.
Frozen binary95cc29f2/sourceb32c06a6; mainb147208c has only a comment
correction, no further runtime optimization. Next same-source profiling;
presence-only query candidate/test inputs remain unrun and unimplemented.
The53positive retained cases and3shared original Hodge failures are not a
general mathematical acceptance claim. Earlier pending entries are history.
Post-repair profile3861850 SUBMITTED, ONE job from empty queue in aH4uDHA0,
pinf1fccec7c0bcaa0f437012230575604fce9f8037a2adf57dffbc976df61a027c.
Same frozen598source,22checks/two existing loads/separate sampling build;
no presence-cache implementation, new group coverage or new speed claim.

LATEST3856744 FINAL: six checkers/596compiled inventory/exactly2nullary
assertion failures/unchanged production/final integrity. Report33e8ffff875f0905892b0a8ee0cbd061ffa77118c3ae753c96fb7fc5a2b21e55.
Original recursive-row overload history is accepted; Rust exits139/signal11
in24.26s/RSS6207724KiB, not a timeout or accepted speed ratio. Added its full
golden and two tests before the one-function is_close repair. Local598test
candidate b32c06a6 now awaits after verification; it ports original equality
ordering, recursive-name boundaries and structural pruning. No cache yet.
Require complete repaired outputs, retained56rank1/12language cases and
same-node fresh-build A/B before another speedup claim. Prior pending entries
below are historical; see loading_performance_rank1 slice for current scope.
After3858574 is now SUBMITTED, ONE job from empty queue, stagejTo2AfSy,
pind0de137da82017f2266ccd7ab69074c3bbb5830d490fada5b93c1240d87515a0.
38checkers/598inventory/full retained gates/48timed processes; not accepted.

CURRENT R3candidate3856241 FINAL inVBs8Hh3A, pin3b3dc8de0b7c361b97c31a822657eed3db2371eec025ba90d01cb25f30d20e96.
Report4b234d258c6df5909f9f26a0b7a2e53d4ead9a18ec06ce9dd1a993e10128f877.
24checkers/both releases/expected before-and-after unit gates/final integrity
pass; all56rank1streams (53positive/3shared original Hodge failures) and
12language cases unchanged.594compiled inventory is not full594testPASS.
Four-round same-node A/B: full unitarity23.4033->12.4552s (1.879x), finite
AV-ann51.0667->19.0573s (2.680x). STILL51.71x/49.04x original end-to-end
time, RSS~260MiB unchanged. Continue small-rank performance; no rank expansion,
kernel/multicore speedup or general mathematical acceptance. See performance slice.
Original Git HEAD rechecked06:28:20Z and remains the oracle's7e1b958c.
Followup3856293 FINAL, stage i1cxrpAU, reportf4c27c536339a3de4ed0b47ab1ece8ec12618d8c76e508fa82182952b38e6454.
14checkers/two unchanged loading streams/two profiles/integrity PASS.
AV equivalent27.09%self/Cartan partition14.91%self; resolved stacks identify
merged_variants -> is_close -> equivalent. Not an inclusive total or speedup.
Original confirms void-overload replacement: Rust adds duplicates and emits
2Type ambiguities. Full original golden and two permanent tests added;
production unchanged. Before3856744 SUBMITTED inYyVCvrMg, pin
f297701754def664f638d622cc04ea63c3473d0e76084904b641591b54a37e96,
6checkers/596expected inventory/2actual assertion failures pending.
Original conversion table is linear too; structural pruning/ordered overload
storage are the actual source differences. See performance slice.
Followup sampling now uses a separate same-source frame-pointer build to
improve call-stack diagnosis. Accepted binary stays unchanged for timings/
discovery; full output equality required; diagnostic build/profile timings
excluded from speed ratios. Followup is FINAL; no local tests.
Additional source-only boundary: distinct nominal self-row types may recurse
indefinitely in Rust is_close because expansion precedes the original's
recursive-identity guard. Preserve overload_recursive_row_identity.atlas as
UNCAPTURED discovery, now submitted in3856744; it is not in3856293 and is
not a confirmed crash. No runtime change; original capture precedes repair.
UPDATE3856744 partial discovery now has original acceptance and Rust139/
signal11 after24.26s with RSS6207724KiB/empty stdout. Source-level is_close
recursion remains the caller hypothesis, not proven localization.6checker
tests pass, before-unit compilation/final integrity still pending. Preserve
signal/resource classification; no accepted timing or higher-rank claim.

R2candidate3856122 (84qj5jbF,pin80e594ab) FAILED at a harness-only empty
negative-payload condition after56unchanged math histories and10language
cases. All three engines correctly reject the eleventh input; Rust streams
match exactly. R3 retains full ordered recovery and diagnostic/exit checks,
adds six framing regressions and checkpoints failure rows. No benchmark or
final integrity in R2; preserve reporte267b48a. Prepared profile awaits R3.

Live3856122: expected3semanticPASS/1workFAIL before;4PASS/all55type/3coercion
after,594inventory and both release builds complete. Rank1 replays and A/B
still pending; node-visit reduction is not a measured process speedup.
Prepared loading_followup profiling is unsubmitted/unchecked and requires
this job's FINAL acceptance; no second runtime optimization or rank growth.
See the performance slice for the precise follow-up and architectural leads.

UPDATE3856060 stopped at a missing harness import before testing/building.
No candidate result. R2 repairs packaging only (classifier plus targeted
catalog/12unchanged fixtures and18checker checks); see performance slice.

LATEST3856033 FINAL: loading dominates small-rank process times. Type
validation52.80%/equality19.68%self profile samples identify the first target.
Local type-equivalence candidate732380b6 and four permanent tests are NOT
verified; preserve all constructor/alias checks and full before/after outputs.
See [performance slice](slices/loading_performance_rank1_2026-09-30.md).
Candidate gate3856060 is now SUBMITTED, not accepted: one job, same-node
fresh before/after builds,594inventory/four focused tests/type/coercion units,
56retained rank1 streams/12language cases/48timed processes. Preserve all
three shared original Hodge failures without counting them as passes.

PRIORITY OVERRIDE 2026-09-30: performance repair first, from small rank.
Pause new mathematical coverage/rank expansion; retain complete existing
rank1 outputs as regression gates. Separate loading/computation and profile
before changing runtime. Repeated controlled before/after measurements and
unchanged results are required. Default one HPC job, at most ten outstanding.
Single diagnostic3856033 submitted, not verified; no runtime change. See
[small-rank performance](slices/loading_performance_rank1_2026-09-30.md).

LATEST3856011 FINAL: all six rank1real/complex unitarity inputs match complete
original mathematical tails;30gamma inventories/46parameters/46final terms,
SIXactual empty compact-form sets, no nonstandard/zero rows.15checker/final
integrity PASS, no new runtime edit. Loader prefix differs; full-process Rust
22.6–22.8s vsoriginal0.23–0.26s is not an algorithm or parallel speedup.
Other numberings/infinite/higher-rank/Hodge/FPP/cycle coverage stays open.
See [rank1 form evidence](slices/unitarity_rank1_forms_2026-09-30.md).

LATEST3856011 SUBMITTED: one rank1 unitarity job across six original real/
complex forms, five complete gamma inventories per form, exact old SL2R
control retained. Fifteen checker/full outputs/integrity pending; no new
runtime/broad correctness claim. Complex rank1 real datum rank2 is explicit.
See [rank1 form expansion](slices/unitarity_rank1_forms_2026-09-30.md).

LATEST3856006 FINAL:15checker/integrity PASS; both frozen3855999inputs have
complete mathematical equality after correcting the ORBIT/COROOTS classifier.
No Atlas rerun or runtime edit. Four finite-module dimensions1,2,3,5/both
AV-ann routes/full characters plus old scaling control are accepted ONLY in
this bounded SL2R scope. Generic/nonzero-orbit cycle multiplicities and other
forms remain open. Report5c366f5c; both jobs complete/queue empty. See
[rank1 cycle evidence](slices/associated_cycle_rank1_anchor_2026-09-30.md).

LATEST3855999 FINAL: four finite SL2Rmodule anchors match complete original
math; old scaling control needs a checker-only reclassification, because
ORBIT_COROOTS was falsely counted as ORBIT. Keep raw reporta83efb1b and both
outputs;15-checker review-only3856006 is SUBMITTED, not another Atlas run. No generic
cycle acceptance or speed claim. See
[rank1 cycle evidence](slices/associated_cycle_rank1_anchor_2026-09-30.md).

LATEST3855999 SUBMITTED: one rank1 finite-module AV-ann/derived-cycle anchor
plus retained SL2R scaling control,12checker tests, exact verified binary;
not accepted yet. K_Nilpotent av only returns support indices, and arbitrary
Q-solution coefficients cannot replace the missing general fiber-multiplicity
map. Source audit and pin/receipt:
[rank1 cycle anchor](slices/associated_cycle_rank1_anchor_2026-09-30.md).

LATEST3855929 FINAL: SL2R full five-gamma unitarity histories AND old control
match complete original math tails;15parameters/15final constituents,9checker/
integrity PASS. Exact3855872binary reused. Actual nonstandard/zero/empty counts
are all0, so do not claim those mathematical cases covered. Other forms and
higher ranks remain pending. Full-process Rust23.643svsoriginal0.278s on this
sample includes imports; no algorithm speedup or multicore claim. See
[rank1 unitarity evidence](slices/unitarity_rank1_parameters_2026-09-30.md).

LATEST3855872 FINAL: ordinary full_deform recursion repair passes its rank1
gate,45positive complete math matches plus3retained shared Hodge failures,
2unchanged regressions/2retained A1units/29checker/release/integrity.590is the
compiled inventory, not a full unit sweep. Hodge cold/warm v=s identities and
complete coefficients restored; loader/diagnostic differences remain. Report
beeb3e280738d528460e1935c00861c8540afe37e89c59f04773ffb2cd817226.
Higher-rank and full Hodge/cycle validation remains open. Next capture is
the prepared SL2R full-parameter unitarity consumer, not rank2. See the
[ordinary deformation slice](slices/ordinary_full_deform_rank1_2026-09-30.md).
The next SL2R full-parameter unitarity capture is now3855929 SUBMITTED,
stageQh9dNMvS/pin0716191df1332366a4f27fcf9fe843c114a13624a5f56421aae59f7e9a983aee,
from an empty queue, reusing the verified candidate. Two cases/9checker tests;
not yet original acceptance or mathematical verification, no higher ranks.

PREPARED ONLY: [rank1 finite-cycle anchor](slices/associated_cycle_rank1_anchor_2026-09-30.md)
uses SL2 finite modules with dimensions1,2,3,5, both AV-ann routes and complete
character formulas. All share zero support but have different AC(M) point
multiplicities by a stated good-filtration deduction. It is neither submitted
nor a generic associated-cycle implementation/acceptance; nonzero orbits,
multiplicities across forms and exceptional groups remain open requirements.

CURRENT override:3855838 FINAL proves full_deform compactPASS/splitFAIL on
unchanged runtime,590compiled inventory/integrityPASS. A complete recursive
Split candidate now exists, NOT accepted yet. Preserve both unchanged goldens;
verify2regressions,2A1cache/alcove units,41foundation and7Hodge cases in one
rank1job before considering any escalation. No full590PASS or general Hodge
claim. Details/pins: [ordinary deformation](slices/ordinary_full_deform_rank1_2026-09-30.md).
After3855872 now SUBMITTED from empty queue (one job), stageSBSbHIxR/pin
95ea63398928cc04c1ccef04560b777b0ce1b24af7e5907be2144abd8701aec5.
No after acceptance yet; collect this job rather than duplicate or raise rank.

LATEST ordinary full_deform: original-backed bare3855753 compactMATCH/
splitMISMATCH after elementary3855719 confirms4Rust v=s failures.2permanent
session tests/goldens added, before3855838 pending on rank1Cartan candidate,
590inventory only. No runtime fix. Root cause is missing1-s AND missing
recursive child contribution; see
[full_deform repair gate](slices/ordinary_full_deform_rank1_2026-09-30.md).

LATEST rank1foundation3855559 FINAL41/41math matches on Cartan candidate,
15checker/build/integrity PASS, not higher-rank/F4/Hodge acceptance. New
rank1Hodge elementary regression3855719 submitted from empty queue with all
4prior inputs retained and7checker tests. No runtime edit or rank escalation;
see progressive slice and exact submission receipt before collecting results.

NEW rank1 Hodge ERROR:3855673 full bound4trace has original elementary
checks[true,true,true,true] but Rust[true,false,true,false], hidden before both
engines hit the shared trailing-zero array error. Separate mathematical
specialization regression/expectation added BEFORE any fix; reduced original
capture is next. See progressive validation slice; do not waive these checks.

NEW progressive rank1job3855559: ONE job only, empty queue at submission,
new conservative10-cap/uncertain-intent guard. Candidate build+41rank1 cases
pending; NOT repaired acceptance, no higher-rank submission. See
[progressive gate](slices/progressive_validation_2026-09-30.md).
Original GitHub HEAD remains7e1b958c as checked2026-09-30; no oracle drift.
Rank1 Hodge padding source hypothesis and pending minimal fixture are indexed
in that slice. Original SL2 array failures are not Rust-only failures; keep
complete old inputs and do not strip their coefficients to manufacture passes.
Hodge discovery3855673 now submitted separately on verified six-union;
unpadded control/padding failures/full SL2R traces4+12. Both current jobs rank1,
total2at submission. First1CPU/8GiB request was explicitly rejected and its
intent reconciled before fresh2CPU/8GiB stage. No math failure inferred.

USER2026-09-30OVERRIDE: pause submissions; maximum10jobs per batch/total
outstanding Atlas jobs, counting every array task and build/review. Inspect
small-group mathematical results before advancing rank one step; no prequeued
later-rank chains. Older bulk/quota100dispatch instructions are superseded;
full catalogs remain backlogs, not accepted coverage. See AGENTS hard rule8.

Cartan before3852364 FINAL1controlPASS/1actual streamFAIL on588inventory.
Local identity-map replacement now exists but is UNVERIFIED, no after job
submitted. Resume with small-rank gates under the new cap.640KGBreview3850878
FINAL640MATCH on old six-union, not on this new candidate. All prior goldens
and mathematical failures remain retained.

FINAL3851489:ALL640block inventories MATCH on current six-union. Exported
1114nonempty/1014zero-size original dual pairs, no missing original forms,
final integrity PASS. Next extend complete KLV per pair with explicit
quadratic resource/output budgets; no all1114KLV or higher-consumer claim.
See rank6_block_klv slice for exact report/pair manifest.640KGBreview remains
pending; metadata arrays must not be resubmitted. Cartan before3852364live.

Full408review3852258 FINAL (old572):289math matches/4mismatches/13Rustfail/
3Rusttimeout/2Rustresource/75originalfail/13originaltimeout/9rejection matches.
Do not relabel later repaired FPP/GL2/G2/PSp failures current. SO44/SL3R
unitarity and E6/E7/SO26AV-ann need exact current-source consumers; original
Hodge rejection frontier remains large. See
[full408frontier](slices/full408_review_2026-09-30.md).
Cartan before-regression3852364 is submitted, NOT yet verified or repaired.

Cartan3852252 FINAL originalACCEPTS all5673rank<=6permutation rows; Rust
complete output mismatches.11canonical controlsMATCH,7bare relabelled
diagramsMISMATCH. Complete original goldens and2permanent session tests added
(local588, old verified586); before-only gate required before production edit.
Reuse existing original-tie-compatible bourbaki_permutation; keep all old
F4class/Springer/AV-ann consumers.408review-only32GiB retry3852258 submitted.

FINAL3851780: ALL24fixed-gamma parameter grids MATCH;3original-accepted
F4AV-ann consumers FAIL on current six-union;1rejection discovery retains
diagnostic-envelope difference. FINAL F4trace3851976 pinpoints missing C3Levi
class, duplicate order6class, failed Kondo12signature lookup. Source defect:
Cartan_matrix_type always exports identity instead of Bourbaki-to-input map.
Reduced fixtures +5673rank<=6Cartan permutations added before repair; no
runtime change yet. See parameter/F4 frontier for complete evidence.
Old408full review3848053 OOM at2GiB; retry only unchanged reviewer32GiB/fat,
not mathematical arrays.640KGB/640block reviews remain pending.

NEW3851779/3851780:24complete fixed-gamma parameter grids plus3legacyF4
AV-ann replays/1rejection discovery,7-checker3851750FINAL PASS. Current
six-union still fails F4compact/rankone AV-ann (3851782/83originalACCEPT,
RustFAIL); first compactA1/rho3851803MATCH. F4class/signature reduction3851976
submitted, no runtime change. Prior required-solve wording is not a proven
cause: class_tables.at also requisitions a failed binary_lookup. See
[parameter/F4 frontier](slices/rank6_parameter_frontier_2026-09-30.md).
K_Nilpotent av returns support, not certified associated-cycle multiplicities.

LATEST272inventory replay3850467 FINAL264MATCH/8original rejections;
48old Rust complex-rank failures recovered,216matches preserved. Complete
complexA3/A4/A5/A6KLV scale cases FINAL MATCH (A6job3851324,5040rows). CPU130KGB
FINAL130MATCH; fat510/all640 remain separate. Block metadataALL640submitted,
final review3851489 pending; no remaining shard dispatch. See rank6_dispatch;
older pending/count snapshots below are historical. Higher consumers and
reducible/diagonal/singular/fractional coverage remain open.

Source audit identifies a concrete future A/B candidate: original raw_KL
retains block-owned kl_tab, while Rust recreates it on every call; dual_KL
is local in the original too. Rust already interns polynomial contents.
Do not implement/cache globally or claim a measured bottleneck from this
inspection alone. Ownership, pool-index and quadratic-output caveats are in
[rank6 KLV performance prerequisites](slices/rank6_block_klv_2026-09-29.md).

NEW CPU130KGB review3851146 FINAL:130/130complete graphs MATCH,1000nodes/
2904edges,15checker tests/final integrity PASS. Fat510not yet independently
reviewed. Complete complexA3/A4/A5/A6KLV scale array3851324 submitted with
unchanged full-output fixtures. Block metadata534submitted/106remaining;
see latest shard receipt and rank6_dispatch slice for resumable submission.

UPDATE2026-09-30: all three six-union rank6preflights FINAL PASS.272inventory
arrays3850465/66/review67 and all640KGB (review3850878) submitted. KGB smoke
scope10graphs=5841nodes/67960edges; block preflight10inventories/12full KLV
matches plus rejection recovery. Full640block metadata163submitted/477not
yet submitted due queue quota. Resume verified9-test quota-aware dispatcher,
not old all-at-once launchers; preserve all indices and interrupted-submission
intents. See [dispatch and coverage frontier](slices/rank6_dispatch_2026-09-30.md).
No new full-rank6, higher-consumer or calibrated performance acceptance.

LATEST six-union3850248 FINAL586core/515domain/all retained gates PASS.
Three new preflights submitted on this exact combined source:640KGB3850436,
272inventory3850437,640dual-block inventory3850448. Block-grid checker3850433
FINAL12PASS; original pairs include Rust failures/zero sizes, while original
failures/timeouts remain explicit incomplete coverage. No bulk acceptance
yet. See HANDOFF, [KGB](slices/rank6_form_graphs_2026-09-29.md) and
[KLV](slices/rank6_block_klv_2026-09-29.md).

LATEST3850366 FINAL:8block inventories and ALL12complete raw/dual KLV
blocks MATCH, full coefficient pools/order/cold-warm histories retained;
two rejected overloads recover with diagnostic-envelope difference recorded.
This is583/511eight-form scope, not640or singular/fractional/Hodge/cycle/AV-ann
acceptance. Next FINALsix-union ->640KGB/272replay and full block metadata/
per-pair extension. See [rank6 KLV](slices/rank6_block_klv_2026-09-29.md).

UPDATE3850338 FINAL:8complete block inventories MATCH,15dual pairs with
12nonempty blocks; original rejects both whole-tuple equality assertions
in KLV template. This is an input-contract error, NOT Rust math failure.
R33850366 uses original-supported component equality and retains full tables/
warm histories. Original Program-error overload envelope versus Rust Type
error is now checked explicitly with both failed function names. Pending,
not KLV acceptance. See [rank6 KLV](slices/rank6_block_klv_2026-09-29.md).

LATEST full-block KLV discovery3850338 SUBMITTED (8forms/all original
nonempty pairs; complete raw/dual/coefficient pools/cold-warm histories).
R13850300 failed a checker placeholder assumption before Atlas; only that
test is corrected. See [rank6 KLV](slices/rank6_block_klv_2026-09-29.md).
Graph R33850330 FINAL15checker tests PASS, including final-integrity fault
injection; use new R3package for640launch after live six-union3850248 FINAL.
No640math acceptance or new performance result yet.

LATEST capacity3849939 FINAL575core/515domain/all retained gates PASS;
six-repair union3850248 SUBMITTED (586/515required), not verified. All640
KGB harness R23850194 FINAL14checker tests PASS; R1syntax failure preserved.
272replay harness3850280 FINAL13checker tests PASS; no Atlas replay yet.
This does not execute640graphs. Next FINAL union ->10complete graph smokes
including complexA5/A6 ->640array+independent review; replay272inventory too.
See HANDOFF and [per-form graph scope](slices/rank6_form_graphs_2026-09-29.md).

Source-backed next KLV grid: original basic.at2038-2049 enumerates ALL
nonempty block(rf,drf) pairs using block_sizes and dual_real_forms. Freeze
per-pair metadata before computing raw_KL/dual_KL complete index matrices,
polynomial coefficient pools and length stops; include cold/warm histories.
The matrix is quadratic in block size: allocate/time-limit each pair
explicitly, never silently skip large pairs or call timeouts passes. Original
atlas-types.w9084-9155 uses Matrix(n), which matrix.h282 documents as IDENTITY,
so absence of an explicit diagonal loop is not a Rust diagonal bug.
K_highest_weights.at438-470 all_parameters_gamma can generate finite complete
parameter grids at a fixed gamma; regular/singular/fractional gamma and
standard/final status need original validation before higher consumers.
No new KLV/parameter fixture or mathematical acceptance yet from this audit.

LATEST five-repair union3849627 FINAL PASS on583core/511domain and every
full parent gate (26mathematical tails, all7unitarity,4complexFPP/no-loss).
Capacity R23849865 passes3original regressions but fails old `[0;8]` test
compilation; R33849939 fixes only that missed identity assertion and has
full575core/515domain PASS in live logs, final consumer report pending.
Exact six-parent586/515union driver is prepared, not submitted/verified.

Rank6 forms3850019 FINAL: original264accepted inventories give640form
presentations, INCLUDING48whose old Rust construction failed;8original
rejections retained.9checker tests and8complete KGB graphs (81nodes/280edges)
PASS on verified583union, not all640. Next after capacity/union gates:
full272metadata and640KGB arrays, then parameter-level coverage. See
[per-form KGB coverage](slices/rank6_form_graphs_2026-09-29.md),
[capacity repair](slices/complex_rank_capacity_2026-09-29.md) and HANDOFF.

NEW complex rank before3849785 FINAL: rank4PASS/rank5+6FAIL with exactly
two cap8Runtime errors,575inventory and unchanged production/integrity.
Width32 candidate3849865 SUBMITTED, NOT after-verified: same3original
regressions plus4wide-kernel tests,575core/515domain/full streams/no-loss.
Count budgets and rank<=8 u64 path unchanged. Isolated candidate excludes
five-repair union3849627; local586/515 is still unverified together.
First staging stopped before sbatch on old/missing bridge helpers; R2 pins
the exact FINAL PSp helpers rather than relaxing guards. Next collect both
gates, verify exact union, replay272 and extend per-form KGB/KLV/cycle/AV-ann
coverage. Details: [complex rank capacity](slices/complex_rank_capacity_2026-09-29.md).

Rank6 full272review3849376 FINAL:216inventory matches,48Rustcap8failures,
8original canonicalD4/D6quotient-u rejections; no mismatch/timeout. Real
156match/8reject; complex60match/48fail. Exact572, inventory only. Complete
reduced original rank4/5/6goldens retained; before3regressions next. See
[rank6 results](slices/rank6_validation_2026-09-29.md) and complex_rank_capacity.

LATEST PSp3849212 FINAL PASS; exact ParamPol/FPP/GL2/G2/PSp union3849627
SUBMITTED,583core/511domain/all parent complete gates required. See
[five-repair integration](slices/five_repair_union_2026-09-29.md).
New rank6 observations expose WeylElt=[u8;8] rejecting complex ranks5/6
(real datum10/12), despite original acceptance. Four bare/control/rejection
fixtures in complex_rank_catalog precede any repair; checker17 is separate
from frozen union16. Keep element-count budgets; audit representation width,
identity arrays, u64 fast path and root-permutation encoding before a change.
Reduced discovery3849710 FINAL: original3positiveaccept, rank4fullmatch,
rank5/6eachRustRuntimecap8; wrong-size rejection/recovery works but diagnostic
wording differs. Freeze goldens/before regressions next, no runtime edit yet.
See [complex rank capacity](slices/complex_rank_capacity_2026-09-29.md).

NEW rank<=6 directive: separate272-case simple real/complex inventory, including
all intermediate simple central quotients and unequal D-even classes; complex
rank<=6 deliberately reaches real root-datum rank12. Nine harness/eight oracle
smoke checks PASS, all eight complete math tails equal (24form presentations).
CPU64cases3849374/fat208cases3849375 and review3849376 are SUBMITTED, not
full272accepted. This is exact572 discovery, not high-level acceptance
or validation of local583union. Products/tori/diagonal quotients and systematic
per-form representation tests follow. See
[rank6 validation](slices/rank6_validation_2026-09-29.md).
G23848845 is now FINAL PASS, reportc28f26b28e56ebe78fcf955dcac5e837c99111405ee1fb17071304666090bdf9;
keep isolated573scope and require merged validation with other repair parents.

PSpbefore3848989 FINAL2pass/1expectedfail with3exactRuntime diagnostics;
equal-locator relative-shift candidate3849212 now submitted. Same tests/
goldens/575core/511domain/all5histories/no-loss/integrity gates, no cache
clearing or relaxed links. Pending, not accepted. After G2/PSp gates finish,
verify the exact union with FINAL ParamPol/FPP/GL2 before broad replay.

PSp R23848939 FINAL provides a bare failing shared-block history, while
uncached partial blocks still match. New3stream before-only3848989 requires
2controls pass/1history fail with3exact Cayley Runtime diagnostics. Suspect
equal-locator merge shortcut wrongly assumes zero shift; original always
computes the overlap modifier. No production edit until proof. See indexed
psp4_cayley_history slice and HANDOFF for full goldens/pins.

PSp cold/triple-warm3848885 FINAL both match complete original streams;
full script history still fails. Do not repair the cold generator on this
disproven premise. R2all-term/isolated-unitary histories3848939 submitted.
Details: [PSp Cayley history](slices/psp4_cayley_history_2026-09-29.md).

GL23848801 FINAL PASS restores its complete unitary companion, retains six
controls and passes full574core/511domain/6GL2streams/no-loss/integrity.
Not verified together with FPP/ParamPol/G2. NewPSp cold/warm reduction3848885
is pending, checker16, no production edit; master273/full408 unchanged.

G2before3848829 FINAL:1exact Runtime failure after setup/recovery,573inventory,
source unchanged. Cross-call-only repair3848845 submitted; full original
stream/G2live trace/573core/511domain/no-loss/integrity gates pending.
Next PSp reduction must preserve actual adjoint C2coordinates and cold/warm
history at x6lambda[4,3]/2,nu[28,21]/6; do not relax downward-Cayley closure.

UPDATE FPP3848729 FINAL: all4complex FPP tails now match original on
isolated574repair, with full574core/511domain/4product/no-loss/integrity gates.
Not merged localunion/full408. G2/PSp3848811 FINAL accepts all3original
inputs and rejects all3inRust. G2before3848829 freezes full original golden
and one new573regression; wait for exact failure proof before cross repair.
PSp's downward-Cayley guard is a separate frontier. See HANDOFF and slices.

LATEST exact572 CPU187review3848054 FINAL:166math matches,6rejection matches,
5Rust failures,9original failures,1timeout.22form KGB/KLV/AV-ann samples all
match; same132inputs gain19matches/no losses. Fat221/full408 remain pending.
GL2before3848755 verified2failures; distinguished-only repair3848801submitted.
Ordinary G2deformation now needs regression proof for concrete
PartialBlock.cross(row,generator) versus inherent(generator,row) mismatch;
PSp4RCayley interval failure remains separate. New3case reduction capture
uses checker15, no ordinary-deform runtime edit. Read HANDOFF before reuse.

LATEST3847661 FINAL verifies the isolated575 ParamPol comparator and complete
three cycle traces/PHI_ORBITS, not general associated-cycle correctness.
GL2R remains wrong;3848669 now supplies accepted bare-core full goldens and
two new regression units, before-only574package excludes FPP/ParamPol edits.
FPP3848594 is FINAL HARNESS FAIL after2regressions/574core/511domain/products:
old reader omitted real_form_grids. Fresh3848729 carries the verified reader
and3catalog checks, unchanged math sources/goldens. Local579union is unverified.
Next: collect GL2before failure proof; audit distinguished twisted KL actual
common-block row/modifier/singular flags without weakening halving; collect
FPP and independent408reviews later, never block on or duplicate live jobs.
See newest HANDOFF and indexed slices. Older statuses below are historical.

LATEST: full408 CPU3848051/fat3848052 and independent reviews3848053/3848054
submitted after fresh build/preflight. FPPafter3848594 is a narrow574candidate
following proven2before failures, not yet accepted. GL2R3848183 proves the
zero-scale twisted KL sum incorrectly EMPTY (original1*p); ordinary KL is
correct. Add bare-core original-backed singular-column regressions before
repair; never relax coefficient halving. See newest HANDOFF and unitarity/FPP
slices. Keep these candidates separate from ParamPol575 and localunion577.

GL2Rbare-core positive/rejected companions now submitted3848639, catalog4
retains both original script failures; original acceptance is provisional.
FPP3848594live focused regressions2PASS, full after gate remains running.
ParamPol3847661last confirmed live in unitarity phase after prerequisites.

CURRENT merged3847093 FINAL passes full511domain/572core/prerequisite and
reduced mathematical gates. Six final-constituent unitarity companions now
match; GL2R retains Inexact halving. Fresh408build3847662/preflight3847663
submitted; only launch arrays after both gates pass. Separate575ParamPol
after gate3847661 is submitted, not verified. FPP4capture3847592 wholly
accepts valid8type product input in both engines but outputs differ; leading
T1.A2adjoint constructor failures are separately retained. No FPP repair yet.
Read newest HANDOFF and indexed ordering/FPP/unitarity slices for exact pins.
Older pending/next-action entries below are superseded chronological records.

Next FPPgate3847713 proves the two new complete-stream tests fail before
repair, isolated574source without ParamPol. Whole positive/negative goldens
are retained; no FPP runtime change.408preflight3847663 is FINAL PASS;
release3847662 remains live. GL2R remaining error is after scale0finalize,
before twisted deformation's progress message: reduce fixed KL coefficient
halving without relaxing parity. See unitarity_final_constituents slice.

Before3847509 FINAL proves3ParamPol order assertions fail; comparator-only
repair prepared with unchanged tests, not after-verified. Its full gate
awaits accepted merged3847093; keep575candidate separate from572foundation.
FPP product ordering has a distinct root_components min-versus-max RootNbr
ordering hypothesis and original-capture fixtures; no FPP repair yet.
See [FPP product frontier](slices/fpp_product_order_2026-09-29.md) and HANDOFF.

CURRENT merged R4job3847093 is running. R3import failure is preserved;
staging now inherits complete R2harness and pins transitive dependencies.
Ordering3847089 accepts valid A2/T3/T1positive streams and confirms both
comparator errors. Three original-backed tests added, no runtime change;
before-failure gate submitted separately. Next after-fix/full consumers,
not output normalization. See [ordering evidence](slices/parampol_order_2026-09-29.md)
and HANDOFF for source/test-count separation and exact receipts.

Current merged gate is R3job3847086, notfailedR2: R2completed all units,
273capture/11diagnostics with no losses, then incorrectly required unique
SET_BITS/FACTOR_WEYL_ORDER loop labels. Four new marker regressions and
exact10/9inventories preserve full first-to-end comparison; runtime unchanged.
See HANDOFF. No408submission before accepted FINAL foundation.
Ordering3847067 identifies provisional-input type errors; retain negatives,
capture explicit-Split valid companions before touching the comparator.

Newest cycle replay3847046 FINAL supersedes the missing-Cayley-pair next
action: root-sign repair restores all reduced A2terms/final KTypePols;
complete intermediate tails remain unequal ONLY in ParamPol term order.
Capture3847067 now tests packed torsion and descending gamma separately,
original/exact566, checker12. No sorting runtime edit yet. Preserve full
failures and original-backed goldens; see indexed associated_cycle_frontier.
All408 merged replay workflow prepared in hpc/stage_merged_math.py, NOT
submitted; first require FINAL3847039, fresh build and63-test preflight.

Merged gate R2 now SUBMITTED3847039, same exact union and full gates,
verified bridge861b1e48; read HANDOFF and R2receipt. No math acceptance yet.

LATEST merge3847021 passes511domain/572core/build, but fails OLD bridge's
six-test log guard after all10actual checks pass. R2 explicitly stages the
verified configurable bridge; keep this failure and all math unchanged.
Cycle trace3847030 now locates A2loss inside coherent continuation on the
disconnected sl2R.gl1R cuspidal Levi, BEFORE real induction/K-type conversion.
Next compare Cayley_sum/cross and replay the existing root-sign fix; see HANDOFF.

Full292-form survey3846278 is FINAL:164match/21mismatch/47RustFailure/
57OracleFailure/3OracleTimeout, exact566. All36KGB/36metadata pass but
high-level coverage remains incomplete; see real_complex_forms table.
Merged511domain/572core gate3847021 and reduced A2induction2probe3847027
are now submitted in separate immutable stages. Read HANDOFF for exact pins.

CURRENT: torus572R3job3846731 and root-sign CLI3846760 are FINAL PASS within
their own scopes. Exact-union511domain/572core/CLI/273+11+4 gate now prepared;
read HANDOFF and math_verified_merge_submission_2026_09_29.json before acting.
Full408 consumers must use that verified merged source, not either parent.

Seven final-constituent unitarity companions3846973 all ACCEPT originally;
SU21/PSU21/Sp11 and GL2R remain original-accepted Rust failures. A2Phi trace
first differs at theta_induce_standard(param(q),G).K_type_pol, after matching
twisted input and rho shifts. Preserve all terms and independently reduce
induction versus conversion; never manufacture missing coefficients.
See [unitarity companions](slices/unitarity_final_constituents_2026-09-29.md)
and [cycle frontier](slices/associated_cycle_frontier_2026-09-29.md).
Older submission entries below retain chronology and are superseded above.

UPDATE root-sign kernel3846740 FINAL PASS (3beforefail/3afterpass/511domain/
integrity). Consumer3846760 now verifies four complete original CLI block
histories on exact kernel-after; separate torus572R3 and future merge pending.
See [root-sign evidence](slices/positive_negative_roots_2026-09-29.md).

Pos_neg kernel3846740 now freezes three regressions and the empty-set repair
on exact569parent, requiring all511domain tests. It is not torus572R3 and
not CLI acceptance; merge only after both are independently verified.
See [root-sign/common-block repair](slices/positive_negative_roots_2026-09-29.md).

LATEST: torus R2after2pass/1full-output failure; R3job3846731 also canonicalizes
RootDatum torus order, preserving all goldens. Separate PSp block4capture
3846724 proves direct generator and full cross-link errors. Source pos_to_neg
starts with all positive roots instead of original empty set; add independent
identity/simple/word-image and original PSp link regressions before repair.
Read HANDOFF for frozen pins. No candidate is generally accepted yet.

Candidate5723846692 is now submitted: torus-factor exclusion and duplicate
bitset union, after original11capture3846570 accepted all9positives and
retained2negative streams. Frozen three before/after regressions and full
508domain/572core/273capture/11consumer gates; no after acceptance yet.
See [torus factors and bitsets](slices/torus_factors_bitset_2026-09-29.md).
First gate3846643 correctly failed all three before tests but its panic-log
parser rejected Rust's numeric thread IDs. R2 only repairs that parser; no
after result exists yet and all original tests/goldens/patches are unchanged.

Latest foundation5693846487 is FINAL PASS (reportf56c0ee29d2c5a2a020b0454b5028c08fd24ad40efcc4ecc9b78b0b586e22036):
508domain/569core/CLI/273capture/no losses and original-backed fundamental
vectors/errors/historical fixture. Four-A1xA1-orbit assertion restored;
general cycle/Phi equality remains open. Next scoped repair: simple_factors
must exclude tori; to_bitset must use idempotent union. Supplemental11probe
3846570 adds direct factor/order_W evidence to retained8diagnostics.

## Latest real-form errors and fundamental candidate — 2026-09-29

Exact566CPUreview3846279 FINAL132:100math matches,1PSp4R KLV mismatch,
22Rust failures,9original failures. Full160fat cases are not reviewed yet.
AV-ann negative-set-bit assertion reaches fractional SL2R and rank-zero tori;
reduce it before repair. Coordinate/isogeny-sensitive unitarity also fails.
See [real/complex evidence](slices/real_complex_forms_2026-09-29.md).
Reference2733846359 now accepts two full corrected fundamental probes in
original and rejects unchanged566Rust. Candidate569job3846487 passes live
before3fail/after3pass; full gates still pending. Exact rational actual-lattice
formula and signed32/discard checks replace old coordinate shortcuts. See
[fundamental repair gates](slices/fundamental_lattice_2026-09-29.md).
Supplemental8diagnostics3846542 (separate catalog, not master273) now probe
AV-ann stages, cold/warm adjoint partial blocks, duplicate bitsets and valid
actual-lattice FPP inputs. No production fix for these yet. Current CPU1core
QOS rejects8G and accepts4G; failed stage retained. See indexed real/complex
slice for job/pin and inspect all original outputs before changing semantics.
UPDATE3846542 FINAL original accepts all7positives: to_bitset duplicate error
confirmed; AV-ann first fails character_table(T1), with source mismatch in
simple_factors wrongly including T. Add direct factor/order_W regressions,
then repair with full before/after gates. Cold/warm PSp4R is history-sensitive;
complex FPP displayed two middle product terms swap order. See indexed slice.

## Real/complex group expansion — 2026-09-29

User-requested36 named forms x8 operations plus4 large-rank inventories adds
292provisional cases to the retained116. See
[real and complex forms](slices/real_complex_forms_2026-09-29.md) for source
semantics, groups, ranks, coverage limits and frozen HPC submission gates.
R3deform3846153 now FINAL508domain/566core/265capture, no positive losses;
its exact release3846235 and63-test survey preflight3846269 are complete.
New292cases submitted CPU3846276/fat3846277, full/CPUreviews3846278/3846279.
No mathematical acceptance follows from submission; no runtime fix this turn.
Original271 fundamental positive probes reject bare-core ratvec*vec setup;
preserve them, add valid companions before using them as numerical goldens.

## Newest verified frontier — 2026-09-29

LATEST concrete cycle root cause: original267trace3846161accepts the full
program; Rust loses A1.A1Levi[1] because fundamental_coweight ignores its
ambient embedding. fundamental_weight has the companion e_i shortcut.
Master271/reference3846184 adds span/duality/embedded/negative coverage;
no fundamental production repair yet. See
[fundamental lattice coordinates](slices/fundamental_lattice_2026-09-29.md).
Separate R3deform3846153 now passes7regressions/508domain/566core and CLI
release; full265capture/final guards still required before accepting it.

NEWEST: exact559CPUreview3845923 FINAL49math+3language matches,7rejections,
4math mismatches,4Rust failures,1independent-invariant failure,18original
failures (SHA8130c40a345001b57e9803642fe38de662b3dbd99a02690741430f4f750cac37).
B2/C2/D4AV-ann newly match; no prior match lost. Cycles A2/B2/C2/G2 execute
but differ, while A1.A1complex-orbit count3rather than4fails the independent
checker. Keep these exact high-level failures; full116review remains open.
The A1.A1raw payload specifically omits H[0,1]; A2instead loses Phi terms
and changes matrix entries. Master267adds two reduced original-discovery
traces before repairs. See the indexed
[associated-cycle frontier](slices/associated_cycle_frontier_2026-09-29.md).

R2deform3846124 FINAL FAIL:4deform and2orientationpositive regressions pass,
but rejection/discard test fails. Correct the wrapper's BuildAndDrop policy
to Skip and make_dominant error translation; do not change the2error/full
survivor expectation. Runtime active orientation is promising, not accepted:
full508domain/566core/CLI/265capture and116consumers remain required.

R2orientation3846059 FINAL265 proves G2orientation low by1 on fractional
partial-block rows and wrong nonstandard acceptance, with historical anchors
retained. Current UNVERIFIED566candidate ports the active dominant/complex-
descent/floor-parity algorithm once in RepContext and delegates orientation_nr.
Seven original-backed regressions and full508domain/566core/265 gates are
required; exp_i and all mathematical invariant checks remain unchanged.

Deform3845869 FINAL FAIL: before4fail, after4fail. A2/B2/C2whole-stream
assertions omit the original loop Value row via a reports-only collector;
G2 independently violates exp_i parity. Preserve all goldens/invariants;
the local collector repair still needs an HPC rerun. Master263adds
orientation discovery across A2/B2/C2/G2 plus historical A1/A2 controls,
before changing duplicated inactive-branch orientation code. See the
[ordinary deformation slice](slices/ordinary_deform_common_block_2026-09-29.md).
Exact559matrix build/preflight now PASS with1473file equality, and116math
consumers are submitted3845920/3845921 with reviews3845922/3845923.
No downstream cycle/AV-ann or deformation correctness claim yet.

## Latest reviewed mathematical frontier — 2026-09-29

FINAL matrix3845829 passes559core/CLI/final1473source and full257replay:
117whole positives/no losses,8negative errors and survivor stdout validated.
Complete report96269f8c3645ca40441d00bd258056f4f9cc9852a1357e74b4bb782cb6dd53a8.
Its exact release3845871/preflight3845872 are submitted; require their full
source equality before116math consumers. Cycle/AV-ann acceptance stays open.
Ordinary-deform563candidate3845869 is submitted separately after4original
positive captures/4old552panics. Require unchanged559before/563after proof,
all563core/CLI/261capture, and retained parent positives. See HANDOFF for pins.

Matrix repair3845829 is submitted after original R2accepted both whole
positive companions. Requires4before failures,559core/CLI/frozen257capture
and no positive losses; no downstream116math acceptance yet. Parent boundary
3845705 is FINAL556core/251capture/5syntax fixes with115whole positives retained.
Next source mismatch is ordinary deform's obsolete full-block approximation;
see [deform common block](slices/ordinary_deform_common_block_2026-09-29.md).
Master261adds4original-discovery probes; no ordinary-deform runtime edit yet.
Those probes run in3845837 against exact552/original; matrix3845829 retains
frozen257inputs and has reached after compilation following before failures.
UPDATE:3845837 FINAL gives4original acceptances/4old552panics. Four new
whole-original-stream units and an unverified common-block wrapper candidate
are local; its563build requires the pending matrix559final gate first.
Matrix3845829 passes3after units and559core; complete CLI/capture still open.

Exact552CPU3845474 final:46math+3language matches,7rejections,12Rust failures,
18original failures; nine new matches/no losses vs536CPU. Full116fat/review
remain separate. Prioritize the indexed matrix-axis defect, then the distinct
B2height-parity/C2weight-integrality/G2lambda-rho unitarity failures. F4AV-ann
has a separate solve failure and must not be assumed repaired by the axes.
See `slices/matrix_index_axes_2026-09-29.md` and `MATH_VALIDATION.md`.
Matrix3845716 original-positive R1 instead rejects unloaded script assert;
retain it. Master257original/exact552capture3845762 is now FINAL with
builtin-only and historical2x2companions accepted. Obtain unchanged-before/after
evidence before claiming any repair.

## Latest mathematical frontier — 2026-09-29 continuation

NEWEST552build3844955/61-test harness3844956 PASS with1459source equality.
All116math now3845471/3845472; independent reviews3845473/3845474 pending.
Live mathematical progress includes A2unitarity/AV-ann and scriptKLV cases,
but actual matrix-axis failures now affect A2cycle/B2AV-ann, and B2unitarity
has a separate height-parity failure. Read
[matrix axes](slices/matrix_index_axes_2026-09-29.md); new asymmetric positive/
negative fixtures extend master255, no matrix implementation change yet.
Original253arrow3845481 confirms true arrow-print expectation and primitive
void acceptance. Boundary556candidate is separate: group-root grammar and
virtual alias-spec newline recovery. Before3845482failed a panic-log guard
AFTER1pass/3intended failures; exact-test checker rerun3845638 precedes after
gates. Keep these source/catalog scopes separate.

CURRENT: recursive552 foundation3844656 is FINAL (552core/CLI/1459sources,
114whole positives in245capture, all four high-level imports load but stdout
differs). Exact standard build3844955/preflight3844956 are submitted; full116
consumer arrays await their verified reports/source equality. Explicit
CPU6/fat24GiB replaces the old hidden6GiB child cap on fat; same cap for
both engines, preflight required, no cross-limit speedup claim.
Same552followup3844760 FINAL251 has no positive loss and one additional whole
positive match; SHA d8f1d7fabcb30c80460e630b1c581fd05c80375f28f291d0e1038d77e6a87c27.
Its five negative cases remain wrong-phase/recovery regressions. Four new
TEST-ONLY session checks await HPC. Master253 arrow/root fixtures have no
original capture yet; the matching whattype stream is NOT printer-failure
evidence. See generic_recursive_groups slice and current HANDOFF.

LIVE544job3844249:five before failures -> five after passes; all544core PASS.
CLI/release/243/final report still pending. Original R3job3844285 FINAL245:
full local/captured/user-op A1/A2/B2/G2 positive accepted; new negative stream
retained. Still needs fresh544comparison, not acceptance via old523discovery.

Polynomial544job3844249 RUNNING: all five before assertions execute and fail;
after compilation/full544/243 gates still open. Master245is a separate newer
discovery inventory; never transplant its checker into frozen243stage.

Live-destination3844021 is FINAL:539core/CLI/1442sources/241capture PASS,
107whole positives/no losses. Report69ed01a390e698b8702f85a6ca5ede3603a01af3e9dfb7ead8474863a10eff21.
Polynomial coefficient writes/transforms are now implemented in an UNVERIFIED
candidate with five new regression tests (expected544core), not accepted.
The hidden-index-let fallback preserves original dynamic/identifier ordering;
foreign-owner writes deliberately reject original key corruption. No generic
recursive-group implementation or high-level acceptance. See polynomial slice;
older unimplemented/running notes below preserve the preceding state.

LATEST transaction3843795 FINAL:538core/CLI/final1440source and238capture pass;
106whole positives retained/no losses, two exact rollback stdout matches.
Report0284810cbf6801ed922d975b2c36840156947eb3a6f8735af3c12b26f7f5b0f4.
Discovery3843926 confirms wrong nested component values([7,2]vs[7,7]); repair
3844021 has an executed before failure, after/full539/241still required.
See [live destinations](slices/component_assignment_live_target_2026-09-29.md).
Master243write-contract discovery3844074 is separate from frozen241/238.
All116exact536consumers submitted3843929/3843930, reviews3843931/3843932.
CPUreview3843932 FINAL86:40matches(37math),7rejections,21Rust/18original
failures; full116review still pending. Live539has before-fail/after-pass and
all539core pass, but CLI/241capture/final report remain open.
R2write discovery3844074 FINAL243 establishes dynamic transform ORDER21,
bare-identifier key rereading and nine finality/type/reversal errors; see
polynomial slice. No polynomial runtime write implementation yet.
No coefficient-write or generic-recursive runtime support yet. The next
paragraphs retain earlier intermediate snapshots.

Final guard3843259 passes536core/CLI/final1436source integrity and retains
106whole positive matches in234capture, report68bec0c7c7750fb56a9d776378b8584958f50ca2e60ad5fe40b83ee1ed2600b4.
High-level libraries still fail generic recursive groups and polynomial
coefficient writes; this is NOT unitarity/Hodge/AV-ann/cycle acceptance.
Transactional grouped publication3843795 is running: both regressions fail
before and pass after, but full538core/238capture remain required. Parser/SCC
work is separate. Master241adds Hodge chained-write/live-destination and
foreign-owner diagnostics before any coefficient-write implementation. See
[polynomial coefficients](slices/polynomial_coefficients_2026-09-29.md) and
[recursive groups](slices/generic_recursive_groups_2026-09-29.md).

## Mathematical validation against latest original — 2026-09-28

LATEST while guard candidate3843259 RUNNING: four unchanged before/after
regressions, full536core and frozen234capture. Original3843183 accepts entire
mode/nested positives and preserves six negative controls; R1invalid scoped
DoExpr remains retained. Static depth and runtime unwind are both repaired in
candidate only, no verified after-pass yet. See while_guard_break slice.
NOW unchanged before3fail/1pass -> after4pass plus full536core all pass.
CLI/234capture/final integrity remain open; no high-level acceptance yet.
Subgroup532standard build/preflight3843161/3843162 PASS; all116consumer arrays
3843196/3843197 and reviews3843198/3843199 submitted. These do not test the
later guard repair. Master236recursive atomicity/SCC discovery is separate.
Subgroup CPU3843199 completes86cases:40matches(37math),7rejections,
21Rust failures/18original failures; full116review3843198 remains open.
Discovery3843306/Tl10Axe9 tests nongeneric duplicate-field rollback and function
collisions, plus anonymous recursive components and allowed external-constructor
boundaries. See generic_recursive_groups slice; no generic runtime edits yet.
3843306 now COMPLETE: old Rust wrongly accepts both nongeneric definitions
and poisons later bindings; original rejects atomically. Component positive
has original alias-tag ambiguity and >= setup failures, both retained.
Master237adds the corrected companion; see the indexed generic group slice.

CURRENT subgroup3842990 COMPLETE:532core/CLI/final1430source gates pass,
104whole positives/no losses in228capture. Minimal/broad subgroup streams
equal diagnostic original, while unmodified-original mathematical errors stay
explicit. All four high-level imports now fail W_orbit.at329 at a break in
the while condition; AV-ann also retains lazy_lists.at5. Exact-source release
3843161/preflight3843162 submitted; all116consumer gates still open.
Three before-repair guard fixtures extend master230to233; read the indexed
slice [while guard break](slices/while_guard_break_2026-09-29.md).
Earlier running/unimplemented notes below are chronological history.

Latest original causal3842817 proves the ignored-Gens defect: two forwarding
fixes pass minimal empty subgroup and entire broad orbit/witness assertions,
while all nine negatives remain byte-identical. Patched original remains a
diagnostic only. New Rust subgroup candidate adds four overloads and seven
tests; full532core/228capture3842990 at MKxzyGsd is RUNNING, not accepted.
Its live7new/all532core/CLIcheck/release results pass; full228capture and
final integrity/report are still required.
See slices/weyl_subgroup_orbits_2026-09-29.md for source pins and proof boundary.
Tagged525 CPUreview3842500 retains40matches (3language),7rejections,21Rust
loader failures/18original failures; full116review3842499 remains separate.
Generic mutually recursive type groups in lazy_lists.at5 remain independent:
new230original discovery3843104 COMPLETE keeps228oldcases, accepts entire
finite-stream fixture and rejects all7intended errors with rollback/survivor
values intact. No group runtime edits; see generic_recursive_groups slice.

Original3842475 itself violates the empty-subgroup orbit invariant (A2 input
[-1,-2] becomes[2,1] while witnesses are identity). Source header ignores g.
Keep this mathematical failure, add the minimal regression in228master, and
run the separate diagnostic forwarding patch at ZwXZpKGH; do not silently
port the bad result or relabel a patched original the oracle. Runtime subgroup
implementation remains open. See weyl_subgroup_orbits for full evidence.

Canonical86CPU review3842130 completes40math/7reject matches,21Rust failures
(old suffix loading),18original failures; no mathematical successes lost.
Tagged exact-source release3842314/preflight3842315 PASS; ALL116arrays
3842497/3842498 and reviews3842499/3842500 submitted separately. Subgroup
R3discovery3842475 keeps227inputs, replacing neither rejected fixture; its
new companion uses supported unary vec negation. Runtime subgroup port open.

Subgroup discovery3842256 captures all nine original runtime negatives but
the positive uses invalid bare adjoint(RootDatum).226companion3842448 fixes
only the fixture constructor to adjoint(LieType,bool), preserving every
mathematical check and the old failed source. Runtime subgroup port remains
open; see weyl_subgroup_orbits. Tagged release3842314/preflight3842315 are
submitted separately from canonical116arrays; do not transfer acceptance.

FINAL tagged3842065 passes525core/CLI/1426source/full required223streams,
104whole positive matches/no prior losses; diagnostic differences and
original signals retained. New frontier is FOUR missing subgroup variants
of Weyl_orbit/Weyl_orbit_ws (existing names hid missing signatures), plus
lazy_lists.at5 recursive-type syntax on AV-ann. Original225discovery3842256
is submitted; runtime orbit implementation NOT yet made. See indexed
[subgroup orbit slice](slices/weyl_subgroup_orbits_2026-09-29.md).
Canonical exact-source release3842063/preflight3842064 PASS; ALL116cases
CPU3842127/fat3842128 and reviews3842129/3842130 submitted. Tagged release
rI7iffCH is separate; no full consumer or fresh parallel acceptance yet.

FINAL3842014 passes508domain/523core/CLI/integrity and all expanded canonical
metadata outputs,102whole positives versus99/no losses. Captured SC D4/D6/D8/E7
representative mismatch is repaired; full116consumers/parallel still required.
Standard release stage DNq0OCvS is being submitted. New UNVERIFIED tagged-case
525core/223capture candidate at pqwSIJ7O follows original3842032's four false
acceptances and two error-priority cases. See canonical_seed_sections and
tagged_case_suffix slices; current local core is newer than verified3842014.

LIVE3842014 passes508domain/523core logs; release/capture/final integrity
still pending.222suffix isolation3842018 now establishes complete accepted
ordinary-expression original output and a matched subject-stop while control;
bound prefix/suffix and discard while cases signal originally. Keep these
separate, never use a successful prefix as a whole golden. New223discovery
adds branch coverage/default/diagnostic-order negatives before parser repair.

LATEST canonical-section kernel:3841963 executes3before failures;3841994
passes all6unchanged regressions plus27controls/properties. Shared source
candidate is under full508domain/523core/217capture integration3842014.
Original217discovery3841964 proves SC D4/D6 as well as D8/E7 differences.
See canonical_seed_sections. Corrected CPUreview3841968 verifies86pre-section
cases:40math matches,7rejections,21Rust failures,18original failures; full
review3841969/fat3841831 pending. No transfer of acceptance to new sources.
Suffix222discovery3842018 isolates a second original signal (while), keeping
both compound failures and all old inputs; parser repair remains open.

CURRENT:212capture3841698 completes99whole positive matches/no losses, but
D8/E7 canonical seed/central-fiber outputs differ; E8metadata alone newly
matches. Before kernel3841963 and217original discovery3841964 submitted,
no production seed fix yet. See
[canonical sections](slices/canonical_seed_sections_2026-09-29.md).
Exact523standardrelease3841793 is complete; all116cases CPU3841830/fat3841831
submitted. Failed reviewer3841833 used the wrong manifest hash; corrected
review-only3841968/3841969 retains all outputs. No full consumer acceptance.
Suffix discovery3841829 retains an invalid original-signal positive and adds
a valid companion in217; ordinary-expression grammar repair remains open.
Older pending statuses below are historical; current source/report pins are
in the indexed slices and submission receipts.

NEXT ordinary-expression suffix-tagged case grammar blocks combinatorics.at944
after lazy523 advances groups.at loading. Three original-discovery fixtures
expand language212->215; no parser change or goldens yet. See
[tagged case suffix](slices/tagged_case_suffix_2026-09-29.md). Keep all full
mathematical fixtures; loader markers after errors are not accepted passes.

LATEST R4lazy3841689 COMPLETE523core/CLI/final source integrity; report
9e14806e2efcaabefa88558cd9285a01a4005c5657a97b3ae6d70cff645b735f
verified local/HPC.212after-capture3841698 RUNNING. Standard release3841793
RUNNING, preflight3841794 PASS; source equality and all116unchanged math
consumers remain required before acceptance. See lazy_real_form slice and
R4release/build receipts. Older pending statuses below are historical.

E7/D8 full KGB A/B on exact517 source is independently VERIFIED3841670/3841690,
four full-output-equal rounds each. Rust1/Rust4 medians2.183940/2.303566;
Rust4 still sloweroriginal4.39x/3.55x and all arms below60s. This is scoped
parallel screening, not lazy523/high-level acceptance or a global default.
See [parallel A/B](slices/parallel_ab_2026-09-28.md) and exact report receipts.

LATEST R4lazy3841689 is confirmed RUNNING;212capture3841698 follows afterok.
R3failed metadata test setup before mathematics; R4constructs the same identity
matrix directly with all assertions retained. Cancelled unstarted R3capture
3841649 and both earlier compile failures remain recorded. Full116eager517
arrays3841517/3841518 are still live and separate from lazy523. See
[lazy real form](slices/lazy_real_form_2026-09-29.md) and R4capture receipt.

CURRENT lazy R3job3841627 is SUBMITTED,523expected core. R2job3841592
fails only the test's unavailable KgbId constructor; its report e2ee7d94af31a1b3a12187b592bf163a5cd64aa188ace96e4f0e281745451473
is retained. R3uses an actual id from an independent healthy graph, without
changing runtime or weakening span assertions. See lazy_real_form slice.

Lazy3841545 first compile fails(E0425/E0277 in print helpers), report
ca92f3e4f7d9984c9a5d975981dd6270ee008a4e99fcf84b12bbd6d072dbb41f
retained and verified. R2job3841592 is submitted with fallible renderer/span
propagation and additional checks inside the existing span test;523expected
core inventory unchanged. No lazy math acceptance yet. See lazy_real_form
slice and R2receipt; retain all212capture and116math inputs.

NEW lazy KGB/Rep_table candidate3841545 is UNVERIFIED (523expected core,
six owner/metadata/retry/concurrency/span tests). Original212capture3841527
now confirms all three full D8/E7/E8 metadata streams; eager517Rust times
out45s in each. Local next renderer corrections must use a new immutable
stage. See [lazy-owner migration](slices/lazy_real_form_2026-09-29.md).
Current full116release jobs are still eager517, not this source.

CURRENT integration3841489 verifies517core,10Cartan and all earlier direct
kernel controls, CLI check/release and complete source integrity. Interpreter
is wired through explicit budget/cache mode; this supersedes historical
UNUSED notes below.209three-armcapture3841497 completes98whole positive
matches,3gains/no losses: full D8/E7/E8 Cartan outputs equal original.
Report0fb1483b85a7c488a3e966b31cb846f13d9f6634b51f31fa992a139865f9c61b.
High-level imports still45s timeout; retain aggregate original failures.
Exact-source standard release3841502 COMPLETE, all1423files equal3841489;
preflight3841503 PASS. ALL116math cases submitted CPU3841517/fat3841518,
fullreview3841519/CPUreview3841520. Original3841464
is complete and confirms full D8/E7/E8 inventory totals. No end-to-end
mathematical or parallel speed acceptance yet. Local212generic corpus adds
three provisional metadata-only real-form fixtures before lazy migration,
submitted in212capture3841527 against unchanged517release3841502;
all existing KGB/high-level cases stay. See direct-generation and
library-context slices for reports, pending gates and owner/cache guards.

Direct-generation prototype3841440 verifies full sets/matrices in32contexts
and compact D8/E7/E8 closure; new generator/partition APIs3841461 are not yet
wired into the interpreter. Need explicit budget/cache migration, original
full Cartan/KGB/KL/FPP revalidation and separate lazy real-form ownership.
See [direct generation](slices/direct_twisted_generation_2026-09-29.md).
Kernel3841461 now VERIFIED at32exact full partitions, all controls and large
D8/E7/E8 partitions; report9cc355680ad943d48c8f7734a4674dd7d408cfc149f6f3605a070fbd09805618.
This remains an unused API, not a repaired high-level interpreter path.
209case original-reference extension keeps every prior input; initial3841457
is a pre-execution report-path failure, not mathematical evidence. R2 uses
the same binaries with the real verified release build.json.
R2job3841464 is RUNNING; preserve its live handle and collect full artifacts.
Current516CPU review3840814 verifies40math/7rejection matches,21Rust and18
original failures over86cases (40includes3language/load cases). High-level
math and full116review3840813 remain open; no speed claim from kernel tests.

Current516release advances high-level loading to groups.at251's E8 context,
then hits the4000000 full-Weyl enumeration cap (A2_klv3840825, original
succeeds). Exact debug sample3840859 separately locates eager E6_s KGB work
at groups.at237. These are capacity/initialization barriers before high-level
math, not an A2 coefficient mismatch. Next port boundaries and source-backed
direct twisted-involution enumeration lead are indexed in
[library context loading](slices/library_context_loading_2026-09-29.md).
Full R2suite review3840813 remains pending; preserve every earlier failure.

Exact516release3840661 is BUILT with all1423source files equal foundation;
report9cf3bd131d9aa12550fa5013f052aa54dfc1233edf37a16919e646341a3ce185.
First arrays3840709/3840710 failed BEFORE execution because the submitter
omitted SUITE_INPUTS_SHA256. Preserve those reports; fresh unchanged-input
stage80u4BXD7 submits all116under3840811/3840812 after preflight3840810,
fullreview3840813 and separate CPUreview3840814, with hash exported to BOTH.
206discovery3840662 confirms original acceptance but45s debug timeout for
all four independent unitarity/Hodge/AV-ann/cycle imports. Report
086bee469851c525c0f10f025cee90a41fe43c73af72f4dbf616380a4488a948.
These are loader failures, not arithmetic diagnoses. Test-only orbit probe
3840707 is separate; production algorithm unchanged, see indexed
[generator orbit experiment](slices/twisted_orbit_search_2026-09-29.md).

FINAL516candidate3840634 passes core/CLI/final integrity,95whole original
positive matches in202cases (four gains/no losses versus511). Full three
torus streams and extension positive now agree; extension negative stdout
and category agree, stderr envelope differs. Report
bb020272fb63cb71d3664cc3f4bb34b7445be7e93f5fd1875f98b4c07c1a8ed8.
groups.at is now a45s debug timeout with no output, not a successful load.
Exact516release3840661 is building for all116mathematical cases; separate
206case discovery3840662 adds unmodified high-level-library imports.
These submissions are not high-level acceptance. See HANDOFF/receipts.

Full514candidate3840560 FAILED513pass/1fail: the old typed registry test
rejects adjoint(A1.T1,false), contradicted by original3840488 exact case.
Preserve the failure, migrate to captured RootDatum/value and rerun fullgate.
Discovery3840609 completes202cases, confirming T0/Tn normalization errors
and17extension validation sites; full original-backed positive/rejection
session tests now precede the candidate fix. See
[Lie-type extension](slices/lie_type_extend_2026-09-29.md).
Exact511CPU review3840622 remains40math/7reject/21Rustfail/18originalfail,
same86case statuses as509. Full116review3840522 remains pending on fat.

Torus candidate3840560 is submitted:514core/full200capture; root-free
annihilator dimensions, exported columns and adjoint torus acceptance.
Kernel3840555 already verifies2before-fail ->2after-pass plus9controls,
with final source guards. Full session/consumer acceptance remains pending.
Exact511release3840501 is verified (1413source files equal3840381), with
all116math cases under3840520/3840521/review3840522. It excludes torus edits.
Next202case discovery3840609 captures extend(T,0), canonical torus factors
and17negative extension sites on unchanged511-core. No extension repair yet.

Torus R2 discovery3840498 COMPLETE: independently confirmed B2 exported
root_coradical transpose and pure-T1 alcove-center missing central equations,
in addition to wrong radical rank/adjoint rejection. Complete original
fixtures are preserved; no repairs yet. See torus_radical slice/AGENTS.
Exact511-core release build3840501 is now submitted; must verify source
identity before full116shared-consumer comparison on that newer snapshot.

FINAL empty-datum3840381:511core/CLI/final integrity; full positive matrix
output matches,91whole positive streams/no losses. Seven-negative stdout
and session diagnostics match; CLI stderr presentation differs. groups.at
now stops217/basic.at1744 at involution(extend(lt,"T",r),...,ict), with
Too few inner class symbols; inspect zero-rank extension before repairing.
Torus/adjoint/radical200case discovery3840498 remains separate and active.

NEW torus3840488 confirms wrong SC T1 (co)radical bases and wrong rejection
of adjoint(T1), with complete accepted original28data. Runtime untouched;
see [torus radical/adjoint frontier](slices/torus_radical_2026-09-29.md).
Nonsymmetric root_coradical orientation still needs an independent control
because the early adjoint rejection aborts the combined loop. Keep all cases.

Empty-datum3840381 now passes both focused session regressions; full511core/
197capture gate remains pending. Discovery3840368 plus3840376 pins legal
Nx0, seven matrix rejections, and a separate ORIGINAL empty-list exit139.
See [empty root datum](slices/empty_root_datum_2026-09-29.md). New198case
discovery investigates torus radical/coradical dimension/orientation on
UNCHANGED509-core; no radical implementation changes yet.
Release3840371 uses exactly509-core1407files and submits all116math cases
under3840394/3840393/review3840420; excludes empty-datum candidate.
Older506-core86CPU review3840370 is complete (40match/7reject/21Rust failure/
18original failure), while its full116review3840271 remains pending.

FINAL signed-rational3840284:509core/CLI/integrity and both full signed
outputs match;90whole positive streams/no losses. Shared high-level release
still required. Latest506-core high-level failures expose explicit empty
root datum construction at groups.at3/basic.at1180, not polynomial writes.
Unchanged509-core discovery3840368 retains192 and adds three empty-datum/
rejection/groups-load fixtures. Matrix0xN must not become Nx0 through the
legacy row adapter. See HANDOFF for pins; no constructor repair accepted yet.

FINAL projection3840261:507core/CLI/integrity and full T2/A1.T2 output pass;
88whole positive streams, no losses. This supersedes pending507gate notes.
Next509-core signed-rational repair3840284 remains pending.116math release
jobs3840269/3840270 use earlier506-core; combined shared-kernel revalidation
must follow the newer candidate's full gate, not be inferred from this run.

Signed discovery3840264 now confirms two additional mathematical defects on
unchanged506-core: negative coroot coordinates fail exact reconstruction,
and negative A1 alcove centers lose sign. New regressions/candidate use exact
signed TryFrom<&Rational> at only those proven sites; see
[signed rational extraction](slices/signed_rational_2026-09-29.md).
509core/192capture candidate is distinct from the frozen506-core release.
That release3840262 is source-identical (1402files); ALL116math comparisons
now submitted: preflight3840268, cpu3840269, fat3840270, review3840271.
K-type503-core review3840192 FINAL retains45math/9rejection matches and all
seven original failures, two Rust failures and two timeouts; no losses.

FINAL polynomial/sign3840188:506core/CLI/integrity,87whole positive matches,
no losses. Current original basic.at now loads completely/no diagnostics;
declaration-output differences retained. Fresh exact-source release3840262
will revalidate ALL116math cases, including actual high-level unitary/Hodge/
AV-ann/cycle operations. No high-level acceptance merely from basic.at loading.
Projection kernel3840260 proves before3fail/1pass, after4pass plus19controls;
full507core/190capture3840261 pending. Separate192case signed-coroot/alcove
discovery is staged against unchanged506-core; do not mix source/case counts.

New signed arithmetic frontier: polynomial3840181 exposes loss of rational
sign; R2job3840188 passes focused unchanged regressions, full gate pending.
Projection3840186 proves a separate skew T2/A1.T2 canonical-lift mismatch
despite accepted execution and coset self-consistency. Direct regression and
Euclidean quotient candidate are indexed in
[signed projection](slices/signed_projection_2026-09-29.md).
K-type release3840182 source-identical to503-core3840172 now runs65cases under
preflight3840189, cpu3840190, fat3840191 and independent review3840192.
This release excludes the newer sign/read/projection changes. Kernel3840183
passes9KType/1B2anchor/10context checks and final source integrity; it is not
general mathematical acceptance.

LATEST: K-type Euclidean3840172 verifies503core/CLI/integrity and full
original formula output;189case replay3840179 verifies the broader complete
coset stream,86full positive matches/no losses. Release3840182 is pending
shared-kernel math revalidation. Previous selector release review3840164
verifies45math/9rejection matches among65cases, including repaired F4Cartan.
General high-level gates remain open. Active polynomial READ candidate3840181
targets basic.at2437 with505core/189capture gates; writes remain separate.
See [polynomial coefficient contracts](slices/polynomial_coefficients_2026-09-29.md)
and [K-type formulas/cache boundaries](slices/ktype_formula_2026-09-29.md).
Kernel3840173 passes9KType/10context tests but fails its zero-test guard;
corrected3840183 uses the actual B2 transported-lift anchor. Direct projection
coverage is absent;190case discovery adds signed-echelon/skew T2/A1.T2 cases
to investigate another truncating-division site before changing it.

LATEST2026-09-29: selector3840085 passes500core/CLI/integrity; basic.at now
reaches2437's polynomial coefficient subscription. Original discovery3840100
accepts reads/writes; receiver-before-index and cross-form write asymmetry
are retained. Candidate3840098's complete K-type formula unit uncovers A2
noncanonical keys and incorrect term coalescing from truncating lambda_unique
division. R2 uses the actual upstream Euclidean rule; full after-pass pending.
See [K-type formulas/cache boundaries](slices/ktype_formula_2026-09-29.md).
Release3840096 is verified source-identical to3840085;65math cases are under
arrays3840103/3840104/3840138 and independent review3840164. That release does
NOT contain the K-type or Euclidean candidate. General high-level gates stay open.

The basic.at671 selector frontier and original-backed evaluation-order/generic
Cartesian-product contracts are indexed in
[full unit selectors](slices/full_unit_selectors_2026-09-29.md).

FINAL named-update3840019:496core/CLI/final integrity,82of177whole language
matches/no losses. basic.at now stops671at a full-unit dot selector, not402.
Corrected Cartan survey finds F4 numbering-sensitive B2/C2 type mismatch;
the focused regression and selector discoveries extend language catalog180.
See HANDOFF and integrality/named-update slices for source contracts/pins.

Review3840066 independently verifies58release cases:38mathmatches and
9rejection matches; E7FPP/D8/E7KL remain incomplete. The eight new Cartan
inputs use unsupported bare-core int*mat, so establish no Cartan acceptance.
Corrective fixture and timeout-forwarding tests are staged separately;
the prior jobs actually used300s despite intended600/1200 exports. See
HANDOFF and the indexed integrality slice; preserve old failures and inputs.

FINAL3839978 verifies493core/CLI/integrity with normal stack. Complete root
and B2 positive outputs match original;81of177whole streams, no previous
losses. Wrong-rank controls reject correctly with differing stderr formatting.
The current shared-helper release revalidation expands math cases108to116
with Cartan consumers; it is separate from the ongoing named-update build
3839981 and original-field diagnostic3839980. See HANDOFF for frozen pins.

Named compound update grammar/regressions and the suspected original global
field-update bounds defect are indexed in
[named update diagnosis](slices/named_update_2026-09-29.md). Preserve the
complete effects fixture; do not turn its anomalous no-update result into a
Rust golden before checked-original diagnosis.

Combined B2/root/analysis candidate3839978 is submitted with493expected core
tests and177frozen capture cases. The analysis split preserves all40 branch
bodies and normal stack; mathematical acceptance remains pending. Subsequent
named-update work must use a separate candidate and retain the full positive,
effect-order and negative original3839528 fixtures. See HANDOFF for pins.

The B2 integrality counterexample, shared-helper scope and independent stack
failure are indexed in [integrality subsystem diagnosis](slices/integrality_subsystem_2026-09-29.md).

New confirmed B2 integral-subsystem wrong rank/datum/dominance and five
wrong-rank acceptances are preserved by3839528. The shared simple_basis
ambient-coordinate criterion is invalid; candidate reflection-positivity
repair needs FPP/Cartan revalidation too. Root-interface3839527 independently
hits a stack overflow in its new whole-session positive. See HANDOFF and
AGENTS: no root-interface acceptance, do not weaken tests or raise the normal
stack as a repair. Named-update discovery now has accepted original contracts.

FINAL iffor3839502 passes489core/CLI/final integrity and79of169whole streams
with no prior losses; latest basic.at now fails at402's named update AND:=.
Original3839518 accepts new root/integrality and compactA1 stored-cache
positives, which Rust rejects by name. W_refl/integrality_simples now have an
unverified implementation candidate and exact original-backed units; the
remaining raw K-type plus four stored-cache interfaces are still unimplemented.
Cold and warm query semantics must remain distinct; warmed KL/deformation
did not populate Q in the captured compact example. Receipts are in HANDOFF.

Current-original3839492 exposes nine unregistered names, despite historical
registry-completion claims: query, system, integrality_simples, W_refl,
K_type_formula_raw, stored_full_deform, stored_twisted_full_deform,
stored_KL_sum_at_s, stored_KL_Q_polynomials. Completion names are not support.
New original discovery covers root/integrality across classical+exceptional
groups and compactA1 cold/warm caches. Quiet-if/iffor candidate3839502 remains
under full489core/169language verification. Exact receipts and findings are
in HANDOFF; high-level mathematical gates are unchanged.

FINAL void/for3839432 verifies483core/CLI/final integrity and74of159whole
language streams, no previous losses. basic.at moves to378 filtered for/iffor;
this is the next script-loading implementation frontier. Separate confirmed
wrong let-pattern acceptance and runtime no-value container behavior remain
in retained fixtures; constant expressions require separate folding checks.
See HANDOFF and slices/void_boundaries_2026-09-29.md for exact evidence.

Void-boundary candidate3839432 is submitted with483expected core tests and
159frozen language cases. No after-pass yet. New161case local discovery adds
explicit-consumer and no-value probes; see HANDOFF and the indexed void slice.

Original3839396 now confirms void value boundaries (including SCALAR7,
GLOBAL42, CALL6 but EXPLICIT_TUPLE((),8)/EXPLICIT_ROW[(),()]). Preserve
for3839386's unchanged failing unit; audit all explicit discard consumers
before removing eager void wrapping. Both void fixtures and iffor_values are
accepted originals. The visible ## override probe is rejected before override,
not valid hidden-join evidence. Exact receipts and values are in HANDOFF.

Void-discard boundary diagnosis is indexed in
[`slices/void_boundaries_2026-09-29.md`](slices/void_boundaries_2026-09-29.md).
Original discovery3839396 is submitted with159cases on verified3839335.

For3839386 now compiles but fails its unchanged void-context assertion:
VOID() rather than original VOID6.4of5new units pass; full gates not reached.
Next159case discovery isolates structural/inferred void boundaries and iffor.
Detail3839388's intended multi-term A1 polynomial collapses to one canonical
term; retain it but add distinct-key coverage rather than claiming term-order
validation. Count narrowing has bare error envelopes still unclassified;
the fourth alleged overflow input does not reject in original. See HANDOFF.

ByteR2 3839335 is now terminal with475core/CLI/source-integrity PASS and
65of145complete language matches (six gains, no previous losses). Overall
FOUNDATION_UNITS_PASS_ORACLE_UNAVAILABLE retains the original signal. Latest
basic.at reaches reversed counted for at183; the next general-for candidate
adds five captured-fixture units and is unverified. No added high-level math
acceptance. See HANDOFF and slices/for_iteration_2026-09-29.md for scope/pins.

ByteR1 3839304 fails compilation on two test-only exhaustive event matches;
R2 3839335 fixes those only and is SUBMITTED with unchanged475expected core
tests and145language cases. Retain R1's E0004 report, no skipped assertions.
Independent149case for-iteration discovery3839328 is also submitted on the
unchanged3839234 runtime; do not mix its inventory with the build gate.

Byte candidate3839304 is SUBMITTED (475core expected, frozen145language).
The current local next-discovery catalog149 adds general for iteration across
all seven upstream receiver kinds, including ParamPol/KTypePol, reversal and
captured-frame/context contracts. No loop repair is in3839304. Source routing:
[for_iteration_2026-09-29.md](slices/for_iteration_2026-09-29.md).
The numeric/string/domain byte expectations and retained rejected '+' fixture
are indexed in the string_bytes slice; do not conflate either language suite
with the108case mathematical acceptance catalog.

Latest3839234: all465core/CLI/final integrity pass without skips or a larger
test stack;59of135whole language streams match, no losses versus cast3839029.
Overall remains FOUNDATION_UNITS_PASS_ORACLE_UNAVAILABLE from the retained
original signal. Byte3839240 confirms original1111/Rust3333 without either
engine rejecting. The current local byte-value/output/slice repair is UNVERIFIED;
new discovery3839257 captures143cases against unchanged3839234, including
legacy ascii unit sources and raw transport/domain diagnostics. Keep those
captured failures, normal-stack units and full-stream gates. No high-level
math acceptance has been added; see HANDOFF and indexed string_bytes slice.

Update:3839232 passes the new RETURN grammar, named-function context/linkage
and structural-negative regressions, then still stack-overflows in the unchanged
structural-positive test. GDB3839222 measures a97320byte debug evaluate frame;
larger diagnostic stack yields RECURSIVE120 then parser failures. Combined
3839234 splits the38unchanged evaluator arms into sixfamilies with normal test
stack unchanged (465core/135language required). Original byte-string/non-row
slice discovery3839236 is separate139case evidence; output bytes must remain
exact, not UTF-8 replacement. See HANDOFF for hashes and active handles.

Return3839117 finishes459core/CLI/final integrity but loses three named-function
declaration streams;54whole matches, gains2/losses3versus cast. Expanded3839163
passes asymmetric slice but FAILS structural-return rejection (wrong_explicit
installed) and stack-overflows in its positive. Both reports are retained.
Root cause of wrong_explicit: RETURN expr consumes the following semicolon,
unlike upstream RETURN tertiary. Current unverified candidate repairs precedence
and non-destructive named-function matching, adding three units (465expected).
Exact-binary HPC stack diagnosis is separate; no test is skipped. See HANDOFF.
This is not unitarity/Hodge/AV/cycle acceptance. String slicing and general
for reversal remain language prerequisites.

Byte-string/slice source contracts and the required value-to-output boundary
are indexed in [string_bytes_2026-09-29.md](slices/string_bytes_2026-09-29.md).
Do not label an ASCII-only implementation as the full upstream string contract.

2026-09-29 current frontier: cast3839029 passes455core/CLI/final integrity,
55whole language streams; basic.at now fails string slicing at86:10. The
return_loop and return_inference discoveries3839034/3839086 expose SILENT
wrong results and wrong type acceptance, preserved in tests/math/generics.
Priority repair3839117 is submitted (459core expected,130case frozen master);
safe shared result contexts cover early returns, casts, next and balancing.
It is unverified; see HANDOFF/receipts for exact pins. Local next corpus134
adds structural result constraints and string-slice probes, not math successes.

Operator binding candidate3838965 fails its positive regression at `@` after
compilation; negatives pass. Preserve the unit failure. Original3838987 now
validates exact-first operator casts, generic fallback/lexical formals and
global-table lookup despite local shadowing. A combined455core/125case candidate
is staged; see HANDOFF for pins. This is still prerequisite, not math acceptance.

Break3838716 is now verified448core/CLI/integrity,50whole-stream language
matches with no losses. basic.at advances to line91 symbol binding. Original
3838739/3838953 back the new operator-pattern candidate3838965 (452core expected,
122language cases). Full global/local/tuple/parameter/loop contracts preserve
nonfunction rejection and atomicity; current runtime remains unverified.
See HANDOFF and generic-language slice for stage/pins. Mathematical acceptance
still stops before high-level scripts; do not count these units as math passes.

Function-value R4 job3837531 passes425/427core tests, with the same two known
assignment failures separately executed, CLI and final source integrity checks.
The failed3837257 printer assertion now passes. Master100 has33full-stream
matches but retains original's signal; status explicitly separates verified
build/unit checks from unavailable complete differential acceptance. R2's same
95-input comparison gains five exact matches27to32 with none lost. Balance
discovery3837421 confirms wrong rejection AND wrong acceptance before repair.
Implicit-constness repair3837694 finishes429core all-pass, zero skips, both
unchanged old failures separately passing, CLI and final integrity checks.
Four mutation fixtures now reject and match stdout; stderr envelopes remain
different. Same100inputs retain33whole matches with no losses. Original signal
keeps overall status unavailable/nonzero, not complete differential acceptance.
New original-backed mutation tests preserve concrete siblings, shadowing/rebinding
and state. Loop repair3837799 passes430core tests, CLI and final integrity;
loop stdout now matches (whole stderr still differs). Original permits counted
index assignment contrary to its source comment. Next binding-scope before
probe3837952 executes its intended assertion failure (global_imported=false).
After3837985 stores defining floors with variable types;432core all-pass,
unchanged assertion passes, CLI and final integrity pass. All100input and Rust
stdout/stderr hashes are unchanged versus3837799, with33whole matches and the
retained original signal still invalidating full capture. TypeCell checkpoint is
committed c5437a55. any_type integration3838237 compiles but its accepted identity
regression fails in legacy lambda specialisation; scoped lambda/export repair
3838315 passes that assertion but fails an old escaping-recursive-closure unit
in full core (438pass/1fail). R3 ports the recursive path too. Master library104
cases now also tests actual basic.at loading, runtime continuation and recursion.
No new mathematical acceptance.

Update3838431: abstraction/recursive-context candidate passes441all core,
CLI and final integrity, with13new full-stream matches (33to46on the same104
cases), no lost matches. Retained original signal keeps full capture invalid.
basic.at now fails at case-controlled while/DONT line73. Discovery3838473
adds positive guard/body scope and missing-else rejection in106case master;
see HANDOFF and its submission receipt. General do_expr must retain let/case
bindings across the guard and body, not duplicate a discriminator/effect.
Discovery3838473 now verifies the four original scope outputs and missing-else
rejection. Scope candidate3838545 (106cases/443expected core) is SUBMITTED.
Separate context discovery3838562 (108language cases, not the108math catalog)
uses verified R3 to capture count/void/reversal and dead-body/scope negatives.
These jobs have different frozen inventories; collect, never mutate them.
Context discovery3838562 is now complete: original COUNT3/VOID2/reversed
row/exact guard effects/nested dont and three analysis errors are captured.
Context-mode build3838593 is SUBMITTED (expected445core/108language cases);
current uncommitted runtime is this candidate. Scope build3838545 has all443
core passing but no final CLI/capture/integrity result yet. HANDOFF records
exact pins. Do not enlarge mathematical acceptance from these prerequisites.
Scope3838545 is now terminal:443core, CLI and final integrity pass; its scope
fixture matches full streams,47total full matches/no prior losses. basic.at
now exposes take[void]/[A] and count[void]/int semantic errors. Count is in
active3838593. Break still wrongly forces void; two new provisional fixtures
extend language inventory110/58positive. Keep lexical negatives and capture
original before repair; source axis.w's breaker imposes no result type.
Mode3838593 failed compilation (Fn ownership restriction); preserve report.
R2 mode3838648 now uses FnOnce and adds a zero/one-use ownership unit;
446core/108case verification pending. Break discovery3838627 confirms both
generic definitions/values in original but rejects its later numeric break1.
Latest grammar repeats BREAK tokens. Keep those failed sources; new isolated
companions grow local language inventory113/59provisional positives. Break
implementation is still unchanged; capture whole valid fixtures before repair.
Latest: mode R23838648 verifies446core/CLI/final integrity and48full language
matches. Break R23838661 fully accepts the repeated positive in original;
history3838675 captures unchanged two/three-level values and rejects old
numeric spelling. Current break candidate3838716 fixes bottom result typing,
repeated syntax and depth wording, with two session tests and original-backed
historical test migration. Expected448core/frozen116cases; not yet verified.
Local language master118/61positive adds bare operator-value positive/negative
discovery, needed later in basic.at. See HANDOFF for separate immutable stages.
See HANDOFF and the generic-language slice for exact pins.

Active overload/member wave3836397 failed full core:416pass/3fail/2filtered
(all163 filtered checks pass). Original3836455/3836507 confirms the next repair:
ordinary generic row/printer/error overloads must replace the old hidden
fallback;3#[] is ambiguous, empty/concrete row joins succeed, concrete bool-row
cardinality conflicts with generic.88 original intents confirmed48accept40reject.
Row-build3836533 COMPLETE14:01: full423-test inventory421pass/0ignored, both
known failures separately executed;164filtered checks/CLI pass.88-case capture
has27full-stream matches, versus19 for verified3835776 on the same corpus.
Row registry, mixed joins and projector/field positive controls now match.
Negative diagnostics and tagged-union stdout still differ; do not claim full
generic/basic.at compatibility. Pins/all failures are in HANDOFF/the slice.
Function-value before-capture3836435 verifies all82 pre-row intents;3836409
remains an invalid earlier classifier run, not a language verdict. any_type,
TypeCell scopes, function-value capture/direct-call inference and current
positional union syntax remain before latest basic.at/high-level math.
Function-detail discovery3836975 adds three probes (catalog91) but FAILS because
original exits139 in the combined selection-negative fixture. This is not a
normal rejection; retain it and isolate the trigger. The frozen88-case row
gate is separate. Captured builtin display/tuple packing succeeds in original;
the runtime discovery's lowercase ascii must be retained as a name rejection
and supplemented by uppercase ASCII before making runtime-trace claims.

Constructor R3 candidate3835776 COMPLETE8:34: full413-test inventory confirms
411pass/0ignored and both known assignment failures separately execute;155
filtered tests and CLI pass.62-case capture confirms all original intents;
18full-stream matches versus15previously. Explicit structural constructors now
match; arity negatives reach the right category, but projector unification and
any_type still need implementation. Report/source pins are in HANDOFF.
R2 build3835267
passes43type/3coercion/48syntax checks then fails its new session test because
it inspects Output rather than typed ReportLine; all expected values are in
the failure log. R3 changes only that assertion and reruns full-core/CLI/capture.
Keep both failed builds; source/pin/report details are in HANDOFF and the indexed
generic-language slice. Member capture3835786 confirms66/67 provisional intents:
generated-projector ambiguity and linked result constraints are real; the
accidental reserved fi variable is a fixture syntax failure, retained with a
separate valid companion. Replay3836211 verifies wrong projector ambiguity
(Rust returns99) and field-assignment rejection.3836223 disproves the historical
case/in/function union syntax against latest original. Both discovery failures
remain. Catalog73cases41accept32reject adds current positional-pattern syntax
and an isolated monomorphic legacy negative. Replay3836236 COMPLETE26s using
the exact3835776 candidate confirms all73original intents and18full matches;
the current pattern case gives8/0, while Rust wrongly accepts the legacy form.
All jobs terminal; runtime committedadf40792. Continue active inference and
field/discrimination repairs, not another unchanged contract replay.
Actual any_type scopes, scheme inference and latest high-level math remain open.

Current build3835058 COMPLETE7:45 verifies the complete407-test core inventory:
405pass,0ignored, two known polymorphic assignment failures separately executed.
150 filtered checks and CLI check/build pass; all56 original intents confirmed,
15 full stdout/stderr matches. Report
0dc45ae377d879bc213e893b03ec6096d1619f403582950b8c64939195d6400f.
Exact declaration probe3835038 first confirmed retained Pair and all values;
only then was the historical assertion updated. FPP/F4/E6 units remain passing.
All jobs terminal; generic grammar/inference and high-level mathematics remain
open. Neither known assignment error is repaired; no debug speed comparison.

Scope boundary capture3834951 COMPLETE55cases: constructor multiline fields,
sibling scopes, sequential polymorphic declarations and implicit recursive-group
arguments accepted; TYPE_VAR value binding rejected. Mandatory constructor bang
on next line is rejected, disproving the provisional intent. Source remains in
the library. Lazy lexical-scope foundation3834975 COMPLETE6:50:149related tests,
CLI check/build and55case capture pass;14 full matches retained. Six new tests
verify scope/token infrastructure; actual generic grammar/inference still open.
Core review3834919 fails only the historical alias declaration report assertion
(398pass/1fail/2filtered); exact source added as56th original probe before any
expectation update. See HANDOFF and generic slice; do not duplicate terminal jobs.

Named-type before3834815 COMPLETE45cases: all original intents confirmed;
simple union discrimination and forget/reuse expose two active Rust failures.
Candidate3834852 failed3session assertions after84related units passed; no CLI
build/capture. Edge3834868 confirms49original intents and disproves intuitive
named-void discarding and projector cleanup. R2 candidate3834895 COMPLETE6:30:
143 related units and CLI check/build pass;49case capture has14 whole-output
matches (previously2). Named union/forget failures now pass unchanged inputs.
Named negative categories match but diagnostics differ; identifier queries and
closure printing remain gaps. The active type table/structural consumers are
ported on these cases, not just report text. Full-core review3834919 exposes
the historical declaration assertion described above. Full generic scopes, implicit
constness and high-level math gates remain open. Debug timings are not speedups.

Persistent type-name parser/annotation bridge3834754 COMPLETE6:17 passes136
unit tests and CLI check/build. Its36case capture is INVALID: original startup
fails GLIBCXX_3.4.26/29 due to missing GCC runtime-library path. Corrected
replay3834785 COMPLETE52s reuses the exact candidate;9 checker tests and all36
oracle intents confirmed. Named annotation/recursive calls now execute and
bad components reach type rejection; TYPE_ID parameter binding wrong acceptance
is removed. Names/query reports and diagnostics still differ, and only the two
existing monomorphic controls match whole streams. Both jobs terminal; source,
failed capture and corrected replay retained separately. No duplicate needed.
Full generic scopes and scheme-carrying analyzer, implicit constness and named
type retention remain separate required integration work. No speed comparison
from the debug candidate. See HANDOFF and the generic-language slice.

NEW TYPE MATCHING:3834447 COMPLETE4:44;38type/3coercion tests and CLI check
pass. Structural matching, direct-function substitution and full two-sided
rollback verified internally; active parser/analyzer still unmigrated.
R7 capture3834702 COMPLETE34s confirms31 oracle intents (17accept/14reject),
including named annotations, direct/tuple polymorphic calls and rigid-result
rejection. Rust still fails those new cases at syntax. Global/local wrong
assignment regressions remain open. Both jobs terminal; see matching/R7
receipts and `slices/generic_language_2026-09-28.md` for exact evidence.

PREVIOUS TYPE SCOPE:3834389 COMPLETE4:15;27type/3coercion tests and CLI check pass.
Owned InferredType + nine new internal tests carry pending substitutions and
fixed/free ranges together. Known global unit still fails. This foundation
is not yet connected to the parser/analyzer; no latest-script support claim.
See HANDOFF and `slices/generic_language_2026-09-28.md`; no duplicate job needed.

NEW TYPE FOUNDATION (not language acceptance):3833858 COMPLETE,17 type units,
3 coercion units and CLI check pass; the existing global assignment regression
still fails. Owned schemes/substitutions are implemented but not integrated
with syntax, analyzer and overload resolution. R4/R5 captures3834266/3834282
add nested-scope/local-binding contracts and expose a second wrong acceptance:
polymorphic local empty-row assignment. Concrete local [int] assignment fully
matches. Local before-unit3834285 COMPLETE executes0pass/1fail; no repair.
R6 capture3834288 COMPLETE confirms25oracle intents. Foundation R2 job3834296
COMPLETE: structure unit before-fail/after-pass,18type/3coercion units and CLI
check pass. Do not duplicate these terminal jobs. No high-level support yet.
See `slices/generic_language_2026-09-28.md` for exact evidence and the remaining
scope-carrying analysis migration. High-level mathematics remains blocked on
that real port; do not replace or preprocess latest original scripts.

Parallel-computation A/B is now an explicit user requirement. Existing timing
records forced Rayon1; same-binary Rayon1/4 E7 KGB screening is now verified
by3834377 (54 checker tests, four whole-output matches,36 rehashed artifacts).
Median paired scaling2.958576x, peak RSS+18.54%; Rust4 still about17.7x slower
than original. Selected only for this E7 workload, no global default change.
See `slices/parallel_ab_2026-09-28.md`; all upper-level math gates remain open.

CURRENT: E6 Cayley build3833716 and independent review3833739 COMPLETE:
14mathmatches/1rejection across15 retained cases, including complete E6 core
KL and F4 history coefficients. Unchanged E6 regression executes/fails before
and passes after at integer/half scale. All targeted units and release pass.
Do not duplicate completed jobs; see HANDOFF and exact submission receipts.
High-level language prerequisite capture3833740 now has11 isolated probes:
four intended positives pass original, generic/concrete overload is ambiguous,
and duplicate formals are accepted despite the initial rejection hypothesis.
Rust fails every original-positive case. Freeze actual contracts before porting
scoped variables/substitution/overload resolution; do not rewrite basic.at.
All E7/resource, broader group/real-form and upper-level mathematics gates remain.
Generic capture3833758 verifies16 oracle contracts and exposes Rust's wrong
acceptance of assignment to a polymorphic empty list. Probe3833788 COMPLETE
executes the unchanged-runtime unit failure;3833787 COMPLETE adds the concrete
`[int]` control, whose full output matches. The new unit remains failing.
See `slices/generic_language_2026-09-28.md` before implementing type schemes.

The following records preserve the earlier progression.

NEWEST: E6 review3833590 completed:20mathmatches/1rejection/1F4 mismatch/
1E6 failure. All8 KGB cases match. The E6 constructor repair exposes a
deeper partial-KL `second image` panic (raw3833584), not whole-E6 acceptance.
F4 coefficient probe3833589 executes the same wrong result with AND without
history; report565e9084... is retained. A minimal first_endgame_pair control-
flow repair is staged at atlas-math-endgame-repair-20260928.IMRdjiLk, with
the unchanged regression and independent before/after targets. No after-pass
or full-output acceptance yet. Preflight3833611 PASS50; build3833612 and
dependent15-case array3833613 submitted. The unchanged F4 coefficient unit
now fails before and passes after in both contexts; release/full review pending.
E6 probe3833616 failed source staging (extra.orig, no tests); corrected
same-module probe3833624 is submitted. Collect these exact jobs, do not duplicate.

### Earlier wave snapshots (latest status is above)

Review3833539 completed with12mathmatches/1rejection, residual F4
history MATH_MISMATCH, E6 constructor failure and original E7 allocation
failure (Rust also allocation-aborts). F4 core KL is now fully equal; do not
equate that with case107 acceptance. A stored-output diagnosis is being
staged without rerunning the interpreters. E6 coordinate repair is frozen
at atlas-math-e6-form-repair-20260928.l7uK40QO; preflight3833556,
build3833558 COMPLETE with unchanged before-failure/after-pass proof and
release CLI;23-case array3833560 running, review3833590 submitted. See receipts.
F4 diagnosis3833564 now isolates exactly pool102/P(4,334): original
[0,0,2,3,3,2], Rust[0,0,4,6,5,3]. All partial results and complete index
matrices match. Focused cold/history probe3833589 is submitted; audit
first_endgame_pair's inverted missing-cross/no-valid-t control flow against
original kl.cpp:318-340. This is a source-backed suspect, not a verified fix.

CURRENT WAVE: combined KL boundary + coroot closure candidate is staged at
atlas-math-kl-coroot-repair-20260928.vCEV2VU4. Preflight3833256 PASSES44checks;
build3833274 COMPLETED with unchanged before-fail/after-pass tests and release
CLI. Array3833289 running (16 full-output cases including F4 half/zero/unit
history); independently pinned review3833539 submitted afterany. Collect it
while continuing E6 repair; do not duplicate or accept based only on units.
Source and acceptance pins are in math_kl_coroot_repair_submission_2026_09_28.json.
All regressions and positivity guards retained; no runtime acceptance yet.
Use the compute time for E6 external-form coordinate diagnosis in parallel.
Unchanged-runtime E6 probe3833407 is COMPLETE: basis e1/e3 is correct but
e3 first flips the nonsimple imaginary root[0,0,1,1,1,0], so the old map
rejects it. Executed before-failure artifact/receipt and AGENTS guard retained.
Next verify fixed ambient basis coordinates directly; do not alter the
partition specialGrading election or remove the coordinate invariant.

Newest evidence: FPP reflection repair before/after unit passes, and independent
review3832333 confirms D4 wall and E6 complete outputs now agree. Eight positive
matches/eight negative matches; E7 still times out at300s, so the whole FPP gate
remains open. Review3832506 confirms the same candidate also times out at600s:
original24.1906s/981172KiB, Rust600.2182s/2588124KiB. No completed speed ratio.
R11 review3832340 adds four original-pass/Rust-fail partial-KL regressions
(history, containment, nonstandard rejection), after retaining and correcting
R10's script-only list-equality fixture mistake. Catalog now108cases, with an
additional F4 partial/full/partial history regression at index107.
Candidate core now routes partial_KL_block through actual-parameter lookup,
complete Hasse downset and locator-aware condensation/interner. Preflight3832344
passes38checks; build3832345 and array3832350/review3832517 are COMPLETE.
Review:11mathmatches/1rejection match, F4 Rust KL recursion panic, E6 existing
constructor failure, E7 original allocation failure plus the same Rust panic.
A2/B2/C2/D4/G2 and all new targeted regressions pass. Next fix must address
the eagerly unwrapped `cross of extremal` in kl_table.rs:265. Focused
unchanged-runtime probe3832534 reproduces the panic and identifies a real-II
cross leaving the interval (x263,y278,s2). A local candidate preserves the
required descent links and zeroes only this legitimate missing real-II term,
as original KL_pol(UndefBlock,sy) does. Build3832615 still FAILS: the integer
scale passes its partial-parameter assertion, but half scale hits the separate
locator positivity failure. Dependent array3832623 was cancelled without
execution. R12b review3832614 confirms that cold-half F4 fails too while the
original succeeds. Static source audit finds root-versus-coroot additive
closure in locator.rs. Probe3832726 now executes and fails both new exact
B2/F4 regressions on unchanged runtime (4vs8 closure elements and positivity).
NEXT: checked coroot-additive closure repair, same-unit after-pass, then the
combined candidate's full differential. No active job remains from this wave.
See the new AGENTS repair guard, HANDOFF and receipts.
See HANDOFF and new receipts. Do not claim or commit the runtime as accepted
until the corresponding differential gates and independent review are examined.

The user has explicitly added sequential repair of discovered errors to the
objective. Next repair: conjugating suffix in FPP reflection_word, guarded by
the already-failing D4 wall fixture and new exact root-action unit regression.
Do not call it fixed until the HPC before/after and differential artifacts pass.
Subsequent root causes remain partial-KL parameter/Bruhat lookup, E6 construction,
and latest generic-script compatibility. Record reusable repair lessons.
FPP candidate preflight3832250 passes36checks. Build3832260 failed because
the after command reused the before test binary via a shared Cargo target;
both archived source hashes are correct. Replacement3832301 uses independent
target-before/target-after in atlas-math-fpp-repair-r2-20260928.49AOGABt,
after preflight3832300. It is IN PROGRESS, NOT differentially accepted.
See the R2 submission receipt and AGENTS repair guard. Keep the main baseline
separate; do not rerun the invalid shared-target procedure.

R9 now has103cases with D6/D8 KGB and nontrivial A2/G2 Hodge at half/zero
scale and bounds4/20. Review3832222 verifies all8 original Hodge failures:
negative branch bounds in A2, real-form mismatches in G2. These are not limited
to trivial parameters. Source-bound repeated benchmark3832212 and its
dependent review3832228 are recorded in math_suite_r9_submissions_2026_09_28.json.
That benchmark review is now complete: four E7/D6 pairs match every output byte;
E7 median1.35766s original versus69.4160s Rust, D6 both below1s. D8 is blocked
by Rust's4000000 enumeration cap while original succeeds. The source enumerates
the entire compact Weyl group before filtering twisted involutions; analyze
that algorithm instead of simply raising the limit.

R3 broad54review3832103 is complete (artifact math_suite_broad_review_2026_09_28.json).
MATH_VALIDATION now has the full initial72-case operation-by-group matrix.
E6 KGB fails Rust construction. E7 KGB is exact but69.80s vs1.30s original
in one run; RSS1.39GiB vs40.7MiB. E6/E7 Rust FPP reaches300s while original
completes0.28/24.46s. These are current-main evidence, not historical optimized
candidate comparisons. No stable speed ratio or failed-run speedup is claimed.
The original itself times out in nine broad inputs and allocation-fails in
E7 KLV/Hodge under6GiB; separate these from Rust mathematical defects.

R7 adds index92 for product numerical cycle multiplicities1/2/3 with full
Phi identity binding and independent exact kernel/tensor checks. The catalog
has93candidate cases; general classical/exceptional cycles, cutoff proof and
richer coefficients remain open. All execution/checker tests stay on HPC.
R8 review3832201 now confirms original passes all30 numerical reconstructions,
the8 full Phi sign identities and full rational kernels of dimensions27/75.
Rust still fails latest basic.at loading. R7's any(Maybe<vec>) fixture failure
is preserved; the correction uses succeeds and does not modify upstream scripts.

New kernel findings (independently verified R5 review3832147): A2/G2
partial_KL_block returns singletons instead of4/10entries (half-scale2versus1),
and D4 FPP wall representatives can send integral simple roots negative.
The R6 catalog had92cases including explicit regressions with original-backed
expectations. See `slices/math_failures_2026-09-28.md`; never repair by changing
goldens or dropping mismatching Weyl elements. Source leads: reversed conjugate
reflection suffix and actual-parameter Rep_table/Bruhat closure, respectively.
These are not yet fixed or fully source-repair verified.
Full-output KLV differences also affect B2/C2/D4/F4. E6 instead fails Rust
real-form construction; E7 original hits std::bad_alloc under6GiB AS cap.
The latter is incomplete resource-limited evidence, not a mathematical verdict.

R4 independent review3832122 confirms six preserved Hodge original failures,
including increased bounds and complex-form examples. A2 traces reach an empty
tensor product whose height-1 is passed to branch; G2 traces use ambient G2
as owner of Levi K-types. Do not hide these by modifying the pinned oracle.

Current objective is mathematical correctness, not merely builtin registration.
All tests/builds/benchmarks run on HPC. `tests/math/catalog.json` tracks72 initial
classical/exceptional cases plus3language prerequisite cases and explicit gaps.
General associated
cycles must include multiplicities and cutoff justification; AV-ann, orbit
support and bounded KNilpotent matrices do not close that requirement. Hodge
grading specialization at one small bound does not prove full filtration
coverage. Further real forms/isogenies and D6/D8/E7 minute-scale benchmarks are
still required. Memory is a capacity guardrail, not the priority.

Remote Rust main remains05625c5d after pull. Latest original is7e1b958c, not the
historical4d3e9449 oracle. Source diff spans257files. Static inspection identifies
generic `set_type Pair<S,T>` / `any_type` in latest basic.at versus missing main
grammar; HPC independent reviews3831996/3832014 confirm the loading gap.
Do not silently mix in older optimized/PGO candidate results or alter latest
original scripts. The shared HPC Rust working checkout is dirty and unrelated
to this immutable-main baseline; preserve it.

Build3831844 completed and source/script/binary hashes were independently
verified. Pilot3831897 and corrective3831999 are fully reviewed: A2/G2 root
data, KGB and FPP match; rank-rejected cases agree. Latest-script KLV,
unitarity, AV-ann and cycle-foundation fail in Rust before mathematics begins.
Hodge bound4 fails in the original too (A2 negative branching level; G2
K-type real-form mismatch); keep these cases and investigate, not waive them.
The KGB/FPP initial failures were fixture errors (unloaded unary matrix minus
and `ones`), corrected without changing the upstream scripts.

R3 checker preflight3831995 passes17tests. Broader54-case array3832011 remains
in flight; review3832103 depends afterany on that exact array. Full receipts,
hashes and frozen paths are in HANDOFF and `tests/reference/hpc/math_suite_*`.
Do not duplicate jobs. `docs/MATH_VALIDATION.md` separates verified results,
unresolved mathematics and historical speed evidence. These single-shot small
cases do not establish stable speedup. Every newly discovered Rust calculation
error must be retained as a regression test under AGENTS hard rule7.

## full_deform common-block recursion verified (2026-08-20)

`full_deform` now looks up the interval-below partial block at each
reducibility point and evaluates singular contributions with the stored
block modifier and per-row `gamma_lambda`. This removes the two spurious
B2 `[13]` terms from the previous full-block approximation. The accepted
anchor is `tests/fixtures/domain/full_deform_proper.atlas`; local output
matches the oracle for integral, half-integral, and non-final parameters.
Commits `466e066` and `b267f2a` contain the implementation and export. HPC
capture `3586752` and fat differential `3599345` both pass; the fixture is
now `verified_hpc`.

## twisted_deform proper subsystems landed (2026-08-19/20, capture 3591165 in flight)

`twisted_deform` dispatch drops the Full-or-NYI guard and passes the parent
through to the slice-3 `ProperSubsystem` arm of `with_integral_block`
(`24fab16`); `twisted_deformation_terms` takes `parent: &KlSumParent` with
per-row lambda_rho via `KlSumParent::sr` on Partial parents. Fixtures
`twisted_deform_proper{,_terms,_rejected}` are local-oracle IDENTICAL;
HPC capture 3591165 submitted (registration + fat differential pending).

Two findings recorded from that slice (agent-91):

- **Alcove-wall closure overshoot → NDEBUG truncation** (pre-existing
  divergence, NOT a slice-4 gap): for gamma on the top alcove wall in
  non-simply-laced data (anchor: B2 `param(KGB(rfb,10),[0,0],[1,0]/2)`),
  `int_item`'s alcove-wall closure (`additive_closure` of
  {α2, -(α1+α2)}) overshoots to the full B2 datum — rootdata.cpp:685-707
  does the same upstream — and upstream's `codec::internalise` assert
  (repr.cpp:104) is compiled out under the oracle's `-DNDEBUG` build,
  silently truncating 3/2→1. The Rust port's honest `IntegralCodec`
  invariant rejects instead. Affects the slice-3 paths identically
  (`twisted_KL_sum_at_s` fails the same way on that anchor). Any fixture
  with gamma on the top alcove wall of a non-simply-laced datum hits this.
- **q2/pb locator collision** (FULLY RESOLVED 2026-08-21, branch
  `f9dd1a4`): `param(KGB(rfb,10),…)` and the pb anchor intern the same
  block record under different locators; the second lookup gets a
  non-identity query-to-stored attitude. The relative-attitude merge +
  cofolded ext_block generators are wired, AND the final print-path
  divergence turned out to be a pool-shape bug: `with_integral_block`
  used `lookup_full_block` where upstream uses the partial `lookup`
  (repr.cpp:2378-2382, 2605-2606), so deform interned a full block the
  oracle never pools. Fixture `domain/locator_attitude` pins the
  combined deform+print battery; HPC capture 3603952 in flight.

## twisted_full_deform partial lookup landed locally (2026-08-20, HPC pending)

Recursive `twisted_full_deform` now uses `RepTable::lookup` at every
non-singleton reducibility point, including when the integral subsystem is
the full root system. This matches `repr.cpp:2605`: the deformation terms
run over the interval-below partial common block, not a rebuilt full block.
The old full-block path produced two spurious B2 `[13]` terms.

`scaled_extended_finalise` also now scales `RepContext::nu(sr)` while keeping
`lambda_rho` fixed; it previously fed `gamma` back as `nu`, which is only
equivalent when `lambda_rho` is zero. The B2 nonzero-lambda case is pinned by
a unit test. `tests/fixtures/domain/twisted_full_deform_proper.atlas` is
byte-identical against the local oracle for the x=5 integral anchor, its
half-integral counterpart, and the x=10 non-final input. HPC capture and full
differential registration remain pending.

## global.w batch 4 landed locally (2026-08-19, HPC capture pending)

The LAST global.w slice (work order `docs/slices/global_batch4_workorder.md`):
the flag-bitfield matrix slicer and the two GF(2) builtins, implemented in
`crates/atlas-core/src/matreduc.rs` (`swiss_matrix_knife`, `mod2_section`,
`subspace_normal`, plus a 10-line port of `permutations::standardization`,
permutations.cpp:257-282) and three `ScalarOp`s in
`crates/atlas-core/src/typed.rs`:

- `swiss_matrix_knife` (int,mat,int,int,int,int->mat) (global.w:4675-4809,
  install :5195-5196): rows `i:k`, columns `j:l`, half-open, 0-based. Flag
  bits: 0/3 reverse output rows/columns; 1/2 and 4/5 read the row/column
  bounds FROM THE END (`lwb = dim - bound`); 6 transposes (dims swapped
  BEFORE the copy); 7 negates (WRAPPING i32 — `mat: [[-2147483648]]`
  survives). `flags` truncates to its low 8 bits (BitSet<8>: `-1` sets all
  bits, `256 == 0`, never throws); the four bounds narrow via `ulong_val()`
  in upstream pop order `l, j, k, i` ("Negative integer where unsigned is
  required" / "Integer value to big for conversion", typo verbatim). The
  bounds diagnostic fires AFTER all six arguments evaluate and BEFORE the
  no-value gate, uses the RAW bounds (from-end bits do not relax it), and
  reads "Range exceeds bounds: ... out of range, actual limits are2, 2" —
  NO space after "are", space after the comma; inverted ranges clamp to
  empty keeping the (swapped) shape.
- `mod2_section` (mat->mat) (global.w:5043-5053, bitvector.cpp:346-405):
  GF(2) section `ABA=A`, `BAB=B`, TRANSPOSE-shaped (n_cols x n_rows) output,
  entries `(x&1)!=0` (negative odd -> 1). NO validation and NO no-value gate
  before the compute (only the push is gated). >64 rows/columns are guarded
  upstream by `assert`s only — compiled out under NDEBUG, i.e. UB: the
  pinned oracle silently drops the out-of-range bits (probed:
  `mod2_section(null(65,1))` -> 1x65 zero, `null(1,65)` -> 65x1 zero,
  `null(70,70)` -> 70x70 zero, no error). Rust MASKS row bits >= 64 on input
  (and basis bits >= 64 for >64 columns), reproducing that observed
  silent-drop; keep >64 inputs out of fixtures.
- `subspace_normal` (mat->mat,mat,mat,[int]) (global.w:5062-5174): GF(2)
  reduced column-echelon for possibly dependent generators, tracking
  combinations and relations. Output columns are PIVOT-ASCENDING via
  `standardization` (NOT loop order); relation columns follow generator
  order minus pivoters (`d = j - l`). Validation BEFORE the no-value gate,
  dim first: "Dimension too large: 65>64" / "Too many generators: 65>64"
  (NO spaces around ">").

Fixtures `tests/fixtures/eval/global_batch4.atlas` (35 lines) and
`..._rejected.atlas` (14 rejections) verify against the local oracle:
accepted diff is byte-identical; rejected payloads are verbatim-identical
and exit codes match (the atlas-cli vs oracle error-report wrapper
divergence is pre-existing). HPC reference capture NOT yet run — do not
claim differential-verified status until that lands. The `i32::MIN`-negation
slicer corner is unit-test pinned only; the work order defers its fixture to
an HPC capture of the wrapping regime.

Skipped / documented exclusions (the remaining 3 of the 6 unported global.w
signatures from the batch-4 sweep; global.w is now FULLY dispositioned):

- hidden `"matrix slicer"` (global.w:5197-5198): LANDED locally 2026-08-19
  (HPC capture pending) as the two-dimensional slice syntax `M[i:k, j:l]`
  (parser.y:660-705). The LALRPOP grammar gained the second slice
  production (literal `[` only — there is no `~[` 2-D form upstream, and no
  third axis), `Expr::Slice`/`SliceFlags` carry the column bounds and their
  from-end bits (0x10/0x20), and the typed/eval arm drives the same
  `matreduc::swiss_matrix_knife` engine; the hidden name is NOT registered.
  Oracle probes pinned: absent bounds zero-fill with the from-end bit set
  (`M[:, :]` == identity); inverted ranges clamp to empty keeping the shape
  (`M[2:1, 0:2]` -> "The 0x2 matrix"); the bounds diagnostic uses RAW bounds
  ("Range exceeds bounds: upper row bound 9 and upper column bound 8 out of
  range, actual limits are3, 2" — no space after "are"); negative bounds hit
  `ulong_val` narrowing ("Negative integer where unsigned is required"); the
  base converts against `mat` (a row-of-rows display coerces; `v[0:1, 0:1]`
  with `v : [int]` fails "found [int] while mat was needed."); a mistyped
  bound rejects the whole desugared argument tuple ("found
  (int,mat,int,string,int,int) while (int,mat,int,int,int,int) was
  needed."). Fixtures `tests/fixtures/eval/matrix_2d_slice.atlas` /
  `..._rejected.atlas` diff byte-identical / payload-identical against the
  local oracle.
- hidden `"transpose "` (global.w:5188): LANDED locally 2026-08-19 (HPC
  capture pending) as the commabarlist row-display `[a,b | c,d]`
  (parser.y:370-376, commabarlist :402-410), via a dedicated
  `Expr::BarList`/`TypedExpr::BarList` node that builds the matrix directly
  — segments become the ROWS of the result. The direct construction keeps
  the oracle's exact diagnostics and is immune to user `^`(mat) overloads
  (probed: after `set ^(mat N) = null(1,1)`, `[7,8 | 9,10]` still
  transposes). Entries convert against `int` ("found rat/string/vec while
  int was needed."); ragged rows are a RUNTIME error "Vector sizes differ
  in conversion to matrix" (the per-row [int]->vec narrowing precedes the
  size check); both diagnostics fire in no-value contexts. `;` inside a
  segment SEQUENCES upstream (`[1,2|3,4; 5]` -> rows (1,2),(3,5)), and an
  empty segment is a syntax error (`[1,2 | ]` -> bare "unexpected ']'").
  Fixtures `tests/fixtures/eval/commabarlist.atlas` / `..._rejected.atlas`
  diff byte-identical / payload-identical against the local oracle. Known
  divergence NOT covered by the fixtures: `[ | 3]` enters the union-TYPE
  grammar upstream ("unexpected INT, expecting '|'") while the Rust parser
  reports "expecting ']'" for the same offending INT token.
- `readline_completions` (string->[string]) (global.w:4390-4391): LANDED
  2026-08-21 (`138e7c5`, differential 3604405 PASS). The insertion-order
  problem was solved by capturing the 294-name startup list verbatim from
  the oracle (`STARTUP_COMPLETION_NAMES` in typed.rs) and tracking session
  names in first-definition order; the three startup system variables
  `input_path`/`prelude_log`/`back_trace` (main.w:408-435) landed with it.

## global.w batch 3 landed locally (2026-08-19, HPC capture pending)

The linear algebra builtins (work order `docs/slices/global_batch3_workorder.md`)
are implemented in the new module `crates/atlas-core/src/matreduc.rs` — an
operation-for-operation port of upstream `utilities/matreduc.h`
(`gcd`+recorder, `column_echelon`, `echelon_solve`),
`utilities/matreduc.cpp` (`diagonalise`, `adapted_basis`, `Smith_basis`),
`utilities/matrix.cpp:471-498` (`inverse`), and
`structure/lattice.cpp:133-160` (`kernel`, `eigen_lattice`, `row_saturate`)
over a column-major wrapping-i32 `PidMatrix` — plus ten `ScalarOp`s in
`crates/atlas-core/src/typed.rs`:

- `Bezout` (vec->int,mat) (5201): gcd with unimodular recorder,
  `v*C == [d,0,...]`; `det(C)` may be -1; `Bezout([])` is a TYPE error
  upstream (`[*]` does not coerce to vec) — use `null(0)`.
- `echelon` (mat->mat,mat,[int],int) (5202): E has zero columns REMOVED
  (rank columns), kernel columns rotated right in C, pivots ascending,
  flip = sign det(C).
- `linear_solve` (mat,vec->|vec,int,mat) (5203): the FIRST union-returning
  builtin — representable after all: `Type::union_of([void,(vec,int,mat)])`
  plus `Value::Union{tag,injector_name}` with hardcoded injector names
  `empty_set`/`affine_subspace` (upstream `match_literal`, global.w:4910/4921).
  `echelon_solve` failure is CAUGHT into `().empty_set`, never thrown.
- `diagonalize` (mat->vec,mat,mat) (5204): (diagonal, row, column) —
  diagonal FIRST; only its first entry may be negative; det(row)=det(col)=1.
- `adapted_basis` (mat->mat,vec) (5205): diagonal NOT divisibility-ordered.
- `kernel` (mat->mat) (5206): basis order is oracle-defined (echelon
  recorder rotation), pinned by fixture.
- `eigen_lattice` (mat,int->mat) (5207): NO square check; diagonal touch up
  to min(rows,cols); the `int_val()` narrowing fires BEFORE the no-value
  gate (upstream pops the int first).
- `row_saturate` (mat->mat) (5208): keeps upstream's operator hunger 3.
- `Smith` (mat->mat,vec) (5209): factors positive, divisibility-ordered by
  the correction loop (matreduc.cpp:369-381); zero matrix -> (id, []).
- `invert` (mat->mat,int) (5210): (N,d), N/d = M^-1, d = bigint lcm > 0;
  a SINGULAR square matrix returns the zero matrix with d=0 and NO error;
  the non-square diagnostic `Cannot invert a RxC matrix` fires BEFORE the
  no-value gate.

Fixtures `tests/fixtures/eval/global_batch3.atlas` (64 lines) and
`..._rejected.atlas` (8 rejections) verify against the local oracle:
accepted diff is byte-identical; rejected payloads are verbatim-identical
(the atlas-cli vs oracle error-report wrapper divergence is pre-existing).
HPC reference capture NOT yet run — do not claim differential-verified
status until that lands.

Semantic surprises found while oracle-checking (pinned in unit tests
`global_batch3_builtins_match_the_upstream_linear_algebra_surface` and
`matreduc::tests`):

- Top-level `{ stmt; stmt }` blocks evaluate NOTHING in the oracle (even
  `{ 1\0; 7 }` is silent) — so they cannot exercise the no-value gate; use
  `for i:2 do X od` bodies instead (validation-before-gate pinned that way
  in the rejected fixture and the unit test).
- `linear_solve` on a rank-deficient or zero-column system returns the
  solution vector at FULL width m (recorder pivot block applied to the
  rank-length initial solution); `linear_solve(null(0,3), null(0))` gives
  `([ 0, 0, 0 ],1,id_3).affine_subspace`.
- `diagonalise`'s sign bookkeeping folds the final gcd flip into
  `row_minus` AGAIN after the loop (matreduc.cpp:201): on the
  `d == old_d` exit that is the ROW gcd flip (also already folded into
  `col_minus`); on the `d >= old_d` exit it re-folds the column gcd flip,
  cancelling matreduc.cpp:193. Ported verbatim.
- `Smith`'s correction loop computes `1 - pa/d` in UNSIGNED Denom_t
  (wrapping) before truncating to machine int; positive-entry inputs never
  show it, but the port reproduces the regime.
- Machine-int wrapping is observable in Bezout recorders:
  `Bezout([2147483647,-2])` has bottom-right entry -2147483647.

Skipped / deferred:

- `mod2_section` (5211) and `subspace_normal` (5212): GF(2), deferred to
  batch 4 by the work order — LANDED in batch 4 (see above).
- `swiss_matrix_knife` + hidden `"matrix slicer"` (5195-5198): the batch-3
  work order does NOT list them (batch 2's gap note had guessed batch 3) —
  `swiss_matrix_knife` LANDED in batch 4; the hidden copy is a documented
  parser-gap exclusion (see above).
- `gcd` was already landed in batch 2 (per the work order's claim note).

## global.w batch 2 verified_hpc (2026-08-19, differential 3574922)

Capture 3574906 froze the reference; the fat full-suite differential
**3574922** @ `b9843aa` returned 291 PASS + 1 declared PARTIAL
(container_syntax_errors) across 292 fixtures, with both
`eval/global_batch2` fixtures exact on stdout/diagnostics/exit status;
both metas are `verified_hpc`.

All remaining mechanical global.w signatures are now `ScalarOp`s in
`crates/atlas-core/src/typed.rs`, with matrix constructors/arithmetic helpers
(`identity`/`diagonal`/`transposed`/`negated`/`added_diagonal`/`added`/
`subtracted`/`multiplied`/`multiplied_vec`/`left_multiplied_vec`/
`multiplied_ratvec`/`left_multiplied_ratvec`/`is_zero`) added to
`crates/atlas-core/src/linear_values.rs`:

- int utilities (global.w:2966-2994): `succ`/`pred`, call-syntax bitwise
  `AND`/`OR`/`XOR`/`AND_NOT` (two's-complement bit strings; NOT infix
  operators upstream — `6 AND 3` is a syntax error), `bitwise_subset`,
  `nth_set_bit`, `bit_length`, `to_bitset(vec->int)`.
- container relations (4405-4420): `=`/`!=` on vec/ratvec/mat (unary zero
  tests and binary), dominance `>=`/`>` on vec/ratvec.
- container arithmetic (4421-4451): vec `+`/`-`/unary `-`/`*int`/`\int`/
  `%int`, ratvec `%`(unfraction)/`+`/`-`/unary `-`/`*int`/`/int`/`%int`/
  `*rat`/`/rat`, mat `±int`/`int±mat`/`±mat`, vec dot `*`, `flex_add`/
  `flex_sub`/`convolve`, and all five matrix/vector products. The nine `*`
  overloads that were `domain_builtin_skip` placeholders are now real
  scalars (same registry slots, upstream hunger levels).
- selectors/joins (4396-4399): `#` vec suffix (hunger 1) and prefix
  (hunger 2), `##` on (vec,vec) and [vec].
- constructors (5183-5194): `null(int->vec)`, `^`(vec->mat one-row),
  `^`(mat->mat) transpose, `id_mat`, `diagonal`, `stack_rows` (ragged,
  zero-padded), `#`(int,[vec]->mat) combine_columns, `^`(int,[vec]->mat)
  combine_rows.
- `gcd(vec->int)` (5200): checked upstream — it is the plain non-negative
  integer gcd of the entries (matreduc machinery is only the
  implementation), so it landed here rather than in batch 3. Empty vec -> 0.
  The fold runs in machine-int width: `gcd(vec: [-2147483648])` prints
  `-2147483648` upstream and here.
- `elapsed_ms(->int)` (5245): static stopwatch primed on first call.

Fixtures `tests/fixtures/eval/global_batch2.atlas` / `..._rejected.atlas`
diff byte-identical against the local oracle for accepted input; rejected
fixtures exit 1 on both with verbatim payloads (the atlas-cli vs oracle
report wrapper divergence is pre-existing). HPC reference capture NOT yet
run — do not claim differential-verified status until that lands.

Semantic surprises found while oracles-checking (all pinned in the unit
test `global_batch2_builtins_match_the_upstream_scalar_surface`):

- vec `\`/`%` take a NON-NEGATIVE remainder (`[7]%(-3)=[1]`,
  `[7]\(-3)=[-2]`), unlike int `\`/`%` which floor (`7%(-3)=-2`). Ratvec
  `%int` follows the vec convention per entry (numerator mod d*|m|).
- `mat±int` adds the scalar to the main diagonal up to min(rows,cols);
  non-square matrices are accepted upstream.
- `mat:` literals list COLUMNS (batch-1 knowledge), so all products are
  standard row-times-column once that is accounted for.
- `bit_length` of a negative is `-(significant_bits(~n)+1)`;
  `nth_set_bit` with a negative index counts CLEARED bits; a missing
  n-th set bit of a non-negative value yields -1.
- ratvec `*rat`/`/rat` narrowing ("Integer value to big for conversion")
  fires INSIDE the no-value gate upstream (the computation is gated);
  everywhere else narrowing/zero-divisor diagnostics fire before it.
- upstream parse-time rewriting turns `x+1`/`x-1` into `succ`/`pred`
  calls and folds `-literal`; these rewrites are not ported (observable
  behavior is identical).

Skipped / not ported:

- No BitSet value type exists upstream OR in Rust (bitwise ops live on
  int), so "bitset utilities" were the int ops above — nothing to skip.
- `swiss_matrix_knife`/`"matrix slicer"` (int,mat,int,int,int,int->mat)
  (5195-5198): flag-bitfield slicer, deferred to batch 3.
- `"transpose "` (mat->mat) (5188): upstream registers it with a trailing
  space precisely so users cannot name it; not registered here (same for
  `"matrix slicer"`).
- `readline_completions` (4390): LANDED 2026-08-21 (see the batch-4-era
  note above); batch-mode semantics pinned by `eval/readline_completions`.

~~Pre-existing divergence surfaced by this batch (not global.w's)~~ LANDED
locally 2026-08-19 (HPC capture pending): the generic axis.w row operators
`##`/`#` (join on ([*],[*])/[[*]], suffix/prefix on ([*],*)/(*,[*]),
axis.w:2544-2595) are hidden scalar builtins recognised from the a-priori
type between the exact and coercible ordinary overloads. Bare
`[1,2]##[3,4]`/`[1,2]#3` now resolve the row generics and print compact
(`[1,2,3,4]`/`[1,2,3]`), matching the oracle; fixtures
`tests/fixtures/eval/row_operators.atlas` / `..._rejected.atlas` diff
byte-identical / payload-identical against the local oracle. Suffix beats
prefix when both apply; `*` row components adopt the element type; unequal
row pairs (`[]##[1,2]`) fail with "Failed to match".

~~The `#:=` combined assignment remains a pre-existing parser gap~~ LANDED
2026-08-19 (local gates green, HPC capture pending): the whole assignment
family of parser.y:263-278 — component `a[i] := v` / `a~[i] := v`, field
`p.f := v`, and the `op:=` transforms (component, field, and bare
`x op:= e`, which desugars in the parser to `x := op(x,e)`). The grammar
routes assignment targets through identifier-anchored `Postfix`
productions (upstream's `assignable_subsn` prefix sharing,
parser.y:578-586); evaluation follows upstream order (uninitialised check,
rhs, index, range check) with the `range_mess` wording
"in component assignment". Known divergences, none fixtured: the
out-of-range TRANSFORM diagnostic prints the converted call upstream
(`a[5] succ@int:= ()` via the `x+1→succ(x)` optimisation); the
syntax-error `expecting` list after a non-assignable target says `'='`
where bison says `'\n'`.
~~vec/mat/ratvec subscription reads and vec/mat component assignment and
transforms~~ LANDED 2026-08-19/20 (`0b7e84d` + `6a64816`, local gates
green — 329 atlas-core lib tests; HPC fixture capture optional, the
behavior is rejection/wording-heavy and unit+battery verified):
`v[i]`/`rv[i]` → int/rat, `M[i]` → vec column, `M[i,j]` → int entry
(column-major storage, parser.y:585-598), `~[` counts from the end (BOTH
indices for the pair form). Writes: vec element, matrix column
(`M[i] := v`, size mismatch "Cannot replace column of size R by one of
size S"), matrix entry (`M[i,j] := v`), and all `op:=` transform forms.
ratvec is READ-ONLY ("Cannot subscript value of type ratvec with index
of type int in assignment"). Transform range checks fire on the synthetic
READ (selection wording: "in subscription v[5]" / "in matrix column
selection M[5]" / "in matrix subscription M[5,0]"); assignment checks
quote the assignment node incl. conversion tags ("in matrix column
assignment M[5]:=V[I]:[1,2]", pair WITH parens for entry assignment,
without for subscription). Unit test
`vector_and_matrix_subscriptions_read_and_write_components` + full oracle
battery diffed clean (message texts verbatim; only the diagnostic frame
formatting differs, which the harness normalizes).
Fixtures `combined_assignment{,_rejected}` cover the rest.

Two-index subscription gap — FIXED (2026-08-19): `M[i,j]` / `M~[i,j]`
(and the assignment/transform forms `a[i,j] := v`, `a[i,j] op:= v`) now
parse with a two-element tuple-display index (parser.y:585-598,606-613
`expr ',' expr` productions; new `PostfixSuffix` arms + `pair_index` in
grammar.lalrpop/syntax.rs), then fail typing exactly like the oracle:
"Cannot subscript value of type [[int]] with index of type (int,int)",
the "in assignment" / "in transforming assignment" variants, and
`M[0,1,2]` stays the syntax error "unexpected ',', expecting ']'".
Unit test `two_index_subscription_parses_then_fails_typing_like_the_oracle`;
CLI probed byte-equal against the local oracle on the rejected battery.

`set_type` alias declaration gap — FIXED (2026-08-19, post-63e8118):
`p: Pair` after `set_type Pair = (int x, int y)` now declares, by
re-routing a bare-identifier `Command::Define` right side that resolves
via `TypeTable::resolve_name` to the declaration path (typed.rs
`Command::Define`). This mirrors parser.y, where a defined type name
lexes as TYPE_ID and the command parses as a declaration. Known residual
divergence (not fixtured): upstream then rejects `set Pair = 5` with
"syntax error, unexpected TYPE_ID"; we still treat the alias name as an
ordinary identifier in expression positions. `let p: Pair = …` is a
syntax error upstream too (no type ascription in let bindings), so no
gap there. Fixtures still spell the structural type (`q: (int,int)`);
recapturing them with the alias form is optional churn.

Remaining global.w gap after batch 2 (batch-3 work order):

- `swiss_matrix_knife` + hidden `"matrix slicer"` (int,mat,int,int,int,int->mat)
- `Bezout` (vec->int,mat)
- `echelon` (mat->mat,mat,[int],int)
- `linear_solve` (mat,vec->|vec,int,mat) — union result, first of its kind
- `diagonalize` (mat->vec,mat,mat)
- `adapted_basis` (mat->mat,vec)
- `kernel` (mat->mat)
- `eigen_lattice` (mat,int->mat)
- `row_saturate` (mat->mat)
- `Smith` (mat->mat,vec)
- `invert` (mat->mat,int)
- `mod2_section` (mat->mat)
- `subspace_normal` (mat->mat,mat,mat,[int])
- hidden `"transpose "` (mat->mat): deliberately unregistered (the grammar
  desugars the 2-D slice syntax; the name is unnameable by users).
  `readline_completions`: LANDED 2026-08-21 (differential 3604405).

## global.w batch 1 landed locally (2026-08-18, HPC capture pending)

Rat decomposers `floor`/`ceil`/`frac` (global.w:3249-3251), string
`##([string])`/`ascii` x2 (4387-4389), cardinality `#` on string/vec/ratvec/mat
(4392-4395, mat = column count), and matrix `shape`/`row`/`column`/`rows`/
`columns` (4400-4404; `rows`/`columns` return `[vec]`, not `int`) are
implemented as `ScalarOp`s in `crates/atlas-core/src/typed.rs`, with
`Matrix::row`/`column` accessors in `linear_values.rs`. Fixtures
`tests/fixtures/eval/global_batch1.atlas` and `..._rejected.atlas` diff
byte-identical against the local oracle for accepted input (stdout and exit
codes; rejected fixtures share the pre-existing atlas-cli vs oracle error-report
wrapper divergence, message payloads match). HPC reference capture is NOT yet
run — do not claim differential-verified status until that lands. Notes for the
next batch: upstream `matrix_column_wrapper` truncates the u64 index to
`unsigned int` before the bounds check (indices >= 2^32 wrap; unreachable for
constructible matrices — Rust compares the full u64); `ascii` on a string is
byte-based (empty -> -1); the `[*]` empty row cannot resolve `##([string])`
upstream either.

## Registry reconciliation 2026-08-18 (supersedes the 2026-08-13 counts)

All 305 upstream `atlas-types.w` signatures now have exact Rust counterparts
(zero missing, zero result-type mismatches). The only registry-level gap is
`global.w`: 89 signatures (int/bitset utilities, rat `floor`/`ceil`/`frac`,
string `ascii`/`##`/`#`, cardinality and shape/row/column accessors, container
relations and arithmetic, matrix constructors, `gcd`/`echelon`/`Smith`/
`kernel`/`invert` linear algebra, `elapsed_ms`). These block source-level
`basic.at` library compatibility. Reachable loud NYIs that remain: the
generator-attitude gates (`partial_block`/`W_graph`/`W_cells`/`block_Hasse` on
Param), proper-subsystem twisted/ext recursion, non-integral common blocks,
and cross-block partial merge (rep_table.rs).

## Locator canonicalization probe (2026-08-18)

`tests/fixtures/domain/common_block_locator.atlas` pins the first observable
nonidentity block modifier: A2 split (`inner_class(rd,[[0,1],[1,0]])`, form 0,
SL(3,R)), `p = param(KGB(rf,3),[0,0],[2,1]/2)` installs a rank-one common
block, then `q = param(KGB(rf,0),[0,0],[-2,-1]/2)` collides with it under a
Weyl-conjugate integral subsystem and the oracle prints
`as transformed by <1>` plus transported rows for
`print_common_block`/`partial_block`/`block_Hasse`/`W_graph`/`KL_sum_at_s`.
Current Rust silently diverges here: it builds a fresh block per query AND its
`print_common_block(p)` rows already differ from the oracle for this A2
configuration (gamma-lambda shifted by [0,1] on rows 0 and 2) even at
"identity" attitude — the identity-attitude assumption is not shift-correct
for this family. Reference capture: job 3574723. Implementation follows the
locator design (upstream `InnerClass::int_item` innerclass.cpp:1116-1182,
`block_modifier` repr.h:493-499, `make_relative_to` repr.cpp:338-350).

## Signature-level reconciliation (2026-08-13)

### P0 status and Rep_table blocker (2026-08-13)

P0 now has the exact `from_dominant`, `Cartan_info`, `KL_block`, and
`KL_column` types plus the `(int,Param)` `cross`/`Cayley` overloads.  The
parameter transforms must use `IntegralSubsystem`/`CommonContext`: counting
integral ambient simple roots is wrong when the integral subsystem has a
non-simple parent root (the B2 `[3,1]/2` case is the regression anchor).
`KL_block` validates standardness before its no-value gate; `KL_column`
validates standardness then finality; `from_dominant` validates only rank in a
no-value context; `Cartan_info` skips its computation there.

The accepted P0 fixture still differs at exactly one sequence-sensitive
observation: after `KL_block(p)`, upstream `KL_column(p)` reports raw block row
`1`, while a fresh Rust partial block reports row `0`.  Do not restore the
rejected exact-`StandardRepr` seed cache.  Upstream `Rep_table` keys a block
family by `Reduced_param = (transformed x, integral-system id,
codec(gamma_lambda) mod Smith diagonal)`, stores a locator/block modifier,
swallows related partial blocks when a full block is installed, and lets every
full-block materializer affect later `lookup`.  The fix therefore belongs in a
shared per-`RealFormContext` block pool with faithful reduced keys and relative
locators, not in individual builtin callers.  Required sequence fixtures are
`KL_block -> KL_column`, `print_common_block -> KL_column`, a related parameter
in the same family, and a changed-gamma/block-modifier case.

Oracle job `3545765` now freezes the first three sequence classes plus the
negative install matrix in `rep_table_sequence{,_rejected}`: value-demanded
`KL_block` and `print_common_block` install, while discarded `KL_block` and
the direct block printers do not.  The capture took 0.015/0.009s and
4508/4360 KiB; report SHA256 is
`b078c04a0fe0dd854deb7400fa491bd535e8fe1255532b605ba28504cc7d0ec9`.

The first faithful implementation slice may support the full-integral,
identity-locator domain, but it must already use an explicit
`ReducedParamKey { x, integral_system: Full, residue }`. Compute `residue`
from the transported `RealProjection::lift_mat` and the Smith-diagonal codec,
using Euclidean remainders and upstream's wrapping `u32` mixed-radix packing.
Register every representative of a materialized block through `co_reduce`
(reverse insertion so the smallest row wins); registering only the queried
seed is still the rejected seed cache in disguise.

The first proper-system path is now active: the table interns an exact embedded
integral subsystem by its ordered parent-simple root IDs, keys rows with the
actual subsystem parent coroots, and builds full common blocks through
subsystem generators.  B2 `[3,1]/2` gives a rank-one three-row block, and the
A2 `KL_column` proper-subsystem event is runnable against its frozen oracle
event. ~~This remains deliberately weaker than upstream
`int_item(gamma, locator)`: Weyl-conjugate systems are not yet canonicalized
together, and no `w`, `simple_pi`, or nontrivial block modifier is stored.~~
RESOLVED 2026-08: `locator.rs` canonicalizes Weyl-conjugate integral systems
(`int_item`-equivalent, locator.rs:313) and the block modifier (`w`,
`simple_pi`, shift) is stored and transported.

The proper-system print surface is now covered too: `print_block(Param)` and
`print_common_block(Param)` render an embedded B2 rank-one block and are
differential-verified by `domain/print_common_block_proper` (HPC 3551242).
Extended and twisted consumers still need subsystem-aware extended-block
generator metadata.

The proper-system Param W-graph surface is now covered for identity generator
attitude: `W_graph(Param)` and `W_cells(Param)` consume the shared
`RepTable`/`PartialBlock` KL topology, including imaginary compact/noncompact
grading. The B2 `[3,1]/2` fixture is byte-exact in differential `3564991 @
3adbd42b89dbea029ed4fb0e9c53f47b3e46173e`; the 283-fixture run has runnable
status PASS with two declared pending fixtures, and this fixture took 0.009s /
7376 KiB exact peak RSS. Report SHA256 is
`1cdb3d5924a1cf76b6166d0b632eced4570ba112fd751af95a4c7babec786c8d`.
Nonidentity `simple_pi` transport and canonical locator attitude remain
unimplemented, so this is not a claim for every Weyl-conjugate proper system.

The same identity-attitude scope now covers `block_Hasse(Param)`. Its Hasse
recursion is shared by classic and partial block topologies, and the B2 proper
fixture returns the oracle's three stored parameters and rank-one incidence
matrix exactly. Differential `3565080 @
659646a1290d7a842766c2f5984cc6636211eab0` passed all runnable observations in
284 fixtures; this fixture took 0.006s / 7256 KiB exact peak RSS. Report SHA256
is `a128613557474a2d1f88fe415c62a6a826c64da4fff17370c65be3fc4eaabd4d`.

The reusable mathematical base of that slice is now present in
`atlas-real-group::rep_table`: a crate-private `ReducedParamKey` and
`IntegralCodec` built from the transported `RealProjection::lift_mat`, using
the existing Smith diagonaliser, exact divisibility checks, Euclidean
remainders, and upstream-compatible wrapping `u32` mixed-radix packing.  This
does not yet add the shared pool, locator, full common-block builder, or any
language registration.

The complete pool belongs to the shared real-form value and needs stable block
IDs, reduced-key `Place`s, locators/modifiers, and partial/full promotion. Keep
the mutex structural: reduce/probe under a short lock, materialize block/KL
data outside it, then re-probe and atomically commit. Full materializers
include `print_common_block`, `block(Param)`, `block_Hasse`, `KL_block`,
`dual_KL_block`, Param `W_graph`/`W_cells`, and `KL_sum_at_s_to_height`;
partial consumers include `length`, partial block/KL operations, `KL_column`,
`KL_sum_at_s`, and deformation recursion. `print_block(Param)`,
`print_partial_block`, and standalone external-delta extended computations do
not install entries. Tests must also prove same-context default-form sharing,
custom-form isolation, and isolation between separate `TypedContext`s.

The KL ownership boundary is now ready for stored common blocks.  A sealed
`BlockTopology` trait covers classic and partial common blocks plus borrowed
and `Arc` handles; `KlSupport`/`KlTable` store the handle generically rather
than borrowing only `BlockGraph`.  Construction validates the KL rank,
nondecreasing length order, complete cells, and in-range links before any
recursion, so malformed topology returns a structural error instead of
panicking.  This is infrastructure only; it does not materialize or cache a
full common block yet.

The crate-private shared-table kernel is now implemented for the same
full-integral, identity-locator domain.  `RepTable` is lifetime-bound to the
three owners borrowed by its `RepContext`; it stores append-only block IDs,
superseded tombstones, all-row reduced places, partial/full records, and
relative shifts.  Materialisation runs outside the structural mutex and
commit re-probes atomically.  Fresh partial lookup returns the exact seed row,
whereas later reduced-key hits use reverse registration's smallest row;
promotion retires all overlapping partial records with one place-table pass.
Deterministic tests cover full/partial commit races and failure-atomic overlap
rejection.  ~~Partial-partial merging remains a loud NYI, and the kernel is not
yet owned by `RealFormContext` or consumed by language builtins.~~ RESOLVED
2026-08: the RepTable is owned by `RealFormContext`, partial-partial merging
with cross-attitude row transport is implemented (`rep_table.rs`
`State::overlap_hits` + `make_relative_to` merge, repr.cpp:1601-1707) and
unit-tested (`rep_table.rs:2527+`).

The first full common-block constructor is now present behind the language
boundary as `PartialBlock::build_full`.  Its verified implementation domain is
rank zero and the full-integral, identity-locator subsystem.  It ports the
upstream top ascent, real-root orbit, FIFO involution packets, Cayley fiber
completion, global `y` numbering, length reversal, sort/remap, and lookup by
the complete `StandardReprMod`.  A1 seeds at all three rows and the pinned B2
12-row block agree, including the two distinct rows with `x=10`; a proper
nonempty subsystem still returns an explicit `NotYetImplemented`.  This is
crate infrastructure only: it does not yet install a shared `RepTable` block,
provide a locator/modifier, register `block(Param)`, or establish an HPC
language-compatibility claim.

The denominator `> 2^rank` alcove-center preprocessing used by ordinary and
twisted full deformation is now implemented.  The shared real-group helper
preserves `x` and `lambda_rho`, replaces `gamma`, and rebuilds the standard
parameter through `RepContext::sr_gamma`; ordinary deformation applies it at
each final helper input, while the twisted path does not manufacture a flip.
The threshold explicitly treats rank 63 and above without signed shifting, and
the rational solver rejects contradictory residual rows.  This closes only
the shrink preprocessing fixed by `deform_alcove_shrink{,_rejected}`.  It does
not supply the missing ordinary recursive deformation formula, RepTable memo,
proper-subsystem modifiers, or timed overload/cancellation semantics.

### P2 Block W-graph status (2026-08-13)

`W_graph(Block)` and `W_cells(Block)` are implemented and locally match the
HPC-captured A1 values, rejected calls, and upstream's observable no-value
assignment bug (the graph is still built before its value is dropped).
~~`block(Param)` remains deliberately unregistered.~~ RESOLVED 2026-08:
`block(Param)` is registered (`typed.rs:9054`) and dispatched
(`domain_builtins.rs:13124`) through the RepTable common block.  A tempting implementation
through `common_block_srms`/the classic Block graph is not a compatible
subset: the legal A1 parameter `param(KGB(rf,2),[1],[1]/2)` triggers Rust's
height-parity invariant although upstream returns a one-element common block,
and a non-standard parameter must fail the upstream `test_standard` gate even
when the result is discarded.  Exact completion is part of the shared
RepTable/ReducedParam work above, not a builtin-local approximation.

The P2 pipeline plans therefore run the Block graph/cell events and declare
both the accepted `block(Param)` event and the affected `block(RealForm)`
overload-rejection event pending.  Removing one overload changes Atlas's
candidate-set diagnostic, so rejected fixtures must be rechecked whenever a
signature is temporarily withheld; preserving only the runtime call lines is
not sufficient evidence of language compatibility.

The name-level closure below was insufficient: upstream `atlas-types.w`
registers **305 distinct `(name, argument type, result type)` signatures**
across 187 names. A fresh comparison against the Rust registry found:

- 277 exact signature matches;
- 28 exact missing/mismatched signatures (23 missing argument signatures,
  5 result-type mismatches);
- 23 signatures grouped into 16 small wrapper/registration tasks;
- 12 registered signatures with an explicit reachable unsupported branch;
- 6 signatures whose completion depends on larger deformation/common-block
  algorithms.

The simple queue is, in order: `from_dominant` (two vec overloads),
`Cartan_info` result type, `KL_block` result order, `KL_column` row result,
`cross`/`Cayley` on a simple-root index and `Param`, Weyl `#`/`##`,
`Cartan_class(KGBElt)`, unary/list polynomial operations for KTypePol and
ParamPol, `block(Param)`, Block `W_graph`/`W_cells`, and Param `twist`
overloads. Larger work includes arbitrary-root Param transforms,
proper-integral common blocks, non-integral W-graphs/ext-KL, alcove shrinking,
and timed deformation/cancellation semantics.

There is no KL-file or GNU-readline builtin in these 305 interpreter
registrations. `filekl` is a stand-alone/interface concern; readline is CLI
infrastructure (the separate `global.w` helper `readline_completions` is not
part of `atlas-types.w`). Completion claims must therefore use the 305
signature ledger, not unique builtin-name counts.

## Batch status (2026-08-12 late — full reconciliation)

Reconciliation vs upstream atlas-types.w (187 unique install_function
names — count via a multiline-tolerant scan; an earlier strict regex
undercounted at 152): **170+ live in typed.rs, 10 never registered.
A first-pass "3 skip-only + 12 partial-skip arms" finding was retracted
after empirical probing — those typed.rs skip registrations are dead
code shadowed by live arms** (see below; operator names like `!= # ##
% * + - / =` are live via the operator layer;
`classify_involution`/`element`/`index` are live via
`domain_builtin_validate`, which name-based scans must treat as a real
registration).

Never registered:
- E2: scale_extended, K_type_pol_extended, finalize_extended
- E3: twisted_deform, twisted_full_deform, twisted_KL_sum_at_s,
  block_deform
- shift_flip — **LANDED 2026-08-12 (`46963fd`)**, differential 3541888
  in flight
- print_partial_block, print_partial_common_block (upstream installs
  them; fixture + reference captured, language layer pending — brief
  /tmp/slice_ppb_brief.md)

**NDEBUG assert parity lesson (2026-08-12, `f668589`)**: upstream
`assert`s (e.g. ext_block.h:356 `(1+theta_x)*shift==0`,
ext_block.cpp:938 `same_standard_reps` in same_sign) are compiled out
in the oracle (-DNDEBUG). The shift_flip wrapper reaches both with
violating inputs; as Rust debug_asserts they panicked where the oracle
returns `false`. When porting, omit any upstream assert that the
wrapper layer can reach with violating inputs, with a comment citing
NDEBUG parity.

Skip-only / partial-signature skips — **retracted (2026-08-12
empirical)**: probing every upstream signature on the committed tree
(dafdc03) shows the "skip" arms in typed.rs are dead registrations
shadowed by live ones. dual(RootDatum), inner_class(RealForm),
involution(KGBElt/CartanClass), twist(KGBElt), twist(KGBElt,mat),
K_type(Param), param(KType), real_form(Param/KType), dual(Block),
`#`(Block), KL_block(Param), dual_KL(Block — the only upstream
signature, atlas-types.w:9102), KL_sum_at_s_to_height(Param,int) ALL
evaluate correctly. There is no skip-arm tail. (These conversion arms
are live but several lack dedicated fixtures — a coverage gap, not an
implementation gap.)

So the entire remaining builtin surface is the 10 never-registered
names above.

## Batch status (2026-08-12)

Landed and verified_hpc since 08-11: ext three-builtin registration
(extended_block/raw_ext_KL/partial_extended_KL_block, differential
3537192), slice A (coroot_queries sweep + root numbering family,
differential 3537366). Slice B (orbit/ladder + poly surface) committed
`53872bb`, differential 3538136 in flight. Crate additions:
RootSystem min_roots_for/min_coroots_for + bourbaki_permutation
(`57049ca`), global_KGB + print_X layout (`64048ac`),
BlockDescent::dual + BlockGraph::dual (`1e7fcc4`).

The former imaginary-grading gap for Param `W_graph`/`W_cells` is closed in the
identity generator attitude: the subsystem-aware `PartialBlock` supplies the
compact/noncompact grading and `domain/w_graph_param_proper` covers it in HPC
differential 3564991. The remaining boundary is nonidentity `simple_pi`
transport and locator canonicalization, which must land before claiming the
full Weyl-conjugate proper-integral domain.

Slice E recon (agent-45, /tmp/slice_e_brief.md) revised the
ext_param+star estimate upward: the whole ext_param layer including
star (ext_block.cpp:990-2280, ~1300 lines C++) is missing from the
crate — estimated 1400-1800 lines of Rust. Split plan: E1 crate
ext_param+star core → E2 finalise three-piece set (needs new fixtures
+ HPC probe capture first) → E3 twisted family + block_deform
(fixtures already verified_hpc_reference). Correction: dual_KL is NOT
unlocked by BlockGraph::dual — upstream raw_dual_KL_wrapper uses a
block with swapped real forms + dual_map.

## Batch status (2026-08-11)

Differential `3533446` PASS (199 fixtures: 198 PASS, 1 known PARTIAL, 0
FAIL); the five Weyl/B2 metas upgraded to verified_hpc (`7a5eba5`) — all
harness fixtures now verified_hpc. ext_kl crate slice landed (`602fce6`):
DescentTable + ExtKlTable (KL_table) + condense + ext_kl_matrix with
A2-trivial/A2-flip/Sp4 oracle anchors. Crate side of the ext family is
now complete (ext_block `28e6109` + ext_kl); language registration of
extended_block/raw_ext_KL/partial_extended_KL_block is the next slice
once agent-30's alcove/FPP slice frees atlas-core. Per-slice recon for
ALL 50 missing builtins is now complete in
docs/slices/post_weyl_lang_queue.md (§3 ext registration, §4 print
family + shift_flip dependency correction, §5.1-5.5 ladder/orbit/
small-sweep/deform anchors): 8 near-flips (skip arms already shared
with live siblings + semisimple_rank + reducibility_points), the rest
mapped to concrete crate/language gaps.

Reconciliation vs upstream atlas-types.w (178 install_function names):
128 live in typed.rs, 50 missing (28 never registered + 22
skip-placeholder only; several skip names have main overloads live and
lack only partial signatures).

Remaining: alcove_center/alcove_root_vertex/FPP_numers/FPP_w_shifts (in
flight, agent-30), extended_block/raw_ext_KL/partial_extended_KL_block
(crate ready, wrappers atlas-types.w:7366-7431/8682-8728/7445-7468),
shift_flip (NOT cheap: needs per-parameter shifted_default_extension —
belongs to the ext_param+star slice, post_weyl_lang_queue.md §4),
ext_param+star (largest single block ~1000-1200 lines), finalise three
(finalize_extended/K_type_pol_extended/scale_extended), affine_orbit_ws/
basic_orbit_ws, root_ladder_bottoms/coroot_ladder_bottoms,
root_expression/root_index/coroot_expression/coroot_index/
root_permutation/root_involution (oracle root numbering blocked),
twisted_deform/twisted_full_deform/twisted_KL_sum_at_s/block_deform/
dual_KL_block (+KL_block/dual_KL/KL_sum_at_s_to_height/
truncate_above_height partial signatures, common-block srm pool),
print_gradings/print_real_Weyl/print_blockstabilizer (RealWeyl crate
**已移植** `51b9d83`：real_weyl.rs 1858 行含 10 个字节级锚点测试；坑：
对偶侧必须用精确 `-θ` fiber 链——取 primal 代表元 canonical word 在
对偶 datum 重放后右乘对偶最长元，不能用对偶 classification 的
canonical 代表元，cartanclass.cpp:121；尚缺语言层 wrapper 注册), print_X (GlobalTitsGroup 600+), print_common_block/
print_block(Param)/print_common_block (srm pool, last; print_partial_* 钉住版未安装),
KType/Rep skips (null_module 变体/W_cells——K_type_pol/first_term/
last_term 的 PolP 强转 gate 已落地 `domain/polp_coercion`;
reducibility_points 已注册并修复复根 lwb 播种), small items (semisimple_rank/two_rho_check/
simple_coroots/poscoroots/coroot_radical/mod_central_torus_info/adjoint).

## Batch status (2026-08-09)

Weyl layer landed (`9111b7d`, agent-30): walls/walls_attitude
(alcoves.cpp:112-236), Weyl_orbit/Weyl_orbit_ws both argument orders
(rootdata.cpp:1690-1876), from_dominant corrected (lattice_rank torus
pass-through, true simple-root pairings). RootNumbering keys on coroot
level/coordinates when the datum prefers coroots (rootdata.cpp:164-167).
Fixtures weyl_orbit(+_rejected)/walls(+_rejected) frozen from the local
pinned oracle; HPC differential pending.
**B2 block_sizes root cause fixed**: fiberSize is the STRONG-real fiber
orbit class size (innerclass.cpp:603-614), not the adjoint weak partition;
`fiber_size` switched to `StrongRealClassification::fiber_size`, B2 rows
restored in the block_sizes fixture (oracle 4/5/12 now reproduced).
~~Known gap: Weyl_orbit/Weyl_orbit_ws oversize-vector semantics (wrapper
does no size check; v.size()!=rank output diverges from the oracle,
details in docs/slices/post_weyl_lang_queue.md §1.5).~~ STALE as of
2026-08-20: probed `Weyl_orbit(rb,[1,2,3,4])` on rank-2 B2 — Rust output
is byte-identical to the oracle (both print the 8-column orbit matrix
using the leading entries). Also `integrality_points` RatVec-list display
(the row-3 caveat at the batch table) now matches the oracle verbatim
(`[2/3,1/1]` / `[]` for [1,1]/2 and [1,0]/3 on B2).

Remaining (unchanged): ~~alcove_center/alcove_root_vertex,
FPP_numers/FPP_w_shifts, root_expression/root_index/root_permutation
(oracle root numbering), root_ladder_bottoms/coroot_ladder_bottoms~~ ALL
STALE as of 2026-08-20 (direct local probes vs the pinned oracle, B2):
alcove_center(Param), alcove_root_vertex, FPP_numers/FPP_w_shifts
(incl. the "Rational weight is not in fundamental alcove (coroot -4,
value N/D)" runtime wording), affine_orbit_ws, root_ladder_bottoms/
coroot_ladder_bottoms, root_expression/root_index/coroot_expression/
coroot_index, root_permutation(WeylElt), root_involution(RootDatum,int),
semisimple_rank, two_rho_check, simple_coroots, poscoroots,
coroot_radical, mod_central_torus_info, adjoint(LieType,bool) — every
one registered with the oracle's signature and byte-identical output.
basic_orbit_ws shares the oracle's (RootDatum,[int],int) signature.
The genuinely-open remainder is the ext_param+star family (incl.
shift_flip), the finalise-three partial signatures, print_X
(GlobalTitsGroup), reducibility_points, the KType/Rep skips, and the
locator non-identity-attitude gates. The
ext_block builtins (extended_block/raw_ext_KL/partial_extended_KL_block —
crate side landed 28e6109 + ext_kl in flight; shift_flip;
finalize_extended/K_type_pol_extended/scale_extended; dual_KL_block),
block_deform series (block_deform/twisted_deform/twisted_full_deform/
KL_block/twisted_KL_sum_at_s), and the print family (print_X/
print_gradings/print_real_Weyl/print_blockstabilizer/print_common_block).

## Batch status (2026-08-06, updated 02:50)

Second sweep round: 56 more skip-registrations live-ized (arms already
implemented) — Cartan_* family, integrality family, simple_roots/
simple_factors/simply_connected/adjoint/derived_info/fundamental_*/
is_Cartan_matrix, Smith_Cartan, posroots/nr_of_*/prefers_coroots,
occurrence_matrix, partial_block, raw_KL, two_rho, strong_components,
normal/theta_stable/to_canonical_fiber/dominant, torus_*, dual_real_form(s).
All 82 non-rejected domain fixtures diff clean; the 42 rejected fixtures
differ only in the known L1 Runtime-error line format. Reverted (kept
skip) where arms were partial: first_term, K_type_pol, truncate_above_height,
KL_block (common-block/PolP gaps). `integrality_datum` now keeps the full
lattice (A1.T1 at half-integral) with SC/Other isogeny. HPC differential
`3520179` running (cargo offline + synced cache/index).

## Batch status (2026-08-06)

Overnight sweep (00:40-01:05 local) landed ~25 more builtins, all
VERBATIM against the oracle on A2/B2/G2/A3/A1A1 probes:

- `cofolded` (InnerClass->RootDatum): fold_orbits + cofold via
  `RootInvolutionData::image_permutation`; B2 identity, A2/G2/A3 split
  (A1.T1), and the orthogonal A1A1 two-type pair all byte-identical.
- KType predicates: `height`, `is_standard`, `is_dominant`, `is_zero`,
  `is_final`, `is_semifinal`, `dominant`, `to_canonical_fiber` (live
  registrations; the dominant/normal/theta_stable/to_canonical_fiber
  transform arm already existed).
- Param predicates: `height`, `is_standard`, `is_dominant`, `is_zero`,
  `is_final`, `is_semifinal` (StandardRepr methods 2500-2603).
- `dual_datum` (InnerClass->RootDatum, G->dual_datum),
  `quasisplit_form`/`dual_quasisplit_form` (InnerClass->RealForm via
  build_real_form + quasisplit_external).
- `dual` overloads (RootDatum->RootDatum rd->dual(), InnerClass->InnerClass
  G->dual(), Block->Block) — the RootDatum arm uses `dual::dual_datum`
  (now `pub`).
- `form_names`/`dual_form_names` (InnerClass->[string] via
  RealFormPresentation::name), `form_number`, `distinguished_involution`.
- `root_datum` InnerClass coercion (G->datum), `central_fiber`
  (strong_real::central_fiber -> [vec]), `KGB_size`.
- `cross` (int, Param -> Param): repr.cpp:891-910 port (made_dominant +
  gamma_lambda - pos_neg real-root correction + simple reflection +
  sr_gamma). `Cayley` (int, Param -> Param): repr.cpp:943-1002 port
  (ImaginaryNoncompact raise with parity/rho_r corrections, or real
  inverse-Cayley with parity gate; Cayley_error passes the input
  parameter back unchanged).
- Live registrations for `rank` (RootDatum/LieType), `length`
  (KGBElt), `orientation_nr` (Param) — arms already existed.

Remaining (unchanged): walls/walls_attitude, Weyl_orbit family,
alcove_center/alcove_root_vertex, FPP_numers/FPP_w_shifts,
root_expression/root_index/root_permutation (oracle root numbering),
root_ladder_bottoms/coroot_ladder_bottoms (root_perm/link), then the
ext_block layer (extended_block/finalize_extended/partial_extended_KL_block/
dual_KL_block/K_type_pol_extended/scale_extended/raw_ext_KL/shift_flip),
block_deform series (block_deform/twisted_deform/twisted_full_deform/
KL_block/twisted_KL_sum_at_s), and the print family (print_X/
print_gradings/print_real_Weyl/print_blockstabilizer/print_common_block).


The language gate is complete (166/166 frozen fixtures verified_hpc).
The upstream interpreter registers 132 distinct builtin names; the Rust
typed layer registers 102. This ledger tracks the 50 missing names in
implementation batches. Each batch follows the per-slice loop: probe
the oracle (local `/Users/hoxide/mycodes/atlasofliegroups/atlas` works),
freeze a fixture, implement, gate, HPC differential, meta upgrade.

## Batch status (2026-08-05)

| Batch | scope | status |
|---|---|---|
| 1 | root-datum surface | DONE (simple_roots/simple_coroots/is_Cartan_matrix/dual_datum, two_rho, fundamental_weight/coweight, simple_factors, Cartan_matrix_type) |
| 3 | root/radical data | DONE except root_ladder_bottoms/coroot_ladder_bottoms (need root_perm/link); integrality_rank/integrality_datum/is_integrally_dominant DONE `174ae58` (fixture `domain/integrality` VERBATIM; integrality_points implemented but its RatVec-list display differs from the oracle RatNum list — recorded in meta) |
| 4 | print family | PARTIAL (RealWeyl crate ported `51b9d83`; still needs global KGB for print_X, srm pools for print_common_block, language-layer wrappers for the rest) |
| 5 | W-cells/KL | DONE except twisted_KL_sum_at_s (needs ext_block) |
| 6 | extended blocks | PARTIAL (default_extended/extend/partial_block/partial_KL_block done; rest need ext_block layer) |
| 7 | deform variants | PARTIAL (full_deform done; rest need block_deformation_to_height / common-block srm pool) |
| 8 | misc | DONE except shift_flip (needs ext_block); Cartan_matrix_type done |

Remaining (recorded): walls/walls_attitude (weyl::wall_set), from_dominant (WeylElt decompose),
derived_info / mod_central_torus_info (PreRootDatum projector), cofolded (construct_cofolded),
Weyl_orbit family, alcove_center/alcove_root_vertex, FPP_numers/FPP_w_shifts, root_expression/
root_index/root_permutation (oracle root numbering), then the ext_block / print / block_deform
layers. Performance work (2026-08-04/05) is in docs/BENCHMARKS.md: E6 13.7s->0.45s warm, E7 10.3s/4.1GB
->8.4s/2.2GB via rho-descent longest, compact [u8;8] WeylElt, u8 root permutations, full-content
classification cache, rayon parallelization (7 sites).

## Batch status (2026-08-01)

| Batch | scope | status |
|---|---|---|
| 1 | root-datum surface: simple_roots, simple_coroots, is_Cartan_matrix, dual_datum(InnerClass) | DONE `4857d2a`, fixture `domain/simple_roots` VERBATIM |
| 2 | KGB Bruhat printers: print_KGB_order, print_KGB_graph (KgbGraph::bruhat_hasse, n_bruhat_comparable) | DONE `0894ccf`, fixture `domain/kgb_bruhat` VERBATIM |
| 3 | root/radical data | DONE: root_coradical, coroot_radical (`domain/radical`), components_rank, strong_components (`domain/components_rank`), two_rho/two_rho_check (`domain/two_rho`, HPC `3507991`) all VERBATIM + HPC. Only root_ladder_bottoms / coroot_ladder_bottoms remain (they need the root_perm/link permutations of rootdata.cpp:243-313 that RootTable does not store). |
| 4 | print family: print_X, print_gradings, print_real_Weyl, print_blockstabilizer, print_common_block | NOT STARTED — print_X (KGB global), print_gradings (Cartan grading bits + Bourbaki numbering of the imaginary subsystem), print_real_Weyl (real Weyl group), print_blockstabilizer / print_common_block (common-block stabilizer) all need deeper layers (global KGB, realweyl, srm pools); print_gradings additionally needs the oracle's root numbering for the simple-root listing. |
| 5 | W-cells and KL access: W_cells, W_graph, KL_column, raw_KL, raw_ext_KL, dual_KL, KL_sum_at_s, KL_sum_at_s_to_height, twisted_KL_sum_at_s | DONE EXCEPT twisted_KL_sum_at_s: W_cells/W_graph(Param) (`domain/w_graph_param`), raw_KL/dual_KL (`domain/raw_kl`), KL_sum_at_s/_to_height (`domain/kl_sum_at_s`), KL_column (`domain/kl_column`, HPC `3508248`) all VERBATIM + HPC. The KL-table Cayley argument-order fix (`24ba188`) unlocked B2/G2 KL (HPC `3508004`); the multi-bit grading-shift fix (`fbed749`) unlocked A3+ dual real forms (raw_kl covers A2/B2/G2/A3/D4, HPC `3508109`; w_graph_param/kl_sum_at_s cover A3, HPC `3508132`) — all 0 FAIL. twisted_KL_sum_at_s needs ext_block. Known: KL_sum_at_s uses the input parameter's lambda-rho for every block element (height-parity mismatch for mid-block parameters; fixtures use the block's lowest element). |
| 6 | extended blocks: default_extended, extend, extended_block, finalize_extended, partial_block, partial_KL_block, partial_extended_KL_block, dual_KL_block, K_type_pol_extended, scale_extended | PARTIAL: **default_extended** COMPLETE (`fab1593`+`6855ca2`) — the 4-tuple (lambda, tau, l, t) via the srm gamma-lambda unique mod X* (real_unique) + ell, with the generic twist solved by matreduc::find_solution (exact rational Gaussian elimination); A2 identity + A3 non-identity byte-identical; **extend** (`9b0abbb`); **partial_block** (`domain/partial_block`, HPC `3511402`); partial_KL_block (HPC `3511377`); the rest need the ext_block layer. |
| 7 | deform variants: twisted_deform, twisted_full_deform, block_deform, full_deform, KL_block | PARTIAL: **full_deform** (`domain/full_deform`, HPC verified) — finals_for + reducibility-point recursion; the rest need block_deformation_to_height (repr.cpp:2027-2124, the partial-block deform recursion) and/or the common-block srm pool (KL_block needs lookup_full_block + survivors condensation). |
| 8 | misc: Cartan_info, KGB_Hasse, block_Hasse, orientation_nr, shift_flip | DONE except shift_flip: Cartan_info (`domain/cartan_info`), KGB_Hasse (`domain/kgb_hasse`), block_Hasse (`domain/block_hasse`), orientation_nr (`domain/orientation_nr`) all VERBATIM + HPC. shift_flip needs the ext_block layer (Batch 6). |

## Oracle-probed shapes (A2)

- `simple_roots(simply_connected A2)` → `| 2, -1 | / | -1, 2 |`; `simple_coroots` → identity
- `is_Cartan_matrix([[2,-1],[-1,2]])` → true; identity → false
- `dual_datum(ic)` → `adjoint root datum of Lie type 'A2'`
- `print_KGB_order(rf)` → kgbsize + Hasse rows + comparable-pair count
- `print_KGB_graph(rf)` → Graphviz digraph with black/blue/green/gray edges
- `root_coradical(simply_connected A2)` → Cartan rows (coradical empty); `coroot_radical` → identity
- `root_coradical(adjoint A2)` → identity; `coroot_radical` → Cartan rows
- `root_ladder_bottoms(ra, 0)` → `[-3,-1,0,1]` on A2
- `Cartan_info(CartanClass)` → `((2,0,0),[ ],(1,4),(A2,empty,empty))` on A2 — the first triple is
  `classify_involution` (already ported, identity A2 → (2,0,0) verified)

## E6/D5 column-echelon — RESOLVED (2026-08-04)

The `RealProjection::build` port is fixed. Root cause: the incremental
column-echelon port is not equivalent to C++'s one-shot `column_apply`.
The fix (commit 248aeb9) combines:
1. one-shot ops-matrix sweeps with `ops(mindex,mindex)=-1` recorded
   (matreduc.h:70-122 + column_apply);
2. Euclidean row-reduction inverse of the unimodular `col` matrix
   (no scaling division);
3. HISTORICAL CLAIM, DISPROVED2026-09-29: this repair chose truncating
   division in `lambda_unique` and incorrectly claimed divide(-1,2)==0.
   Current arithmetic.h249-253 gives-1. Original3840093 and failing Rust
   regression3840098 demonstrate wrong A2 canonical keys/term coalescing.
   Keep the old evidence, but do not use those anchors as authority to
   restore truncation. See slices/ktype_formula_2026-09-29.md; Euclidean
   candidate3840172 still requires all targeted/full regression gates.
E6 involution 187 and the D5 so*(10) real form now factor
`lift_mat * M_real == 1-theta`; E6/D5 KL_column, deform, raw_KL,
KL_sum_at_s all byte-identical vs the oracle. E7 kgb_hasse verified on
HPC fat (swap 3515688: 506s, 12.4G peak RSS).

## E6 column-echelon debugging notes (2026-08-03, resolved upstream)

The `RealProjection::build` port of `matreduc::column_echelon` fails its
`lift_mat * M_real == 1-theta` check for E6's involution 187. The
investigation produced these verified facts (all in Python reproductions
and Rust experiments):

1. The original incremental port (`column_operation` mutating `a`
   directly) is NOT equivalent to C++'s `column_apply(M, ops)` one-shot
   semantics — the E6 factorization only holds with the one-shot ops.
2. With one-shot ops, E6 involution 187 needs BOTH the local-pivot
   flip (`row[mindex] = -row[mindex]`) AND `ops(mindex,mindex) = -1`
   recorded: `flip+record` -> zero_columns=4 check=True;
   `flip+no-record` -> zero_columns=2 check=False.
3. `col` is unimodular but the plain Gauss-Jordan inverse with scaling
   division breaks on non-±1 pivots; the Euclidean row-reduction inverse
   (row swaps + subtractions only) is the working variant.
4. CONTRADICTION: the A2 su(2,1) anchor `K_type(x4,[1,0])` gives
   lambda_rho [1,0] in the oracle (== the no-record variant), while E6
   needs the record variant. The same C++ code cannot produce both under
   the current simulation — the A2 single-active-column flip+swap case
   must cancel the recorded -1 differently in C++ (matrix.h
   swapColumns/columnApply interplay), which the simulation misses.
   Root-cause understanding of that cancellation is the open task.

Suggested next step: instrument the real C++ `involutions.cpp` build for
A2 x4 (or read `matrix.h`'s PID_Matrix swapColumns/columnApply once more
for hidden sign flips), then reconcile the A2/E6 split.

## D5 column-echelon limit (2026-08-04)

The E6 involution-187 `RealProjection` failure also hits D5: the so*(10)
real form's `KL_sum_at_s` panics on "image basis factorization". The
same root cause (incremental column-echelon port vs C++ one-shot
`column_apply`, see the E6 notes below). `raw_KL` on the D5 block passes,
so the block graph itself is fine — only packet involutions of certain
real forms trip the projection. Verified fixtures must avoid D5/D6+ real
forms until the column-echelon port is reconciled.

## Root-index builtins limit (2026-08-04, detail) — STALE, unblocked 2026-08-11

**2026-08-11 update: this limit is stale.** RootNumbering
(domain_builtins.rs:2809-2880) now ports the oracle order and is
differential-verified on B2 (fixtures 3516408 posroots order
`[1,0],[0,1],[2,1],[1,1]`, 3533446 walls). See
docs/slices/post_weyl_lang_queue.md §5.6. The notes below are kept for
history.

The oracle's B2 positive-root order is [1,0],[0,1],[1,2],[1,1] (probe):
root_expression(rb,2) = [1,2] = alpha_1 + 2 alpha_2, so the oracle's B2
uses the Bourbaki numbering (alpha_1 SHORT), while Rust's standard B2
Cartan [[2,-2],[-1,2]] has alpha_1 LONG. The oracle `ri` order is the
roots_at_level generation order (rootdata.cpp:144-219), which depends on
this numbering; mapping oracle RootNbrs to Rust roots therefore needs the
Bourbaki simple-root renumbering first. That renumbering would touch the
whole RootDatum surface (simple_roots, Cartan_matrix, KGB block orders),
so the root-index family stays unimplemented and fixtures avoid it.

## Root-index builtins limit (2026-08-04) — STALE, unblocked 2026-08-11

See the 2026-08-11 note on the detail section above; kept for history.

`root_expression`/`coroot_expression`/`root_permutation`/`root_involution`
take an oracle RootNbr (internal_root_index: N + numPosRoots, positive
roots only). The Rust `RootSystem::roots()` orders positive roots by
ambient-coordinate lexicographic order, which differs from the oracle's
`ri` (roots_at_level) order — and the oracle's B2 order ([1,0],[0,1],[1,2],
[1,1]) is not the naive height/level order either, so a simple re-sort
does not match. Porting the oracle's level-generation order (rootdata.cpp
:144-219) is the open task; until then the root-index family stays
unimplemented (fixtures avoid them).

## Known structural limit: E6-and-larger Rep_context

The `RealProjection::build` column-echelon port (matreduc.h:129-161,
the `1-theta` image basis) fails its `lift_mat * M_real == 1-theta`
check for E6's involution 187 (packet 74): product 7 vs expected -1 at
entry (0,5). Every smaller rank (A1..A4, B2..B4, C3/C4, G2, F4, D4)
passes; the E6 class-1 real form's KL/deform surface is therefore
unavailable (KGB_Hasse still works — it does not build a Rep_context).
The failure is in the column-echelon port (or its divisor semantics),
not in the KL machinery. Fixing it unlocks E6 KL_sum_at_s / deform /
W_cells and is a 1-2 hour debugging task against upstream matreduc.

## Polynomial term owner identity and integer conversion (2026-08-13)

- The upstream KTypePol/ParamPol term wrappers compare the owning real-form
  `shared_ptr`, not structural real-form equality.  Rust therefore keeps
  canonical/default real forms in one logical owner class, while every
  genuinely custom real-form construction receives a distinct owner token.
  In contrast, `equivalent(KType,KType)` and `equivalent(Param,Param)` compare
  structural real-form values and must accept identical custom constructions.
  Keep this distinction when moving ownership into the future session
  `Rep_table`; `Arc::ptr_eq` alone is also wrong because Rust currently
  rebuilds canonical real-form values.
- `big_int::int_val()` accepts exactly the signed 32-bit range.  Oracle probes
  confirm that positive `2147483648` reports `Integer value to big for
  conversion`; it does not wrap to `-2147483648`.  Preserve the checked `i32`
  conversion for Weyl generator builtins and their no-value validation paths.
- Bulk polynomial term-list addition is a volume-oriented API upstream.  The
  Rust implementation appends expansions, sorts once, and linearly coalesces
  equal terms; do not regress it to repeated linear `Vec::position` merging.

## Parameter twist sentinel contract (2026-08-13)

- The explicit outer twist of an otherwise valid KGB element or parameter can
  produce upstream `UndefKGB`, whose language-visible number is `~0u`, printed
  as `4294967295`.  It is a real observable value, not the same outcome as the
  `Inexistent KGB element` diagnostic.
- Rust models that value explicitly but never treats it as a `KgbGraph` index.
  A parameter sentinel also retains the already transported lambda/nu needed
  for the upstream display.  Ordinary follow-up operations reject the
  sentinel through stable checked paths rather than indexing or panicking.
- Unary `twist(Param)` first calls `make_dominant`; `twist(Param,mat)` validates
  compatibility and twists the parameter exactly as supplied.  Do not merge
  these paths even when the matrix is the distinguished involution.
- Oracle jobs `3543783`, `3543792`, `3543798`, and `3543906` pin the
  nonstandard case, both sentinel constructors, and the safe field-only
  surface. Strict equality, `%`, `height(Param)`, and `real_form(Param)` remain
  valid on the sentinel; only operations that need a graph element reject it.
  The P3 differential must include all six twist fixtures before closure.

## Full-deform outer term merge contract (2026-08-13)

- `full_deform(Param)` accumulates its outer result by KType key. Distinct
  KTypes with equal Split coefficients must both survive; equal KTypes combine
  coefficients and zero sums disappear. The previous coefficient-only merge
  silently discarded valid terms.
- Reference capture `3543807` pins a minimal A2 two-term result and its rank
  rejection. This is only a narrow polynomial-accumulation repair; it does not
  complete proper-subsystem deformation, high-denominator alcove shrinking,
  recursion, cancellation/deadline support, or `full_deform(Param,int)`.

## Param W-graph type and generic row-size contract (2026-08-13)

The next deformation boundary is frozen by oracle job `3546215`: rank-one
gamma denominator 3 crosses the `2^rank` threshold, `alcove_center` visibly
changes `nu` from `[1]/3` to `[1]/2`, and both full deformation variants
return the pinned one-term KType polynomial. The accepted/rejected captures
took 0.012/0.008s and 4368/4288 KiB; report SHA256 is
`623e0650b86d18c795ba5d35b851f75cb681fb071b310cde3102b409759f9c2a`.

- `W_graph(Param)` returns `(int,[([int],[(int,int)])])` and
  `W_cells(Param)` returns `(int,[([int],[([int],[(int,int)])])])`.
  Treating the edge lists as `vec` preserves printed values but breaks nested
  destructuring and overload resolution; reference capture `3543933` pins the
  exact accepted and rejected static types.
- Unary `#` on a row is an interpreter special operator in `axis.w`, not one
  of the 305 `atlas-types.w` `install_function` entries.  Registry audits must
  therefore include core generic operators as a separate language surface.
  Its contract is polymorphic `#([T])->int`, hunger 0, including the unstable
  empty-row type `[*]`; keep its wildcard matching local to unary `#` so that
  undetermined types do not become generally coercible.

## Builtin hunger contract (2026-08-13)

- The fourth `install_function` argument called `hunger` is not a coercion or
  overload-selection mask.  `axis.w` uses it when a simple assignment feeds
  the destination value back into a builtin: it controls pilfer/in-place reuse
  and, for hunger 1, right-to-left argument evaluation.  Signature inventories
  must compare hunger separately from `(name,args,result)` compatibility.
- A hunger mismatch is directly observable only when the builtin result can be
  assigned back to the consumed destination type.  The current actionable
  cases are `LieType*LieType`, `WeylElt*vec`, and `vec*WeylElt`; the fixture
  pair `hunger_contract{,_rejected}` pins alias preservation, assignment
  results, evaluation gates, and rank diagnostics.  The other domain entries
  are retained as metadata/algorithm coverage, not mislabeled as type gaps.
- The three observable cases are now implemented with the upstream
  simple-assignment rewrite: hunger 1 evaluates the non-pilfered right operand
  first, hunger 2 remains left-to-right, local and global destinations are
  moved out of their slots, aliases retain copy-on-write values, and a failed
  builtin leaves the destination uninitialized.  Oracle capture `3545219`
  pins the accepted/rejected runtime and assignment contracts.  The five
  runnable fixtures pass exact in fat differential `3545729` at `196dd7c`
  (0.004-0.006s, 5920-7316 KiB; report SHA256
  `b0285ed87cf6898c245edbc1ea476d21b90468277c86c10e53f25a7f6b634bda`).
  The timed `twisted_full_deform(Param,int)` probe is now implemented and
  included. Differential `3564233 @ 8851395` passes the positive, cache,
  timeout, and validation-order contracts exactly (0.006-0.007s,
  7080-7276 KiB; report SHA256
  `1c24fcb33dc4d60755d0b1e0434fa5390e687b44d6731efa18e14029927ed107`).

## Arbitrary-root parameter transforms (2026-08-13)

- `cross(vec,Param)` and `Cayley(vec,Param)` use the parameter's integral
  subsystem, not an ambient simple-root shortcut.  The port preserves parent
  root/reflection words, negative-root normalization, KGB word direction,
  lambda/gamma/y-bit shifts, undefined Cayley returning the original Param,
  and the wrapper's no-value skip.
- `any_Cayley` makes a standard source integrally dominant before validating
  the supplied root; therefore a nonstandard source plus an invalid vector
  reports `Cannot make non-standard parameter integrally dominant` first.
  Oracle jobs `3545170` and `3545520` pin A2 simple/non-simple/negative paths,
  a successful three-reflection A3 dominance/rebuild, and that diagnostic
  ordering. The recorded A3 word `[1,2,1]` is palindromic, so exact root-word
  iteration direction remains justified directly by the root-first overload
  `SubSystem::permuted_root(rt,w)` (`rootdata.h:320-324`) rather than by that
  fixture alone; add a non-palindromic oracle case when one is available.

## Timed full-deformation contract (2026-08-13)

- Oracle job `3547426` freezes the missing
  `full_deform(Param,int)->(void|KTypePol)` overload. A completed computation
  uses the `.done` union branch; fresh-process deadlines of `0` and `-1`
  milliseconds use `().timed_out`.
- The timeout is cooperative and cache-sensitive, not a shell/process timeout.
  A discarded timed call does not warm the completed-result deformation
  cache, unary `full_deform` does warm it, and a later zero-millisecond call
  can therefore complete. Integer narrowing still runs before the no-value
  early return and diagnoses an oversized timer.
- The four captures took 0.009--0.014 seconds and 4344--4484 KiB RSS; report
  SHA256 is
  `97931b44e402672b0704a1caca595fcb4e5c91582d95325ab3ff82536fb75b04`.
  Rust commit `3b42183` implements the overload with a typed per-real-form
  completed-result cache and cooperative checks in the recursive ordinary
  deformation loops. Differential job `3551338` matches all four fixtures
  exactly (0.008s, 6972--7104 KiB); report SHA256 is
  `d59adb977b717ab1f43559f877ee8f64896d8b64a7e887da86b99341afaa31d0`.
  Partial formula progress is not retained after timeout and remains an
  unprobed compatibility/performance boundary.
# Latest K-type boundary and script frontier — 2026-09-29

See [K-type formulas/cache contracts](slices/ktype_formula_2026-09-29.md).
Original3840093 verifies signed32 wrong acceptance in existing Rust memo
K_type_formula, plus missing raw and four stored interfaces. A raw/memo/cache
candidate is being frozen; no after-pass or acceleration claim yet.
Selector3840085 is fully verified500core/CLI/source integrity and84whole
positive language matches, no losses versus F4candidate3840083's83.
basic.at now stops at2437's KTypePol[KType] coefficient subscription. Capture
KTypePol/ParamPol positive, rejected and no-value boundaries before porting.
Release3840096 reuses exact3840085source for mathematical revalidation.
Stored queries, high-level mathematics and E7/D8 capacity remain open.
# Latest continuation — polynomial544 finalized, recursive groups remain

FINAL5523844656 passes full build/source/245capture with114whole positives/no
losses. Four high-level libraries now LOAD successfully, but their full stdout
differs and actual mathematics still needs the exact552all116consumer suite.
New251boundary capture3844760 runs the same552binary separately to preserve
known remaining grammar/anonymous-argument display mismatches before R2.

Live3844656 now passes3after regressions,5graph controls and all552core.
Before3fail retained. CLI/release/245capture/final guards remain outstanding;
new251boundary discovery3844689 is separate, not a changed candidate catalog.

Candidate552group job3844656 is submitted, not accepted. It couples generic
grammar/formal forwarding to full RHS SCC identities and staged publication,
with three unchanged session regressions, five graph controls and245capture.
Do not treat declarations or unit success as AV-ann/cycle mathematics.

Build3844249 FINAL passes five before/after regressions,544core/CLI/source and
243capture (110whole positives/no losses). Followup3844321 FINAL uses the SAME
binary for245cases (111whole positives/no losses); local/captured/user-op
effects and all six negative diagnostics/survivor output pass. Negative full
stderr differs and original foreign-owner corruption stays an explicit
exception. Reports are indexed in HANDOFF/MATH_VALIDATION. All four high-level
imports still fail lazy_lists.at5; generic mutually recursive type groups are
the next shared blocker. Three original-backed regression tests precede work.
