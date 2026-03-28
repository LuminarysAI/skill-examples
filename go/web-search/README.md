# Web Search Skill (Go)

Web search via the Tavily API. Requires a `TAVILY_API_KEY` environment variable.

## Methods

| Method | Description |
|--------|-------------|
| `search` | Search the web and return results |

## Build

```bash
lmsk generate .
GOOS=wasip1 GOARCH=wasm go build -buildmode=c-shared -o web-search-skill.wasm .
lmsk sign web-search-skill.wasm
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `TAVILY_API_KEY` | API key for Tavily search service |

## Manifest

```yaml
id: web-search
path: skills/ai.luminarys.go.web-search.skill
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
