# File Transfer Skill

Copy files between cluster nodes.

## Methods

| Method | Parameters | Description |
|--------|-----------|-------------|
| `copy` | `src`, `dst` | Copy files locally or between nodes (`node-id:///path`) |
| `list` | `dir` | List directory contents |
| `nodes` | — | List known cluster nodes |

## Build

```bash
lmsk generate -lang rust ./src
cargo build --target wasm32-wasip1 --release
lmsk sign target/wasm32-wasip1/release/file_transfer_skill.wasm
```

## Manifest

```yaml
id: file-transfer
path: skills/ai.luminarys.rust.file-transfer.skill
permissions:
  fs:
    enabled: true
    dirs: ["/data:rw"]
  file_transfer:
    enabled: true
    allowed_nodes: ["*"]
    local_dirs: ["/data:rw"]
invoke_policy:
  can_invoke: []
  can_be_invoked_by: ["*"]
mcp:
  mapping: per_method
```
