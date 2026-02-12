//! Statement code generation.
//!
//! This module implements code generation for Lak statements, including
//! expression statements, `let` bindings, `let _ = ...` discard statements,
//! `return` statements, and control flow (`if`, `while`, `break`, `continue`).

use super::Codegen;
use super::binding::VarBinding;
use super::error::CodegenError;
use crate::ast::{Expr, Stmt, StmtKind, Type};
use crate::token::Span;
use inkwell::values::BasicValueEnum;

impl<'ctx> Codegen<'ctx> {
    /// Generates LLVM IR for a single statement.
    pub(super) fn generate_stmt(&mut self, stmt: &Stmt) -> Result<(), CodegenError> {
        match &stmt.kind {
            StmtKind::Expr(expr) => {
                self.generate_expr(expr)?;
                Ok(())
            }
            StmtKind::Discard(expr) => self.generate_discard(expr, stmt.span),
            StmtKind::Return(value) => self.generate_return(value.as_ref(), stmt.span),
            StmtKind::Let { name, ty, init } => self.generate_let(name, ty, init, stmt.span),
            StmtKind::If {
                condition,
                then_branch,
                else_branch,
            } => self.generate_if(condition, then_branch, else_branch.as_deref(), stmt.span),
            StmtKind::While { condition, body } => self.generate_while(condition, body, stmt.span),
            StmtKind::Break => self.generate_break(stmt.span),
            StmtKind::Continue => self.generate_continue(stmt.span),
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

        let condition_value = match self.generate_expr_value(condition, &Type::Bool)? {
            BasicValueEnum::IntValue(value) => value,
            _ => {
                return Err(CodegenError::internal_non_integer_value(
                    "if condition",
                    span,
                ));
            }
        };

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
            let has_terminator = self
                .builder
                .get_insert_block()
                .and_then(|bb| bb.get_terminator())
                .is_some();
            if has_terminator {
                break;
            }
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
        self.exit_variable_scope(span)?;

        let mut else_has_terminator = false;
        if let (Some(else_bb), Some(else_stmts)) = (else_block, else_branch) {
            self.enter_variable_scope();
            self.builder.position_at_end(else_bb);
            for stmt in else_stmts {
                let has_terminator = self
                    .builder
                    .get_insert_block()
                    .and_then(|bb| bb.get_terminator())
                    .is_some();
                if has_terminator {
                    break;
                }
                self.generate_stmt(stmt)?;
            }
            else_has_terminator = self
                .builder
                .get_insert_block()
                .and_then(|bb| bb.get_terminator())
                .is_some();
            if !else_has_terminator {
                self.builder
                    .build_unconditional_branch(merge_block)
                    .map_err(|e| CodegenError::internal_branch_failed(&e.to_string(), span))?;
            }
            self.exit_variable_scope(span)?;
        }

        if else_block.is_some() && then_has_terminator && else_has_terminator {
            self.builder.position_at_end(merge_block);
            self.builder
                .build_unreachable()
                .map_err(|e| CodegenError::internal_unreachable_failed(&e.to_string(), span))?;
            return Ok(());
        }

        self.builder.position_at_end(merge_block);
        Ok(())
    }

    /// Generates LLVM IR for a while statement.
    pub(super) fn generate_while(
        &mut self,
        condition: &Expr,
        body: &[Stmt],
        span: Span,
    ) -> Result<(), CodegenError> {
        let parent_fn = self
            .builder
            .get_insert_block()
            .and_then(|bb| bb.get_parent())
            .ok_or_else(|| CodegenError::internal_no_current_function(span))?;

        let cond_block = self.context.append_basic_block(parent_fn, "while_cond");
        let body_block = self.context.append_basic_block(parent_fn, "while_body");
        let end_block = self.context.append_basic_block(parent_fn, "while_end");

        self.builder
            .build_unconditional_branch(cond_block)
            .map_err(|e| CodegenError::internal_branch_failed(&e.to_string(), span))?;

        self.builder.position_at_end(cond_block);
        let condition_value = match self.generate_expr_value(condition, &Type::Bool)? {
            BasicValueEnum::IntValue(value) => value,
            _ => {
                return Err(CodegenError::internal_non_integer_value(
                    "while condition",
                    span,
                ));
            }
        };
        self.builder
            .build_conditional_branch(condition_value, body_block, end_block)
            .map_err(|e| CodegenError::internal_branch_failed(&e.to_string(), span))?;

        self.builder.position_at_end(body_block);
        self.enter_variable_scope();
        self.push_loop_control(cond_block, end_block);

        let body_result = (|| -> Result<(), CodegenError> {
            for stmt in body {
                let has_terminator = self
                    .builder
                    .get_insert_block()
                    .and_then(|bb| bb.get_terminator())
                    .is_some();
                if has_terminator {
                    break;
                }
                self.generate_stmt(stmt)?;
            }

            let body_has_terminator = self
                .builder
                .get_insert_block()
                .and_then(|bb| bb.get_terminator())
                .is_some();
            if !body_has_terminator {
                self.builder
                    .build_unconditional_branch(cond_block)
                    .map_err(|e| CodegenError::internal_branch_failed(&e.to_string(), span))?;
            }

            Ok(())
        })();

        self.pop_loop_control(span)?;
        self.exit_variable_scope(span)?;
        body_result?;

        self.builder.position_at_end(end_block);
        Ok(())
    }

    /// Generates LLVM IR for a break statement.
    pub(super) fn generate_break(&mut self, span: Span) -> Result<(), CodegenError> {
        let break_block = self
            .current_loop_control()
            .map(|loop_control| loop_control.break_block)
            .ok_or_else(|| CodegenError::internal_break_outside_loop(span))?;

        self.builder
            .build_unconditional_branch(break_block)
            .map_err(|e| CodegenError::internal_branch_failed(&e.to_string(), span))?;
        Ok(())
    }

    /// Generates LLVM IR for a continue statement.
    pub(super) fn generate_continue(&mut self, span: Span) -> Result<(), CodegenError> {
        let continue_block = self
            .current_loop_control()
            .map(|loop_control| loop_control.continue_block)
            .ok_or_else(|| CodegenError::internal_continue_outside_loop(span))?;

        self.builder
            .build_unconditional_branch(continue_block)
            .map_err(|e| CodegenError::internal_branch_failed(&e.to_string(), span))?;
        Ok(())
    }

    fn generate_discard(&mut self, expr: &Expr, span: Span) -> Result<(), CodegenError> {
        match expr.kind {
            crate::ast::ExprKind::Call { .. } | crate::ast::ExprKind::ModuleCall { .. } => {
                self.generate_expr(expr)
            }
            _ => Err(CodegenError::internal_invalid_expr_stmt(span)),
        }
    }

    fn generate_return(&mut self, value: Option<&Expr>, span: Span) -> Result<(), CodegenError> {
        let parent_fn = self
            .builder
            .get_insert_block()
            .and_then(|bb| bb.get_parent())
            .ok_or_else(|| CodegenError::internal_no_current_function(span))?;
        let llvm_fn_name = parent_fn.get_name().to_string_lossy().to_string();

        if llvm_fn_name == "main" {
            if value.is_some() {
                return Err(CodegenError::internal_main_return_with_value(span));
            }
            let zero = self.context.i32_type().const_int(0, false);
            self.builder
                .build_return(Some(&zero))
                .map_err(|e| CodegenError::internal_main_return_build_failed(&e.to_string()))?;
            return Ok(());
        }
        let display_fn_name = super::user_facing_function_name(&llvm_fn_name);

        let return_ty = self
            .function_return_types
            .get(&llvm_fn_name)
            .cloned()
            .ok_or_else(|| {
                CodegenError::internal_function_signature_not_found(display_fn_name, span)
            })?;

        match return_ty {
            None => {
                if value.is_some() {
                    return Err(CodegenError::internal_return_value_in_void_function(span));
                }
                self.builder.build_return(None).map_err(|e| {
                    CodegenError::internal_return_build_failed(display_fn_name, &e.to_string())
                })?;
            }
            Some(expected_ty) => {
                let value =
                    value.ok_or_else(|| CodegenError::internal_missing_return_value(span))?;
                let return_value = self.generate_expr_value(value, &expected_ty)?;
                self.builder
                    .build_return(Some(&return_value))
                    .map_err(|e| {
                        CodegenError::internal_return_build_failed(display_fn_name, &e.to_string())
                    })?;
            }
        }

        Ok(())
    }
}
