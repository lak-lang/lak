//! Integration tests for the Lak compiler.
//!
//! These tests verify the full compilation pipeline from source code
//! to executable output.

use lak::ast::{Expr, ExprKind, FnDef, Program, Stmt, StmtKind, Type};
use lak::codegen::{Codegen, CodegenError};
use lak::lexer::Lexer;
use lak::parser::Parser;
use lak::token::Span;

use inkwell::context::Context;
use std::process::Command;
use tempfile::tempdir;

/// Path to the Lak runtime library, set at compile time by build.rs.
const LAK_RUNTIME_PATH: &str = env!("LAK_RUNTIME_PATH");

/// Creates a dummy span for test AST construction.
fn dummy_span() -> Span {
    Span::new(0, 0, 1, 1)
}

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
    codegen
        .compile(&program)
        .map_err(|e: CodegenError| e.message)?;

    // Write object file
    let temp_dir = tempdir().map_err(|e| e.to_string())?;
    let object_path = temp_dir.path().join("test.o");
    let executable_path = temp_dir.path().join("test");

    codegen
        .write_object_file(&object_path)
        .map_err(|e| e.to_string())?;

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
        Err(e) => Some((CompileStage::Codegen, e.message)),
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
            body: vec![Stmt::new(
                StmtKind::Expr(Expr::new(
                    ExprKind::Call {
                        callee: "println".to_string(),
                        args: vec![Expr::new(
                            ExprKind::StringLiteral("direct AST".to_string()),
                            dummy_span(),
                        )],
                    },
                    dummy_span(),
                )),
                dummy_span(),
            )],
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

// ===================
// Let statement tests
// ===================

#[test]
fn test_let_statement_basic() {
    // Variable definition only (no output)
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = 42
}"#,
    )
    .unwrap();
    assert_eq!(output, "");
}

#[test]
fn test_multiple_let_statements() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let a: i32 = 1
    let b: i64 = 2
    let c: i32 = 3
}"#,
    )
    .unwrap();
    assert_eq!(output, "");
}

#[test]
fn test_let_with_println() {
    // println mixed with let statements
    let output = compile_and_run(
        r#"fn main() -> void {
    println("before")
    let x: i32 = 42
    println("after")
}"#,
    )
    .unwrap();
    assert_eq!(output, "before\nafter\n");
}

#[test]
fn test_let_with_variable_reference() {
    // Variable reference to initialize another variable
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = 100
    let y: i32 = x
    println("done")
}"#,
    )
    .unwrap();
    assert_eq!(output, "done\n");
}

#[test]
fn test_let_i64_variable() {
    let output = compile_and_run(
        r#"fn main() -> void {
    let big: i64 = 9223372036854775807
    println("ok")
}"#,
    )
    .unwrap();
    assert_eq!(output, "ok\n");
}

#[test]
fn test_let_chain_references() {
    // Chain of variable references: a -> b -> c
    let output = compile_and_run(
        r#"fn main() -> void {
    let a: i32 = 1
    let b: i32 = a
    let c: i32 = b
    println("chain complete")
}"#,
    )
    .unwrap();
    assert_eq!(output, "chain complete\n");
}

#[test]
fn test_compile_error_duplicate_variable() {
    let result = compile_error(
        r#"fn main() -> void {
    let x: i32 = 1
    let x: i32 = 2
}"#,
    );
    let (stage, msg) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Codegen),
        "Expected Codegen error, got {:?}: {}",
        stage,
        msg
    );
    assert!(
        msg.contains("already defined"),
        "Expected 'already defined' in error: {}",
        msg
    );
}

#[test]
fn test_compile_error_undefined_variable() {
    let result = compile_error(
        r#"fn main() -> void {
    let x: i32 = undefined_var
}"#,
    );
    let (stage, msg) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Codegen),
        "Expected Codegen error, got {:?}: {}",
        stage,
        msg
    );
    assert!(
        msg.contains("Undefined variable"),
        "Expected 'Undefined variable' in error: {}",
        msg
    );
}

