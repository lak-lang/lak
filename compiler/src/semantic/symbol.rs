//! Symbol table for semantic analysis.
//!
//! This module provides [`SymbolTable`] for tracking function and variable
//! definitions during semantic analysis, with support for scoped variable lookup.

use super::error::SemanticError;
use crate::ast::Type;
use crate::token::Span;
use std::collections::HashMap;

/// Information about a defined function.
#[derive(Debug, Clone)]
pub struct FunctionInfo {
    /// The function name.
    pub name: String,
    /// The return type (currently only "void").
    pub return_type: String,
    /// The span of the return type token (for error reporting).
    pub return_type_span: Span,
    /// The span of the function definition (for "previously defined here" messages).
    pub definition_span: Span,
}

/// Information about a defined variable.
#[derive(Debug, Clone)]
pub struct VariableInfo {
    /// The variable name.
    pub name: String,
    /// The declared type.
    pub ty: Type,
    /// The span of the variable definition (for "previously defined here" messages).
    pub definition_span: Span,
}

/// A scope containing variable definitions.
#[derive(Debug, Clone)]
struct Scope {
    variables: HashMap<String, VariableInfo>,
}

impl Scope {
    fn new() -> Self {
        Scope {
            variables: HashMap::new(),
        }
    }
}

/// Symbol table for semantic analysis.
///
/// Manages function and variable definitions with scoping rules.
/// Currently supports:
/// - Global function definitions (flat namespace)
/// - Function-local variables with single scope
/// - Future: nested block scopes
pub struct SymbolTable {
    /// All function definitions (global namespace).
    functions: HashMap<String, FunctionInfo>,
    /// Stack of variable scopes (top = current scope).
    scopes: Vec<Scope>,
}

impl SymbolTable {
    /// Creates a new empty symbol table.
    pub fn new() -> Self {
        SymbolTable {
            functions: HashMap::new(),
            scopes: Vec::new(),
        }
    }

    // Function management

    /// Defines a new function. Returns error if already defined.
    pub fn define_function(&mut self, info: FunctionInfo) -> Result<(), SemanticError> {
        if let Some(existing) = self.functions.get(&info.name) {
            return Err(SemanticError::duplicate_function(
                &info.name,
                existing.definition_span.line,
                existing.definition_span.column,
                info.definition_span,
            ));
        }
        self.functions.insert(info.name.clone(), info);
        Ok(())
    }

    /// Looks up a function by name.
    pub fn lookup_function(&self, name: &str) -> Option<&FunctionInfo> {
        self.functions.get(name)
    }

    // Scope management

    /// Enters a new scope (e.g., function body).
    pub fn enter_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    /// Exits the current scope, discarding all variables in that scope.
    pub fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    // Variable management

    /// Defines a variable in the current scope. Returns error if already defined.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No scope is active (internal error - `enter_scope` was not called)
    /// - The variable is already defined in the current scope
    pub fn define_variable(&mut self, info: VariableInfo) -> Result<(), SemanticError> {
        let definition_span = info.definition_span;
        let current_scope = self
            .scopes
            .last_mut()
            .ok_or_else(|| SemanticError::internal_no_scope(&info.name, definition_span))?;

        if let Some(existing) = current_scope.variables.get(&info.name) {
            return Err(SemanticError::duplicate_variable(
                &info.name,
                existing.definition_span.line,
                existing.definition_span.column,
                info.definition_span,
            ));
        }
        current_scope.variables.insert(info.name.clone(), info);
        Ok(())
    }

    /// Looks up a variable in the current scope chain.
    /// Searches from innermost to outermost scope.
    pub fn lookup_variable(&self, name: &str) -> Option<&VariableInfo> {
        // Search from innermost (top of stack) to outermost
        for scope in self.scopes.iter().rev() {
            if let Some(info) = scope.variables.get(name) {
                return Some(info);
            }
        }
        None
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}
