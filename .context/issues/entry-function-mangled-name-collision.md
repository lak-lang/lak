# Entry Function Names Can Collide with Mangled Imported Symbols (`_L...`)

## Summary

An entry-module function can be defined with a name that matches the mangled symbol form used for imported module functions (`_L{len}_{prefix}_{function}`). In this case, module-qualified calls can execute the entry function instead of the imported function.

## Details

### Reproduction

`utils.lak`:

```lak
pub fn foo() -> void {
  println("from utils")
}
```

`main.lak`:

```lak
import "./utils"

fn _L5_utils_foo() -> void {
  println("from main")
}

fn main() -> void {
  utils.foo()
  _L5_utils_foo()
}
```

Observed execution result (`cargo run -- run main.lak`):

```
from main
from main
```

Control case with non-colliding local function name:

```lak
import "./utils"

fn local_foo() -> void {
  println("from main")
}

fn main() -> void {
  utils.foo()
  local_foo()
}
```

Observed execution result:

```
from utils
from main
```

### Related Code Locations

- Mangled name format:
  - `compiler/src/codegen/mod.rs:152`
  - `compiler/src/codegen/mod.rs:163`
- Imported module functions are declared using mangled names:
  - `compiler/src/codegen/mod.rs:414`
  - `compiler/src/codegen/mod.rs:417`
  - `compiler/src/codegen/mod.rs:420`
- Entry module functions are declared with original names:
  - `compiler/src/codegen/mod.rs:414`
  - `compiler/src/codegen/mod.rs:415`
- Multi-module compilation path:
  - `compiler/src/main.rs:693`
  - `compiler/src/main.rs:695`

## Expected Behavior

The language specification states that importing a module loads its public definitions and uses module-qualified access (`.context/SPEC.md:827`, `.context/SPEC.md:837`), e.g., `utils.helper()`.  
For module-qualified calls, the executed function body corresponds to the imported module’s public function associated with that qualifier and function name.

