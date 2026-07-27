//! Stateful, command-at-a-time Atlas execution.
//!
//! Atlas's lexer classifies input using state changed by prior commands. This
//! module therefore owns the outer loop and never pre-tokenizes a whole file.

use crate::{
    diagnostic::{Diagnostic, SourceSpan},
    eval::{evaluate_with_context, EvalContext, EvalEvent},
    lex::{Lexer, Token, TokenKind},
    source::SourceText,
    syntax::{parse_command, Program},
    value::Value,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SessionEvent {
    Value { value: Value, span: SourceSpan },
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
                execute_command(&mut command, source, context, &mut events);
            }
            Ok(token) if token.kind == TokenKind::Eof => {
                execute_command(&mut command, source, context, &mut events);
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

fn execute_command(
    tokens: &mut Vec<Token>,
    source: &SourceText,
    context: &mut EvalContext,
    events: &mut Vec<SessionEvent>,
) {
    if tokens.is_empty() {
        return;
    }

    let expression = match parse_command(tokens, source) {
        Ok(expression) => expression,
        Err(diagnostic) => {
            events.push(SessionEvent::Diagnostic(diagnostic));
            tokens.clear();
            return;
        }
    };
    tokens.clear();

    let program = Program {
        expressions: vec![expression],
    };
    match evaluate_with_context(&program, context) {
        Ok(command_events) => {
            events.extend(command_events.into_iter().map(|event| match event {
                EvalEvent::Value { value, span } => SessionEvent::Value { value, span },
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
}
