//! Function definition parsing tests.
//!
//! Tests for:
//! - Basic function definition parsing
//! - FnDef span calculation
//! - Return type span calculation

use super::*;
use crate::ast::Visibility;

// ===================
// Function definition parsing
// ===================

#[test]
fn test_empty_program() {
    let program = parse("").unwrap();
    assert!(program.functions.is_empty());
}

#[test]
fn test_main_function_empty_body() {
    let program = parse("fn main() -> void {}").unwrap();
    assert_eq!(program.functions.len(), 1);
    assert_eq!(program.functions[0].name, "main");
    assert_eq!(program.functions[0].return_type, "void");
    assert!(program.functions[0].body.is_empty());
}

#[test]
fn test_main_function_with_body() {
    let program = parse(r#"fn main() -> void { println("hello") }"#).unwrap();
    assert_eq!(program.functions.len(), 1);
    assert_eq!(program.functions[0].name, "main");
    assert_eq!(program.functions[0].body.len(), 1);
}

#[test]
fn test_function_with_single_parameter() {
    let program = parse("fn greet(name: string) -> void { println(name) }").unwrap();
    assert_eq!(program.functions.len(), 1);

    let fn_def = &program.functions[0];
    assert_eq!(fn_def.name, "greet");
    assert_eq!(fn_def.params.len(), 1);
    assert_eq!(fn_def.params[0].name, "name");
    assert_eq!(fn_def.params[0].ty, Type::String);
}

#[test]
fn test_function_with_multiple_parameters() {
    let program = parse("fn add(a: i32, b: i64, ok: bool) -> void {}").unwrap();
    assert_eq!(program.functions.len(), 1);

    let fn_def = &program.functions[0];
    assert_eq!(fn_def.name, "add");
    assert_eq!(fn_def.params.len(), 3);
    assert_eq!(fn_def.params[0].name, "a");
    assert_eq!(fn_def.params[0].ty, Type::I32);
    assert_eq!(fn_def.params[1].name, "b");
    assert_eq!(fn_def.params[1].ty, Type::I64);
    assert_eq!(fn_def.params[2].name, "ok");
    assert_eq!(fn_def.params[2].ty, Type::Bool);
}

#[test]
fn test_function_with_float_parameters() {
    let program = parse("fn blend(a: f32, b: f64) -> void {}").unwrap();
    let fn_def = &program.functions[0];
    assert_eq!(fn_def.params.len(), 2);
    assert_eq!(fn_def.params[0].ty, Type::F32);
    assert_eq!(fn_def.params[1].ty, Type::F64);
}

#[test]
fn test_function_parameters_allow_newlines() {
    let program = parse(
        r#"fn greet(
            name: string,
            age: i32
        ) -> void {
            println(name)
            println(age)
        }"#,
    )
    .unwrap();

    let fn_def = &program.functions[0];
    assert_eq!(fn_def.params.len(), 2);
    assert_eq!(fn_def.params[0].name, "name");
    assert_eq!(fn_def.params[0].ty, Type::String);
    assert_eq!(fn_def.params[1].name, "age");
    assert_eq!(fn_def.params[1].ty, Type::I32);
}

#[test]
fn test_multiple_functions() {
    let program = parse("fn foo() -> void {}\nfn bar() -> void {}").unwrap();
    assert_eq!(program.functions.len(), 2);
    assert_eq!(program.functions[0].name, "foo");
    assert_eq!(program.functions[1].name, "bar");
}

#[test]
fn test_function_with_multiple_statements() {
    let program = parse(
        r#"fn main() -> void {
            println("first")
            println("second")
        }"#,
    )
    .unwrap();
    assert_eq!(program.functions[0].body.len(), 2);
}

// ============================================================
// FnDef span calculation tests
// ============================================================

#[test]
fn test_fn_def_span_simple() {
    // "fn main() -> void {}"
    // 0123456789...
    let source = "fn main() -> void {}";
    let program = parse(source).unwrap();
    assert_eq!(program.functions.len(), 1);

    let fn_def = &program.functions[0];
    // Span should cover "fn main() -> void " (from 'f' to just before '{')
    assert_eq!(fn_def.span.start, 0, "span.start should be at 'f'");
    assert_eq!(
        fn_def.span.end, 18,
        "span.end should be just before '{{' at position 18"
    );
    assert_eq!(fn_def.span.line, 1);
    assert_eq!(fn_def.span.column, 1);
}

