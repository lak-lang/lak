//! End-to-end tests for import syntax parsing.
//!
//! These tests verify that import statements are correctly parsed.
//! Note: Actual module resolution is not yet implemented (Phase 3).
//! Currently, imports are parsed but do not affect program execution.

mod common;

use common::compile_and_run;

#[test]
fn test_import_basic() {
    // Import is parsed but currently has no effect
    let output = compile_and_run(
        r#"import "./math"

fn main() -> void {
    println("imported")
}"#,
    )
    .unwrap();
    assert_eq!(output, "imported\n");
}

#[test]
fn test_import_with_alias() {
    let output = compile_and_run(
        r#"import "./utils" as u

fn main() -> void {
    println("with alias")
}"#,
    )
    .unwrap();
    assert_eq!(output, "with alias\n");
}

#[test]
fn test_multiple_imports() {
    let output = compile_and_run(
        r#"import "./a"
import "./b"
import "./c" as c

fn main() -> void {
    println("multiple")
}"#,
    )
    .unwrap();
    assert_eq!(output, "multiple\n");
}

#[test]
fn test_import_before_pub_fn() {
    let output = compile_and_run(
        r#"import "./lib"

pub fn main() -> void {
    println("pub after import")
}"#,
    )
    .unwrap();
    assert_eq!(output, "pub after import\n");
}
