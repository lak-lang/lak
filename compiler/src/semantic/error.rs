//! Semantic analysis error types.
//!
//! This module defines [`SemanticError`], which represents errors that can occur
//! during semantic analysis (name resolution, type checking, etc.).
//!
//! # Helper Constructors
//!
//! This module provides specialized constructor methods for common error cases,
//! ensuring consistent error messaging across the compiler. Prefer using these
//! helpers over constructing errors manually with [`SemanticError::new()`].
//!
//! Available helper methods are organized by category:
//! - **Name resolution**: [`undefined_variable()`](SemanticError::undefined_variable),
//!   [`undefined_function()`](SemanticError::undefined_function),
//!   [`duplicate_variable()`](SemanticError::duplicate_variable),
//!   [`duplicate_function()`](SemanticError::duplicate_function)
//! - **Type errors**: [`type_mismatch_int_to_string()`](SemanticError::type_mismatch_int_to_string),
//!   [`type_mismatch_variable()`](SemanticError::type_mismatch_variable), etc.
//! - **Argument errors**: [`invalid_argument_println_count()`](SemanticError::invalid_argument_println_count), etc.
//! - **Expression errors**: [`invalid_expression_string_literal()`](SemanticError::invalid_expression_string_literal), etc.
//! - **Structural errors**: [`invalid_main_signature()`](SemanticError::invalid_main_signature)
//! - **Internal errors**: [`internal_check_integer_range_string()`](SemanticError::internal_check_integer_range_string), etc.

use crate::token::Span;

/// The kind of semantic analysis error.
///
/// This enum allows error handling code to match on error types structurally
/// rather than relying on string matching, which is fragile.
///
/// # Error Categories
///
/// Error kinds fall into four categories based on their typical span behavior:
///
/// - **Name resolution errors** (have span): [`DuplicateFunction`](Self::DuplicateFunction),
///   [`DuplicateVariable`](Self::DuplicateVariable), [`UndefinedVariable`](Self::UndefinedVariable),
///   [`UndefinedFunction`](Self::UndefinedFunction)
/// - **Type errors** (have span): [`TypeMismatch`](Self::TypeMismatch),
///   [`IntegerOverflow`](Self::IntegerOverflow), [`InvalidArgument`](Self::InvalidArgument),
///   [`InvalidExpression`](Self::InvalidExpression)
/// - **Structural errors**: [`MissingMainFunction`](Self::MissingMainFunction) (no span),
///   [`InvalidMainSignature`](Self::InvalidMainSignature) (has span pointing to return type)
/// - **Internal errors** (have span): [`InternalError`](Self::InternalError) - indicates a compiler bug
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticErrorKind {
    /// A function was defined multiple times.
    DuplicateFunction,
    /// A variable was defined multiple times in the same scope.
    DuplicateVariable,
    /// A variable was referenced but not defined.
    UndefinedVariable,
    /// A function was called but not defined.
    UndefinedFunction,
    /// Type mismatch between expected and actual types.
    TypeMismatch,
    /// Integer value is out of range for the target type.
    IntegerOverflow,
    /// Invalid function arguments (wrong count or type).
    InvalidArgument,
    /// Expression used in an invalid context (e.g., literal as statement).
    InvalidExpression,
    /// No main function was found in the program.
    MissingMainFunction,
    /// The main function has an invalid signature (e.g., wrong return type).
    InvalidMainSignature,
    /// Internal compiler error (should never occur in normal operation).
    InternalError,
}

/// An error that occurred during semantic analysis.
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
/// - [`missing_main()`](Self::missing_main) - Convenience for missing main function errors
#[derive(Debug)]
pub struct SemanticError {
    /// A human-readable description of the error.
    message: String,
    /// The source location where the error occurred, if available.
    span: Option<Span>,
    /// The kind of error, for structured error handling.
    kind: SemanticErrorKind,
    /// Optional help text with suggestions for fixing the error.
    help: Option<String>,
}

impl SemanticError {
    /// Creates a new error with a source location.
    ///
    /// Use this for errors that can be traced to a specific location in
    /// the source code (e.g., undefined variable, type mismatch).
    pub fn new(kind: SemanticErrorKind, message: impl Into<String>, span: Span) -> Self {
        SemanticError {
            message: message.into(),
            span: Some(span),
            kind,
            help: None,
        }
    }

