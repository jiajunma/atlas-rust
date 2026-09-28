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
