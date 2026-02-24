//! Unit tests for code generation.

use super::*;
use crate::ast::{
    BinaryOperator, Expr, ExprKind, FnDef, FnParam, Program, Stmt, StmtKind, Type, UnaryOperator,
    Visibility,
};
use crate::resolver::ResolvedModule;
use crate::token::Span;
use inkwell::context::Context;

fn dummy_span() -> Span {
    Span::new(0, 0, 1, 1)
}

/// Helper to create a Program with a main function
fn make_program(body: Vec<Stmt>) -> Program {
    Program {
        imports: vec![],
        functions: vec![FnDef {
            visibility: Visibility::Private,
            name: "main".to_string(),
            params: vec![],
            return_type: "void".to_string(),
            return_type_span: dummy_span(),
            body,
            span: dummy_span(),
        }],
    }
}

/// Helper to create an expression statement
fn expr_stmt(kind: ExprKind) -> Stmt {
    let expr = Expr::new(kind, dummy_span());
    Stmt::new(StmtKind::Expr(expr), dummy_span())
}

/// Helper to create a let statement
fn let_stmt(name: &str, ty: Type, init_kind: ExprKind) -> Stmt {
    let init = Expr::new(init_kind, dummy_span());
    Stmt::new(
        StmtKind::Let {
            is_mutable: false,
            name: name.to_string(),
            ty,
            init,
        },
        dummy_span(),
    )
}

/// Helper to create an empty program (no imports, no functions).
fn empty_program() -> Program {
    Program {
        imports: vec![],
        functions: vec![],
    }
}

fn collect_user_function_signatures(module: &inkwell::module::Module<'_>) -> Vec<(String, String)> {
    let mut signatures = module
        .get_functions()
        .filter_map(|function| {
            let llvm_name = function.get_name().to_str().unwrap();
            if builtins::BUILTIN_NAMES.contains(&llvm_name) {
                return None;
            }

            Some((
                user_facing_function_name(llvm_name).to_string(),
                function.get_type().print_to_string().to_string(),
            ))
        })
        .collect::<Vec<_>>();
    signatures.sort();
    signatures
}

#[test]
fn test_codegen_new() {
    let context = Context::create();
    let codegen = Codegen::new(&context, "test_module");
    assert_eq!(codegen.module.get_name().to_str().unwrap(), "test_module");
}

#[test]
fn test_compile_empty_main() {
    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");

    let program = make_program(vec![]);
    codegen
        .compile(&program)
        .expect("Empty main should compile");

    assert!(codegen.module.get_function("main").is_some());
}

#[test]
fn test_compile_while_with_break_and_continue() {
    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");

    let program = make_program(vec![Stmt::new(
        StmtKind::While {
            condition: Expr::new(ExprKind::BoolLiteral(true), dummy_span()),
            body: vec![
                Stmt::new(
                    StmtKind::If {
                        condition: Expr::new(ExprKind::BoolLiteral(false), dummy_span()),
                        then_branch: vec![Stmt::new(StmtKind::Continue, dummy_span())],
                        else_branch: None,
                    },
                    dummy_span(),
                ),
                Stmt::new(StmtKind::Break, dummy_span()),
            ],
        },
        dummy_span(),
    )]);

    codegen
        .compile(&program)
        .expect("while program should compile");
}

#[test]
fn test_compile_non_void_function_with_while_true_return_compiles() {
    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");

    let program = Program {
        imports: vec![],
        functions: vec![
            FnDef {
                visibility: Visibility::Private,
                name: "foo".to_string(),
                params: vec![],
                return_type: "i64".to_string(),
                return_type_span: dummy_span(),
                body: vec![Stmt::new(
                    StmtKind::While {
                        condition: Expr::new(ExprKind::BoolLiteral(true), dummy_span()),
                        body: vec![Stmt::new(
                            StmtKind::Return(Some(Expr::new(
                                ExprKind::IntLiteral(1),
                                dummy_span(),
                            ))),
                            dummy_span(),
                        )],
                    },
                    dummy_span(),
                )],
                span: dummy_span(),
            },
            FnDef {
                visibility: Visibility::Private,
                name: "main".to_string(),
                params: vec![],
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![],
                span: dummy_span(),
            },
        ],
    };

    codegen
        .compile(&program)
        .expect("`while true` with return should compile for non-void functions");
}

#[test]
fn test_compile_non_void_function_with_if_true_return_compiles() {
    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");

    let program = Program {
        imports: vec![],
        functions: vec![
            FnDef {
                visibility: Visibility::Private,
                name: "foo".to_string(),
                params: vec![],
                return_type: "i64".to_string(),
                return_type_span: dummy_span(),
                body: vec![Stmt::new(
                    StmtKind::If {
                        condition: Expr::new(ExprKind::BoolLiteral(true), dummy_span()),
                        then_branch: vec![Stmt::new(
                            StmtKind::Return(Some(Expr::new(
                                ExprKind::IntLiteral(1),
                                dummy_span(),
                            ))),
                            dummy_span(),
                        )],
                        else_branch: None,
                    },
                    dummy_span(),
                )],
                span: dummy_span(),
            },
            FnDef {
                visibility: Visibility::Private,
                name: "main".to_string(),
                params: vec![],
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![],
                span: dummy_span(),
            },
        ],
    };

    codegen
        .compile(&program)
        .expect("`if true` with return should compile for non-void functions");
}

#[test]
fn test_compile_non_void_function_with_if_not_false_return_compiles() {
    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");

    let program = Program {
        imports: vec![],
        functions: vec![
            FnDef {
                visibility: Visibility::Private,
                name: "foo".to_string(),
                params: vec![],
                return_type: "i64".to_string(),
                return_type_span: dummy_span(),
                body: vec![Stmt::new(
                    StmtKind::If {
                        condition: Expr::new(
                            ExprKind::UnaryOp {
                                op: UnaryOperator::Not,
                                operand: Box::new(Expr::new(
                                    ExprKind::BoolLiteral(false),
                                    dummy_span(),
                                )),
                            },
                            dummy_span(),
                        ),
                        then_branch: vec![Stmt::new(
                            StmtKind::Return(Some(Expr::new(
                                ExprKind::IntLiteral(1),
                                dummy_span(),
                            ))),
                            dummy_span(),
                        )],
                        else_branch: None,
                    },
                    dummy_span(),
                )],
                span: dummy_span(),
            },
            FnDef {
                visibility: Visibility::Private,
                name: "main".to_string(),
                params: vec![],
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![],
                span: dummy_span(),
            },
        ],
    };

    codegen
        .compile(&program)
        .expect("`if !false` with return should compile for non-void functions");
}

#[test]
fn test_compile_non_void_function_with_if_true_and_true_return_compiles() {
    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");

    let program = Program {
        imports: vec![],
        functions: vec![
            FnDef {
                visibility: Visibility::Private,
                name: "foo".to_string(),
                params: vec![],
                return_type: "i64".to_string(),
                return_type_span: dummy_span(),
                body: vec![Stmt::new(
                    StmtKind::If {
                        condition: Expr::new(
                            ExprKind::BinaryOp {
                                left: Box::new(Expr::new(
                                    ExprKind::BoolLiteral(true),
                                    dummy_span(),
                                )),
                                op: BinaryOperator::LogicalAnd,
                                right: Box::new(Expr::new(
                                    ExprKind::BoolLiteral(true),
                                    dummy_span(),
                                )),
                            },
                            dummy_span(),
                        ),
                        then_branch: vec![Stmt::new(
                            StmtKind::Return(Some(Expr::new(
                                ExprKind::IntLiteral(1),
                                dummy_span(),
                            ))),
                            dummy_span(),
                        )],
                        else_branch: None,
                    },
                    dummy_span(),
                )],
                span: dummy_span(),
            },
            FnDef {
                visibility: Visibility::Private,
                name: "main".to_string(),
                params: vec![],
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![],
                span: dummy_span(),
            },
        ],
    };

    codegen
        .compile(&program)
        .expect("`if true && true` with return should compile for non-void functions");
}

#[test]
fn test_compile_non_void_function_with_if_false_else_return_compiles() {
    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");

    let program = Program {
        imports: vec![],
        functions: vec![
            FnDef {
                visibility: Visibility::Private,
                name: "foo".to_string(),
                params: vec![],
                return_type: "i64".to_string(),
                return_type_span: dummy_span(),
                body: vec![Stmt::new(
                    StmtKind::If {
                        condition: Expr::new(ExprKind::BoolLiteral(false), dummy_span()),
                        then_branch: vec![],
                        else_branch: Some(vec![Stmt::new(
                            StmtKind::Return(Some(Expr::new(
                                ExprKind::IntLiteral(1),
                                dummy_span(),
                            ))),
                            dummy_span(),
                        )]),
                    },
                    dummy_span(),
                )],
                span: dummy_span(),
            },
            FnDef {
                visibility: Visibility::Private,
                name: "main".to_string(),
                params: vec![],
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![],
                span: dummy_span(),
            },
        ],
    };

    codegen
        .compile(&program)
        .expect("`if false` with else return should compile for non-void functions");
}

