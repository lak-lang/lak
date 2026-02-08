//! Expression code generation.
//!
//! This module implements code generation for Lak expressions, including
//! function calls, literals, and variable references.

use super::Codegen;
use super::builtins::BUILTIN_NAMES;
use super::error::CodegenError;
use super::mangle_name;
use crate::ast::{BinaryOperator, Expr, ExprKind, Type, UnaryOperator};
use inkwell::IntPredicate;
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum};

impl<'ctx> Codegen<'ctx> {
    /// Generates LLVM IR for an expression used as a statement.
    ///
    /// # Behavior
    ///
    /// For function calls: generates the call IR and returns `Ok(())`
    ///
    /// # Errors
    ///
    /// Returns an internal error for non-call expressions (literals, identifiers,
    /// binary/unary ops, member access). Both regular function calls and
    /// module-qualified function calls are valid.
    /// Semantic analysis guarantees that only valid expression statements reach codegen.
    pub(super) fn generate_expr(&mut self, expr: &Expr) -> Result<(), CodegenError> {
        match &expr.kind {
            ExprKind::Call { callee, args } => {
                if callee == "println" {
                    self.generate_println(args, expr.span)?;
                } else if callee == "panic" {
                    self.generate_panic(args, expr.span)?;
                } else {
                    self.generate_user_function_call(callee, expr.span)?;
                }
            }
            ExprKind::ModuleCall {
                module,
                function,
                args,
            } => {
                if !args.is_empty() {
                    return Err(CodegenError::internal_module_call_with_args(
                        module,
                        function,
                        args.len(),
                        expr.span,
                    ));
                }
                self.generate_module_call(module, function, expr.span)?;
            }
            ExprKind::StringLiteral(_)
            | ExprKind::IntLiteral(_)
            | ExprKind::BoolLiteral(_)
            | ExprKind::Identifier(_)
            | ExprKind::BinaryOp { .. }
            | ExprKind::UnaryOp { .. }
            | ExprKind::MemberAccess { .. } => {
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
        // When generating code for an imported module, function calls within
        // that module need to use mangled names (e.g., "_L5_utils_helper" instead
        // of "helper"), since all imported module functions are declared with
        // mangled names in the LLVM module.
        let function = if let Some(ref prefix) = self.current_module_prefix {
            let mangled = mangle_name(prefix, callee);
            self.module.get_function(&mangled).or_else(|| {
                // Only allow fallback to unmangled name for known builtins.
                // Builtins are declared with their original names (not mangled).
                if BUILTIN_NAMES.contains(&callee) {
                    self.module.get_function(callee)
                } else {
                    None
                }
            })
        } else {
            self.module.get_function(callee)
        };

        let function =
            function.ok_or_else(|| CodegenError::internal_function_not_found(callee, span))?;

        self.builder
            .build_call(function, &[], "")
            .map_err(|e| CodegenError::internal_call_failed(callee, &e.to_string(), span))?;

        Ok(())
    }

    /// Generates a call to a module-qualified function.
    ///
    /// The function name is mangled using the length-prefix scheme via `mangle_name()`.
    /// If the module name is an alias, it is resolved to the real module name first.
    ///
    /// # Arguments
    ///
    /// * `module_alias` - The module name or alias used in the source code
    /// * `function` - The function name
    /// * `span` - The source span of the call expression
    fn generate_module_call(
        &mut self,
        module_alias: &str,
        function: &str,
        span: crate::token::Span,
    ) -> Result<(), CodegenError> {
        // Resolve alias to real module name for correct name mangling
        let real_module = self.resolve_module_alias(module_alias, span)?;
        let mangled_name = mangle_name(&real_module, function);

        let llvm_function = self
            .module
            .get_function(&mangled_name)
            .ok_or_else(|| CodegenError::internal_function_not_found(&mangled_name, span))?;

        self.builder
            .build_call(llvm_function, &[], "")
            .map_err(|e| CodegenError::internal_call_failed(&mangled_name, &e.to_string(), span))?;

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
        &mut self,
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
                    Type::Bool => Err(CodegenError::internal_int_as_bool(*value, expr.span)),
                }
            }
            ExprKind::BoolLiteral(value) => {
                // Semantic analysis guarantees the expected type is Bool.
                if *expected_ty != Type::Bool {
                    return Err(CodegenError::internal_bool_as_type(
                        &expected_ty.to_string(),
                        expr.span,
                    ));
                }
                let llvm_value = self.context.bool_type().const_int(*value as u64, false);
                Ok(llvm_value.into())
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
            ExprKind::UnaryOp { op, operand } => {
                self.generate_unary_op(operand, *op, expected_ty, expr.span)
            }
            ExprKind::MemberAccess { .. } => {
                // Module-qualified expressions are not yet supported
                Err(CodegenError::internal_member_access_not_implemented(
                    expr.span,
                ))
            }
            ExprKind::ModuleCall {
                module,
                function,
                args: _,
            } => Err(CodegenError::internal_module_call_as_value(
                module, function, expr.span,
            )),
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
        &mut self,
        left: &Expr,
        op: BinaryOperator,
        right: &Expr,
        expected_ty: &Type,
        span: crate::token::Span,
    ) -> Result<BasicValueEnum<'ctx>, CodegenError> {
        // Semantic analysis guarantees the type is numeric (not string or bool)
        if *expected_ty == Type::String || *expected_ty == Type::Bool {
            return Err(CodegenError::internal_binary_op_string(op, span));
        }

        // Generate values for both operands
        let left_basic = self.generate_expr_value(left, expected_ty)?;
        let left_value = match left_basic {
            BasicValueEnum::IntValue(v) => v,
            _ => return Err(CodegenError::internal_non_integer_value("binary", span)),
        };
        let right_basic = self.generate_expr_value(right, expected_ty)?;
        let right_value = match right_basic {
            BasicValueEnum::IntValue(v) => v,
            _ => return Err(CodegenError::internal_non_integer_value("binary", span)),
        };

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
            BinaryOperator::Div => {
                self.generate_division_zero_check(right_value, "division by zero", span)?;
                self.builder
                    .build_int_signed_div(left_value, right_value, "div_tmp")
                    .map_err(|e| {
                        CodegenError::internal_binary_op_failed(op, &e.to_string(), span)
                    })?
            }
            BinaryOperator::Mod => {
                self.generate_division_zero_check(right_value, "modulo by zero", span)?;
                self.builder
                    .build_int_signed_rem(left_value, right_value, "mod_tmp")
                    .map_err(|e| {
                        CodegenError::internal_binary_op_failed(op, &e.to_string(), span)
                    })?
            }
        };

