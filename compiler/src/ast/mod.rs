//! Abstract Syntax Tree definitions for the Lak programming language.
//!
//! This module defines the data structures that represent parsed Lak programs.
//! The AST is produced by the [`crate::parser`], validated by the [`crate::semantic`],
//! and consumed by the [`crate::codegen`].
//!
//! # Structure
//!
//! The AST has a hierarchical structure:
//! - [`Program`] - The root node containing all function definitions
//! - [`FnDef`] - A function definition with name, return type, and body
//! - [`Stmt`] - Individual statements (expression statements and let declarations)
//! - [`Expr`] - Expressions (string literals, integer literals, identifiers, and function calls)
//! - [`Type`] - Type annotations for variable declarations
//!
//! Each AST node includes source location information ([`Span`](crate::token::Span)) for error reporting.
//!
//! # Module Structure
//!
//! - [`types`] - Type annotations (i32, i64)
//! - [`expr`] - Expression nodes and kinds
//! - [`stmt`] - Statement nodes and kinds
//! - [`program`] - Top-level program structure (Program, FnDef)
//!
//! # See Also
//!
//! * [`crate::parser`] - Produces the AST from tokens
//! * [`crate::semantic`] - Validates the AST
//! * [`crate::codegen`] - Generates LLVM IR from the AST

mod expr;
mod program;
mod stmt;
mod types;

#[cfg(test)]
mod tests;

pub use expr::{Expr, ExprKind};
pub use program::{FnDef, Program};
pub use stmt::{Stmt, StmtKind};
pub use types::Type;
