//! Lexer error tests for the Lak compiler.
//!
//! These tests verify that lexical analysis errors are properly detected
//! and reported.

mod common;

use common::{CompileStage, compile_error};

#[test]
fn test_compile_error_syntax() {
    let result = compile_error(r#"fn main() -> void { println("unclosed }"#);
    let (stage, msg) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Lex),
        "Expected Lex error, got {:?}: {}",
        stage,
        msg
    );
    assert!(
        msg.contains("Unterminated"),
        "Expected 'Unterminated' in error: {}",
        msg
    );
}

#[test]
fn test_compile_error_invalid_escape() {
    let result = compile_error(r#"fn main() -> void { println("\z") }"#);
    let (stage, msg) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Lex),
        "Expected Lex error, got {:?}: {}",
        stage,
        msg
    );
    assert!(
        msg.contains("Unknown escape"),
        "Expected 'Unknown escape' in error: {}",
        msg
    );
}

#[test]
fn test_compile_error_i64_overflow() {
    // i64::MAX + 1 should cause lexer error (overflow during parsing)
    let result = compile_error(
        r#"fn main() -> void {
    let x: i64 = 9223372036854775808
}"#,
    );
    let (stage, msg) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Lex),
        "Expected Lex error for i64 overflow, got {:?}: {}",
        stage,
        msg
    );
    assert!(
        msg.contains("out of range for i64"),
        "Expected 'out of range for i64' in error: {}",
        msg
    );
}

#[test]
fn test_negative_number_not_supported() {
    // Negative literals like -42 are not supported; `-` is only valid in `->`
    let result = compile_error(
        r#"fn main() -> void {
    let x: i32 = -42
}"#,
    );
    let (stage, msg) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Lex),
        "Expected Lex error for negative number, got {:?}: {}",
        stage,
        msg
    );
    // The lexer expects `->` after `-`, so we get an error about that
    assert!(
        msg.contains("Expected '>' after '-'"),
        "Expected error about '->' arrow, got: {}",
        msg
    );
}
