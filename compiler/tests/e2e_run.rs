//! End-to-end tests for the `lak run` command.
//!
//! These tests verify that the `run` command correctly compiles and executes
//! Lak programs, and properly cleans up temporary files.

mod common;

use common::{copy_lak_binary_to, lak_binary, runtime_library_path_for_binary};
use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_run_hello_world() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("hello.lak");

    fs::write(
        &source_path,
        r#"fn main() -> void {
    println("Hello from run!")
}"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .args(["run", source_path.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "Hello from run!\n");
}

#[test]
fn test_run_multiple_println() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("multi.lak");

    fs::write(
        &source_path,
        r#"fn main() -> void {
    println("line 1")
    println("line 2")
    println("line 3")
}"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .args(["run", source_path.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "line 1\nline 2\nline 3\n"
    );
}

#[test]
fn test_run_empty_main() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("empty.lak");

    fs::write(&source_path, "fn main() -> void {}").unwrap();

    let output = Command::new(lak_binary())
        .args(["run", source_path.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "");
}

#[test]
fn test_run_cleanup() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("cleanup_test.lak");

    fs::write(&source_path, "fn main() -> void {}").unwrap();

    let output = Command::new(lak_binary())
        .args(["run", source_path.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(output.status.success());

    // Verify no .o files or executables left in the source directory
    let entries: Vec<_> = fs::read_dir(temp.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();

    // Only the source file should remain
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].file_name().to_str().unwrap(), "cleanup_test.lak");
}

#[test]
fn test_run_nonexistent_file() {
    let output = Command::new(lak_binary())
        .args(["run", "nonexistent.lak"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Error: Failed to read file 'nonexistent.lak'"));
}

#[test]
fn test_run_compile_error_missing_main() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("invalid.lak");

    // Missing main function
    fs::write(&source_path, "fn helper() -> void {}").unwrap();

    let output = Command::new(lak_binary())
        .args(["run", source_path.to_str().unwrap()])
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
fn test_run_with_variables() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("vars.lak");

    // Verify variable values via println output
    fs::write(
        &source_path,
        r#"fn main() -> void {
    let x: i32 = 42
    let y: i64 = 100
    println(x)
    println(y)
}"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .args(["run", source_path.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "42\n100\n");
}

#[test]
fn test_run_exit_code_zero() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("exit_zero.lak");

    fs::write(&source_path, "fn main() -> void {}").unwrap();

    let output = Command::new(lak_binary())
        .args(["run", source_path.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn test_run_lexer_error() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("lex_error.lak");

    // Unterminated string literal
    fs::write(
        &source_path,
        r#"fn main() -> void { println("unterminated }"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .args(["run", source_path.to_str().unwrap()])
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
fn test_run_parser_error() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("parse_error.lak");

    // Invalid syntax: missing right paren
    fs::write(&source_path, "fn main( -> void {}").unwrap();

    let output = Command::new(lak_binary())
        .args(["run", source_path.to_str().unwrap()])
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
fn test_run_path_with_spaces() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("hello world.lak");

    fs::write(&source_path, r#"fn main() -> void { println("ok") }"#).unwrap();

    let output = Command::new(lak_binary())
        .args(["run", source_path.to_str().unwrap()])
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
fn test_run_requires_runtime_library_next_to_lak_binary() {
    let tools_dir = tempdir().unwrap();
    let source_dir = tempdir().unwrap();
    let copied_lak =
        copy_lak_binary_to(tools_dir.path()).expect("failed to copy lak binary to tools directory");
    let source_path = source_dir.path().join("main.lak");
    fs::write(&source_path, "fn main() -> void {}").unwrap();

    let output = Command::new(&copied_lak)
        .args([
            "run",
            source_path
                .to_str()
                .expect("source path should be valid UTF-8"),
        ])
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
fn test_run_succeeds_with_runtime_library_next_to_lak_binary() {
    let tools_dir = tempdir().unwrap();
    let source_dir = tempdir().unwrap();
    let copied_lak =
        copy_lak_binary_to(tools_dir.path()).expect("failed to copy lak binary to tools directory");

    let original_lak = std::path::PathBuf::from(lak_binary());
    let source_runtime = runtime_library_path_for_binary(&original_lak)
        .expect("lak binary path should have a parent directory");
    let copied_runtime = runtime_library_path_for_binary(&copied_lak)
        .expect("copied lak binary path should have a parent directory");
    fs::copy(&source_runtime, &copied_runtime).expect("failed to copy runtime library");

    let source_path = source_dir.path().join("main.lak");
    fs::write(&source_path, r#"fn main() -> void { println("ok") }"#).unwrap();

    let output = Command::new(&copied_lak)
        .args([
            "run",
            source_path
                .to_str()
                .expect("source path should be valid UTF-8"),
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "run should succeed when runtime is co-located: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "ok\n");
}

// ========================================
// Unary minus tests
// ========================================

#[test]
fn test_run_with_negative_values() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("negative.lak");

    fs::write(
        &source_path,
        r#"fn main() -> void {
    let x: i32 = -42
    let y: i64 = -1000
    let z: i32 = --10
    println(x)
    println(y)
    println(z)
}"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .args(["run", source_path.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "-42\n-1000\n10\n");
}

#[test]
fn test_run_unary_minus_in_expressions() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("unary_expr.lak");

    fs::write(
        &source_path,
        r#"fn main() -> void {
    let a: i32 = 10
    let b: i32 = -a + 5
    let c: i32 = -(a * 2)
    println(b)
    println(c)
}"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .args(["run", source_path.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "-5\n-20\n");
}

#[test]
fn test_run_module_file_not_found() {
    let temp = tempdir().unwrap();
    let source_path = temp.path().join("main.lak");

    fs::write(
        &source_path,
        r#"import "./missing"

fn main() -> void {}
"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .current_dir(temp.path())
        .args(["run", source_path.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Module not found"),
        "Expected 'Module not found' error, got: {}",
        stderr
    );
    assert!(
        stderr.contains("Cannot find module './missing'"),
        "Expected error message to mention the module path, got: {}",
        stderr
    );
}

#[test]
fn test_run_circular_import() {
    let temp = tempdir().unwrap();

    let a_path = temp.path().join("a.lak");
    fs::write(
        &a_path,
        r#"import "./b"

pub fn foo() -> void {
    b.bar()
}
"#,
    )
    .unwrap();

    let b_path = temp.path().join("b.lak");
    fs::write(
        &b_path,
        r#"import "./a"

pub fn bar() -> void {
    a.foo()
}
"#,
    )
    .unwrap();

    let main_path = temp.path().join("main.lak");
    fs::write(
        &main_path,
        r#"import "./a"

fn main() -> void {
    a.foo()
}
"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .current_dir(temp.path())
        .args(["run", main_path.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Circular import"),
        "Expected 'Circular import' error, got: {}",
        stderr
    );
}
