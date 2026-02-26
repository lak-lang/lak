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
            is_mutable: false,
            name: "x".to_string(),
            ty: Type::I32,
            init: Expr::new(ExprKind::IntLiteral(42), dummy_span()),
        },
        dummy_span(),
    );
    match stmt.kind {
        StmtKind::Let {
            is_mutable,
            name,
            ty,
            init,
        } => {
            assert!(!is_mutable);
            assert_eq!(name, "x");
            assert_eq!(ty, Type::I32);
            assert!(matches!(init.kind, ExprKind::IntLiteral(42)));
        }
        _ => panic!("Expected Let statement"),
    }
}

#[test]
fn test_stmt_let_inferred_type() {
    let stmt = Stmt::new(
        StmtKind::Let {
            is_mutable: false,
            name: "x".to_string(),
            ty: Type::Inferred,
            init: Expr::new(ExprKind::IntLiteral(42), dummy_span()),
        },
        dummy_span(),
    );
    match stmt.kind {
        StmtKind::Let { ty, .. } => assert_eq!(ty, Type::Inferred),
        _ => panic!("Expected Let statement"),
    }
}

#[test]
fn test_stmt_assign() {
    let stmt = Stmt::new(
        StmtKind::Assign {
            name: "x".to_string(),
            value: Expr::new(ExprKind::IntLiteral(10), dummy_span()),
        },
        dummy_span(),
    );

    match stmt.kind {
        StmtKind::Assign { name, value } => {
            assert_eq!(name, "x");
            assert!(matches!(value.kind, ExprKind::IntLiteral(10)));
        }
        _ => panic!("Expected Assign statement"),
    }
}

#[test]
fn test_stmt_while() {
    let stmt = Stmt::new(
        StmtKind::While {
            condition: Expr::new(ExprKind::BoolLiteral(true), dummy_span()),
            body: vec![Stmt::new(StmtKind::Break, dummy_span())],
        },
        dummy_span(),
    );
    match stmt.kind {
        StmtKind::While { condition, body } => {
            assert!(matches!(condition.kind, ExprKind::BoolLiteral(true)));
            assert_eq!(body.len(), 1);
            assert!(matches!(body[0].kind, StmtKind::Break));
        }
        _ => panic!("Expected While statement"),
    }
}

#[test]
fn test_stmt_break_and_continue() {
    let break_stmt = Stmt::new(StmtKind::Break, dummy_span());
    assert!(matches!(break_stmt.kind, StmtKind::Break));

    let continue_stmt = Stmt::new(StmtKind::Continue, dummy_span());
    assert!(matches!(continue_stmt.kind, StmtKind::Continue));
}

#[test]
fn test_expr_int_literal() {
    let expr = Expr::new(ExprKind::IntLiteral(100), dummy_span());
    assert!(matches!(expr.kind, ExprKind::IntLiteral(100)));
}

#[test]
fn test_expr_is_integer_literal_int_literal() {
    let expr = Expr::new(ExprKind::IntLiteral(100), dummy_span());
    assert!(expr.is_integer_literal());
}

#[test]
fn test_expr_is_integer_literal_negated_int_literal() {
    let expr = Expr::new(
        ExprKind::UnaryOp {
            op: UnaryOperator::Neg,
            operand: Box::new(Expr::new(ExprKind::IntLiteral(1), dummy_span())),
        },
        dummy_span(),
    );
    assert!(expr.is_integer_literal());
}

#[test]
fn test_expr_is_integer_literal_non_literal() {
    let expr = Expr::new(ExprKind::Identifier("x".to_string()), dummy_span());
    assert!(!expr.is_integer_literal());
}

#[test]
fn test_infer_common_binary_operand_type_same_type() {
    let left = Expr::new(ExprKind::Identifier("x".to_string()), dummy_span());
    let right = Expr::new(ExprKind::Identifier("y".to_string()), dummy_span());
    let ty = Expr::infer_common_binary_operand_type(&left, &Type::I32, &right, &Type::I32);
    assert_eq!(ty, Some(Type::I32));
}

#[test]
fn test_infer_common_binary_operand_type_both_integer_literals() {
    let left = Expr::new(ExprKind::IntLiteral(1), dummy_span());
    let right = Expr::new(ExprKind::IntLiteral(2), dummy_span());
    let ty = Expr::infer_common_binary_operand_type(&left, &Type::I64, &right, &Type::I64);
    assert_eq!(ty, Some(Type::I64));
}

