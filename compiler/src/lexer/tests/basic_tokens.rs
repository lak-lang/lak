//! Tests for basic token recognition (punctuation, braces, etc.)

use super::*;

#[test]
fn test_empty_input() {
    let kinds = tokenize_kinds("");
    assert_eq!(kinds, vec![TokenKind::Eof]);
}

#[test]
fn test_whitespace_only() {
    let kinds = tokenize_kinds("   \n\t");
    assert_eq!(kinds, vec![TokenKind::Eof]);
}

#[test]
fn test_left_paren() {
    let kinds = tokenize_kinds("(");
    assert_eq!(kinds, vec![TokenKind::LeftParen, TokenKind::Eof]);
}

#[test]
fn test_right_paren() {
    let kinds = tokenize_kinds(")");
    assert_eq!(kinds, vec![TokenKind::RightParen, TokenKind::Eof]);
}

#[test]
fn test_comma() {
    let kinds = tokenize_kinds(",");
    assert_eq!(kinds, vec![TokenKind::Comma, TokenKind::Eof]);
}

#[test]
fn test_multiple_punctuation() {
    let kinds = tokenize_kinds("(,)");
    assert_eq!(
        kinds,
        vec![
            TokenKind::LeftParen,
            TokenKind::Comma,
            TokenKind::RightParen,
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_left_brace() {
    let kinds = tokenize_kinds("{");
    assert_eq!(kinds, vec![TokenKind::LeftBrace, TokenKind::Eof]);
}

#[test]
fn test_right_brace() {
    let kinds = tokenize_kinds("}");
    assert_eq!(kinds, vec![TokenKind::RightBrace, TokenKind::Eof]);
}

#[test]
fn test_arrow() {
    let kinds = tokenize_kinds("->");
    assert_eq!(kinds, vec![TokenKind::Arrow, TokenKind::Eof]);
}

#[test]
fn test_punctuation_with_spaces() {
    let kinds = tokenize_kinds("( , )");
    assert_eq!(
        kinds,
        vec![
            TokenKind::LeftParen,
            TokenKind::Comma,
            TokenKind::RightParen,
            TokenKind::Eof
        ]
    );
}
