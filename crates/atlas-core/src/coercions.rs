//! The axis coercion table and type-proximity predicates.
//!
//! Ports the upstream registrations (global.w:2526-2552 followed by
//! atlas-types.w:9137-9144) IN ORDER — `coercion_between` and
//! `row_coercion` are first-match scans in registration order (the gate
//! index prunes only entries `same` would certainly reject), so a list
//! display in `mat` context must find `[vec]->mat` before any other
//! row-into-mat entry — plus `is_close` (axis-types.w:3246-3285; three
//! bits: 0x1 = left coerces to right, 0x2 = right to left, 0x4 = close)
//! and `broader_eq` (axis-types.w:3339-3364), the balancing order.
//! Conversion NODES arrive with the typed pipeline; this module only
//! answers applicability.

use std::sync::OnceLock;

use crate::types::{Prim, Type, TypeTable};

/// One registered coercion; `tag` is the upstream conversion name that
/// prints as `tag:expr` on conversion nodes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Coercion {
    pub from: Type,
    pub to: Type,
    pub tag: &'static str,
}

fn prim(prim: Prim) -> Type {
    Type::Primitive(prim)
}

fn row(component: Type) -> Type {
    Type::row(component)
}

/// The full registration list, upstream order.
pub fn coercion_table() -> &'static [Coercion] {
    static TABLE: OnceLock<Vec<Coercion>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let int = || prim(Prim::Int);
        let rat = || prim(Prim::Rat);
        let vec = || prim(Prim::Vec);
        let mat = || prim(Prim::Mat);
        let ratvec = || prim(Prim::RatVec);
        vec![
            Coercion {
                from: int(),
                to: rat(),
                tag: "QI",
            },
            Coercion {
                from: row(int()),
                to: vec(),
                tag: "V[I]",
            },
            Coercion {
                from: row(rat()),
                to: ratvec(),
                tag: "Qv[Q]",
            },
            Coercion {
                from: vec(),
                to: row(int()),
                tag: "[I]V",
            },
            Coercion {
                from: ratvec(),
                to: row(rat()),
                tag: "[Q]Qv",
            },
            Coercion {
                from: vec(),
                to: ratvec(),
                tag: "QvV",
            },
            Coercion {
                from: row(int()),
                to: ratvec(),
                tag: "Qv[I]",
            },
            Coercion {
                from: vec(),
                to: row(rat()),
                tag: "[Q]V",
            },
            Coercion {
                from: row(int()),
                to: row(rat()),
                tag: "[Q][I]",
            },
            Coercion {
                from: row(vec()),
                to: mat(),
                tag: "M[V]",
            },
            Coercion {
                from: row(row(int())),
                to: mat(),
                tag: "M[[I]]",
            },
            Coercion {
                from: row(row(int())),
                to: row(vec()),
                tag: "[V][[I]]",
            },
            Coercion {
                from: row(vec()),
                to: row(row(int())),
                tag: "[[I]][V]",
            },
            Coercion {
                from: mat(),
                to: row(vec()),
                tag: "[V]M",
            },
            Coercion {
                from: mat(),
                to: row(row(int())),
                tag: "[[I]]M",
            },
            Coercion {
                from: mat(),
                to: row(ratvec()),
                tag: "[Qv]M",
            },
            Coercion {
                from: mat(),
                to: row(row(rat())),
                tag: "[[Q]]M",
            },
            Coercion {
                from: row(vec()),
                to: row(ratvec()),
                tag: "[Qv][V]",
            },
            Coercion {
                from: row(vec()),
                to: row(row(rat())),
                tag: "[[Q]][V]",
            },
            Coercion {
                from: row(row(int())),
                to: row(ratvec()),
                tag: "[Qv][[I]]",
            },
            Coercion {
                from: row(row(int())),
                to: row(row(rat())),
                tag: "[[Q]][[I]]",
            },
            Coercion {
                from: prim(Prim::String),
                to: prim(Prim::LieType),
                tag: "LT",
            },
            Coercion {
                from: prim(Prim::InnerClass),
                to: prim(Prim::RootDatum),
                tag: "RdIc",
            },
            Coercion {
                from: prim(Prim::RealForm),
                to: prim(Prim::InnerClass),
                tag: "IcRf",
            },
            Coercion {
                from: prim(Prim::RealForm),
                to: prim(Prim::RootDatum),
                tag: "RdRf",
            },
            Coercion {
                from: int(),
                to: prim(Prim::Split),
                tag: "SpI",
            },
            Coercion {
                from: Type::tuple(vec![prim(Prim::Int), prim(Prim::Int)]),
                to: prim(Prim::Split),
                tag: "Sp(I,I)",
            },
            Coercion {
                from: prim(Prim::KType),
                to: prim(Prim::KTypePol),
                tag: "KpolK",
            },
            Coercion {
                from: prim(Prim::Param),
                to: prim(Prim::ParamPol),
                tag: "PolP",
            },
        ]
    })
}

