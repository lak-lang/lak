# lexer Module

Lexical analysis (tokenization) of Lak source code.

## Overview

Converts source code text into a stream of `Token`s for the parser. Handles character-by-character scanning, source position tracking, and automatic newline insertion.

## Module Structure

| File | Responsibility |
|------|----------------|
| `mod.rs` | `Lexer` struct, `tokenize()` entry point |
| `error.rs` | `LexError` type |
| `cursor.rs` | Position tracking (`current_char`, `advance`, `is_eof`) |
| `skip.rs` | Whitespace/comment skipping, newline emission logic |
| `tokens.rs` | Token recognition (`next_token`, `read_*` methods) |
| `tests.rs` | Unit tests |

## Supported Tokens

| Category | Tokens |
|----------|--------|
| Keywords | `fn`, `let` |
| Identifiers | Unicode alphabetic/underscore start, alphanumeric/underscore continue |
| Integer literals | ASCII digit sequences, stored as `i64` (overflow → error) |
| String literals | Double-quoted, supports `\n`, `\t`, `\r`, `\\`, `\"` |
| Punctuation | `(`, `)`, `{`, `}`, `,`, `:`, `=`, `->` |
| Special | `Newline` (auto-inserted), `Eof` |

## Automatic Newline Insertion

Inspired by Go's semicolon insertion. `Newline` tokens are emitted after:
- Identifiers
- Literals (string, integer)
- `)` (right parenthesis)
- `}` (right brace)

This enables statement termination without explicit semicolons.

## Error Types

`LexError` contains:
- `message: String` - Human-readable description
- `span: Span` - Source location

Errors occur for:
- Unexpected characters
- Unterminated string literals
- Unknown escape sequences
- Integer overflow (exceeds `i64::MAX`)
- Invalid `-` usage (only `->` is valid)

## Lifetime `'a`

```rust
pub struct Lexer<'a> { ... }
```

`'a` ties the lexer to the input string slice. The input must remain valid while tokenizing.

## Position Tracking

- `pos: usize` - Byte position in input
- `line: usize` - Line number (1-indexed)
- `column: usize` - Column number (1-indexed)
- Handles multi-byte UTF-8 characters correctly

## Extension Guidelines

1. New keywords: add pattern in `read_identifier()` in `tokens.rs`
2. New punctuation: add case in `next_token()` in `tokens.rs`
3. New literal types: add `read_*` method in `tokens.rs`
4. Newline insertion rules: modify `should_emit_newline()` in `skip.rs`
