//! Semantic analysis for the Lak programming language.
//!
//! This module provides the [`SemanticAnalyzer`] which validates a Lak AST
//! for semantic correctness before code generation.
//!
//! # Responsibilities
//!
//! The semantic analyzer performs the following validations:
//!
//! - **Name resolution**: Checks for duplicate/undefined functions and variables
//! - **Type checking**: Validates type consistency in assignments and expressions
//! - **Structural validation**: Ensures main function exists with correct signature
//!
//! # Pipeline Position
//!
//! ```text
//! Source → Lexer → Parser → Semantic Analyzer → Codegen → Executable
//! ```
//!
//! The semantic analyzer sits between the parser and code generator. It takes
//! an AST and either returns success (allowing codegen to proceed) or an error
//! describing the semantic problem.

mod error;
mod symbol;

#[cfg(test)]
mod tests;

pub use error::{SemanticError, SemanticErrorKind};
use symbol::{FunctionInfo, SymbolTable, VariableInfo};

use crate::ast::{Expr, ExprKind, FnDef, Program, Stmt, StmtKind, Type};
use crate::token::Span;

/// Semantic analyzer for Lak programs.
///
/// Performs semantic validation on an AST without modifying it:
/// - Name resolution (duplicate/undefined checks)
/// - Type checking
/// - Structural validation (main function)
///
/// The analyzer guarantees that if `analyze()` succeeds, the AST is
/// semantically valid and code generation can proceed without semantic errors.
pub struct SemanticAnalyzer {
    symbols: SymbolTable,
}

impl SemanticAnalyzer {
    /// Creates a new semantic analyzer.
    pub fn new() -> Self {
        SemanticAnalyzer {
            symbols: SymbolTable::new(),
        }
    }

    /// Analyzes a program for semantic correctness.
    ///
    /// Performs complete semantic validation in this order:
    /// 1. Collect all function definitions (check for duplicates)
    /// 2. Validate main function exists and has correct signature
    /// 3. Analyze each function body (variables, types, expressions)
    ///
    /// # Errors
    ///
    /// Returns an error if any semantic violation is found:
    /// - Duplicate function definitions
    /// - Missing main function
    /// - Invalid main signature
    /// - Duplicate variable definitions
    /// - Undefined variable/function references
    /// - Type mismatches
    /// - Integer overflow
    pub fn analyze(&mut self, program: &Program) -> Result<(), SemanticError> {
        // Phase 1: Collect function definitions
        self.collect_functions(program)?;

        // Phase 2: Validate main function
        self.validate_main_function(program)?;

        // Phase 3: Analyze function bodies
        for function in &program.functions {
            self.analyze_function(function)?;
        }

        Ok(())
    }

    // Phase 1: Function collection

    fn collect_functions(&mut self, program: &Program) -> Result<(), SemanticError> {
        for function in &program.functions {
            let info = FunctionInfo {
                name: function.name.clone(),
                return_type: function.return_type.clone(),
                return_type_span: function.return_type_span,
                definition_span: function.span,
            };

            self.symbols.define_function(info)?;
        }
        Ok(())
    }

    // Phase 2: Main function validation

    fn validate_main_function(&self, program: &Program) -> Result<(), SemanticError> {
        // Check main exists
        let main_fn = self.symbols.lookup_function("main").ok_or_else(|| {
            if program.functions.is_empty() {
                SemanticError::missing_main(
                    "No main function found: program contains no function definitions",
                )
            } else {
                let names: Vec<_> = program.functions.iter().map(|f| f.name.as_str()).collect();
                SemanticError::missing_main(format!(
                    "No main function found. Defined functions: {:?}",
                    names
                ))
            }
        })?;

        // Validate signature
        if main_fn.return_type != "void" {
            return Err(SemanticError::new(
                SemanticErrorKind::InvalidMainSignature,
                format!(
                    "main function must return void, but found return type '{}'",
                    main_fn.return_type
                ),
                main_fn.return_type_span,
            ));
        }

        Ok(())
    }

    // Phase 3: Function body analysis

    fn analyze_function(&mut self, function: &FnDef) -> Result<(), SemanticError> {
        self.symbols.enter_scope();

        for stmt in &function.body {
            self.analyze_stmt(stmt)?;
        }

        self.symbols.exit_scope();
        Ok(())
    }

    fn analyze_stmt(&mut self, stmt: &Stmt) -> Result<(), SemanticError> {
        match &stmt.kind {
            StmtKind::Expr(expr) => self.analyze_expr_stmt(expr),
            StmtKind::Let { name, ty, init } => self.analyze_let(name, ty, init, stmt.span),
        }
    }

    fn analyze_let(
        &mut self,
        name: &str,
        ty: &Type,
        init: &Expr,
        span: Span,
    ) -> Result<(), SemanticError> {
        // Check for duplicate variable
        let info = VariableInfo {
            name: name.to_string(),
            ty: ty.clone(),
            definition_span: span,
        };

        self.symbols.define_variable(info)?;

        // Type check initializer
        self.check_expr_type(init, ty)?;

        Ok(())
    }

    fn analyze_expr_stmt(&self, expr: &Expr) -> Result<(), SemanticError> {
        match &expr.kind {
            ExprKind::Call { callee, args } => self.analyze_call(callee, args, expr.span),
            ExprKind::StringLiteral(_) => Err(SemanticError::new(
                SemanticErrorKind::InvalidExpression,
                "String literal as a statement has no effect. Did you mean to pass it to a function?",
                expr.span,
            )),
            ExprKind::IntLiteral(_) => Err(SemanticError::new(
                SemanticErrorKind::InvalidExpression,
                "Integer literal as a statement has no effect. Did you mean to assign it to a variable?",
                expr.span,
            )),
            ExprKind::Identifier(name) => Err(SemanticError::new(
                SemanticErrorKind::InvalidExpression,
                format!(
                    "Variable '{}' used as a statement has no effect. Did you mean to use it in an expression?",
                    name
                ),
                expr.span,
            )),
        }
    }