/// Expand a tabled type one level for structural comparison.
fn expanded<'a>(type_: &'a Type, table: &'a TypeTable) -> &'a Type {
    match type_ {
        Type::Tabled(number) => table.expansion(*number),
        other => other,
    }
}

/// Structural equality through tabled expansions.
fn same(a: &Type, b: &Type, table: &TypeTable) -> bool {
    // The table canonicalises, so two tabled types compare by NUMBER
    // (types.rs:110-113). Besides being cheaper, this is what keeps
    // recursive types (e.g. inf_list = (->inf_node), whose expansion
    // refers back to itself) from expanding forever here.
    if let (Type::Tabled(x), Type::Tabled(y)) = (a, b) {
        return x == y;
    }
    let (a, b) = (expanded(a, table), expanded(b, table));
    match (a, b) {
        (Type::Primitive(x), Type::Primitive(y)) => x == y,
        (Type::Undetermined, Type::Undetermined) => true,
        (Type::Row(x), Type::Row(y)) => same(x, y, table),
        (Type::Function(x), Type::Function(y)) => {
            same(&x.0, &y.0, table) && same(&x.1, &y.1, table)
        }
        (Type::Tuple(xs), Type::Tuple(ys)) | (Type::Union(xs), Type::Union(ys)) => {
            xs.len() == ys.len() && xs.iter().zip(ys).all(|(x, y)| same(x, y, table))
        }
        _ => false,
    }
}

/// Top-level shape of a type for coercion shortlisting: `same` holds only
/// between equal gates (its match arms require equal constructors, and equal
/// primitives), so entries whose endpoint gates differ can never match.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Gate {
    Primitive(Prim),
    Row,
    Tuple,
    Union,
    Function,
    Undetermined,
}

fn gate_of(type_: &Type, table: &TypeTable) -> Option<Gate> {
    match expanded(type_, table) {
        Type::Primitive(prim) => Some(Gate::Primitive(*prim)),
        Type::Row(_) => Some(Gate::Row),
        Type::Tuple(_) => Some(Gate::Tuple),
        Type::Union(_) => Some(Gate::Union),
        Type::Function(_) => Some(Gate::Function),
        Type::Undetermined => Some(Gate::Undetermined),
        // A self-referential placeholder mid-`set_type` (expansion follows
        // chains, so a Tabled result refers to itself): no gate, scan all.
        Type::Tabled(_) => None,
    }
}

/// Gate of a ground table entry endpoint (entries contain no tabled types).
fn gate_of_ground(type_: &Type) -> Gate {
    match type_ {
        Type::Primitive(prim) => Gate::Primitive(*prim),
        Type::Row(_) => Gate::Row,
        Type::Tuple(_) => Gate::Tuple,
        Type::Union(_) => Gate::Union,
        Type::Function(_) => Gate::Function,
        Type::Undetermined => Gate::Undetermined,
        Type::Tabled(_) => unreachable!("coercion table entries are ground"),
    }
}

/// Per-entry endpoint gates, parallel to [`coercion_table`].
fn coercion_gates() -> &'static [(Gate, Gate)] {
    static GATES: OnceLock<Vec<(Gate, Gate)>> = OnceLock::new();
    GATES.get_or_init(|| {
        coercion_table()
            .iter()
            .map(|coercion| (gate_of_ground(&coercion.from), gate_of_ground(&coercion.to)))
            .collect()
    })
}

/// Gate codes for the bucket index: the twenty primitives keep their
/// declaration order, the structural constructors follow.
const GATE_KINDS: usize = 25;

