# Web Search Skill (AS)

Web search via DuckDuckGo Instant Answer API.

## Methods

`search`.

## Build

```bash
npm install
lmsk generate -lang as .
npx asc assembly/lib.ts --target release
lmsk sign dist/web-search-skill.wasm
```

## Manifest

```yaml
id: web-search
path: skills/ai.luminarys.as.web-search.skill
permissions:
  http: true
  allowlist:
    - "https://**"
```
