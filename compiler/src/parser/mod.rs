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
//! The current Lak grammar:
//!
//! ```text
//! program     → fn_def* EOF
//! fn_def      → "fn" IDENTIFIER "(" ")" "->" IDENTIFIER "{" stmt* "}"
//! stmt        → let_stmt | expr_stmt
//! let_stmt    → "let" IDENTIFIER ":" type "=" expr
//! type        → "i32" | "i64"
//! expr_stmt   → expr
//! expr        → call | IDENTIFIER | STRING | INT
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
//! let mut lexer = Lexer::new("fn main() -> void { println(\"hello\") }");
//! let tokens = lexer.tokenize().unwrap();
//!
//! let mut parser = Parser::new(tokens);
//! let program = parser.parse().unwrap();
//!
//! assert_eq!(program.functions.len(), 1);
//! assert_eq!(program.functions[0].name, "main");
//! ```
//!
//! # Module Structure
//!
//! - [`error`] - Parse error types
//! - `helpers` - Token navigation and basic parsing operations
//! - `fn_def` - Function definition parsing
//! - `stmt` - Statement parsing
//! - `types` - Type annotation parsing
//! - `expr` - Expression parsing
//! - `tests` - Unit tests (test-only)
//!
//! # See Also
//!
//! * [`crate::lexer`] - Produces the token stream consumed by the parser
//! * [`crate::ast`] - Defines the AST types produced by the parser
//! * [`crate::codegen`] - Consumes the AST to generate LLVM IR

mod error;
mod expr;
mod fn_def;
mod helpers;
mod stmt;
mod types;

#[cfg(test)]
mod tests;

pub use error::ParseError;

use crate::ast::Program;
use crate::token::Token;

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
    /// function definitions until the end of file is reached.
    ///
    /// # Returns
    ///
    /// * `Ok(Program)` - The parsed program AST
    /// * `Err(ParseError)` - If a syntax error is encountered
    ///
    /// # Errors
    ///
    /// Returns an error if any function definition fails to parse. Common causes:
    /// - Missing `fn` keyword at top level
    /// - Malformed function signature
    /// - Syntax errors in function body
    pub fn parse(&mut self) -> Result<Program, ParseError> {
        let mut functions = Vec::new();

        while !self.is_eof() {
            let fn_def = self.parse_fn_def()?;
            functions.push(fn_def);
        }

        Ok(Program { functions })
    }
}
