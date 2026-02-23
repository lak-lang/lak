use super::{SemanticAnalyzer, SemanticError, SemanticErrorKind};

use crate::ast::{BinaryOperator, Expr, ExprKind, IfExprBlock, Type, UnaryOperator};
use crate::token::Span;

impl SemanticAnalyzer {
    pub(super) fn check_expr_type(
        &mut self,
        expr: &Expr,
        expected_ty: &Type,
    ) -> Result<(), SemanticError> {
        match &expr.kind {
            ExprKind::IntLiteral(value) => {
                if *expected_ty == Type::String {
                    return Err(SemanticError::type_mismatch_int_to_string(
                        *value, expr.span,
                    ));
                }
                if *expected_ty == Type::Bool {
                    return Err(SemanticError::type_mismatch_int_to_bool(*value, expr.span));
                }
                if expected_ty.is_float() {
                    return Err(SemanticError::type_mismatch_int_to_type(
                        *value,
                        &expected_ty.to_string(),
                        expr.span,
                    ));
                }
                self.check_integer_range(*value, expected_ty, expr.span)
            }
            ExprKind::FloatLiteral(_) => {
                if !expected_ty.is_float() {
                    return Err(SemanticError::type_mismatch_float_to_type(
                        &expected_ty.to_string(),
                        expr.span,
                    ));
                }
                Ok(())
            }
            ExprKind::Identifier(name) => {
                let var_info = self
                    .symbols
                    .lookup_variable(name)
                    .ok_or_else(|| SemanticError::undefined_variable(name, expr.span))?;

                if var_info.ty != *expected_ty {
                    return Err(SemanticError::type_mismatch_variable(
                        name,
                        &var_info.ty.to_string(),
                        &expected_ty.to_string(),
                        expr.span,
                    ));
                }

                Ok(())
            }
            ExprKind::StringLiteral(_) => {
                if *expected_ty != Type::String {
                    return Err(SemanticError::type_mismatch_string_to_type(
                        &expected_ty.to_string(),
                        expr.span,
                    ));
                }
                Ok(())
            }
            ExprKind::BoolLiteral(_) => {
                if *expected_ty != Type::Bool {
                    return Err(SemanticError::type_mismatch_bool_to_type(
                        &expected_ty.to_string(),
                        expr.span,
                    ));
                }
                Ok(())
            }
            ExprKind::Call { callee, args } => {
                let actual_ty = self.analyze_call_value(callee, args, expr.span)?;
                if actual_ty != *expected_ty {
                    return Err(SemanticError::type_mismatch_call_return(
                        callee,
                        &actual_ty.to_string(),
                        &expected_ty.to_string(),
                        expr.span,
                    ));
                }
                Ok(())
            }
            ExprKind::BinaryOp { left, op, right } => {
                self.check_binary_op_type(left, *op, right, expected_ty, expr.span)
            }
            ExprKind::UnaryOp { op, operand } => {
                self.check_unary_op_type(operand, *op, expected_ty, expr.span)
            }
            ExprKind::IfExpr {
                condition,
                then_block,
                else_block,
            } => {
                self.check_expr_type(condition, &Type::Bool)?;
                let then_contextual = self.analyze_if_expr_block(then_block, Some(expected_ty));
                let else_contextual = self.analyze_if_expr_block(else_block, Some(expected_ty));

                if then_contextual.is_ok() && else_contextual.is_ok() {
                    return Ok(());
                }

                // Prefer branch-local concrete diagnostics (e.g. IntegerOverflow)
                // over generic if-expression type mismatch.
                if let Some(err) = then_contextual
                    .as_ref()
                    .err()
                    .filter(|err| err.kind() != SemanticErrorKind::TypeMismatch)
                {
                    return Err(err.clone());
                }
                if let Some(err) = else_contextual
                    .as_ref()
                    .err()
                    .filter(|err| err.kind() != SemanticErrorKind::TypeMismatch)
                {
                    return Err(err.clone());
                }

                // If contextual typing failed, infer branch result types without context
                // to determine whether this is a branch mismatch or an expected-type mismatch.
                let then_ty = self.analyze_if_expr_block(then_block, None)?;
                let else_ty = self.analyze_if_expr_block(else_block, None)?;
                if then_ty != else_ty {
                    return Err(SemanticError::if_expression_branch_type_mismatch(
                        &then_ty.to_string(),
                        &else_ty.to_string(),
                        expr.span,
                    ));
                }

                if then_ty != *expected_ty {
                    return Err(SemanticError::type_mismatch_if_expression_to_type(
                        &then_ty.to_string(),
                        &expected_ty.to_string(),
                        expr.span,
                    ));
                }

                then_contextual?;
                else_contextual?;
                Ok(())
            }
            ExprKind::MemberAccess { .. } => {
                Err(SemanticError::module_access_not_implemented(expr.span))
            }
            ExprKind::ModuleCall {
                module,
                function,
                args,
            } => {
                let actual_ty =
                    self.analyze_module_call_value(module, function, args, expr.span)?;
                if actual_ty != *expected_ty {
                    return Err(SemanticError::type_mismatch_call_return(
                        &format!("{}.{}", module, function),
                        &actual_ty.to_string(),
                        &expected_ty.to_string(),
                        expr.span,
                    ));
                }
                Ok(())
            }
        }
    }

