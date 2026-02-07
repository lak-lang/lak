# Comparison Operators

## Phase
Phase 1: Basic Expressions (High Priority)

## Overview
比較演算子 (`==`, `!=`, `<`, `>`, `<=`, `>=`) を実装する。

これらの演算子は2つのオペランドを比較し、`bool` 型の結果を返す。数値型（i32, i64など）の比較をサポートする。

### Operators
| Operator | Description |
|----------|-------------|
| `==` | Equal |
| `!=` | Not equal |
| `<` | Less than |
| `>` | Greater than |
| `<=` | Less than or equal |
| `>=` | Greater than or equal |

### Precedence
- `<`, `>`, `<=`, `>=`: Level 4
- `==`, `!=`: Level 5

## Dependencies
- `bool` type (completed)
- i32/i64 types (completed)

## Dependents
- `if`/`else` statement (phase2-01)
- `while` loop (phase2-04)
- `match` expression (phase6-03)
