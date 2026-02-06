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
                name: "x".to_string(),
                ty: Type::I32,
                init: Expr::new(ExprKind::IntLiteral(1), dummy_span()),
            },
            span_at(2, 5),
        ),
        Stmt::new(
            StmtKind::Let {
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
