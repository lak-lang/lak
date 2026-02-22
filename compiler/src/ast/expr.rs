//! Expression nodes for the Lak AST.

use super::Type;
use crate::token::Span;
use std::fmt;

/// Unary operators.
///
/// These operators are used in unary expressions like `-x` and `!x`.
/// Unary operators have the highest precedence (level 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    /// Negation operator `-`
    Neg,
    /// Logical NOT operator `!`
    Not,
}

impl fmt::Display for UnaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOperator::Neg => write!(f, "-"),
            UnaryOperator::Not => write!(f, "!"),
        }
    }
}

/// Binary operators for arithmetic and comparison operations.
///
/// These operators are used in binary expressions like `a + b`, `x * y`, or `a < b`.
/// All operators are left-associative with standard precedence (tightest to loosest):
/// - Multiplicative operators (`*`, `/`, `%`)
/// - Additive operators (`+`, `-`)
/// - Comparison operators (`<`, `>`, `<=`, `>=`)
/// - Equality operators (`==`, `!=`)
/// - Logical AND (`&&`)
/// - Logical OR (`||`)
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
    /// Equal operator `==`
    Equal,
    /// Not equal operator `!=`
    NotEqual,
    /// Less than operator `<`
    LessThan,
    /// Greater than operator `>`
    GreaterThan,
    /// Less than or equal operator `<=`
    LessEqual,
    /// Greater than or equal operator `>=`
    GreaterEqual,
    /// Logical AND operator `&&`
    LogicalAnd,
    /// Logical OR operator `||`
    LogicalOr,
}

impl fmt::Display for BinaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryOperator::Add => write!(f, "+"),
            BinaryOperator::Sub => write!(f, "-"),
            BinaryOperator::Mul => write!(f, "*"),
            BinaryOperator::Div => write!(f, "/"),
            BinaryOperator::Mod => write!(f, "%"),
            BinaryOperator::Equal => write!(f, "=="),
            BinaryOperator::NotEqual => write!(f, "!="),
            BinaryOperator::LessThan => write!(f, "<"),
            BinaryOperator::GreaterThan => write!(f, ">"),
            BinaryOperator::LessEqual => write!(f, "<="),
            BinaryOperator::GreaterEqual => write!(f, ">="),
            BinaryOperator::LogicalAnd => write!(f, "&&"),
            BinaryOperator::LogicalOr => write!(f, "||"),
        }
    }
}

impl BinaryOperator {
    /// Returns true if this is any comparison operator (==, !=, <, >, <=, >=).
    pub fn is_comparison(&self) -> bool {
        matches!(
            self,
            BinaryOperator::Equal
                | BinaryOperator::NotEqual
                | BinaryOperator::LessThan
                | BinaryOperator::GreaterThan
                | BinaryOperator::LessEqual
                | BinaryOperator::GreaterEqual
        )
    }

    /// Returns true if this is an equality operator (== or !=).
    pub fn is_equality(&self) -> bool {
        matches!(self, BinaryOperator::Equal | BinaryOperator::NotEqual)
    }

    /// Returns true if this is an arithmetic operator (+, -, *, /, %).
    pub fn is_arithmetic(&self) -> bool {
        matches!(
            self,
            BinaryOperator::Add
                | BinaryOperator::Sub
                | BinaryOperator::Mul
                | BinaryOperator::Div
                | BinaryOperator::Mod
        )
    }

    /// Returns true if this is a logical operator (&& or ||).
    pub fn is_logical(&self) -> bool {
        matches!(self, BinaryOperator::LogicalAnd | BinaryOperator::LogicalOr)
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
    /// The value is stored as i128 so the parser can preserve all
    /// lexer-accepted unsigned literals (`u64::MAX`) and signed folded
    /// literals (`i64::MIN`) until semantic range checking.
    IntLiteral(i128),

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

    /// An `if` expression that yields a value.
    ///
    /// Unlike [`crate::ast::StmtKind::If`], this form always requires `else`
    /// and each branch must end with a value expression.
    IfExpr {
        /// The condition expression. Must evaluate to `bool`.
        condition: Box<Expr>,
        /// The branch evaluated when condition is true.
        then_block: IfExprBlock,
        /// The branch evaluated when condition is false.
        else_block: IfExprBlock,
    },
}

/// A branch block used by `if` expressions.
///
/// Each branch can contain zero or more statements for side effects and must
/// end with a value expression that becomes the branch result.
#[derive(Debug, Clone)]
pub struct IfExprBlock {
    /// Statements executed before the branch result expression.
    pub stmts: Vec<crate::ast::Stmt>,
    /// The final expression whose value is returned by the branch.
    pub value: Box<Expr>,
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

    /// Returns true if this expression is an integer literal, including `-<int>`.
    pub fn is_integer_literal(&self) -> bool {
        match &self.kind {
            ExprKind::IntLiteral(_) => true,
            ExprKind::UnaryOp { op, operand } => {
                *op == UnaryOperator::Neg && matches!(operand.kind, ExprKind::IntLiteral(_))
            }
            _ => false,
        }
    }

    /// Infers a common operand type for binary operations with integer-literal adaptation.
    ///
    /// Rules:
    /// - Same type on both sides => that type
    /// - Integer literal mixed with an integer type => non-literal integer side
    /// - Otherwise => no common type (`None`)
    pub fn infer_common_binary_operand_type(
        left: &Expr,
        left_ty: &Type,
        right: &Expr,
        right_ty: &Type,
    ) -> Option<Type> {
        if left_ty == right_ty {
            return Some(left_ty.clone());
        }
        if left.is_integer_literal() && right_ty.is_integer() {
            return Some(right_ty.clone());
        }
        if right.is_integer_literal() && left_ty.is_integer() {
            return Some(left_ty.clone());
        }
        None
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
