//! Unit tests for the SymbolTable data structure.
//!
//! Covers:
//! - Function definition and lookup
//! - Variable scoping
//! - Duplicate detection
//! - Scope shadowing

use super::*;
use crate::ast::Visibility;
use crate::semantic::error::SemanticErrorKind;
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
        visibility: Visibility::Private,
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
        visibility: Visibility::Private,
    };
    let info2 = FunctionInfo {
        name: "dup".to_string(),
        return_type: "void".to_string(),
        return_type_span: dummy_span(),
        definition_span: span_at(5, 1),
        visibility: Visibility::Private,
    };

    assert!(table.define_function(info1).is_ok());
    let result = table.define_function(info2);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::DuplicateFunction);
    assert_eq!(err.message(), "Function 'dup' is already defined at 1:1");
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
    let err = result.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::InternalError);
    assert_eq!(
        err.message(),
        "Internal error: attempted to define variable 'orphan' outside a scope. This is a compiler bug."
    );
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
    let err = result.unwrap_err();
    assert_eq!(err.kind(), SemanticErrorKind::DuplicateVariable);
    assert_eq!(err.message(), "Variable 'x' is already defined at 1:1");
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
