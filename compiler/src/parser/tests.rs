//! Unit tests for parsing.

use super::*;
use crate::ast::{Expr, ExprKind, StmtKind, Type};
use crate::lexer::Lexer;

/// Helper function to parse input and return the Program.
fn parse(input: &str) -> Result<Program, ParseError> {
    let mut lexer = Lexer::new(input);
    let tokens = lexer
        .tokenize()
        .unwrap_or_else(|e| panic!("Lexer failed on parser test input {:?}: {}", input, e));
    let mut parser = Parser::new(tokens);
    parser.parse()
}

/// Helper function to parse a function definition and extract the first expression from its body.
fn parse_first_expr(body_code: &str) -> Expr {
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
fn parse_error(input: &str) -> ParseError {
    match parse(input) {
        Ok(program) => panic!(
            "Expected parsing to fail for input {:?}, but it succeeded with {} functions",
            input,
            program.functions.len()
        ),
        Err(e) => e,
    }
}

// ===================
// Function definition parsing
// ===================

#[test]
fn test_empty_program() {
    let program = parse("").unwrap();
    assert!(program.functions.is_empty());
}

#[test]
fn test_main_function_empty_body() {
    let program = parse("fn main() -> void {}").unwrap();
    assert_eq!(program.functions.len(), 1);
    assert_eq!(program.functions[0].name, "main");
    assert_eq!(program.functions[0].return_type, "void");
    assert!(program.functions[0].body.is_empty());
}

#[test]
fn test_main_function_with_body() {
    let program = parse(r#"fn main() -> void { println("hello") }"#).unwrap();
    assert_eq!(program.functions.len(), 1);
    assert_eq!(program.functions[0].name, "main");
    assert_eq!(program.functions[0].body.len(), 1);
}

#[test]
fn test_multiple_functions() {
    let program = parse("fn foo() -> void {}\nfn bar() -> void {}").unwrap();
    assert_eq!(program.functions.len(), 2);
    assert_eq!(program.functions[0].name, "foo");
    assert_eq!(program.functions[1].name, "bar");
}

#[test]
fn test_function_with_multiple_statements() {
    let program = parse(
        r#"fn main() -> void {
            println("first")
            println("second")
        }"#,
    )
    .unwrap();
    assert_eq!(program.functions[0].body.len(), 2);
}

// ===================
// Basic expression parsing (within function body)
// ===================

#[test]
fn test_call_no_args() {
    let expr = parse_first_expr("func()");
    match expr.kind {
        ExprKind::Call { callee, args } => {
            assert_eq!(callee, "func");
            assert!(args.is_empty());
        }
        _ => panic!("Expected Call expression"),
    }
}

#[test]
fn test_call_one_arg() {
    let expr = parse_first_expr(r#"println("hello")"#);
    match expr.kind {
        ExprKind::Call { callee, args } => {
            assert_eq!(callee, "println");
            assert_eq!(args.len(), 1);
            assert!(matches!(&args[0].kind, ExprKind::StringLiteral(s) if s == "hello"));
        }
        _ => panic!("Expected Call expression"),
    }
}

#[test]
fn test_call_multiple_args() {
    let expr = parse_first_expr(r#"f("a", "b", "c")"#);
    match expr.kind {
        ExprKind::Call { callee, args } => {
            assert_eq!(callee, "f");
            assert_eq!(args.len(), 3);
            assert!(matches!(&args[0].kind, ExprKind::StringLiteral(s) if s == "a"));
            assert!(matches!(&args[1].kind, ExprKind::StringLiteral(s) if s == "b"));
            assert!(matches!(&args[2].kind, ExprKind::StringLiteral(s) if s == "c"));
        }
        _ => panic!("Expected Call expression"),
    }
}

// ===================
// Nested calls
// ===================

#[test]
fn test_nested_call_single() {
    let expr = parse_first_expr("outer(inner())");
    match expr.kind {
        ExprKind::Call { callee, args } => {
            assert_eq!(callee, "outer");
            assert_eq!(args.len(), 1);
            match &args[0].kind {
                ExprKind::Call {
                    callee: inner_callee,
                    args: inner_args,
                } => {
                    assert_eq!(inner_callee, "inner");
                    assert!(inner_args.is_empty());
                }
                _ => panic!("Expected nested Call"),
            }
        }
        _ => panic!("Expected Call expression"),
    }
}

#[test]
fn test_nested_call_with_arg() {
    let expr = parse_first_expr(r#"outer(inner("x"))"#);
    match expr.kind {
        ExprKind::Call { callee, args } => {
            assert_eq!(callee, "outer");
            assert_eq!(args.len(), 1);
            match &args[0].kind {
                ExprKind::Call {
                    callee: inner_callee,
                    args: inner_args,
                } => {
                    assert_eq!(inner_callee, "inner");
                    assert_eq!(inner_args.len(), 1);
                    assert!(matches!(&inner_args[0].kind, ExprKind::StringLiteral(s) if s == "x"));
                }
                _ => panic!("Expected nested Call"),
            }
        }
        _ => panic!("Expected Call expression"),
    }
}

#[test]
fn test_deeply_nested() {
    let expr = parse_first_expr("a(b(c(d())))");
    match expr.kind {
        ExprKind::Call { callee, args } => {
            assert_eq!(callee, "a");
            assert_eq!(args.len(), 1);
            // Verify structure: a -> b -> c -> d
            match &args[0].kind {
                ExprKind::Call { callee: b, args } => {
                    assert_eq!(b, "b");
                    match &args[0].kind {
                        ExprKind::Call { callee: c, args } => {
                            assert_eq!(c, "c");
                            match &args[0].kind {
                                ExprKind::Call { callee: d, args } => {
                                    assert_eq!(d, "d");
                                    assert!(args.is_empty());
                                }
                                _ => panic!("Expected d call"),
                            }
                        }
                        _ => panic!("Expected c call"),
                    }
                }
                _ => panic!("Expected b call"),
            }
        }
        _ => panic!("Expected Call expression"),
    }
}

#[test]
fn test_nested_multiple_args() {
    let expr = parse_first_expr(r#"f(g(), h(), "x")"#);
    match expr.kind {
        ExprKind::Call { callee, args } => {
            assert_eq!(callee, "f");
            assert_eq!(args.len(), 3);
            assert!(matches!(&args[0].kind, ExprKind::Call { callee, .. } if callee == "g"));
            assert!(matches!(&args[1].kind, ExprKind::Call { callee, .. } if callee == "h"));
            assert!(matches!(&args[2].kind, ExprKind::StringLiteral(s) if s == "x"));
        }
        _ => panic!("Expected Call expression"),
    }
}

// ===================
// Multiple statements in function body
// ===================

#[test]
fn test_multiple_statements_in_body_with_newline() {
    let program = parse("fn main() -> void { f()\ng() }").unwrap();
    assert_eq!(program.functions[0].body.len(), 2);

    match &program.functions[0].body[0].kind {
        StmtKind::Expr(expr) => match &expr.kind {
            ExprKind::Call { callee, .. } => assert_eq!(callee, "f"),
            _ => panic!("Expected f call"),
        },
        _ => panic!("Expected Expr statement"),
    }
    match &program.functions[0].body[1].kind {
        StmtKind::Expr(expr) => match &expr.kind {
            ExprKind::Call { callee, .. } => assert_eq!(callee, "g"),
            _ => panic!("Expected g call"),
        },
        _ => panic!("Expected Expr statement"),
    }
}

#[test]
fn test_error_multiple_statements_same_line() {
    let err = parse_error("fn main() -> void { f() g() }");
    assert!(
        err.message.contains("newline"),
        "Expected error about newline, got: {}",
        err.message
    );
}

#[test]
fn test_error_multiple_functions_same_line() {
    let err = parse_error("fn foo() -> void {}fn bar() -> void {}");
    assert!(
        err.message.contains("newline"),
        "Expected error about newline, got: {}",
        err.message
    );
}

#[test]
fn test_error_multiple_functions_same_line_with_space() {
    // Space between functions but still on same line
    let err = parse_error("fn foo() -> void {} fn bar() -> void {}");
    assert!(
        err.message.contains("newline"),
        "Expected error about newline, got: {}",
        err.message
    );
}

#[test]
fn test_error_let_statements_same_line() {
    let err = parse_error("fn main() -> void { let x: i32 = 1 let y: i32 = 2 }");
    assert!(
        err.message.contains("newline"),
        "Expected error about newline, got: {}",
        err.message
    );
}

#[test]
fn test_error_let_then_expr_same_line() {
    let err = parse_error("fn main() -> void { let x: i32 = 1 f() }");
    assert!(
        err.message.contains("newline"),
        "Expected error about newline, got: {}",
        err.message
    );
}

#[test]
fn test_error_expr_then_let_same_line() {
    let err = parse_error("fn main() -> void { f() let x: i32 = 1 }");
    assert!(
        err.message.contains("newline"),
        "Expected error about newline, got: {}",
        err.message
    );
}

#[test]
fn test_error_same_line_reports_correct_token() {
    // Verify the error message includes what token was found
    let err = parse_error("fn main() -> void { f() g() }");
    assert!(
        err.message.contains("identifier 'g'"),
        "Expected error to mention the found token, got: {}",
        err.message
    );
}

#[test]
fn test_error_same_line_span_points_to_offending_token() {
    // Verify the error span points to the second statement
    let err = parse_error("fn main() -> void { f() g() }");
    // "fn main() -> void { f() g() }"
    //  ^--- column 1
    //                          ^--- column 25 (where 'g' starts)
    assert_eq!(
        err.span.column, 25,
        "Expected error at column 25 (the 'g'), got column {}",
        err.span.column
    );
}

#[test]
fn test_statements_with_comments() {
    let program = parse("fn main() -> void { f() // c\ng() }").unwrap();
    assert_eq!(program.functions[0].body.len(), 2);
}

// ===================
// Expression types
// ===================

#[test]
fn test_string_literal_as_arg() {
    let expr = parse_first_expr(r#"f("str")"#);
    match expr.kind {
        ExprKind::Call { args, .. } => {
            assert!(matches!(&args[0].kind, ExprKind::StringLiteral(s) if s == "str"));
        }
        _ => panic!("Expected Call"),
    }
}

#[test]
fn test_call_as_arg() {
    let expr = parse_first_expr("f(g())");
    match expr.kind {
        ExprKind::Call { args, .. } => {
            assert!(matches!(&args[0].kind, ExprKind::Call { callee, .. } if callee == "g"));
        }
        _ => panic!("Expected Call"),
    }
}

// ===================
// Error cases
// ===================

#[test]
fn test_error_top_level_statement() {
    let err = parse_error(r#"println("hello")"#);
    assert!(
        err.message.contains("'fn' keyword"),
        "Expected error about 'fn' keyword, got: {}",
        err.message
    );
}

#[test]
fn test_error_missing_function_name() {
    let err = parse_error("fn () -> void {}");
    assert!(
        err.message.contains("identifier"),
        "Expected error about 'identifier', got: {}",
        err.message
    );
}

#[test]
fn test_error_missing_arrow() {
    let err = parse_error("fn main() void {}");
    assert!(
        err.message.contains("'->'"),
        "Expected error about '->', got: {}",
        err.message
    );
}

#[test]
fn test_error_missing_return_type() {
    let err = parse_error("fn main() -> {}");
    assert!(
        err.message.contains("identifier"),
        "Expected error about 'identifier', got: {}",
        err.message
    );
}

#[test]
fn test_error_missing_left_brace() {
    let err = parse_error("fn main() -> void }");
    assert!(
        err.message.contains("'{'"),
        "Expected error about '{{', got: {}",
        err.message
    );
}

#[test]
fn test_error_missing_right_brace() {
    let err = parse_error("fn main() -> void {");
    assert!(
        err.message.contains("'}'"),
        "Expected error about '}}', got: {}",
        err.message
    );
}

#[test]
fn test_error_missing_right_paren_in_call() {
    let err = parse_error(r#"fn main() -> void { func("a" }"#);
    assert!(
        err.message.contains("')'"),
        "Expected error about ')', got: {}",
        err.message
    );
}

#[test]
fn test_error_double_comma() {
    let err = parse_error(r#"fn main() -> void { f("a",,"b") }"#);
    assert!(err.message.contains("Unexpected token"));
}

#[test]
fn test_error_leading_comma() {
    let err = parse_error(r#"fn main() -> void { f(,"a") }"#);
    assert!(err.message.contains("Unexpected token"));
}

#[test]
fn test_error_trailing_comma() {
    let err = parse_error(r#"fn main() -> void { f("a",) }"#);
    assert!(err.message.contains("Unexpected token"));
}

#[test]
fn test_error_unexpected_string_after_identifier() {
    let err = parse_error(r#"fn main() -> void { println"hello" }"#);
    assert!(
        err.message.contains("Unexpected string literal"),
        "Expected error about unexpected string literal, got: {}",
        err.message
    );
    assert!(
        err.message.contains("println"),
        "Expected error to mention 'println', got: {}",
        err.message
    );
}

#[test]
fn test_error_unexpected_int_after_identifier() {
    let err = parse_error("fn main() -> void { foo 42 }");
    assert!(
        err.message.contains("Unexpected integer literal"),
        "Expected error about unexpected integer literal, got: {}",
        err.message
    );
    assert!(
        err.message.contains("foo"),
        "Expected error to mention 'foo', got: {}",
        err.message
    );
}

#[test]
fn test_error_unexpected_identifier_after_identifier() {
    let err = parse_error("fn main() -> void { foo bar }");
    assert!(
        err.message.contains("Unexpected identifier"),
        "Expected error about unexpected identifier, got: {}",
        err.message
    );
    assert!(
        err.message.contains("bar"),
        "Expected error to mention 'bar', got: {}",
        err.message
    );
}

#[test]
fn test_error_message_suggests_parentheses() {
    let err = parse_error(r#"fn main() -> void { println"hello" }"#);
    assert!(
        err.message.contains("add parentheses"),
        "Expected helpful suggestion in error message, got: {}",
        err.message
    );
}

#[test]
fn test_error_unexpected_string_in_let_init() {
    let err = parse_error(r#"fn main() -> void { let x: i32 = foo"hello" }"#);
    assert!(
        err.message.contains("Unexpected string literal"),
        "Expected error about unexpected string literal in let init, got: {}",
        err.message
    );
    assert!(
        err.message.contains("foo"),
        "Expected error to mention 'foo', got: {}",
        err.message
    );
}

#[test]
fn test_error_unexpected_string_in_call_arg() {
    let err = parse_error(r#"fn main() -> void { f(foo"hello") }"#);
    assert!(
        err.message.contains("Unexpected string literal"),
        "Expected error about unexpected string literal in call arg, got: {}",
        err.message
    );
    assert!(
        err.message.contains("foo"),
        "Expected error to mention 'foo', got: {}",
        err.message
    );
}

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
    use crate::token::Span;
    let err = ParseError {
        message: "Test error".to_string(),
        span: Span::new(0, 1, 2, 3),
    };
    let display = format!("{}", err);
    assert!(display.contains("2:3"));
    assert!(display.contains("Test error"));
}

// ===================
// Additional coverage tests
// ===================

#[test]
fn test_error_missing_left_paren_in_fn_def() {
    // Function definition without left parenthesis after name
    let err = parse_error("fn main -> void {}");
    assert!(
        err.message.contains("'('"),
        "Expected error about '(', got: {}",
        err.message
    );
}

#[test]
fn test_error_function_with_params() {
    // Function definition with parameters (not supported yet)
    let err = parse_error("fn main(x) -> void {}");
    assert!(
        err.message.contains("')'"),
        "Expected error about ')' (params not supported), got: {}",
        err.message
    );
}

#[test]
fn test_unicode_function_name() {
    let program = parse("fn 挨拶() -> void {}").unwrap();
    assert_eq!(program.functions.len(), 1);
    assert_eq!(program.functions[0].name, "挨拶");
}

#[test]
fn test_unicode_return_type() {
    // Although not a valid type, parser should accept any identifier
    let program = parse("fn main() -> 型 {}").unwrap();
    assert_eq!(program.functions[0].return_type, "型");
}

// ===================
// Let statement parsing
// ===================

#[test]
fn test_let_stmt_i32() {
    let program = parse("fn main() -> void { let x: i32 = 42 }").unwrap();
    assert_eq!(program.functions[0].body.len(), 1);
    match &program.functions[0].body[0].kind {
        StmtKind::Let { name, ty, init } => {
            assert_eq!(name, "x");
            assert_eq!(*ty, Type::I32);
            assert!(matches!(init.kind, ExprKind::IntLiteral(42)));
        }
        _ => panic!("Expected Let statement"),
    }
}

#[test]
fn test_let_stmt_i64() {
    let program = parse("fn main() -> void { let y: i64 = 100 }").unwrap();
    match &program.functions[0].body[0].kind {
        StmtKind::Let { name, ty, .. } => {
            assert_eq!(name, "y");
            assert_eq!(*ty, Type::I64);
        }
        _ => panic!("Expected Let statement"),
    }
}

#[test]
fn test_let_with_variable_reference() {
    let program = parse("fn main() -> void { let x: i32 = 1\nlet y: i32 = x }").unwrap();
    assert_eq!(program.functions[0].body.len(), 2);
    match &program.functions[0].body[1].kind {
        StmtKind::Let { name, init, .. } => {
            assert_eq!(name, "y");
            assert!(matches!(&init.kind, ExprKind::Identifier(s) if s == "x"));
        }
        _ => panic!("Expected Let statement"),
    }
}

#[test]
fn test_let_mixed_with_println() {
    let program = parse(
        r#"fn main() -> void {
            let x: i32 = 42
            println("hello")
            let y: i64 = 100
        }"#,
    )
    .unwrap();
    assert_eq!(program.functions[0].body.len(), 3);
    assert!(matches!(
        &program.functions[0].body[0].kind,
        StmtKind::Let { .. }
    ));
    assert!(matches!(
        &program.functions[0].body[1].kind,
        StmtKind::Expr(_)
    ));
    assert!(matches!(
        &program.functions[0].body[2].kind,
        StmtKind::Let { .. }
    ));
}

#[test]
fn test_error_let_missing_colon() {
    let err = parse_error("fn main() -> void { let x i32 = 42 }");
    assert!(
        err.message.contains("':'"),
        "Expected error about ':', got: {}",
        err.message
    );
}

#[test]
fn test_error_let_missing_type() {
    let err = parse_error("fn main() -> void { let x: = 42 }");
    assert!(
        err.message.contains("identifier"),
        "Expected error about identifier, got: {}",
        err.message
    );
}

#[test]
fn test_error_let_unknown_type() {
    let err = parse_error("fn main() -> void { let x: unknown = 42 }");
    assert!(
        err.message.contains("Unknown type"),
        "Expected error about unknown type, got: {}",
        err.message
    );
}

#[test]
fn test_error_let_missing_equals() {
    let err = parse_error("fn main() -> void { let x: i32 42 }");
    assert!(
        err.message.contains("'='"),
        "Expected error about '=', got: {}",
        err.message
    );
}

#[test]
fn test_error_let_missing_initializer() {
    let err = parse_error("fn main() -> void { let x: i32 = }");
    assert!(
        err.message.contains("Unexpected token"),
        "Expected error about unexpected token, got: {}",
        err.message
    );
}

// ===================
// Integer literal parsing
// ===================

#[test]
fn test_int_literal_zero() {
    let program = parse("fn main() -> void { let x: i32 = 0 }").unwrap();
    match &program.functions[0].body[0].kind {
        StmtKind::Let { init, .. } => {
            assert!(matches!(init.kind, ExprKind::IntLiteral(0)));
        }
        _ => panic!("Expected Let statement"),
    }
}

#[test]
fn test_int_literal_large() {
    let program = parse("fn main() -> void { let x: i64 = 9223372036854775807 }").unwrap();
    match &program.functions[0].body[0].kind {
        StmtKind::Let { init, .. } => {
            assert!(matches!(
                init.kind,
                ExprKind::IntLiteral(9223372036854775807)
            ));
        }
        _ => panic!("Expected Let statement"),
    }
}

// ===================
// Variable reference parsing
// ===================

#[test]
fn test_variable_reference_in_init() {
    let program = parse("fn main() -> void { let a: i32 = 1\nlet b: i32 = a }").unwrap();
    match &program.functions[0].body[1].kind {
        StmtKind::Let { init, .. } => {
            assert!(matches!(&init.kind, ExprKind::Identifier(s) if s == "a"));
        }
        _ => panic!("Expected Let statement"),
    }
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

// ============================================================
// FnDef span calculation tests
// ============================================================

#[test]
fn test_fn_def_span_simple() {
    // "fn main() -> void {}"
    // 0123456789...
    let source = "fn main() -> void {}";
    let program = parse(source).unwrap();
    assert_eq!(program.functions.len(), 1);

    let fn_def = &program.functions[0];
    // Span should cover "fn main() -> void " (from 'f' to just before '{')
    assert_eq!(fn_def.span.start, 0, "span.start should be at 'f'");
    assert_eq!(
        fn_def.span.end, 18,
        "span.end should be just before '{{' at position 18"
    );
    assert_eq!(fn_def.span.line, 1);
    assert_eq!(fn_def.span.column, 1);
}

#[test]
fn test_fn_def_span_with_leading_whitespace() {
    // "  fn foo() -> void {}"
    // 0123456789...
    let source = "  fn foo() -> void {}";
    let program = parse(source).unwrap();
    assert_eq!(program.functions.len(), 1);

    let fn_def = &program.functions[0];
    // Span should start at 'f' (position 2), not at the leading whitespace
    assert_eq!(
        fn_def.span.start, 2,
        "span.start should skip leading whitespace"
    );
    assert_eq!(fn_def.span.line, 1);
    assert_eq!(fn_def.span.column, 3);
}

#[test]
fn test_fn_def_span_with_body() {
    // Function definition with body content
    let source = "fn main() -> void {\n    println(\"hello\")\n}";
    let program = parse(source).unwrap();
    assert_eq!(program.functions.len(), 1);

    let fn_def = &program.functions[0];
    // Span should cover only the signature "fn main() -> void " (0..18)
    // not the body content
    assert_eq!(fn_def.span.start, 0);
    assert_eq!(fn_def.span.end, 18);
    assert_eq!(fn_def.span.line, 1);
    assert_eq!(fn_def.span.column, 1);
}

#[test]
fn test_fn_def_span_multiple_functions() {
    let source = "fn foo() -> void {}\nfn bar() -> void {}";
    let program = parse(source).unwrap();
    assert_eq!(program.functions.len(), 2);

    // First function: "fn foo() -> void " spans 0..17
    let foo = &program.functions[0];
    assert_eq!(foo.span.start, 0);
    assert_eq!(foo.span.line, 1);

    // Second function: starts at position 20 (after "fn foo() -> void {}\n")
    let bar = &program.functions[1];
    assert_eq!(bar.span.start, 20);
    assert_eq!(bar.span.line, 2);
    assert_eq!(bar.span.column, 1);
}

// ============================================================
// return_type_span calculation tests
// ============================================================

#[test]
fn test_return_type_span_simple() {
    // "fn main() -> void {}"
    // 0         1
    // 0123456789012345678901
    // void starts at position 13
    let source = "fn main() -> void {}";
    let program = parse(source).unwrap();
    let fn_def = &program.functions[0];

    // "void" spans from 13 to 17
    assert_eq!(fn_def.return_type_span.start, 13);
    assert_eq!(fn_def.return_type_span.end, 17);
    assert_eq!(fn_def.return_type_span.line, 1);
    assert_eq!(fn_def.return_type_span.column, 14);
}

#[test]
fn test_return_type_span_int() {
    // "fn main() -> int {}"
    // 0         1
    // 0123456789012345678
    // int starts at position 13
    let source = "fn main() -> int {}";
    let program = parse(source).unwrap();
    let fn_def = &program.functions[0];

    // "int" spans from 13 to 16
    assert_eq!(fn_def.return_type_span.start, 13);
    assert_eq!(fn_def.return_type_span.end, 16);
}

#[test]
fn test_return_type_span_with_leading_whitespace() {
    // "  fn foo() -> void {}"
    // 0         1
    // 012345678901234567890
    //   fn foo() -> void {}
    //               ^--- void starts at position 14
    let source = "  fn foo() -> void {}";
    let program = parse(source).unwrap();
    let fn_def = &program.functions[0];

    assert_eq!(fn_def.return_type_span.start, 14);
    assert_eq!(fn_def.return_type_span.end, 18);
    assert_eq!(fn_def.return_type_span.column, 15);
}

#[test]
fn test_return_type_span_multiple_functions() {
    // "fn foo() -> void {}\nfn bar() -> int {}"
    // 0         1         2         3
    // 0123456789012345678901234567890123456789
    // fn foo() -> void {}\nfn bar() -> int {}
    //             ^--- void at 12
    //                                 ^--- int at 32
    let source = "fn foo() -> void {}\nfn bar() -> int {}";
    let program = parse(source).unwrap();

    // First function: "void" at position 12
    let foo = &program.functions[0];
    assert_eq!(foo.return_type_span.start, 12);
    assert_eq!(foo.return_type_span.end, 16);

    // Second function: "int" at position 32 (20 + 12)
    let bar = &program.functions[1];
    assert_eq!(bar.return_type_span.start, 32);
    assert_eq!(bar.return_type_span.end, 35);
    assert_eq!(bar.return_type_span.line, 2);
}

// ============================================================
// Span edge case tests
// ============================================================

#[test]
fn test_fn_def_span_with_extra_whitespace() {
    // Test function with extra whitespace between tokens
    let source = "fn   main()   ->   void   {}";
    let program = parse(source).unwrap();
    let fn_def = &program.functions[0];

    // Function span should start at 'f' and end before '{'
    assert_eq!(fn_def.span.start, 0);
    // return_type_span should point to 'void'
    assert_eq!(fn_def.return_type, "void");
    assert!(fn_def.return_type_span.start > 0);
    assert!(fn_def.return_type_span.end > fn_def.return_type_span.start);
}

#[test]
fn test_fn_def_span_with_comment_before() {
    // Comment before function definition
    let source = "// This is a comment\nfn main() -> void {}";
    let program = parse(source).unwrap();
    let fn_def = &program.functions[0];

    // Function span should start at 'fn', not at the comment
    assert_eq!(fn_def.span.start, 21); // After "// This is a comment\n"
    assert_eq!(fn_def.span.line, 2);
    assert_eq!(fn_def.span.column, 1);
}

#[test]
fn test_fn_def_span_with_long_function_name() {
    // Test with a longer function name
    let source = "fn very_long_function_name_here() -> void {}";
    let program = parse(source).unwrap();
    let fn_def = &program.functions[0];

    assert_eq!(fn_def.name, "very_long_function_name_here");
    // return_type_span should still correctly point to 'void'
    let rt_start = fn_def.return_type_span.start;
    let rt_end = fn_def.return_type_span.end;
    assert_eq!(rt_end - rt_start, 4); // "void" is 4 characters
}

#[test]
fn test_fn_def_span_with_unicode_name() {
    // Test with unicode function name
    let source = "fn 日本語() -> void {}";
    let program = parse(source).unwrap();
    let fn_def = &program.functions[0];

    assert_eq!(fn_def.name, "日本語");
    // Spans use byte offsets, so unicode characters affect positions
    // "fn " = 3 bytes, "日本語" = 9 bytes (3 chars * 3 bytes each)
    // "() -> " = 6 bytes, so "void" starts at 3 + 9 + 6 = 18
    assert_eq!(fn_def.return_type_span.start, 18);
}

#[test]
fn test_return_type_span_with_unicode_return_type() {
    // Test with unicode return type (hypothetical future feature)
    let source = "fn main() -> 型 {}";
    let program = parse(source).unwrap();
    let fn_def = &program.functions[0];

    assert_eq!(fn_def.return_type, "型");
    // "fn main() -> " = 13 bytes, "型" = 3 bytes
    assert_eq!(fn_def.return_type_span.start, 13);
    assert_eq!(fn_def.return_type_span.end, 16); // 13 + 3 bytes
}
