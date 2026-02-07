//! Module resolution for multi-file Lak programs.
//!
//! This module handles loading and parsing imported modules, building a
//! dependency graph, and detecting circular imports.

mod error;

pub use error::{ResolverError, ResolverErrorKind};

use crate::ast::Program;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::token::Span;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A resolved module with its parsed AST and metadata.
#[derive(Debug)]
pub struct ResolvedModule {
    /// The canonical absolute path to the module file.
    path: PathBuf,
    /// The module name (last segment of path, without extension).
    name: String,
    /// The parsed program AST.
    program: Program,
    /// The source code (needed for error reporting).
    source: String,
    /// Module names that this module imports.
    dependencies: Vec<String>,
    /// Map from import path strings to their resolved canonical paths.
    resolved_imports: HashMap<String, PathBuf>,
}

impl ResolvedModule {
    /// Returns the canonical absolute path to the module file.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the module name (last segment of path, without extension).
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the parsed program AST.
    pub fn program(&self) -> &Program {
        &self.program
    }

    /// Returns the source code (needed for error reporting).
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Returns the module names that this module imports.
    pub fn dependencies(&self) -> &[String] {
        &self.dependencies
    }

    /// Returns the mapping from import path strings to canonical paths.
    pub fn resolved_imports(&self) -> &HashMap<String, PathBuf> {
        &self.resolved_imports
    }

    /// Creates a `ResolvedModule` for testing purposes.
    ///
    /// This constructor allows tests to create `ResolvedModule` instances
    /// without going through the resolver's file-based resolution pipeline.
    #[cfg(test)]
    pub fn for_testing(
        path: std::path::PathBuf,
        name: String,
        program: Program,
        source: String,
    ) -> Self {
        ResolvedModule {
            path,
            name,
            program,
            source,
            dependencies: Vec::new(),
            resolved_imports: HashMap::new(),
        }
    }
}

/// Module resolver that handles import resolution and cycle detection.
pub struct ModuleResolver {
    /// Cache of resolved modules (key: canonical absolute path).
    modules: HashMap<PathBuf, ResolvedModule>,
    /// Current resolution stack for cycle detection.
    resolving: Vec<PathBuf>,
}

impl ModuleResolver {
    /// Creates a new module resolver.
    pub fn new() -> Self {
        ModuleResolver {
            modules: HashMap::new(),
            resolving: Vec::new(),
        }
    }

    /// Resolves a module starting from an entry point file.
    ///
    /// Returns all transitively imported modules (including entry module).
    pub fn resolve_from_entry(&mut self, entry_path: &Path) -> Result<(), ResolverError> {
        let canonical = entry_path
            .canonicalize()
            .map_err(|e| ResolverError::io_error(&e.to_string()))?;

        self.resolve_module(&canonical, None, None)
    }

    /// Resolves a module starting from an entry point file with pre-read source.
    ///
    /// This avoids reading the entry file a second time when the caller has
    /// already read the source.
    pub fn resolve_from_entry_with_source(
        &mut self,
        entry_path: &Path,
        source: String,
    ) -> Result<(), ResolverError> {
        let canonical = entry_path
            .canonicalize()
            .map_err(|e| ResolverError::io_error(&e.to_string()))?;

        self.resolve_module(&canonical, None, Some(source))
    }

    /// Resolves a single module and its dependencies recursively.
    fn resolve_module(
        &mut self,
        path: &Path,
        import_span: Option<Span>,
        pre_read_source: Option<String>,
    ) -> Result<(), ResolverError> {
        // Check for circular dependency
        if self.resolving.iter().any(|p| p == path) {
            let cycle = self.format_cycle(path);
            return match import_span {
                Some(span) => Err(ResolverError::circular_import(&cycle, span)),
                None => Err(ResolverError::new(
                    ResolverErrorKind::CircularImport,
                    format!("Circular import detected: {}", cycle),
                )),
            };
        }

        // Check cache
        if self.modules.contains_key(path) {
            return Ok(());
        }

        // Add to resolution stack; always cleaned up via pop below
        self.resolving.push(path.to_path_buf());
        let result = self.resolve_module_inner(path, import_span, pre_read_source);
        self.resolving.pop();
        result
    }

