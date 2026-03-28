// Package main implements git-skill — Git operations via shell_exec.
//
// @skill:id      ai.luminarys.go.git
// @skill:name    "Git Skill"
// @skill:version 1.0.0
// @skill:desc    "Git repository operations: status, diff, log, blame, commit, branch management."
//
// @skill:require shell git **
//
//go:generate lmsk -lang go -verbose .
package main

import (
	"fmt"
	"strings"

	sdk "github.com/LuminarysAI/sdk-go"
)

// ── helpers ───────────────────────────────────────────────────────────────────

func git(ctx *sdk.Context, workdir string, args ...string) (sdk.ShellExecResult, error) {
	cmd := "git " + strings.Join(args, " ")
	return sdk.ShellExec(sdk.ShellExecRequest{
		Command:   cmd,
		Workdir:   workdir,
		TimeoutMs: 30000,
	})
}

func gitOutput(ctx *sdk.Context, workdir string, args ...string) (string, error) {
	result, err := git(ctx, workdir, args...)
	if err != nil {
		return "", err
	}
	if result.ExitCode != 0 {
		msg := strings.TrimSpace(result.Output)
		return "", fmt.Errorf("git %s: exit %d: %s", args[0], result.ExitCode, msg)
	}
	return result.Output, nil
}

// ── Read-only operations ──────────────────────────────────────────────────────

// Init initializes a new git repository in the given directory.
// @skill:method init "Initialize a new git repository."
// @skill:param  workdir required "Absolute path to the directory to initialize"
// @skill:result "Initialization confirmation"
func Init(ctx *sdk.Context, workdir string) (string, error) {
	return gitOutput(ctx, workdir, "init")
}

// Status returns the working tree status (porcelain v2 format).
// @skill:method status "Show working tree status: staged, unstaged, and untracked files."
// @skill:param  workdir required "Absolute path to the git repository"
// @skill:result "Porcelain status output, one file per line"
func Status(ctx *sdk.Context, workdir string) (string, error) {
	return gitOutput(ctx, workdir, "status", "--porcelain=v2", "--branch")
}

// Diff returns the diff of unstaged changes, or staged changes if staged=true.
// @skill:method diff "Show file changes as unified diff."
// @skill:param  workdir required "Absolute path to the git repository"
// @skill:param  path    optional "Limit diff to this file or directory"
// @skill:param  staged  optional "Show staged (cached) changes instead of unstaged" default:false
// @skill:result "Unified diff output"
func Diff(ctx *sdk.Context, workdir, path string, staged bool) (string, error) {
	args := []string{"diff", "--no-color"}
	if staged {
		args = append(args, "--cached")
	}
	if path != "" {
		args = append(args, "--", path)
	}
	return gitOutput(ctx, workdir, args...)
}

// Log returns recent commit history.
// @skill:method log "Show commit history."
// @skill:param  workdir required "Absolute path to the git repository"
// @skill:param  count   optional "Number of commits to show" default:20
// @skill:param  path    optional "Limit log to this file or directory"
// @skill:param  oneline optional "One line per commit (hash + subject)" default:false
// @skill:result "Git log output"
func Log(ctx *sdk.Context, workdir string, count int64, path string, oneline bool) (string, error) {
	n := count
	if n <= 0 {
		n = 20
	}
	args := []string{"log", fmt.Sprintf("-n%d", n), "--no-color"}
	if oneline {
		args = append(args, "--oneline")
	} else {
		args = append(args, "--format=format:%H %ai %an <%ae>%n%s%n")
	}
	if path != "" {
		args = append(args, "--", path)
	}
	return gitOutput(ctx, workdir, args...)
}

// Show displays the contents of a commit.
// @skill:method show "Show commit details and diff."
// @skill:param  workdir required "Absolute path to the git repository"
// @skill:param  ref     optional "Commit hash, tag, or branch name" default:HEAD
// @skill:param  stat    optional "Show diffstat only (no full diff)" default:false
// @skill:result "Commit details and diff"
func Show(ctx *sdk.Context, workdir, ref string, stat bool) (string, error) {
	if ref == "" {
		ref = "HEAD"
	}
	args := []string{"show", "--no-color", ref}
	if stat {
		args = []string{"show", "--no-color", "--stat", ref}
	}
	return gitOutput(ctx, workdir, args...)
}

// Blame shows line-by-line authorship for a file.
// @skill:method blame "Show who last modified each line of a file."
// @skill:param  workdir required "Absolute path to the git repository"
// @skill:param  path    required "File path relative to repo root"
// @skill:result "Blame output with commit, author, and line content"
func Blame(ctx *sdk.Context, workdir, path string) (string, error) {
	return gitOutput(ctx, workdir, "blame", "--no-color", "--", path)
}

