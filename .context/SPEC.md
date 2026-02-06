# Lak Language Specification

## Overview

Lak is a programming language emphasizing simplicity, safety, and minimal syntax sugar.

**Design Philosophy:**
- **Simplicity First**: Minimize what needs to be learned. A language that requires less thinking.
- **Safety**: Prevent bugs at compile time. Defaults are safe.
- **Minimal Syntax Sugar**: Basically one way to write each feature.
- **Influences**: Go (simplicity), Rust (safety), V (ease of use)

---

## Syntax Rules

- **Semicolons**: Not required. Newlines terminate statements.
- **Comments**: Line comments only (`//`). No block comments.
- **Return Type**: Function return types are mandatory. Use `-> void` for no return value.
- **Return Statement**: Required for functions with return values. Optional at the end of `void` functions.
- **No Syntax Sugar**: No `?.`, `??`, `?`, `if let`, etc.
- **Defaults**: Variables are immutable. Interface implementation is implicit.

---

## Types

### Primitive Types

#### Numeric Types

| Type | Description |
|------|-------------|
| `int` | Signed integer (platform default size) |
| `uint` | Unsigned integer (platform default size) |
| `i8`, `i16`, `i32`, `i64` | Fixed-size signed integers |
| `u8`, `u16`, `u32`, `u64` | Fixed-size unsigned integers |
| `f32`, `f64` | Floating-point numbers |

#### Other Types

| Type | Description |
|------|-------------|
| `bool` | Boolean (`true` / `false`) |
| `string` | UTF-8 string (immutable) |
| `byte` | Alias for `u8` |
| `any` | Any type (for generic output functions) |
| `never` | Return type of functions that never return (e.g., `panic`) |
| `void` | No return value |

### any Type

The `any` type represents any value and is used for functions that accept values of any type, such as `println`.

```lak
fn println(value: any) -> void   // Accepts any type
```

**Behavior:**
- All types can be implicitly converted to `any`
- Values of `any` type can be printed using their default format
- Types implementing `Stringer` interface use `to_string()` for formatting

**Default Format:**
- Integers: `42`, `-10`
- Floats: `3.14`, `-0.5`
- Booleans: `true`, `false`
- Strings: `hello` (no quotes)
- Structs: `User { name: "alice", age: 30 }`
- Enums: `Option.Some(42)`, `Color.Red`

**Stringer Priority:**

If a type implements the `Stringer` interface, `to_string()` is used instead of the default format.

```lak
interface Stringer {
    fn to_string(self) -> string
}

struct User {
    pub name: string

    pub fn to_string(self) -> string {
        return "User: " + self.name
    }
}

let u = User { name: "alice" }
println(u)    // "User: alice" (uses to_string())
```

**Restrictions:**
- `any` cannot be used as a variable type: `let x: any = 1` is not allowed
- `any` is only valid as a function parameter type for specific built-in functions

### Tuple Type

Groups multiple values. No limit on element count.

```lak
let pair: (int, string) = (1, "hello")
let triple = (1, "a", true)            // Inferred as (int, string, bool)
```

**Element Access**: Use `.0`, `.1`, etc.

```lak
let pair = (1, "hello")
let n = pair.0          // 1
let s = pair.1          // "hello"

// Nested access
let nested = ((1, 2), "test")
let a = nested.0.1      // 2
```

**Destructuring**:

```lak
let pair = (1, "hello")
let x, y = pair         // x = 1, y = "hello"
```

### Collection Types

#### List

```lak
let numbers = [1, 2, 3]                // Inferred as List<int>
let names: List<string> = []            // Empty list requires type annotation
```

#### Map

Keys are limited to primitive types (`int`, `uint`, `i8`-`i64`, `u8`-`u64`, `string`, `bool`).

```lak
let ages = {"alice": 30, "bob": 25}     // Inferred as Map<string, int>
let empty: Map<string, int> = {}        // Empty map requires type annotation
```

#### Element Access

No index syntax (`[]`). Access via methods.

```lak
let numbers = [1, 2, 3]
let first = numbers.get(0)              // Option<int>

let ages = {"alice": 30}
let age = ages.get("alice")             // Option<int>

// Partial list
let sub = numbers.slice(0, 2)           // [1, 2]
```

#### Collection Mutability

To modify collection contents, declare variable with `mut`.

```lak
let numbers = [1, 2, 3]
numbers.push(4)                        // Compile error: immutable

let mut items = [1, 2, 3]
items.push(4)                          // OK
```

---

## Variables

Default is immutable. Use `mut` to make mutable.

