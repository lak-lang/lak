# semantic Module

Semantic analysis for the Lak programming language.

## Overview

Validates an AST for semantic correctness before code generation. Performs name resolution, type checking, and structural validation. If analysis succeeds, the AST is guaranteed to be semantically valid.

## Module Structure

| File | Responsibility |
|------|----------------|
| `mod.rs` | `SemanticAnalyzer` struct, `analyze()`, `analyze_with_modules()`, `analyze_module()` entry points |
| `error.rs` | `SemanticError`, `SemanticErrorKind` |
| `symbol.rs` | `SymbolTable`, `FunctionInfo`, `VariableInfo` |
| `module_table.rs` | `ModuleTable`, `ModuleExports`, `FunctionExport` (import tracking) |
| `tests/` | Unit tests (see below) |

### Test Structure

| File | Coverage |
|------|----------|
| `tests/mod.rs` | Test helpers, SemanticError Display tests |
| `tests/function_tests.rs` | Function definition, main validation, undefined function, scope isolation |
| `tests/variable_tests.rs` | Duplicate/undefined variable detection, mutable declaration path |
| `tests/type_tests.rs` | Type mismatch, overflow, invalid expressions, println arguments, valid programs |
| `tests/symbol_table_tests.rs` | SymbolTable data structure unit tests, mutable flag preservation |

## Analysis Phases

The analyzer runs in three sequential phases:

1. **Function collection**: Gather all function definitions, check for duplicates
2. **Main validation**: Verify `main` exists and has correct signature (`-> void`)
3. **Body analysis**: Analyze each function body (variables, types, expressions)

Note: `analyze_module()` (used for imported modules) skips phase 2 (main validation) since imported modules are libraries, not entry points.

## Error Types

`SemanticErrorKind`:

| Kind | Description |
|------|-------------|
| `DuplicateFunction` | Function defined multiple times |
| `DuplicateVariable` | Variable defined multiple times in same scope |
| `UndefinedVariable` | Variable used but not defined |
| `UndefinedFunction` | Function called but not defined |
| `TypeMismatch` | Expected vs actual type mismatch |
| `IntegerOverflow` | Integer out of range for target type |
| `InvalidArgument` | Wrong argument count or type |
| `InvalidControlFlow` | Invalid loop control usage (e.g., break/continue outside loops) |
| `InvalidExpression` | Expression in invalid context (e.g., literal as statement) |
| `MissingMainFunction` | No main function found |
| `InvalidMainSignature` | Main has wrong signature |
| `InternalError` | Compiler bug (should never occur in normal operation) |
| `ModuleAccessNotImplemented` | Non-call member access on modules not yet implemented |
| `ModuleNotImported` | Module-qualified function call requires an import |
| `UndefinedModule` | Module not found (not imported) |
| `UndefinedModuleFunction` | Function not found in module |
| `DuplicateModuleImport` | Duplicate module import |
| `CrossModuleCallInImportedModule` | Cross-module call in imported module not supported |

## Symbol Table

`SymbolTable` provides:
- Global function namespace (flat)
- Scoped variable lookup (stack-based)

`VariableInfo` carries:
- `name`
- `is_mutable` (`let mut` declarations)
- `ty`
- `definition_span`

### Scope Management

```rust
symbols.enter_scope();   // Enter function body
// ... analyze statements ...
symbols.exit_scope();    // Exit function body
```

Variables are looked up from innermost to outermost scope.

## Type Checking

Currently supports:
- `i32`: values must be in range `i32::MIN..=i32::MAX`
- `i64`: all lexer-parsed integers are valid
- `string`: string literals
- `bool`: boolean literals

Type checking occurs in:
- `let` statements (`let` / `let mut`): initializer must match declared type
- Variable references: variable type must match expected type

## Built-in Functions

- `println`: Requires exactly 1 argument (string, i32, i64, or bool)
- `panic`: Requires exactly 1 argument (string only)
- `println` / `panic` are reserved prelude names and cannot be redefined by user functions

User-defined function calls are validated for: existence, zero arguments (parameters not yet supported), non-main target, and void return type.

## Expression Statement Validation

Only function calls and module-qualified function calls are valid as expression statements. The following are rejected:
- String literals (no effect)
- Integer literals (no effect)
- Boolean literals (no effect)
- Bare identifiers (no effect)
- Binary operations (no effect)
- Unary operations (no effect)
- Member access (only module function calls are supported, not general member access)

## Guarantees to Codegen

If `analyze()` returns `Ok(())`, codegen can assume:
- `main` function exists with `void` return type
- All variable references are defined
- All variable types match their usage
- All integer literals fit their target types
- All function calls reference defined functions
- All module references are valid (imported and resolved)
- All module function calls reference existing public functions

## Extension Guidelines

1. New types: add case in `check_integer_range()` and `check_expr_type()`
2. New built-in functions: add case in `analyze_call()`
3. New expression forms: add case in `check_expr_type()` and `analyze_expr_stmt()`
4. New statement forms: add case in `analyze_stmt()`
5. Scoping changes: modify `enter_scope()` / `exit_scope()` in `symbol.rs`
