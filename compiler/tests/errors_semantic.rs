//! Semantic analysis error tests for the Lak compiler.
//!
//! These tests verify that semantic errors are properly detected
//! and reported during semantic analysis.

mod common;

use common::{CompileStage, compile_error};

#[test]
fn test_compile_error_unknown_function() {
    let result = compile_error(r#"fn main() -> void { unknown_func("test") }"#);
    let (stage, msg) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
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
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
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
fn test_compile_error_missing_main_with_other_functions() {
    let result = compile_error("fn hoge() -> void {}");
    let (stage, msg) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert!(
        msg.contains("No main function"),
        "Expected 'No main function' in error: {}",
        msg
    );
    assert!(
        msg.contains("hoge"),
        "Expected defined function name 'hoge' in error: {}",
        msg
    );
}

#[test]
fn test_compile_error_main_wrong_return_type() {
    let result = compile_error("fn main() -> int {}");
    let (stage, msg) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
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
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
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
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
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
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
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
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
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
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
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
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
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
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
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
fn test_compile_error_string_literal_to_int() {
    // String literal cannot be assigned to i32 variable
    let result = compile_error(
        r#"fn main() -> void {
    let x: i32 = "hello"
}"#,
    );
    let (stage, msg) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert!(
        msg.contains("string literal cannot be assigned to type 'i32'"),
        "Expected type mismatch error: {}",
        msg
    );
}

#[test]
fn test_compile_error_int_literal_to_string() {
    // Integer literal cannot be assigned to string variable
    let result = compile_error(
        r#"fn main() -> void {
    let s: string = 42
}"#,
    );
    let (stage, msg) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert!(
        msg.contains("integer literal 42 cannot be assigned to type 'string'"),
        "Expected type mismatch error: {}",
        msg
    );
}

#[test]
fn test_compile_error_duplicate_function() {
    // Duplicate function definition (issues/1.md)
    let result = compile_error(
        r#"fn main() -> void {}
fn main() -> void {}"#,
    );
    let (stage, msg) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
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
fn test_compile_error_function_call_with_args() {
    // Calling a parameterless function with arguments should error
    let result = compile_error(
        r#"
fn helper() -> void {
    println("test")
}

fn main() -> void {
    helper(42)
}
"#,
    );
    let (stage, msg) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert!(
        msg.contains("expects 0 arguments"),
        "Expected 'expects 0 arguments' in error: {}",
        msg
    );
}

#[test]
fn test_compile_error_function_call_with_multiple_args() {
    // Calling a parameterless function with multiple arguments
    let result = compile_error(
        r#"
fn helper() -> void {
    println("test")
}

fn main() -> void {
    helper("a", "b", "c")
}
"#,
    );
    let (stage, msg) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert!(
        msg.contains("expects 0 arguments") && msg.contains("3"),
        "Expected 'expects 0 arguments...3' in error: {}",
        msg
    );
}

#[test]
fn test_compile_error_call_main_directly() {
    // Calling main function directly should be an error
    let result = compile_error(
        r#"
fn helper() -> void {
    main()
}

fn main() -> void {
    println("test")
}
"#,
    );
    let (stage, msg) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert!(
        msg.contains("Cannot call 'main'"),
        "Expected 'Cannot call main' in error: {}",
        msg
    );
}

#[test]
fn test_compile_error_call_main_from_main() {
    // Calling main from main itself should also be an error
    let result = compile_error(
        r#"
fn main() -> void {
    main()
}
"#,
    );
    let (stage, msg) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert!(
        msg.contains("Cannot call 'main'"),
        "Expected 'Cannot call main' in error: {}",
        msg
    );
}

#[test]
fn test_compile_error_int_variable_to_string() {
    // Int variable cannot be assigned to string variable
    let result = compile_error(
        r#"fn main() -> void {
    let x: i32 = 42
    let s: string = x
}"#,
    );
    let (stage, msg) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
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
fn test_compile_error_string_variable_as_statement() {
    // String variable used as standalone statement should error (no effect)
    let result = compile_error(
        r#"fn main() -> void {
    let s: string = "hello"
    s
}"#,
    );
    let (stage, msg) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert!(
        msg.contains("used as a statement has no effect"),
        "Expected 'used as a statement has no effect' in error: {}",
        msg
    );
}
