//! Token reading and recognition for the lexer.
//!
//! This module provides methods for reading various token types from the input,
//! including identifiers, keywords, string literals, integer literals, and punctuation.

use super::Lexer;
use super::error::LexError;
use crate::token::{Span, Token, TokenKind};

impl<'a> Lexer<'a> {
    /// Creates a single-character token and advances the lexer.
    fn single_char_token(
        &mut self,
        kind: TokenKind,
        start_pos: usize,
        start_line: usize,
        start_column: usize,
    ) -> Token {
        self.advance();
        let span = Span::new(start_pos, self.pos, start_line, start_column);
        Token::new(kind, span)
    }

    /// Reads and returns the next token from the input.
    ///
    /// This method is called repeatedly by [`tokenize`](super::Lexer::tokenize) to
    /// produce the token stream. It assumes that whitespace and comments
    /// have already been skipped.
    ///
    /// # Errors
    ///
    /// Returns a [`LexError`] if an unexpected character is encountered
    /// or if a string literal is malformed.
    pub(super) fn next_token(&mut self) -> Result<Token, LexError> {
        let c = self.current_char().ok_or_else(|| {
            LexError::unexpected_eof(Span::new(self.pos, self.pos, self.line, self.column))
        })?;

        let start_pos = self.pos;
        let start_line = self.line;
        let start_column = self.column;

        if let Some(token) =
            self.next_punctuation_or_operator(c, start_pos, start_line, start_column)
        {
            return token;
        }

        if let Some(token) = self.next_literal(c, start_pos, start_line, start_column) {
            return token;
        }

        if let Some(token) = self.next_identifier_or_keyword(c, start_pos, start_line, start_column)
        {
            return token;
        }

        // Provide specific error message for non-ASCII alphabetic characters
        let span = Span::new(self.pos, self.pos + c.len_utf8(), self.line, self.column);
        if c.is_alphabetic() {
            Err(LexError::invalid_identifier_character(c, span))
        } else {
            Err(LexError::unexpected_character(c, span))
        }
    }

