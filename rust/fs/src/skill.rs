//! fs-skill — sandboxed file system operations.
//!
//! All paths passed to skill methods must be absolute. The host validates
//! each path against the directories declared in manifest.yaml.
//!
//! @skill:id      ai.luminarys.rust.fs
//! @skill:name    "File System Skill"
//! @skill:version 1.0.0
//! @skill:desc    "File system operations. All paths must be absolute (e.g. /home/user/project/src/main.go). The host validates access against declared directories."
//!
//! Build:
//!   lmsk generate -lang rust -out src .
//!   cargo build --target wasm32-wasip1 --release

use luminarys_sdk::prelude::*;

// ── read / read_lines ─────────────────────────────────────────────────────────

/// Read the full contents of a text file.
///
/// @skill:method read "Read the full contents of a text file. Rejects binary files."
/// @skill:param  path required "Absolute path to the file (e.g. /home/user/project/src/main.go)" example:src/main.go
/// @skill:result "File contents as UTF-8 text, or an error if the file is binary"
pub fn read_file(_ctx: &mut Context, path: String) -> Result<String, SkillError> {
    let tc = fs_read_lines(FsReadLinesRequest { path, offset: 0, limit: 0 })?;
    Ok(tc.lines.join("\n"))
}

/// Read a range of lines from a file (1-based, inclusive).
///
/// Set end_line to 0 to read until the end of the file.
/// Each line is prefixed with its 1-based line number for easy reference.
///
/// @skill:method read_lines "Read a range of lines. Efficient for large files — only the requested slice is transferred."
/// @skill:param  path       required "Absolute path to the file"
/// @skill:param  start_line required "First line to read (1-based)" example:10
/// @skill:param  end_line   required "Last line to read (inclusive). 0 = read to end of file." example:30
/// @skill:result "Header with line range and total, then each line prefixed with its line number"
pub fn read_lines(
    _ctx: &mut Context,
    path: String,
    start_line: i64,
    end_line: i64,
) -> Result<String, SkillError> {
    let start = start_line.max(1);
    let offset = start - 1;
    let limit = if end_line > 0 {
        if end_line < start {
            return Err(SkillError(format!(
                "start_line ({start_line}) is after end_line ({end_line})"
            )));
        }
        end_line - start + 1
    } else {
        0
    };

    let tc = fs_read_lines(FsReadLinesRequest { path, offset, limit })?;
    let actual_start = tc.offset + 1;
    let actual_end = tc.offset + tc.lines.len() as i64;

    let header = if tc.is_truncated {
        format!(
            "Showing lines {actual_start}-{actual_end} of {} total (truncated)\n\n",
            tc.total_lines
        )
    } else {
        format!(
            "Showing lines {actual_start}-{actual_end} of {} total\n\n",
            tc.total_lines
        )
    };
    let body: String = tc
        .lines
        .iter()
        .enumerate()
        .map(|(i, l)| format!("{:5}: {}\n", actual_start + i as i64, l))
        .collect();
    Ok(format!("{header}{}", body.trim_end_matches('\n')))
}

// ── write ─────────────────────────────────────────────────────────────────────

/// Write text content to a file, replacing its entire contents.
/// Creates the file (and any missing parent directories) if it does not exist.
/// Returns a unified diff so the LLM can verify the exact changes made.
///
/// @skill:method write "Write text content to a file, replacing it entirely. Returns a diff of the changes."
/// @skill:param  path    required "Absolute path to the file" example:src/config.yaml
/// @skill:param  content required "Complete new file content (UTF-8 text)"
/// @skill:result "Success message with unified diff showing what changed"
pub fn write_file(_ctx: &mut Context, path: String, content: String) -> Result<String, SkillError> {
    let (original, is_new) = match fs_read(&path) {
        Ok(b) => (String::from_utf8_lossy(&b).into_owned(), false),
        Err(_) => (String::new(), true),
    };
    fs_write(&path, content.as_bytes().to_vec())?;
    Ok(format_write_result(&path, &original, &content, is_new))
}

/// Append a single line of text (with trailing newline) to a file.
/// Creates the file if it does not exist.
///
/// @skill:method append_line "Append a line of text to a file. Creates the file if needed."
/// @skill:param  path required "Absolute path to the file"
/// @skill:param  line required "Text to append (a newline is added automatically)"
/// @skill:result "Success message"
pub fn append_line(_ctx: &mut Context, path: String, line: String) -> Result<String, SkillError> {
    let mut data = fs_read(&path).unwrap_or_default();
    data.extend_from_slice(line.trim_end_matches('\n').as_bytes());
    data.push(b'\n');
    fs_write(&path, data)?;
    Ok(format!("Appended 1 line to {path}"))
}

