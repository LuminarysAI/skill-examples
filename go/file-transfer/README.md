# File Transfer Skill (Go)

Copy files between cluster nodes. Enables cross-node file operations in a multi-agent cluster.

## Methods

| Method | Description |
|--------|-------------|
| `copy` | Copy a file to/from another node |
| `list` | List files on a remote node |
| `nodes` | List available cluster nodes |

## Build

```bash
lmsk generate .
GOOS=wasip1 GOARCH=wasm go build -buildmode=c-shared -o file-transfer-skill.wasm .
lmsk sign file-transfer-skill.wasm
```

## Manifest

```yaml
id: file-transfer
path: skills/ai.luminarys.go.file-transfer.skill
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
