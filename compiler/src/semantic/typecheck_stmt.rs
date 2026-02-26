use super::symbol::VariableInfo;
use super::{SemanticAnalyzer, SemanticError, SemanticErrorKind};

use crate::ast::{BinaryOperator, Expr, ExprKind, FnDef, Stmt, StmtKind, Type, UnaryOperator};
use crate::token::Span;

impl SemanticAnalyzer {
    pub(super) fn analyze_function(&mut self, function: &FnDef) -> Result<(), SemanticError> {
        self.current_function_return_type = Some(function.return_type.clone());
        self.loop_depth = 0;
        self.symbols.enter_scope();

        let result = (|| -> Result<(), SemanticError> {
            if function.return_type != "void" {
                self.return_type_name_to_type(&function.return_type, function.return_type_span)?;
            }

            for param in &function.params {
                let info = VariableInfo {
                    name: param.name.clone(),
                    is_mutable: false,
                    ty: param.ty.clone(),
                    definition_span: param.span,
                };
                self.symbols.define_variable(info)?;
            }

            let mut always_returns = false;
            for stmt in &function.body {
                let stmt_returns = self.analyze_stmt(stmt)?;
                // Continue analyzing even after guaranteed return so unreachable
                // statements are still type-checked and resolved.
                if !always_returns {
                    always_returns = stmt_returns;
                }
            }

            if function.return_type != "void" && !always_returns {
                return Err(SemanticError::missing_return_in_non_void_function(
                    &function.name,
                    &function.return_type,
                    function.return_type_span,
                ));
            }

            Ok(())
        })();
        self.symbols.exit_scope();
        self.current_function_return_type = None;
        self.loop_depth = 0;
        result
    }

    pub(super) fn analyze_stmt(&mut self, stmt: &Stmt) -> Result<bool, SemanticError> {
        match &stmt.kind {
            StmtKind::Expr(expr) => {
                self.analyze_expr_stmt(expr)?;
                Ok(false)
            }
            StmtKind::Let {
                is_mutable,
                name,
                ty,
                init,
            } => {
                self.analyze_let(*is_mutable, name, ty, init, stmt.span)?;
                Ok(false)
            }
            StmtKind::Assign { name, value } => {
                self.analyze_assign(name, value, stmt.span)?;
                Ok(false)
            }
            StmtKind::Discard(expr) => {
                self.analyze_discard(expr, stmt.span)?;
                Ok(false)
            }
            StmtKind::Return(value) => self.analyze_return(value.as_ref(), stmt.span),
            StmtKind::If {
                condition,
                then_branch,
                else_branch,
            } => self.analyze_if(condition, then_branch, else_branch.as_deref()),
            StmtKind::While { condition, body } => self.analyze_while(condition, body),
            StmtKind::Break => self.analyze_break(stmt.span),
            StmtKind::Continue => self.analyze_continue(stmt.span),
        }
    }

    fn analyze_if(
        &mut self,
        condition: &Expr,
        then_branch: &[Stmt],
        else_branch: Option<&[Stmt]>,
    ) -> Result<bool, SemanticError> {
        self.check_expr_type(condition, &Type::Bool)?;

        let then_returns = self.analyze_block_scoped(then_branch)?;

        let else_returns = if let Some(else_stmts) = else_branch {
            self.analyze_block_scoped(else_stmts)?
        } else {
            false
        };

        let always_returns = match Self::const_bool_expr_value(condition) {
            Some(true) => then_returns,
            Some(false) => else_returns,
            _ => then_returns && else_returns,
        };

        Ok(always_returns)
    }

    fn const_bool_expr_value(expr: &Expr) -> Option<bool> {
        match &expr.kind {
            ExprKind::BoolLiteral(value) => Some(*value),
            ExprKind::UnaryOp { op, operand } => match op {
                UnaryOperator::Not => Self::const_bool_expr_value(operand).map(|value| !value),
                UnaryOperator::Neg => None,
            },
            ExprKind::BinaryOp { left, op, right } => match op {
                BinaryOperator::LogicalAnd => {
                    let left = Self::const_bool_expr_value(left)?;
                    let right = Self::const_bool_expr_value(right)?;
                    Some(left && right)
                }
                BinaryOperator::LogicalOr => {
                    let left = Self::const_bool_expr_value(left)?;
                    let right = Self::const_bool_expr_value(right)?;
                    Some(left || right)
                }
                _ => None,
            },
            _ => None,
        }
    }

