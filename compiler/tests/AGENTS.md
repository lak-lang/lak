# tests Module

Integration and end-to-end tests for the Lak compiler.

## Overview

Comprehensive test suite verifying the compiler's functionality from source code to executable output. Tests cover the complete compilation pipeline (lexing → parsing → semantic analysis → codegen → linking → execution), error detection at each stage, and pipeline integration.

## Module Structure

| File | Category | Tests | Description |
|------|----------|-------|-------------|
| `common/mod.rs` | Utilities | - | Shared test helpers and pipeline functions |
| `e2e_any.rs` | E2E | 13 | Mixed type println tests |
| `e2e_arithmetic.rs` | E2E | 99 | Arithmetic operations, division-by-zero, and integer overflow |
| `e2e_basic.rs` | E2E | 7 | Basic functionality (println, comments, functions) |
| `e2e_bool.rs` | E2E | 12 | Boolean type handling |
| `e2e_build.rs` | E2E | 15 | `lak build` command behavior |
| `e2e_comparison.rs` | E2E | 75 | Comparison operators (==, !=, <, >, <=, >=) |
| `e2e_functions.rs` | E2E | 25 | User-defined function calls |
| `e2e_if_else.rs` | E2E | 6 | `if`/`else if`/`else` statement behavior |
| `e2e_if_expression.rs` | E2E | 9 | `if` expression behavior |
| `e2e_imports.rs` | E2E | 4 | `import` syntax parsing |
| `e2e_modules.rs` | E2E | 22 | Multi-file module compilation |
| `e2e_panic.rs` | E2E | 7 | `panic()` function behavior |
| `e2e_run.rs` | E2E | 17 | `lak run` command execution |
| `e2e_strings.rs` | E2E | 21 | String literals and escape sequences |
| `e2e_variables.rs` | E2E | 15 | Variable declarations (`let`, `let mut`, i32, i64) |
| `e2e_visibility.rs` | E2E | 4 | `pub fn` visibility keyword |
| `e2e_while.rs` | E2E | 6 | `while` loop, `break`, `continue`, and return behavior |
| `errors_codegen.rs` | Errors | 1 | Codegen internal/user-facing error diagnostics |
| `errors_lex.rs` | Errors | 8 | Lexical analysis error detection |
| `errors_modules.rs` | Errors | 18 | Module resolution error detection |
| `errors_parse.rs` | Errors | 27 | Parser error detection |
| `errors_semantic.rs` | Errors | 108 | Semantic analysis error detection |
| `pipeline.rs` | Integration | 9 | Phase integration and direct AST construction |

## Test Categories

### E2E Tests (356 tests)

Compile, link, and execute real Lak programs, validating stdout output.

```rust
let output = compile_and_run(r#"fn main() -> void { println("test") }"#).unwrap();
assert_eq!(output, "test\n");
```

### Error Tests (161 tests)

Verify errors are detected at the correct compilation stage with correct error kind.

```rust
// Preferred: Use compile_error_with_kind to verify both message and error kind
let result = compile_error_with_kind(source);
let (stage, msg, short_msg, kind) = result.expect("Expected compilation to fail");
assert!(matches!(stage, CompileStage::Semantic));
assert_eq!(msg, "Undefined function: 'unknown_func'");
assert_eq!(short_msg, "Undefined function");
assert_eq!(kind, CompileErrorKind::Semantic(SemanticErrorKind::UndefinedFunction));
```

**Best Practice**: Always verify the error kind, not just the message. This ensures:
- Correct error categorization for programmatic error handling
- Resilience against message wording changes
- Type-safe error matching

### Integration Tests (9 tests)

Verify correct interaction between compiler phases. Direct AST construction for edge cases not expressible in source code.

## Common Utilities

| Function | Purpose |
|----------|---------|
| `compile_and_run(source)` | Full pipeline: compile → link → execute → return stdout |
| `compile_error(source)` | Return `Some((CompileStage, error_message))` on failure, `None` on success |
| `compile_error_with_kind(source)` | Return `Some((CompileStage, error_message, CompileErrorKind))` on failure, including typed error kind |
| `dummy_span()` | Create placeholder span for test AST construction |
| `CompileStage` | Enum: `Lex`, `Parse`, `Semantic`, `Codegen` |
| `CompileErrorKind` | Enum wrapping all error kinds: `Lex(LexErrorKind)`, `Parse(ParseErrorKind)`, `Semantic(SemanticErrorKind)`, `Codegen(CodegenErrorKind)` |

## Running Tests

```bash
cargo test                    # All tests
cargo test e2e_               # All E2E tests
cargo test errors_            # All error tests
cargo test test_hello_world   # Single test
```

## Extension Guidelines

1. **New E2E test**: add to appropriate `e2e_*.rs`, use `compile_and_run()`, assert stdout
2. **New error test**: add to `errors_*.rs` matching the stage, use `compile_error_with_kind()`, verify stage, message, AND error kind
3. **New integration test**: add to `pipeline.rs`, use direct AST construction with `dummy_span()` if needed
4. **New test file**: create `<category>_<feature>.rs`, add `mod common;`, update this AGENTS.md
5. **Test naming**: follow pattern `test_<descriptive_name>` or `test_<category>_<specific_case>`