    /// Inner implementation of module resolution (called by `resolve_module`).
    ///
    /// The resolving stack push/pop is handled by the caller to ensure
    /// cleanup even when this method returns an error.
    fn resolve_module_inner(
        &mut self,
        path: &Path,
        import_span: Option<Span>,
        pre_read_source: Option<String>,
    ) -> Result<(), ResolverError> {
        // Load and parse file
        let source = match pre_read_source {
            Some(s) => s,
            None => std::fs::read_to_string(path).map_err(|e| {
                let file_display = path.display();
                if let Some(span) = import_span {
                    ResolverError::io_error_with_span(
                        &format!("Failed to read '{}': {}", file_display, e),
                        span,
                    )
                } else {
                    ResolverError::io_error(&format!("Failed to read '{}': {}", file_display, e))
                }
            })?,
        };

        let mut lexer = Lexer::new(&source);
        let tokens = lexer.tokenize().map_err(|e| {
            if import_span.is_some() {
                // For imported modules, use the original error span within the imported file
                // and carry the imported file's source context for proper error reporting.
                ResolverError::with_source_context(
                    ResolverErrorKind::LexError,
                    format!(
                        "Lexical error in module '{}': {}",
                        path.display(),
                        e.message()
                    ),
                    e.span(),
                    path.display().to_string(),
                    source.clone(),
                )
            } else {
                // For the entry module, use the original error span directly.
                ResolverError::with_span(
                    ResolverErrorKind::LexError,
                    format!(
                        "Lexical error in module '{}': {}",
                        path.display(),
                        e.message()
                    ),
                    e.span(),
                )
            }
        })?;

        let mut parser = Parser::new(tokens);
        let program = parser.parse().map_err(|e| {
            if import_span.is_some() {
                ResolverError::with_source_context(
                    ResolverErrorKind::ParseError,
                    format!(
                        "Parse error in module '{}': {}",
                        path.display(),
                        e.message()
                    ),
                    e.span(),
                    path.display().to_string(),
                    source.clone(),
                )
            } else {
                ResolverError::with_span(
                    ResolverErrorKind::ParseError,
                    format!(
                        "Parse error in module '{}': {}",
                        path.display(),
                        e.message()
                    ),
                    e.span(),
                )
            }
        })?;

        // Extract module name
        // For entry modules (import_span is None), skip validation and use file stem as-is
        // since entry modules are never referenced by name in import statements
        let module_name = if let Some(span) = import_span {
            Self::module_name_from_path(path, span)?
        } else {
            // Entry module: use file stem without validation
            path.file_stem()
                .and_then(|s| s.to_str())
                .ok_or_else(|| {
                    ResolverError::io_error(&format!(
                        "Cannot extract module name from entry path '{}'",
                        path.display()
                    ))
                })?
                .to_string()
        };

        // Collect dependencies and resolve them
        let mut dependencies = Vec::new();
        let mut resolved_imports = HashMap::new();
        for import in &program.imports {
            let import_path = Self::resolve_import_path(&import.path, path, import.span)?;
            let import_module_name = Self::module_name_from_path(&import_path, import.span)?;
            dependencies.push(import_module_name);
            resolved_imports.insert(import.path.clone(), import_path.clone());

            // Recursively resolve imported module
            self.resolve_module(&import_path, Some(import.span), None)?;
        }

        // Cache the resolved module
        let module = ResolvedModule {
            path: path.to_path_buf(),
            name: module_name,
            program,
            source,
            dependencies,
            resolved_imports,
        };
        self.modules.insert(path.to_path_buf(), module);

        Ok(())
    }

