# Mathematical validation status — 2026-09-28

## Baselines and acceptance

Rust remote main `05625c5d` versus original `7e1b958c`, with complete archive
pins in `tests/math/baseline.json`. Build3831844 was performed on an HPC
compute node. Reviews3831996 and3832014 independently rehashed606 original
and1212 Rust source files,264 upstream scripts, binaries and full raw outputs.
Historical optimized/PGO worktrees are separate candidates, not this main.

All builds, tests, differential executions, verifier tests and benchmarks run
on HPC. Positive acceptance requires successful execution, no error stream and
complete mathematical output equality; rejected inputs have explicit category
checks. Equal errors are never a mathematical pass. Time and peak RSS are
retained even for failures, but failed/partial executions are not speedups.

## Independently reviewed A2/G2 pilot

| Operation | Evidence | Conclusion limited to the tested inputs |
|---|---|---|
| Root data |3831996|Both groups match, including exact identities.|
| KGB enumeration |3832014|All elements and simple-root operations match.|
| FPP builtins |3832014|Full numerator/shift results match at zero, wall and interior inputs.|
| FPP wrong rank |3831996|Both groups reject in the intended category.|
| KLV, unitarity, AV-ann, cycle prerequisites |3831996|Original succeeds; Rust fails loading latest `basic.at`. No numerical comparison reached.|
| Hodge bound4 |3831996|Original also fails: A2 negative branch level, G2 real-form mismatch. Not accepted.|
| Generic pair, any_type, basic.at load |3832014|Three minimal cases confirm the Rust language gap.|

The first KGB/FPP templates incorrectly assumed script-defined unary matrix
minus and `ones` were bare builtins. The corrected templates use `0-mat` and
an explicitly constructed vector. R2 failed inputs are retained on HPC; no
upstream script was rewritten and no error was suppressed.

Reviewed JSON artifacts:
- `tests/reference/hpc/math_suite_pilot_review_2026_09_28.json`
- `tests/reference/hpc/math_suite_corrective_review_2026_09_28.json`
- `tests/reference/hpc/math_suite_r3_preflight_2026_09_28.json`

The catalog now contains108 cases, not108 verified successes. Case107 extends
the F4 partial-KL panic reproducer to cold/full/warm history at three scales;
review3832614 confirms original success but Rust's cold half-scale integral
image positivity failure. Broad array3832011
and independent review3832103 are complete. The review rehashed the pinned
sources, scripts, binaries and all54 complete result sets. Artifact:
`tests/reference/hpc/math_suite_broad_review_2026_09_28.json`, SHA
`31c0e62d1a6b9b3227cfc2b55fc5ea213a29bb75f82a5f5cb28f1986feea1250`.
Do not infer mathematical acceptance from SLURM COMPLETED or replace timed-out
cases with easier inputs under the same ID.

## Repair candidates (not remote main)

Latest script-loading prerequisite: constructor commitadf40792 is verified
by HPC3835776 (413core tests:411pass, two retained known assignment failures
each executed separately;155filtered checks and CLI pass). Its62-case capture
has18 complete original matches, previously15. Explicit generic constructors,
arity rejection and row/function structural uses are now connected; generated
projector/injector inference, any_type and latest basic.at loading remain open.
Member replays expose wrong concrete-overload selection and rejected field
writes, plus obsolete positional-union syntax still accepted by Rust. Keep
these failures in the separate generic prerequisite corpus; they do not replace
the108-case classical/exceptional mathematical catalog. See the indexed
generic-language slice for exact originals, pins and subsequent captures.
No debug-candidate timings are speed ratios, and no new high-level unitarity,
Hodge, AV-ann or associated-cycle acceptance is claimed by this language work.

Latest candidate3833716 is independently reviewed by3833739:
14 complete mathematical matches and1 expected rejection across15 cases.
This includes A2/B2/C2/D4/G2/F4/E6 core KL, D4 FPP wall, partial-KL
history/containment, compact-A1 nonstandard rejection and full F4 history.
The same E6 missing-Cayley regression executes/fails before and passes after
at integer/half scale; prior targeted units and release build pass.
Build SHA62aa96181c87d0ae816928dcbab37b5b7b448d1d1d00632106e28214319b2e63;
review SHA4a7f3cc2b77832f553977336912ca6dcb2fc6889fa48454f73d40e8dfd6ca98e.
E6 observed0.8952s/77364KiB original versus1.5082s/135536KiB Rust;
F4 history0.1811s/9440KiB versus2.1913s/22816KiB. These are single-shot
small-case timings, not minute-scale repeated speedup benchmarks.
Scoped correctness fixes can be retained; the complete user objective is open.

