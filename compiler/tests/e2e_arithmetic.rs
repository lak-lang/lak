//! End-to-end tests for arithmetic operators.
//!
//! These tests verify that arithmetic operations are correctly compiled
//! and executed, including operator precedence, associativity, and
//! parenthesized expressions.

mod common;

use common::{compile_and_run, lak_binary};
use std::fs;
use std::process::Command;
use tempfile::tempdir;

// ============================================================================
// Basic Operations
// ============================================================================

#[test]
fn test_addition() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = 3 + 5
    println(x)
}"#,
    )
    .unwrap();
    assert_eq!(output, "8\n");
}

#[test]
fn test_subtraction() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = 10 - 3
    println(x)
}"#,
    )
    .unwrap();
    assert_eq!(output, "7\n");
}

#[test]
fn test_multiplication() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = 4 * 5
    println(x)
}"#,
    )
    .unwrap();
    assert_eq!(output, "20\n");
}

#[test]
fn test_division() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = 20 / 4
    println(x)
}"#,
    )
    .unwrap();
    assert_eq!(output, "5\n");
}

#[test]
fn test_modulo() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = 17 % 5
    println(x)
}"#,
    )
    .unwrap();
    assert_eq!(output, "2\n");
}

// ============================================================================
// Operator Precedence
// ============================================================================

#[test]
fn test_precedence_mul_before_add() {
    // 2 + 3 * 4 should be 2 + (3 * 4) = 2 + 12 = 14, not (2 + 3) * 4 = 20
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = 2 + 3 * 4
    println(x)
}"#,
    )
    .unwrap();
    assert_eq!(output, "14\n");
}

#[test]
fn test_precedence_div_before_sub() {
    // 10 - 6 / 2 should be 10 - (6 / 2) = 10 - 3 = 7, not (10 - 6) / 2 = 2
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = 10 - 6 / 2
    println(x)
}"#,
    )
    .unwrap();
    assert_eq!(output, "7\n");
}

#[test]
fn test_precedence_mod_before_add() {
    // 5 + 7 % 3 should be 5 + (7 % 3) = 5 + 1 = 6, not (5 + 7) % 3 = 0
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = 5 + 7 % 3
    println(x)
}"#,
    )
    .unwrap();
    assert_eq!(output, "6\n");
}

#[test]
fn test_precedence_complex() {
    // 1 + 2 * 3 + 4 = 1 + 6 + 4 = 11
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = 1 + 2 * 3 + 4
    println(x)
}"#,
    )
    .unwrap();
    assert_eq!(output, "11\n");
}

// ============================================================================
// Associativity (Left-to-Right)
// ============================================================================

#[test]
fn test_left_associative_sub() {
    // 10 - 5 - 2 should be (10 - 5) - 2 = 3, not 10 - (5 - 2) = 7
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = 10 - 5 - 2
    println(x)
}"#,
    )
    .unwrap();
    assert_eq!(output, "3\n");
}

#[test]
fn test_left_associative_div() {
    // 100 / 10 / 2 should be (100 / 10) / 2 = 5, not 100 / (10 / 2) = 20
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = 100 / 10 / 2
    println(x)
}"#,
    )
    .unwrap();
    assert_eq!(output, "5\n");
}

#[test]
fn test_left_associative_mod() {
    // 100 % 30 % 7 should be (100 % 30) % 7 = 10 % 7 = 3
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = 100 % 30 % 7
    println(x)
}"#,
    )
    .unwrap();
    assert_eq!(output, "3\n");
}

// ============================================================================
// Parentheses
// ============================================================================

#[test]
fn test_parens_override_precedence() {
    // (2 + 3) * 4 should be 5 * 4 = 20, overriding default precedence
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = (2 + 3) * 4
    println(x)
}"#,
    )
    .unwrap();
    assert_eq!(output, "20\n");
}

#[test]
fn test_nested_parens() {
    // ((1 + 2) * 3) + 4 = (3 * 3) + 4 = 9 + 4 = 13
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = ((1 + 2) * 3) + 4
    println(x)
}"#,
    )
    .unwrap();
    assert_eq!(output, "13\n");
}

