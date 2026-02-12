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
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Undefined function: 'unknown_func'");
    assert_eq!(short_msg, "Undefined function");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::UndefinedFunction),
        "Expected UndefinedFunction error kind"
    );
}

#[test]
fn test_compile_error_missing_main() {
    let result = compile_error_with_kind("");
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
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
    assert_eq!(short_msg, "Missing main function");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::MissingMainFunction),
        "Expected MissingMainFunction error kind"
    );
}

#[test]
fn test_compile_error_missing_main_with_other_functions() {
    let result = compile_error_with_kind("fn hoge() -> void {}");
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "No main function found. Defined functions: 'hoge'");
    assert_eq!(short_msg, "Missing main function");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::MissingMainFunction),
        "Expected MissingMainFunction error kind"
    );
}

#[test]
fn test_compile_error_main_wrong_return_type() {
    let result = compile_error_with_kind("fn main() -> int {}");
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
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
    assert_eq!(short_msg, "Invalid main signature");
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
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Variable 'x' is already defined at 2:5");
    assert_eq!(short_msg, "Duplicate variable");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::DuplicateVariable),
        "Expected DuplicateVariable error kind"
    );
}

#[test]
fn test_compile_error_duplicate_parameter_name() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {}
fn dup(a: i32, a: i32) -> void {}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Variable 'a' is already defined at 2:8");
    assert_eq!(short_msg, "Duplicate variable");
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
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Undefined variable: 'undefined_var'");
    assert_eq!(short_msg, "Undefined variable");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::UndefinedVariable),
        "Expected UndefinedVariable error kind"
    );
}

#[test]
fn test_compile_error_self_referential_let_initializer() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: i32 = x
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Undefined variable: 'x'");
    assert_eq!(short_msg, "Undefined variable");
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
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
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
    assert_eq!(short_msg, "Type mismatch");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch),
        "Expected TypeMismatch error kind"
    );
}

#[test]
fn test_compile_error_if_condition_must_be_bool() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    if 1 {
        println("x")
    }
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Type mismatch: integer literal '1' cannot be assigned to type 'bool'"
    );
    assert_eq!(short_msg, "Type mismatch");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch),
        "Expected TypeMismatch error kind"
    );
}

#[test]
fn test_compile_error_if_expression_branch_type_mismatch() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let value: i64 = if true { 42 } else { "hello" }
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Type mismatch in if expression: then branch is 'i64', else branch is 'string'"
    );
    assert_eq!(short_msg, "If expression branch type mismatch");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::IfExpressionBranchTypeMismatch),
        "Expected IfExpressionBranchTypeMismatch error kind"
    );
}

#[test]
fn test_compile_error_if_expression_type_mismatch_to_expected_type() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let value: bool = if true { 42 } else { 24 }
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Type mismatch: if expression has type 'i64', expected 'bool'"
    );
    assert_eq!(short_msg, "Type mismatch");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch),
        "Expected TypeMismatch error kind"
    );
}

#[test]
fn test_compile_error_if_expression_branch_i32_overflow_is_preserved() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: i32 = if true { 2147483648 } else { 1 }
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
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
    assert_eq!(short_msg, "Integer overflow");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::IntegerOverflow),
        "Expected IntegerOverflow error kind"
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
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
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
    assert_eq!(short_msg, "Integer overflow");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::IntegerOverflow),
        "Expected IntegerOverflow error kind"
    );
}

#[test]
fn test_compile_error_i32_overflow_in_println_binary_op_with_variable() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: i32 = 1
    println(2147483648 + x)
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
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
    assert_eq!(short_msg, "Integer overflow");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::IntegerOverflow),
        "Expected IntegerOverflow error kind"
    );
}

#[test]
fn test_compile_error_i32_overflow_in_typed_binary_op_with_variable() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: i32 = 1
    let y: i32 = 2147483648 + x
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
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
    assert_eq!(short_msg, "Integer overflow");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::IntegerOverflow),
        "Expected IntegerOverflow error kind"
    );
}

#[test]
fn test_compile_error_i32_negative_overflow_in_println_binary_op_with_variable() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: i32 = 1
    println(-2147483649 + x)
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Integer literal '-2147483649' is out of range for i32 (valid range: -2147483648 to 2147483647)"
    );
    assert_eq!(short_msg, "Integer overflow");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::IntegerOverflow),
        "Expected IntegerOverflow error kind"
    );
}

