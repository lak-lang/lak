//! Type annotations for variable declarations.

use std::fmt;

/// A type annotation in variable declarations.
///
/// This enum represents the types that can be specified in Lak code.
/// Currently supports integer primitives, floating-point primitives, strings,
/// booleans, and an internal inference placeholder (`Type::Inferred`).
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
    /// 32-bit floating-point type (`f32` in Lak source code).
    F32,
    /// 64-bit floating-point type (`f64` in Lak source code).
    F64,
    /// UTF-8 string type (`string` in Lak source code).
    String,
    /// Boolean type (`bool` in Lak source code).
    Bool,
    /// Type to be inferred from initializer expression (`let x = ...`).
    ///
    /// This variant is an AST-level placeholder created by the parser.
    /// Semantic analysis resolves inferred bindings in symbol metadata.
    /// Codegen resolves placeholders where needed, and safety-net guards
    /// reject unresolved placeholders that reach low-level type mapping.
    /// Use `Type::is_resolved()` for invariant checks instead of equality
    /// comparisons against concrete variants.
    Inferred,
}

impl Type {
    /// Decodes a source-level type name into a [`Type`].
    ///
    /// `Type::Inferred` is intentionally excluded because it is an internal
    /// AST placeholder, not a source-level type keyword.
    pub(crate) fn from_source_name(name: &str) -> Option<Self> {
        match name {
            "i8" => Some(Self::I8),
            "i16" => Some(Self::I16),
            "i32" => Some(Self::I32),
            "i64" => Some(Self::I64),
            "u8" | "byte" => Some(Self::U8),
            "u16" => Some(Self::U16),
            "u32" => Some(Self::U32),
            "u64" => Some(Self::U64),
            "f32" => Some(Self::F32),
            "f64" => Some(Self::F64),
            "string" => Some(Self::String),
            "bool" => Some(Self::Bool),
            _ => None,
        }
    }

    /// Decodes a source-level function return type.
    ///
    /// `None` inside `Some` represents `void`.
    pub(crate) fn from_function_return_name(name: &str) -> Option<Option<Self>> {
        if name == "void" {
            Some(None)
        } else {
            Self::from_source_name(name).map(Some)
        }
    }

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

    /// Returns true when this type is one of Lak's floating-point primitives.
    pub fn is_float(&self) -> bool {
        matches!(self, Type::F32 | Type::F64)
    }

    /// Returns true when this type is numeric (integer or floating-point).
    pub fn is_numeric(&self) -> bool {
        self.is_integer() || self.is_float()
    }

    /// Returns true when this type is concrete and safe for backend mapping.
    pub fn is_resolved(&self) -> bool {
        !matches!(self, Type::Inferred)
    }
}

/// Displays a human-readable type label for diagnostics.
///
/// Concrete variants use Lak source syntax (e.g. `i32`, `string`), while
/// internal placeholders use explicit markers (e.g. `<inferred>`).
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
            Type::F32 => write!(f, "f32"),
            Type::F64 => write!(f, "f64"),
            Type::String => write!(f, "string"),
            Type::Bool => write!(f, "bool"),
            // Keep internal placeholders visually explicit in diagnostics.
            Type::Inferred => write!(f, "<inferred>"),
        }
    }
}
