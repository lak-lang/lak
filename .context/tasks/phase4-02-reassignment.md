# Reassignment for Mutable Variables

## Phase
Phase 4: Variables Enhancement (Medium Priority)

## Overview
`mut` で宣言された変数への再代入を実装する。

### Syntax
```lak
let mut count = 0
count = count + 1
count = 10
```

### Rules
- `mut` で宣言された変数のみ再代入可能
- immutable 変数への再代入はコンパイルエラー
- 再代入時の型は元の型と一致する必要がある

### Examples
```lak
let x = 5
x = 10              // Compile error: x is immutable

let mut y = 5
y = 10              // OK

let mut z: int = 5
z = "hello"         // Compile error: type mismatch
```

## Dependencies
- `mut` modifier (phase4-01)

## Dependents
- Collection mutability (future)
- Loop counter patterns