#[test]
fn test_compile_error_i32_large_value_overflow() {
    // Test that very large values (i64 max) are rejected for i32.
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: i32 = 9223372036854775807
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
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
    assert_eq!(short_msg, "Integer overflow");
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
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Variable 'x' is already defined at 2:5");
    assert_eq!(short_msg, "Duplicate variable");
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
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Undefined variable: 'y'");
    assert_eq!(short_msg, "Undefined variable");
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
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
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
    assert_eq!(short_msg, "Type mismatch");
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
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
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
    assert_eq!(short_msg, "Type mismatch");
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
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Function 'main' is already defined at 1:1");
    assert_eq!(short_msg, "Duplicate function");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::DuplicateFunction),
        "Expected DuplicateFunction error kind"
    );
}

#[test]
fn test_compile_error_reserved_prelude_function_println() {
    let result = compile_error_with_kind(
        r#"fn println() -> void {}
fn main() -> void {}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Function name 'println' is reserved by the prelude and cannot be redefined"
    );
    assert_eq!(short_msg, "Invalid argument");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::InvalidArgument),
        "Expected InvalidArgument error kind"
    );
}

#[test]
fn test_compile_error_reserved_prelude_function_panic() {
    let result = compile_error_with_kind(
        r#"fn panic() -> void {}
fn main() -> void {}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Function name 'panic' is reserved by the prelude and cannot be redefined"
    );
    assert_eq!(short_msg, "Invalid argument");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::InvalidArgument),
        "Expected InvalidArgument error kind"
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
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Function 'helper' expects 0 arguments, but got 1");
    assert_eq!(short_msg, "Invalid argument");
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
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Function 'helper' expects 0 arguments, but got 3");
    assert_eq!(short_msg, "Invalid argument");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::InvalidArgument),
        "Expected InvalidArgument error kind"
    );
}

#[test]
fn test_compile_error_function_call_with_too_few_args_for_parameterized_function() {
    let result = compile_error_with_kind(
        r#"
fn helper(name: string, age: i32) -> void {
    println(name)
    println(age)
}

fn main() -> void {
    helper("alice")
}
"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Function 'helper' expects 2 arguments, but got 1");
    assert_eq!(short_msg, "Invalid argument");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::InvalidArgument),
        "Expected InvalidArgument error kind"
    );
}

#[test]
fn test_compile_error_function_call_with_param_type_mismatch() {
    let result = compile_error_with_kind(
        r#"
fn helper(name: string) -> void {
    println(name)
}

fn main() -> void {
    helper(42)
}
"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
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
    assert_eq!(short_msg, "Type mismatch");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch),
        "Expected TypeMismatch error kind"
    );
}

#[test]
fn test_compile_error_main_function_with_parameters() {
    let result = compile_error_with_kind(
        r#"
fn main(x: i32) -> void {
    println(x)
}
"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "main function must not have parameters, but found 1");
    assert_eq!(short_msg, "Invalid main signature");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::InvalidMainSignature),
        "Expected InvalidMainSignature error kind"
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
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Cannot call 'main' function directly");
    assert_eq!(short_msg, "Invalid argument");
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
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Cannot call 'main' function directly");
    assert_eq!(short_msg, "Invalid argument");
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
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
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
    assert_eq!(short_msg, "Type mismatch");
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
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
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
    assert_eq!(short_msg, "Invalid expression");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::InvalidExpression),
        "Expected InvalidExpression error kind"
    );
}

#[test]
fn test_compile_error_binary_op_type_mismatch_in_operand() {
    // Binary operation with mismatched operand type (i64 variable used in i32 context)
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let a: i64 = 10
    let b: i32 = a + 1
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Type mismatch: variable 'a' has type 'i64', expected 'i32'"
    );
    assert_eq!(short_msg, "Type mismatch");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch),
        "Expected TypeMismatch error kind"
    );
}

#[test]
fn test_compile_error_binary_op_to_string_type() {
    // Binary operation result cannot be assigned to string type
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: string = 1 + 2
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Operator '+' cannot be used with 'string' type");
    assert_eq!(short_msg, "Type mismatch");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch),
        "Expected TypeMismatch error kind"
    );
}

#[test]
fn test_compile_error_binary_op_as_statement() {
    // Binary operation as statement has no effect
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: i32 = 5
    x + 10
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "This expression computes a value but the result is not used"
    );
    assert_eq!(short_msg, "Invalid expression");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::InvalidExpression),
        "Expected InvalidExpression error kind"
    );
}

