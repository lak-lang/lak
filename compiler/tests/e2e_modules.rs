//! End-to-end tests for the module and import system.
//!
//! These tests verify that the compiler correctly resolves imports,
//! compiles multi-file programs, and handles module-qualified function calls.

mod common;

use common::lak_binary;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_basic_import() {
    let temp = tempdir().unwrap();

    // Create utils.lak with a public function
    let utils_path = temp.path().join("utils.lak");
    fs::write(
        &utils_path,
        r#"pub fn greet() -> void {
    println("Hello from utils!")
}
"#,
    )
    .unwrap();

    // Create main.lak that imports and calls utils.greet()
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

    // Build
    let build_output = Command::new(lak_binary())
        .current_dir(temp.path())
        .args(["build", "main.lak"])
        .output()
        .unwrap();

    assert!(
        build_output.status.success(),
        "Build failed: {}",
        String::from_utf8_lossy(&build_output.stderr)
    );

    // Run the executable
    let exec_path = temp.path().join("main");
    let run_output = Command::new(&exec_path).output().unwrap();

    assert!(run_output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&run_output.stdout),
        "Hello from utils!\n"
    );
}

#[test]
fn test_import_with_alias() {
    let temp = tempdir().unwrap();

    // Create utils.lak
    let utils_path = temp.path().join("utils.lak");
    fs::write(
        &utils_path,
        r#"pub fn greet() -> void {
    println("Hello!")
}
"#,
    )
    .unwrap();

    // Create main.lak with alias
    let main_path = temp.path().join("main.lak");
    fs::write(
        &main_path,
        r#"import "./utils" as u

fn main() -> void {
    u.greet()
}
"#,
    )
    .unwrap();

    let build_output = Command::new(lak_binary())
        .current_dir(temp.path())
        .args(["build", "main.lak"])
        .output()
        .unwrap();

    assert!(
        build_output.status.success(),
        "Build failed: {}",
        String::from_utf8_lossy(&build_output.stderr)
    );

    let exec_path = temp.path().join("main");
    let run_output = Command::new(&exec_path).output().unwrap();

    assert!(run_output.status.success());
    assert_eq!(String::from_utf8_lossy(&run_output.stdout), "Hello!\n");
}

#[test]
fn test_multiple_imports() {
    let temp = tempdir().unwrap();

    // Create utils.lak
    let utils_path = temp.path().join("utils.lak");
    fs::write(
        &utils_path,
        r#"pub fn greet() -> void {
    println("Hello!")
}
"#,
    )
    .unwrap();

    // Create math.lak
    let math_path = temp.path().join("math.lak");
    fs::write(
        &math_path,
        r#"pub fn show_number() -> void {
    println(42)
}
"#,
    )
    .unwrap();

    // Create main.lak
    let main_path = temp.path().join("main.lak");
    fs::write(
        &main_path,
        r#"import "./utils"
import "./math"

fn main() -> void {
    utils.greet()
    math.show_number()
}
"#,
    )
    .unwrap();

    let build_output = Command::new(lak_binary())
        .current_dir(temp.path())
        .args(["build", "main.lak"])
        .output()
        .unwrap();

    assert!(
        build_output.status.success(),
        "Build failed: {}",
        String::from_utf8_lossy(&build_output.stderr)
    );

    let exec_path = temp.path().join("main");
    let run_output = Command::new(&exec_path).output().unwrap();

    assert!(run_output.status.success());
    assert_eq!(String::from_utf8_lossy(&run_output.stdout), "Hello!\n42\n");
}

#[test]
fn test_transitive_import() {
    let temp = tempdir().unwrap();

    // Create base.lak (imported by utils)
    let base_path = temp.path().join("base.lak");
    fs::write(
        &base_path,
        r#"pub fn hello() -> void {
    println("Hello from base!")
}
"#,
    )
    .unwrap();

    // Create utils.lak (imports base, re-exposes via its own function)
    let utils_path = temp.path().join("utils.lak");
    fs::write(
        &utils_path,
        r#"import "./base"

pub fn greet() -> void {
    base.hello()
}
"#,
    )
    .unwrap();

    // Create main.lak (imports utils)
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

    let build_output = Command::new(lak_binary())
        .current_dir(temp.path())
        .args(["build", "main.lak"])
        .output()
        .unwrap();

    assert!(
        build_output.status.success(),
        "Build failed: {}",
        String::from_utf8_lossy(&build_output.stderr)
    );

    let exec_path = temp.path().join("main");
    let run_output = Command::new(&exec_path).output().unwrap();

    assert!(run_output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&run_output.stdout),
        "Hello from base!\n"
    );
}

