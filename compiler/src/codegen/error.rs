//! Code generation error types.
//!
//! This module defines [`CodegenError`], which represents errors that can occur
//! during LLVM code generation.
//!
//! # Helper Constructors
//!
//! This module provides specialized constructor methods for common error cases,
//! ensuring consistent error messaging across the compiler. Prefer using these
//! helpers over constructing errors manually with [`CodegenError::new()`] or
//! [`CodegenError::without_span()`].
//!
//! Available helper methods are organized by category:
//! - **Target errors** (without span): [`target_init_failed()`](CodegenError::target_init_failed),
//!   [`target_from_triple_failed()`](CodegenError::target_from_triple_failed),
//!   [`target_machine_creation_failed()`](CodegenError::target_machine_creation_failed), etc.
//! - **Internal errors** (with span): [`internal_variable_not_found()`](CodegenError::internal_variable_not_found),
//!   [`internal_function_not_found()`](CodegenError::internal_function_not_found),
//!   [`internal_println_arg_count()`](CodegenError::internal_println_arg_count), etc.
//! - **Internal errors** (without span): [`internal_builtin_not_found()`](CodegenError::internal_builtin_not_found),
//!   [`internal_return_build_failed()`](CodegenError::internal_return_build_failed), etc.

use crate::token::Span;

/// The kind of code generation error.
///
/// This enum allows error handling code to match on error types structurally
/// rather than relying on string matching, which is fragile.
///
/// # Note
///
/// Semantic errors (undefined variables, type mismatches, etc.) are detected
/// during semantic analysis. This enum only contains errors that can occur
/// during LLVM IR generation or object file output.
///
/// # Error Categories
///
/// - **Infrastructure errors** (typically no span): [`InternalError`](Self::InternalError),
///   [`TargetError`](Self::TargetError)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodegenErrorKind {
    /// Internal compiler error (should not happen in normal use).
    InternalError,
    /// LLVM target or code generation infrastructure error.
    TargetError,
}

/// An error that occurred during code generation.
///
/// Contains a human-readable message and optionally the source location
/// where the error occurred, enabling rich error reporting.
///
/// # Construction
///
/// Use the appropriate constructor based on your error type:
///
/// - [`new()`](Self::new) - For errors with a specific source location
/// - [`without_span()`](Self::without_span) - For errors without a source location
///   (infrastructure errors)
#[derive(Debug)]
pub struct CodegenError {
    /// A human-readable description of the error.
    message: String,
    /// The source location where the error occurred, if available.
    span: Option<Span>,
    /// The kind of error, for structured error handling.
    kind: CodegenErrorKind,
}

impl CodegenError {
    /// Creates a new error with a source location.
    ///
    /// Use this for internal errors that can be traced to a specific location in
    /// the source code (e.g., LLVM instruction build failures).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// CodegenError::new(
    ///     CodegenErrorKind::InternalError,
    ///     format!("Internal error: failed to build instruction: {}", e),
    ///     stmt.span,
    /// )
    /// ```
    pub fn new(kind: CodegenErrorKind, message: impl Into<String>, span: Span) -> Self {
        CodegenError {
            message: message.into(),
            span: Some(span),
            kind,
        }
    }

    /// Creates a new error without a source location.
    ///
    /// Use this for infrastructure errors or errors that cannot be traced to
    /// a specific location (e.g., LLVM internal errors, target errors).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// CodegenError::without_span(
    ///     CodegenErrorKind::InternalError,
    ///     format!("Internal error: failed to build instruction: {}", e),
    /// )
    /// ```
    pub fn without_span(kind: CodegenErrorKind, message: impl Into<String>) -> Self {
        CodegenError {
            message: message.into(),
            span: None,
            kind,
        }
    }

    /// Returns the error message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the source location where the error occurred, if available.
    pub fn span(&self) -> Option<Span> {
        self.span
    }

    /// Returns the kind of error.
    pub fn kind(&self) -> CodegenErrorKind {
        self.kind
    }

