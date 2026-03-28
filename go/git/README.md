# Git Skill (Go)

Git repository operations. Provides read and write access to Git repositories through shell-based git commands.

## Methods

| Method | Description |
|--------|-------------|
| `init` | Initialize a new repository |
| `status` | Show working tree status |
| `diff` | Show changes between commits/working tree |
| `log` | Show commit log |
| `show` | Show commit details |
| `blame` | Show line-by-line authorship |
| `branches` | List branches |
| `tags` | List tags |
| `diff_stat` | Show diff statistics |
| `add` | Stage files |
| `commit` | Create a commit |
| `create_branch` | Create a new branch |
| `checkout` | Switch branches or restore files |
| `stash` | Stash changes |
| `stash_pop` | Apply and drop top stash entry |
| `stash_list` | List stash entries |
| `restore` | Restore working tree files |

## Build

```bash
lmsk generate .
GOOS=wasip1 GOARCH=wasm go build -buildmode=c-shared -o git-skill.wasm .
lmsk sign git-skill.wasm
```

## Manifest

```yaml
id: git-skill
path: skills/ai.luminarys.go.git.skill
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
