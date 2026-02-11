//! Unit tests for the semantic analyzer.

use super::*;
use crate::ast::{Expr, ExprKind, FnDef, Program, Stmt, StmtKind, Type, UnaryOperator, Visibility};
use crate::token::Span;

mod function_tests;
mod symbol_table_tests;
mod type_tests;
mod variable_tests;

// ============================================================================
// Test helpers
// ============================================================================

pub fn dummy_span() -> Span {
    Span::new(0, 0, 1, 1)
}

pub fn span_at(line: usize, column: usize) -> Span {
    Span::new(0, 0, line, column)
}

/// Helper to create a minimal valid program with main
pub fn program_with_main(body: Vec<Stmt>) -> Program {
    Program {
        imports: vec![],
        functions: vec![FnDef {
            visibility: Visibility::Private,
            name: "main".to_string(),
            return_type: "void".to_string(),
            return_type_span: dummy_span(),
            body,
            span: dummy_span(),
        }],
    }
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
// Error constructor tests
// ============================================================================

#[test]
fn test_undefined_variable_constructor() {
    let err = SemanticError::undefined_variable("foo", span_at(10, 5));
    assert_eq!(err.kind(), SemanticErrorKind::UndefinedVariable);
    assert_eq!(err.message(), "Undefined variable: 'foo'");
    assert_eq!(err.span().unwrap().line, 10);
    assert_eq!(err.span().unwrap().column, 5);
}

#[test]
fn test_undefined_function_constructor() {
    let err = SemanticError::undefined_function("bar", span_at(20, 3));
    assert_eq!(err.kind(), SemanticErrorKind::UndefinedFunction);
    assert_eq!(err.message(), "Undefined function: 'bar'");
    assert_eq!(err.span().unwrap().line, 20);
}

#[test]
fn test_duplicate_variable_constructor() {
    let err = SemanticError::duplicate_variable("x", 5, 10, span_at(15, 8));
    assert_eq!(err.kind(), SemanticErrorKind::DuplicateVariable);
    assert_eq!(err.message(), "Variable 'x' is already defined at 5:10");
    assert_eq!(err.span().unwrap().line, 15);
}

#[test]
fn test_duplicate_function_constructor() {
    let err = SemanticError::duplicate_function("helper", 1, 1, span_at(50, 1));
    assert_eq!(err.kind(), SemanticErrorKind::DuplicateFunction);
    assert_eq!(err.message(), "Function 'helper' is already defined at 1:1");
}

#[test]
fn test_type_mismatch_int_to_string_constructor() {
    let err = SemanticError::type_mismatch_int_to_string(42, span_at(3, 5));
    assert_eq!(err.kind(), SemanticErrorKind::TypeMismatch);
    assert_eq!(
        err.message(),
        "Type mismatch: integer literal '42' cannot be assigned to type 'string'"
    );
}

#[test]
fn test_type_mismatch_variable_constructor() {
    let err = SemanticError::type_mismatch_variable("x", "i32", "i64", span_at(7, 1));
    assert_eq!(err.kind(), SemanticErrorKind::TypeMismatch);
    assert_eq!(
        err.message(),
        "Type mismatch: variable 'x' has type 'i32', expected 'i64'"
    );
}

#[test]
fn test_if_expression_branch_type_mismatch_constructor() {
    let err = SemanticError::if_expression_branch_type_mismatch("i64", "string", span_at(8, 9));
    assert_eq!(
        err.kind(),
        SemanticErrorKind::IfExpressionBranchTypeMismatch
    );
    assert_eq!(
        err.message(),
        "Type mismatch in if expression: then branch is 'i64', else branch is 'string'"
    );
}

#[test]
fn test_invalid_argument_println_count_constructor() {
    let err = SemanticError::invalid_argument_println_count(span_at(5, 5));
    assert_eq!(err.kind(), SemanticErrorKind::InvalidArgument);
    assert_eq!(err.message(), "println expects exactly 1 argument");
}

#[test]
fn test_reserved_prelude_function_name_constructor() {
    let err = SemanticError::reserved_prelude_function_name("println", span_at(2, 1));
    assert_eq!(err.kind(), SemanticErrorKind::InvalidArgument);
    assert_eq!(
        err.message(),
        "Function name 'println' is reserved by the prelude and cannot be redefined"
    );
    assert_eq!(
        err.help(),
        Some("use a different name; prelude names 'println' and 'panic' are reserved")
    );
}

#[test]
fn test_invalid_expression_string_literal_constructor() {
    let err = SemanticError::invalid_expression_string_literal(span_at(1, 1));
    assert_eq!(err.kind(), SemanticErrorKind::InvalidExpression);
    assert_eq!(
        err.message(),
        "String literal as a statement has no effect. Did you mean to pass it to a function?"
    );
}

#[test]
fn test_invalid_main_signature_constructor() {
    let err = SemanticError::invalid_main_signature("i32", span_at(1, 20));
    assert_eq!(err.kind(), SemanticErrorKind::InvalidMainSignature);
    assert_eq!(
        err.message(),
        "main function must return void, but found return type 'i32'"
    );
    // InvalidMainSignature has a span (pointing to return type)
    assert!(err.span().is_some());
}

#[test]
fn test_integer_overflow_i32_constructor() {
    let err = SemanticError::integer_overflow_i32(3_000_000_000, span_at(1, 1));
    assert_eq!(err.kind(), SemanticErrorKind::IntegerOverflow);
    assert_eq!(
        err.message(),
        "Integer literal '3000000000' is out of range for i32 (valid range: -2147483648 to 2147483647)"
    );
}

#[test]
fn test_internal_no_scope_constructor() {
    let err = SemanticError::internal_no_scope("x", span_at(1, 1));
    assert_eq!(err.kind(), SemanticErrorKind::InternalError);
    assert_eq!(
        err.message(),
        "Internal error: attempted to define variable 'x' outside a scope. This is a compiler bug."
    );
}

// ============================================================================
// Module table internal error constructor tests
// ============================================================================

#[test]
fn test_internal_function_export_empty_name_constructor() {
    let err = SemanticError::internal_function_export_empty_name(dummy_span());
    assert_eq!(err.kind(), SemanticErrorKind::InternalError);
    assert!(err.span().is_some());
    assert_eq!(
        err.message(),
        "Internal error: FunctionExport name must not be empty. This is a compiler bug."
    );
}

#[test]
fn test_internal_function_export_empty_return_type_constructor() {
    let err = SemanticError::internal_function_export_empty_return_type(dummy_span());
    assert_eq!(err.kind(), SemanticErrorKind::InternalError);
    assert!(err.span().is_some());
    assert_eq!(
        err.message(),
        "Internal error: FunctionExport return type must not be empty. This is a compiler bug."
    );
}

#[test]
fn test_internal_resolved_path_not_found_constructor() {
    let err = SemanticError::internal_resolved_path_not_found("./utils", dummy_span());
    assert_eq!(err.kind(), SemanticErrorKind::InternalError);
    assert!(err.span().is_some());
    assert_eq!(
        err.message(),
        "Internal error: resolved path for import './utils' not found. This is a compiler bug."
    );
}

#[test]
fn test_internal_resolved_module_not_found_constructor() {
    let err = SemanticError::internal_resolved_module_not_found("./utils", dummy_span());
    assert_eq!(err.kind(), SemanticErrorKind::InternalError);
    assert!(err.span().is_some());
    assert_eq!(
        err.message(),
        "Internal error: resolved module for './utils' not found in resolution map. This is a compiler bug."
    );
}

// ============================================================================
// Module error constructor tests
// ============================================================================

#[test]
fn test_module_access_not_implemented_constructor() {
    let err = SemanticError::module_access_not_implemented(dummy_span());
    assert_eq!(err.kind(), SemanticErrorKind::ModuleAccessNotImplemented);
    assert!(err.span().is_some());
    assert_eq!(
        err.message(),
        "Module-qualified access (e.g., module.function) is not yet implemented"
    );
    assert_eq!(
        err.help(),
        Some("only module function calls are supported: module.function()")
    );
}

#[test]
fn test_module_call_return_value_not_supported_constructor() {
    let err =
        SemanticError::module_call_return_value_not_supported("utils", "get_value", dummy_span());
    assert_eq!(err.kind(), SemanticErrorKind::TypeMismatch);
    assert!(err.span().is_some());
    assert_eq!(
        err.message(),
        "Module function call 'utils.get_value()' cannot be used as a value \
         (return values from module functions are not yet supported)"
    );
    assert_eq!(
        err.help(),
        Some("call the module function as a statement instead")
    );
}

#[test]
fn test_module_not_imported_constructor() {
    let err = SemanticError::module_not_imported("utils", "greet", dummy_span());
    assert_eq!(err.kind(), SemanticErrorKind::ModuleNotImported);
    assert!(err.span().is_some());
    assert_eq!(
        err.message(),
        "Module-qualified function call 'utils.greet()' requires an import statement. \
         Add: import \"./utils\""
    );
    assert!(err.help().is_none());
}

#[test]
fn test_undefined_module_constructor() {
    let err = SemanticError::undefined_module("math", dummy_span());
    assert_eq!(err.kind(), SemanticErrorKind::UndefinedModule);
    assert!(err.span().is_some());
    assert_eq!(err.message(), "Module 'math' is not defined");
    assert_eq!(
        err.help(),
        Some("Did you forget to import it? Add: import \"./module_name\"")
    );
}

#[test]
fn test_undefined_module_function_constructor() {
    let err = SemanticError::undefined_module_function("utils", "nonexistent", dummy_span());
    assert_eq!(err.kind(), SemanticErrorKind::UndefinedModuleFunction);
    assert!(err.span().is_some());
    assert_eq!(
        err.message(),
        "Function 'nonexistent' not found in module 'utils'"
    );
    assert_eq!(
        err.help(),
        Some("Check that the function exists in 'utils' and is marked 'pub'")
    );
}

#[test]
fn test_duplicate_module_import_constructor() {
    let err =
        SemanticError::duplicate_module_import("utils", "./utils", "../lib/utils", dummy_span());
    assert_eq!(err.kind(), SemanticErrorKind::DuplicateModuleImport);
    assert!(err.span().is_some());
    assert_eq!(
        err.message(),
        "Module name 'utils' is already imported from './utils'"
    );
    assert_eq!(
        err.help(),
        Some("Use an alias: import \"../lib/utils\" as <alias>")
    );
}

#[test]
fn test_cross_module_call_in_imported_module_constructor() {
    let err =
        SemanticError::cross_module_call_in_imported_module("helper", "do_work", dummy_span());
    assert_eq!(
        err.kind(),
        SemanticErrorKind::CrossModuleCallInImportedModule
    );
    assert!(err.span().is_some());
    assert_eq!(
        err.message(),
        "Cross-module function call 'helper.do_work()' in an imported module is not yet supported. \
         Imported modules cannot call functions from other imported modules."
    );
    assert!(err.help().is_none());
}

// ============================================================================
// Missing main function helper tests
// ============================================================================

#[test]
fn test_missing_main_no_functions_constructor() {
    let err = SemanticError::missing_main_no_functions();
    assert_eq!(err.kind(), SemanticErrorKind::MissingMainFunction);
    assert!(err.span().is_none());
    assert_eq!(
        err.message(),
        "No main function found: program contains no function definitions"
    );
}

#[test]
fn test_missing_main_with_functions_constructor() {
    let err = SemanticError::missing_main_with_functions(&["helper", "greet"]);
    assert_eq!(err.kind(), SemanticErrorKind::MissingMainFunction);
    assert!(err.span().is_none());
    assert_eq!(
        err.message(),
        "No main function found. Defined functions: 'helper', 'greet'"
    );
}

// ============================================================================
// wrap_in_unary_context tests
// ============================================================================

#[test]
fn test_wrap_in_unary_context_non_unary_error() {
    let base = SemanticError::undefined_variable("x", dummy_span());
    let wrapped = SemanticError::wrap_in_unary_context(&base, UnaryOperator::Neg, dummy_span());
    assert_eq!(wrapped.kind(), SemanticErrorKind::UndefinedVariable);
    assert_eq!(
        wrapped.message(),
        "in unary '-' operation: Undefined variable: 'x'"
    );
}

#[test]
fn test_wrap_in_unary_context_with_help_preserves_help() {
    let base = SemanticError::invalid_binary_op_type(
        crate::ast::BinaryOperator::Add,
        "string",
        dummy_span(),
    );
    assert!(base.help().is_some());
    let wrapped = SemanticError::wrap_in_unary_context(&base, UnaryOperator::Neg, dummy_span());
    assert_eq!(
        wrapped.message(),
        "in unary '-' operation: Operator '+' cannot be used with 'string' type"
    );
    assert_eq!(
        wrapped.help(),
        Some("arithmetic operators (+, -, *, /, %) only work with numeric types (i32, i64)")
    );
}

#[test]
fn test_wrap_in_unary_context_without_help_returns_none() {
    let base = SemanticError::undefined_variable("x", dummy_span());
    assert!(base.help().is_none());
    let wrapped = SemanticError::wrap_in_unary_context(&base, UnaryOperator::Neg, dummy_span());
    assert!(wrapped.help().is_none());
}

#[test]
fn test_wrap_in_unary_context_already_unary_preserves_original() {
    let base = SemanticError::invalid_unary_op_type(UnaryOperator::Neg, "string", dummy_span());
    let wrapped = SemanticError::wrap_in_unary_context(&base, UnaryOperator::Neg, dummy_span());
    // Should not double-wrap: message and help should be the original
    assert_eq!(wrapped.message(), base.message());
    assert_eq!(wrapped.help(), base.help());
}
