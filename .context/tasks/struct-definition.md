# Struct Definition

## Overview
Implement struct definitions.

### Syntax
```lak
struct User {
    pub name: string
    age: int          // Private (default)
}
```

### Visibility
- Fields are private by default.
- Use the `pub` keyword to make fields public.

### Rules
- Fields have a name and type.
- Duplicate field names are not allowed.
- The struct itself can also be public with `pub`.

```lak
pub struct User {
    pub name: string
    age: int
}
```
