//! End-to-end tests for the `lak build` command.
//!
//! These tests verify that the `build` command correctly compiles Lak programs
//! into executables and handles various error conditions.

mod common;

use common::lak_binary;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

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
    assert!(temp.path().join("test").exists());
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
    assert!(!temp.path().join("test").exists());
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
    // Verify short_message in report title
    assert!(stderr.contains("\x1b[31mError:\x1b[0m Unterminated string"));
    // Verify detailed message in label
    assert!(stderr.contains("Unterminated string literal"));
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
    // Verify short_message in report title
    assert!(stderr.contains("\x1b[31mError:\x1b[0m Unexpected token"));
    // Verify detailed message in label
    assert!(stderr.contains("Expected ')', found '->'"));
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
    let exec_path = temp.path().join("hello");
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

    assert!(output.status.success());
    assert!(temp.path().join("hello world").exists());
}