Generic capture3833740 records11 independent language probes with time/RSS.
It disproves concrete-overload preference and duplicate-formal rejection
hypotheses. Original accepts5 probes; Rust accepts none. Arity diagnostics
need exact contract review, and equal syntax labels alone are not acceptance.
High-level unitarity/Hodge/AV/associated-cycle coverage is still blocked by
latest basic.at, not repaired by the bare-core KL successes above.
Follow-ups3833758/3833787 expand this prerequisite library to17 probes and
confirm a separate wrong acceptance of polymorphic empty-row assignment.
The explicit `[int]` control matches; before unit3833788 executes/fails on
the incorrect polymorphic acceptance. See the indexed generic-language slice.

Earlier candidate progression (superseded only for the cases covered above):

F4 endgame candidate3833612 now has unchanged-unit before-failure/after-pass
AND complete retained original-backed history equality. Corrected independent
review3833712 verifies13mathmatches/1rejection/1E6 failure across15cases,
rechecking all source files, scripts and raw streams. First review3833635
failed archive-path spelling before mathematics and is retained. Only safe
relative archive names were canonicalized; no numerical output was normalized.
E6 still panics on a missing second imaginary-II Cayley image; probe3833624
reproduces it. Its minimal candidate3833716/array3833718 is submitted after
56 passing checker tests, with no after-pass claimed yet.

Latest review3833590 (E6 coordinate candidate3833558) independently confirms
20mathematical matches,1expected rejection,1F4 coefficient mismatch and1E6
KL failure. All8 KGB cases match, but E6 core KL now reaches a deeper
`second image` panic, so removing its constructor failure is not blanket
acceptance. F4 probe3833589 reproduces P(4,334)'s wrong coefficients in both
fresh-full and history contexts. A minimal endgame control-flow candidate
is being tested separately; before/after and full differential are required.
Artifacts: `math_e6_form_repair_review_2026_09_28.json` and
`math_f4_full_kl_probe_2026_09_28.json` in `tests/reference/hpc`.

The following paragraphs retain the earlier repair progression.

FPP reflection-word candidate build3832301 has an exact before-fail/after-pass
root-reflection identity test for ten classical/exceptional Lie types and both
root-numbering choices, using independent Cargo targets. Independent
review3832333 verifies17 full differential cases:8 mathematical matches,
8 intended rejections,1 E7 timeout. D4's preserved wall/positivity regression
now matches. E6 finishes with exact output in19.012s Rust versus0.282s original,
instead of the unchanged main's300s timeout. These are single-shot diagnostic
times, not a repeated speed claim. E7 still takes more than300s (2.31GiB RSS)
against24.43s original; it is NOT mathematically accepted. A separate600s
follow-up retains the original300s failure; review3832506 also records a600s
Rust timeout (2588124KiB) against24.1906s original. No whole-FPP completion claim.
Artifact: `tests/reference/hpc/math_fpp_repair_r2_review_2026_09_28.json`.

R11 independent review3832340 verifies four more baseline partial-KL failures:
A2/G2 cache-history queries lose predecessors; B2 containment triggers a
height-parity error; compact A1 nonstandard input is wrongly accepted. The
original passes all four cases. A separate candidate now routes partial KL
through actual-parameter lookup, Hasse downsets, locator-aware singular
condensation and zero/one-seeded polynomial interning. Independent review3832517
now verifies11mathematical matches and1rejection match, including the complete
A2/B2/C2/D4/G2 outputs and all four new regressions. F4 instead reaches a
deeper `cross of extremal` panic in KL recursion; E7 hits the same Rust panic
while its original leg still fails allocation. E6 remains a constructor error.
The candidate is therefore only partially verified, not accepted as a complete
partial-KL implementation. Preserve and repair these remaining cases.

