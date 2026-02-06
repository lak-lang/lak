//! Built-in function code generation.
//!
//! This module implements code generation for Lak's built-in functions,
//! such as `println`.

use super::Codegen;
use super::error::{CodegenError, CodegenErrorKind};
use crate::ast::{Expr, ExprKind, Type};
use crate::token::Span;
use inkwell::AddressSpace;
use inkwell::module::Linkage;
use inkwell::values::BasicMetadataValueEnum;

impl<'ctx> Codegen<'ctx> {
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

    /// Returns the type of an expression for println dispatch.
    ///
    /// This is used to determine which println runtime function to call.
    /// The type dispatch is compile-time: each supported type maps to a dedicated
    /// runtime function (`lak_println`, `lak_println_i32`, `lak_println_i64`).
    ///
    /// Type mapping:
    /// - `IntLiteral` → `Type::I64` (bare integer literals always use i64)
    /// - `StringLiteral` → `Type::String`
    /// - `Identifier` → the variable's declared type (may be i32, i64, or string)
    ///
    /// Note: Integer literals passed directly to println are always treated as i64.
    /// To print an i32, assign the literal to an i32 variable first.
    ///
    /// # Returns
    ///
    /// - `Ok(Type)` - The resolved type for supported expressions
    /// - `Err(CodegenError)` - An internal error for unsupported expressions
    pub(super) fn get_expr_type(&self, expr: &Expr) -> Result<Type, CodegenError> {
        match &expr.kind {
            ExprKind::IntLiteral(_) => Ok(Type::I64),
            ExprKind::StringLiteral(_) => Ok(Type::String),
            ExprKind::Identifier(name) => self
                .variables
                .get(name)
                .map(|b| b.ty().clone())
                .ok_or_else(|| {
                    CodegenError::new(
                        CodegenErrorKind::InternalError,
                        format!(
                            "Internal error: undefined variable '{}' in codegen. \
                             Semantic analysis should have caught this. This is a compiler bug.",
                            name
                        ),
                        expr.span,
                    )
                }),
            ExprKind::Call { callee, .. } => Err(CodegenError::new(
                CodegenErrorKind::InternalError,
                format!(
                    "Internal error: function call '{}()' cannot be used as println argument. \
                     Semantic analysis should have caught this. This is a compiler bug.",
                    callee
                ),
                expr.span,
            )),
        }
    }

