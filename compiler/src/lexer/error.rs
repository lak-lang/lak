//! Lexical analysis error types.
//!
//! This module defines [`LexError`], which represents errors that can occur
//! during tokenization.
//!
//! # Helper Constructors
//!
//! This module provides specialized constructor methods for common error cases,
//! ensuring consistent error messaging across the compiler. Prefer using these
//! helpers over constructing errors manually with [`LexError::new()`].
//!
//! Available helper methods are organized by category:
//! - **EOF errors**: [`unexpected_eof()`](LexError::unexpected_eof)
//! - **Character errors**: [`unexpected_character()`](LexError::unexpected_character),
//!   [`invalid_identifier_character()`](LexError::invalid_identifier_character),
//!   [`invalid_whitespace()`](LexError::invalid_whitespace)
//! - **String errors**: [`unknown_escape_sequence()`](LexError::unknown_escape_sequence),
//!   [`unterminated_string()`](LexError::unterminated_string),
//!   [`unterminated_string_newline()`](LexError::unterminated_string_newline)
//! - **Integer errors**: [`integer_overflow()`](LexError::integer_overflow)
//! - **Float errors**: [`invalid_float_literal()`](LexError::invalid_float_literal)

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
    /// Unknown escape sequence in string literal.
    UnknownEscapeSequence,
    /// String literal not closed before end of line or file.
    UnterminatedString,
    /// Integer literal exceeds representable range.
    IntegerOverflow,
    /// Float literal could not be parsed.
    InvalidFloatLiteral,
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

    /// Returns a short, human-readable description of the error kind.
    ///
    /// This is used as the report title in error messages, while `message()`
    /// provides the detailed explanation shown in the label.
    pub fn short_message(&self) -> &'static str {
        match self.kind {
            LexErrorKind::UnexpectedEof => "Unexpected end of input",
            LexErrorKind::UnexpectedCharacter => "Unexpected character",
            LexErrorKind::InvalidIdentifierCharacter => "Invalid identifier character",
            LexErrorKind::InvalidWhitespace => "Invalid whitespace",
            LexErrorKind::UnknownEscapeSequence => "Unknown escape sequence",
            LexErrorKind::UnterminatedString => "Unterminated string",
            LexErrorKind::IntegerOverflow => "Integer overflow",
            LexErrorKind::InvalidFloatLiteral => "Invalid float literal",
        }
    }

    // =========================================================================
    // EOF errors
    // =========================================================================

    /// Creates an "unexpected end of input" error.
    pub fn unexpected_eof(span: Span) -> Self {
        Self::new(LexErrorKind::UnexpectedEof, "Unexpected end of input", span)
    }

    // =========================================================================
    // Character errors
    // =========================================================================

    /// Creates an "unexpected character" error.
    pub fn unexpected_character(ch: char, span: Span) -> Self {
        Self::new(
            LexErrorKind::UnexpectedCharacter,
            format!("Unexpected character: '{}'", ch),
            span,
        )
    }

    /// Creates an "invalid identifier character" error.
    pub fn invalid_identifier_character(ch: char, span: Span) -> Self {
        Self::new(
            LexErrorKind::InvalidIdentifierCharacter,
            format!(
                "Invalid character '{}' in identifier. Only ASCII letters (a-z, A-Z), digits (0-9), and underscores (_) are allowed",
                ch
            ),
            span,
        )
    }

    /// Creates an "invalid whitespace" error.
    pub fn invalid_whitespace(ch: char, span: Span) -> Self {
        Self::new(
            LexErrorKind::InvalidWhitespace,
            format!(
                "Invalid whitespace character '{}' (U+{:04X}). Only space, tab, carriage return, and newline are allowed",
                ch, ch as u32
            ),
            span,
        )
    }

    // =========================================================================
    // String errors
    // =========================================================================

    /// Creates an "unknown escape sequence" error.
    pub fn unknown_escape_sequence(ch: char, span: Span) -> Self {
        Self::new(
            LexErrorKind::UnknownEscapeSequence,
            format!("Unknown escape sequence: '\\{}'", ch),
            span,
        )
    }

    /// Creates an "unterminated string" error.
    pub fn unterminated_string(span: Span) -> Self {
        Self::new(
            LexErrorKind::UnterminatedString,
            "Unterminated string literal",
            span,
        )
    }

    /// Creates an "unterminated string (newline)" error.
    pub fn unterminated_string_newline(span: Span) -> Self {
        Self::new(
            LexErrorKind::UnterminatedString,
            "Unterminated string literal (newline in string)",
            span,
        )
    }

    // =========================================================================
    // Integer errors
    // =========================================================================

    /// Creates an "integer overflow" error.
    pub fn integer_overflow(value_str: &str, span: Span) -> Self {
        Self::new(
            LexErrorKind::IntegerOverflow,
            format!(
                "Integer literal '{}' is too large (exceeds maximum representable value)",
                value_str
            ),
            span,
        )
    }

    /// Creates an "invalid float literal" error.
    pub fn invalid_float_literal(value_str: &str, span: Span) -> Self {
        Self::new(
            LexErrorKind::InvalidFloatLiteral,
            format!("Invalid float literal '{}'", value_str),
            span,
        )
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
