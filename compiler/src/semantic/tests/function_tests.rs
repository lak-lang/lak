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
                params: vec![],
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![],
                span: span_at(1, 1),
            },
            FnDef {
                visibility: Visibility::Private,
                name: "main".to_string(),
                params: vec![],
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
                params: vec![],
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![],
                span: dummy_span(),
            },
            FnDef {
                visibility: Visibility::Private,
                name: "helper".to_string(),
                params: vec![],
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![],
                span: span_at(5, 1),
            },
            FnDef {
                visibility: Visibility::Private,
                name: "helper".to_string(),
                params: vec![],
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

#[test]
fn test_reserved_prelude_function_println_error() {
    let program = Program {
        imports: vec![],
        functions: vec![
            FnDef {
                visibility: Visibility::Private,
                name: "println".to_string(),
                params: vec![],
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![],
                span: span_at(1, 1),
            },
            FnDef {
                visibility: Visibility::Private,
                name: "main".to_string(),
                params: vec![],
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![],
                span: span_at(3, 1),
            },
        ],
    };

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::InvalidArgument);
    assert_eq!(
        err.message(),
        "Function name 'println' is reserved by the prelude and cannot be redefined"
    );
}

#[test]
fn test_reserved_prelude_function_panic_error() {
    let program = Program {
        imports: vec![],
        functions: vec![
            FnDef {
                visibility: Visibility::Private,
                name: "panic".to_string(),
                params: vec![],
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![],
                span: span_at(1, 1),
            },
            FnDef {
                visibility: Visibility::Private,
                name: "main".to_string(),
                params: vec![],
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![],
                span: span_at(3, 1),
            },
        ],
    };

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::InvalidArgument);
    assert_eq!(
        err.message(),
        "Function name 'panic' is reserved by the prelude and cannot be redefined"
    );
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
            params: vec![],
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
        "No main function found. Defined functions: 'helper'"
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
            params: vec![],
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

