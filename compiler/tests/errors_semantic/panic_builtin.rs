use crate::helpers::assert_semantic_error;
use lak::semantic::SemanticErrorKind;

// ========================================
// panic() built-in function error tests
// ========================================

#[test]
fn test_compile_error_panic_no_args() {
    assert_semantic_error(
        r#"fn main() -> void { panic() }"#,
        "panic expects exactly 1 argument",
        "Invalid argument",
        SemanticErrorKind::InvalidArgument,
    );
}

#[test]
fn test_compile_error_panic_multiple_args() {
    assert_semantic_error(
        r#"fn main() -> void { panic("a", "b") }"#,
        "panic expects exactly 1 argument",
        "Invalid argument",
        SemanticErrorKind::InvalidArgument,
    );
}

#[test]
fn test_compile_error_panic_int_arg() {
    assert_semantic_error(
        r#"fn main() -> void { panic(42) }"#,
        "panic requires a string argument, but got 'integer literal'",
        "Invalid argument",
        SemanticErrorKind::InvalidArgument,
    );
}

#[test]
fn test_compile_error_panic_i32_variable() {
    assert_semantic_error(
        r#"fn main() -> void {
    let x: i32 = 42
    panic(x)
}"#,
        "panic requires a string argument, but got 'i32'",
        "Invalid argument",
        SemanticErrorKind::InvalidArgument,
    );
}

#[test]
fn test_compile_error_panic_i64_variable() {
    assert_semantic_error(
        r#"fn main() -> void {
    let x: i64 = 42
    panic(x)
}"#,
        "panic requires a string argument, but got 'i64'",
        "Invalid argument",
        SemanticErrorKind::InvalidArgument,
    );
}

#[test]
fn test_compile_error_panic_undefined_variable() {
    assert_semantic_error(
        r#"fn main() -> void { panic(unknown) }"#,
        "Undefined variable: 'unknown'",
        "Undefined variable",
        SemanticErrorKind::UndefinedVariable,
    );
}

#[test]
fn test_compile_error_panic_binary_op() {
    assert_semantic_error(
        r#"fn main() -> void {
    panic(1 + 2)
}"#,
        "panic requires a string argument, but got 'expression'",
        "Invalid argument",
        SemanticErrorKind::InvalidArgument,
    );
}

#[test]
fn test_compile_error_panic_function_call() {
    assert_semantic_error(
        r#"fn get_msg() -> void {}
fn main() -> void {
    panic(get_msg())
}"#,
        "Function call 'get_msg' returns 'void' and cannot be used as a value",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}
