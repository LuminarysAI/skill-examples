/**
 * @skill:id      ai.luminarys.as.go-toolchain
 * @skill:name    "Go Toolchain (AS)"
 * @skill:version 1.0.0
 * @skill:desc    "Build, test, format, and run Go projects. Manage background processes."
 * @skill:sdk     "@luminarys/sdk-as"
 * @skill:require shell go **
 * @skill:require shell gofmt **
 * @skill:require shell ps **
 * @skill:require shell kill **
 * @skill:require shell ./**
 */

import { Context, shellExec, ShellResult } from "@luminarys/sdk-as";

function goCmd(workdir: string, args: string, timeoutMs: i32 = 30000): ShellResult {
  return shellExec("go " + args, workdir, timeoutMs);
}

// @skill:method mod_init "Initialize a new Go module (go mod init)."
// @skill:param  workdir required "Project directory (absolute path)"
// @skill:param  module  required "Module path (e.g. github.com/user/project)"
// @skill:result "Status message"
export function modInit(_ctx: Context, workdir: string, module: string): string {
  const r = goCmd(workdir, "mod init " + module);
  if (r.exit_code != 0) throw new Error("go mod init failed (exit " + r.exit_code.toString() + "):\n" + r.output);
  return "module initialized: " + module;
}

// @skill:method mod_tidy "Run go mod tidy to sync dependencies."
// @skill:param  workdir required "Project directory"
// @skill:result "Status message"
export function modTidy(_ctx: Context, workdir: string): string {
  const r = goCmd(workdir, "mod tidy", 600000);
  if (r.exit_code != 0) throw new Error("go mod tidy failed (exit " + r.exit_code.toString() + "):\n" + r.output);
  return "go mod tidy: ok";
}

// @skill:method get "Install a Go dependency (go get)."
// @skill:param  workdir required "Project directory"
// @skill:param  pkg     required "Package to install (e.g. github.com/gin-gonic/gin@latest)"
// @skill:result "Status message"
export function get(_ctx: Context, workdir: string, pkg: string): string {
  const r = goCmd(workdir, "get " + pkg, 600000);
  if (r.exit_code != 0) throw new Error("go get failed (exit " + r.exit_code.toString() + "):\n" + r.output);
  return "installed: " + pkg;
}

// @skill:method build "Build Go project (go build)."
// @skill:param  workdir required "Project directory"
// @skill:param  output  optional "Output binary name"
// @skill:param  tags    optional "Build tags (comma-separated)"
// @skill:result "Build status"
export function build(_ctx: Context, workdir: string, output: string, tags: string): string {
  let args = "build";
  if (tags.length > 0) args += " -tags " + tags;
  if (output.length > 0) args += " -o " + output;
  args += " .";
  const r = goCmd(workdir, args, 600000);
  if (r.exit_code != 0) throw new Error("build failed (exit " + r.exit_code.toString() + "):\n" + r.output);
  return output.length > 0 ? "build ok: " + output : "build ok";
}

// @skill:method test "Run Go tests (go test)."
// @skill:param  workdir required "Project directory"
// @skill:param  pkg     optional "Package pattern (default: ./...)"
// @skill:param  run     optional "Run only matching tests (regex)"
// @skill:param  verbose optional "Show verbose output (true/false)"
// @skill:param  cover   optional "Enable coverage (true/false)"
// @skill:result "Test output"
export function test(_ctx: Context, workdir: string, pkg: string, run: string, verbose: string, cover: string): string {
  let args = "test";
  if (verbose == "true") args += " -v";
  if (cover == "true") args += " -cover";
  if (run.length > 0) args += " -run " + run;
  args += " " + (pkg.length > 0 ? pkg : "./...");
  const r = goCmd(workdir, args, 600000);
  if (r.exit_code != 0) return "TESTS FAILED (exit " + r.exit_code.toString() + "):\n" + r.output;
  return r.output;
}

// @skill:method fmt "Format Go source files (gofmt)."
// @skill:param  workdir required "Project directory"
// @skill:result "Format status"
export function fmt(_ctx: Context, workdir: string): string {
  const r = shellExec("gofmt -w .", workdir);
  if (r.exit_code != 0) throw new Error("gofmt failed (exit " + r.exit_code.toString() + "):\n" + r.output);
  return r.output.length > 0 ? "gofmt:\n" + r.output : "gofmt: all files formatted";
}

// @skill:method vet "Run go vet to check for suspicious code."
// @skill:param  workdir required "Project directory"
// @skill:result "Vet status"
export function vet(_ctx: Context, workdir: string): string {
  const r = goCmd(workdir, "vet ./...", 600000);
  if (r.exit_code != 0) throw new Error("go vet found issues (exit " + r.exit_code.toString() + "):\n" + r.output);
  return "go vet: ok";
}

// @skill:method run "Run a command in the background as a daemon. Returns PID."
// @skill:param  workdir  required "Working directory"
// @skill:param  command  required "Command to run (e.g. ./myapp -port 8080)"
// @skill:param  log_file optional "Log file path (default: auto-generated)"
// @skill:result "PID and log file path"
export function run(_ctx: Context, workdir: string, command: string, logFile: string): string {
  const r = shellExec(command, workdir, 0, 0, "", true, logFile);
  return "pid: " + r.pid.toString() + "\nlog: " + r.log_file;
}

// @skill:method ps "List running processes. Use grep to filter."
// @skill:param  grep optional "Filter output by regex"
// @skill:result "Process list"
export function ps(_ctx: Context, grep: string): string {
  const r = shellExec("ps aux", "", 0, 0, grep);
  return r.output;
}

// @skill:method kill "Kill a process by PID."
// @skill:param  pid    required "Process ID to kill"
// @skill:param  signal optional "Signal number (default: 15/TERM, use 9 for KILL)"
// @skill:result "Kill status"
export function kill(_ctx: Context, pid: string, signal: string): string {
  let cmd = "kill";
  if (signal.length > 0) cmd += " -" + signal;
  cmd += " " + pid;
  const r = shellExec(cmd);
  if (r.exit_code != 0) throw new Error("kill failed (exit " + r.exit_code.toString() + "): " + r.output);
  return "signal sent to pid " + pid;
}