    fn next_punctuation_or_operator(
        &mut self,
        c: char,
        start_pos: usize,
        start_line: usize,
        start_column: usize,
    ) -> Option<Result<Token, LexError>> {
        let token = match c {
            '(' => Ok(self.single_char_token(
                TokenKind::LeftParen,
                start_pos,
                start_line,
                start_column,
            )),
            ')' => Ok(self.single_char_token(
                TokenKind::RightParen,
                start_pos,
                start_line,
                start_column,
            )),
            '{' => Ok(self.single_char_token(
                TokenKind::LeftBrace,
                start_pos,
                start_line,
                start_column,
            )),
            '}' => Ok(self.single_char_token(
                TokenKind::RightBrace,
                start_pos,
                start_line,
                start_column,
            )),
            ',' => {
                Ok(self.single_char_token(TokenKind::Comma, start_pos, start_line, start_column))
            }
            ':' => {
                Ok(self.single_char_token(TokenKind::Colon, start_pos, start_line, start_column))
            }
            '.' => Ok(self.single_char_token(TokenKind::Dot, start_pos, start_line, start_column)),
            '=' => {
                self.advance();
                if self.current_char() == Some('=') {
                    self.advance();
                    let span = Span::new(start_pos, self.pos, start_line, start_column);
                    Ok(Token::new(TokenKind::EqualEqual, span))
                } else {
                    let span = Span::new(start_pos, self.pos, start_line, start_column);
                    Ok(Token::new(TokenKind::Equals, span))
                }
            }
            '!' => {
                self.advance();
                if self.current_char() == Some('=') {
                    self.advance();
                    let span = Span::new(start_pos, self.pos, start_line, start_column);
                    Ok(Token::new(TokenKind::BangEqual, span))
                } else {
                    let span = Span::new(start_pos, self.pos, start_line, start_column);
                    Ok(Token::new(TokenKind::Bang, span))
                }
            }
            '&' => {
                self.advance();
                if self.current_char() == Some('&') {
                    self.advance();
                    let span = Span::new(start_pos, self.pos, start_line, start_column);
                    Ok(Token::new(TokenKind::AndAnd, span))
                } else {
                    Err(LexError::unexpected_character(
                        '&',
                        Span::new(start_pos, self.pos, start_line, start_column),
                    ))
                }
            }
            '|' => {
                self.advance();
                if self.current_char() == Some('|') {
                    self.advance();
                    let span = Span::new(start_pos, self.pos, start_line, start_column);
                    Ok(Token::new(TokenKind::OrOr, span))
                } else {
                    Err(LexError::unexpected_character(
                        '|',
                        Span::new(start_pos, self.pos, start_line, start_column),
                    ))
                }
            }
            '<' => {
                self.advance();
                if self.current_char() == Some('=') {
                    self.advance();
                    let span = Span::new(start_pos, self.pos, start_line, start_column);
                    Ok(Token::new(TokenKind::LessEqual, span))
                } else {
                    let span = Span::new(start_pos, self.pos, start_line, start_column);
                    Ok(Token::new(TokenKind::LessThan, span))
                }
            }
            '>' => {
                self.advance();
                if self.current_char() == Some('=') {
                    self.advance();
                    let span = Span::new(start_pos, self.pos, start_line, start_column);
                    Ok(Token::new(TokenKind::GreaterEqual, span))
                } else {
                    let span = Span::new(start_pos, self.pos, start_line, start_column);
                    Ok(Token::new(TokenKind::GreaterThan, span))
                }
            }
            '+' => Ok(self.single_char_token(TokenKind::Plus, start_pos, start_line, start_column)),
            '-' => {
                self.advance();
                if self.current_char() == Some('>') {
                    self.advance();
                    let span = Span::new(start_pos, self.pos, start_line, start_column);
                    Ok(Token::new(TokenKind::Arrow, span))
                } else {
                    // Minus token (not part of arrow)
                    let span = Span::new(start_pos, self.pos, start_line, start_column);
                    Ok(Token::new(TokenKind::Minus, span))
                }
            }
            '*' => Ok(self.single_char_token(TokenKind::Star, start_pos, start_line, start_column)),
            '/' => {
                Ok(self.single_char_token(TokenKind::Slash, start_pos, start_line, start_column))
            }
            '%' => {
                Ok(self.single_char_token(TokenKind::Percent, start_pos, start_line, start_column))
            }
            _ => return None,
        };

        Some(token)
    }

    fn next_literal(
        &mut self,
        c: char,
        start_pos: usize,
        start_line: usize,
        start_column: usize,
    ) -> Option<Result<Token, LexError>> {
        match c {
            '"' => Some(self.read_string(start_pos, start_line, start_column)),
            _ if c.is_ascii_digit() => Some(self.read_number(start_pos, start_line, start_column)),
            _ => None,
        }
    }