// ── edit ──────────────────────────────────────────────────────────────────────

/// Replace an exact string within a file with new text.
///
/// This is the preferred way to make targeted changes: supply the exact
/// surrounding context you want to replace and the replacement text.
/// The operation fails if old_string is not found or matches multiple
/// locations (use replace_all to replace every occurrence).
///
/// To create a new file, set old_string to "" (empty).
/// To delete text, set new_string to "" (empty).
///
/// @skill:method edit "Replace an exact string in a file with new text. Fails if old_string is not found or is ambiguous."
/// @skill:param  path        required "Absolute path to the file" example:src/main.rs
/// @skill:param  old_string  required "The exact text to replace, including surrounding context. Must match exactly once unless replace_all is true. Use empty string to create a new file."
/// @skill:param  new_string  required "The replacement text. Use empty string to delete old_string."
/// @skill:param  replace_all optional "Replace every occurrence of old_string instead of requiring a unique match." default:false
/// @skill:result "Unified diff of the change, or an error describing why the edit failed"
pub fn edit_file(
    _ctx: &mut Context,
    path: String,
    old_string: String,
    new_string: String,
    replace_all: bool,
) -> Result<String, SkillError> {
    let (current, is_new) = match fs_read(&path) {
        Ok(b) => (
            String::from_utf8_lossy(&b).into_owned().replace("\r\n", "\n"),
            false,
        ),
        Err(_) => (String::new(), true),
    };

    if old_string.is_empty() && !is_new {
        return Err(SkillError(format!(
            "file already exists: {path} — use an empty old_string only to create a new file"
        )));
    }
    if !is_new && !old_string.is_empty() {
        let n = current.matches(old_string.as_str()).count();
        match n {
            0 => return Err(SkillError(format!(
                "old_string not found in {path} — check whitespace, indentation and surrounding context"
            ))),
            c if c > 1 && !replace_all => return Err(SkillError(format!(
                "old_string matches {c} locations in {path} — add more context or set replace_all:true"
            ))),
            _ => {}
        }
        if old_string == new_string {
            return Err(SkillError(
                "old_string and new_string are identical — no changes to apply".into(),
            ));
        }
    }

    let new_content = if replace_all {
        current.replace(&old_string, &new_string)
    } else {
        match current.find(&old_string) {
            Some(pos) => {
                let mut s = current.clone();
                s.replace_range(pos..pos + old_string.len(), &new_string);
                s
            }
            None => current.clone(),
        }
    };

    if !is_new && new_content == current {
        return Err(SkillError(
            "resulting content is identical to current content — no changes applied".into(),
        ));
    }
    fs_write(&path, new_content.as_bytes().to_vec())
        .map_err(|e| SkillError(format!("write failed: {e}")))?;
    Ok(format_edit_result(&path, &current, &new_content, is_new))
}

// ── delete ────────────────────────────────────────────────────────────────────

/// Permanently delete a file.
///
/// @skill:method delete "Permanently delete a file."
/// @skill:param  path required "Absolute path to the file"
/// @skill:result "Deleted file path"
pub fn delete_file(_ctx: &mut Context, path: String) -> Result<String, SkillError> {
    fs_delete(&path)?;
    Ok(path)
}

// ── directory operations ──────────────────────────────────────────────────────

/// Create a directory and all missing parents.
///
/// @skill:method mkdir "Create a directory (parents created automatically)."
/// @skill:param  path required "Absolute directory path. Supports brace expansion: /data/{logs,cache} creates two dirs" example:/home/user/project/reports/2024
/// @skill:result "Created directory path"
pub fn make_dir(_ctx: &mut Context, path: String) -> Result<String, SkillError> {
    fs_mkdir(&path)?;
    Ok(path)
}

