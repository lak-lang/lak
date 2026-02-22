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
    /// type → "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "byte" | "string" | "bool"
    /// ```
    pub(super) fn parse_type(&mut self) -> Result<Type, ParseError> {
        let type_span = self.current_span();
        let name = self.expect_identifier()?;
        match name.as_str() {
            "i8" => Ok(Type::I8),
            "i16" => Ok(Type::I16),
            "i32" => Ok(Type::I32),
            "i64" => Ok(Type::I64),
            "u8" | "byte" => Ok(Type::U8),
            "u16" => Ok(Type::U16),
            "u32" => Ok(Type::U32),
            "u64" => Ok(Type::U64),
            "string" => Ok(Type::String),
            "bool" => Ok(Type::Bool),
            _ => Err(ParseError::unknown_type(&name, type_span)),
        }
    }
}
