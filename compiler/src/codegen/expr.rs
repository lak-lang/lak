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
    /// # Errors
    ///
    /// Returns an internal error for non-call expressions (literals, identifiers).
    /// Semantic analysis guarantees that only valid expression statements reach codegen.
    pub(super) fn generate_expr(&mut self, expr: &Expr) -> Result<(), CodegenError> {
        match &expr.kind {
            ExprKind::Call { callee, args } => {
                if callee == "println" {
                    self.generate_println(args, expr.span)?;
                } else {
                    self.generate_user_function_call(callee, expr.span)?;
                }
            }
            ExprKind::StringLiteral(_) | ExprKind::IntLiteral(_) | ExprKind::Identifier(_) => {
                return Err(CodegenError::new(
                    CodegenErrorKind::InternalError,
                    "Internal error: invalid expression statement in codegen. \
                     Semantic analysis should have rejected this. This is a compiler bug.",
                    expr.span,
                ));
            }
        }
        Ok(())
    }

    /// Generates a call to a user-defined function.
    ///
    /// # Arguments
    ///
    /// * `callee` - The name of the function to call
    /// * `span` - The source span of the call expression
    ///
    /// # Errors
    ///
    /// Returns an error if the function is not found (internal error, should be
    /// caught by semantic analysis).
    fn generate_user_function_call(
        &mut self,
        callee: &str,
        span: crate::token::Span,
    ) -> Result<(), CodegenError> {
        let function = self.module.get_function(callee).ok_or_else(|| {
            CodegenError::new(
                CodegenErrorKind::InternalError,
                format!(
                    "Internal error: undefined function '{}' in codegen. \
                     Semantic analysis should have caught this. This is a compiler bug.",
                    callee
                ),
                span,
            )
        })?;

        self.builder.build_call(function, &[], "").map_err(|e| {
            CodegenError::new(
                CodegenErrorKind::InternalError,
                format!(
                    "Internal error: failed to generate call to '{}'. This is a compiler bug: {}",
                    callee, e
                ),
                span,
            )
        })?;

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
    /// Returns an internal error if semantic validation was bypassed (undefined variable,
    /// type mismatch, string literal as integer, function call as value). Semantic analysis
    /// guarantees these conditions cannot occur.
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

                if *binding.ty() != *expected_ty {
                    return Err(CodegenError::new(
                        CodegenErrorKind::InternalError,
                        format!(
                            "Internal error: type mismatch for variable '{}' in codegen. \
                             Expected '{}', but variable has type '{}'. \
                             Semantic analysis should have caught this. This is a compiler bug.",
                            name,
                            expected_ty,
                            binding.ty()
                        ),
                        expr.span,
                    ));
                }

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
            ExprKind::StringLiteral(_) => Err(CodegenError::new(
                CodegenErrorKind::InternalError,
                "Internal error: string literal used as integer value in codegen. \
                 Semantic analysis should have caught this. This is a compiler bug.",
                expr.span,
            )),
            ExprKind::Call { callee, .. } => Err(CodegenError::new(
                CodegenErrorKind::InternalError,
                format!(
                    "Internal error: function call '{}' used as value in codegen. \
                     Semantic analysis should have caught this. This is a compiler bug.",
                    callee
                ),
                expr.span,
            )),
        }
    }
}
