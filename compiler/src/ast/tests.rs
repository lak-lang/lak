//! Unit tests for AST nodes.

use super::*;
use crate::ast::Visibility;
use crate::token::Span;

fn dummy_span() -> Span {
    Span::new(0, 0, 1, 1)
}

#[test]
fn test_expr_string_literal() {
    let expr = Expr::new(ExprKind::StringLiteral("hello".to_string()), dummy_span());
    assert!(matches!(expr.kind, ExprKind::StringLiteral(ref s) if s == "hello"));
}

#[test]
fn test_expr_call_no_args() {
    let expr = Expr::new(
        ExprKind::Call {
            callee: "func".to_string(),
            args: vec![],
        },
        dummy_span(),
    );
    match expr.kind {
        ExprKind::Call { callee, args } => {
            assert_eq!(callee, "func");
            assert!(args.is_empty());
        }
        _ => panic!("Expected Call"),
    }
}

#[test]
fn test_expr_call_with_args() {
    let expr = Expr::new(
        ExprKind::Call {
            callee: "println".to_string(),
            args: vec![
                Expr::new(ExprKind::StringLiteral("a".to_string()), dummy_span()),
                Expr::new(ExprKind::StringLiteral("b".to_string()), dummy_span()),
            ],
        },
        dummy_span(),
    );
    match expr.kind {
        ExprKind::Call { callee, args } => {
            assert_eq!(callee, "println");
            assert_eq!(args.len(), 2);
        }
        _ => panic!("Expected Call"),
    }
}

#[test]
fn test_expr_call_nested() {
    let inner = Expr::new(
        ExprKind::Call {
            callee: "inner".to_string(),
            args: vec![],
        },
        dummy_span(),
    );
    let outer = Expr::new(
        ExprKind::Call {
            callee: "outer".to_string(),
            args: vec![inner],
        },
        dummy_span(),
    );
    match outer.kind {
        ExprKind::Call { callee, args } => {
            assert_eq!(callee, "outer");
            assert_eq!(args.len(), 1);
            assert!(matches!(&args[0].kind, ExprKind::Call { callee, .. } if callee == "inner"));
        }
        _ => panic!("Expected Call"),
    }
}

#[test]
fn test_stmt_expr() {
    let expr = Expr::new(ExprKind::StringLiteral("test".to_string()), dummy_span());
    let stmt = Stmt::new(StmtKind::Expr(expr), dummy_span());
    match stmt.kind {
        StmtKind::Expr(e) => {
            assert!(matches!(e.kind, ExprKind::StringLiteral(ref s) if s == "test"));
        }
        _ => panic!("Expected Expr statement"),
    }
}

#[test]
fn test_stmt_let() {
    let stmt = Stmt::new(
        StmtKind::Let {
            name: "x".to_string(),
            ty: Type::I32,
            init: Expr::new(ExprKind::IntLiteral(42), dummy_span()),
        },
        dummy_span(),
    );
    match stmt.kind {
        StmtKind::Let { name, ty, init } => {
            assert_eq!(name, "x");
            assert_eq!(ty, Type::I32);
            assert!(matches!(init.kind, ExprKind::IntLiteral(42)));
        }
        _ => panic!("Expected Let statement"),
    }
}

#[test]
fn test_expr_int_literal() {
    let expr = Expr::new(ExprKind::IntLiteral(100), dummy_span());
    assert!(matches!(expr.kind, ExprKind::IntLiteral(100)));
}

#[test]
fn test_expr_identifier() {
    let expr = Expr::new(ExprKind::Identifier("my_var".to_string()), dummy_span());
    assert!(matches!(expr.kind, ExprKind::Identifier(ref s) if s == "my_var"));
}

#[test]
fn test_type_i32() {
    let ty = Type::I32;
    assert_eq!(ty, Type::I32);
}

#[test]
fn test_type_i64() {
    let ty = Type::I64;
    assert_eq!(ty, Type::I64);
}

#[test]
fn test_type_clone() {
    let ty1 = Type::I32;
    let ty2 = ty1.clone();
    assert_eq!(ty1, ty2);
}

#[test]
fn test_program_empty() {
    let program = Program {
        imports: vec![],
        functions: vec![],
    };
    assert!(program.functions.is_empty());
}

#[test]
fn test_program_with_functions() {
    let functions = vec![FnDef {
        visibility: Visibility::Private,
        name: "main".to_string(),
        return_type: "void".to_string(),
        return_type_span: dummy_span(),
        body: vec![Stmt::new(
            StmtKind::Expr(Expr::new(
                ExprKind::Call {
                    callee: "println".to_string(),
                    args: vec![Expr::new(
                        ExprKind::StringLiteral("hello".to_string()),
                        dummy_span(),
                    )],
                },
                dummy_span(),
            )),
            dummy_span(),
        )],
        span: dummy_span(),
    }];
    let program = Program {
        imports: vec![],
        functions,
    };
    assert_eq!(program.functions.len(), 1);
    assert_eq!(program.functions[0].name, "main");
}

#[test]
fn test_fn_def() {
    let fn_def = FnDef {
        visibility: Visibility::Private,
        name: "test".to_string(),
        return_type: "void".to_string(),
        return_type_span: dummy_span(),
        body: vec![],
        span: dummy_span(),
    };
    assert_eq!(fn_def.name, "test");
    assert_eq!(fn_def.return_type, "void");
    assert!(fn_def.body.is_empty());
}

