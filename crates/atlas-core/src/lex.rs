//! Atlas lexical tokens.
//!
//! The scanner is intentionally stateful.  Atlas comments can be nested and
//! an input may be consumed a token at a time by a parser or a REPL.  The
//! [`tokenize`] function remains as a compatibility convenience for callers
//! that want the complete token stream in one operation.

use crate::{
    diagnostic::{Diagnostic, ErrorKind},
    source::SourceText,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TokenKind {
    Keyword(String),
    Identifier,
    Integer,
    String,
    Operator(String),
    Punctuation(char),
    Newline,
    Eof,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    /// The exact source spelling, including quotes for string literals.
    pub lexeme: String,
    /// The decoded value where the lexical form has one (currently strings).
    pub value: Option<String>,
    pub span: crate::diagnostic::SourceSpan,
}

const KEYWORDS: &[&str] = &[
    "quit", "set", "let", "in", "begin", "end", "if", "then", "else", "elif", "fi",
    "and", "or", "not", "next", "do", "dont", "from", "downto", "while", "for", "od",
    "case", "esac", "rec_fun", "true", "false", "die", "break", "return", "set_type",
    "any_type", "whattype", "showall", "forget",
];

/// A stateful Atlas scanner.
///
/// `next_token` consumes one token.  Whitespace and comments are consumed
/// internally, while newlines are observable tokens.  After end of input (or
/// a fatal unterminated string/comment error), subsequent calls return EOF.
pub struct Lexer<'a> {
    source: &'a SourceText,
    offset: usize,
    ended: bool,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a SourceText) -> Self {
        Self {
            source,
            offset: 0,
            ended: false,
        }
    }

    pub fn source(&self) -> &'a SourceText {
        self.source
    }

    /// Current byte offset in the source.  This is useful for diagnostics and
    /// for parser integrations that need to identify the point of failure.
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Alias used by streaming consumers that treat the lexer as a cursor.
    pub fn next(&mut self) -> Result<Token, Diagnostic> {
        self.next_token()
    }

    pub fn next_token(&mut self) -> Result<Token, Diagnostic> {
        if self.ended {
            return Ok(self.eof_token());
        }

        // Copy the reference out of `self` so scanning can update `offset`
        // and call the nested scanners without borrowing the whole lexer.
        let source = self.source;
        let bytes = source.as_str().as_bytes();
        loop {
            if self.offset >= bytes.len() {
                self.ended = true;
                return Ok(self.eof_token());
            }

            let start = self.offset;
            match bytes[self.offset] {
                b' ' | b'\t' | b'\r' => {
                    self.offset += 1;
                }
                b'\n' => {
                    self.offset += 1;
                    return Ok(token(source, TokenKind::Newline, start, self.offset));
                }
                b'{' => {
                    self.consume_comment(start)?;
                }
                b'0'..=b'9' => {
                    self.offset += 1;
                    while self.offset < bytes.len() && bytes[self.offset].is_ascii_digit() {
                        self.offset += 1;
                    }
                    return Ok(token(source, TokenKind::Integer, start, self.offset));
                }
                b'_' | b'a'..=b'z' | b'A'..=b'Z' => {
                    self.offset += 1;
                    while self.offset < bytes.len()
                        && (bytes[self.offset].is_ascii_alphanumeric() || bytes[self.offset] == b'_')
                    {
                        self.offset += 1;
                    }
                    let word = &source.as_str()[start..self.offset];
                    let kind = if KEYWORDS.contains(&word) {
                        TokenKind::Keyword(word.to_owned())
                    } else {
                        TokenKind::Identifier
                    };
                    return Ok(token(source, kind, start, self.offset));
                }
                b'"' => return self.consume_string(start),
                b':' if bytes.get(self.offset + 1) == Some(&b'=') => {
                    self.offset += 2;
                    return Ok(token(
                        source,
                        TokenKind::Operator(":=".into()),
                        start,
                        self.offset,
                    ));
                }
                b'-' if bytes.get(self.offset + 1) == Some(&b'>') => {
                    self.offset += 2;
                    return Ok(token(
                        source,
                        TokenKind::Operator("->".into()),
                        start,
                        self.offset,
                    ));
                }
                b'~' if bytes.get(self.offset + 1) == Some(&b'[') => {
                    self.offset += 2;
                    return Ok(token(
                        source,
                        TokenKind::Operator("~[".into()),
                        start,
                        self.offset,
                    ));
                }
                b'+' | b'-' | b'*' | b'/' | b'=' | b'!' | b'<' | b'>' | b'&' | b'|' | b'@' => {
                    self.offset += 1;
                    return Ok(token(
                        source,
                        TokenKind::Operator(source.as_str()[start..self.offset].into()),
                        start,
                        self.offset,
                    ));
                }
                c if b"()[]{};,?.~".contains(&c) => {
                    self.offset += 1;
                    return Ok(token(
                        source,
                        TokenKind::Punctuation(c as char),
                        start,
                        self.offset,
                    ));
                }
                _ => {
                    self.offset += 1;
                    return Err(Diagnostic::new(
                        ErrorKind::Lexical,
                        "invalid character",
                        Some(source.span(start, self.offset)),
                    ));
                }
            }
        }
    }

    fn eof_token(&self) -> Token {
        Token {
            kind: TokenKind::Eof,
            lexeme: String::new(),
            value: None,
            span: self.source.span(self.offset, self.offset),
        }
    }

    fn consume_comment(&mut self, start: usize) -> Result<(), Diagnostic> {
        let bytes = self.source.as_str().as_bytes();
        self.offset += 1;
        let mut depth = 1usize;
        while self.offset < bytes.len() && depth > 0 {
            match bytes[self.offset] {
                b'{' => depth += 1,
                b'}' => depth -= 1,
                _ => {}
            }
            self.offset += 1;
        }
        if depth == 0 {
            Ok(())
        } else {
            self.ended = true;
            Err(Diagnostic::new(
                ErrorKind::Lexical,
                "unterminated comment",
                Some(self.source.span(start, self.offset)),
            ))
        }
    }

    fn consume_string(&mut self, start: usize) -> Result<Token, Diagnostic> {
        let bytes = self.source.as_str().as_bytes();
        self.offset += 1;
        let mut closed = false;
        while self.offset < bytes.len() {
            match bytes[self.offset] {
                // Atlas escapes a quote by doubling it.
                b'"' if self.offset + 1 < bytes.len() && bytes[self.offset + 1] == b'"' => {
                    self.offset += 2;
                }
                b'"' => {
                    self.offset += 1;
                    closed = true;
                    break;
                }
                b'\n' => break,
                _ => self.offset += 1,
            }
        }
        if !closed {
            self.ended = true;
            return Err(Diagnostic::new(
                ErrorKind::Lexical,
                "unterminated string",
                Some(self.source.span(start, self.offset)),
            ));
        }
        let raw = &self.source.as_str()[start + 1..self.offset - 1];
        Ok(Token {
            kind: TokenKind::String,
            lexeme: self.source.as_str()[start..self.offset].to_owned(),
            value: Some(raw.replace("\"\"", "\"")),
            span: self.source.span(start, self.offset),
        })
    }
}

