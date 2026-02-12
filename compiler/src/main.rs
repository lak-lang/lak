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
//! 3. **Module Resolution** ([`resolver`]) - Loads and parses imported modules, detects cycles
//! 4. **Semantic Analysis** ([`semantic`]) - Validates the AST for correctness
//! 5. **Code Generation** ([`codegen`]) - Generates LLVM IR from the AST
//! 6. **Linking** - Uses the system linker to produce an executable
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
use lak::resolver::{ModuleResolver, ResolvedModule, ResolverError};
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
            if let Err(e) = build(&file, output.as_deref()) {
                e.report();
                std::process::exit(1);
            }
        }
        Commands::Run { file } => match run(&file) {
            Ok(exit_code) => std::process::exit(exit_code),
            Err(e) => {
                e.report();
                std::process::exit(1);
            }
        },
    }
}

/// A compilation error from any phase of the compiler.
///
/// This enum unifies errors from module resolution, semantic analysis,
/// code generation, linking, and I/O to simplify error handling in the build pipeline.
enum CompileError {
    /// An error during module resolution (includes lex/parse errors in modules).
    Resolve(ResolverError),
    /// An error during semantic analysis.
    Semantic(SemanticError),
    /// A semantic error in an imported module, with its own source context.
    /// Boxed to keep the `CompileError` enum size reasonable.
    ModuleSemantic(Box<ModuleSemanticContext>),
    /// An error during code generation.
    Codegen(CodegenError),
    /// An error during linking.
    Link(LinkError),
    /// A path is not valid UTF-8.
    /// Uses `PathBuf` instead of `String` because the path cannot be converted to valid UTF-8.
    PathNotUtf8 {
        path: PathBuf,
        context: &'static str,
    },
    /// Failed to read a source file.
    FileReadError {
        path: String,
        source: std::io::Error,
    },
    /// Failed to resolve (canonicalize) a file path.
    PathResolutionError {
        path: String,
        source: std::io::Error,
    },
    /// Failed to create a temporary directory.
    TempDirCreationError(std::io::Error),
    /// Failed to run the compiled executable.
    ExecutableRunError(std::io::Error),
    /// Entry module not found after resolution.
    EntryModuleNotFound { path: String },
    /// Cannot determine filename from path or filename is not valid UTF-8.
    FilenameError { path: String, reason: &'static str },
}

/// Context for a semantic error in an imported module.
struct ModuleSemanticContext {
    error: SemanticError,
    filename: String,
    source: String,
}

/// A linker error.
enum LinkError {
    /// Failed to execute the linker command.
    ExecutionFailed(std::io::Error),
    /// Failed to resolve the absolute path of the current executable.
    CurrentExecutablePathResolutionFailed(std::io::Error),
    /// Current executable path has no parent directory.
    CurrentExecutableParentNotFound { executable: PathBuf },
    /// Lak runtime library was not found next to the lak executable.
    RuntimeLibraryNotFound { executable: PathBuf, path: PathBuf },
    /// Lak runtime library path exists but is not a regular file.
    RuntimeLibraryNotAFile { executable: PathBuf, path: PathBuf },
    /// Failed to access the runtime library path due to an I/O error.
    RuntimeLibraryAccessFailed {
        executable: PathBuf,
        path: PathBuf,
        source: std::io::Error,
    },
    /// Unsupported architecture for MSVC linker auto-detection.
    #[cfg(all(target_os = "windows", target_env = "msvc"))]
    UnsupportedMsvcArchitecture { arch: String },
    /// Failed to find MSVC linker automatically.
    #[cfg(all(target_os = "windows", target_env = "msvc"))]
    MsvcLinkerNotFound { msvc_arch: &'static str },
    /// Linker exited with non-zero status.
    Failed {
        exit_code: String,
        stdout: String,
        stderr: String,
    },
}

impl std::fmt::Display for LinkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LinkError::ExecutionFailed(io_err) => {
                write!(f, "Failed to run linker: {}", io_err)
            }
            LinkError::CurrentExecutablePathResolutionFailed(io_err) => {
                write!(f, "Failed to resolve current executable path: {}", io_err)
            }
            LinkError::CurrentExecutableParentNotFound { executable } => write!(
                f,
                "Current executable path '{}' has no parent directory. This is a compiler bug.",
                executable.display()
            ),
            LinkError::RuntimeLibraryNotFound { executable, path } => write!(
                f,
                "Lak runtime library not found at '{}' (resolved from executable '{}'). Place the 'lak' executable and runtime library in the same directory.",
                path.display(),
                executable.display()
            ),
            LinkError::RuntimeLibraryNotAFile { executable, path } => write!(
                f,
                "Lak runtime library path '{}' is not a regular file (resolved from executable '{}'). Place the 'lak' executable and runtime library in the same directory.",
                path.display(),
                executable.display()
            ),
            LinkError::RuntimeLibraryAccessFailed {
                executable,
                path,
                source,
            } => write!(
                f,
                "Failed to access Lak runtime library path '{}' (resolved from executable '{}'): {}",
                path.display(),
                executable.display(),
                source
            ),
            #[cfg(all(target_os = "windows", target_env = "msvc"))]
            LinkError::UnsupportedMsvcArchitecture { arch } => write!(
                f,
                "Unsupported architecture '{}' for MSVC linker auto-detection. Supported architectures are 'x86_64', 'aarch64', and 'x86'.",
                arch
            ),
            #[cfg(all(target_os = "windows", target_env = "msvc"))]
            LinkError::MsvcLinkerNotFound { msvc_arch } => write!(
                f,
                "Failed to find MSVC linker (link.exe) for target architecture '{}'. Install Visual Studio Build Tools with C++ build tools.",
                msvc_arch
            ),
            LinkError::Failed {
                exit_code,
                stdout,
                stderr,
            } => {
                write!(f, "Linker failed with exit code {}", exit_code)?;
                if !stdout.is_empty() {
                    write!(f, "\n[stdout]\n{}", stdout)?;
                }
                if !stderr.is_empty() {
                    write!(f, "\n[stderr]\n{}", stderr)?;
                }
                Ok(())
            }
        }
    }
}