#[test]
fn test_compile_missing_return_error_uses_source_function_name() {
    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");

    let program = Program {
        imports: vec![],
        functions: vec![
            FnDef {
                visibility: Visibility::Private,
                name: "foo".to_string(),
                params: vec![],
                return_type: "i64".to_string(),
                return_type_span: dummy_span(),
                body: vec![],
                span: dummy_span(),
            },
            FnDef {
                visibility: Visibility::Private,
                name: "main".to_string(),
                params: vec![],
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![],
                span: dummy_span(),
            },
        ],
    };

    let err = codegen
        .compile(&program)
        .expect_err("non-void function without return should produce internal error");
    assert_eq!(err.kind(), CodegenErrorKind::InternalError);
    assert_eq!(
        err.message(),
        "Internal error: function 'foo' with return type 'i64' reached end without return. Semantic analysis should have rejected this. This is a compiler bug."
    );
    assert!(
        !err.message().contains("_L"),
        "internal diagnostics must not expose mangled names: {}",
        err.message()
    );
}

#[test]
fn test_compile_invalid_return_type_reports_internal_error_at_return_type_span() {
    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");

    let invalid_return_type_span = Span::new(0, 0, 1, 16);
    let program = Program {
        imports: vec![],
        functions: vec![
            FnDef {
                visibility: Visibility::Private,
                name: "helper".to_string(),
                params: vec![],
                return_type: "int".to_string(),
                return_type_span: invalid_return_type_span,
                body: vec![],
                span: dummy_span(),
            },
            FnDef {
                visibility: Visibility::Private,
                name: "main".to_string(),
                params: vec![],
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![],
                span: dummy_span(),
            },
        ],
    };

    let err = codegen
        .compile(&program)
        .expect_err("invalid return type must fail in codegen declaration path");
    assert_eq!(err.kind(), CodegenErrorKind::InternalError);
    assert_eq!(
        err.message(),
        "Internal error: unsupported function return type 'int' in codegen. Semantic analysis should have rejected this. This is a compiler bug."
    );
    assert_eq!(err.span(), Some(invalid_return_type_span));
}

#[test]
fn test_compile_if_condition_non_bool_returns_internal_error() {
    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");

    let program = make_program(vec![Stmt::new(
        StmtKind::If {
            condition: Expr::new(ExprKind::StringLiteral("oops".to_string()), dummy_span()),
            then_branch: vec![],
            else_branch: None,
        },
        dummy_span(),
    )]);

    let result = codegen.compile(&program);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), CodegenErrorKind::InternalError);
    assert_eq!(
        err.message(),
        "Internal error: string literal used as 'bool' value in codegen. Semantic analysis should have caught this. This is a compiler bug."
    );
}

#[test]
fn test_compile_while_condition_non_bool_returns_internal_error() {
    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");

    let program = make_program(vec![Stmt::new(
        StmtKind::While {
            condition: Expr::new(ExprKind::StringLiteral("oops".to_string()), dummy_span()),
            body: vec![],
        },
        dummy_span(),
    )]);

    let result = codegen.compile(&program);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), CodegenErrorKind::InternalError);
    assert_eq!(
        err.message(),
        "Internal error: string literal used as 'bool' value in codegen. Semantic analysis should have caught this. This is a compiler bug."
    );
}

#[test]
fn test_compile_println() {
    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");

    let program = make_program(vec![expr_stmt(ExprKind::Call {
        callee: "println".to_string(),
        args: vec![Expr::new(
            ExprKind::StringLiteral("hello".to_string()),
            dummy_span(),
        )],
    })]);

    codegen
        .compile(&program)
        .expect("println program should compile");

    assert!(codegen.module.get_function("lak_println").is_some());
}

#[test]
fn test_compile_single_file_user_functions_are_mangled() {
    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");

    let program = Program {
        imports: vec![],
        functions: vec![
            FnDef {
                visibility: Visibility::Private,
                name: "helper".to_string(),
                params: vec![],
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![],
                span: dummy_span(),
            },
            FnDef {
                visibility: Visibility::Private,
                name: "main".to_string(),
                params: vec![],
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![expr_stmt(ExprKind::Call {
                    callee: "helper".to_string(),
                    args: vec![],
                })],
                span: dummy_span(),
            },
        ],
    };

    codegen
        .compile(&program)
        .expect("single-file functions should compile");

    assert!(codegen.module.get_function("main").is_some());
    assert!(codegen.module.get_function("helper").is_none());
    assert!(codegen.module.get_function("_L5_entry_helper").is_some());
}

#[test]
fn test_compile_single_file_function_with_parameters() {
    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");

    let program = Program {
        imports: vec![],
        functions: vec![
            FnDef {
                visibility: Visibility::Private,
                name: "helper".to_string(),
                params: vec![FnParam {
                    name: "name".to_string(),
                    ty: Type::String,
                    span: dummy_span(),
                }],
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![expr_stmt(ExprKind::Call {
                    callee: "println".to_string(),
                    args: vec![Expr::new(
                        ExprKind::Identifier("name".to_string()),
                        dummy_span(),
                    )],
                })],
                span: dummy_span(),
            },
            FnDef {
                visibility: Visibility::Private,
                name: "main".to_string(),
                params: vec![],
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![expr_stmt(ExprKind::Call {
                    callee: "helper".to_string(),
                    args: vec![Expr::new(
                        ExprKind::StringLiteral("hello".to_string()),
                        dummy_span(),
                    )],
                })],
                span: dummy_span(),
            },
        ],
    };

    codegen
        .compile(&program)
        .expect("single-file parameterized functions should compile");

    let helper = codegen.module.get_function("_L5_entry_helper").unwrap();
    assert_eq!(helper.count_params(), 1);
}

#[test]
fn test_compile_multiple_println() {
    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");

    let program = make_program(vec![
        expr_stmt(ExprKind::Call {
            callee: "println".to_string(),
            args: vec![Expr::new(
                ExprKind::StringLiteral("first".to_string()),
                dummy_span(),
            )],
        }),
        expr_stmt(ExprKind::Call {
            callee: "println".to_string(),
            args: vec![Expr::new(
                ExprKind::StringLiteral("second".to_string()),
                dummy_span(),
            )],
        }),
    ]);

    codegen
        .compile(&program)
        .expect("Multiple println program should compile");
}

#[test]
fn test_compile_println_with_escape_sequences() {
    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");

    let program = make_program(vec![expr_stmt(ExprKind::Call {
        callee: "println".to_string(),
        args: vec![Expr::new(
            ExprKind::StringLiteral("hello\nworld\t!".to_string()),
            dummy_span(),
        )],
    })]);

    codegen
        .compile(&program)
        .expect("Escape sequences program should compile");
}

#[test]
fn test_write_object_file() {
    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");

    let program = make_program(vec![expr_stmt(ExprKind::Call {
        callee: "println".to_string(),
        args: vec![Expr::new(
            ExprKind::StringLiteral("test".to_string()),
            dummy_span(),
        )],
    })]);

    codegen.compile(&program).unwrap();

    let temp_dir = tempfile::tempdir().unwrap();
    let object_path = temp_dir.path().join("test.o");

    let result = codegen.write_object_file(&object_path);
    assert!(result.is_ok());
    assert!(object_path.exists());
}

#[test]
fn test_module_name() {
    let context = Context::create();
    let codegen = Codegen::new(&context, "my_custom_module");
    assert_eq!(
        codegen.module.get_name().to_str().unwrap(),
        "my_custom_module"
    );
}

#[test]
fn test_main_function_signature() {
    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");

    let program = make_program(vec![]);
    codegen.compile(&program).unwrap();

    let main_fn = codegen.module.get_function("main").unwrap();
    // main returns i32
    assert!(main_fn.get_type().get_return_type().is_some());
    // main takes no arguments
    assert_eq!(main_fn.count_params(), 0);
}

#[test]
fn test_main_function_not_first() {
    // Verify that main is found even when it's not the first function
    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");

    let program = Program {
        imports: vec![],
        functions: vec![
            FnDef {
                visibility: Visibility::Private,
                name: "helper".to_string(),
                params: vec![],
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![],
                span: dummy_span(),
            },
            FnDef {
                visibility: Visibility::Private,
                name: "main".to_string(),
                params: vec![],
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![],
                span: dummy_span(),
            },
        ],
    };

    codegen
        .compile(&program)
        .expect("Should find main even if not first");
    assert!(codegen.module.get_function("main").is_some());
}

// ===================
// Let statement tests
// ===================

#[test]
fn test_compile_let_i32() {
    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");

    let program = make_program(vec![let_stmt("x", Type::I32, ExprKind::IntLiteral(42))]);

    codegen
        .compile(&program)
        .expect("Let statement should compile");
}

#[test]
fn test_compile_let_i64() {
    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");

    let program = make_program(vec![let_stmt("y", Type::I64, ExprKind::IntLiteral(100))]);

    codegen
        .compile(&program)
        .expect("Let i64 statement should compile");
}

