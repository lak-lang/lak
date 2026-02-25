# resolver Module

Module resolution for multi-file Lak programs.

## Overview

Loads imported modules, parses them, and builds a canonical module graph with cycle
detection. This module is responsible for import-path resolution rules and
module-resolution-specific diagnostics.

## Module Structure

| File | Responsibility |
|------|----------------|
| `mod.rs` | `ModuleResolver`, `ResolvedModule`, cycle tracking, import resolution, module graph construction |
| `error.rs` | `ResolverError`, `ResolverErrorKind`, source-context support for imported-module errors |

## Core Types

| Type | Purpose |
|------|---------|
| `ModuleResolver` | Stateful resolver with module cache and active cycle tracker |
| `ResolvedModule` | Canonical module metadata (`path`, `name`, parsed `Program`, `source`, `resolved_imports`) |
| `CycleTracker` | Maintains stack/index for O(1) cycle checks and cycle message formatting |

## Resolution Behavior

- Entry point: `resolve_from_entry_with_source(entry_path, source)`
- Recursively resolves dependencies with `resolve_module(...)`
- Parses each module via lexer + parser
- Caches modules by canonical absolute path
- Returns sorted modules via `into_modules()`

## Import Path Rules

- Only relative imports are supported (`./` or `../`).
- Import paths must not include file extensions.
- `.lak` extension is added internally before canonicalization.
- Standard-library style imports are currently unsupported and produce
  `StandardLibraryNotSupported`.

## Module Name Rules

- Module names are derived from file stems.
- Names must be valid Lak identifiers (`[A-Za-z_][A-Za-z0-9_]*`, ASCII only).
- Entry modules and imported modules are both validated.
- Utility function `extract_module_name()` applies the same identifier validation.

## Error Model

`ResolverErrorKind` includes:
- `FileNotFound`
- `InvalidImportPath`
- `CircularImport`
- `IoError`
- `InvalidModuleName`
- `LexError`
- `ParseError`
- `StandardLibraryNotSupported`

For lex/parse failures inside imported modules, `ResolverError` can carry
`source_context` (filename + source text) so diagnostics can report the correct file.

## Invariants

- Resolution stack push/pop is balanced even on errors.
- Cycle detection runs before cache checks to report direct/indirect cycles precisely.
- `resolved_imports` stores original import strings mapped to canonical paths.

## Extension Guidelines

1. Keep path canonicalization and cache keys aligned (always canonical absolute paths).
2. Route new resolver diagnostics through `ResolverError` helper constructors.
3. If import schemes are expanded (e.g., standard library), add explicit branch logic and tests.
4. Preserve source-context capture for cross-file lex/parse errors.
5. Add tests for cycle formatting, import validation, and identifier rules when behavior changes.
