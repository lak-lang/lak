//! Module symbol table for tracking public exports.
//!
//! This module provides [`ModuleTable`], which collects and provides access
//! to public symbols exported by imported modules.

use crate::ast::Visibility;
use crate::resolver::ResolvedModule;
use crate::semantic::SemanticError;
use crate::token::Span;

use std::collections::HashMap;

/// Information about a module's public function exports.
#[derive(Debug, Clone)]
pub struct FunctionExport {
    /// The function name.
    name: String,
    /// The return type.
    return_type: String,
    /// The span of the function definition.
    definition_span: Span,
}

impl FunctionExport {
    /// Creates a new FunctionExport.
    fn new(
        name: String,
        return_type: String,
        definition_span: Span,
    ) -> Result<Self, SemanticError> {
        if name.is_empty() {
            return Err(SemanticError::internal_function_export_empty_name(
                definition_span,
            ));
        }
        if return_type.is_empty() {
            return Err(SemanticError::internal_function_export_empty_return_type(
                definition_span,
            ));
        }
        Ok(FunctionExport {
            name,
            return_type,
            definition_span,
        })
    }

    /// Returns the function name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the return type.
    pub fn return_type(&self) -> &str {
        &self.return_type
    }

    /// Returns the span of the function definition.
    pub fn definition_span(&self) -> Span {
        self.definition_span
    }
}

/// Information about a module's public exports.
#[derive(Debug, Clone)]
pub struct ModuleExports {
    /// The module name.
    name: String,
    /// Public functions exported by this module.
    functions: HashMap<String, FunctionExport>,
}

impl ModuleExports {
    /// Creates a new ModuleExports from a resolved module.
    pub fn from_module(module: &ResolvedModule) -> Result<Self, SemanticError> {
        let mut exports = ModuleExports {
            name: module.name().to_string(),
            functions: HashMap::new(),
        };

        // Extract public functions
        for function in &module.program().functions {
            if function.visibility == Visibility::Public {
                let export = FunctionExport::new(
                    function.name.clone(),
                    function.return_type.clone(),
                    function.span,
                )?;
                exports.functions.insert(function.name.clone(), export);
            }
        }

        Ok(exports)
    }

    /// Returns the module name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns a reference to the functions map.
    pub fn functions(&self) -> &HashMap<String, FunctionExport> {
        &self.functions
    }

    /// Looks up a function by name.
    pub fn get_function(&self, name: &str) -> Option<&FunctionExport> {
        self.functions.get(name)
    }
}

/// Table tracking all available modules and their exports.
#[derive(Debug, Default)]
pub struct ModuleTable {
    /// Map from module name (or alias) to its exports.
    modules: HashMap<String, ModuleExports>,
}

impl ModuleTable {
    /// Creates a new, empty module table.
    pub fn new() -> Self {
        ModuleTable {
            modules: HashMap::new(),
        }
    }

    /// Builds the module table from resolved modules and import declarations.
    ///
    /// # Arguments
    ///
    /// * `resolved_modules` - All resolved modules
    /// * `entry_module` - The entry module (contains import declarations and resolved import paths)
    ///
    /// The module table maps module names (or aliases) to their public exports.
    ///
    /// # Errors
    ///
    /// Returns an error if two imports resolve to the same key (module name or alias)
    /// without using distinct aliases.
    pub fn from_resolved_modules(
        resolved_modules: &[ResolvedModule],
        entry_module: &ResolvedModule,
    ) -> Result<Self, SemanticError> {
        let mut table = Self::new();

        // Track which import path first used each key for error reporting
        let mut key_to_import_path: HashMap<String, String> = HashMap::new();

        // Build a map from canonical path to resolved module for quick lookup
        let path_to_module: HashMap<_, _> = resolved_modules
            .iter()
            .map(|m| (m.path().to_path_buf(), m))
            .collect();

        // Use the pre-resolved import paths from the resolver
        let resolved_imports = entry_module.resolved_imports();

        // Process each import in the entry program
        for import in &entry_module.program().imports {
            // Look up the canonical path from the resolver's pre-computed mapping
            let canonical = resolved_imports.get(&import.path).ok_or_else(|| {
                SemanticError::internal_resolved_path_not_found(&import.path, import.span)
            })?;

            let module = path_to_module.get(canonical).ok_or_else(|| {
                SemanticError::internal_resolved_module_not_found(&import.path, import.span)
            })?;

            let exports = ModuleExports::from_module(module)?;

            // Use alias if provided, otherwise use module name
            let key = import
                .alias
                .clone()
                .unwrap_or_else(|| module.name().to_string());

            // Check for duplicate keys
            if let Some(first_path) = key_to_import_path.get(&key) {
                return Err(SemanticError::duplicate_module_import(
                    &key,
                    first_path,
                    &import.path,
                    import.span,
                ));
            }

            key_to_import_path.insert(key.clone(), import.path.clone());
            table.modules.insert(key, exports);
        }

        Ok(table)
    }

