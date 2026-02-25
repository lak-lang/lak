//! Semantic analysis error tests for the Lak compiler.
//!
//! These tests verify that semantic errors are properly detected
//! and reported during semantic analysis.

mod common;

#[path = "errors_semantic/bindings_and_types.rs"]
mod bindings_and_types;
#[path = "errors_semantic/calls_and_expressions.rs"]
mod calls_and_expressions;
#[path = "errors_semantic/comparisons_and_logical.rs"]
mod comparisons_and_logical;
#[path = "errors_semantic/helpers.rs"]
mod helpers;
#[path = "errors_semantic/module_access.rs"]
mod module_access;
#[path = "errors_semantic/panic_builtin.rs"]
mod panic_builtin;
#[path = "errors_semantic/returns_and_discard.rs"]
mod returns_and_discard;
#[path = "errors_semantic/unary_and_boolean.rs"]
mod unary_and_boolean;
