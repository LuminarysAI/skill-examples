/**
 * @skill:id      ai.luminarys.as.python-toolchain
 * @skill:name    "Python Toolchain (AS)"
 * @skill:version 1.0.0
 * @skill:desc    "Create virtual environments, install packages, run Python scripts, manage background processes."
 * @skill:sdk     "@luminarys/sdk-as"
 * @skill:require shell python -m venv .venv
 * @skill:require shell .venv/bin/pip **
 * @skill:require shell .venv/bin/python **
 * @skill:require shell .venv/bin/pytest **
 * @skill:require shell ps **
 * @skill:require shell kill **
 * @skill:require shell ./**
 */

import { Context, shellExec, ShellResult } from "@luminarys/sdk-as";

// @skill:method venv "Create a Python virtual environment."
// @skill:param  workdir required "Project directory (absolute path)"
// @skill:result "Status message"
export function venv(_ctx: Context, workdir: string): string {
  const r = shellExec("python -m venv .venv", workdir, 120000);
  if (r.exit_code != 0) throw new Error("venv creation failed (exit " + r.exit_code.toString() + "):\n" + r.output);
  return "venv created: " + workdir + "/.venv";
}

// @skill:method pip_install "Install Python packages via pip."
// @skill:param  workdir  required "Project directory"
// @skill:param  packages required "Package(s) to install (space-separated)"
// @skill:result "Status message"
export function pipInstall(_ctx: Context, workdir: string, packages: string): string {
  const r = shellExec(".venv/bin/pip install " + packages, workdir, 300000);
  if (r.exit_code != 0) throw new Error("pip install failed (exit " + r.exit_code.toString() + "):\n" + r.output);
  return "installed: " + packages;
}

// @skill:method pip_freeze "List installed packages (pip freeze)."
// @skill:param  workdir required "Project directory"
// @skill:result "Installed packages"
export function pipFreeze(_ctx: Context, workdir: string): string {
  const r = shellExec(".venv/bin/pip freeze", workdir);
  return r.output.length > 0 ? r.output : "(no packages installed)";
}

// @skill:method run_script "Run a Python script."
// @skill:param  workdir required "Project directory"
// @skill:param  script  required "Script filename or -c 'code'"
// @skill:param  args    optional "Command line arguments"
// @skill:result "Script output"
export function runScript(_ctx: Context, workdir: string, script: string, args: string): string {
  let cmd = ".venv/bin/python " + script;
  if (args.length > 0) cmd += " " + args;
  const r = shellExec(cmd, workdir, 120000);
  if (r.exit_code != 0) return "FAILED (exit " + r.exit_code.toString() + "):\n" + r.output;
  return r.output;
}

// @skill:method test "Run pytest."
// @skill:param  workdir required "Project directory"
// @skill:param  path    optional "Test file or directory (default: .)"
// @skill:param  verbose optional "Verbose output (true/false)"
// @skill:result "Test output"
export function test(_ctx: Context, workdir: string, path: string, verbose: string): string {
  let cmd = ".venv/bin/python -m pytest";
  if (verbose == "true") cmd += " -v";
  cmd += " " + (path.length > 0 ? path : ".");
  const r = shellExec(cmd, workdir, 300000);
  if (r.exit_code != 0) return "TESTS FAILED (exit " + r.exit_code.toString() + "):\n" + r.output;
  return r.output;
}

// @skill:method run "Start a Python application as a background daemon. Returns PID."
// @skill:param  workdir  required "Working directory"
// @skill:param  command  required "Command to run (e.g. .venv/bin/python app.py)"
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
