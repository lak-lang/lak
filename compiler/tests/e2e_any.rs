//! End-to-end tests for any type support in println.
//!
//! These tests verify that println accepts values of any type (string, i32, i64).
//!
//! Note on integer literals:
//! - Integer literals passed directly to println are always treated as i64.
//! - To print an i32, assign the literal to an i32 variable first.
//!
//! Note on negative integers:
//! - Negative integer literals (e.g., `-42`) are not yet supported by the lexer.
//! - Once supported, negative values will work through variables.

mod common;

use common::compile_and_run;

#[test]
fn test_println_string_literal() {
    let output = compile_and_run(
        r#"fn main() -> void {
    println("direct string literal")
}"#,
    )
    .unwrap();
    assert_eq!(output, "direct string literal\n");
}

#[test]
fn test_println_integer_literal() {
    let output = compile_and_run(
        r#"fn main() -> void {
    println(42)
}"#,
    )
    .unwrap();
    assert_eq!(output, "42\n");
}

#[test]
fn test_println_large_integer_literal() {
    let output = compile_and_run(
        r#"fn main() -> void {
    println(9223372036854775807)
}"#,
    )
    .unwrap();
    assert_eq!(output, "9223372036854775807\n");
}

#[test]
fn test_println_i32_variable() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = 100
    println(x)
}"#,
    )
    .unwrap();
    assert_eq!(output, "100\n");
}

#[test]
fn test_println_i64_variable() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let y: i64 = 200
    println(y)
}"#,
    )
    .unwrap();
    assert_eq!(output, "200\n");
}

#[test]
fn test_println_string_variable() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let s: string = "test"
    println(s)
}"#,
    )
    .unwrap();
    assert_eq!(output, "test\n");
}

#[test]
fn test_println_mixed_types() {
    let output = compile_and_run(
        r#"fn main() -> void {
    println("Hello, World!")
    println(42)

    let x: i32 = 100
    let y: i64 = 200
    let s: string = "test"

    println(x)
    println(y)
    println(s)
}"#,
    )
    .unwrap();
    assert_eq!(output, "Hello, World!\n42\n100\n200\ntest\n");
}

#[test]
fn test_println_i32_max_value() {
    // Note: negative literals are not yet supported
    let output = compile_and_run(
        r#"fn main() -> void {
    let max: i32 = 2147483647
    println(max)
}"#,
    )
    .unwrap();
    assert_eq!(output, "2147483647\n");
}

#[test]
fn test_println_zero() {
    let output = compile_and_run(
        r#"fn main() -> void {
    println(0)
    let x: i32 = 0
    println(x)
    let y: i64 = 0
    println(y)
}"#,
    )
    .unwrap();
    assert_eq!(output, "0\n0\n0\n");
}
