//! The `block_modifier` of a located common block and the `Rep_context`
//! Weyl-attitude arithmetic that operates through it.
//!
//! This is step 2 of the "nonidentity generator attitude" slice: a pure,
//! unwired port of upstream `repr::block_modifier` (gkmod/repr.h:493-499,
//! repr.cpp:1401-1419) together with the `Rep_context` methods
//! `transform` (repr.cpp:712-754), `shift` (repr.cpp:352-356),
//! `make_diff_integral_orthogonal` (repr.cpp:317-329),
//! `make_relative_to` (repr.cpp:338-350), and the modifier-carrying `sr`
//! (repr.cpp:815-823).  Nothing here is called by `RepTable` or any other
//! existing consumer yet; step 3 wires it into block lookup behind
//! attitude gates.
//!
//! Correspondences with existing crate pieces (all reused, none forked):
//!
//! - upstream `StandardReprMod` is [`StandardReprMod`]
//!   (partial_block.rs: the KGB element plus the `real_unique`-normalised
//!   `gamma_lambda`);
//! - `Rep_context::sr_gamma` is [`RepContext::sr_gamma`], and the
//!   no-modifier `sr(srm, gamma)` (repr.cpp:808-813) is
//!   [`StandardReprMod::to_standard`], which embeds
//!   `gamma_lambda_rho(srm) = srm.gamma_lambda() + rho`
//!   (repr.h:329-330);
//! - `InvolutionTable::real_unique` is [`RepContext::real_unique`];
//! - `InnerClass::integrality_codec` (innerclass.cpp:1184-1194) and
//!   `Rep_context::theta_1_preimage` (repr.cpp:297-313) are
//!   `RepTable::integral_codec` and `IntegralCodec::theta_1_preimage` in
//!   rep_table.rs.
//!
//! One deliberate deviation, none in semantics for the intended domain:
//! upstream `transform` walks `Weyl_group().word(w)`, the word stored with
//! the group element by the Weyl-group transducer, while the crate walks
//! [`WeylElement::reduced_word`], its canonical lowest-left-descent
//! reduced word.  Both are reduced words for the same element; the
//! per-letter operation is a (partial) group action, and using the same
//! canonical word in both directions keeps `transform<false>` the
//! letter-by-letter inverse of `transform<true>`, which is all
//! `make_relative_to` and `sr` rely on.

use crate::rep_table::RepTable;
use crate::{
    BasedRootDatum, BlockLocator, IntegralSubsystem, KgbStatus, RationalWeight, RepContext, RootId,
    RootSystem, StandardRepr, StandardReprMod, StructureError, WeylElement,
};

/// Upstream `repr::block_modifier` (repr.h:493-499): a [`BlockLocator`]
/// (`struct block_modifier : public locator`) plus the `RatWeight shift`
/// that is added to `gamlam` before the Weyl transport when a stored
/// block's rows are read at the query's attitude.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlockModifier {
    locator: BlockLocator,
    shift: RationalWeight,
}

impl BlockModifier {
    /// `block_modifier(const common_block&)` (repr.cpp:1403-1409): the
    /// trivial modifier of a block — identity `w` and `simple_pi` over the
    /// block's simply-integral roots, zero shift.
    ///
    /// Upstream reads `b.simply_integrals` and both ranks off the stored
    /// block; the crate's `PartialBlock` does not retain its construction
    /// context, so the caller passes the simply-integral simple roots (the
    /// `simp_int` list) and the root system.  `int_sys` gets the upstream
    /// `-1` sentinel, `u32::MAX` here: like upstream's, the trivial
    /// modifier is for local use and its datum id is never read.
    pub fn trivial(system: &RootSystem, simp_int: Vec<RootId>) -> Result<Self, StructureError> {
        let block_rank = simp_int.len();
        Ok(Self {
            locator: BlockLocator::from_parts(
                u32::MAX,
                WeylElement::identity(system)?,
                simp_int,
                (0..block_rank).collect(),
            ),
            shift: RationalWeight::zero(system.lattice_rank())?,
        })
    }

