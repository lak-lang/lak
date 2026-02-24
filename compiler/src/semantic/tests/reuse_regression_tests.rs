//! Regression tests for SemanticAnalyzer reuse behavior (RP-022).
//!
//! These tests ensure analyzer state does not leak across successive calls to:
//! - `analyze`
//! - `analyze_module`
//! - `analyze_with_modules`

use super::*;

fn program_calling_function(callee: &str) -> Program {
    program_with_main(vec![Stmt::new(
        StmtKind::Expr(Expr::new(
            ExprKind::Call {
                callee: callee.to_string(),
                args: vec![],
            },
            span_at(2, 5),
        )),
        span_at(2, 5),
    )])
}

fn program_calling_module(module: &str, function: &str) -> Program {
    program_with_main(vec![Stmt::new(
        StmtKind::Expr(Expr::new(
            ExprKind::ModuleCall {
                module: module.to_string(),
                function: function.to_string(),
                args: vec![],
            },
            span_at(2, 5),
        )),
        span_at(2, 5),
    )])
}

fn module_with_public_void_fn(name: &str) -> Program {
    Program {
        imports: vec![],
        functions: vec![FnDef {
            visibility: Visibility::Public,
            name: name.to_string(),
            params: vec![],
            return_type: "void".to_string(),
            return_type_span: span_at(1, 20),
            body: vec![],
            span: span_at(1, 1),
        }],
    }
}

fn module_table_with_void_fn(module: &str, function: &str) -> crate::semantic::ModuleTable {
    let mut module_table = crate::semantic::ModuleTable::new();
    let exports = crate::semantic::module_table::ModuleExports::for_testing(
        module.to_string(),
        vec![(function.to_string(), "void".to_string(), span_at(1, 1))],
    )
    .unwrap();
    module_table.insert_for_testing(module.to_string(), exports);
    module_table
}

fn duplicate_main_program() -> Program {
    Program {
        imports: vec![],
        functions: vec![
            FnDef {
                visibility: Visibility::Private,
                name: "main".to_string(),
                params: vec![],
                return_type: "void".to_string(),
                return_type_span: span_at(1, 15),
                body: vec![],
                span: span_at(1, 1),
            },
            FnDef {
                visibility: Visibility::Private,
                name: "main".to_string(),
                params: vec![],
                return_type: "void".to_string(),
                return_type_span: span_at(3, 15),
                body: vec![],
                span: span_at(3, 1),
            },
        ],
    }
}

fn duplicate_helper_module_program() -> Program {
    Program {
        imports: vec![],
        functions: vec![
            FnDef {
                visibility: Visibility::Public,
                name: "helper".to_string(),
                params: vec![],
                return_type: "void".to_string(),
                return_type_span: span_at(1, 17),
                body: vec![],
                span: span_at(1, 1),
            },
            FnDef {
                visibility: Visibility::Public,
                name: "helper".to_string(),
                params: vec![],
                return_type: "void".to_string(),
                return_type_span: span_at(3, 17),
                body: vec![],
                span: span_at(3, 1),
            },
        ],
    }
}

fn valid_variable_program() -> Program {
    program_with_main(vec![Stmt::new(
        StmtKind::Let {
            is_mutable: false,
            name: "x".to_string(),
            ty: Type::I32,
            init: Expr::new(ExprKind::IntLiteral(1), span_at(2, 18)),
        },
        span_at(2, 5),
    )])
}

fn duplicate_variable_program() -> Program {
    program_with_main(vec![
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
            StmtKind::Let {
                is_mutable: false,
                name: "x".to_string(),
                ty: Type::I32,
                init: Expr::new(ExprKind::IntLiteral(2), span_at(3, 18)),
            },
            span_at(3, 5),
        ),
    ])
}

#[test]
fn test_reuse_across_entry_points_keeps_mode_isolated() {
    let mut analyzer = SemanticAnalyzer::new();

    let first = analyzer.analyze(&program_with_main(vec![]));
    assert!(first.is_ok());

    let second = analyzer.analyze_module(&module_with_public_void_fn("helper"), None);
    assert!(second.is_ok());

    let third = analyzer.analyze_with_modules(
        &program_calling_module("utils", "greet"),
        module_table_with_void_fn("utils", "greet"),
    );
    assert!(third.is_ok());

    let fourth = analyzer.analyze(&program_calling_module("utils", "greet"));
    assert!(fourth.is_err());
    let err = fourth.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::ModuleNotImported);
    assert_eq!(
        err.message(),
        "Module-qualified function call 'utils.greet()' requires an import statement. Add: import \"./utils\""
    );
}

