//! Parser error tests for the Lak compiler.
//!
//! These tests verify that syntax errors are properly detected
//! and reported during parsing.

mod common;

use common::{CompileErrorKind, CompileStage, compile_error_with_kind};
use lak::parser::ParseErrorKind;
use lak::semantic::SemanticErrorKind;

#[test]
fn test_compile_error_top_level_statement() {
    let result = compile_error_with_kind(r#"println("hello")"#);
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Parse),
        "Expected Parse error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(short_msg, "Unexpected token");
    assert_eq!(
        kind,
        CompileErrorKind::Parse(ParseErrorKind::UnexpectedToken),
        "Expected UnexpectedToken error kind"
    );
}

#[test]
fn test_compile_error_unknown_type() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: unknown = 42
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Parse),
        "Expected Parse error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Unknown type: 'unknown'. Expected 'i8', 'i16', 'i32', 'i64', 'u8', 'u16', 'u32', 'u64', 'f32', 'f64', 'byte', 'string', or 'bool'"
    );
    assert_eq!(short_msg, "Unknown type");
    assert_eq!(
        kind,
        CompileErrorKind::Parse(ParseErrorKind::ExpectedType),
        "Expected ExpectedType error kind"
    );
}

#[test]
fn test_compile_error_let_missing_variable_name() {
    // `let : i32 = 42` - missing variable name should be a parse error
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let : i32 = 42
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Parse),
        "Expected Parse error, got {:?}: {}",
        stage,
        msg
    );
    // Parser expects an identifier after 'let'
    assert_eq!(msg, "Expected identifier, found ':'");
    assert_eq!(short_msg, "Expected identifier");
    assert_eq!(
        kind,
        CompileErrorKind::Parse(ParseErrorKind::ExpectedIdentifier),
        "Expected ExpectedIdentifier error kind"
    );
}

#[test]
fn test_compile_error_let_missing_type_separator_or_initializer() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x i32 = 42
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Parse),
        "Expected Parse error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Expected ':' for type annotation or '=' for initializer, found identifier 'i32'"
    );
    assert_eq!(short_msg, "Unexpected token");
    assert_eq!(
        kind,
        CompileErrorKind::Parse(ParseErrorKind::UnexpectedToken),
        "Expected UnexpectedToken error kind"
    );
}

#[test]
fn test_compile_error_let_mut_discard_binding() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let mut _ = panic("boom")
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Parse),
        "Expected Parse error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Discard binding '_' cannot be declared mutable");
    assert_eq!(short_msg, "Unexpected token");
    assert_eq!(
        kind,
        CompileErrorKind::Parse(ParseErrorKind::UnexpectedToken),
        "Expected UnexpectedToken error kind"
    );
}

#[test]
fn test_compile_error_let_typed_discard_binding() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let _: i32 = panic("boom")
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Parse),
        "Expected Parse error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Discard binding '_' cannot have a type annotation; use `let _ = expr`"
    );
    assert_eq!(short_msg, "Unexpected token");
    assert_eq!(
        kind,
        CompileErrorKind::Parse(ParseErrorKind::UnexpectedToken),
        "Expected UnexpectedToken error kind"
    );
}

#[test]
fn test_compile_error_discard_missing_equals() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let _ let _ = 1
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Parse),
        "Expected Parse error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Expected '=', found 'let' keyword");
    assert_eq!(short_msg, "Unexpected token");
    assert_eq!(
        kind,
        CompileErrorKind::Parse(ParseErrorKind::UnexpectedToken),
        "Expected UnexpectedToken error kind"
    );
}

#[test]
fn test_compile_error_missing_statement_terminator() {
    // Two statements on same line without separator should error
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: i32 = 1 let y: i32 = 2
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Parse),
        "Expected Parse error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Expected newline after statement, found 'let' keyword");
    assert_eq!(short_msg, "Missing statement terminator");
    assert_eq!(
        kind,
        CompileErrorKind::Parse(ParseErrorKind::MissingStatementTerminator),
        "Expected MissingStatementTerminator error kind"
    );
}

