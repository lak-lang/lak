# for Loop

## Overview
Implement the `for` loop for collection iteration.

### Syntax
```lak
for i, item in list {
    println("${i}: ${item}")
}
```

### Iteration Patterns
Loop variables receive the return value of the iterator's `next` method.

```lak
// List<T> iterator returns (int, T) - index and element
for i, item in list {
    println("${i}: ${item}")
}

// Discard index with _
for _, item in list {
    println(item)
}

// Map<K, V> iterator returns (K, V)
for key, value in map {
    println("${key}: ${value}")
}

// Single-value iterator (e.g., range)
for i in range(0, 10) {
    println(i)
}
```

### Loop Control
- `break`: exit the loop
- `continue`: move to the next iteration

```lak
for _, item in list {
    if item == 3 {
        continue
    }
    if item == 7 {
        break
    }
    println(item)
}
```

### Rules
- Loop variables are immutable (cannot be reassigned inside the loop).
