# `lak` Binary Embeds a Build-Host Absolute Runtime Library Path

## Summary

The compiled `lak` binary embeds an absolute path to the runtime static library from the build host, and the link step uses that path directly.  
The release workflow archives include the `lak` executable but do not include a runtime library file in the archive payload.

## Details

### Build-time environment embedding and runtime link usage

- `compiler/build.rs:35` to `compiler/build.rs:41` computes `target/<profile>/<runtime-lib>` and exports it as `LAK_RUNTIME_PATH`.
- `compiler/src/main.rs:511` defines `const LAK_RUNTIME_PATH: &str = env!("LAK_RUNTIME_PATH");`.
- `compiler/src/main.rs:594` to `compiler/src/main.rs:596` passes `LAK_RUNTIME_PATH` to `cc` during link.

Relevant snippets:

```rust
// compiler/build.rs
let runtime_lib = workspace_root
    .join("target")
    .join(&profile)
    .join(runtime_lib_name);
println!("cargo:rustc-env=LAK_RUNTIME_PATH={}", runtime_lib.display());
```

```rust
// compiler/src/main.rs
const LAK_RUNTIME_PATH: &str = env!("LAK_RUNTIME_PATH");
let output = Command::new("cc")
    .args([object_str, LAK_RUNTIME_PATH, "-o", output_str])
    .output()?;
```

### Embedded path observed from compiled binary

Observed command output:

```text
$ strings target/debug/lak | rg -o "/Users/.*/liblak_runtime\.a|[A-Za-z]:\\[^ ]*lak_runtime\.lib"
/Users/koki/work/repos/github.com/lak-lang/lak/target/debug/liblak_runtime.a
```

### Release archive composition

- `.github/workflows/release-please.yml:102` copies only `target/${TARGET}/release/lak`.
- `.github/workflows/release-please.yml:103` copies `LICENSE` and `README.md`.
- `.github/workflows/release-please.yml:104` creates the archive from that directory.

Relevant snippet:

```yaml
cp "target/${TARGET}/release/lak" archive/
cp LICENSE README.md archive/
tar czvf "$ARCHIVE_NAME" -C archive .
```

## Expected Behavior

Language-level compilation is specified around source files/modules and entry-point execution (`.context/SPEC.md:803`, `.context/SPEC.md:822`, `.context/SPEC.md:851`).  
`lak build <file.lak>` and `lak run <file.lak>` behavior is documented as command-level compilation/execution in the compiler entrypoint docs (`compiler/src/main.rs:9`, `compiler/src/main.rs:11`).  
The observed runtime library resolution in the binary is expected to be consistent with distributed command execution context rather than tied to one build-host absolute path.
