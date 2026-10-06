//! Stateful, command-at-a-time Atlas execution.
//!
//! Atlas's lexer classifies input using state changed by prior commands. This
//! module therefore owns the outer loop and never pre-tokenizes a whole file.

use crate::{
    diagnostic::{Diagnostic, SourceSpan},
    lex::{Lexer, Token, TokenKind},
    source::SourceText,
    syntax::parse_command_fragment_in,
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
    ReportBytes {
        text: crate::value::AtlasString,
        span: SourceSpan,
    },
    OutputBytes {
        text: crate::value::AtlasString,
        span: SourceSpan,
    },
    Diagnostic(Diagnostic),
}

impl SessionEvent {
    pub fn output(text: crate::value::AtlasString, span: SourceSpan) -> Self {
        match String::from_utf8(text.into_bytes()) {
            Ok(text) => Self::Output { text, span },
            Err(error) => Self::OutputBytes { text: error.into_bytes().into(), span },
        }
    }
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
        match next_session_token(&mut lexer, context) {
            Ok(token) if token.kind == TokenKind::Newline => {
                execute_tokens(&mut command, source, context, &mut events, Some(&token));
            }
            Ok(token) if token.kind == TokenKind::Eof => {
                execute_tokens(&mut command, source, context, &mut events, Some(&token));
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

/// Record identifiers at consumption time, not by pretokenizing a source:
/// included files and later commands must retain original first-use order.
pub(crate) fn next_session_token(
    lexer: &mut Lexer<'_>, context: &mut TypedContext,
) -> Result<Token, Diagnostic> {
    let token = lexer.next_token()?;
    context.note_completion_token(&token);
    Ok(token)
}

pub(crate) fn execute_tokens(
    tokens: &mut Vec<Token>,
    source: &SourceText,
    context: &mut TypedContext,
    events: &mut Vec<SessionEvent>,
    terminator: Option<&Token>,
) {
    if tokens.is_empty() {
        return;
    }

    let allow_more = terminator.is_some_and(|token| token.kind == TokenKind::Newline);
    let mut command = match parse_command_fragment_in(tokens, source, context.types(), allow_more) {
        Ok(Some(command)) => command,
        // Retain the prefix, with no evaluation or diagnostic. Both raw and
        // file-backed sessions append the next line to this same buffer.
        Ok(None) => return,
        Err(diagnostic) => {
            events.push(SessionEvent::Diagnostic(diagnostic));
            tokens.clear();
            return;
        }
    };
    // Original parser.y includes the command's newline in set_type's @$.
    // Use the actual lexer terminator (including trailing comments/spacing),
    // not a guessed extra column after the last semantic token. Atlas locates
    // a consumed newline on its old line, one column past the newline itself.
    if let (crate::syntax::Command::SetType { span, .. }, Some(end)) = (&mut command, terminator) {
        let end_position = crate::diagnostic::SourcePosition {
            line: end.span.start.line,
            column: end.span.start.column + usize::from(end.kind == TokenKind::Newline),
        };
        *span = SourceSpan::new(span.source_id(), span.byte_start(), end.span.byte_end(),
            span.start, end_position);
    }
    tokens.clear();

    match context.execute(&command) {
        Ok(command_events) => events.extend(command_events.into_iter().map(session_event)),
        Err(diagnostic) => {
            // Upstream's mid-evaluation stdout writes survive a runtime
            // error (ext_kl.cpp:947): drain them ahead of the diagnostic.
            let span = diagnostic.span;
            events.extend(
                context
                    .drain_failed_printed(span)
                    .into_iter()
                    .map(session_event),
            );
            events.push(SessionEvent::Diagnostic(diagnostic));
        }
    }
}

/// Lift one typed-command event to the session layer (the value/report/
/// output shapes are identical; only the value keeps its void flag).
fn session_event(event: TypedCommandEvent) -> SessionEvent {
    match event {
        TypedCommandEvent::Diagnostic(diagnostic) => SessionEvent::Diagnostic(diagnostic),
        TypedCommandEvent::Value { value, type_, span } => SessionEvent::Value {
            value,
            is_void_type: type_.is_void(),
            span,
        },
        TypedCommandEvent::ReportLine { text, span } => SessionEvent::ReportLine { text, span },
        TypedCommandEvent::ReportBytes { text, span } => SessionEvent::ReportBytes { text, span },
        TypedCommandEvent::Output { text, span } => SessionEvent::Output { text, span },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{diagnostic::ErrorKind, source::SourceText, value::Value};

    fn for_iteration_reports(source: &str) -> Vec<u8> {
        let events = run_source(&SourceText::new(source));
        assert!(!events.iter().any(|e| matches!(e, SessionEvent::Diagnostic(_))), "{events:?}");
        let mut output = Vec::new();
        for event in events {
            match event {
                SessionEvent::ReportLine { text, .. } => output.extend_from_slice(text.as_bytes()),
                SessionEvent::ReportBytes { text, .. } => output.extend_from_slice(text.as_bytes()),
                _ => {}
            }
        }
        output
    }

    #[test]
    fn weyl_subgroup_empty_invariant_is_checked_in_the_language() {
        // The original fails this invariant: an empty generating set fixes
        // every input. This expectation does not come from a patched oracle.
        let reports = for_iteration_reports(include_str!(
            "../../../tests/math/generics/weyl_subgroup_empty_regression.atlas"));
        let reports = String::from_utf8(reports).unwrap();
        for marker in [841, 842] {
            assert_eq!(reports.matches(&format!("RECOVER{marker}\n")).count(), 1);
        }
    }

    #[test]
    fn weyl_subgroup_rejects_invalid_generators_even_for_discarded_values() {
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/weyl_subgroup_orbit_rejected.atlas")));
        let errors: Vec<_> = events.iter().filter_map(|event| match event {
            SessionEvent::Diagnostic(error) => Some(error), _ => None,
        }).collect();
        let expected = [
            "Illegal root index 3", "Illegal root index -4",
            "Matrix for root indices is not a Cartan matrix: \n  [2,2|2,2]",
            "Matrix for root indices is not a Cartan matrix: \n  [2,-2|-2,2]",
            "Matrix for root indices is not a Cartan matrix: \n  [2,1|1,2]",
            "Illegal root index 3",
            "Matrix for root indices is not a Cartan matrix: \n  [2,2|2,2]",
            "Matrix for root indices is not a Cartan matrix: \n  [2,-2|-2,2]",
            "Integer value too big for conversion",
        ];
        assert_eq!(errors.len(), expected.len(), "{events:?}");
        for (error, message) in errors.iter().zip(expected) {
            assert_eq!(error.kind, ErrorKind::Runtime);
            assert_eq!(error.message, message);
        }
        let reports: String = events.iter().filter_map(|event| match event {
            SessionEvent::ReportLine { text, .. } => Some(text.as_str()), _ => None,
        }).collect();
        assert!(!reports.contains("WRONGLY_ACCEPTED"));
        for marker in 821..=829 {
            assert_eq!(reports.matches(&format!("RECOVER{marker}\n")).count(), 1);
        }
    }

    #[test]
    fn tagged_case_suffix_expressions_match_current_original() {
        // Original3842018 accepts this ENTIRE fixture; the earlier compound
        // while signals remain separate and are not accepted goldens.
        assert_eq!(for_iteration_reports(include_str!(
            "../../../tests/math/generics/tagged_case_suffix_expressions.atlas")),
            include_bytes!("../../../tests/math/generics/tagged_case_suffix_expressions.expected"));
    }

    #[test]
    fn tagged_case_coverage_checks_follow_original_analysis_order() {
        // Original3842032 rejects all six expressions; unchanged Rust
        // wrongly prints 1 for the first four and misses duplicate priority.
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/tagged_case_branch_coverage_rejected.atlas")));
        let errors: Vec<_> = events.iter().filter_map(|event| match event {
            SessionEvent::Diagnostic(error) => Some(error), _ => None,
        }).collect();
        let expected = [
            "Missing branch for variant coverage_b of type (int|int|void) in discrimination clause",
            "Multiple branches with label coverage_a",
            "Multiple default branches present",
            "Spurious default branch present",
            "Multiple branches with label coverage_a",
            "Undefined identifier 'coverage_undefined'",
        ];
        assert_eq!(errors.len(), expected.len(), "{events:?}");
        for (index, (error, message)) in errors.iter().zip(expected).enumerate() {
            assert_eq!(error.message, message);
            assert_eq!(error.kind, if index == 5 { ErrorKind::Name } else { ErrorKind::Program });
        }
        let reports: String = events.iter().filter_map(|event| match event {
            SessionEvent::ReportLine { text, .. } => Some(text.as_str()), _ => None,
        }).collect();
        assert!(!reports.lines().any(|line| line == "1"), "rejected case was evaluated: {reports}");
        for marker in 951..=956 {
            assert_eq!(reports.matches(&format!("RECOVER{marker}\n")).count(), 1);
        }
    }

    #[test]
    fn lie_type_extend_values_match_original() {
        // Original3840609: zero torus extension is the identity, Tn consists
        // of n T1 factors, and the G2 involution has no extra inner symbol.
        assert_eq!(for_iteration_reports(include_str!(
            "../../../tests/math/generics/lie_type_extend_values.atlas")),
            include_bytes!("../../../tests/math/generics/lie_type_extend_values.expected"));
    }

    #[test]
    fn lie_type_extend_rejects_before_discarding() {
        // Original3840609 rejects every one of these 17 calls. Unchanged
        // Rust accepts invalid factors/ranks and reaches both UNREACHABLEs.
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/lie_type_extend_rejected.atlas")));
        let errors: Vec<_> = events.iter().filter_map(|event| match event {
            SessionEvent::Diagnostic(error) => Some(error), _ => None,
        }).collect();
        let expected = [
            "Invalid type letter 'X'",
            "Invalid type letter 'a'",
            "Too small rank 0 for Lie type A",
            "Too small rank 1 for Lie type B",
            "Too small rank 1 for Lie type C",
            "Too small rank 3 for Lie type D",
            "Too small rank 5 for Lie type E",
            "Too large rank 9 for Lie type E",
            "Too large rank 5 for Lie type F",
            "Too large rank 3 for Lie type G",
            "Rank 33 exceeds implementation limit 32",
            "Total rank 33 exceeds implementation limit 32",
            "Rank 4294967295 exceeds implementation limit 32",
            "Integer value too big for conversion",
            "Integer value too big for conversion",
            "Too small rank 0 for Lie type G",
            "Invalid type letter 'X'",
        ];
        assert_eq!(errors.len(), expected.len(), "{events:?}");
        for (error, message) in errors.iter().zip(expected) {
            assert_eq!(error.kind, ErrorKind::Runtime);
            assert_eq!(error.message, message);
        }
        let reports: String = events.iter().filter_map(|event| match event {
            SessionEvent::ReportLine { text, .. } => Some(text.as_str()), _ => None,
        }).collect();
        assert_eq!(reports.as_bytes(), include_bytes!(
            "../../../tests/math/generics/lie_type_extend_rejected.expected"));
        assert!(!events.iter().any(|event| matches!(event,
            SessionEvent::Value { is_void_type: false, .. })), "{events:?}");
    }

    #[test]
    fn torus_radical_and_adjoint_bases_match_original() {
        // Original3840488: all 28 pure/product root data, both numberings
        // and lattices. Complete matrices, not only rank or determinant.
        assert_eq!(for_iteration_reports(include_str!(
            "../../../tests/math/generics/torus_radical_shapes.atlas")),
            include_bytes!("../../../tests/math/generics/torus_radical_shapes.expected"));
    }

    #[test]
    fn torus_radical_nonsymmetric_columns_match_original() {
        // Original3840498 reaches all 20 SC controls independently of the
        // old adjoint-torus rejection. In B2 a transpose retains the shape
        // but changes the actual basis; preserve every coefficient.
        assert_eq!(for_iteration_reports(include_str!(
            "../../../tests/math/generics/torus_radical_sc_values.atlas")),
            include_bytes!("../../../tests/math/generics/torus_radical_sc_values.expected"));
    }

    #[test]
    fn torus_radical_alcove_center_retains_central_character() {
        // Original3840498: with no roots, every central character remains
        // unchanged, for both lambda classes and seven signed nu values.
        assert_eq!(for_iteration_reports(include_str!(
            "../../../tests/math/generics/torus_alcove_center.atlas")),
            include_bytes!("../../../tests/math/generics/torus_alcove_center.expected"));
    }

    #[test]
    fn empty_root_datum_values_match_original() {
        // Original3840368 accepts every n-by-zero matrix pair (n=0/1/2/4),
        // retaining shape, duality and both root-numbering preferences.
        // Unchanged509-core rejects at the very first rank-zero constructor.
        assert_eq!(for_iteration_reports(include_str!(
            "../../../tests/math/generics/empty_root_datum_values.atlas")),
            include_bytes!("../../../tests/math/generics/empty_root_datum_values.expected"));
    }

    #[test]
    fn empty_root_datum_shapes_reject_before_discarding() {
        // Matrix-only companion to original3840368. Keep the separate
        // empty vector-list signal fixture out of rejection expectations.
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/empty_root_datum_shapes_rejected.atlas")));
        let errors: Vec<_> = events.iter().filter_map(|event| match event {
            SessionEvent::Diagnostic(error) => Some(error), _ => None,
        }).collect();
        let expected = [
            "Matrices of (co)roots give invalid Cartan matrix",
            "Sizes (0,1),(0,2) of simple (co)root systems differ",
            "Sizes (1,0),(2,0) of simple (co)root systems differ",
            "Sizes (0,0),(1,0) of simple (co)root systems differ",
            "Matrices of (co)roots give invalid Cartan matrix",
            "Matrices of (co)roots give invalid Cartan matrix",
            "Sizes (1,0),(2,0) of simple (co)root systems differ",
        ];
        assert_eq!(errors.len(), expected.len(), "{events:?}");
        for (error, message) in errors.iter().zip(expected) {
            assert_eq!(error.kind, ErrorKind::Runtime);
            assert_eq!(error.message, message);
        }
        let reports: String = events.iter().filter_map(|event| match event {
            SessionEvent::ReportLine { text, .. } => Some(text.as_str()), _ => None,
        }).collect();
        assert_eq!(reports, "RECOVER811\nRECOVER812\nRECOVER813\nRECOVER814\nRECOVER815\nRECOVER816\nRECOVER817\n");
        assert!(!events.iter().any(|event| matches!(event,
            SessionEvent::Value { is_void_type: false, .. })), "{events:?}");
    }

    #[test]
    fn signed_rational_coroot_expressions_reconstruct_every_root() {
        // Original3840264 accepts all 712 signed roots/coroots in 28 data.
        // The fixture checks both exact reconstructions, not just signs or
        // counts; the unchanged Rust fails at the first A2 negative coroot.
        let report = String::from_utf8(for_iteration_reports(include_str!(
            "../../../tests/math/generics/signed_coroot_expression.atlas"))).unwrap();
        assert_eq!(report.matches("SIGNED_ROOT_DATUM").count(), 28);
        assert_eq!(report.lines().filter(|line| line.starts_with("SIGNED_ROOT")
            && !line.starts_with("SIGNED_ROOT_DATUM")).count(), 712);
        assert!(report.ends_with("RECOVER787\n"));
    }

    #[test]
    fn signed_rational_alcove_centers_match_original() {
        // Original3840264: nu=-7/3 maps to -5/2, -1 stays -1,
        // and -1/3 maps to -1/2; retain zero/positive controls too.
        assert_eq!(for_iteration_reports(include_str!(
            "../../../tests/math/generics/signed_alcove_center.atlas")),
            include_bytes!("../../../tests/math/generics/signed_alcove_center.expected"));
    }

    #[test]
    fn projection_signed_echelon_matches_original() {
        // Original3840186: retain every basis, KType and Param report on
        // skew T2/A1.T2. Coset self-consistency alone passed with wrong lifts.
        assert_eq!(for_iteration_reports(include_str!(
            "../../../tests/math/generics/projection_signed_echelon.atlas")),
            include_bytes!("../../../tests/math/generics/projection_signed_echelon.expected"));
    }

    #[test]
    fn overload_void_redefinition_original() {
        // Original3856293 preserves the captured11 while replacing11 by23,
        // forgetting the sole active overload and then installing37.
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/overload_void_redefinition.atlas")));
        let output = matrix_axes_stdout(&events);
        let text = std::str::from_utf8(&output).unwrap();
        assert!(text.contains("OV0_FIRST|11|11\n"), "initial definition/capture failed");
        assert!(text.ends_with("OV0_RECOVERY|881\n"), "missing command recovery");
        let errors: Vec<_> = events.iter().filter_map(|event| match event {
            SessionEvent::Diagnostic(error) => Some(error), _ => None,
        }).collect();
        eprintln!("OVERLOAD_VOID_READY diagnostics={}", errors.len());
        let expected = include_bytes!("../../../tests/math/generics/overload_void_redefinition.oracle.stdout")
            .strip_prefix(b"MATH_BEGIN generic_overload_void_redefinition\n")
            .and_then(|body| body.strip_suffix(b"MATH_END generic_overload_void_redefinition\nBye.\n"))
            .expect("complete accepted original zero-argument overload capture");
        assert_eq!(output, expected, "complete original zero-argument overload history");
        assert!(errors.is_empty(), "unexpected diagnostics: {errors:?}");
    }

    #[test]
    fn overload_recursive_row_identity_original() {
        // Original3856744 accepts the complete history. The unchanged Rust
        // CLI exited139/signal11 before repair; keep this original golden.
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/overload_recursive_row_identity.atlas")));
        let output = matrix_axes_stdout(&events);
        let expected = include_bytes!("../../../tests/math/generics/overload_recursive_row_identity.oracle.stdout")
            .strip_prefix(b"MATH_BEGIN generic_overload_recursive_row_identity\n")
            .and_then(|body| body.strip_suffix(b"MATH_END generic_overload_recursive_row_identity\nBye.\n"))
            .expect("complete accepted original recursive-row capture");
        assert_eq!(output, expected, "complete original recursive-row overload history");
        assert!(!events.iter().any(|event| matches!(event, SessionEvent::Diagnostic(_))), "{events:?}");
    }

    fn full_deform_rank1_original_stream(source: &str, original: &[u8], name: &str, rows: usize, recovery: &str) {
        // Regression-first: retain every original split coefficient and cold/KL-warm
        // result. Matching ordinary KLV tables does not certify these Split terms.
        let events = run_source(&SourceText::new(source));
        let output = matrix_axes_stdout(&events);
        let text = std::str::from_utf8(&output).unwrap();
        for marker in ["FULL_DEFORM_ROW|", "FULL_DEFORM_COLD|", "FULL_DEFORM_KL|", "FULL_DEFORM_WARM|"] {
            assert_eq!(text.matches(marker).count(), rows, "incomplete full_deform rows: {marker}");
        }
        assert!(text.ends_with(recovery), "missing full_deform recovery");
        assert!(!events.iter().any(|event| matches!(event, SessionEvent::Diagnostic(_))), "{events:?}");
        eprintln!("FULL_DEFORM_RANK1_READY {name}");
        let begin = format!("MATH_BEGIN {name}\n");
        let end = format!("MATH_END {name}\nBye.\n");
        let expected = original.strip_prefix(begin.as_bytes())
            .and_then(|body| body.strip_suffix(end.as_bytes()))
            .expect("complete original full_deform capture");
        assert_eq!(output, expected, "complete original full_deform stream");
    }

    #[test]
    fn full_deform_rank1_control_original() {
        full_deform_rank1_original_stream(
            include_str!("../../../tests/math/progressive/full_deform_rank1_control.atlas"),
            include_bytes!("../../../tests/math/progressive/full_deform_rank1_control.oracle.stdout"),
            "full_deform_rank1_control", 4, "FULL_DEFORM_RECOVERY|677\n");
    }

    #[test]
    fn full_deform_rank1_split_original() {
        full_deform_rank1_original_stream(
            include_str!("../../../tests/math/progressive/full_deform_rank1_split.atlas"),
            include_bytes!("../../../tests/math/progressive/full_deform_rank1_split.oracle.stdout"),
            "full_deform_rank1_split", 12, "FULL_DEFORM_RECOVERY|683\n");
    }

    fn cartan_type_original_stream(source: &str, original: &[u8], name: &str, marker: &str, rows: usize, recovery: &str) {
        // Original3852252: canonical controls pass, but the old identity-only
        // map loses the Bourbaki-to-input permutation on relabelled diagrams.
        let events = run_source(&SourceText::new(source));
        let output = matrix_axes_stdout(&events);
        let text = std::str::from_utf8(&output).unwrap();
        assert_eq!(text.matches(marker).count(), rows, "incomplete Cartan rows");
        assert!(text.ends_with(recovery), "missing Cartan recovery");
        assert!(!events.iter().any(|event| matches!(event, SessionEvent::Diagnostic(_))), "{events:?}");
        eprintln!("CARTAN_TYPE_READY {name}");
        let begin = format!("MATH_BEGIN {name}\n");
        let end = format!("MATH_END {name}\nBye.\n");
        let expected = original.strip_prefix(begin.as_bytes())
            .and_then(|body| body.strip_suffix(end.as_bytes()))
            .expect("complete original Cartan capture");
        assert_eq!(output, expected, "complete original Cartan stream");
    }

    #[test]
    fn cartan_type_canonical_original() {
        cartan_type_original_stream(
            include_str!("../../../tests/math/generics/cartan_type_canonical.atlas"),
            include_bytes!("../../../tests/math/generics/cartan_type_canonical.oracle.stdout"),
            "cartan_type_canonical", "CARTAN_CANONICAL|", 11, "CARTAN_RECOVERY|601\n");
    }

    #[test]
    fn cartan_type_relabelled_original() {
        cartan_type_original_stream(
            include_str!("../../../tests/math/generics/cartan_type_relabelled.atlas"),
            include_bytes!("../../../tests/math/generics/cartan_type_relabelled.oracle.stdout"),
            "cartan_type_relabelled", "CARTAN_RELABELLED|", 7, "CARTAN_RECOVERY|607\n");
    }

    fn complex_rank_original_stream(source: &str, original: &[u8], name: &str, setup: &str, recovery: &str) {
        // Original3849710 accepts every SC/adjoint and numbering variant.
        // Rank4 is a passing control; rank5/6 expose the eight-piece Weyl
        // capacity. Check setup/recovery before diagnosing a failed run.
        let events = run_source(&SourceText::new(source));
        let output = matrix_axes_stdout(&events);
        let text = std::str::from_utf8(&output).unwrap();
        assert!(text.contains(setup), "missing setup: {text}");
        assert!(text.ends_with(recovery), "missing recovery: {text}");
        let expected = orientation_original_payload(original, name);
        let errors: Vec<_> = events.iter().filter_map(|event| match event {
            SessionEvent::Diagnostic(error) => Some((&error.kind, &error.message)), _ => None,
        }).collect();
        eprintln!("COMPLEX_RANK_READY {name}");
        assert!(errors.is_empty(), "COMPLEX_RANK_UNEXPECTED_DIAGNOSTICS {errors:?}");
        assert_eq!(output, expected);
    }

    #[test]
    fn complex_rank4_inventory_original() {
        complex_rank_original_stream(
            include_str!("../../../tests/math/generics/complex_rank4_inventory.atlas"),
            include_bytes!("../../../tests/math/generics/complex_rank4_inventory.oracle.stdout"),
            "complex_rank4_inventory", "COMPLEX_RANK4_SETUPfalsefalse\n",
            "COMPLEX_RANK4_RECOVER541\n");
    }

    #[test]
    fn complex_rank5_inventory_original() {
        complex_rank_original_stream(
            include_str!("../../../tests/math/generics/complex_rank5_inventory.atlas"),
            include_bytes!("../../../tests/math/generics/complex_rank5_inventory.oracle.stdout"),
            "complex_rank5_inventory", "COMPLEX_RANK5_SETUPfalsefalse\n",
            "COMPLEX_RANK5_RECOVER547\n");
    }

    #[test]
    fn complex_rank6_inventory_original() {
        complex_rank_original_stream(
            include_str!("../../../tests/math/generics/complex_rank6_inventory.atlas"),
            include_bytes!("../../../tests/math/generics/complex_rank6_inventory.oracle.stdout"),
            "complex_rank6_inventory", "COMPLEX_RANK6_SETUPfalsefalse\n",
            "COMPLEX_RANK6_RECOVER557\n");
    }

    fn psp4_cayley_original_stream(source: &str, original: &[u8], name: &str, markers: &[&str]) {
        // Original3848939 accepts these whole programs. Cold/triple-warm
        // controls pass even before repair; only the all-KL-term history
        // exposes the pooled-block failure. Preserve all terms and links.
        let events = run_source(&SourceText::new(source));
        let output = matrix_axes_stdout(&events);
        let text = std::str::from_utf8(&output).unwrap();
        for marker in markers {
            assert!(text.contains(marker), "missing setup/recovery {marker}: {text}");
        }
        let expected = orientation_original_payload(original, name);
        let errors: Vec<_> = events.iter().filter_map(|event| match event {
            SessionEvent::Diagnostic(error) => Some((&error.kind, &error.message)), _ => None,
        }).collect();
        eprintln!("PSP4_CAYLEY_READY {name}");
        assert!(errors.is_empty(), "PSP4_CAYLEY_UNEXPECTED_DIAGNOSTICS {errors:?}");
        assert_eq!(output, expected);
    }

    #[test]
    fn psp4_cayley_cold_original() {
        psp4_cayley_original_stream(
            include_str!("../../../tests/math/generics/psp4_cayley_bare_cold.atlas"),
            include_bytes!("../../../tests/math/generics/psp4_cayley_bare_cold.oracle.stdout"),
            "psp4_cayley_bare_cold",
            &["CAYLEY_COLD_INPUTdisconnected split real group with Lie algebra 'sp(4,R)'",
              "CAYLEY_COLD_RECOVER401\n", "CAYLEY_COLD_RECOVER409\n", "CAYLEY_COLD_RECOVER419\n",
              "CAYLEY_COLD_RECOVER421\n", "CAYLEY_COLD_END431\n"]);
    }

    #[test]
    fn psp4_cayley_triple_warm_original() {
        psp4_cayley_original_stream(
            include_str!("../../../tests/math/generics/psp4_cayley_bare_warm.atlas"),
            include_bytes!("../../../tests/math/generics/psp4_cayley_bare_warm.oracle.stdout"),
            "psp4_cayley_bare_warm",
            &["CAYLEY_WARM_INPUTdisconnected split real group with Lie algebra 'sp(4,R)'",
              "CAYLEY_WARM_RECOVER433\n", "CAYLEY_WARM_RECOVER439\n", "CAYLEY_WARM_RECOVER443\n",
              "CAYLEY_WARM_RECOVER449\n", "CAYLEY_WARM_END457\n"]);
    }

    #[test]
    fn psp4_cayley_kl_history_original() {
        psp4_cayley_original_stream(
            include_str!("../../../tests/math/generics/psp4_cayley_bare_kl_history.atlas"),
            include_bytes!("../../../tests/math/generics/psp4_cayley_bare_kl_history.oracle.stdout"),
            "psp4_cayley_bare_kl_history",
            &["CAYLEY_KL_HISTORY_INPUTdisconnected split real group with Lie algebra 'sp(4,R)'",
              "CAYLEY_KL_HISTORY_BEFOREfinal parameter(x=6,lambda=[4,3]/2,nu=[28,21]/6)",
              "CAYLEY_KL_HISTORY_RECOVER461\n", "CAYLEY_KL_HISTORY_RECOVER463\n",
              "CAYLEY_KL_HISTORY_RECOVER467\n", "CAYLEY_KL_HISTORY_RECOVER479\n",
              "CAYLEY_KL_HISTORY_RECOVER487\n", "CAYLEY_KL_HISTORY_END491\n"]);
    }

    #[test]
    fn component_assignment_live_destination_retains_nested_effects() {
        // Original3843926: preserve both assignments, RHS rebinding and
        // index-side writes on the same destination, including captured
        // locals. A pre-RHS aggregate snapshot loses these changes.
        assert_eq!(for_iteration_reports(include_str!(
            "../../../tests/math/generics/component_assignment_live_target.atlas")),
            include_bytes!("../../../tests/math/generics/component_assignment_live_target.expected"));
    }

    fn matrix_axes_stdout(events: &[SessionEvent]) -> Vec<u8> {
        let mut output = Vec::new();
        for event in events {
            match event {
                SessionEvent::ReportLine { text, .. } | SessionEvent::Output { text, .. } =>
                    output.extend_from_slice(text.as_bytes()),
                SessionEvent::ReportBytes { text, .. } | SessionEvent::OutputBytes { text, .. } =>
                    output.extend_from_slice(text.as_bytes()),
                SessionEvent::Value { value, is_void_type: false, .. } =>
                    output.extend_from_slice(format!("Value: {value}\n").as_bytes()),
                _ => {}
            }
        }
        output
    }

    #[test]
    fn matrix_axes_rectangular_values() {
        // Original3845762 accepts the ENTIRE builtin-only companion.
        // Preserve 2x3 reads, reversals, assignment/transform return values,
        // single-column selection, unchanged aliases and local mutations.
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/matrix_row_column_values_valid.atlas")));
        assert!(!events.iter().any(|e| matches!(e, SessionEvent::Diagnostic(_))), "{events:?}");
        assert_eq!(matrix_axes_stdout(&events), include_bytes!(
            "../../../tests/math/generics/matrix_row_column_values_valid.expected"));
    }

    #[test]
    fn matrix_axes_historical_unit_control() {
        // The historical typed unit expected transposed entries. Capture
        // the identical asymmetric2x2matrix/operation order on original
        // before correcting that unit; row1,col0 must stay3, not become9.
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/matrix_row_column_historical_unit.atlas")));
        assert!(!events.iter().any(|e| matches!(e, SessionEvent::Diagnostic(_))), "{events:?}");
        assert_eq!(matrix_axes_stdout(&events), include_bytes!(
            "../../../tests/math/generics/matrix_row_column_historical_unit.expected"));
    }

    #[test]
    fn matrix_axes_rejected_bounds_preserve_matrix() {
        // Original3845716/3845762: all eight checks fail, and NONE may
        // mutate the matrix. One Runtime label cannot detect lost errors.
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/matrix_row_column_rejected.atlas")));
        let errors: Vec<_> = events.iter().filter_map(|e| match e {
            SessionEvent::Diagnostic(error) => Some(error), _ => None,
        }).collect();
        assert_eq!(errors.len(), 8, "{events:?}");
        let mut messages = String::new();
        for error in errors {
            assert_eq!(error.kind, ErrorKind::Runtime, "{events:?}");
            // Only the subscription's source pretty-print differs: the
            // original wraps tuple indices in parentheses. Keep every
            // bound, operation, target and assignment value unchanged.
            messages.push_str(&error.message.replace("[2,0]", "[(2,0)]")
                .replace("[0,3]", "[(0,3)]").replace("[-1,0]", "[(-1,0)]"));
            messages.push('\n');
        }
        assert_eq!(messages, include_str!(
            "../../../tests/math/generics/matrix_row_column_rejected.messages"));
        assert_eq!(matrix_axes_stdout(&events), include_bytes!(
            "../../../tests/math/generics/matrix_row_column_rejected.expected"));
    }

    fn ordinary_deform_original_stream(source: &str, original: &[u8], group: &str) {
        // These files are the untouched, hashed original3845837 stdout,
        // not Rust-generated goldens. Strip only the capture envelope.
        let begin = format!("MATH_BEGIN generic_deform_common_block_{group}\n");
        let end = format!("MATH_END generic_deform_common_block_{group}\nBye.\n");
        let expected = original.strip_prefix(begin.as_bytes())
            .and_then(|body| body.strip_suffix(end.as_bytes()))
            .expect("complete original capture envelope");
        // Unlike the reports-only helper, retain the top-level row of
        // void values returned by the reducibility-point loop. It appears
        // in the untouched original stdout as `Value: [(),...]`.
        let events = run_source(&SourceText::new(source));
        assert!(!events.iter().any(|e| matches!(e, SessionEvent::Diagnostic(_))), "{events:?}");
        assert_eq!(matrix_axes_stdout(&events), expected);
    }

    fn orientation_original_payload<'a>(original: &'a [u8], name: &str) -> &'a [u8] {
        let begin = format!("MATH_BEGIN generic_{name}\n");
        let end = format!("MATH_END generic_{name}\nBye.\n");
        original.strip_prefix(begin.as_bytes())
            .and_then(|body| body.strip_suffix(end.as_bytes()))
            .expect("complete original orientation capture")
    }

    fn weyl_context_original_payload<'a>(original: &'a [u8], name: &str) -> &'a [u8] {
        let begin = format!("MATH_BEGIN {name}\n");
        let end = format!("MATH_END {name}\nBye.\n");
        original.strip_prefix(begin.as_bytes())
            .and_then(|body| body.strip_suffix(end.as_bytes()))
            .expect("complete original Weyl-context capture")
    }

