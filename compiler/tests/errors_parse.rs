//! Parser error tests for the Lak compiler.
//!
//! These tests verify that syntax errors are properly detected
//! and reported during parsing.

mod common;

use common::{CompileErrorKind, CompileStage, compile_error_with_kind};
use lak::parser::ParseErrorKind;

#[test]
fn test_compile_error_top_level_statement() {
    let result = compile_error_with_kind(r#"println("hello")"#);
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Parse),
        "Expected Parse error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(short_msg, "Unexpected token");
    assert_eq!(
        kind,
        CompileErrorKind::Parse(ParseErrorKind::UnexpectedToken),
        "Expected UnexpectedToken error kind"
    );
}

#[test]
fn test_compile_error_unknown_type() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: unknown = 42
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Parse),
        "Expected Parse error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Unknown type: 'unknown'. Expected 'i32', 'i64', or 'string'"
    );
    assert_eq!(short_msg, "Unknown type");
    assert_eq!(
        kind,
        CompileErrorKind::Parse(ParseErrorKind::ExpectedType),
        "Expected ExpectedType error kind"
    );
}

#[test]
fn test_compile_error_let_missing_variable_name() {
    // `let : i32 = 42` - missing variable name should be a parse error
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let : i32 = 42
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Parse),
        "Expected Parse error, got {:?}: {}",
        stage,
        msg
    );
    // Parser expects an identifier after 'let'
    assert_eq!(msg, "Expected identifier, found ':'");
    assert_eq!(short_msg, "Expected identifier");
    assert_eq!(
        kind,
        CompileErrorKind::Parse(ParseErrorKind::ExpectedIdentifier),
        "Expected ExpectedIdentifier error kind"
    );
}

#[test]
fn test_compile_error_missing_statement_terminator() {
    // Two statements on same line without separator should error
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: i32 = 1 let y: i32 = 2
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Parse),
        "Expected Parse error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Expected newline after statement, found 'let' keyword");
    assert_eq!(short_msg, "Missing statement terminator");
    assert_eq!(
        kind,
        CompileErrorKind::Parse(ParseErrorKind::MissingStatementTerminator),
        "Expected MissingStatementTerminator error kind"
    );
}

#[test]
fn test_compile_error_missing_function_call_parens_string() {
    // Missing parens: println "test" instead of println("test")
    let result = compile_error_with_kind(r#"fn main() -> void { println "test" }"#);
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Parse),
        "Expected Parse error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Unexpected string literal after 'println'. If this is a function call, add parentheses: println(...)"
    );
    assert_eq!(short_msg, "Missing function call parentheses");
    assert_eq!(
        kind,
        CompileErrorKind::Parse(ParseErrorKind::MissingFunctionCallParentheses),
        "Expected MissingFunctionCallParentheses error kind"
    );
}

#[test]
fn test_compile_error_missing_function_call_parens_int() {
    // Missing parens: foo 42 instead of foo(42)
    let result = compile_error_with_kind(r#"fn main() -> void { foo 42 }"#);
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Parse),
        "Expected Parse error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Unexpected integer literal after 'foo'. If this is a function call, add parentheses: foo(...)"
    );
    assert_eq!(short_msg, "Missing function call parentheses");
    assert_eq!(
        kind,
        CompileErrorKind::Parse(ParseErrorKind::MissingFunctionCallParentheses),
        "Expected MissingFunctionCallParentheses error kind"
    );
}
