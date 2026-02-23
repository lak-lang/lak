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
//! - **Module path errors** (without span): [`non_utf8_path_component()`](CodegenError::non_utf8_path_component),
//!   [`duplicate_mangle_prefix()`](CodegenError::duplicate_mangle_prefix)
//! - **Internal errors** (without span, module path): [`internal_empty_mangle_prefix()`](CodegenError::internal_empty_mangle_prefix)

use std::path::Path;

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
/// during the code generation phase, including module path validation, LLVM IR
/// generation, and object file output.
///
/// # Error Categories
///
/// - **Infrastructure errors** (typically no span): [`InternalError`](Self::InternalError),
///   [`TargetError`](Self::TargetError)
/// - **Module path errors** (no span): [`InvalidModulePath`](Self::InvalidModulePath)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodegenErrorKind {
    /// Internal compiler error (should not happen in normal use).
    InternalError,
    /// LLVM target or code generation infrastructure error.
    TargetError,
    /// Invalid module path (e.g., non-UTF-8 path components, duplicate prefixes).
    InvalidModulePath,
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
#[derive(Debug, Clone)]
pub struct CodegenError {
    /// A human-readable description of the error.
    message: String,
    /// The source location where the error occurred, if available.
    span: Option<Span>,
    /// The kind of error, for structured error handling.
    kind: CodegenErrorKind,
    /// Whether this error already includes unary operation context.
    /// Used by `wrap_in_unary_context()` to prevent double-wrapping
    /// (e.g., avoiding "in unary '-' operation: in unary '-' operation: ...").
    has_unary_context: bool,
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
            has_unary_context: false,
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
            has_unary_context: false,
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

    /// Sets the unary context flag, preventing double-wrapping by `wrap_in_unary_context()`.
    fn with_unary_context(mut self) -> Self {
        self.has_unary_context = true;
        self
    }

    /// Returns a short, human-readable description of the error kind.
    ///
    /// This is used as the report title in error messages, while `message()`
    /// provides the detailed explanation shown in the label.
    pub fn short_message(&self) -> &'static str {
        match self.kind {
            CodegenErrorKind::InternalError => "Internal error",
            CodegenErrorKind::TargetError => "Target error",
            CodegenErrorKind::InvalidModulePath => "Invalid module path",
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

    /// Creates an internal error for break used outside of loops.
    pub fn internal_break_outside_loop(span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            "Internal error: break statement used outside loop during codegen. \
             Semantic analysis should have rejected this. This is a compiler bug.",
            span,
        )
    }

