//! Parser helper methods for token navigation and basic operations.

use super::Parser;
use super::error::ParseError;
use crate::token::{Span, Token, TokenKind};

impl Parser {
    /// Returns a user-friendly display string for a token kind.
    pub(super) fn token_kind_display(kind: &TokenKind) -> String {
        match kind {
            TokenKind::Fn => "'fn' keyword".to_string(),
            TokenKind::LeftBrace => "'{'".to_string(),
            TokenKind::RightBrace => "'}'".to_string(),
            TokenKind::LeftParen => "'('".to_string(),
            TokenKind::RightParen => "')'".to_string(),
            TokenKind::Arrow => "'->'".to_string(),
            TokenKind::Comma => "','".to_string(),
            TokenKind::Identifier(s) => format!("identifier '{}'", s),
            TokenKind::StringLiteral(s) => {
                if s.len() > 20 {
                    format!("string \"{}...\"", &s[..20])
                } else {
                    format!("string \"{}\"", s)
                }
            }
            TokenKind::Eof => "end of file".to_string(),
            TokenKind::Let => "'let' keyword".to_string(),
            TokenKind::Colon => "':'".to_string(),
            TokenKind::Equals => "'='".to_string(),
            TokenKind::IntLiteral(n) => format!("integer '{}'", n),
            TokenKind::Newline => "newline".to_string(),
        }
    }

    /// Returns a reference to the current token.
    ///
    /// This method is safe to call at any time - if the position is past
    /// the end, it returns the last token (which should be `Eof`).
    pub(super) fn current(&self) -> &Token {
        // new() ensures tokens is non-empty (len >= 1)
        // advance() doesn't increment pos past Eof
        // Therefore idx is always valid: 0 <= idx < tokens.len()
        let idx = self.pos.min(self.tokens.len() - 1);
        &self.tokens[idx]
    }

    /// Returns the kind of the current token.
    pub(super) fn current_kind(&self) -> &TokenKind {
        &self.current().kind
    }

    /// Returns the span of the current token.
    pub(super) fn current_span(&self) -> Span {
        self.current().span
    }

    /// Returns `true` if the current token is `Eof`.
    pub(super) fn is_eof(&self) -> bool {
        matches!(self.current_kind(), TokenKind::Eof)
    }

    /// Advances to the next token.
    ///
    /// Does nothing if already at `Eof`.
    pub(super) fn advance(&mut self) {
        if !self.is_eof() {
            self.pos += 1;
        }
    }

    /// Skips all consecutive Newline tokens.
    ///
    /// This is used in contexts where newlines are not significant
    /// (e.g., inside braces, after certain tokens).
    pub(super) fn skip_newlines(&mut self) {
        while matches!(self.current_kind(), TokenKind::Newline) && !self.is_eof() {
            self.advance();
        }
    }

    /// Expects the current token to match `expected` and advances.
    ///
    /// # Arguments
    ///
    /// * `expected` - The expected token kind
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the current token matches and was consumed
    /// * `Err(ParseError)` - If the current token does not match
    pub(super) fn expect(&mut self, expected: &TokenKind) -> Result<(), ParseError> {
        if self.current_kind() == expected {
            self.advance();
            Ok(())
        } else {
            Err(ParseError {
                message: format!(
                    "Expected {}, found {}",
                    Self::token_kind_display(expected),
                    Self::token_kind_display(self.current_kind())
                ),
                span: self.current_span(),
            })
        }
    }

    /// Expects an identifier token and returns its name.
    pub(super) fn expect_identifier(&mut self) -> Result<String, ParseError> {
        if let TokenKind::Identifier(name) = self.current_kind() {
            let name = name.clone();
            self.advance();
            Ok(name)
        } else {
            Err(ParseError {
                message: format!(
                    "Expected identifier, found {}",
                    Self::token_kind_display(self.current_kind())
                ),
                span: self.current_span(),
            })
        }
    }
}
