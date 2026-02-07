//! End-to-end tests for boolean type support.
//!
//! Tests boolean literals (true/false), boolean variables, and println output.

mod common;

use common::compile_and_run;

// =============================================================================
// Boolean literals with println
// =============================================================================

#[test]
fn test_println_bool_true() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    println(true)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_println_bool_false() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    println(false)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "false\n");
}

#[test]
fn test_println_bool_both() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    println(true)
    println(false)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\nfalse\n");
}

// =============================================================================
// Boolean variables
// =============================================================================

#[test]
fn test_bool_variable_true() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let flag: bool = true
    println(flag)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_bool_variable_false() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let flag: bool = false
    println(flag)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "false\n");
}

#[test]
fn test_multiple_bool_variables() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let a: bool = true
    let b: bool = false
    let c: bool = true
    println(a)
    println(b)
    println(c)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\nfalse\ntrue\n");
}

// =============================================================================
// Mixed types
// =============================================================================

#[test]
fn test_mixed_bool_and_other_types() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let flag: bool = true
    let count: i32 = 42
    let name: string = "test"
    println(flag)
    println(count)
    println(name)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n42\ntest\n");
}

#[test]
fn test_bool_literal_and_variable_mixed() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    println(true)
    let x: bool = false
    println(x)
    println(false)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\nfalse\nfalse\n");
}
