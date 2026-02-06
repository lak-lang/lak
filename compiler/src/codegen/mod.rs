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
//! - Handles variable declarations (`let` statements) with stack allocation
//! - Writes the output to a native object file
//!
//! # Architecture
//!
//! The generated code follows the C calling convention and links against
//! the Lak runtime library for I/O operations (using `lak_println`).
//!
//! # Example
//!
//! ```no_run
//! use inkwell::context::Context;
//! use lak::codegen::Codegen;
//! use lak::ast::{Program, FnDef, Stmt, StmtKind, Expr, ExprKind};
//! use lak::token::Span;
//! use std::path::Path;
//!
//! let context = Context::create();
//! let mut codegen = Codegen::new(&context, "example");
//!
//! let program = Program {
//!     functions: vec![FnDef {
//!         name: "main".to_string(),
//!         return_type: "void".to_string(),
//!         return_type_span: Span::new(0, 0, 1, 1),
//!         body: vec![Stmt::new(
//!             StmtKind::Expr(Expr::new(
//!                 ExprKind::Call {
//!                     callee: "println".to_string(),
//!                     args: vec![Expr::new(
//!                         ExprKind::StringLiteral("Hello!".to_string()),
//!                         Span::new(0, 0, 1, 1),
//!                     )],
//!                 },
//!                 Span::new(0, 0, 1, 1),
//!             )),
//!             Span::new(0, 0, 1, 1),
//!         )],
//!         span: Span::new(0, 0, 1, 1),
//!     }],
//! };
//!
//! codegen.compile(&program).unwrap();
//! codegen.write_object_file(Path::new("output.o")).unwrap();
//! ```
//!
//! # Module Structure
//!
//! - [`error`] - Error types for code generation
//! - [`binding`] - Variable binding management
//! - [`stmt`] - Statement code generation
//! - [`expr`] - Expression code generation
//! - [`builtins`] - Built-in function implementations
//! - [`target`] - Target machine and object file output
//! - `tests` - Unit tests (test-only)
//!
//! # See Also
//!
//! * [`crate::ast`] - The AST types consumed by this module
//! * [Inkwell documentation](https://thedan64.github.io/inkwell/)
//! * [LLVM Language Reference](https://llvm.org/docs/LangRef.html)

mod binding;
mod builtins;
mod error;
mod expr;
mod stmt;
mod target;

#[cfg(test)]
mod tests;

pub use error::{CodegenError, CodegenErrorKind};

use crate::ast::{FnDef, Program, Type};
use binding::VarBinding;
use inkwell::AddressSpace;
use inkwell::context::Context;
use inkwell::types::BasicTypeEnum;
use std::collections::HashMap;

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
    /// Symbol table mapping variable names to their allocations.
    ///
    /// This table is cleared when generating the main function to implement
    /// function-scoped variables. Currently only main is supported; this will
    /// be cleared for each function once multiple user-defined functions are
    /// implemented.
    variables: HashMap<String, VarBinding<'ctx>>,
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
            variables: HashMap::new(),
        }
    }

    /// Compiles a Lak program to LLVM IR.
    ///
    /// This method generates the complete LLVM IR for the program using a two-pass
    /// approach to allow functions to call each other regardless of definition order:
    ///
    /// 1. **Pass 1**: Declare all functions (create LLVM function signatures)
    /// 2. **Pass 2**: Generate function bodies
    ///
    /// After calling this method, use [`write_object_file`](Self::write_object_file)
    /// to output the compiled code.
    ///
    /// # Arguments
    ///
    /// * `program` - A semantically validated Lak program to compile
    ///
    /// # Errors
    ///
    /// Returns an error if LLVM IR generation fails (internal errors).
    pub fn compile(&mut self, program: &Program) -> Result<(), CodegenError> {
        // Declare built-in functions
        self.declare_lak_println();
        self.declare_lak_println_i32();
        self.declare_lak_println_i64();
        self.declare_lak_panic();

        // Pass 1: Declare all user-defined functions (except main, which has a special signature)
        for function in &program.functions {
            if function.name != "main" {
                self.declare_user_function(function)?;
            }
        }

        // Pass 2: Generate function bodies
        for function in &program.functions {
            if function.name == "main" {
                self.generate_main(function)?;
            } else {
                self.generate_user_function(function)?;
            }
        }

        Ok(())
    }

    /// Declares a user-defined function (creates LLVM function signature only).
    ///
    /// This method is called in Pass 1 to create function declarations before
    /// any function bodies are generated. This allows forward references.
    ///
    /// # Arguments
    ///
    /// * `fn_def` - The function definition from the AST
    fn declare_user_function(&mut self, fn_def: &FnDef) -> Result<(), CodegenError> {
        // Currently only void functions with no parameters are supported
        let void_type = self.context.void_type();
        let fn_type = void_type.fn_type(&[], false);

        self.module.add_function(&fn_def.name, fn_type, None);
        Ok(())
    }

    /// Generates the body of a user-defined function.
    ///
    /// Creates the function body with an entry block, generates all statements,
    /// and adds a void return at the end.
    ///
    /// # Arguments
    ///
    /// * `fn_def` - The function definition from the AST
    fn generate_user_function(&mut self, fn_def: &FnDef) -> Result<(), CodegenError> {
        // Clear variable table for this function's scope
        self.variables.clear();

        // Get the function declaration created in Pass 1
        let function = self
            .module
            .get_function(&fn_def.name)
            .ok_or_else(|| CodegenError::internal_function_not_found_no_span(&fn_def.name))?;

        // Create entry block
        let entry = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry);

        // Generate statements
        for stmt in &fn_def.body {
            self.generate_stmt(stmt)?;
        }

        // Add void return
        self.builder.build_return(None).map_err(|e| {
            CodegenError::internal_return_build_failed(&fn_def.name, &e.to_string())
        })?;

        Ok(())
    }

    /// Generates the `main` function from a Lak function definition.
    ///
    /// Creates an LLVM function with the signature `int main()` that executes
    /// all statements in the function body and returns 0 on success.
    ///
    /// # Arguments
    ///
    /// * `main_fn_def` - The Lak `main` function definition
    fn generate_main(&mut self, main_fn_def: &FnDef) -> Result<(), CodegenError> {
        self.variables.clear();

        let i32_type = self.context.i32_type();
        let main_type = i32_type.fn_type(&[], false);
        let main_fn = self.module.add_function("main", main_type, None);

        let entry = self.context.append_basic_block(main_fn, "entry");
        self.builder.position_at_end(entry);

        for stmt in &main_fn_def.body {
            self.generate_stmt(stmt)?;
        }

        let zero = i32_type.const_int(0, false);
        self.builder
            .build_return(Some(&zero))
            .map_err(|e| CodegenError::internal_main_return_build_failed(&e.to_string()))?;

        Ok(())
    }

    /// Returns the LLVM type corresponding to a Lak type.
    ///
    /// # Type Mapping
    ///
    /// - `Type::I32` → LLVM `i32`
    /// - `Type::I64` → LLVM `i64`
    /// - `Type::String` → LLVM `ptr` (i8*)
    fn get_llvm_type(&self, ty: &Type) -> BasicTypeEnum<'ctx> {
        match ty {
            Type::I32 => self.context.i32_type().into(),
            Type::I64 => self.context.i64_type().into(),
            Type::String => self.context.ptr_type(AddressSpace::default()).into(),
        }
    }
}