/// List the contents of a directory.
///
/// @skill:method list "List the contents of a directory."
/// @skill:param  path     required "Absolute path to the directory" example:/home/user/project
/// @skill:param  detailed optional "Include modification time and permissions" default:false
/// @skill:result "Directory listing, one entry per line"
pub fn list_dir(_ctx: &mut Context, path: String, detailed: bool) -> Result<String, SkillError> {
    let entries = fs_ls(&path, detailed)?;
    if entries.is_empty() {
        return Ok(format!("(empty directory: {path})"));
    }
    let mut out = format!("{path}:\n");
    for e in &entries {
        if e.is_dir {
            if detailed && e.mod_time != 0 {
                out.push_str(&format!("  {}/\t[dir]\t{}\t{}\n", e.name, e.mod_time, e.mode_str));
            } else {
                out.push_str(&format!("  {}/\t[dir]\n", e.name));
            }
        } else {
            let size = format_size(e.size);
            if detailed && e.mod_time != 0 {
                out.push_str(&format!("  {}\t{}\t{}\t{}\n", e.name, size, e.mod_time, e.mode_str));
            } else {
                out.push_str(&format!("  {}\t{}\n", e.name, size));
            }
        }
    }
    Ok(out.trim_end_matches('\n').to_string())
}

/// Check if a path exists and get its metadata. Never errors — missing paths return exists:false.
///
/// @skill:method stat "Check if a path exists and get its metadata."
/// @skill:param  path required "File or directory path"
/// @skill:result "Metadata: path, exists, type, size, modified, permissions"
pub fn stat_file(_ctx: &mut Context, path: String) -> Result<String, SkillError> {
    let trimmed = path.trim_end_matches('/');
    let (name, dir) = match trimmed.rfind('/') {
        Some(i) => (&trimmed[i + 1..], if i == 0 { "/" } else { &trimmed[..i] }),
        None => (trimmed, "."),
    };
    if let Ok(entries) = fs_ls(dir, true) {
        for e in &entries {
            if e.name == name {
                let typ = if e.is_dir { "directory" } else { "file" };
                return Ok(format!(
                    "path: {path}\nexists: true\ntype: {typ}\nsize: {} bytes\nmodified: {} UTC\npermissions: {}",
                    e.size, e.mod_time, e.mode_str
                ));
            }
        }
    }
    if fs_ls(&path, false).is_ok() {
        return Ok(format!("path: {path}\nexists: true\ntype: directory"));
    }
    Ok(format!("path: {path}\nexists: false"))
}

/// Change file or directory permissions.
///
/// @skill:method chmod "Change file or directory permissions."
/// @skill:param  path      required "File or directory path"
/// @skill:param  mode      required "Permission bits as decimal: 420=0644, 493=0755, 384=0600" example:420
/// @skill:param  recursive optional "Apply recursively to directory contents" default:false
/// @skill:result "Path with new permissions applied"
pub fn change_permissions(
    _ctx: &mut Context,
    path: String,
    mode: i64,
    recursive: bool,
) -> Result<String, SkillError> {
    fs_chmod(&path, mode as u32, recursive)?;
    Ok(format!("{path} (mode {:04o})", mode))
}

// ── move / copy / count_lines ─────────────────────────────────────────────────

/// Rename or move a file within the sandbox (copy + delete).
///
/// @skill:method move "Rename or move a file within the sandbox."
/// @skill:param  src required "Absolute source file path"
/// @skill:param  dst required "Absolute destination file path"
/// @skill:result "Destination path"
pub fn move_file(_ctx: &mut Context, src: String, dst: String) -> Result<String, SkillError> {
    let data = fs_read(&src).map_err(|e| SkillError(format!("move: read {src}: {e}")))?;
    fs_write(&dst, data).map_err(|e| SkillError(format!("move: write {dst}: {e}")))?;
    fs_delete(&src)
        .map_err(|e| SkillError(format!("move: cleanup {src}: {e} (file was copied to {dst})")))?;
    Ok(dst)
}

/// Copy a file to a new path within the sandbox.
///
/// @skill:method copy "Copy a file to a new path within the sandbox."
/// @skill:param  src required "Absolute source file path"
/// @skill:param  dst required "Absolute destination file path"
/// @skill:result "Destination path"
pub fn copy_file(_ctx: &mut Context, src: String, dst: String) -> Result<String, SkillError> {
    let data = fs_read(&src).map_err(|e| SkillError(format!("copy: read {src}: {e}")))?;
    fs_write(&dst, data).map_err(|e| SkillError(format!("copy: write {dst}: {e}")))?;
    Ok(dst)
}

/// Count the number of lines in a file.
///
/// @skill:method count_lines "Count the number of lines in a file."
/// @skill:param  path required "Absolute file path"
/// @skill:result "Line count"
pub fn count_lines(_ctx: &mut Context, path: String) -> Result<i64, SkillError> {
    let tc = fs_read_lines(FsReadLinesRequest { path, offset: 0, limit: 1 })?;
    Ok(tc.total_lines)
}

