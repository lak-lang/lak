//! Unit tests for parsing.
//!
//! Tests are organized by parser component:
//! - [`fn_def`]: Function definition parsing and spans
//! - [`stmt`]: Statement parsing (let, expression statements)
//! - [`expr`]: Expression parsing (calls, literals, identifiers)
//! - [`errors`]: Error detection and message quality
//! - [`helpers`]: Parser utilities and edge cases

use super::*;
use crate::ast::{BinaryOperator, Expr, ExprKind, StmtKind, Type, UnaryOperator};
use crate::lexer::Lexer;
use crate::token::Span;

mod errors;
mod expr;
mod fn_def;
mod helpers;
mod import;
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
    assert_eq!(err.message(), "Expected newline after statement, found '+'");
    assert_eq!(err.span().line, 5);
}

#[test]
fn test_parse_error_unexpected_token_constructor() {
    let err = ParseError::unexpected_token("identifier", "'{'", span_at(3, 7));
    assert_eq!(err.kind(), ParseErrorKind::UnexpectedToken);
    assert_eq!(err.message(), "Expected identifier, found '{'");
}

#[test]
fn test_parse_error_expected_identifier_constructor() {
    let err = ParseError::expected_identifier("'123'", span_at(2, 5));
    assert_eq!(err.kind(), ParseErrorKind::ExpectedIdentifier);
    assert_eq!(err.message(), "Expected identifier, found '123'");
}

#[test]
fn test_parse_error_unknown_type_constructor() {
    let err = ParseError::unknown_type("int", span_at(1, 15));
    assert_eq!(err.kind(), ParseErrorKind::ExpectedType);
    assert_eq!(
        err.message(),
        "Unknown type: 'int'. Expected 'i32', 'i64', or 'string'"
    );
}

#[test]
fn test_parse_error_missing_fn_call_parens_string_constructor() {
    let err = ParseError::missing_fn_call_parens_string("println", span_at(4, 8));
    assert_eq!(err.kind(), ParseErrorKind::MissingFunctionCallParentheses);
    assert_eq!(
        err.message(),
        "Unexpected string literal after 'println'. If this is a function call, add parentheses: println(...)"
    );
}

#[test]
fn test_parse_error_missing_fn_call_parens_int_constructor() {
    let err = ParseError::missing_fn_call_parens_int("foo", dummy_span());
    assert_eq!(err.kind(), ParseErrorKind::MissingFunctionCallParentheses);
    assert_eq!(
        err.message(),
        "Unexpected integer literal after 'foo'. If this is a function call, add parentheses: foo(...)"
    );
}

#[test]
fn test_parse_error_missing_fn_call_parens_ident_constructor() {
    let err = ParseError::missing_fn_call_parens_ident("bar", "x", dummy_span());
    assert_eq!(err.kind(), ParseErrorKind::MissingFunctionCallParentheses);
    assert_eq!(
        err.message(),
        "Unexpected identifier 'x' after 'bar'. If this is a function call, add parentheses: bar(...)"
    );
}

#[test]
fn test_parse_error_unexpected_expression_start_constructor() {
    let err = ParseError::unexpected_expression_start("'}'", span_at(10, 1));
    assert_eq!(err.kind(), ParseErrorKind::UnexpectedToken);
    assert_eq!(err.message(), "Unexpected token: '}'");
}

#[test]
fn test_parse_error_display() {
    let err = ParseError::unexpected_token("'('", "'}'", span_at(7, 12));
    let display = format!("{}", err);
    assert_eq!(display, "7:12: Expected '(', found '}'");
}

// ============================================================================
// ParseError short_message tests
// ============================================================================

#[test]
fn test_parse_error_short_message_missing_statement_terminator() {
    let err = ParseError::missing_statement_terminator("'+'", dummy_span());
    assert_eq!(err.short_message(), "Missing statement terminator");
}

#[test]
fn test_parse_error_short_message_unexpected_token() {
    let err = ParseError::unexpected_token("identifier", "'{'", dummy_span());
    assert_eq!(err.short_message(), "Unexpected token");
}

#[test]
fn test_parse_error_short_message_expected_identifier() {
    let err = ParseError::expected_identifier("'123'", dummy_span());
    assert_eq!(err.short_message(), "Expected identifier");
}

#[test]
fn test_parse_error_short_message_expected_type() {
    let err = ParseError::unknown_type("int", dummy_span());
    assert_eq!(err.short_message(), "Unknown type");
}

#[test]
fn test_parse_error_short_message_missing_function_call_parentheses() {
    let err = ParseError::missing_fn_call_parens_string("println", dummy_span());
    assert_eq!(err.short_message(), "Missing function call parentheses");
}

#[test]
fn test_parse_error_short_message_internal_error() {
    let err = ParseError::internal_binary_op_inconsistency(dummy_span());
    assert_eq!(err.short_message(), "Internal error");
}
