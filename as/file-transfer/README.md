# File Transfer Skill (AS)

Copy files between cluster nodes.

## Methods

`copy`, `list`, `nodes`.

## Build

```bash
npm install
lmsk generate -lang as .
npx asc assembly/lib.ts --target release
lmsk sign dist/file-transfer-skill.wasm
```

## Manifest

```yaml
id: file-transfer
path: skills/ai.luminarys.as.file-transfer.skill
permissions:
  fs: true
  file_transfer: true
```
