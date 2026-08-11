//! The real Weyl group and block stabilizer of a (real form, Cartan) pair
//! (upstream `io/realweyl.{h,cpp}`, printed by `io/realweyl_io.cpp`).
//!
//! [`RealWeyl`] ports the upstream `realweyl::RealWeyl` constructor
//! (realweyl.h:36-181, realweyl.cpp:34-69): for a Cartan class `cc`, an
//! adjoint fiber element `x` (the real form's representative) and a dual
//! adjoint fiber element `y` (the dual real form's representative), it
//! collects the simple imaginary/real/complex roots with their subsystem
//! types, the compact-root bases and the two R-groups.
//! [`RealWeylGenerators`] ports `realweyl::RealWeylGenerators`
//! (realweyl.h:183-236, realweyl.cpp:81-159): one Weyl element per listed
//! root (per kernel bit vector for the R-groups, per root plus its
//! involution image for the complex roots).
//!
//! The printing layer ports `realweyl_io::common_print`
//! (realweyl_io.cpp:72-146) behind the two headers of `printRealWeyl`
//! (:175-183) and `printBlockStabilizer` (:164-173), reached here through
//! [`RealWeylContext::real_weyl_print`] and
//! [`RealWeylContext::block_stabilizer_print`]. Those entry points
//! also port the interpreter wrappers (atlas-types.w:8828-8847 and
//! :8920-8932) and the `x`/`y` selection of `output::printRealWeyl`
//! (output.cpp:445-474: `x = G_C.representative(rf,cn)`, `y` the zero
//! element of the dual adjoint fiber) and `output::printBlockStabilizer`
//! (output.cpp:361-390: `y = G_C.dualRepresentative(drf,cn)`); the
//! representative lookups are innerclass.h:457-470. Upstream's gkmod
//! `blockstabilizer` subsystem is NOT needed: the wrapper only forwards
//! `(rf, cn, drf)`.
//!
//! Helper ports:
//!
//! - `orthogonalMAlpha` (realweyl.cpp:234-249) and `rGenerators`
//!   (realweyl.cpp:264-279, the `BinaryMap::kernel` of
//!   bitvector.cpp:234-280 over the `Gauss_Jordan` canonical basis of
//!   bitvector.cpp:673-697) appear as [`fiber_side`].
//! - `compactTwoRho` (cartanclass.cpp:827-830) is inlined in
//!   [`fiber_side`]; the all-ones base grading (`Fiber::noncompactRoots`,
//!   cartanclass.cpp:706-712) is evaluated per root as the parity of the
//!   simple-imaginary coordinate sum, read off as `<alpha, rho_vee_im>`
//!   with `2 rho_vee_im` the sum of the positive imaginary coroots — the
//!   same linear extension upstream `makeBaseGrading` uses, avoiding the
//!   rational-matrix inverse of `real_form_order.rs`.
//! - `RootSystem::simpleBasis` (rootdata.cpp:621-652) is [`simple_basis`],
//!   including the outer-loop break quirk: once a candidate removes
//!   itself, later candidates are never examined.
//! - `CartanClass::makeSimpleComplex` (cartanclass.cpp:1002-1044) is
//!   [`simple_complex`].
//! - `RootSystem::subsystem_type` (rootdata.cpp:537-540) is
//!   [`subsystem_cartan`] plus [`lie_type`]; the crate's
//!   `dynkin::classify` reproduces upstream's component order.
//!
//! Dual side: upstream reads the real side off `cc.dualFiber()`, the fiber
//! built from `theta.negative_transposed()` on the dual datum
//! (cartanclass.cpp:121), whose root NUMBERS are the primal ones. The
//! crate rebuilds that fiber chain ad hoc: the dual Cartan involution
//! `-theta` is the twisted involution `tw * w0` of the dual inner class
//! (innerclass.cpp:435-441), which is generally only a CONJUGATE of the
//! canonical dual Cartan representative, so the dual classification's
//! stored fiber cannot be reused; [`RealWeylContext::dual_side`] rebuilds
//! fiber, grading, weak-real partition, and form labels at `tw * w0`
//! exactly. The resulting dual roots are then mapped back to primal
//! [`RootId`]s through the coroot vectors (dual root vectors ARE primal
//! coroot vectors). The dual subsystem Cartan matrix is the primal one
//! transposed, so `realType` and `realCompactType` are computed with a
//! transposed pairing; this is where the B/C swap enters (oracle:
//! `Sp(4,R)` Cartan #3 prints `W^R is a Weyl group of type B2` on a C2
//! datum).
//!
//! Deviations from upstream, all deliberate:
//!
//! - Generator WORDS are canonical by construction: upstream accumulates
//!   `rd.reflection_word` products but prints `WeylGroup::word`
//!   (weyl.cpp:944-957), the transducer canonical word. The crate builds
//!   the elements from root reflections directly and prints
//!   [`WeylElement::canonical_word`] — the same canonical words, so the
//!   verbatim `reflection_word`/`to_dominant` machinery is not ported.
//! - The `#ifndef NDEBUG` size assertions at the end of
//!   `output::printRealWeyl`/`printBlockStabilizer` (and the `weylsize`
//!   computations feeding them) are not ported.
//! - `printDualRealWeyl` (realweyl_io.cpp:186-195) is not ported: no
//!   builtin wrapper uses it. The generator lists it would need
//!   (`imaginary`, `real` and their types) are still computed, so adding
//!   it later is a printing-only change.

use std::collections::BTreeSet;

use crate::cartan_classification::upstream_positive_key;
use crate::grading::try_capacity;
use crate::{
    longest_action, pair, AdjointCartanFiber, AdjointFiberElement, CartanClass,
    CartanClassification, CartanClassificationBudget, CartanFiber, CartanGradingData, CartanId,
    CayleyCrossDecomposition, ExternalFormOrder, InnerClass, ModTwoSubspace, ModTwoVector,
    RealFormLabels, RootId, RootInvolutionData, RootKind, RootSystem, StructureError,
    TwistedInvolution, WeakRealFormId, WeakRealFormPartition, Weight, WeylAction, WeylElement,
    WeylGroup, WeylInterface,
};

/// One typed component of a Lie type, printed by upstream
/// `operator<<(SimpleLieType)` (basic_io.cpp:78-81) as letter then rank.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LieTypeComponent {
    pub letter: char,
    pub rank: usize,
}

/// The upstream `realweyl::RealWeyl` payload (realweyl.h:36-181).
///
/// All root lists hold PRIMAL root ids in upstream list order (ascending
/// upstream `RootNbr` order, except `complex`, which is the
/// `makeSimpleComplex` output order); the dual-side lists (`real_compact`,
/// `real_orth`) are mapped back through the coroot vectors as described in
/// the module header. The R-group bit vectors have one coordinate per
/// `orth` entry, in the upstream kernel-generator order of
/// `BinaryMap::kernel` (bitvector.cpp:268-277).
#[derive(Clone, Debug)]
pub struct RealWeyl {
    imaginary: Vec<RootId>,
    imaginary_compact: Vec<RootId>,
    imaginary_orth: Vec<RootId>,
    imaginary_r: Vec<ModTwoVector>,
    real: Vec<RootId>,
    real_compact: Vec<RootId>,
    real_orth: Vec<RootId>,
    real_r: Vec<ModTwoVector>,
    complex: Vec<RootId>,
    complex_type: Vec<LieTypeComponent>,
    imaginary_type: Vec<LieTypeComponent>,
    imaginary_compact_type: Vec<LieTypeComponent>,
    real_type: Vec<LieTypeComponent>,
    real_compact_type: Vec<LieTypeComponent>,
}

/// The shared inputs of the two print wrappers: the primal and dual inner
/// classes with their Cartan classifications, plus the classification
/// budget for the ad-hoc dual fiber chain of
/// [`RealWeylContext::dual_side`].
#[derive(Clone, Copy, Debug)]
pub struct RealWeylContext<'a> {
    pub inner_class: &'a InnerClass,
    pub classification: &'a CartanClassification,
    pub dual_inner_class: &'a InnerClass,
    pub dual_classification: &'a CartanClassification,
    pub budget: &'a CartanClassificationBudget,
}