#[test]
fn test_compile_error_type_mismatch() {
    let result = compile_error(
        r#"fn main() -> void {
    let x: i32 = 42
    let y: i64 = x
}"#,
    );
    let (stage, msg) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Codegen),
        "Expected Codegen error, got {:?}: {}",
        stage,
        msg
    );
    assert!(
        msg.contains("Type mismatch"),
        "Expected 'Type mismatch' in error: {}",
        msg
    );
}

#[test]
fn test_compile_error_unknown_type() {
    let result = compile_error(
        r#"fn main() -> void {
    let x: unknown = 42
}"#,
    );
    let (stage, msg) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Parse),
        "Expected Parse error, got {:?}: {}",
        stage,
        msg
    );
    assert!(
        msg.contains("Unknown type"),
        "Expected 'Unknown type' in error: {}",
        msg
    );
}

#[test]
fn test_ast_let_to_codegen() {
    // Build AST with Let statement directly and compile
    let program = Program {
        functions: vec![FnDef {
            name: "main".to_string(),
            return_type: "void".to_string(),
            body: vec![
                Stmt::new(
                    StmtKind::Let {
                        name: "x".to_string(),
                        ty: Type::I32,
                        init: Expr::new(ExprKind::IntLiteral(42), dummy_span()),
                    },
                    dummy_span(),
                ),
                Stmt::new(
                    StmtKind::Let {
                        name: "y".to_string(),
                        ty: Type::I32,
                        init: Expr::new(ExprKind::Identifier("x".to_string()), dummy_span()),
                    },
                    dummy_span(),
                ),
            ],
        }],
    };

    let context = Context::create();
    let mut codegen = Codegen::new(&context, "let_ast_test");
    codegen
        .compile(&program)
        .expect("Direct AST Let compilation should succeed");
}

#[test]
fn test_compile_error_i32_overflow() {
    // i32::MAX + 1 = 2147483648 should overflow i32
    let result = compile_error(
        r#"fn main() -> void {
    let x: i32 = 2147483648
}"#,
    );
    let (stage, msg) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Codegen),
        "Expected Codegen error, got {:?}: {}",
        stage,
        msg
    );
    assert!(
        msg.contains("out of range for i32"),
        "Expected 'out of range for i32' in error: {}",
        msg
    );
}

#[test]
fn test_compile_error_i32_large_value_overflow() {
    // Test that very large values (i64 max) are rejected for i32
    // Note: Negative literals are not yet supported
    let result = compile_error(
        r#"fn main() -> void {
    let x: i32 = 9223372036854775807
}"#,
    );
    let (stage, msg) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Codegen),
        "Expected Codegen error, got {:?}: {}",
        stage,
        msg
    );
    assert!(
        msg.contains("out of range for i32"),
        "Expected 'out of range for i32' in error: {}",
        msg
    );
}

#[test]
fn test_i32_max_value_valid() {
    // i32::MAX = 2147483647 should be valid for i32
    let output = compile_and_run(
        r#"fn main() -> void {
    let x: i32 = 2147483647
    println("ok")
}"#,
    )
    .unwrap();
    assert_eq!(output, "ok\n");
}

// ===================
// Additional error tests for complete coverage
// ===================

#[test]
fn test_compile_error_i64_overflow() {
    // i64::MAX + 1 should cause lexer error (overflow during parsing)
    let result = compile_error(
        r#"fn main() -> void {
    let x: i64 = 9223372036854775808
}"#,
    );
    let (stage, msg) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Lex),
        "Expected Lex error for i64 overflow, got {:?}: {}",
        stage,
        msg
    );
    assert!(
        msg.contains("out of range for i64"),
        "Expected 'out of range for i64' in error: {}",
        msg
    );
}

#[test]
fn test_compile_error_duplicate_variable_different_type() {
    // Duplicate variable with different type should still be rejected
    let result = compile_error(
        r#"fn main() -> void {
    let x: i32 = 1
    let x: i64 = 2
}"#,
    );
    let (stage, msg) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Codegen),
        "Expected Codegen error, got {:?}: {}",
        stage,
        msg
    );
    assert!(
        msg.contains("already defined"),
        "Expected 'already defined' in error: {}",
        msg
    );
}

