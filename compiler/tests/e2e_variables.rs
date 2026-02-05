//! Variable and let statement tests for the Lak compiler.
//!
//! These tests verify variable declarations, type annotations,
//! and variable references.

mod common;

use common::compile_and_run;

#[test]
fn test_let_statement_basic() {
    // Variable definition only (no output)
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = 42
}"#,
    )
    .unwrap();
    assert_eq!(output, "");
}

#[test]
fn test_multiple_let_statements() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let a: i32 = 1
    let b: i64 = 2
    let c: i32 = 3
}"#,
    )
    .unwrap();
    assert_eq!(output, "");
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
    println("done")
}"#,
    )
    .unwrap();
    assert_eq!(output, "done\n");
}

#[test]
fn test_let_i64_variable() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let big: i64 = 9223372036854775807
    println("ok")
}"#,
    )
    .unwrap();
    assert_eq!(output, "ok\n");
}

#[test]
fn test_let_chain_references() {
    // Chain of variable references: a -> b -> c
    let output = compile_and_run(
        r#"fn main() -> void {
    let a: i32 = 1
    let b: i32 = a
    let c: i32 = b
    println("chain complete")
}"#,
    )
    .unwrap();
    assert_eq!(output, "chain complete\n");
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
fn test_let_unicode_variable_name() {
    // Unicode variable names should work
    let output = compile_and_run(
        r#"fn main() -> void {
    let 変数: i32 = 42
    let αβγ: i64 = 100
    println("unicode vars ok")
}"#,
    )
    .unwrap();
    assert_eq!(output, "unicode vars ok\n");
}

#[test]
fn test_i32_max_value_valid() {
    // i32::MAX = 2147483647 should be valid for i32
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = 2147483647
    println("ok")
}"#,
    )
    .unwrap();
    assert_eq!(output, "ok\n");
}
