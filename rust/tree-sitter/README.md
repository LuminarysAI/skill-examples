# Tree-Sitter Skill

Source code parsing to AST and symbol extraction.

## Methods

| Method | Parameters | Description |
|--------|-----------|-------------|
| `parse` | `code`, `language`, `path` | Parse code, return top-level symbols |
| `parse_file` | `path`, `language` | Read and parse a file (auto-detect language) |
| `ast` | `code`, `language`, `path`, `max_depth` | Full syntax tree as JSON |
| `languages` | — | List supported languages and extensions |

Supported languages: Go, Python, JavaScript, TypeScript, Rust, C, Java, JSON, Bash, HTML, CSS.

## Prerequisites

Requires [WASI SDK](https://github.com/WebAssembly/wasi-sdk/releases) for compiling tree-sitter C grammars to WASM.

### Setup

1. Download WASI SDK (e.g. `wasi-sdk-32.0-x86_64-linux.tar.gz`)
2. Extract to project root: `.toolchain/wasi-sdk-32.0-x86_64-linux/`
3. Set environment variables:

**Linux/macOS:**
```bash
export CC_wasm32_wasip1=$PWD/.toolchain/wasi-sdk-32.0-x86_64-linux/bin/clang
export CXX_wasm32_wasip1=$PWD/.toolchain/wasi-sdk-32.0-x86_64-linux/bin/clang++
export AR_wasm32_wasip1=$PWD/.toolchain/wasi-sdk-32.0-x86_64-linux/bin/ar
export CFLAGS_wasm32_wasip1="--sysroot=$PWD/.toolchain/wasi-sdk-32.0-x86_64-linux/share/wasi-sysroot"
export CXXFLAGS_wasm32_wasip1="--sysroot=$PWD/.toolchain/wasi-sdk-32.0-x86_64-linux/share/wasi-sysroot -fno-exceptions"
```

**Windows (cmd):**
```cmd
set CC_wasm32_wasip1=%CD%\.toolchain\wasi-sdk-32.0-x86_64-windows\bin\clang.exe
set CXX_wasm32_wasip1=%CD%\.toolchain\wasi-sdk-32.0-x86_64-windows\bin\clang++.exe
set AR_wasm32_wasip1=%CD%\.toolchain\wasi-sdk-32.0-x86_64-windows\bin\ar.exe
set CFLAGS_wasm32_wasip1=--sysroot=%CD%\.toolchain\wasi-sdk-32.0-x86_64-windows\share\wasi-sysroot
set CXXFLAGS_wasm32_wasip1=--sysroot=%CD%\.toolchain\wasi-sdk-32.0-x86_64-windows\share\wasi-sysroot -fno-exceptions
```

## Build

```bash
lmsk generate -lang rust ./src
cargo build --target wasm32-wasip1 --release
lmsk sign target/wasm32-wasip1/release/tree_sitter_skill.wasm
```

## Manifest

```yaml
id: tree-sitter
path: skills/ai.luminarys.rust.tree-sitter.skill
permissions:
  fs:
    enabled: true
    dirs: ["/data:ro"]
invoke_policy:
  can_invoke: []
  can_be_invoked_by: ["*"]
mcp:
  mapping: per_method
```
