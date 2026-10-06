//! The axis type model (language phase B stage 1).
//!
//! Ports upstream `type_expr` (axis-types.w:289-388): a tag plus payload,
//! with void as the empty tuple, length-1 tuples and unions unrepresentable
//! (constructors collapse them), variant/field names living in the typedef
//! table rather than the type, recursive types compared nominally, and
//! `specialise` as the only permitted mutation (most-general-unifier on
//! success, explicitly NOT commit-or-rollback — `can_specialise` exists for
//! callers that need rollback). Display matches the upstream spellings
//! byte for byte (axis-types.w:1610-1675).

use std::fmt;
use std::sync::Arc;

pub mod polymorphic;
mod recursive;

#[cfg(test)]
#[path = "types/revision_tests.rs"]
mod revision_tests;

#[cfg(test)]
thread_local! {
    // Per-thread deterministic work measurement, not a wall-clock assertion.
    static VALIDATION_VISITS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

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
    /// A linked type variable. Rigidity is determined by the enclosing
    /// scheme/assignment's fixed-variable threshold, not by this node.
    Variable(usize),
    Primitive(Prim),
    /// Argument and result; multi-argument functions carry a tuple argument.
    Function(Box<(Type, Type)>),
    /// `[component]`.
    Row(Box<Type>),
    Tuple(Vec<Type>),
    Union(Vec<Type>),
    /// A named typedef-table entry. Recursive entries compare nominally;
    /// distinct nonrecursive names may have compatible structural expansions.
    Tabled(TypeNumber),
    /// An application of a declared type constructor, retaining its name and
    /// arguments even when its structural expansion contains unused formals.
    Applied(TypeNumber, Vec<Type>),
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

    /// Inspect outer structure without losing the owning expression's name.
    /// Ordinary types borrow; constructor substitution owns only the expanded
    /// result. Children retain names, including recursive applications.
    pub fn expanded<'a>(&'a self, table: &'a TypeTable) -> std::borrow::Cow<'a, Type> {
        let mut current = self;
        // Well-formed entries have a structural top. The bound also makes a
        // malformed, directly cyclic placeholder inspectable without looping.
        for _ in 0..=table.bindings.len() {
            if matches!(current, Type::Applied(_, _)) {
                return std::borrow::Cow::Owned(polymorphic::expanded_top(current, table)
                    .unwrap_or_else(|_| current.clone()));
            }
            let Type::Tabled(number) = current else { break; };
            current = table.expansion(*number);
        }
        std::borrow::Cow::Borrowed(current)
    }

    /// Semantic equality, distinct from textual/table-slot equality and from
    /// compatibility with holes. Recursive names are a terminating boundary.
    pub fn equivalent(&self, other: &Type, table: &TypeTable) -> bool {
        // Named heads may expose a different shape. All other unequal heads
        // are an immediate negative result, including malformed descendants:
        // validation could only turn that same result into false again.
        match (self, other) {
            (Type::Tabled(_) | Type::Applied(..), _)
            | (_, Type::Tabled(_) | Type::Applied(..)) => {}
            (Type::Primitive(a), Type::Primitive(b)) => return a == b,
            (Type::Variable(a), Type::Variable(b)) => return a == b,
            (Type::Undetermined, Type::Undetermined) => return true,
            (Type::Row(_), Type::Row(_)) | (Type::Function(_), Type::Function(_)) => {}
            (Type::Tuple(a), Type::Tuple(b)) | (Type::Union(a), Type::Union(b))
                if a.len() == b.len() => {}
            _ => return false,
        }
        if table.validate_applications(self).is_err() || table.validate_applications(other).is_err() {
            return false;
        }
        self.equivalent_validated(other, table)
    }

    /// Both visible structural trees have already been validated. Descending
    /// through them must not rescan every remaining subtree. Expanding a named
    /// definition exposes NEW nodes and therefore re-enters `equivalent`.
    fn equivalent_validated(&self, other: &Type, table: &TypeTable) -> bool {
        match (self, other) {
            (Type::Tabled(a), Type::Tabled(b)) if a == b => true,
            (Type::Tabled(a), Type::Tabled(b))
                if table.is_recursive(*a) && table.is_recursive(*b) => false,
            (Type::Tabled(a), Type::Applied(b, args))
            | (Type::Applied(b, args), Type::Tabled(a)) if a == b && args.is_empty() => true,
            (Type::Tabled(a), Type::Applied(b, _))
            | (Type::Applied(b, _), Type::Tabled(a))
                if table.is_recursive(*a) && table.is_recursive(*b) => false,
            (Type::Tabled(a), b) => table.expansion(*a).equivalent(b, table),
            (a, Type::Tabled(b)) => a.equivalent(table.expansion(*b), table),
            (Type::Applied(a, xs), Type::Applied(b, ys)) if a == b =>
                xs.len() == ys.len() && xs.iter().zip(ys).all(|(x, y)| x.equivalent_validated(y, table)),
            (Type::Applied(a, _), Type::Applied(b, _))
                if table.is_recursive(*a) && table.is_recursive(*b) => false,
            (Type::Applied(..), b) => table.expand_application(self)
                .is_ok_and(|a| a.equivalent(b, table)),
            (a, Type::Applied(..)) => table.expand_application(other)
                .is_ok_and(|b| a.equivalent(&b, table)),
            (Type::Row(a), Type::Row(b)) => a.equivalent_validated(b, table),
            (Type::Function(a), Type::Function(b)) =>
                a.0.equivalent_validated(&b.0, table) && a.1.equivalent_validated(&b.1, table),
            (Type::Tuple(a), Type::Tuple(b)) | (Type::Union(a), Type::Union(b)) =>
                a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x.equivalent_validated(y, table)),
            _ => self == other,
        }
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
            (Type::Tabled(own), Type::Tabled(other)) if own == other => true,
            (Type::Tabled(own), Type::Tabled(other))
                if table.is_recursive(*own) && table.is_recursive(*other) => false,
            (Type::Applied(own, _), Type::Applied(other, _))
            | (Type::Applied(own, _), Type::Tabled(other))
            | (Type::Tabled(own), Type::Applied(other, _))
                if own != other && table.is_recursive(*own) && table.is_recursive(*other) => false,
            (Type::Applied(own, args), Type::Applied(other, patterns)) if own == other => {
                args.len() == patterns.len()
                    && args.iter_mut().zip(patterns).all(|(a, b)| a.specialise(b, table))
            }
            (Type::Applied(..), _) => {
                // Unlike a read-only compatibility query, successful
                // specialisation exposes the requested structural shape.
                // Preserve refinements to holes in constructor arguments.
                let Ok(mut expanded) = table.expand_application(self) else { return false; };
                if !expanded.specialise(pattern, table) { return false; }
                *self = expanded;
                true
            }
            (_, Type::Applied(..)) => table.expand_application(pattern)
                .is_ok_and(|expanded| self.specialise(&expanded, table)),
            (Type::Tabled(number), _) => {
                // axis-types.w:942-979: successful structural specialisation
                // must expose the receiver, not merely return compatibility.
                let mut expansion = table.expansion(*number).clone();
                if !expansion.specialise(pattern, table) { return false; }
                *self = expansion;
                true
            }
            (_, Type::Tabled(number)) => {
                let expansion = table.expansion(*number).clone();
                self.specialise(&expansion, table)
            }
            (Type::Primitive(own), Type::Primitive(other)) => own == other,
            (Type::Variable(own), Type::Variable(other)) => own == other,
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
            (Type::Tabled(own), Type::Tabled(other)) if own == other => true,
            (Type::Tabled(own), Type::Tabled(other))
                if table.is_recursive(*own) && table.is_recursive(*other) => false,
            (Type::Applied(own, _), Type::Applied(other, _))
            | (Type::Applied(own, _), Type::Tabled(other))
            | (Type::Tabled(own), Type::Applied(other, _))
                if own != other && table.is_recursive(*own) && table.is_recursive(*other) => false,
            (Type::Applied(own, args), Type::Applied(other, patterns)) if own == other => {
                args.len() == patterns.len()
                    && args.iter().zip(patterns).all(|(a, b)| a.can_specialise(b, table))
            }
            (Type::Applied(..), _) => table.expand_application(self)
                .is_ok_and(|expanded| expanded.can_specialise(pattern, table)),
            (_, Type::Applied(..)) => table.expand_application(pattern)
                .is_ok_and(|expanded| self.can_specialise(&expanded, table)),
            (Type::Tabled(number), _) => table.expansion(*number).can_specialise(pattern, table),
            (_, Type::Tabled(number)) => self.can_specialise(table.expansion(*number), table),
            (Type::Primitive(own), Type::Primitive(other)) => own == other,
            (Type::Variable(own), Type::Variable(other)) => own == other,
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

/// Immutable type identities plus live identifier bindings. Redefinition or
/// forgetting changes only the latter: existing values keep their old types.
#[derive(Clone, Default)]
pub struct TypeTable {
    bindings: Vec<TypeBinding>,
    active: std::collections::BTreeMap<String, TypeNumber>,
    /// Only new constructor entries occur here. Legacy tabled entries retain
    /// their nominal recursive-type behavior until the declaration layer moves.
    constructors: std::collections::BTreeMap<usize, (usize, bool)>,
    // An owned, nonsemantic snapshot identity. Clones share it until a
    // mutation; cached readers keep it alive, preventing address reuse.
    // Arc preserves TypeTable's Send+Sync auto traits.
    revision: Arc<()>,
}

impl PartialEq for TypeTable {
    fn eq(&self, other: &Self) -> bool {
        self.bindings == other.bindings && self.active == other.active
            && self.constructors == other.constructors
    }
}

impl Eq for TypeTable {}

impl fmt::Debug for TypeTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TypeTable").field("bindings", &self.bindings)
            .field("active", &self.active).field("constructors", &self.constructors).finish()
    }
}

