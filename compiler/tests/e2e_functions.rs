//! End-to-end tests for user-defined function calls.
//!
//! These tests verify that user-defined functions can be:
//! - Defined and called
//! - Called in any order (regardless of definition order)
//! - Called multiple times
//! - Called from other functions (nested calls)

mod common;

use common::compile_and_run;

#[test]
fn test_simple_function_call() {
    let output = compile_and_run(
        r#"
fn helper() -> void {
    println("Hello from helper")
}

fn main() -> void {
    helper()
}
"#,
    )
    .unwrap();
    assert_eq!(output, "Hello from helper\n");
}

#[test]
fn test_multiple_function_calls() {
    let output = compile_and_run(
        r#"
fn greet() -> void {
    println("Hello")
}

fn farewell() -> void {
    println("Goodbye")
}

fn main() -> void {
    greet()
    farewell()
    greet()
}
"#,
    )
    .unwrap();
    assert_eq!(output, "Hello\nGoodbye\nHello\n");
}

#[test]
fn test_nested_function_calls() {
    let output = compile_and_run(
        r#"
fn inner() -> void {
    println("inner")
}

fn outer() -> void {
    println("outer start")
    inner()
    println("outer end")
}

fn main() -> void {
    outer()
}
"#,
    )
    .unwrap();
    assert_eq!(output, "outer start\ninner\nouter end\n");
}

#[test]
fn test_function_definition_order_independent() {
    // main defined first, helper defined after - should still work
    let output = compile_and_run(
        r#"
fn main() -> void {
    helper()
}

fn helper() -> void {
    println("Order independent")
}
"#,
    )
    .unwrap();
    assert_eq!(output, "Order independent\n");
}

#[test]
fn test_empty_function() {
    let output = compile_and_run(
        r#"
fn noop() -> void {
}

fn main() -> void {
    noop()
    println("after noop")
}
"#,
    )
    .unwrap();
    assert_eq!(output, "after noop\n");
}

#[test]
fn test_function_with_local_variables() {
    // Verify local variables have correct values via println
    let output = compile_and_run(
        r#"
fn setup() -> void {
    let x: i32 = 42
    println(x)
}

fn main() -> void {
    setup()
    let y: i32 = 100
    println(y)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "42\n100\n");
}

#[test]
fn test_deeply_nested_calls() {
    let output = compile_and_run(
        r#"
fn level3() -> void {
    println("level 3")
}

fn level2() -> void {
    println("level 2 start")
    level3()
    println("level 2 end")
}

fn level1() -> void {
    println("level 1 start")
    level2()
    println("level 1 end")
}

fn main() -> void {
    level1()
}
"#,
    )
    .unwrap();
    assert_eq!(
        output,
        "level 1 start\nlevel 2 start\nlevel 3\nlevel 2 end\nlevel 1 end\n"
    );
}

#[test]
fn test_mutual_calls() {
    // Functions calling each other (not recursive - just different order)
    let output = compile_and_run(
        r#"
fn a() -> void {
    println("a")
}

fn b() -> void {
    println("b")
    a()
}

fn c() -> void {
    println("c")
    b()
}

fn main() -> void {
    c()
    a()
}
"#,
    )
    .unwrap();
    assert_eq!(output, "c\nb\na\na\n");
}

#[test]
fn test_recursive_function_compiles() {
    // Test that recursive function calls compile correctly.
    // We don't actually execute the recursion (which would cause stack overflow),
    // but we verify the compiler handles self-referential function calls.
    let output = compile_and_run(
        r#"
fn recursive() -> void {
    println("tick")
    recursive()
}

fn main() -> void {
    println("compiled successfully")
}
"#,
    )
    .unwrap();
    assert_eq!(output, "compiled successfully\n");
}

#[test]
fn test_variable_isolation_across_calls() {
    // Verify that variables in one function don't interfere with variables
    // in other functions at runtime by printing each x value.
    let output = compile_and_run(
        r#"
fn set_x_to_100() -> void {
    let x: i32 = 100
    println(x)
}

fn set_x_to_200() -> void {
    let x: i32 = 200
    println(x)
}

fn main() -> void {
    let x: i32 = 42
    println(x)
    set_x_to_100()
    set_x_to_200()
    println(x)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "42\n100\n200\n42\n");
}

// ========================================
// Unary minus with functions tests
// ========================================

#[test]
fn test_function_with_negative_local_variables() {
    let output = compile_and_run(
        r#"
fn setup() -> void {
    let x: i32 = -42
    println(x)
}

fn main() -> void {
    setup()
    let y: i32 = -100
    println(y)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "-42\n-100\n");
}

#[test]
fn test_negated_variable_in_function() {
    let output = compile_and_run(
        r#"
fn process() -> void {
    let value: i32 = 50
    let negated: i32 = -value
    println(value)
    println(negated)
}

fn main() -> void {
    process()
}
"#,
    )
    .unwrap();
    assert_eq!(output, "50\n-50\n");
}

#[test]
fn test_variable_isolation_with_negative_values() {
    // Verify positive and negative values are isolated across functions
    let output = compile_and_run(
        r#"
fn set_x_to_neg_100() -> void {
    let x: i32 = -100
    println(x)
}

fn set_x_to_pos_200() -> void {
    let x: i32 = 200
    println(x)
}

fn main() -> void {
    let x: i32 = 42
    println(x)
    set_x_to_neg_100()
    set_x_to_pos_200()
    println(x)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "42\n-100\n200\n42\n");
}
