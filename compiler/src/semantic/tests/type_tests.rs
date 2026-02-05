//! Tests for type-related semantic analysis.
//!
//! Covers:
//! - Type mismatch errors
//! - Integer overflow checks
//! - Invalid expression statements
//! - Invalid argument errors (println)
//! - Valid program tests

use super::*;

// ============================================================================
// Type mismatch tests
// ============================================================================

#[test]
fn test_type_mismatch() {
    let program = program_with_main(vec![
        Stmt::new(
            StmtKind::Let {
                name: "x".to_string(),
                ty: Type::I32,
                init: Expr::new(ExprKind::IntLiteral(1), dummy_span()),
            },
            dummy_span(),
        ),
        Stmt::new(
            StmtKind::Let {
                name: "y".to_string(),
                ty: Type::I64,
                init: Expr::new(ExprKind::Identifier("x".to_string()), span_at(3, 18)),
            },
            dummy_span(),
        ),
    ]);

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::TypeMismatch);
    assert!(err.message().contains("Type mismatch"));
}

#[test]
fn test_string_literal_as_integer() {
    let program = program_with_main(vec![Stmt::new(
        StmtKind::Let {
            name: "x".to_string(),
            ty: Type::I32,
            init: Expr::new(ExprKind::StringLiteral("hello".to_string()), span_at(2, 18)),
        },
        dummy_span(),
    )]);

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::TypeMismatch);
    assert!(err.message().contains("String literals cannot be used"));
}

// ============================================================================
// Integer overflow tests
// ============================================================================

#[test]
fn test_integer_overflow_i32() {
    let program = program_with_main(vec![Stmt::new(
        StmtKind::Let {
            name: "x".to_string(),
            ty: Type::I32,
            init: Expr::new(ExprKind::IntLiteral(2147483648), span_at(2, 18)), // i32::MAX + 1
        },
        dummy_span(),
    )]);

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::IntegerOverflow);
    assert!(err.message().contains("out of range for i32"));
}

#[test]
fn test_integer_negative_overflow_i32() {
    let program = program_with_main(vec![Stmt::new(
        StmtKind::Let {
            name: "x".to_string(),
            ty: Type::I32,
            init: Expr::new(ExprKind::IntLiteral(-2147483649), span_at(2, 18)), // i32::MIN - 1
        },
        dummy_span(),
    )]);

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::IntegerOverflow);
}

#[test]
fn test_i64_no_overflow() {
    // Large value that fits in i64 but not i32
    let program = program_with_main(vec![Stmt::new(
        StmtKind::Let {
            name: "x".to_string(),
            ty: Type::I64,
            init: Expr::new(ExprKind::IntLiteral(9223372036854775807), dummy_span()), // i64::MAX
        },
        dummy_span(),
    )]);

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_ok());
}

// ============================================================================
// Invalid expression tests
// ============================================================================

#[test]
fn test_string_literal_as_statement() {
    let program = program_with_main(vec![Stmt::new(
        StmtKind::Expr(Expr::new(
            ExprKind::StringLiteral("orphan string".to_string()),
            span_at(2, 5),
        )),
        dummy_span(),
    )]);

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::InvalidExpression);
    assert!(err.message().contains("no effect"));
}

#[test]
fn test_integer_literal_as_statement() {
    let program = program_with_main(vec![Stmt::new(
        StmtKind::Expr(Expr::new(ExprKind::IntLiteral(42), span_at(2, 5))),
        dummy_span(),
    )]);

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::InvalidExpression);
}

#[test]
fn test_identifier_as_statement() {
    let program = program_with_main(vec![
        Stmt::new(
            StmtKind::Let {
                name: "x".to_string(),
                ty: Type::I32,
                init: Expr::new(ExprKind::IntLiteral(1), dummy_span()),
            },
            dummy_span(),
        ),
        Stmt::new(
            StmtKind::Expr(Expr::new(
                ExprKind::Identifier("x".to_string()),
                span_at(3, 5),
            )),
            dummy_span(),
        ),
    ]);

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::InvalidExpression);
}

// ============================================================================
// Invalid argument tests (println)
// ============================================================================

#[test]
fn test_println_no_arguments() {
    let program = program_with_main(vec![Stmt::new(
        StmtKind::Expr(Expr::new(
            ExprKind::Call {
                callee: "println".to_string(),
                args: vec![],
            },
            span_at(2, 5),
        )),
        dummy_span(),
    )]);

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::InvalidArgument);
    assert!(err.message().contains("1 argument"));
}

#[test]
fn test_println_too_many_arguments() {
    let program = program_with_main(vec![Stmt::new(
        StmtKind::Expr(Expr::new(
            ExprKind::Call {
                callee: "println".to_string(),
                args: vec![
                    Expr::new(ExprKind::StringLiteral("a".to_string()), dummy_span()),
                    Expr::new(ExprKind::StringLiteral("b".to_string()), dummy_span()),
                ],
            },
            span_at(2, 5),
        )),
        dummy_span(),
    )]);

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::InvalidArgument);
}

#[test]
fn test_println_non_string_argument() {
    let program = program_with_main(vec![Stmt::new(
        StmtKind::Expr(Expr::new(
            ExprKind::Call {
                callee: "println".to_string(),
                args: vec![Expr::new(ExprKind::IntLiteral(42), span_at(2, 13))],
            },
            span_at(2, 5),
        )),
        dummy_span(),
    )]);

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::InvalidArgument);
    assert!(err.message().contains("string literal"));
}

#[test]
fn test_println_with_variable_argument() {
    // Variable reference as println argument should fail (println requires string literal)
    let program = program_with_main(vec![
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
                ExprKind::Call {
                    callee: "println".to_string(),
                    args: vec![Expr::new(
                        ExprKind::Identifier("x".to_string()),
                        span_at(3, 13),
                    )],
                },
                span_at(3, 5),
            )),
            dummy_span(),
        ),
    ]);

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::InvalidArgument);
    assert!(err.message().contains("string literal"));
}

// ============================================================================
// Valid program tests
// ============================================================================

#[test]
fn test_valid_minimal_program() {
    let program = program_with_main(vec![]);

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_ok());
}

#[test]
fn test_valid_program_with_println() {
    let program = program_with_main(vec![Stmt::new(
        StmtKind::Expr(Expr::new(
            ExprKind::Call {
                callee: "println".to_string(),
                args: vec![Expr::new(
                    ExprKind::StringLiteral("Hello, World!".to_string()),
                    dummy_span(),
                )],
            },
            dummy_span(),
        )),
        dummy_span(),
    )]);

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_ok());
}

#[test]
fn test_valid_program_with_variables() {
    let program = program_with_main(vec![
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
        Stmt::new(
            StmtKind::Expr(Expr::new(
                ExprKind::Call {
                    callee: "println".to_string(),
                    args: vec![Expr::new(
                        ExprKind::StringLiteral("done".to_string()),
                        dummy_span(),
                    )],
                },
                dummy_span(),
            )),
            dummy_span(),
        ),
    ]);

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_ok());
}

#[test]
fn test_valid_program_i32_boundary_values() {
    let program = program_with_main(vec![
        Stmt::new(
            StmtKind::Let {
                name: "min".to_string(),
                ty: Type::I32,
                init: Expr::new(ExprKind::IntLiteral(i32::MIN as i64), dummy_span()),
            },
            dummy_span(),
        ),
        Stmt::new(
            StmtKind::Let {
                name: "max".to_string(),
                ty: Type::I32,
                init: Expr::new(ExprKind::IntLiteral(i32::MAX as i64), dummy_span()),
            },
            dummy_span(),
        ),
    ]);

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_ok());
}