F4 probe3832534 identifies the panic's legitimate absent real-II cross.
KL-only repair build3832615 eliminates that panic at unit scale but the SAME
regression then fails at half scale in the locator; there is no after-pass or
release candidate. Its dependent differential array was cancelled without
execution. The new failure remains mandatory, with an additional root/coroot
closure regression being prepared; see HANDOFF and the repair receipts.

Combined KL/coroot build3833274 executes the same before-fail/after-pass
tests. Independent review3833539 confirms12mathmatches and1rejection,
including now-complete F4 core KL output equality. F4 history case107 still
has a full-output mismatch, E6 still fails construction, and both E7 legs
fail allocation under the6GiB child cap. No full acceptance or failed-run
speed ratio is claimed. Stored-output diagnosis3833562 isolates the F4
difference: all three parameters' cold/warm partial results match; only
unit-scale FULL differs, with identical336x336 indices and parameters but
different polynomial coefficients. Retain the complete failing case.
E6 probe3833407 independently proves that the external-order map confuses
an imaginary-subsystem root position with an ambient coweight coordinate;
a separate repair is under HPC before/after and differential verification.

## Current full initial grid

This table combines the R2 pilot, the four R3 corrected KGB/FPP inputs, and
the54-case R3 broad review. It covers the initial72cases only, not all later
diagnostic additions. Every group's wrong-rank FPP rejection matched.

| Group | Root data | KGB | FPP | Script KLV | Unitarity | Hodge | AV-ann | Cycle foundation |
|---|---|---|---|---|---|---|---|---|
| A2 | match | match | match | load fail | load fail | oracle error | load fail | load fail |
| B2 | match | match | match | load fail | load fail | oracle error | load fail | load fail |
| C2 | match | match | match | load fail | load fail | oracle error | load fail | load fail |
| D4 | match | match | wrong result | load fail | oracle timeout | oracle timeout | load fail | load fail |
| G2 | match | match | match | load fail | load fail | oracle error | load fail | load fail |
| F4 | match | match | match | load fail | oracle timeout | oracle timeout | load fail | load fail |
| E6 | match | constructor fail | Rust timeout | load fail | oracle timeout | oracle timeout | load fail | oracle timeout |
| E7 | match | match | Rust timeout | oracle allocation fail | oracle timeout | oracle allocation fail | load fail | oracle timeout |

“Load fail” means the original completed but Rust rejected latest basic.at.
In cells where the original also failed/timed out, Rust independently failed
the same script load; the oracle failure must not hide this second failure.
B2/C2 Hodge errors are real-form mismatches. E7 allocation failures occurred
under the6GiB child address-space limit, not proof of incorrect oracle math.
Cycle foundation is still not a cycle-multiplicity computation.

The accepted E7 KGB observation3832151 is materially slower in current Rust:
original1.3032s /41672KiB versus Rust69.8035s /1459140KiB, about53.6x time
and35.0x peak RSS in this **single** run. This is a concrete regression lead,
not a stable benchmark estimate or a statement about the older optimized branch.
E6/E7 FPP originals completed in0.2799s/24.4600s; Rust reached300s in both
cases. Their full Rust results remain unverified, so no completed-run ratio
is claimed. D4/F4 original cycle-foundation cases take66.6s/146.7s but Rust
does not get past script loading; they cannot support a speedup claim.

## New kernel regressions and Hodge diagnostics

The user now also requires computational parallelism evaluated by A/B tests.
Existing observer timings below force one Rayon thread. The controlled1/4
thread experiment and its correctness/resource gates are documented in
[`slices/parallel_ab_2026-09-28.md`](slices/parallel_ab_2026-09-28.md).
Experiment3834321 and independent review3834377 now verify four whole-output
matches on E7 KGB, with54 reviewer tests and36 rehashed raw artifacts. On repaired
build3833716, medians are original1.327984s, Rust1 69.595196s and Rust4 23.523671s.
Paired scaling2.958576x, peak RSS+18.54%; selected for this workload only, no
global thread setting change. Rust4 remains about17.7x slower than original,
so this is NOT Rust beating C++. The historical main05625 table below remains
a separate serial baseline; do not silently substitute this later build.

R9 independently reviewed benchmark3832228 now provides four alternating,
fresh-process pairs for E7 and D6 KGB. Every mathematical byte agrees both
within and across those rounds:

