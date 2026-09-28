//! Owned second-order type machinery, following original axis-types.w:2264+.
//!
//! A scheme never contains independent `Undetermined` holes: wrapping assigns
//! a fresh variable to each hole, but preserves repeated explicit variables.
//! Substitutions belong to one analysis/overload trial, not to global bindings.
//! The language parser and evaluator are migrated separately; these primitives
//! alone do not establish support for `any_type` or latest basic.at.

use super::{Type, TypeTable};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypeError {
    VariableOutOfRange(usize),
    UnknownConstructor(usize),
    Arity { expected: usize, found: usize },
    UndeterminedInAssignment,
    PendingAssignments,
    ExpectedFunction,
    RecursiveExpansion,
    ScopeCapture,
    IndexOverflow,
}

/// Rebuild a type, transforming variables and independent holes, without
/// expanding named constructors (only their arguments are traversed).
fn transform(
    t: &Type,
    variable: &mut impl FnMut(Option<usize>) -> Result<Type, TypeError>,
) -> Result<Type, TypeError> {
    Ok(match t {
        Type::Undetermined => return variable(None),
        Type::Variable(n) => return variable(Some(*n)),
        Type::Primitive(_) | Type::Tabled(_) => t.clone(),
        Type::Row(c) => Type::row(transform(c, variable)?),
        Type::Function(f) => Type::function(transform(&f.0, variable)?, transform(&f.1, variable)?),
        Type::Tuple(cs) => Type::tuple(cs.iter().map(|c| transform(c, variable)).collect::<Result<_, _>>()?),
        Type::Union(cs) => Type::union_of(cs.iter().map(|c| transform(c, variable)).collect::<Result<_, _>>()?),
        Type::Applied(n, args) => Type::Applied(*n, args.iter().map(|a| transform(a, variable)).collect::<Result<_, _>>()?),
    })
}

/// Substitute constructor formals simultaneously. Variables inside supplied
/// arguments are NOT substituted again, preventing accidental capture.
pub fn substitute_parameters(t: &Type, args: &[Type]) -> Result<Type, TypeError> {
    transform(t, &mut |v| match v {
        None => Err(TypeError::UndeterminedInAssignment),
        Some(n) => args.get(n).cloned().ok_or(TypeError::VariableOutOfRange(n)),
    })
}

/// Shift only variables above the fixed threshold (axis-types.w::shift).
pub fn shift(t: &Type, fixed: usize, amount: usize) -> Result<Type, TypeError> {
    transform(t, &mut |v| match v {
        None => Err(TypeError::UndeterminedInAssignment),
        Some(n) if n < fixed => Ok(Type::Variable(n)),
        Some(n) => Ok(Type::Variable(n.checked_add(amount).ok_or(TypeError::IndexOverflow)?)),
    })
}

fn named(t: &Type) -> Option<super::TypeNumber> {
    match t { Type::Tabled(n) | Type::Applied(n, _) => Some(*n), _ => None }
}

/// Expose only the top constructor chain, not recursive children. A repeated
/// complete application is an invalid unguarded cycle. Do not reject repeated
/// constructor IDs alone: Identity<Identity<int>> is a finite valid chain.
pub(super) fn expanded_top(t: &Type, table: &TypeTable) -> Result<Type, TypeError> {
    let mut result = t.clone();
    let mut seen = Vec::new();
    while named(&result).is_some() {
        if seen.contains(&result) { return Err(TypeError::RecursiveExpansion); }
        seen.push(result.clone());
        result = table.expand_application(&result)?;
    }
    Ok(result)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypeScheme {
    body: Type,
    fixed: usize,
    degree: usize,
}

impl TypeScheme {
    /// Pack in traversal/first-occurrence order. Distinct holes stay distinct;
    /// repeated variable numbers retain the same substitution slot.
    pub fn wrap(t: &Type, fixed: usize) -> Result<Self, TypeError> {
        let mut translated = Vec::new();
        let body = transform(t, &mut |v| {
            if let Some(n) = v {
                if n < fixed { return Ok(Type::Variable(n)); }
                if let Some(i) = translated.iter().position(|old| *old == Some(n)) {
                    return Ok(Type::Variable(fixed.checked_add(i).ok_or(TypeError::IndexOverflow)?));
                }
            }
            let n = fixed.checked_add(translated.len()).ok_or(TypeError::IndexOverflow)?;
            translated.push(v);
            Ok(Type::Variable(n))
        })?;
        fixed.checked_add(translated.len()).ok_or(TypeError::IndexOverflow)?;
        Ok(Self { body, fixed, degree: translated.len() })
    }

    /// Unlike wrapping a value, a constructor retains its DECLARED arity and
    /// parameter numbering, including unused parameters (duplicate-formal probe).
    pub fn constructor(body: Type, degree: usize) -> Result<Self, TypeError> {
        transform(&body, &mut |v| match v {
            Some(n) if n < degree => Ok(Type::Variable(n)),
            Some(n) => Err(TypeError::VariableOutOfRange(n)),
            None => Err(TypeError::UndeterminedInAssignment),
        })?;
        Ok(Self { body, fixed: 0, degree })
    }

    pub fn body(&self) -> &Type { &self.body }
    pub fn fixed(&self) -> usize { self.fixed }
    pub fn degree(&self) -> usize { self.degree }
    pub fn is_polymorphic(&self) -> bool { self.degree != 0 }
}

/// Acyclic substitutions for variables in [fixed, fixed + degree). Variables
/// below fixed are rigid. Matches original type_assignment's partial mutation
/// on failed unification; use `try_unify` for a rollback-capable trial.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypeAssignment {
    fixed: usize,
    equivalents: Vec<Option<Type>>,
}

impl TypeAssignment {
    pub fn new(fixed: usize, degree: usize) -> Result<Self, TypeError> {
        fixed.checked_add(degree).ok_or(TypeError::IndexOverflow)?;
        Ok(Self { fixed, equivalents: vec![None; degree] })
    }

    pub fn fixed(&self) -> usize { self.fixed }
    pub fn degree(&self) -> usize { self.equivalents.len() }

    fn grow(&mut self, count: usize) -> Result<(), TypeError> {
        let degree = self.degree().checked_add(count).ok_or(TypeError::IndexOverflow)?;
        self.fixed.checked_add(degree).ok_or(TypeError::IndexOverflow)?;
        self.equivalents.resize(degree, None);
        Ok(())
    }

