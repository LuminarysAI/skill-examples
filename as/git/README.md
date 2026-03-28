# Git Skill (AS)

Git repository operations via shell commands.

## Methods

`init`, `status`, `diff`, `log`, `show`, `blame`, `branches`, `tags`, `add`, `commit`, `create_branch`, `checkout`.

## Build

```bash
npm install
lmsk generate -lang as .
npx asc assembly/lib.ts --target release
lmsk sign dist/git-skill.wasm
```

## Manifest

```yaml
id: git-skill
path: skills/ai.luminarys.as.git.skill
permissions:
  fs: true
  shell: true
  shell_allowlist:
    - "git **"
```
