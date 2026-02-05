//! Tests for string literals and escape sequences.

use super::*;

#[test]
fn test_string_empty() {
    let kinds = tokenize_kinds(r#""""#);
    assert_eq!(
        kinds,
        vec![TokenKind::StringLiteral("".to_string()), TokenKind::Eof]
    );
}

#[test]
fn test_string_simple() {
    let kinds = tokenize_kinds(r#""hello""#);
    assert_eq!(
        kinds,
        vec![
            TokenKind::StringLiteral("hello".to_string()),
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_string_with_spaces() {
    let kinds = tokenize_kinds(r#""hello world""#);
    assert_eq!(
        kinds,
        vec![
            TokenKind::StringLiteral("hello world".to_string()),
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_string_escape_newline() {
    let kinds = tokenize_kinds(r#""a\nb""#);
    assert_eq!(
        kinds,
        vec![TokenKind::StringLiteral("a\nb".to_string()), TokenKind::Eof]
    );
}

#[test]
fn test_string_escape_tab() {
    let kinds = tokenize_kinds(r#""a\tb""#);
    assert_eq!(
        kinds,
        vec![TokenKind::StringLiteral("a\tb".to_string()), TokenKind::Eof]
    );
}

#[test]
fn test_string_escape_carriage_return() {
    let kinds = tokenize_kinds(r#""a\rb""#);
    assert_eq!(
        kinds,
        vec![TokenKind::StringLiteral("a\rb".to_string()), TokenKind::Eof]
    );
}

#[test]
fn test_string_escape_backslash() {
    let kinds = tokenize_kinds(r#""a\\b""#);
    assert_eq!(
        kinds,
        vec![TokenKind::StringLiteral("a\\b".to_string()), TokenKind::Eof]
    );
}

#[test]
fn test_string_escape_quote() {
    let kinds = tokenize_kinds(r#""a\"b""#);
    assert_eq!(
        kinds,
        vec![TokenKind::StringLiteral("a\"b".to_string()), TokenKind::Eof]
    );
}

#[test]
fn test_string_multiple_escapes() {
    let kinds = tokenize_kinds(r#""\n\t\r""#);
    assert_eq!(
        kinds,
        vec![
            TokenKind::StringLiteral("\n\t\r".to_string()),
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_string_all_escapes_combined() {
    let kinds = tokenize_kinds(r#""\n\t\r\\\"end""#);
    assert_eq!(
        kinds,
        vec![
            TokenKind::StringLiteral("\n\t\r\\\"end".to_string()),
            TokenKind::Eof
        ]
    );
}
