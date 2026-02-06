//! Expression parsing using Pratt parsing (precedence climbing).
//!
//! This module implements expression parsing with proper operator precedence
//! using the Pratt parsing algorithm. The parser handles:
//! - Primary expressions (literals, identifiers, function calls, parenthesized expressions)
//! - Binary operations with correct precedence and left-associativity

use super::Parser;
use super::error::ParseError;
use crate::ast::{BinaryOperator, Expr, ExprKind};
use crate::token::{Span, TokenKind};

/// Operator precedence levels (higher number = lower precedence = looser binding).
///
/// Lower precedence operators are parsed later, forming parent nodes in the AST.
/// For example, `2 + 3 * 4` is parsed as `2 + (3 * 4)` because multiplication
/// (precedence 2) binds tighter than addition (precedence 3).
///
/// Levels follow the Lak specification:
/// - Level 2: `*`, `/`, `%` (multiplicative) - tighter binding
/// - Level 3: `+`, `-` (additive) - looser binding
const PRECEDENCE_MULTIPLICATIVE: u8 = 2;
const PRECEDENCE_ADDITIVE: u8 = 3;

/// Returns the precedence of a binary operator token, if it is one.
///
/// Returns `None` for non-operator tokens.
fn binary_op_precedence(kind: &TokenKind) -> Option<u8> {
    match kind {
        TokenKind::Star | TokenKind::Slash | TokenKind::Percent => Some(PRECEDENCE_MULTIPLICATIVE),
        TokenKind::Plus | TokenKind::Minus => Some(PRECEDENCE_ADDITIVE),
        _ => None,
    }
}

/// Converts a token kind to a binary operator.
///
/// Returns `None` for non-operator tokens.
fn token_to_binary_op(kind: &TokenKind) -> Option<BinaryOperator> {
    match kind {
        TokenKind::Plus => Some(BinaryOperator::Add),
        TokenKind::Minus => Some(BinaryOperator::Sub),
        TokenKind::Star => Some(BinaryOperator::Mul),
        TokenKind::Slash => Some(BinaryOperator::Div),
        TokenKind::Percent => Some(BinaryOperator::Mod),
        _ => None,
    }
}

