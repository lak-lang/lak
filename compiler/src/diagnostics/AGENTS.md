# diagnostics Module

Compiler diagnostic rendering and error reporting.

## Overview

Converts structured compile errors into user-facing diagnostics. Uses `ariadne`
for source-highlighted reports and provides plain-text fallbacks if rendering fails.

## Module Structure

| File | Responsibility |
|------|----------------|
| `mod.rs` | `report_error()` entry point, ariadne report construction, fallback reporting |

## Reporting Flow

`report_error(filename, source, error)` dispatches by `CompileError` variant:
- `Resolve(ResolverError)`: reports using module source context when available
- `Semantic(SemanticError)`: reports entry-module semantic errors
- `ModuleSemantic(...)`: reports semantic errors in imported modules
- `Codegen(CodegenError)`: reports codegen errors with optional span
- `Link` and infrastructure errors: prints concise plain-text messages

## Key Helpers

| Helper | Purpose |
|--------|---------|
| `print_range_report()` | Builds and prints an ariadne error report for a byte range |
| `report_semantic_error()` | Semantic-specific reporting, including no-span fallback |
| `semantic_no_span_label_message()` | Missing-main specific label wording |
| `semantic_no_span_help_message()` | Missing-main specific help wording |
| `end_of_source_range()` | Stable fallback span when no source location is available |

## Span and Source Rules

- Ariadne is configured with byte indexing (`IndexType::Byte`).
- For imported-module resolver errors, prefer `source_filename()` / `source_content()`
  from `ResolverError`.
- For semantic errors without spans, anchor diagnostics at end-of-source.
- If ariadne printing fails, emit plain-text fallback with message and help text.

## Extension Guidelines

1. Keep rendering and fallback behavior consistent across error kinds.
2. Add kind-specific wording through helpers instead of branching inline in multiple places.
3. Preserve source-context routing for imported-module errors.
4. Add unit tests for no-span behavior and helper message rules when changing semantics.
