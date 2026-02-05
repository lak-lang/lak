//! Expression code generation.
//!
//! This module implements code generation for Lak expressions, including
//! function calls, literals, and variable references.

use super::Codegen;
use super::error::{CodegenError, CodegenErrorKind};
use crate::ast::{Expr, ExprKind, Type};
use inkwell::values::BasicValueEnum;

impl<'ctx> Codegen<'ctx> {
    /// Generates LLVM IR for an expression used as a statement.
    ///
    /// # Behavior
    ///
    /// For function calls: generates the call IR and returns `Ok(())`
    ///
    /// # Panics
    ///
    /// Panics for non-call expressions (literals, identifiers). Semantic analysis
    /// guarantees that only valid expression statements reach codegen.
    pub(super) fn generate_expr(&mut self, expr: &Expr) -> Result<(), CodegenError> {
        match &expr.kind {
            ExprKind::Call { callee, args } => {
                // Semantic analysis guarantees only valid function calls
                debug_assert!(
                    callee == "println",
                    "Unknown function '{}' should have been caught by semantic analysis",
                    callee
                );
                self.generate_println(args, expr.span)?;
            }
            ExprKind::StringLiteral(_) | ExprKind::IntLiteral(_) | ExprKind::Identifier(_) => {
                // Semantic analysis should reject these as statements
                panic!("Invalid expression statement should have been caught by semantic analysis");
            }
        }
        Ok(())
    }

    /// Generates LLVM IR for an expression and returns its value.
    ///
    /// This method is used when an expression's value is needed (e.g., as
    /// an initializer for a variable).
    ///
    /// # Arguments
    ///
    /// * `expr` - The expression to evaluate
    /// * `expected_ty` - The expected type of the result (validated by semantic analysis)
    ///
    /// # Errors
    ///
    /// Returns an error if LLVM operations fail (internal errors).
    ///
    /// # Panics
    ///
    /// Panics if semantic validation was bypassed (undefined variable, type mismatch,
    /// string literal as integer, function call as value). Semantic analysis guarantees
    /// these conditions cannot occur.
    pub(super) fn generate_expr_value(
        &self,
        expr: &Expr,
        expected_ty: &Type,
    ) -> Result<BasicValueEnum<'ctx>, CodegenError> {
        match &expr.kind {
            ExprKind::IntLiteral(value) => {
                // Semantic analysis guarantees the value fits in the expected type
                let llvm_type = self.get_llvm_type(expected_ty);
                // Cast to u64 for the LLVM API. For const_int, LLVM takes the bottom N bits
                // where N is the type's bit width. The sign_extend parameter controls whether
                // those bits are sign-extended when the value is loaded to a wider register.
                let llvm_value = llvm_type.const_int(*value as u64, true);
                Ok(llvm_value.into())
            }
            ExprKind::Identifier(name) => {
                // Semantic analysis guarantees the variable exists and has the correct type
                let binding = self.variables.get(name).ok_or_else(|| {
                    CodegenError::new(
                        CodegenErrorKind::InternalError,
                        format!(
                            "Internal error: undefined variable '{}' in codegen. \
                             Semantic analysis should have caught this. This is a compiler bug.",
                            name
                        ),
                        expr.span,
                    )
                })?;

                debug_assert!(
                    *binding.ty() == *expected_ty,
                    "Type mismatch should have been caught by semantic analysis"
                );

                let llvm_type = self.get_llvm_type(binding.ty());
                let loaded = self
                    .builder
                    .build_load(llvm_type, binding.alloca(), &format!("{}_load", name))
                    .map_err(|e| {
                        CodegenError::new(
                            CodegenErrorKind::InternalError,
                            format!(
                                "Internal error: failed to load variable '{}'. This is a compiler bug: {}",
                                name, e
                            ),
                            expr.span,
                        )
                    })?;

                Ok(loaded)
            }
            ExprKind::StringLiteral(_) => {
                panic!(
                    "String literal as integer value should have been caught by semantic analysis"
                );
            }
            ExprKind::Call { callee, .. } => {
                panic!(
                    "Function call '{}' as value should have been caught by semantic analysis",
                    callee
                );
            }
        }
    }
}
