# Tasks Index

In this directory, each task file is managed as an independent specification memo.
Execution order and dependencies are documented only in this `README.md`, and should not be written inside each task file.

## Operating Rules
- Do not include sequence numbers or phase numbers in task file names (example: `if-else.md`).
- Task files should contain only "Overview, Specification, and Examples."
- Add or modify implementation order and dependencies in this file.

## Task List

### Basic Expressions
- `logical-operators.md` - Logical Operators
- `string-comparison-operators.md` - Ordered Comparison for string
- `modules-imports.md` - Modules and Imports

### Control Flow
- `if-else.md` - if/else Statement
- `if-expression.md` - if as Expression
- `return-statement.md` - Return Statement
- `while-loop.md` - while Loop with break/continue

### Functions
- `function-parameters.md` - Function Parameters
- `non-void-return.md` - Non-void Return Types
- `print-function.md` - print Built-in Function

### Variables
- `mut-modifier.md` - mut Modifier
- `reassignment.md` - Reassignment for Mutable Variables
- `type-inference.md` - Type Inference

### Types
- `integer-types.md` - Remaining Integer Types
- `float-types.md` - Float Types
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
1. `logical-operators.md`
2. `string-comparison-operators.md`
3. `modules-imports.md`
4. `if-else.md`
5. `if-expression.md`
6. `return-statement.md`
7. `while-loop.md`
8. `function-parameters.md`
9. `non-void-return.md`
10. `print-function.md`
11. `mut-modifier.md`
12. `reassignment.md`
13. `type-inference.md`
14. `integer-types.md`
15. `float-types.md`
16. `tuple-type.md`
17. `simple-enums.md`
18. `match-expression.md`
19. `enums-with-values.md`
20. `struct-definition.md`
21. `struct-instantiation.md`
22. `field-access.md`
23. `methods.md`
24. `interfaces.md`
25. `generics.md`
26. `option-type.md`
27. `result-type.md`
28. `list-type.md`
29. `map-type.md`
30. `for-loop.md`
31. `string-interpolation.md`

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
