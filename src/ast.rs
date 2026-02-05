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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expr_string_literal() {
        let expr = Expr::StringLiteral("hello".to_string());
        assert!(matches!(expr, Expr::StringLiteral(s) if s == "hello"));
    }

    #[test]
    fn test_expr_call_no_args() {
        let expr = Expr::Call {
            callee: "func".to_string(),
            args: vec![],
        };
        match expr {
            Expr::Call { callee, args } => {
                assert_eq!(callee, "func");
                assert!(args.is_empty());
            }
            _ => panic!("Expected Call"),
        }
    }

    #[test]
    fn test_expr_call_with_args() {
        let expr = Expr::Call {
            callee: "println".to_string(),
            args: vec![
                Expr::StringLiteral("a".to_string()),
                Expr::StringLiteral("b".to_string()),
            ],
        };
        match expr {
            Expr::Call { callee, args } => {
                assert_eq!(callee, "println");
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected Call"),
        }
    }

    #[test]
    fn test_expr_call_nested() {
        let inner = Expr::Call {
            callee: "inner".to_string(),
            args: vec![],
        };
        let outer = Expr::Call {
            callee: "outer".to_string(),
            args: vec![inner],
        };
        match outer {
            Expr::Call { callee, args } => {
                assert_eq!(callee, "outer");
                assert_eq!(args.len(), 1);
                assert!(matches!(&args[0], Expr::Call { callee, .. } if callee == "inner"));
            }
            _ => panic!("Expected Call"),
        }
    }

    #[test]
    fn test_stmt_expr() {
        let expr = Expr::StringLiteral("test".to_string());
        let stmt = Stmt::Expr(expr);
        match stmt {
            Stmt::Expr(e) => {
                assert!(matches!(e, Expr::StringLiteral(s) if s == "test"));
            }
        }
    }

    #[test]
    fn test_program_empty() {
        let program = Program { stmts: vec![] };
        assert!(program.stmts.is_empty());
    }

    #[test]
    fn test_program_with_stmts() {
        let stmts = vec![
            Stmt::Expr(Expr::Call {
                callee: "a".to_string(),
                args: vec![],
            }),
            Stmt::Expr(Expr::Call {
                callee: "b".to_string(),
                args: vec![],
            }),
        ];
        let program = Program { stmts };
        assert_eq!(program.stmts.len(), 2);
    }

    #[test]
    fn test_expr_clone() {
        let expr1 = Expr::Call {
            callee: "test".to_string(),
            args: vec![Expr::StringLiteral("arg".to_string())],
        };
        let expr2 = expr1.clone();

        // Verify both have same structure
        match (&expr1, &expr2) {
            (
                Expr::Call {
                    callee: c1,
                    args: a1,
                },
                Expr::Call {
                    callee: c2,
                    args: a2,
                },
            ) => {
                assert_eq!(c1, c2);
                assert_eq!(a1.len(), a2.len());
            }
            _ => panic!("Clone failed"),
        }
    }

    #[test]
    fn test_stmt_clone() {
        let stmt1 = Stmt::Expr(Expr::StringLiteral("test".to_string()));
        let stmt2 = stmt1.clone();

        match (&stmt1, &stmt2) {
            (Stmt::Expr(e1), Stmt::Expr(e2)) => {
                assert!(matches!(e1, Expr::StringLiteral(s) if s == "test"));
                assert!(matches!(e2, Expr::StringLiteral(s) if s == "test"));
            }
        }
    }

    #[test]
    fn test_expr_debug() {
        let expr = Expr::StringLiteral("hello".to_string());
        let debug_str = format!("{:?}", expr);
        assert!(debug_str.contains("StringLiteral"));
        assert!(debug_str.contains("hello"));
    }

    #[test]
    fn test_stmt_debug() {
        let stmt = Stmt::Expr(Expr::Call {
            callee: "func".to_string(),
            args: vec![],
        });
        let debug_str = format!("{:?}", stmt);
        assert!(debug_str.contains("Expr"));
        assert!(debug_str.contains("Call"));
        assert!(debug_str.contains("func"));
    }

    #[test]
    fn test_program_debug() {
        let program = Program {
            stmts: vec![Stmt::Expr(Expr::StringLiteral("test".to_string()))],
        };
        let debug_str = format!("{:?}", program);
        assert!(debug_str.contains("Program"));
        assert!(debug_str.contains("stmts"));
    }
}
