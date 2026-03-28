# Go Toolchain Skill (Go)

Go development environment with process management. Wraps the Go toolchain and provides process lifecycle control.

## Methods

| Method | Description |
|--------|-------------|
| `mod_init` | Initialize a Go module |
| `mod_tidy` | Tidy module dependencies |
| `get` | Download module dependencies |
| `build` | Build Go packages |
| `test` | Run tests |
| `fmt` | Format Go source code |
| `vet` | Run Go vet analysis |
| `run` | Run a Go program |
| `ps` | List running processes |
| `kill` | Kill a running process |

## Build

```bash
lmsk generate .
GOOS=wasip1 GOARCH=wasm go build -buildmode=c-shared -o go-skill.wasm .
lmsk sign go-skill.wasm
```

## Manifest

```yaml
id: go-toolchain
path: skills/ai.luminarys.go.go-toolchain.skill
permissions:
  fs:
    enabled: true
    dirs: ["/data:rw"]
  shell:
    enabled: true
    allowlist: ["go **", "gofmt **", "ps **", "kill **", "./**"]
    allowed_dirs: ["/data"]
invoke_policy:
  can_invoke: []
  can_be_invoked_by: ["*"]
mcp:
  mapping: per_method
```