/// The ad-hoc dual fiber chain built by [`RealWeylContext::dual_side`]:
/// the exact `tw * w0` twisted involution with its grading, weak real form
/// partition, and form labels.
struct DualSide {
    twisted: TwistedInvolution,
    grading: CartanGradingData,
    partition: WeakRealFormPartition,
    labels: RealFormLabels,
}

impl RealWeylContext<'_> {
    /// The upstream constructor (realweyl.cpp:34-69), with the `x`/`y`
    /// representatives resolved as in `output::printRealWeyl`/
    /// `printBlockStabilizer`: `x` is the representative of `form` on
    /// `cartan`; `y` is the zero dual adjoint fiber element when
    /// `dual_form` is `None`, else the representative of that dual form on
    /// the dual Cartan of `cartan`.
    ///
    /// `RealFormNotDefinedOnCartan` is the wrapper's "Cartan class not
    /// defined for real form" (atlas-types.w:8842-8846): the (dual) form
    /// has no fiber on this Cartan class.
    pub fn real_weyl(
        &self,
        form: WeakRealFormId,
        cartan: CartanId,
        dual_form: Option<WeakRealFormId>,
    ) -> Result<RealWeyl, StructureError> {
        let root_system = self.inner_class.root_system();
        let classification = self.classification;
        let cartan_class =
            classification
                .cartan_class(cartan)
                .ok_or(StructureError::IndexOutOfRange {
                    index: cartan.0,
                    upper_bound: classification.cartan_classes().len(),
                })?;
        let local = cartan_class
            .labels()
            .labels()
            .iter()
            .position(|&label| label == form)
            .ok_or(StructureError::RealFormNotDefinedOnCartan)?;
        let x = cartan_class
            .partition()
            .class_representative(WeakRealFormId(local))
            .ok_or(StructureError::CartanClassificationInvariantViolation {
                invariant: "real-form representative",
            })?;

        let dual = self.dual_side(cartan_class)?;
        let dual_system = self.dual_inner_class.root_system();
        let y = match dual_form {
            None => dual.grading.adjoint_fiber().identity()?,
            Some(dual_form) => {
                let dual_local = dual
                    .labels
                    .labels()
                    .iter()
                    .position(|&label| label == dual_form)
                    .ok_or(StructureError::RealFormNotDefinedOnCartan)?;
                dual.partition
                    .class_representative(WeakRealFormId(dual_local))
                    .ok_or(StructureError::CartanClassificationInvariantViolation {
                        invariant: "dual real-form representative",
                    })?
                    .clone()
            }
        };

        let involution = cartan_class.representative().root_involution();
        let imaginary = sort_by_upstream_key(root_system, involution.imaginary_simple_roots())?;
        let real = sort_by_upstream_key(root_system, involution.real_simple_roots())?;
        let complex = simple_complex(root_system, involution)?;
        let primal_side = fiber_side(root_system, involution, cartan_class.grading(), x)?;

        let dual_side = fiber_side(
            dual_system,
            dual.twisted.root_involution(),
            &dual.grading,
            &y,
        )?;
        let real_compact =
            primal_roots_of_dual(root_system, dual_system, &dual_side.compact_basis)?;
        let real_orth = primal_roots_of_dual(root_system, dual_system, &dual_side.orth)?;

        // `complexType`/`imaginaryType`/`imaginaryCompactType` pair on the
        // primal side; `realType`/`realCompactType` are upstream
        // `drd.subsystem_type` calls, whose Cartan matrices are the primal
        // ones transposed.
        let complex_type = lie_type(&subsystem_cartan(root_system, &complex, false)?)?;
        let imaginary_type = lie_type(&subsystem_cartan(root_system, &imaginary, false)?)?;
        let imaginary_compact_type = lie_type(&subsystem_cartan(
            root_system,
            &primal_side.compact_basis,
            false,
        )?)?;
        let real_type = lie_type(&subsystem_cartan(root_system, &real, true)?)?;
        let real_compact_type = lie_type(&subsystem_cartan(root_system, &real_compact, true)?)?;

        Ok(RealWeyl {
            imaginary,
            imaginary_compact: primal_side.compact_basis,
            imaginary_orth: primal_side.orth,
            imaginary_r: primal_side.r_vectors,
            real,
            real_compact,
            real_orth,
            real_r: dual_side.r_vectors,
            complex,
            complex_type,
            imaginary_type,
            imaginary_compact_type,
            real_type,
            real_compact_type,
        })
    }

    /// `output::printRealWeyl` (output.cpp:445-474) with the wrapper's
    /// external form numbering (atlas-types.w:8828-8847): `form` is the
    /// interpreter's `real_form(ic, form)` number, translated through
    /// [`ExternalFormOrder`]. The dual fiber element is the zero element
    /// (the dual quasisplit representative), as upstream hard-codes.
    pub fn real_weyl_print(
        &self,
        form: usize,
        cartan: usize,
    ) -> Result<RealWeylPrint, StructureError> {
        let order = ExternalFormOrder::build(self.inner_class, self.classification)?;
        let internal = order
            .internal(form)
            .ok_or(StructureError::IndexOutOfRange {
                index: form,
                upper_bound: order.form_count(),
            })?;
        let cartan_id = checked_cartan(self.classification, cartan)?;
        let real_weyl = self.real_weyl(internal, cartan_id, None)?;
        let involution = self
            .classification
            .cartan_class(cartan_id)
            .expect("checked cartan id")
            .representative()
            .root_involution();
        let generators =
            RealWeylGenerators::build(&real_weyl, self.inner_class.root_system(), involution)?;
        common_print(
            PrintKind::RealWeyl,
            &real_weyl,
            &generators,
            self.inner_class.root_system(),
        )
    }

    /// `output::printBlockStabilizer` (output.cpp:361-390) with external
    /// form numbers on both sides (atlas-types.w:8920-8932): `dual_form`
    /// is an external form number of the DUAL inner class, e.g. the dual
    /// quasisplit form (`ExternalFormOrder::quasisplit_external` of the
    /// dual).
    pub fn block_stabilizer_print(
        &self,
        form: usize,
        cartan: usize,
        dual_form: usize,
    ) -> Result<RealWeylPrint, StructureError> {
        let order = ExternalFormOrder::build(self.inner_class, self.classification)?;
        let internal = order
            .internal(form)
            .ok_or(StructureError::IndexOutOfRange {
                index: form,
                upper_bound: order.form_count(),
            })?;
        let dual_order = ExternalFormOrder::build(self.dual_inner_class, self.dual_classification)?;
        let dual_internal =
            dual_order
                .internal(dual_form)
                .ok_or(StructureError::IndexOutOfRange {
                    index: dual_form,
                    upper_bound: dual_order.form_count(),
                })?;
        let cartan_id = checked_cartan(self.classification, cartan)?;
        let real_weyl = self.real_weyl(internal, cartan_id, Some(dual_internal))?;
        let involution = self
            .classification
            .cartan_class(cartan_id)
            .expect("checked cartan id")
            .representative()
            .root_involution();
        let generators =
            RealWeylGenerators::build(&real_weyl, self.inner_class.root_system(), involution)?;
        common_print(
            PrintKind::BlockStabilizer,
            &real_weyl,
            &generators,
            self.inner_class.root_system(),
        )
    }

    /// The ad-hoc dual fiber chain at the exact Cartan involution `-theta`
    /// of this Cartan class (see the module header): the primal
    /// representative's canonical Weyl word, replayed on the dual datum
    /// and right-multiplied by the dual longest element, is `tw * w0` of
    /// the dual inner class; fiber, grading, weak real form partition and
    /// labels are rebuilt on it exactly as [`CartanClassification::build`]
    /// does per class.
    fn dual_side(&self, cartan_class: &CartanClass) -> Result<DualSide, StructureError> {
        let primal_system = self.inner_class.root_system();
        let interface = WeylInterface::new(self.inner_class.datum().cartan_matrix())?;
        let word =
            WeylElement::from_action(primal_system, cartan_class.representative().weyl_action())?
                .canonical_word(primal_system, &interface)?;

        let dual_group = WeylGroup::new(self.dual_inner_class.datum().clone());
        let mut action = dual_group.identity()?;
        for generator in word {
            action = action.compose(&dual_group.simple_reflection(generator)?)?;
        }
        action = action.compose(&longest_action(
            self.dual_inner_class,
            self.budget.weyl_budget(),
        )?)?;

        let dual_system = self.dual_inner_class.root_system();
        let twisted = TwistedInvolution::new(
            self.dual_inner_class.datum(),
            dual_system,
            self.dual_inner_class
                .distinguished_involution()
                .involution(),
            action,
        )?;
        let data = twisted.root_involution();
        let source = CartanFiber::build(data.involution(), self.budget.integer_lattice())?;
        let adjoint =
            AdjointCartanFiber::build(dual_system, data, &source, self.budget.adjoint_fiber())?;
        let grading = CartanGradingData::build(dual_system, data, &adjoint)?;
        let partition = WeakRealFormPartition::build(&grading, self.budget.max_fiber_elements())?;
        let decomposition = CayleyCrossDecomposition::build(
            self.dual_inner_class,
            &twisted,
            self.budget.max_peeling_steps(),
        )?;
        let fundamental = self.dual_classification.cartan_class(CartanId(0)).ok_or(
            StructureError::CartanClassificationInvariantViolation {
                invariant: "dual fundamental class",
            },
        )?;
        let labels = RealFormLabels::build(
            self.dual_inner_class,
            fundamental.grading(),
            fundamental.partition(),
            &grading,
            &partition,
            &decomposition,
        )?;
        Ok(DualSide {
            twisted,
            grading,
            partition,
            labels,
        })
    }
}