impl CompileError {
    fn path_not_utf8(path: impl Into<PathBuf>, context: &'static str) -> Self {
        CompileError::PathNotUtf8 {
            path: path.into(),
            context,
        }
    }

    fn file_read_error(path: impl Into<String>, source: std::io::Error) -> Self {
        CompileError::FileReadError {
            path: path.into(),
            source,
        }
    }

    fn path_resolution_error(path: impl Into<String>, source: std::io::Error) -> Self {
        CompileError::PathResolutionError {
            path: path.into(),
            source,
        }
    }

    fn temp_dir_creation_error(source: std::io::Error) -> Self {
        CompileError::TempDirCreationError(source)
    }

    fn executable_run_error(source: std::io::Error) -> Self {
        CompileError::ExecutableRunError(source)
    }

    fn entry_module_not_found(path: impl Into<String>) -> Self {
        CompileError::EntryModuleNotFound { path: path.into() }
    }

    fn filename_error(path: impl Into<String>, reason: &'static str) -> Self {
        CompileError::FilenameError {
            path: path.into(),
            reason,
        }
    }

    fn module_semantic(module: &ResolvedModule, error: SemanticError) -> Self {
        CompileError::ModuleSemantic(Box::new(ModuleSemanticContext {
            error,
            filename: module.path().display().to_string(),
            source: module.source().to_string(),
        }))
    }
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::Resolve(e) => write!(f, "{}", e),
            CompileError::Semantic(e) => write!(f, "{}", e),
            CompileError::ModuleSemantic(ctx) => write!(f, "{}: {}", ctx.filename, ctx.error),
            CompileError::Codegen(e) => write!(f, "{}", e),
            CompileError::Link(e) => write!(f, "{}", e),
            CompileError::PathNotUtf8 { path, context } => {
                write!(
                    f,
                    "{} path '{}' is not valid UTF-8",
                    context,
                    path.display()
                )
            }
            CompileError::FileReadError { path, source } => {
                write!(f, "Failed to read file '{}': {}", path, source)
            }
            CompileError::PathResolutionError { path, source } => {
                write!(f, "Failed to resolve path '{}': {}", path, source)
            }
            CompileError::TempDirCreationError(source) => {
                write!(f, "Failed to create temporary directory: {}", source)
            }
            CompileError::ExecutableRunError(source) => {
                write!(f, "Failed to run executable: {}", source)
            }
            CompileError::EntryModuleNotFound { path } => {
                write!(
                    f,
                    "Entry module '{}' not found after resolution. This is a compiler bug.",
                    path
                )
            }
            CompileError::FilenameError { path, reason } => {
                write!(f, "{}: {}", reason, path)
            }
        }
    }
}