#[test]
fn test_analyze_module_then_analyze_does_not_leak_function_symbols() {
    let mut analyzer = SemanticAnalyzer::new();

    let first = analyzer.analyze_module(&module_with_public_void_fn("helper"), None);
    assert!(first.is_ok());

    let second = analyzer.analyze(&program_calling_function("helper"));
    assert!(second.is_err());
    let err = second.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::UndefinedFunction);
    assert_eq!(err.message(), "Undefined function: 'helper'");
}

#[test]
fn test_analyze_reuse_does_not_leak_variable_bindings_between_programs() {
    let mut analyzer = SemanticAnalyzer::new();

    let first = analyzer.analyze(&valid_variable_program());
    assert!(first.is_ok());

    let second = analyzer.analyze(&program_with_main(vec![Stmt::new(
        StmtKind::Assign {
            name: "x".to_string(),
            value: Expr::new(ExprKind::IntLiteral(2), span_at(2, 9)),
        },
        span_at(2, 5),
    )]));
    assert!(second.is_err());
    let err = second.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::UndefinedVariable);
    assert_eq!(err.message(), "Undefined variable: 'x'");
}

#[test]
fn test_analyze_with_modules_reuse_uses_fresh_module_table() {
    let mut analyzer = SemanticAnalyzer::new();
    let program = program_calling_module("utils", "greet");

    let first =
        analyzer.analyze_with_modules(&program, module_table_with_void_fn("utils", "greet"));
    assert!(first.is_ok());

    let second = analyzer.analyze_with_modules(&program, crate::semantic::ModuleTable::new());
    assert!(second.is_err());
    let err = second.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::UndefinedModule);
    assert_eq!(err.message(), "Module 'utils' is not defined");
}

#[test]
fn test_duplicate_function_contract_in_analyze_after_reuse() {
    let mut analyzer = SemanticAnalyzer::new();

    let first = analyzer.analyze(&program_with_main(vec![]));
    assert!(first.is_ok());

    let second = analyzer.analyze(&duplicate_main_program());
    assert!(second.is_err());
    let err = second.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::DuplicateFunction);
    assert_eq!(err.message(), "Function 'main' is already defined at 1:1");
}

#[test]
fn test_duplicate_function_contract_in_analyze_module_after_reuse() {
    let mut analyzer = SemanticAnalyzer::new();

    let first = analyzer.analyze_module(&module_with_public_void_fn("helper"), None);
    assert!(first.is_ok());

    let second = analyzer.analyze_module(&duplicate_helper_module_program(), None);
    assert!(second.is_err());
    let err = second.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::DuplicateFunction);
    assert_eq!(err.message(), "Function 'helper' is already defined at 1:1");
}

#[test]
fn test_duplicate_function_contract_in_analyze_with_modules_after_reuse() {
    let mut analyzer = SemanticAnalyzer::new();

    let first = analyzer.analyze_with_modules(
        &program_with_main(vec![]),
        crate::semantic::ModuleTable::new(),
    );
    assert!(first.is_ok());

    let second = analyzer.analyze_with_modules(
        &duplicate_main_program(),
        crate::semantic::ModuleTable::new(),
    );
    assert!(second.is_err());
    let err = second.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::DuplicateFunction);
    assert_eq!(err.message(), "Function 'main' is already defined at 1:1");
}

#[test]
fn test_duplicate_variable_contract_in_analyze_after_reuse() {
    let mut analyzer = SemanticAnalyzer::new();

    let first = analyzer.analyze(&valid_variable_program());
    assert!(first.is_ok());

    let second = analyzer.analyze(&duplicate_variable_program());
    assert!(second.is_err());
    let err = second.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::DuplicateVariable);
    assert_eq!(err.message(), "Variable 'x' is already defined at 2:5");
}
