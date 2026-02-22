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
fn test_function_call_with_single_parameter() {
    let output = compile_and_run(
        r#"
fn greet(name: string) -> void {
    println(name)
}

fn main() -> void {
    greet("lak")
}
"#,
    )
    .unwrap();
    assert_eq!(output, "lak\n");
}

#[test]
fn test_function_call_with_multiple_parameters() {
    let output = compile_and_run(
        r#"
fn show(name: string, age: i32, ok: bool) -> void {
    println(name)
    println(age)
    println(ok)
}

fn main() -> void {
    show("alice", 20, true)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "alice\n20\ntrue\n");
}

#[test]
fn test_user_function_name_lak_println_does_not_override_builtin() {
    let output = compile_and_run(
        r#"
fn lak_println() -> void {}

fn main() -> void {
    println("hello")
    lak_println()
}
"#,
    )
    .unwrap();
    assert_eq!(output, "hello\n");
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

#[test]
fn test_non_void_function_return_i32() {
    let output = compile_and_run(
        r#"
fn add(a: i32, b: i32) -> i32 {
    return a + b
}

fn main() -> void {
    let result: i32 = add(2, 3)
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "5\n");
}

#[test]
fn test_non_void_function_return_bool() {
    let output = compile_and_run(
        r#"
fn is_positive(x: i32) -> bool {
    if x > 0 {
        return true
    } else {
        return false
    }
}

fn main() -> void {
    let ok: bool = is_positive(10)
    println(ok)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "true\n");
}

#[test]
fn test_non_void_function_early_return() {
    let output = compile_and_run(
        r#"
fn abs_or_inc(x: i32) -> i32 {
    if x < 0 {
        return -x
    }
    return x + 1
}

fn main() -> void {
    println(abs_or_inc(-3))
    println(abs_or_inc(4))
}
"#,
    )
    .unwrap();
    assert_eq!(output, "3\n5\n");
}

#[test]
fn test_non_void_function_return_i64() {
    let output = compile_and_run(
        r#"
fn add_big(a: i64, b: i64) -> i64 {
    return a + b
}

fn main() -> void {
    let result: i64 = add_big(9000000000, 10)
    println(result)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "9000000010\n");
}

#[test]
fn test_non_void_function_return_remaining_integer_types() {
    let output = compile_and_run(
        r#"
fn ret_i8() -> i8 {
    return -8
}

fn ret_i16() -> i16 {
    return -16
}

fn ret_u8() -> u8 {
    return 200
}

fn ret_u16() -> u16 {
    return 50000
}

fn ret_u32() -> u32 {
    return 4000000000
}

fn ret_u64() -> u64 {
    return 18446744073709551615
}

fn ret_byte() -> byte {
    return 255
}

fn main() -> void {
    let a: i8 = ret_i8()
    let b: i16 = ret_i16()
    let c: u8 = ret_u8()
    let d: u16 = ret_u16()
    let e: u32 = ret_u32()
    let f: u64 = ret_u64()
    let g: byte = ret_byte()
    println(a)
    println(b)
    println(c)
    println(d)
    println(e)
    println(f)
    println(g)
}
"#,
    )
    .unwrap();
    assert_eq!(
        output,
        "-8\n-16\n200\n50000\n4000000000\n18446744073709551615\n255\n"
    );
}

#[test]
fn test_non_void_function_return_string() {
    let output = compile_and_run(
        r#"
fn greet() -> string {
    return "hello from return"
}

fn main() -> void {
    let msg: string = greet()
    println(msg)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "hello from return\n");
}

#[test]
fn test_println_with_function_call_result() {
    let output = compile_and_run(
        r#"
fn add(a: i32, b: i32) -> i32 {
    return a + b
}

fn main() -> void {
    println(add(2, 3))
}
"#,
    )
    .unwrap();
    assert_eq!(output, "5\n");
}

#[test]
fn test_discard_return_value_with_underscore() {
    let output = compile_and_run(
        r#"
fn side_effect() -> i32 {
    println("from side effect")
    return 1
}

fn main() -> void {
    let _ = side_effect()
    println("done")
}
"#,
    )
    .unwrap();
    assert_eq!(output, "from side effect\ndone\n");
}

#[test]
fn test_early_bare_return_in_main() {
    let output = compile_and_run(
        r#"
fn main() -> void {
    println("before")
    return
    println("after")
}
"#,
    )
    .unwrap();
    assert_eq!(output, "before\n");
}

#[test]
fn test_early_bare_return_in_void_function() {
    let output = compile_and_run(
        r#"
fn maybe_print(flag: bool) -> void {
    if !flag {
        return
    }
    println("hit")
}

fn main() -> void {
    maybe_print(false)
    maybe_print(true)
}
"#,
    )
    .unwrap();
    assert_eq!(output, "hit\n");
}