// ========================================
// panic() built-in function error tests
// ========================================

#[test]
fn test_compile_error_panic_no_args() {
    let result = compile_error_with_kind(r#"fn main() -> void { panic() }"#);
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "panic expects exactly 1 argument");
    assert_eq!(short_msg, "Invalid argument");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::InvalidArgument),
        "Expected InvalidArgument error kind"
    );
}

#[test]
fn test_compile_error_panic_multiple_args() {
    let result = compile_error_with_kind(r#"fn main() -> void { panic("a", "b") }"#);
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "panic expects exactly 1 argument");
    assert_eq!(short_msg, "Invalid argument");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::InvalidArgument),
        "Expected InvalidArgument error kind"
    );
}

#[test]
fn test_compile_error_panic_int_arg() {
    let result = compile_error_with_kind(r#"fn main() -> void { panic(42) }"#);
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "panic requires a string argument, but got 'integer literal'"
    );
    assert_eq!(short_msg, "Invalid argument");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::InvalidArgument),
        "Expected InvalidArgument error kind"
    );
}

#[test]
fn test_compile_error_panic_i32_variable() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: i32 = 42
    panic(x)
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "panic requires a string argument, but got 'i32'");
    assert_eq!(short_msg, "Invalid argument");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::InvalidArgument),
        "Expected InvalidArgument error kind"
    );
}

#[test]
fn test_compile_error_panic_i64_variable() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: i64 = 42
    panic(x)
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "panic requires a string argument, but got 'i64'");
    assert_eq!(short_msg, "Invalid argument");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::InvalidArgument),
        "Expected InvalidArgument error kind"
    );
}

#[test]
fn test_compile_error_panic_undefined_variable() {
    let result = compile_error_with_kind(r#"fn main() -> void { panic(unknown) }"#);
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Undefined variable: 'unknown'");
    assert_eq!(short_msg, "Undefined variable");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::UndefinedVariable),
        "Expected UndefinedVariable error kind"
    );
}

#[test]
fn test_compile_error_panic_binary_op() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    panic(1 + 2)
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "panic requires a string argument, but got 'expression'"
    );
    assert_eq!(short_msg, "Invalid argument");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::InvalidArgument),
        "Expected InvalidArgument error kind"
    );
}

#[test]
fn test_compile_error_panic_function_call() {
    let result = compile_error_with_kind(
        r#"fn get_msg() -> void {}
fn main() -> void {
    panic(get_msg())
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Function call 'get_msg' cannot be used as println argument (functions returning values not yet supported)"
    );
    assert_eq!(short_msg, "Invalid argument");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::InvalidArgument),
        "Expected InvalidArgument error kind"
    );
}

#[test]
fn test_compile_error_unary_minus_on_string_variable() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let s: string = "hello"
    let x: string = -s
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Unary operator '-' cannot be used with 'string' type");
    assert_eq!(short_msg, "Type mismatch");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch),
        "Expected TypeMismatch error kind"
    );
}

#[test]
fn test_compile_error_unary_minus_on_string_variable_in_println() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let s: string = "hello"
    println(-s)
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Unary operator '-' cannot be used with 'string' type");
    assert_eq!(short_msg, "Type mismatch");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch),
        "Expected TypeMismatch error kind"
    );
}

#[test]
fn test_compile_error_unary_minus_on_string_literal() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: string = -"hello"
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Unary operator '-' cannot be used with 'string' type");
    assert_eq!(short_msg, "Type mismatch");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch),
        "Expected TypeMismatch error kind"
    );
}

#[test]
fn test_compile_error_unary_minus_on_string_literal_in_println() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    println(-"hello")
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Unary operator '-' cannot be used with 'string' type");
    assert_eq!(short_msg, "Type mismatch");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch),
        "Expected TypeMismatch error kind"
    );
}

// ========================================
// Boolean type error tests
// ========================================

#[test]
fn test_compile_error_bool_literal_to_i32() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: i32 = true
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Type mismatch: boolean literal cannot be assigned to type 'i32'"
    );
    assert_eq!(short_msg, "Type mismatch");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch),
        "Expected TypeMismatch error kind"
    );
}

#[test]
fn test_compile_error_bool_literal_to_string() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: string = false
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Type mismatch: boolean literal cannot be assigned to type 'string'"
    );
    assert_eq!(short_msg, "Type mismatch");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch),
        "Expected TypeMismatch error kind"
    );
}