#[test]
fn test_parens_both_sides() {
    // (1 + 2) * (3 + 4) = 3 * 7 = 21
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = (1 + 2) * (3 + 4)
    println(x)
}"#,
    )
    .unwrap();
    assert_eq!(output, "21\n");
}

// ============================================================================
// Complex Expressions
// ============================================================================

#[test]
fn test_complex_expression() {
    // (1 + 2) * (3 + 4) - 5 = 3 * 7 - 5 = 21 - 5 = 16
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = (1 + 2) * (3 + 4) - 5
    println(x)
}"#,
    )
    .unwrap();
    assert_eq!(output, "16\n");
}

#[test]
fn test_all_operators() {
    // 10 + 20 - 5 * 2 / 2 % 3 = 10 + 20 - ((5 * 2) / 2) % 3 = 10 + 20 - (10 / 2) % 3 = 10 + 20 - 5 % 3 = 10 + 20 - 2 = 28
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = 10 + 20 - 5 * 2 / 2 % 3
    println(x)
}"#,
    )
    .unwrap();
    assert_eq!(output, "28\n");
}

// ============================================================================
// Variables in Expressions
// ============================================================================

#[test]
fn test_arithmetic_with_variables() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let a: i32 = 10
    let b: i32 = 5
    let sum: i32 = a + b
    let diff: i32 = a - b
    let prod: i32 = a * b
    let quot: i32 = a / b
    let rem: i32 = a % b
    println(sum)
    println(diff)
    println(prod)
    println(quot)
    println(rem)
}"#,
    )
    .unwrap();
    assert_eq!(output, "15\n5\n50\n2\n0\n");
}

#[test]
fn test_variable_in_complex_expression() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = 5
    let y: i32 = 3
    let z: i32 = (x + y) * (x - y)
    println(z)
}"#,
    )
    .unwrap();
    // (5 + 3) * (5 - 3) = 8 * 2 = 16
    assert_eq!(output, "16\n");
}

// ============================================================================
// i64 Arithmetic
// ============================================================================

#[test]
fn test_i64_arithmetic() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i64 = 1000000000 + 1000000000
    println(x)
}"#,
    )
    .unwrap();
    assert_eq!(output, "2000000000\n");
}

#[test]
fn test_i64_large_multiplication() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i64 = 1000000 * 1000000
    println(x)
}"#,
    )
    .unwrap();
    assert_eq!(output, "1000000000000\n");
}

// ============================================================================
// Negative Results
// ============================================================================

#[test]
fn test_negative_result() {
    // 5 - 10 = -5
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = 5 - 10
    println(x)
}"#,
    )
    .unwrap();
    assert_eq!(output, "-5\n");
}

#[test]
fn test_negative_modulo() {
    // -7 % 3 depends on platform, but for signed div: 17 % 5 = 2, then -(that) isn't what we're testing
    // Let's test: 3 - 10 = -7, then -7 % 4 = -3 (signed remainder in LLVM)
    let output = compile_and_run(
        r#"fn main() -> void {
    let a: i32 = 3 - 10
    let b: i32 = a % 4
    println(b)
}"#,
    )
    .unwrap();
    // -7 % 4 = -3 (sign follows dividend in LLVM srem)
    assert_eq!(output, "-3\n");
}

// ============================================================================
// println with Binary Operations
// ============================================================================

#[test]
fn test_println_binary_op_directly() {
    // println can take a binary operation expression
    let output = compile_and_run(
        r#"fn main() -> void {
    println(3 + 5)
}"#,
    )
    .unwrap();
    // Direct binary op in println uses i64 type (integer literals default to i64)
    assert_eq!(output, "8\n");
}

#[test]
fn test_println_binary_op_literal_plus_literal() {
    let output = compile_and_run(
        r#"fn main() -> void {
    println(3 + 4)
}"#,
    )
    .unwrap();
    assert_eq!(output, "7\n");
}

#[test]
fn test_println_complex_expression() {
    let output = compile_and_run(
        r#"fn main() -> void {
    println(2 * 3 + 4)
}"#,
    )
    .unwrap();
    assert_eq!(output, "10\n");
}

