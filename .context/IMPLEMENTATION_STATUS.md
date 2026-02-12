# Lak Language Implementation Status

This document tracks the implementation status of Lak language features as defined in `.context/SPEC.md`.

**Legend:**
- `[x]` - Implemented
- `[ ]` - Not implemented
- Nested items indicate sub-features or implementation details

---

## 1. Syntax Rules

- [x] Semicolons not required (newlines terminate statements)
- [x] Line comments (`//`)
- [x] Function return type annotation (`-> type`)
- [x] `-> void` for no return value
- [x] Return statement for functions with return values
- [x] Optional trailing return for void functions

---

## 2. Types

### 2.1 Primitive Types

#### Numeric Types

- [ ] `int` - Platform-default signed integer
- [ ] `uint` - Platform-default unsigned integer
- [ ] `i8` - 8-bit signed integer
- [ ] `i16` - 16-bit signed integer
- [x] `i32` - 32-bit signed integer
- [x] `i64` - 64-bit signed integer
- [ ] `u8` - 8-bit unsigned integer
- [ ] `u16` - 16-bit unsigned integer
- [ ] `u32` - 32-bit unsigned integer
- [ ] `u64` - 64-bit unsigned integer
- [ ] `f32` - 32-bit floating point
- [ ] `f64` - 64-bit floating point

#### Other Primitive Types

- [x] `bool` type
  - [x] `true` literal
  - [x] `false` literal
- [x] `string` type
  - [x] String literals with double quotes
  - [x] Escape sequences (`\n`, `\t`, `\r`, `\\`, `\"`)
  - [ ] String concatenation with `+`
- [ ] `byte` (alias for `u8`)
- [x] `any` type (for generic output functions)
  - [x] Implicit conversion from any type to `any` (compile-time dispatch)
  - [x] Default format for integer types (i32, i64)
  - [ ] Default format for other types
  - [ ] Stringer interface priority