#[test]
fn test_compile_error_int_literal_to_bool() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: bool = 42
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Type mismatch: integer literal '42' cannot be assigned to type 'bool'"
    );
    assert_eq!(short_msg, "Type mismatch");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch),
        "Expected TypeMismatch error kind"
    );
}

#[test]
fn test_compile_error_string_literal_to_bool() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: bool = "hello"
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Type mismatch: string literal cannot be assigned to type 'bool'"
    );
    assert_eq!(short_msg, "Type mismatch");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch),
        "Expected TypeMismatch error kind"
    );
}

#[test]
fn test_compile_error_bool_literal_as_statement() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    true
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Boolean literal as a statement has no effect. Did you mean to use it in a condition?"
    );
    assert_eq!(short_msg, "Invalid expression");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::InvalidExpression),
        "Expected InvalidExpression error kind"
    );
}

#[test]
fn test_compile_error_panic_bool_literal() {
    let result = compile_error_with_kind(r#"fn main() -> void { panic(true) }"#);
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "panic requires a string argument, but got 'boolean literal'"
    );
    assert_eq!(short_msg, "Invalid argument");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::InvalidArgument),
        "Expected InvalidArgument error kind"
    );
}

#[test]
fn test_compile_error_panic_bool_variable() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let flag: bool = true
    panic(flag)
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "panic requires a string argument, but got 'bool'");
    assert_eq!(short_msg, "Invalid argument");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::InvalidArgument),
        "Expected InvalidArgument error kind"
    );
}

#[test]
fn test_compile_error_bool_variable_type_mismatch() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let flag: bool = true
    let x: i32 = flag
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Type mismatch: variable 'flag' has type 'bool', expected 'i32'"
    );
    assert_eq!(short_msg, "Type mismatch");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch),
        "Expected TypeMismatch error kind"
    );
}

// ========================================
// Boolean operator error tests
// ========================================

#[test]
fn test_compile_error_bool_addition() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: bool = true + false
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Operator '+' cannot be used with 'bool' type");
    assert_eq!(short_msg, "Type mismatch");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch),
        "Expected TypeMismatch error kind"
    );
}

#[test]
fn test_compile_error_bool_subtraction() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: bool = true - false
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Operator '-' cannot be used with 'bool' type");
    assert_eq!(short_msg, "Type mismatch");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch),
        "Expected TypeMismatch error kind"
    );
}

#[test]
fn test_compile_error_bool_multiplication() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: bool = true * false
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Operator '*' cannot be used with 'bool' type");
    assert_eq!(short_msg, "Type mismatch");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch),
        "Expected TypeMismatch error kind"
    );
}

#[test]
fn test_compile_error_bool_division() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: bool = true / false
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Operator '/' cannot be used with 'bool' type");
    assert_eq!(short_msg, "Type mismatch");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch),
        "Expected TypeMismatch error kind"
    );
}

#[test]
fn test_compile_error_bool_modulo() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: bool = true % false
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Operator '%' cannot be used with 'bool' type");
    assert_eq!(short_msg, "Type mismatch");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch),
        "Expected TypeMismatch error kind"
    );
}

#[test]
fn test_compile_error_unary_minus_on_bool_literal() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: bool = -true
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Unary operator '-' cannot be used with 'bool' type");
    assert_eq!(short_msg, "Type mismatch");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch),
        "Expected TypeMismatch error kind"
    );
}

#[test]
fn test_compile_error_unary_minus_on_bool_variable() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let flag: bool = true
    let x: bool = -flag
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Unary operator '-' cannot be used with 'bool' type");
    assert_eq!(short_msg, "Type mismatch");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch),
        "Expected TypeMismatch error kind"
    );
}

// ========================================
// Module access error tests (Phase 2)
// ========================================

#[test]
fn test_compile_error_member_call_not_implemented() {
    // Module-qualified function call (module.function()) without an import returns ModuleNotImported
    let result = compile_error_with_kind(
        r#"import "./math" as math

fn main() -> void {
    math.add()
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Module-qualified function call 'math.add()' requires an import statement. Add: import \"./math\""
    );
    assert_eq!(short_msg, "Module not imported");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::ModuleNotImported),
        "Expected ModuleNotImported error kind"
    );
}

