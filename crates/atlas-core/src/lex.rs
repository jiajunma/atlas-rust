//! Atlas lexical tokens. This module intentionally exposes no parser policy.

use crate::{diagnostic::{Diagnostic, ErrorKind}, source::SourceText};

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
    pub lexeme: String,
    pub value: Option<String>,
    pub span: crate::diagnostic::SourceSpan,
}

const KEYWORDS: &[&str] = &[
    "quit", "set", "let", "in", "begin", "end", "if", "then", "else", "elif", "fi",
    "and", "or", "not", "next", "do", "dont", "from", "downto", "while", "for", "od",
    "case", "esac", "rec_fun", "true", "false", "die", "break", "return", "set_type",
    "any_type", "whattype", "showall", "forget",
];

pub fn tokenize(source: &SourceText) -> Result<Vec<Token>, Vec<Diagnostic>> {
    let text = source.as_str().as_bytes();
    let mut tokens = Vec::new();
    let mut errors = Vec::new();
    let mut i = 0;
    while i < text.len() {
        let start = i;
        match text[i] {
            b' ' | b'\t' | b'\r' => { i += 1; }
            b'\n' => { i += 1; tokens.push(token(source, TokenKind::Newline, start, i)); }
            b'{' => {
                i += 1;
                let mut depth = 1;
                while i < text.len() && depth > 0 {
                    match text[i] { b'{' => depth += 1, b'}' => depth -= 1, _ => {} }
                    i += 1;
                }
                if depth != 0 {
                    errors.push(Diagnostic::new(ErrorKind::Lexical, "unterminated comment", Some(source.span(start, i))));
                    break;
                }
            }
            b'0'..=b'9' => {
                i += 1; while i < text.len() && text[i].is_ascii_digit() { i += 1; }
                tokens.push(token(source, TokenKind::Integer, start, i));
            }
            b'_' | b'a'..=b'z' | b'A'..=b'Z' => {
                i += 1; while i < text.len() && (text[i].is_ascii_alphanumeric() || text[i] == b'_') { i += 1; }
                let word = &source.as_str()[start..i];
                let kind = if KEYWORDS.contains(&word) { TokenKind::Keyword(word.to_owned()) } else { TokenKind::Identifier };
                tokens.push(token(source, kind, start, i));
            }
            b'"' => {
                i += 1; let mut closed = false;
                while i < text.len() {
                    match text[i] { b'"' if i + 1 < text.len() && text[i + 1] == b'"' => i += 2,
                        b'"' => { i += 1; closed = true; break }, b'\n' => break, _ => i += 1 }
                }
                if closed {
                    let raw = &source.as_str()[start + 1..i - 1];
                    tokens.push(Token { kind: TokenKind::String, lexeme: source.as_str()[start..i].to_owned(), value: Some(raw.replace("\"\"", "\"")), span: source.span(start, i) });
                }
                else { errors.push(Diagnostic::new(ErrorKind::Lexical, "unterminated string", Some(source.span(start, i)))); break; }
            }
            b':' if text.get(i + 1) == Some(&b'=') => { i += 2; tokens.push(token(source, TokenKind::Operator(":=".into()), start, i)); }
            b'-' if text.get(i + 1) == Some(&b'>') => { i += 2; tokens.push(token(source, TokenKind::Operator("->".into()), start, i)); }
            b'~' if text.get(i + 1) == Some(&b'[') => { i += 2; tokens.push(token(source, TokenKind::Operator("~[".into()), start, i)); }
            b'+' | b'-' | b'*' | b'/' | b'=' | b'!' | b'<' | b'>' | b'&' | b'|' | b'@' => {
                i += 1; tokens.push(token(source, TokenKind::Operator(source.as_str()[start..i].into()), start, i));
            }
            c if b"()[]{};,?.~".contains(&c) => { i += 1; tokens.push(token(source, TokenKind::Punctuation(c as char), start, i)); }
            _ => { i += 1; errors.push(Diagnostic::new(ErrorKind::Lexical, "invalid character", Some(source.span(start, i)))); }
        }
    }
    if errors.is_empty() { tokens.push(Token { kind: TokenKind::Eof, lexeme: String::new(), value: None, span: source.span(i, i) }); Ok(tokens) } else { Err(errors) }
}

fn token(source: &SourceText, kind: TokenKind, start: usize, end: usize) -> Token {
    Token { kind, lexeme: source.as_str()[start..end].to_owned(), value: None, span: source.span(start, end) }
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
        assert_eq!(tokens[3].span.start.column, 11);
        assert_eq!(tokens[4].kind, TokenKind::Newline);
        assert_eq!(tokens[5].kind, TokenKind::Eof);
    }

    #[test]
    fn doubled_quotes_are_one_string_token() {
        let tokens = tokenize(&SourceText::new("\"a\"\"b\"")).expect("valid string");
        assert_eq!(tokens[0].kind, TokenKind::String);
        assert_eq!(tokens[0].lexeme, "\"a\"\"b\"");
        assert_eq!(tokens[0].value.as_deref(), Some("a\"b"));
    }

    #[test]
    fn unterminated_string_is_lexical_error() {
        let errors = tokenize(&SourceText::new("\"oops\n")).expect_err("must reject");
        assert_eq!(errors[0].kind, ErrorKind::Lexical);
        assert_eq!(errors[0].span.expect("span").start.column, 1);
    }
}