- [ ] `never` type (for functions that don't return)
- [x] `void` type

### 2.2 Tuple Type

- [ ] Tuple type declaration `(T1, T2, ...)`
- [ ] Tuple literal `(value1, value2, ...)`
- [ ] Element access with `.0`, `.1`, etc.
- [ ] Nested tuple access `tuple.0.1`
- [ ] Tuple destructuring `let x, y = pair`

### 2.3 Collection Types

#### List

- [ ] List literal `[1, 2, 3]`
- [ ] Empty list with type annotation `List<T>`
- [ ] `get(index)` method returning `Option<T>`
- [ ] `slice(start, end)` method
- [ ] `push(item)` method for mutable lists

#### Map

- [ ] Map literal `{"key": value}`
- [ ] Empty map with type annotation `Map<K, V>`
- [ ] `get(key)` method returning `Option<V>`
- [ ] Key type restriction (primitives only)

#### Collection Mutability

- [ ] Immutable collections by default
- [ ] Mutable collections with `mut`

---

## 3. Variables

### 3.1 Variable Declaration

- [x] Basic `let` declaration
- [x] Explicit type annotation `let x: type = value`
- [ ] Type inference `let x = value`
- [ ] Mutable declaration `let mut x = value`
- [ ] Mutable with type annotation `let mut x: type = value`

### 3.2 Reassignment

- [ ] Reassignment for `mut` variables
- [ ] Error on reassignment of immutable variables

### 3.3 Shadowing

- [x] Disallow redeclaration in same scope
- [ ] Allow shadowing in nested scopes (symbol table ready, awaiting control flow)

---

## 4. Operators

### 4.1 Arithmetic Operators

- [x] `+` addition
  - [x] Integer addition (i32, i64)
  - [x] Overflow detection with panic
  - [x] Integer literal adaptation (`int_literal <op> i32/i64`)
  - [ ] Non-literal mixed integer arithmetic (`i32` + `i64`) with widening to `i64`
  - [ ] Float addition
  - [ ] Mixed float arithmetic (`f32` + `f64`) with widening to `f64`
  - [ ] Integer/float mixed arithmetic requires explicit cast (no implicit conversion)
  - [ ] String concatenation
- [x] `-` subtraction (i32, i64)
  - [x] Overflow detection with panic
- [x] `*` multiplication (i32, i64)
  - [x] Overflow detection with panic
- [x] `/` division (i32, i64)
  - [x] Overflow detection with panic
- [x] `%` modulo (i32, i64)
  - [x] Overflow detection with panic

### 4.2 Comparison Operators

- [x] `==` equal (i32, i64, bool, string)
- [x] `!=` not equal (i32, i64, bool, string)
- [x] `<` less than (i32, i64)
- [x] `>` greater than (i32, i64)
- [x] `<=` less than or equal (i32, i64)
- [x] `>=` greater than or equal (i32, i64)
- [x] Integer literal adaptation in numeric comparisons
- [ ] Non-literal mixed integer comparison (`i32` vs `i64`) with widening to `i64`
- [ ] Mixed float comparison (`f32` vs `f64`) with widening to `f64`
- [ ] Integer/float mixed comparison requires explicit cast (no implicit conversion)

### 4.3 Logical Operators

- [x] `&&` logical AND
- [x] `||` logical OR

### 4.4 Unary Operators

- [x] `-` negation (unary minus)
  - [x] Overflow detection with panic
- [x] `!` logical NOT

### 4.5 Assignment Operators

- [x] `=` initial assignment (in let)
- [ ] `=` reassignment (for mut variables)

### 4.6 Operator Precedence

- [x] Precedence levels for arithmetic (multiplicative > additive)
- [ ] Full precedence levels 1-8 as specified
- [x] Parentheses for grouping `(expr)`

---

## 5. Functions

### 5.1 Function Definition

- [x] `fn` keyword
- [x] Function name (identifier)
- [x] Parameter list `()` (empty only)
  - [x] Single parameter `(name: type)`
  - [x] Multiple parameters `(a: type, b: type)`
- [x] Return type annotation `-> type`
- [x] Function body `{ ... }`

### 5.2 Return Statement

- [x] `return expr` for non-void functions
- [x] `return` (bare) for early return in void functions
- [ ] Implicit return (last expression as return value)

### 5.3 Multiple Return Values

- [ ] Tuple return type `-> (T1, T2)`
- [ ] Multiple value return `return (a, b)`
- [ ] Destructuring assignment `let x, y = func()`

### 5.4 Function Call

- [x] Basic function call `func()`
- [x] User-defined function calls
- [x] Function call with arguments `func(arg1, arg2)`
- [x] Discard return value with `_`
- [x] Error on ignored return value

### 5.5 Built-in Functions

- [x] `println(value: any)` - print any value with newline
  - [x] String argument support
  - [x] Integer argument support (i32, i64)
  - [x] Bool argument support
  - [ ] Float argument support (f32, f64)
  - [ ] Struct argument support (default format)
  - [ ] Stringer interface priority
- [ ] `print(value: any)` - print any value without newline
- [x] `panic(message: string) -> never` - terminate program

---

## 6. Structs

### 6.1 Struct Definition

- [ ] `struct` keyword
- [ ] Field definitions
- [ ] `pub` visibility for fields
- [ ] Private fields by default

### 6.2 Instance Creation

- [ ] Struct literal `Type { field: value }`
- [ ] All fields required (no defaults)
- [ ] Private field restriction to same module

### 6.3 Factory Functions

- [ ] Public factory function pattern

### 6.4 Methods

- [ ] Method definition inside struct
- [ ] `self` parameter (immutable)
- [ ] `mut self` parameter (mutable)
- [ ] `pub` visibility for methods
- [ ] Method calls `instance.method()`

---

## 7. Interfaces

- [ ] `interface` keyword
- [ ] Method signature declarations
- [ ] `self` in interface methods
- [ ] `mut self` in interface methods
- [ ] Implicit interface implementation
- [ ] Interface as parameter type
- [ ] Interface constraint checking

---

## 8. Enums (Algebraic Data Types)

### 8.1 Simple Enums

- [ ] `enum` keyword
- [ ] Variants without values

### 8.2 Enums with Values

- [ ] Single-value variants `Variant(T)`
- [ ] Multi-value variants `Variant(T1, T2)`
- [ ] Mixed variants (some with values, some without)

### 8.3 Variant Name Resolution

- [ ] Full qualification `Enum.Variant`
- [ ] Abbreviation inside `match`
- [ ] Error on unqualified use outside `match`

---

## 9. Generics

### 9.1 Generic Functions

- [ ] Type parameter syntax `fn name<T>()`
- [ ] Multiple type parameters `<A, B>`
- [ ] Type parameter usage in parameters/return

### 9.2 Generic Structs

- [ ] Struct type parameters `struct Name<T>`

### 9.3 Interface Constraints

- [ ] Single constraint `<T: Interface>`
- [ ] Multiple constraints `<T: A + B>`

---

## 10. Control Flow

### 10.1 if Expression

- [x] Basic `if condition { ... }`
- [x] `else` branch `if condition { ... } else { ... }`
- [x] `else if` chain
- [x] if as expression (returns value)
- [x] Type matching between branches
- [x] Error on missing `else` when used as expression

### 10.2 for Loop

- [ ] `for item in collection { ... }`
- [ ] Index-value iteration `for i, item in list`
- [ ] Key-value iteration `for key, value in map`
- [ ] Single-value iteration `for i in range()`
- [ ] Discard with `_` pattern
- [ ] `break` statement
- [ ] `continue` statement
- [ ] Immutable loop variable

### 10.3 while Loop

- [ ] `while condition { ... }`
- [ ] Infinite loop `while true { ... }`
- [ ] `break` in while
- [ ] `continue` in while

### 10.4 match Expression

- [ ] `match expr { ... }`
- [ ] Pattern arms `pattern => expr`
- [ ] Value binding in patterns `Some(x) =>`
- [ ] Exhaustiveness checking for enums
- [ ] Wildcard pattern `_`
- [ ] Multiple patterns with `|`
- [ ] Common binding in `|` patterns
- [ ] match as expression (returns value)
- [ ] Type matching between arms

---

## 11. String Interpolation

- [ ] `${}` syntax in string literals
- [ ] Variable interpolation `"${name}"`
- [ ] Expression interpolation `"${a + b}"`
- [ ] Default formatting for types
- [ ] `Stringer` interface priority

---

## 12. Option Type (Null Safety)

- [ ] `Option<T>` enum definition in prelude
- [ ] `Option.Some(value)` variant
- [ ] `Option.None` variant
- [ ] Pattern matching on Option
- [ ] Abbreviation in match (`Some`, `None`)

---

## 13. Result Type (Error Handling)

- [ ] `Result<T, E>` enum definition in prelude
- [ ] `Result.Ok(value)` variant
- [ ] `Result.Err(error)` variant
- [ ] Pattern matching on Result
- [ ] Abbreviation in match (`Ok`, `Err`)

---

## 14. panic (Unrecoverable Errors)

- [x] `panic(message)` function in prelude
- [ ] Returns `never` type (type system integration)
- [x] Program termination
- [ ] `never` type compatibility in match arms

---

## 15. Prelude

### 15.1 Prelude Types

- [ ] `Option<T>` automatically available
- [ ] `Result<T, E>` automatically available

### 15.2 Prelude Functions

- [x] `println(value: any)` available (string, i32, i64)
- [ ] `print(value: any)` available
- [x] `panic(message: string)` available

### 15.3 Prelude Reserved Names

- [x] `println` and `panic` cannot be redefined by local functions

---

## 16. Modules

### 16.1 Visibility

- [x] `pub` keyword for functions
- [ ] `pub` keyword for structs
- [ ] `pub` keyword for enums
- [x] Private by default

### 16.2 import Statement

- [x] `import "path"` syntax
- [ ] Standard library import `import "math"` (returns error: not yet supported)
- [ ] Submodule import `import "math/calc"`
- [x] Local file import `import "./utils"`
- [x] Module name from last path segment
- [x] `as` alias `import "path" as alias`

### 16.3 Module Resolution

- [x] Public definitions accessible
- [x] Private definitions hidden
- [x] Imported module's `main` not executed
- [x] Circular import detection
- [x] Transitive import support
- [x] Name mangling for multi-module compilation

---

## 17. Entry Point

- [x] `main` function as entry point
- [x] `main` signature: `fn main() -> void`
- [x] `main` doesn't need `pub`
- [x] Only declarations at top level
- [x] Error on executable statements at top level

---

## 18. Compiler Infrastructure

### 18.1 Lexer

- [x] Token position tracking (line, column)
- [x] Error recovery / reporting
- [x] ASCII-only identifiers (a-z, A-Z, 0-9, _; Unicode rejected with error)
- [x] Integer overflow detection
- [x] Automatic newline insertion

### 18.2 Parser

- [x] AST generation
- [x] Error messages with source location
- [x] Operator precedence parsing (Pratt parsing for arithmetic)

### 18.3 Semantic Analyzer

- [x] Symbol table management
- [x] Scope handling
- [x] Duplicate definition detection
- [x] Undefined reference detection
- [x] Type checking (basic)
- [ ] Full type inference
- [ ] Mutability checking
- [ ] Control flow analysis
- [ ] Exhaustiveness checking

### 18.4 Code Generation

- [x] LLVM IR generation
- [x] Object file output
- [x] System linker integration
- [x] Runtime library linking
- [x] Forward reference support (2-pass declaration/definition)
- [ ] Debug info generation
- [ ] Optimization passes

### 18.5 Runtime Library

- [x] `lak_println` function (string)
- [x] `lak_println_i32` function
- [x] `lak_println_i64` function
- [x] `lak_println_bool` function
- [ ] `lak_print` function
- [x] `lak_panic` function
- [ ] Memory allocation functions
- [ ] String operations

---

## Statistics

| Category | Implemented | Total | Progress |
|----------|-------------|-------|----------|
| Types | 6 | 26 | 23% |
| Variables | 3 | 8 | 38% |
| Operators | 12 | 21 | 57% |
| Functions | 8 | 18 | 44% |
| Structs | 0 | 12 | 0% |
| Interfaces | 0 | 7 | 0% |
| Enums | 0 | 12 | 0% |
| Generics | 0 | 5 | 0% |
| Control Flow | 0 | 28 | 0% |
| String Interpolation | 0 | 5 | 0% |
| Error Handling | 0 | 12 | 0% |
| Modules | 10 | 15 | 67% |
| Entry Point | 5 | 5 | 100% |

**Overall Progress: ~25%**
