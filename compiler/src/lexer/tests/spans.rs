//! Tests for span position verification.

use super::*;

#[test]
fn test_span_positions() {
    let mut lexer = Lexer::new("foo");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].span.start, 0);
    assert_eq!(tokens[0].span.end, 3);
}

#[test]
fn test_span_line_column() {
    let mut lexer = Lexer::new("foo");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].span.line, 1);
    assert_eq!(tokens[0].span.column, 1);
}

#[test]
fn test_span_multiline() {
    let mut lexer = Lexer::new("a\nb");
    let tokens = lexer.tokenize().unwrap();

    // First token 'a' on line 1
    assert_eq!(tokens[0].span.line, 1);
    assert_eq!(tokens[0].span.column, 1);

    // Second token is Newline on line 1
    assert!(matches!(tokens[1].kind, TokenKind::Newline));
    assert_eq!(tokens[1].span.line, 1);

    // Third token 'b' on line 2
    assert_eq!(tokens[2].span.line, 2);
    assert_eq!(tokens[2].span.column, 1);
}

#[test]
fn test_span_string_literal() {
    let mut lexer = Lexer::new(r#""hello""#);
    let tokens = lexer.tokenize().unwrap();

    // String includes quotes in span
    assert_eq!(tokens[0].span.start, 0);
    assert_eq!(tokens[0].span.end, 7); // includes both quotes
}

#[test]
fn test_span_after_whitespace() {
    let mut lexer = Lexer::new("   foo");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].span.start, 3);
    assert_eq!(tokens[0].span.end, 6);
    assert_eq!(tokens[0].span.column, 4);
}

#[test]
fn test_arrow_span() {
    let mut lexer = Lexer::new("->");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].span.start, 0);
    assert_eq!(tokens[0].span.end, 2); // `->` is 2 characters
    assert_eq!(tokens[0].span.line, 1);
    assert_eq!(tokens[0].span.column, 1);
}

#[test]
fn test_brace_span() {
    let mut lexer = Lexer::new("{ }");
    let tokens = lexer.tokenize().unwrap();

    // Left brace
    assert_eq!(tokens[0].span.start, 0);
    assert_eq!(tokens[0].span.end, 1);

    // Right brace
    assert_eq!(tokens[1].span.start, 2);
    assert_eq!(tokens[1].span.end, 3);
}
