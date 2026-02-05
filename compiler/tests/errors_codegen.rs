//! Code generation error tests for the Lak compiler.
//!
//! These tests verify that semantic errors are properly detected
//! and reported during code generation.

mod common;

use common::{CompileStage, compile_error};

#[test]
fn test_compile_error_unknown_function() {
    let result = compile_error(r#"fn main() -> void { unknown_func("test") }"#);
    let (stage, msg) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Codegen),
        "Expected Codegen error, got {:?}: {}",
        stage,
        msg
    );
    assert!(
        msg.contains("Unknown function"),
        "Expected 'Unknown function' in error: {}",
        msg
    );
}

#[test]
fn test_compile_error_missing_main() {
    let result = compile_error("");
    let (stage, msg) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Codegen),
        "Expected Codegen error, got {:?}: {}",
        stage,
        msg
    );
    assert!(
        msg.contains("No main function"),
        "Expected 'No main function' in error: {}",
        msg
    );
}

#[test]
fn test_compile_error_main_wrong_return_type() {
    let result = compile_error("fn main() -> int {}");
    let (stage, msg) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Codegen),
        "Expected Codegen error, got {:?}: {}",
        stage,
        msg
    );
    assert!(
        msg.contains("must return void"),
        "Expected 'must return void' in error: {}",
        msg
    );
}

#[test]
fn test_compile_error_duplicate_variable() {
    let result = compile_error(
        r#"fn main() -> void {
    let x: i32 = 1
    let x: i32 = 2
}"#,
    );
    let (stage, msg) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Codegen),
        "Expected Codegen error, got {:?}: {}",
        stage,
        msg
    );
    assert!(
        msg.contains("already defined"),
        "Expected 'already defined' in error: {}",
        msg
    );
}

#[test]
fn test_compile_error_undefined_variable() {
    let result = compile_error(
        r#"fn main() -> void {
    let x: i32 = undefined_var
}"#,
    );
    let (stage, msg) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Codegen),
        "Expected Codegen error, got {:?}: {}",
        stage,
        msg
    );
    assert!(
        msg.contains("Undefined variable"),
        "Expected 'Undefined variable' in error: {}",
        msg
    );
}

#[test]
fn test_compile_error_type_mismatch() {
    let result = compile_error(
        r#"fn main() -> void {
    let x: i32 = 42
    let y: i64 = x
}"#,
    );
    let (stage, msg) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Codegen),
        "Expected Codegen error, got {:?}: {}",
        stage,
        msg
    );
    assert!(
        msg.contains("Type mismatch"),
        "Expected 'Type mismatch' in error: {}",
        msg
    );
}

#[test]
fn test_compile_error_i32_overflow() {
    // i32::MAX + 1 = 2147483648 should overflow i32
    let result = compile_error(
        r#"fn main() -> void {
    let x: i32 = 2147483648
}"#,
    );
    let (stage, msg) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Codegen),
        "Expected Codegen error, got {:?}: {}",
        stage,
        msg
    );
    assert!(
        msg.contains("out of range for i32"),
        "Expected 'out of range for i32' in error: {}",
        msg
    );
}

#[test]
fn test_compile_error_i32_large_value_overflow() {
    // Test that very large values (i64 max) are rejected for i32
    // Note: Negative literals are not yet supported
    let result = compile_error(
        r#"fn main() -> void {
    let x: i32 = 9223372036854775807
}"#,
    );
    let (stage, msg) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Codegen),
        "Expected Codegen error, got {:?}: {}",
        stage,
        msg
    );
    assert!(
        msg.contains("out of range for i32"),
        "Expected 'out of range for i32' in error: {}",
        msg
    );
}

#[test]
fn test_compile_error_duplicate_variable_different_type() {
    // Duplicate variable with different type should still be rejected
    let result = compile_error(
        r#"fn main() -> void {
    let x: i32 = 1
    let x: i64 = 2
}"#,
    );
    let (stage, msg) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Codegen),
        "Expected Codegen error, got {:?}: {}",
        stage,
        msg
    );
    assert!(
        msg.contains("already defined"),
        "Expected 'already defined' in error: {}",
        msg
    );
}

#[test]
fn test_compile_error_forward_reference() {
    // Variable used before it's defined
    let result = compile_error(
        r#"fn main() -> void {
    let x: i32 = y
    let y: i32 = 1
}"#,
    );
    let (stage, msg) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Codegen),
        "Expected Codegen error, got {:?}: {}",
        stage,
        msg
    );
    assert!(
        msg.contains("Undefined variable"),
        "Expected 'Undefined variable' in error: {}",
        msg
    );
}

#[test]
fn test_compile_error_variable_in_println() {
    // Passing a variable to println should error (println only accepts string literals)
    let result = compile_error(
        r#"fn main() -> void {
    let x: i32 = 42
    println(x)
}"#,
    );
    let (stage, msg) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Codegen),
        "Expected Codegen error, got {:?}: {}",
        stage,
        msg
    );
    assert!(
        msg.contains("string literal"),
        "Expected 'string literal' in error: {}",
        msg
    );
}

#[test]
fn test_compile_error_println_int_literal() {
    // Passing an integer literal directly to println should error
    let result = compile_error(
        r#"fn main() -> void {
    println(42)
}"#,
    );
    let (stage, msg) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Codegen),
        "Expected Codegen error, got {:?}: {}",
        stage,
        msg
    );
    assert!(
        msg.contains("string literal"),
        "Expected 'string literal' in error: {}",
        msg
    );
}
