//! Whitespace and comment skipping for the lexer.
//!
//! This module provides methods for skipping over whitespace characters
//! and line comments during tokenization.

use super::Lexer;
use super::error::LexError;
use crate::token::{Span, TokenKind};

impl<'a> Lexer<'a> {
    /// Skips consecutive whitespace characters except newlines.
    ///
    /// Valid whitespace characters (following Go specification):
    /// - Space (U+0020)
    /// - Horizontal tab (U+0009)
    /// - Carriage return (U+000D)
    /// - Newline (U+000A) - handled separately in tokenize()
    ///
    /// Non-ASCII whitespace characters (e.g., U+3000 full-width space)
    /// are rejected with a clear error message.
    pub(super) fn skip_whitespace(&mut self) -> Result<(), LexError> {
        while let Some(c) = self.current_char() {
            match c {
                ' ' | '\t' | '\r' => {
                    self.advance();
                }
                '\n' => {
                    // Newlines handled separately in tokenize()
                    break;
                }
                _ => {
                    // Check if this is a non-ASCII whitespace character
                    if c.is_whitespace() {
                        return Err(LexError::invalid_whitespace(
                            c,
                            Span::new(self.pos, self.pos + c.len_utf8(), self.line, self.column),
                        ));
                    }
                    // Not whitespace at all, stop skipping
                    break;
                }
            }
        }
        Ok(())
    }

    /// Returns true if a Newline token should be emitted after the last token.
    ///
    /// Inspired by Go's automatic semicolon insertion rules, newlines
    /// are significant (act as statement terminators) only after certain tokens:
    /// - Identifiers
    /// - Literals (string, integer, boolean)
    /// - `return` keyword
    /// - `break` / `continue` keywords
    /// - `)` (right parenthesis)
    /// - `}` (right brace)
    pub(super) fn should_emit_newline(&self) -> bool {
        matches!(
            &self.last_token_kind,
            Some(TokenKind::Identifier(_))
                | Some(TokenKind::IntLiteral(_))
                | Some(TokenKind::StringLiteral(_))
                | Some(TokenKind::BoolLiteral(_))
                | Some(TokenKind::Return)
                | Some(TokenKind::Break)
                | Some(TokenKind::Continue)
                | Some(TokenKind::RightParen)
                | Some(TokenKind::RightBrace)
        )
    }

    /// Skips a line comment if one is present at the current position.
    ///
    /// Line comments start with `//` and extend to the end of the line.
    /// If a trailing newline is present, it is consumed.
    ///
    /// # Returns
    ///
    /// - `None` if no comment was present
    /// - `Some(true)` if a comment was skipped and ended with a newline
    /// - `Some(false)` if a comment was skipped but ended at EOF (no newline)
    pub(super) fn skip_comment(&mut self) -> Option<bool> {
        if self.input[self.pos..].starts_with("//") {
            let mut consumed_newline = false;
            while let Some(c) = self.current_char() {
                self.advance();
                if c == '\n' {
                    consumed_newline = true;
                    break;
                }
            }
            Some(consumed_newline)
        } else {
            None
        }
    }
}
