# Echo Skill (Go)

ABI compatibility smoke-test. Useful for verifying that the host correctly loads a WASM skill, dispatches methods, and returns results.

## Methods

| Method | Description |
|--------|-------------|
| `echo` | Returns the input text as-is |
| `ping` | Returns `pong` |
| `reverse` | Returns the input text reversed |

## Build

```bash
lmsk generate .
GOOS=wasip1 GOARCH=wasm go build -buildmode=c-shared -o echo-skill.wasm .
lmsk sign echo-skill.wasm
```

## Manifest

```yaml
id: echo-skill
path: skills/ai.luminarys.go.echo.skill
permissions:
  fs: { enabled: false }
invoke_policy:
  can_invoke: []
  can_be_invoked_by: ["*"]
mcp:
  mapping: per_method
```