    /// Checks expression type compatibility, allowing implicit `f32 -> f64`
    /// widening only when the expected operand type is `f64`.
    ///
    /// This helper is used for arithmetic/comparison operands so Lak's
    /// mixed-float promotion rules are enforced while keeping other contexts
    /// strict (no implicit narrowing, no int/float mixing).
    fn check_expr_type_with_float_widening(
        &mut self,
        expr: &Expr,
        expected_ty: &Type,
    ) -> Result<(), SemanticError> {
        if *expected_ty == Type::F64 {
            let actual_ty = self.infer_expr_type(expr)?;
            if actual_ty == Type::F32 {
                return self.check_expr_type(expr, &Type::F32);
            }
        }
        self.check_expr_type(expr, expected_ty)
    }

    fn infer_arithmetic_operand_type(
        &mut self,
        left: &Expr,
        op: BinaryOperator,
        right: &Expr,
        span: Span,
    ) -> Result<Type, SemanticError> {
        let left_ty = self.infer_expr_type(left)?;
        let right_ty = self.infer_expr_type(right)?;

        if let Some(operand_ty) =
            Expr::infer_common_binary_operand_type(left, &left_ty, right, &right_ty)
        {
            if operand_ty.is_numeric() {
                if op == BinaryOperator::Mod && !operand_ty.is_integer() {
                    return Err(SemanticError::invalid_binary_op_type(
                        op,
                        &operand_ty.to_string(),
                        span,
                    ));
                }
                return Ok(operand_ty);
            }
            return Err(SemanticError::invalid_binary_op_type(
                op,
                &operand_ty.to_string(),
                span,
            ));
        }

        if !left_ty.is_numeric() {
            return Err(SemanticError::invalid_binary_op_type(
                op,
                &left_ty.to_string(),
                span,
            ));
        }
        if !right_ty.is_numeric() {
            return Err(SemanticError::invalid_binary_op_type(
                op,
                &right_ty.to_string(),
                span,
            ));
        }

        // Keep existing mismatch diagnostics for non-literal mixed integer types
        // by checking the right operand against the left operand type.
        self.check_expr_type(right, &left_ty)?;

        // Defensive fallback: this path should be unreachable when semantic checks
        // remain consistent, but never silently infer a type here.
        Err(SemanticError::internal_binary_operand_type_mismatch(
            &left_ty.to_string(),
            &right_ty.to_string(),
            span,
        ))
    }

