//! Unit tests for code generation.

use super::*;
use crate::ast::{Expr, ExprKind, FnDef, Program, Stmt, StmtKind, Type};
use crate::token::Span;
use inkwell::context::Context;

fn dummy_span() -> Span {
    Span::new(0, 0, 1, 1)
}

/// Helper to create a Program with a main function
fn make_program(body: Vec<Stmt>) -> Program {
    Program {
        functions: vec![FnDef {
            name: "main".to_string(),
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
            name: name.to_string(),
            ty,
            init,
        },
        dummy_span(),
    )
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
        functions: vec![
            FnDef {
                name: "helper".to_string(),
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![],
                span: dummy_span(),
            },
            FnDef {
                name: "main".to_string(),
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
    assert!(display.contains("3:5"));
    assert!(display.contains("test error"));
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
}