/// Context needed for compiling a source file.
#[derive(Clone)]
struct CompileContext {
    filename: String,
    source: String,
}

impl CompileContext {
    fn new(filename: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            filename: filename.into(),
            source: source.into(),
        }
    }

    /// Combines this context with an error to create a reportable error.
    fn with_error(self, error: CompileError) -> CompileErrorWithContext {
        CompileErrorWithContext {
            context: self,
            error,
        }
    }
}

/// A compilation error with the context needed for reporting.
struct CompileErrorWithContext {
    context: CompileContext,
    error: CompileError,
}

impl CompileErrorWithContext {
    /// Reports this error using ariadne for beautiful error messages.
    fn report(&self) {
        report_error(&self.context.filename, &self.context.source, &self.error);
    }
}

/// Reports a semantic error using ariadne with proper source highlighting.
///
/// This is shared between entry module semantic errors and imported module semantic errors.
fn report_semantic_error(filename: &str, source: &str, e: &SemanticError) {
    if let Some(span) = e.span() {
        let mut report = Report::build(ReportKind::Error, (filename, span.start..span.end))
            .with_config(Config::default().with_index_type(IndexType::Byte))
            .with_message(e.short_message())
            .with_label(
                Label::new((filename, span.start..span.end))
                    .with_message(e.message())
                    .with_color(Color::Red),
            );

        if let Some(help) = e.help() {
            report = report.with_help(help);
        }

        if let Err(report_err) = report.finish().eprint((filename, Source::from(source))) {
            eprintln!("Error: {} (at {}:{})", e.message(), span.line, span.column);
            if let Some(help) = e.help() {
                eprintln!("Help: {}", help);
            }
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
        let help_msg: Option<&str> = if is_missing_main {
            Some("add a main function: fn main() -> void { ... }")
        } else {
            e.help()
        };

        let mut report = Report::build(ReportKind::Error, (filename, span_range.clone()))
            .with_config(Config::default().with_index_type(IndexType::Byte))
            .with_message(e.short_message())
            .with_label(
                Label::new((filename, span_range))
                    .with_message(label_msg)
                    .with_color(Color::Red),
            );

        if let Some(help) = help_msg {
            report = report.with_help(help);
        }

        if let Err(report_err) = report.finish().eprint((filename, Source::from(source))) {
            eprintln!("Error in {}: {}", filename, e.message());
            if let Some(help) = help_msg {
                eprintln!("Help: {}", help);
            }
            eprintln!("(Failed to display detailed error report: {})", report_err);
        }
    }
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
fn report_error(filename: &str, source: &str, error: &CompileError) {
    match error {
        CompileError::Resolve(e) => {
            // If the error carries its own source context (e.g., lex/parse error in imported module),
            // use that for rendering instead of the entry module's context.
            let (report_filename, report_source) = if let (Some(src_file), Some(src_content)) =
                (e.source_filename(), e.source_content())
            {
                (src_file, src_content)
            } else {
                (filename, source)
            };

            if let Some(span) = e.span() {
                let mut report =
                    Report::build(ReportKind::Error, (report_filename, span.start..span.end))
                        .with_config(Config::default().with_index_type(IndexType::Byte))
                        .with_message(e.short_message())
                        .with_label(
                            Label::new((report_filename, span.start..span.end))
                                .with_message(e.message())
                                .with_color(Color::Red),
                        );

                if let Some(help) = e.help() {
                    report = report.with_help(help);
                }

                if let Err(report_err) = report
                    .finish()
                    .eprint((report_filename, Source::from(report_source)))
                {
                    eprintln!(
                        "Error: {}: {} (at {}:{})",
                        e.short_message(),
                        e.message(),
                        span.line,
                        span.column
                    );
                    if let Some(help) = e.help() {
                        eprintln!("Help: {}", help);
                    }
                    eprintln!("(Failed to display detailed error report: {})", report_err);
                }
            } else {
                // No span available - use plain error output (resolver errors without spans
                // are typically I/O errors unrelated to source locations)
                eprintln!("Error: {}", e.message());
                if let Some(help) = e.help() {
                    eprintln!("Help: {}", help);
                }
            }
        }
        CompileError::Semantic(e) => {
            report_semantic_error(filename, source, e);
        }
        CompileError::ModuleSemantic(ctx) => {
            let ModuleSemanticContext {
                error: e,
                filename: module_file,
                source: module_source,
            } = ctx.as_ref();
            report_semantic_error(module_file, module_source, e);
        }
        CompileError::Codegen(e) => {
            // Codegen errors (InternalError, TargetError, InvalidModulePath)
            // typically don't have source locations
            if let Some(span) = e.span() {
                if let Err(report_err) =
                    Report::build(ReportKind::Error, (filename, span.start..span.end))
                        .with_config(Config::default().with_index_type(IndexType::Byte))
                        .with_message(e.short_message())
                        .with_label(
                            Label::new((filename, span.start..span.end))
                                .with_message(e.message())
                                .with_color(Color::Red),
                        )
                        .finish()
                        .eprint((filename, Source::from(source)))
                {
                    // Fallback to basic error output if report printing fails
                    eprintln!(
                        "Error: {}: {} (at {}:{})",
                        e.short_message(),
                        e.message(),
                        span.line,
                        span.column
                    );
                    eprintln!("(Failed to display detailed error report: {})", report_err);
                }
            } else {
                // Infrastructure errors without source location
                eprintln!("Error in {}: {}", filename, e.message());
            }
        }
        CompileError::Link(e) => {
            eprintln!("Error: {}", e);
        }
        CompileError::PathNotUtf8 { .. }
        | CompileError::FileReadError { .. }
        | CompileError::PathResolutionError { .. }
        | CompileError::TempDirCreationError(_)
        | CompileError::ExecutableRunError(_)
        | CompileError::EntryModuleNotFound { .. }
        | CompileError::FilenameError { .. } => {
            eprintln!("Error: {}", error);
        }
    }
}

/// Returns the runtime static library filename for the current target.
#[cfg(all(target_os = "windows", target_env = "msvc"))]
fn runtime_library_filename() -> &'static str {
    "lak_runtime.lib"
}

/// Returns the runtime static library filename for the current target.
#[cfg(not(all(target_os = "windows", target_env = "msvc")))]
fn runtime_library_filename() -> &'static str {
    "liblak_runtime.a"
}

