# Scoped numerical associated-cycle test

Verified original-only numerical evidence: R8 execution3832200 and independent
review3832201. Original passes all30 reconstructions and8 Phi sign identities;
the Q matrices have shape14x41 /22x97, exact ranks14/22 and full kernel
dimensions27/75 at N4/N6. Rust fails loading latest basic.at before computation.
HPC preflight3832199 and review each pass31checker tests.
The review is stored in math_suite_cycle_review_2026_09_28.json, SHA
30924499bb053599239cfc9095dfa666f0e3d3bd4d20b242be8ba84dad414656.
No mathematical Rust acceptance or performance ratio follows from this failure.

The R7 product fixture is a latest-original recapture, not acceptance based
on the historical 2026-09-18 build. It uses SL(2,R) x SU(2), five selected
SL2 infinite-dimensional irreducibles, and compact highest weights m=0,1,2.
Each is reconstructed at N=4 and N=6: thirty numerical two-orbit rank pairs.
The complete matrices, inducing K-types, Phi polynomials and module
character classes are retained, not only the final rank pairs.

## Expected mathematics, independent of the solve

The five SL2 families have positive/negative maximal-orbit numerical ranks
(1,0), (1,0), (0,1), (0,1), (1,1). The fixture identifies these via final
K-types and their complete irreducible character classes. With I0,...,I4
denoting those classes with trivial compact factor, the full Phi identities
I4-I3 and I0 bind the positive orbit; I4-I1 and I2 bind the negative orbit.
Those identities, the inducing dimension1, and the actual orbit-tagged columns
are checked anew. A changed orbit ordering requires explicit remapping;
it is not inferred by fitting the final ranks.

For the compact irreducible V_m, dim(V_m)=m+1. Give M external-tensor V_m
the filtration F_i M tensor V_m. The compact factor preserves each step,
so its associated graded is gr(M) tensor V_m over the first factor's
noncompact symmetric algebra. Forgetting equivariance gives m+1 copies
of gr(M); generic lengths on maximal components are multiplied by m+1.
Thus the independent prediction is

    (m+1,0), (m+1,0), (0,m+1), (0,m+1), (m+1,m+1).

The cases m=1,2 concern irreducible external tensor products, not merely
direct sums of Harish-Chandra modules. These dimension-valued predictions
do not recover representation-valued stabilizer coefficients.

## Independent checks and limits

The Python checker redoes integer products and rational elimination from the
complete printed matrices. With quotient matrix Q, reported kernel K and
dimension row r, it requires QK=0, rank(K)=columns(K)=columns(Q)-rank(Q),
and rK=0. Hence r is independent of which integral solution reconstructs
the class inside this finite system. Every solution is separately checked
integral with zero residual; PX annihilates the captured boundary.
It does not assert saturation of the integer kernel lattice.

The Atlas fixture also checks Q=PX*T, every orbit-tagged column, the complete
character formula, finality and infinite dimension. The Python checker
cross-checks complete Phi sign identities in the common ambient basis.
Mutation tests include a coordinated change of dimensions, rows and rank
outputs that remains algebraically consistent but violates the tensor rule.

Neither agreement at two bounds nor exact finite linear algebra proves that
the heuristic Phi_upto cutoff contains every geometric column. General
associated cycles, exceptional-group multiplicities and richer coefficients
remain open. Latest unmodified script-loading failures in Rust remain failures;
old scripts or an optimized Rust worktree must not be substituted.
