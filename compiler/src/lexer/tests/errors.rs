//! Tests for error cases and error handling.

use super::*;
use crate::lexer::LexErrorKind;
use crate::token::Span;

#[test]
fn test_error_unterminated_string() {
    let err = tokenize_error(r#""hello"#);
    assert_eq!(err.kind(), LexErrorKind::UnterminatedString);
    assert_eq!(err.message(), "Unterminated string literal");
}

#[test]
fn test_error_newline_in_string() {
    let err = tokenize_error("\"hello\nworld\"");
    assert_eq!(err.kind(), LexErrorKind::UnterminatedString);
    assert_eq!(
        err.message(),
        "Unterminated string literal (newline in string)"
    );
}

#[test]
fn test_error_unknown_escape() {
    let err = tokenize_error(r#""\x""#);
    assert_eq!(err.kind(), LexErrorKind::UnknownEscapeSequence);
    assert_eq!(err.message(), "Unknown escape sequence: '\\x'");
}

#[test]
fn test_error_unexpected_char_at() {
    let err = tokenize_error("@");
    assert_eq!(err.kind(), LexErrorKind::UnexpectedCharacter);
    assert_eq!(err.message(), "Unexpected character: '@'");
}

#[test]
fn test_error_unexpected_char_hash() {
    let err = tokenize_error("#");
    assert_eq!(err.kind(), LexErrorKind::UnexpectedCharacter);
    assert_eq!(err.message(), "Unexpected character: '#'");
}

#[test]
fn test_error_unexpected_char_dollar() {
    let err = tokenize_error("$");
    assert_eq!(err.kind(), LexErrorKind::UnexpectedCharacter);
    assert_eq!(err.message(), "Unexpected character: '$'");
}

#[test]
fn test_error_single_ampersand() {
    let err = tokenize_error("&");
    assert_eq!(err.kind(), LexErrorKind::UnexpectedCharacter);
    assert_eq!(err.message(), "Unexpected character: '&'");
}

#[test]
fn test_error_single_pipe() {
    let err = tokenize_error("|");
    assert_eq!(err.kind(), LexErrorKind::UnexpectedCharacter);
    assert_eq!(err.message(), "Unexpected character: '|'");
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
    assert_eq!(display, "2:3: Test error");
}
