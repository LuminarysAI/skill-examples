# Go Toolchain Skill (AS)

Go development environment with process management.

## Methods

`mod_init`, `mod_tidy`, `get`, `build`, `test`, `fmt`, `vet`, `run`, `ps`, `kill`.

## Build

```bash
npm install
lmsk generate -lang as .
npx asc assembly/lib.ts --target release
lmsk sign dist/go-skill.wasm
```

## Manifest

```yaml
id: go-toolchain
path: skills/ai.luminarys.as.go-toolchain.skill
permissions:
  fs: true
  shell: true
  shell_allowlist:
    - "go **"
    - "gofmt **"
    - "ps **"
    - "kill **"
    - "./**"
```
