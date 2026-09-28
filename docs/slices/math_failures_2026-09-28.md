# Mathematical failures discovered by the latest-original survey

Baseline: Rust main05625c5d, original7e1b958c, build3831844. All executions
below are HPC compute jobs. Do not substitute old optimized Rust candidates,
normalize away the differences or modify the frozen original scripts.

## D4 FPP Weyl representatives

R3 case32/job3832088 finishes successfully in both engines with empty stderr.
At gamma=[1,0,0,0]/100, all eight FPP orbit numerators agree, but four of
the six returned Weyl representatives differ. This is not established as
mere alternate spelling: the additional R5 case89/job3832137 prints full
root and basis actions, and Rust triggers `D4 FPP integral simple image is
negative`; the original passes. For example, Rust row1 maps root13 to4
(negative, with12positive roots). The integral simple roots here are1,2,3;
upstream alcoves.cpp:998-1015 explicitly normalizes their images positive.

Regression: `tests/math/templates/fpp_d4_wall.atlas`. Keep the original six
representatives as well as every shift, root action, basis action and the
positivity check. R5 independent review3832147 has now rehashed and verified
the complete regression output and failure. R3 broad review3832103 remains
pending at this checkpoint. No fix has been made.

Static repair lead: domain_builtins.rs `reflection_word` copies the descent
prefix to the end in forward order. Original rootdata.cpp:596-612 appends it
in reverse to form the conjugate reflection. A forward copy is not generally
a reflection once there are multiple noncommuting descent steps. This is a
source-backed defect hypothesis for this failure, not proof that changing
that line fixes all FPP behavior. Test the root-reflection identity directly
across classical/exceptional systems before accepting a repair.

Candidate evidence: build3832301 now provides exact before-fail/after-pass
identity tests over A2/B2/C2/D4/D6/D8/G2/F4/E6/E7 and both root numberings.
Review3832333 confirms the D4 regression, all smaller FPP cases, and E6.
E7 remains timed out at300s, so not all FPP gates pass. Original E7 finishes
in24.43s; Rust's killed leg uses2.31GiB. A separate600s run preserves this
failure. The600s follow-up is now terminal: review3832506 independently
verifies original24.1906s/981172KiB versus Rust timeout600.2182s/2588124KiB.
Increasing the observation window did not close the gate; do not resubmit
the same input/build with yet another timeout or call it a completed ratio.
Potential performance lead only: original rootdata.h:296/320 uses
precomputed simple-root permutations, whereas simple_reflect_root_nbr rebuilds
ambient vectors and looks up roots on each letter. The FPP Weyl-value export
also applies every word through matrix-based WeylElement multiplication.
Profile/isolate the stages before assigning the remaining timeout to either.

## Partial KL blocks collapse to a singleton

After replacing the script-only `infinitesimal_character` alias with builtin
`%Param` destructuring, R5 core KLV cases can reach the actual mathematics.
Both engines exit0 with empty stderr, but the complete outputs disagree:

| Case / raw job | Original trivial block | Rust | Original half-scaled block | Rust |
|---|---:|---:|---:|---:|
| A2 /3832138 |4|1|2|1|
| G2 /3832142 |10|1|2|1|

These are missing parameters and KL-matrix entries, not print-format noise.
G2 also loses the nonconstant [1,1] polynomial in the integral example.
R5 review3832147 is independently verified. It confirms full-output
mismatches in A2/B2/C2/D4/G2/F4, not only the two examples above.
The complete core cases remain in the library;
new focused regressions `partial_kl_regression.atlas` assert original-backed
sizes and retain every parameter, matrix coefficient and polynomial, rather
than replacing the full comparison by a count. Independent R6 review3832167
confirms that both focused cases pass in the original and fail in unchanged
Rust with `partial KL block lost Bruhat predecessors`.

Static implementation gap: the current `partial_KL_block` arm in
domain_builtins.rs:14855 builds a dual-quasisplit full block, locates the first
row with the same x, and walks only selected descents. Latest original
atlas-types.w:7254 uses `Rep_table::lookup` for the actual parameter followed
by its full Bruhat downward closure and singular condensation. The difference
must be repaired at the correct parameter/locator boundary, not by hardcoding
the observed sizes. No runtime change has been made.

Further source audit for the next sequential repair:

1. The existing `partial_block` arm already uses `context.rep.lookup` on the
   actual parameter, `block_bruhat_hasse` for the full downward closure,
   `located_singular_flags`, and `located_row_parameter`. Reuse this boundary;
   `x` alone does not identify a standard representation.
