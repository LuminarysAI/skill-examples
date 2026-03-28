# AssemblyScript Skills

WASM skills built with the [Luminarys AS SDK](https://www.npmjs.com/package/@luminarys/sdk-as).

## Prerequisites

- Node.js (18+)
- `lmsk` CLI (skill toolchain)
- Signing key: `lmsk genkey`

## Build

```bash
cd echo
npm install
lmsk generate -lang as .
npx asc assembly/lib.ts --target release
lmsk sign dist/echo-skill.wasm
```

## Skills

| Skill | Description |
|-------|-------------|
| [echo](echo/) | Echo, ping, reverse, word count + ABI probes |
| [fs](fs/) | File system operations |
| [web](web/) | HTTP/HTTPS outbound requests |
| [web-search](web-search/) | Web search via DuckDuckGo |
| [git](git/) | Git repository operations |
| [archive](archive/) | Create and extract archives |
| [file-transfer](file-transfer/) | Copy files between cluster nodes |
| [go](go/) | Go toolchain and process management |
| [python](python/) | Python toolchain and process management |
