use inkwell::context::Context;
use lak::codegen::{Codegen, CodegenError};
use lak::resolver::{ModuleResolver, ResolvedModule, ResolverError};
use lak::semantic::SemanticAnalyzer;
use lak::semantic::SemanticError;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};
use tempfile::TempDir;

#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;

mod link;

/// A compilation error from any phase of the compiler.
///
/// This enum unifies errors from module resolution, semantic analysis,
/// code generation, linking, and I/O to simplify error handling in the build pipeline.
pub(crate) enum CompileError {
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
pub(crate) struct ModuleSemanticContext {
    pub(crate) error: SemanticError,
    pub(crate) filename: String,
    pub(crate) source: String,
}

/// A linker error.
pub(crate) enum LinkError {
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
pub(crate) struct CompileErrorWithContext {
    context: CompileContext,
    error: CompileError,
}

impl CompileErrorWithContext {
    pub(crate) fn filename(&self) -> &str {
        &self.context.filename
    }

    pub(crate) fn source(&self) -> &str {
        &self.context.source
    }

    pub(crate) fn error(&self) -> &CompileError {
        &self.error
    }
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
    link::link(object_path, output_path)?;

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
pub(crate) fn build(file: &str, output: Option<&str>) -> Result<(), Box<CompileErrorWithContext>> {
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
pub(crate) fn run(file: &str) -> Result<i32, Box<CompileErrorWithContext>> {
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
