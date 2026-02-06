//! The Lak programming language compiler.
//!
//! This is the main entry point for the Lak compiler CLI. It provides
//! commands to build Lak source files into native executables.
//!
//! # Usage
//!
//! ```text
//! lak build <file.lak>
//! lak build <file.lak> -o <output>
//! lak run <file.lak>
//! ```
//!
//! # Architecture
//!
//! The compiler follows a traditional pipeline:
//!
//! 1. **Lexing** ([`lexer`]) - Converts source text into tokens
//! 2. **Parsing** ([`parser`]) - Builds an AST from tokens
//! 3. **Semantic Analysis** ([`semantic`]) - Validates the AST for correctness
//! 4. **Code Generation** ([`codegen`]) - Generates LLVM IR from the AST
//! 5. **Linking** - Uses the system linker to produce an executable
//!
//! # Error Reporting
//!
//! The compiler uses [ariadne](https://docs.rs/ariadne) for beautiful,
//! colorful error messages that show the exact location of problems
//! in the source code.

use ariadne::{Color, Config, IndexType, Label, Report, ReportKind, Source};
use clap::{Parser, Subcommand};
use inkwell::context::Context;
use lak::codegen::{Codegen, CodegenError};
use lak::lexer::{LexError, Lexer};
use lak::parser::{ParseError, Parser as LakParser};
use lak::semantic::{SemanticAnalyzer, SemanticError, SemanticErrorKind};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};
use tempfile::TempDir;

#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;

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

        /// Output path for the executable (e.g., `-o myprogram`).
        /// If not specified, uses the input filename without extension.
        #[arg(short = 'o', long = "output")]
        output: Option<String>,
    },
    /// Compile and run a Lak program.
    Run {
        /// The source file to run (e.g., `hello.lak`).
        file: String,
    },
}

/// Entry point for the Lak compiler.
///
/// Parses command-line arguments and dispatches to the appropriate handler.
fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build { file, output } => {
            if build(&file, output.as_deref()).is_err() {
                std::process::exit(1);
            }
        }
        Commands::Run { file } => match run(&file) {
            Ok(exit_code) => std::process::exit(exit_code),
            Err(_) => {
                std::process::exit(1);
            }
        },
    }
}

/// A compilation error from any phase of the compiler.
///
/// This enum unifies errors from lexing, parsing, semantic analysis,
/// and code generation to simplify error handling in the build pipeline.
enum CompileError {
    /// An error during lexical analysis.
    Lex(LexError),
    /// An error during parsing.
    Parse(ParseError),
    /// An error during semantic analysis.
    Semantic(SemanticError),
    /// An error during code generation.
    Codegen(CodegenError),
}

