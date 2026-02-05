//! Type annotations for variable declarations.

use std::fmt;

/// A type annotation in variable declarations.
///
/// This enum represents the types that can be specified in Lak code.
/// Currently supports basic integer types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    /// 32-bit signed integer type (`i32` in Lak source code).
    I32,
    /// 64-bit signed integer type (`i64` in Lak source code).
    I64,
}

/// Displays the type as it would appear in Lak source code.
///
/// This is used for generating user-facing error messages.
impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::I32 => write!(f, "i32"),
            Type::I64 => write!(f, "i64"),
        }
    }
}
