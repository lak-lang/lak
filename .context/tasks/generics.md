# Generics

## Overview
ジェネリクス（型パラメータ）を実装する。

### Generic Functions
```lak
fn first<T>(list: List<T>) -> Option<T> {
    return list.get(0)
}

fn swap<A, B>(pair: (A, B)) -> (B, A) {
    return (pair.1, pair.0)
}
```

### Generic Structs
```lak
struct Pair<A, B> {
    pub first: A
    pub second: B
}

let p = Pair<int, string> { first: 1, second: "hello" }
```

### Interface Constraints
```lak
// Single constraint
fn print_all<T: Stringer>(list: List<T>) {
    for _, item in list {
        println(item.to_string())
    }
}

// Multiple constraints
fn compare_and_print<T: Stringer + Comparable>(a: T, b: T) {
    if a.compare(b) > 0 {
        println(a.to_string())
    } else {
        println(b.to_string())
    }
}
```

