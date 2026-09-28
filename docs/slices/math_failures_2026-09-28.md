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