2. A cached located block may contain more rows after another partial/full
   query. The selected downset must be computed from `located.raw_row()` on
   every call, not assumed to be the entire stored block. Add a history-order
   fixture (partial, full, partial) retaining full parameters and coefficients.
3. Existing `KL_block` has parameter-adapted singular condensation through
   `partial_block_finals_for` and `located.with_kl_table`. Its full lookup
   and all-survivor set must become partial lookup plus the selected downset
   for the partial operation; do not use descent reachability as a substitute
   for the Hasse closure.
4. Latest original atlas-types.w:7219-7246 reserves polynomial indices0/1
   for zero/one, initializes the index matrix as identity, and interns only
   strict-upper entries in row order. The faulty partial arm also lacks
   this convention. Counts alone cannot detect a wrong polynomial pool.
5. Preserve `test_standard` rejection before lookup and locator transport of
   each row at the caller's infinitesimal character. Test nonintegral and
   singular parameters as well as the existing A2/G2 predecessor regressions.

R10/R11 cover those histories plus compact A1 negative imaginary weight.
R10 discovered that bare Atlas lacks list/tuple equality for these types;
R11 uses elementwise Param/vec/int equality without importing basic.at.
Independent review3832340 confirms all four original cases pass while Rust
loses A2/G2 predecessors, errors on B2's singleton and accepts the nonstandard
A1 input. The candidate repair is frozen in build3832345. Independent review
3832517 now confirms A2/B2/C2/D4/G2 full KL outputs and all four new regressions
pass. F4 reaches a deeper panic at kl_table.rs:265 (`cross of extremal`), also
seen in the E7 Rust leg. Keep case80/job3832508 as its complete original-backed
reproducer. The Rust recursion fetches `cross(x,s).expect(...)` before its
descent switch, whereas original kl.cpp:400-445 fetches cross only in the
complex-descent and real-II branches. Identify the actual missing-link case,
add a focused regression, and preserve required-link invariants when repairing.
This is not permission to return zero for every missing link or waive F4.

Focused probe3832534 now identifies the exact missing link: at unit scale,
`x=263, y=278, s=2`, descent `RealTypeII`. Its executed core regression
fails with the same panic, not a compilation/fixture failure. Report SHA
14d877bb00d5c4b4e4d25c7d9dacf7c58105c63ade97b905ea357c0390b7f616;
unit log SHA2636e6f0fb1b15f50ff347048597eda4dc2114edb6121c0ba9097568a3800c70.
Original partial-block construction explicitly allows this cross to leave
the interval; `KL_pol(UndefBlock, sy)` is zero. The candidate now moves cross
lookup into the complex/real-II branches and supplies zero only for an
absent real-II cross. Complex and Cayley descent invariants remain required.
The before/after F4 unit and full-output differential are still required;
case107 additionally checks F4 cold/full/warm histories at three scales.
Isolated build3832615 is now terminal: the same before unit reproduces the
cross panic, and the after unit gets past the integer-scale calculation but
fails on half-scale lookup with integral-image positivity. Build SHA
1f9d64d78c695a1a292f9d1e1c31b1fe5103569aa351ccfcad2aaa352336409d.
Keep this FAIL result; there is no release binary or differential acceptance.
The never-runnable dependent array3832623 was cancelled without execution.

R12b review3832614 independently verifies case107: original full history
passes, but the pre-boundary-repair Rust binary fails on the initial half-scale
call with `Weyl-element integral image positivity invariant was violated`.
This is a distinct locator failure before any full-lookup cache enlargement,
not the real-II panic. Report SHA
89813b870cae2d89336598b6d0eb7df22cb9eca48b02a86fe8b304ddee16ffbf.
Static audit finds locator.rs computes root additive closure; upstream
rootdata.h:131 defaults `additive_closure` to `for_coroots=true`, and
InnerClass::int_item uses that default. Non-simply-laced systems distinguish
these closures. The precise relationship to this failure and the correction
still require HPC regression/differential proof. A minimal B2 coroot-sum
identity and the F4 exact-integral-root-set test are specified in HANDOFF.
Probe3832726 executes both new locator unit regressions on unchanged runtime:
B2 incorrectly returns4 instead of8 closure elements; F4 hits the positivity
invariant. Report SHA88aaa653d427f1705f36ef992a280a5db52acc3a662d2b4c329196cd2e455e43.
This is before-failure evidence, not an after-fix or full-differential pass.