#[test]
fn test_println_binary_op_mixed_literal_left_i32() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = 6
    println(1 + x)
}"#,
    )
    .unwrap();
    assert_eq!(output, "7\n");
}

#[test]
fn test_println_binary_op_mixed_literal_right_i32() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = 6
    println(x + 1)
}"#,
    )
    .unwrap();
    assert_eq!(output, "7\n");
}

#[test]
fn test_println_binary_op_mixed_literal_left_i64() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i64 = 6
    println(1 + x)
}"#,
    )
    .unwrap();
    assert_eq!(output, "7\n");
}

#[test]
fn test_println_binary_op_mixed_literal_right_i64() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i64 = 6
    println(x + 1)
}"#,
    )
    .unwrap();
    assert_eq!(output, "7\n");
}

#[test]
fn test_println_binary_op_mixed_negative_literal_left_i32() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = 6
    println(-1 + x)
}"#,
    )
    .unwrap();
    assert_eq!(output, "5\n");
}

#[test]
fn test_println_binary_op_mixed_negative_literal_left_i64() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i64 = 6
    println(-1 + x)
}"#,
    )
    .unwrap();
    assert_eq!(output, "5\n");
}

#[test]
fn test_println_nested_binary_op_literal_adaptation_i32() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = 3
    println((x + 1) * 2)
}"#,
    )
    .unwrap();
    assert_eq!(output, "8\n");
}

#[test]
fn test_println_nested_binary_op_literal_adaptation_i64() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i64 = 3
    println((x + 1) * 2)
}"#,
    )
    .unwrap();
    assert_eq!(output, "8\n");
}

#[test]
fn test_println_binary_op_mixed_literal_subtraction_i32() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = 3
    println(10 - x)
}"#,
    )
    .unwrap();
    assert_eq!(output, "7\n");
}

#[test]
fn test_println_binary_op_mixed_literal_subtraction_i64() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i64 = 3
    println(10 - x)
}"#,
    )
    .unwrap();
    assert_eq!(output, "7\n");
}

#[test]
fn test_println_binary_op_mixed_literal_multiplication_i32() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = 4
    println(3 * x)
}"#,
    )
    .unwrap();
    assert_eq!(output, "12\n");
}

#[test]
fn test_println_binary_op_mixed_literal_multiplication_i64() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i64 = 4
    println(3 * x)
}"#,
    )
    .unwrap();
    assert_eq!(output, "12\n");
}

#[test]
fn test_println_binary_op_mixed_literal_division_i32() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = 4
    println(20 / x)
}"#,
    )
    .unwrap();
    assert_eq!(output, "5\n");
}

#[test]
fn test_println_binary_op_mixed_literal_division_i64() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i64 = 4
    println(20 / x)
}"#,
    )
    .unwrap();
    assert_eq!(output, "5\n");
}

#[test]
fn test_println_binary_op_mixed_literal_modulo_i32() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = 4
    println(22 % x)
}"#,
    )
    .unwrap();
    assert_eq!(output, "2\n");
}

#[test]
fn test_println_binary_op_mixed_literal_modulo_i64() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i64 = 4
    println(22 % x)
}"#,
    )
    .unwrap();
    assert_eq!(output, "2\n");
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_zero_division_result() {
    // 0 / 5 = 0
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = 0 / 5
    println(x)
}"#,
    )
    .unwrap();
    assert_eq!(output, "0\n");
}

#[test]
fn test_identity_operations() {
    // x + 0 = x, x * 1 = x, x - 0 = x
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = 42
    let a: i32 = x + 0
    let b: i32 = x * 1
    let c: i32 = x - 0
    println(a)
    println(b)
    println(c)
}"#,
    )
    .unwrap();
    assert_eq!(output, "42\n42\n42\n");
}

// ============================================================================
// Division by Zero Runtime Checks
// ============================================================================

#[test]
fn test_division_by_zero_literal() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("div_zero_lit.lak");

    fs::write(
        &source_path,
        r#"fn main() -> void {
    let x: i32 = 10 / 0
    println(x)
}"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .args(["run", source_path.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(!output.status.success(), "division by zero should panic");
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "panic: division by zero\n"
    );
}

