//! Expression nodes for the Lak AST.

use crate::token::Span;
use std::fmt;

/// Unary operators.
///
/// These operators are used in unary expressions like `-x`.
/// Unary operators have the highest precedence (level 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    /// Negation operator `-`
    Neg,
}

impl fmt::Display for UnaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOperator::Neg => write!(f, "-"),
        }
    }
}

/// Binary operators for arithmetic operations.
///
/// These operators are used in binary expressions like `a + b` or `x * y`.
/// All operators are left-associative with standard arithmetic precedence:
/// - Multiplicative operators (`*`, `/`, `%`) have higher precedence
/// - Additive operators (`+`, `-`) have lower precedence
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    /// Addition operator `+`
    Add,
    /// Subtraction operator `-`
    Sub,
    /// Multiplication operator `*`
    Mul,
    /// Division operator `/`
    Div,
    /// Modulo (remainder) operator `%`
    Mod,
}

impl fmt::Display for BinaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryOperator::Add => write!(f, "+"),
            BinaryOperator::Sub => write!(f, "-"),
            BinaryOperator::Mul => write!(f, "*"),
            BinaryOperator::Div => write!(f, "/"),
            BinaryOperator::Mod => write!(f, "%"),
        }
    }
}

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

    /// A boolean literal value (`true` or `false`).
    BoolLiteral(bool),

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

    /// A binary operation expression.
    ///
    /// Represents expressions like `a + b`, `x * y`, etc.
    /// The left and right operands are boxed to avoid infinite
    /// size due to recursive type definition.
    BinaryOp {
        /// The left operand.
        left: Box<Expr>,
        /// The operator.
        op: BinaryOperator,
        /// The right operand.
        right: Box<Expr>,
    },

    /// A unary operation expression.
    ///
    /// Represents expressions like `-x`.
    /// The operand is boxed to avoid infinite size due to recursive type definition.
    UnaryOp {
        /// The operator.
        op: UnaryOperator,
        /// The operand.
        operand: Box<Expr>,
    },

    /// A member access expression.
    ///
    /// Represents expressions like `module.function` for module-qualified
    /// function access. In the future, this will also be used for struct
    /// field access.
    ///
    /// Note: Tuple access uses numeric indices (`.0`, `.1`) which are parsed
    /// differently and won't conflict with this.
    MemberAccess {
        /// The object being accessed (e.g., module name).
        object: Box<Expr>,
        /// The member name being accessed (e.g., function name).
        member: String,
    },

    /// A module-qualified function call.
    ///
    /// Represents expressions like `module.function(args)` where `module`
    /// is an imported module name. This is distinct from regular `Call` to
    /// allow semantic analysis to properly validate module function access.
    ModuleCall {
        /// The module name (e.g., "math").
        module: String,
        /// The function name (e.g., "sqrt").
        function: String,
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

    /// Creates a member access expression for testing purposes.
    ///
    /// This is a convenience method for creating `MemberAccess` expressions
    /// in tests without needing to construct the full AST manually.
    ///
    /// # Arguments
    ///
    /// * `object` - The name of the object being accessed (e.g., module name)
    /// * `member` - The member name being accessed (e.g., function name)
    ///
    /// # Panics
    ///
    /// Panics if `object` or `member` is empty.
    #[cfg(test)]
    pub fn member_access_for_testing(object: &str, member: &str) -> Self {
        assert!(!object.is_empty(), "object name cannot be empty");
        assert!(!member.is_empty(), "member name cannot be empty");

        let dummy_span = Span::new(0, 0, 1, 1);
        let object_expr = Expr::new(ExprKind::Identifier(object.to_string()), dummy_span);

        Expr::new(
            ExprKind::MemberAccess {
                object: Box::new(object_expr),
                member: member.to_string(),
            },
            dummy_span,
        )
    }

    /// Creates a module call expression for testing purposes.
    ///
    /// # Arguments
    ///
    /// * `module` - The module name
    /// * `function` - The function name
    /// * `args` - The arguments (empty vector for no args)
    #[cfg(test)]
    pub fn module_call_for_testing(module: &str, function: &str) -> Self {
        assert!(!module.is_empty(), "module name cannot be empty");
        assert!(!function.is_empty(), "function name cannot be empty");

        let dummy_span = Span::new(0, 0, 1, 1);
        Expr::new(
            ExprKind::ModuleCall {
                module: module.to_string(),
                function: function.to_string(),
                args: Vec::new(),
            },
            dummy_span,
        )
    }
}