#[test]
fn test_compile_error_missing_function_call_parens_string() {
    // Missing parens: println "test" instead of println("test")
    let result = compile_error_with_kind(r#"fn main() -> void { println "test" }"#);
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Parse),
        "Expected Parse error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Unexpected string literal after 'println'. If this is a function call, add parentheses: println(...)"
    );
    assert_eq!(short_msg, "Missing function call parentheses");
    assert_eq!(
        kind,
        CompileErrorKind::Parse(ParseErrorKind::MissingFunctionCallParentheses),
        "Expected MissingFunctionCallParentheses error kind"
    );
}

#[test]
fn test_compile_error_missing_function_call_parens_int() {
    // Missing parens: foo 42 instead of foo(42)
    let result = compile_error_with_kind(r#"fn main() -> void { foo 42 }"#);
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Parse),
        "Expected Parse error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Unexpected integer literal after 'foo'. If this is a function call, add parentheses: foo(...)"
    );
    assert_eq!(short_msg, "Missing function call parentheses");
    assert_eq!(
        kind,
        CompileErrorKind::Parse(ParseErrorKind::MissingFunctionCallParentheses),
        "Expected MissingFunctionCallParentheses error kind"
    );
}

#[test]
fn test_compile_error_if_expression_missing_else() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let value: i64 = if true { 42 }
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Parse),
        "Expected Parse error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "if expression requires an else branch");
    assert_eq!(short_msg, "Missing else in if expression");
    assert_eq!(
        kind,
        CompileErrorKind::Parse(ParseErrorKind::MissingElseInIfExpression),
        "Expected MissingElseInIfExpression error kind"
    );
}

#[test]
fn test_compile_error_if_expression_missing_branch_value() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let value: i64 = if true { let x: i64 = 1 } else { 2 }
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Parse),
        "Expected Parse error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "if expression then branch must end with a value expression"
    );
    assert_eq!(short_msg, "Missing branch value in if expression");
    assert_eq!(
        kind,
        CompileErrorKind::Parse(ParseErrorKind::MissingIfExpressionBranchValue),
        "Expected MissingIfExpressionBranchValue error kind"
    );
}

// ========================================
// Import syntax error tests
// ========================================

#[test]
fn test_compile_error_import_missing_path() {
    // `import` without path
    let result = compile_error_with_kind("import\nfn main() -> void {}");
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Parse),
        "Expected Parse error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Expected string literal for import path, found 'fn' keyword"
    );
    assert_eq!(short_msg, "Expected string literal");
    assert_eq!(
        kind,
        CompileErrorKind::Parse(ParseErrorKind::ExpectedStringLiteral),
        "Expected ExpectedStringLiteral error kind"
    );
}

#[test]
fn test_compile_error_import_non_string_path() {
    // `import 123` - non-string path
    let result = compile_error_with_kind("import 123\nfn main() -> void {}");
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Parse),
        "Expected Parse error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Expected string literal for import path, found integer '123'"
    );
    assert_eq!(short_msg, "Expected string literal");
    assert_eq!(
        kind,
        CompileErrorKind::Parse(ParseErrorKind::ExpectedStringLiteral),
        "Expected ExpectedStringLiteral error kind"
    );
}

#[test]
fn test_compile_error_import_empty_path() {
    // `import ""` - empty path
    let result = compile_error_with_kind(
        r#"import ""
fn main() -> void {}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Parse),
        "Expected Parse error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Import path cannot be empty");
    assert_eq!(short_msg, "Empty import path");
    assert_eq!(
        kind,
        CompileErrorKind::Parse(ParseErrorKind::EmptyImportPath),
        "Expected EmptyImportPath error kind"
    );
}

