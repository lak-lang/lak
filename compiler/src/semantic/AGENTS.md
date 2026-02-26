# semantic Module

Semantic analysis for the Lak programming language.

## Overview

Validates an AST for semantic correctness before code generation. Performs name resolution, type checking, and structural validation. If analysis succeeds, the AST is guaranteed to be semantically valid.

## Module Structure

| File | Responsibility |
|------|----------------|
| `mod.rs` | Thin facade: `SemanticAnalyzer` struct, session lifecycle, public entry points |
| `error.rs` | `SemanticError`, `SemanticErrorKind` |
| `symbol.rs` | `SymbolTable`, `FunctionInfo`, `VariableInfo` |
| `symbols.rs` | Function collection, main signature validation, call target resolution |
| `typecheck_stmt.rs` | Statement analysis (`analyze_stmt` family) |
| `typecheck_expr.rs` | Expression typing/inference (`check_expr_type`, `infer_expr_type`) |
| `module_table.rs` | `ModuleTable`, `ModuleExports`, `FunctionExport` (import tracking) |
| `tests/` | Unit tests (see below) |

### Test Structure

| File | Coverage |
|------|----------|
| `tests/mod.rs` | Test helpers, SemanticError Display tests |
| `tests/function_tests.rs` | Function definition, main validation, undefined function, scope isolation |
| `tests/variable_tests.rs` | Duplicate/undefined variable detection, mutable declaration/reassignment paths |
| `tests/type_tests.rs` | Type mismatch, overflow, invalid expressions, println arguments, valid programs |
| `tests/symbol_table_tests.rs` | SymbolTable data structure unit tests, mutable flag preservation |
| `tests/reuse_regression_tests.rs` | Regression coverage for analyzer/session state reuse |

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
| `ImmutableVariableReassignment` | Reassignment to immutable variable |
| `UndefinedFunction` | Function called but not defined |
| `TypeMismatch` | Expected vs actual type mismatch |
| `IfExpressionBranchTypeMismatch` | `if` expression branches yield different types |
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
- Signed integers: `i8`, `i16`, `i32`, `i64`
- Unsigned integers: `u8`, `u16`, `u32`, `u64`
- Floating-point: `f32`, `f64`
- `byte` alias (normalized to `u8` in parsing)
- Integer literal adaptation to contextual integer type with per-type range checks
- Float literal adaptation (`f64` literal with contextual `f32` checks)
- Mixed float arithmetic/comparison (`f32` + `f64`) with widening to `f64`
- Integer/float mixed arithmetic and comparison are rejected without explicit casts
- Float modulo (`%`) is rejected
- `string`: string literals
- `bool`: boolean literals
- `let` inference: for `let x = expr` / `let mut x = expr`, `infer_expr_type` determines the concrete type (`i64` for integers, `f64` for floats)

Type checking occurs in:
- `let` statements (`let` / `let mut`): when an explicit type exists, the initializer must match it; otherwise the type is inferred from the initializer
- reassignment statements (`x = expr`): variable must be mutable and RHS must match variable type
- Variable references: variable type must match expected type

## Built-in Functions

- `println`: Requires exactly 1 argument (string, bool, integer, or float types)
- `panic`: Requires exactly 1 argument (string only)
- `println` / `panic` are reserved prelude names and cannot be redefined by user functions

User-defined function calls are validated for: existence, argument count/type
matching, non-main target, and return-type compatibility at use sites.

## Expression Statement Validation

Only function calls and module-qualified function calls are valid as expression statements. The following are rejected:
- String literals (no effect)
- Integer literals (no effect)
- Float literals (no effect)
- Boolean literals (no effect)
- Bare identifiers (no effect)
- Binary operations (no effect)
- Unary operations (no effect)
- Member access (only module function calls are supported, not general member access)

## Guarantees to Codegen

If `analyze()` returns `Ok(())`, codegen can assume:
- `main` function exists, has zero parameters, and has `void` return type
- All variable references are defined
- Reassignments target mutable variables only
- All variable types match their usage
- All inferred `let` bindings are resolved to concrete types in semantic symbol metadata and exported via `SemanticAnalyzer::inferred_binding_types()` for strict codegen paths
- All integer literals fit their target types
- All function calls reference defined functions with compatible argument and return types
- All non-void functions satisfy return-path requirements
- All module references are valid (imported and resolved)
- All module function calls reference existing public functions

## Extension Guidelines

1. New types: update `check_expr_type()`, `infer_expr_type()`, operator type rules, and (for integers) `check_integer_range()`
2. New built-in functions: add validation in `analyze_call_stmt()` / `analyze_call_value()`
3. New expression forms: add case in `check_expr_type()` and `analyze_expr_stmt()`
4. New statement forms: add case in `analyze_stmt()`
5. Scoping changes: modify `enter_scope()` / `exit_scope()` in `symbol.rs`
