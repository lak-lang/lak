# codegen Module

LLVM IR generation and native object file output.

## Overview

Transforms Lak AST into LLVM IR and generates native object files. Uses Inkwell (safe Rust bindings for LLVM).

## Module Structure

| File | Responsibility |
|------|----------------|
| `mod.rs` | `Codegen` struct, `compile()`/`compile_modules()` entry points, `mangle_name()`, `derive_mangle_prefix()`, `compute_mangle_prefixes()`, `compute_entry_mangle_prefix()`, `path_components_to_strings()`, `get_mangle_prefix()` |
| `error.rs` | `CodegenError`, `CodegenErrorKind` |
| `binding.rs` | `VarBinding` (stack allocation and type info for variables) |
| `builtins.rs` | Built-in functions (`println` compile-time dispatch to typed `lak_println*` symbols) |
| `expr.rs` | Expression codegen (literals, variable references, calls) |
| `stmt.rs` | Statement codegen (expression/let/assign/discard/return/if/while/break/continue) |
| `target.rs` | Target machine initialization and object file output |
| `tests.rs` | Unit tests |

## Preconditions

**Important**: This module assumes the AST has passed semantic analysis.

- Undefined variables, type mismatches, duplicate variables are already caught
- Violations are handled by returning `CodegenError::InternalError` (not `panic!` or `debug_assert!`)
- Errors in this module are limited to infrastructure issues (LLVM failures, target errors) and module path validation

## Error Types

`CodegenErrorKind`:
- `InternalError` - Invariant violations that indicate compiler bugs (LLVM IR generation failures, module path resolution errors, etc.)
- `TargetError` - Target initialization or object file output failures
- `InvalidModulePath` - Module path validation errors (non-UTF-8 components, duplicate prefixes)

## Lifetime `'ctx`

```rust
pub struct Codegen<'ctx> { ... }
```

`'ctx` is tied to the LLVM `Context`. The context must outlive the `Codegen` instance.

## Variable Management

- `VarBinding` holds stack allocation (`alloca`) and type information
- Managed via scoped stack `variables: Vec<HashMap<String, VarBinding>>`
- Cleared per function
- `StmtKind::Assign` reuses the existing variable binding (`alloca`) and emits `store`
- Mutability is validated in semantic analysis; codegen assumes reassignment is semantically valid

## Type Mapping

| Lak Type | LLVM Type |
|----------|-----------|
| `i8` | `i8` |
| `i16` | `i16` |
| `i32` | `i32` |
| `i64` | `i64` |
| `u8` | `i8` |
| `u16` | `i16` |
| `u32` | `i32` |
| `u64` | `i64` |
| `string` | `ptr` |
| `bool` | `i1` |

## Runtime Integration

- Runtime functions declared as external:
  - `lak_println`
  - `lak_println_i8`, `lak_println_i16`, `lak_println_i32`, `lak_println_i64`
  - `lak_println_u8`, `lak_println_u16`, `lak_println_u32`, `lak_println_u64`
  - `lak_println_bool`
  - `lak_panic`
- Implemented in the `runtime/` crate
- Final binary links against the runtime library

## Generated Code Characteristics

- Uses C calling convention
- `main` function generated with `int main()` signature
- All user-defined functions except entry `main` are emitted with mangled symbol names
- Name mangling applies to both single-file (`compile`) and multi-module (`compile_modules`) paths
- Returns 0 on success

## Extension Guidelines

1. New expressions/statements: add patterns to `expr.rs` / `stmt.rs`
2. New built-in functions: add to `builtins.rs` with `declare_*` and `generate_*`, and add the function name to `BUILTIN_NAMES`
3. New types: add to `get_llvm_type()` and `VarBinding::new()`
4. Return `CodegenError::InternalError` for conditions guaranteed by semantic analysis (not `debug_assert!` or `panic!`)
