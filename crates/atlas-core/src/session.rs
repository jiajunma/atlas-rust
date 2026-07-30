//! Stateful, command-at-a-time Atlas execution.
//!
//! Atlas's lexer classifies input using state changed by prior commands. This
//! module therefore owns the outer loop and never pre-tokenizes a whole file.

use crate::{
    diagnostic::{Diagnostic, SourceSpan},
    lex::{Lexer, Token, TokenKind},
    source::SourceText,
    syntax::parse_command,
    typed::{TypedCommandEvent, TypedContext},
    value::Value,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SessionEvent {
    Value {
        value: Value,
        is_void_type: bool,
        span: SourceSpan,
    },
    Output {
        text: String,
        span: SourceSpan,
    },
    ReportLine {
        text: String,
        span: SourceSpan,
    },
    Diagnostic(Diagnostic),
}

pub fn run_source(source: &SourceText) -> Vec<SessionEvent> {
    let mut context = TypedContext::new();
    run_source_with_context(source, &mut context)
}

pub fn run_source_with_context(
    source: &SourceText,
    context: &mut TypedContext,
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
            Ok(token) if matches!(token.kind, TokenKind::Directive(_)) => {
                events.push(SessionEvent::Diagnostic(Diagnostic::new(
                    crate::diagnostic::ErrorKind::Io,
                    "file inclusion is only available through a session frame",
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

pub(crate) fn execute_tokens(
    tokens: &mut Vec<Token>,
    source: &SourceText,
    context: &mut TypedContext,
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

    match context.execute(&command) {
        Ok(command_events) => events.extend(command_events.into_iter().map(|event| match event {
            TypedCommandEvent::Value { value, type_, span } => SessionEvent::Value {
                value,
                is_void_type: type_.is_void(),
                span,
            },
            TypedCommandEvent::ReportLine { text, span } => SessionEvent::ReportLine { text, span },
        })),
        Err(diagnostic) => events.push(SessionEvent::Diagnostic(diagnostic)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{diagnostic::ErrorKind, source::SourceText, value::Value};

    #[test]
    fn kgb_pipeline_is_scriptable_end_to_end() {
        // The phase-1 gate: simply connected A1, equal-rank inner class,
        // external form order (compact = 0, split = 1), KGB observables.
        let source = SourceText::new(concat!(
            "ic : inner_class(simply_connected(Lie_type(\"A1\"), true), mat: [[1]])\n",
            "nr_of_real_forms(ic)\n",
            "KGB_size(real_form(ic, 0))\n",
            "KGB_size(real_form(ic, 1))\n",
            "status(0, KGB(real_form(ic, 1), 0))\n",
        ));
        let events = run_source(&source);
        let values: Vec<&Value> = events
            .iter()
            .filter_map(|event| match event {
                SessionEvent::Value { value, .. } => Some(value),
                _ => None,
            })
            .collect();
        assert_eq!(values[0], &Value::Integer(2.into()));
        assert_eq!(values[1], &Value::Integer(1.into()));
        assert_eq!(values[2], &Value::Integer(3.into()));
        // Element 0 of the split form is noncompact imaginary: status 3.
        assert_eq!(values[3], &Value::Integer(3.into()));
    }

    #[test]
    fn sp4r_kgb_sizes_match_the_oracle_through_the_language() {
        let source = SourceText::new(concat!(
            "ic : inner_class(simply_connected(Lie_type(\"B2\"), true), mat: [[1,0],[0,1]])\n",
            "KGB_size(real_form(ic, 0))\n",
            "KGB_size(real_form(ic, 1))\n",
            "KGB_size(real_form(ic, 2))\n",
            "KGB(real_form(ic, 2), 10)\n",
            "Cayley(0, KGB(real_form(ic, 2), 0))\n",
        ));
        let events = run_source(&source);
        let mut values = events.iter().filter_map(|event| match event {
            SessionEvent::Value { value, .. } => Some(value),
            _ => None,
        });
        assert_eq!(values.next(), Some(&Value::Integer(1.into())));
        assert_eq!(values.next(), Some(&Value::Integer(4.into())));
        assert_eq!(values.next(), Some(&Value::Integer(11.into())));
        let element = values.next().expect("element value");
        assert_eq!(element.to_string(), "KGB element #10");
        let cayleyed = values.next().expect("cayley value");
        assert!(cayleyed.to_string().starts_with("KGB element #"));
    }

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
            SessionEvent::ReportLine { ref text, .. } if text == "Variable x: int\n"
        ));
        let values: Vec<_> = events
            .iter()
            .filter_map(|event| match event {
                SessionEvent::Value { value, .. } => Some(value.clone()),
                SessionEvent::Output { .. }
                | SessionEvent::ReportLine { .. }
                | SessionEvent::Diagnostic(_) => None,
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
            SessionEvent::Value { value: Value::Integer(ref value), .. } if value == &malachite::Integer::from(10)
        ));
        assert!(matches!(events[3], SessionEvent::Diagnostic(ref d) if d.kind == ErrorKind::Name));
        assert!(matches!(
            events[4],
            SessionEvent::Value { value: Value::Integer(ref value), .. } if value == &malachite::Integer::from(10)
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
            SessionEvent::Value { value: Value::Integer(ref value), .. } if value == &malachite::Integer::from(3)
        ));
        assert!(matches!(
            events[3],
            SessionEvent::Value { value: Value::Integer(ref value), .. } if value == &malachite::Integer::from(6)
        ));
        assert!(
            matches!(events[4], SessionEvent::Diagnostic(ref d) if d.kind == ErrorKind::Runtime)
        );
        assert!(matches!(
            events[5],
            SessionEvent::Value { value: Value::Integer(ref value), .. } if value == &malachite::Integer::from(3)
        ));
        assert!(matches!(
            events[6],
            SessionEvent::Value { value: Value::Integer(ref value), .. } if value == &malachite::Integer::from(4)
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
            SessionEvent::ReportLine { ref text, .. } if text == "Declaring identifier 'x': int\n"
        ));
        assert!(
            matches!(events[1], SessionEvent::Diagnostic(ref d) if d.kind == ErrorKind::Runtime)
        );
        assert!(matches!(
            events[2],
            SessionEvent::Value { value: Value::Integer(ref value), .. } if value == &malachite::Integer::from(3)
        ));
        assert!(matches!(
            events[4],
            SessionEvent::ReportLine { ref text, .. } if text == "Declaring identifier 'r': rat\n"
        ));
        assert!(matches!(
            events[5],
            SessionEvent::Value { value: Value::Rational(ref value), .. }
                if value == &malachite::Rational::from(2)
        ));
        assert!(matches!(
            events[7],
            SessionEvent::ReportLine { ref text, .. }
                if text == "Declaring identifier 's': string\n"
        ));
        assert!(matches!(
            events[8],
            SessionEvent::Value { value: Value::String(ref value), .. } if value == "atlas"
        ));
        assert!(matches!(
            events[10],
            SessionEvent::ReportLine { ref text, .. }
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

    #[test]
    fn let_bindings_shadow_without_leaking_local_assignments() {
        let source = SourceText::new(include_str!("../../../tests/fixtures/commands/let.atlas"));
        let events = run_source(&source);

        assert_eq!(events.len(), 12);
        assert!(matches!(
            events[0],
            SessionEvent::ReportLine { ref text, .. } if text == "Variable x: int\n"
        ));
        let values = events
            .iter()
            .filter_map(|event| match event {
                SessionEvent::Value { value, .. } => Some(value.clone()),
                SessionEvent::Output { .. }
                | SessionEvent::ReportLine { .. }
                | SessionEvent::Diagnostic(_) => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            values,
            vec![
                Value::Integer(3.into()),
                Value::Integer(10.into()),
                Value::Integer(4.into()),
                Value::Integer(11.into()),
                Value::Integer(10.into()),
                Value::Integer(3.into()),
                Value::Integer(7.into()),
                Value::Integer(11.into()),
                Value::Integer(2.into()),
                Value::Integer(2.into()),
                Value::Integer(10.into()),
            ]
        );
    }

    #[test]
    fn let_errors_do_not_create_or_mutate_global_bindings() {
        let source = SourceText::new(include_str!(
            "../../../tests/fixtures/commands/let_errors.atlas"
        ));
        let events = run_source(&source);

        assert_eq!(events.len(), 10);
        assert!(matches!(events[0], SessionEvent::Diagnostic(ref d) if d.kind == ErrorKind::Name));
        assert!(matches!(events[1], SessionEvent::Diagnostic(ref d) if d.kind == ErrorKind::Name));
        assert!(matches!(events[2], SessionEvent::Diagnostic(ref d) if d.kind == ErrorKind::Name));
        assert!(matches!(events[3], SessionEvent::Diagnostic(ref d) if d.kind == ErrorKind::Type));
        assert!(matches!(
            events[4],
            SessionEvent::ReportLine { ref text, .. } if text == "Variable x: int\n"
        ));
        assert!(matches!(
            events[5],
            SessionEvent::Value { value: Value::Integer(ref value), .. } if value == &malachite::Integer::from(3)
        ));
        assert!(matches!(
            events[6],
            SessionEvent::Value { value: Value::Integer(ref value), .. } if value == &malachite::Integer::from(10)
        ));
        assert!(
            matches!(events[7], SessionEvent::Diagnostic(ref d) if d.kind == ErrorKind::Runtime)
        );
        assert!(matches!(events[8], SessionEvent::Diagnostic(ref d) if d.kind == ErrorKind::Name));
        assert!(matches!(
            events[9],
            SessionEvent::Value { value: Value::Integer(ref value), .. } if value == &malachite::Integer::from(10)
        ));
    }

    #[test]
    fn let_validates_initializers_before_duplicate_binding_errors() {
        let source = SourceText::new(include_str!(
            "../../../tests/fixtures/commands/let_error_order.atlas"
        ));
        let events = run_source(&source);

        assert_eq!(events.len(), 1);
        let SessionEvent::Diagnostic(diagnostic) = &events[0] else {
            panic!("expected a diagnostic");
        };
        assert_eq!(diagnostic.kind, ErrorKind::Name);
        assert_eq!(diagnostic.message, "Undefined identifier 'missing'");
        assert_eq!(diagnostic.span.map(|span| span.start.column), Some(9));
    }

    #[test]
    fn container_errors_recover_at_command_boundaries() {
        let source = SourceText::new(include_str!(
            "../../../tests/fixtures/eval/container_errors.atlas"
        ));
        let events = run_source(&source);

        assert_eq!(events.len(), 4);
        assert!(matches!(
            events[0],
            SessionEvent::Diagnostic(ref d) if d.kind == ErrorKind::Type
        ));
        assert!(matches!(
            events[1],
            SessionEvent::Diagnostic(ref d) if d.kind == ErrorKind::Type
        ));
        assert!(matches!(
            events[2],
            SessionEvent::Diagnostic(ref d) if d.kind == ErrorKind::Type
        ));
        assert!(matches!(
            events[3],
            SessionEvent::Diagnostic(ref d) if d.kind == ErrorKind::Runtime
        ));
    }

    #[test]
    fn nested_container_assignments_coerce_recursively() {
        let source = SourceText::new(include_str!(
            "../../../tests/fixtures/commands/container_assignments.atlas"
        ));
        let events = run_source(&source);

        assert_eq!(events.len(), 6);
        assert!(matches!(
            events[0],
            SessionEvent::ReportLine { ref text, .. } if text == "Variable nested: [[rat]]\n"
        ));
        assert!(matches!(
            events[1],
            SessionEvent::Value { value: Value::List(ref values), .. }
                if values == &vec![Value::List(vec![Value::Rational(malachite::Rational::from(1))])]
        ));
        assert!(matches!(
            events[2],
            SessionEvent::Value { value: Value::List(ref values), .. }
                if values == &vec![Value::List(vec![Value::Rational(malachite::Rational::from(1))])]
        ));
        assert!(matches!(
            events[3],
            SessionEvent::ReportLine { ref text, .. } if text == "Variable pairs: [(int,rat)]\n"
        ));
        assert!(matches!(
            events[4],
            SessionEvent::Value { value: Value::List(ref values), .. }
                if values == &vec![Value::Tuple(vec![
                    Value::Integer(malachite::Integer::from(2)),
                    Value::Rational(malachite::Rational::from(3)),
                ])]
        ));
        assert!(matches!(
            events[5],
            SessionEvent::Value { value: Value::List(ref values), .. }
                if values == &vec![Value::Tuple(vec![
                    Value::Integer(malachite::Integer::from(2)),
                    Value::Rational(malachite::Rational::from(3)),
                ])]
        ));
    }

    #[test]
    fn subscription_errors_recover_at_command_boundaries() {
        let source = SourceText::new(include_str!(
            "../../../tests/fixtures/commands/subscription_errors.atlas"
        ));
        let events = run_source(&source);

        assert_eq!(events.len(), 12);
        for index in [0, 2, 4, 10] {
            assert!(matches!(
                events[index],
                SessionEvent::Diagnostic(ref diagnostic)
                    if diagnostic.kind == ErrorKind::Runtime
            ));
        }
        for index in [6, 8] {
            assert!(matches!(
                events[index],
                SessionEvent::Diagnostic(ref diagnostic)
                    if diagnostic.kind == ErrorKind::Type
            ));
        }
        for (index, expected) in [(1, 7), (3, 8), (5, 9), (7, 10), (9, 11), (11, 6)] {
            assert!(matches!(
                events[index],
                SessionEvent::Value { value: Value::Integer(ref value), .. }
                    if value == &malachite::Integer::from(expected)
            ));
        }
    }

    #[test]
    fn subscription_evaluates_index_before_row_expression() {
        let source = SourceText::new(include_str!(
            "../../../tests/fixtures/commands/subscription_order.atlas"
        ));
        let events = run_source(&source);

        assert_eq!(events.len(), 3);
        assert!(matches!(
            events[1],
            SessionEvent::Value { value: Value::Integer(ref value), .. }
                if value == &malachite::Integer::from(1)
        ));
        assert!(matches!(
            events[2],
            SessionEvent::Value { value: Value::Integer(ref value), .. }
                if value == &malachite::Integer::from(2)
        ));
    }

    #[test]
    fn slice_errors_recover_at_command_boundaries() {
        let source = SourceText::new(include_str!(
            "../../../tests/fixtures/commands/slice_errors.atlas"
        ));
        let events = run_source(&source);

        assert_eq!(events.len(), 12);
        for index in [0, 2, 4] {
            assert!(matches!(
                events[index],
                SessionEvent::Diagnostic(ref diagnostic)
                    if diagnostic.kind == ErrorKind::Runtime
            ));
        }
        for index in [6, 8, 10] {
            assert!(matches!(
                events[index],
                SessionEvent::Diagnostic(ref diagnostic)
                    if diagnostic.kind == ErrorKind::Type
            ));
        }
        for (index, expected) in [(1, 1), (3, 2), (5, 3), (7, 4), (9, 5), (11, 6)] {
            assert!(matches!(
                events[index],
                SessionEvent::Value { value: Value::Integer(ref value), .. }
                    if value == &malachite::Integer::from(expected)
            ));
        }
    }

    #[test]
    fn slice_evaluates_upper_then_lower_then_row_expression() {
        let source = SourceText::new(include_str!(
            "../../../tests/fixtures/commands/slice_order.atlas"
        ));
        let events = run_source(&source);

        assert_eq!(events.len(), 3);
        assert!(matches!(
            events[1],
            SessionEvent::Value { value: Value::List(ref values), .. }
                if values.is_empty()
        ));
        assert!(matches!(
            events[2],
            SessionEvent::Value { value: Value::Integer(ref value), .. }
                if value == &malachite::Integer::from(2034)
        ));
    }

    #[test]
    fn empty_row_subscription_flows_through_typed_assignment() {
        let source = SourceText::new(include_str!(
            "../../../tests/fixtures/commands/subscription_context.atlas"
        ));
        let events = run_source(&source);

        assert_eq!(events.len(), 15);
        assert!(matches!(
            events[0],
            SessionEvent::ReportLine { ref text, .. } if text == "Declaring identifier 'x': int\n"
        ));
        assert!(matches!(
            events[1],
            SessionEvent::Diagnostic(ref diagnostic)
                if diagnostic.kind == ErrorKind::Runtime
        ));
        assert!(matches!(
            events[2],
            SessionEvent::Value { value: Value::Integer(ref value), .. }
                if value == &malachite::Integer::from(1)
        ));
        assert!(matches!(
            events[3],
            SessionEvent::ReportLine { ref text, .. } if text == "Declaring identifier 'y': string\n"
        ));
        assert!(matches!(
            events[4],
            SessionEvent::Diagnostic(ref diagnostic)
                if diagnostic.kind == ErrorKind::Type
        ));
        assert!(matches!(
            events[5],
            SessionEvent::Value { value: Value::Integer(ref value), .. }
                if value == &malachite::Integer::from(2)
        ));
        assert!(matches!(
            events[6],
            SessionEvent::ReportLine { ref text, .. } if text == "Declaring identifier 'z': string\n"
        ));
        assert!(matches!(
            events[7],
            SessionEvent::Diagnostic(ref diagnostic)
                if diagnostic.kind == ErrorKind::Type
        ));
        assert!(matches!(
            events[8],
            SessionEvent::Value { value: Value::Integer(ref value), .. }
                if value == &malachite::Integer::from(3)
        ));
        assert!(matches!(
            events[9],
            SessionEvent::ReportLine { ref text, .. } if text == "Declaring identifier 'w': string\n"
        ));
        assert!(matches!(
            events[10],
            SessionEvent::Diagnostic(ref diagnostic)
                if diagnostic.kind == ErrorKind::Type
        ));
        assert!(matches!(
            events[11],
            SessionEvent::Value { value: Value::Integer(ref value), .. }
                if value == &malachite::Integer::from(4)
        ));
        assert!(matches!(
            events[12],
            SessionEvent::ReportLine { ref text, .. } if text == "Declaring identifier 'u': int\n"
        ));
        assert!(matches!(
            events[13],
            SessionEvent::Diagnostic(ref diagnostic)
                if diagnostic.kind == ErrorKind::Type
                    && diagnostic.message == "Failed to match '+' with argument type (*,*)"
        ));
        assert!(matches!(
            events[14],
            SessionEvent::Value { value: Value::Integer(ref value), .. }
                if value == &malachite::Integer::from(5)
        ));
    }

    #[test]
    fn malformed_container_commands_recover_without_swallowing_closed_lines() {
        let source = SourceText::new(include_str!(
            "../../../tests/fixtures/commands/container_syntax_errors.atlas"
        ));
        let events = run_source(&source);

        assert_eq!(events.len(), 7);
        assert!(
            matches!(events[0], SessionEvent::Diagnostic(ref d) if d.kind == ErrorKind::Syntax)
        );
        assert!(matches!(
            events[1],
            SessionEvent::Value { value: Value::Integer(ref value), .. }
                if value == &malachite::Integer::from(1)
        ));
        assert!(
            matches!(events[2], SessionEvent::Diagnostic(ref d) if d.kind == ErrorKind::Syntax)
        );
        assert!(matches!(
            events[3],
            SessionEvent::Value { value: Value::Integer(ref value), .. }
                if value == &malachite::Integer::from(2)
        ));
        assert!(
            matches!(events[4], SessionEvent::Diagnostic(ref d) if d.kind == ErrorKind::Syntax)
        );
        assert!(matches!(
            events[5],
            SessionEvent::Value { value: Value::Integer(ref value), .. }
                if value == &malachite::Integer::from(3)
        ));
        assert!(
            matches!(events[6], SessionEvent::Diagnostic(ref d) if d.kind == ErrorKind::Syntax)
        );
    }

    #[test]
    fn settype_b5_fixture_matches_the_frozen_events() {
        let source = SourceText::new(include_str!(
            "../../../tests/fixtures/eval/settype_b5.atlas"
        ));
        let events = run_source(&source);

        assert_eq!(events.len(), 12);
        assert!(matches!(
            events[0],
            SessionEvent::ReportLine { ref text, .. }
                if text == "Type name 'Pair' defined as (int,int)\n  with projectors: x, y.\n"
        ));
        assert!(matches!(
            events[1],
            SessionEvent::ReportLine { ref text, .. } if text == "Defined type: (int,int)\n"
        ));
        assert!(matches!(
            events[2],
            SessionEvent::ReportLine { ref text, .. } if text == "Type: (int,int)\n"
        ));
        assert!(matches!(
            events[3],
            SessionEvent::Value { value: Value::Integer(ref value), .. }
                if value == &malachite::Integer::from(1)
        ));
        assert!(matches!(
            events[4],
            SessionEvent::ReportLine { ref text, .. }
                if text == "Type name 'U' defined as (int|string)\n  with injectors: i, s.\n"
        ));
        assert!(matches!(
            events[5],
            SessionEvent::Value {
                value: Value::Union { tag: 0, ref injector_name, ref value },
                ..
            } if injector_name == "i"
                && matches!(value.as_ref(), Value::Integer(ref payload)
                    if payload == &malachite::Integer::from(3))
        ));
        assert!(matches!(
            events[6],
            SessionEvent::Value { value: Value::Integer(ref value), .. }
                if value == &malachite::Integer::from(4)
        ));
        assert!(matches!(
            events[7],
            SessionEvent::Value { value: Value::Integer(ref value), .. }
                if value == &malachite::Integer::from(2)
        ));
        assert!(matches!(
            events[8],
            SessionEvent::ReportLine { ref text, .. }
                if text == "Type name 'IntList' defined as (void|(int,IntList))\n  with injectors: nil, cons.\n"
        ));
        assert!(matches!(
            events[9],
            SessionEvent::ReportLine { ref text, .. }
                if text == "Defined type: ( void nil | (int,IntList) cons )\n"
        ));
        assert!(matches!(
            events[10],
            SessionEvent::Value { value: Value::Union { tag: 1, ref injector_name, .. }, .. }
                if injector_name == "cons"
        ));
        assert_eq!(
            match &events[10] {
                SessionEvent::Value { value, .. } => value.to_string(),
                other => panic!("expected a value event, got {other:?}"),
            },
            "(1,().nil).cons"
        );
        assert!(matches!(
            events[11],
            SessionEvent::ReportLine { ref text, .. } if text == "Type: IntList\n"
        ));
    }

    #[test]
    fn settype_b5_rejected_fixture_matches_the_frozen_events() {
        let source = SourceText::new(include_str!(
            "../../../tests/fixtures/eval/settype_b5_rejected.atlas"
        ));
        let events = run_source(&source);

        assert_eq!(events.len(), 5);
        assert!(matches!(
            events[0],
            SessionEvent::Diagnostic(ref diagnostic)
                if diagnostic.kind == ErrorKind::Syntax
                    && diagnostic.message == "syntax error, unexpected ':'"
        ));
        assert!(matches!(
            events[1],
            SessionEvent::ReportLine { ref text, .. }
                if text == "Type name 'U' defined as (int|string)\n  with injectors: i, s.\n"
        ));
        assert!(matches!(
            events[2],
            SessionEvent::Diagnostic(ref diagnostic)
                if diagnostic.kind == ErrorKind::Type
                    && diagnostic.message == "Discrimination on expression of type (int|string) requires using 'set_type' for this type, and naming injectors for it"
        ));
        assert!(matches!(
            events[3],
            SessionEvent::ReportLine { ref text, .. }
                if text == "Type name 'V' defined as (int|string)\n  with injectors: a, b.\n"
        ));
        assert!(matches!(
            events[4],
            SessionEvent::Diagnostic(ref diagnostic)
                if diagnostic.kind == ErrorKind::Type
                    && diagnostic.message == "found string while int was needed."
        ));
    }

    #[test]
    fn casefor_b6_fixture_matches_the_frozen_events() {
        let source = SourceText::new(include_str!(
            "../../../tests/fixtures/eval/casefor_b6.atlas"
        ));
        let events = run_source(&source);

        assert_eq!(events.len(), 11);
        let integers = vec![20, 10, 99, 77, 4];
        for (event, expected) in [&events[0], &events[1], &events[2], &events[3], &events[5]]
            .into_iter()
            .zip(integers)
        {
            assert!(matches!(
                event,
                SessionEvent::Value { value: Value::Integer(ref value), .. }
                    if value == &malachite::Integer::from(expected)
            ));
        }
        assert!(matches!(
            events[4],
            SessionEvent::ReportLine { ref text, .. }
                if text == "Type name 'U' defined as (int|string)\n  with injectors: i, s.\n"
        ));
        for (event, expected) in [
            (&events[6], "[0,1,2]"),
            (&events[7], "[40,50]"),
            (&events[8], "[3,2,1]"),
            (&events[9], "[7,7,7]"),
            (&events[10], "[1,2,3]"),
        ] {
            assert_eq!(
                match event {
                    SessionEvent::Value { value, .. } => value.to_string(),
                    other => panic!("expected a value event, got {other:?}"),
                },
                expected
            );
        }
    }

    #[test]
    fn casefor_b6_rejected_fixture_matches_the_frozen_events() {
        let source = SourceText::new(include_str!(
            "../../../tests/fixtures/eval/casefor_b6_rejected.atlas"
        ));
        let events = run_source(&source);

        assert_eq!(events.len(), 3);
        assert!(matches!(
            events[0],
            SessionEvent::ReportLine { ref text, .. }
                if text == "Type name 'U' defined as (int|string)\n  with injectors: i, s.\n"
        ));
        assert!(matches!(
            events[1],
            SessionEvent::Diagnostic(ref diagnostic)
                if diagnostic.kind == ErrorKind::Type
                    && diagnostic.message == "found int while (int->*) was needed."
        ));
        assert!(matches!(
            events[2],
            SessionEvent::Diagnostic(ref diagnostic)
                if diagnostic.kind == ErrorKind::Type
                    && diagnostic.message == "found string while int was needed."
        ));
    }

    #[test]
    fn commands_b7_fixture_matches_the_frozen_events() {
        let source = SourceText::new(include_str!(
            "../../../tests/fixtures/eval/commands_b7.atlas"
        ));
        let events = run_source(&source);

        assert_eq!(events.len(), 4);
        assert!(matches!(
            events[0],
            SessionEvent::ReportLine { ref text, .. } if text == "Identifier 'x' not known\n"
        ));
        assert!(matches!(
            events[1],
            SessionEvent::Value { value: Value::Integer(ref value), .. }
                if value == &malachite::Integer::from(42)
        ));
        assert!(matches!(
            events[2],
            SessionEvent::ReportLine { ref text, .. }
                if text == "Definition of '+@(int,int)' forgotten\n"
        ));
        // With the int+int overload forgotten, 1 + 2 resolves through the
        // int->rat coercion and yields the rational 3/1.
        assert!(matches!(
            events[3],
            SessionEvent::Value { value: Value::Rational(ref value), .. }
                if value == &malachite::Rational::from(3)
        ));
    }

    #[test]
    fn commands_b7_rejected_fixture_matches_the_frozen_events() {
        let source = SourceText::new(include_str!(
            "../../../tests/fixtures/eval/commands_b7_rejected.atlas"
        ));
        let events = run_source(&source);

        assert_eq!(events.len(), 2);
        assert!(matches!(
            events[0],
            SessionEvent::Diagnostic(ref diagnostic)
                if diagnostic.kind == ErrorKind::Runtime && diagnostic.message == "I die"
        ));
        assert!(matches!(
            events[1],
            SessionEvent::Diagnostic(ref diagnostic)
                if diagnostic.kind == ErrorKind::Name
                    && diagnostic.message == "Undefined identifier 'x'"
        ));
    }
}
