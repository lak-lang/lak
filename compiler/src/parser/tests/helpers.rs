//! Parser helper and edge case tests.
//!
//! Tests for:
//! - Parser invariants (panic tests)
//! - Whitespace and newline handling
//! - Unicode edge cases
//! - Span tracking
//! - Trailing newline handling

use super::*;
use crate::parser::ParseErrorKind;
use crate::token::Span;

// ===================
// Panic tests
// ===================

#[test]
#[should_panic(expected = "Token list must not be empty")]
fn test_parser_new_panics_on_empty() {
    Parser::new(vec![]);
}

// ===================
// Edge cases
// ===================

#[test]
fn test_whitespace_in_call() {
    let expr = parse_first_expr("func  (  )");
    match expr.kind {
        ExprKind::Call { callee, args } => {
            assert_eq!(callee, "func");
            assert!(args.is_empty());
        }
        _ => panic!("Expected Call"),
    }
}

#[test]
fn test_newlines_in_call() {
    let expr = parse_first_expr("func(\n\"a\"\n)");
    match expr.kind {
        ExprKind::Call { callee, args } => {
            assert_eq!(callee, "func");
            assert_eq!(args.len(), 1);
        }
        _ => panic!("Expected Call"),
    }
}

#[test]
fn test_parse_error_display() {
    let err = ParseError::new(
        ParseErrorKind::UnexpectedToken,
        "Test error",
        Span::new(0, 1, 2, 3),
    );
    let display = format!("{}", err);
    assert!(display.contains("2:3"));
    assert!(display.contains("Test error"));
}

// ===================
// Span tracking tests
// ===================

#[test]
fn test_expr_span_tracking() {
    let expr = parse_first_expr("x");
    // Identifier 'x' should have a valid span
    assert!(expr.span.start <= expr.span.end);
    assert!(expr.span.line >= 1);
    assert!(expr.span.column >= 1);
}

#[test]
fn test_let_stmt_span_tracking() {
    let program = parse("fn main() -> void { let x: i32 = 42 }").unwrap();
    let stmt = &program.functions[0].body[0];
    // Let statement should have a valid span
    assert!(stmt.span.start < stmt.span.end);
    assert!(stmt.span.line >= 1);
}

// ===================
// Trailing newline handling
// ===================

#[test]
fn test_parse_with_trailing_newline() {
    // Files typically end with a trailing newline
    let source = "fn main() -> void { println(\"hello\") }\n";
    let program = parse(source).unwrap();
    assert_eq!(program.functions.len(), 1);
    assert_eq!(program.functions[0].name, "main");
}

#[test]
fn test_parse_with_multiple_trailing_newlines() {
    // Multiple trailing newlines should also work
    let source = "fn main() -> void {}\n\n\n";
    let program = parse(source).unwrap();
    assert_eq!(program.functions.len(), 1);
}

#[test]
fn test_parse_multiple_functions_with_trailing_newlines() {
    // Multiple functions with newlines between and after
    let source = "fn foo() -> void {}\n\nfn bar() -> void {}\n";
    let program = parse(source).unwrap();
    assert_eq!(program.functions.len(), 2);
    assert_eq!(program.functions[0].name, "foo");
    assert_eq!(program.functions[1].name, "bar");
}

#[test]
fn test_parse_only_newlines() {
    // File containing only newlines should produce empty program
    let source = "\n\n\n";
    let program = parse(source).unwrap();
    assert!(program.functions.is_empty());
}

#[test]
fn test_parse_with_leading_newlines() {
    // Leading newlines before first function
    let source = "\n\nfn main() -> void {}";
    let program = parse(source).unwrap();
    assert_eq!(program.functions.len(), 1);
    assert_eq!(program.functions[0].name, "main");
}
