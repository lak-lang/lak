# codegen Module

LLVM IR generation and native object file output.

## Overview

Transforms Lak AST into LLVM IR and generates native object files. Uses Inkwell (safe Rust bindings for LLVM).

## Module Structure

| File | Responsibility |
|------|----------------|
| `mod.rs` | `Codegen` struct, `compile()` entry point |
| `error.rs` | `CodegenError`, `CodegenErrorKind` |
| `binding.rs` | `VarBinding` (stack allocation and type info for variables) |
| `builtins.rs` | Built-in functions (`println` → `lak_println`) |
| `expr.rs` | Expression codegen (literals, variable references, calls) |
| `stmt.rs` | Statement codegen (expression statements, `let` statements) |
| `target.rs` | Target machine initialization and object file output |
| `tests.rs` | Unit tests |

## Preconditions

**Important**: This module assumes the AST has passed semantic analysis.

- Undefined variables, type mismatches, duplicate variables are already caught
- Violations are handled with `debug_assert!` or `panic!` (indicating compiler bugs)
- Runtime errors are limited to infrastructure errors (LLVM failures, target errors)

## Error Types

`CodegenErrorKind`:
- `InternalError` - LLVM IR generation failures (compiler bug)
- `TargetError` - Target initialization or object file output failures

## Lifetime `'ctx`

```rust
pub struct Codegen<'ctx> { ... }
```

`'ctx` is tied to the LLVM `Context`. The context must outlive the `Codegen` instance.

## Variable Management

- `VarBinding` holds stack allocation (`alloca`) and type information
- Managed via `variables: HashMap<String, VarBinding>`
- Cleared per function (currently only `main`)

## Type Mapping

| Lak Type | LLVM Type |
|----------|-----------|
| `i32` | `i32` |
| `i64` | `i64` |

## Runtime Integration

- `lak_println` declared as external function
- Implemented in the `runtime/` crate
- Final binary links against the runtime library

## Generated Code Characteristics

- Uses C calling convention
- `main` function generated with `int main()` signature
- Returns 0 on success

## Extension Guidelines

1. New expressions/statements: add patterns to `expr.rs` / `stmt.rs`
2. New built-in functions: add to `builtins.rs` with `declare_*` and `generate_*`
3. New types: add to `get_llvm_type()` and `VarBinding::new()`
4. Use `debug_assert!` for conditions guaranteed by semantic analysis
