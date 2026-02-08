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
//! - Compiles function calls (`println`, `panic`, user-defined functions, module-qualified calls)
//! - Computes path-based mangle prefixes for multi-module compilation
//! - Handles variable declarations (`let` statements) with stack allocation
//! - Writes the output to a native object file
//!
//! # Architecture
//!
//! The generated code follows the C calling convention and links against
//! the Lak runtime library for I/O and panic operations.
//!
//! # Example
//!
//! ```no_run
//! use inkwell::context::Context;
//! use lak::codegen::Codegen;
//! use lak::ast::{Program, FnDef, Stmt, StmtKind, Expr, ExprKind, Visibility};
//! use lak::token::Span;
//! use std::path::Path;
//!
//! let context = Context::create();
//! let mut codegen = Codegen::new(&context, "example");
//!
//! let program = Program {
//!     imports: vec![],
//!     functions: vec![FnDef {
//!         visibility: Visibility::Private,
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
use crate::resolver::ResolvedModule;
use binding::VarBinding;
use inkwell::AddressSpace;
use inkwell::context::Context;
use inkwell::types::BasicTypeEnum;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

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
    /// This table is cleared at the start of each function body to implement
    /// function-scoped variables.
    variables: HashMap<String, VarBinding<'ctx>>,
    /// Mapping from module alias to its mangle prefix.
    ///
    /// When an import has no alias (e.g., `import "./utils"`), the key is the
    /// imported module's filename stem (e.g., `"utils"`). When an alias is
    /// provided (e.g., `import "./utils" as u`), the key is the alias (e.g., `"u"`).
    ///
    /// The mangle prefix is derived from the module's path relative to the
    /// entry directory, ensuring unique mangled names even for modules with
    /// the same filename in different directories.
    module_aliases: HashMap<String, String>,
    /// The current module's mangle prefix for name mangling.
    ///
    /// When generating code for an imported module's functions, this is set
    /// to the module's mangle prefix (derived from its path relative to the
    /// entry directory) so that intra-module function calls can be resolved
    /// to their mangled names.
    ///
    /// For example, if the current module is at `lib/utils.lak` relative to
    /// the entry directory, this would be `Some("lib__utils")`, and a call
    /// to function `helper()` within the same module would resolve to `_L10_lib__utils_helper`.
    ///
    /// `None` when generating the entry module (its functions are not mangled).
    current_module_prefix: Option<String>,
}

/// Creates a mangled function name using a length-prefix scheme.
///
/// Format: `_L{prefix_len}_{prefix}_{function}`.
/// The prefix length ensures unambiguous parsing, making collisions
/// impossible regardless of prefix or function name content.
///
/// # Examples
///
/// - `("utils", "greet")` → `"_L5_utils_greet"`
/// - `("dir__foo", "bar")` → `"_L8_dir__foo_bar"`
/// - `("a__b", "c")` → `"_L4_a__b_c"`
fn mangle_name(prefix: &str, function: &str) -> String {
    format!("_L{}_{}_{}", prefix.len(), prefix, function)
}

/// Extracts [`Normal`](std::path::Component::Normal) path components as UTF-8 strings, rejecting non-canonical paths.
///
/// Only [`std::path::Component::Normal`] components are included in the result.
/// Root (`/`) and prefix (`C:\`) components are silently excluded since they
/// are not meaningful for mangle prefixes. `.` and `..` components cause an
/// error because paths must be canonical before reaching code generation.
///
/// # Errors
///
/// Returns an error if:
/// - Any Normal component contains non-UTF-8 data
/// - The path contains `.` or `..` components (paths must be canonical)
fn path_components_to_strings<'a>(
    path: &'a Path,
    original_module_path: &Path,
) -> Result<Vec<&'a str>, CodegenError> {
    use std::path::Component;
    path.components()
        .filter_map(|c| match c {
            Component::Normal(os_str) => Some(
                os_str
                    .to_str()
                    .ok_or_else(|| CodegenError::non_utf8_path_component(original_module_path)),
            ),
            Component::CurDir | Component::ParentDir => Some(Err(
                CodegenError::internal_non_canonical_path(original_module_path),
            )),
            // Root (`/`) and Prefix (`C:\`) are not meaningful for mangle prefixes
            _ => None,
        })
        .collect()
}

