//! End-to-end tests for comparison operators.
//!
//! These tests verify that comparison operations are correctly compiled
//! and executed, including operator precedence, type checking, and println integration.

mod common;

use common::compile_and_run;

// ============================================================================
// Equality Operators
// ============================================================================

#[test]
fn test_equal_true() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let result: bool = 5 == 5
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_equal_false() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let result: bool = 5 == 3
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "false\n");
}

#[test]
fn test_not_equal_true() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let result: bool = 5 != 3
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_not_equal_false() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let result: bool = 5 != 5
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "false\n");
}

// ============================================================================
// Relational Operators
// ============================================================================

#[test]
fn test_less_than_true() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let result: bool = 3 < 5
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_less_than_false() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let result: bool = 5 < 3
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "false\n");
}

#[test]
fn test_less_than_equal_values() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let result: bool = 5 < 5
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "false\n");
}

#[test]
fn test_greater_than_true() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let result: bool = 5 > 3
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_greater_than_false() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let result: bool = 3 > 5
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "false\n");
}

#[test]
fn test_greater_than_equal_values() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let result: bool = 5 > 5
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "false\n");
}

#[test]
fn test_unsigned_greater_than_high_bit_values() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let a: u8 = 200
    let b: u8 = 100
    let result: bool = a > b
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_unsigned_less_than_high_bit_values() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let a: u16 = 50000
    let b: u16 = 30000
    let result: bool = a < b
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "false\n");
}

#[test]
fn test_less_equal_true_less() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let result: bool = 3 <= 5
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_less_equal_true_equal() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let result: bool = 5 <= 5
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_less_equal_false() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let result: bool = 5 <= 3
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "false\n");
}

#[test]
fn test_greater_equal_true_greater() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let result: bool = 5 >= 3
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_greater_equal_true_equal() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let result: bool = 5 >= 5
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_greater_equal_false() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let result: bool = 3 >= 5
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "false\n");
}

#[test]
fn test_float_comparison_less_than() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let result: bool = 1.5 < 2.0
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_float_comparison_mixed_f32_f64() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let a: f32 = 2.5
    let b: f64 = 2.5
    let result: bool = a == b
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

// ============================================================================
// Type Compatibility
// ============================================================================

#[test]
fn test_comparison_with_i32_variables() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let x: i32 = 5
    let y: i32 = 3
    let result: bool = x > y
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_comparison_i32_equal() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let x: i32 = 5
    let y: i32 = 5
    let result: bool = x == y
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_comparison_i32_not_equal() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let x: i32 = 5
    let y: i32 = 3
    let result: bool = x != y
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_comparison_i32_less_than() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let x: i32 = 3
    let y: i32 = 5
    let result: bool = x < y
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_comparison_i32_less_equal() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let x: i32 = 5
    let y: i32 = 5
    let result: bool = x <= y
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_comparison_i32_greater_equal() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let x: i32 = 5
    let y: i32 = 3
    let result: bool = x >= y
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_comparison_with_i64_variables() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let x: i64 = 1000000000000
    let y: i64 = 999999999999
    let result: bool = x > y
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_comparison_i64_equal() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let x: i64 = 1000000000000
    let y: i64 = 1000000000000
    let result: bool = x == y
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_comparison_i64_not_equal() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let x: i64 = 1000000000000
    let y: i64 = 999999999999
    let result: bool = x != y
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_comparison_i64_less_than() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let x: i64 = 999999999999
    let y: i64 = 1000000000000
    let result: bool = x < y
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_comparison_i64_less_equal() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let x: i64 = 1000000000000
    let y: i64 = 1000000000000
    let result: bool = x <= y
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_comparison_i64_greater_equal() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let x: i64 = 1000000000000
    let y: i64 = 999999999999
    let result: bool = x >= y
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_comparison_negative_numbers() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let x: i32 = -5
    let y: i32 = -3
    let result: bool = x < y
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_comparison_negative_greater_than() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let x: i32 = -3
    let y: i32 = -5
    let result: bool = x > y
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_comparison_negative_equal() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let x: i32 = -5
    let y: i32 = -5
    let result: bool = x == y
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_comparison_negative_not_equal() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let x: i32 = -5
    let y: i32 = -3
    let result: bool = x != y
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_comparison_negative_less_equal() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let x: i32 = -5
    let y: i32 = -3
    let result: bool = x <= y
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_comparison_negative_greater_equal() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let x: i32 = -3
    let y: i32 = -5
    let result: bool = x >= y
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_comparison_negative_vs_positive() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let x: i32 = -1
    let y: i32 = 0
    let result: bool = x > y
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "false\n");
}

#[test]
fn test_comparison_positive_vs_negative() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let x: i32 = 0
    let y: i32 = -1
    let result: bool = x >= y
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

// ============================================================================
// Operator Precedence
// ============================================================================

