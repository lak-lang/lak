//! Lexer error tests for the Lak compiler.
//!
//! These tests verify that lexical analysis errors are properly detected
//! and reported.

mod common;

use common::{CompileErrorKind, CompileStage, compile_error_with_kind};
use lak::lexer::LexErrorKind;

#[test]
fn test_compile_error_syntax() {
    let result = compile_error_with_kind(r#"fn main() -> void { println("unclosed }"#);
    let (stage, msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Lex),
        "Expected Lex error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Unterminated string literal");
    assert_eq!(
        kind,
        CompileErrorKind::Lex(LexErrorKind::UnterminatedString),
        "Expected UnterminatedString error kind"
    );
}

#[test]
fn test_compile_error_invalid_escape() {
    let result = compile_error_with_kind(r#"fn main() -> void { println("\z") }"#);
    let (stage, msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Lex),
        "Expected Lex error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Unknown escape sequence: '\\z'");
    assert_eq!(
        kind,
        CompileErrorKind::Lex(LexErrorKind::UnknownEscapeSequence),
        "Expected UnknownEscapeSequence error kind"
    );
}

#[test]
fn test_compile_error_i64_overflow() {
    // i64::MAX + 1 should cause lexer error (overflow during parsing)
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: i64 = 9223372036854775808
}"#,
    );
    let (stage, msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Lex),
        "Expected Lex error for i64 overflow, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Integer literal '9223372036854775808' is out of range for i64 (exceeds maximum value)"
    );
    assert_eq!(
        kind,
        CompileErrorKind::Lex(LexErrorKind::IntegerOverflow),
        "Expected IntegerOverflow error kind"
    );
}

#[test]
fn test_non_ascii_identifier_rejected() {
    // Non-ASCII characters in identifiers should be rejected
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let 変数: i32 = 42
}"#,
    );
    let (stage, msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Lex),
        "Expected Lex error for non-ASCII identifier, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Invalid character '変' in identifier. Only ASCII letters (a-z, A-Z), digits (0-9), and underscores (_) are allowed"
    );
    assert_eq!(
        kind,
        CompileErrorKind::Lex(LexErrorKind::InvalidIdentifierCharacter),
        "Expected InvalidIdentifierCharacter error kind"
    );
}

#[test]
fn test_greek_letters_rejected() {
    // Greek letters should be rejected in identifiers
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let αβγ: i64 = 100
}"#,
    );
    let (stage, msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Lex),
        "Expected Lex error for Greek letters, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Invalid character 'α' in identifier. Only ASCII letters (a-z, A-Z), digits (0-9), and underscores (_) are allowed"
    );
    assert_eq!(
        kind,
        CompileErrorKind::Lex(LexErrorKind::InvalidIdentifierCharacter),
        "Expected InvalidIdentifierCharacter error kind"
    );
}

#[test]
fn test_emoji_in_code_rejected() {
    // Emoji should be rejected with "Unexpected character"
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let 🚀: i32 = 42
}"#,
    );
    let (stage, msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Lex),
        "Expected Lex error for emoji, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Unexpected character: '🚀'");
    assert_eq!(
        kind,
        CompileErrorKind::Lex(LexErrorKind::UnexpectedCharacter),
        "Expected UnexpectedCharacter error kind"
    );
}

#[test]
fn test_fullwidth_space_rejected() {
    // U+3000 (full-width space) should be rejected
    let result = compile_error_with_kind("fn main() -> void {\u{3000}println(\"test\")\n}");
    let (stage, msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Lex),
        "Expected Lex error for full-width space, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Invalid whitespace character '\u{3000}' (U+3000). Only space, tab, carriage return, and newline are allowed"
    );
    assert_eq!(
        kind,
        CompileErrorKind::Lex(LexErrorKind::InvalidWhitespace),
        "Expected InvalidWhitespace error kind"
    );
}

#[test]
fn test_nbsp_rejected() {
    // U+00A0 (non-breaking space) should be rejected
    let result = compile_error_with_kind("fn main() -> void {\n    let\u{00A0}x: i32 = 42\n}");
    let (stage, msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Lex),
        "Expected Lex error for non-breaking space, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Invalid whitespace character '\u{00A0}' (U+00A0). Only space, tab, carriage return, and newline are allowed"
    );
    assert_eq!(
        kind,
        CompileErrorKind::Lex(LexErrorKind::InvalidWhitespace),
        "Expected InvalidWhitespace error kind"
    );
}
