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
    /// stmt → let_stmt | assign_stmt | return_stmt | if_stmt | while_stmt | break_stmt | continue_stmt | expr_stmt
    /// ```
    pub(super) fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        match self.current_kind() {
            TokenKind::Let => self.parse_let_stmt(),
            TokenKind::Return => self.parse_return_stmt(),
            TokenKind::If => self.parse_if_stmt(),
            TokenKind::While => self.parse_while_stmt(),
            TokenKind::Break => self.parse_break_stmt(),
            TokenKind::Continue => self.parse_continue_stmt(),
            _ => {
                let next_kind = self.tokens.get(self.pos + 1).map(|token| &token.kind);
                if matches!(self.current_kind(), TokenKind::Identifier(_))
                    && matches!(next_kind, Some(TokenKind::Equals))
                {
                    return self.parse_assign_stmt();
                }

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
    /// let_stmt → "let" "mut"? IDENTIFIER ":" type "=" expr | "let" "_" "=" expr
    /// type → integer/float primitives | "string" | "bool"
    /// ```
    pub(super) fn parse_let_stmt(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();

        // Expect `let`
        self.expect(&TokenKind::Let)?;

        // Optional `mut`
        let is_mutable = matches!(self.current_kind(), TokenKind::Mut);
        if is_mutable {
            self.advance();
        }

        // Expect variable name
        let name_span = self.current_span();
        let name = self.expect_identifier()?;

        // Special discard form: `let _ = expr` (`let mut _ = expr` is invalid)
        if name == "_" {
            if is_mutable {
                return Err(ParseError::invalid_mutable_discard(name_span));
            }

            if matches!(self.current_kind(), TokenKind::Equals) {
                self.expect(&TokenKind::Equals)?;
                let expr = self.parse_expr()?;
                let span = Span::new(
                    start_span.start,
                    expr.span.end,
                    start_span.line,
                    start_span.column,
                );
                return Ok(Stmt::new(StmtKind::Discard(expr), span));
            }

            if matches!(self.current_kind(), TokenKind::Colon) {
                return Err(ParseError::invalid_typed_discard(name_span));
            }

            return Err(ParseError::unexpected_token(
                &Self::token_kind_display(&TokenKind::Equals),
                &Self::token_kind_display(self.current_kind()),
                self.current_span(),
            ));
        }

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

        Ok(Stmt::new(
            StmtKind::Let {
                is_mutable,
                name,
                ty,
                init,
            },
            span,
        ))
    }

    /// Parses a reassignment statement.
    ///
    /// # Grammar
    ///
    /// ```text
    /// assign_stmt → IDENTIFIER "=" expr
    /// ```
    pub(super) fn parse_assign_stmt(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        let name = self.expect_identifier()?;
        self.expect(&TokenKind::Equals)?;
        let value = self.parse_expr()?;

        let span = Span::new(
            start_span.start,
            value.span.end,
            start_span.line,
            start_span.column,
        );

        Ok(Stmt::new(StmtKind::Assign { name, value }, span))
    }

    /// Parses a return statement.
    ///
    /// # Grammar
    ///
    /// ```text
    /// return_stmt → "return" expr?
    /// ```
    pub(super) fn parse_return_stmt(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.expect(&TokenKind::Return)?;

        // Bare return if statement ends immediately.
        if matches!(
            self.current_kind(),
            TokenKind::Newline | TokenKind::RightBrace | TokenKind::Eof
        ) {
            return Ok(Stmt::new(StmtKind::Return(None), start_span));
        }

        let value = self.parse_expr()?;
        let span = Span::new(
            start_span.start,
            value.span.end,
            start_span.line,
            start_span.column,
        );
        Ok(Stmt::new(StmtKind::Return(Some(value)), span))
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

    /// Parses a while statement.
    ///
    /// # Grammar
    ///
    /// ```text
    /// while_stmt → "while" expr "{" stmt* "}"
    /// ```
    pub(super) fn parse_while_stmt(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.expect(&TokenKind::While)?;

        let condition = self.parse_expr()?;
        let body = self.parse_block_stmts()?;

        let end = body
            .last()
            .map(|stmt| stmt.span.end)
            .unwrap_or(condition.span.end);
        let span = Span::new(start_span.start, end, start_span.line, start_span.column);

        Ok(Stmt::new(StmtKind::While { condition, body }, span))
    }

    /// Parses a break statement.
    ///
    /// # Grammar
    ///
    /// ```text
    /// break_stmt → "break"
    /// ```
    pub(super) fn parse_break_stmt(&mut self) -> Result<Stmt, ParseError> {
        let span = self.current_span();
        self.expect(&TokenKind::Break)?;
        Ok(Stmt::new(StmtKind::Break, span))
    }

    /// Parses a continue statement.
    ///
    /// # Grammar
    ///
    /// ```text
    /// continue_stmt → "continue"
    /// ```
    pub(super) fn parse_continue_stmt(&mut self) -> Result<Stmt, ParseError> {
        let span = self.current_span();
        self.expect(&TokenKind::Continue)?;
        Ok(Stmt::new(StmtKind::Continue, span))
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
