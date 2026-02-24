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
    /// type → "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "f32" | "f64" | "byte" | "string" | "bool"
    /// ```
    pub(super) fn parse_type(&mut self) -> Result<Type, ParseError> {
        let type_span = self.current_span();
        let name = self.expect_identifier()?;
        Type::from_source_name(&name).ok_or_else(|| ParseError::unknown_type(&name, type_span))
    }
}
