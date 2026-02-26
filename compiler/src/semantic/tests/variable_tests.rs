//! Tests for variable-related semantic analysis.
//!
//! Covers:
//! - Duplicate variable definitions
//! - Undefined variable references

use super::*;

// ============================================================================
// Duplicate variable tests
// ============================================================================

#[test]
fn test_duplicate_variable() {
    let program = program_with_main(vec![
        Stmt::new(
            StmtKind::Let {
                is_mutable: false,
                name: "x".to_string(),
                ty: Type::I32,
                init: Expr::new(ExprKind::IntLiteral(1), dummy_span()),
            },
            span_at(2, 5),
        ),
        Stmt::new(
            StmtKind::Let {
                is_mutable: false,
                name: "x".to_string(),
                ty: Type::I32,
                init: Expr::new(ExprKind::IntLiteral(2), dummy_span()),
            },
            span_at(3, 5),
        ),
    ]);

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::DuplicateVariable);
    assert_eq!(err.message(), "Variable 'x' is already defined at 2:5");
}

#[test]
fn test_duplicate_variable_mutable_then_immutable() {
    let program = program_with_main(vec![
        Stmt::new(
            StmtKind::Let {
                is_mutable: true,
                name: "x".to_string(),
                ty: Type::I32,
                init: Expr::new(ExprKind::IntLiteral(1), dummy_span()),
            },
            span_at(2, 5),
        ),
        Stmt::new(
            StmtKind::Let {
                is_mutable: false,
                name: "x".to_string(),
                ty: Type::I32,
                init: Expr::new(ExprKind::IntLiteral(2), dummy_span()),
            },
            span_at(3, 5),
        ),
    ]);

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::DuplicateVariable);
    assert_eq!(err.message(), "Variable 'x' is already defined at 2:5");
}

#[test]
fn test_duplicate_variable_immutable_then_mutable() {
    let program = program_with_main(vec![
        Stmt::new(
            StmtKind::Let {
                is_mutable: false,
                name: "x".to_string(),
                ty: Type::I32,
                init: Expr::new(ExprKind::IntLiteral(1), dummy_span()),
            },
            span_at(2, 5),
        ),
        Stmt::new(
            StmtKind::Let {
                is_mutable: true,
                name: "x".to_string(),
                ty: Type::I32,
                init: Expr::new(ExprKind::IntLiteral(2), dummy_span()),
            },
            span_at(3, 5),
        ),
    ]);

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::DuplicateVariable);
    assert_eq!(err.message(), "Variable 'x' is already defined at 2:5");
}

// ============================================================================
// Undefined variable tests
// ============================================================================

#[test]
fn test_undefined_variable() {
    let program = program_with_main(vec![Stmt::new(
        StmtKind::Let {
            is_mutable: false,
            name: "x".to_string(),
            ty: Type::I32,
            init: Expr::new(ExprKind::Identifier("y".to_string()), span_at(2, 18)),
        },
        dummy_span(),
    )]);

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::UndefinedVariable);
    assert_eq!(err.message(), "Undefined variable: 'y'");
}

#[test]
fn test_self_referential_let_initializer_is_undefined_variable() {
    let program = program_with_main(vec![Stmt::new(
        StmtKind::Let {
            is_mutable: false,
            name: "x".to_string(),
            ty: Type::I32,
            init: Expr::new(ExprKind::Identifier("x".to_string()), span_at(2, 18)),
        },
        span_at(2, 5),
    )]);

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::UndefinedVariable);
    assert_eq!(err.message(), "Undefined variable: 'x'");
}

#[test]
fn test_mutable_variable_declaration_is_valid() {
    let program = program_with_main(vec![Stmt::new(
        StmtKind::Let {
            is_mutable: true,
            name: "x".to_string(),
            ty: Type::I32,
            init: Expr::new(ExprKind::IntLiteral(1), span_at(2, 24)),
        },
        span_at(2, 5),
    )]);

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_ok());
}

#[test]
fn test_inferred_variable_defaults_to_i64() {
    let program = program_with_main(vec![
        Stmt::new(
            StmtKind::Let {
                is_mutable: false,
                name: "x".to_string(),
                ty: Type::Inferred,
                init: Expr::new(ExprKind::IntLiteral(1), span_at(2, 13)),
            },
            span_at(2, 5),
        ),
        Stmt::new(
            StmtKind::Let {
                is_mutable: false,
                name: "y".to_string(),
                ty: Type::I64,
                init: Expr::new(ExprKind::Identifier("x".to_string()), span_at(3, 18)),
            },
            span_at(3, 5),
        ),
    ]);

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_ok());
}