    fn weyl_context_oracle_diagnostics(original: &[u8]) -> Vec<(ErrorKind, String)> {
        if original.is_empty() {
            return Vec::new();
        }
        let body = original.strip_suffix(b"\n")
            .expect("original Weyl-context stderr has a final newline");
        let lines = std::str::from_utf8(body)
            .expect("original Weyl-context stderr is UTF-8")
            .split('\n')
            .collect::<Vec<_>>();
        assert_eq!(lines.len() % 3, 0, "complete three-line diagnostic blocks");
        lines.chunks_exact(3).map(|block| {
            assert_eq!(block[0], "Runtime error:");
            assert_eq!(block[2], "Evaluation aborted.");
            let message = block[1].strip_prefix("  ")
                .expect("indented original runtime message");
            (ErrorKind::Runtime, message.to_owned())
        }).collect()
    }

    fn weyl_context_original_regression(
        source: &str,
        original_stdout: &[u8],
        original_stderr: &[u8],
        name: &str,
        recovery: &str,
    ) {
        let events = run_source(&SourceText::new(source));
        let output = matrix_axes_stdout(&events);
        let recovery_count = std::str::from_utf8(&output)
            .expect("Weyl-context output is UTF-8")
            .matches(recovery)
            .count();
        assert_eq!(recovery_count, 1, "Weyl-context setup did not recover: {events:?}");
        let diagnostics = events.iter().filter_map(|event| match event {
            SessionEvent::Diagnostic(error) => {
                Some((error.kind.clone(), error.message.clone()))
            }
            _ => None,
        }).collect::<Vec<_>>();
        let expected_output = weyl_context_original_payload(original_stdout, name).to_vec();
        let expected_diagnostics = weyl_context_oracle_diagnostics(original_stderr);
        eprintln!("WEYL_CONTEXT_CORE_READY {name} diagnostics={}", diagnostics.len());
        assert_eq!(
            (output, diagnostics),
            (expected_output, expected_diagnostics),
            "complete original Weyl-context stdout and ordered diagnostics",
        );
    }

    #[test]
    fn weyl_context_core_cold_dual_original() {
        weyl_context_original_regression(
            include_str!("../../../tests/math/generics/weyl_context_core_cold_dual.atlas"),
            include_bytes!(
                "../../../tests/math/generics/weyl_context_core_cold_dual.oracle.stdout"),
            include_bytes!(
                "../../../tests/math/generics/weyl_context_core_cold_dual.oracle.stderr"),
            "weyl_context_core_cold_dual",
            "WC_RECOVERY|727\n",
        );
    }

    #[test]
    fn weyl_context_core_prewarmed_dual_original() {
        weyl_context_original_regression(
            include_str!(
                "../../../tests/math/generics/weyl_context_core_prewarmed_dual.atlas"),
            include_bytes!(
                "../../../tests/math/generics/weyl_context_core_prewarmed_dual.oracle.stdout"),
            include_bytes!(
                "../../../tests/math/generics/weyl_context_core_prewarmed_dual.oracle.stderr"),
            "weyl_context_core_prewarmed_dual",
            "WCN_RECOVERY|733\n",
        );
    }

    #[test]
    fn root_ladder_coordinate_boundary_original() {
        // Unchanged original3868832 full stream:11 data,22 root/coroot
        // ladders, both numbering choices, dual data and recovery.
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/root_ladder_coordinate_boundary.atlas")));
        assert!(!events.iter().any(|e| matches!(e, SessionEvent::Diagnostic(_))), "{events:?}");
        assert_eq!(matrix_axes_stdout(&events), orientation_original_payload(include_bytes!(
            "../../../tests/math/generics/root_ladder_coordinate_boundary.oracle.stdout"),
            "root_ladder_coordinate_boundary"));
    }

    fn completion_snapshot_original(source: &str, original: &[u8], kind: &str) {
        // Entire original3868572 stdout, unchanged including declarations,
        // startup inventory, recovery and order. Only remove the capture frame.
        let events = run_source(&SourceText::new(source));
        let errors: Vec<_> = events.iter().filter_map(|event| match event {
            SessionEvent::Diagnostic(error) => Some(error), _ => None,
        }).collect();
        match kind {
            "types" => {
                // Keep the invalid >= constructor draft as a rejection case;
                // its accepted alias/member/recovery outputs must still agree.
                assert_eq!(errors.len(), 1, "{events:?}");
                assert_eq!(errors[0].kind, ErrorKind::Syntax);
                assert_eq!(errors[0].message, "syntax error, unexpected OPERATOR");
            }
            "rejected" => {
                assert_eq!(errors.len(), 3, "{events:?}");
                for (error, (kind, message)) in errors.iter().zip([
                    (ErrorKind::Type, "found int while string was needed."),
                    (ErrorKind::Runtime, "I die"),
                    (ErrorKind::Type, "found int while bool was needed."),
                ]) {
                    assert_eq!(error.kind, kind);
                    assert_eq!(error.message, message);
                }
            }
            _ => assert!(errors.is_empty(), "{events:?}"),
        }
        let name = format!("completion_incremental_{kind}");
        let expected = orientation_original_payload(original, &name);
        eprintln!("COMPLETION_SNAPSHOT_READY {kind}");
        assert_eq!(matrix_axes_stdout(&events), expected);
    }

