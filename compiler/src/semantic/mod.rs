//! Semantic analysis for the Lak programming language.
//!
//! This module provides the [`SemanticAnalyzer`] which validates a Lak AST
//! for semantic correctness before code generation.
//!
//! # Responsibilities
//!
//! The semantic analyzer performs the following validations:
//!
//! - **Name resolution**: Checks for duplicate/undefined functions and variables
//! - **Type checking**: Validates type consistency in assignments and expressions
//! - **Structural validation**: Ensures main function exists with correct signature
//!
//! # Pipeline Position
//!
//! ```text
//! Source → Lexer → Parser → [Module Resolver] → Semantic Analyzer → Codegen → Executable
//! ```
//!
//! The semantic analyzer sits between the parser and code generator. It takes
//! an AST and either returns success (allowing codegen to proceed) or an error
//! describing the semantic problem.

mod error;
mod module_table;
mod symbol;

#[cfg(test)]
mod tests;

pub use error::{SemanticError, SemanticErrorKind};
pub use module_table::ModuleTable;
use symbol::{FunctionInfo, SymbolTable, VariableInfo};

use crate::ast::{
    BinaryOperator, Expr, ExprKind, FnDef, IfExprBlock, Program, Stmt, StmtKind, Type,
    UnaryOperator,
};
use crate::token::Span;

/// The mode of semantic analysis, determining which validations are performed.
enum AnalysisMode {
    /// Analyzing a single-file program (no imports).
    SingleFile,
    /// Analyzing an entry module with imports. Contains the module table
    /// for validating cross-module references.
    EntryWithModules(ModuleTable),
    /// Analyzing an imported module (no main function required).
    /// Optionally carries a module table for imported modules that have their own imports.
    ImportedModule(Option<ModuleTable>),
}

/// Semantic analyzer for Lak programs.
///
/// Performs semantic validation on an AST without modifying it:
/// - Name resolution (duplicate/undefined checks)
/// - Type checking
/// - Structural validation (main function)
///
/// The analyzer guarantees that if `analyze()` succeeds, the AST is
/// semantically valid and code generation can proceed without semantic errors.
pub struct SemanticAnalyzer {
    symbols: SymbolTable,
    mode: AnalysisMode,
}

impl SemanticAnalyzer {
    /// Creates a new semantic analyzer.
    pub fn new() -> Self {
        SemanticAnalyzer {
            symbols: SymbolTable::new(),
            mode: AnalysisMode::SingleFile,
        }
    }

    /// Analyzes a program for semantic correctness.
    ///
    /// Performs complete semantic validation in this order:
    /// 1. Collect all function definitions (check for duplicates)
    /// 2. Validate main function exists and has correct signature
    /// 3. Analyze each function body (variables, types, expressions)
    ///
    /// # Errors
    ///
    /// Returns an error if any semantic violation is found:
    /// - Duplicate function definitions
    /// - Missing main function
    /// - Invalid main signature
    /// - Duplicate variable definitions
    /// - Undefined variable/function references
    /// - Type mismatches
    /// - Integer overflow
    pub fn analyze(&mut self, program: &Program) -> Result<(), SemanticError> {
        // Phase 1: Collect function definitions
        self.collect_functions(program)?;

        // Phase 2: Validate main function
        self.validate_main_function(program)?;

        // Phase 3: Analyze function bodies
        for function in &program.functions {
            self.analyze_function(function)?;
        }

        Ok(())
    }

    /// Analyzes a program for semantic correctness with module context.
    ///
    /// This is used when compiling programs that import other modules.
    pub fn analyze_with_modules(
        &mut self,
        program: &Program,
        module_table: ModuleTable,
    ) -> Result<(), SemanticError> {
        self.mode = AnalysisMode::EntryWithModules(module_table);
        self.analyze(program)
    }

    /// Analyzes an imported module for semantic correctness.
    ///
    /// Unlike `analyze()`, this method does NOT require a main function,
    /// since imported modules are libraries, not entry points.
    ///
    /// Performs:
    /// 1. Function collection (check for duplicates)
    /// 2. Function body analysis (variables, types, expressions)
    pub fn analyze_module(
        &mut self,
        program: &Program,
        module_table: Option<ModuleTable>,
    ) -> Result<(), SemanticError> {
        self.mode = AnalysisMode::ImportedModule(module_table);

        // Phase 1: Collect function definitions
        self.collect_functions(program)?;

        // Phase 2: Analyze function bodies
        for function in &program.functions {
            self.analyze_function(function)?;
        }

        Ok(())
    }

