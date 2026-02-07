# for Loop

## Phase
Phase 8: Collections (Lower Priority)

## Overview
`for` ループを実装する。コレクションのイテレーション用。

### Syntax
```lak
for i, item in list {
    println("${i}: ${item}")
}
```

### Iteration Patterns
ループ変数はイテレータの `next` メソッドの戻り値を受け取る。

```lak
// List<T> iterator returns (int, T) - index and element
for i, item in list {
    println("${i}: ${item}")
}

// Discard index with _
for _, item in list {
    println(item)
}

// Map<K, V> iterator returns (K, V)
for key, value in map {
    println("${key}: ${value}")
}

// Single-value iterator (e.g., range)
for i in range(0, 10) {
    println(i)
}
```

### Loop Control
- `break`: ループを終了
- `continue`: 次のイテレーションへ

```lak
for _, item in list {
    if item == 3 {
        continue
    }
    if item == 7 {
        break
    }
    println(item)
}
```

### Rules
- ループ変数は immutable（ループ内で変更不可）

## Dependencies
- List type (phase8-01)
- Map type (phase8-02)
- `while` loop (phase2-04) - similar control flow, break/continue
- Tuple destructuring (phase5-03) - for multi-value iteration

## Dependents
- Iterator protocol (future)
