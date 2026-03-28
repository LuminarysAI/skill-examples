# Web Skill (Go)

HTTP/HTTPS outbound requests. Supports GET, POST, HEAD, and arbitrary HTTP methods with headers and body.

## Methods

| Method | Description |
|--------|-------------|
| `get` | HTTP GET request, returns body as text |
| `get_json` | HTTP GET request, returns parsed JSON |
| `post` | HTTP POST request with body |
| `request` | Arbitrary HTTP request (any method) |
| `head` | HTTP HEAD request, returns headers only |

## Build

```bash
lmsk generate .
GOOS=wasip1 GOARCH=wasm go build -buildmode=c-shared -o web-skill.wasm .
lmsk sign web-skill.wasm
```

## Manifest

```yaml
id: web-skill
path: skills/ai.luminarys.go.web.skill
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
