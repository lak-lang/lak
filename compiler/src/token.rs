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

    /// The `fn` keyword for function definitions.
    Fn,

    /// The `let` keyword for variable declarations.
    Let,

    /// A left parenthesis `(`.
    LeftParen,

    /// A right parenthesis `)`.
    RightParen,

    /// A left brace `{`.
    LeftBrace,

    /// A right brace `}`.
    RightBrace,

    /// An arrow `->` for return type annotation.
    Arrow,

    /// A comma `,`.
    Comma,

    /// A colon `:` for type annotation.
    Colon,

    /// An equals sign `=` for variable initialization in let statements.
    Equals,

    /// An integer literal (e.g., 42, 100).
    IntLiteral(i64),

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_new() {
        let span = Span::new(0, 5, 1, 1);
        assert_eq!(span.start, 0);
        assert_eq!(span.end, 5);
        assert_eq!(span.line, 1);
        assert_eq!(span.column, 1);
    }

    #[test]
    fn test_span_equality() {
        let span1 = Span::new(10, 20, 2, 5);
        let span2 = Span::new(10, 20, 2, 5);
        assert_eq!(span1, span2);
    }

    #[test]
    fn test_span_inequality() {
        let span1 = Span::new(0, 5, 1, 1);
        let span2 = Span::new(0, 6, 1, 1);
        assert_ne!(span1, span2);
    }

    #[test]
    fn test_span_copy() {
        let span1 = Span::new(0, 5, 1, 1);
        let span2 = span1; // Copy
        assert_eq!(span1, span2);
        // span1 is still usable after copy
        assert_eq!(span1.start, 0);
    }

    #[test]
    fn test_token_new() {
        let span = Span::new(0, 7, 1, 1);
        let token = Token::new(TokenKind::Identifier("println".to_string()), span);
        assert!(matches!(token.kind, TokenKind::Identifier(ref s) if s == "println"));
        assert_eq!(token.span.start, 0);
        assert_eq!(token.span.end, 7);
    }

    #[test]
    fn test_token_kind_identifier() {
        let kind = TokenKind::Identifier("my_func".to_string());
        assert!(matches!(kind, TokenKind::Identifier(ref s) if s == "my_func"));
    }

    #[test]
    fn test_token_kind_string_literal() {
        let kind = TokenKind::StringLiteral("hello world".to_string());
        assert!(matches!(kind, TokenKind::StringLiteral(ref s) if s == "hello world"));
    }

    #[test]
    fn test_token_kind_punctuation() {
        assert!(matches!(TokenKind::LeftParen, TokenKind::LeftParen));
        assert!(matches!(TokenKind::RightParen, TokenKind::RightParen));
        assert!(matches!(TokenKind::LeftBrace, TokenKind::LeftBrace));
        assert!(matches!(TokenKind::RightBrace, TokenKind::RightBrace));
        assert!(matches!(TokenKind::Comma, TokenKind::Comma));
        assert!(matches!(TokenKind::Arrow, TokenKind::Arrow));
    }

    #[test]
    fn test_token_kind_fn() {
        assert!(matches!(TokenKind::Fn, TokenKind::Fn));
    }

    #[test]
    fn test_token_kind_let() {
        assert!(matches!(TokenKind::Let, TokenKind::Let));
    }

    #[test]
    fn test_token_kind_colon() {
        assert!(matches!(TokenKind::Colon, TokenKind::Colon));
    }

    #[test]
    fn test_token_kind_equals() {
        assert!(matches!(TokenKind::Equals, TokenKind::Equals));
    }

    #[test]
    fn test_token_kind_int_literal() {
        let kind = TokenKind::IntLiteral(42);
        assert!(matches!(kind, TokenKind::IntLiteral(42)));
    }

    #[test]
    fn test_token_kind_eof() {
        assert!(matches!(TokenKind::Eof, TokenKind::Eof));
    }

    #[test]
    fn test_token_clone() {
        let span = Span::new(0, 5, 1, 1);
        let token1 = Token::new(TokenKind::Identifier("test".to_string()), span);
        let token2 = token1.clone();
        assert_eq!(token1, token2);
    }

    #[test]
    fn test_token_kind_partial_eq() {
        let kind1 = TokenKind::Identifier("foo".to_string());
        let kind2 = TokenKind::Identifier("foo".to_string());
        let kind3 = TokenKind::Identifier("bar".to_string());

        assert_eq!(kind1, kind2);
        assert_ne!(kind1, kind3);
        assert_eq!(TokenKind::LeftParen, TokenKind::LeftParen);
        assert_ne!(TokenKind::LeftParen, TokenKind::RightParen);
    }
}
