# File System Skill

Sandboxed file system operations. All paths must be absolute.

## Methods

| Method | Parameters | Description |
|--------|-----------|-------------|
| `read` | `path` | Read full file contents |
| `read_lines` | `path`, `start_line`, `end_line` | Read a range of lines |
| `write` | `path`, `content` | Write/overwrite a file |
| `append_line` | `path`, `line` | Append a line to a file |
| `edit` | `path`, `old_string`, `new_string`, `replace_all` | Replace text in a file |
| `delete` | `path` | Delete a file |
| `mkdir` | `path` | Create directory with parents |
| `list` | `path`, `show_hidden` | List directory contents |
| `stat` | `path` | File/directory metadata |
| `chmod` | `path`, `mode`, `recursive` | Change permissions |
| `move` | `src`, `dst` | Move or rename |
| `copy` | `src`, `dst` | Copy a file |
| `count_lines` | `path` | Count lines in file |
| `find` | `dir`, `pattern`, `type` | Find files by glob |
| `tree` | `dir`, `max_depth` | Directory tree view |
| `search` | `dir`, `pattern`, `glob`, `context_lines` | Search text in files |
| `search_code` | `dir`, `pattern`, `glob`, `context_lines` | Search in source code only |
| `search_files` | `dir`, `pattern`, `glob` | Return file paths matching pattern |
| `search_in_file` | `path`, `pattern`, `context_lines`, `max_matches` | Search within a single file |

## Build

```bash
lmsk generate -lang rust ./src
cargo build --target wasm32-wasip1 --release
lmsk sign target/wasm32-wasip1/release/fs_skill.wasm
```

## Manifest

```yaml
id: fs-skill
path: skills/ai.luminarys.rust.fs.skill
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