#[test]
fn test_fn_def_span_with_leading_whitespace() {
    // "  fn foo() -> void {}"
    // 0123456789...
    let source = "  fn foo() -> void {}";
    let program = parse(source).unwrap();
    assert_eq!(program.functions.len(), 1);

    let fn_def = &program.functions[0];
    // Span should start at 'f' (position 2), not at the leading whitespace
    assert_eq!(
        fn_def.span.start, 2,
        "span.start should skip leading whitespace"
    );
    assert_eq!(fn_def.span.line, 1);
    assert_eq!(fn_def.span.column, 3);
}

#[test]
fn test_fn_def_span_with_body() {
    // Function definition with body content
    let source = "fn main() -> void {\n    println(\"hello\")\n}";
    let program = parse(source).unwrap();
    assert_eq!(program.functions.len(), 1);

    let fn_def = &program.functions[0];
    // Span should cover only the signature "fn main() -> void " (0..18)
    // not the body content
    assert_eq!(fn_def.span.start, 0);
    assert_eq!(fn_def.span.end, 18);
    assert_eq!(fn_def.span.line, 1);
    assert_eq!(fn_def.span.column, 1);
}

#[test]
fn test_fn_def_span_multiple_functions() {
    let source = "fn foo() -> void {}\nfn bar() -> void {}";
    let program = parse(source).unwrap();
    assert_eq!(program.functions.len(), 2);

    // First function: "fn foo() -> void " spans 0..17
    let foo = &program.functions[0];
    assert_eq!(foo.span.start, 0);
    assert_eq!(foo.span.line, 1);

    // Second function: starts at position 20 (after "fn foo() -> void {}\n")
    let bar = &program.functions[1];
    assert_eq!(bar.span.start, 20);
    assert_eq!(bar.span.line, 2);
    assert_eq!(bar.span.column, 1);
}

// ============================================================
// return_type_span calculation tests
// ============================================================

#[test]
fn test_return_type_span_simple() {
    // "fn main() -> void {}"
    // 0         1
    // 0123456789012345678901
    // void starts at position 13
    let source = "fn main() -> void {}";
    let program = parse(source).unwrap();
    let fn_def = &program.functions[0];

    // "void" spans from 13 to 17
    assert_eq!(fn_def.return_type_span.start, 13);
    assert_eq!(fn_def.return_type_span.end, 17);
    assert_eq!(fn_def.return_type_span.line, 1);
    assert_eq!(fn_def.return_type_span.column, 14);
}

#[test]
fn test_return_type_span_int() {
    // "fn main() -> int {}"
    // 0         1
    // 0123456789012345678
    // int starts at position 13
    let source = "fn main() -> int {}";
    let program = parse(source).unwrap();
    let fn_def = &program.functions[0];

    // "int" spans from 13 to 16
    assert_eq!(fn_def.return_type_span.start, 13);
    assert_eq!(fn_def.return_type_span.end, 16);
}

#[test]
fn test_return_type_span_with_leading_whitespace() {
    // "  fn foo() -> void {}"
    // 0         1
    // 012345678901234567890
    //   fn foo() -> void {}
    //               ^--- void starts at position 14
    let source = "  fn foo() -> void {}";
    let program = parse(source).unwrap();
    let fn_def = &program.functions[0];

    assert_eq!(fn_def.return_type_span.start, 14);
    assert_eq!(fn_def.return_type_span.end, 18);
    assert_eq!(fn_def.return_type_span.column, 15);
}

#[test]
fn test_return_type_span_multiple_functions() {
    // "fn foo() -> void {}\nfn bar() -> int {}"
    // 0         1         2         3
    // 0123456789012345678901234567890123456789
    // fn foo() -> void {}\nfn bar() -> int {}
    //             ^--- void at 12
    //                                 ^--- int at 32
    let source = "fn foo() -> void {}\nfn bar() -> int {}";
    let program = parse(source).unwrap();

    // First function: "void" at position 12
    let foo = &program.functions[0];
    assert_eq!(foo.return_type_span.start, 12);
    assert_eq!(foo.return_type_span.end, 16);

    // Second function: "int" at position 32 (20 + 12)
    let bar = &program.functions[1];
    assert_eq!(bar.return_type_span.start, 32);
    assert_eq!(bar.return_type_span.end, 35);
    assert_eq!(bar.return_type_span.line, 2);
}