// ── glob / tree ───────────────────────────────────────────────────────────────

/// Find files matching glob patterns.
///
/// @skill:method find "Find files matching glob patterns. ** = recursive, {a,b} = alternatives."
/// @skill:param  patterns   required "Glob patterns, comma-separated" example:**/*.rs
/// @skill:param  path       optional "Absolute directory path to search. Empty = all allowed directories." default:""
/// @skill:param  only_files optional "Return only files, not directories (default: true)" default:true
/// @skill:result "Matching paths, one per line"
pub fn find_files(
    _ctx: &mut Context,
    patterns: String,
    path: String,
    only_files: bool,
) -> Result<String, SkillError> {
    let pat_list: Vec<String> = patterns
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let entries = fs_glob(GlobOptions {
        patterns: pat_list,
        path,
        only_files,
        ..Default::default()
    })?;
    if entries.is_empty() {
        return Ok(format!("(no matches: {patterns})"));
    }
    Ok(entries
        .iter()
        .map(|e| if e.is_dir { format!("{}/", e.path) } else { e.path.clone() })
        .collect::<Vec<_>>()
        .join("\n"))
}

/// Show an indented directory tree.
///
/// @skill:method tree "Show an indented directory tree."
/// @skill:param  path      required "Absolute path to root directory" example:/home/user/project
/// @skill:param  max_depth optional "Maximum depth (default: 3, 0 = unlimited)" default:3
/// @skill:result "Indented tree"
pub fn tree_view(_ctx: &mut Context, path: String, max_depth: i64) -> Result<String, SkillError> {
    let depth = if max_depth == 0 { 3 } else { max_depth };
    let entries = fs_glob(GlobOptions {
        patterns: vec!["**".into()],
        path: path.clone(),
        max_depth: depth,
        ..Default::default()
    })?;
    if entries.is_empty() {
        return Ok(format!("(empty: {path})"));
    }
    let root = if path.is_empty() { ".".to_string() } else { path.clone() };
    let mut out = format!("{root}\n");
    for e in &entries {
        if e.path == "." || e.path == root { continue; }
        let depth_n = e.path.matches('/').count();
        let indent = "    ".repeat(depth_n);
        let name = e.path.rsplit('/').next().unwrap_or(&e.path);
        if e.is_dir {
            out.push_str(&format!("{indent}├── {name}/\n"));
        } else {
            out.push_str(&format!("{indent}├── {name}\n"));
        }
    }
    Ok(out.trim_end_matches('\n').to_string())
}

// ── search (grep) ─────────────────────────────────────────────────────────────

/// Search for a pattern across all text files.
///
/// @skill:method search "Search for a pattern across all text files."
/// @skill:param  pattern          required "RE2 regex or literal string" example:TODO
/// @skill:param  path             optional "Directory or file to search (default: entire sandbox)" default:""
/// @skill:param  fixed            optional "Treat pattern as literal text (useful for URLs, version strings)" default:false
/// @skill:param  case_insensitive optional "Case-insensitive matching" default:false
/// @skill:result "Formatted results: summary line, then per-file blocks with line numbers and text"
pub fn search_text(
    _ctx: &mut Context,
    pattern: String,
    path: String,
    fixed: bool,
    case_insensitive: bool,
) -> Result<String, SkillError> {
    let matches = fs_grep(GrepOptions {
        pattern: pattern.clone(),
        path,
        fixed,
        case_insensitive,
        with_lines: true,
        type_filter: "text".into(),
        ..Default::default()
    })?;
    Ok(format_grep_results(&matches, &pattern))
}

/// Search in source code files only.
///
/// @skill:method search_code "Search in source code files only (excludes configs, data, docs)."
/// @skill:param  pattern          required "RE2 regex or literal string" example:fn New
/// @skill:param  path             optional "Directory or file to search" default:""
/// @skill:param  fixed            optional "Literal text matching" default:false
/// @skill:param  case_insensitive optional "Case-insensitive matching" default:false
/// @skill:result "Formatted results with file paths, line numbers, and matching text"
pub fn search_code(
    _ctx: &mut Context,
    pattern: String,
    path: String,
    fixed: bool,
    case_insensitive: bool,
) -> Result<String, SkillError> {
    let matches = fs_grep(GrepOptions {
        pattern: pattern.clone(),
        path,
        fixed,
        case_insensitive,
        with_lines: true,
        type_filter: "code".into(),
        ..Default::default()
    })?;
    Ok(format_grep_results(&matches, &pattern))
}

