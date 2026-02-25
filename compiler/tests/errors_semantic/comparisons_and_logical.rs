use crate::helpers::assert_semantic_error;
use lak::semantic::SemanticErrorKind;

// ============================================================================
// Comparison operator type errors
// ============================================================================

#[test]
fn test_comparison_string_and_i32_operands() {
    assert_semantic_error(
        r#"fn main() -> void {
    let x: string = "hello"
    let y: i32 = 10
    let result: bool = x < y
}"#,
        "Type mismatch: variable 'y' has type 'i32', expected 'string'",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_comparison_bool_operands() {
    assert_semantic_error(
        r#"fn main() -> void {
    let x: bool = true
    let y: bool = false
    let result: bool = x < y
}"#,
        "Ordering operator '<' cannot be used with 'bool' type",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_comparison_mismatched_types() {
    assert_semantic_error(
        r#"fn main() -> void {
    let x: i32 = 5
    let y: i64 = 10
    let result: bool = x < y
}"#,
        "Type mismatch: variable 'y' has type 'i64', expected 'i32'",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_comparison_assigned_to_non_bool() {
    assert_semantic_error(
        r#"fn main() -> void {
    let result: i32 = 5 < 10
}"#,
        "Comparison operator '<' produces 'bool', but expected 'i32'",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_equality_assigned_to_non_bool() {
    assert_semantic_error(
        r#"fn main() -> void {
    let result: i32 = 5 == 10
}"#,
        "Comparison operator '==' produces 'bool', but expected 'i32'",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_not_equal_assigned_to_non_bool() {
    assert_semantic_error(
        r#"fn main() -> void {
    let result: i32 = 5 != 3
}"#,
        "Comparison operator '!=' produces 'bool', but expected 'i32'",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_greater_than_assigned_to_non_bool() {
    assert_semantic_error(
        r#"fn main() -> void {
    let result: i32 = 5 > 3
}"#,
        "Comparison operator '>' produces 'bool', but expected 'i32'",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_less_equal_assigned_to_non_bool() {
    assert_semantic_error(
        r#"fn main() -> void {
    let result: i32 = 3 <= 5
}"#,
        "Comparison operator '<=' produces 'bool', but expected 'i32'",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_greater_equal_assigned_to_non_bool() {
    assert_semantic_error(
        r#"fn main() -> void {
    let result: i32 = 5 >= 3
}"#,
        "Comparison operator '>=' produces 'bool', but expected 'i32'",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_ordering_string_and_i32_operands_greater() {
    assert_semantic_error(
        r#"fn main() -> void {
    let x: string = "hello"
    let y: i32 = 10
    let result: bool = x > y
}"#,
        "Type mismatch: variable 'y' has type 'i32', expected 'string'",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_ordering_bool_operands_greater() {
    assert_semantic_error(
        r#"fn main() -> void {
    let x: bool = true
    let y: bool = false
    let result: bool = x > y
}"#,
        "Ordering operator '>' cannot be used with 'bool' type",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_println_comparison_type_mismatch() {
    assert_semantic_error(
        r#"fn main() -> void {
    println(5 < "hello")
}"#,
        "Type mismatch: string literal cannot be assigned to type 'i64'",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_println_ordering_string_and_i32_operands() {
    assert_semantic_error(
        r#"fn main() -> void {
    let x: string = "hello"
    let y: i32 = 10
    println(x < y)
}"#,
        "Type mismatch: variable 'y' has type 'i32', expected 'string'",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_ordering_string_and_i32_operands_less_equal() {
    assert_semantic_error(
        r#"fn main() -> void {
    let x: string = "hello"
    let y: i32 = 10
    let result: bool = x <= y
}"#,
        "Type mismatch: variable 'y' has type 'i32', expected 'string'",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_ordering_string_and_i32_operands_greater_equal() {
    assert_semantic_error(
        r#"fn main() -> void {
    let x: string = "hello"
    let y: i32 = 10
    let result: bool = x >= y
}"#,
        "Type mismatch: variable 'y' has type 'i32', expected 'string'",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_ordering_bool_operands_less_equal() {
    assert_semantic_error(
        r#"fn main() -> void {
    let x: bool = true
    let y: bool = false
    let result: bool = x <= y
}"#,
        "Ordering operator '<=' cannot be used with 'bool' type",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_ordering_bool_operands_greater_equal() {
    assert_semantic_error(
        r#"fn main() -> void {
    let x: bool = true
    let y: bool = false
    let result: bool = x >= y
}"#,
        "Ordering operator '>=' cannot be used with 'bool' type",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

// ============================================================================
// Equality operator type mismatch errors
// ============================================================================

#[test]
fn test_equality_int_vs_string() {
    assert_semantic_error(
        r#"fn main() -> void {
    let result: bool = 5 == "hello"
}"#,
        "Type mismatch: string literal cannot be assigned to type 'i64'",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_equality_bool_vs_int() {
    assert_semantic_error(
        r#"fn main() -> void {
    let x: bool = true
    let y: i32 = 5
    let result: bool = x != y
}"#,
        "Type mismatch: variable 'y' has type 'i32', expected 'bool'",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

// ============================================================================
// println with comparison operator errors
// ============================================================================

#[test]
fn test_println_ordering_bool_operands() {
    assert_semantic_error(
        r#"fn main() -> void {
    let x: bool = true
    let y: bool = false
    println(x < y)
}"#,
        "Ordering operator '<' cannot be used with 'bool' type",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_println_arithmetic_string_operand() {
    assert_semantic_error(
        r#"fn main() -> void {
    println("a" + 1)
}"#,
        "Operator '+' cannot be used with 'string' type",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_println_arithmetic_bool_operand() {
    assert_semantic_error(
        r#"fn main() -> void {
    let b: bool = true
    println(b + 1)
}"#,
        "Operator '+' cannot be used with 'bool' type",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_println_arithmetic_mixed_i32_i64_variables() {
    assert_semantic_error(
        r#"fn main() -> void {
    let a: i32 = 1
    let b: i64 = 2
    println(a + b)
}"#,
        "Type mismatch: variable 'b' has type 'i64', expected 'i32'",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_println_comparison_mixed_i32_i64_variables() {
    assert_semantic_error(
        r#"fn main() -> void {
    let a: i32 = 1
    let b: i64 = 2
    println(a > b)
}"#,
        "Type mismatch: variable 'b' has type 'i64', expected 'i32'",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_println_arithmetic_mixed_float_and_integer_variables() {
    assert_semantic_error(
        r#"fn main() -> void {
    let a: f64 = 1.0
    let b: i32 = 2
    println(a + b)
}"#,
        "Type mismatch: variable 'b' has type 'i32', expected 'f64'",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_println_float_modulo_error() {
    assert_semantic_error(
        r#"fn main() -> void {
    let a: f64 = 5.0
    println(a % 2.0)
}"#,
        "Operator '%' cannot be used with 'f64' type",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_println_comparison_mixed_float_and_integer_literal() {
    assert_semantic_error(
        r#"fn main() -> void {
    let a: f32 = 1.0
    println(a < 1)
}"#,
        "Type mismatch: integer literal '1' cannot be assigned to type 'f32'",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

// ============================================================================
// Logical operator type mismatch errors
// ============================================================================

#[test]
fn test_logical_assigned_to_non_bool() {
    assert_semantic_error(
        r#"fn main() -> void {
    let result: i32 = true && false
}"#,
        "Logical operator '&&' produces 'bool', but expected 'i32'",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_logical_non_bool_operands() {
    assert_semantic_error(
        r#"fn main() -> void {
    let result: bool = 1 || 2
}"#,
        "Operator '||' cannot be used with 'i64' type",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_unary_not_non_bool_operand() {
    assert_semantic_error(
        r#"fn main() -> void {
    let result: bool = !1
}"#,
        "Unary operator '!' cannot be used with 'i64' type",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_break_outside_loop_error() {
    assert_semantic_error(
        r#"fn main() -> void {
    break
}"#,
        "break statement can only be used inside a loop",
        "Invalid control flow",
        SemanticErrorKind::InvalidControlFlow,
    );
}

#[test]
fn test_continue_outside_loop_error() {
    assert_semantic_error(
        r#"fn main() -> void {
    continue
}"#,
        "continue statement can only be used inside a loop",
        "Invalid control flow",
        SemanticErrorKind::InvalidControlFlow,
    );
}
