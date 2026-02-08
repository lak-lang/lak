# compiler Crate

The main Lak compiler crate. Provides both a CLI tool and a library for compiling Lak source code to native executables.

## Crate Structure

```
compiler/
├── src/
│   ├── main.rs      # CLI entry point (lak build, lak run)
│   ├── lib.rs       # Library crate exposing all modules
│   ├── token/       # Token and Span definitions (see token/CLAUDE.md)
│   ├── ast/         # AST node definitions (see ast/CLAUDE.md)
│   ├── lexer/       # Lexical analysis (see lexer/CLAUDE.md)
│   ├── parser/      # Parsing (see parser/CLAUDE.md)
│   ├── semantic/    # Semantic analysis (see semantic/CLAUDE.md)
│   ├── resolver/    # Module resolution (multi-file compilation)
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
    ↓ ModuleResolver::resolve_from_entry_with_source() (always invoked; discovers imported modules if present)
Vec<ResolvedModule> (all modules with ASTs)
    ↓ SemanticAnalyzer::analyze() / analyze_with_modules()
Program (validated)
    ↓ Codegen::compile() / compile_modules()
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
| `resolver` | Module resolution, dependency graph, cycle detection |
| `codegen` | AST → LLVM IR → Object file |

## Error Handling

### Error Type Hierarchy

`CompileError` in `main.rs` unifies all error types:
- `Resolve(ResolverError)` - Module resolution errors
- `Semantic(SemanticError)` - Semantic analysis errors
- `ModuleSemantic(...)` - Semantic errors in imported modules (with source context)
- `Codegen(CodegenError)` - Code generation errors
- `Link(LinkError)` - Linker errors
- `PathNotUtf8`, `FileReadError`, `PathResolutionError`, `TempDirCreationError`, `ExecutableRunError`, `EntryModuleNotFound`, `FilenameError` - Infrastructure errors (typed variants, not generic strings)

Phase-specific errors may include `Span` for source location (see details below). `report_error()` in `main.rs` uses ariadne for formatted output.

### Error Type Structure

Each phase-specific error type (`LexError`, `ParseError`, `SemanticError`, `CodegenError`, `ResolverError`) follows a consistent pattern:

- **Private fields** with accessor methods: `message()`, `span()`, `kind()`, `short_message()`. `span()` returns `Option<Span>` for `SemanticError`, `CodegenError`, `ResolverError`; returns `Span` (always present) for `LexError`, `ParseError`. `SemanticError` additionally provides `help()`
- **ErrorKind enum** for structured error matching: `LexErrorKind`, `ParseErrorKind`, `SemanticErrorKind`, `CodegenErrorKind`, `ResolverErrorKind`
- **Base constructors** (internal to `error.rs`): `Error::new(kind, message, span)`, `without_span()`. `SemanticError` also provides `new_with_help()`
- **Helper constructors** (public API): descriptive methods like `undefined_variable(name, span)`

```rust
// Matching on error kind (not fragile string matching)
match err.kind() {
    LexErrorKind::UnterminatedString => { /* handle */ }
    LexErrorKind::IntegerOverflow => { /* handle */ }
    _ => { /* other cases */ }
}
```

### Helper Constructor Pattern

**All error messages must be defined in `error.rs` via helper constructors.** Call sites must not construct error messages directly.

This pattern ensures:
- Error messages are centralized and consistent
- Message wording can be changed in one place
- No risk of format divergence across call sites

```rust
// GOOD: Use helper constructor
return Err(SemanticError::undefined_variable(name, span));
return Err(CodegenError::internal_entry_module_not_found(&entry_path));

// BAD: Direct constructor with inline message
return Err(SemanticError::new(
    SemanticErrorKind::UndefinedVariable,
    format!("Undefined variable: '{}'", name),
    span,
));

// BAD: direct struct construction instead of helper constructor
return Err(CompileError::FileReadError { path: path.into(), source: io_err });
```

**Rules for call sites:**
1. Never call `Error::new()`, `without_span()`, or `new_with_help()` outside of `error.rs`
2. Never use `format!` to construct error messages outside of `error.rs`
3. If no suitable helper exists, add one to `error.rs` first, then use it

**Helper organization in `error.rs`:**
Helpers are grouped by category using section comments:
```rust
// =========================================================================
// Name resolution errors
// =========================================================================
pub fn undefined_variable(name: &str, span: Span) -> Self { ... }
pub fn undefined_function(name: &str, span: Span) -> Self { ... }

