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

// ============================================================================
// String variable tests
// ============================================================================

#[test]
fn test_string_variable_basic() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let s: string = "hello"
    println(s)
}"#,
    )
    .unwrap();
    assert_eq!(output, "hello\n");
}

#[test]
fn test_string_variable_with_escapes() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let msg: string = "hello\nworld"
    println(msg)
}"#,
    )
    .unwrap();
    assert_eq!(output, "hello\nworld\n");
}

#[test]
fn test_multiple_string_variables() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let a: string = "first"
    let b: string = "second"
    println(a)
    println(b)
}"#,
    )
    .unwrap();
    assert_eq!(output, "first\nsecond\n");
}

#[test]
fn test_string_variable_copy() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let original: string = "message"
    let copy: string = original
    println(copy)
}"#,
    )
    .unwrap();
    assert_eq!(output, "message\n");
}

#[test]
fn test_mixed_string_and_int_variables() {
    // Verify both string and integer variables are correctly stored
    let output = compile_and_run(
        r#"fn main() -> void {
    let s: string = "text"
    let n: i32 = 42
    println(s)
    println(n)
}"#,
    )
    .unwrap();
    assert_eq!(output, "text\n42\n");
}

#[test]
fn test_string_variable_empty() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let empty: string = ""
    println(empty)
}"#,
    )
    .unwrap();
    assert_eq!(output, "\n");
}

#[test]
fn test_string_variable_unicode() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let greeting: string = "こんにちは"
    println(greeting)
}"#,
    )
    .unwrap();
    assert_eq!(output, "こんにちは\n");
}

#[test]
fn test_string_literal_very_long() {
    // Test buffer handling with a 5000 character string
    let long_str = "x".repeat(5000);
    let code = format!(
        r#"fn main() -> void {{
    println("{}")
}}"#,
        long_str
    );
    let output = compile_and_run(&code).unwrap();
    assert_eq!(output, format!("{}\n", long_str));
}

#[test]
fn test_string_variable_very_long() {
    // Test string variable with a 5000 character string
    let long_str = "y".repeat(5000);
    let code = format!(
        r#"fn main() -> void {{
    let s: string = "{}"
    println(s)
}}"#,
        long_str
    );
    let output = compile_and_run(&code).unwrap();
    assert_eq!(output, format!("{}\n", long_str));
}

#[test]
fn test_string_variable_emoji() {
    // Test 4-byte UTF-8 characters (emoji)
    let output = compile_and_run(
        r#"fn main() -> void {
    let greeting: string = "Hello 👋 World 🌍"
    println(greeting)
}"#,
    )
    .unwrap();
    assert_eq!(output, "Hello 👋 World 🌍\n");
}

#[test]
fn test_all_escape_sequences() {
    // Test all supported escape sequences together
    let output = compile_and_run(
        r#"fn main() -> void {
    println("tab:\there\nnewline\rcarriage\\backslash\"quote")
}"#,
    )
    .unwrap();
    assert_eq!(output, "tab:\there\nnewline\rcarriage\\backslash\"quote\n");
}

#[test]
fn test_string_variable_copy_independence() {
    // Verify that original and copy are both accessible
    let output = compile_and_run(
        r#"fn main() -> void {
    let original: string = "message"
    let copy: string = original
    println(original)
    println(copy)
}"#,
    )
    .unwrap();
    assert_eq!(output, "message\nmessage\n");
}

#[test]
fn test_multiple_string_variables_reuse() {
    // Test multiple string variables with reuse
    let output = compile_and_run(
        r#"fn main() -> void {
    let a: string = "first"
    let b: string = "second"
    let c: string = "third"
    println(a)
    println(b)
    println(c)
    println(a)
}"#,
    )
    .unwrap();
    assert_eq!(output, "first\nsecond\nthird\nfirst\n");
}
