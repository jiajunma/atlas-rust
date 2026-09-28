# Mathematical test and benchmark library

This library is being established against upstream Atlas **7e1b958c** (remote
master checked 2026-09-28) and Rust remote main **05625c5d** (pull was already
up to date). The historical optimized/PGO worktrees are separate candidates,
not silently substituted for main. `baseline.json` locks both complete source
archives. Neither the original installed oracle nor any historical job folder
is updated in place.

All tests, builds, differential runs and benchmarks run on **HPC compute
nodes**. Local work is limited to source inspection, editing and synchronization.

The separate `generics/catalog.json` now retains36 prerequisite-language
discovery cases (not part of the108 mathematical-case index space). It includes
global/local polymorphic-constness failures, their concrete mutable controls,
fixed-type local mutation, nested abstractions, named annotations/direct calls,
recursive result constraints and retained rejected syntax probes. Capture3834702
confirms the previous31 oracle intents; replay3834785 confirms all36 after
correcting bridge3834754's oracle loader environment. It compares original,
old Rust and a new debug candidate (not a speed benchmark); named-type behavior
improves, but complete output/diagnostic compatibility remains open.
See `docs/slices/generic_language_2026-09-28.md` for evidence and
disproved fixture hypotheses. Passing internal type units does not close the
latest `basic.at` gate or establish any high-level mathematical feature.

## Coverage and honest status

`catalog.json` expands nine cases (eight positive, one negative) for each of
A2, B2, C2, D4, G2, F4, E6 and E7: **72 initial cases**. Templates and the
expanded per-job input are both hashed. These are candidate fixtures, not
claims of verified mathematical support. A successful catalog self-test does
not verify Atlas syntax, numerical results, or performance.

Root-data, KGB and FPP cases call core builtins without loading `basic.at`.
The KGB constructor is the exact `basic.at:split_form` expression. This
separates mathematical-kernel gaps from latest script-language gaps. Higher
level unitarity/Hodge/AV tests deliberately load the unmodified latest scripts.
Upstream basic.at now contains generic types and `any_type`; static inspection
shows the pulled main grammar does not implement those declarations. HPC
execution must establish the actual failures; do not rewrite upstream scripts
to make those compatibility failures disappear.

Three additional minimal language cases (indices72–74, without moving any of
the72 domain indices) isolate generic named pairs, polymorphic `any_type`
functions, and the exact latest `basic.at` load. Total catalog size is75.
These are prerequisite diagnostics, not substitute mathematical coverage.
Exit137 is signal/resource/timeout-escalation ambiguous until raw scheduler
evidence resolves it; it is not automatically labelled a timeout.

The first A2/G2 pilot exposed two fixture dependencies: unary matrix negation
and `ones` belong to basic.at, not the bare core. The core templates now use
the builtin `0-mat` overload and an explicitly constructed all-ones vector;
failed R2 inputs/results remain immutable. Its Hodge failures on the original
are retained separately and are not waived because elementary specializations
printed true before the branching failures.

R4 appends14cases without moving the existing75indices: eight core KLV cases
expand the original's split_form/trivial construction into builtin calls, and
six Hodge diagnostic cases preserve split A2/G2 bound4, add bound20 and include
the upstream complex A2/G2 examples at bounds15/50. Total catalog size is89.
These are candidate probes pending HPC acceptance, not new support claims.
Core KLV does not replace the failing latest-script case; Hodge back traces
diagnose the original failures without editing upstream scripts or lowering
the acceptance criteria. The original Hodge fixture remains unchanged.

R4 review3832122 confirms all six Hodge probes fail in the original too;
increasing the split bound to20 does not remove the observed failure. Its
two core KLV probes exposed another fixture-only alias dependency:
`infinitesimal_character` is script-defined. R5 replaces that alias by the
exact builtin `%Param` destructuring from basic.at; R4 inputs remain frozen.
R5 also appends index89, `D4_fpp_wall_regression` (90catalog cases total),
from the differing FPP Weyl representatives in3832088. It preserves full
actions and original representatives, and independently checks positivity of
the integral simple roots; equal orbit numerators alone must not mask a bad
Weyl element. Its baseline failure and source repair are not yet accepted.

R6 appends A2/G2 `partial_kl_regression` at indices90/91 (92total), after
R5 raw core probes returned complete original blocks of4/10 entries but only
one Rust entry; half-scaled inputs had2versus1. The focused tests assert those
original-backed sizes and retain every parameter, matrix entry and polynomial.
They are explicitly failing regression candidates, not supported features.

R7 appends index92, A1.A1_cycle_product_ranks (93total). It recaptures
15 SL(2,R) x SU(2) irreducibles at bounds4/6 against the latest unchanged
scripts, not historical goldens. The compact highest weights0/1/2 give
independent tensor-dimension predictions with multiplicities1/2/3.
Four full Phi identities per bound bind positive/negative orbit labels before
the prediction is applied. The HPC checker independently uses exact rational
elimination to verify rank-nullity, the entire rational kernel, dimension rows,
boundary projection and every integral reconstruction residual. Both the
driver and independent reviewer reject violated mathematical checks even if
the two interpreters print the same output. See
`docs/slices/math_cycle_product_2026-09-28.md` for scope and derivation.
This does not establish general cycles, richer representation-valued
coefficients, exceptional-group multiplicities or cutoff completeness.
R7 execution3832193 exposed an old-fixture interface mismatch: latest
vector/solve return Maybe<vec>, tested by succeeds rather than any.
R8 changes those two fixture checks only (plus the preventive harness test).
The R7 original failure and raw streams remain frozen; upstream scripts
and the numerical acceptance conditions are unchanged.

