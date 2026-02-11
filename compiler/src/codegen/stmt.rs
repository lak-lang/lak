//! Statement code generation.
//!
//! This module implements code generation for Lak statements, including
//! expression statements and let bindings.

use super::Codegen;
use super::binding::VarBinding;
use super::error::CodegenError;
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
            StmtKind::If {
                condition,
                then_branch,
                else_branch,
            } => self.generate_if(condition, then_branch, else_branch.as_deref(), stmt.span),
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
    /// Returns an internal error if the variable is already defined. This should
    /// never happen because semantic analysis guarantees no duplicate variables.
    pub(super) fn generate_let(
        &mut self,
        name: &str,
        ty: &Type,
        init: &Expr,
        span: Span,
    ) -> Result<(), CodegenError> {
        // Semantic analysis guarantees no duplicate variables in the same scope.
        if self.variable_in_current_scope(name) {
            return Err(CodegenError::internal_duplicate_variable(name, span));
        }

        let binding = VarBinding::new(&self.builder, self.context, ty, name, span)?;

        let init_value = self.generate_expr_value(init, ty)?;

        self.builder
            .build_store(binding.alloca(), init_value)
            .map_err(|e| {
                CodegenError::internal_variable_store_failed(name, &e.to_string(), span)
            })?;

        self.define_variable_in_current_scope(name, binding, span)?;

        Ok(())
    }

    /// Generates LLVM IR for an if statement.
    pub(super) fn generate_if(
        &mut self,
        condition: &Expr,
        then_branch: &[Stmt],
        else_branch: Option<&[Stmt]>,
        span: Span,
    ) -> Result<(), CodegenError> {
        let parent_fn = self
            .builder
            .get_insert_block()
            .and_then(|bb| bb.get_parent())
            .ok_or_else(|| CodegenError::internal_no_current_function(span))?;

        let then_block = self.context.append_basic_block(parent_fn, "if_then");
        let merge_block = self.context.append_basic_block(parent_fn, "if_end");
        let else_block = else_branch.map(|_| self.context.append_basic_block(parent_fn, "if_else"));

        let condition_value = self
            .generate_expr_value(condition, &Type::Bool)?
            .into_int_value();

        if let Some(else_bb) = else_block {
            self.builder
                .build_conditional_branch(condition_value, then_block, else_bb)
                .map_err(|e| CodegenError::internal_branch_failed(&e.to_string(), span))?;
        } else {
            self.builder
                .build_conditional_branch(condition_value, then_block, merge_block)
                .map_err(|e| CodegenError::internal_branch_failed(&e.to_string(), span))?;
        }

        self.enter_variable_scope();
        self.builder.position_at_end(then_block);
        for stmt in then_branch {
            self.generate_stmt(stmt)?;
        }
        let then_has_terminator = self
            .builder
            .get_insert_block()
            .and_then(|bb| bb.get_terminator())
            .is_some();
        if !then_has_terminator {
            self.builder
                .build_unconditional_branch(merge_block)
                .map_err(|e| CodegenError::internal_branch_failed(&e.to_string(), span))?;
        }
        self.exit_variable_scope();

        if let (Some(else_bb), Some(else_stmts)) = (else_block, else_branch) {
            self.enter_variable_scope();
            self.builder.position_at_end(else_bb);
            for stmt in else_stmts {
                self.generate_stmt(stmt)?;
            }
            let else_has_terminator = self
                .builder
                .get_insert_block()
                .and_then(|bb| bb.get_terminator())
                .is_some();
            if !else_has_terminator {
                self.builder
                    .build_unconditional_branch(merge_block)
                    .map_err(|e| CodegenError::internal_branch_failed(&e.to_string(), span))?;
            }
            self.exit_variable_scope();
        }

        self.builder.position_at_end(merge_block);
        Ok(())
    }
}
