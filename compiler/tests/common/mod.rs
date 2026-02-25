//! Common test utilities for Lak integration tests.
//!
//! This module provides shared helper functions and types used across
//! all integration test files.

// Each test file is compiled as a separate crate, so not all functions
// are used in every test file. This is expected behavior.
#![allow(dead_code)]

use lak::codegen::{Codegen, CodegenError, CodegenErrorKind};
use lak::lexer::{LexErrorKind, Lexer};
use lak::linker;
use lak::parser::{ParseErrorKind, Parser};
use lak::semantic::{SemanticAnalyzer, SemanticErrorKind};
use lak::token::Span;

use inkwell::context::Context;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;

/// Returns the runtime static library filename for the current target.
pub fn runtime_library_filename() -> &'static str {
    linker::runtime_library_filename()
}

/// Returns the runtime library path expected next to the given `lak` binary path.
pub fn runtime_library_path_for_binary(binary_path: &Path) -> Result<PathBuf, String> {
    linker::runtime_library_path_for_binary(binary_path).map_err(|e| e.to_string())
}

/// Copies the built `lak` binary into `dir` and returns the copied path.
pub fn copy_lak_binary_to(dir: &Path) -> Result<PathBuf, String> {
    let source = PathBuf::from(lak_binary());
    let file_name = source.file_name().ok_or_else(|| {
        format!(
            "Lak binary path '{}' does not contain a file name",
            source.display()
        )
    })?;
    let destination = dir.join(file_name);
    fs::copy(&source, &destination).map_err(|e| {
        format!(
            "Failed to copy lak binary from '{}' to '{}': {}",
            source.display(),
            destination.display(),
            e
        )
    })?;
    Ok(destination)
}

/// Resolves the runtime library path for integration test linking and
/// validates that the path exists and points to a regular file.
fn resolve_runtime_library_path() -> Result<PathBuf, String> {
    let lak_path = PathBuf::from(lak_binary());
    linker::resolve_runtime_library_path_for_binary(&lak_path).map_err(|e| e.to_string())
}

/// Returns an executable filename with the correct platform extension.
/// e.g., "test" → "test" on Unix, "test.exe" on Windows.
pub fn executable_name(name: &str) -> String {
    format!("{}{}", name, std::env::consts::EXE_SUFFIX)
}

/// Returns the path to the lak binary built by cargo.
///
/// Used for tests that need to verify panic behavior and exit codes,
/// which the standard `compile_and_run()` helper doesn't expose.
pub fn lak_binary() -> String {
    env!("CARGO_BIN_EXE_lak").to_string()
}

/// Creates a dummy span for test AST construction.
pub fn dummy_span() -> Span {
    Span::new(0, 0, 1, 1)
}

/// Compiles a single-file Lak program to an object file.
pub fn compile_source_to_object(source: &str, object_path: &Path) -> Result<(), String> {
    // Lex
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().map_err(|e| e.to_string())?;

    // Parse
    let mut parser = Parser::try_new(tokens).map_err(|e| e.to_string())?;
    let program = parser.parse().map_err(|e| e.to_string())?;

    // Semantic analysis
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(&program).map_err(|e| e.to_string())?;

    // Codegen
    let context = Context::create();
    let mut codegen = Codegen::new(&context, "integration_test");
    codegen
        .compile(&program)
        .map_err(|e: CodegenError| e.message().to_string())?;

    codegen
        .write_object_file(object_path)
        .map_err(|e| e.to_string())
}

/// Links an object file into an executable.
pub fn link_object(object_path: &Path, executable_path: &Path) -> Result<(), String> {
    let object_str = object_path
        .to_str()
        .ok_or_else(|| format!("Object path {:?} is not valid UTF-8", object_path))?;
    let exec_str = executable_path
        .to_str()
        .ok_or_else(|| format!("Executable path {:?} is not valid UTF-8", executable_path))?;
    let runtime_path = resolve_runtime_library_path()?;
    let runtime_str = runtime_path
        .to_str()
        .ok_or_else(|| format!("Runtime path {:?} is not valid UTF-8", runtime_path))?;

    let link_output = linker::create_linker_command(object_str, runtime_str, exec_str)
        .map_err(|e| e.to_string())?
        .output()
        .map_err(|e| format!("Failed to run linker: {}", e))?;

    if !link_output.status.success() {
        let stdout = String::from_utf8_lossy(&link_output.stdout);
        let stderr = String::from_utf8_lossy(&link_output.stderr);
        return Err(format!(
            "Linker failed (exit code {:?}):\n[stdout] {}\n[stderr] {}",
            link_output.status.code(),
            stdout,
            stderr
        ));
    }

    Ok(())
}