    /// Looks up a module by name (or alias).
    pub fn get_module(&self, name: &str) -> Option<&ModuleExports> {
        self.modules.get(name)
    }

    /// Gets the real module name for a given alias (or module name).
    ///
    /// This is used by codegen to generate the correct mangled function name.
    /// For example, if `import "./utils" as u`, calling `get_real_module_name("u")`
    /// returns `Some("utils")`.
    pub fn get_real_module_name(&self, alias_or_name: &str) -> Option<&str> {
        self.modules.get(alias_or_name).map(|m| m.name())
    }

    /// Looks up a function in a specific module.
    pub fn get_module_function(
        &self,
        module_name: &str,
        function_name: &str,
    ) -> Option<&FunctionExport> {
        self.modules
            .get(module_name)
            .and_then(|m| m.get_function(function_name))
    }

    /// Returns true if the table is empty (no modules).
    pub fn is_empty(&self) -> bool {
        self.modules.is_empty()
    }

    /// Returns the number of modules in the table.
    pub fn len(&self) -> usize {
        self.modules.len()
    }
}

#[cfg(test)]
impl ModuleTable {
    /// Inserts a module with the given exports for testing.
    pub fn insert_for_testing(&mut self, name: String, exports: ModuleExports) {
        self.modules.insert(name, exports);
    }
}