#[test]
fn test_division_by_zero_variable() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("div_zero_var.lak");

    fs::write(
        &source_path,
        r#"fn main() -> void {
    let x: i32 = 10
    let y: i32 = 0
    let z: i32 = x / y
    println(z)
}"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .args(["run", source_path.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "panic: division by zero\n"
    );
}

#[test]
fn test_modulo_by_zero_literal() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("mod_zero_lit.lak");

    fs::write(
        &source_path,
        r#"fn main() -> void {
    let x: i32 = 10 % 0
    println(x)
}"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .args(["run", source_path.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(!output.status.success(), "modulo by zero should panic");
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "panic: modulo by zero\n"
    );
}

#[test]
fn test_modulo_by_zero_variable() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("mod_zero_var.lak");

    fs::write(
        &source_path,
        r#"fn main() -> void {
    let a: i32 = 17
    let b: i32 = 5 - 5
    let c: i32 = a % b
    println(c)
}"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .args(["run", source_path.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "panic: modulo by zero\n"
    );
}

#[test]
fn test_division_by_zero_i64() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("div_zero_i64.lak");

    fs::write(
        &source_path,
        r#"fn main() -> void {
    let x: i64 = 1000000000
    let y: i64 = 0
    let z: i64 = x / y
    println(z)
}"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .args(["run", source_path.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "panic: division by zero\n"
    );
}

#[test]
fn test_division_by_nonzero_still_works() {
    // Ensure we didn't break normal division
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = 0
    let y: i32 = 5
    let z: i32 = x / y
    println(z)
}"#,
    )
    .unwrap();
    assert_eq!(output, "0\n");
}

#[test]
fn test_nested_division_zero_check() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("nested_div.lak");

    fs::write(
        &source_path,
        r#"fn main() -> void {
    let x: i32 = 10 + 20 / 0
    println(x)
}"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .args(["run", source_path.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "panic: division by zero\n"
    );
}

#[test]
fn test_division_by_negative() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let divisor: i32 = 0 - 2
    let x: i32 = 10 / divisor
    println(x)
}"#,
    )
    .unwrap();
    assert_eq!(output, "-5\n");
}

#[test]
fn test_chained_division_zero_check() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("chained_div.lak");

    fs::write(
        &source_path,
        r#"fn main() -> void {
    let x: i32 = 100 / 10 / 0
    println(x)
}"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .args(["run", source_path.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "panic: division by zero\n"
    );
}

#[test]
fn test_modulo_by_zero_i64() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("mod_zero_i64.lak");

    fs::write(
        &source_path,
        r#"fn main() -> void {
    let x: i64 = 1000000000
    let y: i64 = 0
    let z: i64 = x % y
    println(z)
}"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .args(["run", source_path.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "panic: modulo by zero\n"
    );
}

// ============================================================================
// Unary Minus
// ============================================================================

#[test]
fn test_unary_minus_literal() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = -5
    println(x)
}"#,
    )
    .unwrap();
    assert_eq!(output, "-5\n");
}

#[test]
fn test_unary_minus_variable() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let a: i32 = 10
    let b: i32 = -a
    println(b)
}"#,
    )
    .unwrap();
    assert_eq!(output, "-10\n");
}

#[test]
fn test_unary_minus_precedence() {
    // -2 * 3 should be (-2) * 3 = -6, not -(2 * 3) = -6 (same result here, try another)
    // -2 + 3 should be (-2) + 3 = 1, not -(2 + 3) = -5
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = -2 + 3
    println(x)
}"#,
    )
    .unwrap();
    assert_eq!(output, "1\n");
}

#[test]
fn test_unary_minus_multiply() {
    // -2 * 3 = (-2) * 3 = -6
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = -2 * 3
    println(x)
}"#,
    )
    .unwrap();
    assert_eq!(output, "-6\n");
}

#[test]
fn test_double_unary_minus() {
    // --5 = -(-5) = 5
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = --5
    println(x)
}"#,
    )
    .unwrap();
    assert_eq!(output, "5\n");
}