#[test]
fn test_compile_error_forward_reference() {
    // Variable used before it's defined
    let result = compile_error(
        r#"fn main() -> void {
    let x: i32 = y
    let y: i32 = 1
}"#,
    );
    let (stage, msg) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Codegen),
        "Expected Codegen error, got {:?}: {}",
        stage,
        msg
    );
    assert!(
        msg.contains("Undefined variable"),
        "Expected 'Undefined variable' in error: {}",
        msg
    );
}

#[test]
fn test_let_single_char_name() {
    // Single character variable name
    let output = compile_and_run(
        r#"fn main() -> void {
    let a: i32 = 1
    println("ok")
}"#,
    )
    .unwrap();
    assert_eq!(output, "ok\n");
}

#[test]
fn test_let_underscore_prefix_name() {
    // Variable name starting with underscore
    let output = compile_and_run(
        r#"fn main() -> void {
    let _private: i32 = 42
    println("ok")
}"#,
    )
    .unwrap();
    assert_eq!(output, "ok\n");
}

#[test]
fn test_error_string_literal_as_int_value() {
    // String literal cannot be used as integer initializer
    // (This requires direct AST construction since parser would reject it)
    let program = Program {
        functions: vec![FnDef {
            name: "main".to_string(),
            return_type: "void".to_string(),
            body: vec![Stmt::new(
                StmtKind::Let {
                    name: "x".to_string(),
                    ty: Type::I32,
                    init: Expr::new(ExprKind::StringLiteral("hello".to_string()), dummy_span()),
                },
                dummy_span(),
            )],
        }],
    };

    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");
    let err = codegen.compile(&program).expect_err("Should fail");
    assert!(
        err.message
            .contains("String literals cannot be used as integer values"),
        "Expected 'String literals cannot be used as integer values' in error: {}",
        err.message
    );
}

#[test]
fn test_error_function_call_as_int_value() {
    // Function call cannot be used as integer initializer (void functions)
    // (This requires direct AST construction since parser would allow it syntactically)
    let program = Program {
        functions: vec![FnDef {
            name: "main".to_string(),
            return_type: "void".to_string(),
            body: vec![Stmt::new(
                StmtKind::Let {
                    name: "x".to_string(),
                    ty: Type::I32,
                    init: Expr::new(
                        ExprKind::Call {
                            callee: "some_func".to_string(),
                            args: vec![],
                        },
                        dummy_span(),
                    ),
                },
                dummy_span(),
            )],
        }],
    };

    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");
    let err = codegen.compile(&program).expect_err("Should fail");
    assert!(
        err.message.contains("cannot be used as a value"),
        "Expected 'cannot be used as a value' in error: {}",
        err.message
    );
}

#[test]
fn test_error_int_literal_as_statement() {
    // Integer literal used as statement should error
    let program = Program {
        functions: vec![FnDef {
            name: "main".to_string(),
            return_type: "void".to_string(),
            body: vec![Stmt::new(
                StmtKind::Expr(Expr::new(ExprKind::IntLiteral(42), dummy_span())),
                dummy_span(),
            )],
        }],
    };

    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");
    let err = codegen.compile(&program).expect_err("Should fail");
    assert!(
        err.message
            .contains("Integer literal as a statement has no effect"),
        "Expected 'Integer literal as a statement has no effect' in error: {}",
        err.message
    );
}

#[test]
fn test_error_identifier_as_statement() {
    // Variable used as statement should error
    let program = Program {
        functions: vec![FnDef {
            name: "main".to_string(),
            return_type: "void".to_string(),
            body: vec![
                Stmt::new(
                    StmtKind::Let {
                        name: "x".to_string(),
                        ty: Type::I32,
                        init: Expr::new(ExprKind::IntLiteral(42), dummy_span()),
                    },
                    dummy_span(),
                ),
                Stmt::new(
                    StmtKind::Expr(Expr::new(
                        ExprKind::Identifier("x".to_string()),
                        dummy_span(),
                    )),
                    dummy_span(),
                ),
            ],
        }],
    };

    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");
    let err = codegen.compile(&program).expect_err("Should fail");
    assert!(
        err.message.contains("used as a statement has no effect"),
        "Expected 'used as a statement has no effect' in error: {}",
        err.message
    );
}

