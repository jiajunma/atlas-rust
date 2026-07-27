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
    Unsupported(String),
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
    "quit", "set", "let", "in", "begin", "end", "if", "then", "else", "elif", "fi", "and", "or",
    "not", "next", "do", "dont", "from", "downto", "while", "for", "od", "case", "esac", "rec_fun",
    "true", "false", "die", "break", "return", "set_type", "whattype", "showall", "forget",
];

/// A stateful Atlas scanner.
///
/// `next_token` consumes one token. Whitespace and comments are consumed
/// internally, while only newlines that can terminate the current command are
/// observable tokens. The command-boundary state mirrors the upstream lexer:
/// grouping constructs and continuation tokens suppress a newline.
pub struct Lexer<'a> {
    source: &'a SourceText,
    offset: usize,
    ended: bool,
    pending: Option<Token>,
    nesting: Vec<NestingKind>,
    prevent_termination: Option<char>,
    previous_termination: Option<char>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum NestingKind {
    Group,
    Let,
    Block,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a SourceText) -> Self {
        Self {
            source,
            offset: 0,
            ended: false,
            pending: None,
            nesting: Vec::new(),
            prevent_termination: None,
            previous_termination: None,
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

    /// Discard the rest of the physical line after the parser rejects a
    /// command and reset continuation state for the next command.
    pub fn recover_command(&mut self) {
        let bytes = self.source.as_str().as_bytes();
        while self.offset < bytes.len() {
            let byte = bytes[self.offset];
            self.offset += 1;
            if byte == b'\n' {
                break;
            }
        }
        self.pending = None;
        self.nesting.clear();
        self.prevent_termination = None;
        self.previous_termination = None;
        self.ended = self.offset >= bytes.len();
    }

    /// Alias used by streaming consumers that treat the lexer as a cursor.
    pub fn next(&mut self) -> Result<Token, Diagnostic> {
        self.next_token()
    }

    pub fn next_token(&mut self) -> Result<Token, Diagnostic> {
        if let Some(token) = self.pending.take() {
            return Ok(token);
        }
        if self.ended {
            return Ok(self.eof_token());
        }

        let source = self.source;
        let bytes = source.as_str().as_bytes();
        self.skip_space()?;
        if self.offset >= bytes.len() {
            self.ended = true;
            return Ok(self.eof_token());
        }

        let start = self.offset;
        self.previous_termination = self.prevent_termination.take();
        match bytes[self.offset] {
            b'\n' => {
                self.offset += 1;
                Ok(token(source, TokenKind::Newline, start, self.offset))
            }
            b'0'..=b'9' => {
                self.offset += 1;
                while self.offset < bytes.len() && bytes[self.offset].is_ascii_digit() {
                    self.offset += 1;
                }
                Ok(token(source, TokenKind::Integer, start, self.offset))
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
                    self.apply_keyword(word);
                    TokenKind::Keyword(word.to_owned())
                } else {
                    TokenKind::Identifier
                };
                Ok(token(source, kind, start, self.offset))
            }
            b'"' => self.consume_string(start),
            b':' if bytes.get(self.offset + 1) == Some(&b'=') => {
                self.offset += 2;
                self.prevent_termination = Some(':');
                Ok(token(
                    source,
                    TokenKind::Operator(":=".into()),
                    start,
                    self.offset,
                ))
            }
            b'!' if bytes.get(self.offset + 1) == Some(&b'=') => {
                self.offset += 2;
                self.operator_termination('=');
                Ok(token(
                    source,
                    TokenKind::Operator("!=".into()),
                    start,
                    self.offset,
                ))
            }
            b'<' | b'>' if bytes.get(self.offset + 1) == Some(&b'=') => {
                let operator = source.as_str()[start..self.offset + 2].to_owned();
                self.offset += 2;
                self.operator_termination(operator.as_bytes()[0] as char);
                Ok(token(
                    source,
                    TokenKind::Operator(operator),
                    start,
                    self.offset,
                ))
            }
            b'-' if bytes.get(self.offset + 1) == Some(&b'>') => {
                self.offset += 2;
                self.operator_termination('-');
                Ok(token(
                    source,
                    TokenKind::Operator("->".into()),
                    start,
                    self.offset,
                ))
            }
            b'~' if bytes.get(self.offset + 1) == Some(&b'[') => {
                self.offset += 2;
                self.nesting.push(NestingKind::Group);
                Ok(token(
                    source,
                    TokenKind::Operator("~[".into()),
                    start,
                    self.offset,
                ))
            }
            b'\\' if bytes.get(self.offset + 1) == Some(&b'%') => {
                self.offset += 2;
                self.operator_termination('%');
                Ok(token(
                    source,
                    TokenKind::Operator("\\%".into()),
                    start,
                    self.offset,
                ))
            }
            b'#' if bytes.get(self.offset + 1) == Some(&b'#') => {
                self.offset += 2;
                self.operator_termination('#');
                Ok(token(
                    source,
                    TokenKind::Operator("##".into()),
                    start,
                    self.offset,
                ))
            }
            b'+' | b'-' | b'*' | b'/' | b'=' | b'!' | b'<' | b'>' | b'&' | b'|' | b'@' | b'%'
            | b'^' | b'#' | b'\\' => {
                self.offset += 1;
                let operator = source.as_str()[start..self.offset].to_owned();
                if operator != "!" {
                    self.operator_termination(operator.chars().next().unwrap_or('~'));
                }
                Ok(token(
                    source,
                    TokenKind::Operator(operator),
                    start,
                    self.offset,
                ))
            }
            c if b"()[]{};,?.~:".contains(&c) => {
                self.offset += 1;
                self.apply_punctuation(c as char);
                Ok(token(
                    source,
                    TokenKind::Punctuation(c as char),
                    start,
                    self.offset,
                ))
            }
            _ => {
                let character = source.as_str()[start..]
                    .chars()
                    .next()
                    .expect("offset is before end of source");
                self.offset += character.len_utf8();
                Ok(token(
                    source,
                    TokenKind::Unsupported(character.to_string()),
                    start,
                    self.offset,
                ))
            }
        }
    }

    fn skip_space(&mut self) -> Result<(), Diagnostic> {
        let bytes = self.source.as_str().as_bytes();
        loop {
            if self.offset >= bytes.len() {
                return Ok(());
            }
            match bytes[self.offset] {
                b' ' | b'\t' | b'\r' => self.offset += 1,
                b'\n' if self.nesting.is_empty() && self.prevent_termination.is_none() => {
                    return Ok(())
                }
                b'\n' => self.offset += 1,
                b'{' => {
                    let start = self.offset;
                    self.consume_comment(start)?;
                }
                _ => return Ok(()),
            }
        }
    }

    fn apply_keyword(&mut self, word: &str) {
        match word {
            "let" => self.nesting.push(NestingKind::Let),
            "begin" | "if" | "while" | "for" | "case" => self.nesting.push(NestingKind::Block),
            "in" => {
                if self.nesting.last() == Some(&NestingKind::Let) {
                    self.nesting.pop();
                    self.prevent_termination = Some('I');
                }
            }
            "end" | "fi" | "od" | "esac" => {
                if !self.nesting.is_empty() {
                    self.nesting.pop();
                }
            }
            "and" | "or" | "not" => self.prevent_termination = Some('~'),
            "whattype" => self.prevent_termination = Some('W'),
            "set" => self.prevent_termination = Some('S'),
            "set_type" => self.prevent_termination = Some('T'),
            _ => {}
        }
    }

    fn apply_punctuation(&mut self, c: char) {
        match c {
            '(' | '[' => self.nesting.push(NestingKind::Group),
            ')' | ']' => {
                if !self.nesting.is_empty() {
                    self.nesting.pop();
                }
            }
            ',' | ';' | '.' | ':' => self.prevent_termination = Some(c),
            '~' => self.prevent_termination = Some('~'),
            _ => {}
        }
    }

    fn operator_termination(&mut self, symbol: char) {
        self.prevent_termination = if self.previous_termination == Some('.') {
            None
        } else {
            Some(symbol)
        };
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
            let span = self.source.span(start, self.offset);
            Err(Diagnostic::new(
                ErrorKind::Lexical,
                format!(
                    "Comment that started on line {}, column {} is never closed.",
                    span.start.line, span.start.column
                ),
                Some(span),
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
            let recovered = Token {
                kind: TokenKind::String,
                lexeme: self.source.as_str()[start..self.offset].to_owned(),
                value: Some(self.source.as_str()[start + 1..self.offset].replace("\"\"", "\"")),
                span: self.source.span(start, self.offset),
            };
            self.pending = Some(recovered);
            return Err(Diagnostic::new(
                ErrorKind::Lexical,
                "Closing string denotation.",
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
    let output = tokenize_with_diagnostics(source);
    if output.diagnostics.is_empty() {
        Ok(output.tokens)
    } else {
        Err(output.diagnostics)
    }
}

/// Result of a recoverable lexical pass. Diagnostics are kept in encounter
/// order while all tokens that can be recovered are retained. This is needed
/// for Atlas's REPL behavior, where a broken string reports an error but still
/// leaves the token and following commands available to the parser.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LexOutput {
    pub tokens: Vec<Token>,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn tokenize_with_diagnostics(source: &SourceText) -> LexOutput {
    let mut lexer = Lexer::new(source);
    let mut tokens = Vec::new();
    let mut errors = Vec::new();
    loop {
        match lexer.next_token() {
            Ok(token) if token.kind == TokenKind::Eof => {
                tokens.push(token);
                return LexOutput {
                    tokens,
                    diagnostics: errors,
                };
            }
            Ok(token) => tokens.push(token),
            Err(error) => {
                errors.push(error);
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
    fn oracle_non_keyword_is_scanned_as_an_identifier() {
        let tokens = tokenize(&SourceText::new("any_type\n")).expect("valid identifier");
        assert_eq!(tokens[0].kind, TokenKind::Identifier);
    }

    #[test]
    fn lexer_consumes_one_token_and_stays_at_eof() {
        let source = SourceText::new("x\n");
        let mut lexer = Lexer::new(&source);
        assert_eq!(
            lexer.next_token().expect("identifier").kind,
            TokenKind::Identifier
        );
        assert_eq!(
            lexer.next_token().expect("newline").kind,
            TokenKind::Newline
        );
        assert_eq!(lexer.next_token().expect("eof").kind, TokenKind::Eof);
        assert_eq!(lexer.next_token().expect("stable eof").kind, TokenKind::Eof);
    }

    #[test]
    fn cursor_peek_does_not_consume() {
        let source = SourceText::new("x -> y");
        let mut cursor = TokenCursor::new(&source);
        assert_eq!(cursor.peek().expect("peek").kind, TokenKind::Identifier);
        assert_eq!(cursor.bump().expect("bump").kind, TokenKind::Identifier);
        assert_eq!(
            cursor.bump().expect("arrow").kind,
            TokenKind::Operator("->".into())
        );
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
        let tokens = tokenize(&SourceText::new("( ) [ ] ; , ? . ~")).expect("valid punctuation");
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
    fn command_continues_after_an_operator_newline() {
        let tokens = tokenize(&SourceText::new("1 +\n2\n")).expect("continued expression");
        let kinds: Vec<_> = tokens.into_iter().map(|token| token.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Integer,
                TokenKind::Operator("+".into()),
                TokenKind::Integer,
                TokenKind::Newline,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn grouping_and_let_nesting_suppress_internal_newlines() {
        let grouped = tokenize(&SourceText::new("(1 +\n2)\n")).expect("grouped expression");
        assert_eq!(
            grouped
                .iter()
                .filter(|token| token.kind == TokenKind::Newline)
                .count(),
            1
        );

        let binding = tokenize(&SourceText::new("let x =\n1 in\nx\n")).expect("let expression");
        assert_eq!(
            binding
                .iter()
                .filter(|token| token.kind == TokenKind::Newline)
                .count(),
            1
        );
    }

    #[test]
    fn empty_lines_and_top_level_newlines_are_command_boundaries() {
        let tokens = tokenize(&SourceText::new("\n1\n\n2\n")).expect("command stream");
        let kinds: Vec<_> = tokens.into_iter().map(|token| token.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Newline,
                TokenKind::Integer,
                TokenKind::Newline,
                TokenKind::Newline,
                TokenKind::Integer,
                TokenKind::Newline,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn operator_after_dot_may_end_a_command() {
        let tokens = tokenize(&SourceText::new("x.+\n")).expect("operator member access");
        assert_eq!(tokens[tokens.len() - 2].kind, TokenKind::Newline);
    }

    #[test]
    fn atlas_arithmetic_operator_spellings_are_tokens_and_continue_lines() {
        let tokens = tokenize(&SourceText::new("1 %\n2 ^\n3 ##\n4 \\\n5\n"))
            .expect("Atlas arithmetic operators");
        let operators: Vec<_> = tokens
            .iter()
            .filter_map(|token| match &token.kind {
                TokenKind::Operator(operator) => Some(operator.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(operators, vec!["%", "^", "##", "\\"]);
        assert_eq!(
            tokens
                .iter()
                .filter(|token| token.kind == TokenKind::Newline)
                .count(),
            1
        );
    }

    #[test]
    fn comparison_and_colon_spellings_are_preserved() {
        let tokens =
            tokenize(&SourceText::new("a != b <= c >= d : e\n")).expect("comparison operators");
        let operators: Vec<_> = tokens
            .iter()
            .filter_map(|token| match &token.kind {
                TokenKind::Operator(operator) => Some(operator.as_str()),
                TokenKind::Punctuation(':') => Some(":"),
                _ => None,
            })
            .collect();
        assert_eq!(operators, vec!["!=", "<=", ">=", ":"]);
    }

    #[test]
    fn unterminated_string_reports_then_yields_recovered_token() {
        let source = SourceText::new("\"oops\ntail\n");
        let mut lexer = Lexer::new(&source);

        let error = lexer.next_token().expect_err("must diagnose");
        assert_eq!(error.kind, ErrorKind::Lexical);
        assert_eq!(error.message, "Closing string denotation.");

        let string = lexer.next_token().expect("recovered string");
        assert_eq!(string.kind, TokenKind::String);
        assert_eq!(string.lexeme, "\"oops");
        assert_eq!(string.value.as_deref(), Some("oops"));
        assert_eq!(
            lexer.next_token().expect("newline").kind,
            TokenKind::Newline
        );
        assert_eq!(
            lexer.next_token().expect("tail").kind,
            TokenKind::Identifier
        );
    }

    #[test]
    fn diagnostic_collecting_api_keeps_recovered_tokens() {
        let output = tokenize_with_diagnostics(&SourceText::new("\"oops\ntail\n"));
        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(output.tokens[0].kind, TokenKind::String);
        assert_eq!(output.tokens[1].kind, TokenKind::Newline);
        assert_eq!(output.tokens[2].kind, TokenKind::Identifier);
        assert_eq!(output.tokens[3].kind, TokenKind::Newline);
        assert_eq!(output.tokens[4].kind, TokenKind::Eof);
    }

    #[test]
    fn unterminated_comment_reports_then_reaches_stable_eof() {
        let source = SourceText::new("{oops");
        let mut lexer = Lexer::new(&source);
        let error = lexer.next_token().expect_err("must diagnose");
        assert_eq!(error.kind, ErrorKind::Lexical);
        assert_eq!(lexer.next_token().expect("eof").kind, TokenKind::Eof);
        assert_eq!(lexer.next_token().expect("stable eof").kind, TokenKind::Eof);
    }

    #[test]
    fn unrecognized_characters_are_preserved_for_the_parser() {
        let source = SourceText::new("a$b");
        let mut lexer = Lexer::new(&source);
        assert_eq!(lexer.next_token().expect("a").kind, TokenKind::Identifier);
        assert_eq!(
            lexer.next_token().expect("raw token").kind,
            TokenKind::Unsupported("$".into())
        );
        assert_eq!(lexer.next_token().expect("b").kind, TokenKind::Identifier);
    }

    #[test]
    fn unicode_unsupported_token_keeps_scalar_byte_span() {
        let source = SourceText::new("é");
        let tokens = tokenize(&source).expect("unsupported scalar remains a parser token");
        assert_eq!(tokens[0].kind, TokenKind::Unsupported("é".into()));
        assert_eq!(tokens[0].span.byte_range(), 0..2);
        assert_eq!(tokens[0].span.start.column, 1);
        assert_eq!(tokens[0].span.end.column, 2);
    }
}
