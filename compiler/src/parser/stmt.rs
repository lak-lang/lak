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
    /// stmt → let_stmt | if_stmt | expr_stmt
    /// ```
    pub(super) fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        match self.current_kind() {
            TokenKind::Let => self.parse_let_stmt(),
            TokenKind::If => self.parse_if_stmt(),
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

    /// Parses an if statement.
    ///
    /// # Grammar
    ///
    /// ```text
    /// if_stmt → "if" expr "{" stmt* "}" ("else" (if_stmt | "{" stmt* "}"))?
    /// ```
    pub(super) fn parse_if_stmt(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.expect(&TokenKind::If)?;

        let condition = self.parse_expr()?;
        let then_branch = self.parse_block_stmts()?;

        let else_branch = if self.consume_newlines_before_else() {
            self.expect(&TokenKind::Else)?;

            if matches!(self.current_kind(), TokenKind::If) {
                let nested_if = self.parse_if_stmt()?;
                Some(vec![nested_if])
            } else {
                Some(self.parse_block_stmts()?)
            }
        } else {
            None
        };

        let end = else_branch
            .as_ref()
            .and_then(|branch| branch.last())
            .map(|stmt| stmt.span.end)
            .unwrap_or_else(|| {
                then_branch
                    .last()
                    .map(|stmt| stmt.span.end)
                    .unwrap_or(condition.span.end)
            });
        let span = Span::new(start_span.start, end, start_span.line, start_span.column);

        Ok(Stmt::new(
            StmtKind::If {
                condition,
                then_branch,
                else_branch,
            },
            span,
        ))
    }

    pub(super) fn parse_block_stmts(&mut self) -> Result<Vec<Stmt>, ParseError> {
        self.expect(&TokenKind::LeftBrace)?;
        self.skip_newlines();

        let mut body = Vec::new();
        while !matches!(self.current_kind(), TokenKind::RightBrace) && !self.is_eof() {
            let stmt = self.parse_stmt()?;
            body.push(stmt);
            self.expect_statement_terminator()?;
        }

        self.expect(&TokenKind::RightBrace)?;
        Ok(body)
    }

    /// If the next non-newline token is `else`, consumes preceding newlines and
    /// positions the parser at `else`.
    pub(super) fn consume_newlines_before_else(&mut self) -> bool {
        let mut lookahead = self.pos;
        while lookahead < self.tokens.len()
            && matches!(self.tokens[lookahead].kind, TokenKind::Newline)
        {
            lookahead += 1;
        }

        if lookahead < self.tokens.len() && matches!(self.tokens[lookahead].kind, TokenKind::Else) {
            self.pos = lookahead;
            true
        } else {
            false
        }
    }
}
