//! Top-level program structure for the Lak AST.

use crate::token::Span;

use super::stmt::Stmt;

/// A function definition in the Lak language.
///
/// Functions are the primary organizational unit in Lak. Every program
/// must have a `main` function as its entry point.
///
/// # Invariants
///
/// The following invariants should hold for a well-formed `FnDef`:
/// - `name` should be a non-empty valid identifier
/// - `return_type` should be a valid type name (currently only "void")
/// - `return_type_span` should point to the actual return type token in source
/// - `span` should encompass the function signature from `fn` to before `{`
/// - `span.start <= span.end` (valid span range)
///
/// These invariants are enforced by the parser. Direct construction should
/// only be done in tests using [`FnDef::for_testing`].
///
/// # Examples
///
/// ```text
/// fn main() -> void {
///     println("Hello, world!")
/// }
/// ```
#[derive(Debug, Clone)]
pub struct FnDef {
    /// The name of the function.
    pub name: String,
    /// The return type of the function. Currently only "void" is supported.
    pub return_type: String,
    /// The source location of the return type token (e.g., `void` or `int`).
    pub return_type_span: Span,
    /// The statements that make up the function body.
    pub body: Vec<Stmt>,
    /// The source location of the function definition (from `fn` to `{`).
    pub span: Span,
}

impl FnDef {
    /// Creates a `FnDef` for testing purposes with dummy spans.
    ///
    /// This constructor is intended for unit tests where span information
    /// is not relevant. For production code, `FnDef` should be constructed
    /// by the parser which provides accurate span information.
    ///
    /// # Arguments
    ///
    /// * `name` - The function name
    /// * `return_type` - The return type (e.g., "void")
    /// * `body` - The function body statements
    #[cfg(test)]
    pub fn for_testing(name: &str, return_type: &str, body: Vec<Stmt>) -> Self {
        let dummy = Span::new(0, 0, 1, 1);
        FnDef {
            name: name.to_string(),
            return_type: return_type.to_string(),
            return_type_span: dummy,
            body,
            span: dummy,
        }
    }
}

/// The root node of a Lak program's AST.
///
/// A `Program` contains a collection of function definitions.
/// Every valid program must have at least a `main` function.
///
/// # Examples
///
/// A simple program with a `main` function:
///
/// ```text
/// // Lak source code:
/// fn main() -> void {
///     println("Hello, world!")
/// }
/// ```
#[derive(Debug)]
pub struct Program {
    /// The function definitions in this program.
    pub functions: Vec<FnDef>,
}
