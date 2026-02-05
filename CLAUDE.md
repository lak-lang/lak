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

# Run tests
cargo test

# Run a single test
cargo test <test_name>

# Run tests for a specific module
cargo test lexer::tests
cargo test parser::tests

# Run integration tests only
cargo test --test integration

# Format code
cargo fmt

# Lint code
cargo clippy
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
- `compiler/src/lexer.rs` - Lexical analysis, produces token stream from source text
- `compiler/src/parser.rs` - Recursive descent parser, builds AST from tokens
- `compiler/src/ast.rs` - AST node definitions (`Program`, `Stmt`, `Expr`)
- `compiler/src/codegen.rs` - LLVM IR generation using inkwell, outputs native object files
- `compiler/tests/integration.rs` - End-to-end integration tests (compile and run)
- `runtime/src/lib.rs` - Lak runtime library providing I/O functions (`lak_println`)

### Key Dependencies

- **inkwell** - Safe Rust bindings to LLVM (using LLVM 21.1)
- **ariadne** - Beautiful error reporting with source highlighting
- **clap** - Command-line argument parsing

### Current Language Features

The compiler currently supports:
- Function definitions with `fn name() -> void { ... }` syntax
- `main` function as the program entry point (required)
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