    /// Import another assignment into a disjoint variable range, including
    /// substitutions already pending there (axis-types.w::append). Merely
    /// reserving new holes would silently discard its inferred constraints.
    pub fn append(&mut self, other: &Self) -> Result<usize, TypeError> {
        if self.fixed < other.fixed { return Err(TypeError::ScopeCapture); }
        let start = self.fixed.checked_add(self.degree()).ok_or(TypeError::IndexOverflow)?;
        let degree = self.degree().checked_add(other.degree()).ok_or(TypeError::IndexOverflow)?;
        self.fixed.checked_add(degree).ok_or(TypeError::IndexOverflow)?;
        let diff = start - other.fixed;
        let shifted = other.equivalents.iter().map(|value| {
            value.as_ref().map(|t| shift(t, other.fixed, diff)).transpose()
        }).collect::<Result<Vec<_>, _>>()?;
        self.equivalents.extend(shifted);
        Ok(diff)
    }

    /// Import a fresh use of a scheme, leaving its rigid variables alone.
    pub fn instantiate(&mut self, scheme: &TypeScheme) -> Result<Type, TypeError> {
        if self.fixed < scheme.fixed { return Err(TypeError::ScopeCapture); }
        let start = self.fixed.checked_add(self.degree()).ok_or(TypeError::IndexOverflow)?;
        let degree = self.degree().checked_add(scheme.degree).ok_or(TypeError::IndexOverflow)?;
        self.fixed.checked_add(degree).ok_or(TypeError::IndexOverflow)?;
        let body = shift(&scheme.body, scheme.fixed, start - scheme.fixed)?;
        self.equivalents.resize(degree, None);
        Ok(body)
    }

    /// Leave an abstraction: its newest rigid variables become polymorphic.
    pub fn lower_floor(&mut self, count: usize) -> Result<(), TypeError> {
        let fixed = self.fixed.checked_sub(count).ok_or(TypeError::ScopeCapture)?;
        let size = self.degree().checked_add(count).ok_or(TypeError::IndexOverflow)?;
        self.equivalents.resize(size, None);
        self.equivalents.rotate_right(count);
        self.fixed = fixed;
        Ok(())
    }

    fn validate(&self, t: &Type) -> Result<(), TypeError> {
        transform(t, &mut |v| match v {
            None => Err(TypeError::UndeterminedInAssignment),
            Some(n) if n < self.fixed + self.degree() => Ok(Type::Variable(n)),
            Some(n) => Err(TypeError::VariableOutOfRange(n)),
        })?;
        Ok(())
    }

    fn equivalent(&self, n: usize) -> Option<&Type> {
        n.checked_sub(self.fixed).and_then(|i| self.equivalents.get(i)).and_then(Option::as_ref)
    }

    fn occurs(&self, n: usize, t: &Type) -> bool {
        match t {
            Type::Variable(v) => self.equivalent(*v).map_or(*v == n, |t| self.occurs(n, t)),
            Type::Row(c) => self.occurs(n, c),
            Type::Function(f) => self.occurs(n, &f.0) || self.occurs(n, &f.1),
            Type::Tuple(cs) | Type::Union(cs) | Type::Applied(_, cs) => cs.iter().any(|c| self.occurs(n, c)),
            Type::Primitive(_) | Type::Tabled(_) | Type::Undetermined => false,
        }
    }

    fn assign(&mut self, n: usize, t: &Type) -> bool {
        if self.occurs(n, t) { return false; }
        self.equivalents[n - self.fixed] = Some(t.clone());
        true
    }

    pub fn unify(&mut self, p: &Type, q: &Type, table: &TypeTable) -> Result<bool, TypeError> {
        self.validate(p)?;
        self.validate(q)?;
        table.validate_applications(p)?;
        table.validate_applications(q)?;
        self.unify_inner(p, q, table)
    }

    pub fn try_unify(&mut self, p: &Type, q: &Type, table: &TypeTable) -> Result<bool, TypeError> {
        let saved = self.clone();
        let result = self.unify(p, q, table);
        if result != Ok(true) { *self = saved; }
        result
    }

    fn unify_inner(&mut self, p: &Type, q: &Type, table: &TypeTable) -> Result<bool, TypeError> {
        if let Type::Variable(n) = p {
            if q == p { return Ok(true); }
            if *n >= self.fixed {
                return match self.equivalent(*n).cloned() {
                    Some(t) => self.unify_inner(&t, q, table),
                    None => Ok(self.assign(*n, q)),
                };
            }
        }
        if let Type::Variable(n) = q {
            if *n >= self.fixed {
                return match self.equivalent(*n).cloned() {
                    Some(t) => self.unify_inner(p, &t, table),
                    None => Ok(self.assign(*n, p)),
                };
            }
        }
        // Handle variables BEFORE named types, retaining the constructor name
        // when an unassigned variable can bind to the unexpanded application.
        let named = |t: &Type| match t {
            Type::Tabled(n) | Type::Applied(n, _) => Some(*n),
            _ => None,
        };
        if let (Some(a), Some(b)) = (named(p), named(q)) {
            if a == b {
                let args = |t: &Type| match t {
                    Type::Applied(_, xs) => xs.clone(),
                    _ => Vec::new(),
                };
                return self.unify_lists(&args(p), &args(q), table);
            }
            if table.is_recursive(a) && table.is_recursive(b) { return Ok(false); }
            return self.unify_inner(&table.expand_application(p)?, &table.expand_application(q)?, table);
        }
        if named(p).is_some() { return self.unify_inner(&table.expand_application(p)?, q, table); }
        if named(q).is_some() { return self.unify_inner(p, &table.expand_application(q)?, table); }
        Ok(match (p, q) {
            (Type::Primitive(a), Type::Primitive(b)) => a == b,
            (Type::Variable(a), Type::Variable(b)) => a == b,
            (Type::Row(a), Type::Row(b)) => self.unify_inner(a, b, table)?,
            (Type::Function(a), Type::Function(b)) => self.unify_inner(&a.0, &b.0, table)? && self.unify_inner(&a.1, &b.1, table)?,
            (Type::Tuple(a), Type::Tuple(b)) | (Type::Union(a), Type::Union(b)) => self.unify_lists(a, b, table)?,
            _ => false,
        })
    }

    fn unify_lists(&mut self, a: &[Type], b: &[Type], table: &TypeTable) -> Result<bool, TypeError> {
        if a.len() != b.len() { return Ok(false); }
        for (x, y) in a.iter().zip(b) {
            if !self.unify_inner(x, y, table)? { return Ok(false); }
        }
        Ok(true)
    }

    /// Apply all substitutions, compacting remaining polymorphic indices while
    /// preserving rigid ones. The returned type belongs to the compacted scope;
    /// it must not be fed back into this old assignment without re-wrapping.
    pub fn substitution(&self, t: &Type) -> Result<Type, TypeError> {
        self.validate(t)?;
        transform(t, &mut |v| {
            let n = v.ok_or(TypeError::UndeterminedInAssignment)?;
            if let Some(t) = self.equivalent(n) { return self.substitution(t); }
            let n = if n < self.fixed { n } else {
                self.fixed + self.equivalents[..n - self.fixed].iter().filter(|t| t.is_none()).count()
            };
            Ok(Type::Variable(n))
        })
    }
}

