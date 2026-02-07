//! Pipeline and direct AST construction tests for the Lak compiler.
//!
//! These tests verify the integration between compiler phases
//! and test scenarios that require direct AST construction.

mod common;

use common::dummy_span;

use lak::ast::{Expr, ExprKind, FnDef, Program, Stmt, StmtKind, Type, Visibility};
use lak::codegen::Codegen;
use lak::lexer::Lexer;
use lak::parser::Parser;
use lak::semantic::SemanticAnalyzer;

use inkwell::context::Context;

#[test]
fn test_lexer_parser_integration() {
    let source = r#"fn main() -> void { println("test") }"#;

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    // fn, main, (, ), ->, void, {, println, (, string, ), }, eof
    assert_eq!(tokens.len(), 13);

    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();
    assert_eq!(program.functions.len(), 1);
    assert_eq!(program.functions[0].name, "main");
}

#[test]
fn test_ast_to_codegen() {
    // Build AST directly and compile
    let program = Program {
        imports: vec![],
        functions: vec![FnDef {
            name: "main".to_string(),
            visibility: Visibility::Private,
            return_type: "void".to_string(),
            return_type_span: dummy_span(),
            body: vec![Stmt::new(
                StmtKind::Expr(Expr::new(
                    ExprKind::Call {
                        callee: "println".to_string(),
                        args: vec![Expr::new(
                            ExprKind::StringLiteral("direct AST".to_string()),
                            dummy_span(),
                        )],
                    },
                    dummy_span(),
                )),
                dummy_span(),
            )],
            span: dummy_span(),
        }],
    };

    // Semantic analysis
    let mut analyzer = SemanticAnalyzer::new();
    analyzer
        .analyze(&program)
        .expect("Semantic analysis should pass");

    let context = Context::create();
    let mut codegen = Codegen::new(&context, "direct_ast_test");
    codegen
        .compile(&program)
        .expect("Direct AST compilation should succeed");
}

#[test]
fn test_ast_let_to_codegen() {
    // Build AST with Let statement directly and compile
    let program = Program {
        imports: vec![],
        functions: vec![FnDef {
            name: "main".to_string(),
            visibility: Visibility::Private,
            return_type: "void".to_string(),
            return_type_span: dummy_span(),
            body: vec![
                Stmt::new(
                    StmtKind::Let {
                        name: "x".to_string(),
                        ty: Type::I32,
                        init: Expr::new(ExprKind::IntLiteral(42), dummy_span()),
                    },
                    dummy_span(),
                ),
                Stmt::new(
                    StmtKind::Let {
                        name: "y".to_string(),
                        ty: Type::I32,
                        init: Expr::new(ExprKind::Identifier("x".to_string()), dummy_span()),
                    },
                    dummy_span(),
                ),
            ],
            span: dummy_span(),
        }],
    };

    // Semantic analysis
    let mut analyzer = SemanticAnalyzer::new();
    analyzer
        .analyze(&program)
        .expect("Semantic analysis should pass");

    let context = Context::create();
    let mut codegen = Codegen::new(&context, "let_ast_test");
    codegen
        .compile(&program)
        .expect("Direct AST Let compilation should succeed");
}

#[test]
fn test_error_string_literal_as_int_value() {
    // String literal cannot be used as integer initializer
    // Detected by semantic analysis
    let program = Program {
        imports: vec![],
        functions: vec![FnDef {
            name: "main".to_string(),
            visibility: Visibility::Private,
            return_type: "void".to_string(),
            return_type_span: dummy_span(),
            body: vec![Stmt::new(
                StmtKind::Let {
                    name: "x".to_string(),
                    ty: Type::I32,
                    init: Expr::new(ExprKind::StringLiteral("hello".to_string()), dummy_span()),
                },
                dummy_span(),
            )],
            span: dummy_span(),
        }],
    };

    let mut analyzer = SemanticAnalyzer::new();
    let err = analyzer.analyze(&program).expect_err("Should fail");
    assert_eq!(
        err.message(),
        "Type mismatch: string literal cannot be assigned to type 'i32'"
    );
}

