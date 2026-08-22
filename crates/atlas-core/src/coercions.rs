//! The axis coercion table and type-proximity predicates.
//!
//! Ports the upstream registrations (global.w:2526-2552 followed by
//! atlas-types.w:9137-9144) IN ORDER — `coercion_between` and
//! `row_coercion` are first-match linear scans, so a list display in `mat`
//! context must find `[vec]->mat` before any other row-into-mat entry —
//! plus `is_close` (axis-types.w:3246-3285; three bits: 0x1 = left
//! coerces to right, 0x2 = right to left, 0x4 = close) and `broader_eq`
//! (axis-types.w:3339-3364), the balancing order. Conversion NODES arrive
//! with the typed pipeline; this module only answers applicability.

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

/// The first registered coercion from `from` to `to`, if any.
pub fn coercion_between<'a>(from: &Type, to: &Type, table: &TypeTable) -> Option<&'a Coercion> {
    coercion_table()
        .iter()
        .find(|coercion| same(&coercion.from, from, table) && same(&coercion.to, to, table))
}

/// For a list display in non-row context `final_type`: the FIRST table
/// entry with a row `from` and that target, yielding the component type the
/// display's elements must take (`mat` context yields `vec`, never `[int]`).
pub fn row_coercion<'a>(final_type: &Type, table: &TypeTable) -> Option<(&'a Coercion, &'a Type)> {
    coercion_table().iter().find_map(|coercion| {
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
    let mut bits = 0;
    if coercion_between(x, y, table).is_some() {
        bits |= 0x1;
    }
    if coercion_between(y, x, table).is_some() {
        bits |= 0x2;
    }
    if bits != 0 {
        return bits | 0x4;
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
