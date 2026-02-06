//! Variable and let statement tests for the Lak compiler.
//!
//! These tests verify variable declarations, type annotations,
//! and variable references.

mod common;

use common::compile_and_run;

#[test]
fn test_let_statement_basic() {
    // Variable definition with value verification
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = 42
    println(x)
}"#,
    )
    .unwrap();
    assert_eq!(output, "42\n");
}

#[test]
fn test_multiple_let_statements() {
    // Multiple variables with value verification
    let output = compile_and_run(
        r#"fn main() -> void {
    let a: i32 = 1
    let b: i64 = 2
    let c: i32 = 3
    println(a)
    println(b)
    println(c)
}"#,
    )
    .unwrap();
    assert_eq!(output, "1\n2\n3\n");
}

#[test]
fn test_let_with_println() {
    // println mixed with let statements
    let output = compile_and_run(
        r#"fn main() -> void {
    println("before")
    let x: i32 = 42
    println("after")
}"#,
    )
    .unwrap();
    assert_eq!(output, "before\nafter\n");
}

#[test]
fn test_let_with_variable_reference() {
    // Variable reference to initialize another variable
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = 100
    let y: i32 = x
    println(x)
    println(y)
}"#,
    )
    .unwrap();
    assert_eq!(output, "100\n100\n");
}

#[test]
fn test_let_i64_variable() {
    // i64 MAX value verification
    let output = compile_and_run(
        r#"fn main() -> void {
    let big: i64 = 9223372036854775807
    println(big)
}"#,
    )
    .unwrap();
    assert_eq!(output, "9223372036854775807\n");
}

#[test]
fn test_let_chain_references() {
    // Chain of variable references: a -> b -> c with value verification
    let output = compile_and_run(
        r#"fn main() -> void {
    let a: i32 = 1
    let b: i32 = a
    let c: i32 = b
    println(a)
    println(b)
    println(c)
}"#,
    )
    .unwrap();
    assert_eq!(output, "1\n1\n1\n");
}

#[test]
fn test_let_single_char_name() {
    // Single character variable name
    let output = compile_and_run(
        r#"fn main() -> void {
    let a: i32 = 1
    println("ok")
}"#,
    )
    .unwrap();
    assert_eq!(output, "ok\n");
}

#[test]
fn test_let_underscore_prefix_name() {
    // Variable name starting with underscore
    let output = compile_and_run(
        r#"fn main() -> void {
    let _private: i32 = 42
    println("ok")
}"#,
    )
    .unwrap();
    assert_eq!(output, "ok\n");
}

#[test]
fn test_i32_max_value_valid() {
    // i32::MAX = 2147483647 should be valid for i32 with value verification
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = 2147483647
    println(x)
}"#,
    )
    .unwrap();
    assert_eq!(output, "2147483647\n");
}