#[test]
fn test_compile_multiple_let_statements() {
    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");

    let program = make_program(vec![
        let_stmt("a", Type::I32, ExprKind::IntLiteral(1)),
        let_stmt("b", Type::I32, ExprKind::IntLiteral(2)),
        let_stmt("c", Type::I64, ExprKind::IntLiteral(3)),
    ]);

    codegen
        .compile(&program)
        .expect("Multiple let statements should compile");
}

#[test]
fn test_compile_let_with_variable_reference() {
    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");

    let program = make_program(vec![
        let_stmt("x", Type::I32, ExprKind::IntLiteral(42)),
        let_stmt("y", Type::I32, ExprKind::Identifier("x".to_string())),
    ]);

    codegen
        .compile(&program)
        .expect("Let with variable reference should compile");
}

#[test]
fn test_compile_let_mixed_with_println() {
    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");

    let program = make_program(vec![
        let_stmt("x", Type::I32, ExprKind::IntLiteral(42)),
        expr_stmt(ExprKind::Call {
            callee: "println".to_string(),
            args: vec![Expr::new(
                ExprKind::StringLiteral("hello".to_string()),
                dummy_span(),
            )],
        }),
        let_stmt("y", Type::I64, ExprKind::IntLiteral(100)),
    ]);

    codegen
        .compile(&program)
        .expect("Mixed let and println should compile");
}

// ===================
// CodegenError tests
// ===================

#[test]
fn test_codegen_error_with_span() {
    let span = Span::new(10, 20, 3, 5);
    let err = CodegenError::new(CodegenErrorKind::InternalError, "test error", span);
    assert!(err.span().is_some());
    assert_eq!(err.span().unwrap().line, 3);
    assert_eq!(err.span().unwrap().column, 5);
    assert_eq!(err.kind(), CodegenErrorKind::InternalError);
    let display = format!("{}", err);
    assert_eq!(display, "3:5: test error");
}

#[test]
fn test_codegen_error_without_span() {
    let err = CodegenError::without_span(CodegenErrorKind::TargetError, "test error");
    assert!(err.span().is_none());
    assert_eq!(err.kind(), CodegenErrorKind::TargetError);
    let display = format!("{}", err);
    assert_eq!(display, "test error");
}

#[test]
fn test_codegen_error_kinds() {
    // Verify error kinds are distinct
    assert_ne!(
        CodegenErrorKind::InternalError,
        CodegenErrorKind::TargetError
    );
    assert_ne!(
        CodegenErrorKind::InternalError,
        CodegenErrorKind::InvalidModulePath
    );
    assert_ne!(
        CodegenErrorKind::TargetError,
        CodegenErrorKind::InvalidModulePath
    );
}

#[test]
fn test_codegen_error_short_message_internal() {
    let err = CodegenError::new(CodegenErrorKind::InternalError, "test error", dummy_span());
    assert_eq!(err.short_message(), "Internal error");
}

#[test]
fn test_codegen_error_short_message_target() {
    let err = CodegenError::without_span(CodegenErrorKind::TargetError, "test error");
    assert_eq!(err.short_message(), "Target error");
}

// =================================
// Error constructor tests (target)
// =================================

#[test]
fn test_target_init_failed_constructor() {
    let err = CodegenError::target_init_failed("initialization error");
    assert_eq!(err.kind(), CodegenErrorKind::TargetError);
    assert!(err.span().is_none());
    assert_eq!(
        err.message(),
        "Failed to initialize native target: initialization error"
    );
}

#[test]
fn test_target_from_triple_failed_constructor() {
    let err = CodegenError::target_from_triple_failed("x86_64-unknown-linux", "unsupported");
    assert_eq!(err.kind(), CodegenErrorKind::TargetError);
    assert!(err.span().is_none());
    assert_eq!(
        err.message(),
        "Failed to get target for triple 'x86_64-unknown-linux': unsupported"
    );
}

#[test]
fn test_target_cpu_invalid_utf8_constructor() {
    let err = CodegenError::target_cpu_invalid_utf8();
    assert_eq!(err.kind(), CodegenErrorKind::TargetError);
    assert!(err.span().is_none());
    assert_eq!(err.message(), "CPU name contains invalid UTF-8");
}

#[test]
fn test_target_machine_creation_failed_constructor() {
    let err = CodegenError::target_machine_creation_failed("x86_64", "generic");
    assert_eq!(err.kind(), CodegenErrorKind::TargetError);
    assert!(err.span().is_none());
    assert_eq!(
        err.message(),
        "Failed to create target machine for triple 'x86_64', CPU 'generic'. This may indicate an unsupported platform or LLVM configuration issue."
    );
}

// ===================================
// Error constructor tests (internal)
// ===================================

#[test]
fn test_internal_variable_not_found_constructor() {
    let err = CodegenError::internal_variable_not_found("x", dummy_span());
    assert_eq!(err.kind(), CodegenErrorKind::InternalError);
    assert!(err.span().is_some());
    assert_eq!(
        err.message(),
        "Internal error: undefined variable 'x' in codegen. Semantic analysis should have caught this. This is a compiler bug."
    );
}

#[test]
fn test_internal_function_not_found_constructor() {
    let err = CodegenError::internal_function_not_found("foo", dummy_span());
    assert_eq!(err.kind(), CodegenErrorKind::InternalError);
    assert!(err.span().is_some());
    assert_eq!(
        err.message(),
        "Internal error: undefined function 'foo' in codegen. Semantic analysis should have caught this. This is a compiler bug."
    );
}

#[test]
fn test_internal_println_arg_count_constructor() {
    let err = CodegenError::internal_println_arg_count(3, dummy_span());
    assert_eq!(err.kind(), CodegenErrorKind::InternalError);
    assert!(err.span().is_some());
    assert_eq!(
        err.message(),
        "Internal error: println expects 1 argument, but got 3 in codegen. Semantic analysis should have caught this. This is a compiler bug."
    );
}

#[test]
fn test_internal_variable_type_mismatch_constructor() {
    let err = CodegenError::internal_variable_type_mismatch("x", "i32", "string", dummy_span());
    assert_eq!(err.kind(), CodegenErrorKind::InternalError);
    assert!(err.span().is_some());
    assert_eq!(
        err.message(),
        "Internal error: type mismatch for variable 'x' in codegen. Expected 'i32', but variable has type 'string'. Semantic analysis should have caught this. This is a compiler bug."
    );
}

#[test]
fn test_internal_builtin_not_found_constructor() {
    let err = CodegenError::internal_builtin_not_found("lak_println");
    assert_eq!(err.kind(), CodegenErrorKind::InternalError);
    assert!(err.span().is_none());
    assert_eq!(
        err.message(),
        "Internal error: lak_println function not found. This is a compiler bug."
    );
}

#[test]
fn test_internal_return_build_failed_constructor() {
    let err = CodegenError::internal_return_build_failed("main", "LLVM error");
    assert_eq!(err.kind(), CodegenErrorKind::InternalError);
    assert!(err.span().is_none());
    assert_eq!(
        err.message(),
        "Internal error: failed to build return for function 'main'. This is a compiler bug: LLVM error"
    );
}

#[test]
fn test_internal_break_outside_loop_constructor() {
    let err = CodegenError::internal_break_outside_loop(dummy_span());
    assert_eq!(err.kind(), CodegenErrorKind::InternalError);
    assert!(err.span().is_some());
    assert_eq!(
        err.message(),
        "Internal error: break statement used outside loop during codegen. Semantic analysis should have rejected this. This is a compiler bug."
    );
}

#[test]
fn test_internal_continue_outside_loop_constructor() {
    let err = CodegenError::internal_continue_outside_loop(dummy_span());
    assert_eq!(err.kind(), CodegenErrorKind::InternalError);
    assert!(err.span().is_some());
    assert_eq!(
        err.message(),
        "Internal error: continue statement used outside loop during codegen. Semantic analysis should have rejected this. This is a compiler bug."
    );
}

#[test]
fn test_internal_no_loop_control_scope_constructor() {
    let err = CodegenError::internal_no_loop_control_scope(dummy_span());
    assert_eq!(err.kind(), CodegenErrorKind::InternalError);
    assert!(err.span().is_some());
    assert_eq!(
        err.message(),
        "Internal error: attempted to pop loop control scope when no loop is active in codegen. This is a compiler bug."
    );
}

// ====================
// get_expr_type tests
// ====================

#[test]
fn test_get_expr_type_int_literal() {
    let context = Context::create();
    let codegen = Codegen::new(&context, "test");

    let expr = Expr::new(ExprKind::IntLiteral(42), dummy_span());
    let result = codegen.get_expr_type(&expr);
    assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
    assert_eq!(result.unwrap(), Type::I64);
}

#[test]
fn test_get_expr_type_string_literal() {
    let context = Context::create();
    let codegen = Codegen::new(&context, "test");

    let expr = Expr::new(ExprKind::StringLiteral("hello".to_string()), dummy_span());
    let result = codegen.get_expr_type(&expr);
    assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
    assert_eq!(result.unwrap(), Type::String);
}

