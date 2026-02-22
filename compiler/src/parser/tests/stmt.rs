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
    assert_eq!(
        err.message(),
        "Expected newline after statement, found identifier 'g'"
    );
}

#[test]
fn test_error_multiple_functions_same_line() {
    let err = parse_error("fn foo() -> void {}fn bar() -> void {}");
    assert_eq!(
        err.message(),
        "Expected newline after statement, found 'fn' keyword"
    );
}

#[test]
fn test_error_multiple_functions_same_line_with_space() {
    // Space between functions but still on same line
    let err = parse_error("fn foo() -> void {} fn bar() -> void {}");
    assert_eq!(
        err.message(),
        "Expected newline after statement, found 'fn' keyword"
    );
}

#[test]
fn test_error_let_statements_same_line() {
    let err = parse_error("fn main() -> void { let x: i32 = 1 let y: i32 = 2 }");
    assert_eq!(
        err.message(),
        "Expected newline after statement, found 'let' keyword"
    );
}

#[test]
fn test_error_let_then_expr_same_line() {
    let err = parse_error("fn main() -> void { let x: i32 = 1 f() }");
    assert_eq!(
        err.message(),
        "Expected newline after statement, found identifier 'f'"
    );
}

#[test]
fn test_error_expr_then_let_same_line() {
    let err = parse_error("fn main() -> void { f() let x: i32 = 1 }");
    assert_eq!(
        err.message(),
        "Expected newline after statement, found 'let' keyword"
    );
}

