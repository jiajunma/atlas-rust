//! Owned source text and one-based line/column locations.

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceText {
    text: String,
    line_starts: Vec<usize>,
}

impl SourceText {
    pub fn new(text: impl Into<String>) -> Self {
        let text = text.into();
        let mut line_starts = vec![0];
        for (index, byte) in text.bytes().enumerate() {
            if byte == b'\n' {
                line_starts.push(index + 1);
            }
        }
        Self { text, line_starts }
    }

    pub fn as_str(&self) -> &str { &self.text }

    pub fn position(&self, byte: usize) -> crate::diagnostic::SourcePosition {
        let byte = byte.min(self.text.len());
        let line = self.line_starts.partition_point(|&start| start <= byte);
        let line_index = line.saturating_sub(1);
        let line_start = self.line_starts[line_index];
        // Lexical errors can point at the first byte of a non-ASCII scalar.
        // Count character starts rather than slicing at `byte`, which may be
        // in the middle of a UTF-8 sequence.
        let column = self.text[line_start..]
            .char_indices()
            .take_while(|(offset, _)| line_start + *offset < byte)
            .count()
            + 1;
        crate::diagnostic::SourcePosition {
            line: line_index + 1,
            column,
        }
    }

    pub fn span(&self, start: usize, end: usize) -> crate::diagnostic::SourceSpan {
        crate::diagnostic::SourceSpan { start: self.position(start), end: self.position(end) }
    }
}