    fn analyze_while(&mut self, condition: &Expr, body: &[Stmt]) -> Result<bool, SemanticError> {
        self.check_expr_type(condition, &Type::Bool)?;

        self.loop_depth += 1;
        let body_returns = self.analyze_block_scoped(body);
        self.loop_depth -= 1;

        let body_returns = body_returns?;
        let is_infinite_loop = matches!(condition.kind, ExprKind::BoolLiteral(true));

        Ok(is_infinite_loop && body_returns)
    }

    fn analyze_break(&self, span: Span) -> Result<bool, SemanticError> {
        if self.loop_depth == 0 {
            return Err(SemanticError::break_outside_loop(span));
        }
        Ok(false)
    }

    fn analyze_continue(&self, span: Span) -> Result<bool, SemanticError> {
        if self.loop_depth == 0 {
            return Err(SemanticError::continue_outside_loop(span));
        }
        Ok(false)
    }

    fn analyze_block_scoped(&mut self, stmts: &[Stmt]) -> Result<bool, SemanticError> {
        self.symbols.enter_scope();
        let result = (|| -> Result<bool, SemanticError> {
            let mut always_returns = false;
            for stmt in stmts {
                let stmt_returns = self.analyze_stmt(stmt)?;
                // Preserve "block always returns" once established while still
                // validating later (possibly unreachable) statements.
                if !always_returns {
                    always_returns = stmt_returns;
                }
            }
            Ok(always_returns)
        })();
        self.symbols.exit_scope();
        result
    }

    fn analyze_discard(&mut self, expr: &Expr, span: Span) -> Result<(), SemanticError> {
        match &expr.kind {
            ExprKind::Call { callee, args } => {
                self.analyze_call_value(callee, args, expr.span)?;
                Ok(())
            }
            ExprKind::ModuleCall {
                module,
                function,
                args,
            } => {
                self.analyze_module_call_value(module, function, args, expr.span)?;
                Ok(())
            }
            _ => Err(SemanticError::invalid_discard_target(span)),
        }
    }

    fn analyze_return(&mut self, value: Option<&Expr>, span: Span) -> Result<bool, SemanticError> {
        let return_type_name = self
            .current_function_return_type
            .as_deref()
            .ok_or_else(|| SemanticError::internal_return_outside_function(span))?;

        if return_type_name == "void" {
            if value.is_some() {
                return Err(SemanticError::return_value_in_void_function(span));
            }
            return Ok(true);
        }

        let expected_ty = self.return_type_name_to_type(return_type_name, span)?;
        let value =
            value.ok_or_else(|| SemanticError::return_value_required(return_type_name, span))?;

        // For integer return types, try contextual checking first so wide literals
        // (e.g. u64::MAX) are validated against the declared return type rather
        // than defaulting to i64 during context-free inference.
        if expected_ty.is_integer() && self.check_expr_type(value, &expected_ty).is_ok() {
            return Ok(true);
        }

        // Infer and validate the return expression in its own type context first.
        // This avoids misleading diagnostics such as reporting arithmetic operators
        // against the function return type when the expression itself is valid.
        let actual_ty = self.infer_expr_type(value)?;
        self.check_expr_type(value, &actual_ty)?;

        if actual_ty != expected_ty {
            // Keep contextual integer-literal adaptation (e.g. `return 1` for `-> i32`).
            match self.check_expr_type(value, &expected_ty) {
                Ok(()) => return Ok(true),
                Err(err) if err.kind() != SemanticErrorKind::TypeMismatch => return Err(err),
                Err(_) => {}
            }

            return Err(SemanticError::type_mismatch_return_value(
                &actual_ty.to_string(),
                &expected_ty.to_string(),
                value.span,
            ));
        }

        Ok(true)
    }