#[test]
fn test_module_with_multiple_functions() {
    let temp = tempdir().unwrap();

    // Create utils.lak with multiple public functions
    let utils_path = temp.path().join("utils.lak");
    fs::write(
        &utils_path,
        r#"pub fn hello() -> void {
    println("Hello")
}

pub fn goodbye() -> void {
    println("Goodbye")
}

fn private_helper() -> void {
    println("This is private")
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
    utils.hello()
    utils.goodbye()
}
"#,
    )
    .unwrap();

    let build_output = Command::new(lak_binary())
        .current_dir(temp.path())
        .args(["build", "main.lak"])
        .output()
        .unwrap();

    assert!(
        build_output.status.success(),
        "Build failed: {}",
        String::from_utf8_lossy(&build_output.stderr)
    );

    let exec_path = temp.path().join("main");
    let run_output = Command::new(&exec_path).output().unwrap();

    assert!(run_output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&run_output.stdout),
        "Hello\nGoodbye\n"
    );
}

#[test]
fn test_lak_run_with_import() {
    let temp = tempdir().unwrap();

    // Create utils.lak
    let utils_path = temp.path().join("utils.lak");
    fs::write(
        &utils_path,
        r#"pub fn greet() -> void {
    println("Hello from run!")
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
        .args(["run", "main.lak"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "Run failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "Hello from run!\n");
}

#[test]
fn test_diamond_dependency() {
    // Tests the diamond dependency pattern: A→B,C, B→D, C→D
    // Verifies that module D is not compiled twice (caching works)
    let temp = tempdir().unwrap();

    // Create base.lak (shared dependency)
    let base_path = temp.path().join("base.lak");
    fs::write(
        &base_path,
        r#"pub fn shared() -> void {
    println("shared")
}
"#,
    )
    .unwrap();

    // Create left.lak (imports base)
    let left_path = temp.path().join("left.lak");
    fs::write(
        &left_path,
        r#"import "./base"

pub fn left() -> void {
    base.shared()
}
"#,
    )
    .unwrap();

    // Create right.lak (imports base)
    let right_path = temp.path().join("right.lak");
    fs::write(
        &right_path,
        r#"import "./base"

pub fn right() -> void {
    base.shared()
}
"#,
    )
    .unwrap();

    // Create main.lak (imports both left and right)
    let main_path = temp.path().join("main.lak");
    fs::write(
        &main_path,
        r#"import "./left"
import "./right"

fn main() -> void {
    left.left()
    right.right()
}
"#,
    )
    .unwrap();

    let build_output = Command::new(lak_binary())
        .current_dir(temp.path())
        .args(["build", "main.lak"])
        .output()
        .unwrap();

    assert!(
        build_output.status.success(),
        "Build failed: {}",
        String::from_utf8_lossy(&build_output.stderr)
    );

    let exec_path = temp.path().join("main");
    let run_output = Command::new(&exec_path).output().unwrap();

    assert!(run_output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&run_output.stdout),
        "shared\nshared\n"
    );
}

#[test]
fn test_module_with_main_function() {
    // Tests that imported module's main function is not executed
    let temp = tempdir().unwrap();

    // Create utils.lak with a main function that should not run
    let utils_path = temp.path().join("utils.lak");
    fs::write(
        &utils_path,
        r#"fn main() -> void {
    println("Utils main - should not run!")
}

pub fn helper() -> void {
    println("Helper")
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
    utils.helper()
}
"#,
    )
    .unwrap();

    let build_output = Command::new(lak_binary())
        .current_dir(temp.path())
        .args(["build", "main.lak"])
        .output()
        .unwrap();

    assert!(
        build_output.status.success(),
        "Build failed: {}",
        String::from_utf8_lossy(&build_output.stderr)
    );

    let exec_path = temp.path().join("main");
    let run_output = Command::new(&exec_path).output().unwrap();

    assert!(run_output.status.success());
    // Only "Helper" should be printed, not "Utils main - should not run!"
    assert_eq!(String::from_utf8_lossy(&run_output.stdout), "Helper\n");
}