#[test]
fn test_compile_error_import_missing_alias_after_as() {
    // `import "./lib" as` - missing alias after as
    let result = compile_error_with_kind(
        r#"import "./lib" as
fn main() -> void {}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Parse),
        "Expected Parse error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Expected identifier, found 'fn' keyword");
    assert_eq!(short_msg, "Expected identifier");
    assert_eq!(
        kind,
        CompileErrorKind::Parse(ParseErrorKind::ExpectedIdentifier),
        "Expected ExpectedIdentifier error kind"
    );
}

#[test]
fn test_compile_error_import_non_identifier_alias() {
    // `import "./lib" as 123` - non-identifier alias
    let result = compile_error_with_kind(
        r#"import "./lib" as 123
fn main() -> void {}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Parse),
        "Expected Parse error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Expected identifier, found integer '123'");
    assert_eq!(short_msg, "Expected identifier");
    assert_eq!(
        kind,
        CompileErrorKind::Parse(ParseErrorKind::ExpectedIdentifier),
        "Expected ExpectedIdentifier error kind"
    );
}

// ========================================
// Member access error tests
// ========================================

#[test]
fn test_compile_error_nested_member_access_call() {
    // Nested member access in function call (a.b.c()) should be rejected at parse stage
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    a.b.c()
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Parse),
        "Expected Parse error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Nested member access (e.g., a.b.c) is not yet supported. Only simple module.function access is allowed."
    );
    assert_eq!(short_msg, "Nested member access not supported");
    assert_eq!(
        kind,
        CompileErrorKind::Parse(ParseErrorKind::NestedMemberAccessNotSupported),
        "Expected NestedMemberAccessNotSupported error kind"
    );
}

#[test]
fn test_compile_error_nested_member_access_in_let() {
    // Nested member access in let statement (let x = a.b.c) should be rejected at parse stage
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: i32 = a.b.c
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Parse),
        "Expected Parse error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Nested member access (e.g., a.b.c) is not yet supported. Only simple module.function access is allowed."
    );
    assert_eq!(short_msg, "Nested member access not supported");
    assert_eq!(
        kind,
        CompileErrorKind::Parse(ParseErrorKind::NestedMemberAccessNotSupported),
        "Expected NestedMemberAccessNotSupported error kind"
    );
}

// ========================================
// pub keyword error tests
// ========================================

#[test]
fn test_compile_error_pub_let() {
    // `pub let` - pub can only be used with fn
    let result = compile_error_with_kind("pub let x: i32 = 1\nfn main() -> void {}");
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Parse),
        "Expected Parse error, got {:?}: {}",
        stage,
        msg
    );
    // pub must be followed by fn, not let
    assert_eq!(short_msg, "Unexpected token");
    assert_eq!(
        kind,
        CompileErrorKind::Parse(ParseErrorKind::UnexpectedToken),
        "Expected UnexpectedToken error kind"
    );
}

#[test]
fn test_compile_error_pub_wrong_position() {
    // `fn pub main` - pub in wrong position should error
    let result = compile_error_with_kind("fn pub main() -> void {}");
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Parse),
        "Expected Parse error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(short_msg, "Expected identifier");
    assert_eq!(
        kind,
        CompileErrorKind::Parse(ParseErrorKind::ExpectedIdentifier),
        "Expected ExpectedIdentifier error kind"
    );
}

#[test]
fn test_compile_error_duplicate_pub() {
    // `pub pub fn main` - duplicate pub should error
    let result = compile_error_with_kind("pub pub fn main() -> void {}");
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Parse),
        "Expected Parse error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(short_msg, "Unexpected token");
    assert_eq!(
        kind,
        CompileErrorKind::Parse(ParseErrorKind::UnexpectedToken),
        "Expected UnexpectedToken error kind"
    );
}

// ========================================
// Import position error tests
// ========================================