impl RealWeyl {
    pub fn imaginary(&self) -> &[RootId] {
        &self.imaginary
    }

    pub fn imaginary_compact(&self) -> &[RootId] {
        &self.imaginary_compact
    }

    pub fn imaginary_orth(&self) -> &[RootId] {
        &self.imaginary_orth
    }

    pub fn imaginary_r(&self) -> &[ModTwoVector] {
        &self.imaginary_r
    }

    pub fn real(&self) -> &[RootId] {
        &self.real
    }

    pub fn real_compact(&self) -> &[RootId] {
        &self.real_compact
    }

    pub fn real_orth(&self) -> &[RootId] {
        &self.real_orth
    }

    pub fn real_r(&self) -> &[ModTwoVector] {
        &self.real_r
    }

    pub fn complex(&self) -> &[RootId] {
        &self.complex
    }

    pub fn complex_type(&self) -> &[LieTypeComponent] {
        &self.complex_type
    }

    pub fn imaginary_type(&self) -> &[LieTypeComponent] {
        &self.imaginary_type
    }

    pub fn imaginary_compact_type(&self) -> &[LieTypeComponent] {
        &self.imaginary_compact_type
    }

    pub fn real_type(&self) -> &[LieTypeComponent] {
        &self.real_type
    }

    pub fn real_compact_type(&self) -> &[LieTypeComponent] {
        &self.real_compact_type
    }
}

/// The upstream `realweyl::RealWeylGenerators` payload (realweyl.h:183-236):
/// one Weyl element per root list of [`RealWeyl`], products over the kernel
/// bit vectors for the R-groups, and `s_rn . s_theta(rn)` for the complex
/// roots (realweyl.cpp:148-156).
#[derive(Clone, Debug)]
pub struct RealWeylGenerators {
    imaginary: Vec<WeylElement>,
    imaginary_compact: Vec<WeylElement>,
    imaginary_r: Vec<WeylElement>,
    real: Vec<WeylElement>,
    real_compact: Vec<WeylElement>,
    real_r: Vec<WeylElement>,
    complex: Vec<WeylElement>,
}

impl RealWeylGenerators {
    /// The upstream constructor (realweyl.cpp:81-159). Elements are built
    /// from the root reflections directly; the printed words are canonical
    /// regardless (see the module header).
    pub fn build(
        real_weyl: &RealWeyl,
        root_system: &RootSystem,
        involution: &RootInvolutionData,
    ) -> Result<Self, StructureError> {
        let datum = root_system.datum();
        let reflect = |root: RootId| -> Result<WeylElement, StructureError> {
            WeylElement::from_action(
                root_system,
                &WeylAction::root_reflection(datum, root_system, root)?,
            )
        };
        let identity = WeylElement::identity(root_system)?;

        let mut imaginary = try_capacity(real_weyl.imaginary.len())?;
        for &root in &real_weyl.imaginary {
            imaginary.push(reflect(root)?);
        }
        let mut imaginary_compact = try_capacity(real_weyl.imaginary_compact.len())?;
        for &root in &real_weyl.imaginary_compact {
            imaginary_compact.push(reflect(root)?);
        }
        let imaginary_r = r_group_elements(
            root_system,
            &real_weyl.imaginary_orth,
            &real_weyl.imaginary_r,
            &identity,
            &reflect,
        )?;
        let mut real = try_capacity(real_weyl.real.len())?;
        for &root in &real_weyl.real {
            real.push(reflect(root)?);
        }
        let mut real_compact = try_capacity(real_weyl.real_compact.len())?;
        for &root in &real_weyl.real_compact {
            real_compact.push(reflect(root)?);
        }
        let real_r = r_group_elements(
            root_system,
            &real_weyl.real_orth,
            &real_weyl.real_r,
            &identity,
            &reflect,
        )?;
        let mut complex = try_capacity(real_weyl.complex.len())?;
        for &root in &real_weyl.complex {
            let image =
                involution
                    .image(root)
                    .ok_or(StructureError::RootSystemInvariantViolation {
                        invariant: "involution image",
                    })?;
            complex.push(reflect(root)?.multiply(root_system, &reflect(image)?)?);
        }

        Ok(Self {
            imaginary,
            imaginary_compact,
            imaginary_r,
            real,
            real_compact,
            real_r,
            complex,
        })
    }

    pub fn imaginary(&self) -> &[WeylElement] {
        &self.imaginary
    }

    pub fn imaginary_compact(&self) -> &[WeylElement] {
        &self.imaginary_compact
    }

    pub fn imaginary_r(&self) -> &[WeylElement] {
        &self.imaginary_r
    }

    pub fn real(&self) -> &[WeylElement] {
        &self.real
    }

    pub fn real_compact(&self) -> &[WeylElement] {
        &self.real_compact
    }

    pub fn real_r(&self) -> &[WeylElement] {
        &self.real_r
    }

    pub fn complex(&self) -> &[WeylElement] {
        &self.complex
    }
}

/// One `generators for ...` block of the printout: the header line (with
/// upstream's inconsistent colon, realweyl_io.cpp:105-145) and one printed
/// word per generator.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RealWeylGeneratorSection {
    pub header: String,
    pub words: Vec<String>,
}

/// The structured printout of `realweyl_io::common_print`
/// (realweyl_io.cpp:72-146): header line, four summary lines, a blank
/// line, then the generator sections of the nontrivial factors.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RealWeylPrint {
    pub header: String,
    pub summaries: Vec<String>,
    pub generator_sections: Vec<RealWeylGeneratorSection>,
}

impl RealWeylPrint {
    /// The exact byte stream upstream writes: every line terminated,
    /// exactly one blank line between the summaries and the sections, and
    /// nothing after the last section (or after the blank line when there
    /// are no sections).
    pub fn render(&self) -> String {
        let mut output = String::new();
        output.push_str(&self.header);
        output.push('\n');
        for line in &self.summaries {
            output.push_str(line);
            output.push('\n');
        }
        output.push('\n');
        for section in &self.generator_sections {
            output.push_str(&section.header);
            output.push('\n');
            for word in &section.words {
                output.push_str(word);
                output.push('\n');
            }
        }
        output
    }
}

/// Upstream `which_group` (realweyl_io.cpp:52); `dual_real_W` is unported.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PrintKind {
    RealWeyl,
    BlockStabilizer,
}

