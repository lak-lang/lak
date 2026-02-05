//! Unit tests for the semantic analyzer.

use super::*;
use crate::ast::{Expr, ExprKind, FnDef, Program, Stmt, StmtKind, Type};
use crate::token::Span;

fn dummy_span() -> Span {
    Span::new(0, 0, 1, 1)
}

fn span_at(line: usize, column: usize) -> Span {
    Span::new(0, 0, line, column)
}

// Helper to create a minimal valid program with main
fn program_with_main(body: Vec<Stmt>) -> Program {
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
    assert!(err.message().contains("already defined"));
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
    assert!(err.message().contains("Undefined variable"));
    assert!(err.message().contains("'y'"));
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
    assert!(err.message().contains("Unknown function"));
}

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
// println with variable argument tests
// ============================================================================

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
// SemanticError Display tests
// ============================================================================

#[test]
fn test_semantic_error_display_with_span() {
    let err = SemanticError::new(
        SemanticErrorKind::UndefinedVariable,
        "Undefined variable: 'x'",
        span_at(5, 10),
    );
    let display = format!("{}", err);
    assert_eq!(display, "5:10: Undefined variable: 'x'");
}

#[test]
fn test_semantic_error_display_without_span() {
    let err = SemanticError::without_span(
        SemanticErrorKind::MissingMainFunction,
        "No main function found",
    );
    let display = format!("{}", err);
    assert_eq!(display, "No main function found");
}

#[test]
fn test_semantic_error_missing_main_display() {
    let err = SemanticError::missing_main("No main function found in the program");
    let display = format!("{}", err);
    assert_eq!(display, "No main function found in the program");
    assert!(err.span().is_none());
    assert_eq!(err.kind(), SemanticErrorKind::MissingMainFunction);
}

// ============================================================================
// SymbolTable unit tests
// ============================================================================

mod symbol_table_tests {
    use super::*;
    use crate::semantic::symbol::{FunctionInfo, SymbolTable, VariableInfo};

    #[test]
    fn test_symbol_table_new() {
        let table = SymbolTable::new();
        assert!(table.lookup_function("main").is_none());
    }

    #[test]
    fn test_symbol_table_default() {
        let table = SymbolTable::default();
        assert!(table.lookup_function("any").is_none());
    }

    #[test]
    fn test_define_and_lookup_function() {
        let mut table = SymbolTable::new();
        let info = FunctionInfo {
            name: "test_fn".to_string(),
            return_type: "void".to_string(),
            return_type_span: dummy_span(),
            definition_span: span_at(1, 1),
        };

        assert!(table.define_function(info).is_ok());
        assert!(table.lookup_function("test_fn").is_some());
        assert_eq!(table.lookup_function("test_fn").unwrap().name, "test_fn");
    }

    #[test]
    fn test_duplicate_function_error() {
        let mut table = SymbolTable::new();
        let info1 = FunctionInfo {
            name: "dup".to_string(),
            return_type: "void".to_string(),
            return_type_span: dummy_span(),
            definition_span: span_at(1, 1),
        };
        let info2 = FunctionInfo {
            name: "dup".to_string(),
            return_type: "void".to_string(),
            return_type_span: dummy_span(),
            definition_span: span_at(5, 1),
        };

        assert!(table.define_function(info1).is_ok());
        let result = table.define_function(info2);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already defined"));
    }

    #[test]
    fn test_enter_and_exit_scope() {
        let mut table = SymbolTable::new();

        table.enter_scope();
        let info = VariableInfo {
            name: "x".to_string(),
            ty: Type::I32,
            definition_span: dummy_span(),
        };
        assert!(table.define_variable(info).is_ok());
        assert!(table.lookup_variable("x").is_some());

        table.exit_scope();
        assert!(table.lookup_variable("x").is_none());
    }

    #[test]
    fn test_define_variable_outside_scope_error() {
        let mut table = SymbolTable::new();
        let info = VariableInfo {
            name: "orphan".to_string(),
            ty: Type::I32,
            definition_span: dummy_span(),
        };

        let result = table.define_variable(info);
        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert!(err_msg.contains("Internal error"));
        assert!(err_msg.contains("orphan"));
    }

    #[test]
    fn test_duplicate_variable_in_same_scope() {
        let mut table = SymbolTable::new();
        table.enter_scope();

        let info1 = VariableInfo {
            name: "x".to_string(),
            ty: Type::I32,
            definition_span: span_at(1, 1),
        };
        let info2 = VariableInfo {
            name: "x".to_string(),
            ty: Type::I64,
            definition_span: span_at(2, 1),
        };

        assert!(table.define_variable(info1).is_ok());
        let result = table.define_variable(info2);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already defined"));
    }

    #[test]
    fn test_lookup_variable_searches_outer_scopes() {
        let mut table = SymbolTable::new();

        // Outer scope
        table.enter_scope();
        let outer_var = VariableInfo {
            name: "outer".to_string(),
            ty: Type::I32,
            definition_span: dummy_span(),
        };
        assert!(table.define_variable(outer_var).is_ok());

        // Inner scope
        table.enter_scope();
        let inner_var = VariableInfo {
            name: "inner".to_string(),
            ty: Type::I64,
            definition_span: dummy_span(),
        };
        assert!(table.define_variable(inner_var).is_ok());

        // Should find both variables from inner scope
        assert!(table.lookup_variable("inner").is_some());
        assert!(table.lookup_variable("outer").is_some());

        // Exit inner scope
        table.exit_scope();

        // Should only find outer variable now
        assert!(table.lookup_variable("inner").is_none());
        assert!(table.lookup_variable("outer").is_some());
    }

    #[test]
    fn test_inner_scope_shadows_outer() {
        let mut table = SymbolTable::new();

        // Outer scope with x: i32
        table.enter_scope();
        let outer_x = VariableInfo {
            name: "x".to_string(),
            ty: Type::I32,
            definition_span: span_at(1, 1),
        };
        assert!(table.define_variable(outer_x).is_ok());

        // Inner scope with x: i64 (shadows outer)
        table.enter_scope();
        let inner_x = VariableInfo {
            name: "x".to_string(),
            ty: Type::I64,
            definition_span: span_at(3, 1),
        };
        assert!(table.define_variable(inner_x).is_ok());

        // Looking up x should find the inner (i64) version
        let found = table.lookup_variable("x").unwrap();
        assert_eq!(found.ty, Type::I64);

        // Exit inner scope
        table.exit_scope();

        // Now x should be the outer (i32) version
        let found = table.lookup_variable("x").unwrap();
        assert_eq!(found.ty, Type::I32);
    }
}
