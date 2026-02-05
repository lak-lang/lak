//! Token definitions for the Lak lexer.
//!
//! This module provides the fundamental token types used throughout the Lak compiler.
//! It defines [`Span`] for source location tracking and [`Token`] for representing
//! lexical units with their positions.
//!
//! # Overview
//!
//! The lexer produces a stream of [`Token`]s, each containing:
//! - A [`TokenKind`] describing what type of token it is
//! - A [`Span`] indicating where in the source code it appears
//!
//! # See Also
//!
//! * [`crate::lexer`] - The lexer that produces these tokens
//! * [`crate::parser`] - The parser that consumes these tokens

/// A span representing a range in the source code.
///
/// `Span` tracks both byte offsets (for slicing the source string) and
/// human-readable positions (line and column numbers) for error reporting.
///
/// # Fields
///
/// * `start` - The starting byte offset (inclusive) in the source string
/// * `end` - The ending byte offset (exclusive) in the source string
/// * `line` - The 1-indexed line number where this span begins
/// * `column` - The 1-indexed column number where this span begins
///
/// # Examples
///
/// ```
/// use lak::token::Span;
///
/// let span = Span::new(0, 5, 1, 1);
/// assert_eq!(span.start, 0);
/// assert_eq!(span.end, 5);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// The starting byte offset (inclusive) in the source string.
    pub start: usize,
    /// The ending byte offset (exclusive) in the source string.
    pub end: usize,
    /// The 1-indexed line number where this span begins.
    pub line: usize,
    /// The 1-indexed column number where this span begins.
    pub column: usize,
}

impl Span {
    /// Creates a new `Span` with the given byte offsets and position.
    ///
    /// # Arguments
    ///
    /// * `start` - The starting byte offset (inclusive)
    /// * `end` - The ending byte offset (exclusive)
    /// * `line` - The 1-indexed line number
    /// * `column` - The 1-indexed column number
    ///
    /// # Returns
    ///
    /// A new `Span` instance with the specified values.
    pub fn new(start: usize, end: usize, line: usize, column: usize) -> Self {
        Span {
            start,
            end,
            line,
            column,
        }
    }
}

/// The kind of token recognized by the lexer.
///
/// This enum represents all possible token types in the Lak language.
/// Each variant may carry associated data (e.g., the actual string value
/// for identifiers and string literals).
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    /// An identifier (function name, variable name, etc.).
    ///
    /// Identifiers start with a letter or underscore, followed by
    /// any number of alphanumeric characters or underscores.
    Identifier(String),

    /// A string literal enclosed in double quotes.
    ///
    /// The contained `String` is the unescaped value (escape sequences
    /// like `\n` are already converted to their actual characters).
    StringLiteral(String),

    /// A left parenthesis `(`.
    LeftParen,

    /// A right parenthesis `)`.
    RightParen,

    /// A comma `,`.
    Comma,

    /// End of file marker.
    ///
    /// This is always the last token in a valid token stream.
    Eof,
}

/// A token with its kind and source location.
///
/// `Token` combines a [`TokenKind`] with a [`Span`], allowing the parser
/// and error reporting systems to know both what kind of token was found
/// and where it appeared in the source code.
///
/// # See Also
///
/// * [`TokenKind`] - The different types of tokens
/// * [`Span`] - Source location information
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// The kind of this token.
    pub kind: TokenKind,
    /// The source location of this token.
    pub span: Span,
}

impl Token {
    /// Creates a new `Token` with the given kind and span.
    ///
    /// # Arguments
    ///
    /// * `kind` - The type of token
    /// * `span` - The source location of this token
    ///
    /// # Returns
    ///
    /// A new `Token` instance.
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Token { kind, span }
    }
}
