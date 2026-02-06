//! Tests for function-related semantic analysis.
//!
//! Covers:
//! - Duplicate function definitions
//! - Missing main function
//! - Invalid main signature
//! - Undefined function calls
//! - User-defined function calls
//! - Function scope isolation

use super::*;

// ============================================================================
// Duplicate function tests
// ============================================================================

#[test]
fn test_duplicate_function_error() {
    let program = Program {
        functions: vec![
            FnDef {
                name: "main".to_string(),
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![],
                span: span_at(1, 1),
            },
            FnDef {
                name: "main".to_string(),
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![],
                span: span_at(5, 1),
            },
        ],
    };

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::DuplicateFunction);
    assert!(err.message().contains("already defined"));
}

#[test]
fn test_duplicate_non_main_function_error() {
    let program = Program {
        functions: vec![
            FnDef {
                name: "main".to_string(),
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![],
                span: dummy_span(),
            },
            FnDef {
                name: "helper".to_string(),
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![],
                span: span_at(5, 1),
            },
            FnDef {
                name: "helper".to_string(),
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![],
                span: span_at(10, 1),
            },
        ],
    };

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::DuplicateFunction);
    assert!(err.message().contains("helper"));
}

// ============================================================================
// Missing main function tests
// ============================================================================

#[test]
fn test_missing_main_function_empty_program() {
    let program = Program { functions: vec![] };

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::MissingMainFunction);
    assert!(err.message().contains("No main function found"));
}

#[test]
fn test_missing_main_function_with_other_functions() {
    let program = Program {
        functions: vec![FnDef {
            name: "helper".to_string(),
            return_type: "void".to_string(),
            return_type_span: dummy_span(),
            body: vec![],
            span: dummy_span(),
        }],
    };

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::MissingMainFunction);
    assert!(err.message().contains("Defined functions"));
}

// ============================================================================
// Invalid main signature tests
// ============================================================================

#[test]
fn test_invalid_main_signature() {
    let program = Program {
        functions: vec![FnDef {
            name: "main".to_string(),
            return_type: "int".to_string(),
            return_type_span: span_at(1, 15),
            body: vec![],
            span: dummy_span(),
        }],
    };

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::InvalidMainSignature);
    assert!(err.message().contains("must return void"));
}

// ============================================================================
// Undefined function tests
// ============================================================================

#[test]
fn test_undefined_function() {
    let program = program_with_main(vec![Stmt::new(
        StmtKind::Expr(Expr::new(
            ExprKind::Call {
                callee: "unknown".to_string(),
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
    assert_eq!(err.kind(), SemanticErrorKind::UndefinedFunction);
    assert!(err.message().contains("Undefined function"));
}

// ============================================================================
// User-defined function call tests
// ============================================================================

#[test]
fn test_call_user_defined_function() {
    // Call a user-defined function from main
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
                body: vec![Stmt::new(
                    StmtKind::Expr(Expr::new(
                        ExprKind::Call {
                            callee: "helper".to_string(),
                            args: vec![],
                        },
                        dummy_span(),
                    )),
                    dummy_span(),
                )],
                span: dummy_span(),
            },
        ],
    };

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_ok());
}

#[test]
fn test_call_multiple_user_defined_functions() {
    // Multiple user-defined functions calling each other concept
    let program = Program {
        functions: vec![
            FnDef {
                name: "foo".to_string(),
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![],
                span: dummy_span(),
            },
            FnDef {
                name: "bar".to_string(),
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![],
                span: dummy_span(),
            },
            FnDef {
                name: "main".to_string(),
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![
                    Stmt::new(
                        StmtKind::Expr(Expr::new(
                            ExprKind::Call {
                                callee: "foo".to_string(),
                                args: vec![],
                            },
                            dummy_span(),
                        )),
                        dummy_span(),
                    ),
                    Stmt::new(
                        StmtKind::Expr(Expr::new(
                            ExprKind::Call {
                                callee: "bar".to_string(),
                                args: vec![],
                            },
                            dummy_span(),
                        )),
                        dummy_span(),
                    ),
                ],
                span: dummy_span(),
            },
        ],
    };

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_ok());
}

// ============================================================================
// Function scope isolation tests
// ============================================================================

#[test]
fn test_function_variable_scope_isolation() {
    // Same variable name in different functions should be allowed
    let program = Program {
        functions: vec![
            FnDef {
                name: "helper".to_string(),
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![Stmt::new(
                    StmtKind::Let {
                        name: "x".to_string(),
                        ty: Type::I32,
                        init: Expr::new(ExprKind::IntLiteral(1), dummy_span()),
                    },
                    dummy_span(),
                )],
                span: dummy_span(),
            },
            FnDef {
                name: "main".to_string(),
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![Stmt::new(
                    StmtKind::Let {
                        name: "x".to_string(), // Same name as in helper, should be OK
                        ty: Type::I32,
                        init: Expr::new(ExprKind::IntLiteral(2), dummy_span()),
                    },
                    dummy_span(),
                )],
                span: dummy_span(),
            },
        ],
    };

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_ok());
}

#[test]
fn test_function_scope_isolation_different_types() {
    // Same variable name with different types in different functions
    let program = Program {
        functions: vec![
            FnDef {
                name: "helper".to_string(),
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![Stmt::new(
                    StmtKind::Let {
                        name: "value".to_string(),
                        ty: Type::I32,
                        init: Expr::new(ExprKind::IntLiteral(100), dummy_span()),
                    },
                    dummy_span(),
                )],
                span: dummy_span(),
            },
            FnDef {
                name: "main".to_string(),
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![Stmt::new(
                    StmtKind::Let {
                        name: "value".to_string(), // Same name, different type
                        ty: Type::I64,
                        init: Expr::new(ExprKind::IntLiteral(200), dummy_span()),
                    },
                    dummy_span(),
                )],
                span: dummy_span(),
            },
        ],
    };

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_ok());
}

// ============================================================================
// Valid multiple functions test
// ============================================================================

#[test]
fn test_valid_multiple_functions() {
    let program = Program {
        functions: vec![
            FnDef {
                name: "main".to_string(),
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![],
                span: dummy_span(),
            },
            FnDef {
                name: "helper".to_string(),
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![],
                span: dummy_span(),
            },
        ],
    };

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_ok());
}
