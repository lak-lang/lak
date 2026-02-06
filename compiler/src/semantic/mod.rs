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

use crate::ast::{
    BinaryOperator, Expr, ExprKind, FnDef, Program, Stmt, StmtKind, Type, UnaryOperator,
};
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
            return Err(SemanticError::invalid_main_signature(
                &main_fn.return_type,
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
            ExprKind::StringLiteral(_) => {
                Err(SemanticError::invalid_expression_string_literal(expr.span))
            }
            ExprKind::IntLiteral(_) => {
                Err(SemanticError::invalid_expression_int_literal(expr.span))
            }
            ExprKind::Identifier(name) => Err(SemanticError::invalid_expression_identifier(
                name, expr.span,
            )),
            ExprKind::BinaryOp { .. } => {
                // Binary operations as statements have no effect
                Err(SemanticError::invalid_expression_binary_op(expr.span))
            }
            ExprKind::UnaryOp { .. } => {
                // Unary operations as statements have no effect
                Err(SemanticError::invalid_expression_unary_op(expr.span))
            }
        }
    }

    fn analyze_call(&self, callee: &str, args: &[Expr], span: Span) -> Result<(), SemanticError> {
        // Built-in function: println
        // println accepts any type (string, i32, i64)
        if callee == "println" {
            if args.len() != 1 {
                return Err(SemanticError::invalid_argument_println_count(span));
            }

            // println accepts string literals, integer literals, or any variable
            match &args[0].kind {
                ExprKind::StringLiteral(_) => {}
                ExprKind::IntLiteral(_) => {}
                ExprKind::Identifier(name) => {
                    // Verify the variable exists (type doesn't matter, any type is accepted)
                    self.symbols
                        .lookup_variable(name)
                        .ok_or_else(|| SemanticError::undefined_variable(name, args[0].span))?;
                }
                ExprKind::Call {
                    callee: inner_callee,
                    ..
                } => {
                    return Err(SemanticError::invalid_argument_call_not_supported(
                        inner_callee,
                        args[0].span,
                    ));
                }
                ExprKind::BinaryOp { left, right, .. } => {
                    // For binary operations, we need to infer the type from the operands
                    // and validate that all variables exist
                    self.validate_expr_for_println(left)?;
                    self.validate_expr_for_println(right)?;
                }
                ExprKind::UnaryOp { .. } => {
                    // For unary operations, validate that all variables exist
                    // and that no unary operator is applied to a string type
                    self.validate_expr_for_println(&args[0])?;
                }
            }

            return Ok(());
        }

        // Built-in function: panic
        if callee == "panic" {
            if args.len() != 1 {
                return Err(SemanticError::invalid_argument_panic_count(span));
            }

            // panic only accepts string arguments
            match &args[0].kind {
                ExprKind::StringLiteral(_) => {}
                ExprKind::Identifier(name) => {
                    // Verify the variable exists and is a string
                    let var_info = self
                        .symbols
                        .lookup_variable(name)
                        .ok_or_else(|| SemanticError::undefined_variable(name, args[0].span))?;

                    if var_info.ty != Type::String {
                        return Err(SemanticError::invalid_argument_panic_type(
                            &var_info.ty.to_string(),
                            args[0].span,
                        ));
                    }
                }
                ExprKind::IntLiteral(_) => {
                    return Err(SemanticError::invalid_argument_panic_type(
                        "integer literal",
                        args[0].span,
                    ));
                }
                ExprKind::Call {
                    callee: inner_callee,
                    ..
                } => {
                    return Err(SemanticError::invalid_argument_call_not_supported(
                        inner_callee,
                        args[0].span,
                    ));
                }
                ExprKind::BinaryOp { .. } => {
                    return Err(SemanticError::invalid_argument_panic_type(
                        "expression",
                        args[0].span,
                    ));
                }
                ExprKind::UnaryOp { .. } => {
                    return Err(SemanticError::invalid_argument_panic_type(
                        "expression",
                        args[0].span,
                    ));
                }
            }

            return Ok(());
        }

        // Check if function is defined
        let func_info = self
            .symbols
            .lookup_function(callee)
            .ok_or_else(|| SemanticError::undefined_function(callee, span))?;

        // Disallow calling main function directly
        if callee == "main" {
            return Err(SemanticError::invalid_argument_cannot_call_main(span));
        }

        // Check argument count (currently only parameterless functions are supported)
        if !args.is_empty() {
            return Err(SemanticError::invalid_argument_fn_expects_no_args(
                callee,
                args.len(),
                span,
            ));
        }

        // Check that the function returns void (only void functions can be called as statements)
        if func_info.return_type != "void" {
            return Err(SemanticError::type_mismatch_non_void_fn_as_stmt(
                callee,
                &func_info.return_type,
                span,
            ));
        }

        Ok(())
    }

    fn check_expr_type(&self, expr: &Expr, expected_ty: &Type) -> Result<(), SemanticError> {
        match &expr.kind {
            ExprKind::IntLiteral(value) => {
                if *expected_ty == Type::String {
                    return Err(SemanticError::type_mismatch_int_to_string(
                        *value, expr.span,
                    ));
                }
                self.check_integer_range(*value, expected_ty, expr.span)
            }
            ExprKind::Identifier(name) => {
                let var_info = self
                    .symbols
                    .lookup_variable(name)
                    .ok_or_else(|| SemanticError::undefined_variable(name, expr.span))?;

                if var_info.ty != *expected_ty {
                    return Err(SemanticError::type_mismatch_variable(
                        name,
                        &var_info.ty.to_string(),
                        &expected_ty.to_string(),
                        expr.span,
                    ));
                }

                Ok(())
            }
            ExprKind::StringLiteral(_) => {
                if *expected_ty != Type::String {
                    return Err(SemanticError::type_mismatch_string_to_type(
                        &expected_ty.to_string(),
                        expr.span,
                    ));
                }
                Ok(())
            }
            ExprKind::Call { callee, .. } => Err(SemanticError::type_mismatch_call_as_value(
                callee, expr.span,
            )),
            ExprKind::BinaryOp { left, op, right } => {
                self.check_binary_op_type(left, *op, right, expected_ty, expr.span)
            }
            ExprKind::UnaryOp { op, operand } => {
                self.check_unary_op_type(operand, *op, expected_ty, expr.span)
            }
        }
    }

    /// Checks the types of a binary operation.
    ///
    /// Binary operations require:
    /// 1. Both operands to have the expected type
    /// 2. The expected type to be numeric (i32 or i64)
    fn check_binary_op_type(
        &self,
        left: &Expr,
        op: BinaryOperator,
        right: &Expr,
        expected_ty: &Type,
        span: Span,
    ) -> Result<(), SemanticError> {
        // Verify the expected type is numeric (not string)
        if *expected_ty == Type::String {
            return Err(SemanticError::invalid_binary_op_type(op, "string", span));
        }

        // Check both operands have the expected type
        self.check_expr_type(left, expected_ty)?;
        self.check_expr_type(right, expected_ty)?;

        Ok(())
    }

    /// Checks the types of a unary operation.
    ///
    /// Unary operations require:
    /// 1. The operand to have the expected type
    /// 2. The expected type to be numeric (i32 or i64)
    fn check_unary_op_type(
        &self,
        operand: &Expr,
        op: UnaryOperator,
        expected_ty: &Type,
        span: Span,
    ) -> Result<(), SemanticError> {
        // Verify the expected type is numeric (not string)
        if *expected_ty == Type::String {
            return Err(SemanticError::invalid_unary_op_type(op, "string", span));
        }

        // Check the operand has the expected type, adding unary context to errors
        self.check_expr_type(operand, expected_ty).map_err(|e| {
            // Add unary operation context if not already present
            if e.message().contains("unary") || e.message().contains("Unary") {
                e
            } else {
                SemanticError::new_with_help(
                    e.kind(),
                    format!("in unary '{}' operation: {}", op, e.message()),
                    span,
                    e.help().unwrap_or(""),
                )
            }
        })?;

        Ok(())
    }

    /// Validates an expression for use in println.
    ///
    /// This recursively validates that:
    /// 1. All variables referenced in the expression exist
    /// 2. No unary operator is applied to a string type
    fn validate_expr_for_println(&self, expr: &Expr) -> Result<(), SemanticError> {
        match &expr.kind {
            ExprKind::IntLiteral(_) | ExprKind::StringLiteral(_) => Ok(()),
            ExprKind::Identifier(name) => {
                self.symbols
                    .lookup_variable(name)
                    .ok_or_else(|| SemanticError::undefined_variable(name, expr.span))?;
                Ok(())
            }
            ExprKind::BinaryOp { left, right, .. } => {
                self.validate_expr_for_println(left)?;
                self.validate_expr_for_println(right)?;
                Ok(())
            }
            ExprKind::UnaryOp { op, operand } => {
                // First validate the operand recursively
                self.validate_expr_for_println(operand)?;

                // Then check that the immediate operand is not a string type
                match &operand.kind {
                    ExprKind::StringLiteral(_) => {
                        return Err(SemanticError::invalid_unary_op_type(
                            *op, "string", expr.span,
                        ));
                    }
                    ExprKind::Identifier(name) => {
                        // Variable was already validated to exist in the recursive call
                        if let Some(var_info) = self.symbols.lookup_variable(name)
                            && var_info.ty == Type::String
                        {
                            return Err(SemanticError::invalid_unary_op_type(
                                *op, "string", expr.span,
                            ));
                        }
                    }
                    _ => {}
                }
                Ok(())
            }
            ExprKind::Call { callee, .. } => Err(
                SemanticError::invalid_argument_call_not_supported(callee, expr.span),
            ),
        }
    }

    fn check_integer_range(&self, value: i64, ty: &Type, span: Span) -> Result<(), SemanticError> {
        match ty {
            Type::I32 => {
                if value < i32::MIN as i64 || value > i32::MAX as i64 {
                    return Err(SemanticError::integer_overflow_i32(value, span));
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
                return Err(SemanticError::internal_check_integer_range_string(
                    value, span,
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