/// Resolves the runtime static library path next to the running `lak` binary.
fn resolve_runtime_library_path() -> Result<PathBuf, CompileError> {
    let executable = std::env::current_exe()
        .map_err(|e| CompileError::Link(LinkError::CurrentExecutablePathResolutionFailed(e)))?;
    let executable_dir = executable.parent().ok_or_else(|| {
        CompileError::Link(LinkError::CurrentExecutableParentNotFound {
            executable: executable.clone(),
        })
    })?;
    let runtime_path = executable_dir.join(runtime_library_filename());
    // Preflight diagnostics for missing runtime libraries.
    // The final link step is still authoritative, so a TOCTOU race here is acceptable.
    match std::fs::metadata(&runtime_path) {
        Ok(metadata) => {
            if !metadata.is_file() {
                return Err(CompileError::Link(LinkError::RuntimeLibraryNotAFile {
                    executable: executable.clone(),
                    path: runtime_path,
                }));
            }
        }
        Err(io_err) if io_err.kind() == std::io::ErrorKind::NotFound => {
            return Err(CompileError::Link(LinkError::RuntimeLibraryNotFound {
                executable: executable.clone(),
                path: runtime_path,
            }));
        }
        Err(io_err) => {
            return Err(CompileError::Link(LinkError::RuntimeLibraryAccessFailed {
                executable: executable.clone(),
                path: runtime_path,
                source: io_err,
            }));
        }
    }

    Ok(runtime_path)
}

