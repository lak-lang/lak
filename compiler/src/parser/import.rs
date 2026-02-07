//! Import declaration parsing.

use super::Parser;
use super::error::ParseError;
use crate::ast::ImportDecl;
use crate::token::{Span, TokenKind};

impl Parser {
    /// Parses an import declaration.
    ///
    /// # Grammar
    ///
    /// ```text
    /// import → "import" STRING ("as" IDENTIFIER)?
    /// ```
    ///
    /// # Examples
    ///
    /// ```text
    /// import "math"
    /// import "math/calc" as mc
    /// import "./utils"
    /// ```
    pub(super) fn parse_import(&mut self) -> Result<ImportDecl, ParseError> {
        let start_span = self.current().span;

        // Expect `import` keyword
        self.expect(&TokenKind::Import)?;

        // Expect string literal for path
        // Store span before consuming to avoid index issues
        let path_span = self.current_span();
        let path = self.expect_string_literal()?;

        // Validate that path is not empty
        if path.is_empty() {
            return Err(ParseError::empty_import_path(path_span));
        }

        // Check for optional `as` alias
        let (alias, end_span) = if matches!(self.current_kind(), TokenKind::As) {
            self.advance();
            // Store span before consuming to avoid index issues
            let alias_span = self.current_span();
            let alias_name = self.expect_identifier()?;
            // Defensive check: alias should not be empty
            if alias_name.is_empty() {
                return Err(ParseError::expected_identifier("empty alias", alias_span));
            }
            (Some(alias_name), alias_span)
        } else {
            (None, path_span)
        };

        // Calculate span from start to end
        let span = Span {
            start: start_span.start,
            end: end_span.end,
            line: start_span.line,
            column: start_span.column,
        };

        Ok(ImportDecl { path, alias, span })
    }
}