/// Return paths of files containing the pattern. Fast triage — no line detail.
///
/// @skill:method search_files "Return paths of files containing the pattern. Fast triage — no line detail."
/// @skill:param  pattern          required "RE2 regex or literal string" example:@deprecated
/// @skill:param  path             optional "Directory to search" default:""
/// @skill:param  case_insensitive optional "Case-insensitive matching" default:false
/// @skill:result "File paths, one per line"
pub fn search_files(
    _ctx: &mut Context,
    pattern: String,
    path: String,
    case_insensitive: bool,
) -> Result<String, SkillError> {
    let matches = fs_grep(GrepOptions {
        pattern: pattern.clone(),
        path,
        case_insensitive,
        filename_only: true,
        type_filter: "text".into(),
        ..Default::default()
    })?;
    if matches.is_empty() {
        return Ok(format!("(no files contain: {pattern})"));
    }
    Ok(matches.iter().map(|m| m.path.as_str()).collect::<Vec<_>>().join("\n"))
}

/// Search within a specific file.
///
/// @skill:method search_in_file "Search within a specific file by exact path."
/// @skill:param  pattern          required "RE2 regex or literal string"
/// @skill:param  path             required "Exact path to the file"
/// @skill:param  fixed            optional "Literal text matching" default:false
/// @skill:param  case_insensitive optional "Case-insensitive matching" default:false
/// @skill:result "Matching lines with line numbers"
pub fn search_in_file(
    _ctx: &mut Context,
    pattern: String,
    path: String,
    fixed: bool,
    case_insensitive: bool,
) -> Result<String, SkillError> {
    let matches = fs_grep(GrepOptions {
        pattern: pattern.clone(),
        path,
        fixed,
        case_insensitive,
        with_lines: true,
        type_filter: "all".into(),
        ..Default::default()
    })?;
    Ok(format_grep_results(&matches, &pattern))
}

// ── Diff utilities (internal helpers, not exported as methods) ────────────────

fn split_lines(s: &str) -> Vec<&str> {
    if s.is_empty() { return vec![]; }
    s.trim_end_matches('\n').split('\n').collect()
}

const MAX_DIFF_LINES: usize = 2000;

fn lcs_lines<'a>(a: &[&'a str], b: &[&'a str]) -> Vec<[usize; 2]> {
    let (m, n) = (a.len(), b.len());
    if m == 0 || n == 0 { return vec![]; }
    // Guard: O(m*n) DP table — 2000×2000×8 = 32 MB max.
    if m > MAX_DIFF_LINES || n > MAX_DIFF_LINES { return vec![]; }
    let mut dp = vec![vec![0usize; n + 1]; m + 1];
    for i in (0..m).rev() {
        for j in (0..n).rev() {
            dp[i][j] = if a[i] == b[j] {
                dp[i + 1][j + 1] + 1
            } else {
                dp[i + 1][j].max(dp[i][j + 1])
            };
        }
    }
    let mut result = vec![];
    let (mut i, mut j) = (0, 0);
    while i < m && j < n {
        if a[i] == b[j] { result.push([i, j]); i += 1; j += 1; }
        else if dp[i + 1][j] >= dp[i][j + 1] { i += 1; } else { j += 1; }
    }
    result
}

struct DiffLine { kind: char, text: String }
struct DiffHunk { old_start: usize, old_count: usize, new_start: usize, new_count: usize, lines: Vec<DiffLine> }

