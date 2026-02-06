//! Common test utilities for Lak integration tests.
//!
//! This module provides shared helper functions and types used across
//! all integration test files.

// Each test file is compiled as a separate crate, so not all functions
// are used in every test file. This is expected behavior.
#![allow(dead_code)]

use lak::codegen::{Codegen, CodegenError, CodegenErrorKind};
use lak::lexer::{LexErrorKind, Lexer};
use lak::parser::{ParseErrorKind, Parser};
use lak::semantic::{SemanticAnalyzer, SemanticErrorKind};
use lak::token::Span;

use inkwell::context::Context;
use std::process::Command;
use tempfile::tempdir;

/// Path to the Lak runtime library, set at compile time by build.rs.
pub const LAK_RUNTIME_PATH: &str = env!("LAK_RUNTIME_PATH");

/// Creates a dummy span for test AST construction.
pub fn dummy_span() -> Span {
    Span::new(0, 0, 1, 1)
}

/// Compiles a Lak program to an executable and runs it, returning stdout output.
///
/// This function performs the complete compilation pipeline:
/// lexing -> parsing -> codegen -> linking -> execution.
/// Only stdout is captured; stderr is not included in the success case.
pub fn compile_and_run(source: &str) -> Result<String, String> {
    // Lex
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().map_err(|e| e.to_string())?;

    // Parse
    let mut parser = Parser::new(tokens);
    let program = parser.parse().map_err(|e| e.to_string())?;

    // Codegen
    let context = Context::create();
    let mut codegen = Codegen::new(&context, "integration_test");
    codegen
        .compile(&program)
        .map_err(|e: CodegenError| e.message().to_string())?;

    // Write object file
    let temp_dir = tempdir().map_err(|e| e.to_string())?;
    let object_path = temp_dir.path().join("test.o");
    let executable_path = temp_dir.path().join("test");

    codegen
        .write_object_file(&object_path)
        .map_err(|e| e.to_string())?;

    // Link
    let object_str = object_path
        .to_str()
        .ok_or_else(|| format!("Object path {:?} is not valid UTF-8", object_path))?;
    let exec_str = executable_path
        .to_str()
        .ok_or_else(|| format!("Executable path {:?} is not valid UTF-8", executable_path))?;

    let link_output = Command::new("cc")
        .args([object_str, LAK_RUNTIME_PATH, "-o", exec_str])
        .output()
        .map_err(|e| format!("Failed to run linker: {}", e))?;

    if !link_output.status.success() {
        return Err(format!(
            "Linker failed: {}",
            String::from_utf8_lossy(&link_output.stderr)
        ));
    }

    // Run
    let run_output = Command::new(&executable_path)
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

    let mut parser = Parser::new(tokens);
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
/// Returns the stage, error message, and error kind if any stage fails.
pub fn compile_error_with_kind(source: &str) -> Option<(CompileStage, String, CompileErrorKind)> {
    let mut lexer = Lexer::new(source);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => {
            return Some((
                CompileStage::Lex,
                e.message().to_string(),
                CompileErrorKind::Lex(e.kind()),
            ));
        }
    };

    let mut parser = Parser::new(tokens);
    let program = match parser.parse() {
        Ok(p) => p,
        Err(e) => {
            return Some((
                CompileStage::Parse,
                e.message().to_string(),
                CompileErrorKind::Parse(e.kind()),
            ));
        }
    };

    let mut analyzer = SemanticAnalyzer::new();
    if let Err(e) = analyzer.analyze(&program) {
        return Some((
            CompileStage::Semantic,
            e.message().to_string(),
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
            CompileErrorKind::Codegen(e.kind()),
        )),
    }
}
