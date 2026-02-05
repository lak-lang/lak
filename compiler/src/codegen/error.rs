//! Code generation error types.
//!
//! This module defines [`CodegenError`], which represents errors that can occur
//! during LLVM code generation.

use crate::token::Span;

/// An error that occurred during code generation.
///
/// Contains a human-readable message and optionally the source location
/// where the error occurred, enabling rich error reporting.
#[derive(Debug)]
pub struct CodegenError {
    /// A human-readable description of the error.
    pub message: String,
    /// The source location where the error occurred, if available.
    pub span: Option<Span>,
}

impl CodegenError {
    /// Creates a new error with a message and source location.
    pub fn new(message: impl Into<String>, span: Span) -> Self {
        CodegenError {
            message: message.into(),
            span: Some(span),
        }
    }

    /// Creates a new error with only a message (no source location).
    pub fn without_span(message: impl Into<String>) -> Self {
        CodegenError {
            message: message.into(),
            span: None,
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
