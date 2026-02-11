# List Type

## Overview
リスト型を実装する。

### Syntax
```lak
let numbers = [1, 2, 3]                // Inferred as List<int>
let names: List<string> = []           // Empty list requires type annotation
```

### Element Access
インデックス構文 (`[]`) はなし。メソッドでアクセス。

```lak
let numbers = [1, 2, 3]
let first = numbers.get(0)             // Option<int>
let sub = numbers.slice(0, 2)          // [1, 2]
```

### Mutability
コンテンツを変更するには `mut` で宣言。

```lak
let numbers = [1, 2, 3]
numbers.push(4)                        // Compile error: immutable

let mut items = [1, 2, 3]
items.push(4)                          // OK
```

### Methods
- `get(index: int) -> Option<T>`
- `slice(start: int, end: int) -> List<T>`
- `push(item: T)` (requires mut)
- `len() -> int`

