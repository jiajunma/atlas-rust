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

The catalog now contains93 cases, not93 verified successes. Broad array3832011
and independent review3832103 are complete. The review rehashed the pinned
sources, scripts, binaries and all54 complete result sets. Artifact:
`tests/reference/hpc/math_suite_broad_review_2026_09_28.json`, SHA
`31c0e62d1a6b9b3227cfc2b55fc5ea213a29bb75f82a5f5cb28f1986feea1250`.
Do not infer mathematical acceptance from SLURM COMPLETED or replace timed-out
cases with easier inputs under the same ID.

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