    // Phase 1: Function collection

    fn collect_functions(&mut self, program: &Program) -> Result<(), SemanticError> {
        for function in &program.functions {
            if matches!(function.name.as_str(), "println" | "panic") {
                return Err(SemanticError::reserved_prelude_function_name(
                    &function.name,
                    function.span,
                ));
            }

            let info = FunctionInfo {
                name: function.name.clone(),
                return_type: function.return_type.clone(),
                return_type_span: function.return_type_span,
                definition_span: function.span,
                visibility: function.visibility,
            };

            self.symbols.define_function(info)?;
        }
        Ok(())
    }

    // Phase 2: Main function validation

    fn validate_main_function(&self, program: &Program) -> Result<(), SemanticError> {
        // Check main exists
        let main_fn = self.symbols.lookup_function("main").ok_or_else(|| {
            if program.functions.is_empty() {
                SemanticError::missing_main_no_functions()
            } else {
                let names: Vec<_> = program.functions.iter().map(|f| f.name.as_str()).collect();
                SemanticError::missing_main_with_functions(&names)
            }
        })?;

        // Validate signature
        if main_fn.return_type != "void" {
            return Err(SemanticError::invalid_main_signature(
                &main_fn.return_type,
                main_fn.return_type_span,
            ));
        }

        Ok(())
    }

    // Phase 3: Function body analysis

    fn analyze_function(&mut self, function: &FnDef) -> Result<(), SemanticError> {
        self.symbols.enter_scope();

        for stmt in &function.body {
            self.analyze_stmt(stmt)?;
        }

        self.symbols.exit_scope();
        Ok(())
    }

    fn analyze_stmt(&mut self, stmt: &Stmt) -> Result<(), SemanticError> {
        match &stmt.kind {
            StmtKind::Expr(expr) => self.analyze_expr_stmt(expr),
            StmtKind::Let { name, ty, init } => self.analyze_let(name, ty, init, stmt.span),
            StmtKind::If {
                condition,
                then_branch,
                else_branch,
            } => self.analyze_if(condition, then_branch, else_branch.as_deref()),
        }
    }

    fn analyze_if(
        &mut self,
        condition: &Expr,
        then_branch: &[Stmt],
        else_branch: Option<&[Stmt]>,
    ) -> Result<(), SemanticError> {
        self.check_expr_type(condition, &Type::Bool)?;

        self.symbols.enter_scope();
        for stmt in then_branch {
            self.analyze_stmt(stmt)?;
        }
        self.symbols.exit_scope();

        if let Some(else_stmts) = else_branch {
            self.symbols.enter_scope();
            for stmt in else_stmts {
                self.analyze_stmt(stmt)?;
            }
            self.symbols.exit_scope();
        }

        Ok(())
    }

    fn analyze_let(
        &mut self,
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
        // This rejects self-referential initializers like `let x: i32 = x`.
        self.check_expr_type(init, ty)?;

        let info = VariableInfo {
            name: name.to_string(),
            ty: ty.clone(),
            definition_span: span,
        };
        self.symbols.define_variable(info)?;

        Ok(())
    }

    fn analyze_expr_stmt(&mut self, expr: &Expr) -> Result<(), SemanticError> {
        match &expr.kind {
            ExprKind::Call { callee, args } => self.analyze_call(callee, args, expr.span),
            ExprKind::StringLiteral(_) => {
                Err(SemanticError::invalid_expression_string_literal(expr.span))
            }
            ExprKind::IntLiteral(_) => {
                Err(SemanticError::invalid_expression_int_literal(expr.span))
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
            } => self.analyze_module_call(module, function, args, expr.span),
        }
    }

