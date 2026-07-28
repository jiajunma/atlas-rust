//! The axis type model (language phase B stage 1).
//!
//! Ports upstream `type_expr` (axis-types.w:289-388): a tag plus payload,
//! with void as the empty tuple, length-1 tuples and unions unrepresentable
//! (constructors collapse them), variant/field names living in the typedef
//! table rather than the type, tabled types equal by number, and
//! `specialise` as the only permitted mutation (most-general-unifier on
//! success, explicitly NOT commit-or-rollback — `can_specialise` exists for
//! callers that need rollback). Display matches the upstream spellings
//! byte for byte (axis-types.w:1610-1675).

use std::fmt;

/// All twenty upstream primitive types, in the upstream prim_names order
/// (axis-types.w:295-315). Every name is load-bearing from B1 on: the lexer
/// reserves them all positionally, even before a primitive's value layer
/// exists.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Prim {
    Int,
    Rat,
    String,
    Bool,
    Vec,
    Mat,
    RatVec,
    LieType,
    RootDatum,
    WeylElt,
    InnerClass,
    RealForm,
    CartanClass,
    KgbElt,
    Block,
    Split,
    KType,
    KTypePol,
    Param,
    ParamPol,
}

impl Prim {
    /// The upstream type name (axis-types.w prim_names order).
    pub fn name(self) -> &'static str {
        match self {
            Self::Int => "int",
            Self::Rat => "rat",
            Self::String => "string",
            Self::Bool => "bool",
            Self::Vec => "vec",
            Self::Mat => "mat",
            Self::RatVec => "ratvec",
            Self::LieType => "LieType",
            Self::RootDatum => "RootDatum",
            Self::WeylElt => "WeylElt",
            Self::InnerClass => "InnerClass",
            Self::RealForm => "RealForm",
            Self::CartanClass => "CartanClass",
            Self::KgbElt => "KGBElt",
            Self::Block => "Block",
            Self::Split => "Split",
            Self::KType => "KType",
            Self::KTypePol => "KTypePol",
            Self::Param => "Param",
            Self::ParamPol => "ParamPol",
        }
    }

    /// Every primitive, in upstream order — the lexer's PRIMTYPE list.
    pub const ALL: [Prim; 20] = [
        Self::Int,
        Self::Rat,
        Self::String,
        Self::Bool,
        Self::Vec,
        Self::Mat,
        Self::RatVec,
        Self::LieType,
        Self::RootDatum,
        Self::WeylElt,
        Self::InnerClass,
        Self::RealForm,
        Self::CartanClass,
        Self::KgbElt,
        Self::Block,
        Self::Split,
        Self::KType,
        Self::KTypePol,
        Self::Param,
        Self::ParamPol,
    ];
}

/// Index into the typedef table.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TypeNumber(pub(crate) usize);

/// A structural axis type. `Tuple(vec![])` is void; length-1 tuples and
/// unions never exist (use [`Type::tuple`] / [`Type::union_of`]).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Type {
    /// `*` — as-yet-undetermined, narrowed only by `specialise`.
    Undetermined,
    Primitive(Prim),
    /// Argument and result; multi-argument functions carry a tuple argument.
    Function(Box<(Type, Type)>),
    /// `[component]`.
    Row(Box<Type>),
    Tuple(Vec<Type>),
    Union(Vec<Type>),
    /// A typedef-table entry; equality is by number (the table
    /// canonicalises, so distinct numbers are distinct types).
    Tabled(TypeNumber),
}

impl Type {
    pub fn void() -> Self {
        Self::Tuple(Vec::new())
    }

    pub fn is_void(&self) -> bool {
        matches!(self, Self::Tuple(components) if components.is_empty())
    }

    /// Build a tuple, collapsing the length-1 case to its component.
    pub fn tuple(mut components: Vec<Type>) -> Self {
        if components.len() == 1 {
            components.pop().expect("length was checked")
        } else {
            Self::Tuple(components)
        }
    }

    /// Build a union, collapsing the length-1 case to its variant.
    pub fn union_of(mut variants: Vec<Type>) -> Self {
        if variants.len() == 1 {
            variants.pop().expect("length was checked")
        } else {
            Self::Union(variants)
        }
    }

    pub fn function(argument: Type, result: Type) -> Self {
        Self::Function(Box::new((argument, result)))
    }

    pub fn row(component: Type) -> Self {
        Self::Row(Box::new(component))
    }