#[test]
fn test_get_expr_type_float_literal() {
    let context = Context::create();
    let codegen = Codegen::new(&context, "test");

    let expr = Expr::new(ExprKind::FloatLiteral(2.5), dummy_span());
    let result = codegen.get_expr_type(&expr);
    assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
    assert_eq!(result.unwrap(), Type::F64);
}

#[test]
fn test_get_expr_type_undefined_identifier() {
    let context = Context::create();
    let codegen = Codegen::new(&context, "test");

    let expr = Expr::new(
        ExprKind::Identifier("undefined_var".to_string()),
        dummy_span(),
    );
    let result = codegen.get_expr_type(&expr);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), CodegenErrorKind::InternalError);
    assert_eq!(
        err.message(),
        "Internal error: undefined variable 'undefined_var' in codegen. Semantic analysis should have caught this. This is a compiler bug."
    );
}

#[test]
fn test_get_expr_type_function_call() {
    let context = Context::create();
    let codegen = Codegen::new(&context, "test");

    let expr = Expr::new(
        ExprKind::Call {
            callee: "some_function".to_string(),
            args: vec![],
        },
        dummy_span(),
    );
    let result = codegen.get_expr_type(&expr);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), CodegenErrorKind::InternalError);
    assert_eq!(
        err.message(),
        "Internal error: undefined function 'some_function' in codegen. Semantic analysis should have caught this. This is a compiler bug."
    );
}

#[test]
fn test_get_expr_type_defined_i32_variable() {
    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");

    // Compile a program that defines an i32 variable
    let program = make_program(vec![let_stmt("x", Type::I32, ExprKind::IntLiteral(42))]);
    codegen
        .compile(&program)
        .expect("Program should compile successfully");

    // Now test get_expr_type with the defined variable
    let expr = Expr::new(ExprKind::Identifier("x".to_string()), dummy_span());
    let result = codegen.get_expr_type(&expr);
    assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
    assert_eq!(result.unwrap(), Type::I32);
}

#[test]
fn test_get_expr_type_defined_i64_variable() {
    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");

    // Compile a program that defines an i64 variable
    let program = make_program(vec![let_stmt("y", Type::I64, ExprKind::IntLiteral(100))]);
    codegen
        .compile(&program)
        .expect("Program should compile successfully");

    // Now test get_expr_type with the defined variable
    let expr = Expr::new(ExprKind::Identifier("y".to_string()), dummy_span());
    let result = codegen.get_expr_type(&expr);
    assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
    assert_eq!(result.unwrap(), Type::I64);
}

#[test]
fn test_get_expr_type_mixed_float_f32_f64_promotes_to_f64() {
    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");

    let program = make_program(vec![
        let_stmt("a", Type::F32, ExprKind::FloatLiteral(1.0)),
        let_stmt("b", Type::F64, ExprKind::FloatLiteral(2.0)),
    ]);
    codegen
        .compile(&program)
        .expect("Program should compile successfully");

    let expr = Expr::new(
        ExprKind::BinaryOp {
            left: Box::new(Expr::new(
                ExprKind::Identifier("a".to_string()),
                dummy_span(),
            )),
            op: crate::ast::BinaryOperator::Add,
            right: Box::new(Expr::new(
                ExprKind::Identifier("b".to_string()),
                dummy_span(),
            )),
        },
        dummy_span(),
    );
    let result = codegen.get_expr_type(&expr);
    assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
    assert_eq!(result.unwrap(), Type::F64);
}

#[test]
fn test_get_expr_type_binary_literal_plus_i32_variable() {
    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");

    let program = make_program(vec![let_stmt("x", Type::I32, ExprKind::IntLiteral(42))]);
    codegen
        .compile(&program)
        .expect("Program should compile successfully");

    let expr = Expr::new(
        ExprKind::BinaryOp {
            left: Box::new(Expr::new(ExprKind::IntLiteral(1), dummy_span())),
            op: crate::ast::BinaryOperator::Add,
            right: Box::new(Expr::new(
                ExprKind::Identifier("x".to_string()),
                dummy_span(),
            )),
        },
        dummy_span(),
    );
    let result = codegen.get_expr_type(&expr);
    assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
    assert_eq!(result.unwrap(), Type::I32);
}

#[test]
fn test_get_expr_type_binary_literal_plus_i64_variable() {
    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");

    let program = make_program(vec![let_stmt("y", Type::I64, ExprKind::IntLiteral(42))]);
    codegen
        .compile(&program)
        .expect("Program should compile successfully");

    let expr = Expr::new(
        ExprKind::BinaryOp {
            left: Box::new(Expr::new(ExprKind::IntLiteral(1), dummy_span())),
            op: crate::ast::BinaryOperator::Add,
            right: Box::new(Expr::new(
                ExprKind::Identifier("y".to_string()),
                dummy_span(),
            )),
        },
        dummy_span(),
    );
    let result = codegen.get_expr_type(&expr);
    assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
    assert_eq!(result.unwrap(), Type::I64);
}

#[test]
fn test_get_expr_type_binary_mixed_i32_i64_variables_returns_internal_error() {
    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");

    let program = make_program(vec![
        let_stmt("x", Type::I32, ExprKind::IntLiteral(1)),
        let_stmt("y", Type::I64, ExprKind::IntLiteral(2)),
    ]);
    codegen
        .compile(&program)
        .expect("Program should compile successfully");

    let expr = Expr::new(
        ExprKind::BinaryOp {
            left: Box::new(Expr::new(
                ExprKind::Identifier("x".to_string()),
                dummy_span(),
            )),
            op: crate::ast::BinaryOperator::Add,
            right: Box::new(Expr::new(
                ExprKind::Identifier("y".to_string()),
                dummy_span(),
            )),
        },
        dummy_span(),
    );
    let result = codegen.get_expr_type(&expr);
    assert!(result.is_err(), "Expected Err, got: {:?}", result);
    let err = result.unwrap_err();
    assert_eq!(err.kind(), CodegenErrorKind::InternalError);
    assert_eq!(
        err.message(),
        "Internal error: binary operands have incompatible types 'i32' and 'i64' after literal adaptation in codegen. Semantic analysis should have rejected this. This is a compiler bug."
    );
}

#[test]
fn test_get_expr_type_binary_negative_literal_plus_i32_variable() {
    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");

    let program = make_program(vec![let_stmt("x", Type::I32, ExprKind::IntLiteral(42))]);
    codegen
        .compile(&program)
        .expect("Program should compile successfully");

    let expr = Expr::new(
        ExprKind::BinaryOp {
            left: Box::new(Expr::new(
                ExprKind::UnaryOp {
                    op: UnaryOperator::Neg,
                    operand: Box::new(Expr::new(ExprKind::IntLiteral(1), dummy_span())),
                },
                dummy_span(),
            )),
            op: crate::ast::BinaryOperator::Add,
            right: Box::new(Expr::new(
                ExprKind::Identifier("x".to_string()),
                dummy_span(),
            )),
        },
        dummy_span(),
    );
    let result = codegen.get_expr_type(&expr);
    assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
    assert_eq!(result.unwrap(), Type::I32);
}

#[test]
fn test_get_expr_type_binary_negative_literal_plus_i64_variable() {
    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");

    let program = make_program(vec![let_stmt("y", Type::I64, ExprKind::IntLiteral(42))]);
    codegen
        .compile(&program)
        .expect("Program should compile successfully");

    let expr = Expr::new(
        ExprKind::BinaryOp {
            left: Box::new(Expr::new(
                ExprKind::UnaryOp {
                    op: UnaryOperator::Neg,
                    operand: Box::new(Expr::new(ExprKind::IntLiteral(1), dummy_span())),
                },
                dummy_span(),
            )),
            op: crate::ast::BinaryOperator::Add,
            right: Box::new(Expr::new(
                ExprKind::Identifier("y".to_string()),
                dummy_span(),
            )),
        },
        dummy_span(),
    );
    let result = codegen.get_expr_type(&expr);
    assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
    assert_eq!(result.unwrap(), Type::I64);
}

#[test]
fn test_get_expr_type_defined_string_variable() {
    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");

    // Compile a program that defines a string variable
    let program = make_program(vec![let_stmt(
        "s",
        Type::String,
        ExprKind::StringLiteral("hello".to_string()),
    )]);
    codegen
        .compile(&program)
        .expect("Program should compile successfully");

    // Now test get_expr_type with the defined variable
    let expr = Expr::new(ExprKind::Identifier("s".to_string()), dummy_span());
    let result = codegen.get_expr_type(&expr);
    assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
    assert_eq!(result.unwrap(), Type::String);
}

// ====================
// Unary operation tests
// ====================

#[test]
fn test_compile_unary_minus_i32() {
    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");

    let program = make_program(vec![let_stmt(
        "x",
        Type::I32,
        ExprKind::UnaryOp {
            op: UnaryOperator::Neg,
            operand: Box::new(Expr::new(ExprKind::IntLiteral(5), dummy_span())),
        },
    )]);

    codegen
        .compile(&program)
        .expect("Unary negation should compile");
}

