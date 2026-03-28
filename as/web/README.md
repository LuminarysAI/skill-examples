# Web Skill (AS)

HTTP/HTTPS outbound requests.

## Methods

`get`, `get_json`, `post`, `head`, `request`.

## Build

```bash
npm install
lmsk generate -lang as .
npx asc assembly/lib.ts --target release
lmsk sign dist/web-skill.wasm
```

## Manifest

```yaml
id: web-skill
path: skills/ai.luminarys.as.web.skill
permissions:
  http: true
  allowlist:
    - "**"
```
