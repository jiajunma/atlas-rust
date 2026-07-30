//! Regression coverage for the typed session pipeline.
//!
//! These tests deliberately enter through `session::run_source` (or its
//! context-taking sibling).  Keeping the fixtures at this boundary makes the
//! tests exercise command recovery, typed conversion, and evaluation together
//! without retaining the removed dynamic evaluator as a second implementation.

use malachite::{Integer as BigInt, Rational as BigRational};

use crate::{
    diagnostic::ErrorKind,
    session::{run_source, run_source_with_context, SessionEvent},
    source::SourceText,
    typed::TypedContext,
    value::Value,
};

fn integer(value: impl Into<BigInt>) -> Value {
    Value::Integer(value.into())
}

fn rational(numerator: impl Into<BigInt>, denominator: impl Into<BigInt>) -> Value {
    Value::Rational(BigRational::from_integers(
        numerator.into(),
        denominator.into(),
    ))
}

fn values(events: &[SessionEvent]) -> Vec<Value> {
    events
        .iter()
        .filter_map(|event| match event {
            SessionEvent::Value { value, .. } => Some(value.clone()),
            SessionEvent::Output { .. }
            | SessionEvent::ReportLine { .. }
            | SessionEvent::Diagnostic(_) => None,
        })
        .collect()
}

fn diagnostics(events: &[SessionEvent]) -> Vec<crate::diagnostic::Diagnostic> {
    events
        .iter()
        .filter_map(|event| match event {
            SessionEvent::Diagnostic(diagnostic) => Some(diagnostic.clone()),
            SessionEvent::Value { .. }
            | SessionEvent::Output { .. }
            | SessionEvent::ReportLine { .. } => None,
        })
        .collect()
}

fn assert_fixture_values(source: &'static str, expected: Vec<Value>) {
    let events = run_source(&SourceText::new(source));
    assert!(
        diagnostics(&events).is_empty(),
        "fixture emitted diagnostics: {events:?}"
    );
    assert_eq!(values(&events), expected);
}

fn assert_single_runtime_error(source: &'static str, expected_message: &str) {
    let events = run_source(&SourceText::new(source));
    assert_eq!(events.len(), 1, "one-line error fixture: {events:?}");
    let diagnostic = single_diagnostic(&events);
    assert_eq!(diagnostic.kind, ErrorKind::Runtime);
    assert_eq!(diagnostic.message, expected_message);
}

fn single_diagnostic(events: &[SessionEvent]) -> crate::diagnostic::Diagnostic {
    let mut all = diagnostics(events);
    assert_eq!(all.len(), 1, "expected one diagnostic, got {events:?}");
    all.remove(0)
}

#[test]
fn scalar_fixture_preserves_source_order_and_values() {
    assert_fixture_values(
        include_str!("../../../tests/fixtures/eval/scalars.atlas"),
        vec![
            integer(42),
            Value::Boolean(true),
            Value::String("atlas".into()),
            rational(7, 1),
            Value::Boolean(true),
            Value::Boolean(true),
        ],
    );
}

#[test]
fn context_fixture_reads_a_binding_from_the_same_typed_context() {
    let mut context = TypedContext::new();
    let setup = run_source_with_context(&SourceText::new("answer: 41\n"), &mut context);
    assert!(
        diagnostics(&setup).is_empty(),
        "context setup failed: {setup:?}"
    );

    let events = run_source_with_context(
        &SourceText::new(include_str!("../../../tests/fixtures/eval/context.atlas")),
        &mut context,
    );
    assert!(
        diagnostics(&events).is_empty(),
        "context fixture failed: {events:?}"
    );
    assert_eq!(values(&events), vec![integer(41), integer(42)]);
}

#[test]
fn exact_numeric_fixture_keeps_big_integer_and_rational_precision() {
    assert_fixture_values(
        include_str!("../../../tests/fixtures/eval/exact_numerics.atlas"),
        vec![
            integer(
                "1234567890123456789012345678901234567890"
                    .parse::<BigInt>()
                    .expect("valid expected integer"),
            ),
            integer(
                "-1234567890123456789012345678901234567890"
                    .parse::<BigInt>()
                    .expect("valid expected integer"),
            ),
            rational(-1, 2),
            integer(
                "1000000000000000000000000000000000000000000000000000000000000"
                    .parse::<BigInt>()
                    .expect("valid expected integer"),
            ),
            integer(1),
            rational(1, 3),
            rational(1, 1),
            Value::Boolean(true),
            Value::Boolean(true),
            rational(5, 6),
            rational(2, 1),
        ],
    );
}

#[test]
fn container_fixture_preserves_nested_row_coercions() {
    assert_fixture_values(
        include_str!("../../../tests/fixtures/eval/containers.atlas"),
        vec![
            Value::Tuple(vec![
                integer(1),
                Value::String("a".into()),
                Value::Boolean(true),
            ]),
            Value::List(vec![integer(1), integer(2), integer(3)]),
            Value::List(Vec::new()),
            Value::List(vec![rational(1, 1), rational(1, 2)]),
            Value::List(vec![
                Value::List(vec![rational(1, 1)]),
                Value::List(vec![rational(1, 2)]),
            ]),
            Value::List(vec![
                Value::Tuple(vec![integer(1), rational(2, 1)]),
                Value::Tuple(vec![integer(3), rational(1, 2)]),
            ]),
            Value::Tuple(vec![integer(0), integer(1)]),
        ],
    );
}

