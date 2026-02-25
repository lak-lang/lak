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
//! program     → import* fn_def* EOF
//! import      → "import" STRING ("as" IDENTIFIER)?
//! fn_def      → ("pub")? "fn" IDENTIFIER "(" param_list? ")" "->" IDENTIFIER "{" stmt* "}"
//! param_list  → IDENTIFIER ":" type ("," IDENTIFIER ":" type)*
//! stmt        → let_stmt | assign_stmt | return_stmt | if_stmt | while_stmt | break_stmt | continue_stmt | expr_stmt
//! let_stmt    → "let" "mut"? IDENTIFIER ":" type "=" expr | "let" "_" "=" expr
//! assign_stmt → IDENTIFIER "=" expr
//! return_stmt → "return" expr?
//! if_stmt     → "if" expr "{" stmt* "}" ("else" (if_stmt | "{" stmt* "}"))?
//! while_stmt  → "while" expr "{" stmt* "}"
//! break_stmt  → "break"
//! continue_stmt → "continue"
//! type        → integer primitives | "string" | "bool"
//! expr_stmt   → expr
//! expr        → if_expr | call | member_access | IDENTIFIER | STRING | INT
//! if_expr     → "if" expr "{" stmt* expr "}" "else" "{" stmt* expr "}"
//! call        → IDENTIFIER "(" arguments? ")"
//! member_access → IDENTIFIER "." IDENTIFIER
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
//! - `import` - Import declaration parsing
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
mod import;
mod stmt;
mod types;

#[cfg(test)]
mod tests;

pub use error::{ParseError, ParseErrorKind};

use crate::ast::Program;
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

impl Parser {
    fn eof_placeholder_token() -> Token {
        Token::new(TokenKind::Eof, Span::new(0, 0, 1, 1))
    }

    /// Creates a new parser from a token list.
    ///
    /// This compatibility constructor never panics. If `tokens` is empty,
    /// it inserts a synthetic `Eof` token to preserve parser invariants.
    pub fn new(tokens: Vec<Token>) -> Self {
        if tokens.is_empty() {
            return Parser {
                tokens: vec![Self::eof_placeholder_token()],
                pos: 0,
            };
        }

        Parser { tokens, pos: 0 }
    }

    /// Creates a new parser from a token list, returning an error if it is empty.
    ///
    /// Use this when callers want explicit validation instead of normalization.
    pub fn try_new(tokens: Vec<Token>) -> Result<Self, ParseError> {
        if tokens.is_empty() {
            return Err(ParseError::internal(
                "Internal error: parser received an empty token stream. This is a compiler bug.",
                Span::new(0, 0, 1, 1),
            ));
        }

        Ok(Parser { tokens, pos: 0 })
    }

    /// Parses the entire token stream into a [`Program`].
    ///
    /// This is the main entry point for parsing. It first parses import
    /// declarations, then function definitions until the end of file is reached.
    ///
    /// # Returns
    ///
    /// * `Ok(Program)` - The parsed program AST
    /// * `Err(ParseError)` - If a syntax error is encountered
    ///
    /// # Errors
    ///
    /// Returns an error if any import or function definition fails to parse.
    /// Common causes:
    /// - Missing `fn` keyword at top level
    /// - Malformed function signature
    /// - Syntax errors in function body
    /// - Invalid import syntax
    pub fn parse(&mut self) -> Result<Program, ParseError> {
        let mut imports = Vec::new();
        let mut functions = Vec::new();

        // Parse imports first (must come before function definitions)
        while !self.is_eof() {
            self.skip_newlines();
            if self.is_eof() {
                break;
            }

            // Check if this is an import statement
            if matches!(self.current_kind(), TokenKind::Import) {
                let import = self.parse_import()?;
                imports.push(import);
                self.expect_statement_terminator()?;
            } else {
                // Not an import, break to parse function definitions
                break;
            }
        }

        // Parse function definitions
        while !self.is_eof() {
            self.skip_newlines();
            if self.is_eof() {
                break;
            }
            let fn_def = self.parse_fn_def()?;
            functions.push(fn_def);
            self.expect_statement_terminator()?;
        }

        Ok(Program { imports, functions })
    }
}
