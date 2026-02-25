use crate::helpers::assert_semantic_error;
use lak::semantic::SemanticErrorKind;

#[test]
fn test_compile_error_unknown_function() {
    assert_semantic_error(
        r#"fn main() -> void { unknown_func("test") }"#,
        "Undefined function: 'unknown_func'",
        "Undefined function",
        SemanticErrorKind::UndefinedFunction,
    );
}

#[test]
fn test_compile_error_missing_main() {
    assert_semantic_error(
        "",
        "No main function found: program contains no function definitions",
        "Missing main function",
        SemanticErrorKind::MissingMainFunction,
    );
}

#[test]
fn test_compile_error_missing_main_with_other_functions() {
    assert_semantic_error(
        "fn hoge() -> void {}",
        "No main function found. Defined functions: 'hoge'",
        "Missing main function",
        SemanticErrorKind::MissingMainFunction,
    );
}

#[test]
fn test_compile_error_main_wrong_return_type() {
    assert_semantic_error(
        "fn main() -> int {}",
        "main function must return void, but found return type 'int'",
        "Invalid main signature",
        SemanticErrorKind::InvalidMainSignature,
    );
}

#[test]
fn test_compile_error_duplicate_variable() {
    assert_semantic_error(
        r#"fn main() -> void {
    let x: i32 = 1
    let x: i32 = 2
}"#,
        "Variable 'x' is already defined at 2:5",
        "Duplicate variable",
        SemanticErrorKind::DuplicateVariable,
    );
}

#[test]
fn test_compile_error_duplicate_parameter_name() {
    assert_semantic_error(
        r#"fn main() -> void {}
fn dup(a: i32, a: i32) -> void {}"#,
        "Variable 'a' is already defined at 2:8",
        "Duplicate variable",
        SemanticErrorKind::DuplicateVariable,
    );
}

#[test]
fn test_compile_error_undefined_variable() {
    assert_semantic_error(
        r#"fn main() -> void {
    let x: i32 = undefined_var
}"#,
        "Undefined variable: 'undefined_var'",
        "Undefined variable",
        SemanticErrorKind::UndefinedVariable,
    );
}

#[test]
fn test_compile_error_self_referential_let_initializer() {
    assert_semantic_error(
        r#"fn main() -> void {
    let x: i32 = x
}"#,
        "Undefined variable: 'x'",
        "Undefined variable",
        SemanticErrorKind::UndefinedVariable,
    );
}

#[test]
fn test_compile_error_reassignment_to_immutable_variable() {
    assert_semantic_error(
        r#"fn main() -> void {
    let x: i32 = 1
    x = 2
}"#,
        "Cannot reassign immutable variable 'x'",
        "Invalid assignment",
        SemanticErrorKind::ImmutableVariableReassignment,
    );
}

#[test]
fn test_compile_error_reassignment_type_mismatch() {
    assert_semantic_error(
        r#"fn main() -> void {
    let mut x: i32 = 1
    x = "hello"
}"#,
        "Type mismatch: string literal cannot be assigned to type 'i32'",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_compile_error_reassignment_to_undefined_variable() {
    assert_semantic_error(
        r#"fn main() -> void {
    x = 1
}"#,
        "Undefined variable: 'x'",
        "Undefined variable",
        SemanticErrorKind::UndefinedVariable,
    );
}

