//! Structured diagnostics shared by the parser and evaluator.

use std::fmt;
use std::ops::Range;

/// Stable identity assigned to a source buffer by its owner.
///
/// Zero is reserved for source text whose owner does not need to distinguish
/// it from other anonymous input.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourceId(u64);

impl SourceId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn anonymous() -> Self {
        Self(0)
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

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
    pub source_id: SourceId,
    pub byte_start: usize,
    pub byte_end: usize,
    pub start: SourcePosition,
    pub end: SourcePosition,
}

impl SourceSpan {
    pub fn new(
        source_id: SourceId,
        byte_start: usize,
        byte_end: usize,
        start: SourcePosition,
        end: SourcePosition,
    ) -> Self {
        assert!(byte_start <= byte_end, "source span start exceeds its end");
        Self {
            source_id,
            byte_start,
            byte_end,
            start,
            end,
        }
    }

    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub const fn byte_start(&self) -> usize {
        self.byte_start
    }

    pub const fn byte_end(&self) -> usize {
        self.byte_end
    }

    pub fn byte_range(&self) -> Range<usize> {
        self.byte_start..self.byte_end
    }
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
    /// A warning is reported to the user but does not make the session
    /// unclean (upstream lexical recovery prints to stderr without setting
    /// the clean flag; exit status stays 0).
    pub warning: bool,
}

impl Diagnostic {
    pub fn new(kind: ErrorKind, message: impl Into<String>, span: Option<SourceSpan>) -> Self {
        Self {
            kind,
            message: message.into(),
            span,
            warning: false,
        }
    }

    pub fn warning(kind: ErrorKind, message: impl Into<String>, span: Option<SourceSpan>) -> Self {
        Self {
            kind,
            message: message.into(),
            span,
            warning: true,
        }
    }

    pub fn is_warning(&self) -> bool {
        self.warning
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
