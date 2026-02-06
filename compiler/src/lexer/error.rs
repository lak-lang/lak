//! Lexical analysis error types.
//!
//! This module defines [`LexError`], which represents errors that can occur
//! during tokenization.

use crate::token::Span;

/// The kind of lexical analysis error.
///
/// This enum allows error handling code to match on error types structurally
/// rather than relying on string matching, which is fragile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LexErrorKind {
    /// Unexpected end of input while tokenizing.
    UnexpectedEof,
    /// Invalid character that doesn't start any token.
    UnexpectedCharacter,
    /// Non-ASCII character in identifier.
    InvalidIdentifierCharacter,
    /// Non-ASCII whitespace character.
    InvalidWhitespace,
    /// Incomplete arrow operator (found '-' without '>').
    IncompleteArrow,
    /// Unknown escape sequence in string literal.
    UnknownEscapeSequence,
    /// String literal not closed before end of line or file.
    UnterminatedString,
    /// Integer literal exceeds i64 range.
    IntegerOverflow,
}

/// An error that occurred during lexical analysis.
///
/// `LexError` contains a human-readable message and the source location
/// where the error occurred, enabling rich error reporting with tools
/// like [`ariadne`].
///
/// # Construction
///
/// Use [`new()`](Self::new) to create a new error with a specific source location.
///
/// [`ariadne`]: https://docs.rs/ariadne
#[derive(Debug)]
pub struct LexError {
    /// A human-readable description of the error.
    message: String,
    /// The source location where the error occurred.
    span: Span,
    /// The kind of error, for structured error handling.
    kind: LexErrorKind,
}

impl LexError {
    /// Creates a new lexical error.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind of error
    /// * `message` - A human-readable description of the error
    /// * `span` - The source location where the error occurred
    pub fn new(kind: LexErrorKind, message: impl Into<String>, span: Span) -> Self {
        LexError {
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
    pub fn kind(&self) -> LexErrorKind {
        self.kind
    }
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

impl std::error::Error for LexError {}
