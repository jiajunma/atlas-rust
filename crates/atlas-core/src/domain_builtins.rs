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
//! real-form and inner-class prints are documented placeholders pending the
//! form-name machinery (adapter-sensitive, recorded in the bridge design).

use std::fmt;
use std::sync::Arc;

use malachite::Integer as BigInt;

use atlas_real_group::{
    AdjointFiberBudget, BasedRootDatum, CartanClassification, CartanClassificationBudget, Coweight,
    ExternalFormOrder, InnerClass, IntegerLatticeBudget, InvolutionTable, InvolutionTableBudget,
    KgbGraph, KgbId, KgbStatus, LatticeInvolution, RealFormSeed, StrongRealClassification,
    WeakRealFormId, Weight,
};

use crate::diagnostic::{Diagnostic, ErrorKind, SourceSpan};
use crate::value::Value;

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
    description: String,
}

/// The per-inner-class pipeline shared by every real form of the class.
#[derive(Debug)]
pub struct InnerClassContext {
    inner_class: InnerClass,
    classification: CartanClassification,
    strong: StrongRealClassification,
    order: ExternalFormOrder,
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
            Self::RootDatum(handle) => write!(formatter, "{}", handle.description),
            Self::InnerClass(context) => write!(
                formatter,
                "inner class with {} real forms",
                context.order.form_count()
            ),
            Self::RealForm(context) => {
                write!(formatter, "real form #{} of inner class", context.external)
            }
            Self::KgbElement(_, id) => write!(formatter, "KGB element #{}", id.index()),
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
    let flavour = if simply {
        "simply connected "
    } else {
        "adjoint "
    };
    Ok(RootDatumHandle {
        datum: Arc::new(datum),
        description: format!("{flavour}root datum of Lie type '{}'", lie_type.render()),
    })
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
    let inner_class = InnerClass::new((*handle.datum).clone(), involution, ROOT_BUDGET)
        .map_err(|error| runtime(span, error.to_string()))?;
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
    Ok(Arc::new(InnerClassContext {
        inner_class,
        classification,
        strong,
        order,
    }))
}

fn build_real_form(
    parent: &Arc<InnerClassContext>,
    external: usize,
    span: SourceSpan,
) -> Result<Arc<RealFormContext>, Diagnostic> {
    let internal = parent
        .order
        .internal(external)
        .ok_or_else(|| runtime(span, "Illegal real form number"))?;
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
    let Value::List(rows) = value else {
        return Err(type_error(span, "expected a matrix as a list of int lists"));
    };
    rows.iter()
        .map(|row| {
            let Value::List(entries) = row else {
                return Err(type_error(span, "expected a matrix as a list of int lists"));
            };
            entries
                .iter()
                .map(|entry| match entry {
                    Value::Integer(value) => i32::try_from(value)
                        .map_err(|_| type_error(span, "matrix entry out of range")),
                    _ => Err(type_error(span, "expected integer matrix entries")),
                })
                .collect()
        })
        .collect()
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

fn check_generator(
    context: &Arc<RealFormContext>,
    generator: usize,
    span: SourceSpan,
) -> Result<(), Diagnostic> {
    let rank = context.graph.semisimple_rank();
    if generator >= rank {
        // Posroot and negative indices are a documented phase-1 deferral.
        return Err(runtime(span, "Illegal root index"));
    }
    Ok(())
}

/// Dispatch one named application. Unknown names are Name errors.
pub(crate) fn call(name: &str, arguments: &[Value], span: SourceSpan) -> Result<Value, Diagnostic> {
    match name {
        "Lie_type" => {
            arity(name, arguments, 1, span)?;
            Ok(Value::Domain(DomainValue::LieType(as_lie_type(
                &arguments[0],
                span,
            )?)))
        }
        "simply_connected" | "adjoint" => {
            arity(name, arguments, 1, span)?;
            let lie_type = as_lie_type(&arguments[0], span)?;
            let handle = build_datum(&lie_type, name == "simply_connected", span)?;
            Ok(Value::Domain(DomainValue::RootDatum(handle)))
        }
        "inner_class" => {
            arity(name, arguments, 2, span)?;
            let Value::Domain(DomainValue::RootDatum(handle)) = &arguments[0] else {
                return Err(type_error(span, "expected a RootDatum"));
            };
            let matrix = as_matrix(&arguments[1], span)?;
            let context = build_inner_class(handle, matrix, span)?;
            Ok(Value::Domain(DomainValue::InnerClass(context)))
        }
        "nr_of_real_forms" => {
            arity(name, arguments, 1, span)?;
            let Value::Domain(DomainValue::InnerClass(context)) = &arguments[0] else {
                return Err(type_error(span, "expected an InnerClass"));
            };
            Ok(Value::Integer(BigInt::from(context.order.form_count())))
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
        "quasisplit_form" => {
            arity(name, arguments, 1, span)?;
            let Value::Domain(DomainValue::InnerClass(context)) = &arguments[0] else {
                return Err(type_error(span, "expected an InnerClass"));
            };
            let external = context.order.quasisplit_external();
            let form = build_real_form(context, external, span)?;
            Ok(Value::Domain(DomainValue::RealForm(form)))
        }
        "form_number" => {
            arity(name, arguments, 1, span)?;
            let context = as_real_form(&arguments[0], span)?;
            Ok(Value::Integer(BigInt::from(context.external)))
        }
        "KGB_size" => {
            arity(name, arguments, 1, span)?;
            let context = as_real_form(&arguments[0], span)?;
            Ok(Value::Integer(BigInt::from(context.graph.size())))
        }
        "KGB" => {
            arity(name, arguments, 2, span)?;
            let context = as_real_form(&arguments[0], span)?;
            let index = as_usize(&arguments[1], span)?;
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
            let (context, id) = as_kgb_element(&arguments[0], span)?;
            let length = context
                .graph
                .length(id)
                .ok_or_else(|| runtime(span, "Inexistent KGB element"))?;
            Ok(Value::Integer(BigInt::from(length)))
        }
        "torus_factor" => {
            arity(name, arguments, 1, span)?;
            let (context, id) = as_kgb_element(&arguments[0], span)?;
            let factor = context
                .graph
                .torus_factor(id, &context.table)
                .map_err(|error| runtime(span, error.to_string()))?;
            Ok(Value::List(
                factor
                    .to_rationals()
                    .into_iter()
                    .map(Value::Rational)
                    .collect(),
            ))
        }
        unknown => Err(Diagnostic::new(
            ErrorKind::Name,
            format!("undefined function `{unknown}`"),
            Some(span),
        )),
    }
}