impl TypeTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn revision(&self) -> &Arc<()> {
        &self.revision
    }

    fn changed(&mut self) {
        self.revision = Arc::new(());
    }

    pub fn add(&mut self, binding: TypeBinding) -> TypeNumber {
        let number = TypeNumber(self.bindings.len());
        if !binding.name.is_empty() {
            self.active.insert(binding.name.clone(), number);
        }
        self.bindings.push(binding);
        self.changed();
        number
    }

    /// Replace a placeholder binding with its resolved definition; used by
    /// the two-pass bracketed `set_type`, which registers every name of a
    /// group before resolving any right-hand side (recursion).
    pub fn update(&mut self, number: TypeNumber, definition: Type, fields: Vec<Option<String>>) {
        let binding = &mut self.bindings[number.0];
        binding.definition = definition;
        binding.fields = fields;
        self.changed();
    }

    pub fn binding(&self, number: TypeNumber) -> &TypeBinding {
        &self.bindings[number.0]
    }

    pub fn add_constructor(
        &mut self,
        binding: TypeBinding,
        arity: usize,
        recursive: bool,
    ) -> TypeNumber {
        let number = self.add(binding);
        self.constructors.insert(number.0, (arity, recursive));
        number
    }

    pub fn constructor_arity(&self, number: TypeNumber) -> usize {
        self.constructors.get(&number.0).map_or(0, |entry| entry.0)
    }

    /// Find field/tag metadata in ALL retained definitions, not just live
    /// names or the current projector overloads (axis-types.w:1454). A fresh
    /// formal constructor application is trialled for each binding, preserving
    /// the receiver's rigid floor and isolating the candidate's free variables.
    pub fn matching_bindings(&self, receiver: &polymorphic::InferredType)
        -> Result<Vec<TypeNumber>, polymorphic::TypeError>
    {
        let mut receiver = receiver.clone();
        receiver.wring_out()?;
        let mut matches = Vec::new();
        for (index, binding) in self.bindings.iter().enumerate() {
            if binding.fields.is_empty() { continue; }
            let number = TypeNumber(index);
            let arity = self.constructor_arity(number);
            let formal = if arity == 0 { Type::Tabled(number) }
                else { Type::Applied(number, (0..arity).map(Type::Variable).collect()) };
            if receiver.has_unifier(&formal, self)? { matches.push(number); }
        }
        Ok(matches)
    }

    pub fn is_recursive(&self, number: TypeNumber) -> bool {
        self.constructors.get(&number.0).is_none_or(|entry| entry.1)
    }

    /// Check applications without expanding definitions, so recursive names
    /// remain finite. Validate even equal applications and variable bindings:
    /// neither fast path is permission to accept a malformed constructor.
    pub fn validate_applications(&self, ty: &Type) -> Result<(), polymorphic::TypeError> {
        #[cfg(test)]
        VALIDATION_VISITS.with(|visits| visits.set(visits.get() + 1));
        match ty {
            Type::Tabled(number) => self.validate_constructor(*number, 0),
            Type::Applied(number, args) => {
                self.validate_constructor(*number, args.len())?;
                args.iter().try_for_each(|arg| self.validate_applications(arg))
            }
            Type::Row(component) => self.validate_applications(component),
            Type::Function(parts) => {
                self.validate_applications(&parts.0)?;
                self.validate_applications(&parts.1)
            }
            Type::Tuple(parts) | Type::Union(parts) =>
                parts.iter().try_for_each(|part| self.validate_applications(part)),
            Type::Primitive(_) | Type::Undetermined | Type::Variable(_) => Ok(()),
        }
    }

    fn validate_constructor(&self, number: TypeNumber, found: usize) -> Result<(), polymorphic::TypeError> {
        if self.bindings.get(number.0).is_none() {
            return Err(polymorphic::TypeError::UnknownConstructor(number.0));
        }
        let expected = self.constructor_arity(number);
        if found != expected {
            return Err(polymorphic::TypeError::Arity { expected, found });
        }
        Ok(())
    }

    /// One expansion only: recursive applications in the body stay references.
    pub fn expand_application(&self, applied: &Type) -> Result<Type, polymorphic::TypeError> {
        let (number, args) = match applied {
            Type::Tabled(number) => (*number, &[][..]),
            Type::Applied(number, args) => (*number, args.as_slice()),
            other => return Ok(other.clone()),
        };
        self.validate_applications(applied)?;
        let binding = &self.bindings[number.0];
        polymorphic::substitute_parameters(&binding.definition, args)
    }

    pub fn expansion(&self, number: TypeNumber) -> &Type {
        &self.bindings[number.0].definition
    }

    pub fn lookup(&self, name: &str) -> Option<TypeNumber> {
        self.active.get(name).copied()
    }

    /// Retain a simple type definition, including copied field metadata.
    /// Only the outer alias expands; names inside the body remain visible.
    pub fn add_simple(&mut self, binding: TypeBinding) -> TypeNumber {
        self.add_simple_constructor(binding, 0)
    }

    pub fn add_simple_constructor(&mut self, mut binding: TypeBinding, arity: usize) -> TypeNumber {
        if let Type::Tabled(number) | Type::Applied(number, _) = &binding.definition {
            binding.fields = self.binding(*number).fields.clone();
        }
        binding.definition = binding.definition.expanded(self).into_owned();
        if let Some(index) = self.bindings.iter().enumerate().position(|(index, old)|
            old.name == binding.name && old.definition == binding.definition
                && self.constructor_arity(TypeNumber(index)) == arity)
        {
            self.active.insert(binding.name.clone(), TypeNumber(index));
            self.bindings[index].fields = binding.fields;
            self.changed();
            return TypeNumber(index);
        }
        self.add_constructor(binding, arity, false)
    }

    /// Convenience for a simple definition without explicitly named fields.
    pub fn add_alias(&mut self, name: impl Into<String>, definition: Type) {
        self.add_simple(TypeBinding { name: name.into(), definition, fields: Vec::new() });
    }

    /// Resolve the active definition, not the oldest retained table entry.
    pub fn resolve_name(&self, name: &str) -> Option<Type> {
        self.lookup(name).map(Type::Tabled)
    }

    /// Remove an identifier without invalidating stored type references.
    pub fn forget(&mut self, name: &str) -> bool {
        let removed = self.active.remove(name).is_some();
        if removed { self.changed(); }
        removed
    }

    /// Classify a token without cloning its structural expansion.
    pub fn is_type_name(&self, name: &str) -> bool {
        self.active.contains_key(name)
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
        Type::Variable(number) => {
            // axis-types.w prints a single character starting at A (including
            // punctuation after Z), not spreadsheet-style AA, AB, ... .
            let code = u32::try_from(*number).ok()
                .and_then(|n| u32::from('A').checked_add(n))
                .and_then(char::from_u32).ok_or(fmt::Error)?;
            write!(out, "{code}")
        }
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
        Type::Tabled(number) | Type::Applied(number, _)
            if table.binding(*number).name.is_empty() => {
                let expanded = table.expand_application(type_).map_err(|_| fmt::Error)?;
                write_type(&expanded, table, out)
            }
        Type::Tabled(number) => write!(out, "{}", table.binding(*number).name),
        Type::Applied(number, args) => {
            write!(out, "{}<", table.binding(*number).name)?;
            for (i, arg) in args.iter().enumerate() {
                if i != 0 { write!(out, ",")?; }
                write_type(arg, table, out)?;
            }
            write!(out, ">")
        }
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
        Type::Tabled(number) | Type::Applied(number, _)
            if table.binding(*number).name.is_empty() => {
                let expanded = table.expand_application(type_).map_err(|_| fmt::Error)?;
                write_arrow_side(&expanded, table, out)
            }
        Type::Tuple(_) | Type::Union(_) => write_naked(type_, table, out),
        other => write_type(other, table, out),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_equivalence_checks_named_arguments_even_when_unused() {
        let mut table = TypeTable::new();
        let constant = table.add_constructor(TypeBinding {
            name: "Constant".into(), definition: Type::Primitive(Prim::Int), fields: vec![],
        }, 1, false);
        let bad_args = [Type::Tabled(TypeNumber(999)), Type::Applied(constant, vec![])];
        for bad in bad_args {
            let applied = Type::Applied(constant, vec![bad]);
            // Substitution would erase the bad unused argument; validation
            // must reject it before either identity or expansion can succeed.
            assert!(!applied.equivalent(&applied, &table));
            assert!(!applied.equivalent(&Type::Primitive(Prim::Int), &table));
            assert!(!Type::Primitive(Prim::Int).equivalent(&applied, &table));
            for outer in [Type::row(applied.clone()),
                          Type::function(Type::Primitive(Prim::Int), applied.clone()),
                          Type::tuple(vec![Type::Primitive(Prim::Int), applied.clone()]),
                          Type::union_of(vec![Type::Primitive(Prim::Int), applied])] {
                assert!(!outer.equivalent(&outer, &table));
            }
        }
    }

    #[test]
    fn type_equivalence_revalidates_newly_exposed_alias_bodies() {
        let mut table = TypeTable::new();
        let invalid = table.add_constructor(TypeBinding {
            name: "InvalidBody".into(),
            definition: Type::row(Type::Tabled(TypeNumber(999))), fields: vec![],
        }, 0, false);
        let named = Type::Tabled(invalid);
        // Preserve nominal identity; the invalid body is checked when an
        // expansion is actually requested, not by changing table semantics.
        assert!(named.equivalent(&named, &table));
        let malformed = table.expansion(invalid).clone();
        assert!(!named.equivalent(&malformed, &table));
        assert!(!malformed.equivalent(&named, &table));
        for structural in [Type::row(Type::Primitive(Prim::Int)), Type::Primitive(Prim::Int)] {
            assert!(!named.equivalent(&structural, &table));
            assert!(!structural.equivalent(&named, &table));
        }
    }

    #[test]
    fn type_equivalence_keeps_complete_structural_and_named_matrix() {
        let mut table = TypeTable::new();
        table.add_alias("Integer", Type::Primitive(Prim::Int));
        let integer = table.resolve_name("Integer").unwrap();
        let identity = table.add_constructor(TypeBinding {
            name: "Identity".into(), definition: Type::Variable(0), fields: vec![],
        }, 1, false);
        let values = [(Type::Primitive(Prim::Int), 0), (integer.clone(), 0),
            (Type::Applied(identity, vec![integer]), 0), (Type::Primitive(Prim::Rat), 1),
            (Type::Undetermined, 2), (Type::Variable(0), 3), (Type::Variable(1), 4),
            (Type::void(), 5)];
        let mut corpus = Vec::new();
        for (base, group) in values {
            corpus.push((base.clone(), (0, group)));
            corpus.push((Type::row(base.clone()), (1, group)));
            corpus.push((Type::function(base.clone(), Type::Primitive(Prim::Bool)), (2, group)));
            corpus.push((Type::tuple(vec![base.clone(), Type::Primitive(Prim::Bool)]), (3, group)));
            corpus.push((Type::union_of(vec![base, Type::Primitive(Prim::Bool)]), (4, group)));
        }
        for (left, expected_left) in &corpus {
            for (right, expected_right) in &corpus {
                assert_eq!(left.equivalent(right, &table), expected_left == expected_right,
                           "{left:?} versus {right:?}");
            }
        }
    }

    #[test]
    fn type_equivalence_validation_is_linear_in_structural_nodes() {
        let table = TypeTable::new();
        let depth = 128;
        let left = (0..depth).fold(Type::Primitive(Prim::Int), |t, _| Type::row(t));
        let right = left.clone();
        VALIDATION_VISITS.with(|visits| visits.set(0));
        assert!(left.equivalent(&right, &table));
        assert_eq!(VALIDATION_VISITS.with(|visits| visits.get()), 2 * (depth + 1),
                   "each input structural node must be validated exactly once");
        VALIDATION_VISITS.with(|visits| visits.set(0));
        assert!(!left.equivalent(&Type::Primitive(Prim::Int), &table));
        assert_eq!(VALIDATION_VISITS.with(|visits| visits.get()), 0,
                   "different structural heads cannot match; do not walk their children");
    }

    fn show(type_: &Type) -> String {
        type_.display(&TypeTable::new()).to_string()
    }

    #[test]
    fn simple_names_survive_redefinition_and_forget() {
        let mut table = TypeTable::new();
        table.add_alias("Saved", Type::row(Type::Primitive(Prim::Int)));
        let old = table.resolve_name("Saved").unwrap();
        assert!(matches!(old, Type::Tabled(_)));
        assert_eq!(old.display(&table).to_string(), "Saved");
        table.add_alias("Copy", old.clone());
        let copy = table.resolve_name("Copy").unwrap();
        table.add_alias("Saved", Type::row(Type::Primitive(Prim::String)));
        let new = table.resolve_name("Saved").unwrap();
        assert_ne!(old, new);
        assert_eq!(&*old.expanded(&table), &Type::row(Type::Primitive(Prim::Int)));
        assert!(copy.equivalent(&old, &table));
        assert!(!old.equivalent(&new, &table));
        assert!(!old.can_specialise(&new, &table));
        assert!(table.forget("Saved"));
        assert!(!table.is_type_name("Saved"));
        assert!(table.resolve_name("Saved").is_none());
        assert_eq!(old.display(&table).to_string(), "Saved");
    }

    #[test]
    fn structural_view_borrows_ordinary_types_and_substitutes_applications() {
        use std::borrow::Cow;
        let mut table = TypeTable::new();
        let row = table.add_simple(TypeBinding {
            name: "Ints".into(), definition: Type::row(Type::Primitive(Prim::Int)), fields: vec![],
        });
        assert!(matches!(Type::Tabled(row).expanded(&table), Cow::Borrowed(_)));
        let id = table.add_simple_constructor(TypeBinding {
            name: "Identity".into(), definition: Type::Variable(0), fields: vec![],
        }, 1);
        let nested = (0..5).fold(Type::Primitive(Prim::Int), |arg, _| Type::Applied(id, vec![arg]));
        assert_eq!(&*nested.expanded(&table), &Type::Primitive(Prim::Int));
        let different_arity = table.add_simple_constructor(TypeBinding {
            name: "Identity".into(), definition: Type::Variable(0), fields: vec![],
        }, 2);
        assert_ne!(id, different_arity);
        assert_eq!(table.constructor_arity(id), 1);
        assert_eq!(table.constructor_arity(different_arity), 2);
    }

    #[test]
    fn simple_aliases_copy_fields_and_preserve_nested_names() {
        let mut table = TypeTable::new();
        table.add_alias("Element", Type::Primitive(Prim::Int));
        let element = table.resolve_name("Element").unwrap();
        let record = table.add_simple(TypeBinding {
            name: "Record".into(),
            definition: Type::tuple(vec![element.clone(), Type::Primitive(Prim::String)]),
            fields: vec![Some("first".into()), Some("second".into())],
        });
        table.add_alias("Copy", Type::Tabled(record));
        let copied = table.lookup("Copy").unwrap();
        assert_eq!(table.binding(copied).fields, table.binding(record).fields);
        assert_eq!(table.expansion(copied).display(&table).to_string(), "(Element,string)");
        let mut copy = Type::Tabled(copied);
        assert!(copy.specialise(&Type::tuple(vec![Type::Undetermined, Type::Undetermined]), &table));
        assert_eq!(copy, Type::tuple(vec![element, Type::Primitive(Prim::String)]));
    }

    #[test]
    fn simple_equality_is_structural_but_does_not_treat_holes_as_equal() {
        let mut table = TypeTable::new();
        table.add_alias("Left", Type::row(Type::Primitive(Prim::Int)));
        table.add_alias("Right", Type::row(Type::Primitive(Prim::Int)));
        let mut left = table.resolve_name("Left").unwrap();
        let right = table.resolve_name("Right").unwrap();
        assert_ne!(left, right);
        assert!(left.equivalent(&right, &table));
        assert!(left.specialise(&right, &table));
        assert_eq!(left, Type::row(Type::Primitive(Prim::Int)));
        assert!(!Type::Undetermined.equivalent(&Type::Primitive(Prim::Int), &table));
    }

    #[test]
    fn semantic_equality_terminates_at_recursive_identity_and_validates_arity() {
        let mut table = TypeTable::new();
        let a = table.add_constructor(TypeBinding {
            name: "Loop".into(), definition: Type::Undetermined, fields: vec![],
        }, 0, true);
        table.update(a, Type::union_of(vec![Type::void(), Type::row(Type::Tabled(a))]), vec![]);
        let b = table.add_constructor(TypeBinding {
            name: "Other".into(), definition: table.expansion(a).clone(), fields: vec![],
        }, 0, true);
        assert!(Type::Tabled(a).equivalent(&Type::Applied(a, vec![]), &table));
        assert!(!Type::Tabled(a).equivalent(&Type::Applied(b, vec![]), &table));
        let invalid = Type::Applied(a, vec![Type::Primitive(Prim::Int)]);
        assert!(!invalid.equivalent(&invalid, &table));
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
