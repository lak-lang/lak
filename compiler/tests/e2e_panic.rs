//! End-to-end tests for the `panic` built-in function.
//!
//! These tests verify that `panic` correctly terminates the program
//! with exit code 1 and outputs the error message to stderr.

mod common;

use common::lak_binary;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_panic_string_literal() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("panic.lak");

    fs::write(
        &source_path,
        r#"fn main() -> void {
    panic("something went wrong")
}"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .args(["run", source_path.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(!output.status.success(), "panic should cause non-zero exit");
    assert_eq!(
        output.status.code(),
        Some(1),
        "panic should exit with code 1"
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "panic: something went wrong\n"
    );
}

#[test]
fn test_panic_string_variable() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("panic_var.lak");

    fs::write(
        &source_path,
        r#"fn main() -> void {
    let msg: string = "error message"
    panic(msg)
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
        "panic: error message\n"
    );
}

#[test]
fn test_panic_before_println() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("panic_before.lak");

    fs::write(
        &source_path,
        r#"fn main() -> void {
    println("before")
    panic("abort")
    println("after")
}"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .args(["run", source_path.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    // "before" should be printed to stdout
    assert_eq!(String::from_utf8_lossy(&output.stdout), "before\n");
    // panic message should be on stderr
    assert_eq!(String::from_utf8_lossy(&output.stderr), "panic: abort\n");
}

#[test]
fn test_panic_empty_message() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("panic_empty.lak");

    fs::write(
        &source_path,
        r#"fn main() -> void {
    panic("")
}"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .args(["run", source_path.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(String::from_utf8_lossy(&output.stderr), "panic: \n");
}

#[test]
fn test_panic_with_escape_sequences() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("panic_escape.lak");

    fs::write(
        &source_path,
        r#"fn main() -> void {
    panic("line1\nline2\ttab")
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
        "panic: line1\nline2\ttab\n"
    );
}

#[test]
fn test_panic_if_expression_string() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("panic_if_expr.lak");

    fs::write(
        &source_path,
        r#"fn main() -> void {
    let cond: bool = true
    panic(if cond { "left" } else { "right" })
}"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .args(["run", source_path.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(String::from_utf8_lossy(&output.stderr), "panic: left\n");
}
