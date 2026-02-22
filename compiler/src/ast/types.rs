//! Type annotations for variable declarations.

use std::fmt;

/// A type annotation in variable declarations.
///
/// This enum represents the types that can be specified in Lak code.
/// Currently supports integer primitives, strings, and booleans.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    /// 8-bit signed integer type (`i8` in Lak source code).
    I8,
    /// 16-bit signed integer type (`i16` in Lak source code).
    I16,
    /// 32-bit signed integer type (`i32` in Lak source code).
    I32,
    /// 64-bit signed integer type (`i64` in Lak source code).
    I64,
    /// 8-bit unsigned integer type (`u8` in Lak source code).
    U8,
    /// 16-bit unsigned integer type (`u16` in Lak source code).
    U16,
    /// 32-bit unsigned integer type (`u32` in Lak source code).
    U32,
    /// 64-bit unsigned integer type (`u64` in Lak source code).
    U64,
    /// UTF-8 string type (`string` in Lak source code).
    String,
    /// Boolean type (`bool` in Lak source code).
    Bool,
}

impl Type {
    /// Returns true when this type is one of Lak's integer primitives.
    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            Type::I8
                | Type::I16
                | Type::I32
                | Type::I64
                | Type::U8
                | Type::U16
                | Type::U32
                | Type::U64
        )
    }

    /// Returns true when this type is a signed integer primitive.
    pub fn is_signed_integer(&self) -> bool {
        matches!(self, Type::I8 | Type::I16 | Type::I32 | Type::I64)
    }

    /// Returns true when this type is an unsigned integer primitive.
    pub fn is_unsigned_integer(&self) -> bool {
        matches!(self, Type::U8 | Type::U16 | Type::U32 | Type::U64)
    }
}

/// Displays the type as it would appear in Lak source code.
///
/// This is used for generating user-facing error messages.
impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::I8 => write!(f, "i8"),
            Type::I16 => write!(f, "i16"),
            Type::I32 => write!(f, "i32"),
            Type::I64 => write!(f, "i64"),
            Type::U8 => write!(f, "u8"),
            Type::U16 => write!(f, "u16"),
            Type::U32 => write!(f, "u32"),
            Type::U64 => write!(f, "u64"),
            Type::String => write!(f, "string"),
            Type::Bool => write!(f, "bool"),
        }
    }
}