    #[test]
    fn completion_snapshot_history_original() {
        completion_snapshot_original(include_str!(
            "../../../tests/math/generics/completion_incremental_history.atlas"),
            include_bytes!("../../../tests/math/generics/completion_incremental_history.oracle.stdout"), "history");
    }

    #[test]
    fn completion_snapshot_visibility_original() {
        completion_snapshot_original(include_str!(
            "../../../tests/math/generics/completion_incremental_visibility.atlas"),
            include_bytes!("../../../tests/math/generics/completion_incremental_visibility.oracle.stdout"), "visibility");
    }

    #[test]
    fn completion_snapshot_types_recovery_original() {
        completion_snapshot_original(include_str!(
            "../../../tests/math/generics/completion_incremental_types.atlas"),
            include_bytes!("../../../tests/math/generics/completion_incremental_types.oracle.stdout"), "types");
    }

    #[test]
    fn completion_snapshot_rejected_original() {
        completion_snapshot_original(include_str!(
            "../../../tests/math/generics/completion_incremental_rejected.atlas"),
            include_bytes!("../../../tests/math/generics/completion_incremental_rejected.oracle.stdout"), "rejected");
    }

    fn overload_command_original(source: &str, original: &[u8], rejected: bool) {
        // Untouched original3868735 stdout, including all declarations and
        // recovery. Error envelopes differ; preserve the two concrete causes.
        let events = run_source(&SourceText::new(source));
        let errors: Vec<_> = events.iter().filter_map(|event| match event {
            SessionEvent::Diagnostic(error) => Some(error), _ => None,
        }).collect();
        let kind = if rejected {
            assert_eq!(errors.len(), 2, "{events:?}");
            for (error, (kind, message)) in errors.iter().zip([
                (ErrorKind::Type, "Failed to match '+' with argument type (int,bool)"),
                (ErrorKind::Name, "Undefined identifier 'oc3n_dispatch'"),
            ]) {
                assert_eq!(error.kind, kind);
                assert_eq!(error.message, message);
            }
            "rejected"
        } else {
            assert!(errors.is_empty(), "{events:?}");
            "history"
        };
        let name = format!("overload_command_{kind}");
        assert_eq!(matrix_axes_stdout(&events), orientation_original_payload(original, &name));
    }

    #[test]
    fn overload_command_reuse_history_original() {
        overload_command_original(include_str!(
            "../../../tests/math/generics/overload_command_history.atlas"), include_bytes!(
            "../../../tests/math/generics/overload_command_history.oracle.stdout"), false);
    }

    #[test]
    fn overload_command_reuse_rejected_original() {
        overload_command_original(include_str!(
            "../../../tests/math/generics/overload_command_rejected.atlas"), include_bytes!(
            "../../../tests/math/generics/overload_command_rejected.oracle.stdout"), true);
    }

    #[test]
    fn orientation_partial_block_parameters() {
        // Original3846059 accepts every parameter and records its exact
        // orientation, including negative complex-root images in G2.
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/orientation_deformation_parameters_valid.atlas")));
        assert!(!events.iter().any(|e| matches!(e, SessionEvent::Diagnostic(_))), "{events:?}");
        assert_eq!(matrix_axes_stdout(&events), orientation_original_payload(include_bytes!(
            "../../../tests/math/generics/orientation_deformation_parameters_valid.oracle.stdout"),
            "orientation_deformation_parameters_valid"));
    }

    #[test]
    fn orientation_historical_anchors() {
        // Preserve the complete A1 real/compact and A2 complex anchors,
        // re-captured on the current original, not just historical numbers.
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/orientation_historical_anchors.atlas")));
        assert!(!events.iter().any(|e| matches!(e, SessionEvent::Diagnostic(_))), "{events:?}");
        assert_eq!(matrix_axes_stdout(&events), orientation_original_payload(include_bytes!(
            "../../../tests/math/generics/orientation_historical_anchors.oracle.stdout"),
            "orientation_historical_anchors"));
    }

    #[test]
    fn orientation_nonstandard_rejection_and_discard() {
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/orientation_nonstandard_rejected.atlas")));
        let errors: Vec<_> = events.iter().filter_map(|event| match event {
            SessionEvent::Diagnostic(error) => Some(error), _ => None,
        }).collect();
        assert_eq!(errors.len(), 2, "{events:?}");
        assert_eq!(errors[0].kind, ErrorKind::Runtime);
        assert_eq!(errors[0].message, "Non standard parameter in make_dominant");
        assert_eq!(errors[1].kind, ErrorKind::Type);
        assert_eq!(errors[1].message, "found [int] while Param was needed.");
        // A discarded call does NOT invoke orientation_number. Keep its
        // recovery and forbid the old spurious value on the rejected call.
        assert_eq!(matrix_axes_stdout(&events), orientation_original_payload(include_bytes!(
            "../../../tests/math/generics/orientation_nonstandard_rejected.oracle.stdout"),
            "orientation_nonstandard_rejected"));
    }

    #[test]
    fn ordinary_deform_g2_cross_original() {
        // Original3848811 accepts all four reducibility points, including
        // the non-normal 1/3 input. Retain every term, loop return value,
        // recovery and repeated direct calculation from its complete stream.
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/deform_cross_g2_bare.atlas")));
        let output = matrix_axes_stdout(&events);
        let text = std::str::from_utf8(&output).unwrap();
        for marker in [
            "DEFORM_CROSS_INPUTconnected split real group with Lie algebra 'g2(R)'",
            "POINTS[1/3,5/9,7/9,1/1]",
            "DEFORM_CROSS_BEFORE1/3non-normal parameter(x=7,lambda=[1,1]/1,nu=[0,1]/2)",
            "DEFORM_CROSS_RECOVER389\n", "DEFORM_CROSS_DIRECT\n", "DEFORM_CROSS_END397\n",
        ] {
            assert!(text.contains(marker), "missing setup/recovery marker {marker}: {text}");
        }
        let expected = orientation_original_payload(include_bytes!(
            "../../../tests/math/generics/deform_cross_g2_bare.oracle.stdout"),
            "deform_cross_g2_bare");
        let errors: Vec<_> = events.iter().filter_map(|event| match event {
            SessionEvent::Diagnostic(error) => Some((&error.kind, &error.message)), _ => None,
        }).collect();
        eprintln!("DEFORM_CROSS_READY g2");
        assert!(errors.is_empty(), "DEFORM_CROSS_UNEXPECTED_DIAGNOSTICS {errors:?}");
        assert_eq!(output, expected);
    }

    #[test]
    fn ordinary_deform_a2_complete_common_block_terms() {
        ordinary_deform_original_stream(
            include_str!("../../../tests/math/generics/deform_common_block_a2.atlas"),
            include_bytes!("../../../tests/math/generics/deform_common_block_a2.oracle.stdout"), "a2");
    }

    #[test]
    fn fundamental_lattice_broad_exact() {
        // Original3846359 accepts every D8/E8/SC/adjoint/torus/product
        // vector and the independent pairing, span and duality assertions.
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/fundamental_lattice_values_valid.atlas")));
        assert!(!events.iter().any(|e| matches!(e, SessionEvent::Diagnostic(_))), "{events:?}");
        assert_eq!(matrix_axes_stdout(&events), orientation_original_payload(include_bytes!(
            "../../../tests/math/generics/fundamental_lattice_values_valid.oracle.stdout"),
            "fundamental_lattice_values_valid"));
    }

    #[test]
    fn fundamental_lattice_embedded_exact() {
        // The full original stream includes explicit second-factor,
        // oblique, GL2, G2-Levi and BC-with-center coordinate anchors.
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/fundamental_lattice_embedded_valid.atlas")));
        assert!(!events.iter().any(|e| matches!(e, SessionEvent::Diagnostic(_))), "{events:?}");
        assert_eq!(matrix_axes_stdout(&events), orientation_original_payload(include_bytes!(
            "../../../tests/math/generics/fundamental_lattice_embedded_valid.oracle.stdout"),
            "fundamental_lattice_embedded_valid"));
    }

    #[test]
    fn fundamental_lattice_index_rejection_and_discard() {
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/fundamental_lattice_rejected.atlas")));
        let errors: Vec<_> = events.iter().filter_map(|event| match event {
            SessionEvent::Diagnostic(error) => Some(error), _ => None,
        }).collect();
        let expected = [
            "Invalid index -1", "Invalid index -1", "Invalid index 1",
            "Invalid index 1", "Invalid index 1", "Invalid index -1",
            "Integer value too big for conversion", "Integer value too big for conversion",
            "Integer value too big for conversion", "Integer value too big for conversion",
            "Invalid index 0", "Invalid index 0",
            "found (RootDatum,[int]) while (RootDatum,int) was needed.",
            "found ([int],int) while (RootDatum,int) was needed.",
        ];
        assert_eq!(errors.len(), expected.len(), "{events:?}");
        for (index, (error, message)) in errors.iter().zip(expected).enumerate() {
            assert_eq!(error.kind, if index < 12 { ErrorKind::Runtime } else { ErrorKind::Type });
            assert_eq!(error.message, message, "error {index}");
        }
        // Unlike orientation_nr, these wrappers validate even discarded
        // results. No WRONG_* survivor may appear in the complete stream.
        assert_eq!(matrix_axes_stdout(&events), orientation_original_payload(include_bytes!(
            "../../../tests/math/generics/fundamental_lattice_rejected.oracle.stdout"),
            "fundamental_lattice_rejected"));
    }

    #[test]
    fn gl2_twisted_kl_singular_original() {
        // Original3848669: retain the singular diagonal term, all four nu
        // values, both root numberings, and cold/repeated twisted histories.
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/gl2r_twisted_kl_bare_valid.atlas")));
        assert!(!events.iter().any(|e| matches!(e, SessionEvent::Diagnostic(_))), "{events:?}");
        let expected = orientation_original_payload(include_bytes!(
            "../../../tests/math/generics/gl2r_twisted_kl_bare_valid.oracle.stdout"),
            "gl2r_twisted_kl_bare_valid");
        eprintln!("GL2_TWISTED_READY positive");
        assert_eq!(matrix_axes_stdout(&events), expected);
    }

    #[test]
    fn gl2_twisted_kl_recovery_original() {
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/gl2r_twisted_kl_bare_recovery.atlas")));
        let errors: Vec<_> = events.iter().filter_map(|event| match event {
            SessionEvent::Diagnostic(error) => Some(error), _ => None,
        }).collect();
        assert_eq!(errors.len(), 2, "{events:?}");
        for (error, message) in errors.iter().zip([
            "Rank mismatch: (2,2,1)", "Rank mismatch: (2,2,3)",
        ]) {
            assert_eq!(error.kind, ErrorKind::Runtime);
            assert_eq!(error.message, message);
        }
        let expected = orientation_original_payload(include_bytes!(
            "../../../tests/math/generics/gl2r_twisted_kl_bare_recovery.oracle.stdout"),
            "gl2r_twisted_kl_bare_recovery");
        eprintln!("GL2_TWISTED_READY negative");
        assert_eq!(matrix_axes_stdout(&events), expected);
    }

    #[test]
    fn fpp_product_order_original() {
        // Original3847592 accepts all eight product types, both numberings
        // and both isogenies. Preserve every vector, Weyl/shift witness,
        // multiplicity, returned void row and recovery line in that order.
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/fpp_product_order_valid.atlas")));
        assert!(!events.iter().any(|e| matches!(e, SessionEvent::Diagnostic(_))), "{events:?}");
        let expected = orientation_original_payload(include_bytes!(
            "../../../tests/math/generics/fpp_product_order_valid.oracle.stdout"),
            "fpp_product_order_valid");
        eprintln!("FPP_ORDER_READY positive");
        assert_eq!(matrix_axes_stdout(&events), expected);
    }

    #[test]
    fn fpp_product_recovery_original() {
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/fpp_product_rejected.atlas")));
        let errors: Vec<_> = events.iter().filter_map(|event| match event {
            SessionEvent::Diagnostic(error) => Some(error), _ => None,
        }).collect();
        let messages = [
            "Rank and rational weight size mismatch 4:2",
            "Rank and rational weight size mismatch 4:5",
            "Rational weight is not in fundamental alcove (coroot 0, value -1/100)",
            "Rational weight is not in fundamental alcove (coroot -6, value -3/1)",
        ];
        assert_eq!(errors.len(), messages.len(), "{events:?}");
        for (error, message) in errors.iter().zip(messages) {
            assert_eq!(error.kind, ErrorKind::Runtime);
            assert_eq!(error.message, message);
        }
        let expected = orientation_original_payload(include_bytes!(
            "../../../tests/math/generics/fpp_product_rejected.oracle.stdout"),
            "fpp_product_rejected");
        // Diagnostic prose is tracked separately. Keep all four recovery
        // markers and the final whole valid product result, not a subset.
        eprintln!("FPP_ORDER_READY negative");
        assert_eq!(matrix_axes_stdout(&events), expected);
    }

    #[test]
    fn torus_bitset_duplicates_original() {
        // Original3846570: repeated vector indices denote set union,
        // including duplicates at limb boundaries, never arithmetic carry.
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/bitset_duplicate_values.atlas")));
        assert!(!events.iter().any(|e| matches!(e, SessionEvent::Diagnostic(_))), "{events:?}");
        assert_eq!(matrix_axes_stdout(&events), orientation_original_payload(include_bytes!(
            "../../../tests/math/generics/bitset_duplicate_values.oracle.stdout"),
            "bitset_duplicate_values"));
    }

    #[test]
    fn torus_bitset_factors_original() {
        // The complete original stream pins ordering and excludes tori
        // in pure/mixed factors, both root numberings, and extend results.
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/simple_factors_torus_values.atlas")));
        assert!(!events.iter().any(|e| matches!(e, SessionEvent::Diagnostic(_))), "{events:?}");
        assert_eq!(matrix_axes_stdout(&events), orientation_original_payload(include_bytes!(
            "../../../tests/math/generics/simple_factors_torus_values.oracle.stdout"),
            "simple_factors_torus_values"));
    }

    #[test]
    fn torus_bitset_factor_rejections_original() {
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/simple_factors_rejected.atlas")));
        let errors: Vec<_> = events.iter().filter_map(|event| match event {
            SessionEvent::Diagnostic(error) => Some(error), _ => None,
        }).collect();
        // String->LieType is a VALID implicit conversion in the original;
        // only integer and row arguments reject. Keep its Value: [] line.
        assert_eq!(errors.len(), 2, "{events:?}");
        for (error, expected) in errors.iter().zip([
            "found int while LieType was needed.",
            "found [LieType] while LieType was needed.",
        ]) {
            assert_eq!(error.kind, ErrorKind::Type);
            assert_eq!(error.message, expected);
        }
        assert_eq!(matrix_axes_stdout(&events), orientation_original_payload(include_bytes!(
            "../../../tests/math/generics/simple_factors_rejected.oracle.stdout"),
            "simple_factors_rejected"));
    }

    #[test]
    fn ordinary_deform_b2_complete_common_block_terms() {
        ordinary_deform_original_stream(
            include_str!("../../../tests/math/generics/deform_common_block_b2.atlas"),
            include_bytes!("../../../tests/math/generics/deform_common_block_b2.oracle.stdout"), "b2");
    }

    #[test]
    fn ordinary_deform_c2_complete_common_block_terms() {
        ordinary_deform_original_stream(
            include_str!("../../../tests/math/generics/deform_common_block_c2.atlas"),
            include_bytes!("../../../tests/math/generics/deform_common_block_c2.oracle.stdout"), "c2");
    }

    #[test]
    fn ordinary_deform_g2_complete_common_block_terms() {
        ordinary_deform_original_stream(
            include_str!("../../../tests/math/generics/deform_common_block_g2.atlas"),
            include_bytes!("../../../tests/math/generics/deform_common_block_g2.oracle.stdout"), "g2");
    }

    #[test]
    fn polynomial_coefficient_write_replace_insert_delete() {
        assert_eq!(for_iteration_reports(include_str!(
            "../../../tests/math/generics/polynomial_coefficient_assignment.atlas")),
            include_bytes!("../../../tests/math/generics/polynomial_coefficient_assignment.expected"));
    }

    #[test]
    fn polynomial_coefficient_write_nested_live_destinations() {
        assert_eq!(for_iteration_reports(include_str!(
            "../../../tests/math/generics/polynomial_coefficient_assignment_effects.atlas")),
            include_bytes!("../../../tests/math/generics/polynomial_coefficient_assignment_effects.expected"));
    }

    #[test]
    fn polynomial_coefficient_write_transform_evaluation_order() {
        // Original3844074: dynamic keys evaluate once BEFORE the operand;
        // bare identifier keys are read again AFTER the operand changes them.
        assert_eq!(for_iteration_reports(include_str!(
            "../../../tests/math/generics/polynomial_coefficient_assignment_transforms.atlas")),
            include_bytes!("../../../tests/math/generics/polynomial_coefficient_assignment_transforms.expected"));
    }

    #[test]
    fn polynomial_coefficient_write_boundaries_preserve_every_survivor() {
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/polynomial_coefficient_assignment_boundaries.atlas")));
        let errors: Vec<_> = events.iter().filter_map(|event| match event {
            SessionEvent::Diagnostic(error) => Some(error), _ => None,
        }).collect();
        let kfinal = "In coefficient assignment for KTypePol value:\n  non-standard K-type K_type(x=0, lambda=[-1]/1)\n  K-type is not dominant";
        let pfinal = "In coefficient assignment for ParamPol value:\n  non-standard parameter(x=0,lambda=[-1]/1,nu=[0]/1)\n  Parameter is not dominant";
        let expected = [
            (ErrorKind::Runtime, kfinal), (ErrorKind::Runtime, pfinal),
            (ErrorKind::Runtime, kfinal), (ErrorKind::Runtime, pfinal),
            (ErrorKind::Program, "Cannot subscript value of type KTypePol with index of type int in assignment"),
            (ErrorKind::Program, "Cannot subscript value of type ParamPol with index of type KType in assignment"),
            (ErrorKind::Type, "found string while Split was needed."),
            (ErrorKind::Type, "found string while Split was needed."),
            (ErrorKind::Program, "Cannot do reversed subscription of a KTypePol"),
        ];
        assert_eq!(errors.len(), expected.len(), "{events:?}");
        for (error, (kind, message)) in errors.iter().zip(expected) {
            assert_eq!(error.kind, kind, "{error:?}");
            assert!(error.message.contains(message), "{error:?}");
        }
        let reports: String = events.iter().filter_map(|event| match event {
            SessionEvent::ReportLine { text, .. } => Some(text.as_str()), _ => None,
        }).collect();
        assert_eq!(reports.as_bytes(), include_bytes!(
            "../../../tests/math/generics/polynomial_coefficient_assignment_boundaries.expected"));
        assert!(!events.iter().any(|event| matches!(event,
            SessionEvent::Value { is_void_type: false, .. })), "{events:?}");
    }

