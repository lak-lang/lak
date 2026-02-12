//! Parse error tests.
//!
//! Tests for:
//! - Error detection and message quality
//! - Function definition errors
//! - Call expression errors
//! - Unexpected token errors

use super::*;
use crate::parser::ParseErrorKind;

// ===================
// Top-level errors
// ===================

#[test]
fn test_error_top_level_statement() {
    let err = parse_error(r#"println("hello")"#);
    assert_eq!(
        err.message(),
        "Expected 'fn' keyword, found identifier 'println'"
    );
    assert_eq!(err.kind(), ParseErrorKind::UnexpectedToken);
}

// ===================
// Function definition errors
// ===================

#[test]
fn test_error_missing_function_name() {
    let err = parse_error("fn () -> void {}");
    assert_eq!(err.kind(), ParseErrorKind::ExpectedIdentifier);
    assert_eq!(err.message(), "Expected identifier, found '('");
}

#[test]
fn test_error_missing_arrow() {
    let err = parse_error("fn main() void {}");
    assert_eq!(err.kind(), ParseErrorKind::UnexpectedToken);
    assert_eq!(err.message(), "Expected '->', found identifier 'void'");
}

#[test]
fn test_error_missing_return_type() {
    let err = parse_error("fn main() -> {}");
    assert_eq!(err.kind(), ParseErrorKind::ExpectedIdentifier);
    assert_eq!(err.message(), "Expected identifier, found '{'");
}

#[test]
fn test_error_missing_left_brace() {
    let err = parse_error("fn main() -> void }");
    assert_eq!(err.message(), "Expected '{', found '}'");
    assert_eq!(err.kind(), ParseErrorKind::UnexpectedToken);
}

#[test]
fn test_error_missing_right_brace() {
    let err = parse_error("fn main() -> void {");
    assert_eq!(err.message(), "Expected '}', found end of file");
    assert_eq!(err.kind(), ParseErrorKind::UnexpectedToken);
}

#[test]
fn test_error_missing_left_paren_in_fn_def() {
    // Function definition without left parenthesis after name
    let err = parse_error("fn main -> void {}");
    assert_eq!(err.message(), "Expected '(', found '->'");
    assert_eq!(err.kind(), ParseErrorKind::UnexpectedToken);
}

#[test]
fn test_error_function_param_missing_type_annotation() {
    let err = parse_error("fn main(x i32) -> void {}");
    assert_eq!(err.message(), "Expected ':', found identifier 'i32'");
    assert_eq!(err.kind(), ParseErrorKind::UnexpectedToken);
}

#[test]
fn test_error_function_param_expected_name_or_right_paren() {
    let err = parse_error("fn main( -> void {}");
    assert_eq!(err.message(), "Expected parameter name or ')', found '->'");
    assert_eq!(err.kind(), ParseErrorKind::UnexpectedToken);
}

// ===================
// Call expression errors
// ===================

#[test]
fn test_error_missing_right_paren_in_call() {
    let err = parse_error(r#"fn main() -> void { func("a" }"#);
    assert_eq!(err.message(), "Expected ')', found '}'");
    assert_eq!(err.kind(), ParseErrorKind::UnexpectedToken);
}

#[test]
fn test_error_double_comma() {
    let err = parse_error(r#"fn main() -> void { f("a",,"b") }"#);
    assert_eq!(err.kind(), ParseErrorKind::UnexpectedToken);
    assert_eq!(err.message(), "Unexpected token: ','");
}

#[test]
fn test_error_leading_comma() {
    let err = parse_error(r#"fn main() -> void { f(,"a") }"#);
    assert_eq!(err.kind(), ParseErrorKind::UnexpectedToken);
    assert_eq!(err.message(), "Unexpected token: ','");
}

#[test]
fn test_error_trailing_comma() {
    let err = parse_error(r#"fn main() -> void { f("a",) }"#);
    assert_eq!(err.kind(), ParseErrorKind::UnexpectedToken);
    assert_eq!(err.message(), "Unexpected token: ')'");
}

// ===================
// Unexpected token errors
// ===================

#[test]
fn test_error_unexpected_string_after_identifier() {
    let err = parse_error(r#"fn main() -> void { println"hello" }"#);
    assert_eq!(
        err.message(),
        "Unexpected string literal after 'println'. If this is a function call, add parentheses: println(...)"
    );
    assert_eq!(err.kind(), ParseErrorKind::MissingFunctionCallParentheses);
}

#[test]
fn test_error_unexpected_int_after_identifier() {
    let err = parse_error("fn main() -> void { foo 42 }");
    assert_eq!(
        err.message(),
        "Unexpected integer literal after 'foo'. If this is a function call, add parentheses: foo(...)"
    );
    assert_eq!(err.kind(), ParseErrorKind::MissingFunctionCallParentheses);
}

#[test]
fn test_error_unexpected_identifier_after_identifier() {
    let err = parse_error("fn main() -> void { foo bar }");
    assert_eq!(
        err.message(),
        "Unexpected identifier 'bar' after 'foo'. If this is a function call, add parentheses: foo(...)"
    );
    assert_eq!(err.kind(), ParseErrorKind::MissingFunctionCallParentheses);
}

#[test]
fn test_error_message_suggests_parentheses() {
    let err = parse_error(r#"fn main() -> void { println"hello" }"#);
    assert_eq!(
        err.message(),
        "Unexpected string literal after 'println'. If this is a function call, add parentheses: println(...)"
    );
    assert_eq!(err.kind(), ParseErrorKind::MissingFunctionCallParentheses);
}

#[test]
fn test_error_unexpected_string_in_let_init() {
    let err = parse_error(r#"fn main() -> void { let x: i32 = foo"hello" }"#);
    assert_eq!(
        err.message(),
        "Unexpected string literal after 'foo'. If this is a function call, add parentheses: foo(...)"
    );
    assert_eq!(err.kind(), ParseErrorKind::MissingFunctionCallParentheses);
}

#[test]
fn test_error_unexpected_string_in_call_arg() {
    let err = parse_error(r#"fn main() -> void { f(foo"hello") }"#);
    assert_eq!(
        err.message(),
        "Unexpected string literal after 'foo'. If this is a function call, add parentheses: foo(...)"
    );
    assert_eq!(err.kind(), ParseErrorKind::MissingFunctionCallParentheses);
}
