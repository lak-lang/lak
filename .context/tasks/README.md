# Tasks Index

このディレクトリでは、各タスクファイルを独立した仕様メモとして管理する。
順序や依存関係はこの `README.md` にのみ記載し、各タスクファイル内には記載しない。

## 運用ルール
- タスクファイル名には連番やフェーズ番号を含めない（例: `if-else.md`）。
- タスクファイルには「概要・仕様・例」のみを書く。
- 実装順序や依存関係の追加・変更はこのファイルで行う。

## タスク一覧

### Basic Expressions
- `logical-operators.md` - Logical Operators
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

## 推奨実装順序
1. `logical-operators.md`
2. `modules-imports.md`
3. `if-else.md`
4. `if-expression.md`
5. `return-statement.md`
6. `while-loop.md`
7. `function-parameters.md`
8. `non-void-return.md`
9. `print-function.md`
10. `mut-modifier.md`
11. `reassignment.md`
12. `type-inference.md`
13. `integer-types.md`
14. `float-types.md`
15. `tuple-type.md`
16. `simple-enums.md`
17. `match-expression.md`
18. `enums-with-values.md`
19. `struct-definition.md`
20. `struct-instantiation.md`
21. `field-access.md`
22. `methods.md`
23. `interfaces.md`
24. `generics.md`
25. `option-type.md`
26. `result-type.md`
27. `list-type.md`
28. `map-type.md`
29. `for-loop.md`
30. `string-interpolation.md`

## タスク間依存関係
- `if-else.md` depends on `logical-operators.md`.
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

## 補足
- 依存関係に含まれていないタスクは、このディレクトリ内タスクへの明確な依存がない（または既存実装への依存のみ）。
- 依存が循環しそうな場合は、タスクを分割して `README.md` の依存グラフを更新する。