    /// A modifier whose locator part is filled from an existing locator,
    /// as `Reduced_param::reduce` writes the query's locator through the
    /// `locator&` base subobject (repr.cpp:110-125) and
    /// `append_block_containing` copies it (repr.cpp:1684-1685).  The
    /// shift is supplied separately; `make_relative_to` overwrites it.
    pub fn from_locator(locator: BlockLocator, shift: RationalWeight) -> Self {
        Self { locator, shift }
    }

    /// `block_modifier::clear(block_rank, datum_rank)`
    /// (repr.cpp:1412-1419): drop all modifications — identity `w`,
    /// identity `simple_pi` of size `block_rank`, zero shift of the datum
    /// rank.  `int_sys` and `simp_int` are not relative and remain.
    /// Upstream takes both ranks; here the datum rank is read from
    /// `system`.
    pub fn clear(&mut self, system: &RootSystem, block_rank: usize) -> Result<(), StructureError> {
        self.locator = BlockLocator::from_parts(
            self.locator.int_sys(),
            WeylElement::identity(system)?,
            self.locator.simp_int().to_vec(),
            (0..block_rank).collect(),
        );
        self.shift = RationalWeight::zero(system.lattice_rank())?;
        Ok(())
    }

    /// The locator part (attitude data) of the modifier.
    pub fn locator(&self) -> &BlockLocator {
        &self.locator
    }

    /// The `RatWeight shift` field (repr.h:494).
    pub fn shift(&self) -> &RationalWeight {
        &self.shift
    }

    /// `locator::int_sys_nr`.
    pub fn int_sys(&self) -> u32 {
        self.locator.int_sys()
    }

    /// `locator::w`, applied by `transform<false>` when reading stored
    /// rows at the query attitude.
    pub fn w(&self) -> &WeylElement {
        self.locator.w()
    }

    /// `locator::simp_int`.
    pub fn simp_int(&self) -> &[RootId] {
        self.locator.simp_int()
    }

    /// `locator::simple_pi`.
    pub fn simple_pi(&self) -> &[usize] {
        self.locator.simple_pi()
    }
}