| Full KGB workload | Original median | Rust median | Observed scope |
|---|---:|---:|---|
| E7 |1.35766s|69.4160s|Rust range69.3362-69.5214s; about51x slower. Only Rust meets the60-600s target.|
| D6 |0.15040s|0.82214s|Both below1s; not a minute-scale workload.|
| D8 |3.75700s (one run)|fails|Rust reaches its4000000 enumeration limit before constructing the group; no accepted ratio.|

E7 peak RSS was41640-41656KiB original and1460808-1470724KiB Rust.
This is current main05625 against original7e1, not the historical optimized
or PGO builds. The verified artifact is
`tests/reference/hpc/math_suite_benchmark_review_2026_09_28.json` (SHA
`2bf2059d8147c0f2223e91646cd1d926a22b368c6997128e67e138d539cbe408`).
Benchmarks stop after a failed pair and never calculate a failure speedup.

R9 Hodge review3832222 preserves eight additional nontrivial probes:
A2/G2 at half-scale and singular zero-scale, bounds4/20. Original A2
still fails with negative internal branching level; G2 still reports
K-type real-form mismatch. Rust independently fails generic script loading.
Full grading, character, trace and matrix outputs remain in the raw artifacts.
These failures are not confined to the earlier trivial-module probes.

R5 independent review3832147 confirms genuine problems beyond script loading:
D4 FPP returns Weyl representatives that violate the positive integral-simple
image invariant; A2/G2 partial KL blocks lose Bruhat predecessors, returning
one parameter instead of4/10, and one instead of2 at half scale. Full original
and Rust outputs are retained. Focused regression templates assert the
original-backed expectations and preserve complete mathematical output.
These failures are not fixed. Core KLV differs in A2/B2/C2/D4/G2/F4; E6 fails
Rust real-form construction, and E7's original hits an allocation failure
under the6GiB child cap. Neither failed comparison supplies a speed ratio.
R6 review3832167 confirms both focused A2/G2 assertions pass in the original
and fail in unchanged Rust, supplying explicit before-repair regression proof.

R4 review3832122 has independently verified six further Hodge failures on
the original. Raising split bounds from4 to20 does not help; complex A2/G2
examples at15/50 also fail. Back traces distinguish an empty tensor-product
sum with height-1 from a Levi/ambient real-form mismatch. Read
`docs/slices/math_failures_2026-09-28.md` for exact sources and job handles.
Original-side defects are not Rust mathematical passes or speed results.

## Requirements still open

R8 product review3832201 adds scoped numerical-cycle evidence: original
SL(2,R) x SU(2) passes30 reconstructions (15irreducibles at bounds4/6), with
multiplicities1/2/3. Exact Q ranks14/22 and full kernel dimensions27/75 are
independently checked; all8 full Phi identities bind positive/negative labels.
Rust fails latest generic script loading before the calculation. Artifact:
`tests/reference/hpc/math_suite_cycle_review_2026_09_28.json` (SHA
`30924499bb053599239cfc9095dfa666f0e3d3bd4d20b242be8ba84dad414656`).
R7's old-fixture any(Maybe<vec>) failure is separately preserved; R8 changes
only the fixture's success predicates to latest succeeds. All upstream
scripts stay unchanged. This is not general associated-cycle acceptance.

- General associated cycles: orbit identities AND multiplicities, independent
  dimension/rank checks, boundary support and justified cutoff completeness.
  AV-ann, AV support and bounded Phi matrices do not satisfy this requirement.
  The product numerical example is now revalidated on this original baseline,
  but Rust does not run it; meaningful classical/exceptional and richer
  coefficient coverage is still required.
- Hodge: investigate the preserved original failures, then cover nontrivial
  irreducibles, different bounds and singular/nonintegral parameters. Elementary
  specialization identities alone are not full filtration validation.
- Script-level FPP applications and broader basic/domain operations.
- Additional real forms, isogenies and products, not only split simply-connected
  groups. Latest language loading is a real compatibility gap, not a reason to
  silently fall back to old scripts.
- Matched repeated60–600-second correctness-accepted workloads, including D6/D8
  and E7. Current millisecond single-shot observations are not stable speed
  evidence. Memory is a capacity/OOM guardrail, not the optimization objective.

Every discovered Rust calculation error must enter the test library before
its repair, with before/after HPC evidence. Deferred repairs retain explicitly
failing cases. Never derive expected mathematics from faulty Rust output.