/// `realweyl_io::common_print` (realweyl_io.cpp:72-146), including the
/// `isomorphic to` prefix of a nontrivial `W^C` and the missing colons of
/// the A-group section headers.
fn common_print(
    kind: PrintKind,
    real_weyl: &RealWeyl,
    generators: &RealWeylGenerators,
    root_system: &RootSystem,
) -> Result<RealWeylPrint, StructureError> {
    let interface = WeylInterface::new(root_system.datum().cartan_matrix())?;
    let words_of = |elements: &[WeylElement]| -> Result<Vec<String>, StructureError> {
        let mut words = try_capacity(elements.len())?;
        for element in elements {
            words.push(format_word(
                &element.canonical_word(root_system, &interface)?,
            ));
        }
        Ok(words)
    };

    let header = match kind {
        PrintKind::RealWeyl => "real weyl group is W^C.((A.W_ic) x W^R), where:".to_string(),
        PrintKind::BlockStabilizer => {
            "block stabilizer is W^C.((A_i.W_ic) x (A_r.W_rc)), where:".to_string()
        }
    };

    let mut summaries = try_capacity(4)?;
    summaries.push(format!(
        "W^C is {}{}",
        if real_weyl.complex.is_empty() {
            ""
        } else {
            "isomorphic to "
        },
        weyl_type(real_weyl.complex.len(), &real_weyl.complex_type),
    ));
    let imaginary_r_label = match kind {
        PrintKind::RealWeyl => "A",
        PrintKind::BlockStabilizer => "A_i",
    };
    summaries.push(format!(
        "{imaginary_r_label} is {}",
        two_type(real_weyl.imaginary_r.len()),
    ));
    summaries.push(format!(
        "W_ic is {}",
        weyl_type(
            real_weyl.imaginary_compact.len(),
            &real_weyl.imaginary_compact_type,
        ),
    ));
    match kind {
        PrintKind::RealWeyl => summaries.push(format!(
            "W^R is {}",
            weyl_type(real_weyl.real.len(), &real_weyl.real_type),
        )),
        PrintKind::BlockStabilizer => {
            summaries.push(format!("A_r is {}", two_type(real_weyl.real_r.len()),));
            summaries.push(format!(
                "W_rc is {}",
                weyl_type(real_weyl.real_compact.len(), &real_weyl.real_compact_type,),
            ));
        }
    }

    let mut generator_sections = Vec::new();
    if !real_weyl.complex.is_empty() {
        generator_sections.push(RealWeylGeneratorSection {
            header: "generators for W^C:".to_string(),
            words: words_of(&generators.complex)?,
        });
    }
    if !real_weyl.imaginary_r.is_empty() {
        generator_sections.push(RealWeylGeneratorSection {
            header: format!("generators for {imaginary_r_label}"),
            words: words_of(&generators.imaginary_r)?,
        });
    }
    if !real_weyl.imaginary_compact.is_empty() {
        generator_sections.push(RealWeylGeneratorSection {
            header: "generators for W_ic:".to_string(),
            words: words_of(&generators.imaginary_compact)?,
        });
    }
    match kind {
        PrintKind::RealWeyl => {
            if !real_weyl.real.is_empty() {
                generator_sections.push(RealWeylGeneratorSection {
                    header: "generators for W^R:".to_string(),
                    words: words_of(&generators.real)?,
                });
            }
        }
        PrintKind::BlockStabilizer => {
            if !real_weyl.real_r.is_empty() {
                generator_sections.push(RealWeylGeneratorSection {
                    header: "generators for A_r".to_string(),
                    words: words_of(&generators.real_r)?,
                });
            }
            if !real_weyl.real_compact.is_empty() {
                generator_sections.push(RealWeylGeneratorSection {
                    header: "generators for W_rc:".to_string(),
                    words: words_of(&generators.real_compact)?,
                });
            }
        }
    }

    Ok(RealWeylPrint {
        header,
        summaries,
        generator_sections,
    })
}

/// `Weyl_type` (realweyl_io.cpp:56-62): `n` the generator count, `t` the
/// subsystem Lie type.
fn weyl_type(count: usize, lie_type: &[LieTypeComponent]) -> String {
    if count == 0 {
        return "trivial".to_string();
    }
    let components = lie_type
        .iter()
        .map(|component| format!("{}{}", component.letter, component.rank))
        .collect::<Vec<_>>()
        .join(".");
    format!("a Weyl group of type {components}")
}

/// `two_type` (realweyl_io.cpp:64-70).
fn two_type(rank: usize) -> String {
    if rank == 0 {
        return "trivial".to_string();
    }
    format!("an elementary abelian 2-group of rank {rank}")
}

/// `operator<<(WeylWord)` (basic_io.cpp:109-124): the identity prints `e`,
/// anything else the comma-joined 1-based generator numbers.
fn format_word(word: &[usize]) -> String {
    if word.is_empty() {
        return "e".to_string();
    }
    word.iter()
        .map(|generator| (generator + 1).to_string())
        .collect::<Vec<_>>()
        .join(",")
}

/// The one-sided fiber package of the upstream `RealWeyl` constructor
/// (realweyl.cpp:54-68): the compact simple basis, the orthogonal
/// noncompact roots, and the R-group kernel generators. Runs on either the
/// primal or the dual side, in the side's own root ids.
struct FiberSide {
    compact_basis: Vec<RootId>,
    orth: Vec<RootId>,
    r_vectors: Vec<ModTwoVector>,
}

fn fiber_side(
    root_system: &RootSystem,
    involution: &RootInvolutionData,
    grading: &CartanGradingData,
    element: &AdjointFiberElement,
) -> Result<FiberSide, StructureError> {
    let positive_imaginary = sort_by_upstream_key(
        root_system,
        &involution
            .roots_of_kind(RootKind::Imaginary)
            .filter(|&root| root_system.is_positive(root) == Some(true))
            .collect::<Vec<_>>(),
    )?;

    // `Fiber::noncompactRoots` (cartanclass.cpp:706-712): the linear
    // extension of the all-ones base grading, translated by the fiber
    // element. The base grading of a root is the parity of its
    // simple-imaginary coordinate sum, here evaluated as
    // `<alpha, rho_vee_im>` with `2 rho_vee_im` the sum of the positive
    // imaginary coroots (each simple-imaginary root pairs to 1); the
    // translation is the mod-two dot with the ambient representative.
    let ambient = grading.adjoint_fiber().canonical_representative(element)?;
    let rank = root_system.lattice_rank();
    let mut compact = try_capacity(positive_imaginary.len())?;
    let mut noncompact = try_capacity(positive_imaginary.len())?;
    let mut two_rho_ic = vec![0_i32; rank];
    for &alpha in &positive_imaginary {
        let mut doubled_sum = 0_i64;
        for &beta in &positive_imaginary {
            doubled_sum = doubled_sum
                .checked_add(i64::from(root_system.bracket(alpha, beta)?))
                .ok_or(StructureError::ArithmeticOverflow)?;
        }
        if doubled_sum % 2 != 0 {
            return Err(StructureError::RootSystemInvariantViolation {
                invariant: "imaginary-simple coordinates",
            });
        }
        let base_noncompact = (doubled_sum / 2) % 2 != 0;
        let coordinates = root_system.simple_coordinates(alpha).ok_or(
            StructureError::RootSystemInvariantViolation {
                invariant: "root lookup",
            },
        )?;
        let weight =
            root_system
                .root(alpha)
                .ok_or(StructureError::RootSystemInvariantViolation {
                    invariant: "root lookup",
                })?;
        if base_noncompact ^ parity_dot(&ambient, coordinates) {
            noncompact.push(alpha);
        } else {
            compact.push(alpha);
            for (slot, &value) in two_rho_ic.iter_mut().zip(weight.as_slice()) {
                *slot = slot
                    .checked_add(value)
                    .ok_or(StructureError::ArithmeticOverflow)?;
            }
        }
    }

    // `rd.simpleBasis(f.compactRoots(x))` (realweyl.cpp:55).
    let compact_basis = simple_basis(root_system, &compact)?;

    // `orthogonalMAlpha` (realweyl.cpp:234-249): the positive noncompact
    // imaginary roots orthogonal to `compactTwoRho` — all strongly
    // orthogonal, so an A_1^n system.
    let two_rho = Weight::new(two_rho_ic);
    let mut orth = try_capacity(noncompact.len())?;
    for &alpha in &noncompact {
        let coroot =
            root_system
                .coroot(alpha)
                .ok_or(StructureError::RootSystemInvariantViolation {
                    invariant: "root lookup",
                })?;
        if pair(&two_rho, coroot)? == 0 {
            orth.push(alpha);
        }
    }

    // `rGenerators` (realweyl.cpp:264-279): the kernel of the mod-two map
    // taking each orth root to its `m_alpha`, computed as upstream
    // `BitMatrix::kernel` (bitvector.cpp:234-280): one generator per free
    // column, ascending, with pivot bits read off the canonical basis rows.
    let fiber = grading.adjoint_fiber().ambient_fiber();
    let count = orth.len();
    let mut rows = ModTwoSubspace::new(count)?;
    if count > 0 {
        let mut row_ones = vec![Vec::new(); fiber.dimension()];
        for (column_index, &alpha) in orth.iter().enumerate() {
            let coroot =
                root_system
                    .coroot(alpha)
                    .ok_or(StructureError::RootSystemInvariantViolation {
                        invariant: "root lookup",
                    })?;
            let m_alpha = fiber.element_from_coweight_mod_two(coroot)?;
            let coordinates = fiber.coordinates(&m_alpha)?;
            for (bit, ones) in row_ones.iter_mut().enumerate() {
                if coordinates.bit(bit) == Some(true) {
                    ones.push(column_index);
                }
            }
        }
        for ones in row_ones {
            if !ones.is_empty() {
                rows.insert(ModTwoVector::from_ones(count, ones)?)?;
            }
        }
    }
    let pivot_columns: BTreeSet<usize> = rows.pivot_rows().map(|(pivot, _)| pivot).collect();
    let mut r_vectors = Vec::new();
    for free in 0..count {
        if pivot_columns.contains(&free) {
            continue;
        }
        let mut ones = vec![free];
        for (pivot, row) in rows.pivot_rows() {
            if row.bit(free) == Some(true) {
                ones.push(pivot);
            }
        }
        r_vectors.push(ModTwoVector::from_ones(count, ones)?);
    }

    Ok(FiberSide {
        compact_basis,
        orth,
        r_vectors,
    })
}

