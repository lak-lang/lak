//! Built-in function code generation.
//!
//! This module implements code generation for Lak's built-in functions,
//! such as `println`.

use super::Codegen;
use super::error::{CodegenError, CodegenErrorKind};
use crate::ast::{Expr, ExprKind};
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

    /// Generates LLVM IR for a `println` call.
    ///
    /// Implements `println(string)` by calling the Lak runtime `lak_println` function.
    ///
    /// # Arguments
    ///
    /// * `args` - The arguments passed to `println` (validated by semantic analysis)
    /// * `span` - The source location of the println call
    ///
    /// # Errors
    ///
    /// Returns an internal error if semantic validation was bypassed (wrong argument
    /// count or type). Semantic analysis guarantees println receives exactly one
    /// string literal or string variable.
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

        // Get string pointer from string literal or string variable
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
                    "Internal error: println argument is not a string literal or string variable. \
                     Semantic analysis should have caught this. This is a compiler bug.",
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
}