#[test]
fn test_empty_module() {
    // Tests that an empty module (no functions) can be imported
    let temp = tempdir().unwrap();

    // Create empty.lak with just a comment
    let empty_path = temp.path().join("empty.lak");
    fs::write(
        &empty_path,
        r#"// This module is intentionally empty
"#,
    )
    .unwrap();

    // Create main.lak that imports the empty module
    let main_path = temp.path().join("main.lak");
    fs::write(
        &main_path,
        r#"import "./empty"

fn main() -> void {
    println("test")
}
"#,
    )
    .unwrap();

    let build_output = Command::new(lak_binary())
        .current_dir(temp.path())
        .args(["build", "main.lak"])
        .output()
        .unwrap();

    assert!(
        build_output.status.success(),
        "Build failed: {}",
        String::from_utf8_lossy(&build_output.stderr)
    );

    let exec_path = temp.path().join("main");
    let run_output = Command::new(&exec_path).output().unwrap();

    assert!(run_output.status.success());
    assert_eq!(String::from_utf8_lossy(&run_output.stdout), "test\n");
}

#[test]
fn test_multiple_aliases_to_same_module() {
    // Tests importing the same module with different aliases
    let temp = tempdir().unwrap();

    // Create utils.lak
    let utils_path = temp.path().join("utils.lak");
    fs::write(
        &utils_path,
        r#"pub fn greet() -> void {
    println("Hello")
}
"#,
    )
    .unwrap();

    // Create main.lak with two different aliases
    let main_path = temp.path().join("main.lak");
    fs::write(
        &main_path,
        r#"import "./utils" as u
import "./utils" as util

fn main() -> void {
    u.greet()
    util.greet()
}
"#,
    )
    .unwrap();

    let build_output = Command::new(lak_binary())
        .current_dir(temp.path())
        .args(["build", "main.lak"])
        .output()
        .unwrap();

    assert!(
        build_output.status.success(),
        "Build failed: {}",
        String::from_utf8_lossy(&build_output.stderr)
    );

    let exec_path = temp.path().join("main");
    let run_output = Command::new(&exec_path).output().unwrap();

    assert!(run_output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&run_output.stdout),
        "Hello\nHello\n"
    );
}

#[test]
fn test_local_function_same_name_as_imported() {
    // Tests that local and imported functions with same name don't conflict
    let temp = tempdir().unwrap();

    // Create utils.lak with greet function
    let utils_path = temp.path().join("utils.lak");
    fs::write(
        &utils_path,
        r#"pub fn greet() -> void {
    println("From utils")
}
"#,
    )
    .unwrap();

    // Create main.lak with its own greet function
    let main_path = temp.path().join("main.lak");
    fs::write(
        &main_path,
        r#"import "./utils"

fn greet() -> void {
    println("From main")
}

fn main() -> void {
    greet()
    utils.greet()
}
"#,
    )
    .unwrap();

    let build_output = Command::new(lak_binary())
        .current_dir(temp.path())
        .args(["build", "main.lak"])
        .output()
        .unwrap();

    assert!(
        build_output.status.success(),
        "Build failed: {}",
        String::from_utf8_lossy(&build_output.stderr)
    );

    let exec_path = temp.path().join("main");
    let run_output = Command::new(&exec_path).output().unwrap();

    assert!(run_output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&run_output.stdout),
        "From main\nFrom utils\n"
    );
}