    fn analyze_call_stmt(
        &mut self,
        callee: &str,
        args: &[Expr],
        span: Span,
    ) -> Result<(), SemanticError> {
        if callee == "println" {
            if args.len() != 1 {
                return Err(SemanticError::invalid_argument_println_count(span));
            }
            self.validate_expr_for_println(&args[0])?;
            return Ok(());
        }

        if callee == "panic" {
            if args.len() != 1 {
                return Err(SemanticError::invalid_argument_panic_count(span));
            }

            match &args[0].kind {
                ExprKind::StringLiteral(_) => {}
                ExprKind::Identifier(name) => {
                    let var_info = self
                        .symbols
                        .lookup_variable(name)
                        .ok_or_else(|| SemanticError::undefined_variable(name, args[0].span))?;

                    if var_info.ty != Type::String {
                        return Err(SemanticError::invalid_argument_panic_type(
                            &var_info.ty.to_string(),
                            args[0].span,
                        ));
                    }
                }
                ExprKind::IntLiteral(_) => {
                    return Err(SemanticError::invalid_argument_panic_type(
                        "integer literal",
                        args[0].span,
                    ));
                }
                ExprKind::FloatLiteral(_) => {
                    return Err(SemanticError::invalid_argument_panic_type(
                        "float literal",
                        args[0].span,
                    ));
                }
                ExprKind::BoolLiteral(_) => {
                    return Err(SemanticError::invalid_argument_panic_type(
                        "boolean literal",
                        args[0].span,
                    ));
                }
                ExprKind::BinaryOp { .. } | ExprKind::UnaryOp { .. } => {
                    return Err(SemanticError::invalid_argument_panic_type(
                        "expression",
                        args[0].span,
                    ));
                }
                ExprKind::IfExpr { .. } => {
                    let arg_ty = self.infer_expr_type(&args[0])?;
                    if arg_ty != Type::String {
                        return Err(SemanticError::invalid_argument_panic_type(
                            "if expression",
                            args[0].span,
                        ));
                    }
                }
                ExprKind::Call { .. } | ExprKind::ModuleCall { .. } => {
                    let arg_ty = self.infer_expr_type(&args[0])?;
                    if arg_ty != Type::String {
                        return Err(SemanticError::invalid_argument_panic_type(
                            "function call",
                            args[0].span,
                        ));
                    }
                }
                ExprKind::MemberAccess { .. } => {
                    return Err(SemanticError::module_access_not_implemented(args[0].span));
                }
            }
            return Ok(());
        }

        let return_type = self.resolve_user_call(callee, args, span)?;
        if return_type != "void" {
            return Err(SemanticError::type_mismatch_non_void_fn_as_stmt(
                callee,
                &return_type,
                span,
            ));
        }

        Ok(())
    }

    pub(super) fn analyze_call_value(
        &mut self,
        callee: &str,
        args: &[Expr],
        span: Span,
    ) -> Result<Type, SemanticError> {
        if callee == "println" || callee == "panic" {
            self.analyze_call_stmt(callee, args, span)?;
            return Err(SemanticError::void_function_call_as_value(callee, span));
        }

        let return_type = self.resolve_user_call(callee, args, span)?;
        if return_type == "void" {
            return Err(SemanticError::void_function_call_as_value(callee, span));
        }

        self.return_type_name_to_type(&return_type, span)
    }

    pub(super) fn analyze_module_call_stmt(
        &mut self,
        module_name: &str,
        function_name: &str,
        args: &[Expr],
        span: Span,
    ) -> Result<(), SemanticError> {
        let return_type = self.resolve_module_call(module_name, function_name, args, span)?;
        if return_type != "void" {
            return Err(SemanticError::type_mismatch_non_void_fn_as_stmt(
                &format!("{}.{}", module_name, function_name),
                &return_type,
                span,
            ));
        }

        Ok(())
    }

