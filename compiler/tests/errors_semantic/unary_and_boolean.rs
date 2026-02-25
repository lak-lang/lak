use crate::helpers::assert_semantic_error;
use lak::semantic::SemanticErrorKind;

#[test]
fn test_compile_error_unary_minus_on_string_variable() {
    assert_semantic_error(
        r#"fn main() -> void {
    let s: string = "hello"
    let x: string = -s
}"#,
        "Unary operator '-' cannot be used with 'string' type",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_compile_error_unary_minus_on_string_variable_in_println() {
    assert_semantic_error(
        r#"fn main() -> void {
    let s: string = "hello"
    println(-s)
}"#,
        "Unary operator '-' cannot be used with 'string' type",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_compile_error_unary_minus_on_string_literal() {
    assert_semantic_error(
        r#"fn main() -> void {
    let x: string = -"hello"
}"#,
        "Unary operator '-' cannot be used with 'string' type",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_compile_error_unary_minus_on_string_literal_in_println() {
    assert_semantic_error(
        r#"fn main() -> void {
    println(-"hello")
}"#,
        "Unary operator '-' cannot be used with 'string' type",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

// ========================================
// Boolean type error tests
// ========================================

#[test]
fn test_compile_error_bool_literal_to_i32() {
    assert_semantic_error(
        r#"fn main() -> void {
    let x: i32 = true
}"#,
        "Type mismatch: boolean literal cannot be assigned to type 'i32'",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_compile_error_bool_literal_to_string() {
    assert_semantic_error(
        r#"fn main() -> void {
    let x: string = false
}"#,
        "Type mismatch: boolean literal cannot be assigned to type 'string'",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_compile_error_int_literal_to_bool() {
    assert_semantic_error(
        r#"fn main() -> void {
    let x: bool = 42
}"#,
        "Type mismatch: integer literal '42' cannot be assigned to type 'bool'",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_compile_error_string_literal_to_bool() {
    assert_semantic_error(
        r#"fn main() -> void {
    let x: bool = "hello"
}"#,
        "Type mismatch: string literal cannot be assigned to type 'bool'",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_compile_error_bool_literal_as_statement() {
    assert_semantic_error(
        r#"fn main() -> void {
    true
}"#,
        "Boolean literal as a statement has no effect. Did you mean to use it in a condition?",
        "Invalid expression",
        SemanticErrorKind::InvalidExpression,
    );
}

#[test]
fn test_compile_error_panic_bool_literal() {
    assert_semantic_error(
        r#"fn main() -> void { panic(true) }"#,
        "panic requires a string argument, but got 'boolean literal'",
        "Invalid argument",
        SemanticErrorKind::InvalidArgument,
    );
}

#[test]
fn test_compile_error_panic_bool_variable() {
    assert_semantic_error(
        r#"fn main() -> void {
    let flag: bool = true
    panic(flag)
}"#,
        "panic requires a string argument, but got 'bool'",
        "Invalid argument",
        SemanticErrorKind::InvalidArgument,
    );
}

#[test]
fn test_compile_error_bool_variable_type_mismatch() {
    assert_semantic_error(
        r#"fn main() -> void {
    let flag: bool = true
    let x: i32 = flag
}"#,
        "Type mismatch: variable 'flag' has type 'bool', expected 'i32'",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

// ========================================
// Boolean operator error tests
// ========================================

#[test]
fn test_compile_error_bool_addition() {
    assert_semantic_error(
        r#"fn main() -> void {
    let x: bool = true + false
}"#,
        "Operator '+' cannot be used with 'bool' type",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_compile_error_bool_subtraction() {
    assert_semantic_error(
        r#"fn main() -> void {
    let x: bool = true - false
}"#,
        "Operator '-' cannot be used with 'bool' type",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_compile_error_bool_multiplication() {
    assert_semantic_error(
        r#"fn main() -> void {
    let x: bool = true * false
}"#,
        "Operator '*' cannot be used with 'bool' type",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_compile_error_bool_division() {
    assert_semantic_error(
        r#"fn main() -> void {
    let x: bool = true / false
}"#,
        "Operator '/' cannot be used with 'bool' type",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_compile_error_bool_modulo() {
    assert_semantic_error(
        r#"fn main() -> void {
    let x: bool = true % false
}"#,
        "Operator '%' cannot be used with 'bool' type",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_compile_error_unary_minus_on_bool_literal() {
    assert_semantic_error(
        r#"fn main() -> void {
    let x: bool = -true
}"#,
        "Unary operator '-' cannot be used with 'bool' type",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_compile_error_unary_minus_on_bool_variable() {
    assert_semantic_error(
        r#"fn main() -> void {
    let flag: bool = true
    let x: bool = -flag
}"#,
        "Unary operator '-' cannot be used with 'bool' type",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_compile_error_unary_minus_on_u32_variable() {
    assert_semantic_error(
        r#"fn main() -> void {
    let value: u32 = 1
    let x: u32 = -value
}"#,
        "Unary operator '-' cannot be used with 'u32' type",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}
