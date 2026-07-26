//! Structured diagnostics shared by the parser and evaluator.

use std::fmt;

/// A source position using one-based line and column numbers, matching the
/// user-facing Atlas convention.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourcePosition {
    pub line: usize,
    pub column: usize,
}

/// A source span with an inclusive start and exclusive end.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceSpan {
    pub start: SourcePosition,
    pub end: SourcePosition,
}

/// Stable error category. Messages are deliberately separate from category so
/// clients can compare behavior without depending on wording.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    Lexical,
    Syntax,
    Name,
    Type,
    Runtime,
    Io,
}

/// A diagnostic that can be rendered by the CLI or consumed by an editor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    pub kind: ErrorKind,
    pub message: String,
    pub span: Option<SourceSpan>,
}

impl Diagnostic {
    pub fn new(kind: ErrorKind, message: impl Into<String>, span: Option<SourceSpan>) -> Self {
        Self {
            kind,
            message: message.into(),
            span,
        }
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(span) = self.span {
            write!(
                formatter,
                "{}:{}: {}",
                span.start.line, span.start.column, self.message
            )
        } else {
            formatter.write_str(&self.message)
        }
    }
}

impl std::error::Error for Diagnostic {}
