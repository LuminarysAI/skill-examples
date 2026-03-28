# Echo Skill

ABI compatibility smoke-test. Echoes input, reverses strings, health-checks.

## Methods

| Method | Parameters | Description |
|--------|-----------|-------------|
| `echo` | `message` (string, required) | Return the input string unchanged |
| `ping` | — | Health check, returns "pong" |
| `reverse` | `message` (string, required) | Reverse characters of a string |

## Build

```bash
lmsk generate -lang rust ./src
cargo build --target wasm32-wasip1 --release
lmsk sign target/wasm32-wasip1/release/echo_skill.wasm
```

## Manifest

```yaml
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
