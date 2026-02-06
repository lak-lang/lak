//! Expression code generation.
//!
//! This module implements code generation for Lak expressions, including
//! function calls, literals, and variable references.

use super::Codegen;
use super::error::CodegenError;
use crate::ast::{BinaryOperator, Expr, ExprKind, Type};
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
            ExprKind::StringLiteral(_)
            | ExprKind::IntLiteral(_)
            | ExprKind::Identifier(_)
            | ExprKind::BinaryOp { .. } => {
                return Err(CodegenError::internal_invalid_expr_stmt(expr.span));
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
        let function = self
            .module
            .get_function(callee)
            .ok_or_else(|| CodegenError::internal_function_not_found(callee, span))?;

        self.builder
            .build_call(function, &[], "")
            .map_err(|e| CodegenError::internal_call_failed(callee, &e.to_string(), span))?;

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
                // Semantic analysis guarantees the value fits in the expected type.
                // Use match to handle integer types explicitly and reject string type.
                match expected_ty {
                    Type::I32 | Type::I64 => {
                        let llvm_type = self.get_llvm_type(expected_ty).into_int_type();
                        // Cast to u64 for the LLVM API. For const_int, LLVM takes the bottom N bits
                        // where N is the type's bit width. The sign_extend parameter controls whether
                        // those bits are sign-extended when the value is loaded to a wider register.
                        let llvm_value = llvm_type.const_int(*value as u64, true);
                        Ok(llvm_value.into())
                    }
                    Type::String => Err(CodegenError::internal_int_as_string(*value, expr.span)),
                }
            }
            ExprKind::Identifier(name) => {
                // Semantic analysis guarantees the variable exists and has the correct type
                let binding = self
                    .variables
                    .get(name)
                    .ok_or_else(|| CodegenError::internal_variable_not_found(name, expr.span))?;

                if *binding.ty() != *expected_ty {
                    return Err(CodegenError::internal_variable_type_mismatch(
                        name,
                        &expected_ty.to_string(),
                        &binding.ty().to_string(),
                        expr.span,
                    ));
                }

                let llvm_type = self.get_llvm_type(binding.ty());
                let loaded = self
                    .builder
                    .build_load(llvm_type, binding.alloca(), &format!("{}_load", name))
                    .map_err(|e| {
                        CodegenError::internal_variable_load_failed(name, &e.to_string(), expr.span)
                    })?;

                Ok(loaded)
            }
            ExprKind::StringLiteral(s) => {
                if *expected_ty != Type::String {
                    return Err(CodegenError::internal_string_as_type(
                        &expected_ty.to_string(),
                        expr.span,
                    ));
                }
                // Create a global constant string in read-only memory.
                // The pointer remains valid for the program's lifetime.
                let str_ptr = self
                    .builder
                    .build_global_string_ptr(s, "str")
                    .map_err(|e| {
                        CodegenError::internal_string_ptr_failed(&e.to_string(), expr.span)
                    })?;
                Ok(str_ptr.as_pointer_value().into())
            }
            ExprKind::Call { callee, .. } => {
                Err(CodegenError::internal_call_as_value(callee, expr.span))
            }
            ExprKind::BinaryOp { left, op, right } => {
                self.generate_binary_op(left, *op, right, expected_ty, expr.span)
            }
        }
    }

    /// Generates LLVM IR for a binary operation.
    ///
    /// # Arguments
    ///
    /// * `left` - The left operand expression
    /// * `op` - The binary operator
    /// * `right` - The right operand expression
    /// * `expected_ty` - The expected type of the result
    /// * `span` - The source span for error reporting
    ///
    /// # Errors
    ///
    /// Returns an internal error if:
    /// - The expected type is string (semantic analysis should catch this)
    /// - LLVM instruction building fails
    fn generate_binary_op(
        &self,
        left: &Expr,
        op: BinaryOperator,
        right: &Expr,
        expected_ty: &Type,
        span: crate::token::Span,
    ) -> Result<BasicValueEnum<'ctx>, CodegenError> {
        // Semantic analysis guarantees the type is numeric
        if *expected_ty == Type::String {
            return Err(CodegenError::internal_binary_op_string(op, span));
        }

        // Generate values for both operands
        let left_value = self
            .generate_expr_value(left, expected_ty)?
            .into_int_value();
        let right_value = self
            .generate_expr_value(right, expected_ty)?
            .into_int_value();

        // Generate the appropriate LLVM instruction based on the operator
        let result = match op {
            BinaryOperator::Add => self
                .builder
                .build_int_add(left_value, right_value, "add_tmp")
                .map_err(|e| CodegenError::internal_binary_op_failed(op, &e.to_string(), span))?,
            BinaryOperator::Sub => self
                .builder
                .build_int_sub(left_value, right_value, "sub_tmp")
                .map_err(|e| CodegenError::internal_binary_op_failed(op, &e.to_string(), span))?,
            BinaryOperator::Mul => self
                .builder
                .build_int_mul(left_value, right_value, "mul_tmp")
                .map_err(|e| CodegenError::internal_binary_op_failed(op, &e.to_string(), span))?,
            BinaryOperator::Div => self
                .builder
                .build_int_signed_div(left_value, right_value, "div_tmp")
                .map_err(|e| CodegenError::internal_binary_op_failed(op, &e.to_string(), span))?,
            BinaryOperator::Mod => self
                .builder
                .build_int_signed_rem(left_value, right_value, "mod_tmp")
                .map_err(|e| CodegenError::internal_binary_op_failed(op, &e.to_string(), span))?,
        };

        Ok(result.into())
    }
}