/// A type expression together with the scope that gives its variable numbers
/// meaning. Pending substitutions stay owned by this inference/overload trial.
/// Following original `type` in axis-types.w:2990+, this is the bridge needed
/// before replacing bare Type cells in the active analyzer; it is not that
/// integration and does not by itself make generic Atlas programs supported.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InferredType {
    body: Type,
    assignment: TypeAssignment,
}

impl InferredType {
    pub fn from_scheme(scheme: TypeScheme) -> Result<Self, TypeError> {
        Ok(Self {
            assignment: TypeAssignment::new(scheme.fixed, scheme.degree)?,
            body: scheme.body,
        })
    }

    pub fn wrap(body: &Type, fixed: usize) -> Result<Self, TypeError> {
        Self::from_scheme(TypeScheme::wrap(body, fixed)?)
    }

    pub fn bottom(fixed: usize) -> Result<Self, TypeError> {
        Self::wrap(&Type::Undetermined, fixed)
    }

    /// Component scopes are independent. Bake and re-pack each with its OWN
    /// threshold before importing into the tuple's current scope.
    pub fn wrap_tuple(components: Vec<Self>, fixed: usize) -> Result<Self, TypeError> {
        let mut assignment = TypeAssignment::new(fixed, 0)?;
        let mut bodies = Vec::with_capacity(components.len());
        for component in components {
            let scheme = TypeScheme::wrap(&component.bake()?, component.fixed())?;
            bodies.push(assignment.instantiate(&scheme)?);
        }
        Ok(Self { body: Type::tuple(bodies), assignment })
    }

    /// Raw body access does not substitute pending assignments. Keep its scope
    /// attached; use bake only at an explicit scope-transfer boundary.
    pub fn body(&self) -> &Type { &self.body }
    pub fn assignments(&self) -> &TypeAssignment { &self.assignment }
    pub fn fixed(&self) -> usize { self.assignment.fixed() }
    pub fn degree(&self) -> usize { self.assignment.degree() }
    pub fn is_clean(&self) -> bool { self.assignment.equivalents.iter().all(Option::is_none) }
    pub fn is_polymorphic(&self) -> bool { self.assignment.equivalents.iter().any(Option::is_none) }

    pub fn bake(&self) -> Result<Type, TypeError> {
        self.assignment.substitution(&self.body)
    }

    /// Bake/re-pack only when there are assignments, preserving declared but
    /// unused constructor parameters when the type is already clean.
    pub fn wring_out(&mut self) -> Result<(), TypeError> {
        if !self.is_clean() { *self = Self::wrap(&self.bake()?, self.fixed())?; }
        Ok(())
    }

    pub fn raise_floor(&mut self, count: usize) -> Result<(), TypeError> {
        let mut shifted = self.clone();
        shifted.wring_out()?;
        let fixed = shifted.fixed().checked_add(count).ok_or(TypeError::IndexOverflow)?;
        let body = shift(&shifted.body, shifted.fixed(), count)?;
        shifted.assignment = TypeAssignment::new(fixed, shifted.degree())?;
        shifted.body = body;
        *self = shifted;
        Ok(())
    }

    pub fn lower_floor(&mut self, count: usize) -> Result<(), TypeError> {
        self.assignment.lower_floor(count)
    }

    /// Forget trial assignments and restore its original number of slots.
    /// Reject restoring a range too short for the unchanged body.
    pub fn clear(&mut self, degree: usize) -> Result<(), TypeError> {
        let assignment = TypeAssignment::new(self.fixed(), degree)?;
        assignment.validate(&self.body)?;
        self.assignment = assignment;
        Ok(())
    }

    /// Original unify_to retains partial substitutions on a failed match and
    /// never mutates the other type. Use try_unify_to for a transactional trial.
    pub fn unify_to(&mut self, other: &Self, table: &TypeTable) -> Result<bool, TypeError> {
        if self.fixed() < other.fixed() { self.raise_floor(other.fixed() - self.fixed())?; }
        let body = if other.is_polymorphic() {
            let diff = self.assignment.append(&other.assignment)?;
            shift(&other.body, other.fixed(), diff)?
        } else {
            other.bake()?
        };
        self.assignment.unify(&self.body, &body, table)
    }

    pub fn try_unify_to(&mut self, other: &Self, table: &TypeTable) -> Result<bool, TypeError> {
        let saved = self.clone();
        let result = self.unify_to(other, table);
        if result != Ok(true) { *self = saved; }
        result
    }

    /// Two-sided unification follows the original assignment-owner choice.
    /// Work on owned snapshots so failure also restores remapped bodies/scopes,
    /// not just the slots assigned during the final recursive comparison.
    pub fn unify(&mut self, other: &mut Self, table: &TypeTable) -> Result<bool, TypeError> {
        let (mut left, mut right) = (self.clone(), other.clone());
        let left_poly = left.is_polymorphic();
        let right_poly = right.is_polymorphic();
        if !left_poly { left.wring_out()?; }
        if !right_poly { right.wring_out()?; }
        let owner_left = if left_poly && right_poly {
            left.fixed() >= right.fixed()
        } else {
            left_poly || !right_poly
        };
        // Fixed variables from a later context must never be captured by an
        // earlier owner's free range, including when the other type is rigid.
        if owner_left && left.fixed() < right.fixed() {
            left.raise_floor(right.fixed() - left.fixed())?;
        } else if !owner_left && right.fixed() < left.fixed() {
            right.raise_floor(left.fixed() - right.fixed())?;
        }
        if left_poly && right_poly {
            if owner_left {
                let diff = left.assignment.append(&right.assignment)?;
                right.body = shift(&right.body, right.fixed(), diff)?;
            } else {
                let diff = right.assignment.append(&left.assignment)?;
                left.body = shift(&left.body, left.fixed(), diff)?;
            }
        }
        let matched = if owner_left {
            left.assignment.unify(&left.body, &right.body, table)?
        } else {
            right.assignment.unify(&left.body, &right.body, table)?
        };
        if !matched { return Ok(false); }
        if left_poly && right_poly {
            if owner_left { right.assignment = left.assignment.clone(); }
            else { left.assignment = right.assignment.clone(); }
        }
        *self = left;
        *other = right;
        Ok(true)
    }

    pub fn has_unifier(&self, formal: &Type, table: &TypeTable) -> Result<bool, TypeError> {
        if !self.is_clean() { return Err(TypeError::PendingAssignments); }
        let mut assignment = self.assignment.clone();
        let formal = assignment.instantiate(&TypeScheme::wrap(formal, 0)?)?;
        assignment.unify(&self.body, &formal, table)
    }

    fn top_expr(&self) -> &Type {
        let mut top = &self.body;
        while let Type::Variable(n) = top {
            match self.assignment.equivalent(*n) {
                Some(next) => top = next,
                None => break,
            }
        }
        top
    }

