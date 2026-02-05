# compiler Crate

The main Lak compiler crate. Provides both a CLI tool and a library for compiling Lak source code to native executables.

## Crate Structure

```
compiler/
├── src/
│   ├── main.rs      # CLI entry point (lak build, lak run)
│   ├── lib.rs       # Library crate exposing all modules
│   ├── token.rs     # Token and Span definitions
│   ├── ast.rs       # AST node definitions
│   ├── lexer/       # Lexical analysis (see lexer/CLAUDE.md)
│   ├── parser/      # Parsing (see parser/CLAUDE.md)
│   ├── semantic/    # Semantic analysis (see semantic/CLAUDE.md)
│   └── codegen/     # LLVM codegen (see codegen/CLAUDE.md)
├── tests/           # Integration and E2E tests
├── build.rs         # Build script for runtime path
└── Cargo.toml
```

## CLI Commands

```bash
lak build <file.lak>              # Compile to executable
lak build <file.lak> -o <output>  # Specify output path
lak run <file.lak>                # Compile and run
```

## Compilation Pipeline

```
Source (.lak)
    ↓ Lexer::tokenize()
Vec<Token>
    ↓ Parser::parse()
Program (AST)
    ↓ SemanticAnalyzer::analyze()
Program (validated)
    ↓ Codegen::compile()
LLVM IR
    ↓ Codegen::write_object_file()
Object file (.o)
    ↓ cc (system linker)
Executable
```

## Key Dependencies

| Dependency | Purpose |
|------------|---------|
| `inkwell` | LLVM bindings (LLVM 21.1) |
| `ariadne` | Beautiful error reporting |
| `clap` | CLI argument parsing |
| `tempfile` | Temporary files for `lak run` |

## Core Modules

| Module | Purpose |
|--------|---------|
| `token` | `Token`, `TokenKind`, `Span` definitions |
| `ast` | `Program`, `FnDef`, `Stmt`, `Expr`, `Type` |
| `lexer` | Source → Token stream |
| `parser` | Token stream → AST |
| `semantic` | AST validation |
| `codegen` | AST → LLVM IR → Object file |

## Error Handling

`CompileError` unifies all error types:
- `LexError` - Lexical analysis errors
- `ParseError` - Parsing errors
- `SemanticError` - Semantic analysis errors
- `CodegenError` - Code generation errors

All errors include `Span` for source location. `report_error()` uses ariadne for beautiful output.

## Build Script (build.rs)

Sets `LAK_RUNTIME_PATH` environment variable at compile time, pointing to `liblak_runtime.a`. This path is used by the linker to link generated programs with the runtime.

## Tests

| File | Coverage |
|------|----------|
| `e2e_basic.rs` | Basic compilation tests |
| `e2e_strings.rs` | String literal handling |
| `e2e_variables.rs` | Variable declarations |
| `e2e_run.rs` | `lak run` command |
| `errors_lex.rs` | Lexer error cases |
| `errors_parse.rs` | Parser error cases |
| `errors_semantic.rs` | Semantic error cases |
| `pipeline.rs` | Full pipeline tests |
| `common/mod.rs` | Shared test utilities |

## Extension Guidelines

1. New CLI commands: add variant to `Commands` enum in `main.rs`
2. New AST nodes: add to `ast.rs`, update parser and codegen
3. New token types: add to `token.rs`, update lexer
4. Error handling: use appropriate error type with `Span`