impl RepContext<'_> {
    /// `Rep_context::transform<left_to_right>` (repr.cpp:712-754): the
    /// Weyl action on a standard parameter modulo `X^*`, one simple
    /// generator at a time:
    ///
    /// - complex: cross `x` and reflect `gamlam`'s numerator
    ///   (`rd.simple_reflect(s,gln)`);
    /// - real: `x` is fixed (the cross action of a real root is trivial)
    ///   and the numerator gets the affine reflection centred at
    ///   `-\rho_R` (`rd.simple_reflect(s,gln,den)`);
    /// - imaginary: upstream throws `Bad Weyl group element SRM
    ///   transform`; the checked analogue is returned here.
    ///
    /// `LEFT_TO_RIGHT` selects the iteration direction over the word of
    /// `w`: `true` applies the first letter first, `false` the last —
    /// making `transform<false>` the letter-by-letter inverse of
    /// `transform<true>` for the same `w`.  The value is `real_unique`
    /// re-normalised at the final `x` (repr.cpp:753).
    pub fn transform_srm<const LEFT_TO_RIGHT: bool>(
        &self,
        w: &WeylElement,
        srm: &mut StandardReprMod,
    ) -> Result<(), StructureError> {
        let datum = self.datum();
        let word = w.reduced_word(self.root_system())?;
        let mut x = srm.x();
        let mut numerator = srm.gamma_lambda().numerator().to_vec();
        let denominator = srm.gamma_lambda().denominator();
        let letters: Vec<usize> = if LEFT_TO_RIGHT {
            word
        } else {
            word.into_iter().rev().collect()
        };
        for s in letters {
            match self.kgb_status(x, s)? {
                KgbStatus::Complex => {
                    x = self.cross_at(x, s)?;
                    simple_reflect_numerator(datum, s, &mut numerator, 0)?;
                }
                KgbStatus::Real => {
                    simple_reflect_numerator(datum, s, &mut numerator, denominator)?;
                }
                KgbStatus::ImaginaryCompact | KgbStatus::ImaginaryNoncompact => {
                    return Err(StructureError::RepInvariantViolation {
                        invariant: "Weyl group element SRM transform on an imaginary root",
                    });
                }
            }
        }
        let mut gamma_lambda = RationalWeight::new(numerator, denominator)?;
        let involution = self.involution_of(x)?;
        self.real_unique(involution, &mut gamma_lambda)?;
        srm.set_x(x);
        srm.set_gamma_lambda(gamma_lambda);
        Ok(())
    }

    /// `Rep_context::shift` (repr.cpp:352-356): add `amount` to `gamlam`
    /// and re-normalise with `real_unique` at the unchanged involution of
    /// `srm.x()`.
    pub fn shift_srm(
        &self,
        amount: &RationalWeight,
        srm: &mut StandardReprMod,
    ) -> Result<(), StructureError> {
        let mut gamma_lambda = srm.gamma_lambda().add(amount)?;
        let involution = self.involution_of(srm.x())?;
        self.real_unique(involution, &mut gamma_lambda)?;
        srm.set_gamma_lambda(gamma_lambda);
        Ok(())
    }

    /// `Rep_context::make_diff_integral_orthogonal` (repr.cpp:317-329):
    /// the difference `gamlam - srm.gamma_lambda()` of two representatives
    /// with identical integral-coroot evaluations, minus its fixed
    /// preimage in `(1-theta)X^*`, so the result is orthogonal to the
    /// integral root system of `srm.gamma_lambda()`.
    pub fn make_diff_integral_orthogonal(
        &self,
        gamlam: &RationalWeight,
        srm: &StandardReprMod,
    ) -> Result<RationalWeight, StructureError> {
        let mut result = gamlam.sub(srm.gamma_lambda())?;
        if !result.is_zero() {
            // `InnerClass::integrality_codec(srm.gamma_lambda(), inv)`
            // (innerclass.cpp:1184-1194): the coroot matrix of the
            // integral simples of `srm.gamma_lambda()` against the real
            // projection at `inv = inv_nr(srm.x())`.
            let subsystem = IntegralSubsystem::integral(self.root_system(), srm.gamma_lambda())?;
            let codec = RepTable::integral_codec(self, srm.x(), &subsystem)?;
            let preimage = codec.theta_1_preimage(&result)?;
            result = result.sub(&RationalWeight::from_weight(&preimage)?)?;
            #[cfg(debug_assertions)]
            {
                // upstream `assert((cd.coroots_matrix*result).is_zero())`
                // (repr.cpp:326); `internalise` post-processes the raw
                // evaluations by the invertible row transform `in`, so it
                // vanishes exactly when they do.
                let evaluations = codec.internalise(&result)?;
                debug_assert!(
                    evaluations.iter().all(|&evaluation| evaluation == 0),
                    "difference made orthogonal to the integral system"
                );
            }
        }
        Ok(result)
    }

    /// `Rep_context::make_relative_to` (repr.cpp:338-350): adapt `bm`,
    /// whose locator part currently holds the query's attitude, so that it
    /// relates the stored block (locator `loc`, representative `srm0`) to
    /// the query `srm1`: post-multiply `bm.w` by `loc.w^{-1}`,
    /// right-compose `bm.simple_pi` with the inverse of `loc.simple_pi`,
    /// move `srm1` back to the base attitude by `transform<true>(bm.w)`,
    /// and set `bm.shift` to the integral-orthogonal difference of the
    /// `gamma_lambda` values.
    pub fn make_relative_to(
        &self,
        loc: &BlockLocator,
        srm0: &StandardReprMod,
        bm: &mut BlockModifier,
        mut srm1: StandardReprMod,
    ) -> Result<(), StructureError> {
        bm.locator.make_relative_to(self.root_system(), loc)?; // repr.cpp:343-345
        self.transform_srm::<true>(bm.w(), &mut srm1)?; // repr.cpp:347
        let shift = self.make_diff_integral_orthogonal(srm1.gamma_lambda(), srm0)?; // repr.cpp:348-349
        bm.shift = shift;
        Ok(())
    }

    /// `Rep_context::sr(srm, bm, gamma)` (repr.cpp:815-823): read a stored
    /// row `srm` at the query attitude as a full standard parameter at
    /// infinitesimal character `gamma` — apply `bm.shift` to `gamlam`
    /// first, then transport by `bm.w` via `transform<false>`, then
    /// restore through the no-modifier `sr` (repr.cpp:808-813), which
    /// [`StandardReprMod::to_standard`] already is.
    pub fn sr_with_modifier(
        &self,
        srm: &StandardReprMod,
        bm: &BlockModifier,
        gamma: &RationalWeight,
    ) -> Result<StandardRepr, StructureError> {
        let mut srm = srm.clone();
        let shifted = srm.gamma_lambda().add(bm.shift())?; // repr.cpp:819
        srm.set_gamma_lambda(shifted);
        self.transform_srm::<false>(bm.w(), &mut srm)?; // repr.cpp:820
        srm.to_standard(self, gamma) // repr.cpp:821-822
    }
}

