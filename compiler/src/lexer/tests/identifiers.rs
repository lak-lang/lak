//! Tests for identifier recognition including Unicode support.

use super::*;

#[test]
fn test_identifier_simple() {
    let kinds = tokenize_kinds("println");
    assert_eq!(
        kinds,
        vec![TokenKind::Identifier("println".to_string()), TokenKind::Eof]
    );
}

#[test]
fn test_identifier_with_underscore() {
    let kinds = tokenize_kinds("my_func");
    assert_eq!(
        kinds,
        vec![TokenKind::Identifier("my_func".to_string()), TokenKind::Eof]
    );
}

#[test]
fn test_identifier_starts_with_underscore() {
    let kinds = tokenize_kinds("_private");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Identifier("_private".to_string()),
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_identifier_with_numbers() {
    let kinds = tokenize_kinds("func123");
    assert_eq!(
        kinds,
        vec![TokenKind::Identifier("func123".to_string()), TokenKind::Eof]
    );
}

#[test]
fn test_identifier_underscore_only() {
    let kinds = tokenize_kinds("_");
    assert_eq!(
        kinds,
        vec![TokenKind::Identifier("_".to_string()), TokenKind::Eof]
    );
}

#[test]
fn test_multiple_identifiers() {
    let kinds = tokenize_kinds("foo bar");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Identifier("foo".to_string()),
            TokenKind::Identifier("bar".to_string()),
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_identifier_unicode() {
    let kinds = tokenize_kinds("日本語");
    assert_eq!(
        kinds,
        vec![TokenKind::Identifier("日本語".to_string()), TokenKind::Eof]
    );
}
