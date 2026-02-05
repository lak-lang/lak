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
    /// Returns an error if LLVM operations fail (internal errors).
    ///
    /// # Panics
    ///
    /// Panics if semantic validation was bypassed (wrong argument count or type).
    /// Semantic analysis guarantees println receives exactly one string literal.
    pub(super) fn generate_println(
        &mut self,
        args: &[Expr],
        span: Span,
    ) -> Result<(), CodegenError> {
        // Semantic analysis guarantees exactly one argument
        debug_assert!(
            args.len() == 1,
            "println argument count should have been validated by semantic analysis"
        );

        let arg = &args[0];
        let string_value = match &arg.kind {
            ExprKind::StringLiteral(s) => s,
            _ => {
                panic!("println argument type should have been validated by semantic analysis");
            }
        };

        let lak_println = self.module.get_function("lak_println").ok_or_else(|| {
            CodegenError::without_span(
                CodegenErrorKind::InternalError,
                "Internal error: lak_println function not found. This is a compiler bug.",
            )
        })?;

        let str_value = self
            .builder
            .build_global_string_ptr(string_value, "str")
            .map_err(|e| {
                CodegenError::new(
                    CodegenErrorKind::InternalError,
                    format!(
                        "Internal error: failed to create string literal. This is a compiler bug: {}",
                        e
                    ),
                    arg.span,
                )
            })?;

        self.builder
            .build_call(
                lak_println,
                &[BasicMetadataValueEnum::PointerValue(
                    str_value.as_pointer_value(),
                )],
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
