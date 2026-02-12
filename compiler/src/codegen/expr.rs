//! Expression code generation.
//!
//! This module implements code generation for Lak expressions, including
//! function calls, literals, variable references, binary operations
//! (arithmetic and comparison), and unary operations.

use super::Codegen;
use super::builtins::BUILTIN_NAMES;
use super::error::CodegenError;
use super::mangle_name;
use crate::ast::{BinaryOperator, Expr, ExprKind, IfExprBlock, Type, UnaryOperator};
use inkwell::IntPredicate;
use inkwell::intrinsics::Intrinsic;
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
                    self.generate_user_function_call(callee, args, expr.span)?;
                }
            }
            ExprKind::ModuleCall {
                module,
                function,
                args,
            } => self.generate_module_call(module, function, args, expr.span)?,
            ExprKind::StringLiteral(_)
            | ExprKind::IntLiteral(_)
            | ExprKind::BoolLiteral(_)
            | ExprKind::Identifier(_)
            | ExprKind::BinaryOp { .. }
            | ExprKind::UnaryOp { .. }
            | ExprKind::IfExpr { .. }
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
        args: &[Expr],
        span: crate::token::Span,
    ) -> Result<(), CodegenError> {
        // When generating code for an imported module, function calls within
        // that module need to use mangled names (e.g., "_L5_utils_helper" instead
        // of "helper"), since all imported module functions are declared with
        // mangled names in the LLVM module.
        let (llvm_name, function) = if let Some(ref prefix) = self.current_module_prefix {
            let mangled = mangle_name(prefix, callee);
            let function = self.module.get_function(&mangled).or_else(|| {
                // Only allow fallback to unmangled name for known builtins.
                // Builtins are declared with their original names (not mangled).
                if BUILTIN_NAMES.contains(&callee) {
                    self.module.get_function(callee)
                } else {
                    None
                }
            });
            let function =
                function.ok_or_else(|| CodegenError::internal_function_not_found(callee, span))?;
            let llvm_name = if self.function_param_types.contains_key(&mangled) {
                mangled
            } else {
                callee.to_string()
            };
            (llvm_name, function)
        } else {
            let function = self
                .module
                .get_function(callee)
                .ok_or_else(|| CodegenError::internal_function_not_found(callee, span))?;
            (callee.to_string(), function)
        };

        let expected_param_types = self
            .function_param_types
            .get(&llvm_name)
            .cloned()
            .ok_or_else(|| CodegenError::internal_function_signature_not_found(&llvm_name, span))?;

        if args.len() != expected_param_types.len() {
            return Err(CodegenError::internal_call_arg_count_mismatch(
                callee,
                expected_param_types.len(),
                args.len(),
                span,
            ));
        }

        let llvm_args: Vec<BasicMetadataValueEnum<'ctx>> = args
            .iter()
            .zip(expected_param_types.iter())
            .map(|(arg, expected_ty)| self.generate_expr_value(arg, expected_ty).map(Into::into))
            .collect::<Result<_, _>>()?;

        self.builder
            .build_call(function, &llvm_args, "")
            .map_err(|e| CodegenError::internal_call_failed(callee, &e.to_string(), span))?;

        Ok(())
    }

    /// Generates a call to a module-qualified function.
    ///
    /// The function name is mangled using the length-prefix scheme via `mangle_name()`.
    /// If the module name is an alias, it is resolved to its mangle prefix first.
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
        args: &[Expr],
        span: crate::token::Span,
    ) -> Result<(), CodegenError> {
        // Resolve alias to mangle prefix for correct name mangling
        let mangle_prefix = self.resolve_module_alias(module_alias, span)?;
        let mangled_name = mangle_name(&mangle_prefix, function);

        let llvm_function = self
            .module
            .get_function(&mangled_name)
            .ok_or_else(|| CodegenError::internal_function_not_found(&mangled_name, span))?;

        let expected_param_types =
            self.function_param_types
                .get(&mangled_name)
                .ok_or_else(|| {
                    CodegenError::internal_function_signature_not_found(&mangled_name, span)
                })?;
        let expected_param_types = expected_param_types.clone();

        if args.len() != expected_param_types.len() {
            return Err(CodegenError::internal_call_arg_count_mismatch(
                &format!("{}.{}", module_alias, function),
                expected_param_types.len(),
                args.len(),
                span,
            ));
        }

        let llvm_args: Vec<BasicMetadataValueEnum<'ctx>> = args
            .iter()
            .zip(expected_param_types.iter())
            .map(|(arg, expected_ty)| self.generate_expr_value(arg, expected_ty).map(Into::into))
            .collect::<Result<_, _>>()?;

        self.builder
            .build_call(llvm_function, &llvm_args, "")
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
                    .lookup_variable(name)
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
            ExprKind::IfExpr {
                condition,
                then_block,
                else_block,
            } => self.generate_if_expr_value(
                condition,
                then_block,
                else_block,
                expected_ty,
                expr.span,
            ),
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

    /// Infers the type of an expression for comparison operator codegen.
    ///
    /// Semantic analysis has already validated types. This re-infers the operand
    /// type at the codegen level to determine which LLVM instructions to emit.
    ///
    /// Kept in sync with `SemanticAnalyzer::infer_expr_type` in `semantic/mod.rs`
    /// and `Codegen::get_expr_type` in `codegen/builtins.rs`.
    fn infer_binary_operand_type_for_codegen(
        &self,
        left: &Expr,
        right: &Expr,
        span: crate::token::Span,
    ) -> Result<Type, CodegenError> {
        let left_ty = self.infer_expr_type_for_comparison(left)?;
        let right_ty = self.infer_expr_type_for_comparison(right)?;

        if let Some(common_ty) =
            Expr::infer_common_binary_operand_type(left, &left_ty, right, &right_ty)
        {
            return Ok(common_ty);
        }

        Err(CodegenError::internal_binary_operand_type_mismatch(
            &left_ty.to_string(),
            &right_ty.to_string(),
            span,
        ))
    }

    fn infer_expr_type_for_comparison(&self, expr: &Expr) -> Result<Type, CodegenError> {
        match &expr.kind {
            ExprKind::IntLiteral(_) => Ok(Type::I64),
            ExprKind::Identifier(name) => {
                let binding = self
                    .lookup_variable(name)
                    .ok_or_else(|| CodegenError::internal_variable_not_found(name, expr.span))?;
                Ok(binding.ty().clone())
            }
            ExprKind::BinaryOp { left, op, right } => {
                if op.is_comparison() || op.is_logical() {
                    Ok(Type::Bool)
                } else {
                    self.infer_binary_operand_type_for_codegen(left, right, expr.span)
                }
            }
            ExprKind::UnaryOp { op, operand } => match op {
                UnaryOperator::Not => Ok(Type::Bool),
                UnaryOperator::Neg => self.infer_expr_type_for_comparison(operand),
            },
            ExprKind::IfExpr {
                condition: _,
                then_block: _,
                else_block: _,
            } => self.get_expr_type(expr),
            ExprKind::BoolLiteral(_) => Ok(Type::Bool),
            ExprKind::StringLiteral(_) => Ok(Type::String),
            ExprKind::Call { callee, .. } => {
                Err(CodegenError::internal_call_as_value(callee, expr.span))
            }
            ExprKind::MemberAccess { .. } => Err(
                CodegenError::internal_member_access_not_implemented(expr.span),
            ),
            ExprKind::ModuleCall {
                module, function, ..
            } => Err(CodegenError::internal_module_call_as_value(
                module, function, expr.span,
            )),
        }
    }

    /// Generates LLVM IR for an `if` expression value.
    fn generate_if_expr_value(
        &mut self,
        condition: &Expr,
        then_block: &IfExprBlock,
        else_block: &IfExprBlock,
        expected_ty: &Type,
        span: crate::token::Span,
    ) -> Result<BasicValueEnum<'ctx>, CodegenError> {
        let parent_fn = self
            .builder
            .get_insert_block()
            .and_then(|bb| bb.get_parent())
            .ok_or_else(|| CodegenError::internal_no_current_function(span))?;

        let llvm_branch_type = self.get_llvm_type(expected_ty);

        let then_bb = self.context.append_basic_block(parent_fn, "if_expr_then");
        let else_bb = self.context.append_basic_block(parent_fn, "if_expr_else");
        let merge_bb = self.context.append_basic_block(parent_fn, "if_expr_merge");

        let condition_value = self.generate_expr_value(condition, &Type::Bool)?;
        let condition_value = match condition_value {
            BasicValueEnum::IntValue(v) => v,
            _ => {
                return Err(CodegenError::internal_non_integer_value(
                    "if expression condition",
                    span,
                ));
            }
        };

        self.builder
            .build_conditional_branch(condition_value, then_bb, else_bb)
            .map_err(|e| CodegenError::internal_branch_failed(&e.to_string(), span))?;

        self.builder.position_at_end(then_bb);
        self.enter_variable_scope();
        let then_value_result = (|| -> Result<BasicValueEnum<'ctx>, CodegenError> {
            for stmt in &then_block.stmts {
                self.generate_stmt(stmt)?;
            }
            self.generate_expr_value(&then_block.value, expected_ty)
        })();
        self.exit_variable_scope();
        let then_value = then_value_result?;
        let then_end_bb = self
            .builder
            .get_insert_block()
            .ok_or_else(|| CodegenError::internal_no_current_function(span))?;
        self.builder
            .build_unconditional_branch(merge_bb)
            .map_err(|e| CodegenError::internal_branch_failed(&e.to_string(), span))?;

        self.builder.position_at_end(else_bb);
        self.enter_variable_scope();
        let else_value_result = (|| -> Result<BasicValueEnum<'ctx>, CodegenError> {
            for stmt in &else_block.stmts {
                self.generate_stmt(stmt)?;
            }
            self.generate_expr_value(&else_block.value, expected_ty)
        })();
        self.exit_variable_scope();
        let else_value = else_value_result?;
        let else_end_bb = self
            .builder
            .get_insert_block()
            .ok_or_else(|| CodegenError::internal_no_current_function(span))?;
        self.builder
            .build_unconditional_branch(merge_bb)
            .map_err(|e| CodegenError::internal_branch_failed(&e.to_string(), span))?;

        self.builder.position_at_end(merge_bb);
        let phi = self
            .builder
            .build_phi(llvm_branch_type, "if_expr_phi")
            .map_err(|e| CodegenError::internal_branch_failed(&e.to_string(), span))?;
        phi.add_incoming(&[(&then_value, then_end_bb), (&else_value, else_end_bb)]);

        Ok(phi.as_basic_value())
    }

    /// Generates LLVM IR for a comparison operation.
    ///
    /// Supports integer (i32, i64), bool (== and != only), and string (== and != only) comparisons.
    fn generate_comparison_op(
        &mut self,
        left: &Expr,
        op: BinaryOperator,
        right: &Expr,
        span: crate::token::Span,
    ) -> Result<BasicValueEnum<'ctx>, CodegenError> {
        let operand_ty = self.infer_binary_operand_type_for_codegen(left, right, span)?;

        match operand_ty {
            Type::I32 | Type::I64 | Type::Bool => {
                // Reject ordering operators for bool
                if operand_ty == Type::Bool && !op.is_equality() {
                    return Err(CodegenError::internal_binary_op_failed(
                        op,
                        "ordering operators not supported for bool",
                        span,
                    ));
                }

                let left_basic = self.generate_expr_value(left, &operand_ty)?;
                let left_value = match left_basic {
                    BasicValueEnum::IntValue(v) => v,
                    _ => return Err(CodegenError::internal_non_integer_value("comparison", span)),
                };
                let right_basic = self.generate_expr_value(right, &operand_ty)?;
                let right_value = match right_basic {
                    BasicValueEnum::IntValue(v) => v,
                    _ => return Err(CodegenError::internal_non_integer_value("comparison", span)),
                };

                let predicate = match op {
                    BinaryOperator::Equal => IntPredicate::EQ,
                    BinaryOperator::NotEqual => IntPredicate::NE,
                    BinaryOperator::LessThan => IntPredicate::SLT,
                    BinaryOperator::GreaterThan => IntPredicate::SGT,
                    BinaryOperator::LessEqual => IntPredicate::SLE,
                    BinaryOperator::GreaterEqual => IntPredicate::SGE,
                    BinaryOperator::Add
                    | BinaryOperator::Sub
                    | BinaryOperator::Mul
                    | BinaryOperator::Div
                    | BinaryOperator::Mod
                    | BinaryOperator::LogicalAnd
                    | BinaryOperator::LogicalOr => {
                        return Err(CodegenError::internal_binary_op_failed(
                            op,
                            "not a comparison operator",
                            span,
                        ));
                    }
                };

                let label = if operand_ty == Type::Bool {
                    "bool_cmp_tmp"
                } else {
                    "cmp_tmp"
                };
                let result = self
                    .builder
                    .build_int_compare(predicate, left_value, right_value, label)
                    .map_err(|e| CodegenError::internal_compare_failed(&e.to_string(), span))?;
                Ok(result.into())
            }
            Type::String => {
                let left_basic = self.generate_expr_value(left, &Type::String)?;
                let left_ptr = match left_basic {
                    BasicValueEnum::PointerValue(v) => v,
                    _ => {
                        return Err(CodegenError::internal_non_pointer_value(
                            "string comparison",
                            span,
                        ));
                    }
                };
                let right_basic = self.generate_expr_value(right, &Type::String)?;
                let right_ptr = match right_basic {
                    BasicValueEnum::PointerValue(v) => v,
                    _ => {
                        return Err(CodegenError::internal_non_pointer_value(
                            "string comparison",
                            span,
                        ));
                    }
                };

                let lak_streq = self.module.get_function("lak_streq").ok_or_else(|| {
                    CodegenError::internal_builtin_not_found_with_span("lak_streq", span)
                })?;

                let call_result = self
                    .builder
                    .build_call(
                        lak_streq,
                        &[
                            BasicMetadataValueEnum::PointerValue(left_ptr),
                            BasicMetadataValueEnum::PointerValue(right_ptr),
                        ],
                        "streq_tmp",
                    )
                    .map_err(|e| CodegenError::internal_streq_call_failed(&e.to_string(), span))?;

                let eq_result = match call_result.try_as_basic_value() {
                    inkwell::values::ValueKind::Basic(BasicValueEnum::IntValue(v)) => v,
                    inkwell::values::ValueKind::Basic(_) => {
                        return Err(CodegenError::internal_streq_unexpected_basic_type(span));
                    }
                    inkwell::values::ValueKind::Instruction(_) => {
                        return Err(CodegenError::internal_streq_returned_void(span));
                    }
                };

                match op {
                    BinaryOperator::Equal => Ok(eq_result.into()),
                    BinaryOperator::NotEqual => {
                        // build_not is bitwise NOT, which is equivalent to logical NOT
                        // only for i1 (1-bit) values. This relies on declare_lak_streq
                        // declaring the return type as bool_type() (i1).
                        let negated =
                            self.builder
                                .build_not(eq_result, "strneq_tmp")
                                .map_err(|e| {
                                    CodegenError::internal_compare_failed(&e.to_string(), span)
                                })?;
                        Ok(negated.into())
                    }
                    _ => Err(CodegenError::internal_binary_op_failed(
                        op,
                        "ordering operators not supported for string",
                        span,
                    )),
                }
            }
        }
    }

    /// Generates LLVM IR for logical operators with short-circuit evaluation.
    fn generate_logical_op_short_circuit(
        &mut self,
        left: &Expr,
        op: BinaryOperator,
        right: &Expr,
        span: crate::token::Span,
    ) -> Result<BasicValueEnum<'ctx>, CodegenError> {
        let current_fn = self
            .builder
            .get_insert_block()
            .and_then(|bb| bb.get_parent())
            .ok_or_else(|| CodegenError::internal_no_current_function(span))?;

        let left_basic = self.generate_expr_value(left, &Type::Bool)?;
        let left_value = match left_basic {
            BasicValueEnum::IntValue(v) => v,
            _ => return Err(CodegenError::internal_non_integer_value("logical", span)),
        };

        let rhs_block = self.context.append_basic_block(current_fn, "logical_rhs");
        let short_block = self.context.append_basic_block(current_fn, "logical_short");
        let merge_block = self.context.append_basic_block(current_fn, "logical_merge");

        match op {
            BinaryOperator::LogicalAnd => {
                self.builder
                    .build_conditional_branch(left_value, rhs_block, short_block)
                    .map_err(|e| CodegenError::internal_branch_failed(&e.to_string(), span))?;
            }
            BinaryOperator::LogicalOr => {
                self.builder
                    .build_conditional_branch(left_value, short_block, rhs_block)
                    .map_err(|e| CodegenError::internal_branch_failed(&e.to_string(), span))?;
            }
            _ => {
                return Err(CodegenError::internal_binary_op_failed(
                    op,
                    "not a logical operator",
                    span,
                ));
            }
        }

        self.builder.position_at_end(short_block);
        let short_value = match op {
            BinaryOperator::LogicalAnd => self.context.bool_type().const_zero(),
            BinaryOperator::LogicalOr => self.context.bool_type().const_all_ones(),
            _ => {
                return Err(CodegenError::internal_binary_op_failed(
                    op,
                    "not a logical operator",
                    span,
                ));
            }
        };
        let short_value_block = self
            .builder
            .get_insert_block()
            .ok_or_else(|| CodegenError::internal_no_current_function(span))?;
        self.builder
            .build_unconditional_branch(merge_block)
            .map_err(|e| CodegenError::internal_branch_failed(&e.to_string(), span))?;

        self.builder.position_at_end(rhs_block);
        let right_basic = self.generate_expr_value(right, &Type::Bool)?;
        let right_value = match right_basic {
            BasicValueEnum::IntValue(v) => v,
            _ => return Err(CodegenError::internal_non_integer_value("logical", span)),
        };
        let right_value_block = self
            .builder
            .get_insert_block()
            .ok_or_else(|| CodegenError::internal_no_current_function(span))?;
        self.builder
            .build_unconditional_branch(merge_block)
            .map_err(|e| CodegenError::internal_branch_failed(&e.to_string(), span))?;

        self.builder.position_at_end(merge_block);
        let phi = self
            .builder
            .build_phi(self.context.bool_type(), "logical_phi")
            .map_err(|e| CodegenError::internal_binary_op_failed(op, &e.to_string(), span))?;
        phi.add_incoming(&[
            (&short_value, short_value_block),
            (&right_value, right_value_block),
        ]);

        Ok(phi.as_basic_value())
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
    /// - A comparison operator is dispatched with a non-bool expected type
    /// - The expected type is string or bool for an arithmetic operator
    /// - LLVM instruction building fails
    ///
    /// # Runtime Panics
    ///
    /// For `+`, `-`, `*` operators, generates overflow-checked operations using
    /// LLVM signed overflow intrinsics. The compiled program panics at runtime
    /// if the result overflows the integer type's range.
    ///
    /// For `/` and `%` operators, generates both a division-by-zero check and
    /// an overflow check (MIN / -1 and MIN % -1). The compiled program panics
    /// at runtime with "integer overflow" if the dividend is the type's minimum
    /// value and the divisor is -1.
    fn generate_binary_op(
        &mut self,
        left: &Expr,
        op: BinaryOperator,
        right: &Expr,
        expected_ty: &Type,
        span: crate::token::Span,
    ) -> Result<BasicValueEnum<'ctx>, CodegenError> {
        if op.is_logical() {
            if *expected_ty != Type::Bool {
                return Err(CodegenError::internal_logical_expected_bool(
                    op,
                    expected_ty,
                    span,
                ));
            }
            return self.generate_logical_op_short_circuit(left, op, right, span);
        }

        // Dispatch comparison operators to dedicated handler
        if op.is_comparison() {
            if *expected_ty != Type::Bool {
                return Err(CodegenError::internal_comparison_expected_bool(
                    op,
                    expected_ty,
                    span,
                ));
            }
            return self.generate_comparison_op(left, op, right, span);
        }

        // Arithmetic operators below: semantic analysis guarantees the type is numeric (not string or bool)
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
            BinaryOperator::Add => self.generate_overflow_checked_binop(
                left_value,
                right_value,
                "llvm.sadd.with.overflow",
                span,
            )?,
            BinaryOperator::Sub => self.generate_overflow_checked_binop(
                left_value,
                right_value,
                "llvm.ssub.with.overflow",
                span,
            )?,
            BinaryOperator::Mul => self.generate_overflow_checked_binop(
                left_value,
                right_value,
                "llvm.smul.with.overflow",
                span,
            )?,
            BinaryOperator::Div => {
                self.generate_division_zero_check(right_value, "division by zero", span)?;
                self.generate_division_overflow_check(left_value, right_value, span)?;
                self.builder
                    .build_int_signed_div(left_value, right_value, "div_tmp")
                    .map_err(|e| {
                        CodegenError::internal_binary_op_failed(op, &e.to_string(), span)
                    })?
            }
            BinaryOperator::Mod => {
                self.generate_division_zero_check(right_value, "modulo by zero", span)?;
                self.generate_division_overflow_check(left_value, right_value, span)?;
                self.builder
                    .build_int_signed_rem(left_value, right_value, "mod_tmp")
                    .map_err(|e| {
                        CodegenError::internal_binary_op_failed(op, &e.to_string(), span)
                    })?
            }
            BinaryOperator::Equal
            | BinaryOperator::NotEqual
            | BinaryOperator::LessThan
            | BinaryOperator::GreaterThan
            | BinaryOperator::LessEqual
            | BinaryOperator::GreaterEqual
            | BinaryOperator::LogicalAnd
            | BinaryOperator::LogicalOr => {
                return Err(CodegenError::internal_binary_op_failed(
                    op,
                    "non-arithmetic operators should have been dispatched before reaching this match",
                    span,
                ));
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
    ///
    /// # Runtime Panics
    ///
    /// For negation, generates overflow-checked subtraction from zero
    /// (`0 - operand`) using `llvm.ssub.with.overflow`. The compiled program
    /// panics at runtime if the result overflows (e.g., negating the minimum
    /// value of a signed integer type).
    fn generate_unary_op(
        &mut self,
        operand: &Expr,
        op: UnaryOperator,
        expected_ty: &Type,
        span: crate::token::Span,
    ) -> Result<BasicValueEnum<'ctx>, CodegenError> {
        let result = match op {
            UnaryOperator::Neg => {
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

                let zero = operand_value.get_type().const_int(0, false);
                self.generate_overflow_checked_binop(
                    zero,
                    operand_value,
                    "llvm.ssub.with.overflow",
                    span,
                )
                .map_err(|e| CodegenError::wrap_in_unary_context(&e, op, span))?
            }
            UnaryOperator::Not => {
                if *expected_ty != Type::Bool {
                    return Err(CodegenError::internal_unary_not_expected_bool(
                        expected_ty,
                        span,
                    ));
                }
                let operand_basic = self
                    .generate_expr_value(operand, &Type::Bool)
                    .map_err(|e| CodegenError::wrap_in_unary_context(&e, op, span))?;
                let operand_value = match operand_basic {
                    BasicValueEnum::IntValue(v) => v,
                    _ => return Err(CodegenError::internal_non_integer_value("unary", span)),
                };
                self.builder
                    .build_not(operand_value, "not_tmp")
                    .map_err(|e| {
                        CodegenError::wrap_in_unary_context(
                            &CodegenError::internal_compare_failed(&e.to_string(), span),
                            op,
                            span,
                        )
                    })?
            }
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
    ///   ; Caller continues here (may insert additional checks before division)
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
            .ok_or_else(|| CodegenError::internal_builtin_not_found_with_span("lak_panic", span))?;

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

    /// Generates a runtime check for division/modulo overflow (MIN / -1).
    ///
    /// Signed integer division overflows when the dividend is the type's minimum
    /// value and the divisor is -1, because the result (positive of MIN) exceeds
    /// the type's maximum value. LLVM's `sdiv` and `srem` instructions have
    /// undefined behavior for this case.
    ///
    /// This check must be called after `generate_division_zero_check` (which
    /// guarantees the divisor is non-zero) and before the actual `sdiv`/`srem`.
    ///
    /// # LLVM IR Pattern
    ///
    /// ```text
    ///   %is_neg_one = icmp eq <type> %divisor, -1
    ///   %is_min = icmp eq <type> %dividend, TYPE_MIN
    ///   %is_overflow = and i1 %is_neg_one, %is_min
    ///   br i1 %is_overflow, label %div_overflow_panic, label %div_overflow_safe
    ///
    /// div_overflow_panic:
    ///   call void @lak_panic("integer overflow")
    ///   unreachable
    ///
    /// div_overflow_safe:
    ///   ; actual division/modulo happens here
    /// ```
    ///
    /// # Arguments
    ///
    /// * `dividend` - The dividend (left operand) value to check
    /// * `divisor` - The divisor (right operand) value to check
    /// * `span` - The source span for error reporting
    fn generate_division_overflow_check(
        &mut self,
        dividend: inkwell::values::IntValue<'ctx>,
        divisor: inkwell::values::IntValue<'ctx>,
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
            .append_basic_block(current_fn, "div_overflow_panic");
        let safe_block = self
            .context
            .append_basic_block(current_fn, "div_overflow_safe");

        // Create -1 constant: all bits set to 1
        let neg_one = divisor.get_type().const_all_ones();

        // Create TYPE_MIN constant: only the sign bit set
        // For i32: 0x80000000, for i64: 0x8000000000000000
        let bit_width = dividend.get_type().get_bit_width();
        let min_value = 1u64 << (bit_width - 1);
        let type_min = dividend.get_type().const_int(min_value, false);

        // Check if divisor == -1
        let is_neg_one = self
            .builder
            .build_int_compare(IntPredicate::EQ, divisor, neg_one, "is_neg_one")
            .map_err(|e| CodegenError::internal_compare_failed(&e.to_string(), span))?;

        // Check if dividend == TYPE_MIN
        let is_min = self
            .builder
            .build_int_compare(IntPredicate::EQ, dividend, type_min, "is_min")
            .map_err(|e| CodegenError::internal_compare_failed(&e.to_string(), span))?;

        // Both conditions must be true for overflow
        let is_overflow = self
            .builder
            .build_and(is_neg_one, is_min, "is_div_overflow")
            .map_err(|e| CodegenError::internal_compare_failed(&e.to_string(), span))?;

        // Branch: if overflow goto panic_block, else goto safe_block
        self.builder
            .build_conditional_branch(is_overflow, panic_block, safe_block)
            .map_err(|e| CodegenError::internal_branch_failed(&e.to_string(), span))?;

        // Build panic block
        self.builder.position_at_end(panic_block);

        // Create global string for error message
        let panic_msg = self
            .builder
            .build_global_string_ptr("integer overflow", "div_overflow_msg")
            .map_err(|e| CodegenError::internal_string_ptr_failed(&e.to_string(), span))?
            .as_pointer_value();

        // Call lak_panic
        let lak_panic = self
            .module
            .get_function("lak_panic")
            .ok_or_else(|| CodegenError::internal_builtin_not_found_with_span("lak_panic", span))?;

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

    /// Generates an overflow-checked binary operation using LLVM intrinsics.
    ///
    /// Uses LLVM signed overflow intrinsics (e.g., `llvm.sadd.with.overflow.i32`)
    /// that return a `{result, overflow_flag}` struct. If the overflow flag is set,
    /// the program panics.
    ///
    /// # LLVM IR Pattern
    ///
    /// ```text
    ///   %result_struct = call {i32, i1} @llvm.sadd.with.overflow.i32(i32 %left, i32 %right)
    ///   %result = extractvalue {i32, i1} %result_struct, 0
    ///   %overflow = extractvalue {i32, i1} %result_struct, 1
    ///   br i1 %overflow, label %overflow_panic, label %overflow_safe
    ///
    /// overflow_panic:
    ///   call void @lak_panic("integer overflow")
    ///   unreachable
    ///
    /// overflow_safe:
    ///   ; continue with %result
    /// ```
    ///
    /// # Arguments
    ///
    /// * `left` - The left operand
    /// * `right` - The right operand
    /// * `intrinsic_prefix` - The LLVM intrinsic prefix (e.g., "llvm.sadd.with.overflow")
    /// * `span` - The source span for error reporting
    fn generate_overflow_checked_binop(
        &mut self,
        left: inkwell::values::IntValue<'ctx>,
        right: inkwell::values::IntValue<'ctx>,
        intrinsic_prefix: &str,
        span: crate::token::Span,
    ) -> Result<inkwell::values::IntValue<'ctx>, CodegenError> {
        let int_type = left.get_type();

        // Look up the LLVM intrinsic
        let intrinsic = Intrinsic::find(intrinsic_prefix)
            .ok_or_else(|| CodegenError::internal_intrinsic_not_found(intrinsic_prefix, span))?;

        // Get intrinsic declaration for the specific integer type
        let intrinsic_fn = intrinsic
            .get_declaration(&self.module, &[int_type.into()])
            .ok_or_else(|| {
                CodegenError::internal_intrinsic_declaration_failed(intrinsic_prefix, span)
            })?;

        // Call the intrinsic: returns {result, overflow_flag}
        let call_result = self
            .builder
            .build_call(intrinsic_fn, &[left.into(), right.into()], "overflow_check")
            .map_err(|e| {
                CodegenError::internal_intrinsic_call_failed(intrinsic_prefix, &e.to_string(), span)
            })?;

        let result_struct = match call_result.try_as_basic_value() {
            inkwell::values::ValueKind::Basic(BasicValueEnum::StructValue(sv)) => sv,
            inkwell::values::ValueKind::Basic(_) => {
                return Err(CodegenError::internal_intrinsic_call_failed(
                    intrinsic_prefix,
                    "expected struct return type",
                    span,
                ));
            }
            inkwell::values::ValueKind::Instruction(_) => {
                return Err(CodegenError::internal_intrinsic_call_failed(
                    intrinsic_prefix,
                    "intrinsic returned void",
                    span,
                ));
            }
        };

        // Extract result value (index 0) and overflow flag (index 1)
        let result_value = match self
            .builder
            .build_extract_value(result_struct, 0, "result")
            .map_err(|e| CodegenError::internal_extract_value_failed(&e.to_string(), span))?
        {
            BasicValueEnum::IntValue(v) => v,
            _ => {
                return Err(CodegenError::internal_extract_value_failed(
                    "expected integer value at index 0",
                    span,
                ));
            }
        };

        let overflow_flag = match self
            .builder
            .build_extract_value(result_struct, 1, "overflow")
            .map_err(|e| CodegenError::internal_extract_value_failed(&e.to_string(), span))?
        {
            BasicValueEnum::IntValue(v) => v,
            _ => {
                return Err(CodegenError::internal_extract_value_failed(
                    "expected integer value at index 1",
                    span,
                ));
            }
        };

        // Create basic blocks for panic and safe paths
        let current_fn = self
            .builder
            .get_insert_block()
            .and_then(|bb| bb.get_parent())
            .ok_or_else(|| CodegenError::internal_no_current_function(span))?;

        let panic_block = self
            .context
            .append_basic_block(current_fn, "overflow_panic");
        let safe_block = self.context.append_basic_block(current_fn, "overflow_safe");

        // Branch: if overflow goto panic_block, else goto safe_block
        self.builder
            .build_conditional_branch(overflow_flag, panic_block, safe_block)
            .map_err(|e| CodegenError::internal_branch_failed(&e.to_string(), span))?;

        // Build panic block
        self.builder.position_at_end(panic_block);

        let panic_msg = self
            .builder
            .build_global_string_ptr("integer overflow", "overflow_msg")
            .map_err(|e| CodegenError::internal_string_ptr_failed(&e.to_string(), span))?
            .as_pointer_value();

        let lak_panic = self
            .module
            .get_function("lak_panic")
            .ok_or_else(|| CodegenError::internal_builtin_not_found_with_span("lak_panic", span))?;

        self.builder
            .build_call(
                lak_panic,
                &[BasicMetadataValueEnum::PointerValue(panic_msg)],
                "",
            )
            .map_err(|e| CodegenError::internal_panic_call_failed(&e.to_string(), span))?;

        self.builder
            .build_unreachable()
            .map_err(|e| CodegenError::internal_unreachable_failed(&e.to_string(), span))?;

        // Position builder at safe block
        self.builder.position_at_end(safe_block);

        Ok(result_value)
    }
}
