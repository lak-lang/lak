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

// Arithmetic operator tests

#[test]
fn test_plus() {
    let kinds = tokenize_kinds("+");
    assert_eq!(kinds, vec![TokenKind::Plus, TokenKind::Eof]);
}

#[test]
fn test_minus() {
    let kinds = tokenize_kinds("- ");
    assert_eq!(kinds, vec![TokenKind::Minus, TokenKind::Eof]);
}

#[test]
fn test_minus_followed_by_identifier() {
    let kinds = tokenize_kinds("-x");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Minus,
            TokenKind::Identifier("x".to_string()),
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_arrow_still_works() {
    let kinds = tokenize_kinds("-> ->");
    assert_eq!(
        kinds,
        vec![TokenKind::Arrow, TokenKind::Arrow, TokenKind::Eof]
    );
}

#[test]
fn test_star() {
    let kinds = tokenize_kinds("*");
    assert_eq!(kinds, vec![TokenKind::Star, TokenKind::Eof]);
}

#[test]
fn test_slash() {
    let kinds = tokenize_kinds("/ 2");
    assert_eq!(
        kinds,
        vec![TokenKind::Slash, TokenKind::IntLiteral(2), TokenKind::Eof]
    );
}

#[test]
fn test_slash_not_comment() {
    let kinds = tokenize_kinds("1 / 2");
    assert_eq!(
        kinds,
        vec![
            TokenKind::IntLiteral(1),
            TokenKind::Slash,
            TokenKind::IntLiteral(2),
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_comment_still_works() {
    let kinds = tokenize_kinds("1 // comment");
    assert_eq!(kinds, vec![TokenKind::IntLiteral(1), TokenKind::Eof]);
}

#[test]
fn test_comment_with_newline() {
    let kinds = tokenize_kinds("1 // comment\n2");
    assert_eq!(
        kinds,
        vec![
            TokenKind::IntLiteral(1),
            TokenKind::Newline,
            TokenKind::IntLiteral(2),
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_percent() {
    let kinds = tokenize_kinds("%");
    assert_eq!(kinds, vec![TokenKind::Percent, TokenKind::Eof]);
}

#[test]
fn test_arithmetic_expression() {
    let kinds = tokenize_kinds("1 + 2 * 3");
    assert_eq!(
        kinds,
        vec![
            TokenKind::IntLiteral(1),
            TokenKind::Plus,
            TokenKind::IntLiteral(2),
            TokenKind::Star,
            TokenKind::IntLiteral(3),
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_subtraction_expression() {
    let kinds = tokenize_kinds("10 - 5 - 2");
    assert_eq!(
        kinds,
        vec![
            TokenKind::IntLiteral(10),
            TokenKind::Minus,
            TokenKind::IntLiteral(5),
            TokenKind::Minus,
            TokenKind::IntLiteral(2),
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_all_arithmetic_operators() {
    let kinds = tokenize_kinds("+ - * / %");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Plus,
            TokenKind::Minus,
            TokenKind::Star,
            TokenKind::Slash,
            TokenKind::Percent,
            TokenKind::Eof
        ]
    );
}
