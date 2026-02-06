//! Statement parsing tests.
//!
//! Tests for:
//! - Multiple statements in function body
//! - Newline requirements between statements
//! - Let statement parsing

use super::*;

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
        err.message().contains("newline"),
        "Expected error about newline, got: {}",
        err.message()
    );
}

#[test]
fn test_error_multiple_functions_same_line() {
    let err = parse_error("fn foo() -> void {}fn bar() -> void {}");
    assert!(
        err.message().contains("newline"),
        "Expected error about newline, got: {}",
        err.message()
    );
}

#[test]
fn test_error_multiple_functions_same_line_with_space() {
    // Space between functions but still on same line
    let err = parse_error("fn foo() -> void {} fn bar() -> void {}");
    assert!(
        err.message().contains("newline"),
        "Expected error about newline, got: {}",
        err.message()
    );
}

#[test]
fn test_error_let_statements_same_line() {
    let err = parse_error("fn main() -> void { let x: i32 = 1 let y: i32 = 2 }");
    assert!(
        err.message().contains("newline"),
        "Expected error about newline, got: {}",
        err.message()
    );
}

#[test]
fn test_error_let_then_expr_same_line() {
    let err = parse_error("fn main() -> void { let x: i32 = 1 f() }");
    assert!(
        err.message().contains("newline"),
        "Expected error about newline, got: {}",
        err.message()
    );
}

#[test]
fn test_error_expr_then_let_same_line() {
    let err = parse_error("fn main() -> void { f() let x: i32 = 1 }");
    assert!(
        err.message().contains("newline"),
        "Expected error about newline, got: {}",
        err.message()
    );
}

#[test]
fn test_error_same_line_reports_correct_token() {
    // Verify the error message includes what token was found
    let err = parse_error("fn main() -> void { f() g() }");
    assert!(
        err.message().contains("identifier 'g'"),
        "Expected error to mention the found token, got: {}",
        err.message()
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
        err.span().column,
        25,
        "Expected error at column 25 (the 'g'), got column {}",
        err.span().column
    );
}

#[test]
fn test_statements_with_comments() {
    let program = parse("fn main() -> void { f() // c\ng() }").unwrap();
    assert_eq!(program.functions[0].body.len(), 2);
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
        err.message().contains("':'"),
        "Expected error about ':', got: {}",
        err.message()
    );
}

#[test]
fn test_error_let_missing_type() {
    let err = parse_error("fn main() -> void { let x: = 42 }");
    assert!(
        err.message().contains("identifier"),
        "Expected error about identifier, got: {}",
        err.message()
    );
}

#[test]
fn test_error_let_unknown_type() {
    let err = parse_error("fn main() -> void { let x: unknown = 42 }");
    assert!(
        err.message().contains("Unknown type"),
        "Expected error about unknown type, got: {}",
        err.message()
    );
}

#[test]
fn test_error_let_missing_equals() {
    let err = parse_error("fn main() -> void { let x: i32 42 }");
    assert!(
        err.message().contains("'='"),
        "Expected error about '=', got: {}",
        err.message()
    );
}

#[test]
fn test_error_let_missing_initializer() {
    let err = parse_error("fn main() -> void { let x: i32 = }");
    assert!(
        err.message().contains("Unexpected token"),
        "Expected error about unexpected token, got: {}",
        err.message()
    );
}