        Ok(result.into())
    }

    /// Generates LLVM IR for a unary operation.
    ///
    /// # Arguments
    ///
    /// * `operand` - The operand expression
    /// * `op` - The unary operator
    /// * `expected_ty` - The expected type of the result
    /// * `span` - The source span for error reporting
    ///
    /// # Errors
    ///
    /// Returns an internal error if:
    /// - The expected type is string (semantic analysis should catch this)
    /// - LLVM instruction building fails
    fn generate_unary_op(
        &mut self,
        operand: &Expr,
        op: UnaryOperator,
        expected_ty: &Type,
        span: crate::token::Span,
    ) -> Result<BasicValueEnum<'ctx>, CodegenError> {
        // Semantic analysis guarantees the type is numeric (not string or bool)
        if *expected_ty == Type::String || *expected_ty == Type::Bool {
            return Err(CodegenError::internal_unary_op_string(op, span));
        }

        // Generate value for the operand, adding context to any errors
        let operand_basic = self
            .generate_expr_value(operand, expected_ty)
            .map_err(|e| CodegenError::wrap_in_unary_context(&e, op, span))?;
        let operand_value = match operand_basic {
            BasicValueEnum::IntValue(v) => v,
            _ => return Err(CodegenError::internal_non_integer_value("unary", span)),
        };

        // Generate the appropriate LLVM instruction based on the operator
        let result = match op {
            UnaryOperator::Neg => self
                .builder
                .build_int_neg(operand_value, "neg_tmp")
                .map_err(|e| CodegenError::internal_unary_op_failed(op, &e.to_string(), span))?,
        };

        Ok(result.into())
    }

    /// Generates a runtime check for division/modulo by zero.
    ///
    /// This creates a conditional branch that panics if the divisor is zero,
    /// otherwise continues to a safe block where the division/modulo occurs.
    ///
    /// # LLVM IR Pattern
    ///
    /// This check is inserted inline at the current insertion point:
    ///
    /// ```text
    ///   %is_zero = icmp eq <type> %divisor, 0
    ///   br i1 %is_zero, label %div_zero_panic, label %div_zero_safe
    ///
    /// div_zero_panic:
    ///   call void @lak_panic("division by zero")
    ///   unreachable
    ///
    /// div_zero_safe:
    ///   %result = sdiv <type> %left, %divisor
    ///   ; Execution continues here after the division
    /// ```
    ///
    /// # Arguments
    ///
    /// * `divisor` - The divisor value to check
    /// * `error_message` - The panic message (e.g., "division by zero")
    /// * `span` - The source span for error reporting
    fn generate_division_zero_check(
        &mut self,
        divisor: inkwell::values::IntValue<'ctx>,
        error_message: &str,
        span: crate::token::Span,
    ) -> Result<(), CodegenError> {
        // Get the current function
        let current_fn = self
            .builder
            .get_insert_block()
            .and_then(|bb| bb.get_parent())
            .ok_or_else(|| CodegenError::internal_no_current_function(span))?;

        // Create basic blocks
        let panic_block = self
            .context
            .append_basic_block(current_fn, "div_zero_panic");
        let safe_block = self.context.append_basic_block(current_fn, "div_zero_safe");

        // Create zero constant of the same type as divisor
        let zero = divisor.get_type().const_int(0, false);

        // Compare divisor with zero
        let is_zero = self
            .builder
            .build_int_compare(IntPredicate::EQ, divisor, zero, "is_zero")
            .map_err(|e| CodegenError::internal_compare_failed(&e.to_string(), span))?;

        // Branch: if zero goto panic_block, else goto safe_block
        self.builder
            .build_conditional_branch(is_zero, panic_block, safe_block)
            .map_err(|e| CodegenError::internal_branch_failed(&e.to_string(), span))?;

        // Build panic block
        self.builder.position_at_end(panic_block);

        // Create global string for error message
        let panic_msg = self
            .builder
            .build_global_string_ptr(error_message, "div_zero_msg")
            .map_err(|e| CodegenError::internal_string_ptr_failed(&e.to_string(), span))?
            .as_pointer_value();

        // Call lak_panic
        let lak_panic = self
            .module
            .get_function("lak_panic")
            .ok_or_else(|| CodegenError::internal_builtin_not_found("lak_panic"))?;

        self.builder
            .build_call(
                lak_panic,
                &[BasicMetadataValueEnum::PointerValue(panic_msg)],
                "",
            )
            .map_err(|e| CodegenError::internal_panic_call_failed(&e.to_string(), span))?;

        // Insert unreachable instruction
        self.builder
            .build_unreachable()
            .map_err(|e| CodegenError::internal_unreachable_failed(&e.to_string(), span))?;

        // Position builder at safe block for the actual division/modulo
        self.builder.position_at_end(safe_block);

        Ok(())
    }
}
