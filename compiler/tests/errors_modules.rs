//! Error tests for the module and import system.
//!
//! These tests verify that the compiler correctly reports errors for
//! various module-related issues.
//!
//! # Testing Approach
//!
//! These tests use `tempdir()` and invoke the compiler binary as a subprocess.
//! This is necessary because multi-module compilation requires real files on disk
//! (the resolver reads imported modules from the filesystem). The in-process
//! `compile_and_run()` / `compile_error()` helpers from `common/mod.rs` only
//! support single-file compilation, so they cannot test import-related errors.

mod common;

use common::{executable_name, lak_binary};
use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_error_module_not_found() {
    let temp = tempdir().unwrap();

    let main_path = temp.path().join("main.lak");
    fs::write(
        &main_path,
        r#"import "./nonexistent"

fn main() -> void {}
"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .current_dir(temp.path())
        .args(["build", "main.lak"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Verify short_message in report title
    assert!(
        stderr.contains("\x1b[31mError:\x1b[0m Module not found"),
        "Expected 'Module not found' error, got: {}",
        stderr
    );
    // Verify detailed message in label
    assert!(
        stderr.contains("Cannot find module './nonexistent'"),
        "Expected error message to mention the module path, got: {}",
        stderr
    );
    // Verify help text
    assert!(
        stderr.contains(
            "\x1b[38;5;115mHelp\x1b[0m: check that the file exists and the path is correct"
        ),
        "Expected help text about checking file existence, got: {}",
        stderr
    );
}

#[test]
fn test_error_circular_import() {
    let temp = tempdir().unwrap();

    // Create a.lak that imports b
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

    // Create b.lak that imports a (circular)
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

    // Create main.lak that imports a
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
        .args(["build", "main.lak"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Verify short_message in report title
    assert!(
        stderr.contains("\x1b[31mError:\x1b[0m Circular import"),
        "Expected 'Circular import' error, got: {}",
        stderr
    );
    // Verify detailed message in label
    assert!(
        stderr.contains("Circular import detected: a -> b -> a"),
        "Expected cycle details in error, got: {}",
        stderr
    );
    // Verify help text
    assert!(
        stderr.contains("\x1b[38;5;115mHelp\x1b[0m: break the cycle by removing one of the imports or restructuring your modules"),
        "Expected help text about breaking the cycle, got: {}",
        stderr
    );
}

#[test]
fn test_error_undefined_module() {
    let temp = tempdir().unwrap();

    // Create main.lak that calls a module that was never imported
    let main_path = temp.path().join("main.lak");
    fs::write(
        &main_path,
        r#"fn main() -> void {
    nonexistent.foo()
}
"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .current_dir(temp.path())
        .args(["build", "main.lak"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Verify short_message in report title
    assert!(
        stderr.contains("\x1b[31mError:\x1b[0m Undefined module"),
        "Expected 'Undefined module' error, got: {}",
        stderr
    );
    // Verify detailed message in label
    assert!(
        stderr.contains("Module 'nonexistent' is not defined"),
        "Expected error message to mention the module name, got: {}",
        stderr
    );
    // Verify help text
    assert!(
        stderr.contains(
            "\x1b[38;5;115mHelp\x1b[0m: Did you forget to import it? Add: import \"./module_name\""
        ),
        "Expected help text about importing, got: {}",
        stderr
    );
}

#[test]
fn test_error_undefined_module_function() {
    let temp = tempdir().unwrap();

    // Create utils.lak with a public function
    let utils_path = temp.path().join("utils.lak");
    fs::write(
        &utils_path,
        r#"pub fn greet() -> void {
    println("Hello")
}
"#,
    )
    .unwrap();

    // Create main.lak that calls a nonexistent function
    let main_path = temp.path().join("main.lak");
    fs::write(
        &main_path,
        r#"import "./utils"

fn main() -> void {
    utils.nonexistent()
}
"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .current_dir(temp.path())
        .args(["build", "main.lak"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Verify short_message in report title
    assert!(
        stderr.contains("\x1b[31mError:\x1b[0m Undefined module function"),
        "Expected 'Undefined module function' error, got: {}",
        stderr
    );
    // Verify detailed message in label
    assert!(
        stderr.contains("Function 'nonexistent' not found in module 'utils'"),
        "Expected error message to mention the function and module, got: {}",
        stderr
    );
    // Verify help text
    assert!(
        stderr.contains("\x1b[38;5;115mHelp\x1b[0m: Check that the function exists in 'utils' and is marked 'pub'"),
        "Expected help text about pub visibility, got: {}",
        stderr
    );
}

#[test]
fn test_error_private_function_access() {
    let temp = tempdir().unwrap();

    // Create utils.lak with a private function
    let utils_path = temp.path().join("utils.lak");
    fs::write(
        &utils_path,
        r#"fn private_helper() -> void {
    println("Private")
}

pub fn greet() -> void {
    println("Hello")
}
"#,
    )
    .unwrap();

    // Create main.lak that tries to call the private function
    let main_path = temp.path().join("main.lak");
    fs::write(
        &main_path,
        r#"import "./utils"

fn main() -> void {
    utils.private_helper()
}
"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .current_dir(temp.path())
        .args(["build", "main.lak"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Private functions appear as "not found" since they're not in the public exports
    // Verify short_message in report title
    assert!(
        stderr.contains("\x1b[31mError:\x1b[0m Undefined module function"),
        "Expected 'Undefined module function' error, got: {}",
        stderr
    );
    // Verify detailed message in label
    assert!(
        stderr.contains("Function 'private_helper' not found in module 'utils'"),
        "Expected error message to mention the private function, got: {}",
        stderr
    );
    // Verify help text
    assert!(
        stderr.contains("\x1b[38;5;115mHelp\x1b[0m: Check that the function exists in 'utils' and is marked 'pub'"),
        "Expected help text about pub visibility, got: {}",
        stderr
    );
}

#[test]
fn test_error_standard_library_not_supported() {
    let temp = tempdir().unwrap();

    // Create main.lak that tries to import a standard library
    let main_path = temp.path().join("main.lak");
    fs::write(
        &main_path,
        r#"import "math"

fn main() -> void {}
"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .current_dir(temp.path())
        .args(["build", "main.lak"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Verify short_message in report title
    assert!(
        stderr.contains("\x1b[31mError:\x1b[0m Standard library not supported"),
        "Expected 'Standard library not supported' error, got: {}",
        stderr
    );
    // Verify detailed message in label
    assert!(
        stderr.contains("Standard library imports are not yet supported: 'math'"),
        "Expected error message to mention the library name, got: {}",
        stderr
    );
    // Verify help text
    assert!(
        stderr.contains("\x1b[38;5;115mHelp\x1b[0m: use relative paths like './module' instead"),
        "Expected help text about using relative paths, got: {}",
        stderr
    );
}

#[test]
fn test_error_invalid_module_name() {
    let temp = tempdir().unwrap();

    // Create a module with invalid name (starts with number)
    let invalid_path = temp.path().join("123invalid.lak");
    fs::write(
        &invalid_path,
        r#"pub fn foo() -> void {
    println("Hello")
}
"#,
    )
    .unwrap();

    // Create main.lak that imports it
    let main_path = temp.path().join("main.lak");
    fs::write(
        &main_path,
        r#"import "./123invalid"

fn main() -> void {}
"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .current_dir(temp.path())
        .args(["build", "main.lak"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Verify short_message in report title
    assert!(
        stderr.contains("\x1b[31mError:\x1b[0m Invalid module name"),
        "Expected 'Invalid module name' error, got: {}",
        stderr
    );
    // Verify detailed message in label
    assert!(
        stderr.contains("Cannot extract module name from path"),
        "Expected error message to mention module name extraction, got: {}",
        stderr
    );
    // Verify help text
    assert!(
        stderr.contains("\x1b[38;5;115mHelp\x1b[0m: module names must be valid identifiers (start with an ASCII letter or underscore, contain only ASCII letters, digits, and underscores)"),
        "Expected help text about valid identifiers, got: {}",
        stderr
    );
}

#[test]
fn test_error_lex_error_in_imported_module() {
    let temp = tempdir().unwrap();

    // Create utils.lak with a lexer error
    let utils_path = temp.path().join("utils.lak");
    fs::write(
        &utils_path,
        r#"pub fn greet() -> void {
    println("unterminated
}
"#,
    )
    .unwrap();

    // Create main.lak
    let main_path = temp.path().join("main.lak");
    fs::write(
        &main_path,
        r#"import "./utils"

fn main() -> void {
    utils.greet()
}
"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .current_dir(temp.path())
        .args(["build", "main.lak"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Verify short_message in report title
    assert!(
        stderr.contains("\x1b[31mError:\x1b[0m Lexical error in module"),
        "Expected 'Lexical error in module' error, got: {}",
        stderr
    );
    // Verify detailed message in label
    assert!(
        stderr.contains("Unterminated string literal"),
        "Expected error message to mention unterminated string, got: {}",
        stderr
    );
}

#[test]
fn test_error_parse_error_in_imported_module() {
    let temp = tempdir().unwrap();

    // Create utils.lak with a parse error
    let utils_path = temp.path().join("utils.lak");
    fs::write(
        &utils_path,
        r#"pub fn greet( -> void {
    println("Hello")
}
"#,
    )
    .unwrap();

    // Create main.lak
    let main_path = temp.path().join("main.lak");
    fs::write(
        &main_path,
        r#"import "./utils"

fn main() -> void {
    utils.greet()
}
"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .current_dir(temp.path())
        .args(["build", "main.lak"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Verify short_message in report title
    assert!(
        stderr.contains("\x1b[31mError:\x1b[0m Parse error in module"),
        "Expected 'Parse error in module' error, got: {}",
        stderr
    );
    // Verify detailed message in label
    assert!(
        stderr.contains("Expected parameter name or ')', found '->'"),
        "Expected parse error details in label, got: {}",
        stderr
    );
}

#[test]
fn test_error_self_import() {
    // Tests that a module importing itself is detected as circular import
    let temp = tempdir().unwrap();

    // Create recursive.lak that imports itself
    let recursive_path = temp.path().join("recursive.lak");
    fs::write(
        &recursive_path,
        r#"import "./recursive"

pub fn foo() -> void {
    println("test")
}

fn main() -> void {}
"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .current_dir(temp.path())
        .args(["build", "recursive.lak"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Verify short_message in report title
    assert!(
        stderr.contains("\x1b[31mError:\x1b[0m Circular import"),
        "Expected 'Circular import' error for self-import, got: {}",
        stderr
    );
    // Verify detailed message in label
    assert!(
        stderr.contains("Circular import detected: recursive -> recursive"),
        "Expected cycle to show 'Circular import detected: recursive -> recursive', got: {}",
        stderr
    );
    // Verify help text
    assert!(
        stderr.contains("\x1b[38;5;115mHelp\x1b[0m: break the cycle by removing one of the imports or restructuring your modules"),
        "Expected help text about breaking the cycle, got: {}",
        stderr
    );
}

#[test]
fn test_error_duplicate_module_import() {
    // Tests that importing two modules with the same name without aliases produces an error
    let temp = tempdir().unwrap();

    // Create lib/utils.lak
    fs::create_dir(temp.path().join("lib")).unwrap();
    let lib_utils_path = temp.path().join("lib/utils.lak");
    fs::write(
        &lib_utils_path,
        r#"pub fn greet() -> void {
    println("Hello from lib")
}
"#,
    )
    .unwrap();

    // Create utils.lak in root
    let utils_path = temp.path().join("utils.lak");
    fs::write(
        &utils_path,
        r#"pub fn greet() -> void {
    println("Hello from root")
}
"#,
    )
    .unwrap();

    // Create main.lak that imports both (same module name "utils")
    let main_path = temp.path().join("main.lak");
    fs::write(
        &main_path,
        r#"import "./utils"
import "./lib/utils"

fn main() -> void {
    utils.greet()
}
"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .current_dir(temp.path())
        .args(["build", "main.lak"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Verify short_message in report title
    assert!(
        stderr.contains("\x1b[31mError:\x1b[0m Duplicate module import"),
        "Expected 'Duplicate module import' error, got: {}",
        stderr
    );
    // Verify detailed message in label
    assert!(
        stderr.contains("Module name 'utils' is already imported from './utils'"),
        "Expected error message to mention duplicate module name, got: {}",
        stderr
    );
    // Verify help text
    assert!(
        stderr
            .contains("\x1b[38;5;115mHelp\x1b[0m: Use an alias: import \"./lib/utils\" as <alias>"),
        "Expected help text about using alias, got: {}",
        stderr
    );
}

#[test]
fn test_error_semantic_error_in_imported_module() {
    let temp = tempdir().unwrap();

    // Create utils.lak with an undefined variable (semantic error)
    let utils_path = temp.path().join("utils.lak");
    fs::write(
        &utils_path,
        r#"pub fn greet() -> void {
    println(undefined_var)
}
"#,
    )
    .unwrap();

    // Create main.lak that imports utils
    let main_path = temp.path().join("main.lak");
    fs::write(
        &main_path,
        r#"import "./utils"

fn main() -> void {
    utils.greet()
}
"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .current_dir(temp.path())
        .args(["build", "main.lak"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Verify short_message in report title
    assert!(
        stderr.contains("\x1b[31mError:\x1b[0m Undefined variable"),
        "Expected 'Undefined variable' error for imported module, got: {}",
        stderr
    );
    // Verify detailed message in label
    assert!(
        stderr.contains("Undefined variable: 'undefined_var'"),
        "Expected error to mention the undefined variable name, got: {}",
        stderr
    );
}

#[test]
fn test_error_module_call_with_arguments() {
    let temp = tempdir().unwrap();

    // Create utils.lak with a parameterless function
    let utils_path = temp.path().join("utils.lak");
    fs::write(
        &utils_path,
        r#"pub fn greet() -> void {
    println("Hello")
}
"#,
    )
    .unwrap();

    // Create main.lak that calls with an argument
    let main_path = temp.path().join("main.lak");
    fs::write(
        &main_path,
        r#"import "./utils"

fn main() -> void {
    utils.greet(42)
}
"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .current_dir(temp.path())
        .args(["build", "main.lak"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Verify short_message in report title
    assert!(
        stderr.contains("\x1b[31mError:\x1b[0m Invalid argument"),
        "Expected 'Invalid argument' error, got: {}",
        stderr
    );
    // Verify detailed message in label
    assert!(
        stderr.contains("Function 'utils.greet' expects 0 arguments, but got 1"),
        "Expected error to mention argument count, got: {}",
        stderr
    );
}

#[test]
fn test_error_module_call_missing_required_argument() {
    let temp = tempdir().unwrap();

    let utils_path = temp.path().join("utils.lak");
    fs::write(
        &utils_path,
        r#"pub fn greet(name: string) -> void {
    println(name)
}
"#,
    )
    .unwrap();

    let main_path = temp.path().join("main.lak");
    fs::write(
        &main_path,
        r#"import "./utils"

fn main() -> void {
    utils.greet()
}
"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .current_dir(temp.path())
        .args(["build", "main.lak"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("\x1b[31mError:\x1b[0m Invalid argument"),
        "Expected 'Invalid argument' error, got: {}",
        stderr
    );
    assert!(
        stderr.contains("Function 'utils.greet' expects 1 arguments, but got 0"),
        "Expected error to mention argument count, got: {}",
        stderr
    );
}

#[test]
fn test_error_three_level_circular_import() {
    let temp = tempdir().unwrap();

    // Create a.lak that imports b
    let a_path = temp.path().join("a.lak");
    fs::write(
        &a_path,
        r#"import "./b"

pub fn a_func() -> void {
    b.b_func()
}
"#,
    )
    .unwrap();

    // Create b.lak that imports c
    let b_path = temp.path().join("b.lak");
    fs::write(
        &b_path,
        r#"import "./c"

pub fn b_func() -> void {
    c.c_func()
}
"#,
    )
    .unwrap();

    // Create c.lak that imports a (completing the cycle)
    let c_path = temp.path().join("c.lak");
    fs::write(
        &c_path,
        r#"import "./a"

pub fn c_func() -> void {
    a.a_func()
}
"#,
    )
    .unwrap();

    // Create main.lak that imports a
    let main_path = temp.path().join("main.lak");
    fs::write(
        &main_path,
        r#"import "./a"

fn main() -> void {
    a.a_func()
}
"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .current_dir(temp.path())
        .args(["build", "main.lak"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Verify short_message in report title
    assert!(
        stderr.contains("\x1b[31mError:\x1b[0m Circular import"),
        "Expected 'Circular import' error for three-level cycle, got: {}",
        stderr
    );
    // Verify detailed message in label
    assert!(
        stderr.contains("Circular import detected: a -> b -> c -> a"),
        "Expected cycle details for three-level cycle, got: {}",
        stderr
    );
    // Verify help text
    assert!(
        stderr.contains("\x1b[38;5;115mHelp\x1b[0m: break the cycle by removing one of the imports or restructuring your modules"),
        "Expected help text about breaking the cycle, got: {}",
        stderr
    );
}

#[test]
fn test_name_mangling_no_collision() {
    // Verifies that the length-prefix mangling scheme prevents collisions:
    // Module "a" with function "b__greet" → "_L1_a_b__greet"
    // Module "a__b" with function "greet" → "_L4_a__b_greet"
    // These are distinct, so both functions should work correctly.
    let temp = tempdir().unwrap();

    // Create a.lak with a function containing double underscores
    let a_path = temp.path().join("a.lak");
    fs::write(
        &a_path,
        r#"pub fn b__greet() -> void {
    println("from a.b__greet")
}
"#,
    )
    .unwrap();

    // Create a__b.lak with a function that would collide under naive mangling
    let a_b_module_path = temp.path().join("a__b.lak");
    fs::write(
        &a_b_module_path,
        r#"pub fn greet() -> void {
    println("from a__b.greet")
}
"#,
    )
    .unwrap();

    // Create main.lak that imports both
    let main_path = temp.path().join("main.lak");
    fs::write(
        &main_path,
        r#"import "./a"
import "./a__b"

fn main() -> void {
    a.b__greet()
    a__b.greet()
}
"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .current_dir(temp.path())
        .args(["build", "main.lak"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "Length-prefix mangling should prevent collisions. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Run the executable and verify both functions produce correct output
    let exec_path = temp.path().join(executable_name("main"));
    let run_output = Command::new(&exec_path).output().unwrap();
    assert!(
        run_output.status.success(),
        "Executable should run successfully"
    );
    let stdout = String::from_utf8_lossy(&run_output.stdout);
    assert_eq!(stdout, "from a.b__greet\nfrom a__b.greet\n");
}

#[test]
fn test_error_import_with_lak_extension() {
    // Tests that importing with explicit .lak extension is rejected as invalid import path
    let temp = tempdir().unwrap();

    let main_path = temp.path().join("main.lak");
    fs::write(
        &main_path,
        r#"import "./nonexistent.lak"

fn main() -> void {}
"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .current_dir(temp.path())
        .args(["build", "main.lak"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Verify short_message in report title
    assert!(
        stderr.contains("\x1b[31mError:\x1b[0m Invalid import path"),
        "Expected 'Invalid import path' error, got: {}",
        stderr
    );
    // Verify detailed message in label
    assert!(
        stderr.contains("Import path must not include file extension: './nonexistent.lak'"),
        "Expected error message about file extension, got: {}",
        stderr
    );
    // Verify help text
    assert!(
        stderr.contains("\x1b[38;5;115mHelp\x1b[0m: use './nonexistent' instead"),
        "Expected help text suggesting path without extension, got: {}",
        stderr
    );
}

#[test]
fn test_error_invalid_entry_module_name() {
    // Tests that entry modules with invalid identifiers as filenames are rejected
    let temp = tempdir().unwrap();

    let invalid_path = temp.path().join("my-app.lak");
    fs::write(
        &invalid_path,
        r#"fn main() -> void {
    println("Hello")
}
"#,
    )
    .unwrap();

    let output = Command::new(lak_binary())
        .current_dir(temp.path())
        .args(["build", "my-app.lak"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Verify error message (no ANSI codes - this error has no span, output via eprintln)
    assert!(
        stderr.contains("Module names must be valid identifiers"),
        "Expected 'Module names must be valid identifiers' error for entry module with hyphen, got: {}",
        stderr
    );
    // Verify the problematic filename is mentioned
    assert!(
        stderr.contains("my-app"),
        "Expected error to mention the invalid filename 'my-app', got: {}",
        stderr
    );
}
