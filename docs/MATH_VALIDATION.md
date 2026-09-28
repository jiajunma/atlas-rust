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

The catalog now contains92 cases, not92 verified successes. Additional
54 B2/C2/D4/F4/E6/E7 cases are running as array3832011; independent review
3832103 is queued after that exact array. Do not infer acceptance from SLURM
COMPLETED alone or replace timed-out cases with easier inputs under the same ID.

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

- General associated cycles: orbit identities AND multiplicities, independent
  dimension/rank checks, boundary support and justified cutoff completeness.
  AV-ann, AV support and bounded Phi matrices do not satisfy this requirement.
  Old SL2/product numerical examples must be revalidated against this baseline
  and supplemented by meaningful classical/exceptional examples.
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