    fn analyze_call(
        &mut self,
        callee: &str,
        args: &[Expr],
        span: Span,
    ) -> Result<(), SemanticError> {
        // Built-in function: println
        // println accepts any type (string, i32, i64, bool)
        if callee == "println" {
            if args.len() != 1 {
                return Err(SemanticError::invalid_argument_println_count(span));
            }

            // println accepts string literals, integer literals, boolean literals, or any variable
            match &args[0].kind {
                ExprKind::StringLiteral(_) => {}
                ExprKind::IntLiteral(_) => {}
                ExprKind::BoolLiteral(_) => {}
                ExprKind::Identifier(name) => {
                    // Verify the variable exists (type doesn't matter, any type is accepted)
                    self.symbols
                        .lookup_variable(name)
                        .ok_or_else(|| SemanticError::undefined_variable(name, args[0].span))?;
                }
                ExprKind::Call {
                    callee: inner_callee,
                    ..
                } => {
                    return Err(SemanticError::invalid_argument_call_not_supported(
                        inner_callee,
                        args[0].span,
                    ));
                }
                ExprKind::BinaryOp { .. } => {
                    self.validate_expr_for_println(&args[0])?;
                }
                ExprKind::UnaryOp { .. } => {
                    // For unary operations, validate that all variables exist
                    // and that no unary operator is applied to a string type
                    self.validate_expr_for_println(&args[0])?;
                }
                ExprKind::IfExpr { .. } => {
                    self.validate_expr_for_println(&args[0])?;
                }
                ExprKind::MemberAccess { .. } => {
                    return Err(SemanticError::module_access_not_implemented(args[0].span));
                }
                ExprKind::ModuleCall {
                    module, function, ..
                } => {
                    return Err(SemanticError::module_call_return_value_not_supported(
                        module,
                        function,
                        args[0].span,
                    ));
                }
            }

            return Ok(());
        }

        // Built-in function: panic
        if callee == "panic" {
            if args.len() != 1 {
                return Err(SemanticError::invalid_argument_panic_count(span));
            }

            // panic only accepts string arguments
            match &args[0].kind {
                ExprKind::StringLiteral(_) => {}
                ExprKind::Identifier(name) => {
                    // Verify the variable exists and is a string
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
                ExprKind::BoolLiteral(_) => {
                    return Err(SemanticError::invalid_argument_panic_type(
                        "boolean literal",
                        args[0].span,
                    ));
                }
                ExprKind::Call {
                    callee: inner_callee,
                    ..
                } => {
                    return Err(SemanticError::invalid_argument_call_not_supported(
                        inner_callee,
                        args[0].span,
                    ));
                }
                ExprKind::BinaryOp { .. } => {
                    return Err(SemanticError::invalid_argument_panic_type(
                        "expression",
                        args[0].span,
                    ));
                }
                ExprKind::UnaryOp { .. } => {
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
                ExprKind::MemberAccess { .. } => {
                    return Err(SemanticError::module_access_not_implemented(args[0].span));
                }
                ExprKind::ModuleCall {
                    module, function, ..
                } => {
                    return Err(SemanticError::module_call_return_value_not_supported(
                        module,
                        function,
                        args[0].span,
                    ));
                }
            }

            return Ok(());
        }

        // Check if function is defined
        let func_info = self
            .symbols
            .lookup_function(callee)
            .ok_or_else(|| SemanticError::undefined_function(callee, span))?;

        // Disallow calling main function directly
        if callee == "main" {
            return Err(SemanticError::invalid_argument_cannot_call_main(span));
        }

        // Check argument count (currently only parameterless functions are supported)
        if !args.is_empty() {
            return Err(SemanticError::invalid_argument_fn_expects_no_args(
                callee,
                args.len(),
                span,
            ));
        }

        // Check that the function returns void (only void functions can be called as statements)
        if func_info.return_type != "void" {
            return Err(SemanticError::type_mismatch_non_void_fn_as_stmt(
                callee,
                &func_info.return_type,
                span,
            ));
        }

        Ok(())
    }

    fn analyze_module_call(
        &mut self,
        module_name: &str,
        function_name: &str,
        args: &[Expr],
        span: Span,
    ) -> Result<(), SemanticError> {
        // Get the module table based on the current analysis mode
        let module_table = match &self.mode {
            AnalysisMode::EntryWithModules(table) => table,
            AnalysisMode::ImportedModule(Some(table)) => table,
            AnalysisMode::ImportedModule(None) => {
                // Imported module has no module table (no imports of its own).
                // This means it has a ModuleCall expression but no way to resolve it.
                return Err(SemanticError::cross_module_call_in_imported_module(
                    module_name,
                    function_name,
                    span,
                ));
            }
            AnalysisMode::SingleFile => {
                // No module table means module resolution is not enabled
                return Err(SemanticError::module_not_imported(
                    module_name,
                    function_name,
                    span,
                ));
            }
        };

        // Look up the module
        let module_exports = module_table
            .get_module(module_name)
            .ok_or_else(|| SemanticError::undefined_module(module_name, span))?;

        // Look up the function in the module
        let func_export = module_exports.get_function(function_name).ok_or_else(|| {
            SemanticError::undefined_module_function(module_name, function_name, span)
        })?;

        // Check argument count (currently only parameterless functions are supported)
        if !args.is_empty() {
            return Err(SemanticError::invalid_argument_fn_expects_no_args(
                &format!("{}.{}", module_name, function_name),
                args.len(),
                span,
            ));
        }

        // Check that the function returns void (only void functions can be called as statements)
        if func_export.return_type() != "void" {
            return Err(SemanticError::type_mismatch_non_void_fn_as_stmt(
                &format!("{}.{}", module_name, function_name),
                func_export.return_type(),
                span,
            ));
        }

        Ok(())
    }

    fn check_expr_type(&mut self, expr: &Expr, expected_ty: &Type) -> Result<(), SemanticError> {
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
                self.check_integer_range(*value, expected_ty, expr.span)
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
            ExprKind::Call { callee, .. } => Err(SemanticError::type_mismatch_call_as_value(
                callee, expr.span,
            )),
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
                module, function, ..
            } => Err(SemanticError::module_call_return_value_not_supported(
                module, function, expr.span,
            )),
        }
    }