/// `RootSystem::simpleBasis` (rootdata.cpp:621-652), including the quirk
/// that a candidate which removes ITSELF ends the whole scan: later
/// candidates are never examined. Callers pass positive roots only
/// (upstream clears the negative half of its input set first).
fn simple_basis(
    root_system: &RootSystem,
    positive_roots: &[RootId],
) -> Result<Vec<RootId>, StructureError> {
    let ordered = sort_by_upstream_key(root_system, positive_roots)?;
    let mut candidates: BTreeSet<RootId> = ordered.iter().copied().collect();
    'outer: for &alpha in &ordered {
        if !candidates.contains(&alpha) {
            continue;
        }
        for &beta in &ordered {
            if beta == alpha {
                continue;
            }
            // `gamma = s_alpha(beta) < beta` iff `<beta, alpha_vee> > 0`,
            // since the reflection lowers the height by that multiple of
            // the (positive) height of alpha.
            let pairing = root_system.bracket(beta, alpha)?;
            if pairing > 0 {
                let gamma = reflect_weight(root_system, beta, alpha, pairing)?;
                let gamma_id = root_system.id_of(&gamma).ok_or(
                    StructureError::RootSystemInvariantViolation {
                        invariant: "simple-basis reflection",
                    },
                )?;
                if root_system.is_positive(gamma_id) == Some(true) {
                    candidates.remove(&beta);
                } else {
                    candidates.remove(&alpha);
                    break 'outer;
                }
            }
        }
    }
    Ok(ordered
        .into_iter()
        .filter(|root| candidates.contains(root))
        .collect())
}

/// `CartanClass::makeSimpleComplex` (cartanclass.cpp:1002-1044): a simple
/// basis of the complex root subsystem, chosen so that the Dynkin
/// components pair up under the involution and only one of each pair is
/// kept (the FIRST later component containing a root non-orthogonal to the
/// image of the current component's lowest root is erased).
fn simple_complex(
    root_system: &RootSystem,
    involution: &RootInvolutionData,
) -> Result<Vec<RootId>, StructureError> {
    let rank = root_system.lattice_rank();
    let mut tri = vec![0_i32; rank];
    let mut trr = vec![0_i32; rank];
    for root in involution.roots_of_kind(RootKind::Imaginary) {
        accumulate_weight(root_system, root, &mut tri)?;
    }
    for root in involution.roots_of_kind(RootKind::Real) {
        accumulate_weight(root_system, root, &mut trr)?;
    }
    let tri = Weight::new(tri);
    let trr = Weight::new(trr);

    let mut orthogonal = Vec::new();
    for (id, _, coroot) in root_system.entries() {
        if root_system.is_positive(id) != Some(true) {
            continue;
        }
        if pair(&tri, coroot)? == 0 && pair(&trr, coroot)? == 0 {
            orthogonal.push(id);
        }
    }
    let rb = simple_basis(root_system, &orthogonal)?;

    let cartan = subsystem_cartan(root_system, &rb, false)?;
    let mut components = crate::dynkin::classify(&cartan)?;
    let mut result = Vec::new();
    let mut index = 0;
    while index < components.len() {
        for &vertex in &components[index].support.clone() {
            result.push(rb[vertex]);
        }
        let image = involution.image(rb[components[index].offset()]).ok_or(
            StructureError::RootSystemInvariantViolation {
                invariant: "involution image",
            },
        )?;
        let mut later = index + 1;
        while later < components.len() {
            let mut hit = false;
            for &vertex in &components[later].support {
                if root_system.bracket(rb[vertex], image)? != 0 {
                    hit = true;
                    break;
                }
            }
            if hit {
                components.remove(later);
                break;
            }
            later += 1;
        }
        index += 1;
    }
    Ok(result)
}

/// The subsystem Cartan matrix (rootdata.cpp:529-534): entry `(i,j)` is
/// `<basis[i], coroot(basis[j])>` — or the transpose when `transposed`,
/// reproducing a `drd.subsystem_type` call (dual roots are primal
/// coroots).
fn subsystem_cartan(
    root_system: &RootSystem,
    basis: &[RootId],
    transposed: bool,
) -> Result<Vec<Vec<i32>>, StructureError> {
    let mut cartan = try_capacity(basis.len())?;
    for &row_root in basis {
        let mut entries = try_capacity(basis.len())?;
        for &column_root in basis {
            let (root, coroot) = if transposed {
                (column_root, row_root)
            } else {
                (row_root, column_root)
            };
            entries.push(root_system.bracket(root, coroot)?);
        }
        cartan.push(entries);
    }
    Ok(cartan)
}

/// `dynkin::Lie_type` of a subsystem Cartan matrix: the typed components
/// in classification order.
fn lie_type(cartan: &[Vec<i32>]) -> Result<Vec<LieTypeComponent>, StructureError> {
    Ok(crate::dynkin::classify(cartan)?
        .iter()
        .map(|component| LieTypeComponent {
            letter: component.letter,
            rank: component.position.len(),
        })
        .collect())
}

/// The R-group elements (realweyl.cpp:105-114): one product of orth-root
/// reflections per kernel bit vector, set bits taken in ascending order
/// and right-multiplied.
fn r_group_elements(
    root_system: &RootSystem,
    orth: &[RootId],
    vectors: &[ModTwoVector],
    identity: &WeylElement,
    reflect: &impl Fn(RootId) -> Result<WeylElement, StructureError>,
) -> Result<Vec<WeylElement>, StructureError> {
    let mut elements = try_capacity(vectors.len())?;
    for vector in vectors {
        let mut element = identity.clone();
        for (bit, &root) in orth.iter().enumerate() {
            if vector.bit(bit) == Some(true) {
                element = element.multiply(root_system, &reflect(root)?)?;
            }
        }
        elements.push(element);
    }
    Ok(elements)
}

