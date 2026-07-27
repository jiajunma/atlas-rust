//! Owned source text and one-based line/column locations.

use crate::diagnostic::{SourceId, SourcePosition, SourceSpan};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceText {
    source_id: SourceId,
    text: String,
    line_starts: Vec<usize>,
}

impl SourceText {
    pub fn new(text: impl Into<String>) -> Self {
        Self::with_id(SourceId::anonymous(), text)
    }

    pub fn with_id(source_id: SourceId, text: impl Into<String>) -> Self {
        let text = text.into();
        let mut line_starts = vec![0];
        for (index, byte) in text.bytes().enumerate() {
            if byte == b'\n' {
                line_starts.push(index + 1);
            }
        }
        Self {
            source_id,
            text,
            line_starts,
        }
    }

    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub fn as_str(&self) -> &str {
        &self.text
    }

    pub fn position(&self, byte: usize) -> SourcePosition {
        let mut byte = byte.min(self.text.len());
        while !self.text.is_char_boundary(byte) {
            byte -= 1;
        }
        let line = self.line_starts.partition_point(|&start| start <= byte);
        let line_index = line.saturating_sub(1);
        let line_start = self.line_starts[line_index];
        let column = self.text[line_start..byte].chars().count() + 1;
        SourcePosition {
            line: line_index + 1,
            column,
        }
    }

    pub fn span(&self, start: usize, end: usize) -> SourceSpan {
        let start = start.min(self.text.len());
        let end = end.min(self.text.len());
        SourceSpan::new(
            self.source_id,
            start,
            end,
            self.position(start),
            self.position(end),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::{SourceId, SourcePosition};

    #[test]
    fn new_uses_stable_anonymous_source_id() {
        let first = SourceText::new("first");
        let second = SourceText::new("second");

        assert_eq!(first.source_id(), SourceId::anonymous());
        assert_eq!(first.source_id(), second.source_id());
    }

    #[test]
    fn with_id_is_preserved_by_spans() {
        let source = SourceText::with_id(SourceId::new(42), "alpha");
        let span = source.span(1, 4);

        assert_eq!(source.source_id(), SourceId::new(42));
        assert_eq!(span.source_id(), SourceId::new(42));
        assert_eq!(span.byte_start(), 1);
        assert_eq!(span.byte_end(), 4);
        assert_eq!(span.byte_range(), 1..4);
        assert_eq!(span.start, SourcePosition { line: 1, column: 2 });
        assert_eq!(span.end, SourcePosition { line: 1, column: 5 });
    }

    #[test]
    fn position_counts_unicode_scalars_and_newlines() {
        let source = SourceText::with_id(SourceId::new(9), "aé\n中x");

        assert_eq!(source.position(0), SourcePosition { line: 1, column: 1 });
        assert_eq!(source.position(1), SourcePosition { line: 1, column: 2 });
        assert_eq!(source.position(2), SourcePosition { line: 1, column: 2 });
        assert_eq!(source.position(3), SourcePosition { line: 1, column: 3 });
        assert_eq!(source.position(4), SourcePosition { line: 2, column: 1 });
        assert_eq!(source.position(6), SourcePosition { line: 2, column: 1 });
        assert_eq!(source.position(7), SourcePosition { line: 2, column: 2 });
        assert_eq!(source.position(8), SourcePosition { line: 2, column: 3 });
        assert_eq!(source.position(99), SourcePosition { line: 2, column: 3 });

        let span = source.span(1, 7);
        assert_eq!(span.byte_range(), 1..7);
        assert_eq!(span.start, SourcePosition { line: 1, column: 2 });
        assert_eq!(span.end, SourcePosition { line: 2, column: 2 });
    }
}