#[test]
fn test_error_string_literal_as_statement() {
    // String literal used as statement should error
    let program = Program {
        functions: vec![FnDef {
            name: "main".to_string(),
            return_type: "void".to_string(),
            body: vec![Stmt::new(
                StmtKind::Expr(Expr::new(
                    ExprKind::StringLiteral("hello".to_string()),
                    dummy_span(),
                )),
                dummy_span(),
            )],
        }],
    };

    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");
    let err = codegen.compile(&program).expect_err("Should fail");
    assert!(
        err.message
            .contains("String literal as a statement has no effect"),
        "Expected 'String literal as a statement has no effect' in error: {}",
        err.message
    );
}

#[test]
fn test_compile_error_variable_in_println() {
    // Passing a variable to println should error (println only accepts string literals)
    let result = compile_error(
        r#"fn main() -> void {
    let x: i32 = 42
    println(x)
}"#,
    );
    let (stage, msg) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Codegen),
        "Expected Codegen error, got {:?}: {}",
        stage,
        msg
    );
    assert!(
        msg.contains("string literal"),
        "Expected 'string literal' in error: {}",
        msg
    );
}

#[test]
fn test_negative_number_not_supported() {
    // Negative literals like -42 are not supported; `-` is only valid in `->`
    let result = compile_error(
        r#"fn main() -> void {
    let x: i32 = -42
}"#,
    );
    let (stage, msg) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Lex),
        "Expected Lex error for negative number, got {:?}: {}",
        stage,
        msg
    );
    // The lexer expects `->` after `-`, so we get an error about that
    assert!(
        msg.contains("Expected '>' after '-'"),
        "Expected error about '->' arrow, got: {}",
        msg
    );
}

#[test]
fn test_error_i32_underflow_via_ast() {
    // Test that values less than i32::MIN are rejected for i32 type.
    // This requires direct AST construction since negative literals aren't
    // supported in source code yet.
    let program = Program {
        functions: vec![FnDef {
            name: "main".to_string(),
            return_type: "void".to_string(),
            body: vec![Stmt::new(
                StmtKind::Let {
                    name: "x".to_string(),
                    ty: Type::I32,
                    init: Expr::new(ExprKind::IntLiteral(-2147483649), dummy_span()), // i32::MIN - 1
                },
                dummy_span(),
            )],
        }],
    };

    let context = Context::create();
    let mut codegen = Codegen::new(&context, "test");
    let err = codegen.compile(&program).expect_err("Should fail");
    assert!(
        err.message.contains("out of range for i32"),
        "Expected 'out of range for i32' in error: {}",
        err.message
    );
}

#[test]
fn test_let_unicode_variable_name() {
    // Unicode variable names should work
    let output = compile_and_run(
        r#"fn main() -> void {
    let 変数: i32 = 42
    let αβγ: i64 = 100
    println("unicode vars ok")
}"#,
    )
    .unwrap();
    assert_eq!(output, "unicode vars ok\n");
}

#[test]
fn test_compile_error_println_int_literal() {
    // Passing an integer literal directly to println should error
    let result = compile_error(
        r#"fn main() -> void {
    println(42)
}"#,
    );
    let (stage, msg) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Codegen),
        "Expected Codegen error, got {:?}: {}",
        stage,
        msg
    );
    assert!(
        msg.contains("string literal"),
        "Expected 'string literal' in error: {}",
        msg
    );
}

#[test]
fn test_compile_error_let_missing_variable_name() {
    // `let : i32 = 42` - missing variable name should be a parse error
    let result = compile_error(
        r#"fn main() -> void {
    let : i32 = 42
}"#,
    );
    let (stage, msg) = result.expect("Expected compilation to fail");
    assert!(
        matches!(stage, CompileStage::Parse),
        "Expected Parse error, got {:?}: {}",
        stage,
        msg
    );
    // Parser expects an identifier after 'let'
    assert!(
        msg.contains("Expected identifier") || msg.contains("expected identifier"),
        "Expected 'identifier' in error: {}",
        msg
    );
}
