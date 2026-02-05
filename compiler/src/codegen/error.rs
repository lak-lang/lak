//! Code generation error types.
//!
//! This module defines [`CodegenError`], which represents errors that can occur
//! during LLVM code generation.

use crate::token::Span;

/// The kind of code generation error.
///
/// This enum allows error handling code to match on error types structurally
/// rather than relying on string matching, which is fragile.
///
/// # Note
///
/// Most semantic errors (undefined variables, type mismatches, etc.) are now
/// detected during semantic analysis. This enum only contains errors that can
/// occur during LLVM IR generation or object file output.
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