/// Map dual root ids back to primal ones through the weight vectors: dual
/// root vectors are primal coroot vectors (the dual datum interchanges
/// roots and coroots), and positivity is preserved.
fn primal_roots_of_dual(
    primal: &RootSystem,
    dual: &RootSystem,
    dual_roots: &[RootId],
) -> Result<Vec<RootId>, StructureError> {
    let mut by_coroot = std::collections::HashMap::new();
    for (id, _, coroot) in primal.entries() {
        by_coroot.insert(coroot.as_slice().to_vec(), id);
    }
    let mut result = try_capacity(dual_roots.len())?;
    for &dual_root in dual_roots {
        let weight = dual
            .root(dual_root)
            .ok_or(StructureError::RootSystemInvariantViolation {
                invariant: "root lookup",
            })?;
        result.push(*by_coroot.get(weight.as_slice()).ok_or(
            StructureError::CartanClassificationInvariantViolation {
                invariant: "dual root correspondence",
            },
        )?);
    }
    Ok(result)
}

/// `beta - pairing * alpha` as a root-lattice weight (the reflection of
/// `beta` in `alpha`).
fn reflect_weight(
    root_system: &RootSystem,
    beta: RootId,
    alpha: RootId,
    pairing: i32,
) -> Result<Weight, StructureError> {
    let beta_weight =
        root_system
            .root(beta)
            .ok_or(StructureError::RootSystemInvariantViolation {
                invariant: "root lookup",
            })?;
    let alpha_weight =
        root_system
            .root(alpha)
            .ok_or(StructureError::RootSystemInvariantViolation {
                invariant: "root lookup",
            })?;
    let mut gamma = try_capacity(beta_weight.rank())?;
    for (&beta_value, &alpha_value) in beta_weight.as_slice().iter().zip(alpha_weight.as_slice()) {
        gamma.push(
            beta_value
                .checked_sub(
                    pairing
                        .checked_mul(alpha_value)
                        .ok_or(StructureError::ArithmeticOverflow)?,
                )
                .ok_or(StructureError::ArithmeticOverflow)?,
        );
    }
    Ok(Weight::new(gamma))
}

/// Add a positive root's weight to the accumulator (negative roots skip;
/// upstream sums over the positive half of its root sets).
fn accumulate_weight(
    root_system: &RootSystem,
    root: RootId,
    accumulator: &mut [i32],
) -> Result<(), StructureError> {
    if root_system.is_positive(root) != Some(true) {
        return Ok(());
    }
    let weight = root_system
        .root(root)
        .ok_or(StructureError::RootSystemInvariantViolation {
            invariant: "root lookup",
        })?;
    for (slot, &value) in accumulator.iter_mut().zip(weight.as_slice()) {
        *slot = slot
            .checked_add(value)
            .ok_or(StructureError::ArithmeticOverflow)?;
    }
    Ok(())
}

/// Mod-two pairing of ambient fiber coordinates with a root's datum-simple
/// coordinates (the linear extension of the crate's grading shifts; same
/// formula as `real_form_order.rs`).
fn parity_dot(ambient: &ModTwoVector, coordinates: &[i32]) -> bool {
    let mut parity = false;
    for (index, &value) in coordinates.iter().enumerate() {
        if value % 2 != 0 && ambient.bit(index) == Some(true) {
            parity = !parity;
        }
    }
    parity
}

/// Sort positive roots into upstream `RootNbr` order: (height,
/// reverse-lexicographic simple coordinates), see
/// `cartan_classification::upstream_positive_key`.
fn sort_by_upstream_key(
    root_system: &RootSystem,
    roots: &[RootId],
) -> Result<Vec<RootId>, StructureError> {
    let mut keyed = try_capacity(roots.len())?;
    for &root in roots {
        let coordinates = root_system.simple_coordinates(root).ok_or(
            StructureError::RootSystemInvariantViolation {
                invariant: "root lookup",
            },
        )?;
        keyed.push((upstream_positive_key(coordinates)?, root));
    }
    keyed.sort_by(|(left, _), (right, _)| left.cmp(right));
    Ok(keyed.into_iter().map(|(_, root)| root).collect())
}

