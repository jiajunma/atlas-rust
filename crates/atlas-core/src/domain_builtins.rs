//! Domain builtins: the language bridge to `atlas-real-group` (phase 1).
//!
//! Named-function application dispatches here. Handles are Arc-backed
//! context bundles built EAGERLY (build-then-freeze — the KGB pipeline
//! needs `&mut` state that an immutable `Value` cannot carry) and compare
//! STRUCTURALLY on (inner-class value, internal form number, element id),
//! matching upstream's memoized-handle observable equality. Construction
//! budgets are session constants, revisited when the language exposes
//! budget control. Display strings follow upstream byte-for-byte where the
//! upstream form is stable (`KGB element #n`, `Lie type '...'`); the
//! inner-class and real-form prints replicate `inner_class_value::print`
//! and `real_form_value::print` (interpreter/atlas-types.w:3164-3172,
//! 3566-3575) from the layout, dual-count, and presentation machinery of
//! `atlas-real-group`.

use std::cmp::Ordering;
use std::collections::{BTreeSet, HashMap};
use std::fmt;
use std::fmt::Write as _;
use std::num::{NonZeroI32, NonZeroU64};
use std::sync::{Arc, Mutex, Weak};
use std::time::{Duration, Instant};

use malachite::{Integer as BigInt, Rational as BigRational};

use atlas_real_group::ext_block::ExtBlock;
use atlas_real_group::ext_kl::ExtKlTable;
use atlas_real_group::ext_param::{
    extended_finalise, extended_restrict_to_k, is_default, scaled_extended_finalise,
    shifted_default_extension, ExtRepContext,
};
use atlas_real_group::CompactWeyl;
use atlas_real_group::{
    adapted_basis, adapted_relation_basis, alcove_center as domain_alcove_center,
    annihilator_modulo as relation_annihilator_modulo, block_deformation_to_height,
    bourbaki_permutation, bruhat_below, bruhat_hasse as block_bruhat_hasse, build_presentations,
    central_fiber, checked_inner_class_letters, classify_involution as domain_classify_involution,
    denominator_exceeds_alcove_bound, dual_cartan_correspondence, dual_inner_class,
    dual_involution as block_dual_involution, elected_square_root, fiber_rank,
    filter_relation_units as domain_filter_relation_units, inner_class_with_twisted_involution,
    integral_block_scope, layout_involution, longest_action, minimal_torus_part,
    on_basis as lattice_on_basis, pair, quotient_relation_basis as domain_quotient_relation_basis,
    replace_relation_generators as domain_replace_relation_generators, singular_orbits_at,
    twisted_deformation_terms, twisted_deformation_with_cancel, twisted_kl_column_at_s,
    twisted_kl_sum, AdjointFiberBudget, BasedRootDatum, BlockDescent, BlockGraph, BlockTopology,
    CartanClassification, CartanClassificationBudget, CartanId, CommonContext, Coweight,
    ExternalFormOrder, GlobalKgb, InnerClass, InnerClassLayout, IntegerLatticeBudget,
    IntegralBlockScope, IntegralSubsystem, InvolutionId, InvolutionTable, InvolutionTableBudget,
    KType, KgbGraph, KgbId, KgbStatus, KlPol, KlTable, LatticeInvolution, LocatedBlock,
    ModTwoVector, PartialBlock, RankFlags, RationalWeight, RealFormPresentation, RealFormSeed,
    RelationBasis, RelationError, RelationGenerator, RelationMatrix, RepContext, RepTableOwner,
    RootId, RootInvolutionData, RootKind, RootSystem, SplitInteger, StandardRepr, StandardReprMod,
    StrongRealClassification, StructureError, WeakRealFormId, Weight, WeylAction, WeylElement,
    WeylInterface,
};

use crate::diagnostic::{Diagnostic, ErrorKind, SourceSpan};
use crate::value::{Matrix, RatVec, Value, Vec32};

/// Upstream Lie-type letter bounds (atlas-types.w:165-211) and RANK_MAX.
const RANK_MAX: usize = 32;

const INTEGER_BUDGET: IntegerLatticeBudget =
    IntegerLatticeBudget::new(64, 1_000_000, 1_000_000, 256);
/// Covers |W| up to E6 (51,840); larger groups need budget control first.
const WEYL_BUDGET: usize = 4_000_000; // E7's Weyl group has 2,903,040 elements
const FIBER_BUDGET: usize = 1 << 20;
const ROOT_BUDGET: usize = 4_096;

/// One simple or torus factor of a Lie type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LieTypeValue {
    factors: Vec<(char, usize)>,
}

impl LieTypeValue {
    fn total_rank(&self) -> usize {
        self.factors.iter().map(|&(_, rank)| rank).sum()
    }

    fn semisimple_factors(&self) -> impl Iterator<Item = (char, usize)> + '_ {
        self.factors
            .iter()
            .copied()
            .filter(|&(letter, _)| letter != 'T')
    }

    fn add_simple_factor(&mut self, letter: char, rank: usize) {
        self.factors.push((letter, rank));
    }

    fn render(&self) -> String {
        if self.factors.is_empty() {
            return String::new();
        }
        self.factors
            .iter()
            .map(|(letter, rank)| format!("{letter}{rank}"))
            .collect::<Vec<_>>()
            .join(".")
    }
}

/// A root datum handle with its construction provenance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RootDatumHandle {
    datum: Arc<BasedRootDatum>,
    lie_type: LieTypeValue,
    isogeny: DatumIsogeny,
    prefers_coroots: bool,
}

impl RootDatumHandle {
    pub(crate) fn lie_type(&self) -> &LieTypeValue {
        &self.lie_type
    }

    pub(crate) fn prefers_coroots(&self) -> bool {
        self.prefers_coroots
    }

    fn description(&self) -> String {
        let prefix = self
            .isogeny
            .label()
            .map(|label| format!("{label} "))
            .unwrap_or_default();
        if self.lie_type.factors.is_empty() {
            format!("{prefix}root datum of empty Lie type")
        } else {
            format!(
                "{prefix}root datum of Lie type '{}'",
                self.lie_type.render()
            )
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DatumIsogeny {
    SimplyConnected,
    Adjoint,
    Both,
    Other,
}

impl DatumIsogeny {
    fn label(self) -> Option<&'static str> {
        match self {
            Self::SimplyConnected => Some("simply connected"),
            Self::Adjoint => Some("adjoint"),
            Self::Both => Some("simply connected adjoint"),
            Self::Other => None,
        }
    }
}

#[cfg(test)]
#[derive(Clone)]
struct CanonicalBuildTestGate {
    reached: std::sync::mpsc::Sender<()>,
    release: Arc<Mutex<std::sync::mpsc::Receiver<()>>>,
}

#[cfg(test)]
impl fmt::Debug for CanonicalBuildTestGate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("CanonicalBuildTestGate")
    }
}

#[cfg(test)]
impl CanonicalBuildTestGate {
    fn wait(&self) -> Result<(), &'static str> {
        use std::time::Duration;

        self.reached
            .send(())
            .map_err(|_| "canonical real form test gate disconnected")?;
        self.release
            .lock()
            .map_err(|_| "canonical real form test gate poisoned")?
            .recv_timeout(Duration::from_secs(5))
            .map_err(|_| "canonical real form test gate timed out")
    }
}

/// The per-inner-class pipeline shared by every real form of the class.
#[derive(Debug)]
pub struct InnerClassContext {
    root_datum: RootDatumHandle,
    inner_class: InnerClass,
    classification: std::sync::Arc<CartanClassification>,
    strong: StrongRealClassification,
    order: ExternalFormOrder,
    layout: InnerClassLayout,
    dual_form_count: usize,
    /// Per Cartan class (crate Cartan order): the corresponding dual
    /// inner-class Cartan and its weak-real-form count, the upstream
    /// `numDualRealForms` of the class's dual fiber.
    dual_cartans: Vec<(CartanId, usize)>,
    forms: Vec<RealFormPresentation>,
    /// Upstream memoizes each canonical real-form owner inside its inner
    /// class. Weak entries avoid a parent -> form -> parent ownership cycle.
    canonical_forms: Mutex<Vec<Weak<RealFormContext>>>,
    #[cfg(test)]
    canonical_build_test_gate: Mutex<Option<CanonicalBuildTestGate>>,
}

/// One real form's frozen pipeline: seed, completed table, and KGB graph.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct FullDeformKey {
    x: KgbId,
    y_bits: ModTwoVector,
    gamma: RationalWeight,
}

#[derive(Debug, Default)]
struct DeformationCache {
    values: HashMap<FullDeformKey, Vec<(SplitValue, KType)>>,
}

#[derive(Debug)]
pub struct RealFormContext {
    parent: Arc<InnerClassContext>,
    external: usize,
    internal: WeakRealFormId,
    table: Arc<InvolutionTable>,
    graph: Arc<KgbGraph>,
    rep: Arc<RepTableOwner>,
    full_deform_cache: Mutex<DeformationCache>,
    twisted_full_deform_cache: Mutex<DeformationCache>,
}

/// A Block value: the owning real form and dual real form contexts with
/// the fibred-product graph (upstream `Block_value`,
/// interpreter/atlas-types.w:4748-4764). The graph is boxed so the
/// `DomainValue` variants stay one pointer wide.
#[derive(Clone, Debug)]
pub struct BlockValue {
    rf: Arc<RealFormContext>,
    dual_rf: Arc<RealFormContext>,
    graph: Box<BlockGraph>,
}

/// The Weyl side of one root datum: the enumerated semisimple root system
/// the word-level kernel operates on, plus the internal generator
/// renumbering that fixes the upstream canonical-word choice.
#[derive(Debug)]
pub struct WeylEltContext {
    handle: RootDatumHandle,
    system: RootSystem,
    interface: WeylInterface,
}

/// A WeylElt value: the element with its construction context and its
/// canonical reduced word, frozen at construction so Display and `word`
/// are pure reads.
#[derive(Clone, Debug)]
pub struct WeylEltValue {
    context: Arc<WeylEltContext>,
    element: WeylElement,
    word: Vec<usize>,
}

/// An Atlas `Split`: a dual number e + f*s with s²=1, stored as the
/// (e, f) pair. The arithmetic matches upstream `Split_integer`
/// (utilities/arithmetic.h:152-213) on its machine-int representation:
/// componentwise sum and difference, negation, and the dual product
/// (e1e2+f1f2, e1f2+f1e2). Machine-width overflow wraps, as upstream's
/// `int` arithmetic does in practice.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SplitValue {
    e: i32,
    f: i32,
}

impl SplitValue {
    fn new(e: i32, f: i32) -> Self {
        Self { e, f }
    }

    pub fn e(&self) -> i32 {
        self.e
    }

    pub fn f(&self) -> i32 {
        self.f
    }

    fn is_zero(&self) -> bool {
        self.e == 0 && self.f == 0
    }

    fn add(self, other: Self) -> Self {
        Self::new(self.e.wrapping_add(other.e), self.f.wrapping_add(other.f))
    }

    fn sub(self, other: Self) -> Self {
        Self::new(self.e.wrapping_sub(other.e), self.f.wrapping_sub(other.f))
    }

    fn neg(self) -> Self {
        Self::new(self.e.wrapping_neg(), self.f.wrapping_neg())
    }

    fn mul(self, other: Self) -> Self {
        Self::new(
            self.e
                .wrapping_mul(other.e)
                .wrapping_add(self.f.wrapping_mul(other.f)),
            self.e
                .wrapping_mul(other.f)
                .wrapping_add(self.f.wrapping_mul(other.e)),
        )
    }
}

/// Whether a Split-scaled product keeps the term whose coefficient is
/// `term` (split_mult_K_type_pol_wrapper, atlas-types.w:5868-5900): a
/// zero-divisor scalar (a multiple of 1-s or 1+s) kills the terms whose
/// evaluation at the annihilated point is zero; any other scalar keeps
/// every term.
fn split_keeps(term: &SplitValue, scalar: SplitValue) -> bool {
    let term_at_one = term.e() + term.f();
    let term_at_minus_one = term.e() - term.f();
    if scalar.e() + scalar.f() == 0 {
        term_at_minus_one != 0
    } else if scalar.e() - scalar.f() == 0 {
        term_at_one != 0
    } else {
        true
    }
}

impl fmt::Display for SplitValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // arithmetic::print_split (io/basic_io.cpp:150-154): the sign
        // folds into the separator and the s-component prints unsigned.
        write!(
            formatter,
            "({}{}{}s)",
            self.e,
            if self.f < 0 { '-' } else { '+' },
            self.f.unsigned_abs(),
        )
    }
}

/// A K-type value: the crate `KType` plus the real form that owns it
/// (upstream `K_type_value`, interpreter/atlas-types.w:5175-5192).
#[derive(Clone, Debug)]
pub struct KTypeValue {
    context: Arc<RealFormContext>,
    ktype: KType,
}

/// A standard-module parameter value: the crate `StandardRepr` plus the
/// real form that owns it (upstream `module_parameter_value`).
#[derive(Clone, Debug)]
pub struct ParamValue {
    context: Arc<RealFormContext>,
    repr: StandardRepr,
}

/// A K-type polynomial: ordered `(Split, KType)` terms over one real form
/// (upstream `K_type_pol`, gkmod/K_repr.h). Adding a like term merges the
/// Split coefficient; a zero coefficient removes the term.
#[derive(Clone, Debug)]
pub struct KTypePolValue {
    rf: Arc<RealFormContext>,
    terms: Vec<(SplitValue, KType)>,
}

/// A virtual module: ordered `(Split, StandardRepr)` terms over one real
/// form (upstream `SR_poly`, gkmod/repr.h). Like terms merge and zero
/// coefficients drop, matching `SR_poly::add_term`.
#[derive(Clone, Debug)]
pub struct ParamPolValue {
    rf: Arc<RealFormContext>,
    terms: Vec<(SplitValue, StandardRepr)>,
}

/// The domain payload of [`Value::Domain`]. Equality is STRUCTURAL: two
/// independently constructed handles for the same mathematical object
/// compare equal, matching upstream's memoized handles.
#[derive(Clone, Debug)]
pub enum DomainValue {
    LieType(LieTypeValue),
    RootDatum(RootDatumHandle),
    InnerClass(Arc<InnerClassContext>),
    RealForm(Arc<RealFormContext>),
    KgbElement(Arc<RealFormContext>, KgbId),
    Block(BlockValue),
    WeylElement(WeylEltValue),
    CartanClass(Arc<InnerClassContext>, CartanId),
    Split(SplitValue),
    KType(KTypeValue),
    KTypePol(KTypePolValue),
    Param(ParamValue),
    ParamPol(ParamPolValue),
}

impl PartialEq for DomainValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::LieType(left), Self::LieType(right)) => left == right,
            (Self::RootDatum(left), Self::RootDatum(right)) => left == right,
            (Self::InnerClass(left), Self::InnerClass(right)) => {
                left.inner_class == right.inner_class
            }
            (Self::RealForm(left), Self::RealForm(right)) => same_real_form(left, right),
            (Self::KgbElement(left, left_id), Self::KgbElement(right, right_id)) => {
                left.parent.inner_class == right.parent.inner_class
                    && left.internal == right.internal
                    && left_id == right_id
            }
            // Structural handle equality on the two owning forms, in the
            // same scheme as the RealForm arm above.
            (Self::Block(left), Self::Block(right)) => {
                left.rf.parent.inner_class == right.rf.parent.inner_class
                    && left.rf.internal == right.rf.internal
                    && left.dual_rf.parent.inner_class == right.dual_rf.parent.inner_class
                    && left.dual_rf.internal == right.dual_rf.internal
            }
            // Group equality on the canonical root-permutation
            // representation: braid-equivalent words compare equal.
            (Self::WeylElement(left), Self::WeylElement(right)) => {
                left.context.handle == right.context.handle && left.element == right.element
            }
            (Self::CartanClass(left, left_id), Self::CartanClass(right, right_id)) => {
                left.inner_class == right.inner_class && left_id == right_id
            }
            (Self::Split(left), Self::Split(right)) => left == right,
            // K_type_value::operator== (atlas-types.w:5310-5316): the
            // owning real form and the strict KType components.
            (Self::KType(left), Self::KType(right)) => {
                same_real_form(&left.context, &right.context) && left.ktype == right.ktype
            }
            // module_parameter_value::operator==
            // (atlas-types.w:6344-6350): the owning real form and the
            // strict StandardRepr components (height is derived, excluded).
            (Self::Param(left), Self::Param(right)) => {
                same_real_form(&left.context, &right.context) && left.repr == right.repr
            }
            // K_type_pol/SR_poly equality: same owning form and identical
            // ordered term lists (atlas-types.w:5549-5568, 7731-7748).
            (Self::KTypePol(left), Self::KTypePol(right)) => {
                same_real_form(&left.rf, &right.rf) && left.terms == right.terms
            }
            (Self::ParamPol(left), Self::ParamPol(right)) => {
                same_real_form(&left.rf, &right.rf) && left.terms == right.terms
            }
            _ => false,
        }
    }
}

impl Eq for DomainValue {}

/// The owning-form identity of a [`RealFormContext`], matching the
/// `RealReductiveGroup operator==` (realredgp.h:142-149): same inner
/// class and form, same base cocharacter, same initial torus part — the
/// custom-seed identity.
fn same_real_form(left: &RealFormContext, right: &RealFormContext) -> bool {
    left.parent.inner_class == right.parent.inner_class
        && left.internal == right.internal
        && left.graph.cocharacter() == right.graph.cocharacter()
        && left.graph.seed_element().torus_bits() == right.graph.seed_element().torus_bits()
}

/// Pointer-owner equality used by wrappers that compare their
/// `shared_real_form` fields directly. Canonical construction shares one
/// `RepTableOwner` through the parent weak cache; custom construction always
/// creates a fresh owner even when the mathematical real forms are equal.
fn same_real_form_owner(left: &RealFormContext, right: &RealFormContext) -> bool {
    Arc::ptr_eq(&left.rep, &right.rep)
}

impl fmt::Display for DomainValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LieType(value) => {
                if value.factors.is_empty() {
                    write!(formatter, "empty Lie type")
                } else {
                    write!(formatter, "Lie type '{}'", value.render())
                }
            }
            Self::RootDatum(handle) => write!(formatter, "{}", handle.description()),
            Self::InnerClass(context) => {
                // inner_class_value::print (interpreter/atlas-types.w:3164-3172).
                let real = context.order.form_count();
                let dual = context.dual_form_count;
                write!(
                    formatter,
                    "Complex reductive group of type {}, with involution defining\n\
                     inner class of type '{}', with {} real {} and {} dual real {}",
                    context.layout.lie_type_string(),
                    context.layout.inner_class_string(),
                    real,
                    if real == 1 { "form" } else { "forms" },
                    dual,
                    if dual == 1 { "form" } else { "forms" },
                )
            }
            Self::RealForm(context) => {
                // real_form_value::print (interpreter/atlas-types.w:3566-3575).
                let presentation = &context.parent.forms[context.external];
                if presentation.compact {
                    write!(formatter, "compact ")?;
                }
                write!(
                    formatter,
                    "{} ",
                    if presentation.connected {
                        "connected"
                    } else {
                        "disconnected"
                    }
                )?;
                if presentation.quasisplit {
                    write!(
                        formatter,
                        "{}split ",
                        if presentation.split { "" } else { "quasi" }
                    )?;
                }
                write!(
                    formatter,
                    "real group with Lie algebra '{}'",
                    presentation.name
                )
            }
            Self::KgbElement(_, id) => write!(formatter, "KGB element #{}", id.index()),
            // Block_value::print (interpreter/atlas-types.w:4774-4775).
            Self::Block(value) => write!(formatter, "Block of {} elements", value.graph.size()),
            // W_elt_value::print (interpreter/atlas-types.w:2326-2333):
            // the canonical reduced word, dot-separated in angle brackets.
            Self::WeylElement(value) => {
                write!(formatter, "<")?;
                for (index, generator) in value.word.iter().enumerate() {
                    if index > 0 {
                        write!(formatter, ".")?;
                    }
                    write!(formatter, "{generator}")?;
                }
                write!(formatter, ">")
            }
            // Cartan_class_value::print (interpreter/atlas-types.w:4007-4016):
            // the class number with its real-form and dual-real-form counts —
            // the weak-real partition sizes of the class's fiber and of the
            // corresponding dual class's fiber.
            Self::CartanClass(context, id) => {
                let number =
                    cartan_number(context, *id).expect("CartanClass values carry an in-range id");
                let real = context
                    .classification
                    .cartan_class(*id)
                    .expect("CartanClass values carry an in-range id")
                    .partition()
                    .class_count();
                let dual = context
                    .dual_cartans
                    .get(number)
                    .expect("the correspondence covers every Cartan class")
                    .1;
                write!(
                    formatter,
                    "Cartan class #{number}, occurring for {real} real {} \
                     and for {dual} dual real {}",
                    if real == 1 { "form" } else { "forms" },
                    if dual == 1 { "form" } else { "forms" },
                )
            }
            Self::Split(value) => write!(formatter, "{value}"),
            // K_type_value::print (atlas-types.w:5212-5215): the adjective
            // chain, ` K-type`, then print_K_type (basic_io.cpp:158-163)
            // whose own leading space gives `final K-type K_type(...)`.
            Self::KType(value) => {
                let rc = rep_context(&value.context);
                let adjectives = ktype_adjective(&rc, &value.ktype);
                write!(
                    formatter,
                    "{adjectives} K-type K_type(x={}, lambda={})",
                    value.ktype.x().index(),
                    rational_weight_display(
                        &rc.lambda_of_ktype(&value.ktype)
                            .expect("KType lambda is computable"),
                    ),
                )
            }
            // module_parameter_value::print (atlas-types.w:6183-6185): the
            // adjective chain, a space, then print_stdrep
            // (basic_io.cpp:203-207).
            Self::Param(value) => {
                if value.repr.is_undefined() {
                    return match value.repr.undefined_print_weights() {
                        Some((lambda, nu)) => write!(
                            formatter,
                            "final parameter(x={},lambda={},nu={})",
                            value.repr.x().index(),
                            rational_weight_display(lambda),
                            rational_weight_display(nu),
                        ),
                        None => write!(
                            formatter,
                            "final parameter(x={},lambda=<unavailable>,nu=<unavailable>)",
                            value.repr.x().index(),
                        ),
                    };
                }
                let rc = rep_context(&value.context);
                let adjectives = repr_adjective(&rc, &value.repr);
                write!(
                    formatter,
                    "{adjectives} parameter(x={},lambda={},nu={})",
                    value.repr.x().index(),
                    rational_weight_display(
                        &rc.lambda(&value.repr)
                            .expect("parameter lambda is computable"),
                    ),
                    rational_weight_display(
                        &rc.nu(&value.repr).expect("parameter nu is computable"),
                    ),
                )
            }
            Self::KTypePol(value) => write!(formatter, "{}", ktype_pol_display(value)),
            Self::ParamPol(value) => write!(formatter, "{}", param_pol_display(value)),
        }
    }
}

/// Borrow the representation context of a real form's frozen pipeline from
/// its shared owner. The owner keeps the table, graph, derived invariants, and
/// future common-block cache on the same lifetime boundary.
fn rep_context(context: &RealFormContext) -> RepContext<'_> {
    context.rep.context()
}

/// The 6-way adjective chain for a K-type (atlas-types.w:5228-5235).
fn ktype_adjective(rc: &RepContext<'_>, value: &KType) -> &'static str {
    if !value
        .is_standard(rc)
        .expect("K-type predicate is computable")
    {
        "non-standard"
    } else if !value
        .is_dominant(rc)
        .expect("K-type predicate is computable")
    {
        "non-dominant"
    } else if !value
        .is_nonzero(rc)
        .expect("K-type predicate is computable")
    {
        "zero"
    } else if !value
        .is_semifinal(rc)
        .expect("K-type predicate is computable")
    {
        "non-final"
    } else if !value.is_normal(rc).expect("K-type predicate is computable") {
        "non-normal"
    } else {
        "final"
    }
}

/// The 6-way adjective chain for a module parameter
/// (atlas-types.w:6199-6206).
fn repr_adjective(rc: &RepContext<'_>, value: &StandardRepr) -> &'static str {
    if !value
        .is_standard(rc)
        .expect("parameter predicate is computable")
    {
        "non-standard"
    } else if !value
        .is_dominant(rc)
        .expect("parameter predicate is computable")
    {
        "non-dominant"
    } else if !value
        .is_nonzero(rc)
        .expect("parameter predicate is computable")
    {
        "zero"
    } else if !value
        .is_semifinal(rc)
        .expect("parameter predicate is computable")
    {
        "non-final"
    } else if !value
        .is_normal(rc)
        .expect("parameter predicate is computable")
    {
        "non-normal"
    } else {
        "final"
    }
}

/// The ratvec `seqPrint` of a crate rational weight: bracket-enclosed,
/// comma-separated numerators then `/denominator`, no inner spaces
/// (basic_io.cpp:124-132, matrix::seqPrint). Used by print_K_type and
/// print_stdrep, in contrast with the language RatVec display `[ n ]/d`.
fn rational_weight_display(weight: &RationalWeight) -> String {
    let numerator = weight
        .numerator()
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",");
    format!("[{numerator}]/{}", weight.denominator())
}

/// print_K_type_pol (basic_io.cpp:165-202): `Empty sum of K-types` for the
/// zero polynomial, otherwise one `\n`-prefixed term per entry — the
/// coefficient embellishment (full `(e+fs)` only when both components occur
/// somewhere across the terms), then `*` + print_K_type (whose own leading
/// space follows the `*`) + ` [height]`.
fn ktype_pol_display(value: &KTypePolValue) -> String {
    if value.terms.is_empty() {
        return "Empty sum of K-types".to_string();
    }
    let has_one = value
        .terms
        .iter()
        .any(|(coefficient, _)| coefficient.e() != 0);
    let has_s = value
        .terms
        .iter()
        .any(|(coefficient, _)| coefficient.f() != 0);
    let rc = rep_context(&value.rf);
    let mut out = String::new();
    for (coefficient, ktype) in &value.terms {
        out.push('\n');
        if has_one && has_s {
            out.push_str(&format!(
                "({}{}{}s)",
                coefficient.e(),
                if coefficient.f() < 0 { '-' } else { '+' },
                coefficient.f().unsigned_abs(),
            ));
        } else if has_one {
            out.push_str(&coefficient.e().to_string());
        } else {
            out.push_str(&format!("{}s", coefficient.f()));
        }
        out.push('*');
        out.push_str(&format!(
            " K_type(x={}, lambda={})",
            ktype.x().index(),
            rational_weight_display(
                &rc.lambda_of_ktype(ktype)
                    .expect("KType lambda is computable"),
            ),
        ));
        out.push_str(&format!(" [{}]", ktype.height()));
    }
    out
}

/// print_SR_poly (basic_io.cpp:214-244): like print_K_type_pol, but the
/// parameter text (`parameter(...)`, no leading space) follows the `*`
/// directly, and the empty text is `Empty sum of standard modules`.
fn param_pol_display(value: &ParamPolValue) -> String {
    if value.terms.is_empty() {
        return "Empty sum of standard modules".to_string();
    }
    let has_one = value
        .terms
        .iter()
        .any(|(coefficient, _)| coefficient.e() != 0);
    let has_s = value
        .terms
        .iter()
        .any(|(coefficient, _)| coefficient.f() != 0);
    let rc = rep_context(&value.rf);
    let mut out = String::new();
    for (coefficient, repr) in &value.terms {
        out.push('\n');
        if has_one && has_s {
            out.push_str(&format!(
                "({}{}{}s)",
                coefficient.e(),
                if coefficient.f() < 0 { '-' } else { '+' },
                coefficient.f().unsigned_abs(),
            ));
        } else if has_one {
            out.push_str(&coefficient.e().to_string());
        } else {
            out.push_str(&format!("{}s", coefficient.f()));
        }
        out.push('*');
        out.push_str(&format!(
            "parameter(x={},lambda={},nu={})",
            repr.x().index(),
            rational_weight_display(&rc.lambda(repr).expect("parameter lambda is computable"),),
            rational_weight_display(&rc.nu(repr).expect("parameter nu is computable")),
        ));
        out.push_str(&format!(" [{}]", repr.height()));
    }
    out
}

/// The language-facing kind name, used by diagnostics and type printing.
pub fn kind_name(value: &DomainValue) -> &'static str {
    match value {
        DomainValue::LieType(_) => "LieType",
        DomainValue::RootDatum(_) => "RootDatum",
        DomainValue::InnerClass(_) => "InnerClass",
        DomainValue::RealForm(_) => "RealForm",
        DomainValue::KgbElement(_, _) => "KGBElt",
        DomainValue::Block(_) => "Block",
        DomainValue::WeylElement(_) => "WeylElt",
        DomainValue::CartanClass(_, _) => "CartanClass",
        DomainValue::Split(_) => "Split",
        DomainValue::KType(_) => "KType",
        DomainValue::KTypePol(_) => "KTypePol",
        DomainValue::Param(_) => "Param",
        DomainValue::ParamPol(_) => "ParamPol",
    }
}

fn runtime(span: SourceSpan, message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(ErrorKind::Runtime, message.into(), Some(span))
}

fn type_error(span: SourceSpan, message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(ErrorKind::Type, message.into(), Some(span))
}

/// Upstream Lie-type string parser (atlas-types.w:222-247): repeat
/// { skip punctuation/whitespace; one letter of "ABCDEFGT"; unsigned
/// decimal }; per-letter rank bounds; `Tr` becomes r copies of `T1`.
fn parse_lie_type(text: &str, span: SourceSpan) -> Result<LieTypeValue, Diagnostic> {
    let bad = || {
        runtime(
            span,
            format!("Error in string '{text}' that should specify a Lie type"),
        )
    };
    let mut factors = Vec::new();
    let mut characters = text.chars().peekable();
    loop {
        while matches!(characters.peek(), Some(c) if c.is_ascii_punctuation() || c.is_whitespace())
        {
            characters.next();
        }
        let Some(letter) = characters.next() else {
            break;
        };
        if !"ABCDEFGT".contains(letter) {
            return Err(bad());
        }
        let mut digits = String::new();
        while matches!(characters.peek(), Some(c) if c.is_ascii_digit()) {
            digits.push(characters.next().expect("peeked digit"));
        }
        let rank: usize = digits.parse().map_err(|_| bad())?;
        let (lower, upper) = match letter {
            'A' => (1, RANK_MAX),
            'B' | 'C' => (2, RANK_MAX),
            'D' => (4, RANK_MAX),
            'E' => (6, 8),
            'F' => (4, 4),
            'G' => (2, 2),
            'T' => (0, RANK_MAX),
            _ => unreachable!(),
        };
        if rank < lower || rank > upper {
            return Err(bad());
        }
        if letter == 'T' {
            for _ in 0..rank {
                factors.push(('T', 1));
            }
        } else {
            factors.push((letter, rank));
        }
    }
    let total: usize = factors.iter().map(|&(_, rank)| rank).sum();
    if total > RANK_MAX {
        return Err(bad());
    }
    Ok(LieTypeValue { factors })
}

/// Upstream `lietype.cpp` Cartan-entry convention for one simple factor.
fn factor_cartan(letter: char, rank: usize) -> Vec<Vec<i32>> {
    let mut matrix = vec![vec![0; rank]; rank];
    for (index, row) in matrix.iter_mut().enumerate() {
        row[index] = 2;
    }
    let link = |a: usize, b: usize, upper: i32, lower: i32, matrix: &mut Vec<Vec<i32>>| {
        matrix[a][b] = upper;
        matrix[b][a] = lower;
    };
    match letter {
        'A' => {
            for i in 0..rank - 1 {
                link(i, i + 1, -1, -1, &mut matrix);
            }
        }
        'B' => {
            for i in 0..rank - 1 {
                let (upper, lower) = if i == rank - 2 { (-2, -1) } else { (-1, -1) };
                link(i, i + 1, upper, lower, &mut matrix);
            }
        }
        'C' => {
            for i in 0..rank - 1 {
                let (upper, lower) = if i == rank - 2 { (-1, -2) } else { (-1, -1) };
                link(i, i + 1, upper, lower, &mut matrix);
            }
        }
        'D' => {
            for i in 0..rank - 3 {
                link(i, i + 1, -1, -1, &mut matrix);
            }
            link(rank - 3, rank - 2, -1, -1, &mut matrix);
            link(rank - 3, rank - 1, -1, -1, &mut matrix);
        }
        'G' => link(0, 1, -1, -3, &mut matrix),
        'F' => {
            link(0, 1, -1, -1, &mut matrix);
            link(1, 2, -2, -1, &mut matrix);
            link(2, 3, -1, -1, &mut matrix);
        }
        'E' => {
            link(0, 2, -1, -1, &mut matrix);
            for i in 1..rank - 1 {
                if i == 1 {
                    link(1, 3, -1, -1, &mut matrix);
                } else {
                    link(i, i + 1, -1, -1, &mut matrix);
                }
            }
        }
        _ => unreachable!("validated letter"),
    }
    matrix
}

/// Block-diagonal Cartan matrix over the semisimple factors.
fn block_cartan(lie_type: &LieTypeValue) -> Vec<Vec<i32>> {
    let semisimple: usize = lie_type.semisimple_factors().map(|(_, rank)| rank).sum();
    let mut matrix = vec![vec![0; semisimple]; semisimple];
    let mut offset = 0;
    for (letter, rank) in lie_type.semisimple_factors() {
        let block = factor_cartan(letter, rank);
        for (i, row) in block.iter().enumerate() {
            for (j, &value) in row.iter().enumerate() {
                matrix[offset + i][offset + j] = value;
            }
        }
        offset += rank;
    }
    matrix
}

fn relation_diagnostic(error: RelationError, span: SourceSpan) -> Diagnostic {
    let message = match error {
        RelationError::Structure(error) => error.to_string(),
        RelationError::TooManyFactors { factors, columns } => {
            format!("Too many factors: {factors} for {columns} columns")
        }
        RelationError::ColumnLengthsDoNotMatch => "Column lengths do not match".to_string(),
        RelationError::NotEnoughReplacementColumns => "Not enough replacement columns".to_string(),
        RelationError::TooManyReplacementColumns => "Too many replacement columns".to_string(),
        RelationError::GeneratorLengthMismatch {
            generator,
            actual,
            expected,
        } => format!("Length mismatch for generator {generator}: {actual}:{expected}"),
        RelationError::ImproperGeneratorEntry {
            numerator,
            denominator,
            factor,
        } => format!(
            "Improper generator entry: {numerator}/{denominator} not a multiple of 1/{factor}"
        ),
        RelationError::IntegerOutOfRange => {
            "Integer value too big for matrix conversion".to_string()
        }
    };
    runtime(span, message)
}

fn relation_matrix(matrix: &Matrix, span: SourceSpan) -> Result<RelationMatrix, Diagnostic> {
    let rows = matrix.rows();
    let columns = matrix.cols();
    let entries =
        (0..rows).flat_map(|row| (0..columns).filter_map(move |column| matrix.entry(row, column)));
    RelationMatrix::from_i32_iter(rows, columns, entries, &INTEGER_BUDGET)
        .map_err(|error| relation_diagnostic(error, span))
}

fn relation_value(matrix: &RelationMatrix, span: SourceSpan) -> Result<Value, Diagnostic> {
    let rows = matrix.rows();
    let columns = matrix.columns();
    let entry_count = RelationMatrix::preflight_shape(rows, columns, &INTEGER_BUDGET)
        .map_err(|error| relation_diagnostic(error, span))?;
    let row_entries = matrix
        .try_i32_rows()
        .map_err(|error| relation_diagnostic(error, span))?;
    let mut entries = Vec::new();
    entries.try_reserve_exact(entry_count).map_err(|_| {
        relation_diagnostic(
            RelationError::Structure(StructureError::AllocationFailed {
                requested: entry_count,
            }),
            span,
        )
    })?;
    for column in 0..columns {
        for row in &row_entries {
            entries.push(row[column]);
        }
    }
    Matrix::from_columns(rows, columns, entries)
        .map(Value::Matrix)
        .ok_or_else(|| runtime(span, "Invalid matrix dimensions"))
}

fn smith_cartan(
    lie_type: &LieTypeValue,
    span: SourceSpan,
) -> Result<(RelationMatrix, Vec<i32>), Diagnostic> {
    let rank = lie_type.total_rank();
    let entry_count = RelationMatrix::preflight_shape(rank, rank, &INTEGER_BUDGET)
        .map_err(|error| relation_diagnostic(error, span))?;
    let mut basis_entries = Vec::new();
    basis_entries.try_reserve_exact(entry_count).map_err(|_| {
        relation_diagnostic(
            RelationError::Structure(StructureError::AllocationFailed {
                requested: entry_count,
            }),
            span,
        )
    })?;
    basis_entries.resize(entry_count, 0);
    let mut factors = Vec::new();
    factors.try_reserve_exact(rank).map_err(|_| {
        relation_diagnostic(
            RelationError::Structure(StructureError::AllocationFailed { requested: rank }),
            span,
        )
    })?;

    let mut offset = 0;
    for &(letter, factor_rank) in &lie_type.factors {
        if letter == 'T' {
            for index in 0..factor_rank {
                basis_entries[(offset + index) * rank + offset + index] = 1;
                factors.push(0);
            }
        } else {
            let cartan = factor_cartan(letter, factor_rank);
            let mut transposed = Vec::new();
            transposed.try_reserve_exact(factor_rank).map_err(|_| {
                relation_diagnostic(
                    RelationError::Structure(StructureError::AllocationFailed {
                        requested: factor_rank,
                    }),
                    span,
                )
            })?;
            for (row, _) in cartan.iter().enumerate() {
                let mut entries = Vec::new();
                entries.try_reserve_exact(factor_rank).map_err(|_| {
                    relation_diagnostic(
                        RelationError::Structure(StructureError::AllocationFailed {
                            requested: factor_rank,
                        }),
                        span,
                    )
                })?;
                entries.extend((0..factor_rank).map(|column| cartan[column][row]));
                transposed.push(entries);
            }
            let adapted = adapted_relation_basis(&transposed, &INTEGER_BUDGET)
                .map_err(|error| relation_diagnostic(error, span))?;
            let (block, block_factors) = adapted.into_parts();
            let mut block_rows = block
                .try_i32_rows()
                .map_err(|error| relation_diagnostic(error, span))?;
            if letter == 'D' && factor_rank % 2 == 0 {
                for row in &mut block_rows {
                    row[factor_rank - 2] = row[factor_rank - 2]
                        .checked_add(row[factor_rank - 1])
                        .ok_or_else(|| {
                        relation_diagnostic(RelationError::IntegerOutOfRange, span)
                    })?;
                }
            }
            if block_factors.len() != factor_rank {
                return Err(runtime(span, "Cartan matrix reduction lost rank"));
            }
            for row in 0..factor_rank {
                for column in 0..factor_rank {
                    basis_entries[(offset + row) * rank + offset + column] =
                        block_rows[row][column];
                }
            }
            factors.extend(block_factors);
        }
        offset += factor_rank;
    }

    let basis = RelationMatrix::from_i32_entries(rank, rank, &basis_entries, &INTEGER_BUDGET)
        .map_err(|error| relation_diagnostic(error, span))?;
    Ok((basis, factors))
}

fn filter_relation_units_adapter(
    basis: &Matrix,
    factors: &[i32],
    span: SourceSpan,
) -> Result<(RelationMatrix, Vec<i32>), Diagnostic> {
    let basis = relation_matrix(basis, span)?;
    domain_filter_relation_units(&basis, factors, &INTEGER_BUDGET)
        .map(RelationBasis::into_parts)
        .map_err(|error| relation_diagnostic(error, span))
}

fn replace_relation_generators_adapter(
    basis: &Matrix,
    factors: &[i32],
    replacements: &Matrix,
    span: SourceSpan,
) -> Result<RelationMatrix, Diagnostic> {
    let basis = relation_matrix(basis, span)?;
    let replacements = relation_matrix(replacements, span)?;
    domain_replace_relation_generators(&basis, factors, &replacements, &INTEGER_BUDGET)
        .map_err(|error| relation_diagnostic(error, span))
}

fn smith_value(lie_type: &LieTypeValue, span: SourceSpan) -> Result<Value, Diagnostic> {
    let (basis, factors) = smith_cartan(lie_type, span)?;
    Ok(Value::Tuple(vec![
        relation_value(&basis, span)?,
        Value::Vector(Vec32(factors)),
    ]))
}

fn relation_pair<'a>(
    name: &str,
    arguments: &'a [Value],
    span: SourceSpan,
) -> Result<(&'a Matrix, &'a [i32]), Diagnostic> {
    let (first, second) = match arguments {
        [Value::Tuple(pair)] if pair.len() == 2 => (&pair[0], &pair[1]),
        [first, second] => (first, second),
        _ => return Err(runtime(span, format!("{name} expects 2 argument(s)"))),
    };
    let (Value::Matrix(matrix), Value::Vector(Vec32(factors))) = (first, second) else {
        return Err(type_error(span, format!("{name} expects (mat,vec)")));
    };
    Ok((matrix, factors))
}

fn quotient_relation_basis_adapter(
    lie_type: &LieTypeValue,
    generators: &[Value],
    span: SourceSpan,
) -> Result<Value, Diagnostic> {
    for generator in generators {
        let Value::RatVector(_) = generator else {
            return Err(type_error(span, "quotient_basis expects a row of ratvec"));
        };
    }
    let relation_generators = RelationGenerator::try_collect(
        lie_type.total_rank(),
        generators.iter().map(|generator| {
            let Value::RatVector(generator) = generator else {
                unreachable!("generator types were checked before budgeted collection")
            };
            let denominator = NonZeroU64::new(generator.denominator())
                .expect("RatVec maintains a nonzero denominator");
            RelationGenerator::new(generator.numerators(), denominator)
        }),
        &INTEGER_BUDGET,
    )
    .map_err(|error| relation_diagnostic(error, span))?;
    let (smith, factors) = smith_cartan(lie_type, span)?;
    let basis =
        domain_quotient_relation_basis(&smith, &factors, &relation_generators, &INTEGER_BUDGET)
            .map_err(|error| relation_diagnostic(error, span))?;
    relation_value(&basis, span)
}

fn build_datum(
    lie_type: &LieTypeValue,
    simply: bool,
    prefers_coroots: bool,
    span: SourceSpan,
) -> Result<RootDatumHandle, Diagnostic> {
    let cartan = block_cartan(lie_type);
    let semisimple = cartan.len();
    let lattice_rank = lie_type.total_rank();
    let pad = |mut coordinates: Vec<i32>| {
        coordinates.resize(lattice_rank, 0);
        coordinates
    };
    let (roots, coroots): (Vec<Weight>, Vec<Coweight>) = if simply {
        // Weight-lattice basis: roots are Cartan rows, coroots the basis.
        (0..semisimple)
            .map(|index| {
                let root = Weight::new(pad(cartan[index].clone()));
                let mut coordinates = vec![0; lattice_rank];
                coordinates[index] = 1;
                (root, Coweight::new(coordinates))
            })
            .unzip()
    } else {
        // Root-lattice basis: roots are the basis, coroots Cartan columns.
        (0..semisimple)
            .map(|index| {
                let mut coordinates = vec![0; lattice_rank];
                coordinates[index] = 1;
                let column: Vec<i32> = (0..semisimple).map(|row| cartan[row][index]).collect();
                (Weight::new(coordinates), Coweight::new(pad(column)))
            })
            .unzip()
    };
    let datum = BasedRootDatum::from_simple_data(lattice_rank, cartan, roots, coroots)
        .map_err(|error| runtime(span, error.to_string()))?;
    let isogeny = if lattice_rank != semisimple {
        DatumIsogeny::Other
    } else {
        classify_isogeny(&datum)
    };
    Ok(RootDatumHandle {
        datum: Arc::new(datum),
        lie_type: lie_type.clone(),
        isogeny,
        prefers_coroots,
    })
}

fn build_quotient_datum(
    lie_type: &LieTypeValue,
    lattice: &[Vec<i32>],
    prefers_coroots: bool,
    span: SourceSpan,
) -> Result<RootDatumHandle, Diagnostic> {
    let rank = lie_type.total_rank();
    if lattice.len() != rank || lattice.iter().any(|row| row.len() != rank) {
        return Err(runtime(
            span,
            format!("Sub-lattice matrix should have size {rank}x{rank}"),
        ));
    }
    let inverse = invert_integer_matrix(lattice)
        .ok_or_else(|| runtime(span, "Dependent lattice generators"))?;
    let simply_connected = build_datum(lie_type, true, prefers_coroots, span)?;

    build_quotient_from_handle(&simply_connected, lattice, inverse, span)
}

fn build_quotient_from_handle(
    source: &RootDatumHandle,
    lattice: &[Vec<i32>],
    inverse: Vec<Vec<BigRational>>,
    span: SourceSpan,
) -> Result<RootDatumHandle, Diagnostic> {
    let rank = source.datum.lattice_rank();
    if lattice.len() != rank || lattice.iter().any(|row| row.len() != rank) {
        return Err(runtime(
            span,
            format!("Sub-lattice matrix should have size {rank}x{rank}"),
        ));
    }

    let roots = source
        .datum
        .simple_roots()
        .iter()
        .map(|root| {
            inverse
                .iter()
                .map(|row| {
                    let coordinate = row.iter().zip(root.as_slice()).fold(
                        BigRational::from(0),
                        |sum, (coefficient, entry)| {
                            sum + coefficient.clone() * BigRational::from(*entry)
                        },
                    );
                    let integer = BigInt::try_from(coordinate).map_err(|_| {
                        runtime(span, "Sub-lattice does not contain the root lattice")
                    })?;
                    i32::try_from(&integer)
                        .map_err(|_| runtime(span, "Integer value to big for conversion"))
                })
                .collect::<Result<Vec<_>, _>>()
                .map(Weight::new)
        })
        .collect::<Result<Vec<_>, _>>()?;

    let coroots = source
        .datum
        .simple_coroots()
        .iter()
        .map(|coroot| {
            (0..rank)
                .map(|column| {
                    let coordinate = (0..rank).try_fold(0i128, |sum, row| {
                        sum.checked_add(
                            i128::from(lattice[row][column])
                                .checked_mul(i128::from(coroot.as_slice()[row]))?,
                        )
                    });
                    coordinate
                        .and_then(|value| i32::try_from(value).ok())
                        .ok_or_else(|| runtime(span, "Integer value to big for conversion"))
                })
                .collect::<Result<Vec<_>, _>>()
                .map(Coweight::new)
        })
        .collect::<Result<Vec<_>, _>>()?;

    let datum = BasedRootDatum::from_simple_data(
        rank,
        source.datum.cartan_matrix().to_vec(),
        roots,
        coroots,
    )
    .map_err(|error| runtime(span, error.to_string()))?;
    let isogeny = classify_isogeny(&datum);
    Ok(RootDatumHandle {
        datum: Arc::new(datum),
        lie_type: source.lie_type.clone(),
        isogeny,
        prefers_coroots: source.prefers_coroots,
    })
}

/// The based root datum dual to `handle`'s: transposed Cartan matrix,
/// simple roots and simple coroots interchanged, and the coroot preference
/// switched — the upstream `RootSystem(rs,DualTag)` (`prefer_co(not
/// rs.prefer_co)`, rootdata.cpp:341) behind `root_datum_value::dual`
/// (atlas-types.w:1147-1155). The stored Lie type dualizes letter-wise
/// (B<->C per factor, every other factor unchanged), which keeps torus
/// factors and factor order exactly as constructed; `Lie_type` of the dual
/// datum computes the same letters from the transposed Cartan matrix.
fn dual_root_datum(
    handle: &RootDatumHandle,
    span: SourceSpan,
) -> Result<RootDatumHandle, Diagnostic> {
    let datum = &*handle.datum;
    let dual_roots: Vec<Weight> = datum
        .simple_coroots()
        .iter()
        .map(|coroot| Weight::new(coroot.as_slice().to_vec()))
        .collect();
    let dual_coroots: Vec<Coweight> = datum
        .simple_roots()
        .iter()
        .map(|root| Coweight::new(root.as_slice().to_vec()))
        .collect();
    let dual = BasedRootDatum::from_simple_data(
        datum.lattice_rank(),
        transpose(datum.cartan_matrix()),
        dual_roots,
        dual_coroots,
    )
    .map_err(|error| runtime(span, error.to_string()))?;
    Ok(RootDatumHandle {
        isogeny: classify_isogeny(&dual),
        datum: Arc::new(dual),
        lie_type: dual_lie_type(&handle.lie_type),
        prefers_coroots: !handle.prefers_coroots,
    })
}

/// The Lie type of the dual datum: every B factor becomes the C factor of
/// the same rank and vice versa; all other factors are self-dual.
fn dual_lie_type(lie_type: &LieTypeValue) -> LieTypeValue {
    LieTypeValue {
        factors: lie_type
            .factors
            .iter()
            .map(|&(letter, rank)| {
                let dual_letter = match letter {
                    'B' => 'C',
                    'C' => 'B',
                    other => other,
                };
                (dual_letter, rank)
            })
            .collect(),
    }
}

/// Build a root datum from explicit simple-root and simple-coroot matrices.
/// Matrix columns are the basis vectors, matching Atlas's `mat` convention.
fn build_explicit_datum(
    simple_roots: &[Vec<i32>],
    simple_coroots: &[Vec<i32>],
    prefers_coroots: bool,
    span: SourceSpan,
) -> Result<RootDatumHandle, Diagnostic> {
    let lattice_rank = simple_roots.len();
    let semisimple_rank = simple_roots.first().map_or(0, Vec::len);
    if lattice_rank == 0 || semisimple_rank == 0 {
        return Err(runtime(
            span,
            "Implicit conversion to matrix for an empty set of vectors",
        ));
    }
    if simple_roots.iter().any(|row| row.len() != semisimple_rank)
        || simple_coroots.len() != lattice_rank
        || simple_coroots
            .iter()
            .any(|row| row.len() != semisimple_rank)
    {
        let root_shape = format!("{},{}", lattice_rank, semisimple_rank);
        let coroot_shape = format!(
            "{},{}",
            simple_coroots.len(),
            simple_coroots.first().map_or(0, Vec::len)
        );
        return Err(runtime(
            span,
            format!("Sizes ({root_shape}),({coroot_shape}) of simple (co)root systems differ"),
        ));
    }

    let roots = (0..semisimple_rank)
        .map(|column| {
            Weight::new(
                simple_roots
                    .iter()
                    .map(|row| row[column])
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();
    let coroots = (0..semisimple_rank)
        .map(|column| {
            Coweight::new(
                simple_coroots
                    .iter()
                    .map(|row| row[column])
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();
    let mut cartan = Vec::with_capacity(semisimple_rank);
    for root in &roots {
        let mut row = Vec::with_capacity(semisimple_rank);
        for coroot in &coroots {
            let value = root
                .as_slice()
                .iter()
                .zip(coroot.as_slice())
                .try_fold(0_i128, |sum, (&left, &right)| {
                    sum.checked_add(i128::from(left) * i128::from(right))
                })
                .ok_or_else(|| runtime(span, "Integer value too big for Cartan pairing"))?;
            row.push(
                i32::try_from(value)
                    .map_err(|_| runtime(span, "Integer value too big for Cartan pairing"))?,
            );
        }
        cartan.push(row);
    }
    let lie_type = infer_lie_type(&cartan, lattice_rank, span)?;
    let datum = BasedRootDatum::from_simple_data(lattice_rank, cartan, roots, coroots)
        .map_err(|error| runtime(span, error.to_string()))?;
    Ok(RootDatumHandle {
        isogeny: classify_isogeny(&datum),
        datum: Arc::new(datum),
        lie_type,
        prefers_coroots,
    })
}

fn infer_lie_type(
    cartan: &[Vec<i32>],
    lattice_rank: usize,
    span: SourceSpan,
) -> Result<LieTypeValue, Diagnostic> {
    let rank = cartan.len();
    let mut seen = vec![false; rank];
    let mut factors = Vec::new();
    for start in 0..rank {
        if seen[start] {
            continue;
        }
        let mut component = Vec::new();
        let mut pending = vec![start];
        seen[start] = true;
        while let Some(row) = pending.pop() {
            component.push(row);
            for column in 0..rank {
                if !seen[column] && (cartan[row][column] != 0 || cartan[column][row] != 0) {
                    seen[column] = true;
                    pending.push(column);
                }
            }
        }
        component.sort_unstable();
        let submatrix = component
            .iter()
            .map(|&row| {
                component
                    .iter()
                    .map(|&column| cartan[row][column])
                    .collect()
            })
            .collect::<Vec<Vec<_>>>();
        let size = component.len();
        let candidates = candidate_types(size);
        // B2 and C2 become one another after reversing both indices.  Atlas
        // nevertheless distinguishes their canonical ordered matrices, so an
        // exact match must win before considering arbitrary Dynkin relabeling.
        let candidate = candidates
            .iter()
            .copied()
            .find(|&(letter, candidate_rank)| submatrix == factor_cartan(letter, candidate_rank))
            .or_else(|| {
                candidates
                    .iter()
                    .copied()
                    .find(|&(letter, candidate_rank)| {
                        let expected = factor_cartan(letter, candidate_rank);
                        cartan_matches_up_to_permutation(&submatrix, &expected)
                    })
            });
        let Some((letter, candidate_rank)) = candidate else {
            return Err(runtime(
                span,
                "Matrices of (co)roots give an unrecognized Cartan matrix",
            ));
        };
        factors.push((letter, candidate_rank));
    }
    for _ in rank..lattice_rank {
        factors.push(('T', 1));
    }
    Ok(LieTypeValue { factors })
}

/// Cartan matrices are presentation-dependent: Atlas accepts the same root
/// system after a simultaneous permutation of the simple-root and coroot
/// indices. Match the invariant matrix up to that relabeling; comparing both
/// directed entries preserves the orientation of every multiple edge.
fn cartan_matches_up_to_permutation(actual: &[Vec<i32>], expected: &[Vec<i32>]) -> bool {
    if actual.len() != expected.len()
        || actual.iter().any(|row| row.len() != actual.len())
        || expected.iter().any(|row| row.len() != expected.len())
    {
        return false;
    }
    let rank = actual.len();
    let mut permutation = vec![usize::MAX; rank];
    let mut used = vec![false; rank];

    fn search(
        position: usize,
        actual: &[Vec<i32>],
        expected: &[Vec<i32>],
        permutation: &mut [usize],
        used: &mut [bool],
    ) -> bool {
        if position == actual.len() {
            return true;
        }
        for candidate in 0..expected.len() {
            if used[candidate] || actual[position][position] != expected[candidate][candidate] {
                continue;
            }
            let compatible = (0..position).all(|previous| {
                actual[position][previous] == expected[candidate][permutation[previous]]
                    && actual[previous][position] == expected[permutation[previous]][candidate]
            });
            if !compatible {
                continue;
            }
            permutation[position] = candidate;
            used[candidate] = true;
            if search(position + 1, actual, expected, permutation, used) {
                return true;
            }
            used[candidate] = false;
            permutation[position] = usize::MAX;
        }
        false
    }

    search(0, actual, expected, &mut permutation, &mut used)
}

fn candidate_types(rank: usize) -> Vec<(char, usize)> {
    let mut result = Vec::new();
    if (1..=RANK_MAX).contains(&rank) {
        result.push(('A', rank));
    }
    if (2..=RANK_MAX).contains(&rank) {
        result.push(('B', rank));
        result.push(('C', rank));
    }
    if (4..=RANK_MAX).contains(&rank) {
        result.push(('D', rank));
    }
    if (6..=8).contains(&rank) {
        result.push(('E', rank));
    }
    if rank == 4 {
        result.push(('F', rank));
    }
    if rank == 2 {
        result.push(('G', rank));
    }
    result
}

fn invert_integer_matrix(matrix: &[Vec<i32>]) -> Option<Vec<Vec<BigRational>>> {
    let rank = matrix.len();
    let mut augmented = (0..rank)
        .map(|row| {
            (0..rank)
                .map(|column| BigRational::from(matrix[row][column]))
                .chain((0..rank).map(|column| BigRational::from(usize::from(row == column))))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    for column in 0..rank {
        let pivot_row = (column..rank).find(|&row| augmented[row][column] != 0)?;
        augmented.swap(column, pivot_row);
        let pivot = augmented[column][column].clone();
        for entry in &mut augmented[column] {
            *entry = entry.clone() / pivot.clone();
        }
        let pivot_values = augmented[column].clone();
        for (row, entries) in augmented.iter_mut().enumerate() {
            if row == column {
                continue;
            }
            let factor = entries[column].clone();
            for (entry, pivot_entry) in entries.iter_mut().zip(&pivot_values) {
                *entry = entry.clone() - factor.clone() * pivot_entry.clone();
            }
        }
    }
    Some(
        augmented
            .into_iter()
            .map(|row| row.into_iter().skip(rank).collect())
            .collect(),
    )
}

fn classify_isogeny(datum: &BasedRootDatum) -> DatumIsogeny {
    let rank = datum.lattice_rank();
    if rank != datum.semisimple_rank() {
        return DatumIsogeny::Other;
    }
    let coroot_lattice = datum
        .simple_coroots()
        .iter()
        .map(|coroot| coroot.as_slice().to_vec())
        .collect::<Vec<_>>();
    let coroot_unimodular = is_unimodular(&coroot_lattice);
    let root_lattice = datum
        .simple_roots()
        .iter()
        .map(|root| root.as_slice().to_vec())
        .collect::<Vec<_>>();
    let root_unimodular = is_unimodular(&root_lattice);
    match (coroot_unimodular, root_unimodular) {
        (true, true) => DatumIsogeny::Both,
        (true, false) => DatumIsogeny::SimplyConnected,
        (false, true) => DatumIsogeny::Adjoint,
        (false, false) => DatumIsogeny::Other,
    }
}

fn is_unimodular(matrix: &[Vec<i32>]) -> bool {
    invert_integer_matrix(matrix).is_some_and(|inverse| {
        inverse
            .into_iter()
            .flatten()
            .all(|entry| BigInt::try_from(entry).is_ok())
    })
}

/// Cache of Cartan classifications keyed by the FULL datum content
/// (lattice rank + Cartan + simple roots + coroots — the earlier attempt
/// keyed only the Cartan matrix, which conflated e.g. simply-connected
/// and adjoint data and broke datum-equality checks), the theta matrices,
/// and the budget limits.
type ClassificationFingerprint = (
    atlas_real_group::BasedRootDatum,
    Vec<Vec<i32>>,
    Vec<Vec<i32>>,
    usize,
    usize,
    usize,
);

static CLASSIFICATION_CACHE: std::sync::OnceLock<
    std::sync::Mutex<
        std::collections::HashMap<ClassificationFingerprint, std::sync::Arc<CartanClassification>>,
    >,
> = std::sync::OnceLock::new();

fn classification_cache() -> &'static std::sync::Mutex<
    std::collections::HashMap<ClassificationFingerprint, std::sync::Arc<CartanClassification>>,
> {
    CLASSIFICATION_CACHE.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()))
}

fn classification_cached(
    inner_class: &InnerClass,
    class_budget: &CartanClassificationBudget,
    span: SourceSpan,
) -> Result<std::sync::Arc<CartanClassification>, Diagnostic> {
    let fingerprint = (
        inner_class.datum().clone(),
        inner_class
            .distinguished_involution()
            .involution()
            .weight_matrix()
            .to_vec(),
        inner_class
            .distinguished_involution()
            .involution()
            .coweight_matrix()
            .to_vec(),
        class_budget.weyl_budget(),
        class_budget.max_fiber_elements(),
        class_budget.max_peeling_steps(),
    );
    if let Some(existing) = classification_cache()
        .lock()
        .expect("classification cache poisoned")
        .get(&fingerprint)
    {
        return Ok(std::sync::Arc::clone(existing));
    }
    let classification = std::sync::Arc::new(
        CartanClassification::build(inner_class, class_budget)
            .map_err(|error| runtime(span, error.to_string()))?,
    );
    let _ = span;
    classification_cache()
        .lock()
        .expect("classification cache poisoned")
        .insert(fingerprint, std::sync::Arc::clone(&classification));
    Ok(classification)
}

/// The session-constant Cartan classification budget, shared by the inner
/// class pipeline and the ad-hoc dual fiber chains of the RealWeyl prints.
fn cartan_classification_budget() -> CartanClassificationBudget {
    CartanClassificationBudget::new(
        INTEGER_BUDGET,
        AdjointFiberBudget::new(INTEGER_BUDGET, 1_000_000, 10_000_000),
        WEYL_BUDGET,
        4_096,
        4_096,
    )
}

fn build_inner_class_context(
    handle: &RootDatumHandle,
    inner_class: InnerClass,
    span: SourceSpan,
) -> Result<Arc<InnerClassContext>, Diagnostic> {
    let class_budget = cartan_classification_budget();
    let classification = classification_cached(&inner_class, &class_budget, span)?;
    let strong = StrongRealClassification::build(&classification, FIBER_BUDGET)
        .map_err(|error| runtime(span, error.to_string()))?;
    let order = ExternalFormOrder::build(&inner_class, &classification)
        .map_err(|error| runtime(span, error.to_string()))?;
    let layout = InnerClassLayout::build(&inner_class, &INTEGER_BUDGET)
        .map_err(|error| runtime(span, error.to_string()))?;
    // The dual side once: the dual form count is the dual fundamental weak
    // partition size, and the correspondence pairs each Cartan class with its
    // dual class (upstream dual InnerClass constructor, innerclass.cpp:435).
    let dual_inner = dual_inner_class(&inner_class, WEYL_BUDGET, ROOT_BUDGET)
        .map_err(|error| runtime(span, error.to_string()))?;
    let dual_classification = classification_cached(&dual_inner, &class_budget, span)?;
    let dual_form_count = dual_classification.weak_real_form_count();
    let dual_cartans = dual_cartan_correspondence(
        &inner_class,
        &classification,
        &dual_inner,
        &dual_classification,
        WEYL_BUDGET,
    )
    .map_err(|error| runtime(span, error.to_string()))?;
    let forms = build_presentations(
        &inner_class,
        &classification,
        &order,
        &layout,
        &INTEGER_BUDGET,
    )
    .map_err(|error| runtime(span, error.to_string()))?;
    let canonical_forms = Mutex::new(
        std::iter::repeat_with(Weak::new)
            .take(order.form_count())
            .collect(),
    );
    Ok(Arc::new(InnerClassContext {
        root_datum: handle.clone(),
        inner_class,
        classification,
        strong,
        order,
        layout,
        dual_form_count,
        dual_cartans,
        forms,
        canonical_forms,
        #[cfg(test)]
        canonical_build_test_gate: Mutex::new(None),
    }))
}

fn build_inner_class(
    handle: &RootDatumHandle,
    matrix: Vec<Vec<i32>>,
    span: SourceSpan,
) -> Result<Arc<InnerClassContext>, Diagnostic> {
    let transpose = |m: &[Vec<i32>]| -> Vec<Vec<i32>> {
        let size = m.len();
        (0..size)
            .map(|i| (0..size).map(|j| m[j][i]).collect())
            .collect()
    };
    let coweight = transpose(&matrix);
    let involution = LatticeInvolution::new(&handle.datum, matrix, coweight)
        .map_err(|error| runtime(span, error.to_string()))?;
    // Upstream accepts any root-datum involution here and left-composes it
    // into a distinguished one (`check_involution`, atlas-types.w:2829).
    let inner_class =
        InnerClass::from_root_involution((*handle.datum).clone(), involution, ROOT_BUDGET)
            .map_err(|error| runtime(span, error.to_string()))?;
    build_inner_class_context(handle, inner_class, span)
}

fn build_dual_inner_class(
    parent: &Arc<InnerClassContext>,
    span: SourceSpan,
) -> Result<Arc<InnerClassContext>, Diagnostic> {
    let inner_class = dual_inner_class(&parent.inner_class, WEYL_BUDGET, ROOT_BUDGET)
        .map_err(|error| runtime(span, error.to_string()))?;
    let datum = inner_class.datum().clone();
    // The dual inner class carries the dual datum (inner_class_value::dual,
    // atlas-types.w:3152-3156): its coroot preference is switched
    // (RootSystem DualTag, rootdata.cpp:341) and its Lie type is the
    // letter-wise dual of the parent's.
    let handle = RootDatumHandle {
        isogeny: classify_isogeny(&datum),
        datum: Arc::new(datum),
        lie_type: dual_lie_type(&parent.root_datum.lie_type),
        prefers_coroots: !parent.root_datum.prefers_coroots,
    };
    build_inner_class_context(&handle, inner_class, span)
}

/// Apply the domain-owned coercions registered by the Atlas type layer.
/// Keeping these conversions here preserves the root-datum provenance carried
/// by the immutable handles instead of reconstructing a mathematically equal
/// but observably different value in the evaluator.
pub(crate) fn coerce(tag: &str, value: Value, span: SourceSpan) -> Result<Value, Diagnostic> {
    match tag {
        "LT" => call("Lie_type", &[value], span),
        "IcRf" => call("inner_class", &[value], span),
        "RdIc" => match value {
            Value::Domain(DomainValue::InnerClass(context)) => Ok(Value::Domain(
                DomainValue::RootDatum(context.root_datum.clone()),
            )),
            other => Err(type_error(
                span,
                format!("expected an InnerClass, found {other}"),
            )),
        },
        "RdRf" => match value {
            Value::Domain(DomainValue::RealForm(context)) => Ok(Value::Domain(
                DomainValue::RootDatum(context.parent.root_datum.clone()),
            )),
            other => Err(type_error(
                span,
                format!("expected a RealForm, found {other}"),
            )),
        },
        // int_to_split_coercion (atlas-types.w:5113-5117): the plain part,
        // narrowed like every Atlas int-to-machine-int extraction.
        "SpI" => match value {
            Value::Integer(value) => Ok(Value::Domain(DomainValue::Split(SplitValue::new(
                narrow_split_component(&value, span)?,
                0,
            )))),
            other => Err(type_error(span, format!("expected an int, found {other}"))),
        },
        // pair_to_split_coercion (atlas-types.w:5119-5125): (e, f) order.
        "Sp(I,I)" => match value {
            Value::Tuple(components) => match components.as_slice() {
                [Value::Integer(e), Value::Integer(f)] => {
                    Ok(Value::Domain(DomainValue::Split(SplitValue::new(
                        narrow_split_component(e, span)?,
                        narrow_split_component(f, span)?,
                    ))))
                }
                _ => Err(type_error(
                    span,
                    format!(
                        "expected an (int,int) pair, found {}",
                        Value::Tuple(components)
                    ),
                )),
            },
            other => Err(type_error(
                span,
                format!("expected an (int,int) pair, found {other}"),
            )),
        },
        other => Err(runtime(
            span,
            format!("conversion '{other}' is not implemented"),
        )),
    }
}

fn build_real_form(
    parent: &Arc<InnerClassContext>,
    external: usize,
    span: SourceSpan,
) -> Result<Arc<RealFormContext>, Diagnostic> {
    let internal = parent
        .order
        .internal(external)
        .ok_or_else(|| runtime(span, format!("Illegal real form number: {external}")))?;
    {
        let canonical_forms = parent
            .canonical_forms
            .lock()
            .map_err(|_| runtime(span, "canonical real form cache poisoned"))?;
        if let Some(existing) = canonical_forms.get(external).and_then(Weak::upgrade) {
            return Ok(existing);
        }
    }

    // Construct outside the cache lock. KGB completion can be expensive, and
    // concurrent callers must not serialize unrelated real forms behind it.
    let mut table =
        InnerClassContext::fresh_table(parent).map_err(|error| runtime(span, error.to_string()))?;
    let fundamental = parent
        .classification
        .cartan_ids()
        .next()
        .ok_or_else(|| runtime(span, "empty classification"))?;
    table
        .add_cartan(&parent.classification, fundamental)
        .map_err(|error| runtime(span, error.to_string()))?;
    let seed = RealFormSeed::build(
        &parent.inner_class,
        &parent.classification,
        &parent.strong,
        &table,
        internal,
        &INTEGER_BUDGET,
        FIBER_BUDGET,
    )
    .map_err(|error| runtime(span, error.to_string()))?;
    let graph = KgbGraph::build(
        &parent.inner_class,
        &parent.classification,
        &parent.strong,
        &mut table,
        &seed,
    )
    .map_err(|error| runtime(span, error.to_string()))?;
    let table = Arc::new(table);
    let graph = Arc::new(graph);
    let rep = Arc::new(
        RepTableOwner::from_shared(Arc::clone(&table), Arc::clone(&graph))
            .map_err(|error| runtime(span, error.to_string()))?,
    );
    let candidate = Arc::new(RealFormContext {
        parent: Arc::clone(parent),
        external,
        internal,
        table,
        graph,
        rep,
        full_deform_cache: Mutex::new(DeformationCache::default()),
        twisted_full_deform_cache: Mutex::new(DeformationCache::default()),
    });

    #[cfg(test)]
    {
        let gate = parent
            .canonical_build_test_gate
            .lock()
            .map_err(|_| runtime(span, "canonical real form test gate poisoned"))?
            .clone();
        if let Some(gate) = gate {
            gate.wait().map_err(|message| runtime(span, message))?;
        }
    }

    // A concurrent builder may have installed the same form while this KGB
    // graph was being completed. Preserve the first live canonical owner.
    let mut canonical_forms = parent
        .canonical_forms
        .lock()
        .map_err(|_| runtime(span, "canonical real form cache poisoned"))?;
    let slot = canonical_forms
        .get_mut(external)
        .ok_or_else(|| runtime(span, "canonical real form cache is inconsistent"))?;
    if let Some(existing) = slot.upgrade() {
        return Ok(existing);
    }
    *slot = Arc::downgrade(&candidate);
    Ok(candidate)
}

/// The custom-seed construction of `real_form_value::build`
/// (atlas-types.w:3543-3544): a fresh KGB pipeline seeded with the
/// caller's (cocharacter, torus part) pair rather than the form's elected
/// seed. Only `synthetic_real_form` plans that failed the default test
/// reach here.
fn build_custom_real_form(
    parent: &Arc<InnerClassContext>,
    plan: &SyntheticRealForm,
    span: SourceSpan,
) -> Result<Arc<RealFormContext>, Diagnostic> {
    let mut table =
        InnerClassContext::fresh_table(parent).map_err(|error| runtime(span, error.to_string()))?;
    let fundamental = parent
        .classification
        .cartan_ids()
        .next()
        .ok_or_else(|| runtime(span, "empty classification"))?;
    table
        .add_cartan(&parent.classification, fundamental)
        .map_err(|error| runtime(span, error.to_string()))?;
    let seed = RealFormSeed::custom(
        &parent.inner_class,
        &parent.classification,
        &table,
        plan.internal,
        &plan.cocharacter,
        plan.torus_part.clone(),
    )
    .map_err(|error| runtime(span, error.to_string()))?;
    let graph = KgbGraph::build(
        &parent.inner_class,
        &parent.classification,
        &parent.strong,
        &mut table,
        &seed,
    )
    .map_err(|error| runtime(span, error.to_string()))?;
    let table = Arc::new(table);
    let graph = Arc::new(graph);
    let rep = Arc::new(
        RepTableOwner::from_shared(Arc::clone(&table), Arc::clone(&graph))
            .map_err(|error| runtime(span, error.to_string()))?,
    );
    Ok(Arc::new(RealFormContext {
        parent: Arc::clone(parent),
        external: plan.external,
        internal: plan.internal,
        table,
        graph,
        rep,
        full_deform_cache: Mutex::new(DeformationCache::default()),
        twisted_full_deform_cache: Mutex::new(DeformationCache::default()),
    }))
}

impl InnerClassContext {
    fn fresh_table(
        parent: &Arc<InnerClassContext>,
    ) -> Result<InvolutionTable, atlas_real_group::StructureError> {
        InvolutionTable::new(
            &parent.inner_class,
            InvolutionTableBudget::new(FIBER_BUDGET, INTEGER_BUDGET),
        )
    }
}

fn as_lie_type(value: &Value, span: SourceSpan) -> Result<LieTypeValue, Diagnostic> {
    match value {
        Value::Domain(DomainValue::LieType(lie_type)) => Ok(lie_type.clone()),
        // Upstream's implicit string->LieType coercion.
        Value::String(text) => parse_lie_type(text, span),
        other => Err(type_error(
            span,
            format!("expected a Lie type, found {other}"),
        )),
    }
}

fn as_usize(value: &Value, span: SourceSpan) -> Result<usize, Diagnostic> {
    match value {
        Value::Integer(value) => usize::try_from(value)
            .map_err(|_| type_error(span, "expected a nonnegative machine integer")),
        other => Err(type_error(span, format!("expected an int, found {other}"))),
    }
}

fn as_matrix(value: &Value, span: SourceSpan) -> Result<Vec<Vec<i32>>, Diagnostic> {
    if let Value::Matrix(matrix) = value {
        if matrix.rows() != matrix.cols() {
            return Err(type_error(
                span,
                format!(
                    "expected a square mat; received a {}x{} matrix",
                    matrix.rows(),
                    matrix.cols()
                ),
            ));
        }
    }
    let rows = as_matrix_rows(value, span)?;
    if rows.iter().any(|row| row.len() != rows.len()) {
        return Err(type_error(span, "expected a square mat"));
    }
    Ok(rows)
}

fn as_matrix_rows(value: &Value, span: SourceSpan) -> Result<Vec<Vec<i32>>, Diagnostic> {
    let rows = match value {
        Value::Matrix(matrix) => {
            // A zero-row matrix still carries its column count.  Represent
            // those invalid rectangular shapes as empty rows so callers that
            // validate dimensions do not accidentally treat 0xN as 0x0.
            if matrix.rows() == 0 && matrix.cols() > 0 {
                return Ok((0..matrix.cols()).map(|_| Vec::new()).collect());
            }
            return Ok((0..matrix.rows())
                .map(|row| {
                    (0..matrix.cols())
                        .map(|column| {
                            matrix
                                .entry(row, column)
                                .expect("matrix dimensions guarantee in-range entries")
                        })
                        .collect()
                })
                .collect());
        }
        // The pre-typed dynamic evaluator still constructs nested lists.
        Value::List(rows) => rows,
        _ => return Err(type_error(span, "expected a mat")),
    };
    let column_count = rows.first().map_or(0, |row| match row {
        Value::List(entries) => entries.len(),
        _ => 0,
    });
    let mut converted = Vec::with_capacity(rows.len());
    for row in rows {
        let Value::List(entries) = row else {
            return Err(type_error(span, "expected a mat"));
        };
        if entries.len() != column_count {
            return Err(type_error(span, "expected a rectangular mat"));
        }
        converted.push(
            entries
                .iter()
                .map(|entry| match entry {
                    Value::Integer(value) => i32::try_from(value)
                        .map_err(|_| type_error(span, "matrix entry out of range")),
                    _ => Err(type_error(span, "expected integer matrix entries")),
                })
                .collect::<Result<Vec<_>, _>>()?,
        );
    }
    Ok(converted)
}

/// Merge one (KType, SplitValue) term into an ordered K-type polynomial
/// (KTypePolValue::add_term semantics: same K-type merges, zero drops).
fn merge_ktype_term(terms: &mut Vec<(SplitValue, KType)>, ktype: KType, split: SplitValue) {
    merge_pol_term(terms, split, ktype);
}

fn full_deform_key(repr: &StandardRepr) -> FullDeformKey {
    FullDeformKey {
        x: repr.x(),
        y_bits: repr.y_bits().clone(),
        gamma: repr.gamma().clone(),
    }
}

fn deadline_expired(deadline: Option<Instant>) -> bool {
    deadline.is_some_and(|deadline| Instant::now() >= deadline)
}

fn cached_deformation(
    cache: &Mutex<DeformationCache>,
    key: &FullDeformKey,
    span: SourceSpan,
) -> Result<Option<Vec<(SplitValue, KType)>>, Diagnostic> {
    cache
        .lock()
        .map_err(|_| runtime(span, "deformation cache poisoned"))
        .map(|cache| cache.values.get(key).cloned())
}

fn store_deformation(
    cache: &Mutex<DeformationCache>,
    key: FullDeformKey,
    terms: Vec<(SplitValue, KType)>,
    span: SourceSpan,
) -> Result<(), Diagnostic> {
    cache
        .lock()
        .map_err(|_| runtime(span, "deformation cache poisoned"))?
        .values
        .insert(key, terms);
    Ok(())
}

/// The full deformation of one final standard parameter (repr.cpp:
/// 2251-2290): the finals of the scale-0 parameter contribute their
/// K-types, then each reducibility point's scaled parameter is deformed
/// via the block's deformation terms, scaled back to its previous
/// reducibility point.
fn full_deformation_terms(
    rc: &RepContext<'_>,
    z: &StandardRepr,
    context: &Arc<RealFormContext>,
    span: SourceSpan,
    deadline: Option<Instant>,
) -> Result<Option<Vec<(KType, SplitValue)>>, Diagnostic> {
    if deadline_expired(deadline) {
        return Ok(None);
    }
    let centered = if denominator_exceeds_alcove_bound(rc.rank(), z.gamma().denominator()) {
        Some(domain_alcove_center(rc, z).map_err(|error| structure_diagnostic(error, span))?)
    } else {
        None
    };
    if deadline_expired(deadline) {
        return Ok(None);
    }
    let z = centered.as_ref().unwrap_or(z);
    let mut result: Vec<(KType, SplitValue)> = Vec::new();
    // Scale-0 base (repr.cpp:2257-2266).
    let z0 = rc
        .scale(z, 0, 1)
        .map_err(|error| structure_diagnostic(error, span))?;
    let base = rc
        .finals_for(&z0)
        .map_err(|error| structure_diagnostic(error, span))?;
    if deadline_expired(deadline) {
        return Ok(None);
    }
    for (final_sr, coef) in &base {
        if deadline_expired(deadline) {
            return Ok(None);
        }
        let lambda_rho = rc
            .lambda_rho(final_sr)
            .map_err(|error| structure_diagnostic(error, span))?;
        let ktype = KType::sr_k(rc, final_sr.x(), &lambda_rho)
            .map_err(|error| structure_diagnostic(error, span))?;
        result.push((ktype, SplitValue::new(*coef, 0)));
        if deadline_expired(deadline) {
            return Ok(None);
        }
    }
    // Reducibility-point recursion (repr.cpp:2268-2289).
    let rp = rc
        .reducibility_points(z)
        .map_err(|error| structure_diagnostic(error, span))?;
    if deadline_expired(deadline) {
        return Ok(None);
    }
    for &(num, den) in rp.iter().rev() {
        if deadline_expired(deadline) {
            return Ok(None);
        }
        let zi = rc
            .scale(z, num, den)
            .map_err(|error| structure_diagnostic(error, span))?;
        let zi = zi
            .deform_readjust(rc)
            .map_err(|error| structure_diagnostic(error, span))?;
        if deadline_expired(deadline) {
            return Ok(None);
        }
        let dual_parent = build_dual_inner_class(&context.parent, span)?;
        let dual_quasisplit = dual_parent.order.quasisplit_external();
        let dual_rf = build_real_form(&dual_parent, dual_quasisplit, span)?;
        let block = build_block(context, &dual_rf, span)?;
        if deadline_expired(deadline) {
            return Ok(None);
        }
        let mut kl_table =
            KlTable::new(&block.graph).map_err(|error| structure_diagnostic(error, span))?;
        kl_table
            .fill(0)
            .map_err(|error| structure_diagnostic(error, span))?;
        if deadline_expired(deadline) {
            return Ok(None);
        }
        let y = (0..block.graph.size())
            .find(|&y| block.graph.x(y) == Some(zi.x()))
            .ok_or_else(|| runtime(span, "deformation parameter not in the block"))?;
        let lambda_rho = rc
            .lambda_rho(&zi)
            .map_err(|error| structure_diagnostic(error, span))?;
        let terms = rc
            .deformation_terms(&block.graph, y, zi.gamma(), &lambda_rho, &kl_table)
            .map_err(|error| structure_diagnostic(error, span))?;
        if deadline_expired(deadline) {
            return Ok(None);
        }
        for (term, coef) in terms {
            if deadline_expired(deadline) {
                return Ok(None);
            }
            let term_rp = rc
                .reducibility_points(&term)
                .map_err(|error| structure_diagnostic(error, span))?;
            let index = if term_rp.last() == Some(&(1, 1)) {
                term_rp.len().saturating_sub(1)
            } else {
                term_rp.len()
            };
            let point = if index > 0 {
                term_rp[index - 1]
            } else {
                (0, 1)
            };
            let scaled = rc
                .scale(&term, point.0, point.1)
                .map_err(|error| structure_diagnostic(error, span))?;
            let scaled_lambda = rc
                .lambda_rho(&scaled)
                .map_err(|error| structure_diagnostic(error, span))?;
            let ktype = KType::sr_k(rc, scaled.x(), &scaled_lambda)
                .map_err(|error| structure_diagnostic(error, span))?;
            result.push((ktype, SplitValue::new(coef, 0)));
            if deadline_expired(deadline) {
                return Ok(None);
            }
        }
    }
    if deadline_expired(deadline) {
        return Ok(None);
    }
    Ok(Some(result))
}

fn compute_full_deform(
    parameter: &ParamValue,
    span: SourceSpan,
    deadline: Option<Instant>,
) -> Result<Option<Vec<(SplitValue, KType)>>, Diagnostic> {
    let rc = rep_context(&parameter.context);
    let finals = rc
        .finals_for(&parameter.repr)
        .map_err(|error| structure_diagnostic(error, span))?;
    if deadline_expired(deadline) {
        return Ok(None);
    }
    let mut terms: Vec<(SplitValue, KType)> = Vec::new();
    for (final_sr, coef) in &finals {
        if deadline_expired(deadline) {
            return Ok(None);
        }
        let Some(deformed) =
            full_deformation_terms(&rc, final_sr, &parameter.context, span, deadline)?
        else {
            return Ok(None);
        };
        for (ktype, split) in deformed {
            let scaled_split = SplitValue::new(split.e() * *coef, split.f() * *coef);
            merge_ktype_term(&mut terms, ktype, scaled_split);
            if deadline_expired(deadline) {
                return Ok(None);
            }
        }
    }
    sort_ktypepol_terms(&mut terms);
    if deadline_expired(deadline) {
        return Ok(None);
    }
    Ok(Some(terms))
}

fn compute_twisted_full_deform(
    parameter: &ParamValue,
    span: SourceSpan,
    timer_ms: Option<i32>,
) -> Result<Option<Vec<(SplitValue, KType)>>, Diagnostic> {
    let rc = rep_context(&parameter.context);
    let (delta, twist) = distinguished_twist(parameter, span)?;
    let context = ExtRepContext::new(&rc, delta.clone())
        .map_err(|error| structure_diagnostic(error, span))?;
    let finals = extended_finalise(&context, &parameter.repr)
        .map_err(|error| structure_diagnostic(error, span))?;
    // Atlas starts the timed computation after extended_finalise (axis.w:
    // 8303-8308), so setup cost is outside the cooperative deadline.
    let deadline =
        timer_ms.and_then(|timer| Instant::now().checked_add(Duration::from_millis(timer as u64)));
    if deadline_expired(deadline) {
        return Ok(None);
    }

    let real_form = &parameter.context;
    let mut lookup =
        |zi: &StandardRepr| twisted_reducibility_lookup(real_form, &rc, &delta, &twist, zi);
    let mut cancelled = || deadline_expired(deadline);
    let mut terms: Vec<(SplitValue, KType)> = Vec::new();
    for (final_sr, finalise_flip) in &finals {
        if cancelled() {
            return Ok(None);
        }
        let Some((deformed, flip)) =
            twisted_deformation_with_cancel(&context, final_sr, &mut lookup, &mut cancelled)
                .map_err(|error| structure_diagnostic(error, span))?
        else {
            return Ok(None);
        };
        let coefficient = if flip != *finalise_flip {
            SplitValue::new(0, 1)
        } else {
            SplitValue::new(1, 0)
        };
        for (ktype, split) in deformed {
            let split: (i32, i32) = split.into();
            let scaled = SplitValue::new(split.0, split.1).mul(coefficient);
            merge_pol_term(&mut terms, scaled, ktype);
            if cancelled() {
                return Ok(None);
            }
        }
    }
    sort_ktypepol_terms(&mut terms);
    if cancelled() {
        return Ok(None);
    }
    Ok(Some(terms))
}

/// The transpose of a row-major matrix.
fn transpose_matrix(rows: &[Vec<i32>]) -> Vec<Vec<i32>> {
    let columns = rows.first().map_or(0, Vec::len);
    (0..columns)
        .map(|column| rows.iter().map(|row| row[column]).collect())
        .collect()
}

fn matrix_value(rows: &[Vec<i32>], span: SourceSpan) -> Result<Value, Diagnostic> {
    let row_count = rows.len();
    let column_count = rows.first().map_or(0, Vec::len);
    if rows.iter().any(|row| row.len() != column_count) {
        return Err(runtime(span, "internal non-rectangular matrix"));
    }
    let data = (0..column_count)
        .flat_map(|column| (0..row_count).map(move |row| rows[row][column]))
        .collect();
    Matrix::from_columns(row_count, column_count, data)
        .map(Value::Matrix)
        .ok_or_else(|| runtime(span, "matrix dimensions exceed machine range"))
}

/// The row-major view of a column-major `Matrix`.
/// Determinant of a square i32 matrix by Laplace expansion (ranks here
/// are small; Cartan determinants stay tiny for classical types).
fn determinant_i32(matrix: &[Vec<i32>]) -> i64 {
    let rank = matrix.len();
    if rank == 0 {
        return 1;
    }
    if rank == 1 {
        return i64::from(matrix[0][0]);
    }
    let mut total = 0_i64;
    for (column, &entry) in matrix[0].iter().enumerate() {
        if entry == 0 {
            continue;
        }
        let minor: Vec<Vec<i32>> = matrix[1..]
            .iter()
            .map(|row| {
                row.iter()
                    .enumerate()
                    .filter_map(|(index, &value)| (index != column).then_some(value))
                    .collect()
            })
            .collect();
        let sign = if column % 2 == 0 { 1 } else { -1 };
        total += sign * i64::from(entry) * determinant_i32(&minor);
    }
    total
}

/// Cramer's rule for `matrix x = rhs`: returns the i64 solution and the
/// (nonzero) determinant.
fn cramer_solution(matrix: &[Vec<i32>], rhs: &[i32]) -> Option<(Vec<i64>, i64)> {
    let rank = matrix.len();
    if rhs.len() != rank || matrix.iter().any(|row| row.len() != rank) {
        return None;
    }
    let denominator = determinant_i32(matrix);
    if denominator == 0 {
        return None;
    }
    let mut numerators = Vec::with_capacity(rank);
    for column in 0..rank {
        let mut replaced: Vec<Vec<i32>> = matrix.to_vec();
        for (row_index, entry) in replaced.iter_mut().enumerate() {
            entry[column] = rhs[row_index];
        }
        numerators.push(determinant_i32(&replaced));
    }
    Some((numerators, denominator))
}

fn matrix_rows(matrix: &Matrix) -> Vec<Vec<i32>> {
    (0..matrix.rows())
        .map(|row| {
            (0..matrix.cols())
                .filter_map(|col| matrix.entry(row, col))
                .collect()
        })
        .collect()
}

/// Tarjan's strong components (graph.cpp:186-294) with the induced quotient
/// graph (add_links, graph.cpp:345-361). Returns `(partition, induced)` where
/// `partition[c]` lists the vertices of component `c` (discovered in sink-first
/// topological order, vertices within a class in ascending order) and
/// `induced[c]` lists the component targets of `c`'s outgoing edges.
pub(crate) fn strong_components(graph: &[Vec<usize>]) -> (Vec<Vec<usize>>, Vec<Vec<usize>>) {
    let size = graph.len();
    let nil = size;
    let infinity = size + 1;
    let mut rank: Vec<usize> = vec![0; size];
    let mut class_of: Vec<usize> = vec![0; size];
    let mut partition: Vec<Vec<usize>> = Vec::new();
    let mut induced: Vec<Vec<usize>> = Vec::new();

    for x0 in 0..size {
        if rank[x0] >= infinity {
            continue;
        }
        let mut count = 1;
        rank[x0] = count;
        count += 1;
        // active entries: (vertex, parent location, next edge index, min rank)
        let mut active: Vec<(usize, usize, usize, usize)> = vec![(x0, nil, 0, rank[x0])];
        let mut cur_pos = 0;
        while cur_pos != nil {
            let (v, parent, next_edge, min) = active[cur_pos];
            let edges = &graph[v];
            let mut next_edge = next_edge;
            let mut min = min;
            let mut advanced = false;
            while next_edge < edges.len() {
                let y = edges[next_edge];
                next_edge += 1;
                if rank[y] == 0 {
                    let x_pos = cur_pos;
                    let y_pos = active.len();
                    rank[y] = count;
                    count += 1;
                    active.push((y, x_pos, 0, rank[y]));
                    active[x_pos].2 = next_edge;
                    active[x_pos].3 = min;
                    cur_pos = y_pos;
                    advanced = true;
                    break;
                } else if rank[y] < min {
                    min = rank[y];
                }
            }
            if advanced {
                continue;
            }
            // x matures
            let new_pos = parent;
            if min == rank[v] {
                let class_index = partition.len();
                partition.push(Vec::new());
                let mut out: Vec<&[usize]> = Vec::new();
                for entry in active.iter().skip(cur_pos) {
                    let y = entry.0;
                    partition[class_index].push(y);
                    class_of[y] = class_index;
                    rank[y] = infinity;
                    out.push(&graph[y]);
                }
                active.truncate(cur_pos);
                let mut seen = std::collections::BTreeSet::new();
                for list in &out {
                    for &e in *list {
                        seen.insert(class_of[e]);
                    }
                }
                seen.remove(&class_index);
                induced.push(seen.into_iter().collect());
            } else if min < active[new_pos].3 {
                active[new_pos].3 = min;
            }
            cur_pos = new_pos;
        }
    }
    (partition, induced)
}

/// Build the full W-graph exposed by the Block overloads of `W_graph` and
/// `W_cells` (atlas-types.w:8738-8808).  The vertices retain Block numbering;
/// every nonzero mu-pair contributes an undirected labelled edge.
type WGraphDescentSets = Vec<BTreeSet<usize>>;
type WGraphEdges = Vec<Vec<(usize, i32)>>;

fn block_w_graph_data(
    block: &BlockValue,
    span: SourceSpan,
) -> Result<(WGraphDescentSets, WGraphEdges), Diagnostic> {
    let mut kl_table =
        KlTable::new(&block.graph).map_err(|error| structure_diagnostic(error, span))?;
    kl_table
        .fill(0)
        .map_err(|error| structure_diagnostic(error, span))?;
    let size = block.graph.size();
    let rank = block.graph.rank();
    let descent_sets = (0..size)
        .map(|z| {
            let descents = kl_table.support().descent_set(z);
            (0..rank)
                .filter(|&generator| descents.is_set(generator))
                .collect()
        })
        .collect();
    let mut edges: Vec<Vec<(usize, i32)>> = vec![Vec::new(); size];
    for y in 0..size {
        for pair in kl_table.mu_column(y) {
            edges[y].push((pair.x, pair.coef));
            edges[pair.x].push((y, pair.coef));
        }
    }
    for targets in &mut edges {
        targets.sort_unstable();
    }
    Ok((descent_sets, edges))
}

fn w_graph_vertex_value(
    element: usize,
    targets: &[(usize, i32)],
    descent_sets: &[BTreeSet<usize>],
) -> Value {
    Value::Tuple(vec![
        Value::List(
            descent_sets[element]
                .iter()
                .map(|&generator| Value::Integer(BigInt::from(generator)))
                .collect(),
        ),
        Value::List(
            targets
                .iter()
                .map(|&(target, coefficient)| {
                    Value::Tuple(vec![
                        Value::Integer(BigInt::from(target)),
                        Value::Integer(BigInt::from(coefficient)),
                    ])
                })
                .collect(),
        ),
    ])
}

fn block_w_graph_value(
    name: &str,
    block: &BlockValue,
    span: SourceSpan,
) -> Result<Value, Diagnostic> {
    let (descent_sets, edges) = block_w_graph_data(block, span)?;
    if name == "W_graph" {
        return Ok(Value::List(
            edges
                .iter()
                .enumerate()
                .map(|(element, targets)| w_graph_vertex_value(element, targets, &descent_sets))
                .collect(),
        ));
    }

    // DecomposedWGraph (wgraph.cpp:58-116): retain precisely the edges
    // oriented out of a vertex whose target descent set is not a superset.
    let oriented: Vec<Vec<usize>> = edges
        .iter()
        .enumerate()
        .map(|(x, targets)| {
            targets
                .iter()
                .filter_map(|&(y, _)| (!descent_sets[y].is_superset(&descent_sets[x])).then_some(y))
                .collect()
        })
        .collect();
    let (mut partition, _induced) = strong_components(&oriented);
    for members in &mut partition {
        members.sort_unstable();
    }
    Ok(Value::List(
        partition
            .iter()
            .map(|members| {
                let mut relative = vec![0_usize; edges.len()];
                for (position, &member) in members.iter().enumerate() {
                    relative[member] = position;
                }
                let member_set: BTreeSet<usize> = members.iter().copied().collect();
                let vertices = members
                    .iter()
                    .map(|&member| {
                        let targets: Vec<(usize, i32)> = edges[member]
                            .iter()
                            .copied()
                            .filter(|&(target, _)| member_set.contains(&target))
                            .map(|(target, coefficient)| (relative[target], coefficient))
                            .collect();
                        w_graph_vertex_value(member, &targets, &descent_sets)
                    })
                    .collect();
                Value::Tuple(vec![
                    Value::List(
                        members
                            .iter()
                            .map(|&member| Value::Integer(BigInt::from(member)))
                            .collect(),
                    ),
                    Value::List(vertices),
                ])
            })
            .collect(),
    ))
}

/// The upstream KLV polynomial printing (polynomials_def.h:300-331,
/// `printMonomial` + `Polynomial::print`): least-degree-first coefficients,
/// printed highest degree down, `q`/`q^d` monomials with a `+` separator for
/// positive middle coefficients, `1` for the constant.
fn print_klpol(polynomial: &KlPol) -> String {
    if polynomial.is_zero() {
        return "0".to_string();
    }
    let degree = polynomial.degree();
    let mut out = String::new();
    for i in (0..=degree).rev() {
        let coefficient = polynomial.coefficient(i);
        if coefficient == 0 {
            continue;
        }
        if i < degree && coefficient > 0 {
            out.push('+');
        }
        if i == 0 {
            out.push_str(&coefficient.to_string());
        } else {
            if coefficient == -1 {
                out.push('-');
            } else if coefficient != 1 {
                out.push_str(&coefficient.to_string());
            }
            out.push('q');
            if i > 1 {
                out.push_str(&format!("^{i}"));
            }
        }
    }
    out
}

/// `kl_tab.KL_pol(x, y)` resolved through the hash pool.
fn kl_pol_at<B: BlockTopology>(
    kl_table: &KlTable<'_, B>,
    x: usize,
    y: usize,
    span: SourceSpan,
) -> Result<KlPol, Diagnostic> {
    let index = kl_table
        .kl_pol(x, y)
        .map_err(|error| structure_diagnostic(error, span))?;
    kl_table
        .pool()
        .get(index)
        .cloned()
        .ok_or_else(|| runtime(span, "internal KL pool miss"))
}

/// Restore one stored representation-block row at the querying parameter's
/// infinitesimal character.  A cache hit can carry a central shift relative
/// to the representative that originally materialized the block, so using
/// the stored `gamma_lambda` directly is observably wrong.
fn located_row_parameter(
    context: &Arc<RealFormContext>,
    located: &LocatedBlock,
    row: usize,
) -> Result<StandardRepr, StructureError> {
    let rc = rep_context(context);
    let block = located.block();
    let stored = block.element(row).ok_or(StructureError::IndexOutOfRange {
        index: row,
        upper_bound: block.size(),
    })?;
    let shifted = stored.gamma_lambda().add(located.relative_shift())?;
    StandardReprMod::build(&rc, stored.x(), &shifted)?
        .to_standard(&rc, located.prepared_query().gamma())
}

/// Merge two ascending-by-row contribution lists, summing the
/// coefficients of like rows (upstream `combine`, repr.cpp:1833-1852).
fn combine_contributions(a: &[(usize, i32)], b: &[(usize, i32)]) -> Vec<(usize, i32)> {
    let mut merged = a.to_vec();
    merged.extend_from_slice(b);
    merged.sort_by_key(|&(row, _)| row);
    let mut result: Vec<(usize, i32)> = Vec::with_capacity(merged.len());
    for (row, coefficient) in merged {
        match result.last_mut() {
            Some((last_row, last_coefficient)) if *last_row == row => {
                *last_coefficient = last_coefficient.wrapping_add(coefficient);
            }
            _ => result.push((row, coefficient)),
        }
    }
    result
}

/// `KL_sum_at_s` (atlas-types.w:8350-8360) via
/// `Rep_table::KL_column_at_s` (repr.cpp:2127-2164) over the shared
/// RepTable partial lookup: the KL column of a final parameter evaluated
/// at q = s. Each block element is expanded into its final-row
/// contributions (repr.cpp:1861-1898), so singular parameters pick up
/// multi-row coefficients; the retired classic path assumed a regular
/// infinitesimal character and a singleton contribution per element.
/// Shared core of `KL_sum_at_s` / `KL_sum_at_s_to_height`
/// (atlas-types.w:8350-8368 over repr.cpp:2127-2230).  `height_bound` is the
/// `to_height` row filter: terms whose reconstructed parameter exceeds it are
/// dropped, matching the retained-set restriction of the upstream inversion
/// algorithm (KL polynomials vanish across a height step, so filtering the
/// final terms coincides with inverting the restricted dual matrix).
fn kl_sum_at_s_terms(
    parameter: &ParamValue,
    span: SourceSpan,
    height_bound: Option<u32>,
) -> Result<Vec<(SplitValue, StandardRepr)>, Diagnostic> {
    test_standard(parameter, "Cannot compute Kazhdan-Lusztig sum", span)?;
    test_final(parameter, "Cannot compute Kazhdan-Lusztig sum", span)?;
    let rc = rep_context(&parameter.context);
    let normalised = parameter
        .repr
        .normalised(&rc)
        .map_err(|error| structure_diagnostic(error, span))?;
    let located = parameter
        .context
        .rep
        .lookup(&normalised)
        .map_err(|error| structure_diagnostic(error, span))?;
    if !located.has_identity_generator_attitude() {
        return Err(structure_diagnostic(
            StructureError::NotYetImplemented {
                feature: "KL_sum_at_s on a non-identity integral-subsystem attitude",
            },
            span,
        ));
    }
    let block = located.block();
    let z = located.raw_row();
    let common = CommonContext::integral(&rc, located.adapted_representative().gamma_lambda())
        .map_err(|error| structure_diagnostic(error, span))?;
    let singular_flags = common
        .singular_flags(located.prepared_query().gamma())
        .map_err(|error| structure_diagnostic(error, span))?;
    // contributions (repr.cpp:1861-1898): expand rows 0..=z into final
    // rows for the singular system, following the first singular descent.
    let mut contrib: Vec<Vec<(usize, i32)>> = vec![Vec::new(); z + 1];
    for row in 0..=z {
        let mut is_final = true;
        for (s, &is_singular) in singular_flags.iter().enumerate().take(block.rank()) {
            if !is_singular {
                continue;
            }
            match block.descent(row, s) {
                Some(descent) if descent.is_descent() => {
                    is_final = false;
                    match descent {
                        BlockDescent::ComplexDescent => {
                            let target = block
                                .cross(s, row)
                                .ok_or(StructureError::RepInvariantViolation {
                                    invariant: "contributions complex cross",
                                })
                                .map_err(|error| structure_diagnostic(error, span))?;
                            contrib[row] = contrib[target].clone();
                        }
                        BlockDescent::RealTypeII => {
                            let target = block
                                .cayley(s, row)
                                .and_then(|pair| pair.0)
                                .ok_or(StructureError::RepInvariantViolation {
                                    invariant: "contributions inverse Cayley",
                                })
                                .map_err(|error| structure_diagnostic(error, span))?;
                            contrib[row] = contrib[target].clone();
                        }
                        BlockDescent::RealTypeI => {
                            let pair = block
                                .cayley(s, row)
                                .ok_or(StructureError::RepInvariantViolation {
                                    invariant: "contributions inverse Cayley pair",
                                })
                                .map_err(|error| structure_diagnostic(error, span))?;
                            let first = pair
                                .0
                                .ok_or(StructureError::RepInvariantViolation {
                                    invariant: "contributions inverse Cayley first",
                                })
                                .map_err(|error| structure_diagnostic(error, span))?;
                            let second = pair
                                .1
                                .ok_or(StructureError::RepInvariantViolation {
                                    invariant: "contributions inverse Cayley second",
                                })
                                .map_err(|error| structure_diagnostic(error, span))?;
                            contrib[row] = combine_contributions(&contrib[first], &contrib[second]);
                        }
                        // ImaginaryCompact: leave the row's expansion empty.
                        _ => {}
                    }
                    break;
                }
                _ => {}
            }
        }
        if is_final {
            contrib[row] = vec![(row, 1)];
        }
    }
    let mut terms: Vec<(SplitValue, StandardRepr)> = Vec::new();
    located
        .with_kl_table(|kl_table| {
            kl_table.fill(z + 1)?;
            let z_length = block
                .length(z)
                .ok_or(StructureError::RepInvariantViolation {
                    invariant: "KL column target length",
                })?;
            for x in (0..=z).rev() {
                let index = kl_table.kl_pol(x, z)?;
                let pol =
                    kl_table
                        .pool()
                        .get(index)
                        .ok_or(StructureError::RepInvariantViolation {
                            invariant: "representation KL polynomial pool index",
                        })?;
                if pol.is_zero() {
                    continue;
                }
                // Evaluate at q = s by Horner (repr.cpp:2151-2153), the
                // coefficients stored least degree first.
                let mut eval = SplitValue::new(0, 0);
                let s_value = SplitValue::new(0, 1);
                let mut d = pol.degree() + 1;
                while d > 0 {
                    d -= 1;
                    eval = eval
                        .mul(s_value)
                        .add(SplitValue::new(pol.coefficient(d), 0));
                }
                let x_length = block
                    .length(x)
                    .ok_or(StructureError::RepInvariantViolation {
                        invariant: "KL column source length",
                    })?;
                if (z_length - x_length) % 2 != 0 {
                    eval = eval.neg();
                }
                if eval.is_zero() {
                    continue;
                }
                for &(row, coefficient) in &contrib[x] {
                    let repr = located_row_parameter(&parameter.context, &located, row)?;
                    if height_bound.is_none_or(|bound| repr.height() <= bound) {
                        merge_pol_term(&mut terms, eval.mul(SplitValue::new(coefficient, 0)), repr);
                    }
                }
            }
            Ok(())
        })
        .map_err(|error| structure_diagnostic(error, span))?;
    sort_parampol_terms(&mut terms);
    Ok(terms)
}

/// `Block_base::finals_for` over the representation table's common-block
/// topology.  This is kept separate from the classic `BlockGraph` adapter so
/// the first caller-routing slice does not perturb the still-independent
/// block builtins.
fn partial_block_finals_for(
    block: &PartialBlock,
    z: usize,
    singular: u32,
) -> Result<Vec<usize>, StructureError> {
    let mut result = Vec::new();
    let mut z = z;
    loop {
        let mut descended = false;
        for s in 0..block.rank() {
            if singular & (1 << s) == 0 {
                continue;
            }
            match block.descent(z, s) {
                // `finals_for` may have accumulated the first branch of an
                // earlier real type-I descent before the second branch
                // reaches a compact imaginary wall; only that branch
                // vanishes (blocks.cpp:169-201).
                Some(BlockDescent::ImaginaryCompact) => return Ok(result),
                Some(BlockDescent::ComplexDescent) => {
                    z = block
                        .cross(s, z)
                        .ok_or(StructureError::RepInvariantViolation {
                            invariant: "finals complex cross",
                        })?;
                    descended = true;
                    break;
                }
                Some(BlockDescent::RealTypeII) => {
                    z = block.cayley(s, z).and_then(|pair| pair.0).ok_or(
                        StructureError::RepInvariantViolation {
                            invariant: "finals inverse Cayley",
                        },
                    )?;
                    descended = true;
                    break;
                }
                Some(BlockDescent::RealTypeI) => {
                    let pair = block
                        .cayley(s, z)
                        .ok_or(StructureError::RepInvariantViolation {
                            invariant: "finals inverse Cayley pair",
                        })?;
                    match pair {
                        (Some(z0), Some(z1)) => {
                            result.extend(partial_block_finals_for(block, z0, singular)?);
                            z = z1;
                            descended = true;
                            break;
                        }
                        (Some(z0), None) => {
                            result.extend(partial_block_finals_for(block, z0, singular)?);
                            return Ok(result);
                        }
                        (None, _) => return Ok(result),
                    }
                }
                _ => {}
            }
        }
        if !descended {
            result.push(z);
            return Ok(result);
        }
    }
}

/// `Block_base::finals_for` (blocks.cpp:169-201): the survivors reached by
/// descending through the singular generators' descents from `z`; empty
/// when an ImaginaryCompact descent is met (the module vanishes).
fn block_finals_for(
    block: &BlockValue,
    z: usize,
    singular: u32,
    kl_table: &KlTable<'_>,
    span: SourceSpan,
) -> Result<Vec<usize>, Diagnostic> {
    let mut result = Vec::new();
    let mut z = z;
    let rank = kl_table.support().rank();
    loop {
        let mut descended = false;
        for s in 0..rank {
            if singular & (1 << s) == 0 {
                continue;
            }
            match block.graph.descent_value(z, s) {
                Some(BlockDescent::ImaginaryCompact) => {
                    return Ok(Vec::new());
                }
                Some(BlockDescent::ComplexDescent) => {
                    z = block
                        .graph
                        .cross(z, s)
                        .ok_or_else(|| runtime(span, "finals cross"))?;
                    descended = true;
                    break;
                }
                Some(BlockDescent::RealTypeII) => {
                    z = block
                        .graph
                        .inverse_cayley(z, s)
                        .and_then(|pair| pair.0)
                        .ok_or_else(|| runtime(span, "finals inverse Cayley"))?;
                    descended = true;
                    break;
                }
                Some(BlockDescent::RealTypeI) => {
                    let pair = block
                        .graph
                        .inverse_cayley(z, s)
                        .ok_or_else(|| runtime(span, "finals inverse"))?;
                    match pair {
                        (Some(z0), Some(z1)) => {
                            result.extend(block_finals_for(block, z0, singular, kl_table, span)?);
                            z = z1;
                            descended = true;
                            break;
                        }
                        (Some(z0), None) => {
                            result.extend(block_finals_for(block, z0, singular, kl_table, span)?);
                            return Ok(result);
                        }
                        (None, _) => {
                            return Ok(result);
                        }
                    }
                }
                _ => {}
            }
        }
        if !descended {
            result.push(z);
            return Ok(result);
        }
    }
}

/// The members of a parameter's common block: the fibred-product
/// closure of the parameter's block element under all cross/Cayley/
/// inverse-Cayley transforms (blocks.cpp:740-1030 z_pool), with parity
/// real type-I descents filtered like the oracle. The full srm
/// gamma-lambda matching (which distinguishes mid-block singular
/// parameters such as A2 x=3) needs the srm pool layer.
fn common_block_members(
    block: &BlockValue,
    z0: usize,
    rc: &RepContext<'_>,
    lambda_rho: &Weight,
    gamma: &RationalWeight,
    span: SourceSpan,
) -> Result<Vec<bool>, Diagnostic> {
    let size = block.graph.size();
    let rank = block.graph.rank();
    let mut closed = vec![false; size];
    let mut stack = vec![z0];
    closed[z0] = true;
    while let Some(z) = stack.pop() {
        let z_x = block.graph.x(z).expect("in-range");
        for s in 0..rank {
            match block.graph.descent_value(z, s) {
                Some(BlockDescent::ComplexAscent) | Some(BlockDescent::ComplexDescent) => {
                    if let Some(target) = block.graph.cross(z, s) {
                        if !closed[target] {
                            closed[target] = true;
                            stack.push(target);
                        }
                    }
                }
                Some(BlockDescent::ImaginaryTypeI) | Some(BlockDescent::ImaginaryTypeII) => {
                    if let Some(pair) = block.graph.cayley(z, s) {
                        for target in [pair.0, pair.1].into_iter().flatten() {
                            if !closed[target] {
                                closed[target] = true;
                                stack.push(target);
                            }
                        }
                    }
                }
                Some(BlockDescent::ImaginaryCompact) => {
                    if let Some(pair) = block.graph.inverse_cayley(z, s) {
                        for target in [pair.0, pair.1].into_iter().flatten() {
                            if !closed[target] {
                                closed[target] = true;
                                stack.push(target);
                            }
                        }
                    }
                }
                Some(BlockDescent::RealTypeI) => {
                    if rc
                        .is_parity(s, z_x, lambda_rho, gamma)
                        .map_err(|error| structure_diagnostic(error, span))?
                    {
                        if let Some(pair) = block.graph.inverse_cayley(z, s) {
                            for target in [pair.0, pair.1].into_iter().flatten() {
                                if !closed[target] {
                                    closed[target] = true;
                                    stack.push(target);
                                }
                            }
                        }
                    }
                }
                Some(BlockDescent::RealTypeII) => {
                    // Upstream gates the real descents of both flavors on
                    // the parity condition (blocks.cpp:914, `down_Cayley`).
                    let parity = rc
                        .is_parity(s, z_x, lambda_rho, gamma)
                        .map_err(|error| structure_diagnostic(error, span))?;
                    if parity {
                        if let Some(first) =
                            block.graph.inverse_cayley(z, s).and_then(|pair| pair.0)
                        {
                            if !closed[first] {
                                closed[first] = true;
                                stack.push(first);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    Ok(closed)
}

/// `test_standard` (atlas-types.w:6605-6611): reject a non-standard
/// parameter with the oracle's two-line diagnostic; `descr` is
/// "Cannot generate block" or "Cannot generate extended block".
fn test_standard(parameter: &ParamValue, descr: &str, span: SourceSpan) -> Result<(), Diagnostic> {
    let rc = rep_context(&parameter.context);
    let standard = parameter
        .repr
        .is_standard(&rc)
        .map_err(|error| structure_diagnostic(error, span))?;
    if standard {
        return Ok(());
    }
    let shown = DomainValue::Param(parameter.clone()).to_string();
    Err(runtime(
        span,
        format!("{descr}:\n  {shown}\n  Parameter not standard"),
    ))
}

/// `test_final` for module parameters (atlas-types.w:6632-6647): reject a
/// non-final parameter with the oracle's two-line diagnostic. Unlike
/// `test_standard` there is no "not standard" case — the reason chain is
/// dominant, then normal, then nonzero, then semifinal.
fn test_final(parameter: &ParamValue, descr: &str, span: SourceSpan) -> Result<(), Diagnostic> {
    let rc = rep_context(&parameter.context);
    let repr = &parameter.repr;
    let reason = if !repr
        .is_dominant(&rc)
        .map_err(|error| structure_diagnostic(error, span))?
    {
        "not dominant"
    } else if !repr
        .is_normal(&rc)
        .map_err(|error| structure_diagnostic(error, span))?
    {
        "not normal"
    } else if !repr
        .is_nonzero(&rc)
        .map_err(|error| structure_diagnostic(error, span))?
    {
        "not nonzero"
    } else if !repr
        .is_semifinal(&rc)
        .map_err(|error| structure_diagnostic(error, span))?
    {
        "not semifinal"
    } else {
        return Ok(());
    };
    let shown = DomainValue::Param(parameter.clone()).to_string();
    Err(runtime(
        span,
        format!("{descr}:\n  {shown}\n  Parameter is {reason}"),
    ))
}

fn parameter_integrality_rank(
    parameter: &ParamValue,
    span: SourceSpan,
) -> Result<usize, Diagnostic> {
    let rc = rep_context(&parameter.context);
    IntegralSubsystem::integral(rc.root_system(), parameter.repr.gamma())
        .map(|subsystem| subsystem.rank())
        .map_err(|error| structure_diagnostic(error, span))
}

fn parameter_generator(
    parameter: &ParamValue,
    value: &Value,
    span: SourceSpan,
) -> Result<usize, Diagnostic> {
    let integer = as_integer(value, span)?;
    let narrowed = i32::try_from(&integer)
        .map_err(|_| runtime(span, "Integer value to big for conversion"))?;
    let rank = parameter_integrality_rank(parameter, span)?;
    if (narrowed as u32) >= rank as u32 {
        return Err(runtime(
            span,
            format!("Illegal simple reflection: {narrowed}, should be <{rank}"),
        ));
    }
    Ok(narrowed as usize)
}

fn validate_from_dominant(arguments: &[Value], span: SourceSpan) -> Result<(), Diagnostic> {
    arity("from_dominant", arguments, 2, span)?;
    let decompose = matches!(&arguments[0], Value::Domain(DomainValue::RootDatum(_)));
    let vector = if decompose {
        &arguments[1]
    } else {
        &arguments[0]
    };
    let size = match vector {
        Value::Vector(Vec32(entries)) => entries.len(),
        Value::List(entries) => entries.len(),
        other => return Err(type_error(span, format!("expected a vec, found {other}"))),
    };
    let datum = if decompose {
        as_root_datum(&arguments[0], span)?
    } else {
        as_root_datum(&arguments[1], span)?
    };
    let rank = datum.datum.lattice_rank();
    if size != rank {
        return Err(runtime(
            span,
            if decompose {
                format!("Rank and weight size mismatch {rank}:{size}")
            } else {
                format!("Coweight size and rank mismatch {size}:{rank}")
            },
        ));
    }
    Ok(())
}

fn validate_kl_column(parameter: &ParamValue, span: SourceSpan) -> Result<(), Diagnostic> {
    test_standard(parameter, "Cannot compute Kazhdan-Lusztig column", span)?;
    test_final(parameter, "Cannot compute Kazhdan-Lusztig column", span)
}

fn kl_column_candidate_rows(raw_y: usize) -> std::ops::RangeInclusive<usize> {
    0..=raw_y
}

/// The `(1-delta)*gamma == 0` test of the ext-block wrappers
/// (atlas-types.w:7372, 8697; ext_kl.cpp:945), at numerator level.
fn involution_fixes_gamma(matrix: &[Vec<i32>], gamma: &RationalWeight) -> bool {
    let numerator = gamma.numerator();
    numerator.iter().enumerate().all(|(row, &entry)| {
        let image: i64 = matrix[row]
            .iter()
            .zip(numerator.iter())
            .map(|(&factor, &coordinate)| i64::from(factor) * coordinate)
            .sum();
        image == entry
    })
}

/// Whether `gamma` pairs integrally with every simple coroot. Upstream
/// builds the common block on the integral subsystem when this fails
/// (common_context, repr.cpp:2666-2670); that slice is deferred, so the
/// ext-block wrappers stop at this gate instead.
fn gamma_is_integral(datum: &BasedRootDatum, gamma: &RationalWeight) -> bool {
    let numerator = gamma.numerator();
    let denominator = gamma.denominator();
    datum.simple_coroots().iter().all(|coroot| {
        let pairing: i64 = coroot
            .as_slice()
            .iter()
            .zip(numerator.iter())
            .map(|(&c, &g)| i64::from(c) * g)
            .sum();
        pairing % denominator == 0
    })
}

/// The common block's per-element `gamma_lambda` field, computed
/// directly from each element's `(x, y)` pair: the dual KGB element's
/// Tits torus bits are packed into the torsion part of `x`'s involution
/// (`InvolutionTable::y_pack`, involutions.h:211), giving the
/// `StandardReprMod` value `gamma_lambda(x, y_bits, gamma)`
/// (repr.cpp:206-218), reduced to its canonical representative modulo
/// the `(1-theta)` image (`InvolutionTable::real_unique`,
/// involutions.cpp:334-342) exactly as upstream's `z_pool` stores it
/// (blocks.cpp:935,1013, StandardReprMod::build repr.cpp:61-67).
fn common_block_gamma_lambdas(
    block: &BlockValue,
    members: &[bool],
    rc: &RepContext<'_>,
    gamma: &RationalWeight,
    span: SourceSpan,
) -> Result<Vec<Option<RationalWeight>>, Diagnostic> {
    let size = block.graph.size();
    let mut gl: Vec<Option<RationalWeight>> = vec![None; size];
    for z in 0..size {
        if !members[z] {
            continue;
        }
        let x = block.graph.x(z).expect("in-range block element");
        let y = block.graph.y(z).expect("in-range block element");
        let dual_bits = block
            .dual_rf
            .graph
            .element(y)
            .ok_or_else(|| runtime(span, "dual KGB element out of range"))?
            .torus_bits();
        let y_bits = rc
            .torus_part(x, dual_bits)
            .map_err(|error| structure_diagnostic(error, span))?;
        let mut value = rc
            .gamma_lambda(x, &y_bits, gamma)
            .map_err(|error| structure_diagnostic(error, span))?;
        let involution = rc
            .involution_of(x)
            .map_err(|error| structure_diagnostic(error, span))?;
        rc.real_unique(involution, &mut value)
            .map_err(|error| structure_diagnostic(error, span))?;
        gl[z] = Some(value);
    }
    Ok(gl)
}

/// The common block's per-element `gamma_lambda` values (upstream's
/// `z_pool`, blocks.cpp:733-1076), reconstructed with
/// `common_context::cross/down_Cayley/up_Cayley` (repr.cpp:2694-2776)
/// specialised to a full integral subsystem: for integral `gamma` the
/// conjugation words are trivial, so the `pos_to_neg` corrections vanish
/// except the simple-root term of `cross`. Valid only for integral
/// `gamma`; the caller gates on `gamma_is_integral`.
///
/// The propagation mirrors the full block constructor: the seed srm is
/// moved up to the most split fiber (step 2, blocks.cpp:760-786), the top
/// involution packet is saturated through real cross links (step 3), and
/// values flow downward through complex cross descents and parity-gated
/// real Cayley descents (step 4). Within an involution packet the value
/// depends only on the y-column (the bundle alignment invariant,
/// blocks.cpp:857-870), so every assignment is spread across its column.
/// Every value is normalised per involution with `StandardReprMod::build`
/// (repr.cpp:61-67), so revisits must agree; a disagreement or an
/// unassigned element means the port is wrong and is reported rather than
/// silently emitting wrong parameters.
fn common_block_srms(
    block: &BlockValue,
    z0: usize,
    rc: &RepContext<'_>,
    lambda_rho: &Weight,
    gamma: &RationalWeight,
    span: SourceSpan,
) -> Result<Vec<Option<RationalWeight>>, Diagnostic> {
    let size = block.graph.size();
    let rank = block.graph.rank();
    let datum = rc.datum();
    let system = rc.inner_class().root_system();
    let x_of = |z: usize| block.graph.x(z).expect("in-range block element");
    let coroot_of = |s: usize| datum.simple_coroots()[s].as_slice().to_vec();
    // `<g, alpha_s^vee>`, exact at numerator level.
    let eval_at = |g: &RationalWeight, s: usize| -> Result<i64, Diagnostic> {
        let numerator: i64 = coroot_of(s)
            .iter()
            .zip(g.numerator().iter())
            .map(|(&c, &coord)| i64::from(c) * coord)
            .sum();
        if numerator % g.denominator() != 0 {
            return Err(runtime(
                span,
                "non-integral coroot evaluation in common block",
            ));
        }
        Ok(numerator / g.denominator())
    };
    // `root_sum(pos_to_neg) ∩ real` correction check plus
    // `subsys().simple_reflect(s, numerator)`, both with trivial
    // conjugation: `reflect_s(g - [alpha_s if real at x])`
    // (repr.cpp:2694-2709).
    let cross_value =
        |x: KgbId, s: usize, g: &RationalWeight| -> Result<RationalWeight, Diagnostic> {
            let mut shifted = g.clone();
            let simple_id = system
                .id_of(&datum.simple_roots()[s])
                .ok_or_else(|| runtime(span, "simple root missing from root system"))?;
            let reals = rc
                .positive_real_roots_at(x)
                .map_err(|error| structure_diagnostic(error, span))?;
            if reals.contains(&simple_id) {
                let alpha = RationalWeight::from_weight(&datum.simple_roots()[s])
                    .map_err(|error| structure_diagnostic(error, span))?;
                shifted = shifted
                    .sub(&alpha)
                    .map_err(|error| structure_diagnostic(error, span))?;
            }
            let eval = eval_at(&shifted, s)?;
            let alpha = datum.simple_roots()[s].as_slice();
            let numerator = shifted
                .numerator()
                .iter()
                .zip(alpha.iter())
                .map(|(&coord, &a)| coord - eval * i64::from(a))
                .collect();
            RationalWeight::new(numerator, shifted.denominator())
                .map_err(|error| structure_diagnostic(error, span))
        };
    // `common_context::is_parity` (repr.cpp:2712-2723) for a simple
    // integral generator that is real at `x`.
    let is_parity = |x: KgbId, s: usize, g: &RationalWeight| -> Result<bool, Diagnostic> {
        let eval = eval_at(g, s)?;
        let reals = rc
            .positive_real_roots_at(x)
            .map_err(|error| structure_diagnostic(error, span))?;
        let two_rho_real = two_rho(system, &reals);
        let corr_twice: i64 = two_rho_real
            .as_slice()
            .iter()
            .zip(coroot_of(s).iter())
            .map(|(&coord, &c)| i64::from(coord) * i64::from(c))
            .sum();
        if corr_twice % 2 != 0 {
            return Err(runtime(span, "rho_r parity correction not integral"));
        }
        Ok((eval + corr_twice / 2) % 2 != 0)
    };
    // The `up_Cayley` parity correction (repr.cpp:2744-2776): when
    // `<gamma_lambda, alpha_s^vee> + rho_r_corr` is even, add `alpha_s/2`.
    let up_cayley_value =
        |x2: KgbId, s: usize, g: &RationalWeight| -> Result<RationalWeight, Diagnostic> {
            let reals = rc
                .positive_real_roots_at(x2)
                .map_err(|error| structure_diagnostic(error, span))?;
            let two_rho_real = two_rho(system, &reals);
            let corr_twice: i64 = two_rho_real
                .as_slice()
                .iter()
                .zip(coroot_of(s).iter())
                .map(|(&coord, &c)| i64::from(coord) * i64::from(c))
                .sum();
            if corr_twice % 2 != 0 {
                return Err(runtime(span, "up Cayley rho_r correction not integral"));
            }
            let eval = eval_at(g, s)?;
            if (eval + corr_twice / 2) % 2 == 0 {
                let alpha = datum.simple_roots()[s].as_slice();
                let numerator = g
                    .numerator()
                    .iter()
                    .zip(alpha.iter())
                    .map(|(&coord, &a)| 2 * coord + g.denominator() * i64::from(a))
                    .collect();
                RationalWeight::new(numerator, 2 * g.denominator())
                    .map_err(|error| structure_diagnostic(error, span))
            } else {
                Ok(g.clone())
            }
        };
    // Seed: `StandardReprMod::mod_reduce` (repr.cpp:58-67): gamma-lambda-rho
    // made real_unique at the seed involution and normalised. This equals
    // `z_pool[z0]` because the block modifier is trivial for an integral
    // dominant gamma (repr.cpp:1773-1794 with identity locator).
    let seed = {
        let lambda = RationalWeight::from_weight(lambda_rho)
            .map_err(|error| structure_diagnostic(error, span))?;
        let diff = gamma
            .sub(rc.rho())
            .and_then(|value| value.sub(&lambda))
            .map_err(|error| structure_diagnostic(error, span))?;
        rc.build_srm(x_of(z0), &diff)
            .map_err(|error| structure_diagnostic(error, span))?
    };
    // Step 2 (blocks.cpp:760-786): move up towards the most split fiber,
    // through complex ascents and imaginary noncompact Cayleys.
    let mut z_top = z0;
    let mut g_top = seed;
    'move_up: loop {
        for s in 0..rank {
            match block.graph.descent_value(z_top, s) {
                Some(BlockDescent::ComplexAscent) => {
                    let target = block
                        .graph
                        .cross(z_top, s)
                        .ok_or_else(|| runtime(span, "missing cross link in common block"))?;
                    g_top = rc
                        .build_srm(x_of(target), &cross_value(x_of(z_top), s, &g_top)?)
                        .map_err(|error| structure_diagnostic(error, span))?;
                    z_top = target;
                    continue 'move_up;
                }
                Some(BlockDescent::ImaginaryTypeI) | Some(BlockDescent::ImaginaryTypeII) => {
                    if let Some((Some(target), _)) = block.graph.cayley(z_top, s) {
                        let x2 = x_of(target);
                        g_top = rc
                            .build_srm(x2, &up_cayley_value(x2, s, &g_top)?)
                            .map_err(|error| structure_diagnostic(error, span))?;
                        z_top = target;
                        continue 'move_up;
                    }
                }
                _ => {}
            }
        }
        break;
    }
    // Involution packets with y-columns: within a packet (elements whose x
    // share the involution) the rows correspond to distinct x values and
    // the columns to distinct y values; `gamma_lambda` is constant along
    // each column (blocks.cpp:857-870).
    let mut packets: std::collections::BTreeMap<InvolutionId, Vec<usize>> =
        std::collections::BTreeMap::new();
    for z in 0..size {
        let involution = rc
            .involution_of(x_of(z))
            .map_err(|error| structure_diagnostic(error, span))?;
        packets.entry(involution).or_default().push(z);
    }
    let mut column_of = vec![usize::MAX; size];
    let mut packet_columns: Vec<Vec<Vec<usize>>> = Vec::new();
    let mut packet_of = vec![usize::MAX; size];
    for elements in packets.into_values() {
        let packet = packet_columns.len();
        let mut ys: Vec<usize> = elements
            .iter()
            .map(|&z| block.graph.y(z).expect("in-range block element").index())
            .collect();
        ys.sort_unstable();
        ys.dedup();
        let mut columns: Vec<Vec<usize>> = vec![Vec::new(); ys.len()];
        for z in elements {
            let y = block.graph.y(z).expect("in-range block element").index();
            let column = ys
                .binary_search(&y)
                .map_err(|_| runtime(span, "packet y lookup failed"))?;
            column_of[z] = column;
            packet_of[z] = packet;
            columns[column].push(z);
        }
        packet_columns.push(columns);
    }
    let mut gl: Vec<Option<RationalWeight>> = vec![None; size];
    let mut assignments: Vec<(usize, RationalWeight)> = vec![(z_top, g_top)];
    let mut edge_queue: Vec<usize> = Vec::new();
    loop {
        while let Some((z, raw)) = assignments.pop() {
            for &mate in &packet_columns[packet_of[z]][column_of[z]] {
                let normalised = rc
                    .build_srm(x_of(mate), &raw)
                    .map_err(|error| structure_diagnostic(error, span))?;
                match &gl[mate] {
                    Some(previous) if *previous == normalised => {}
                    Some(_) => {
                        return Err(runtime(
                            span,
                            "inconsistent gamma_lambda propagation in common block",
                        ))
                    }
                    None => {
                        gl[mate] = Some(normalised);
                        edge_queue.push(mate);
                    }
                }
            }
        }
        let Some(z) = edge_queue.pop() else {
            break;
        };
        let g = gl[z].clone().expect("assigned element");
        let x_z = x_of(z);
        for s in 0..rank {
            match block.graph.descent_value(z, s) {
                // Step 4 cross into a packet below (blocks.cpp:918-944).
                Some(BlockDescent::ComplexDescent) => {
                    if let Some(target) = block.graph.cross(z, s) {
                        assignments.push((target, cross_value(x_z, s, &g)?));
                    }
                }
                Some(descent @ (BlockDescent::RealTypeI | BlockDescent::RealTypeII)) => {
                    // Real reflection cross within the packet (step 3's
                    // fiber saturation, blocks.cpp:800-840); fixed for
                    // type I, moving to another column for type II.
                    if let Some(target) = block.graph.cross(z, s) {
                        if target != z {
                            assignments.push((target, cross_value(x_z, s, &g)?));
                        }
                    }
                    // Parity-gated down Cayley (blocks.cpp:1037-1058).
                    if is_parity(x_z, s, &g)? {
                        if let Some(pair) = block.graph.inverse_cayley(z, s) {
                            if let Some(first) = pair.0 {
                                // `down_Cayley` leaves gamma_lambda
                                // unchanged (repr.cpp:2724-2742 with
                                // trivial conjugation).
                                assignments.push((first, g.clone()));
                                if descent == BlockDescent::RealTypeI {
                                    if let Some(second) = pair.1 {
                                        // The type I second image is the
                                        // cross of the first (blocks.cpp
                                        // :1047-1052).
                                        let first_normalised = rc
                                            .build_srm(x_of(first), &g)
                                            .map_err(|error| structure_diagnostic(error, span))?;
                                        assignments.push((
                                            second,
                                            cross_value(x_of(first), s, &first_normalised)?,
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    if gl.iter().any(Option::is_none) {
        return Err(runtime(
            span,
            "common block propagation left elements unassigned",
        ));
    }
    Ok(gl)
}

/// `gamma_rho.integer_diff<int>` (atlas-types.w:7402): the exact integral
/// difference of two rational weights, as a `Weight`.
fn integer_diff_weight(
    gamma_rho: &RationalWeight,
    gl: &RationalWeight,
    span: SourceSpan,
) -> Result<Weight, Diagnostic> {
    let diff = gamma_rho
        .sub(gl)
        .map_err(|e| structure_diagnostic(e, span))?;
    if diff.denominator() != 1 {
        return Err(runtime(span, "non-integral lambda shift in common block"));
    }
    Ok(Weight::new(
        diff.numerator().iter().map(|&v| v as i32).collect(),
    ))
}

/// The ext-block fiber over the common block: the extended elements whose
/// parent block element is a common-block member, in extended order
/// (ext_block.cpp:630-635 collects the delta-fixed points; intersecting
/// with the common block mirrors upstream's `common_block::extended_block`
/// built directly over the common block). Returns the fiber list and the
/// old-to-fiber index map.
fn ext_fiber(eb: &ExtBlock, members: &[bool]) -> (Vec<usize>, Vec<usize>) {
    let mut loc = vec![usize::MAX; eb.size()];
    let mut fiber = Vec::new();
    for n in 0..eb.size() {
        if members[eb.z(n)] {
            loc[n] = fiber.len();
            fiber.push(n);
        }
    }
    (fiber, loc)
}

/// `M.rowOperation(target, source, c)`: `row[target] += c * row[source]`
/// (mirrors the private helper of `ext_kl::condense`, ext_kl.cpp:2031).
fn kl_row_operation(m: &mut [Vec<KlPol>], target: usize, source: usize, c: i32) {
    debug_assert!(target != source);
    let (target_row, source_row) = if target < source {
        let (low, high) = m.split_at_mut(source);
        (&mut low[target], &high[0])
    } else {
        let (low, high) = m.split_at_mut(target);
        (&mut high[0], &low[source])
    };
    for (entry, contribution) in target_row.iter_mut().zip(source_row.iter()) {
        if !contribution.is_zero() {
            *entry = entry.add(&contribution.scaled(c));
        }
    }
}

/// The distinguished-involution twist data of the parameter's inner class
/// (the `extended_block` wrapper builds over the distinguished involution
/// regardless of the user's delta — atlas-types.w:7392).
fn distinguished_twist(
    parameter: &ParamValue,
    span: SourceSpan,
) -> Result<(LatticeInvolution, Vec<usize>), Diagnostic> {
    let inner_class = &parameter.context.parent.inner_class;
    let delta = inner_class.distinguished_involution().involution().clone();
    let twist = inner_class
        .based_involution_twist(delta.clone())
        .map_err(|error| match error {
            StructureError::InvalidBasedAutomorphism => runtime(
                span,
                "Root datum involution is not distinguished".to_string(),
            ),
            other => structure_diagnostic(other, span),
        })?;
    Ok((delta, twist))
}

/// Build the extended block over the parameter's full block with the given
/// twist data (ext_block::ext_block constructor, ext_block.cpp:618-668).
fn build_ext_block(
    block: &BlockValue,
    parameter: &ParamValue,
    delta: &LatticeInvolution,
    twist: &[usize],
    span: SourceSpan,
) -> Result<ExtBlock, Diagnostic> {
    let dual_parent = &block.dual_rf.parent;
    let matrix = delta.weight_matrix();
    let dual_delta = LatticeInvolution::new(
        dual_parent.inner_class.datum(),
        transpose(matrix),
        matrix.to_vec(),
    )
    .map_err(|e| structure_diagnostic(e, span))?;
    let dual_twist = dual_parent
        .inner_class
        .based_involution_twist(dual_delta.clone())
        .map_err(|e| structure_diagnostic(e, span))?;
    let cartan = parameter.context.parent.root_datum.datum.cartan_matrix();
    ExtBlock::build(
        &block.graph,
        &block.rf.graph,
        &block.rf.table,
        &block.dual_rf.graph,
        &block.dual_rf.table,
        delta,
        twist,
        &dual_delta,
        &dual_twist,
        cartan,
    )
    .map_err(|e| structure_diagnostic(e, span))
}

/// Integral solution of A x = b (matreduc::find_solution, matreduc.cpp:
/// 403-437): Smith diagonalisation followed by back-substitution. This
/// workspace version uses exact rational arithmetic, which is exact for
/// the small rank-<=8 systems of the ext-block layer.
fn find_solution(matrix: &[Vec<i64>], rhs: &[i64]) -> Result<Vec<i64>, String> {
    let rows = matrix.len();
    let cols = matrix.first().map_or(0, Vec::len);
    // Augmented [A | b] with malachite rationals.
    let mut aug: Vec<Vec<BigRational>> = Vec::new();
    for (row, matrix_row) in matrix.iter().enumerate() {
        let mut aug_row: Vec<BigRational> = matrix_row
            .iter()
            .map(|&entry| BigRational::from(entry))
            .collect();
        aug_row.push(BigRational::from(rhs.get(row).copied().unwrap_or(0)));
        aug.push(aug_row);
    }
    let mut pivot_row = 0usize;
    for column in 0..cols {
        // Find a row at or below pivot_row with a nonzero entry in column.
        let mut chosen = None;
        for (row, aug_row) in aug.iter().enumerate().skip(pivot_row) {
            if aug_row[column] != 0 {
                chosen = Some(row);
                break;
            }
        }
        let Some(chosen) = chosen else { continue };
        aug.swap(pivot_row, chosen);
        let pivot_row_values = aug[pivot_row].clone();
        let pivot = pivot_row_values[column].clone();
        for (row, aug_row) in aug.iter_mut().enumerate() {
            if row == pivot_row {
                continue;
            }
            let factor = aug_row[column].clone() / pivot.clone();
            if factor == 0 {
                continue;
            }
            for (entry, value) in aug_row.iter_mut().enumerate().skip(column) {
                let reduced = value.clone() - factor.clone() * pivot_row_values[entry].clone();
                *value = reduced;
            }
        }
        pivot_row += 1;
        if pivot_row == rows {
            break;
        }
    }
    // Consistency: rows below the pivot block must have zero rhs.
    for aug_row in aug.iter().skip(pivot_row) {
        if aug_row[cols] != 0 {
            return Err("unsolvable integral system".into());
        }
    }
    // Back-substitute: free variables are set to zero.
    let mut solution = vec![0_i64; cols];
    let mut pivot_positions: Vec<usize> = Vec::new();
    for aug_row in aug.iter().take(pivot_row) {
        let position = aug_row.iter().position(|entry| *entry != 0);
        if let Some(column) = position {
            pivot_positions.push(column);
        }
    }
    for (index, &column) in pivot_positions.iter().enumerate() {
        let pivot = aug[index][column].clone();
        let mut value = aug[index][cols].clone();
        for (other, &solution_entry) in solution.iter().enumerate().skip(column + 1) {
            if aug[index][other] != 0 {
                value -= aug[index][other].clone() * BigRational::from(solution_entry);
            }
        }
        let quotient = value / pivot;
        let text = quotient.to_string();
        if text.contains('/') {
            return Err("unsolvable integral system".into());
        }
        solution[column] = text
            .parse()
            .map_err(|_| "solution out of range".to_string())?;
    }
    Ok(solution)
}

/// The transitive closure of a Hasse diagram as a `lesseq` matrix/// The transitive closure of a Hasse diagram as a `lesseq` matrix
/// (poset.cpp:197-229 style, rows are the closure sets).
fn bruhat_closure(hasse: &[Vec<usize>]) -> Vec<Vec<bool>> {
    let n = hasse.len();
    let mut closure: Vec<std::collections::BTreeSet<usize>> = Vec::with_capacity(n);
    for row in hasse {
        let mut cl = std::collections::BTreeSet::new();
        cl.insert(closure.len());
        for &j in row {
            cl.extend(closure[j].iter().copied());
        }
        closure.push(cl);
    }
    closure
        .iter()
        .map(|cl| (0..n).map(|i| cl.contains(&i)).collect())
        .collect()
}

/// The block descent code printed by `block_io::printDescent`
/// (block_io.cpp:373-420): C-, C+, ic, rn, i1, i2, r1, r2.
fn block_descent_code(descent: BlockDescent) -> &'static str {
    match descent {
        BlockDescent::ComplexDescent => "C-",
        BlockDescent::ComplexAscent => "C+",
        BlockDescent::ImaginaryCompact => "ic",
        BlockDescent::RealNonparity => "rn",
        BlockDescent::ImaginaryTypeI => "i1",
        BlockDescent::ImaginaryTypeII => "i2",
        BlockDescent::RealTypeI => "r1",
        BlockDescent::RealTypeII => "r2",
    }
}

/// The reduced word of a Weyl element in ascending-generator order
/// (weyl.cpp greedy left-descent, `W.word`).
fn weyl_reduced_word(inner: &InnerClass, element: &WeylElement) -> Vec<usize> {
    let mut result = Vec::new();
    let mut current = element.clone();
    while !current.is_identity() {
        let mut generator = 0;
        while !current
            .has_left_descent(inner.root_system(), generator)
            .unwrap_or(false)
        {
            generator += 1;
        }
        result.push(generator);
        current = current
            .left_multiply_simple(inner.root_system(), generator)
            .expect("left descent shortens")
            .0;
    }
    result
}

/// The `[..]` descent line of a block element (block_io.cpp:373-420),
/// with generators whose mask bit is unset shown as `* `.
fn block_descent_set(graph: &BlockGraph, z: usize, _rank: usize, mask: &[bool]) -> String {
    let mut out = String::from("[");
    for (s, &visible) in mask.iter().enumerate() {
        if s != 0 {
            out.push(',');
        }
        if !visible {
            out.push_str("* ");
        } else {
            out.push_str(block_descent_code(
                graph.descent_value(z, s).expect("in-range"),
            ));
        }
    }
    out.push(']');
    out
}

/// The sum of the positive roots in `roots` (rootdata.h:746-748 `twoRho`).
fn two_rho(root_system: &RootSystem, roots: &[RootId]) -> Weight {
    let rank = root_system.lattice_rank();
    let mut sum = vec![0_i32; rank];
    for &id in roots {
        if root_system.is_positive(id).unwrap_or(false) {
            if let Some(root) = root_system.root(id) {
                for (entry, &coordinate) in sum.iter_mut().zip(root.as_slice()) {
                    *entry += coordinate;
                }
            }
        }
    }
    Weight::new(sum)
}

/// Whether `weight` pairs trivially with the coroot of `root`.
fn weight_orthogonal(root_system: &RootSystem, weight: &Weight, root: RootId) -> bool {
    let Some(coroot) = root_system.coroot(root) else {
        return false;
    };
    pair(weight, coroot).is_ok_and(|pair| pair == 0)
}

/// `RootSystem::simpleBasis` (rootdata.cpp:621-652), implemented by the
/// positivity test: a positive root is simple in the subsystem spanned by
/// `rs` iff no other positive root of `rs` has strictly smaller coordinates.
/// The simple roots of the integrality subsystem of `gamma`
/// (rootdata.cpp:1483-1501): positive coroots with integral pairing, then
/// the minimal positive roots spanning them.
fn integrality_simples_roots(
    handle: &RootDatumHandle,
    gamma: &RatVec,
    span: SourceSpan,
) -> Result<Vec<RootId>, Diagnostic> {
    let root_system = RootSystem::enumerate(&handle.datum, ROOT_BUDGET)
        .map_err(|error| runtime(span, error.to_string()))?;
    let denominator = gamma.denominator() as i64;
    let mut integral = Vec::new();
    for index in 0..root_system.roots().len() {
        let id = RootId::from_usize(index);
        if !root_system.is_positive(id).unwrap_or(false) {
            continue;
        }
        if let Some(coroot) = root_system.coroot(id) {
            let dot = gamma
                .numerators()
                .iter()
                .zip(coroot.as_slice())
                .map(|(g, &c)| g * i64::from(c))
                .sum::<i64>();
            if dot % denominator == 0 {
                integral.push(id);
            }
        }
    }
    simple_basis(&root_system, &integral).map_err(|error| runtime(span, error.to_string()))
}

/// <gamma, alpha^vee> for a positive coroot (rootdata.cpp `posCoroot(i).dot`).
/// gamma.dot_Q(coroot).mod1() as a fraction s/d in [0,1) (alcoves.cpp:36-41);
/// negative coroots with integral evaluation round up to 1.
fn frac_eval_value(root_system: &RootSystem, id: RootId, gamma: &RatVec) -> Option<(i64, i64)> {
    let coroot = root_system.coroot(id)?;
    let denominator = gamma.denominator() as i64;
    let dot = gamma
        .numerators()
        .iter()
        .zip(coroot.as_slice())
        .map(|(g, &c)| g * i64::from(c))
        .sum::<i64>();
    let mut s = dot.rem_euclid(denominator);
    if s == 0 && !root_system.is_positive(id).unwrap_or(false) {
        s = denominator; // negative coroots round up
    }
    Some((s, denominator))
}

/// Simple-coroot coordinates of a root's coroot: the unique rational
/// solution of `SimpleCoroots * m = coroot`, integral because every coroot
/// is an integral combination of the simple coroots.
fn simple_coroot_coordinates(root_system: &RootSystem, id: RootId) -> Option<Vec<i32>> {
    let datum = root_system.datum();
    let semisimple = datum.semisimple_rank();
    let ambient = datum.lattice_rank();
    let coroot = root_system.coroot(id)?;
    let simple_coroots = datum.simple_coroots();
    let mut aug: Vec<Vec<BigRational>> = (0..ambient)
        .map(|row| {
            let mut line: Vec<BigRational> = (0..semisimple)
                .map(|column| BigRational::from(simple_coroots[column].as_slice()[row]))
                .collect();
            line.push(BigRational::from(coroot.as_slice()[row]));
            line
        })
        .collect();
    let mut pivot_row = 0;
    for column in 0..semisimple {
        let found = (pivot_row..ambient).find(|&row| aug[row][column] != 0)?;
        aug.swap(pivot_row, found);
        let pivot = aug[pivot_row][column].clone();
        for entry in &mut aug[pivot_row] {
            *entry /= &pivot;
        }
        for row in 0..ambient {
            if row == pivot_row || aug[row][column] == 0 {
                continue;
            }
            let factor = aug[row][column].clone();
            let (pivot_line, target) = if row < pivot_row {
                let (head, tail) = aug.split_at_mut(pivot_row);
                (&tail[0], &mut head[row])
            } else {
                let (head, tail) = aug.split_at_mut(row);
                (&head[pivot_row], &mut tail[0])
            };
            for (target_entry, pivot_entry) in target.iter_mut().zip(pivot_line.iter()) {
                let subtracted = pivot_entry.clone() * &factor;
                *target_entry -= subtracted;
            }
        }
        pivot_row += 1;
    }
    // The pivot row of column `c` is row `c`; only after the full
    // Gauss-Jordan sweep is its solution entry final (later column
    // eliminations still rewrite earlier pivot rows).
    let mut coordinates = vec![0i32; semisimple];
    for (column, entry) in coordinates.iter_mut().enumerate() {
        let value = &aug[column][semisimple];
        let denominator = i64::try_from(value.denominator_ref()).ok()?;
        if denominator != 1 {
            return None;
        }
        *entry = i32::try_from(value.numerator_ref()).ok()?;
    }
    Some(coordinates)
}

/// Upstream `RootNbr` numbering for a Rust [`RootSystem`]
/// (rootdata.cpp:131-283): negative roots come first in reverse positive
/// order, positive roots after, ordered by level and then by `root_compare`
/// (simple coordinates compared from the LAST index down), so the simple
/// roots occupy `npos..npos+rank` in generator order. A coroot-preferring
/// datum generates the root system for the dual system first
/// (rootdata.cpp:164-167), so the level and the compared coordinates are
/// the COROOT ones; the B2/G2 oracle dumps confirm this. The Rust
/// enumeration order is lexicographic in ambient coordinates, so every
/// alcove/wall algorithm that mirrors upstream list order goes through
/// this map.
#[derive(Debug)]
struct RootNumbering {
    /// Number of positive roots; a RootNbr below this is a negative root.
    npos: usize,
    /// RootNbr -> Rust root.
    by_nbr: Vec<RootId>,
    /// Rust root index -> RootNbr.
    nbr_of: Vec<usize>,
}

impl RootNumbering {
    fn new(root_system: &RootSystem, prefer_coroots: bool) -> Self {
        let total = root_system.roots().len();
        let sort_coordinates = |id: RootId| -> Vec<i32> {
            if prefer_coroots {
                simple_coroot_coordinates(root_system, id).unwrap_or_default()
            } else {
                root_system.simple_coordinates(id).unwrap_or(&[]).to_vec()
            }
        };
        let mut positives: Vec<RootId> = (0..total)
            .map(RootId::from_usize)
            .filter(|&id| root_system.is_positive(id).unwrap_or(false))
            .collect();
        // (level, root_compare): level is the coordinate sum; root_compare
        // (rootdata.cpp:118-129) compares from the last coordinate down to
        // the first.
        positives.sort_by(|&left, &right| {
            let left_coords = sort_coordinates(left);
            let right_coords = sort_coordinates(right);
            let left_level: i32 = left_coords.iter().sum();
            let right_level: i32 = right_coords.iter().sum();
            left_level.cmp(&right_level).then_with(|| {
                for index in (0..left_coords.len()).rev() {
                    let diff = left_coords[index] - right_coords[index];
                    if diff != 0 {
                        return diff.cmp(&0);
                    }
                }
                Ordering::Equal
            })
        });
        let npos = positives.len();
        let mut pos_index_of: std::collections::BTreeMap<Vec<i32>, usize> =
            std::collections::BTreeMap::new();
        let mut by_nbr = vec![RootId::from_usize(0); total];
        let mut nbr_of = vec![0usize; total];
        for (pos, &id) in positives.iter().enumerate() {
            pos_index_of.insert(
                root_system.simple_coordinates(id).unwrap_or(&[]).to_vec(),
                pos,
            );
            by_nbr[npos + pos] = id;
            nbr_of[id.index()] = npos + pos;
        }
        for (index, slot) in nbr_of.iter_mut().enumerate() {
            let id = RootId::from_usize(index);
            if root_system.is_positive(id).unwrap_or(false) {
                continue;
            }
            let negated: Vec<i32> = root_system
                .simple_coordinates(id)
                .unwrap_or(&[])
                .iter()
                .map(|&value| -value)
                .collect();
            // rootMinus (rootdata.h:264-265): negative of positive root `p`
            // has RootNbr `npos - 1 - p`.
            let pos = *pos_index_of
                .get(&negated)
                .expect("every negative root has a positive counterpart");
            by_nbr[npos - 1 - pos] = id;
            *slot = npos - 1 - pos;
        }
        Self {
            npos,
            by_nbr,
            nbr_of,
        }
    }

    fn id(&self, nbr: usize) -> RootId {
        self.by_nbr[nbr]
    }

    fn nbr(&self, id: RootId) -> usize {
        self.nbr_of[id.index()]
    }

    fn is_negative(&self, nbr: usize) -> bool {
        nbr < self.npos
    }

    /// convert_to_signed_root_index (atlas-types.w:1478-1485).
    fn signed(&self, nbr: usize) -> i64 {
        nbr as i64 - self.npos as i64
    }
}

/// weyl::wall_set (alcoves.cpp:112-138): the coroots defining the walls of
/// the alcove reached from `gamma` by a small dominant displacement.
/// `integrals` collects the walls whose evaluation on `gamma` is integral
/// (upstream `on_wall_coroots`). The filter keeps exactly the roots whose
/// coroot CANNOT be subtracted from the taken coroot — upstream
/// `min_coroots_for` membership (rootdata.h:154-157: "coroots for which
/// coroot |i| cannot be subtracted"), i.e. `alpha^vee - beta^vee` is not
/// itself a coroot.
fn wall_set(
    root_system: &RootSystem,
    numbering: &RootNumbering,
    gamma: &RatVec,
) -> (BTreeSet<usize>, BTreeSet<usize>) {
    let num_roots = root_system.roots().len();
    let coroot_table: BTreeSet<Vec<i32>> = (0..num_roots)
        .filter_map(|index| {
            root_system
                .coroot(RootId::from_usize(index))
                .map(|coroot| coroot.as_slice().to_vec())
        })
        .collect();
    // Initial list in RootNbr order (upstream builds it 0..numRoots).
    let mut levels: Vec<(usize, (i64, i64))> = (0..num_roots)
        .map(|nbr| {
            let level = frac_eval_value(root_system, numbering.id(nbr), gamma)
                .expect("every root has a coroot");
            (nbr, level)
        })
        .collect();
    let mut walls = BTreeSet::new();
    let mut integrals = BTreeSet::new();
    while !levels.is_empty() {
        // get_minima (alcoves.cpp:55-77): stable move of every minimal
        // level to the front, preserving relative order.
        let min_level = levels
            .iter()
            .map(|(_, level)| *level)
            .min()
            .expect("nonempty levels");
        levels.sort_by_key(|(_, level)| (*level != min_level) as u8);
        let mut n_min = levels
            .iter()
            .take_while(|(_, level)| *level == min_level)
            .count();
        while n_min > 0 && !levels.is_empty() {
            let (alpha, level) = levels.remove(0);
            if level.0 == 0 {
                integrals.insert(alpha);
            }
            walls.insert(alpha);
            n_min -= 1;
            // filter_up (alcoves.cpp:81-91): drop the coroots that are not
            // ladder bottoms of alpha's coroot; dropped copies of the
            // minimum count against n_min.
            let alpha_coroot = root_system
                .coroot(numbering.id(alpha))
                .expect("every root has a coroot")
                .as_slice()
                .to_vec();
            let mut kept = Vec::new();
            for item in levels.drain(..) {
                let beta_coroot = root_system
                    .coroot(numbering.id(item.0))
                    .expect("every root has a coroot")
                    .as_slice();
                let difference: Vec<i32> = alpha_coroot
                    .iter()
                    .zip(beta_coroot)
                    .map(|(&a, &b)| a - b)
                    .collect();
                if !coroot_table.contains(&difference) {
                    kept.push(item);
                } else if item.1 == min_level {
                    n_min = n_min.saturating_sub(1);
                }
            }
            levels = kept;
        }
    }
    (walls, integrals)
}

/// rootdata::components (rootdata.cpp:1443-1467): the connected components
/// of a root subset under non-orthogonality, each in RootNbr-ascending
/// order. Component order does not affect the alcove consumers below.
fn root_components(
    root_system: &RootSystem,
    numbering: &RootNumbering,
    walls: &BTreeSet<usize>,
) -> Vec<Vec<usize>> {
    let sorted: Vec<usize> = walls.iter().copied().collect();
    let mut parent: Vec<usize> = (0..sorted.len()).collect();
    fn find(parent: &mut [usize], index: usize) -> usize {
        let mut root = index;
        while parent[root] != root {
            root = parent[root];
        }
        let mut current = index;
        while parent[current] != current {
            let next = parent[current];
            parent[current] = root;
            current = next;
        }
        root
    }
    for left in 0..sorted.len() {
        for right in (left + 1)..sorted.len() {
            let bracket = root_system
                .bracket(numbering.id(sorted[left]), numbering.id(sorted[right]))
                .unwrap_or(0);
            if bracket != 0 {
                let a = find(&mut parent, left);
                let b = find(&mut parent, right);
                if a != b {
                    parent[b] = a;
                }
            }
        }
    }
    let mut components: Vec<Vec<usize>> = Vec::new();
    let mut root_to_component: std::collections::BTreeMap<usize, usize> =
        std::collections::BTreeMap::new();
    for (index, &nbr) in sorted.iter().enumerate() {
        let root = find(&mut parent, index);
        match root_to_component.get(&root) {
            Some(&slot) => components[slot].push(nbr),
            None => {
                root_to_component.insert(root, components.len());
                components.push(vec![nbr]);
            }
        }
    }
    components
}

/// labels_for_component (alcoves.cpp:141-154): the unique primitive
/// integer relation between the coroots of one wall component, made
/// positive. Ambient coroot coordinates carry the same linear relations
/// as upstream's simple-coroot coordinates, so the kernel is computed
/// from the ambient table.
fn labels_for_component(
    root_system: &RootSystem,
    numbering: &RootNumbering,
    component: &[usize],
) -> Result<Vec<i64>, String> {
    let columns: Vec<Vec<i32>> = component
        .iter()
        .map(|&nbr| {
            root_system
                .coroot(numbering.id(nbr))
                .expect("every root has a coroot")
                .as_slice()
                .to_vec()
        })
        .collect();
    let rows = columns.first().map_or(0, Vec::len);
    // Rational Gauss-Jordan elimination on the transpose; the kernel of A
    // is the solution set of RREF(A) x = 0 with one free variable.
    let mut matrix: Vec<Vec<BigRational>> = (0..rows)
        .map(|row| {
            columns
                .iter()
                .map(|column| BigRational::from(column[row]))
                .collect()
        })
        .collect();
    let width = columns.len();
    let mut pivot_of_column = vec![None; width];
    let mut pivot_row = 0;
    for column in 0..width {
        if pivot_row >= rows {
            break;
        }
        let Some(found) = (pivot_row..rows).find(|&row| matrix[row][column] != 0) else {
            continue;
        };
        matrix.swap(pivot_row, found);
        let pivot = matrix[pivot_row][column].clone();
        for entry in &mut matrix[pivot_row] {
            *entry /= &pivot;
        }
        for row in 0..rows {
            if row == pivot_row || matrix[row][column] == 0 {
                continue;
            }
            let factor = matrix[row][column].clone();
            let (pivot_line, target) = if row < pivot_row {
                let (head, tail) = matrix.split_at_mut(pivot_row);
                (&tail[0], &mut head[row])
            } else {
                let (head, tail) = matrix.split_at_mut(row);
                (&head[pivot_row], &mut tail[0])
            };
            for (target_entry, pivot_entry) in target.iter_mut().zip(pivot_line.iter()) {
                let subtracted = pivot_entry.clone() * &factor;
                *target_entry -= subtracted;
            }
        }
        pivot_of_column[column] = Some(pivot_row);
        pivot_row += 1;
    }
    let free: Vec<usize> = (0..width)
        .filter(|&col| pivot_of_column[col].is_none())
        .collect();
    if free.len() != 1 {
        return Err(format!(
            "alcove wall component has {} coroot relations, expected 1",
            free.len()
        ));
    }
    let mut relation: Vec<BigRational> = vec![BigRational::from(0); width];
    relation[free[0]] = BigRational::from(1);
    for (col, pivot) in pivot_of_column.iter().enumerate() {
        if let Some(row) = pivot {
            relation[col] = -matrix[*row][free[0]].clone();
        }
    }
    // Clear denominators, divide out the gcd, and flip so the first entry
    // is positive (upstream asserts it is nonzero and negates if needed).
    let mut denominator = 1i64;
    for entry in &relation {
        let den = i64::try_from(entry.denominator_ref())
            .map_err(|_| "coroot relation denominator overflow".to_string())?;
        denominator = lcm(denominator, den);
    }
    let mut integral: Vec<i64> = relation
        .iter()
        .map(|entry| {
            let scaled = entry * BigRational::from(denominator);
            if scaled.denominator_ref() != &BigInt::from(1) {
                return Err("coroot relation did not clear denominators".to_string());
            }
            i64::try_from(scaled.numerator_ref())
                .map_err(|_| "coroot relation entry overflow".to_string())
        })
        .collect::<Result<_, _>>()?;
    let mut divisor = 0i64;
    for &entry in &integral {
        divisor = gcd(divisor, entry.abs());
    }
    if divisor > 1 {
        for entry in &mut integral {
            *entry /= divisor;
        }
    }
    if integral.first().is_some_and(|&first| first < 0) {
        for entry in &mut integral {
            *entry = -*entry;
        }
    }
    Ok(integral)
}

fn gcd(mut a: i64, mut b: i64) -> i64 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a.abs()
}

fn lcm(a: i64, b: i64) -> i64 {
    if a == 0 || b == 0 {
        return 0;
    }
    a / gcd(a, b) * b
}

/// weyl::sorted_by_label (alcoves.cpp:157-184): all walls sorted by
/// decreasing component label, ties in RootNbr order.
fn sorted_by_label(
    root_system: &RootSystem,
    numbering: &RootNumbering,
    walls: &BTreeSet<usize>,
) -> Result<Vec<usize>, String> {
    let mut labelled: Vec<(i64, usize)> = Vec::with_capacity(walls.len());
    for component in root_components(root_system, numbering, walls) {
        let labels = labels_for_component(root_system, numbering, &component)?;
        for (nbr, label) in component.iter().zip(labels) {
            labelled.push((label, *nbr));
        }
    }
    labelled.sort_by(|left, right| right.0.cmp(&left.0).then(left.1.cmp(&right.1)));
    Ok(labelled.into_iter().map(|(_, nbr)| nbr).collect())
}

/// weyl::from_fundamental_alcove (alcoves.cpp:186-236): a Weyl word whose
/// alcove has the given wall set. One unit-labelled wall per component is
/// set aside; the rest is moved to the simple system by
/// to_positive_system (rootdata.cpp:1329-1347), and the reversed steps
/// indexed through the final simple values give the word.
fn from_fundamental_alcove(
    root_system: &RootSystem,
    numbering: &RootNumbering,
    walls: &BTreeSet<usize>,
) -> Result<Vec<usize>, String> {
    let mut remaining: BTreeSet<usize> = walls.clone();
    for component in root_components(root_system, numbering, walls) {
        let labels = labels_for_component(root_system, numbering, &component)?;
        let special = component
            .iter()
            .zip(&labels)
            .find(|(_, &label)| label == 1)
            .map(|(&nbr, _)| nbr)
            .ok_or_else(|| "alcove wall component has no unit label".to_string())?;
        remaining.remove(&special);
    }
    let mut wall_vec: Vec<usize> = remaining.iter().copied().collect();
    let mut steps: Vec<usize> = Vec::new(); // positions holding a negative root
    loop {
        let found = wall_vec
            .iter()
            .enumerate()
            .find(|(_, &nbr)| numbering.is_negative(nbr))
            .map(|(position, &nbr)| (position, nbr));
        let Some((position, alpha)) = found else {
            break;
        };
        steps.push(position);
        // Apply the reflection in root alpha to every entry.
        let alpha_id = numbering.id(alpha);
        let alpha_root = root_system
            .root(alpha_id)
            .expect("root id in range")
            .as_slice()
            .to_vec();
        let alpha_coroot = root_system
            .coroot(alpha_id)
            .expect("root id in range")
            .as_slice()
            .to_vec();
        for entry in &mut wall_vec {
            let beta = root_system
                .root(numbering.id(*entry))
                .expect("root id in range")
                .as_slice();
            let coefficient: i64 = beta
                .iter()
                .zip(&alpha_coroot)
                .map(|(&b, &c)| i64::from(b) * i64::from(c))
                .sum();
            let reflected: Vec<i32> = beta
                .iter()
                .zip(&alpha_root)
                .map(|(&b, &a)| b - (coefficient as i32) * a)
                .collect();
            let reflected_id = root_system
                .id_of(&Weight::new(reflected))
                .ok_or_else(|| "reflected root missing from system".to_string())?;
            *entry = numbering.nbr(reflected_id);
        }
    }
    steps.reverse();
    let mut result = Vec::with_capacity(steps.len());
    for position in steps {
        let nbr = wall_vec[position];
        if numbering.is_negative(nbr)
            || nbr >= numbering.npos + root_system.datum().semisimple_rank()
        {
            return Err("alcove walls did not reduce to simple roots".to_string());
        }
        result.push(nbr - numbering.npos);
    }
    Ok(result)
}

/// The fundamental alcove has the simple coroots plus one lowest coroot
/// per irreducible component (RootSystem::fundamental_alcove_walls,
/// rootdata.cpp:474-481), so its size is rank + components. Only the size
/// is observable (the "Too few walls" check).
fn fundamental_alcove_wall_count(datum: &BasedRootDatum) -> usize {
    let rank = datum.semisimple_rank();
    let cartan = datum.cartan_matrix();
    let mut parent: Vec<usize> = (0..rank).collect();
    fn find(parent: &mut [usize], index: usize) -> usize {
        let mut root = index;
        while parent[root] != root {
            root = parent[root];
        }
        let mut current = index;
        while parent[current] != current {
            let next = parent[current];
            parent[current] = root;
            current = next;
        }
        root
    }
    for (left, row) in cartan.iter().enumerate() {
        for (offset, (&entry, &mirror)) in row[left + 1..]
            .iter()
            .zip(cartan[left + 1..].iter().map(|other| &other[left]))
            .enumerate()
        {
            if entry != 0 || mirror != 0 {
                let a = find(&mut parent, left);
                let b = find(&mut parent, left + 1 + offset);
                if a != b {
                    parent[b] = a;
                }
            }
        }
    }
    let components = (0..rank)
        .filter(|&index| find(&mut parent, index) == index)
        .count();
    rank + components
}

/// Exact inverse of a Cartan matrix as (numerator, denominator), by
/// rational Gauss-Jordan elimination. Only ratios matter downstream (the
/// coset enumeration in `basic_orbit` is invariant under positive scaling
/// of the fundamental (co)weight), so the lcm denominator is as good as
/// upstream's `C_denom`.
fn inverse_cartan(cartan: &[Vec<i32>]) -> (Vec<Vec<i64>>, i64) {
    let rank = cartan.len();
    let mut matrix: Vec<Vec<BigRational>> = (0..rank)
        .map(|row| {
            (0..2 * rank)
                .map(|column| {
                    if column < rank {
                        BigRational::from(cartan[row][column])
                    } else {
                        BigRational::from((column - rank == row) as i32)
                    }
                })
                .collect()
        })
        .collect();
    for column in 0..rank {
        let Some(found) = (column..rank).find(|&row| matrix[row][column] != 0) else {
            continue;
        };
        matrix.swap(column, found);
        let pivot = matrix[column][column].clone();
        for entry in &mut matrix[column] {
            *entry /= &pivot;
        }
        for row in 0..rank {
            if row == column || matrix[row][column] == 0 {
                continue;
            }
            let factor = matrix[row][column].clone();
            let (pivot_line, target) = if row < column {
                let (head, tail) = matrix.split_at_mut(column);
                (&tail[0], &mut head[row])
            } else {
                let (head, tail) = matrix.split_at_mut(row);
                (&head[column], &mut tail[0])
            };
            for (target_entry, pivot_entry) in target.iter_mut().zip(pivot_line.iter()) {
                let subtracted = pivot_entry.clone() * &factor;
                *target_entry -= subtracted;
            }
        }
    }
    let mut denominator = 1i64;
    for row in &matrix {
        for entry in &row[rank..] {
            if let Ok(den) = i64::try_from(entry.denominator_ref()) {
                denominator = lcm(denominator, den);
            }
        }
    }
    let numerator: Vec<Vec<i64>> = matrix
        .iter()
        .map(|row| {
            row[rank..]
                .iter()
                .map(|entry| {
                    let scaled = entry * BigRational::from(denominator);
                    i64::try_from(scaled.numerator_ref()).unwrap_or(i64::MAX)
                })
                .collect()
        })
        .collect();
    (numerator, denominator)
}

/// factor_dominant (rootdata.cpp:1117-1135): reflect `v` across simple
/// roots until dominant, accumulating the conversion word (push_back).
fn factor_dominant_word(datum: &BasedRootDatum, v: &mut [i32]) -> Vec<usize> {
    let rank = datum.semisimple_rank();
    let mut word = Vec::new();
    loop {
        let mut acted = false;
        for s in 0..rank {
            let pairing: i64 = v
                .iter()
                .zip(datum.simple_coroots()[s].as_slice())
                .map(|(&a, &b)| i64::from(a) * i64::from(b))
                .sum();
            if pairing < 0 {
                let root = datum.simple_roots()[s].as_slice();
                for (slot, &coordinate) in v.iter_mut().zip(root) {
                    *slot -= (pairing as i32) * coordinate;
                }
                word.push(s);
                acted = true;
                break;
            }
        }
        if !acted {
            return word;
        }
    }
}

/// factor_codominant (rootdata.cpp:1138-1155): reflect `v` across simple
/// coroots until codominant, accumulating the conversion word (push_front).
fn factor_codominant_word(datum: &BasedRootDatum, v: &mut [i32]) -> Vec<usize> {
    let rank = datum.semisimple_rank();
    let mut word = Vec::new();
    loop {
        let mut acted = false;
        for s in 0..rank {
            let pairing: i64 = v
                .iter()
                .zip(datum.simple_roots()[s].as_slice())
                .map(|(&a, &b)| i64::from(a) * i64::from(b))
                .sum();
            if pairing < 0 {
                let coroot = datum.simple_coroots()[s].as_slice();
                for (slot, &coordinate) in v.iter_mut().zip(coroot) {
                    *slot -= (pairing as i32) * coordinate;
                }
                word.insert(0, s);
                acted = true;
                break;
            }
        }
        if !acted {
            return word;
        }
    }
}

/// One level of the parabolic-quotient coset tree: the weight that
/// enumerates the coset, the generator that produced it, and the parent
/// index (basic_orbit, rootdata.cpp:1710-1754).
fn basic_orbit(
    datum: &BasedRootDatum,
    inverse: &(Vec<Vec<i64>>, i64),
    dual: bool,
    stab: &mut [bool],
    generator: usize,
) -> Vec<(Vec<i32>, usize, usize)> {
    let rank = datum.semisimple_rank();
    // Fundamental (co)weight numerator in ambient coordinates
    // (rootdata.cpp:847-853): weights use ROW i of the inverse Cartan
    // numerator over the simple roots, coweights COLUMN i over coroots.
    let mut e: Vec<i64> = vec![0; datum.lattice_rank()];
    for j in 0..rank {
        let coefficient = if dual {
            inverse.0[j][generator]
        } else {
            inverse.0[generator][j]
        };
        let basis = if dual {
            datum.simple_coroots()[j].as_slice()
        } else {
            datum.simple_roots()[j].as_slice()
        };
        for (slot, &coordinate) in e.iter_mut().zip(basis) {
            *slot += coefficient * i64::from(coordinate);
        }
    }
    let mut e: Vec<i32> = e.into_iter().map(|value| value as i32).collect();
    let mut result: Vec<(Vec<i32>, usize, usize)> = vec![(e.clone(), usize::MAX, usize::MAX)];
    stab[generator] = true;
    if dual {
        simple_coreflect(datum, generator, &mut e);
    } else {
        simple_reflect(datum, generator, &mut e);
    }
    result.push((e, generator, 0));
    let mut start = 1;
    let mut finish = 2;
    // Upstream iterates the live RankFlags (`for (auto s : stab)`), which
    // `basic_orbit` never mutates after setting `generator`, so a snapshot
    // of the stabbing generators is equivalent.
    let stabbing: Vec<usize> = stab
        .iter()
        .enumerate()
        .filter_map(|(s, &fixed)| fixed.then_some(s))
        .collect();
    loop {
        for index in start..finish {
            for &s in &stabbing {
                let mut weight = result[index].0.clone();
                let level = if dual {
                    dot_i32(&weight, datum.simple_roots()[s].as_slice())
                } else {
                    dot_i32(&weight, datum.simple_coroots()[s].as_slice())
                };
                if level <= 0 {
                    continue;
                }
                let step = if dual {
                    datum.simple_coroots()[s].as_slice()
                } else {
                    datum.simple_roots()[s].as_slice()
                };
                for (slot, &coordinate) in weight.iter_mut().zip(step) {
                    *slot -= level * coordinate;
                }
                if !result[finish..].iter().any(|(seen, _, _)| *seen == weight) {
                    result.push((weight, s, index));
                }
            }
        }
        if finish == result.len() {
            return result;
        }
        start = finish;
        finish = result.len();
    }
}

fn dot_i32(left: &[i32], right: &[i32]) -> i32 {
    left.iter().zip(right).map(|(&a, &b)| a * b).sum()
}

fn simple_reflect(datum: &BasedRootDatum, s: usize, weight: &mut [i32]) {
    let level = dot_i32(weight, datum.simple_coroots()[s].as_slice());
    for (slot, &coordinate) in weight.iter_mut().zip(datum.simple_roots()[s].as_slice()) {
        *slot -= level * coordinate;
    }
}

fn simple_coreflect(datum: &BasedRootDatum, s: usize, weight: &mut [i32]) {
    let level = dot_i32(weight, datum.simple_roots()[s].as_slice());
    for (slot, &coordinate) in weight.iter_mut().zip(datum.simple_coroots()[s].as_slice()) {
        *slot -= level * coordinate;
    }
}

/// extend_orbit (rootdata.cpp:1782-1808): each current orbit element is
/// replaced by its segment [x, t(x,c1), t(x,c2), ...] in coset order,
/// with t applying the coset generator to the parent segment entry.
fn extend_orbit_weights(
    datum: &BasedRootDatum,
    inverse: &(Vec<Vec<i64>>, i64),
    dual: bool,
    orbit: &mut Vec<Vec<i32>>,
    stab: &mut [bool],
    generator: usize,
) {
    let cosets = basic_orbit(datum, inverse, dual, stab, generator);
    let mut next = Vec::new();
    for element in orbit.iter() {
        let mut segment: Vec<Vec<i32>> = vec![element.clone()];
        for &(_, s, prev) in &cosets[1..] {
            let mut weight = segment[prev].clone();
            if dual {
                simple_coreflect(datum, s, &mut weight);
            } else {
                simple_reflect(datum, s, &mut weight);
            }
            segment.push(weight);
        }
        next.extend(segment);
    }
    *orbit = next;
}

/// extend_orbit_words (rootdata.cpp:1756-1780): the same segment rebuild
/// on Weyl words; non-dual left-multiplies (prepends the generator), dual
/// right-multiplies (appends it).
fn extend_orbit_words(
    datum: &BasedRootDatum,
    inverse: &(Vec<Vec<i64>>, i64),
    dual: bool,
    orbit: &mut Vec<Vec<usize>>,
    stab: &mut [bool],
    generator: usize,
) {
    let cosets = basic_orbit(datum, inverse, dual, stab, generator);
    let mut next = Vec::new();
    for word in orbit.iter() {
        let mut segment: Vec<Vec<usize>> = vec![word.clone()];
        for &(_, s, prev) in &cosets[1..] {
            let mut extended = segment[prev].clone();
            if dual {
                extended.push(s);
            } else {
                extended.insert(0, s);
            }
            segment.push(extended);
        }
        next.extend(segment);
    }
    *orbit = next;
}

/// The vec argument of the Weyl orbit builtins: a `vec` value or an int
/// row (the typed layer accepts both for a `vec` parameter).
fn as_weight_coordinates(value: &Value, span: SourceSpan) -> Result<Vec<i32>, Diagnostic> {
    match value {
        Value::Vector(Vec32(entries)) => Ok(entries.clone()),
        Value::List(entries) => entries
            .iter()
            .map(|entry| {
                Ok(as_integer(entry, span)?
                    .to_string()
                    .parse::<i32>()
                    .unwrap_or(i32::MAX))
            })
            .collect(),
        other => Err(type_error(span, format!("expected a vec, found {other}"))),
    }
}

fn positive_coroot_pairing(root_system: &RootSystem, id: RootId, gamma: &RatVec) -> Option<i64> {
    let coroot = root_system.coroot(id)?;
    Some(
        gamma
            .numerators()
            .iter()
            .zip(coroot.as_slice())
            .map(|(g, &c)| g * i64::from(c))
            .sum(),
    )
}

/// Convert a pub-fielded IntegerMatrix to row-major i32.
fn integer_matrix_i32(matrix: &atlas_real_group::IntegerMatrix) -> Vec<Vec<i32>> {
    let rows = matrix.rows;
    let cols = matrix.columns;
    (0..rows)
        .map(|row| {
            (0..cols)
                .map(|column| {
                    matrix.entries[row * cols + column]
                        .to_string()
                        .parse::<i32>()
                        .unwrap_or(i32::MAX)
                })
                .collect()
        })
        .collect()
}

/// Transpose a row-major matrix.
fn transpose_matrix_i32(matrix: &[Vec<i32>]) -> Vec<Vec<i32>> {
    let rows = matrix.len();
    let cols = matrix.first().map_or(0, Vec::len);
    (0..cols)
        .map(|column| (0..rows).map(|row| matrix[row][column]).collect())
        .collect()
}

/// The first `rows` rows of a row-major matrix (block(0,0,rows,cols)).
fn block_rows(matrix: &[Vec<i32>], rows: usize) -> Vec<Vec<i32>> {
    matrix.iter().take(rows).cloned().collect()
}

/// The first `rows` rows and `cols` columns of a row-major matrix.
fn block_rows_cols(matrix: &[Vec<i32>], rows: usize, cols: usize) -> Vec<Vec<i32>> {
    matrix
        .iter()
        .take(rows)
        .map(|row| row.iter().take(cols).copied().collect())
        .collect()
}

/// Row-major matrix product (left rows x left cols, right rows x right cols).
fn mat_mul_i32(left: &[Vec<i32>], right: &[Vec<i32>]) -> Result<Vec<Vec<i32>>, String> {
    let left_rows = left.len();
    let left_cols = left.first().map_or(0, Vec::len);
    let right_rows = right.len();
    let right_cols = right.first().map_or(0, Vec::len);
    if left_cols != right_rows {
        return Err(format!(
            "matrix shape mismatch: {left_rows}x{left_cols} * {right_rows}x{right_cols}"
        ));
    }
    let mut out = vec![vec![0; right_cols]; left_rows];
    for row in 0..left_rows {
        for column in 0..right_cols {
            let mut total = 0_i64;
            for k in 0..left_cols {
                total += i64::from(left[row][k]) * i64::from(right[k][column]);
            }
            out[row][column] = total as i32;
        }
    }
    Ok(out)
}

fn simple_basis(root_system: &RootSystem, rs: &[RootId]) -> Result<Vec<RootId>, StructureError> {
    let mut result = Vec::new();
    for &alpha in rs {
        if !root_system.is_positive(alpha).unwrap_or(false) {
            continue;
        }
        let coords = root_system.simple_coordinates(alpha);
        let Some(coords) = coords else {
            continue;
        };
        let mut is_simple = true;
        for &beta in rs {
            if beta == alpha || !root_system.is_positive(beta).unwrap_or(false) {
                continue;
            }
            let beta_coords = root_system.simple_coordinates(beta);
            if let Some(beta_coords) = beta_coords {
                if beta_coords.len() == coords.len()
                    && beta_coords != coords
                    && beta_coords.iter().zip(coords).all(|(b, a)| b <= a)
                {
                    is_simple = false;
                    break;
                }
            }
        }
        if is_simple {
            result.push(alpha);
        }
    }
    Ok(result)
}

// ============================================================
// alcove/FPP layer (alcoves.cpp): root vertices, the center
// classifier, facet orbits, and the FPP generation loops.
// ============================================================

/// Determinant and adjugate of an integer matrix, by fraction-free
/// Bareiss elimination for the determinant and cofactor expansion for
/// the adjugate (ranks here stay <= 9). `adj / det` is the inverse.
fn adjugate_det(matrix: &[Vec<i32>]) -> (Vec<Vec<i64>>, i64) {
    let n = matrix.len();
    if n == 0 {
        return (Vec::new(), 1);
    }
    fn minor_det(matrix: &[Vec<i64>], skip_row: usize, skip_col: usize) -> i64 {
        let n = matrix.len();
        let mut sub: Vec<Vec<i64>> = Vec::with_capacity(n - 1);
        for (row, line) in matrix.iter().enumerate() {
            if row == skip_row {
                continue;
            }
            sub.push(
                line.iter()
                    .enumerate()
                    .filter(|(col, _)| *col != skip_col)
                    .map(|(_, &entry)| entry)
                    .collect(),
            );
        }
        bareiss_det(&sub)
    }
    let wide: Vec<Vec<i64>> = matrix
        .iter()
        .map(|row| row.iter().map(|&entry| i64::from(entry)).collect())
        .collect();
    let det = bareiss_det(&wide);
    let mut adj = vec![vec![0i64; n]; n];
    for (column, adj_row) in adj.iter_mut().enumerate() {
        for (row, slot) in adj_row.iter_mut().enumerate() {
            let sign = if (row + column) % 2 == 0 { 1 } else { -1 };
            // adjugate is the transpose of the cofactor matrix.
            *slot = sign * minor_det(&wide, row, column);
        }
    }
    (adj, det)
}

/// Fraction-free Bareiss determinant.
fn bareiss_det(matrix: &[Vec<i64>]) -> i64 {
    let n = matrix.len();
    if n == 0 {
        return 1;
    }
    let mut a: Vec<Vec<i64>> = matrix.to_vec();
    let mut sign = 1i64;
    let mut previous = 1i64;
    for pivot in 0..n - 1 {
        if a[pivot][pivot] == 0 {
            let Some(swap) = (pivot + 1..n).find(|&row| a[row][pivot] != 0) else {
                return 0;
            };
            a.swap(pivot, swap);
            sign = -sign;
        }
        let pivot_value = a[pivot][pivot];
        for row in pivot + 1..n {
            for column in pivot + 1..n {
                a[row][column] =
                    (a[row][column] * pivot_value - a[row][pivot] * a[pivot][column]) / previous;
            }
        }
        previous = pivot_value;
    }
    sign * a[n - 1][n - 1]
}

/// weyl::root_vertex_simple (alcoves.cpp:345-408): the unique vertex of
/// the alcove's projection on one wall component that lies in the root
/// lattice. `ev_floors` holds the integer parts of the wall evaluations.
fn root_vertex_simple(
    root_system: &RootSystem,
    numbering: &RootNumbering,
    component: &[usize],
    ev_floors: &[i64],
) -> Result<Vec<i32>, String> {
    let datum = root_system.datum();
    let rank = datum.lattice_rank();
    let labels = labels_for_component(root_system, numbering, component)?;
    // The first label-1 wall serves as the "lowest coroot" wall; the
    // others generate the finite part of the component.
    let first_one = labels
        .iter()
        .position(|&label| label == 1)
        .ok_or_else(|| "alcove wall component has no label-1 wall".to_string())?;
    let mut generators: Vec<usize> = Vec::new();
    let mut labels_one: Vec<usize> = Vec::new();
    let mut floors: Vec<i64> = Vec::new();
    for (index, &nbr) in component.iter().enumerate() {
        if index == first_one {
            continue;
        }
        generators.push(nbr);
        if labels[index] == 1 {
            labels_one.push(generators.len() - 1);
        }
        floors.push(ev_floors[index]);
    }
    let cartan: Vec<Vec<i32>> = generators
        .iter()
        .map(|&alpha| {
            generators
                .iter()
                .map(|&beta| {
                    root_system
                        .bracket(numbering.id(alpha), numbering.id(beta))
                        .unwrap_or(0)
                })
                .collect()
        })
        .collect();
    let (numer, denom) = inverse_cartan(&transpose_matrix_i32(&cartan));
    let scale = |extra_column: Option<usize>| -> Vec<i64> {
        (0..generators.len())
            .map(|row| {
                let mut total = numer[row]
                    .iter()
                    .zip(&floors)
                    .map(|(&entry, &floor)| entry * floor)
                    .sum::<i64>();
                if let Some(column) = extra_column {
                    total += numer[row][column];
                }
                total
            })
            .collect()
    };
    let mut result = vec![0i32; rank];
    let mut try_vertex = |extra_column: Option<usize>| -> bool {
        let scaled = scale(extra_column);
        if !scaled.iter().all(|&entry| entry % denom == 0) {
            return false;
        }
        for (index, &nbr) in generators.iter().enumerate() {
            let coefficient = scaled[index] / denom;
            let root = root_system
                .root(numbering.id(nbr))
                .expect("every root has an ambient vector");
            for (slot, &coordinate) in result.iter_mut().zip(root.as_slice()) {
                *slot += (coefficient as i32) * coordinate;
            }
        }
        true
    };
    if try_vertex(None) {
        return Ok(result);
    }
    for &column in &labels_one {
        if try_vertex(Some(column)) {
            return Ok(result);
        }
    }
    Err("no root lattice vertex found for alcove component".to_string())
}

/// center_classifier (alcoves.cpp:793-870): tabulates the subsets of
/// fundamental weights whose sum lies in the root lattice, grouped by
/// root-lattice coset (adjoint-coordinate fractional part). All
/// arithmetic is on the adjugate/determinant representation of the
/// inverse Cartan, matching upstream's `C_denom` exactly.
struct CenterClassifier {
    det: i64,
    /// Root-lattice coset fractional parts, sorted (the `table`).
    table: Vec<(Vec<u8>, Vec<u32>)>,
    /// Coset index per subset of generators (`rts_tab[..].cls`).
    class_of: Vec<usize>,
    /// Adjoint-integral part of each subset sum (`rts_tab[..].shift`).
    shift_of: Vec<Vec<i64>>,
}

impl CenterClassifier {
    fn new(cartan: &[Vec<i32>]) -> Self {
        let rank = cartan.len();
        let (adj, det) = adjugate_det(cartan);
        let mut buckets: std::collections::BTreeMap<Vec<u8>, Vec<u32>> =
            std::collections::BTreeMap::new();
        let mut shift_of = vec![vec![0i64; rank]; 1 << rank];
        for subset in 0u32..(1 << rank) {
            let mut sum = vec![0i64; rank];
            for (s, adj_row) in adj.iter().enumerate() {
                if subset & (1 << s) != 0 {
                    for (entry, &adj_entry) in sum.iter_mut().zip(adj_row) {
                        *entry += adj_entry;
                    }
                }
            }
            let fractional: Vec<u8> = sum
                .iter()
                .map(|&entry| entry.rem_euclid(det) as u8)
                .collect();
            shift_of[subset as usize] = sum.iter().map(|&entry| entry.div_euclid(det)).collect();
            buckets.entry(fractional).or_default().push(subset);
        }
        let table: Vec<(Vec<u8>, Vec<u32>)> = buckets.into_iter().collect();
        let mut class_of = vec![0usize; 1 << rank];
        for (class, (_, subsets)) in table.iter().enumerate() {
            for &subset in subsets {
                class_of[subset as usize] = class;
            }
        }
        CenterClassifier {
            det,
            table,
            class_of,
            shift_of,
        }
    }

    /// center_classifier::shifts (alcoves.cpp:846-869): the root-lattice
    /// shifts `fw(fix+A)-fw(B)` for subsets A of `pos` and B of `neg`,
    /// in adjoint (simple root) coordinates.
    fn shifts(&self, fix: u32, pos: u32, neg: u32) -> Vec<Vec<i64>> {
        let fix_ev = &self.table[self.class_of[fix as usize]].0;
        let base = self.shift_of[fix as usize].clone();
        let neg_bits: Vec<u32> = (0..32).filter(|&s| neg & (1 << s) != 0).collect();
        let mut result = Vec::new();
        for bits in 0u32..(1 << neg_bits.len()) {
            let mut negset = 0u32;
            for (index, &bit) in neg_bits.iter().enumerate() {
                if bits & (1 << index) != 0 {
                    negset |= 1 << bit;
                }
            }
            let mut rts: Vec<i64> = base
                .iter()
                .zip(&self.shift_of[negset as usize])
                .map(|(a, b)| a - b)
                .collect();
            let mut diff: Vec<i64> = self.table[self.class_of[negset as usize]]
                .0
                .iter()
                .map(|&entry| i64::from(entry))
                .collect();
            for (index, entry) in diff.iter_mut().enumerate() {
                let fix_entry = i64::from(fix_ev[index]);
                if fix_entry <= *entry {
                    *entry -= fix_entry;
                } else {
                    rts[index] += 1;
                    *entry -= fix_entry - self.det;
                }
            }
            let diff_bytes: Vec<u8> = diff.iter().map(|&entry| entry as u8).collect();
            if let Ok(class) = self
                .table
                .binary_search_by(|probe| probe.0.cmp(&diff_bytes))
            {
                for &subset in &self.table[class].1 {
                    if pos & subset == subset {
                        result.push(
                            rts.iter()
                                .zip(&self.shift_of[subset as usize])
                                .map(|(a, b)| a + b)
                                .collect(),
                        );
                    }
                }
            }
        }
        result
    }
}

/// orbit_elem (alcoves.cpp:437-445) for the adjoint-coordinate orbits.
#[derive(Clone, Debug)]
struct AdjOrbitElem {
    v: Vec<i32>,
    s: usize,
    seen: u64,
    prev: usize,
}

/// Shared BFS core of basic_orbit/vertex_orbit (alcoves.cpp:480-565):
/// new elements are inserted after `finish` keeping the tail DECREASING,
/// and each finished level is reversed to increasing order.
#[allow(clippy::too_many_arguments)]
fn adjoint_orbit_bfs(
    width: usize,
    generators: usize,
    vertex: Vec<i32>,
    reflect: impl Fn(&[i32], usize) -> Option<(i32, Vec<i32>)>,
) -> Vec<AdjOrbitElem> {
    let mut result: Vec<AdjOrbitElem> = vec![AdjOrbitElem {
        v: vertex,
        s: usize::MAX,
        seen: 0,
        prev: usize::MAX,
    }];
    let mut start = 0;
    let mut finish = 1;
    let mut count = 0;
    loop {
        let mut index = start;
        while index < finish {
            for s in 0..generators {
                if result[index].seen & (1 << s) != 0 {
                    continue;
                }
                let Some((_, wt)) = reflect(&result[index].v, s) else {
                    continue;
                };
                let mut jt = finish;
                while jt < result.len() && wt < result[jt].v {
                    jt += 1;
                }
                if jt < result.len() && wt == result[jt].v {
                    result[jt].seen |= 1 << s;
                } else {
                    result.insert(
                        jt,
                        AdjOrbitElem {
                            v: wt,
                            s,
                            seen: 1 << s,
                            prev: count,
                        },
                    );
                }
            }
            count += 1;
            index += 1;
        }
        if finish == result.len() {
            return result;
        }
        result[finish..].reverse();
        start = finish;
        finish = result.len();
        let _ = width;
    }
}

/// basic_orbit (alcoves.cpp:526-565): Levi subquotient orbit in adjoint
/// coordinates for the first `i+1` generators of `cartan`.
fn basic_orbit_adjoint(cartan: &[Vec<i32>], i: usize) -> Vec<AdjOrbitElem> {
    let n = i + 1;
    let adj_coroot: Vec<Vec<i32>> = (0..cartan.len())
        .map(|column| (0..n).map(|row| cartan[row][column]).collect())
        .collect();
    let sub: Vec<Vec<i32>> = cartan[..n].iter().map(|row| row[..n].to_vec()).collect();
    let (adj, _det) = adjugate_det(&sub);
    let vertex: Vec<i32> = adj[i].iter().map(|&entry| entry as i32).collect();
    adjoint_orbit_bfs(n, n, vertex, |v, s| {
        let level: i64 = adj_coroot[s]
            .iter()
            .zip(v)
            .map(|(&a, &b)| i64::from(a) * i64::from(b))
            .sum();
        if level == 0 {
            return None;
        }
        let mut wt = v.to_vec();
        wt[s] -= level as i32;
        Some((level as i32, wt))
    })
}

/// vertex_orbit (alcoves.cpp:458-518): the modular variant used for the
/// final extension along a label > 1.
fn vertex_orbit(cartan: &[Vec<i32>], i: usize, label: i64) -> Vec<AdjOrbitElem> {
    let rank = cartan.len();
    let adj_coroot: Vec<Vec<i32>> = (0..rank)
        .map(|column| (0..rank).map(|row| cartan[row][column]).collect())
        .collect();
    let (adj, det) = adjugate_det(cartan);
    let modulus = det * label;
    let vertex: Vec<i32> = adj[i]
        .iter()
        .map(|&entry| entry.rem_euclid(modulus) as i32)
        .collect();
    adjoint_orbit_bfs(rank, rank, vertex, |v, s| {
        let level: i64 = adj_coroot[s]
            .iter()
            .zip(v)
            .map(|(&a, &b)| i64::from(a) * i64::from(b))
            .sum();
        if level % modulus == 0 {
            return None;
        }
        let mut wt = v.to_vec();
        wt[s] = (i64::from(wt[s]) - level).rem_euclid(modulus) as i32;
        Some((level as i32, wt))
    })
}

/// convert_to_words (alcoves.cpp:746-774): expand the identity through
/// the coset tree, LEFT-multiplying each step's reflection word onto the
/// parent segment entry.
fn convert_to_words(cosets: &[AdjOrbitElem], reflections: &[Vec<usize>]) -> Vec<Vec<usize>> {
    let orbit: Vec<Vec<usize>> = vec![Vec::new()];
    let mut result: Vec<Vec<usize>> = Vec::new();
    for word in &orbit {
        let mut segment: Vec<Vec<usize>> = vec![word.clone()];
        for elem in &cosets[1..] {
            let mut extended = reflections[elem.s].clone();
            extended.extend_from_slice(&segment[elem.prev]);
            segment.push(extended);
        }
        result.extend(segment);
    }
    result
}

/// The reflection across simple root `s` applied to a RootNbr.
fn simple_reflect_root_nbr(
    root_system: &RootSystem,
    numbering: &RootNumbering,
    s: usize,
    nbr: usize,
) -> usize {
    let datum = root_system.datum();
    let id = numbering.id(nbr);
    let pairing = root_system
        .bracket(id, numbering.id(numbering.npos + s))
        .unwrap_or(0);
    let reflected: Vec<i32> = root_system
        .root(id)
        .expect("every root has an ambient vector")
        .as_slice()
        .iter()
        .zip(datum.simple_roots()[s].as_slice())
        .map(|(&coordinate, &simple)| coordinate - pairing * simple)
        .collect();
    let reflected_id = root_system
        .id_of(&Weight::new(reflected))
        .expect("a reflected root is a root");
    numbering.nbr(reflected_id)
}

/// RootSystem::reflection_word (rootdata.cpp:601-618): descend along the
/// first descent until simple, then mirror the path.
fn reflection_word(root_system: &RootSystem, numbering: &RootNumbering, nbr: usize) -> Vec<usize> {
    let datum = root_system.datum();
    let rank = datum.semisimple_rank();
    let npos = numbering.npos;
    // make_positive: work with the positive partner.
    let mut alpha = if numbering.is_negative(nbr) {
        2 * npos - 1 - nbr
    } else {
        nbr
    };
    let mut word = Vec::new();
    while alpha >= npos + rank {
        let id = numbering.id(alpha);
        let s = (0..rank)
            .find(|&s| root_system.bracket(id, numbering.id(npos + s)).unwrap_or(0) > 0)
            .expect("a non-simple positive root has a descent");
        word.push(s);
        alpha = simple_reflect_root_nbr(root_system, numbering, s, alpha);
    }
    word.push(alpha - npos);
    let mirror: Vec<usize> = word[..word.len() - 1].to_vec();
    word.extend(mirror);
    word
}

/// RootSystem::permuted_root (rootdata.h:313-318): a Weyl word acts on a
/// root with the LAST letter applied first.
fn word_act_root(
    root_system: &RootSystem,
    numbering: &RootNumbering,
    word: &[usize],
    nbr: usize,
) -> usize {
    let mut current = nbr;
    for &s in word.iter().rev() {
        current = simple_reflect_root_nbr(root_system, numbering, s, current);
    }
    current
}

/// WeylGroup::act on a weight (weyl.cpp:1071-1082): the same convention.
fn word_act_weight(datum: &BasedRootDatum, word: &[usize], weight: &mut [i32]) {
    for &s in word.iter().rev() {
        simple_reflect(datum, s, weight);
    }
}

/// RootSystem::fundamental_alcove_walls (rootdata.cpp:474-481): the
/// simple roots plus, per Dynkin component, the negative root whose
/// coroot no simple coroot can be subtracted from.
fn fundamental_alcove_walls(
    root_system: &RootSystem,
    numbering: &RootNumbering,
) -> BTreeSet<usize> {
    let datum = root_system.datum();
    let rank = datum.semisimple_rank();
    let npos = numbering.npos;
    let num_roots = root_system.roots().len();
    let coroot_table: BTreeSet<Vec<i32>> = (0..num_roots)
        .filter_map(|index| {
            root_system
                .coroot(RootId::from_usize(index))
                .map(|coroot| coroot.as_slice().to_vec())
        })
        .collect();
    let coroot_of = |nbr: usize| {
        root_system
            .coroot(numbering.id(nbr))
            .expect("every root has a coroot")
            .as_slice()
            .to_vec()
    };
    let mut walls: BTreeSet<usize> = (0..npos).collect();
    for s in 0..rank {
        let simple_coroot = coroot_of(npos + s);
        walls.retain(|&beta| {
            let difference: Vec<i32> = coroot_of(beta)
                .iter()
                .zip(&simple_coroot)
                .map(|(&b, &a)| b - a)
                .collect();
            !coroot_table.contains(&difference)
        });
    }
    walls.extend(npos..npos + rank);
    walls
}

/// list_roots_and_labels (alcoves.cpp:677-707): the component's roots
/// with `stab` first, the rest sorted by decreasing label (RootNbr
/// ascending on ties), labels aligned.
fn list_roots_and_labels(
    root_system: &RootSystem,
    numbering: &RootNumbering,
    component: &BTreeSet<usize>,
    stab: &BTreeSet<usize>,
) -> Result<(Vec<usize>, Vec<i64>), String> {
    let stab_size = stab.len();
    let mut roots: Vec<usize> = stab
        .iter()
        .copied()
        .chain(component.difference(stab).copied())
        .collect();
    let mut labels = labels_for_component(root_system, numbering, &roots)?;
    let comp_size = roots.len();
    if comp_size > stab_size + 1 {
        let mut pairs: Vec<(i64, usize)> = (stab_size..comp_size)
            .map(|index| (labels[index], roots[index]))
            .collect();
        pairs.sort_by(|left, right| right.0.cmp(&left.0).then(left.1.cmp(&right.1)));
        for (offset, (label, root)) in pairs.into_iter().enumerate() {
            labels[stab_size + offset] = label;
            roots[stab_size + offset] = root;
        }
    }
    Ok((roots, labels))
}

/// The Cartan bracket matrix of a RootNbr list.
fn cartan_of_roots(
    root_system: &RootSystem,
    numbering: &RootNumbering,
    roots: &[usize],
) -> Vec<Vec<i32>> {
    roots
        .iter()
        .map(|&alpha| {
            roots
                .iter()
                .map(|&beta| {
                    root_system
                        .bracket(numbering.id(alpha), numbering.id(beta))
                        .unwrap_or(0)
                })
                .collect()
        })
        .collect()
}

/// facet_orbit_ws (alcoves.cpp:894-943): the per-component coset word
/// lists for the quotient of W by a facet's stabiliser.
fn facet_orbit_ws(
    root_system: &RootSystem,
    numbering: &RootNumbering,
    stabilising_walls: &BTreeSet<usize>,
) -> Result<Vec<Vec<Vec<usize>>>, String> {
    let walls = fundamental_alcove_walls(root_system, numbering);
    let comps = root_components(root_system, numbering, &walls);
    let mut coset_lists: Vec<Vec<Vec<usize>>> = Vec::new();
    for comp in comps {
        let comp_set: BTreeSet<usize> = comp.into_iter().collect();
        let stab: BTreeSet<usize> = stabilising_walls.intersection(&comp_set).copied().collect();
        let stab_size = stab.len();
        let (mut roots, labels) = list_roots_and_labels(root_system, numbering, &comp_set, &stab)?;
        let last = roots
            .pop()
            .ok_or_else(|| "empty fundamental alcove component".to_string())?;
        let cartan = cartan_of_roots(root_system, numbering, &roots);
        let reflections: Vec<Vec<usize>> = roots
            .iter()
            .map(|&root| reflection_word(root_system, numbering, root))
            .collect();
        for s in stab_size..roots.len() {
            coset_lists.push(convert_to_words(
                &basic_orbit_adjoint(&cartan, s),
                &reflections,
            ));
        }
        if *labels.last().expect("nonempty labels") > 1 {
            // The final extension replaces the largest-index label-1
            // stabilised root by the "affine" root |last|.
            let mut index = stab_size;
            let chosen = loop {
                if index == 0 {
                    return Err("no label-1 stabilised wall for affine extension".to_string());
                }
                index -= 1;
                if labels[index] == 1 {
                    break index;
                }
            };
            let mut affine_roots = roots.clone();
            affine_roots[chosen] = last;
            let affine_cartan = cartan_of_roots(root_system, numbering, &affine_roots);
            let mut affine_reflections = reflections.clone();
            affine_reflections[chosen] = reflection_word(root_system, numbering, last);
            coset_lists.push(convert_to_words(
                &vertex_orbit(
                    &affine_cartan,
                    chosen,
                    *labels.last().expect("nonempty labels"),
                ),
                &affine_reflections,
            ));
        }
    }
    Ok(coset_lists)
}

/// extend_orbit_words single version (alcoves.cpp:573-597) on Weyl
/// words: each current orbit element grows the segment
/// [w, gens[c1]*w, gens[c2]*w, ...] in coset order — convert_to_words
/// generalised to a nonempty starting orbit.
fn extend_word_orbit(
    orbit: &mut Vec<Vec<usize>>,
    cosets: &[AdjOrbitElem],
    reflections: &[Vec<usize>],
) {
    let mut next = Vec::new();
    for word in orbit.iter() {
        let mut segment: Vec<Vec<usize>> = vec![word.clone()];
        for elem in &cosets[1..] {
            let mut extended = reflections[elem.s].clone();
            extended.extend_from_slice(&segment[elem.prev]);
            segment.push(extended);
        }
        next.extend(segment);
    }
    *orbit = next;
}

/// extend_affine_component (alcoves.cpp:665-697): extend `orbit` by the
/// cosets of the group generated by `comp` over the subgroup generated
/// by `comp ∩ stab`, including the final extension along a label > 1.
fn extend_affine_component(
    root_system: &RootSystem,
    numbering: &RootNumbering,
    orbit: &mut Vec<Vec<usize>>,
    comp: &BTreeSet<usize>,
    stab: &BTreeSet<usize>,
) -> Result<(), String> {
    let stab: BTreeSet<usize> = stab.intersection(comp).copied().collect();
    let stab_size = stab.len();
    let (mut roots, labels) = list_roots_and_labels(root_system, numbering, comp, &stab)?;
    let last = roots
        .pop()
        .ok_or_else(|| "empty affine component".to_string())?;
    if roots.len() > stab_size {
        let cartan = cartan_of_roots(root_system, numbering, &roots);
        let reflections: Vec<Vec<usize>> = roots
            .iter()
            .map(|&root| reflection_word(root_system, numbering, root))
            .collect();
        for s in stab_size..roots.len() {
            extend_word_orbit(orbit, &basic_orbit_adjoint(&cartan, s), &reflections);
        }
    }
    if *labels.last().expect("nonempty labels") > 1 {
        // The final extension replaces the largest-index label-1
        // stabilised root by the "affine" root |last|.
        let mut index = stab_size;
        let chosen = loop {
            if index == 0 {
                return Err("no label-1 stabilised wall for affine extension".to_string());
            }
            index -= 1;
            if labels[index] == 1 {
                break index;
            }
        };
        let mut affine_roots = roots;
        affine_roots[chosen] = last;
        let affine_cartan = cartan_of_roots(root_system, numbering, &affine_roots);
        let affine_reflections: Vec<Vec<usize>> = affine_roots
            .iter()
            .map(|&root| reflection_word(root_system, numbering, root))
            .collect();
        extend_word_orbit(
            orbit,
            &vertex_orbit(
                &affine_cartan,
                chosen,
                *labels.last().expect("nonempty labels"),
            ),
            &affine_reflections,
        );
    }
    Ok(())
}

/// finite_subquotient (alcoves.cpp:699-711): the word representatives
/// of W(stab ∪ {alpha}) modulo W(stab) when the extension stays finite.
fn finite_subquotient(
    root_system: &RootSystem,
    numbering: &RootNumbering,
    stab: &BTreeSet<usize>,
    alpha: usize,
) -> Vec<Vec<usize>> {
    let mut walls: Vec<usize> = stab.iter().copied().collect();
    walls.push(alpha);
    let cartan = cartan_of_roots(root_system, numbering, &walls);
    let reflections: Vec<Vec<usize>> = walls
        .iter()
        .map(|&root| reflection_word(root_system, numbering, root))
        .collect();
    convert_to_words(&basic_orbit_adjoint(&cartan, walls.len() - 1), &reflections)
}

/// complete_affine_component (alcoves.cpp:713-722): the affine variant,
/// reduced to a finite computation modulo the root lattice.
fn complete_affine_component(
    root_system: &RootSystem,
    numbering: &RootNumbering,
    stab: &BTreeSet<usize>,
    alpha: usize,
) -> Result<Vec<Vec<usize>>, String> {
    let mut orbit = vec![Vec::new()];
    let mut comp = stab.clone();
    comp.insert(alpha);
    extend_affine_component(root_system, numbering, &mut orbit, &comp, stab)?;
    Ok(orbit)
}

/// Fold Weyl words into language W_elt values (the shared tail of the
/// orbit-ws builtins).
fn weyl_word_values(
    handle: &RootDatumHandle,
    words: Vec<Vec<usize>>,
    span: SourceSpan,
) -> Result<Value, Diagnostic> {
    let context = build_weyl_context(handle, span)?;
    let mut result = Vec::with_capacity(words.len());
    for word in words {
        let mut element = WeylElement::identity(&context.system)
            .map_err(|error| runtime(span, error.to_string()))?;
        for generator in word {
            let (next, _) = element
                .right_multiply_simple(&context.system, generator)
                .map_err(|error| runtime(span, error.to_string()))?;
            element = next;
        }
        result.push(weyl_elt_value(Arc::clone(&context), element, span)?);
    }
    Ok(Value::List(result))
}

/// rootdata::additive_closure (rootdata.cpp:685-707, default
/// `for_coroots=true` as used by FPP, alcoves.cpp:980): close a root set
/// under negation and COROOT addition — the sum test and insertion are
/// done on coroot coordinates, not root coordinates.
fn additive_closure(
    root_system: &RootSystem,
    numbering: &RootNumbering,
    generators: &BTreeSet<usize>,
) -> BTreeSet<usize> {
    let npos = numbering.npos;
    let coroot_id: std::collections::BTreeMap<Vec<i32>, RootId> = (0..root_system.roots().len())
        .filter_map(|index| {
            let id = RootId::from_usize(index);
            root_system
                .coroot(id)
                .map(|coroot| (coroot.as_slice().to_vec(), id))
        })
        .collect();
    let coroot_of = |nbr: usize| {
        root_system
            .coroot(numbering.id(nbr))
            .expect("every root has a coroot")
            .as_slice()
            .to_vec()
    };
    let mut set: BTreeSet<usize> = generators
        .iter()
        .flat_map(|&nbr| [nbr, 2 * npos - 1 - nbr])
        .collect();
    loop {
        let current: Vec<usize> = set.iter().copied().collect();
        let mut grew = false;
        for (i, &alpha) in current.iter().enumerate() {
            for &beta in &current[i + 1..] {
                let sum: Vec<i32> = coroot_of(alpha)
                    .iter()
                    .zip(coroot_of(beta))
                    .map(|(&a, b)| a + b)
                    .collect();
                if let Some(&id) = coroot_id.get(&sum) {
                    if set.insert(numbering.nbr(id)) {
                        grew = true;
                    }
                }
            }
        }
        if !grew {
            return set;
        }
    }
}

/// to_positive_system (rootdata.cpp:1329-1347): the reflection steps
/// that make every entry of `delta` positive, applied as they are found.
fn to_positive_system(
    root_system: &RootSystem,
    numbering: &RootNumbering,
    delta: &mut [usize],
) -> Vec<(usize, usize)> {
    let mut steps = Vec::new();
    'scan: loop {
        for s in 0..delta.len() {
            if numbering.is_negative(delta[s]) {
                steps.push((s, delta[s]));
                let word = reflection_word(root_system, numbering, delta[s]);
                for entry in delta.iter_mut() {
                    *entry = word_act_root(root_system, numbering, &word, *entry);
                }
                continue 'scan;
            }
        }
        return steps;
    }
}

/// One FPP orbit node: a Weyl word and its root-lattice shift vectors.
type FppShiftOrbit = Vec<(Vec<usize>, Vec<Vec<i32>>)>;

/// weyl::FPP_w_shifts (alcoves.cpp:945-1058): the orbit of a fundamental
/// alcove point as (Weyl word, root-lattice shifts) pairs. Weyl words
/// stand for upstream WeylElts throughout (product = concatenation,
/// action = last letter first); display canonicalization happens at the
/// value boundary.
fn fpp_w_shifts(
    datum: &BasedRootDatum,
    root_system: &RootSystem,
    numbering: &RootNumbering,
    gamma: &RatVec,
) -> Result<FppShiftOrbit, String> {
    let rank = datum.lattice_rank();
    let semisimple = datum.semisimple_rank();
    let npos = numbering.npos;
    let denominator = gamma.denominator() as i64;
    let numer: Vec<i32> = gamma
        .numerators()
        .iter()
        .map(|&entry| entry as i32)
        .collect();
    let walls = fundamental_alcove_walls(root_system, numbering);
    let mut stabilising_walls = BTreeSet::new();
    for &alpha in &walls {
        let coroot = root_system
            .coroot(numbering.id(alpha))
            .expect("every root has a coroot");
        let mut evaluation: i64 = gamma
            .numerators()
            .iter()
            .zip(coroot.as_slice())
            .map(|(&g, &c)| g * i64::from(c))
            .sum();
        if !(npos..npos + semisimple).contains(&alpha) {
            evaluation += denominator; // ev += 1 for the affine walls
        }
        if evaluation == 0 {
            stabilising_walls.insert(alpha);
        }
    }
    let coset_lists = facet_orbit_ws(root_system, numbering, &stabilising_walls)?;
    let classifier = CenterClassifier::new(datum.cartan_matrix());
    // integrality_simples (rootdata.cpp:1494-1499): the simple roots of
    // the integral root system, in ascending RootNbr order.
    let integral_ids: Vec<RootId> = (0..root_system.roots().len())
        .map(RootId::from_usize)
        .filter(|&id| {
            root_system.is_positive(id).unwrap_or(false)
                && root_system.coroot(id).is_some_and(|coroot| {
                    gamma
                        .numerators()
                        .iter()
                        .zip(coroot.as_slice())
                        .map(|(&g, &c)| g * i64::from(c))
                        .sum::<i64>()
                        % denominator
                        == 0
                })
        })
        .collect();
    let simple_ids = simple_basis(root_system, &integral_ids)
        .map_err(|error| format!("integrality simple basis failed: {error}"))?;
    let mut delta: Vec<usize> = simple_ids.iter().map(|&id| numbering.nbr(id)).collect();
    delta.sort_unstable();
    let int_gens: Vec<Vec<usize>> = delta
        .iter()
        .map(|&nbr| reflection_word(root_system, numbering, nbr))
        .collect();
    let init = additive_closure(root_system, numbering, &stabilising_walls);

    #[derive(Clone)]
    struct State {
        w: Vec<usize>,
        image: Vec<usize>,
        integral_roots: BTreeSet<usize>,
        it: usize,
    }
    let mut states: Vec<State> = (0..=coset_lists.len())
        .map(|_| State {
            w: Vec::new(),
            image: delta.clone(),
            integral_roots: init.clone(),
            it: 0,
        })
        .collect();
    let mut result: FppShiftOrbit = Vec::new();
    loop {
        let mut w = states.last().expect("sentinel state").w.clone();
        let steps = to_positive_system(
            root_system,
            numbering,
            &mut states.last_mut().expect("sentinel state").image.clone(),
        );
        for &(first, _) in &steps {
            w.extend_from_slice(&int_gens[first]);
        }
        let mut image_weight = numer.clone();
        word_act_weight(datum, &w, &mut image_weight);
        let last = states.last().expect("sentinel state");
        let mut fix = 0u32;
        let mut ups = 0u32;
        let mut downs = 0u32;
        for s in 0..semisimple {
            if last.integral_roots.contains(&(npos + s)) {
                let evaluation: i32 = datum.simple_coroots()[s]
                    .as_slice()
                    .iter()
                    .zip(&image_weight)
                    .map(|(&c, &v)| c * v)
                    .sum();
                if evaluation >= 0 {
                    if evaluation == 0 {
                        ups |= 1 << s;
                    } else {
                        downs |= 1 << s;
                    }
                } else {
                    fix |= 1 << s;
                    ups |= 1 << s;
                }
            } else {
                // W.has_descent(s,w) is a LEFT descent (ell(s*w)<ell(w)),
                // i.e. w^{-1} sends alpha_s negative; the inverse element
                // acts through the reversed word.
                let mut inverse = w.clone();
                inverse.reverse();
                let image_root = word_act_root(root_system, numbering, &inverse, npos + s);
                if numbering.is_negative(image_root) {
                    fix |= 1 << s;
                }
            }
        }
        let mut node_shifts: Vec<Vec<i32>> = Vec::new();
        for shift in classifier.shifts(fix, ups, downs) {
            let mut v = vec![0i32; rank];
            for (s, &coefficient) in shift.iter().enumerate() {
                if coefficient != 0 {
                    for (slot, &coordinate) in v.iter_mut().zip(datum.simple_roots()[s].as_slice())
                    {
                        *slot += (coefficient as i32) * coordinate;
                    }
                }
            }
            node_shifts.push(v);
        }
        result.push((w, node_shifts));
        // Odometer increment over the coset lists (alcoves.cpp:1034-1050).
        let mut i = states.len();
        let exhausted = loop {
            let greater = i > 1;
            i -= 1;
            if !greater {
                break true;
            }
            states[i].it += 1;
            if states[i].it < coset_lists[i - 1].len() {
                break false;
            }
            states[i].it = 0;
        };
        if exhausted {
            break;
        }
        let word = coset_lists[i - 1][states[i].it].clone();
        let mut next_w = word.clone();
        next_w.extend_from_slice(&states[i - 1].w);
        let next_image: Vec<usize> = states[i - 1]
            .image
            .iter()
            .map(|&nbr| word_act_root(root_system, numbering, &word, nbr))
            .collect();
        let next_integrals: BTreeSet<usize> = states[i - 1]
            .integral_roots
            .iter()
            .map(|&nbr| word_act_root(root_system, numbering, &word, nbr))
            .collect();
        states[i].w = next_w;
        states[i].image = next_image;
        states[i].integral_roots = next_integrals;
        i += 1;
        while i < states.len() {
            states[i].w = states[i - 1].w.clone();
            states[i].image = states[i - 1].image.clone();
            states[i].integral_roots = states[i - 1].integral_roots.clone();
            i += 1;
        }
    }
    Ok(result)
}

/// The connected components of a Cartan matrix (index sets).
fn cartan_components(cartan: &[Vec<i32>]) -> Vec<Vec<usize>> {
    let rank = cartan.len();
    let mut seen = vec![false; rank];
    let mut components = Vec::new();
    for start in 0..rank {
        if seen[start] {
            continue;
        }
        let mut component = Vec::new();
        let mut pending = vec![start];
        seen[start] = true;
        while let Some(row) = pending.pop() {
            component.push(row);
            for column in 0..rank {
                if !seen[column] && (cartan[row][column] != 0 || cartan[column][row] != 0) {
                    seen[column] = true;
                    pending.push(column);
                }
            }
        }
        component.sort_unstable();
        components.push(component);
    }
    components
}

/// `CartanClass::makeSimpleComplex` (cartanclass.cpp:1002-1043): a choice
/// of simple roots for the complex factor `RC_0`, keeping one component of
/// every pair interchanged by the Cartan involution.
fn make_simple_complex(
    inner: &InnerClass,
    root_involution: &RootInvolutionData,
) -> Result<Vec<RootId>, StructureError> {
    let root_system = inner.root_system();
    let imaginary: Vec<RootId> = root_involution.roots_of_kind(RootKind::Imaginary).collect();
    let real: Vec<RootId> = root_involution.roots_of_kind(RootKind::Real).collect();
    let tri = two_rho(root_system, &imaginary);
    let trr = two_rho(root_system, &real);
    let orthogonal_roots: Vec<RootId> = (0..root_system.roots().len())
        .filter(|&index| {
            let id = RootId::from_usize(index);
            weight_orthogonal(root_system, &tri, id) && weight_orthogonal(root_system, &trr, id)
        })
        .map(RootId::from_usize)
        .collect();
    let basis = simple_basis(root_system, &orthogonal_roots)?;
    if basis.is_empty() {
        return Ok(Vec::new());
    }
    let cartan: Vec<Vec<i32>> = basis
        .iter()
        .map(|&root| {
            basis
                .iter()
                .map(|&coroot| root_system.bracket(root, coroot))
                .collect::<Result<Vec<_>, _>>()
        })
        .collect::<Result<Vec<_>, _>>()?;
    let components = cartan_components(&cartan);
    let mut result = Vec::new();
    let mut erased = vec![false; components.len()];
    for (index, component) in components.iter().enumerate() {
        if erased[index] {
            continue;
        }
        for &root_index in component {
            result.push(basis[root_index]);
        }
        let first = basis[component[0]];
        let image =
            root_involution
                .image(first)
                .ok_or_else(|| StructureError::IndexOutOfRange {
                    index: first.index(),
                    upper_bound: root_system.roots().len(),
                })?;
        let image_weight =
            root_system
                .root(image)
                .ok_or_else(|| StructureError::IndexOutOfRange {
                    index: image.index(),
                    upper_bound: root_system.roots().len(),
                })?;
        for later in (index + 1)..components.len() {
            if erased[later] {
                continue;
            }
            let pairs = components[later].iter().any(|&root_index| {
                let Some(coroot) = root_system.coroot(basis[root_index]) else {
                    return false;
                };
                pair(image_weight, coroot).is_ok_and(|pair| pair != 0)
            });
            if pairs {
                erased[later] = true;
                break;
            }
        }
    }
    Ok(result)
}

/// `RootSystem::subsystem_type` (rootdata.cpp:537-540): the Lie type of
/// the root subsystem spanned by `roots`, from its Cartan matrix.
fn subsystem_type_value(
    inner: &InnerClass,
    roots: &[RootId],
    span: SourceSpan,
) -> Result<Value, Diagnostic> {
    let root_system = inner.root_system();
    // The subsystem simple roots arrive in ambient-coordinate order; the
    // upstream `simpleBasis` returns datum root-number order (the long root
    // first for B2), so order by the first nonzero datum-simple coordinate.
    let mut ordered = roots.to_vec();
    ordered.sort_by_key(|&root| {
        let coordinates = root_system.simple_coordinates(root).unwrap_or_default();
        (
            coordinates
                .iter()
                .position(|&coordinate| coordinate != 0)
                .unwrap_or(usize::MAX),
            root.index(),
        )
    });
    let cartan: Vec<Vec<i32>> = ordered
        .iter()
        .map(|&root| {
            ordered
                .iter()
                .map(|&coroot| {
                    root_system
                        .bracket(root, coroot)
                        .map_err(|error| structure_diagnostic(error, span))
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .collect::<Result<Vec<_>, _>>()?;
    let lie_type = infer_lie_type(&cartan, cartan.len(), span)?;
    Ok(Value::Domain(DomainValue::LieType(lie_type)))
}

/// `Block_base::length_first` (blocks.cpp:250-260): the first block
/// element whose length is at least `l` (lower bound on the lengths).
fn block_length_first(graph: &BlockGraph, l: usize) -> usize {
    let (mut min, mut max) = (0_usize, graph.size());
    while max > min {
        let z = (min + max) / 2;
        if graph.length(z).is_some_and(|length| length >= l) {
            max = z;
        } else {
            min = z + 1;
        }
    }
    min
}

/// `KL_table::primitives` (kl.cpp:163-170): every primitive `x` for the
/// descent set of `y`, walked by `prim_back_up` from the length floor.
fn kl_primitives(kl_table: &KlTable, y: usize) -> Vec<usize> {
    let support = kl_table.support();
    let limit = support.length_floor(y);
    let desc_y = support.descent_set(y).clone();
    let mut result = Vec::new();
    let mut x = limit;
    while support.prim_back_up(&mut x, &desc_y) {
        result.push(x);
    }
    // prim_back_up walks downward, but upstream KL_table::primitives
    // collects into a BitMap whose iteration is ascending (kl.cpp:163-172,
    // consumed by kl_io::printPrimitiveKL kl_io.cpp:117). Reverse to match.
    result.reverse();
    result
}

/// `polynomials::compare` on the coefficient vectors (least degree first).
fn compare_klpol(a: &KlPol, b: &KlPol) -> std::cmp::Ordering {
    // polynomials::compare (polynomials_def.h:275-285): first by size
    // (coefficient count), then by coefficients from highest to lowest.
    let a_coeffs = a.as_slice();
    let b_coeffs = b.as_slice();
    if a_coeffs.len() != b_coeffs.len() {
        return a_coeffs.len().cmp(&b_coeffs.len());
    }
    for i in (0..a_coeffs.len()).rev() {
        let (ca, cb) = (a_coeffs[i], b_coeffs[i]);
        if ca != cb {
            return ca.cmp(&cb);
        }
    }
    std::cmp::Ordering::Equal
}

/// A matrix whose columns are the given lattice vectors — the by-columns
/// `matrix_value` constructor behind the simple_roots/posroots wrappers
/// (atlas-types.w:1631-1636).
fn columns_matrix_value(
    columns: &[Vec<i32>],
    row_count: usize,
    span: SourceSpan,
) -> Result<Value, Diagnostic> {
    let rows: Vec<Vec<i32>> = (0..row_count)
        .map(|row| columns.iter().map(|column| column[row]).collect())
        .collect();
    matrix_value(&rows, span)
}

fn datum_preference(name: &str, arguments: &[Value], span: SourceSpan) -> Result<bool, Diagnostic> {
    match arguments {
        [_] => Ok(false), // Temporary compatibility for the pre-typed evaluator.
        [_, Value::Boolean(preference)] => Ok(*preference),
        [_, other] => Err(type_error(
            span,
            format!("{name} expects a bool preference, found {other}"),
        )),
        _ => Err(type_error(
            span,
            format!("{name} expects 2 argument(s), found {}", arguments.len()),
        )),
    }
}

fn ratvec_from_rationals(
    rationals: Vec<BigRational>,
    span: SourceSpan,
) -> Result<RatVec, Diagnostic> {
    let mut denominator = BigInt::from(1);
    for rational in &rationals {
        let entry_denominator = BigInt::from(rational.denominator_ref().clone());
        let divisor = gcd_big(denominator.clone(), entry_denominator.clone());
        denominator = denominator * entry_denominator / divisor;
    }
    let numerators = rationals
        .iter()
        .map(|rational| {
            let scaled = BigRational::from(denominator.clone()) * rational.clone();
            let numerator = BigInt::try_from(scaled)
                .expect("scaling by the common denominator yields an integer");
            i64::try_from(&numerator)
                .map_err(|_| runtime(span, "Integer value to big for conversion"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let denominator = u64::try_from(&denominator)
        .map_err(|_| runtime(span, "Integer value to big for conversion"))?;
    RatVec::new(numerators, denominator)
        .ok_or_else(|| runtime(span, "ratvec denominator must be nonzero"))
}

fn gcd_big(mut left: BigInt, mut right: BigInt) -> BigInt {
    while right != 0 {
        let remainder = left % right.clone();
        left = right;
        right = remainder;
    }
    if left < 0 {
        -left
    } else {
        left
    }
}

fn as_real_form(value: &Value, span: SourceSpan) -> Result<&Arc<RealFormContext>, Diagnostic> {
    match value {
        Value::Domain(DomainValue::RealForm(context)) => Ok(context),
        other => Err(type_error(
            span,
            format!("expected a RealForm, found {other}"),
        )),
    }
}

fn as_kgb_element(
    value: &Value,
    span: SourceSpan,
) -> Result<(&Arc<RealFormContext>, KgbId), Diagnostic> {
    match value {
        Value::Domain(DomainValue::KgbElement(_, id)) if id.is_undefined() => {
            Err(runtime(span, "Inexistent KGB element"))
        }
        Value::Domain(DomainValue::KgbElement(context, id)) => Ok((context, *id)),
        other => Err(type_error(
            span,
            format!("expected a KGBElt, found {other}"),
        )),
    }
}

fn as_cartan_class(
    value: &Value,
    span: SourceSpan,
) -> Result<(&Arc<InnerClassContext>, CartanId), Diagnostic> {
    match value {
        Value::Domain(DomainValue::CartanClass(context, id)) => Ok((context, *id)),
        other => Err(type_error(
            span,
            format!("expected a CartanClass, found {other}"),
        )),
    }
}

fn as_block(value: &Value, span: SourceSpan) -> Result<&BlockValue, Diagnostic> {
    match value {
        Value::Domain(DomainValue::Block(block)) => Ok(block),
        other => Err(type_error(span, format!("expected a Block, found {other}"))),
    }
}

fn as_ktype(value: &Value, span: SourceSpan) -> Result<&KTypeValue, Diagnostic> {
    match value {
        Value::Domain(DomainValue::KType(ktype)) => Ok(ktype),
        other => Err(type_error(span, format!("expected a KType, found {other}"))),
    }
}

fn as_ktypepol(value: &Value, span: SourceSpan) -> Result<&KTypePolValue, Diagnostic> {
    match value {
        Value::Domain(DomainValue::KTypePol(pol)) => Ok(pol),
        other => Err(type_error(
            span,
            format!("expected a KTypePol, found {other}"),
        )),
    }
}

/// The language `vec` payload of an Atlas weight argument.
fn as_weight_vec(value: &Value, span: SourceSpan) -> Result<Vec<i32>, Diagnostic> {
    match value {
        Value::Vector(Vec32(entries)) => Ok(entries.clone()),
        other => Err(type_error(span, format!("expected a vec, found {other}"))),
    }
}

/// The language `ratvec` payload as the crate's gcd-normalized rational
/// weight (ratvec.cpp:172 normalization is shared by both layers).
fn as_rational_weight(value: &Value, span: SourceSpan) -> Result<RationalWeight, Diagnostic> {
    match value {
        Value::RatVector(factor) => RationalWeight::new(
            factor.numerators().to_vec(),
            i64::try_from(factor.denominator())
                .map_err(|_| runtime(span, "Integer value to big for conversion"))?,
        )
        .map_err(|error| runtime(span, error.to_string())),
        other => Err(type_error(
            span,
            format!("expected a ratvec, found {other}"),
        )),
    }
}

/// A crate rational weight as a language `ratvec` (both layers keep the
/// gcd-normalized common-denominator form, ratvec.cpp:172).
fn ratvec_from_rational_weight(
    weight: &RationalWeight,
    span: SourceSpan,
) -> Result<RatVec, Diagnostic> {
    let denominator = u64::try_from(weight.denominator())
        .map_err(|_| runtime(span, "Integer value to big for conversion"))?;
    RatVec::new(weight.numerator().to_vec(), denominator)
        .ok_or_else(|| runtime(span, "ratvec denominator must be nonzero"))
}

/// The `(numerator, denominator)` of an Atlas rational, narrowed to i64
/// (the parameter/polynomial scaling factors, repr.cpp:701-709).
fn rational_pair(value: &BigRational, span: SourceSpan) -> Result<(i64, i64), Diagnostic> {
    let numerator = i64::try_from(value.numerator_ref())
        .map_err(|_| runtime(span, "Integer value to big for conversion"))?;
    let denominator = i64::try_from(value.denominator_ref())
        .map_err(|_| runtime(span, "Integer value to big for conversion"))?;
    Ok((numerator, denominator))
}

/// Map a crate structure error to the runtime diagnostic wording.
fn structure_diagnostic(error: StructureError, span: SourceSpan) -> Diagnostic {
    runtime(span, error.to_string())
}

/// The owning-form identity check used by polynomial wrappers
/// (atlas-types.w:5668-5676, 7786-7803): the mismatch diagnostic precedes
/// the wrapper's no-value gate.
fn require_same_form_owner(
    left: &RealFormContext,
    right: &RealFormContext,
    message: &str,
    span: SourceSpan,
) -> Result<(), Diagnostic> {
    if same_real_form_owner(left, right) {
        Ok(())
    } else {
        Err(runtime(span, message))
    }
}

/// Structural real-form compatibility used by KType/Param equivalence
/// (atlas-types.w:5323-5331, 6340-6347). Unlike polynomial term ownership,
/// independently constructed custom forms with the same value are compatible.
fn require_same_form_value(
    left: &RealFormContext,
    right: &RealFormContext,
    message: &str,
    span: SourceSpan,
) -> Result<(), Diagnostic> {
    if same_real_form(left, right) {
        Ok(())
    } else {
        Err(runtime(span, message))
    }
}

/// The final-K-type expansion of one K-type with Split coefficients
/// (`Rep_context::finals_for`, K_repr.cpp:290-396).
fn finals_of_final(
    ktype: &KTypeValue,
    rc: &RepContext<'_>,
    span: SourceSpan,
) -> Result<Vec<(SplitValue, KType)>, Diagnostic> {
    let finals = ktype
        .ktype
        .finals_for(rc)
        .map_err(|error| structure_diagnostic(error, span))?;
    Ok(finals
        .into_iter()
        .map(|(term, coefficient)| (SplitValue::new(coefficient, 0), term))
        .collect())
}

/// The final-parameter expansion of one parameter (`Rep_context::
/// expand_final`, repr.cpp:1299-1309).
fn expand_final(
    parameter: &ParamValue,
    rc: &RepContext<'_>,
    span: SourceSpan,
) -> Result<Vec<(SplitValue, StandardRepr)>, Diagnostic> {
    let finals = rc
        .expand_final(&parameter.repr)
        .map_err(|error| structure_diagnostic(error, span))?;
    Ok(finals
        .into_iter()
        .map(|(term, coefficient)| (SplitValue::new(coefficient, 0), term))
        .collect())
}

/// Insert or merge one polynomial term (upstream
/// `K_type_pol::add_term` / `SR_poly::add_term`): like terms sum their
/// Split coefficients and a zero coefficient removes the term.
fn merge_pol_term<T: Clone + PartialEq>(
    terms: &mut Vec<(SplitValue, T)>,
    coefficient: SplitValue,
    term: T,
) {
    if coefficient.is_zero() {
        return;
    }
    if let Some(index) = terms.iter().position(|(_, existing)| *existing == term) {
        let updated = terms[index].0.add(coefficient);
        if updated.is_zero() {
            terms.remove(index);
        } else {
            terms[index].0 = updated;
        }
    } else {
        terms.push((coefficient, term));
    }
}

/// The upstream `K_type_pol` term order (K_repr.h:59-70): increasing
/// height, then increasing KGB element, then lambda-rho lexicographic.
fn sort_ktypepol_terms(terms: &mut [(SplitValue, KType)]) {
    terms.sort_by(|(_, left), (_, right)| {
        left.height()
            .cmp(&right.height())
            .then_with(|| left.x().index().cmp(&right.x().index()))
            .then_with(|| {
                left.lambda_rho()
                    .as_slice()
                    .cmp(right.lambda_rho().as_slice())
            })
    });
}

/// The upstream `SR_poly` term order (repr.cpp:41-54): increasing height,
/// then DECREASING KGB element, then the packed torsion part, then the
/// infinitesimal character cross-multiplied.
fn sort_parampol_terms(terms: &mut [(SplitValue, StandardRepr)]) {
    terms.sort_by(|(_, left), (_, right)| {
        left.height()
            .cmp(&right.height())
            .then_with(|| right.x().index().cmp(&left.x().index()))
            .then_with(|| {
                for index in 0..left.y_bits().dimension().max(right.y_bits().dimension()) {
                    let l = left.y_bits().bit(index).unwrap_or(false);
                    let r = right.y_bits().bit(index).unwrap_or(false);
                    if l != r {
                        return l.cmp(&r);
                    }
                }
                std::cmp::Ordering::Equal
            })
            .then_with(|| {
                let left_den = left.gamma().denominator();
                let right_den = right.gamma().denominator();
                left.gamma()
                    .numerator()
                    .iter()
                    .zip(right.gamma().numerator())
                    .map(|(&l, &r)| (l * right_den).cmp(&(r * left_den)))
                    .find(|ordering| *ordering != std::cmp::Ordering::Equal)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });
}

/// Merge adjacent equal terms after the caller has sorted by the polynomial's
/// canonical term order. This is the bulk counterpart of `merge_pol_term`:
/// list-valued addition stays O(n log n) instead of repeatedly scanning a
/// growing vector.
fn coalesce_sorted_terms<T: PartialEq>(terms: Vec<(SplitValue, T)>) -> Vec<(SplitValue, T)> {
    let mut merged: Vec<(SplitValue, T)> = Vec::with_capacity(terms.len());
    for (coefficient, term) in terms {
        if coefficient.is_zero() {
            continue;
        }
        if let Some((previous_coefficient, previous_term)) = merged.last_mut() {
            if *previous_term == term {
                let updated = previous_coefficient.add(coefficient);
                if updated.is_zero() {
                    merged.pop();
                } else {
                    *previous_coefficient = updated;
                }
                continue;
            }
        }
        merged.push((coefficient, term));
    }
    merged
}

/// Keep the KTypePol terms whose height is at most `bound`; a negative
/// bound keeps every term.
fn truncate_ktypepol(pol: &KTypePolValue, bound: i64) -> Vec<(SplitValue, KType)> {
    if bound < 0 {
        pol.terms.clone()
    } else {
        pol.terms
            .iter()
            .filter(|(_, ktype)| i64::from(ktype.height()) <= bound)
            .cloned()
            .collect()
    }
}

/// Keep the ParamPol terms whose height is at most `bound`; a negative
/// bound keeps every term.
fn truncate_parampol(pol: &ParamPolValue, bound: i64) -> Vec<(SplitValue, StandardRepr)> {
    if bound < 0 {
        pol.terms.clone()
    } else {
        pol.terms
            .iter()
            .filter(|(_, repr)| i64::from(repr.height()) <= bound)
            .cloned()
            .collect()
    }
}

/// The `is_dual` gate of `Fokko_block_wrapper` (atlas-types.w:4782-4793):
/// the second form's inner class must be the dual of the first form's. The
/// crate's structural dual-inner-class equality covers upstream's
/// datum-pointer and dual-distinguished-involution pair.
fn check_dual_pair(
    rf: &Arc<RealFormContext>,
    df: &Arc<RealFormContext>,
    span: SourceSpan,
) -> Result<(), Diagnostic> {
    let dual = dual_inner_class(&rf.parent.inner_class, WEYL_BUDGET, ROOT_BUDGET)
        .map_err(|error| runtime(span, error.to_string()))?;
    if dual != df.parent.inner_class {
        return Err(runtime(
            span,
            "Inner class mismatch between real form and dual real form",
        ));
    }
    Ok(())
}

/// The `Block::build(rf->val, dual_rf->val)` construction of `Block_value`
/// (atlas-types.w:4753-4756): the fibred product of the two forms' full
/// KGB sets, frozen with the owning contexts.
fn build_block(
    rf: &Arc<RealFormContext>,
    df: &Arc<RealFormContext>,
    span: SourceSpan,
) -> Result<BlockValue, Diagnostic> {
    let graph = BlockGraph::build(
        &rf.graph,
        &rf.table,
        &df.graph,
        &df.table,
        &df.parent.inner_class,
        WEYL_BUDGET,
    )
    .map_err(|error| runtime(span, error.to_string()))?;
    Ok(BlockValue {
        rf: Arc::clone(rf),
        dual_rf: Arc::clone(df),
        graph: Box::new(graph),
    })
}

/// The block wrappers' integer extraction: upstream narrows the Atlas int
/// to a C++ `int` (`int_val()` — "Integer value to big for conversion" on
/// overflow) and re-reads it as the unsigned `BlockElt`/`unsigned int`
/// (atlas-types.w:4829, 4893-4895, 4941-4943), so negative indices echo
/// wrapped in the out-of-range diagnostics.
fn as_wrapped_u32(value: &Value, span: SourceSpan) -> Result<u32, Diagnostic> {
    let integer = as_integer(value, span)?;
    let narrowed = i32::try_from(&integer)
        .map_err(|_| runtime(span, "Integer value to big for conversion"))?;
    Ok(narrowed as u32)
}

/// The shared generator gate of the four per-generator block wrappers
/// (atlas-types.w:4896-4900, 4924-4928, 4944-4948, 4970-4974).
fn block_generator_check(
    block: &BlockValue,
    value: &Value,
    span: SourceSpan,
) -> Result<usize, Diagnostic> {
    let generator = as_wrapped_u32(value, span)?;
    let rank = block.rf.graph.semisimple_rank();
    if generator as usize >= rank {
        return Err(runtime(
            span,
            format!("Illegal simple reflection: {generator}"),
        ));
    }
    Ok(generator as usize)
}

/// The shared element gate of the block wrappers (`Block element {i} out
/// of range (<{size})`): the wrapped index is echoed, as upstream prints
/// the unsigned `BlockElt`.
fn block_element_index(
    block: &BlockValue,
    value: &Value,
    span: SourceSpan,
) -> Result<usize, Diagnostic> {
    let index = as_wrapped_u32(value, span)?;
    let size = block.graph.size();
    if index as usize >= size {
        return Err(runtime(
            span,
            format!("Block element {index} out of range (<{size})"),
        ));
    }
    Ok(index as usize)
}

/// The fiber-compatibility gate of `block_index_wrapper`
/// (atlas-types.w:4865-4870): `dual_involution` of the x involution (in
/// the BLOCK's KGB) must equal the y involution (in the block's dual KGB).
fn block_fiber_check(
    block: &BlockValue,
    x: KgbId,
    y: KgbId,
    span: SourceSpan,
) -> Result<(), Diagnostic> {
    let mismatch = || runtime(span, "Fiber mismatch KGB and dual KGB elements");
    let x_involution = block
        .rf
        .graph
        .involution_of(x)
        .and_then(|involution| block.rf.table.record(involution))
        .ok_or_else(mismatch)?;
    let word = x_involution
        .weyl_element()
        .reduced_word(block.rf.table.root_system())
        .map_err(|error| runtime(span, error.to_string()))?;
    let dual_class = &block.dual_rf.parent.inner_class;
    let dual_twist = dual_class
        .generator_twist()
        .map_err(|error| runtime(span, error.to_string()))?;
    let longest = longest_action(dual_class, WEYL_BUDGET)
        .map_err(|error| runtime(span, error.to_string()))?;
    let dual_longest = WeylElement::from_action(dual_class.root_system(), &longest)
        .map_err(|error| runtime(span, error.to_string()))?;
    let dual_w = block_dual_involution(&word, dual_class.root_system(), &dual_twist, &dual_longest)
        .map_err(|error| runtime(span, error.to_string()))?;
    let y_involution = block
        .dual_rf
        .graph
        .involution_of(y)
        .and_then(|involution| block.dual_rf.table.record(involution))
        .ok_or_else(mismatch)?;
    if dual_w != *y_involution.weyl_element() {
        return Err(mismatch());
    }
    Ok(())
}

/// The language-visible number of a Cartan id: its position in the crate
/// Cartan order, which is the fundamental-first numbering upstream prints.
fn cartan_number(context: &InnerClassContext, id: CartanId) -> Option<usize> {
    context
        .classification
        .cartan_ids()
        .position(|other| other == id)
}

/// The shared out-of-range diagnostic of both `Cartan_class` wrappers
/// (atlas-types.w:4019-4029, 4040-4050): the signed index is echoed, and
/// `owner` selects the "inner class" / "real form" wording.
fn check_cartan_number(
    index: &BigInt,
    count: usize,
    owner: &str,
    span: SourceSpan,
) -> Result<usize, Diagnostic> {
    usize::try_from(index)
        .ok()
        .filter(|&number| number < count)
        .ok_or_else(|| {
            runtime(
                span,
                format!(
                    "Illegal Cartan class number: {index}, this {owner} only has {count} of them"
                ),
            )
        })
}

fn cartan_class_value(context: &Arc<InnerClassContext>, id: CartanId) -> Value {
    Value::Domain(DomainValue::CartanClass(Arc::clone(context), id))
}

/// The guard clauses of `fiber_partition_wrapper` (atlas-types.w:4199-4213):
/// both values must belong to one inner class, and the class must occur for
/// the real form. Runs before the upstream no-value gate, so `validate`
/// reuses it.
fn fiber_partition_membership(
    cartan_context: &Arc<InnerClassContext>,
    id: CartanId,
    form: &Arc<RealFormContext>,
    span: SourceSpan,
) -> Result<(), Diagnostic> {
    if cartan_context.inner_class != form.parent.inner_class {
        return Err(runtime(
            span,
            "Inner class mismatch between real form and Cartan class",
        ));
    }
    let occurs = form
        .parent
        .classification
        .cartan_set(form.internal)
        .expect("a real form's internal number is in range")
        .contains(&id);
    if !occurs {
        return Err(runtime(span, "Cartan class not defined for this real form"));
    }
    Ok(())
}

/// big_int::ulong_val (bigint.cpp:164-171): the unsigned extraction an int
/// argument goes through before `block_size_wrapper`'s bounds checks, so a
/// negative or over-wide value throws here, ahead of them.
fn as_unsigned_long(value: &Value, span: SourceSpan) -> Result<u64, Diagnostic> {
    let Value::Integer(value) = value else {
        return Err(type_error(span, format!("expected an int, found {value}")));
    };
    if *value < 0 {
        return Err(runtime(span, "Negative integer where unsigned is required"));
    }
    u64::try_from(value).map_err(|_| runtime(span, "Integer value to big for conversion"))
}

/// The guard clauses of `block_size_wrapper` (atlas-types.w:3337-3357): the
/// form numbers are extracted as unsigned longs and bounds-checked — real
/// form first — all before the upstream no-value gate, so `validate` reuses
/// this. Returns `(real form, dual real form)` external numbers.
fn block_size_numbers(
    context: &InnerClassContext,
    arguments: &[Value],
    span: SourceSpan,
) -> Result<(u64, u64), Diagnostic> {
    let form = as_unsigned_long(&arguments[1], span)?;
    let dual_form = as_unsigned_long(&arguments[2], span)?;
    if form >= context.order.form_count() as u64 {
        return Err(runtime(
            span,
            format!("Real form number {form} out of bounds"),
        ));
    }
    if dual_form >= context.dual_form_count as u64 {
        return Err(runtime(
            span,
            format!("Dual real form number {dual_form} out of bounds"),
        ));
    }
    Ok((form, dual_form))
}

/// fiberSize/dualFiberSize (innerclass.cpp:603-614): the class size of the
/// form's strong-real fiber orbit at one Cartan — the full fiber group
/// partitioned by central square classes (`fiber_partition(square_class)
/// .classSize`), NOT the adjoint weak partition. The B2 simply-connected
/// complex inner class separates the two (oracle 4/5/12 vs the adjoint
/// count 3/3/8).
fn fiber_size(
    strong: &StrongRealClassification,
    form: WeakRealFormId,
    cartan: CartanId,
    span: SourceSpan,
) -> Result<u64, Diagnostic> {
    let size = strong
        .fiber_size(form, cartan)
        .ok_or_else(|| runtime(span, "internal strong-real range error"))?;
    u64::try_from(size).map_err(|_| runtime(span, "internal fiber size overflow"))
}

/// InnerClass::block_size (innerclass.cpp:1100-1114): the sum, over the
/// Cartan classes shared by the real form and the dual real form, of
/// `orbitSize * fiberSize * dualFiberSize` — NOT a Block::build.
fn block_size_sum(
    context: &Arc<InnerClassContext>,
    dual: &Arc<InnerClassContext>,
    internal: WeakRealFormId,
    dual_internal: WeakRealFormId,
    span: SourceSpan,
) -> Result<u64, Diagnostic> {
    let cartan_set = context
        .classification
        .cartan_set(internal)
        .expect("a real form's internal number is in range");
    let dual_set = dual
        .classification
        .cartan_set(dual_internal)
        .expect("a real form's internal number is in range");
    let mut total = 0_u64;
    for (number, id) in context.classification.cartan_ids().enumerate() {
        if !cartan_set.contains(&id) {
            continue;
        }
        let (dual_id, _) = context
            .dual_cartans
            .get(number)
            .expect("the correspondence covers every Cartan class");
        if !dual_set.contains(dual_id) {
            continue;
        }
        let cartan = context
            .classification
            .cartan_class(id)
            .expect("Cartan ids enumerate in-range classes");
        let orbit = u64::try_from(cartan.twisted_involution_count())
            .map_err(|_| runtime(span, "internal block size overflow"))?;
        let factor = fiber_size(&context.strong, internal, id, span)?;
        let dual_factor = fiber_size(&dual.strong, dual_internal, *dual_id, span)?;
        let term = orbit
            .checked_mul(factor)
            .and_then(|product| product.checked_mul(dual_factor))
            .ok_or_else(|| runtime(span, "internal block size overflow"))?;
        total = total
            .checked_add(term)
            .ok_or_else(|| runtime(span, "internal block size overflow"))?;
    }
    Ok(total)
}

fn arity(
    name: &str,
    arguments: &[Value],
    expected: usize,
    span: SourceSpan,
) -> Result<(), Diagnostic> {
    if arguments.len() != expected {
        return Err(type_error(
            span,
            format!(
                "{name} expects {expected} argument(s), found {}",
                arguments.len()
            ),
        ));
    }
    Ok(())
}

/// The element's mod-two torus bits as a language `vec` of 0/1 entries,
/// in lattice coordinates (upstream `int_Vector` view of the torus part).
fn torus_bits_value(
    context: &Arc<RealFormContext>,
    id: KgbId,
    span: SourceSpan,
) -> Result<Value, Diagnostic> {
    let element = context
        .graph
        .element(id)
        .ok_or_else(|| runtime(span, "Inexistent KGB element"))?;
    let bits = element.torus_bits();
    let coordinates = (0..bits.dimension())
        .map(|index| i32::from(bits.bit(index) == Some(true)))
        .collect();
    Ok(Value::Vector(Vec32(coordinates)))
}

/// Combined forward-and-inverse Cayley (`any_Cayley`): real descents take
/// the FIRST inverse image, noncompact imaginary the forward image, and
/// everything else returns the argument unchanged (upstream parity).
fn any_cayley(
    context: &Arc<RealFormContext>,
    generator: usize,
    id: KgbId,
    span: SourceSpan,
) -> Result<KgbId, Diagnostic> {
    let graph = &context.graph;
    match graph
        .status(id, generator)
        .ok_or_else(|| runtime(span, "Inexistent KGB element"))?
    {
        KgbStatus::Real => Ok(graph
            .inverse_cayley(id, generator)
            .map_err(|error| runtime(span, error.to_string()))?
            .map(|(first, _)| first)
            .unwrap_or(id)),
        KgbStatus::ImaginaryNoncompact => Ok(graph
            .cayley(id, generator)
            .map_err(|error| runtime(span, error.to_string()))?
            .unwrap_or(id)),
        KgbStatus::Complex | KgbStatus::ImaginaryCompact => Ok(id),
    }
}

/// Upstream status coding: 0=C- 1=ic 2=r 3=nc 4=C+.
fn status_code(context: &Arc<RealFormContext>, generator: usize, id: KgbId) -> Option<i32> {
    let graph = &context.graph;
    Some(match graph.status(id, generator)? {
        KgbStatus::ImaginaryCompact => 1,
        KgbStatus::Real => 2,
        KgbStatus::ImaginaryNoncompact => 3,
        KgbStatus::Complex => {
            if graph.is_descent(id, generator)? {
                0
            } else {
                4
            }
        }
    })
}

/// C++ `RootSystem::root_compare` (rootdata.cpp:117-129): lexicographic with
/// the LAST simple coordinate most significant. It orders each height level
/// during positive-root generation, and with it the language-visible root
/// numbering.
#[derive(Clone, Eq, PartialEq)]
struct ByLastCoordinate(Vec<i32>);

impl Ord for ByLastCoordinate {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.iter().rev().cmp(other.0.iter().rev())
    }
}

impl PartialOrd for ByLastCoordinate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn dot_row(left: &[i32], right: &[i32]) -> i32 {
    left.iter().zip(right).map(|(&a, &b)| a * b).sum()
}

fn dot_column(vector: &[i32], matrix: &[Vec<i32>], column: usize) -> i32 {
    vector
        .iter()
        .enumerate()
        .map(|(row, &entry)| entry * matrix[row][column])
        .sum()
}

fn transpose(matrix: &[Vec<i32>]) -> Vec<Vec<i32>> {
    let size = matrix.len();
    (0..size)
        .map(|i| (0..size).map(|j| matrix[j][i]).collect())
        .collect()
}

/// Integer matrix product with wide accumulation: user matrices are
/// unbounded, and the products feed equality tests only (the twist
/// compatibility checks below).
fn integer_matrix_product(left: &[Vec<i32>], right: &[Vec<i32>]) -> Vec<Vec<i128>> {
    let columns = right.first().map_or(0, Vec::len);
    let mut product = vec![vec![0i128; columns]; left.len()];
    for (row, left_row) in left.iter().enumerate() {
        for (middle, &entry) in left_row.iter().enumerate() {
            for (column, &right_entry) in right[middle].iter().enumerate() {
                product[row][column] += i128::from(entry) * i128::from(right_entry);
            }
        }
    }
    product
}

fn is_identity_square(matrix: &[Vec<i32>]) -> bool {
    integer_matrix_product(matrix, matrix)
        .iter()
        .enumerate()
        .all(|(row, entries)| {
            entries
                .iter()
                .enumerate()
                .all(|(column, &entry)| entry == i128::from(row == column))
        })
}

/// The common wrapper-side checks of Atlas's involution constructors.
///
/// `mat` retains dimensions that cannot be recovered from `Vec<Vec<_>>`
/// (notably `0xN`), so inspect the value before adapting it to rows.  Atlas
/// elects the matrix row count as the expected rank for the standalone
/// classifier; datum-owned constructors supply their lattice rank instead.
fn checked_involution_matrix(
    value: &Value,
    expected_rank: Option<usize>,
    span: SourceSpan,
) -> Result<Vec<Vec<i32>>, Diagnostic> {
    let Value::Matrix(matrix) = value else {
        return Err(type_error(span, "expected a mat"));
    };
    let rows = matrix.rows();
    let columns = matrix.cols();
    let rank = expected_rank.unwrap_or(rows);
    if rows != rank || columns != rank {
        return Err(runtime(
            span,
            format!(
                "Involution should be a {rank}x{rank} matrix; received a {rows}x{columns} matrix"
            ),
        ));
    }
    let matrix = as_matrix_rows(value, span)?;
    if !is_identity_square(&matrix) {
        return Err(runtime(span, "Given transformation is not an involution"));
    }
    Ok(matrix)
}

/// `checked_permutation` (atlas-types.w:829-846): the `[int]` row must be a
/// permutation of `0..size`. Entries are read through the upstream unsigned
/// accessor, so a negative entry wraps to a huge value before the
/// too-big/repeated diagnostics.
fn checked_permutation(entries: &[Value], span: SourceSpan) -> Result<Vec<usize>, Diagnostic> {
    let size = entries.len();
    let mut seen = vec![false; size];
    let mut result = Vec::with_capacity(size);
    for value in entries {
        let integer = as_integer(value, span)?;
        let entry: u64 = i64::try_from(&integer)
            .map(|narrow| narrow as u64)
            .unwrap_or(u64::MAX);
        if entry >= size as u64 {
            return Err(runtime(span, format!("Permutation entry {entry} too big")));
        }
        let index = entry as usize;
        if seen[index] {
            return Err(runtime(
                span,
                format!("Permutation has repeated entry {entry}"),
            ));
        }
        seen[index] = true;
        result.push(index);
    }
    Ok(result)
}

/// The inner-class string argument of the primitive involution wrappers.
fn as_inner_class_symbols(value: &Value, span: SourceSpan) -> Result<&str, Diagnostic> {
    match value {
        Value::String(symbols) => Ok(symbols),
        other => Err(type_error(
            span,
            format!("expected a string of inner class symbols, found {other}"),
        )),
    }
}

/// `basic_involution_wrapper` (atlas-types.w:860-880): pack
/// `Layout{type, checked_inner_class_type(symbols), checked_permutation(perm)}`
/// and return `lietype::involution(lo)` on the simply-connected fundamental
/// weight basis. The permutation size check precedes the letter and entry
/// checks, in upstream's gate order.
fn basic_primitive_involution(
    lie_type: &LieTypeValue,
    entries: &[Value],
    symbols: &str,
    span: SourceSpan,
) -> Result<Value, Diagnostic> {
    let rank = lie_type.total_rank();
    if entries.len() != rank {
        return Err(runtime(
            span,
            format!(
                "Permutation size {} does not match rank {rank} of Lie type",
                entries.len()
            ),
        ));
    }
    let letters = checked_inner_class_letters(symbols, &lie_type.factors)
        .map_err(|error| runtime(span, error.to_string()))?;
    let perm = checked_permutation(entries, span)?;
    matrix_value(&layout_involution(&lie_type.factors, &letters, &perm), span)
}

/// `based_involution_wrapper` (atlas-types.w:902-927):
/// `lietype::involution(type, class).on_basis(basis)` with the identity
/// Bourbaki permutation; the inexact-division failure (and a singular basis)
/// is relabeled "Inner class is not compatible with given lattice". The
/// upstream shape guard reads `n_rows()!=r or n_rows()!=r` — the second
/// clause was meant for the column count (the message documents a square
/// matrix), so both dimensions are checked here.
fn based_primitive_involution(
    lie_type: &LieTypeValue,
    basis: &[Vec<i32>],
    symbols: &str,
    span: SourceSpan,
) -> Result<Value, Diagnostic> {
    let rank = lie_type.total_rank();
    if basis.len() != rank || basis.iter().any(|row| row.len() != rank) {
        return Err(runtime(
            span,
            format!("Basis should be given by {rank}x{rank} matrix"),
        ));
    }
    let letters = checked_inner_class_letters(symbols, &lie_type.factors)
        .map_err(|error| runtime(span, error.to_string()))?;
    let identity: Vec<usize> = (0..rank).collect();
    let involution = layout_involution(&lie_type.factors, &letters, &identity);
    let transported = lattice_on_basis(&involution, basis)
        .ok_or_else(|| runtime(span, "Inner class is not compatible with given lattice"))?;
    matrix_value(&transported, span)
}

/// Dispatch the two primitive `involution(LieType,...) -> mat` wrappers on
/// the second argument's shape: a row is the basic wrapper's permutation, a
/// matrix is the based wrapper's sublattice basis.
fn primitive_involution(
    lie_type: &LieTypeValue,
    lattice: &Value,
    symbols: &Value,
    span: SourceSpan,
) -> Result<Value, Diagnostic> {
    let symbols = as_inner_class_symbols(symbols, span)?;
    match lattice {
        Value::List(entries) => basic_primitive_involution(lie_type, entries, symbols, span),
        basis => {
            let basis = as_matrix_rows(basis, span)?;
            based_primitive_involution(lie_type, &basis, symbols, span)
        }
    }
}

/// Number a root like Atlas's `RootDatum` presentation: positive roots use
/// `0..npos`, while the negative of positive slot `p` is represented by the
/// unsigned value `UINT_MAX-p` in the wrapper diagnostic.
fn atlas_root_number(
    handle: &RootDatumHandle,
    image_root: &Weight,
    span: SourceSpan,
) -> Result<u32, Diagnostic> {
    let table = RootTable::build(handle, span)?;
    for (position, root) in table.roots.iter().enumerate() {
        let position =
            u32::try_from(position).map_err(|_| runtime(span, "internal root index overflow"))?;
        if root.as_slice() == image_root.as_slice() {
            return Ok(position);
        }
        if root
            .iter()
            .zip(image_root.as_slice())
            .all(|(&positive, &image)| i64::from(positive) == -i64::from(image))
        {
            return Ok(u32::MAX - position);
        }
    }
    Err(runtime(
        span,
        format!("root-system image {image_root:?} is not in the presentation table"),
    ))
}

fn twisted_involution_diagnostic(
    handle: &RootDatumHandle,
    error: StructureError,
    span: SourceSpan,
) -> Diagnostic {
    match error {
        StructureError::SimpleRootImageNotRoot { simple_root } => runtime(
            span,
            format!("Matrix maps simple root {simple_root} to non-root"),
        ),
        StructureError::SimpleCorootImageMismatch {
            simple_root,
            image_root,
        } => match atlas_root_number(handle, &image_root, span) {
            Ok(image) => runtime(
                span,
                format!("Matrix does not map simple coroot {simple_root} to coroot {image}"),
            ),
            Err(error) => error,
        },
        other => runtime(span, other.to_string()),
    }
}

/// Validate and normalize a user involution into the domain-owned factor and
/// fully built inner-class handle.  The language Weyl value is deliberately
/// not assembled here: Atlas performs this work before its no-value gate but
/// constructs the returned pair only when a value is requested.
fn build_twisted_involution(
    handle: &RootDatumHandle,
    value: &Value,
    span: SourceSpan,
) -> Result<(WeylElement, Arc<InnerClassContext>), Diagnostic> {
    let matrix = checked_involution_matrix(value, Some(handle.datum.lattice_rank()), span)?;
    let involution = LatticeInvolution::new(&handle.datum, matrix.clone(), transpose(&matrix))
        .map_err(|error| runtime(span, error.to_string()))?;
    let (factor, inner_class) =
        inner_class_with_twisted_involution((*handle.datum).clone(), involution, ROOT_BUDGET)
            .map_err(|error| twisted_involution_diagnostic(handle, error, span))?;
    let context = build_inner_class_context(handle, inner_class, span)?;
    Ok((factor, context))
}

/// Port of upstream `test_compatible` (interpreter/atlas-types.w:4625-4632)
/// with the wrapper's exact diagnostics: the user's matrix must be an
/// involution of the BASED root datum of the element's inner class — the
/// distinguished rejection rides on the crate's
/// [`atlas_real_group::InnerClass::based_involution_twist`] — and it must
/// commute with the class's distinguished involution. Returns the
/// validated involution and its induced simple-root twist.
fn compatible_outer_twist(
    context: &Arc<RealFormContext>,
    value: &Value,
    span: SourceSpan,
) -> Result<(LatticeInvolution, Vec<usize>), Diagnostic> {
    // The upstream size diagnostic prints the matrix's true shape; the row
    // adapter represents a 0xN matrix as N empty rows, so recover that one
    // case from the value itself.
    let zero_row =
        matches!(value, Value::Matrix(matrix) if matrix.rows() == 0 && matrix.cols() > 0);
    let matrix = as_matrix_rows(value, span)?;
    let inner_class = &context.parent.inner_class;
    let rank = inner_class.datum().lattice_rank();
    let columns = matrix.first().map_or(0, Vec::len);
    if matrix.len() != rank || columns != rank {
        let (rows, cols) = if zero_row {
            (0, matrix.len())
        } else {
            (matrix.len(), columns)
        };
        return Err(runtime(
            span,
            format!("Involution should be a {rank}x{rank} matrix; received a {rows}x{cols} matrix"),
        ));
    }
    if !is_identity_square(&matrix) {
        return Err(runtime(span, "Given transformation is not an involution"));
    }
    let delta = LatticeInvolution::new(inner_class.datum(), matrix.clone(), transpose(&matrix))
        .map_err(|error| runtime(span, error.to_string()))?;
    let twist = inner_class
        .based_involution_twist(delta.clone())
        .map_err(|error| {
            runtime(
                span,
                match error {
                    StructureError::InvalidBasedAutomorphism => {
                        "Root datum involution is not distinguished".to_string()
                    }
                    // check_root_datum_involution wordings
                    // (atlas-types.w:2764-2785).
                    StructureError::SimpleRootImageNotRoot { simple_root } => {
                        format!("Matrix maps simple root {simple_root} to non-root")
                    }
                    other => other.to_string(),
                },
            )
        })?;
    let distinguished = inner_class
        .distinguished_involution()
        .involution()
        .weight_matrix();
    if integer_matrix_product(&matrix, distinguished)
        != integer_matrix_product(distinguished, &matrix)
    {
        return Err(runtime(span, "Non commuting distinguished involution"));
    }
    Ok((delta, twist))
}

/// Shared gates of `shift_flip_wrapper` (interpreter/atlas-types.w:
/// 7341-7362): `test_compatible`, then the two delta-fix checks, in the
/// upstream order and with the upstream wordings — the user's rational
/// weight first, then the parameter's own infinitesimal character. All
/// three precede the wrapper's no_value gate, so both `call` and
/// `validate` run them. Returns the validated involution and the user's
/// gamma as a rational weight.
fn shift_flip_gates(
    parameter: &ParamValue,
    matrix: &Value,
    gamma: &Value,
    span: SourceSpan,
) -> Result<(LatticeInvolution, RationalWeight), Diagnostic> {
    let (delta, _twist) = compatible_outer_twist(&parameter.context, matrix, span)?;
    let gamma = as_rational_weight(gamma, span)?;
    if !involution_fixes_gamma(delta.weight_matrix(), &gamma) {
        // atlas-types.w:7351-7352.
        return Err(runtime(span, "Involution does not fix rational weight"));
    }
    if !involution_fixes_gamma(delta.weight_matrix(), parameter.repr.gamma()) {
        // atlas-types.w:7353-7354.
        return Err(runtime(
            span,
            "Involution does not fix infinitesimal character",
        ));
    }
    Ok((delta, gamma))
}

/// `Rep_context::is_fixed` (gkmod/repr.cpp:669-675) as the ext_finalise
/// wrappers use it: normalise the parameter (`make_dominant`, then move to
/// the singular-canonical involution) and require the delta-twist to be
/// the identity — `z == twisted(z,delta)` (repr.cpp:1132-1143).
///
/// The crate's `RepContext::is_fixed` is the raw gamma check of the
/// ext_block wrappers (extended_block uses
/// `(1-delta)*gamma.numerator()==0` directly, atlas-types.w:7372), NOT
/// this predicate, and the crate keeps `to_singular_canonical` and the
/// `y_act` transport private. The language layer therefore normalises
/// with the public `StandardRepr::normalised` (repr.cpp:659-667 — the
/// same two steps plus the singular complex descent crosses, a
/// deterministic function of the canonical form). For the comparison it
/// uses a convention-free equivalent of the `y_act(i,i,y,delta) == y`
/// condition: `twisted(z,delta)` has `lambda_rho' == delta*lambda_rho`
/// (its `y_lift' == (1-theta)*delta*y_lift/2`, and `(1+theta)*y_lift == 0`
/// since theta is an involution), so the twisted parameter is rebuilt
/// with `RepContext::sr_gamma` — which repacks `y_bits` from `lambda_rho`
/// through the crate-private `y_pack` — and compared componentwise
/// (`x`, `y_bits`, `gamma`; repr.cpp:36-40), exactly `z == twisted(z,delta)`.
fn ext_is_fixed(
    context: &Arc<RealFormContext>,
    parameter: &StandardRepr,
    delta: &LatticeInvolution,
    twist: &[usize],
    span: SourceSpan,
) -> Result<bool, Diagnostic> {
    let rc = rep_context(context);
    let z = parameter
        .normalised(&rc)
        .map_err(|error| structure_diagnostic(error, span))?;
    // x: the twisted element must be z's own; `None` is upstream's
    // UndefKGB, which never equals a real element number.
    let Some(twisted_x) = context
        .graph
        .twisted(z.x(), &context.table, delta, twist)
        .map_err(|error| structure_diagnostic(error, span))?
    else {
        return Ok(false);
    };
    let matrix = delta.weight_matrix();
    // gamma: the twisted infinitesimal character is delta*gamma.
    let gamma = z.gamma();
    let gamma_image: Vec<i64> = matrix
        .iter()
        .map(|row| {
            row.iter()
                .zip(gamma.numerator())
                .map(|(&factor, &coordinate)| i64::from(factor) * coordinate)
                .sum()
        })
        .collect();
    let twisted_gamma = RationalWeight::new(gamma_image, gamma.denominator())
        .map_err(|error| structure_diagnostic(error, span))?;
    // lambda_rho: the twisted lambda_rho is delta*lambda_rho (doc comment).
    // Rebuild twisted(z,delta) with `sr_gamma` — it packs y_bits from
    // lambda_rho via the crate-private `y_pack` — and compare (x, y_bits,
    // gamma; repr.cpp:36-40), exactly upstream's `z == twisted(z,delta)`.
    let lambda_rho = rc
        .lambda_rho(&z)
        .map_err(|error| structure_diagnostic(error, span))?;
    let mut lambda_image = Vec::with_capacity(matrix.len());
    for row in matrix {
        let image: i64 = row
            .iter()
            .zip(lambda_rho.as_slice())
            .map(|(&factor, &coordinate)| i64::from(factor) * i64::from(coordinate))
            .sum();
        lambda_image.push(
            i32::try_from(image)
                .map_err(|_| structure_diagnostic(StructureError::ArithmeticOverflow, span))?,
        );
    }
    match rc.sr_gamma(twisted_x, &Weight::new(lambda_image), &twisted_gamma) {
        Ok(twisted) => Ok(twisted == z),
        // A twisted datum whose torsion part does not pack is not a valid
        // parameter, hence cannot equal z.
        Err(_) => Ok(false),
    }
}

/// Gates of `scale_extended_wrapper` (interpreter/atlas-types.w:8449-8472),
/// in the upstream order: `test_final`, then the strictly-positive factor
/// check (`is_positive`, so 0 is rejected), then `test_compatible`, then
/// the delta-fix test on the parameter. All four precede the wrapper's
/// no_value gate, so both `call` and `validate` run them. Returns the
/// validated involution and the narrowed factor.
fn scale_extended_gates(
    parameter: &ParamValue,
    matrix: &Value,
    factor: &BigRational,
    span: SourceSpan,
) -> Result<(LatticeInvolution, i64, i64), Diagnostic> {
    test_final(parameter, "Cannot scale extended parameter", span)?;
    let (numerator, denominator) = rational_pair(factor, span)?;
    if numerator <= 0 {
        return Err(runtime(span, "Factor in scale_extended must be positive"));
    }
    let (delta, twist) = compatible_outer_twist(&parameter.context, matrix, span)?;
    if !ext_is_fixed(&parameter.context, &parameter.repr, &delta, &twist, span)? {
        return Err(runtime(
            span,
            "Parameter to be scaled not fixed by given involution",
        ));
    }
    Ok((delta, numerator, denominator))
}

/// Gates of `K_type_pol_extended_wrapper` (interpreter/atlas-types.w:
/// 8487-8500): `test_standard` (whose descr carries the upstream literal
/// "|" typo, preserved verbatim in the recorded oracle events), then
/// `test_compatible`, then the delta-fix test.
fn k_type_pol_extended_gates(
    parameter: &ParamValue,
    matrix: &Value,
    span: SourceSpan,
) -> Result<LatticeInvolution, Diagnostic> {
    test_standard(
        parameter,
        "Parameter in K_type_pol_extended| must be standard",
        span,
    )?;
    let (delta, twist) = compatible_outer_twist(&parameter.context, matrix, span)?;
    if !ext_is_fixed(&parameter.context, &parameter.repr, &delta, &twist, span)? {
        return Err(runtime(span, "Parameter not fixed by given involution"));
    }
    Ok(delta)
}

/// Gates of `finalize_extended_wrapper` (interpreter/atlas-types.w:
/// 8514-8537): `test_standard`, `test_compatible`, the delta-fix test, and
/// finally the commutation of the parameter's Cartan involution with
/// delta.
fn finalize_extended_gates(
    parameter: &ParamValue,
    matrix: &Value,
    span: SourceSpan,
) -> Result<LatticeInvolution, Diagnostic> {
    test_standard(parameter, "Cannot finalize extended parameter", span)?;
    let (delta, twist) = compatible_outer_twist(&parameter.context, matrix, span)?;
    if !ext_is_fixed(&parameter.context, &parameter.repr, &delta, &twist, span)? {
        return Err(runtime(span, "Parameter not fixed by given involution"));
    }
    // atlas-types.w:8528-8532: theta = i_tab.matrix(kgb.involution(x)).
    let rc = rep_context(&parameter.context);
    let theta = rc
        .theta(&parameter.repr)
        .map_err(|error| structure_diagnostic(error, span))?;
    if integer_matrix_product(theta.weight_matrix(), delta.weight_matrix())
        != integer_matrix_product(delta.weight_matrix(), theta.weight_matrix())
    {
        return Err(runtime(
            span,
            "Involution of parameter does not commute with delta",
        ));
    }
    Ok(delta)
}

/// Gates of `twisted_deform_wrapper` (interpreter/atlas-types.w:
/// 8120-8134), in the upstream order: `test_standard`, the
/// distinguished-involution fix check, then `test_final`. All three
/// precede the wrapper's no_value gate, so both `call` and `validate`
/// run them.
fn twisted_deform_gates(parameter: &ParamValue, span: SourceSpan) -> Result<(), Diagnostic> {
    test_standard(parameter, "Cannot compute twisted deformation terms", span)?;
    let rc = rep_context(&parameter.context);
    if !rc.is_delta_fixed(&parameter.repr) {
        return Err(runtime(
            span,
            "Parameter not fixed by inner class involution",
        ));
    }
    test_final(
        parameter,
        "Twisted deformation requires final parameter",
        span,
    )?;
    Ok(())
}

/// Gates of `twisted_full_deform_wrapper` (atlas-types.w:8229-8240) —
/// and of the timed second overload (:8609-8621), whose preconditions
/// are identical: `test_standard`, then the distinguished-involution
/// fix check. No `test_final` (unlike `twisted_deform`).
fn twisted_full_deform_gates(parameter: &ParamValue, span: SourceSpan) -> Result<(), Diagnostic> {
    test_standard(parameter, "Cannot compute full twisted deformation", span)?;
    let rc = rep_context(&parameter.context);
    if !rc.is_delta_fixed(&parameter.repr) {
        return Err(runtime(
            span,
            "Parameter not fixed by inner class involution",
        ));
    }
    Ok(())
}

/// Gates of the distinguished-involution `twisted_KL_sum_at_s_wrapper`
/// (atlas-types.w:8370-8382): `test_standard` and `test_final`, BOTH
/// with descr "Cannot compute Kazhdan-Lusztig sum", then the delta-fix
/// check on a made-dominant COPY of the parameter (the computation runs
/// on that copy). Returns the dominant parameter.
fn twisted_kl_sum_gates(
    parameter: &ParamValue,
    span: SourceSpan,
) -> Result<StandardRepr, Diagnostic> {
    test_standard(parameter, "Cannot compute Kazhdan-Lusztig sum", span)?;
    test_final(parameter, "Cannot compute Kazhdan-Lusztig sum", span)?;
    let rc = rep_context(&parameter.context);
    let sr = parameter
        .repr
        .made_dominant(&rc)
        .map_err(|error| structure_diagnostic(error, span))?;
    if !rc.is_delta_fixed(&sr) {
        return Err(runtime(
            span,
            "Parameter not fixed by inner class involution",
        ));
    }
    Ok(sr)
}

/// Gates of `external_twisted_KL_sum_at_s_wrapper` (atlas-types.w:
/// 8420-8431): the same `test_standard`/`test_final` pair, then
/// `test_compatible`, then the fix check against the USER's involution
/// (`Rep_context::is_fixed`, ported as [`ext_is_fixed`]). Returns the
/// validated involution and its simple-root twist.
fn external_twisted_kl_sum_gates(
    parameter: &ParamValue,
    matrix: &Value,
    span: SourceSpan,
) -> Result<(LatticeInvolution, Vec<usize>), Diagnostic> {
    test_standard(parameter, "Cannot compute Kazhdan-Lusztig sum", span)?;
    test_final(parameter, "Cannot compute Kazhdan-Lusztig sum", span)?;
    let (delta, twist) = compatible_outer_twist(&parameter.context, matrix, span)?;
    if !ext_is_fixed(&parameter.context, &parameter.repr, &delta, &twist, span)? {
        return Err(runtime(span, "Parameter not fixed by given involution"));
    }
    Ok((delta, twist))
}

/// The parameter's full block against the dual quasisplit form — the
/// `lookup_full_block` shape shared by the deform arms (matching the
/// verified `deform` arm).
fn full_block_of(parameter: &ParamValue, span: SourceSpan) -> Result<BlockValue, Diagnostic> {
    let dual_parent = build_dual_inner_class(&parameter.context.parent, span)?;
    let dual_quasisplit = dual_parent.order.quasisplit_external();
    let dual_rf = build_real_form(&dual_parent, dual_quasisplit, span)?;
    build_block(&parameter.context, &dual_rf, span)
}

/// The loud rejection of a proper-integral-subsystem common block (no
/// `SubSystem`/`simp_int` port exists; the crate's
/// [`IntegralBlockScope::ProperSubsystem`] case).
fn proper_subsystem_diagnostic(span: SourceSpan) -> Diagnostic {
    structure_diagnostic(
        StructureError::NotYetImplemented {
            feature: "common block on a proper integral subsystem",
        },
        span,
    )
}

/// Locate `sr` in its full block, as `Rep_table::lookup` reports it: an
/// x-coordinate match that is present in the extended block and whose
/// reconstruction at `sr`'s own data equals `sr` (the `block_element_of`
/// helper semantics of the crate's deform.rs tests).
fn twisted_block_index(
    block: &BlockGraph,
    eblock: &ExtBlock,
    rc: &RepContext<'_>,
    sr: &StandardRepr,
    lambda_rho: &Weight,
    span: SourceSpan,
) -> Result<usize, Diagnostic> {
    (0..block.size())
        .find(|&z| {
            block.x(z) == Some(sr.x())
                && eblock.is_present(z)
                && rc.sr_gamma(sr.x(), lambda_rho, sr.gamma()).ok().as_ref() == Some(sr)
        })
        .ok_or_else(|| runtime(span, "parameter not in the common block"))
}

/// The `rt.lookup(zi, index, bm)` + `block.extended_block(bm, ...)` step
/// of `Rep_table::twisted_deformation` at a reducibility point with
/// INTEGRAL gamma (repr.cpp:2617-2633, trivial block modifier): rebuild
/// `zi`'s full block against the dual class's quasisplit form, the
/// extended block over `ctx`'s delta, and `zi`'s parent block index.
/// Crate calls only, so the closure [`twisted_deformation`] expects can
/// stay `StructureError`-typed.
fn twisted_reducibility_lookup(
    context: &Arc<RealFormContext>,
    rc: &RepContext<'_>,
    delta: &LatticeInvolution,
    twist: &[usize],
    zi: &StandardRepr,
) -> Result<(BlockGraph, ExtBlock, usize), StructureError> {
    let parent = &context.parent;
    let dual_inner = dual_inner_class(&parent.inner_class, WEYL_BUDGET, ROOT_BUDGET)?;
    let dual_classification =
        CartanClassification::build(&dual_inner, &cartan_classification_budget())?;
    let dual_strong = StrongRealClassification::build(&dual_classification, FIBER_BUDGET)?;
    let dual_order = ExternalFormOrder::build(&dual_inner, &dual_classification)?;
    let dual_internal = dual_order
        .internal(dual_order.quasisplit_external())
        .ok_or(StructureError::RepInvariantViolation {
            invariant: "dual inner class has no quasisplit form",
        })?;
    let mut dual_table = InvolutionTable::new(
        &dual_inner,
        InvolutionTableBudget::new(FIBER_BUDGET, INTEGER_BUDGET),
    )?;
    let fundamental =
        dual_classification
            .cartan_ids()
            .next()
            .ok_or(StructureError::RepInvariantViolation {
                invariant: "empty Cartan classification",
            })?;
    dual_table.add_cartan(&dual_classification, fundamental)?;
    let seed = RealFormSeed::build(
        &dual_inner,
        &dual_classification,
        &dual_strong,
        &dual_table,
        dual_internal,
        &INTEGER_BUDGET,
        FIBER_BUDGET,
    )?;
    let dual_graph = KgbGraph::build(
        &dual_inner,
        &dual_classification,
        &dual_strong,
        &mut dual_table,
        &seed,
    )?;
    let block = BlockGraph::build(
        &context.graph,
        &context.table,
        &dual_graph,
        &dual_table,
        &dual_inner,
        WEYL_BUDGET,
    )?;
    // build_ext_block's dual twist data (ext_block.cpp:618-668).
    let matrix = delta.weight_matrix();
    let dual_delta =
        LatticeInvolution::new(dual_inner.datum(), transpose(matrix), matrix.to_vec())?;
    let dual_twist = dual_inner.based_involution_twist(dual_delta.clone())?;
    let eblock = ExtBlock::build(
        &block,
        &context.graph,
        &context.table,
        &dual_graph,
        &dual_table,
        delta,
        twist,
        &dual_delta,
        &dual_twist,
        parent.root_datum.datum.cartan_matrix(),
    )?;
    let lambda_rho = rc.lambda_rho(zi)?;
    let index = (0..block.size())
        .find(|&z| {
            block.x(z) == Some(zi.x())
                && eblock.is_present(z)
                && rc.sr_gamma(zi.x(), &lambda_rho, zi.gamma()).ok().as_ref() == Some(zi)
        })
        .ok_or(StructureError::RepInvariantViolation {
            invariant: "twisted deformation: reducibility parameter not in its block",
        })?;
    Ok((block, eblock, index))
}

/// Shared tail of the three twisted wrappers after their gates: run
/// `compute` on the full block plus the extended block over `delta`, or
/// short-circuit the rank-0 integral subsystem (the common block is the
/// singleton `{p}` of length 0 — empty deformation terms, and `1*p` for
/// the KL sums, repr.cpp:2435-2436). A proper integral subsystem fails
/// loudly (no `SubSystem` port).
fn with_integral_block<T>(
    parameter: &ParamValue,
    rc: &RepContext<'_>,
    sr: &StandardRepr,
    twist_data: &(LatticeInvolution, Vec<usize>),
    span: SourceSpan,
    singleton: impl FnOnce() -> T,
    compute: impl FnOnce(&BlockGraph, &ExtBlock, usize, &Weight) -> Result<T, Diagnostic>,
) -> Result<T, Diagnostic> {
    match integral_block_scope(rc, sr.gamma()).map_err(|error| structure_diagnostic(error, span))? {
        IntegralBlockScope::Singleton => Ok(singleton()),
        IntegralBlockScope::ProperSubsystem => Err(proper_subsystem_diagnostic(span)),
        IntegralBlockScope::Full => {
            let block = full_block_of(parameter, span)?;
            let eblock = build_ext_block(&block, parameter, &twist_data.0, &twist_data.1, span)?;
            let lambda_rho = rc
                .lambda_rho(sr)
                .map_err(|error| structure_diagnostic(error, span))?;
            let y0 = twisted_block_index(&block.graph, &eblock, rc, sr, &lambda_rho, span)?;
            compute(&block.graph, &eblock, y0, &lambda_rho)
        }
    }
}

/// Shared tail of both `twist` wrappers: apply the crate twist and rewrap
/// the target in the same real-form context. Upstream's `UndefKGB` remains
/// language-visible as element number `~0u`; it is never graph-indexed.
fn twist_element(
    context: &Arc<RealFormContext>,
    id: KgbId,
    delta: &LatticeInvolution,
    twist: &[usize],
    span: SourceSpan,
) -> Result<Value, Diagnostic> {
    if id.is_undefined() {
        return Err(runtime(span, "Inexistent KGB element"));
    }
    let target = context
        .graph
        .twisted(id, &context.table, delta, twist)
        .map_err(|error| runtime(span, error.to_string()))?
        .unwrap_or(KgbId::UNDEFINED);
    Ok(Value::Domain(DomainValue::KgbElement(
        Arc::clone(context),
        target,
    )))
}

/// Rewrap a crate parameter twist in its original real-form owner, preserving
/// both the owner and an explicit `UndefKGB` representation.
fn twist_parameter(
    parameter: &ParamValue,
    twisted: Result<StandardRepr, StructureError>,
    span: SourceSpan,
) -> Result<Value, Diagnostic> {
    let repr = twisted.map_err(|error| match error {
        // Rep_context::make_dominant throws this exact prose upstream
        // (repr.cpp:577); the crate keeps the stable invariant key.
        StructureError::RepInvariantViolation {
            invariant: "standard parameter in make_dominant",
        } => runtime(span, "Non standard parameter in make_dominant"),
        other => structure_diagnostic(other, span),
    })?;
    Ok(Value::Domain(DomainValue::Param(ParamValue {
        context: Arc::clone(&parameter.context),
        repr,
    })))
}

/// Shared body of the synthetic KGB constructor `KGB_elt(RealForm,mat,
/// ratvec)` (build_KGB_element_wrapper, interpreter/atlas-types.w:4580-4607).
/// Every diagnostic fires before the wrapper's no_value gate, so `validate`
/// runs the same pipeline and drops the result. The arithmetic order is the
/// upstream one: size check, torus-factor symmetrization with its
/// denominator rejection, THEN `twisted_from_involution` with its
/// involution and inner-class checks, then the per-form KGB lookup.
fn build_kgb_element(
    context: &Arc<RealFormContext>,
    theta: &Value,
    factor: &Value,
    span: SourceSpan,
) -> Result<KgbId, Diagnostic> {
    let inner_class = &context.parent.inner_class;
    let rank = inner_class.datum().lattice_rank();
    let Value::RatVector(factor) = factor else {
        return Err(type_error(
            span,
            format!("expected a ratvec, found {factor}"),
        ));
    };
    if factor.numerators().len() != rank {
        return Err(runtime(span, "Torus factor size mismatch"));
    }
    // Upstream applies right_prod to the raw matrix before any theta check;
    // a non-square matrix aborts there, so the classify chunk's shape
    // diagnostic (atlas-types.w:2723-2729) is the closest defined behavior.
    let zero_row =
        matches!(theta, Value::Matrix(matrix) if matrix.rows() == 0 && matrix.cols() > 0);
    let matrix = as_matrix_rows(theta, span)?;
    let columns = matrix.first().map_or(0, Vec::len);
    if matrix.len() != rank || columns != rank {
        let (rows, cols) = if zero_row {
            (0, matrix.len())
        } else {
            (matrix.len(), columns)
        };
        return Err(runtime(
            span,
            format!("Involution should be a {rank}x{rank} matrix; received a {rows}x{cols} matrix"),
        ));
    }
    let factor: Vec<BigRational> = factor
        .numerators()
        .iter()
        .map(|&numerator| BigRational::from(numerator) / BigRational::from(factor.denominator()))
        .collect();
    let Some(bits) = context
        .graph
        .seed_torus_part(&matrix, &factor)
        .map_err(|error| runtime(span, error.to_string()))?
    else {
        return Err(runtime(
            span,
            "Torus factor not in cocharacter coset of real form",
        ));
    };
    if !is_identity_square(&matrix) {
        return Err(runtime(span, "Given transformation is not an involution"));
    }
    let involution =
        LatticeInvolution::new(inner_class.datum(), matrix.clone(), transpose(&matrix))
            .map_err(|error| runtime(span, error.to_string()))?;
    let element = inner_class
        .twisted_from_involution(involution)
        .map_err(|error| {
            runtime(
                span,
                match error {
                    StructureError::InvalidBasedAutomorphism => {
                        "Involution not in this inner class".to_string()
                    }
                    other => other.to_string(),
                },
            )
        })?;
    // A twisted involution whose Cartan the form does not meet is upstream's
    // empty tau packet: UndefKGB either way.
    let Some(involution_id) = context.table.lookup(&element) else {
        return Err(runtime(span, "KGB element not present"));
    };
    context
        .graph
        .lookup(&context.table, involution_id, bits)
        .map_err(|error| runtime(span, error.to_string()))?
        .ok_or_else(|| runtime(span, "KGB element not present"))
}

/// The seed plan of a synthetic real form: the weak form, the elected
/// cocharacter, and the minimal torus part, plus whether
/// `real_form_value::build` (atlas-types.w:3534-3545) drops to the shared
/// default construction because the pair coincides with the elected seed.
struct SyntheticRealForm {
    external: usize,
    internal: WeakRealFormId,
    cocharacter: Vec<BigRational>,
    torus_part: ModTwoVector,
    default_seed: bool,
}

/// Shared body of the synthetic real-form constructor
/// `real_form(InnerClass,mat,ratvec)` (synthetic_real_form_wrapper,
/// interpreter/atlas-types.w:3851-3871): the weak real form claiming the
/// (involution, torus factor) datum, together with the seed
/// `real_form_value::build` stores. Every diagnostic fires before the
/// wrapper's no_value gate, so `validate` runs the same pipeline and drops
/// the result. The order is the upstream one — size check,
/// `twisted_from_involution` with its shape, involution, and inner-class
/// checks, THEN the doubled theta-fixed projection with its centrality
/// parity test and halving — the exact reverse of `build_KGB_element`'s
/// arithmetic-first order. After classification the wrapper computes
/// `real_form_of`'s `coch` output and `minimal_torus_part`
/// (realredgp.cpp:212-309), and `build` elects the shared default value
/// only when the pair equals the form's elected seed
/// (atlas-types.w:3534-3545); the caller builds the custom-seed KGB
/// pipeline otherwise.
fn synthetic_real_form(
    context: &Arc<InnerClassContext>,
    theta: &Value,
    factor: &Value,
    span: SourceSpan,
) -> Result<SyntheticRealForm, Diagnostic> {
    let inner_class = &context.inner_class;
    let rank = inner_class.datum().lattice_rank();
    let Value::RatVector(factor) = factor else {
        return Err(type_error(
            span,
            format!("expected a ratvec, found {factor}"),
        ));
    };
    if factor.numerators().len() != rank {
        return Err(runtime(span, "Torus factor size mismatch"));
    }
    // The upstream shape diagnostic prints the matrix's true shape; the
    // row adapter represents a 0xN matrix as N empty rows, so recover
    // that one case from the value itself.
    let zero_row =
        matches!(theta, Value::Matrix(matrix) if matrix.rows() == 0 && matrix.cols() > 0);
    let matrix = as_matrix_rows(theta, span)?;
    let columns = matrix.first().map_or(0, Vec::len);
    if matrix.len() != rank || columns != rank {
        let (rows, cols) = if zero_row {
            (0, matrix.len())
        } else {
            (matrix.len(), columns)
        };
        return Err(runtime(
            span,
            format!("Involution should be a {rank}x{rank} matrix; received a {rows}x{cols} matrix"),
        ));
    }
    if !is_identity_square(&matrix) {
        return Err(runtime(span, "Given transformation is not an involution"));
    }
    let involution =
        LatticeInvolution::new(inner_class.datum(), matrix.clone(), transpose(&matrix))
            .map_err(|error| runtime(span, error.to_string()))?;
    let element = inner_class
        .twisted_from_involution(involution)
        .map_err(|error| match error {
            StructureError::InvalidBasedAutomorphism => {
                runtime(span, "Involution not in this inner class")
            }
            // The root-datum automorphism rejections share the involution
            // decomposition slice's upstream wording.
            other => twisted_involution_diagnostic(&context.root_datum, other, span),
        })?;
    // Doubled theta-fixed projection on the numerator (upstream
    // `num += theta->val.right_prod(num)`), then the centrality parity
    // test of the DOUBLED factor: every simple root must evaluate to a
    // multiple of twice the denominator (upstream `is_central`).
    let denominator = BigInt::from(factor.denominator());
    let doubled: Vec<BigInt> = (0..rank)
        .map(|column| {
            let mut symmetrized = BigInt::from(factor.numerators()[column]);
            for (row, theta_row) in matrix.iter().enumerate() {
                symmetrized +=
                    BigInt::from(theta_row[column]) * BigInt::from(factor.numerators()[row]);
            }
            symmetrized
        })
        .collect();
    let twice_denominator = BigInt::from(2) * &denominator;
    for root in inner_class.datum().simple_roots() {
        let mut evaluation = BigInt::from(0);
        for (&coordinate, numerator) in root.as_slice().iter().zip(&doubled) {
            evaluation += BigInt::from(coordinate) * numerator;
        }
        if &evaluation % &twice_denominator != 0 {
            return Err(runtime(
                span,
                "Torus factor does not define a valid strong involution",
            ));
        }
    }
    let projected: Vec<BigRational> = doubled
        .iter()
        .map(|numerator| {
            BigRational::from(numerator.clone()) / BigRational::from(&twice_denominator)
        })
        .collect();
    let (internal, cartan) = context
        .classification
        .real_form_of_detailed(inner_class, &element, &projected)
        .map_err(|error| runtime(span, error.to_string()))?;
    // real_form_of's `coch` output: the stable log of the shifted square.
    let cocharacter = elected_square_root(inner_class, &element, &projected, &INTEGER_BUDGET)
        .map_err(|error| runtime(span, error.to_string()))?;
    // Make sure the involution table knows the Cartan class of the
    // involution and every class below it (atlas-types.w:3902-3907).
    let mut table = InnerClassContext::fresh_table(context)
        .map_err(|error| runtime(span, error.to_string()))?;
    let mut classes: Vec<CartanId> = context
        .classification
        .cartan_ids()
        .filter(|&id| id == cartan || context.classification.is_below(id, cartan) == Some(true))
        .collect();
    classes.sort_unstable();
    for id in classes {
        table
            .add_cartan(&context.classification, id)
            .map_err(|error| runtime(span, error.to_string()))?;
    }
    let torus_part = minimal_torus_part(
        inner_class,
        &context.classification,
        &table,
        internal,
        &cocharacter,
        &element,
        &projected,
    )
    .map_err(|error| runtime(span, error.to_string()))?;
    // real_form_value::build's default test (atlas-types.w:3538-3541):
    // drop to the shared construction when the pair equals the elected
    // seed — `some_coch` and `x0_torus_part`, which is exactly what
    // `RealFormSeed::build` computes.
    let default = RealFormSeed::build(
        inner_class,
        &context.classification,
        &context.strong,
        &table,
        internal,
        &INTEGER_BUDGET,
        FIBER_BUDGET,
    )
    .map_err(|error| runtime(span, error.to_string()))?;
    let default_seed = cocharacter == default.square_class_cocharacter().to_rationals()
        && torus_part == *default.element().torus_bits();
    let external = context
        .order
        .external(internal)
        .ok_or_else(|| runtime(span, "real form number out of range"))?;
    Ok(SyntheticRealForm {
        external,
        internal,
        cocharacter,
        torus_part,
        default_seed,
    })
}

/// A simple-coordinate root or coroot table.
type CoordinateTable = Vec<Vec<i32>>;

/// Positive roots and coroots of `cartan` in simple coordinates, in the
/// oracle's presentation order (rootdata.cpp `RootSystem::RootSystem`
/// generation): roots appear by height, each level ordered by
/// [`ByLastCoordinate`]; coroots complete from the first descent's downward
/// link. The `4 * rank` level bound is the upstream one (`E8` needs 30).
fn generate_positive(
    cartan: &[Vec<i32>],
    span: SourceSpan,
) -> Result<(CoordinateTable, CoordinateTable), Diagnostic> {
    let rank = cartan.len();
    if rank == 0 {
        return Ok((Vec::new(), Vec::new()));
    }
    let mut levels: Vec<BTreeSet<ByLastCoordinate>> = vec![BTreeSet::new(); 2];
    for i in 0..rank {
        let mut simple = vec![0; rank];
        simple[i] = 1;
        levels[1].insert(ByLastCoordinate(simple));
    }
    let mut roots: Vec<Vec<i32>> = Vec::new();
    let mut links: Vec<Vec<usize>> = Vec::new();
    // `level_start[l]` is the index of the first level-`l` root in `roots`.
    let mut level_start: Vec<usize> = vec![0];
    let mut level = 1;
    while level < levels.len() && !levels[level].is_empty() {
        level_start.push(roots.len());
        let current = std::mem::take(&mut levels[level]);
        for alpha in current {
            let alpha = alpha.0;
            let cur = roots.len();
            roots.push(alpha.clone());
            links.push(vec![usize::MAX; rank]);
            for i in 0..rank {
                let coefficient = dot_column(&alpha, cartan, i);
                if coefficient == 0 {
                    links[cur][i] = cur;
                } else if coefficient > 0 {
                    // A descent, except that a simple root reflects to minus
                    // itself, which is not on the list.
                    if level > 1 {
                        let mut beta = alpha.clone();
                        beta[i] -= coefficient;
                        let lower = level - coefficient as usize;
                        let candidates = level_start[lower]..level_start[lower + 1];
                        let Some(j) = candidates.into_iter().find(|&j| roots[j] == beta) else {
                            return Err(runtime(span, "internal root-generation descent miss"));
                        };
                        links[cur][i] = j;
                        links[j][i] = cur;
                    }
                } else {
                    let mut beta = alpha.clone();
                    beta[i] -= coefficient;
                    let upper = coefficient
                        .checked_neg()
                        .and_then(|rise| usize::try_from(rise).ok())
                        .and_then(|rise| level.checked_add(rise))
                        .ok_or_else(|| runtime(span, "internal root-generation level overflow"))?;
                    if upper > 4 * rank {
                        return Err(runtime(span, "internal root-generation level bound"));
                    }
                    while levels.len() <= upper {
                        levels.push(BTreeSet::new());
                    }
                    levels[upper].insert(ByLastCoordinate(beta));
                }
            }
        }
        level += 1;
    }
    let mut coroots: Vec<Vec<i32>> = Vec::with_capacity(roots.len());
    for i in 0..rank {
        let mut simple = vec![0; rank];
        simple[i] = 1;
        coroots.push(simple);
    }
    for alpha in rank..roots.len() {
        let descent = (0..rank)
            .find(|&i| dot_column(&roots[alpha], cartan, i) > 0)
            .ok_or_else(|| runtime(span, "internal root-generation descent set"))?;
        let beta = links[alpha][descent];
        let mut coroot = coroots[beta].clone();
        coroot[descent] -= dot_row(&coroots[beta], &cartan[descent]);
        coroots.push(coroot);
    }
    Ok((roots, coroots))
}

/// Irreducible Dynkin-diagram components: index sets linked by nonzero
/// Cartan entries.
fn components(cartan: &[Vec<i32>]) -> Vec<Vec<usize>> {
    let rank = cartan.len();
    let mut seen = vec![false; rank];
    let mut components = Vec::new();
    for start in 0..rank {
        if seen[start] {
            continue;
        }
        seen[start] = true;
        let mut component = vec![start];
        let mut cursor = 0;
        while cursor < component.len() {
            let i = component[cursor];
            cursor += 1;
            for (j, &linked) in cartan[i].iter().enumerate() {
                if linked != 0 && !seen[j] {
                    seen[j] = true;
                    component.push(j);
                }
            }
        }
        components.push(component);
    }
    components
}

/// Relative squared simple-root lengths, one free scale per component:
/// `lengths[j] = lengths[i] * C[j][i] / C[i][j]` along every edge, so
/// `v, w |-> sum v_i * C[i][j] * w_j * lengths[j]` is the Weyl-invariant
/// form with `lengths[j]` the half squared length of simple root `j`.
fn simple_lengths(cartan: &[Vec<i32>], components: &[Vec<usize>]) -> Vec<BigRational> {
    let rank = cartan.len();
    let mut lengths = vec![BigRational::from(0u32); rank];
    for component in components {
        let mut assigned = vec![false; rank];
        let mut frontier = Vec::new();
        for &start in component {
            if !assigned[start] {
                assigned[start] = true;
                lengths[start] = BigRational::from(1u32);
                frontier.push(start);
            }
            while let Some(i) = frontier.pop() {
                for &j in component {
                    if assigned[j] || cartan[i][j] == 0 {
                        continue;
                    }
                    assigned[j] = true;
                    lengths[j] = lengths[i].clone() * BigRational::from(i64::from(cartan[j][i]))
                        / BigRational::from(i64::from(cartan[i][j]));
                    frontier.push(j);
                }
            }
        }
    }
    lengths
}

/// The invariant form evaluated on simple coordinates: `B(v,v)` with
/// `B(α_i,α_j) = C[i][j] * lengths[j]`, symmetric by construction.
fn squared_length(cartan: &[Vec<i32>], lengths: &[BigRational], coords: &[i32]) -> BigRational {
    let mut total = BigRational::from(0u32);
    for (i, &left) in coords.iter().enumerate() {
        if left == 0 {
            continue;
        }
        for (j, &right) in coords.iter().enumerate() {
            if right != 0 && cartan[i][j] != 0 {
                total += BigRational::from(i64::from(left) * i64::from(right))
                    * BigRational::from(i64::from(cartan[i][j]))
                    * &lengths[j];
            }
        }
    }
    total
}

/// Per-vector length flags: long exactly when the squared length exceeds
/// the component minimum (so simply laced components are all-short, the
/// upstream convention from atlas-types.w:1521-1526).
fn length_flags(cartan: &[Vec<i32>], components: &[Vec<usize>], vectors: &[Vec<i32>]) -> Vec<bool> {
    let lengths = simple_lengths(cartan, components);
    vectors
        .iter()
        .map(|coords| {
            let Some(support) = coords.iter().position(|&entry| entry != 0) else {
                return false;
            };
            let component = components
                .iter()
                .find(|component| component.contains(&support))
                .expect("a root's support lies in one component");
            let minimum = component
                .iter()
                .map(|&j| BigRational::from(2u32) * &lengths[j])
                .min()
                .expect("components are nonempty");
            squared_length(cartan, &lengths, coords) > minimum
        })
        .collect()
}

/// Positive roots and coroots of one datum in the oracle's presentation
/// order, with per-vector length flags. A coroot-preferring datum generates
/// from the TRANSPOSED Cartan matrix and swaps the tables afterwards, the
/// upstream `prefer_co` path in rootdata.cpp.
struct RootTable {
    roots: Vec<Vec<i32>>,
    coroots: Vec<Vec<i32>>,
    long_roots: Vec<bool>,
    long_coroots: Vec<bool>,
}

impl RootTable {
    fn build(handle: &RootDatumHandle, span: SourceSpan) -> Result<Self, Diagnostic> {
        let datum = &*handle.datum;
        let cartan = datum.cartan_matrix();
        let transposed = transpose(cartan);
        let generation = if handle.prefers_coroots {
            &transposed
        } else {
            cartan
        };
        let (roots, coroots) = generate_positive(generation, span)?;
        let (roots, coroots) = if handle.prefers_coroots {
            (coroots, roots)
        } else {
            (roots, coroots)
        };
        let components = components(cartan);
        let root_basis: Vec<&[i32]> = datum.simple_roots().iter().map(Weight::as_slice).collect();
        let coroot_basis: Vec<&[i32]> = datum
            .simple_coroots()
            .iter()
            .map(Coweight::as_slice)
            .collect();
        Ok(Self {
            long_roots: length_flags(cartan, &components, &roots),
            long_coroots: length_flags(&transposed, &components, &coroots),
            roots: express(&roots, &root_basis),
            coroots: express(&coroots, &coroot_basis),
        })
    }
}

/// Simple-coordinate vectors expressed against an ambient lattice basis.
fn express(simple: &[Vec<i32>], basis: &[&[i32]]) -> Vec<Vec<i32>> {
    let lattice_rank = basis.first().map_or(0, |vector| vector.len());
    simple
        .iter()
        .map(|coords| {
            let mut ambient = vec![0; lattice_rank];
            for (&coefficient, vector) in coords.iter().zip(basis) {
                for (entry, &basis_entry) in ambient.iter_mut().zip(*vector) {
                    *entry += coefficient * basis_entry;
                }
            }
            ambient
        })
        .collect()
}

/// Language-level (co)root index to the positive table slot, with the
/// upstream signed convention (atlas-types.w `internal_root_index`): user
/// index `i` is valid for `-npos <= i < npos`, and a negative index negates
/// the positive root at `-1 - i` (internally the negative roots sit before
/// the positive ones, ordered so negation is `numRoots-1-alpha`).
fn positive_slot(
    index: &BigInt,
    positive_count: usize,
    coroot: bool,
    span: SourceSpan,
) -> Result<(usize, bool), Diagnostic> {
    let npos =
        i64::try_from(positive_count).map_err(|_| runtime(span, "internal root count overflow"))?;
    let internal = i64::try_from(index)
        .ok()
        .and_then(|index| index.checked_add(npos));
    let Some(internal) = internal.filter(|&internal| internal >= 0 && internal < 2 * npos) else {
        return Err(runtime(
            span,
            format!(
                "Illegal {}root index {index}",
                if coroot { "co" } else { "" }
            ),
        ));
    };
    let internal =
        usize::try_from(internal).map_err(|_| runtime(span, "internal root index overflow"))?;
    let npos = usize::try_from(npos).expect("positive count fits usize");
    if internal >= npos {
        Ok((internal - npos, false))
    } else {
        Ok((npos - 1 - internal, true))
    }
}

fn root_query(
    handle: &RootDatumHandle,
    index: &BigInt,
    coroot: bool,
    span: SourceSpan,
) -> Result<Value, Diagnostic> {
    let table = RootTable::build(handle, span)?;
    let (positive, negate) = positive_slot(index, table.roots.len(), coroot, span)?;
    let vector = if coroot {
        &table.coroots[positive]
    } else {
        &table.roots[positive]
    };
    let coordinates = if negate {
        vector.iter().map(|entry| -entry).collect()
    } else {
        vector.clone()
    };
    Ok(Value::Vector(Vec32(coordinates)))
}

fn length_query(
    handle: &RootDatumHandle,
    index: &BigInt,
    coroot: bool,
    span: SourceSpan,
) -> Result<Value, Diagnostic> {
    let table = RootTable::build(handle, span)?;
    let (positive, _) = positive_slot(index, table.roots.len(), coroot, span)?;
    let flag = if coroot {
        table.long_coroots[positive]
    } else {
        table.long_roots[positive]
    };
    Ok(Value::Boolean(flag))
}

/// The enumerated root system of one datum plus its upstream `RootNbr`
/// numbering: the shared setup of the signed root-numbering builtins
/// (atlas-types.w:1478-1485).
fn signed_roots(
    handle: &RootDatumHandle,
    span: SourceSpan,
) -> Result<(RootSystem, RootNumbering), Diagnostic> {
    let system = RootSystem::enumerate(&handle.datum, ROOT_BUDGET)
        .map_err(|error| runtime(span, error.to_string()))?;
    let numbering = RootNumbering::new(&system, handle.prefers_coroots());
    Ok((system, numbering))
}

/// The internal `RootNbr` of a user (co)root index, reusing the signed
/// convention check of [`positive_slot`].
fn internal_root_nbr(
    index: &BigInt,
    numbering: &RootNumbering,
    coroot: bool,
    span: SourceSpan,
) -> Result<usize, Diagnostic> {
    let (positive, negate) = positive_slot(index, numbering.npos, coroot, span)?;
    Ok(if negate {
        numbering.npos - 1 - positive
    } else {
        numbering.npos + positive
    })
}

/// Images of every root under a Weyl action, listed in internal `RootNbr`
/// order (upstream `permuted_root` over `0..numRoots`, rootdata.cpp).
fn weyl_root_permutation(
    system: &RootSystem,
    numbering: &RootNumbering,
    action: &WeylAction,
    span: SourceSpan,
) -> Result<Value, Diagnostic> {
    let images = system
        .action_permutation(action)
        .map_err(|error| runtime(span, error.to_string()))?;
    let mut entries = Vec::with_capacity(system.roots().len());
    for nbr in 0..system.roots().len() {
        let image = images[numbering.id(nbr).index()];
        entries.push(
            i32::try_from(numbering.nbr(image))
                .map_err(|_| runtime(span, "internal root index overflow"))?,
        );
    }
    Ok(Value::Vector(Vec32(entries)))
}

fn as_integer(value: &Value, span: SourceSpan) -> Result<BigInt, Diagnostic> {
    match value {
        Value::Integer(value) => Ok(value.clone()),
        other => Err(type_error(span, format!("expected an int, found {other}"))),
    }
}

fn as_root_datum(value: &Value, span: SourceSpan) -> Result<&RootDatumHandle, Diagnostic> {
    match value {
        Value::Domain(DomainValue::RootDatum(handle)) => Ok(handle),
        other => Err(type_error(
            span,
            format!("expected a RootDatum, found {other}"),
        )),
    }
}

fn as_weyl_elt(value: &Value, span: SourceSpan) -> Result<&WeylEltValue, Diagnostic> {
    match value {
        Value::Domain(DomainValue::WeylElement(element)) => Ok(element),
        other => Err(type_error(
            span,
            format!("expected a WeylElt, found {other}"),
        )),
    }
}

/// The datum's Weyl side, built on demand: every finite root system fits
/// the shared root budget (E8 needs 240 of the 4096 slots).
fn build_weyl_context(
    handle: &RootDatumHandle,
    span: SourceSpan,
) -> Result<Arc<WeylEltContext>, Diagnostic> {
    let system = RootSystem::enumerate(&handle.datum, ROOT_BUDGET)
        .map_err(|error| runtime(span, error.to_string()))?;
    let interface = WeylInterface::new(handle.datum.cartan_matrix())
        .map_err(|error| runtime(span, error.to_string()))?;
    Ok(Arc::new(WeylEltContext {
        handle: handle.clone(),
        system,
        interface,
    }))
}

/// Freeze an element into a language value, computing its canonical
/// reduced word once (upstream `WeylGroup::word`, weyl.cpp:944-957).
fn weyl_elt_value(
    context: Arc<WeylEltContext>,
    element: WeylElement,
    span: SourceSpan,
) -> Result<Value, Diagnostic> {
    let word = element
        .canonical_word(&context.system, &context.interface)
        .map_err(|error| runtime(span, error.to_string()))?;
    Ok(Value::Domain(DomainValue::WeylElement(WeylEltValue {
        context,
        element,
        word,
    })))
}

/// `check_Weyl_word` (atlas-types.w:2344-2359): entries convert to
/// unsigned first (`ulong_val` rejects negatives), then must lie below
/// the semisimple rank.
fn check_weyl_word(
    value: &Value,
    semisimple_rank: usize,
    span: SourceSpan,
) -> Result<Vec<usize>, Diagnostic> {
    let Value::List(entries) = value else {
        return Err(type_error(span, "expected a row of int"));
    };
    let mut word = Vec::with_capacity(entries.len());
    for entry in entries {
        let integer = as_integer(entry, span)?;
        if integer < 0 {
            return Err(runtime(span, "Negative integer where unsigned is required"));
        }
        let generator = u64::try_from(&integer)
            .map_err(|_| runtime(span, "Integer value to big for conversion"))?;
        if generator >= semisimple_rank as u64 {
            return Err(runtime(
                span,
                format!("Illegal Weyl word entry {integer} (should be <{semisimple_rank})"),
            ));
        }
        word.push(generator as usize);
    }
    Ok(word)
}

/// `int_value::int_val` followed by `check_Weyl_gen`
/// (atlas-types.w:2447-2476): narrowing to the upstream machine `int`
/// precedes the signed range check.
fn check_weyl_generator(
    generator: &BigInt,
    semisimple_rank: usize,
    span: SourceSpan,
) -> Result<usize, Diagnostic> {
    let generator = i32::try_from(generator)
        .map_err(|_| runtime(span, "Integer value to big for conversion"))?;
    usize::try_from(generator)
        .ok()
        .filter(|&index| index < semisimple_rank)
        .ok_or_else(|| {
            runtime(
                span,
                format!(
                    "Generator {generator} out of range for Weyl group (should be <{semisimple_rank})"
                ),
            )
        })
}

fn check_generator(
    context: &Arc<RealFormContext>,
    generator: usize,
    span: SourceSpan,
) -> Result<(), Diagnostic> {
    let rank = context.graph.semisimple_rank();
    if generator >= rank {
        // Posroot and negative indices are a documented phase-1 deferral;
        // the message echoes the user index like upstream
        // `get_reflection_index` (atlas-types.w:4481-4489).
        return Err(runtime(span, format!("Illegal root index: {generator}")));
    }
    Ok(())
}

/// Validate the cheap part of wrappers whose Atlas implementation suppresses
/// construction in `no_value` context. The typed evaluator has already
/// evaluated and type-checked the arguments; these checks preserve the
/// wrapper's observable out-of-range diagnostics without building a new
/// graph or applying a transform.
pub(crate) fn validate(
    name: &str,
    arguments: &[Value],
    span: SourceSpan,
) -> Result<(), Diagnostic> {
    match name {
        // Both real_form wrappers: real_form_wrapper (InnerClass,int) and
        // synthetic_real_form_wrapper (InnerClass,mat,ratvec), dispatched
        // by argument count like the other overloaded names.
        "real_form" => {
            let Some(Value::Domain(DomainValue::InnerClass(context))) = arguments.first() else {
                return Err(type_error(span, "expected an InnerClass"));
            };
            match arguments.len() {
                2 => {
                    let external = as_usize(&arguments[1], span)?;
                    context.order.internal(external).ok_or_else(|| {
                        runtime(span, format!("Illegal real form number: {external}"))
                    })?;
                }
                3 => {
                    synthetic_real_form(context, &arguments[1], &arguments[2], span)?;
                }
                count => {
                    return Err(type_error(
                        span,
                        format!("real_form expects 2 or 3 argument(s), found {count}"),
                    ));
                }
            }
        }
        "KGB" => {
            arity(name, arguments, 2, span)?;
            let context = as_real_form(&arguments[0], span)?;
            let index = as_integer(&arguments[1], span)?;
            let size = BigInt::from(context.graph.size());
            if index < 0 || index >= size {
                return Err(runtime(span, format!("Inexistent KGB element: {index}")));
            }
        }
        // build_KGB_element_wrapper runs every check before its no_value
        // gate (atlas-types.w:4580-4607), so validation runs the full
        // constructor pipeline and drops the element.
        "KGB_elt" => {
            arity(name, arguments, 3, span)?;
            let context = as_real_form(&arguments[0], span)?;
            build_kgb_element(context, &arguments[1], &arguments[2], span)?;
        }
        // KL_block_wrapper calls test_standard before its no-value gate
        // (atlas-types.w:6868-6872).
        "KL_block" => {
            arity(name, arguments, 1, span)?;
            let Value::Domain(DomainValue::Param(parameter)) = &arguments[0] else {
                return Err(type_error(span, "KL_block expects a Param"));
            };
            test_standard(parameter, "KL_block requires a standard parameter", span)?;
        }
        "KL_column" => {
            arity(name, arguments, 1, span)?;
            let Value::Domain(DomainValue::Param(parameter)) = &arguments[0] else {
                return Err(type_error(span, "KL_column expects a Param"));
            };
            validate_kl_column(parameter, span)?;
        }
        "from_dominant" => validate_from_dominant(arguments, span)?,
        // classify_involution_wrapper validates squareness and M^2=I before
        // its no-value gate, but the integer-lattice classification follows
        // it (atlas-types.w:2697-2710).
        "classify_involution" => {
            arity(name, arguments, 1, span)?;
            checked_involution_matrix(&arguments[0], None, span)?;
        }
        // twisted_involution_wrapper builds and normalizes the inner class,
        // retaining the Weyl factor, before checking no_value.  Only the
        // language Weyl value and result pair are suppressed.
        "twisted_involution" => {
            arity(name, arguments, 2, span)?;
            let handle = as_root_datum(&arguments[0], span)?;
            build_twisted_involution(handle, &arguments[1], span)?;
        }
        // basic_involution_wrapper's permutation SIZE check precedes its
        // no_value gate; the letter and permutation-entry checks follow it
        // (atlas-types.w:862-868). based_involution_wrapper has no early
        // gate: the basis shape, the letters, and the lattice compatibility
        // all fire in a no-value context (atlas-types.w:909-926).
        "involution" => {
            arity(name, arguments, 3, span)?;
            let lie_type = as_lie_type(&arguments[0], span)?;
            let symbols = as_inner_class_symbols(&arguments[2], span)?;
            match &arguments[1] {
                Value::List(entries) => {
                    let rank = lie_type.total_rank();
                    if entries.len() != rank {
                        return Err(runtime(
                            span,
                            format!(
                                "Permutation size {} does not match rank {rank} of Lie type",
                                entries.len()
                            ),
                        ));
                    }
                }
                basis => {
                    let basis = as_matrix_rows(basis, span)?;
                    based_primitive_involution(&lie_type, &basis, symbols, span)?;
                }
            }
        }
        // The KGB wrappers bounds-check the generator and element before
        // their no_value gates; the (int,Block,int) block wrappers run the
        // generator gate first, then the block-element gate
        // (atlas-types.w:4896-4906, 4924-4934, 4944-4954, 4970-4980).
        "cross" | "Cayley" | "status" | "inverse_Cayley" => {
            if arguments.len() == 3 {
                let block = as_block(&arguments[1], span)?;
                block_generator_check(block, &arguments[0], span)?;
                block_element_index(block, &arguments[2], span)?;
            } else {
                arity(name, arguments, 2, span)?;
                if let Value::Domain(DomainValue::Param(parameter)) = &arguments[1] {
                    parameter_generator(parameter, &arguments[0], span)?;
                    return Ok(());
                }
                let generator = as_usize(&arguments[0], span)?;
                let (context, id) = as_kgb_element(&arguments[1], span)?;
                check_generator(context, generator, span)?;
                if context.graph.element(id).is_none() {
                    return Err(runtime(span, "Inexistent KGB element"));
                }
            }
        }
        // Fokko_block_wrapper's is_dual gate precedes its no_value check
        // (atlas-types.w:4790-4794). The Param overload
        // (common_block_wrapper, atlas-types.w:6748-6752) gates
        // test_standard before its no_value check.
        "block" => {
            if arguments.len() == 1 {
                let Value::Domain(DomainValue::Param(parameter)) = &arguments[0] else {
                    return Err(type_error(span, "expected a Param"));
                };
                test_standard(parameter, "Cannot generate block", span)?;
                return Ok(());
            }
            arity(name, arguments, 2, span)?;
            let rf = as_real_form(&arguments[0], span)?;
            let df = as_real_form(&arguments[1], span)?;
            check_dual_pair(rf, df, span)?;
        }
        // block_element_wrapper bounds-checks before its no_value gate
        // (atlas-types.w:4830-4836).
        "element" => {
            arity(name, arguments, 2, span)?;
            let block = as_block(&arguments[0], span)?;
            block_element_index(block, &arguments[1], span)?;
        }
        // block_index_wrapper's three gates precede its no_value gate
        // (atlas-types.w:4861-4872).
        "index" => {
            arity(name, arguments, 3, span)?;
            let block = as_block(&arguments[0], span)?;
            let (x_context, x) = as_kgb_element(&arguments[1], span)?;
            let (y_context, y) = as_kgb_element(&arguments[2], span)?;
            if x_context.parent.inner_class != block.rf.parent.inner_class {
                return Err(runtime(span, "Real form not in inner class of block"));
            }
            if y_context.parent.inner_class != block.dual_rf.parent.inner_class {
                return Err(runtime(span, "Dual real form not in inner class of block"));
            }
            block_fiber_check(block, x, y, span)?;
        }
        // KGB_outer_twist_wrapper runs test_compatible BEFORE its no_value
        // check (atlas-types.w:4634), so the compatibility diagnostics fire
        // even in a value-suppressed context.
        "twist" => {
            arity(name, arguments, 2, span)?;
            let context = match &arguments[0] {
                Value::Domain(DomainValue::KgbElement(context, _)) => context,
                Value::Domain(DomainValue::Param(parameter)) => &parameter.context,
                other => {
                    return Err(type_error(
                        span,
                        format!("twist expects a KGBElt or Param, found {other}"),
                    ));
                }
            };
            compatible_outer_twist(context, &arguments[1], span)?;
        }
        // shift_flip_wrapper runs test_compatible and both gamma-fix
        // checks BEFORE its no_value gate (atlas-types.w:7348-7355), so
        // validation runs the same gates and drops the result.
        "shift_flip" => {
            arity(name, arguments, 3, span)?;
            let Value::Domain(DomainValue::Param(parameter)) = &arguments[0] else {
                return Err(type_error(span, "expected a Param"));
            };
            shift_flip_gates(parameter, &arguments[1], &arguments[2], span)?;
        }
        // The ext_finalise wrappers run every precondition before their
        // no_value gates (atlas-types.w:8449-8537), so validation runs the
        // same gates and drops the result.
        "scale_extended" => {
            arity(name, arguments, 3, span)?;
            let Value::Domain(DomainValue::Param(parameter)) = &arguments[0] else {
                return Err(type_error(span, "expected a Param"));
            };
            let Value::Rational(factor) = &arguments[2] else {
                return Err(type_error(span, "expected a rat"));
            };
            scale_extended_gates(parameter, &arguments[1], factor, span)?;
        }
        "K_type_pol_extended" => {
            arity(name, arguments, 2, span)?;
            let Value::Domain(DomainValue::Param(parameter)) = &arguments[0] else {
                return Err(type_error(span, "expected a Param"));
            };
            k_type_pol_extended_gates(parameter, &arguments[1], span)?;
        }
        "finalize_extended" => {
            arity(name, arguments, 2, span)?;
            let Value::Domain(DomainValue::Param(parameter)) = &arguments[0] else {
                return Err(type_error(span, "expected a Param"));
            };
            finalize_extended_gates(parameter, &arguments[1], span)?;
        }
        // The twisted-deformation wrappers run every precondition before
        // their no_value gates (atlas-types.w:8120-8134, 8229-8240,
        // 8370-8382, 8420-8431 — the timed twisted_full_deform overload
        // gates identically), so validation runs the same gates and drops
        // the result. block_deform is absent here: its no_value gate
        // comes FIRST (atlas-types.w:8182), so it is registered Skip.
        "twisted_deform" => {
            arity(name, arguments, 1, span)?;
            let Value::Domain(DomainValue::Param(parameter)) = &arguments[0] else {
                return Err(type_error(span, "expected a Param"));
            };
            twisted_deform_gates(parameter, span)?;
        }
        "full_deform" => {
            if arguments.len() != 2 {
                return Err(type_error(
                    span,
                    format!(
                        "full_deform expects 2 argument(s), found {}",
                        arguments.len()
                    ),
                ));
            }
            let Value::Domain(DomainValue::Param(_parameter)) = &arguments[0] else {
                return Err(type_error(span, "expected a Param"));
            };
            let _ = i32::try_from(&as_integer(&arguments[1], span)?)
                .map_err(|_| runtime(span, "Integer value to big for conversion"))?;
        }
        "twisted_full_deform" => {
            if arguments.is_empty() || arguments.len() > 2 {
                return Err(type_error(
                    span,
                    format!(
                        "twisted_full_deform expects 1 or 2 argument(s), found {}",
                        arguments.len()
                    ),
                ));
            }
            let Value::Domain(DomainValue::Param(parameter)) = &arguments[0] else {
                return Err(type_error(span, "expected a Param"));
            };
            if arguments.len() == 2 {
                let _ = i32::try_from(&as_integer(&arguments[1], span)?)
                    .map_err(|_| runtime(span, "Integer value to big for conversion"))?;
            }
            twisted_full_deform_gates(parameter, span)?;
        }
        "W_graph" | "W_cells" => {
            let Value::Domain(DomainValue::Param(parameter)) = &arguments[0] else {
                return Err(type_error(span, "expected a Param"));
            };
            test_standard(parameter, "Cannot generate block", span)?;
        }
        "block_Hasse" => {
            arity(name, arguments, 1, span)?;
            let Value::Domain(DomainValue::Param(parameter)) = &arguments[0] else {
                return Err(type_error(span, "expected a Param"));
            };
            test_standard(parameter, "Cannot generate block", span)?;
        }
        "partial_block" => {
            arity(name, arguments, 1, span)?;
            let Value::Domain(DomainValue::Param(parameter)) = &arguments[0] else {
                return Err(type_error(span, "expected a Param"));
            };
            test_standard(parameter, "Cannot generate block", span)?;
        }
        "KL_sum_at_s" => {
            arity(name, arguments, 1, span)?;
            let Value::Domain(DomainValue::Param(parameter)) = &arguments[0] else {
                return Err(type_error(span, "expected a Param"));
            };
            test_standard(parameter, "Cannot compute Kazhdan-Lusztig sum", span)?;
            test_final(parameter, "Cannot compute Kazhdan-Lusztig sum", span)?;
        }
        "KL_sum_at_s_to_height" => {
            arity(name, arguments, 2, span)?;
            let Value::Domain(DomainValue::Param(parameter)) = &arguments[0] else {
                return Err(type_error(span, "expected a Param"));
            };
            test_standard(parameter, "Cannot compute Kazhdan-Lusztig sum", span)?;
            test_final(parameter, "Cannot compute Kazhdan-Lusztig sum", span)?;
        }
        "twisted_KL_sum_at_s" => {
            let Value::Domain(DomainValue::Param(parameter)) = &arguments[0] else {
                return Err(type_error(span, "expected a Param"));
            };
            match arguments.len() {
                1 => {
                    twisted_kl_sum_gates(parameter, span)?;
                }
                2 => {
                    external_twisted_kl_sum_gates(parameter, &arguments[1], span)?;
                }
                count => {
                    return Err(type_error(
                        span,
                        format!("twisted_KL_sum_at_s expects 1 or 2 argument(s), found {count}"),
                    ));
                }
            }
        }
        // check_Weyl_gen/check_Weyl_word precede the no-value gates of the
        // four generator/word multiplication wrappers.
        "#" => {
            arity(name, arguments, 2, span)?;
            let (element, generator) = match arguments {
                [Value::Domain(DomainValue::WeylElement(element)), Value::Integer(generator)]
                | [Value::Integer(generator), Value::Domain(DomainValue::WeylElement(element))] => {
                    (element, generator)
                }
                _ => return Err(type_error(span, "expected WeylElt and int")),
            };
            let rank = element.context.handle.datum.semisimple_rank();
            check_weyl_generator(generator, rank, span)?;
        }
        "##" => {
            arity(name, arguments, 2, span)?;
            let (element, word) = match arguments {
                [Value::Domain(DomainValue::WeylElement(element)), word]
                | [word, Value::Domain(DomainValue::WeylElement(element))] => (element, word),
                _ => return Err(type_error(span, "expected WeylElt and [int]")),
            };
            check_weyl_word(word, element.context.handle.datum.semisimple_rank(), span)?;
        }
        // compose_Lie_types_wrapper and both Weyl (co)weight actions run
        // their rank/size checks before their no-value gates.
        "*" => match arguments {
            [Value::Domain(DomainValue::LieType(left)), Value::Domain(DomainValue::LieType(right))] =>
            {
                validate_combined_rank(left, right, span)?;
            }
            [Value::Domain(DomainValue::WeylElement(weyl)), Value::Vector(vector)] => {
                validate_weight_rank(weyl, vector, span)?;
            }
            [Value::Vector(vector), Value::Domain(DomainValue::WeylElement(weyl))] => {
                validate_coweight_rank(vector, weyl, span)?;
            }
            _ => return Err(type_error(span, "expected hungry * operands")),
        },
        // Both Cartan_class wrappers bounds-check before their no_value gate
        // (atlas-types.w:4025-4033, 4046-4056).
        "Cartan_class" => {
            arity(name, arguments, 2, span)?;
            let index = as_integer(&arguments[1], span)?;
            match &arguments[0] {
                Value::Domain(DomainValue::InnerClass(context)) => {
                    check_cartan_number(
                        &index,
                        context.classification.cartan_ids().len(),
                        "inner class",
                        span,
                    )?;
                }
                Value::Domain(DomainValue::RealForm(form)) => {
                    let count = form
                        .parent
                        .classification
                        .cartan_set(form.internal)
                        .expect("a real form's internal number is in range")
                        .len();
                    check_cartan_number(&index, count, "real form", span)?;
                }
                other => {
                    return Err(type_error(
                        span,
                        format!("expected an InnerClass or RealForm, found {other}"),
                    ));
                }
            }
        }
        // fiber_partition_wrapper's mismatch and occurrence checks precede
        // its no_value gate (atlas-types.w:4202-4213).
        "fiber_partition" => {
            arity(name, arguments, 2, span)?;
            let (context, id) = as_cartan_class(&arguments[0], span)?;
            let form = as_real_form(&arguments[1], span)?;
            fiber_partition_membership(context, id, form, span)?;
        }
        // block_size_wrapper's unsigned extraction and bounds checks
        // precede its no_value gate (atlas-types.w:3337-3357).
        "block_size" => {
            arity(name, arguments, 3, span)?;
            let Value::Domain(DomainValue::InnerClass(context)) = &arguments[0] else {
                return Err(type_error(span, "expected an InnerClass"));
            };
            block_size_numbers(context, arguments, span)?;
        }
        // filter_units_wrapper checks the factor count against the matrix
        // column count before its no-value gate (atlas-types.w:479-503), but
        // conversion to the exact relation lattice and filtering follow it.
        "filter_units" => {
            let (basis, factors) = relation_pair(name, arguments, span)?;
            if factors.len() > basis.cols() {
                return Err(relation_diagnostic(
                    RelationError::TooManyFactors {
                        factors: factors.len(),
                        columns: basis.cols(),
                    },
                    span,
                ));
            }
        }
        // K_type_wrapper's rank check precedes its no-value gate
        // (atlas-types.w:5243-5249).
        "K_type" => match arguments {
            [kgb, lam] => {
                let (context, _) = as_kgb_element(kgb, span)?;
                let lam_rho = as_weight_vec(lam, span)?;
                let rank = context.parent.inner_class.datum().lattice_rank();
                if lam_rho.len() != rank {
                    return Err(runtime(
                        span,
                        format!("Rank mismatch: ({rank},{})", lam_rho.len()),
                    ));
                }
            }
            [_] => {}
            _ => {
                return Err(type_error(
                    span,
                    format!(
                        "K_type expects 1 or 2 argument(s), found {}",
                        arguments.len()
                    ),
                ));
            }
        },
        // module_parameter_wrapper's rank check precedes its no-value gate
        // (atlas-types.w:6224-6230).
        "param" => match arguments {
            [kgb, lam, nu] => {
                let (context, _) = as_kgb_element(kgb, span)?;
                let lam_rho = as_weight_vec(lam, span)?;
                let nu_weight = as_rational_weight(nu, span)?;
                let rank = context.parent.inner_class.datum().lattice_rank();
                if nu_weight.rank() != lam_rho.len() || nu_weight.rank() != rank {
                    return Err(runtime(
                        span,
                        format!(
                            "Rank mismatch: ({rank},{},{})",
                            lam_rho.len(),
                            nu_weight.rank()
                        ),
                    ));
                }
            }
            [_] => {}
            _ => {
                return Err(type_error(
                    span,
                    format!(
                        "param expects 1 or 3 argument(s), found {}",
                        arguments.len()
                    ),
                ));
            }
        },
        // K_type_equivalent_wrapper's real-form identity check precedes
        // its no-value gate (atlas-types.w:5325-5327).
        "equivalent" => {
            arity(name, arguments, 2, span)?;
            match (&arguments[0], &arguments[1]) {
                (
                    Value::Domain(DomainValue::KType(left)),
                    Value::Domain(DomainValue::KType(right)),
                ) => {
                    require_same_form_value(
                        &left.context,
                        &right.context,
                        "Real form mismatch when testing equivalence",
                        span,
                    )?;
                }
                (
                    Value::Domain(DomainValue::Param(left)),
                    Value::Domain(DomainValue::Param(right)),
                ) => {
                    require_same_form_value(
                        &left.context,
                        &right.context,
                        "Real form mismatch when testing equivalence",
                        span,
                    )?;
                }
                _ => {
                    return Err(type_error(
                        span,
                        "equivalent expects two KTypes or two Params",
                    ));
                }
            }
        }
        // KGP_sum_wrapper's semifinal precondition precedes its no-value
        // gate (atlas-types.w:5997-6001).
        "KGP_sum" => {
            arity(name, arguments, 1, span)?;
            let ktype = as_ktype(&arguments[0], span)?;
            let rc = rep_context(&ktype.context);
            if !ktype
                .ktype
                .is_semifinal(&rc)
                .map_err(|error| structure_diagnostic(error, span))?
            {
                return Err(runtime(
                    span,
                    "K-type has parity real roots (so not semifinal)",
                ));
            }
        }
        // K_type_formula_wrapper's semifinal precondition precedes its
        // no-value gate (atlas-types.w:6035-6039).
        "K_type_formula" => {
            arity(name, arguments, 2, span)?;
            let ktype = as_ktype(&arguments[0], span)?;
            let rc = rep_context(&ktype.context);
            if !ktype
                .ktype
                .is_semifinal(&rc)
                .map_err(|error| structure_diagnostic(error, span))?
            {
                return Err(runtime(
                    span,
                    "K-type has parity real roots (so not semifinal)",
                ));
            }
        }
        // branch_wrapper's negative-bound check precedes its no-value
        // gate (atlas-types.w:6058-6060).
        "branch" => {
            arity(name, arguments, 2, span)?;
            as_ktypepol(&arguments[0], span)?;
            let bound = i64::try_from(&as_integer(&arguments[1], span)?)
                .map_err(|_| runtime(span, "Integer value to big for conversion"))?;
            if bound < 0 {
                return Err(runtime(span, "Maximum level in branch cannot be negative"));
            }
        }
        // add/subtract_K_type_wrapper and add/subtract_module_wrapper's
        // real-form identity checks precede their no-value gates
        // (atlas-types.w:5670-5673, 5684-5687, 7788-7791, 7800-7803).
        "+" | "-" => match arguments {
            [Value::Domain(DomainValue::KTypePol(accumulator)), Value::Domain(DomainValue::KType(ktype))] =>
            {
                require_same_form_owner(
                    &accumulator.rf,
                    &ktype.context,
                    if name == "+" {
                        "Real form mismatch when adding a KType to a KTypePol"
                    } else {
                        "Real form mismatch when subtracting a KType from a KTypePol"
                    },
                    span,
                )?;
            }
            [Value::Domain(DomainValue::KTypePol(accumulator)), Value::Domain(DomainValue::KTypePol(other))] =>
            {
                require_same_form_owner(
                    &accumulator.rf,
                    &other.rf,
                    if name == "+" {
                        "Real form mismatch when adding two K_types"
                    } else {
                        "Real form mismatch when subtracting two K_types"
                    },
                    span,
                )?;
            }
            [Value::Domain(DomainValue::KTypePol(accumulator)), Value::Tuple(term)]
                if matches!(
                    term.as_slice(),
                    [
                        Value::Domain(DomainValue::Split(_)),
                        Value::Domain(DomainValue::KType(_))
                    ]
                ) =>
            {
                let Value::Domain(DomainValue::KType(ktype)) = &term[1] else {
                    unreachable!()
                };
                require_same_form_owner(
                    &accumulator.rf,
                    &ktype.context,
                    "Real form mismatch when adding a term to a K_type",
                    span,
                )?;
            }
            [Value::Domain(DomainValue::ParamPol(accumulator)), Value::Domain(DomainValue::Param(parameter))] =>
            {
                require_same_form_owner(
                    &accumulator.rf,
                    &parameter.context,
                    if name == "+" {
                        "Real form mismatch when adding a Param to a ParamPol"
                    } else {
                        "Real form mismatch when subtracting a Param from a ParamPol"
                    },
                    span,
                )?;
            }
            [Value::Domain(DomainValue::ParamPol(accumulator)), Value::Domain(DomainValue::ParamPol(other))] =>
            {
                require_same_form_owner(
                    &accumulator.rf,
                    &other.rf,
                    if name == "+" {
                        "Real form mismatch when adding two modules"
                    } else {
                        "Real form mismatch when subtracting two modules"
                    },
                    span,
                )?;
            }
            [Value::Domain(DomainValue::ParamPol(accumulator)), Value::Tuple(term)]
                if matches!(
                    term.as_slice(),
                    [
                        Value::Domain(DomainValue::Split(_)),
                        Value::Domain(DomainValue::Param(_))
                    ]
                ) =>
            {
                let Value::Domain(DomainValue::Param(parameter)) = &term[1] else {
                    unreachable!()
                };
                require_same_form_owner(
                    &accumulator.rf,
                    &parameter.context,
                    "Real form mismatch when adding a term to a module",
                    span,
                )?;
            }
            _ => {}
        },
        // ann_mod_wrapper extracts the Atlas `int` before its no-value gate,
        // but matrix reduction and the nonzero precondition are behind it.
        "ann_mod" => {
            arity(name, arguments, 2, span)?;
            let Value::Matrix(_) = &arguments[0] else {
                return Err(type_error(span, "ann_mod expects a mat"));
            };
            narrow_ann_modulus(&arguments[1], span)?;
        }
        other => {
            return Err(runtime(
                span,
                format!("no validation policy registered for '{other}'"),
            ));
        }
    }
    Ok(())
}

/// One row of a common-block print: the `do_print` fields (block_io.cpp:
/// 54-110, traditional=false so no local `(x,y):` prefix) plus the data for
/// the `common_block::print` suffix (block_io.cpp:128-147). Cross/Cayley
/// entries are indices into the row list itself.
struct CommonBlockRow {
    x: KgbId,
    length: usize,
    descents: Vec<BlockDescent>,
    crosses: Vec<Option<usize>>,
    cayleys: Vec<(Option<usize>, Option<usize>)>,
    gamma_lambda: RationalWeight,
    survives: bool,
}

/// printInvolution of the KGB involution at `x` (prettyprint.cpp:219-232):
/// one-based generator digits, '^' for crosses, 'x' for conjugations, `e`
/// closing.
fn involution_expression(context: &RealFormContext, x: KgbId) -> String {
    let record = context
        .table
        .record(context.graph.involution_of(x).expect("in-range"))
        .expect("in-range");
    let word = context
        .parent
        .inner_class
        .canonical_involution_expr(record.weyl_element())
        .expect("a KGB involution is a twisted involution of the class");
    let mut text = String::new();
    for entry in word {
        if entry >= 0 {
            text.push(char::from(
                b'1' + u8::try_from(entry).expect("generator rank"),
            ));
            text.push('^');
        } else {
            text.push(char::from(
                b'1' + u8::try_from(!entry).expect("generator rank"),
            ));
            text.push('x');
        }
    }
    text.push('e');
    text
}

/// The shared engine of print_param_block_wrapper and print_c_block_wrapper
/// (atlas-types.w:6653-6695): the rows of the common block of `sr`'s srm,
/// fresh-built per call — upstream's Rep_table pool (repr.cpp:1773-1794) is
/// memoization only, and for a dominant gamma the block modifier it
/// maintains is trivial (identity transform word, zero shift), so a fresh
/// build prints identically (the partial_block/KL_block precedent).
/// Returns the rows in block order and the init index: the row whose
/// `(x, gamma-lambda)` matches the srm — matching on `x` alone is ambiguous
/// within an R-packet.
fn common_block_rows(
    context: &Arc<RealFormContext>,
    sr: &StandardRepr,
    span: SourceSpan,
) -> Result<(Vec<CommonBlockRow>, usize), Diagnostic> {
    let rc = rep_context(context);
    let datum = context.parent.root_datum.datum.clone();
    let gamma = sr.gamma().clone();
    let (seed_x, seed_gamma_lambda) = rc
        .mod_reduce(sr)
        .map_err(|error| structure_diagnostic(error, span))?;
    if !gamma_is_integral(&datum, &gamma) {
        // Upstream builds the common block on the integral subsystem of
        // gamma (common_context/SubSystem, repr.cpp:2666-2670); that slice
        // is not ported. Only the rank-0 case is reproduced: the block is
        // the seed element alone, with no generators.
        let system = rc.inner_class().root_system();
        let numerator = gamma.numerator();
        let denominator = gamma.denominator();
        let has_integral_root = (0..system.roots().len())
            .map(RootId::from_usize)
            .filter(|&id| system.is_positive(id) == Some(true))
            .any(|id| {
                let coroot = system.coroot(id).expect("in-range root");
                let pairing: i64 = coroot
                    .as_slice()
                    .iter()
                    .zip(numerator.iter())
                    .map(|(&c, &g)| i64::from(c) * g)
                    .sum();
                pairing % denominator == 0
            });
        if has_integral_root {
            return Err(runtime(
                span,
                "common block at a non-integral infinitesimal character is not yet implemented",
            ));
        }
        let row = CommonBlockRow {
            x: seed_x,
            length: 0,
            descents: Vec::new(),
            crosses: Vec::new(),
            cayleys: Vec::new(),
            gamma_lambda: seed_gamma_lambda,
            survives: true,
        };
        return Ok((vec![row], 0));
    }
    let dual_parent = build_dual_inner_class(&context.parent, span)?;
    let dual_quasisplit = dual_parent.order.quasisplit_external();
    let dual_rf = build_real_form(&dual_parent, dual_quasisplit, span)?;
    let block = build_block(context, &dual_rf, span)?;
    let size = block.graph.size();
    let rank = block.graph.rank();
    // For an integral gamma the common block is the full block
    // (atlas-types.w:7093-7107); its per-element gamma-lambda values are
    // the z_pool entries (blocks.cpp:733-1076), reproduced by the srm
    // propagation — computing them from the full block's (x,y) pairs
    // directly (common_block_gamma_lambdas) diverges at singular elements,
    // where the common block's element is a different y-class of the same
    // x (the KL_block arm has the same split).
    let lambda_rho = rc
        .lambda_rho(sr)
        .map_err(|error| structure_diagnostic(error, span))?;
    let z0 = (0..size)
        .find(|&z| block.graph.x(z) == Some(seed_x))
        .ok_or_else(|| runtime(span, "parameter not in the common block"))?;
    let srms = common_block_srms(&block, z0, &rc, &lambda_rho, &gamma, span)?;
    // init_index matches the input srm on (x, gamma-lambda), not on x
    // alone (R-packets); the anchor z0 always carries the seed value, so
    // the fallback is unreachable but keeps the tie-breaking explicit.
    let init = (0..size)
        .find(|&z| block.graph.x(z) == Some(seed_x) && srms[z].as_ref() == Some(&seed_gamma_lambda))
        .unwrap_or(z0);
    // common_block::singular (blocks.cpp:701-720) with a trivial block
    // modifier: the simply-integral roots are the datum's simple roots, so
    // generator s is singular iff <gamma, alpha_s^vee> vanishes.
    let singular: Vec<bool> = (0..rank)
        .map(|s| {
            let pairing: i64 = datum.simple_coroots()[s]
                .as_slice()
                .iter()
                .zip(gamma.numerator().iter())
                .map(|(&c, &g)| i64::from(c) * g)
                .sum();
            pairing == 0
        })
        .collect();
    let mut rows = Vec::with_capacity(size);
    for (z, srm) in srms.iter().enumerate() {
        let mut descents = Vec::with_capacity(rank);
        let mut crosses = Vec::with_capacity(rank);
        let mut cayleys = Vec::with_capacity(rank);
        let mut survives = true;
        for (s, &is_singular) in singular.iter().enumerate() {
            let descent = block.graph.descent_value(z, s).expect("in-range");
            if is_singular && descent.is_descent() {
                survives = false;
            }
            descents.push(descent);
            crosses.push(block.graph.cross(z, s));
            // do_print (block_io.cpp:96-102): the inverse Cayley at weak
            // descents, the Cayley transform otherwise.
            cayleys.push(if descent.is_descent() {
                block.graph.inverse_cayley(z, s).expect("in-range")
            } else {
                block.graph.cayley(z, s).expect("in-range")
            });
        }
        rows.push(CommonBlockRow {
            x: block.graph.x(z).expect("in-range"),
            length: block.graph.length(z).expect("in-range"),
            descents,
            crosses,
            cayleys,
            gamma_lambda: srm
                .clone()
                .expect("every full-block element carries an srm value"),
            survives,
        });
    }
    Ok((rows, init))
}

/// Render rows from a block installed in the real form's representation
/// table.  Unlike [`common_block_rows`], this path deliberately participates
/// in the shared lookup sequence used by `KL_column` and `KL_block`.
fn located_common_block_rows(
    context: &Arc<RealFormContext>,
    located: &LocatedBlock,
    span: SourceSpan,
) -> Result<Vec<CommonBlockRow>, Diagnostic> {
    let block = located.block();
    let rc = rep_context(context);
    let common = CommonContext::integral(&rc, located.adapted_representative().gamma_lambda())
        .map_err(|error| structure_diagnostic(error, span))?;
    let singular = common
        .singular_flags(located.prepared_query().gamma())
        .map_err(|error| structure_diagnostic(error, span))?;
    let mut rows = Vec::with_capacity(block.size());
    for z in 0..block.size() {
        let mut descents = Vec::with_capacity(block.rank());
        let mut crosses = Vec::with_capacity(block.rank());
        let mut cayleys = Vec::with_capacity(block.rank());
        for s in 0..block.rank() {
            descents.push(
                block
                    .descent(z, s)
                    .ok_or_else(|| runtime(span, "common block descent out of range"))?,
            );
            crosses.push(block.as_ref().cross(s, z));
            cayleys.push(
                block
                    .as_ref()
                    .cayley(s, z)
                    .ok_or_else(|| runtime(span, "common block Cayley link out of range"))?,
            );
        }
        let stored = block
            .element(z)
            .ok_or_else(|| runtime(span, "common block element out of range"))?;
        rows.push(CommonBlockRow {
            x: stored.x(),
            length: block
                .length(z)
                .ok_or_else(|| runtime(span, "common block length out of range"))?,
            descents,
            crosses,
            cayleys,
            gamma_lambda: stored
                .gamma_lambda()
                .add(located.relative_shift())
                .map_err(|error| structure_diagnostic(error, span))?,
            survives: block.survives(z, &singular),
        });
    }
    Ok(rows)
}

/// The seed-to-rows path of the partial-block printers
/// (atlas-types.w:6700-6735): `StandardReprMod::mod_reduce` of `seed_repr`
/// (repr.cpp:52-58), the `common_context` on its gamma_lambda
/// (repr.cpp:2666-2670), the Bruhat interval below the seed
/// (`Rep_table::Bruhat_below`, repr.cpp:1565-1573), and the partial
/// `common_block` over that interval (blocks.cpp:1086-1248) — one row per
/// element, renumbered by the block's final `(length, x, y)` sort. The
/// survives flag uses `block.singular(gamma)` (blocks.cpp:701-708) with the
/// CALLER's gamma: both wrappers pass `p->val.gamma()`, even when the seed
/// was normalised first (print_pc_block_wrapper). Also returns the seed's
/// row number (upstream's `init_index`), used by the partial-common header.
fn partial_block_rows(
    context: &Arc<RealFormContext>,
    seed_repr: &StandardRepr,
    gamma: &RationalWeight,
    span: SourceSpan,
) -> Result<(Vec<CommonBlockRow>, usize), Diagnostic> {
    let rc = rep_context(context);
    let seed = StandardReprMod::mod_reduce(&rc, seed_repr)
        .map_err(|error| structure_diagnostic(error, span))?;
    let ctxt = CommonContext::integral(&rc, seed.gamma_lambda())
        .map_err(|error| structure_diagnostic(error, span))?;
    let interval = bruhat_below(&ctxt, &seed).map_err(|error| structure_diagnostic(error, span))?;
    let block =
        PartialBlock::build(&ctxt, &interval).map_err(|error| structure_diagnostic(error, span))?;
    let singular = ctxt
        .singular_flags(gamma)
        .map_err(|error| structure_diagnostic(error, span))?;
    let rank = block.rank();
    let mut rows = Vec::with_capacity(block.size());
    for z in 0..block.size() {
        let mut descents = Vec::with_capacity(rank);
        let mut crosses = Vec::with_capacity(rank);
        let mut cayleys = Vec::with_capacity(rank);
        for s in 0..rank {
            descents.push(
                block
                    .descent(z, s)
                    .ok_or_else(|| runtime(span, "partial block descent out of range"))?,
            );
            crosses.push(block.cross(s, z));
            // do_print (block_io.cpp:96-102) switches between inverseCayley
            // and cayley by the weak descent, but both accessors return the
            // stored pair of their side and undef otherwise (blocks.h:
            // 143-157), so the stored pair is always what prints.
            cayleys.push(
                block
                    .cayley(s, z)
                    .ok_or_else(|| runtime(span, "partial block Cayley link out of range"))?,
            );
        }
        rows.push(CommonBlockRow {
            x: block
                .x(z)
                .ok_or_else(|| runtime(span, "partial block element out of range"))?,
            length: block
                .length(z)
                .ok_or_else(|| runtime(span, "partial block element out of range"))?,
            descents,
            crosses,
            cayleys,
            gamma_lambda: block
                .gamma_lambda(z)
                .cloned()
                .ok_or_else(|| runtime(span, "partial block element out of range"))?,
            survives: block.survives(z, &singular),
        });
    }
    // The seed generates last and has the maximal length in the interval,
    // so it sorts to the final row; upstream asserts this via
    // `which == last(subset)` (repr.cpp:1819). Report instead of assuming.
    let init = block
        .lookup(&seed)
        .ok_or_else(|| runtime(span, "seed missing from its Bruhat interval"))?;
    Ok((rows, init))
}

/// The common-block print (block_io.cpp:54-110 `do_print` with
/// traditional=false, then `common_block::print` at :128-147):
/// `z: length [descents] cross... (cayley,...) *(x=..,gamma-lambda=..)
/// involution-word`. The gamma-lambda field width is 3*rk+4 with rk the
/// FULL datum's semisimple rank (common_block::print uses
/// root_datum().semisimple_rank(), not the block rank), which shows on
/// rank-0 integral subsystems.
fn render_common_block(context: &RealFormContext, rows: &[CommonBlockRow]) -> String {
    let datum_rank = context.parent.root_datum.datum.semisimple_rank();
    let size = rows.len();
    let width = digits(size - 1);
    let lwidth = digits(rows[size - 1].length);
    let xwidth = digits(rows.iter().map(|row| row.x.index()).max().unwrap_or(0));
    let gwidth = 3 * datum_rank + 4;
    let pad = 2;
    let mut text = String::new();
    for (z, row) in rows.iter().enumerate() {
        text.push_str(&format!("{:width$}:", z));
        text.push_str(&format!("{:width$}", row.length, width = lwidth + pad));
        text.push_str(&" ".repeat(pad));
        text.push('[');
        for (s, descent) in row.descents.iter().enumerate() {
            if s > 0 {
                text.push(',');
            }
            text.push_str(block_descent_code(*descent));
        }
        text.push(']');
        for cross in &row.crosses {
            match cross {
                Some(target) => text.push_str(&format!("{:width$}", target, width = width + pad)),
                // Rust left-aligns char under a width; upstream's setw
                // right-aligns the undef marker like any other field.
                None => {
                    text.push_str(&" ".repeat(width + pad - 1));
                    text.push('*');
                }
            }
        }
        text.push_str(&" ".repeat(pad + 1));
        for pair in &row.cayleys {
            text.push('(');
            match pair.0 {
                Some(first) => text.push_str(&format!("{:width$}", first)),
                None => {
                    text.push_str(&" ".repeat(width - 1));
                    text.push('*');
                }
            }
            text.push(',');
            match pair.1 {
                Some(second) => text.push_str(&format!("{:width$}", second)),
                None => {
                    text.push_str(&" ".repeat(width - 1));
                    text.push('*');
                }
            }
            text.push(')');
            text.push_str(&" ".repeat(pad));
        }
        text.push(if row.survives { '*' } else { ' ' });
        text.push_str(&format!("(x={:xwidth$}", row.x.index()));
        text.push_str(",gamma-lambda=");
        // The gamma-lambda values are gcd-normalized by construction
        // (RationalWeight::new), so the explicit `normalize()` upstream
        // applies at print time (block_io.cpp:137) is a no-op here.
        text.push_str(&format!(
            "{:>gwidth$}",
            rational_weight_display(&row.gamma_lambda)
        ));
        text.push(')');
        text.push_str(&" ".repeat(2));
        text.push_str(&involution_expression(context, row.x));
        text.push('\n');
    }
    text
}

/// Printer wrappers (atlas-types.w:8944-8957, 8850-8859): the report text
/// of one `print_*` builtin. Upstream prints unconditionally, so the text
/// is produced at both evaluation levels; no diagnostics precede the
/// no-value gate.
pub(crate) fn print_text(
    name: &str,
    arguments: &[Value],
    span: SourceSpan,
) -> Result<String, Diagnostic> {
    match name {
        "print_KGB" => {
            let context = as_real_form(&arguments[0], span)?;
            if arguments.len() == 1 {
                return Ok(print_kgb(context, None));
            }
            // print_KGB_selection_wrapper (atlas-types.w:8958-8973): the
            // listed elements must belong to the SAME real form.
            arity(name, arguments, 2, span)?;
            let Value::List(entries) = &arguments[1] else {
                return Err(type_error(span, "expected a row of KGBElt"));
            };
            let mut which = Vec::with_capacity(entries.len());
            for entry in entries {
                let (element_context, id) = as_kgb_element(entry, span)?;
                if element_context.parent.inner_class != context.parent.inner_class
                    || element_context.internal != context.internal
                {
                    return Err(runtime(
                        span,
                        "Real form mismatch when printing KGB element",
                    ));
                }
                which.push(id);
            }
            Ok(print_kgb(context, Some(&which)))
        }
        "print_strong_real" => {
            arity(name, arguments, 1, span)?;
            let (context, id) = as_cartan_class(&arguments[0], span)?;
            print_strong_real(context, id, span)
        }
        // print_gradings_wrapper (atlas-types.w:4260-4300): the imaginary
        // root subsystem line, then the gradings of the real form's part.
        "print_gradings" => {
            arity(name, arguments, 2, span)?;
            let (context, id) = as_cartan_class(&arguments[0], span)?;
            let form = as_real_form(&arguments[1], span)?;
            print_gradings(context, id, form, span)
        }
        // print_X_wrapper (atlas-types.w:8999-9008): no checks upstream;
        // `kgb::global_KGB kgb(G)` builds a fresh InvolutionTable per call.
        "print_X" => {
            arity(name, arguments, 1, span)?;
            let Value::Domain(DomainValue::InnerClass(context)) = &arguments[0] else {
                return Err(type_error(span, "expected an InnerClass"));
            };
            print_x(context, span)
        }
        // print_real_Weyl_wrapper (atlas-types.w:8831-8847): the checks run
        // in the arm, in the wrapper's wording and order, before the crate
        // print — a foreign external form number would otherwise translate
        // silently through ExternalFormOrder.
        "print_real_Weyl" => {
            arity(name, arguments, 2, span)?;
            let form = as_real_form(&arguments[0], span)?;
            let (context, id) = as_cartan_class(&arguments[1], span)?;
            print_real_weyl(context, id, form, span)
        }
        // print_blockstabilizer_wrapper (atlas-types.w:8920-8932): no checks
        // upstream; the block only donates its two real forms.
        "print_blockstabilizer" => {
            arity(name, arguments, 2, span)?;
            let block = as_block(&arguments[0], span)?;
            let (context, id) = as_cartan_class(&arguments[1], span)?;
            print_blockstabilizer(block, context, id, span)
        }
        // print_KGB_order / print_KGB_graph (atlas-types.w:9119-9131): the
        // Bruhat Hasse rows and the Graphviz digraph of the KGB.
        "print_KGB_order" | "print_KGB_graph" => {
            arity(name, arguments, 1, span)?;
            let context = as_real_form(&arguments[0], span)?;
            let graph = &context.graph;
            let hasse = graph.bruhat_hasse();
            if name == "print_KGB_order" {
                let mut text = format!("kgbsize: {}\n", graph.size());
                text.push_str("0:\n");
                for (y, row) in hasse.iter().enumerate().skip(1) {
                    let entries = row
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(",");
                    text.push_str(&format!("{y}: {entries}\n"));
                }
                text.push_str(&format!(
                    "Number of comparable pairs = {}\n",
                    KgbGraph::n_bruhat_comparable(&hasse)
                ));
                Ok(text)
            } else {
                // makeDotFile (kgb_io.cpp:182-274): one pass over the
                // source elements and generators emits the cross/cayley
                // edges (black/blue/green); a second pass adds gray
                // closure edges for Hasse pairs not already covered.
                let ids: Vec<KgbId> = graph.ids().collect();
                let rank = graph.semisimple_rank();
                let mut text = format!(
                    "kgbsize: {}\ndigraph G {{\nratio=\"1.5\"\nsize=\"7.5,10.0\"\n",
                    graph.size()
                );
                for i in 0..graph.size() {
                    text.push_str(&format!("v{i}\n"));
                }
                let mut edges: Vec<std::collections::BTreeSet<usize>> =
                    vec![std::collections::BTreeSet::new(); graph.size()];
                for (i, &id) in ids.iter().enumerate() {
                    for j in 0..rank {
                        match graph.status(id, j) {
                            Some(KgbStatus::Complex) => {
                                let ca = graph.cross(id, j).expect("complex cross").index();
                                if ca > i {
                                    text.push_str(&format!(
                                        "v{ca} -> v{i}[color=black] [arrowhead=none] [style=bold]\n"
                                    ));
                                    edges[ca].insert(i);
                                }
                            }
                            Some(KgbStatus::ImaginaryNoncompact) => {
                                let ca = graph.cross(id, j).expect("cross").index();
                                let ct = graph
                                    .cayley(ids[i], j)
                                    .expect("cayley")
                                    .expect("noncompact has a link")
                                    .index();
                                let color = if ca != i { "blue" } else { "green" };
                                text.push_str(&format!(
                                    "v{ct} -> v{i}[color={color}] [arrowhead=none] [style=bold]\n"
                                ));
                                edges[ct].insert(i);
                            }
                            _ => {}
                        }
                    }
                }
                for i in 0..graph.size() {
                    for &e in &hasse[i] {
                        if !edges[i].contains(&e) {
                            text.push_str(&format!("v{i} -> v{e}[color=gray] [arrowhead=none]\n"));
                        }
                    }
                }
                text.push_str("}\n");
                Ok(text)
            }
        }
        // print_block / print_blockd (atlas-types.w:8869-8901, block_io.cpp:
        // 48-126): the block's elements as `z(x,y): length [descents]
        // cross... cayley... Cartan involution-word` rows; print_block uses
        // the Weyl word (as_invol_expr=false), print_blockd the twisted
        // reduced expression (as_invol_expr=true).
        "print_block" | "print_blockd" => {
            arity(name, arguments, 1, span)?;
            // print_param_block_wrapper (atlas-types.w:6653-6666,
            // blocks.cpp:1272-1286): the common block of the parameter's
            // own srm (no make_dominant), with the singular flags of the
            // parameter's own gamma (block.singular(p->val.gamma())).
            if name == "print_block" {
                if let Value::Domain(DomainValue::Param(parameter)) = &arguments[0] {
                    test_standard(parameter, "Cannot generate block", span)?;
                    let rc = rep_context(&parameter.context);
                    if matches!(
                        integral_block_scope(&rc, parameter.repr.gamma())
                            .map_err(|error| structure_diagnostic(error, span))?,
                        IntegralBlockScope::ProperSubsystem
                    ) {
                        let located = parameter
                            .context
                            .rep
                            .lookup_full_block(&parameter.repr)
                            .map_err(|error| structure_diagnostic(error, span))?;
                        if !located.has_identity_generator_attitude() {
                            return Err(structure_diagnostic(
                                StructureError::NotYetImplemented {
                                    feature:
                                        "print_block on a non-identity integral-subsystem attitude",
                                },
                                span,
                            ));
                        }
                        let rows = located_common_block_rows(&parameter.context, &located, span)?;
                        let mut text = format!(
                            "Parameter defines element {} of the following block:\n",
                            located.raw_row()
                        );
                        text.push_str(&render_common_block(&parameter.context, &rows));
                        return Ok(text);
                    }
                    let (rows, init) =
                        common_block_rows(&parameter.context, &parameter.repr, span)?;
                    let mut text =
                        format!("Parameter defines element {init} of the following block:\n");
                    text.push_str(&render_common_block(&parameter.context, &rows));
                    return Ok(text);
                }
            }
            let Value::Domain(DomainValue::Block(block)) = &arguments[0] else {
                return Err(type_error(span, "expected a Block"));
            };
            let graph = &block.graph;
            let primal = &block.rf.graph;
            let size = graph.size();
            let width = digits(size - 1);
            let mut max_x = 0;
            let mut max_y = 0;
            let mut last_length = 0;
            for z in 0..size {
                if let Some(x) = graph.x(z) {
                    max_x = max_x.max(x.index());
                }
                if let Some(y) = graph.y(z) {
                    max_y = max_y.max(y.index());
                }
                if let Some(length) = graph.length(z) {
                    last_length = last_length.max(length);
                }
            }
            let xwidth = digits(max_x);
            let ywidth = digits(max_y);
            let lwidth = digits(last_length);
            let pad = 2;
            let rank = graph.rank();
            // The word column reproduces WeylGroup::word's ELECTED canonical
            // expression (weyl.cpp:944-958), not an arbitrary reduced word
            // (greedy smallest-descent diverges, e.g. w0 of B2); build the
            // transducer group once per print.
            let compact_weyl = if name == "print_block" {
                Some(
                    CompactWeyl::new(
                        block
                            .rf
                            .parent
                            .inner_class
                            .root_system()
                            .datum()
                            .cartan_matrix(),
                    )
                    .map_err(|error| structure_diagnostic(error, span))?,
                )
            } else {
                None
            };
            let mut text = String::new();
            for z in 0..size {
                let x = graph.x(z).expect("in-range").index();
                let y = graph.y(z).expect("in-range").index();
                let length = graph.length(z).expect("in-range");
                text.push_str(&format!("{:width$}({:xwidth$},{:ywidth$}):", z, x, y));
                text.push_str(&format!("{:width$}", length, width = lwidth + pad));
                text.push_str(&" ".repeat(pad));
                // descents
                text.push('[');
                for generator in 0..rank {
                    if generator > 0 {
                        text.push(',');
                    }
                    text.push_str(block_descent_code(
                        graph.descent_value(z, generator).expect("in-range"),
                    ));
                }
                text.push(']');
                // cross actions
                for generator in 0..rank {
                    match graph.cross(z, generator) {
                        Some(target) => {
                            text.push_str(&format!("{:width$}", target, width = width + pad))
                        }
                        None => text.push_str(&format!("{:>width$}", '*', width = width + pad)),
                    }
                }
                text.push_str(&" ".repeat(pad + 1));
                // Cayley transforms
                for generator in 0..rank {
                    let pair = if graph
                        .descent_value(z, generator)
                        .expect("in-range")
                        .is_descent()
                    {
                        graph.inverse_cayley(z, generator).expect("in-range")
                    } else {
                        graph.cayley(z, generator).expect("in-range")
                    };
                    text.push('(');
                    match pair.0 {
                        Some(first) => text.push_str(&format!("{:width$}", first)),
                        // block_io.cpp:197/205: setw right-aligns the '*'
                        // (chars left-align in Rust fmt, so say so).
                        None => text.push_str(&format!("{:>width$}", '*')),
                    }
                    text.push(',');
                    match pair.1 {
                        Some(second) => text.push_str(&format!("{:width$}", second)),
                        None => text.push_str(&format!("{:>width$}", '*')),
                    }
                    text.push(')');
                    text.push_str(&" ".repeat(pad));
                }
                // Block::print: Cartan class then the involution word.
                let id = graph.x(z).expect("in-range");
                let cartan = primal.cartan_of(id).expect("in-range");
                let number = cartan_number(&block.rf.parent, cartan)
                    .expect("the graph's Cartans are in range");
                let cwidth = digits(
                    cartan_number(
                        &block.rf.parent,
                        primal
                            .cartan_of(graph.x(size - 1).expect("in-range"))
                            .expect("in-range"),
                    )
                    .expect("in-range"),
                );
                text.push_str(&format!("{:width$}", number, width = cwidth));
                text.push_str(&" ".repeat(2));
                // Weyl word (prettyprint::printWeylElt, basic_io.cpp:100-118):
                // one-based generators comma-separated, `e` for the identity.
                let record = block
                    .rf
                    .table
                    .record(primal.involution_of(id).expect("in-range"))
                    .expect("in-range");
                if name == "print_block" {
                    // Weyl word (prettyprint::printWeylElt over
                    // WeylGroup::word's elected expression): one-based
                    // generators comma-separated, `e` for the identity.
                    let word = compact_weyl
                        .as_ref()
                        .expect("print_block built the transducer group")
                        .canonical_word(&weyl_reduced_word(
                            &block.rf.parent.inner_class,
                            record.weyl_element(),
                        ));
                    if word.is_empty() {
                        text.push('e');
                    } else {
                        let digits: Vec<String> = word
                            .iter()
                            .map(|&generator| (generator + 1).to_string())
                            .collect();
                        text.push_str(&digits.join(","));
                    }
                } else {
                    // printInvolution (prettyprint.cpp:219-232): one-based
                    // generator digits, '^' for crosses, 'x' for
                    // conjugations, `e` closing.
                    let word = block
                        .rf
                        .parent
                        .inner_class
                        .canonical_involution_expr(record.weyl_element())
                        .expect("a KGB involution is a twisted involution of the class");
                    for entry in word {
                        if entry >= 0 {
                            text.push(char::from(
                                b'1' + u8::try_from(entry).expect("generator rank"),
                            ));
                            text.push('^');
                        } else {
                            text.push(char::from(
                                b'1' + u8::try_from(!entry).expect("generator rank"),
                            ));
                            text.push('x');
                        }
                    }
                    text.push('e');
                }
                text.push('\n');
            }
            Ok(text)
        }
        // print_c_block_wrapper (atlas-types.w:6668-6695): test_standard,
        // then lookup_full_block (repr.cpp:1773-1794) makes the parameter
        // dominant in place — the block and the singular flags both use
        // that dominant gamma. The header's transform word is always empty
        // here: the block modifier is trivial for a dominant gamma and the
        // Rep_table pool is memoization only (fresh-build-per-call).
        "print_common_block" => {
            arity(name, arguments, 1, span)?;
            let Value::Domain(DomainValue::Param(parameter)) = &arguments[0] else {
                return Err(type_error(
                    span,
                    format!(
                        "{name} has no matching overload for {} argument(s)",
                        arguments.len()
                    ),
                ));
            };
            test_standard(parameter, "Cannot generate block", span)?;
            let rc = rep_context(&parameter.context);
            let dominant = parameter
                .repr
                .made_dominant(&rc)
                .map_err(|error| structure_diagnostic(error, span))?;
            match integral_block_scope(&rc, dominant.gamma())
                .map_err(|error| structure_diagnostic(error, span))?
            {
                IntegralBlockScope::Singleton => {
                    let (rows, init) = common_block_rows(&parameter.context, &dominant, span)?;
                    let mut text = format!(
                        "Parameter defines element {init} of the following common block,\nas transformed by <>:\n"
                    );
                    text.push_str(&render_common_block(&parameter.context, &rows));
                    return Ok(text);
                }
                IntegralBlockScope::ProperSubsystem | IntegralBlockScope::Full => {}
            }
            let located = parameter
                .context
                .rep
                .lookup_full_block(&dominant)
                .map_err(|error| structure_diagnostic(error, span))?;
            if !located.has_identity_generator_attitude() {
                return Err(structure_diagnostic(
                    StructureError::NotYetImplemented {
                        feature: "print_common_block on a non-identity integral-subsystem attitude",
                    },
                    span,
                ));
            }
            let rows = located_common_block_rows(&parameter.context, &located, span)?;
            let init = located.raw_row();
            let mut text = format!(
                "Parameter defines element {init} of the following common block,\nas transformed by <>:\n"
            );
            text.push_str(&render_common_block(&parameter.context, &rows));
            Ok(text)
        }
        // print_part_param_block_wrapper (atlas-types.w:6700-6711): the
        // Bruhat interval below the parameter's own srm, printed as a
        // partial common block with no header.
        "print_partial_block" => {
            arity(name, arguments, 1, span)?;
            let Value::Domain(DomainValue::Param(parameter)) = &arguments[0] else {
                return Err(type_error(
                    span,
                    format!(
                        "{name} has no matching overload for {} argument(s)",
                        arguments.len()
                    ),
                ));
            };
            test_standard(parameter, "Cannot generate block", span)?;
            let (rows, _) = partial_block_rows(
                &parameter.context,
                &parameter.repr,
                parameter.repr.gamma(),
                span,
            )?;
            Ok(render_common_block(&parameter.context, &rows))
        }
        // print_pc_block_wrapper (atlas-types.w:6713-6735): `Rep_table::
        // lookup` (repr.cpp:1796-1824) normalises the parameter and, on a
        // fresh table, builds only the Bruhat interval below it
        // (add_block_below, repr.cpp:1585-1645); the block modifier is then
        // cleared ("relative to ourselves"), so the shift is a no-op and
        // the singular flags use the parameter's own gamma. The seed is the
        // top element of its interval, so below(init_index) is full with
        // init_index == size-1 and NO header prints: the "Elements <= ..."
        // header (atlas-types.w:6721-6722) needs init_index+1 < size; the
        // "Subset ..." branch fires on a cross-call block cache hit with a
        // non-full below-set.
        "print_partial_common_block" => {
            arity(name, arguments, 1, span)?;
            let Value::Domain(DomainValue::Param(parameter)) = &arguments[0] else {
                return Err(type_error(
                    span,
                    format!(
                        "{name} has no matching overload for {} argument(s)",
                        arguments.len()
                    ),
                ));
            };
            test_standard(parameter, "Cannot generate block", span)?;
            let located = parameter
                .context
                .rep
                .lookup(&parameter.repr)
                .map_err(|error| structure_diagnostic(error, span))?;
            if !located.has_identity_generator_attitude() {
                return Err(runtime(
                    span,
                    "partial common block on a non-identity integral-subsystem attitude is not yet supported",
                ));
            }
            let block = located.block();
            let init = located.raw_row();
            let hasse = block_bruhat_hasse(block.as_ref());
            // Strict Bruhat downset of the start element: every row reachable
            // from `init` through the Hasse diagram (hasse edges only go
            // downward, so `init` itself is never reached).
            let mut below = vec![false; block.size()];
            let mut stack = vec![init];
            while let Some(z) = stack.pop() {
                for &down in &hasse[z] {
                    if !below[down] {
                        below[down] = true;
                        stack.push(down);
                    }
                }
            }
            let mut text = String::new();
            if (0..init).all(|z| below[z]) {
                if init + 1 < block.size() {
                    text.push_str(&format!("Elements <= {init} of following block\n"));
                }
            } else {
                text.push_str("Subset {");
                for (z, &is_below) in below.iter().enumerate() {
                    if is_below {
                        text.push_str(&format!("{z},"));
                    }
                }
                text.push_str(&format!("{init}}} in the following common block:\n"));
            }
            let rows = located_common_block_rows(&parameter.context, &located, span)?;
            text.push_str(&render_common_block(&parameter.context, &rows));
            Ok(text)
        }
        // only the unitary block elements (the involution support is
        // contained in the weak descents), with the filtered descent sets
        // and the twisted reduced expression.
        // print_blocku (atlas-types.w:8894-8917, block_io.cpp:282-369):
        // only the unitary block elements (the involution support is
        // contained in the weak descents), with the filtered descent sets
        // and the twisted reduced expression.
        "print_blocku" => {
            arity(name, arguments, 1, span)?;
            let Value::Domain(DomainValue::Block(block)) = &arguments[0] else {
                return Err(type_error(span, "expected a Block"));
            };
            let graph = &block.graph;
            let primal = &block.rf.graph;
            let size = graph.size();
            let width = digits(size - 1);
            let mut max_x = 0;
            let mut max_y = 0;
            let mut last_length = 0;
            let mut last_cartan = 0;
            for z in 0..size {
                if let Some(x) = graph.x(z) {
                    max_x = max_x.max(x.index());
                }
                if let Some(y) = graph.y(z) {
                    max_y = max_y.max(y.index());
                }
                if let Some(length) = graph.length(z) {
                    last_length = last_length.max(length);
                }
                let id = graph.x(z).expect("in-range");
                let number =
                    cartan_number(&block.rf.parent, primal.cartan_of(id).expect("in-range"))
                        .expect("in-range");
                last_cartan = last_cartan.max(number);
            }
            let xwidth = digits(max_x);
            let ywidth = digits(max_y);
            let lwidth = digits(last_length);
            let cwidth = digits(last_cartan);
            let pad = 2;
            let rank = graph.rank();
            let mut text = String::new();
            for z in 0..size {
                let support = {
                    let id = graph.x(z).expect("in-range");
                    let record = block
                        .rf
                        .table
                        .record(primal.involution_of(id).expect("in-range"))
                        .expect("in-range");
                    let word =
                        weyl_reduced_word(&block.rf.parent.inner_class, record.weyl_element());
                    let mut flags = vec![false; rank];
                    for generator in word {
                        if generator < rank {
                            flags[generator] = true;
                        }
                    }
                    flags
                };
                let mut unitary = true;
                for (s, &in_support) in support.iter().enumerate() {
                    if in_support && !graph.descent_value(z, s).expect("in-range").is_descent() {
                        unitary = false;
                        break;
                    }
                }
                if !unitary {
                    continue;
                }
                let x = graph.x(z).expect("in-range").index();
                let y = graph.y(z).expect("in-range").index();
                let length = graph.length(z).expect("in-range");
                text.push_str(&format!("{:width$}({:xwidth$},{:ywidth$}):", z, x, y));
                text.push_str(&format!("{:width$}", length, width = lwidth + pad));
                let id = graph.x(z).expect("in-range");
                let number =
                    cartan_number(&block.rf.parent, primal.cartan_of(id).expect("in-range"))
                        .expect("in-range");
                text.push_str(&format!("{:width$}", number, width = cwidth + pad));
                text.push_str(&" ".repeat(pad));
                text.push_str(&block_descent_set(graph, z, rank, &vec![true; rank]));
                text.push_str(&" ".repeat(pad));
                text.push_str(&block_descent_set(graph, z, rank, &support));
                text.push_str(&" ".repeat(pad));
                let record = block
                    .rf
                    .table
                    .record(primal.involution_of(id).expect("in-range"))
                    .expect("in-range");
                let word = block
                    .rf
                    .parent
                    .inner_class
                    .canonical_involution_expr(record.weyl_element())
                    .expect("in-range");
                for entry in word {
                    if entry >= 0 {
                        text.push(char::from(
                            b'1' + u8::try_from(entry).expect("generator rank"),
                        ));
                        text.push('^');
                    } else {
                        text.push(char::from(
                            b'1' + u8::try_from(!entry).expect("generator rank"),
                        ));
                        text.push('x');
                    }
                }
                text.push('e');
                text.push('\n');
            }
            Ok(text)
        }
        // print_KL_basis / print_prim_KL / print_KL_list / print_W_graph /
        // print_W_cells (atlas-types.w:9017-9083): the KLV polynomials of
        // the block's KL table, the W-graph, and its cell decomposition.
        "print_KL_basis" | "print_prim_KL" | "print_KL_list" | "print_W_graph"
        | "print_W_cells" => {
            arity(name, arguments, 1, span)?;
            let Value::Domain(DomainValue::Block(block)) = &arguments[0] else {
                return Err(type_error(span, "expected a Block"));
            };
            let mut kl_table =
                KlTable::new(&block.graph).map_err(|error| structure_diagnostic(error, span))?;
            kl_table
                .fill(0)
                .map_err(|error| structure_diagnostic(error, span))?;
            let size = block.graph.size();
            let width = digits(size.saturating_sub(1));
            let tab = 2;
            let mut text = String::new();
            if name == "print_KL_basis" {
                text.push_str("Full list of non-zero Kazhdan-Lusztig-Vogan polynomials:\n\n");
                let mut count = 0_usize;
                for y in 0..size {
                    text.push_str(&format!("{y:width$}: "));
                    let mut first = true;
                    for x in 0..=y {
                        let polynomial = kl_pol_at(&kl_table, x, y, span)?;
                        if polynomial.is_zero() {
                            continue;
                        }
                        if first {
                            text.push_str(&format!("{x:width$}: "));
                            first = false;
                        } else {
                            text.push_str(&" ".repeat(width + tab));
                            text.push_str(&format!("{x:width$}: "));
                        }
                        text.push_str(&print_klpol(&polynomial));
                        text.push('\n');
                        count += 1;
                    }
                    text.push('\n');
                }
                let hasse = block.graph.bruhat_hasse();
                let comparable = BlockGraph::n_bruhat_comparable(&hasse);
                let zeros = comparable - count;
                text.push_str(&format!(
                    "{count} nonzero polynomial{}",
                    if count == 1 { "" } else { "s" }
                ));
                text.push_str(&format!(
                    ", and {zeros} zero polynomial{},\n",
                    if zeros == 1 { "" } else { "s" }
                ));
                text.push_str(&format!(
                    " at {comparable} Bruhat-comparable {}\n",
                    if comparable == 1 { "pair." } else { "pairs." }
                ));
            } else if name == "print_prim_KL" {
                text.push_str(
                    "Non-zero Kazhdan-Lusztig-Vogan polynomials for primitive pairs:\n\n",
                );
                let hasse = block.graph.bruhat_hasse();
                let lesseq: Vec<Vec<bool>> = bruhat_closure(&hasse);
                let mut count = 0_usize;
                let mut zero_count = 0_usize;
                let mut incomp_count = 0_usize;
                for (y, lesseq_row) in lesseq.iter().enumerate() {
                    let prims = kl_primitives(&kl_table, y);
                    text.push_str(&format!("{y:width$}: "));
                    let mut first = true;
                    for x in prims {
                        if lesseq_row[x] {
                            count += 1;
                            let polynomial = kl_pol_at(&kl_table, x, y, span)?;
                            if polynomial.is_zero() {
                                zero_count += 1;
                            }
                            if first {
                                text.push_str(&format!("{x:width$}: "));
                                first = false;
                            } else {
                                text.push_str(&" ".repeat(width + tab));
                                text.push_str(&format!("{x:width$}: "));
                            }
                            text.push_str(&print_klpol(&polynomial));
                            text.push('\n');
                        } else {
                            incomp_count += 1;
                        }
                    }
                    count += 1; // P_{y,y}
                    if !first {
                        // upstream pads setw(width+tab) before the P_{y,y}
                        // line (kl_io.cpp:138-139)
                        text.push_str(&format!("{:pad$}", "", pad = width + tab));
                    }
                    text.push_str(&format!("{y:width$}: 1\n\n"));
                }
                text.push_str(&format!(
                    "{count} Bruhat-comparable primitive {}",
                    if count == 1 { "pair" } else { "pairs" }
                ));
                text.push_str(&format!(
                    ", of which {zero_count} ha{} null polynomial,\n",
                    if zero_count == 1 { "s" } else { "ve" }
                ));
                text.push_str(&format!(
                    " and {incomp_count} incomparable primitive {}\n",
                    if incomp_count == 1 { "pair" } else { "pairs" }
                ));
            } else if name == "print_W_graph" {
                // print_W_graph (atlas-types.w:9068-9078, wgraph_io.cpp:47-62):
                // the W-graph of the block: descent sets and the mu edges.
                let rank = block.graph.rank();
                // kl::wGraph (kl.cpp:1042-1058): every mu pair contributes
                // an edge in BOTH directions, sorted by target.
                let mut edges: Vec<Vec<(usize, i32)>> = vec![Vec::new(); size];
                for y in 0..size {
                    for pair in kl_table.mu_column(y) {
                        edges[y].push((pair.x, pair.coef));
                        edges[pair.x].push((y, pair.coef));
                    }
                }
                for edges_z in edges.iter_mut() {
                    edges_z.sort_unstable();
                }
                for (z, edges_z) in edges.iter().enumerate() {
                    text.push_str(&format!("{z}:"));
                    let desc = kl_table.support().descent_set(z);
                    let mut gens: Vec<String> = Vec::new();
                    for s in 0..rank {
                        if desc.is_set(s) {
                            gens.push((s + 1).to_string());
                        }
                    }
                    text.push_str(&format!("{{{}}}:{{", gens.join(",")));
                    for (j, &(target, coef)) in edges_z.iter().enumerate() {
                        if j > 0 {
                            text.push(',');
                        }
                        text.push_str(&format!("({target},{coef})"));
                    }
                    text.push_str("}\n");
                }
            } else if name == "print_W_cells" {
                // print_W_cells (atlas-types.w:9058-9065, wgraph.cpp:58-116,
                // wgraph_io.cpp:78-122): the cell decomposition of the
                // W-graph via the oriented graph's strong components.
                let rank = block.graph.rank();
                let mut edges: Vec<Vec<(usize, i32)>> = vec![Vec::new(); size];
                for y in 0..size {
                    for pair in kl_table.mu_column(y) {
                        edges[y].push((pair.x, pair.coef));
                        edges[pair.x].push((y, pair.coef));
                    }
                }
                for edges_z in edges.iter_mut() {
                    edges_z.sort_unstable();
                }
                // oriented_graph (wgraph.cpp:58-72): drop x -> y when the
                // descent set of x is contained in that of y.
                let mut oriented: Vec<Vec<usize>> = Vec::with_capacity(size);
                for (x, edges_x) in edges.iter().enumerate() {
                    let desc_x = kl_table.support().descent_set(x).clone();
                    let mut targets = Vec::new();
                    for &(y, _) in edges_x {
                        if !kl_table.support().descent_set(y).contains(&desc_x) {
                            targets.push(y);
                        }
                    }
                    oriented.push(targets);
                }
                let (mut partition, induced) = strong_components(&oriented);
                // The oracle's Partition lists a cell's vertices in
                // ascending order (Partition::iterator traversal).
                for members in partition.iter_mut() {
                    members.sort_unstable();
                }
                // Cells and their vertices.
                text.push_str("// Cells and their vertices.\n");
                for (i, members) in partition.iter().enumerate() {
                    let joined = members
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(",");
                    text.push_str(&format!("#{i}={{{joined}}}\n"));
                }
                // Induced graph on cells.
                text.push_str("\n// Induced graph on cells.\n");
                for (i, targets) in induced.iter().enumerate() {
                    text.push_str(&format!("#{i}:"));
                    for (j, &target) in targets.iter().enumerate() {
                        text.push_str(if j == 0 { "->#" } else { ",#" });
                        text.push_str(&target.to_string());
                    }
                    text.push_str(".\n");
                }
                // Individual cells.
                text.push_str("\n// Individual cells.\n");
                let mut relno = vec![0_usize; size];
                for (i, members) in partition.iter().enumerate() {
                    text.push_str(&format!("// cell #{i}:\n"));
                    for (j, &original) in members.iter().enumerate() {
                        relno[original] = j;
                    }
                    for (j, &original) in members.iter().enumerate() {
                        text.push_str(&format!("{j}[{original}]: "));
                        let desc = kl_table.support().descent_set(original);
                        let mut gens: Vec<String> = Vec::new();
                        for s in 0..rank {
                            if desc.is_set(s) {
                                gens.push((s + 1).to_string());
                            }
                        }
                        text.push_str(&format!("{{{}}}", gens.join(",")));
                        let mut first_edge = true;
                        for &(target, coef) in &edges[original] {
                            if !partition[i].contains(&target) {
                                continue;
                            }
                            text.push_str(if first_edge { " --> " } else { "," });
                            first_edge = false;
                            if coef == 1 {
                                text.push_str(&relno[target].to_string());
                            } else {
                                text.push_str(&format!("({},{coef})", relno[target]));
                            }
                        }
                        text.push('\n');
                    }
                    text.push('\n');
                }
            } else {
                // print_KL_list (kl_io.cpp:155-170): the sorted distinct
                // nonzero polynomials of the KL store (the whole pool,
                // which for an empty block still holds the constant one).
                let mut polynomials: Vec<KlPol> = Vec::new();
                for index in 0..kl_table.pool().len() {
                    let Some(polynomial) = kl_table.pool().get(index) else {
                        continue;
                    };
                    if !polynomial.is_zero() && !polynomials.contains(polynomial) {
                        polynomials.push(polynomial.clone());
                    }
                }
                polynomials.sort_by(compare_klpol);
                for polynomial in &polynomials {
                    text.push_str(&print_klpol(polynomial));
                    text.push('\n');
                }
            }
            Ok(text)
        }
        other => panic!("printer dispatch saw {other}"),
    }
}

/// Decimal digit count with `digits(0) == 1` (ioutils.cpp:32-41).
fn digits(mut value: usize) -> usize {
    let mut count = 1;
    while value >= 10 {
        count += 1;
        value /= 10;
    }
    count
}

/// kgb_io::var_print_KGB (kgb_io.cpp:140-148) and plain kgb_io::print on a
/// selection (kgb_io.cpp:59-126, NON-traditional branch), behind the
/// wrappers at atlas-types.w:8944-8973. `which = None` is the full table:
/// the wrapper's `kgbsize` line and the `Base grading` header print first.
/// `Some` is the selection variant: no header lines, the listed rows in
/// list order; the `#` flag's inner class is present in BOTH variants.
fn print_kgb(context: &Arc<RealFormContext>, which: Option<&[KgbId]>) -> String {
    let graph = &context.graph;
    let parent = &context.parent;
    let rank = graph.semisimple_rank();
    let size = graph.size();
    let last = graph.ids().last().expect("a KGB graph is nonempty");
    let width = digits(size - 1);
    let cartan_width = digits(
        cartan_number(
            parent,
            graph
                .cartan_of(last)
                .expect("the last element has a Cartan"),
        )
        .expect("the graph's Cartans are in range"),
    );
    let length_width = digits(graph.length(last).expect("the last element has a length"));

    let mut text = String::new();
    if which.is_none() {
        text.push_str(&format!("kgbsize: {size}\nBase grading: ["));
        for &bit in graph.base_grading() {
            text.push(if bit { '1' } else { '0' });
        }
        text.push_str("].\n");
    }
    let ids: Vec<KgbId> = match which {
        Some(selection) => selection.to_vec(),
        None => graph.ids().collect(),
    };
    for id in ids {
        let length = graph.length(id).expect("in-range element");
        let cartan = graph.cartan_of(id).expect("in-range element");
        let number = cartan_number(parent, cartan).expect("the graph's Cartans are in range");
        // The '#' flag (kgb_io.cpp:109-112): the element's involution IS
        // its Cartan class's canonical representative.
        let flag = {
            let representative = parent
                .classification
                .cartan_class(cartan)
                .expect("the graph's Cartans are in range")
                .representative();
            let canonical = WeylElement::from_action(
                parent.inner_class.root_system(),
                representative.weyl_action(),
            )
            .expect("the Cartan representative realizes in the root system");
            context.table.lookup(&canonical) == graph.involution_of(id)
        };
        let record = context
            .table
            .record(graph.involution_of(id).expect("in-range element"))
            .expect("the graph's involutions are table records");
        let word = parent
            .inner_class
            .canonical_involution_expr(record.weyl_element())
            .expect("a KGB involution is a twisted involution of the class");

        write!(text, "{:>width$}:  ", id.index()).expect("string write");
        write!(text, "{length:>length_width$}").expect("string write");
        text.push_str("  ");
        text.push('[');
        for generator in 0..rank {
            if generator > 0 {
                text.push(',');
            }
            // prettyprint::printStatus (prettyprint.cpp:284-313).
            text.push(
                match graph.status(id, generator).expect("in-range element") {
                    KgbStatus::Complex => 'C',
                    KgbStatus::ImaginaryCompact => 'c',
                    KgbStatus::ImaginaryNoncompact => 'n',
                    KgbStatus::Real => 'r',
                },
            );
        }
        text.push(']');
        text.push(' ');
        for generator in 0..rank {
            let cross = graph.cross(id, generator).expect("in-range element");
            write!(text, "{:>width$}", cross.index(), width = width + 2).expect("string write");
        }
        text.push_str("  ");
        for generator in 0..rank {
            match graph.cayley(id, generator).expect("in-range element") {
                Some(cayley) => write!(text, "{:>width$}", cayley.index(), width = width + 2)
                    .expect("string write"),
                None => write!(text, "{:>width$}", '*', width = width + 2).expect("string write"),
            }
        }
        text.push_str("  ");
        // The element's torus part (kgb.cpp:750-754 via
        // prettyprint.cpp:72-85): bits comma-separated in parentheses.
        text.push('(');
        let bits = graph.element(id).expect("in-range element").torus_bits();
        for index in 0..bits.dimension() {
            if index > 0 {
                text.push(',');
            }
            text.push(if bits.bit(index) == Some(true) {
                '1'
            } else {
                '0'
            });
        }
        text.push(')');
        text.push(if flag { '#' } else { ' ' });
        write!(text, "{number:>cartan_width$} ").expect("string write");
        // prettyprint::printInvolution (prettyprint.cpp:219-232): one-based
        // generator digits, '^' for crosses, 'x' for conjugations, `e`
        // closing (digits beyond 9 wrap through the ASCII table, as
        // upstream's char arithmetic does).
        for entry in word {
            if entry >= 0 {
                text.push(char::from(
                    b'1' + u8::try_from(entry).expect("generator rank"),
                ));
                text.push('^');
            } else {
                text.push(char::from(
                    b'1' + u8::try_from(!entry).expect("generator rank"),
                ));
                text.push('x');
            }
        }
        text.push('e');
        text.push('\n');
    }
    text
}

/// output::printStrongReal (output.cpp:490-540) behind
/// print_strongreal_wrapper (atlas-types.w:8850-8859). The ioutils::foldLine
/// wrap of overlong `real form` lines is an upstream large-orbit refinement
/// the frozen contracts never reach.
fn print_strong_real(
    context: &Arc<InnerClassContext>,
    id: CartanId,
    span: SourceSpan,
) -> Result<String, Diagnostic> {
    let blocks = atlas_real_group::strong_real_class_prints(
        &context.inner_class,
        &context.classification,
        &context.strong,
        &context.order,
        id,
        &INTEGER_BUDGET,
    )
    .map_err(|error| runtime(span, error.to_string()))?;
    let mut text = String::new();
    if blocks.len() > 1 {
        writeln!(text, "there are {} real form classes:\n", blocks.len()).expect("string write");
    }
    for block in &blocks {
        // The RatWeight print of basic_io.cpp:138-145: the numerator as a
        // compact seqPrint vector, then `/denominator`.
        let (numerators, denominator) = block.square();
        let mut square = String::from("[");
        for (index, numerator) in numerators.iter().enumerate() {
            if index > 0 {
                square.push(',');
            }
            write!(square, "{numerator}").expect("string write");
        }
        square.push(']');
        writeln!(
            text,
            "class #{}, possible square: exp(2i\\pi({square}/{denominator}))",
            block.class_number()
        )
        .expect("string write");
        for (external, elements) in block.real_forms() {
            write!(text, "real form #{external}: [").expect("string write");
            for (index, element) in elements.iter().enumerate() {
                if index > 0 {
                    text.push(',');
                }
                write!(text, "{element}").expect("string write");
            }
            writeln!(text, "] ({})", elements.len()).expect("string write");
        }
        if blocks.len() > 1 {
            text.push('\n');
        }
    }
    Ok(text)
}

/// ioutils::foldLine (ioutils.cpp:67-127) as `print_gradings` calls it:
/// pre-hyphens empty, post-hyphen ",", indent 4, line size 79. Breaks land
/// just after a comma; with no comma in range the break is brutal.
fn fold_line(line: &str) -> String {
    const LINE_SIZE: usize = 79;
    const INDENT: usize = 4;
    if line.len() <= LINE_SIZE {
        return line.to_string();
    }
    let bytes = line.as_bytes(); // the gradings line is pure ASCII
    let rfind_comma = |pos: usize| {
        let last = pos.min(bytes.len() - 1);
        (0..=last).rev().find(|&index| bytes[index] == b',')
    };
    let mut output = String::new();
    let mut point = 0_usize;
    loop {
        let old_break = point;
        let indent = if old_break == 0 { 0 } else { INDENT };
        if let Some(bp) = rfind_comma(old_break + LINE_SIZE - indent) {
            if bp + 1 > point {
                point = bp + 1;
            }
        }
        if point == old_break {
            point += LINE_SIZE - indent;
        }
        output.push_str(&" ".repeat(indent));
        output.push_str(&line[old_break..point]);
        output.push('\n');
        if line.len() <= point + LINE_SIZE - INDENT {
            break;
        }
    }
    if point < line.len() {
        output.push_str(&" ".repeat(INDENT));
        output.push_str(&line[point..]);
    }
    output
}

/// print_X_wrapper (atlas-types.w:8999-9008, installed :9124): the global
/// Tits group X* table of the inner class, unconditional like print_KGB;
/// upstream builds a fresh InvolutionTable inside `global_KGB kgb(G)`.
fn print_x(context: &Arc<InnerClassContext>, span: SourceSpan) -> Result<String, Diagnostic> {
    let mut table = InnerClassContext::fresh_table(context)
        .map_err(|error| runtime(span, error.to_string()))?;
    let kgb = GlobalKgb::build(
        &context.inner_class,
        &context.classification,
        &mut table,
        &INTEGER_BUDGET,
    )
    .map_err(|error| runtime(span, error.to_string()))?;
    Ok(kgb
        .print_layout()
        .map_err(|error| runtime(span, error.to_string()))?
        .render())
}

/// print_gradings_wrapper (atlas-types.w:4260-4300): the guard clauses are
/// fiber_partition's, then the imaginary subsystem header in Bourbaki order
/// and one grading bit string per fiber element of the real form's part
/// (upstream `sigma.pull_back(gr)`: printed bit i is bit sigma[i]).
fn print_gradings(
    context: &Arc<InnerClassContext>,
    id: CartanId,
    form: &Arc<RealFormContext>,
    span: SourceSpan,
) -> Result<String, Diagnostic> {
    fiber_partition_membership(context, id, form, span)?;
    let cartan = context
        .classification
        .cartan_class(id)
        .expect("CartanClass values carry an in-range id");
    let grading_data = cartan.grading();
    let root_system = context.inner_class.root_system();
    let numbering = RootNumbering::new(root_system, context.root_datum.prefers_coroots());
    // Upstream `si` is the fiber's simpleImaginary in RootNbr-ascending
    // order (the oracle prints B2's compact Cartan as "simple roots 4,5").
    // The crate list order differs, but the GRADING bits stay aligned with
    // it, so each entry carries its bit index along.
    let mut simples: Vec<(RootId, usize)> = grading_data
        .imaginary_simple_roots()
        .iter()
        .copied()
        .enumerate()
        .map(|(bit, root)| (root, bit))
        .collect();
    simples.sort_by_key(|&(root, _)| numbering.nbr(root));
    // root_datum().Cartan_matrix(si): the subsystem Cartan matrix in si
    // order; sigma is its Bourbaki numbering (DynkinDiagram(cm).perm()).
    let cartan_matrix = simples
        .iter()
        .map(|&(root, _)| {
            simples
                .iter()
                .map(|&(coroot, _)| {
                    root_system
                        .bracket(root, coroot)
                        .map_err(|error| runtime(span, error.to_string()))
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .collect::<Result<Vec<Vec<_>>, _>>()?;
    let sigma =
        bourbaki_permutation(&cartan_matrix).map_err(|error| runtime(span, error.to_string()))?;
    let mut text = String::from("Imaginary root system is ");
    if simples.is_empty() {
        text.push_str("empty.\n");
    } else {
        // dynkin::Lie_type(cm): the letter of a rank-two double edge is
        // decided by the GIVEN order (dynkin.cpp:113), so the type comes
        // from the unpermuted matrix.
        let lie_type = infer_lie_type(&cartan_matrix, cartan_matrix.len(), span)?;
        write!(
            text,
            "of type {}, with simple root{}",
            lie_type.render(),
            if simples.len() == 1 { " " } else { "s " }
        )
        .expect("string write");
        for (position, &source) in sigma.iter().enumerate() {
            write!(
                text,
                "{}{}",
                numbering.nbr(simples[source].0),
                if position + 1 < sigma.len() {
                    ","
                } else {
                    ".\n"
                }
            )
            .expect("string write");
        }
    }
    let fiber = grading_data.adjoint_fiber();
    let dimension = fiber.dimension();
    // The partition's mask-bits bound keeps this shift in range.
    let element_count = 1_u64
        .checked_shl(
            u32::try_from(dimension)
                .map_err(|_| runtime(span, "internal fiber dimension overflow"))?,
        )
        .ok_or_else(|| runtime(span, "internal fiber dimension overflow"))?;
    let mut line = String::new();
    let mut first = true;
    for mask in 0..element_count {
        let local = cartan
            .partition()
            .class_of_mask(mask)
            .map_err(|error| runtime(span, error.to_string()))?;
        if cartan.labels().label(local) != Some(form.internal) {
            continue;
        }
        line.push(if first { '[' } else { ',' });
        first = false;
        // The fiber element of this mask (weak_real_form's element_of
        // recipe): xor the basis representatives of the set bits, then
        // interpret the ambient class as an adjoint fiber element.
        let mut representative = ModTwoVector::zero(fiber.datum().rank())
            .map_err(|error| runtime(span, error.to_string()))?;
        for (index, basis) in fiber.basis_representatives().iter().enumerate() {
            if mask & (1_u64 << index) != 0 {
                representative
                    .xor_assign(basis)
                    .map_err(|error| runtime(span, error.to_string()))?;
            }
        }
        let element = fiber
            .element_from_ambient(representative)
            .map_err(|error| runtime(span, error.to_string()))?;
        let grading = grading_data
            .grading(&element)
            .map_err(|error| runtime(span, error.to_string()))?;
        for &source in &sigma {
            let noncompact = grading
                .is_noncompact(simples[source].1)
                .expect("sigma indexes the imaginary simple roots");
            line.push(if noncompact { '1' } else { '0' });
        }
    }
    line.push_str("]\n");
    text.push_str(&fold_line(&line));
    Ok(text)
}

/// print_real_Weyl_wrapper (atlas-types.w:8831-8847): mismatch and
/// membership checks in the wrapper's wording and order, then the crate
/// print on the primal/dual inner-class pair.
fn print_real_weyl(
    context: &Arc<InnerClassContext>,
    id: CartanId,
    form: &Arc<RealFormContext>,
    span: SourceSpan,
) -> Result<String, Diagnostic> {
    if context.inner_class != form.parent.inner_class {
        return Err(runtime(span, "Inner class mismatch between arguments"));
    }
    let occurs = form
        .parent
        .classification
        .cartan_set(form.internal)
        .expect("a real form's internal number is in range")
        .contains(&id);
    if !occurs {
        return Err(runtime(span, "Cartan class not defined for real form"));
    }
    let dual = build_dual_inner_class(context, span)?;
    let budget = cartan_classification_budget();
    let cartan = cartan_number(context, id).expect("CartanClass values carry an in-range id");
    let print = atlas_real_group::real_weyl::RealWeylContext {
        inner_class: &context.inner_class,
        classification: &context.classification,
        dual_inner_class: &dual.inner_class,
        dual_classification: &dual.classification,
        budget: &budget,
    }
    .real_weyl_print(form.external, cartan)
    .map_err(|error| runtime(span, error.to_string()))?;
    Ok(print.render())
}

/// print_blockstabilizer_wrapper (atlas-types.w:8920-8932): upstream runs
/// no checks; the context is the BLOCK's inner class (dual rebuilt from
/// it), paired with the Cartan class's raw number from its own context
/// (output::printBlockStabilizer, output.cpp:361-390).
fn print_blockstabilizer(
    block: &BlockValue,
    cartan_context: &Arc<InnerClassContext>,
    id: CartanId,
    span: SourceSpan,
) -> Result<String, Diagnostic> {
    let parent = &block.rf.parent;
    let dual = build_dual_inner_class(parent, span)?;
    let budget = cartan_classification_budget();
    let cartan =
        cartan_number(cartan_context, id).expect("CartanClass values carry an in-range id");
    let print = atlas_real_group::real_weyl::RealWeylContext {
        inner_class: &parent.inner_class,
        classification: &parent.classification,
        dual_inner_class: &dual.inner_class,
        dual_classification: &dual.classification,
        budget: &budget,
    }
    .block_stabilizer_print(block.rf.external, cartan, block.dual_rf.external)
    .map_err(|error| runtime(span, error.to_string()))?;
    Ok(print.render())
}

/// Dispatch one named application. Unknown names are Name errors.
pub(crate) fn call(name: &str, arguments: &[Value], span: SourceSpan) -> Result<Value, Diagnostic> {
    call_with_printed(name, arguments, span, &mut Vec::new())
}

/// Owned evaluator dispatch. The three same-result-type hunger products
/// consume their pilfered operand directly; every other domain call keeps the
/// ordinary borrowed adapter path.
pub(crate) fn call_owned_with_printed(
    name: &str,
    arguments: Vec<Value>,
    span: SourceSpan,
    printed: &mut Vec<String>,
) -> Result<Value, Diagnostic> {
    if name == "*"
        && matches!(
            arguments.as_slice(),
            [
                Value::Domain(DomainValue::LieType(_)),
                Value::Domain(DomainValue::LieType(_))
            ] | [Value::Domain(DomainValue::WeylElement(_)), Value::Vector(_)]
                | [Value::Vector(_), Value::Domain(DomainValue::WeylElement(_))]
        )
    {
        return hungry_product_owned(arguments, span);
    }
    call_with_printed(name, &arguments, span, printed)
}

fn hungry_product_owned(mut arguments: Vec<Value>, span: SourceSpan) -> Result<Value, Diagnostic> {
    let right = arguments
        .pop()
        .expect("owned hungry product has two operands");
    let left = arguments
        .pop()
        .expect("owned hungry product has two operands");
    debug_assert!(arguments.is_empty());
    match (left, right) {
        (
            Value::Domain(DomainValue::LieType(mut left)),
            Value::Domain(DomainValue::LieType(right)),
        ) => {
            validate_combined_rank(&left, &right, span)?;
            left.factors.extend(right.factors);
            Ok(Value::Domain(DomainValue::LieType(left)))
        }
        (Value::Domain(DomainValue::WeylElement(weyl)), Value::Vector(mut vector)) => {
            validate_weight_rank(&weyl, &vector, span)?;
            word_act_weight(&weyl.context.handle.datum, &weyl.word, &mut vector.0);
            Ok(Value::Vector(vector))
        }
        (Value::Vector(mut vector), Value::Domain(DomainValue::WeylElement(weyl))) => {
            validate_coweight_rank(&vector, &weyl, span)?;
            for &generator in &weyl.word {
                simple_coreflect(&weyl.context.handle.datum, generator, &mut vector.0);
            }
            Ok(Value::Vector(vector))
        }
        _ => unreachable!("owned hunger dispatcher checks operand types"),
    }
}

fn validate_combined_rank(
    left: &LieTypeValue,
    right: &LieTypeValue,
    span: SourceSpan,
) -> Result<(), Diagnostic> {
    let combined_rank = left.total_rank().saturating_add(right.total_rank());
    if combined_rank > RANK_MAX {
        return Err(runtime(
            span,
            format!("Combined rank {combined_rank} exceeds implementation limit {RANK_MAX}"),
        ));
    }
    Ok(())
}

fn validate_weight_rank(
    weyl: &WeylEltValue,
    vector: &Vec32,
    span: SourceSpan,
) -> Result<(), Diagnostic> {
    let rank = weyl.context.handle.datum.lattice_rank();
    if vector.0.len() != rank {
        return Err(runtime(
            span,
            format!("Rank and weight size mismatch {rank}:{}", vector.0.len()),
        ));
    }
    Ok(())
}

fn validate_coweight_rank(
    vector: &Vec32,
    weyl: &WeylEltValue,
    span: SourceSpan,
) -> Result<(), Diagnostic> {
    let rank = weyl.context.handle.datum.lattice_rank();
    if vector.0.len() != rank {
        return Err(runtime(
            span,
            format!("Coweight size and rank mismatch {}:{rank}", vector.0.len()),
        ));
    }
    Ok(())
}

/// `call` with a side channel for mid-evaluation stdout writes; only
/// `partial_extended_KL_block` uses it (ext_kl.cpp:945-948).
pub(crate) fn call_with_printed(
    name: &str,
    arguments: &[Value],
    span: SourceSpan,
    printed: &mut Vec<String>,
) -> Result<Value, Diagnostic> {
    match name {
        // extend (atlas-types.w:280-289): append a simple factor to a
        // Lie type.
        "extend" => {
            arity(name, arguments, 3, span)?;
            let mut lie_type = as_lie_type(&arguments[0], span)?;
            let Value::String(type_string) = &arguments[1] else {
                return Err(type_error(span, "extend requires a string"));
            };
            let rank = as_usize(&arguments[2], span)?;
            let letter = type_string.chars().next().unwrap_or('T');
            lie_type.add_simple_factor(letter, rank);
            Ok(Value::Domain(DomainValue::LieType(lie_type)))
        }
        "Lie_type" => {
            arity(name, arguments, 1, span)?;
            let lie_type = match &arguments[0] {
                Value::Domain(DomainValue::RootDatum(handle)) => handle.lie_type().clone(),
                value => as_lie_type(value, span)?,
            };
            Ok(Value::Domain(DomainValue::LieType(lie_type)))
        }
        "Smith_Cartan" => {
            arity(name, arguments, 1, span)?;
            let lie_type = as_lie_type(&arguments[0], span)?;
            smith_value(&lie_type, span)
        }
        "filter_units" => {
            let (basis, factors) = relation_pair(name, arguments, span)?;
            let (basis, factors) = filter_relation_units_adapter(basis, factors, span)?;
            Ok(Value::Tuple(vec![
                relation_value(&basis, span)?,
                Value::Vector(Vec32(factors)),
            ]))
        }
        "ann_mod" => {
            arity(name, arguments, 2, span)?;
            let Value::Matrix(matrix) = &arguments[0] else {
                return Err(type_error(span, "ann_mod expects a mat"));
            };
            let denominator = narrow_ann_modulus(&arguments[1], span)?;
            let denominator = NonZeroI32::new(denominator)
                .ok_or_else(|| runtime(span, "ann_mod modulus must be nonzero"))?;
            let matrix = relation_matrix(matrix, span)?;
            let annihilator = relation_annihilator_modulo(&matrix, denominator, &INTEGER_BUDGET)
                .map_err(|error| relation_diagnostic(error, span))?;
            relation_value(&annihilator, span)
        }
        "replace_gen" => {
            arity(name, arguments, 2, span)?;
            let (basis, factors) = relation_pair(name, std::slice::from_ref(&arguments[0]), span)?;
            let Value::Matrix(replacements) = &arguments[1] else {
                return Err(type_error(span, "replace_gen expects replacement columns"));
            };
            let result = replace_relation_generators_adapter(basis, factors, replacements, span)?;
            relation_value(&result, span)
        }
        "quotient_basis" => {
            arity(name, arguments, 2, span)?;
            let lie_type = as_lie_type(&arguments[0], span)?;
            let Value::List(generators) = &arguments[1] else {
                return Err(type_error(span, "quotient_basis expects a row of ratvec"));
            };
            quotient_relation_basis_adapter(&lie_type, generators, span)
        }
        "prefers_coroots" => {
            arity(name, arguments, 1, span)?;
            let Value::Domain(DomainValue::RootDatum(handle)) = &arguments[0] else {
                return Err(type_error(span, "expected a RootDatum"));
            };
            Ok(Value::Boolean(handle.prefers_coroots()))
        }
        "simply_connected" | "adjoint" => {
            let prefers_coroots = datum_preference(name, arguments, span)?;
            let lie_type = as_lie_type(&arguments[0], span)?;
            let semisimple_rank: usize = lie_type.semisimple_factors().map(|(_, rank)| rank).sum();
            if name == "adjoint" && semisimple_rank != lie_type.total_rank() {
                let rank = lie_type.total_rank();
                return Err(runtime(
                    span,
                    format!("Sub-lattice matrix should have size {rank}x{rank}"),
                ));
            }
            let handle = build_datum(&lie_type, name == "simply_connected", prefers_coroots, span)?;
            Ok(Value::Domain(DomainValue::RootDatum(handle)))
        }
        "root_datum" => match arguments {
            [Value::Domain(DomainValue::LieType(lie_type)), lattice, Value::Boolean(prefers_coroots)] =>
            {
                let lattice = as_matrix_rows(lattice, span)?;
                let handle = build_quotient_datum(lie_type, &lattice, *prefers_coroots, span)?;
                Ok(Value::Domain(DomainValue::RootDatum(handle)))
            }
            [simple_roots, simple_coroots, Value::Boolean(prefers_coroots)] => {
                let simple_roots = as_matrix_rows(simple_roots, span)?;
                let simple_coroots = as_matrix_rows(simple_coroots, span)?;
                let handle =
                    build_explicit_datum(&simple_roots, &simple_coroots, *prefers_coroots, span)?;
                Ok(Value::Domain(DomainValue::RootDatum(handle)))
            }
            [Value::Domain(DomainValue::RootDatum(source)), lattice] => {
                let lattice = as_matrix_rows(lattice, span)?;
                let rank = source.datum.lattice_rank();
                if lattice.len() != rank || lattice.iter().any(|row| row.len() != rank) {
                    return Err(runtime(
                        span,
                        format!("Sub-lattice matrix should have size {rank}x{rank}"),
                    ));
                }
                let inverse = invert_integer_matrix(&lattice)
                    .ok_or_else(|| runtime(span, "Dependent lattice generators"))?;
                let handle = build_quotient_from_handle(source, &lattice, inverse, span)?;
                Ok(Value::Domain(DomainValue::RootDatum(handle)))
            }
            [Value::Domain(DomainValue::InnerClass(context))] => Ok(Value::Domain(
                DomainValue::RootDatum(context.root_datum.clone()),
            )),
            [Value::Domain(DomainValue::WeylElement(value))] => Ok(Value::Domain(
                DomainValue::RootDatum(value.context.handle.clone()),
            )),
            _ => Err(type_error(
                span,
                format!(
                    "{name} has no matching overload for {} argument(s)",
                    arguments.len()
                ),
            )),
        },
        "Cartan_matrix" => {
            arity(name, arguments, 1, span)?;
            match &arguments[0] {
                Value::Domain(DomainValue::LieType(lie_type)) => {
                    matrix_value(&block_cartan(lie_type), span)
                }
                Value::Domain(DomainValue::RootDatum(handle)) => {
                    matrix_value(handle.datum.cartan_matrix(), span)
                }
                other => Err(type_error(
                    span,
                    format!("expected a LieType or RootDatum, found {other}"),
                )),
            }
        }
        "nr_of_posroots" => {
            arity(name, arguments, 1, span)?;
            let handle = as_root_datum(&arguments[0], span)?;
            let table = RootTable::build(handle, span)?;
            Ok(Value::Integer(BigInt::from(table.roots.len())))
        }
        // positive_roots_wrapper / positive_coroots_wrapper
        // (atlas-types.w:1656-1671): the positive (co)root table as a
        // by-columns matrix, in the oracle's presentation order.
        "posroots" | "poscoroots" => {
            arity(name, arguments, 1, span)?;
            let handle = as_root_datum(&arguments[0], span)?;
            let table = RootTable::build(handle, span)?;
            let columns = if name == "poscoroots" {
                &table.coroots
            } else {
                &table.roots
            };
            columns_matrix_value(columns, handle.datum.lattice_rank(), span)
        }
        // simple_roots/simple_coroots (atlas-types.w:1638-1658): the
        // oracle prints the simple (co)roots as matrix COLUMNS
        // (rootdata.cpp:1442-1455 posroots shape).
        "simple_roots" | "simple_coroots" => {
            arity(name, arguments, 1, span)?;
            let handle = as_root_datum(&arguments[0], span)?;
            let columns: Vec<Vec<i32>> = if name == "simple_coroots" {
                handle
                    .datum
                    .simple_coroots()
                    .iter()
                    .map(|coweight| coweight.as_slice().to_vec())
                    .collect()
            } else {
                handle
                    .datum
                    .simple_roots()
                    .iter()
                    .map(|weight| weight.as_slice().to_vec())
                    .collect()
            };
            matrix_value(&transpose_matrix(&columns), span)
        }
        // root_coradical / coroot_radical (atlas-types.w:2254-2255): the
        // simple roots/coroots followed by a basis of the kernel of the
        // coroots/roots (the coradical/radical). root_coradical prints its
        // vectors as matrix rows; coroot_radical as matrix columns.
        "root_coradical" | "coroot_radical" => {
            arity(name, arguments, 1, span)?;
            let handle = as_root_datum(&arguments[0], span)?;
            if name == "coroot_radical" {
                let mut columns: Vec<Vec<i32>> = handle
                    .datum
                    .simple_coroots()
                    .iter()
                    .map(|coweight| coweight.as_slice().to_vec())
                    .collect();
                let extra: Vec<Vec<i32>> = handle
                    .datum
                    .radical_basis()
                    .map_err(|error| structure_diagnostic(error, span))?
                    .iter()
                    .map(|coweight| coweight.as_slice().to_vec())
                    .collect();
                columns.extend(extra);
                columns_matrix_value(&columns, handle.datum.lattice_rank(), span)
            } else {
                let mut rows: Vec<Vec<i32>> = handle
                    .datum
                    .simple_roots()
                    .iter()
                    .map(|weight| weight.as_slice().to_vec())
                    .collect();
                let extra: Vec<Vec<i32>> = handle
                    .datum
                    .coradical_basis()
                    .map_err(|error| structure_diagnostic(error, span))?
                    .iter()
                    .map(|weight| weight.as_slice().to_vec())
                    .collect();
                rows.extend(extra);
                matrix_value(&rows, span)
            }
        }
        // is_Cartan_matrix (atlas-types.w:368-375): the matrix is a Cartan
        // matrix iff its Dynkin classification succeeds.
        "is_Cartan_matrix" => {
            arity(name, arguments, 1, span)?;
            let Value::Matrix(matrix) = &arguments[0] else {
                return Err(type_error(
                    span,
                    format!(
                        "{name} has no matching overload for {} argument(s)",
                        arguments.len()
                    ),
                ));
            };
            let rows = matrix_rows(matrix);
            let recognized = infer_lie_type(&rows, rows.len(), span).is_ok();
            Ok(Value::Boolean(recognized))
        }
        "rank" => {
            arity(name, arguments, 1, span)?;
            match &arguments[0] {
                Value::Domain(DomainValue::RootDatum(handle)) => {
                    Ok(Value::Integer(BigInt::from(handle.datum.lattice_rank())))
                }
                Value::Domain(DomainValue::LieType(lie)) => Ok(Value::Integer(BigInt::from(
                    lie.factors.iter().map(|(_, rank)| *rank).sum::<usize>(),
                ))),
                _ => Err(type_error(span, "expected a RootDatum or LieType")),
            }
        }
        // semisimple_rank (atlas-types.w:1397-1400): the number of simple
        // roots.
        "semisimple_rank" => {
            arity(name, arguments, 1, span)?;
            let handle = as_root_datum(&arguments[0], span)?;
            Ok(Value::Integer(BigInt::from(handle.datum.semisimple_rank())))
        }
        "root" | "coroot" => {
            arity(name, arguments, 2, span)?;
            let handle = as_root_datum(&arguments[0], span)?;
            let index = as_integer(&arguments[1], span)?;
            root_query(handle, &index, name == "coroot", span)
        }
        "is_long_root" | "is_long_coroot" => {
            arity(name, arguments, 2, span)?;
            let handle = as_root_datum(&arguments[0], span)?;
            let index = as_integer(&arguments[1], span)?;
            length_query(handle, &index, name == "is_long_coroot", span)
        }
        // root_expression/coroot_expression (atlas-types.w:1487-1504):
        // root_expr/coroot_expr — the simple (co)root coordinates of the
        // (co)root with the given signed number.
        "root_expression" | "coroot_expression" => {
            arity(name, arguments, 2, span)?;
            let handle = as_root_datum(&arguments[0], span)?;
            let index = as_integer(&arguments[1], span)?;
            let coroot = name == "coroot_expression";
            let (system, numbering) = signed_roots(handle, span)?;
            let id = numbering.id(internal_root_nbr(&index, &numbering, coroot, span)?);
            let coordinates = if coroot {
                simple_coroot_coordinates(&system, id)
            } else {
                system.simple_coordinates(id).map(<[i32]>::to_vec)
            }
            .ok_or_else(|| runtime(span, "missing simple coordinates"))?;
            Ok(Value::Vector(Vec32(coordinates)))
        }
        // root_index/coroot_index (atlas-types.w:1505-1518): find_index
        // over the datum's (co)root list in its native lattice basis,
        // then convert_to_signed_root_index; a miss yields numPosRoots.
        "root_index" | "coroot_index" => {
            arity(name, arguments, 2, span)?;
            let handle = as_root_datum(&arguments[0], span)?;
            let coordinates = as_weight_coordinates(&arguments[1], span)?;
            let table = RootTable::build(handle, span)?;
            let vectors = if name == "coroot_index" {
                &table.coroots
            } else {
                &table.roots
            };
            let npos = vectors.len();
            let signed = if let Some(positive) = vectors.iter().position(|v| *v == coordinates) {
                i64::try_from(positive).map_err(|_| runtime(span, "root index overflow"))?
            } else {
                let negated: Vec<i32> = coordinates.iter().map(|entry| -entry).collect();
                match vectors.iter().position(|v| *v == negated) {
                    Some(positive) => {
                        -1 - i64::try_from(positive)
                            .map_err(|_| runtime(span, "root index overflow"))?
                    }
                    None => {
                        i64::try_from(npos).map_err(|_| runtime(span, "root index overflow"))?
                    }
                }
            };
            Ok(Value::Integer(BigInt::from(signed)))
        }
        // root_involution (atlas-types.w:1519-1526): the reflection in
        // |alpha| as a permutation of all roots, in internal RootNbr order.
        "root_involution" => {
            arity(name, arguments, 2, span)?;
            let handle = as_root_datum(&arguments[0], span)?;
            let index = as_integer(&arguments[1], span)?;
            let (system, numbering) = signed_roots(handle, span)?;
            // rt_abs: a root and its negative define the same reflection.
            let (positive, _) = positive_slot(&index, numbering.npos, false, span)?;
            let alpha = numbering.id(numbering.npos + positive);
            let action = WeylAction::root_reflection(&handle.datum, &system, alpha)
                .map_err(|error| runtime(span, error.to_string()))?;
            weyl_root_permutation(&system, &numbering, &action, span)
        }
        // root_ladder_bottoms/coroot_ladder_bottoms wrappers
        // (atlas-types.w:1569-1597): min_roots_for/min_coroots_for — the
        // (co)roots beta for which beta-alpha is not a (co)root, alpha
        // included — as signed root numbers in ascending internal order.
        "root_ladder_bottoms" | "coroot_ladder_bottoms" => {
            arity(name, arguments, 2, span)?;
            let handle = as_root_datum(&arguments[0], span)?;
            let index = as_integer(&arguments[1], span)?;
            let coroot = name == "coroot_ladder_bottoms";
            let (system, numbering) = signed_roots(handle, span)?;
            let id = numbering.id(internal_root_nbr(&index, &numbering, coroot, span)?);
            let bottoms = if coroot {
                system.min_coroots_for(id)
            } else {
                system.min_roots_for(id)
            }
            .ok_or_else(|| runtime(span, "missing ladder bottom table"))?;
            let mut nbrs: Vec<usize> = bottoms.iter().map(|id| numbering.nbr(id)).collect();
            nbrs.sort_unstable();
            Ok(Value::List(
                nbrs.into_iter()
                    .map(|nbr| Value::Integer(BigInt::from(numbering.signed(nbr))))
                    .collect(),
            ))
        }
        // root_permutation (atlas-types.w:2604-2618): the images of all
        // roots under w, in internal RootNbr order.
        "root_permutation" => {
            arity(name, arguments, 1, span)?;
            let value = as_weyl_elt(&arguments[0], span)?;
            let context = &value.context;
            let datum = &*context.handle.datum;
            let mut action =
                WeylAction::identity(datum).map_err(|error| runtime(span, error.to_string()))?;
            for &generator in &value.word {
                let reflection = WeylAction::simple_reflection(datum, generator)
                    .map_err(|error| runtime(span, error.to_string()))?;
                action = action
                    .compose(&reflection)
                    .map_err(|error| runtime(span, error.to_string()))?;
            }
            let numbering = RootNumbering::new(&context.system, context.handle.prefers_coroots());
            weyl_root_permutation(&context.system, &numbering, &action, span)
        }
        "inner_class" => match arguments {
            [Value::Domain(DomainValue::RealForm(context))] => Ok(Value::Domain(
                DomainValue::InnerClass(Arc::clone(&context.parent)),
            )),
            [Value::Domain(DomainValue::RootDatum(handle)), matrix] => {
                let matrix = as_matrix(matrix, span)?;
                let context = build_inner_class(handle, matrix, span)?;
                Ok(Value::Domain(DomainValue::InnerClass(context)))
            }
            _ => Err(type_error(
                span,
                "inner_class expects a RealForm or (RootDatum,mat)",
            )),
        },
        "classify_involution" => {
            arity(name, arguments, 1, span)?;
            let matrix = checked_involution_matrix(&arguments[0], None, span)?;
            let classification = domain_classify_involution(&matrix, &INTEGER_BUDGET).map_err(
                |error| match error {
                    StructureError::InvalidInvolution => {
                        runtime(span, "Given transformation is not an involution")
                    }
                    other => runtime(span, other.to_string()),
                },
            )?;
            let (compact, complex, split) = classification.as_tuple();
            Ok(Value::Tuple(vec![
                Value::Integer(BigInt::from(compact)),
                Value::Integer(BigInt::from(complex)),
                Value::Integer(BigInt::from(split)),
            ]))
        }
        "twisted_involution" => {
            arity(name, arguments, 2, span)?;
            let handle = as_root_datum(&arguments[0], span)?;
            let (factor, inner_class) = build_twisted_involution(handle, &arguments[1], span)?;
            let weyl_context = build_weyl_context(handle, span)?;
            let factor = weyl_elt_value(weyl_context, factor, span)?;
            Ok(Value::Tuple(vec![
                factor,
                Value::Domain(DomainValue::InnerClass(inner_class)),
            ]))
        }
        "distinguished_involution" => {
            arity(name, arguments, 1, span)?;
            let Value::Domain(DomainValue::InnerClass(context)) = &arguments[0] else {
                return Err(type_error(span, "expected an InnerClass"));
            };
            matrix_value(
                context
                    .inner_class
                    .distinguished_involution()
                    .involution()
                    .weight_matrix(),
                span,
            )
        }
        "nr_of_real_forms" => {
            arity(name, arguments, 1, span)?;
            let Value::Domain(DomainValue::InnerClass(context)) = &arguments[0] else {
                return Err(type_error(span, "expected an InnerClass"));
            };
            Ok(Value::Integer(BigInt::from(context.order.form_count())))
        }
        "nr_of_dual_real_forms" => {
            arity(name, arguments, 1, span)?;
            let Value::Domain(DomainValue::InnerClass(context)) = &arguments[0] else {
                return Err(type_error(span, "expected an InnerClass"));
            };
            Ok(Value::Integer(BigInt::from(context.dual_form_count)))
        }
        "form_names" => {
            arity(name, arguments, 1, span)?;
            let Value::Domain(DomainValue::InnerClass(context)) = &arguments[0] else {
                return Err(type_error(span, "expected an InnerClass"));
            };
            Ok(Value::List(
                context
                    .forms
                    .iter()
                    .map(|form| Value::String(form.name.clone()))
                    .collect(),
            ))
        }
        "dual_form_names" => {
            arity(name, arguments, 1, span)?;
            let Value::Domain(DomainValue::InnerClass(context)) = &arguments[0] else {
                return Err(type_error(span, "expected an InnerClass"));
            };
            let dual = build_dual_inner_class(context, span)?;
            Ok(Value::List(
                dual.forms
                    .iter()
                    .map(|form| Value::String(form.name.clone()))
                    .collect(),
            ))
        }
        // real_form_wrapper (InnerClass,int) and
        // synthetic_real_form_wrapper (InnerClass,mat,ratvec), dispatched
        // by argument count.
        "real_form" => {
            // The four projection wrappers (atlas-types.w:5279-5287,
            // 5543-5548, 6269-6275, 7621-7626) return the owning form.
            match arguments {
                [Value::Domain(DomainValue::KType(ktype))] => Ok(Value::Domain(
                    DomainValue::RealForm(Arc::clone(&ktype.context)),
                )),
                [Value::Domain(DomainValue::KTypePol(pol))] => {
                    Ok(Value::Domain(DomainValue::RealForm(Arc::clone(&pol.rf))))
                }
                [Value::Domain(DomainValue::Param(parameter))] => Ok(Value::Domain(
                    DomainValue::RealForm(Arc::clone(&parameter.context)),
                )),
                [Value::Domain(DomainValue::ParamPol(pol))] => {
                    Ok(Value::Domain(DomainValue::RealForm(Arc::clone(&pol.rf))))
                }
                arguments => {
                    let Some(Value::Domain(DomainValue::InnerClass(context))) = arguments.first()
                    else {
                        return Err(type_error(span, "expected an InnerClass"));
                    };
                    match arguments.len() {
                        2 => {
                            let external = as_usize(&arguments[1], span)?;
                            let form = build_real_form(context, external, span)?;
                            Ok(Value::Domain(DomainValue::RealForm(form)))
                        }
                        3 => {
                            let plan =
                                synthetic_real_form(context, &arguments[1], &arguments[2], span)?;
                            let form = if plan.default_seed {
                                build_real_form(context, plan.external, span)?
                            } else {
                                build_custom_real_form(context, &plan, span)?
                            };
                            Ok(Value::Domain(DomainValue::RealForm(form)))
                        }
                        count => Err(type_error(
                            span,
                            format!("real_form expects 1, 2, or 3 argument(s), found {count}"),
                        )),
                    }
                }
            }
        }
        // dual_datum_wrapper (atlas-types.w:1713-1717),
        // dual_inner_class_wrapper (atlas-types.w:3254-3258), and
        // dual_block_wrapper (atlas-types.w:4882-4886): datum/inner-class
        // duality, or the block rebuilt on the swapped forms.
        "dual" => {
            arity(name, arguments, 1, span)?;
            match &arguments[0] {
                Value::Domain(DomainValue::RootDatum(handle)) => {
                    let dual = dual_root_datum(handle, span)?;
                    Ok(Value::Domain(DomainValue::RootDatum(dual)))
                }
                Value::Domain(DomainValue::InnerClass(context)) => {
                    let dual = build_dual_inner_class(context, span)?;
                    Ok(Value::Domain(DomainValue::InnerClass(dual)))
                }
                Value::Domain(DomainValue::Block(block)) => {
                    // dual_block_wrapper: Block_value(dual_rf, rf) — a fresh
                    // fibred product on the swapped contexts.
                    let dual = build_block(&block.dual_rf, &block.rf, span)?;
                    Ok(Value::Domain(DomainValue::Block(dual)))
                }
                other => Err(type_error(
                    span,
                    format!("expected a RootDatum, InnerClass, or Block, found {other}"),
                )),
            }
        }
        // dual_datum_of_inner_class_wrapper (atlas-types.w:3247-3251,
        // 3412-3413): the dual inner class's own root datum.
        // two_rho / two_rho_check (atlas-types.w:1409-1421): the sum of
        // the positive roots (respectively positive coroots).
        "fundamental_weight" => {
            arity(name, arguments, 2, span)?;
            let handle = as_root_datum(&arguments[0], span)?;
            let index = as_usize(&arguments[1], span)?;
            if index >= handle.datum.semisimple_rank() {
                return Err(runtime(span, "index out of range"));
            }
            let mut numerator = vec![0_i64; handle.datum.lattice_rank()];
            numerator[index] = 1;
            let value = RatVec::new(numerator, 1)
                .ok_or_else(|| runtime(span, "invalid fundamental weight"))?;
            Ok(Value::RatVector(value))
        }
        "fundamental_coweight" => {
            arity(name, arguments, 2, span)?;
            let handle = as_root_datum(&arguments[0], span)?;
            let index = as_usize(&arguments[1], span)?;
            if index >= handle.datum.semisimple_rank() {
                return Err(runtime(span, "index out of range"));
            }
            // C^{-1} column i via Cramer: solve C x = e_i.
            let cartan = handle.datum.cartan_matrix();
            let mut rhs = vec![0_i32; cartan.len()];
            rhs[index] = 1;
            let (mut numerator, denominator) = cramer_solution(cartan, &rhs)
                .ok_or_else(|| runtime(span, "singular Cartan matrix"))?;
            numerator.resize(handle.datum.lattice_rank(), 0);
            let value = RatVec::new(numerator, denominator.unsigned_abs())
                .ok_or_else(|| runtime(span, "invalid fundamental coweight"))?;
            Ok(Value::RatVector(value))
        }
        "simple_factors" => {
            arity(name, arguments, 1, span)?;
            let lie = as_lie_type(&arguments[0], span)?;
            let factors = lie
                .factors
                .into_iter()
                .map(|(letter, rank)| {
                    Value::Tuple(vec![
                        Value::String(letter.to_string()),
                        Value::Integer(rank.into()),
                    ])
                })
                .collect();
            Ok(Value::List(factors))
        }
        "Cartan_matrix_type" => {
            arity(name, arguments, 1, span)?;
            let matrix = as_matrix(&arguments[0], span)?;
            let lie_type = infer_lie_type(&matrix, matrix.len(), span)?;
            let permutation: Vec<i64> = (0..matrix.len()).map(|index| index as i64).collect();
            Ok(Value::Tuple(vec![
                Value::Domain(DomainValue::LieType(lie_type)),
                Value::List(
                    permutation
                        .into_iter()
                        .map(|index| Value::Integer(index.into()))
                        .collect(),
                ),
            ]))
        }
        // walls_wrapper (atlas-types.w:1912-1943): the size check runs
        // before the no-value gate upstream, so validation stays eager.
        "walls" => {
            arity(name, arguments, 2, span)?;
            let handle = as_root_datum(&arguments[0], span)?;
            let Value::RatVector(gamma) = &arguments[1] else {
                return Err(type_error(span, "expected a rational vector"));
            };
            let rank = handle.datum.lattice_rank();
            if gamma.numerators().len() != rank {
                return Err(runtime(
                    span,
                    format!(
                        "Rational weight size mismatch: {}:{}",
                        gamma.numerators().len(),
                        rank
                    ),
                ));
            }
            let root_system = RootSystem::enumerate(&handle.datum, ROOT_BUDGET)
                .map_err(|error| runtime(span, error.to_string()))?;
            let numbering = RootNumbering::new(&root_system, handle.prefers_coroots());
            let (walls, integrals) = wall_set(&root_system, &numbering, gamma);
            // Output: integral walls first, then non-integral, each in
            // sorted_by_label order (atlas-types.w:1928-1938).
            let sorted = sorted_by_label(&root_system, &numbering, &walls)
                .map_err(|error| runtime(span, error))?;
            let ordered = sorted
                .iter()
                .filter(|&alpha| integrals.contains(alpha))
                .chain(sorted.iter().filter(|&alpha| !integrals.contains(alpha)));
            Ok(Value::Tuple(vec![
                Value::List(
                    ordered
                        .map(|&alpha| Value::Integer(BigInt::from(numbering.signed(alpha))))
                        .collect(),
                ),
                Value::Integer(BigInt::from(integrals.len())),
            ]))
        }
        // walls_attitude_wrapper (atlas-types.w:1960-1989): the acute-angle
        // and size checks run before the no-value gate upstream.
        "walls_attitude" => {
            arity(name, arguments, 2, span)?;
            let handle = as_root_datum(&arguments[0], span)?;
            let Value::List(wall_entries) = &arguments[1] else {
                return Err(type_error(span, "expected a row of integers"));
            };
            let root_system = RootSystem::enumerate(&handle.datum, ROOT_BUDGET)
                .map_err(|error| runtime(span, error.to_string()))?;
            let numbering = RootNumbering::new(&root_system, handle.prefers_coroots());
            let num_roots = root_system.roots().len();
            let mut walls = BTreeSet::new();
            for entry in wall_entries {
                let index = as_integer(entry, span)?
                    .to_string()
                    .parse::<i64>()
                    .unwrap_or(i64::MAX);
                // internal_root_index (atlas-types.w:1428-1439).
                let nbr = index + numbering.npos as i64;
                if nbr < 0 || nbr >= num_roots as i64 {
                    return Err(runtime(span, format!("Illegal root index {index}")));
                }
                walls.insert(nbr as usize);
            }
            let ordered: Vec<usize> = walls.iter().copied().collect();
            for (position, &alpha) in ordered.iter().enumerate() {
                for &beta in &ordered[position + 1..] {
                    let bracket = root_system
                        .bracket(numbering.id(alpha), numbering.id(beta))
                        .map_err(|error| runtime(span, error.to_string()))?;
                    if bracket > 0 {
                        return Err(runtime(
                            span,
                            format!(
                                "Roots set involves roots with acute angle: {} and {}",
                                numbering.signed(alpha),
                                numbering.signed(beta)
                            ),
                        ));
                    }
                }
            }
            let minimum = fundamental_alcove_wall_count(&handle.datum);
            if walls.len() < minimum {
                return Err(runtime(
                    span,
                    format!("Too few walls: {} < {}", walls.len(), minimum),
                ));
            }
            let word = from_fundamental_alcove(&root_system, &numbering, &walls)
                .map_err(|error| runtime(span, error))?;
            let context = build_weyl_context(handle, span)?;
            let mut element = WeylElement::identity(&context.system)
                .map_err(|error| runtime(span, error.to_string()))?;
            for generator in word {
                let (next, _) = element
                    .right_multiply_simple(&context.system, generator)
                    .map_err(|error| runtime(span, error.to_string()))?;
                element = next;
            }
            weyl_elt_value(context, element, span)
        }
        // Weyl_orbit / Weyl_orbit_ws wrappers (atlas-types.w:1840-1888):
        // the (RootDatum,vec) order acts on weights, (vec,RootDatum) on
        // coweights. Upstream performs no size validation; coordinates
        // past the end read as zero and extras are dropped.
        "Weyl_orbit" | "Weyl_orbit_ws" => {
            arity(name, arguments, 2, span)?;
            let dual = !matches!(&arguments[0], Value::Domain(DomainValue::RootDatum(_)));
            let (handle, coordinates) = if dual {
                (
                    as_root_datum(&arguments[1], span)?,
                    as_weight_coordinates(&arguments[0], span)?,
                )
            } else {
                (
                    as_root_datum(&arguments[0], span)?,
                    as_weight_coordinates(&arguments[1], span)?,
                )
            };
            let datum = &handle.datum;
            let rank = datum.lattice_rank();
            let semisimple = datum.semisimple_rank();
            let mut weight = coordinates;
            weight.resize(rank, 0);
            // Weyl_orbit / Weyl_orbit_words (rootdata.cpp:1810-1876): move
            // to the (co)dominant chamber, then extend over the
            // non-stabilising generators in ascending order.
            let mut word = if dual {
                factor_codominant_word(datum, &mut weight)
            } else {
                factor_dominant_word(datum, &mut weight)
            };
            let mut stab = vec![false; semisimple];
            for (s, fixed) in stab.iter_mut().enumerate() {
                let pairing = if dual {
                    dot_i32(&weight, datum.simple_roots()[s].as_slice())
                } else {
                    dot_i32(&weight, datum.simple_coroots()[s].as_slice())
                };
                *fixed = pairing == 0;
            }
            let inverse = inverse_cartan(datum.cartan_matrix());
            let non_stab: Vec<usize> = stab
                .iter()
                .enumerate()
                .filter_map(|(s, &fixed)| (!fixed).then_some(s))
                .collect();
            if name == "Weyl_orbit" {
                let mut orbit = vec![weight];
                for s in non_stab {
                    extend_orbit_weights(datum, &inverse, dual, &mut orbit, &mut stab, s);
                }
                let data: Vec<i32> = orbit.iter().flatten().copied().collect();
                let matrix = Matrix::from_columns(rank, orbit.len(), data)
                    .expect("orbit columns share the rank");
                Ok(Value::Matrix(matrix))
            } else {
                word.reverse(); // need the word moving TO, not from, (co)dominant
                let mut orbit = vec![word];
                for s in non_stab {
                    extend_orbit_words(datum, &inverse, dual, &mut orbit, &mut stab, s);
                }
                let context = build_weyl_context(handle, span)?;
                let mut result = Vec::with_capacity(orbit.len());
                for word in orbit {
                    let mut element = WeylElement::identity(&context.system)
                        .map_err(|error| runtime(span, error.to_string()))?;
                    for generator in word {
                        let (next, _) = element
                            .right_multiply_simple(&context.system, generator)
                            .map_err(|error| runtime(span, error.to_string()))?;
                        element = next;
                    }
                    result.push(weyl_elt_value(Arc::clone(&context), element, span)?);
                }
                Ok(Value::List(result))
            }
        }
        // basic_orbit_ws_wrapper (atlas-types.w:2014-2041): the size and
        // acute-angle checks run before the no-value gate upstream.
        "basic_orbit_ws" => {
            arity(name, arguments, 3, span)?;
            let handle = as_root_datum(&arguments[0], span)?;
            let Value::List(entries) = &arguments[1] else {
                return Err(type_error(span, "expected a row of integers"));
            };
            let stab_rank = as_usize(&arguments[2], span)?;
            if entries.len() <= stab_rank {
                return Err(runtime(
                    span,
                    "Index too large for given list of root numbers",
                ));
            }
            let (system, numbering) = signed_roots(handle, span)?;
            // internal_root_index per entry (atlas-types.w:2068-2075).
            let mut walls = Vec::with_capacity(entries.len());
            for entry in entries {
                walls.push(internal_root_nbr(
                    &as_integer(entry, span)?,
                    &numbering,
                    false,
                    span,
                )?);
            }
            // The acute-angle sweep (:2077-2090) covers the full list,
            // final root included; equal values at distinct positions
            // count as acute.
            for (left, &alpha) in walls.iter().enumerate() {
                for (right, &beta) in walls.iter().enumerate() {
                    if left == right {
                        continue;
                    }
                    let bracket = system
                        .bracket(numbering.id(alpha), numbering.id(beta))
                        .map_err(|error| runtime(span, error.to_string()))?;
                    if bracket > 0 {
                        return Err(runtime(
                            span,
                            format!(
                                "Roots {} and {} have acute angle.",
                                numbering.signed(alpha),
                                numbering.signed(beta)
                            ),
                        ));
                    }
                }
            }
            let mut stab: BTreeSet<usize> = walls[..stab_rank].iter().copied().collect();
            let final_root = walls[stab_rank];
            // to_affine_orbit (:2100-2116): intersect stab with the
            // Dynkin component of final; a singular coroot Cartan
            // submatrix (a dependency among the component's coroots)
            // selects the affine branch.
            let mut subset = stab.clone();
            subset.insert(final_root);
            let mut affine = false;
            for comp in root_components(&system, &numbering, &subset) {
                if comp.contains(&final_root) {
                    let comp_set: BTreeSet<usize> = comp.iter().copied().collect();
                    stab = stab.intersection(&comp_set).copied().collect();
                    let (_, det) = adjugate_det(&cartan_of_roots(&system, &numbering, &comp));
                    affine = det == 0;
                    break;
                }
            }
            let words = if affine {
                complete_affine_component(&system, &numbering, &stab, final_root)
                    .map_err(|error| runtime(span, error))?
            } else {
                finite_subquotient(&system, &numbering, &stab, final_root)
            };
            weyl_word_values(handle, words, span)
        }
        // affine_orbit_ws_wrapper (atlas-types.w:2043-2063): the size
        // check precedes the no-value gate.
        "affine_orbit_ws" => {
            arity(name, arguments, 2, span)?;
            let handle = as_root_datum(&arguments[0], span)?;
            let Value::RatVector(gamma) = &arguments[1] else {
                return Err(type_error(span, "expected a rational vector"));
            };
            let rank = handle.datum.lattice_rank();
            if gamma.numerators().len() != rank {
                return Err(runtime(
                    span,
                    format!(
                        "Rank and rational weight size mismatch {}:{}",
                        rank,
                        gamma.numerators().len()
                    ),
                ));
            }
            let (system, numbering) = signed_roots(handle, span)?;
            let (walls, stabiliser) = wall_set(&system, &numbering, gamma);
            // affine_orbit_ws (alcoves.cpp:725-738): extend over every
            // wall component not wholly stabilised.
            let mut words = vec![Vec::new()];
            for comp in root_components(&system, &numbering, &walls) {
                let comp_set: BTreeSet<usize> = comp.into_iter().collect();
                if !comp_set.is_subset(&stabiliser) {
                    extend_affine_component(
                        &system,
                        &numbering,
                        &mut words,
                        &comp_set,
                        &stabiliser,
                    )
                    .map_err(|error| runtime(span, error))?;
                }
            }
            weyl_word_values(handle, words, span)
        }
        // alcove_root_vertex_wrapper (atlas-types.w:1994-2011): the
        // unique root-lattice vertex of the alcove containing `gamma`.
        "alcove_root_vertex" => {
            arity(name, arguments, 2, span)?;
            let handle = as_root_datum(&arguments[0], span)?;
            let Value::RatVector(gamma) = &arguments[1] else {
                return Err(type_error(span, "expected a rational vector"));
            };
            let rank = handle.datum.lattice_rank();
            if gamma.numerators().len() != rank {
                return Err(runtime(
                    span,
                    format!(
                        "Rational weight size mismatch: {}:{}",
                        gamma.numerators().len(),
                        rank
                    ),
                ));
            }
            let root_system = RootSystem::enumerate(&handle.datum, ROOT_BUDGET)
                .map_err(|error| runtime(span, error.to_string()))?;
            let numbering = RootNumbering::new(&root_system, handle.prefers_coroots());
            let (walls, _integrals) = wall_set(&root_system, &numbering, gamma);
            let mut vertex = vec![0i32; rank];
            for comp in root_components(&root_system, &numbering, &walls) {
                let floors: Vec<i64> = comp
                    .iter()
                    .map(|&nbr| {
                        gamma
                            .numerators()
                            .iter()
                            .zip(
                                root_system
                                    .coroot(numbering.id(nbr))
                                    .expect("every root has a coroot")
                                    .as_slice(),
                            )
                            .map(|(&g, &c)| g * i64::from(c))
                            .sum::<i64>()
                            .div_euclid(gamma.denominator() as i64)
                    })
                    .collect();
                let piece = root_vertex_simple(&root_system, &numbering, &comp, &floors)
                    .map_err(|error| runtime(span, error))?;
                for (slot, value) in vertex.iter_mut().zip(piece) {
                    *slot += value;
                }
            }
            Ok(Value::Vector(Vec32(vertex)))
        }
        // FPP_numers_wrapper/FPP_w_shifts_wrapper (atlas-types.w:2122-2196):
        // both share the size and fundamental-alcove checks before the
        // no-value gate.
        "FPP_numers" | "FPP_w_shifts" => {
            arity(name, arguments, 2, span)?;
            let handle = as_root_datum(&arguments[0], span)?;
            let Value::RatVector(gamma) = &arguments[1] else {
                return Err(type_error(span, "expected a rational vector"));
            };
            let rank = handle.datum.lattice_rank();
            if gamma.numerators().len() != rank {
                return Err(runtime(
                    span,
                    format!(
                        "Rank and rational weight size mismatch {}:{}",
                        rank,
                        gamma.numerators().len()
                    ),
                ));
            }
            let root_system = RootSystem::enumerate(&handle.datum, ROOT_BUDGET)
                .map_err(|error| runtime(span, error.to_string()))?;
            let numbering = RootNumbering::new(&root_system, handle.prefers_coroots());
            let semisimple = handle.datum.semisimple_rank();
            let denominator = gamma.denominator() as i64;
            for &alpha in &fundamental_alcove_walls(&root_system, &numbering) {
                let mut evaluation: i64 = gamma
                    .numerators()
                    .iter()
                    .zip(
                        root_system
                            .coroot(numbering.id(alpha))
                            .expect("every root has a coroot")
                            .as_slice(),
                    )
                    .map(|(&g, &c)| g * i64::from(c))
                    .sum::<i64>();
                if !(numbering.npos..numbering.npos + semisimple).contains(&alpha) {
                    evaluation += denominator;
                }
                if evaluation < 0 {
                    let divisor = gcd(evaluation.abs(), denominator);
                    return Err(runtime(
                        span,
                        format!(
                            "Rational weight is not in fundamental alcove (coroot {}, value {}/{})",
                            numbering.signed(alpha),
                            evaluation / divisor,
                            denominator / divisor
                        ),
                    ));
                }
            }
            let pairs = fpp_w_shifts(&handle.datum, &root_system, &numbering, gamma)
                .map_err(|error| runtime(span, error))?;
            let numer: Vec<i32> = gamma
                .numerators()
                .iter()
                .map(|&entry| entry as i32)
                .collect();
            if name == "FPP_numers" {
                // FPP_orbit_numers (alcoves.cpp:1060-1075).
                let mut result = Vec::new();
                for (word, shifts) in &pairs {
                    if shifts.is_empty() {
                        continue;
                    }
                    let mut image = numer.clone();
                    word_act_weight(&handle.datum, word, &mut image);
                    for shift in shifts {
                        result.push(Value::Vector(Vec32(
                            image
                                .iter()
                                .zip(shift)
                                .map(|(&base, &step)| base + step * denominator as i32)
                                .collect(),
                        )));
                    }
                }
                Ok(Value::List(result))
            } else {
                let context = build_weyl_context(handle, span)?;
                let mut result = Vec::new();
                for (word, shifts) in &pairs {
                    if shifts.is_empty() {
                        continue;
                    }
                    let mut element = WeylElement::identity(&context.system)
                        .map_err(|error| runtime(span, error.to_string()))?;
                    for &generator in word {
                        let (next, _) = element
                            .right_multiply_simple(&context.system, generator)
                            .map_err(|error| runtime(span, error.to_string()))?;
                        element = next;
                    }
                    let weyl_value = weyl_elt_value(Arc::clone(&context), element, span)?;
                    let shift_values: Vec<Value> = shifts
                        .iter()
                        .map(|shift| Value::Vector(Vec32(shift.clone())))
                        .collect();
                    result.push(Value::Tuple(vec![weyl_value, Value::List(shift_values)]));
                }
                Ok(Value::List(result))
            }
        }
        // alcove_center_wrapper (atlas-types.w:1945-1952,
        // alcoves.cpp:277-341): solve for the alcove barycentre keeping
        // the coradical coordinates, then rebuild the parameter there.
        "alcove_center" => {
            arity(name, arguments, 1, span)?;
            let Value::Domain(DomainValue::Param(parameter)) = &arguments[0] else {
                return Err(type_error(
                    span,
                    format!(
                        "{name} has no matching overload for {} argument(s)",
                        arguments.len()
                    ),
                ));
            };
            let rc = rep_context(&parameter.context);
            let centered = domain_alcove_center(&rc, &parameter.repr)
                .map_err(|error| structure_diagnostic(error, span))?;
            Ok(Value::Domain(DomainValue::Param(ParamValue {
                context: Arc::clone(&parameter.context),
                repr: centered,
            })))
        }
        "derived_info" | "mod_central_torus_info" => {
            arity(name, arguments, 1, span)?;
            let handle = as_root_datum(&arguments[0], span)?;
            let datum = &handle.datum;
            let semisimple = datum.semisimple_rank();
            let lattice = datum.lattice_rank();
            let roots_matrix: Vec<Vec<i32>> = datum
                .simple_roots()
                .iter()
                .map(|root| root.as_slice().to_vec())
                .collect();
            let coroots_matrix: Vec<Vec<i32>> = datum
                .simple_coroots()
                .iter()
                .map(|coroot| coroot.as_slice().to_vec())
                .collect();
            let (projector, derived_roots, derived_coroots) = if name == "derived_info" {
                // DerivedTag (prerootdata.cpp:67-82): M = adapted_basis of the
                // simple coroots as COLUMNS (the upstream r x s layout);
                // projector = M^T rows 0..s; section = M^-1 rows 0..s.
                let adapted =
                    adapted_basis(&transpose_matrix_i32(&coroots_matrix), &INTEGER_BUDGET)
                        .map_err(|error| runtime(span, error.to_string()))?;
                let m = integer_matrix_i32(&adapted.basis);
                let minv = integer_matrix_i32(&adapted.inverse);
                let projector = block_rows(&transpose_matrix_i32(&m), semisimple);
                let section = block_rows(&minv, semisimple);
                // simple_roots/projector are stored root-as-row; C++ multiplies
                // the r x s column-root matrices, so use the transposed product.
                let derived_roots = mat_mul_i32(&roots_matrix, &transpose_matrix_i32(&projector))
                    .map_err(|error| runtime(span, error))?;
                let derived_coroots = mat_mul_i32(&coroots_matrix, &transpose_matrix_i32(&section))
                    .map_err(|error| runtime(span, error))?;
                (projector, derived_roots, derived_coroots)
            } else {
                // CoderivedTag (prerootdata.cpp:84-97): M = adapted_basis of
                // the simple roots as COLUMNS (the upstream r x s layout);
                // injector = M block(0,0,r,s); cosection = M^-1 rows 0..s.
                let adapted = adapted_basis(&transpose_matrix_i32(&roots_matrix), &INTEGER_BUDGET)
                    .map_err(|error| runtime(span, error.to_string()))?;
                let m = integer_matrix_i32(&adapted.basis);
                let minv = integer_matrix_i32(&adapted.inverse);
                let injector = block_rows_cols(&m, lattice, semisimple);
                let cosection = block_rows(&minv, semisimple);
                let derived_roots = mat_mul_i32(&roots_matrix, &transpose_matrix_i32(&cosection))
                    .map_err(|error| runtime(span, error))?;
                let derived_coroots = mat_mul_i32(&coroots_matrix, &injector)
                    .map_err(|error| runtime(span, error))?;
                (injector, derived_roots, derived_coroots)
            };
            // The derived datum is semisimple of rank = semisimple_rank.
            let rank = derived_roots.len();
            let cartan: Vec<Vec<i32>> = (0..rank)
                .map(|row| {
                    (0..rank)
                        .map(|column| {
                            derived_roots[row]
                                .iter()
                                .zip(&derived_coroots[column])
                                .map(|(r, c)| r * c)
                                .sum()
                        })
                        .collect()
                })
                .collect();
            let derived_weights: Vec<Weight> = derived_roots
                .iter()
                .map(|row| Weight::new(row.clone()))
                .collect();
            let derived_coweights: Vec<Coweight> = derived_coroots
                .iter()
                .map(|row| Coweight::new(row.clone()))
                .collect();
            let derived =
                BasedRootDatum::from_simple_data(rank, cartan, derived_weights, derived_coweights)
                    .map_err(|error| runtime(span, error.to_string()))?;
            let lie_type = infer_lie_type(derived.cartan_matrix(), rank, span)?;
            // The derived/mod-central-torus datum keeps whatever isogeny its
            // simple (co)roots span (adjoint B2 stays adjoint), so classify
            // from the datum rather than assuming simply connected.
            let isogeny = classify_isogeny(&derived);
            let derived_value = Value::Domain(DomainValue::RootDatum(RootDatumHandle {
                datum: std::sync::Arc::new(derived),
                lie_type,
                isogeny,
                prefers_coroots: false,
            }));
            // `projector` is stored row-major; `Matrix::from_columns` wants
            // column-major data, so flatten the transpose (prerootdata.cpp
            // pushes the projector/injector int_Matrix with its own layout).
            let matrix_rows = projector.len();
            let matrix_cols = projector.first().map_or(0, Vec::len);
            let matrix_value = Value::Matrix(
                Matrix::from_columns(
                    matrix_rows,
                    matrix_cols,
                    transpose_matrix_i32(&projector)
                        .into_iter()
                        .flatten()
                        .collect(),
                )
                .expect("derived projector is rectangular"),
            );
            Ok(Value::Tuple(vec![derived_value, matrix_value]))
        }
        "integrality_rank" => {
            arity(name, arguments, 2, span)?;
            let handle = as_root_datum(&arguments[0], span)?;
            let Value::RatVector(gamma) = &arguments[1] else {
                return Err(type_error(span, "expected a rational vector"));
            };
            let simple = integrality_simples_roots(handle, gamma, span)?;
            Ok(Value::Integer(simple.len().into()))
        }
        "is_integrally_dominant" => {
            arity(name, arguments, 2, span)?;
            let handle = as_root_datum(&arguments[0], span)?;
            let Value::RatVector(gamma) = &arguments[1] else {
                return Err(type_error(span, "expected a rational vector"));
            };
            let root_system = RootSystem::enumerate(&handle.datum, ROOT_BUDGET)
                .map_err(|error| runtime(span, error.to_string()))?;
            let simple = integrality_simples_roots(handle, gamma, span)?;
            for &root in &simple {
                if let Some(dot) = positive_coroot_pairing(&root_system, root, gamma) {
                    if dot < 0 {
                        return Ok(Value::Boolean(false));
                    }
                }
            }
            Ok(Value::Boolean(true))
        }
        "integrality_points" => {
            arity(name, arguments, 2, span)?;
            let handle = as_root_datum(&arguments[0], span)?;
            let Value::RatVector(gamma) = &arguments[1] else {
                return Err(type_error(span, "expected a rational vector"));
            };
            // atlas-types.w:1808-1819: the wrapper rejects a vector whose
            // length differs from the datum rank BEFORE evaluating.
            if gamma.numerators().len() != handle.datum.lattice_rank() {
                return Err(runtime(
                    span,
                    format!(
                        "Length {} of rational vector differs from rank {}",
                        gamma.numerators().len(),
                        handle.datum.lattice_rank()
                    ),
                ));
            }
            let root_system = RootSystem::enumerate(&handle.datum, ROOT_BUDGET)
                .map_err(|error| runtime(span, error.to_string()))?;
            let denominator = gamma.denominator();
            let mut products: std::collections::BTreeSet<i64> = std::collections::BTreeSet::new();
            for index in 0..root_system.roots().len() {
                let id = RootId::from_usize(index);
                if !root_system.is_positive(id).unwrap_or(false) {
                    continue;
                }
                if let Some(dot) = positive_coroot_pairing(&root_system, id, gamma) {
                    if dot != 0 {
                        products.insert(dot.abs());
                    }
                }
            }
            // rootdata.cpp:1508-1527: fracs is a std::set<RatNum>, so the
            // rationals are NORMALISED (2/2 folds into 1/1) and sorted by
            // value. BigRational normalises on construction and orders by
            // value, so a BTreeSet<BigRational> reproduces both behaviours.
            let mut fracs: std::collections::BTreeSet<BigRational> =
                std::collections::BTreeSet::new();
            for &p in &products {
                let mut s = denominator as i64;
                while s <= p {
                    fracs.insert(BigRational::from_integers(BigInt::from(s), BigInt::from(p)));
                    s += denominator as i64;
                }
            }
            let values = fracs.into_iter().map(Value::Rational).collect();
            Ok(Value::List(values))
        }
        "integrality_datum" => {
            arity(name, arguments, 2, span)?;
            let handle = as_root_datum(&arguments[0], span)?;
            let Value::RatVector(gamma) = &arguments[1] else {
                return Err(type_error(span, "expected a rational vector"));
            };
            let simple = integrality_simples_roots(handle, gamma, span)?;
            let root_system = RootSystem::enumerate(&handle.datum, ROOT_BUDGET)
                .map_err(|error| runtime(span, error.to_string()))?;
            // Order the simple roots by their first nonzero datum-simple
            // coordinate (the oracle's simpleBasis RootNbr order), so the
            // subsystem Cartan classifies with the oracle's B/C convention.
            let mut ordered = simple.clone();
            ordered.sort_by_key(|&root| {
                let coordinates = root_system.simple_coordinates(root).unwrap_or_default();
                (
                    coordinates
                        .iter()
                        .position(|&coordinate| coordinate != 0)
                        .unwrap_or(usize::MAX),
                    root.index(),
                )
            });
            let cartan: Vec<Vec<i32>> = ordered
                .iter()
                .map(|&alpha| {
                    ordered
                        .iter()
                        .map(|&beta| root_system.bracket(alpha, beta).unwrap_or(0))
                        .collect()
                })
                .collect();
            // The subsystem lives in the original lattice (the oracle's
            // integrality_datum prints the full lattice, e.g. 'A1.T1' for
            // A2 at a half-integral character).
            let mut simple_weights = Vec::with_capacity(ordered.len());
            let mut simple_coweights = Vec::with_capacity(ordered.len());
            for &id in &ordered {
                let Some(root) = root_system.root(id) else {
                    return Err(runtime(span, "subsystem root missing".to_string()));
                };
                let Some(coroot) = root_system.coroot(id) else {
                    return Err(runtime(span, "subsystem coroot missing".to_string()));
                };
                simple_weights.push(root.clone());
                simple_coweights.push(coroot.clone());
            }
            let datum = BasedRootDatum::from_simple_data(
                handle.datum.lattice_rank(),
                cartan,
                simple_weights,
                simple_coweights,
            )
            .map_err(|error| runtime(span, error.to_string()))?;
            let lie_type = infer_lie_type(datum.cartan_matrix(), datum.lattice_rank(), span)?;
            // The integrality datum keeps the full lattice. With no torus
            // factor it is simply connected (oracle prints "simply
            // connected root datum ..."); with a torus (e.g. 'A1.T1' for
            // A2 at a half-integral character) it is neither.
            let isogeny = if datum.lattice_rank() == datum.semisimple_rank() {
                DatumIsogeny::SimplyConnected
            } else {
                DatumIsogeny::Other
            };
            Ok(Value::Domain(DomainValue::RootDatum(RootDatumHandle {
                datum: std::sync::Arc::new(datum),
                lie_type,
                isogeny,
                prefers_coroots: false,
            })))
        }
        "two_rho" | "two_rho_check" => {
            arity(name, arguments, 1, span)?;
            let Value::Domain(DomainValue::RootDatum(handle)) = &arguments[0] else {
                return Err(type_error(span, "expected a RootDatum"));
            };
            let root_system = RootSystem::enumerate(&handle.datum, ROOT_BUDGET)
                .map_err(|error| runtime(span, error.to_string()))?;
            let positive: Vec<RootId> = (0..root_system.roots().len())
                .filter(|&index| {
                    root_system
                        .is_positive(RootId::from_usize(index))
                        .unwrap_or(false)
                })
                .map(RootId::from_usize)
                .collect();
            let coordinates = if name == "two_rho" {
                two_rho(&root_system, &positive)
            } else {
                let mut sum = vec![0_i32; root_system.lattice_rank()];
                for &root in &positive {
                    if let Some(coroot) = root_system.coroot(root) {
                        for (slot, &coordinate) in sum.iter_mut().zip(coroot.as_slice()) {
                            *slot += coordinate;
                        }
                    }
                }
                Weight::new(sum)
            };
            Ok(Value::Vector(Vec32(coordinates.as_slice().to_vec())))
        }
        "dual_datum" => {
            arity(name, arguments, 1, span)?;
            let Value::Domain(DomainValue::InnerClass(context)) = &arguments[0] else {
                return Err(type_error(span, "expected an InnerClass"));
            };
            let dual = build_dual_inner_class(context, span)?;
            Ok(Value::Domain(DomainValue::RootDatum(
                dual.root_datum.clone(),
            )))
        }
        // Fokko_block_wrapper (atlas-types.w:4786-4796): the is_dual gate,
        // then the fibred product of the two forms' KGB sets. The Param
        // overload (common_block_wrapper, atlas-types.w:6748-6780,
        // repr.cpp:1773-1794 lookup_full_block) returns the survivor
        // parameters of the parameter's common block plus the start index
        // (or -1 when the original parameter is not final).
        "block" => {
            if arguments.len() == 1 {
                let Value::Domain(DomainValue::Param(parameter)) = &arguments[0] else {
                    return Err(type_error(span, "expected a Param"));
                };
                test_standard(parameter, "Cannot generate block", span)?;
                let rc = rep_context(&parameter.context);
                let dominant = parameter
                    .repr
                    .made_dominant(&rc)
                    .map_err(|error| structure_diagnostic(error, span))?;
                match integral_block_scope(&rc, dominant.gamma())
                    .map_err(|error| structure_diagnostic(error, span))?
                {
                    IntegralBlockScope::Singleton => {
                        return Ok(Value::Tuple(vec![
                            Value::List(vec![Value::Domain(DomainValue::Param(ParamValue {
                                context: parameter.context.clone(),
                                repr: dominant,
                            }))]),
                            Value::Integer(BigInt::from(0)),
                        ]));
                    }
                    IntegralBlockScope::ProperSubsystem | IntegralBlockScope::Full => {}
                }
                let located = parameter
                    .context
                    .rep
                    .lookup_full_block(&dominant)
                    .map_err(|error| structure_diagnostic(error, span))?;
                if !located.has_identity_generator_attitude() {
                    return Err(structure_diagnostic(
                        StructureError::NotYetImplemented {
                            feature: "block on a non-identity integral-subsystem attitude",
                        },
                        span,
                    ));
                }
                let block = located.block();
                let common =
                    CommonContext::integral(&rc, located.adapted_representative().gamma_lambda())
                        .map_err(|error| structure_diagnostic(error, span))?;
                let singular_flags = common
                    .singular_flags(located.prepared_query().gamma())
                    .map_err(|error| structure_diagnostic(error, span))?;
                let mut params: Vec<Value> = Vec::new();
                let mut start_pos: i64 = -1;
                for z in 0..block.size() {
                    if block.survives(z, &singular_flags) {
                        if z == located.raw_row() {
                            start_pos = params.len() as i64;
                        }
                        let repr = located_row_parameter(&parameter.context, &located, z)
                            .map_err(|error| structure_diagnostic(error, span))?;
                        params.push(Value::Domain(DomainValue::Param(ParamValue {
                            context: parameter.context.clone(),
                            repr,
                        })));
                    }
                }
                return Ok(Value::Tuple(vec![
                    Value::List(params),
                    Value::Integer(BigInt::from(start_pos)),
                ]));
            }
            arity(name, arguments, 2, span)?;
            let rf = as_real_form(&arguments[0], span)?;
            let df = as_real_form(&arguments[1], span)?;
            check_dual_pair(rf, df, span)?;
            Ok(Value::Domain(DomainValue::Block(build_block(
                rf, df, span,
            )?)))
        }
        "dual_real_form" => {
            arity(name, arguments, 2, span)?;
            let Value::Domain(DomainValue::InnerClass(context)) = &arguments[0] else {
                return Err(type_error(span, "expected an InnerClass"));
            };
            let external = as_usize(&arguments[1], span)?;
            let dual = build_dual_inner_class(context, span)?;
            let form = build_real_form(&dual, external, span)?;
            Ok(Value::Domain(DomainValue::RealForm(form)))
        }
        "quasisplit_form" => {
            arity(name, arguments, 1, span)?;
            let Value::Domain(DomainValue::InnerClass(context)) = &arguments[0] else {
                return Err(type_error(span, "expected an InnerClass"));
            };
            let external = context.order.quasisplit_external();
            let form = build_real_form(context, external, span)?;
            Ok(Value::Domain(DomainValue::RealForm(form)))
        }
        // strong_components (atlas-types.w:7525-7556): Tarjan's strong
        // components of an adjacency-list graph, then the induced quotient
        // graph's edge lists. Returns (partition, induced).
        "strong_components" => {
            arity(name, arguments, 1, span)?;
            let Value::List(rows) = &arguments[0] else {
                return Err(type_error(span, "strong_components expects a row of rows"));
            };
            let mut graph: Vec<Vec<usize>> = Vec::with_capacity(rows.len());
            for entry in rows {
                let Value::List(edges) = entry else {
                    return Err(type_error(span, "strong_components expects a row of rows"));
                };
                let mut targets = Vec::with_capacity(edges.len());
                for edge in edges {
                    let target = as_usize(edge, span)?;
                    if target >= rows.len() {
                        return Err(runtime(
                            span,
                            format!(
                                "Edge target {target} out of bounds (should be <{})",
                                rows.len()
                            ),
                        ));
                    }
                    targets.push(target);
                }
                graph.push(targets);
            }
            let (partition, induced) = strong_components(&graph);
            let partition_value = Value::List(
                partition
                    .iter()
                    .map(|class| {
                        Value::List(
                            class
                                .iter()
                                .map(|&v| Value::Integer(BigInt::from(v)))
                                .collect(),
                        )
                    })
                    .collect(),
            );
            let induced_value = Value::List(
                induced
                    .iter()
                    .map(|class| {
                        Value::List(
                            class
                                .iter()
                                .map(|&v| Value::Integer(BigInt::from(v)))
                                .collect(),
                        )
                    })
                    .collect(),
            );
            Ok(Value::Tuple(vec![partition_value, induced_value]))
        }
        "dual_quasisplit_form" => {
            arity(name, arguments, 1, span)?;
            let Value::Domain(DomainValue::InnerClass(context)) = &arguments[0] else {
                return Err(type_error(span, "expected an InnerClass"));
            };
            let dual = build_dual_inner_class(context, span)?;
            let external = dual.order.quasisplit_external();
            let form = build_real_form(&dual, external, span)?;
            Ok(Value::Domain(DomainValue::RealForm(form)))
        }
        "form_number" => {
            arity(name, arguments, 1, span)?;
            let context = as_real_form(&arguments[0], span)?;
            Ok(Value::Integer(BigInt::from(context.external)))
        }
        // ic_Cartan_class_wrapper (atlas-types.w:4019-4034) and
        // rf_Cartan_class_wrapper (atlas-types.w:4040-4060): the real-form
        // overload translates its per-form index through the form's Cartan
        // set into the inner-class numbering (`Cartan_set().n_th(i)`).
        "Cartan_class" => {
            if let [Value::Domain(DomainValue::KgbElement(form, id))] = arguments {
                let cartan = form
                    .graph
                    .cartan_of(*id)
                    .ok_or_else(|| runtime(span, "Inexistent KGB element"))?;
                return Ok(cartan_class_value(&form.parent, cartan));
            }
            arity(name, arguments, 2, span)?;
            let index = as_integer(&arguments[1], span)?;
            match &arguments[0] {
                Value::Domain(DomainValue::InnerClass(context)) => {
                    let number = check_cartan_number(
                        &index,
                        context.classification.cartan_ids().len(),
                        "inner class",
                        span,
                    )?;
                    let id = context
                        .classification
                        .cartan_ids()
                        .nth(number)
                        .expect("bounds checked against the class count");
                    Ok(cartan_class_value(context, id))
                }
                Value::Domain(DomainValue::RealForm(form)) => {
                    let set = form
                        .parent
                        .classification
                        .cartan_set(form.internal)
                        .expect("a real form's internal number is in range");
                    let id = set[check_cartan_number(&index, set.len(), "real form", span)?];
                    Ok(cartan_class_value(&form.parent, id))
                }
                other => Err(type_error(
                    span,
                    format!("expected an InnerClass or RealForm, found {other}"),
                )),
            }
        }
        // n_Cartan_classes_wrapper (atlas-types.w:3308-3322) and
        // count_Cartans_wrapper (atlas-types.w:3698-3704).
        "nr_of_Cartan_classes" => {
            arity(name, arguments, 1, span)?;
            match &arguments[0] {
                Value::Domain(DomainValue::InnerClass(context)) => Ok(Value::Integer(
                    BigInt::from(context.classification.cartan_ids().len()),
                )),
                Value::Domain(DomainValue::RealForm(form)) => Ok(Value::Integer(BigInt::from(
                    form.parent
                        .classification
                        .cartan_set(form.internal)
                        .expect("a real form's internal number is in range")
                        .len(),
                ))),
                other => Err(type_error(
                    span,
                    format!("expected an InnerClass or RealForm, found {other}"),
                )),
            }
        }
        // most_split_Cartan_wrapper (atlas-types.w:4065-4071).
        "most_split_Cartan" => {
            arity(name, arguments, 1, span)?;
            let form = as_real_form(&arguments[0], span)?;
            let id = form
                .parent
                .classification
                .most_split(form.internal)
                .expect("most-split uniqueness is a construction invariant");
            Ok(cartan_class_value(&form.parent, id))
        }
        // components_rank (atlas-types.w:3936-3939): the number of dual
        // component-group generators of the form, i.e. the rank of the
        // kernel of the restriction map on the most-split Cartan
        // (realredgp.cpp:73-75 dual_pi0_gens).
        // KGB_Hasse (atlas-types.w:3735-3743): the Bruhat Hasse matrix of
        // the real form's KGB, `M[i][j] = 1` when `i` is an immediate
        // Bruhat predecessor of `j`.
        "KGB_Hasse" => {
            arity(name, arguments, 1, span)?;
            let context = as_real_form(&arguments[0], span)?;
            let graph = &context.graph;
            let n = graph.size();
            let hasse = graph.bruhat_hasse();
            let mut columns = vec![vec![0_i32; n]; n];
            for (z, row) in hasse.iter().enumerate() {
                for &down in row {
                    columns[z][down] = 1;
                }
            }
            columns_matrix_value(&columns, n, span)
        }
        "components_rank" => {
            arity(name, arguments, 1, span)?;
            let form = as_real_form(&arguments[0], span)?;
            let id = form
                .parent
                .classification
                .most_split(form.internal)
                .expect("most-split uniqueness is a construction invariant");
            let representative = form
                .parent
                .classification
                .cartan_class(id)
                .expect("most-split Cartan is in range")
                .representative();
            let theta = representative
                .root_involution()
                .involution()
                .weight_matrix()
                .to_vec();
            let rank = atlas_real_group::dual_component_group_rank(
                &theta,
                &form.parent.inner_class.datum().clone(),
                &atlas_real_group::IntegerLatticeBudget::new(64, 100_000, 100_000, 128),
            )
            .map_err(|error| structure_diagnostic(error, span))?;
            Ok(Value::Integer(BigInt::from(rank)))
        }
        // real_forms_of_Cartan_wrapper (atlas-types.w:4155-4168): every
        // external form whose Cartan set contains the class, in external
        // order.
        "real_forms" => {
            arity(name, arguments, 1, span)?;
            let (context, id) = as_cartan_class(&arguments[0], span)?;
            let mut forms = Vec::new();
            for external in 0..context.order.form_count() {
                let internal = context
                    .order
                    .internal(external)
                    .expect("external numbers are in range");
                if context
                    .classification
                    .cartan_set(internal)
                    .expect("a real form's internal number is in range")
                    .contains(&id)
                {
                    forms.push(Value::Domain(DomainValue::RealForm(build_real_form(
                        context, external, span,
                    )?)));
                }
            }
            Ok(Value::List(forms))
        }
        // dual_real_forms_of_Cartan_wrapper (atlas-types.w:4171-4185): the
        // same sweep on the dual side, at the corresponding dual Cartan.
        "dual_real_forms" => {
            arity(name, arguments, 1, span)?;
            let (context, id) = as_cartan_class(&arguments[0], span)?;
            let number =
                cartan_number(context, id).expect("CartanClass values carry an in-range id");
            let (dual_id, _) = context
                .dual_cartans
                .get(number)
                .expect("the correspondence covers every Cartan class");
            let dual = build_dual_inner_class(context, span)?;
            let mut forms = Vec::new();
            for external in 0..dual.order.form_count() {
                let internal = dual
                    .order
                    .internal(external)
                    .expect("external numbers are in range");
                if dual
                    .classification
                    .cartan_set(internal)
                    .expect("a real form's internal number is in range")
                    .contains(dual_id)
                {
                    forms.push(Value::Domain(DomainValue::RealForm(build_real_form(
                        &dual, external, span,
                    )?)));
                }
            }
            Ok(Value::List(forms))
        }
        // square_classes_wrapper (atlas-types.w:4229-4246): per square class,
        // each strong orbit's weak real form, as an EXTERNAL form number.
        "square_classes" => {
            arity(name, arguments, 1, span)?;
            let (context, id) = as_cartan_class(&arguments[0], span)?;
            let cartan = context
                .classification
                .cartan_class(id)
                .expect("CartanClass values carry an in-range id");
            let data = context
                .strong
                .strong_real_data(id)
                .expect("the strong layer covers every Cartan class");
            let mut classes = Vec::new();
            for square in data.square_classes() {
                let mut orbits = Vec::new();
                for orbit in 0..data
                    .fiber_orbit_count(square)
                    .expect("square_classes yields in-range ids")
                {
                    let local = data
                        .weak_real_of_orbit(square, orbit)
                        .expect("orbit numbers are bounded by the orbit count");
                    let internal = cartan
                        .labels()
                        .label(local)
                        .expect("toWeakReal lands in a local class");
                    let external = context
                        .order
                        .external(internal)
                        .expect("real-form labels land in a global form");
                    orbits.push(Value::Integer(BigInt::from(external)));
                }
                classes.push(Value::List(orbits));
            }
            Ok(Value::List(classes))
        }
        // fiber_partition_wrapper (atlas-types.w:4199-4223): the fiber
        // elements whose weak class labels back to the given real form,
        // numbered by the fiber's canonical enumeration.
        "fiber_partition" => {
            arity(name, arguments, 2, span)?;
            let (context, id) = as_cartan_class(&arguments[0], span)?;
            let form = as_real_form(&arguments[1], span)?;
            fiber_partition_membership(context, id, form, span)?;
            let cartan = context
                .classification
                .cartan_class(id)
                .expect("CartanClass values carry an in-range id");
            let dimension = cartan.grading().adjoint_fiber().dimension();
            // The partition's mask-bits bound keeps this shift in range.
            let element_count = 1_u64
                .checked_shl(
                    u32::try_from(dimension)
                        .map_err(|_| runtime(span, "internal fiber dimension overflow"))?,
                )
                .ok_or_else(|| runtime(span, "internal fiber dimension overflow"))?;
            let mut members = Vec::new();
            for mask in 0..element_count {
                let local = cartan
                    .partition()
                    .class_of_mask(mask)
                    .map_err(|error| runtime(span, error.to_string()))?;
                if cartan.labels().label(local) == Some(form.internal) {
                    members.push(Value::Integer(BigInt::from(mask)));
                }
            }
            Ok(Value::List(members))
        }
        // occurrence_matrix_wrapper (atlas-types.w:3361-3379): rows indexed
        // by real forms, columns by Cartan classes; membership reads the
        // per-form Cartan set.
        "occurrence_matrix" => {
            arity(name, arguments, 1, span)?;
            let Value::Domain(DomainValue::InnerClass(context)) = &arguments[0] else {
                return Err(type_error(span, "expected an InnerClass"));
            };
            let class_count = context.classification.cartan_ids().len();
            let mut rows = Vec::with_capacity(context.order.form_count());
            for external in 0..context.order.form_count() {
                let internal = context
                    .order
                    .internal(external)
                    .expect("external numbers are in range");
                let set = context
                    .classification
                    .cartan_set(internal)
                    .expect("a real form's internal number is in range");
                let mut row = vec![0_i32; class_count];
                for (column, id) in context.classification.cartan_ids().enumerate() {
                    if set.contains(&id) {
                        row[column] = 1;
                    }
                }
                rows.push(row);
            }
            matrix_value(&rows, span)
        }
        // dual_occurrence_matrix_wrapper (atlas-types.w:3381-3400): the same
        // sweep over the dual real forms, mapped back to this inner class's
        // Cartan numbering through the dual correspondence.
        "dual_occurrence_matrix" => {
            arity(name, arguments, 1, span)?;
            let Value::Domain(DomainValue::InnerClass(context)) = &arguments[0] else {
                return Err(type_error(span, "expected an InnerClass"));
            };
            let dual = build_dual_inner_class(context, span)?;
            let class_count = context.classification.cartan_ids().len();
            let mut rows = Vec::with_capacity(dual.order.form_count());
            for external in 0..dual.order.form_count() {
                let internal = dual
                    .order
                    .internal(external)
                    .expect("external numbers are in range");
                let set = dual
                    .classification
                    .cartan_set(internal)
                    .expect("a real form's internal number is in range");
                let mut row = vec![0_i32; class_count];
                for (column, slot) in row.iter_mut().enumerate() {
                    let (dual_id, _) = context
                        .dual_cartans
                        .get(column)
                        .expect("the correspondence covers every Cartan class");
                    if set.contains(dual_id) {
                        *slot = 1;
                    }
                }
                rows.push(row);
            }
            matrix_value(&rows, span)
        }
        // block_sizes_wrapper (atlas-types.w:3323-3335): the full
        // numRealForms x numDualRealForms table of InnerClass::block_size.
        "block_sizes" => {
            arity(name, arguments, 1, span)?;
            let Value::Domain(DomainValue::InnerClass(context)) = &arguments[0] else {
                return Err(type_error(span, "expected an InnerClass"));
            };
            let dual = build_dual_inner_class(context, span)?;
            let mut rows = Vec::with_capacity(context.order.form_count());
            for external in 0..context.order.form_count() {
                let internal = context
                    .order
                    .internal(external)
                    .expect("external numbers are in range");
                let mut row = Vec::with_capacity(dual.order.form_count());
                for dual_external in 0..dual.order.form_count() {
                    let dual_internal = dual
                        .order
                        .internal(dual_external)
                        .expect("external numbers are in range");
                    let size = block_size_sum(context, &dual, internal, dual_internal, span)?;
                    row.push(
                        i32::try_from(size)
                            .map_err(|_| runtime(span, "block size exceeds the mat entry range"))?,
                    );
                }
                rows.push(row);
            }
            matrix_value(&rows, span)
        }
        // block_size_wrapper (atlas-types.w:3337-3357): the unsigned
        // extraction and both bounds checks precede the no-value gate, so
        // `validate` shares them via block_size_numbers.
        "block_size" => {
            arity(name, arguments, 3, span)?;
            let Value::Domain(DomainValue::InnerClass(context)) = &arguments[0] else {
                return Err(type_error(span, "expected an InnerClass"));
            };
            let (external, dual_external) = block_size_numbers(context, arguments, span)?;
            let internal = context
                .order
                .internal(usize::try_from(external).expect("bounds checked against the form count"))
                .expect("bounds checked against the form count");
            let dual = build_dual_inner_class(context, span)?;
            let dual_internal = dual
                .order
                .internal(
                    usize::try_from(dual_external)
                        .expect("bounds checked against the dual form count"),
                )
                .expect("bounds checked against the dual form count");
            Ok(Value::Integer(BigInt::from(block_size_sum(
                context,
                &dual,
                internal,
                dual_internal,
                span,
            )?)))
        }
        // Cartan_order_wrapper (atlas-types.w:3709-3724): the 0/1 matrix of
        // the inner-class Cartan poset over 0..n-1 with n = numCartan(rf) —
        // upstream indexes the poset DIRECTLY, without the form's Cartan-set
        // remapping, and fills only the i<=j triangle.
        "Cartan_order" => {
            arity(name, arguments, 1, span)?;
            let form = as_real_form(&arguments[0], span)?;
            let classification = &form.parent.classification;
            let count = classification
                .cartan_set(form.internal)
                .expect("a real form's internal number is in range")
                .len();
            let mut rows = Vec::with_capacity(count);
            for i in 0..count {
                let mut row = vec![0_i32; count];
                for (j, slot) in row.iter_mut().enumerate().skip(i) {
                    // poset::Poset::lesseq (poset.h:84-85): reflexive on the
                    // diagonal, the strict closed Cayley order off it.
                    let related = i == j
                        || classification
                            .is_below(
                                classification
                                    .cartan_ids()
                                    .nth(i)
                                    .expect("i is bounded by the class count"),
                                classification
                                    .cartan_ids()
                                    .nth(j)
                                    .expect("j is bounded by the class count"),
                            )
                            .expect("Cartan ids are in range");
                    if related {
                        *slot = 1;
                    }
                }
                rows.push(row);
            }
            matrix_value(&rows, span)
        }
        "KGB_size" => {
            arity(name, arguments, 1, span)?;
            let context = as_real_form(&arguments[0], span)?;
            Ok(Value::Integer(BigInt::from(context.graph.size())))
        }
        // central_fiber_wrapper (atlas-types.w:3915-3929): the fundamental
        // fiber's stabilizer torus parts, wrapped as a row of vec.
        "central_fiber" => {
            arity(name, arguments, 1, span)?;
            let form = as_real_form(&arguments[0], span)?;
            let parts = central_fiber(
                &form.parent.classification,
                &form.parent.strong,
                form.internal,
            )
            .map_err(|error| runtime(span, error.to_string()))?;
            let mut rows = Vec::with_capacity(parts.len());
            for part in parts {
                let mut entries = Vec::with_capacity(part.dimension());
                for index in 0..part.dimension() {
                    entries.push(i32::from(
                        part.bit(index).expect("indices stay below the dimension"),
                    ));
                }
                rows.push(Value::Vector(Vec32(entries)));
            }
            Ok(Value::List(rows))
        }
        "KGB" => {
            arity(name, arguments, 2, span)?;
            let context = as_real_form(&arguments[0], span)?;
            let index = as_integer(&arguments[1], span)?;
            // Upstream rejects negative and oversized numbers alike with the
            // value echoed (atlas-types.w:4412 `KGB_elt_wrapper`).
            let size = BigInt::from(context.graph.size());
            if index < 0 || index >= size {
                return Err(runtime(span, format!("Inexistent KGB element: {index}")));
            }
            let index =
                usize::try_from(&index).map_err(|_| runtime(span, "Inexistent KGB element"))?;
            let id = context
                .graph
                .ids()
                .nth(index)
                .ok_or_else(|| runtime(span, "Inexistent KGB element"))?;
            Ok(Value::Domain(DomainValue::KgbElement(
                Arc::clone(context),
                id,
            )))
        }
        // build_KGB_element_wrapper (atlas-types.w:4580-4607): the synthetic
        // (RealForm,mat,ratvec) constructor.
        "KGB_elt" => {
            arity(name, arguments, 3, span)?;
            let context = as_real_form(&arguments[0], span)?;
            let id = build_kgb_element(context, &arguments[1], &arguments[2], span)?;
            Ok(Value::Domain(DomainValue::KgbElement(
                Arc::clone(context),
                id,
            )))
        }
        // block_element_wrapper (atlas-types.w:4826-4845): the pair of KGB
        // elements identifying the block element, the y component read in
        // the dual real form's own KGB numbering.
        "element" => {
            arity(name, arguments, 2, span)?;
            let block = as_block(&arguments[0], span)?;
            let index = block_element_index(block, &arguments[1], span)?;
            let x = block.graph.x(index).expect("an in-range block element");
            let y = block.graph.y(index).expect("an in-range block element");
            Ok(Value::Tuple(vec![
                Value::Domain(DomainValue::KgbElement(Arc::clone(&block.rf), x)),
                Value::Domain(DomainValue::KgbElement(Arc::clone(&block.dual_rf), y)),
            ]))
        }
        // block_index_wrapper (atlas-types.w:4857-4876): the inverse
        // lookup, gated on the inner classes and the involution fibers; the
        // element numbers read in the BLOCK's KGB sets, like upstream's
        // `b->rf->kgb()` (only the inner classes are gated, not the forms).
        "index" => {
            arity(name, arguments, 3, span)?;
            let block = as_block(&arguments[0], span)?;
            let (x_context, x) = as_kgb_element(&arguments[1], span)?;
            let (y_context, y) = as_kgb_element(&arguments[2], span)?;
            if x_context.parent.inner_class != block.rf.parent.inner_class {
                return Err(runtime(span, "Real form not in inner class of block"));
            }
            if y_context.parent.inner_class != block.dual_rf.parent.inner_class {
                return Err(runtime(span, "Dual real form not in inner class of block"));
            }
            block_fiber_check(block, x, y, span)?;
            let z = block
                .graph
                .element(x, y)
                .map_err(|error| runtime(span, error.to_string()))?;
            Ok(Value::Integer(BigInt::from(z)))
        }
        "cross" => {
            if arguments.len() == 2 {
                // parameter_cross_wrapper (atlas-types.w:3849-3869):
                // (int, Param -> Param), the simple-reflection cross on the
                // standard parameter (repr.cpp:891-910).
                if let Value::Domain(DomainValue::Param(parameter)) = &arguments[1] {
                    if matches!(arguments[0], Value::Vector(_)) {
                        // root_parameter_cross_wrapper
                        // (atlas-types.w:6474-6483): unlike the int overload,
                        // this does not make the parameter dominant first.
                        let coordinates = as_weight_vec(&arguments[0], span)?;
                        let rc = rep_context(&parameter.context);
                        let root = rc
                            .root_system()
                            .id_of(&Weight::new(coordinates))
                            .ok_or_else(|| runtime(span, "Not a root"))?;
                        if !rc
                            .is_integral_root(root, parameter.repr.gamma())
                            .map_err(|error| structure_diagnostic(error, span))?
                        {
                            return Err(runtime(span, "Not an integral root"));
                        }
                        let result = rc
                            .cross_root(root, &parameter.repr)
                            .map_err(|error| structure_diagnostic(error, span))?;
                        return Ok(Value::Domain(DomainValue::Param(ParamValue {
                            context: parameter.context.clone(),
                            repr: result,
                        })));
                    }
                    let s = parameter_generator(parameter, &arguments[0], span)?;
                    let rc = rep_context(&parameter.context);
                    let z = parameter
                        .repr
                        .made_dominant(&rc)
                        .map_err(|e| runtime(span, e.to_string()))?;
                    let gamma = z.gamma().clone();
                    let seed = StandardReprMod::mod_reduce(&rc, &z)
                        .map_err(|e| structure_diagnostic(e, span))?;
                    let context = CommonContext::integral(&rc, &gamma)
                        .map_err(|e| structure_diagnostic(e, span))?;
                    let result = context
                        .cross(s, &seed)
                        .and_then(|srm| srm.to_standard(&rc, &gamma))
                        .map_err(|e| structure_diagnostic(e, span))?;
                    return Ok(Value::Domain(DomainValue::Param(ParamValue {
                        context: parameter.context.clone(),
                        repr: result,
                    })));
                }
            } else if arguments.len() == 3 {
                // block_cross_wrapper (atlas-types.w:4920-4937).
                let block = as_block(&arguments[1], span)?;
                let generator = block_generator_check(block, &arguments[0], span)?;
                let index = block_element_index(block, &arguments[2], span)?;
                let target = block
                    .graph
                    .cross(index, generator)
                    .expect("an in-range block element and generator");
                return Ok(Value::Integer(BigInt::from(target)));
            }
            arity(name, arguments, 2, span)?;
            let generator = as_usize(&arguments[0], span)?;
            let (context, id) = as_kgb_element(&arguments[1], span)?;
            check_generator(context, generator, span)?;
            let target = context
                .graph
                .cross(id, generator)
                .ok_or_else(|| runtime(span, "Inexistent KGB element"))?;
            Ok(Value::Domain(DomainValue::KgbElement(
                Arc::clone(context),
                target,
            )))
        }
        "Cayley" => {
            if arguments.len() == 2 {
                // parameter_Cayley_wrapper (atlas-types.w:3871-3887):
                // (int, Param -> Param), the KL Cayley transform
                // (repr.cpp:943-1002). A Cayley_error returns the input
                // parameter unchanged.
                if let Value::Domain(DomainValue::Param(parameter)) = &arguments[1] {
                    if matches!(arguments[0], Value::Vector(_)) {
                        // root_parameter_Cayley_wrapper
                        // (atlas-types.w:6485-6518): a missing ambient root
                        // and a nonintegral one deliberately share the same
                        // diagnostic; an undefined transform returns input.
                        let coordinates = as_weight_vec(&arguments[0], span)?;
                        let rc = rep_context(&parameter.context);
                        let root = Weight::new(coordinates);
                        let Some(result) = rc.any_cayley_root(&root, &parameter.repr).map_err(
                            |error| match error {
                                StructureError::RepInvariantViolation {
                                    invariant: "integral parameter root",
                                } => runtime(span, "Not an integral root"),
                                StructureError::RepInvariantViolation {
                                    invariant: "standard parameter in integral make_dominant",
                                } => runtime(
                                    span,
                                    "Cannot make non-standard parameter integrally dominant",
                                ),
                                other => structure_diagnostic(other, span),
                            },
                        )?
                        else {
                            return Ok(Value::Domain(DomainValue::Param(parameter.clone())));
                        };
                        return Ok(Value::Domain(DomainValue::Param(ParamValue {
                            context: parameter.context.clone(),
                            repr: result,
                        })));
                    }
                    let s = parameter_generator(parameter, &arguments[0], span)?;
                    let rc = rep_context(&parameter.context);
                    let z = parameter
                        .repr
                        .made_dominant(&rc)
                        .map_err(|e| runtime(span, e.to_string()))?;
                    let gamma = z.gamma().clone();
                    let seed = StandardReprMod::mod_reduce(&rc, &z)
                        .map_err(|e| structure_diagnostic(e, span))?;
                    let context = CommonContext::integral(&rc, &gamma)
                        .map_err(|e| structure_diagnostic(e, span))?;
                    let transformed = match context
                        .status(s, seed.x())
                        .map_err(|e| structure_diagnostic(e, span))?
                        .0
                    {
                        KgbStatus::ImaginaryNoncompact => Some(
                            context
                                .up_cayley(s, &seed)
                                .map_err(|e| structure_diagnostic(e, span))?,
                        ),
                        KgbStatus::Real
                            if context
                                .is_parity(s, &seed)
                                .map_err(|e| structure_diagnostic(e, span))? =>
                        {
                            Some(
                                context
                                    .down_cayley(s, &seed)
                                    .map_err(|e| structure_diagnostic(e, span))?,
                            )
                        }
                        _ => None,
                    };
                    let Some(transformed) = transformed else {
                        return Ok(Value::Domain(DomainValue::Param(parameter.clone())));
                    };
                    let result = transformed
                        .to_standard(&rc, &gamma)
                        .map_err(|e| structure_diagnostic(e, span))?;
                    return Ok(Value::Domain(DomainValue::Param(ParamValue {
                        context: parameter.context.clone(),
                        repr: result,
                    })));
                }
            } else if arguments.len() == 3 {
                // block_Cayley_wrapper (atlas-types.w:4939-4963): an
                // undefined Cayley returns the INPUT index as the signal.
                let block = as_block(&arguments[1], span)?;
                let generator = block_generator_check(block, &arguments[0], span)?;
                let index = block_element_index(block, &arguments[2], span)?;
                let (first, _) = block
                    .graph
                    .cayley(index, generator)
                    .expect("an in-range block element and generator");
                return Ok(match first {
                    Some(target) => Value::Integer(BigInt::from(target)),
                    None => arguments[2].clone(),
                });
            }
            arity(name, arguments, 2, span)?;
            let generator = as_usize(&arguments[0], span)?;
            let (context, id) = as_kgb_element(&arguments[1], span)?;
            check_generator(context, generator, span)?;
            let target = any_cayley(context, generator, id, span)?;
            Ok(Value::Domain(DomainValue::KgbElement(
                Arc::clone(context),
                target,
            )))
        }
        // block_inverse_Cayley_wrapper (atlas-types.w:4965-4989): like
        // block_Cayley_wrapper, undefined returns the input index.
        "inverse_Cayley" => {
            arity(name, arguments, 3, span)?;
            let block = as_block(&arguments[1], span)?;
            let generator = block_generator_check(block, &arguments[0], span)?;
            let index = block_element_index(block, &arguments[2], span)?;
            let (first, _) = block
                .graph
                .inverse_cayley(index, generator)
                .expect("an in-range block element and generator");
            Ok(match first {
                Some(target) => Value::Integer(BigInt::from(target)),
                None => arguments[2].clone(),
            })
        }
        "status" => {
            if arguments.len() == 3 {
                // block_status_wrapper (atlas-types.w:4892-4914): the
                // renumbered DescentStatus of the block element.
                let block = as_block(&arguments[1], span)?;
                let generator = block_generator_check(block, &arguments[0], span)?;
                let index = block_element_index(block, &arguments[2], span)?;
                let code = block
                    .graph
                    .status_code(index, generator)
                    .expect("an in-range block element and generator");
                return Ok(Value::Integer(BigInt::from(code)));
            }
            arity(name, arguments, 2, span)?;
            let generator = as_usize(&arguments[0], span)?;
            let (context, id) = as_kgb_element(&arguments[1], span)?;
            check_generator(context, generator, span)?;
            let code = status_code(context, generator, id)
                .ok_or_else(|| runtime(span, "Inexistent KGB element"))?;
            Ok(Value::Integer(BigInt::from(code)))
        }
        "length" => {
            arity(name, arguments, 1, span)?;
            if let Value::Domain(DomainValue::WeylElement(value)) = &arguments[0] {
                // W_length_wrapper (atlas-types.w:2391-2394).
                return Ok(Value::Integer(BigInt::from(value.element.length())));
            }
            if let Value::Domain(DomainValue::Param(parameter)) = &arguments[0] {
                // param_length_wrapper (atlas-types.w:6368-6373):
                // Rep_table::length (repr.cpp:1435-1442) — make_dominant,
                // then the shared partial-block lookup on the integral
                // subsystem; the length is the representative's height
                // inside that located block (never the full-rank block).
                let rc = rep_context(&parameter.context);
                let z = parameter
                    .repr
                    .made_dominant(&rc)
                    .map_err(|e| runtime(span, e.to_string()))?;
                let located = match parameter.context.rep.lookup(&z) {
                    Ok(located) => located,
                    // The shared table cannot yet merge overlapping partial
                    // blocks (RepTable::commit_partial NYI); fall back to the
                    // full block, whose per-row length is the same Bruhat
                    // height — the element's whole downset is present in
                    // both blocks.
                    Err(StructureError::NotYetImplemented { .. }) => parameter
                        .context
                        .rep
                        .lookup_full_block(&z)
                        .map_err(|error| structure_diagnostic(error, span))?,
                    Err(error) => return Err(structure_diagnostic(error, span)),
                };
                let length = located
                    .block()
                    .length(located.raw_row())
                    .ok_or_else(|| runtime(span, "block length unavailable"))?;
                return Ok(Value::Integer(length.into()));
            }
            let (context, id) = as_kgb_element(&arguments[0], span)?;
            let length = context
                .graph
                .length(id)
                .ok_or_else(|| runtime(span, "Inexistent KGB element"))?;
            Ok(Value::Integer(BigInt::from(length)))
        }
        "involution" => {
            if arguments.len() == 3 {
                // basic_involution_wrapper (atlas-types.w:860-880) and
                // based_involution_wrapper (atlas-types.w:902-927).
                let lie_type = as_lie_type(&arguments[0], span)?;
                return primitive_involution(&lie_type, &arguments[1], &arguments[2], span);
            }
            arity(name, arguments, 1, span)?;
            // Cartan_involution_wrapper (atlas-types.w:4080-4084): the
            // class's distinguished involution matrix.
            if let Value::Domain(DomainValue::CartanClass(context, id)) = &arguments[0] {
                let cartan = context
                    .classification
                    .cartan_class(*id)
                    .expect("CartanClass values carry an in-range id");
                return matrix_value(
                    cartan
                        .representative()
                        .root_involution()
                        .involution()
                        .weight_matrix(),
                    span,
                );
            }
            let (context, id) = as_kgb_element(&arguments[0], span)?;
            let involution = context
                .graph
                .involution_of(id)
                .and_then(|involution| context.table.record(involution))
                .ok_or_else(|| runtime(span, "Inexistent KGB element"))?;
            matrix_value(involution.theta().weight_matrix(), span)
        }
        // Cartan_info (atlas-types.w:4102-4160): the classify triple, the
        // Cartan involution's Weyl word, the orbit/fiber sizes, and the
        // subsystem types of the imaginary, real and complex simple roots.
        // orientation_nr (atlas-types.w:6546-6552, repr.cpp:455-493): the
        // orientation number of a standard parameter — the count of
        // non-integral real roots whose coroot pairing with gamma is
        // mis-oriented, plus one per conjugate complex pair.
        // reducibility_points (atlas-types.w:6561-6568, repr.cpp:825-925):
        // the reducibility fractions of a standard parameter, ascending.
        "reducibility_points" => {
            arity(name, arguments, 1, span)?;
            let Value::Domain(DomainValue::Param(parameter)) = &arguments[0] else {
                return Err(type_error(
                    span,
                    format!(
                        "{name} has no matching overload for {} argument(s)",
                        arguments.len()
                    ),
                ));
            };
            let rc = rep_context(&parameter.context);
            let points = rc
                .reducibility_points(&parameter.repr)
                .map_err(|error| structure_diagnostic(error, span))?;
            Ok(Value::List(
                points
                    .iter()
                    .map(|&(numerator, denominator)| {
                        Value::Rational(BigRational::from_signeds(numerator, denominator))
                    })
                    .collect(),
            ))
        }
        "orientation_nr" => {
            arity(name, arguments, 1, span)?;
            let Value::Domain(DomainValue::Param(parameter)) = &arguments[0] else {
                return Err(type_error(
                    span,
                    format!(
                        "{name} has no matching overload for {} argument(s)",
                        arguments.len()
                    ),
                ));
            };
            let rc = rep_context(&parameter.context);
            let root_system = rc.inner_class().root_system();
            let z = &parameter.repr;
            let involution_id = parameter
                .context
                .graph
                .involution_of(z.x())
                .ok_or_else(|| runtime(span, "Inexistent KGB element"))?;
            let record = rc
                .table()
                .record(involution_id)
                .ok_or_else(|| runtime(span, "Inexistent involution"))?;
            let root_involution = record.twisted_involution().root_involution();
            let real: std::collections::HashSet<usize> = root_involution
                .roots_of_kind(RootKind::Real)
                .map(|root| root.index())
                .collect();
            let all_roots: Vec<RootId> = (0..root_system.roots().len())
                .map(RootId::from_usize)
                .collect();
            let two_rho_sum = two_rho(root_system, &all_roots);
            let real_roots: Vec<RootId> = root_involution.roots_of_kind(RootKind::Real).collect();
            let two_rho_real = two_rho(root_system, &real_roots);
            let lifted = rc
                .y_lift(involution_id, z.y_bits())
                .map_err(|error| structure_diagnostic(error, span))?;
            let test_wt: Vec<i32> = lifted
                .as_slice()
                .iter()
                .zip(two_rho_sum.as_slice())
                .zip(two_rho_real.as_slice())
                .map(|((&a, &b), &c)| a + b - c)
                .collect();
            let numer = z.gamma().numerator();
            let denom = z.gamma().denominator();
            // Positive-root indices in the root system's ambient order.
            let mut positive_indices: Vec<usize> = (0..root_system.roots().len())
                .filter(|&index| {
                    root_system
                        .is_positive(RootId::from_usize(index))
                        .unwrap_or(false)
                })
                .collect();
            positive_indices.sort_by_key(|&index| {
                // rt_abs ordering: coroot coordinates, ascending.
                root_system
                    .coroot(RootId::from_usize(index))
                    .map(|coroot| coroot.as_slice().to_vec())
                    .unwrap_or_default()
            });
            let mut count = 0_usize;
            for &alpha_index in positive_indices.iter() {
                let Some(coroot_alpha) = root_system.coroot(RootId::from_usize(alpha_index)) else {
                    continue;
                };
                let num: i64 = coroot_alpha
                    .as_slice()
                    .iter()
                    .zip(numer)
                    .map(|(&c, &n)| i64::from(c) * n)
                    .sum();
                if num.rem_euclid(denom) != 0 {
                    if real.contains(&alpha_index) {
                        let test_pair: i64 = coroot_alpha
                            .as_slice()
                            .iter()
                            .zip(&test_wt)
                            .map(|(&c, &t)| i64::from(c) * i64::from(t))
                            .sum();
                        let eps = if test_pair.rem_euclid(4) == 0 {
                            0
                        } else {
                            denom
                        };
                        let oriented = (num > 0) == ((num + eps).rem_euclid(2 * denom) < denom);
                        if oriented {
                            count += 1;
                        }
                    } else {
                        let beta = root_involution
                            .image(RootId::from_usize(alpha_index))
                            .ok_or_else(|| runtime(span, "Inexistent root"))?;
                        let beta_index = beta.index();
                        let beta_coroot = root_system
                            .coroot(beta)
                            .ok_or_else(|| runtime(span, "Inexistent root"))?;
                        let beta_pair: i64 = beta_coroot
                            .as_slice()
                            .iter()
                            .zip(numer)
                            .map(|(&c, &n)| i64::from(c) * n)
                            .sum();
                        // Consider only the first of the conjugate pair:
                        // compare the positive-root order of alpha and beta.
                        let alpha_order = positive_indices.iter().position(|&r| r == alpha_index);
                        let beta_order = positive_indices.iter().position(|&r| r == beta_index);
                        if let (Some(a), Some(b)) = (alpha_order, beta_order) {
                            if a < b && (num > 0) != (beta_pair > 0) {
                                count += 1;
                            }
                        }
                    }
                }
            }
            Ok(Value::Integer(BigInt::from(count)))
        }
        // block_Hasse (atlas-types.w:6825-6852): the full block of a
        // standard parameter plus its Bruhat Hasse matrix. Each block
        // element is reported as the parameter `sr(representative(z),
        // bm, gamma)` — with the identity block modifier the lambda-rho is
        // carried from the input parameter.
        // KL_sum_at_s / KL_sum_at_s_to_height (atlas-types.w:8350-8388,
        // repr.cpp:2127-2210): the KL column of a final parameter,
        // evaluated at q = s, as a ParamPol. The identity block modifier
        // and a regular infinitesimal character give each block element a
        // singleton contribution.
        "KL_sum_at_s" | "KL_sum_at_s_to_height" => {
            let height_bound = if name == "KL_sum_at_s_to_height" {
                let bound = as_integer(&arguments[1], span)?;
                let narrowed = i32::try_from(&bound)
                    .map_err(|_| runtime(span, "Integer value to big for conversion"))?;
                if narrowed >= 0 {
                    Some(narrowed as u32)
                } else {
                    None
                }
            } else {
                None
            };
            let parameter = match &arguments[0] {
                Value::Domain(DomainValue::Param(parameter)) => parameter,
                _ => {
                    return Err(type_error(
                        span,
                        format!(
                            "{name} has no matching overload for {} argument(s)",
                            arguments.len()
                        ),
                    ))
                }
            };
            let terms = kl_sum_at_s_terms(parameter, span, height_bound)?;
            Ok(Value::Domain(DomainValue::ParamPol(ParamPolValue {
                rf: Arc::clone(&parameter.context),
                terms,
            })))
        }
        // raw_KL / dual_KL (atlas-types.w:9101-9102, 8603-8674): the KL
        // table of a block as (matrix of polynomial indices, polynomial
        // pool as coefficient vectors, length stops). dual_KL builds the
        // block on the SWAPPED real forms and pulls its polynomials through
        // blocks::dual_map (blocks.cpp:1715-1725): entry (x,y) is
        // KL_pol_index(dual[y], dual[x]) in the dual block's table, with
        // the pool of that dual table; the length stops stay the original
        // block's.
        "raw_KL" | "dual_KL" => {
            arity(name, arguments, 1, span)?;
            let Value::Domain(DomainValue::Block(block)) = &arguments[0] else {
                return Err(type_error(span, "expected a Block"));
            };
            let size = block.graph.size();
            // The dual table borrows the swapped block, which therefore
            // outlives it in this scope.
            let dual_block = if name == "dual_KL" {
                Some(build_block(&block.dual_rf, &block.rf, span)?)
            } else {
                None
            };
            let kl_graph = dual_block.as_ref().map_or(&block.graph, |dual| &dual.graph);
            let mut kl_table =
                KlTable::new(kl_graph).map_err(|error| structure_diagnostic(error, span))?;
            kl_table
                .fill(0)
                .map_err(|error| structure_diagnostic(error, span))?;
            let dual_permutation = match &dual_block {
                Some(dual) => {
                    let mut permutation = Vec::with_capacity(size);
                    for z in 0..size {
                        let x = block
                            .graph
                            .x(z)
                            .ok_or_else(|| runtime(span, "block element x lookup failed"))?;
                        let y = block
                            .graph
                            .y(z)
                            .ok_or_else(|| runtime(span, "block element y lookup failed"))?;
                        // dual_b.element(b.y(z), b.x(z)): swapped coordinates.
                        permutation.push(
                            dual.graph
                                .element(y, x)
                                .map_err(|error| structure_diagnostic(error, span))?,
                        );
                    }
                    Some(permutation)
                }
                None => None,
            };
            let mut columns = vec![vec![0_i32; size]; size];
            for (y, column) in columns.iter_mut().enumerate().skip(1) {
                for (x, slot) in column.iter_mut().enumerate().take(y) {
                    let (row, col) = match &dual_permutation {
                        // dual reverses the Bruhat order: y > x maps to
                        // dual[y] < dual[x] in the dual block.
                        Some(permutation) => (permutation[y], permutation[x]),
                        None => (x, y),
                    };
                    let index = kl_table
                        .kl_pol(row, col)
                        .map_err(|error| structure_diagnostic(error, span))?;
                    *slot = i32::try_from(index).map_err(|_| runtime(span, "KL index overflow"))?;
                }
            }
            // Diagonal entries P_{y,y} = 1 (index of the constant 1).
            for (y, column) in columns.iter_mut().enumerate() {
                let diagonal = match &dual_permutation {
                    Some(permutation) => permutation[y],
                    None => y,
                };
                let index = kl_table
                    .kl_pol(diagonal, diagonal)
                    .map_err(|error| structure_diagnostic(error, span))?;
                column[y] = i32::try_from(index).map_err(|_| runtime(span, "KL index overflow"))?;
            }
            let matrix = columns_matrix_value(&columns, size, span)?;
            // The polynomial pool: every stored polynomial's coefficient
            // vector, least degree first.
            let pool = kl_table.pool();
            let mut polys = Vec::new();
            for index in 0..pool.len() {
                let polynomial = pool.get(index).expect("in-range pool index");
                let mut coefficients = Vec::new();
                if !polynomial.is_zero() {
                    for degree in 0..=polynomial.degree() {
                        coefficients.push(polynomial.coefficient(degree));
                    }
                }
                polys.push(Value::Vector(Vec32(coefficients)));
            }
            let polys_value = Value::List(polys);
            // Length stops: [0, length_first(1), ..., length_first(max), size].
            let max_length = if size == 0 {
                0
            } else {
                block.graph.length(size - 1).unwrap_or_default()
            };
            let mut stops = vec![0_i32; max_length + 2];
            for (index, stop) in stops.iter_mut().enumerate().skip(1) {
                *stop = block_length_first(&block.graph, index) as i32;
            }
            let stops_value = Value::Vector(Vec32(stops));
            Ok(Value::Tuple(vec![matrix, polys_value, stops_value]))
        }
        // KL_column (atlas-types.w:6882-6905, repr.cpp:2060-2075): the
        // Kazhdan-Lusztig column of a final standard parameter, restricted
        // to the parameter's partial block (Bruhat-down closure filtered by
        // the singular coroots' survives).
        "KL_column" => {
            arity(name, arguments, 1, span)?;
            let Value::Domain(DomainValue::Param(parameter)) = &arguments[0] else {
                return Err(type_error(
                    span,
                    format!(
                        "{name} has no matching overload for {} argument(s)",
                        arguments.len()
                    ),
                ));
            };
            validate_kl_column(parameter, span)?;
            let rc = rep_context(&parameter.context);
            let normalised = parameter
                .repr
                .normalised(&rc)
                .map_err(|error| structure_diagnostic(error, span))?;
            match integral_block_scope(&rc, normalised.gamma())
                .map_err(|error| structure_diagnostic(error, span))?
            {
                IntegralBlockScope::Singleton => {
                    return Ok(Value::List(vec![Value::Tuple(vec![
                        Value::Integer(BigInt::from(0)),
                        Value::Domain(DomainValue::Param(ParamValue {
                            context: parameter.context.clone(),
                            repr: normalised,
                        })),
                        Value::Vector(Vec32(vec![1])),
                    ])]));
                }
                IntegralBlockScope::ProperSubsystem | IntegralBlockScope::Full => {}
            }
            let located = parameter
                .context
                .rep
                .lookup(&normalised)
                .map_err(|error| structure_diagnostic(error, span))?;
            if !located.has_identity_generator_attitude() {
                return Err(structure_diagnostic(
                    StructureError::NotYetImplemented {
                        feature: "KL_column on a non-identity integral-subsystem attitude",
                    },
                    span,
                ));
            }
            let raw_y = located.raw_row();
            let entries = located
                .with_kl_table(|kl_table| {
                    kl_table.fill(raw_y + 1)?;
                    let mut entries = Vec::new();
                    for raw_x in kl_column_candidate_rows(raw_y) {
                        let index = kl_table.kl_pol(raw_x, raw_y)?;
                        let polynomial = kl_table.pool().get(index).cloned().ok_or(
                            StructureError::RepInvariantViolation {
                                invariant: "representation KL polynomial pool index",
                            },
                        )?;
                        if polynomial.is_zero() {
                            continue;
                        }
                        let repr = located_row_parameter(&parameter.context, &located, raw_x)?;
                        entries.push((raw_x, repr, polynomial.as_slice().to_vec()));
                    }
                    Ok(entries)
                })
                .map_err(|error| structure_diagnostic(error, span))?;
            Ok(Value::List(
                entries
                    .into_iter()
                    .map(|(raw_x, repr, coefficients)| {
                        Value::Tuple(vec![
                            Value::Integer(BigInt::from(raw_x)),
                            Value::Domain(DomainValue::Param(ParamValue {
                                context: parameter.context.clone(),
                                repr,
                            })),
                            Value::Vector(Vec32(coefficients)),
                        ])
                    })
                    .collect(),
            ))
        }
        // KL_block (atlas-types.w:6868-6912, repr.cpp:2060-2075): the
        // condensed KL matrix over the parameter's common-block
        // survivors (lookup_full_block), plus the start index.
        "KL_block" => {
            arity(name, arguments, 1, span)?;
            let Value::Domain(DomainValue::Param(parameter)) = &arguments[0] else {
                return Err(type_error(
                    span,
                    format!(
                        "{name} has no matching overload for {} argument(s)",
                        arguments.len()
                    ),
                ));
            };
            test_standard(parameter, "KL_block requires a standard parameter", span)?;
            let rc = rep_context(&parameter.context);
            let dominant = parameter
                .repr
                .made_dominant(&rc)
                .map_err(|error| structure_diagnostic(error, span))?;
            match integral_block_scope(&rc, dominant.gamma())
                .map_err(|error| structure_diagnostic(error, span))?
            {
                IntegralBlockScope::Singleton => {
                    return Ok(Value::Tuple(vec![
                        Value::List(vec![Value::Domain(DomainValue::Param(ParamValue {
                            context: parameter.context.clone(),
                            repr: dominant,
                        }))]),
                        Value::Integer(BigInt::from(0)),
                        matrix_value(&[vec![1]], span)?,
                        Value::List(vec![
                            Value::Vector(Vec32(Vec::new())),
                            Value::Vector(Vec32(vec![1])),
                        ]),
                    ]));
                }
                IntegralBlockScope::ProperSubsystem | IntegralBlockScope::Full => {}
            }
            let located = parameter
                .context
                .rep
                .lookup_full_block(&dominant)
                .map_err(|error| structure_diagnostic(error, span))?;
            if !located.has_identity_generator_attitude() {
                return Err(structure_diagnostic(
                    StructureError::NotYetImplemented {
                        feature: "KL_block on a non-identity integral-subsystem attitude",
                    },
                    span,
                ));
            }
            let raw_start = located.raw_row();
            let block = located.block();
            let common =
                CommonContext::integral(&rc, located.adapted_representative().gamma_lambda())
                    .map_err(|error| structure_diagnostic(error, span))?;
            let singular_flags = common
                .singular_flags(located.prepared_query().gamma())
                .map_err(|error| structure_diagnostic(error, span))?;
            let singular = singular_flags
                .iter()
                .enumerate()
                .fold(0_u32, |bits, (s, &flag)| bits | (u32::from(flag) << s));
            let survivors: Vec<usize> = (0..block.size())
                .filter(|&raw| block.survives(raw, &singular_flags))
                .collect();
            let mut loc = vec![usize::MAX; block.size()];
            for (position, &raw) in survivors.iter().enumerate() {
                loc[raw] = position;
            }
            let n = survivors.len();
            let matrix = located
                .with_kl_table(|kl_table| {
                    kl_table.fill(0)?;
                    let mut matrix = vec![vec![KlPol::zero(); n]; n];
                    for raw_x in 0..block.size() {
                        for final_raw in partial_block_finals_for(&block, raw_x, singular)? {
                            let i = loc[final_raw];
                            if i == usize::MAX {
                                continue;
                            }
                            let raw_length = block.length(raw_x).ok_or(
                                StructureError::RepInvariantViolation {
                                    invariant: "common block KL source length",
                                },
                            )?;
                            let final_length = block.length(final_raw).ok_or(
                                StructureError::RepInvariantViolation {
                                    invariant: "common block KL final length",
                                },
                            )?;
                            let sign_even =
                                (raw_length as i64 - final_length as i64).rem_euclid(2) == 0;
                            for (j, &raw_y) in survivors.iter().enumerate() {
                                if raw_y < raw_x || i >= j {
                                    continue;
                                }
                                let index = kl_table.kl_pol(raw_x, raw_y)?;
                                let polynomial = kl_table.pool().get(index).ok_or(
                                    StructureError::RepInvariantViolation {
                                        invariant: "representation KL polynomial pool index",
                                    },
                                )?;
                                matrix[i][j] = if sign_even {
                                    matrix[i][j].add(polynomial)
                                } else {
                                    matrix[i][j].sub(polynomial)
                                };
                            }
                        }
                    }
                    Ok(matrix)
                })
                .map_err(|error| structure_diagnostic(error, span))?;
            // Upstream's condensed store reserves zero and one before any
            // strict-upper entry is interned; the matrix starts as identity.
            let mut polys: Vec<KlPol> = vec![KlPol::zero(), KlPol::monomial(0)];
            let mut index_of: std::collections::HashMap<Vec<i32>, usize> =
                std::collections::HashMap::new();
            index_of.insert(Vec::new(), 0);
            index_of.insert(vec![1], 1);
            let mut index_matrix = vec![vec![0_usize; n]; n];
            for (row, values) in index_matrix.iter_mut().enumerate() {
                values[row] = 1;
                for column in row + 1..n {
                    let coefficients = matrix[row][column].as_slice().to_vec();
                    let index = *index_of.entry(coefficients.clone()).or_insert_with(|| {
                        polys.push(matrix[row][column].clone());
                        polys.len() - 1
                    });
                    values[column] = index;
                }
            }
            // Parameters of the survivors.
            let mut params = Vec::new();
            for &raw in &survivors {
                let repr = located_row_parameter(&parameter.context, &located, raw)
                    .map_err(|error| structure_diagnostic(error, span))?;
                params.push(Value::Domain(DomainValue::Param(ParamValue {
                    context: parameter.context.clone(),
                    repr,
                })));
            }
            let rows: Vec<Vec<i32>> = index_matrix
                .iter()
                .map(|row| row.iter().map(|&index| index as i32).collect())
                .collect();
            let matrix_value = matrix_value(&rows, span)?;
            let polys_value = Value::List(
                polys
                    .iter()
                    .map(|pol| Value::Vector(Vec32(pol.as_slice().to_vec())))
                    .collect(),
            );
            let start_value = Value::Integer(BigInt::from(if loc[raw_start] == usize::MAX {
                -1
            } else {
                loc[raw_start] as i64
            }));
            Ok(Value::Tuple(vec![
                Value::List(params),
                start_value,
                matrix_value,
                polys_value,
            ]))
        }
        // dual_KL_block (atlas-types.w:7053-7133, blocks.cpp:474-509
        // `Bare_block::dual`): the KL matrix of the dual block over the
        // parameter's common-block survivors, with no condensing; the
        // polynomial pool is seeded with {0, 1} and the matrix entry
        // M[loc[x]][loc[y]] is the pool index of the dual polynomial
        // P_{last-x,last-y}.
        "dual_KL_block" => {
            arity(name, arguments, 1, span)?;
            let Value::Domain(DomainValue::Param(parameter)) = &arguments[0] else {
                return Err(type_error(
                    span,
                    format!(
                        "{name} has no matching overload for {} argument(s)",
                        arguments.len()
                    ),
                ));
            };
            let dual_parent = build_dual_inner_class(&parameter.context.parent, span)?;
            let dual_quasisplit = dual_parent.order.quasisplit_external();
            let dual_rf = build_real_form(&dual_parent, dual_quasisplit, span)?;
            let block = build_block(&parameter.context, &dual_rf, span)?;
            let dual_graph = block.graph.dual();
            let mut dual_kl =
                KlTable::new(&dual_graph).map_err(|error| structure_diagnostic(error, span))?;
            dual_kl
                .fill(0)
                .map_err(|error| structure_diagnostic(error, span))?;
            let size = block.graph.size();
            let datum = parameter.context.parent.root_datum.datum.clone();
            let rc = rep_context(&parameter.context);
            let lambda_rho = rc
                .lambda_rho(&parameter.repr)
                .map_err(|error| structure_diagnostic(error, span))?;
            let gamma = parameter.repr.gamma().clone();
            let z0 = (0..size)
                .find(|&z| block.graph.x(z) == Some(parameter.repr.x()))
                .ok_or_else(|| runtime(span, "parameter not in the common block"))?;
            // The common block: for integral gamma it is the full block
            // (upstream then considers every element for survival,
            // atlas-types.w:7093-7107) and per-element gamma_lambda values
            // come from the z_pool propagation. For non-integral gamma
            // upstream builds the block on the proper integral subsystem;
            // that slice is deferred, so the parity-gated closure with the
            // uniform seed lambda_rho is kept there (the rank-0 integral
            // systems exercised so far make both coincide).
            let integral_gamma = gamma_is_integral(&datum, &gamma);
            let srms = if integral_gamma {
                common_block_srms(&block, z0, &rc, &lambda_rho, &gamma, span)?
            } else {
                Vec::new()
            };
            let members = if integral_gamma {
                vec![true; size]
            } else {
                common_block_members(&block, z0, &rc, &lambda_rho, &gamma, span)?
            };
            // Singular coroots: <gamma, alpha_s^vee> vanishes.
            let mut singular = 0_u32;
            for s in 0..datum.semisimple_rank() {
                let coroot = &datum.simple_coroots()[s];
                let numerator = gamma.numerator();
                let _denominator = gamma.denominator();
                let mut total: i64 = 0;
                for (index, &coordinate) in coroot.as_slice().iter().enumerate() {
                    if coordinate == 0 {
                        continue;
                    }
                    let entry = numerator
                        .get(index)
                        .ok_or_else(|| runtime(span, "rational weight rank"))?;
                    total += i64::from(coordinate) * *entry;
                }
                if total == 0 {
                    singular |= 1 << s;
                }
            }
            // Survivors of the common block (loc[z] = survivors.size()),
            // computed on the original block.
            let mut loc = vec![usize::MAX; size];
            let mut survivors: Vec<usize> = Vec::new();
            for z in 0..size {
                if !members[z] {
                    continue;
                }
                let mut survives = true;
                for s in 0..datum.semisimple_rank() {
                    if singular & (1 << s) != 0
                        && block
                            .graph
                            .descent_value(z, s)
                            .is_some_and(|d| d.is_descent())
                    {
                        survives = false;
                        break;
                    }
                }
                if survives {
                    loc[z] = survivors.len();
                    survivors.push(z);
                }
            }
            let n = survivors.len();
            let last = size - 1;
            // The pool starts with the zero polynomial and the constant 1
            // (atlas-types.w:7113-7133). Filled column-major over the
            // survivors (y outer, x from y on) so that hash.match assigns
            // pool indices in the oracle's order.
            let mut polys: Vec<KlPol> = vec![KlPol::zero(), KlPol::monomial(0)];
            let mut index_of: std::collections::HashMap<Vec<i32>, usize> =
                std::collections::HashMap::new();
            index_of.insert(Vec::new(), 0);
            index_of.insert(vec![1], 1);
            let mut index_matrix = vec![vec![0_usize; n]; n];
            for (j, &y) in survivors.iter().enumerate() {
                for (i, &x) in survivors.iter().enumerate().skip(j) {
                    let polynomial = kl_pol_at(&dual_kl, last - x, last - y, span)?;
                    let coefficients = polynomial.as_slice().to_vec();
                    let index = *index_of.entry(coefficients).or_insert_with(|| {
                        polys.push(polynomial.clone());
                        polys.len() - 1
                    });
                    index_matrix[i][j] = index;
                }
            }
            // Parameters of the survivors, in original-block order:
            // rc.sr(z_pool[z], bm, gamma) with trivial bm (atlas-types.w
            // shared chunk at :6951-6962).
            let gamma_rho = gamma
                .sub(rc.rho())
                .map_err(|error| structure_diagnostic(error, span))?;
            let mut params = Vec::new();
            for &z in &survivors {
                let lambda_rho_z = if integral_gamma {
                    let gl = srms[z]
                        .as_ref()
                        .ok_or_else(|| runtime(span, "survivor outside the common block"))?;
                    integer_diff_weight(&gamma_rho, gl, span)?
                } else {
                    lambda_rho.clone()
                };
                let sr = rc
                    .sr_gamma(block.graph.x(z).expect("in-range"), &lambda_rho_z, &gamma)
                    .map_err(|error| structure_diagnostic(error, span))?;
                params.push(Value::Domain(DomainValue::Param(ParamValue {
                    context: parameter.context.clone(),
                    repr: sr,
                })));
            }
            let rows: Vec<Vec<i32>> = index_matrix
                .iter()
                .map(|row| row.iter().map(|&index| index as i32).collect())
                .collect();
            let matrix_value = matrix_value(&rows, span)?;
            let polys_value = Value::List(
                polys
                    .iter()
                    .map(|pol| Value::Vector(Vec32(pol.as_slice().to_vec())))
                    .collect(),
            );
            let start_value = Value::Integer(BigInt::from(if loc[z0] == usize::MAX {
                -1
            } else {
                loc[z0] as i64
            }));
            Ok(Value::Tuple(vec![
                Value::List(params),
                start_value,
                matrix_value,
                polys_value,
            ]))
        }
        // partial_block (atlas-types.w:6786-6820, repr.cpp:1796-1824):
        // the returned block may be larger after an earlier full lookup, so
        // restrict it to the start element's Bruhat downset before applying
        // the singular-coroot survivor filter.
        "partial_block" => {
            arity(name, arguments, 1, span)?;
            let Value::Domain(DomainValue::Param(parameter)) = &arguments[0] else {
                return Err(type_error(
                    span,
                    format!(
                        "{name} has no matching overload for {} argument(s)",
                        arguments.len()
                    ),
                ));
            };
            test_standard(parameter, "Cannot generate block", span)?;
            let rc = rep_context(&parameter.context);
            let located = parameter
                .context
                .rep
                .lookup(&parameter.repr)
                .map_err(|error| structure_diagnostic(error, span))?;
            if !located.has_identity_generator_attitude() {
                return Err(runtime(
                    span,
                    "partial block on a non-identity integral-subsystem attitude is not yet supported",
                ));
            }
            let block = located.block();
            let hasse = block_bruhat_hasse(block.as_ref());
            let mut subset = vec![false; block.size()];
            let mut stack = vec![located.raw_row()];
            subset[located.raw_row()] = true;
            while let Some(z) = stack.pop() {
                for &down in &hasse[z] {
                    if !subset[down] {
                        subset[down] = true;
                        stack.push(down);
                    }
                }
            }
            let common =
                CommonContext::integral(&rc, located.adapted_representative().gamma_lambda())
                    .map_err(|error| structure_diagnostic(error, span))?;
            let singular_flags = common
                .singular_flags(located.prepared_query().gamma())
                .map_err(|error| structure_diagnostic(error, span))?;
            let mut params = Vec::new();
            for (z, &in_downset) in subset.iter().enumerate() {
                if in_downset && block.survives(z, &singular_flags) {
                    let repr = located_row_parameter(&parameter.context, &located, z)
                        .map_err(|error| structure_diagnostic(error, span))?;
                    params.push(Value::Domain(DomainValue::Param(ParamValue {
                        context: parameter.context.clone(),
                        repr,
                    })));
                }
            }
            Ok(Value::List(params))
        }
        // full_deform (atlas-types.w:8213-8227, repr.cpp:2251-2290): the
        // full K-type deformation of a final standard parameter: the
        // finals of its scale-0 parameter, plus the deformation terms of
        // each reducibility point, merged into a K-type polynomial.
        "full_deform" => {
            if arguments.is_empty() || arguments.len() > 2 {
                return Err(type_error(
                    span,
                    format!(
                        "{name} has no matching overload for {} argument(s)",
                        arguments.len()
                    ),
                ));
            }
            let Value::Domain(DomainValue::Param(parameter)) = &arguments[0] else {
                return Err(type_error(
                    span,
                    format!(
                        "{name} has no matching overload for {} argument(s)",
                        arguments.len()
                    ),
                ));
            };
            let key = full_deform_key(&parameter.repr);
            let cached = cached_deformation(&parameter.context.full_deform_cache, &key, span)?;
            let make_polynomial = |terms: Vec<(SplitValue, KType)>| {
                Value::Domain(DomainValue::KTypePol(KTypePolValue {
                    rf: Arc::clone(&parameter.context),
                    terms,
                }))
            };

            if arguments.len() == 1 {
                let terms = match cached {
                    Some(terms) => terms,
                    None => {
                        let terms = compute_full_deform(parameter, span, None)?
                            .expect("an unbounded deformation cannot time out");
                        store_deformation(
                            &parameter.context.full_deform_cache,
                            key,
                            terms.clone(),
                            span,
                        )?;
                        terms
                    }
                };
                return Ok(make_polynomial(terms));
            }

            let timer = i32::try_from(&as_integer(&arguments[1], span)?)
                .map_err(|_| runtime(span, "Integer value to big for conversion"))?;
            if let Some(terms) = cached {
                return Ok(Value::Union {
                    tag: 1,
                    injector_name: "done".into(),
                    value: Box::new(make_polynomial(terms)),
                });
            }
            if timer <= 0 {
                return Ok(Value::Union {
                    tag: 0,
                    injector_name: "timed_out".into(),
                    value: Box::new(Value::Tuple(Vec::new())),
                });
            }
            let deadline = Instant::now().checked_add(Duration::from_millis(timer as u64));
            let Some(terms) = compute_full_deform(parameter, span, deadline)? else {
                return Ok(Value::Union {
                    tag: 0,
                    injector_name: "timed_out".into(),
                    value: Box::new(Value::Tuple(Vec::new())),
                });
            };
            if deadline_expired(deadline) {
                return Ok(Value::Union {
                    tag: 0,
                    injector_name: "timed_out".into(),
                    value: Box::new(Value::Tuple(Vec::new())),
                });
            }
            store_deformation(
                &parameter.context.full_deform_cache,
                key,
                terms.clone(),
                span,
            )?;
            Ok(Value::Union {
                tag: 1,
                injector_name: "done".into(),
                value: Box::new(make_polynomial(terms)),
            })
        }
        // partial_KL_block (atlas-types.w:6998-7051, repr.cpp:2060-2075):
        // the condensed KL matrix over the parameter's partial block
        // survivors, plus the parameter and polynomial lists.
        "partial_KL_block" => {
            arity(name, arguments, 1, span)?;
            let Value::Domain(DomainValue::Param(parameter)) = &arguments[0] else {
                return Err(type_error(
                    span,
                    format!(
                        "{name} has no matching overload for {} argument(s)",
                        arguments.len()
                    ),
                ));
            };
            let dual_parent = build_dual_inner_class(&parameter.context.parent, span)?;
            let dual_quasisplit = dual_parent.order.quasisplit_external();
            let dual_rf = build_real_form(&dual_parent, dual_quasisplit, span)?;
            let block = build_block(&parameter.context, &dual_rf, span)?;
            let mut kl_table =
                KlTable::new(&block.graph).map_err(|error| structure_diagnostic(error, span))?;
            kl_table
                .fill(0)
                .map_err(|error| structure_diagnostic(error, span))?;
            let size = block.graph.size();
            let datum = parameter.context.parent.root_datum.datum.clone();
            let rc = rep_context(&parameter.context);
            let lambda_rho = rc
                .lambda_rho(&parameter.repr)
                .map_err(|error| structure_diagnostic(error, span))?;
            let gamma = parameter.repr.gamma().clone();
            let z0 = (0..size)
                .find(|&z| block.graph.x(z) == Some(parameter.repr.x()))
                .ok_or_else(|| runtime(span, "parameter not in the common block"))?;
            // Partial block: the KL descent closure of z0 (block_below).
            let mut subset: Vec<bool> = vec![false; size];
            let mut stack = vec![z0];
            subset[z0] = true;
            while let Some(z) = stack.pop() {
                let z_x = block.graph.x(z).expect("in-range");
                for s in 0..datum.semisimple_rank() {
                    match block.graph.descent_value(z, s) {
                        Some(BlockDescent::ComplexDescent) => {
                            if let Some(target) = block.graph.cross(z, s) {
                                if !subset[target] {
                                    subset[target] = true;
                                    stack.push(target);
                                }
                            }
                        }
                        Some(BlockDescent::RealTypeI) => {
                            let parity = rc
                                .is_parity(s, z_x, &lambda_rho, &gamma)
                                .map_err(|error| structure_diagnostic(error, span))?;
                            if !parity {
                                continue;
                            }
                            if let Some(pair) = block.graph.inverse_cayley(z, s) {
                                for target in [pair.0, pair.1].into_iter().flatten() {
                                    if !subset[target] {
                                        subset[target] = true;
                                        stack.push(target);
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            // Singular coroots: <gamma, alpha_s^vee> non-integral.
            let mut singular = 0_u32;
            for s in 0..datum.semisimple_rank() {
                let coroot = &datum.simple_coroots()[s];
                let numerator = gamma.numerator();
                let _denominator = gamma.denominator();
                let mut total: i64 = 0;
                for (index, &coordinate) in coroot.as_slice().iter().enumerate() {
                    if coordinate == 0 {
                        continue;
                    }
                    let entry = numerator
                        .get(index)
                        .ok_or_else(|| runtime(span, "rational weight rank"))?;
                    total += i64::from(coordinate) * *entry;
                }
                if total == 0 {
                    singular |= 1 << s;
                }
            }
            // Survivors in subset order (loc[z] = survivors.size()).
            let mut loc = vec![usize::MAX; size];
            let mut survivors: Vec<usize> = Vec::new();
            for z in 0..size {
                if !subset[z] {
                    continue;
                }
                let mut survives = true;
                for s in 0..datum.semisimple_rank() {
                    if singular & (1 << s) != 0
                        && block
                            .graph
                            .descent_value(z, s)
                            .is_some_and(|d| d.is_descent())
                    {
                        survives = false;
                        break;
                    }
                }
                if survives {
                    loc[z] = survivors.len();
                    survivors.push(z);
                }
            }
            let n = survivors.len();
            // Condense the KL polynomials into M (atlas-types.w:6922-6948):
            // M(loc[f], loc[y]) +=/-= KL_pol(x, y) over the finals of x.
            let mut matrix: Vec<Vec<KlPol>> = vec![vec![KlPol::zero(); n]; n];
            for x in 0..size {
                for f in block_finals_for(&block, x, singular, &kl_table, span)? {
                    let i = loc[f];
                    if i == usize::MAX {
                        continue;
                    }
                    let sign_even = block.graph.length(x).is_some_and(|lx| {
                        (lx as i64 - block.graph.length(f).unwrap_or(0) as i64).rem_euclid(2) == 0
                    });
                    for (j, &y) in survivors.iter().enumerate() {
                        let polynomial = kl_pol_at(&kl_table, x, y, span)?;
                        if polynomial.is_zero() {
                            continue;
                        }
                        if sign_even {
                            matrix[i][j] = matrix[i][j].add(&polynomial);
                        } else {
                            matrix[i][j] = matrix[i][j].sub(&polynomial);
                        }
                    }
                }
            }
            // Distinct polynomials: the oracle's store starts with the zero
            // polynomial at index 0.
            let mut polys: Vec<KlPol> = vec![KlPol::zero()];
            let mut index_of: std::collections::HashMap<Vec<i32>, usize> =
                std::collections::HashMap::new();
            index_of.insert(Vec::new(), 0);
            let mut index_matrix = vec![vec![0_usize; n]; n];
            for row in 0..n {
                for column in 0..n {
                    let coefficients = matrix[row][column].as_slice().to_vec();
                    let index = *index_of.entry(coefficients.clone()).or_insert_with(|| {
                        polys.push(matrix[row][column].clone());
                        polys.len() - 1
                    });
                    index_matrix[row][column] = index;
                }
            }
            // Parameters of the survivors.
            let mut params = Vec::new();
            for &z in &survivors {
                let sr = rc
                    .sr_gamma(block.graph.x(z).expect("in-range"), &lambda_rho, &gamma)
                    .map_err(|error| structure_diagnostic(error, span))?;
                params.push(Value::Domain(DomainValue::Param(ParamValue {
                    context: parameter.context.clone(),
                    repr: sr,
                })));
            }
            let rows: Vec<Vec<i32>> = index_matrix
                .iter()
                .map(|row| row.iter().map(|&index| index as i32).collect())
                .collect();
            let matrix_value = matrix_value(&rows, span)?;
            let polys_value = Value::List(
                polys
                    .iter()
                    .map(|pol| Value::Vector(Vec32(pol.as_slice().to_vec())))
                    .collect(),
            );
            Ok(Value::Tuple(vec![
                Value::List(params),
                matrix_value,
                polys_value,
            ]))
        }
        // W_graph / W_cells (atlas-types.w:7147-7170, 7210-7245): the
        // W-graph of a standard parameter's full block, respectively its
        // cell decomposition. W_graph returns (start, vertices) with each
        // vertex a (descent set, [(target, coefficient)]) pair; W_cells
        // returns (start, [(members, vertices)]).
        "W_graph" | "W_cells" => {
            arity(name, arguments, 1, span)?;
            if let Value::Domain(DomainValue::Block(block)) = &arguments[0] {
                // The Block overloads return the graph/cell list itself;
                // unlike the Param overloads there is no start index.
                return block_w_graph_value(name, block, span);
            }
            let Value::Domain(DomainValue::Param(parameter)) = &arguments[0] else {
                return Err(type_error(
                    span,
                    format!(
                        "{name} has no matching overload for {} argument(s)",
                        arguments.len()
                    ),
                ));
            };
            test_standard(parameter, "Cannot generate block", span)?;
            // param_W_graph/param_W_cells (atlas-types.w:7143-7205) consume
            // the full common block returned by Rep_table::lookup_full_block.
            // Its PartialBlock topology is already expressed in integral-
            // subsystem generator numbering, including imaginary grading.
            let located = parameter
                .context
                .rep
                .lookup_full_block(&parameter.repr)
                .map_err(|error| structure_diagnostic(error, span))?;
            if !located.has_identity_generator_attitude() {
                return Err(runtime(
                    span,
                    "W-graph on a non-identity integral-subsystem attitude is not yet supported",
                ));
            }
            let start = located.raw_row();
            let block = located.block();
            let vertex_count = block.size();
            let (descent_sets, edges) = located
                .with_kl_table(|kl_table| {
                    kl_table.fill(0)?;
                    let descent_sets: Vec<BTreeSet<usize>> = (0..vertex_count)
                        .map(|z| {
                            let descents = kl_table.support().descent_set(z);
                            (0..block.rank())
                                .filter(|&generator| descents.is_set(generator))
                                .collect::<BTreeSet<_>>()
                        })
                        .collect();
                    // kl::wGraph (kl.cpp:1042-1058): every mu pair adds an
                    // edge in both directions in common-block numbering.
                    let mut edges: Vec<Vec<(usize, i32)>> = vec![Vec::new(); vertex_count];
                    for y in 0..vertex_count {
                        for pair in kl_table.mu_column(y) {
                            edges[y].push((pair.x, pair.coef));
                            edges[pair.x].push((y, pair.coef));
                        }
                    }
                    for targets in &mut edges {
                        targets.sort_unstable();
                    }
                    Ok((descent_sets, edges))
                })
                .map_err(|error| structure_diagnostic(error, span))?;
            let vertex = |element: usize, targets: &[(usize, i32)]| -> Value {
                let descents = Value::List(
                    descent_sets[element]
                        .iter()
                        .map(|&generator| Value::Integer(BigInt::from(generator)))
                        .collect(),
                );
                let out_edges = Value::List(
                    targets
                        .iter()
                        .map(|&(target, coef)| {
                            Value::Tuple(vec![
                                Value::Integer(BigInt::from(target)),
                                Value::Integer(BigInt::from(coef)),
                            ])
                        })
                        .collect(),
                );
                Value::Tuple(vec![descents, out_edges])
            };
            if name == "W_graph" {
                let vertices = Value::List(
                    edges
                        .iter()
                        .enumerate()
                        .map(|(element, targets)| vertex(element, targets))
                        .collect(),
                );
                Ok(Value::Tuple(vec![
                    Value::Integer(BigInt::from(start)),
                    vertices,
                ]))
            } else {
                // DecomposedWGraph (wgraph.cpp:58-116): the oriented graph's
                // strong components.
                let mut oriented: Vec<Vec<usize>> = Vec::with_capacity(vertex_count);
                for (x, edges_x) in edges.iter().enumerate() {
                    let mut targets = Vec::new();
                    for &(y, _) in edges_x {
                        if !descent_sets[y].is_superset(&descent_sets[x]) {
                            targets.push(y);
                        }
                    }
                    oriented.push(targets);
                }
                let (mut partition, _induced) = strong_components(&oriented);
                // The oracle's Partition lists a cell's vertices ascending.
                for members in partition.iter_mut() {
                    members.sort_unstable();
                }
                let mut cells = Vec::new();
                for members in &partition {
                    let mut relno = vec![0_usize; vertex_count];
                    for (position, &member) in members.iter().enumerate() {
                        relno[member] = position;
                    }
                    let cell_members: std::collections::BTreeSet<usize> =
                        members.iter().copied().collect();
                    let vertices_list = Value::List(
                        members
                            .iter()
                            .map(|&member| {
                                let cell_edges: Vec<(usize, i32)> = edges[member]
                                    .iter()
                                    .copied()
                                    .filter(|&(target, _)| cell_members.contains(&target))
                                    .map(|(target, coef)| (relno[target], coef))
                                    .collect();
                                vertex(member, &cell_edges)
                            })
                            .collect(),
                    );
                    cells.push(Value::Tuple(vec![
                        Value::List(
                            members
                                .iter()
                                .map(|&member| Value::Integer(BigInt::from(member)))
                                .collect(),
                        ),
                        vertices_list,
                    ]));
                }
                Ok(Value::Tuple(vec![
                    Value::Integer(BigInt::from(start)),
                    Value::List(cells),
                ]))
            }
        }
        "block_Hasse" => {
            arity(name, arguments, 1, span)?;
            let Value::Domain(DomainValue::Param(parameter)) = &arguments[0] else {
                return Err(type_error(
                    span,
                    format!(
                        "{name} has no matching overload for {} argument(s)",
                        arguments.len()
                    ),
                ));
            };
            test_standard(parameter, "Cannot generate block", span)?;
            let located = parameter
                .context
                .rep
                .lookup_full_block(&parameter.repr)
                .map_err(|error| structure_diagnostic(error, span))?;
            if !located.has_identity_generator_attitude() {
                return Err(runtime(
                    span,
                    "block Hasse diagram on a non-identity integral-subsystem attitude is not yet supported",
                ));
            }
            let block = located.block();
            let n = block.size();
            let mut param_list = Vec::with_capacity(n);
            for z in 0..n {
                let repr = located_row_parameter(&parameter.context, &located, z)
                    .map_err(|error| structure_diagnostic(error, span))?;
                param_list.push(Value::Domain(DomainValue::Param(ParamValue {
                    context: Arc::clone(&parameter.context),
                    repr,
                })));
            }
            let hasse = block_bruhat_hasse(block.as_ref());
            let mut columns = vec![vec![0_i32; n]; n];
            for (position, downs) in hasse.iter().enumerate() {
                for &down in downs {
                    columns[position][down] = 1;
                }
            }
            Ok(Value::Tuple(vec![
                Value::List(param_list),
                columns_matrix_value(&columns, n, span)?,
            ]))
        }
        // shift_flip (atlas-types.w:7341-7362): whether the default
        // extension of the parameter, shifted to the given rational weight,
        // is opposite to the default extension at that weight. No
        // test_standard/test_final: the three `shift_flip_gates` checks
        // run in the upstream order, then the work is
        // Ext_rep_context + shifted_default_extension + is_default
        // (repr.h:682-714, ext_block.h:352-364).
        "shift_flip" => {
            arity(name, arguments, 3, span)?;
            let Value::Domain(DomainValue::Param(parameter)) = &arguments[0] else {
                return Err(type_error(
                    span,
                    format!(
                        "{name} has no matching overload for {} argument(s)",
                        arguments.len()
                    ),
                ));
            };
            let (delta, gamma) = shift_flip_gates(parameter, &arguments[1], &arguments[2], span)?;
            let rc = rep_context(&parameter.context);
            let context = ExtRepContext::new(&rc, delta)
                .map_err(|error| structure_diagnostic(error, span))?;
            let extension = shifted_default_extension(&context, &parameter.repr, &gamma)
                .map_err(|error| structure_diagnostic(error, span))?;
            let flipped = !is_default(&context, &extension)
                .map_err(|error| structure_diagnostic(error, span))?;
            Ok(Value::Boolean(flipped))
        }
        // extended_block (atlas-types.w:7366-7431), raw_ext_KL
        // (atlas-types.w:8682-8728), partial_extended_KL_block
        // (atlas-types.w:7445-7468, ext_kl.cpp:939-1018): the extended
        // block of a standard parameter, its raw KLV table, and its
        // condensed partial KLV matrix. All three build the parameter's
        // common block as a fiber of the full block, then the extended
        // block over it; `extended_block` uses the inner class's
        // DISTINGUISHED involution for the construction (the user's delta
        // only gates the gamma-fix test), the KLV wrappers use the user's
        // delta.
        "extended_block" | "raw_ext_KL" | "partial_extended_KL_block" => {
            arity(name, arguments, 2, span)?;
            let Value::Domain(DomainValue::Param(parameter)) = &arguments[0] else {
                return Err(type_error(
                    span,
                    format!(
                        "{name} has no matching overload for {} argument(s)",
                        arguments.len()
                    ),
                ));
            };
            // test_standard (atlas-types.w:6605-6611): each wrapper passes
            // its own description string.
            let descr = if name == "partial_extended_KL_block" {
                "Cannot generate extended block"
            } else {
                "Cannot generate block"
            };
            test_standard(parameter, descr, span)?;
            // test_compatible (atlas-types.w:4627-4633).
            let (delta, twist) = compatible_outer_twist(&parameter.context, &arguments[1], span)?;
            let gamma = parameter.repr.gamma().clone();
            if !involution_fixes_gamma(delta.weight_matrix(), &gamma) {
                match name {
                    "extended_block" => {
                        // atlas-types.w:7372-7373.
                        return Err(runtime(
                            span,
                            "Involution does not fix infinitesimal character",
                        ));
                    }
                    "raw_ext_KL" => {
                        // Block not globally stable: empty values
                        // (atlas-types.w:8697-8702).
                        return Ok(Value::Tuple(vec![
                            matrix_value(&[], span)?,
                            Value::List(Vec::new()),
                            Value::Vector(Vec32(Vec::new())),
                        ]));
                    }
                    _ => {
                        // ext_kl.cpp:945-948: the report goes to standard
                        // output before the error is thrown.
                        printed.push(format!(
                            "Delta does not fix gamma={}.\n",
                            rational_weight_display(&gamma)
                        ));
                        return Err(runtime(span, "No valid extended block"));
                    }
                }
            }
            let datum = parameter.context.parent.root_datum.datum.clone();
            if !gamma_is_integral(&datum, &gamma) {
                return Err(runtime(
                    span,
                    format!(
                        "{name} at a non-integral infinitesimal character is not yet implemented"
                    ),
                ));
            }
            let dual_parent = build_dual_inner_class(&parameter.context.parent, span)?;
            let dual_quasisplit = dual_parent.order.quasisplit_external();
            let dual_rf = build_real_form(&dual_parent, dual_quasisplit, span)?;
            let block = build_block(&parameter.context, &dual_rf, span)?;
            let rc = rep_context(&parameter.context);
            let lambda_rho = rc
                .lambda_rho(&parameter.repr)
                .map_err(|error| structure_diagnostic(error, span))?;
            let z0 = (0..block.graph.size())
                .find(|&z| block.graph.x(z) == Some(parameter.repr.x()))
                .ok_or_else(|| runtime(span, "parameter not in the common block"))?;
            let members = common_block_members(&block, z0, &rc, &lambda_rho, &gamma, span)?;
            // The per-element gamma_lambda field, computed directly from
            // each member's (x, y) pair.
            let gamma_lambdas = common_block_gamma_lambdas(&block, &members, &rc, &gamma, span)?;
            let gamma_rho = gamma
                .sub(rc.rho())
                .map_err(|error| structure_diagnostic(error, span))?;
            // The per-element parameters of the fiber (atlas-types.w:
            // 7398-7404, ext_kl.cpp:1005-1007).
            let fiber_param = |z: usize,
                               gamma_lambdas: &Vec<Option<RationalWeight>>|
             -> Result<Value, Diagnostic> {
                let gl = gamma_lambdas[z]
                    .as_ref()
                    .expect("fiber elements carry gamma_lambda");
                let lambda_rho_z = integer_diff_weight(&gamma_rho, gl, span)?;
                let repr = rc
                    .sr_gamma(block.graph.x(z).expect("in-range"), &lambda_rho_z, &gamma)
                    .map_err(|error| structure_diagnostic(error, span))?;
                Ok(Value::Domain(DomainValue::Param(ParamValue {
                    context: Arc::clone(&parameter.context),
                    repr,
                })))
            };
            match name {
                "extended_block" => {
                    // Upstream constructs over the distinguished involution
                    // (atlas-types.w:7392), ignoring the user's delta.
                    let (eff_delta, eff_twist) = distinguished_twist(parameter, span)?;
                    let eb = build_ext_block(&block, parameter, &eff_delta, &eff_twist, span)?;
                    let (fiber, loc) = ext_fiber(&eb, &members);
                    let size = fiber.len();
                    let signed = |eb: &ExtBlock, s: usize, n: usize, link: Option<usize>| -> i32 {
                        match link {
                            None => size as i32,
                            Some(m) => {
                                let mapped = loc[m];
                                debug_assert!(mapped != usize::MAX, "links stay in the fiber");
                                let mapped = if mapped == usize::MAX { size } else { mapped };
                                if eb.epsilon(s, n, m) < 0 {
                                    -1 - mapped as i32
                                } else {
                                    mapped as i32
                                }
                            }
                        }
                    };
                    let mut params = Vec::with_capacity(size);
                    let mut types = vec![vec![0_i32; eb.rank()]; size];
                    let mut links0 = vec![vec![0_i32; eb.rank()]; size];
                    let mut links1 = vec![vec![0_i32; eb.rank()]; size];
                    for (new_n, &n) in fiber.iter().enumerate() {
                        params.push(fiber_param(eb.z(n), &gamma_lambdas)?);
                        for s in 0..eb.rank() {
                            let kind = eb.descent_type(s, n);
                            types[new_n][s] = kind as usize as i32;
                            // The wrapper encoding (atlas-types.w:7405-7424).
                            if kind.is_like_compact() || kind.is_like_nonparity() {
                                links0[new_n][s] = size as i32;
                                links1[new_n][s] = size as i32;
                            } else {
                                let first = if kind.is_complex() {
                                    eb.cross(s, n)
                                } else {
                                    eb.cayley(s, n)
                                };
                                links0[new_n][s] = signed(&eb, s, n, first);
                                if kind.link_count() == 1 {
                                    links1[new_n][s] = size as i32;
                                } else {
                                    let second = if kind.has_double_image() {
                                        eb.cayleys(s, n).1
                                    } else {
                                        eb.cross(s, n)
                                    };
                                    links1[new_n][s] = signed(&eb, s, n, second);
                                }
                            }
                        }
                    }
                    Ok(Value::Tuple(vec![
                        Value::List(params),
                        matrix_value(&types, span)?,
                        matrix_value(&links0, span)?,
                        matrix_value(&links1, span)?,
                    ]))
                }
                "raw_ext_KL" => {
                    let eb = build_ext_block(&block, parameter, &delta, &twist, span)?;
                    let (fiber, loc) = ext_fiber(&eb, &members);
                    let size = fiber.len();
                    let mut table =
                        ExtKlTable::new(&eb).map_err(|e| structure_diagnostic(e, span))?;
                    table
                        .fill_columns(0)
                        .map_err(|e| structure_diagnostic(e, span))?;
                    // Rebuild the printed pool in insertion order (columns
                    // ascending, each from the back); the crate's pool may
                    // order multi-fiber recursion intermediates
                    // differently. Entries 0 and 1 are the primed zero and
                    // one polynomials.
                    let mut polys: Vec<KlPol> = vec![KlPol::zero(), KlPol::monomial(0)];
                    let mut index_of: std::collections::HashMap<Vec<i32>, usize> =
                        std::collections::HashMap::new();
                    index_of.insert(Vec::new(), 0);
                    index_of.insert(vec![1], 1);
                    let mut matrix = vec![vec![0_i32; size]; size];
                    for (new_y, &y) in fiber.iter().enumerate() {
                        // int_Matrix(klt.size()) is the identity
                        // (matrix.h:287); only x<y entries are overwritten.
                        matrix[new_y][new_y] = 1;
                        for x in table.nonzero_column(y) {
                            if x == y {
                                continue;
                            }
                            let new_x = loc[x];
                            if new_x == usize::MAX {
                                continue;
                            }
                            let (pool_index, flip) = table.kl_pol_index(x, y);
                            let pol = table
                                .polys()
                                .get(pool_index)
                                .expect("in-range pool index")
                                .clone();
                            let key = pol.as_slice().to_vec();
                            let new_index = *index_of.entry(key).or_insert_with(|| {
                                polys.push(pol);
                                polys.len() - 1
                            });
                            matrix[new_x][new_y] = if flip {
                                -(new_index as i32)
                            } else {
                                new_index as i32
                            };
                        }
                    }
                    // Length stops over the fiber, indexed by length
                    // within the common block: upstream generates the
                    // block below the entry element and assigns fresh
                    // lengths there, which for the members of one fiber
                    // equals the full-block length minus the fiber
                    // minimum (blocks.cpp:976-990; atlas-types.w:8717).
                    let min_length = fiber
                        .iter()
                        .filter_map(|&n| block.graph.length(eb.z(n)))
                        .min()
                        .unwrap_or(0);
                    let adjusted: Vec<usize> = fiber
                        .iter()
                        .map(|&n| block.graph.length(eb.z(n)).unwrap_or(0) - min_length)
                        .collect();
                    let max_length = adjusted.iter().copied().max().unwrap_or(0);
                    let mut stops = vec![0_i32; max_length + 2];
                    for (i, stop) in stops.iter_mut().enumerate().skip(1) {
                        *stop = adjusted
                            .iter()
                            .position(|&length| length >= i)
                            .map_or(size as i32, |position| position as i32);
                    }
                    Ok(Value::Tuple(vec![
                        matrix_value(&matrix, span)?,
                        Value::List(
                            polys
                                .iter()
                                .map(|pol| Value::Vector(Vec32(pol.as_slice().to_vec())))
                                .collect(),
                        ),
                        Value::Vector(Vec32(stops)),
                    ]))
                }
                _ => {
                    // partial_extended_KL_block.
                    let eb = build_ext_block(&block, parameter, &delta, &twist, span)?;
                    let size_big = eb.element(z0 + 1);
                    // B.singular(gamma): the simply-singular coroots.
                    let mut singular = RankFlags::empty();
                    for (s, coroot) in datum.simple_coroots().iter().enumerate() {
                        let pairing: i64 = coroot
                            .as_slice()
                            .iter()
                            .zip(gamma.numerator().iter())
                            .map(|(&c, &g)| i64::from(c) * g)
                            .sum();
                        if pairing == 0 {
                            singular.set(s);
                        }
                    }
                    let singular_orbits = eb.singular_orbits(&singular);
                    // Upstream truncates the extended block at the entry
                    // element's fiber: `size = eblock.element(
                    // entry_element+1)` (ext_kl.cpp:962-963), so only
                    // extended elements at or below the entry participate.
                    let (fiber, loc) = {
                        let (fiber, loc) = ext_fiber(&eb, &members);
                        (
                            fiber
                                .into_iter()
                                .filter(|&n| n < size_big)
                                .collect::<Vec<_>>(),
                            loc,
                        )
                    };
                    let fiber_size = fiber.len();
                    let mut table =
                        ExtKlTable::new(&eb).map_err(|e| structure_diagnostic(e, span))?;
                    table
                        .fill_columns(size_big)
                        .map_err(|e| structure_diagnostic(e, span))?;
                    // Upstream builds the extended block over the common
                    // block itself, so the KLV matrix and the condensation
                    // see only member elements (ext_kl.cpp:955-970). Here
                    // the table comes from the full block; the fiber
                    // submatrix is used instead, and descents whose links
                    // the parity gate kept out of the common block count
                    // as nonparity ascents (no descent) there.
                    let mut p_mat = vec![vec![KlPol::zero(); fiber_size]; fiber_size];
                    for (i, row) in p_mat.iter_mut().enumerate() {
                        for (j, entry) in row.iter_mut().enumerate().skip(i + 1) {
                            *entry = table.p(fiber[i], fiber[j]);
                        }
                    }
                    // A singular descent of `y` inside the common block:
                    // compact descents count (the row represents zero);
                    // linked descents count only when their first link
                    // stayed in the fiber.
                    let restricted_descent = |s: usize, y: usize| -> Option<usize> {
                        let kind = eb.descent_type(s, y);
                        if !kind.is_descent() {
                            return None;
                        }
                        if kind.is_like_compact() {
                            return Some(y);
                        }
                        let first = if kind.has_double_image() {
                            eb.cayleys(s, y).0
                        } else {
                            eb.some_scent(s, y)
                        };
                        match first {
                            Some(x) if loc[x] != usize::MAX => Some(x),
                            _ => None,
                        }
                    };
                    // condense (ext_block.cpp:2015-2048): push every row
                    // with a singular descent down to its descent rows
                    // (sign `-epsilon`); the reverse loop is essential.
                    let mut survivors = Vec::new();
                    for yi in (0..fiber_size).rev() {
                        let y = fiber[yi];
                        let Some(s) = (0..eb.rank()).find(|&s| {
                            singular_orbits.is_set(s) && restricted_descent(s, y).is_some()
                        }) else {
                            survivors.push(y);
                            continue;
                        };
                        let kind = eb.descent_type(s, y);
                        if kind.is_like_compact() {
                            continue; // no descents: `y` represents zero
                        }
                        if kind.has_double_image() {
                            let (first, second) = eb.cayleys(s, y);
                            for target in [first, second].into_iter().flatten() {
                                let ti = loc[target];
                                debug_assert!(ti != usize::MAX);
                                kl_row_operation(&mut p_mat, ti, yi, -eb.epsilon(s, target, y));
                            }
                        } else {
                            let x = eb.some_scent(s, y).expect("condense: descent has a link");
                            let xi = loc[x];
                            debug_assert!(xi != usize::MAX);
                            kl_row_operation(&mut p_mat, xi, yi, -eb.epsilon(s, x, y));
                        }
                    }
                    survivors.reverse(); // pushed in decreasing order
                    let n = survivors.len();
                    // Compress to the surviving rows and columns, then flip
                    // signs for odd length distance (ext_kl.cpp:982-1003);
                    // the fiber shares a constant length offset, so the
                    // full-block lengths give the same parities.
                    let mut matrix = vec![vec![KlPol::zero(); n]; n];
                    for (i, &si) in survivors.iter().enumerate() {
                        for (j, &sj) in survivors.iter().enumerate().skip(i) {
                            matrix[i][j] = p_mat[loc[si]][loc[sj]].clone();
                        }
                    }
                    for j in 0..n {
                        let parity = eb.length(survivors[j]) % 2;
                        for i in 0..j {
                            if eb.length(survivors[i]) % 2 != parity {
                                matrix[i][j] = matrix[i][j].scaled(-1);
                            }
                        }
                    }
                    // Rebuild the pool and index matrix over the condensed
                    // matrix (ext_kl.cpp:1009-1016): entries 0 and 1 are
                    // the primed zero and one; P_index_mat starts as the
                    // identity, only i<j entries are matched.
                    let mut polys: Vec<KlPol> = vec![KlPol::zero(), KlPol::monomial(0)];
                    let mut index_of: std::collections::HashMap<Vec<i32>, usize> =
                        std::collections::HashMap::new();
                    index_of.insert(Vec::new(), 0);
                    index_of.insert(vec![1], 1);
                    let mut index_matrix = vec![vec![0_i32; n]; n];
                    for j in 0..n {
                        index_matrix[j][j] = 1;
                        for i in 0..j {
                            let pol = matrix[i][j].clone();
                            let key = pol.as_slice().to_vec();
                            let index = *index_of.entry(key).or_insert_with(|| {
                                polys.push(pol);
                                polys.len() - 1
                            });
                            index_matrix[i][j] = index as i32;
                        }
                    }
                    let mut params = Vec::with_capacity(n);
                    for &survivor in &survivors {
                        params.push(fiber_param(eb.z(survivor), &gamma_lambdas)?);
                    }
                    Ok(Value::Tuple(vec![
                        Value::List(params),
                        matrix_value(&index_matrix, span)?,
                        Value::List(
                            polys
                                .iter()
                                .map(|pol| Value::Vector(Vec32(pol.as_slice().to_vec())))
                                .collect(),
                        ),
                    ]))
                }
            }
        }
        // scale_extended_wrapper (atlas-types.w:8449-8472): scale the
        // infinitesimal character of a final parameter by a positive
        // rational at the extended-parameter level
        // (ext_block::scaled_extended_finalise, ext_block.cpp:2736-2807),
        // returning the final parameter paired with the net flip of the
        // default extension choice. Upstream pushes the Param FIRST, then
        // whether(flip), then wraps the pair.
        "scale_extended" => {
            arity(name, arguments, 3, span)?;
            let Value::Domain(DomainValue::Param(parameter)) = &arguments[0] else {
                return Err(type_error(
                    span,
                    format!(
                        "{name} has no matching overload for {} argument(s)",
                        arguments.len()
                    ),
                ));
            };
            let Value::Rational(factor) = &arguments[2] else {
                return Err(type_error(
                    span,
                    format!("expected a rat, found {}", arguments[2]),
                ));
            };
            let (delta, factor_num, factor_den) =
                scale_extended_gates(parameter, &arguments[1], factor, span)?;
            let rc = rep_context(&parameter.context);
            let context = ExtRepContext::new(&rc, delta)
                .map_err(|error| structure_diagnostic(error, span))?;
            let (repr, flip) =
                scaled_extended_finalise(&context, &parameter.repr, factor_num, factor_den)
                    .map_err(|error| structure_diagnostic(error, span))?;
            Ok(Value::Tuple(vec![
                Value::Domain(DomainValue::Param(ParamValue {
                    context: Arc::clone(&parameter.context),
                    repr,
                })),
                Value::Boolean(flip),
            ]))
        }
        // K_type_pol_extended_wrapper (atlas-types.w:8487-8500): restrict
        // the extended parameter to K
        // (ext_block::extended_restrict_to_K, ext_block.cpp:2435-2547).
        // Each survivor contributes with Split(1,0) when it is
        // default-aligned, Split(0,1) when flipped; like terms merge and
        // the sum is ordered with the K_type_pol comparator
        // (K_repr.h:59-70).
        "K_type_pol_extended" => {
            arity(name, arguments, 2, span)?;
            let Value::Domain(DomainValue::Param(parameter)) = &arguments[0] else {
                return Err(type_error(
                    span,
                    format!(
                        "{name} has no matching overload for {} argument(s)",
                        arguments.len()
                    ),
                ));
            };
            let delta = k_type_pol_extended_gates(parameter, &arguments[1], span)?;
            let rc = rep_context(&parameter.context);
            let context = ExtRepContext::new(&rc, delta)
                .map_err(|error| structure_diagnostic(error, span))?;
            let restricted = extended_restrict_to_k(&context, &parameter.repr)
                .map_err(|error| structure_diagnostic(error, span))?;
            let mut terms: Vec<(SplitValue, KType)> = Vec::new();
            for (ktype, (e, f)) in restricted {
                merge_pol_term(&mut terms, SplitValue::new(e, f), ktype);
            }
            sort_ktypepol_terms(&mut terms);
            Ok(Value::Domain(DomainValue::KTypePol(KTypePolValue {
                rf: Arc::clone(&parameter.context),
                terms,
            })))
        }
        // finalize_extended_wrapper (atlas-types.w:8514-8537): finalize the
        // extended parameter into an SR_poly
        // (ext_block::extended_finalise, ext_block.cpp:2598-2721). A
        // flipped survivor contributes Split(0,1) ("1s*"), a
        // default-aligned one Split(1,0); the sum is ordered with the
        // SR_poly comparator (repr.cpp:41-54).
        "finalize_extended" => {
            arity(name, arguments, 2, span)?;
            let Value::Domain(DomainValue::Param(parameter)) = &arguments[0] else {
                return Err(type_error(
                    span,
                    format!(
                        "{name} has no matching overload for {} argument(s)",
                        arguments.len()
                    ),
                ));
            };
            let delta = finalize_extended_gates(parameter, &arguments[1], span)?;
            let rc = rep_context(&parameter.context);
            let context = ExtRepContext::new(&rc, delta)
                .map_err(|error| structure_diagnostic(error, span))?;
            let finalized = extended_finalise(&context, &parameter.repr)
                .map_err(|error| structure_diagnostic(error, span))?;
            let mut terms: Vec<(SplitValue, StandardRepr)> = Vec::new();
            for (repr, flip) in finalized {
                let coefficient = if flip {
                    SplitValue::new(0, 1)
                } else {
                    SplitValue::new(1, 0)
                };
                merge_pol_term(&mut terms, coefficient, repr);
            }
            sort_parampol_terms(&mut terms);
            Ok(Value::Domain(DomainValue::ParamPol(ParamPolValue {
                rf: Arc::clone(&parameter.context),
                terms,
            })))
        }
        "Cartan_info" => {
            arity(name, arguments, 1, span)?;
            let (context, id) = as_cartan_class(&arguments[0], span)?;
            let cartan = context
                .classification
                .cartan_class(id)
                .expect("CartanClass values carry an in-range id");
            let representative = cartan.representative();
            let root_involution = representative.root_involution();
            // tori::classify (atlas-types.w:4105-4112).
            let classification = domain_classify_involution(
                root_involution.involution().weight_matrix(),
                &IntegerLatticeBudget::new(64, 1_000_000, 1_000_000, 256),
            )
            .map_err(|error| structure_diagnostic(error, span))?;
            let (compact, _complex_torus, split) = classification.as_tuple();
            // The Weyl word of the Cartan involution (Weyl_group().word —
            // the ELECTED canonical expression, not an arbitrary reduced
            // word; see the print_block word column).
            let weyl = WeylElement::from_action(
                context.inner_class.root_system(),
                representative.weyl_action(),
            )
            .map_err(|error| structure_diagnostic(error, span))?;
            let word = CompactWeyl::new(context.inner_class.root_system().datum().cartan_matrix())
                .map_err(|error| structure_diagnostic(error, span))?
                .canonical_word(&weyl_reduced_word(&context.inner_class, &weyl));
            let word_value = Value::Vector(Vec32(
                word.iter()
                    .map(|&generator| generator as i32)
                    .collect::<Vec<_>>(),
            ));
            // orbitSize and fiberSize (atlas-types.w:2143-2144).
            let orbit_size = cartan.twisted_involution_count();
            let fiber_rank = fiber_rank(
                root_involution.involution().weight_matrix(),
                &IntegerLatticeBudget::new(64, 1_000_000, 1_000_000, 256),
            )
            .map_err(|error| structure_diagnostic(error, span))?;
            let fiber_size = 1_usize << fiber_rank;
            // subsystem types of the simple imaginary / real / complex roots.
            let imaginary = root_involution.imaginary_simple_roots();
            let real = root_involution.real_simple_roots();
            let complex = make_simple_complex(&context.inner_class, root_involution)
                .map_err(|error| structure_diagnostic(error, span))?;
            Ok(Value::Tuple(vec![
                Value::Tuple(vec![
                    Value::Integer(BigInt::from(compact)),
                    Value::Integer(BigInt::from(_complex_torus)),
                    Value::Integer(BigInt::from(split)),
                ]),
                word_value,
                Value::Tuple(vec![
                    Value::Integer(BigInt::from(orbit_size)),
                    Value::Integer(BigInt::from(fiber_size)),
                ]),
                Value::Tuple(vec![
                    subsystem_type_value(&context.inner_class, imaginary, span)?,
                    subsystem_type_value(&context.inner_class, real, span)?,
                    subsystem_type_value(&context.inner_class, &complex, span)?,
                ]),
            ]))
        }
        "torus_factor" => {
            arity(name, arguments, 1, span)?;
            let (context, id) = as_kgb_element(&arguments[0], span)?;
            let factor = context
                .graph
                .torus_factor(id, &context.table)
                .map_err(|error| runtime(span, error.to_string()))?;
            Ok(Value::RatVector(ratvec_from_rationals(
                factor.to_rationals(),
                span,
            )?))
        }
        // default_extended (atlas-types.w:7313-7337, ext_block.cpp:2352-
        // 2420): the components of a default extended parameter for a
        // delta-twisted group. The wrapper returns the 4-tuple
        // (lambda, tau, l, t); with the identity twist tau and t vanish.
        "default_extended" => {
            arity(name, arguments, 2, span)?;
            let Value::Domain(DomainValue::Param(parameter)) = &arguments[0] else {
                return Err(type_error(span, "default_extended requires a Param"));
            };
            let delta: Vec<Vec<i64>> = as_matrix(&arguments[1], span)?
                .iter()
                .map(|row| row.iter().map(|&entry| i64::from(entry)).collect())
                .collect();
            let datum = parameter.context.parent.root_datum.datum.clone();
            // test_compatible (atlas-types.w:4627-4633): the twist must
            // preserve the root datum.
            {
                let root_system = RootSystem::enumerate(&datum, ROOT_BUDGET)
                    .map_err(|error| structure_diagnostic(error, span))?;
                let simple_roots = datum.simple_roots().to_vec();
                let simple_coroots = datum.simple_coroots().to_vec();
                for (index, simple_root) in simple_roots.iter().enumerate() {
                    let coordinates: Vec<i64> = simple_root
                        .as_slice()
                        .iter()
                        .enumerate()
                        .map(|(row, _entry)| {
                            delta[row]
                                .iter()
                                .zip(simple_root.as_slice())
                                .map(|(&d, &c)| d * i64::from(c))
                                .sum::<i64>()
                        })
                        .collect();
                    // The image must be a root: find it in the root system.
                    let position = root_system.roots().iter().position(|root| {
                        root.as_slice()
                            .iter()
                            .map(|&e| i64::from(e))
                            .collect::<Vec<i64>>()
                            == coordinates
                    });
                    let Some(position) = position else {
                        return Err(runtime(
                            span,
                            format!("Matrix maps simple root {index} to non-root"),
                        ));
                    };
                    // The coroot image must match: delta * coroot == coroot(image).
                    let coroot_image: Vec<i64> = simple_coroots[index]
                        .as_slice()
                        .iter()
                        .enumerate()
                        .map(|(row, _entry)| {
                            delta[row]
                                .iter()
                                .zip(simple_coroots[index].as_slice())
                                .map(|(&d, &c)| d * i64::from(c))
                                .sum::<i64>()
                        })
                        .collect();
                    let expected = root_system
                        .coroot(RootId::from_usize(position))
                        .map(|c| {
                            c.as_slice()
                                .iter()
                                .map(|&e| i64::from(e))
                                .collect::<Vec<i64>>()
                        })
                        .unwrap_or_default();
                    if coroot_image != expected {
                        return Err(runtime(
                            span,
                            format!("Matrix does not map simple coroot {index} to coroot"),
                        ));
                    }
                }
            }
            let gamma = parameter.repr.gamma().clone();
            // The twist must fix the infinitesimal character.
            {
                let numerator = gamma.numerator();
                let mut image = vec![0_i64; delta.len()];
                for (row, delta_row) in delta.iter().enumerate() {
                    for (column, &entry) in delta_row.iter().enumerate() {
                        image[row] += entry * numerator.get(column).copied().unwrap_or(0);
                    }
                }
                for (index, &entry) in numerator.iter().enumerate() {
                    if image.get(index).copied().unwrap_or(0) != entry {
                        return Err(runtime(
                            span,
                            "Involution does not fix infinitesimal character",
                        ));
                    }
                }
            }
            let rc = rep_context(&parameter.context);
            let x = parameter.repr.x();
            // srm: gamma-lambda unique modulo X* (StandardReprMod::mod_reduce).
            let bits = parameter
                .context
                .graph
                .element(x)
                .ok_or_else(|| runtime(span, "KGB element"))?;
            let y_bits = rc
                .torus_part(x, bits.torus_bits())
                .map_err(|error| structure_diagnostic(error, span))?;
            let mut gamma_lambda = rc
                .gamma_lambda(x, &y_bits, &gamma)
                .map_err(|error| structure_diagnostic(error, span))?;
            let involution = rc
                .involution_of(x)
                .map_err(|error| structure_diagnostic(error, span))?;
            rc.real_unique(involution, &mut gamma_lambda)
                .map_err(|error| structure_diagnostic(error, span))?;
            // lambda = (gamma - gamma_lambda) in integer coordinates.
            let d1 = gamma.denominator();
            let d2 = gamma_lambda.denominator();
            let mut lambda = Vec::new();
            for index in 0..datum.semisimple_rank() {
                let n1 = gamma.numerator().get(index).copied().unwrap_or(0);
                let n2 = gamma_lambda.numerator().get(index).copied().unwrap_or(0);
                let common = d1 * d2;
                let diff = (n1 * d2 - n2 * d1) / common;
                lambda.push(diff);
            }
            // l = base_grading_vector - torus_factor(x) (ell, ext_block.cpp:215).
            let cocharacter = parameter.context.graph.cocharacter().to_rationals();
            let factor = parameter
                .context
                .graph
                .torus_factor(x, &parameter.context.table)
                .map_err(|error| structure_diagnostic(error, span))?;
            let factor_rat = factor.to_rationals();
            let mut l = Vec::new();
            for index in 0..datum.semisimple_rank() {
                let c = cocharacter.get(index).cloned().unwrap_or_default();
                let f = factor_rat.get(index).cloned().unwrap_or_default();
                let diff = c - f;
                let text = diff.to_string();
                let integer: i64 = text
                    .split_once('/')
                    .and_then(|(n, d)| {
                        let n: i64 = n.parse().ok()?;
                        let d: i64 = d.parse().ok()?;
                        (d == 1).then_some(n)
                    })
                    .or_else(|| text.parse().ok())
                    .ok_or_else(|| runtime(span, "l not integral"))?;
                l.push(integer);
            }
            // tau = find_solution(1-theta, (delta-1)*lambda_rho) and
            // t = find_solution(theta^T+1, (delta-1)*l) via the
            // integral-solution layer (ext_block.cpp:221-232).
            let theta_id = rc
                .involution_of(x)
                .map_err(|error| structure_diagnostic(error, span))?;
            let theta_rows = parameter
                .context
                .table
                .record(theta_id)
                .ok_or_else(|| runtime(span, "involution record"))?
                .theta()
                .weight_matrix();
            let rank = datum.semisimple_rank();
            let one_minus_theta: Vec<Vec<i64>> = (0..rank)
                .map(|row| {
                    (0..rank)
                        .map(|column| {
                            let entry = theta_rows[row].get(column).copied().unwrap_or(0) as i64;
                            i64::from(row == column) - entry
                        })
                        .collect()
                })
                .collect();
            let theta_transpose_plus: Vec<Vec<i64>> = (0..rank)
                .map(|row| {
                    (0..rank)
                        .map(|column| {
                            let entry = theta_rows[column].get(row).copied().unwrap_or(0) as i64;
                            i64::from(row == column) + entry
                        })
                        .collect()
                })
                .collect();
            let apply_delta_minus_one = |v: &[i64]| -> Vec<i64> {
                (0..rank)
                    .map(|row| {
                        let mut image = -v[row];
                        for (column, &entry) in delta[row].iter().enumerate() {
                            image += entry * v.get(column).copied().unwrap_or(0);
                        }
                        image
                    })
                    .collect()
            };
            let b_tau = apply_delta_minus_one(&lambda);
            let tau = find_solution(&one_minus_theta, &b_tau)
                .map_err(|message| runtime(span, message))?;
            let b_t = apply_delta_minus_one(&l);
            let t = find_solution(&theta_transpose_plus, &b_t)
                .map_err(|message| runtime(span, message))?;
            Ok(Value::Tuple(vec![
                Value::Vector(Vec32(lambda.iter().map(|&e| e as i32).collect())),
                Value::Vector(Vec32(tau.iter().map(|&e| e as i32).collect())),
                Value::Vector(Vec32(l.iter().map(|&e| e as i32).collect())),
                Value::Vector(Vec32(t.iter().map(|&e| e as i32).collect())),
            ]))
        }
        // base_grading_vector_wrapper (atlas-types.w:3689): the form's
        // elected g_rho_check, already frozen into the KGB graph.
        "base_grading_vector" => {
            arity(name, arguments, 1, span)?;
            let context = as_real_form(&arguments[0], span)?;
            Ok(Value::RatVector(ratvec_from_rationals(
                context.graph.cocharacter().to_rationals(),
                span,
            )?))
        }
        // x0_torus_part_wrapper (atlas-types.w:3695): the seed's torus
        // bits. Element #0 IS the seed (the BFS root).
        "initial_torus_bits" => {
            arity(name, arguments, 1, span)?;
            let context = as_real_form(&arguments[0], span)?;
            let base = context
                .graph
                .ids()
                .next()
                .ok_or_else(|| runtime(span, "Inexistent KGB element"))?;
            torus_bits_value(context, base, span)
        }
        // torus_bits_wrapper (atlas-types.w:4714): the element's torus part
        // as a 0/1 int vector.
        "torus_bits" => {
            arity(name, arguments, 1, span)?;
            let (context, id) = as_kgb_element(&arguments[0], span)?;
            torus_bits_value(context, id, span)
        }
        // K_type_wrapper (atlas-types.w:5240-5250): the (KGBElt,vec)
        // constructor rank-checks before its no-value gate, then builds
        // through Rep_context::sr_K (K_repr.cpp:25-32).
        // param_to_K_type_wrapper (atlas-types.w:6270-6280): restricting a
        // parameter to K ignores nu (repr.h:232-233).
        "K_type" => match arguments {
            [kgb, lam] => {
                let (context, x) = as_kgb_element(kgb, span)?;
                let lam_rho = as_weight_vec(lam, span)?;
                let rank = context.parent.inner_class.datum().lattice_rank();
                if lam_rho.len() != rank {
                    return Err(runtime(
                        span,
                        format!("Rank mismatch: ({rank},{})", lam_rho.len()),
                    ));
                }
                let rc = rep_context(context);
                let ktype = KType::sr_k(&rc, x, &Weight::new(lam_rho.clone()))
                    .map_err(|error| structure_diagnostic(error, span))?;
                Ok(Value::Domain(DomainValue::KType(KTypeValue {
                    context: Arc::clone(context),
                    ktype,
                })))
            }
            [Value::Domain(DomainValue::Param(parameter))] => {
                let rc = rep_context(&parameter.context);
                let ktype = rc
                    .sr_k_of_standard(&parameter.repr)
                    .map_err(|error| structure_diagnostic(error, span))?;
                Ok(Value::Domain(DomainValue::KType(KTypeValue {
                    context: Arc::clone(&parameter.context),
                    ktype,
                })))
            }
            _ => Err(type_error(
                span,
                format!(
                    "{name} has no matching overload for {} argument(s)",
                    arguments.len()
                ),
            )),
        },
        // module_parameter_wrapper (atlas-types.w:6215-6231): the
        // (KGBElt,vec,ratvec) constructor rank-checks both vectors against
        // the form's rank before its no-value gate (message order
        // (rank, lambda size, nu size)), then builds through
        // Rep_context::sr (repr.h:242-244). K_type_to_param_wrapper
        // (atlas-types.w:6283-6292) extends a K-type with nu = 0.
        "param" => match arguments {
            [kgb, lam, nu] => {
                let (context, x) = as_kgb_element(kgb, span)?;
                let lam_rho = as_weight_vec(lam, span)?;
                let nu_weight = as_rational_weight(nu, span)?;
                let rank = context.parent.inner_class.datum().lattice_rank();
                if nu_weight.rank() != lam_rho.len() || nu_weight.rank() != rank {
                    return Err(runtime(
                        span,
                        format!(
                            "Rank mismatch: ({rank},{},{})",
                            lam_rho.len(),
                            nu_weight.rank()
                        ),
                    ));
                }
                let rc = rep_context(context);
                let repr = rc
                    .sr(x, &Weight::new(lam_rho), &nu_weight)
                    .map_err(|error| structure_diagnostic(error, span))?;
                Ok(Value::Domain(DomainValue::Param(ParamValue {
                    context: Arc::clone(context),
                    repr,
                })))
            }
            [Value::Domain(DomainValue::KType(ktype))] => {
                let rc = rep_context(&ktype.context);
                let repr = rc
                    .sr_of_ktype(&ktype.ktype)
                    .map_err(|error| structure_diagnostic(error, span))?;
                Ok(Value::Domain(DomainValue::Param(ParamValue {
                    context: Arc::clone(&ktype.context),
                    repr,
                })))
            }
            _ => Err(type_error(
                span,
                format!(
                    "{name} has no matching overload for {} argument(s)",
                    arguments.len()
                ),
            )),
        },
        // K_type_height_wrapper (atlas-types.w:5291) and
        // parameter_height_wrapper (atlas-types.w:6311-6317): the stored
        // height.
        "height" => match arguments {
            [Value::Domain(DomainValue::KType(ktype))] => {
                Ok(Value::Integer(BigInt::from(ktype.ktype.height())))
            }
            [Value::Domain(DomainValue::Param(parameter))] => {
                Ok(Value::Integer(BigInt::from(parameter.repr.height())))
            }
            _ => Err(type_error(
                span,
                format!(
                    "{name} has no matching overload for {} argument(s)",
                    arguments.len()
                ),
            )),
        },
        // The predicate set shared by both value kinds
        // (atlas-types.w:5346-5377, 6360-6384): each predicate has one
        // wrapper per kind; `is_zero` is the negation of `is_nonzero`.
        "is_standard" | "is_dominant" | "is_zero" | "is_semifinal" | "is_final" => {
            arity(name, arguments, 1, span)?;
            let result = match &arguments[0] {
                Value::Domain(DomainValue::KType(ktype)) => {
                    let rc = rep_context(&ktype.context);
                    match name {
                        "is_standard" => ktype.ktype.is_standard(&rc),
                        "is_dominant" => ktype.ktype.is_dominant(&rc),
                        "is_zero" => ktype.ktype.is_nonzero(&rc).map(|nonzero| !nonzero),
                        "is_semifinal" => ktype.ktype.is_semifinal(&rc),
                        "is_final" => ktype.ktype.is_final(&rc),
                        _ => unreachable!(),
                    }
                }
                Value::Domain(DomainValue::Param(parameter)) => {
                    let rc = rep_context(&parameter.context);
                    match name {
                        "is_standard" => parameter.repr.is_standard(&rc),
                        "is_dominant" => parameter.repr.is_dominant(&rc),
                        "is_zero" => parameter.repr.is_nonzero(&rc).map(|nonzero| !nonzero),
                        "is_semifinal" => parameter.repr.is_semifinal(&rc),
                        "is_final" => parameter.repr.is_final(&rc),
                        _ => unreachable!(),
                    }
                }
                other => {
                    return Err(type_error(
                        span,
                        format!("expected a KType or Param, found {other}"),
                    ));
                }
            };
            result
                .map(Value::Boolean)
                .map_err(|error| structure_diagnostic(error, span))
        }
        // K_type_equivalent_wrapper (atlas-types.w:5323-5331): the real
        // form identity is checked before the no-value gate; equivalence
        // then moves both K-types to the canonical fiber of their Cartan
        // class (K_repr.cpp:159-171).
        "equivalent" => {
            arity(name, arguments, 2, span)?;
            match (&arguments[0], &arguments[1]) {
                (
                    Value::Domain(DomainValue::KType(left)),
                    Value::Domain(DomainValue::KType(right)),
                ) => {
                    require_same_form_value(
                        &left.context,
                        &right.context,
                        "Real form mismatch when testing equivalence",
                        span,
                    )?;
                    let rc = rep_context(&left.context);
                    let result = left
                        .ktype
                        .equivalent(&rc, &right.ktype)
                        .map_err(|error| structure_diagnostic(error, span))?;
                    Ok(Value::Boolean(result))
                }
                (
                    Value::Domain(DomainValue::Param(left)),
                    Value::Domain(DomainValue::Param(right)),
                ) => {
                    require_same_form_value(
                        &left.context,
                        &right.context,
                        "Real form mismatch when testing equivalence",
                        span,
                    )?;
                    let rc = rep_context(&left.context);
                    let result = left
                        .repr
                        .equivalent(&rc, &right.repr)
                        .map_err(|error| structure_diagnostic(error, span))?;
                    Ok(Value::Boolean(result))
                }
                _ => Err(type_error(
                    span,
                    "equivalent expects two KTypes or two Params",
                )),
            }
        }
        // K_type_dominant/normal/theta_stable/to_canonical_fiber wrappers
        // (atlas-types.w:5397-5444): KType-preserving transforms computed
        // behind the no-value gate. `normal` is the elected equivalence
        // class representative, `dominant` the complex-dominant form, and
        // `theta_stable` the form without complex descents.
        "dominant" | "normal" | "theta_stable" | "to_canonical_fiber" => {
            arity(name, arguments, 1, span)?;
            match (&arguments[0], name) {
                (Value::Domain(DomainValue::KType(ktype)), _) => {
                    let rc = rep_context(&ktype.context);
                    let transformed = match name {
                        "dominant" => ktype.ktype.made_dominant(&rc),
                        "normal" => ktype.ktype.normalised(&rc),
                        "theta_stable" => ktype.ktype.made_theta_stable(&rc),
                        "to_canonical_fiber" => ktype.ktype.to_canonical_fiber(&rc),
                        _ => unreachable!(),
                    }
                    .map_err(|error| structure_diagnostic(error, span))?;
                    Ok(Value::Domain(DomainValue::KType(KTypeValue {
                        context: Arc::clone(&ktype.context),
                        ktype: transformed,
                    })))
                }
                (Value::Domain(DomainValue::Param(parameter)), "dominant" | "normal") => {
                    let rc = rep_context(&parameter.context);
                    let transformed = match name {
                        "dominant" => parameter.repr.made_dominant(&rc),
                        "normal" => parameter.repr.normalised(&rc),
                        _ => unreachable!(),
                    }
                    .map_err(|error| structure_diagnostic(error, span))?;
                    Ok(Value::Domain(DomainValue::Param(ParamValue {
                        context: Arc::clone(&parameter.context),
                        repr: transformed,
                    })))
                }
                _ => Err(type_error(
                    span,
                    format!("{name} expects a KType or Param, found {}", arguments[0]),
                )),
            }
        }
        // K_type_pol_wrapper / virtual_module_wrapper
        // (atlas-types.w:5537-5543, 7613-7620): the empty sum of one real
        // form.
        "null_K_module" | "null_module" => {
            arity(name, arguments, 1, span)?;
            let rf = as_real_form(&arguments[0], span)?;
            if name == "null_K_module" {
                Ok(Value::Domain(DomainValue::KTypePol(KTypePolValue {
                    rf: Arc::clone(rf),
                    terms: Vec::new(),
                })))
            } else {
                Ok(Value::Domain(DomainValue::ParamPol(ParamPolValue {
                    rf: Arc::clone(rf),
                    terms: Vec::new(),
                })))
            }
        }
        // first/last_K_type_term_wrapper and first/last_term_wrapper
        // (atlas-types.w:5910-5943, 7996-8027): the selected term as a
        // (Split, KType/Param) pair, with the value-kind wording of the
        // upstream empty-poly error.
        "first_term" | "last_term" => {
            arity(name, arguments, 1, span)?;
            match &arguments[0] {
                Value::Domain(DomainValue::KTypePol(pol)) => {
                    let empty = if name == "first_term" {
                        "Empty KTypePol has no first term"
                    } else {
                        "Empty KTypePol has no last term"
                    };
                    let Some((coefficient, ktype)) = (if name == "first_term" {
                        pol.terms.first()
                    } else {
                        pol.terms.last()
                    }) else {
                        return Err(runtime(span, empty));
                    };
                    Ok(Value::Tuple(vec![
                        Value::Domain(DomainValue::Split(*coefficient)),
                        Value::Domain(DomainValue::KType(KTypeValue {
                            context: Arc::clone(&pol.rf),
                            ktype: ktype.clone(),
                        })),
                    ]))
                }
                Value::Domain(DomainValue::ParamPol(pol)) => {
                    let empty = if name == "first_term" {
                        "Empty module has no first term"
                    } else {
                        "Empty module has no last term"
                    };
                    let Some((coefficient, repr)) = (if name == "first_term" {
                        pol.terms.first()
                    } else {
                        pol.terms.last()
                    }) else {
                        return Err(runtime(span, empty));
                    };
                    Ok(Value::Tuple(vec![
                        Value::Domain(DomainValue::Split(*coefficient)),
                        Value::Domain(DomainValue::Param(ParamValue {
                            context: Arc::clone(&pol.rf),
                            repr: repr.clone(),
                        })),
                    ]))
                }
                other => Err(type_error(
                    span,
                    format!("expected a KTypePol or ParamPol, found {other}"),
                )),
            }
        }
        // param_poly_to_K_type_poly_wrapper (atlas-types.w:7717-7730):
        // restrict every term to K (sr_K) and expand through finals_for.
        "K_type_pol" => {
            arity(name, arguments, 1, span)?;
            let Value::Domain(DomainValue::ParamPol(pol)) = &arguments[0] else {
                return Err(type_error(span, "expected a ParamPol"));
            };
            let rc = rep_context(&pol.rf);
            let mut terms: Vec<(SplitValue, KType)> = Vec::new();
            for (coefficient, repr) in &pol.terms {
                let ktype = rc
                    .sr_k_of_standard(repr)
                    .map_err(|error| structure_diagnostic(error, span))?;
                let finals = ktype
                    .finals_for(&rc)
                    .map_err(|error| structure_diagnostic(error, span))?;
                for (final_ktype, multiplicity) in finals {
                    merge_pol_term(
                        &mut terms,
                        coefficient.mul(SplitValue::new(multiplicity, 0)),
                        final_ktype,
                    );
                }
            }
            sort_ktypepol_terms(&mut terms);
            Ok(Value::Domain(DomainValue::KTypePol(KTypePolValue {
                rf: Arc::clone(&pol.rf),
                terms,
            })))
        }
        // KGP_sum_wrapper (atlas-types.w:5995-6010): the KGP set of a
        // semifinal K-type as a row of length-parity-signed (int, KType)
        // pairs; the semifinal precondition precedes the no-value gate.
        "KGP_sum" => {
            arity(name, arguments, 1, span)?;
            let ktype = as_ktype(&arguments[0], span)?;
            let rc = rep_context(&ktype.context);
            if !ktype
                .ktype
                .is_semifinal(&rc)
                .map_err(|error| structure_diagnostic(error, span))?
            {
                return Err(runtime(
                    span,
                    "K-type has parity real roots (so not semifinal)",
                ));
            }
            let list = ktype
                .ktype
                .kgp_set(&rc)
                .map_err(|error| structure_diagnostic(error, span))?;
            let length = rc
                .graph()
                .length(ktype.ktype.x())
                .ok_or_else(|| runtime(span, "Inexistent KGB element"))?;
            let row = list
                .into_iter()
                .map(|term| {
                    let term_length = rc
                        .graph()
                        .length(term.x())
                        .expect("KGP terms are KGB elements of the same graph");
                    let difference = length as i64 - term_length as i64;
                    Value::Tuple(vec![
                        Value::Integer(BigInt::from(if difference % 2 == 0 { 1 } else { -1 })),
                        Value::Domain(DomainValue::KType(KTypeValue {
                            context: Arc::clone(&ktype.context),
                            ktype: term,
                        })),
                    ])
                })
                .collect();
            Ok(Value::List(row))
        }
        // K_type_formula_wrapper (atlas-types.w:6030-6054): the K-type
        // formula with a height cutoff; a negative bound means unbounded.
        "K_type_formula" => {
            arity(name, arguments, 2, span)?;
            let ktype = as_ktype(&arguments[0], span)?;
            let bound = i64::try_from(&as_integer(&arguments[1], span)?)
                .map_err(|_| runtime(span, "Integer value to big for conversion"))?;
            let rc = rep_context(&ktype.context);
            if !ktype
                .ktype
                .is_semifinal(&rc)
                .map_err(|error| structure_diagnostic(error, span))?
            {
                return Err(runtime(
                    span,
                    "K-type has parity real roots (so not semifinal)",
                ));
            }
            let max_level = if bound < 0 {
                u32::MAX
            } else {
                u32::try_from(bound)
                    .map_err(|_| runtime(span, "Integer value to big for conversion"))?
            };
            let formula = rc
                .k_type_formula(&ktype.ktype, max_level)
                .map_err(|error| structure_diagnostic(error, span))?;
            let mut terms: Vec<(SplitValue, KType)> = formula
                .into_iter()
                .map(|(term, coefficient)| (SplitValue::new(coefficient, 0), term))
                .collect();
            sort_ktypepol_terms(&mut terms);
            Ok(Value::Domain(DomainValue::KTypePol(KTypePolValue {
                rf: Arc::clone(&ktype.context),
                terms,
            })))
        }
        // branch_wrapper (atlas-types.w:6055-6070): repeatedly move the
        // least term of the remainder into the result and subtract its
        // K_type_formula from the remainder until it cancels
        // (K_repr.cpp:592-622).
        "branch" => {
            arity(name, arguments, 2, span)?;
            let pol = as_ktypepol(&arguments[0], span)?;
            let bound = i64::try_from(&as_integer(&arguments[1], span)?)
                .map_err(|_| runtime(span, "Integer value to big for conversion"))?;
            if bound < 0 {
                return Err(runtime(span, "Maximum level in branch cannot be negative"));
            }
            let max_level = u32::try_from(bound)
                .map_err(|_| runtime(span, "Integer value to big for conversion"))?;
            let rc = rep_context(&pol.rf);
            let mut remainder = pol.terms.clone();
            let mut result: Vec<(SplitValue, KType)> = Vec::new();
            let mut count: u64 = 0;
            while !remainder.is_empty() {
                count += 1;
                // Periodically flatten the remainder to remove leading
                // zero terms (K_repr.cpp:599-603).
                if count * count > 2 * remainder.len() as u64 {
                    remainder.retain(|(coefficient, _)| !coefficient.is_zero());
                    count = 0;
                    if remainder.is_empty() {
                        break;
                    }
                }
                // Upstream keeps the lead IN the remainder while its
                // K_type_formula is subtracted, so the formula's own
                // lead term (coefficient 1) cancels it
                // (K_repr.cpp:611-616); removing it first would leave a
                // -coef lead term that cycles forever.
                let (lead_coefficient, lead) = remainder[0].clone();
                if lead.height() > max_level {
                    remainder.remove(0);
                    // Drop input terms that are already too high.
                    continue;
                }
                merge_pol_term(&mut result, lead_coefficient, lead.clone());
                let formula = rc
                    .k_type_formula(&lead, max_level)
                    .map_err(|error| structure_diagnostic(error, span))?;
                for (term, multiplicity) in formula {
                    let scaled = lead_coefficient.mul(SplitValue::new(multiplicity, 0));
                    merge_pol_term(&mut remainder, scaled.neg(), term);
                }
                sort_ktypepol_terms(&mut remainder);
            }
            sort_ktypepol_terms(&mut result);
            Ok(Value::Domain(DomainValue::KTypePol(KTypePolValue {
                rf: Arc::clone(&pol.rf),
                terms: result,
            })))
        }
        // deform_wrapper (atlas-types.w:8084-8105): for every final
        // parameter of the input, compute its deformation terms in the
        // common block and accumulate an SR_poly. The crate's
        // deformation_terms returns integer coefficients; the wrapper
        // scales by `Split_integer(c, -c)` = c(1-s) and by the
        // finals_for coefficient.
        "deform" => {
            arity(name, arguments, 1, span)?;
            let Value::Domain(DomainValue::Param(parameter)) = &arguments[0] else {
                return Err(type_error(
                    span,
                    format!(
                        "{name} has no matching overload for {} argument(s)",
                        arguments.len()
                    ),
                ));
            };
            let rc = rep_context(&parameter.context);
            let finals = rc
                .finals_for_standard(&parameter.repr)
                .map_err(|error| structure_diagnostic(error, span))?;
            let dual_parent = build_dual_inner_class(&parameter.context.parent, span)?;
            // The deform block pairs the real form with its dual's
            // quasisplit form (matching lookup_full_block).
            let dual_quasisplit = dual_parent.order.quasisplit_external();
            let dual_rf = build_real_form(&dual_parent, dual_quasisplit, span)?;
            let mut terms: Vec<(SplitValue, StandardRepr)> = Vec::new();
            for (final_sr, final_coef) in finals {
                let block = BlockGraph::build(
                    &parameter.context.graph,
                    &parameter.context.table,
                    &dual_rf.graph,
                    &dual_rf.table,
                    &dual_rf.parent.inner_class,
                    WEYL_BUDGET,
                )
                .map_err(|error| structure_diagnostic(error, span))?;
                let mut kl_table =
                    KlTable::new(&block).map_err(|error| structure_diagnostic(error, span))?;
                kl_table
                    .fill(0)
                    .map_err(|error| structure_diagnostic(error, span))?;
                let q_index = (0..block.size())
                    .find(|&z| block.x(z) == Some(final_sr.x()))
                    .ok_or_else(|| runtime(span, "parameter not in the common block"))?;
                let lam_rho = rc
                    .lambda_rho(&final_sr)
                    .map_err(|error| structure_diagnostic(error, span))?;
                let dterms = rc
                    .deformation_terms(&block, q_index, final_sr.gamma(), &lam_rho, &kl_table)
                    .map_err(|error| structure_diagnostic(error, span))?;
                for (term_sr, coefficient) in dterms {
                    // Split_integer(c, -c) * it->second (atlas-types.w:8103).
                    let scaled = SplitValue::new(
                        coefficient.wrapping_mul(final_coef),
                        (-coefficient).wrapping_mul(final_coef),
                    );
                    merge_pol_term(&mut terms, scaled, term_sr);
                }
            }
            sort_parampol_terms(&mut terms);
            Ok(Value::Domain(DomainValue::ParamPol(ParamPolValue {
                rf: Arc::clone(&parameter.context),
                terms,
            })))
        }
        // twisted_deform_wrapper (atlas-types.w:8120-8150): the twisted
        // deformation terms of a final, delta-fixed parameter, over the
        // common block on the INTEGRAL subsystem of gamma and the
        // distinguished-twist extended block. The crate's
        // twisted_deformation_terms returns integer coefficients; the
        // wrapper maps each to `Split_integer(c, -c)` = c(1-s)
        // (atlas-types.w:8146-8147). The rank-0 integral subsystem (the
        // A1 nu=[1]/2 case) is the singleton block, whose terms are
        // empty (repr.cpp:2435-2436); a proper subsystem is not ported.
        "twisted_deform" => {
            arity(name, arguments, 1, span)?;
            let Value::Domain(DomainValue::Param(parameter)) = &arguments[0] else {
                return Err(type_error(
                    span,
                    format!(
                        "{name} has no matching overload for {} argument(s)",
                        arguments.len()
                    ),
                ));
            };
            twisted_deform_gates(parameter, span)?;
            let rc = rep_context(&parameter.context);
            let (delta, twist) = distinguished_twist(parameter, span)?;
            let mut terms: Vec<(SplitValue, StandardRepr)> = with_integral_block(
                parameter,
                &rc,
                &parameter.repr,
                &(delta, twist),
                span,
                Vec::new,
                |block, eblock, y0, lambda_rho| {
                    let gamma = parameter.repr.gamma();
                    let singular_orbits = singular_orbits_at(&rc, eblock, gamma)
                        .map_err(|error| structure_diagnostic(error, span))?;
                    let raw = twisted_deformation_terms(
                        &rc,
                        block,
                        eblock,
                        y0,
                        &singular_orbits,
                        gamma,
                        lambda_rho,
                    )
                    .map_err(|error| structure_diagnostic(error, span))?;
                    let mut terms = Vec::new();
                    for (term_sr, coefficient) in raw {
                        merge_pol_term(
                            &mut terms,
                            SplitValue::new(coefficient, coefficient.wrapping_neg()),
                            term_sr,
                        );
                    }
                    Ok(terms)
                },
            )?;
            sort_parampol_terms(&mut terms);
            Ok(Value::Domain(DomainValue::ParamPol(ParamPolValue {
                rf: Arc::clone(&parameter.context),
                terms,
            })))
        }
        // twisted_KL_sum_at_s (atlas-types.w:8370-8382 distinguished
        // overload, :8420-8431 external-delta overload): the alternating
        // twisted KL column sum at q = s. The distinguished path runs on
        // a made-dominant copy and signs by the PARENT block's length
        // function (`twisted_kl_column_at_s`, repr.cpp:2371-2423); the
        // external-delta path builds the extended block over the USER's
        // delta and signs by the extended block's own lengths
        // (`twisted_kl_sum`, repr.cpp:2304-2350). The rank-0 integral
        // subsystem's singleton block gives `1*p` (P_{y,y} = 1).
        "twisted_KL_sum_at_s" => {
            let Value::Domain(DomainValue::Param(parameter)) = &arguments[0] else {
                return Err(type_error(
                    span,
                    format!(
                        "{name} has no matching overload for {} argument(s)",
                        arguments.len()
                    ),
                ));
            };
            let rc = rep_context(&parameter.context);
            let (sr, twist_data) = match arguments.len() {
                1 => {
                    let sr = twisted_kl_sum_gates(parameter, span)?;
                    let (delta, twist) = distinguished_twist(parameter, span)?;
                    (sr, (delta, twist))
                }
                2 => {
                    let (delta, twist) =
                        external_twisted_kl_sum_gates(parameter, &arguments[1], span)?;
                    (parameter.repr.clone(), (delta, twist))
                }
                count => {
                    return Err(type_error(
                        span,
                        format!("{name} has no matching overload for {count} argument(s)"),
                    ));
                }
            };
            let distinguished = arguments.len() == 1;
            let mut terms: Vec<(SplitValue, StandardRepr)> = with_integral_block(
                parameter,
                &rc,
                &sr,
                &twist_data,
                span,
                // The singleton column sum is 1*sr (repr.cpp:2435-2436
                // leaves only the x == y entry of the KL table).
                || vec![(SplitValue::new(1, 0), sr.clone())],
                |block, eblock, y0, lambda_rho| {
                    let gamma = sr.gamma();
                    let raw = if distinguished {
                        twisted_kl_column_at_s(&rc, eblock, block, y0, gamma, lambda_rho)
                    } else {
                        let ext_y = eblock.element(y0);
                        twisted_kl_sum(&rc, eblock, ext_y, block, gamma, lambda_rho)
                    }
                    .map_err(|error| structure_diagnostic(error, span))?;
                    let mut terms = Vec::new();
                    for (term_sr, coefficient) in raw {
                        let coefficient: (i32, i32) = coefficient.into();
                        merge_pol_term(
                            &mut terms,
                            SplitValue::new(coefficient.0, coefficient.1),
                            term_sr,
                        );
                    }
                    Ok(terms)
                },
            )?;
            sort_parampol_terms(&mut terms);
            Ok(Value::Domain(DomainValue::ParamPol(ParamPolValue {
                rf: Arc::clone(&parameter.context),
                terms,
            })))
        }
        // twisted_full_deform_wrapper (atlas-types.w:8229-8251): the full
        // recursive twisted K-type deformation over the distinguished
        // involution — extended_finalise (E2) followed by
        // `Rep_table::twisted_deformation` (repr.cpp:2552-2653) of each
        // final, added with Split(0,1) when the finalise and deformation
        // flips differ and Split(1,0) when they agree
        // (atlas-types.w:8245-8246). The timed second overload uses the
        // same completed-result cache and cooperative recursion probe.
        "twisted_full_deform" => {
            if arguments.is_empty() || arguments.len() > 2 {
                return Err(type_error(
                    span,
                    format!(
                        "{name} has no matching overload for {} argument(s)",
                        arguments.len()
                    ),
                ));
            }
            let Value::Domain(DomainValue::Param(parameter)) = &arguments[0] else {
                return Err(type_error(
                    span,
                    format!(
                        "{name} has no matching overload for {} argument(s)",
                        arguments.len()
                    ),
                ));
            };
            let timer = if arguments.len() == 2 {
                Some(
                    i32::try_from(&as_integer(&arguments[1], span)?)
                        .map_err(|_| runtime(span, "Integer value to big for conversion"))?,
                )
            } else {
                None
            };
            twisted_full_deform_gates(parameter, span)?;
            let key = full_deform_key(&parameter.repr);
            let cached =
                cached_deformation(&parameter.context.twisted_full_deform_cache, &key, span)?;
            let make_polynomial = |terms| {
                Value::Domain(DomainValue::KTypePol(KTypePolValue {
                    rf: Arc::clone(&parameter.context),
                    terms,
                }))
            };

            if timer.is_none() {
                let terms = match cached {
                    Some(terms) => terms,
                    None => {
                        let terms = compute_twisted_full_deform(parameter, span, None)?
                            .expect("an unbounded twisted deformation cannot time out");
                        store_deformation(
                            &parameter.context.twisted_full_deform_cache,
                            key,
                            terms.clone(),
                            span,
                        )?;
                        terms
                    }
                };
                return Ok(make_polynomial(terms));
            }

            if let Some(terms) = cached {
                return Ok(Value::Union {
                    tag: 1,
                    injector_name: "done".into(),
                    value: Box::new(make_polynomial(terms)),
                });
            }
            let timer = timer.expect("the unary case returned above");
            if timer <= 0 {
                return Ok(Value::Union {
                    tag: 0,
                    injector_name: "timed_out".into(),
                    value: Box::new(Value::Tuple(Vec::new())),
                });
            }
            let Some(terms) = compute_twisted_full_deform(parameter, span, Some(timer))? else {
                return Ok(Value::Union {
                    tag: 0,
                    injector_name: "timed_out".into(),
                    value: Box::new(Value::Tuple(Vec::new())),
                });
            };
            store_deformation(
                &parameter.context.twisted_full_deform_cache,
                key,
                terms.clone(),
                span,
            )?;
            Ok(Value::Union {
                tag: 1,
                injector_name: "done".into(),
                value: Box::new(make_polynomial(terms)),
            })
        }
        // block_deform_wrapper (atlas-types.w:8178-8204): deform the
        // terms of p's block found in the accumulator, to the given
        // height bound (negative = maximal level), then slide each
        // deformed term down its reducibility points (drop a trailing
        // 1/1, scale to the previous point or to nu = 0,
        // atlas-types.w:8192-8198). NO test_standard/test_final; the
        // wrapper's no_value gate comes first (registered Skip). The
        // block's terms are EXTRACTED from the accumulator
        // (repr.cpp:2040-2056 queue.erase): the second component is the
        // accumulator minus the consumed terms, recomputed fresh on each
        // call since Atlas values are immutable. Push order: deformed
        // FIRST, then the remainder, wrapped as a pair.
        "block_deform" => {
            arity(name, arguments, 3, span)?;
            let Value::Domain(DomainValue::Param(parameter)) = &arguments[0] else {
                return Err(type_error(
                    span,
                    format!(
                        "{name} has no matching overload for {} argument(s)",
                        arguments.len()
                    ),
                ));
            };
            let Value::Domain(DomainValue::ParamPol(accumulator)) = &arguments[1] else {
                return Err(type_error(span, "expected a ParamPol"));
            };
            let bound = i32::try_from(&as_integer(&arguments[2], span)?)
                .map_err(|_| runtime(span, "Integer value to big for conversion"))?;
            let height_bound = if bound < 0 { u32::MAX } else { bound as u32 };
            let rc = rep_context(&parameter.context);
            let mut deformed_terms: Vec<(SplitValue, StandardRepr)> = Vec::new();
            let mut remainder_terms = accumulator.terms.clone();
            let nu = rc
                .nu(&parameter.repr)
                .map_err(|error| structure_diagnostic(error, span))?;
            if nu.numerator().iter().any(|&entry| entry != 0) {
                // lookup_full_block makes p dominant (repr.cpp:2035).
                let p = parameter
                    .repr
                    .made_dominant(&rc)
                    .map_err(|error| structure_diagnostic(error, span))?;
                let gamma = p.gamma().clone();
                let lambda_rho = rc
                    .lambda_rho(&p)
                    .map_err(|error| structure_diagnostic(error, span))?;
                let block = full_block_of(parameter, span)?;
                let accumulator_terms: Vec<(StandardRepr, SplitInteger)> = accumulator
                    .terms
                    .iter()
                    .map(|(coefficient, sr)| {
                        (
                            sr.clone(),
                            SplitInteger::new(coefficient.e(), coefficient.f()),
                        )
                    })
                    .collect();
                let (raw, consumed) = block_deformation_to_height(
                    &rc,
                    &block.graph,
                    &gamma,
                    &lambda_rho,
                    height_bound,
                    &accumulator_terms,
                )
                .map_err(|error| structure_diagnostic(error, span))?;
                for (sr, coefficient) in raw {
                    let rps = rc
                        .reducibility_points(&sr)
                        .map_err(|error| structure_diagnostic(error, span))?;
                    let index = if rps.last() == Some(&(1, 1)) {
                        rps.len().saturating_sub(1)
                    } else {
                        rps.len()
                    };
                    let (num, den) = if index > 0 { rps[index - 1] } else { (0, 1) };
                    let scaled = rc
                        .scale(&sr, num, den)
                        .map_err(|error| structure_diagnostic(error, span))?;
                    let coefficient: (i32, i32) = coefficient.into();
                    merge_pol_term(
                        &mut deformed_terms,
                        SplitValue::new(coefficient.0, coefficient.1),
                        scaled,
                    );
                }
                remainder_terms = accumulator
                    .terms
                    .iter()
                    .zip(&consumed)
                    .filter(|(_, &used)| !used)
                    .map(|((coefficient, sr), _)| (*coefficient, sr.clone()))
                    .collect();
            }
            sort_parampol_terms(&mut deformed_terms);
            Ok(Value::Tuple(vec![
                Value::Domain(DomainValue::ParamPol(ParamPolValue {
                    rf: Arc::clone(&parameter.context),
                    terms: deformed_terms,
                })),
                Value::Domain(DomainValue::ParamPol(ParamPolValue {
                    rf: Arc::clone(&accumulator.rf),
                    terms: remainder_terms,
                })),
            ]))
        }
        // truncate_K_type_poly_above_wrapper /
        // truncate_param_poly_above_wrapper
        // (atlas-types.w:5945-5976, 8033-8056): keep the terms whose
        // height does not exceed the bound; a negative bound keeps
        // everything (upstream converts it to the maximum level).
        "truncate_above_height" => {
            arity(name, arguments, 2, span)?;
            let bound = i64::try_from(&as_integer(&arguments[1], span)?)
                .map_err(|_| runtime(span, "Integer value to big for conversion"))?;
            match &arguments[0] {
                Value::Domain(DomainValue::KTypePol(pol)) => {
                    let terms = truncate_ktypepol(pol, bound);
                    Ok(Value::Domain(DomainValue::KTypePol(KTypePolValue {
                        rf: Arc::clone(&pol.rf),
                        terms,
                    })))
                }
                Value::Domain(DomainValue::ParamPol(pol)) => {
                    let terms = truncate_parampol(pol, bound);
                    Ok(Value::Domain(DomainValue::ParamPol(ParamPolValue {
                        rf: Arc::clone(&pol.rf),
                        terms,
                    })))
                }
                other => Err(type_error(
                    span,
                    format!("expected a KTypePol or ParamPol, found {other}"),
                )),
            }
        }
        // decompose_KGB_wrapper (atlas-types.w:4429): the owning real form
        // and the element number, wrapped as a pair. decompose_block_wrapper
        // (atlas-types.w:4809-4818) unwraps the block's two forms instead,
        // and from_split_wrapper (atlas-types.w:5127-5134) the (e, f) pair.
        "%" => {
            arity(name, arguments, 1, span)?;
            match &arguments[0] {
                Value::Domain(DomainValue::KgbElement(context, id)) => Ok(Value::Tuple(vec![
                    Value::Domain(DomainValue::RealForm(Arc::clone(context))),
                    Value::Integer(BigInt::from(id.index())),
                ])),
                Value::Domain(DomainValue::Block(block)) => Ok(Value::Tuple(vec![
                    Value::Domain(DomainValue::RealForm(Arc::clone(&block.rf))),
                    Value::Domain(DomainValue::RealForm(Arc::clone(&block.dual_rf))),
                ])),
                Value::Domain(DomainValue::Split(value)) => Ok(Value::Tuple(vec![
                    Value::Integer(BigInt::from(value.e())),
                    Value::Integer(BigInt::from(value.f())),
                ])),
                // unwrap_K_type_wrapper (atlas-types.w:5266-5277): the
                // owning KGB element and the ELECTED lambda-rho.
                Value::Domain(DomainValue::KType(ktype)) => Ok(Value::Tuple(vec![
                    Value::Domain(DomainValue::KgbElement(
                        Arc::clone(&ktype.context),
                        ktype.ktype.x(),
                    )),
                    Value::Vector(Vec32(ktype.ktype.lambda_rho().as_slice().to_vec())),
                ])),
                // unwrap_parameter_wrapper (atlas-types.w:6252-6267): the
                // KGB element, lambda-rho, and the info character gamma —
                // NOT the input nu.
                Value::Domain(DomainValue::Param(parameter)) => {
                    let rc = rep_context(&parameter.context);
                    let (lam_rho, gamma) = if parameter.repr.is_undefined() {
                        rc.undefined_decomposition(&parameter.repr)
                            .map_err(|error| structure_diagnostic(error, span))?
                    } else {
                        (
                            rc.lambda_rho(&parameter.repr)
                                .map_err(|error| structure_diagnostic(error, span))?,
                            parameter.repr.gamma().clone(),
                        )
                    };
                    Ok(Value::Tuple(vec![
                        Value::Domain(DomainValue::KgbElement(
                            Arc::clone(&parameter.context),
                            parameter.repr.x(),
                        )),
                        Value::Vector(Vec32(lam_rho.as_slice().to_vec())),
                        Value::RatVector(ratvec_from_rational_weight(&gamma, span)?),
                    ]))
                }
                other => Err(type_error(
                    span,
                    format!("expected a KGBElt, Block, Split, KType, or Param, found {other}"),
                )),
            }
        }
        // The KGB and Param twist families share the name but differ in
        // parameter normalization: parameter_twist_wrapper first makes its
        // input dominant through Rep_context::inner_twisted, while the
        // explicit-matrix parameter overload calls raw twisted. Both outer
        // wrappers validate the matrix through test_compatible first.
        "twist" => match arguments {
            [Value::Domain(DomainValue::KgbElement(context, id))] => {
                let delta = context
                    .parent
                    .inner_class
                    .distinguished_involution()
                    .involution()
                    .clone();
                let twist = context
                    .parent
                    .inner_class
                    .based_involution_twist(delta.clone())
                    .map_err(|error| runtime(span, error.to_string()))?;
                twist_element(context, *id, &delta, &twist, span)
            }
            [Value::Domain(DomainValue::Param(parameter))] => {
                let rc = rep_context(&parameter.context);
                twist_parameter(parameter, rc.inner_twisted(&parameter.repr), span)
            }
            [Value::Domain(DomainValue::KgbElement(context, id)), matrix] => {
                let (delta, twist) = compatible_outer_twist(context, matrix, span)?;
                twist_element(context, *id, &delta, &twist, span)
            }
            [Value::Domain(DomainValue::Param(parameter)), matrix] => {
                let (delta, twist) = compatible_outer_twist(&parameter.context, matrix, span)?;
                let rc = rep_context(&parameter.context);
                twist_parameter(parameter, rc.twisted(&parameter.repr, &delta, &twist), span)
            }
            _ => Err(type_error(
                span,
                format!(
                    "{name} has no matching overload for {} argument(s)",
                    arguments.len()
                ),
            )),
        },
        // W_elt_wrapper (atlas-types.w:2361-2368): the word check runs
        // before the no_value gate upstream, so validation stays eager.
        "from_dominant" => {
            arity(name, arguments, 2, span)?;
            // Two overloads: (RootDatum, vec) -> (WeylElt, vec) decompose, and
            // (vec, RootDatum) -> (vec, WeylElt) codecompose.
            let decompose = matches!(&arguments[0], Value::Domain(DomainValue::RootDatum(_)));
            let vector_value = if decompose {
                &arguments[1]
            } else {
                &arguments[0]
            };
            let coords: Vec<i32> = match vector_value {
                Value::List(entries) => {
                    let mut out = Vec::with_capacity(entries.len());
                    for entry in entries {
                        out.push(
                            as_integer(entry, span)?
                                .to_string()
                                .parse::<i32>()
                                .unwrap_or(i32::MAX),
                        );
                    }
                    out
                }
                Value::Vector(Vec32(entries)) => entries.clone(),
                other => {
                    return Err(type_error(span, format!("expected a vec, found {other}")));
                }
            };
            let (handle, coords) = if decompose {
                (as_root_datum(&arguments[0], span)?, coords)
            } else {
                (as_root_datum(&arguments[1], span)?, coords)
            };
            validate_from_dominant(arguments, span)?;
            let rank = handle.datum.lattice_rank();
            let datum = &handle.datum;
            let semisimple = datum.semisimple_rank();
            let mut current = Weight::new(coords);
            let mut word: Vec<usize> = Vec::new();
            loop {
                let mut acted = false;
                for s in 0..semisimple {
                    if decompose {
                        // factor_dominant (rootdata.cpp:1117-1135): reflect while
                        // <v, alpha_s^vee> < 0, accumulating the word.
                        let pairing = pair(&current, &datum.simple_coroots()[s])
                            .map_err(|error| runtime(span, error.to_string()))?;
                        if pairing < 0 {
                            let root_coords = datum.simple_roots()[s].as_slice();
                            let mut next = Vec::with_capacity(rank);
                            for (slot, &coordinate) in current.as_slice().iter().zip(root_coords) {
                                next.push(slot - pairing * coordinate);
                            }
                            current = Weight::new(next);
                            word.push(s);
                            acted = true;
                            break;
                        }
                    } else {
                        // factor_codominant (rootdata.cpp:1138-1155): reflect across
                        // the coroot while <v, alpha_s> < 0; the word is prepended.
                        let pairing: i64 = current
                            .as_slice()
                            .iter()
                            .zip(datum.simple_roots()[s].as_slice())
                            .map(|(&a, &b)| i64::from(a) * i64::from(b))
                            .sum();
                        if pairing < 0 {
                            let coroot_coords = datum.simple_coroots()[s].as_slice();
                            let mut next = Vec::with_capacity(rank);
                            for (slot, &coordinate) in current.as_slice().iter().zip(coroot_coords)
                            {
                                next.push(slot - (pairing as i32) * coordinate);
                            }
                            current = Weight::new(next);
                            word.insert(0, s);
                            acted = true;
                            break;
                        }
                    }
                }
                if !acted {
                    break;
                }
            }
            let context = build_weyl_context(handle, span)?;
            let mut element = WeylElement::identity(&context.system)
                .map_err(|error| runtime(span, error.to_string()))?;
            for generator in word {
                let (next, _) = element
                    .right_multiply_simple(&context.system, generator)
                    .map_err(|error| runtime(span, error.to_string()))?;
                element = next;
            }
            let weyl = weyl_elt_value(context, element, span)?;
            let vec = Value::Vector(Vec32(current.as_slice().to_vec()));
            if decompose {
                Ok(Value::Tuple(vec![weyl, vec]))
            } else {
                Ok(Value::Tuple(vec![vec, weyl]))
            }
        }
        "cofolded" => {
            arity(name, arguments, 1, span)?;
            let context = match &arguments[0] {
                Value::Domain(DomainValue::InnerClass(context)) => context.clone(),
                other => {
                    return Err(type_error(
                        span,
                        format!("expected an InnerClass, found {other}"),
                    ))
                }
            };
            let datum = &context.inner_class.datum();
            let rank = datum.semisimple_rank();
            let system = RootSystem::enumerate(datum, ROOT_BUDGET)
                .map_err(|error| runtime(span, error.to_string()))?;
            let distinguished = context.inner_class.distinguished_involution();
            let image = distinguished.image_permutation();
            let simple_ids: Vec<RootId> = datum
                .simple_roots()
                .iter()
                .map(|root| {
                    system
                        .id_of(root)
                        .ok_or_else(|| runtime(span, "simple root not found".to_string()))
                })
                .collect::<Result<_, _>>()?;
            let simple_coroots: Vec<Vec<i64>> = datum
                .simple_coroots()
                .iter()
                .map(|coroot| coroot.as_slice().iter().map(|&x| i64::from(x)).collect())
                .collect();
            let simple_roots: Vec<Vec<i64>> = datum
                .simple_roots()
                .iter()
                .map(|root| root.as_slice().iter().map(|&x| i64::from(x)).collect())
                .collect();
            let mut used = vec![false; rank];
            let mut folded_roots: Vec<Vec<i64>> = Vec::new();
            let mut folded_coroots: Vec<Vec<i64>> = Vec::new();
            for s in 0..rank {
                if used[s] {
                    continue;
                }
                let alpha_id = simple_ids[s];
                let image_id = image[alpha_id.index()];
                if image_id == alpha_id {
                    // ext_gen::one: the simple root is fixed by the twist.
                    folded_roots.push(simple_roots[s].clone());
                    folded_coroots.push(simple_coroots[s].clone());
                    used[s] = true;
                    continue;
                }
                let image_simple = (0..rank).find(|&t| simple_ids[t] == image_id);
                let Some(t) = image_simple else {
                    return Err(runtime(span, "Not a distinguished involution".to_string()));
                };
                let (s0, s1) = if alpha_id.index() < image_id.index() {
                    (s, t)
                } else {
                    (t, s)
                };
                used[s0] = true;
                used[s1] = true;
                // Is the pair orthogonal? (pair(alpha_s0, coroot(s1)) == 0)
                let root0 = Weight::new(simple_roots[s0].iter().map(|&x| x as i32).collect());
                let coroot1 = Coweight::new(simple_coroots[s1].iter().map(|&x| x as i32).collect());
                let orthogonal =
                    pair(&root0, &coroot1).map_err(|e| runtime(span, e.to_string()))? == 0;
                if orthogonal {
                    // ext_gen::two: the coroot is not folded.
                    folded_roots.push(
                        simple_roots[s0]
                            .iter()
                            .zip(&simple_roots[s1])
                            .map(|(a, b)| a + b)
                            .collect(),
                    );
                    folded_coroots.push(simple_coroots[s0].clone());
                } else {
                    // ext_gen::three: both root and coroot are folded.
                    folded_roots.push(
                        simple_roots[s0]
                            .iter()
                            .zip(&simple_roots[s1])
                            .map(|(a, b)| a + b)
                            .collect(),
                    );
                    folded_coroots.push(
                        simple_coroots[s0]
                            .iter()
                            .zip(&simple_coroots[s1])
                            .map(|(a, b)| a + b)
                            .collect(),
                    );
                }
            }
            // Build the folded Cartan matrix from the folded simple data.
            let folded_rank = folded_roots.len();
            let mut cartan = vec![vec![0_i32; folded_rank]; folded_rank];
            for (i, folded_root) in folded_roots.iter().enumerate() {
                for (j, folded_coroot) in folded_coroots.iter().enumerate() {
                    let root = Weight::new(folded_root.iter().map(|&x| x as i32).collect());
                    let coroot = Coweight::new(folded_coroot.iter().map(|&x| x as i32).collect());
                    cartan[i][j] =
                        pair(&root, &coroot).map_err(|e| runtime(span, e.to_string()))?;
                }
            }
            let folded_datum = BasedRootDatum::from_simple_data(
                datum.lattice_rank(),
                cartan,
                folded_roots
                    .iter()
                    .map(|row| Weight::new(row.iter().map(|&x| x as i32).collect()))
                    .collect(),
                folded_coroots
                    .iter()
                    .map(|row| Coweight::new(row.iter().map(|&x| x as i32).collect()))
                    .collect(),
            )
            .map_err(|error| runtime(span, error.to_string()))?;
            let folded_lie_type =
                infer_lie_type(folded_datum.cartan_matrix(), datum.lattice_rank(), span)?;
            let folded_isogeny = classify_isogeny(&folded_datum);
            Ok(Value::Domain(DomainValue::RootDatum(RootDatumHandle {
                datum: Arc::new(folded_datum),
                lie_type: folded_lie_type,
                isogeny: folded_isogeny,
                prefers_coroots: false,
            })))
        }
        "W_elt" => {
            arity(name, arguments, 2, span)?;
            let handle = as_root_datum(&arguments[0], span)?;
            let word = check_weyl_word(&arguments[1], handle.datum.semisimple_rank(), span)?;
            let context = build_weyl_context(handle, span)?;
            let mut element = WeylElement::identity(&context.system)
                .map_err(|error| runtime(span, error.to_string()))?;
            for generator in word {
                let (next, _) = element
                    .right_multiply_simple(&context.system, generator)
                    .map_err(|error| runtime(span, error.to_string()))?;
                element = next;
            }
            weyl_elt_value(context, element, span)
        }
        // W_word_wrapper (atlas-types.w:2373-2382): the canonical reduced
        // word as a plain row (unpadded, unlike vec display).
        "word" => {
            arity(name, arguments, 1, span)?;
            let value = as_weyl_elt(&arguments[0], span)?;
            Ok(Value::List(
                value
                    .word
                    .iter()
                    .map(|&generator| Value::Integer(BigInt::from(generator)))
                    .collect(),
            ))
        }
        // W_elt_unary_eq/neq_wrapper (atlas-types.w:2395-2406): identity
        // test. split_unary_eq/neq_wrapper (atlas-types.w:5055-5064): the
        // zero-pair test. Binary equality is a domain relation (typed.rs)
        // in both cases, so only the unary overloads dispatch here.
        "=" | "!=" => {
            arity(name, arguments, 1, span)?;
            let test = match &arguments[0] {
                Value::Domain(DomainValue::WeylElement(value)) => value.element.is_identity(),
                Value::Domain(DomainValue::Split(value)) => value.is_zero(),
                Value::Domain(DomainValue::KTypePol(value)) => value.terms.is_empty(),
                Value::Domain(DomainValue::ParamPol(value)) => value.terms.is_empty(),
                other => {
                    return Err(type_error(
                        span,
                        format!("expected a WeylElt, Split, KTypePol, or ParamPol, found {other}"),
                    ))
                }
            };
            Ok(Value::Boolean(if name == "=" { test } else { !test }))
        }
        // split_plus/minus_wrapper (atlas-types.w:5079-5095): componentwise.
        "+" => match arguments {
            [Value::Domain(DomainValue::Split(left)), Value::Domain(DomainValue::Split(right))] => {
                Ok(Value::Domain(DomainValue::Split(left.add(*right))))
            }
            // add_K_type_wrapper (atlas-types.w:5668-5679): expand the
            // K-type to final terms and add them (finals_for); the real
            // form mismatch check precedes the no-value gate.
            [Value::Domain(DomainValue::KTypePol(accumulator)), Value::Domain(DomainValue::KType(ktype))] =>
            {
                require_same_form_owner(
                    &accumulator.rf,
                    &ktype.context,
                    "Real form mismatch when adding a KType to a KTypePol",
                    span,
                )?;
                let rc = rep_context(&accumulator.rf);
                let finals = finals_of_final(ktype, &rc, span)?;
                let mut terms = accumulator.terms.clone();
                for (coefficient, term) in finals {
                    merge_pol_term(&mut terms, coefficient, term);
                }
                sort_ktypepol_terms(&mut terms);
                Ok(Value::Domain(DomainValue::KTypePol(KTypePolValue {
                    rf: Arc::clone(&accumulator.rf),
                    terms,
                })))
            }
            // add_K_type_pols_wrapper (atlas-types.w:5780-5791): merge the
            // two polynomials term by term.
            [Value::Domain(DomainValue::KTypePol(accumulator)), Value::Domain(DomainValue::KTypePol(addend))] =>
            {
                require_same_form_owner(
                    &accumulator.rf,
                    &addend.rf,
                    "Real form mismatch when adding two K_types",
                    span,
                )?;
                let mut terms = accumulator.terms.clone();
                for (coefficient, term) in &addend.terms {
                    merge_pol_term(&mut terms, *coefficient, term.clone());
                }
                sort_ktypepol_terms(&mut terms);
                Ok(Value::Domain(DomainValue::KTypePol(KTypePolValue {
                    rf: Arc::clone(&accumulator.rf),
                    terms,
                })))
            }
            // add_K_type_term_wrapper (atlas-types.w:5701-5722): a term
            // with an explicit Split coefficient.
            [Value::Domain(DomainValue::KTypePol(accumulator)), Value::Tuple(term)]
                if matches!(
                    term.as_slice(),
                    [
                        Value::Domain(DomainValue::Split(_)),
                        Value::Domain(DomainValue::KType(_))
                    ]
                ) =>
            {
                let Value::Domain(DomainValue::Split(coefficient)) = &term[0] else {
                    unreachable!()
                };
                let Value::Domain(DomainValue::KType(ktype)) = &term[1] else {
                    unreachable!()
                };
                require_same_form_owner(
                    &accumulator.rf,
                    &ktype.context,
                    "Real form mismatch when adding a term to a K_type",
                    span,
                )?;
                let rc = rep_context(&accumulator.rf);
                let finals = finals_of_final(ktype, &rc, span)?;
                let mut terms = accumulator.terms.clone();
                for (final_coefficient, final_term) in finals {
                    merge_pol_term(&mut terms, final_coefficient.mul(*coefficient), final_term);
                }
                sort_ktypepol_terms(&mut terms);
                Ok(Value::Domain(DomainValue::KTypePol(KTypePolValue {
                    rf: Arc::clone(&accumulator.rf),
                    terms,
                })))
            }
            // add_K_type_termlist_wrapper (atlas-types.w:5741-5775):
            // expand every K-type through finals_for in source-list order.
            [Value::Domain(DomainValue::KTypePol(accumulator)), Value::List(term_list)] => {
                let rc = rep_context(&accumulator.rf);
                let mut terms = accumulator.terms.clone();
                for term in term_list {
                    let Value::Tuple(term) = term else {
                        return Err(type_error(span, "expected a (Split,KType) term"));
                    };
                    let [Value::Domain(DomainValue::Split(coefficient)), Value::Domain(DomainValue::KType(ktype))] =
                        term.as_slice()
                    else {
                        return Err(type_error(span, "expected a (Split,KType) term"));
                    };
                    require_same_form_owner(
                        &accumulator.rf,
                        &ktype.context,
                        "Real form mismatch when adding terms to a K_type",
                        span,
                    )?;
                    for (final_coefficient, final_term) in finals_of_final(ktype, &rc, span)? {
                        terms.push((final_coefficient.mul(*coefficient), final_term));
                    }
                }
                sort_ktypepol_terms(&mut terms);
                let terms = coalesce_sorted_terms(terms);
                Ok(Value::Domain(DomainValue::KTypePol(KTypePolValue {
                    rf: Arc::clone(&accumulator.rf),
                    terms,
                })))
            }
            // add_module_wrapper (atlas-types.w:7786-7795): expand the
            // final parameter and add it (expand_final).
            [Value::Domain(DomainValue::ParamPol(accumulator)), Value::Domain(DomainValue::Param(parameter))] =>
            {
                require_same_form_owner(
                    &accumulator.rf,
                    &parameter.context,
                    "Real form mismatch when adding a Param to a ParamPol",
                    span,
                )?;
                let rc = rep_context(&accumulator.rf);
                let expanded = expand_final(parameter, &rc, span)?;
                let mut terms = accumulator.terms.clone();
                for (coefficient, term) in expanded {
                    merge_pol_term(&mut terms, coefficient, term);
                }
                sort_parampol_terms(&mut terms);
                Ok(Value::Domain(DomainValue::ParamPol(ParamPolValue {
                    rf: Arc::clone(&accumulator.rf),
                    terms,
                })))
            }
            // add_virtual_modules_wrapper (atlas-types.w:7866-7877).
            [Value::Domain(DomainValue::ParamPol(accumulator)), Value::Domain(DomainValue::ParamPol(addend))] =>
            {
                require_same_form_owner(
                    &accumulator.rf,
                    &addend.rf,
                    "Real form mismatch when adding two modules",
                    span,
                )?;
                let mut terms = accumulator.terms.clone();
                for (coefficient, term) in &addend.terms {
                    merge_pol_term(&mut terms, *coefficient, term.clone());
                }
                sort_parampol_terms(&mut terms);
                Ok(Value::Domain(DomainValue::ParamPol(ParamPolValue {
                    rf: Arc::clone(&accumulator.rf),
                    terms,
                })))
            }
            // add_module_term_wrapper / add_module_termlist_wrapper
            // (atlas-types.w:7818-7862): expand parameters to final terms
            // and scale them by the supplied Split coefficient.
            [Value::Domain(DomainValue::ParamPol(accumulator)), Value::Tuple(term)]
                if matches!(
                    term.as_slice(),
                    [
                        Value::Domain(DomainValue::Split(_)),
                        Value::Domain(DomainValue::Param(_))
                    ]
                ) =>
            {
                let Value::Domain(DomainValue::Split(coefficient)) = &term[0] else {
                    unreachable!()
                };
                let Value::Domain(DomainValue::Param(parameter)) = &term[1] else {
                    unreachable!()
                };
                require_same_form_owner(
                    &accumulator.rf,
                    &parameter.context,
                    "Real form mismatch when adding a term to a module",
                    span,
                )?;
                let rc = rep_context(&accumulator.rf);
                let mut terms = accumulator.terms.clone();
                for (final_coefficient, final_term) in expand_final(parameter, &rc, span)? {
                    merge_pol_term(&mut terms, final_coefficient.mul(*coefficient), final_term);
                }
                sort_parampol_terms(&mut terms);
                Ok(Value::Domain(DomainValue::ParamPol(ParamPolValue {
                    rf: Arc::clone(&accumulator.rf),
                    terms,
                })))
            }
            [Value::Domain(DomainValue::ParamPol(accumulator)), Value::List(term_list)] => {
                let rc = rep_context(&accumulator.rf);
                let mut terms = accumulator.terms.clone();
                for term in term_list {
                    let Value::Tuple(term) = term else {
                        return Err(type_error(span, "expected a (Split,Param) term"));
                    };
                    let [Value::Domain(DomainValue::Split(coefficient)), Value::Domain(DomainValue::Param(parameter))] =
                        term.as_slice()
                    else {
                        return Err(type_error(span, "expected a (Split,Param) term"));
                    };
                    require_same_form_owner(
                        &accumulator.rf,
                        &parameter.context,
                        "Real form mismatch when adding terms to a module",
                        span,
                    )?;
                    for (final_coefficient, final_term) in expand_final(parameter, &rc, span)? {
                        terms.push((final_coefficient.mul(*coefficient), final_term));
                    }
                }
                sort_parampol_terms(&mut terms);
                let terms = coalesce_sorted_terms(terms);
                Ok(Value::Domain(DomainValue::ParamPol(ParamPolValue {
                    rf: Arc::clone(&accumulator.rf),
                    terms,
                })))
            }
            _ => Err(Diagnostic::new(
                ErrorKind::Name,
                format!("undefined function `{name}`"),
                Some(span),
            )),
        },
        // split_minus_wrapper and split_unary_minus_wrapper
        // (atlas-types.w:5085-5100): binary difference, unary negation.
        "-" => match arguments {
            [Value::Domain(DomainValue::Split(value))] => {
                Ok(Value::Domain(DomainValue::Split(value.neg())))
            }
            [Value::Domain(DomainValue::Split(left)), Value::Domain(DomainValue::Split(right))] => {
                Ok(Value::Domain(DomainValue::Split(left.sub(*right))))
            }
            // subtract_K_type_wrapper (atlas-types.w:5682-5693): add the
            // final terms with negated coefficients.
            [Value::Domain(DomainValue::KTypePol(accumulator)), Value::Domain(DomainValue::KType(ktype))] =>
            {
                require_same_form_owner(
                    &accumulator.rf,
                    &ktype.context,
                    "Real form mismatch when subtracting a KType from a KTypePol",
                    span,
                )?;
                let rc = rep_context(&accumulator.rf);
                let finals = finals_of_final(ktype, &rc, span)?;
                let mut terms = accumulator.terms.clone();
                for (coefficient, term) in finals {
                    merge_pol_term(&mut terms, coefficient.neg(), term);
                }
                sort_ktypepol_terms(&mut terms);
                Ok(Value::Domain(DomainValue::KTypePol(KTypePolValue {
                    rf: Arc::clone(&accumulator.rf),
                    terms,
                })))
            }
            // subtract_K_type_pols_wrapper (atlas-types.w:5794-5805).
            [Value::Domain(DomainValue::KTypePol(accumulator)), Value::Domain(DomainValue::KTypePol(subtrahend))] =>
            {
                require_same_form_owner(
                    &accumulator.rf,
                    &subtrahend.rf,
                    "Real form mismatch when subtracting two K_types",
                    span,
                )?;
                let mut terms = accumulator.terms.clone();
                for (coefficient, term) in &subtrahend.terms {
                    merge_pol_term(&mut terms, coefficient.neg(), term.clone());
                }
                sort_ktypepol_terms(&mut terms);
                Ok(Value::Domain(DomainValue::KTypePol(KTypePolValue {
                    rf: Arc::clone(&accumulator.rf),
                    terms,
                })))
            }
            // subtract_module_wrapper (atlas-types.w:7798-7807).
            [Value::Domain(DomainValue::ParamPol(accumulator)), Value::Domain(DomainValue::Param(parameter))] =>
            {
                require_same_form_owner(
                    &accumulator.rf,
                    &parameter.context,
                    "Real form mismatch when subtracting a Param from a ParamPol",
                    span,
                )?;
                let rc = rep_context(&accumulator.rf);
                let expanded = expand_final(parameter, &rc, span)?;
                let mut terms = accumulator.terms.clone();
                for (coefficient, term) in expanded {
                    merge_pol_term(&mut terms, coefficient.neg(), term);
                }
                sort_parampol_terms(&mut terms);
                Ok(Value::Domain(DomainValue::ParamPol(ParamPolValue {
                    rf: Arc::clone(&accumulator.rf),
                    terms,
                })))
            }
            // subtract_virtual_modules_wrapper (atlas-types.w:7880-7891).
            [Value::Domain(DomainValue::ParamPol(accumulator)), Value::Domain(DomainValue::ParamPol(subtrahend))] =>
            {
                require_same_form_owner(
                    &accumulator.rf,
                    &subtrahend.rf,
                    "Real form mismatch when subtracting two modules",
                    span,
                )?;
                let mut terms = accumulator.terms.clone();
                for (coefficient, term) in &subtrahend.terms {
                    merge_pol_term(&mut terms, coefficient.neg(), term.clone());
                }
                sort_parampol_terms(&mut terms);
                Ok(Value::Domain(DomainValue::ParamPol(ParamPolValue {
                    rf: Arc::clone(&accumulator.rf),
                    terms,
                })))
            }
            _ => Err(Diagnostic::new(
                ErrorKind::Name,
                format!("undefined function `{name}`"),
                Some(span),
            )),
        },
        // W_elt_prod_wrapper (atlas-types.w:2421-2432): the group product.
        // split_times_wrapper (atlas-types.w:5102-5107): the dual product
        // (e1e2+f1f2, e1f2+f1e2).
        "*" => match arguments {
            [Value::Domain(DomainValue::LieType(left)), Value::Domain(DomainValue::LieType(right))] =>
            {
                validate_combined_rank(left, right, span)?;
                let mut factors = left.factors.clone();
                factors.extend(right.factors.iter().copied());
                Ok(Value::Domain(DomainValue::LieType(LieTypeValue {
                    factors,
                })))
            }
            [Value::Domain(DomainValue::WeylElement(left)), Value::Domain(DomainValue::WeylElement(right))] =>
            {
                if left.context.handle != right.context.handle {
                    return Err(runtime(span, "Weyl group mismatch"));
                }
                let product = left
                    .element
                    .multiply(&left.context.system, &right.element)
                    .map_err(|error| runtime(span, error.to_string()))?;
                weyl_elt_value(Arc::clone(&left.context), product, span)
            }
            [Value::Domain(DomainValue::WeylElement(weyl)), Value::Vector(vector)] => {
                validate_weight_rank(weyl, vector, span)?;
                let mut acted = vector.0.clone();
                word_act_weight(&weyl.context.handle.datum, &weyl.word, &mut acted);
                Ok(Value::Vector(Vec32(acted)))
            }
            [Value::Vector(vector), Value::Domain(DomainValue::WeylElement(weyl))] => {
                validate_coweight_rank(vector, weyl, span)?;
                let mut acted = vector.0.clone();
                for &generator in &weyl.word {
                    simple_coreflect(&weyl.context.handle.datum, generator, &mut acted);
                }
                Ok(Value::Vector(Vec32(acted)))
            }
            [Value::Domain(DomainValue::Split(left)), Value::Domain(DomainValue::Split(right))] => {
                Ok(Value::Domain(DomainValue::Split(left.mul(*right))))
            }
            // int_mult_K_type_pol_wrapper (atlas-types.w:5821-5840) and
            // int_mult_virtual_module_wrapper (atlas-types.w:7907-7926):
            // scale every coefficient by the Atlas int (both Split
            // components, arithmetic.h:187).
            [Value::Integer(scalar), Value::Domain(DomainValue::KTypePol(pol))] => {
                let scalar = narrow_split_component(scalar, span)?;
                let terms = pol
                    .terms
                    .iter()
                    .map(|(coefficient, ktype)| {
                        (
                            SplitValue::new(
                                coefficient.e().wrapping_mul(scalar),
                                coefficient.f().wrapping_mul(scalar),
                            ),
                            ktype.clone(),
                        )
                    })
                    .filter(|(coefficient, _)| !coefficient.is_zero())
                    .collect();
                Ok(Value::Domain(DomainValue::KTypePol(KTypePolValue {
                    rf: Arc::clone(&pol.rf),
                    terms,
                })))
            }
            [Value::Integer(scalar), Value::Domain(DomainValue::ParamPol(pol))] => {
                let scalar = narrow_split_component(scalar, span)?;
                let terms = pol
                    .terms
                    .iter()
                    .map(|(coefficient, repr)| {
                        (
                            SplitValue::new(
                                coefficient.e().wrapping_mul(scalar),
                                coefficient.f().wrapping_mul(scalar),
                            ),
                            repr.clone(),
                        )
                    })
                    .filter(|(coefficient, _)| !coefficient.is_zero())
                    .collect();
                Ok(Value::Domain(DomainValue::ParamPol(ParamPolValue {
                    rf: Arc::clone(&pol.rf),
                    terms,
                })))
            }
            // split_mult_K_type_pol_wrapper (atlas-types.w:5863-5907) and
            // split_mult_virtual_module_wrapper (atlas-types.w:7949-7994):
            // scale every coefficient by the Split, dropping the terms a
            // zero-divisor factor (a multiple of 1-s or 1+s) kills.
            [Value::Domain(DomainValue::Split(scalar)), Value::Domain(DomainValue::KTypePol(pol))] =>
            {
                let terms = pol
                    .terms
                    .iter()
                    .filter(|(coefficient, _)| split_keeps(coefficient, *scalar))
                    .map(|(coefficient, ktype)| (coefficient.mul(*scalar), ktype.clone()))
                    .collect();
                Ok(Value::Domain(DomainValue::KTypePol(KTypePolValue {
                    rf: Arc::clone(&pol.rf),
                    terms,
                })))
            }
            [Value::Domain(DomainValue::Split(scalar)), Value::Domain(DomainValue::ParamPol(pol))] =>
            {
                let terms = pol
                    .terms
                    .iter()
                    .filter(|(coefficient, _)| split_keeps(coefficient, *scalar))
                    .map(|(coefficient, repr)| (coefficient.mul(*scalar), repr.clone()))
                    .collect();
                Ok(Value::Domain(DomainValue::ParamPol(ParamPolValue {
                    rf: Arc::clone(&pol.rf),
                    terms,
                })))
            }
            // scale_parameter_wrapper (atlas-types.w:6582-6592,
            // (Param,rat)): scale the infinitesimal character by the rat
            // (repr.cpp:701-709).
            [Value::Domain(DomainValue::Param(parameter)), Value::Rational(factor)] => {
                let (numerator, denominator) = rational_pair(factor, span)?;
                let rc = rep_context(&parameter.context);
                let repr = rc
                    .scale(&parameter.repr, numerator, denominator)
                    .map_err(|error| structure_diagnostic(error, span))?;
                Ok(Value::Domain(DomainValue::Param(ParamValue {
                    context: Arc::clone(&parameter.context),
                    repr,
                })))
            }
            // scale_poly_wrapper (atlas-types.w:8058-8066,
            // (ParamPol,rat)): scale every parameter and re-expand through
            // finals_for (repr.cpp:1161-1170).
            [Value::Domain(DomainValue::ParamPol(pol)), Value::Rational(factor)] => {
                let (numerator, denominator) = rational_pair(factor, span)?;
                let rc = rep_context(&pol.rf);
                let mut terms: Vec<(SplitValue, StandardRepr)> = Vec::new();
                for (coefficient, repr) in &pol.terms {
                    let scaled = rc
                        .scale(repr, numerator, denominator)
                        .map_err(|error| structure_diagnostic(error, span))?;
                    let finals = rc
                        .expand_final(&scaled)
                        .map_err(|error| structure_diagnostic(error, span))?;
                    for (final_repr, multiplicity) in finals {
                        merge_pol_term(
                            &mut terms,
                            coefficient.mul(SplitValue::new(multiplicity, 0)),
                            final_repr,
                        );
                    }
                }
                sort_parampol_terms(&mut terms);
                Ok(Value::Domain(DomainValue::ParamPol(ParamPolValue {
                    rf: Arc::clone(&pol.rf),
                    terms,
                })))
            }
            _ => Err(Diagnostic::new(
                ErrorKind::Name,
                format!("undefined function `{name}`"),
                Some(span),
            )),
        },
        // W_elt_invert_wrapper (atlas-types.w:2433-2440).
        "/" => match arguments {
            [Value::Domain(DomainValue::WeylElement(value))] => {
                weyl_elt_value(Arc::clone(&value.context), value.element.inverse(), span)
            }
            _ => Err(Diagnostic::new(
                ErrorKind::Name,
                format!("undefined function `{name}`"),
                Some(span),
            )),
        },
        // W_elt_gen_prod_wrapper / W_gen_elt_prod_wrapper
        // (atlas-types.w:2456-2476): multiplication by one simple
        // generator on either side; check_Weyl_gen echoes the signed index.
        // size_of_block_wrapper (atlas-types.w:4820-4824): the block size.
        "#" => match arguments {
            [Value::Domain(DomainValue::WeylElement(value)), Value::Integer(generator)] => {
                let rank = value.context.handle.datum.semisimple_rank();
                let generator = check_weyl_generator(generator, rank, span)?;
                let (product, _) = value
                    .element
                    .right_multiply_simple(&value.context.system, generator)
                    .map_err(|error| runtime(span, error.to_string()))?;
                weyl_elt_value(Arc::clone(&value.context), product, span)
            }
            [Value::Integer(generator), Value::Domain(DomainValue::WeylElement(value))] => {
                let rank = value.context.handle.datum.semisimple_rank();
                let generator_index = check_weyl_generator(generator, rank, span)?;
                let (product, _) = value
                    .element
                    .left_multiply_simple(&value.context.system, generator_index)
                    .map_err(|error| runtime(span, error.to_string()))?;
                weyl_elt_value(Arc::clone(&value.context), product, span)
            }
            [Value::Domain(DomainValue::Block(block))] => {
                Ok(Value::Integer(BigInt::from(block.graph.size())))
            }
            // K_type_pol_size_wrapper (atlas-types.w:5594-5600) and
            // virtual_module_size_wrapper (atlas-types.w:7671-7677): the
            // TERM count.
            [Value::Domain(DomainValue::KTypePol(pol))] => {
                Ok(Value::Integer(BigInt::from(pol.terms.len())))
            }
            [Value::Domain(DomainValue::ParamPol(pol))] => {
                Ok(Value::Integer(BigInt::from(pol.terms.len())))
            }
            _ => Err(Diagnostic::new(
                ErrorKind::Name,
                format!("undefined function `{name}`"),
                Some(span),
            )),
        },
        // W_elt_word_prod_wrapper / W_word_elt_prod_wrapper
        // (atlas-types.w:2478-2499). A WeylWord denotes
        // s[word[0]]...s[word[n-1]], hence left multiplication applies its
        // letters in reverse while right multiplication applies forwards.
        "##" => match arguments {
            [Value::Domain(DomainValue::WeylElement(value)), word] => {
                let rank = value.context.handle.datum.semisimple_rank();
                let word = check_weyl_word(word, rank, span)?;
                let mut product = value.element.clone();
                for generator in word {
                    product = product
                        .right_multiply_simple(&value.context.system, generator)
                        .map_err(|error| runtime(span, error.to_string()))?
                        .0;
                }
                weyl_elt_value(Arc::clone(&value.context), product, span)
            }
            [word, Value::Domain(DomainValue::WeylElement(value))] => {
                let rank = value.context.handle.datum.semisimple_rank();
                let word = check_weyl_word(word, rank, span)?;
                let mut product = value.element.clone();
                for generator in word.into_iter().rev() {
                    product = product
                        .left_multiply_simple(&value.context.system, generator)
                        .map_err(|error| runtime(span, error.to_string()))?
                        .0;
                }
                weyl_elt_value(Arc::clone(&value.context), product, span)
            }
            _ => Err(Diagnostic::new(
                ErrorKind::Name,
                format!("undefined function `{name}`"),
                Some(span),
            )),
        },
        unknown => Err(Diagnostic::new(
            ErrorKind::Name,
            format!("undefined function `{unknown}`"),
            Some(span),
        )),
    }
}

fn narrow_ann_modulus(value: &Value, span: SourceSpan) -> Result<i32, Diagnostic> {
    let modulus = as_integer(value, span)?;
    i32::try_from(&modulus).map_err(|_| runtime(span, "Integer value to big for conversion"))
}

/// Narrow an Atlas int into one Split component (upstream
/// `big_int::int_val`, utilities/bigint.cpp:142-146, typo included).
fn narrow_split_component(value: &BigInt, span: SourceSpan) -> Result<i32, Diagnostic> {
    i32::try_from(value).map_err(|_| runtime(span, "Integer value to big for conversion"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::{SourceId, SourcePosition};
    use crate::value::Matrix;

    #[test]
    fn strong_components_probe() {
        let g1 = vec![vec![1], vec![2], vec![0]];
        let (p, i) = super::strong_components(&g1);
        eprintln!("g1 partition={p:?} induced={i:?}");
        let g2 = vec![vec![1, 2], vec![], vec![2]];
        let (p, i) = super::strong_components(&g2);
        eprintln!("g2 partition={p:?} induced={i:?}");
    }

    fn span() -> SourceSpan {
        SourceSpan::new(
            SourceId::anonymous(),
            0,
            0,
            SourcePosition { line: 1, column: 1 },
            SourcePosition { line: 1, column: 1 },
        )
    }

    #[test]
    fn hungry_products_match_the_oracle_values_and_diagnostics() {
        let left = call("Lie_type", &[Value::String("A1".into())], span()).unwrap();
        let right = call("Lie_type", &[Value::String("B2".into())], span()).unwrap();
        assert_eq!(
            call("*", &[left, right], span()).unwrap().to_string(),
            "Lie type 'A1.B2'"
        );
        let too_large_left = call("Lie_type", &[Value::String("A16".into())], span()).unwrap();
        let too_large_right = call("Lie_type", &[Value::String("B17".into())], span()).unwrap();
        assert_eq!(
            call(
                "*",
                &[too_large_left.clone(), too_large_right.clone()],
                span()
            )
            .unwrap_err()
            .message,
            "Combined rank 33 exceeds implementation limit 32"
        );
        assert_eq!(
            validate("*", &[too_large_left, too_large_right], span())
                .unwrap_err()
                .message,
            "Combined rank 33 exceeds implementation limit 32"
        );

        let datum = call(
            "simply_connected",
            &[
                call("Lie_type", &[Value::String("A2".into())], span()).unwrap(),
                Value::Boolean(true),
            ],
            span(),
        )
        .unwrap();
        let weyl = call(
            "W_elt",
            &[datum, Value::List(vec![Value::Integer(BigInt::from(0))])],
            span(),
        )
        .unwrap();
        let vector = Value::Vector(Vec32(vec![1, 2]));
        assert_eq!(
            call("*", &[weyl.clone(), vector.clone()], span())
                .unwrap()
                .to_string(),
            "[ -1,  3 ]"
        );
        assert_eq!(
            call(
                "*",
                &[Value::Vector(Vec32(vec![-1, 3])), weyl.clone()],
                span()
            )
            .unwrap()
            .to_string(),
            "[ 4, 3 ]"
        );
        assert_eq!(
            call("*", &[weyl.clone(), Value::Vector(Vec32(vec![1]))], span())
                .unwrap_err()
                .message,
            "Rank and weight size mismatch 2:1"
        );
        assert_eq!(
            call("*", &[Value::Vector(Vec32(vec![1])), weyl], span())
                .unwrap_err()
                .message,
            "Coweight size and rank mismatch 1:2"
        );
    }

    #[test]
    fn matrix_adapter_consumes_column_major_mat_values_as_rows() {
        let matrix = Matrix::from_columns(2, 2, vec![1, 3, 2, 4]).expect("valid matrix");
        assert_eq!(
            as_matrix(&Value::Matrix(matrix), span()).expect("mat value"),
            vec![vec![1, 2], vec![3, 4]]
        );
        let error = as_matrix(
            &Value::List(vec![Value::List(vec![
                Value::Integer(1.into()),
                Value::Integer(0.into()),
            ])]),
            span(),
        )
        .expect_err("rectangular legacy list");
        assert_eq!(error.kind, ErrorKind::Type);
        assert_eq!(error.message, "expected a square mat");
    }

    #[test]
    fn root_datum_retains_type_isogeny_and_coroot_preference() {
        let lie_type = call("Lie_type", &[Value::String("A1".into())], span()).expect("Lie type");
        let datum = call(
            "simply_connected",
            &[lie_type, Value::Boolean(true)],
            span(),
        )
        .expect("root datum");
        assert_eq!(
            call("Lie_type", std::slice::from_ref(&datum), span())
                .expect("type attribute")
                .to_string(),
            "Lie type 'A1'"
        );
        assert_eq!(
            call("prefers_coroots", std::slice::from_ref(&datum), span()),
            Ok(Value::Boolean(true))
        );
        assert_eq!(
            datum.to_string(),
            "simply connected root datum of Lie type 'A1'"
        );
    }

    #[test]
    fn rational_coordinates_normalise_to_one_ratvec_denominator() {
        let factor = ratvec_from_rationals(
            vec![
                BigRational::from_signeds(1i64, 2),
                BigRational::from_signeds(1i64, 3),
            ],
            span(),
        )
        .expect("small rational coordinates");
        assert_eq!(factor.numerators(), &[3, 2]);
        assert_eq!(factor.denominator(), 6);
    }

    #[test]
    fn cartan_matrix_is_returned_as_a_column_major_mat_value() {
        let lie_type = call("Lie_type", &[Value::String("B2".into())], span()).expect("Lie type");
        let datum = call(
            "simply_connected",
            &[lie_type, Value::Boolean(false)],
            span(),
        )
        .expect("root datum");
        let matrix = call("Cartan_matrix", &[datum], span()).expect("Cartan matrix");
        let Value::Matrix(matrix) = matrix else {
            panic!("Cartan_matrix must return mat")
        };
        assert_eq!(matrix.rows(), 2);
        assert_eq!(matrix.cols(), 2);
        assert_eq!(matrix.entry(0, 0), Some(2));
        assert_eq!(matrix.entry(0, 1), Some(-2));
        assert_eq!(matrix.entry(1, 0), Some(-1));
        assert_eq!(matrix.entry(1, 1), Some(2));
    }

    #[test]
    fn quotient_datum_applies_the_lattice_basis_exactly() {
        let lie_type = call("Lie_type", &[Value::String("A1".into())], span()).expect("Lie type");
        let identity = Value::Matrix(Matrix::from_columns(1, 1, vec![1]).expect("identity"));
        let simply_connected = call(
            "root_datum",
            &[lie_type.clone(), identity, Value::Boolean(true)],
            span(),
        )
        .expect("weight lattice");
        assert_eq!(
            simply_connected.to_string(),
            "simply connected root datum of Lie type 'A1'"
        );
        assert_eq!(
            call(
                "prefers_coroots",
                std::slice::from_ref(&simply_connected),
                span()
            ),
            Ok(Value::Boolean(true))
        );

        let root_lattice =
            Value::Matrix(Matrix::from_columns(1, 1, vec![2]).expect("root lattice"));
        let adjoint = call(
            "root_datum",
            &[lie_type.clone(), root_lattice, Value::Boolean(false)],
            span(),
        )
        .expect("root lattice quotient");
        assert_eq!(adjoint.to_string(), "adjoint root datum of Lie type 'A1'");

        let dependent = Value::Matrix(Matrix::from_columns(1, 1, vec![0]).expect("zero matrix"));
        let error = call(
            "root_datum",
            &[lie_type, dependent, Value::Boolean(false)],
            span(),
        )
        .expect_err("dependent generators");
        assert_eq!(error.message, "Dependent lattice generators");

        let lie_type = call("Lie_type", &[Value::String("A1".into())], span()).expect("Lie type");
        let too_small =
            Value::Matrix(Matrix::from_columns(1, 1, vec![3]).expect("index-three lattice"));
        let error = call(
            "root_datum",
            &[lie_type, too_small, Value::Boolean(false)],
            span(),
        )
        .expect_err("lattice must contain the A1 root lattice");
        assert_eq!(
            error.message,
            "Sub-lattice does not contain the root lattice"
        );

        let roots = Value::Matrix(Matrix::from_columns(1, 1, vec![2]).expect("A1 root"));
        let coroots = Value::Matrix(Matrix::from_columns(1, 1, vec![1]).expect("A1 coroot"));
        let explicit = call(
            "root_datum",
            &[roots, coroots, Value::Boolean(true)],
            span(),
        )
        .expect("explicit root datum");
        assert_eq!(
            explicit.to_string(),
            "simply connected root datum of Lie type 'A1'"
        );

        let quotient_matrix =
            Value::Matrix(Matrix::from_columns(1, 1, vec![2]).expect("root-lattice basis"));
        let quotient = call("root_datum", &[explicit.clone(), quotient_matrix], span())
            .expect("existing-datum quotient");
        assert_eq!(quotient.to_string(), "adjoint root datum of Lie type 'A1'");
    }

    #[test]
    fn explicit_root_datum_accepts_a_simultaneous_dynkin_relabeling() {
        let roots = Value::Matrix(
            Matrix::from_columns(3, 3, vec![2, -1, 0, 0, -1, 2, -1, 2, -1])
                .expect("permuted A3 roots"),
        );
        let coroots = Value::Matrix(
            Matrix::from_columns(3, 3, vec![1, 0, 0, 0, 0, 1, 0, 1, 0])
                .expect("permuted A3 coroots"),
        );
        let datum = call(
            "root_datum",
            &[roots, coroots, Value::Boolean(true)],
            span(),
        )
        .expect("permuted A3 should be recognized");
        assert_eq!(
            datum.to_string(),
            "simply connected root datum of Lie type 'A3'"
        );
    }

    #[test]
    fn cartan_inference_prefers_canonical_b2_and_c2_before_relabeling() {
        for (letter, expected) in [('B', "B2"), ('C', "C2")] {
            let inferred = infer_lie_type(&factor_cartan(letter, 2), 2, span())
                .expect("canonical rank-two Cartan matrix");
            assert_eq!(inferred.render(), expected);
        }
    }

    #[test]
    fn cartan_inference_accepts_a_non_symmetric_b3_relabeling() {
        let canonical = factor_cartan('B', 3);
        let permutation = [2, 0, 1];
        let relabeled = permutation
            .iter()
            .map(|&row| {
                permutation
                    .iter()
                    .map(|&column| canonical[row][column])
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        assert_ne!(relabeled, canonical);
        let inferred = infer_lie_type(&relabeled, 3, span()).expect("permuted B3 Cartan matrix");
        assert_eq!(inferred.render(), "B3");
    }

    #[test]
    fn zero_row_matrices_do_not_collapse_into_zero_by_zero_shapes() {
        let matrix = Value::Matrix(Matrix::from_columns(0, 1, Vec::new()).expect("0x1 matrix"));
        let error = call(
            "inner_class",
            &[
                call(
                    "simply_connected",
                    &[
                        call("Lie_type", &[Value::String("A1".into())], span()).unwrap(),
                        Value::Boolean(true),
                    ],
                    span(),
                )
                .unwrap(),
                matrix,
            ],
            span(),
        )
        .expect_err("a 0x1 involution is not square");
        assert!(error.message.contains("square mat"));
    }

    #[test]
    fn real_form_recovers_its_shared_inner_class() {
        let lie_type = call("Lie_type", &[Value::String("A1".into())], span()).expect("Lie type");
        let datum = call(
            "simply_connected",
            &[lie_type, Value::Boolean(false)],
            span(),
        )
        .expect("root datum");
        let matrix =
            Value::Matrix(Matrix::from_columns(1, 1, vec![1]).expect("identity involution"));
        let inner = call("inner_class", &[datum, matrix], span()).expect("inner class");
        let real = call(
            "real_form",
            &[inner.clone(), Value::Integer(BigInt::from(1))],
            span(),
        )
        .expect("real form");
        assert_eq!(call("inner_class", &[real], span()), Ok(inner));
    }

    #[test]
    fn kgb_involution_is_returned_as_a_lattice_matrix() {
        let lie_type = call("Lie_type", &[Value::String("A1".into())], span()).expect("Lie type");
        let datum = call(
            "simply_connected",
            &[lie_type, Value::Boolean(false)],
            span(),
        )
        .expect("root datum");
        let matrix =
            Value::Matrix(Matrix::from_columns(1, 1, vec![1]).expect("identity involution"));
        let inner = call("inner_class", &[datum, matrix], span()).expect("inner class");
        let real = call(
            "real_form",
            &[inner, Value::Integer(BigInt::from(1))],
            span(),
        )
        .expect("split real form");
        let element =
            call("KGB", &[real, Value::Integer(BigInt::from(0))], span()).expect("KGB element");
        let involution = call("involution", &[element], span()).expect("KGB involution");
        let Value::Matrix(involution) = involution else {
            panic!("involution must return mat")
        };
        assert_eq!(involution.rows(), 1);
        assert_eq!(involution.cols(), 1);
        assert_eq!(involution.entry(0, 0), Some(1));
    }

    #[test]
    fn torus_factor_crosses_the_adapter_as_a_ratvec() {
        let lie_type = call("Lie_type", &[Value::String("A1".into())], span()).expect("Lie type");
        let datum = call(
            "simply_connected",
            &[lie_type, Value::Boolean(false)],
            span(),
        )
        .expect("root datum");
        let matrix =
            Value::Matrix(Matrix::from_columns(1, 1, vec![1]).expect("identity involution matrix"));
        let inner_class = call("inner_class", &[datum, matrix], span()).expect("inner class");
        let real_form = call(
            "quasisplit_form",
            std::slice::from_ref(&inner_class),
            span(),
        )
        .expect("quasisplit form");
        let element = call("KGB", &[real_form, Value::Integer(BigInt::from(0))], span())
            .expect("KGB element");
        let factor = call("torus_factor", &[element], span()).expect("torus factor");
        assert!(matches!(factor, Value::RatVector(_)));
    }

    fn sl2r_split_form() -> Value {
        let lie_type = call("Lie_type", &[Value::String("A1".into())], span()).expect("Lie type");
        let datum = call(
            "simply_connected",
            &[lie_type, Value::Boolean(true)],
            span(),
        )
        .expect("root datum");
        let matrix =
            Value::Matrix(Matrix::from_columns(1, 1, vec![1]).expect("identity involution"));
        let inner = call("inner_class", &[datum, matrix], span()).expect("inner class");
        call(
            "real_form",
            &[inner, Value::Integer(BigInt::from(1))],
            span(),
        )
        .expect("split real form")
    }

    #[test]
    fn decompose_returns_the_real_form_and_element_number() {
        let real = sl2r_split_form();
        let element =
            call("KGB", &[real, Value::Integer(BigInt::from(0))], span()).expect("KGB element");
        let decomposed = call("%", &[element], span()).expect("decompose");
        let Value::Tuple(parts) = &decomposed else {
            panic!("decompose must return a tuple")
        };
        assert_eq!(parts.len(), 2);
        assert!(matches!(parts[0], Value::Domain(DomainValue::RealForm(_))));
        assert_eq!(parts[1], Value::Integer(BigInt::from(0)));
        assert_eq!(
            decomposed.to_string(),
            "(connected split real group with Lie algebra 'sl(2,R)',0)"
        );
    }

    #[test]
    fn distinguished_twist_fixes_sl2r_elements() {
        let real = sl2r_split_form();
        for index in [0, 2] {
            let element = call(
                "KGB",
                &[real.clone(), Value::Integer(BigInt::from(index))],
                span(),
            )
            .expect("KGB element");
            let twisted = call("twist", std::slice::from_ref(&element), span()).expect("twist");
            assert_eq!(twisted, element);
        }
    }

    #[test]
    fn p3_param_twist_preserves_owner_and_matches_oracle_values() {
        let datum = fixture_datum("A2", true);
        let flip = matrix(2, 2, vec![0, 1, 1, 0]);
        let inner = call("inner_class", &[datum, flip.clone()], span()).expect("inner class");
        let real = call("real_form", &[inner, int(0)], span()).expect("real form");
        let element = call("KGB", &[real, int(1)], span()).expect("KGB element");
        let parameter = call(
            "param",
            &[
                element,
                Value::Vector(Vec32(vec![0, 0])),
                Value::RatVector(RatVec::new(vec![0, 0], 1).expect("ratvec")),
            ],
            span(),
        )
        .expect("parameter");
        let Value::Domain(DomainValue::Param(source)) = &parameter else {
            panic!("param constructor must return a Param")
        };

        for (arguments, expected) in [
            (
                vec![parameter.clone()],
                "final parameter(x=0,lambda=[0,1]/1,nu=[0,0]/1)",
            ),
            (
                vec![parameter.clone(), flip],
                "non-dominant parameter(x=2,lambda=[1,1]/1,nu=[0,0]/1)",
            ),
            (
                vec![parameter.clone(), matrix(2, 2, vec![1, 0, 0, 1])],
                "non-dominant parameter(x=1,lambda=[1,1]/1,nu=[0,0]/1)",
            ),
        ] {
            let twisted = call("twist", &arguments, span()).expect("parameter twist");
            assert_eq!(twisted.to_string(), expected);
            let Value::Domain(DomainValue::Param(target)) = twisted else {
                panic!("twist(Param) must return a Param")
            };
            assert!(Arc::ptr_eq(&target.context, &source.context));
        }
    }

    #[test]
    fn p3_param_outer_twist_validation_dispatches_without_regressing_kgb() {
        let datum = fixture_datum("A2", true);
        let flip = matrix(2, 2, vec![0, 1, 1, 0]);
        let inner = call("inner_class", &[datum, flip], span()).expect("inner class");
        let real = call("real_form", &[inner, int(0)], span()).expect("real form");
        let element = call("KGB", &[real, int(1)], span()).expect("KGB element");
        let parameter = call(
            "param",
            &[
                element.clone(),
                Value::Vector(Vec32(vec![0, 0])),
                Value::RatVector(RatVec::new(vec![0, 0], 1).expect("ratvec")),
            ],
            span(),
        )
        .expect("parameter");
        let invalid = matrix(1, 1, vec![1]);

        for first in [parameter, element] {
            let error = validate("twist", &[first, invalid.clone()], span())
                .expect_err("outer twist validates the matrix for both overload families");
            assert_eq!(
                error.message,
                "Involution should be a 2x2 matrix; received a 1x1 matrix"
            );
        }
    }

    #[test]
    fn p3_unary_param_twist_maps_make_dominant_nonstandard_error_exactly() {
        let real = sl2r_split_form();
        let parameter = sl2r_param(&real, 1, &[-2], &[0], 1);

        let error = call("twist", std::slice::from_ref(&parameter), span())
            .expect_err("inner twist first makes the parameter dominant");
        assert_eq!(error.message, "Non standard parameter in make_dominant");

        let identity = matrix(1, 1, vec![1]);
        assert_eq!(
            call("twist", &[parameter.clone(), identity], span())
                .expect("the raw outer twist does not run make_dominant"),
            parameter
        );
    }

    #[test]
    fn p3_undefined_outer_twists_match_oracle_and_reject_follow_up_safely() {
        let datum = fixture_datum("A3", true);
        let identity = matrix(3, 3, vec![1, 0, 0, 0, 1, 0, 0, 0, 1]);
        let anti_diagonal = matrix(3, 3, vec![0, 0, 1, 0, 1, 0, 1, 0, 0]);
        let inner = call("inner_class", &[datum, identity], span()).expect("inner class");
        let real = call("real_form", &[inner, int(0)], span()).expect("compact form");
        let element = call("KGB", &[real.clone(), int(0)], span()).expect("KGB element");

        let undefined_kgb = call("twist", &[element, anti_diagonal.clone()], span())
            .expect("KGB twist preserves UndefKGB");
        assert_eq!(undefined_kgb.to_string(), "KGB element #4294967295");
        let Value::Domain(DomainValue::KgbElement(_, id)) = &undefined_kgb else {
            panic!("twist(KGBElt,mat) returns KGBElt")
        };
        assert!(id.is_undefined());
        assert_eq!(
            call("length", std::slice::from_ref(&undefined_kgb), span())
                .expect_err("undefined KGB follow-up is rejected")
                .message,
            "Inexistent KGB element"
        );

        let parameter = call(
            "param",
            &[
                call("KGB", &[real.clone(), int(0)], span()).expect("KGB element"),
                Value::Vector(Vec32(vec![0, 0, 0])),
                Value::RatVector(RatVec::new(vec![0, 0, 0], 1).expect("ratvec")),
            ],
            span(),
        )
        .expect("parameter");
        let undefined_param = call("twist", &[parameter, anti_diagonal], span())
            .expect("Param twist preserves UndefKGB");
        assert_eq!(
            undefined_param.to_string(),
            "final parameter(x=4294967295,lambda=[1,1,1]/1,nu=[0,0,0]/1)"
        );
        assert_eq!(
            call("height", std::slice::from_ref(&undefined_param), span())
                .expect("height is stored independently of the graph"),
            int(10)
        );
        assert_eq!(
            call("real_form", std::slice::from_ref(&undefined_param), span())
                .expect("the owner is stored independently of the graph"),
            real
        );
        assert_eq!(
            call("%", std::slice::from_ref(&undefined_param), span())
                .expect("cached fields support parameter decomposition")
                .to_string(),
            "(KGB element #4294967295,[ 0, 0, 0 ],[ 1, 1, 1 ]/1)"
        );
    }

    fn sl2r_param(real: &Value, x: i64, lambda: &[i32], nu: &[i64], nu_denominator: u64) -> Value {
        let element = call(
            "KGB",
            &[real.clone(), Value::Integer(BigInt::from(x))],
            span(),
        )
        .expect("KGB element");
        call(
            "param",
            &[
                element,
                Value::Vector(Vec32(lambda.to_vec())),
                Value::RatVector(RatVec::new(nu.to_vec(), nu_denominator).expect("ratvec")),
            ],
            span(),
        )
        .expect("param")
    }

    fn kl_column_single_raw_row(parameter: &Value) -> i64 {
        let Value::List(entries) =
            call("KL_column", std::slice::from_ref(parameter), span()).expect("KL column")
        else {
            panic!("KL_column must return a list")
        };
        let [Value::Tuple(entry)] = entries.as_slice() else {
            panic!("A1 fixture must have one nonzero KL entry")
        };
        let [Value::Integer(raw_row), _, _] = entry.as_slice() else {
            panic!("KL column entry must be (raw row, Param, polynomial)")
        };
        i64::try_from(raw_row).expect("A1 raw row fits i64")
    }

    #[test]
    fn proper_integral_parameter_w_graph_uses_the_subsystem_topology() {
        let datum = fixture_datum("B2", true);
        let inner = call(
            "inner_class",
            &[datum, matrix(2, 2, vec![1, 0, 0, 1])],
            span(),
        )
        .expect("B2 inner class");
        let real = call("real_form", &[inner, int(2)], span()).expect("split B2 form");
        let parameter = sl2r_param(&real, 5, &[1, 1], &[1, 0], 2);

        assert_eq!(
            call("W_graph", std::slice::from_ref(&parameter), span())
                .expect("proper W graph")
                .to_string(),
            "(1,[([],[(2,1)]),([],[(2,1)]),([0],[(0,1),(1,1)])])"
        );
        assert_eq!(
            call("W_cells", std::slice::from_ref(&parameter), span())
                .expect("proper W cells")
                .to_string(),
            "(1,[([0],[([],[])]),([1],[([],[])]),([2],[([0],[])])])"
        );

        let sl2r = sl2r_split_form();
        let nonstandard = sl2r_param(&sl2r, 1, &[-2], &[0], 1);
        for name in ["W_graph", "W_cells"] {
            let error = validate(name, std::slice::from_ref(&nonstandard), span())
                .expect_err("discarded parameter W graph checks standardness");
            assert_eq!(
                error.message,
                "Cannot generate block:\n  \
                 non-standard parameter(x=1,lambda=[-1]/1,nu=[0]/1)\n  \
                 Parameter not standard"
            );
        }
    }

    #[test]
    fn proper_integral_parameter_block_hasse_uses_the_subsystem_topology() {
        let datum = fixture_datum("B2", true);
        let inner = call(
            "inner_class",
            &[datum, matrix(2, 2, vec![1, 0, 0, 1])],
            span(),
        )
        .expect("B2 inner class");
        let real = call("real_form", &[inner, int(2)], span()).expect("split B2 form");
        let parameter = sl2r_param(&real, 5, &[1, 1], &[1, 0], 2);

        assert_eq!(
            call("block_Hasse", std::slice::from_ref(&parameter), span())
                .expect("proper block Hasse")
                .to_string(),
            "([final parameter(x=4,lambda=[2,2]/1,nu=[1,-1]/2),\
             final parameter(x=5,lambda=[2,2]/1,nu=[1,-1]/2),\
             final parameter(x=10,lambda=[1,2]/1,nu=[1,7]/2)],\n\
             | 0, 0, 1 |\n\
             | 0, 0, 1 |\n\
             | 0, 0, 0 |\n)"
        );

        let sl2r = sl2r_split_form();
        let nonstandard = sl2r_param(&sl2r, 1, &[-2], &[0], 1);
        let error = validate("block_Hasse", &[nonstandard], span())
            .expect_err("discarded block_Hasse checks standardness");
        assert_eq!(
            error.message,
            "Cannot generate block:\n  \
             non-standard parameter(x=1,lambda=[-1]/1,nu=[0]/1)\n  \
             Parameter not standard"
        );
    }

    #[test]
    fn proper_integral_partial_block_uses_the_start_downset_after_full_cache_hits() {
        let make_parameter = || {
            let datum = fixture_datum("B2", true);
            let inner = call(
                "inner_class",
                &[datum, matrix(2, 2, vec![1, 0, 0, 1])],
                span(),
            )
            .expect("B2 inner class");
            let real = call("real_form", &[inner, int(2)], span()).expect("split B2 form");
            sl2r_param(&real, 5, &[1, 1], &[1, 0], 2)
        };
        let expected = "[final parameter(x=5,lambda=[2,2]/1,nu=[1,-1]/2)]";

        let cold = make_parameter();
        assert_eq!(
            call("partial_block", std::slice::from_ref(&cold), span())
                .expect("cold proper partial block")
                .to_string(),
            expected
        );

        let warm = make_parameter();
        call("block_Hasse", std::slice::from_ref(&warm), span())
            .expect("install the full common block");
        assert_eq!(
            call("partial_block", std::slice::from_ref(&warm), span())
                .expect("warm proper partial block")
                .to_string(),
            expected
        );

        let sl2r = sl2r_split_form();
        let nonstandard = sl2r_param(&sl2r, 1, &[-2], &[0], 1);
        let error = validate("partial_block", &[nonstandard], span())
            .expect_err("discarded partial_block checks standardness");
        assert_eq!(
            error.message,
            "Cannot generate block:\n  \
             non-standard parameter(x=1,lambda=[-1]/1,nu=[0]/1)\n  \
             Parameter not standard"
        );
    }

    #[test]
    fn proper_integral_kl_sum_at_s_uses_the_parameter_partial_block() {
        let datum = fixture_datum("B2", true);
        let inner = call(
            "inner_class",
            &[datum, matrix(2, 2, vec![1, 0, 0, 1])],
            span(),
        )
        .expect("B2 inner class");
        let real = call("real_form", &[inner, int(2)], span()).expect("split B2 form");
        let parameter = sl2r_param(&real, 5, &[1, 1], &[1, 0], 2);

        assert_eq!(
            call("KL_sum_at_s", std::slice::from_ref(&parameter), span())
                .expect("proper KL sum")
                .to_string(),
            "\n1*parameter(x=5,lambda=[2,2]/1,nu=[1,-1]/2) [12]"
        );
        validate("KL_sum_at_s", std::slice::from_ref(&parameter), span())
            .expect("final proper parameter passes KL sum gates");

        // KL_sum_at_s_to_height (atlas-types.w:8358-8368): the height bound
        // filters the reconstructed final terms; a negative bound means no
        // filter, reproducing the unbounded column sum.
        let height_sum = |bound: i64| {
            call(
                "KL_sum_at_s_to_height",
                &[parameter.clone(), int(bound)],
                span(),
            )
            .expect("proper KL sum to height")
            .to_string()
        };
        assert_eq!(height_sum(0), "Empty sum of standard modules");
        assert_eq!(
            height_sum(-1),
            "\n1*parameter(x=5,lambda=[2,2]/1,nu=[1,-1]/2) [12]"
        );
        assert_eq!(
            height_sum(12),
            "\n1*parameter(x=5,lambda=[2,2]/1,nu=[1,-1]/2) [12]"
        );
        validate(
            "KL_sum_at_s_to_height",
            &[parameter.clone(), int(0)],
            span(),
        )
        .expect("final proper parameter passes KL sum to-height gates");
    }

    #[test]
    fn representation_table_callers_share_only_lookup_materializations() {
        let make_parameter = || {
            let real = sl2r_split_form();
            sl2r_param(&real, 1, &[0], &[1], 2)
        };

        let standalone = make_parameter();
        assert_eq!(kl_column_single_raw_row(&standalone), 0);

        let after_kl_block = make_parameter();
        call("KL_block", std::slice::from_ref(&after_kl_block), span()).expect("full KL block");
        assert_eq!(kl_column_single_raw_row(&after_kl_block), 1);

        let after_no_value_kl_block = make_parameter();
        validate(
            "KL_block",
            std::slice::from_ref(&after_no_value_kl_block),
            span(),
        )
        .expect("no-value KL block validates without materializing");
        assert_eq!(kl_column_single_raw_row(&after_no_value_kl_block), 0);

        let after_common_print = make_parameter();
        print_text(
            "print_common_block",
            std::slice::from_ref(&after_common_print),
            span(),
        )
        .expect("common block print");
        assert_eq!(kl_column_single_raw_row(&after_common_print), 1);

        for direct_printer in ["print_block", "print_partial_block"] {
            let parameter = make_parameter();
            print_text(direct_printer, std::slice::from_ref(&parameter), span())
                .expect("direct block print");
            assert_eq!(
                kl_column_single_raw_row(&parameter),
                0,
                "{direct_printer} must not install a representation block"
            );
        }
    }

    #[test]
    fn kl_column_candidate_rows_stop_at_the_query_row() {
        assert_eq!(
            kl_column_candidate_rows(3).collect::<Vec<_>>(),
            vec![0, 1, 2, 3]
        );
    }

    // The strings below are the verified HPC oracle output of
    // tests/fixtures/domain/print_common_block.atlas.
    #[test]
    fn print_common_block_reports_the_sl2r_blocks() {
        let real = sl2r_split_form();
        // param(KGB(rf,2),[1],[1]/2): non-integral gamma, rank-0 integral
        // subsystem, so the common block is the seed element alone.
        let p = sl2r_param(&real, 2, &[1], &[1], 2);
        assert_eq!(
            print_text("print_common_block", std::slice::from_ref(&p), span()).expect("print"),
            "Parameter defines element 0 of the following common block,\nas transformed by <>:\n0:  0  []   *(x=2,gamma-lambda=  [1]/2)  1^e\n"
        );
        // The print_block(Param) overload (atlas-types.w:6653-6666) prints
        // the same block under the plain header.
        assert_eq!(
            print_text("print_block", std::slice::from_ref(&p), span()).expect("print"),
            "Parameter defines element 0 of the following block:\n0:  0  []   *(x=2,gamma-lambda=  [1]/2)  1^e\n"
        );
        // param(KGB(rf,0),[1],[0]/1): gamma = lambda = [2], no singular
        // coroot, so every element survives.
        let q = sl2r_param(&real, 0, &[1], &[0], 1);
        assert_eq!(
            print_text("print_common_block", std::slice::from_ref(&q), span()).expect("print"),
            "Parameter defines element 0 of the following common block,\nas transformed by <>:\n0:  0  [i1]  1   (2,*)  *(x=0,gamma-lambda=  [0]/1)  e\n1:  0  [i1]  0   (2,*)  *(x=1,gamma-lambda=  [0]/1)  e\n2:  1  [r1]  2   (0,1)  *(x=2,gamma-lambda=  [0]/1)  1^e\n"
        );
        // param(KGB(rf,2),[1],[0]/1): gamma = nu = [0], the simple coroot
        // is singular, so the r1 descent at z=2 drops the star.
        let q3 = sl2r_param(&real, 2, &[1], &[0], 1);
        assert_eq!(
            print_text("print_common_block", std::slice::from_ref(&q3), span()).expect("print"),
            "Parameter defines element 2 of the following common block,\nas transformed by <>:\n0:  0  [i1]  1   (2,*)  *(x=0,gamma-lambda=  [0]/1)  e\n1:  0  [i1]  0   (2,*)  *(x=1,gamma-lambda=  [0]/1)  e\n2:  1  [r1]  2   (0,1)   (x=2,gamma-lambda=  [0]/1)  1^e\n"
        );
    }

    #[test]
    fn print_common_block_star_column_uses_each_parameters_own_gamma() {
        let lie_type = call("Lie_type", &[Value::String("B2".into())], span()).expect("Lie type");
        let datum = call(
            "simply_connected",
            &[lie_type, Value::Boolean(true)],
            span(),
        )
        .expect("root datum");
        let matrix = Value::Matrix(
            Matrix::from_columns(2, 2, vec![1, 0, 0, 1]).expect("identity involution"),
        );
        let inner = call("inner_class", &[datum, matrix], span()).expect("inner class");
        let real = call(
            "real_form",
            &[inner, Value::Integer(BigInt::from(2))],
            span(),
        )
        .expect("split real form");
        let b2_param = |x: i64| {
            let element = call(
                "KGB",
                &[real.clone(), Value::Integer(BigInt::from(x))],
                span(),
            )
            .expect("KGB element");
            call(
                "param",
                &[
                    element,
                    Value::Vector(Vec32(vec![1, 1])),
                    Value::RatVector(RatVec::new(vec![0, 0], 1).expect("ratvec")),
                ],
                span(),
            )
            .expect("param")
        };
        // pb = param(KGB(rfb,0),[1,1],[0,0]/1): gamma = lambda = [2,2], no
        // singular coroot, so all twelve rows keep the star.
        let pb = print_text("print_common_block", &[b2_param(0)], span()).expect("print");
        assert!(pb.starts_with(
            "Parameter defines element 0 of the following common block,\nas transformed by <>:\n"
        ));
        let pb_rows: Vec<&str> = pb.lines().skip(2).collect();
        assert_eq!(pb_rows.len(), 12);
        assert!(pb_rows.iter().all(|line| line.contains("*(x=")));
        // pb2 = param(KGB(rfb,5),[1,1],[0,0]/1): gamma = (1+theta_5)[1,1]
        // is singular on coroot 0, so the rows with a descent at generator
        // 0 (4, 5, 7, 10) lose the star while the table is unchanged.
        let pb2 = print_text("print_common_block", &[b2_param(5)], span()).expect("print");
        assert!(pb2.starts_with(
            "Parameter defines element 5 of the following common block,\nas transformed by <>:\n"
        ));
        let pb2_rows: Vec<&str> = pb2.lines().skip(2).collect();
        assert_eq!(pb2_rows.len(), 12);
        for (z, (left, right)) in pb_rows.iter().zip(&pb2_rows).enumerate() {
            let starred = ![4, 5, 7, 10].contains(&z);
            assert_eq!(right.contains("*(x="), starred, "row {z}");
            // Dropping the star is the only difference between the tables.
            assert_eq!(right.replacen(" (x=", "*(x=", 1), *left, "row {z}");
        }
    }

    // The strings below are the verified HPC oracle output of
    // tests/fixtures/domain/print_partial_block.atlas; the two partial
    // printers emit byte-identical text per parameter.
    #[test]
    fn print_partial_block_reports_the_sl2r_intervals() {
        let real = sl2r_split_form();
        // param(KGB(rf,2),[1],[1]/2): half-integral gamma, rank-0 integral
        // subsystem, so the interval is the seed element alone.
        let p = sl2r_param(&real, 2, &[1], &[1], 2);
        let p_rows = "0:  0  []   *(x=2,gamma-lambda=  [1]/2)  1^e\n";
        assert_eq!(
            print_text("print_partial_block", std::slice::from_ref(&p), span()).expect("print"),
            p_rows
        );
        assert_eq!(
            print_text(
                "print_partial_common_block",
                std::slice::from_ref(&p),
                span()
            )
            .expect("print"),
            p_rows
        );
        // param(KGB(rf,2),[1],[0]/1): gamma = nu = [0], the simple coroot
        // is singular, so the r1 descent at row 2 drops the star.
        let q3 = sl2r_param(&real, 2, &[1], &[0], 1);
        let q3_rows = "0:  0  [i1]  1   (2,*)  *(x=0,gamma-lambda=  [0]/1)  e\n1:  0  [i1]  0   (2,*)  *(x=1,gamma-lambda=  [0]/1)  e\n2:  1  [r1]  2   (0,1)   (x=2,gamma-lambda=  [0]/1)  1^e\n";
        assert_eq!(
            print_text("print_partial_block", std::slice::from_ref(&q3), span()).expect("print"),
            q3_rows
        );
        assert_eq!(
            print_text(
                "print_partial_common_block",
                std::slice::from_ref(&q3),
                span()
            )
            .expect("print"),
            q3_rows
        );
    }

    #[test]
    fn print_partial_block_reports_the_b2_intervals() {
        let lie_type = call("Lie_type", &[Value::String("B2".into())], span()).expect("Lie type");
        let datum = call(
            "simply_connected",
            &[lie_type, Value::Boolean(true)],
            span(),
        )
        .expect("root datum");
        let matrix = Value::Matrix(
            Matrix::from_columns(2, 2, vec![1, 0, 0, 1]).expect("identity involution"),
        );
        let inner = call("inner_class", &[datum, matrix], span()).expect("inner class");
        let real = call(
            "real_form",
            &[inner, Value::Integer(BigInt::from(2))],
            span(),
        )
        .expect("split real form");
        let b2_param = |x: i64| {
            let element = call(
                "KGB",
                &[real.clone(), Value::Integer(BigInt::from(x))],
                span(),
            )
            .expect("KGB element");
            call(
                "param",
                &[
                    element,
                    Value::Vector(Vec32(vec![1, 1])),
                    Value::RatVector(RatVec::new(vec![0, 0], 1).expect("ratvec")),
                ],
                span(),
            )
            .expect("param")
        };
        // pb = param(KGB(rfb,0),[1,1],[0,0]/1): the most compact element
        // has only imaginary ascents, so the interval is the singleton.
        let pb_rows = "0:  0  [i1,i1]  *  *   (*,*)  (*,*)  *(x=0,gamma-lambda=   [0,0]/1)  e\n";
        // pb2 = param(KGB(rfb,5),[1,1],[0,0]/1): the 3-row interval
        // x=2,3,5; gamma = (1+theta_5)[1,1] is singular on coroot 0, so
        // the r1 descent at row 2 drops the star.
        let pb2_rows = "0:  0  [i1,i1]  1  *   (2,*)  (*,*)  *(x=2,gamma-lambda=   [0,0]/1)  e\n1:  0  [i1,ic]  0  1   (2,*)  (*,*)  *(x=3,gamma-lambda=   [0,0]/1)  e\n2:  1  [r1,C+]  2  *   (0,1)  (*,*)   (x=5,gamma-lambda=   [0,0]/1)  1^e\n";
        for name in ["print_partial_block", "print_partial_common_block"] {
            assert_eq!(
                print_text(name, &[b2_param(0)], span()).expect("print"),
                pb_rows
            );
            assert_eq!(
                print_text(name, &[b2_param(5)], span()).expect("print"),
                pb2_rows
            );
        }
    }

    #[test]
    fn outer_twist_applies_and_validates_like_the_oracle() {
        let real = sl2r_split_form();
        let element = |index: i64| {
            call(
                "KGB",
                &[real.clone(), Value::Integer(BigInt::from(index))],
                span(),
            )
            .expect("KGB element")
        };
        let matrix = |entry: i32| {
            Value::Matrix(Matrix::from_columns(1, 1, vec![entry]).expect("1x1 matrix"))
        };

        // The identity outer twist fixes the split element #2.
        let twisted = call("twist", &[element(2), matrix(1)], span()).expect("outer twist");
        assert_eq!(twisted, element(2));

        // [[-1]] is a root-datum involution, but not one of the BASED datum.
        let error = call("twist", &[element(0), matrix(-1)], span())
            .expect_err("unbased involution is rejected");
        assert_eq!(error.kind, ErrorKind::Runtime);
        assert_eq!(error.message, "Root datum involution is not distinguished");
        let error = validate("twist", &[element(0), matrix(-1)], span())
            .expect_err("the no-value path validates identically");
        assert_eq!(error.message, "Root datum involution is not distinguished");

        // A non-involutive matrix fails the earlier check.
        let error = call("twist", &[element(0), matrix(2)], span())
            .expect_err("non-involution is rejected");
        assert_eq!(error.message, "Given transformation is not an involution");

        // A wrongly sized matrix fails first of all.
        let wide = Value::Matrix(Matrix::from_columns(1, 2, vec![1, 0]).expect("1x2 matrix"));
        let error =
            call("twist", &[element(0), wide], span()).expect_err("wrong-size matrix is rejected");
        assert_eq!(
            error.message,
            "Involution should be a 1x1 matrix; received a 1x2 matrix"
        );
    }

    #[test]
    fn shift_flip_matches_the_oracle_gates_and_sign() {
        // A2 simply connected, split inner class (the diagram flip as
        // distinguished involution), split real form: the setup of
        // tests/fixtures/domain/shift_flip{,_rejected}.atlas.
        let datum = fixture_datum("A2", true);
        let flip = matrix(2, 2, vec![0, 1, 1, 0]);
        let split_class =
            call("inner_class", &[datum.clone(), flip.clone()], span()).expect("split inner class");
        let form = call("real_form", &[split_class, int(0)], span()).expect("split real form");
        let element = call("KGB", &[form, int(0)], span()).expect("KGB element");
        let ratvec =
            |entries: &[i64]| Value::RatVector(RatVec::new(entries.to_vec(), 1).expect("ratvec"));
        let param = |nu: &[i64]| {
            call(
                "param",
                &[
                    element.clone(),
                    Value::Vector(Vec32(vec![0, 0])),
                    ratvec(nu),
                ],
                span(),
            )
            .expect("param")
        };

        // Zero shift (new_gamma == p.gamma()): the extension is the
        // default one, hence not flipped. NOTE: nonzero shifts at a
        // compact Cartan (theta_x = +1) trip the debug_assert in
        // shifted_default_extension (ext_param.rs:754-759), whose
        // precondition the upstream wrapper does NOT guarantee — the
        // oracle builds with -DNDEBUG (upstream Makefile:86-87) and
        // returns false for e.g. shift_flip(pa,[[1]],[0]/1). That crate
        // fix is outside this slice.
        let q = param(&[0, 0]);
        assert_eq!(
            call(
                "shift_flip",
                &[q.clone(), flip.clone(), ratvec(&[1, 1])],
                span()
            ),
            Ok(Value::Boolean(false))
        );

        // (1-delta)*[1,0] = [1,-1] != 0: the rational-weight check fires,
        // identically on the no-value validation path.
        for result in [
            call(
                "shift_flip",
                &[q.clone(), flip.clone(), ratvec(&[1, 0])],
                span(),
            ),
            validate(
                "shift_flip",
                &[q.clone(), flip.clone(), ratvec(&[1, 0])],
                span(),
            )
            .map(|()| Value::Tuple(Vec::new())),
        ] {
            assert_eq!(
                result.expect_err("gamma is not delta-fixed").message,
                "Involution does not fix rational weight"
            );
        }

        // p2's own gamma normalises to [1,-1]/2, which the flip does not
        // fix: the infinitesimal-character check fires once the
        // rational-weight check has passed.
        let p2 = param(&[1, 0]);
        let error = call("shift_flip", &[p2, flip.clone(), ratvec(&[0, 0])], span())
            .expect_err("the parameter's gamma is not delta-fixed");
        assert_eq!(
            error.message,
            "Involution does not fix infinitesimal character"
        );

        // test_compatible fires first of all: non-involutive and
        // wrong-size matrices are rejected before either gamma check.
        let non_involution = matrix(2, 2, vec![1, 0, 1, 1]);
        let error = call(
            "shift_flip",
            &[q.clone(), non_involution, ratvec(&[0, 0])],
            span(),
        )
        .expect_err("non-involution is rejected");
        assert_eq!(error.message, "Given transformation is not an involution");
        let three_by_three = matrix(3, 3, vec![1, 0, 0, 0, 1, 0, 0, 0, 1]);
        let error = call(
            "shift_flip",
            &[q.clone(), three_by_three, ratvec(&[0, 0])],
            span(),
        )
        .expect_err("wrong-size matrix is rejected");
        assert_eq!(
            error.message,
            "Involution should be a 2x2 matrix; received a 3x3 matrix"
        );

        // The flip is ACCEPTED by test_compatible on the compact
        // (identity) inner class (oracle probe, shift_flip_rejected.
        // meta.json): the call evaluates rather than erroring. Zero
        // shift, so the default extension is not flipped.
        let compact_class = call(
            "inner_class",
            &[datum, matrix(2, 2, vec![1, 0, 0, 1])],
            span(),
        )
        .expect("compact inner class");
        let compact_form =
            call("real_form", &[compact_class, int(1)], span()).expect("quasisplit real form");
        let compact_element = call("KGB", &[compact_form, int(0)], span()).expect("KGB element");
        let qc = call(
            "param",
            &[
                compact_element,
                Value::Vector(Vec32(vec![0, 0])),
                ratvec(&[0, 0]),
            ],
            span(),
        )
        .expect("param");
        assert_eq!(
            call("shift_flip", &[qc, flip, ratvec(&[1, 1])], span()),
            Ok(Value::Boolean(false))
        );

        // A real shift at the SPLIT Cartan (theta_x = -1, so the crate's
        // debug_assert holds): shift_flip(pa2,[[1]],[0]/1) on SL(2,R),
        // fixture line 13, oracle answer false.
        let sl2r = sl2r_split_form();
        let pa2 = sl2r_param(&sl2r, 2, &[0], &[1], 1);
        let zero = Value::RatVector(RatVec::new(vec![0], 1).expect("ratvec"));
        let identity = matrix(1, 1, vec![1]);
        assert_eq!(
            call("shift_flip", &[pa2, identity, zero], span()),
            Ok(Value::Boolean(false))
        );
    }

    // The strings below are the verified HPC oracle output of
    // tests/fixtures/domain/ext_finalise.atlas (job 3538977).
    #[test]
    fn ext_finalise_trio_matches_the_oracle_anchors() {
        // A1, compact inner class, split form SL(2,R): the x=0 and x=2
        // accepted cases (fixture lines 10-16).
        let sl2r = sl2r_split_form();
        let identity = matrix(1, 1, vec![1]);
        let pa = sl2r_param(&sl2r, 0, &[0], &[0], 1);
        let expected_pa = "final parameter(x=0,lambda=[1]/1,nu=[0]/1)";
        assert_eq!(
            call(
                "scale_extended",
                &[pa.clone(), identity.clone(), rat(2, 1)],
                span()
            )
            .expect("scale_extended")
            .to_string(),
            format!("({expected_pa},false)")
        );
        assert_eq!(
            call(
                "K_type_pol_extended",
                &[pa.clone(), identity.clone()],
                span()
            )
            .expect("K_type_pol_extended")
            .to_string(),
            "\n1* K_type(x=0, lambda=[1]/1) [1]"
        );
        assert_eq!(
            call("finalize_extended", &[pa, identity], span())
                .expect("finalize_extended")
                .to_string(),
            "\n1*parameter(x=0,lambda=[1]/1,nu=[0]/1) [1]"
        );
        let pa2 = sl2r_param(&sl2r, 2, &[0], &[1], 1);
        assert_eq!(
            call(
                "scale_extended",
                &[pa2, matrix(1, 1, vec![1]), rat(3, 2)],
                span()
            )
            .expect("scale_extended")
            .to_string(),
            "(final parameter(x=2,lambda=[1]/1,nu=[3]/2),false)"
        );

        // A2, both inner classes, x=0 (fixture lines 26-38): same output
        // for the compact and the split inner class.
        let datum = fixture_datum("A2", true);
        let flip = matrix(2, 2, vec![0, 1, 1, 0]);
        let identity2 = matrix(2, 2, vec![1, 0, 0, 1]);
        for (twist, form_number) in [(identity2.clone(), 1), (flip.clone(), 0)] {
            let class = call("inner_class", &[datum.clone(), twist], span()).expect("inner class");
            let form = call("real_form", &[class, int(form_number)], span()).expect("real form");
            let element = call("KGB", &[form, int(0)], span()).expect("KGB element");
            let q = call(
                "param",
                &[
                    element,
                    Value::Vector(Vec32(vec![0, 0])),
                    Value::RatVector(RatVec::new(vec![0, 0], 1).expect("ratvec")),
                ],
                span(),
            )
            .expect("param");
            let delta = matrix(2, 2, vec![1, 0, 0, 1]);
            let delta = if form_number == 0 {
                flip.clone()
            } else {
                delta
            };
            assert_eq!(
                call(
                    "scale_extended",
                    &[q.clone(), delta.clone(), rat(2, 1)],
                    span()
                )
                .expect("scale_extended")
                .to_string(),
                "(final parameter(x=0,lambda=[1,1]/1,nu=[0,0]/1),false)"
            );
            assert_eq!(
                call("K_type_pol_extended", &[q.clone(), delta.clone()], span())
                    .expect("K_type_pol_extended")
                    .to_string(),
                "\n1* K_type(x=0, lambda=[1,1]/1) [4]"
            );
            assert_eq!(
                call("finalize_extended", &[q, delta], span())
                    .expect("finalize_extended")
                    .to_string(),
                "\n1*parameter(x=0,lambda=[1,1]/1,nu=[0,0]/1) [4]"
            );
        }

        // SL(3,R), x=3 (fixture lines 42-43): the flip case — both polys
        // carry the Split(0,1) "1s*" coefficient.
        let split_class =
            call("inner_class", &[datum.clone(), flip.clone()], span()).expect("split inner class");
        let split_form = call("real_form", &[split_class, int(0)], span()).expect("split form");
        let element3 = call("KGB", &[split_form, int(3)], span()).expect("KGB element");
        let p = call(
            "param",
            &[
                element3,
                Value::Vector(Vec32(vec![1, 1])),
                Value::RatVector(RatVec::new(vec![0, 0], 1).expect("ratvec")),
            ],
            span(),
        )
        .expect("param");
        assert_eq!(
            call("finalize_extended", &[p.clone(), flip.clone()], span())
                .expect("finalize_extended")
                .to_string(),
            "\n1s*parameter(x=0,lambda=[0,0]/1,nu=[0,0]/1) [0]"
        );
        assert_eq!(
            call("K_type_pol_extended", &[p, flip], span())
                .expect("K_type_pol_extended")
                .to_string(),
            "\n1s* K_type(x=0, lambda=[0,0]/1) [0]"
        );

        // B2, split form so(3,2), x=8 (fixture lines 53-54): the two-term
        // case; ParamPol orders x descending, KTypePol x ascending.
        let datum_b2 = fixture_datum("B2", true);
        let class_b2 =
            call("inner_class", &[datum_b2, identity2.clone()], span()).expect("inner class");
        let form_b2 = call("real_form", &[class_b2, int(2)], span()).expect("real form");
        let element8 = call("KGB", &[form_b2, int(8)], span()).expect("KGB element");
        let pb = call(
            "param",
            &[
                element8,
                Value::Vector(Vec32(vec![0, 0])),
                Value::RatVector(RatVec::new(vec![0, 0], 1).expect("ratvec")),
            ],
            span(),
        )
        .expect("param");
        assert_eq!(
            call(
                "finalize_extended",
                &[pb.clone(), identity2.clone()],
                span()
            )
            .expect("finalize_extended")
            .to_string(),
            "\n1*parameter(x=1,lambda=[0,1]/1,nu=[0,0]/1) [3]\
             \n1*parameter(x=0,lambda=[0,1]/1,nu=[0,0]/1) [3]"
        );
        assert_eq!(
            call("K_type_pol_extended", &[pb, identity2], span())
                .expect("K_type_pol_extended")
                .to_string(),
            "\n1* K_type(x=0, lambda=[0,1]/1) [3]\
             \n1* K_type(x=1, lambda=[0,1]/1) [3]"
        );
    }

    // The messages below are the verified HPC oracle diagnostics of
    // tests/fixtures/domain/ext_finalise_rejected.atlas, with the
    // two-space continuation indents the comparison pipeline strips.
    #[test]
    fn ext_finalise_rejections_match_the_oracle_gates() {
        let datum = fixture_datum("A2", true);
        let flip = matrix(2, 2, vec![0, 1, 1, 0]);
        let split_class =
            call("inner_class", &[datum, flip.clone()], span()).expect("split inner class");
        let split_form = call("real_form", &[split_class, int(0)], span()).expect("split form");
        let param = |x: i64, nu: &[i64]| {
            let element = call("KGB", &[split_form.clone(), int(x)], span()).expect("KGB element");
            call(
                "param",
                &[
                    element,
                    Value::Vector(Vec32(vec![1, 1])),
                    Value::RatVector(RatVec::new(nu.to_vec(), 1).expect("ratvec")),
                ],
                span(),
            )
            .expect("param")
        };
        let p = param(3, &[0, 0]);
        let p2 = {
            let element = call("KGB", &[split_form.clone(), int(0)], span()).expect("KGB element");
            call(
                "param",
                &[
                    element,
                    Value::Vector(Vec32(vec![0, 0])),
                    Value::RatVector(RatVec::new(vec![1, 0], 1).expect("ratvec")),
                ],
                span(),
            )
            .expect("param")
        };

        // test_final fires first of all (rejected line 11).
        let error = call(
            "scale_extended",
            &[p.clone(), flip.clone(), rat(2, 1)],
            span(),
        )
        .expect_err("non-final parameter is rejected");
        assert_eq!(
            error.message,
            "Cannot scale extended parameter:\n  \
             non-final parameter(x=3,lambda=[2,2]/1,nu=[0,0]/1)\n  \
             Parameter is not semifinal"
        );
        for result in [
            call("KL_column", std::slice::from_ref(&p), span()),
            validate("KL_column", std::slice::from_ref(&p), span())
                .map(|()| Value::Tuple(Vec::new())),
        ] {
            assert_eq!(
                result
                    .expect_err("KL_column requires a final parameter")
                    .message,
                "Cannot compute Kazhdan-Lusztig column:\n  \
                 non-final parameter(x=3,lambda=[2,2]/1,nu=[0,0]/1)\n  \
                 Parameter is not semifinal"
            );
        }

        // The factor check precedes test_compatible and is_fixed
        // (rejected line 12), identically on the no-value path.
        for result in [
            call(
                "scale_extended",
                &[p2.clone(), flip.clone(), rat(0, 1)],
                span(),
            ),
            validate(
                "scale_extended",
                &[p2.clone(), flip.clone(), rat(0, 1)],
                span(),
            )
            .map(|()| Value::Tuple(Vec::new())),
        ] {
            assert_eq!(
                result.expect_err("a zero factor is rejected").message,
                "Factor in scale_extended must be positive"
            );
        }

        // p2's gamma [1,-1]/2 is not fixed by the distinguished delta
        // (rejected lines 13-14).
        let error = call(
            "scale_extended",
            &[p2.clone(), flip.clone(), rat(2, 1)],
            span(),
        )
        .expect_err("not delta-fixed");
        assert_eq!(
            error.message,
            "Parameter to be scaled not fixed by given involution"
        );
        let error = call("K_type_pol_extended", &[p2.clone(), flip.clone()], span())
            .expect_err("not delta-fixed");
        assert_eq!(error.message, "Parameter not fixed by given involution");

        // The x=1 parameter IS delta-fixed, but its Cartan involution does
        // not commute with delta (rejected line 15).
        let non_commuting = {
            let element = call("KGB", &[split_form.clone(), int(1)], span()).expect("KGB element");
            call(
                "param",
                &[
                    element,
                    Value::Vector(Vec32(vec![0, 0])),
                    Value::RatVector(RatVec::new(vec![0, 0], 1).expect("ratvec")),
                ],
                span(),
            )
            .expect("param")
        };
        let error = call("finalize_extended", &[non_commuting, flip.clone()], span())
            .expect_err("non-commuting involution is rejected");
        assert_eq!(
            error.message,
            "Involution of parameter does not commute with delta"
        );

        // test_compatible rejections (rejected lines 16-17).
        let error = call(
            "scale_extended",
            &[p2.clone(), matrix(2, 2, vec![1, 0, 1, 1]), rat(2, 1)],
            span(),
        )
        .expect_err("non-involution is rejected");
        assert_eq!(error.message, "Given transformation is not an involution");
        let error = call(
            "scale_extended",
            &[p2, matrix(3, 3, vec![1, 0, 0, 0, 1, 0, 0, 0, 1]), rat(2, 1)],
            span(),
        )
        .expect_err("wrong-size matrix is rejected");
        assert_eq!(
            error.message,
            "Involution should be a 2x2 matrix; received a 3x3 matrix"
        );

        // A1, the non-standard n3 (rejected lines 26-28): test_standard
        // fires for finalize/K_type_pol (the latter keeps the upstream "|"
        // typo), while scale_extended's test_final reports "not dominant".
        let sl2r = sl2r_split_form();
        let n3 = sl2r_param(&sl2r, 1, &[-2], &[0], 1);
        let identity = matrix(1, 1, vec![1]);
        let error = call("finalize_extended", &[n3.clone(), identity.clone()], span())
            .expect_err("non-standard parameter is rejected");
        assert_eq!(
            error.message,
            "Cannot finalize extended parameter:\n  \
             non-standard parameter(x=1,lambda=[-1]/1,nu=[0]/1)\n  \
             Parameter not standard"
        );
        let error = call(
            "K_type_pol_extended",
            &[n3.clone(), identity.clone()],
            span(),
        )
        .expect_err("non-standard parameter is rejected");
        assert_eq!(
            error.message,
            "Parameter in K_type_pol_extended| must be standard:\n  \
             non-standard parameter(x=1,lambda=[-1]/1,nu=[0]/1)\n  \
             Parameter not standard"
        );
        let error = call("scale_extended", &[n3, identity, rat(2, 1)], span())
            .expect_err("non-dominant parameter is rejected");
        assert_eq!(
            error.message,
            "Cannot scale extended parameter:\n  \
             non-standard parameter(x=1,lambda=[-1]/1,nu=[0]/1)\n  \
             Parameter is not dominant"
        );
    }

    // The strings below are the verified HPC oracle output of
    // tests/fixtures/domain/twisted_family.atlas (job 3536421) and
    // tests/fixtures/domain/block_deform.atlas (job 3536583).
    fn su21_quasisplit_form() -> Value {
        let datum = fixture_datum("A2", true);
        let inner = call(
            "inner_class",
            &[datum, matrix(2, 2, vec![1, 0, 0, 1])],
            span(),
        )
        .expect("compact inner class");
        call("real_form", &[inner, int(1)], span()).expect("quasisplit su(2,1)")
    }

    fn su21_param(real: &Value, x: i64, lambda: &[i32], nu: &[i64], nu_denominator: u64) -> Value {
        let element = call("KGB", &[real.clone(), int(x)], span()).expect("KGB element");
        call(
            "param",
            &[
                element,
                Value::Vector(Vec32(lambda.to_vec())),
                Value::RatVector(RatVec::new(nu.to_vec(), nu_denominator).expect("ratvec")),
            ],
            span(),
        )
        .expect("param")
    }

    #[test]
    fn twisted_family_matches_the_oracle_anchors() {
        let sl2r = sl2r_split_form();
        // p = param(KGB(rf,2),[0],[1]/2): non-integral gamma with a
        // rank-0 integral subsystem — the singleton common block.
        let p = sl2r_param(&sl2r, 2, &[0], &[1], 2);
        assert_eq!(
            call("twisted_deform", std::slice::from_ref(&p), span())
                .expect("twisted_deform(p)")
                .to_string(),
            "Empty sum of standard modules"
        );
        assert_eq!(
            call("twisted_KL_sum_at_s", std::slice::from_ref(&p), span())
                .expect("twisted_KL_sum_at_s(p)")
                .to_string(),
            "\n1*parameter(x=2,lambda=[1]/1,nu=[1]/2) [0]"
        );
        // q = param(KGB(rf,0),[1],[0]/1): the discrete series, integral.
        let q = sl2r_param(&sl2r, 0, &[1], &[0], 1);
        assert_eq!(
            call("twisted_deform", std::slice::from_ref(&q), span())
                .expect("twisted_deform(q)")
                .to_string(),
            "Empty sum of standard modules"
        );
        assert_eq!(
            call("twisted_KL_sum_at_s", std::slice::from_ref(&q), span())
                .expect("twisted_KL_sum_at_s(q)")
                .to_string(),
            "\n1*parameter(x=0,lambda=[2]/1,nu=[0]/1) [2]"
        );
        assert_eq!(
            call("twisted_full_deform", std::slice::from_ref(&q), span())
                .expect("twisted_full_deform(q)")
                .to_string(),
            "\n1* K_type(x=0, lambda=[2]/1) [2]"
        );
        // q2 = param(KGB(rfb,0),[0,0],[0,0]/1) in su(2,1).
        let su21 = su21_quasisplit_form();
        let q2 = su21_param(&su21, 0, &[0, 0], &[0, 0], 1);
        assert_eq!(
            call("twisted_deform", std::slice::from_ref(&q2), span())
                .expect("twisted_deform(q2)")
                .to_string(),
            "Empty sum of standard modules"
        );
        assert_eq!(
            call("twisted_full_deform", std::slice::from_ref(&q2), span())
                .expect("twisted_full_deform(q2)")
                .to_string(),
            "\n1* K_type(x=0, lambda=[1,1]/1) [4]"
        );
        // The two twisted_KL_sum_at_s overloads print identically.
        let identity = matrix(2, 2, vec![1, 0, 0, 1]);
        for arguments in [vec![q2.clone()], vec![q2.clone(), identity]] {
            assert_eq!(
                call("twisted_KL_sum_at_s", &arguments, span())
                    .expect("twisted_KL_sum_at_s(q2)")
                    .to_string(),
                "\n1*parameter(x=0,lambda=[1,1]/1,nu=[0,0]/1) [4]"
            );
        }
        // The no-value path runs the same gates without computing.
        for name in [
            "twisted_deform",
            "twisted_full_deform",
            "twisted_KL_sum_at_s",
        ] {
            validate(name, std::slice::from_ref(&q), span())
                .unwrap_or_else(|error| panic!("{name} validates: {error:?}"));
        }
    }

    #[test]
    fn a1_alcove_center_shrinks_thirds_to_the_half_center() {
        // Oracle job 3546215, deform_alcove_shrink.atlas: the center of the
        // A1 alcove containing nu=[1]/3 is nu=[1]/2, while x/lambda stay put.
        let sl2r = sl2r_split_form();
        let p = sl2r_param(&sl2r, 2, &[0], &[1], 3);
        assert_eq!(
            call("alcove_center", &[p], span())
                .expect("alcove_center(p)")
                .to_string(),
            "final parameter(x=2,lambda=[1]/1,nu=[1]/2)"
        );
    }

    #[test]
    fn deform_alcove_shrink_matches_the_oracle_fixture() {
        // Exact value events after the declarations in the verified positive
        // fixture from oracle job 3546215.
        let sl2r = sl2r_split_form();
        let p = sl2r_param(&sl2r, 2, &[0], &[1], 3);
        assert_eq!(p.to_string(), "final parameter(x=2,lambda=[1]/1,nu=[1]/3)");
        assert_eq!(
            call("alcove_center", std::slice::from_ref(&p), span())
                .expect("alcove_center(p)")
                .to_string(),
            "final parameter(x=2,lambda=[1]/1,nu=[1]/2)"
        );
        let expected = "\n1* K_type(x=2, lambda=[1]/1) [0]";
        assert_eq!(
            call("full_deform", std::slice::from_ref(&p), span())
                .expect("full_deform(p)")
                .to_string(),
            expected
        );
        assert_eq!(
            call("twisted_full_deform", &[p], span())
                .expect("twisted_full_deform(p)")
                .to_string(),
            expected
        );
    }

    #[test]
    fn timed_full_deform_uses_the_real_result_cache() {
        let sl2r = sl2r_split_form();
        let p = sl2r_param(&sl2r, 0, &[1], &[0], 1);
        let zero = int(0);

        validate("full_deform", &[p.clone(), zero.clone()], span())
            .expect("discarded timed call validates");
        let too_large = Value::Integer(BigInt::from(i64::from(i32::MAX) + 1));
        let error = validate("full_deform", &[p.clone(), too_large], span())
            .expect_err("timer uses upstream signed-int narrowing");
        assert_eq!(error.message, "Integer value to big for conversion");
        assert_eq!(
            call("full_deform", &[p.clone(), zero.clone()], span())
                .expect("uncached zero-millisecond call")
                .to_string(),
            "().timed_out"
        );
        assert_eq!(
            call("full_deform", &[p.clone(), int(-1)], span())
                .expect("negative timer")
                .to_string(),
            "().timed_out"
        );

        let expected = "\n1* K_type(x=0, lambda=[2]/1) [2]";
        assert_eq!(
            call("full_deform", std::slice::from_ref(&p), span())
                .expect("unary full deformation")
                .to_string(),
            expected
        );
        assert_eq!(
            call("full_deform", &[p, zero], span())
                .expect("cached zero-millisecond call")
                .to_string(),
            format!("{expected}.done")
        );

        let fresh = sl2r_split_form();
        let fresh_p = sl2r_param(&fresh, 0, &[1], &[0], 1);
        assert_eq!(
            call("full_deform", &[fresh_p, int(1_000)], span())
                .expect("positive timer")
                .to_string(),
            format!("{expected}.done")
        );
    }

    #[test]
    fn timed_twisted_full_deform_uses_the_real_result_cache() {
        let sl2r = sl2r_split_form();
        let p = sl2r_param(&sl2r, 0, &[1], &[0], 1);
        let zero = int(0);

        validate("twisted_full_deform", &[p.clone(), zero.clone()], span())
            .expect("discarded timed twisted call validates");
        assert_eq!(
            call("twisted_full_deform", &[p.clone(), zero.clone()], span(),)
                .expect("uncached zero-millisecond twisted call")
                .to_string(),
            "().timed_out"
        );
        let too_large = Value::Integer(BigInt::from(i64::from(i32::MAX) + 1));
        let error = validate("twisted_full_deform", &[p.clone(), too_large], span())
            .expect_err("twisted timer uses upstream signed-int narrowing");
        assert_eq!(error.message, "Integer value to big for conversion");

        let expected = "\n1* K_type(x=0, lambda=[2]/1) [2]";
        assert_eq!(
            call("twisted_full_deform", std::slice::from_ref(&p), span())
                .expect("unary twisted full deformation")
                .to_string(),
            expected
        );
        assert_eq!(
            call("twisted_full_deform", &[p, zero], span())
                .expect("cached zero-millisecond twisted call")
                .to_string(),
            format!("{expected}.done")
        );

        let p2 = sl2r_param(&sl2r, 2, &[0], &[1], 2);
        assert_eq!(
            call("twisted_full_deform", &[p2, int(-1)], span())
                .expect("negative twisted timer")
                .to_string(),
            "().timed_out"
        );
    }

    #[test]
    fn deform_alcove_shrink_rejected_matches_the_oracle_fixture() {
        // The shrink preprocessing must remain behind the wrapper's standard
        // gate; the verified rejected fixture never reaches the center call.
        let sl2r = sl2r_split_form();
        let p = sl2r_param(&sl2r, 1, &[-2], &[0], 1);
        let error = call("twisted_full_deform", &[p], span())
            .expect_err("non-standard parameter is rejected before deformation");
        assert_eq!(
            error.message,
            "Cannot compute full twisted deformation:\n  \
             non-standard parameter(x=1,lambda=[-1]/1,nu=[0]/1)\n  \
             Parameter not standard"
        );
    }

    #[test]
    fn block_deform_matches_the_oracle_height_boundary() {
        // p = param(KGB(rf,3),[0,0],[1,1]/1) in su(2,1); d = deform(p)
        // holds the two height-4 terms with (1-1s) coefficients.
        let su21 = su21_quasisplit_form();
        let p = su21_param(&su21, 3, &[0, 0], &[1, 1], 1);
        let d = call("deform", std::slice::from_ref(&p), span()).expect("deform(p)");
        let d_text = "\n(1-1s)*parameter(x=2,lambda=[1,1]/1,nu=[0,0]/1) [4]\n(1-1s)*parameter(x=0,lambda=[1,1]/1,nu=[0,0]/1) [4]";
        assert_eq!(d.to_string(), d_text);
        // Bounds 0 and 3 keep both height-4 terms in the accumulator;
        // 4, 5, and the negative bound's maximal level move them.
        for bound in [0, 3] {
            let result = call("block_deform", &[p.clone(), d.clone(), int(bound)], span())
                .expect("block_deform");
            assert_eq!(
                result.to_string(),
                format!("(Empty sum of standard modules,{d_text})"),
                "bound {bound}"
            );
        }
        for bound in [4, 5, -1] {
            let result = call("block_deform", &[p.clone(), d.clone(), int(bound)], span())
                .expect("block_deform");
            assert_eq!(
                result.to_string(),
                format!("({d_text},Empty sum of standard modules)"),
                "bound {bound}"
            );
        }
        // The accumulator is immutable: reusing d reproduces the split.
        let again = call("block_deform", &[p, d, int(4)], span()).expect("block_deform again");
        assert_eq!(
            again.to_string(),
            format!("({d_text},Empty sum of standard modules)")
        );
    }

    fn fixture_datum(lie_type: &str, prefers_coroots: bool) -> Value {
        let lie_type =
            call("Lie_type", &[Value::String(lie_type.into())], span()).expect("Lie type");
        call(
            "simply_connected",
            &[lie_type, Value::Boolean(prefers_coroots)],
            span(),
        )
        .expect("root datum")
    }

    fn int(value: i64) -> Value {
        Value::Integer(BigInt::from(value))
    }

    fn rat(numerator: i64, denominator: i64) -> Value {
        Value::Rational(BigRational::from_signeds(numerator, denominator))
    }

    fn matrix(rows: usize, columns: usize, column_major: Vec<i32>) -> Value {
        Value::Matrix(
            Matrix::from_columns(rows, columns, column_major).expect("consistent matrix shape"),
        )
    }

    #[test]
    fn involution_classifier_matches_the_frozen_edge_anchors() {
        let cases = [
            (matrix(0, 0, Vec::new()), (0, 0, 0)),
            (matrix(1, 1, vec![1]), (1, 0, 0)),
            (matrix(1, 1, vec![-1]), (0, 0, 1)),
            (matrix(2, 2, vec![1, 1, 0, -1]), (0, 1, 0)),
            (matrix(2, 2, vec![1, 2, 0, -1]), (1, 0, 1)),
        ];
        for (involution, expected) in cases {
            assert_eq!(
                call("classify_involution", &[involution], span()),
                Ok(Value::Tuple(vec![
                    int(expected.0),
                    int(expected.1),
                    int(expected.2),
                ]))
            );
        }
    }

    #[test]
    fn involution_decomposition_matches_the_frozen_a2_anchor() {
        let datum = fixture_datum("A2", true);
        let opposition = matrix(2, 2, vec![0, -1, -1, 0]);
        let decomposition = call(
            "twisted_involution",
            &[datum.clone(), opposition.clone()],
            span(),
        )
        .expect("A2 opposition decomposes");
        assert_eq!(
            decomposition.to_string(),
            "(<0.1.0>,Complex reductive group of type A2, with involution defining\n\
             inner class of type 'c', with 2 real forms and 1 dual real form)"
        );

        let Value::Tuple(parts) = decomposition else {
            panic!("decomposition is a pair")
        };
        assert_eq!(
            call(
                "distinguished_involution",
                std::slice::from_ref(&parts[1]),
                span(),
            )
            .expect("distinguished involution")
            .to_string(),
            "\n| 1, 0 |\n| 0, 1 |\n"
        );

        assert_eq!(
            call(
                "twisted_involution",
                &[datum, matrix(2, 2, vec![1, 0, 0, 1])],
                span(),
            )
            .expect("identity decomposes")
            .to_string(),
            "(<>,Complex reductive group of type A2, with involution defining\n\
             inner class of type 'c', with 2 real forms and 1 dual real form)"
        );
    }

    #[test]
    fn involution_decomposition_rejections_and_no_value_validation_are_exact() {
        let nonsquare = matrix(1, 2, vec![1, 0]);
        let error = call(
            "classify_involution",
            std::slice::from_ref(&nonsquare),
            span(),
        )
        .expect_err("classifier rejects a nonsquare matrix");
        assert_eq!(
            error.message,
            "Involution should be a 1x1 matrix; received a 1x2 matrix"
        );
        let zero_by_one = matrix(0, 1, Vec::new());
        let error = call(
            "classify_involution",
            std::slice::from_ref(&zero_by_one),
            span(),
        )
        .expect_err("classifier retains a zero-row matrix's column count");
        assert_eq!(
            error.message,
            "Involution should be a 0x0 matrix; received a 0x1 matrix"
        );

        let non_involution = matrix(2, 2, vec![2, 0, 0, 2]);
        for result in [
            call(
                "classify_involution",
                std::slice::from_ref(&non_involution),
                span(),
            ),
            validate(
                "classify_involution",
                std::slice::from_ref(&non_involution),
                span(),
            )
            .map(|()| Value::Tuple(Vec::new())),
        ] {
            assert_eq!(
                result.expect_err("M squared is not the identity").message,
                "Given transformation is not an involution"
            );
        }

        let b2 = fixture_datum("B2", true);
        let foreign = matrix(2, 2, vec![1, 2, 0, -1]);
        for result in [
            call("twisted_involution", &[b2.clone(), foreign.clone()], span()),
            validate("twisted_involution", &[b2, foreign], span())
                .map(|()| Value::Tuple(Vec::new())),
        ] {
            assert_eq!(
                result
                    .expect_err("matrix does not preserve B2 roots")
                    .message,
                "Matrix maps simple root 0 to non-root"
            );
        }
    }

    #[test]
    fn root_and_coroot_queries_follow_the_oracle_presentation_order() {
        let a2 = fixture_datum("A2", true);
        assert_eq!(
            call("nr_of_posroots", std::slice::from_ref(&a2), span()),
            Ok(Value::Integer(BigInt::from(3)))
        );
        assert_eq!(
            call("root", &[a2.clone(), int(0)], span())
                .expect("root")
                .to_string(),
            "[  2, -1 ]"
        );
        assert_eq!(
            call("coroot", &[a2.clone(), int(0)], span())
                .expect("coroot")
                .to_string(),
            "[ 1, 0 ]"
        );
        assert_eq!(
            call("root", &[a2.clone(), int(1)], span())
                .expect("root")
                .to_string(),
            "[ -1,  2 ]"
        );
        assert_eq!(
            call("coroot", &[a2.clone(), int(1)], span())
                .expect("coroot")
                .to_string(),
            "[ 0, 1 ]"
        );
        assert_eq!(
            call("root", &[a2.clone(), int(2)], span())
                .expect("root")
                .to_string(),
            "[ 1, 1 ]"
        );
        assert_eq!(
            call("coroot", &[a2, int(2)], span())
                .expect("coroot")
                .to_string(),
            "[ 1, 1 ]"
        );

        let b2 = fixture_datum("B2", true);
        assert_eq!(
            call("nr_of_posroots", std::slice::from_ref(&b2), span()),
            Ok(Value::Integer(BigInt::from(4)))
        );
        assert_eq!(
            call("root", &[b2.clone(), int(0)], span())
                .expect("root")
                .to_string(),
            "[  2, -2 ]"
        );
        assert_eq!(
            call("coroot", &[b2.clone(), int(0)], span())
                .expect("coroot")
                .to_string(),
            "[ 1, 0 ]"
        );
        assert_eq!(
            call("root", &[b2.clone(), int(3)], span())
                .expect("root")
                .to_string(),
            "[ 1, 0 ]"
        );
        assert_eq!(
            call("coroot", &[b2.clone(), int(3)], span())
                .expect("coroot")
                .to_string(),
            "[ 2, 1 ]"
        );
        // Negative indices negate the positive root at -1 - i (upstream
        // `internal_root_index` with the palindromic root numbering).
        assert_eq!(
            call("root", &[b2.clone(), int(-1)], span())
                .expect("root")
                .to_string(),
            "[ -2,  2 ]"
        );
        assert_eq!(
            call("coroot", &[b2.clone(), int(-4)], span())
                .expect("coroot")
                .to_string(),
            "[ -2, -1 ]"
        );
        assert_eq!(
            call("rank", std::slice::from_ref(&b2), span()),
            Ok(Value::Integer(BigInt::from(2)))
        );
    }

    #[test]
    fn root_index_out_of_range_is_the_oracle_runtime_error() {
        let b2 = fixture_datum("B2", true);
        let error = call("root", &[b2.clone(), int(4)], span())
            .expect_err("one past the last positive root");
        assert_eq!(error.kind, ErrorKind::Runtime);
        assert_eq!(error.message, "Illegal root index 4");
        let error =
            call("coroot", &[b2, int(-5)], span()).expect_err("one past the first negative coroot");
        assert_eq!(error.kind, ErrorKind::Runtime);
        assert_eq!(error.message, "Illegal coroot index -5");
    }

    #[test]
    fn length_flags_follow_the_simply_laced_all_short_convention() {
        let a2 = fixture_datum("A2", true);
        assert_eq!(
            call("is_long_root", &[a2.clone(), int(0)], span()),
            Ok(Value::Boolean(false))
        );
        assert_eq!(
            call("is_long_root", &[a2, int(2)], span()),
            Ok(Value::Boolean(false))
        );
        let b2 = fixture_datum("B2", true);
        assert_eq!(
            call("is_long_root", &[b2.clone(), int(0)], span()),
            Ok(Value::Boolean(true))
        );
        assert_eq!(
            call("is_long_coroot", &[b2.clone(), int(0)], span()),
            Ok(Value::Boolean(false))
        );
        assert_eq!(
            call("is_long_root", &[b2.clone(), int(3)], span()),
            Ok(Value::Boolean(false))
        );
        assert_eq!(
            call("is_long_coroot", &[b2, int(3)], span()),
            Ok(Value::Boolean(true))
        );
    }

    #[test]
    fn coroot_preference_reorders_multi_laced_roots_like_the_oracle_swap() {
        // Without the coroot preference the generation runs on the
        // untransposed Cartan matrix, so the height order dominates and
        // α1+α2 precedes α1+2α2.
        let b2 = fixture_datum("B2", false);
        assert_eq!(
            call("root", &[b2.clone(), int(2)], span())
                .expect("root")
                .to_string(),
            "[ 1, 0 ]"
        );
        assert_eq!(
            call("root", &[b2.clone(), int(3)], span())
                .expect("root")
                .to_string(),
            "[ 0, 2 ]"
        );
    }

    #[test]
    fn posroots_and_poscoroots_render_the_oracle_column_order() {
        // Frozen dual_order probe anchors (HPC job 3502700): the same
        // generation order as the root/coroot queries, laid out by columns.
        let b2 = fixture_datum("B2", false);
        assert_eq!(
            call("posroots", std::slice::from_ref(&b2), span())
                .expect("posroots")
                .to_string(),
            "\n|  2, -1, 1, 0 |\n| -2,  2, 0, 2 |\n"
        );
        assert_eq!(
            call("poscoroots", std::slice::from_ref(&b2), span())
                .expect("poscoroots")
                .to_string(),
            "\n| 1, 0, 2, 1 |\n| 0, 1, 1, 1 |\n"
        );
        // The coroot preference generates on the transposed Cartan matrix
        // and swaps the tables back, reordering the non-simple entries.
        let b2_coroot = fixture_datum("B2", true);
        assert_eq!(
            call("posroots", std::slice::from_ref(&b2_coroot), span())
                .expect("posroots")
                .to_string(),
            "\n|  2, -1, 0, 1 |\n| -2,  2, 2, 0 |\n"
        );
        assert_eq!(
            call("poscoroots", std::slice::from_ref(&b2_coroot), span())
                .expect("poscoroots")
                .to_string(),
            "\n| 1, 0, 1, 2 |\n| 0, 1, 1, 1 |\n"
        );
    }

    #[test]
    fn dual_root_datum_dualizes_type_isogeny_and_coroot_preference() {
        let b2 = fixture_datum("B2", false);
        let dual = call("dual", std::slice::from_ref(&b2), span()).expect("dual");
        assert_eq!(dual.to_string(), "adjoint root datum of Lie type 'C2'");
        assert_eq!(
            call("prefers_coroots", std::slice::from_ref(&dual), span()),
            Ok(Value::Boolean(true))
        );
        // The dual's positive roots are the original's positive coroots.
        assert_eq!(
            call("posroots", std::slice::from_ref(&dual), span())
                .expect("posroots")
                .to_string(),
            call("poscoroots", std::slice::from_ref(&b2), span())
                .expect("poscoroots")
                .to_string()
        );
        // The double dual reproduces the original datum.
        let c2 = fixture_datum("C2", true);
        let double = call(
            "dual",
            std::slice::from_ref(
                &call("dual", std::slice::from_ref(&c2.clone()), span()).expect("dual"),
            ),
            span(),
        )
        .expect("double dual");
        assert_eq!(double.to_string(), c2.to_string());
        assert_eq!(
            call("prefers_coroots", std::slice::from_ref(&double), span()),
            Ok(Value::Boolean(true))
        );
    }

    #[test]
    fn dual_inner_class_carries_the_dual_datum() {
        // Frozen dual_order probe anchors (HPC job 3502700): dual(ic)
        // displays the dual type, and root_datum(di) is the dual datum with
        // the switched coroot preference.
        let b2 = fixture_datum("B2", false);
        let identity = matrix(2, 2, vec![1, 0, 0, 1]);
        let inner = call("inner_class", &[b2, identity], span()).expect("inner class");
        assert_eq!(
            inner.to_string(),
            "Complex reductive group of type B2, with involution defining\n\
             inner class of type 'c', with 3 real forms and 3 dual real forms"
        );
        let dual_inner = call("dual", std::slice::from_ref(&inner), span()).expect("dual");
        assert_eq!(
            dual_inner.to_string(),
            "Complex reductive group of type C2, with involution defining\n\
             inner class of type 'c', with 3 real forms and 3 dual real forms"
        );
        let dual_datum = call("root_datum", std::slice::from_ref(&dual_inner), span())
            .expect("root_datum of dual inner class");
        assert_eq!(
            dual_datum.to_string(),
            "adjoint root datum of Lie type 'C2'"
        );
        assert_eq!(
            call("prefers_coroots", std::slice::from_ref(&dual_datum), span()),
            Ok(Value::Boolean(true))
        );
    }

    #[test]
    fn g2_generation_covers_triple_bonds_and_both_length_classes() {
        let g2 = fixture_datum("G2", true);
        assert_eq!(
            call("nr_of_posroots", std::slice::from_ref(&g2), span()),
            Ok(Value::Integer(BigInt::from(6)))
        );
        // Order from the transposed-Cartan generation, swapped back.
        let expected_roots = [
            "[  2, -1 ]",
            "[ -3,  2 ]",
            "[  3, -1 ]",
            "[ 0, 1 ]",
            "[ -1,  1 ]",
            "[ 1, 0 ]",
        ];
        let long_roots = [false, true, true, true, false, false];
        let long_coroots = [true, false, false, false, true, true];
        for (index, ((expected_root, &long_root), &long_coroot)) in expected_roots
            .iter()
            .zip(&long_roots)
            .zip(&long_coroots)
            .enumerate()
        {
            let index = index as i64;
            assert_eq!(
                call("root", &[g2.clone(), int(index)], span())
                    .expect("root")
                    .to_string(),
                *expected_root
            );
            assert_eq!(
                call("is_long_root", &[g2.clone(), int(index)], span()),
                Ok(Value::Boolean(long_root)),
                "is_long_root at {index}"
            );
            assert_eq!(
                call("is_long_coroot", &[g2.clone(), int(index)], span()),
                Ok(Value::Boolean(long_coroot)),
                "is_long_coroot at {index}"
            );
        }
    }

    fn row(entries: &[i64]) -> Value {
        Value::List(entries.iter().map(|&entry| int(entry)).collect())
    }

    #[test]
    fn weyl_elt_surface_matches_the_frozen_a2_b2_anchors() {
        let a2 = fixture_datum("A2", true);
        let w = call("W_elt", &[a2.clone(), row(&[0, 1, 0])], span()).expect("W_elt");
        assert_eq!(w.to_string(), "<0.1.0>");
        // The braid-equivalent word builds the same group element.
        let v = call("W_elt", &[a2.clone(), row(&[1, 0, 1])], span()).expect("W_elt");
        assert_eq!(v.to_string(), "<0.1.0>");
        assert_eq!(w, v, "group equality is braid-aware");
        assert_eq!(
            call("word", std::slice::from_ref(&w), span())
                .expect("word")
                .to_string(),
            "[0,1,0]"
        );
        assert_eq!(
            call("length", std::slice::from_ref(&w), span()),
            Ok(Value::Integer(BigInt::from(3)))
        );
        // w*v = identity: empty word, length zero, unary relations.
        let product = call("*", &[w.clone(), v.clone()], span()).expect("product");
        assert_eq!(product.to_string(), "<>");
        assert_eq!(
            call("word", std::slice::from_ref(&product), span())
                .expect("word")
                .to_string(),
            "[]"
        );
        assert_eq!(
            call("length", std::slice::from_ref(&product), span()),
            Ok(Value::Integer(BigInt::from(0)))
        );
        assert_eq!(
            call("=", std::slice::from_ref(&product), span()),
            Ok(Value::Boolean(true))
        );
        assert_eq!(
            call("!=", std::slice::from_ref(&w), span()),
            Ok(Value::Boolean(true))
        );
        // The longest element of A2 is an involution.
        assert_eq!(
            call(
                "word",
                &[call("/", std::slice::from_ref(&w), span()).expect("inverse")],
                span()
            )
            .expect("word")
            .to_string(),
            "[0,1,0]"
        );
        // w # 1 reduces to s1 s0.
        let shifted = call("#", &[w.clone(), int(1)], span()).expect("generator product");
        assert_eq!(shifted.to_string(), "<1.0>");

        let b2 = fixture_datum("B2", true);
        let a = call("W_elt", &[b2.clone(), row(&[0, 1, 0, 1])], span()).expect("W_elt");
        assert_eq!(a.to_string(), "<1.0.1.0>");
        let b = call("W_elt", &[b2.clone(), row(&[1, 0, 1, 0])], span()).expect("W_elt");
        assert_eq!(a, b);
        assert_eq!(
            call("length", std::slice::from_ref(&a), span()),
            Ok(Value::Integer(BigInt::from(4)))
        );
        assert_eq!(
            call("root_datum", std::slice::from_ref(&a), span())
                .expect("root_datum")
                .to_string(),
            "simply connected root datum of Lie type 'B2'"
        );
    }

    #[test]
    fn weyl_elt_rejections_echo_the_oracle_messages() {
        let b2 = fixture_datum("B2", true);
        let error = call("W_elt", &[b2.clone(), row(&[5])], span())
            .expect_err("entry past the semisimple rank");
        assert_eq!(error.kind, ErrorKind::Runtime);
        assert_eq!(error.message, "Illegal Weyl word entry 5 (should be <2)");
        let error =
            call("W_elt", &[b2.clone(), row(&[0, -1])], span()).expect_err("negative entry");
        assert_eq!(error.message, "Negative integer where unsigned is required");

        let w = call("W_elt", &[b2.clone(), row(&[0, 1])], span()).expect("W_elt");
        let error = call("#", &[w.clone(), int(2)], span()).expect_err("generator past the rank");
        assert_eq!(
            error.message,
            "Generator 2 out of range for Weyl group (should be <2)"
        );
        let error = call("#", &[w.clone(), int(-1)], span()).expect_err("negative generator");
        assert_eq!(
            error.message,
            "Generator -1 out of range for Weyl group (should be <2)"
        );
        // Products across different root data mismatch like upstream.
        let other =
            call("W_elt", &[fixture_datum("B2", true), row(&[0, 1])], span()).expect("W_elt");
        assert_eq!(w, other, "structurally equal data share the group");
        let a2 = call("W_elt", &[fixture_datum("A2", true), row(&[0, 1])], span()).expect("W_elt");
        let error = call("*", &[w, a2], span()).expect_err("mismatched data");
        assert_eq!(error.message, "Weyl group mismatch");
    }

    #[test]
    fn p1_weyl_products_match_both_upstream_operand_orders() {
        let a2 = fixture_datum("A2", true);
        let w = call("W_elt", &[a2, row(&[0])], span()).expect("W_elt");

        for (arguments, expected) in [
            (vec![int(1), w.clone()], "[1,0]"),
            (vec![w.clone(), row(&[1])], "[0,1]"),
            (vec![row(&[1]), w.clone()], "[1,0]"),
            (vec![w.clone(), row(&[0, 1])], "[1]"),
            (vec![row(&[0, 1]), w.clone()], "[0,1,0]"),
            (vec![w.clone(), row(&[1, 0])], "[0,1,0]"),
            (vec![row(&[1, 0]), w.clone()], "[1]"),
        ] {
            let operator = if arguments
                .iter()
                .any(|value| matches!(value, Value::List(_)))
            {
                "##"
            } else {
                "#"
            };
            let product = call(operator, &arguments, span()).expect("Weyl product");
            assert_eq!(
                call("word", &[product], span())
                    .expect("canonical word")
                    .to_string(),
                expected,
            );
        }

        let error = call("##", &[w.clone(), row(&[0, 2])], span())
            .expect_err("right word entry past the rank");
        assert_eq!(error.message, "Illegal Weyl word entry 2 (should be <2)");
        let error = call("##", &[row(&[2]), w], span()).expect_err("left word entry past the rank");
        assert_eq!(error.message, "Illegal Weyl word entry 2 (should be <2)");

        let w = call("W_elt", &[fixture_datum("A2", true), row(&[0])], span()).expect("W_elt");
        for arguments in [vec![int(2), w.clone()], vec![int(-1), w.clone()]] {
            let error = call("#", &arguments, span()).expect_err("invalid left generator");
            assert_eq!(
                error.message,
                format!(
                    "Generator {} out of range for Weyl group (should be <2)",
                    arguments[0]
                )
            );
            let validation = validate("#", &arguments, span())
                .expect_err("no-value validates the left generator");
            assert_eq!(validation.message, error.message);
        }
        let invalid_word = [w, row(&[2])];
        let error = validate("##", &invalid_word, span())
            .expect_err("no-value validates every Weyl-word entry");
        assert_eq!(error.message, "Illegal Weyl word entry 2 (should be <2)");

        let huge = Value::Integer(
            "999999999999999999999999"
                .parse::<BigInt>()
                .expect("huge integer"),
        );
        for arguments in [
            vec![huge.clone(), invalid_word[0].clone()],
            vec![invalid_word[0].clone(), huge.clone()],
        ] {
            let error = call("#", &arguments, span()).expect_err("generator must narrow to int");
            assert_eq!(error.message, "Integer value to big for conversion");
            let validation = validate("#", &arguments, span())
                .expect_err("no-value performs the same int narrowing");
            assert_eq!(validation.message, error.message);
        }
        let huge_word = Value::List(vec![huge]);
        let error = call("##", &[invalid_word[0].clone(), huge_word.clone()], span())
            .expect_err("word entries must narrow to unsigned long");
        assert_eq!(error.message, "Integer value to big for conversion");
        let validation = validate("##", &[invalid_word[0].clone(), huge_word], span())
            .expect_err("no-value performs the same unsigned-long narrowing");
        assert_eq!(validation.message, error.message);

        // bigint.cpp:142-146 accepts exactly signed 32-bit values. Positive
        // 2^31 needs a second sign digit and therefore does not wrap.
        for text in ["2147483648", "4294967295", "4294967296", "-2147483649"] {
            let boundary = Value::Integer(text.parse::<BigInt>().expect("integer boundary"));
            for arguments in [
                vec![boundary.clone(), invalid_word[0].clone()],
                vec![invalid_word[0].clone(), boundary.clone()],
            ] {
                let error = call("#", &arguments, span()).expect_err("outside signed int range");
                assert_eq!(error.message, "Integer value to big for conversion");
                assert_eq!(
                    validate("#", &arguments, span())
                        .expect_err("no-value performs signed int narrowing")
                        .message,
                    error.message,
                );
            }
        }
        let minimum = Value::Integer("-2147483648".parse::<BigInt>().expect("minimum signed int"));
        let error = call("#", &[minimum, invalid_word[0].clone()], span())
            .expect_err("minimum signed int is a range error, not conversion overflow");
        assert_eq!(
            error.message,
            "Generator -2147483648 out of range for Weyl group (should be <2)"
        );
    }

    #[test]
    fn p1_polynomial_terms_use_the_upstream_real_form_owner_identity() {
        let datum = fixture_datum("A1", true);
        let identity = matrix(1, 1, vec![1]);
        let inner = call("inner_class", &[datum, identity.clone()], span()).expect("inner class");
        let custom = || {
            call(
                "real_form",
                &[
                    inner.clone(),
                    identity.clone(),
                    Value::RatVector(RatVec::new(vec![3], 2).expect("ratvec")),
                ],
                span(),
            )
            .expect("custom real form")
        };
        let left = custom();
        let right = custom();
        assert_eq!(left, right, "real-form value equality remains structural");

        // Equivalence wrappers compare the RealReductiveGroup value rather
        // than its shared owner (atlas-types.w:5326, 6341). Both the ordinary
        // call and its no-value validation must therefore accept these forms.
        let make_ktype = |rf: Value| {
            call(
                "K_type",
                &[
                    call("KGB", &[rf, int(0)], span()).expect("KGB element"),
                    Value::Vector(Vec32(vec![0])),
                ],
                span(),
            )
            .expect("K-type")
        };
        let equivalent_ktypes = [make_ktype(left.clone()), make_ktype(right.clone())];
        validate("equivalent", &equivalent_ktypes, span())
            .expect("no-value KType equivalence is structural");
        assert_eq!(
            call("equivalent", &equivalent_ktypes, span()).expect("KType equivalence"),
            Value::Boolean(true),
        );

        let make_param = |rf: Value| {
            call(
                "param",
                &[
                    call("KGB", &[rf, int(0)], span()).expect("KGB element"),
                    Value::Vector(Vec32(vec![0])),
                    Value::RatVector(RatVec::new(vec![0], 1).expect("ratvec")),
                ],
                span(),
            )
            .expect("parameter")
        };
        let equivalent_params = [make_param(left.clone()), make_param(right.clone())];
        validate("equivalent", &equivalent_params, span())
            .expect("no-value Param equivalence is structural");
        assert_eq!(
            call("equivalent", &equivalent_params, span()).expect("Param equivalence"),
            Value::Boolean(true),
        );

        let zero_param =
            call("null_module", std::slice::from_ref(&left), span()).expect("zero ParamPol");
        let parameter = call(
            "param",
            &[
                call("KGB", &[right.clone(), int(0)], span()).expect("KGB element"),
                Value::Vector(Vec32(vec![0])),
                Value::RatVector(RatVec::new(vec![0], 1).expect("ratvec")),
            ],
            span(),
        )
        .expect("parameter");
        let coefficient = Value::Domain(DomainValue::Split(SplitValue::new(1, 0)));
        let tuple_arguments = [
            zero_param.clone(),
            Value::Tuple(vec![coefficient.clone(), parameter.clone()]),
        ];
        for error in [
            validate("+", &tuple_arguments, span()).expect_err("tuple no-value checks owner"),
            call("+", &tuple_arguments, span()).expect_err("tuple checks owner"),
        ] {
            assert_eq!(
                error.message,
                "Real form mismatch when adding a term to a module"
            );
        }
        let error = call(
            "+",
            &[
                zero_param,
                Value::List(vec![Value::Tuple(vec![coefficient.clone(), parameter])]),
            ],
            span(),
        )
        .expect_err("list checks each owner after the no-value gate");
        assert_eq!(
            error.message,
            "Real form mismatch when adding terms to a module"
        );

        let zero_ktype =
            call("null_K_module", std::slice::from_ref(&left), span()).expect("zero KTypePol");
        let ktype = call(
            "K_type",
            &[
                call("KGB", &[right, int(0)], span()).expect("KGB element"),
                Value::Vector(Vec32(vec![0])),
            ],
            span(),
        )
        .expect("K-type");
        let error = call(
            "+",
            &[
                zero_ktype,
                Value::List(vec![Value::Tuple(vec![coefficient, ktype])]),
            ],
            span(),
        )
        .expect_err("KType list checks custom owner identity");
        assert_eq!(
            error.message,
            "Real form mismatch when adding terms to a K_type"
        );

        // Default forms are upstream-memoized, so independent canonical
        // construction shares the same cached context and owner.
        let canonical_left = call("real_form", &[inner.clone(), int(0)], span()).expect("form");
        let canonical_right = call("real_form", &[inner, int(0)], span()).expect("form");
        let zero = call("null_module", std::slice::from_ref(&canonical_left), span())
            .expect("zero module");
        let canonical_param = call(
            "param",
            &[
                call("KGB", &[canonical_right, int(0)], span()).expect("KGB element"),
                Value::Vector(Vec32(vec![0])),
                Value::RatVector(RatVec::new(vec![0], 1).expect("ratvec")),
            ],
            span(),
        )
        .expect("parameter");
        call(
            "+",
            &[
                zero,
                Value::Tuple(vec![
                    Value::Domain(DomainValue::Split(SplitValue::new(1, 0))),
                    canonical_param,
                ]),
            ],
            span(),
        )
        .expect("canonical forms share their logical owner");
    }

    #[test]
    fn canonical_real_forms_share_the_cached_context_and_rep_owner() {
        let datum = fixture_datum("A1", true);
        let identity = matrix(1, 1, vec![1]);
        let inner = call("inner_class", &[datum, identity], span()).expect("inner class");
        let Value::Domain(DomainValue::InnerClass(parent)) = inner else {
            panic!("inner_class must return an InnerClass")
        };

        let first = build_real_form(&parent, 0, span()).expect("first canonical form");
        let second = build_real_form(&parent, 0, span()).expect("cached canonical form");

        assert!(Arc::ptr_eq(&first, &second));
        assert!(Arc::ptr_eq(&first.rep, &second.rep));
        assert!(std::ptr::eq(first.table.as_ref(), first.rep.table()));
        assert!(std::ptr::eq(first.graph.as_ref(), first.rep.graph()));
    }

    #[test]
    fn concurrent_canonical_builders_converge_at_the_second_cache_check() {
        use std::sync::mpsc;
        use std::time::Duration;

        let datum = fixture_datum("A1", true);
        let identity = matrix(1, 1, vec![1]);
        let inner = call("inner_class", &[datum, identity], span()).expect("inner class");
        let Value::Domain(DomainValue::InnerClass(parent)) = inner else {
            panic!("inner_class must return an InnerClass")
        };

        let (reached_tx, reached_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        *parent
            .canonical_build_test_gate
            .lock()
            .expect("test gate lock") = Some(CanonicalBuildTestGate {
            reached: reached_tx,
            release: Arc::new(Mutex::new(release_rx)),
        });

        let builders = (0..2)
            .map(|_| {
                let parent = Arc::clone(&parent);
                std::thread::spawn(move || build_real_form(&parent, 0, span()))
            })
            .collect::<Vec<_>>();

        for _ in 0..2 {
            reached_rx
                .recv_timeout(Duration::from_secs(5))
                .expect("both candidates reach the pre-commit gate");
        }
        release_tx.send(()).expect("release first builder");
        release_tx.send(()).expect("release second builder");

        let mut builders = builders.into_iter();
        let first = builders
            .next()
            .expect("first builder handle")
            .join()
            .expect("first builder does not panic")
            .expect("first canonical form");
        let second = builders
            .next()
            .expect("second builder handle")
            .join()
            .expect("second builder does not panic")
            .expect("second canonical form");
        assert!(Arc::ptr_eq(&first, &second));
        assert!(Arc::ptr_eq(&first.rep, &second.rep));

        let cached = parent.canonical_forms.lock().expect("canonical cache lock")[0]
            .upgrade()
            .expect("winner remains cached while handles are live");
        assert!(Arc::ptr_eq(&first, &cached));
        assert!(Arc::ptr_eq(&first.rep, &cached.rep));
    }

    #[test]
    fn canonical_real_form_cache_is_weak_and_rebuilds_after_last_handle_drops() {
        let datum = fixture_datum("A1", true);
        let identity = matrix(1, 1, vec![1]);
        let inner = call("inner_class", &[datum, identity], span()).expect("inner class");
        let Value::Domain(DomainValue::InnerClass(parent)) = inner else {
            panic!("inner_class must return an InnerClass")
        };

        let first = build_real_form(&parent, 0, span()).expect("first canonical form");
        let context_weak = Arc::downgrade(&first);
        let rep_weak = Arc::downgrade(&first.rep);
        drop(first);
        assert!(context_weak.upgrade().is_none());
        assert!(rep_weak.upgrade().is_none());

        let rebuilt = build_real_form(&parent, 0, span()).expect("rebuilt canonical form");
        assert!(std::ptr::eq(rebuilt.table.as_ref(), rebuilt.rep.table()));
        assert!(std::ptr::eq(rebuilt.graph.as_ref(), rebuilt.rep.graph()));
    }

    #[test]
    fn canonical_real_form_cache_poison_is_a_stable_runtime_diagnostic() {
        let datum = fixture_datum("A1", true);
        let identity = matrix(1, 1, vec![1]);
        let inner = call("inner_class", &[datum, identity], span()).expect("inner class");
        let Value::Domain(DomainValue::InnerClass(parent)) = inner else {
            panic!("inner_class must return an InnerClass")
        };

        let poisoned_parent = Arc::clone(&parent);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
            let _guard = poisoned_parent
                .canonical_forms
                .lock()
                .expect("cache starts healthy");
            panic!("poison canonical cache for the diagnostic contract");
        }));
        assert!(result.is_err());

        let error = build_real_form(&parent, 0, span()).expect_err("poison must be diagnosed");
        assert_eq!(error.message, "canonical real form cache poisoned");
    }

    #[test]
    fn real_form_owner_caches_are_isolated_for_custom_and_distinct_inner_contexts() {
        let datum = fixture_datum("A1", true);
        let identity = matrix(1, 1, vec![1]);
        let first_inner =
            call("inner_class", &[datum.clone(), identity.clone()], span()).expect("inner class");
        let second_inner =
            call("inner_class", &[datum, identity.clone()], span()).expect("inner class");
        let Value::Domain(DomainValue::InnerClass(first_parent)) = first_inner.clone() else {
            panic!("inner_class must return an InnerClass")
        };
        let Value::Domain(DomainValue::InnerClass(second_parent)) = second_inner else {
            panic!("inner_class must return an InnerClass")
        };

        let first_canonical =
            build_real_form(&first_parent, 0, span()).expect("first canonical form");
        let second_canonical =
            build_real_form(&second_parent, 0, span()).expect("second canonical form");
        assert!(!Arc::ptr_eq(&first_canonical, &second_canonical));
        assert!(!Arc::ptr_eq(&first_canonical.rep, &second_canonical.rep));

        let custom = || {
            call(
                "real_form",
                &[
                    first_inner.clone(),
                    identity.clone(),
                    Value::RatVector(RatVec::new(vec![3], 2).expect("ratvec")),
                ],
                span(),
            )
            .expect("custom real form")
        };
        let Value::Domain(DomainValue::RealForm(first_custom)) = custom() else {
            panic!("real_form must return a RealForm")
        };
        let Value::Domain(DomainValue::RealForm(second_custom)) = custom() else {
            panic!("real_form must return a RealForm")
        };
        assert!(same_real_form(&first_custom, &second_custom));
        assert!(!Arc::ptr_eq(&first_custom, &second_custom));
        assert!(!Arc::ptr_eq(&first_custom.rep, &second_custom.rep));
        assert!(!same_real_form_owner(&first_custom, &second_custom));
    }

    #[test]
    fn full_deform_outer_merge_uses_ktype_keys_and_drops_zero_sums() {
        let datum = fixture_datum("A2", true);
        let identity = matrix(2, 2, vec![1, 0, 0, 1]);
        let inner = call("inner_class", &[datum, identity], span()).expect("inner class");
        let form = call("real_form", &[inner, int(1)], span()).expect("compact form");
        let make_ktype = |x, lambda| {
            let value = call(
                "K_type",
                &[
                    call("KGB", &[form.clone(), int(x)], span()).expect("KGB element"),
                    Value::Vector(Vec32(lambda)),
                ],
                span(),
            )
            .expect("K-type");
            let Value::Domain(DomainValue::KType(value)) = value else {
                panic!("K_type must return KType")
            };
            value.ktype
        };
        let first = make_ktype(0, vec![0, 0]);
        let second = make_ktype(1, vec![0, 0]);
        assert_ne!(first, second);

        let mut terms = Vec::new();
        merge_ktype_term(&mut terms, second.clone(), SplitValue::new(1, 1));
        merge_ktype_term(&mut terms, first.clone(), SplitValue::new(1, 1));
        assert_eq!(terms.len(), 2, "equal coefficients do not identify terms");

        merge_ktype_term(&mut terms, first.clone(), SplitValue::new(2, -1));
        let first_coefficient = terms
            .iter()
            .find(|(_, term)| *term == first)
            .map(|(coefficient, _)| *coefficient);
        assert_eq!(first_coefficient, Some(SplitValue::new(3, 0)));

        merge_ktype_term(&mut terms, first.clone(), SplitValue::new(-3, 0));
        assert!(terms.iter().all(|(_, term)| *term != first));
        assert_eq!(terms, vec![(SplitValue::new(1, 1), second)]);

        let parameter = call(
            "param",
            &[
                call("KGB", &[form, int(5)], span()).expect("KGB element"),
                Value::Vector(Vec32(vec![2, 0])),
                Value::RatVector(RatVec::new(vec![0, 0], 1).expect("ratvec")),
            ],
            span(),
        )
        .expect("parameter");
        assert_eq!(
            call("full_deform", &[parameter], span())
                .expect("full deformation")
                .to_string(),
            "\n1* K_type(x=0, lambda=[0,1]/1) [2]\
             \n1* K_type(x=1, lambda=[0,1]/1) [2]"
        );
    }

    #[test]
    fn p1_kgb_cartan_and_polynomial_unary_and_term_list_operations_match_oracle() {
        let a2 = fixture_datum("A2", true);
        let identity = matrix(2, 2, vec![1, 0, 0, 1]);
        let inner = call("inner_class", &[a2, identity], span()).expect("inner class");
        let form = call("real_form", &[inner, int(1)], span()).expect("real form");
        let x = call("KGB", &[form.clone(), int(4)], span()).expect("KGB element");
        assert_eq!(
            call("Cartan_class", std::slice::from_ref(&x), span())
                .expect("KGB Cartan class")
                .to_string(),
            "Cartan class #1, occurring for 1 real form and for 1 dual real form",
        );

        let ktype = call("K_type", &[x, Value::Vector(Vec32(vec![1, 0]))], span()).expect("K-type");
        let zero_k =
            call("null_K_module", std::slice::from_ref(&form), span()).expect("zero K module");
        assert_eq!(
            call("=", std::slice::from_ref(&zero_k), span()),
            Ok(Value::Boolean(true))
        );
        assert_eq!(
            call("!=", std::slice::from_ref(&zero_k), span()),
            Ok(Value::Boolean(false))
        );
        let k_terms = Value::List(vec![
            Value::Tuple(vec![
                Value::Domain(DomainValue::Split(SplitValue::new(1, 0))),
                ktype.clone(),
            ]),
            Value::Tuple(vec![
                Value::Domain(DomainValue::Split(SplitValue::new(2, 0))),
                ktype,
            ]),
        ]);
        let k_pol = call("+", &[zero_k, k_terms], span()).expect("KType term list");
        assert_eq!(
            call("=", std::slice::from_ref(&k_pol), span()),
            Ok(Value::Boolean(false))
        );
        assert_eq!(call("!=", &[k_pol], span()), Ok(Value::Boolean(true)));

        let p_x = call("KGB", &[form.clone(), int(5)], span()).expect("KGB element");
        let parameter = call(
            "param",
            &[
                p_x,
                Value::Vector(Vec32(vec![0, 0])),
                Value::RatVector(RatVec::new(vec![0, 0], 1).expect("ratvec")),
            ],
            span(),
        )
        .expect("parameter");
        let zero_p = call("null_module", &[form], span()).expect("zero module");
        assert_eq!(
            call("=", std::slice::from_ref(&zero_p), span()),
            Ok(Value::Boolean(true))
        );
        assert_eq!(
            call("!=", std::slice::from_ref(&zero_p), span()),
            Ok(Value::Boolean(false))
        );
        let coefficient = Value::Domain(DomainValue::Split(SplitValue::new(1, 0)));
        let singleton = call(
            "+",
            &[
                zero_p.clone(),
                Value::Tuple(vec![coefficient.clone(), parameter.clone()]),
            ],
            span(),
        )
        .expect("Param tuple term");
        assert!(matches!(singleton, Value::Domain(DomainValue::ParamPol(_))));
        let list = Value::List(vec![
            Value::Tuple(vec![coefficient, parameter.clone()]),
            Value::Tuple(vec![
                Value::Domain(DomainValue::Split(SplitValue::new(2, 0))),
                parameter,
            ]),
        ]);
        let p_pol = call("+", &[zero_p, list], span()).expect("Param term list");
        assert_eq!(
            call("=", std::slice::from_ref(&p_pol), span()),
            Ok(Value::Boolean(false))
        );
        assert_eq!(call("!=", &[p_pol], span()), Ok(Value::Boolean(true)));
    }

    fn compact_a2_inner_class() -> Value {
        let matrix =
            Value::Matrix(Matrix::from_columns(2, 2, vec![0, -1, -1, 0]).expect("2x2 matrix"));
        call("inner_class", &[fixture_datum("A2", true), matrix], span()).expect("inner class")
    }

    #[test]
    fn real_form_labels_match_the_frozen_a2_anchors() {
        let inner = compact_a2_inner_class();
        assert_eq!(
            call("occurrence_matrix", std::slice::from_ref(&inner), span())
                .expect("occurrence_matrix")
                .to_string(),
            "\n| 1, 0 |\n| 1, 1 |\n"
        );
        assert_eq!(
            call(
                "dual_occurrence_matrix",
                std::slice::from_ref(&inner),
                span()
            )
            .expect("dual_occurrence_matrix")
            .to_string(),
            "\n| 1, 1 |\n"
        );
        assert_eq!(
            call("block_sizes", std::slice::from_ref(&inner), span())
                .expect("block_sizes")
                .to_string(),
            "\n| 1 |\n| 6 |\n"
        );
        assert_eq!(
            call("block_size", &[inner.clone(), int(0), int(0)], span()),
            Ok(Value::Integer(BigInt::from(1)))
        );
        assert_eq!(
            call("block_size", &[inner.clone(), int(1), int(0)], span()),
            Ok(Value::Integer(BigInt::from(6)))
        );
        let quasisplit = call("real_form", &[inner.clone(), int(1)], span()).expect("real form");
        assert_eq!(
            quasisplit.to_string(),
            "connected quasisplit real group with Lie algebra 'su(2,1)'"
        );
        assert_eq!(
            call("Cartan_order", std::slice::from_ref(&quasisplit), span())
                .expect("Cartan_order")
                .to_string(),
            "\n| 1, 1 |\n| 0, 1 |\n"
        );
    }

    #[test]
    fn block_size_bounds_diagnostics_echo_the_oracle_on_both_levels() {
        let inner = compact_a2_inner_class();
        let error = call("block_size", &[inner.clone(), int(5), int(0)], span())
            .expect_err("real form out of bounds");
        assert_eq!(error.kind, ErrorKind::Runtime);
        assert_eq!(error.message, "Real form number 5 out of bounds");
        let error = validate("block_size", &[inner.clone(), int(5), int(0)], span())
            .expect_err("the no-value path validates identically");
        assert_eq!(error.message, "Real form number 5 out of bounds");
        let error = call("block_size", &[inner.clone(), int(0), int(5)], span())
            .expect_err("dual real form out of bounds");
        assert_eq!(error.message, "Dual real form number 5 out of bounds");
        let error = call("block_size", &[inner.clone(), int(-1), int(0)], span())
            .expect_err("negative form number");
        assert_eq!(error.message, "Negative integer where unsigned is required");
    }

    fn relation_lie_type(text: &str) -> Value {
        call("Lie_type", &[Value::String(text.into())], span()).expect("Lie type")
    }

    fn relation_smith(text: &str) -> Value {
        call("Smith_Cartan", &[relation_lie_type(text)], span()).expect("Smith basis")
    }

    #[test]
    fn relation_lattice_constructors_match_the_frozen_oracle() {
        let a2_smith = relation_smith("A2");
        assert_eq!(a2_smith.to_string(), "(\n|  1, 0 |\n| -2, 1 |\n,[ 1, 3 ])");
        assert_eq!(
            call("filter_units", std::slice::from_ref(&a2_smith), span())
                .expect("filtered Smith basis")
                .to_string(),
            "(\n| 0 |\n| 1 |\n,[ 3 ])"
        );
        assert_eq!(
            relation_smith("A1.B2").to_string(),
            "(\n| 1,  0, 0 |\n| 0,  1, 0 |\n| 0, -2, 1 |\n,[ 2, 1, 2 ])"
        );

        let modulo_matrix =
            Value::Matrix(Matrix::from_columns(2, 2, vec![2, 4, 6, 3]).expect("2x2 matrix"));
        assert_eq!(
            call("ann_mod", &[modulo_matrix, int(2)], span())
                .expect("annihilator modulo two")
                .to_string(),
            "\n| -1,  4 |\n|  0, -2 |\n"
        );

        let replacement =
            Value::Matrix(Matrix::from_columns(2, 1, vec![0, 3]).expect("one replacement column"));
        assert_eq!(
            call("replace_gen", &[a2_smith, replacement], span())
                .expect("replaced generators")
                .to_string(),
            "\n|  1, 0 |\n| -2, 3 |\n"
        );

        for generator in [
            RatVec::new(vec![1], 3).expect("one third"),
            RatVec::new(vec![2], 6).expect("normalised one third"),
        ] {
            assert_eq!(
                call(
                    "quotient_basis",
                    &[
                        relation_lie_type("A2"),
                        Value::List(vec![Value::RatVector(generator)]),
                    ],
                    span(),
                )
                .expect("quotient basis")
                .to_string(),
                "\n|  1, 0 |\n| -2, 3 |\n"
            );
        }
    }

    #[test]
    fn relation_lattice_rejections_match_the_frozen_oracle() {
        let extra_columns = Value::Matrix(
            Matrix::from_columns(2, 2, vec![0, 1, 3, 1]).expect("two replacement columns"),
        );
        let error = call(
            "replace_gen",
            &[relation_smith("A2"), extra_columns],
            span(),
        )
        .expect_err("one non-unit factor accepts one replacement column");
        assert_eq!(error.kind, ErrorKind::Runtime);
        assert_eq!(error.message, "Too many replacement columns");

        let improper = call(
            "quotient_basis",
            &[
                relation_lie_type("A2"),
                Value::List(vec![Value::RatVector(
                    RatVec::new(vec![1], 2).expect("one half"),
                )]),
            ],
            span(),
        )
        .expect_err("one half is not a generator of the order-three factor");
        assert_eq!(
            improper.message,
            "Improper generator entry: 1/2 not a multiple of 1/3"
        );

        let wrong_length = call(
            "quotient_basis",
            &[
                relation_lie_type("A2"),
                Value::List(vec![Value::RatVector(
                    RatVec::new(vec![1, 0], 3).expect("two-entry generator"),
                )]),
            ],
            span(),
        )
        .expect_err("generator length follows the filtered invariant factors");
        assert_eq!(wrong_length.message, "Length mismatch for generator 0: 2:1");
    }

    #[test]
    fn relation_lattice_extended_oracle_elections_are_preserved() {
        assert_eq!(
            relation_smith("T2").to_string(),
            "(\n| 1, 0 |\n| 0, 1 |\n,[ 0, 0 ])"
        );
        assert_eq!(
            relation_smith("D4").to_string(),
            "(\n|  1,  0, 0, 0 |\n| -2,  1, 0, 0 |\n|  1, -2, 1, 0 |\n|  1,  0, 0, 1 |\n,[ 1, 1, 2, 2 ])"
        );

        let cases = [
            (
                Matrix::from_columns(2, 2, vec![0, 0, 0, 0]).expect("zero matrix"),
                "\n| 1, 0 |\n| 0, 1 |\n",
            ),
            (
                Matrix::from_columns(2, 1, vec![1, 0]).expect("two by one"),
                "\n| 2, 0 |\n| 0, 1 |\n",
            ),
            (
                Matrix::from_columns(1, 2, vec![1, 0]).expect("one by two"),
                "\n| 2 |\n",
            ),
        ];
        for (matrix, expected) in cases {
            assert_eq!(
                call("ann_mod", &[Value::Matrix(matrix), int(2)], span())
                    .expect("annihilator")
                    .to_string(),
                expected
            );
        }
        assert_eq!(
            call(
                "ann_mod",
                &[
                    Value::Matrix(Matrix::from_columns(1, 1, vec![1]).expect("one by one")),
                    int(-2),
                ],
                span(),
            )
            .expect("negative modulus follows the oracle")
            .to_string(),
            "\n| -2 |\n"
        );
        assert_eq!(
            call(
                "ann_mod",
                &[
                    Value::Matrix(Matrix::from_columns(1, 1, vec![3]).expect("one by one")),
                    int(-3),
                ],
                span(),
            )
            .expect("negative modulus uses the upstream unsigned gcd")
            .to_string(),
            "\n| -3 |\n"
        );

        let quotient = call(
            "quotient_basis",
            &[
                relation_lie_type("A3"),
                Value::List(vec![
                    Value::RatVector(RatVec::new(vec![1], 2).expect("one half")),
                    Value::RatVector(RatVec::new(vec![1], 4).expect("one quarter")),
                ]),
            ],
            span(),
        )
        .expect("mixed denominator quotient");
        assert_eq!(
            quotient.to_string(),
            "\n|  1,  0, 0 |\n| -2,  1, 0 |\n|  1, -2, 4 |\n"
        );
    }

    #[test]
    fn relation_lattice_extended_diagnostics_and_safe_zero_divergence() {
        let identity = Value::Tuple(vec![
            Value::Matrix(Matrix::from_columns(2, 2, vec![1, 0, 0, 1]).expect("identity")),
            Value::Vector(Vec32(vec![2, 2])),
        ]);
        let one_replacement =
            Value::Matrix(Matrix::from_columns(2, 1, vec![1, 0]).expect("one replacement column"));
        let error = call("replace_gen", &[identity, one_replacement], span())
            .expect_err("two non-unit factors require two columns");
        assert_eq!(error.message, "Not enough replacement columns");

        let singleton =
            Value::Matrix(Matrix::from_columns(1, 1, vec![1]).expect("singleton matrix"));
        let overflow = call(
            "ann_mod",
            &[
                singleton.clone(),
                Value::Integer(BigInt::from(2_147_483_648_i64)),
            ],
            span(),
        )
        .expect_err("the wrapper narrows to Atlas int before domain arithmetic");
        assert_eq!(overflow.message, "Integer value to big for conversion");

        let zero = call("ann_mod", &[singleton, int(0)], span())
            .expect_err("Rust must not reproduce the upstream SIGFPE");
        assert_eq!(zero.message, "ann_mod modulus must be nonzero");

        let over_rank =
            Value::Matrix(Matrix::from_columns(65, 0, Vec::new()).expect("empty 65 by 0 matrix"));
        let error = call("ann_mod", &[over_rank, int(1)], span())
            .expect_err("matrix rank is checked before adapter copying");
        assert_eq!(
            error.message,
            "integer-lattice rank exceeded its limit of 64"
        );

        let generators = Value::List(
            (0..65)
                .map(|_| Value::RatVector(RatVec::new(vec![0], 1).expect("zero generator")))
                .collect(),
        );
        let error = call(
            "quotient_basis",
            &[relation_lie_type("A1"), generators],
            span(),
        )
        .expect_err("generator count is checked before adapter copying");
        assert_eq!(
            error.message,
            "integer-lattice rank exceeded its limit of 64"
        );
    }
}
