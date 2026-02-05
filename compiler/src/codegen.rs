//! LLVM code generation for the Lak programming language.
//!
//! This module provides the [`Codegen`] struct which transforms a Lak AST
//! into LLVM IR and compiles it to native object code.
//!
//! # Overview
//!
//! The code generator uses [Inkwell](https://github.com/TheDan64/inkwell),
//! a safe Rust wrapper around the LLVM C API. It performs the following tasks:
//!
//! - Creates an LLVM module and builder
//! - Generates a `main` function as the program entry point
//! - Compiles function calls (currently only `println`)
//! - Writes the output to a native object file
//!
//! # Architecture
//!
//! The generated code follows the C calling convention and links against
//! the Lak runtime library for I/O operations (using `lak_println`).
//!
//! # Examples
//!
//! ```no_run
//! use inkwell::context::Context;
//! use lak::codegen::Codegen;
//! use lak::ast::{Program, Stmt, Expr};
//! use std::path::Path;
//!
//! let context = Context::create();
//! let mut codegen = Codegen::new(&context, "example");
//!
//! let program = Program {
//!     stmts: vec![Stmt::Expr(Expr::Call {
//!         callee: "println".to_string(),
//!         args: vec![Expr::StringLiteral("Hello!".to_string())],
//!     })],
//! };
//!
//! codegen.compile(&program).unwrap();
//! codegen.write_object_file(Path::new("output.o")).unwrap();
//! ```
//!
//! # See Also
//!
//! * [`crate::ast`] - The AST types consumed by this module
//! * [Inkwell documentation](https://thedan64.github.io/inkwell/)
//! * [LLVM Language Reference](https://llvm.org/docs/LangRef.html)

use crate::ast::{Expr, Program, Stmt};
use inkwell::AddressSpace;
use inkwell::OptimizationLevel;
use inkwell::context::Context;
use inkwell::module::Linkage;
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};
use inkwell::values::BasicMetadataValueEnum;
use std::path::Path;

/// LLVM code generator for Lak programs.
///
/// `Codegen` holds the LLVM context, module, and builder required for
/// generating LLVM IR. It provides methods to compile a Lak [`Program`]
/// and write the output to an object file.
///
/// # Lifetime
///
/// The `'ctx` lifetime parameter ties this struct to an LLVM [`Context`].
/// The context must outlive the code generator.
///
/// # Thread Safety
///
/// LLVM contexts are not thread-safe. Each thread should have its own
/// context and code generator.
pub struct Codegen<'ctx> {
    /// Reference to the LLVM context.
    context: &'ctx Context,
    /// The LLVM module being built.
    module: inkwell::module::Module<'ctx>,
    /// The IR builder for creating instructions.
    builder: inkwell::builder::Builder<'ctx>,
}

impl<'ctx> Codegen<'ctx> {
    /// Creates a new code generator with the given LLVM context and module name.
    ///
    /// # Arguments
    ///
    /// * `context` - The LLVM context to use for creating IR
    /// * `module_name` - A name for the LLVM module (used in debug output)
    ///
    /// # Returns
    ///
    /// A new `Codegen` instance ready to compile programs.
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();

