//! Tests for edge cases and platform compatibility.

use super::*;

#[test]
fn test_error_escape_at_eof() {
    let err = tokenize_error(r#""hello\"#);
    assert!(err.message.contains("Unterminated string"));
}

#[test]
fn test_windows_line_endings() {
    // Windows line endings: \r is skipped as whitespace, then \n emits Newline
    let kinds = tokenize_kinds("a\r\nb");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Identifier("a".to_string()),
            TokenKind::Newline,
            TokenKind::Identifier("b".to_string()),
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_multiple_consecutive_backslashes() {
    let kinds = tokenize_kinds(r#""\\\\""#);
    assert_eq!(
        kinds,
        vec![TokenKind::StringLiteral("\\\\".to_string()), TokenKind::Eof]
    );
}

#[test]
fn test_multiple_consecutive_quotes() {
    let kinds = tokenize_kinds(r#""\"\"""#);
    assert_eq!(
        kinds,
        vec![TokenKind::StringLiteral("\"\"".to_string()), TokenKind::Eof]
    );
}

#[test]
fn test_integer_literal_leading_zeros() {
    // Leading zeros are treated as decimal (not octal)
    let kinds = tokenize_kinds("007");
    assert_eq!(kinds, vec![TokenKind::IntLiteral(7), TokenKind::Eof]);
}

#[test]
fn test_integer_literal_all_zeros() {
    let kinds = tokenize_kinds("000");
    assert_eq!(kinds, vec![TokenKind::IntLiteral(0), TokenKind::Eof]);
}
