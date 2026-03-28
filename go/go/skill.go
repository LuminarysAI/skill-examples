// Package main implements go-skill — Go toolchain operations for AI agents.
//
// @skill:id      ai.luminarys.go.go-toolchain
// @skill:name    "Go Toolchain"
// @skill:version 1.0.0
// @skill:desc    "Build, test, format, and run Go projects. Manage background processes."
//
// @skill:require shell go *
// @skill:require shell gofmt *
// @skill:require shell ps *
// @skill:require shell kill *
//
//go:generate lmsk -lang go -verbose .
package main

import (
	"fmt"
	"strconv"
	"strings"

	sdk "github.com/LuminarysAI/sdk-go"
)

// ── Go toolchain commands ─────────────────────────────────────────────────────

// @skill:method mod_init "Initialize a new Go module (go mod init)."
// @skill:param  workdir required "Project directory (absolute path)"
// @skill:param  module  required "Module path (e.g. github.com/user/project)"
// @skill:result "Status message"
func ModInit(ctx *sdk.Context, workdir, module string) (string, error) {
	res, err := sdk.ShellExec(sdk.ShellExecRequest{
		Command: "go mod init " + module,
		Workdir: workdir,
	})
	if err != nil {
		return "", err
	}
	if res.ExitCode != 0 {
		return "", fmt.Errorf("go mod init failed (exit %d):\n%s", res.ExitCode, res.Output)
	}
	return "module initialized: " + module, nil
}

// @skill:method mod_tidy "Run go mod tidy to sync dependencies."
// @skill:param  workdir required "Project directory"
// @skill:result "Status message"
func ModTidy(ctx *sdk.Context, workdir string) (string, error) {
	res, err := sdk.ShellExec(sdk.ShellExecRequest{
		Command:   "go mod tidy",
		Workdir:   workdir,
		TimeoutMs: 120000,
	})
	if err != nil {
		return "", err
	}
	if res.ExitCode != 0 {
		return "", fmt.Errorf("go mod tidy failed (exit %d):\n%s", res.ExitCode, res.Output)
	}
	return "go mod tidy: ok", nil
}

// @skill:method get "Install a Go dependency (go get)."
// @skill:param  workdir required "Project directory"
// @skill:param  pkg     required "Package to install (e.g. github.com/gin-gonic/gin@latest)"
// @skill:result "Status message"
func Get(ctx *sdk.Context, workdir, pkg string) (string, error) {
	res, err := sdk.ShellExec(sdk.ShellExecRequest{
		Command:   "go get " + pkg,
		Workdir:   workdir,
		TimeoutMs: 120000,
	})
	if err != nil {
		return "", err
	}
	if res.ExitCode != 0 {
		return "", fmt.Errorf("go get failed (exit %d):\n%s", res.ExitCode, res.Output)
	}
	return "installed: " + pkg, nil
}

// @skill:method build "Build Go project (go build)."
// @skill:param  workdir required "Project directory"
// @skill:param  output  optional "Output binary name (default: project dir name)"
// @skill:param  tags    optional "Build tags (comma-separated)"
// @skill:result "Build status"
func Build(ctx *sdk.Context, workdir, output, tags string) (string, error) {
	args := []string{"go", "build"}
	if tags != "" {
		args = append(args, "-tags", tags)
	}
	if output != "" {
		args = append(args, "-o", output)
	}
	args = append(args, ".")

	res, err := sdk.ShellExec(sdk.ShellExecRequest{
		Command:   strings.Join(args, " "),
		Workdir:   workdir,
		TimeoutMs: 300000,
	})
	if err != nil {
		return "", err
	}
	if res.ExitCode != 0 {
		return "", fmt.Errorf("build failed (exit %d):\n%s", res.ExitCode, res.Output)
	}
	if output != "" {
		return "build ok: " + output, nil
	}
	return "build ok", nil
}

