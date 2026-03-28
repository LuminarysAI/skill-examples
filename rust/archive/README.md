# Archive Skill

Create and extract tar.gz/zip archives.

## Methods

| Method | Parameters | Description |
|--------|-----------|-------------|
| `pack` | `dir`, `output`, `format`, `include`, `exclude` | Create archive |
| `unpack` | `archive`, `output`, `format`, `strip`, `include`, `exclude` | Extract archive |
| `list` | `archive`, `format`, `include`, `exclude` | List archive contents |

Supported formats: `tar.gz` (default), `zip`.

## Build

```bash
lmsk generate -lang rust ./src
cargo build --target wasm32-wasip1 --release
lmsk sign target/wasm32-wasip1/release/archive_skill.wasm
```

## Manifest

```yaml
id: archive-skill
path: skills/ai.luminarys.rust.archive.skill
permissions:
  fs:
    enabled: true
    dirs: ["/data:rw"]
invoke_policy:
  can_invoke: []
  can_be_invoked_by: ["*"]
mcp:
  mapping: per_method
```
