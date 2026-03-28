# Python Engine Skill

Sandboxed Python 3 execution using built-in functions only.

## Methods

| Method | Parameters | Description |
|--------|-----------|-------------|
| `execute` | `code` (string, required) | Execute Python code and return output |

Available built-ins: `print`, `len`, `range`, `list`, `dict`, `tuple`, `set`, `str`, `int`, `float`, `bool`, `abs`, `min`, `max`, `sum`, `round`, `pow`, `sorted`, `reversed`, `enumerate`, `zip`, `map`, `filter`, `any`, `all`, `isinstance`, `type`, `repr`, `hash`, `hex`, `oct`, `bin`, `chr`, `ord`.

No imports or standard library access.

## Build

```bash
lmsk generate -lang rust ./src
cargo build --target wasm32-wasi --no-default-features --features compiler --release
lmsk sign target/wasm32-wasip1/release/python_engine_skill.wasm
```

## Manifest

```yaml
id: python-engine
path: skills/ai.luminarys.rust.python-engine.skill
permissions:
  fs: { enabled: false }
invoke_policy:
  can_invoke: []
  can_be_invoked_by: ["*"]
mcp:
  mapping: per_method
```