```lak
let x = 5                   // Immutable, type inferred
let mut count = 0            // Mutable
let name: string = "lak"     // Explicit type annotation
let mut flag: bool = true    // Mutable + explicit type annotation
```

### Reassignment

Only variables declared with `mut` can be reassigned.

```lak
let mut count = 0
count = count + 1
```

### Shadowing

Redeclaration of same-named variables in the same scope is not allowed.

```lak
let x = 5
let x = 10    // Compile error: variable x is already declared
```

---

## Operators

### Arithmetic Operators

| Operator | Description |
|----------|-------------|
| `+` | Addition (numeric), concatenation (string) |
| `-` | Subtraction |
| `*` | Multiplication |
| `/` | Division |
| `%` | Modulo |

### Comparison Operators

| Operator | Description |
|----------|-------------|
| `==` | Equal |
| `!=` | Not equal |
| `<` | Less than |
| `>` | Greater than |
| `<=` | Less than or equal |
| `>=` | Greater than or equal |

### Logical Operators

| Operator | Description |
|----------|-------------|
| `&&` | Logical AND |
| `\|\|` | Logical OR |

### Unary Operators

| Operator | Description |
|----------|-------------|
| `-` | Negation |
| `!` | Logical NOT |

### Assignment Operators

| Operator | Description |
|----------|-------------|
| `=` | Assignment |

No compound assignment operators (`+=`, `-=`, etc.).

### Operator Precedence (highest to lowest)

| Precedence | Operators |
|------------|-----------|
| 1 (highest) | `-` (unary), `!` |
| 2 | `*`, `/`, `%` |
| 3 | `+`, `-` |
| 4 | `<`, `>`, `<=`, `>=` |
| 5 | `==`, `!=` |
| 6 | `&&` |
| 7 | `\|\|` |
| 8 (lowest) | `=` |

---

## Functions

Rust-style syntax. `fn` keyword, colon for parameter type annotations, `->` for return type.
Return type is mandatory. Use `-> void` for no return value.

```lak
fn add(a: int, b: int) -> int {
    return a + b
}

fn greet() -> void {
    println("hello")
}
```

### Return Statement

- Functions with return values require `return` statement.
- `void` functions can omit trailing `return`. Use bare `return` for early return.

```lak
fn abs(x: int) -> int {
    if x < 0 {
        return -x
    }
    return x              // return required
}

fn greet_if(flag: bool) -> void {
    if !flag {
        return            // Early return (no value)
    }
    println("hello")
                          // Trailing return can be omitted
}
```

### Multiple Return Values

Functions can return multiple values. Return type is expressed as tuple type.
Receiver side doesn't need type annotation (inferred from function signature).

```lak
fn min_max(list: List<int>) -> (int, int) {
    return (1, 10)
}

let min, max = min_max(numbers)    // Destructuring
let result = min_max(numbers)      // Receive as tuple: (int, int)
```

### Receiving Return Values

When calling functions/methods with return values, the return value must be received.
Unwanted return values can be discarded with `_`.

```lak
let min, max = min_max(numbers)     // OK
let _, max = min_max(numbers)       // OK: discard min
let min, _ = min_max(numbers)       // OK: discard max
min_max(numbers)                    // Compile error: return value not received
```

---

## Structs

Default is private. Use `pub` keyword to make public.

```lak
struct User {
    pub name: string
    age: int          // Private
}
```

### Instance Creation

Create instances with struct literals. All fields must be specified (no default values).
Struct literals with private fields can only be written within the same module.

```lak
let u = User { name: "alice", age: 30 }

let u2 = User { name: "bob" }    // Compile error: age not specified
```

To allow external instance creation, expose a factory function.

```lak
// user.lak
pub fn new_user(name: string, age: int) -> User {
    return User { name: name, age: age }
}

// main.lak
import "./user"
let u = user.new_user("alice", 30)
```

### Methods

Defined inside the struct. Take `self` as first parameter.
`self` is immutable by default. Use `mut self` to modify.
`mut self` methods can only be called on variables declared with `mut`.
Methods are private by default. Use `pub` to make public.

```lak
struct User {
    pub name: string
    age: int

    pub fn greet(self) -> string {
        return "Hello, " + self.name
    }

    pub fn is_adult(self) -> bool {
        return self.age >= 18
    }

    fn set_name(mut self, name: string) {
        self.name = name
    }
}
```

---

## Interfaces

Go-style implicit implementation. If a struct has all required methods, it automatically satisfies the interface.
Methods that modify `self` are defined with `mut self`.