#[test]
fn test_compile_unary_minus_i64() {
    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");

    let program = make_program(vec![let_stmt(
        "x",
        Type::I64,
        ExprKind::UnaryOp {
            op: UnaryOperator::Neg,
            operand: Box::new(Expr::new(ExprKind::IntLiteral(100), dummy_span())),
        },
    )]);

    codegen
        .compile(&program)
        .expect("Unary negation i64 should compile");
}

#[test]
fn test_compile_unary_minus_nested() {
    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");

    // --5 (double negation)
    let program = make_program(vec![let_stmt(
        "x",
        Type::I32,
        ExprKind::UnaryOp {
            op: UnaryOperator::Neg,
            operand: Box::new(Expr::new(
                ExprKind::UnaryOp {
                    op: UnaryOperator::Neg,
                    operand: Box::new(Expr::new(ExprKind::IntLiteral(5), dummy_span())),
                },
                dummy_span(),
            )),
        },
    )]);

    codegen
        .compile(&program)
        .expect("Nested unary negation should compile");
}

#[test]
fn test_compile_unary_minus_with_variable() {
    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");

    let program = make_program(vec![
        let_stmt("a", Type::I32, ExprKind::IntLiteral(10)),
        let_stmt(
            "b",
            Type::I32,
            ExprKind::UnaryOp {
                op: UnaryOperator::Neg,
                operand: Box::new(Expr::new(
                    ExprKind::Identifier("a".to_string()),
                    dummy_span(),
                )),
            },
        ),
    ]);

    codegen
        .compile(&program)
        .expect("Unary negation on variable should compile");
}

#[test]
fn test_get_expr_type_unary_minus() {
    let context = Context::create();
    let codegen = Codegen::new(&context, "test");

    // -42 should infer as i64 (same as integer literal)
    let expr = Expr::new(
        ExprKind::UnaryOp {
            op: UnaryOperator::Neg,
            operand: Box::new(Expr::new(ExprKind::IntLiteral(42), dummy_span())),
        },
        dummy_span(),
    );
    let result = codegen.get_expr_type(&expr);
    assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
    assert_eq!(result.unwrap(), Type::I64);
}

#[test]
fn test_get_expr_type_unary_minus_variable() {
    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");

    // Define an i32 variable
    let program = make_program(vec![let_stmt("x", Type::I32, ExprKind::IntLiteral(42))]);
    codegen
        .compile(&program)
        .expect("Program should compile successfully");

    // -x should infer as i32 (same as the variable type)
    let expr = Expr::new(
        ExprKind::UnaryOp {
            op: UnaryOperator::Neg,
            operand: Box::new(Expr::new(
                ExprKind::Identifier("x".to_string()),
                dummy_span(),
            )),
        },
        dummy_span(),
    );
    let result = codegen.get_expr_type(&expr);
    assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
    assert_eq!(result.unwrap(), Type::I32);
}

// ==================================
// Operation error constructor tests
// ==================================

#[test]
fn test_internal_binary_op_string_constructor() {
    let err = CodegenError::internal_binary_op_string(BinaryOperator::Add, dummy_span());
    assert_eq!(err.kind(), CodegenErrorKind::InternalError);
    assert!(err.span().is_some());
    assert_eq!(
        err.message(),
        "Internal error: binary operator '+' dispatched with non-numeric type in codegen. Semantic analysis should have caught this. This is a compiler bug."
    );
}

#[test]
fn test_internal_unary_op_string_constructor() {
    let err = CodegenError::internal_unary_op_string(UnaryOperator::Neg, dummy_span());
    assert_eq!(err.kind(), CodegenErrorKind::InternalError);
    assert!(err.span().is_some());
    assert_eq!(
        err.message(),
        "Internal error: unary operator '-' dispatched with non-signed-integer-or-float type in codegen. Semantic analysis should have caught this. This is a compiler bug."
    );
}

// ==================================
// Module compilation error constructor tests
// ==================================

#[test]
fn test_internal_entry_module_not_found_constructor() {
    let path = std::path::Path::new("/tmp/test.lak");
    let err = CodegenError::internal_entry_module_not_found(path);
    assert_eq!(err.kind(), CodegenErrorKind::InternalError);
    assert!(err.span().is_none());
    assert_eq!(
        err.message(),
        "Internal error: entry module '/tmp/test.lak' not found in module list. This is a compiler bug."
    );
}

#[test]
fn test_internal_import_path_not_resolved_constructor() {
    let err = CodegenError::internal_import_path_not_resolved("./utils", dummy_span());
    assert_eq!(err.kind(), CodegenErrorKind::InternalError);
    assert!(err.span().is_some());
    assert_eq!(
        err.message(),
        "Internal error: import path './utils' not found in resolved imports. This is a compiler bug."
    );
}

#[test]
fn test_internal_resolved_module_not_found_for_path_constructor() {
    let path = std::path::Path::new("/tmp/utils.lak");
    let err = CodegenError::internal_resolved_module_not_found_for_path(path, dummy_span());
    assert_eq!(err.kind(), CodegenErrorKind::InternalError);
    assert!(err.span().is_some());
    assert_eq!(
        err.message(),
        "Internal error: resolved module not found for path '/tmp/utils.lak'. This is a compiler bug."
    );
}

#[test]
fn test_internal_module_alias_not_found_constructor() {
    let err = CodegenError::internal_module_alias_not_found("u", dummy_span());
    assert_eq!(err.kind(), CodegenErrorKind::InternalError);
    assert!(err.span().is_some());
    assert_eq!(
        err.message(),
        "Internal error: module alias 'u' not found in alias map. This is a compiler bug."
    );
}

#[test]
fn test_internal_variable_alloca_failed_constructor() {
    let err = CodegenError::internal_variable_alloca_failed("x", "LLVM error", dummy_span());
    assert_eq!(err.kind(), CodegenErrorKind::InternalError);
    assert!(err.span().is_some());
    assert_eq!(
        err.message(),
        "Internal error: failed to allocate variable 'x'. This is a compiler bug: LLVM error"
    );
}

#[test]
fn test_internal_non_integer_value_constructor() {
    let err = CodegenError::internal_non_integer_value("binary", dummy_span());
    assert_eq!(err.kind(), CodegenErrorKind::InternalError);
    assert!(err.span().is_some());
    assert_eq!(
        err.message(),
        "Internal error: expected integer value in binary operation, but got non-integer. Semantic analysis should have caught this. This is a compiler bug."
    );
}

// ==================================
// wrap_in_unary_context tests
// ==================================

#[test]
fn test_wrap_in_unary_context_non_unary_error() {
    let base = CodegenError::internal_variable_not_found("x", dummy_span());
    let wrapped = CodegenError::wrap_in_unary_context(&base, UnaryOperator::Neg, dummy_span());
    assert_eq!(wrapped.kind(), CodegenErrorKind::InternalError);
    assert_eq!(
        wrapped.message(),
        "in unary '-' operation: Internal error: undefined variable 'x' in codegen. \
         Semantic analysis should have caught this. This is a compiler bug."
    );
}

#[test]
fn test_wrap_in_unary_context_already_unary_error() {
    let base = CodegenError::internal_unary_op_string(UnaryOperator::Neg, dummy_span());
    let wrapped = CodegenError::wrap_in_unary_context(&base, UnaryOperator::Neg, dummy_span());
    assert_eq!(wrapped.kind(), CodegenErrorKind::InternalError);
    // Should not double-wrap: message should be the original
    assert_eq!(wrapped.message(), base.message());
}

// ==================================
// mangle_name tests
// ==================================

#[test]
fn test_mangle_name_basic() {
    assert_eq!(mangle_name("utils", "greet"), "_L5_utils_greet");
}

#[test]
fn test_mangle_name_single_char_module() {
    assert_eq!(mangle_name("a", "b"), "_L1_a_b");
}

#[test]
fn test_mangle_name_with_underscores_in_function() {
    assert_eq!(mangle_name("a", "b__c"), "_L1_a_b__c");
}

#[test]
fn test_mangle_name_with_underscores_in_module() {
    assert_eq!(mangle_name("a__b", "c"), "_L4_a__b_c");
}

#[test]
fn test_mangle_name_no_collision_between_similar_names() {
    // Module "a" with function "b__c" should differ from module "a__b" with function "c"
    let name1 = mangle_name("a", "b__c");
    let name2 = mangle_name("a__b", "c");
    assert_ne!(
        name1, name2,
        "Length-prefix scheme should prevent collisions"
    );
}

#[test]
fn test_mangle_name_long_module() {
    assert_eq!(
        mangle_name("my_long_module", "func"),
        "_L14_my_long_module_func"
    );
}

#[test]
fn test_internal_non_pointer_value_constructor() {
    let err = CodegenError::internal_non_pointer_value("println_string load", dummy_span());
    assert_eq!(err.kind(), CodegenErrorKind::InternalError);
    assert!(err.span().is_some());
    assert_eq!(
        err.message(),
        "Internal error: expected pointer value in println_string load operation, but got non-pointer. Semantic analysis should have caught this. This is a compiler bug."
    );
}

