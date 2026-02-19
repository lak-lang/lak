//! End-to-end tests for if/else statements.

mod common;

use common::compile_and_run;

#[test]
fn test_if_then_branch() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    if true {
        println("then")
    }
}
"#,
    )
    .unwrap();
    assert_eq!(output, "then\n");
}

#[test]
fn test_if_else_branch() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    if false {
        println("then")
    } else {
        println("else")
    }
}
"#,
    )
    .unwrap();
    assert_eq!(output, "else\n");
}

#[test]
fn test_else_if_chain() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    if false {
        println("a")
    } else if false {
        println("b")
    } else if true {
        println("c")
    } else {
        println("d")
    }
}
"#,
    )
    .unwrap();
    assert_eq!(output, "c\n");
}

#[test]
fn test_if_condition_with_logical_expression() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    if true && !false {
        println("ok")
    } else {
        println("ng")
    }
}
"#,
    )
    .unwrap();
    assert_eq!(output, "ok\n");
}

#[test]
fn test_variables_in_if_branch_do_not_leak_scope() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let cond: bool = true
    if cond {
        let x: i32 = 1
        println(x)
    }
    println(cond)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "1\ntrue\n");
}

#[test]
fn test_shadowing_inside_if_branch() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    let x: i32 = 1
    if true {
        let x: i32 = 2
        println(x)
    }
    println(x)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "2\n1\n");
}

#[test]
fn test_non_void_function_return_inside_if_true() {
    let output = compile_and_run(
        r#"
fn helper() -> i64 {
    if true {
        return 1
    }
}

fn main() -> void {
    println(helper())
}
"#,
    )
    .unwrap();
    assert_eq!(output, "1\n");
}

#[test]
fn test_non_void_function_return_inside_if_not_false() {
    let output = compile_and_run(
        r#"
fn helper() -> i64 {
    if !false {
        return 1
    }
}

fn main() -> void {
    println(helper())
}
"#,
    )
    .unwrap();
    assert_eq!(output, "1\n");
}

#[test]
fn test_non_void_function_return_inside_if_false_else() {
    let output = compile_and_run(
        r#"
fn helper() -> i64 {
    if false {
        return 0
    } else {
        return 1
    }
}

fn main() -> void {
    println(helper())
}
"#,
    )
    .unwrap();
    assert_eq!(output, "1\n");
}
