//! Owned source text and one-based line/column locations.

use crate::diagnostic::{SourceId, SourcePosition, SourceSpan};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceText {
    source_id: SourceId,
    text: String,
    line_starts: Vec<usize>,
}

/// An incremental line/column cursor for consumers that scan a source
/// front-to-back (the lexer takes a span per token, and on a single-line
/// file the independent `position()` calls cost O(line) per token — O(n^2)
/// over the file). Position queries for non-decreasing byte offsets advance
/// the cursor by the distance only; a rare rewind (lookahead probes,
/// out-of-order diagnostic spans) falls back to a one-off prefix rescan.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LineCursor {
    /// Byte offset the position below describes (always a char boundary).
    offset: usize,
    /// Zero-based line index of `offset`.
    line_index: usize,
    /// One-based column of `offset`, counting Unicode scalar values.
    column: usize,
}

impl LineCursor {
    pub fn new() -> Self {
        Self {
            offset: 0,
            line_index: 0,
            column: 1,
        }
    }
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

    /// `byte` clamped into the text and snapped back onto a char boundary
    /// (a position inside a multi-byte scalar resolves to its start).
    fn adjusted(&self, byte: usize) -> usize {
        let mut byte = byte.min(self.text.len());
        while !self.text.is_char_boundary(byte) {
            byte -= 1;
        }
        byte
    }

    fn line_index(&self, byte: usize) -> usize {
        self.line_starts
            .partition_point(|&start| start <= byte)
            .saturating_sub(1)
    }

    /// One-based column of `byte` within the line starting at `line_start`
    /// (both char boundaries): counting Unicode scalar values is counting
    /// the non-continuation bytes, which needs no decoding.
    fn column(&self, line_start: usize, byte: usize) -> usize {
        self.text.as_bytes()[line_start..byte]
            .iter()
            .filter(|&&b| b & 0xC0 != 0x80)
            .count()
            + 1
    }

    pub fn position(&self, byte: usize) -> SourcePosition {
        let byte = self.adjusted(byte);
        let line_index = self.line_index(byte);
        SourcePosition {
            line: line_index + 1,
            column: self.column(self.line_starts[line_index], byte),
        }
    }

    pub fn span(&self, start: usize, end: usize) -> SourceSpan {
        let byte_start = start.min(self.text.len());
        let byte_end = end.min(self.text.len());
        let (start, end) = (self.adjusted(start), self.adjusted(end));
        let start_line = self.line_index(start);
        let end_line = self.line_index(end);
        let start_column = self.column(self.line_starts[start_line], start);
        // Spans almost never cross lines: the end column then EXTENDS the
        // start column instead of rescanning the line prefix.
        let end_column = if end_line == start_line && start <= end {
            start_column
                + self.text.as_bytes()[start..end]
                    .iter()
                    .filter(|&&b| b & 0xC0 != 0x80)
                    .count()
        } else {
            self.column(self.line_starts[end_line], end)
        };
        SourceSpan::new(
            self.source_id,
            byte_start,
            byte_end,
            SourcePosition {
                line: start_line + 1,
                column: start_column,
            },
            SourcePosition {
                line: end_line + 1,
                column: end_column,
            },
        )
    }

    /// [`SourceText::position`] through an incremental cursor: advances from
    /// the cursor's byte to `byte` (counting newlines and non-continuation
    /// bytes), or rescans the line prefix once when `byte` lies behind the
    /// cursor. Same positions as `position`, byte for byte.
    pub fn position_with_cursor(&self, cursor: &mut LineCursor, byte: usize) -> SourcePosition {
        let byte = self.adjusted(byte);
        if byte < cursor.offset {
            let line_index = self.line_index(byte);
            let column = self.column(self.line_starts[line_index], byte);
            *cursor = LineCursor {
                offset: byte,
                line_index,
                column,
            };
            return SourcePosition {
                line: line_index + 1,
                column,
            };
        }
        let bytes = self.text.as_bytes();
        while cursor.offset < byte {
            match bytes[cursor.offset] {
                b'\n' => {
                    cursor.line_index += 1;
                    cursor.column = 1;
                }
                // Continuation bytes are not scalar values: no column.
                b if b & 0xC0 != 0x80 => cursor.column += 1,
                _ => {}
            }
            cursor.offset += 1;
        }
        SourcePosition {
            line: cursor.line_index + 1,
            column: cursor.column,
        }
    }

    /// [`SourceText::span`] through an incremental cursor (see
    /// [`SourceText::position_with_cursor`]); the lexer's per-token path.
    pub fn span_with_cursor(
        &self,
        cursor: &mut LineCursor,
        start: usize,
        end: usize,
    ) -> SourceSpan {
        let byte_start = start.min(self.text.len());
        let byte_end = end.min(self.text.len());
        let start = self.position_with_cursor(cursor, start);
        let end = self.position_with_cursor(cursor, end);
        SourceSpan::new(self.source_id, byte_start, byte_end, start, end)
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

    #[test]
    fn span_positions_match_the_independent_position_calls() {
        let source = SourceText::with_id(SourceId::new(3), "aé bc\n中x def\nlast é\n");
        for start in 0..=source.as_str().len() + 1 {
            for end in 0..=source.as_str().len() + 1 {
                if start.min(source.as_str().len()) > end.min(source.as_str().len()) {
                    continue; // SourceSpan::new requires start <= end
                }
                let span = source.span(start, end);
                assert_eq!(
                    (span.start, span.end),
                    (source.position(start), source.position(end)),
                    "span({start}, {end})"
                );
            }
        }
    }

    #[test]
    fn cursor_spans_match_the_independent_position_calls() {
        // Multiline + Unicode text, walked the way the lexer walks: token
        // spans in scan order (monotonic), with out-of-order probes mixed in
        // to exercise the rewind fallback.
        let source = SourceText::with_id(SourceId::new(4), "aé bc\n中x def\nlast é\n");
        let mut cursor = LineCursor::new();
        let len = source.as_str().len();
        for end in 0..=len + 1 {
            for start in (0..=end).step_by(3) {
                let span = source.span_with_cursor(&mut cursor, start, end);
                assert_eq!(
                    (span.start, span.end),
                    (source.position(start), source.position(end)),
                    "span_with_cursor({start}, {end})"
                );
                assert_eq!(span.byte_range(), start.min(len)..end.min(len));
            }
        }
        // A single very long line (the E8_small_block shape): monotonic
        // per-token spans must stay correct without rescanning the prefix.
        let long = SourceText::with_id(SourceId::new(5), "x".repeat(100_000));
        let mut cursor = LineCursor::new();
        for start in (0..100_000).step_by(997) {
            let span = long.span_with_cursor(&mut cursor, start, start + 500);
            assert_eq!(
                (span.start, span.end),
                (long.position(start), long.position(start + 500)),
                "long-line span at {start}"
            );
        }
    }
}
