//! Statement parsing.

use super::Parser;
use super::error::ParseError;
use crate::ast::{Stmt, StmtKind};
use crate::token::{Span, TokenKind};

impl Parser {
    /// Parses a single statement.
    ///
    /// # Grammar
    ///
    /// ```text
    /// stmt → let_stmt | expr_stmt
    /// ```
    pub(super) fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        match self.current_kind() {
            TokenKind::Let => self.parse_let_stmt(),
            _ => {
                let expr = self.parse_expr()?;
                let span = expr.span;
                Ok(Stmt::new(StmtKind::Expr(expr), span))
            }
        }
    }

    /// Parses a let statement.
    ///
    /// # Grammar
    ///
    /// ```text
    /// let_stmt → "let" IDENTIFIER ":" type "=" expr
    /// type → "i32" | "i64"
    /// ```
    pub(super) fn parse_let_stmt(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();

        // Expect `let`
        self.expect(&TokenKind::Let)?;

        // Expect variable name
        let name = self.expect_identifier()?;

        // Expect `:` type annotation
        self.expect(&TokenKind::Colon)?;
        let ty = self.parse_type()?;

        // Expect `=` initializer
        self.expect(&TokenKind::Equals)?;
        let init = self.parse_expr()?;

        // Span covers from 'let' to end of initializer expression
        let span = Span::new(
            start_span.start,
            init.span.end,
            start_span.line,
            start_span.column,
        );

        Ok(Stmt::new(StmtKind::Let { name, ty, init }, span))
    }
}
