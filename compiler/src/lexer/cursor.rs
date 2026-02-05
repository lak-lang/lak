//! Cursor position management for the lexer.
//!
//! This module provides methods for tracking and advancing the lexer's
//! position within the input source code.

use super::Lexer;

impl<'a> Lexer<'a> {
    /// Returns the current character without consuming it.
    ///
    /// Returns `None` if the end of input has been reached.
    pub(super) fn current_char(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    /// Returns `true` if the end of input has been reached.
    pub(super) fn is_eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    /// Advances the lexer by one character.
    ///
    /// Updates the position, line, and column tracking. Handles multi-byte
    /// UTF-8 characters correctly and increments the line counter on newlines.
    pub(super) fn advance(&mut self) {
        if let Some(c) = self.current_char() {
            self.pos += c.len_utf8();
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
    }
}