#[test]
fn test_intra_module_function_call() {
    let temp = tempdir().unwrap();

    // Create utils.lak where a pub function calls a private sibling function
    let utils_path = temp.path().join("utils.lak");
    fs::write(
        &utils_path,
        r#"fn helper() -> void {
    println("from helper")
}

pub fn greet() -> void {
    helper()
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

    let build_output = Command::new(lak_binary())
        .current_dir(temp.path())
        .args(["build", "main.lak"])
        .output()
        .unwrap();

    assert!(
        build_output.status.success(),
        "Build failed: {}",
        String::from_utf8_lossy(&build_output.stderr)
    );

    let exec_path = temp.path().join("main");
    let run_output = Command::new(&exec_path).output().unwrap();

    assert!(run_output.status.success());
    assert_eq!(String::from_utf8_lossy(&run_output.stdout), "from helper\n");
}

#[test]
fn test_parent_relative_import() {
    let temp = tempdir().unwrap();

    // Create lib/ subdirectory
    fs::create_dir(temp.path().join("lib")).unwrap();

    // Create utils.lak in root
    let utils_path = temp.path().join("utils.lak");
    fs::write(
        &utils_path,
        r#"pub fn greet() -> void {
    println("Hello from parent!")
}
"#,
    )
    .unwrap();

    // Create lib/app.lak that imports ../utils
    let app_path = temp.path().join("lib").join("app.lak");
    fs::write(
        &app_path,
        r#"import "../utils"

pub fn run() -> void {
    utils.greet()
}
"#,
    )
    .unwrap();

    // Create main.lak in root that imports lib/app
    let main_path = temp.path().join("main.lak");
    fs::write(
        &main_path,
        r#"import "./lib/app"

fn main() -> void {
    app.run()
}
"#,
    )
    .unwrap();

    let build_output = Command::new(lak_binary())
        .current_dir(temp.path())
        .args(["build", "main.lak"])
        .output()
        .unwrap();

    assert!(
        build_output.status.success(),
        "Build failed: {}",
        String::from_utf8_lossy(&build_output.stderr)
    );

    let exec_path = temp.path().join("main");
    let run_output = Command::new(&exec_path).output().unwrap();

    assert!(run_output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&run_output.stdout),
        "Hello from parent!\n"
    );
}

#[test]
fn test_import_with_lak_extension() {
    // Tests that import paths with explicit .lak extension work correctly
    let temp = tempdir().unwrap();

    // Create utils.lak
    let utils_path = temp.path().join("utils.lak");
    fs::write(
        &utils_path,
        r#"pub fn greet() -> void {
    println("Hello from lak extension!")
}
"#,
    )
    .unwrap();

    // Create main.lak with explicit .lak extension in import
    let main_path = temp.path().join("main.lak");
    fs::write(
        &main_path,
        r#"import "./utils.lak"

fn main() -> void {
    utils.greet()
}
"#,
    )
    .unwrap();

    let build_output = Command::new(lak_binary())
        .current_dir(temp.path())
        .args(["build", "main.lak"])
        .output()
        .unwrap();

    assert!(
        build_output.status.success(),
        "Build failed: {}",
        String::from_utf8_lossy(&build_output.stderr)
    );

    let exec_path = temp.path().join("main");
    let run_output = Command::new(&exec_path).output().unwrap();

    assert!(run_output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&run_output.stdout),
        "Hello from lak extension!\n"
    );
}

#[test]
fn test_same_name_modules_in_different_directories() {
    let temp = tempdir().unwrap();

    // Create dir/ subdirectory
    fs::create_dir(temp.path().join("dir")).unwrap();

    // Create foo.lak in root
    let foo_path = temp.path().join("foo.lak");
    fs::write(
        &foo_path,
        r#"pub fn foo() -> void {
    println("foo1")
}
"#,
    )
    .unwrap();

    // Create dir/foo.lak
    let dir_foo_path = temp.path().join("dir").join("foo.lak");
    fs::write(
        &dir_foo_path,
        r#"pub fn foo() -> void {
    println("foo2")
}
"#,
    )
    .unwrap();

    // Create main.lak importing both
    let main_path = temp.path().join("main.lak");
    fs::write(
        &main_path,
        r#"import "./foo"
import "./dir/foo" as foo2

fn main() -> void {
    foo.foo()
    foo2.foo()
}
"#,
    )
    .unwrap();

    let build_output = Command::new(lak_binary())
        .current_dir(temp.path())
        .args(["build", "main.lak"])
        .output()
        .unwrap();

    assert!(
        build_output.status.success(),
        "Build failed: {}",
        String::from_utf8_lossy(&build_output.stderr)
    );

    let exec_path = temp.path().join("main");
    let run_output = Command::new(&exec_path).output().unwrap();

    assert!(run_output.status.success());
    assert_eq!(String::from_utf8_lossy(&run_output.stdout), "foo1\nfoo2\n");
}

