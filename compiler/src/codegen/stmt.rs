//! Statement code generation.
//!
//! This module implements code generation for Lak statements, including
//! expression statements and let bindings.

use super::Codegen;
use super::binding::VarBinding;
use super::error::{CodegenError, CodegenErrorKind};
use crate::ast::{Expr, Stmt, StmtKind, Type};
use crate::token::Span;

impl<'ctx> Codegen<'ctx> {
    /// Generates LLVM IR for a single statement.
    pub(super) fn generate_stmt(&mut self, stmt: &Stmt) -> Result<(), CodegenError> {
        match &stmt.kind {
            StmtKind::Expr(expr) => {
                self.generate_expr(expr)?;
                Ok(())
            }
            StmtKind::Let { name, ty, init } => self.generate_let(name, ty, init, stmt.span),
        }
    }

    /// Generates LLVM IR for a let statement.
    ///
    /// Creates a stack allocation for the variable, evaluates the initializer,
    /// and stores the value. The variable is registered in the symbol table
    /// for later reference.
    ///
    /// # Arguments
    ///
    /// * `name` - The variable name
    /// * `ty` - The declared type
    /// * `init` - The initializer expression
    /// * `span` - The source location of the let statement
    ///
    /// # Errors
    ///
    /// Returns an error if LLVM fails to create allocation or store instructions.
    ///
    /// # Panics
    ///
    /// Panics if the variable is already defined. This should never happen
    /// because semantic analysis guarantees no duplicate variables.
    pub(super) fn generate_let(
        &mut self,
        name: &str,
        ty: &Type,
        init: &Expr,
        span: Span,
    ) -> Result<(), CodegenError> {
        // Semantic analysis guarantees no duplicate variables
        debug_assert!(
            !self.variables.contains_key(name),
            "Duplicate variable '{}' should have been caught by semantic analysis",
            name
        );

        let binding = VarBinding::new(&self.builder, self.context, ty, name)
            .map_err(|e| CodegenError::new(CodegenErrorKind::InternalError, e, span))?;

        let init_value = self.generate_expr_value(init, ty)?;

        self.builder
            .build_store(binding.alloca(), init_value)
            .map_err(|e| {
                CodegenError::new(
                    CodegenErrorKind::InternalError,
                    format!(
                        "Internal error: failed to store initial value for '{}'. \
                         This is a compiler bug: {}",
                        name, e
                    ),
                    span,
                )
            })?;

        self.variables.insert(name.to_string(), binding);

        Ok(())
    }
}
