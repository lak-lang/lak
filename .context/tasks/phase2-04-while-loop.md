# while Loop with break/continue

## Phase
Phase 2: Control Flow (High Priority)

## Overview
`while` ループと `break`/`continue` 文を実装する。

### Syntax
```lak
while condition {
    // ...
}
```

### Features
- 条件が true の間繰り返す
- `while true { ... }` で無限ループ
- `break` でループを終了
- `continue` で次のイテレーションへ

### Examples
```lak
// 条件付きループ
while condition {
    // ...
}

// 無限ループ
while true {
    if should_stop {
        break
    }
}

// continue
while condition {
    if skip_this {
        continue
    }
    // ...
}
```

## Dependencies
- `bool` type (completed)
- Comparison operators (phase1-03) - for loop conditions
- Logical operators (phase1-04) - for complex conditions

## Dependents
- `for` loop (phase8-03) - similar loop control flow
