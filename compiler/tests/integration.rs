//! Integration tests for the Lak compiler.
//!
//! These tests verify the full compilation pipeline from source code
//! to executable output.

use lak::ast::{Expr, FnDef, Program, Stmt};
use lak::codegen::Codegen;
use lak::lexer::Lexer;
use lak::parser::Parser;

use inkwell::context::Context;
use std::process::Command;
use tempfile::tempdir;

/// Path to the Lak runtime library, set at compile time by build.rs.
const LAK_RUNTIME_PATH: &str = env!("LAK_RUNTIME_PATH");

/// Compiles a Lak program to an executable and runs it, returning stdout output.
///
/// This function performs the complete compilation pipeline:
/// lexing -> parsing -> codegen -> linking -> execution.
/// Only stdout is captured; stderr is not included in the success case.
fn compile_and_run(source: &str) -> Result<String, String> {
    // Lex
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().map_err(|e| e.to_string())?;

    // Parse
    let mut parser = Parser::new(tokens);
    let program = parser.parse().map_err(|e| e.to_string())?;

    // Codegen
    let context = Context::create();
    let mut codegen = Codegen::new(&context, "integration_test");
    codegen.compile(&program)?;

    // Write object file
    let temp_dir = tempdir().map_err(|e| e.to_string())?;
    let object_path = temp_dir.path().join("test.o");
    let executable_path = temp_dir.path().join("test");

    codegen.write_object_file(&object_path)?;

    // Link
    let object_str = object_path
        .to_str()
        .ok_or_else(|| format!("Object path {:?} is not valid UTF-8", object_path))?;
    let exec_str = executable_path
        .to_str()
        .ok_or_else(|| format!("Executable path {:?} is not valid UTF-8", executable_path))?;

    let link_output = Command::new("cc")
        .args([object_str, LAK_RUNTIME_PATH, "-o", exec_str])
        .output()
        .map_err(|e| format!("Failed to run linker: {}", e))?;

    if !link_output.status.success() {
        return Err(format!(
            "Linker failed: {}",
            String::from_utf8_lossy(&link_output.stderr)
        ));
    }

    // Run
    let run_output = Command::new(&executable_path)
        .output()
        .map_err(|e| format!("Failed to run executable: {}", e))?;

    if !run_output.status.success() {
        return Err(format!(
            "Executable failed with exit code: {:?}",
            run_output.status.code()
        ));
    }

    Ok(String::from_utf8_lossy(&run_output.stdout).to_string())
}

/// Represents the stage at which compilation failed.
#[derive(Debug)]
enum CompileStage {
    Lex,
    Parse,
    Codegen,
}

/// Attempts to lex, parse, and compile a program.
/// Returns the stage and error message if any stage fails.
fn compile_error(source: &str) -> Option<(CompileStage, String)> {
    let mut lexer = Lexer::new(source);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => return Some((CompileStage::Lex, e.to_string())),
    };

    let mut parser = Parser::new(tokens);
    let program = match parser.parse() {
        Ok(p) => p,
        Err(e) => return Some((CompileStage::Parse, e.to_string())),
    };

    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");
    match codegen.compile(&program) {
        Ok(()) => None,
        Err(e) => Some((CompileStage::Codegen, e)),
    }
}

// ===================
// End-to-end tests
// ===================

#[test]
fn test_hello_world() {
    let output = compile_and_run(
        r#"fn main() -> void {
    println("Hello, World!")
}"#,
    )
    .unwrap();
    assert_eq!(output, "Hello, World!\n");
}

#[test]
fn test_multiple_println() {
    let output = compile_and_run(
        r#"fn main() -> void {
    println("first")
    println("second")
    println("third")
}"#,
    )
    .unwrap();
    assert_eq!(output, "first\nsecond\nthird\n");
}

#[test]
fn test_escape_sequences() {
    let output = compile_and_run(
        r#"fn main() -> void {
    println("hello\tworld")
}"#,
    )
    .unwrap();
    assert_eq!(output, "hello\tworld\n");
}

#[test]
fn test_escape_newline() {
    let output = compile_and_run(
        r#"fn main() -> void {
    println("line1\nline2")
}"#,
    )
    .unwrap();
    assert_eq!(output, "line1\nline2\n");
}

#[test]
fn test_empty_string() {
    let output = compile_and_run(
        r#"fn main() -> void {
    println("")
}"#,
    )
    .unwrap();
    assert_eq!(output, "\n");
}

#[test]
fn test_escaped_quotes() {
    let output = compile_and_run(
        r#"fn main() -> void {
    println("say \"hello\"")
}"#,
    )
    .unwrap();
    assert_eq!(output, "say \"hello\"\n");
}

#[test]
fn test_escaped_backslash() {
    let output = compile_and_run(
        r#"fn main() -> void {
    println("path\\to\\file")
}"#,
    )
    .unwrap();
    assert_eq!(output, "path\\to\\file\n");
}

#[test]
fn test_empty_main() {
    // Empty main should compile and run without output
    let output = compile_and_run("fn main() -> void {}").unwrap();
    assert_eq!(output, "");
}