#[test]
fn test_compile_error_member_access_not_implemented() {
    // Member access without call (module.value) triggers ModuleAccessNotImplemented
    let result = compile_error_with_kind(
        r#"import "./lib" as lib

fn main() -> void {
    lib.value
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Module-qualified access (e.g., module.function) is not yet implemented"
    );
    assert_eq!(short_msg, "Module access not implemented");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::ModuleAccessNotImplemented),
        "Expected ModuleAccessNotImplemented error kind"
    );
}

#[test]
fn test_compile_error_member_access_in_let() {
    // Member access in let statement also triggers ModuleAccessNotImplemented
    let result = compile_error_with_kind(
        r#"import "./config" as cfg

fn main() -> void {
    let x: i32 = cfg.value
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Module-qualified access (e.g., module.function) is not yet implemented"
    );
    assert_eq!(short_msg, "Module access not implemented");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::ModuleAccessNotImplemented),
        "Expected ModuleAccessNotImplemented error kind"
    );
}

// ============================================================================
// Comparison operator type errors
// ============================================================================

#[test]
fn test_comparison_string_operands() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: string = "hello"
    let y: string = "world"
    let result: bool = x < y
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Ordering operator '<' cannot be used with 'string' type"
    );
    assert_eq!(short_msg, "Type mismatch");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch),
        "Expected TypeMismatch error kind"
    );
}

#[test]
fn test_comparison_bool_operands() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: bool = true
    let y: bool = false
    let result: bool = x < y
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Ordering operator '<' cannot be used with 'bool' type");
    assert_eq!(short_msg, "Type mismatch");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch),
        "Expected TypeMismatch error kind"
    );
}

#[test]
fn test_comparison_mismatched_types() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: i32 = 5
    let y: i64 = 10
    let result: bool = x < y
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Type mismatch: variable 'y' has type 'i64', expected 'i32'"
    );
    assert_eq!(short_msg, "Type mismatch");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch),
        "Expected TypeMismatch error kind"
    );
}

#[test]
fn test_comparison_assigned_to_non_bool() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let result: i32 = 5 < 10
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Comparison operator '<' produces 'bool', but expected 'i32'"
    );
    assert_eq!(short_msg, "Type mismatch");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch),
        "Expected TypeMismatch error kind"
    );
}

#[test]
fn test_equality_assigned_to_non_bool() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let result: i32 = 5 == 10
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Comparison operator '==' produces 'bool', but expected 'i32'"
    );
    assert_eq!(short_msg, "Type mismatch");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch),
        "Expected TypeMismatch error kind"
    );
}

#[test]
fn test_not_equal_assigned_to_non_bool() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let result: i32 = 5 != 3
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Comparison operator '!=' produces 'bool', but expected 'i32'"
    );
    assert_eq!(short_msg, "Type mismatch");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch)
    );
}

#[test]
fn test_greater_than_assigned_to_non_bool() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let result: i32 = 5 > 3
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Comparison operator '>' produces 'bool', but expected 'i32'"
    );
    assert_eq!(short_msg, "Type mismatch");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch)
    );
}

#[test]
fn test_less_equal_assigned_to_non_bool() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let result: i32 = 3 <= 5
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Comparison operator '<=' produces 'bool', but expected 'i32'"
    );
    assert_eq!(short_msg, "Type mismatch");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch)
    );
}

#[test]
fn test_greater_equal_assigned_to_non_bool() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let result: i32 = 5 >= 3
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Comparison operator '>=' produces 'bool', but expected 'i32'"
    );
    assert_eq!(short_msg, "Type mismatch");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch)
    );
}

#[test]
fn test_ordering_string_operands_greater() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: string = "hello"
    let y: string = "world"
    let result: bool = x > y
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Ordering operator '>' cannot be used with 'string' type"
    );
    assert_eq!(short_msg, "Type mismatch");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch),
        "Expected TypeMismatch error kind"
    );
}

#[test]
fn test_ordering_bool_operands_greater() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: bool = true
    let y: bool = false
    let result: bool = x > y
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Ordering operator '>' cannot be used with 'bool' type");
    assert_eq!(short_msg, "Type mismatch");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch),
        "Expected TypeMismatch error kind"
    );
}

#[test]
fn test_println_comparison_type_mismatch() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    println(5 < "hello")
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Type mismatch: string literal cannot be assigned to type 'i64'"
    );
    assert_eq!(short_msg, "Type mismatch");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch),
        "Expected TypeMismatch error kind"
    );
}