    /// Creates an internal error for continue used outside of loops.
    pub fn internal_continue_outside_loop(span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            "Internal error: continue statement used outside loop during codegen. \
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

    /// Creates an internal error for missing function signature metadata.
    pub fn internal_function_signature_not_found(name: &str, span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: function signature metadata for '{}' not found in codegen. This is a compiler bug.",
                name
            ),
            span,
        )
    }

    /// Creates an internal error for call argument count mismatch.
    pub fn internal_call_arg_count_mismatch(
        callee: &str,
        expected: usize,
        got: usize,
        span: Span,
    ) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: call to '{}' has {} argument(s), but function expects {} in codegen. This is a compiler bug.",
                callee, got, expected
            ),
            span,
        )
    }

    /// Creates an internal error for missing function parameter in LLVM function.
    pub fn internal_function_param_missing(function: &str, index: usize, span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: function '{}' is missing parameter {} in LLVM IR. This is a compiler bug.",
                function, index
            ),
            span,
        )
    }

    /// Creates an internal error for integer used as string.
    pub fn internal_int_as_string(value: i128, span: Span) -> Self {
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
    pub fn internal_int_as_bool(value: i128, span: Span) -> Self {
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

    /// Creates an internal error for float used as non-float type.
    pub fn internal_float_as_type(expected: &str, span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: float literal used as '{}' value in codegen. \
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

    /// Creates an internal error for function call returning void unexpectedly.
    pub fn internal_call_returned_void(callee: &str, span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: function call '{}' returned void in a value context. \
                 Semantic analysis should have caught this. This is a compiler bug.",
                callee
            ),
            span,
        )
    }

    /// Creates an internal error for unsupported function return type.
    pub fn internal_unsupported_function_return_type(return_type: &str, span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: unsupported function return type '{}' in codegen. \
                 Semantic analysis should have rejected this. This is a compiler bug.",
                return_type
            ),
            span,
        )
    }

    /// Creates an internal error for returning a value from a void function.
    pub fn internal_return_value_in_void_function(span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            "Internal error: return value in void function reached codegen. \
             Semantic analysis should have rejected this. This is a compiler bug.",
            span,
        )
    }

    /// Creates an internal error for missing return value in non-void function.
    pub fn internal_missing_return_value(span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            "Internal error: missing return value in non-void function reached codegen. \
             Semantic analysis should have rejected this. This is a compiler bug.",
            span,
        )
    }

    /// Creates an internal error for return with value in main function.
    pub fn internal_main_return_with_value(span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            "Internal error: main return with value reached codegen. \
             Semantic analysis should have rejected this. This is a compiler bug.",
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
                "Internal error: failed to store value for '{}'. \
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

    /// Creates an internal error for invalid println string argument.
    pub fn internal_println_invalid_string_arg(span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            "Internal error: println string argument is not a string literal or string variable. \
             This is a compiler bug.",
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
            "Internal error: panic argument is not a string-producing expression. \
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

    /// Creates an internal error for binary operation dispatched with a non-numeric type.
    pub fn internal_binary_op_string(op: crate::ast::BinaryOperator, span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: binary operator '{}' dispatched with non-numeric type in codegen. \
                 Semantic analysis should have caught this. This is a compiler bug.",
                op
            ),
            span,
        )
    }

    /// Creates an internal error for non-adaptable binary operand types.
    pub fn internal_binary_operand_type_mismatch(
        left_ty: &str,
        right_ty: &str,
        span: Span,
    ) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: binary operands have incompatible types '{}' and '{}' after \
                 literal adaptation in codegen. Semantic analysis should have rejected this. \
                 This is a compiler bug.",
                left_ty, right_ty
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
            "Internal error: no current function when generating runtime check. \
             This is a compiler bug.",
            span,
        )
    }

    /// Creates an internal error for missing variable scope.
    pub fn internal_no_variable_scope(span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            "Internal error: attempted to define variable outside a scope in codegen. \
             This is a compiler bug.",
            span,
        )
    }

    /// Creates an internal error for missing loop control scope.
    pub fn internal_no_loop_control_scope(span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            "Internal error: attempted to pop loop control scope when no loop is active in codegen. \
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

    /// Creates an internal error for failed string equality call.
    pub fn internal_streq_call_failed(error: &str, span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: failed to call lak_streq. This is a compiler bug: {}",
                error
            ),
            span,
        )
    }

    /// Creates an internal error when lak_streq returns a non-IntValue basic value.
    pub fn internal_streq_unexpected_basic_type(span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            "Internal error: lak_streq returned a non-integer basic value, expected i1. \
             This is a compiler bug.",
            span,
        )
    }

    /// Creates an internal error when lak_streq returns void (InstructionValue).
    pub fn internal_streq_returned_void(span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            "Internal error: lak_streq returned void instead of a boolean value. \
             This is a compiler bug.",
            span,
        )
    }

    /// Creates an internal error when a comparison operator is dispatched with a non-bool expected type.
    pub fn internal_comparison_expected_bool(
        op: crate::ast::BinaryOperator,
        expected_ty: &crate::ast::Type,
        span: Span,
    ) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: comparison operator '{}' expected bool type but got '{}'. \
                 Semantic analysis should have caught this. This is a compiler bug.",
                op, expected_ty
            ),
            span,
        )
    }

    /// Creates an internal error when a logical operator is dispatched with a non-bool expected type.
    pub fn internal_logical_expected_bool(
        op: crate::ast::BinaryOperator,
        expected_ty: &crate::ast::Type,
        span: Span,
    ) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: logical operator '{}' expected bool type but got '{}'. \
                 Semantic analysis should have caught this. This is a compiler bug.",
                op, expected_ty
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

    /// Creates an internal error for unary operation dispatched with an invalid type.
    pub fn internal_unary_op_string(op: crate::ast::UnaryOperator, span: Span) -> Self {
        let err = Self::new(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: unary operator '{}' dispatched with non-signed-integer-or-float type in codegen. \
                 Semantic analysis should have caught this. This is a compiler bug.",
                op
            ),
            span,
        );
        err.with_unary_context()
    }

    /// Creates an internal error when logical NOT is dispatched with a non-bool expected type.
    pub fn internal_unary_not_expected_bool(expected_ty: &crate::ast::Type, span: Span) -> Self {
        let err = Self::new(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: unary operator '!' expected bool type but got '{}'. \
                 Semantic analysis should have caught this. This is a compiler bug.",
                expected_ty
            ),
            span,
        );
        err.with_unary_context()
    }

    /// Creates an internal error for LLVM intrinsic not found.
    pub fn internal_intrinsic_not_found(name: &str, span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: LLVM intrinsic '{}' not found. This is a compiler bug.",
                name
            ),
            span,
        )
    }

    /// Creates an internal error for LLVM intrinsic declaration failure.
    pub fn internal_intrinsic_declaration_failed(name: &str, span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: failed to get declaration for LLVM intrinsic '{}'. This is a compiler bug.",
                name
            ),
            span,
        )
    }

    /// Creates an internal error for LLVM intrinsic call failure.
    pub fn internal_intrinsic_call_failed(name: &str, error: &str, span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: failed to call LLVM intrinsic '{}'. This is a compiler bug: {}",
                name, error
            ),
            span,
        )
    }

    /// Creates an internal error for failed extractvalue instruction.
    pub fn internal_extract_value_failed(error: &str, span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: failed to extract value from intrinsic result. This is a compiler bug: {}",
                error
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

    /// Creates an internal error for builtin function not found (with span).
    pub fn internal_builtin_not_found_with_span(name: &str, span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: {} function not found. This is a compiler bug.",
                name
            ),
            span,
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

    /// Creates an internal error for function parameter count mismatch.
    pub fn internal_function_param_count_mismatch(
        function: &str,
        expected: usize,
        actual: usize,
    ) -> Self {
        Self::without_span(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: function '{}' has {} parameter(s) in AST but {} in LLVM declaration. This is a compiler bug.",
                function, expected, actual
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

    /// Creates an internal error for non-void function that falls through.
    pub fn internal_missing_return_in_non_void_function(fn_name: &str, return_type: &str) -> Self {
        Self::without_span(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: function '{}' with return type '{}' reached end without return. Semantic analysis should have rejected this. This is a compiler bug.",
                fn_name, return_type
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

    /// Creates an internal error for member access not implemented.
    pub fn internal_member_access_not_implemented(span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            "Internal error: member access expression reached codegen. \
             Semantic analysis should have rejected this. This is a compiler bug.",
            span,
        )
    }

    /// Creates an internal error for module call used as value.
    pub fn internal_module_call_as_value(module: &str, function: &str, span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: module call '{}.{}()' used as value in codegen. \
                 Semantic analysis should have caught this. This is a compiler bug.",
                module, function
            ),
            span,
        )
    }

    // =========================================================================
    // Module compilation internal errors
    // =========================================================================

    /// Creates an internal error for entry module not found in module list.
    pub fn internal_entry_module_not_found(entry_path: &Path) -> Self {
        Self::without_span(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: entry module '{}' not found in module list. This is a compiler bug.",
                entry_path.display()
            ),
        )
    }

    /// Creates an internal error for import path not found in resolved imports.
    pub fn internal_import_path_not_resolved(import_path: &str, span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: import path '{}' not found in resolved imports. \
                 This is a compiler bug.",
                import_path
            ),
            span,
        )
    }

    /// Creates an internal error for resolved module not found for a canonical path.
    pub fn internal_resolved_module_not_found_for_path(canonical_path: &Path, span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: resolved module not found for path '{}'. \
                 This is a compiler bug.",
                canonical_path.display()
            ),
            span,
        )
    }

    /// Creates an internal error for module alias not found in alias map.
    pub fn internal_module_alias_not_found(alias_or_name: &str, span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: module alias '{}' not found in alias map. \
                 This is a compiler bug.",
                alias_or_name
            ),
            span,
        )
    }

    /// Creates an internal error for mangle prefix not found for a module path.
    pub fn internal_mangle_prefix_not_found(module_path: &Path) -> Self {
        Self::without_span(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: mangle prefix not found for module '{}'. This is a compiler bug.",
                module_path.display()
            ),
        )
    }

    /// Creates an internal error when entry path has no parent directory.
    ///
    /// The entry path is expected to be a file path with at least one directory
    /// component. A path without a parent indicates a bug in path resolution
    /// before code generation.
    pub fn internal_entry_path_no_parent(entry_path: &Path) -> Self {
        Self::without_span(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: entry path '{}' has no parent directory. \
                 This is a compiler bug.",
                entry_path.display()
            ),
        )
    }

    /// Creates an internal error for non-canonical path reaching codegen.
    ///
    /// Module paths must be canonicalized by the resolver before reaching
    /// code generation. The presence of `.` or `..` components indicates
    /// a compiler bug.
    pub fn internal_non_canonical_path(module_path: &Path) -> Self {
        Self::without_span(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: module path '{}' contains '.' or '..' components. \
                 Paths must be canonicalized before code generation. This is a compiler bug.",
                module_path.display()
            ),
        )
    }

    /// Creates an internal error when a module path produces an empty mangle prefix.
    ///
    /// An empty prefix means the module path has no Normal components (e.g., `/`
    /// on Unix). This indicates a bug in the resolver, as such a path should never
    /// reach code generation.
    pub fn internal_empty_mangle_prefix(module_path: &Path) -> Self {
        Self::without_span(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: module path '{}' produces an empty mangle prefix. This is a compiler bug.",
                module_path.display()
            ),
        )
    }

    // =========================================================================
    // Module path errors (without span)
    // =========================================================================

    /// Creates an error for non-UTF-8 path component in module path.
    pub fn non_utf8_path_component(module_path: &Path) -> Self {
        Self::without_span(
            CodegenErrorKind::InvalidModulePath,
            format!(
                "Module path '{}' contains a non-UTF-8 component.",
                module_path.display()
            ),
        )
    }

    /// Creates an error when two module paths produce the same mangle prefix.
    ///
    /// The paths are sorted alphabetically to ensure deterministic error messages
    /// regardless of the order in which the paths are passed.
    pub fn duplicate_mangle_prefix(prefix: &str, path1: &Path, path2: &Path) -> Self {
        let (first, second) = if path1 <= path2 {
            (path1, path2)
        } else {
            (path2, path1)
        };
        Self::without_span(
            CodegenErrorKind::InvalidModulePath,
            format!(
                "Modules '{}' and '{}' produce the same mangle prefix '{}'. \
                 Rename one of the modules to avoid the collision.",
                first.display(),
                second.display(),
                prefix
            ),
        )
    }

    /// Creates an internal error for failed variable allocation.
    pub fn internal_variable_alloca_failed(name: &str, error: &str, span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: failed to allocate variable '{}'. This is a compiler bug: {}",
                name, error
            ),
            span,
        )
    }

    /// Creates an internal error for non-integer value where integer was expected.
    pub fn internal_non_integer_value(operation: &str, span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: expected integer value in {} operation, but got non-integer. \
                 Semantic analysis should have caught this. This is a compiler bug.",
                operation
            ),
            span,
        )
    }

    /// Creates an internal error for non-float value where float was expected.
    pub fn internal_non_float_value(operation: &str, span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: expected float value in {} operation, but got non-float. \
                 Semantic analysis should have caught this. This is a compiler bug.",
                operation
            ),
            span,
        )
    }

    /// Creates an internal error for non-pointer value where pointer was expected.
    pub fn internal_non_pointer_value(operation: &str, span: Span) -> Self {
        Self::new(
            CodegenErrorKind::InternalError,
            format!(
                "Internal error: expected pointer value in {} operation, but got non-pointer. \
                 Semantic analysis should have caught this. This is a compiler bug.",
                operation
            ),
            span,
        )
    }

    /// Wraps an error with unary operation context.
    ///
    /// If the error already has unary context, a new error is created that
    /// preserves the original kind, message, and span (without adding
    /// additional wrapping context).
    /// Otherwise, a new error is created with unary context prepended.
    /// Note: This method does not propagate help text (CodegenError has no help field).
    pub fn wrap_in_unary_context(
        base_error: &Self,
        op: crate::ast::UnaryOperator,
        span: Span,
    ) -> Self {
        if base_error.has_unary_context {
            base_error.clone()
        } else {
            let message = format!("in unary '{}' operation: {}", op, base_error.message());
            Self::new(
                base_error.kind(),
                message,
                base_error.span().unwrap_or(span),
            )
            .with_unary_context()
        }
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
