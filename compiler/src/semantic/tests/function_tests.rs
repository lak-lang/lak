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
        imports: vec![],
        functions: vec![
            FnDef {
                visibility: Visibility::Private,
                name: "main".to_string(),
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![],
                span: span_at(1, 1),
            },
            FnDef {
                visibility: Visibility::Private,
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
    assert_eq!(err.message(), "Function 'main' is already defined at 1:1");
}

#[test]
fn test_duplicate_non_main_function_error() {
    let program = Program {
        imports: vec![],
        functions: vec![
            FnDef {
                visibility: Visibility::Private,
                name: "main".to_string(),
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![],
                span: dummy_span(),
            },
            FnDef {
                visibility: Visibility::Private,
                name: "helper".to_string(),
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![],
                span: span_at(5, 1),
            },
            FnDef {
                visibility: Visibility::Private,
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
    assert_eq!(err.message(), "Function 'helper' is already defined at 5:1");
}

// ============================================================================
// Missing main function tests
// ============================================================================

#[test]
fn test_missing_main_function_empty_program() {
    let program = Program {
        imports: vec![],
        functions: vec![],
    };

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::MissingMainFunction);
    assert_eq!(
        err.message(),
        "No main function found: program contains no function definitions"
    );
}

#[test]
fn test_missing_main_function_with_other_functions() {
    let program = Program {
        imports: vec![],
        functions: vec![FnDef {
            visibility: Visibility::Private,
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
    assert_eq!(
        err.message(),
        "No main function found. Defined functions: [\"helper\"]"
    );
}

// ============================================================================
// Invalid main signature tests
// ============================================================================

#[test]
fn test_invalid_main_signature() {
    let program = Program {
        imports: vec![],
        functions: vec![FnDef {
            visibility: Visibility::Private,
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
    assert_eq!(
        err.message(),
        "main function must return void, but found return type 'int'"
    );
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
    assert_eq!(err.message(), "Undefined function: 'unknown'");
}

// ============================================================================
// User-defined function call tests
// ============================================================================

#[test]
fn test_call_user_defined_function() {
    // Call a user-defined function from main
    let program = Program {
        imports: vec![],
        functions: vec![
            FnDef {
                visibility: Visibility::Private,
                name: "helper".to_string(),
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![],
                span: dummy_span(),
            },
            FnDef {
                visibility: Visibility::Private,
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
        imports: vec![],
        functions: vec![
            FnDef {
                visibility: Visibility::Private,
                name: "foo".to_string(),
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![],
                span: dummy_span(),
            },
            FnDef {
                visibility: Visibility::Private,
                name: "bar".to_string(),
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![],
                span: dummy_span(),
            },
            FnDef {
                visibility: Visibility::Private,
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
        imports: vec![],
        functions: vec![
            FnDef {
                visibility: Visibility::Private,
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
                visibility: Visibility::Private,
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
        imports: vec![],
        functions: vec![
            FnDef {
                visibility: Visibility::Private,
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
                visibility: Visibility::Private,
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
        imports: vec![],
        functions: vec![
            FnDef {
                visibility: Visibility::Private,
                name: "main".to_string(),
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![],
                span: dummy_span(),
            },
            FnDef {
                visibility: Visibility::Private,
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
