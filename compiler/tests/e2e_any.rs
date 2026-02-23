//! End-to-end tests for any type support in println.
//!
//! These tests verify that println accepts values of any type (string, integer primitives, bool).
//!
//! Note on integer literals:
//! - Integer literals passed directly to println are always treated as i64.
//! - To print an i32, assign the literal to an i32 variable first.
//!
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
fn test_println_f32_variable() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: f32 = 3.5
    println(x)
}"#,
    )
    .unwrap();
    assert_eq!(output, "3.5\n");
}

#[test]
fn test_println_f64_literal() {
    let output = compile_and_run(
        r#"fn main() -> void {
    println(2.25)
}"#,
    )
    .unwrap();
    assert_eq!(output, "2.25\n");
}

#[test]
fn test_println_remaining_integer_types() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let a: i8 = -1
    let b: i16 = -2
    let c: u8 = 3
    let d: u16 = 4
    let e: u32 = 5
    let f: u64 = 6

    println(a)
    println(b)
    println(c)
    println(d)
    println(e)
    println(f)
}"#,
    )
    .unwrap();
    assert_eq!(output, "-1\n-2\n3\n4\n5\n6\n");
}

#[test]
fn test_println_byte_alias() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let b: byte = 255
    println(b)
}"#,
    )
    .unwrap();
    assert_eq!(output, "255\n");
}

#[test]
fn test_println_u64_max_variable() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let max: u64 = 18446744073709551615
    println(max)
}"#,
    )
    .unwrap();
    assert_eq!(output, "18446744073709551615\n");
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

// ========================================
// Negative number tests
// ========================================

#[test]
fn test_println_negative_i32() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = -42
    println(x)
}"#,
    )
    .unwrap();
    assert_eq!(output, "-42\n");
}

#[test]
fn test_println_negative_i64() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let y: i64 = -1000000000
    println(y)
}"#,
    )
    .unwrap();
    assert_eq!(output, "-1000000000\n");
}

#[test]
fn test_println_unary_minus_expression() {
    let output = compile_and_run(
        r#"fn main() -> void {
    println(-(100 + 50))
}"#,
    )
    .unwrap();
    assert_eq!(output, "-150\n");
}

#[test]
fn test_println_negative_literal() {
    let output = compile_and_run(
        r#"fn main() -> void {
    println(-42)
}"#,
    )
    .unwrap();
    assert_eq!(output, "-42\n");
}