#[test]
fn test_precedence_arithmetic_before_comparison() {
    // 2 + 3 < 4 * 2 should be (2 + 3) < (4 * 2) = 5 < 8 = true
    let output = compile_and_run(
        r#"
fn main() -> void {
    let result: bool = 2 + 3 < 4 * 2
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_precedence_comparison_before_equality() {
    // 1 < 2 == true → (1 < 2) == true → true
    let output = compile_and_run(
        r#"
fn main() -> void {
    let result: bool = 1 < 2 == true
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_precedence_comparison_before_inequality() {
    // 3 > 2 != false → (3 > 2) != false → true
    let output = compile_and_run(
        r#"
fn main() -> void {
    let result: bool = 3 > 2 != false
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

// ============================================================================
// Println Integration
// ============================================================================

#[test]
fn test_println_comparison_directly() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    println(5 > 3)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_println_equality_with_arithmetic() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    println(2 + 3 == 5)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_println_less_than_directly() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    println(3 < 5)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_println_not_equal_directly() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    println(5 != 3)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_println_less_equal_directly() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    println(3 <= 5)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_println_greater_equal_directly() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    println(5 >= 3)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_println_bool_not_equal_directly() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    println(true != false)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_println_string_not_equal_directly() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    println("hello" != "world")
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_println_comparison_with_i32_variable_and_literal_left() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let x: i32 = 6
    println(5 > x)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "false\n");
}

#[test]
fn test_println_comparison_with_i32_variable_and_literal_right() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let x: i32 = 6
    println(x < 10)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_println_comparison_with_i64_variable_and_literal_left() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let x: i64 = 6
    println(5 > x)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "false\n");
}

#[test]
fn test_println_comparison_with_i64_variable_and_literal_right() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let x: i64 = 6
    println(x < 10)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_println_equality_with_i32_variable_and_literal_left() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let x: i32 = 6
    println(6 == x)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_println_inequality_with_i32_variable_and_literal_right() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let x: i32 = 6
    println(x != 10)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_println_equality_with_i64_variable_and_literal_left() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let x: i64 = 6
    println(6 == x)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_println_inequality_with_i64_variable_and_literal_right() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let x: i64 = 6
    println(x != 10)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

// ============================================================================
// Bool Equality
// ============================================================================

#[test]
fn test_bool_equal_true() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let result: bool = true == true
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_bool_equal_false() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let result: bool = true == false
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "false\n");
}

#[test]
fn test_bool_not_equal_true() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let result: bool = true != false
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_bool_not_equal_false() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let result: bool = false != false
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "false\n");
}

#[test]
fn test_bool_equal_with_variables() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let x: bool = true
    let y: bool = true
    let result: bool = x == y
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_println_bool_equality_directly() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    println(true == true)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

// ============================================================================
// String Equality
// ============================================================================

#[test]
fn test_string_equal_true() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let result: bool = "hello" == "hello"
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_string_equal_false() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let result: bool = "hello" == "world"
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "false\n");
}

#[test]
fn test_string_not_equal_true() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let result: bool = "hello" != "world"
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_string_not_equal_false() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let result: bool = "hello" != "hello"
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "false\n");
}

#[test]
fn test_string_equal_with_variables() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let x: string = "hello"
    let y: string = "hello"
    let result: bool = x == y
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_string_not_equal_with_variables() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let x: string = "hello"
    let y: string = "world"
    let result: bool = x != y
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_string_equal_empty_strings() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let result: bool = "" == ""
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_println_string_equality_directly() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    println("hello" == "hello")
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_string_less_than_true() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let result: bool = "apple" < "banana"
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_string_greater_than_true() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let result: bool = "banana" > "apple"
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_string_less_than_false() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let result: bool = "banana" < "apple"
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "false\n");
}

#[test]
fn test_string_greater_than_false() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let result: bool = "apple" > "banana"
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "false\n");
}

#[test]
fn test_string_less_equal_true_equal() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let result: bool = "lak" <= "lak"
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_string_less_equal_true_less() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let result: bool = "apple" <= "banana"
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_string_less_equal_false() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let result: bool = "banana" <= "apple"
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "false\n");
}

#[test]
fn test_string_greater_equal_true_equal() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let result: bool = "lak" >= "lak"
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_string_greater_equal_true_greater() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let result: bool = "banana" >= "apple"
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_string_greater_equal_false() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let result: bool = "apple" >= "banana"
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "false\n");
}

#[test]
fn test_string_less_than_equal_values() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let result: bool = "lak" < "lak"
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "false\n");
}

#[test]
fn test_string_greater_than_equal_values() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let result: bool = "lak" > "lak"
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "false\n");
}

#[test]
fn test_string_ordering_with_variables() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let a: string = "alpha"
    let b: string = "beta"
    println(a < b)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_println_string_ordering_directly() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    println("apple" < "banana")
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_string_ordering_lexicographical_numeric_text() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let x: string = "z"
    let y: string = "10"
    println(x > y)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

// ============================================================================
// Comparison of Comparisons
// ============================================================================

#[test]
fn test_comparison_of_comparisons_equal() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let a: bool = 5 > 3
    let b: bool = 2 < 4
    let result: bool = a == b
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_comparison_of_comparisons_not_equal() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let a: bool = 5 > 3
    let b: bool = 2 > 4
    let result: bool = a != b
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

// ============================================================================
// Zero Comparison
// ============================================================================

#[test]
fn test_comparison_zero_equal() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let result: bool = 0 == 0
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_comparison_zero_less_than() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let x: i32 = 0
    let y: i32 = 1
    let result: bool = x < y
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_comparison_negative_less_than_zero() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let x: i32 = -1
    let y: i32 = 0
    let result: bool = x < y
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_comparison_zero_greater_equal_negative() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let x: i32 = 0
    let y: i32 = -1
    let result: bool = x >= y
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}
