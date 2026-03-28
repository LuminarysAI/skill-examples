/**
 * echo-skill — ABI test skill.
 *
 * @skill:id      ai.luminarys.as.echo
 * @skill:name    "Echo Skill"
 * @skill:version 1.0.0
 * @skill:desc    "Echo, reverse, word count + ABI probes (sys_info, time_now, disk_usage, env_get, tcp_request)."
 * @skill:sdk     "@luminarys/sdk-as"
 */

import { Context,  sysInfo, timeNow, diskUsage, getEnv, tcpRequest, shellExec, log } from "@luminarys/sdk-as";

// @skill:method echo "Return the input string unchanged."
// @skill:param  message required "Any string"
// @skill:result "The same string"
export function echo(_ctx: Context, message: string): string {
  return message;
}

// @skill:method ping "Health-check. Always returns pong."
// @skill:result "Always pong"
export function ping(_ctx: Context): string {
  return "pong";
}

// @skill:method reverse "Reverse the characters of a string."
// @skill:param  message required "String to reverse"
// @skill:result "Reversed string"
export function reverse(_ctx: Context, message: string): string {
  let result = "";
  for (let i = message.length - 1; i >= 0; i--) {
    result += message.charAt(i);
  }
  return result;
}

// @skill:method word_count "Count the number of words in text."
// @skill:param  text required "Input text"
// @skill:result "Word count as a number"
export function wordCount(_ctx: Context, text: string): i32 {
  if (text.trim().length == 0) return 0;
  let count = 1;
  let inSpace = false;
  for (let i = 0; i < text.length; i++) {
    const c = text.charCodeAt(i);
    if (c == 32 || c == 9 || c == 10 || c == 13) {
      inSpace = true;
    } else if (inSpace) {
      count++;
      inSpace = false;
    }
  }
  return count;
}

// ── ABI probe methods ────────────────────────────────────────────────────────

// @skill:method sys_info "Return host OS/arch/hostname/cpu info."
// @skill:result "System info string"
export function sysInfoProbe(_ctx: Context): string {
  const info = sysInfo();
  return info.os + "/" + info.arch + " hostname=" + info.hostname + " cpus=" + info.num_cpu.toString();
}

// @skill:method time_now "Return current host time."
// @skill:result "Current time"
export function timeNowProbe(_ctx: Context): string {
  const t = timeNow();
  return t.rfc3339 + " tz=" + t.timezone + " unix=" + t.unix.toString();
}

// @skill:method disk_usage "Return disk usage for /data."
// @skill:result "Disk usage info"
export function diskUsageProbe(_ctx: Context): string {
  const d = diskUsage("/data");
  return "total=" + d.total_bytes.toString() + " free=" + d.free_bytes.toString() + " used=" + d.used_pct.toString() + "%";
}

// @skill:method env_get "Read an environment variable from manifest."
// @skill:param  key required "Variable name"
// @skill:result "Variable value or empty"
export function envGetProbe(_ctx: Context, key: string): string {
  const val = getEnv(key);
  return val.length > 0 ? val : "(not set)";
}

// @skill:method cwd_test "Test shell_exec workdir — runs pwd in given directory."
// @skill:param  workdir required "Directory to run pwd in"
// @skill:result "Current working directory from pwd"
export function cwdTest(_ctx: Context, workdir: string): string {
  const r = shellExec("pwd", workdir);
  if (r.exit_code != 0) return "ERROR: pwd failed: " + r.output;
  return "cwd=" + r.output.trim();
}

// @skill:method tcp_ping "Test tcp_request ABI by connecting to a TCP endpoint."
// @skill:param  addr required "Host:port to connect to (e.g. nats:4222)"
// @skill:result "First line of TCP response"
export function tcpPing(_ctx: Context, addr: string): string {
  // Send empty data — many protocols (NATS, HTTP, SMTP) send a greeting on connect.
  const emptyData = new Uint8Array(0);
  const resp = tcpRequest(addr, emptyData, false, false, 5000, 4096);
  return "received " + resp.length.toString() + " bytes: " + String.UTF8.decode(resp.buffer).substring(0, 200);
}

// @skill:method log_test "Test log_write ABI by emitting a log message."
// @skill:param  message required "Message to log"
// @skill:result "Confirmation"
export function logTest(_ctx: Context, message: string): string {
  log("info", message);
  return "logged: " + message;
}
