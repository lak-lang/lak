//! Function definition parsing.

use super::Parser;
use super::error::ParseError;
use crate::ast::FnDef;
use crate::token::{Span, TokenKind};

impl Parser {
    /// Parses a function definition.
    ///
    /// # Grammar
    ///
    /// ```text
    /// fn_def → "fn" IDENTIFIER "(" ")" "->" IDENTIFIER "{" stmt* "}"
    /// ```
    ///
    /// Currently only parameterless functions are supported.
    pub(super) fn parse_fn_def(&mut self) -> Result<FnDef, ParseError> {
        // Record start position for span
        let start_span = self.current().span;

        // Expect `fn` keyword
        self.expect(&TokenKind::Fn)?;

        // Expect function name (identifier)
        let name = self.expect_identifier()?;

        // Expect `(` `)`
        self.expect(&TokenKind::LeftParen)?;
        self.expect(&TokenKind::RightParen)?;

        // Expect `->` return_type
        self.expect(&TokenKind::Arrow)?;
        // Capture return type span before consuming the token
        let return_type_span = self.current().span;
        let return_type = self.expect_identifier()?;

        // Record end position (before `{`) for span
        let end_span = self.current().span;

        // Expect `{` body `}`
        self.expect(&TokenKind::LeftBrace)?;
        self.skip_newlines(); // Skip newlines after opening brace

        let mut body = Vec::new();
        while !matches!(self.current_kind(), TokenKind::RightBrace) && !self.is_eof() {
            let stmt = self.parse_stmt()?;
            body.push(stmt);
            self.expect_statement_terminator()?;
        }

        self.expect(&TokenKind::RightBrace)?;

        // Create span from `fn` to just before `{`
        let span = Span {
            start: start_span.start,
            end: end_span.start,
            line: start_span.line,
            column: start_span.column,
        };

        Ok(FnDef {
            name,
            return_type,
            return_type_span,
            body,
            span,
        })
    }
}
