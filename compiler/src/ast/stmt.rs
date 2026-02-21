//! Statement nodes for the Lak AST.

use crate::token::Span;

use super::expr::Expr;
use super::types::Type;

/// The kind of a statement in the Lak language.
///
/// This enum represents the different types of statements without
/// source location information. Use [`Stmt`] for the full AST node
/// with span information.
#[derive(Debug, Clone)]
pub enum StmtKind {
    /// An expression statement.
    ///
    /// Evaluates the expression for its side effects. The result value
    /// (if any) is discarded.
    Expr(Expr),

    /// A variable declaration with `let`.
    ///
    /// Declares a new variable with an explicit type annotation and
    /// initializer expression.
    Let {
        /// Whether this binding is declared as mutable (`let mut`).
        is_mutable: bool,
        /// The name of the variable being declared.
        name: String,
        /// The type annotation for the variable.
        ty: Type,
        /// The initializer expression.
        init: Expr,
    },

    /// A return statement.
    ///
    /// `return` without a value is represented as `None`.
    /// `return expr` is represented as `Some(expr)`.
    Return(Option<Expr>),

    /// Explicitly discards an expression result.
    ///
    /// This corresponds to `let _ = expr` and is used to acknowledge
    /// intentionally ignored return values.
    Discard(Expr),

    /// A conditional statement with optional `else` branch.
    ///
    /// The `else if` chain is represented as an `else_branch` containing
    /// a single nested `StmtKind::If`.
    If {
        /// The condition expression. Must evaluate to `bool`.
        condition: Expr,
        /// Statements executed when the condition is true.
        then_branch: Vec<Stmt>,
        /// Optional statements executed when the condition is false.
        else_branch: Option<Vec<Stmt>>,
    },

    /// A while loop statement.
    ///
    /// Repeatedly executes `body` while `condition` evaluates to true.
    While {
        /// The loop condition. Must evaluate to `bool`.
        condition: Expr,
        /// Statements executed for each iteration.
        body: Vec<Stmt>,
    },

    /// Exits the innermost enclosing loop.
    Break,

    /// Skips to the next iteration of the innermost enclosing loop.
    Continue,
}

/// A statement in the Lak language with source location.
///
/// Statements are constructs within function bodies.
#[derive(Debug, Clone)]
pub struct Stmt {
    /// The kind of statement.
    pub kind: StmtKind,
    /// The source location of this statement.
    pub span: Span,
}

impl Stmt {
    /// Creates a new statement with the given kind and span.
    pub fn new(kind: StmtKind, span: Span) -> Self {
        Stmt { kind, span }
    }
}