/// A small lookahead cursor for parsers that need to inspect one token before
/// consuming it.  Lexical errors are cached just like tokens, so `peek` and
/// `bump` observe the same result.
pub struct TokenCursor<'a> {
    lexer: Lexer<'a>,
    lookahead: Option<Result<Token, Diagnostic>>,
}

impl<'a> TokenCursor<'a> {
    pub fn new(source: &'a SourceText) -> Self {
        Self {
            lexer: Lexer::new(source),
            lookahead: None,
        }
    }

    pub fn peek(&mut self) -> Result<&Token, &Diagnostic> {
        if self.lookahead.is_none() {
            self.lookahead = Some(self.lexer.next_token());
        }
        match self.lookahead.as_ref().expect("lookahead was filled") {
            Ok(token) => Ok(token),
            Err(error) => Err(error),
        }
    }

    pub fn bump(&mut self) -> Result<Token, Diagnostic> {
        self.lookahead
            .take()
            .unwrap_or_else(|| self.lexer.next_token())
    }

    pub fn next(&mut self) -> Result<Token, Diagnostic> {
        self.bump()
    }

    pub fn offset(&self) -> usize {
        self.lexer.offset()
    }
}

/// Consume a complete source while retaining the original all-or-nothing API.
pub fn tokenize(source: &SourceText) -> Result<Vec<Token>, Vec<Diagnostic>> {
    let mut lexer = Lexer::new(source);
    let mut tokens = Vec::new();
    let mut errors = Vec::new();
    loop {
        match lexer.next_token() {
            Ok(token) if token.kind == TokenKind::Eof => {
                if errors.is_empty() {
                    tokens.push(token);
                    return Ok(tokens);
                }
                return Err(errors);
            }
            Ok(token) => tokens.push(token),
            Err(error) => {
                errors.push(error);
                // Unterminated strings/comments are fatal and leave the
                // cursor at EOF.  Invalid characters are recoverable, so the
                // next call continues scanning after the offending byte.
                if lexer.ended {
                    return Err(errors);
                }
            }
        }
    }
}

