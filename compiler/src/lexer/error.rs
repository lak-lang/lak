//! Lexical analysis error types.
//!
//! This module defines [`LexError`], which represents errors that can occur
//! during tokenization.

use crate::token::Span;

/// An error that occurred during lexical analysis.
///
/// `LexError` contains a human-readable message and the source location
/// where the error occurred, enabling rich error reporting with tools
/// like [`ariadne`].
///
/// [`ariadne`]: https://docs.rs/ariadne
#[derive(Debug)]
pub struct LexError {
    /// A human-readable description of the error.
    pub message: String,
    /// The source location where the error occurred.
    pub span: Span,
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}: {}",
            self.span.line, self.span.column, self.message
        )
    }
}
