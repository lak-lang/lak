//! Tests for integer literal parsing.

use super::*;

#[test]
fn test_integer_literal_simple() {
    let kinds = tokenize_kinds("123");
    assert_eq!(kinds, vec![TokenKind::IntLiteral(123), TokenKind::Eof]);
}

#[test]
fn test_integer_literal_zero() {
    let kinds = tokenize_kinds("0");
    assert_eq!(kinds, vec![TokenKind::IntLiteral(0), TokenKind::Eof]);
}

#[test]
fn test_integer_literal_large() {
    let kinds = tokenize_kinds("9223372036854775807");
    assert_eq!(
        kinds,
        vec![TokenKind::IntLiteral(9223372036854775807), TokenKind::Eof]
    );
}

#[test]
fn test_let_statement_tokens() {
    let kinds = tokenize_kinds("let x: i32 = 42");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Let,
            TokenKind::Identifier("x".to_string()),
            TokenKind::Colon,
            TokenKind::Identifier("i32".to_string()),
            TokenKind::Equals,
            TokenKind::IntLiteral(42),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn test_integer_literal_i64_min_abs() {
    let kinds = tokenize_kinds("9223372036854775808");
    assert_eq!(
        kinds,
        vec![TokenKind::IntLiteral(9223372036854775808), TokenKind::Eof]
    );
}

#[test]
fn test_integer_literal_u64_max() {
    let kinds = tokenize_kinds("18446744073709551615");
    assert_eq!(
        kinds,
        vec![TokenKind::IntLiteral(18446744073709551615), TokenKind::Eof]
    );
}