impl Parser {
    /// Parses an expression using Pratt parsing.
    ///
    /// This is the main entry point for expression parsing. It handles
    /// operator precedence and associativity correctly.
    ///
    /// # Grammar
    ///
    /// ```text
    /// expr → primary (binary_op primary)*
    /// primary → IDENTIFIER | IDENTIFIER "(" arguments? ")" | STRING | INT | "(" expr ")"
    /// binary_op → "+" | "-" | "*" | "/" | "%"
    /// ```
    pub(super) fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_expr_pratt(u8::MAX)
    }

    /// Parses an expression with Pratt parsing, respecting minimum precedence.
    ///
    /// This method implements the core Pratt parsing algorithm:
    /// 1. Parse a primary expression (atom)
    /// 2. While the current token is an operator with precedence >= min_precedence:
    ///    a. Consume the operator
    ///    b. Recursively parse the right-hand side with higher precedence
    ///    c. Build a BinaryOp node
    ///
    /// # Arguments
    ///
    /// * `min_precedence` - The minimum precedence level to parse at this level.
    ///   Lower precedence numbers mean higher priority (tighter binding).
    fn parse_expr_pratt(&mut self, min_precedence: u8) -> Result<Expr, ParseError> {
        // Parse the left-hand side (primary expression)
        let mut left = self.parse_primary_expr()?;

        // Continue parsing binary operators while they have sufficient precedence
        loop {
            // Check if the current token is a binary operator
            let Some(precedence) = binary_op_precedence(self.current_kind()) else {
                break;
            };

            // Stop if this operator has lower precedence (higher number) than our minimum
            if precedence > min_precedence {
                break;
            }

            // Get the operator and its span
            let op_span = self.current_span();
            let op = token_to_binary_op(self.current_kind())
                .ok_or_else(|| ParseError::internal_binary_op_inconsistency(op_span))?;
            self.advance();

            // Skip newlines after operator (allows multi-line expressions)
            self.skip_newlines();

            // Parse the right-hand side with `precedence - 1` for left-associativity.
            // This makes the current operator bind tighter than itself, so `a - b - c`
            // parses as `(a - b) - c` rather than `a - (b - c)`.
            let right = self.parse_expr_pratt(precedence - 1)?;

            // Build the BinaryOp node with span covering both operands
            let span = Span::new(
                left.span.start,
                right.span.end,
                left.span.line,
                left.span.column,
            );

            left = Expr::new(
                ExprKind::BinaryOp {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                },
                span,
            );
        }

        Ok(left)
    }

    /// Parses a primary expression (atom).
    ///
    /// Primary expressions are the basic building blocks:
    /// - Integer literals
    /// - String literals
    /// - Identifiers (variable references)
    /// - Function calls
    /// - Parenthesized expressions
    fn parse_primary_expr(&mut self) -> Result<Expr, ParseError> {
        let start_span = self.current_span();

        match self.current_kind() {
            TokenKind::LeftParen => {
                // Parenthesized expression
                self.advance(); // consume '('
                self.skip_newlines();

                let inner = self.parse_expr()?;

                self.skip_newlines();
                self.expect(&TokenKind::RightParen)?;

                // Return the inner expression with updated span covering the parens
                let span = Span::new(
                    start_span.start,
                    self.tokens[self.pos.saturating_sub(1)].span.end,
                    start_span.line,
                    start_span.column,
                );
                Ok(Expr::new(inner.kind, span))
            }
            TokenKind::Identifier(name) => {
                let name = name.clone();
                self.advance();

                if matches!(self.current_kind(), TokenKind::LeftParen) {
                    // Function call
                    self.parse_call(name, start_span)
                } else {
                    // Check for syntax error: identifier followed by expression-start token
                    // without an intervening Newline (which would indicate a new statement)
                    // Note: Operators are valid after identifiers in binary expressions
                    match self.current_kind() {
                        TokenKind::StringLiteral(_) => Err(
                            ParseError::missing_fn_call_parens_string(&name, self.current_span()),
                        ),
                        TokenKind::IntLiteral(_) => Err(ParseError::missing_fn_call_parens_int(
                            &name,
                            self.current_span(),
                        )),
                        TokenKind::Identifier(next_name) => {
                            let next_name = next_name.clone();
                            Err(ParseError::missing_fn_call_parens_ident(
                                &name,
                                &next_name,
                                self.current_span(),
                            ))
                        }
                        // Any other token (including Newline, Eof, operators, etc.)
                        // means this is a valid variable reference
                        _ => Ok(Expr::new(ExprKind::Identifier(name), start_span)),
                    }
                }
            }
            TokenKind::StringLiteral(value) => {
                let value = value.clone();
                self.advance();
                Ok(Expr::new(ExprKind::StringLiteral(value), start_span))
            }
            TokenKind::IntLiteral(value) => {
                let value = *value;
                self.advance();
                Ok(Expr::new(ExprKind::IntLiteral(value), start_span))
            }
            _ => Err(ParseError::unexpected_expression_start(
                &Self::token_kind_display(self.current_kind()),
                start_span,
            )),
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
    /// * `start_span` - The span of the callee identifier
    ///
    /// # Grammar
    ///
    /// ```text
    /// call → IDENTIFIER "(" arguments? ")"
    /// arguments → expr ("," expr)*
    /// ```
    pub(super) fn parse_call(
        &mut self,
        callee: String,
        start_span: Span,
    ) -> Result<Expr, ParseError> {
        self.expect(&TokenKind::LeftParen)?;
        self.skip_newlines(); // Skip newlines after opening paren

        let mut args = Vec::new();

        if !matches!(self.current_kind(), TokenKind::RightParen) {
            loop {
                let arg = self.parse_expr()?;
                args.push(arg);
                self.skip_newlines(); // Skip newlines after argument

                if matches!(self.current_kind(), TokenKind::Comma) {
                    self.advance();
                    self.skip_newlines(); // Skip newlines after comma
                } else {
                    break;
                }
            }
        }

        self.skip_newlines(); // Skip newlines before closing paren
        let end_span = self.current_span();
        self.expect(&TokenKind::RightParen)?;

        // Span covers from callee to closing paren
        let span = Span::new(
            start_span.start,
            end_span.end,
            start_span.line,
            start_span.column,
        );

        Ok(Expr::new(ExprKind::Call { callee, args }, span))
    }
}
