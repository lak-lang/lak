//! Tests for keyword recognition and disambiguation from identifiers.

use super::*;

#[test]
fn test_keyword_fn() {
    let kinds = tokenize_kinds("fn");
    assert_eq!(kinds, vec![TokenKind::Fn, TokenKind::Eof]);
}

#[test]
fn test_fn_not_prefix() {
    // "fn_test" should be an identifier, not fn + identifier
    let kinds = tokenize_kinds("fn_test");
    assert_eq!(
        kinds,
        vec![TokenKind::Identifier("fn_test".to_string()), TokenKind::Eof]
    );
}

#[test]
fn test_keyword_let() {
    let kinds = tokenize_kinds("let");
    assert_eq!(kinds, vec![TokenKind::Let, TokenKind::Eof]);
}

#[test]
fn test_let_not_prefix() {
    // "letter" should be an identifier, not let + identifier
    let kinds = tokenize_kinds("letter");
    assert_eq!(
        kinds,
        vec![TokenKind::Identifier("letter".to_string()), TokenKind::Eof]
    );
}

#[test]
fn test_colon() {
    let kinds = tokenize_kinds(":");
    assert_eq!(kinds, vec![TokenKind::Colon, TokenKind::Eof]);
}

#[test]
fn test_equals() {
    let kinds = tokenize_kinds("=");
    assert_eq!(kinds, vec![TokenKind::Equals, TokenKind::Eof]);
}

#[test]
fn test_function_definition_tokens() {
    let kinds = tokenize_kinds("fn main() -> void {}");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Fn,
            TokenKind::Identifier("main".to_string()),
            TokenKind::LeftParen,
            TokenKind::RightParen,
            TokenKind::Arrow,
            TokenKind::Identifier("void".to_string()),
            TokenKind::LeftBrace,
            TokenKind::RightBrace,
            TokenKind::Eof
        ]
    );
}