        Codegen {
            context,
            module,
            builder,
        }
    }

    /// Compiles a Lak program to LLVM IR.
    ///
    /// This method generates the complete LLVM IR for the program, including:
    /// - External function declarations (e.g., `lak_println`)
    /// - The `main` function with the program's statements
    ///
    /// After calling this method, use [`write_object_file`](Self::write_object_file)
    /// to output the compiled code.
    ///
    /// # Arguments
    ///
    /// * `program` - The parsed Lak program to compile
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - An unknown function is called
    /// - A built-in function is called with incorrect arguments
    /// - LLVM IR generation fails
    pub fn compile(&mut self, program: &Program) -> Result<(), String> {
        self.declare_lak_println();
        self.generate_main(program)?;
        Ok(())
    }

    /// Declares the Lak runtime `lak_println` function for use in generated code.
    ///
    /// This creates an external function declaration with the signature:
    /// `void lak_println(const char* s)`
    fn declare_lak_println(&self) {
        let void_type = self.context.void_type();
        let i8_ptr_type = self.context.ptr_type(AddressSpace::default());

        let println_type = void_type.fn_type(&[i8_ptr_type.into()], false);
        self.module
            .add_function("lak_println", println_type, Some(Linkage::External));
    }

    /// Generates the `main` function containing all program statements.
    ///
    /// Creates a function with the signature `int main()` that executes
    /// all statements in the program and returns 0 on success.
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

    /// Generates LLVM IR for a single statement.
    fn generate_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Expr(expr) => {
                self.generate_expr(expr)?;
                Ok(())
            }
        }
    }

    /// Generates LLVM IR for an expression.
    ///
    /// Currently handles:
    /// - Function calls (dispatches to built-in handlers)
    /// - String literals (only valid as function arguments; produce no IR in expression context)
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

    /// Generates LLVM IR for a `println` call.
    ///
    /// Implements `println(string)` by calling the Lak runtime `lak_println` function.
    ///
    /// # Arguments
    ///
    /// * `args` - The arguments passed to `println` (must be exactly one string literal)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The number of arguments is not exactly 1
    /// - The argument is not a string literal
    fn generate_println(&mut self, args: &[Expr]) -> Result<(), String> {
        if args.len() != 1 {
            return Err("println expects exactly 1 argument".to_string());
        }

        let arg = &args[0];
        let string_value = match arg {
            Expr::StringLiteral(s) => s,
            _ => return Err("println argument must be a string literal".to_string()),
        };

        let lak_println = self
            .module
            .get_function("lak_println")
            .ok_or("Internal error: lak_println function not found. This is a compiler bug.")?;

        let str_value = self
            .builder
            .build_global_string_ptr(string_value, "str")
            .map_err(|e| format!("Failed to create string literal: {:?}", e))?;

        self.builder
            .build_call(
                lak_println,
                &[BasicMetadataValueEnum::PointerValue(
                    str_value.as_pointer_value(),
                )],
                "",
            )
            .map_err(|e| format!("Failed to generate lak_println call: {:?}", e))?;

        Ok(())
    }

    /// Writes the compiled module to a native object file.
    ///
    /// This method initializes the native target (if not already done),
    /// creates a target machine for the host platform, and writes the
    /// compiled LLVM IR to an object file that can be linked.
    ///
    /// # Arguments
    ///
    /// * `path` - The path where the object file should be written
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Failed to initialize the native target
    /// - Failed to create the target machine
    /// - Failed to write the object file
    ///
    /// # Platform Support
    ///
    /// The object file format depends on the host platform:
    /// - macOS: Mach-O
    /// - Linux: ELF
    /// - Windows: COFF
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codegen_new() {
        let context = Context::create();
        let codegen = Codegen::new(&context, "test_module");
        assert_eq!(codegen.module.get_name().to_str().unwrap(), "test_module");
    }

    #[test]
    fn test_compile_empty_program() {
        let context = Context::create();
        let mut codegen = Codegen::new(&context, "test");

        let program = Program { stmts: vec![] };
        codegen
            .compile(&program)
            .expect("Empty program should compile");

        assert!(codegen.module.get_function("main").is_some());
    }

    #[test]
    fn test_compile_println() {
        let context = Context::create();
        let mut codegen = Codegen::new(&context, "test");

        let program = Program {
            stmts: vec![Stmt::Expr(Expr::Call {
                callee: "println".to_string(),
                args: vec![Expr::StringLiteral("hello".to_string())],
            })],
        };

        codegen
            .compile(&program)
            .expect("println program should compile");

        assert!(codegen.module.get_function("lak_println").is_some());
    }

    #[test]
    fn test_compile_multiple_println() {
        let context = Context::create();
        let mut codegen = Codegen::new(&context, "test");

        let program = Program {
            stmts: vec![
                Stmt::Expr(Expr::Call {
                    callee: "println".to_string(),
                    args: vec![Expr::StringLiteral("first".to_string())],
                }),
                Stmt::Expr(Expr::Call {
                    callee: "println".to_string(),
                    args: vec![Expr::StringLiteral("second".to_string())],
                }),
            ],
        };

        codegen
            .compile(&program)
            .expect("Multiple println program should compile");
    }

    #[test]
    fn test_compile_println_with_escape_sequences() {
        let context = Context::create();
        let mut codegen = Codegen::new(&context, "test");

        let program = Program {
            stmts: vec![Stmt::Expr(Expr::Call {
                callee: "println".to_string(),
                args: vec![Expr::StringLiteral("hello\nworld\t!".to_string())],
            })],
        };

        codegen
            .compile(&program)
            .expect("Escape sequences program should compile");
    }

    #[test]
    fn test_error_unknown_function() {
        let context = Context::create();
        let mut codegen = Codegen::new(&context, "test");

        let program = Program {
            stmts: vec![Stmt::Expr(Expr::Call {
                callee: "unknown_function".to_string(),
                args: vec![],
            })],
        };

        let err = codegen
            .compile(&program)
            .expect_err("Should fail for unknown function");
        assert!(
            err.contains("Unknown function"),
            "Expected 'Unknown function' in error: {}",
            err
        );
    }

    #[test]
    fn test_error_println_no_args() {
        let context = Context::create();
        let mut codegen = Codegen::new(&context, "test");

        let program = Program {
            stmts: vec![Stmt::Expr(Expr::Call {
                callee: "println".to_string(),
                args: vec![],
            })],
        };

        let err = codegen
            .compile(&program)
            .expect_err("Should fail for println with no args");
        assert!(
            err.contains("exactly 1 argument"),
            "Expected 'exactly 1 argument' in error: {}",
            err
        );
    }

    #[test]
    fn test_error_println_too_many_args() {
        let context = Context::create();
        let mut codegen = Codegen::new(&context, "test");

        let program = Program {
            stmts: vec![Stmt::Expr(Expr::Call {
                callee: "println".to_string(),
                args: vec![
                    Expr::StringLiteral("a".to_string()),
                    Expr::StringLiteral("b".to_string()),
                ],
            })],
        };

        let err = codegen
            .compile(&program)
            .expect_err("Should fail for println with too many args");
        assert!(
            err.contains("exactly 1 argument"),
            "Expected 'exactly 1 argument' in error: {}",
            err
        );
    }

    #[test]
    fn test_error_println_non_string() {
        let context = Context::create();
        let mut codegen = Codegen::new(&context, "test");

        let program = Program {
            stmts: vec![Stmt::Expr(Expr::Call {
                callee: "println".to_string(),
                args: vec![Expr::Call {
                    callee: "other".to_string(),
                    args: vec![],
                }],
            })],
        };

        let err = codegen
            .compile(&program)
            .expect_err("Should fail for println with non-string arg");
        assert!(
            err.contains("string literal"),
            "Expected 'string literal' in error: {}",
            err
        );
    }

    #[test]
    fn test_write_object_file() {
        let context = Context::create();
        let mut codegen = Codegen::new(&context, "test");

        let program = Program {
            stmts: vec![Stmt::Expr(Expr::Call {
                callee: "println".to_string(),
                args: vec![Expr::StringLiteral("test".to_string())],
            })],
        };

        codegen.compile(&program).unwrap();

        let temp_dir = tempfile::tempdir().unwrap();
        let object_path = temp_dir.path().join("test.o");

        let result = codegen.write_object_file(&object_path);
        assert!(result.is_ok());
        assert!(object_path.exists());
    }

    #[test]
    fn test_module_name() {
        let context = Context::create();
        let codegen = Codegen::new(&context, "my_custom_module");
        assert_eq!(
            codegen.module.get_name().to_str().unwrap(),
            "my_custom_module"
        );
    }

    #[test]
    fn test_main_function_signature() {
        let context = Context::create();
        let mut codegen = Codegen::new(&context, "test");

        let program = Program { stmts: vec![] };
        codegen.compile(&program).unwrap();

        let main_fn = codegen.module.get_function("main").unwrap();
        // main returns i32
        assert!(main_fn.get_type().get_return_type().is_some());
        // main takes no arguments
        assert_eq!(main_fn.count_params(), 0);
    }
}
