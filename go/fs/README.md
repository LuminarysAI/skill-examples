# File System Skill (Go)

Sandboxed file system operations with unified diff on writes. All paths are restricted to directories declared in the manifest.

## Methods

| Method | Description |
|--------|-------------|
| `read` | Read entire file contents |
| `read_lines` | Read a range of lines from a file |
| `write` | Write content to a file |
| `edit` | Apply a unified diff edit to a file |
| `append_line` | Append a line to a file |
| `delete` | Delete a file or directory |
| `mkdir` | Create a directory |
| `ls` | List directory contents |
| `stat` | Get file metadata |
| `move` | Move/rename a file or directory |
| `copy` | Copy a file or directory |
| `count_lines` | Count lines in a file |
| `search` | Search file contents (grep-like) |
| `find` | Find files by name pattern |
| `tree` | Display directory tree |
| `chmod` | Change file permissions |
| `allowed_dirs` | List allowed directories |

## Build

```bash
lmsk generate .
GOOS=wasip1 GOARCH=wasm go build -buildmode=c-shared -o fs-skill.wasm .
lmsk sign fs-skill.wasm
```

## Manifest

```yaml
id: fs-skill
path: skills/ai.luminarys.go.fs.skill
permissions:
  fs:
    enabled: true
    dirs: ["/data:rw"]
invoke_policy:
  can_invoke: []
  can_be_invoked_by: ["*"]
mcp:
  mapping: per_method
```
