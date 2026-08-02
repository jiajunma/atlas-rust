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
use std::collections::BTreeSet;
use std::fmt;
use std::fmt::Write as _;
use std::num::{NonZeroI32, NonZeroU64};
use std::sync::Arc;

use malachite::{Integer as BigInt, Rational as BigRational};

use atlas_real_group::{
    adapted_relation_basis, annihilator_modulo as relation_annihilator_modulo, build_presentations,
    central_fiber, checked_inner_class_letters, classify_involution as domain_classify_involution,
    dual_cartan_correspondence, dual_inner_class, dual_involution as block_dual_involution,
    elected_square_root, fiber_rank, filter_relation_units as domain_filter_relation_units,
    inner_class_with_twisted_involution, layout_involution, longest_action, minimal_torus_part,
    on_basis as lattice_on_basis, pair, quotient_relation_basis as domain_quotient_relation_basis,
    replace_relation_generators as domain_replace_relation_generators, AdjointFiberBudget,
    BasedRootDatum, BlockDescent, BlockGraph, CartanClass, CartanClassification,
    CartanClassificationBudget, CartanId, Coweight, ExternalFormOrder, InnerClass,
    InnerClassLayout, IntegerLatticeBudget, InvolutionTable, InvolutionTableBudget, KType,
    KgbGraph, KgbId, KgbStatus, KlPol, KlTable, LatticeInvolution, ModTwoVector, RationalWeight,
    RealFormPresentation, RealFormSeed, RelationBasis, RelationError, RelationGenerator,
    RelationMatrix, RepContext, RootId, RootInvolutionData, RootKind, RootSystem, StandardRepr,
    StrongRealClassification, StructureError, WeakRealFormId, Weight, WeylElement, WeylInterface,
};

use crate::diagnostic::{Diagnostic, ErrorKind, SourceSpan};
use crate::value::{Matrix, RatVec, Value, Vec32};

/// Upstream Lie-type letter bounds (atlas-types.w:165-211) and RANK_MAX.
const RANK_MAX: usize = 32;

const INTEGER_BUDGET: IntegerLatticeBudget =
    IntegerLatticeBudget::new(64, 1_000_000, 1_000_000, 256);
/// Covers |W| up to E6 (51,840); larger groups need budget control first.
const WEYL_BUDGET: usize = 200_000;
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

/// The per-inner-class pipeline shared by every real form of the class.
#[derive(Debug)]
pub struct InnerClassContext {
    root_datum: RootDatumHandle,
    inner_class: InnerClass,
    classification: CartanClassification,
    strong: StrongRealClassification,
    order: ExternalFormOrder,
    layout: InnerClassLayout,
    dual_form_count: usize,
    /// Per Cartan class (crate Cartan order): the corresponding dual
    /// inner-class Cartan and its weak-real-form count, the upstream
    /// `numDualRealForms` of the class's dual fiber.
    dual_cartans: Vec<(CartanId, usize)>,
    forms: Vec<RealFormPresentation>,
}