    #[test]
    fn polynomial_coefficient_write_foreign_owner_is_rejected_atomically() {
        // Explicit mathematical divergence: original3844074 reinterprets
        // foreign keys under the target owner, yielding KEY_RETAINEDfalse.
        // Reject both writes and retain the two empty target polynomials.
        let mut context = TypedContext::new();
        let events = run_source_with_context(&SourceText::new(include_str!(
            "../../../tests/math/generics/polynomial_coefficient_assignment_owner.atlas")), &mut context);
        let errors: Vec<_> = events.iter().filter_map(|event| match event {
            SessionEvent::Diagnostic(error) => Some(error), _ => None,
        }).collect();
        let expected = [
            "Real form mismatch when assigning coefficient of KTypePol value",
            "Real form mismatch when assigning coefficient of ParamPol value",
            "Real form mismatch when subscripting KTypePol value",
            "Real form mismatch when subscripting ParamPol value",
        ];
        assert_eq!(errors.len(), expected.len(), "{events:?}");
        for (error, message) in errors.iter().zip(expected) {
            assert_eq!(error.kind, ErrorKind::Runtime);
            assert_eq!(error.message, message);
        }
        let followup = run_source_with_context(&SourceText::new(
            "prints(\"OWNER_EMPTY\",#owner_Q,#owner_P)\n"), &mut context);
        assert!(!followup.iter().any(|e| matches!(e, SessionEvent::Diagnostic(_))), "{followup:?}");
        let reports: String = followup.iter().filter_map(|event| match event {
            SessionEvent::ReportLine { text, .. } => Some(text.as_str()), _ => None,
        }).collect();
        assert_eq!(reports, "OWNER_EMPTY00\n");
    }

    #[test]
    fn polynomial_coefficient_values_match_original() {
        // Original3840100: complete coefficients, missing-term zero, equal
        // real forms with distinct owners, dominant-copy lookup and ORDER12.
        assert_eq!(for_iteration_reports(include_str!(
            "../../../tests/math/generics/polynomial_coefficient_values.atlas")),
            include_bytes!("../../../tests/math/generics/polynomial_coefficient_values.expected"));
    }

    #[test]
    fn polynomial_coefficient_rejections_validate_discarded_reads() {
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/polynomial_coefficient_rejected.atlas")));
        let errors: Vec<_> = events.iter().filter_map(|event| match event {
            SessionEvent::Diagnostic(error) => Some(error), _ => None,
        }).collect();
        let kfinal = "In subscription of KTypePol value:\n  non-standard K-type K_type(x=0, lambda=[-1]/1)\n  K-type is not dominant";
        let pstandard = "In subscription of ParamPol value:\n  non-standard parameter(x=0,lambda=[-1]/1,nu=[0]/1)\n  Parameter not standard";
        let expected = [
            (ErrorKind::Runtime, "Real form mismatch when subscripting KTypePol value"),
            (ErrorKind::Runtime, "Real form mismatch when subscripting ParamPol value"),
            (ErrorKind::Runtime, kfinal), (ErrorKind::Runtime, pstandard),
            (ErrorKind::Runtime, kfinal), (ErrorKind::Runtime, pstandard),
            (ErrorKind::Program, "Cannot do reversed subscription of a KTypePol"),
            (ErrorKind::Program, "Cannot do reversed subscription of a ParamPol"),
            (ErrorKind::Program, "Cannot subscript value of type KTypePol with index of type int"),
            (ErrorKind::Program, "Cannot subscript value of type ParamPol with index of type KType"),
        ];
        assert_eq!(errors.len(), expected.len(), "{events:?}");
        for (error, (kind, message)) in errors.iter().zip(expected) {
            assert_eq!(error.kind, kind);
            assert_eq!(error.message, message);
        }
        let reports: String = events.iter().filter_map(|event| match event {
            SessionEvent::ReportLine { text, .. } => Some(text.as_str()), _ => None,
        }).collect();
        assert_eq!(reports, "Variable coeff_rd: RootDatum\nVariable coeff_rf: RealForm\nVariable coeff_split: RealForm\nVariable coeff_x: KGBElt\nVariable coeff_k: KType\nVariable coeff_p: Param\nVariable coeff_Q: KTypePol\nVariable coeff_P: ParamPol\nAFTER_K_FORM11\nAFTER_P_FORM13\nAFTER_K_FINAL17\nAFTER_P_STANDARD19\nAFTER_K_REVERSE31\nAFTER_P_REVERSE37\nAFTER_K_INDEX41\nAFTER_P_INDEX43\nRECOVER751\n");
        assert!(!events.iter().any(|event| matches!(event,
            SessionEvent::Value { is_void_type: false, .. })), "{events:?}");
    }

    #[test]
    fn ktype_formula_bounds_reject_before_discarding() {
        // Original3840093 rejects all six calls. The unchanged Rust wrongly
        // returned polynomials for both memo bounds and continued after a
        // discarded invalid call; missing raw names must not mask that defect.
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/ktype_formula_bounds_rejected.atlas")));
        let errors: Vec<_> = events.iter().filter_map(|event| match event {
            SessionEvent::Diagnostic(error) => Some(error), _ => None,
        }).collect();
        assert_eq!(errors.len(), 6, "{events:?}");
        for error in errors {
            assert_eq!(error.kind, ErrorKind::Runtime);
            assert_eq!(error.message, "Integer value too big for conversion");
        }
        // Successful recovery prints emit internal void Value events, which
        // the CLI suppresses. None of the six rejected calls may emit a
        // non-void polynomial (the unchanged before candidate did).
        assert!(!events.iter().any(|event| matches!(event,
            SessionEvent::Value { is_void_type: false, .. })), "{events:?}");
        let reports: Vec<_> = events.iter().filter_map(|event| match event {
            SessionEvent::ReportLine { text, .. } => Some(text.as_str()), _ => None,
        }).collect();
        assert_eq!(reports, ["Variable formula_rd: RootDatum\n", "Variable formula_G: RealForm\n",
            "Variable formula_k: KType\n", "AFTER_MEMO_HIGH11\n", "AFTER_MEMO_LOW13\n",
            "AFTER_RAW_HIGH17\n", "AFTER_RAW_LOW19\n", "RECOVER703\n"]);
    }

    #[test]
    fn ktype_formula_raw_and_memo_match_original() {
        // Complete mathematical report stream from original3840093, including
        // non-dominant inputs, distinct terms and decreasing/increasing bounds.
        assert_eq!(for_iteration_reports(include_str!(
            "../../../tests/math/generics/ktype_formula_bounds_values.atlas")),
            include_bytes!("../../../tests/math/generics/ktype_formula_bounds_values.expected"));
    }

    #[test]
    fn selector_unit_values_match_original() {
        // Complete report lines from original3840081. In ORDER821 the
        // selector-producing expression executes before its receiver.
        assert_eq!(for_iteration_reports(include_str!(
            "../../../tests/math/generics/selector_unit_values.atlas")),
            b"GROUPED7\nRECURSIVE120\nBEGIN5\nCONDITIONAL7\nCHAIN8\nVariable selector_trace: int\nORDER821\nDefined selector_words: ([[A]]->[[A]])\nWORDS[[1,3],[1,4],[2,3],[2,4]]\nSTRINGS[[\"a\",\"c\"],[\"b\",\"c\"]]\nEMPTY[][]\n");
    }

    #[test]
    fn selector_unit_rejections_reach_name_and_type_analysis() {
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/selector_unit_rejected.atlas")));
        let errors: Vec<_> = events.iter().filter_map(|event| match event {
            SessionEvent::Diagnostic(error) => Some(error), _ => None,
        }).collect();
        assert_eq!(errors.len(), 4, "{events:?}");
        assert_eq!(errors.iter().map(|error| error.kind.clone()).collect::<Vec<_>>(),
            vec![ErrorKind::Type, ErrorKind::Name, ErrorKind::Type, ErrorKind::Type]);
        assert!(errors[1].message.contains("Undefined identifier 'missing_selector'"));
        assert!(errors[2].message.contains("found int while string was needed."));
        assert!(errors[3].message.contains("found bool while int was needed."));
        let reports: Vec<_> = events.iter().filter_map(|event| match event {
            SessionEvent::ReportLine { text, .. } => Some(text.as_str()), _ => None,
        }).collect();
        assert_eq!(reports, ["AFTER_BOOLEAN11\n", "AFTER_NAME13\n",
                            "AFTER_ARGUMENT17\n", "AFTER_RESULT19\n"]);
    }

    #[test]
    fn f4_cartan_subsystem_types_preserve_root_numbering() {
        // Original corrective survey3840078 distinguishes real B2 under
        // coroot numbering from real C2 under root numbering. The fixture
        // asserts both, retaining SC/adjoint and both identity signs.
        let output = for_iteration_reports(include_str!(
            "../../../tests/math/generics/f4_cartan_numbering.atlas"));
        assert_eq!(output.split(|&byte| byte == b'\n')
            .filter(|line| line.starts_with(b"F4_CARTAN")).count(), 8);
    }

    #[test]
    fn named_update_values_match_original() {
        // Full report stream from original3839528, SHA
        // b6ec9ff7ced629dcd031b36fdc0b8a38e142add5089c66b1667e8b5f7d8684c6.
        assert_eq!(for_iteration_reports(include_str!(
            "../../../tests/math/generics/named_update_values.atlas")),
            b"Variable n: int\nAND2N2\nOR10N10\nDefined add_twice: (int,int->int)\nCUSTOM16N16\nVariable r: [int]\nROW3R[6,3,8]\nREVERSE9R[6,3,9]\nVariable v: vec\nVECTOR10V[ 10,  7,  8 ]\nType name 'UpdatePair' defined as (int,int)\n  with projectors: update_first, update_second.\nVariable p: UpdatePair\nFIELD2P(2,9)\nLOCAL7\nBITS[22,20,16]\nRECOVER677\n");
    }

    #[test]
    fn named_update_rejected_calls_preserve_targets_and_recover() {
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/named_update_rejected.atlas")));
        let errors: Vec<_> = events.iter().filter_map(|e| match e {
            SessionEvent::Diagnostic(d) => Some(d), _ => None,
        }).collect();
        assert_eq!(errors.len(), 5, "{events:?}");
        assert_eq!(errors[0].kind, ErrorKind::Name);
        assert!(errors[0].message.contains("Undefined identifier 'unknown_update'"));
        for error in &errors[1..3] {
            assert_eq!(error.kind, ErrorKind::Type);
            assert!(error.message.contains("found bool while int was needed."));
        }
        assert!(errors[3].message.contains("Name 'x' is constant in assignment x:=AND(x,0)"));
        assert_eq!(errors[4].kind, ErrorKind::Runtime);
        assert!(errors[4].message.contains("index 5 out of range (0<= . <2)"));
        let reports: Vec<_> = events.iter().filter_map(|e| match e {
            SessionEvent::ReportLine { text, .. } => Some(text.as_str()), _ => None,
        }).collect();
        assert_eq!(reports, ["Variable n: int\n", "RECOVER_A6\n",
            "Defined bad_update: (int,int->bool)\n", "RECOVER_B6\n",
            "Variable r: [int]\n", "RECOVER_C[6,7]\n", "RECOVER_D691\n",
            "RECOVER_E[6,7]\n"]);
    }

    #[test]
    fn integrality_subsystem_keeps_ambient_comparable_simple_roots() {
        // Original3839528 accepts; unchanged Rust reports rank1/A1.T1/true
        // and fails BOTH retained mathematical assertions in the fixture.
        assert_eq!(for_iteration_reports(include_str!(
            "../../../tests/math/generics/integrality_subsystem_boundary.atlas")),
            b"Variable rd: RootDatum\nVariable gamma: ratvec\nRANK2\nDATUMroot datum of Lie type 'A1.A1'\nCARTAN\n| 2, 0 |\n| 0, 2 |\n\nDOMINANTfalse\nRECOVER701\n");
    }

    #[test]
    fn integrality_subsystem_rank_errors_survive_discarding() {
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/integrality_rank_boundaries_rejected.atlas")));
        let errors: Vec<_> = events.iter().filter_map(|e| match e {
            SessionEvent::Diagnostic(d) => Some(d), _ => None,
        }).collect();
        assert_eq!(errors.len(), 5, "{events:?}");
        for (error, length) in errors.iter().zip([1,3,1,3,1]) {
            assert_eq!(error.kind, ErrorKind::Runtime);
            assert_eq!(error.message, format!(
                "Length {length} of rational vector differs from rank 2"));
        }
        let reports: Vec<_> = events.iter().filter_map(|e| match e {
            SessionEvent::ReportLine { text, .. } => Some(text.as_str()), _ => None,
        }).collect();
        assert_eq!(reports, ["Variable rd: RootDatum\n", "RECOVER709\n",
            "RECOVER719\n", "RECOVER727\n", "RECOVER733\n", "RECOVER739\n"]);
    }

    #[test]
    fn current_root_math_values_match_original_classical_and_exceptional() {
        // Original3839518 stdout SHA a19618e000ad9d1870f20a24026395afd167ce87701baaf9224714d44b89e9cd.
        // Preserve every DATUM/INTEGRAL/REFLECTION/RECOVER report line. The
        // independent HPC differential also compares the enclosing loop value.
        assert_eq!(for_iteration_reports(include_str!(
            "../../../tests/math/generics/current_root_math_values.atlas")),
            include_bytes!("../../../tests/math/generics/current_root_math_values.expected"));
    }

    #[test]
    fn current_root_math_rejections_validate_even_when_discarded() {
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/current_root_math_rejected.atlas")));
        let errors: Vec<_> = events.iter().filter_map(|e| match e {
            SessionEvent::Diagnostic(d) => Some(d), _ => None,
        }).collect();
        assert_eq!(errors.len(), 5, "{events:?}");
        let expected = [
            "Length 1 of rational vector differs from rank 2",
            "Length 3 of rational vector differs from rank 2",
            "Illegal root index 3", "Illegal root index -4",
            "Integer value too big for conversion",
        ];
        for (error, message) in errors.iter().zip(expected) {
            assert_eq!(error.kind, ErrorKind::Runtime);
            assert_eq!(error.message, message);
        }
        let reports: Vec<_> = events.iter().filter_map(|e| match e {
            SessionEvent::ReportLine { text, .. } => Some(text.as_str()), _ => None,
        }).collect();
        assert_eq!(reports, ["Variable rd: RootDatum\n", "RECOVER643\n",
            "RECOVER647\n", "RECOVER653\n", "RECOVER659\n", "RECOVER661\n",
            "VALID[0,1][0]\n"]);
    }

    #[test]
    fn iffor_values_match_original_singletons_filters_and_flattening() {
        // Whole accepted original3839396/3839480 stream.
        assert_eq!(for_iteration_reports(include_str!(
            "../../../tests/math/generics/iffor_values.atlas")),
            b"TRUE[7]\nFALSE[]\nFILTER[(10,0),(30,2)]\nNESTED[(1,0),(2,0),(2,1)]\nREVERSED[(0,0),(0,1),(1,0),(1,1),(2,0),(2,1)]\nIF_FOR[0,1,2]\nIF_IF[]\nCOUNT_FROM[5,7]\nANONYMOUS[7,7]\nEFFECTS2\nRECOVER503\n");
    }

    #[test]
    fn iffor_current_hidden_completion_inventory_matches_original() {
        assert_eq!(for_iteration_reports(include_str!(
            "../../../tests/math/generics/current_hidden_completion_names.atlas")),
            include_bytes!("../../../tests/math/generics/current_hidden_completion_names.expected"));
    }

    #[test]
    fn iffor_hidden_join_survives_visible_generic_override() {
        assert_eq!(for_iteration_reports(include_str!(
            "../../../tests/math/generics/iffor_hidden_join_generic.atlas")),
            b"Redefined ##: ([[A]]->[A])\nVISIBLE[]\nHIDDEN_JOIN[0,2]\nHIDDEN_NESTED[0,1,10,11]\nRECOVER617\n");
    }

    #[test]
    fn iffor_rejected_guards_bodies_and_contexts_still_recover() {
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/iffor_rejected.atlas")));
        let errors: Vec<_> = events.iter().filter_map(|e| match e {
            SessionEvent::Diagnostic(d) => Some(d), _ => None,
        }).collect();
        assert_eq!(errors.len(), 4, "{events:?}");
        assert_eq!(errors[0].message, "found int while bool was needed.");
        assert!(errors[1].message.contains("Undefined identifier 'unknown_iffor_value'"));
        assert_eq!(errors[2].message, "found bool while int was needed.");
        // The original rejects the singleton row against int; full error
        // presentation remains a separate differential gate.
        assert_eq!(errors[3].kind, ErrorKind::Type);
        assert!(errors[3].message.contains("int"));
        let reports: Vec<_> = events.iter().filter_map(|e| match e {
            SessionEvent::ReportLine { text, .. } => Some(text.as_str()), _ => None,
        }).collect();
        assert_eq!(reports, ["RECOVER_A521\n", "RECOVER_B523\n", "RECOVER_C541\n", "RECOVER_D547\n"]);
    }

    #[test]
    fn void_boundary_values_and_container_consumers_match_original() {
        // Whole accepted stream from original3839396, not a Rust golden.
        assert_eq!(for_iteration_reports(include_str!(
            "../../../tests/math/generics/void_value_boundaries.atlas")),
            b"SCALAR7\nROW[1,2]\nTUPLE(7,\"x\")\nCONDITIONAL7\nSEQUENCE11\nINFERRED_TUPLE(7,8)\nEXPLICIT_TUPLE((),8)\nINFERRED_ROW[()]\nEXPLICIT_ROW[(),()]\nVariable discarded: void\nGLOBAL42\nDefined f: (int->)\nCALL6\nRECOVER557\n");
    }

    #[test]
    fn void_boundary_loop_sequences_and_collected_bodies_match_original() {
        assert_eq!(for_iteration_reports(include_str!(
            "../../../tests/math/generics/for_void_scope_values.atlas")),
            b"CAST_SEQUENCE6\nCAST_LOOP6\nCOUNTED_SEQUENCE3\nCOUNTED_LOOP3\nINFERRED_BODY[(),()]\nEXPLICIT_BODY[(),()]\nRECOVER563\n");
    }

