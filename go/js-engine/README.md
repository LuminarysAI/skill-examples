# JavaScript Engine Skill (Go)

Sandboxed JavaScript execution using the goja runtime. Supports `console.log` output capture. No network or filesystem access from within the JS sandbox.

## Methods

| Method | Description |
|--------|-------------|
| `execute` | Execute JavaScript code and return the result |

## Build

```bash
lmsk generate .
GOOS=wasip1 GOARCH=wasm go build -buildmode=c-shared -o js-engine-skill.wasm .
lmsk sign js-engine-skill.wasm
```

## Manifest

```yaml
id: js-engine
path: skills/ai.luminarys.go.js-engine.skill
permissions:
  fs: { enabled: false }
invoke_policy:
  can_invoke: []
  can_be_invoked_by: ["*"]
mcp:
  mapping: per_method
```
