//! The Lak programming language compiler.
//!
//! This is the main entry point for the Lak compiler CLI. It provides
//! commands to build Lak source files into native executables.
//!
//! # Usage
//!
//! ```text
//! lak build <file.lak>
//! ```
//!
//! # Architecture
//!
//! The compiler follows a traditional pipeline:
//!
//! 1. **Lexing** ([`lexer`]) - Converts source text into tokens
//! 2. **Parsing** ([`parser`]) - Builds an AST from tokens
//! 3. **Code Generation** ([`codegen`]) - Generates LLVM IR from the AST
//! 4. **Linking** - Uses the system linker to produce an executable
//!
//! # Error Reporting
//!
//! The compiler uses [ariadne](https://docs.rs/ariadne) for beautiful,
//! colorful error messages that show the exact location of problems
//! in the source code.

use ariadne::{Color, Label, Report, ReportKind, Source};
use clap::{Parser, Subcommand};
use inkwell::context::Context;
use lak::codegen::Codegen;
use lak::lexer::{LexError, Lexer};
use lak::parser::{ParseError, Parser as LakParser};
use std::path::Path;
use std::process::Command;

/// Command-line interface for the Lak compiler.
///
/// Uses [clap](https://docs.rs/clap) for argument parsing with
/// derive-based configuration.
#[derive(Parser)]
#[command(name = "lak")]
#[command(about = "The Lak programming language", long_about = None)]
struct Cli {
    /// The subcommand to execute.
    #[command(subcommand)]
    command: Commands,
}

/// Available CLI subcommands.
#[derive(Subcommand)]
enum Commands {
    /// Build a Lak program into a native executable.
    Build {
        /// The source file to compile (e.g., `hello.lak`).
        file: String,
    },
}

/// Entry point for the Lak compiler.
///
/// Parses command-line arguments and dispatches to the appropriate handler.
fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build { file } => {
            if let Err(e) = build(&file) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }
}

/// A compilation error from any phase of the compiler.
///
/// This enum unifies errors from lexing, parsing, and code generation
/// to simplify error handling in the build pipeline.
enum CompileError {
    /// An error during lexical analysis.
    Lex(LexError),
    /// An error during parsing.
    Parse(ParseError),
    /// An error during code generation.
    Codegen(String),
}

/// Reports a compilation error with source location highlighting.
///
/// Uses [ariadne](https://docs.rs/ariadne) to produce beautiful error
/// messages that show the exact location in the source code.
///
/// # Arguments
///
/// * `filename` - The name of the source file (for display purposes)
/// * `source` - The source code content
/// * `error` - The error to report
fn report_error(filename: &str, source: &str, error: CompileError) {
    match error {
        CompileError::Lex(e) => {
            Report::build(ReportKind::Error, (filename, e.span.start..e.span.end))
                .with_message(&e.message)
                .with_label(
                    Label::new((filename, e.span.start..e.span.end))
                        .with_message(&e.message)
                        .with_color(Color::Red),
                )
                .finish()
                .eprint((filename, Source::from(source)))
                .ok();
        }
        CompileError::Parse(e) => {
            Report::build(ReportKind::Error, (filename, e.span.start..e.span.end))
                .with_message(&e.message)
                .with_label(
                    Label::new((filename, e.span.start..e.span.end))
                        .with_message(&e.message)
                        .with_color(Color::Red),
                )
                .finish()
                .eprint((filename, Source::from(source)))
                .ok();
        }
        CompileError::Codegen(msg) => {
            eprintln!("Error: {}", msg);
        }
    }
}

/// Builds a Lak source file into a native executable.
///
/// This function orchestrates the entire compilation pipeline:
///
/// 1. Read the source file
/// 2. Tokenize the source code
/// 3. Parse tokens into an AST
/// 4. Generate LLVM IR
/// 5. Write to an object file
/// 6. Link with the system linker (`cc`)
/// 7. Clean up temporary files
///
/// # Arguments
///
/// * `file` - Path to the Lak source file
///
/// # Returns
///
/// * `Ok(())` - Compilation succeeded, executable written to disk
/// * `Err(String)` - Compilation failed with an error message
///
/// # Output Files
///
/// Given an input file `example.lak`, this function produces:
/// - `example` - The final executable
/// - `example.o` - Temporary object file (deleted after linking)
fn build(file: &str) -> Result<(), String> {
    let source =
        std::fs::read_to_string(file).map_err(|e| format!("Failed to read file: {}", e))?;

    let mut lexer = Lexer::new(&source);
    let tokens = match lexer.tokenize() {
        Ok(tokens) => tokens,
        Err(e) => {
            report_error(file, &source, CompileError::Lex(e));
            return Err("Compilation failed".to_string());
        }
    };

    let mut parser = LakParser::new(tokens);
    let program = match parser.parse() {
        Ok(program) => program,
        Err(e) => {
            report_error(file, &source, CompileError::Parse(e));
            return Err("Compilation failed".to_string());
        }
    };

    let context = Context::create();
    let mut codegen = Codegen::new(&context, "lak_module");
    if let Err(e) = codegen.compile(&program) {
        report_error(file, &source, CompileError::Codegen(e));
        return Err("Compilation failed".to_string());
    }

    let source_path = Path::new(file);
    let stem = source_path
        .file_stem()
        .ok_or_else(|| format!("Cannot determine filename from path: {}", file))?
        .to_str()
        .ok_or_else(|| format!("Filename contains invalid UTF-8: {}", file))?;

    let object_path = format!("{}.o", stem);
    let output_path = stem.to_string();

    codegen.write_object_file(Path::new(&object_path))?;

    let output = Command::new("cc")
        .args([&object_path, "-o", &output_path])
        .output()
        .map_err(|e| format!("Failed to run linker: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let exit_code = output
            .status
            .code()
            .map(|c| c.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        return Err(format!(
            "Linker failed with exit code {}:\n{}",
            exit_code, stderr
        ));
    }

    if let Err(e) = std::fs::remove_file(&object_path)
        && e.kind() != std::io::ErrorKind::NotFound
    {
        eprintln!(
            "Warning: Failed to remove temporary file '{}': {}",
            object_path, e
        );
    }

    println!("Built: {}", output_path);
    Ok(())
}
