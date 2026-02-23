use super::symbol::FunctionInfo;
use super::{AnalysisMode, SemanticAnalyzer, SemanticError};

use crate::ast::{Expr, Program, Type};
use crate::token::Span;

impl SemanticAnalyzer {
    // Phase 1: Function collection

    pub(super) fn collect_functions(&mut self, program: &Program) -> Result<(), SemanticError> {
        for function in &program.functions {
            if matches!(function.name.as_str(), "println" | "panic") {
                return Err(SemanticError::reserved_prelude_function_name(
                    &function.name,
                    function.span,
                ));
            }

            let info = FunctionInfo {
                name: function.name.clone(),
                param_types: function
                    .params
                    .iter()
                    .map(|param| param.ty.clone())
                    .collect(),
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

    pub(super) fn validate_main_function(&self, program: &Program) -> Result<(), SemanticError> {
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
        if !main_fn.param_types.is_empty() {
            return Err(SemanticError::invalid_main_signature_has_params(
                main_fn.param_types.len(),
                main_fn.definition_span,
            ));
        }

        if main_fn.return_type != "void" {
            return Err(SemanticError::invalid_main_signature(
                &main_fn.return_type,
                main_fn.return_type_span,
            ));
        }

        Ok(())
    }

    pub(super) fn return_type_name_to_type(
        &self,
        name: &str,
        span: Span,
    ) -> Result<Type, SemanticError> {
        match name {
            "i8" => Ok(Type::I8),
            "i16" => Ok(Type::I16),
            "i32" => Ok(Type::I32),
            "i64" => Ok(Type::I64),
            "u8" | "byte" => Ok(Type::U8),
            "u16" => Ok(Type::U16),
            "u32" => Ok(Type::U32),
            "u64" => Ok(Type::U64),
            "f32" => Ok(Type::F32),
            "f64" => Ok(Type::F64),
            "string" => Ok(Type::String),
            "bool" => Ok(Type::Bool),
            _ => Err(SemanticError::invalid_function_return_type(name, span)),
        }
    }

    pub(super) fn resolve_user_call(
        &mut self,
        callee: &str,
        args: &[Expr],
        span: Span,
    ) -> Result<String, SemanticError> {
        let (param_types, return_type) = {
            let func_info = self
                .symbols
                .lookup_function(callee)
                .ok_or_else(|| SemanticError::undefined_function(callee, span))?;
            (func_info.param_types.clone(), func_info.return_type.clone())
        };

        if callee == "main" {
            return Err(SemanticError::invalid_argument_cannot_call_main(span));
        }

        let expected_arg_count = param_types.len();
        if args.len() != expected_arg_count {
            return Err(if expected_arg_count == 0 {
                SemanticError::invalid_argument_fn_expects_no_args(callee, args.len(), span)
            } else {
                SemanticError::invalid_argument_fn_expects_args(
                    callee,
                    expected_arg_count,
                    args.len(),
                    span,
                )
            });
        }

        for (arg, expected_ty) in args.iter().zip(param_types.iter()) {
            self.check_expr_type(arg, expected_ty)?;
        }

        Ok(return_type)
    }

    pub(super) fn resolve_module_call(
        &mut self,
        module_name: &str,
        function_name: &str,
        args: &[Expr],
        span: Span,
    ) -> Result<String, SemanticError> {
        let (param_types, return_type) = {
            let module_table = match &self.mode {
                AnalysisMode::EntryWithModules(table) => table,
                AnalysisMode::ImportedModule(Some(table)) => table,
                AnalysisMode::ImportedModule(None) => {
                    return Err(SemanticError::cross_module_call_in_imported_module(
                        module_name,
                        function_name,
                        span,
                    ));
                }
                AnalysisMode::SingleFile => {
                    return Err(SemanticError::module_not_imported(
                        module_name,
                        function_name,
                        span,
                    ));
                }
            };

            let module_exports = module_table
                .get_module(module_name)
                .ok_or_else(|| SemanticError::undefined_module(module_name, span))?;

            let func_export = module_exports.get_function(function_name).ok_or_else(|| {
                SemanticError::undefined_module_function(module_name, function_name, span)
            })?;

            (
                func_export.param_types().to_vec(),
                func_export.return_type().to_string(),
            )
        };

        let full_function_name = format!("{}.{}", module_name, function_name);
        let expected_arg_count = param_types.len();
        if args.len() != expected_arg_count {
            return Err(if expected_arg_count == 0 {
                SemanticError::invalid_argument_fn_expects_no_args(
                    &full_function_name,
                    args.len(),
                    span,
                )
            } else {
                SemanticError::invalid_argument_fn_expects_args(
                    &full_function_name,
                    expected_arg_count,
                    args.len(),
                    span,
                )
            });
        }

        for (arg, expected_ty) in args.iter().zip(param_types.iter()) {
            self.check_expr_type(arg, expected_ty)?;
        }

        Ok(return_type)
    }
}