E6's core KLV probe fails earlier in Rust with `real-form order twist-fixed
generator coordinate invariant was violated`, while the original succeeds.
E7's original reports `std::bad_alloc` under the6GiB child address-space cap;
this is a resource-limited comparison, not evidence that Rust is correct or
that the original algorithm is mathematically wrong. Do not report its
incomplete6.9-second original leg versus161.6-second Rust leg as a speed ratio.

## Hodge original failures are not simply a small-bound issue

R4 array3832118 and independent review3832122 preserve all six Hodge probes:
split A2/G2 at bounds4 and20, complex A2 at15 and complex G2 at50. Every
original run still fails; Rust independently fails loading the latest generic
basic.at. Original source and full back traces identify these paths:

- A2 split: `tensor_product.at:84` calls `branch_std(P,height(P))` with an
  empty KTypePol P. `basic.at:2655-2656` defines its height as-1; the builtin
  atlas-types.w:6275 rejects a negative level. This internally chosen level
  is not the positive outer Hodge bound. The trace reaches this path at
  bound20 as well. Complex A2/G2 also report negative levels; their complete
  traces must be checked before assigning every occurrence the same cause.
- G2 split: the trace at `hodge_K_type_formula.at:156` passes K-types of a
  Levi sl(2,R).u(1) to `Hodge_K_type_pol(temp,G_orig)` with ambient G2 as the
  owner. The term-list builtin rejects mismatching real forms at
  atlas-types.w:5931/5944. This is inside the original's local Phi_S helper,
  not evidence of a Rust numerical disagreement.

Do not fix the reference silently or waive these tests because four elementary
specializations print true. Preserve them as original-side failures while
separately extending nontrivial filtration coverage and establishing the
correct mathematical expectations. Any future patched-original experiment
must have its own source pin and cannot be labelled unmodified7e1b958c.

## Broader grid: capacity, time and construction gaps

R3 review3832103 independently verifies54 B2/C2/D4/F4/E6/E7 cases. All root
data cases match. E6 KGB3832113 fails in the same Rust real-form constructor
as core KLV; the full KGB case remains its reproducer. E7 KGB3832151 has
equal complete mathematical output, but takes69.8035s/1459140KiB in Rust
versus1.3032s/41672KiB in the original. This is single-shot diagnostic evidence,
not a repeated speed benchmark.

E6 FPP3832129 and E7 FPP3832162 time out at300s in Rust; originals complete
in0.2799s/24.4600s. Rust stdout is empty after the kills, so it does not identify
which of the three gamma values or two FPP functions consumed the time.
Do not assign the timeout to a particular helper without targeted evidence.
The already identified reflection_word defect also affects the normalization
path in to_positive_system; this is a source lead, not a proved explanation
for these timeouts.

The original times out in D4/F4/E6/E7 unitarity, D4/F4/E6 Hodge and E6/E7
cycle foundation (300s/engine). E7 KLV/Hodge hit std::bad_alloc under the6GiB
cap. Rust independently fails latest-script loading in all those cases.
Neither side's incomplete execution supplies a valid speedup.

## Product fixture migration, not a mathematical error

R7 product job3832193 and review3832194 preserve an old-fixture API failure:
any(v) cannot accept latest Maybe<vec>. Source basic.at:31 defines succeeds;
K_Nilpotent.at:13/18 and basic.at:923 return Maybe<vec> from vector/solve.
R8 changes only the two fixture success predicates to succeeds, retaining
every assertion and unchanged latest scripts. No old output is reused as
the expected mathematics. R7's checker tests passed31cases on HPC, but
the failed Atlas execution never reached numerical acceptance.

## Larger-rank and nontrivial follow-up

R9 benchmark review3832228 rehashes four E7 and four D6 fresh-process pairs;
full mathematical output is identical within and across rounds. E7's median
is1.35766s original versus69.4160s Rust (about51x slower), with1.39-1.40GiB
Rust peak RSS versus40.7MiB original. D6 remains sub-second and is not a
minute-scale benchmark. D8 original completes in3.757s; Rust refuses to
construct the group at its4000000 enumeration cap. This is a capacity/algorithm
gap, not an accepted computation or a speedup.

Source lead for the larger-rank problem: domain_builtins.rs defines
WEYL_BUDGET=4000000 and passes it into CartanClassification. In
inner_class.rs:639-662, enumerated_twisted_involutions calls compact.enumerate
for the entire Weyl group before filtering twisted involutions. Do not merely
raise the cap: inspect the original's direct construction and prove its
adaptation on the existing complete KGB tests before claiming improvement.

R9 Hodge review3832222 adds half-scaled nonintegral and zero-scaled singular
parameters for A2/G2, bounds4/20. All8 original executions still fail; A2
reports negative internal branch level and G2 reports real-form mismatch.
They retain full back traces, grading/character and attempted matrix output.
Rust independently fails latest basic.at loading. Fixing only the original
trivial parameter or lowering bounds would not address this broader evidence.