    #[test]
    fn void_boundary_named_loop_contexts_match_original() {
        // A named void context and a collected named-void body differ.
        // Original3839388 supplies both complete values.
        assert_eq!(for_iteration_reports(include_str!(
            "../../../tests/math/generics/for_named_void_context.atlas")),
            b"Type name 'MathLoopVoid' defined as void\nVariable named_loop: MathLoopVoid\nVariable named_body: [MathLoopVoid]\nNAMED_LOOP[0,1,2]\nNAMED_BODY[(),()]\nVOID_BRANCH3\nRECOVER461\n");
    }

    #[test]
    fn for_iteration_keeps_input_and_output_reversal_independent() {
        // Complete positive captured with the original in3839328.
        assert_eq!(for_iteration_reports(include_str!(
            "../../../tests/math/generics/for_reversal_values.atlas")),
            b"ROW[(30,2),(20,1),(10,0)]\nRESULT[(30,2),(20,1),(10,0)]\nBOTH[(10,0),(20,1),(30,2)]\nCOUNTED[2,1,0][2,1,0]\nFROM[7,6,5]\nDefined last_true: ([bool]->int)\nLAST2-1\nRECOVER229\n");
    }

    #[test]
    fn for_iteration_nonrow_values_preserve_columns_rationals_and_bytes() {
        assert_eq!(for_iteration_reports(include_str!(
            "../../../tests/math/generics/for_nonrow_values.atlas")),
            b"VECTOR[(10,0),(20,1),(30,2)]\nRATVECTOR[(1/2,0),(2/3,1),(3/4,2)]\nMATRIX[([ 1, 2 ],0),([ 3, 4 ],1),([ 5, 6 ],2)]\nSTRING[(1,0),(1,1),(1,2)]\nSTRING_RAW[\"\xc3\",\"\xa9\",\"Z\"]\nREVERSE[(30,2),(20,1),(10,0)]\nBOTH[(\"\xc3\",0),(\"\xa9\",1),(\"Z\",2)]\nEMPTY[][]\nRECOVER389\n");
    }

    #[test]
    fn for_iteration_polynomial_indices_are_domain_keys() {
        assert_eq!(for_iteration_reports(include_str!(
            "../../../tests/math/generics/for_polynomial_values.atlas")),
            b"Variable rd: RootDatum\nVariable ic: InnerClass\nVariable rf: RealForm\nVariable p: Param\nVariable W: ParamPol\nPARAM_TERMS[((1+0s),final parameter(x=0,lambda=[0,0]/1,nu=[0,0]/1))]\nPARAM_REVERSE[((1+0s),final parameter(x=0,lambda=[0,0]/1,nu=[0,0]/1))]\nVariable Q: KTypePol\nKTYPE_TERMS[((1+0s),final K-type K_type(x=0, lambda=[0,0]/1))]\nKTYPE_EMPTY[]\nPARAM_EMPTY[]\nRECOVER397\n");
    }

    #[test]
    fn for_iteration_distinct_polynomial_keys_keep_order_and_cancellation() {
        // Original3839480 confirms TWO nonzero compact-A1 terms, unlike
        // the retained split-A1 fixture that canonicalized to one key.
        assert_eq!(for_iteration_reports(include_str!(
            "../../../tests/math/generics/for_polynomial_distinct_keys.atlas")),
            b"Variable rd: RootDatum\nVariable ic: InnerClass\nVariable rf: RealForm\nVariable x: KGBElt\nVariable K2: KType\nVariable K4: KType\nVariable Q2: KTypePol\nVariable Q4: KTypePol\nVariable Q: KTypePol\nKCOUNT2\nKORDER[((1-1s),final K-type K_type(x=0, lambda=[3]/1)),((2+1s),final K-type K_type(x=0, lambda=[5]/1))]\nKREVERSE[((2+1s),final K-type K_type(x=0, lambda=[5]/1)),((1-1s),final K-type K_type(x=0, lambda=[3]/1))]\nKBOTH[((1-1s),final K-type K_type(x=0, lambda=[3]/1)),((2+1s),final K-type K_type(x=0, lambda=[5]/1))]\nKCANCEL[((1-1s),final K-type K_type(x=0, lambda=[3]/1))]\nVariable p2: Param\nVariable p4: Param\nVariable W2: ParamPol\nVariable W4: ParamPol\nVariable W: ParamPol\nPCOUNT2\nPORDER[((1-1s),final parameter(x=0,lambda=[3]/1,nu=[0]/1)),((2+1s),final parameter(x=0,lambda=[5]/1,nu=[0]/1))]\nPREVERSE[((2+1s),final parameter(x=0,lambda=[5]/1,nu=[0]/1)),((1-1s),final parameter(x=0,lambda=[3]/1,nu=[0]/1))]\nPBOTH[((1-1s),final parameter(x=0,lambda=[3]/1,nu=[0]/1)),((2+1s),final parameter(x=0,lambda=[5]/1,nu=[0]/1))]\nPCANCEL[((1-1s),final parameter(x=0,lambda=[3]/1,nu=[0]/1))]\nRECOVER619\n");
    }

    #[test]
    fn for_iteration_context_break_and_capture_match_original() {
        assert_eq!(for_iteration_reports(include_str!(
            "../../../tests/math/generics/for_context_and_capture.atlas")),
            b"VOID6\nVEC_RESULT[ 1, 4, 9 ]\nBREAK_REVERSE[2,1]\nVariable closures: [(->int,int)]\nCAPTURES(30,2)(20,1)(10,0)\nDISCARD[7,7,7]\nRECOVER401\n");
    }

    #[test]
    fn for_iteration_rejects_receiver_mutation_and_integer_result() {
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/for_nonrow_rejected.atlas")));
        let errors: Vec<_> = events.iter().filter_map(|e| match e {
            SessionEvent::Diagnostic(d) => Some(d), _ => None,
        }).collect();
        assert_eq!(errors.len(), 4, "{events:?}");
        assert_eq!(errors[0].message, "Cannot iterate over value of type int");
        assert!(errors[1].message.contains("Name 'x' is constant"), "{errors:?}");
        assert!(errors[2].message.contains("Name 'i' is constant"), "{errors:?}");
        assert_eq!(errors[3].message, "found [*] while int was needed.");
        let reports: Vec<_> = events.iter().filter_map(|e| match e {
            SessionEvent::ReportLine { text, .. } => Some(text.as_str()), _ => None,
        }).collect();
        assert_eq!(reports, ["RECOVER_A409\n", "RECOVER_B419\n", "RECOVER_C421\n", "RECOVER_D431\n"]);
    }

    #[test]
    fn byte_string_subscription_preserves_one_byte_values() {
        // Both engines accepted this exact source in3839240, but Rust's
        // lossy conversion printed3333 where original prints1111.
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/string_byte_subscription.atlas")));
        assert!(!events.iter().any(|e| matches!(e, SessionEvent::Diagnostic(_))), "{events:?}");
        let reports: Vec<_> = events.iter().filter_map(|e| match e {
            SessionEvent::ReportLine { text, .. } => Some(text.as_str()),
            _ => None,
        }).collect();
        assert_eq!(reports, ["INDEX_LENGTHS1111\n", "RECOVER349\n"]);
    }

    #[test]
    fn byte_string_slice_bounds_reject_at_runtime_and_recover() {
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/string_slice_bounds_rejected.atlas")));
        let errors: Vec<_> = events.iter().filter_map(|e| match e {
            SessionEvent::Diagnostic(d) => Some(d), _ => None,
        }).collect();
        assert_eq!(errors.len(), 3, "{events:?}");
        assert!(errors.iter().all(|d| d.kind == ErrorKind::Runtime), "{errors:?}");
        assert!(errors[0].message.starts_with("lower bound -1 out of range"));
        assert!(errors[1].message.starts_with("upper bound 4 out of range"));
        assert!(errors[2].message.starts_with("lower bound -1 out of range"));
        let reports: Vec<_> = events.iter().filter_map(|e| match e {
            SessionEvent::ReportLine { text, .. } => Some(text.as_str()), _ => None,
        }).collect();
        assert_eq!(reports, ["RECOVER_A331\n", "RECOVER_B337\n", "RECOVER_C347\n"]);
    }

    #[test]
    fn byte_string_numeric_observations_match_original() {
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/string_byte_values.atlas")));
        assert!(!events.iter().any(|e| matches!(e, SessionEvent::Diagnostic(_))), "{events:?}");
        let reports: Vec<_> = events.iter().filter_map(|e| match e {
            SessionEvent::ReportLine { text, .. } => Some(text.as_str()), _ => None,
        }).collect();
        assert_eq!(reports, ["BYTE_SIZE28\n", "BYTE_INDEX195169\n", "BYTE_SPLIT19516911\n",
            "BYTE_JOINé\n", "BYTE_REVERSE90169\n", "BYTE_EMPTY00-1\n", "RECOVER311\n"]);
    }

    #[test]
    fn byte_string_domain_and_error_messages_preserve_bytes() {
        //3839257 captures these three raw original messages separately.
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/string_byte_domain_rejected.atlas")));
        let errors: Vec<_> = events.iter().filter_map(|e| match e {
            SessionEvent::Diagnostic(d) => Some(d), _ => None,
        }).collect();
        assert_eq!(errors.len(), 3, "{events:?}");
        assert!(errors.iter().all(|d| d.kind == ErrorKind::Runtime));
        assert_eq!(errors[0].message_bytes(), b"Error in string '\xc3' that should specify a Lie type");
        assert_eq!(errors[1].message_bytes(), b"Unknown inner class symbol `\xc3'");
        assert_eq!(errors[2].message_bytes(), b"BYTE_ERROR<\xa9>");
    }

    #[test]
    fn byte_string_legacy_ascii_is_not_a_latest_builtin() {
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/ascii_legacy_unit_sources.atlas")));
        let errors: Vec<_> = events.iter().filter_map(|e| match e {
            SessionEvent::Diagnostic(d) => Some(d), _ => None,
        }).collect();
        assert_eq!(errors.len(), 7, "{events:?}");
        assert!(errors.iter().all(|d| d.message == "Undefined identifier 'ascii'"));
    }

    #[test]
    fn abstractions_instantiate_functions_and_keep_outer_bindings_independent() {
        // Accepted whole-source contracts captured in3837308 and3837985.
        for (source, expected) in [
            (include_str!("../../../tests/math/generics/identity_instances.atlas"),
             vec!["INSTANCES73/4[2,3]\n", "NESTED[[2],[3,4]]\n"]),
            (include_str!("../../../tests/math/generics/direct_polymorphic_function.atlas"),
             vec!["DIRECT([2],[3/4])\n"]),
            (include_str!("../../../tests/math/generics/nested_abstraction.atlas"),
             vec!["NESTED(2,\"abc\")(true,3/4)\n"]),
            (include_str!("../../../tests/math/generics/sibling_abstractions.atlas"),
             vec!["SIBLINGS(2,\"abc\")\n", "RESTORED19\n"]),
            (include_str!("../../../tests/math/generics/abstraction_binding_imports.atlas"),
             vec!["GLOBAL[][]\n", "LOCAL([],[])\n", "FIXED(7,\"a\")(true,3/4)\n"]),
            (include_str!("../../../tests/math/generics/fixed_local_assignment.atlas"),
             vec!["FIXED_LOCAL7new\n"]),
        ] {
            let events = run_source(&SourceText::new(source));
            assert!(!events.iter().any(|e| matches!(e, SessionEvent::Diagnostic(_))), "{source}\n{events:?}");
            for expected in expected {
                assert!(events.iter().any(|e| matches!(e,
                    SessionEvent::ReportLine { text, .. } if text == expected)), "missing {expected:?}: {events:?}");
            }
        }
    }

    #[test]
    fn abstractions_reject_rigid_mismatches_and_keep_the_incoming_context() {
        for (source, count) in [
            (include_str!("../../../tests/math/generics/rigid_body_rejected.atlas"), 1),
            (include_str!("../../../tests/math/generics/rigid_function_result_rejected.atlas"), 1),
            (include_str!("../../../tests/math/generics/abstraction_rigid_assignment_rejected.atlas"), 1),
            (include_str!("../../../tests/math/generics/abstraction_concrete_context.atlas"), 2),
        ] {
            let events = run_source(&SourceText::new(&format!("{source}\nprints(\"AFTER\",43)\n")));
            let errors: Vec<_> = events.iter().filter_map(|e| match e {
                SessionEvent::Diagnostic(d) => Some(d), _ => None,
            }).collect();
            assert_eq!(errors.len(), count, "{source}\n{events:?}");
            assert!(errors.iter().all(|d| d.kind == ErrorKind::Type), "{events:?}");
            assert!(events.iter().any(|e| matches!(e,
                SessionEvent::ReportLine { text, .. } if text == "AFTER43\n")), "{events:?}");
        }
    }

    #[test]
    fn polymorphic_declarations_keep_order_and_continue_after_a_failed_sibling() {
        let source = include_str!("../../../tests/math/generics/sequential_partial_failure.atlas");
        let events = run_source(&SourceText::new(source));
        let errors: Vec<_> = events.iter().enumerate().filter_map(|(i, e)| match e {
            SessionEvent::Diagnostic(d) => Some((i, d)), _ => None,
        }).collect();
        assert_eq!(errors.len(), 1, "{events:?}");
        assert_eq!(errors[0].1.kind, ErrorKind::Type);
        let first = events.iter().position(|e| matches!(e, SessionEvent::ReportLine { text, .. }
            if text.starts_with("Defined partial_first:"))).expect("first declaration retained");
        let last = events.iter().position(|e| matches!(e, SessionEvent::ReportLine { text, .. }
            if text.starts_with("Defined partial_last:"))).expect("last declaration attempted");
        assert!(first < errors[0].0 && errors[0].0 < last, "{events:?}");
        for expected in ["FIRST7seven\n", "LAST11eleven\n", "RESTORED29\n"] {
            assert!(events.iter().any(|e| matches!(e,
                SessionEvent::ReportLine { text, .. } if text == expected)), "{events:?}");
        }
        // Do not change ordinary SET's simultaneous analysis to get the above.
        let mut context = TypedContext::new();
        let events = run_source_with_context(&SourceText::new(include_str!(
            "../../../tests/math/generics/ordinary_parallel_declarations.atlas"
        )), &mut context);
        assert!(events.iter().any(|e| matches!(e,
            SessionEvent::Diagnostic(d) if d.kind == ErrorKind::Name)), "{events:?}");
        assert!(!events.iter().any(|e| matches!(e,
            SessionEvent::ReportLine { text, .. } if text.starts_with("Defined ordinary_"))), "{events:?}");
    }

