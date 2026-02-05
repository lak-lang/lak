use crate::token::{Span, Token, TokenKind};

pub struct Lexer<'a> {
    input: &'a str,
    pos: usize,
    line: usize,
    column: usize,
}

#[derive(Debug)]
pub struct LexError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}: {}", self.line, self.column, self.message)
    }
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            input,
            pos: 0,
            line: 1,
            column: 1,
        }
    }

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

    fn current_char(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.input.len()
    }

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

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            self.skip_whitespace();
            if !self.skip_comment() {
                break;
            }
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current_char() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

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

    fn next_token(&mut self) -> Result<Token, LexError> {
        let c = self.current_char().ok_or_else(|| LexError {
            message: "Unexpected end of input".to_string(),
            line: self.line,
            column: self.column,
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
                line: self.line,
                column: self.column,
            }),
        }
    }

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
                                line: self.line,
                                column: self.column,
                            });
                        }
                        None => {
                            return Err(LexError {
                                message: "Unterminated string literal".to_string(),
                                line: start_line,
                                column: start_column,
                            });
                        }
                    }
                }
                Some('\n') => {
                    return Err(LexError {
                        message: "Unterminated string literal (newline in string)".to_string(),
                        line: self.line,
                        column: self.column,
                    });
                }
                Some(c) => {
                    value.push(c);
                    self.advance();
                }
                None => {
                    return Err(LexError {
                        message: "Unterminated string literal".to_string(),
                        line: start_line,
                        column: start_column,
                    });
                }
            }
        }
    }

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