```lak
interface Stringer {
    fn to_string(self) -> string
}

interface Writer {
    fn write(mut self, data: string)
}

// User automatically satisfies Stringer if it has to_string()
struct User {
    pub name: string

    pub fn to_string(self) -> string {
        return self.name
    }
}

fn print_string(s: Stringer) {
    println(s.to_string())
}
```

---

## Enums (Algebraic Data Types)

Supports both simple enums (no values) and algebraic data types (with values).

```lak
// Simple enum
enum Color {
    Red
    Green
    Blue
}

// Enum with values (algebraic data type)
enum Shape {
    Circle(f64)
    Rectangle(f64, f64)
    Point
}
```

### Variant Name Resolution

Variant name abbreviation is only allowed inside `match` expressions. Full qualification required elsewhere.

```lak
// Inside match: abbreviation allowed
match result {
    Ok(value) => println(value)      // Not Result.Ok, just Ok
    Err(e) => println(e.message())   // Not Result.Err, just Err
}

// Elsewhere: full qualification required
let name: Option<string> = Option.None
let value = Option.Some("alice")

// Return values: full qualification
fn find(id: int) -> Option<string> {
    return Option.None
}

// Arguments: full qualification
fn process(value: Option<int>) { ... }
process(Option.None)
```

---

## Generics

Type parameters with angle brackets `<T>`. Interface constraints supported.

```lak
// Generic function
fn first<T>(list: List<T>) -> Option<T> {
    // ...
}

// Generic struct
struct Pair<A, B> {
    pub first: A
    pub second: B
}

// Interface constraint
fn print_all<T: Stringer>(list: List<T>) {
    for item in list {
        println(item.to_string())
    }
}

// Multiple interface constraints
fn compare_and_print<T: Stringer + Comparable>(a: T, b: T) {
    if a.compare(b) > 0 {
        println(a.to_string())
    } else {
        println(b.to_string())
    }
}
```

---

## Control Flow

### if Expression

`if` can return a value as an expression.

```lak
if condition {
    // ...
} else {
    // ...
}

// if as expression
let max = if a > b { a } else { b }
```

**Rules when used as expression:**
- `else` is required.
- Both branches must have matching types for the last expression.
- The last evaluated expression in a block becomes the branch's value.

```lak
// OK: both branches are int
let max = if a > b { a } else { b }

// OK: nesting allowed
let result = if x > 0 {
    if x > 100 { 100 } else { x }
} else {
    0
}

// Compile error: no else
let value = if condition { 42 }

// Compile error: types don't match
let value = if condition { 42 } else { "hello" }
```

**When used as statement:**
- `else` can be omitted when not receiving a value.

```lak
if condition {
    println("yes")
}
```

### for Loop

Loop variable is immutable. Cannot be modified inside the loop.

`for` loop receives the return value of the iterator's `next` method. The number of receiving variables must match the return type.

```lak
// List<T> iterator returns (int, T) - index and element
for i, item in list {
    println("${i}: ${item}")
}

// Discard index with _
for _, item in list {
    println(item.to_string())
}

// Map<K, V> iterator returns (K, V)
for key, value in map {
    println("${key}: ${value}")
}

// Single-value iterator (e.g., range)
for i in range(0, 10) {
    println(i.to_string())
}

// break / continue
for _, item in list {
    if item == 3 {
        continue
    }
    if item == 7 {
        break
    }
    println(item.to_string())
}
```

### while Loop

Repeats while condition is true.

```lak
// Conditional loop
while condition {
    // ...
}

// Infinite loop
while true {
    if should_stop {
        break
    }
}
```

### match Expression

`match` can return a value as an expression. Has exhaustiveness checking.
Enums must handle all variants or it's a compile error. Non-enum values (integers, etc.) require `_` (wildcard).

**Rules when used as expression:**
- All arms must have matching types for the last expression.
- The last evaluated expression in an arm becomes that arm's value.

```lak
// As expression: all arms return same type (f64)
let area = match shape {
    Circle(r) => 3.14 * r * r
    Rectangle(w, h) => w * h
    Point => 0.0
}

match color {
    Red => "red"
    Green => "green"
    Blue => "blue"
}

// Default case (wildcard)
match value {
    1 => "one"
    2 => "two"
    _ => "other"
}

// Multiple patterns (separated by |)
match color {
    Red | Green => "warm"
    Blue => "cool"
}

match shape {
    Circle(r) | Square(r) => r * r    // Common binding variable required
    Rectangle(w, h) => w * h
}
```

---

## String Interpolation

Embed expressions in strings with `${}` syntax.

```lak
let name = "lak"
let version = 1
println("${name} v${version}")    // "lak v1"
```

