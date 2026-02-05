# tests Module

Integration and end-to-end tests for the Lak compiler.

## Overview

Comprehensive test suite verifying the compiler's functionality from source code to executable output. Tests cover the complete compilation pipeline (lexing → parsing → semantic analysis → codegen → linking → execution), error detection at each stage, and pipeline integration.

## Module Structure

| File | Category | Tests | Description |
|------|----------|-------|-------------|
| `common/mod.rs` | Utilities | - | Shared test helpers and pipeline functions |
| `e2e_basic.rs` | E2E | 7 | Basic functionality (println, comments, functions) |
| `e2e_strings.rs` | E2E | 8 | String literals and escape sequences |
| `e2e_variables.rs` | E2E | 10 | Variable declarations (i32, i64) |
| `e2e_run.rs` | E2E | 11 | `lak run` command execution |
| `errors_lex.rs` | Errors | 4 | Lexical analysis error detection |
| `errors_parse.rs` | Errors | 3 | Parser error detection |
| `errors_semantic.rs` | Errors | 14 | Semantic analysis error detection |
| `pipeline.rs` | Integration | 9 | Phase integration and direct AST construction |

## Test Categories

### E2E Tests (36 tests)

Compile, link, and execute real Lak programs, validating stdout output.

```rust
let output = compile_and_run(r#"fn main() -> void { println("test") }"#).unwrap();
assert_eq!(output, "test\n");
```

### Error Tests (21 tests)

Verify errors are detected at the correct compilation stage.

```rust
let result = compile_error(source);
let (stage, msg) = result.expect("Expected compilation to fail");
assert!(matches!(stage, CompileStage::Semantic));
assert!(msg.contains("Unknown function"));
```

### Integration Tests (9 tests)

Verify correct interaction between compiler phases. Direct AST construction for edge cases not expressible in source code.

## Common Utilities

| Function | Purpose |
|----------|---------|
| `compile_and_run(source)` | Full pipeline: compile → link → execute → return stdout |
| `compile_error(source)` | Return `Some((CompileStage, error_message))` on failure, `None` on success |
| `dummy_span()` | Create placeholder span for test AST construction |
| `CompileStage` | Enum: `Lex`, `Parse`, `Semantic`, `Codegen` |

## Running Tests

```bash
cargo test                    # All tests
cargo test e2e_               # All E2E tests
cargo test errors_            # All error tests
cargo test test_hello_world   # Single test
```

## Extension Guidelines

1. **New E2E test**: add to appropriate `e2e_*.rs`, use `compile_and_run()`, assert stdout
2. **New error test**: add to `errors_*.rs` matching the stage, use `compile_error()`, verify stage and message
3. **New integration test**: add to `pipeline.rs`, use direct AST construction with `dummy_span()` if needed
4. **New test file**: create `<category>_<feature>.rs`, add `mod common;`, update this CLAUDE.md
5. **Test naming**: follow pattern `test_<descriptive_name>` or `test_<category>_<specific_case>`
