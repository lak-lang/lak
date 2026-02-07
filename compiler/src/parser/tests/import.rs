//! Import statement parsing tests.
//!
//! Tests for:
//! - Basic import parsing
//! - Import with alias
//! - Import span calculation

use super::*;

// ===================
// Basic import parsing
// ===================

#[test]
fn test_parse_import_basic() {
    let program = parse(
        r#"import "math"

fn main() -> void {}"#,
    )
    .unwrap();
    assert_eq!(program.imports.len(), 1);
    assert_eq!(program.imports[0].path, "math");
    assert!(program.imports[0].alias.is_none());
}

#[test]
fn test_parse_import_with_alias() {
    let program = parse(
        r#"import "utils" as u

fn main() -> void {}"#,
    )
    .unwrap();
    assert_eq!(program.imports.len(), 1);
    assert_eq!(program.imports[0].path, "utils");
    assert_eq!(program.imports[0].alias, Some("u".to_string()));
}

#[test]
fn test_parse_import_long_alias() {
    let program = parse(
        r#"import "some/long/path" as my_module

fn main() -> void {}"#,
    )
    .unwrap();
    assert_eq!(program.imports.len(), 1);
    assert_eq!(program.imports[0].path, "some/long/path");
    assert_eq!(program.imports[0].alias, Some("my_module".to_string()));
}

#[test]
fn test_parse_multiple_imports() {
    let program = parse(
        r#"import "a"
import "b" as b_mod
import "c"

fn main() -> void {}"#,
    )
    .unwrap();
    assert_eq!(program.imports.len(), 3);
    assert_eq!(program.imports[0].path, "a");
    assert!(program.imports[0].alias.is_none());
    assert_eq!(program.imports[1].path, "b");
    assert_eq!(program.imports[1].alias, Some("b_mod".to_string()));
    assert_eq!(program.imports[2].path, "c");
    assert!(program.imports[2].alias.is_none());
}

// ===================
// Import span tests
// ===================

#[test]
fn test_import_span_basic() {
    // import "math"
    // 0123456789012
    let program = parse(
        r#"import "math"

fn main() -> void {}"#,
    )
    .unwrap();
    let import = &program.imports[0];

    // Span should start at 'import' and end after the path
    assert_eq!(import.span.start, 0);
    assert_eq!(import.span.line, 1);
    assert_eq!(import.span.column, 1);
}

#[test]
fn test_import_span_with_alias() {
    // import "math" as m
    // 0         1
    // 012345678901234567
    let program = parse(
        r#"import "math" as m

fn main() -> void {}"#,
    )
    .unwrap();
    let import = &program.imports[0];

    // Span should cover entire import statement including alias
    assert_eq!(import.span.start, 0);
    assert!(import.span.end > 0);
}

#[test]
fn test_import_span_with_leading_whitespace() {
    let program = parse(
        r#"  import "lib"

fn main() -> void {}"#,
    )
    .unwrap();
    let import = &program.imports[0];

    // Span should start at 'import', not at leading whitespace
    assert_eq!(import.span.start, 2);
    assert_eq!(import.span.column, 3);
}

// ===================
// Import with functions
// ===================

#[test]
fn test_import_before_function() {
    let program = parse(
        r#"import "io"

fn main() -> void {
    println("hello")
}"#,
    )
    .unwrap();
    assert_eq!(program.imports.len(), 1);
    assert_eq!(program.functions.len(), 1);
    assert_eq!(program.functions[0].name, "main");
}

#[test]
fn test_import_before_pub_function() {
    let program = parse(
        r#"import "io"

pub fn helper() -> void {}

fn main() -> void {}"#,
    )
    .unwrap();
    assert_eq!(program.imports.len(), 1);
    assert_eq!(program.functions.len(), 2);
}