#[test]
fn test_unary_minus_with_parens() {
    // -(3 + 2) = -5
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = -(3 + 2)
    println(x)
}"#,
    )
    .unwrap();
    assert_eq!(output, "-5\n");
}

#[test]
fn test_unary_minus_in_println() {
    // println(-5) directly
    let output = compile_and_run(
        r#"fn main() -> void {
    println(-5)
}"#,
    )
    .unwrap();
    assert_eq!(output, "-5\n");
}

#[test]
fn test_unary_minus_expression_in_println() {
    // println(-(3 + 2)) directly
    let output = compile_and_run(
        r#"fn main() -> void {
    println(-(3 + 2))
}"#,
    )
    .unwrap();
    assert_eq!(output, "-5\n");
}

#[test]
fn test_unary_minus_i64() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i64 = -1000000000
    println(x)
}"#,
    )
    .unwrap();
    assert_eq!(output, "-1000000000\n");
}

#[test]
fn test_unary_minus_complex() {
    // -a * -b = (-a) * (-b) = a * b
    let output = compile_and_run(
        r#"fn main() -> void {
    let a: i32 = 3
    let b: i32 = 4
    let c: i32 = -a * -b
    println(c)
}"#,
    )
    .unwrap();
    assert_eq!(output, "12\n");
}

#[test]
fn test_unary_minus_with_division() {
    // -10 / 2 = -5
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = -10 / 2
    println(x)
}"#,
    )
    .unwrap();
    assert_eq!(output, "-5\n");
}

#[test]
fn test_unary_minus_with_modulo() {
    // -10 % 3 = -1 (sign follows dividend in LLVM srem)
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = -10 % 3
    println(x)
}"#,
    )
    .unwrap();
    assert_eq!(output, "-1\n");
}

#[test]
fn test_subtraction_vs_unary_minus() {
    // 5 - -3 should parse as 5 - (-3) = 8
    let output = compile_and_run(
        r#"fn main() -> void {
    let a: i32 = 5
    let b: i32 = 3
    let x: i32 = a - -b
    println(x)
}"#,
    )
    .unwrap();
    assert_eq!(output, "8\n");
}

#[test]
fn test_unary_minus_triple_negation() {
    // ---5 = -(-(-5)) = -5
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = ---5
    println(x)
}"#,
    )
    .unwrap();
    assert_eq!(output, "-5\n");
}

#[test]
fn test_unary_minus_in_function_arg() {
    // println(-42) with unary minus in function argument
    let output = compile_and_run(
        r#"fn main() -> void {
    println(-42)
}"#,
    )
    .unwrap();
    assert_eq!(output, "-42\n");
}

#[test]
fn test_unary_minus_variable_in_function_arg() {
    // println(-x) with variable
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = 100
    println(-x)
}"#,
    )
    .unwrap();
    assert_eq!(output, "-100\n");
}

// ============================================================================
// Integer Overflow Behavior
// ============================================================================

#[test]
fn test_unary_minus_i32_min_overflow() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("neg_i32_min.lak");

    fs::write(
        &source_path,
        r#"fn main() -> void {
    let x: i32 = -2147483647 - 1
    let y: i32 = -x
    println(y)
}"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .args(["run", source_path.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(!output.status.success(), "negation overflow should panic");
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "panic: integer overflow\n"
    );
}

#[test]
fn test_unary_minus_i64_min_overflow() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("neg_i64_min.lak");

    fs::write(
        &source_path,
        r#"fn main() -> void {
    let x: i64 = -9223372036854775808
    let y: i64 = -x
    println(y)
}"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .args(["run", source_path.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(!output.status.success(), "negation overflow should panic");
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "panic: integer overflow\n"
    );
}

#[test]
fn test_addition_overflow_i32() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("add_overflow_i32.lak");

    fs::write(
        &source_path,
        r#"fn main() -> void {
    let x: i32 = 2147483647
    let y: i32 = x + 1
    println(y)
}"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .args(["run", source_path.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(!output.status.success(), "addition overflow should panic");
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "panic: integer overflow\n"
    );
}

