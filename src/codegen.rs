use crate::ast::{Expr, Program, Stmt};
use inkwell::context::Context;
use inkwell::module::Linkage;
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};
use inkwell::values::BasicMetadataValueEnum;
use inkwell::AddressSpace;
use inkwell::OptimizationLevel;
use std::path::Path;

pub struct Codegen<'ctx> {
    context: &'ctx Context,
    module: inkwell::module::Module<'ctx>,
    builder: inkwell::builder::Builder<'ctx>,
}

impl<'ctx> Codegen<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();

        Codegen {
            context,
            module,
            builder,
        }
    }

    pub fn compile(&mut self, program: &Program) -> Result<(), String> {
        self.declare_printf();
        self.generate_main(program)?;
        Ok(())
    }

    fn declare_printf(&self) {
        let i32_type = self.context.i32_type();
        let i8_ptr_type = self.context.ptr_type(AddressSpace::default());

        let printf_type = i32_type.fn_type(&[i8_ptr_type.into()], true);
        self.module
            .add_function("printf", printf_type, Some(Linkage::External));
    }

    fn generate_main(&mut self, program: &Program) -> Result<(), String> {
        let i32_type = self.context.i32_type();
        let main_type = i32_type.fn_type(&[], false);
        let main_fn = self.module.add_function("main", main_type, None);

        let entry = self.context.append_basic_block(main_fn, "entry");
        self.builder.position_at_end(entry);

        for stmt in &program.stmts {
            self.generate_stmt(stmt)?;
        }

        let zero = i32_type.const_int(0, false);
        self.builder
            .build_return(Some(&zero))
            .map_err(|e| format!("Failed to build return instruction: {:?}", e))?;

        Ok(())
    }

    fn generate_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Expr(expr) => {
                self.generate_expr(expr)?;
                Ok(())
            }
        }
    }

    fn generate_expr(&mut self, expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::Call { callee, args } => {
                if callee == "println" {
                    self.generate_println(args)?;
                } else {
                    return Err(format!("Unknown function: {}", callee));
                }
            }
            Expr::StringLiteral(_) => {}
        }
        Ok(())
    }

    fn generate_println(&mut self, args: &[Expr]) -> Result<(), String> {
        if args.len() != 1 {
            return Err("println expects exactly 1 argument".to_string());
        }

        let arg = &args[0];
        let string_value = match arg {
            Expr::StringLiteral(s) => s,
            _ => return Err("println argument must be a string literal".to_string()),
        };

        let printf = self
            .module
            .get_function("printf")
            .ok_or("Internal error: printf function not found. This is a compiler bug.")?;

        let format_str = self
            .builder
            .build_global_string_ptr("%s\n", "fmt")
            .map_err(|e| format!("Failed to create format string: {:?}", e))?;

        let str_value = self
            .builder
            .build_global_string_ptr(string_value, "str")
            .map_err(|e| format!("Failed to create string literal: {:?}", e))?;

        self.builder
            .build_call(
                printf,
                &[
                    BasicMetadataValueEnum::PointerValue(format_str.as_pointer_value()),
                    BasicMetadataValueEnum::PointerValue(str_value.as_pointer_value()),
                ],
                "printf_call",
            )
            .map_err(|e| format!("Failed to generate printf call: {:?}", e))?;

        Ok(())
    }

    pub fn write_object_file(&self, path: &Path) -> Result<(), String> {
        Target::initialize_native(&InitializationConfig::default())
            .map_err(|e| format!("Failed to initialize native target: {}", e))?;

        let triple = TargetMachine::get_default_triple();
        let target =
            Target::from_triple(&triple).map_err(|e| format!("Failed to get target: {}", e))?;

        let cpu = TargetMachine::get_host_cpu_name();
        let features = TargetMachine::get_host_cpu_features();

        let cpu_str = cpu
            .to_str()
            .map_err(|_| "CPU name contains invalid UTF-8")?;
        let features_str = features
            .to_str()
            .map_err(|_| "CPU features contain invalid UTF-8")?;

        let target_machine = target
            .create_target_machine(
                &triple,
                cpu_str,
                features_str,
                OptimizationLevel::Default,
                RelocMode::Default,
                CodeModel::Default,
            )
            .ok_or("Failed to create target machine")?;

        target_machine
            .write_to_file(&self.module, FileType::Object, path)
            .map_err(|e| format!("Failed to write object file: {}", e))?;

        Ok(())
    }
}
