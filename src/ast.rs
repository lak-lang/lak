//! Abstract Syntax Tree definitions for the Lak programming language.
//!
//! This module defines the data structures that represent parsed Lak programs.
//! The AST is produced by the [`crate::parser`] and consumed by the [`crate::codegen`].
//!
//! # Structure
//!
//! The AST has a hierarchical structure:
//! - [`Program`] - The root node containing all statements
//! - [`Stmt`] - Individual statements (currently only expression statements)
//! - [`Expr`] - Expressions (string literals and function calls)
//!
//! # See Also
//!
//! * [`crate::parser`] - Produces the AST from tokens
//! * [`crate::codegen`] - Generates LLVM IR from the AST

/// An expression in the Lak language.
///
/// Expressions are the building blocks of Lak programs. They can be
/// evaluated to produce values or trigger side effects (in the case
/// of function calls).
#[derive(Debug, Clone)]
pub enum Expr {
    /// A string literal value.
    ///
    /// The contained `String` is the unescaped content of the literal
    /// (escape sequences have already been processed by the lexer).
    StringLiteral(String),

    /// A function call expression.
    ///
    /// # Fields
    ///
    /// * `callee` - The name of the function being called
    /// * `args` - The list of argument expressions passed to the function
    Call {
        /// The name of the function being called.
        callee: String,
        /// The arguments passed to the function.
        args: Vec<Expr>,
    },
}

/// A statement in the Lak language.
///
/// Statements are the top-level constructs that make up a program.
/// Currently, Lak only supports expression statements, but this enum
/// provides a foundation for adding more statement types in the future
/// (e.g., variable declarations, control flow).
#[derive(Debug, Clone)]
pub enum Stmt {
    /// An expression statement.
    ///
    /// Evaluates the expression for its side effects. The result value
    /// (if any) is discarded.
    Expr(Expr),
}

/// The root node of a Lak program's AST.
///
/// A `Program` contains a sequence of statements that are executed
/// in order when the program runs.
///
/// # Examples
///
/// A simple program with one `println` call:
///
/// ```text
/// // Lak source code:
/// println("Hello, world!")
///
/// // Corresponding AST:
/// Program {
///     stmts: vec![
///         Stmt::Expr(Expr::Call {
///             callee: "println".to_string(),
///             args: vec![Expr::StringLiteral("Hello, world!".to_string())],
///         }),
///     ],
/// }
/// ```
#[derive(Debug)]
pub struct Program {
    /// The statements that make up this program.
    pub stmts: Vec<Stmt>,
}