/// Reports a compilation error with source location highlighting.
///
/// Uses [ariadne](https://docs.rs/ariadne) to produce beautiful error
/// messages that show the exact location in the source code.
///
/// # Arguments
///
/// * `filename` - The name of the source file (for display purposes)
/// * `source` - The source code content (must be the actual contents of `filename`)
/// * `error` - The error to report
///
/// # Important
///
/// The `source` parameter **must** be the actual contents of `filename`.
/// Passing mismatched values will produce confusing error messages that
/// point to the wrong file or show incorrect source context.
fn report_error(filename: &str, source: &str, error: CompileError) {
    match error {
        CompileError::Lex(e) => {
            if let Err(report_err) =
                Report::build(ReportKind::Error, (filename, e.span.start..e.span.end))
                    .with_config(Config::default().with_index_type(IndexType::Byte))
                    .with_message(&e.message)
                    .with_label(
                        Label::new((filename, e.span.start..e.span.end))
                            .with_message(&e.message)
                            .with_color(Color::Red),
                    )
                    .finish()
                    .eprint((filename, Source::from(source)))
            {
                // Fallback to basic error output if report printing fails
                eprintln!(
                    "Error: {} (at {}:{})",
                    e.message, e.span.line, e.span.column
                );
                eprintln!("(Failed to display detailed error report: {})", report_err);
            }
        }
        CompileError::Parse(e) => {
            if let Err(report_err) =
                Report::build(ReportKind::Error, (filename, e.span.start..e.span.end))
                    .with_config(Config::default().with_index_type(IndexType::Byte))
                    .with_message(&e.message)
                    .with_label(
                        Label::new((filename, e.span.start..e.span.end))
                            .with_message(&e.message)
                            .with_color(Color::Red),
                    )
                    .finish()
                    .eprint((filename, Source::from(source)))
            {
                // Fallback to basic error output if report printing fails
                eprintln!(
                    "Error: {} (at {}:{})",
                    e.message, e.span.line, e.span.column
                );
                eprintln!("(Failed to display detailed error report: {})", report_err);
            }
        }
        CompileError::Semantic(e) => {
            if let Some(span) = e.span() {
                if let Err(report_err) =
                    Report::build(ReportKind::Error, (filename, span.start..span.end))
                        .with_config(Config::default().with_index_type(IndexType::Byte))
                        .with_message(e.message())
                        .with_label(
                            Label::new((filename, span.start..span.end))
                                .with_message(e.message())
                                .with_color(Color::Red),
                        )
                        .finish()
                        .eprint((filename, Source::from(source)))
                {
                    // Fallback to basic error output if report printing fails
                    eprintln!("Error: {} (at {}:{})", e.message(), span.line, span.column);
                    eprintln!("(Failed to display detailed error report: {})", report_err);
                }
            } else {
                // No span available - show error report pointing to end of file.
                let end = source.len().saturating_sub(1);
                let span_range = if source.is_empty() {
                    0..0
                } else {
                    end..source.len()
                };

                // Customize label and help based on error kind
                let is_missing_main = e.kind() == SemanticErrorKind::MissingMainFunction;
                let label_msg: &str = if is_missing_main {
                    "main function not found"
                } else {
                    e.message()
                };

                let mut report = Report::build(ReportKind::Error, (filename, span_range.clone()))
                    .with_config(Config::default().with_index_type(IndexType::Byte))
                    .with_message(e.message())
                    .with_label(
                        Label::new((filename, span_range))
                            .with_message(label_msg)
                            .with_color(Color::Red),
                    );

                if is_missing_main {
                    report = report.with_help("add a main function: fn main() -> void { ... }");
                }

                if let Err(report_err) = report.finish().eprint((filename, Source::from(source))) {
                    // Fallback to basic error output if report printing fails
                    eprintln!("Error in {}: {}", filename, e.message());
                    eprintln!("(Failed to display detailed error report: {})", report_err);
                }
            }
        }
        CompileError::Codegen(e) => {
            // Codegen errors are infrastructure errors (InternalError, TargetError)
            // that typically don't have source locations
            if let Some(span) = e.span() {
                if let Err(report_err) =
                    Report::build(ReportKind::Error, (filename, span.start..span.end))
                        .with_config(Config::default().with_index_type(IndexType::Byte))
                        .with_message(e.message())
                        .with_label(
                            Label::new((filename, span.start..span.end))
                                .with_message(e.message())
                                .with_color(Color::Red),
                        )
                        .finish()
                        .eprint((filename, Source::from(source)))
                {
                    // Fallback to basic error output if report printing fails
                    eprintln!("Error: {} (at {}:{})", e.message(), span.line, span.column);
                    eprintln!("(Failed to display detailed error report: {})", report_err);
                }
            } else {
                // Infrastructure errors without source location
                eprintln!("Error in {}: {}", filename, e.message());
            }
        }
    }
}

/// Path to the Lak runtime library, set at compile time by build.rs.
const LAK_RUNTIME_PATH: &str = env!("LAK_RUNTIME_PATH");

/// Formats an exit status for display, including signal information on Unix.
fn format_exit_status(status: &ExitStatus) -> String {
    if let Some(code) = status.code() {
        return code.to_string();
    }

    #[cfg(unix)]
    {
        if let Some(signal) = status.signal() {
            return format!("signal {}", signal);
        }
    }

    "unknown".to_string()
}

/// Returns the exit code from an exit status, handling signals on Unix.
///
/// On Unix, if the process was terminated by a signal, returns 128 + signal number
/// following the shell convention. Otherwise returns the exit code or 1 as fallback.
fn get_exit_code_with_signal(status: &ExitStatus) -> i32 {
    if let Some(code) = status.code() {
        return code;
    }

    #[cfg(unix)]
    {
        if let Some(signal) = status.signal() {
            eprintln!("Program terminated by signal {}", signal);
            return 128 + signal;
        }
    }

    eprintln!("Program terminated abnormally");
    1
}

