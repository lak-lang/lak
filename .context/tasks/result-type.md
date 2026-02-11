# Result<T, E> in Prelude

## Overview
Add the `Result<T, E>` type to the prelude. It is an enum that represents success/failure.

### Definition
```lak
enum Result<T, E> {
    Ok(T)
    Err(E)
}
```

### Usage
```lak
fn read_file(path: string) -> Result<string, FileError> {
    // ...
}

match read_file("data.txt") {
    Ok(content) => println(content)   // short form inside match
    Err(e) => println(e.message())
}
```

### Features
- Automatically available via prelude (usable without import)
- Error handling without an exception mechanism
- Handle errors explicitly with pattern matching

### When to Use
- `Result`: recoverable errors for callers (missing files, network errors, etc.)
- `panic`: programming errors, invariant violations, unrecoverable states