    fn infer_comparison_operand_type(
        &mut self,
        left: &Expr,
        op: BinaryOperator,
        right: &Expr,
        span: Span,
    ) -> Result<Type, SemanticError> {
        let left_ty = self.infer_expr_type(left)?;
        let right_ty = self.infer_expr_type(right)?;

        let operand_ty = if let Some(common_ty) =
            Expr::infer_common_binary_operand_type(left, &left_ty, right, &right_ty)
        {
            common_ty
        } else {
            // Keep existing mismatch diagnostics for non-literal mixed integer types
            // by checking the right operand against the left operand type.
            self.check_expr_type(right, &left_ty)?;

            // Defensive fallback: this path should be unreachable when semantic
            // checks remain consistent.
            return Err(SemanticError::internal_binary_operand_type_mismatch(
                &left_ty.to_string(),
                &right_ty.to_string(),
                span,
            ));
        };

        if !(op.is_equality()
            || operand_ty.is_integer()
            || operand_ty.is_float()
            || operand_ty == Type::String)
        {
            return Err(SemanticError::invalid_ordering_op_type(
                op,
                &operand_ty.to_string(),
                span,
            ));
        }

        self.check_expr_type_with_float_widening(left, &operand_ty)?;
        self.check_expr_type_with_float_widening(right, &operand_ty)?;
        Ok(operand_ty)
    }

    /// Infers the type of an expression without an external expected-type context.
    ///
    /// Used by semantic validation paths that need expression-local type inference,
    /// including arithmetic/comparison operand resolution, println argument checks,
    /// and if-expression branch type convergence.
    ///
    /// Kept in sync with `Codegen::infer_expr_type_for_comparison` in `codegen/expr.rs`
    /// and `Codegen::get_expr_type` in `codegen/builtins.rs`.
    pub(super) fn infer_expr_type(&mut self, expr: &Expr) -> Result<Type, SemanticError> {
        match &expr.kind {
            ExprKind::IntLiteral(_) => Ok(Type::I64),
            ExprKind::FloatLiteral(_) => Ok(Type::F64),
            ExprKind::StringLiteral(_) => Ok(Type::String),
            ExprKind::BoolLiteral(_) => Ok(Type::Bool),
            ExprKind::Identifier(name) => {
                let var = self
                    .symbols
                    .lookup_variable(name)
                    .ok_or_else(|| SemanticError::undefined_variable(name, expr.span))?;
                Ok(var.ty.clone())
            }
            ExprKind::BinaryOp { left, op, right } => {
                if op.is_comparison() || op.is_logical() {
                    Ok(Type::Bool)
                } else if op.is_arithmetic() {
                    // Integer literals adapt to the non-literal integer operand type.
                    self.infer_arithmetic_operand_type(left, *op, right, expr.span)
                } else {
                    Err(SemanticError::internal_unhandled_binary_operator(
                        *op, expr.span,
                    ))
                }
            }
            ExprKind::UnaryOp { op, operand } => match op {
                UnaryOperator::Not => Ok(Type::Bool),
                UnaryOperator::Neg => self.infer_expr_type(operand),
            },
            ExprKind::IfExpr {
                condition,
                then_block,
                else_block,
            } => {
                self.check_expr_type(condition, &Type::Bool)?;
                let then_ty = self.analyze_if_expr_block(then_block, None)?;
                let else_ty = self.analyze_if_expr_block(else_block, None)?;
                if then_ty != else_ty {
                    return Err(SemanticError::if_expression_branch_type_mismatch(
                        &then_ty.to_string(),
                        &else_ty.to_string(),
                        expr.span,
                    ));
                }
                Ok(then_ty)
            }
            ExprKind::Call { callee, args } => self.analyze_call_value(callee, args, expr.span),
            ExprKind::MemberAccess { .. } => {
                Err(SemanticError::module_access_not_implemented(expr.span))
            }
            ExprKind::ModuleCall {
                module,
                function,
                args,
            } => self.analyze_module_call_value(module, function, args, expr.span),
        }
    }

