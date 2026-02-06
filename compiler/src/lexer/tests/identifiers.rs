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
fn test_identifier_unicode_rejected() {
    let err = tokenize_error("日本語");
    assert_eq!(
        err.message(),
        "Invalid character '日' in identifier. Only ASCII letters (a-z, A-Z), digits (0-9), and underscores (_) are allowed"
    );
}

#[test]
fn test_identifier_mixed_ascii_unicode_rejected() {
    let err = tokenize_error("hello世界");
    assert_eq!(
        err.message(),
        "Invalid character '世' in identifier. Only ASCII letters (a-z, A-Z), digits (0-9), and underscores (_) are allowed"
    );
}

#[test]
fn test_identifier_underscore_then_unicode_rejected() {
    let err = tokenize_error("_日本語");
    assert_eq!(
        err.message(),
        "Invalid character '日' in identifier. Only ASCII letters (a-z, A-Z), digits (0-9), and underscores (_) are allowed"
    );
}

#[test]
fn test_identifier_emoji_rejected() {
    let err = tokenize_error("🚀hello");
    assert_eq!(err.message(), "Unexpected character: '🚀'");
}

#[test]
fn test_identifier_multiple_underscores() {
    let kinds = tokenize_kinds("__private__");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Identifier("__private__".to_string()),
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_identifier_unicode_start_then_ascii_rejected() {
    // Unicode at start followed by ASCII should be rejected
    let err = tokenize_error("世界hello");
    assert_eq!(
        err.message(),
        "Invalid character '世' in identifier. Only ASCII letters (a-z, A-Z), digits (0-9), and underscores (_) are allowed"
    );
}

#[test]
fn test_identifier_ascii_unicode_ascii_rejected() {
    // Non-ASCII sandwiched between ASCII should be rejected
    let err = tokenize_error("abc中def");
    assert_eq!(
        err.message(),
        "Invalid character '中' in identifier. Only ASCII letters (a-z, A-Z), digits (0-9), and underscores (_) are allowed"
    );
}

#[test]
fn test_identifier_underscore_digit_valid() {
    // Underscore followed by digits is a valid identifier
    let kinds = tokenize_kinds("_123");
    assert_eq!(
        kinds,
        vec![TokenKind::Identifier("_123".to_string()), TokenKind::Eof]
    );
}
