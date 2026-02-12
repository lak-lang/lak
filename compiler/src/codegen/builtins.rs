//! Built-in function code generation.
//!
//! This module implements code generation for Lak's built-in functions:
//! println (string, i32, i64, bool variants), panic, and streq (string equality).

use super::Codegen;
use super::error::{CodegenError, CodegenErrorKind};
use crate::ast::{Expr, ExprKind, StmtKind, Type};
use crate::token::Span;
use inkwell::AddressSpace;
use inkwell::module::Linkage;
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum};
use std::collections::HashMap;

/// Names of all builtin runtime functions declared by `declare_builtins()`.
///
/// This list is used by `generate_user_function_call()` in `expr.rs` to
/// determine which functions should be looked up with unmangled names
/// when generating code for imported modules.
///
/// This list must be kept in sync with the functions declared in `declare_builtins()`.
/// Enforced by `test_builtin_names_matches_declare_builtins` in `tests.rs`.
pub(super) const BUILTIN_NAMES: &[&str] = &[
    "lak_println",
    "lak_println_i32",
    "lak_println_i64",
    "lak_println_bool",
    "lak_panic",
    "lak_streq",
];

impl<'ctx> Codegen<'ctx> {
    /// Loads a value from a stack allocation and extracts it as an `IntValue`.
    fn load_and_extract_int_value(
        &self,
        ty: inkwell::types::IntType<'ctx>,
        alloca: inkwell::values::PointerValue<'ctx>,
        var_name: &str,
        context_label: &str,
        span: Span,
    ) -> Result<inkwell::values::IntValue<'ctx>, CodegenError> {
        let loaded = self
            .builder
            .build_load(ty, alloca, &format!("{}_load", var_name))
            .map_err(|e| {
                CodegenError::internal_variable_load_failed(var_name, &e.to_string(), span)
            })?;
        match loaded {
            BasicValueEnum::IntValue(v) => Ok(v),
            _ => Err(CodegenError::internal_non_integer_value(
                context_label,
                span,
            )),
        }
    }

    /// Loads a value from a stack allocation and extracts it as a `PointerValue`.
    fn load_and_extract_pointer_value(
        &self,
        alloca: inkwell::values::PointerValue<'ctx>,
        var_name: &str,
        context_label: &str,
        span: Span,
    ) -> Result<inkwell::values::PointerValue<'ctx>, CodegenError> {
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let loaded = self
            .builder
            .build_load(ptr_type, alloca, &format!("{}_load", var_name))
            .map_err(|e| {
                CodegenError::internal_variable_load_failed(var_name, &e.to_string(), span)
            })?;
        match loaded {
            BasicValueEnum::PointerValue(v) => Ok(v),
            _ => Err(CodegenError::internal_non_pointer_value(
                context_label,
                span,
            )),
        }
    }

    /// Declares the Lak runtime `lak_println` function for use in generated code.
    ///
    /// This creates an external function declaration with the signature:
    /// `void lak_println(const char* s)`
    pub(super) fn declare_lak_println(&self) {
        let void_type = self.context.void_type();
        let i8_ptr_type = self.context.ptr_type(AddressSpace::default());

        let println_type = void_type.fn_type(&[i8_ptr_type.into()], false);
        self.module
            .add_function("lak_println", println_type, Some(Linkage::External));
    }

    /// Declares the Lak runtime `lak_println_i32` function for use in generated code.
    ///
    /// This creates an external function declaration with the signature:
    /// `void lak_println_i32(i32 value)`
    pub(super) fn declare_lak_println_i32(&self) {
        let void_type = self.context.void_type();
        let i32_type = self.context.i32_type();

        let println_type = void_type.fn_type(&[i32_type.into()], false);
        self.module
            .add_function("lak_println_i32", println_type, Some(Linkage::External));
    }

    /// Declares the Lak runtime `lak_println_i64` function for use in generated code.
    ///
    /// This creates an external function declaration with the signature:
    /// `void lak_println_i64(i64 value)`
    pub(super) fn declare_lak_println_i64(&self) {
        let void_type = self.context.void_type();
        let i64_type = self.context.i64_type();

        let println_type = void_type.fn_type(&[i64_type.into()], false);
        self.module
            .add_function("lak_println_i64", println_type, Some(Linkage::External));
    }

    /// Declares the Lak runtime `lak_println_bool` function for use in generated code.
    ///
    /// This creates an external function declaration with the signature:
    /// `void lak_println_bool(bool value)`
    pub(super) fn declare_lak_println_bool(&self) {
        let void_type = self.context.void_type();
        let bool_type = self.context.bool_type();

        let println_type = void_type.fn_type(&[bool_type.into()], false);
        self.module
            .add_function("lak_println_bool", println_type, Some(Linkage::External));
    }

    /// Declares the Lak runtime `lak_panic` function for use in generated code.
    ///
    /// This creates an external function declaration with the signature:
    /// `void lak_panic(const char* message)` with noreturn attribute.
    ///
    /// The noreturn attribute tells LLVM that this function never returns,
    /// allowing for proper control flow analysis and optimization.
    pub(super) fn declare_lak_panic(&self) {
        let void_type = self.context.void_type();
        let i8_ptr_type = self.context.ptr_type(AddressSpace::default());

        let panic_type = void_type.fn_type(&[i8_ptr_type.into()], false);
        let panic_fn = self
            .module
            .add_function("lak_panic", panic_type, Some(Linkage::External));

        // Add noreturn attribute to the function
        let noreturn_kind_id = inkwell::attributes::Attribute::get_named_enum_kind_id("noreturn");
        // create_enum_attribute(kind_id, value): value is 0 for boolean attributes like noreturn
        let noreturn_attr = self.context.create_enum_attribute(noreturn_kind_id, 0);
        panic_fn.add_attribute(inkwell::attributes::AttributeLoc::Function, noreturn_attr);
    }

    /// Declares the Lak runtime `lak_streq` function for use in generated code.
    ///
    /// This creates an external function declaration with the signature:
    /// `bool (i1) lak_streq(ptr a, ptr b)`
    ///
    /// The return type is declared as `bool_type()` (LLVM `i1`), matching the Rust
    /// `bool` return type of the runtime function. This is consistent with how
    /// `declare_lak_println_bool` uses `bool_type()` for its bool parameter.
    /// The `i1` return type is required by `generate_comparison_op` in `expr.rs`,
    /// which uses `build_not` (bitwise NOT) for `!=` — this is only equivalent
    /// to logical NOT for `i1` values.
    pub(super) fn declare_lak_streq(&self) {
        let bool_type = self.context.bool_type();
        let i8_ptr_type = self.context.ptr_type(AddressSpace::default());

        let streq_type = bool_type.fn_type(&[i8_ptr_type.into(), i8_ptr_type.into()], false);
        self.module
            .add_function("lak_streq", streq_type, Some(Linkage::External));
    }

    /// Infers a common binary operand type with integer-literal adaptation.
    ///
    /// This mirrors semantic analysis rules used by `infer_expr_type`:
    /// - same type on both sides => that type
    /// - integer literal mixed with `i32`/`i64` => non-literal integer type
    /// - non-adaptable mix => internal error (semantic should have rejected it)
    fn infer_binary_operand_type_with_locals(
        &self,
        left: &Expr,
        right: &Expr,
        local_types: &HashMap<String, Type>,
        span: Span,
    ) -> Result<Type, CodegenError> {
        let left_ty = self.get_expr_type_with_locals(left, local_types)?;
        let right_ty = self.get_expr_type_with_locals(right, local_types)?;

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

    /// Returns the type of an expression for println dispatch.
    ///
    /// This is used to determine which println runtime function to call.
    /// The type dispatch is compile-time: each supported type maps to a dedicated
    /// runtime function (`lak_println`, `lak_println_i32`, `lak_println_i64`,
    /// `lak_println_bool`).
    ///
    /// Type mapping:
    /// - `IntLiteral` → `Type::I64` (standalone integer literals default to i64)
    /// - `StringLiteral` → `Type::String`
    /// - `BoolLiteral` → `Type::Bool`
    /// - `Identifier` → the variable's declared type (may be i32, i64, string, or bool)
    ///
    /// Integer literals in arithmetic/comparison expressions are adapted to
    /// the non-literal integer operand type (`i32` or `i64`).
    ///
    /// Kept in sync with `SemanticAnalyzer::infer_expr_type` in `semantic/mod.rs`
    /// and `Codegen::infer_expr_type_for_comparison` in `codegen/expr.rs`.
    ///
    /// # Returns
    ///
    /// - `Ok(Type)` - The resolved type for supported expressions
    /// - `Err(CodegenError)` - An internal error for unsupported expressions
    fn get_expr_type_with_locals(
        &self,
        expr: &Expr,
        local_types: &HashMap<String, Type>,
    ) -> Result<Type, CodegenError> {
        match &expr.kind {
            ExprKind::IntLiteral(_) => Ok(Type::I64),
            ExprKind::StringLiteral(_) => Ok(Type::String),
            ExprKind::BoolLiteral(_) => Ok(Type::Bool),
            ExprKind::Identifier(name) => {
                if let Some(ty) = local_types.get(name) {
                    return Ok(ty.clone());
                }
                self.lookup_variable(name)
                    .map(|b| b.ty().clone())
                    .ok_or_else(|| CodegenError::internal_variable_not_found(name, expr.span))
            }
            ExprKind::Call { callee, .. } => {
                Err(CodegenError::internal_println_call_arg(callee, expr.span))
            }
            ExprKind::BinaryOp { left, op, right } => {
                if op.is_comparison() || op.is_logical() {
                    Ok(Type::Bool)
                } else {
                    self.infer_binary_operand_type_with_locals(left, right, local_types, expr.span)
                }
            }
            ExprKind::UnaryOp { op, operand } => match op {
                crate::ast::UnaryOperator::Not => Ok(Type::Bool),
                // For arithmetic negation, infer the type from the operand.
                crate::ast::UnaryOperator::Neg => {
                    self.get_expr_type_with_locals(operand, local_types)
                }
            },
            ExprKind::IfExpr {
                condition: _,
                then_block,
                else_block,
            } => {
                let mut then_locals = local_types.clone();
                for stmt in &then_block.stmts {
                    if let StmtKind::Let { name, ty, .. } = &stmt.kind {
                        then_locals.insert(name.clone(), ty.clone());
                    }
                }
                let then_ty = self.get_expr_type_with_locals(&then_block.value, &then_locals)?;

                let mut else_locals = local_types.clone();
                for stmt in &else_block.stmts {
                    if let StmtKind::Let { name, ty, .. } = &stmt.kind {
                        else_locals.insert(name.clone(), ty.clone());
                    }
                }
                let else_ty = self.get_expr_type_with_locals(&else_block.value, &else_locals)?;

                if then_ty != else_ty {
                    return Err(CodegenError::new(
                        CodegenErrorKind::InternalError,
                        format!(
                            "Internal error: if expression branches have mismatched types '{}' and '{}' in codegen. Semantic analysis should have caught this. This is a compiler bug.",
                            then_ty, else_ty
                        ),
                        expr.span,
                    ));
                }

                Ok(then_ty)
            }
            ExprKind::MemberAccess { .. } => Err(
                CodegenError::internal_member_access_not_implemented(expr.span),
            ),
            ExprKind::ModuleCall {
                module,
                function,
                args: _,
            } => Err(CodegenError::internal_module_call_as_value(
                module, function, expr.span,
            )),
        }
    }

    pub(super) fn get_expr_type(&self, expr: &Expr) -> Result<Type, CodegenError> {
        self.get_expr_type_with_locals(expr, &HashMap::new())
    }

    /// Generates LLVM IR for a `println` call.
    ///
    /// Implements `println(value)` by calling the appropriate Lak runtime function
    /// based on the argument type. Type dispatch is performed via `get_expr_type()`,
    /// which determines the type from the expression kind or variable declaration.
    ///
    /// Type dispatch:
    /// - `string` → `lak_println` (string literals or string variables)
    /// - `i32` → `lak_println_i32` (i32 values, including adapted literal mixes)
    /// - `i64` → `lak_println_i64` (i64 values and standalone integer literals)
    /// - `bool` → `lak_println_bool` (boolean literals or bool variables)
    ///
    /// # Validation responsibilities
    ///
    /// - **Semantic analysis**: Validates argument count (exactly 1), variable existence,
    ///   and type consistency (including integer literal adaptation and integer range checks).
    /// - **Codegen (`get_expr_type`)**: Determines the actual type of the argument and
    ///   dispatches to the appropriate runtime function.
    ///
    /// # Arguments
    ///
    /// * `args` - The arguments passed to `println`
    /// * `span` - The source location of the println call
    ///
    /// # Errors
    ///
    /// Returns an internal error if:
    /// - Argument count is not 1 (semantic analysis should have caught this)
    /// - Expression type cannot be determined (e.g., function calls, undefined variables)
    pub(super) fn generate_println(
        &mut self,
        args: &[Expr],
        span: Span,
    ) -> Result<(), CodegenError> {
        // Semantic analysis guarantees exactly one argument
        if args.len() != 1 {
            return Err(CodegenError::internal_println_arg_count(args.len(), span));
        }

        let arg = &args[0];

        // Determine the type of the argument and call the appropriate runtime function
        let arg_type = self.get_expr_type(arg)?;

        match arg_type {
            Type::String => self.generate_println_string(arg, span),
            Type::I32 => self.generate_println_i32(arg, span),
            Type::I64 => self.generate_println_i64(arg, span),
            Type::Bool => self.generate_println_bool(arg, span),
        }
    }

    /// Generates LLVM IR for `println` with a string argument.
    fn generate_println_string(&mut self, arg: &Expr, span: Span) -> Result<(), CodegenError> {
        let string_ptr = match &arg.kind {
            ExprKind::StringLiteral(s) => self
                .builder
                .build_global_string_ptr(s, "str")
                .map_err(|e| CodegenError::internal_string_ptr_failed(&e.to_string(), arg.span))?
                .as_pointer_value(),
            ExprKind::Identifier(name) => {
                let binding = self
                    .lookup_variable(name)
                    .ok_or_else(|| CodegenError::internal_variable_not_found(name, arg.span))?;

                self.load_and_extract_pointer_value(
                    binding.alloca(),
                    name,
                    "println_string load",
                    arg.span,
                )?
            }
            ExprKind::IfExpr { .. } => match self.generate_expr_value(arg, &Type::String)? {
                BasicValueEnum::PointerValue(v) => v,
                _ => {
                    return Err(CodegenError::internal_println_invalid_string_arg(arg.span));
                }
            },
            _ => {
                return Err(CodegenError::internal_println_invalid_string_arg(arg.span));
            }
        };

        let lak_println = self
            .module
            .get_function("lak_println")
            .ok_or_else(|| CodegenError::internal_builtin_not_found("lak_println"))?;

        self.builder
            .build_call(
                lak_println,
                &[BasicMetadataValueEnum::PointerValue(string_ptr)],
                "",
            )
            .map_err(|e| CodegenError::internal_println_call_failed(&e.to_string(), span))?;

        Ok(())
    }

    /// Generates LLVM IR for `println` with an i32 argument.
    ///
    /// Note: i32-typed expressions, including mixed literal arithmetic adapted to i32,
    /// reach this function.
    fn generate_println_i32(&mut self, arg: &Expr, span: Span) -> Result<(), CodegenError> {
        let i32_value = match &arg.kind {
            ExprKind::Identifier(name) => {
                let binding = self
                    .lookup_variable(name)
                    .ok_or_else(|| CodegenError::internal_variable_not_found(name, arg.span))?;

                if binding.ty() != &Type::I32 {
                    return Err(CodegenError::internal_println_type_mismatch(
                        name,
                        "i32",
                        &binding.ty().to_string(),
                        arg.span,
                    ));
                }

                self.load_and_extract_int_value(
                    self.context.i32_type(),
                    binding.alloca(),
                    name,
                    "println_i32 load",
                    arg.span,
                )?
            }
            ExprKind::BinaryOp { .. } | ExprKind::UnaryOp { .. } | ExprKind::IfExpr { .. } => {
                // For binary/unary operations, generate the expression value
                match self.generate_expr_value(arg, &Type::I32)? {
                    BasicValueEnum::IntValue(v) => v,
                    _ => {
                        return Err(CodegenError::internal_non_integer_value(
                            "println_i32 expr",
                            arg.span,
                        ));
                    }
                }
            }
            // Standalone integer literals are routed to generate_println_i64().
            _ => {
                return Err(CodegenError::internal_println_invalid_i32_arg(arg.span));
            }
        };

        let lak_println_i32 = self
            .module
            .get_function("lak_println_i32")
            .ok_or_else(|| CodegenError::internal_builtin_not_found("lak_println_i32"))?;

        self.builder
            .build_call(
                lak_println_i32,
                &[BasicMetadataValueEnum::IntValue(i32_value)],
                "",
            )
            .map_err(|e| CodegenError::internal_println_call_failed(&e.to_string(), span))?;

        Ok(())
    }

    /// Generates LLVM IR for `println` with a bool argument.
    ///
    /// This handles boolean literals, bool variables, and bool-producing expressions
    /// (such as comparison operations).
    fn generate_println_bool(&mut self, arg: &Expr, span: Span) -> Result<(), CodegenError> {
        let bool_value = match &arg.kind {
            ExprKind::BoolLiteral(value) => {
                self.context.bool_type().const_int(*value as u64, false)
            }
            ExprKind::Identifier(name) => {
                let binding = self
                    .lookup_variable(name)
                    .ok_or_else(|| CodegenError::internal_variable_not_found(name, arg.span))?;

                if binding.ty() != &Type::Bool {
                    return Err(CodegenError::internal_println_type_mismatch(
                        name,
                        "bool",
                        &binding.ty().to_string(),
                        arg.span,
                    ));
                }

                self.load_and_extract_int_value(
                    self.context.bool_type(),
                    binding.alloca(),
                    name,
                    "println_bool load",
                    arg.span,
                )?
            }
            ExprKind::BinaryOp { .. } | ExprKind::UnaryOp { .. } | ExprKind::IfExpr { .. } => {
                // For binary/unary operations, generate the expression value
                match self.generate_expr_value(arg, &Type::Bool)? {
                    BasicValueEnum::IntValue(v) => v,
                    _ => {
                        return Err(CodegenError::internal_non_integer_value(
                            "println_bool expr",
                            arg.span,
                        ));
                    }
                }
            }
            _ => {
                return Err(CodegenError::internal_println_invalid_bool_arg(arg.span));
            }
        };

        let lak_println_bool = self
            .module
            .get_function("lak_println_bool")
            .ok_or_else(|| CodegenError::internal_builtin_not_found("lak_println_bool"))?;

        self.builder
            .build_call(
                lak_println_bool,
                &[BasicMetadataValueEnum::IntValue(bool_value)],
                "",
            )
            .map_err(|e| CodegenError::internal_println_call_failed(&e.to_string(), span))?;

        Ok(())
    }

    /// Generates LLVM IR for `println` with an i64 argument.
    ///
    /// This handles standalone integer literals (default i64),
    /// i64 variables, and i64-typed binary operations.
    fn generate_println_i64(&mut self, arg: &Expr, span: Span) -> Result<(), CodegenError> {
        let i64_value = match &arg.kind {
            ExprKind::IntLiteral(value) => self.context.i64_type().const_int(*value as u64, true),
            ExprKind::Identifier(name) => {
                let binding = self
                    .lookup_variable(name)
                    .ok_or_else(|| CodegenError::internal_variable_not_found(name, arg.span))?;

                if binding.ty() != &Type::I64 {
                    return Err(CodegenError::internal_println_type_mismatch(
                        name,
                        "i64",
                        &binding.ty().to_string(),
                        arg.span,
                    ));
                }

                self.load_and_extract_int_value(
                    self.context.i64_type(),
                    binding.alloca(),
                    name,
                    "println_i64 load",
                    arg.span,
                )?
            }
            ExprKind::BinaryOp { .. } | ExprKind::UnaryOp { .. } | ExprKind::IfExpr { .. } => {
                // For binary/unary operations, generate the expression value
                match self.generate_expr_value(arg, &Type::I64)? {
                    BasicValueEnum::IntValue(v) => v,
                    _ => {
                        return Err(CodegenError::internal_non_integer_value(
                            "println_i64 expr",
                            arg.span,
                        ));
                    }
                }
            }
            _ => {
                return Err(CodegenError::internal_println_invalid_i64_arg(arg.span));
            }
        };

        let lak_println_i64 = self
            .module
            .get_function("lak_println_i64")
            .ok_or_else(|| CodegenError::internal_builtin_not_found("lak_println_i64"))?;

        self.builder
            .build_call(
                lak_println_i64,
                &[BasicMetadataValueEnum::IntValue(i64_value)],
                "",
            )
            .map_err(|e| CodegenError::internal_println_call_failed(&e.to_string(), span))?;

        Ok(())
    }

    /// Generates LLVM IR for a `panic` call.
    ///
    /// Implements `panic(message)` by:
    /// 1. Calling the Lak runtime `lak_panic` function with the message
    /// 2. Inserting an `unreachable` instruction after the call
    ///
    /// The `unreachable` instruction tells LLVM that execution never reaches
    /// this point, which is guaranteed by the `noreturn` attribute on `lak_panic`.
    ///
    /// # Arguments
    ///
    /// * `args` - The arguments passed to `panic` (must contain exactly 1 string)
    /// * `span` - The source location of the panic call
    ///
    /// # Errors
    ///
    /// Returns an internal error if:
    /// - Argument count is not 1 (semantic analysis should have caught this)
    /// - Argument does not produce a string value
    pub(super) fn generate_panic(&mut self, args: &[Expr], span: Span) -> Result<(), CodegenError> {
        // Semantic analysis guarantees exactly one argument of Type::String.
        // This can be any expression that evaluates to string.
        if args.len() != 1 {
            return Err(CodegenError::internal_panic_arg_count(args.len(), span));
        }

        let arg = &args[0];

        // Get the string pointer (literal or variable)
        let string_ptr = match &arg.kind {
            ExprKind::StringLiteral(s) => self
                .builder
                .build_global_string_ptr(s, "panic_str")
                .map_err(|e| CodegenError::internal_string_ptr_failed(&e.to_string(), arg.span))?
                .as_pointer_value(),
            ExprKind::Identifier(name) => {
                let binding = self
                    .lookup_variable(name)
                    .ok_or_else(|| CodegenError::internal_variable_not_found(name, arg.span))?;

                self.load_and_extract_pointer_value(binding.alloca(), name, "panic load", arg.span)?
            }
            ExprKind::IfExpr { .. } => match self.generate_expr_value(arg, &Type::String)? {
                BasicValueEnum::PointerValue(v) => v,
                _ => {
                    return Err(CodegenError::internal_panic_invalid_arg(arg.span));
                }
            },
            _ => {
                return Err(CodegenError::internal_panic_invalid_arg(arg.span));
            }
        };

        // Call lak_panic
        let lak_panic = self
            .module
            .get_function("lak_panic")
            .ok_or_else(|| CodegenError::internal_builtin_not_found("lak_panic"))?;

        self.builder
            .build_call(
                lak_panic,
                &[BasicMetadataValueEnum::PointerValue(string_ptr)],
                "",
            )
            .map_err(|e| CodegenError::internal_panic_call_failed(&e.to_string(), span))?;

        // Insert unreachable instruction
        // This tells LLVM that execution never reaches past this point
        self.builder
            .build_unreachable()
            .map_err(|e| CodegenError::internal_unreachable_failed(&e.to_string(), span))?;

        Ok(())
    }
}