#[test]
fn test_infer_common_binary_operand_type_left_literal_right_i32() {
    let left = Expr::new(ExprKind::IntLiteral(1), dummy_span());
    let right = Expr::new(ExprKind::Identifier("x".to_string()), dummy_span());
    let ty = Expr::infer_common_binary_operand_type(&left, &Type::I64, &right, &Type::I32);
    assert_eq!(ty, Some(Type::I32));
}

#[test]
fn test_infer_common_binary_operand_type_right_literal_left_i32() {
    let left = Expr::new(ExprKind::Identifier("x".to_string()), dummy_span());
    let right = Expr::new(ExprKind::IntLiteral(1), dummy_span());
    let ty = Expr::infer_common_binary_operand_type(&left, &Type::I32, &right, &Type::I64);
    assert_eq!(ty, Some(Type::I32));
}

#[test]
fn test_infer_common_binary_operand_type_non_adaptable_mixed_integer_variables() {
    let left = Expr::new(ExprKind::Identifier("x".to_string()), dummy_span());
    let right = Expr::new(ExprKind::Identifier("y".to_string()), dummy_span());
    let ty = Expr::infer_common_binary_operand_type(&left, &Type::I32, &right, &Type::I64);
    assert_eq!(ty, None);
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
fn test_type_from_source_name_allows_primitives() {
    assert_eq!(Type::from_source_name("i8"), Some(Type::I8));
    assert_eq!(Type::from_source_name("i16"), Some(Type::I16));
    assert_eq!(Type::from_source_name("i32"), Some(Type::I32));
    assert_eq!(Type::from_source_name("i64"), Some(Type::I64));
    assert_eq!(Type::from_source_name("u8"), Some(Type::U8));
    assert_eq!(Type::from_source_name("u16"), Some(Type::U16));
    assert_eq!(Type::from_source_name("u32"), Some(Type::U32));
    assert_eq!(Type::from_source_name("u64"), Some(Type::U64));
    assert_eq!(Type::from_source_name("f32"), Some(Type::F32));
    assert_eq!(Type::from_source_name("f64"), Some(Type::F64));
    assert_eq!(Type::from_source_name("string"), Some(Type::String));
    assert_eq!(Type::from_source_name("bool"), Some(Type::Bool));
}

#[test]
fn test_type_from_source_name_supports_byte_alias() {
    assert_eq!(Type::from_source_name("byte"), Some(Type::U8));
}

#[test]
fn test_type_from_source_name_rejects_unknown() {
    assert_eq!(Type::from_source_name("int"), None);
}

#[test]
fn test_type_from_function_return_name_handles_void_and_types() {
    assert_eq!(Type::from_function_return_name("void"), Some(None));
    assert_eq!(
        Type::from_function_return_name("i64"),
        Some(Some(Type::I64))
    );
    assert_eq!(
        Type::from_function_return_name("string"),
        Some(Some(Type::String))
    );
    assert_eq!(
        Type::from_function_return_name("byte"),
        Some(Some(Type::U8))
    );
}

#[test]
fn test_type_from_function_return_name_rejects_unknown() {
    assert_eq!(Type::from_function_return_name("int"), None);
}

#[test]
fn test_type_is_integer() {
    assert!(Type::I32.is_integer());
    assert!(Type::I64.is_integer());
    assert!(!Type::String.is_integer());
    assert!(!Type::Bool.is_integer());
}

#[test]
fn test_type_inferred_display_is_internal_marker() {
    assert_eq!(Type::Inferred.to_string(), "<inferred>");
}

#[test]
fn test_type_inferred_is_not_numeric_or_resolved() {
    assert!(!Type::Inferred.is_integer());
    assert!(!Type::Inferred.is_signed_integer());
    assert!(!Type::Inferred.is_unsigned_integer());
    assert!(!Type::Inferred.is_float());
    assert!(!Type::Inferred.is_numeric());
    assert!(!Type::Inferred.is_resolved());
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
        params: vec![],
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
        params: vec![],
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
        params: vec![],
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
        params: vec![],
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
            params: vec![],
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
        "Program { imports: [], functions: [FnDef { visibility: Private, name: \"main\", params: [], return_type: \"void\", return_type_span: Span { start: 0, end: 0, line: 1, column: 1 }, body: [Stmt { kind: Expr(Expr { kind: StringLiteral(\"test\"), span: Span { start: 0, end: 0, line: 1, column: 1 } }), span: Span { start: 0, end: 0, line: 1, column: 1 } }], span: Span { start: 0, end: 0, line: 1, column: 1 } }] }"
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
