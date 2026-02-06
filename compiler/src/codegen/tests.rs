//! Unit tests for code generation.

use super::*;
use crate::ast::{Expr, ExprKind, FnDef, Program, Stmt, StmtKind, Type, UnaryOperator};
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
        "Internal error: function call 'some_function()' cannot be used as println argument. Semantic analysis should have caught this. This is a compiler bug."
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
// Unary operation error constructor tests
// ==================================

#[test]
fn test_internal_unary_op_string_constructor() {
    let err = CodegenError::internal_unary_op_string(UnaryOperator::Neg, dummy_span());
    assert_eq!(err.kind(), CodegenErrorKind::InternalError);
    assert!(err.span().is_some());
    assert_eq!(
        err.message(),
        "Internal error: unary operator '-' applied to string type in codegen. Semantic analysis should have caught this. This is a compiler bug."
    );
}

#[test]
fn test_internal_unary_op_failed_constructor() {
    let err =
        CodegenError::internal_unary_op_failed(UnaryOperator::Neg, "LLVM error", dummy_span());
    assert_eq!(err.kind(), CodegenErrorKind::InternalError);
    assert!(err.span().is_some());
    assert_eq!(
        err.message(),
        "Internal error: failed to generate unary '-' instruction. This is a compiler bug: LLVM error"
    );
}
