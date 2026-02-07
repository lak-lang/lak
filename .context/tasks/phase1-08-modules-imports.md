# Modules and Imports

## Phase
Phase 1: Basic Expressions (High Priority)

## Overview
モジュールシステムとimport文を実装する。

1ファイル = 1モジュール。`pub` キーワードで関数/構造体/enumを公開する。

### Visibility
- デフォルトはprivate
- `pub` キーワードで公開

### Import Syntax
```lak
import "math"              // 標準ライブラリ
import "math/calc"         // サブモジュール
import "./utils"           // ローカルファイル（相対パス）
import "path" as alias     // エイリアス
```

### Module Resolution
- モジュール名はパスの最後のセグメント
- importされたモジュールの `main` は実行されない
- publicな定義のみアクセス可能

## Dependencies
- Function definitions (completed)
- `pub` visibility keyword (new)

## Dependents
- Standard library access
- Code organization for larger projects
