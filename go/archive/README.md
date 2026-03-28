# Archive Skill (Go)

Create and extract tar.gz and zip archives.

## Methods

| Method | Description |
|--------|-------------|
| `pack` | Create an archive from files/directories |
| `unpack` | Extract an archive |
| `list` | List archive contents |

## Build

```bash
lmsk generate .
GOOS=wasip1 GOARCH=wasm go build -buildmode=c-shared -o archive-skill.wasm .
lmsk sign archive-skill.wasm
```

## Manifest

```yaml
id: archive-skill
path: skills/ai.luminarys.go.archive.skill
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
