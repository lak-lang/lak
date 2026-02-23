use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;

use super::{CompileError, LinkError};

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
    cc::windows_registry::find(arch, "link.exe").ok_or(CompileError::Link(
        LinkError::MsvcLinkerNotFound { msvc_arch: arch },
    ))
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
pub(super) fn link(object_path: &Path, output_path: &Path) -> Result<(), CompileError> {
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
