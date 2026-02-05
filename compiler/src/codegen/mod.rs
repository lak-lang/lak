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
use inkwell::context::Context;
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
    /// This method generates the complete LLVM IR for the program, including:
    /// - External function declarations (e.g., `lak_println`)
    /// - The `main` function as the program entry point
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
    /// - No `main` function is found
    /// - The `main` function has an invalid signature
    /// - An unknown function is called
    /// - A built-in function is called with incorrect arguments
    /// - LLVM IR generation fails
    pub fn compile(&mut self, program: &Program) -> Result<(), CodegenError> {
        self.declare_lak_println();

        // Find the main function
        let main_fn = program
            .functions
            .iter()
            .find(|f| f.name == "main")
            .ok_or_else(|| {
                if program.functions.is_empty() {
                    CodegenError::missing_main(
                        "No main function found: program contains no function definitions",
                    )
                } else {
                    let names: Vec<_> = program.functions.iter().map(|f| f.name.as_str()).collect();
                    CodegenError::missing_main(format!(
                        "No main function found. Defined functions: {:?}",
                        names
                    ))
                }
            })?;

        // Validate main function signature
        if main_fn.return_type != "void" {
            return Err(CodegenError::new(
                CodegenErrorKind::InvalidMainSignature,
                format!(
                    "main function must return void, but found return type '{}'",
                    main_fn.return_type
                ),
                main_fn.return_type_span,
            ));
        }

        self.generate_main(main_fn)?;
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
        self.builder.build_return(Some(&zero)).map_err(|e| {
            CodegenError::without_span(
                CodegenErrorKind::InternalError,
                format!(
                    "Internal error: failed to build return instruction. This is a compiler bug: {}",
                    e
                ),
            )
        })?;

        Ok(())
    }

    /// Returns the LLVM type corresponding to a Lak type.
    ///
    /// # Type Mapping
    ///
    /// - `Type::I32` → LLVM `i32`
    /// - `Type::I64` → LLVM `i64`
    fn get_llvm_type(&self, ty: &Type) -> inkwell::types::IntType<'ctx> {
        match ty {
            Type::I32 => self.context.i32_type(),
            Type::I64 => self.context.i64_type(),
        }
    }
}
