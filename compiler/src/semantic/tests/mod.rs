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