// =========================================================================
// Type errors
// =========================================================================
pub fn type_mismatch_int_to_string(value: i64, span: Span) -> Self { ... }
```

### CompileError Design

`CompileError` in `main.rs` uses **typed variants** instead of generic string wrappers. Each variant carries structured data relevant to the error:

```rust
// GOOD: Typed variant with structured data
CompileError::FileReadError { path, source }
CompileError::PathNotUtf8 { path, context }

// BAD: Generic string wrapper (removed)
CompileError::Io(format!("Failed to read {}: {}", path, err))
```

`CompileError` also provides helper constructors:
```rust
CompileError::file_read_error(&path, io_error)
CompileError::path_not_utf8(path, "object file")
```

### Error Message Conventions

**Message levels** (used by ariadne error reporting):

| Level | Method | Purpose | Example |
|-------|--------|---------|---------|
| Title | `short_message()` | Brief error category (report header) | `"Type mismatch"` |
| Label | `message()` | Detailed explanation (shown at source location) | `"Type mismatch: variable 'x' has type 'string', expected 'i32'"` |
| Help | `help()` | Fix suggestion (optional) | `"arithmetic operators only work with numeric types"` |

**Formatting rules:**
- Never use `{:?}` (Rust Debug format) in user-facing messages. Use `Display` or manual formatting (e.g., `join(", ")`)
- Internal errors must include `"This is a compiler bug."` suffix
- Use single quotes for identifiers in messages: `'variable_name'`, `'i32'`

### Adding New Errors

1. **Add ErrorKind variant** to the `ErrorKind` enum in `error.rs`
2. **Add `short_message()` arm** for the new variant
3. **Add helper constructor** in the appropriate category section
4. **Use the helper** at the call site
5. **Add tests** that verify both `kind()` and `message()` of the new error

## Build Script (build.rs)

Sets `LAK_RUNTIME_PATH` environment variable at compile time, pointing to `liblak_runtime.a`. This path is used by the linker to link generated programs with the runtime.

## Tests

| File | Coverage |
|------|----------|
| `e2e_basic.rs` | Basic compilation tests |
| `e2e_strings.rs` | String literal handling |
| `e2e_variables.rs` | Variable declarations |
| `e2e_run.rs` | `lak run` command |
| `e2e_any.rs` | Mixed type println tests |
| `e2e_arithmetic.rs` | Arithmetic operations and division-by-zero |
| `e2e_functions.rs` | User-defined function calls |
| `e2e_panic.rs` | `panic()` function behavior |
| `e2e_bool.rs` | Boolean type handling |
| `e2e_visibility.rs` | `pub fn` visibility keyword |
| `e2e_imports.rs` | `import` syntax parsing |
| `e2e_modules.rs` | Multi-file module compilation |
| `e2e_build.rs` | `lak build` command behavior |
| `errors_lex.rs` | Lexer error cases |
| `errors_parse.rs` | Parser error cases |
| `errors_semantic.rs` | Semantic error cases |
| `errors_modules.rs` | Module resolution error cases |
| `pipeline.rs` | Full pipeline tests |
| `common/mod.rs` | Shared test utilities |

### Testing Conventions

- **ariadne output**: Tests that verify ariadne stderr output (e.g., `errors_modules.rs`, `e2e_run.rs`, `e2e_build.rs`) use `.contains()` to verify exact substrings of each output element — short titles (e.g., `\x1b[31mError:\x1b[0m Missing main function`), labels, and help text. Full output exact matching is impractical due to ariadne's formatting, but each verified substring must be an exact match including ANSI escape codes.
- **Error messages**: Tests that verify error messages from `compile_error_with_kind()` should use `assert_eq!()` for exact matching, not `contains()`.

## Coding Guidelines

### Error Handling

- **Avoid `panic!`**: Return `Result` with appropriate error types instead. Even for "impossible" states that semantic analysis should prevent, return an `InternalError` with a helpful message rather than panicking.
- **Avoid `debug_assert!` for invariant checks**: Use runtime checks that return errors. `debug_assert!` is compiled out in release builds, which can hide bugs.
- **Exception**: `panic!` is acceptable in tests.

## Extension Guidelines

1. New CLI commands: add variant to `Commands` enum in `main.rs`
2. New AST nodes: add to appropriate file in `ast/`, update parser and codegen
3. New token types: add to `token.rs`, update lexer
4. Error handling: use appropriate error type with `Span`
