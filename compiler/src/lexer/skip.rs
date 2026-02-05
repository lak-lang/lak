//! Whitespace and comment skipping for the lexer.
//!
//! This module provides methods for skipping over whitespace characters
//! and line comments during tokenization.

use super::Lexer;

impl<'a> Lexer<'a> {
    /// Skips whitespace and comments in a loop.
    ///
    /// This method handles the case where a comment might be followed by
    /// whitespace, which might be followed by another comment, etc.
    pub(super) fn skip_whitespace_and_comments(&mut self) {
        loop {
            self.skip_whitespace();
            if !self.skip_comment() {
                break;
            }
        }
    }

    /// Skips consecutive whitespace characters.
    fn skip_whitespace(&mut self) {
        while self.current_char().is_some_and(|c| c.is_whitespace()) {
            self.advance();
        }
    }

    /// Skips a line comment if one is present at the current position.
    ///
    /// Line comments start with `//` and extend to the end of the line.
    ///
    /// # Returns
    ///
    /// `true` if a comment was skipped, `false` otherwise.
    fn skip_comment(&mut self) -> bool {
        if self.input[self.pos..].starts_with("//") {
            while let Some(c) = self.current_char() {
                if c == '\n' {
                    self.advance();
                    break;
                }
                self.advance();
            }
            true
        } else {
            false
        }
    }
}