fn gate_code(gate: Gate) -> usize {
    match gate {
        Gate::Primitive(prim) => prim as usize,
        Gate::Row => 20,
        Gate::Tuple => 21,
        Gate::Union => 22,
        Gate::Function => 23,
        Gate::Undetermined => 24,
    }
}

/// Entry numbers bucketed by (from-gate, to-gate): `buckets` holds
/// (start, len) ranges into `entries`, with registration order preserved
/// inside each bucket so the first-match result is the linear scan's.
fn coercion_index() -> &'static ([(u16, u16); GATE_KINDS * GATE_KINDS], Vec<u8>) {
    static INDEX: OnceLock<([(u16, u16); GATE_KINDS * GATE_KINDS], Vec<u8>)> = OnceLock::new();
    INDEX.get_or_init(|| {
        let gates = coercion_gates();
        let mut counts = [0u16; GATE_KINDS * GATE_KINDS];
        for &(from_gate, to_gate) in gates {
            counts[gate_code(from_gate) * GATE_KINDS + gate_code(to_gate)] += 1;
        }
        let mut buckets = [(0u16, 0u16); GATE_KINDS * GATE_KINDS];
        let mut total = 0u16;
        for (bucket, &count) in buckets.iter_mut().zip(&counts) {
            *bucket = (total, count);
            total += count;
        }
        let mut entries = vec![0u8; total as usize];
        let mut filled = [0u16; GATE_KINDS * GATE_KINDS];
        for (index, &(from_gate, to_gate)) in gates.iter().enumerate() {
            let slot = gate_code(from_gate) * GATE_KINDS + gate_code(to_gate);
            entries[(buckets[slot].0 + filled[slot]) as usize] = index as u8;
            filled[slot] += 1;
        }
        (buckets, entries)
    })
}

/// The first registered coercion from `from` to `to`, if any. The gate
/// index only prunes entries that `same` would certainly reject, so the
/// first-match order of the surviving entries is unchanged.
pub fn coercion_between<'a>(from: &Type, to: &Type, table: &TypeTable) -> Option<&'a Coercion> {
    if let (Some(from_gate), Some(to_gate)) = (gate_of(from, table), gate_of(to, table)) {
        let (buckets, entries) = coercion_index();
        let (start, len) = buckets[gate_code(from_gate) * GATE_KINDS + gate_code(to_gate)];
        return entries[start as usize..(start + len) as usize]
            .iter()
            .map(|&index| &coercion_table()[index as usize])
            .find(|coercion| same(&coercion.from, from, table) && same(&coercion.to, to, table));
    }
    // A self-referential placeholder mid-`set_type` has no gate: scan all.
    coercion_table()
        .iter()
        .find(|coercion| same(&coercion.from, from, table) && same(&coercion.to, to, table))
}

/// For a list display in non-row context `final_type`: the FIRST table
/// entry with a row `from` and that target, yielding the component type the
/// display's elements must take (`mat` context yields `vec`, never `[int]`).
pub fn row_coercion<'a>(final_type: &Type, table: &TypeTable) -> Option<(&'a Coercion, &'a Type)> {
    let target_gate = gate_of(final_type, table);
    coercion_table()
        .iter()
        .zip(coercion_gates())
        // The gate prunes only entries `same` would certainly reject.
        .filter(move |(_, gates)| target_gate.is_none_or(|gate| gates.1 == gate))
        .map(|(coercion, _)| coercion)
        .find_map(|coercion| {
            if !same(&coercion.to, final_type, table) {
                return None;
            }
            match &coercion.from {
                Type::Row(component) => Some((coercion, component.as_ref())),
                _ => None,
            }
        })
}