fn token(source: &SourceText, kind: TokenKind, start: usize, end: usize) -> Token {
    Token {
        kind,
        lexeme: source.as_str()[start..end].to_owned(),
        value: None,
        span: source.span(start, end),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::ErrorKind;

    #[test]
    fn scans_keywords_literals_and_spans() {
        let source = SourceText::new("set x := 42\n");
        let tokens = tokenize(&source).expect("valid fixture");
        assert_eq!(tokens[0].kind, TokenKind::Keyword("set".into()));
        assert_eq!(tokens[1].kind, TokenKind::Identifier);
        assert_eq!(tokens[2].kind, TokenKind::Operator(":=".into()));
        assert_eq!(tokens[3].kind, TokenKind::Integer);
        assert_eq!(tokens[3].span.start.line, 1);
        assert_eq!(tokens[3].span.start.column, 10);
        assert_eq!(tokens[4].kind, TokenKind::Newline);
        assert_eq!(tokens[5].kind, TokenKind::Eof);
    }

    #[test]
    fn lexer_consumes_one_token_and_stays_at_eof() {
        let source = SourceText::new("x\n");
        let mut lexer = Lexer::new(&source);
        assert_eq!(lexer.next_token().expect("identifier").kind, TokenKind::Identifier);
        assert_eq!(lexer.next_token().expect("newline").kind, TokenKind::Newline);
        assert_eq!(lexer.next_token().expect("eof").kind, TokenKind::Eof);
        assert_eq!(lexer.next_token().expect("stable eof").kind, TokenKind::Eof);
    }

    #[test]
    fn cursor_peek_does_not_consume() {
        let source = SourceText::new("x -> y");
        let mut cursor = TokenCursor::new(&source);
        assert_eq!(cursor.peek().expect("peek").kind, TokenKind::Identifier);
        assert_eq!(cursor.bump().expect("bump").kind, TokenKind::Identifier);
        assert_eq!(cursor.bump().expect("arrow").kind, TokenKind::Operator("->".into()));
    }

    #[test]
    fn nested_comments_are_skipped_and_newline_after_comment_survives() {
        let source = SourceText::new("a { outer { inner } outer }\nb");
        let tokens = tokenize(&source).expect("valid comments");
        assert_eq!(tokens[0].kind, TokenKind::Identifier);
        assert_eq!(tokens[1].kind, TokenKind::Newline);
        assert_eq!(tokens[2].kind, TokenKind::Identifier);
        assert_eq!(tokens[3].kind, TokenKind::Eof);
    }

    #[test]
    fn strings_and_multi_character_operators_keep_raw_lexemes() {
        let source = SourceText::new("\"a\"\"b\" := x -> y ~[ z");
        let tokens = tokenize(&source).expect("valid fixture");
        assert_eq!(tokens[0].kind, TokenKind::String);
        assert_eq!(tokens[0].lexeme, "\"a\"\"b\"");
        assert_eq!(tokens[0].value.as_deref(), Some("a\"b"));
        assert_eq!(tokens[1].kind, TokenKind::Operator(":=".into()));
        assert_eq!(tokens[2].kind, TokenKind::Identifier);
        assert_eq!(tokens[3].kind, TokenKind::Operator("->".into()));
        assert_eq!(tokens[5].kind, TokenKind::Operator("~[".into()));
    }

    #[test]
    fn punctuation_is_emitted_individually() {
        let tokens = tokenize(&SourceText::new("( ) [ ] ; , ? . ~"))
            .expect("valid punctuation");
        let punctuation: Vec<_> = tokens
            .iter()
            .take(9)
            .map(|token| token.kind.clone())
            .collect();
        assert_eq!(
            punctuation,
            vec![
                TokenKind::Punctuation('('),
                TokenKind::Punctuation(')'),
                TokenKind::Punctuation('['),
                TokenKind::Punctuation(']'),
                TokenKind::Punctuation(';'),
                TokenKind::Punctuation(','),
                TokenKind::Punctuation('?'),
                TokenKind::Punctuation('.'),
                TokenKind::Punctuation('~'),
            ]
        );
    }

    #[test]
    fn unterminated_string_and_comment_are_fatal_cursor_errors() {
        for input in ["\"oops\n", "{oops"] {
            let source = SourceText::new(input);
            let mut lexer = Lexer::new(&source);
            let error = lexer.next_token().expect_err("must reject");
            assert_eq!(error.kind, ErrorKind::Lexical);
            assert_eq!(lexer.next_token().expect("eof").kind, TokenKind::Eof);
        }
    }

    #[test]
    fn invalid_characters_are_recoverable() {
        let source = SourceText::new("a#b");
        let mut lexer = Lexer::new(&source);
        assert_eq!(lexer.next_token().expect("a").kind, TokenKind::Identifier);
        assert_eq!(lexer.next_token().expect_err("# is invalid").kind, ErrorKind::Lexical);
        assert_eq!(lexer.next_token().expect("b").kind, TokenKind::Identifier);
    }

    #[test]
    fn unicode_invalid_byte_does_not_break_source_positions() {
        let source = SourceText::new("é");
        let errors = tokenize(&source).expect_err("non-ASCII identifiers are not yet supported");
        assert_eq!(errors[0].span.expect("span").start.column, 1);
    }
}