    /// Checks the types of a binary operation.
    ///
    /// For arithmetic operators:
    /// 1. Both operands must have the expected type
    /// 2. The expected type must be an integer primitive
    ///
    /// For comparison operators:
    /// 1. The expected type must be bool (comparison result type)
    /// 2. Equality operators accept all operand types; ordering operators require numeric
    /// 3. Both operands must have the same type
    fn check_binary_op_type(
        &mut self,
        left: &Expr,
        op: BinaryOperator,
        right: &Expr,
        expected_ty: &Type,
        span: Span,
    ) -> Result<(), SemanticError> {
        if op.is_comparison() {
            // Comparison operators produce bool
            if *expected_ty != Type::Bool {
                return Err(SemanticError::type_mismatch_comparison_to_type(
                    op,
                    &expected_ty.to_string(),
                    span,
                ));
            }

            self.infer_comparison_operand_type(left, op, right, span)?;
            Ok(())
        } else if op.is_logical() {
            // Logical operators produce bool
            if *expected_ty != Type::Bool {
                return Err(SemanticError::type_mismatch_logical_to_type(
                    op,
                    &expected_ty.to_string(),
                    span,
                ));
            }

            let left_ty = self.infer_expr_type(left)?;
            if left_ty != Type::Bool {
                return Err(SemanticError::invalid_logical_op_type(
                    op,
                    &left_ty.to_string(),
                    span,
                ));
            }

            let right_ty = self.infer_expr_type(right)?;
            if right_ty != Type::Bool {
                return Err(SemanticError::invalid_logical_op_type(
                    op,
                    &right_ty.to_string(),
                    span,
                ));
            }

            self.check_expr_type(left, &Type::Bool)?;
            self.check_expr_type(right, &Type::Bool)?;

            Ok(())
        } else if op.is_arithmetic() {
            // Arithmetic operators: expected type must be numeric
            if !expected_ty.is_numeric() {
                return Err(SemanticError::invalid_binary_op_type(
                    op,
                    &expected_ty.to_string(),
                    span,
                ));
            }
            if op == BinaryOperator::Mod && !expected_ty.is_integer() {
                return Err(SemanticError::invalid_binary_op_type(
                    op,
                    &expected_ty.to_string(),
                    span,
                ));
            }

            // Check both operands have the expected type
            self.check_expr_type_with_float_widening(left, expected_ty)?;
            self.check_expr_type_with_float_widening(right, expected_ty)?;

            Ok(())
        } else {
            Err(SemanticError::internal_unhandled_binary_operator(op, span))
        }
    }

    /// Checks the types of a unary operation.
    ///
    /// Unary operations require:
    /// 1. The operand to have the expected type
    /// 2. For unary `-`, the expected type to be a signed integer primitive
    /// 3. For unary `!`, the expected type to be `bool`
    fn check_unary_op_type(
        &mut self,
        operand: &Expr,
        op: UnaryOperator,
        expected_ty: &Type,
        span: Span,
    ) -> Result<(), SemanticError> {
        match op {
            UnaryOperator::Neg => {
                if !(expected_ty.is_signed_integer() || expected_ty.is_float()) {
                    return Err(SemanticError::invalid_unary_op_type(
                        op,
                        &expected_ty.to_string(),
                        span,
                    ));
                }

                // Check the operand has the expected type, adding unary context to errors
                self.check_expr_type(operand, expected_ty)
                    .map_err(|e| SemanticError::wrap_in_unary_context(&e, op, span))?;
                Ok(())
            }
            UnaryOperator::Not => {
                if *expected_ty != Type::Bool {
                    return Err(SemanticError::invalid_unary_op_type(
                        op,
                        &expected_ty.to_string(),
                        span,
                    ));
                }

                let operand_ty = self.infer_expr_type(operand)?;
                if operand_ty != Type::Bool {
                    return Err(SemanticError::invalid_unary_op_type(
                        op,
                        &operand_ty.to_string(),
                        span,
                    ));
                }

                self.check_expr_type(operand, &Type::Bool)
                    .map_err(|e| SemanticError::wrap_in_unary_context(&e, op, span))?;
                Ok(())
            }
        }
    }

    /// Validates an expression for use in println.
    ///
    /// Validation is done via:
    /// 1. `infer_expr_type` for contextual type inference (including literal adaptation)
    /// 2. `check_expr_type` for deep structural type validation and precise diagnostics
    pub(super) fn validate_expr_for_println(&mut self, expr: &Expr) -> Result<(), SemanticError> {
        let inferred_ty = self.infer_expr_type(expr)?;
        self.check_expr_type(expr, &inferred_ty)?;
        Ok(())
    }