#[test]
fn test_error_same_line_reports_correct_token() {
    // Verify the error message includes what token was found
    let err = parse_error("fn main() -> void { f() g() }");
    assert_eq!(
        err.message(),
        "Expected newline after statement, found identifier 'g'"
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
        StmtKind::Let {
            is_mutable,
            name,
            ty,
            init,
        } => {
            assert!(!is_mutable);
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
fn test_let_stmt_remaining_integer_types_and_byte_alias() {
    let program = parse(
        r#"fn main() -> void {
            let a: i8 = -1
            let b: i16 = 2
            let c: u8 = 3
            let d: u16 = 4
            let e: u32 = 5
            let f: u64 = 6
            let g: byte = 7
        }"#,
    )
    .unwrap();

    let expected = [
        Type::I8,
        Type::I16,
        Type::U8,
        Type::U16,
        Type::U32,
        Type::U64,
        Type::U8, // byte alias
    ];

    for (idx, expected_ty) in expected.iter().enumerate() {
        match &program.functions[0].body[idx].kind {
            StmtKind::Let { ty, .. } => assert_eq!(ty, expected_ty),
            _ => panic!("Expected Let statement"),
        }
    }
}

#[test]
fn test_let_with_variable_reference() {
    let program = parse("fn main() -> void { let x: i32 = 1\nlet y: i32 = x }").unwrap();
    assert_eq!(program.functions[0].body.len(), 2);
    match &program.functions[0].body[1].kind {
        StmtKind::Let {
            is_mutable,
            name,
            init,
            ..
        } => {
            assert!(!is_mutable);
            assert_eq!(name, "y");
            assert!(matches!(&init.kind, ExprKind::Identifier(s) if s == "x"));
        }
        _ => panic!("Expected Let statement"),
    }
}

#[test]
fn test_let_mut_stmt_i32() {
    let program = parse("fn main() -> void { let mut x: i32 = 42 }").unwrap();
    assert_eq!(program.functions[0].body.len(), 1);
    match &program.functions[0].body[0].kind {
        StmtKind::Let {
            is_mutable,
            name,
            ty,
            init,
        } => {
            assert!(*is_mutable);
            assert_eq!(name, "x");
            assert_eq!(*ty, Type::I32);
            assert!(matches!(init.kind, ExprKind::IntLiteral(42)));
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

// ===================
// Assignment statement parsing
// ===================

#[test]
fn test_assign_stmt_basic() {
    let program = parse(
        r#"fn main() -> void {
            let mut x: i32 = 1
            x = 2
        }"#,
    )
    .unwrap();

    assert_eq!(program.functions[0].body.len(), 2);
    match &program.functions[0].body[1].kind {
        StmtKind::Assign { name, value } => {
            assert_eq!(name, "x");
            assert!(matches!(value.kind, ExprKind::IntLiteral(2)));
        }
        _ => panic!("Expected Assign statement"),
    }
}

#[test]
fn test_assign_stmt_with_binary_expression() {
    let program = parse(
        r#"fn main() -> void {
            let mut x: i32 = 1
            x = x + 1
        }"#,
    )
    .unwrap();

    match &program.functions[0].body[1].kind {
        StmtKind::Assign { name, value } => {
            assert_eq!(name, "x");
            assert!(matches!(value.kind, ExprKind::BinaryOp { .. }));
        }
        _ => panic!("Expected Assign statement"),
    }
}

#[test]
fn test_expression_statement_equality_is_not_assignment() {
    let program = parse(
        r#"fn main() -> void {
            let x: bool = true
            x == true
        }"#,
    )
    .unwrap();

    assert!(matches!(
        program.functions[0].body[1].kind,
        StmtKind::Expr(_)
    ));
}

#[test]
fn test_error_let_missing_colon() {
    let err = parse_error("fn main() -> void { let x i32 = 42 }");
    assert_eq!(err.message(), "Expected ':', found identifier 'i32'");
}

#[test]
fn test_error_let_mut_missing_variable_name() {
    let err = parse_error("fn main() -> void { let mut : i32 = 42 }");
    assert_eq!(err.message(), "Expected identifier, found ':'");
}

#[test]
fn test_error_let_mut_discard_binding() {
    let err = parse_error("fn main() -> void { let mut _ = f() }");
    assert_eq!(
        err.message(),
        "Discard binding '_' cannot be declared mutable"
    );
}

#[test]
fn test_error_let_missing_type() {
    let err = parse_error("fn main() -> void { let x: = 42 }");
    assert_eq!(err.message(), "Expected identifier, found '='");
}

#[test]
fn test_error_let_unknown_type() {
    let err = parse_error("fn main() -> void { let x: unknown = 42 }");
    assert_eq!(
        err.message(),
        "Unknown type: 'unknown'. Expected 'i8', 'i16', 'i32', 'i64', 'u8', 'u16', 'u32', 'u64', 'byte', 'string', or 'bool'"
    );
}

#[test]
fn test_error_let_missing_equals() {
    let err = parse_error("fn main() -> void { let x: i32 42 }");
    assert_eq!(err.message(), "Expected '=', found integer '42'");
}

#[test]
fn test_error_let_missing_initializer() {
    let err = parse_error("fn main() -> void { let x: i32 = }");
    assert_eq!(err.message(), "Unexpected token: '}'");
}

#[test]
fn test_discard_stmt_call() {
    let program = parse("fn main() -> void { let _ = f() }").unwrap();
    match &program.functions[0].body[0].kind {
        StmtKind::Discard(expr) => match &expr.kind {
            ExprKind::Call { callee, .. } => assert_eq!(callee, "f"),
            _ => panic!("Expected function call"),
        },
        _ => panic!("Expected Discard statement"),
    }
}

#[test]
fn test_error_let_typed_discard_binding() {
    let err = parse_error("fn main() -> void { let _: i32 = 1 }");
    assert_eq!(
        err.message(),
        "Discard binding '_' cannot have a type annotation; use `let _ = expr`"
    );
}

#[test]
fn test_error_discard_missing_equals() {
    let err = parse_error("fn main() -> void { let _ let _ = 1 }");
    assert_eq!(err.message(), "Expected '=', found 'let' keyword");
}

#[test]
fn test_error_assign_missing_value() {
    let err = parse_error("fn main() -> void { x = }");
    assert_eq!(err.message(), "Unexpected token: '}'");
}

// ===================
// Return statement parsing
// ===================

#[test]
fn test_return_stmt_with_value() {
    let program = parse("fn main() -> void { return 42 }").unwrap();
    match &program.functions[0].body[0].kind {
        StmtKind::Return(Some(expr)) => {
            assert!(matches!(expr.kind, ExprKind::IntLiteral(42)));
        }
        _ => panic!("Expected return with value"),
    }
}

#[test]
fn test_return_stmt_bare() {
    let program = parse("fn main() -> void { return }").unwrap();
    assert!(matches!(
        program.functions[0].body[0].kind,
        StmtKind::Return(None)
    ));
}

// ===================
// If statement parsing
// ===================

#[test]
fn test_if_stmt_without_else() {
    let program = parse(
        r#"fn main() -> void {
            if true {
                println("then")
            }
        }"#,
    )
    .unwrap();

    assert_eq!(program.functions[0].body.len(), 1);
    match &program.functions[0].body[0].kind {
        StmtKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            assert!(matches!(condition.kind, ExprKind::BoolLiteral(true)));
            assert_eq!(then_branch.len(), 1);
            assert!(else_branch.is_none());
        }
        _ => panic!("Expected If statement"),
    }
}