#[test]
fn test_addition_overflow_i64() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("add_overflow_i64.lak");

    fs::write(
        &source_path,
        r#"fn main() -> void {
    let x: i64 = 9223372036854775807
    let y: i64 = x + 1
    println(y)
}"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .args(["run", source_path.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(!output.status.success(), "addition overflow should panic");
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "panic: integer overflow\n"
    );
}

#[test]
fn test_subtraction_overflow_i32() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("sub_overflow_i32.lak");

    fs::write(
        &source_path,
        r#"fn main() -> void {
    let x: i32 = -2147483647 - 1
    let y: i32 = x - 1
    println(y)
}"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .args(["run", source_path.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "subtraction overflow should panic"
    );
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "panic: integer overflow\n"
    );
}

#[test]
fn test_subtraction_overflow_i64() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("sub_overflow_i64.lak");

    fs::write(
        &source_path,
        r#"fn main() -> void {
    let x: i64 = -9223372036854775807 - 1
    let y: i64 = x - 1
    println(y)
}"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .args(["run", source_path.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "subtraction overflow should panic"
    );
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "panic: integer overflow\n"
    );
}

#[test]
fn test_multiplication_overflow_i32() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("mul_overflow_i32.lak");

    fs::write(
        &source_path,
        r#"fn main() -> void {
    let x: i32 = 100000
    let y: i32 = x * x
    println(y)
}"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .args(["run", source_path.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "multiplication overflow should panic"
    );
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "panic: integer overflow\n"
    );
}

#[test]
fn test_multiplication_overflow_i64() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("mul_overflow_i64.lak");

    fs::write(
        &source_path,
        r#"fn main() -> void {
    let x: i64 = 10000000000000
    let y: i64 = x * x
    println(y)
}"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .args(["run", source_path.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "multiplication overflow should panic"
    );
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "panic: integer overflow\n"
    );
}

#[test]
fn test_no_overflow_i32_max_safe() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = 2147483646
    let y: i32 = x + 1
    println(y)
}"#,
    )
    .unwrap();
    assert_eq!(output, "2147483647\n");
}

#[test]
fn test_no_overflow_negation_safe() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = 2147483647
    let y: i32 = -x
    println(y)
}"#,
    )
    .unwrap();
    assert_eq!(output, "-2147483647\n");
}

#[test]
fn test_addition_negative_overflow_i32() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("add_neg_overflow_i32.lak");

    fs::write(
        &source_path,
        r#"fn main() -> void {
    let x: i32 = -2147483647 - 1
    let y: i32 = -1
    let z: i32 = x + y
    println(z)
}"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .args(["run", source_path.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "negative addition overflow should panic"
    );
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "panic: integer overflow\n"
    );
}

#[test]
fn test_addition_negative_overflow_i64() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("add_neg_overflow_i64.lak");

    fs::write(
        &source_path,
        r#"fn main() -> void {
    let x: i64 = -9223372036854775807 - 1
    let y: i64 = -1
    let z: i64 = x + y
    println(z)
}"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .args(["run", source_path.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "negative addition overflow should panic"
    );
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "panic: integer overflow\n"
    );
}

#[test]
fn test_no_overflow_i64_max_safe() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i64 = 9223372036854775806
    let y: i64 = x + 1
    println(y)
}"#,
    )
    .unwrap();
    assert_eq!(output, "9223372036854775807\n");
}

#[test]
fn test_chained_overflow_i32() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("chained_overflow_i32.lak");

    fs::write(
        &source_path,
        r#"fn main() -> void {
    let x: i32 = 2147483647
    let y: i32 = x + 1 - 1
    println(y)
}"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .args(["run", source_path.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "chained expression should panic at first overflow"
    );
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "panic: integer overflow\n"
    );
}

#[test]
fn test_i64_min_literal() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i64 = -9223372036854775808
    println(x)
}"#,
    )
    .unwrap();
    assert_eq!(output, "-9223372036854775808\n");
}

#[test]
fn test_i64_min_direct_println() {
    let output = compile_and_run(
        r#"fn main() -> void {
    println(-9223372036854775808)
}"#,
    )
    .unwrap();
    assert_eq!(output, "-9223372036854775808\n");
}