    /// Specialise `self` toward `pattern`, mutating only by narrowing `*`
    /// holes; returns whether the two are compatible. On failure `self` may
    /// already be partially specialised (upstream semantics) — use
    /// [`Type::can_specialise`] first when rollback matters.
    pub fn specialise(&mut self, pattern: &Type, table: &TypeTable) -> bool {
        match (&mut *self, pattern) {
            (_, Type::Undetermined) => true,
            (Type::Undetermined, _) => {
                *self = pattern.clone();
                true
            }
            (Type::Tabled(own), Type::Tabled(other)) => own == other,
            (Type::Tabled(number), _) => {
                // Table types contain no holes, so this is a pure check.
                let expansion = table.expansion(*number).clone();
                expansion.can_specialise(pattern, table)
            }
            (_, Type::Tabled(number)) => {
                let expansion = table.expansion(*number).clone();
                self.specialise(&expansion, table)
            }
            (Type::Primitive(own), Type::Primitive(other)) => own == other,
            (Type::Function(own), Type::Function(other)) => {
                own.0.specialise(&other.0, table) && own.1.specialise(&other.1, table)
            }
            (Type::Row(own), Type::Row(other)) => own.specialise(other, table),
            (Type::Tuple(own), Type::Tuple(other)) | (Type::Union(own), Type::Union(other)) => {
                own.len() == other.len()
                    && own
                        .iter_mut()
                        .zip(other)
                        .all(|(component, pattern)| component.specialise(pattern, table))
            }
            _ => false,
        }
    }

    /// Whether `specialise` would succeed, without mutating.
    pub fn can_specialise(&self, pattern: &Type, table: &TypeTable) -> bool {
        match (self, pattern) {
            (_, Type::Undetermined) | (Type::Undetermined, _) => true,
            (Type::Tabled(own), Type::Tabled(other)) => own == other,
            (Type::Tabled(number), _) => table.expansion(*number).can_specialise(pattern, table),
            (_, Type::Tabled(number)) => self.can_specialise(table.expansion(*number), table),
            (Type::Primitive(own), Type::Primitive(other)) => own == other,
            (Type::Function(own), Type::Function(other)) => {
                own.0.can_specialise(&other.0, table) && own.1.can_specialise(&other.1, table)
            }
            (Type::Row(own), Type::Row(other)) => own.can_specialise(other, table),
            (Type::Tuple(own), Type::Tuple(other)) | (Type::Union(own), Type::Union(other)) => {
                own.len() == other.len()
                    && own
                        .iter()
                        .zip(other)
                        .all(|(component, pattern)| component.can_specialise(pattern, table))
            }
            _ => false,
        }
    }

    /// Display with the typedef table so tabled types print their names.
    pub fn display<'a>(&'a self, table: &'a TypeTable) -> TypeDisplay<'a> {
        TypeDisplay { type_: self, table }
    }
}

/// One typedef-table entry: variant/field names live here, never in `Type`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypeBinding {
    pub name: String,
    pub definition: Type,
    /// Field (tuple) or injector (union) names, positionally; `None` for
    /// anonymous components. Empty when the definition has none.
    pub fields: Vec<Option<String>>,
}

/// The typedef table (upstream `type_expr::type_map`).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TypeTable {
    bindings: Vec<TypeBinding>,
}

impl TypeTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, binding: TypeBinding) -> TypeNumber {
        self.bindings.push(binding);
        TypeNumber(self.bindings.len() - 1)
    }

    pub fn binding(&self, number: TypeNumber) -> &TypeBinding {
        &self.bindings[number.0]
    }

    pub fn expansion(&self, number: TypeNumber) -> &Type {
        &self.bindings[number.0].definition
    }

    pub fn lookup(&self, name: &str) -> Option<TypeNumber> {
        self.bindings
            .iter()
            .position(|binding| binding.name == name)
            .map(TypeNumber)
    }
}

pub struct TypeDisplay<'a> {
    type_: &'a Type,
    table: &'a TypeTable,
}

impl fmt::Display for TypeDisplay<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_type(self.type_, self.table, formatter)
    }
}

/// Top-level printing: `void` for the empty tuple, otherwise as a
/// parenthesised list; upstream axis-types.w:1610-1675.
fn write_type(type_: &Type, table: &TypeTable, out: &mut fmt::Formatter<'_>) -> fmt::Result {
    match type_ {
        Type::Undetermined => write!(out, "*"),
        Type::Primitive(prim) => write!(out, "{}", prim.name()),
        Type::Row(component) => {
            write!(out, "[")?;
            write_type(component, table, out)?;
            write!(out, "]")
        }
        Type::Tuple(components) if components.is_empty() => write!(out, "void"),
        Type::Tuple(_) | Type::Union(_) | Type::Function(_) => {
            write!(out, "(")?;
            write_naked(type_, table, out)?;
            write!(out, ")")
        }
        Type::Tabled(number) => write!(out, "{}", table.binding(*number).name),
    }
}

/// Inside parentheses tuples, unions, and function arrows print WITHOUT
/// their own parens, and a void side of an arrow prints as nothing:
/// `(int,int->int)`, `(int|string)`, `(->)`.
fn write_naked(type_: &Type, table: &TypeTable, out: &mut fmt::Formatter<'_>) -> fmt::Result {
    match type_ {
        Type::Function(parts) => {
            let (argument, result) = &**parts;
            if !argument.is_void() {
                write_arrow_side(argument, table, out)?;
            }
            write!(out, "->")?;
            if !result.is_void() {
                write_arrow_side(result, table, out)?;
            }
            Ok(())
        }
        Type::Tuple(components) => {
            for (index, component) in components.iter().enumerate() {
                if index > 0 {
                    write!(out, ",")?;
                }
                write_type(component, table, out)?;
            }
            Ok(())
        }
        Type::Union(variants) => {
            for (index, variant) in variants.iter().enumerate() {
                if index > 0 {
                    write!(out, "|")?;
                }
                write_type(variant, table, out)?;
            }
            Ok(())
        }
        other => write_type(other, table, out),
    }
}