#[test]
fn test_if_stmt_with_else() {
    let program = parse(
        r#"fn main() -> void {
            if true {
                println("then")
            } else {
                println("else")
            }
        }"#,
    )
    .unwrap();

    match &program.functions[0].body[0].kind {
        StmtKind::If {
            then_branch,
            else_branch,
            ..
        } => {
            assert_eq!(then_branch.len(), 1);
            assert_eq!(else_branch.as_ref().map(Vec::len), Some(1));
        }
        _ => panic!("Expected If statement"),
    }
}

#[test]
fn test_if_stmt_with_else_if_chain() {
    let program = parse(
        r#"fn main() -> void {
            if false {
                println("a")
            } else if true {
                println("b")
            } else {
                println("c")
            }
        }"#,
    )
    .unwrap();

    match &program.functions[0].body[0].kind {
        StmtKind::If { else_branch, .. } => {
            let outer_else = else_branch.as_ref().expect("outer else should exist");
            assert_eq!(outer_else.len(), 1);
            match &outer_else[0].kind {
                StmtKind::If {
                    condition,
                    then_branch,
                    else_branch,
                } => {
                    assert!(matches!(condition.kind, ExprKind::BoolLiteral(true)));
                    assert_eq!(then_branch.len(), 1);
                    assert_eq!(else_branch.as_ref().map(Vec::len), Some(1));
                }
                _ => panic!("Expected nested If statement for else-if"),
            }
        }
        _ => panic!("Expected If statement"),
    }
}

#[test]
fn test_if_else_on_next_line() {
    let program = parse(
        r#"fn main() -> void {
            if true {
                println("then")
            }
            else {
                println("else")
            }
        }"#,
    )
    .unwrap();

    match &program.functions[0].body[0].kind {
        StmtKind::If { else_branch, .. } => {
            assert_eq!(else_branch.as_ref().map(Vec::len), Some(1));
        }
        _ => panic!("Expected If statement"),
    }
}

// ===================
// While statement parsing
// ===================

#[test]
fn test_while_stmt_basic() {
    let program = parse(
        r#"fn main() -> void {
            while true {
                println("loop")
            }
        }"#,
    )
    .unwrap();

    assert_eq!(program.functions[0].body.len(), 1);
    match &program.functions[0].body[0].kind {
        StmtKind::While { condition, body } => {
            assert!(matches!(condition.kind, ExprKind::BoolLiteral(true)));
            assert_eq!(body.len(), 1);
        }
        _ => panic!("Expected While statement"),
    }
}

#[test]
fn test_while_stmt_with_break_and_continue() {
    let program = parse(
        r#"fn main() -> void {
            while true {
                if false {
                    continue
                }
                break
            }
        }"#,
    )
    .unwrap();

    match &program.functions[0].body[0].kind {
        StmtKind::While { body, .. } => {
            assert_eq!(body.len(), 2);
            assert!(matches!(&body[0].kind, StmtKind::If { .. }));
            assert!(matches!(&body[1].kind, StmtKind::Break));
        }
        _ => panic!("Expected While statement"),
    }
}

#[test]
fn test_break_stmt() {
    let program = parse("fn main() -> void { break }").unwrap();
    assert!(matches!(program.functions[0].body[0].kind, StmtKind::Break));
}

#[test]
fn test_continue_stmt() {
    let program = parse("fn main() -> void { continue }").unwrap();
    assert!(matches!(
        program.functions[0].body[0].kind,
        StmtKind::Continue
    ));
}

#[test]
fn test_error_break_continue_same_line() {
    let err = parse_error("fn main() -> void { break continue }");
    assert_eq!(
        err.message(),
        "Expected newline after statement, found 'continue' keyword"
    );
}
