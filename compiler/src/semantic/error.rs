//! Semantic analysis error types.
//!
//! This module defines [`SemanticError`], which represents errors that can occur
//! during semantic analysis (name resolution, type checking, etc.).

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
/// - **Structural errors** (no span): [`MissingMainFunction`](Self::MissingMainFunction),
///   [`InvalidMainSignature`](Self::InvalidMainSignature) (has span for return type)
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
