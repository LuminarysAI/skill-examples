# Rust Skills

WASM skills built with the [Luminarys Rust SDK](https://crates.io/crates/luminarys-sdk).

## Prerequisites

- Rust toolchain (1.75+)
- WASM target: `rustup target add wasm32-wasip1`
- `lmsk` CLI (skill toolchain)
- Signing key: `lmsk genkey`

### WASI SDK (optional, for tree-sitter)

tree-sitter uses native C grammars and requires WASI SDK:

1. Download from https://github.com/WebAssembly/wasi-sdk/releases
2. Extract to `.toolchain/wasi-sdk-*`
3. Set environment:
   ```bash
   export CC_wasm32_wasip1=/path/to/wasi-sdk/bin/clang
   export AR_wasm32_wasip1=/path/to/wasi-sdk/bin/ar
   export CFLAGS_wasm32_wasip1="--sysroot=/path/to/wasi-sdk/share/wasi-sysroot"
   ```

## Build

```bash
cd echo
lmsk generate -lang rust ./src
cargo build --target wasm32-wasip1 --release
lmsk sign target/wasm32-wasip1/release/echo_skill.wasm
```

## Skills

| Skill | Description |
|-------|-------------|
| [echo](echo/) | Echo, ping, reverse — ABI smoke test |
| [fs](fs/) | File system operations (read, write, edit, search) |
| [web](web/) | HTTP/HTTPS outbound requests |
| [web-search](web-search/) | Web search via Tavily API |
| [git](git/) | Git repository operations |
| [archive](archive/) | Create and extract tar.gz/zip archives |
| [file-transfer](file-transfer/) | Copy files between cluster nodes |
| [python-engine](python-engine/) | Sandboxed Python 3 execution |
| [tree-sitter](tree-sitter/) | Source code parsing to AST/symbols |
| [intent-classifier](intent-classifier/) | Intent classification for request routing (ONNX + Tract) |
