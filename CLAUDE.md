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
├── .context/          # Language specification documents
│   └── SPEC.md        # Lak language specification (for AI agents)
├── compiler/          # Main compiler crate (see compiler/CLAUDE.md)
├── runtime/           # Lak runtime library (see runtime/CLAUDE.md)
└── examples/          # Example .lak programs
```

## Language Specification

The complete Lak language specification is documented in `.context/SPEC.md`. Refer to this document for:
- Type system (primitives, tuples, collections, generics)
- Variable declarations and mutability
- Functions, structs, interfaces, and enums
- Control flow (if, for, while, match)
- Error handling (Option, Result, panic)
- Module system

## Architecture

The compiler follows a traditional pipeline:

```
Source (.lak) → Lexer → Parser → Semantic Analyzer → Codegen → LLVM → Object File → Linker → Executable
```

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
4. `SemanticAnalyzer::analyze()` validates the AST
5. `Codegen::compile()` generates LLVM IR
6. `Codegen::write_object_file()` outputs `.o` file
7. System linker (`cc`) produces final executable

### Development Tools

- **lefthook** - Git hooks for pre-commit checks (cargo fmt --check, cargo clippy)
- **mise** - Tool version management

## Commit Message Convention

Use the following prefixes based on the type of change:

- **Affects application behavior**: `fix:`, `feat:`
- **Does not affect application behavior**: `ci:`, `chore:`, `docs:`
