# Tasks Index

In this directory, each task file is managed as an independent specification memo.
Execution order and dependencies are documented only in this `README.md`, and should not be written inside each task file.

## Operating Rules
- Do not include sequence numbers or phase numbers in task file names (example: `if-else.md`).
- Task files should contain only "Overview, Specification, and Examples."
- Add or modify implementation order and dependencies in this file.

## Task List

Status:
- `[Done]`: Task specification is implemented.
- `[Partial]`: Implemented in part, but the specification is not fully complete.
- `[Todo]`: Not implemented yet.
- No mark: Not implemented yet (used in `Task List`).

### Basic Expressions
- `logical-operators.md` - Logical Operators [Done]
- `string-comparison-operators.md` - Ordered Comparison for string [Done]
- `modules-imports.md` - Modules and Imports [Partial]

### Control Flow
- `if-else.md` - if/else Statement [Done]
- `if-expression.md` - if as Expression [Done]
- `return-statement.md` - Return Statement [Done]
- `while-loop.md` - while Loop with break/continue [Done]

### Functions
- `function-parameters.md` - Function Parameters [Done]
- `non-void-return.md` - Non-void Return Types [Done]
- `print-function.md` - print Built-in Function

### Variables
- `mut-modifier.md` - mut Modifier [Done]
- `reassignment.md` - Reassignment for Mutable Variables [Done]
- `type-inference.md` - Type Inference [Done]

### Types
- `integer-types.md` - Remaining Integer Types [Done]
- `float-types.md` - Float Types [Done]
- `tuple-type.md` - Tuple Type

### Algebraic Data Types
- `simple-enums.md` - Simple Enums
- `match-expression.md` - match Expression
- `enums-with-values.md` - Enums with Values
- `option-type.md` - Option<T> in Prelude
- `result-type.md` - Result<T, E> in Prelude

### User-Defined Types
- `struct-definition.md` - Struct Definition
- `struct-instantiation.md` - Struct Instantiation
- `field-access.md` - Field Access
- `methods.md` - Methods

### Collections
- `list-type.md` - List Type
- `map-type.md` - Map Type
- `for-loop.md` - for Loop

### Advanced Features
- `interfaces.md` - Interfaces
- `generics.md` - Generics
- `string-interpolation.md` - String Interpolation

## Recommended Implementation Order
1. `logical-operators.md` [Done]
2. `string-comparison-operators.md` [Done]
3. `modules-imports.md` [Partial]
4. `if-else.md` [Done]
5. `if-expression.md` [Done]
6. `return-statement.md` [Done]
7. `while-loop.md` [Done]
8. `function-parameters.md` [Done]
9. `non-void-return.md` [Done]
10. `print-function.md` [Todo]
11. `mut-modifier.md` [Done]
12. `reassignment.md` [Done]
13. `type-inference.md` [Done]
14. `integer-types.md` [Done]
15. `float-types.md` [Done]
16. `tuple-type.md` [Todo]
17. `simple-enums.md` [Todo]
18. `match-expression.md` [Todo]
19. `enums-with-values.md` [Todo]
20. `struct-definition.md` [Todo]
21. `struct-instantiation.md` [Todo]
22. `field-access.md` [Todo]
23. `methods.md` [Todo]
24. `interfaces.md` [Todo]
25. `generics.md` [Todo]
26. `option-type.md` [Todo]
27. `result-type.md` [Todo]
28. `list-type.md` [Todo]
29. `map-type.md` [Todo]
30. `for-loop.md` [Todo]
31. `string-interpolation.md` [Todo]

## Inter-Task Dependencies
- `if-else.md` depends on `logical-operators.md`.
- `string-comparison-operators.md` depends on `logical-operators.md`.
- `if-expression.md` depends on `if-else.md`.
- `return-statement.md` depends on `if-else.md`.
- `while-loop.md` depends on `logical-operators.md`.
- `non-void-return.md` depends on `function-parameters.md`, `return-statement.md`.
- `reassignment.md` depends on `mut-modifier.md`.
- `tuple-type.md` depends on `type-inference.md`.
- `simple-enums.md` and `match-expression.md` should be introduced together.
- `match-expression.md` depends on `if-else.md`, `simple-enums.md`.
- `enums-with-values.md` depends on `simple-enums.md`, `match-expression.md`.
- `option-type.md` depends on `simple-enums.md`, `enums-with-values.md`, `match-expression.md`, `generics.md`.
- `result-type.md` depends on `simple-enums.md`, `enums-with-values.md`, `match-expression.md`, `generics.md`.
- `struct-definition.md` depends on `modules-imports.md`.
- `struct-instantiation.md` depends on `struct-definition.md`, `function-parameters.md`, `modules-imports.md`.
- `field-access.md` depends on `struct-definition.md`, `struct-instantiation.md`, `modules-imports.md`.
- `methods.md` depends on `struct-definition.md`, `field-access.md`, `mut-modifier.md`, `function-parameters.md`.
- `interfaces.md` depends on `struct-definition.md`, `methods.md`, `function-parameters.md`.
- `generics.md` depends on `interfaces.md`, `struct-definition.md`, `function-parameters.md`.
- `list-type.md` depends on `generics.md`, `option-type.md`, `mut-modifier.md`.
- `map-type.md` depends on `generics.md`, `option-type.md`, `mut-modifier.md`.
- `for-loop.md` depends on `list-type.md`, `map-type.md`, `while-loop.md`, `tuple-type.md`.
- `string-interpolation.md` depends on `interfaces.md`.

## Notes
- Tasks not listed in dependencies have no explicit dependency on tasks in this directory (or depend only on existing implementations).
- If dependencies start to form a cycle, split tasks and update the dependency graph in `README.md`.