#[test]
fn test_fn_def_with_body() {
    let fn_def = FnDef {
        visibility: Visibility::Private,
        name: "greet".to_string(),
        return_type: "void".to_string(),
        return_type_span: dummy_span(),
        body: vec![
            Stmt::new(
                StmtKind::Expr(Expr::new(
                    ExprKind::Call {
                        callee: "println".to_string(),
                        args: vec![Expr::new(
                            ExprKind::StringLiteral("hello".to_string()),
                            dummy_span(),
                        )],
                    },
                    dummy_span(),
                )),
                dummy_span(),
            ),
            Stmt::new(
                StmtKind::Expr(Expr::new(
                    ExprKind::Call {
                        callee: "println".to_string(),
                        args: vec![Expr::new(
                            ExprKind::StringLiteral("world".to_string()),
                            dummy_span(),
                        )],
                    },
                    dummy_span(),
                )),
                dummy_span(),
            ),
        ],
        span: dummy_span(),
    };
    assert_eq!(fn_def.body.len(), 2);
}

#[test]
fn test_fn_def_clone() {
    let fn_def = FnDef {
        visibility: Visibility::Private,
        name: "test".to_string(),
        return_type: "void".to_string(),
        return_type_span: dummy_span(),
        body: vec![Stmt::new(
            StmtKind::Expr(Expr::new(
                ExprKind::StringLiteral("x".to_string()),
                dummy_span(),
            )),
            dummy_span(),
        )],
        span: dummy_span(),
    };
    let cloned = fn_def.clone();
    assert_eq!(fn_def.name, cloned.name);
    assert_eq!(fn_def.return_type, cloned.return_type);
    assert_eq!(fn_def.body.len(), cloned.body.len());
}

#[test]
fn test_expr_clone() {
    let expr1 = Expr::new(
        ExprKind::Call {
            callee: "test".to_string(),
            args: vec![Expr::new(
                ExprKind::StringLiteral("arg".to_string()),
                dummy_span(),
            )],
        },
        dummy_span(),
    );
    let expr2 = expr1.clone();

    // Verify both have same structure
    match (&expr1.kind, &expr2.kind) {
        (
            ExprKind::Call {
                callee: c1,
                args: a1,
            },
            ExprKind::Call {
                callee: c2,
                args: a2,
            },
        ) => {
            assert_eq!(c1, c2);
            assert_eq!(a1.len(), a2.len());
        }
        _ => panic!("Clone failed"),
    }
}

#[test]
fn test_stmt_clone() {
    let stmt1 = Stmt::new(
        StmtKind::Expr(Expr::new(
            ExprKind::StringLiteral("test".to_string()),
            dummy_span(),
        )),
        dummy_span(),
    );
    let stmt2 = stmt1.clone();

    match (&stmt1.kind, &stmt2.kind) {
        (StmtKind::Expr(e1), StmtKind::Expr(e2)) => {
            assert!(matches!(e1.kind, ExprKind::StringLiteral(ref s) if s == "test"));
            assert!(matches!(e2.kind, ExprKind::StringLiteral(ref s) if s == "test"));
        }
        _ => panic!("Expected Expr statements"),
    }
}

#[test]
fn test_expr_debug() {
    let expr = Expr::new(ExprKind::StringLiteral("hello".to_string()), dummy_span());
    let debug_str = format!("{:?}", expr);
    assert_eq!(
        debug_str,
        "Expr { kind: StringLiteral(\"hello\"), span: Span { start: 0, end: 0, line: 1, column: 1 } }"
    );
}

#[test]
fn test_stmt_debug() {
    let stmt = Stmt::new(
        StmtKind::Expr(Expr::new(
            ExprKind::Call {
                callee: "func".to_string(),
                args: vec![],
            },
            dummy_span(),
        )),
        dummy_span(),
    );
    let debug_str = format!("{:?}", stmt);
    assert_eq!(
        debug_str,
        "Stmt { kind: Expr(Expr { kind: Call { callee: \"func\", args: [] }, span: Span { start: 0, end: 0, line: 1, column: 1 } }), span: Span { start: 0, end: 0, line: 1, column: 1 } }"
    );
}

#[test]
fn test_program_debug() {
    let program = Program {
        imports: vec![],
        functions: vec![FnDef {
            visibility: Visibility::Private,
            name: "main".to_string(),
            return_type: "void".to_string(),
            return_type_span: dummy_span(),
            body: vec![Stmt::new(
                StmtKind::Expr(Expr::new(
                    ExprKind::StringLiteral("test".to_string()),
                    dummy_span(),
                )),
                dummy_span(),
            )],
            span: dummy_span(),
        }],
    };
    let debug_str = format!("{:?}", program);
    assert_eq!(
        debug_str,
        "Program { imports: [], functions: [FnDef { visibility: Private, name: \"main\", return_type: \"void\", return_type_span: Span { start: 0, end: 0, line: 1, column: 1 }, body: [Stmt { kind: Expr(Expr { kind: StringLiteral(\"test\"), span: Span { start: 0, end: 0, line: 1, column: 1 } }), span: Span { start: 0, end: 0, line: 1, column: 1 } }], span: Span { start: 0, end: 0, line: 1, column: 1 } }] }"
    );
}

#[test]
fn test_expr_span() {
    let span = Span::new(10, 20, 2, 5);
    let expr = Expr::new(ExprKind::IntLiteral(42), span);
    assert_eq!(expr.span.start, 10);
    assert_eq!(expr.span.end, 20);
    assert_eq!(expr.span.line, 2);
    assert_eq!(expr.span.column, 5);
}

#[test]
fn test_stmt_span() {
    let span = Span::new(0, 15, 1, 1);
    let stmt = Stmt::new(
        StmtKind::Expr(Expr::new(ExprKind::IntLiteral(1), dummy_span())),
        span,
    );
    assert_eq!(stmt.span.start, 0);
    assert_eq!(stmt.span.end, 15);
}
