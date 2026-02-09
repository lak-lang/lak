//! Build script for the Lak compiler.
//!
//! This script sets up the path to the Lak runtime library so that
//! the compiler can find it at runtime when linking generated programs.

use std::env;
use std::path::PathBuf;

fn main() {
    // Get the workspace root directory
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR not set. This build script must be run by Cargo.");
    let workspace_root = PathBuf::from(&manifest_dir)
        .parent()
        .expect("Failed to find workspace root")
        .to_path_buf();

    // Determine the build profile (debug or release)
    let profile =
        env::var("PROFILE").expect("PROFILE not set. This build script must be run by Cargo.");

    // Construct the path to the runtime static library.
    // Cargo converts package name "lak-runtime" to platform-specific names:
    // - Unix and Windows GNU (MinGW): "liblak_runtime.a"
    // - Windows MSVC: "lak_runtime.lib"
    let target_os = env::var("CARGO_CFG_TARGET_OS")
        .expect("CARGO_CFG_TARGET_OS not set. This build script must be run by Cargo.");
    let target_env = env::var("CARGO_CFG_TARGET_ENV")
        .expect("CARGO_CFG_TARGET_ENV not set. This build script must be run by Cargo.");
    let runtime_lib_name = if target_os == "windows" && target_env == "msvc" {
        "lak_runtime.lib"
    } else {
        "liblak_runtime.a"
    };
    let runtime_lib = workspace_root
        .join("target")
        .join(&profile)
        .join(runtime_lib_name);

    // Export the runtime library path as a compile-time environment variable
    println!("cargo:rustc-env=LAK_RUNTIME_PATH={}", runtime_lib.display());

    // On Windows MSVC, detect the MSVC linker (link.exe) path at build time.
    // This is necessary because the system PATH may contain a GNU coreutils
    // link.exe (from Git for Windows) that shadows the MSVC linker.
    // The cc crate uses COM interfaces, vswhere.exe, and registry lookups
    // to reliably find the MSVC toolchain.
    if target_os == "windows" && target_env == "msvc" {
        let compiler = cc::Build::new().get_compiler();
        let tools_dir = compiler
            .path()
            .parent()
            .expect("Failed to determine MSVC tools directory from C compiler path");
        let link_exe = tools_dir.join("link.exe");
        println!("cargo:rustc-env=LAK_MSVC_LINKER={}", link_exe.display());

        // Capture MSVC environment variables needed by link.exe at runtime.
        // The LIB variable tells link.exe where to find system libraries
        // (msvcrt.lib, legacy_stdio_definitions.lib, etc.). Without it,
        // linking fails in CI environments where vcvarsall.bat has not been run.
        for (key, value) in compiler.env() {
            if key == "LIB" {
                println!(
                    "cargo:rustc-env=LAK_MSVC_LIB={}",
                    value
                        .to_str()
                        .expect("MSVC LIB environment variable is not valid UTF-8")
                );
            }
        }
    }

    // Rebuild if the runtime library changes
    println!("cargo::rerun-if-changed=../runtime/src/lib.rs");
    println!("cargo::rerun-if-changed=../runtime/Cargo.toml");
}
