//! Expression nodes for the Lak AST.

use crate::token::Span;

/// The kind of an expression in the Lak language.
///
/// This enum represents the different types of expressions without
/// source location information. Use [`Expr`] for the full AST node
/// with span information.
#[derive(Debug, Clone)]
pub enum ExprKind {
    /// A string literal value.
    ///
    /// The contained `String` is the unescaped content of the literal
    /// (escape sequences have already been processed by the lexer).
    StringLiteral(String),

    /// An integer literal value.
    ///
    /// The value is stored as i64 internally and converted to the
    /// appropriate type during code generation.
    IntLiteral(i64),

    /// A variable reference.
    ///
    /// Refers to a variable by name. The variable must be declared before
    /// use; this is verified during semantic analysis, not parsing.
    Identifier(String),

    /// A function call expression.
    Call {
        /// The name of the function being called.
        callee: String,
        /// The arguments passed to the function.
        args: Vec<Expr>,
    },
}

/// An expression in the Lak language with source location.
///
/// Expressions are the building blocks of Lak programs. They can be
/// evaluated to produce values or trigger side effects (in the case
/// of function calls).
#[derive(Debug, Clone)]
pub struct Expr {
    /// The kind of expression.
    pub kind: ExprKind,
    /// The source location of this expression.
    pub span: Span,
}

impl Expr {
    /// Creates a new expression with the given kind and span.
    pub fn new(kind: ExprKind, span: Span) -> Self {
        Expr { kind, span }
    }
}
