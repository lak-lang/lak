//! Unit tests for parsing.
//!
//! Tests are organized by parser component:
//! - [`fn_def`]: Function definition parsing and spans
//! - [`stmt`]: Statement parsing (let, expression statements)
//! - [`expr`]: Expression parsing (calls, literals, identifiers)
//! - [`errors`]: Error detection and message quality
//! - [`helpers`]: Parser utilities and edge cases

use super::*;
use crate::ast::{Expr, ExprKind, StmtKind, Type};
use crate::lexer::Lexer;
use crate::token::Span;

mod errors;
mod expr;
mod fn_def;
mod helpers;
mod stmt;

/// Helper function to parse input and return the Program.
pub(super) fn parse(input: &str) -> Result<Program, ParseError> {
    let mut lexer = Lexer::new(input);
    let tokens = lexer
        .tokenize()
        .unwrap_or_else(|e| panic!("Lexer failed on parser test input {:?}: {}", input, e));
    let mut parser = Parser::new(tokens);
    parser.parse()
}

/// Helper function to parse a function definition and extract the first expression from its body.
pub(super) fn parse_first_expr(body_code: &str) -> Expr {
    let input = format!("fn test() -> void {{ {} }}", body_code);
    let program =
        parse(&input).unwrap_or_else(|e| panic!("Failed to parse input {:?}: {}", input, e));

    let first_fn = program
        .functions
        .first()
        .unwrap_or_else(|| panic!("Input {:?} produced no functions", input));

    let first_stmt = first_fn
        .body
        .first()
        .unwrap_or_else(|| panic!("Function has no statements"));

    match &first_stmt.kind {
        StmtKind::Expr(expr) => expr.clone(),
        _ => panic!("Expected expression statement"),
    }
}

/// Helper function to parse input and return the error.
pub(super) fn parse_error(input: &str) -> ParseError {
    match parse(input) {
        Ok(program) => panic!(
            "Expected parsing to fail for input {:?}, but it succeeded with {} functions",
            input,
            program.functions.len()
        ),
        Err(e) => e,
    }
}

// ============================================================================
// ParseError constructor tests
// ============================================================================

fn dummy_span() -> Span {
    Span::new(0, 0, 1, 1)
}

fn span_at(line: usize, column: usize) -> Span {
    Span::new(0, 0, line, column)
}

#[test]
fn test_parse_error_missing_statement_terminator_constructor() {
    let err = ParseError::missing_statement_terminator("'+'", span_at(5, 10));
    assert_eq!(err.kind(), ParseErrorKind::MissingStatementTerminator);
    assert!(err.message().contains("newline"));
    assert!(err.message().contains("'+'"));
    assert_eq!(err.span().line, 5);
}

#[test]
fn test_parse_error_unexpected_token_constructor() {
    let err = ParseError::unexpected_token("identifier", "'{'", span_at(3, 7));
    assert_eq!(err.kind(), ParseErrorKind::UnexpectedToken);
    assert!(err.message().contains("identifier"));
    assert!(err.message().contains("'{'"));
}

#[test]
fn test_parse_error_expected_identifier_constructor() {
    let err = ParseError::expected_identifier("'123'", span_at(2, 5));
    assert_eq!(err.kind(), ParseErrorKind::ExpectedIdentifier);
    assert!(err.message().contains("identifier"));
    assert!(err.message().contains("'123'"));
}

#[test]
fn test_parse_error_unknown_type_constructor() {
    let err = ParseError::unknown_type("int", span_at(1, 15));
    assert_eq!(err.kind(), ParseErrorKind::ExpectedType);
    assert!(err.message().contains("int"));
    assert!(err.message().contains("Unknown type"));
}

#[test]
fn test_parse_error_missing_fn_call_parens_string_constructor() {
    let err = ParseError::missing_fn_call_parens_string("println", span_at(4, 8));
    assert_eq!(err.kind(), ParseErrorKind::MissingFunctionCallParentheses);
    assert!(err.message().contains("println"));
    assert!(err.message().contains("add parentheses"));
}

#[test]
fn test_parse_error_missing_fn_call_parens_int_constructor() {
    let err = ParseError::missing_fn_call_parens_int("foo", dummy_span());
    assert_eq!(err.kind(), ParseErrorKind::MissingFunctionCallParentheses);
    assert!(err.message().contains("foo"));
    assert!(err.message().contains("integer literal"));
}

#[test]
fn test_parse_error_missing_fn_call_parens_ident_constructor() {
    let err = ParseError::missing_fn_call_parens_ident("bar", "x", dummy_span());
    assert_eq!(err.kind(), ParseErrorKind::MissingFunctionCallParentheses);
    assert!(err.message().contains("bar"));
    assert!(err.message().contains("x"));
}

#[test]
fn test_parse_error_unexpected_expression_start_constructor() {
    let err = ParseError::unexpected_expression_start("'}'", span_at(10, 1));
    assert_eq!(err.kind(), ParseErrorKind::UnexpectedToken);
    assert!(err.message().contains("'}'"));
    assert!(err.message().contains("Unexpected token"));
}

#[test]
fn test_parse_error_display() {
    let err = ParseError::unexpected_token("'('", "'}'", span_at(7, 12));
    let display = format!("{}", err);
    assert!(display.contains("7:12"));
    assert!(display.contains("'('"));
    assert!(display.contains("'}'"));
}