    fn check_integer_range(&self, value: i128, ty: &Type, span: Span) -> Result<(), SemanticError> {
        match ty {
            Type::I8 => {
                if value < i8::MIN as i128 || value > i8::MAX as i128 {
                    return Err(SemanticError::integer_overflow_for_type(
                        value,
                        "i8",
                        i8::MIN as i128,
                        i8::MAX as i128,
                        span,
                    ));
                }
            }
            Type::I16 => {
                if value < i16::MIN as i128 || value > i16::MAX as i128 {
                    return Err(SemanticError::integer_overflow_for_type(
                        value,
                        "i16",
                        i16::MIN as i128,
                        i16::MAX as i128,
                        span,
                    ));
                }
            }
            Type::I32 => {
                if value < i32::MIN as i128 || value > i32::MAX as i128 {
                    return Err(SemanticError::integer_overflow_for_type(
                        value,
                        "i32",
                        i32::MIN as i128,
                        i32::MAX as i128,
                        span,
                    ));
                }
            }
            Type::I64 => {
                if value < i64::MIN as i128 || value > i64::MAX as i128 {
                    return Err(SemanticError::integer_overflow_for_type(
                        value,
                        "i64",
                        i64::MIN as i128,
                        i64::MAX as i128,
                        span,
                    ));
                }
            }
            Type::U8 => {
                if value < u8::MIN as i128 || value > u8::MAX as i128 {
                    return Err(SemanticError::integer_overflow_for_type(
                        value,
                        "u8",
                        u8::MIN as i128,
                        u8::MAX as i128,
                        span,
                    ));
                }
            }
            Type::U16 => {
                if value < u16::MIN as i128 || value > u16::MAX as i128 {
                    return Err(SemanticError::integer_overflow_for_type(
                        value,
                        "u16",
                        u16::MIN as i128,
                        u16::MAX as i128,
                        span,
                    ));
                }
            }
            Type::U32 => {
                if value < u32::MIN as i128 || value > u32::MAX as i128 {
                    return Err(SemanticError::integer_overflow_for_type(
                        value,
                        "u32",
                        u32::MIN as i128,
                        u32::MAX as i128,
                        span,
                    ));
                }
            }
            Type::U64 => {
                if value < u64::MIN as i128 || value > u64::MAX as i128 {
                    return Err(SemanticError::integer_overflow_for_type(
                        value,
                        "u64",
                        u64::MIN as i128,
                        u64::MAX as i128,
                        span,
                    ));
                }
            }
            Type::F32 => {
                // This branch should never be reached because check_expr_type
                // handles float expectations before calling check_integer_range.
                return Err(SemanticError::internal_check_integer_range_unexpected_f32(
                    value, span,
                ));
            }
            Type::F64 => {
                // This branch should never be reached because check_expr_type
                // handles float expectations before calling check_integer_range.
                return Err(SemanticError::internal_check_integer_range_unexpected_f64(
                    value, span,
                ));
            }
            Type::String => {
                // This branch should never be reached because check_expr_type
                // handles Type::String before calling check_integer_range.
                // Return an internal error to signal a compiler bug if this is reached.
                return Err(SemanticError::internal_check_integer_range_string(
                    value, span,
                ));
            }
            Type::Bool => {
                // This branch should never be reached because check_expr_type
                // handles Type::Bool before calling check_integer_range.
                // Return an internal error to signal a compiler bug if this is reached.
                return Err(SemanticError::internal_check_integer_range_bool(
                    value, span,
                ));
            }
        }
        Ok(())
    }

    /// Analyzes an if-expression branch block and returns its result type.
    ///
    /// If `expected_ty` is provided, the branch value is checked directly against
    /// that type to preserve contextual typing (e.g. i32 integer literals).
    fn analyze_if_expr_block(
        &mut self,
        block: &IfExprBlock,
        expected_ty: Option<&Type>,
    ) -> Result<Type, SemanticError> {
        self.symbols.enter_scope();
        let result = (|| -> Result<Type, SemanticError> {
            for stmt in &block.stmts {
                self.analyze_stmt(stmt)?;
            }
            if let Some(expected) = expected_ty {
                self.check_expr_type(&block.value, expected)?;
                return Ok(expected.clone());
            }
            let value_ty = self.infer_expr_type(&block.value)?;
            self.check_expr_type(&block.value, &value_ty)?;
            Ok(value_ty)
        })();
        self.symbols.exit_scope();
        result
    }
}