/// Three-bit proximity: 0x1 `x` coerces to `y`, 0x2 `y` to `x`, 0x4 close.
/// Equal types give 0x7; void and `*` are close to nothing.
pub fn is_close(x: &Type, y: &Type, table: &TypeTable) -> u8 {
    let (x, y) = (expanded(x, table), expanded(y, table));
    if x.is_void() || y.is_void() {
        return 0;
    }
    if matches!(x, Type::Undetermined) || matches!(y, Type::Undetermined) {
        return 0;
    }
    if same(x, y, table) {
        return 0x7;
    }
    // Upstream (axis-types.w:3258-3285) consults the coercion table only
    // when a primitive is involved; aggregate pairs go straight to the
    // componentwise recursion. The two flows agree here: every row-to-row
    // table entry derives from a component coercion the recursion also
    // finds ([int]->[rat] exactly because int->rat, [vec]->[[int]] exactly
    // because vec and [int] convert both ways, and likewise for the
    // ratvec/[rat] entries), no tuple-to-tuple or function entries exist,
    // and gate-mismatched pairs can never satisfy `same`.
    if matches!(x, Type::Primitive(_)) || matches!(y, Type::Primitive(_)) {
        let mut bits = 0;
        if coercion_between(x, y, table).is_some() {
            bits |= 0x1;
        }
        if coercion_between(y, x, table).is_some() {
            bits |= 0x2;
        }
        return if bits == 0 { bits } else { bits | 0x4 };
    }
    match (x, y) {
        (Type::Row(a), Type::Row(b)) => is_close(a, b, table),
        (Type::Tuple(xs), Type::Tuple(ys)) if xs.len() == ys.len() => xs
            .iter()
            .zip(ys)
            .fold(0x7, |bits, (a, b)| bits & is_close(a, b, table)),
        _ => 0,
    }
}

