//! Tests for whitespace handling (Go-compatible specification).
//!
//! Valid whitespace: space (U+0020), tab (U+0009), CR (U+000D), LF (U+000A).
//! All other Unicode whitespace characters are rejected.

use super::*;

#[test]
fn test_valid_whitespace_space() {
    let kinds = tokenize_kinds("a   b");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Identifier("a".to_string()),
            TokenKind::Identifier("b".to_string()),
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_valid_whitespace_tab() {
    let kinds = tokenize_kinds("a\tb");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Identifier("a".to_string()),
            TokenKind::Identifier("b".to_string()),
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_valid_whitespace_carriage_return() {
    let kinds = tokenize_kinds("a\rb");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Identifier("a".to_string()),
            TokenKind::Identifier("b".to_string()),
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_valid_whitespace_mixed() {
    let kinds = tokenize_kinds("a \t\r b");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Identifier("a".to_string()),
            TokenKind::Identifier("b".to_string()),
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_invalid_whitespace_fullwidth_space() {
    // U+3000 (full-width space) should be rejected
    let err = tokenize_error("a\u{3000}b");
    assert!(
        err.message.contains("Invalid whitespace character"),
        "Expected 'Invalid whitespace character' in error: {}",
        err.message
    );
    assert!(
        err.message.contains("U+3000"),
        "Expected Unicode codepoint in error: {}",
        err.message
    );
}

#[test]
fn test_invalid_whitespace_nbsp() {
    // U+00A0 (non-breaking space) should be rejected
    let err = tokenize_error("a\u{00A0}b");
    assert!(
        err.message.contains("Invalid whitespace character"),
        "Expected 'Invalid whitespace character' in error: {}",
        err.message
    );
    assert!(
        err.message.contains("U+00A0"),
        "Expected Unicode codepoint in error: {}",
        err.message
    );
}

#[test]
fn test_invalid_whitespace_line_separator() {
    // U+2028 (line separator) should be rejected
    let err = tokenize_error("a\u{2028}b");
    assert!(
        err.message.contains("Invalid whitespace character"),
        "Expected 'Invalid whitespace character' in error: {}",
        err.message
    );
}

#[test]
fn test_invalid_whitespace_paragraph_separator() {
    // U+2029 (paragraph separator) should be rejected
    let err = tokenize_error("a\u{2029}b");
    assert!(
        err.message.contains("Invalid whitespace character"),
        "Expected 'Invalid whitespace character' in error: {}",
        err.message
    );
}

#[test]
fn test_invalid_whitespace_en_quad() {
    // U+2000 (en quad) should be rejected
    let err = tokenize_error("a\u{2000}b");
    assert!(
        err.message.contains("Invalid whitespace character"),
        "Expected 'Invalid whitespace character' in error: {}",
        err.message
    );
}

#[test]
fn test_invalid_whitespace_vertical_tab() {
    // U+000B (vertical tab) should be rejected
    let err = tokenize_error("a\u{000B}b");
    assert!(
        err.message.contains("Invalid whitespace character"),
        "Expected 'Invalid whitespace character' in error: {}",
        err.message
    );
    assert!(
        err.message.contains("U+000B"),
        "Expected Unicode codepoint in error: {}",
        err.message
    );
}

#[test]
fn test_invalid_whitespace_form_feed() {
    // U+000C (form feed) should be rejected
    let err = tokenize_error("a\u{000C}b");
    assert!(
        err.message.contains("Invalid whitespace character"),
        "Expected 'Invalid whitespace character' in error: {}",
        err.message
    );
    assert!(
        err.message.contains("U+000C"),
        "Expected Unicode codepoint in error: {}",
        err.message
    );
}

#[test]
fn test_invalid_whitespace_next_line() {
    // U+0085 (NEL - Next Line) should be rejected
    let err = tokenize_error("a\u{0085}b");
    assert!(
        err.message.contains("Invalid whitespace character"),
        "Expected 'Invalid whitespace character' in error: {}",
        err.message
    );
    assert!(
        err.message.contains("U+0085"),
        "Expected Unicode codepoint in error: {}",
        err.message
    );
}