    /// Returns a short, human-readable description of the error kind.
    ///
    /// This is used as the report title in error messages, while `message()`
    /// provides the detailed explanation shown in the label.
    pub fn short_message(&self) -> &'static str {
        match self.kind {
            CodegenErrorKind::InternalError => "Internal error",
            CodegenErrorKind::TargetError => "Target error",
        }
    }

    // =========================================================================
    // Target errors (without span)
    // =========================================================================

    /// Creates a target initialization error.
    pub fn target_init_failed(error: &str) -> Self {
        Self::without_span(
            CodegenErrorKind::TargetError,
            format!("Failed to initialize native target: {}", error),
        )
    }

    /// Creates a target from triple error.
    pub fn target_from_triple_failed(triple: &str, error: &str) -> Self {
        Self::without_span(
            CodegenErrorKind::TargetError,
            format!("Failed to get target for triple '{}': {}", triple, error),
        )
    }

    /// Creates a CPU name invalid UTF-8 error.
    pub fn target_cpu_invalid_utf8() -> Self {
        Self::without_span(
            CodegenErrorKind::TargetError,
            "CPU name contains invalid UTF-8",
        )
    }

    /// Creates a CPU features invalid UTF-8 error.
    pub fn target_features_invalid_utf8() -> Self {
        Self::without_span(
            CodegenErrorKind::TargetError,
            "CPU features contain invalid UTF-8",
        )
    }

    /// Creates a target machine creation error.
    pub fn target_machine_creation_failed(triple: &str, cpu: &str) -> Self {
        Self::without_span(
            CodegenErrorKind::TargetError,
            format!(
                "Failed to create target machine for triple '{}', CPU '{}'. \
                 This may indicate an unsupported platform or LLVM configuration issue.",
                triple, cpu
            ),
        )
    }

    /// Creates a target write error.
    pub fn target_write_failed(path: &std::path::Path, error: &str) -> Self {
        Self::without_span(
            CodegenErrorKind::TargetError,
            format!(
                "Failed to write object file to '{}': {}",
                path.display(),
                error
            ),
        )
    }

    // =========================================================================
    // Internal errors with span
    // =========================================================================

    /// Creates an internal error for invalid expression statement.
    pub fn internal_invalid_expr_stmt(span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            "Internal error: invalid expression statement in codegen. \
             Semantic analysis should have rejected this. This is a compiler bug.",
            span,
        )
    }

    /// Creates an internal error for undefined function.
    pub fn internal_function_not_found(name: &str, span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: undefined function '{}' in codegen. \
                 Semantic analysis should have caught this. This is a compiler bug.",
                name
            ),
            span,
        )
    }

    /// Creates an internal error for failed function call.
    pub fn internal_call_failed(callee: &str, error: &str, span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: failed to generate call to '{}'. This is a compiler bug: {}",
                callee, error
            ),
            span,
        )
    }

    /// Creates an internal error for integer used as string.
    pub fn internal_int_as_string(value: i64, span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: integer literal {} used as 'string' value in codegen. \
                 Semantic analysis should have caught this. This is a compiler bug.",
                value
            ),
            span,
        )
    }

    /// Creates an internal error for integer used as bool.
    pub fn internal_int_as_bool(value: i64, span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: integer literal {} used as 'bool' value in codegen. \
                 Semantic analysis should have caught this. This is a compiler bug.",
                value
            ),
            span,
        )
    }

    /// Creates an internal error for bool used as non-bool type.
    pub fn internal_bool_as_type(expected: &str, span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: boolean literal used as '{}' value in codegen. \
                 Semantic analysis should have caught this. This is a compiler bug.",
                expected
            ),
            span,
        )
    }

    /// Creates an internal error for undefined variable.
    pub fn internal_variable_not_found(name: &str, span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: undefined variable '{}' in codegen. \
                 Semantic analysis should have caught this. This is a compiler bug.",
                name
            ),
            span,
        )
    }

    /// Creates an internal error for variable type mismatch.
    pub fn internal_variable_type_mismatch(
        name: &str,
        expected: &str,
        actual: &str,
        span: Span,
    ) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: type mismatch for variable '{}' in codegen. \
                 Expected '{}', but variable has type '{}'. \
                 Semantic analysis should have caught this. This is a compiler bug.",
                name, expected, actual
            ),
            span,
        )
    }

    /// Creates an internal error for failed variable load.
    pub fn internal_variable_load_failed(name: &str, error: &str, span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: failed to load variable '{}'. This is a compiler bug: {}",
                name, error
            ),
            span,
        )
    }

    /// Creates an internal error for string used as non-string type.
    pub fn internal_string_as_type(expected: &str, span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: string literal used as '{}' value in codegen. \
                 Semantic analysis should have caught this. This is a compiler bug.",
                expected
            ),
            span,
        )
    }

    /// Creates an internal error for failed string pointer creation.
    pub fn internal_string_ptr_failed(error: &str, span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: failed to create string literal. This is a compiler bug: {}",
                error
            ),
            span,
        )
    }

    /// Creates an internal error for function call used as value.
    pub fn internal_call_as_value(callee: &str, span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: function call '{}' used as value in codegen. \
                 Semantic analysis should have caught this. This is a compiler bug.",
                callee
            ),
            span,
        )
    }

    /// Creates an internal error for duplicate variable.
    pub fn internal_duplicate_variable(name: &str, span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: duplicate variable '{}' in codegen. \
                 Semantic analysis should have caught this. This is a compiler bug.",
                name
            ),
            span,
        )
    }

    /// Creates an internal error for failed variable store.
    pub fn internal_variable_store_failed(name: &str, error: &str, span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: failed to store initial value for '{}'. \
                 This is a compiler bug: {}",
                name, error
            ),
            span,
        )
    }

    /// Creates an internal error for println argument count mismatch.
    pub fn internal_println_arg_count(count: usize, span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: println expects 1 argument, but got {} in codegen. \
                 Semantic analysis should have caught this. This is a compiler bug.",
                count
            ),
            span,
        )
    }

    /// Creates an internal error for println with function call argument.
    pub fn internal_println_call_arg(callee: &str, span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: function call '{}()' cannot be used as println argument. \
                 Semantic analysis should have caught this. This is a compiler bug.",
                callee
            ),
            span,
        )
    }

    /// Creates an internal error for invalid println string argument.
    pub fn internal_println_invalid_string_arg(span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            "Internal error: println string argument is not a string literal or string variable. \
             This is a compiler bug.",
            span,
        )
    }

    /// Creates an internal error for invalid println i32 argument.
    pub fn internal_println_invalid_i32_arg(span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            "Internal error: generate_println_i32 received a non-identifier expression. \
             Integer literals should be routed to generate_println_i64. This is a compiler bug.",
            span,
        )
    }

    /// Creates an internal error for invalid println i64 argument.
    pub fn internal_println_invalid_i64_arg(span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            "Internal error: println i64 argument is not an integer literal or i64 variable. This is a compiler bug.",
            span,
        )
    }

    /// Creates an internal error for invalid println bool argument.
    pub fn internal_println_invalid_bool_arg(span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            "Internal error: println bool argument is not a boolean literal or bool variable. This is a compiler bug.",
            span,
        )
    }

    /// Creates an internal error for variable type mismatch in println.
    pub fn internal_println_type_mismatch(
        name: &str,
        expected: &str,
        actual: &str,
        span: Span,
    ) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: variable '{}' has type '{}' but was expected to be {}. \
                 This indicates a bug in type inference or get_expr_type(). This is a compiler bug.",
                name, actual, expected
            ),
            span,
        )
    }

    /// Creates an internal error for failed println call.
    pub fn internal_println_call_failed(error: &str, span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: failed to generate println call. This is a compiler bug: {}",
                error
            ),
            span,
        )
    }

    /// Creates an internal error for panic argument count mismatch.
    pub fn internal_panic_arg_count(count: usize, span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: panic expects 1 argument, but got {} in codegen. \
                 Semantic analysis should have caught this. This is a compiler bug.",
                count
            ),
            span,
        )
    }

    /// Creates an internal error for invalid panic argument.
    pub fn internal_panic_invalid_arg(span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            "Internal error: panic argument is not a string literal or string variable. \
             Semantic analysis should have caught this. This is a compiler bug.",
            span,
        )
    }

    /// Creates an internal error for failed panic call.
    pub fn internal_panic_call_failed(error: &str, span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: failed to generate panic call. This is a compiler bug: {}",
                error
            ),
            span,
        )
    }

    /// Creates an internal error for failed unreachable instruction.
    pub fn internal_unreachable_failed(error: &str, span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: failed to build unreachable instruction. This is a compiler bug: {}",
                error
            ),
            span,
        )
    }

    /// Creates an internal error for binary operation on string type.
    pub fn internal_binary_op_string(op: crate::ast::BinaryOperator, span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: binary operator '{}' applied to string type in codegen. \
                 Semantic analysis should have caught this. This is a compiler bug.",
                op
            ),
            span,
        )
    }

    /// Creates an internal error for failed binary operation.
    pub fn internal_binary_op_failed(
        op: crate::ast::BinaryOperator,
        error: &str,
        span: Span,
    ) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: failed to generate '{}' instruction. This is a compiler bug: {}",
                op, error
            ),
            span,
        )
    }

    /// Creates an internal error for no current function.
    pub fn internal_no_current_function(span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            "Internal error: no current function when generating division zero check. \
             This is a compiler bug.",
            span,
        )
    }

    /// Creates an internal error for failed comparison.
    pub fn internal_compare_failed(error: &str, span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: failed to generate comparison instruction. This is a compiler bug: {}",
                error
            ),
            span,
        )
    }

    /// Creates an internal error for failed branch.
    pub fn internal_branch_failed(error: &str, span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: failed to generate branch instruction. This is a compiler bug: {}",
                error
            ),
            span,
        )
    }

    /// Creates an internal error for unary operation on string type.
    pub fn internal_unary_op_string(op: crate::ast::UnaryOperator, span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: unary operator '{}' applied to string type in codegen. \
                 Semantic analysis should have caught this. This is a compiler bug.",
                op
            ),
            span,
        )
    }

    /// Creates an internal error for failed unary operation.
    pub fn internal_unary_op_failed(
        op: crate::ast::UnaryOperator,
        error: &str,
        span: Span,
    ) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: failed to generate unary '{}' instruction. This is a compiler bug: {}",
                op, error
            ),
            span,
        )
    }

    // =========================================================================
    // Internal errors without span
    // =========================================================================

    /// Creates an internal error for builtin function not found.
    pub fn internal_builtin_not_found(name: &str) -> Self {
        Self::without_span(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: {} function not found. This is a compiler bug.",
                name
            ),
        )
    }

    /// Creates an internal error for function not found (no span).
    pub fn internal_function_not_found_no_span(name: &str) -> Self {
        Self::without_span(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: function '{}' not found in module. This is a compiler bug.",
                name
            ),
        )
    }

    /// Creates an internal error for failed return build.
    pub fn internal_return_build_failed(fn_name: &str, error: &str) -> Self {
        Self::without_span(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: failed to build return for function '{}'. This is a compiler bug: {}",
                fn_name, error
            ),
        )
    }

    /// Creates an internal error for failed return build in main.
    pub fn internal_main_return_build_failed(error: &str) -> Self {
        Self::without_span(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: failed to build return instruction. This is a compiler bug: {}",
                error
            ),
        )
    }
}

impl std::fmt::Display for CodegenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(span) = &self.span {
            write!(f, "{}:{}: {}", span.line, span.column, self.message)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

impl std::error::Error for CodegenError {}
