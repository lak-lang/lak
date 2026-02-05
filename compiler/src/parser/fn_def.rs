//! Function definition parsing.

use super::Parser;
use super::error::ParseError;
use crate::ast::FnDef;
use crate::token::TokenKind;

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
        // Skip any leading newlines before function definition
        self.skip_newlines();

        // Expect `fn` keyword
        self.expect(&TokenKind::Fn)?;

        // Expect function name (identifier)
        let name = self.expect_identifier()?;

        // Expect `(` `)`
        self.expect(&TokenKind::LeftParen)?;
        self.expect(&TokenKind::RightParen)?;

        // Expect `->` return_type
        self.expect(&TokenKind::Arrow)?;
        let return_type = self.expect_identifier()?;

        // Expect `{` body `}`
        self.expect(&TokenKind::LeftBrace)?;
        self.skip_newlines(); // Skip newlines after opening brace

        let mut body = Vec::new();
        while !matches!(self.current_kind(), TokenKind::RightBrace) && !self.is_eof() {
            let stmt = self.parse_stmt()?;
            body.push(stmt);
            self.skip_newlines(); // Skip newlines between statements
        }

        self.expect(&TokenKind::RightBrace)?;

        Ok(FnDef {
            name,
            return_type,
            body,
        })
    }
}