    /// Resolves an import path to an absolute file path.
    fn resolve_import_path(
        import_path: &str,
        importing_file: &Path,
        span: Span,
    ) -> Result<PathBuf, ResolverError> {
        if import_path.starts_with("./") || import_path.starts_with("../") {
            // Relative path: resolve relative to the importing file's directory
            let base_dir = importing_file.parent().ok_or_else(|| {
                ResolverError::invalid_import_path("Cannot determine parent directory", span)
            })?;

            let mut resolved = base_dir.join(import_path);

            // Add .lak extension if not present
            if resolved.extension().is_none() {
                resolved.set_extension("lak");
            }

            // Canonicalize to absolute path
            // Note: canonicalize does not prevent path traversal attacks.
            // Import paths like "../../../etc/passwd" could resolve to
            // files outside the project. This is acceptable for a compiler
            // since the user controls the source files.
            resolved.canonicalize().map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    ResolverError::file_not_found(import_path, span)
                } else {
                    ResolverError::io_error_with_span(
                        &format!("Failed to resolve import path '{}': {}", import_path, e),
                        span,
                    )
                }
            })
        } else {
            // Standard library path (not implemented yet)
            Err(ResolverError::standard_library_not_supported(
                import_path,
                span,
            ))
        }
    }

    /// Extracts module name from a file path.
    fn module_name_from_path(path: &Path, span: Span) -> Result<String, ResolverError> {
        let file_stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| ResolverError::invalid_module_name(&path.display().to_string(), span))?;

        if !is_valid_identifier(file_stem) {
            return Err(ResolverError::invalid_module_name(
                &path.display().to_string(),
                span,
            ));
        }

        Ok(file_stem.to_string())
    }

    /// Formats a circular import error message.
    fn format_cycle(&self, path: &Path) -> String {
        let mut cycle_parts = Vec::new();
        let mut found = false;

        for p in &self.resolving {
            if p == path {
                found = true;
            }
            if found {
                if let Some(name) = p.file_stem().and_then(|s| s.to_str()) {
                    cycle_parts.push(name.to_string());
                } else {
                    cycle_parts.push(p.display().to_string());
                }
            }
        }

        if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
            cycle_parts.push(name.to_string());
        } else {
            cycle_parts.push(path.display().to_string());
        }

        cycle_parts.join(" -> ")
    }

    /// Returns all resolved modules.
    pub fn into_modules(self) -> Vec<ResolvedModule> {
        let mut modules: Vec<_> = self.modules.into_values().collect();
        modules.sort_by(|a, b| a.path().cmp(b.path()));
        modules
    }
}

impl Default for ModuleResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Checks if a string is a valid Lak identifier.
///
/// Valid identifiers start with an ASCII letter or underscore, followed by
/// ASCII alphanumeric characters or underscores.
fn is_valid_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Extracts module name from an import path.
///
/// Uses `Path::file_stem()` for robust path handling and validates that
/// the resulting name is a valid identifier.
///
/// # Examples
///
/// - `"./utils"` -> `Some("utils")`
/// - `"./lib/math"` -> `Some("math")`
/// - `"../helpers"` -> `Some("helpers")`
/// - `"./utils.lak"` -> `Some("utils")`
/// - `"./123invalid"` -> `None`
pub fn extract_module_name(path: &str) -> Option<String> {
    let name = std::path::Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())?;
    if is_valid_identifier(&name) {
        Some(name)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_module_name() {
        assert_eq!(extract_module_name("./utils"), Some("utils".to_string()));
        assert_eq!(extract_module_name("./lib/math"), Some("math".to_string()));
        assert_eq!(
            extract_module_name("../helpers"),
            Some("helpers".to_string())
        );
        assert_eq!(
            extract_module_name("./utils.lak"),
            Some("utils".to_string())
        );
    }

    #[test]
    fn test_is_valid_identifier() {
        assert!(super::is_valid_identifier("foo"));
        assert!(super::is_valid_identifier("_bar"));
        assert!(super::is_valid_identifier("foo_bar"));
        assert!(super::is_valid_identifier("foo123"));
        assert!(super::is_valid_identifier("_"));
        assert!(!super::is_valid_identifier(""));
        assert!(!super::is_valid_identifier("123foo"));
        assert!(!super::is_valid_identifier("foo-bar"));
        assert!(!super::is_valid_identifier("foo.bar"));
    }

    #[test]
    fn test_extract_module_name_validates_identifier() {
        assert_eq!(extract_module_name("./123invalid"), None);
        assert_eq!(extract_module_name("./foo-bar"), None);
        assert_eq!(
            extract_module_name("./valid_name"),
            Some("valid_name".to_string())
        );
    }
}