    /// Infers the type of an expression without checking against an expected type.
    ///
    /// Used for comparison operators where we need to determine operand types
    /// before checking they match each other.
    ///
    /// Kept in sync with `Codegen::infer_expr_type_for_comparison` in `codegen/expr.rs`
    /// and `Codegen::get_expr_type` in `codegen/builtins.rs`.
    fn infer_expr_type(&mut self, expr: &Expr) -> Result<Type, SemanticError> {
        match &expr.kind {
            ExprKind::IntLiteral(_) => Ok(Type::I64),
            ExprKind::StringLiteral(_) => Ok(Type::String),
            ExprKind::BoolLiteral(_) => Ok(Type::Bool),
            ExprKind::Identifier(name) => {
                let var = self
                    .symbols
                    .lookup_variable(name)
                    .ok_or_else(|| SemanticError::undefined_variable(name, expr.span))?;
                Ok(var.ty.clone())
            }
            ExprKind::BinaryOp { left, op, .. } => {
                if op.is_comparison() || op.is_logical() {
                    Ok(Type::Bool)
                } else {
                    self.infer_expr_type(left)
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
            ExprKind::Call { callee, .. } => Err(SemanticError::type_mismatch_call_as_value(
                callee, expr.span,
            )),
            ExprKind::MemberAccess { .. } => {
                Err(SemanticError::module_access_not_implemented(expr.span))
            }
            ExprKind::ModuleCall {
                module, function, ..
            } => Err(SemanticError::module_call_return_value_not_supported(
                module, function, expr.span,
            )),
        }
    }

    /// Validates operands for a comparison operation.
    ///
    /// Checks that:
    /// 1. For ordering operators (<, >, <=, >=), the operand type is numeric (i32 or i64)
    /// 2. The right operand matches the left operand's type
    ///
    /// Returns the inferred operand type on success.
    fn validate_comparison_operands(
        &mut self,
        left: &Expr,
        op: BinaryOperator,
        right: &Expr,
        span: Span,
    ) -> Result<Type, SemanticError> {
        let operand_ty = self.infer_expr_type(left)?;
        if !op.is_equality() && operand_ty != Type::I32 && operand_ty != Type::I64 {
            return Err(SemanticError::invalid_ordering_op_type(
                op,
                &operand_ty.to_string(),
                span,
            ));
        }
        self.check_expr_type(right, &operand_ty)?;
        Ok(operand_ty)
    }

    /// Checks the types of a binary operation.
    ///
    /// For arithmetic operators:
    /// 1. Both operands must have the expected type
    /// 2. The expected type must be numeric (i32 or i64)
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

            let operand_ty = self.validate_comparison_operands(left, op, right, span)?;
            // Recursively validate sub-expressions within the left operand.
            // infer_expr_type only returns the outermost type; this deeply validates
            // the entire expression tree (e.g., nested binary ops).
            self.check_expr_type(left, &operand_ty)?;

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
            if *expected_ty == Type::String || *expected_ty == Type::Bool {
                return Err(SemanticError::invalid_binary_op_type(
                    op,
                    &expected_ty.to_string(),
                    span,
                ));
            }

            // Check both operands have the expected type
            self.check_expr_type(left, expected_ty)?;
            self.check_expr_type(right, expected_ty)?;

            Ok(())
        } else {
            Err(SemanticError::internal_unhandled_binary_operator(op, span))
        }
    }

