//! Tests for compound expressions (function calls, nested calls, etc.)

use super::*;

#[test]
fn test_println_call() {
    let kinds = tokenize_kinds(r#"println("hello")"#);
    assert_eq!(
        kinds,
        vec![
            TokenKind::Identifier("println".to_string()),
            TokenKind::LeftParen,
            TokenKind::StringLiteral("hello".to_string()),
            TokenKind::RightParen,
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_nested_call() {
    let kinds = tokenize_kinds(r#"outer(inner("x"))"#);
    assert_eq!(
        kinds,
        vec![
            TokenKind::Identifier("outer".to_string()),
            TokenKind::LeftParen,
            TokenKind::Identifier("inner".to_string()),
            TokenKind::LeftParen,
            TokenKind::StringLiteral("x".to_string()),
            TokenKind::RightParen,
            TokenKind::RightParen,
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_multiple_args() {
    let kinds = tokenize_kinds(r#"func("a", "b", "c")"#);
    assert_eq!(
        kinds,
        vec![
            TokenKind::Identifier("func".to_string()),
            TokenKind::LeftParen,
            TokenKind::StringLiteral("a".to_string()),
            TokenKind::Comma,
            TokenKind::StringLiteral("b".to_string()),
            TokenKind::Comma,
            TokenKind::StringLiteral("c".to_string()),
            TokenKind::RightParen,
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_multiple_statements() {
    // Newline after `)` emits Newline token
    let kinds = tokenize_kinds("foo()\nbar()");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Identifier("foo".to_string()),
            TokenKind::LeftParen,
            TokenKind::RightParen,
            TokenKind::Newline,
            TokenKind::Identifier("bar".to_string()),
            TokenKind::LeftParen,
            TokenKind::RightParen,
            TokenKind::Eof
        ]
    );
}
