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
    assert_eq!(err.message(), "Unexpected end of input");
    assert_eq!(err.span().line, 5);
    assert_eq!(err.span().column, 10);
}

#[test]
fn test_lex_error_unexpected_character_constructor() {
    let err = LexError::unexpected_character('@', span_at(3, 7));
    assert_eq!(err.kind(), LexErrorKind::UnexpectedCharacter);
    assert_eq!(err.message(), "Unexpected character: '@'");
}

#[test]
fn test_lex_error_invalid_identifier_character_constructor() {
    let err = LexError::invalid_identifier_character('日', span_at(1, 5));
    assert_eq!(err.kind(), LexErrorKind::InvalidIdentifierCharacter);
    assert_eq!(
        err.message(),
        "Invalid character '日' in identifier. Only ASCII letters (a-z, A-Z), digits (0-9), and underscores (_) are allowed"
    );
}

#[test]
fn test_lex_error_invalid_whitespace_constructor() {
    let err = LexError::invalid_whitespace('\u{00A0}', dummy_span());
    assert_eq!(err.kind(), LexErrorKind::InvalidWhitespace);
    assert_eq!(
        err.message(),
        "Invalid whitespace character '\u{00A0}' (U+00A0). Only space, tab, carriage return, and newline are allowed"
    );
}

#[test]
fn test_lex_error_incomplete_arrow_constructor() {
    let err = LexError::incomplete_arrow(span_at(2, 15));
    assert_eq!(err.kind(), LexErrorKind::IncompleteArrow);
    assert_eq!(err.message(), "Expected '>' after '-'");
}

#[test]
fn test_lex_error_unknown_escape_sequence_constructor() {
    let err = LexError::unknown_escape_sequence('q', dummy_span());
    assert_eq!(err.kind(), LexErrorKind::UnknownEscapeSequence);
    assert_eq!(err.message(), "Unknown escape sequence: '\\q'");
}

#[test]
fn test_lex_error_unterminated_string_constructor() {
    let err = LexError::unterminated_string(dummy_span());
    assert_eq!(err.kind(), LexErrorKind::UnterminatedString);
    assert_eq!(err.message(), "Unterminated string literal");
}

#[test]
fn test_lex_error_unterminated_string_newline_constructor() {
    let err = LexError::unterminated_string_newline(dummy_span());
    assert_eq!(err.kind(), LexErrorKind::UnterminatedString);
    assert_eq!(
        err.message(),
        "Unterminated string literal (newline in string)"
    );
}

#[test]
fn test_lex_error_integer_overflow_constructor() {
    let err = LexError::integer_overflow("99999999999999999999", dummy_span());
    assert_eq!(err.kind(), LexErrorKind::IntegerOverflow);
    assert_eq!(
        err.message(),
        "Integer literal '99999999999999999999' is out of range for i64 (exceeds maximum value)"
    );
}

#[test]
fn test_lex_error_display() {
    let err = LexError::unexpected_character('#', span_at(10, 20));
    let display = format!("{}", err);
    assert_eq!(display, "10:20: Unexpected character: '#'");
}