#[test]
fn test_i32_min_literal() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = -2147483648
    println(x)
}"#,
    )
    .unwrap();
    assert_eq!(output, "-2147483648\n");
}

// ============================================================================
// Division / Modulo Overflow (MIN / -1, MIN % -1)
// ============================================================================

#[test]
fn test_division_overflow_i32_min_div_neg_one() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("div_overflow_i32.lak");

    fs::write(
        &source_path,
        r#"fn main() -> void {
    let x: i32 = -2147483647 - 1
    let y: i32 = 0 - 1
    let z: i32 = x / y
    println(z)
}"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .args(["run", source_path.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "i32 MIN / -1 should panic with overflow"
    );
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "panic: integer overflow\n"
    );
}

#[test]
fn test_division_overflow_i64_min_div_neg_one() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("div_overflow_i64.lak");

    fs::write(
        &source_path,
        r#"fn main() -> void {
    let x: i64 = -9223372036854775807 - 1
    let y: i64 = 0 - 1
    let z: i64 = x / y
    println(z)
}"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .args(["run", source_path.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "i64 MIN / -1 should panic with overflow"
    );
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "panic: integer overflow\n"
    );
}

#[test]
fn test_modulo_overflow_i32_min_mod_neg_one() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("mod_overflow_i32.lak");

    fs::write(
        &source_path,
        r#"fn main() -> void {
    let x: i32 = -2147483647 - 1
    let y: i32 = 0 - 1
    let z: i32 = x % y
    println(z)
}"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .args(["run", source_path.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "i32 MIN % -1 should panic with overflow"
    );
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "panic: integer overflow\n"
    );
}

#[test]
fn test_modulo_overflow_i64_min_mod_neg_one() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("mod_overflow_i64.lak");

    fs::write(
        &source_path,
        r#"fn main() -> void {
    let x: i64 = -9223372036854775807 - 1
    let y: i64 = 0 - 1
    let z: i64 = x % y
    println(z)
}"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .args(["run", source_path.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "i64 MIN % -1 should panic with overflow"
    );
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "panic: integer overflow\n"
    );
}

#[test]
fn test_division_no_false_positive_min_div_two() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = -2147483647 - 1
    let y: i32 = x / 2
    println(y)
}"#,
    )
    .unwrap();
    assert_eq!(output, "-1073741824\n");
}

#[test]
fn test_division_no_false_positive_neg_one_safe() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = 10
    let y: i32 = 0 - 1
    let z: i32 = x / y
    println(z)
}"#,
    )
    .unwrap();
    assert_eq!(output, "-10\n");
}

#[test]
fn test_modulo_no_false_positive_min_mod_two() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = -2147483647 - 1
    let y: i32 = x % 2
    println(y)
}"#,
    )
    .unwrap();
    assert_eq!(output, "0\n");
}

#[test]
fn test_modulo_no_false_positive_neg_one_safe() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = 10
    let y: i32 = 0 - 1
    let z: i32 = x % y
    println(z)
}"#,
    )
    .unwrap();
    assert_eq!(output, "0\n");
}

#[test]
fn test_division_no_false_positive_i64_min_div_two() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i64 = -9223372036854775807 - 1
    let y: i64 = x / 2
    println(y)
}"#,
    )
    .unwrap();
    assert_eq!(output, "-4611686018427387904\n");
}

#[test]
fn test_division_no_false_positive_i64_neg_one_safe() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i64 = 10
    let y: i64 = 0 - 1
    let z: i64 = x / y
    println(z)
}"#,
    )
    .unwrap();
    assert_eq!(output, "-10\n");
}

#[test]
fn test_modulo_no_false_positive_i64_min_mod_two() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i64 = -9223372036854775807 - 1
    let y: i64 = x % 2
    println(y)
}"#,
    )
    .unwrap();
    assert_eq!(output, "0\n");
}

#[test]
fn test_modulo_no_false_positive_i64_neg_one_safe() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i64 = 10
    let y: i64 = 0 - 1
    let z: i64 = x % y
    println(z)
}"#,
    )
    .unwrap();
    assert_eq!(output, "0\n");
}
