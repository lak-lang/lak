//! Semantic analysis error tests for the Lak compiler.
//!
//! These tests verify that semantic errors are properly detected
//! and reported during semantic analysis.

mod common;

use common::{CompileErrorKind, CompileStage, compile_error_with_kind};
use lak::semantic::SemanticErrorKind;

#[test]
fn test_compile_error_unknown_function() {
    let result = compile_error_with_kind(r#"fn main() -> void { unknown_func("test") }"#);
    let (stage, msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Undefined function: 'unknown_func'");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::UndefinedFunction),
        "Expected UndefinedFunction error kind"
    );
}

#[test]
fn test_compile_error_missing_main() {
    let result = compile_error_with_kind("");
    let (stage, msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "No main function found: program contains no function definitions"
    );
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::MissingMainFunction),
        "Expected MissingMainFunction error kind"
    );
}

#[test]
fn test_compile_error_missing_main_with_other_functions() {
    let result = compile_error_with_kind("fn hoge() -> void {}");
    let (stage, msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "No main function found. Defined functions: [\"hoge\"]");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::MissingMainFunction),
        "Expected MissingMainFunction error kind"
    );
}

#[test]
fn test_compile_error_main_wrong_return_type() {
    let result = compile_error_with_kind("fn main() -> int {}");
    let (stage, msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "main function must return void, but found return type 'int'"
    );
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::InvalidMainSignature),
        "Expected InvalidMainSignature error kind"
    );
}

#[test]
fn test_compile_error_duplicate_variable() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: i32 = 1
    let x: i32 = 2
}"#,
    );
    let (stage, msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Variable 'x' is already defined at 2:5");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::DuplicateVariable),
        "Expected DuplicateVariable error kind"
    );
}

#[test]
fn test_compile_error_undefined_variable() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: i32 = undefined_var
}"#,
    );
    let (stage, msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Undefined variable: 'undefined_var'");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::UndefinedVariable),
        "Expected UndefinedVariable error kind"
    );
}

#[test]
fn test_compile_error_type_mismatch() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: i32 = 42
    let y: i64 = x
}"#,
    );
    let (stage, msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Type mismatch: variable 'x' has type 'i32', expected 'i64'"
    );
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch),
        "Expected TypeMismatch error kind"
    );
}

#[test]
fn test_compile_error_i32_overflow() {
    // i32::MAX + 1 = 2147483648 should overflow i32
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: i32 = 2147483648
}"#,
    );
    let (stage, msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Integer literal '2147483648' is out of range for i32 (valid range: -2147483648 to 2147483647)"
    );
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::IntegerOverflow),
        "Expected IntegerOverflow error kind"
    );
}

#[test]
fn test_compile_error_i32_large_value_overflow() {
    // Test that very large values (i64 max) are rejected for i32
    // Note: Negative literals are not yet supported
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: i32 = 9223372036854775807
}"#,
    );
    let (stage, msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Integer literal '9223372036854775807' is out of range for i32 (valid range: -2147483648 to 2147483647)"
    );
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::IntegerOverflow),
        "Expected IntegerOverflow error kind"
    );
}

#[test]
fn test_compile_error_duplicate_variable_different_type() {
    // Duplicate variable with different type should still be rejected
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: i32 = 1
    let x: i64 = 2
}"#,
    );
    let (stage, msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Variable 'x' is already defined at 2:5");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::DuplicateVariable),
        "Expected DuplicateVariable error kind"
    );
}

#[test]
fn test_compile_error_forward_reference() {
    // Variable used before it's defined
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: i32 = y
    let y: i32 = 1
}"#,
    );
    let (stage, msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Undefined variable: 'y'");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::UndefinedVariable),
        "Expected UndefinedVariable error kind"
    );
}

#[test]
fn test_compile_error_string_literal_to_int() {
    // String literal cannot be assigned to i32 variable
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: i32 = "hello"
}"#,
    );
    let (stage, msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Type mismatch: string literal cannot be assigned to type 'i32'"
    );
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch),
        "Expected TypeMismatch error kind"
    );
}

#[test]
fn test_compile_error_int_literal_to_string() {
    // Integer literal cannot be assigned to string variable
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let s: string = 42
}"#,
    );
    let (stage, msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Type mismatch: integer literal '42' cannot be assigned to type 'string'"
    );
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch),
        "Expected TypeMismatch error kind"
    );
}

#[test]
fn test_compile_error_duplicate_function() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {}
fn main() -> void {}"#,
    );
    let (stage, msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Function 'main' is already defined at 1:1");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::DuplicateFunction),
        "Expected DuplicateFunction error kind"
    );
}

#[test]
fn test_compile_error_function_call_with_args() {
    // Calling a parameterless function with arguments should error
    let result = compile_error_with_kind(
        r#"
fn helper() -> void {
    println("test")
}

fn main() -> void {
    helper(42)
}
"#,
    );
    let (stage, msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Function 'helper' expects 0 arguments, but got 1");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::InvalidArgument),
        "Expected InvalidArgument error kind"
    );
}

#[test]
fn test_compile_error_function_call_with_multiple_args() {
    // Calling a parameterless function with multiple arguments
    let result = compile_error_with_kind(
        r#"
fn helper() -> void {
    println("test")
}

fn main() -> void {
    helper("a", "b", "c")
}
"#,
    );
    let (stage, msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Function 'helper' expects 0 arguments, but got 3");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::InvalidArgument),
        "Expected InvalidArgument error kind"
    );
}

#[test]
fn test_compile_error_call_main_directly() {
    // Calling main function directly should be an error
    let result = compile_error_with_kind(
        r#"
fn helper() -> void {
    main()
}

fn main() -> void {
    println("test")
}
"#,
    );
    let (stage, msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Cannot call 'main' function directly");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::InvalidArgument),
        "Expected InvalidArgument error kind"
    );
}

#[test]
fn test_compile_error_call_main_from_main() {
    // Calling main from main itself should also be an error
    let result = compile_error_with_kind(
        r#"
fn main() -> void {
    main()
}
"#,
    );
    let (stage, msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Cannot call 'main' function directly");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::InvalidArgument),
        "Expected InvalidArgument error kind"
    );
}

#[test]
fn test_compile_error_int_variable_to_string() {
    // Int variable cannot be assigned to string variable
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: i32 = 42
    let s: string = x
}"#,
    );
    let (stage, msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Type mismatch: variable 'x' has type 'i32', expected 'string'"
    );
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch),
        "Expected TypeMismatch error kind"
    );
}

#[test]
fn test_compile_error_string_variable_as_statement() {
    // String variable used as standalone statement should error (no effect)
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let s: string = "hello"
    s
}"#,
    );
    let (stage, msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Variable 's' used as a statement has no effect. Did you mean to use it in an expression?"
    );
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::InvalidExpression),
        "Expected InvalidExpression error kind"
    );
}