#[test]
fn test_inferred_variable_from_expression_defaults_to_i64() {
    let program = program_with_main(vec![
        Stmt::new(
            StmtKind::Let {
                is_mutable: false,
                name: "x".to_string(),
                ty: Type::Inferred,
                init: Expr::new(
                    ExprKind::BinaryOp {
                        left: Box::new(Expr::new(ExprKind::IntLiteral(5), span_at(2, 13))),
                        op: crate::ast::BinaryOperator::Add,
                        right: Box::new(Expr::new(ExprKind::IntLiteral(10), span_at(2, 17))),
                    },
                    span_at(2, 13),
                ),
            },
            span_at(2, 5),
        ),
        Stmt::new(
            StmtKind::Let {
                is_mutable: false,
                name: "y".to_string(),
                ty: Type::I64,
                init: Expr::new(ExprKind::Identifier("x".to_string()), span_at(3, 18)),
            },
            span_at(3, 5),
        ),
    ]);

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_ok());
}

#[test]
fn test_inferred_float_variable_defaults_to_f64() {
    let program = program_with_main(vec![
        Stmt::new(
            StmtKind::Let {
                is_mutable: false,
                name: "x".to_string(),
                ty: Type::Inferred,
                init: Expr::new(ExprKind::FloatLiteral(2.5), span_at(2, 13)),
            },
            span_at(2, 5),
        ),
        Stmt::new(
            StmtKind::Let {
                is_mutable: false,
                name: "y".to_string(),
                ty: Type::F64,
                init: Expr::new(ExprKind::Identifier("x".to_string()), span_at(3, 18)),
            },
            span_at(3, 5),
        ),
    ]);

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_ok());
}

#[test]
fn test_inferred_string_variable_defaults_to_string() {
    let program = program_with_main(vec![
        Stmt::new(
            StmtKind::Let {
                is_mutable: false,
                name: "s".to_string(),
                ty: Type::Inferred,
                init: Expr::new(ExprKind::StringLiteral("hello".to_string()), span_at(2, 13)),
            },
            span_at(2, 5),
        ),
        Stmt::new(
            StmtKind::Let {
                is_mutable: false,
                name: "copy".to_string(),
                ty: Type::String,
                init: Expr::new(ExprKind::Identifier("s".to_string()), span_at(3, 22)),
            },
            span_at(3, 5),
        ),
    ]);

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_ok());
}

#[test]
fn test_inferred_bool_variable_defaults_to_bool() {
    let program = program_with_main(vec![
        Stmt::new(
            StmtKind::Let {
                is_mutable: false,
                name: "ok".to_string(),
                ty: Type::Inferred,
                init: Expr::new(ExprKind::BoolLiteral(true), span_at(2, 14)),
            },
            span_at(2, 5),
        ),
        Stmt::new(
            StmtKind::Let {
                is_mutable: false,
                name: "copy".to_string(),
                ty: Type::Bool,
                init: Expr::new(ExprKind::Identifier("ok".to_string()), span_at(3, 20)),
            },
            span_at(3, 5),
        ),
    ]);

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_ok());
}

#[test]
fn test_inferred_self_referential_initializer_is_undefined_variable() {
    let program = program_with_main(vec![Stmt::new(
        StmtKind::Let {
            is_mutable: false,
            name: "x".to_string(),
            ty: Type::Inferred,
            init: Expr::new(ExprKind::Identifier("x".to_string()), span_at(2, 13)),
        },
        span_at(2, 5),
    )]);

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::UndefinedVariable);
    assert_eq!(err.message(), "Undefined variable: 'x'");
}

#[test]
fn test_inferred_binding_types_exports_resolved_spans() {
    let inferred_span = span_at(2, 5);
    let explicit_span = span_at(3, 5);
    let program = program_with_main(vec![
        Stmt::new(
            StmtKind::Let {
                is_mutable: false,
                name: "x".to_string(),
                ty: Type::Inferred,
                init: Expr::new(ExprKind::IntLiteral(1), span_at(2, 13)),
            },
            inferred_span,
        ),
        Stmt::new(
            StmtKind::Let {
                is_mutable: false,
                name: "y".to_string(),
                ty: Type::I32,
                init: Expr::new(ExprKind::IntLiteral(2), span_at(3, 18)),
            },
            explicit_span,
        ),
    ]);

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_ok());

    let inferred = analyzer.inferred_binding_types();
    assert_eq!(inferred.get(&inferred_span), Some(&Type::I64));
    assert!(!inferred.contains_key(&explicit_span));
}