fn checked_cartan(
    classification: &CartanClassification,
    cartan: usize,
) -> Result<CartanId, StructureError> {
    if cartan >= classification.cartan_classes().len() {
        return Err(StructureError::IndexOutOfRange {
            index: cartan,
            upper_bound: classification.cartan_classes().len(),
        });
    }
    Ok(CartanId(cartan))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        dual_inner_class, AdjointFiberBudget, BasedRootDatum, CartanClassificationBudget, Coweight,
        IntegerLatticeBudget, LatticeInvolution,
    };

    // Probe oracle: /tmp/probe_rw_all.at against the pinned upstream build
    // (rev 4d3e9449); outputs regenerated 2026-08-11 and reproduced here
    // byte-for-byte, including the trailing blank line when no generator
    // sections exist.

    const REAL_W_HEADER: &str = "real weyl group is W^C.((A.W_ic) x W^R), where:";
    const BLOCK_HEADER: &str = "block stabilizer is W^C.((A_i.W_ic) x (A_r.W_rc)), where:";

    struct ClassFixture {
        inner: InnerClass,
        classification: CartanClassification,
        dual: InnerClass,
        dual_classification: CartanClassification,
        budget: CartanClassificationBudget,
    }

    impl ClassFixture {
        fn context(&self) -> RealWeylContext<'_> {
            RealWeylContext {
                inner_class: &self.inner,
                classification: &self.classification,
                dual_inner_class: &self.dual,
                dual_classification: &self.dual_classification,
                budget: &self.budget,
            }
        }
    }

    fn classification_budget(weyl: usize) -> CartanClassificationBudget {
        let integer = IntegerLatticeBudget::new(64, 100_000, 100_000, 128);
        CartanClassificationBudget::new(
            integer.clone(),
            AdjointFiberBudget::new(integer, 50_000, 100_000),
            weyl,
            64,
            64,
        )
    }

    fn fixture(
        datum: BasedRootDatum,
        involution: LatticeInvolution,
        roots: usize,
        weyl: usize,
    ) -> ClassFixture {
        let inner = InnerClass::new(datum, involution, roots).unwrap();
        let budget = classification_budget(weyl);
        let classification = CartanClassification::build(&inner, &budget).unwrap();
        let dual = dual_inner_class(&inner, weyl, roots).unwrap();
        let dual_classification = CartanClassification::build(&dual, &budget).unwrap();
        ClassFixture {
            inner,
            classification,
            dual,
            dual_classification,
            budget,
        }
    }

    fn based_datum(
        rank: usize,
        cartan: Vec<Vec<i32>>,
        roots: Vec<Vec<i32>>,
        coroots: Vec<Vec<i32>>,
    ) -> BasedRootDatum {
        BasedRootDatum::from_simple_data(
            rank,
            cartan,
            roots.into_iter().map(Weight::new).collect(),
            coroots.into_iter().map(Coweight::new).collect(),
        )
        .unwrap()
    }

    fn transpose(matrix: &[Vec<i32>]) -> Vec<Vec<i32>> {
        (0..matrix.len())
            .map(|row| matrix.iter().map(|entries| entries[row]).collect())
            .collect()
    }

    /// For these distinguished involutions (signed permutation matrices)
    /// the coweight action is the transpose of the weight action;
    /// `LatticeInvolution::new` validates the pairing regardless.
    fn involution_of(datum: &BasedRootDatum, weight: Vec<Vec<i32>>) -> LatticeInvolution {
        LatticeInvolution::new(datum, weight.clone(), transpose(&weight)).unwrap()
    }

    /// The SL(3) datum of `groups.at` (columns are the simple roots):
    /// roots (1,-1),(1,2); coroots (1,-1),(0,1).
    fn sl3_datum() -> BasedRootDatum {
        based_datum(
            2,
            vec![vec![2, -1], vec![-1, 2]],
            vec![vec![1, -1], vec![1, 2]],
            vec![vec![1, -1], vec![0, 1]],
        )
    }

    /// `InnerClass:SL(2,R)`: sc A1, delta = identity; forms SU(2), SL(2,R).
    fn ic2() -> ClassFixture {
        let datum = based_datum(1, vec![vec![2]], vec![vec![2]], vec![vec![1]]);
        fixture(
            datum.clone(),
            LatticeInvolution::identity(&datum).unwrap(),
            2,
            2,
        )
    }

    /// `InnerClass:SU(2,1)`: the SL(3) datum, delta = identity; forms
    /// SU(3), SU(2,1).
    fn icu() -> ClassFixture {
        let datum = sl3_datum();
        fixture(
            datum.clone(),
            LatticeInvolution::identity(&datum).unwrap(),
            8,
            8,
        )
    }

    /// `InnerClass:SL(3,R)`: the SL(3) datum, delta = w0; the single form
    /// SL(3,R).
    fn ics() -> ClassFixture {
        let datum = sl3_datum();
        let delta = involution_of(&datum, vec![vec![1, 0], vec![1, -1]]);
        fixture(datum, delta, 8, 8)
    }

    /// `InnerClass:Sp(4,R)`: the Sp(4) datum (roots (1,-1),(0,2); coroots
    /// (1,-1),(0,1)), delta = identity; forms Sp(2), Sp(1,1), Sp(4,R).
    fn icp() -> ClassFixture {
        let datum = based_datum(
            2,
            vec![vec![2, -1], vec![-2, 2]],
            vec![vec![1, -1], vec![0, 2]],
            vec![vec![1, -1], vec![0, 1]],
        );
        fixture(
            datum.clone(),
            LatticeInvolution::identity(&datum).unwrap(),
            8,
            16,
        )
    }

    /// `InnerClass:SL(2,C)`: doubled sc A1, delta = factor swap; the
    /// single complex form.
    fn icc() -> ClassFixture {
        let datum = based_datum(
            2,
            vec![vec![2, 0], vec![0, 2]],
            vec![vec![2, 0], vec![0, 2]],
            vec![vec![1, 0], vec![0, 1]],
        );
        let delta = involution_of(&datum, vec![vec![0, 1], vec![1, 0]]);
        fixture(datum, delta, 4, 4)
    }

    /// `InnerClass:SL(4,R)`: the SL(4) datum, delta = w0; forms sl(2,H),
    /// SL(4,R).
    fn ic4() -> ClassFixture {
        let datum = based_datum(
            3,
            vec![vec![2, -1, 0], vec![-1, 2, -1], vec![0, -1, 2]],
            vec![vec![1, -1, 0], vec![0, 1, -1], vec![1, 1, 2]],
            vec![vec![1, -1, 0], vec![0, 1, -1], vec![0, 0, 1]],
        );
        let delta = involution_of(&datum, vec![vec![1, 0, 0], vec![1, 0, -1], vec![1, -1, 0]]);
        fixture(datum, delta, 16, 64)
    }

    /// `InnerClass:SL(6,R)`: the SL(6) datum, delta = w0; forms sl(3,H),
    /// SL(6,R).
    fn ic6() -> ClassFixture {
        let datum = based_datum(
            5,
            vec![
                vec![2, -1, 0, 0, 0],
                vec![-1, 2, -1, 0, 0],
                vec![0, -1, 2, -1, 0],
                vec![0, 0, -1, 2, -1],
                vec![0, 0, 0, -1, 2],
            ],
            vec![
                vec![1, -1, 0, 0, 0],
                vec![0, 1, -1, 0, 0],
                vec![0, 0, 1, -1, 0],
                vec![0, 0, 0, 1, -1],
                vec![1, 1, 1, 1, 2],
            ],
            vec![
                vec![1, -1, 0, 0, 0],
                vec![0, 1, -1, 0, 0],
                vec![0, 0, 1, -1, 0],
                vec![0, 0, 0, 1, -1],
                vec![0, 0, 0, 0, 1],
            ],
        );
        let delta = involution_of(
            &datum,
            vec![
                vec![1, 0, 0, 0, 0],
                vec![1, 0, 0, 0, -1],
                vec![1, 0, 0, -1, 0],
                vec![1, 0, -1, 0, 0],
                vec![1, -1, 0, 0, 0],
            ],
        );
        fixture(datum, delta, 64, 4096)
    }

    /// Join lines with trailing newlines, matching `RealWeylPrint::render`
    /// byte-for-byte (the blank line is an empty entry).
    fn expected(lines: &[&str]) -> String {
        let mut output = String::new();
        for line in lines {
            output.push_str(line);
            output.push('\n');
        }
        output
    }

    fn real_weyl_output(fx: &ClassFixture, form: usize, cartan: usize) -> String {
        fx.context().real_weyl_print(form, cartan).unwrap().render()
    }

    /// `print_blockstabilizer(block(G, dual_quasisplit_form(ic)), C)`.
    fn block_stabilizer_output(fx: &ClassFixture, form: usize, cartan: usize) -> String {
        let dual_quasisplit = ExternalFormOrder::build(&fx.dual, &fx.dual_classification)
            .unwrap()
            .quasisplit_external();
        fx.context()
            .block_stabilizer_print(form, cartan, dual_quasisplit)
            .unwrap()
            .render()
    }

    #[test]
    fn fixture_form_and_cartan_counts_match_the_oracle() {
        let counts = |fx: &ClassFixture| {
            (
                fx.classification.weak_real_form_count(),
                fx.classification.cartan_classes().len(),
            )
        };
        assert_eq!(counts(&ic2()), (2, 2));
        assert_eq!(counts(&icu()), (2, 2));
        assert_eq!(counts(&ics()), (1, 2));
        assert_eq!(counts(&icp()), (3, 4));
        assert_eq!(counts(&icc()), (1, 1));
        assert_eq!(counts(&ic4()), (2, 3));
        assert_eq!(counts(&ic6()), (2, 4));
    }

    #[test]
    fn a1_real_weyl_matches_oracle() {
        let fx = ic2();
        assert_eq!(
            real_weyl_output(&fx, 0, 0),
            expected(&[
                REAL_W_HEADER,
                "W^C is trivial",
                "A is trivial",
                "W_ic is a Weyl group of type A1",
                "W^R is trivial",
                "",
                "generators for W_ic:",
                "1",
            ])
        );
        // Split form on the fundamental Cartan: all factors trivial, the
        // output ends at the blank line.
        assert_eq!(
            real_weyl_output(&fx, 1, 0),
            expected(&[
                REAL_W_HEADER,
                "W^C is trivial",
                "A is trivial",
                "W_ic is trivial",
                "W^R is trivial",
                "",
            ])
        );
        assert_eq!(
            real_weyl_output(&fx, 1, 1),
            expected(&[
                REAL_W_HEADER,
                "W^C is trivial",
                "A is trivial",
                "W_ic is trivial",
                "W^R is a Weyl group of type A1",
                "",
                "generators for W^R:",
                "1",
            ])
        );
    }

    #[test]
    fn su21_real_weyl_matches_oracle() {
        let fx = icu();
        assert_eq!(
            real_weyl_output(&fx, 0, 0),
            expected(&[
                REAL_W_HEADER,
                "W^C is trivial",
                "A is trivial",
                "W_ic is a Weyl group of type A2",
                "W^R is trivial",
                "",
                "generators for W_ic:",
                "1",
                "2",
            ])
        );
        assert_eq!(
            real_weyl_output(&fx, 1, 0),
            expected(&[
                REAL_W_HEADER,
                "W^C is trivial",
                "A is trivial",
                "W_ic is a Weyl group of type A1",
                "W^R is trivial",
                "",
                "generators for W_ic:",
                "1,2,1",
            ])
        );
        assert_eq!(
            real_weyl_output(&fx, 1, 1),
            expected(&[
                REAL_W_HEADER,
                "W^C is trivial",
                "A is trivial",
                "W_ic is trivial",
                "W^R is a Weyl group of type A1",
                "",
                "generators for W^R:",
                "1,2,1",
            ])
        );
    }

    #[test]
    fn sl3r_real_weyl_and_block_stabilizer_match_oracle() {
        let fx = ics();
        assert_eq!(
            real_weyl_output(&fx, 0, 0),
            expected(&[
                REAL_W_HEADER,
                "W^C is trivial",
                "A is an elementary abelian 2-group of rank 1",
                "W_ic is trivial",
                "W^R is trivial",
                "",
                "generators for A",
                "1,2,1",
            ])
        );
        assert_eq!(
            real_weyl_output(&fx, 0, 1),
            expected(&[
                REAL_W_HEADER,
                "W^C is trivial",
                "A is trivial",
                "W_ic is trivial",
                "W^R is a Weyl group of type A2",
                "",
                "generators for W^R:",
                "1",
                "2",
            ])
        );
        assert_eq!(
            block_stabilizer_output(&fx, 0, 0),
            expected(&[
                BLOCK_HEADER,
                "W^C is trivial",
                "A_i is an elementary abelian 2-group of rank 1",
                "W_ic is trivial",
                "A_r is trivial",
                "W_rc is trivial",
                "",
                "generators for A_i",
                "1,2,1",
            ])
        );
        assert_eq!(
            block_stabilizer_output(&fx, 0, 1),
            expected(&[
                BLOCK_HEADER,
                "W^C is trivial",
                "A_i is trivial",
                "W_ic is trivial",
                "A_r is trivial",
                "W_rc is a Weyl group of type A1",
                "",
                "generators for W_rc:",
                "1,2,1",
            ])
        );
    }

    #[test]
    fn sp4_real_weyl_and_block_stabilizer_match_oracle() {
        let fx = icp();
        assert_eq!(
            real_weyl_output(&fx, 0, 0),
            expected(&[
                REAL_W_HEADER,
                "W^C is trivial",
                "A is trivial",
                "W_ic is a Weyl group of type C2",
                "W^R is trivial",
                "",
                "generators for W_ic:",
                "1",
                "2",
            ])
        );
        assert_eq!(
            real_weyl_output(&fx, 1, 0),
            expected(&[
                REAL_W_HEADER,
                "W^C is trivial",
                "A is trivial",
                "W_ic is a Weyl group of type A1.A1",
                "W^R is trivial",
                "",
                "generators for W_ic:",
                "2",
                "1,2,1",
            ])
        );
        assert_eq!(
            real_weyl_output(&fx, 1, 1),
            expected(&[
                REAL_W_HEADER,
                "W^C is trivial",
                "A is trivial",
                "W_ic is a Weyl group of type A1",
                "W^R is a Weyl group of type A1",
                "",
                "generators for W_ic:",
                "1",
                "generators for W^R:",
                "2,1,2",
            ])
        );
        assert_eq!(
            real_weyl_output(&fx, 2, 0),
            expected(&[
                REAL_W_HEADER,
                "W^C is trivial",
                "A is trivial",
                "W_ic is a Weyl group of type A1",
                "W^R is trivial",
                "",
                "generators for W_ic:",
                "2,1,2",
            ])
        );
        assert_eq!(
            real_weyl_output(&fx, 2, 1),
            expected(&[
                REAL_W_HEADER,
                "W^C is trivial",
                "A is an elementary abelian 2-group of rank 1",
                "W_ic is trivial",
                "W^R is a Weyl group of type A1",
                "",
                "generators for A",
                "1",
                "generators for W^R:",
                "2,1,2",
            ])
        );
        assert_eq!(
            real_weyl_output(&fx, 2, 2),
            expected(&[
                REAL_W_HEADER,
                "W^C is trivial",
                "A is trivial",
                "W_ic is trivial",
                "W^R is a Weyl group of type A1",
                "",
                "generators for W^R:",
                "1,2,1",
            ])
        );
        // The dual-side transposed pairing turns the C2 real subsystem
        // into B2.
        assert_eq!(
            real_weyl_output(&fx, 2, 3),
            expected(&[
                REAL_W_HEADER,
                "W^C is trivial",
                "A is trivial",
                "W_ic is trivial",
                "W^R is a Weyl group of type B2",
                "",
                "generators for W^R:",
                "1",
                "2",
            ])
        );
        assert_eq!(
            block_stabilizer_output(&fx, 2, 1),
            expected(&[
                BLOCK_HEADER,
                "W^C is trivial",
                "A_i is an elementary abelian 2-group of rank 1",
                "W_ic is trivial",
                "A_r is an elementary abelian 2-group of rank 1",
                "W_rc is trivial",
                "",
                "generators for A_i",
                "1",
                "generators for A_r",
                "2,1,2",
            ])
        );
        assert_eq!(
            block_stabilizer_output(&fx, 2, 2),
            expected(&[
                BLOCK_HEADER,
                "W^C is trivial",
                "A_i is trivial",
                "W_ic is trivial",
                "A_r is an elementary abelian 2-group of rank 1",
                "W_rc is trivial",
                "",
                "generators for A_r",
                "1,2,1",
            ])
        );
    }

    #[test]
    fn sl2c_complex_weyl_matches_oracle() {
        let fx = icc();
        assert_eq!(
            real_weyl_output(&fx, 0, 0),
            expected(&[
                REAL_W_HEADER,
                "W^C is isomorphic to a Weyl group of type A1",
                "A is trivial",
                "W_ic is trivial",
                "W^R is trivial",
                "",
                "generators for W^C:",
                "1,2",
            ])
        );
    }

    #[test]
    fn sl4r_real_weyl_matches_oracle() {
        let fx = ic4();
        assert_eq!(
            real_weyl_output(&fx, 1, 0),
            expected(&[
                REAL_W_HEADER,
                "W^C is isomorphic to a Weyl group of type A1",
                "A is an elementary abelian 2-group of rank 1",
                "W_ic is trivial",
                "W^R is trivial",
                "",
                "generators for W^C:",
                "1,3",
                "generators for A",
                "1,2,1,3,2,1",
            ])
        );
    }

    #[test]
    fn sl6r_multi_generator_r_group_matches_oracle() {
        let fx = ic6();
        // The key kernel-ordering case: a rank-2 A-group whose generators
        // come out in ascending free-column order (bitvector.cpp:268-277).
        assert_eq!(
            real_weyl_output(&fx, 1, 0),
            expected(&[
                REAL_W_HEADER,
                "W^C is isomorphic to a Weyl group of type A2",
                "A is an elementary abelian 2-group of rank 2",
                "W_ic is trivial",
                "W^R is trivial",
                "",
                "generators for W^C:",
                "1,5",
                "2,4",
                "generators for A",
                "2,3,2,4,3,2",
                "1,2,3,2,4,5,4,3,2,1",
            ])
        );
        assert_eq!(
            real_weyl_output(&fx, 1, 1),
            expected(&[
                REAL_W_HEADER,
                "W^C is isomorphic to a Weyl group of type A1",
                "A is an elementary abelian 2-group of rank 2",
                "W_ic is trivial",
                "W^R is a Weyl group of type A1",
                "",
                "generators for W^C:",
                "2,4",
                "generators for A",
                "3",
                "2,3,4,3,2",
                "generators for W^R:",
                "1,2,3,4,5,4,3,2,1",
            ])
        );
    }

    #[test]
    fn undefined_cartan_for_form_is_the_wrapper_error() {
        // Oracle: "Cartan class not defined for real form" (the compact
        // SU(3) has only the fundamental Cartan).
        let fx = icu();
        assert!(matches!(
            fx.context().real_weyl_print(0, 1),
            Err(StructureError::RealFormNotDefinedOnCartan)
        ));
        // Oracle: Sp(2) has only Cartan #0; Sp(1,1) has Cartans #0 and #1.
        let fx = icp();
        for (form, cartan) in [(0, 1), (0, 2), (1, 2)] {
            assert!(matches!(
                fx.context().real_weyl_print(form, cartan),
                Err(StructureError::RealFormNotDefinedOnCartan)
            ));
        }
    }

    #[test]
    fn out_of_range_form_and_cartan_are_index_errors() {
        let fx = ic2();
        assert!(matches!(
            fx.context().real_weyl_print(2, 0),
            Err(StructureError::IndexOutOfRange {
                index: 2,
                upper_bound: 2,
            })
        ));
        assert!(matches!(
            fx.context().real_weyl_print(0, 2),
            Err(StructureError::IndexOutOfRange {
                index: 2,
                upper_bound: 2,
            })
        ));
    }
}