#[test]
fn test_comments_only() {
    let output = compile_and_run(
        r#"// this is a comment
fn main() -> void {
    // another comment
}"#,
    )
    .unwrap();
    assert_eq!(output, "");
}

#[test]
fn test_code_with_comments() {
    let output = compile_and_run(
        r#"// Print greeting
fn main() -> void {
    println("hi") // inline comment
    // end
}"#,
    )
    .unwrap();
    assert_eq!(output, "hi\n");
}

#[test]
fn test_multiple_functions_definition() {
    // Multiple function definitions should compile; only main executes
    let output = compile_and_run(
        r#"fn helper() -> void {}
fn main() -> void {
    println("main executed")
}
fn another() -> void {}"#,
    )
    .unwrap();
    assert_eq!(output, "main executed\n");
}

#[test]
fn test_main_not_first() {
    // main function doesn't need to be first
    let output = compile_and_run(
        r#"fn first() -> void {}
fn second() -> void {}
fn main() -> void {
    println("found main")
}"#,
    )
    .unwrap();
    assert_eq!(output, "found main\n");
}

// ===================
// Compile error tests
// ===================

#[test]
fn test_compile_error_syntax() {
    let result = compile_error(r#"fn main() -> void { println("unclosed }"#);
    let (stage, msg) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Lex),
        "Expected Lex error, got {:?}: {}",
        stage,
        msg
    );
    assert!(
        msg.contains("Unterminated"),
        "Expected 'Unterminated' in error: {}",
        msg
    );
}

#[test]
fn test_compile_error_unknown_function() {
    let result = compile_error(r#"fn main() -> void { unknown_func("test") }"#);
    let (stage, msg) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Codegen),
        "Expected Codegen error, got {:?}: {}",
        stage,
        msg
    );
    assert!(
        msg.contains("Unknown function"),
        "Expected 'Unknown function' in error: {}",
        msg
    );
}

#[test]
fn test_compile_error_missing_main() {
    let result = compile_error("");
    let (stage, msg) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Codegen),
        "Expected Codegen error, got {:?}: {}",
        stage,
        msg
    );
    assert!(
        msg.contains("No main function"),
        "Expected 'No main function' in error: {}",
        msg
    );
}

#[test]
fn test_compile_error_main_wrong_return_type() {
    let result = compile_error("fn main() -> int {}");
    let (stage, msg) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Codegen),
        "Expected Codegen error, got {:?}: {}",
        stage,
        msg
    );
    assert!(
        msg.contains("must return void"),
        "Expected 'must return void' in error: {}",
        msg
    );
}

#[test]
fn test_compile_error_top_level_statement() {
    let result = compile_error(r#"println("hello")"#);
    let (stage, msg) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Parse),
        "Expected Parse error, got {:?}: {}",
        stage,
        msg
    );
}

#[test]
fn test_compile_error_invalid_escape() {
    let result = compile_error(r#"fn main() -> void { println("\z") }"#);
    let (stage, msg) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Lex),
        "Expected Lex error, got {:?}: {}",
        stage,
        msg
    );
    assert!(
        msg.contains("Unknown escape"),
        "Expected 'Unknown escape' in error: {}",
        msg
    );
}

// ===================
// Full pipeline tests
// ===================

#[test]
fn test_lexer_parser_integration() {
    let source = r#"fn main() -> void { println("test") }"#;

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    // fn, main, (, ), ->, void, {, println, (, string, ), }, eof
    assert_eq!(tokens.len(), 13);

    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();
    assert_eq!(program.functions.len(), 1);
    assert_eq!(program.functions[0].name, "main");
}

#[test]
fn test_ast_to_codegen() {
    // Build AST directly and compile
    let program = Program {
        functions: vec![FnDef {
            name: "main".to_string(),
            return_type: "void".to_string(),
            body: vec![Stmt::Expr(Expr::Call {
                callee: "println".to_string(),
                args: vec![Expr::StringLiteral("direct AST".to_string())],
            })],
        }],
    };

    let context = Context::create();
    let mut codegen = Codegen::new(&context, "direct_ast_test");
    codegen
        .compile(&program)
        .expect("Direct AST compilation should succeed");
}

#[test]
fn test_special_characters() {
    let output = compile_and_run(
        r#"fn main() -> void {
    println("!@#$%^&*(){}[]|;:'<>,.?/")
}"#,
    )
    .unwrap();
    assert_eq!(output, "!@#$%^&*(){}[]|;:'<>,.?/\n");
}

#[test]
fn test_unicode_string() {
    let output = compile_and_run(
        r#"fn main() -> void {
    println("こんにちは世界")
}"#,
    )
    .unwrap();
    assert_eq!(output, "こんにちは世界\n");
}

#[test]
fn test_mixed_escapes() {
    let output = compile_and_run(
        r#"fn main() -> void {
    println("tab:\there\nnewline")
}"#,
    )
    .unwrap();
    assert_eq!(output, "tab:\there\nnewline\n");
}