| Area | Output checked | Additional mathematical check |
|---|---|---|
| Basic root data | Full roots, coroots, Cartan matrix and dual | Double dual; exact rational identity |
| Enumerate KGB | Every element, every simple-root edge, statuses, lengths, involutions | Simple cross action squares to identity |
| KLV | Complete partial-block records, matrices and polynomial pools | Differential only initially; independent KL identities remain needed |
| Unitarity | Actual `is_unitary` results, not deformation proxies | Trivial representation must be unitary |
| Hodge | Complete standard/irreducible grading polynomials; K-type branching at explicit bound 4 | Upstream specialization identities at 1 and s |
| FPP | Complete numerator and Weyl-shift lists, zero/wall/interior inputs | Wrong-rank input must be rejected |
| Annihilator variety | Full orbit, H, diagram, dimension, datum | Trivial module has zero-dimensional orbit |
| Cycle prerequisites | Full Phi/T/Y/Q/P_X data and orbit labels at bounds 2 and 4 | Does **not** certify cycle multiplicities or cutoff completeness |

General associated cycles remain an explicit, mandatory open requirement.
Upstream `K_Nilpotent.at` currently exposes `av` as orbit **support**, whereas
the objective requires a cycle, including multiplicities. Do not count AV-ann,
`av`, bounded matrices, or the older SL2 examples as general-cycle completion.
The catalog retains this gap even if every currently executable case passes.
Additional real forms, isogenies, nontrivial Hodge examples, independent
identities and D6/D8/minute-scale workloads are also tracked there.

## Evidence and acceptance

1. Build the two locked source archives with `hpc/math_baseline_build.sbatch`.
   Build output alone is `BUILT_NOT_DIFFERENTIALLY_VERIFIED`, never PASS for
   mathematics. Compilers, commands, source files, scripts and binary hashes
   are recorded. CWEB output is generated only in the job's build directory.
2. Submit `hpc/math_suite.sbatch` as a bounded array after the build has passed.
   Each job first runs the Python verifier tests, then executes one case in
   both engines with identical input, scripts, node and single-thread settings.
3. A positive mathematical match requires both engines to exit successfully,
   empty error streams, exactly one begin/end pair and equality of **every
   byte inside the mathematical result section**. Full raw stdout/stderr and
   their equality are recorded separately. No line sampling, coefficient
   normalization, digest-only correctness comparison, or equal-error PASS.
4. Negative cases require the intended diagnostic and a nonzero exit from
   both engines. Timeout, OOM and missing metrics cannot pass. An original
   failure is a fixture/oracle investigation, not evidence of Rust correctness.
5. Each report retains GNU-time wall/peak-RSS data. The first run is a
   correctness survey, not a reliable speed benchmark. Only after correctness
   acceptance and runtime calibration will paired, rotated repeated runs
   establish speed ratios on workloads taking roughly 60–600 seconds.
6. Independently review outputs, hashes, scope and errors before accepting a
   report. Count coverage by operation AND group; do not hide missing cases
   in a single overall pass percentage.

Memory is a capacity/OOM guardrail, not the optimization objective. The initial
array uses cpu, 8 GiB per job, 6 GiB child address-space limit and a 300-second
limit per engine. A capacity failure remains visible as incomplete evidence;
it must not be replaced by a smaller calculation labelled as equivalent.
No E8 or fat jobs are enabled by this library.

## Repeated benchmarks and additional Hodge inputs

R9 appends D6/D8 full KGB cases at93/94 and eight nontrivial A2/G2 Hodge
cases at95-102 (103catalog cases total). Hodge includes half-scaled
nonintegral and zero-scaled singular spherical parameters at bounds4/20,
complete standard/irreducible branching, specialization checks and full
Hodge K-type matrices with upstream coefficient positivity. The original
trivial-module failures remain unchanged, not replaced by these additions.

`benchmarks.json` pins E7/D6/D8 catalog identities. The benchmark driver
runs four fresh-process pairs on the same node with alternating engine order,
retaining each complete output and exact RSS. A nonmatching, failed or timed-out
pair stops further repetitions and cannot produce a speed ratio. Even if both
engines agree within each pair, changed mathematics across rounds is rejected.
The review rehashes the full source archives/binaries and every raw round before
acceptance. Per-engine 60-600-second calibration is reported separately:
short workloads are not padded and are not claimed to satisfy that time window.
The CPU8GiB/child6GiB envelope remains in force. A capacity failure stays visible.

## Partial KL repair regressions

R10/R11 append indices103-106 (107cases total): A2/G2 partial/full/partial
lookup at half, zero and unit scale; B2 singleton/containing-interval lookup;
and compact A1 nonstandard rejection. Every parameter, matrix entry and
polynomial is printed and compared. R10 preserved a fixture dependency error:
bare Atlas has no equality overload for `[Param]` or the complete KL tuple.
R11 uses elementwise builtin comparisons instead of loading/rewriting basic.at.
Independent review3832340 confirms all four original cases behave as expected
and all four baseline Rust cases fail. The nonstandard case succeeds wrongly
in Rust; equal exit codes are therefore not assumed.

The explicit `diagnostic_rank_suffix:false` option keeps the nonstandard
diagnostic unchanged; existing FPP rank diagnostics still include both ranks.
`MATH_CASE_TIMEOUT` optionally sets the batch child timeout (default300,
driver-enforced maximum600seconds). The observed limit is recorded for each
engine. Longer follow-ups get a fresh stage and never overwrite shorter
timeout failures or turn those failures into speed ratios.