- All types have a default format (e.g., `1`, `true`, `User { name: "alice", age: 30 }`).
- Types satisfying the `Stringer` interface preferentially use `to_string()`.

---

## Option Type (Null Safety)

Presence/absence of value is represented with `Option<T>` type. `None` corresponds to null.
No forced unwrap. Must safely handle with `match`.
Built-in enum provided in prelude, no import required.

```lak
enum Option<T> {
    Some(T)
    None
}
```

```lak
let name: Option<string> = Option.None

match name {
    Some(n) => println(n)           // Abbreviation allowed in match
    None => println("anonymous")
}
```

---

## Result Type (Error Handling)

Errors are represented with `Result<T, E>` type. No exception mechanism.
No error propagation syntax sugar (`?` operator, etc.). Must explicitly handle with `match`.
Built-in enum provided in prelude, no import required.

```lak
enum Result<T, E> {
    Ok(T)
    Err(E)
}
```

```lak
fn read_file(path: string) -> Result<string, FileError> {
    // ...
}

match read_file("data.txt") {
    Ok(content) => println(content)   // Abbreviation allowed in match
    Err(e) => println(e.message())
}
```

---

## panic (Unrecoverable Errors)

When an unrecoverable error occurs, `panic` immediately terminates the program.
Built-in function provided in prelude, no import required.

```lak
fn panic(message: string) -> never
```

`panic`'s return type is `never`, indicating this function never returns.
This allows type checking to work correctly in code like:

```lak
fn get_value(opt: Option<int>) -> int {
    match opt {
        Some(v) => v
        None => panic("value is required")  // never is compatible with int
    }
}
```

### When to Use panic vs Result

- **Result**: Errors recoverable by caller (file not found, network error, etc.)
- **panic**: Programming errors, invariant violations, unrecoverable states

```lak
// Use Result
fn read_file(path: string) -> Result<string, FileError> {
    // Return Err if file doesn't exist
}

// Use panic
fn divide(a: int, b: int) -> int {
    if b == 0 {
        panic("division by zero")
    }
    return a / b
}
```

---

## Prelude

The prelude is a set of types and functions implicitly available in all files.
Can be used without import statements.

### Prelude Contents

**Types:**
- `Option<T>` - Represents presence/absence of value
- `Result<T, E>` - Represents success/failure

**Functions:**
- `println(value: any) -> void` - Output any value (with newline)
- `print(value: any) -> void` - Output any value
- `panic(message: string) -> never` - Terminate program

### Prelude Override

If a local definition has the same name, the local definition takes precedence.

```lak
// Define custom println (overrides prelude's println)
fn println(value: any) -> void {
    // Custom implementation
}
```

---

## Modules

1 file = 1 module. Use `pub` keyword to make functions/structs/enums public.

### Visibility

```lak
pub fn add(a: int, b: int) -> int {
    return a + b
}

fn helper() -> int {       // Private (file-local only)
    return 42
}

pub struct User {
    pub name: string
    age: int               // Field is private
}
```

### import

Specify with string path. Module name is the last path segment.
Import loads the module's public definitions (functions, structs, enums, etc.).

```lak
// Standard library
import "math"              // Use as math.add()

// Standard library submodule
import "math/calc"         // Use as calc.add()

// Local file (relative path, must start with ./)
import "./utils"           // Use as utils.helper()
import "./lib/parser"      // Use as parser.parse()
```

The `main` function of imported modules is not executed (only the entry point's `main` is executed).

### Alias

Use `as` to specify alias to avoid name conflicts.

```lak
import "math/calc" as mc   // Use as mc.add()
```

---

## Entry Point

Program execution starts from the `main` function.

```lak
fn main() -> void {
    println("hello, world")
}
```

- `main` function takes no arguments and has no return value (`-> void`).
- `main` function doesn't need to be `pub`.
- The file with a `main` function becomes the entry point.

### Top-Level Constraints

Only declarations can be written at the top level. Executable statements go inside functions.

```lak
// OK: declarations at top level
struct User {
    pub name: string
}

fn helper() -> void {
    println("helper")
}

fn main() -> void {
    println("hello")    // Executable statements inside functions
}

// Compile error: executable statements not allowed at top level
// println("hello")
// let x = 1
```

---

## Undecided Items

The following items will be designed in future sessions:

- Memory management (GC, ownership, reference counting, etc.)
- Concurrency model
- Standard library details (`range` implementation, List/Map method list)
- Closures / anonymous functions
- Iteration protocol (`Iterator` interface details)
- Compile-time constants (`const`)
