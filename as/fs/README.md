# File System Skill (AS)

Sandboxed filesystem operations.

## Methods

`read`, `write`, `create`, `delete`, `mkdir`, `ls`, `copy`, `read_lines`, `grep`, `glob`, `chmod`, `allowed_dirs`.

## Build

```bash
npm install
lmsk generate -lang as .
npx asc assembly/lib.ts --target release
lmsk sign dist/fs-skill.wasm
```

## Manifest

```yaml
id: fs-skill
path: skills/ai.luminarys.as.fs.skill
permissions:
  fs: true
  dirs:
    - "/data:rw"
```
