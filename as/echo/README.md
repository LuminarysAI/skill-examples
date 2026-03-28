# Echo Skill (AS)

ABI compatibility test. Echoes input, reverses strings, probes host ABI (sys_info, time_now, etc.).

## Methods

`echo`, `ping`, `reverse`, `word_count`, `sys_info`, `time_now`, `disk_usage`, `env_get`, `cwd_test`, `tcp_ping`, `log_test`.

## Build

```bash
npm install
lmsk generate -lang as .
npx asc assembly/lib.ts --target release
lmsk sign dist/echo-skill.wasm
```

## Manifest

```yaml
id: echo-skill
path: skills/ai.luminarys.as.echo.skill
permissions:
  # Basic methods (echo, ping, reverse, word_count) need no special permissions.
  # ABI probe methods need additional capabilities:
  fs: true
  shell: true
  tcp: true
```