fn compute_diff(old_text: &str, new_text: &str) -> Vec<DiffHunk> {
    let ol = split_lines(old_text);
    let nl = split_lines(new_text);
    let lcs = lcs_lines(&ol, &nl);

    let mut ops: Vec<(char, &str, &str)> = vec![];
    let (mut oi, mut ni) = (0usize, 0usize);
    for &[li, ri] in &lcs {
        while oi < li { ops.push(('-', ol[oi], "")); oi += 1; }
        while ni < ri { ops.push(('+', "", nl[ni])); ni += 1; }
        ops.push(('=', ol[oi], nl[ni])); oi += 1; ni += 1;
    }
    while oi < ol.len() { ops.push(('-', ol[oi], "")); oi += 1; }
    while ni < nl.len() { ops.push(('+', "", nl[ni])); ni += 1; }

    let mut op_old = vec![1usize; ops.len()];
    let mut op_new = vec![1usize; ops.len()];
    let (mut o, mut n) = (1usize, 1usize);
    for (idx, &(k, _, _)) in ops.iter().enumerate() {
        op_old[idx] = o; op_new[idx] = n;
        if k == '=' || k == '-' { o += 1; }
        if k == '=' || k == '+' { n += 1; }
    }

    const CTX: usize = 3;
    let mut hunks: Vec<DiffHunk> = vec![];
    let mut i = 0;
    while i < ops.len() {
        if ops[i].0 == '=' { i += 1; continue; }
        let s = i;
        while i < ops.len() && ops[i].0 != '=' { i += 1; }
        let e = i;
        let ctx_start = s.saturating_sub(CTX);
        let ctx_end = (e + CTX).min(ops.len());

        let mut hunk = DiffHunk {
            old_start: op_old[ctx_start], old_count: 0,
            new_start: op_new[ctx_start], new_count: 0,
            lines: vec![],
        };
        for &(k, old, new) in &ops[ctx_start..ctx_end] {
            match k {
                '=' => { hunk.lines.push(DiffLine { kind: ' ', text: old.into() }); hunk.old_count += 1; hunk.new_count += 1; }
                '-' => { hunk.lines.push(DiffLine { kind: '-', text: old.into() }); hunk.old_count += 1; }
                '+' => { hunk.lines.push(DiffLine { kind: '+', text: new.into() }); hunk.new_count += 1; }
                _ => {}
            }
        }
        hunks.push(hunk);
    }
    hunks
}

fn format_diff(filename: &str, old_text: &str, new_text: &str) -> String {
    let hunks = compute_diff(old_text, new_text);
    if hunks.is_empty() { return "(no changes)".into(); }
    let mut out = format!("--- {filename} (original)\n+++ {filename} (modified)\n");
    for h in &hunks {
        out.push_str(&format!("@@ -{},{} +{},{} @@\n", h.old_start, h.old_count, h.new_start, h.new_count));
        for l in &h.lines { out.push_str(&format!("{}{}\n", l.kind, l.text)); }
    }
    out.trim_end_matches('\n').to_string()
}

fn diff_summary(old_text: &str, new_text: &str) -> String {
    let hunks = compute_diff(old_text, new_text);
    let (mut added, mut removed) = (0usize, 0usize);
    for h in &hunks {
        for l in &h.lines {
            if l.kind == '+' { added += 1; }
            if l.kind == '-' { removed += 1; }
        }
    }
    if added == 0 && removed == 0 { return "no line changes".into(); }
    format!("+{added}/-{removed} lines")
}

fn format_write_result(path: &str, original: &str, content: &str, is_new: bool) -> String {
    let filename = path.rsplit('/').next().unwrap_or(path);
    if is_new {
        format!("Created new file: {path}\n\n{}", format_diff(filename, original, content))
    } else {
        format!("Wrote {path} ({})\n\n{}", diff_summary(original, content), format_diff(filename, original, content))
    }
}

fn format_edit_result(path: &str, current: &str, new_content: &str, is_new: bool) -> String {
    let filename = path.rsplit('/').next().unwrap_or(path);
    if is_new {
        format!("Created new file: {path}\n\n{}", format_diff(filename, current, new_content))
    } else {
        format!("Edited {path} ({})\n\n{}", diff_summary(current, new_content), format_diff(filename, current, new_content))
    }
}

fn format_grep_results(matches: &[GrepFileMatch], pattern: &str) -> String {
    if matches.is_empty() { return format!("(no matches for: {pattern})"); }
    let total: usize = matches.iter().map(|m| m.matches.len()).sum();
    let mut out = format!("{total} {} across {} {}\n",
        if total == 1 { "match" } else { "matches" },
        matches.len(),
        if matches.len() == 1 { "file" } else { "files" });
    for fm in matches {
        let n = fm.matches.len();
        out.push_str(&format!("\n{} ({n} {})\n", fm.path, if n == 1 { "match" } else { "matches" }));
        for lm in &fm.matches {
            out.push_str(&format!("  {:5}: {}\n", lm.line_num, lm.line));
        }
    }
    out.trim_end_matches('\n').to_string()
}

fn format_size(n: i64) -> String {
    if n < 1024 { format!("{n}B") }
    else if n < 1024 * 1024 { format!("{:.1}KB", n as f64 / 1024.0) }
    else { format!("{:.1}MB", n as f64 / (1024.0 * 1024.0)) }
}