// ============================================================
// Span edge case tests
// ============================================================

#[test]
fn test_fn_def_span_with_extra_whitespace() {
    // Test function with extra whitespace between tokens
    let source = "fn   main()   ->   void   {}";
    let program = parse(source).unwrap();
    let fn_def = &program.functions[0];

    // Function span should start at 'f' and end before '{'
    assert_eq!(fn_def.span.start, 0);
    // return_type_span should point to 'void'
    assert_eq!(fn_def.return_type, "void");
    assert!(fn_def.return_type_span.start > 0);
    assert!(fn_def.return_type_span.end > fn_def.return_type_span.start);
}

#[test]
fn test_fn_def_span_with_comment_before() {
    // Comment before function definition
    let source = "// This is a comment\nfn main() -> void {}";
    let program = parse(source).unwrap();
    let fn_def = &program.functions[0];

    // Function span should start at 'fn', not at the comment
    assert_eq!(fn_def.span.start, 21); // After "// This is a comment\n"
    assert_eq!(fn_def.span.line, 2);
    assert_eq!(fn_def.span.column, 1);
}

#[test]
fn test_fn_def_span_with_long_function_name() {
    // Test with a longer function name
    let source = "fn very_long_function_name_here() -> void {}";
    let program = parse(source).unwrap();
    let fn_def = &program.functions[0];

    assert_eq!(fn_def.name, "very_long_function_name_here");
    // return_type_span should still correctly point to 'void'
    let rt_start = fn_def.return_type_span.start;
    let rt_end = fn_def.return_type_span.end;
    assert_eq!(rt_end - rt_start, 4); // "void" is 4 characters
}

// ============================================================
// Visibility parsing tests
// ============================================================

#[test]
fn test_parse_pub_fn() {
    let program = parse("pub fn test() -> void {}").unwrap();
    assert_eq!(program.functions.len(), 1);
    assert_eq!(program.functions[0].name, "test");
    assert_eq!(program.functions[0].visibility, Visibility::Public);
}

#[test]
fn test_parse_private_fn() {
    let program = parse("fn test() -> void {}").unwrap();
    assert_eq!(program.functions.len(), 1);
    assert_eq!(program.functions[0].name, "test");
    assert_eq!(program.functions[0].visibility, Visibility::Private);
}

#[test]
fn test_parse_pub_fn_with_body() {
    let program = parse(r#"pub fn greet() -> void { println("Hello") }"#).unwrap();
    assert_eq!(program.functions.len(), 1);
    assert_eq!(program.functions[0].visibility, Visibility::Public);
    assert_eq!(program.functions[0].body.len(), 1);
}

#[test]
fn test_parse_mixed_visibility_functions() {
    let program = parse("pub fn a() -> void {}\nfn b() -> void {}\npub fn c() -> void {}").unwrap();
    assert_eq!(program.functions.len(), 3);
    assert_eq!(program.functions[0].visibility, Visibility::Public);
    assert_eq!(program.functions[1].visibility, Visibility::Private);
    assert_eq!(program.functions[2].visibility, Visibility::Public);
}

#[test]
fn test_pub_fn_span_starts_at_pub() {
    // "pub fn test() -> void {}"
    // 0         1         2
    // 0123456789012345678901234
    // pub fn test() -> void {}
    // ^--- span should start at 'p' (position 0)
    let source = "pub fn test() -> void {}";
    let program = parse(source).unwrap();
    let fn_def = &program.functions[0];

    // Span should start at 'pub', not 'fn'
    assert_eq!(fn_def.span.start, 0, "span.start should be at 'pub'");
    assert_eq!(fn_def.span.line, 1);
    assert_eq!(fn_def.span.column, 1);
}

#[test]
fn test_pub_fn_span_with_leading_whitespace() {
    // "  pub fn foo() -> void {}"
    // 0123456789...
    let source = "  pub fn foo() -> void {}";
    let program = parse(source).unwrap();
    let fn_def = &program.functions[0];

    // Span should start at 'pub' (position 2), not at leading whitespace
    assert_eq!(
        fn_def.span.start, 2,
        "span.start should skip leading whitespace"
    );
    assert_eq!(fn_def.span.column, 3);
}