#[test]
fn test_println_ordering_string_operands() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: string = "hello"
    let y: string = "world"
    println(x < y)
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Ordering operator '<' cannot be used with 'string' type"
    );
    assert_eq!(short_msg, "Type mismatch");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch),
        "Expected TypeMismatch error kind"
    );
}

#[test]
fn test_ordering_string_operands_less_equal() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: string = "hello"
    let y: string = "world"
    let result: bool = x <= y
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Ordering operator '<=' cannot be used with 'string' type"
    );
    assert_eq!(short_msg, "Type mismatch");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch),
        "Expected TypeMismatch error kind"
    );
}

#[test]
fn test_ordering_string_operands_greater_equal() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: string = "hello"
    let y: string = "world"
    let result: bool = x >= y
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Ordering operator '>=' cannot be used with 'string' type"
    );
    assert_eq!(short_msg, "Type mismatch");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch),
        "Expected TypeMismatch error kind"
    );
}

#[test]
fn test_ordering_bool_operands_less_equal() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: bool = true
    let y: bool = false
    let result: bool = x <= y
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Ordering operator '<=' cannot be used with 'bool' type"
    );
    assert_eq!(short_msg, "Type mismatch");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch),
        "Expected TypeMismatch error kind"
    );
}

#[test]
fn test_ordering_bool_operands_greater_equal() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: bool = true
    let y: bool = false
    let result: bool = x >= y
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Ordering operator '>=' cannot be used with 'bool' type"
    );
    assert_eq!(short_msg, "Type mismatch");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch),
        "Expected TypeMismatch error kind"
    );
}

// ============================================================================
// Equality operator type mismatch errors
// ============================================================================

#[test]
fn test_equality_int_vs_string() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let result: bool = 5 == "hello"
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Type mismatch: string literal cannot be assigned to type 'i64'"
    );
    assert_eq!(short_msg, "Type mismatch");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch)
    );
}

#[test]
fn test_equality_bool_vs_int() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: bool = true
    let y: i32 = 5
    let result: bool = x != y
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Type mismatch: variable 'y' has type 'i32', expected 'bool'"
    );
    assert_eq!(short_msg, "Type mismatch");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch)
    );
}

// ============================================================================
// println with comparison operator errors
// ============================================================================

#[test]
fn test_println_ordering_bool_operands() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: bool = true
    let y: bool = false
    println(x < y)
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Ordering operator '<' cannot be used with 'bool' type");
    assert_eq!(short_msg, "Type mismatch");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch)
    );
}

#[test]
fn test_println_arithmetic_string_operand() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    println("a" + 1)
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Operator '+' cannot be used with 'string' type");
    assert_eq!(short_msg, "Type mismatch");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch)
    );
}

#[test]
fn test_println_arithmetic_bool_operand() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let b: bool = true
    println(b + 1)
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Operator '+' cannot be used with 'bool' type");
    assert_eq!(short_msg, "Type mismatch");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch)
    );
}

#[test]
fn test_println_arithmetic_mixed_i32_i64_variables() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let a: i32 = 1
    let b: i64 = 2
    println(a + b)
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Type mismatch: variable 'b' has type 'i64', expected 'i32'"
    );
    assert_eq!(short_msg, "Type mismatch");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch)
    );
}

#[test]
fn test_println_comparison_mixed_i32_i64_variables() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let a: i32 = 1
    let b: i64 = 2
    println(a > b)
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Type mismatch: variable 'b' has type 'i64', expected 'i32'"
    );
    assert_eq!(short_msg, "Type mismatch");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch)
    );
}

// ============================================================================
// Logical operator type mismatch errors
// ============================================================================

#[test]
fn test_logical_assigned_to_non_bool() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let result: i32 = true && false
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Logical operator '&&' produces 'bool', but expected 'i32'"
    );
    assert_eq!(short_msg, "Type mismatch");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch)
    );
}

#[test]
fn test_logical_non_bool_operands() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let result: bool = 1 || 2
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Operator '||' cannot be used with 'i64' type");
    assert_eq!(short_msg, "Type mismatch");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch)
    );
}

#[test]
fn test_unary_not_non_bool_operand() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let result: bool = !1
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Unary operator '!' cannot be used with 'i64' type");
    assert_eq!(short_msg, "Type mismatch");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::TypeMismatch)
    );
}
