use crate::helpers::assert_semantic_error;
use lak::semantic::SemanticErrorKind;

#[test]
fn test_compile_error_non_void_function_missing_return_on_some_path() {
    assert_semantic_error(
        r#"fn maybe(flag: bool) -> i32 {
    if flag {
        return 1
    }
}

fn main() -> void {}"#,
        "Function 'maybe' with return type 'i32' must return a value on all code paths",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_compile_error_return_without_value_in_non_void_function() {
    assert_semantic_error(
        r#"fn answer() -> i32 {
    return
}

fn main() -> void {}"#,
        "return statement requires a value of type 'i32'",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_compile_error_return_value_type_mismatch() {
    assert_semantic_error(
        r#"fn answer() -> i32 {
    return "hello"
}

fn main() -> void {}"#,
        "Type mismatch: return expression has type 'string', expected 'i32'",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_compile_error_return_binary_expression_type_mismatch() {
    assert_semantic_error(
        r#"fn add(a: i32, b: i32) -> string {
    return a + b
}

fn main() -> void {}"#,
        "Type mismatch: return expression has type 'i32', expected 'string'",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_compile_error_return_value_in_void_function() {
    assert_semantic_error(
        r#"fn helper() -> void {
    return 1
}

fn main() -> void {}"#,
        "void function cannot return a value",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_compile_error_invalid_non_void_return_type_annotation() {
    assert_semantic_error(
        r#"fn helper() -> int {
    return 1
}

fn main() -> void {}"#,
        "Unsupported function return type 'int'. Expected 'void', 'i8', 'i16', 'i32', 'i64', 'u8', 'u16', 'u32', 'u64', 'f32', 'f64', 'byte', 'string', or 'bool'",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_compile_error_non_void_function_call_as_statement() {
    assert_semantic_error(
        r#"fn answer() -> i32 {
    return 42
}

fn main() -> void {
    answer()
}"#,
        "Function 'answer' returns 'i32', but only void functions can be called as statements",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_compile_error_void_function_call_as_value() {
    assert_semantic_error(
        r#"fn helper() -> void {}

fn main() -> void {
    let x: i32 = helper()
}"#,
        "Function call 'helper' returns 'void' and cannot be used as a value",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_compile_error_function_call_return_type_mismatch_at_call_site() {
    assert_semantic_error(
        r#"fn add(a: i32, b: i32) -> i32 {
    return a + b
}

fn main() -> void {
    let x: i64 = add(2, 3)
}"#,
        "Type mismatch: function 'add' returns 'i32', expected 'i64'",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_compile_error_discard_void_function_call() {
    assert_semantic_error(
        r#"fn helper() -> void {}

fn main() -> void {
    let _ = helper()
}"#,
        "Function call 'helper' returns 'void' and cannot be used as a value",
        "Type mismatch",
        SemanticErrorKind::TypeMismatch,
    );
}

#[test]
fn test_compile_error_discard_non_call_expression() {
    assert_semantic_error(
        r#"fn main() -> void {
    let _ = 1
}"#,
        "Discard statement must discard a function call result",
        "Invalid argument",
        SemanticErrorKind::InvalidArgument,
    );
}
