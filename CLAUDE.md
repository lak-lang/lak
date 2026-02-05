# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Lak is a programming language compiler written in Rust with an LLVM backend. The language emphasizes simplicity, safety, and minimal syntax sugar, influenced by Go (simplicity), Rust (safety), and V (ease of use).

## Development Commands

```bash
# Build the compiler
cargo build

# Build release version
cargo build --release

# Run the compiler
cargo run -- build <file.lak>
cargo run -- build <file.lak> -o <output>  # specify output path
cargo run -- run <file.lak>                # compile and run directly

# Run tests
cargo test

# Run a single test
cargo test <test_name>

# Run tests for a specific module
cargo test lexer::tests
cargo test parser::tests

# Run end-to-end tests only
cargo test --test e2e_basic
cargo test --test e2e_strings
cargo test --test e2e_variables
cargo test --test e2e_run

# Format code
cargo fmt

# Lint code
cargo clippy
```

## Project Structure

```
lak/
├── compiler/          # Main compiler crate
│   ├── src/           # Compiler source code
│   └── tests/         # Integration and end-to-end tests
├── runtime/           # Lak runtime library (staticlib)
└── examples/          # Example .lak programs
```

## Architecture

The compiler follows a traditional pipeline:

```
Source (.lak) → Lexer → Parser → Codegen → LLVM → Object File → Linker → Executable
```

### Module Structure

- `compiler/src/lib.rs` - Library crate exposing all modules for external use and testing
- `compiler/src/main.rs` - CLI entry point using clap, orchestrates the compilation pipeline
- `compiler/src/token.rs` - Token types (`TokenKind`) and source location tracking (`Span`)
- `compiler/src/ast.rs` - AST node definitions (`Program`, `Stmt`, `Expr`)
- `compiler/src/lexer/` - Lexical analysis module
  - `mod.rs` - `Lexer` struct and `tokenize()` method
  - `cursor.rs` - Position tracking and character navigation
  - `error.rs` - `LexError` type
  - `skip.rs` - Whitespace and comment handling
  - `tokens.rs` - Token recognition and reading
  - `tests.rs` - Unit tests
- `compiler/src/parser/` - Recursive descent parser module
  - `mod.rs` - `Parser` struct and `parse()` method
  - `error.rs` - `ParseError` type
  - `expr.rs` - Expression parsing
  - `fn_def.rs` - Function definition parsing
  - `helpers.rs` - Token navigation utilities
  - `stmt.rs` - Statement parsing
  - `types.rs` - Type annotation parsing
  - `tests.rs` - Unit tests
- `compiler/src/codegen/` - LLVM IR generation module
  - `mod.rs` - `Codegen` struct and `compile()` method
  - `error.rs` - `CodegenError` type
  - `binding.rs` - Variable binding management
  - `builtins.rs` - Built-in function implementations (`println`)
  - `expr.rs` - Expression code generation
  - `stmt.rs` - Statement code generation
  - `target.rs` - Target machine and object file output
  - `tests.rs` - Unit tests
- `compiler/tests/` - Integration and end-to-end tests
  - `e2e_basic.rs`, `e2e_strings.rs`, `e2e_variables.rs` - End-to-end tests
  - `pipeline.rs` - Compilation pipeline tests
  - `errors_*.rs` - Error handling tests for each compiler phase
  - `common/mod.rs` - Shared test utilities
- `runtime/src/lib.rs` - Lak runtime library providing I/O functions (`lak_println`)

### Key Dependencies

- **inkwell** - Safe Rust bindings to LLVM (using LLVM 21.1)
- **ariadne** - Beautiful error reporting with source highlighting
- **clap** - Command-line argument parsing
- **tempfile** - Temporary file handling for compilation pipeline

### Current Language Features

The compiler currently supports:
- Function definitions with `fn name() -> void { ... }` syntax
- `main` function as the program entry point (required)
- Variable declarations with `let name: type = value` syntax
- Integer types: `i32` (32-bit signed), `i64` (64-bit signed)
- Integer literals (e.g., `42`, `0`, `9223372036854775807`)
- Variable references in expressions
- `println("string")` - Print with newline (calls Lak runtime `lak_println`)
- String literals with escape sequences (`\n`, `\t`, `\r`, `\\`, `\"`)
- Line comments (`//`)

### Compilation Flow

1. `build()` in main.rs reads source file
2. `Lexer::tokenize()` produces `Vec<Token>`
3. `Parser::parse()` produces `Program` (AST)
4. `Codegen::compile()` generates LLVM IR
5. `Codegen::write_object_file()` outputs `.o` file
6. System linker (`cc`) produces final executable

### Development Tools

- **lefthook** - Git hooks for pre-commit checks (cargo fmt --check, cargo clippy)
- **mise** - Tool version management
