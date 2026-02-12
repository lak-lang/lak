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
//!         params: vec![],
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
use inkwell::basic_block::BasicBlock;
use inkwell::context::Context;
use inkwell::types::{BasicMetadataTypeEnum, BasicTypeEnum};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Mangle prefix used for single-file compilation.
const SINGLE_FILE_MANGLE_PREFIX: &str = "entry";

/// Control-flow targets for the current loop context.
struct LoopControl<'ctx> {
    continue_block: BasicBlock<'ctx>,
    break_block: BasicBlock<'ctx>,
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
    /// Stack of variable scopes (innermost scope is at the end).
    ///
    /// This is reset at the start of each function body and extended for
    /// block statements (e.g., `if` branches) to support shadowing.
    variables: Vec<HashMap<String, VarBinding<'ctx>>>,
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
    /// When generating code for any module's functions, this is set to that
    /// module's mangle prefix so that intra-module function calls can be
    /// resolved to their mangled names.
    ///
    /// For example, if the current module is at `lib/utils.lak` relative to
    /// the entry directory, this would be `Some("lib__utils")`, and a call
    /// to function `helper()` within the same module would resolve to `_L10_lib__utils_helper`.
    ///
    /// `None` when no module is currently being generated.
    current_module_prefix: Option<String>,
    /// Function parameter types keyed by LLVM function name.
    ///
    /// Used to type-check call arguments during code generation.
    function_param_types: HashMap<String, Vec<Type>>,
    /// Function return types keyed by LLVM function name.
    ///
    /// `None` represents `void` return type.
    function_return_types: HashMap<String, Option<Type>>,
    /// Stack of loop control-flow targets (innermost loop at the end).
    loop_controls: Vec<LoopControl<'ctx>>,
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

