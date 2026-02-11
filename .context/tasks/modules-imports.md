# Modules and Imports

## Overview
Implement the module system and import statements.

1 file = 1 module. Use the `pub` keyword to export functions/structs/enums.

### Visibility
- Default is private.
- Use the `pub` keyword to make definitions public.

### Import Syntax
```lak
import "math"              // standard library
import "math/calc"         // submodule
import "./utils"           // local file (relative path)
import "path" as alias     // alias
```

### Module Resolution
- The module name is the last segment of the path.
- `main` in imported modules is not executed.
- Only public definitions are accessible.