    /// Both returned components have the same substituted/compacted scope.
    /// Like bake(), they must not be fed into the old assignment afterward.
    pub fn function_parts(&self, table: &TypeTable) -> Result<(Type, Type), TypeError> {
        let expanded = expanded_top(self.top_expr(), table)?;
        match self.assignment.substitution(&expanded)? {
            Type::Function(parts) => Ok(*parts),
            _ => Err(TypeError::ExpectedFunction),
        }
    }

    pub fn matches_argument(&mut self, argument: &Self, table: &TypeTable) -> Result<bool, TypeError> {
        if !argument.is_clean() { return Err(TypeError::PendingAssignments); }
        if self.fixed() < argument.fixed() { self.raise_floor(argument.fixed() - self.fixed())?; }
        self.wring_out()?;
        self.body = expanded_top(&self.body, table)?;
        let formal = match &self.body {
            Type::Function(parts) => parts.0.clone(),
            _ => return Err(TypeError::ExpectedFunction),
        };
        // append accounts for both the function's degree and any difference
        // between the two floors; only shifting by degree can capture a free
        // variable when importing an argument inferred in an earlier scope.
        let diff = self.assignment.append(&argument.assignment)?;
        let actual = shift(&argument.body, argument.fixed(), diff)?;
        self.assignment.unify(&formal, &actual, table)
    }

    /// Specialise a structural pattern, preserving repeated-variable linkage
    /// on the inferred side. The pattern's explicit variables are not inferred.
    /// Failure may leave partial substitutions; trial users must roll back.
    pub fn unify_specialise(&mut self, pattern: &mut Type, table: &TypeTable) -> Result<bool, TypeError> {
        self.assignment.validate(&self.body)?;
        table.validate_applications(&self.body)?;
        table.validate_applications(pattern)?;
        self.unify_specialise_inner(&self.body.clone(), pattern, table)
    }

    pub fn try_unify_specialise(&mut self, pattern: &mut Type, table: &TypeTable) -> Result<bool, TypeError> {
        let (saved_type, saved_pattern) = (self.clone(), pattern.clone());
        let result = self.unify_specialise(pattern, table);
        if result != Ok(true) { *self = saved_type; *pattern = saved_pattern; }
        result
    }

