# Struct Definition

## Phase
Phase 7: User-Defined Types (Medium Priority)

## Overview
構造体の定義を実装する。

### Syntax
```lak
struct User {
    pub name: string
    age: int          // Private (default)
}
```

### Visibility
- フィールドはデフォルトでprivate
- `pub` キーワードで公開

### Rules
- フィールドは名前と型を持つ
- 同じフィールド名は使用できない
- 構造体自体も `pub` で公開可能

```lak
pub struct User {
    pub name: string
    age: int
}
```

## Dependencies
- Type system basics (completed)
- `pub` visibility keyword (phase1-08)

## Dependents
- Struct instantiation (phase7-02)
- Field access (phase7-03)
- Methods (phase7-04)
