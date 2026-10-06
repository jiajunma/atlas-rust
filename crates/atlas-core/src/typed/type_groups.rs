//! Group-local name resolution before recursive graph construction.
//! Keep validation order and diagnostics from global.w:1632-1860 separate
//! from ordinary aliases, which use their existing resolution contract.

use std::collections::VecDeque;
use crate::diagnostic::{Diagnostic, ErrorKind};
use crate::syntax::{SpannedValue, TypeExpr, TypeSpec};
use crate::types::{Type, TypeNumber, TypeTable};

pub(super) fn resolve_spec(spec: &TypeSpec, table: &TypeTable,
                          locals: &[TypeNumber], arity: usize)
    -> Result<(Type, Vec<Option<String>>), Diagnostic>
{
    let roots: Vec<_> = match spec {
        TypeSpec::Alias(expr) => vec![expr],
        TypeSpec::Struct(fields) | TypeSpec::Union(fields) =>
            fields.iter().map(|field| &field.type_expr).collect(),
    };
    // The original visits each RHS breadth-first, result before argument.
    // Validate an application BEFORE its arguments; local uses never accept
    // explicit arguments, even if their number happens to match the arity.
    let mut work: VecDeque<_> = roots.iter().copied().collect();
    while let Some(expr) = work.pop_front() {
        let named = match expr {
            TypeExpr::Named { name, span } => Some((name, *span, 0, false)),
            TypeExpr::Applied { name, arguments, .. } => {
                work.extend(arguments);
                Some((&name.value, name.span, arguments.len(), true))
            }
            TypeExpr::Row { component, .. } => { work.push_back(component); None }
            TypeExpr::Function { argument, result, .. } => {
                work.push_back(result); work.push_back(argument); None
            }
            TypeExpr::Tuple { components, .. } => { work.extend(components); None }
            TypeExpr::Union { variants, .. } => { work.extend(variants); None }
            _ => None,
        };
        if let Some((name, span, supplied, explicit)) = named {
            let error = |message| Diagnostic::new(ErrorKind::Program, message, Some(span));
            let number = table.lookup(name).ok_or_else(|| error(
                format!("Identifier '{name}' does not refer to any type")))?;
            if locals.contains(&number) {
                if explicit { return Err(error(
                    format!("Type '{name}' being defined cannot be given type arguments"))); }
            } else {
                let expected = table.constructor_arity(number);
                if supplied != expected { return Err(error(format!(
                    "Type constructor '{name}' called with {supplied} type arguments, expected {expected}"))); }
            }
        }
    }
    let resolve = |expr: &TypeExpr| {
        let mut expr = expr.clone();
        forward_formals(&mut expr, table, locals, arity);
        expr.resolve_in(table).map_err(|mut error| { error.kind = ErrorKind::Program; error })
    };
    match spec {
        TypeSpec::Alias(expr) => Ok((resolve(expr)?, vec![])),
        TypeSpec::Struct(fields) | TypeSpec::Union(fields) => {
            let components = fields.iter().map(|field| resolve(&field.type_expr))
                .collect::<Result<Vec<_>, _>>()?;
            let names = fields.iter().map(|field| field.name.as_ref().map(|name| name.value.clone())).collect();
            Ok((if matches!(spec, TypeSpec::Struct(_)) { Type::tuple(components) }
                else { Type::union_of(components) }, names))
        }
    }
}

fn forward_formals(expr: &mut TypeExpr, table: &TypeTable, locals: &[TypeNumber], arity: usize) {
    match expr {
        TypeExpr::Named { name, span } if arity != 0
            && table.lookup(name).is_some_and(|n| locals.contains(&n)) => {
                *expr = TypeExpr::Applied {
                    name: SpannedValue { value: name.clone(), span: *span },
                    arguments: (0..arity).map(|index| TypeExpr::Variable { index, span: *span }).collect(),
                    span: *span,
                };
            }
        TypeExpr::Row { component, .. } => forward_formals(component, table, locals, arity),
        TypeExpr::Function { argument, result, .. } => {
            forward_formals(argument, table, locals, arity);
            forward_formals(result, table, locals, arity);
        }
        TypeExpr::Tuple { components, .. } | TypeExpr::Union { variants: components, .. }
        | TypeExpr::Applied { arguments: components, .. } => {
            for component in components { forward_formals(component, table, locals, arity); }
        }
        _ => {}
    }
}