#[test]
fn test_builtin_names_matches_declare_builtins() {
    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");
    codegen.declare_builtins();

    let mut declared: Vec<&str> = builtins::BUILTIN_NAMES.to_vec();
    declared.sort();

    let mut actual: Vec<String> = codegen
        .module
        .get_functions()
        .map(|f| f.get_name().to_str().unwrap().to_string())
        .collect();
    actual.sort();

    assert_eq!(
        declared,
        actual.iter().map(|s| s.as_str()).collect::<Vec<&str>>(),
        "BUILTIN_NAMES must match the functions declared by declare_builtins()"
    );
}

#[test]
fn test_compile_and_compile_modules_equivalent_for_single_entry_module() {
    let program = Program {
        imports: vec![],
        functions: vec![
            FnDef {
                visibility: Visibility::Private,
                name: "helper".to_string(),
                params: vec![FnParam {
                    name: "message".to_string(),
                    ty: Type::String,
                    span: dummy_span(),
                }],
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![expr_stmt(ExprKind::Call {
                    callee: "println".to_string(),
                    args: vec![Expr::new(
                        ExprKind::Identifier("message".to_string()),
                        dummy_span(),
                    )],
                })],
                span: dummy_span(),
            },
            FnDef {
                visibility: Visibility::Private,
                name: "main".to_string(),
                params: vec![],
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![expr_stmt(ExprKind::Call {
                    callee: "helper".to_string(),
                    args: vec![Expr::new(
                        ExprKind::StringLiteral("hello".to_string()),
                        dummy_span(),
                    )],
                })],
                span: dummy_span(),
            },
        ],
    };

    let context = Context::create();

    let mut single_codegen = Codegen::new(&context, "single");
    single_codegen
        .compile(&program)
        .expect("single-module compile should succeed");

    let entry_path = std::env::temp_dir().join("equivalent_main.lak");
    let entry_module = ResolvedModule::for_testing(
        entry_path.clone(),
        "equivalent_main".to_string(),
        program,
        "".to_string(),
    );
    let modules = [entry_module];

    let mut multi_codegen = Codegen::new(&context, "multi");
    multi_codegen
        .compile_modules(&modules, &entry_path)
        .expect("compile_modules with only entry module should succeed");

    assert_eq!(
        collect_user_function_signatures(&single_codegen.module),
        collect_user_function_signatures(&multi_codegen.module)
    );
}

// ==================================
// compile_modules tests
// ==================================

#[test]
fn test_compile_modules_basic() {
    use crate::ast::ImportDecl;
    use crate::resolver::ResolvedModule;

    let temp_dir = std::env::temp_dir();

    // Create an imported module with a public function that calls println
    let imported_program = Program {
        imports: vec![],
        functions: vec![FnDef {
            visibility: Visibility::Public,
            name: "greet".to_string(),
            params: vec![],
            return_type: "void".to_string(),
            return_type_span: dummy_span(),
            body: vec![expr_stmt(ExprKind::Call {
                callee: "println".to_string(),
                args: vec![Expr::new(
                    ExprKind::StringLiteral("hello from utils".to_string()),
                    dummy_span(),
                )],
            })],
            span: dummy_span(),
        }],
    };
    let imported_path = temp_dir.join("utils.lak");
    let imported_module = ResolvedModule::for_testing(
        imported_path.clone(),
        "utils".to_string(),
        imported_program,
        "".to_string(),
    );

    // Create entry module that imports and calls utils.greet()
    let entry_program = Program {
        imports: vec![ImportDecl {
            path: "./utils".to_string(),
            alias: None,
            span: dummy_span(),
        }],
        functions: vec![FnDef {
            visibility: Visibility::Private,
            name: "main".to_string(),
            params: vec![],
            return_type: "void".to_string(),
            return_type_span: dummy_span(),
            body: vec![expr_stmt(ExprKind::ModuleCall {
                module: "utils".to_string(),
                function: "greet".to_string(),
                args: vec![],
            })],
            span: dummy_span(),
        }],
    };
    let entry_path = temp_dir.join("main.lak");
    let mut entry_module = ResolvedModule::for_testing(
        entry_path.clone(),
        "main".to_string(),
        entry_program,
        "".to_string(),
    );
    entry_module.add_resolved_import_for_testing("./utils".to_string(), imported_path);

    let modules = [imported_module, entry_module];

    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");
    codegen
        .compile_modules(&modules, &entry_path)
        .expect("compile_modules should succeed");

    // Verify main exists
    assert!(codegen.module.get_function("main").is_some());
    // Verify mangled function uses path-based prefix
    assert!(codegen.module.get_function("_L5_utils_greet").is_some());
}

#[test]
fn test_compile_modules_function_call_with_arguments() {
    use crate::ast::ImportDecl;
    use crate::resolver::ResolvedModule;

    let temp_dir = std::env::temp_dir();

    let imported_program = Program {
        imports: vec![],
        functions: vec![FnDef {
            visibility: Visibility::Public,
            name: "greet".to_string(),
            params: vec![FnParam {
                name: "name".to_string(),
                ty: Type::String,
                span: dummy_span(),
            }],
            return_type: "void".to_string(),
            return_type_span: dummy_span(),
            body: vec![expr_stmt(ExprKind::Call {
                callee: "println".to_string(),
                args: vec![Expr::new(
                    ExprKind::Identifier("name".to_string()),
                    dummy_span(),
                )],
            })],
            span: dummy_span(),
        }],
    };
    let imported_path = temp_dir.join("utils_with_args.lak");
    let imported_module = ResolvedModule::for_testing(
        imported_path.clone(),
        "utils_with_args".to_string(),
        imported_program,
        "".to_string(),
    );

    let entry_program = Program {
        imports: vec![ImportDecl {
            path: "./utils_with_args".to_string(),
            alias: Some("utils".to_string()),
            span: dummy_span(),
        }],
        functions: vec![FnDef {
            visibility: Visibility::Private,
            name: "main".to_string(),
            params: vec![],
            return_type: "void".to_string(),
            return_type_span: dummy_span(),
            body: vec![expr_stmt(ExprKind::ModuleCall {
                module: "utils".to_string(),
                function: "greet".to_string(),
                args: vec![Expr::new(
                    ExprKind::StringLiteral("hello".to_string()),
                    dummy_span(),
                )],
            })],
            span: dummy_span(),
        }],
    };
    let entry_path = temp_dir.join("main_with_args.lak");
    let mut entry_module = ResolvedModule::for_testing(
        entry_path.clone(),
        "main_with_args".to_string(),
        entry_program,
        "".to_string(),
    );
    entry_module.add_resolved_import_for_testing("./utils_with_args".to_string(), imported_path);

    let modules = [imported_module, entry_module];

    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");
    codegen
        .compile_modules(&modules, &entry_path)
        .expect("compile_modules with arguments should succeed");

    let greet = codegen
        .module
        .get_function("_L15_utils_with_args_greet")
        .unwrap();
    assert_eq!(greet.count_params(), 1);
}

#[test]
fn test_compile_modules_with_alias() {
    use crate::ast::ImportDecl;
    use crate::resolver::ResolvedModule;

    let temp_dir = std::env::temp_dir();

    // Create an imported module with a public function
    let imported_program = Program {
        imports: vec![],
        functions: vec![FnDef {
            visibility: Visibility::Public,
            name: "greet".to_string(),
            params: vec![],
            return_type: "void".to_string(),
            return_type_span: dummy_span(),
            body: vec![expr_stmt(ExprKind::Call {
                callee: "println".to_string(),
                args: vec![Expr::new(
                    ExprKind::StringLiteral("hello".to_string()),
                    dummy_span(),
                )],
            })],
            span: dummy_span(),
        }],
    };
    let imported_path = temp_dir.join("utils.lak");
    let imported_module = ResolvedModule::for_testing(
        imported_path.clone(),
        "utils".to_string(),
        imported_program,
        "".to_string(),
    );

    // Create entry module with aliased import: import "./utils" as u
    let entry_program = Program {
        imports: vec![ImportDecl {
            path: "./utils".to_string(),
            alias: Some("u".to_string()),
            span: dummy_span(),
        }],
        functions: vec![FnDef {
            visibility: Visibility::Private,
            name: "main".to_string(),
            params: vec![],
            return_type: "void".to_string(),
            return_type_span: dummy_span(),
            body: vec![expr_stmt(ExprKind::ModuleCall {
                module: "u".to_string(),
                function: "greet".to_string(),
                args: vec![],
            })],
            span: dummy_span(),
        }],
    };
    let entry_path = temp_dir.join("main.lak");
    let mut entry_module = ResolvedModule::for_testing(
        entry_path.clone(),
        "main".to_string(),
        entry_program,
        "".to_string(),
    );
    entry_module.add_resolved_import_for_testing("./utils".to_string(), imported_path);

    let modules = [imported_module, entry_module];

    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");
    codegen
        .compile_modules(&modules, &entry_path)
        .expect("compile_modules with alias should succeed");

    // Verify main exists
    assert!(codegen.module.get_function("main").is_some());
    // Verify mangled function uses path-based prefix, not alias
    assert!(codegen.module.get_function("_L5_utils_greet").is_some());
}