#[test]
fn test_error_function_call_as_int_value() {
    // Function call cannot be used as integer initializer (void functions)
    // Detected by semantic analysis
    let program = Program {
        imports: vec![],
        functions: vec![FnDef {
            name: "main".to_string(),
            visibility: Visibility::Private,
            return_type: "void".to_string(),
            return_type_span: dummy_span(),
            body: vec![Stmt::new(
                StmtKind::Let {
                    name: "x".to_string(),
                    ty: Type::I32,
                    init: Expr::new(
                        ExprKind::Call {
                            callee: "some_func".to_string(),
                            args: vec![],
                        },
                        dummy_span(),
                    ),
                },
                dummy_span(),
            )],
            span: dummy_span(),
        }],
    };

    let mut analyzer = SemanticAnalyzer::new();
    let err = analyzer.analyze(&program).expect_err("Should fail");
    assert_eq!(
        err.message(),
        "Function call 'some_func' cannot be used as a value (functions returning values not yet supported)"
    );
}

#[test]
fn test_error_int_literal_as_statement() {
    // Integer literal used as statement should error
    // Detected by semantic analysis
    let program = Program {
        imports: vec![],
        functions: vec![FnDef {
            name: "main".to_string(),
            visibility: Visibility::Private,
            return_type: "void".to_string(),
            return_type_span: dummy_span(),
            body: vec![Stmt::new(
                StmtKind::Expr(Expr::new(ExprKind::IntLiteral(42), dummy_span())),
                dummy_span(),
            )],
            span: dummy_span(),
        }],
    };

    let mut analyzer = SemanticAnalyzer::new();
    let err = analyzer.analyze(&program).expect_err("Should fail");
    assert_eq!(
        err.message(),
        "Integer literal as a statement has no effect. Did you mean to assign it to a variable?"
    );
}

#[test]
fn test_error_identifier_as_statement() {
    // Variable used as statement should error
    // Detected by semantic analysis
    let program = Program {
        imports: vec![],
        functions: vec![FnDef {
            name: "main".to_string(),
            visibility: Visibility::Private,
            return_type: "void".to_string(),
            return_type_span: dummy_span(),
            body: vec![
                Stmt::new(
                    StmtKind::Let {
                        name: "x".to_string(),
                        ty: Type::I32,
                        init: Expr::new(ExprKind::IntLiteral(42), dummy_span()),
                    },
                    dummy_span(),
                ),
                Stmt::new(
                    StmtKind::Expr(Expr::new(
                        ExprKind::Identifier("x".to_string()),
                        dummy_span(),
                    )),
                    dummy_span(),
                ),
            ],
            span: dummy_span(),
        }],
    };

    let mut analyzer = SemanticAnalyzer::new();
    let err = analyzer.analyze(&program).expect_err("Should fail");
    assert_eq!(
        err.message(),
        "Variable 'x' used as a statement has no effect. Did you mean to use it in an expression?"
    );
}

#[test]
fn test_error_string_literal_as_statement() {
    // String literal used as statement should error
    // Detected by semantic analysis
    let program = Program {
        imports: vec![],
        functions: vec![FnDef {
            name: "main".to_string(),
            visibility: Visibility::Private,
            return_type: "void".to_string(),
            return_type_span: dummy_span(),
            body: vec![Stmt::new(
                StmtKind::Expr(Expr::new(
                    ExprKind::StringLiteral("hello".to_string()),
                    dummy_span(),
                )),
                dummy_span(),
            )],
            span: dummy_span(),
        }],
    };

    let mut analyzer = SemanticAnalyzer::new();
    let err = analyzer.analyze(&program).expect_err("Should fail");
    assert_eq!(
        err.message(),
        "String literal as a statement has no effect. Did you mean to pass it to a function?"
    );
}

#[test]
fn test_error_i32_underflow_via_ast() {
    // Test that values less than i32::MIN are rejected for i32 type.
    // This requires direct AST construction since negative literals aren't
    // supported in source code yet.
    // Detected by semantic analysis
    let program = Program {
        imports: vec![],
        functions: vec![FnDef {
            name: "main".to_string(),
            visibility: Visibility::Private,
            return_type: "void".to_string(),
            return_type_span: dummy_span(),
            body: vec![Stmt::new(
                StmtKind::Let {
                    name: "x".to_string(),
                    ty: Type::I32,
                    init: Expr::new(ExprKind::IntLiteral(-2147483649), dummy_span()), // i32::MIN - 1
                },
                dummy_span(),
            )],
            span: dummy_span(),
        }],
    };

    let mut analyzer = SemanticAnalyzer::new();
    let err = analyzer.analyze(&program).expect_err("Should fail");
    assert_eq!(
        err.message(),
        "Integer literal '-2147483649' is out of range for i32 (valid range: -2147483648 to 2147483647)"
    );
}
