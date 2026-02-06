//! Unit tests for the lexer module.

use super::*;
use crate::token::TokenKind;

/// Helper function to tokenize input and return only the kinds.
pub(super) fn tokenize_kinds(input: &str) -> Vec<TokenKind> {
    let mut lexer = Lexer::new(input);
    lexer
        .tokenize()
        .unwrap_or_else(|e| panic!("Tokenization failed for input {:?}: {}", input, e))
        .into_iter()
        .map(|t| t.kind)
        .collect()
}

/// Helper function to tokenize input and return the error.
pub(super) fn tokenize_error(input: &str) -> LexError {
    let mut lexer = Lexer::new(input);
    match lexer.tokenize() {
        Ok(tokens) => panic!(
            "Expected tokenization to fail for input {:?}, but it succeeded with {} tokens",
            input,
            tokens.len()
        ),
        Err(e) => e,
    }
}

mod basic_tokens;
mod comments;
mod compound;
mod edge_cases;
mod errors;
mod identifiers;
mod integers;
mod keywords;
mod newlines;
mod spans;
mod strings;
mod whitespace;

// ============================================================================
// LexError constructor tests
// ============================================================================

use crate::token::Span;

fn dummy_span() -> Span {
    Span::new(0, 0, 1, 1)
}

fn span_at(line: usize, column: usize) -> Span {
    Span::new(0, 0, line, column)
}

#[test]
fn test_lex_error_unexpected_eof_constructor() {
    let err = LexError::unexpected_eof(span_at(5, 10));
    assert_eq!(err.kind(), LexErrorKind::UnexpectedEof);
    assert!(err.message().contains("Unexpected end of input"));
    assert_eq!(err.span().line, 5);
    assert_eq!(err.span().column, 10);
}

#[test]
fn test_lex_error_unexpected_character_constructor() {
    let err = LexError::unexpected_character('@', span_at(3, 7));
    assert_eq!(err.kind(), LexErrorKind::UnexpectedCharacter);
    assert!(err.message().contains("@"));
    assert!(err.message().contains("Unexpected character"));
}

#[test]
fn test_lex_error_invalid_identifier_character_constructor() {
    let err = LexError::invalid_identifier_character('日', span_at(1, 5));
    assert_eq!(err.kind(), LexErrorKind::InvalidIdentifierCharacter);
    assert!(err.message().contains("日"));
    assert!(err.message().contains("ASCII"));
}

#[test]
fn test_lex_error_invalid_whitespace_constructor() {
    let err = LexError::invalid_whitespace('\u{00A0}', dummy_span());
    assert_eq!(err.kind(), LexErrorKind::InvalidWhitespace);
    assert!(err.message().contains("00A0"));
}

#[test]
fn test_lex_error_incomplete_arrow_constructor() {
    let err = LexError::incomplete_arrow(span_at(2, 15));
    assert_eq!(err.kind(), LexErrorKind::IncompleteArrow);
    assert!(err.message().contains(">"));
    assert!(err.message().contains("-"));
}

#[test]
fn test_lex_error_unknown_escape_sequence_constructor() {
    let err = LexError::unknown_escape_sequence('q', dummy_span());
    assert_eq!(err.kind(), LexErrorKind::UnknownEscapeSequence);
    assert!(err.message().contains("\\q"));
}

#[test]
fn test_lex_error_unterminated_string_constructor() {
    let err = LexError::unterminated_string(dummy_span());
    assert_eq!(err.kind(), LexErrorKind::UnterminatedString);
    assert!(err.message().contains("Unterminated string"));
}

#[test]
fn test_lex_error_unterminated_string_newline_constructor() {
    let err = LexError::unterminated_string_newline(dummy_span());
    assert_eq!(err.kind(), LexErrorKind::UnterminatedString);
    assert!(err.message().contains("newline"));
}

#[test]
fn test_lex_error_integer_overflow_constructor() {
    let err = LexError::integer_overflow("99999999999999999999", dummy_span());
    assert_eq!(err.kind(), LexErrorKind::IntegerOverflow);
    assert!(err.message().contains("99999999999999999999"));
    assert!(err.message().contains("i64"));
}

#[test]
fn test_lex_error_display() {
    let err = LexError::unexpected_character('#', span_at(10, 20));
    let display = format!("{}", err);
    assert!(display.contains("10:20"));
    assert!(display.contains("#"));
}