/// One real form's frozen pipeline: seed, completed table, and KGB graph.
#[derive(Debug)]
pub struct RealFormContext {
    parent: Arc<InnerClassContext>,
    external: usize,
    internal: WeakRealFormId,
    table: InvolutionTable,
    graph: KgbGraph,
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

/// Bind the representation context of a real form's frozen pipeline. The
/// `RealFormContext` owns exactly the borrow triple the crate `RepContext`
/// needs (parent inner class, involution table, KGB graph).
fn rep_context(context: &RealFormContext) -> RepContext<'_> {
    RepContext::new(&context.parent.inner_class, &context.table, &context.graph)
        .expect("a constructed real form yields a valid Rep_context")
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

fn build_inner_class_context(
    handle: &RootDatumHandle,
    inner_class: InnerClass,
    span: SourceSpan,
) -> Result<Arc<InnerClassContext>, Diagnostic> {
    let class_budget = CartanClassificationBudget::new(
        INTEGER_BUDGET,
        AdjointFiberBudget::new(INTEGER_BUDGET, 1_000_000, 10_000_000),
        WEYL_BUDGET,
        4_096,
        4_096,
    );
    let classification = CartanClassification::build(&inner_class, &class_budget)
        .map_err(|error| runtime(span, error.to_string()))?;
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
    let dual_classification = CartanClassification::build(&dual_inner, &class_budget)
        .map_err(|error| runtime(span, error.to_string()))?;
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
    Ok(Arc::new(RealFormContext {
        parent: Arc::clone(parent),
        external,
        internal,
        table,
        graph,
    }))
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
    Ok(Arc::new(RealFormContext {
        parent: Arc::clone(parent),
        external: plan.external,
        internal: plan.internal,
        table,
        graph,
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
fn kl_pol_at(
    kl_table: &KlTable,
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

/// The transitive closure of a Hasse diagram as a `lesseq` matrix
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

/// The owning-form identity check shared by the equivalence and polynomial
/// wrappers (atlas-types.w:5323-5331, 5668-5676, 7786-7803): the mismatch
/// diagnostic precedes the wrapper's no-value gate.
fn require_same_form(
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

/// fiberSize/dualFiberSize (innerclass.cpp:603-640): the per-form count of
/// adjoint-fiber elements at one Cartan, enumerated by the fiber's
/// canonical masks exactly like `fiber_partition`.
fn fiber_size(
    cartan: &CartanClass,
    form: WeakRealFormId,
    span: SourceSpan,
) -> Result<u64, Diagnostic> {
    let dimension = cartan.grading().adjoint_fiber().dimension();
    // The partition's mask-bits bound keeps this shift in range.
    let element_count = 1_u64
        .checked_shl(
            u32::try_from(dimension)
                .map_err(|_| runtime(span, "internal fiber dimension overflow"))?,
        )
        .ok_or_else(|| runtime(span, "internal fiber dimension overflow"))?;
    let mut count = 0_u64;
    for mask in 0..element_count {
        let local = cartan
            .partition()
            .class_of_mask(mask)
            .map_err(|error| runtime(span, error.to_string()))?;
        if cartan.labels().label(local) == Some(form) {
            count += 1;
        }
    }
    Ok(count)
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
        let dual_cartan = dual
            .classification
            .cartan_class(*dual_id)
            .expect("the correspondence covers every Cartan class");
        let orbit = u64::try_from(cartan.twisted_involution_count())
            .map_err(|_| runtime(span, "internal block size overflow"))?;
        let factor = fiber_size(cartan, internal, span)?;
        let dual_factor = fiber_size(dual_cartan, dual_internal, span)?;
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

/// Shared tail of both `twist` wrappers: apply the crate twist and rewrap
/// the target in the same real-form context. `Ok(None)` from the crate is
/// upstream's `UndefKGB`, surfaced with the established inexistent
/// wording.
fn twist_element(
    context: &Arc<RealFormContext>,
    id: KgbId,
    delta: &LatticeInvolution,
    twist: &[usize],
    span: SourceSpan,
) -> Result<Value, Diagnostic> {
    let target = context
        .graph
        .twisted(id, &context.table, delta, twist)
        .map_err(|error| runtime(span, error.to_string()))?
        .ok_or_else(|| runtime(span, "Inexistent KGB element"))?;
    Ok(Value::Domain(DomainValue::KgbElement(
        Arc::clone(context),
        target,
    )))
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
        let generator = usize::try_from(&integer).unwrap_or(usize::MAX);
        if generator >= semisimple_rank {
            return Err(runtime(
                span,
                format!("Illegal Weyl word entry {integer} (should be <{semisimple_rank})"),
            ));
        }
        word.push(generator);
    }
    Ok(word)
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
                let generator = as_usize(&arguments[0], span)?;
                let (context, id) = as_kgb_element(&arguments[1], span)?;
                check_generator(context, generator, span)?;
                if context.graph.element(id).is_none() {
                    return Err(runtime(span, "Inexistent KGB element"));
                }
            }
        }
        // Fokko_block_wrapper's is_dual gate precedes its no_value check
        // (atlas-types.w:4790-4794).
        "block" => {
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
            let (context, _) = as_kgb_element(&arguments[0], span)?;
            compatible_outer_twist(context, &arguments[1], span)?;
        }
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
                    require_same_form(
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
                    require_same_form(
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
                require_same_form(
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
                require_same_form(
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
                require_same_form(
                    &accumulator.rf,
                    &ktype.context,
                    "Real form mismatch when adding a term to a K_type",
                    span,
                )?;
            }
            [Value::Domain(DomainValue::ParamPol(accumulator)), Value::Domain(DomainValue::Param(parameter))] =>
            {
                require_same_form(
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
                require_same_form(
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
                        None => text.push_str(&format!("{:width$}", '*', width = width + pad)),
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
                        None => text.push_str(&format!("{:width$}", '*')),
                    }
                    text.push(',');
                    match pair.1 {
                        Some(second) => text.push_str(&format!("{:width$}", second)),
                        None => text.push_str(&format!("{:width$}", '*')),
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
                    // Weyl word (prettyprint::printWeylElt): one-based
                    // generators comma-separated, `e` for the identity.
                    let word =
                        weyl_reduced_word(&block.rf.parent.inner_class, record.weyl_element());
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
                        text.push_str(&format!("{:width$}", ""));
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
                // print_KL_list: the sorted distinct nonzero polynomials.
                let mut polynomials: Vec<KlPol> = Vec::new();
                for y in 0..size {
                    for x in 0..=y {
                        let polynomial = kl_pol_at(&kl_table, x, y, span)?;
                        if !polynomial.is_zero() && !polynomials.contains(&polynomial) {
                            polynomials.push(polynomial);
                        }
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

/// Dispatch one named application. Unknown names are Name errors.
pub(crate) fn call(name: &str, arguments: &[Value], span: SourceSpan) -> Result<Value, Diagnostic> {
    match name {
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
        // simple_roots/simple_coroots (atlas-types.w:1638-1658): one row
        // per simple (co)root.
        "simple_roots" | "simple_coroots" => {
            arity(name, arguments, 1, span)?;
            let handle = as_root_datum(&arguments[0], span)?;
            let rows: Vec<Vec<i32>> = if name == "simple_coroots" {
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
            matrix_value(&rows, span)
        }
        // root_coradical / coroot_radical (atlas-types.w:2254-2255): the
        // simple roots/coroots followed by a basis of the kernel of the
        // coroots/roots (the coradical/radical), as matrix rows.
        "root_coradical" | "coroot_radical" => {
            arity(name, arguments, 1, span)?;
            let handle = as_root_datum(&arguments[0], span)?;
            let mut rows: Vec<Vec<i32>> = if name == "coroot_radical" {
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
            let extra: Vec<Vec<i32>> = if name == "coroot_radical" {
                handle
                    .datum
                    .radical_basis()
                    .map_err(|error| structure_diagnostic(error, span))?
                    .iter()
                    .map(|coweight| coweight.as_slice().to_vec())
                    .collect()
            } else {
                handle
                    .datum
                    .coradical_basis()
                    .map_err(|error| structure_diagnostic(error, span))?
                    .iter()
                    .map(|weight| weight.as_slice().to_vec())
                    .collect()
            };
            rows.extend(extra);
            matrix_value(&rows, span)
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
            let handle = as_root_datum(&arguments[0], span)?;
            Ok(Value::Integer(BigInt::from(handle.datum.lattice_rank())))
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
        // then the fibred product of the two forms' KGB sets.
        "block" => {
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
            if arguments.len() == 3 {
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
            if arguments.len() == 3 {
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
            let rc = rep_context(&parameter.context);
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
            let z = (0..size)
                .find(|&index| block.graph.x(index) == Some(parameter.repr.x()))
                .ok_or_else(|| runtime(span, "parameter not in the common block"))?;
            let z_length = block.graph.length(z).unwrap_or_default();
            let lambda_rho = rc
                .lambda_rho(&parameter.repr)
                .map_err(|error| structure_diagnostic(error, span))?;
            let gamma = parameter.repr.gamma().clone();
            let mut terms: Vec<(SplitValue, StandardRepr)> = Vec::new();
            let mut x = z + 1;
            while x > 0 {
                x -= 1;
                let index = kl_table
                    .kl_pol(x, z)
                    .map_err(|error| structure_diagnostic(error, span))?;
                let pol = kl_table
                    .pool()
                    .get(index)
                    .cloned()
                    .unwrap_or_else(KlPol::zero);
                if pol.is_zero() {
                    continue;
                }
                // Evaluate at q = s by Horner (repr.cpp:2152-2155):
                // pol[d_high] * s + pol[d_high-1], the coefficients stored
                // least degree first.
                let mut eval = SplitValue::new(0, 0);
                let s = SplitValue::new(0, 1);
                let degree = pol.degree();
                let mut d = degree + 1;
                while d > 0 {
                    d -= 1;
                    eval = eval.mul(s).add(SplitValue::new(pol.coefficient(d), 0));
                }
                let x_length = block.graph.length(x).unwrap_or_default();
                if (z_length - x_length) % 2 != 0 {
                    eval = eval.neg();
                }
                if eval.is_zero() {
                    continue;
                }
                let repr = rc
                    .sr_gamma(block.graph.x(x).expect("in-range"), &lambda_rho, &gamma)
                    .map_err(|error| structure_diagnostic(error, span))?;
                if height_bound.is_none_or(|bound| repr.height() <= bound) {
                    terms.push((eval, repr));
                }
            }
            Ok(Value::Domain(DomainValue::ParamPol(ParamPolValue {
                rf: Arc::clone(&parameter.context),
                terms,
            })))
        }
        // raw_KL / dual_KL (atlas-types.w:9101-9102, 8424-8460): the KL
        // table of a block as (matrix of polynomial indices, polynomial
        // pool as coefficient vectors, length stops).
        "raw_KL" | "dual_KL" => {
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
            let mut columns = vec![vec![0_i32; size]; size];
            for (y, column) in columns.iter_mut().enumerate().skip(1) {
                for (x, slot) in column.iter_mut().enumerate().take(y) {
                    let index = kl_table
                        .kl_pol(x, y)
                        .map_err(|error| structure_diagnostic(error, span))?;
                    *slot = i32::try_from(index).map_err(|_| runtime(span, "KL index overflow"))?;
                }
            }
            // Diagonal entries P_{y,y} = 1 (index of the constant 1).
            for (y, column) in columns.iter_mut().enumerate() {
                let index = kl_table
                    .kl_pol(y, y)
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
        // W_graph / W_cells (atlas-types.w:7147-7170, 7210-7245): the
        // W-graph of a standard parameter's full block, respectively its
        // cell decomposition. W_graph returns (start, vertices) with each
        // vertex a (descent set, [(target, coefficient)]) pair; W_cells
        // returns (start, [(members, vertices)]).
        "W_graph" | "W_cells" => {
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
            let start = (0..size)
                .find(|&z| block.graph.x(z) == Some(parameter.repr.x()))
                .ok_or_else(|| runtime(span, "parameter not in the common block"))?;
            // kl::wGraph (kl.cpp:1042-1058): every mu pair contributes an
            // edge in both directions.
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
            let rank = block.graph.rank();
            let vertex = |element: usize, targets: &[(usize, i32)]| -> Value {
                let desc = kl_table.support().descent_set(element);
                let descents = Value::List(
                    (0..rank)
                        .filter(|&generator| desc.is_set(generator))
                        .map(|generator| Value::Integer(BigInt::from(generator)))
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
                let (mut partition, _induced) = strong_components(&oriented);
                // The oracle's Partition lists a cell's vertices ascending.
                for members in partition.iter_mut() {
                    members.sort_unstable();
                }
                let mut cells = Vec::new();
                for members in &partition {
                    let mut relno = vec![0_usize; size];
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
            let dual_parent = build_dual_inner_class(&parameter.context.parent, span)?;
            let dual_quasisplit = dual_parent.order.quasisplit_external();
            let dual_rf = build_real_form(&dual_parent, dual_quasisplit, span)?;
            let block = build_block(&parameter.context, &dual_rf, span)?;
            let rc = rep_context(&parameter.context);
            let lambda_rho = rc
                .lambda_rho(&parameter.repr)
                .map_err(|error| structure_diagnostic(error, span))?;
            let gamma = parameter.repr.gamma().clone();
            let mut param_list = Vec::with_capacity(block.graph.size());
            for z in 0..block.graph.size() {
                let x = block.graph.x(z).expect("in-range");
                let repr = rc
                    .sr_gamma(x, &lambda_rho, &gamma)
                    .map_err(|error| structure_diagnostic(error, span))?;
                param_list.push(Value::Domain(DomainValue::Param(ParamValue {
                    context: Arc::clone(&parameter.context),
                    repr,
                })));
            }
            let n = block.graph.size();
            let hasse = block.graph.bruhat_hasse();
            let mut columns = vec![vec![0_i32; n]; n];
            for (z, row) in hasse.iter().enumerate() {
                for &down in row {
                    columns[z][down] = 1;
                }
            }
            Ok(Value::Tuple(vec![
                Value::List(param_list),
                columns_matrix_value(&columns, n, span)?,
            ]))
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
            // The Weyl word of the Cartan involution (Weyl_group().word).
            let weyl = WeylElement::from_action(
                context.inner_class.root_system(),
                representative.weyl_action(),
            )
            .map_err(|error| structure_diagnostic(error, span))?;
            let word = weyl_reduced_word(&context.inner_class, &weyl);
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
                    require_same_form(
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
                    require_same_form(
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
                    let lam_rho = rc
                        .lambda_rho(&parameter.repr)
                        .map_err(|error| structure_diagnostic(error, span))?;
                    Ok(Value::Tuple(vec![
                        Value::Domain(DomainValue::KgbElement(
                            Arc::clone(&parameter.context),
                            parameter.repr.x(),
                        )),
                        Value::Vector(Vec32(lam_rho.as_slice().to_vec())),
                        Value::RatVector(ratvec_from_rational_weight(
                            parameter.repr.gamma(),
                            span,
                        )?),
                    ]))
                }
                other => Err(type_error(
                    span,
                    format!("expected a KGBElt, Block, Split, KType, or Param, found {other}"),
                )),
            }
        }
        // KGB_twist_wrapper (atlas-types.w:4616) twists by the inner
        // class's distinguished involution; KGB_outer_twist_wrapper
        // (atlas-types.w:4634) validates the user's involution with
        // test_compatible first.
        "twist" => match arguments {
            [element] => {
                let (context, id) = as_kgb_element(element, span)?;
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
                twist_element(context, id, &delta, &twist, span)
            }
            [element, matrix] => {
                let (context, id) = as_kgb_element(element, span)?;
                let (delta, twist) = compatible_outer_twist(context, matrix, span)?;
                twist_element(context, id, &delta, &twist, span)
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
                other => {
                    return Err(type_error(
                        span,
                        format!("expected a WeylElt, found {other}"),
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
                require_same_form(
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
                require_same_form(
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
                require_same_form(
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
            // add_module_wrapper (atlas-types.w:7786-7795): expand the
            // final parameter and add it (expand_final).
            [Value::Domain(DomainValue::ParamPol(accumulator)), Value::Domain(DomainValue::Param(parameter))] =>
            {
                require_same_form(
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
                require_same_form(
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
                require_same_form(
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
                require_same_form(
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
                require_same_form(
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
                require_same_form(
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
        // W_elt_gen_prod_wrapper (atlas-types.w:2456-2465): right
        // multiplication by one simple generator; check_Weyl_gen echoes
        // the signed index on rejection (atlas-types.w:2447-2454).
        // size_of_block_wrapper (atlas-types.w:4820-4824): the block size.
        "#" => match arguments {
            [Value::Domain(DomainValue::WeylElement(value)), Value::Integer(generator)] => {
                let rank = value.context.handle.datum.semisimple_rank();
                // check_Weyl_gen rejects negative (wrapping cast) and
                // over-rank indices alike, echoing the signed value.
                let converted = usize::try_from(generator).ok();
                let Some(generator) = converted.filter(|&generator| generator < rank) else {
                    return Err(runtime(
                        span,
                        format!(
                            "Generator {generator} out of range for Weyl group (should be <{rank})"
                        ),
                    ));
                };
                let (product, _) = value
                    .element
                    .right_multiply_simple(&value.context.system, generator)
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
