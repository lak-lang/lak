//! Lexical analyzer for the Lak programming language.
//!
//! This module provides the [`Lexer`] struct which converts source code text
//! into a stream of [`Token`]s for parsing.
//!
//! # Overview
//!
//! The lexer performs the following tasks:
//! - Scans the input character by character
//! - Recognizes identifiers, string literals, and punctuation
//! - Tracks source positions for error reporting
//! - Skips whitespace and comments
//!
//! # Supported Tokens
//!
//! - **Keywords**: `fn`, `let`
//! - **Identifiers**: Start with an ASCII alphabetic character (a-z, A-Z) or underscore, contain ASCII alphanumerics and underscores. Non-ASCII characters are rejected with an error.
//! - **Integer literals**: Sequences of ASCII digits (e.g., `42`, `0`, `9223372036854775807`).
//!   Stored as `i64` values. Negative literals (e.g., `-42`) are not directly supported;
//!   the minus sign is only recognized as part of the `->` arrow syntax.
//!   Values exceeding `i64::MAX` result in a lexer error.
//! - **String literals**: Enclosed in double quotes, support escape sequences (`\n`, `\t`, `\r`, `\\`, `\"`)
//! - **Punctuation**: `(`, `)`, `{`, `}`, `,`, `:`, `=`, `->`
//! - **Newline**: Emitted after certain tokens (identifiers, literals, `)`, `}`) for statement termination,
//!   inspired by Go's automatic semicolon insertion
//! - **Comments**: Line comments starting with `//`
//!
//! # Examples
//!
//! ```
//! use lak::lexer::Lexer;
//! use lak::token::TokenKind;
//!
//! let mut lexer = Lexer::new("println(\"hello\")");
//! let tokens = lexer.tokenize().unwrap();
//!
//! assert!(matches!(tokens[0].kind, TokenKind::Identifier(_)));
//! assert!(matches!(tokens[1].kind, TokenKind::LeftParen));
//! ```
//!
//! # Module Structure
//!
//! - [`error`] - Error types for lexical analysis
//! - [`cursor`] - Position tracking and character navigation
//! - [`skip`] - Whitespace and comment handling
//! - [`tokens`] - Token recognition and reading
//! - `tests` - Unit tests (test-only)
//!
//! # See Also
//!
//! * [`crate::token`] - Token type definitions
//! * [`crate::parser`] - Parser that consumes the token stream

mod cursor;
mod error;
mod skip;
mod tokens;

#[cfg(test)]
mod tests;

pub use error::LexError;

use crate::token::{Span, Token, TokenKind};

/// A lexical analyzer that tokenizes Lak source code.
///
/// The `Lexer` maintains its position within the input and tracks line/column
/// numbers for error reporting. It is designed to be used once per source file.
///
/// # Lifetime
///
/// The `'a` lifetime parameter ties the lexer to the input string slice,
/// ensuring the input remains valid while the lexer is in use.
pub struct Lexer<'a> {
    /// The input source code being tokenized.
    pub(super) input: &'a str,
    /// Current byte position in the input.
    pub(super) pos: usize,
    /// Current line number (1-indexed).
    pub(super) line: usize,
    /// Current column number (1-indexed).
    pub(super) column: usize,
    /// The kind of the last emitted token.
    /// Used to determine whether to emit a Newline token.
    pub(super) last_token_kind: Option<TokenKind>,
}

impl<'a> Lexer<'a> {
    /// Creates a new `Lexer` for the given input string.
    ///
    /// The lexer starts at the beginning of the input with line and column
    /// numbers initialized to 1.
    ///
    /// # Arguments
    ///
    /// * `input` - The source code to tokenize
    ///
    /// # Returns
    ///
    /// A new `Lexer` instance ready to tokenize the input.
    pub fn new(input: &'a str) -> Self {
        Lexer {
            input,
            pos: 0,
            line: 1,
            column: 1,
            last_token_kind: None,
        }
    }

    /// Tokenizes the entire input and returns a vector of tokens.
    ///
    /// This method consumes the input from start to end, producing tokens
    /// until the end of input is reached. The returned vector always ends
    /// with an [`TokenKind::Eof`] token.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Token>)` - A vector of tokens ending with `Eof`
    /// * `Err(LexError)` - If an invalid character or unterminated string is encountered
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - An unexpected character is encountered (not whitespace, identifier, string, or punctuation)
    /// - A string literal is not properly terminated
    /// - An unknown escape sequence is used in a string literal
    /// - An integer literal exceeds the i64 range
    /// - A minus sign `-` is not followed by `>` (only `->` is valid)
    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();

        loop {
            // Skip non-newline whitespace first
            self.skip_whitespace();

            // Check for newline
            if self.current_char() == Some('\n') {
                if self.should_emit_newline() {
                    let span = Span::new(self.pos, self.pos + 1, self.line, self.column);
                    tokens.push(Token::new(TokenKind::Newline, span));
                    self.last_token_kind = Some(TokenKind::Newline);
                }
                // Always consume the newline, whether we emit a token or not
                self.advance();
                continue;
            }

            // Check for and skip comments (which may consume a trailing newline)
            if let Some(consumed_newline) = self.skip_comment() {
                // Only emit a Newline token if the comment actually consumed a newline
                // and the previous token requires a Newline for statement termination
                if consumed_newline && self.should_emit_newline() {
                    // Emit a Newline token for the newline consumed by the comment.
                    // Since advance() already updated line/column, we use pos-1 and line-1.
                    let span = Span::new(self.pos - 1, self.pos, self.line - 1, 1);
                    tokens.push(Token::new(TokenKind::Newline, span));
                    self.last_token_kind = Some(TokenKind::Newline);
                }
                continue;
            }

            if self.is_eof() {
                let span = Span::new(self.pos, self.pos, self.line, self.column);
                tokens.push(Token::new(TokenKind::Eof, span));
                break;
            }

            let token = self.next_token()?;
            self.last_token_kind = Some(token.kind.clone());
            tokens.push(token);
        }

        Ok(tokens)
    }
}
