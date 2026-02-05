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
//!     }],
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

use crate::ast::{Expr, ExprKind, FnDef, Program, Stmt, StmtKind, Type};
use crate::token::Span;
use inkwell::AddressSpace;
use inkwell::OptimizationLevel;
use inkwell::context::Context;
use inkwell::module::Linkage;
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum, PointerValue};
use std::collections::HashMap;
use std::path::Path;

/// An error that occurred during code generation.
///
/// Contains a human-readable message and optionally the source location
/// where the error occurred, enabling rich error reporting.
#[derive(Debug)]
pub struct CodegenError {
    /// A human-readable description of the error.
    pub message: String,
    /// The source location where the error occurred, if available.
    pub span: Option<Span>,
}

impl CodegenError {
    /// Creates a new error with a message and source location.
    pub fn new(message: impl Into<String>, span: Span) -> Self {
        CodegenError {
            message: message.into(),
            span: Some(span),
        }
    }

    /// Creates a new error with only a message (no source location).
    pub fn without_span(message: impl Into<String>) -> Self {
        CodegenError {
            message: message.into(),
            span: None,
        }
    }
}

impl std::fmt::Display for CodegenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(span) = &self.span {
            write!(f, "{}:{}: {}", span.line, span.column, self.message)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

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
    /// This table is cleared at the start of each function to implement
    /// function-scoped variables.
    variables: HashMap<String, VarBinding<'ctx>>,
}

/// A variable binding in the symbol table.
///
/// Stores the stack allocation pointer and declared type for a variable,
/// enabling variable lookups and type checking during code generation.
///
/// # Invariants
///
/// The LLVM type of `alloca` must correspond to `ty`:
/// - `Type::I32` → `alloca` points to an LLVM `i32`
/// - `Type::I64` → `alloca` points to an LLVM `i64`
///
/// This invariant is enforced by creating bindings only through
/// [`VarBinding::new`], which allocates the correct LLVM type.
#[derive(Clone, Debug)]
struct VarBinding<'ctx> {
    /// The stack allocation for this variable.
    alloca: PointerValue<'ctx>,
    /// The declared type of this variable.
    ty: Type,
}

impl<'ctx> VarBinding<'ctx> {
    /// Creates a new variable binding with a stack allocation.
    ///
    /// This constructor ensures the invariant that `alloca`'s LLVM type
    /// matches the declared `ty`.
    ///
    /// # Arguments
    ///
    /// * `builder` - The LLVM IR builder
    /// * `context` - The LLVM context
    /// * `ty` - The Lak type for this variable
    /// * `name` - The variable name (used for LLVM IR naming)
    ///
    /// # Returns
    ///
    /// * `Ok(VarBinding)` - A new binding with a correctly-typed stack allocation.
    /// * `Err(String)` - If LLVM fails to create the alloca instruction.
    fn new(
        builder: &inkwell::builder::Builder<'ctx>,
        context: &'ctx Context,
        ty: &Type,
        name: &str,
    ) -> Result<Self, String> {
        let llvm_type = match ty {
            Type::I32 => context.i32_type(),
            Type::I64 => context.i64_type(),
        };
        let alloca = builder
            .build_alloca(llvm_type, name)
            .map_err(|e| format!("Failed to allocate variable '{}': {:?}", name, e))?;
        Ok(VarBinding {
            alloca,
            ty: ty.clone(),
        })
    }

