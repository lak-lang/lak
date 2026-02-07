//! Type annotation parsing.

use super::Parser;
use super::error::ParseError;
use crate::ast::Type;

impl Parser {
    /// Parses a type annotation.
    ///
    /// # Grammar
    ///
    /// ```text
    /// type → "i32" | "i64" | "string" | "bool"
    /// ```
    pub(super) fn parse_type(&mut self) -> Result<Type, ParseError> {
        let type_span = self.current_span();
        let name = self.expect_identifier()?;
        match name.as_str() {
            "i32" => Ok(Type::I32),
            "i64" => Ok(Type::I64),
            "string" => Ok(Type::String),
            "bool" => Ok(Type::Bool),
            _ => Err(ParseError::unknown_type(&name, type_span)),
        }
    }
}