/// One side of a function arrow: tuple and union LISTS print naked, but a
/// nested function type keeps its own parentheses.
fn write_arrow_side(type_: &Type, table: &TypeTable, out: &mut fmt::Formatter<'_>) -> fmt::Result {
    match type_ {
        Type::Tuple(_) | Type::Union(_) => write_naked(type_, table, out),
        other => write_type(other, table, out),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn show(type_: &Type) -> String {
        type_.display(&TypeTable::new()).to_string()
    }

    #[test]
    fn prints_the_upstream_spellings() {
        assert_eq!(show(&Type::Primitive(Prim::Int)), "int");
        assert_eq!(show(&Type::Undetermined), "*");
        assert_eq!(show(&Type::void()), "void");
        assert_eq!(show(&Type::row(Type::Primitive(Prim::Vec))), "[vec]");
        assert_eq!(
            show(&Type::tuple(vec![
                Type::Primitive(Prim::Int),
                Type::Primitive(Prim::Rat),
            ])),
            "(int,rat)"
        );
        assert_eq!(
            show(&Type::union_of(vec![
                Type::Primitive(Prim::Int),
                Type::Primitive(Prim::String),
            ])),
            "(int|string)"
        );
        assert_eq!(
            show(&Type::function(
                Type::tuple(vec![Type::Primitive(Prim::Int), Type::Primitive(Prim::Int)]),
                Type::Primitive(Prim::Int),
            )),
            "(int,int->int)"
        );
        assert_eq!(show(&Type::function(Type::void(), Type::void())), "(->)");
        assert_eq!(
            show(&Type::function(
                Type::row(Type::Primitive(Prim::Int)),
                Type::void(),
            )),
            "([int]->)"
        );
        assert_eq!(
            show(&Type::function(
                Type::function(Type::Primitive(Prim::Int), Type::Primitive(Prim::Bool)),
                Type::Primitive(Prim::Bool),
            )),
            "((int->bool)->bool)"
        );
    }

    #[test]
    fn tabled_types_print_their_names_and_compare_by_number() {
        let mut table = TypeTable::new();
        let number = table.add(TypeBinding {
            name: "maybe_a_vec".into(),
            definition: Type::union_of(vec![Type::void(), Type::Primitive(Prim::Vec)]),
            fields: vec![Some("no_vec".into()), Some("solution".into())],
        });
        let tabled = Type::Tabled(number);
        assert_eq!(tabled.display(&table).to_string(), "maybe_a_vec");
        let other = table.add(TypeBinding {
            name: "other".into(),
            definition: Type::union_of(vec![Type::void(), Type::Primitive(Prim::Vec)]),
            fields: Vec::new(),
        });
        assert_ne!(Type::Tabled(number), Type::Tabled(other));
        // Tabled-vs-structural comparison expands the definition.
        let structural = Type::union_of(vec![Type::void(), Type::Primitive(Prim::Vec)]);
        assert!(tabled.can_specialise(&structural, &table));
    }

    #[test]
    fn length_one_tuples_and_unions_collapse() {
        assert_eq!(
            Type::tuple(vec![Type::Primitive(Prim::Int)]),
            Type::Primitive(Prim::Int)
        );
        assert_eq!(
            Type::union_of(vec![Type::Primitive(Prim::Int)]),
            Type::Primitive(Prim::Int)
        );
    }

    #[test]
    fn specialise_narrows_holes_to_a_most_general_unifier() {
        let table = TypeTable::new();
        let mut own = Type::tuple(vec![Type::Primitive(Prim::Int), Type::Undetermined]);
        let pattern = Type::tuple(vec![Type::Undetermined, Type::Primitive(Prim::Rat)]);
        assert!(own.specialise(&pattern, &table));
        assert_eq!(
            own,
            Type::tuple(vec![Type::Primitive(Prim::Int), Type::Primitive(Prim::Rat)])
        );
        // Incompatible tags fail.
        let mut row = Type::row(Type::Primitive(Prim::Int));
        assert!(!row.specialise(&Type::Primitive(Prim::Vec), &table));
        // A function pattern narrows both sides.
        let mut function = Type::function(Type::Undetermined, Type::Undetermined);
        assert!(function.specialise(
            &Type::function(Type::Primitive(Prim::Int), Type::Undetermined),
            &table
        ));
        assert_eq!(
            function,
            Type::function(Type::Primitive(Prim::Int), Type::Undetermined)
        );
    }
}
