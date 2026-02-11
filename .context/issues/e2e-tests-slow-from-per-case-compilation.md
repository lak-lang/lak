# E2E Test Runtime Is Dominated by Per-Case Full Compilation

## Summary

`cargo test` for the `lak` compiler can take more than one minute, and a large portion of runtime is concentrated in specific E2E test targets that repeatedly run full compile-link-execute cycles.

## Details

Observed timings in this repository:

```bash
$ /usr/bin/time -p sh -c 'cargo test -q -p lak >/dev/null'
real 76.69
user 21.95
sys 11.51
```

```bash
$ cd compiler
$ /usr/bin/time -p sh -c 'cargo test -q --test e2e_arithmetic >/dev/null'
real 29.35
user 6.90
sys 2.94

$ /usr/bin/time -p sh -c 'cargo test -q --test e2e_comparison >/dev/null'
real 15.71
user 4.69
sys 1.87
```

Observed call counts in heavy E2E files:

```bash
$ rg -n "compile_and_run\(" compiler/tests/e2e_arithmetic.rs | wc -l
59
$ rg -n "compile_and_run\(" compiler/tests/e2e_comparison.rs | wc -l
67
$ rg -n '^\s*#\[test\]' compiler/tests/e2e_arithmetic.rs | wc -l
82
$ rg -n '^\s*#\[test\]' compiler/tests/e2e_comparison.rs | wc -l
67
```

The shared helper used by many E2E tests executes the full pipeline per invocation:

- `compiler/tests/common/mod.rs:50` starts `compile_and_run(source: &str)`
- `compiler/tests/common/mod.rs:52-68` performs lex/parse/semantic/codegen
- `compiler/tests/common/mod.rs:71-77` writes an object file
- `compiler/tests/common/mod.rs:112-115` invokes `cc` for linking
- `compiler/tests/common/mod.rs:129-131` runs the produced executable

This pattern is used repeatedly across large E2E suites such as:

- `compiler/tests/e2e_arithmetic.rs`
- `compiler/tests/e2e_comparison.rs`

## Expected Behavior

Routine local validation with `cargo test` should complete in a development-friendly time range and should not require minute-scale runtime for common workflows.
