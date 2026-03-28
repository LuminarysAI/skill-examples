# Archive Skill (AS)

Create and extract tar.gz/zip archives.

## Methods

`pack`, `unpack`, `list`.

## Build

```bash
npm install
lmsk generate -lang as .
npx asc assembly/lib.ts --target release
lmsk sign dist/archive-skill.wasm
```

## Manifest

```yaml
id: archive-skill
path: skills/ai.luminarys.as.archive.skill
permissions:
  fs: true
```