/// Runs an executable and returns stdout output.
pub fn run_executable(executable_path: &Path) -> Result<String, String> {
    let run_output = Command::new(executable_path)
        .output()
        .map_err(|e| format!("Failed to run executable: {}", e))?;

    if !run_output.status.success() {
        return Err(format!(
            "Executable failed with exit code: {:?}",
            run_output.status.code()
        ));
    }

    Ok(String::from_utf8_lossy(&run_output.stdout).to_string())
}

/// Compiles a single-file Lak program to an executable and runs it, returning stdout output.
///
/// This function performs the complete single-file compilation pipeline:
/// lexing -> parsing -> semantic analysis -> codegen -> linking -> execution.
/// Only stdout is captured; stderr is not included in the success case.
///
/// Note: This helper does not support multi-module compilation (no resolver).
/// For multi-module tests, use the CLI binary approach (see `e2e_modules.rs`).
pub fn compile_and_run(source: &str) -> Result<String, String> {
    let temp_dir = tempdir().map_err(|e| e.to_string())?;
    let object_path = temp_dir.path().join("test.o");
    let executable_path = temp_dir.path().join(executable_name("test"));

    compile_source_to_object(source, &object_path)?;
    link_object(&object_path, &executable_path)?;
    run_executable(&executable_path)
}

/// Represents the stage at which compilation failed.
#[derive(Debug)]
pub enum CompileStage {
    Lex,
    Parse,
    Semantic,
    Codegen,
}

/// Represents the error kind for each compilation stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompileErrorKind {
    Lex(LexErrorKind),
    Parse(ParseErrorKind),
    Semantic(SemanticErrorKind),
    Codegen(CodegenErrorKind),
}

/// Attempts to lex, parse, analyze, and compile a program.
/// Returns the stage and error message if any stage fails.
pub fn compile_error(source: &str) -> Option<(CompileStage, String)> {
    let mut lexer = Lexer::new(source);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => return Some((CompileStage::Lex, e.message().to_string())),
    };

    let mut parser = match Parser::try_new(tokens) {
        Ok(p) => p,
        Err(e) => return Some((CompileStage::Parse, e.message().to_string())),
    };
    let program = match parser.parse() {
        Ok(p) => p,
        Err(e) => return Some((CompileStage::Parse, e.message().to_string())),
    };

    let mut analyzer = SemanticAnalyzer::new();
    if let Err(e) = analyzer.analyze(&program) {
        return Some((CompileStage::Semantic, e.message().to_string()));
    }

    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");
    match codegen.compile(&program) {
        Ok(()) => None,
        Err(e) => Some((CompileStage::Codegen, e.message().to_string())),
    }
}

/// Attempts to lex, parse, analyze, and compile a program.
/// Returns the stage, error message, short message, and error kind if any stage fails.
///
/// The returned tuple contains:
/// - `CompileStage`: The stage at which the error occurred
/// - `String`: The detailed error message (used in labels)
/// - `String`: The short error message (used in report titles)
/// - `CompileErrorKind`: The kind of error for structured matching
pub fn compile_error_with_kind(
    source: &str,
) -> Option<(CompileStage, String, String, CompileErrorKind)> {
    let mut lexer = Lexer::new(source);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => {
            return Some((
                CompileStage::Lex,
                e.message().to_string(),
                e.short_message().to_string(),
                CompileErrorKind::Lex(e.kind()),
            ));
        }
    };

    let mut parser = match Parser::try_new(tokens) {
        Ok(p) => p,
        Err(e) => {
            return Some((
                CompileStage::Parse,
                e.message().to_string(),
                e.short_message().to_string(),
                CompileErrorKind::Parse(e.kind()),
            ));
        }
    };
    let program = match parser.parse() {
        Ok(p) => p,
        Err(e) => {
            return Some((
                CompileStage::Parse,
                e.message().to_string(),
                e.short_message().to_string(),
                CompileErrorKind::Parse(e.kind()),
            ));
        }
    };

    let mut analyzer = SemanticAnalyzer::new();
    if let Err(e) = analyzer.analyze(&program) {
        return Some((
            CompileStage::Semantic,
            e.message().to_string(),
            e.short_message().to_string(),
            CompileErrorKind::Semantic(e.kind()),
        ));
    }

    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");
    match codegen.compile(&program) {
        Ok(()) => None,
        Err(e) => Some((
            CompileStage::Codegen,
            e.message().to_string(),
            e.short_message().to_string(),
            CompileErrorKind::Codegen(e.kind()),
        )),
    }
}
