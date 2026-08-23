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

    /// Upstream `type_expr::operator==` (axis-types.w:807-825): structural
    /// equality, except that a tabled type equals its expansion (two tabled
    /// types are equal only when their type numbers are, which also keeps
    /// recursive types from expanding forever).
    pub fn equals(&self, other: &Type, table: &TypeTable) -> bool {
        match (self, other) {
            (Type::Tabled(own), Type::Tabled(another)) => own == another,
            (Type::Tabled(number), _) => table.expansion(*number).equals(other, table),
            (_, Type::Tabled(number)) => self.equals(table.expansion(*number), table),
            (Type::Undetermined, Type::Undetermined) => true,
            (Type::Primitive(own), Type::Primitive(another)) => own == another,
            (Type::Function(own), Type::Function(another)) => {
                own.0.equals(&another.0, table) && own.1.equals(&another.1, table)
            }
            (Type::Row(own), Type::Row(another)) => own.equals(another, table),
            (Type::Tuple(own), Type::Tuple(another))
            | (Type::Union(own), Type::Union(another)) => {
                own.len() == another.len()
                    && own
                        .iter()
                        .zip(another)
                        .all(|(component, other)| component.equals(other, table))
            }
            _ => false,
        }
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
    /// When a later `set_type` defines a structurally equivalent type, the
    /// new entry forwards to the earlier canonical number (upstream
    /// `type_expr::add_typedefs` reduces every equivalence class to one
    /// entry, so a re-included identical definition reuses the first
    /// number and old bindings keep matching, axis-types.w:1024-1051).
    pub merged_into: Option<TypeNumber>,
}

/// The typedef table (upstream `type_expr::type_map`). Bracketed
/// `set_type [ … ]` definitions live in `bindings`; the single-name form
/// is a plain alias that never enters the map (axis.w:5146-5168).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TypeTable {
    bindings: Vec<TypeBinding>,
    aliases: std::collections::BTreeMap<String, Type>,
}

impl TypeTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, binding: TypeBinding) -> TypeNumber {
        self.bindings.push(binding);
        TypeNumber(self.bindings.len() - 1)
    }

    /// Replace a placeholder binding with its resolved definition; used by
    /// the two-pass bracketed `set_type`, which registers every name of a
    /// group before resolving any right-hand side (recursion).
    pub fn update(&mut self, number: TypeNumber, definition: Type, fields: Vec<Option<String>>) {
        let binding = &mut self.bindings[number.0];
        binding.definition = definition;
        binding.fields = fields;
    }

    /// Mark `number` as merged into the canonical `target`, and give the
    /// canonical entry the newer field names (upstream
    /// `clean_out_type_identifier`, global.w:1176-1232: the names may
    /// differ even when the types coincide).
    pub fn merge_into(
        &mut self,
        number: TypeNumber,
        target: TypeNumber,
        fields: Vec<Option<String>>,
    ) {
        self.bindings[number.0].merged_into = Some(target);
        self.bindings[target.0].fields = fields;
    }

    /// The canonical number: follows the merge chain.
    pub fn canonical(&self, number: TypeNumber) -> TypeNumber {
        let mut current = number;
        while let Some(next) = self.bindings[current.0].merged_into {
            current = next;
        }
        current
    }

    /// Coinductive structural equivalence of two tabled types
    /// (axis-types.w:976-1000): expansions are compared component by
    /// component, and a pair already under comparison counts as equal, so
    /// recursive types (IntList) equate with their redefinitions.
    pub fn equivalent(&self, a: TypeNumber, b: TypeNumber) -> bool {
        fn go(
            table: &TypeTable,
            a: &Type,
            b: &Type,
            memo: &mut std::collections::HashSet<(usize, usize)>,
        ) -> bool {
            match (a, b) {
                (Type::Tabled(x), Type::Tabled(y)) => {
                    if x == y {
                        return true;
                    }
                    if !memo.insert((x.0, y.0)) {
                        return true;
                    }
                    let xa = table.expansion(*x).clone();
                    let yb = table.expansion(*y).clone();
                    go(table, &xa, &yb, memo)
                }
                (Type::Undetermined, Type::Undetermined) => true,
                (Type::Primitive(x), Type::Primitive(y)) => x == y,
                (Type::Function(x), Type::Function(y)) => {
                    go(table, &x.0, &y.0, memo) && go(table, &x.1, &y.1, memo)
                }
                (Type::Row(x), Type::Row(y)) => go(table, x, y, memo),
                (Type::Tuple(x), Type::Tuple(y)) | (Type::Union(x), Type::Union(y)) => {
                    x.len() == y.len()
                        && x.iter().zip(y).all(|(s, t)| go(table, s, t, memo))
                }
                _ => false,
            }
        }
        let mut memo = std::collections::HashSet::new();
        let ea = self.expansion(a).clone();
        let eb = self.expansion(b).clone();
        go(self, &ea, &eb, &mut memo)
    }

    /// Rewrite every stored reference to a merged number to its canonical
    /// number (group members resolved before their sibling was merged still
    /// point at the placeholder). Aliases are rewritten as well.
    pub fn canonicalise_references(&mut self) {
        let canonical: Vec<TypeNumber> = (0..self.bindings.len())
            .map(|index| self.canonical(TypeNumber(index)))
            .collect();
        fn rewrite(canonical: &[TypeNumber], type_: &mut Type) {
            match type_ {
                Type::Tabled(number) => *number = canonical[number.0],
                Type::Function(parts) => {
                    rewrite(canonical, &mut parts.0);
                    rewrite(canonical, &mut parts.1);
                }
                Type::Row(inner) => rewrite(canonical, inner),
                Type::Tuple(components) | Type::Union(components) => {
                    for component in components {
                        rewrite(canonical, component);
                    }
                }
                Type::Undetermined | Type::Primitive(_) => {}
            }
        }
        for binding in &mut self.bindings {
            rewrite(&canonical, &mut binding.definition);
        }
        for alias in self.aliases.values_mut() {
            rewrite(&canonical, alias);
        }
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
            .map(|number| self.canonical(number))
    }

    /// Register a single-name `set_type` alias; it stays out of the tabled
    /// map, so discrimination on it is rejected.
    pub fn add_alias(&mut self, name: impl Into<String>, definition: Type) {
        self.aliases.insert(name.into(), definition);
    }

    /// Resolve a type name written in a type expression: aliases first,
    /// then the tabled map (mirroring upstream, where a redefinition
    /// shadows outward).
    pub fn resolve_name(&self, name: &str) -> Option<Type> {
        self.aliases
            .get(name)
            .cloned()
            .or_else(|| self.lookup(name).map(Type::Tabled))
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
            merged_into: None,
        });
        let tabled = Type::Tabled(number);
        assert_eq!(tabled.display(&table).to_string(), "maybe_a_vec");
        let other = table.add(TypeBinding {
            name: "other".into(),
            definition: Type::union_of(vec![Type::void(), Type::Primitive(Prim::Vec)]),
            fields: Vec::new(),
            merged_into: None,
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