/// `RootDatum::simple_reflect(s, v)` (rootdata.h:610-611) and the offset
/// variant `simple_reflect(s, v, d)` (rootdata.h:617-618), applied to a
/// rational weight's numerator with fixed denominator: `v -= alpha_s *
/// (<v, coroot_s> + offset)`.
fn simple_reflect_numerator(
    datum: &BasedRootDatum,
    generator: usize,
    numerator: &mut [i64],
    offset: i64,
) -> Result<(), StructureError> {
    if generator >= datum.semisimple_rank() {
        return Err(StructureError::IndexOutOfRange {
            index: generator,
            upper_bound: datum.semisimple_rank(),
        });
    }
    let coroot = datum.simple_coroots()[generator].as_slice();
    let mut dot = offset;
    for (&entry, &coroot_coordinate) in numerator.iter().zip(coroot) {
        let term = i64::from(coroot_coordinate)
            .checked_mul(entry)
            .ok_or(StructureError::ArithmeticOverflow)?;
        dot = dot
            .checked_add(term)
            .ok_or(StructureError::ArithmeticOverflow)?;
    }
    let root = datum.simple_roots()[generator].as_slice();
    for (entry, &root_coordinate) in numerator.iter_mut().zip(root) {
        let term = dot
            .checked_mul(i64::from(root_coordinate))
            .ok_or(StructureError::ArithmeticOverflow)?;
        *entry = entry
            .checked_sub(term)
            .ok_or(StructureError::ArithmeticOverflow)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AdjointFiberBudget, CartanClassification, CartanClassificationBudget, CartanId,
        CommonContext, Coweight, InnerClass, IntegerLatticeBudget, IntegralDatumTable,
        InvolutionTable, InvolutionTableBudget, KgbGraph, KgbId, LatticeInvolution, PartialBlock,
        RealFormSeed, StrongRealClassification, WeakRealFormId, Weight,
    };

    fn class_budget(weyl: usize) -> CartanClassificationBudget {
        CartanClassificationBudget::new(
            IntegerLatticeBudget::new(64, 100_000, 100_000, 128),
            AdjointFiberBudget::new(
                IntegerLatticeBudget::new(64, 100_000, 100_000, 128),
                50_000,
                100_000,
            ),
            weyl,
            64,
            64,
        )
    }

    struct ContextFixture {
        inner_class: InnerClass,
        table: InvolutionTable,
        graph: KgbGraph,
    }

    impl ContextFixture {
        fn rc(&self) -> RepContext<'_> {
            RepContext::new(&self.inner_class, &self.table, &self.graph).unwrap()
        }
    }

    /// The frozen anchor of `tests/fixtures/domain/common_block_locator.atlas`:
    /// `simply_connected(Lie_type("A2"),true)`, split inner class
    /// `[[0,1],[1,0]]`, and the real form with KGB size 4 (SL(3,R), which is
    /// `real_form(ic,0)` upstream — no other weak form of this inner class
    /// has KGB size 4).
    ///
    /// Coordinates are fundamental-weight coordinates, exactly as in the
    /// fixture: the simply-connected datum has simple roots `alpha1 =
    /// [2,-1]`, `alpha2 = [-1,2]` (Cartan rows) and simple coroots `e1`,
    /// `e2`, so `theta = [1,1]` with coroot `[1,1]`, `rho = [1,1]`, and the
    /// root lattice inside X^* = Z^2 is cut out by `2x + y == 0 mod 3`.
    ///
    /// KGB layout (asserted in the round-trip test): x=0 is the base point
    /// of the fundamental fibre with `theta = delta = [[0,1],[1,0]]` (both
    /// simple roots complex); x=1 = cross(s1, x=0) has `theta = s1 delta
    /// s1 = [[1,0],[-1,-1]]` with alpha1 imaginary noncompact; x=2 =
    /// cross(s0, x=0) has `theta = [[-1,-1],[0,1]]` with alpha2 imaginary
    /// noncompact; x=3 is the unique element over the split Cartan with
    /// `theta = -1` (both roots real).
    fn sl3r_fixture() -> ContextFixture {
        let datum = BasedRootDatum::from_simple_data(
            2,
            vec![vec![2, -1], vec![-1, 2]],
            vec![Weight::new(vec![2, -1]), Weight::new(vec![-1, 2])],
            vec![Coweight::new(vec![1, 0]), Coweight::new(vec![0, 1])],
        )
        .unwrap();
        let involution = LatticeInvolution::new(
            &datum,
            vec![vec![0, 1], vec![1, 0]],
            vec![vec![0, 1], vec![1, 0]],
        )
        .unwrap();
        let inner_class = InnerClass::new(datum, involution, 6).unwrap();
        let classification = CartanClassification::build(&inner_class, &class_budget(6)).unwrap();
        let strong = StrongRealClassification::build(&classification, 4_096).unwrap();
        let mut table = InvolutionTable::new(
            &inner_class,
            InvolutionTableBudget::new(64, IntegerLatticeBudget::new(64, 100_000, 100_000, 128)),
        )
        .unwrap();
        let mut graph = None;
        for form in 0..classification.weak_real_form_count() {
            if strong.kgb_size(WeakRealFormId(form)) != Some(4) {
                continue;
            }
            table.add_cartan(&classification, CartanId(0)).unwrap();
            let seed = RealFormSeed::build(
                &inner_class,
                &classification,
                &strong,
                &table,
                WeakRealFormId(form),
                &IntegerLatticeBudget::new(64, 100_000, 100_000, 128),
                4_096,
            )
            .unwrap();
            graph = Some(
                KgbGraph::build(&inner_class, &classification, &strong, &mut table, &seed).unwrap(),
            );
            break;
        }
        ContextFixture {
            inner_class,
            table,
            graph: graph.expect("SL(3,R) has KGB size 4"),
        }
    }

    /// `param(KGB(rf,x), [0,0], nu)` (atlas-types.w:6215-6232): the
    /// standard parameter `sr_gamma(x, [0,0], gamma)` with `gamma = (lambda
    /// + nu + theta(lambda - nu))/2`, `lambda = rho`, reduced modulo `X^*`.
    fn param_srm(
        rc: &RepContext<'_>,
        x: usize,
        nu: &RationalWeight,
    ) -> (StandardReprMod, StandardRepr) {
        let lambda_rho = Weight::new(vec![0, 0]);
        let gamma = rc.gamma(KgbId(x), &lambda_rho, nu).unwrap();
        let sr = rc.sr_gamma(KgbId(x), &lambda_rho, &gamma).unwrap();
        let srm = StandardReprMod::mod_reduce(rc, &sr).unwrap();
        (srm, sr)
    }

    fn rational(numerator: &[i64], denominator: i64) -> RationalWeight {
        RationalWeight::new(numerator.to_vec(), denominator).unwrap()
    }

    /// Field-wise equality, excluding `height`, exactly like upstream's
    /// `StandardRepr::operator==` (repr.cpp:36-40).
    fn same_parameter(left: &StandardRepr, right: &StandardRepr) -> bool {
        left.x() == right.x() && left.y_bits() == right.y_bits() && left.gamma() == right.gamma()
    }

    /// Test (a) of the slice brief: an identity `block_modifier` (trivial
    /// constructor: identity `w`, identity `simple_pi`, zero shift) makes
    /// `sr(srm, bm, gamma)` coincide with the plain `sr(srm, gamma)` —
    /// i.e. `StandardReprMod::to_standard`.
    ///
    /// The shift adds `0` (a no-op) and `transform<false>` walks the empty
    /// word, leaving only the closing `real_unique`, which is idempotent
    /// on `build`-normalised values.
    #[test]
    fn identity_block_modifier_sr_matches_plain_sr() {
        let fixture = sl3r_fixture();
        let rc = fixture.rc();
        let (srm, _) = param_srm(&rc, 3, &rational(&[2, 1], 2));
        let gamma = rational(&[2, 1], 2);

        let simp_int = vec![rc.root_system().simple_root_ids()[0]];
        let bm = BlockModifier::trivial(rc.root_system(), simp_int).unwrap();
        assert!(bm.w().is_identity());
        assert_eq!(bm.simple_pi(), &[0]);
        assert!(bm.shift().is_zero());

        let transported = rc.sr_with_modifier(&srm, &bm, &gamma).unwrap();
        let plain = srm.to_standard(&rc, &gamma).unwrap();
        assert!(same_parameter(&transported, &plain));
    }

    /// Test (b) of the slice brief: the `make_relative_to` round trip on
    /// the A2 SL(3,R) anchor pair of
    /// `tests/fixtures/domain/common_block_locator.atlas`
    /// (p = `param(KGB(rf,3),[0,0],[2,1]/2)` installs the rank-one block,
    /// q = `param(KGB(rf,0),[0,0],[-2,-1]/2)` collides with it; the oracle
    /// prints `as transformed by <1>`).
    ///
    /// Hand derivation, all in fundamental-weight coordinates:
    ///
    /// * x=3 lies over the split Cartan, `theta = -1`, so
    ///   `gamma_p = (rho + nu + theta(rho - nu))/2 = nu = [2,1]/2`.  Its
    ///   integral roots are `{+-alpha1}` (`<gamma, e1> = 1`, `<gamma, e2> =
    ///   1/2`).
    /// * x=0 is the fundamental-fibre base point, `theta = delta`, so
    ///   `gamma_q = ([1,1] + [-1,-1/2] + delta([2,3/2]))/2 = ([1,1] +
    ///   [-1,-1/2] + [3/2,2])/2 = [3,5]/4`.  Its only integral root
    ///   direction is `theta` (`<gamma, [1,1]> = 2`): a Weyl-conjugate
    ///   rank-one system.
    /// * `int_item([2,1]/2)`: the root-lattice alcove vertex is `[1,1]`
    ///   (the `[1,0]` and `[2,0]` wall intersections fail `2x + y == 0 mod
    ///   3`), leaving `[0,-1]/2`; `factor_dominant` gives word `[1,0]` and
    ///   dominant `[1,0]/2`, which lies on the `alpha2` wall only, so the
    ///   canonical datum is the A1 subsystem on `alpha2`.  Filtering the
    ///   word right-to-left keeps both letters (evaluations `1/2` and
    ///   `1/2`), so `loc_p.w = s1*s0` (word `[1,0]`) and `simp_int =
    ///   [s1(s0(alpha2))] = [s1(theta)] = [alpha1]`.
    /// * `int_item([3,5]/4)`: vertex `[1,1]`, leaving `[-1,1]/4`; word
    ///   `[0]`, dominant `[1,0]/4`, again on the `alpha2` wall — the SAME
    ///   canonical datum — and the filter keeps the letter (evaluation
    ///   `1/4`), so `loc_q.w = s0` (word `[0]`) and `simp_int =
    ///   [s0(alpha2)] = [theta]`.
    /// * `make_relative_to(loc_p, srm0, bm = loc_q, srm_q)`:
    ///   `bm.w = s0 * (s1*s0)^{-1} = s0*s0*s1 = s1` — word `[1]`, the
    ///   oracle's `<1>`.
    /// * `srm_p = (x=3, [0,3]/2)` and `srm_q = (x=0, [-1,1]/4)` after
    ///   `real_unique`.  The stored block at p's attitude is the full
    ///   common block of the `alpha1` subsystem: three rows, `(x=1,
    ///   [0,-1]/2)`, `(x=3, [0,1]/2)`, `(x=3, [0,3]/2)`.  q's row `srm0`
    ///   is the one at `x = cross(s1, x=0) = 1`: generator 1 is complex at
    ///   x=0 (`delta(alpha2) = alpha1`), so `transform<false>(s1)` moves
    ///   it to x=0.
    /// * `transform<true>(s1, srm_q)`: cross x=0 to x=1 and reflect the
    ///   numerator, `[-1,1] - 1*[-1,2] = [0,-1]`, giving `srm1_base =
    ///   (x=1, [0,-1]/4)`.  The difference `diff = [0,-1]/4 - [0,-1]/2 =
    ///   [0,1]/4` is already orthogonal to the integral coroot `e1` of
    ///   `srm0.gamma_lambda() = [0,-1]/2` (its integral simples are
    ///   `[alpha1]`), so `theta_1_preimage(diff) = 0` and `bm.shift =
    ///   [0,1]/4`.
    /// * Round trip: `shift` gives `(x=1, [0,-1]/2 + [0,1]/4 = [0,-1]/4)`
    ///   = `srm1_base` exactly, and `transform<false>(s1)` inverts the
    ///   earlier `transform<true>` letter-for-letter, landing exactly on
    ///   `srm_q` — exact equality, not just up to root translation.
    #[test]
    fn a2_sl3r_make_relative_to_round_trip() {
        let fixture = sl3r_fixture();
        let rc = fixture.rc();
        let system = rc.root_system().clone();

        // The documented KGB layout of SL(3,R).
        assert_eq!(
            rc.theta_at(KgbId(0)).unwrap().weight_matrix(),
            &[[0, 1], [1, 0]]
        );
        assert_eq!(
            rc.theta_at(KgbId(1)).unwrap().weight_matrix(),
            &[[1, 0], [-1, -1]]
        );
        assert_eq!(
            rc.theta_at(KgbId(3)).unwrap().weight_matrix(),
            &[[-1, 0], [0, -1]]
        );
        assert_eq!(rc.kgb_status(KgbId(0), 1).unwrap(), KgbStatus::Complex);
        assert_eq!(rc.cross_at(KgbId(0), 1).unwrap(), KgbId(1));
        assert_eq!(rc.kgb_status(KgbId(3), 0).unwrap(), KgbStatus::Real);

        let (srm_p, _) = param_srm(&rc, 3, &rational(&[2, 1], 2));
        let (srm_q, q) = param_srm(&rc, 0, &rational(&[-2, -1], 2));
        assert_eq!(q.gamma(), &rational(&[3, 5], 4));
        assert_eq!(srm_p.x(), KgbId(3));
        assert_eq!(srm_p.gamma_lambda(), &rational(&[0, 3], 2));
        assert_eq!(srm_q.x(), KgbId(0));
        assert_eq!(srm_q.gamma_lambda(), &rational(&[-1, 1], 4));

        // The locators collide on one canonical datum, with attitudes as
        // derived above.
        let mut table = IntegralDatumTable::new();
        let (item_p, loc_p) = table.int_item(&system, &rational(&[2, 1], 2)).unwrap();
        let (item_q, loc_q) = table.int_item(&system, &rational(&[3, 5], 4)).unwrap();
        assert_eq!(item_p, item_q);
        assert_eq!(loc_p.w().reduced_word(&system).unwrap(), vec![1, 0]);
        assert_eq!(loc_q.w().reduced_word(&system).unwrap(), vec![0]);
        let alpha1 = system.simple_root_ids()[0];
        let theta_root = system
            .id_of(&Weight::new(vec![1, 1]))
            .expect("theta is a root");
        assert_eq!(loc_p.simp_int(), &[alpha1]);
        assert_eq!(loc_q.simp_int(), &[theta_root]);
        assert_eq!(loc_p.simple_pi(), &[0]);
        assert_eq!(loc_q.simple_pi(), &[0]);

        // p's stored block at its own attitude (subsystem simples
        // `[alpha1]`), and q's row `srm0` in it.
        let context = CommonContext::integral(&rc, srm_p.gamma_lambda()).unwrap();
        assert_eq!(context.subsystem().parent_root(0), Some(alpha1));
        let (block, _) = PartialBlock::build_full(&context, &srm_p).unwrap();
        assert_eq!(block.size(), 3);
        let srm0 = (0..block.size())
            .map(|z| block.element(z).unwrap())
            .find(|element| element.x() == KgbId(1))
            .expect("q's row is the element over x=1")
            .clone();
        assert_eq!(srm0.gamma_lambda(), &rational(&[0, -1], 2));

        // Reduced_param::reduce installs the query's locator into bm
        // (repr.cpp:1802 passes bm as the locator output); then
        // make_relative_to adapts it relative to the stored block.
        let mut bm = BlockModifier::from_locator(loc_q, RationalWeight::zero(2).unwrap());
        rc.make_relative_to(&loc_p, &srm0, &mut bm, srm_q.clone())
            .unwrap();
        assert_eq!(bm.w().reduced_word(&system).unwrap(), vec![1]); // oracle: <1>
        assert_eq!(bm.simple_pi(), &[0]);
        assert_eq!(bm.shift(), &rational(&[0, 1], 4));

        // The round trip: shift, then transform<false>, lands exactly on
        // the query srm.
        let mut back = srm0.clone();
        rc.shift_srm(bm.shift(), &mut back).unwrap();
        assert_eq!(back.gamma_lambda(), &rational(&[0, -1], 4));
        rc.transform_srm::<false>(bm.w(), &mut back).unwrap();
        assert_eq!(back, srm_q);

        // And the full modifier-carrying sr restores q itself.
        let restored = rc.sr_with_modifier(&srm0, &bm, q.gamma()).unwrap();
        assert!(same_parameter(&restored, &q));
    }

    /// `common_block::singular(bm, gamma)` at a non-identity attitude
    /// (blocks.cpp:711-721, via `simply_ints`, blocks.cpp:1274-1283): the
    /// stored block's simply-integral root is permuted by `bm.w` before the
    /// pairing. On this fixture's rank-one block (simply-integral root
    /// `alpha1`, coroot `e1 = [1,0]`) with `w = s1`, the permuted root is
    /// `s1(alpha1) = theta = [1,1]` with coroot `[1,1]`; `gamma = [1,-1]`
    /// is orthogonal to the latter but not the former, so it is singular
    /// only through the modifier path. An identity `w` reproduces the plain
    /// flags (blocks.cpp:701-708).
    #[test]
    fn singular_flags_permuted_by_block_modifier_w() {
        let fixture = sl3r_fixture();
        let rc = fixture.rc();
        let system = rc.root_system().clone();
        let (srm_p, _) = param_srm(&rc, 3, &rational(&[2, 1], 2));
        let context = CommonContext::integral(&rc, srm_p.gamma_lambda()).unwrap();
        assert_eq!(context.rank(), 1);

        let gamma = rational(&[1, -1], 1);
        assert_eq!(context.singular_flags(&gamma).unwrap(), vec![false]);
        let w = WeylElement::simple_reflection(&system, 1).unwrap();
        assert_eq!(
            context.singular_flags_with_modifier(&gamma, &w).unwrap(),
            vec![true]
        );
        let identity = WeylElement::identity(&system).unwrap();
        assert_eq!(
            context
                .singular_flags_with_modifier(&gamma, &identity)
                .unwrap(),
            vec![false]
        );
    }
}
