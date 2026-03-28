# Python Toolchain Skill (AS)

Python development environment with venv, pip, pytest.

## Methods

`venv`, `pip_install`, `pip_freeze`, `run_script`, `test`, `run`, `ps`, `kill`.

## Build

```bash
npm install
lmsk generate -lang as .
npx asc assembly/lib.ts --target release
lmsk sign dist/python-skill.wasm
```

## Manifest

```yaml
id: python-toolchain
path: skills/ai.luminarys.as.python-toolchain.skill
permissions:
  fs: true
  shell: true
  shell_allowlist:
    - "python -m venv .venv"
    - ".venv/bin/pip **"
    - ".venv/bin/python **"
    - ".venv/bin/pytest **"
    - "ps **"
    - "kill **"
    - "./**"
```