    /// Returns the stack allocation pointer for this variable.
    fn alloca(&self) -> PointerValue<'ctx> {
        self.alloca
    }

    /// Returns the declared type of this variable.
    fn ty(&self) -> &Type {
        &self.ty
    }
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
                    CodegenError::without_span(
                        "No main function found: program contains no function definitions",
                    )
                } else {
                    let names: Vec<_> = program.functions.iter().map(|f| f.name.as_str()).collect();
                    CodegenError::without_span(format!(
                        "No main function found. Defined functions: {:?}",
                        names
                    ))
                }
            })?;

        // Validate main function signature
        if main_fn.return_type != "void" {
            return Err(CodegenError::without_span(format!(
                "main function must return void, but found return type '{}'",
                main_fn.return_type
            )));
        }

        self.generate_main(main_fn)?;
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

    /// Generates the `main` function from a Lak function definition.
    ///
    /// Creates an LLVM function with the signature `int main()` that executes
    /// all statements in the function body and returns 0 on success.
    ///
    /// # Arguments
    ///
    /// * `main_fn_def` - The Lak `main` function definition
    fn generate_main(&mut self, main_fn_def: &FnDef) -> Result<(), CodegenError> {
        // Clear the symbol table for this function scope
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
            CodegenError::without_span(format!("Failed to build return instruction: {:?}", e))
        })?;

        Ok(())
    }

    /// Generates LLVM IR for a single statement.
    fn generate_stmt(&mut self, stmt: &Stmt) -> Result<(), CodegenError> {
        match &stmt.kind {
            StmtKind::Expr(expr) => {
                self.generate_expr(expr)?;
                Ok(())
            }
            StmtKind::Let { name, ty, init } => self.generate_let(name, ty, init, stmt.span),
        }
    }

    /// Generates LLVM IR for an expression used as a statement.
    ///
    /// # Behavior
    ///
    /// - For function calls: generates the call IR and returns `Ok(())`
    /// - For literals and identifiers: returns an error since these have no
    ///   side effects when used as statements
    fn generate_expr(&mut self, expr: &Expr) -> Result<(), CodegenError> {
        match &expr.kind {
            ExprKind::Call { callee, args } => {
                if callee == "println" {
                    self.generate_println(args, expr.span)?;
                } else {
                    return Err(CodegenError::new(
                        format!("Unknown function: {}", callee),
                        expr.span,
                    ));
                }
            }
            ExprKind::StringLiteral(_) => {
                return Err(CodegenError::new(
                    "String literal as a statement has no effect. Did you mean to pass it to a function?",
                    expr.span,
                ));
            }
            ExprKind::IntLiteral(_) => {
                return Err(CodegenError::new(
                    "Integer literal as a statement has no effect. Did you mean to assign it to a variable?",
                    expr.span,
                ));
            }
            ExprKind::Identifier(name) => {
                return Err(CodegenError::new(
                    format!(
                        "Variable '{}' used as a statement has no effect. Did you mean to use it in an expression?",
                        name
                    ),
                    expr.span,
                ));
            }
        }
        Ok(())
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
    /// Returns an error if:
    /// - The variable is already defined in the current scope
    /// - The initializer expression references an undefined variable
    /// - The initializer variable has a type different from the declared type
    /// - An integer literal is out of range for the declared type (e.g., i32)
    /// - LLVM fails to create allocation or store instructions
    fn generate_let(
        &mut self,
        name: &str,
        ty: &Type,
        init: &Expr,
        span: Span,
    ) -> Result<(), CodegenError> {
        // Check for duplicate variable
        if self.variables.contains_key(name) {
            return Err(CodegenError::new(
                format!("Variable '{}' is already defined in this scope", name),
                span,
            ));
        }

        // Create the variable binding with stack allocation
        let binding = VarBinding::new(&self.builder, self.context, ty, name)
            .map_err(|e| CodegenError::new(e, span))?;

        // Evaluate the initializer expression
        let init_value = self.generate_expr_value(init, ty)?;

        // Store the initial value
        self.builder
            .build_store(binding.alloca(), init_value)
            .map_err(|e| {
                CodegenError::new(
                    format!("Failed to store initial value for '{}': {:?}", name, e),
                    span,
                )
            })?;

        // Register the variable in the symbol table
        self.variables.insert(name.to_string(), binding);

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

    /// Generates LLVM IR for an expression and returns its value.
    ///
    /// This method is used when an expression's value is needed (e.g., as
    /// an initializer for a variable).
    ///
    /// # Arguments
    ///
    /// * `expr` - The expression to evaluate
    /// * `expected_ty` - The expected type of the result
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - A referenced variable is undefined
    /// - A variable's type does not match the expected type
    /// - An integer literal is out of range for the expected type
    /// - A string literal is used (strings cannot be converted to integer types)
    /// - A function call is used (functions returning values not yet supported)
    fn generate_expr_value(
        &self,
        expr: &Expr,
        expected_ty: &Type,
    ) -> Result<BasicValueEnum<'ctx>, CodegenError> {
        match &expr.kind {
            ExprKind::IntLiteral(value) => {
                // Validate that the value fits in the expected type
                match expected_ty {
                    Type::I32 => {
                        if *value < i32::MIN as i64 || *value > i32::MAX as i64 {
                            return Err(CodegenError::new(
                                format!(
                                    "Integer literal {} is out of range for i32 (valid range: {} to {})",
                                    value,
                                    i32::MIN,
                                    i32::MAX
                                ),
                                expr.span,
                            ));
                        }
                    }
                    Type::I64 => {
                        // Values exceeding i64 range are caught during lexing
                    }
                }

                let llvm_type = self.get_llvm_type(expected_ty);
                // Cast to u64 to preserve the two's complement bit pattern for negative values.
                // The second parameter (sign_extend=true) indicates that the provided u64 should
                // be treated as a signed value when the LLVM type is wider than the bit pattern.
                let llvm_value = llvm_type.const_int(*value as u64, true);
                Ok(llvm_value.into())
            }
            ExprKind::Identifier(name) => {
                let binding = self.variables.get(name).ok_or_else(|| {
                    CodegenError::new(format!("Undefined variable: '{}'", name), expr.span)
                })?;

                // Type check
                if *binding.ty() != *expected_ty {
                    return Err(CodegenError::new(
                        format!(
                            "Type mismatch: variable '{}' has type '{}', expected '{}'",
                            name,
                            binding.ty(),
                            expected_ty
                        ),
                        expr.span,
                    ));
                }

                let llvm_type = self.get_llvm_type(binding.ty());
                let loaded = self
                    .builder
                    .build_load(llvm_type, binding.alloca(), &format!("{}_load", name))
                    .map_err(|e| {
                        CodegenError::new(
                            format!("Failed to load variable '{}': {:?}", name, e),
                            expr.span,
                        )
                    })?;

                Ok(loaded)
            }
            ExprKind::StringLiteral(_) => Err(CodegenError::new(
                "String literals cannot be used as integer values",
                expr.span,
            )),
            ExprKind::Call { callee, .. } => Err(CodegenError::new(
                format!(
                    "Function call '{}' cannot be used as a value (functions returning values not yet supported)",
                    callee
                ),
                expr.span,
            )),
        }
    }

    /// Generates LLVM IR for a `println` call.
    ///
    /// Implements `println(string)` by calling the Lak runtime `lak_println` function.
    ///
    /// # Arguments
    ///
    /// * `args` - The arguments passed to `println` (must be exactly one string literal)
    /// * `span` - The source location of the println call
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The number of arguments is not exactly 1
    /// - The argument is not a string literal
    fn generate_println(&mut self, args: &[Expr], span: Span) -> Result<(), CodegenError> {
        if args.len() != 1 {
            return Err(CodegenError::new(
                "println expects exactly 1 argument",
                span,
            ));
        }

        let arg = &args[0];
        let string_value = match &arg.kind {
            ExprKind::StringLiteral(s) => s,
            _ => {
                return Err(CodegenError::new(
                    "println argument must be a string literal",
                    arg.span,
                ));
            }
        };

        let lak_println = self.module.get_function("lak_println").ok_or_else(|| {
            CodegenError::without_span(
                "Internal error: lak_println function not found. This is a compiler bug.",
            )
        })?;

        let str_value = self
            .builder
            .build_global_string_ptr(string_value, "str")
            .map_err(|e| {
                CodegenError::new(
                    format!("Failed to create string literal: {:?}", e),
                    arg.span,
                )
            })?;

        self.builder
            .build_call(
                lak_println,
                &[BasicMetadataValueEnum::PointerValue(
                    str_value.as_pointer_value(),
                )],
                "",
            )
            .map_err(|e| {
                CodegenError::new(
                    format!("Failed to generate lak_println call: {:?}", e),
                    span,
                )
            })?;

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

    fn dummy_span() -> Span {
        Span::new(0, 0, 1, 1)
    }

    /// Helper to create a Program with a main function
    fn make_program(body: Vec<Stmt>) -> Program {
        Program {
            functions: vec![FnDef {
                name: "main".to_string(),
                return_type: "void".to_string(),
                body,
            }],
        }
    }

    /// Helper to create an expression statement
    fn expr_stmt(kind: ExprKind) -> Stmt {
        let expr = Expr::new(kind, dummy_span());
        Stmt::new(StmtKind::Expr(expr), dummy_span())
    }

    /// Helper to create a let statement
    fn let_stmt(name: &str, ty: Type, init_kind: ExprKind) -> Stmt {
        let init = Expr::new(init_kind, dummy_span());
        Stmt::new(
            StmtKind::Let {
                name: name.to_string(),
                ty,
                init,
            },
            dummy_span(),
        )
    }

    #[test]
    fn test_codegen_new() {
        let context = Context::create();
        let codegen = Codegen::new(&context, "test_module");
        assert_eq!(codegen.module.get_name().to_str().unwrap(), "test_module");
    }

    #[test]
    fn test_compile_empty_main() {
        let context = Context::create();
        let mut codegen = Codegen::new(&context, "test");

        let program = make_program(vec![]);
        codegen
            .compile(&program)
            .expect("Empty main should compile");

        assert!(codegen.module.get_function("main").is_some());
    }

    #[test]
    fn test_compile_println() {
        let context = Context::create();
        let mut codegen = Codegen::new(&context, "test");

        let program = make_program(vec![expr_stmt(ExprKind::Call {
            callee: "println".to_string(),
            args: vec![Expr::new(
                ExprKind::StringLiteral("hello".to_string()),
                dummy_span(),
            )],
        })]);

        codegen
            .compile(&program)
            .expect("println program should compile");

        assert!(codegen.module.get_function("lak_println").is_some());
    }

    #[test]
    fn test_compile_multiple_println() {
        let context = Context::create();
        let mut codegen = Codegen::new(&context, "test");

        let program = make_program(vec![
            expr_stmt(ExprKind::Call {
                callee: "println".to_string(),
                args: vec![Expr::new(
                    ExprKind::StringLiteral("first".to_string()),
                    dummy_span(),
                )],
            }),
            expr_stmt(ExprKind::Call {
                callee: "println".to_string(),
                args: vec![Expr::new(
                    ExprKind::StringLiteral("second".to_string()),
                    dummy_span(),
                )],
            }),
        ]);

        codegen
            .compile(&program)
            .expect("Multiple println program should compile");
    }

    #[test]
    fn test_compile_println_with_escape_sequences() {
        let context = Context::create();
        let mut codegen = Codegen::new(&context, "test");

        let program = make_program(vec![expr_stmt(ExprKind::Call {
            callee: "println".to_string(),
            args: vec![Expr::new(
                ExprKind::StringLiteral("hello\nworld\t!".to_string()),
                dummy_span(),
            )],
        })]);

        codegen
            .compile(&program)
            .expect("Escape sequences program should compile");
    }

    #[test]
    fn test_error_no_main_function() {
        let context = Context::create();
        let mut codegen = Codegen::new(&context, "test");

        let program = Program { functions: vec![] };

        let err = codegen
            .compile(&program)
            .expect_err("Should fail without main function");
        assert!(
            err.message.contains("No main function"),
            "Expected 'No main function' in error: {}",
            err.message
        );
    }

    #[test]
    fn test_error_main_wrong_return_type() {
        let context = Context::create();
        let mut codegen = Codegen::new(&context, "test");

        let program = Program {
            functions: vec![FnDef {
                name: "main".to_string(),
                return_type: "int".to_string(),
                body: vec![],
            }],
        };

        let err = codegen
            .compile(&program)
            .expect_err("Should fail for main with wrong return type");
        assert!(
            err.message.contains("must return void"),
            "Expected 'must return void' in error: {}",
            err.message
        );
    }

    #[test]
    fn test_error_unknown_function() {
        let context = Context::create();
        let mut codegen = Codegen::new(&context, "test");

        let program = make_program(vec![expr_stmt(ExprKind::Call {
            callee: "unknown_function".to_string(),
            args: vec![],
        })]);

        let err = codegen
            .compile(&program)
            .expect_err("Should fail for unknown function");
        assert!(
            err.message.contains("Unknown function"),
            "Expected 'Unknown function' in error: {}",
            err.message
        );
    }

    #[test]
    fn test_error_println_no_args() {
        let context = Context::create();
        let mut codegen = Codegen::new(&context, "test");

        let program = make_program(vec![expr_stmt(ExprKind::Call {
            callee: "println".to_string(),
            args: vec![],
        })]);

        let err = codegen
            .compile(&program)
            .expect_err("Should fail for println with no args");
        assert!(
            err.message.contains("exactly 1 argument"),
            "Expected 'exactly 1 argument' in error: {}",
            err.message
        );
    }

    #[test]
    fn test_error_println_too_many_args() {
        let context = Context::create();
        let mut codegen = Codegen::new(&context, "test");

        let program = make_program(vec![expr_stmt(ExprKind::Call {
            callee: "println".to_string(),
            args: vec![
                Expr::new(ExprKind::StringLiteral("a".to_string()), dummy_span()),
                Expr::new(ExprKind::StringLiteral("b".to_string()), dummy_span()),
            ],
        })]);

        let err = codegen
            .compile(&program)
            .expect_err("Should fail for println with too many args");
        assert!(
            err.message.contains("exactly 1 argument"),
            "Expected 'exactly 1 argument' in error: {}",
            err.message
        );
    }

    #[test]
    fn test_error_println_non_string() {
        let context = Context::create();
        let mut codegen = Codegen::new(&context, "test");

        let program = make_program(vec![expr_stmt(ExprKind::Call {
            callee: "println".to_string(),
            args: vec![Expr::new(
                ExprKind::Call {
                    callee: "other".to_string(),
                    args: vec![],
                },
                dummy_span(),
            )],
        })]);

        let err = codegen
            .compile(&program)
            .expect_err("Should fail for println with non-string arg");
        assert!(
            err.message.contains("string literal"),
            "Expected 'string literal' in error: {}",
            err.message
        );
    }

    #[test]
    fn test_write_object_file() {
        let context = Context::create();
        let mut codegen = Codegen::new(&context, "test");

        let program = make_program(vec![expr_stmt(ExprKind::Call {
            callee: "println".to_string(),
            args: vec![Expr::new(
                ExprKind::StringLiteral("test".to_string()),
                dummy_span(),
            )],
        })]);

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

        let program = make_program(vec![]);
        codegen.compile(&program).unwrap();

        let main_fn = codegen.module.get_function("main").unwrap();
        // main returns i32
        assert!(main_fn.get_type().get_return_type().is_some());
        // main takes no arguments
        assert_eq!(main_fn.count_params(), 0);
    }

    #[test]
    fn test_main_function_not_first() {
        // Verify that main is found even when it's not the first function
        let context = Context::create();
        let mut codegen = Codegen::new(&context, "test");

        let program = Program {
            functions: vec![
                FnDef {
                    name: "helper".to_string(),
                    return_type: "void".to_string(),
                    body: vec![],
                },
                FnDef {
                    name: "main".to_string(),
                    return_type: "void".to_string(),
                    body: vec![],
                },
            ],
        };

        codegen
            .compile(&program)
            .expect("Should find main even if not first");
        assert!(codegen.module.get_function("main").is_some());
    }

    #[test]
    fn test_error_no_main_with_other_functions() {
        // Verify error message includes defined function names
        let context = Context::create();
        let mut codegen = Codegen::new(&context, "test");

        let program = Program {
            functions: vec![
                FnDef {
                    name: "foo".to_string(),
                    return_type: "void".to_string(),
                    body: vec![],
                },
                FnDef {
                    name: "bar".to_string(),
                    return_type: "void".to_string(),
                    body: vec![],
                },
            ],
        };

        let err = codegen
            .compile(&program)
            .expect_err("Should fail without main");
        assert!(
            err.message.contains("foo") && err.message.contains("bar"),
            "Error should list defined functions: {}",
            err.message
        );
    }

    // ===================
    // Let statement tests
    // ===================

    #[test]
    fn test_compile_let_i32() {
        let context = Context::create();
        let mut codegen = Codegen::new(&context, "test");

        let program = make_program(vec![let_stmt("x", Type::I32, ExprKind::IntLiteral(42))]);

        codegen
            .compile(&program)
            .expect("Let statement should compile");
    }

    #[test]
    fn test_compile_let_i64() {
        let context = Context::create();
        let mut codegen = Codegen::new(&context, "test");

        let program = make_program(vec![let_stmt("y", Type::I64, ExprKind::IntLiteral(100))]);

        codegen
            .compile(&program)
            .expect("Let i64 statement should compile");
    }

    #[test]
    fn test_compile_multiple_let_statements() {
        let context = Context::create();
        let mut codegen = Codegen::new(&context, "test");

        let program = make_program(vec![
            let_stmt("a", Type::I32, ExprKind::IntLiteral(1)),
            let_stmt("b", Type::I32, ExprKind::IntLiteral(2)),
            let_stmt("c", Type::I64, ExprKind::IntLiteral(3)),
        ]);

        codegen
            .compile(&program)
            .expect("Multiple let statements should compile");
    }

    #[test]
    fn test_compile_let_with_variable_reference() {
        let context = Context::create();
        let mut codegen = Codegen::new(&context, "test");

        let program = make_program(vec![
            let_stmt("x", Type::I32, ExprKind::IntLiteral(42)),
            let_stmt("y", Type::I32, ExprKind::Identifier("x".to_string())),
        ]);

        codegen
            .compile(&program)
            .expect("Let with variable reference should compile");
    }

    #[test]
    fn test_compile_let_mixed_with_println() {
        let context = Context::create();
        let mut codegen = Codegen::new(&context, "test");

        let program = make_program(vec![
            let_stmt("x", Type::I32, ExprKind::IntLiteral(42)),
            expr_stmt(ExprKind::Call {
                callee: "println".to_string(),
                args: vec![Expr::new(
                    ExprKind::StringLiteral("hello".to_string()),
                    dummy_span(),
                )],
            }),
            let_stmt("y", Type::I64, ExprKind::IntLiteral(100)),
        ]);

        codegen
            .compile(&program)
            .expect("Mixed let and println should compile");
    }

    #[test]
    fn test_error_duplicate_variable() {
        let context = Context::create();
        let mut codegen = Codegen::new(&context, "test");

        let program = make_program(vec![
            let_stmt("x", Type::I32, ExprKind::IntLiteral(1)),
            let_stmt("x", Type::I32, ExprKind::IntLiteral(2)),
        ]);

        let err = codegen
            .compile(&program)
            .expect_err("Should fail on duplicate variable");
        assert!(
            err.message.contains("already defined"),
            "Expected 'already defined' in error: {}",
            err.message
        );
    }

    #[test]
    fn test_error_undefined_variable() {
        let context = Context::create();
        let mut codegen = Codegen::new(&context, "test");

        let program = make_program(vec![let_stmt(
            "x",
            Type::I32,
            ExprKind::Identifier("undefined_var".to_string()),
        )]);

        let err = codegen
            .compile(&program)
            .expect_err("Should fail on undefined variable");
        assert!(
            err.message.contains("Undefined variable"),
            "Expected 'Undefined variable' in error: {}",
            err.message
        );
    }

    #[test]
    fn test_error_type_mismatch() {
        let context = Context::create();
        let mut codegen = Codegen::new(&context, "test");

        let program = make_program(vec![
            let_stmt("x", Type::I32, ExprKind::IntLiteral(42)),
            let_stmt("y", Type::I64, ExprKind::Identifier("x".to_string())),
        ]);

        let err = codegen
            .compile(&program)
            .expect_err("Should fail on type mismatch");
        assert!(
            err.message.contains("Type mismatch"),
            "Expected 'Type mismatch' in error: {}",
            err.message
        );
    }

    #[test]
    fn test_codegen_error_with_span() {
        let span = Span::new(10, 20, 3, 5);
        let err = CodegenError::new("test error", span);
        assert!(err.span.is_some());
        assert_eq!(err.span.unwrap().line, 3);
        assert_eq!(err.span.unwrap().column, 5);
        let display = format!("{}", err);
        assert!(display.contains("3:5"));
        assert!(display.contains("test error"));
    }

    #[test]
    fn test_codegen_error_without_span() {
        let err = CodegenError::without_span("test error");
        assert!(err.span.is_none());
        let display = format!("{}", err);
        assert_eq!(display, "test error");
    }
}