#[test]
fn test_deeply_nested_module_import() {
    let temp = tempdir().unwrap();

    // Create a/b/c/ subdirectory
    fs::create_dir_all(temp.path().join("a").join("b").join("c")).unwrap();

    // Create a/b/c/deep.lak
    let deep_path = temp.path().join("a").join("b").join("c").join("deep.lak");
    fs::write(
        &deep_path,
        r#"pub fn hello() -> void {
    println("from deep")
}
"#,
    )
    .unwrap();

    // Create main.lak importing deeply nested module
    let main_path = temp.path().join("main.lak");
    fs::write(
        &main_path,
        r#"import "./a/b/c/deep"

fn main() -> void {
    deep.hello()
}
"#,
    )
    .unwrap();

    let build_output = Command::new(lak_binary())
        .current_dir(temp.path())
        .args(["build", "main.lak"])
        .output()
        .unwrap();

    assert!(
        build_output.status.success(),
        "Build failed: {}",
        String::from_utf8_lossy(&build_output.stderr)
    );

    let exec_path = temp.path().join("main");
    let run_output = Command::new(&exec_path).output().unwrap();

    assert!(run_output.status.success());
    assert_eq!(String::from_utf8_lossy(&run_output.stdout), "from deep\n");
}

#[test]
fn test_intra_module_call_in_subdirectory() {
    let temp = tempdir().unwrap();

    fs::create_dir(temp.path().join("lib")).unwrap();

    let utils_path = temp.path().join("lib").join("utils.lak");
    fs::write(
        &utils_path,
        r#"fn helper() -> void {
    println("from helper")
}

pub fn greet() -> void {
    helper()
}
"#,
    )
    .unwrap();

    let main_path = temp.path().join("main.lak");
    fs::write(
        &main_path,
        r#"import "./lib/utils"

fn main() -> void {
    utils.greet()
}
"#,
    )
    .unwrap();

    let build_output = Command::new(lak_binary())
        .current_dir(temp.path())
        .args(["build", "main.lak"])
        .output()
        .unwrap();

    assert!(
        build_output.status.success(),
        "Build failed: {}",
        String::from_utf8_lossy(&build_output.stderr)
    );

    let exec_path = temp.path().join("main");
    let run_output = Command::new(&exec_path).output().unwrap();

    assert!(run_output.status.success());
    assert_eq!(String::from_utf8_lossy(&run_output.stdout), "from helper\n");
}

#[test]
fn test_transitive_import_with_subdirectories() {
    let temp = tempdir().unwrap();

    // Create lib/helpers/ subdirectory
    fs::create_dir_all(temp.path().join("lib").join("helpers")).unwrap();

    // Create lib/helpers/utils.lak
    let utils_path = temp.path().join("lib").join("helpers").join("utils.lak");
    fs::write(
        &utils_path,
        r#"pub fn hello() -> void {
    println("from deep transitive")
}
"#,
    )
    .unwrap();

    // Create lib/app.lak that imports ./helpers/utils
    let app_path = temp.path().join("lib").join("app.lak");
    fs::write(
        &app_path,
        r#"import "./helpers/utils"

pub fn run() -> void {
    utils.hello()
}
"#,
    )
    .unwrap();

    // Create main.lak that imports ./lib/app
    let main_path = temp.path().join("main.lak");
    fs::write(
        &main_path,
        r#"import "./lib/app"

fn main() -> void {
    app.run()
}
"#,
    )
    .unwrap();

    let build_output = Command::new(lak_binary())
        .current_dir(temp.path())
        .args(["build", "main.lak"])
        .output()
        .unwrap();

    assert!(
        build_output.status.success(),
        "Build failed: {}",
        String::from_utf8_lossy(&build_output.stderr)
    );

    let exec_path = temp.path().join("main");
    let run_output = Command::new(&exec_path).output().unwrap();

    assert!(run_output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&run_output.stdout),
        "from deep transitive\n"
    );
}

#[test]
fn test_subdirectory_import_with_alias() {
    let temp = tempdir().unwrap();

    fs::create_dir(temp.path().join("lib")).unwrap();

    let utils_path = temp.path().join("lib").join("utils.lak");
    fs::write(
        &utils_path,
        r#"pub fn greet() -> void {
    println("hello from alias")
}
"#,
    )
    .unwrap();

    let main_path = temp.path().join("main.lak");
    fs::write(
        &main_path,
        r#"import "./lib/utils" as u

fn main() -> void {
    u.greet()
}
"#,
    )
    .unwrap();

    let build_output = Command::new(lak_binary())
        .current_dir(temp.path())
        .args(["build", "main.lak"])
        .output()
        .unwrap();

    assert!(
        build_output.status.success(),
        "Build failed: {}",
        String::from_utf8_lossy(&build_output.stderr)
    );

    let exec_path = temp.path().join("main");
    let run_output = Command::new(&exec_path).output().unwrap();

    assert!(run_output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&run_output.stdout),
        "hello from alias\n"
    );
}
