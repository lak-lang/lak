//! Expression parsing using Pratt parsing (precedence climbing).
//!
//! This module implements expression parsing with proper operator precedence
//! using the Pratt parsing algorithm. The parser handles:
//! - Primary expressions (literals, identifiers, function calls, parenthesized expressions)
//! - Binary operations with correct precedence and left-associativity

use super::Parser;
use super::error::ParseError;
use crate::ast::{BinaryOperator, Expr, ExprKind, UnaryOperator};
use crate::token::{Span, TokenKind};

/// Operator precedence levels (higher number = lower precedence = looser binding).
///
/// Lower precedence operators are parsed later, forming parent nodes in the AST.
/// For example, `2 + 3 * 4` is parsed as `2 + (3 * 4)` because multiplication
/// (precedence 2) binds tighter than addition (precedence 3).
///
/// Levels follow the Lak specification:
/// - Level 1: `-` (unary negation) - tightest binding
/// - Level 2: `*`, `/`, `%` (multiplicative)
/// - Level 3: `+`, `-` (additive)
/// - Level 4: `<`, `>`, `<=`, `>=` (comparison)
/// - Level 5: `==`, `!=` (equality) - looser binding
/// - Level 6: `&&` (logical AND)
/// - Level 7: `||` (logical OR)
const PRECEDENCE_UNARY: u8 = 1;
const PRECEDENCE_MULTIPLICATIVE: u8 = 2;
const PRECEDENCE_ADDITIVE: u8 = 3;
const PRECEDENCE_COMPARISON: u8 = 4;
const PRECEDENCE_EQUALITY: u8 = 5;
const PRECEDENCE_LOGICAL_AND: u8 = 6;
const PRECEDENCE_LOGICAL_OR: u8 = 7;

