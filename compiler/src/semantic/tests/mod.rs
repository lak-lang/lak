//! Unit tests for the semantic analyzer.

use super::*;
use crate::ast::{Expr, ExprKind, FnDef, Program, Stmt, StmtKind, Type};
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
        functions: vec![FnDef {
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
    assert!(err.message().contains("foo"));
    assert!(err.message().contains("Undefined variable"));
    assert_eq!(err.span().unwrap().line, 10);
    assert_eq!(err.span().unwrap().column, 5);
}

#[test]
fn test_undefined_function_constructor() {
    let err = SemanticError::undefined_function("bar", span_at(20, 3));
    assert_eq!(err.kind(), SemanticErrorKind::UndefinedFunction);
    assert!(err.message().contains("bar"));
    assert!(err.message().contains("Undefined function"));
    assert_eq!(err.span().unwrap().line, 20);
}

#[test]
fn test_duplicate_variable_constructor() {
    let err = SemanticError::duplicate_variable("x", 5, 10, span_at(15, 8));
    assert_eq!(err.kind(), SemanticErrorKind::DuplicateVariable);
    assert!(err.message().contains("x"));
    assert!(err.message().contains("5:10")); // original location
    assert_eq!(err.span().unwrap().line, 15);
}

#[test]
fn test_duplicate_function_constructor() {
    let err = SemanticError::duplicate_function("helper", 1, 1, span_at(50, 1));
    assert_eq!(err.kind(), SemanticErrorKind::DuplicateFunction);
    assert!(err.message().contains("helper"));
    assert!(err.message().contains("1:1"));
}

#[test]
fn test_type_mismatch_int_to_string_constructor() {
    let err = SemanticError::type_mismatch_int_to_string(42, span_at(3, 5));
    assert_eq!(err.kind(), SemanticErrorKind::TypeMismatch);
    assert!(err.message().contains("42"));
    assert!(err.message().contains("string"));
}

#[test]
fn test_type_mismatch_variable_constructor() {
    let err = SemanticError::type_mismatch_variable("x", "i32", "i64", span_at(7, 1));
    assert_eq!(err.kind(), SemanticErrorKind::TypeMismatch);
    assert!(err.message().contains("x"));
    assert!(err.message().contains("i32"));
    assert!(err.message().contains("i64"));
}

#[test]
fn test_invalid_argument_println_count_constructor() {
    let err = SemanticError::invalid_argument_println_count(span_at(5, 5));
    assert_eq!(err.kind(), SemanticErrorKind::InvalidArgument);
    assert!(err.message().contains("println"));
    assert!(err.message().contains("1 argument"));
}

#[test]
fn test_invalid_expression_string_literal_constructor() {
    let err = SemanticError::invalid_expression_string_literal(span_at(1, 1));
    assert_eq!(err.kind(), SemanticErrorKind::InvalidExpression);
    assert!(err.message().contains("String literal"));
    assert!(err.message().contains("no effect"));
}

#[test]
fn test_invalid_main_signature_constructor() {
    let err = SemanticError::invalid_main_signature("i32", span_at(1, 20));
    assert_eq!(err.kind(), SemanticErrorKind::InvalidMainSignature);
    assert!(err.message().contains("i32"));
    assert!(err.message().contains("void"));
    // InvalidMainSignature has a span (pointing to return type)
    assert!(err.span().is_some());
}

#[test]
fn test_integer_overflow_i32_constructor() {
    let err = SemanticError::integer_overflow_i32(3_000_000_000, span_at(1, 1));
    assert_eq!(err.kind(), SemanticErrorKind::IntegerOverflow);
    assert!(err.message().contains("3000000000"));
    assert!(err.message().contains("i32"));
}

#[test]
fn test_internal_no_scope_constructor() {
    let err = SemanticError::internal_no_scope("x", span_at(1, 1));
    assert_eq!(err.kind(), SemanticErrorKind::InternalError);
    assert!(err.message().contains("compiler bug"));
    assert!(err.message().contains("x"));
}