#[test]
fn test_inferred_binding_types_duplicate_span_is_internal_error() {
    let duplicate_span = span_at(2, 5);
    let program = program_with_main(vec![
        Stmt::new(
            StmtKind::Let {
                is_mutable: false,
                name: "a".to_string(),
                ty: Type::Inferred,
                init: Expr::new(ExprKind::IntLiteral(1), span_at(2, 13)),
            },
            duplicate_span,
        ),
        Stmt::new(
            StmtKind::Let {
                is_mutable: false,
                name: "b".to_string(),
                ty: Type::Inferred,
                init: Expr::new(ExprKind::BoolLiteral(true), span_at(3, 13)),
            },
            duplicate_span,
        ),
    ]);

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::InternalError);
    assert_eq!(
        err.message(),
        "Internal error: inferred binding for variable 'b' produced a conflicting span-to-type mapping in semantic analysis. The same span resolved to different inferred types. This is a compiler bug."
    );
    assert_eq!(err.span(), Some(duplicate_span));
}

#[test]
fn test_mutable_variable_reassignment_is_valid() {
    let program = program_with_main(vec![
        Stmt::new(
            StmtKind::Let {
                is_mutable: true,
                name: "x".to_string(),
                ty: Type::I32,
                init: Expr::new(ExprKind::IntLiteral(1), span_at(2, 22)),
            },
            span_at(2, 5),
        ),
        Stmt::new(
            StmtKind::Assign {
                name: "x".to_string(),
                value: Expr::new(ExprKind::IntLiteral(2), span_at(3, 9)),
            },
            span_at(3, 5),
        ),
    ]);

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_ok());
}

#[test]
fn test_inferred_mutable_variable_reassignment_is_valid() {
    let program = program_with_main(vec![
        Stmt::new(
            StmtKind::Let {
                is_mutable: true,
                name: "x".to_string(),
                ty: Type::Inferred,
                init: Expr::new(ExprKind::IntLiteral(1), span_at(2, 17)),
            },
            span_at(2, 5),
        ),
        Stmt::new(
            StmtKind::Assign {
                name: "x".to_string(),
                value: Expr::new(ExprKind::IntLiteral(2), span_at(3, 9)),
            },
            span_at(3, 5),
        ),
    ]);

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_ok());
}

#[test]
fn test_reassignment_to_immutable_variable_is_error() {
    let program = program_with_main(vec![
        Stmt::new(
            StmtKind::Let {
                is_mutable: false,
                name: "x".to_string(),
                ty: Type::I32,
                init: Expr::new(ExprKind::IntLiteral(1), span_at(2, 18)),
            },
            span_at(2, 5),
        ),
        Stmt::new(
            StmtKind::Assign {
                name: "x".to_string(),
                value: Expr::new(ExprKind::IntLiteral(2), span_at(3, 9)),
            },
            span_at(3, 5),
        ),
    ]);

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::ImmutableVariableReassignment);
    assert_eq!(err.message(), "Cannot reassign immutable variable 'x'");
}

#[test]
fn test_reassignment_type_mismatch_is_error() {
    let program = program_with_main(vec![
        Stmt::new(
            StmtKind::Let {
                is_mutable: true,
                name: "x".to_string(),
                ty: Type::I32,
                init: Expr::new(ExprKind::IntLiteral(1), span_at(2, 22)),
            },
            span_at(2, 5),
        ),
        Stmt::new(
            StmtKind::Assign {
                name: "x".to_string(),
                value: Expr::new(ExprKind::StringLiteral("hello".to_string()), span_at(3, 9)),
            },
            span_at(3, 5),
        ),
    ]);

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::TypeMismatch);
    assert_eq!(
        err.message(),
        "Type mismatch: string literal cannot be assigned to type 'i32'"
    );
}

#[test]
fn test_reassignment_to_undefined_variable_is_error() {
    let program = program_with_main(vec![Stmt::new(
        StmtKind::Assign {
            name: "x".to_string(),
            value: Expr::new(ExprKind::IntLiteral(2), span_at(2, 9)),
        },
        span_at(2, 5),
    )]);

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::UndefinedVariable);
    assert_eq!(err.message(), "Undefined variable: 'x'");
}
