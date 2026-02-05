//! Parser for the Lak programming language.
//!
//! This module provides the [`Parser`] struct which transforms a token stream
//! into an Abstract Syntax Tree ([`Program`]).
//!
//! # Overview
//!
//! The parser implements a recursive descent parsing strategy. It consumes
//! tokens produced by the [`crate::lexer`] and builds an AST suitable for
//! code generation.
//!
//! # Grammar
//!
//! The current Lak grammar is minimal:
//!
//! ```text
//! program     → stmt* EOF
//! stmt        → expr_stmt
//! expr_stmt   → expr
//! expr        → call | STRING
//! call        → IDENTIFIER "(" arguments? ")"
//! arguments   → expr ("," expr)*
//! ```
//!
//! # Examples
//!
//! ```
//! use lak::lexer::Lexer;
//! use lak::parser::Parser;
//!
//! let mut lexer = Lexer::new("println(\"hello\")");
//! let tokens = lexer.tokenize().unwrap();
//!
//! let mut parser = Parser::new(tokens);
//! let program = parser.parse().unwrap();
//!
//! assert_eq!(program.stmts.len(), 1);
//! ```
//!
//! # See Also
//!
//! * [`crate::lexer`] - Produces the token stream consumed by the parser
//! * [`crate::ast`] - Defines the AST types produced by the parser
//! * [`crate::codegen`] - Consumes the AST to generate LLVM IR

use crate::ast::{Expr, Program, Stmt};
use crate::token::{Span, Token, TokenKind};

/// A recursive descent parser for the Lak language.
///
/// The parser maintains a position within the token stream and provides
/// methods to parse various grammar productions.
///
/// # Usage
///
/// Create a parser with [`Parser::new`], then call [`Parser::parse`] to
/// produce an AST.
pub struct Parser {
    /// The token stream to parse.
    tokens: Vec<Token>,
    /// Current position in the token stream.
    pos: usize,
}

/// An error that occurred during parsing.
///
/// `ParseError` contains a human-readable message and the source location
/// where the error occurred, enabling rich error reporting.
///
/// # See Also
///
/// * [`crate::lexer::LexError`] - Similar error type for lexical errors
#[derive(Debug)]
pub struct ParseError {
    /// A human-readable description of the error.
    pub message: String,
    /// The source location where the error occurred.
    pub span: Span,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}: {}",
            self.span.line, self.span.column, self.message
        )
    }
}

impl Parser {
    /// Creates a new parser from a token list.
    ///
    /// # Panics
    /// Panics if the token list is empty. The lexer should always
    /// produce at least an Eof token.
    pub fn new(tokens: Vec<Token>) -> Self {
        assert!(!tokens.is_empty(), "Token list must not be empty");
        Parser { tokens, pos: 0 }
    }

    /// Parses the entire token stream into a [`Program`].
    ///
    /// This is the main entry point for parsing. It repeatedly parses
    /// statements until the end of file is reached.
    ///
    /// # Returns
    ///
    /// * `Ok(Program)` - The parsed program AST
    /// * `Err(ParseError)` - If a syntax error is encountered
    ///
    /// # Errors
    ///
    /// Returns an error if any statement fails to parse. Common causes:
    /// - Unexpected token where a specific token was expected
    /// - Missing parentheses in function calls
    /// - Unrecognized expression forms
    pub fn parse(&mut self) -> Result<Program, ParseError> {
        let mut stmts = Vec::new();

        while !self.is_eof() {
            let stmt = self.parse_stmt()?;
            stmts.push(stmt);
        }

        Ok(Program { stmts })
    }

    /// Returns a reference to the current token.
    ///
    /// This method is safe to call at any time - if the position is past
    /// the end, it returns the last token (which should be `Eof`).
    fn current(&self) -> &Token {
        // new() ensures tokens is non-empty (len >= 1)
        // advance() doesn't increment pos past Eof
        // Therefore idx is always valid: 0 <= idx < tokens.len()
        let idx = self.pos.min(self.tokens.len() - 1);
        &self.tokens[idx]
    }

    /// Returns the kind of the current token.
    fn current_kind(&self) -> &TokenKind {
        &self.current().kind
    }

    /// Returns the span of the current token.
    fn current_span(&self) -> Span {
        self.current().span
    }

    /// Returns `true` if the current token is `Eof`.
    fn is_eof(&self) -> bool {
        matches!(self.current_kind(), TokenKind::Eof)
    }

    /// Advances to the next token.
    ///
    /// Does nothing if already at `Eof`.
    fn advance(&mut self) {
        if !self.is_eof() {
            self.pos += 1;
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
    fn expect(&mut self, expected: &TokenKind) -> Result<(), ParseError> {
        if self.current_kind() == expected {
            self.advance();
            Ok(())
        } else {
            Err(ParseError {
                message: format!("Expected {:?}, found {:?}", expected, self.current_kind()),
                span: self.current_span(),
            })
        }
    }

    /// Parses a single statement.
    ///
    /// Currently only expression statements are supported.
    fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.parse_expr()?;
        Ok(Stmt::Expr(expr))
    }

    /// Parses an expression.
    ///
    /// Handles identifiers (which must be followed by `(` to form a call)
    /// and string literals.
    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        let token = self.current().clone();

        match &token.kind {
            TokenKind::Identifier(name) => {
                let name = name.clone();
                self.advance();
                if matches!(self.current_kind(), TokenKind::LeftParen) {
                    self.parse_call(name)
                } else {
                    Err(ParseError {
                        message: format!("Expected '(' after identifier '{}'", name),
                        span: self.current_span(),
                    })
                }
            }
            TokenKind::StringLiteral(value) => {
                let value = value.clone();
                self.advance();
                Ok(Expr::StringLiteral(value))
            }
            _ => Err(ParseError {
                message: format!("Unexpected token: {:?}", token.kind),
                span: token.span,
            }),
        }
    }

    /// Parses a function call expression.
    ///
    /// The callee identifier has already been consumed. This method parses
    /// the argument list within parentheses.
    ///
    /// # Arguments
    ///
    /// * `callee` - The name of the function being called
    ///
    /// # Grammar
    ///
    /// ```text
    /// call → IDENTIFIER "(" arguments? ")"
    /// arguments → expr ("," expr)*
    /// ```
    fn parse_call(&mut self, callee: String) -> Result<Expr, ParseError> {
        self.expect(&TokenKind::LeftParen)?;

        let mut args = Vec::new();

        if !matches!(self.current_kind(), TokenKind::RightParen) {
            loop {
                let arg = self.parse_expr()?;
                args.push(arg);

                if matches!(self.current_kind(), TokenKind::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        self.expect(&TokenKind::RightParen)?;

        Ok(Expr::Call { callee, args })
    }
}
