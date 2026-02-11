# mut Modifier

## Overview
変数の可変性を制御する `mut` 修飾子を実装する。

### Syntax
```lak
let x = 5                   // Immutable
let mut count = 0           // Mutable
let mut flag: bool = true   // Mutable + explicit type
```

### Rules
- デフォルトは immutable
- `mut` をつけると mutable
- immutable 変数への再代入はコンパイルエラー