    #[test]
    fn any_type_keyword_cannot_be_bound_and_recovers_at_the_next_command() {
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/any_type_keyword_rejected.atlas"
        )));
        let errors: Vec<_> = events.iter().filter_map(|e| match e {
            SessionEvent::Diagnostic(d) => Some(d), _ => None,
        }).collect();
        assert_eq!(errors.len(), 1, "{events:?}");
        assert_eq!(errors[0].kind, ErrorKind::Syntax);
        assert!(events.iter().any(|e| matches!(e,
            SessionEvent::ReportLine { text, .. } if text == "RECOVERED37\n")), "{events:?}");
    }

    #[test]
    fn abstraction_bang_and_cast_lambdas_match_current_original_values() {
        // Current-original3838354 accepts all four values, including the
        // polymorphic bang instances and the zero-argument result cast.
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/abstraction_symbol_and_casts.atlas"
        )));
        assert!(!events.iter().any(|e| matches!(e, SessionEvent::Diagnostic(_))), "{events:?}");
        for expected in ["BANG(2,[7])(3,[\"a\"])\n", "CAST7seven\n", "ZERO11\n", "RECOVER17\n"] {
            assert!(events.iter().any(|e| matches!(e,
                SessionEvent::ReportLine { text, .. } if text == expected)), "{events:?}");
        }
    }

    #[test]
    fn sequential_runtime_failure_keeps_prints_and_both_adjacent_bindings() {
        // Current-original3838354: a runtime error interrupts only its own
        // raw declaration, and output already printed remains before it.
        let mut context = TypedContext::new();
        let events = run_source_with_context(&SourceText::new(include_str!(
            "../../../tests/math/generics/sequential_runtime_failure.atlas"
        )), &mut context);
        let errors: Vec<_> = events.iter().enumerate().filter_map(|(i, e)| match e {
            SessionEvent::Diagnostic(d) => Some((i, d)), _ => None,
        }).collect();
        assert_eq!(errors.len(), 1, "{events:?}");
        assert_eq!(errors[0].1.kind, ErrorKind::Runtime);
        let printed = events.iter().position(|e| matches!(e,
            SessionEvent::ReportLine { text, .. } if text == "BEFORE_THROW5\n")).expect("printed output kept");
        let after = events.iter().position(|e| matches!(e,
            SessionEvent::ReportLine { text, .. } if text.starts_with("Constant after_throw:"))).expect("later binding kept");
        assert!(printed < errors[0].0 && errors[0].0 < after, "{events:?}");
        assert!(context.globals().lookup("broken").is_none());
        for expected in ["KEPT[][]\n", "RECOVER19\n"] {
            assert!(events.iter().any(|e| matches!(e,
                SessionEvent::ReportLine { text, .. } if text == expected)), "{events:?}");
        }
    }

    #[test]
    fn recursive_boundary_positive_preserves_erased_and_acyclic_dependencies() {
        // Original3844689 and SAME552binary3844760 agree on this entire
        // stream. In particular, its whattype injector signature is NOT a
        // counterexample for anonymous function-type arrow printing.
        assert_eq!(for_iteration_reports(include_str!(
            "../../../tests/math/generics/generic_recursive_group_boundaries.atlas")),
            include_bytes!("../../../tests/math/generics/generic_recursive_group_boundaries.expected"));
    }

    #[test]
    fn recursive_boundary_direct_aliases_are_syntax_errors() {
        // Original3844689 rejects before grouped-type analysis. The frozen
        // 552candidate instead raises Program errors (3844760).
        for (source, expected) in [
            (include_str!("../../../tests/math/generics/generic_recursive_direct_alias_nongeneric.atlas"),
             "DIRECT_ALIAS_REACHED1\n"),
            (include_str!("../../../tests/math/generics/generic_recursive_direct_alias_generic.atlas"),
             "DIRECT_GENERIC_ALIAS_REACHED2\n"),
            (include_str!("../../../tests/math/generics/generic_recursive_direct_self_alias.atlas"),
             "DIRECT_SELF_REACHED3\n"),
        ] {
            let events = run_source(&SourceText::new(source));
            let errors: Vec<_> = events.iter().filter_map(|event| match event {
                SessionEvent::Diagnostic(error) => Some(error), _ => None,
            }).collect();
            assert_eq!(errors.len(), 1, "{events:?}");
            assert_eq!(errors[0].kind, ErrorKind::Syntax, "{events:?}");
            let reports: String = events.iter().filter_map(|event| match event {
                SessionEvent::ReportLine { text, .. } => Some(text.as_str()), _ => None,
            }).collect();
            assert_eq!(reports, expected);
            assert!(!events.iter().any(|event| matches!(event,
                SessionEvent::Value { is_void_type: false, .. })), "{events:?}");
        }
    }

    #[test]
    fn recursive_boundary_bare_nested_constructor_is_syntax_error() {
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/generic_recursive_bare_constructor_nested.atlas")));
        let errors: Vec<_> = events.iter().filter_map(|event| match event {
            SessionEvent::Diagnostic(error) => Some(error), _ => None,
        }).collect();
        assert_eq!(errors.len(), 1, "{events:?}");
        assert_eq!(errors[0].kind, ErrorKind::Syntax, "{events:?}");
        let reports: String = events.iter().filter_map(|event| match event {
            SessionEvent::ReportLine { text, .. } => Some(text.as_str()), _ => None,
        }).collect();
        assert_eq!(reports, "Type name 'BareNestedRows' defined as [A]\nNESTED_RECOVER59\n");
    }

    #[test]
    fn recursive_boundary_bare_alias_preserves_original_recovery() {
        // Original consumes the following prints while looking for '<';
        // merely changing a Type error into Syntax does not repair recovery.
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/generic_recursive_bare_constructor_alias.atlas")));
        let errors: Vec<_> = events.iter().filter_map(|event| match event {
            SessionEvent::Diagnostic(error) => Some(error), _ => None,
        }).collect();
        assert_eq!(errors.len(), 1, "{events:?}");
        assert_eq!(errors[0].kind, ErrorKind::Syntax, "{events:?}");
        let reports: String = events.iter().filter_map(|event| match event {
            SessionEvent::ReportLine { text, .. } => Some(text.as_str()), _ => None,
        }).collect();
        assert_eq!(reports, "Type name 'BareAliasRows' defined as [A]\n");
    }

    #[test]
    fn generic_recursive_group_streams_forward_all_formals() {
        // Original3843104: full int/bool streams, two formal slots, linked
        // projector schemes and restoration of the formal names.
        assert_eq!(for_iteration_reports(include_str!(
            "../../../tests/math/generics/generic_mutual_recursive_values.atlas")),
            include_bytes!("../../../tests/math/generics/generic_mutual_recursive_values.expected"));
    }

    #[test]
    fn generic_recursive_group_anonymous_components_and_existing_instances() {
        // Original3843356: tuple/row nodes on cycles, an existing recursive
        // constructor OUTSIDE the new cycle, and an expanded nonrecursive one.
        assert_eq!(for_iteration_reports(include_str!(
            "../../../tests/math/generics/generic_recursive_components_valid.atlas")),
            include_bytes!("../../../tests/math/generics/generic_recursive_components_valid.expected"));
    }

    #[test]
    fn generic_recursive_group_seven_errors_preserve_every_survivor() {
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/generic_mutual_recursive_rejected.atlas")));
        let errors: Vec<_> = events.iter().filter_map(|event| match event {
            SessionEvent::Diagnostic(error) => Some(error), _ => None,
        }).collect();
        let expected = [
            "Repeated definition of 'ProbeDuplicate' in grouped type definition",
            "Used 'ProbeCollision' as defined type AND as field name",
            "Identifier 'NoSuchProbeType' does not refer to any type",
            "Type constructor 'ProbeExisting' called with 0 type arguments, expected 1",
            "Type definition recursion uses type constructor 'ProbeExisting', itself recursive, which is not allowed",
            "Type 'ProbeExisting' being defined cannot be given type arguments",
            "Cannot define 'probe_value' as a type; it is in use as global variable",
        ];
        assert_eq!(errors.len(), expected.len(), "{events:?}");
        for (error, message) in errors.iter().zip(expected) {
            assert_eq!(error.kind, ErrorKind::Program, "{error:?}");
            assert_eq!(error.message, message);
        }
        let reports: String = events.iter().filter_map(|event| match event {
            SessionEvent::ReportLine { text, .. } => Some(text.as_str()), _ => None,
        }).collect();
        assert_eq!(reports.as_bytes(), include_bytes!(
            "../../../tests/math/generics/generic_mutual_recursive_rejected.expected"));
        assert!(!events.iter().any(|event| matches!(event,
            SessionEvent::Value { is_void_type: false, .. })), "{events:?}");
    }

    #[test]
    fn grouped_type_duplicate_fields_and_function_collisions_are_atomic() {
        // Original3843306 rejects both declarations and preserves the whole
        // saved/fresh/survivor stream; old Rust incorrectly publishes both.
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/recursive_group_atomic_rejected.atlas")));
        let errors: Vec<_> = events.iter().filter_map(|event| match event {
            SessionEvent::Diagnostic(error) => Some(error), _ => None,
        }).collect();
        let expected = [
            "Multiple occurrences of 'atomic_duplicate' cannot be defined in same definition",
            "Cannot define 'atomic_function' as a type; it is in use as function",
        ];
        assert_eq!(errors.len(), expected.len(), "{events:?}");
        for (error, message) in errors.iter().zip(expected) {
            assert_eq!(error.kind, ErrorKind::Program);
            assert_eq!(error.message, message);
        }
        let reports: String = events.iter().filter_map(|event| match event {
            SessionEvent::ReportLine { text, .. } => Some(text.as_str()), _ => None,
        }).collect();
        assert_eq!(reports.as_bytes(), include_bytes!(
            "../../../tests/math/generics/recursive_group_atomic_rejected.expected"));
        assert!(!events.iter().any(|event| matches!(event,
            SessionEvent::Value { value, .. } if value != &Value::Tuple(Vec::new()))));
    }

    #[test]
    fn grouped_type_late_member_error_discards_all_pending_bindings() {
        // Original3843432: an earlier type/projector may be valid, but
        // a later conflicting injector aborts the entire grouped command.
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/recursive_group_member_conflict.atlas")));
        let errors: Vec<_> = events.iter().filter_map(|event| match event {
            SessionEvent::Diagnostic(error) => Some(error), _ => None,
        }).collect();
        assert_eq!(errors.len(), 2, "{events:?}");
        assert_eq!(errors[0].kind, ErrorKind::Program);
        assert_eq!(errors[0].message, concat!(
            "Cannot overload `atomic_conflict':\n",
            "already overloaded type '(int,rat)' is too close to new argument type '(rat,int)',\n",
            "which would make overloading ambiguous for certain arguments. Simultaneous\n",
            "overloading for these types is not possible, forget the other one first."));
        assert_eq!(errors[1].kind, ErrorKind::Name);
        assert_eq!(errors[1].message, "Undefined identifier 'atomic_new_first'");
        let reports: String = events.iter().filter_map(|event| match event {
            SessionEvent::ReportLine { text, .. } => Some(text.as_str()), _ => None,
        }).collect();
        assert_eq!(reports.as_bytes(), include_bytes!(
            "../../../tests/math/generics/recursive_group_member_conflict.expected"));
        assert!(!events.iter().any(|event| matches!(event,
            SessionEvent::Value { value, .. } if value != &Value::Tuple(Vec::new()))));
    }

    #[test]
    fn while_guard_modes_preserve_complete_original_values() {
        // Original3843167: all modes include the guard in the current
        // loop. A breaking guard contributes no body value or iteration.
        assert_eq!(for_iteration_reports(include_str!(
            "../../../tests/math/generics/while_guard_break_values.atlas")),
            include_bytes!("../../../tests/math/generics/while_guard_break_values.expected"));
    }

    #[test]
    fn while_guard_nested_unwind_matches_complete_original() {
        // Original3843183 accepts the whole corrected DoExpr fixture;
        // the earlier syntax-rejected fixture remains in the corpus.
        assert_eq!(for_iteration_reports(include_str!(
            "../../../tests/math/generics/while_guard_break_nested_valid.atlas")),
            include_bytes!("../../../tests/math/generics/while_guard_break_nested_valid.expected"));
    }

    #[test]
    fn while_guard_inner_break_does_not_terminate_the_outer_loop() {
        // Independently three outer iterations, each producing an empty
        // inner row. Original3843167 prints all three; old Rust prints one.
        assert_eq!(for_iteration_reports(
            "prints(\"INNER\",for i:3 do while if i=1 then break fi; false do 17 od od)\n"),
            b"INNER[[],[],[]]\n");
    }

    #[test]
    fn while_guard_rejects_excess_depth_and_lambda_crossing() {
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/while_guard_break_rejected.atlas")));
        let errors: Vec<_> = events.iter().filter_map(|event| match event {
            SessionEvent::Diagnostic(error) => Some(error), _ => None,
        }).collect();
        let expected = [
            "Using 'break break' requires 2 nested levels of loops",
            "Using 'break break break' requires 3 nested levels of loops",
            "Using 'break' not in the reach of any loop",
            "Using 'break' not in the reach of any loop",
            "Using 'break' not in the reach of any loop",
            "found int while bool was needed.",
        ];
        assert_eq!(errors.len(), expected.len(), "{events:?}");
        for (error, message) in errors.iter().zip(expected) {
            assert_eq!(error.kind, ErrorKind::Type);
            assert_eq!(error.message, message);
        }
        let reports: String = events.iter().filter_map(|event| match event {
            SessionEvent::ReportLine { text, .. } => Some(text.as_str()), _ => None,
        }).collect();
        assert_eq!(reports, "RECOVER891\nRECOVER892\nRECOVER893\nRECOVER894\nRECOVER895\nRECOVER896\n");
        assert!(!events.iter().any(|event| matches!(event,
            SessionEvent::Value { value, .. } if value != &Value::Tuple(Vec::new()))));
    }

    #[test]
    fn while_do_scopes_match_original_branch_and_binding_order() {
        // Original discovery3838473: one discriminator evaluation per
        // iteration, payload/let bindings live through their guarded body.
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/while_do_scope.atlas"
        )));
        assert!(!events.iter().any(|e| matches!(e, SessionEvent::Diagnostic(_))), "{events:?}");
        for expected in ["CASE[0,1,2]\n", "LET[3,4]\n", "IF[0,1]\n", "INDEXED[10,20]\n", "STATE522\n", "RECOVER31\n"] {
            assert!(events.iter().any(|e| matches!(e,
                SessionEvent::ReportLine { text, .. } if text == expected)), "{events:?}");
        }
    }

    #[test]
    fn while_do_missing_else_rejects_and_recovers() {
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/while_do_rejected.atlas"
        )));
        let errors: Vec<_> = events.iter().filter_map(|e| match e {
            SessionEvent::Diagnostic(d) => Some(d), _ => None,
        }).collect();
        assert_eq!(errors.len(), 1, "{events:?}");
        assert_eq!(errors[0].kind, ErrorKind::Syntax);
        assert!(events.iter().any(|e| matches!(e,
            SessionEvent::ReportLine { text, .. } if text == "RECOVER37\n")), "{events:?}");
    }

    #[test]
    fn while_context_modes_match_original_counts_values_and_effects() {
        // Original discovery3838562, including nested dont and effects on
        // the final false guard. Count/void bodies need a void type context.
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/while_do_contexts.atlas"
        )));
        assert!(!events.iter().any(|e| matches!(e, SessionEvent::Diagnostic(_))), "{events:?}");
        for expected in ["COUNT3\n", "VOID2\n", "REVERSE[2,1,0]\n", "GUARD[0,1]\n", "EFFECTS32\n", "NESTED[([],0),([],1)]\n", "RECOVER41\n"] {
            assert!(events.iter().any(|e| matches!(e,
                SessionEvent::ReportLine { text, .. } if text == expected)), "{events:?}");
        }
    }

    #[test]
    fn while_invalid_dead_bodies_and_escaping_bindings_still_reject() {
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/while_do_invalid_bodies.atlas"
        )));
        let errors: Vec<_> = events.iter().filter_map(|e| match e {
            SessionEvent::Diagnostic(d) => Some(d), _ => None,
        }).collect();
        assert_eq!(errors.len(), 3, "{events:?}");
        assert_eq!(errors[0].kind, ErrorKind::Name);
        assert!(errors[0].message.contains("not_defined_in_false_branch"));
        assert_eq!(errors[1].kind, ErrorKind::Name);
        assert!(errors[1].message.contains("guarded_local"));
        assert_eq!(errors[2].kind, ErrorKind::Type);
        assert!(errors[2].message.contains("No common type found"));
        for expected in ["RECOVER_A43\n", "RECOVER_B47\n", "RECOVER_C53\n"] {
            assert!(events.iter().any(|e| matches!(e,
                SessionEvent::ReportLine { text, .. } if text == expected)), "{events:?}");
        }
    }

    #[test]
    fn break_preserves_polymorphic_loop_results_and_nested_unwind() {
        // Entire fixture accepted by current original3838661; pre-repair
        // Rust rejects both generic definitions with [void] versus [A].
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/break_polymorphic_valid.atlas"
        )));
        assert!(!events.iter().any(|e| matches!(e, SessionEvent::Diagnostic(_))), "{events:?}");
        for expected in ["PREFIX[7,7][\"seven\",\"seven\"]\n", "TAGGED[7]\n", "BREAK[1]\n", "NESTED[[(0,0),(0,1)]]\n", "RECOVER59\n"] {
            assert!(events.iter().any(|e| matches!(e,
                SessionEvent::ReportLine { text, .. } if text == expected)), "{events:?}");
        }
    }

    #[test]
    fn break_keeps_lexical_checks_and_rejects_legacy_numeric_spelling() {
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/break_lexical_valid.atlas"
        )));
        let errors: Vec<_> = events.iter().filter_map(|e| match e {
            SessionEvent::Diagnostic(d) => Some(d), _ => None,
        }).collect();
        assert_eq!(errors.len(), 3, "{events:?}");
        assert!(errors.iter().all(|d| d.kind == ErrorKind::Type));
        assert_eq!(errors[0].message, "Using 'break' not in the reach of any loop");
        assert_eq!(errors[1].message, errors[0].message);
        assert_eq!(errors[2].message, "Using 'break break' requires 2 nested levels of loops");
        for expected in ["RECOVER_A61\n", "RECOVER_B67\n", "RECOVER_C71\n"] {
            assert!(events.iter().any(|e| matches!(e,
                SessionEvent::ReportLine { text, .. } if text == expected)), "{events:?}");
        }
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/break_numeric_rejected.atlas"
        )));
        let errors: Vec<_> = events.iter().filter_map(|e| match e {
            SessionEvent::Diagnostic(d) => Some(d), _ => None,
        }).collect();
        assert_eq!(errors.len(), 1, "{events:?}");
        assert_eq!(errors[0].kind, ErrorKind::Syntax);
        assert!(events.iter().any(|e| matches!(e,
            SessionEvent::ReportLine { text, .. } if text == "RECOVER73\n")), "{events:?}");
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/break_levels_legacy_rejected.atlas"
        )));
        let errors: Vec<_> = events.iter().filter_map(|e| match e {
            SessionEvent::Diagnostic(d) => Some(d), _ => None,
        }).collect();
        assert_eq!(errors.len(), 5, "{events:?}");
        assert!(errors.iter().all(|d| d.kind == ErrorKind::Syntax));
        assert!(events.iter().any(|e| matches!(e,
            SessionEvent::ReportLine { text, .. } if text == "RECOVER83\n")), "{events:?}");
    }

    #[test]
    fn operator_patterns_accept_selected_and_generic_function_values() {
        // Original3838739 accepts both symbol bindings; unchanged Rust
        // rejects '=' and continues with its unrelated builtin tilde.
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/operator_value_bindings.atlas"
        )));
        assert!(!events.iter().any(|e| matches!(e, SessionEvent::Diagnostic(_))), "{events:?}");
        for expected in ["TILDE-7\n", "BANG7seven\n", "RECOVER97\n"] {
            assert!(events.iter().any(|e| matches!(e,
                SessionEvent::ReportLine { text, .. } if text == expected)), "{events:?}");
        }
    }

    #[test]
    fn operator_patterns_reject_nonfunction_global_values() {
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/operator_value_rejected.atlas"
        )));
        let errors: Vec<_> = events.iter().filter_map(|e| match e {
            SessionEvent::Diagnostic(d) => Some(d), _ => None,
        }).collect();
        assert_eq!(errors.len(), 1, "{events:?}");
        assert_eq!(errors[0].kind, ErrorKind::Type);
        assert_eq!(errors[0].message,
            "Cannot set operator '!' to a value of non-function type int");
        assert!(events.iter().any(|e| matches!(e,
            SessionEvent::ReportLine { text, .. } if text == "RECOVER101\n")), "{events:?}");
    }

    #[test]
    fn operator_patterns_preserve_lexical_dispatch_and_tuple_values() {
        // Whole fixture accepted by original3838953, including local
        // functions shadowing global operator overloads in every scope.
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/operator_pattern_scopes_valid.atlas"
        )));
        assert!(!events.iter().any(|e| matches!(e, SessionEvent::Diagnostic(_))), "{events:?}");
        for expected in ["LOCAL12\n", "TUPLE(15,5)\n", "PARAM16\n", "LOOP[18]\n", "GLOBAL199\n", "RECOVER137\n"] {
            assert!(events.iter().any(|e| matches!(e,
                SessionEvent::ReportLine { text, .. } if text == expected)), "{events:?}");
        }
    }

    #[test]
    fn operator_patterns_reject_nonfunctions_in_each_scope_before_binding() {
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/operator_pattern_rejected_valid.atlas"
        )));
        let errors: Vec<_> = events.iter().filter_map(|e| match e {
            SessionEvent::Diagnostic(d) => Some(d), _ => None,
        }).collect();
        assert_eq!(errors.len(), 5, "{events:?}");
        assert!(errors.iter().all(|d| d.kind == ErrorKind::Type));
        for diagnostic in &errors[..4] {
            assert_eq!(diagnostic.message,
                "Cannot bind operator '!' to an expression of non-function type int");
        }
        assert_eq!(errors[4].message,
            "Cannot set operator '!' to a value of non-function type int");
        for expected in ["RECOVER_A139\n", "RECOVER_B149\n", "RECOVER_C151\n", "RECOVER_D157\n", "ATOMIC19\n", "RECOVER_E163\n"] {
            assert!(events.iter().any(|e| matches!(e,
                SessionEvent::ReportLine { text, .. } if text == expected)), "{events:?}");
        }
    }

    #[test]
    fn operator_casts_select_exact_before_generic_and_ignore_local_shadowing() {
        // Original3838987: selection is NOT ordinary overload application.
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/operator_cast_values.atlas"
        )));
        assert!(!events.iter().any(|e| matches!(e, SessionEvent::Diagnostic(_))), "{events:?}");
        for expected in ["EXACT323/4\n", "APPEND[[1],[2,3]][[\"a\"],[\"b\",\"c\"]]\n", "GLOBAL3\n", "FIXED7seven\n", "FALLBACK3/4\n", "RECOVER167\n"] {
            assert!(events.iter().any(|e| matches!(e,
                SessionEvent::ReportLine { text, .. } if text == expected)), "{events:?}");
        }
    }

    #[test]
    fn operator_casts_reject_coercion_ambiguity_and_missing_names() {
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/operator_cast_rejected.atlas"
        )));
        let errors: Vec<_> = events.iter().filter_map(|e| match e {
            SessionEvent::Diagnostic(d) => Some(d), _ => None,
        }).collect();
        assert_eq!(errors.len(), 3, "{events:?}");
        assert!(errors.iter().all(|d| d.kind == ErrorKind::Type));
        assert_eq!(errors[0].message, "No instance for only_rat@int found");
        assert_eq!(errors[1].message, "Ambiguous argument in function call, specified type (int,int) matches both (A,B) and (A,A)");
        assert_eq!(errors[2].message, "No instance for not_a_registered_function@int found");
        for expected in ["RECOVER_A173\n", "RECOVER_B179\n", "RECOVER_C181\n"] {
            assert!(events.iter().any(|e| matches!(e,
                SessionEvent::ReportLine { text, .. } if text == expected)), "{events:?}");
        }
    }

    #[test]
    fn operator_casts_keep_formals_lexical_and_require_a_closed_type() {
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/operator_cast_scope_rejected.atlas"
        )));
        let errors: Vec<_> = events.iter().filter_map(|e| match e {
            SessionEvent::Diagnostic(d) => Some(d), _ => None,
        }).collect();
        // Original emits four syntax records, including END recovery. The
        // session's envelope can group recovery differently; keep all three
        // rejected commands and require the valid prefix plus later values.
        assert!(errors.len() >= 3, "{events:?}");
        assert!(errors.iter().all(|d| d.kind == ErrorKind::Syntax), "{events:?}");
        for expected in ["Defined selected_length: ([A]->int)\n", "RECOVER_A191\n", "RECOVER_B193\n", "RECOVER_C197\n"] {
            assert!(events.iter().any(|e| matches!(e,
                SessionEvent::ReportLine { text, .. } if text == expected)), "{events:?}");
        }
    }

    #[test]
    fn returns_preserve_loop_values_and_nested_function_boundaries() {
        // Original3839034 accepts this whole fixture. Before repair Rust
        // also succeeds, but silently prints void for false, index 1 and 23.
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/return_loop_values.atlas"
        )));
        assert!(!events.iter().any(|e| matches!(e, SessionEvent::Diagnostic(_))), "{events:?}");
        for expected in ["BOOLfalsetrue\n", "INDEX1-1\n", "GENERIC7first9\n", "NESTED19\n", "WHILE23\n", "RECOVER199\n"] {
            assert!(events.iter().any(|e| matches!(e,
                SessionEvent::ReportLine { text, .. } if text == expected)), "{events:?}");
        }
    }

    #[test]
    fn returns_check_discarded_dead_and_nested_operands() {
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/return_loop_rejected.atlas"
        )));
        let errors: Vec<_> = events.iter().filter_map(|e| match e {
            SessionEvent::Diagnostic(d) => Some(d), _ => None,
        }).collect();
        assert_eq!(errors.len(), 3, "{events:?}");
        assert!(errors.iter().all(|d| d.kind == ErrorKind::Type
            && d.message == "found string while int was needed."), "{events:?}");
        assert!(!events.iter().any(|e| matches!(e,
            SessionEvent::ReportLine { text, .. } if text.starts_with("Defined wrong_"))), "{events:?}");
        for expected in ["RECOVER_A211\n", "RECOVER_B223\n", "RECOVER_C227\n"] {
            assert!(events.iter().any(|e| matches!(e,
                SessionEvent::ReportLine { text, .. } if text == expected)), "{events:?}");
        }
    }

    #[test]
    fn returns_share_ordered_inference_and_result_coercions() {
        // Original3839086: explicit returns fix rather than balance their
        // type, and the first value of NEXT constrains the discarded return.
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/return_inference_values.atlas"
        )));
        assert!(!events.iter().any(|e| matches!(e, SessionEvent::Diagnostic(_))), "{events:?}");
        for expected in ["INFER79\n", "RAT3/21/1\n", "DECLARED1/1\n", "NEXT1/1\n", "NESTED17\n", "RECOVER233\n"] {
            assert!(events.iter().any(|e| matches!(e,
                SessionEvent::ReportLine { text, .. } if text == expected)), "{events:?}");
        }
    }

    #[test]
    fn returns_reject_later_incompatible_result_types() {
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/return_inference_rejected.atlas"
        )));
        let errors: Vec<_> = events.iter().filter_map(|e| match e {
            SessionEvent::Diagnostic(d) => Some(d), _ => None,
        }).collect();
        assert_eq!(errors.len(), 3, "{events:?}");
        assert!(errors.iter().all(|d| d.kind == ErrorKind::Type), "{events:?}");
        assert_eq!(errors[0].message, "found rat while int was needed.");
        assert_eq!(errors[1].message, "found rat while int was needed.");
        assert_eq!(errors[2].message, "found int while string was needed.");
        assert!(!events.iter().any(|e| matches!(e,
            SessionEvent::ReportLine { text, .. } if text.starts_with("Defined "))), "{events:?}");
        for expected in ["RECOVER_A239\n", "RECOVER_B241\n", "RECOVER_C251\n"] {
            assert!(events.iter().any(|e| matches!(e,
                SessionEvent::ReportLine { text, .. } if text == expected)), "{events:?}");
        }
    }

    #[test]
    fn returns_preserve_structures_initializers_guards_and_recursion() {
        // Original3839128 completes all these values; pre-repair Rust
        // reaches a void-operand panic in the final recursive calculation.
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/return_structural_values.atlas"
        )));
        assert!(!events.iter().any(|e| matches!(e, SessionEvent::Diagnostic(_))), "{events:?}");
        for expected in ["PAIR(7,\"early\")(9,\"late\")\n", "ROW[2,3][5]\n", "TUPLE(31,37)\n", "LIST[41]\n", "LET43\n", "GUARD47\n", "FUNCTION53\n", "ABSTRACT59\n", "RECURSIVE120\n", "RECOVER257\n"] {
            assert!(events.iter().any(|e| matches!(e,
                SessionEvent::ReportLine { text, .. } if text == expected)), "{events:?}");
        }
    }

    #[test]
    fn returns_observe_structure_constraints_before_converting_components() {
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/return_structural_rejected.atlas"
        )));
        let errors: Vec<_> = events.iter().filter_map(|e| match e {
            SessionEvent::Diagnostic(d) => Some(d), _ => None,
        }).collect();
        assert_eq!(errors.len(), 3, "{events:?}");
        assert!(errors.iter().all(|d| d.kind == ErrorKind::Type), "{events:?}");
        assert_eq!(errors[0].message, "found int while (A,B) was needed.");
        assert_eq!(errors[1].message, "found int while [A] was needed.");
        assert_eq!(errors[2].message, "found rat while int was needed.");
        assert!(!events.iter().any(|e| matches!(e,
            SessionEvent::ReportLine { text, .. } if text.starts_with("Defined wrong_"))), "{events:?}");
        for expected in ["RECOVER_A263\n", "RECOVER_B269\n", "RECOVER_C271\n"] {
            assert!(events.iter().any(|e| matches!(e,
                SessionEvent::ReportLine { text, .. } if text == expected)), "{events:?}");
        }
    }

    #[test]
    fn slice_direction_uses_reverse_iterator_coordinates() {
        // Original3839145 distinguishes asymmetric intervals from the
        // existing symmetric len4[1:3] control. Both engines accept, but
        // old Rust computes [1,0] instead of [4,3] for a~[0:2].
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/slice_reverse_asymmetric.atlas"
        )));
        assert!(!events.iter().any(|e| matches!(e, SessionEvent::Diagnostic(_))), "{events:?}");
        for expected in ["REVERSE[4,3][3,2,1][4,3][2,1,0][4,3,2,1,0]\n", "LOWER_END[3,4][1,0]\n", "UPPER_END[0,1,2,3][4,3,2,1]\n", "BOTH_END[1,2,3][3,2,1]\n", "EMPTY[][][][]\n", "RECOVER307\n"] {
            assert!(events.iter().any(|e| matches!(e,
                SessionEvent::ReportLine { text, .. } if text == expected)), "{events:?}");
        }
    }

    #[test]
    fn named_types_use_the_live_session_environment() {
        // R7 original capture3834702 accepts the named row cast and parameter,
        // and rejects a string component at type analysis, not at parsing.
        let source = SourceText::new(concat!(
            "set_type MathMonoRow = [int]\n",
            "set probe_mono = MathMonoRow:[2,3]\n",
            "set probe_mono_function(MathMonoRow xs) = xs\n",
            "probe_mono_function(probe_mono)\n",
            "set bad = MathMonoRow:[\"abc\"]\n",
        ));
        let events = run_source(&source);
        let errors: Vec<_> = events.iter().filter_map(|e| match e {
            SessionEvent::Diagnostic(d) => Some(d), _ => None,
        }).collect();
        assert_eq!(errors.len(), 1, "{events:?}");
        assert_eq!(errors[0].kind, ErrorKind::Type, "{events:?}");
        assert!(events.iter().any(|e| matches!(e,
            SessionEvent::Value { value, .. } if value.to_string() == "[2,3]")));
    }

    #[test]
    fn constructor_structural_consumers_keep_the_concrete_arguments() {
        let source = SourceText::new(include_str!("../../../tests/math/generics/constructor_structural_uses_spaced.atlas"));
        let events = run_source(&source);
        assert!(!events.iter().any(|e| matches!(e, SessionEvent::Diagnostic(_))), "{events:?}");
        // `prints` emits ReportLine through the typed command pipeline; Output
        // is a different session event. Original capture3835245 fixes these
        // complete lines, including their terminating newlines.
        for expected in ["ROW3[3,5]3", "CHANGED[7,3,5]", "FUNCTION12", "NESTED13", "RESTORED17"] {
            assert!(events.iter().any(|event| matches!(event,
                SessionEvent::ReportLine { text, .. } if text == &format!("{expected}\n"))),
                "missing {expected}: {events:?}");
        }
    }

    #[test]
    fn generic_projector_calls_freshen_each_instance_and_link_the_result() {
        // Original captures 3835776/3836236: each call instantiates one whole
        // signature, with the same substitution for its argument and result.
        for (source, expected) in [
            (include_str!("../../../tests/math/generics/constructor_field_instances_valid.atlas"),
             vec!["FIRST7seven\n", "SECOND3/4[2,3]\n", "FIRST_AGAIN7seven\n"]),
            (include_str!("../../../tests/math/generics/pair_nested.atlas"),
             vec!["FIELDS[2,3](3/4,true)\n"]),
            (include_str!("../../../tests/math/generics/duplicate_formals_instantiated.atlas"),
             vec!["REPEATED(1,2)12\n"]),
        ] {
            let events = run_source(&SourceText::new(source));
            assert!(!events.iter().any(|e| matches!(e, SessionEvent::Diagnostic(_))), "{events:?}");
            for expected in expected {
                assert!(events.iter().any(|e| matches!(e,
                    SessionEvent::ReportLine { text, .. } if text == expected)), "{events:?}");
            }
        }
    }

    #[test]
    fn ordinary_generic_rows_recover_after_all_empty_argument_ambiguities() {
        // Exact original contracts captured in3836455, including recovery.
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/row_generic_ambiguities.atlas"
        )));
        let errors: Vec<_> = events.iter().filter_map(|e| match e {
            SessionEvent::Diagnostic(d) => Some(d), _ => None,
        }).collect();
        assert_eq!(errors.len(), 5, "{events:?}");
        assert!(errors.iter().all(|d| d.kind == ErrorKind::Type
            && d.message.starts_with("Ambiguous argument in function call")), "{events:?}");
        assert!(events.iter().any(|e| matches!(e,
            SessionEvent::ReportLine { text, .. } if text == "RECOVER17\n")), "{events:?}");

        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/row_generic_mixed.atlas"
        )));
        assert!(!events.iter().any(|e| matches!(e, SessionEvent::Diagnostic(_))), "{events:?}");
        let lines: Vec<_> = events.iter().filter_map(|e| match e {
            SessionEvent::ReportLine { text, .. } => Some(text.as_str()), _ => None,
        }).collect();
        assert_eq!(lines, ["LEFT[1,2]\n", "RIGHT[1,2]\n"]);
    }

    #[test]
    fn captured_generic_and_builtin_functions_are_first_class() {
        // Before3836435/3836533: original succeeds; Rust reports missing
        // identifiers. Each captured scheme must be instantiated per use.
        for (source, expected) in [
            (include_str!("../../../tests/math/generics/constructor_function_values.atlas"),
             vec!["DIRECT7\n", "LOCAL(2,3/4)\n", "TUPLE(2,\"b\")\n"]),
            (include_str!("../../../tests/math/generics/captured_result_selection.atlas"),
             vec!["GENERIC7\n", "CONCRETEtrue\n"]),
            (include_str!("../../../tests/math/generics/captured_builtin_function.atlas"),
             vec!["BUILTIN8\n", "RETURNED10\n", "TUPLE(8,6)\n"]),
            // Original3837308 supplies argument context for the tuple too.
            (include_str!("../../../tests/math/generics/direct_function_argument_context.atlas"),
             vec!["CONTEXT8\n", "RETURNED10\n"]),
        ] {
            let events = run_source(&SourceText::new(source));
            assert!(!events.iter().any(|e| matches!(e, SessionEvent::Diagnostic(_))), "{source}\n{events:?}");
            for expected in expected {
                assert!(events.iter().any(|e| matches!(e,
                    SessionEvent::ReportLine { text, .. } if text == expected)), "{source}\n{events:?}");
            }
        }
    }

    #[test]
    fn captured_builtin_values_preserve_display_and_variadic_packing() {
        // Before3836975/3837092: ordinary builtins retain their original
        // registry name, while variadic arguments stay one packed value.
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/captured_builtin_display.atlas"
        )));
        assert!(!events.iter().any(|e| matches!(e, SessionEvent::Diagnostic(_))), "{events:?}");
        for expected in ["SUCC{succ@int}\n", "PAIR({succ@int},{pred@int})\n",
                         "PRINTER{prints@A}\n", "PACKED(1,2)[3,4]\n"] {
            assert!(events.iter().any(|e| matches!(e,
                SessionEvent::ReportLine { text, .. } if text == expected)), "{events:?}");
        }
        assert!(events.iter().any(|e| matches!(e,
            SessionEvent::Value { value, .. } if value.to_string() == "{succ@int}")), "{events:?}");
    }

    #[test]
    fn polymorphic_list_balance_preserves_acceptance_and_rejection() {
        // Original3837421 accepts independent empty row components, but
        // refuses simultaneous coercion/substitution across tuple entries.
        // The unchanged3836533 Rust does the opposite on these two fixtures.
        let accepted = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/polymorphic_row_balance.atlas"
        )));
        assert!(!accepted.iter().any(|e| matches!(e, SessionEvent::Diagnostic(_))), "{accepted:?}");
        for expected in ["BALANCED[([],[3/4,5/1]),([2],[])]\n", "EMPTY[[],[]]\n", "RECOVER17\n"] {
            assert!(accepted.iter().any(|e| matches!(e,
                SessionEvent::ReportLine { text, .. } if text == expected)), "{accepted:?}");
        }
        let rejected = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/polymorphic_row_balance_mixed_rejected.atlas"
        )));
        let diagnostics: Vec<_> = rejected.iter().filter_map(|e| match e {
            SessionEvent::Diagnostic(d) => Some(d), _ => None,
        }).collect();
        assert_eq!(diagnostics.len(), 1, "{rejected:?}");
        assert_eq!(diagnostics[0].message, "No common type found between components of list expression: { ([rat],[bool]), ([int],[A]) }");
        assert!(rejected.iter().any(|e| matches!(e,
            SessionEvent::ReportLine { text, .. } if text == "RECOVER17\n")), "{rejected:?}");
        assert!(!rejected.iter().any(|e| matches!(e,
            SessionEvent::ReportLine { text, .. } if text.starts_with("MIXED"))), "{rejected:?}");
    }

    #[test]
    fn implicit_polymorphic_constants_preserve_concrete_siblings_and_shadowing() {
        // Original3837531: three global and two local assignments reject.
        // Before3837531 Rust accepts them and mutates the protected values.
        for (source, messages, lines) in [
            (include_str!("../../../tests/math/generics/polymorphic_binding_mutations.atlas"),
             vec!["Name 'frozen_row' is constant in assignment frozen_row:=[3]",
                  "Name 'frozen_row' is constant in multiple assignment set (mutable_count,frozen_row):=(4,[3])",
                  "Name 'frozen_tuple' is constant in assignment frozen_tuple:=([3],2)"],
             vec!["Constant frozen_row: [A]\n", "UNCHANGED2[]\n",
                  "Constant frozen_tuple: ([A],int)\n", "TUPLE([],1)\n",
                  "Variable frozen_row: [int] (overriding previous instance, which had type [A] (constant))\n",
                  "REBOUND[5]\n"]),
            (include_str!("../../../tests/math/generics/polymorphic_local_binding_mutations.atlas"),
             vec!["Name 'xs' is constant in assignment xs:=[3]",
                  "Name 'xs' is constant in multiple assignment set (n,xs):=(4,[3])"],
             vec!["MONO[1]\n", "SHADOW[2]\n", "LEAF(2,[])\n", "RECOVER17\n"]),
        ] {
            let events = run_source(&SourceText::new(source));
            let diagnostics: Vec<_> = events.iter().filter_map(|e| match e {
                SessionEvent::Diagnostic(d) => Some(d), _ => None,
            }).collect();
            assert_eq!(diagnostics.len(), messages.len(), "{events:?}");
            for (diagnostic, expected) in diagnostics.iter().zip(messages) {
                assert_eq!(diagnostic.kind, ErrorKind::Name, "{events:?}");
                assert_eq!(diagnostic.message, expected, "{events:?}");
            }
            for expected in lines {
                assert!(events.iter().any(|e| matches!(e,
                    SessionEvent::ReportLine { text, .. } if text == expected)), "{events:?}");
            }
        }
    }

    #[test]
    fn row_loop_bindings_are_constant_but_counted_loop_index_is_mutable() {
        // Before3837531: Rust accepts ROW/INDEX and rejects COUNT; original
        // does exactly the opposite. Keep both directions and recovery.
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/loop_binding_mutations.atlas"
        )));
        let diagnostics: Vec<_> = events.iter().filter_map(|e| match e {
            SessionEvent::Diagnostic(d) => Some(d), _ => None,
        }).collect();
        assert_eq!(diagnostics.len(), 2, "{events:?}");
        for (diagnostic, expected) in diagnostics.iter().zip([
            "Name 'x' is constant in assignment x:=3",
            "Name 'i' is constant in assignment i:=3",
        ]) {
            assert_eq!(diagnostic.kind, ErrorKind::Name, "{events:?}");
            assert_eq!(diagnostic.message, expected, "{events:?}");
        }
        let lines: Vec<_> = events.iter().filter_map(|e| match e {
            SessionEvent::ReportLine { text, .. } => Some(text.as_str()), _ => None,
        }).collect();
        assert_eq!(lines, ["COUNT[3,3]\n", "RECOVER17\n"]);
    }

    #[test]
    fn captured_complete_signature_ambiguity_rejects_and_recovers() {
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/captured_function_ambiguity.atlas"
        )));
        let errors: Vec<_> = events.iter().filter_map(|e| match e {
            SessionEvent::Diagnostic(d) => Some(d), _ => None,
        }).collect();
        assert_eq!(errors.len(), 1, "{events:?}");
        assert_eq!(errors[0].message, "Ambiguous overloaded symbol 'captured_first': its context type (CaptureAmbiguous<int>->int) matches\n  both (CaptureAmbiguous<A>->A) and (CaptureAmbiguous<int>->int) in overload table");
        assert!(events.iter().any(|e| matches!(e,
            SessionEvent::ReportLine { text, .. } if text == "RECOVER17\n")), "{events:?}");
    }

    #[test]
    fn generic_and_concrete_exact_overloads_are_ambiguous_and_recover() {
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/constructor_projector_ambiguity.atlas")));
        let errors = events.iter().filter_map(|e| match e {
            SessionEvent::Diagnostic(d) => Some(d), _ => None,
        }).collect::<Vec<_>>();
        assert_eq!(errors.len(), 1, "{events:?}");
        assert!(errors[0].message.contains("Ambiguous argument in function call"), "{events:?}");
        assert!(!events.iter().any(|e| matches!(e,
            SessionEvent::Value { value, .. } if value.to_string() == "99")), "{events:?}");
        assert!(events.iter().any(|e| matches!(e,
            SessionEvent::ReportLine { text, .. } if text == "RECOVER3/4\n")), "{events:?}");
    }

    #[test]
    fn generic_projector_result_constraint_does_not_rebind_its_argument() {
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/constructor_projector_result_rejected.atlas")));
        let errors = events.iter().filter_map(|e| match e {
            SessionEvent::Diagnostic(d) => Some(d), _ => None,
        }).collect::<Vec<_>>();
        assert_eq!(errors.len(), 1, "{events:?}");
        assert_eq!(errors[0].message, "found int while string was needed.");
        assert!(events.iter().any(|e| matches!(e,
            SessionEvent::ReportLine { text, .. } if text == "RECOVERseven\n")), "{events:?}");
    }

    #[test]
    fn field_writes_use_retained_definitions_not_projector_values() {
        // Original captures3836211/3836306 verify both overwritten functions
        // and forgotten type names, with complete post-write values.
        for (source, expected) in [
            (include_str!("../../../tests/math/generics/constructor_field_assignment.atlas"),
             vec!["FIELDS_CHANGED(5,7)\n", "INDEPENDENT(11,7)\n", "MONOMORPHIC(13,3)99\n"]),
            (include_str!("../../../tests/math/generics/field_definition_forgotten.atlas"),
             vec!["RETAINED(7,3)\n"]),
        ] {
            let events = run_source(&SourceText::new(source));
            assert!(!events.iter().any(|e| matches!(e, SessionEvent::Diagnostic(_))), "{events:?}");
            for expected in expected {
                assert!(events.iter().any(|e| matches!(e,
                    SessionEvent::ReportLine { text, .. } if text == expected)), "{events:?}");
            }
        }
    }

    #[test]
    fn field_and_tag_metadata_ambiguities_reject_without_mutating_values() {
        for (source, message, recovery) in [
            (include_str!("../../../tests/math/generics/field_definition_ambiguity.atlas"),
             "matches more than one definition with field name 'shared_first'", "RECOVER(2,3)\n"),
            (include_str!("../../../tests/math/generics/union_definition_ambiguity.atlas"),
             "Ambiguity in discrimination clause, possible types are:\n    FirstUnion\n    SecondUnion\n", "RECOVER17\n"),
        ] {
            let events = run_source(&SourceText::new(source));
            let errors = events.iter().filter_map(|e| match e {
                SessionEvent::Diagnostic(d) => Some(d), _ => None,
            }).collect::<Vec<_>>();
            assert_eq!(errors.len(), 1, "{events:?}");
            assert!(errors[0].message.contains(message), "{events:?}");
            assert!(events.iter().any(|e| matches!(e,
                SessionEvent::ReportLine { text, .. } if text == recovery)), "{events:?}");
        }
    }

    #[test]
    fn generic_union_injectors_and_tagged_branches_share_type_arguments() {
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/constructor_union_context.atlas")));
        assert!(!events.iter().any(|e| matches!(e, SessionEvent::Diagnostic(_))), "{events:?}");
        for expected in ["INT8\n", "STRINGseven\n", "NONE0\n", "NONE_STRING0\n"] {
            assert!(events.iter().any(|e| matches!(e,
                SessionEvent::ReportLine { text, .. } if text == expected)), "{events:?}");
        }
    }

    #[test]
    fn repeated_injector_arguments_reject_mixed_types_then_recover() {
        let events = run_source(&SourceText::new(include_str!(
            "../../../tests/math/generics/constructor_repeated_argument_rejected.atlas")));
        let errors = events.iter().filter_map(|e| match e {
            SessionEvent::Diagnostic(d) => Some(d), _ => None,
        }).collect::<Vec<_>>();
        assert_eq!(errors.len(), 1, "{events:?}");
        assert!(errors[0].message.contains("found (int,string)"), "{events:?}");
        assert!(events.iter().any(|e| matches!(e,
            SessionEvent::ReportLine { text, .. } if text == "RECOVER16\n")), "{events:?}");
    }

    #[test]
    fn constructor_arity_rejects_before_conversion_and_recovers_scope() {
        let events = run_source(&SourceText::new(concat!(
            "set_type Pair<S,T> = (S,T) !\n",
            "Pair<int>:(2,3)\n",
            "Pair<int,int,int>:(2,3)\n",
            "Pair<int,int>:(2,3)\n",
            "set T=19\n", "T\n",
        )));
        let errors = events.iter().filter_map(|event| match event {
            SessionEvent::Diagnostic(d) => Some(d), _ => None,
        }).collect::<Vec<_>>();
        assert_eq!(errors.len(), 2, "{events:?}");
        assert!(errors.iter().all(|d| d.kind == ErrorKind::Type), "{events:?}");
        assert!(errors[0].message.contains("1 type arguments, expected 2"));
        assert!(errors[1].message.contains("3 type arguments, expected 2"));
        for expected in ["(2,3)", "19"] {
            assert!(events.iter().any(|e| matches!(e, SessionEvent::Value { value, .. }
                if value.to_string() == expected)), "{events:?}");
        }
    }

    #[test]
    fn named_type_tokens_preserve_type_queries_and_redefinitions() {
        let source = SourceText::new(concat!(
            "set_type MathRow = [int]\n",
            "whattype MathRow\n",
            "whattype MathRow ?\n",
            "set_type MathRow = [string]\n",
            "MathRow:[\"x\"]\n",
        ));
        let events = run_source(&source);
        assert!(!events.iter().any(|e| matches!(e, SessionEvent::Diagnostic(_))), "{events:?}");
        let queries = events.iter().filter(|e| matches!(e,
            SessionEvent::ReportLine { text, .. } if text == "Type defined at <standard input>:1:0-25:\n  MathRow = [int]\n")).count();
        assert_eq!(queries, 2, "{events:?}");
        assert!(events.iter().any(|e| matches!(e,
            SessionEvent::Value { value, .. } if value.to_string() == "[\"x\"]")));
    }

    #[test]
    fn named_types_are_not_expression_identifiers_or_parameter_bindings() {
        for program in ["MathRow", "set bad(int MathRow) = 1"] {
            let mut context = TypedContext::new();
            run_source_with_context(&SourceText::new("set_type MathRow = [int]\n"), &mut context);
            let events = run_source_with_context(&SourceText::new(program), &mut context);
            assert!(events.iter().any(|e| matches!(e,
                SessionEvent::Diagnostic(d) if d.kind == ErrorKind::Syntax)), "{events:?}");
        }
    }

    #[test]
    fn named_type_retention_covers_structural_consumers_and_lifetime() {
        // Before capture3834815 confirms all seven original results; simple
        // union discrimination and forget fail in the unchanged Rust binary.
        let cases = [
            (include_str!("../../../tests/math/generics/named_redefinition_retains_old.atlas"), "NAMED_HISTORY[2,3][5,6][\"x\"]\n"),
            (include_str!("../../../tests/math/generics/named_structural_equivalence.atlas"), "NAMED_EQUAL5[2,3]\n"),
            (include_str!("../../../tests/math/generics/named_row_operations.atlas"), "NAMED_ROW2[2,8][3,9,5]3\n"),
            (include_str!("../../../tests/math/generics/named_function_value.atlas"), "NAMED_FUNCTION35\n"),
            (include_str!("../../../tests/math/generics/named_field_copy.atlas"), "NAMED_FIELDS7s7s\n"),
            (include_str!("../../../tests/math/generics/named_union_discrimination.atlas"), "NAMED_UNION8\n"),
            (include_str!("../../../tests/math/generics/named_forget_binding.atlas"), "NAMED_FORGET7[3]\n"),
        ];
        for (program, expected) in cases {
            let events = run_source(&SourceText::new(program));
            assert!(!events.iter().any(|e| matches!(e, SessionEvent::Diagnostic(_))), "{program}\n{events:?}");
            assert!(events.iter().any(|e| matches!(e, SessionEvent::ReportLine { text, .. } if text == expected)), "{program}\n{events:?}");
        }
    }

    #[test]
    fn named_function_context_retains_declaration_names_and_values() {
        // All three whole streams matched before3839117. Checking only
        // numeric values missed its destructive function-result expansion.
        for (program, declaration, value) in [
            (include_str!("../../../tests/math/generics/named_function_value.atlas"),
                "Defined named_inc: MathFunction\n", "NAMED_FUNCTION35\n"),
            (include_str!("../../../tests/math/generics/named_function_application.atlas"),
                "Defined probe_named_function: MathFunction<int>\n", "NAMED_FUNCTION[7]\n"),
            (include_str!("../../../tests/math/generics/constructor_structural_uses_spaced.atlas"),
                "Defined f: MathFunction<int>\n", "FUNCTION12\n"),
        ] {
            let events = run_source(&SourceText::new(program));
            assert!(!events.iter().any(|e| matches!(e, SessionEvent::Diagnostic(_))), "{events:?}");
            for expected in [declaration, value] {
                assert!(events.iter().any(|e| matches!(e,
                    SessionEvent::ReportLine { text, .. } if text == expected)), "{events:?}");
            }
        }
    }

    #[test]
    fn named_redefinition_and_recursive_identity_reject_wrong_casts() {
        for program in [
            include_str!("../../../tests/math/generics/named_redefinition_old_rejected.atlas"),
            include_str!("../../../tests/math/generics/named_recursive_nominal_rejected.atlas"),
        ] {
            let events = run_source(&SourceText::new(program));
            let errors: Vec<_> = events.iter().filter_map(|e| match e {
                SessionEvent::Diagnostic(d) => Some(d), _ => None,
            }).collect();
            assert_eq!(errors.len(), 1, "{events:?}");
            assert_eq!(errors[0].kind, ErrorKind::Type, "{events:?}");
        }
    }

    #[test]
    fn named_void_and_primitive_contexts_follow_the_captured_oracle() {
        let program = concat!("set_type MathVoid = void\n",
            "set discarded = MathVoid:42\n", "set discarded_row = MathVoid:[1,2]\n",
            "set discarded_function = MathVoid:((int x):x+1)\n",
            "discarded\n", "discarded_row\n", "discarded_function\n");
        let events = run_source(&SourceText::new(program));
        assert!(!events.iter().any(|e| matches!(e, SessionEvent::Diagnostic(_))), "{events:?}");
        assert!(events.iter().any(|e| matches!(e, SessionEvent::Value { value, .. } if value.to_string() == "42")), "{events:?}");
        assert!(events.iter().any(|e| matches!(e, SessionEvent::Value { value, .. } if value.to_string() == "[1,2]")), "{events:?}");
        assert!(events.iter().any(|e| matches!(e, SessionEvent::Value { value: Value::Closure(_), .. })), "{events:?}");
        for program in [
            include_str!("../../../tests/math/generics/named_overload_alias_replacement.atlas"),
            include_str!("../../../tests/math/generics/named_member_redefinition.atlas"),
            include_str!("../../../tests/math/generics/named_primitive_context.atlas"),
        ] {
            let events = run_source(&SourceText::new(program));
            assert!(!events.iter().any(|e| matches!(e, SessionEvent::Diagnostic(_))), "{events:?}");
        }
    }

    #[test]
    fn named_recursive_result_annotations_are_resolved() {
        let source = SourceText::new(concat!(
            "set_type MathRow = [int]\n",
            "set rec_fun repeated(int n) = MathRow: if n=0 then [1] else repeated(n-1) fi\n",
            "repeated(2)\n",
            "set rec_fun invalid(int n) = MathRow: [\"abc\"]\n",
        ));
        let events = run_source(&source);
        let errors: Vec<_> = events.iter().filter_map(|e| match e {
            SessionEvent::Diagnostic(d) => Some(d), _ => None,
        }).collect();
        assert_eq!(errors.len(), 1, "{events:?}");
        assert_eq!(errors[0].kind, ErrorKind::Type, "{events:?}");
        assert!(events.iter().any(|e| matches!(e,
            SessionEvent::Value { value, .. } if value.to_string() == "[1]")));
    }

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
                | SessionEvent::OutputBytes { .. }
                | SessionEvent::ReportBytes { .. }
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
                | SessionEvent::OutputBytes { .. }
                | SessionEvent::ReportBytes { .. }
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
                    // Current original3836366: independent unknown operands
                    // unify with both signatures, so this is exact ambiguity.
                    && diagnostic.message == "Ambiguous argument in function call, argument type (A,B) matches both (int,int) and (rat,int)"
        ), "{events:?}");
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
    fn settype_b5_fixture_matches_current_named_definition_queries() {
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
            SessionEvent::ReportLine { ref text, .. } if text == "Type defined at <standard input>:1:0-31:\n  Pair = \n  ( int x\n  , int y\n  )\n"
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
                if text == "Type defined at <standard input>:9:0-56:\n  IntList = \n  ( void nil\n  | (int,IntList) cons\n  )\n"
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
    fn settype_b5_historical_rejection_now_accepts_simple_named_unions() {
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
            SessionEvent::Value { ref value, .. } if value.to_string() == "1"
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

    #[test]
    fn overloads_b8_fixture_matches_the_frozen_events() {
        let source = SourceText::new(include_str!(
            "../../../tests/fixtures/eval/overloads_b8.atlas"
        ));
        let events = run_source(&source);

        assert_eq!(events.len(), 5);
        assert!(matches!(
            events[0],
            SessionEvent::ReportLine { ref text, .. } if text == "Defined f: (int->int)\n"
        ));
        assert!(matches!(
            events[1],
            SessionEvent::ReportLine { ref text, .. }
                if text == "Added definition [2] of f: (int,int->int)\n"
        ));
        assert!(matches!(
            events[2],
            SessionEvent::ReportLine { ref text, .. }
                if text == "Overloaded instances of 'f'\n  int->int\n  (int,int)->int\n"
        ));
        assert!(matches!(
            events[3],
            SessionEvent::Value { value: Value::Integer(ref value), .. }
                if value == &malachite::Integer::from(3)
        ));
        assert!(matches!(
            events[4],
            SessionEvent::Value { value: Value::Integer(ref value), .. }
                if value == &malachite::Integer::from(7)
        ));
    }

    #[test]
    fn overloads_b8b_fixture_matches_the_frozen_events() {
        let source = SourceText::new(include_str!(
            "../../../tests/fixtures/eval/overloads_b8b.atlas"
        ));
        let events = run_source(&source);

        assert_eq!(events.len(), 8);
        assert!(matches!(
            events[0],
            SessionEvent::ReportLine { ref text, .. } if text == "Defined f: (int->int)\n"
        ));
        // Same signature: the variant is replaced, the count unchanged.
        assert!(matches!(
            events[1],
            SessionEvent::ReportLine { ref text, .. } if text == "Redefined f: (int->int)\n"
        ));
        assert!(matches!(
            events[2],
            SessionEvent::ReportLine { ref text, .. }
                if text == "Overloaded instances of 'f'\n  int->int\n"
        ));
        assert!(matches!(
            events[3],
            SessionEvent::Value { value: Value::Integer(ref value), .. }
                if value == &malachite::Integer::from(6)
        ));
        assert!(matches!(
            events[4],
            SessionEvent::ReportLine { ref text, .. } if text == "Defined g: (int->int)\n"
        ));
        // A non-function value joins the identifier table and coexists
        // with the overload of the same name.
        assert!(matches!(
            events[5],
            SessionEvent::ReportLine { ref text, .. } if text == "Variable g: int\n"
        ));
        assert!(matches!(
            events[6],
            SessionEvent::ReportLine { ref text, .. }
                if text == "Overloaded instances of 'g'\n  int->int\n"
        ));
        assert!(matches!(
            events[7],
            SessionEvent::Value { value: Value::Integer(ref value), .. }
                if value == &malachite::Integer::from(3)
        ));
    }

    #[test]
    fn overloads_b8_rejected_fixture_matches_the_frozen_events() {
        let source = SourceText::new(include_str!(
            "../../../tests/fixtures/eval/overloads_b8_rejected.atlas"
        ));
        let events = run_source(&source);

        assert_eq!(events.len(), 2);
        assert!(matches!(
            events[0],
            SessionEvent::ReportLine { ref text, .. } if text == "Defined f: (int->int)\n"
        ));
        // A single overload applies directly, so the mismatch names the
        // one argument type needed.
        assert!(matches!(
            events[1],
            SessionEvent::Diagnostic(ref diagnostic)
                if diagnostic.kind == ErrorKind::Type
                    && diagnostic.message == "found (int,int,int) while int was needed."
        ));
    }

    #[test]
    fn set_binds_several_declarations_in_parallel() {
        // do_global_set (global.w:911-994): every right-hand side analyses
        // against the tables as they were, so `x` in the second binding
        // refers to an OUTER definition, not the one being made.
        let source = SourceText::new("set x = 1, y = 2");
        let events = run_source(&source);

        assert_eq!(events.len(), 2);
        assert!(matches!(
            events[0],
            SessionEvent::ReportLine { ref text, .. } if text == "Variable x: int\n"
        ));
        assert!(matches!(
            events[1],
            SessionEvent::ReportLine { ref text, .. } if text == "Variable y: int\n"
        ));
    }
}
