//! Expression parsing.

use super::Parser;
use super::error::ParseError;
use crate::ast::{Expr, ExprKind};
use crate::token::{Span, TokenKind};

impl Parser {
    /// Parses an expression.
    ///
    /// Handles identifiers (function calls or variable references),
    /// string literals, and integer literals.
    ///
    /// # Grammar
    ///
    /// ```text
    /// expr → call | IDENTIFIER | STRING | INT
    /// call → IDENTIFIER "(" arguments? ")"
    /// ```
    pub(super) fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        let start_span = self.current_span();

        match self.current_kind() {
            TokenKind::Identifier(name) => {
                let name = name.clone();
                self.advance();
                if matches!(self.current_kind(), TokenKind::LeftParen) {
                    self.parse_call(name, start_span)
                } else {
                    // Check for syntax error: identifier followed by expression-start token
                    // without an intervening Newline (which would indicate a new statement)
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