/// Maps Rust architecture identifiers to MSVC architecture identifiers.
#[cfg(all(target_os = "windows", target_env = "msvc"))]
fn msvc_linker_arch() -> Result<&'static str, CompileError> {
    match std::env::consts::ARCH {
        "x86_64" => Ok("x64"),
        "aarch64" => Ok("arm64"),
        "x86" => Ok("x86"),
        unsupported => Err(CompileError::Link(LinkError::UnsupportedMsvcArchitecture {
            arch: unsupported.to_string(),
        })),
    }
}

/// Resolves an MSVC `link.exe` command with its toolchain environment.
#[cfg(all(target_os = "windows", target_env = "msvc"))]
fn resolve_msvc_linker_command() -> Result<Command, CompileError> {
    let arch = msvc_linker_arch()?;
    // `find` returns a command preconfigured with MSVC environment variables,
    // including LIB, so the linker can resolve system libraries in CI.
    cc::windows_registry::find(arch, "link.exe")
        .ok_or_else(|| CompileError::Link(LinkError::MsvcLinkerNotFound { msvc_arch: arch }))
}

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

/// Links an object file into an executable.
///
/// # Arguments
///
/// * `object_path` - Path to the object file
/// * `output_path` - Path to write the final executable
///
/// # Returns
///
/// * `Ok(())` - Linking succeeded
/// * `Err(CompileError)` - Linking failed
fn link(object_path: &Path, output_path: &Path) -> Result<(), CompileError> {
    let object_str = object_path
        .to_str()
        .ok_or_else(|| CompileError::path_not_utf8(object_path, "Object file"))?;
    let output_str = output_path
        .to_str()
        .ok_or_else(|| CompileError::path_not_utf8(output_path, "Output file"))?;
    let runtime_path = resolve_runtime_library_path()?;
    let runtime_str = runtime_path
        .to_str()
        .ok_or_else(|| CompileError::path_not_utf8(&runtime_path, "Lak runtime library"))?;

    #[cfg(all(target_os = "windows", target_env = "msvc"))]
    let output = {
        let mut cmd = resolve_msvc_linker_command()?;
        cmd.args([
            "/NOLOGO",
            object_str,
            runtime_str,
            &format!("/OUT:{}", output_str),
            "/DEFAULTLIB:msvcrt",
            "/DEFAULTLIB:legacy_stdio_definitions",
            "/DEFAULTLIB:advapi32",
            "/DEFAULTLIB:bcrypt",
            "/DEFAULTLIB:kernel32",
            "/DEFAULTLIB:ntdll",
            "/DEFAULTLIB:userenv",
            "/DEFAULTLIB:ws2_32",
        ])
        .output()
        .map_err(|e| CompileError::Link(LinkError::ExecutionFailed(e)))?
    };

    #[cfg(not(all(target_os = "windows", target_env = "msvc")))]
    let output = Command::new("cc")
        .args([object_str, runtime_str, "-o", output_str])
        .output()
        .map_err(|e| CompileError::Link(LinkError::ExecutionFailed(e)))?;

    if !output.status.success() {
        return Err(CompileError::Link(LinkError::Failed {
            exit_code: format_exit_status(&output.status),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }));
    }

    Ok(())
}

