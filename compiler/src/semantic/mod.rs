//! Semantic analysis for the Lak programming language.
//!
//! This module provides the [`SemanticAnalyzer`] which validates a Lak AST
//! for semantic correctness before code generation.
//!
//! # Responsibilities
//!
//! The semantic analyzer performs the following validations:
//!
//! - **Name resolution**: Checks for duplicate/undefined functions and variables
//! - **Type checking**: Validates type consistency in assignments and expressions
//! - **Structural validation**: Ensures main function exists with correct signature
//!
//! # Pipeline Position
//!
//! ```text
//! Source → Lexer → Parser → [Module Resolver] → Semantic Analyzer → Codegen → Executable
//! ```
//!
//! The semantic analyzer sits between the parser and code generator. It takes
//! an AST and either returns success (allowing codegen to proceed) or an error
//! describing the semantic problem.

mod error;
mod module_table;
mod symbol;
mod symbols;
mod typecheck_expr;
mod typecheck_stmt;

#[cfg(test)]
mod tests;

pub use error::{SemanticError, SemanticErrorKind};
pub use module_table::ModuleTable;
use symbol::SymbolTable;

use crate::ast::Program;

/// The mode of semantic analysis, determining which validations are performed.
enum AnalysisMode {
    /// Analyzing a single-file program (no imports).
    SingleFile,
    /// Analyzing an entry module with imports. Contains the module table
    /// for validating cross-module references.
    EntryWithModules(ModuleTable),
    /// Analyzing an imported module (no main function required).
    /// Optionally carries a module table for imported modules that have their own imports.
    ImportedModule(Option<ModuleTable>),
}

/// Semantic analyzer for Lak programs.
///
/// Performs semantic validation on an AST without modifying it:
/// - Name resolution (duplicate/undefined checks)
/// - Type checking
/// - Structural validation (main function)
///
/// The analyzer guarantees that if `analyze()` succeeds, the AST is
/// semantically valid and code generation can proceed without semantic errors.
pub struct SemanticAnalyzer {
    symbols: SymbolTable,
    mode: AnalysisMode,
    current_function_return_type: Option<String>,
    loop_depth: usize,
}

impl SemanticAnalyzer {
    /// Creates a new semantic analyzer.
    pub fn new() -> Self {
        SemanticAnalyzer {
            symbols: SymbolTable::new(),
            mode: AnalysisMode::SingleFile,
            current_function_return_type: None,
            loop_depth: 0,
        }
    }

    /// Analyzes a program for semantic correctness.
    ///
    /// Performs complete semantic validation in this order:
    /// 1. Collect all function definitions (check for duplicates)
    /// 2. Validate main function exists and has correct signature
    /// 3. Analyze each function body (variables, types, expressions)
    ///
    /// # Errors
    ///
    /// Returns an error if any semantic violation is found:
    /// - Duplicate function definitions
    /// - Missing main function
    /// - Invalid main signature
    /// - Duplicate variable definitions
    /// - Undefined variable/function references
    /// - Type mismatches
    /// - Integer overflow
    pub fn analyze(&mut self, program: &Program) -> Result<(), SemanticError> {
        self.begin_session(AnalysisMode::SingleFile);
        self.analyze_program(program, true)
    }

    fn begin_session(&mut self, mode: AnalysisMode) {
        self.symbols = SymbolTable::new();
        self.mode = mode;
        self.current_function_return_type = None;
        self.loop_depth = 0;
    }

    fn analyze_program(
        &mut self,
        program: &Program,
        validate_main_function: bool,
    ) -> Result<(), SemanticError> {
        // Phase 1: Collect function definitions
        self.collect_functions(program)?;

        // Phase 2: Validate main function
        if validate_main_function {
            self.validate_main_function(program)?;
        }

        // Phase 3: Analyze function bodies
        for function in &program.functions {
            self.analyze_function(function)?;
        }

        Ok(())
    }

    /// Analyzes a program for semantic correctness with module context.
    ///
    /// This is used when compiling programs that import other modules.
    pub fn analyze_with_modules(
        &mut self,
        program: &Program,
        module_table: ModuleTable,
    ) -> Result<(), SemanticError> {
        self.begin_session(AnalysisMode::EntryWithModules(module_table));
        self.analyze_program(program, true)
    }

    /// Analyzes an imported module for semantic correctness.
    ///
    /// Unlike `analyze()`, this method does NOT require a main function,
    /// since imported modules are libraries, not entry points.
    ///
    /// Performs:
    /// 1. Function collection (check for duplicates)
    /// 2. Function body analysis (variables, types, expressions)
    pub fn analyze_module(
        &mut self,
        program: &Program,
        module_table: Option<ModuleTable>,
    ) -> Result<(), SemanticError> {
        self.begin_session(AnalysisMode::ImportedModule(module_table));
        self.analyze_program(program, false)
    }
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
