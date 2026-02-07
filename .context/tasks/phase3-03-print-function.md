# print Built-in Function

## Phase
Phase 3: Functions Enhancement (High Priority)

## Overview
改行なしで出力する `print` 組み込み関数を実装する。

### Signature
```lak
fn print(value: any) -> void
```

### Features
- `println` と同様だが改行なし
- `any` 型を受け取る（全ての型を暗黙的に変換可能）
- 型に応じたデフォルトフォーマットで出力

### Examples
```lak
print("hello")
print(42)
print(true)
```

## Dependencies
- `println` function (completed)
- `any` type (completed)

## Dependents
- String interpolation (phase9-03) - formatted output
