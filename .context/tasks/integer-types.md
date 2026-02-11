# Remaining Integer Types

## Overview
残りの整数型を実装する。

### Types
| Type | Description |
|------|-------------|
| `int` | Signed integer (platform default size) |
| `uint` | Unsigned integer (platform default size) |
| `i8` | 8-bit signed integer |
| `i16` | 16-bit signed integer |
| `u8` | 8-bit unsigned integer |
| `u16` | 16-bit unsigned integer |
| `u32` | 32-bit unsigned integer |
| `u64` | 64-bit unsigned integer |

### Notes
- `int`/`uint` はプラットフォームのデフォルトサイズ（通常64bit）
- `byte` は `u8` のエイリアス
- 各型に対して算術演算子と比較演算子をサポート