#[test]
fn test_compile_error_type_mismatch() {
    assert_semantic_error(
        r#"fn main() -> void {
    let x: i32 = 42
    let y: i64 = x
}"#,
        "Type mismatch: variable 'x' has type 'i32', expected 'i64'",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_compile_error_if_condition_must_be_bool() {
    assert_semantic_error(
        r#"fn main() -> void {
    if 1 {
        println("x")
    }
}"#,
        "Type mismatch: integer literal '1' cannot be assigned to type 'bool'",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_compile_error_while_condition_must_be_bool() {
    assert_semantic_error(
        r#"fn main() -> void {
    while 1 {
        println("x")
    }
}"#,
        "Type mismatch: integer literal '1' cannot be assigned to type 'bool'",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_compile_error_if_expression_branch_type_mismatch() {
    assert_semantic_error(
        r#"fn main() -> void {
    let value: i64 = if true { 42 } else { "hello" }
}"#,
        "Type mismatch in if expression: then branch is 'i64', else branch is 'string'",
        "If expression branch type mismatch",
        SemanticErrorKind::IfExpressionBranchTypeMismatch,
    );
}

#[test]
fn test_compile_error_if_expression_type_mismatch_to_expected_type() {
    assert_semantic_error(
        r#"fn main() -> void {
    let value: bool = if true { 42 } else { 24 }
}"#,
        "Type mismatch: if expression has type 'i64', expected 'bool'",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_compile_error_if_expression_branch_i32_overflow_is_preserved() {
    assert_semantic_error(
        r#"fn main() -> void {
    let x: i32 = if true { 2147483648 } else { 1 }
}"#,
        "Integer literal '2147483648' is out of range for i32 (valid range: -2147483648 to 2147483647)",
        "Integer overflow",
        SemanticErrorKind::IntegerOverflow,
    );
}

#[test]
fn test_compile_error_i32_overflow() {
    // i32::MAX + 1 = 2147483648 should overflow i32
    assert_semantic_error(
        r#"fn main() -> void {
    let x: i32 = 2147483648
}"#,
        "Integer literal '2147483648' is out of range for i32 (valid range: -2147483648 to 2147483647)",
        "Integer overflow",
        SemanticErrorKind::IntegerOverflow,
    );
}

#[test]
fn test_compile_error_i32_overflow_in_println_binary_op_with_variable() {
    assert_semantic_error(
        r#"fn main() -> void {
    let x: i32 = 1
    println(2147483648 + x)
}"#,
        "Integer literal '2147483648' is out of range for i32 (valid range: -2147483648 to 2147483647)",
        "Integer overflow",
        SemanticErrorKind::IntegerOverflow,
    );
}

#[test]
fn test_compile_error_i32_overflow_in_typed_binary_op_with_variable() {
    assert_semantic_error(
        r#"fn main() -> void {
    let x: i32 = 1
    let y: i32 = 2147483648 + x
}"#,
        "Integer literal '2147483648' is out of range for i32 (valid range: -2147483648 to 2147483647)",
        "Integer overflow",
        SemanticErrorKind::IntegerOverflow,
    );
}

#[test]
fn test_compile_error_i32_negative_overflow_in_println_binary_op_with_variable() {
    assert_semantic_error(
        r#"fn main() -> void {
    let x: i32 = 1
    println(-2147483649 + x)
}"#,
        "Integer literal '-2147483649' is out of range for i32 (valid range: -2147483648 to 2147483647)",
        "Integer overflow",
        SemanticErrorKind::IntegerOverflow,
    );
}

#[test]
fn test_compile_error_i32_large_value_overflow() {
    // Test that very large values (i64 max) are rejected for i32.
    assert_semantic_error(
        r#"fn main() -> void {
    let x: i32 = 9223372036854775807
}"#,
        "Integer literal '9223372036854775807' is out of range for i32 (valid range: -2147483648 to 2147483647)",
        "Integer overflow",
        SemanticErrorKind::IntegerOverflow,
    );
}

#[test]
fn test_compile_error_large_literal_without_context_defaults_to_i64() {
    assert_semantic_error(
        r#"fn main() -> void {
    println(9223372036854775808)
}"#,
        "Integer literal '9223372036854775808' is out of range for i64 (valid range: -9223372036854775808 to 9223372036854775807)",
        "Integer overflow",
        SemanticErrorKind::IntegerOverflow,
    );
}

#[test]
fn test_compile_error_duplicate_variable_different_type() {
    // Duplicate variable with different type should still be rejected
    assert_semantic_error(
        r#"fn main() -> void {
    let x: i32 = 1
    let x: i64 = 2
}"#,
        "Variable 'x' is already defined at 2:5",
        "Duplicate variable",
        SemanticErrorKind::DuplicateVariable,
    );
}

#[test]
fn test_compile_error_forward_reference() {
    // Variable used before it's defined
    assert_semantic_error(
        r#"fn main() -> void {
    let x: i32 = y
    let y: i32 = 1
}"#,
        "Undefined variable: 'y'",
        "Undefined variable",
        SemanticErrorKind::UndefinedVariable,
    );
}

#[test]
fn test_compile_error_string_literal_to_int() {
    // String literal cannot be assigned to i32 variable
    assert_semantic_error(
        r#"fn main() -> void {
    let x: i32 = "hello"
}"#,
        "Type mismatch: string literal cannot be assigned to type 'i32'",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_compile_error_int_literal_to_string() {
    // Integer literal cannot be assigned to string variable
    assert_semantic_error(
        r#"fn main() -> void {
    let s: string = 42
}"#,
        "Type mismatch: integer literal '42' cannot be assigned to type 'string'",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}
