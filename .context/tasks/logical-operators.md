# Logical Operators

## Overview
論理演算子 (`&&`, `||`, `!`) を実装する。

- `&&`: 論理AND（短絡評価）
- `||`: 論理OR（短絡評価）
- `!`: 論理NOT（単項演算子）

### Precedence
- `!` (unary): Level 1 (highest)
- `&&`: Level 6
- `||`: Level 7

### Short-circuit Evaluation
`&&` と `||` は短絡評価を行う。左辺で結果が確定した場合、右辺は評価されない。

