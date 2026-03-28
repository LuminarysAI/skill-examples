/**
 * @skill:id      ai.luminarys.as.fs
 * @skill:name    "File System Skill (AS)"
 * @skill:version 1.0.0
 * @skill:desc    "Sandboxed file system operations: read, write, delete, list, copy, mkdir, grep, glob."
 * @skill:sdk     "@luminarys/sdk-as"
 */

import { Context, 
  fsRead, fsWrite, fsCreate, fsDelete, fsMkdir, fsAllowedDirs, fsCopy,
  fsLs, fsReadLines, fsGrep, fsGlob, fsChmod, DirEntry
} from "@luminarys/sdk-as";

// @skill:method read "Read a file and return its contents as text."
// @skill:param  path required "Absolute file path"
// @skill:result "File contents"
export function read(_ctx: Context, path: string): string {
  const data = fsRead(path);
  return String.UTF8.decode(data.buffer);
}

// @skill:method write "Write text content to a file, replacing it entirely."
// @skill:param  path    required "Absolute file path"
// @skill:param  content required "Text content to write"
// @skill:result "Confirmation"
export function write(_ctx: Context, path: string, content: string): string {
  const bytes = Uint8Array.wrap(String.UTF8.encode(content));
  fsWrite(path, bytes);
  return "written: " + path;
}

// @skill:method create "Create a new file. Fails if the file already exists."
// @skill:param  path    required "Absolute file path"
// @skill:param  content required "Text content"
// @skill:result "Confirmation"
export function create(_ctx: Context, path: string, content: string): string {
  const bytes = Uint8Array.wrap(String.UTF8.encode(content));
  fsCreate(path, bytes);
  return "created: " + path;
}

// @skill:method delete "Delete a file or directory."
// @skill:param  path required "Absolute path"
// @skill:result "Confirmation"
export function deleteFn(_ctx: Context, path: string): string {
  fsDelete(path);
  return "deleted: " + path;
}

// @skill:method mkdir "Create a directory (with parents)."
// @skill:param  path required "Absolute directory path. Supports brace expansion: /data/{logs,cache} creates two dirs"
// @skill:result "Confirmation"
export function mkdirFn(_ctx: Context, path: string): string {
  fsMkdir(path);
  return "created: " + path;
}

// @skill:method ls "List directory contents."
// @skill:param  path required "Absolute directory path"
// @skill:result "File listing"
export function ls(_ctx: Context, path: string): string {
  const entries = fsLs(path, false);
  const lines: string[] = [];
  for (let i = 0; i < entries.length; i++) {
    const e = entries[i];
    const kind = e.is_dir ? "dir " : "file";
    lines.push(kind + "  " + e.size.toString().padStart(8, " ") + "  " + e.name);
  }
  if (lines.length == 0) return "(empty directory)";
  return lines.join("\n");
}

// @skill:method copy "Copy a file."
// @skill:param  source required "Source file path"
// @skill:param  dest   required "Destination file path"
// @skill:result "Confirmation"
export function copy(_ctx: Context, source: string, dest: string): string {
  fsCopy(source, dest);
  return "copied: " + source + " → " + dest;
}

// @skill:method read_lines "Read specific lines from a text file."
// @skill:param  path   required "Absolute file path"
// @skill:param  offset optional "Start line (0-based, default 0)"
// @skill:param  limit  optional "Max lines to return (0 = all)"
// @skill:result "Lines with metadata"
export function readLines(_ctx: Context, path: string, offset: i64, limit: i64): string {
  const r = fsReadLines(path, offset as i32, limit as i32);

  let result = "";
  for (let i = 0; i < r.lines.length; i++) {
    const lineNum = r.offset + i + 1;
    result += lineNum.toString().padStart(4, " ") + "│ " + r.lines[i] + "\n";
  }
  result += "--- " + r.lines.length.toString() + "/" + r.total_lines.toString() + " lines";
  if (r.is_truncated) result += " (truncated)";
  return result;
}

// @skill:method grep "Search file contents by regex pattern."
// @skill:param  path    required "File or directory path"
// @skill:param  pattern required "Regex pattern"
// @skill:result "Matching lines"
export function grep(_ctx: Context, path: string, pattern: string): string {
  const matches = fsGrep(pattern, path, false, false, true);
  const lines: string[] = [];
  for (let i = 0; i < matches.length; i++) {
    const fm = matches[i];
    for (let j = 0; j < fm.matches.length; j++) {
      const lm = fm.matches[j];
      lines.push(fm.path + ":" + lm.line_num.toString() + ": " + lm.line);
    }
  }
  if (lines.length == 0) return "(no matches)";
  return lines.join("\n");
}

// @skill:method glob "Find files matching a glob pattern."
// @skill:param  path    required "Base directory"
// @skill:param  pattern required "Glob pattern (e.g. *.go)"
// @skill:result "List of matching file paths"
export function glob(_ctx: Context, path: string, pattern: string): string {
  const entries = fsGlob([pattern], path);
  const lines: string[] = [];
  for (let i = 0; i < entries.length; i++) {
    lines.push(entries[i].path);
  }
  if (lines.length == 0) return "(no matches)";
  return lines.join("\n");
}

// @skill:method chmod "Change file permissions."
// @skill:param  path      required "File or directory path"
// @skill:param  mode      required "Permission bits (e.g. 755)"
// @skill:param  recursive optional "Apply recursively (true/false)"
// @skill:result "Confirmation"
export function chmod(_ctx: Context, path: string, mode: i64, recursive: bool): string {
  // Convert decimal mode (e.g. 755) to octal.
  const octalMode = parseOctalMode(mode as i32);
  fsChmod(path, octalMode as u32, recursive);
  return "chmod: " + path + " → " + mode.toString();
}

// @skill:method allowed_dirs "List directories this skill can access."
// @skill:result "Directory list with permissions"
export function allowedDirs(_ctx: Context): string {
  const dirs = fsAllowedDirs();
  let result = "";
  for (let i = 0; i < dirs.length; i++) {
    result += dirs[i].path + " (" + dirs[i].mode + ")\n";
  }
  return result;
}

/** Convert "755" as decimal i32 to octal value 0o755 = 493. */
function parseOctalMode(mode: i32): i32 {
  let result: i32 = 0;
  let multiplier: i32 = 1;
  let m = mode;
  while (m > 0) {
    result += (m % 10) * multiplier;
    multiplier *= 8;
    m = m / 10;
  }
  return result;
}