    /// Checks the types of a unary operation.
    ///
    /// Unary operations require:
    /// 1. The operand to have the expected type
    /// 2. The expected type to be numeric (i32 or i64)
    fn check_unary_op_type(
        &mut self,
        operand: &Expr,
        op: UnaryOperator,
        expected_ty: &Type,
        span: Span,
    ) -> Result<(), SemanticError> {
        match op {
            UnaryOperator::Neg => {
                // Verify the expected type is numeric (not string or bool)
                if *expected_ty == Type::String || *expected_ty == Type::Bool {
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
    /// This recursively validates that:
    /// 1. All variables referenced in the expression exist
    /// 2. No unary operator is applied to a string type
    /// 3. Comparison operands have matching types and ordering operators use numeric types
    fn validate_expr_for_println(&mut self, expr: &Expr) -> Result<(), SemanticError> {
        match &expr.kind {
            ExprKind::IntLiteral(_) | ExprKind::StringLiteral(_) | ExprKind::BoolLiteral(_) => {
                Ok(())
            }
            ExprKind::Identifier(name) => {
                self.symbols
                    .lookup_variable(name)
                    .ok_or_else(|| SemanticError::undefined_variable(name, expr.span))?;
                Ok(())
            }
            ExprKind::BinaryOp { left, op, right } => {
                // First validate sub-expressions recursively
                self.validate_expr_for_println(left)?;
                self.validate_expr_for_println(right)?;

                // Then type-check the comparison operands
                if op.is_comparison() {
                    self.validate_comparison_operands(left, *op, right, expr.span)?;
                } else if op.is_logical() {
                    let left_ty = self.infer_expr_type(left)?;
                    if left_ty != Type::Bool {
                        return Err(SemanticError::invalid_logical_op_type(
                            *op,
                            &left_ty.to_string(),
                            expr.span,
                        ));
                    }
                    let right_ty = self.infer_expr_type(right)?;
                    if right_ty != Type::Bool {
                        return Err(SemanticError::invalid_logical_op_type(
                            *op,
                            &right_ty.to_string(),
                            expr.span,
                        ));
                    }
                    self.check_expr_type(left, &Type::Bool)?;
                    self.check_expr_type(right, &Type::Bool)?;
                }

                Ok(())
            }
            ExprKind::UnaryOp { op, operand } => {
                // First validate the operand recursively
                self.validate_expr_for_println(operand)?;

                let operand_ty = self.infer_expr_type(operand)?;
                match op {
                    UnaryOperator::Neg
                        if operand_ty == Type::String || operand_ty == Type::Bool =>
                    {
                        return Err(SemanticError::invalid_unary_op_type(
                            *op,
                            &operand_ty.to_string(),
                            expr.span,
                        ));
                    }
                    UnaryOperator::Not if operand_ty != Type::Bool => {
                        return Err(SemanticError::invalid_unary_op_type(
                            *op,
                            &operand_ty.to_string(),
                            expr.span,
                        ));
                    }
                    _ => {}
                }
                Ok(())
            }
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
                Ok(())
            }
            ExprKind::Call { callee, .. } => Err(
                SemanticError::invalid_argument_call_not_supported(callee, expr.span),
            ),
            ExprKind::MemberAccess { .. } => {
                Err(SemanticError::module_access_not_implemented(expr.span))
            }
            ExprKind::ModuleCall {
                module, function, ..
            } => Err(SemanticError::module_call_return_value_not_supported(
                module, function, expr.span,
            )),
        }
    }

    fn check_integer_range(&self, value: i64, ty: &Type, span: Span) -> Result<(), SemanticError> {
        match ty {
            Type::I32 => {
                if value < i32::MIN as i64 || value > i32::MAX as i64 {
                    return Err(SemanticError::integer_overflow_i32(value, span));
                }
            }
            Type::I64 => {
                // Invariant: The parser converts u64 tokens to i64 AST nodes,
                // so any value that made it past parsing is guaranteed to be within i64 range.
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

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
