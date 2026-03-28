/**
 * @skill:id      ai.luminarys.as.git
 * @skill:name    "Git Skill (AS)"
 * @skill:version 1.0.0
 * @skill:desc    "Git operations: init, add, commit, status, diff, log, branches, show, blame, tags."
 * @skill:sdk     "@luminarys/sdk-as"
 * @skill:require shell git **
 */

import { Context,  shellExec, ShellResult } from "@luminarys/sdk-as";

function git(workdir: string, args: string): string {
  const result: ShellResult = shellExec("git " + args, workdir);
  if (result.exit_code != 0) {
    return "ERROR: git " + args.split(" ")[0] + " (exit " + result.exit_code.toString() + "): " + result.output;
  }
  return result.output;
}

// @skill:method init "Initialize a new git repository."
// @skill:param  workdir required "Absolute path to the directory"
// @skill:result "Init confirmation"
export function init(_ctx: Context, workdir: string): string {
  return git(workdir, "init");
}

// @skill:method status "Show working tree status."
// @skill:param  workdir required "Absolute path to the git repository"
// @skill:result "Status output"
export function status(_ctx: Context, workdir: string): string {
  return git(workdir, "status --porcelain=v2 --branch");
}

// @skill:method diff "Show file changes as unified diff."
// @skill:param  workdir required "Absolute path to the git repository"
// @skill:param  staged  optional "Show staged changes (true/false)"
// @skill:result "Diff output"
export function diff(_ctx: Context, workdir: string, staged: bool): string {
  const args = staged ? "diff --cached --no-color" : "diff --no-color";
  return git(workdir, args);
}

// @skill:method log "Show commit history."
// @skill:param  workdir required "Absolute path to the git repository"
// @skill:param  count   optional "Number of commits to show"
// @skill:result "Log output"
export function log(_ctx: Context, workdir: string, count: i64): string {
  const n = count > 0 ? count : 20;
  return git(workdir, "log -n" + n.toString() + " --oneline --no-color");
}

// @skill:method show "Show commit details."
// @skill:param  workdir required "Absolute path to the git repository"
// @skill:param  ref     optional "Commit ref (default: HEAD)"
// @skill:result "Commit details"
export function show(_ctx: Context, workdir: string, ref: string): string {
  const r = ref.length > 0 ? ref : "HEAD";
  return git(workdir, "show " + r + " --no-color --stat");
}

// @skill:method blame "Show who last modified each line."
// @skill:param  workdir required "Absolute path to the git repository"
// @skill:param  path    required "File path relative to repo root"
// @skill:result "Blame output"
export function blame(_ctx: Context, workdir: string, path: string): string {
  return git(workdir, "blame " + path + " --no-color");
}

// @skill:method branches "List all branches."
// @skill:param  workdir required "Absolute path to the git repository"
// @skill:result "Branch list"
export function branches(_ctx: Context, workdir: string): string {
  return git(workdir, "branch -a --no-color");
}

// @skill:method tags "List all tags."
// @skill:param  workdir required "Absolute path to the git repository"
// @skill:result "Tag list"
export function tags(_ctx: Context, workdir: string): string {
  return git(workdir, "tag -l --no-color");
}

// @skill:method add "Stage files for commit."
// @skill:param  workdir required "Absolute path to the git repository"
// @skill:param  paths   required "File paths to stage (space-separated, use . for all)"
// @skill:result "Confirmation"
export function add(_ctx: Context, workdir: string, paths: string): string {
  git(workdir, "add " + paths);
  return "staged: " + paths;
}

// @skill:method commit "Create a commit with staged changes."
// @skill:param  workdir required "Absolute path to the git repository"
// @skill:param  message required "Commit message"
// @skill:result "Commit result"
export function commit(_ctx: Context, workdir: string, message: string): string {
  return git(workdir, 'commit -m "' + message + '"');
}

// @skill:method create_branch "Create and switch to a new branch."
// @skill:param  workdir required "Absolute path to the git repository"
// @skill:param  name    required "Branch name"
// @skill:result "Confirmation"
export function createBranch(_ctx: Context, workdir: string, name: string): string {
  git(workdir, "checkout -b " + name);
  return "created branch: " + name;
}

// @skill:method checkout "Switch to a branch, tag, or commit."
// @skill:param  workdir required "Absolute path to the git repository"
// @skill:param  ref     required "Branch, tag, or commit to switch to"
// @skill:result "Confirmation"
export function checkout(_ctx: Context, workdir: string, ref: string): string {
  return git(workdir, "checkout " + ref);
}
