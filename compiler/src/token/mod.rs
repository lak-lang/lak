//! Token definitions for the Lak lexer.
//!
//! This module provides the fundamental token types used throughout the Lak compiler.
//! It defines [`Span`] for source location tracking, [`TokenKind`] for token types,
//! and [`Token`] for representing lexical units with their positions.
//!
//! # Overview
//!
//! The lexer produces a vector of [`Token`]s, each containing:
//! - A [`TokenKind`] describing what type of token it is
//! - A [`Span`] indicating where in the source code it appears
//!
//! # Module Structure
//!
//! - [`span`] - Source location tracking ([`Span`] struct)
//! - [`kind`] - Token type definitions ([`TokenKind`] enum)
//!
//! # See Also
//!
//! * [`crate::lexer`] - The lexer that produces these tokens
//! * [`crate::parser`] - The parser that consumes these tokens

mod kind;
mod span;

pub use kind::TokenKind;
pub use span::Span;

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
    /// This is a simple constructor that performs no validation.
    /// The caller is responsible for ensuring the span correctly
    /// corresponds to the token's position in the source.
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Token { kind, span }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_new() {
        let span = Span::new(0, 7, 1, 1);
        let token = Token::new(TokenKind::Identifier("println".to_string()), span);
        assert!(matches!(token.kind, TokenKind::Identifier(ref s) if s == "println"));
        assert_eq!(token.span.start, 0);
        assert_eq!(token.span.end, 7);
    }

    #[test]
    fn test_token_clone() {
        let span = Span::new(0, 5, 1, 1);
        let token1 = Token::new(TokenKind::Identifier("test".to_string()), span);
        let token2 = token1.clone();
        assert_eq!(token1, token2);
    }
}
