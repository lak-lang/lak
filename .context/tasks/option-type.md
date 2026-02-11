# Option<T> in Prelude

## Overview
Add the `Option<T>` type to the prelude. It is an enum that represents optional values.

### Definition
```lak
enum Option<T> {
    Some(T)
    None
}
```

### Usage
```lak
let name: Option<string> = Option.None
let value = Option.Some("alice")

match name {
    Some(n) => println(n)           // short form inside match
    None => println("anonymous")
}
```

### Features
- Automatically available via prelude (usable without import)
- Provides null safety
- Extract values safely with pattern matching