// @skill:method test "Run Go tests (go test) and return results."
// @skill:param  workdir required "Project directory"
// @skill:param  pkg     optional "Package pattern (default: ./...)"
// @skill:param  run     optional "Run only matching tests (regex)"
// @skill:param  verbose optional "Show verbose output (true/false)"
// @skill:param  cover   optional "Enable coverage (true/false)"
// @skill:result "Test output"
func Test(ctx *sdk.Context, workdir, pkg, run, verbose, cover string) (string, error) {
	args := []string{"go", "test"}
	if verbose == "true" {
		args = append(args, "-v")
	}
	if cover == "true" {
		args = append(args, "-cover")
	}
	if run != "" {
		args = append(args, "-run", run)
	}
	if pkg == "" {
		pkg = "./..."
	}
	args = append(args, pkg)

	res, err := sdk.ShellExec(sdk.ShellExecRequest{
		Command:   strings.Join(args, " "),
		Workdir:   workdir,
		TimeoutMs: 300000,
	})
	if err != nil {
		return "", err
	}
	if res.ExitCode != 0 {
		return fmt.Sprintf("TESTS FAILED (exit %d):\n%s", res.ExitCode, res.Output), nil
	}
	return res.Output, nil
}

// @skill:method fmt "Format Go source files (gofmt)."
// @skill:param  workdir required "Project directory"
// @skill:result "Format status"
func Fmt(ctx *sdk.Context, workdir string) (string, error) {
	res, err := sdk.ShellExec(sdk.ShellExecRequest{
		Command: "gofmt -w .",
		Workdir: workdir,
	})
	if err != nil {
		return "", err
	}
	if res.ExitCode != 0 {
		return "", fmt.Errorf("gofmt failed (exit %d):\n%s", res.ExitCode, res.Output)
	}
	if res.Output == "" {
		return "gofmt: all files formatted", nil
	}
	return "gofmt:\n" + res.Output, nil
}

// @skill:method vet "Run go vet to check for suspicious code."
// @skill:param  workdir required "Project directory"
// @skill:result "Vet status"
func Vet(ctx *sdk.Context, workdir string) (string, error) {
	res, err := sdk.ShellExec(sdk.ShellExecRequest{
		Command:   "go vet ./...",
		Workdir:   workdir,
		TimeoutMs: 120000,
	})
	if err != nil {
		return "", err
	}
	if res.ExitCode != 0 {
		return "", fmt.Errorf("go vet found issues (exit %d):\n%s", res.ExitCode, res.Output)
	}
	return "go vet: ok", nil
}

// ── Process management ────────────────────────────────────────────────────────

// @skill:method run "Run a command in the background as a daemon. Returns PID and log file path."
// @skill:param  workdir  required "Working directory"
// @skill:param  command  required "Command to run (e.g. ./myapp -port 8080)"
// @skill:param  log_file optional "Log file path (default: auto-generated in /tmp)"
// @skill:result "PID and log file path"
func Run(ctx *sdk.Context, workdir, command, logFile string) (string, error) {
	res, err := sdk.ShellExec(sdk.ShellExecRequest{
		Command:  command,
		Workdir:  workdir,
		AsDaemon: true,
		LogFile:  logFile,
	})
	if err != nil {
		return "", err
	}
	return fmt.Sprintf("pid: %d\nlog: %s", res.Pid, res.LogFile), nil
}

// @skill:method ps "List running processes. Use grep to filter output."
// @skill:param  grep optional "Filter output by regex (e.g. process name)"
// @skill:result "Process list"
func Ps(ctx *sdk.Context, grep string) (string, error) {
	res, err := sdk.ShellExec(sdk.ShellExecRequest{
		Command: "ps aux",
		Grep:    grep,
	})
	if err != nil {
		return "", err
	}
	return res.Output, nil
}

// @skill:method kill "Kill a process by PID."
// @skill:param  pid    required "Process ID to kill"
// @skill:param  signal optional "Signal number (default: 15/TERM, use 9 for KILL)"
// @skill:result "Kill status"
func Kill(ctx *sdk.Context, pid, signal string) (string, error) {
	pidNum, err := strconv.Atoi(pid)
	if err != nil {
		return "", fmt.Errorf("invalid pid: %s", pid)
	}

	args := "kill"
	if signal != "" {
		args += " -" + signal
	}
	args += " " + strconv.Itoa(pidNum)

	res, execErr := sdk.ShellExec(sdk.ShellExecRequest{
		Command: args,
	})
	if execErr != nil {
		return "", execErr
	}
	if res.ExitCode != 0 {
		return "", fmt.Errorf("kill failed (exit %d): %s", res.ExitCode, res.Output)
	}
	return fmt.Sprintf("signal sent to pid %d", pidNum), nil
}
