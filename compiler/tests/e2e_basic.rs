//! Basic end-to-end tests for the Lak compiler.
//!
//! These tests verify fundamental functionality like println,
//! comments, and function definitions.

mod common;

use common::compile_and_run;

#[test]
fn test_hello_world() {
    let output = compile_and_run(
        r#"fn main() -> void {
    println("Hello, World!")
}"#,
    )
    .unwrap();
    assert_eq!(output, "Hello, World!\n");
}

#[test]
fn test_multiple_println() {
    let output = compile_and_run(
        r#"fn main() -> void {
    println("first")
    println("second")
    println("third")
}"#,
    )
    .unwrap();
    assert_eq!(output, "first\nsecond\nthird\n");
}

#[test]
fn test_empty_main() {
    // Empty main should compile and run without output
    let output = compile_and_run("fn main() -> void {}").unwrap();
    assert_eq!(output, "");
}

#[test]
fn test_comments_only() {
    let output = compile_and_run(
        r#"// this is a comment
fn main() -> void {
    // another comment
}"#,
    )
    .unwrap();
    assert_eq!(output, "");
}

#[test]
fn test_code_with_comments() {
    let output = compile_and_run(
        r#"// Print greeting
fn main() -> void {
    println("hi") // inline comment
    // end
}"#,
    )
    .unwrap();
    assert_eq!(output, "hi\n");
}

#[test]
fn test_multiple_functions_definition() {
    // Multiple function definitions should compile; only main executes
    let output = compile_and_run(
        r#"fn helper() -> void {}
fn main() -> void {
    println("main executed")
}
fn another() -> void {}"#,
    )
    .unwrap();
    assert_eq!(output, "main executed\n");
}

#[test]
fn test_main_not_first() {
    // main function doesn't need to be first
    let output = compile_and_run(
        r#"fn first() -> void {}
fn second() -> void {}
fn main() -> void {
    println("found main")
}"#,
    )
    .unwrap();
    assert_eq!(output, "found main\n");
}
