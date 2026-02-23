//! Tests for newline token emission rules.

use super::*;

#[test]
fn test_newline_after_identifier() {
    // Newline after identifier should emit Newline token
    let kinds = tokenize_kinds("foo\nbar");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Identifier("foo".to_string()),
            TokenKind::Newline,
            TokenKind::Identifier("bar".to_string()),
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_newline_after_integer_literal() {
    let kinds = tokenize_kinds("42\n123");
    assert_eq!(
        kinds,
        vec![
            TokenKind::IntLiteral(42),
            TokenKind::Newline,
            TokenKind::IntLiteral(123),
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_newline_after_float_literal() {
    let kinds = tokenize_kinds("2.5\n0.5");
    assert_eq!(
        kinds,
        vec![
            TokenKind::FloatLiteral(2.5),
            TokenKind::Newline,
            TokenKind::FloatLiteral(0.5),
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_newline_after_string_literal() {
    let kinds = tokenize_kinds("\"hello\"\n\"world\"");
    assert_eq!(
        kinds,
        vec![
            TokenKind::StringLiteral("hello".to_string()),
            TokenKind::Newline,
            TokenKind::StringLiteral("world".to_string()),
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_newline_after_right_paren() {
    let kinds = tokenize_kinds("()\nfoo");
    assert_eq!(
        kinds,
        vec![
            TokenKind::LeftParen,
            TokenKind::RightParen,
            TokenKind::Newline,
            TokenKind::Identifier("foo".to_string()),
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_newline_after_right_brace() {
    let kinds = tokenize_kinds("{}\nfoo");
    assert_eq!(
        kinds,
        vec![
            TokenKind::LeftBrace,
            TokenKind::RightBrace,
            TokenKind::Newline,
            TokenKind::Identifier("foo".to_string()),
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_no_newline_after_left_paren() {
    // Newline after `(` should NOT emit Newline token
    let kinds = tokenize_kinds("(\nfoo)");
    assert_eq!(
        kinds,
        vec![
            TokenKind::LeftParen,
            TokenKind::Identifier("foo".to_string()),
            TokenKind::RightParen,
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_no_newline_after_left_brace() {
    // Newline after `{` should NOT emit Newline token
    let kinds = tokenize_kinds("{\nfoo}");
    assert_eq!(
        kinds,
        vec![
            TokenKind::LeftBrace,
            TokenKind::Identifier("foo".to_string()),
            TokenKind::RightBrace,
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_no_newline_after_comma() {
    // Newline after `,` should NOT emit Newline token
    let kinds = tokenize_kinds("a,\nb");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Identifier("a".to_string()),
            TokenKind::Comma,
            TokenKind::Identifier("b".to_string()),
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_no_newline_after_arrow() {
    // Newline after `->` should NOT emit Newline token
    let kinds = tokenize_kinds("->\nvoid");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Arrow,
            TokenKind::Identifier("void".to_string()),
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_no_newline_after_fn() {
    // Newline after `fn` keyword should NOT emit Newline token
    let kinds = tokenize_kinds("fn\nmain");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Fn,
            TokenKind::Identifier("main".to_string()),
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_no_newline_after_let() {
    // Newline after `let` keyword should NOT emit Newline token
    let kinds = tokenize_kinds("let\nx");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Let,
            TokenKind::Identifier("x".to_string()),
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_no_newline_after_while() {
    // Newline after `while` keyword should NOT emit Newline token
    let kinds = tokenize_kinds("while\ntrue");
    assert_eq!(
        kinds,
        vec![
            TokenKind::While,
            TokenKind::BoolLiteral(true),
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_newline_after_return_keyword() {
    let kinds = tokenize_kinds("return\nx");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Return,
            TokenKind::Newline,
            TokenKind::Identifier("x".to_string()),
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_newline_after_break_keyword() {
    let kinds = tokenize_kinds("break\nx");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Break,
            TokenKind::Newline,
            TokenKind::Identifier("x".to_string()),
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_newline_after_continue_keyword() {
    let kinds = tokenize_kinds("continue\nx");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Continue,
            TokenKind::Newline,
            TokenKind::Identifier("x".to_string()),
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_newline_in_let_statement() {
    // Newline after variable value in let statement
    let kinds = tokenize_kinds("let x: i32 = 42\nprintln");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Let,
            TokenKind::Identifier("x".to_string()),
            TokenKind::Colon,
            TokenKind::Identifier("i32".to_string()),
            TokenKind::Equals,
            TokenKind::IntLiteral(42),
            TokenKind::Newline,
            TokenKind::Identifier("println".to_string()),
            TokenKind::Eof
        ]
    );
}