/// Compiles a Lak source file and links it into an executable.
///
/// This is the shared compilation pipeline used by both `build` and `run` commands.
///
/// # Arguments
///
/// * `file` - Path to the source file (for error reporting)
/// * `source` - The source code content
/// * `object_path` - Path to write the object file
/// * `output_path` - Path to write the final executable
///
/// # Returns
///
/// * `Ok(())` - Compilation and linking succeeded
/// * `Err(String)` - Compilation failed with an error message
fn compile_to_executable(
    file: &str,
    source: &str,
    object_path: &Path,
    output_path: &Path,
) -> Result<(), String> {
    let mut lexer = Lexer::new(source);
    let tokens = match lexer.tokenize() {
        Ok(tokens) => tokens,
        Err(e) => {
            report_error(file, source, CompileError::Lex(e));
            return Err("Compilation failed".to_string());
        }
    };

    let mut parser = LakParser::new(tokens);
    let program = match parser.parse() {
        Ok(program) => program,
        Err(e) => {
            report_error(file, source, CompileError::Parse(e));
            return Err("Compilation failed".to_string());
        }
    };

    // Semantic analysis
    let mut analyzer = SemanticAnalyzer::new();
    if let Err(e) = analyzer.analyze(&program) {
        report_error(file, source, CompileError::Semantic(e));
        return Err("Compilation failed".to_string());
    }

    let context = Context::create();
    let mut codegen = Codegen::new(&context, "lak_module");
    codegen.compile(&program).map_err(|e| {
        report_error(file, source, CompileError::Codegen(e));
        "Compilation failed".to_string()
    })?;

    codegen.write_object_file(object_path).map_err(|e| {
        report_error(file, source, CompileError::Codegen(e));
        "Compilation failed".to_string()
    })?;

    let object_str = object_path
        .to_str()
        .ok_or_else(|| format!("Object path {:?} is not valid UTF-8", object_path))?;
    let output_str = output_path
        .to_str()
        .ok_or_else(|| format!("Output path {:?} is not valid UTF-8", output_path))?;

    let output = Command::new("cc")
        .args([object_str, LAK_RUNTIME_PATH, "-o", output_str])
        .output()
        .map_err(|e| format!("Failed to run linker: {}", e))?;

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let exit_code = format_exit_status(&output.status);

        let mut error_msg = format!("Linker failed with exit code {}:", exit_code);
        if !stdout.is_empty() {
            error_msg.push_str(&format!("\n[stdout]\n{}", stdout));
        }
        if !stderr.is_empty() {
            error_msg.push_str(&format!("\n[stderr]\n{}", stderr));
        }
        return Err(error_msg);
    }

    Ok(())
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
/// 6. Link with the system linker (`cc`) and Lak runtime
/// 7. Clean up temporary files
///
/// # Arguments
///
/// * `file` - Path to the Lak source file
/// * `output` - Optional path for the output executable. If `None`, uses input file stem.
///
/// # Returns
///
/// * `Ok(())` - Compilation succeeded, executable written to disk
/// * `Err(String)` - Compilation failed with an error message
///
/// # Output Files
///
/// Given an input file `example.lak`:
/// - Without `-o`: produces `example` executable
/// - With `-o myapp`: produces `myapp` executable
/// - `example.o` - Temporary object file (deleted after linking)
fn build(file: &str, output: Option<&str>) -> Result<(), String> {
    let source =
        std::fs::read_to_string(file).map_err(|e| format!("Failed to read file: {}", e))?;

    let source_path = Path::new(file);
    let stem = source_path
        .file_stem()
        .ok_or_else(|| format!("Cannot determine filename from path: {}", file))?
        .to_str()
        .ok_or_else(|| format!("Filename contains invalid UTF-8: {}", file))?;

    let object_path = PathBuf::from(format!("{}.o", stem));
    let output_path = match output {
        Some(path) => PathBuf::from(path),
        None => PathBuf::from(stem),
    };

    compile_to_executable(file, &source, &object_path, &output_path)?;

    if let Err(e) = std::fs::remove_file(&object_path)
        && e.kind() != std::io::ErrorKind::NotFound
    {
        eprintln!(
            "Warning: Failed to remove temporary file '{}': {}",
            object_path.display(),
            e
        );
    }

    println!("Built: {}", output_path.display());
    Ok(())
}

/// Compiles and runs a Lak source file.
///
/// This function:
/// 1. Creates a temporary directory for build artifacts
/// 2. Compiles the source file to an executable in the temp directory
/// 3. Runs the executable
/// 4. Returns the exit code of the executed program
/// 5. Cleans up temporary files explicitly (with warning on failure)
///
/// # Arguments
///
/// * `file` - Path to the Lak source file
///
/// # Returns
///
/// * `Ok(i32)` - The exit code of the executed program
/// * `Err(String)` - Compilation or execution failed
fn run(file: &str) -> Result<i32, String> {
    let source =
        std::fs::read_to_string(file).map_err(|e| format!("Failed to read file: {}", e))?;

    // Create temporary directory
    let temp_dir =
        TempDir::new().map_err(|e| format!("Failed to create temporary directory: {}", e))?;

    let object_path = temp_dir.path().join("program.o");
    let executable_path = temp_dir.path().join("program");

    compile_to_executable(file, &source, &object_path, &executable_path)?;

    // Run the executable
    let exec_str = executable_path
        .to_str()
        .ok_or_else(|| format!("Executable path {:?} is not valid UTF-8", executable_path))?;

    let status = Command::new(exec_str)
        .status()
        .map_err(|e| format!("Failed to run executable: {}", e))?;

    let exit_code = get_exit_code_with_signal(&status);

    // Explicitly clean up and report any errors
    let temp_path = temp_dir.keep();
    if let Err(e) = std::fs::remove_dir_all(&temp_path) {
        eprintln!(
            "Warning: Failed to clean up temporary directory '{}': {}",
            temp_path.display(),
            e
        );
    }

    Ok(exit_code)
}