    fn analyze_call(&self, callee: &str, args: &[Expr], span: Span) -> Result<(), SemanticError> {
        // Built-in function: println
        // println accepts any type (string, i32, i64)
        if callee == "println" {
            if args.len() != 1 {
                return Err(SemanticError::new(
                    SemanticErrorKind::InvalidArgument,
                    "println expects exactly 1 argument",
                    span,
                ));
            }

            // println accepts string literals, integer literals, or any variable
            match &args[0].kind {
                ExprKind::StringLiteral(_) => {}
                ExprKind::IntLiteral(_) => {}
                ExprKind::Identifier(name) => {
                    // Verify the variable exists (type doesn't matter, any type is accepted)
                    self.symbols.lookup_variable(name).ok_or_else(|| {
                        SemanticError::new(
                            SemanticErrorKind::UndefinedVariable,
                            format!("Undefined variable: '{}'", name),
                            args[0].span,
                        )
                    })?;
                }
                ExprKind::Call {
                    callee: inner_callee,
                    ..
                } => {
                    return Err(SemanticError::new(
                        SemanticErrorKind::InvalidArgument,
                        format!(
                            "Function call '{}' cannot be used as println argument (functions returning values not yet supported)",
                            inner_callee
                        ),
                        args[0].span,
                    ));
                }
            }

            return Ok(());
        }

        // Check if function is defined
        let func_info = self.symbols.lookup_function(callee).ok_or_else(|| {
            SemanticError::new(
                SemanticErrorKind::UndefinedFunction,
                format!("Undefined function: '{}'", callee),
                span,
            )
        })?;

        // Disallow calling main function directly
        if callee == "main" {
            return Err(SemanticError::new(
                SemanticErrorKind::InvalidArgument,
                "Cannot call 'main' function directly",
                span,
            ));
        }

        // Check argument count (currently only parameterless functions are supported)
        if !args.is_empty() {
            return Err(SemanticError::new(
                SemanticErrorKind::InvalidArgument,
                format!(
                    "Function '{}' expects 0 arguments, but got {}",
                    callee,
                    args.len()
                ),
                span,
            ));
        }

        // Check that the function returns void (only void functions can be called as statements)
        if func_info.return_type != "void" {
            return Err(SemanticError::new(
                SemanticErrorKind::TypeMismatch,
                format!(
                    "Function '{}' returns '{}', but only void functions can be called as statements",
                    callee, func_info.return_type
                ),
                span,
            ));
        }

        Ok(())
    }

    fn check_expr_type(&self, expr: &Expr, expected_ty: &Type) -> Result<(), SemanticError> {
        match &expr.kind {
            ExprKind::IntLiteral(value) => {
                if *expected_ty == Type::String {
                    return Err(SemanticError::new(
                        SemanticErrorKind::TypeMismatch,
                        format!(
                            "Type mismatch: integer literal '{}' cannot be assigned to type 'string'",
                            value
                        ),
                        expr.span,
                    ));
                }
                self.check_integer_range(*value, expected_ty, expr.span)
            }
            ExprKind::Identifier(name) => {
                let var_info = self.symbols.lookup_variable(name).ok_or_else(|| {
                    SemanticError::new(
                        SemanticErrorKind::UndefinedVariable,
                        format!("Undefined variable: '{}'", name),
                        expr.span,
                    )
                })?;

                if var_info.ty != *expected_ty {
                    return Err(SemanticError::new(
                        SemanticErrorKind::TypeMismatch,
                        format!(
                            "Type mismatch: variable '{}' has type '{}', expected '{}'",
                            name, var_info.ty, expected_ty
                        ),
                        expr.span,
                    ));
                }

                Ok(())
            }
            ExprKind::StringLiteral(_) => {
                if *expected_ty != Type::String {
                    return Err(SemanticError::new(
                        SemanticErrorKind::TypeMismatch,
                        format!(
                            "Type mismatch: string literal cannot be assigned to type '{}'",
                            expected_ty
                        ),
                        expr.span,
                    ));
                }
                Ok(())
            }
            ExprKind::Call { callee, .. } => Err(SemanticError::new(
                SemanticErrorKind::TypeMismatch,
                format!(
                    "Function call '{}' cannot be used as a value (functions returning values not yet supported)",
                    callee
                ),
                expr.span,
            )),
        }
    }

    fn check_integer_range(&self, value: i64, ty: &Type, span: Span) -> Result<(), SemanticError> {
        match ty {
            Type::I32 => {
                if value < i32::MIN as i64 || value > i32::MAX as i64 {
                    return Err(SemanticError::new(
                        SemanticErrorKind::IntegerOverflow,
                        format!(
                            "Integer literal '{}' is out of range for i32 (valid range: {} to {})",
                            value,
                            i32::MIN,
                            i32::MAX
                        ),
                        span,
                    ));
                }
            }
            Type::I64 => {
                // Invariant: The lexer parses integer literals into i64, so any
                // value that made it past lexing is guaranteed to be within i64 range.
            }
            Type::String => {
                // This branch should never be reached because check_expr_type
                // handles Type::String before calling check_integer_range.
                // Return an internal error to signal a compiler bug if this is reached.
                return Err(SemanticError::new(
                    SemanticErrorKind::InternalError,
                    format!(
                        "Internal error: check_integer_range called with string type \
                         for value '{}'. This is a compiler bug.",
                        value
                    ),
                    span,
                ));
            }
        }
        Ok(())
    }
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
