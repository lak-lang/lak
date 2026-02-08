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
    assert_eq!(
        err.message(),
        "Type mismatch: variable 'x' has type 'i32', expected 'i64'"
    );
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
    assert_eq!(
        err.message(),
        "Type mismatch: string literal cannot be assigned to type 'i32'"
    );
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
    assert_eq!(
        err.message(),
        "Integer literal '2147483648' is out of range for i32 (valid range: -2147483648 to 2147483647)"
    );
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
    assert_eq!(
        err.message(),
        "Integer literal '-2147483649' is out of range for i32 (valid range: -2147483648 to 2147483647)"
    );
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
    assert_eq!(
        err.message(),
        "String literal as a statement has no effect. Did you mean to pass it to a function?"
    );
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
    assert_eq!(err.message(), "println expects exactly 1 argument");
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
fn test_println_with_integer_literal() {
    // println now accepts integer literals (any type support)
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
    assert!(result.is_ok());
}

#[test]
fn test_println_with_i32_variable_argument() {
    // println now accepts i32 variables (any type support)
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
    assert!(result.is_ok());
}

#[test]
fn test_println_with_i64_variable_argument() {
    // println now accepts i64 variables (any type support)
    let program = program_with_main(vec![
        Stmt::new(
            StmtKind::Let {
                name: "y".to_string(),
                ty: Type::I64,
                init: Expr::new(ExprKind::IntLiteral(9223372036854775807), dummy_span()),
            },
            dummy_span(),
        ),
        Stmt::new(
            StmtKind::Expr(Expr::new(
                ExprKind::Call {
                    callee: "println".to_string(),
                    args: vec![Expr::new(
                        ExprKind::Identifier("y".to_string()),
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
    assert!(result.is_ok());
}

#[test]
fn test_println_with_undefined_variable() {
    // println with undefined variable should still fail
    let program = program_with_main(vec![Stmt::new(
        StmtKind::Expr(Expr::new(
            ExprKind::Call {
                callee: "println".to_string(),
                args: vec![Expr::new(
                    ExprKind::Identifier("undefined_var".to_string()),
                    span_at(2, 13),
                )],
            },
            span_at(2, 5),
        )),
        dummy_span(),
    )]);

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::UndefinedVariable);
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

// ============================================================================
// Unary operator tests
// ============================================================================

#[test]
fn test_unary_minus_on_string_error() {
    // Unary minus should not work on string type
    let program = program_with_main(vec![Stmt::new(
        StmtKind::Let {
            name: "x".to_string(),
            ty: Type::String,
            init: Expr::new(
                ExprKind::UnaryOp {
                    op: UnaryOperator::Neg,
                    operand: Box::new(Expr::new(
                        ExprKind::StringLiteral("hello".to_string()),
                        dummy_span(),
                    )),
                },
                span_at(2, 18),
            ),
        },
        dummy_span(),
    )]);

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::TypeMismatch);
    assert_eq!(
        err.message(),
        "Unary operator '-' cannot be used with 'string' type"
    );
}

#[test]
fn test_unary_minus_on_i32_valid() {
    let program = program_with_main(vec![Stmt::new(
        StmtKind::Let {
            name: "x".to_string(),
            ty: Type::I32,
            init: Expr::new(
                ExprKind::UnaryOp {
                    op: UnaryOperator::Neg,
                    operand: Box::new(Expr::new(ExprKind::IntLiteral(5), dummy_span())),
                },
                dummy_span(),
            ),
        },
        dummy_span(),
    )]);

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_ok());
}

#[test]
fn test_unary_minus_as_statement_error() {
    // Unary operations as statements should be rejected
    let program = program_with_main(vec![
        Stmt::new(
            StmtKind::Let {
                name: "x".to_string(),
                ty: Type::I32,
                init: Expr::new(ExprKind::IntLiteral(5), dummy_span()),
            },
            dummy_span(),
        ),
        Stmt::new(
            StmtKind::Expr(Expr::new(
                ExprKind::UnaryOp {
                    op: UnaryOperator::Neg,
                    operand: Box::new(Expr::new(
                        ExprKind::Identifier("x".to_string()),
                        dummy_span(),
                    )),
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
    assert_eq!(err.kind(), SemanticErrorKind::InvalidExpression);
}

#[test]
fn test_unary_minus_type_mismatch_i32_to_i64() {
    // Unary minus on i32 variable assigned to i64 should fail
    let program = program_with_main(vec![
        Stmt::new(
            StmtKind::Let {
                name: "x".to_string(),
                ty: Type::I32,
                init: Expr::new(ExprKind::IntLiteral(5), dummy_span()),
            },
            dummy_span(),
        ),
        Stmt::new(
            StmtKind::Let {
                name: "y".to_string(),
                ty: Type::I64,
                init: Expr::new(
                    ExprKind::UnaryOp {
                        op: UnaryOperator::Neg,
                        operand: Box::new(Expr::new(
                            ExprKind::Identifier("x".to_string()),
                            dummy_span(),
                        )),
                    },
                    span_at(3, 18),
                ),
            },
            dummy_span(),
        ),
    ]);

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::TypeMismatch);
    assert_eq!(
        err.message(),
        "in unary '-' operation: Type mismatch: variable 'x' has type 'i32', expected 'i64'"
    );
}

#[test]
fn test_unary_minus_on_string_variable_in_println() {
    // println(-string_var) should fail
    let program = program_with_main(vec![
        Stmt::new(
            StmtKind::Let {
                name: "s".to_string(),
                ty: Type::String,
                init: Expr::new(ExprKind::StringLiteral("hello".to_string()), dummy_span()),
            },
            dummy_span(),
        ),
        Stmt::new(
            StmtKind::Expr(Expr::new(
                ExprKind::Call {
                    callee: "println".to_string(),
                    args: vec![Expr::new(
                        ExprKind::UnaryOp {
                            op: UnaryOperator::Neg,
                            operand: Box::new(Expr::new(
                                ExprKind::Identifier("s".to_string()),
                                dummy_span(),
                            )),
                        },
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
    assert_eq!(err.kind(), SemanticErrorKind::TypeMismatch);
    assert_eq!(
        err.message(),
        "Unary operator '-' cannot be used with 'string' type"
    );
}

#[test]
fn test_unary_minus_on_string_literal_in_println() {
    // println(-"hello") should fail
    let program = program_with_main(vec![Stmt::new(
        StmtKind::Expr(Expr::new(
            ExprKind::Call {
                callee: "println".to_string(),
                args: vec![Expr::new(
                    ExprKind::UnaryOp {
                        op: UnaryOperator::Neg,
                        operand: Box::new(Expr::new(
                            ExprKind::StringLiteral("hello".to_string()),
                            dummy_span(),
                        )),
                    },
                    span_at(2, 13),
                )],
            },
            span_at(2, 5),
        )),
        dummy_span(),
    )]);

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::TypeMismatch);
    assert_eq!(
        err.message(),
        "Unary operator '-' cannot be used with 'string' type"
    );
}
