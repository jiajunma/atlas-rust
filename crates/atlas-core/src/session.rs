//! Stateful, command-at-a-time Atlas execution.
//!
//! Atlas's lexer classifies input using state changed by prior commands. This
//! module therefore owns the outer loop and never pre-tokenizes a whole file.

use crate::{
    diagnostic::{Diagnostic, SourceSpan},
    eval::{execute_command as evaluate_command, EvalContext, EvalEvent},
    lex::{Lexer, Token, TokenKind},
    source::SourceText,
    syntax::parse_command,
    value::Value,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SessionEvent {
    Value { value: Value, span: SourceSpan },
    Output { text: String, span: SourceSpan },
    Diagnostic(Diagnostic),
}

pub fn run_source(source: &SourceText) -> Vec<SessionEvent> {
    let mut context = EvalContext::new();
    run_source_with_context(source, &mut context)
}

pub fn run_source_with_context(
    source: &SourceText,
    context: &mut EvalContext,
) -> Vec<SessionEvent> {
    let mut lexer = Lexer::new(source);
    let mut command = Vec::new();
    let mut events = Vec::new();

    loop {
        match lexer.next_token() {
            Ok(token) if token.kind == TokenKind::Newline => {
                execute_tokens(&mut command, source, context, &mut events);
            }
            Ok(token) if token.kind == TokenKind::Eof => {
                execute_tokens(&mut command, source, context, &mut events);
                break;
            }
            Ok(token) if matches!(token.kind, TokenKind::Unsupported(_)) => {
                events.push(SessionEvent::Diagnostic(Diagnostic::new(
                    crate::diagnostic::ErrorKind::Syntax,
                    "unexpected token",
                    Some(token.span),
                )));
                command.clear();
                lexer.recover_command();
            }
            Ok(token) => command.push(token),
            Err(diagnostic) => events.push(SessionEvent::Diagnostic(diagnostic)),
        }
    }

    events
}

fn execute_tokens(
    tokens: &mut Vec<Token>,
    source: &SourceText,
    context: &mut EvalContext,
    events: &mut Vec<SessionEvent>,
) {
    if tokens.is_empty() {
        return;
    }

    let command = match parse_command(tokens, source) {
        Ok(command) => command,
        Err(diagnostic) => {
            events.push(SessionEvent::Diagnostic(diagnostic));
            tokens.clear();
            return;
        }
    };
    tokens.clear();

    match evaluate_command(&command, context) {
        Ok(command_events) => {
            events.extend(command_events.into_iter().map(|event| match event {
                EvalEvent::Value { value, span } => SessionEvent::Value { value, span },
                EvalEvent::Output { text, span } => SessionEvent::Output { text, span },
            }));
        }
        Err(diagnostic) => events.push(SessionEvent::Diagnostic(diagnostic)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{diagnostic::ErrorKind, source::SourceText, value::Value};

    #[test]
    fn preserves_events_and_continues_after_command_errors() {
        let source = SourceText::new(include_str!(
            "../../../tests/fixtures/commands/ordered_events.atlas"
        ));
        let events = run_source(&source);
        assert_eq!(events.len(), 4);
        assert!(matches!(
            events[0],
            SessionEvent::Value {
                value: Value::Integer(_),
                ..
            }
        ));
        assert!(
            matches!(events[1], SessionEvent::Diagnostic(ref d) if d.kind == ErrorKind::Runtime)
        );
        assert!(matches!(events[2], SessionEvent::Diagnostic(ref d) if d.kind == ErrorKind::Name));
        assert!(matches!(
            events[3],
            SessionEvent::Value {
                value: Value::Integer(_),
                ..
            }
        ));
    }

    #[test]
    fn invalid_token_rejects_only_its_command() {
        let source = SourceText::new(include_str!(
            "../../../tests/fixtures/commands/invalid_token_continues.atlas"
        ));
        let events = run_source(&source);
        assert_eq!(events.len(), 2);
        assert!(
            matches!(events[0], SessionEvent::Diagnostic(ref d) if d.kind == ErrorKind::Syntax)
        );
        assert!(matches!(
            events[1],
            SessionEvent::Value {
                value: Value::Integer(_),
                ..
            }
        ));
    }

    #[test]
    fn mismatched_delimiter_rejects_only_its_command() {
        let source = SourceText::new(include_str!(
            "../../../tests/fixtures/commands/mismatched_delimiter_continues.atlas"
        ));
        let events = run_source(&source);
        assert_eq!(events.len(), 2);
        assert!(
            matches!(events[0], SessionEvent::Diagnostic(ref d) if d.kind == ErrorKind::Syntax)
        );
        assert!(matches!(
            events[1],
            SessionEvent::Value {
                value: Value::Integer(_),
                ..
            }
        ));
    }

    #[test]
    fn nested_invalid_token_does_not_swallow_the_next_physical_line() {
        let source = SourceText::new(include_str!(
            "../../../tests/fixtures/commands/nested_invalid_token_continues.atlas"
        ));
        let events = run_source(&source);
        assert_eq!(events.len(), 2);
        assert!(
            matches!(events[0], SessionEvent::Diagnostic(ref d) if d.kind == ErrorKind::Syntax)
        );
        assert!(matches!(
            events[1],
            SessionEvent::Value {
                value: Value::Integer(_),
                ..
            }
        ));
    }

    #[test]
    fn definitions_and_assignments_persist_across_commands() {
        let source = SourceText::new(include_str!(
            "../../../tests/fixtures/commands/assignments.atlas"
        ));
        let events = run_source(&source);

        assert_eq!(events.len(), 6);
        assert!(matches!(
            events[0],
            SessionEvent::Output { ref text, .. } if text == "Variable x: int\n"
        ));
        let values: Vec<_> = events
            .iter()
            .filter_map(|event| match event {
                SessionEvent::Value { value, .. } => Some(value.clone()),
                SessionEvent::Output { .. } | SessionEvent::Diagnostic(_) => None,
            })
            .collect();
        assert_eq!(
            values,
            vec![
                Value::Integer(41.into()),
                Value::Integer(42.into()),
                Value::Integer(42.into()),
                Value::Integer(9.into()),
                Value::Integer(9.into()),
            ]
        );
    }

    #[test]
    fn failed_assignments_leave_existing_bindings_unchanged() {
        let source = SourceText::new(include_str!(
            "../../../tests/fixtures/commands/assignment_errors.atlas"
        ));
        let events = run_source(&source);

        assert_eq!(events.len(), 5);
        assert!(matches!(events[1], SessionEvent::Diagnostic(ref d) if d.kind == ErrorKind::Type));
        assert!(matches!(
            events[2],
            SessionEvent::Value { value: Value::Integer(ref value), .. } if value == &10.into()
        ));
        assert!(matches!(events[3], SessionEvent::Diagnostic(ref d) if d.kind == ErrorKind::Name));
        assert!(matches!(
            events[4],
            SessionEvent::Value { value: Value::Integer(ref value), .. } if value == &10.into()
        ));
    }

    #[test]
    fn nested_assignment_side_effects_follow_atlas_evaluation_order() {
        let source = SourceText::new(include_str!(
            "../../../tests/fixtures/commands/assignment_order.atlas"
        ));
        let events = run_source(&source);

        assert_eq!(events.len(), 7);
        assert!(matches!(
            events[2],
            SessionEvent::Value { value: Value::Integer(ref value), .. } if value == &3.into()
        ));
        assert!(matches!(
            events[3],
            SessionEvent::Value { value: Value::Integer(ref value), .. } if value == &6.into()
        ));
        assert!(
            matches!(events[4], SessionEvent::Diagnostic(ref d) if d.kind == ErrorKind::Runtime)
        );
        assert!(matches!(
            events[5],
            SessionEvent::Value { value: Value::Integer(ref value), .. } if value == &3.into()
        ));
        assert!(matches!(
            events[6],
            SessionEvent::Value { value: Value::Integer(ref value), .. } if value == &4.into()
        ));
    }

    #[test]
    fn primitive_declarations_are_uninitialized_until_assignment() {
        let source = SourceText::new(include_str!(
            "../../../tests/fixtures/commands/declarations.atlas"
        ));
        let events = run_source(&source);

        assert_eq!(events.len(), 13);
        assert!(matches!(
            events[0],
            SessionEvent::Output { ref text, .. } if text == "Declaring identifier 'x': int\n"
        ));
        assert!(
            matches!(events[1], SessionEvent::Diagnostic(ref d) if d.kind == ErrorKind::Runtime)
        );
        assert!(matches!(
            events[2],
            SessionEvent::Value { value: Value::Integer(ref value), .. } if value == &3.into()
        ));
        assert!(matches!(
            events[4],
            SessionEvent::Output { ref text, .. } if text == "Declaring identifier 'r': rat\n"
        ));
        assert!(matches!(
            events[5],
            SessionEvent::Value { value: Value::Rational(ref value), .. }
                if value == &num_rational::BigRational::from_integer(2.into())
        ));
        assert!(matches!(
            events[7],
            SessionEvent::Output { ref text, .. }
                if text == "Declaring identifier 's': string\n"
        ));
        assert!(matches!(
            events[8],
            SessionEvent::Value { value: Value::String(ref value), .. } if value == "atlas"
        ));
        assert!(matches!(
            events[10],
            SessionEvent::Output { ref text, .. }
                if text == "Declaring identifier 'b': bool\n"
        ));
        assert!(matches!(
            events[11],
            SessionEvent::Value {
                value: Value::Boolean(true),
                ..
            }
        ));
    }

    #[test]
    fn failed_assignment_leaves_a_declaration_uninitialized() {
        let source = SourceText::new(include_str!(
            "../../../tests/fixtures/commands/declaration_errors.atlas"
        ));
        let events = run_source(&source);

        assert_eq!(events.len(), 3);
        assert!(matches!(events[1], SessionEvent::Diagnostic(ref d) if d.kind == ErrorKind::Type));
        assert!(
            matches!(events[2], SessionEvent::Diagnostic(ref d) if d.kind == ErrorKind::Runtime)
        );
    }
}