/// Compiles a Lak source file and links it into an executable.
///
/// This is the shared compilation pipeline used by both `build` and `run` commands.
/// This function is pure and does not produce any output - error reporting is the
/// caller's responsibility.
///
/// # Arguments
///
/// * `context` - The compilation context containing filename and source
/// * `object_path` - Path to write the object file
/// * `output_path` - Path to write the final executable
///
/// # Returns
///
/// * `Ok(())` - Compilation and linking succeeded
/// * `Err(CompileError)` - Compilation failed
fn compile_to_executable(
    context: &CompileContext,
    object_path: &Path,
    output_path: &Path,
) -> Result<(), CompileError> {
    // Phase 1: Resolve modules (load and parse all imported files)
    let entry_path = Path::new(&context.filename);
    let canonical_entry = entry_path
        .canonicalize()
        .map_err(|e| CompileError::path_resolution_error(&context.filename, e))?;

    let mut resolver = ModuleResolver::new();
    resolver
        .resolve_from_entry_with_source(&canonical_entry, context.source.clone())
        .map_err(CompileError::Resolve)?;

    let modules = resolver.into_modules();

    // Find entry module
    let entry_module = modules
        .iter()
        .find(|m| m.path() == canonical_entry)
        .ok_or_else(|| {
            CompileError::entry_module_not_found(canonical_entry.display().to_string())
        })?;

    // Phase 2a: Semantic analysis on imported modules (basic validation)
    for module in &modules {
        if module.path() != canonical_entry {
            let mut module_analyzer = SemanticAnalyzer::new();

            // Build module table if the imported module has its own imports
            let module_table = if !module.program().imports.is_empty() {
                Some(
                    lak::semantic::ModuleTable::from_resolved_modules(&modules, module)
                        .map_err(|e| CompileError::module_semantic(module, e))?,
                )
            } else {
                None
            };

            module_analyzer
                .analyze_module(module.program(), module_table)
                .map_err(|e| CompileError::module_semantic(module, e))?;
        }
    }

    // Phase 2b: Semantic analysis on entry module
    // Build module table from all resolved modules for import validation
    let module_table = lak::semantic::ModuleTable::from_resolved_modules(&modules, entry_module)
        .map_err(CompileError::Semantic)?;

    let mut analyzer = SemanticAnalyzer::new();
    analyzer
        .analyze_with_modules(entry_module.program(), module_table)
        .map_err(CompileError::Semantic)?;

    // Phase 3: Code generation
    let llvm_context = Context::create();
    let mut codegen = Codegen::new(&llvm_context, "lak_module");

    if modules.len() == 1 {
        // Single module: use simple compile
        codegen
            .compile(entry_module.program())
            .map_err(CompileError::Codegen)?;
    } else {
        // Multiple modules: use multi-module compile
        codegen
            .compile_modules(&modules, entry_module.path())
            .map_err(CompileError::Codegen)?;
    }

    codegen
        .write_object_file(object_path)
        .map_err(CompileError::Codegen)?;

    // Phase 4: Linking
    link(object_path, output_path)?;

    Ok(())
}