/// Derives a mangle prefix from a module path.
///
/// If the module path is under `entry_dir`, the prefix is computed from the
/// relative path (without extension). Otherwise, the full canonical path
/// (without extension) is used.
fn derive_mangle_prefix(module_path: &Path, entry_dir: &Path) -> Result<String, CodegenError> {
    let prefix = if let Ok(relative) = module_path.strip_prefix(entry_dir) {
        let without_ext = relative.with_extension("");
        path_components_to_strings(&without_ext, module_path)?.join("__")
    } else {
        let without_ext = module_path.with_extension("");
        path_components_to_strings(&without_ext, module_path)?.join("__")
    };

    if prefix.is_empty() {
        return Err(CodegenError::internal_empty_mangle_prefix(module_path));
    }

    Ok(prefix)
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

        let prefix = derive_mangle_prefix(module.path(), entry_dir)?;

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

/// Computes a unique mangle prefix for the entry module.
///
/// The base prefix is derived from the entry module path. If it collides with
/// an imported module prefix, `__entry`, `__entry2`, ... suffixes are added
/// until the prefix becomes unique.
fn compute_entry_mangle_prefix(
    entry_path: &Path,
    imported_prefixes: &HashMap<PathBuf, String>,
) -> Result<String, CodegenError> {
    let entry_dir = entry_path
        .parent()
        .ok_or_else(|| CodegenError::internal_entry_path_no_parent(entry_path))?;
    let base = derive_mangle_prefix(entry_path, entry_dir)?;

    let used: HashSet<&str> = imported_prefixes.values().map(String::as_str).collect();
    if !used.contains(base.as_str()) {
        return Ok(base);
    }

    let mut counter = 1usize;
    loop {
        let candidate = if counter == 1 {
            format!("{base}__entry")
        } else {
            format!("{base}__entry{counter}")
        };
        if !used.contains(candidate.as_str()) {
            return Ok(candidate);
        }
        counter += 1;
    }
}

/// Looks up the mangle prefix for a module path.
///
/// Returns an internal error if the prefix is not found, which indicates
/// a compiler bug since all non-entry modules should have been registered
/// by `compute_mangle_prefixes`. The entry module prefix is handled
/// separately by `compute_entry_mangle_prefix`.
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
            variables: Vec::new(),
            module_aliases: HashMap::new(),
            current_module_prefix: None,
            function_param_types: HashMap::new(),
            function_return_types: HashMap::new(),
            loop_controls: Vec::new(),
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
        self.declare_lak_streq();
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
        self.current_module_prefix = Some(SINGLE_FILE_MANGLE_PREFIX.to_string());
        self.function_param_types.clear();
        self.function_return_types.clear();

        let result = (|| -> Result<(), CodegenError> {
            // Pass 1: Declare all user-defined functions (except main, which has a special signature)
            for function in &program.functions {
                if function.name != "main" {
                    let llvm_name = mangle_name(SINGLE_FILE_MANGLE_PREFIX, &function.name);
                    let param_types = function
                        .params
                        .iter()
                        .map(|param| param.ty.clone())
                        .collect::<Vec<_>>();
                    self.declare_function(
                        &llvm_name,
                        &param_types,
                        &function.return_type,
                        function.return_type_span,
                    )?;
                }
            }

            // Pass 2: Generate function bodies
            for function in &program.functions {
                if function.name == "main" {
                    self.generate_main(function)?;
                } else {
                    let llvm_name = mangle_name(SINGLE_FILE_MANGLE_PREFIX, &function.name);
                    self.generate_function_body(&llvm_name, function)?;
                }
            }

            Ok(())
        })();

        self.current_module_prefix = None;
        result
    }

    /// Compiles multiple modules into a single LLVM module.
    ///
    /// This is used when the entry module imports other modules.
    /// All functions from all modules are compiled into a single LLVM module.
    ///
    /// Name mangling: All user-defined functions except the entry `main`
    /// use the pattern `_L{prefix_len}_{mangle_prefix}_{function_name}`
    /// to avoid name collisions.
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
        self.function_param_types.clear();
        self.function_return_types.clear();

        // Validate that entry module exists in the module list
        if !modules.iter().any(|m| m.path() == entry_path) {
            return Err(CodegenError::internal_entry_module_not_found(entry_path));
        }

        let imported_prefixes = compute_mangle_prefixes(modules, entry_path)?;
        let entry_prefix = compute_entry_mangle_prefix(entry_path, &imported_prefixes)?;

        let result = (|| -> Result<(), CodegenError> {
            // Pass 1: Declare all user-defined functions from all modules
            for module in modules {
                let is_entry = module.path() == entry_path;
                let module_prefix = if is_entry {
                    entry_prefix.as_str()
                } else {
                    get_mangle_prefix(&imported_prefixes, module.path())?
                };

                for function in &module.program().functions {
                    if is_entry && function.name == "main" {
                        // Skip main from entry module - it has special signature
                        continue;
                    }

                    let mangled_name = mangle_name(module_prefix, &function.name);
                    let param_types = function
                        .params
                        .iter()
                        .map(|param| param.ty.clone())
                        .collect::<Vec<_>>();
                    self.declare_function(
                        &mangled_name,
                        &param_types,
                        &function.return_type,
                        function.return_type_span,
                    )?;
                }
            }

            // Pass 2: Generate function bodies for all modules
            for module in modules {
                // Set up this module's alias map for resolving ModuleCall expressions
                self.module_aliases.clear();
                for import in &module.program().imports {
                    let canonical_path =
                        module.resolved_imports().get(&import.path).ok_or_else(|| {
                            CodegenError::internal_import_path_not_resolved(
                                &import.path,
                                import.span,
                            )
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
                    let mangle_prefix =
                        get_mangle_prefix(&imported_prefixes, canonical_path.as_path())?;
                    let key = import
                        .alias
                        .clone()
                        .unwrap_or_else(|| imported_module.name().to_string());
                    self.module_aliases.insert(key, mangle_prefix.to_string());
                }

                // Set module prefix for name mangling of intra-module calls
                let is_entry = module.path() == entry_path;
                let module_prefix = if is_entry {
                    entry_prefix.as_str()
                } else {
                    get_mangle_prefix(&imported_prefixes, module.path())?
                };
                self.current_module_prefix = Some(module_prefix.to_string());

                for function in &module.program().functions {
                    if is_entry && function.name == "main" {
                        self.generate_main(function)?;
                    } else {
                        let llvm_name = mangle_name(module_prefix, &function.name);
                        self.generate_function_body(&llvm_name, function)?;
                    }
                }
            }

            Ok(())
        })();

        // Reset module prefix after compilation
        self.current_module_prefix = None;

        result
    }

    /// Declares a user-defined function (creates LLVM function signature only).
    ///
    /// This method is called in Pass 1 to create function declarations before
    /// any function bodies are generated. This allows forward references.
    fn declare_function(
        &mut self,
        name: &str,
        param_types: &[Type],
        return_type: &str,
        return_type_span: crate::token::Span,
    ) -> Result<(), CodegenError> {
        let llvm_param_types: Vec<BasicMetadataTypeEnum<'ctx>> = param_types
            .iter()
            .map(|ty| self.get_llvm_type(ty).into())
            .collect();
        let parsed_return_type = self.parse_return_type(return_type, return_type_span)?;
        let fn_type = match &parsed_return_type {
            None => self.context.void_type().fn_type(&llvm_param_types, false),
            Some(Type::I32) => self.context.i32_type().fn_type(&llvm_param_types, false),
            Some(Type::I64) => self.context.i64_type().fn_type(&llvm_param_types, false),
            Some(Type::String) => self
                .context
                .ptr_type(AddressSpace::default())
                .fn_type(&llvm_param_types, false),
            Some(Type::Bool) => self.context.bool_type().fn_type(&llvm_param_types, false),
        };
        self.module.add_function(name, fn_type, None);
        self.function_param_types
            .insert(name.to_string(), param_types.to_vec());
        self.function_return_types
            .insert(name.to_string(), parsed_return_type);
        Ok(())
    }

    fn parse_return_type(
        &self,
        return_type: &str,
        span: crate::token::Span,
    ) -> Result<Option<Type>, CodegenError> {
        match return_type {
            "void" => Ok(None),
            "i32" => Ok(Some(Type::I32)),
            "i64" => Ok(Some(Type::I64)),
            "string" => Ok(Some(Type::String)),
            "bool" => Ok(Some(Type::Bool)),
            _ => Err(CodegenError::internal_unsupported_function_return_type(
                return_type,
                span,
            )),
        }
    }

    /// Generates the body of a user-defined function.
    ///
    /// Creates the function body with an entry block and generates statements
    /// until a terminator is emitted. A trailing return is synthesized only for
    /// `void` functions; non-void functions must already terminate explicitly.
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
        self.loop_controls.clear();
        self.enter_variable_scope();

        let function = self
            .module
            .get_function(llvm_name)
            .ok_or_else(|| CodegenError::internal_function_not_found_no_span(llvm_name))?;

        let entry = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry);

        if function.count_params() as usize != fn_def.params.len() {
            return Err(CodegenError::internal_function_param_count_mismatch(
                llvm_name,
                fn_def.params.len(),
                function.count_params() as usize,
            ));
        }

        for (idx, param) in fn_def.params.iter().enumerate() {
            let llvm_param = function.get_nth_param(idx as u32).ok_or_else(|| {
                CodegenError::internal_function_param_missing(llvm_name, idx, param.span)
            })?;
            let binding = VarBinding::new(
                &self.builder,
                self.context,
                &param.ty,
                &param.name,
                param.span,
            )?;
            self.builder
                .build_store(binding.alloca(), llvm_param)
                .map_err(|e| {
                    CodegenError::internal_variable_store_failed(
                        &param.name,
                        &e.to_string(),
                        param.span,
                    )
                })?;
            self.define_variable_in_current_scope(&param.name, binding, param.span)?;
        }

        for stmt in &fn_def.body {
            let has_terminator = self
                .builder
                .get_insert_block()
                .and_then(|bb| bb.get_terminator())
                .is_some();
            if has_terminator {
                break;
            }
            self.generate_stmt(stmt)?;
        }

        let has_terminator = self
            .builder
            .get_insert_block()
            .and_then(|bb| bb.get_terminator())
            .is_some();
        if !has_terminator {
            if fn_def.return_type == "void" {
                self.builder.build_return(None).map_err(|e| {
                    CodegenError::internal_return_build_failed(llvm_name, &e.to_string())
                })?;
            } else {
                return Err(CodegenError::internal_missing_return_in_non_void_function(
                    llvm_name,
                    &fn_def.return_type,
                ));
            }
        }

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
        self.loop_controls.clear();
        self.enter_variable_scope();

        let i32_type = self.context.i32_type();
        let main_type = i32_type.fn_type(&[], false);
        let main_fn = self.module.add_function("main", main_type, None);

        let entry = self.context.append_basic_block(main_fn, "entry");
        self.builder.position_at_end(entry);

        for stmt in &main_fn_def.body {
            let has_terminator = self
                .builder
                .get_insert_block()
                .and_then(|bb| bb.get_terminator())
                .is_some();
            if has_terminator {
                break;
            }
            self.generate_stmt(stmt)?;
        }

        let has_terminator = self
            .builder
            .get_insert_block()
            .and_then(|bb| bb.get_terminator())
            .is_some();
        if !has_terminator {
            let zero = i32_type.const_int(0, false);
            self.builder
                .build_return(Some(&zero))
                .map_err(|e| CodegenError::internal_main_return_build_failed(&e.to_string()))?;
        }

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

    fn enter_variable_scope(&mut self) {
        self.variables.push(HashMap::new());
    }

    fn exit_variable_scope(&mut self, span: crate::token::Span) -> Result<(), CodegenError> {
        self.variables
            .pop()
            .map(|_| ())
            .ok_or_else(|| CodegenError::internal_no_variable_scope(span))
    }

    fn variable_in_current_scope(&self, name: &str) -> bool {
        self.variables
            .last()
            .is_some_and(|scope| scope.contains_key(name))
    }

    fn define_variable_in_current_scope(
        &mut self,
        name: &str,
        binding: VarBinding<'ctx>,
        span: crate::token::Span,
    ) -> Result<(), CodegenError> {
        let scope = self
            .variables
            .last_mut()
            .ok_or_else(|| CodegenError::internal_no_variable_scope(span))?;
        scope.insert(name.to_string(), binding);
        Ok(())
    }

    fn lookup_variable(&self, name: &str) -> Option<&VarBinding<'ctx>> {
        self.variables
            .iter()
            .rev()
            .find_map(|scope| scope.get(name))
    }

    fn push_loop_control(
        &mut self,
        continue_block: BasicBlock<'ctx>,
        break_block: BasicBlock<'ctx>,
    ) {
        self.loop_controls.push(LoopControl {
            continue_block,
            break_block,
        });
    }

    fn pop_loop_control(&mut self, span: crate::token::Span) -> Result<(), CodegenError> {
        self.loop_controls
            .pop()
            .map(|_| ())
            .ok_or_else(|| CodegenError::internal_no_loop_control_scope(span))
    }

    fn current_loop_control(&self) -> Option<&LoopControl<'ctx>> {
        self.loop_controls.last()
    }
}