    fn next_identifier_or_keyword(
        &mut self,
        c: char,
        start_pos: usize,
        start_line: usize,
        start_column: usize,
    ) -> Option<Result<Token, LexError>> {
        if c.is_ascii_alphabetic() || c == '_' {
            Some(self.read_identifier(start_pos, start_line, start_column))
        } else {
            None
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
                            return Err(LexError::unknown_escape_sequence(
                                c,
                                Span::new(
                                    self.pos - 1,
                                    self.pos + c.len_utf8(),
                                    self.line,
                                    self.column - 1,
                                ),
                            ));
                        }
                        None => {
                            return Err(LexError::unterminated_string(Span::new(
                                start_pos,
                                self.pos,
                                start_line,
                                start_column,
                            )));
                        }
                    }
                }
                Some('\n') => {
                    return Err(LexError::unterminated_string_newline(Span::new(
                        start_pos,
                        self.pos,
                        start_line,
                        start_column,
                    )));
                }
                Some(c) => {
                    value.push(c);
                    self.advance();
                }
                None => {
                    return Err(LexError::unterminated_string(Span::new(
                        start_pos,
                        self.pos,
                        start_line,
                        start_column,
                    )));
                }
            }
        }
    }

    /// Reads an identifier or keyword from the input.
    ///
    /// Identifiers consist of an ASCII alphabetic character (a-z, A-Z) or underscore
    /// followed by any number of ASCII alphanumeric characters (a-z, A-Z, 0-9) or underscores.
    /// Non-ASCII characters (e.g., Unicode letters) are not allowed in identifiers.
    /// If the identifier matches a keyword (`fn`, `let`, `mut`, `if`, `else`, `return`,
    /// `while`, `break`, `continue`, `pub`, `import`, `as`) or boolean literal
    /// (`true`, `false`), the corresponding token kind is returned instead.
    ///
    /// # Arguments
    ///
    /// * `start_pos` - The byte position of the first character
    /// * `start_line` - The line number of the first character
    /// * `start_column` - The column number of the first character
    ///
    /// # Returns
    ///
    /// A [`Token`] with kind [`TokenKind::Identifier`] or a keyword token.
    ///
    /// # Errors
    ///
    /// Returns a [`LexError`] if a non-ASCII character is encountered in the identifier.
    fn read_identifier(
        &mut self,
        start_pos: usize,
        start_line: usize,
        start_column: usize,
    ) -> Result<Token, LexError> {
        while let Some(c) = self.current_char() {
            if c.is_ascii_alphanumeric() || c == '_' {
                self.advance();
            } else if c.is_alphanumeric() {
                // Non-ASCII alphanumeric character detected
                return Err(LexError::invalid_identifier_character(
                    c,
                    Span::new(self.pos, self.pos + c.len_utf8(), self.line, self.column),
                ));
            } else {
                break;
            }
        }

        let value = self.input[start_pos..self.pos].to_string();
        let span = Span::new(start_pos, self.pos, start_line, start_column);

        // Check for keywords and boolean literals
        let kind = match value.as_str() {
            "fn" => TokenKind::Fn,
            "let" => TokenKind::Let,
            "mut" => TokenKind::Mut,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "return" => TokenKind::Return,
            "while" => TokenKind::While,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            "pub" => TokenKind::Pub,
            "import" => TokenKind::Import,
            "as" => TokenKind::As,
            "true" => TokenKind::BoolLiteral(true),
            "false" => TokenKind::BoolLiteral(false),
            _ => TokenKind::Identifier(value),
        };

        Ok(Token::new(kind, span))
    }

    /// Reads a numeric literal from the input.
    ///
    /// Numeric literals are either:
    /// - Integer literals: one or more ASCII digits
    /// - Float literals: `digits '.' digits`
    ///
    /// # Arguments
    ///
    /// * `start_pos` - The byte position of the first digit
    /// * `start_line` - The line number of the first digit
    /// * `start_column` - The column number of the first digit
    ///
    /// # Returns
    ///
    /// A [`Token`] with kind [`TokenKind::IntLiteral`] or [`TokenKind::FloatLiteral`].
    ///
    /// # Errors
    ///
    /// Returns a [`LexError`] if:
    /// - an integer is too large to fit in a `u64`
    /// - a float literal cannot be parsed
    fn read_number(
        &mut self,
        start_pos: usize,
        start_line: usize,
        start_column: usize,
    ) -> Result<Token, LexError> {
        while self.current_char().is_some_and(|c| c.is_ascii_digit()) {
            self.advance();
        }

        // Parse float literal only when a dot is followed by at least one digit.
        let has_fraction = self.current_char() == Some('.')
            && self
                .input
                .get(self.pos..)
                .and_then(|s| s.chars().nth(1))
                .is_some_and(|c| c.is_ascii_digit());

        if has_fraction {
            self.advance(); // consume '.'
            while self.current_char().is_some_and(|c| c.is_ascii_digit()) {
                self.advance();
            }

            let value_str = &self.input[start_pos..self.pos];
            let span = Span::new(start_pos, self.pos, start_line, start_column);
            let value: f64 = value_str.parse().map_err(|_: std::num::ParseFloatError| {
                LexError::invalid_float_literal(value_str, span)
            })?;
            return Ok(Token::new(TokenKind::FloatLiteral(value), span));
        }

        let value_str = &self.input[start_pos..self.pos];
        let span = Span::new(start_pos, self.pos, start_line, start_column);

        let value: u64 = value_str
            .parse()
            .map_err(|_: std::num::ParseIntError| LexError::integer_overflow(value_str, span))?;

        Ok(Token::new(TokenKind::IntLiteral(value), span))
    }
}
