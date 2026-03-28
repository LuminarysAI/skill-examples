# Web Skill

HTTP/HTTPS outbound requests.

## Methods

| Method | Parameters | Description |
|--------|-----------|-------------|
| `get` | `url`, `headers` | HTTP GET |
| `get_json` | `url`, `headers` | GET with JSON pretty-print |
| `post` | `url`, `body`, `headers` | POST with JSON body |
| `request` | `method`, `url`, `headers`, `body`, `timeout_ms` | Custom HTTP request |
| `head` | `url`, `headers` | HEAD request (status + headers) |

## Build

```bash
lmsk generate -lang rust ./src
cargo build --target wasm32-wasip1 --release
lmsk sign target/wasm32-wasip1/release/web_skill.wasm
```

## Manifest

```yaml
id: web-skill
path: skills/ai.luminarys.rust.web.skill
permissions:
  http:
    enabled: true
    allowlist: ["**"]
invoke_policy:
  can_invoke: []
  can_be_invoked_by: ["*"]
mcp:
  mapping: per_method
```