    pub(super) fn analyze_module_call_value(
        &mut self,
        module_name: &str,
        function_name: &str,
        args: &[Expr],
        span: Span,
    ) -> Result<Type, SemanticError> {
        let return_type = self.resolve_module_call(module_name, function_name, args, span)?;
        if return_type == "void" {
            return Err(SemanticError::void_module_call_as_value(
                module_name,
                function_name,
                span,
            ));
        }

        self.return_type_name_to_type(&return_type, span)
    }

    fn analyze_let(
        &mut self,
        is_mutable: bool,
        name: &str,
        ty: &Type,
        init: &Expr,
        span: Span,
    ) -> Result<(), SemanticError> {
        // Check for duplicate variable in current scope first.
        // This preserves error precedence for redefinitions.
        if let Some(existing) = self.symbols.lookup_variable_in_current_scope(name) {
            return Err(SemanticError::duplicate_variable(
                name,
                existing.definition_span.line,
                existing.definition_span.column,
                span,
            ));
        }

        // Type check initializer before introducing the new binding.
        // This rejects self-referential initializers like `let x: i32 = x`
        // and `let x = x`.
        let resolved_ty = if !ty.is_resolved() {
            let inferred_ty = self.infer_expr_type(init)?;
            if !inferred_ty.is_resolved() {
                return Err(SemanticError::internal_define_variable_unexpected_inferred(
                    name, span,
                ));
            }
            // Re-validate the initializer under the inferred concrete type so
            // structural checks (for example integer range validation) still run.
            self.check_expr_type(init, &inferred_ty)?;
            if let Some(existing_ty) = self.inferred_binding_types.get(&span) {
                if *existing_ty != inferred_ty {
                    return Err(SemanticError::internal_inferred_binding_span_collision(
                        name, span,
                    ));
                }
            } else {
                self.inferred_binding_types
                    .insert(span, inferred_ty.clone());
            }
            inferred_ty
        } else {
            self.check_expr_type(init, ty)?;
            ty.clone()
        };

        let info = VariableInfo {
            name: name.to_string(),
            is_mutable,
            ty: resolved_ty,
            definition_span: span,
        };
        self.symbols.define_variable(info)?;

        Ok(())
    }

    fn analyze_assign(
        &mut self,
        name: &str,
        value: &Expr,
        span: Span,
    ) -> Result<(), SemanticError> {
        let (is_mutable, variable_ty) = {
            let var_info = self
                .symbols
                .lookup_variable(name)
                .ok_or_else(|| SemanticError::undefined_variable(name, span))?;
            (var_info.is_mutable, var_info.ty.clone())
        };

        if !is_mutable {
            return Err(SemanticError::immutable_variable_reassignment(name, span));
        }

        self.check_expr_type(value, &variable_ty)?;
        Ok(())
    }

    fn analyze_expr_stmt(&mut self, expr: &Expr) -> Result<(), SemanticError> {
        match &expr.kind {
            ExprKind::Call { callee, args } => self.analyze_call_stmt(callee, args, expr.span),
            ExprKind::StringLiteral(_) => {
                Err(SemanticError::invalid_expression_string_literal(expr.span))
            }
            ExprKind::IntLiteral(_) => {
                Err(SemanticError::invalid_expression_int_literal(expr.span))
            }
            ExprKind::FloatLiteral(_) => {
                Err(SemanticError::invalid_expression_float_literal(expr.span))
            }
            ExprKind::BoolLiteral(_) => {
                Err(SemanticError::invalid_expression_bool_literal(expr.span))
            }
            ExprKind::Identifier(name) => Err(SemanticError::invalid_expression_identifier(
                name, expr.span,
            )),
            ExprKind::BinaryOp { .. } => {
                Err(SemanticError::invalid_expression_binary_op(expr.span))
            }
            ExprKind::UnaryOp { .. } => Err(SemanticError::invalid_expression_unary_op(expr.span)),
            ExprKind::IfExpr { .. } => Err(SemanticError::invalid_expression_binary_op(expr.span)),
            ExprKind::MemberAccess { .. } => {
                Err(SemanticError::module_access_not_implemented(expr.span))
            }
            ExprKind::ModuleCall {
                module,
                function,
                args,
            } => self.analyze_module_call_stmt(module, function, args, expr.span),
        }
    }
}
