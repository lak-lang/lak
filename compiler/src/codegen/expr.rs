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
    /// - For function calls: generates the call IR and returns `Ok(())`
    /// - For literals and identifiers: returns an error since these have no
    ///   side effects when used as statements
    pub(super) fn generate_expr(&mut self, expr: &Expr) -> Result<(), CodegenError> {
        match &expr.kind {
            ExprKind::Call { callee, args } => {
                if callee == "println" {
                    self.generate_println(args, expr.span)?;
                } else {
                    return Err(CodegenError::new(
                        CodegenErrorKind::UndefinedFunction,
                        format!("Unknown function: {}", callee),
                        expr.span,
                    ));
                }
            }
            ExprKind::StringLiteral(_) => {
                return Err(CodegenError::new(
                    CodegenErrorKind::InvalidExpression,
                    "String literal as a statement has no effect. Did you mean to pass it to a function?",
                    expr.span,
                ));
            }
            ExprKind::IntLiteral(_) => {
                return Err(CodegenError::new(
                    CodegenErrorKind::InvalidExpression,
                    "Integer literal as a statement has no effect. Did you mean to assign it to a variable?",
                    expr.span,
                ));
            }
            ExprKind::Identifier(name) => {
                return Err(CodegenError::new(
                    CodegenErrorKind::InvalidExpression,
                    format!(
                        "Variable '{}' used as a statement has no effect. Did you mean to use it in an expression?",
                        name
                    ),
                    expr.span,
                ));
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
    /// * `expected_ty` - The expected type of the result
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - A referenced variable is undefined
    /// - A variable's type does not match the expected type
    /// - An integer literal is out of range for the expected type
    /// - A string literal is used (strings cannot be converted to integer types)
    /// - A function call is used (functions returning values not yet supported)
    pub(super) fn generate_expr_value(
        &self,
        expr: &Expr,
        expected_ty: &Type,
    ) -> Result<BasicValueEnum<'ctx>, CodegenError> {
        match &expr.kind {
            ExprKind::IntLiteral(value) => {
                // Validate that the value fits in the expected type
                match expected_ty {
                    Type::I32 => {
                        if *value < i32::MIN as i64 || *value > i32::MAX as i64 {
                            return Err(CodegenError::new(
                                CodegenErrorKind::IntegerOverflow,
                                format!(
                                    "Integer literal {} is out of range for i32 (valid range: {} to {})",
                                    value,
                                    i32::MIN,
                                    i32::MAX
                                ),
                                expr.span,
                            ));
                        }
                    }
                    Type::I64 => {
                        // Invariant: The lexer parses integer literals into i64, so any
                        // value that made it past lexing is guaranteed to be within i64 range.
                    }
                }

                let llvm_type = self.get_llvm_type(expected_ty);
                // Cast to u64 for the LLVM API. For const_int, LLVM takes the bottom N bits
                // where N is the type's bit width. The sign_extend parameter controls whether
                // those bits are sign-extended when the value is loaded to a wider register.
                let llvm_value = llvm_type.const_int(*value as u64, true);
                Ok(llvm_value.into())
            }
            ExprKind::Identifier(name) => {
                let binding = self.variables.get(name).ok_or_else(|| {
                    CodegenError::new(
                        CodegenErrorKind::UndefinedVariable,
                        format!("Undefined variable: '{}'", name),
                        expr.span,
                    )
                })?;

                // Type check
                if *binding.ty() != *expected_ty {
                    return Err(CodegenError::new(
                        CodegenErrorKind::TypeMismatch,
                        format!(
                            "Type mismatch: variable '{}' has type '{}', expected '{}'",
                            name,
                            binding.ty(),
                            expected_ty
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
                CodegenErrorKind::TypeMismatch,
                "String literals cannot be used as integer values",
                expr.span,
            )),
            ExprKind::Call { callee, .. } => Err(CodegenError::new(
                CodegenErrorKind::TypeMismatch,
                format!(
                    "Function call '{}' cannot be used as a value (functions returning values not yet supported)",
                    callee
                ),
                expr.span,
            )),
        }
    }
}