// Branches lists all branches (local and remote).
// @skill:method branches "List all branches."
// @skill:param  workdir required "Absolute path to the git repository"
// @skill:result "Branch list with current branch marked"
func Branches(ctx *sdk.Context, workdir string) (string, error) {
	return gitOutput(ctx, workdir, "branch", "-a", "--no-color")
}

// Tags lists all tags.
// @skill:method tags "List all tags."
// @skill:param  workdir required "Absolute path to the git repository"
// @skill:result "Tag list"
func Tags(ctx *sdk.Context, workdir string) (string, error) {
	return gitOutput(ctx, workdir, "tag", "-l")
}

// DiffStat returns diffstat summary (files changed, insertions, deletions).
// @skill:method diff_stat "Show summary of changes between two refs."
// @skill:param  workdir required "Absolute path to the git repository"
// @skill:param  from    optional "Start ref (commit, branch, tag)" default:HEAD~1
// @skill:param  to      optional "End ref" default:HEAD
// @skill:result "Diffstat summary"
func DiffStat(ctx *sdk.Context, workdir, from, to string) (string, error) {
	if from == "" {
		from = "HEAD~1"
	}
	if to == "" {
		to = "HEAD"
	}
	return gitOutput(ctx, workdir, "diff", "--stat", "--no-color", from+"..."+to)
}

// ── Write operations ──────────────────────────────────────────────────────────

// Add stages files for commit.
// @skill:method add "Stage files for the next commit."
// @skill:param  workdir required "Absolute path to the git repository"
// @skill:param  paths   required "Space-separated file paths to stage (use . for all)"
// @skill:result "Status after staging"
func Add(ctx *sdk.Context, workdir, paths string) (string, error) {
	args := append([]string{"add"}, strings.Fields(paths)...)
	if _, err := gitOutput(ctx, workdir, args...); err != nil {
		return "", err
	}
	return gitOutput(ctx, workdir, "status", "--porcelain=v2", "--branch")
}

// Commit creates a new commit with the given message.
// @skill:method commit "Create a commit with staged changes."
// @skill:param  workdir required "Absolute path to the git repository"
// @skill:param  message required "Commit message"
// @skill:result "Commit hash and summary"
func Commit(ctx *sdk.Context, workdir, message string) (string, error) {
	return gitOutput(ctx, workdir, "commit", "-m", fmt.Sprintf("%q", message))
}

// CreateBranch creates and switches to a new branch.
// @skill:method create_branch "Create a new branch and switch to it."
// @skill:param  workdir required "Absolute path to the git repository"
// @skill:param  name    required "New branch name"
// @skill:param  from    optional "Base ref to branch from" default:HEAD
// @skill:result "Confirmation message"
func CreateBranch(ctx *sdk.Context, workdir, name, from string) (string, error) {
	args := []string{"checkout", "-b", name}
	if from != "" {
		args = append(args, from)
	}
	return gitOutput(ctx, workdir, args...)
}

// Checkout switches to an existing branch or ref.
// @skill:method checkout "Switch to a branch, tag, or commit."
// @skill:param  workdir required "Absolute path to the git repository"
// @skill:param  ref     required "Branch name, tag, or commit hash"
// @skill:result "Confirmation message"
func Checkout(ctx *sdk.Context, workdir, ref string) (string, error) {
	return gitOutput(ctx, workdir, "checkout", ref)
}

// Stash saves uncommitted changes to the stash.
// @skill:method stash "Stash uncommitted changes."
// @skill:param  workdir required "Absolute path to the git repository"
// @skill:param  message optional "Stash message"
// @skill:result "Stash confirmation"
func Stash(ctx *sdk.Context, workdir, message string) (string, error) {
	args := []string{"stash", "push"}
	if message != "" {
		args = append(args, "-m", message)
	}
	return gitOutput(ctx, workdir, args...)
}

// StashPop restores the most recent stash.
// @skill:method stash_pop "Restore the most recently stashed changes."
// @skill:param  workdir required "Absolute path to the git repository"
// @skill:result "Restored changes summary"
func StashPop(ctx *sdk.Context, workdir string) (string, error) {
	return gitOutput(ctx, workdir, "stash", "pop")
}

// StashList shows all stash entries.
// @skill:method stash_list "List all stash entries."
// @skill:param  workdir required "Absolute path to the git repository"
// @skill:result "Stash entries"
func StashList(ctx *sdk.Context, workdir string) (string, error) {
	return gitOutput(ctx, workdir, "stash", "list")
}

// Restore discards changes in a file (resets to HEAD).
// @skill:method restore "Discard uncommitted changes in a file."
// @skill:param  workdir required "Absolute path to the git repository"
// @skill:param  path    required "File path to restore"
// @skill:result "Confirmation"
func Restore(ctx *sdk.Context, workdir, path string) (string, error) {
	_, err := gitOutput(ctx, workdir, "checkout", "--", path)
	if err != nil {
		return "", err
	}
	return fmt.Sprintf("restored %s", path), nil
}