#[test]
fn test_invalid_non_main_return_type_uses_return_type_span() {
    let invalid_return_type_span = span_at(1, 16);
    let program = Program {
        imports: vec![],
        functions: vec![
            FnDef {
                visibility: Visibility::Private,
                name: "helper".to_string(),
                params: vec![],
                return_type: "int".to_string(),
                return_type_span: invalid_return_type_span,
                body: vec![Stmt::new(
                    StmtKind::Return(Some(Expr::new(ExprKind::IntLiteral(1), span_at(2, 12)))),
                    span_at(2, 5),
                )],
                span: span_at(1, 1),
            },
            FnDef {
                visibility: Visibility::Private,
                name: "main".to_string(),
                params: vec![],
                return_type: "void".to_string(),
                return_type_span: span_at(4, 14),
                body: vec![],
                span: span_at(4, 1),
            },
        ],
    };

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::TypeMismatch);
    assert_eq!(
        err.message(),
        "Unsupported function return type 'int'. Expected 'void', 'i8', 'i16', 'i32', 'i64', 'u8', 'u16', 'u32', 'u64', 'f32', 'f64', 'byte', 'string', or 'bool'"
    );
    assert_eq!(err.span().unwrap(), invalid_return_type_span);
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
                params: vec![],
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![],
                span: dummy_span(),
            },
            FnDef {
                visibility: Visibility::Private,
                name: "main".to_string(),
                params: vec![],
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
                params: vec![],
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![],
                span: dummy_span(),
            },
            FnDef {
                visibility: Visibility::Private,
                name: "bar".to_string(),
                params: vec![],
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![],
                span: dummy_span(),
            },
            FnDef {
                visibility: Visibility::Private,
                name: "main".to_string(),
                params: vec![],
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

#[test]
fn test_call_user_defined_function_with_params() {
    let program = Program {
        imports: vec![],
        functions: vec![
            FnDef {
                visibility: Visibility::Private,
                name: "helper".to_string(),
                params: vec![
                    FnParam {
                        name: "name".to_string(),
                        ty: Type::String,
                        span: dummy_span(),
                    },
                    FnParam {
                        name: "age".to_string(),
                        ty: Type::I32,
                        span: dummy_span(),
                    },
                ],
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![
                    Stmt::new(
                        StmtKind::Expr(Expr::new(
                            ExprKind::Call {
                                callee: "println".to_string(),
                                args: vec![Expr::new(
                                    ExprKind::Identifier("name".to_string()),
                                    dummy_span(),
                                )],
                            },
                            dummy_span(),
                        )),
                        dummy_span(),
                    ),
                    Stmt::new(
                        StmtKind::Expr(Expr::new(
                            ExprKind::Call {
                                callee: "println".to_string(),
                                args: vec![Expr::new(
                                    ExprKind::Identifier("age".to_string()),
                                    dummy_span(),
                                )],
                            },
                            dummy_span(),
                        )),
                        dummy_span(),
                    ),
                ],
                span: dummy_span(),
            },
            FnDef {
                visibility: Visibility::Private,
                name: "main".to_string(),
                params: vec![],
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![Stmt::new(
                    StmtKind::Expr(Expr::new(
                        ExprKind::Call {
                            callee: "helper".to_string(),
                            args: vec![
                                Expr::new(
                                    ExprKind::StringLiteral("alice".to_string()),
                                    dummy_span(),
                                ),
                                Expr::new(ExprKind::IntLiteral(20), dummy_span()),
                            ],
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
fn test_call_user_defined_function_with_param_type_mismatch() {
    let program = Program {
        imports: vec![],
        functions: vec![
            FnDef {
                visibility: Visibility::Private,
                name: "helper".to_string(),
                params: vec![FnParam {
                    name: "name".to_string(),
                    ty: Type::String,
                    span: dummy_span(),
                }],
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![],
                span: dummy_span(),
            },
            FnDef {
                visibility: Visibility::Private,
                name: "main".to_string(),
                params: vec![],
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![Stmt::new(
                    StmtKind::Expr(Expr::new(
                        ExprKind::Call {
                            callee: "helper".to_string(),
                            args: vec![Expr::new(ExprKind::IntLiteral(42), dummy_span())],
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
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::TypeMismatch);
    assert_eq!(
        err.message(),
        "Type mismatch: integer literal '42' cannot be assigned to type 'string'"
    );
}

#[test]
fn test_main_function_with_params_error() {
    let program = Program {
        imports: vec![],
        functions: vec![FnDef {
            visibility: Visibility::Private,
            name: "main".to_string(),
            params: vec![FnParam {
                name: "x".to_string(),
                ty: Type::I32,
                span: dummy_span(),
            }],
            return_type: "void".to_string(),
            return_type_span: dummy_span(),
            body: vec![],
            span: span_at(1, 1),
        }],
    };

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::InvalidMainSignature);
    assert_eq!(
        err.message(),
        "main function must not have parameters, but found 1"
    );
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
                params: vec![],
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![Stmt::new(
                    StmtKind::Let {
                        is_mutable: false,
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
                params: vec![],
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![Stmt::new(
                    StmtKind::Let {
                        is_mutable: false,
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
                params: vec![],
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![Stmt::new(
                    StmtKind::Let {
                        is_mutable: false,
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
                params: vec![],
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![Stmt::new(
                    StmtKind::Let {
                        is_mutable: false,
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
                params: vec![],
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: vec![],
                span: dummy_span(),
            },
            FnDef {
                visibility: Visibility::Private,
                name: "helper".to_string(),
                params: vec![],
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
// Module call analysis tests (mode-dependent branching)
// ============================================================================

#[test]
fn test_module_call_in_single_file_mode() {
    // In SingleFile mode, a ModuleCall expression should return ModuleNotImported
    let program = program_with_main(vec![Stmt::new(
        StmtKind::Expr(Expr::new(
            ExprKind::ModuleCall {
                module: "utils".to_string(),
                function: "greet".to_string(),
                args: vec![],
            },
            dummy_span(),
        )),
        dummy_span(),
    )]);

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::ModuleNotImported);
    assert_eq!(
        err.message(),
        "Module-qualified function call 'utils.greet()' requires an import statement. Add: import \"./utils\""
    );
}

#[test]
fn test_module_call_in_imported_module_without_table() {
    // In ImportedModule(None) mode, a ModuleCall should return CrossModuleCallInImportedModule
    let program = Program {
        imports: vec![],
        functions: vec![FnDef {
            visibility: Visibility::Public,
            name: "helper".to_string(),
            params: vec![],
            return_type: "void".to_string(),
            return_type_span: dummy_span(),
            body: vec![Stmt::new(
                StmtKind::Expr(Expr::new(
                    ExprKind::ModuleCall {
                        module: "other".to_string(),
                        function: "foo".to_string(),
                        args: vec![],
                    },
                    dummy_span(),
                )),
                dummy_span(),
            )],
            span: dummy_span(),
        }],
    };

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze_module(&program, None);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(
        err.kind(),
        SemanticErrorKind::CrossModuleCallInImportedModule
    );
}

#[test]
fn test_module_call_undefined_module_in_entry() {
    // In EntryWithModules mode, calling a module that's not in the table
    // should return UndefinedModule
    let program = program_with_main(vec![Stmt::new(
        StmtKind::Expr(Expr::new(
            ExprKind::ModuleCall {
                module: "nonexistent".to_string(),
                function: "foo".to_string(),
                args: vec![],
            },
            dummy_span(),
        )),
        dummy_span(),
    )]);

    let module_table = crate::semantic::ModuleTable::new();
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze_with_modules(&program, module_table);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::UndefinedModule);
    assert_eq!(err.message(), "Module 'nonexistent' is not defined");
}

#[test]
fn test_module_call_undefined_function_in_entry() {
    // Module exists but function doesn't → UndefinedModuleFunction
    let program = program_with_main(vec![Stmt::new(
        StmtKind::Expr(Expr::new(
            ExprKind::ModuleCall {
                module: "utils".to_string(),
                function: "nonexistent".to_string(),
                args: vec![],
            },
            dummy_span(),
        )),
        dummy_span(),
    )]);

    let mut module_table = crate::semantic::ModuleTable::new();
    let exports = crate::semantic::module_table::ModuleExports::for_testing(
        "utils".to_string(),
        vec![("greet".to_string(), "void".to_string(), dummy_span())],
    )
    .unwrap();
    module_table.insert_for_testing("utils".to_string(), exports);

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze_with_modules(&program, module_table);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::UndefinedModuleFunction);
    assert_eq!(
        err.message(),
        "Function 'nonexistent' not found in module 'utils'"
    );
    assert_eq!(
        err.help().unwrap(),
        "Check that the function exists in 'utils' and is marked 'pub'"
    );
}

#[test]
fn test_module_call_non_void_function_as_stmt() {
    // Module function returns i32, called as statement → TypeMismatch
    let program = program_with_main(vec![Stmt::new(
        StmtKind::Expr(Expr::new(
            ExprKind::ModuleCall {
                module: "utils".to_string(),
                function: "get_value".to_string(),
                args: vec![],
            },
            dummy_span(),
        )),
        dummy_span(),
    )]);

    let mut module_table = crate::semantic::ModuleTable::new();
    let exports = crate::semantic::module_table::ModuleExports::for_testing(
        "utils".to_string(),
        vec![("get_value".to_string(), "i32".to_string(), dummy_span())],
    )
    .unwrap();
    module_table.insert_for_testing("utils".to_string(), exports);

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze_with_modules(&program, module_table);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::TypeMismatch);
    assert_eq!(
        err.message(),
        "Function 'utils.get_value' returns 'i32', but only void functions can be called as statements"
    );
    assert_eq!(
        err.help(),
        Some(
            "receive the value in a variable, or discard it explicitly: `let _ = utils.get_value(...)`"
        )
    );
}

#[test]
fn test_module_call_in_imported_module_with_table() {
    // ImportedModule(Some(table)) mode - cross-module call with empty table should fail
    let program = Program {
        imports: vec![],
        functions: vec![FnDef {
            visibility: Visibility::Public,
            name: "helper".to_string(),
            params: vec![],
            return_type: "void".to_string(),
            return_type_span: dummy_span(),
            body: vec![Stmt::new(
                StmtKind::Expr(Expr::new(
                    ExprKind::ModuleCall {
                        module: "other".to_string(),
                        function: "foo".to_string(),
                        args: vec![],
                    },
                    dummy_span(),
                )),
                dummy_span(),
            )],
            span: dummy_span(),
        }],
    };

    let module_table = crate::semantic::ModuleTable::new();
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze_module(&program, Some(module_table));
    assert!(result.is_err());
    let err = result.unwrap_err();
    // With empty table, module "other" is not found → UndefinedModule
    assert_eq!(err.kind(), SemanticErrorKind::UndefinedModule);
    assert_eq!(err.message(), "Module 'other' is not defined");
}

// ============================================================================
// Module call success tests
// ============================================================================

#[test]
fn test_module_call_success_in_entry() {
    // EntryWithModules mode: valid module function call should succeed
    let mut module_table = crate::semantic::ModuleTable::new();
    let exports = crate::semantic::module_table::ModuleExports::for_testing(
        "utils".to_string(),
        vec![("greet".to_string(), "void".to_string(), dummy_span())],
    )
    .unwrap();
    module_table.insert_for_testing("utils".to_string(), exports);

    let program = Program {
        imports: vec![],
        functions: vec![FnDef {
            visibility: Visibility::Private,
            name: "main".to_string(),
            params: vec![],
            return_type: "void".to_string(),
            return_type_span: dummy_span(),
            body: vec![Stmt::new(
                StmtKind::Expr(Expr::new(
                    ExprKind::ModuleCall {
                        module: "utils".to_string(),
                        function: "greet".to_string(),
                        args: vec![],
                    },
                    dummy_span(),
                )),
                dummy_span(),
            )],
            span: dummy_span(),
        }],
    };

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze_with_modules(&program, module_table);
    assert!(result.is_ok());
}

#[test]
fn test_module_call_success_with_arguments_in_entry() {
    let mut module_table = crate::semantic::ModuleTable::new();
    let exports = crate::semantic::module_table::ModuleExports::for_testing_with_params(
        "utils".to_string(),
        vec![(
            "greet".to_string(),
            vec![Type::String],
            "void".to_string(),
            dummy_span(),
        )],
    )
    .unwrap();
    module_table.insert_for_testing("utils".to_string(), exports);

    let program = Program {
        imports: vec![],
        functions: vec![FnDef {
            visibility: Visibility::Private,
            name: "main".to_string(),
            params: vec![],
            return_type: "void".to_string(),
            return_type_span: dummy_span(),
            body: vec![Stmt::new(
                StmtKind::Expr(Expr::new(
                    ExprKind::ModuleCall {
                        module: "utils".to_string(),
                        function: "greet".to_string(),
                        args: vec![Expr::new(
                            ExprKind::StringLiteral("hello".to_string()),
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

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze_with_modules(&program, module_table);
    assert!(result.is_ok());
}

#[test]
fn test_module_call_argument_type_mismatch_in_entry() {
    let mut module_table = crate::semantic::ModuleTable::new();
    let exports = crate::semantic::module_table::ModuleExports::for_testing_with_params(
        "utils".to_string(),
        vec![(
            "greet".to_string(),
            vec![Type::String],
            "void".to_string(),
            dummy_span(),
        )],
    )
    .unwrap();
    module_table.insert_for_testing("utils".to_string(), exports);

    let program = Program {
        imports: vec![],
        functions: vec![FnDef {
            visibility: Visibility::Private,
            name: "main".to_string(),
            params: vec![],
            return_type: "void".to_string(),
            return_type_span: dummy_span(),
            body: vec![Stmt::new(
                StmtKind::Expr(Expr::new(
                    ExprKind::ModuleCall {
                        module: "utils".to_string(),
                        function: "greet".to_string(),
                        args: vec![Expr::new(ExprKind::IntLiteral(10), dummy_span())],
                    },
                    dummy_span(),
                )),
                dummy_span(),
            )],
            span: dummy_span(),
        }],
    };

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze_with_modules(&program, module_table);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::TypeMismatch);
    assert_eq!(
        err.message(),
        "Type mismatch: integer literal '10' cannot be assigned to type 'string'"
    );
}

#[test]
fn test_analyze_module_success() {
    // ImportedModule mode: valid module program should succeed
    let program = Program {
        imports: vec![],
        functions: vec![FnDef {
            visibility: Visibility::Public,
            name: "greet".to_string(),
            params: vec![],
            return_type: "void".to_string(),
            return_type_span: dummy_span(),
            body: vec![],
            span: dummy_span(),
        }],
    };

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze_module(&program, None);
    assert!(result.is_ok());
}

// ============================================================================
// Analyzer session isolation tests
// ============================================================================

#[test]
fn test_analyze_reuse_does_not_leak_function_symbols() {
    let program = program_with_main(vec![]);
    let mut analyzer = SemanticAnalyzer::new();

    let first = analyzer.analyze(&program);
    assert!(first.is_ok());

    let second = analyzer.analyze(&program);
    assert!(second.is_ok());
}

#[test]
fn test_analyze_module_reuse_does_not_leak_function_symbols() {
    let program = Program {
        imports: vec![],
        functions: vec![FnDef {
            visibility: Visibility::Public,
            name: "helper".to_string(),
            params: vec![],
            return_type: "void".to_string(),
            return_type_span: dummy_span(),
            body: vec![],
            span: dummy_span(),
        }],
    };

    let mut analyzer = SemanticAnalyzer::new();
    let first = analyzer.analyze_module(&program, None);
    assert!(first.is_ok());

    let second = analyzer.analyze_module(&program, None);
    assert!(second.is_ok());
}

#[test]
fn test_analyze_reuse_resets_mode_after_analyze_with_modules() {
    let mut module_table = crate::semantic::ModuleTable::new();
    let exports = crate::semantic::module_table::ModuleExports::for_testing(
        "utils".to_string(),
        vec![("greet".to_string(), "void".to_string(), dummy_span())],
    )
    .unwrap();
    module_table.insert_for_testing("utils".to_string(), exports);

    let entry_program = program_with_main(vec![Stmt::new(
        StmtKind::Expr(Expr::new(
            ExprKind::ModuleCall {
                module: "utils".to_string(),
                function: "greet".to_string(),
                args: vec![],
            },
            dummy_span(),
        )),
        dummy_span(),
    )]);

    let single_file_program = program_with_main(vec![Stmt::new(
        StmtKind::Expr(Expr::new(
            ExprKind::ModuleCall {
                module: "utils".to_string(),
                function: "greet".to_string(),
                args: vec![],
            },
            dummy_span(),
        )),
        dummy_span(),
    )]);

    let mut analyzer = SemanticAnalyzer::new();

    let first = analyzer.analyze_with_modules(&entry_program, module_table);
    assert!(first.is_ok());

    let second = analyzer.analyze(&single_file_program);
    assert!(second.is_err());
    let err = second.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::ModuleNotImported);
    assert_eq!(
        err.message(),
        "Module-qualified function call 'utils.greet()' requires an import statement. Add: import \"./utils\""
    );
}

// ============================================================================
// while / break / continue tests
// ============================================================================

#[test]
fn test_break_outside_loop_error() {
    let program = program_with_main(vec![Stmt::new(StmtKind::Break, span_at(2, 5))]);

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::InvalidControlFlow);
    assert_eq!(
        err.message(),
        "break statement can only be used inside a loop"
    );
}

#[test]
fn test_continue_outside_loop_error() {
    let program = program_with_main(vec![Stmt::new(StmtKind::Continue, span_at(2, 5))]);

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::InvalidControlFlow);
    assert_eq!(
        err.message(),
        "continue statement can only be used inside a loop"
    );
}

#[test]
fn test_break_inside_if_outside_loop_error() {
    let program = program_with_main(vec![Stmt::new(
        StmtKind::If {
            condition: Expr::new(ExprKind::BoolLiteral(true), span_at(2, 8)),
            then_branch: vec![Stmt::new(StmtKind::Break, span_at(3, 9))],
            else_branch: None,
        },
        span_at(2, 5),
    )]);

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::InvalidControlFlow);
    assert_eq!(
        err.message(),
        "break statement can only be used inside a loop"
    );
}

#[test]
fn test_continue_inside_if_outside_loop_error() {
    let program = program_with_main(vec![Stmt::new(
        StmtKind::If {
            condition: Expr::new(ExprKind::BoolLiteral(true), span_at(2, 8)),
            then_branch: vec![Stmt::new(StmtKind::Continue, span_at(3, 9))],
            else_branch: None,
        },
        span_at(2, 5),
    )]);

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::InvalidControlFlow);
    assert_eq!(
        err.message(),
        "continue statement can only be used inside a loop"
    );
}

#[test]
fn test_while_condition_must_be_bool() {
    let program = program_with_main(vec![Stmt::new(
        StmtKind::While {
            condition: Expr::new(ExprKind::IntLiteral(1), span_at(2, 11)),
            body: vec![],
        },
        span_at(2, 5),
    )]);

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::TypeMismatch);
    assert_eq!(
        err.message(),
        "Type mismatch: integer literal '1' cannot be assigned to type 'bool'"
    );
}

#[test]
fn test_break_inside_if_within_while_is_valid() {
    let program = program_with_main(vec![Stmt::new(
        StmtKind::While {
            condition: Expr::new(ExprKind::BoolLiteral(true), span_at(2, 11)),
            body: vec![
                Stmt::new(
                    StmtKind::If {
                        condition: Expr::new(ExprKind::BoolLiteral(true), span_at(3, 12)),
                        then_branch: vec![Stmt::new(StmtKind::Break, span_at(4, 13))],
                        else_branch: None,
                    },
                    span_at(3, 9),
                ),
                Stmt::new(StmtKind::Break, span_at(6, 9)),
            ],
        },
        span_at(2, 5),
    )]);

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_ok());
}

#[test]
fn test_continue_inside_if_within_while_is_valid() {
    let program = program_with_main(vec![Stmt::new(
        StmtKind::While {
            condition: Expr::new(ExprKind::BoolLiteral(true), span_at(2, 11)),
            body: vec![
                Stmt::new(
                    StmtKind::If {
                        condition: Expr::new(ExprKind::BoolLiteral(true), span_at(3, 12)),
                        then_branch: vec![Stmt::new(StmtKind::Continue, span_at(4, 13))],
                        else_branch: None,
                    },
                    span_at(3, 9),
                ),
                Stmt::new(StmtKind::Break, span_at(6, 9)),
            ],
        },
        span_at(2, 5),
    )]);

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_ok());
}

#[test]
fn test_non_void_function_with_while_true_and_return_is_valid() {
    let program = Program {
        imports: vec![],
        functions: vec![
            FnDef {
                visibility: Visibility::Private,
                name: "helper".to_string(),
                params: vec![],
                return_type: "i64".to_string(),
                return_type_span: span_at(1, 16),
                body: vec![Stmt::new(
                    StmtKind::While {
                        condition: Expr::new(ExprKind::BoolLiteral(true), span_at(2, 11)),
                        body: vec![Stmt::new(
                            StmtKind::Return(Some(Expr::new(
                                ExprKind::IntLiteral(1),
                                span_at(3, 16),
                            ))),
                            span_at(3, 9),
                        )],
                    },
                    span_at(2, 5),
                )],
                span: span_at(1, 1),
            },
            FnDef {
                visibility: Visibility::Private,
                name: "main".to_string(),
                params: vec![],
                return_type: "void".to_string(),
                return_type_span: span_at(6, 14),
                body: vec![],
                span: span_at(6, 1),
            },
        ],
    };

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_ok());
}

#[test]
fn test_non_void_function_with_while_false_still_requires_return() {
    let program = Program {
        imports: vec![],
        functions: vec![
            FnDef {
                visibility: Visibility::Private,
                name: "helper".to_string(),
                params: vec![],
                return_type: "i64".to_string(),
                return_type_span: span_at(1, 16),
                body: vec![Stmt::new(
                    StmtKind::While {
                        condition: Expr::new(ExprKind::BoolLiteral(false), span_at(2, 11)),
                        body: vec![Stmt::new(
                            StmtKind::Return(Some(Expr::new(
                                ExprKind::IntLiteral(1),
                                span_at(3, 16),
                            ))),
                            span_at(3, 9),
                        )],
                    },
                    span_at(2, 5),
                )],
                span: span_at(1, 1),
            },
            FnDef {
                visibility: Visibility::Private,
                name: "main".to_string(),
                params: vec![],
                return_type: "void".to_string(),
                return_type_span: span_at(6, 14),
                body: vec![],
                span: span_at(6, 1),
            },
        ],
    };

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::TypeMismatch);
    assert_eq!(
        err.message(),
        "Function 'helper' with return type 'i64' must return a value on all code paths"
    );
}

#[test]
fn test_non_void_function_with_if_true_and_return_is_valid() {
    let program = Program {
        imports: vec![],
        functions: vec![
            FnDef {
                visibility: Visibility::Private,
                name: "helper".to_string(),
                params: vec![],
                return_type: "i64".to_string(),
                return_type_span: span_at(1, 16),
                body: vec![Stmt::new(
                    StmtKind::If {
                        condition: Expr::new(ExprKind::BoolLiteral(true), span_at(2, 8)),
                        then_branch: vec![Stmt::new(
                            StmtKind::Return(Some(Expr::new(
                                ExprKind::IntLiteral(1),
                                span_at(3, 16),
                            ))),
                            span_at(3, 9),
                        )],
                        else_branch: None,
                    },
                    span_at(2, 5),
                )],
                span: span_at(1, 1),
            },
            FnDef {
                visibility: Visibility::Private,
                name: "main".to_string(),
                params: vec![],
                return_type: "void".to_string(),
                return_type_span: span_at(6, 14),
                body: vec![],
                span: span_at(6, 1),
            },
        ],
    };

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_ok());
}

#[test]
fn test_non_void_function_with_if_not_false_and_return_is_valid() {
    let program = Program {
        imports: vec![],
        functions: vec![
            FnDef {
                visibility: Visibility::Private,
                name: "helper".to_string(),
                params: vec![],
                return_type: "i64".to_string(),
                return_type_span: span_at(1, 16),
                body: vec![Stmt::new(
                    StmtKind::If {
                        condition: Expr::new(
                            ExprKind::UnaryOp {
                                op: UnaryOperator::Not,
                                operand: Box::new(Expr::new(
                                    ExprKind::BoolLiteral(false),
                                    span_at(2, 9),
                                )),
                            },
                            span_at(2, 8),
                        ),
                        then_branch: vec![Stmt::new(
                            StmtKind::Return(Some(Expr::new(
                                ExprKind::IntLiteral(1),
                                span_at(3, 16),
                            ))),
                            span_at(3, 9),
                        )],
                        else_branch: None,
                    },
                    span_at(2, 5),
                )],
                span: span_at(1, 1),
            },
            FnDef {
                visibility: Visibility::Private,
                name: "main".to_string(),
                params: vec![],
                return_type: "void".to_string(),
                return_type_span: span_at(6, 14),
                body: vec![],
                span: span_at(6, 1),
            },
        ],
    };

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_ok());
}

#[test]
fn test_non_void_function_with_if_true_and_true_and_return_is_valid() {
    let program = Program {
        imports: vec![],
        functions: vec![
            FnDef {
                visibility: Visibility::Private,
                name: "helper".to_string(),
                params: vec![],
                return_type: "i64".to_string(),
                return_type_span: span_at(1, 16),
                body: vec![Stmt::new(
                    StmtKind::If {
                        condition: Expr::new(
                            ExprKind::BinaryOp {
                                left: Box::new(Expr::new(
                                    ExprKind::BoolLiteral(true),
                                    span_at(2, 8),
                                )),
                                op: BinaryOperator::LogicalAnd,
                                right: Box::new(Expr::new(
                                    ExprKind::BoolLiteral(true),
                                    span_at(2, 16),
                                )),
                            },
                            span_at(2, 8),
                        ),
                        then_branch: vec![Stmt::new(
                            StmtKind::Return(Some(Expr::new(
                                ExprKind::IntLiteral(1),
                                span_at(3, 16),
                            ))),
                            span_at(3, 9),
                        )],
                        else_branch: None,
                    },
                    span_at(2, 5),
                )],
                span: span_at(1, 1),
            },
            FnDef {
                visibility: Visibility::Private,
                name: "main".to_string(),
                params: vec![],
                return_type: "void".to_string(),
                return_type_span: span_at(6, 14),
                body: vec![],
                span: span_at(6, 1),
            },
        ],
    };

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_ok());
}

#[test]
fn test_non_void_function_with_if_false_and_no_else_still_requires_return() {
    let program = Program {
        imports: vec![],
        functions: vec![
            FnDef {
                visibility: Visibility::Private,
                name: "helper".to_string(),
                params: vec![],
                return_type: "i64".to_string(),
                return_type_span: span_at(1, 16),
                body: vec![Stmt::new(
                    StmtKind::If {
                        condition: Expr::new(ExprKind::BoolLiteral(false), span_at(2, 8)),
                        then_branch: vec![Stmt::new(
                            StmtKind::Return(Some(Expr::new(
                                ExprKind::IntLiteral(1),
                                span_at(3, 16),
                            ))),
                            span_at(3, 9),
                        )],
                        else_branch: None,
                    },
                    span_at(2, 5),
                )],
                span: span_at(1, 1),
            },
            FnDef {
                visibility: Visibility::Private,
                name: "main".to_string(),
                params: vec![],
                return_type: "void".to_string(),
                return_type_span: span_at(6, 14),
                body: vec![],
                span: span_at(6, 1),
            },
        ],
    };

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::TypeMismatch);
    assert_eq!(
        err.message(),
        "Function 'helper' with return type 'i64' must return a value on all code paths"
    );
}

#[test]
fn test_non_void_function_with_if_false_and_else_return_is_valid() {
    let program = Program {
        imports: vec![],
        functions: vec![
            FnDef {
                visibility: Visibility::Private,
                name: "helper".to_string(),
                params: vec![],
                return_type: "i64".to_string(),
                return_type_span: span_at(1, 16),
                body: vec![Stmt::new(
                    StmtKind::If {
                        condition: Expr::new(ExprKind::BoolLiteral(false), span_at(2, 8)),
                        then_branch: vec![],
                        else_branch: Some(vec![Stmt::new(
                            StmtKind::Return(Some(Expr::new(
                                ExprKind::IntLiteral(1),
                                span_at(5, 16),
                            ))),
                            span_at(5, 9),
                        )]),
                    },
                    span_at(2, 5),
                )],
                span: span_at(1, 1),
            },
            FnDef {
                visibility: Visibility::Private,
                name: "main".to_string(),
                params: vec![],
                return_type: "void".to_string(),
                return_type_span: span_at(8, 14),
                body: vec![],
                span: span_at(8, 1),
            },
        ],
    };

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_ok());
}
