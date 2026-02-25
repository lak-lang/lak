use crate::helpers::assert_semantic_error;
use lak::semantic::SemanticErrorKind;

#[test]
fn test_compile_error_duplicate_function() {
    assert_semantic_error(
        r#"fn main() -> void {}
fn main() -> void {}"#,
        "Function 'main' is already defined at 1:1",
        "Duplicate function",
        SemanticErrorKind::DuplicateFunction,
    );
}

#[test]
fn test_compile_error_reserved_prelude_function_println() {
    assert_semantic_error(
        r#"fn println() -> void {}
fn main() -> void {}"#,
        "Function name 'println' is reserved by the prelude and cannot be redefined",
        "Invalid argument",
        SemanticErrorKind::InvalidArgument,
    );
}

#[test]
fn test_compile_error_reserved_prelude_function_panic() {
    assert_semantic_error(
        r#"fn panic() -> void {}
fn main() -> void {}"#,
        "Function name 'panic' is reserved by the prelude and cannot be redefined",
        "Invalid argument",
        SemanticErrorKind::InvalidArgument,
    );
}

#[test]
fn test_compile_error_function_call_with_args() {
    // Calling a parameterless function with arguments should error
    assert_semantic_error(
        r#"
fn helper() -> void {
    println("test")
}

fn main() -> void {
    helper(42)
}
"#,
        "Function 'helper' expects 0 arguments, but got 1",
        "Invalid argument",
        SemanticErrorKind::InvalidArgument,
    );
}

#[test]
fn test_compile_error_function_call_with_multiple_args() {
    // Calling a parameterless function with multiple arguments
    assert_semantic_error(
        r#"
fn helper() -> void {
    println("test")
}

fn main() -> void {
    helper("a", "b", "c")
}
"#,
        "Function 'helper' expects 0 arguments, but got 3",
        "Invalid argument",
        SemanticErrorKind::InvalidArgument,
    );
}

#[test]
fn test_compile_error_function_call_with_too_few_args_for_parameterized_function() {
    assert_semantic_error(
        r#"
fn helper(name: string, age: i32) -> void {
    println(name)
    println(age)
}

fn main() -> void {
    helper("alice")
}
"#,
        "Function 'helper' expects 2 arguments, but got 1",
        "Invalid argument",
        SemanticErrorKind::InvalidArgument,
    );
}

#[test]
fn test_compile_error_function_call_with_param_type_mismatch() {
    assert_semantic_error(
        r#"
fn helper(name: string) -> void {
    println(name)
}

fn main() -> void {
    helper(42)
}
"#,
        "Type mismatch: integer literal '42' cannot be assigned to type 'string'",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_compile_error_main_function_with_parameters() {
    assert_semantic_error(
        r#"
fn main(x: i32) -> void {
    println(x)
}
"#,
        "main function must not have parameters, but found 1",
        "Invalid main signature",
        SemanticErrorKind::InvalidMainSignature,
    );
}

#[test]
fn test_compile_error_call_main_directly() {
    // Calling main function directly should be an error
    assert_semantic_error(
        r#"
fn helper() -> void {
    main()
}

fn main() -> void {
    println("test")
}
"#,
        "Cannot call 'main' function directly",
        "Invalid argument",
        SemanticErrorKind::InvalidArgument,
    );
}

#[test]
fn test_compile_error_call_main_from_main() {
    // Calling main from main itself should also be an error
    assert_semantic_error(
        r#"
fn main() -> void {
    main()
}
"#,
        "Cannot call 'main' function directly",
        "Invalid argument",
        SemanticErrorKind::InvalidArgument,
    );
}

#[test]
fn test_compile_error_int_variable_to_string() {
    // Int variable cannot be assigned to string variable
    assert_semantic_error(
        r#"fn main() -> void {
    let x: i32 = 42
    let s: string = x
}"#,
        "Type mismatch: variable 'x' has type 'i32', expected 'string'",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_compile_error_string_variable_as_statement() {
    // String variable used as standalone statement should error (no effect)
    assert_semantic_error(
        r#"fn main() -> void {
    let s: string = "hello"
    s
}"#,
        "Variable 's' used as a statement has no effect. Did you mean to use it in an expression?",
        "Invalid expression",
        SemanticErrorKind::InvalidExpression,
    );
}

#[test]
fn test_compile_error_binary_op_type_mismatch_in_operand() {
    // Binary operation with mismatched operand type (i64 variable used in i32 context)
    assert_semantic_error(
        r#"fn main() -> void {
    let a: i64 = 10
    let b: i32 = a + 1
}"#,
        "Type mismatch: variable 'a' has type 'i64', expected 'i32'",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_compile_error_binary_op_to_string_type() {
    // Binary operation result cannot be assigned to string type
    assert_semantic_error(
        r#"fn main() -> void {
    let x: string = 1 + 2
}"#,
        "Operator '+' cannot be used with 'string' type",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_compile_error_binary_op_as_statement() {
    // Binary operation as statement has no effect
    assert_semantic_error(
        r#"fn main() -> void {
    let x: i32 = 5
    x + 10
}"#,
        "This expression computes a value but the result is not used",
        "Invalid expression",
        SemanticErrorKind::InvalidExpression,
    );
}
