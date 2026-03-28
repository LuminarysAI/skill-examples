# Web Search Skill

Web search via Tavily API.

## Methods

| Method | Parameters | Description |
|--------|-----------|-------------|
| `search` | `query` (string, required) | Execute web search, return JSON results |

## Environment

Requires `TAVILY_API_KEY` in the deployment manifest:

```yaml
env:
  TAVILY_API_KEY: "${TAVILY_API_KEY}"
```

## Build

```bash
lmsk generate -lang rust ./src
cargo build --target wasm32-wasip1 --release
lmsk sign target/wasm32-wasip1/release/web_search_skill.wasm
```

## Manifest

```yaml
id: web-search
path: skills/ai.luminarys.rust.web-search.skill
permissions:
  http:
    enabled: true
    allowlist: ["https://api.tavily.com/**"]
env:
  TAVILY_API_KEY: "${TAVILY_API_KEY}"
invoke_policy:
  can_invoke: []
  can_be_invoked_by: ["*"]
mcp:
  mapping: per_method
```
