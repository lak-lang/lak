use std::path::Path;
use std::process::ExitStatus;

#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;

use lak::linker::{
    LinkerSetupError, create_linker_command, resolve_runtime_library_path_from_current_exe,
};

use super::{CompileError, LinkError};

fn map_linker_setup_error(error: LinkerSetupError) -> CompileError {
    match error {
        LinkerSetupError::CurrentExecutablePathResolutionFailed(source) => {
            CompileError::Link(LinkError::CurrentExecutablePathResolutionFailed(source))
        }
        LinkerSetupError::ExecutablePathParentNotFound { executable } => {
            CompileError::Link(LinkError::CurrentExecutableParentNotFound { executable })
        }
        LinkerSetupError::RuntimeLibraryNotFound { executable, path } => {
            CompileError::Link(LinkError::RuntimeLibraryNotFound { executable, path })
        }
        LinkerSetupError::RuntimeLibraryNotAFile { executable, path } => {
            CompileError::Link(LinkError::RuntimeLibraryNotAFile { executable, path })
        }
        LinkerSetupError::RuntimeLibraryAccessFailed {
            executable,
            path,
            source,
        } => CompileError::Link(LinkError::RuntimeLibraryAccessFailed {
            executable,
            path,
            source,
        }),
        #[cfg(all(target_os = "windows", target_env = "msvc"))]
        LinkerSetupError::UnsupportedMsvcArchitecture { arch } => {
            CompileError::Link(LinkError::UnsupportedMsvcArchitecture { arch })
        }
        #[cfg(all(target_os = "windows", target_env = "msvc"))]
        LinkerSetupError::MsvcLinkerNotFound { msvc_arch } => {
            CompileError::Link(LinkError::MsvcLinkerNotFound { msvc_arch })
        }
    }
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
    let runtime_path =
        resolve_runtime_library_path_from_current_exe().map_err(map_linker_setup_error)?;
    let runtime_str = runtime_path
        .to_str()
        .ok_or_else(|| CompileError::path_not_utf8(&runtime_path, "Lak runtime library"))?;

    let output = create_linker_command(object_str, runtime_str, output_str)
        .map_err(map_linker_setup_error)?
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
