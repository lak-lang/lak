//! End-to-end tests for the `lak build` command.
//!
//! These tests verify that the `build` command correctly compiles Lak programs
//! into executables and handles various error conditions.

mod common;

use common::{copy_lak_binary_to, executable_name, lak_binary, runtime_library_path_for_binary};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;

fn runtime_library_path_for(binary_path: &std::path::Path) -> PathBuf {
    runtime_library_path_for_binary(binary_path)
        .expect("lak binary path should have a parent directory")
}

fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() {
        return true;
    }
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

#[test]
fn test_build_basic() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("test.lak");
    fs::write(&source_path, "fn main() -> void {}").unwrap();

    let output = Command::new(lak_binary())
        .current_dir(temp.path())
        .args(["build", "test.lak"])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(temp.path().join(executable_name("test")).exists());
}

#[test]
fn test_build_custom_output() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("test.lak");
    fs::write(&source_path, "fn main() -> void {}").unwrap();

    let output = Command::new(lak_binary())
        .current_dir(temp.path())
        .args(["build", "test.lak", "-o", "custom"])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(temp.path().join("custom").exists());
    // Original name should not exist
    assert!(!temp.path().join(executable_name("test")).exists());
}

#[test]
fn test_build_nonexistent_file() {
    let temp = tempdir().unwrap();

    let output = Command::new(lak_binary())
        .current_dir(temp.path())
        .args(["build", "nonexistent.lak"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Error: Failed to read file 'nonexistent.lak'"));
}

#[test]
fn test_build_lexer_error() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("lex_error.lak");

    // Unterminated string literal
    fs::write(
        &source_path,
        r#"fn main() -> void { println("unterminated }"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .current_dir(temp.path())
        .args(["build", "lex_error.lak"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Verify short_message in report title (now goes through resolver)
    assert!(
        stderr.contains("\x1b[31mError:\x1b[0m Lexical error in module"),
        "Expected 'Lexical error in module' in stderr: {}",
        stderr
    );
    // Verify detailed message in label
    assert!(
        stderr.contains("Unterminated string literal"),
        "Expected 'Unterminated string literal' in stderr: {}",
        stderr
    );
}

#[test]
fn test_build_parser_error() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("parse_error.lak");

    // Invalid syntax: missing right paren
    fs::write(&source_path, "fn main( -> void {}").unwrap();

    let output = Command::new(lak_binary())
        .current_dir(temp.path())
        .args(["build", "parse_error.lak"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Verify short_message in report title (now goes through resolver)
    assert!(
        stderr.contains("\x1b[31mError:\x1b[0m Parse error in module"),
        "Expected 'Parse error in module' in stderr: {}",
        stderr
    );
    // Verify detailed message in label
    assert!(
        stderr.contains("Expected parameter name or ')', found '->'"),
        "Expected \"Expected parameter name or ')', found '->'\" in stderr: {}",
        stderr
    );
}

#[test]
fn test_build_semantic_error_missing_main() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("no_main.lak");

    // Missing main function
    fs::write(&source_path, "fn helper() -> void {}").unwrap();

    let output = Command::new(lak_binary())
        .current_dir(temp.path())
        .args(["build", "no_main.lak"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Verify short_message in report title
    assert!(stderr.contains("\x1b[31mError:\x1b[0m Missing main function"));
    // Verify detailed message in label
    assert!(stderr.contains("main function not found"));
    assert!(
        stderr
            .contains("\x1b[38;5;115mHelp\x1b[0m: add a main function: fn main() -> void { ... }")
    );
}

#[test]
fn test_build_cleans_up_object_file() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("cleanup.lak");
    fs::write(&source_path, "fn main() -> void {}").unwrap();

    let output = Command::new(lak_binary())
        .current_dir(temp.path())
        .args(["build", "cleanup.lak"])
        .output()
        .unwrap();

    assert!(output.status.success());

    // Verify no .o files left
    let entries: Vec<_> = fs::read_dir(temp.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "o"))
        .collect();

    assert!(entries.is_empty(), "Object file should be cleaned up");
}

#[test]
fn test_build_does_not_delete_existing_cwd_object_file() {
    let build_cwd = tempdir().unwrap();
    let source_dir = tempdir().unwrap();
    let stem = "collision_target";

    let protected_object = build_cwd.path().join(format!("{stem}.o"));
    fs::write(&protected_object, "KEEP_ME").unwrap();

    let source_path = source_dir.path().join(format!("{stem}.lak"));
    fs::write(&source_path, "fn main() -> void {}").unwrap();

    let output = Command::new(lak_binary())
        .current_dir(build_cwd.path())
        .arg("build")
        .arg(&source_path)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "build should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        protected_object.exists(),
        "existing object file in CWD must not be removed"
    );
    assert_eq!(
        fs::read_to_string(&protected_object).unwrap(),
        "KEEP_ME",
        "existing object file in CWD must not be overwritten"
    );
    assert!(build_cwd.path().join(executable_name(stem)).exists());
}

#[test]
fn test_build_produces_executable() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("hello.lak");
    fs::write(&source_path, r#"fn main() -> void { println("Hello!") }"#).unwrap();

    let build_output = Command::new(lak_binary())
        .current_dir(temp.path())
        .args(["build", "hello.lak"])
        .output()
        .unwrap();

    assert!(build_output.status.success());

    // Run the built executable
    let exec_path = temp.path().join(executable_name("hello"));
    let run_output = Command::new(&exec_path).output().unwrap();

    assert!(run_output.status.success());
    assert_eq!(String::from_utf8_lossy(&run_output.stdout), "Hello!\n");
}

#[test]
fn test_build_path_with_spaces() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("hello world.lak");
    fs::write(&source_path, r#"fn main() -> void { println("ok") }"#).unwrap();

    let output = Command::new(lak_binary())
        .current_dir(temp.path())
        .args(["build", "hello world.lak"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Module names must be valid identifiers"),
        "Expected 'Module names must be valid identifiers' error for filename with spaces, got: {}",
        stderr
    );
}

#[test]
fn test_build_negation_overflow_span_includes_minus() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("neg_overflow.lak");

    fs::write(
        &source_path,
        r#"fn main() -> void {
    println(-9223372036854775809)
}"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .current_dir(temp.path())
        .args(["build", "neg_overflow.lak"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Verify short_message in report title
    assert!(
        stderr.contains("\x1b[31mError:\x1b[0m Parse error in module"),
        "Expected 'Parse error in module' in stderr: {}",
        stderr
    );
    // Verify detailed message in label
    assert!(
        stderr.contains("is out of range for i64"),
        "Expected negation overflow message in stderr: {}",
        stderr
    );
    // Verify the span starts at the `-` sign (column 13), not the digit (column 14).
    // ariadne renders the location header as `neg_overflow.lak:2:13` when the span
    // includes the minus sign.
    assert!(
        stderr.contains("neg_overflow.lak:2:13"),
        "Expected span to start at column 13 (minus sign), not column 14: {}",
        stderr
    );
}

#[test]
fn test_build_requires_runtime_library_next_to_lak_binary() {
    let tools_dir = tempdir().unwrap();
    let source_dir = tempdir().unwrap();
    let copied_lak =
        copy_lak_binary_to(tools_dir.path()).expect("failed to copy lak binary to tools directory");

    fs::write(source_dir.path().join("main.lak"), "fn main() -> void {}").unwrap();

    let output = Command::new(&copied_lak)
        .current_dir(source_dir.path())
        .args(["build", "main.lak"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Lak runtime library not found at"),
        "Expected runtime library missing error, got: {}",
        stderr
    );
    assert!(
        stderr.contains("Place the 'lak' executable and runtime library in the same directory."),
        "Expected placement guidance in error, got: {}",
        stderr
    );
}

#[test]
fn test_build_fails_when_runtime_path_is_not_regular_file() {
    let tools_dir = tempdir().unwrap();
    let source_dir = tempdir().unwrap();
    let copied_lak =
        copy_lak_binary_to(tools_dir.path()).expect("failed to copy lak binary to tools directory");

    let copied_runtime = runtime_library_path_for(&copied_lak);
    fs::create_dir(&copied_runtime).expect("failed to create directory at runtime library path");

    fs::write(source_dir.path().join("main.lak"), "fn main() -> void {}").unwrap();

    let output = Command::new(&copied_lak)
        .current_dir(source_dir.path())
        .args(["build", "main.lak"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Lak runtime library path"),
        "Expected runtime path error, got: {}",
        stderr
    );
    assert!(
        stderr.contains("is not a regular file"),
        "Expected non-regular-file error, got: {}",
        stderr
    );
    assert!(
        stderr.contains("Place the 'lak' executable and runtime library in the same directory."),
        "Expected placement guidance in error, got: {}",
        stderr
    );
}

#[test]
fn test_build_succeeds_with_runtime_library_next_to_lak_binary() {
    let tools_dir = tempdir().unwrap();
    let source_dir = tempdir().unwrap();
    let copied_lak =
        copy_lak_binary_to(tools_dir.path()).expect("failed to copy lak binary to tools directory");

    let original_lak = PathBuf::from(lak_binary());
    let source_runtime = runtime_library_path_for(&original_lak);
    let copied_runtime = runtime_library_path_for(&copied_lak);
    fs::copy(&source_runtime, &copied_runtime).expect("failed to copy runtime library");

    fs::write(source_dir.path().join("main.lak"), "fn main() -> void {}").unwrap();

    let output = Command::new(&copied_lak)
        .current_dir(source_dir.path())
        .args(["build", "main.lak"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "build should succeed when runtime is co-located: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(source_dir.path().join(executable_name("main")).exists());
}

#[test]
fn test_build_binary_does_not_embed_runtime_absolute_path() {
    let lak_path = PathBuf::from(lak_binary());
    let runtime_path = runtime_library_path_for(&lak_path);
    let runtime_path_string = runtime_path.to_string_lossy().to_string();
    let runtime_path_canonical = runtime_path
        .canonicalize()
        .expect("runtime library path should be canonicalizable")
        .to_string_lossy()
        .to_string();

    let bytes = fs::read(&lak_path).unwrap();

    assert!(
        !contains_bytes(&bytes, runtime_path_string.as_bytes()),
        "lak binary unexpectedly embeds runtime absolute path: {}",
        runtime_path_string
    );

    assert!(
        !contains_bytes(&bytes, runtime_path_canonical.as_bytes()),
        "lak binary unexpectedly embeds canonical runtime absolute path: {}",
        runtime_path_canonical
    );
}
