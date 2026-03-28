# Git Skill

Git repository operations via shell commands.

## Methods

| Method | Parameters | Description |
|--------|-----------|-------------|
| `init` | `dir` | Initialize a new repository |
| `status` | `dir` | Working tree status |
| `diff` | `dir`, `ref`, `paths` | Show changes as unified diff |
| `log` | `dir`, `max_count`, `ref`, `paths` | Commit history |
| `show` | `dir`, `ref`, `paths` | Commit details and diff |
| `blame` | `dir`, `path` | Line-by-line authorship |
| `branches` | `dir` | List branches |
| `tags` | `dir` | List tags |
| `diff_stat` | `dir`, `ref_from`, `ref_to` | Change summary between refs |
| `add` | `dir`, `paths` | Stage files |
| `commit` | `dir`, `message` | Create commit |
| `create_branch` | `dir`, `name`, `start_point` | Create and switch to branch |
| `checkout` | `dir`, `ref` | Switch branch/tag/commit |
| `stash` | `dir`, `message` | Stash uncommitted changes |
| `stash_pop` | `dir` | Restore most recent stash |
| `stash_list` | `dir` | List stash entries |
| `restore` | `dir`, `paths` | Discard uncommitted changes |

## Build

```bash
lmsk generate -lang rust ./src
cargo build --target wasm32-wasip1 --release
lmsk sign target/wasm32-wasip1/release/git_skill.wasm
```

## Manifest

```yaml
id: git-skill
path: skills/ai.luminarys.rust.git.skill
permissions:
  fs:
    enabled: true
    dirs: ["/data:rw"]
  shell:
    enabled: true
    allowlist: ["git **"]
    allowed_dirs: ["/data"]
invoke_policy:
  can_invoke: []
  can_be_invoked_by: ["*"]
mcp:
  mapping: per_method
```