#[test]
fn subscription_fixture_uses_forward_and_reverse_row_indices() {
    assert_fixture_values(
        include_str!("../../../tests/fixtures/eval/subscriptions.atlas"),
        vec![
            integer(10),
            integer(30),
            integer(30),
            integer(10),
            integer(3),
            rational(1, 1),
        ],
    );
}

#[test]
fn slice_fixture_uses_half_open_bounds_and_empty_results() {
    assert_fixture_values(
        include_str!("../../../tests/fixtures/eval/slices.atlas"),
        vec![
            Value::List(vec![integer(20), integer(30)]),
            Value::List(vec![integer(10), integer(20)]),
            Value::List(vec![integer(30), integer(40)]),
            Value::List(vec![integer(10), integer(20), integer(30), integer(40)]),
            Value::List(Vec::new()),
            Value::List(Vec::new()),
            Value::List(Vec::new()),
            Value::List(Vec::new()),
        ],
    );
}

#[test]
fn scalar_overload_fixture_covers_rational_and_symbolic_calls() {
    assert_fixture_values(
        include_str!("../../../tests/fixtures/eval/scalar_overloads.atlas"),
        vec![
            rational(1, 2),
            rational(5, 2),
            Value::Tuple(vec![integer(5), integer(2)]),
            rational(9, 2),
            rational(1, 2),
            rational(5, 1),
            rational(5, 4),
            integer(1),
            rational(1, 2),
            rational(17, 6),
            rational(1, 2),
            rational(4, 9),
            Value::Boolean(true),
            Value::Boolean(true),
            Value::Boolean(false),
            Value::Boolean(false),
            integer(-2),
            Value::String("ab".into()),
        ],
    );
}

#[test]
fn negative_fixtures_report_name_and_type_categories() {
    let undefined = run_source(&SourceText::new(include_str!(
        "../../../tests/fixtures/eval/negative_undefined.atlas"
    )));
    let diagnostic = single_diagnostic(&undefined);
    assert_eq!(diagnostic.kind, ErrorKind::Name);
    assert_eq!(diagnostic.message, "Undefined identifier 'missing'");

    let type_error = run_source(&SourceText::new(include_str!(
        "../../../tests/fixtures/eval/negative_type.atlas"
    )));
    let diagnostic = single_diagnostic(&type_error);
    assert_eq!(diagnostic.kind, ErrorKind::Type);
    assert_eq!(
        diagnostic.message,
        "Failed to match '+' with argument type (bool,int)"
    );
}

#[test]
fn container_error_fixture_recovers_each_command_boundary() {
    let events = run_source(&SourceText::new(include_str!(
        "../../../tests/fixtures/eval/container_errors.atlas"
    )));
    assert_eq!(
        events.len(),
        4,
        "one event per malformed command: {events:?}"
    );
    assert!(events.iter().take(3).all(|event| matches!(
        event,
        SessionEvent::Diagnostic(diagnostic) if diagnostic.kind == ErrorKind::Type
    )));
    assert!(matches!(
        events[3],
        SessionEvent::Diagnostic(ref diagnostic) if diagnostic.kind == ErrorKind::Runtime
    ));
}

#[test]
fn scalar_error_fixtures_preserve_oracle_runtime_messages() {
    let cases = [
        (
            include_str!("../../../tests/fixtures/eval/scalar_error_fraction_zero.atlas"),
            "Inverse of zero",
        ),
        (
            include_str!("../../../tests/fixtures/eval/scalar_error_int_power_negative.atlas"),
            "Negative power of integer",
        ),
        (
            include_str!("../../../tests/fixtures/eval/scalar_error_int_power_large.atlas"),
            "Exponent too large in power of integer",
        ),
        (
            include_str!("../../../tests/fixtures/eval/scalar_error_rat_power_negative.atlas"),
            "Negative integer where unsigned is required",
        ),
        (
            include_str!("../../../tests/fixtures/eval/scalar_error_rat_divide_zero.atlas"),
            "Division of rational by integer zero",
        ),
        (
            include_str!("../../../tests/fixtures/eval/scalar_error_rat_quotient_zero.atlas"),
            "Rational quotient by zero",
        ),
        (
            include_str!("../../../tests/fixtures/eval/scalar_error_rat_modulo_zero.atlas"),
            "Division by zero",
        ),
    ];
    for (source, expected_message) in cases {
        assert_single_runtime_error(source, expected_message);
    }
}