/// Computes unique mangle prefixes for imported modules.
///
/// The prefix is derived from the module's path relative to the entry
/// module's directory, with path separators replaced by `__`.
/// This ensures modules with the same filename but different directories
/// get distinct mangled names.
///
/// If the module path is not under the entry directory (e.g., resolved
/// via parent-relative imports), all Normal path components (excluding
/// `/`) from the full path are used as a fallback prefix.
///
/// # Examples
///
/// Given entry path `/project/main.lak`:
/// - `/project/foo.lak` → prefix `"foo"`
/// - `/project/dir/foo.lak` → prefix `"dir__foo"`
/// - `/project/a/b/c/mod.lak` → prefix `"a__b__c__mod"`
///
/// Fallback (outside entry directory):
/// - `/opt/lib/utils.lak` → prefix `"opt__lib__utils"`
///
/// # Errors
///
/// Returns an error if:
/// - A module path contains non-UTF-8 components
/// - A module path contains non-canonical components (`.` or `..`)
/// - The entry path has no parent directory
/// - A module path produces an empty mangle prefix
/// - Two different modules produce the same mangle prefix
fn compute_mangle_prefixes(
    modules: &[ResolvedModule],
    entry_path: &Path,
) -> Result<HashMap<PathBuf, String>, CodegenError> {
    let entry_dir = entry_path
        .parent()
        .ok_or_else(|| CodegenError::internal_entry_path_no_parent(entry_path))?;
    let mut prefixes = HashMap::new();
    let mut prefix_to_path: HashMap<String, PathBuf> = HashMap::new();

    for module in modules {
        if module.path() == entry_path {
            continue;
        }

        let prefix = if let Ok(relative) = module.path().strip_prefix(entry_dir) {
            let without_ext = relative.with_extension("");
            path_components_to_strings(&without_ext, module.path())?.join("__")
        } else {
            let without_ext = module.path().with_extension("");
            path_components_to_strings(&without_ext, module.path())?.join("__")
        };

        if prefix.is_empty() {
            return Err(CodegenError::internal_empty_mangle_prefix(module.path()));
        }

        if let Some(existing_path) = prefix_to_path.get(&prefix) {
            return Err(CodegenError::duplicate_mangle_prefix(
                &prefix,
                existing_path,
                module.path(),
            ));
        }

        prefix_to_path.insert(prefix.clone(), module.path().to_path_buf());
        prefixes.insert(module.path().to_path_buf(), prefix);
    }

    Ok(prefixes)
}

