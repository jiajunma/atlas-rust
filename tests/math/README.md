# Mathematical test and benchmark library

This library is being established against upstream Atlas **7e1b958c** (remote
master checked 2026-09-28) and Rust remote main **05625c5d** (pull was already
up to date). The historical optimized/PGO worktrees are separate candidates,
not silently substituted for main. `baseline.json` locks both complete source
archives. Neither the original installed oracle nor any historical job folder
is updated in place.

All tests, builds, differential runs and benchmarks run on **HPC compute
nodes**. Local work is limited to source inspection, editing and synchronization.

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
