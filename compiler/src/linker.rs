use std::path::{Path, PathBuf};
use std::process::Command;

/// Linker setup and runtime library resolution errors shared by production and tests.
#[derive(Debug)]
pub enum LinkerSetupError {
    /// Failed to resolve the absolute path of the current executable.
    CurrentExecutablePathResolutionFailed(std::io::Error),
    /// Executable path has no parent directory.
    ExecutablePathParentNotFound { executable: PathBuf },
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
}

impl std::fmt::Display for LinkerSetupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LinkerSetupError::CurrentExecutablePathResolutionFailed(io_err) => {
                write!(f, "Failed to resolve current executable path: {}", io_err)
            }
            LinkerSetupError::ExecutablePathParentNotFound { executable } => write!(
                f,
                "Current executable path '{}' has no parent directory. This is a compiler bug.",
                executable.display()
            ),
            LinkerSetupError::RuntimeLibraryNotFound { executable, path } => write!(
                f,
                "Lak runtime library not found at '{}' (resolved from executable '{}'). Place the 'lak' executable and runtime library in the same directory.",
                path.display(),
                executable.display()
            ),
            LinkerSetupError::RuntimeLibraryNotAFile { executable, path } => write!(
                f,
                "Lak runtime library path '{}' is not a regular file (resolved from executable '{}'). Place the 'lak' executable and runtime library in the same directory.",
                path.display(),
                executable.display()
            ),
            LinkerSetupError::RuntimeLibraryAccessFailed {
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
            LinkerSetupError::UnsupportedMsvcArchitecture { arch } => write!(
                f,
                "Unsupported architecture '{}' for MSVC linker auto-detection. Supported architectures are 'x86_64', 'aarch64', and 'x86'.",
                arch
            ),
            #[cfg(all(target_os = "windows", target_env = "msvc"))]
            LinkerSetupError::MsvcLinkerNotFound { msvc_arch } => write!(
                f,
                "Failed to find MSVC linker (link.exe) for target architecture '{}'. Install Visual Studio Build Tools with C++ build tools.",
                msvc_arch
            ),
        }
    }
}

impl std::error::Error for LinkerSetupError {}

/// Returns the runtime static library filename for the current target.
#[cfg(all(target_os = "windows", target_env = "msvc"))]
pub fn runtime_library_filename() -> &'static str {
    "lak_runtime.lib"
}

/// Returns the runtime static library filename for the current target.
#[cfg(not(all(target_os = "windows", target_env = "msvc")))]
pub fn runtime_library_filename() -> &'static str {
    "liblak_runtime.a"
}

/// Returns the runtime library path expected next to the given executable path.
pub fn runtime_library_path_for_binary(
    executable_path: &Path,
) -> Result<PathBuf, LinkerSetupError> {
    let executable_dir =
        executable_path
            .parent()
            .ok_or_else(|| LinkerSetupError::ExecutablePathParentNotFound {
                executable: executable_path.to_path_buf(),
            })?;
    Ok(executable_dir.join(runtime_library_filename()))
}

/// Resolves the runtime library path for the given executable and validates
/// that the path exists and points to a regular file.
pub fn resolve_runtime_library_path_for_binary(
    executable_path: &Path,
) -> Result<PathBuf, LinkerSetupError> {
    let runtime_path = runtime_library_path_for_binary(executable_path)?;
    match std::fs::metadata(&runtime_path) {
        Ok(metadata) => {
            if !metadata.is_file() {
                return Err(LinkerSetupError::RuntimeLibraryNotAFile {
                    executable: executable_path.to_path_buf(),
                    path: runtime_path,
                });
            }
        }
        Err(io_err) if io_err.kind() == std::io::ErrorKind::NotFound => {
            return Err(LinkerSetupError::RuntimeLibraryNotFound {
                executable: executable_path.to_path_buf(),
                path: runtime_path,
            });
        }
        Err(io_err) => {
            return Err(LinkerSetupError::RuntimeLibraryAccessFailed {
                executable: executable_path.to_path_buf(),
                path: runtime_path,
                source: io_err,
            });
        }
    }

    Ok(runtime_path)
}

/// Resolves the runtime static library path next to the running `lak` binary.
pub fn resolve_runtime_library_path_from_current_exe() -> Result<PathBuf, LinkerSetupError> {
    let executable =
        std::env::current_exe().map_err(LinkerSetupError::CurrentExecutablePathResolutionFailed)?;
    resolve_runtime_library_path_for_binary(&executable)
}

/// Maps Rust architecture identifiers to MSVC architecture identifiers.
#[cfg(all(target_os = "windows", target_env = "msvc"))]
fn msvc_linker_arch() -> Result<&'static str, LinkerSetupError> {
    match std::env::consts::ARCH {
        "x86_64" => Ok("x64"),
        "aarch64" => Ok("arm64"),
        "x86" => Ok("x86"),
        unsupported => Err(LinkerSetupError::UnsupportedMsvcArchitecture {
            arch: unsupported.to_string(),
        }),
    }
}

/// Builds a platform-appropriate linker command with standard Lak arguments.
pub fn create_linker_command(
    object_path: &str,
    runtime_path: &str,
    output_path: &str,
) -> Result<Command, LinkerSetupError> {
    #[cfg(all(target_os = "windows", target_env = "msvc"))]
    {
        let arch = msvc_linker_arch()?;
        let mut cmd = cc::windows_registry::find(arch, "link.exe")
            .ok_or(LinkerSetupError::MsvcLinkerNotFound { msvc_arch: arch })?;
        cmd.args([
            "/NOLOGO",
            object_path,
            runtime_path,
            &format!("/OUT:{}", output_path),
            "/DEFAULTLIB:msvcrt",
            "/DEFAULTLIB:legacy_stdio_definitions",
            "/DEFAULTLIB:advapi32",
            "/DEFAULTLIB:bcrypt",
            "/DEFAULTLIB:kernel32",
            "/DEFAULTLIB:ntdll",
            "/DEFAULTLIB:userenv",
            "/DEFAULTLIB:ws2_32",
        ]);
        Ok(cmd)
    }

    #[cfg(not(all(target_os = "windows", target_env = "msvc")))]
    {
        let mut cmd = Command::new("cc");
        cmd.args([object_path, runtime_path, "-o", output_path]);
        Ok(cmd)
    }
}