/// Builds a Lak source file into a native executable.
///
/// This function orchestrates the entire compilation pipeline:
///
/// 1. Read the source file
/// 2. Resolve modules (load and parse all imports, detect cycles)
/// 3. Run semantic analysis on imported modules
/// 4. Run semantic analysis on entry module
/// 5. Generate LLVM IR
/// 6. Write to an object file
/// 7. Link with the system linker (`cc` on Unix, MSVC `link.exe` on Windows) and Lak runtime
/// 8. Clean up temporary files
///
/// # Arguments
///
/// * `file` - Path to the Lak source file
/// * `output` - Optional path for the output executable. If `None`, uses input file stem.
///
/// # Returns
///
/// * `Ok(())` - Compilation succeeded, executable written to disk
/// * `Err(CompileErrorWithContext)` - Compilation failed
///
/// # Output Files
///
/// Given an input file `example.lak`:
/// - Without `-o`: produces `example` executable
/// - With `-o myapp`: produces `myapp` executable
/// - Temporary object file in an isolated temp directory (auto-cleaned)
fn build(file: &str, output: Option<&str>) -> Result<(), Box<CompileErrorWithContext>> {
    let source = std::fs::read_to_string(file).map_err(|e| {
        Box::new(CompileContext::new(file, "").with_error(CompileError::file_read_error(file, e)))
    })?;

    let context = CompileContext::new(file, source);

    let source_path = Path::new(file);
    let stem = source_path
        .file_stem()
        .ok_or_else(|| {
            Box::new(context.clone().with_error(CompileError::filename_error(
                file,
                "Cannot determine filename from path",
            )))
        })?
        .to_str()
        .ok_or_else(|| {
            Box::new(context.clone().with_error(CompileError::filename_error(
                file,
                "Filename contains invalid UTF-8",
            )))
        })?;

    let temp_dir = TempDir::new().map_err(|e| {
        Box::new(
            context
                .clone()
                .with_error(CompileError::temp_dir_creation_error(e)),
        )
    })?;
    let object_path = temp_dir.path().join(format!("{}.o", stem));
    let output_path = match output {
        Some(path) => PathBuf::from(path),
        None => PathBuf::from(format!("{}{}", stem, std::env::consts::EXE_SUFFIX)),
    };

    compile_to_executable(&context, &object_path, &output_path)
        .map_err(|e| Box::new(context.with_error(e)))?;

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
/// 5. Cleans up temporary files automatically
///
/// # Arguments
///
/// * `file` - Path to the Lak source file
///
/// # Returns
///
/// * `Ok(i32)` - The exit code of the executed program
/// * `Err(CompileErrorWithContext)` - Compilation or execution failed
fn run(file: &str) -> Result<i32, Box<CompileErrorWithContext>> {
    let source = std::fs::read_to_string(file).map_err(|e| {
        Box::new(CompileContext::new(file, "").with_error(CompileError::file_read_error(file, e)))
    })?;

    let context = CompileContext::new(file, source);

    // Create temporary directory
    let temp_dir = TempDir::new().map_err(|e| {
        Box::new(
            context
                .clone()
                .with_error(CompileError::temp_dir_creation_error(e)),
        )
    })?;

    let object_path = temp_dir.path().join("program.o");
    let executable_path = temp_dir
        .path()
        .join(format!("program{}", std::env::consts::EXE_SUFFIX));

    compile_to_executable(&context, &object_path, &executable_path)
        .map_err(|e| Box::new(context.clone().with_error(e)))?;

    // Run the executable
    let exec_str = executable_path.to_str().ok_or_else(|| {
        Box::new(
            context
                .clone()
                .with_error(CompileError::path_not_utf8(&executable_path, "Executable")),
        )
    })?;

    let status = Command::new(exec_str)
        .status()
        .map_err(|e| Box::new(context.with_error(CompileError::executable_run_error(e))))?;

    let exit_code = get_exit_code_with_signal(&status);

    // temp_dir is automatically cleaned up when dropped
    drop(temp_dir);

    Ok(exit_code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_entry_module_not_found() {
        let err = CompileError::entry_module_not_found("/tmp/main.lak");
        assert_eq!(
            err.to_string(),
            "Entry module '/tmp/main.lak' not found after resolution. This is a compiler bug."
        );
    }

    #[test]
    fn test_display_file_read_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = CompileError::file_read_error("test.lak", io_err);
        assert_eq!(
            err.to_string(),
            "Failed to read file 'test.lak': file not found"
        );
    }

    #[test]
    fn test_display_path_resolution_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "no such file");
        let err = CompileError::path_resolution_error("test.lak", io_err);
        assert_eq!(
            err.to_string(),
            "Failed to resolve path 'test.lak': no such file"
        );
    }

    #[test]
    fn test_display_temp_dir_creation_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "permission denied");
        let err = CompileError::temp_dir_creation_error(io_err);
        assert_eq!(
            err.to_string(),
            "Failed to create temporary directory: permission denied"
        );
    }

    #[test]
    fn test_display_executable_run_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
        let err = CompileError::executable_run_error(io_err);
        assert_eq!(err.to_string(), "Failed to run executable: not found");
    }

    #[test]
    fn test_display_filename_error() {
        let err =
            CompileError::filename_error("/some/path.lak", "Cannot determine filename from path");
        assert_eq!(
            err.to_string(),
            "Cannot determine filename from path: /some/path.lak"
        );
    }

    #[test]
    fn test_display_link_error_execution_failed() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "cc not found");
        let err = LinkError::ExecutionFailed(io_err);
        assert_eq!(err.to_string(), "Failed to run linker: cc not found");
    }

    #[test]
    fn test_display_link_error_current_executable_path_resolution_failed() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "missing executable");
        let err = LinkError::CurrentExecutablePathResolutionFailed(io_err);
        assert_eq!(
            err.to_string(),
            "Failed to resolve current executable path: missing executable"
        );
    }

    #[test]
    fn test_display_link_error_current_executable_parent_not_found() {
        let err = LinkError::CurrentExecutableParentNotFound {
            executable: PathBuf::from("lak"),
        };
        assert_eq!(
            err.to_string(),
            "Current executable path 'lak' has no parent directory. This is a compiler bug."
        );
    }

    #[test]
    fn test_display_link_error_runtime_library_not_found() {
        let err = LinkError::RuntimeLibraryNotFound {
            executable: PathBuf::from("/tmp/lak"),
            path: PathBuf::from("/tmp/liblak_runtime.a"),
        };
        assert_eq!(
            err.to_string(),
            "Lak runtime library not found at '/tmp/liblak_runtime.a' (resolved from executable '/tmp/lak'). Place the 'lak' executable and runtime library in the same directory."
        );
    }

    #[test]
    fn test_display_link_error_runtime_library_not_a_file() {
        let err = LinkError::RuntimeLibraryNotAFile {
            executable: PathBuf::from("/tmp/lak"),
            path: PathBuf::from("/tmp/liblak_runtime.a"),
        };
        assert_eq!(
            err.to_string(),
            "Lak runtime library path '/tmp/liblak_runtime.a' is not a regular file (resolved from executable '/tmp/lak'). Place the 'lak' executable and runtime library in the same directory."
        );
    }

    #[test]
    fn test_display_link_error_runtime_library_access_failed() {
        let err = LinkError::RuntimeLibraryAccessFailed {
            executable: PathBuf::from("/tmp/lak"),
            path: PathBuf::from("/tmp/liblak_runtime.a"),
            source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "permission denied"),
        };
        assert_eq!(
            err.to_string(),
            "Failed to access Lak runtime library path '/tmp/liblak_runtime.a' (resolved from executable '/tmp/lak'): permission denied"
        );
    }

    #[cfg(all(target_os = "windows", target_env = "msvc"))]
    #[test]
    fn test_display_link_error_unsupported_msvc_architecture() {
        let err = LinkError::UnsupportedMsvcArchitecture {
            arch: "riscv64".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Unsupported architecture 'riscv64' for MSVC linker auto-detection. Supported architectures are 'x86_64', 'aarch64', and 'x86'."
        );
    }

    #[cfg(all(target_os = "windows", target_env = "msvc"))]
    #[test]
    fn test_display_link_error_msvc_linker_not_found() {
        let err = LinkError::MsvcLinkerNotFound { msvc_arch: "x64" };
        assert_eq!(
            err.to_string(),
            "Failed to find MSVC linker (link.exe) for target architecture 'x64'. Install Visual Studio Build Tools with C++ build tools."
        );
    }

    #[test]
    fn test_display_link_error_failed_empty_output() {
        let err = LinkError::Failed {
            exit_code: "1".to_string(),
            stdout: "".to_string(),
            stderr: "".to_string(),
        };
        assert_eq!(err.to_string(), "Linker failed with exit code 1");
    }

    #[test]
    fn test_display_link_error_failed_with_output() {
        let err = LinkError::Failed {
            exit_code: "1".to_string(),
            stdout: "some output".to_string(),
            stderr: "some error".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Linker failed with exit code 1\n[stdout]\nsome output\n[stderr]\nsome error"
        );
    }
}
