//! git-skill — Git repository operations via shell_exec (Rust).
//!
//! @skill:id      ai.luminarys.rust.git
//! @skill:name    "Git Skill (Rust)"
//! @skill:version 1.0.0
//! @skill:desc    "Git repository operations: status, diff, log, blame, commit, branch management."
//!
//! @skill:require shell git **

use luminarys_sdk::prelude::*;

// ── helpers ───────────────────────────────────────────────────────────────────

fn git(workdir: &str, args: &[&str]) -> Result<String, SkillError> {
    let cmd = format!("git {}", args.join(" "));
    let result = shell_exec(&ShellExecRequest {
        command: cmd.clone(),
        workdir: workdir.into(),
        timeout_ms: 30000,
        tail: 0,
        grep: String::new(),
        as_daemon: false,
        log_file: String::new(),
    })?;
    if result.exit_code != 0 {
        let msg = result.output.trim().to_string();
        return Err(SkillError(format!(
            "git {}: exit {}: {}",
            args[0], result.exit_code, msg
        )));
    }
    Ok(result.output)
}

// ── Read-only operations ──────────────────────────────────────────────────────

/// Initialize a new git repository.
/// @skill:method init "Initialize a new git repository."
/// @skill:param  workdir required "Absolute path to the directory to initialize"
/// @skill:result "Initialization confirmation"
pub fn init(_ctx: &mut Context, workdir: String) -> Result<String, SkillError> {
    git(&workdir, &["init"])
}

/// Show working tree status.
/// @skill:method status "Show working tree status: staged, unstaged, and untracked files."
/// @skill:param  workdir required "Absolute path to the git repository"
/// @skill:result "Porcelain status output"
pub fn status(_ctx: &mut Context, workdir: String) -> Result<String, SkillError> {
    git(&workdir, &["status", "--porcelain=v2", "--branch"])
}

/// Show file changes as unified diff.
/// @skill:method diff "Show file changes as unified diff."
/// @skill:param  workdir required "Absolute path to the git repository"
/// @skill:param  path    optional "Limit diff to this file or directory"
/// @skill:param  staged  optional "Show staged changes instead of unstaged"
/// @skill:result "Unified diff output"
pub fn diff(_ctx: &mut Context, workdir: String, path: String, staged: bool) -> Result<String, SkillError> {
    let mut args = vec!["diff", "--no-color"];
    if staged {
        args.push("--cached");
    }
    if !path.is_empty() {
        args.push("--");
        args.push(&path);
    }
    git(&workdir, &args)
}

/// Show commit history.
/// @skill:method log "Show commit history."
/// @skill:param  workdir required "Absolute path to the git repository"
/// @skill:param  count   optional "Number of commits to show"
/// @skill:param  path    optional "Limit log to this file or directory"
/// @skill:param  oneline optional "One line per commit"
/// @skill:result "Git log output"
pub fn log(_ctx: &mut Context, workdir: String, count: i64, path: String, oneline: bool) -> Result<String, SkillError> {
    let n = if count <= 0 { 20 } else { count };
    let n_arg = format!("-n{}", n);
    let mut args = vec!["log", &n_arg, "--no-color"];
    if oneline {
        args.push("--oneline");
    } else {
        args.push("--format=format:%H %ai %an <%ae>%n%s%n");
    }
    if !path.is_empty() {
        args.push("--");
        args.push(&path);
    }
    git(&workdir, &args)
}

/// Show commit details and diff.
/// @skill:method show "Show commit details and diff."
/// @skill:param  workdir required "Absolute path to the git repository"
/// @skill:param  ref     optional "Commit hash, tag, or branch name"
/// @skill:param  stat    optional "Show diffstat only"
/// @skill:result "Commit details and diff"
pub fn show(_ctx: &mut Context, workdir: String, r#ref: String, stat: bool) -> Result<String, SkillError> {
    let ref_val = if r#ref.is_empty() { "HEAD" } else { &r#ref };
    let mut args = vec!["show", "--no-color"];
    if stat {
        args.push("--stat");
    }
    args.push(ref_val);
    git(&workdir, &args)
}

/// Show who last modified each line of a file.
/// @skill:method blame "Show who last modified each line of a file."
/// @skill:param  workdir required "Absolute path to the git repository"
/// @skill:param  path    required "File path relative to repo root"
/// @skill:result "Blame output"
pub fn blame(_ctx: &mut Context, workdir: String, path: String) -> Result<String, SkillError> {
    git(&workdir, &["blame", "--no-color", "--", &path])
}

/// List all branches.
/// @skill:method branches "List all branches."
/// @skill:param  workdir required "Absolute path to the git repository"
/// @skill:result "Branch list"
pub fn branches(_ctx: &mut Context, workdir: String) -> Result<String, SkillError> {
    git(&workdir, &["branch", "-a", "--no-color"])
}

