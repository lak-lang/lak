//! Unit tests for the semantic analyzer.

use super::*;
use crate::ast::{Expr, ExprKind, FnDef, Program, Stmt, StmtKind, Type, UnaryOperator, Visibility};
use crate::token::Span;

mod function_tests;
mod symbol_table_tests;
mod type_tests;
mod variable_tests;

// ============================================================================
// Test helpers
// ============================================================================

pub fn dummy_span() -> Span {
    Span::new(0, 0, 1, 1)
}

pub fn span_at(line: usize, column: usize) -> Span {
    Span::new(0, 0, line, column)
}

/// Helper to create a minimal valid program with main
pub fn program_with_main(body: Vec<Stmt>) -> Program {
    Program {
        imports: vec![],
        functions: vec![FnDef {
            visibility: Visibility::Private,
            name: "main".to_string(),
            return_type: "void".to_string(),
            return_type_span: dummy_span(),
            body,
            span: dummy_span(),
        }],
    }
}

// ============================================================================
// SemanticError Display tests
// ============================================================================

#[test]
fn test_semantic_error_display_with_span() {
    let err = SemanticError::new(
        SemanticErrorKind::UndefinedVariable,
        "Undefined variable: 'x'",
        span_at(5, 10),
    );
    let display = format!("{}", err);
    assert_eq!(display, "5:10: Undefined variable: 'x'");
}

#[test]
fn test_semantic_error_display_without_span() {
    let err = SemanticError::without_span(
        SemanticErrorKind::MissingMainFunction,
        "No main function found",
    );
    let display = format!("{}", err);
    assert_eq!(display, "No main function found");
}

#[test]
fn test_semantic_error_missing_main_display() {
    let err = SemanticError::missing_main("No main function found in the program");
    let display = format!("{}", err);
    assert_eq!(display, "No main function found in the program");
    assert!(err.span().is_none());
    assert_eq!(err.kind(), SemanticErrorKind::MissingMainFunction);
}

// ============================================================================
// Error constructor tests
// ============================================================================

#[test]
fn test_undefined_variable_constructor() {
    let err = SemanticError::undefined_variable("foo", span_at(10, 5));
    assert_eq!(err.kind(), SemanticErrorKind::UndefinedVariable);
    assert_eq!(err.message(), "Undefined variable: 'foo'");
    assert_eq!(err.span().unwrap().line, 10);
    assert_eq!(err.span().unwrap().column, 5);
}

#[test]
fn test_undefined_function_constructor() {
    let err = SemanticError::undefined_function("bar", span_at(20, 3));
    assert_eq!(err.kind(), SemanticErrorKind::UndefinedFunction);
    assert_eq!(err.message(), "Undefined function: 'bar'");
    assert_eq!(err.span().unwrap().line, 20);
}

#[test]
fn test_duplicate_variable_constructor() {
    let err = SemanticError::duplicate_variable("x", 5, 10, span_at(15, 8));
    assert_eq!(err.kind(), SemanticErrorKind::DuplicateVariable);
    assert_eq!(err.message(), "Variable 'x' is already defined at 5:10");
    assert_eq!(err.span().unwrap().line, 15);
}

#[test]
fn test_duplicate_function_constructor() {
    let err = SemanticError::duplicate_function("helper", 1, 1, span_at(50, 1));
    assert_eq!(err.kind(), SemanticErrorKind::DuplicateFunction);
    assert_eq!(err.message(), "Function 'helper' is already defined at 1:1");
}

#[test]
fn test_type_mismatch_int_to_string_constructor() {
    let err = SemanticError::type_mismatch_int_to_string(42, span_at(3, 5));
    assert_eq!(err.kind(), SemanticErrorKind::TypeMismatch);
    assert_eq!(
        err.message(),
        "Type mismatch: integer literal '42' cannot be assigned to type 'string'"
    );
}

#[test]
fn test_type_mismatch_variable_constructor() {
    let err = SemanticError::type_mismatch_variable("x", "i32", "i64", span_at(7, 1));
    assert_eq!(err.kind(), SemanticErrorKind::TypeMismatch);
    assert_eq!(
        err.message(),
        "Type mismatch: variable 'x' has type 'i32', expected 'i64'"
    );
}

#[test]
fn test_invalid_argument_println_count_constructor() {
    let err = SemanticError::invalid_argument_println_count(span_at(5, 5));
    assert_eq!(err.kind(), SemanticErrorKind::InvalidArgument);
    assert_eq!(err.message(), "println expects exactly 1 argument");
}

#[test]
fn test_invalid_expression_string_literal_constructor() {
    let err = SemanticError::invalid_expression_string_literal(span_at(1, 1));
    assert_eq!(err.kind(), SemanticErrorKind::InvalidExpression);
    assert_eq!(
        err.message(),
        "String literal as a statement has no effect. Did you mean to pass it to a function?"
    );
}

#[test]
fn test_invalid_main_signature_constructor() {
    let err = SemanticError::invalid_main_signature("i32", span_at(1, 20));
    assert_eq!(err.kind(), SemanticErrorKind::InvalidMainSignature);
    assert_eq!(
        err.message(),
        "main function must return void, but found return type 'i32'"
    );
    // InvalidMainSignature has a span (pointing to return type)
    assert!(err.span().is_some());
}

#[test]
fn test_integer_overflow_i32_constructor() {
    let err = SemanticError::integer_overflow_i32(3_000_000_000, span_at(1, 1));
    assert_eq!(err.kind(), SemanticErrorKind::IntegerOverflow);
    assert_eq!(
        err.message(),
        "Integer literal '3000000000' is out of range for i32 (valid range: -2147483648 to 2147483647)"
    );
}

#[test]
fn test_internal_no_scope_constructor() {
    let err = SemanticError::internal_no_scope("x", span_at(1, 1));
    assert_eq!(err.kind(), SemanticErrorKind::InternalError);
    assert_eq!(
        err.message(),
        "Internal error: attempted to define variable 'x' outside a scope. This is a compiler bug."
    );
}
