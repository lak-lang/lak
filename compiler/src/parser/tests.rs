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
    let program = parse("fn foo() -> void {} fn bar() -> void {}").unwrap();
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
fn test_multiple_statements_in_body() {
    let program = parse("fn main() -> void { f() g() }").unwrap();
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
    let program = parse("fn main() -> void { let x: i32 = 1 let y: i32 = x }").unwrap();
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
    let program = parse("fn main() -> void { let a: i32 = 1 let b: i32 = a }").unwrap();
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
