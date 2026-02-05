//! String and escape sequence tests for the Lak compiler.
//!
//! These tests verify string literals, escape sequences,
//! special characters, and Unicode support.

mod common;

use common::compile_and_run;

#[test]
fn test_escape_sequences() {
    let output = compile_and_run(
        r#"fn main() -> void {
    println("hello\tworld")
}"#,
    )
    .unwrap();
    assert_eq!(output, "hello\tworld\n");
}

#[test]
fn test_escape_newline() {
    let output = compile_and_run(
        r#"fn main() -> void {
    println("line1\nline2")
}"#,
    )
    .unwrap();
    assert_eq!(output, "line1\nline2\n");
}

#[test]
fn test_empty_string() {
    let output = compile_and_run(
        r#"fn main() -> void {
    println("")
}"#,
    )
    .unwrap();
    assert_eq!(output, "\n");
}

#[test]
fn test_escaped_quotes() {
    let output = compile_and_run(
        r#"fn main() -> void {
    println("say \"hello\"")
}"#,
    )
    .unwrap();
    assert_eq!(output, "say \"hello\"\n");
}

#[test]
fn test_escaped_backslash() {
    let output = compile_and_run(
        r#"fn main() -> void {
    println("path\\to\\file")
}"#,
    )
    .unwrap();
    assert_eq!(output, "path\\to\\file\n");
}

#[test]
fn test_special_characters() {
    let output = compile_and_run(
        r#"fn main() -> void {
    println("!@#$%^&*(){}[]|;:'<>,.?/")
}"#,
    )
    .unwrap();
    assert_eq!(output, "!@#$%^&*(){}[]|;:'<>,.?/\n");
}

#[test]
fn test_unicode_string() {
    let output = compile_and_run(
        r#"fn main() -> void {
    println("こんにちは世界")
}"#,
    )
    .unwrap();
    assert_eq!(output, "こんにちは世界\n");
}

#[test]
fn test_mixed_escapes() {
    let output = compile_and_run(
        r#"fn main() -> void {
    println("tab:\there\nnewline")
}"#,
    )
    .unwrap();
    assert_eq!(output, "tab:\there\nnewline\n");
}