/// List all tags.
/// @skill:method tags "List all tags."
/// @skill:param  workdir required "Absolute path to the git repository"
/// @skill:result "Tag list"
pub fn tags(_ctx: &mut Context, workdir: String) -> Result<String, SkillError> {
    git(&workdir, &["tag", "-l"])
}

/// Show summary of changes between two refs.
/// @skill:method diff_stat "Show summary of changes between two refs."
/// @skill:param  workdir required "Absolute path to the git repository"
/// @skill:param  from    optional "Start ref"
/// @skill:param  to      optional "End ref"
/// @skill:result "Diffstat summary"
pub fn diff_stat(_ctx: &mut Context, workdir: String, from: String, to: String) -> Result<String, SkillError> {
    let f = if from.is_empty() { "HEAD~1".to_string() } else { from };
    let t = if to.is_empty() { "HEAD".to_string() } else { to };
    let range = format!("{}...{}", f, t);
    git(&workdir, &["diff", "--stat", "--no-color", &range])
}

// ── Write operations ──────────────────────────────────────────────────────────

/// Stage files for the next commit.
/// @skill:method add "Stage files for the next commit."
/// @skill:param  workdir required "Absolute path to the git repository"
/// @skill:param  paths   required "Space-separated file paths to stage (use . for all)"
/// @skill:result "Status after staging"
pub fn add(_ctx: &mut Context, workdir: String, paths: String) -> Result<String, SkillError> {
    let parts: Vec<&str> = paths.split_whitespace().collect();
    let mut args = vec!["add"];
    args.extend(parts);
    git(&workdir, &args)?;
    git(&workdir, &["status", "--porcelain=v2", "--branch"])
}

/// Create a commit with staged changes.
/// @skill:method commit "Create a commit with staged changes."
/// @skill:param  workdir required "Absolute path to the git repository"
/// @skill:param  message required "Commit message"
/// @skill:result "Commit hash and summary"
pub fn commit(_ctx: &mut Context, workdir: String, message: String) -> Result<String, SkillError> {
    let quoted = format!("\"{}\"", message.replace('"', "\\\""));
    git(&workdir, &["commit", "-m", &quoted])
}

/// Create a new branch and switch to it.
/// @skill:method create_branch "Create a new branch and switch to it."
/// @skill:param  workdir required "Absolute path to the git repository"
/// @skill:param  name    required "New branch name"
/// @skill:param  from    optional "Base ref to branch from"
/// @skill:result "Confirmation"
pub fn create_branch(_ctx: &mut Context, workdir: String, name: String, from: String) -> Result<String, SkillError> {
    let mut args = vec!["checkout", "-b", &name];
    if !from.is_empty() {
        args.push(&from);
    }
    git(&workdir, &args)
}

/// Switch to a branch, tag, or commit.
/// @skill:method checkout "Switch to a branch, tag, or commit."
/// @skill:param  workdir required "Absolute path to the git repository"
/// @skill:param  ref     required "Branch name, tag, or commit hash"
/// @skill:result "Confirmation"
pub fn checkout(_ctx: &mut Context, workdir: String, r#ref: String) -> Result<String, SkillError> {
    git(&workdir, &["checkout", &r#ref])
}

/// Stash uncommitted changes.
/// @skill:method stash "Stash uncommitted changes."
/// @skill:param  workdir required "Absolute path to the git repository"
/// @skill:param  message optional "Stash message"
/// @skill:result "Stash confirmation"
pub fn stash(_ctx: &mut Context, workdir: String, message: String) -> Result<String, SkillError> {
    let mut args = vec!["stash", "push"];
    if !message.is_empty() {
        args.push("-m");
        args.push(&message);
    }
    git(&workdir, &args)
}

/// Restore the most recently stashed changes.
/// @skill:method stash_pop "Restore the most recently stashed changes."
/// @skill:param  workdir required "Absolute path to the git repository"
/// @skill:result "Restored changes"
pub fn stash_pop(_ctx: &mut Context, workdir: String) -> Result<String, SkillError> {
    git(&workdir, &["stash", "pop"])
}

/// List all stash entries.
/// @skill:method stash_list "List all stash entries."
/// @skill:param  workdir required "Absolute path to the git repository"
/// @skill:result "Stash entries"
pub fn stash_list(_ctx: &mut Context, workdir: String) -> Result<String, SkillError> {
    git(&workdir, &["stash", "list"])
}

/// Discard uncommitted changes in a file.
/// @skill:method restore "Discard uncommitted changes in a file."
/// @skill:param  workdir required "Absolute path to the git repository"
/// @skill:param  path    required "File path to restore"
/// @skill:result "Confirmation"
pub fn restore(_ctx: &mut Context, workdir: String, path: String) -> Result<String, SkillError> {
    git(&workdir, &["checkout", "--", &path])?;
    Ok(format!("restored {}", path))
}
