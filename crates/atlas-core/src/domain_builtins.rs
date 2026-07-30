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
use std::sync::Arc;

use malachite::{Integer as BigInt, Rational as BigRational};

use atlas_real_group::{
    build_presentations, central_fiber, dual_cartan_correspondence, dual_inner_class,
    AdjointFiberBudget, BasedRootDatum, CartanClass, CartanClassification,
    CartanClassificationBudget, CartanId, Coweight, ExternalFormOrder, InnerClass,
    InnerClassLayout, IntegerLatticeBudget, InvolutionTable, InvolutionTableBudget, KgbGraph,
    KgbId, KgbStatus, LatticeInvolution, RealFormPresentation, RealFormSeed, RootSystem,
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
    WeylElement(WeylEltValue),
    CartanClass(Arc<InnerClassContext>, CartanId),
}

impl PartialEq for DomainValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::LieType(left), Self::LieType(right)) => left == right,
            (Self::RootDatum(left), Self::RootDatum(right)) => left == right,
            (Self::InnerClass(left), Self::InnerClass(right)) => {
                left.inner_class == right.inner_class
            }
            (Self::RealForm(left), Self::RealForm(right)) => {
                left.parent.inner_class == right.parent.inner_class
                    && left.internal == right.internal
            }
            (Self::KgbElement(left, left_id), Self::KgbElement(right, right_id)) => {
                left.parent.inner_class == right.parent.inner_class
                    && left.internal == right.internal
                    && left_id == right_id
            }
            // Group equality on the canonical root-permutation
            // representation: braid-equivalent words compare equal.
            (Self::WeylElement(left), Self::WeylElement(right)) => {
                left.context.handle == right.context.handle && left.element == right.element
            }
            (Self::CartanClass(left, left_id), Self::CartanClass(right, right_id)) => {
                left.inner_class == right.inner_class && left_id == right_id
            }
            _ => false,
        }
    }
}

impl Eq for DomainValue {}

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
        }
    }
}

/// The language-facing kind name, used by diagnostics and type printing.
pub fn kind_name(value: &DomainValue) -> &'static str {
    match value {
        DomainValue::LieType(_) => "LieType",
        DomainValue::RootDatum(_) => "RootDatum",
        DomainValue::InnerClass(_) => "InnerClass",
        DomainValue::RealForm(_) => "RealForm",
        DomainValue::KgbElement(_, _) => "KGBElt",
        DomainValue::WeylElement(_) => "WeylElt",
        DomainValue::CartanClass(_, _) => "CartanClass",
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
    let lie_type = infer_lie_type(datum.cartan_matrix(), datum.lattice_rank(), span)?;
    let handle = RootDatumHandle {
        isogeny: classify_isogeny(&datum),
        datum: Arc::new(datum),
        lie_type,
        prefers_coroots: parent.root_datum.prefers_coroots,
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
        "real_form" => {
            arity(name, arguments, 2, span)?;
            let Value::Domain(DomainValue::InnerClass(context)) = &arguments[0] else {
                return Err(type_error(span, "expected an InnerClass"));
            };
            let external = as_usize(&arguments[1], span)?;
            context
                .order
                .internal(external)
                .ok_or_else(|| runtime(span, format!("Illegal real form number: {external}")))?;
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
        "cross" | "Cayley" | "status" => {
            arity(name, arguments, 2, span)?;
            let generator = as_usize(&arguments[0], span)?;
            let (context, id) = as_kgb_element(&arguments[1], span)?;
            check_generator(context, generator, span)?;
            if context.graph.element(id).is_none() {
                return Err(runtime(span, "Inexistent KGB element"));
            }
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
        "real_form" => {
            arity(name, arguments, 2, span)?;
            let Value::Domain(DomainValue::InnerClass(context)) = &arguments[0] else {
                return Err(type_error(span, "expected an InnerClass"));
            };
            let external = as_usize(&arguments[1], span)?;
            let form = build_real_form(context, external, span)?;
            Ok(Value::Domain(DomainValue::RealForm(form)))
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
        "cross" => {
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
        "status" => {
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
        // decompose_KGB_wrapper (atlas-types.w:4429): the owning real form
        // and the element number, wrapped as a pair.
        "%" => {
            arity(name, arguments, 1, span)?;
            let (context, id) = as_kgb_element(&arguments[0], span)?;
            Ok(Value::Tuple(vec![
                Value::Domain(DomainValue::RealForm(Arc::clone(context))),
                Value::Integer(BigInt::from(id.index())),
            ]))
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
        // test. Binary equality is a domain relation (typed.rs), so only
        // the unary overload dispatches here.
        "=" | "!=" => {
            arity(name, arguments, 1, span)?;
            let value = as_weyl_elt(&arguments[0], span)?;
            let identity = value.element.is_identity();
            Ok(Value::Boolean(if name == "=" {
                identity
            } else {
                !identity
            }))
        }
        // W_elt_prod_wrapper (atlas-types.w:2421-2432): the group product.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::{SourceId, SourcePosition};
    use crate::value::Matrix;

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
}
