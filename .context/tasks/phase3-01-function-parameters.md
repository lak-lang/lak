# Function Parameters

## Phase
Phase 3: Functions Enhancement (High Priority)

## Overview
関数パラメータのサポートを実装する。

### Syntax
```lak
fn add(a: int, b: int) -> int {
    return a + b
}

fn greet(name: string) -> void {
    println(name)
}
```

### Features
- 単一パラメータ: `(name: type)`
- 複数パラメータ: `(a: type, b: type)`
- パラメータは関数本体内でローカル変数として使用可能

### Function Call
```lak
let result = add(1, 2)
greet("hello")
```

## Dependencies
- Function definitions (completed)
- Type system (completed for i32, i64, string, bool)

## Dependents
- Non-void return types (phase3-02)
- Methods (phase7-04) - self parameter
- Struct instantiation (phase7-02) - factory functions
