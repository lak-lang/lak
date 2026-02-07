# Interfaces

## Phase
Phase 9: Advanced Features (Lower Priority)

## Overview
インターフェースを実装する。Goスタイルの暗黙的実装。

### Syntax
```lak
interface Stringer {
    fn to_string(self) -> string
}

interface Writer {
    fn write(mut self, data: string)
}
```

### Implicit Implementation
構造体が必要なメソッドを全て持っていれば、自動的にインターフェースを満たす。

```lak
struct User {
    pub name: string

    pub fn to_string(self) -> string {
        return self.name
    }
}

// User automatically satisfies Stringer
fn print_string(s: Stringer) {
    println(s.to_string())
}

let u = User { name: "alice" }
print_string(u)                  // OK
```

### self in Interfaces
- `self`: 読み取りのみのメソッド
- `mut self`: 変更を行うメソッド

## Dependencies
- Struct definition (phase7-01)
- Methods (phase7-04)
- Function parameters (phase3-01)

## Dependents
- Generic constraints (phase9-02)
- Stringer interface for `any` type
- Iterator interface