#[cfg(test)]
impl ModuleExports {
    /// Creates a ModuleExports with the given name and functions for testing.
    pub fn for_testing(
        name: String,
        functions: Vec<(String, String, Span)>,
    ) -> Result<Self, SemanticError> {
        let mut map = HashMap::new();
        for (fn_name, ret_type, span) in functions {
            let export = FunctionExport::new(fn_name.clone(), ret_type, span)?;
            map.insert(fn_name, export);
        }
        Ok(ModuleExports {
            name,
            functions: map,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{FnDef, ImportDecl, Program, Visibility};
    use crate::token::Span;

    fn dummy_span() -> Span {
        Span::new(0, 0, 1, 1)
    }

    #[test]
    fn test_function_export_creation() {
        let result = FunctionExport::new("greet".to_string(), "void".to_string(), dummy_span());
        assert!(result.is_ok());
        let export = result.unwrap();
        assert_eq!(export.name(), "greet");
        assert_eq!(export.return_type(), "void");
    }

    #[test]
    fn test_function_export_empty_name_fails() {
        let result = FunctionExport::new("".to_string(), "void".to_string(), dummy_span());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(
            err.kind(),
            crate::semantic::SemanticErrorKind::InternalError
        );
        assert_eq!(
            err.message(),
            "Internal error: FunctionExport name must not be empty. This is a compiler bug."
        );
    }

    #[test]
    fn test_function_export_empty_return_type_fails() {
        let result = FunctionExport::new("greet".to_string(), "".to_string(), dummy_span());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(
            err.kind(),
            crate::semantic::SemanticErrorKind::InternalError
        );
        assert_eq!(
            err.message(),
            "Internal error: FunctionExport return type must not be empty. This is a compiler bug."
        );
    }

    #[test]
    fn test_module_exports_filters_private_functions() {
        // Create a module with one public and one private function
        let public_fn = FnDef {
            visibility: Visibility::Public,
            name: "greet".to_string(),
            return_type: "void".to_string(),
            return_type_span: dummy_span(),
            body: Vec::new(),
            span: dummy_span(),
        };
        let private_fn = FnDef {
            visibility: Visibility::Private,
            name: "helper".to_string(),
            return_type: "void".to_string(),
            return_type_span: dummy_span(),
            body: Vec::new(),
            span: dummy_span(),
        };

        let program = Program {
            imports: Vec::new(),
            functions: vec![public_fn, private_fn],
        };

        // Create a minimal ResolvedModule using a temporary file
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join("test_module.lak");
        let module = crate::resolver::ResolvedModule::for_testing(
            temp_path,
            "test_module".to_string(),
            program,
            "".to_string(),
        );

        let exports = ModuleExports::from_module(&module).unwrap();
        assert_eq!(exports.functions().len(), 1);
        assert!(exports.get_function("greet").is_some());
        assert!(exports.get_function("helper").is_none());
    }

    // =========================================================================
    // ModuleTable::from_resolved_modules tests
    // =========================================================================

    #[test]
    fn test_from_resolved_modules_missing_resolved_import() {
        // When entry module has imports but for_testing creates empty resolved_imports,
        // from_resolved_modules should return an internal error
        let imported_program = Program {
            imports: Vec::new(),
            functions: vec![FnDef {
                visibility: Visibility::Public,
                name: "greet".to_string(),
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: Vec::new(),
                span: dummy_span(),
            }],
        };

        let temp_dir = std::env::temp_dir();
        let imported_module = crate::resolver::ResolvedModule::for_testing(
            temp_dir.join("utils.lak"),
            "utils".to_string(),
            imported_program,
            "".to_string(),
        );

        let entry_program = Program {
            imports: vec![ImportDecl {
                path: "./utils".to_string(),
                alias: None,
                span: dummy_span(),
            }],
            functions: vec![FnDef {
                visibility: Visibility::Private,
                name: "main".to_string(),
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: Vec::new(),
                span: dummy_span(),
            }],
        };

        let entry_module = crate::resolver::ResolvedModule::for_testing(
            temp_dir.join("main.lak"),
            "main".to_string(),
            entry_program,
            "".to_string(),
        );

        let modules = [imported_module, entry_module];
        let result = ModuleTable::from_resolved_modules(&modules, &modules[1]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(
            err.kind(),
            crate::semantic::SemanticErrorKind::InternalError
        );
        assert_eq!(
            err.message(),
            "Internal error: resolved path for import './utils' not found. This is a compiler bug."
        );
    }

    #[test]
    fn test_from_resolved_modules_missing_module_for_path() {
        let temp_dir = std::env::temp_dir();

        let entry_program = Program {
            imports: vec![ImportDecl {
                path: "./utils".to_string(),
                alias: None,
                span: dummy_span(),
            }],
            functions: vec![FnDef {
                visibility: Visibility::Private,
                name: "main".to_string(),
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: Vec::new(),
                span: dummy_span(),
            }],
        };

        let mut entry_module = crate::resolver::ResolvedModule::for_testing(
            temp_dir.join("main.lak"),
            "main".to_string(),
            entry_program,
            "".to_string(),
        );
        entry_module
            .add_resolved_import_for_testing("./utils".to_string(), temp_dir.join("utils.lak"));

        let modules = [entry_module];
        let result = ModuleTable::from_resolved_modules(&modules, &modules[0]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(
            err.kind(),
            crate::semantic::SemanticErrorKind::InternalError
        );
        assert_eq!(
            err.message(),
            "Internal error: resolved module for './utils' not found in resolution map. This is a compiler bug."
        );
    }

    #[test]
    fn test_module_table_basic_operations() {
        // Test the ModuleTable API directly
        let table = ModuleTable::new();
        assert!(table.is_empty());
        assert_eq!(table.len(), 0);
        assert!(table.get_module("utils").is_none());
    }

    #[test]
    fn test_module_table_get_real_module_name() {
        // Test that get_real_module_name returns the correct name
        let program = Program {
            imports: Vec::new(),
            functions: vec![FnDef {
                visibility: Visibility::Public,
                name: "greet".to_string(),
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: Vec::new(),
                span: dummy_span(),
            }],
        };

        let temp_dir = std::env::temp_dir();
        let module = crate::resolver::ResolvedModule::for_testing(
            temp_dir.join("mymod.lak"),
            "mymod".to_string(),
            program,
            "".to_string(),
        );

        let exports = ModuleExports::from_module(&module).unwrap();
        assert_eq!(exports.name(), "mymod");
        assert!(exports.get_function("greet").is_some());
        assert!(exports.get_function("nonexistent").is_none());
    }

    #[test]
    fn test_module_table_get_module_function() {
        // Directly test get_module_function via ModuleTable
        let table = ModuleTable::new();
        assert!(table.get_module_function("utils", "greet").is_none());
    }

    #[test]
    fn test_from_resolved_modules_success() {
        let temp_dir = std::env::temp_dir();

        // Create an imported module with a public function
        let imported_program = Program {
            imports: Vec::new(),
            functions: vec![FnDef {
                visibility: Visibility::Public,
                name: "greet".to_string(),
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: Vec::new(),
                span: dummy_span(),
            }],
        };
        let imported_path = temp_dir.join("utils.lak");
        let imported_module = crate::resolver::ResolvedModule::for_testing(
            imported_path.clone(),
            "utils".to_string(),
            imported_program,
            "".to_string(),
        );

        // Create entry module with import
        let entry_program = Program {
            imports: vec![ImportDecl {
                path: "./utils".to_string(),
                alias: None,
                span: dummy_span(),
            }],
            functions: vec![FnDef {
                visibility: Visibility::Private,
                name: "main".to_string(),
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: Vec::new(),
                span: dummy_span(),
            }],
        };
        let mut entry_module = crate::resolver::ResolvedModule::for_testing(
            temp_dir.join("main.lak"),
            "main".to_string(),
            entry_program,
            "".to_string(),
        );
        entry_module.add_resolved_import_for_testing("./utils".to_string(), imported_path);

        let modules = [imported_module, entry_module];
        let result = ModuleTable::from_resolved_modules(&modules, &modules[1]);
        assert!(result.is_ok());
        let table = result.unwrap();
        assert_eq!(table.len(), 1);
        assert!(table.get_module("utils").is_some());
        assert!(table.get_module_function("utils", "greet").is_some());
        assert!(table.get_module_function("utils", "helper").is_none());
    }

    #[test]
    fn test_from_resolved_modules_duplicate_key() {
        let temp_dir = std::env::temp_dir();

        // Create two modules that would both resolve to "utils" name
        let utils_program = Program {
            imports: Vec::new(),
            functions: vec![FnDef {
                visibility: Visibility::Public,
                name: "greet".to_string(),
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: Vec::new(),
                span: dummy_span(),
            }],
        };

        let utils_path1 = temp_dir.join("utils.lak");
        let utils_path2 = temp_dir.join("lib").join("utils.lak");

        let utils_program2 = Program {
            imports: Vec::new(),
            functions: vec![FnDef {
                visibility: Visibility::Public,
                name: "greet".to_string(),
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: Vec::new(),
                span: dummy_span(),
            }],
        };

        let utils_module1 = crate::resolver::ResolvedModule::for_testing(
            utils_path1.clone(),
            "utils".to_string(),
            utils_program,
            "".to_string(),
        );
        let utils_module2 = crate::resolver::ResolvedModule::for_testing(
            utils_path2.clone(),
            "utils".to_string(),
            utils_program2,
            "".to_string(),
        );

        // Entry module imports both, which creates duplicate "utils" key
        let entry_program = Program {
            imports: vec![
                ImportDecl {
                    path: "./utils".to_string(),
                    alias: None,
                    span: dummy_span(),
                },
                ImportDecl {
                    path: "./lib/utils".to_string(),
                    alias: None,
                    span: dummy_span(),
                },
            ],
            functions: vec![FnDef {
                visibility: Visibility::Private,
                name: "main".to_string(),
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: Vec::new(),
                span: dummy_span(),
            }],
        };
        let mut entry_module = crate::resolver::ResolvedModule::for_testing(
            temp_dir.join("main.lak"),
            "main".to_string(),
            entry_program,
            "".to_string(),
        );
        entry_module.add_resolved_import_for_testing("./utils".to_string(), utils_path1);
        entry_module.add_resolved_import_for_testing("./lib/utils".to_string(), utils_path2);

        let modules = [utils_module1, utils_module2, entry_module];
        let result = ModuleTable::from_resolved_modules(&modules, &modules[2]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(
            err.kind(),
            crate::semantic::SemanticErrorKind::DuplicateModuleImport
        );
        assert_eq!(
            err.message(),
            "Module name 'utils' is already imported from './utils'"
        );
    }

    #[test]
    fn test_from_resolved_modules_with_alias() {
        let temp_dir = std::env::temp_dir();

        // Create an imported module with a public function
        let imported_program = Program {
            imports: Vec::new(),
            functions: vec![FnDef {
                visibility: Visibility::Public,
                name: "greet".to_string(),
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: Vec::new(),
                span: dummy_span(),
            }],
        };
        let imported_path = temp_dir.join("utils.lak");
        let imported_module = crate::resolver::ResolvedModule::for_testing(
            imported_path.clone(),
            "utils".to_string(),
            imported_program,
            "".to_string(),
        );

        // Create entry module with aliased import: import "./utils" as u
        let entry_program = Program {
            imports: vec![ImportDecl {
                path: "./utils".to_string(),
                alias: Some("u".to_string()),
                span: dummy_span(),
            }],
            functions: vec![FnDef {
                visibility: Visibility::Private,
                name: "main".to_string(),
                return_type: "void".to_string(),
                return_type_span: dummy_span(),
                body: Vec::new(),
                span: dummy_span(),
            }],
        };
        let mut entry_module = crate::resolver::ResolvedModule::for_testing(
            temp_dir.join("main.lak"),
            "main".to_string(),
            entry_program,
            "".to_string(),
        );
        entry_module.add_resolved_import_for_testing("./utils".to_string(), imported_path);

        let modules = [imported_module, entry_module];
        let result = ModuleTable::from_resolved_modules(&modules, &modules[1]);
        assert!(result.is_ok());
        let table = result.unwrap();
        assert_eq!(table.len(), 1);
        // Module is accessible by alias "u", not by real name "utils"
        assert!(table.get_module("u").is_some());
        assert!(table.get_module("utils").is_none());
        // Function lookup via alias works
        assert!(table.get_module_function("u", "greet").is_some());
        // get_real_module_name resolves alias to real name
        assert_eq!(table.get_real_module_name("u"), Some("utils"));
        assert_eq!(table.get_real_module_name("utils"), None);
    }

    #[test]
    fn test_get_real_module_name_with_populated_table() {
        let mut table = ModuleTable::new();
        let exports = ModuleExports::for_testing(
            "utils".to_string(),
            vec![("greet".to_string(), "void".to_string(), dummy_span())],
        )
        .unwrap();
        // Insert with alias key "u" but real name is "utils"
        table.insert_for_testing("u".to_string(), exports);
        assert_eq!(table.get_real_module_name("u"), Some("utils"));
        assert_eq!(table.get_real_module_name("utils"), None);
        assert_eq!(table.get_real_module_name("nonexistent"), None);
    }
}
