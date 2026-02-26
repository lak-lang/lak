# Changelog

## [0.1.0](https://github.com/lak-lang/lak/compare/v0.0.2...v0.1.0) (2026-02-26)


### Features

* add f32/f64 support across compiler pipeline ([bd8312f](https://github.com/lak-lang/lak/commit/bd8312f665902a715c63d32f53f928f1bd45c700))
* add ordered comparison operators for string ([27845d4](https://github.com/lak-lang/lak/commit/27845d419cf35c8c1789dcab0e00d39f1e2a7f60))
* add return statements and non-void returns ([8463122](https://github.com/lak-lang/lak/commit/84631222bdad1e9f8e1bc8dff90dbcbb23594541))
* implement function parameters across parser, semantic, and codegen ([09c3e1e](https://github.com/lak-lang/lak/commit/09c3e1e885a84754cbeb283e03ed9ed65f866645))
* implement reassignment for mutable variables ([cd20d1d](https://github.com/lak-lang/lak/commit/cd20d1d4a99fab659a2281b19c6c226c6acf1aa5))
* implement remaining integer types ([4341ab4](https://github.com/lak-lang/lak/commit/4341ab410809627978093f94f0d5ecef537c77f2))
* implement while loop with break and continue ([a70b113](https://github.com/lak-lang/lak/commit/a70b113a9d4d73fd13263489ed73ac64b75ded7e))
* support let mut declarations and diagnostics ([8995e64](https://github.com/lak-lang/lak/commit/8995e640e4921533d2d309393097774817b2d8de))


### Patches

* adapt integer literals in binary operations ([79cc03c](https://github.com/lak-lang/lak/commit/79cc03c3d388cd6c5b8a8756fb50432f8a6c7c0d))
* avoid exposing mangled names in codegen internal errors ([0f920d1](https://github.com/lak-lang/lak/commit/0f920d1576ccdd00a5010f946aed43eb7d25c3a6))
* avoid false missing return for while true ([fb4a742](https://github.com/lak-lang/lak/commit/fb4a742bda752803123ca91c00d3adf7f77d616b))
* enforce discard binding syntax ([2b6a6ed](https://github.com/lak-lang/lak/commit/2b6a6ed6d06cfe821996322c25b26f0a6a5ef586))
* enforce inferred type contracts across semantic and codegen ([5f24071](https://github.com/lak-lang/lak/commit/5f24071e424ddaa5cf9b0218463b0f8e4bde0b66))
* handle const-bool if return reachability ([69e6732](https://github.com/lak-lang/lak/commit/69e67326af1cf039040ccfb5783f2b24e2c9a62a))
* isolate semantic analyzer state per analysis call ([c9b8097](https://github.com/lak-lang/lak/commit/c9b809780878c4943367cbb07184410fc9ed4da4))
* resolve runtime and msvc linker paths at runtime ([d1e0ed7](https://github.com/lak-lang/lak/commit/d1e0ed7e970662280722706a308e95781d95283b))

## [0.0.2](https://github.com/lak-lang/lak/compare/v0.0.1...v0.0.2) (2026-02-11)


### Features

* Add comparison operators (==, !=, &lt;, &gt;, &lt;=, &gt;=) ([021e20a](https://github.com/lak-lang/lak/commit/021e20a125ed486a5b4bfae6e620376d8dcf297e))
* add logical operators with short-circuit evaluation ([b0e5a16](https://github.com/lak-lang/lak/commit/b0e5a162ae40691ab9027f124af80d4da98cc678))
* implement if/else statements ([d0a7496](https://github.com/lak-lang/lak/commit/d0a74965d395c08a59459bd598d718de5a122d7d))
* Release v0.0.2 ([de152fa](https://github.com/lak-lang/lak/commit/de152fa16e9009b373c2fedd1b8521dfdf327e2c))
* support if expressions ([8365850](https://github.com/lak-lang/lak/commit/836585098e18962c9fefb7e474c2db029b935fce))


### Patches

* isolate build object files to temp dir ([2cbcb93](https://github.com/lak-lang/lak/commit/2cbcb93ba85aef9b549eab4051c3f81ac3d4d9fe))
* isolate function symbols and reserve prelude names ([401cdce](https://github.com/lak-lang/lak/commit/401cdceb6be531db7de0fc443af02e0650c58025))
* reject self-referential let initializer in semantic phase ([24cb880](https://github.com/lak-lang/lak/commit/24cb8800fed5fe9623b35104fbc227441fda2573))

## 0.0.1 (2026-02-10)


### Features

* Add -o flag to build command for custom output path ([ec654f6](https://github.com/lak-lang/lak/commit/ec654f633294246a22b9f8fb164f7b801b7b7af3))
* Add ariadne for beautiful error reporting ([dc55d1b](https://github.com/lak-lang/lak/commit/dc55d1b20387f8c0590b84ec57f02d1164288f8d))
* Add arithmetic operators (+, -, *, /, %) for i32 and i64 ([d07729e](https://github.com/lak-lang/lak/commit/d07729ee85d1ca250ab8afe8e0160ea745745f25))
* Add bool type with true/false literals and comprehensive tests ([71ebc70](https://github.com/lak-lang/lak/commit/71ebc7085ab8b67eca8eee6eb2f1c716d4cc6235))
* Add CLI with `lak run` command using clap ([f98dd4a](https://github.com/lak-lang/lak/commit/f98dd4ab5a6b8444bc83786a11104322dfcf79f0))
* Add cross-platform support for macOS and Windows CI and compilation ([0d77636](https://github.com/lak-lang/lak/commit/0d776367b12fa26f0ee452d4ace384f245a16cb3))
* Add Go-style Newline token for statement termination ([6e9349f](https://github.com/lak-lang/lak/commit/6e9349f66a5054e9de4781d365875ed32c60b237))
* Add help text support to ResolverError for actionable diagnostics ([8cdc242](https://github.com/lak-lang/lak/commit/8cdc24259bbc6987a507b90103805ef5e015ff38))
* Add import statements, visibility (pub), and member access parsing ([653413c](https://github.com/lak-lang/lak/commit/653413c29a07ce08a02875d4b503b06766e6d015))
* Add integer type support for println function ([53fc62f](https://github.com/lak-lang/lak/commit/53fc62f76e0b780e650e33b9e01f5c5bd8b8120d))
* Add module resolution, semantic analysis, and codegen for imports ([041bfba](https://github.com/lak-lang/lak/commit/041bfbaadb0d876b44cbd59ad3bacd052b3ac2fd))
* Add panic built-in function for program termination ([c95e8bf](https://github.com/lak-lang/lak/commit/c95e8bff7eebe2a42864a41ad3b05d72acfc03b4))
* Add runtime division/modulo by zero checks ([d0c08a5](https://github.com/lak-lang/lak/commit/d0c08a5a3c2d38927500c6e92127b06a6ce3e33f))
* Add semantic analysis phase with symbol table and error handling ([666f2e4](https://github.com/lak-lang/lak/commit/666f2e47001611f059efb943caca65b3ed6b7e97))
* Add short_message() to error types for report titles ([1877b93](https://github.com/lak-lang/lak/commit/1877b938c775c1ce48690c69b5b2bb53f7f10ce3))
* Add string type support for variable declarations ([c255062](https://github.com/lak-lang/lak/commit/c255062b75cb5ef68446bb03cfce81ebffc744ea))
* Add structured error handling with CodegenErrorKind and improve span tracking ([7b68532](https://github.com/lak-lang/lak/commit/7b68532390acd8e9644bc3758b3273bb2a1bbdd5))
* Add unary negation operator (-) with comprehensive tests ([948127b](https://github.com/lak-lang/lak/commit/948127b6cfbd1b3fd5dcf0be94e40479fc00966c))
* Add user-defined function call support ([de72dbe](https://github.com/lak-lang/lak/commit/de72dbeb80fd3ef517e8f493a0fa7b4adcae6a1b))
* Add variable declarations and integer types support ([d4e53d4](https://github.com/lak-lang/lak/commit/d4e53d4f16cbb1c793526005d7c9601791310c36))
* Change IntLiteral token from i64 to u64 with parser negation folding ([d4aba8a](https://github.com/lak-lang/lak/commit/d4aba8a72d8445f4550ad5093d30420167bfa2b8))
* Detect division/modulo overflow at runtime for MIN / -1 and MIN % -1 ([3d698f8](https://github.com/lak-lang/lak/commit/3d698f89bc0f380bdb50083ec4c00ea4a4cca283))
* Detect integer overflow at runtime with panic for arithmetic operations ([e865830](https://github.com/lak-lang/lak/commit/e8658305e10f334014eeefdb6398e2635865598c))
* Implement minimal Lak compiler with LLVM backend ([97930dd](https://github.com/lak-lang/lak/commit/97930ddef075d8a08caa66ae29469d61c08dcb4a))
* Implement run command ([9eac2e8](https://github.com/lak-lang/lak/commit/9eac2e8ad375ece5c80f7fde5bdbf593f93b5adb))
* Introduce main function as program entry point ([5db6b69](https://github.com/lak-lang/lak/commit/5db6b695ce929600a5660890981711c34be4363e))
* Reject file extensions in import paths with actionable error ([5843cf6](https://github.com/lak-lang/lak/commit/5843cf6d8fa6076e123850cd8d2894883029adcf))
* Release v0.0.1 ([a10f28c](https://github.com/lak-lang/lak/commit/a10f28c3b490ae7a59a576d505890b3dbf3fb53b))
* Rename `run` command to `build` ([b1ff738](https://github.com/lak-lang/lak/commit/b1ff738ebf361741c0f98cd96bec9fe08a7e4357))
* Restrict identifiers to ASCII-only characters ([3c19542](https://github.com/lak-lang/lak/commit/3c195423563405c103c0bb61a4c9e63f4d9e7992))
* Restrict whitespace to Go-compatible ASCII characters ([7f10d49](https://github.com/lak-lang/lak/commit/7f10d4908b7c72b785406218c6fec01cb33abd7f))


### Patches

* Address PR review feedback for path-based mangle prefix refactoring ([e4aa546](https://github.com/lak-lang/lak/commit/e4aa546005227b0f1f31ee8ede2dbb2215bd9778))
* Detect MSVC linker path at build time to avoid GNU link.exe conflict ([606804e](https://github.com/lak-lang/lak/commit/606804e9c3bd190e005a4d8e27af17d508abccbf))
* Disable inkwell default target features to fix Windows linking ([4daa814](https://github.com/lak-lang/lak/commit/4daa814194bada361a9693e010264cf14c479b82))
* Handle trailing newlines in parser main loop ([7fd658a](https://github.com/lak-lang/lak/commit/7fd658ab91c95e77044c683175e609a75055646f))
* Implement std::error::Error trait for ParseError ([c20ba22](https://github.com/lak-lang/lak/commit/c20ba2288f9eadd05d5faceb24f727e785ae47dc))
* Improve error handling, documentation accuracy, and type safety across compiler ([8aa435c](https://github.com/lak-lang/lak/commit/8aa435cc3983e97cfb8c01ad6630886b3a95f549))
* Improve error messages for missing parentheses in function calls ([0891677](https://github.com/lak-lang/lak/commit/08916779619c583fbabfe9a0c0a906230f424af0))
* Include minus sign in error span for negative integer literal overflow ([c812cea](https://github.com/lak-lang/lak/commit/c812cead4dd177afc77ddb1a4688c5ca24558891))
* Link Windows system libraries required by Rust stdlib ([6a258e8](https://github.com/lak-lang/lak/commit/6a258e83d8225e2514d74fbaead971e9d2ea900b))
* Pass MSVC LIB environment to link.exe for CI compatibility ([9d0a354](https://github.com/lak-lang/lak/commit/9d0a354d90122b7ced62d61f2957d2d3bffa5f52))
* Remove duplicate error message on compilation failure ([3037276](https://github.com/lak-lang/lak/commit/30372763c4a8d2b914f2b15ba30511a53cb11f67))
* Require newline separator between statements and function definitions ([fc5539a](https://github.com/lak-lang/lak/commit/fc5539acb9523a2c8c05c9c57e7c7410824d581d))
* Use byte indexing in ariadne for correct UTF-8 error highlighting ([9b9203b](https://github.com/lak-lang/lak/commit/9b9203bab661dc88ca0d2c53433e5d8bec8de2f7))
* Use cc crate to locate MSVC linker on Windows ([86b830e](https://github.com/lak-lang/lak/commit/86b830ec3aa4e0229ec8bd3b597491dbd481f610))
* Use correct LLVM version for Windows Chocolatey package ([906280b](https://github.com/lak-lang/lak/commit/906280bd9c658cfa01aacf7fee3d20153ac29102))
* Use RelocMode::PIC for PIE-compatible object file generation ([c6a6d97](https://github.com/lak-lang/lak/commit/c6a6d97f4e47c9f075d3e060a55a2f242219cdd0))
* Use vovkos/llvm-package-windows for Windows LLVM setup ([b106fad](https://github.com/lak-lang/lak/commit/b106fadb0eddbadc03c4d2bc20e4cfc8a674037f))
