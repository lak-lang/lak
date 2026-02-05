//! Parser error tests for the Lak compiler.
//!
//! These tests verify that syntax errors are properly detected
//! and reported during parsing.

mod common;

use common::{CompileStage, compile_error};

#[test]
fn test_compile_error_top_level_statement() {
    let result = compile_error(r#"println("hello")"#);
    let (stage, msg) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Parse),
        "Expected Parse error, got {:?}: {}",
        stage,
        msg
    );
}

#[test]
fn test_compile_error_unknown_type() {
    let result = compile_error(
        r#"fn main() -> void {
    let x: unknown = 42
}"#,
    );
    let (stage, msg) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Parse),
        "Expected Parse error, got {:?}: {}",
        stage,
        msg
    );
    assert!(
        msg.contains("Unknown type"),
        "Expected 'Unknown type' in error: {}",
        msg
    );
}

#[test]
fn test_compile_error_let_missing_variable_name() {
    // `let : i32 = 42` - missing variable name should be a parse error
    let result = compile_error(
        r#"fn main() -> void {
    let : i32 = 42
}"#,
    );
    let (stage, msg) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Parse),
        "Expected Parse error, got {:?}: {}",
        stage,
        msg
    );
    // Parser expects an identifier after 'let'
    assert!(
        msg.contains("Expected identifier") || msg.contains("expected identifier"),
        "Expected 'identifier' in error: {}",
        msg
    );
}
