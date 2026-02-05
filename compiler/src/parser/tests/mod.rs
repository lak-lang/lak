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
