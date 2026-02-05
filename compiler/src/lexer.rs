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
//! - **Identifiers**: Start with a Unicode alphabetic character or underscore, contain Unicode alphanumerics and underscores
//! - **String literals**: Enclosed in double quotes, support escape sequences (`\n`, `\t`, `\r`, `\\`, `\"`)
//! - **Punctuation**: `(`, `)`, `,`
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
//! # See Also
//!
//! * [`crate::token`] - Token type definitions
//! * [`crate::parser`] - Parser that consumes the token stream

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
    input: &'a str,
    /// Current byte position in the input.
    pos: usize,
    /// Current line number (1-indexed).
    line: usize,
    /// Current column number (1-indexed).
    column: usize,
}

/// An error that occurred during lexical analysis.
///
/// `LexError` contains a human-readable message and the source location
/// where the error occurred, enabling rich error reporting with tools
/// like [`ariadne`].
///
/// [`ariadne`]: https://docs.rs/ariadne
#[derive(Debug)]
pub struct LexError {
    /// A human-readable description of the error.
    pub message: String,
    /// The source location where the error occurred.
    pub span: Span,
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
    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();

        loop {
            self.skip_whitespace_and_comments();

            if self.is_eof() {
                let span = Span::new(self.pos, self.pos, self.line, self.column);
                tokens.push(Token::new(TokenKind::Eof, span));
                break;
            }

            let token = self.next_token()?;
            tokens.push(token);
        }

