//! End-to-end tests for if expressions.

mod common;

use common::compile_and_run;

#[test]
fn test_if_expression_basic() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let a: i64 = 1
    let b: i64 = 2
    let max: i64 = if a > b { a } else { b }
    println(max)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "2\n");
}

#[test]
fn test_if_expression_nested() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let x: i64 = 120
    let result: i64 = if x > 0 {
        if x > 100 { 100 } else { x }
    } else {
        0
    }
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "100\n");
}

#[test]
fn test_if_expression_with_branch_statements() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let cond: bool = true
    let value: i64 = if cond {
        println("then")
        let x: i64 = 10
        x
    } else {
        println("else")
        20
    }
    println(value)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "then\n10\n");
}

#[test]
fn test_if_expression_in_println_argument() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    println(if false { "ng" } else { "ok" })
}
"#,
    )
    .unwrap();
    assert_eq!(output, "ok\n");
}

#[test]
fn test_if_expression_i32_literal_branches_in_i32_context() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let x: i32 = if true { 1 } else { 2 }
    println(x)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "1\n");
}

#[test]
fn test_if_expression_i32_arithmetic_branches_in_i32_context() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let x: i32 = if true { 1 + 2 } else { 3 + 4 }
    println(x)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "3\n");
}

#[test]
fn test_if_expression_i32_literal_and_i32_variable_branches_in_i32_context() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let y: i32 = 2
    let x: i32 = if true { 1 } else { y }
    println(x)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "1\n");
}

#[test]
fn test_if_expression_with_branch_local_in_println_argument() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    println(if true {
        let x: i64 = 10
        x
    } else {
        20
    })
}
"#,
    )
    .unwrap();
    assert_eq!(output, "10\n");
}

#[test]
fn test_if_expression_with_branch_local_in_comparison() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let ok: bool = if true {
        let x: i64 = 10
        x
    } else {
        20
    } > 5
    println(ok)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}