/// The balancing order: `a` is broader than or equal to `b`. Void is
/// broadest, `*` narrowest; a primitive absorbs whatever coerces into it;
/// rows and tuples go componentwise; functions need equal argument types.
pub fn broader_eq(a: &Type, b: &Type, table: &TypeTable) -> bool {
    // Equal tabled numbers are equal types (the table canonicalises);
    // short-circuiting also keeps recursive types from expanding
    // forever, as in `same`.
    if let (Type::Tabled(x), Type::Tabled(y)) = (a, b) {
        if x == y {
            return true;
        }
    }
    let (a, b) = (expanded(a, table), expanded(b, table));
    if a.is_void() {
        return true;
    }
    if matches!(b, Type::Undetermined) {
        return true;
    }
    if matches!(a, Type::Undetermined) || b.is_void() {
        return false;
    }
    if let Type::Primitive(_) = a {
        return is_close(a, b, table) & 0x2 != 0;
    }
    match (a, b) {
        (Type::Row(a), Type::Row(b)) => broader_eq(a, b, table),
        (Type::Tuple(xs), Type::Tuple(ys)) | (Type::Union(xs), Type::Union(ys)) => {
            xs.len() == ys.len() && xs.iter().zip(ys).all(|(a, b)| broader_eq(a, b, table))
        }
        (Type::Function(a), Type::Function(b)) => {
            same(&a.0, &b.0, table) && broader_eq(&a.1, &b.1, table)
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn int() -> Type {
        Type::Primitive(Prim::Int)
    }

    fn rat() -> Type {
        Type::Primitive(Prim::Rat)
    }

    fn vec_t() -> Type {
        Type::Primitive(Prim::Vec)
    }

    #[test]
    fn the_table_keeps_registration_order_and_first_match_wins() {
        let table = TypeTable::new();
        // mat context must yield component vec (the [vec]->mat entry comes
        // before [[int]]->mat).
        let (coercion, component) =
            row_coercion(&Type::Primitive(Prim::Mat), &table).expect("mat has a row coercion");
        assert_eq!(coercion.tag, "M[V]");
        assert_eq!(component, &vec_t());
        // ratvec context yields component rat.
        let (coercion, component) =
            row_coercion(&Type::Primitive(Prim::RatVec), &table).expect("ratvec row coercion");
        assert_eq!(coercion.tag, "Qv[Q]");
        assert_eq!(component, &rat());
        assert_eq!(coercion_table().len(), 29);
    }

    #[test]
    fn is_close_matches_the_documented_bit_contract() {
        let table = TypeTable::new();
        assert_eq!(is_close(&int(), &int(), &table), 0x7);
        // int coerces to rat but not back.
        assert_eq!(is_close(&int(), &rat(), &table), 0x5);
        assert_eq!(is_close(&rat(), &int(), &table), 0x6);
        // vec and [int] convert both ways: the ambiguous 0x7-unequal case.
        assert_eq!(is_close(&vec_t(), &Type::row(int()), &table), 0x7);
        assert_eq!(is_close(&int(), &Type::Primitive(Prim::String), &table), 0);
        assert_eq!(is_close(&Type::void(), &int(), &table), 0);
        assert_eq!(is_close(&Type::Undetermined, &int(), &table), 0);
        // Tuples AND componentwise: (int,int) vs (rat,rat) keeps only 0x5.
        let pair = |t: fn() -> Type| Type::tuple(vec![t(), t()]);
        assert_eq!(is_close(&pair(int), &pair(rat), &table), 0x5);
    }

    #[test]
    fn the_gate_index_matches_the_plain_first_match_scan() {
        let table = TypeTable::new();
        // A battery of query types hitting every gate shape: primitives
        // with and without entries, rows of each depth, tuples, a union,
        // a function, void, and undetermined.
        let queries = vec![
            int(),
            rat(),
            Type::Primitive(Prim::Vec),
            Type::Primitive(Prim::Mat),
            Type::Primitive(Prim::RatVec),
            Type::Primitive(Prim::String),
            Type::Primitive(Prim::LieType),
            Type::Primitive(Prim::RealForm),
            Type::Primitive(Prim::Split),
            Type::Primitive(Prim::KType),
            Type::Primitive(Prim::Bool),
            Type::row(int()),
            Type::row(rat()),
            Type::row(Type::Primitive(Prim::Vec)),
            Type::row(Type::row(int())),
            Type::row(Type::row(rat())),
            Type::tuple(vec![int(), int()]),
            Type::tuple(vec![int(), rat()]),
            Type::union_of(vec![int(), rat()]),
            Type::function(int(), rat()),
            Type::void(),
            Type::Undetermined,
        ];
        let naive = |from: &Type, to: &Type| {
            coercion_table()
                .iter()
                .find(|coercion| same(&coercion.from, from, &table) && same(&coercion.to, to, &table))
        };
        for from in &queries {
            for to in &queries {
                assert_eq!(
                    coercion_between(from, to, &table),
                    naive(from, to),
                    "gate index diverges from the plain scan for {from:?} -> {to:?}"
                );
            }
        }
    }

    #[test]
    fn is_close_on_aggregates_matches_the_tabled_coercions() {
        let table = TypeTable::new();
        let vec_t = Type::Primitive(Prim::Vec);
        let ratvec = Type::Primitive(Prim::RatVec);
        // Row-to-row entries derive from component coercions.
        assert_eq!(is_close(&Type::row(int()), &Type::row(rat()), &table), 0x5);
        assert_eq!(is_close(&Type::row(rat()), &Type::row(int()), &table), 0x6);
        assert_eq!(is_close(&Type::row(vec_t.clone()), &Type::row(Type::row(int())), &table), 0x7);
        assert_eq!(is_close(&Type::row(Type::row(int())), &Type::row(ratvec.clone()), &table), 0x5);
        // The tuple-into-primitive entry still applies from a tuple side.
        assert_eq!(
            is_close(&Type::tuple(vec![int(), int()]), &Type::Primitive(Prim::Split), &table),
            0x5
        );
        // Aggregates with no tabled path are simply not close.
        assert_eq!(is_close(&Type::row(int()), &Type::tuple(vec![int(), int()]), &table), 0);
        assert_eq!(
            is_close(&Type::function(int(), int()), &Type::function(int(), rat()), &table),
            0
        );
        assert_eq!(is_close(&Type::union_of(vec![int(), rat()]), &Type::union_of(vec![rat(), int()]), &table), 0);
    }

    #[test]
    fn broader_eq_orders_branch_types_for_balancing() {
        let table = TypeTable::new();
        assert!(broader_eq(&Type::void(), &int(), &table));
        assert!(!broader_eq(&int(), &Type::void(), &table));
        assert!(broader_eq(&rat(), &int(), &table));
        assert!(!broader_eq(&int(), &rat(), &table));
        assert!(broader_eq(&int(), &Type::Undetermined, &table));
        assert!(!broader_eq(&Type::Undetermined, &int(), &table));
        assert!(broader_eq(&Type::row(rat()), &Type::row(int()), &table));
        // Functions need equal argument types.
        assert!(broader_eq(
            &Type::function(int(), rat()),
            &Type::function(int(), int()),
            &table
        ));
        assert!(!broader_eq(
            &Type::function(rat(), rat()),
            &Type::function(int(), int()),
            &table
        ));
    }
}