/// Returns the precedence of a binary operator token, if it is one.
///
/// Returns `None` for non-operator tokens.
fn binary_op_precedence(kind: &TokenKind) -> Option<u8> {
    match kind {
        TokenKind::Star | TokenKind::Slash | TokenKind::Percent => Some(PRECEDENCE_MULTIPLICATIVE),
        TokenKind::Plus | TokenKind::Minus => Some(PRECEDENCE_ADDITIVE),
        TokenKind::LessThan
        | TokenKind::GreaterThan
        | TokenKind::LessEqual
        | TokenKind::GreaterEqual => Some(PRECEDENCE_COMPARISON),
        TokenKind::EqualEqual | TokenKind::BangEqual => Some(PRECEDENCE_EQUALITY),
        TokenKind::AndAnd => Some(PRECEDENCE_LOGICAL_AND),
        TokenKind::OrOr => Some(PRECEDENCE_LOGICAL_OR),
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
        TokenKind::EqualEqual => Some(BinaryOperator::Equal),
        TokenKind::BangEqual => Some(BinaryOperator::NotEqual),
        TokenKind::LessThan => Some(BinaryOperator::LessThan),
        TokenKind::GreaterThan => Some(BinaryOperator::GreaterThan),
        TokenKind::LessEqual => Some(BinaryOperator::LessEqual),
        TokenKind::GreaterEqual => Some(BinaryOperator::GreaterEqual),
        TokenKind::AndAnd => Some(BinaryOperator::LogicalAnd),
        TokenKind::OrOr => Some(BinaryOperator::LogicalOr),
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
    /// binary_op → "+" | "-" | "*" | "/" | "%" | "==" | "!=" | "<" | ">" | "<=" | ">="
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

            // Parse the right-hand side with `precedence - 1` for left-associativity
            // of binary operators. This makes the current operator bind tighter than
            // itself, so `a - b - c` parses as `(a - b) - c` rather than `a - (b - c)`.
            // Note: Unary operators are handled in parse_primary_expr() and are
            // right-associative (e.g., `--5` parses as `-(-5)`).
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
    /// - Unary operators (`-`, `!`)
    /// - Integer literals
    /// - String literals
    /// - Identifiers (variable references)
    /// - Function calls
    /// - Parenthesized expressions
    fn parse_primary_expr(&mut self) -> Result<Expr, ParseError> {
        let start_span = self.current_span();

        match self.current_kind() {
            TokenKind::Minus | TokenKind::Bang => {
                // Unary negation or logical NOT
                let op = if matches!(self.current_kind(), TokenKind::Minus) {
                    UnaryOperator::Neg
                } else {
                    UnaryOperator::Not
                };
                self.advance();
                self.skip_newlines(); // allow newlines after operator

                // Fold negation into integer literal to allow i64::MIN
                if matches!(op, UnaryOperator::Neg)
                    && let TokenKind::IntLiteral(unsigned_value) = self.current_kind()
                {
                    let unsigned_value = *unsigned_value;
                    let literal_span = self.current_span();
                    self.advance(); // consume the literal

                    let span = Span::new(
                        start_span.start,
                        literal_span.end,
                        start_span.line,
                        start_span.column,
                    );

                    let signed_value = if unsigned_value <= i64::MAX as u64 {
                        -(unsigned_value as i64)
                    } else if unsigned_value == i64::MIN.unsigned_abs() {
                        i64::MIN
                    } else {
                        return Err(ParseError::integer_literal_out_of_range_negative(
                            unsigned_value,
                            span,
                        ));
                    };

                    return Ok(Expr::new(ExprKind::IntLiteral(signed_value), span));
                }

                // Not a literal — parse as normal unary operation
                let operand = self.parse_expr_pratt(PRECEDENCE_UNARY)?;

                let span = Span::new(
                    start_span.start,
                    operand.span.end,
                    start_span.line,
                    start_span.column,
                );

                Ok(Expr::new(
                    ExprKind::UnaryOp {
                        op,
                        operand: Box::new(operand),
                    },
                    span,
                ))
            }
            TokenKind::LeftParen => {
                // Parenthesized expression
                self.advance(); // consume '('
                self.skip_newlines();

                let inner = self.parse_expr()?;

                self.skip_newlines();
                // Store span before consuming to avoid index issues
                let close_paren_span = self.current_span();
                self.expect(&TokenKind::RightParen)?;

                // Return the inner expression with updated span covering the parens
                let span = Span::new(
                    start_span.start,
                    close_paren_span.end,
                    start_span.line,
                    start_span.column,
                );
                Ok(Expr::new(inner.kind, span))
            }
            TokenKind::Identifier(name) => {
                let name = name.clone();
                self.advance();

                // Parse member access (e.g., module.function)
                // Nested member access (e.g., a.b.c) is detected and rejected with an error
                let mut expr = Expr::new(ExprKind::Identifier(name.clone()), start_span);

                while matches!(self.current_kind(), TokenKind::Dot) {
                    // Check if we're creating a nested member access
                    if matches!(expr.kind, ExprKind::MemberAccess { .. }) {
                        // Nested member access (e.g., a.b.c) is not yet supported
                        // Return error early with span covering the entire expression
                        let span = Span::new(
                            start_span.start,
                            self.current_span().end,
                            start_span.line,
                            start_span.column,
                        );
                        return Err(ParseError::nested_member_access_not_supported(span));
                    }

                    self.advance(); // consume '.'

                    // Expect identifier after dot
                    // Store span before consuming to avoid index issues
                    let member_span = self.current_span();
                    let member = self.expect_identifier()?;

                    let span = Span::new(
                        start_span.start,
                        member_span.end,
                        start_span.line,
                        start_span.column,
                    );

                    expr = Expr::new(
                        ExprKind::MemberAccess {
                            object: Box::new(expr),
                            member,
                        },
                        span,
                    );
                }

                // Check if this is a function call
                if matches!(self.current_kind(), TokenKind::LeftParen) {
                    // Extract callee name for function call
                    match &expr.kind {
                        ExprKind::Identifier(callee) => {
                            let callee = callee.clone();
                            self.parse_call(callee, start_span)
                        }
                        ExprKind::MemberAccess { .. } => {
                            // Module-qualified function call (e.g., math.add(1, 2))
                            self.parse_member_call(expr, start_span)
                        }
                        _ => Err(ParseError::internal(
                            "Internal parser error: unexpected expression kind in function call position. This is a compiler bug, please report it.",
                            expr.span,
                        )),
                    }
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
                        // means this is a valid expression (identifier or member access)
                        _ => Ok(expr),
                    }
                }
            }
            TokenKind::StringLiteral(value) => {
                let value = value.clone();
                self.advance();
                Ok(Expr::new(ExprKind::StringLiteral(value), start_span))
            }
            TokenKind::IntLiteral(unsigned_value) => {
                let unsigned_value = *unsigned_value;
                self.advance();

                if unsigned_value > i64::MAX as u64 {
                    return Err(ParseError::integer_literal_out_of_range_positive(
                        unsigned_value,
                        start_span,
                    ));
                }

                Ok(Expr::new(
                    ExprKind::IntLiteral(unsigned_value as i64),
                    start_span,
                ))
            }
            TokenKind::BoolLiteral(value) => {
                let value = *value;
                self.advance();
                Ok(Expr::new(ExprKind::BoolLiteral(value), start_span))
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

    /// Parses a module-qualified function call expression (e.g., `math.add(1, 2)`).
    ///
    /// The member access expression has already been parsed. This method parses
    /// the argument list within parentheses and converts the member access into
    /// a regular function call with a dot-separated callee name.
    ///
    /// # Arguments
    ///
    /// * `member_expr` - The member access expression (e.g., `math.add`)
    /// * `start_span` - The span from the start of the expression
    ///
    /// # Note
    ///
    /// This method is named `parse_member_call` for future compatibility with
    /// struct method calls. Currently, it only handles module-qualified calls.
    fn parse_member_call(
        &mut self,
        member_expr: Expr,
        start_span: Span,
    ) -> Result<Expr, ParseError> {
        self.expect(&TokenKind::LeftParen)?;
        self.skip_newlines();

        let mut args = Vec::new();

        if !matches!(self.current_kind(), TokenKind::RightParen) {
            loop {
                let arg = self.parse_expr()?;
                args.push(arg);
                self.skip_newlines();

                if matches!(self.current_kind(), TokenKind::Comma) {
                    self.advance();
                    self.skip_newlines();
                } else {
                    break;
                }
            }
        }

        self.skip_newlines();
        let end_span = self.current_span();
        self.expect(&TokenKind::RightParen)?;

        let span = Span::new(
            start_span.start,
            end_span.end,
            start_span.line,
            start_span.column,
        );

        // Module-qualified function calls (e.g., math.add()) are represented as
        // ModuleCall nodes with separate module and function fields. This allows
        // the semantic analyzer to properly validate module access.
        if let ExprKind::MemberAccess { object, member } = member_expr.kind {
            let module = match object.kind {
                ExprKind::Identifier(ref module_name) => module_name.clone(),
                ExprKind::MemberAccess { .. } => {
                    // Nested member access (e.g., a.b.c) is not yet supported
                    return Err(ParseError::nested_member_access_not_supported(span));
                }
                _ => {
                    return Err(ParseError::internal(
                        "Internal parser error: unexpected object kind in member access. This is a compiler bug, please report it.",
                        span,
                    ));
                }
            };

            Ok(Expr::new(
                ExprKind::ModuleCall {
                    module,
                    function: member,
                    args,
                },
                span,
            ))
        } else {
            Err(ParseError::internal(
                "Internal parser error: parse_member_call called with non-MemberAccess expression. This is a compiler bug, please report it.",
                member_expr.span,
            ))
        }
    }
}
