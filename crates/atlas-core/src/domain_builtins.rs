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
use std::sync::Arc;

use malachite::{Integer as BigInt, Rational as BigRational};

use atlas_real_group::{
    build_presentations, dual_real_form_count, AdjointFiberBudget, BasedRootDatum,
    CartanClassification, CartanClassificationBudget, Coweight, ExternalFormOrder, InnerClass,
    InnerClassLayout, IntegerLatticeBudget, InvolutionTable, InvolutionTableBudget, KgbGraph,
    KgbId, KgbStatus, LatticeInvolution, RealFormPresentation, RealFormSeed,
    StrongRealClassification, WeakRealFormId, Weight,
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
    let layout = InnerClassLayout::build(&inner_class, &INTEGER_BUDGET)
        .map_err(|error| runtime(span, error.to_string()))?;
    let dual_form_count = dual_real_form_count(
        &inner_class,
        WEYL_BUDGET,
        &INTEGER_BUDGET,
        &AdjointFiberBudget::new(INTEGER_BUDGET, 1_000_000, 10_000_000),
        FIBER_BUDGET,
        ROOT_BUDGET,
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
        forms,
    }))
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
                .ok_or_else(|| runtime(span, "Illegal real form number"))?;
        }
        "KGB" => {
            arity(name, arguments, 2, span)?;
            let context = as_real_form(&arguments[0], span)?;
            let index = as_usize(&arguments[1], span)?;
            if index >= context.graph.size() {
                return Err(runtime(span, "Inexistent KGB element"));
            }
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
        other => {
            return Err(runtime(
                span,
                format!("no validation policy registered for '{other}'"),
            ));
        }
    }
    Ok(())
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
        "involution" => {
            arity(name, arguments, 1, span)?;
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
}