    /// Creates a new error with a source location and help text.
    ///
    /// Use this for errors that benefit from additional guidance on how to fix them.
    pub fn new_with_help(
        kind: SemanticErrorKind,
        message: impl Into<String>,
        span: Span,
        help: impl Into<String>,
    ) -> Self {
        SemanticError {
            message: message.into(),
            span: Some(span),
            kind,
            help: Some(help.into()),
        }
    }

    /// Creates a new error without a source location.
    ///
    /// Use this for structural errors that cannot be traced to a specific
    /// location (e.g., missing main function).
    pub fn without_span(kind: SemanticErrorKind, message: impl Into<String>) -> Self {
        SemanticError {
            message: message.into(),
            span: None,
            kind,
            help: None,
        }
    }

    /// Creates a "missing main function" error.
    ///
    /// This is a convenience constructor for the common case of reporting
    /// that no main function was found. These errors never have a span
    /// because there's no specific location to point to.
    pub fn missing_main(message: impl Into<String>) -> Self {
        SemanticError {
            message: message.into(),
            span: None,
            kind: SemanticErrorKind::MissingMainFunction,
            help: None,
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
    pub fn kind(&self) -> SemanticErrorKind {
        self.kind
    }

    /// Returns the help text, if available.
    pub fn help(&self) -> Option<&str> {
        self.help.as_deref()
    }

    /// Returns a short, human-readable description of the error kind.
    ///
    /// This is used as the report title in error messages, while `message()`
    /// provides the detailed explanation shown in the label.
    pub fn short_message(&self) -> &'static str {
        match self.kind {
            SemanticErrorKind::DuplicateFunction => "Duplicate function",
            SemanticErrorKind::DuplicateVariable => "Duplicate variable",
            SemanticErrorKind::UndefinedVariable => "Undefined variable",
            SemanticErrorKind::UndefinedFunction => "Undefined function",
            SemanticErrorKind::TypeMismatch => "Type mismatch",
            SemanticErrorKind::IntegerOverflow => "Integer overflow",
            SemanticErrorKind::InvalidArgument => "Invalid argument",
            SemanticErrorKind::InvalidExpression => "Invalid expression",
            SemanticErrorKind::MissingMainFunction => "Missing main function",
            SemanticErrorKind::InvalidMainSignature => "Invalid main signature",
            SemanticErrorKind::InternalError => "Internal error",
        }
    }

    // =========================================================================
    // Name resolution errors
    // =========================================================================

    /// Creates an "undefined variable" error.
    pub fn undefined_variable(name: &str, span: Span) -> Self {
        Self::new(
            SemanticErrorKind::UndefinedVariable,
            format!("Undefined variable: '{}'", name),
            span,
        )
    }

    /// Creates an "undefined function" error.
    pub fn undefined_function(name: &str, span: Span) -> Self {
        Self::new(
            SemanticErrorKind::UndefinedFunction,
            format!("Undefined function: '{}'", name),
            span,
        )
    }

    /// Creates a "duplicate variable" error.
    pub fn duplicate_variable(name: &str, first_line: usize, first_col: usize, span: Span) -> Self {
        Self::new(
            SemanticErrorKind::DuplicateVariable,
            format!(
                "Variable '{}' is already defined at {}:{}",
                name, first_line, first_col
            ),
            span,
        )
    }

    /// Creates a "duplicate function" error.
    pub fn duplicate_function(name: &str, first_line: usize, first_col: usize, span: Span) -> Self {
        Self::new(
            SemanticErrorKind::DuplicateFunction,
            format!(
                "Function '{}' is already defined at {}:{}",
                name, first_line, first_col
            ),
            span,
        )
    }

    // =========================================================================
    // Type errors
    // =========================================================================

    /// Creates a type mismatch error for assigning integer to string.
    pub fn type_mismatch_int_to_string(value: i64, span: Span) -> Self {
        Self::new(
            SemanticErrorKind::TypeMismatch,
            format!(
                "Type mismatch: integer literal '{}' cannot be assigned to type 'string'",
                value
            ),
            span,
        )
    }

    /// Creates a type mismatch error for assigning integer to bool.
    pub fn type_mismatch_int_to_bool(value: i64, span: Span) -> Self {
        Self::new(
            SemanticErrorKind::TypeMismatch,
            format!(
                "Type mismatch: integer literal '{}' cannot be assigned to type 'bool'",
                value
            ),
            span,
        )
    }

    /// Creates a type mismatch error for variable type.
    pub fn type_mismatch_variable(
        name: &str,
        actual_ty: &str,
        expected_ty: &str,
        span: Span,
    ) -> Self {
        Self::new(
            SemanticErrorKind::TypeMismatch,
            format!(
                "Type mismatch: variable '{}' has type '{}', expected '{}'",
                name, actual_ty, expected_ty
            ),
            span,
        )
    }

    /// Creates a type mismatch error for assigning string to non-string type.
    pub fn type_mismatch_string_to_type(expected_ty: &str, span: Span) -> Self {
        Self::new(
            SemanticErrorKind::TypeMismatch,
            format!(
                "Type mismatch: string literal cannot be assigned to type '{}'",
                expected_ty
            ),
            span,
        )
    }

    /// Creates a type mismatch error for assigning bool to non-bool type.
    pub fn type_mismatch_bool_to_type(expected_ty: &str, span: Span) -> Self {
        Self::new(
            SemanticErrorKind::TypeMismatch,
            format!(
                "Type mismatch: boolean literal cannot be assigned to type '{}'",
                expected_ty
            ),
            span,
        )
    }

    /// Creates a type mismatch error for calling non-void function as statement.
    pub fn type_mismatch_non_void_fn_as_stmt(fn_name: &str, return_type: &str, span: Span) -> Self {
        Self::new(
            SemanticErrorKind::TypeMismatch,
            format!(
                "Function '{}' returns '{}', but only void functions can be called as statements",
                fn_name, return_type
            ),
            span,
        )
    }

    /// Creates a type mismatch error for using function call as a value.
    pub fn type_mismatch_call_as_value(callee: &str, span: Span) -> Self {
        Self::new(
            SemanticErrorKind::TypeMismatch,
            format!(
                "Function call '{}' cannot be used as a value (functions returning values not yet supported)",
                callee
            ),
            span,
        )
    }

    // =========================================================================
    // Argument errors
    // =========================================================================

    /// Creates an error for println with wrong argument count.
    pub fn invalid_argument_println_count(span: Span) -> Self {
        Self::new(
            SemanticErrorKind::InvalidArgument,
            "println expects exactly 1 argument",
            span,
        )
    }

    /// Creates an error for using function call as println argument.
    pub fn invalid_argument_call_not_supported(callee: &str, span: Span) -> Self {
        Self::new(
            SemanticErrorKind::InvalidArgument,
            format!(
                "Function call '{}' cannot be used as println argument (functions returning values not yet supported)",
                callee
            ),
            span,
        )
    }

    /// Creates an error for calling main function directly.
    pub fn invalid_argument_cannot_call_main(span: Span) -> Self {
        Self::new(
            SemanticErrorKind::InvalidArgument,
            "Cannot call 'main' function directly",
            span,
        )
    }

    /// Creates an error for calling function with arguments when it expects none.
    pub fn invalid_argument_fn_expects_no_args(fn_name: &str, got: usize, span: Span) -> Self {
        Self::new(
            SemanticErrorKind::InvalidArgument,
            format!(
                "Function '{}' expects 0 arguments, but got {}",
                fn_name, got
            ),
            span,
        )
    }

    /// Creates an error for panic with wrong argument count.
    pub fn invalid_argument_panic_count(span: Span) -> Self {
        Self::new(
            SemanticErrorKind::InvalidArgument,
            "panic expects exactly 1 argument",
            span,
        )
    }

    /// Creates an error for panic with non-string argument type.
    pub fn invalid_argument_panic_type(actual_ty: &str, span: Span) -> Self {
        Self::new_with_help(
            SemanticErrorKind::InvalidArgument,
            format!("panic requires a string argument, but got '{}'", actual_ty),
            span,
            "panic only accepts string literals or string variables",
        )
    }

    // =========================================================================
    // Expression errors
    // =========================================================================

    /// Creates an error for string literal used as statement.
    pub fn invalid_expression_string_literal(span: Span) -> Self {
        Self::new(
            SemanticErrorKind::InvalidExpression,
            "String literal as a statement has no effect. Did you mean to pass it to a function?",
            span,
        )
    }

    /// Creates an error for integer literal used as statement.
    pub fn invalid_expression_int_literal(span: Span) -> Self {
        Self::new(
            SemanticErrorKind::InvalidExpression,
            "Integer literal as a statement has no effect. Did you mean to assign it to a variable?",
            span,
        )
    }

    /// Creates an error for boolean literal used as statement.
    pub fn invalid_expression_bool_literal(span: Span) -> Self {
        Self::new(
            SemanticErrorKind::InvalidExpression,
            "Boolean literal as a statement has no effect. Did you mean to use it in a condition?",
            span,
        )
    }

    /// Creates an error for identifier used as statement.
    pub fn invalid_expression_identifier(name: &str, span: Span) -> Self {
        Self::new(
            SemanticErrorKind::InvalidExpression,
            format!(
                "Variable '{}' used as a statement has no effect. Did you mean to use it in an expression?",
                name
            ),
            span,
        )
    }

    /// Creates an error for binary operation used as statement.
    pub fn invalid_expression_binary_op(span: Span) -> Self {
        Self::new_with_help(
            SemanticErrorKind::InvalidExpression,
            "This expression computes a value but the result is not used",
            span,
            "assign the result to a variable: `let result = ...`",
        )
    }

    /// Creates an error for invalid operand type in binary operation.
    pub fn invalid_binary_op_type(
        op: crate::ast::BinaryOperator,
        actual_ty: &str,
        span: Span,
    ) -> Self {
        Self::new_with_help(
            SemanticErrorKind::TypeMismatch,
            format!("Operator '{}' cannot be used with '{}' type", op, actual_ty),
            span,
            "arithmetic operators (+, -, *, /, %) only work with numeric types (i32, i64)",
        )
    }

    /// Creates an error for invalid operand type in unary operation.
    pub fn invalid_unary_op_type(
        op: crate::ast::UnaryOperator,
        actual_ty: &str,
        span: Span,
    ) -> Self {
        Self::new_with_help(
            SemanticErrorKind::TypeMismatch,
            format!(
                "Unary operator '{}' cannot be used with '{}' type",
                op, actual_ty
            ),
            span,
            "unary negation (-) only works with numeric types (i32, i64)",
        )
    }

    /// Creates an error for unary operation used as statement.
    pub fn invalid_expression_unary_op(span: Span) -> Self {
        Self::new_with_help(
            SemanticErrorKind::InvalidExpression,
            "This expression computes a value but the result is not used",
            span,
            "assign the result to a variable: `let result = ...`",
        )
    }

    // =========================================================================
    // Structural errors
    // =========================================================================

    /// Creates an error for invalid main function signature.
    pub fn invalid_main_signature(return_type: &str, span: Span) -> Self {
        Self::new(
            SemanticErrorKind::InvalidMainSignature,
            format!(
                "main function must return void, but found return type '{}'",
                return_type
            ),
            span,
        )
    }

    // =========================================================================
    // Integer overflow
    // =========================================================================

    /// Creates an integer overflow error for i32 range.
    pub fn integer_overflow_i32(value: i64, span: Span) -> Self {
        Self::new(
            SemanticErrorKind::IntegerOverflow,
            format!(
                "Integer literal '{}' is out of range for i32 (valid range: {} to {})",
                value,
                i32::MIN,
                i32::MAX
            ),
            span,
        )
    }

    // =========================================================================
    // Internal errors
    // =========================================================================

    /// Creates an internal error for check_integer_range called with string type.
    pub fn internal_check_integer_range_string(value: i64, span: Span) -> Self {
        Self::new(
            SemanticErrorKind::InternalError,
            format!(
                "Internal error: check_integer_range called with string type for value '{}'. This is a compiler bug.",
                value
            ),
            span,
        )
    }

    /// Creates an internal error for check_integer_range called with bool type.
    pub fn internal_check_integer_range_bool(value: i64, span: Span) -> Self {
        Self::new(
            SemanticErrorKind::InternalError,
            format!(
                "Internal error: check_integer_range called with bool type for value '{}'. This is a compiler bug.",
                value
            ),
            span,
        )
    }

    /// Creates an internal error for defining variable outside a scope.
    pub fn internal_no_scope(name: &str, span: Span) -> Self {
        Self::new(
            SemanticErrorKind::InternalError,
            format!(
                "Internal error: attempted to define variable '{}' outside a scope. This is a compiler bug.",
                name
            ),
            span,
        )
    }
}

impl std::fmt::Display for SemanticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(span) = &self.span {
            write!(f, "{}:{}: {}", span.line, span.column, self.message)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

impl std::error::Error for SemanticError {}