    fn unify_specialise_inner(&mut self, actual: &Type, pattern: &mut Type, table: &TypeTable)
        -> Result<bool, TypeError>
    {
        if *pattern == Type::Undetermined {
            *pattern = actual.clone();
            return Ok(true);
        }
        if let Type::Variable(n) = actual {
            if *n >= self.fixed() && self.assignment.equivalent(*n).is_none() {
                let ceiling = self.fixed().checked_add(self.degree()).ok_or(TypeError::IndexOverflow)?;
                let plug = TypeScheme::wrap(pattern, ceiling)?;
                self.assignment.grow(plug.degree())?;
                return Ok(pattern.specialise(plug.body(), table) && self.assignment.assign(*n, plug.body()));
            }
        }
        if let (Some(p), Some(q)) = (named(actual), named(pattern)) {
            if p != q {
                if table.is_recursive(p) && table.is_recursive(q) { return Ok(false); }
                let actual = expanded_top(actual, table)?;
                *pattern = expanded_top(pattern, table)?;
                return self.unify_specialise_inner(&actual, pattern, table);
            }
            let actual_args = match actual { Type::Applied(_, args) => args.as_slice(), _ => &[] };
            let pattern_args = match pattern { Type::Applied(_, args) => args.as_mut_slice(), _ => &mut [] };
            if actual_args.len() != pattern_args.len() { return Ok(false); }
            for (a, p) in actual_args.iter().zip(pattern_args) {
                *p = expanded_top(p, table)?;
                if !self.unify_specialise_inner(a, p, table)? { return Ok(false); }
            }
            return Ok(true);
        }
        if named(actual).is_some() {
            return self.unify_specialise_inner(&expanded_top(actual, table)?, pattern, table);
        }
        if named(pattern).is_some() {
            *pattern = expanded_top(pattern, table)?;
            return self.unify_specialise_inner(actual, pattern, table);
        }
        match (actual, pattern) {
            (Type::Variable(n), pattern) => match self.assignment.equivalent(*n).cloned() {
                Some(t) => self.unify_specialise_inner(&t, pattern, table),
                None => Ok(*pattern == Type::Variable(*n)),
            },
            (Type::Primitive(a), Type::Primitive(p)) => Ok(a == p),
            (Type::Row(a), Type::Row(p)) => self.unify_specialise_inner(a, p, table),
            (Type::Function(a), Type::Function(p)) => Ok(
                self.unify_specialise_inner(&a.0, &mut p.0, table)?
                && self.unify_specialise_inner(&a.1, &mut p.1, table)?),
            (Type::Tuple(a), Type::Tuple(p)) | (Type::Union(a), Type::Union(p)) => {
                if a.len() != p.len() { return Ok(false); }
                for (a, p) in a.iter().zip(p) {
                    if !self.unify_specialise_inner(a, p, table)? { return Ok(false); }
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    /// Test one global overload's formal argument, returning the shift that
    /// MUST also be applied when substituting that overload's result type.
    /// The caller clears/restores the original degree between candidate trials;
    /// success does not select an overload or resolve ambiguity by itself.
    pub fn matches(&mut self, formal: &Type, degree: usize, table: &TypeTable)
        -> Result<(bool, usize), TypeError>
    {
        if !self.is_clean() { return Err(TypeError::PendingAssignments); }
        TypeScheme::constructor(formal.clone(), degree)?;
        table.validate_applications(formal)?;
        let diff = self.assignment.append(&TypeAssignment::new(0, degree)?)?;
        let formal = shift(formal, 0, diff)?;
        Ok((self.assignment.unify(&formal, &self.body, table)?, diff))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Prim, TypeBinding};
    fn int() -> Type { Type::Primitive(Prim::Int) }
    fn rat() -> Type { Type::Primitive(Prim::Rat) }
    fn var(n: usize) -> Type { Type::Variable(n) }

    #[test]
    fn wrapping_preserves_linkage_but_not_independent_holes() {
        let scheme = TypeScheme::wrap(&Type::tuple(vec![var(8), Type::Undetermined, var(8), Type::Undetermined, var(0)]), 1).unwrap();
        assert_eq!(scheme.body(), &Type::tuple(vec![var(1), var(2), var(1), var(3), var(0)]));
        assert_eq!(scheme.degree(), 3);
        assert!(!TypeScheme::wrap(&Type::row(var(0)), 1).unwrap().is_polymorphic());
        assert!(TypeScheme::wrap(&Type::row(var(0)), 0).unwrap().is_polymorphic());
    }

    #[test]
    fn repeated_variables_must_have_one_type_and_trials_roll_back() {
        let table = TypeTable::new();
        let mut a = TypeAssignment::new(0, 1).unwrap();
        let before = a.clone();
        assert!(!a.try_unify(&Type::tuple(vec![var(0), var(0)]), &Type::tuple(vec![int(), rat()]), &table).unwrap());
        assert_eq!(a, before);
        assert!(a.unify(&Type::tuple(vec![var(0), var(0)]), &Type::tuple(vec![int(), int()]), &table).unwrap());
        assert_eq!(a.substitution(&Type::row(var(0))).unwrap(), Type::row(int()));
    }

    #[test]
    fn rigid_variables_cannot_be_assigned_but_can_be_referenced() {
        let table = TypeTable::new();
        let mut a = TypeAssignment::new(2, 1).unwrap();
        assert!(!a.unify(&var(0), &int(), &table).unwrap());
        assert!(!a.unify(&var(0), &var(1), &table).unwrap());
        assert!(a.unify(&var(2), &Type::row(var(0)), &table).unwrap());
        assert_eq!(a.substitution(&var(2)).unwrap(), Type::row(var(0)));
    }

    #[test]
    fn occurs_check_rejects_direct_and_indirect_recursive_inference() {
        let table = TypeTable::new();
        let mut a = TypeAssignment::new(0, 2).unwrap();
        assert!(!a.unify(&var(0), &Type::row(var(0)), &table).unwrap());
        assert!(a.unify(&var(0), &var(1), &table).unwrap());
        assert!(!a.unify(&var(1), &Type::row(var(0)), &table).unwrap());
        let mut a = TypeAssignment::new(0, 2).unwrap();
        assert!(!a.unify(&Type::function(var(0), Type::row(var(0))), &Type::function(var(1), var(1)), &table).unwrap());
    }

    #[test]
    fn fresh_instances_do_not_capture_each_other_or_fixed_context() {
        let table = TypeTable::new();
        let scheme = TypeScheme::wrap(&Type::function(var(0), var(0)), 0).unwrap();
        let saved = scheme.clone();
        let mut a = TypeAssignment::new(2, 0).unwrap();
        let first = a.instantiate(&scheme).unwrap();
        let second = a.instantiate(&scheme).unwrap();
        assert_eq!(first, Type::function(var(2), var(2)));
        assert_eq!(second, Type::function(var(3), var(3)));
        assert!(a.unify(&first, &Type::function(int(), int()), &table).unwrap());
        assert!(a.unify(&second, &Type::function(rat(), rat()), &table).unwrap());
        assert_eq!(scheme, saved);
        let fixed = TypeScheme::wrap(&var(0), 1).unwrap();
        assert_eq!(TypeAssignment::new(0, 0).unwrap().instantiate(&fixed), Err(TypeError::ScopeCapture));
    }

    #[test]
    fn field_definition_matching_preserves_rigid_variables_and_forgotten_slots() {
        let mut table = TypeTable::new();
        let generic = table.add_simple_constructor(TypeBinding {
            name: "GenericFields".into(),
            definition: Type::tuple(vec![var(0), var(0)]),
            fields: vec![Some("first".into()), Some("second".into())],
        }, 1);
        let concrete = table.add_simple(TypeBinding {
            name: "ConcreteFields".into(),
            definition: Type::tuple(vec![int(), int()]),
            fields: vec![Some("first".into()), Some("other".into())],
        });
        let rigid = InferredType::wrap(&Type::tuple(vec![var(0), var(0)]), 1).unwrap();
        assert_eq!(table.matching_bindings(&rigid).unwrap(), vec![generic]);
        assert!(rigid.is_clean());
        assert_eq!(rigid.fixed(), 1);
        assert_eq!(rigid.degree(), 0);

        let receiver = InferredType::wrap(&Type::Applied(generic, vec![int()]), 0).unwrap();
        assert_eq!(table.matching_bindings(&receiver).unwrap(), vec![generic, concrete]);
        assert!(table.forget("GenericFields"));
        assert!(table.lookup("GenericFields").is_none());
        assert_eq!(table.matching_bindings(&receiver).unwrap(), vec![generic, concrete]);
        assert_eq!(receiver.bake().unwrap(), Type::Applied(generic, vec![int()]));
    }

    #[test]
    fn leaving_abstraction_preserves_existing_substitution_slots() {
        let table = TypeTable::new();
        let mut a = TypeAssignment::new(2, 1).unwrap();
        assert!(a.unify(&var(2), &Type::row(var(1)), &table).unwrap());
        a.lower_floor(1).unwrap();
        assert_eq!(a.fixed(), 1);
        assert!(a.unify(&var(1), &int(), &table).unwrap());
        assert_eq!(a.substitution(&var(2)).unwrap(), Type::row(int()));
        assert_eq!(a.substitution(&var(0)).unwrap(), var(0));
    }

    #[test]
    fn constructor_substitution_is_simultaneous_and_retains_unused_arity() {
        let mut table = TypeTable::new();
        let body = Type::tuple(vec![var(0), var(0)]);
        assert_eq!(TypeScheme::constructor(body.clone(), 2).unwrap().degree(), 2);
        let n = table.add_constructor(TypeBinding { name: "Repeated".into(), definition: body, fields: vec![] }, 2, false);
        let applied = Type::Applied(n, vec![int(), rat()]);
        assert_eq!(table.expand_application(&applied).unwrap(), Type::tuple(vec![int(), int()]));
        assert_eq!(applied.display(&table).to_string(), "Repeated<int,rat>");
        assert_eq!(table.expand_application(&Type::Applied(n, vec![int()])), Err(TypeError::Arity { expected: 2, found: 1 }));
        assert_eq!(substitute_parameters(&Type::tuple(vec![var(0), var(1)]), &[var(1), int()]).unwrap(), Type::tuple(vec![var(1), int()]));
    }

    #[test]
    fn recursive_constructor_identity_terminates_without_expansion() {
        let mut table = TypeTable::new();
        let a = table.add_constructor(TypeBinding { name: "Loop".into(), definition: Type::Undetermined, fields: vec![] }, 1, true);
        table.update(a, Type::union_of(vec![Type::void(), Type::Applied(a, vec![var(0)])]), vec![]);
        let b = table.add_constructor(TypeBinding { name: "Other".into(), definition: Type::Applied(a, vec![var(0)]), fields: vec![] }, 1, true);
        let mut assign = TypeAssignment::new(0, 1).unwrap();
        assert!(assign.unify(&Type::Applied(a, vec![var(0)]), &Type::Applied(a, vec![int()]), &table).unwrap());
        assert!(!assign.unify(&Type::Applied(a, vec![int()]), &Type::Applied(b, vec![int()]), &table).unwrap());
    }

    #[test]
    fn rejects_holes_out_of_scope_indices_and_overflow() {
        let mut a = TypeAssignment::new(0, 1).unwrap();
        let table = TypeTable::new();
        assert_eq!(a.unify(&Type::Undetermined, &int(), &table), Err(TypeError::UndeterminedInAssignment));
        assert_eq!(a.unify(&var(1), &int(), &table), Err(TypeError::VariableOutOfRange(1)));
        assert_eq!(shift(&var(usize::MAX), 0, 1), Err(TypeError::IndexOverflow));
        assert_eq!(TypeAssignment::new(usize::MAX, 1), Err(TypeError::IndexOverflow));
        assert_eq!(shift(&Type::tuple(vec![var(0), var(1)]), 1, 3).unwrap(), Type::tuple(vec![var(0), var(4)]));
    }

    #[test]
    fn invalid_applications_cannot_bypass_validation_in_fast_paths() {
        let mut table = TypeTable::new();
        let n = table.add_constructor(TypeBinding { name: "Pair".into(), definition: Type::tuple(vec![var(0), var(1)]), fields: vec![] }, 2, false);
        let mut a = TypeAssignment::new(0, 1).unwrap();
        for bad in [Type::Applied(n, vec![int()]), Type::Applied(n, vec![int(), rat(), int()]), Type::Tabled(n)] {
            assert!(matches!(a.unify(&bad, &bad, &table), Err(TypeError::Arity { .. })));
            let before = a.clone();
            assert!(matches!(a.try_unify(&var(0), &Type::row(bad), &table), Err(TypeError::Arity { .. })));
            assert_eq!(a, before);
        }
        let bad = Type::Applied(super::super::TypeNumber(17), vec![]);
        assert_eq!(a.unify(&bad, &bad, &table), Err(TypeError::UnknownConstructor(17)));
        assert_eq!(table.expand_application(&bad), Err(TypeError::UnknownConstructor(17)));
    }

    #[test]
    fn aliases_can_expand_to_fixed_variables_and_structurally_equal_types() {
        let mut table = TypeTable::new();
        let a = table.add_constructor(TypeBinding { name: "Identity".into(), definition: var(0), fields: vec![] }, 1, false);
        let b = table.add_constructor(TypeBinding { name: "Identity2".into(), definition: var(0), fields: vec![] }, 1, false);
        let mut assign = TypeAssignment::new(1, 0).unwrap();
        assert!(assign.unify(&Type::Applied(a, vec![var(0)]), &var(0), &table).unwrap());
        assert!(assign.unify(&var(0), &Type::Applied(a, vec![var(0)]), &table).unwrap());
        assert!(assign.unify(&Type::Applied(a, vec![int()]), &Type::Applied(b, vec![int()]), &table).unwrap());
        assert!(!assign.unify(&Type::Applied(a, vec![int()]), &Type::Applied(b, vec![rat()]), &table).unwrap());
    }

    #[test]
    fn substitution_follows_assignments_then_compacts_unassigned_slots() {
        let table = TypeTable::new();
        let mut a = TypeAssignment::new(1, 4).unwrap();
        assert!(a.unify(&var(1), &Type::row(var(3)), &table).unwrap());
        assert!(a.unify(&var(3), &var(2), &table).unwrap());
        assert_eq!(a.substitution(&Type::tuple(vec![var(0), var(1), var(2), var(3), var(4)])).unwrap(),
                   Type::tuple(vec![var(0), Type::row(var(1)), var(1), var(1), var(2)]));
        assert!(a.unify(&var(2), &int(), &table).unwrap());
        assert_eq!(a.substitution(&var(1)).unwrap(), Type::row(int()));
    }

    #[test]
    fn legacy_specialisation_does_not_expand_distinct_recursive_names_forever() {
        let mut table = TypeTable::new();
        let a = table.add_constructor(TypeBinding { name: "Loop".into(), definition: Type::Undetermined, fields: vec![] }, 1, true);
        table.update(a, Type::union_of(vec![Type::void(), Type::Applied(a, vec![var(0)])]), vec![]);
        let b = table.add_constructor(TypeBinding { name: "Other".into(), definition: Type::Applied(a, vec![var(0)]), fields: vec![] }, 1, true);
        let mut p = Type::Applied(a, vec![int()]);
        let q = Type::Applied(b, vec![int()]);
        assert!(!p.can_specialise(&q, &table));
        assert!(!p.specialise(&q, &table));
    }

    #[test]
    fn specialising_applied_constructor_exposes_and_refines_its_structure() {
        // axis-types.w:942-979 expands the receiver, so callers can access
        // the row/tuple/function components after successful specialisation.
        let mut table = TypeTable::new();
        let n = table.add_constructor(TypeBinding { name: "Rows".into(), definition: Type::row(var(0)), fields: vec![] }, 1, false);
        let mut applied = Type::Applied(n, vec![Type::Undetermined]);
        let pattern = Type::row(int());
        assert!(applied.can_specialise(&pattern, &table));
        assert!(applied.specialise(&pattern, &table));
        assert_eq!(applied, pattern);
    }

    #[test]
    fn appended_assignments_shift_pending_values_and_preserve_rigid_variables() {
        let table = TypeTable::new();
        let mut source = TypeAssignment::new(1, 2).unwrap();
        assert!(source.unify(&var(1), &Type::tuple(vec![var(0), var(2)]), &table).unwrap());
        let mut target = TypeAssignment::new(3, 1).unwrap();
        let diff = target.append(&source).unwrap();
        assert_eq!(diff, 3);
        assert!(target.unify(&var(5), &int(), &table).unwrap());
        assert_eq!(target.substitution(&var(4)).unwrap(), Type::tuple(vec![var(0), int()]));
        assert_eq!(source.substitution(&var(1)).unwrap(), Type::tuple(vec![var(0), var(1)]));
        let mut too_low = TypeAssignment::new(0, 1).unwrap();
        let saved = too_low.clone();
        assert_eq!(too_low.append(&source), Err(TypeError::ScopeCapture));
        assert_eq!(too_low, saved);
    }

    #[test]
    fn inferred_tuple_freshens_components_without_capturing_new_fixed_types() {
        let component = InferredType::wrap(&Type::tuple(vec![var(7), var(7)]), 0).unwrap();
        let tuple = InferredType::wrap_tuple(vec![component.clone(), component], 2).unwrap();
        assert_eq!(tuple.body(), &Type::tuple(vec![
            Type::tuple(vec![var(2), var(2)]), Type::tuple(vec![var(3), var(3)])]));
        assert_eq!((tuple.fixed(), tuple.degree()), (2, 2));
        let empty = InferredType::wrap_tuple(vec![], 3).unwrap();
        assert_eq!((empty.fixed(), empty.degree()), (3, 0));
        assert_eq!(empty.body(), &Type::tuple(vec![]));
        let fixed = InferredType::wrap(&var(1), 2).unwrap();
        assert_eq!(InferredType::wrap_tuple(vec![fixed], 1), Err(TypeError::ScopeCapture));
    }

    #[test]
    fn raising_floor_bakes_pending_types_before_shifting_free_variables() {
        let table = TypeTable::new();
        let mut value = InferredType::wrap(&Type::tuple(vec![var(0), var(1)]), 0).unwrap();
        let requirement = InferredType::wrap(&Type::tuple(vec![int(), Type::Undetermined]), 0).unwrap();
        assert!(value.unify_to(&requirement, &table).unwrap());
        assert!(!value.is_clean());
        value.raise_floor(2).unwrap();
        assert!(value.is_clean());
        assert_eq!((value.fixed(), value.degree()), (2, 1));
        assert_eq!(value.body(), &Type::tuple(vec![int(), var(2)]));
        assert_eq!(requirement.bake().unwrap(), Type::tuple(vec![int(), var(0)]));
    }

    #[test]
    fn inferred_constness_distinguishes_fixed_types_from_free_types() {
        let table = TypeTable::new();
        let mut fixed = InferredType::wrap(&Type::row(var(0)), 1).unwrap();
        assert!(!fixed.is_polymorphic());
        fixed.lower_floor(1).unwrap();
        assert!(fixed.is_polymorphic());
        let concrete = InferredType::wrap(&Type::row(int()), 0).unwrap();
        assert!(fixed.unify_to(&concrete, &table).unwrap());
        assert!(!fixed.is_polymorphic());
        assert_eq!(fixed.bake().unwrap(), Type::row(int()));
        fixed.wring_out().unwrap();
        assert_eq!(fixed.degree(), 0);
    }

    #[test]
    fn unify_to_raises_scope_and_imports_pending_other_assignments() {
        let table = TypeTable::new();
        let mut other = InferredType::wrap(&Type::tuple(vec![var(0), var(2), var(3)]), 1).unwrap();
        let expected = InferredType::wrap(&Type::tuple(vec![var(0), int(), Type::Undetermined]), 1).unwrap();
        assert!(other.unify_to(&expected, &table).unwrap());
        let saved = other.clone();
        let mut bottom = InferredType::bottom(0).unwrap();
        assert!(bottom.unify_to(&other, &table).unwrap());
        assert_eq!(bottom.fixed(), 1);
        assert_eq!(bottom.bake().unwrap(), Type::tuple(vec![var(0), int(), var(1)]));
        assert_eq!(other, saved);
    }

    #[test]
    fn failed_inferred_trial_restores_scope_and_all_assignments() {
        let table = TypeTable::new();
        let mut actual = InferredType::wrap(&Type::tuple(vec![var(0), var(0)]), 0).unwrap();
        let other = InferredType::wrap(&Type::tuple(vec![int(), rat()]), 2).unwrap();
        let saved = actual.clone();
        assert!(!actual.try_unify_to(&other, &table).unwrap());
        assert_eq!(actual, saved);
    }

    #[test]
    fn overload_trial_shift_is_reused_for_the_function_result() {
        let table = TypeTable::new();
        let mut argument = InferredType::wrap(&Type::tuple(vec![var(0), int()]), 1).unwrap();
        let (matched, diff) = argument.matches(&Type::tuple(vec![var(0), var(1)]), 2, &table).unwrap();
        assert!(matched);
        assert_eq!(diff, 1);
        let result = shift(&Type::tuple(vec![var(1), var(0)]), 0, diff).unwrap();
        assert_eq!(argument.assignments().substitution(&result).unwrap(), Type::tuple(vec![int(), var(0)]));
        assert_eq!(argument.matches(&int(), 0, &table), Err(TypeError::PendingAssignments));
        argument.clear(0).unwrap();
        assert!(argument.is_clean());
    }

    #[test]
    fn failed_overload_trial_can_be_cleared_before_next_candidate() {
        let table = TypeTable::new();
        let mut argument = InferredType::wrap(&Type::tuple(vec![int(), rat()]), 0).unwrap();
        assert!(!argument.matches(&Type::tuple(vec![var(0), var(0)]), 1, &table).unwrap().0);
        argument.clear(0).unwrap();
        let (matched, shift) = argument.matches(&Type::tuple(vec![var(0), var(1)]), 2, &table).unwrap();
        assert!(matched);
        assert_eq!(shift, 0);
        assert_eq!(argument.assignments().substitution(&var(1)).unwrap(), rat());
    }

    #[test]
    fn constructor_scope_keeps_unused_parameters_until_explicit_repacking() {
        let scheme = TypeScheme::constructor(Type::tuple(vec![var(0), var(0)]), 2).unwrap();
        let mut constructor = InferredType::from_scheme(scheme).unwrap();
        constructor.wring_out().unwrap();
        assert_eq!(constructor.degree(), 2); // clean: no implicit re-packing
        constructor.raise_floor(2).unwrap();
        assert_eq!((constructor.fixed(), constructor.degree()), (2, 2));
        assert_eq!(constructor.body(), &Type::tuple(vec![var(2), var(2)]));
    }

    #[test]
    fn structural_matching_links_repeated_variables_and_can_roll_back() {
        let table = TypeTable::new();
        let mut actual = InferredType::wrap(&Type::function(var(0), var(0)), 0).unwrap();
        let mut pattern = Type::function(int(), rat());
        let saved = (actual.clone(), pattern.clone());
        assert!(!actual.try_unify_specialise(&mut pattern, &table).unwrap());
        assert_eq!((actual.clone(), pattern), saved);
        let mut pattern = Type::function(int(), int());
        assert!(actual.unify_specialise(&mut pattern, &table).unwrap());
        assert_eq!(actual.bake().unwrap(), pattern);
    }

    #[test]
    fn structural_matching_allocates_holes_in_the_existing_scope() {
        let table = TypeTable::new();
        let mut actual = InferredType::bottom(2).unwrap();
        let mut pattern = Type::function(Type::Undetermined, Type::row(Type::Undetermined));
        assert!(actual.unify_specialise(&mut pattern, &table).unwrap());
        assert_eq!(pattern, Type::function(var(3), Type::row(var(4))));
        assert_eq!(actual.bake().unwrap(), Type::function(var(2), Type::row(var(3))));
        let concrete = InferredType::wrap(&Type::function(int(), Type::row(rat())), 2).unwrap();
        assert!(actual.unify_to(&concrete, &table).unwrap());
        assert_eq!(actual.bake().unwrap(), concrete.bake().unwrap());
    }

    #[test]
    fn structural_matching_does_not_assign_rigid_variables_or_hide_occurs_checks() {
        let table = TypeTable::new();
        let mut fixed = InferredType::wrap(&var(0), 1).unwrap();
        assert!(!fixed.unify_specialise(&mut int(), &table).unwrap());
        assert!(fixed.unify_specialise(&mut var(0), &table).unwrap());
        let mut free = InferredType::bottom(0).unwrap();
        let mut recursive = Type::row(var(0));
        let saved = free.clone();
        assert!(!free.try_unify_specialise(&mut recursive, &table).unwrap());
        assert_eq!(free, saved);
    }

    #[test]
    fn structural_named_matching_exposes_patterns_and_preserves_recursive_identity() {
        let mut table = TypeTable::new();
        let rows = table.add_constructor(TypeBinding { name: "Rows".into(), definition: Type::row(var(0)), fields: vec![] }, 1, false);
        let mut actual = InferredType::wrap(&Type::row(int()), 0).unwrap();
        let mut pattern = Type::Applied(rows, vec![Type::Undetermined]);
        assert!(actual.unify_specialise(&mut pattern, &table).unwrap());
        assert_eq!(pattern, Type::row(int()));
        let a = table.add_constructor(TypeBinding { name: "Loop".into(), definition: Type::void(), fields: vec![] }, 1, true);
        table.update(a, Type::union_of(vec![Type::void(), Type::Applied(a, vec![var(0)])]), vec![]);
        let b = table.add_constructor(TypeBinding { name: "Other".into(), definition: Type::Applied(a, vec![var(0)]), fields: vec![] }, 1, true);
        let mut recursive = InferredType::wrap(&Type::Applied(a, vec![int()]), 0).unwrap();
        assert!(!recursive.unify_specialise(&mut Type::Applied(b, vec![int()]), &table).unwrap());
        let mut same = Type::Applied(a, vec![Type::Undetermined]);
        assert!(recursive.unify_specialise(&mut same, &table).unwrap());
        assert_eq!(same, Type::Applied(a, vec![int()]));
    }

    #[test]
    fn identical_named_patterns_expand_their_argument_aliases() {
        let mut table = TypeTable::new();
        let rows = table.add_constructor(TypeBinding { name: "Rows".into(), definition: Type::row(var(0)), fields: vec![] }, 1, false);
        let number = table.add_constructor(TypeBinding { name: "Number".into(), definition: int(), fields: vec![] }, 0, false);
        let mut actual = InferredType::wrap(&Type::Applied(rows, vec![int()]), 0).unwrap();
        let mut pattern = Type::Applied(rows, vec![Type::Tabled(number)]);
        assert!(actual.unify_specialise(&mut pattern, &table).unwrap());
        assert_eq!(pattern, Type::Applied(rows, vec![int()]));
    }

    #[test]
    fn generic_function_call_substitutes_the_result_without_mutating_the_argument() {
        let table = TypeTable::new();
        let mut function = InferredType::wrap(&Type::function(var(0), Type::row(var(0))), 0).unwrap();
        let argument = InferredType::wrap(&int(), 0).unwrap();
        let saved = argument.clone();
        assert!(function.matches_argument(&argument, &table).unwrap());
        assert_eq!(function.function_parts(&table).unwrap(), (int(), Type::row(int())));
        assert_eq!(argument, saved);
    }

    #[test]
    fn call_preserves_rigid_context_and_independent_polymorphic_argument() {
        let table = TypeTable::new();
        let mut function = InferredType::wrap(&Type::function(var(0), var(0)), 0).unwrap();
        let argument = InferredType::wrap(&Type::tuple(vec![var(0), Type::Undetermined]), 1).unwrap();
        assert!(function.matches_argument(&argument, &table).unwrap());
        assert_eq!(function.fixed(), 1);
        let expected = Type::tuple(vec![var(0), var(1)]);
        assert_eq!(function.function_parts(&table).unwrap(), (expected.clone(), expected));
        let mut late_function = InferredType::wrap(&Type::function(var(3), var(3)), 2).unwrap();
        let early_argument = InferredType::wrap(&Type::row(Type::Undetermined), 0).unwrap();
        assert!(late_function.matches_argument(&early_argument, &table).unwrap());
        assert_eq!(late_function.function_parts(&table).unwrap(), (Type::row(var(2)), Type::row(var(2))));
    }

    #[test]
    fn function_components_follow_constructor_and_assignment_indirections() {
        let mut table = TypeTable::new();
        let n = table.add_constructor(TypeBinding { name: "Function".into(), definition: Type::function(var(0), Type::row(var(0))), fields: vec![] }, 1, false);
        let named = InferredType::wrap(&Type::Applied(n, vec![int()]), 0).unwrap();
        let mut value = InferredType::bottom(0).unwrap();
        assert!(value.unify_to(&named, &table).unwrap());
        assert_eq!(value.function_parts(&table).unwrap(), (int(), Type::row(int())));
        assert!(value.matches_argument(&InferredType::wrap(&int(), 0).unwrap(), &table).unwrap());
        assert_eq!(value.function_parts(&table).unwrap(), (int(), Type::row(int())));
        assert_eq!(InferredType::wrap(&int(), 0).unwrap().function_parts(&table), Err(TypeError::ExpectedFunction));
    }

    #[test]
    fn two_sided_unification_shares_substitutions_without_capturing_scopes() {
        let table = TypeTable::new();
        for reverse in [false, true] {
            let mut left = InferredType::wrap(&Type::function(var(0), var(0)), 0).unwrap();
            let mut right = InferredType::wrap(&Type::function(var(0), Type::Undetermined), 1).unwrap();
            let matched = if reverse { right.unify(&mut left, &table) } else { left.unify(&mut right, &table) };
            assert!(matched.unwrap());
            assert_eq!(left.bake().unwrap(), Type::function(var(0), var(0)));
            assert_eq!(left.bake().unwrap(), right.bake().unwrap());
            assert_eq!(left.assignments(), right.assignments());
            assert!(!left.is_polymorphic());
        }
    }

    #[test]
    fn two_sided_failure_restores_both_types_including_pending_constraints() {
        let table = TypeTable::new();
        let mut left = InferredType::wrap(&Type::tuple(vec![Type::Undetermined, int()]), 0).unwrap();
        let mut right = InferredType::wrap(&Type::tuple(vec![rat(), rat()]), 1).unwrap();
        let saved = (left.clone(), right.clone());
        assert!(!left.unify(&mut right, &table).unwrap());
        assert_eq!((left, right), saved);
    }

    #[test]
    fn has_unifier_is_read_only_and_treats_formal_variables_as_fresh() {
        let table = TypeTable::new();
        let actual = InferredType::wrap(&Type::tuple(vec![var(0), int()]), 1).unwrap();
        let saved = actual.clone();
        assert!(actual.has_unifier(&Type::tuple(vec![var(0), var(1)]), &table).unwrap());
        assert!(!actual.has_unifier(&Type::tuple(vec![var(0), var(0)]), &table).unwrap());
        assert_eq!(actual, saved);
    }
}