#[test]
fn test_compile_modules_entry_and_imported_mangled_name_collision() {
    use crate::ast::ImportDecl;
    use crate::resolver::ResolvedModule;

    let temp_dir = std::env::temp_dir();

    let imported_program = Program {
        imports: vec![],
        functions: vec![FnDef {
            visibility: Visibility::Public,
            name: "foo".to_string(),
            params: vec![],
            return_type: "void".to_string(),
            return_type_span: dummy_span(),
            body: vec![expr_stmt(ExprKind::Call {
                callee: "println".to_string(),
                args: vec![Expr::new(
                    ExprKind::StringLiteral("from utils".to_string()),
                    dummy_span(),
                )],
            })],
            span: dummy_span(),
        }],
    };
    let imported_path = temp_dir.join("utils.lak");
    let imported_module = ResolvedModule::for_testing(
        imported_path.clone(),
        "utils".to_string(),
        imported_program,
        "".to_string(),
    );

    let entry_program = Program {
        imports: vec![ImportDecl {
            path: "./utils".to_string(),
            alias: None,
            span: dummy_span(),
        }],
        functions: vec![
            FnDef {
                visibility: Visibility::Private,
                name: "_L5_utils_foo".to_string(),
                params: vec![],
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![expr_stmt(ExprKind::Call {
                    callee: "println".to_string(),
                    args: vec![Expr::new(
                        ExprKind::StringLiteral("from main".to_string()),
                        dummy_span(),
                    )],
                })],
                span: dummy_span(),
            },
            FnDef {
                visibility: Visibility::Private,
                name: "main".to_string(),
                params: vec![],
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![
                    expr_stmt(ExprKind::ModuleCall {
                        module: "utils".to_string(),
                        function: "foo".to_string(),
                        args: vec![],
                    }),
                    expr_stmt(ExprKind::Call {
                        callee: "_L5_utils_foo".to_string(),
                        args: vec![],
                    }),
                ],
                span: dummy_span(),
            },
        ],
    };
    let entry_path = temp_dir.join("main.lak");
    let mut entry_module = ResolvedModule::for_testing(
        entry_path.clone(),
        "main".to_string(),
        entry_program,
        "".to_string(),
    );
    entry_module.add_resolved_import_for_testing("./utils".to_string(), imported_path);

    let modules = [imported_module, entry_module];

    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");
    codegen
        .compile_modules(&modules, &entry_path)
        .expect("compile_modules should avoid entry/imported symbol collision");

    // Imported module symbol
    assert!(codegen.module.get_function("_L5_utils_foo").is_some());
    // Entry module symbol (main prefix mangling)
    assert!(
        codegen
            .module
            .get_function("_L4_main__L5_utils_foo")
            .is_some()
    );
}

#[test]
fn test_compile_modules_subdirectory() {
    use crate::ast::ImportDecl;
    use crate::resolver::ResolvedModule;

    let temp_dir = std::env::temp_dir();

    // Create an imported module in a subdirectory
    let imported_program = Program {
        imports: vec![],
        functions: vec![FnDef {
            visibility: Visibility::Public,
            name: "greet".to_string(),
            params: vec![],
            return_type: "void".to_string(),
            return_type_span: dummy_span(),
            body: vec![expr_stmt(ExprKind::Call {
                callee: "println".to_string(),
                args: vec![Expr::new(
                    ExprKind::StringLiteral("hello from lib/utils".to_string()),
                    dummy_span(),
                )],
            })],
            span: dummy_span(),
        }],
    };
    // Module is in a subdirectory relative to entry
    let imported_path = temp_dir.join("lib").join("utils.lak");
    let imported_module = ResolvedModule::for_testing(
        imported_path.clone(),
        "utils".to_string(),
        imported_program,
        "".to_string(),
    );

    // Create entry module that imports and calls utils.greet()
    let entry_program = Program {
        imports: vec![ImportDecl {
            path: "./lib/utils".to_string(),
            alias: None,
            span: dummy_span(),
        }],
        functions: vec![FnDef {
            visibility: Visibility::Private,
            name: "main".to_string(),
            params: vec![],
            return_type: "void".to_string(),
            return_type_span: dummy_span(),
            body: vec![expr_stmt(ExprKind::ModuleCall {
                module: "utils".to_string(),
                function: "greet".to_string(),
                args: vec![],
            })],
            span: dummy_span(),
        }],
    };
    let entry_path = temp_dir.join("main.lak");
    let mut entry_module = ResolvedModule::for_testing(
        entry_path.clone(),
        "main".to_string(),
        entry_program,
        "".to_string(),
    );
    entry_module.add_resolved_import_for_testing("./lib/utils".to_string(), imported_path);

    let modules = [imported_module, entry_module];

    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");
    codegen
        .compile_modules(&modules, &entry_path)
        .expect("compile_modules with subdirectory module should succeed");

    // Verify main exists
    assert!(codegen.module.get_function("main").is_some());
    // Verify mangled function uses path-based prefix including directory
    assert!(
        codegen
            .module
            .get_function("_L10_lib__utils_greet")
            .is_some()
    );
}

// ==================================
// Module path error constructor tests
// ==================================

#[test]
fn test_internal_mangle_prefix_not_found_constructor() {
    let path = std::path::Path::new("/tmp/utils.lak");
    let err = CodegenError::internal_mangle_prefix_not_found(path);
    assert_eq!(err.kind(), CodegenErrorKind::InternalError);
    assert!(err.span().is_none());
    assert_eq!(
        err.message(),
        "Internal error: mangle prefix not found for module '/tmp/utils.lak'. This is a compiler bug."
    );
}

#[test]
fn test_non_utf8_path_component_constructor() {
    let path = std::path::Path::new("/tmp/test.lak");
    let err = CodegenError::non_utf8_path_component(path);
    assert_eq!(err.kind(), CodegenErrorKind::InvalidModulePath);
    assert!(err.span().is_none());
    assert_eq!(
        err.message(),
        "Module path '/tmp/test.lak' contains a non-UTF-8 component."
    );
}

#[test]
fn test_internal_entry_path_no_parent_constructor() {
    let path = std::path::Path::new("/tmp/main.lak");
    let err = CodegenError::internal_entry_path_no_parent(path);
    assert_eq!(err.kind(), CodegenErrorKind::InternalError);
    assert!(err.span().is_none());
    assert_eq!(
        err.message(),
        "Internal error: entry path '/tmp/main.lak' has no parent directory. This is a compiler bug."
    );
}

#[test]
fn test_internal_non_canonical_path_constructor() {
    let path = std::path::Path::new("/tmp/test.lak");
    let err = CodegenError::internal_non_canonical_path(path);
    assert_eq!(err.kind(), CodegenErrorKind::InternalError);
    assert!(err.span().is_none());
    assert_eq!(
        err.message(),
        "Internal error: module path '/tmp/test.lak' contains '.' or '..' components. \
         Paths must be canonicalized before code generation. This is a compiler bug."
    );
}

// ==================================
// compute_mangle_prefixes tests
// ==================================

#[test]
fn test_compute_mangle_prefixes_same_directory() {
    let entry = PathBuf::from("/project/main.lak");
    let modules = vec![
        ResolvedModule::for_testing(
            PathBuf::from("/project/main.lak"),
            "main".to_string(),
            empty_program(),
            String::new(),
        ),
        ResolvedModule::for_testing(
            PathBuf::from("/project/utils.lak"),
            "utils".to_string(),
            empty_program(),
            String::new(),
        ),
    ];
    let prefixes = compute_mangle_prefixes(&modules, &entry).unwrap();
    assert_eq!(
        prefixes.get(Path::new("/project/utils.lak")).unwrap(),
        "utils"
    );
}

#[test]
fn test_compute_mangle_prefixes_subdirectory() {
    let entry = PathBuf::from("/project/main.lak");
    let modules = vec![
        ResolvedModule::for_testing(
            PathBuf::from("/project/main.lak"),
            "main".to_string(),
            empty_program(),
            String::new(),
        ),
        ResolvedModule::for_testing(
            PathBuf::from("/project/dir/foo.lak"),
            "foo".to_string(),
            empty_program(),
            String::new(),
        ),
    ];
    let prefixes = compute_mangle_prefixes(&modules, &entry).unwrap();
    assert_eq!(
        prefixes.get(Path::new("/project/dir/foo.lak")).unwrap(),
        "dir__foo"
    );
}

#[test]
fn test_compute_mangle_prefixes_deeply_nested() {
    let entry = PathBuf::from("/project/main.lak");
    let modules = vec![
        ResolvedModule::for_testing(
            PathBuf::from("/project/main.lak"),
            "main".to_string(),
            empty_program(),
            String::new(),
        ),
        ResolvedModule::for_testing(
            PathBuf::from("/project/a/b/c/mod.lak"),
            "mod".to_string(),
            empty_program(),
            String::new(),
        ),
    ];
    let prefixes = compute_mangle_prefixes(&modules, &entry).unwrap();
    assert_eq!(
        prefixes.get(Path::new("/project/a/b/c/mod.lak")).unwrap(),
        "a__b__c__mod"
    );
}

