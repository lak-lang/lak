# Function Parameters

## Overview
Implement support for function parameters.

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
- Single parameter: `(name: type)`
- Multiple parameters: `(a: type, b: type)`
- Parameters can be used as local variables in the function body.

### Function Call
```lak
let result = add(1, 2)
greet("hello")
```