#[test]
fn typed_validation_precedes_short_circuit_runtime_evaluation() {
    let events = run_source(&SourceText::new(
        "false and ((1 / 0) = 0)\ntrue or ((1 / 0) = 0)",
    ));
    assert_eq!(
        values(&events),
        vec![Value::Boolean(false), Value::Boolean(true)]
    );
    assert!(
        diagnostics(&events).is_empty(),
        "short circuit failed: {events:?}"
    );

    for source in ["false and 1", "1 and missing", "true = 1"] {
        let events = run_source(&SourceText::new(source));
        let diagnostic = single_diagnostic(&events);
        assert_eq!(diagnostic.kind, ErrorKind::Type, "source: {source}");
    }
}

#[test]
fn exact_scalar_edges_keep_euclidean_quotients_and_tuple_divmod() {
    let events = run_source(&SourceText::new(
        "(-7) \\ 3\n(-7) % 3\n7 \\ -3\n7 % -3\n1 \\% 2",
    ));
    assert!(
        diagnostics(&events).is_empty(),
        "scalar edge failed: {events:?}"
    );
    assert_eq!(
        values(&events),
        vec![
            integer(-3),
            integer(2),
            integer(-3),
            integer(-2),
            Value::Tuple(vec![integer(0), integer(1)]),
        ]
    );
}

#[test]
fn typed_container_errors_have_the_expected_phase() {
    for (source, expected_kind) in [
        // Undetermined row components do not match a concrete overload in
        // the typed Atlas pipeline; this is intentionally a static error.
        ("[][0] + 1", ErrorKind::Type),
        ("[[][0], 1]", ErrorKind::Runtime),
        ("[][0] + true", ErrorKind::Type),
        ("[][0] % (1 / 2)", ErrorKind::Type),
        ("(not [][0]) + 1", ErrorKind::Type),
        ("([][0] and true) + 1", ErrorKind::Type),
        ("([][0] % 1) + true", ErrorKind::Type),
        ("[1, \"bad\"]", ErrorKind::Type),
    ] {
        let events = run_source(&SourceText::new(source));
        let diagnostic = single_diagnostic(&events);
        assert_eq!(diagnostic.kind, expected_kind, "source: {source}");
    }
}

#[test]
fn involution_table_fixture_prints_the_frozen_kgb_and_strong_real_text() {
    let events = run_source(&SourceText::new(include_str!(
        "../../../tests/fixtures/domain/involution_table.atlas"
    )));
    assert!(
        diagnostics(&events).is_empty(),
        "fixture failed: {events:?}"
    );
    let reports: Vec<&str> = events
        .iter()
        .filter_map(|event| match event {
            SessionEvent::ReportLine { text, .. } => Some(text.as_str()),
            SessionEvent::Value { .. }
            | SessionEvent::Output { .. }
            | SessionEvent::Diagnostic(_) => None,
        })
        .collect();
    assert_eq!(
        reports,
        vec![
            "Declaring identifier 'rd': RootDatum\n",
            "Declaring identifier 'ic': InnerClass\n",
            "Declaring identifier 'rf': RealForm\n",
            "kgbsize: 3\nBase grading: [1].\n0:  0  [n]   1    2  (0)#0 e\n1:  0  [n]   0    2  (1)#0 e\n2:  1  [r]   2    *  (0)#1 1^e\n",
            "Declaring identifier 'rc': RealForm\n",
            "kgbsize: 1\nBase grading: [0].\n0:  0  [c]   0    *  (0)#0 e\n",
            "Declaring identifier 'cc': CartanClass\n",
            "class #0, possible square: exp(2i\\pi([1]/2))\nreal form #1: [0] (1)\n",
        ]
    );
}

#[test]
fn involution_table_rejected_fixture_is_the_two_overload_wording() {
    let events = run_source(&SourceText::new(include_str!(
        "../../../tests/fixtures/domain/involution_table_rejected.atlas"
    )));
    let diagnostic = single_diagnostic(&events);
    assert_eq!(diagnostic.kind, ErrorKind::Type);
    assert_eq!(
        diagnostic.message,
        "Failed to match 'print_KGB' with argument type RootDatum"
    );
}

#[test]
fn split_fixture_covers_dual_number_arithmetic() {
    let events = run_source(&SourceText::new(include_str!(
        "../../../tests/fixtures/eval/split_basic.atlas"
    )));
    assert!(
        diagnostics(&events).is_empty(),
        "split fixture emitted diagnostics: {events:?}"
    );
    let displays: Vec<String> = values(&events).iter().map(ToString::to_string).collect();
    assert_eq!(
        displays,
        vec![
            "(3+2s)", "(3+2s)", "(5+0s)", "(5+0s)", "(8+2s)", "(-2+2s)", "(15+10s)", "(-3-2s)",
            "false", "true", "(3,2)", "(4+2s)",
        ]
    );
}

#[test]
fn split_rejected_fixture_is_the_missing_division_overload() {
    let events = run_source(&SourceText::new(include_str!(
        "../../../tests/fixtures/eval/split_basic_rejected.atlas"
    )));
    let diagnostic = single_diagnostic(&events);
    assert_eq!(diagnostic.kind, ErrorKind::Type);
    assert_eq!(
        diagnostic.message,
        "Failed to match '/' with argument type (Split,int)"
    );
}