        Ok(tokens)
    }

    /// Returns the current character without consuming it.
    ///
    /// Returns `None` if the end of input has been reached.
    fn current_char(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    /// Returns `true` if the end of input has been reached.
    fn is_eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    /// Advances the lexer by one character.
    ///
    /// Updates the position, line, and column tracking. Handles multi-byte
    /// UTF-8 characters correctly and increments the line counter on newlines.
    fn advance(&mut self) {
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

    /// Skips whitespace and comments in a loop.
    ///
    /// This method handles the case where a comment might be followed by
    /// whitespace, which might be followed by another comment, etc.
    fn skip_whitespace_and_comments(&mut self) {
        loop {
            self.skip_whitespace();
            if !self.skip_comment() {
                break;
            }
        }
    }

    /// Skips consecutive whitespace characters.
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current_char() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
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

    /// Reads and returns the next token from the input.
    ///
    /// This method is called repeatedly by [`tokenize`](Self::tokenize) to
    /// produce the token stream. It assumes that whitespace and comments
    /// have already been skipped.
    ///
    /// # Errors
    ///
    /// Returns a [`LexError`] if an unexpected character is encountered
    /// or if a string literal is malformed.
    fn next_token(&mut self) -> Result<Token, LexError> {
        let c = self.current_char().ok_or_else(|| LexError {
            message: "Unexpected end of input".to_string(),
            span: Span::new(self.pos, self.pos, self.line, self.column),
        })?;

        let start_pos = self.pos;
        let start_line = self.line;
        let start_column = self.column;

        match c {
            '(' => {
                self.advance();
                let span = Span::new(start_pos, self.pos, start_line, start_column);
                Ok(Token::new(TokenKind::LeftParen, span))
            }
            ')' => {
                self.advance();
                let span = Span::new(start_pos, self.pos, start_line, start_column);
                Ok(Token::new(TokenKind::RightParen, span))
            }
            ',' => {
                self.advance();
                let span = Span::new(start_pos, self.pos, start_line, start_column);
                Ok(Token::new(TokenKind::Comma, span))
            }
            '"' => self.read_string(start_pos, start_line, start_column),
            _ if c.is_alphabetic() || c == '_' => {
                Ok(self.read_identifier(start_pos, start_line, start_column))
            }
            _ => Err(LexError {
                message: format!("Unexpected character: '{}'", c),
                span: Span::new(self.pos, self.pos + c.len_utf8(), self.line, self.column),
            }),
        }
    }

    /// Reads a string literal from the input.
    ///
    /// The opening double quote should be at the current position. This method
    /// processes escape sequences and returns the unescaped string value.
    ///
    /// # Supported Escape Sequences
    ///
    /// - `\n` - newline
    /// - `\t` - tab
    /// - `\r` - carriage return
    /// - `\\` - backslash
    /// - `\"` - double quote
    ///
    /// # Arguments
    ///
    /// * `start_pos` - The byte position of the opening quote
    /// * `start_line` - The line number of the opening quote
    /// * `start_column` - The column number of the opening quote
    ///
    /// # Errors
    ///
    /// Returns a [`LexError`] if:
    /// - The string contains an unknown escape sequence
    /// - The string is not terminated (reaches end of line or file)
    fn read_string(
        &mut self,
        start_pos: usize,
        start_line: usize,
        start_column: usize,
    ) -> Result<Token, LexError> {
        self.advance(); // skip opening "
        let mut value = String::new();

        loop {
            match self.current_char() {
                Some('"') => {
                    self.advance(); // skip closing "
                    let span = Span::new(start_pos, self.pos, start_line, start_column);
                    return Ok(Token::new(TokenKind::StringLiteral(value), span));
                }
                Some('\\') => {
                    self.advance(); // skip backslash
                    match self.current_char() {
                        Some('n') => {
                            value.push('\n');
                            self.advance();
                        }
                        Some('t') => {
                            value.push('\t');
                            self.advance();
                        }
                        Some('r') => {
                            value.push('\r');
                            self.advance();
                        }
                        Some('\\') => {
                            value.push('\\');
                            self.advance();
                        }
                        Some('"') => {
                            value.push('"');
                            self.advance();
                        }
                        Some(c) => {
                            return Err(LexError {
                                message: format!("Unknown escape sequence: \\{}", c),
                                span: Span::new(
                                    self.pos - 1,
                                    self.pos + c.len_utf8(),
                                    self.line,
                                    self.column - 1,
                                ),
                            });
                        }
                        None => {
                            return Err(LexError {
                                message: "Unterminated string literal".to_string(),
                                span: Span::new(start_pos, self.pos, start_line, start_column),
                            });
                        }
                    }
                }
                Some('\n') => {
                    return Err(LexError {
                        message: "Unterminated string literal (newline in string)".to_string(),
                        span: Span::new(start_pos, self.pos, start_line, start_column),
                    });
                }
                Some(c) => {
                    value.push(c);
                    self.advance();
                }
                None => {
                    return Err(LexError {
                        message: "Unterminated string literal".to_string(),
                        span: Span::new(start_pos, self.pos, start_line, start_column),
                    });
                }
            }
        }
    }

    /// Reads an identifier from the input.
    ///
    /// Identifiers consist of a Unicode alphabetic character or underscore
    /// followed by any number of Unicode alphanumeric characters or underscores.
    ///
    /// # Arguments
    ///
    /// * `start_pos` - The byte position of the first character
    /// * `start_line` - The line number of the first character
    /// * `start_column` - The column number of the first character
    ///
    /// # Returns
    ///
    /// A [`Token`] with kind [`TokenKind::Identifier`] containing the identifier text.
    fn read_identifier(
        &mut self,
        start_pos: usize,
        start_line: usize,
        start_column: usize,
    ) -> Token {
        while let Some(c) = self.current_char() {
            if c.is_alphanumeric() || c == '_' {
                self.advance();
            } else {
                break;
            }
        }

        let value = self.input[start_pos..self.pos].to_string();
        let span = Span::new(start_pos, self.pos, start_line, start_column);
        Token::new(TokenKind::Identifier(value), span)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to tokenize input and return only the kinds.
    fn tokenize_kinds(input: &str) -> Vec<TokenKind> {
        let mut lexer = Lexer::new(input);
        lexer
            .tokenize()
            .unwrap_or_else(|e| panic!("Tokenization failed for input {:?}: {}", input, e))
            .into_iter()
            .map(|t| t.kind)
            .collect()
    }

    /// Helper function to tokenize input and return the error.
    fn tokenize_error(input: &str) -> LexError {
        let mut lexer = Lexer::new(input);
        match lexer.tokenize() {
            Ok(tokens) => panic!(
                "Expected tokenization to fail for input {:?}, but it succeeded with {} tokens",
                input,
                tokens.len()
            ),
            Err(e) => e,
        }
    }

    // ===================
    // Basic tokens
    // ===================

    #[test]
    fn test_empty_input() {
        let kinds = tokenize_kinds("");
        assert_eq!(kinds, vec![TokenKind::Eof]);
    }

    #[test]
    fn test_whitespace_only() {
        let kinds = tokenize_kinds("   \n\t");
        assert_eq!(kinds, vec![TokenKind::Eof]);
    }

    #[test]
    fn test_left_paren() {
        let kinds = tokenize_kinds("(");
        assert_eq!(kinds, vec![TokenKind::LeftParen, TokenKind::Eof]);
    }

    #[test]
    fn test_right_paren() {
        let kinds = tokenize_kinds(")");
        assert_eq!(kinds, vec![TokenKind::RightParen, TokenKind::Eof]);
    }

    #[test]
    fn test_comma() {
        let kinds = tokenize_kinds(",");
        assert_eq!(kinds, vec![TokenKind::Comma, TokenKind::Eof]);
    }

    #[test]
    fn test_multiple_punctuation() {
        let kinds = tokenize_kinds("(,)");
        assert_eq!(
            kinds,
            vec![
                TokenKind::LeftParen,
                TokenKind::Comma,
                TokenKind::RightParen,
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_punctuation_with_spaces() {
        let kinds = tokenize_kinds("( , )");
        assert_eq!(
            kinds,
            vec![
                TokenKind::LeftParen,
                TokenKind::Comma,
                TokenKind::RightParen,
                TokenKind::Eof
            ]
        );
    }

    // ===================
    // Identifiers
    // ===================

    #[test]
    fn test_identifier_simple() {
        let kinds = tokenize_kinds("println");
        assert_eq!(
            kinds,
            vec![TokenKind::Identifier("println".to_string()), TokenKind::Eof]
        );
    }

    #[test]
    fn test_identifier_with_underscore() {
        let kinds = tokenize_kinds("my_func");
        assert_eq!(
            kinds,
            vec![TokenKind::Identifier("my_func".to_string()), TokenKind::Eof]
        );
    }

    #[test]
    fn test_identifier_starts_with_underscore() {
        let kinds = tokenize_kinds("_private");
        assert_eq!(
            kinds,
            vec![
                TokenKind::Identifier("_private".to_string()),
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_identifier_with_numbers() {
        let kinds = tokenize_kinds("func123");
        assert_eq!(
            kinds,
            vec![TokenKind::Identifier("func123".to_string()), TokenKind::Eof]
        );
    }

    #[test]
    fn test_identifier_underscore_only() {
        let kinds = tokenize_kinds("_");
        assert_eq!(
            kinds,
            vec![TokenKind::Identifier("_".to_string()), TokenKind::Eof]
        );
    }

    #[test]
    fn test_multiple_identifiers() {
        let kinds = tokenize_kinds("foo bar");
        assert_eq!(
            kinds,
            vec![
                TokenKind::Identifier("foo".to_string()),
                TokenKind::Identifier("bar".to_string()),
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_identifier_unicode() {
        let kinds = tokenize_kinds("日本語");
        assert_eq!(
            kinds,
            vec![TokenKind::Identifier("日本語".to_string()), TokenKind::Eof]
        );
    }

    // ===================
    // String literals
    // ===================

    #[test]
    fn test_string_empty() {
        let kinds = tokenize_kinds(r#""""#);
        assert_eq!(
            kinds,
            vec![TokenKind::StringLiteral("".to_string()), TokenKind::Eof]
        );
    }

    #[test]
    fn test_string_simple() {
        let kinds = tokenize_kinds(r#""hello""#);
        assert_eq!(
            kinds,
            vec![
                TokenKind::StringLiteral("hello".to_string()),
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_string_with_spaces() {
        let kinds = tokenize_kinds(r#""hello world""#);
        assert_eq!(
            kinds,
            vec![
                TokenKind::StringLiteral("hello world".to_string()),
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_string_escape_newline() {
        let kinds = tokenize_kinds(r#""a\nb""#);
        assert_eq!(
            kinds,
            vec![TokenKind::StringLiteral("a\nb".to_string()), TokenKind::Eof]
        );
    }

    #[test]
    fn test_string_escape_tab() {
        let kinds = tokenize_kinds(r#""a\tb""#);
        assert_eq!(
            kinds,
            vec![TokenKind::StringLiteral("a\tb".to_string()), TokenKind::Eof]
        );
    }

    #[test]
    fn test_string_escape_carriage_return() {
        let kinds = tokenize_kinds(r#""a\rb""#);
        assert_eq!(
            kinds,
            vec![TokenKind::StringLiteral("a\rb".to_string()), TokenKind::Eof]
        );
    }

    #[test]
    fn test_string_escape_backslash() {
        let kinds = tokenize_kinds(r#""a\\b""#);
        assert_eq!(
            kinds,
            vec![TokenKind::StringLiteral("a\\b".to_string()), TokenKind::Eof]
        );
    }

    #[test]
    fn test_string_escape_quote() {
        let kinds = tokenize_kinds(r#""a\"b""#);
        assert_eq!(
            kinds,
            vec![TokenKind::StringLiteral("a\"b".to_string()), TokenKind::Eof]
        );
    }

    #[test]
    fn test_string_multiple_escapes() {
        let kinds = tokenize_kinds(r#""\n\t\r""#);
        assert_eq!(
            kinds,
            vec![
                TokenKind::StringLiteral("\n\t\r".to_string()),
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_string_all_escapes_combined() {
        let kinds = tokenize_kinds(r#""\n\t\r\\\"end""#);
        assert_eq!(
            kinds,
            vec![
                TokenKind::StringLiteral("\n\t\r\\\"end".to_string()),
                TokenKind::Eof
            ]
        );
    }

    // ===================
    // Comments
    // ===================

    #[test]
    fn test_comment_single_line() {
        let kinds = tokenize_kinds("// comment\n");
        assert_eq!(kinds, vec![TokenKind::Eof]);
    }

    #[test]
    fn test_comment_at_eof() {
        let kinds = tokenize_kinds("// comment");
        assert_eq!(kinds, vec![TokenKind::Eof]);
    }

    #[test]
    fn test_comment_after_code() {
        let kinds = tokenize_kinds("println // comment\n");
        assert_eq!(
            kinds,
            vec![TokenKind::Identifier("println".to_string()), TokenKind::Eof]
        );
    }

    #[test]
    fn test_comment_between_tokens() {
        let kinds = tokenize_kinds("a // c\nb");
        assert_eq!(
            kinds,
            vec![
                TokenKind::Identifier("a".to_string()),
                TokenKind::Identifier("b".to_string()),
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_multiple_comments() {
        let kinds = tokenize_kinds("// first\n// second\nfoo");
        assert_eq!(
            kinds,
            vec![TokenKind::Identifier("foo".to_string()), TokenKind::Eof]
        );
    }

    // ===================
    // Compound expressions
    // ===================

    #[test]
    fn test_println_call() {
        let kinds = tokenize_kinds(r#"println("hello")"#);
        assert_eq!(
            kinds,
            vec![
                TokenKind::Identifier("println".to_string()),
                TokenKind::LeftParen,
                TokenKind::StringLiteral("hello".to_string()),
                TokenKind::RightParen,
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_nested_call() {
        let kinds = tokenize_kinds(r#"outer(inner("x"))"#);
        assert_eq!(
            kinds,
            vec![
                TokenKind::Identifier("outer".to_string()),
                TokenKind::LeftParen,
                TokenKind::Identifier("inner".to_string()),
                TokenKind::LeftParen,
                TokenKind::StringLiteral("x".to_string()),
                TokenKind::RightParen,
                TokenKind::RightParen,
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_multiple_args() {
        let kinds = tokenize_kinds(r#"func("a", "b", "c")"#);
        assert_eq!(
            kinds,
            vec![
                TokenKind::Identifier("func".to_string()),
                TokenKind::LeftParen,
                TokenKind::StringLiteral("a".to_string()),
                TokenKind::Comma,
                TokenKind::StringLiteral("b".to_string()),
                TokenKind::Comma,
                TokenKind::StringLiteral("c".to_string()),
                TokenKind::RightParen,
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_multiple_statements() {
        let kinds = tokenize_kinds("foo()\nbar()");
        assert_eq!(
            kinds,
            vec![
                TokenKind::Identifier("foo".to_string()),
                TokenKind::LeftParen,
                TokenKind::RightParen,
                TokenKind::Identifier("bar".to_string()),
                TokenKind::LeftParen,
                TokenKind::RightParen,
                TokenKind::Eof
            ]
        );
    }

    // ===================
    // Span verification
    // ===================

    #[test]
    fn test_span_positions() {
        let mut lexer = Lexer::new("foo");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].span.start, 0);
        assert_eq!(tokens[0].span.end, 3);
    }

    #[test]
    fn test_span_line_column() {
        let mut lexer = Lexer::new("foo");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].span.line, 1);
        assert_eq!(tokens[0].span.column, 1);
    }

    #[test]
    fn test_span_multiline() {
        let mut lexer = Lexer::new("a\nb");
        let tokens = lexer.tokenize().unwrap();

        // First token 'a' on line 1
        assert_eq!(tokens[0].span.line, 1);
        assert_eq!(tokens[0].span.column, 1);

        // Second token 'b' on line 2
        assert_eq!(tokens[1].span.line, 2);
        assert_eq!(tokens[1].span.column, 1);
    }

    #[test]
    fn test_span_string_literal() {
        let mut lexer = Lexer::new(r#""hello""#);
        let tokens = lexer.tokenize().unwrap();

        // String includes quotes in span
        assert_eq!(tokens[0].span.start, 0);
        assert_eq!(tokens[0].span.end, 7); // includes both quotes
    }

    #[test]
    fn test_span_after_whitespace() {
        let mut lexer = Lexer::new("   foo");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].span.start, 3);
        assert_eq!(tokens[0].span.end, 6);
        assert_eq!(tokens[0].span.column, 4);
    }

    // ===================
    // Error cases
    // ===================

    #[test]
    fn test_error_unterminated_string() {
        let err = tokenize_error(r#""hello"#);
        assert!(err.message.contains("Unterminated string"));
    }

    #[test]
    fn test_error_newline_in_string() {
        let err = tokenize_error("\"hello\nworld\"");
        assert!(err.message.contains("newline in string"));
    }

    #[test]
    fn test_error_unknown_escape() {
        let err = tokenize_error(r#""\x""#);
        assert!(err.message.contains("Unknown escape sequence"));
    }

    #[test]
    fn test_error_unexpected_char_at() {
        let err = tokenize_error("@");
        assert!(err.message.contains("Unexpected character"));
    }

    #[test]
    fn test_error_unexpected_char_hash() {
        let err = tokenize_error("#");
        assert!(err.message.contains("Unexpected character"));
    }

    #[test]
    fn test_error_unexpected_char_number() {
        let err = tokenize_error("123");
        assert!(err.message.contains("Unexpected character"));
    }

    #[test]
    fn test_error_unexpected_char_dollar() {
        let err = tokenize_error("$");
        assert!(err.message.contains("Unexpected character"));
    }

    #[test]
    fn test_error_span_location() {
        let err = tokenize_error("foo @");
        assert_eq!(err.span.start, 4);
        assert_eq!(err.span.column, 5);
    }

    #[test]
    fn test_lex_error_display() {
        let err = LexError {
            message: "Test error".to_string(),
            span: Span::new(0, 1, 2, 3),
        };
        let display = format!("{}", err);
        assert!(display.contains("2:3"));
        assert!(display.contains("Test error"));
    }

    // ===================
    // Edge cases
    // ===================

    #[test]
    fn test_error_escape_at_eof() {
        let err = tokenize_error(r#""hello\"#);
        assert!(err.message.contains("Unterminated string"));
    }

    #[test]
    fn test_windows_line_endings() {
        let kinds = tokenize_kinds("a\r\nb");
        assert_eq!(
            kinds,
            vec![
                TokenKind::Identifier("a".to_string()),
                TokenKind::Identifier("b".to_string()),
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_multiple_consecutive_backslashes() {
        let kinds = tokenize_kinds(r#""\\\\""#);
        assert_eq!(
            kinds,
            vec![TokenKind::StringLiteral("\\\\".to_string()), TokenKind::Eof]
        );
    }

    #[test]
    fn test_multiple_consecutive_quotes() {
        let kinds = tokenize_kinds(r#""\"\"""#);
        assert_eq!(
            kinds,
            vec![TokenKind::StringLiteral("\"\"".to_string()), TokenKind::Eof]
        );
    }
}