#[test]
fn test_compute_mangle_prefixes_same_name_different_dirs() {
    let entry = PathBuf::from("/project/main.lak");
    let modules = vec![
        ResolvedModule::for_testing(
            PathBuf::from("/project/main.lak"),
            "main".to_string(),
            empty_program(),
            String::new(),
        ),
        ResolvedModule::for_testing(
            PathBuf::from("/project/foo.lak"),
            "foo".to_string(),
            empty_program(),
            String::new(),
        ),
        ResolvedModule::for_testing(
            PathBuf::from("/project/dir/foo.lak"),
            "foo".to_string(),
            empty_program(),
            String::new(),
        ),
    ];
    let prefixes = compute_mangle_prefixes(&modules, &entry).unwrap();
    assert_eq!(prefixes.get(Path::new("/project/foo.lak")).unwrap(), "foo");
    assert_eq!(
        prefixes.get(Path::new("/project/dir/foo.lak")).unwrap(),
        "dir__foo"
    );
}

#[test]
fn test_compute_mangle_prefixes_skips_entry() {
    let entry = PathBuf::from("/project/main.lak");
    let modules = vec![ResolvedModule::for_testing(
        PathBuf::from("/project/main.lak"),
        "main".to_string(),
        empty_program(),
        String::new(),
    )];
    let prefixes = compute_mangle_prefixes(&modules, &entry).unwrap();
    assert!(prefixes.is_empty());
}

#[test]
fn test_compute_mangle_prefixes_empty_modules() {
    let entry = PathBuf::from("/project/main.lak");
    let prefixes = compute_mangle_prefixes(&[], &entry).unwrap();
    assert!(prefixes.is_empty());
}

#[test]
fn test_compute_mangle_prefixes_outside_entry_dir() {
    let entry = PathBuf::from("/home/user/project/main.lak");
    let modules = vec![
        ResolvedModule::for_testing(
            PathBuf::from("/home/user/project/main.lak"),
            "main".to_string(),
            empty_program(),
            String::new(),
        ),
        ResolvedModule::for_testing(
            PathBuf::from("/opt/lib/utils.lak"),
            "utils".to_string(),
            empty_program(),
            String::new(),
        ),
    ];
    let prefixes = compute_mangle_prefixes(&modules, &entry).unwrap();
    assert_eq!(
        prefixes.get(Path::new("/opt/lib/utils.lak")).unwrap(),
        "opt__lib__utils"
    );
}

#[test]
fn test_codegen_error_short_message_invalid_module_path() {
    let err = CodegenError::without_span(CodegenErrorKind::InvalidModulePath, "test error");
    assert_eq!(err.short_message(), "Invalid module path");
}

#[test]
fn test_duplicate_mangle_prefix_constructor() {
    let path1 = std::path::Path::new("/project/a/utils.lak");
    let path2 = std::path::Path::new("/project/b/utils.lak");
    let err = CodegenError::duplicate_mangle_prefix("utils", path1, path2);
    assert_eq!(err.kind(), CodegenErrorKind::InvalidModulePath);
    assert!(err.span().is_none());
    assert_eq!(
        err.message(),
        "Modules '/project/a/utils.lak' and '/project/b/utils.lak' produce the same mangle prefix 'utils'. \
         Rename one of the modules to avoid the collision."
    );
}

#[test]
fn test_duplicate_mangle_prefix_reverse_order() {
    // Verify that passing paths in reverse alphabetical order
    // still produces a sorted error message.
    let path1 = std::path::Path::new("/project/b/utils.lak");
    let path2 = std::path::Path::new("/project/a/utils.lak");
    let err = CodegenError::duplicate_mangle_prefix("utils", path1, path2);
    assert_eq!(err.kind(), CodegenErrorKind::InvalidModulePath);
    assert!(err.span().is_none());
    assert_eq!(
        err.message(),
        "Modules '/project/a/utils.lak' and '/project/b/utils.lak' produce the same mangle prefix 'utils'. \
         Rename one of the modules to avoid the collision."
    );
}

#[test]
fn test_internal_empty_mangle_prefix_constructor() {
    let path = std::path::Path::new("/project/empty.lak");
    let err = CodegenError::internal_empty_mangle_prefix(path);
    assert_eq!(err.kind(), CodegenErrorKind::InternalError);
    assert!(err.span().is_none());
    assert_eq!(
        err.message(),
        "Internal error: module path '/project/empty.lak' produces an empty mangle prefix. This is a compiler bug."
    );
}

#[test]
fn test_compute_mangle_prefixes_duplicate_detection() {
    // A module inside the entry directory and one outside can collide
    // when the outside module's full Normal components match the inside
    // module's relative path prefix.
    let entry = PathBuf::from("/project/main.lak");
    let modules = vec![
        ResolvedModule::for_testing(
            PathBuf::from("/project/main.lak"),
            "main".to_string(),
            empty_program(),
            String::new(),
        ),
        ResolvedModule::for_testing(
            PathBuf::from("/project/foo.lak"),
            "foo".to_string(),
            empty_program(),
            String::new(),
        ),
        ResolvedModule::for_testing(
            PathBuf::from("/foo.lak"),
            "foo".to_string(),
            empty_program(),
            String::new(),
        ),
    ];
    let result = compute_mangle_prefixes(&modules, &entry);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), CodegenErrorKind::InvalidModulePath);
    assert_eq!(
        err.message(),
        "Modules '/foo.lak' and '/project/foo.lak' produce the same mangle prefix 'foo'. \
         Rename one of the modules to avoid the collision."
    );
}

#[cfg(unix)]
#[test]
fn test_compute_mangle_prefixes_empty_prefix() {
    let entry = PathBuf::from("/project/main.lak");
    let modules = vec![
        ResolvedModule::for_testing(
            PathBuf::from("/project/main.lak"),
            "main".to_string(),
            empty_program(),
            String::new(),
        ),
        ResolvedModule::for_testing(
            PathBuf::from("/"),
            "root".to_string(),
            empty_program(),
            String::new(),
        ),
    ];
    let result = compute_mangle_prefixes(&modules, &entry);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), CodegenErrorKind::InternalError);
    assert_eq!(
        err.message(),
        "Internal error: module path '/' produces an empty mangle prefix. This is a compiler bug."
    );
}

#[test]
fn test_get_mangle_prefix_found() {
    let mut prefixes = HashMap::new();
    prefixes.insert(PathBuf::from("/project/utils.lak"), "utils".to_string());
    prefixes.insert(
        PathBuf::from("/project/lib/helper.lak"),
        "lib__helper".to_string(),
    );
    assert_eq!(
        get_mangle_prefix(&prefixes, Path::new("/project/utils.lak")).unwrap(),
        "utils"
    );
    assert_eq!(
        get_mangle_prefix(&prefixes, Path::new("/project/lib/helper.lak")).unwrap(),
        "lib__helper"
    );
}

#[test]
fn test_get_mangle_prefix_not_found() {
    let prefixes = HashMap::new();
    let module_path = Path::new("/project/missing.lak");
    let result = get_mangle_prefix(&prefixes, module_path);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), CodegenErrorKind::InternalError);
    assert_eq!(
        err.message(),
        "Internal error: mangle prefix not found for module '/project/missing.lak'. This is a compiler bug."
    );
}

#[cfg(unix)]
#[test]
fn test_path_components_to_strings_filters_non_normal() {
    let module_path = Path::new("/a/b/c.lak");

    // Root component is filtered out; only Normal components remain
    let path = Path::new("/a/b/c");
    let result = path_components_to_strings(path, module_path).unwrap();
    assert_eq!(result, vec!["a", "b", "c"]);
}

#[test]
fn test_path_components_to_strings_relative_normal_only() {
    let module_path = Path::new("/a/b/c.lak");
    let path = Path::new("a/b/c");
    let result = path_components_to_strings(path, module_path).unwrap();
    assert_eq!(result, vec!["a", "b", "c"]);
}

#[test]
fn test_path_components_to_strings_rejects_parent_dir() {
    let module_path = Path::new("/a/b/c.lak");
    let path = Path::new("a/../b");
    let result = path_components_to_strings(path, module_path);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), CodegenErrorKind::InternalError);
    assert_eq!(
        err.message(),
        "Internal error: module path '/a/b/c.lak' contains '.' or '..' components. \
         Paths must be canonicalized before code generation. This is a compiler bug."
    );
}

#[test]
fn test_path_components_to_strings_rejects_cur_dir() {
    let module_path = Path::new("/a/b/c.lak");
    let path = Path::new("./a/b");
    let result = path_components_to_strings(path, module_path);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), CodegenErrorKind::InternalError);
    assert_eq!(
        err.message(),
        "Internal error: module path '/a/b/c.lak' contains '.' or '..' components. \
         Paths must be canonicalized before code generation. This is a compiler bug."
    );
}
