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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    /// Helper function to parse input and return the Program.
    fn parse(input: &str) -> Result<Program, ParseError> {
        let mut lexer = Lexer::new(input);
        let tokens = lexer
            .tokenize()
            .unwrap_or_else(|e| panic!("Lexer failed on parser test input {:?}: {}", input, e));
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    /// Helper function to parse input and extract the first expression.
    fn parse_first_expr(input: &str) -> Expr {
        let program =
            parse(input).unwrap_or_else(|e| panic!("Failed to parse input {:?}: {}", input, e));

        let first_stmt = program
            .stmts
            .first()
            .unwrap_or_else(|| panic!("Input {:?} produced no statements", input));

        match first_stmt {
            Stmt::Expr(expr) => expr.clone(),
        }
    }

    /// Helper function to parse input and return the error.
    fn parse_error(input: &str) -> ParseError {
        match parse(input) {
            Ok(program) => panic!(
                "Expected parsing to fail for input {:?}, but it succeeded with {} statements",
                input,
                program.stmts.len()
            ),
            Err(e) => e,
        }
    }

    // ===================
    // Basic parsing
    // ===================

    #[test]
    fn test_empty_program() {
        let program = parse("").unwrap();
        assert!(program.stmts.is_empty());
    }

    #[test]
    fn test_call_no_args() {
        let expr = parse_first_expr("func()");
        match expr {
            Expr::Call { callee, args } => {
                assert_eq!(callee, "func");
                assert!(args.is_empty());
            }
            _ => panic!("Expected Call expression"),
        }
    }

    #[test]
    fn test_call_one_arg() {
        let expr = parse_first_expr(r#"println("hello")"#);
        match expr {
            Expr::Call { callee, args } => {
                assert_eq!(callee, "println");
                assert_eq!(args.len(), 1);
                assert!(matches!(&args[0], Expr::StringLiteral(s) if s == "hello"));
            }
            _ => panic!("Expected Call expression"),
        }
    }

    #[test]
    fn test_call_multiple_args() {
        let expr = parse_first_expr(r#"f("a", "b", "c")"#);
        match expr {
            Expr::Call { callee, args } => {
                assert_eq!(callee, "f");
                assert_eq!(args.len(), 3);
                assert!(matches!(&args[0], Expr::StringLiteral(s) if s == "a"));
                assert!(matches!(&args[1], Expr::StringLiteral(s) if s == "b"));
                assert!(matches!(&args[2], Expr::StringLiteral(s) if s == "c"));
            }
            _ => panic!("Expected Call expression"),
        }
    }

    // ===================
    // Nested calls
    // ===================

    #[test]
    fn test_nested_call_single() {
        let expr = parse_first_expr("outer(inner())");
        match expr {
            Expr::Call { callee, args } => {
                assert_eq!(callee, "outer");
                assert_eq!(args.len(), 1);
                match &args[0] {
                    Expr::Call {
                        callee: inner_callee,
                        args: inner_args,
                    } => {
                        assert_eq!(inner_callee, "inner");
                        assert!(inner_args.is_empty());
                    }
                    _ => panic!("Expected nested Call"),
                }
            }
            _ => panic!("Expected Call expression"),
        }
    }

    #[test]
    fn test_nested_call_with_arg() {
        let expr = parse_first_expr(r#"outer(inner("x"))"#);
        match expr {
            Expr::Call { callee, args } => {
                assert_eq!(callee, "outer");
                assert_eq!(args.len(), 1);
                match &args[0] {
                    Expr::Call {
                        callee: inner_callee,
                        args: inner_args,
                    } => {
                        assert_eq!(inner_callee, "inner");
                        assert_eq!(inner_args.len(), 1);
                        assert!(matches!(&inner_args[0], Expr::StringLiteral(s) if s == "x"));
                    }
                    _ => panic!("Expected nested Call"),
                }
            }
            _ => panic!("Expected Call expression"),
        }
    }

    #[test]
    fn test_deeply_nested() {
        let expr = parse_first_expr("a(b(c(d())))");
        match expr {
            Expr::Call { callee, args } => {
                assert_eq!(callee, "a");
                assert_eq!(args.len(), 1);
                // Verify structure: a -> b -> c -> d
                match &args[0] {
                    Expr::Call { callee: b, args } => {
                        assert_eq!(b, "b");
                        match &args[0] {
                            Expr::Call { callee: c, args } => {
                                assert_eq!(c, "c");
                                match &args[0] {
                                    Expr::Call { callee: d, args } => {
                                        assert_eq!(d, "d");
                                        assert!(args.is_empty());
                                    }
                                    _ => panic!("Expected d call"),
                                }
                            }
                            _ => panic!("Expected c call"),
                        }
                    }
                    _ => panic!("Expected b call"),
                }
            }
            _ => panic!("Expected Call expression"),
        }
    }

    #[test]
    fn test_nested_multiple_args() {
        let expr = parse_first_expr(r#"f(g(), h(), "x")"#);
        match expr {
            Expr::Call { callee, args } => {
                assert_eq!(callee, "f");
                assert_eq!(args.len(), 3);
                assert!(matches!(&args[0], Expr::Call { callee, .. } if callee == "g"));
                assert!(matches!(&args[1], Expr::Call { callee, .. } if callee == "h"));
                assert!(matches!(&args[2], Expr::StringLiteral(s) if s == "x"));
            }
            _ => panic!("Expected Call expression"),
        }
    }

    // ===================
    // Multiple statements
    // ===================

    #[test]
    fn test_multiple_statements() {
        let program = parse("f()\ng()").unwrap();
        assert_eq!(program.stmts.len(), 2);

        match &program.stmts[0] {
            Stmt::Expr(Expr::Call { callee, .. }) => assert_eq!(callee, "f"),
            _ => panic!("Expected f call"),
        }
        match &program.stmts[1] {
            Stmt::Expr(Expr::Call { callee, .. }) => assert_eq!(callee, "g"),
            _ => panic!("Expected g call"),
        }
    }

    #[test]
    fn test_statements_with_comments() {
        let program = parse("f() // c\ng()").unwrap();
        assert_eq!(program.stmts.len(), 2);
    }

    #[test]
    fn test_statements_on_same_line() {
        // Note: In Lak, statements aren't separated by semicolons,
        // but parsing should still work when tokens are contiguous
        let program = parse("a() b()").unwrap();
        assert_eq!(program.stmts.len(), 2);
    }

    // ===================
    // Expression types
    // ===================

    #[test]
    fn test_string_literal_as_arg() {
        let expr = parse_first_expr(r#"f("str")"#);
        match expr {
            Expr::Call { args, .. } => {
                assert!(matches!(&args[0], Expr::StringLiteral(s) if s == "str"));
            }
            _ => panic!("Expected Call"),
        }
    }

    #[test]
    fn test_call_as_arg() {
        let expr = parse_first_expr("f(g())");
        match expr {
            Expr::Call { args, .. } => {
                assert!(matches!(&args[0], Expr::Call { callee, .. } if callee == "g"));
            }
            _ => panic!("Expected Call"),
        }
    }

    // ===================
    // Error cases
    // ===================

    #[test]
    fn test_error_missing_left_paren() {
        let err = parse_error("func");
        assert!(err.message.contains("Expected '('"));
    }

    #[test]
    fn test_error_missing_right_paren() {
        let err = parse_error(r#"func("a""#);
        assert!(err.message.contains("RightParen"));
    }

    #[test]
    fn test_error_unexpected_left_paren() {
        let err = parse_error("(");
        assert!(err.message.contains("Unexpected token"));
    }

    #[test]
    fn test_error_unexpected_comma() {
        let err = parse_error(",");
        assert!(err.message.contains("Unexpected token"));
    }

    #[test]
    fn test_error_double_comma() {
        let err = parse_error(r#"f("a",,"b")"#);
        assert!(err.message.contains("Unexpected token"));
    }

    #[test]
    fn test_error_leading_comma() {
        let err = parse_error(r#"f(,"a")"#);
        assert!(err.message.contains("Unexpected token"));
    }

    #[test]
    fn test_error_trailing_comma() {
        // Trailing comma should error (no implicit nil in Lak)
        let err = parse_error(r#"f("a",)"#);
        assert!(err.message.contains("Unexpected token"));
    }

    // ===================
    // Panic tests
    // ===================

    #[test]
    #[should_panic(expected = "Token list must not be empty")]
    fn test_parser_new_panics_on_empty() {
        Parser::new(vec![]);
    }

    // ===================
    // Edge cases
    // ===================

    #[test]
    fn test_whitespace_in_call() {
        let expr = parse_first_expr("func  (  )");
        match expr {
            Expr::Call { callee, args } => {
                assert_eq!(callee, "func");
                assert!(args.is_empty());
            }
            _ => panic!("Expected Call"),
        }
    }

    #[test]
    fn test_newlines_in_call() {
        let expr = parse_first_expr("func(\n\"a\"\n)");
        match expr {
            Expr::Call { callee, args } => {
                assert_eq!(callee, "func");
                assert_eq!(args.len(), 1);
            }
            _ => panic!("Expected Call"),
        }
    }

    #[test]
    fn test_parse_error_display() {
        let err = ParseError {
            message: "Test error".to_string(),
            span: Span::new(0, 1, 2, 3),
        };
        let display = format!("{}", err);
        assert!(display.contains("2:3"));
        assert!(display.contains("Test error"));
    }
}
