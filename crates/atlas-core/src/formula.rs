//! Atlas operator-priority reduction.
//!
//! The structural parser sees a formula as an alternating sequence of
//! operands and operators. This module reproduces the upstream formula stack
//! algorithm, leaving name lookup and operator overload resolution to later
//! compiler stages.

use crate::diagnostic::SourceSpan;

/// An operator as it appears in an Atlas formula.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FormulaOperator {
    pub symbol: String,
    pub priority: i32,
    pub span: Option<SourceSpan>,
}

impl FormulaOperator {
    pub fn new(symbol: impl Into<String>, priority: i32) -> Self {
        Self {
            symbol: symbol.into(),
            priority,
            span: None,
        }
    }

    pub fn with_span(mut self, span: SourceSpan) -> Self {
        self.span = Some(span);
        self
    }
}

/// The operator-call tree produced after formula priorities are resolved.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FormulaTree<T> {
    Operand(T),
    Unary {
        operator: FormulaOperator,
        operand: Box<FormulaTree<T>>,
    },
    Binary {
        operator: FormulaOperator,
        left: Box<FormulaTree<T>>,
        right: Box<FormulaTree<T>>,
    },
}

/// A partially reduced formula owned by parser actions.
#[derive(Debug)]
pub struct FormulaStack<T> {
    pending: Vec<PendingOperator<T>>,
}

#[derive(Debug)]
struct PendingOperator<T> {
    left: Option<FormulaTree<T>>,
    operator: FormulaOperator,
}

/// Start a formula whose first operator is binary.
pub fn start_formula<T>(operand: T, operator: FormulaOperator) -> FormulaStack<T> {
    FormulaStack {
        pending: vec![PendingOperator {
            left: Some(FormulaTree::Operand(operand)),
            operator,
        }],
    }
}

/// Start a formula with a prefix operator.
///
/// Matching Atlas, an initial unary operator participates in priority
/// comparisons with operators that follow it. A unary operator after a binary
/// operator is part of that binary operator's operand and is reduced by the
/// structural parser before it reaches this stack.
pub fn start_unary_formula<T>(operator: FormulaOperator) -> FormulaStack<T> {
    FormulaStack {
        pending: vec![PendingOperator {
            left: None,
            operator,
        }],
    }
}

/// Add an operand followed by a binary operator to a partial formula.
pub fn extend_formula<T>(
    mut stack: FormulaStack<T>,
    operand: T,
    operator: FormulaOperator,
) -> FormulaStack<T> {
    let mut expression = FormulaTree::Operand(operand);

    while stack
        .pending
        .last()
        .map(|pending| should_reduce(pending.operator.priority, operator.priority))
        .unwrap_or(false)
    {
        if let Some(pending) = stack.pending.pop() {
            expression = reduce(pending, expression);
        }
    }

    stack.pending.push(PendingOperator {
        left: Some(expression),
        operator,
    });
    stack
}

/// Finish a partial formula by reducing every pending operator.
pub fn end_formula<T>(mut stack: FormulaStack<T>, operand: T) -> FormulaTree<T> {
    let mut expression = FormulaTree::Operand(operand);
    while let Some(pending) = stack.pending.pop() {
        expression = reduce(pending, expression);
    }
    expression
}

fn should_reduce(pending_priority: i32, current_priority: i32) -> bool {
    pending_priority > current_priority
        || (pending_priority == current_priority && current_priority % 2 == 0)
}

fn reduce<T>(pending: PendingOperator<T>, right: FormulaTree<T>) -> FormulaTree<T> {
    match pending.left {
        Some(left) => FormulaTree::Binary {
            operator: pending.operator,
            left: Box::new(left),
            right: Box::new(right),
        },
        None => FormulaTree::Unary {
            operator: pending.operator,
            operand: Box::new(right),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn operator(symbol: &str, priority: i32) -> FormulaOperator {
        FormulaOperator::new(symbol, priority)
    }

    fn operand(name: &'static str) -> FormulaTree<&'static str> {
        FormulaTree::Operand(name)
    }

    fn unary(
        symbol: &str,
        priority: i32,
        child: FormulaTree<&'static str>,
    ) -> FormulaTree<&'static str> {
        FormulaTree::Unary {
            operator: operator(symbol, priority),
            operand: Box::new(child),
        }
    }

    fn binary(
        symbol: &str,
        priority: i32,
        left: FormulaTree<&'static str>,
        right: FormulaTree<&'static str>,
    ) -> FormulaTree<&'static str> {
        FormulaTree::Binary {
            operator: operator(symbol, priority),
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    #[test]
    fn even_priority_is_left_associative() {
        let stack = start_formula("a", operator("-", 4));
        let stack = extend_formula(stack, "b", operator("-", 4));
        let actual = end_formula(stack, "c");

        assert_eq!(
            actual,
            binary(
                "-",
                4,
                binary("-", 4, operand("a"), operand("b")),
                operand("c"),
            )
        );
    }

    #[test]
    fn odd_priority_is_right_associative() {
        let stack = start_formula("a", operator("^", 7));
        let stack = extend_formula(stack, "b", operator("^", 7));
        let actual = end_formula(stack, "c");

        assert_eq!(
            actual,
            binary(
                "^",
                7,
                operand("a"),
                binary("^", 7, operand("b"), operand("c")),
            )
        );
    }

    #[test]
    fn higher_priority_binds_more_tightly_in_mixed_formulae() {
        let stack = start_formula("a", operator("+", 4));
        let stack = extend_formula(stack, "b", operator("*", 6));
        let stack = extend_formula(stack, "c", operator("-", 4));
        let actual = end_formula(stack, "d");

        assert_eq!(
            actual,
            binary(
                "-",
                4,
                binary(
                    "+",
                    4,
                    operand("a"),
                    binary("*", 6, operand("b"), operand("c")),
                ),
                operand("d"),
            )
        );
    }

    #[test]
    fn initial_unary_waits_for_tighter_following_operator() {
        let stack = start_unary_formula(operator("-", 4));
        let stack = extend_formula(stack, "a", operator("^", 7));
        let actual = end_formula(stack, "b");

        assert_eq!(
            actual,
            unary("-", 4, binary("^", 7, operand("a"), operand("b")),)
        );
    }
}
