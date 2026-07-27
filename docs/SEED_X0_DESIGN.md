# KGB seed design (stage d)

## Approved scope

Stage (d) of the KGB map: the seed `x0` — the square-class cocharacter
(`some_coch`), the elected base grading (the `TitsCoset` offset), the
binary grading-shift solve, and the central-fiber minimization — handing
stage (e) the pair (coset, reduced seed element) for one weak real form.
Consumes the strong-real layer and stages (a)-(c). DEFERRED with named
consumers: `backtrack_seed` (partial KGB only, already on the stage
map's deferred list) and `minimal_torus_part` (its sole caller is the
synthetic-real-form interpreter path, atlas-types.w:3866).

## Atlas construction (oracle trace)

Citations are to `~/mycodes/atlasofliegroups`, master `4d3e9449`.

- `some_coch(G, csc)` (innerclass.cpp:966-977): elect the square
  class's representative real form — the LOWEST `RealFormNbr` in the
  class (innerclass.cpp:886-888; `Partition::classRep` is the smallest
  index) — take its compact-simple set, sum the RATIONAL fundamental
  coweights over it, and return
  `stable_log(exp_2pi(sum), xi^T)` (y_values.cpp:155-167): the elected
  `xi^T`-stable logarithm of the SQUARE, computed through an adapted
  basis of `xi + 1` with coordinates reduced mod 1. Deterministic given
  the class-representative election and the basis routine. CAVEAT
  (upstream's own stale comment, innerclass.cpp:959-964):
  `exp_pi(some_coch)` is a square root of the square, not necessarily
  the representative's own point, so its compacts may differ —
  `x0_torus_part` compensates.
- `grading_of_simples(G, coch)` (innerclass.cpp:1295-1303): bit `s` set
  iff `<coch, alpha_s>` is EVEN — noncompact for imaginary simples,
  with complex simples marked by the same parity so the offset is
  twist-invariant (the stability of `coch` under `xi^T` mod 1 makes the
  paired parities agree). `g_rho_check()` IS the stored square-class
  cocharacter (realredgp.h:61, 91-92), and
  `base_grading() = grading_of_simples(G, g_rho_check())`
  (realredgp.cpp:147-150) seeds the `TitsCoset` (kgb.cpp:525-526).
- `x0_torus_part(G, rf)` (innerclass.cpp:1070-1095):
  1. `base = compacts_for(exp_pi(some_coch(xi_square(rf))))` — bit set
     iff the pairing is ODD (innerclass.cpp:950-957);
  2. `rf_cpt = simple_roots_x0_compact(rf)` — the weak-orbit
     representative's adjoint-fiber bits moved onto the
     imaginary-simple positions (innerclass.cpp:891-909);
  3. `bits = grading_shift_repr(base XOR rf_cpt)`
     (innerclass.cpp:986-1007): slice the difference to imaginary
     simples, build the mod-2 map (fundamental fiber-group basis vs
     imaginary simple roots), solve by a one-sided section with an
     exactness assert, lift fiber coordinates to rank-bit torus parts;
  4. CENTRAL-FIBER MINIMIZATION (innerclass.cpp:1080-1089):
     `central_fiber(rf)` (1041-1055) solves
     `toAdjoint(y) = wrf_rep(rf) - class_base(csc)` by the section
     (EXACTLY the port's augmented-elimination strong-representative
     solve), enumerates the strong-real W_im-orbit members with the
     SAME toAdjoint image (`preimage`, 1020-1036), and returns their
     differences as torus parts; the elected seed minimizes
     `bits + c` over that set — the SHIFTED value, not `c`
     (innerclass.cpp:1086, the comment is explicit) — under
     `SmallBitVector::operator<` = the bit pattern as an unsigned with
     bit 0 least significant (bitvector.h:184-188);
  5. assert the compacts of `t + bits` equal `rf_cpt`
     (innerclass.cpp:1090-1092).
- KGB consumption (kgb.cpp:489-560): ONE seed element —
  `TitsElt(Tg, x0_torus_part())` at the identity Weyl part — reduced
  ("should be unnecessary") and hashed; the rest of the fundamental
  fiber is DISCOVERED by the closure loop, not enumerated up front.
  `torus_factor(x) = symmetrise(g_rho_check() - lift(bits), theta)`
  (kgb.cpp:699-712) is the reason the cocharacter must be exact
  rational data, not just parities.
- Real-form indexing (the port-critical convention): upstream
  `RealFormNbr` = orbit number in the fundamental fiber's weak-real
  partition, classes ordered by their MINIMAL adjoint-fiber
  representative (partition_def.h:69-96), `classRep` = that minimum,
  quasisplit = 0 (the all-ones base grading orbit,
  cartanclass.cpp:304-327, innerclass.h:342-343), and square classes
  are intrinsic coset coordinates (cartanclass.cpp:541-579) with csc 0
  containing rf 0.

## Port decisions and open questions for review

1. THE NUMBERING NORMALIZATION QUESTION, front and center: every
   election in this stage flows through upstream's weak-real-form
   numbering (lowest-id class representative; orbits ordered by
   minimal adjoint-fiber representative; quasisplit = 0). If the
   port's `WeakRealFormId` order differs, then `some_coch`, the base
   grading, `x0`, and every torus-bit observable diverge from upstream
   oracles — and the stage-(c) convention-coincidence ruling for
   `torus_factor` collapses. The reviews MUST audit the port's actual
   orderings (`weak_real_form.rs` mask-orbit walk order,
   `strong_real.rs` square-class and representative conventions,
   `RealFormLabels`' quasisplit anchor) against upstream's. Outcome A
   (orders coincide): record the proof and proceed. Outcome B (they
   differ): stage (d) adds an explicit REINDEXING to upstream order —
   a deterministic sort by minimal adjoint-fiber representative
   coordinates — at the seed boundary, so that everything downstream
   of stage (d) speaks upstream numbering. The design REFUSES the
   third option (crate-order elections with adapter-deferred torus
   bits), because it silently downgrades torus_factor from directly
   comparable to class-comparable.
2. `some_coch` needs exact rational machinery: fundamental coweights
   (inverse-Cartan rows — the crate already does rational C^T solves
   with malachite in `real_form_labels.rs`) and a port of
   `stable_log`'s adapted-basis election over `xi + 1`. Whether the
   existing `integer_lattice.rs` reductions suffice for the adapted
   basis (a Smith-like normal form with transformation matrices) is a
   review question; if not, the addition lands in `integer_lattice.rs`
   with its own budget threading. The output is an exact rational
   coweight (internal `Vec<Rational>` representation), consumed as
   parities here and as exact data by `torus_factor` later.
3. The grading-shift solve reuses the strong-real layer's
   augmented-elimination idiom over the fundamental fiber group's
   basis (the port's `ModTwoSubquotient` coordinates/lift pair);
   exactness stays an invariant check, mirroring upstream's assert.
4. The central fiber IS strong-real data the port already computes:
   the toAdjoint solve is the strong-representative solve, and the
   orbit members with equal adjoint image come from the
   per-square-class fiber partition. The minimization compares
   `bits + c` (the shifted value) under the numeric bit order;
   `ModTwoVector`'s derived `Ord` (word 0 first, bit 0 least
   significant within a word) agrees with upstream's single-word
   numeric order for lattice rank <= 64 — recorded, with the
   multi-word caveat noted as out of scope (upstream RANK_MAX is 32).
5. Stage output surface: a seed bundle per weak real form —
   `RealFormSeed { grading_offset, cocharacter, element }` — where
   `element` is the REDUCED `TitsElement` at the fundamental
   involution and `grading_offset` feeds `TitsCoset::new`. The
   `x0`-compacts assert (innerclass.cpp:1090-1092) is ported as the
   stage's invariant check, and the stage-(c) `is_valid` note
   (element squares to the identity coset) is the second seed
   verification.

## Data layout and public boundary (sketch for review)

```text
// seed.rs
RealFormSeed {
    grading_offset: Vec<bool>,     // grading_of_simples parity
    cocharacter: <rational coweight>,   // exact some_coch output
    element: TitsElement,          // reduced, at the fundamental involution
}
build_seed(&InnerClass, &InvolutionTable, &StrongRealClassification,
           WeakRealFormId, <budget>) -> Result<RealFormSeed, ...>
```

Constructor shape, provenance gates, the budget question (the rational
reductions are bounded; is a knob warranted?), and whether the
cocharacter's representation is exposed or kept internal are all
review questions — this stage touches three owner layers at once and
the API review must place it against the crate's ownership idioms.

## Tests and fixture gate (sketch for review)

- SL(2,R) and PGL(2,R): quasisplit seeds; base grading all-noncompact
  at the quasisplit form; the x0-compacts invariant holds; the seed
  element is the zero-bit element after reduction where the theory
  says so.
- Sp(4,R) and SU(2,1): seeds for every weak real form of the inner
  class; gradings at the seed match `simple_roots_x0_compact`'s
  complement; determinism (two builds agree).
- The compact form: all-compact seed grading.
- Central-fiber minimization: a case where the fiber has more than one
  grading-identical member, electing the minimal shifted value.
- Numbering normalization: whichever of outcome A/B the reviews
  establish, a test pinning the port's weak-real-form order against
  the upstream convention on B2 (published real-form order for
  Sp(4,R): sp(4,R) split, sp(1,1), sp(2) compact — exact ids per the
  audit).

`tests/fixtures/domain/seed_x0.atlas` is reserved.

## Three independent design checks

Before implementation, three fresh reviews: (1) Atlas semantics — the
some_coch/stable_log election, the grading-shift system, the
central-fiber minimization (shifted-value subtlety), and the
numbering-normalization adjudication against partition_def.h; (2) Rust
internals — integer_lattice capabilities for the adapted basis,
malachite rational plumbing, ModTwoVector order semantics, and an
AUDIT of the port's weak-real-form/square-class orderings against
upstream's; (3) API and consumer fit — the seed bundle surface, what
stage (e) consumes, provenance across three owner layers, and the
budget question. Findings and corrections will be recorded here before
source edits begin.
