# token Module

Token definitions and source location tracking for the Lak compiler.

## Overview

Provides the fundamental token types used throughout the compiler pipeline. Defines source location tracking (`Span`), token classification (`TokenKind`), and token representation (`Token`).

## Module Structure

| File | Responsibility |
|------|----------------|
| `mod.rs` | Module root, `Token` struct, re-exports all public types |
| `span.rs` | `Span` struct for source location tracking |
| `kind.rs` | `TokenKind` enum for token classification |

## Types

### Span

Source location tracking with byte offsets and human-readable positions.

| Field | Type | Description |
|-------|------|-------------|
| `start` | `usize` | Starting byte offset (inclusive). Must be UTF-8 boundary. |
| `end` | `usize` | Ending byte offset (exclusive). Must be UTF-8 boundary. |
| `line` | `usize` | Line number (1-indexed) |
| `column` | `usize` | Column number (1-indexed) |

Used in:
- **All modules**: lexer, parser, AST nodes, semantic analysis, codegen, all error types
- Note: Post-parsing phases (AST, semantic, codegen) import only `Span`, not `Token`/`TokenKind`

### TokenKind

Token classification enum.

| Category | Variants |
|----------|----------|
| Keywords | `Fn`, `Let` |
| Identifiers | `Identifier(String)` |
| Literals | `StringLiteral(String)`, `IntLiteral(i64)` |
| Punctuation | `LeftParen`, `RightParen`, `LeftBrace`, `RightBrace`, `Comma`, `Colon`, `Equals`, `Arrow` |
| Special | `Newline`, `Eof` |

Used in: lexer (creation), parser (consumption).

### Token

Combines `TokenKind` with `Span`.

| Field | Type | Description |
|-------|------|-------------|
| `kind` | `TokenKind` | What type of token |
| `span` | `Span` | Where in source |

Used in: lexer (creation), parser (consumption).

## Usage Patterns

- **Span-only imports**: semantic, codegen, AST (post-parsing phases)
- **All types**: lexer, parser (token processing phases)

## Extension Guidelines

1. New token kinds: add variant to `TokenKind` in `kind.rs`
2. New span features: modify `Span` in `span.rs`
3. All types remain re-exported from `mod.rs` for backward compatibility
