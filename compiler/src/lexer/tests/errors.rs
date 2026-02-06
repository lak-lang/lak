//! Tests for error cases and error handling.

use super::*;
use crate::lexer::LexErrorKind;
use crate::token::Span;

#[test]
fn test_error_unterminated_string() {
    let err = tokenize_error(r#""hello"#);
    assert!(err.message().contains("Unterminated string"));
    assert_eq!(err.kind(), LexErrorKind::UnterminatedString);
}

#[test]
fn test_error_newline_in_string() {
    let err = tokenize_error("\"hello\nworld\"");
    assert!(err.message().contains("newline in string"));
    assert_eq!(err.kind(), LexErrorKind::UnterminatedString);
}

#[test]
fn test_error_unknown_escape() {
    let err = tokenize_error(r#""\x""#);
    assert!(err.message().contains("Unknown escape sequence"));
    assert_eq!(err.kind(), LexErrorKind::UnknownEscapeSequence);
}

#[test]
fn test_error_unexpected_char_at() {
    let err = tokenize_error("@");
    assert!(err.message().contains("Unexpected character"));
    assert_eq!(err.kind(), LexErrorKind::UnexpectedCharacter);
}

#[test]
fn test_error_unexpected_char_hash() {
    let err = tokenize_error("#");
    assert!(err.message().contains("Unexpected character"));
    assert_eq!(err.kind(), LexErrorKind::UnexpectedCharacter);
}

#[test]
fn test_error_unexpected_char_dollar() {
    let err = tokenize_error("$");
    assert!(err.message().contains("Unexpected character"));
    assert_eq!(err.kind(), LexErrorKind::UnexpectedCharacter);
}

#[test]
fn test_error_minus_without_arrow() {
    let err = tokenize_error("-");
    assert!(err.message().contains("Expected '>' after '-'"));
    assert_eq!(err.kind(), LexErrorKind::IncompleteArrow);
}

#[test]
fn test_error_minus_with_other_char() {
    let err = tokenize_error("-x");
    assert!(err.message().contains("Expected '>' after '-'"));
    assert_eq!(err.kind(), LexErrorKind::IncompleteArrow);
}

#[test]
fn test_error_span_location() {
    let err = tokenize_error("foo @");
    assert_eq!(err.span().start, 4);
    assert_eq!(err.span().column, 5);
}

#[test]
fn test_lex_error_display() {
    let err = LexError::new(
        LexErrorKind::UnexpectedCharacter,
        "Test error",
        Span::new(0, 1, 2, 3),
    );
    let display = format!("{}", err);
    assert!(display.contains("2:3"));
    assert!(display.contains("Test error"));
}
