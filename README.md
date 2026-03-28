# Luminarys Skill Examples

Example WASM skills for the [Luminarys](https://luminarys.ai) modular AI agent platform. Each skill is a sandboxed WebAssembly module accessible via MCP (Model Context Protocol), ACP (Agent Context Protocol), and the built-in AI agent.

## Languages

| Language | Directory | SDK |
|----------|-----------|-----|
| AssemblyScript | [as/](as/) | [@luminarys/sdk-as](https://www.npmjs.com/package/@luminarys/sdk-as) |
| Go | [go/](go/) | [sdk-go](https://github.com/LuminarysAI/sdk-go) |
| Rust | [rust/](rust/) | [luminarys-sdk](https://crates.io/crates/luminarys-sdk) |

## Quick Start

1. Install the `lmsk` CLI from [Luminarys releases](https://github.com/LuminarysAI/luminarys/releases)
2. Generate a signing key: `lmsk genkey`
3. Build any skill:

```bash
# AssemblyScript
cd as/echo && npm install && lmsk generate -lang as . && npx asc assembly/lib.ts --target release && lmsk sign dist/echo.wasm

# Go
cd go/echo && lmsk generate . && GOOS=wasip1 GOARCH=wasm go build -buildmode=c-shared -o echo.wasm . && lmsk sign echo.wasm

# Rust
cd rust/echo && lmsk generate -lang rust ./src && cargo build --target wasm32-wasip1 --release && lmsk sign target/wasm32-wasip1/release/echo_skill.wasm
```

4. Add the signed `.skill` file and a manifest to your Luminarys config:

```yaml
# config/skills/echo.yaml
id: echo-skill
path: skills/ai.luminarys.rust.echo.skill
permissions:
  fs: { enabled: false }
invoke_policy:
  can_invoke: []
  can_be_invoked_by: ["*"]
mcp:
  mapping: per_method
```

## Skills

| Skill | AS | Go | Rust | Description |
|-------|----|----|------|-------------|
| echo | [as/echo](as/echo/) | [go/echo](go/echo/) | [rust/echo](rust/echo/) | Echo, ping, reverse — ABI smoke test |
| fs | [as/fs](as/fs/) | [go/fs](go/fs/) | [rust/fs](rust/fs/) | File system operations |
| web | [as/web](as/web/) | [go/web](go/web/) | [rust/web](rust/web/) | HTTP/HTTPS outbound requests |
| web-search | [as/web-search](as/web-search/) | [go/web-search](go/web-search/) | [rust/web-search](rust/web-search/) | Web search |
| git | [as/git](as/git/) | [go/git](go/git/) | [rust/git](rust/git/) | Git repository operations |
| archive | [as/archive](as/archive/) | [go/archive](go/archive/) | [rust/archive](rust/archive/) | Create and extract archives |
| file-transfer | [as/file-transfer](as/file-transfer/) | [go/file-transfer](go/file-transfer/) | [rust/file-transfer](rust/file-transfer/) | Copy files between cluster nodes |
| go-toolchain | [as/go](as/go/) | [go/go](go/go/) | — | Go development environment |
| python-toolchain | [as/python](as/python/) | — | — | Python development environment |
| js-engine | — | [go/js-engine](go/js-engine/) | — | Sandboxed JavaScript execution |
| python-engine | — | — | [rust/python-engine](rust/python-engine/) | Sandboxed Python execution |
| tree-sitter | — | — | [rust/tree-sitter](rust/tree-sitter/) | Source code parsing to AST |
| intent-classifier | — | — | [rust/intent-classifier](rust/intent-classifier/) | Intent classification for routing (ONNX) |

## License

SDK is MIT. See individual skill directories for details.