#[test]
fn test_compile_error_import_after_function() {
    // Import after function definition should be rejected
    let result = compile_error_with_kind(
        r#"fn helper() -> void {}
import "./lib"
fn main() -> void {}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Parse),
        "Expected Parse error, got {:?}: {}",
        stage,
        msg
    );
    // Parser should reject import after functions
    assert_eq!(short_msg, "Unexpected token");
    assert_eq!(
        kind,
        CompileErrorKind::Parse(ParseErrorKind::UnexpectedToken),
        "Expected UnexpectedToken error kind"
    );
}

// ========================================
// Dot operator edge cases
// ========================================

#[test]
fn test_compile_error_trailing_dot() {
    // Trailing dot without member should error
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    math.
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Parse),
        "Expected Parse error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(short_msg, "Expected identifier");
    assert_eq!(
        kind,
        CompileErrorKind::Parse(ParseErrorKind::ExpectedIdentifier),
        "Expected ExpectedIdentifier error kind"
    );
}

#[test]
fn test_compile_error_double_dot() {
    // Double dot should error
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    math..add()
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Parse),
        "Expected Parse error, got {:?}: {}",
        stage,
        msg
    );
    // Parser should reject double dot
    assert_eq!(short_msg, "Expected identifier");
    assert_eq!(
        kind,
        CompileErrorKind::Parse(ParseErrorKind::ExpectedIdentifier),
        "Expected ExpectedIdentifier error kind"
    );
}

// ========================================
// Integer literal overflow error tests
// ========================================

#[test]
fn test_compile_error_positive_i64_overflow() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: i64 = 9223372036854775808
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error for positive i64 overflow, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Integer literal '9223372036854775808' is out of range for i64 (valid range: -9223372036854775808 to 9223372036854775807)"
    );
    assert_eq!(short_msg, "Integer overflow");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::IntegerOverflow),
        "Expected IntegerOverflow error kind"
    );
}

#[test]
fn test_compile_error_positive_i64_overflow_in_println_binary_op_with_variable() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: i64 = 1
    println(9223372036854775808 + x)
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error for positive i64 overflow, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Integer literal '9223372036854775808' is out of range for i64 (valid range: -9223372036854775808 to 9223372036854775807)"
    );
    assert_eq!(short_msg, "Integer overflow");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::IntegerOverflow),
        "Expected IntegerOverflow error kind"
    );
}

#[test]
fn test_compile_error_negation_too_large() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: i64 = -9223372036854775809
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Parse),
        "Expected Parse error for negation overflow, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "Integer literal '-9223372036854775809' is out of range for i64 (valid range: -9223372036854775808 to 9223372036854775807)"
    );
    assert_eq!(short_msg, "Integer overflow");
    assert_eq!(
        kind,
        CompileErrorKind::Parse(ParseErrorKind::IntegerLiteralOutOfRange),
        "Expected IntegerLiteralOutOfRange error kind"
    );
}

// Parentheses prevent negation folding: the literal inside parens is parsed
// as a standalone positive value, triggering the positive overflow path.
#[test]
fn test_compile_error_parenthesized_i64_overflow() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: i64 = -(9223372036854775808)
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Semantic),
        "Expected Semantic error for parenthesized overflow, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(
        msg,
        "in unary '-' operation: Integer literal '9223372036854775808' is out of range for i64 (valid range: -9223372036854775808 to 9223372036854775807)"
    );
    assert_eq!(short_msg, "Integer overflow");
    assert_eq!(
        kind,
        CompileErrorKind::Semantic(SemanticErrorKind::IntegerOverflow),
        "Expected IntegerOverflow error kind"
    );
}

#[test]
fn test_missing_rhs_for_logical_and() {
    let result = compile_error_with_kind(
        r#"fn main() -> void {
    let x: bool = true &&
}"#,
    );
    let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Parse),
        "Expected Parse error, got {:?}: {}",
        stage,
        msg
    );
    assert_eq!(msg, "Unexpected token: '}'");
    assert_eq!(short_msg, "Unexpected token");
    assert_eq!(
        kind,
        CompileErrorKind::Parse(ParseErrorKind::UnexpectedToken),
        "Expected UnexpectedToken error kind"
    );
}
