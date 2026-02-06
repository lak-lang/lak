//! Parse error types.

use crate::token::Span;

/// The kind of parse error.
///
/// This enum allows error handling code to match on error types structurally
/// rather than relying on string matching, which is fragile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseErrorKind {
    /// Expected a statement terminator (newline, '}', or Eof).
    MissingStatementTerminator,
    /// Expected a specific token but found something else.
    UnexpectedToken,
    /// Expected an identifier but found something else.
    ExpectedIdentifier,
    /// Expected a type annotation but found something else.
    ExpectedType,
    /// Expression following identifier without parentheses (likely missing function call syntax).
    MissingFunctionCallParentheses,
}

/// An error that occurred during parsing.
///
/// `ParseError` contains a human-readable message and the source location
/// where the error occurred, enabling rich error reporting.
///
/// # Construction
///
/// Use [`new()`](Self::new) to create a new error with a specific source location.
///
/// # See Also
///
/// * [`crate::lexer::LexError`] - Similar error type for lexical errors
#[derive(Debug)]
pub struct ParseError {
    /// A human-readable description of the error.
    message: String,
    /// The source location where the error occurred.
    span: Span,
    /// The kind of error, for structured error handling.
    kind: ParseErrorKind,
}

impl ParseError {
    /// Creates a new parse error.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind of error
    /// * `message` - A human-readable description of the error
    /// * `span` - The source location where the error occurred
    pub fn new(kind: ParseErrorKind, message: impl Into<String>, span: Span) -> Self {
        ParseError {
            message: message.into(),
            span,
            kind,
        }
    }

    /// Returns the error message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the source location where the error occurred.
    pub fn span(&self) -> Span {
        self.span
    }

    /// Returns the kind of error.
    pub fn kind(&self) -> ParseErrorKind {
        self.kind
    }
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
