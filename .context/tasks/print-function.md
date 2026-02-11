# print Built-in Function

## Overview
Implement the `print` built-in function for output without a newline.

### Signature
```lak
fn print(value: any) -> void
```

### Features
- Same as `println` but without a newline
- Accepts `any` type (all types can be implicitly converted)
- Outputs with default formatting based on type

### Examples
```lak
print("hello")
print(42)
print(true)
```
