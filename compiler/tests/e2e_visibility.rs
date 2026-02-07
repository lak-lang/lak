//! End-to-end tests for visibility (pub keyword).
//!
//! These tests verify that `pub fn` declarations are parsed
//! and compile correctly. Note: actual cross-module visibility
//! enforcement is not yet implemented (Phase 3).

mod common;

use common::compile_and_run;

#[test]
fn test_pub_fn_basic() {
    let output = compile_and_run(
        r#"pub fn main() -> void {
    println("public main")
}"#,
    )
    .unwrap();
    assert_eq!(output, "public main\n");
}

#[test]
fn test_pub_fn_with_private_fn() {
    let output = compile_and_run(
        r#"fn helper() -> void {
    println("helper")
}

pub fn main() -> void {
    helper()
}"#,
    )
    .unwrap();
    assert_eq!(output, "helper\n");
}

#[test]
fn test_multiple_pub_fns() {
    let output = compile_and_run(
        r#"pub fn greet() -> void {
    println("greet")
}

pub fn main() -> void {
    greet()
}"#,
    )
    .unwrap();
    assert_eq!(output, "greet\n");
}

#[test]
fn test_mixed_visibility() {
    let output = compile_and_run(
        r#"fn private_fn() -> void {
    println("private")
}

pub fn public_fn() -> void {
    println("public")
}

fn main() -> void {
    private_fn()
    public_fn()
}"#,
    )
    .unwrap();
    assert_eq!(output, "private\npublic\n");
}
