# Result<T, E> in Prelude

## Overview
`Result<T, E>` 型をpreludeに追加する。成功/失敗を表現するためのenum。

### Definition
```lak
enum Result<T, E> {
    Ok(T)
    Err(E)
}
```

### Usage
```lak
fn read_file(path: string) -> Result<string, FileError> {
    // ...
}

match read_file("data.txt") {
    Ok(content) => println(content)   // match内では省略形
    Err(e) => println(e.message())
}
```

### Features
- preludeで自動的に利用可能（importなしで使用可能）
- 例外機構なしのエラーハンドリング
- パターンマッチングで明示的にエラー処理

### When to Use
- `Result`: 呼び出し側で回復可能なエラー（ファイル不存在、ネットワークエラー等）
- `panic`: プログラミングエラー、不変条件違反、回復不能な状態