/// Looks up the mangle prefix for a module path.
///
/// Returns an internal error if the prefix is not found, which indicates
/// a compiler bug since all non-entry modules should have been registered
/// by `compute_mangle_prefixes`. Entry module functions use unmangled
/// names and are not registered in the prefix map.
fn get_mangle_prefix<'a>(
    prefixes: &'a HashMap<PathBuf, String>,
    module_path: &Path,
) -> Result<&'a str, CodegenError> {
    prefixes
        .get(module_path)
        .map(String::as_str)
        .ok_or_else(|| CodegenError::internal_mangle_prefix_not_found(module_path))
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
            module_aliases: HashMap::new(),
            current_module_prefix: None,
        }
    }

    /// Declares all built-in functions used by the runtime.
    ///
    /// When adding a new builtin here, also update `BUILTIN_NAMES` in `builtins.rs`
    /// and the sync test `test_builtin_names_matches_declare_builtins` in `tests.rs`.
    fn declare_builtins(&mut self) {
        self.declare_lak_println();
        self.declare_lak_println_i32();
        self.declare_lak_println_i64();
        self.declare_lak_println_bool();
        self.declare_lak_panic();
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
        self.declare_builtins();

        // Pass 1: Declare all user-defined functions (except main, which has a special signature)
        for function in &program.functions {
            if function.name != "main" {
                self.declare_function(&function.name)?;
            }
        }

        // Pass 2: Generate function bodies
        for function in &program.functions {
            if function.name == "main" {
                self.generate_main(function)?;
            } else {
                self.generate_function_body(&function.name, function)?;
            }
        }

        Ok(())
    }

    /// Compiles multiple modules into a single LLVM module.
    ///
    /// This is used when the entry module imports other modules.
    /// All functions from all modules are compiled into a single LLVM module.
    ///
    /// Name mangling: Functions from imported modules use the pattern
    /// `_L{prefix_len}_{mangle_prefix}_{function_name}` to avoid name collisions.
    ///
    /// The mangle prefix is derived from the module's path relative to the entry
    /// directory (see `compute_mangle_prefixes`).
    ///
    /// # Arguments
    ///
    /// * `modules` - All resolved modules
    /// * `entry_path` - The canonical path of the entry point module
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The entry module is not found in the module list
    /// - A module path contains non-UTF-8 components
    /// - Two modules produce the same mangle prefix
    /// - An import path is not found in the resolved imports
    /// - LLVM IR generation fails for any module
    pub fn compile_modules(
        &mut self,
        modules: &[ResolvedModule],
        entry_path: &Path,
    ) -> Result<(), CodegenError> {
        // Declare built-in functions
        self.declare_builtins();

        // Validate that entry module exists in the module list
        if !modules.iter().any(|m| m.path() == entry_path) {
            return Err(CodegenError::internal_entry_module_not_found(entry_path));
        }

        let mangle_prefixes = compute_mangle_prefixes(modules, entry_path)?;

        // Pass 1: Declare all user-defined functions from all modules
        for module in modules {
            let is_entry = module.path() == entry_path;
            for function in &module.program().functions {
                if is_entry && function.name == "main" {
                    // Skip main from entry module - it has special signature
                    continue;
                }
                // Use mangled name for functions from imported modules
                let mangled_name = if is_entry {
                    function.name.clone()
                } else {
                    let prefix = get_mangle_prefix(&mangle_prefixes, module.path())?;
                    mangle_name(prefix, &function.name)
                };
                self.declare_function(&mangled_name)?;
            }
        }

        // Pass 2: Generate function bodies for all modules
        for module in modules {
            // Set up this module's alias map for resolving ModuleCall expressions
            self.module_aliases.clear();
            for import in &module.program().imports {
                let canonical_path =
                    module.resolved_imports().get(&import.path).ok_or_else(|| {
                        CodegenError::internal_import_path_not_resolved(&import.path, import.span)
                    })?;
                let imported_module = modules
                    .iter()
                    .find(|m| m.path() == canonical_path.as_path())
                    .ok_or_else(|| {
                        CodegenError::internal_resolved_module_not_found_for_path(
                            canonical_path,
                            import.span,
                        )
                    })?;
                let mangle_prefix = get_mangle_prefix(&mangle_prefixes, canonical_path.as_path())?;
                let key = import
                    .alias
                    .clone()
                    .unwrap_or_else(|| imported_module.name().to_string());
                self.module_aliases.insert(key, mangle_prefix.to_string());
            }

            // Set module prefix for name mangling of intra-module calls
            let is_entry = module.path() == entry_path;
            let module_prefix = if is_entry {
                self.current_module_prefix = None;
                None
            } else {
                let prefix = get_mangle_prefix(&mangle_prefixes, module.path())?;
                self.current_module_prefix = Some(prefix.to_string());
                Some(prefix)
            };

            for function in &module.program().functions {
                if is_entry && function.name == "main" {
                    self.generate_main(function)?;
                } else {
                    let llvm_name = if let Some(prefix) = module_prefix {
                        mangle_name(prefix, &function.name)
                    } else {
                        function.name.clone()
                    };
                    self.generate_function_body(&llvm_name, function)?;
                }
            }
        }

        // Reset module prefix after compilation
        self.current_module_prefix = None;

        Ok(())
    }

    /// Declares a user-defined function (creates LLVM function signature only).
    ///
    /// This method is called in Pass 1 to create function declarations before
    /// any function bodies are generated. This allows forward references.
    fn declare_function(&mut self, name: &str) -> Result<(), CodegenError> {
        let void_type = self.context.void_type();
        let fn_type = void_type.fn_type(&[], false);
        self.module.add_function(name, fn_type, None);
        Ok(())
    }

    /// Generates the body of a user-defined function.
    ///
    /// Creates the function body with an entry block, generates all statements,
    /// and adds a void return at the end.
    ///
    /// # Arguments
    ///
    /// * `llvm_name` - The name the function was declared with in LLVM (may be mangled)
    /// * `fn_def` - The function definition from the AST
    fn generate_function_body(
        &mut self,
        llvm_name: &str,
        fn_def: &FnDef,
    ) -> Result<(), CodegenError> {
        self.variables.clear();

        let function = self
            .module
            .get_function(llvm_name)
            .ok_or_else(|| CodegenError::internal_function_not_found_no_span(llvm_name))?;

        let entry = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry);

        for stmt in &fn_def.body {
            self.generate_stmt(stmt)?;
        }

        self.builder
            .build_return(None)
            .map_err(|e| CodegenError::internal_return_build_failed(llvm_name, &e.to_string()))?;

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
    /// - `Type::String` → LLVM `ptr` (opaque pointer)
    /// - `Type::Bool` → LLVM `i1`
    fn get_llvm_type(&self, ty: &Type) -> BasicTypeEnum<'ctx> {
        match ty {
            Type::I32 => self.context.i32_type().into(),
            Type::I64 => self.context.i64_type().into(),
            Type::String => self.context.ptr_type(AddressSpace::default()).into(),
            Type::Bool => self.context.bool_type().into(),
        }
    }

    /// Resolves a module alias to its mangle prefix.
    ///
    /// Returns the mangle prefix for a given alias. If the alias is not
    /// found in the map, returns an internal error since all imports should
    /// have been registered during compilation.
    pub(crate) fn resolve_module_alias(
        &self,
        alias_or_name: &str,
        span: crate::token::Span,
    ) -> Result<String, CodegenError> {
        self.module_aliases
            .get(alias_or_name)
            .cloned()
            .ok_or_else(|| CodegenError::internal_module_alias_not_found(alias_or_name, span))
    }
}
