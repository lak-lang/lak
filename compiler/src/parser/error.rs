//! Parse error types.

use crate::token::Span;

/// An error that occurred during parsing.
///
/// `ParseError` contains a human-readable message and the source location
/// where the error occurred, enabling rich error reporting.
///
/// # See Also
///
/// * [`crate::lexer::LexError`] - Similar error type for lexical errors
#[derive(Debug)]
pub struct ParseError {
    /// A human-readable description of the error.
    pub message: String,
    /// The source location where the error occurred.
    pub span: Span,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}: {}",
            self.span.line, self.span.column, self.message
        )
    }
}

impl std::error::Error for ParseError {}
