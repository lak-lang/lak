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
    // Note: Cargo converts package name "lak-runtime" to "liblak_runtime.a"
    let runtime_lib = workspace_root
        .join("target")
        .join(&profile)
        .join("liblak_runtime.a");

    // Export the runtime library path as a compile-time environment variable
    println!("cargo:rustc-env=LAK_RUNTIME_PATH={}", runtime_lib.display());

    // Rebuild if the runtime library changes
    println!("cargo::rerun-if-changed=../runtime/src/lib.rs");
    println!("cargo::rerun-if-changed=../runtime/Cargo.toml");
}