    /// Generates LLVM IR for a `println` call.
    ///
    /// Implements `println(value)` by calling the appropriate Lak runtime function
    /// based on the argument type. Type dispatch is performed via `get_expr_type()`,
    /// which determines the type from the expression kind or variable declaration.
    ///
    /// Type dispatch:
    /// - `string` → `lak_println` (string literals or string variables)
    /// - `i32` → `lak_println_i32` (i32 variables only)
    /// - `i64` → `lak_println_i64` (integer literals or i64 variables)
    ///
    /// # Validation responsibilities
    ///
    /// - **Semantic analysis**: Validates argument count (exactly 1) and that variables
    ///   exist. Does NOT validate argument types for println.
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
            return Err(CodegenError::new(
                CodegenErrorKind::InternalError,
                format!(
                    "Internal error: println expects 1 argument, but got {} in codegen. \
                     Semantic analysis should have caught this. This is a compiler bug.",
                    args.len()
                ),
                span,
            ));
        }

        let arg = &args[0];

        // Determine the type of the argument and call the appropriate runtime function
        let arg_type = self.get_expr_type(arg)?;

        match arg_type {
            Type::String => self.generate_println_string(arg, span),
            Type::I32 => self.generate_println_i32(arg, span),
            Type::I64 => self.generate_println_i64(arg, span),
        }
    }

    /// Generates LLVM IR for `println` with a string argument.
    fn generate_println_string(&mut self, arg: &Expr, span: Span) -> Result<(), CodegenError> {
        let string_ptr = match &arg.kind {
            ExprKind::StringLiteral(s) => self
                .builder
                .build_global_string_ptr(s, "str")
                .map_err(|e| {
                    CodegenError::new(
                        CodegenErrorKind::InternalError,
                        format!(
                            "Internal error: failed to create string literal. This is a compiler bug: {}",
                            e
                        ),
                        arg.span,
                    )
                })?
                .as_pointer_value(),
            ExprKind::Identifier(name) => {
                let binding = self.variables.get(name).ok_or_else(|| {
                    CodegenError::new(
                        CodegenErrorKind::InternalError,
                        format!(
                            "Internal error: undefined variable '{}' in codegen. \
                             Semantic analysis should have caught this. This is a compiler bug.",
                            name
                        ),
                        arg.span,
                    )
                })?;

                let ptr_type = self.context.ptr_type(AddressSpace::default());
                self.builder
                    .build_load(ptr_type, binding.alloca(), &format!("{}_load", name))
                    .map_err(|e| {
                        CodegenError::new(
                            CodegenErrorKind::InternalError,
                            format!(
                                "Internal error: failed to load string variable '{}'. This is a compiler bug: {}",
                                name, e
                            ),
                            arg.span,
                        )
                    })?
                    .into_pointer_value()
            }
            _ => {
                return Err(CodegenError::new(
                    CodegenErrorKind::InternalError,
                    "Internal error: println string argument is not a string literal or string variable. \
                     This is a compiler bug.",
                    arg.span,
                ));
            }
        };

        let lak_println = self.module.get_function("lak_println").ok_or_else(|| {
            CodegenError::without_span(
                CodegenErrorKind::InternalError,
                "Internal error: lak_println function not found. This is a compiler bug.",
            )
        })?;

        self.builder
            .build_call(
                lak_println,
                &[BasicMetadataValueEnum::PointerValue(string_ptr)],
                "",
            )
            .map_err(|e| {
                CodegenError::new(
                    CodegenErrorKind::InternalError,
                    format!(
                        "Internal error: failed to generate println call. This is a compiler bug: {}",
                        e
                    ),
                    span,
                )
            })?;

        Ok(())
    }

    /// Generates LLVM IR for `println` with an i32 argument.
    ///
    /// Note: Only i32 variables reach this function. Integer literals are always
    /// treated as i64 by `get_expr_type()` and handled by `generate_println_i64()`.
    fn generate_println_i32(&mut self, arg: &Expr, span: Span) -> Result<(), CodegenError> {
        let i32_value = match &arg.kind {
            ExprKind::Identifier(name) => {
                let binding = self.variables.get(name).ok_or_else(|| {
                    CodegenError::new(
                        CodegenErrorKind::InternalError,
                        format!(
                            "Internal error: undefined variable '{}' in codegen. \
                             Semantic analysis should have caught this. This is a compiler bug.",
                            name
                        ),
                        arg.span,
                    )
                })?;

                // Defensive type check: verify the variable is actually i32
                if binding.ty() != &Type::I32 {
                    return Err(CodegenError::new(
                        CodegenErrorKind::InternalError,
                        format!(
                            "Internal error: variable '{}' has type '{}' but was expected to be i32. \
                             This indicates a bug in type inference or get_expr_type(). This is a compiler bug.",
                            name,
                            binding.ty()
                        ),
                        arg.span,
                    ));
                }

                let i32_type = self.context.i32_type();
                self.builder
                    .build_load(i32_type, binding.alloca(), &format!("{}_load", name))
                    .map_err(|e| {
                        CodegenError::new(
                            CodegenErrorKind::InternalError,
                            format!(
                                "Internal error: failed to load i32 variable '{}'. This is a compiler bug: {}",
                                name, e
                            ),
                            arg.span,
                        )
                    })?
                    .into_int_value()
            }
            // This branch is currently unreachable: integer literals are always typed as i64
            // by get_expr_type() and routed to generate_println_i64().
            _ => {
                return Err(CodegenError::new(
                    CodegenErrorKind::InternalError,
                    "Internal error: generate_println_i32 received a non-identifier expression. \
                     Integer literals should be routed to generate_println_i64. This is a compiler bug.",
                    arg.span,
                ));
            }
        };

        let lak_println_i32 = self.module.get_function("lak_println_i32").ok_or_else(|| {
            CodegenError::without_span(
                CodegenErrorKind::InternalError,
                "Internal error: lak_println_i32 function not found. This is a compiler bug.",
            )
        })?;

        self.builder
            .build_call(
                lak_println_i32,
                &[BasicMetadataValueEnum::IntValue(i32_value)],
                "",
            )
            .map_err(|e| {
                CodegenError::new(
                    CodegenErrorKind::InternalError,
                    format!(
                        "Internal error: failed to generate println_i32 call. This is a compiler bug: {}",
                        e
                    ),
                    span,
                )
            })?;

        Ok(())
    }

    /// Generates LLVM IR for `println` with an i64 argument.
    ///
    /// This handles both integer literals (which are always treated as i64) and
    /// i64 variables.
    fn generate_println_i64(&mut self, arg: &Expr, span: Span) -> Result<(), CodegenError> {
        let i64_value = match &arg.kind {
            ExprKind::IntLiteral(value) => self.context.i64_type().const_int(*value as u64, true),
            ExprKind::Identifier(name) => {
                let binding = self.variables.get(name).ok_or_else(|| {
                    CodegenError::new(
                        CodegenErrorKind::InternalError,
                        format!(
                            "Internal error: undefined variable '{}' in codegen. \
                             Semantic analysis should have caught this. This is a compiler bug.",
                            name
                        ),
                        arg.span,
                    )
                })?;

                // Defensive type check: verify the variable is actually i64
                if binding.ty() != &Type::I64 {
                    return Err(CodegenError::new(
                        CodegenErrorKind::InternalError,
                        format!(
                            "Internal error: variable '{}' has type '{}' but was expected to be i64. \
                             This indicates a bug in type inference or get_expr_type(). This is a compiler bug.",
                            name,
                            binding.ty()
                        ),
                        arg.span,
                    ));
                }

                let i64_type = self.context.i64_type();
                self.builder
                    .build_load(i64_type, binding.alloca(), &format!("{}_load", name))
                    .map_err(|e| {
                        CodegenError::new(
                            CodegenErrorKind::InternalError,
                            format!(
                                "Internal error: failed to load i64 variable '{}'. This is a compiler bug: {}",
                                name, e
                            ),
                            arg.span,
                        )
                    })?
                    .into_int_value()
            }
            _ => {
                return Err(CodegenError::new(
                    CodegenErrorKind::InternalError,
                    "Internal error: println i64 argument is not an integer literal or i64 variable. This is a compiler bug.",
                    arg.span,
                ));
            }
        };

        let lak_println_i64 = self.module.get_function("lak_println_i64").ok_or_else(|| {
            CodegenError::without_span(
                CodegenErrorKind::InternalError,
                "Internal error: lak_println_i64 function not found. This is a compiler bug.",
            )
        })?;

        self.builder
            .build_call(
                lak_println_i64,
                &[BasicMetadataValueEnum::IntValue(i64_value)],
                "",
            )
            .map_err(|e| {
                CodegenError::new(
                    CodegenErrorKind::InternalError,
                    format!(
                        "Internal error: failed to generate println_i64 call. This is a compiler bug: {}",
                        e
                    ),
                    span,
                )
            })?;

        Ok(())
    }
}
