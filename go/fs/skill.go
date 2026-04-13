// Package main implements fs-skill — sandboxed file system operations.
// All paths passed to skill methods must be absolute. The host validates each
// path against the directories declared in manifest.yaml (fs.dirs / fs.root).
//
// @skill:id      ai.luminarys.go.fs
// @skill:name    "File System Skill"
// @skill:version 4.0.0
// @skill:desc    "File system operations. All paths must be absolute (e.g. /home/user/project/src/main.go). The host validates access against declared directories."
//
//go:generate lmsk -lang go -verbose .
package main

import (
	"fmt"
	"strings"
	"time"

	sdk "github.com/LuminarysAI/sdk-go"
)

// =============================================================================
// Internal: diff utilities (ported from diffOptions.ts / edit.ts)
// =============================================================================

// diffLine represents one line in a unified diff hunk.
type diffLine struct {
	kind rune // '+', '-', or ' '
	text string
}

// diffHunk is a contiguous block of changes with surrounding context.
type diffHunk struct {
	oldStart, oldCount int
	newStart, newCount int
	lines              []diffLine
}

// diffStat holds line/char change counts split into model vs user edits.
type diffStat struct {
	addedLines   int
	removedLines int
	addedChars   int
	removedChars int
}

// computeDiff produces a minimal unified diff between oldText and newText.
// context controls how many unchanged lines appear around each change (default 3).
func computeDiff(oldText, newText string, context int) []diffHunk {
	if context < 0 {
		context = 3
	}
	oldLines := splitLines(oldText)
	newLines := splitLines(newText)

	// Myers LCS-based diff: build edit script via DP on line equality.
	lcs := lcsLines(oldLines, newLines)

	type editOp struct {
		kind rune // '+', '-', '='
		old  string
		new  string
	}
	var ops []editOp
	oi, ni := 0, 0
	for _, pair := range lcs {
		// deletions before this match
		for oi < pair[0] {
			ops = append(ops, editOp{'-', oldLines[oi], ""})
			oi++
		}
		// insertions before this match
		for ni < pair[1] {
			ops = append(ops, editOp{'+', "", newLines[ni]})
			ni++
		}
		ops = append(ops, editOp{'=', oldLines[oi], newLines[ni]})
		oi++
		ni++
	}
	for oi < len(oldLines) {
		ops = append(ops, editOp{'-', oldLines[oi], ""})
		oi++
	}
	for ni < len(newLines) {
		ops = append(ops, editOp{'+', "", newLines[ni]})
		ni++
	}

	// Group ops into hunks with context lines.
	var hunks []diffHunk
	oldLine, newLine := 1, 1

	type span struct{ start, end int } // indices into ops that are changed
	// Find changed spans
	var changed []span
	i := 0
	for i < len(ops) {
		if ops[i].kind == '=' {
			i++
			continue
		}
		j := i
		for j < len(ops) && ops[j].kind != '=' {
			j++
		}
		changed = append(changed, span{i, j})
		i = j
	}

	// Build hunks
	opOld := make([]int, len(ops)) // old line number for op[i]
	opNew := make([]int, len(ops))
	{
		o, n := 1, 1
		for idx, op := range ops {
			opOld[idx] = o
			opNew[idx] = n
			if op.kind == '=' || op.kind == '-' {
				o++
			}
			if op.kind == '=' || op.kind == '+' {
				n++
			}
		}
		_, _ = o, n
	}

	_ = oldLine
	_ = newLine

	for ci, s := range changed {
		// context window: from previous hunk's end or start of file
		ctxStart := s.start - context
		if ctxStart < 0 {
			ctxStart = 0
		}
		// merge with previous hunk if close enough
		if ci > 0 {
			prev := changed[ci-1]
			if ctxStart <= prev.end+context {
				// they overlap — already merged by hunk building below
			}
		}
		ctxEnd := s.end + context
		if ctxEnd > len(ops) {
			ctxEnd = len(ops)
		}

		// Skip changed spans already covered by the previous hunk
		if len(hunks) > 0 {
			last := &hunks[len(hunks)-1]
			// check if this span is already included
			endCovered := last.oldStart + last.oldCount - 1
			if opOld[s.start] <= endCovered+context {
				// Extend previous hunk
				extendEnd := ctxEnd
				for extendEnd < len(ops) {
					extendEnd++
					break
				}
				extendEnd = ctxEnd
				for _, op := range ops[len(last.lines):extendEnd] {
					switch op.kind {
					case '=':
						last.lines = append(last.lines, diffLine{' ', op.old})
						last.oldCount++
						last.newCount++
					case '-':
						last.lines = append(last.lines, diffLine{'-', op.old})
						last.oldCount++
					case '+':
						last.lines = append(last.lines, diffLine{'+', op.new})
						last.newCount++
					}
				}
				continue
			}
		}

		var hunk diffHunk
		hunk.oldStart = opOld[ctxStart]
		hunk.newStart = opNew[ctxStart]
		for _, op := range ops[ctxStart:ctxEnd] {
			switch op.kind {
			case '=':
				hunk.lines = append(hunk.lines, diffLine{' ', op.old})
				hunk.oldCount++
				hunk.newCount++
			case '-':
				hunk.lines = append(hunk.lines, diffLine{'-', op.old})
				hunk.oldCount++
			case '+':
				hunk.lines = append(hunk.lines, diffLine{'+', op.new})
				hunk.newCount++
			}
		}
		hunks = append(hunks, hunk)
	}
	return hunks
}

// formatDiff renders hunks as a unified diff string with a given filename.
func formatDiff(filename, oldText, newText string) string {
	hunks := computeDiff(oldText, newText, 3)
	if len(hunks) == 0 {
		return "(no changes)"
	}
	var sb strings.Builder
	fmt.Fprintf(&sb, "--- %s (original)\n", filename)
	fmt.Fprintf(&sb, "+++ %s (modified)\n", filename)
	for _, h := range hunks {
		fmt.Fprintf(&sb, "@@ -%d,%d +%d,%d @@\n", h.oldStart, h.oldCount, h.newStart, h.newCount)
		for _, l := range h.lines {
			fmt.Fprintf(&sb, "%c%s\n", l.kind, l.text)
		}
	}
	return strings.TrimRight(sb.String(), "\n")
}

// calcDiffStat counts added/removed lines and chars between oldText and newText.
func calcDiffStat(oldText, newText string) diffStat {
	hunks := computeDiff(oldText, newText, 0)
	var s diffStat
	for _, h := range hunks {
		for _, l := range h.lines {
			switch l.kind {
			case '+':
				s.addedLines++
				s.addedChars += len(l.text)
			case '-':
				s.removedLines++
				s.removedChars += len(l.text)
			}
		}
	}
	return s
}

// diffSummary returns a one-line human-readable diff summary: "+N/-M lines".
func diffSummary(oldText, newText string) string {
	s := calcDiffStat(oldText, newText)
	if s.addedLines == 0 && s.removedLines == 0 {
		return "no line changes"
	}
	return fmt.Sprintf("+%d/-%d lines", s.addedLines, s.removedLines)
}

// ---------------------------------------------------------------------------
// LCS helper (Hunt-McIlroy simplified, O(n*m) DP)
// ---------------------------------------------------------------------------

// lcsLines returns pairs [oldIdx, newIdx] of matched lines in order.
func lcsLines(a, b []string) [][2]int {
	m, n := len(a), len(b)
	if m == 0 || n == 0 {
		return nil
	}
	// Build index of b lines for fast lookup
	bIdx := make(map[string][]int, n)
	for i, s := range b {
		bIdx[s] = append(bIdx[s], i)
	}

	// DP table (space-optimised: two rows)
	dp := make([][]int, m+1)
	for i := range dp {
		dp[i] = make([]int, n+1)
	}
	for i := m - 1; i >= 0; i-- {
		for j := n - 1; j >= 0; j-- {
			if a[i] == b[j] {
				dp[i][j] = dp[i+1][j+1] + 1
			} else if dp[i+1][j] > dp[i][j+1] {
				dp[i][j] = dp[i+1][j]
			} else {
				dp[i][j] = dp[i][j+1]
			}
		}
	}

	var result [][2]int
	i, j := 0, 0
	for i < m && j < n {
		if a[i] == b[j] {
			result = append(result, [2]int{i, j})
			i++
			j++
		} else if dp[i+1][j] >= dp[i][j+1] {
			i++
		} else {
			j++
		}
	}
	return result
}

// splitLines splits text into lines preserving content (no trailing newline on last line).
func splitLines(s string) []string {
	if s == "" {
		return nil
	}
	s = strings.ReplaceAll(s, "\r\n", "\n")
	lines := strings.Split(s, "\n")
	// Remove trailing empty line caused by final newline
	if len(lines) > 0 && lines[len(lines)-1] == "" {
		lines = lines[:len(lines)-1]
	}
	return lines
}

// countOccurrences returns the number of non-overlapping occurrences of sub in s.
func countOccurrences(s, sub string) int {
	if sub == "" {
		return 0
	}
	return strings.Count(s, sub)
}

// applyReplacement replaces oldString with newString in content.
// If replaceAll, replaces all occurrences; otherwise replaces the first.
// For a new file (content == "" and oldString == ""), returns newString directly.
func applyReplacement(content, oldString, newString string, replaceAll bool) string {
	if oldString == "" && content == "" {
		return newString // new file
	}
	if oldString == "" {
		return content // guard: no-op if file exists
	}
	if replaceAll {
		return strings.ReplaceAll(content, oldString, newString)
	}
	return strings.Replace(content, oldString, newString, 1)
}

// =============================================================================
// read / read_lines
// =============================================================================

// ReadFile reads the full contents of a text file.
//
// For files larger than a few hundred lines, use read_lines with offset and
// limit to avoid loading the whole file — the host will do the slicing.
//
// @skill:method read "Read the full contents of a text file. Rejects binary files."
// @skill:param  path required "Absolute path to the file (e.g. /home/user/project/src/main.go)" example:src/main.go
// @skill:result "File contents as UTF-8 text, or an error if the file is binary"
func ReadFile(path string) (string, error) {
	tc, err := sdk.FsReadLines(sdk.FSReadLinesRequest{Path: path})
	if err != nil {
		return "", err
	}
	return strings.Join(tc.Lines, "\n"), nil
}

// ReadLines reads a range of lines from a file (1-based, inclusive).
//
// The host slices the file server-side — only the requested lines are
// transferred over WASM memory, making this efficient for large files.
// Set end_line to 0 to read until the end of the file.
// Each line is prefixed with its 1-based line number for easy reference.
//
// Typical workflow after search_in_file:
//  1. search_in_file finds a match at line 42
//  2. read_lines with start_line=38, end_line=55 shows context around it
//
// @skill:method read_lines "Read a range of lines. Efficient for large files — only the requested slice is transferred."
// @skill:param  path       required "Absolute path to the file (e.g. /home/user/project/src/main.go)"
// @skill:param  start_line required "First line to read (1-based)" example:10
// @skill:param  end_line   required "Last line to read (inclusive). 0 = read to end of file." example:30
// @skill:result "Header with line range and total, then each line prefixed with its line number"
func ReadLines(path string, startLine, endLine int64) (string, error) {
	if startLine < 1 {
		startLine = 1
	}
	// Convert 1-based inclusive [startLine, endLine] to 0-based offset + limit.
	offset := int(startLine - 1)
	limit := 0
	if endLine > 0 {
		if endLine < startLine {
			return "", fmt.Errorf("start_line (%d) is after end_line (%d)", startLine, endLine)
		}
		limit = int(endLine - startLine + 1)
	}

	tc, err := sdk.FsReadLines(sdk.FSReadLinesRequest{
		Path:   path,
		Offset: offset,
		Limit:  limit,
	})
	if err != nil {
		return "", err
	}

	actualStart := int64(tc.Offset + 1) // back to 1-based
	actualEnd := int64(tc.Offset + len(tc.Lines))

	var sb strings.Builder
	if tc.IsTruncated {
		fmt.Fprintf(&sb, "Showing lines %d-%d of %d total (truncated)\n\n",
			actualStart, actualEnd, tc.TotalLines)
	} else {
		fmt.Fprintf(&sb, "Showing lines %d-%d of %d total\n\n",
			actualStart, actualEnd, tc.TotalLines)
	}
	for i, line := range tc.Lines {
		fmt.Fprintf(&sb, "%5d: %s\n", actualStart+int64(i), line)
	}
	return strings.TrimRight(sb.String(), "\n"), nil
}

// =============================================================================
// write  (full file overwrite — ported from write-file.ts)
// =============================================================================

// WriteFile writes text content to a file, replacing its entire contents.
// Creates the file (and any missing parent directories) if it does not exist.
// Returns a unified diff so the LLM can verify the exact changes made.
//
// @skill:method write "Write text content to a file, replacing it entirely. Returns a diff of the changes."
// @skill:param  path    required "Absolute path to the file (e.g. /home/user/project/src/main.go)" example:src/config.yaml
// @skill:param  content required "Complete new file content (UTF-8 text)"
// @skill:result "Success message with unified diff showing what changed"
func WriteFile(path string, content string) (string, error) {
	// Read existing content for diff (best-effort — file may not exist yet)
	originalContent := ""
	isNewFile := false
	if existing, err := sdk.FsRead(path); err == nil {
		originalContent = string(existing)
	} else {
		isNewFile = true
	}

	if err := sdk.FsWrite(path, []byte(content)); err != nil {
		return "", err
	}

	return formatWriteResult(path, originalContent, content, isNewFile), nil
}

// AppendLine appends a single line of text (with trailing newline) to a file.
// Creates the file if it does not exist.
// @skill:method append_line "Append a line of text to a file. Creates the file if needed."
// @skill:param  path required "Absolute path to the file (e.g. /home/user/project/src/main.go)"
// @skill:param  line required "Text to append (a newline is added automatically)"
// @skill:result "Success message"
func AppendLine(path string, line string) (string, error) {
	existing, _ := sdk.FsRead(path)
	data := append(existing, []byte(strings.TrimRight(line, "\n")+"\n")...)
	if err := sdk.FsWrite(path, data); err != nil {
		return "", err
	}
	return fmt.Sprintf("Appended 1 line to %s", path), nil
}

func formatWriteResult(path, originalContent, newContent string, isNewFile bool) string {
	var sb strings.Builder
	if isNewFile {
		fmt.Fprintf(&sb, "Created new file: %s\n", path)
	} else {
		s := calcDiffStat(originalContent, newContent)
		fmt.Fprintf(&sb, "Wrote %s (%s)\n", path, fmt.Sprintf("+%d/-%d lines", s.addedLines, s.removedLines))
	}
	// Always include the diff so the LLM can verify the exact changes.
	filename := path[strings.LastIndex(path, "/")+1:]
	sb.WriteString("\n")
	sb.WriteString(formatDiff(filename, originalContent, newContent))
	return sb.String()
}

// =============================================================================
// edit  (targeted string replacement — ported from edit.ts)
// =============================================================================

// EditFile replaces an exact string within a file with new text.
//
// This is the preferred way to make targeted changes: instead of rewriting
// the whole file, supply the exact surrounding context you want to replace
// and the replacement text. The operation fails if old_string is not found
// or matches multiple locations (use replace_all to replace every occurrence).
//
// To create a new file, set old_string to "" (empty).
// To delete text, set new_string to "" (empty).
//
// Returns a unified diff confirming exactly what changed.
//
// @skill:method edit "Replace an exact string in a file with new text. Fails if old_string is not found or is ambiguous."
// @skill:param  path        required "Absolute path to the file (e.g. /home/user/project/src/main.go)" example:src/main.go
// @skill:param  old_string  required "The exact text to replace, including surrounding context (whitespace, indentation). Must match exactly once unless replace_all is true. Use empty string to create a new file."
// @skill:param  new_string  optional "The replacement text. Empty string deletes old_string."
// @skill:param  replace_all optional "Replace every occurrence of old_string instead of requiring a unique match." default:false
// @skill:result "Unified diff of the change, or an error describing why the edit failed"
func EditFile(path, oldString, newString string, replaceAll bool) (string, error) {
	// ── Read current content ──────────────────────────────────────────────────
	var currentContent string
	isNewFile := false
	if existing, err := sdk.FsRead(path); err == nil {
		currentContent = strings.ReplaceAll(string(existing), "\r\n", "\n")
	} else {
		isNewFile = true
	}

	// ── Validate ──────────────────────────────────────────────────────────────
	if oldString == "" && !isNewFile {
		return "", fmt.Errorf(
			"file already exists: %s — use an empty old_string only to create a new file, not to modify an existing one", path)
	}
	if !isNewFile && oldString != "" {
		n := countOccurrences(currentContent, oldString)
		switch {
		case n == 0:
			return "", fmt.Errorf(
				"old_string not found in %s — the exact text was not matched; "+
					"check whitespace, indentation and surrounding context", path)
		case n > 1 && !replaceAll:
			return "", fmt.Errorf(
				"old_string matches %d locations in %s — add more context to make it unique, "+
					"or set replace_all:true to replace every occurrence", n, path)
		}
		if oldString == newString {
			return "", fmt.Errorf("old_string and new_string are identical — no changes to apply")
		}
	}

	// ── Apply replacement ─────────────────────────────────────────────────────
	newContent := applyReplacement(currentContent, oldString, newString, replaceAll)

	if !isNewFile && newContent == currentContent {
		return "", fmt.Errorf("resulting content is identical to current content — no changes applied")
	}

	// ── Write ─────────────────────────────────────────────────────────────────
	if err := sdk.FsWrite(path, []byte(newContent)); err != nil {
		return "", fmt.Errorf("write failed: %w", err)
	}

	// ── Build result ──────────────────────────────────────────────────────────
	return formatEditResult(path, currentContent, newContent, isNewFile), nil
}

func formatEditResult(path, currentContent, newContent string, isNewFile bool) string {
	var sb strings.Builder
	if isNewFile {
		fmt.Fprintf(&sb, "Created new file: %s\n", path)
	} else {
		stats := diffSummary(currentContent, newContent)
		fmt.Fprintf(&sb, "Edited %s (%s)\n", path, stats)
	}

	// Diff confirms the exact change — essential for the LLM to verify correctness.
	filename := path[strings.LastIndex(path, "/")+1:]
	sb.WriteString("\n")
	sb.WriteString(formatDiff(filename, currentContent, newContent))

	// Snippet: show a few lines around the change for quick orientation.
	snippet := extractSnippet(currentContent, newContent, 4)
	if snippet != "" {
		sb.WriteString("\n\n--- context ---\n")
		sb.WriteString(snippet)
	}
	return sb.String()
}

// extractSnippet returns a few lines around the first changed region.
// contextLines controls how many unchanged lines are shown on each side.
func extractSnippet(oldText, newText string, contextLines int) string {
	hunks := computeDiff(oldText, newText, contextLines)
	if len(hunks) == 0 {
		return ""
	}
	h := hunks[0] // show the first changed region
	newLines := splitLines(newText)

	start := h.newStart - 1
	end := start + h.newCount
	if start < 0 {
		start = 0
	}
	if end > len(newLines) {
		end = len(newLines)
	}

	var sb strings.Builder
	total := len(newLines)
	for i := start; i < end; i++ {
		fmt.Fprintf(&sb, "%5d: %s\n", i+1, newLines[i])
	}
	if end < total {
		fmt.Fprintf(&sb, "  ... (%d lines follow)\n", total-end)
	}
	return strings.TrimRight(sb.String(), "\n")
}

// =============================================================================
// delete
// =============================================================================

// DeleteFile permanently deletes a file.
// @skill:method delete "Permanently delete a file."
// @skill:param  path required "Absolute path to the file (e.g. /home/user/project/src/main.go)"
// @skill:result "Deleted file path"
func DeleteFile(path string) (string, error) {
	if err := sdk.FsDelete(path); err != nil {
		return "", err
	}
	return path, nil
}

// =============================================================================
// Directory operations
// =============================================================================

// MakeDir creates a directory and all missing parents.
// @skill:method mkdir "Create a directory (parents created automatically)."
// @skill:param  path required "Absolute directory path. Supports brace expansion: /data/{logs,cache} creates two dirs" example:/home/user/project/reports/2024
// @skill:result "Created directory path"
func MakeDir(path string) (string, error) {
	if err := sdk.FsMkdir(path); err != nil {
		return "", err
	}
	return path, nil
}

// ListDir lists the contents of a directory.
// @skill:method list "List the contents of a directory."
// @skill:param  path     required "Absolute path to the directory" example:/home/user/project
// @skill:param  detailed optional "Include modification time and permissions" default:false
// @skill:result "Directory listing, one entry per line"
func ListDir(path string, detailed bool) (string, error) {
	entries, err := sdk.FsLs(path, detailed)
	if err != nil {
		return "", err
	}
	if len(entries) == 0 {
		return fmt.Sprintf("(empty directory: %s)", path), nil
	}
	var sb strings.Builder
	fmt.Fprintf(&sb, "%s:\n", path)
	for _, e := range entries {
		if e.IsDir {
			if detailed && e.ModTime != 0 {
				mod := time.Unix(e.ModTime, 0).UTC().Format("2006-01-02 15:04")
				fmt.Fprintf(&sb, "  %s/\t[dir]\t%s\t%s\n", e.Name, mod, e.ModeStr)
			} else {
				fmt.Fprintf(&sb, "  %s/\t[dir]\n", e.Name)
			}
		} else {
			size := formatSize(e.Size)
			if detailed && e.ModTime != 0 {
				mod := time.Unix(e.ModTime, 0).UTC().Format("2006-01-02 15:04")
				fmt.Fprintf(&sb, "  %s\t%s\t%s\t%s\n", e.Name, size, mod, e.ModeStr)
			} else {
				fmt.Fprintf(&sb, "  %s\t%s\n", e.Name, size)
			}
		}
	}
	return strings.TrimRight(sb.String(), "\n"), nil
}

// StatFile returns metadata about a path. Never errors — missing paths return exists:false.
// @skill:method stat "Check if a path exists and get its metadata."
// @skill:param  path required "File or directory path"
// @skill:result "Metadata: path, exists, type, size, modified, permissions"
func StatFile(path string) (string, error) {
	parts := strings.Split(strings.TrimRight(path, "/"), "/")
	name := parts[len(parts)-1]
	dir := strings.Join(parts[:len(parts)-1], "/")
	if dir == "" {
		dir = "."
	}
	entries, err := sdk.FsLs(dir, true)
	if err == nil {
		for _, e := range entries {
			if e.Name == name {
				typ := "file"
				if e.IsDir {
					typ = "directory"
				}
				mod := "unknown"
				if e.ModTime != 0 {
					mod = time.Unix(e.ModTime, 0).UTC().Format("2006-01-02 15:04:05 UTC")
				}
				return fmt.Sprintf(
					"path: %s\nexists: true\ntype: %s\nsize: %d bytes\nmodified: %s\npermissions: %s",
					path, typ, e.Size, mod, e.ModeStr,
				), nil
			}
		}
	}
	if _, err2 := sdk.FsLs(path, false); err2 == nil {
		return fmt.Sprintf("path: %s\nexists: true\ntype: directory", path), nil
	}
	return fmt.Sprintf("path: %s\nexists: false", path), nil
}

func formatSize(n int64) string {
	switch {
	case n < 1024:
		return fmt.Sprintf("%dB", n)
	case n < 1024*1024:
		return fmt.Sprintf("%.1fKB", float64(n)/1024)
	default:
		return fmt.Sprintf("%.1fMB", float64(n)/(1024*1024))
	}
}

// ChangePermissions changes file or directory permissions.
// @skill:method chmod "Change file or directory permissions."
// @skill:param  path      required "File or directory path"
// @skill:param  mode      required "Permission bits as decimal integer: 420=0644, 493=0755, 384=0600" example:420
// @skill:param  recursive optional "Apply recursively to directory contents" default:false
// @skill:result "Path with new permissions applied"
func ChangePermissions(path string, mode int64, recursive bool) (string, error) {
	if err := sdk.FsChmod(path, uint32(mode), recursive); err != nil {
		return "", err
	}
	return fmt.Sprintf("%s (mode %04o)", path, mode), nil
}

// =============================================================================
// move / copy / count_lines
// =============================================================================

// MoveFile renames or moves a file within the sandbox (implemented as copy+delete).
// @skill:method move "Rename or move a file within the sandbox."
// @skill:param  src required "Absolute source file path"
// @skill:param  dst required "Absolute destination file path"
// @skill:result "Destination path"
func MoveFile(src, dst string) (string, error) {
	data, err := sdk.FsRead(src)
	if err != nil {
		return "", fmt.Errorf("move: read %s: %w", src, err)
	}
	if err := sdk.FsWrite(dst, data); err != nil {
		return "", fmt.Errorf("move: write %s: %w", dst, err)
	}
	if err := sdk.FsDelete(src); err != nil {
		return dst, fmt.Errorf("move: cleanup %s: %w (file was copied to %s)", src, err, dst)
	}
	return dst, nil
}

// CopyFile copies a file to a new path within the sandbox.
// @skill:method copy "Copy a file to a new path within the sandbox."
// @skill:param  src required "Absolute source file path"
// @skill:param  dst required "Absolute destination file path"
// @skill:result "Destination path"
func CopyFile(src, dst string) (string, error) {
	data, err := sdk.FsRead(src)
	if err != nil {
		return "", fmt.Errorf("copy: read %s: %w", src, err)
	}
	if err := sdk.FsWrite(dst, data); err != nil {
		return "", fmt.Errorf("copy: write %s: %w", dst, err)
	}
	return dst, nil
}

// CountLines returns the number of lines in a file.
// @skill:method count_lines "Count the number of lines in a file."
// @skill:param  path required "Absolute file path"
// @skill:result "Line count"
func CountLines(path string) (int64, error) {
	// FsReadLines returns TotalLines without transferring file content.
	tc, err := sdk.FsReadLines(sdk.FSReadLinesRequest{Path: path, Offset: 0, Limit: 1})
	if err != nil {
		return 0, err
	}
	return int64(tc.TotalLines), nil
}

// =============================================================================
// Glob / tree
// =============================================================================

// FindFiles finds files matching glob patterns.
// @skill:method find "Find files matching glob patterns. ** = recursive, {a,b} = alternatives."
// @skill:param  patterns   required "Glob patterns, comma-separated" example:**/*.go
// @skill:param  path       optional "Absolute directory path to search. Empty = all allowed directories" default:""
// @skill:param  only_files optional "Return only files, not directories (default: true)" default:true
// @skill:result "Matching paths, one per line"
func FindFiles(patterns string, path string, onlyFiles bool) (string, error) {
	entries, err := sdk.FsGlob(sdk.GlobOptions{
		Patterns:  splitComma(patterns),
		Path:      path,
		OnlyFiles: onlyFiles,
	})
	if err != nil {
		return "", err
	}
	if len(entries) == 0 {
		return fmt.Sprintf("(no matches: %s)", patterns), nil
	}
	var sb strings.Builder
	for _, e := range entries {
		if e.IsDir {
			fmt.Fprintf(&sb, "%s/\n", e.Path)
		} else {
			fmt.Fprintf(&sb, "%s\n", e.Path)
		}
	}
	return strings.TrimRight(sb.String(), "\n"), nil
}

// TreeView shows an indented directory tree.
// @skill:method tree "Show an indented directory tree."
// @skill:param  path      required "Absolute path to root directory" example:/home/user/project
// @skill:param  max_depth optional "Maximum depth (default: 3, 0 = unlimited)" default:3
// @skill:result "Indented tree"
func TreeView(path string, maxDepth int64) (string, error) {
	if maxDepth == 0 {
		maxDepth = 3
	}
	entries, err := sdk.FsGlob(sdk.GlobOptions{
		Patterns: []string{"**"},
		Path:     path,
		MaxDepth: int(maxDepth),
	})
	if err != nil {
		return "", err
	}
	if len(entries) == 0 {
		return fmt.Sprintf("(empty: %s)", path), nil
	}
	root := path
	if root == "" {
		root = "."
	}
	var sb strings.Builder
	fmt.Fprintf(&sb, "%s\n", root)
	for _, e := range entries {
		if e.Path == "." || e.Path == root {
			continue
		}
		depth := strings.Count(e.Path, "/")
		indent := strings.Repeat("    ", depth)
		name := e.Path[strings.LastIndex(e.Path, "/")+1:]
		if e.IsDir {
			fmt.Fprintf(&sb, "%s├── %s/\n", indent, name)
		} else {
			fmt.Fprintf(&sb, "%s├── %s\n", indent, name)
		}
	}
	return strings.TrimRight(sb.String(), "\n"), nil
}

// =============================================================================
// Search (grep)
// =============================================================================

// SearchText searches for a pattern across all text files.
// Returns matching lines with file path and line numbers.
//
// @skill:method search "Search for a pattern across all text files."
// @skill:param  pattern          required "RE2 regex or literal string (set fixed:true for literal)" example:TODO
// @skill:param  path             optional "Directory or file to search (default: entire sandbox)" default:""
// @skill:param  fixed            optional "Treat pattern as literal text (useful for URLs, version strings)" default:false
// @skill:param  case_insensitive optional "Case-insensitive matching" default:false
// @skill:result "Formatted results: summary line, then per-file blocks with line numbers and text"
func SearchText(pattern string, path string, fixed bool, caseInsensitive bool) (string, error) {
	matches, err := sdk.FsGrep(sdk.GrepOptions{
		Pattern:         pattern,
		Path:            path,
		Fixed:           fixed,
		CaseInsensitive: caseInsensitive,
		WithLines:       true,
		TypeFilter:      "text",
	})
	if err != nil {
		return "", err
	}
	return formatGrepResults(matches, pattern), nil
}

// SearchCode searches for a pattern in source code files only.
// @skill:method search_code "Search in source code files only (excludes configs, data, docs)."
// @skill:param  pattern          required "RE2 regex or literal string" example:func New
// @skill:param  path             optional "Directory or file to search" default:""
// @skill:param  fixed            optional "Literal text matching" default:false
// @skill:param  case_insensitive optional "Case-insensitive matching" default:false
// @skill:result "Formatted results with file paths, line numbers, and matching text"
func SearchCode(pattern string, path string, fixed bool, caseInsensitive bool) (string, error) {
	matches, err := sdk.FsGrep(sdk.GrepOptions{
		Pattern:         pattern,
		Path:            path,
		Fixed:           fixed,
		CaseInsensitive: caseInsensitive,
		WithLines:       true,
		TypeFilter:      "code",
	})
	if err != nil {
		return "", err
	}
	return formatGrepResults(matches, pattern), nil
}

// SearchFiles returns only file paths containing the pattern (no line detail).
// Use this for fast triage before deciding which files to read.
// @skill:method search_files "Return paths of files containing the pattern. Fast triage — no line detail."
// @skill:param  pattern          required "RE2 regex or literal string" example:@deprecated
// @skill:param  path             optional "Directory to search" default:""
// @skill:param  case_insensitive optional "Case-insensitive matching" default:false
// @skill:result "File paths, one per line"
func SearchFiles(pattern string, path string, caseInsensitive bool) (string, error) {
	matches, err := sdk.FsGrep(sdk.GrepOptions{
		Pattern:         pattern,
		Path:            path,
		CaseInsensitive: caseInsensitive,
		FilenameOnly:    true,
		TypeFilter:      "text",
	})
	if err != nil {
		return "", err
	}
	if len(matches) == 0 {
		return fmt.Sprintf("(no files contain: %s)", pattern), nil
	}
	var sb strings.Builder
	for _, m := range matches {
		sb.WriteString(m.Path)
		sb.WriteByte('\n')
	}
	return strings.TrimRight(sb.String(), "\n"), nil
}

// SearchInFile searches within a specific file (no extension filter applied).
// @skill:method search_in_file "Search within a specific file by exact path."
// @skill:param  pattern          required "RE2 regex or literal string"
// @skill:param  path             required "Exact path to the file"
// @skill:param  fixed            optional "Literal text matching" default:false
// @skill:param  case_insensitive optional "Case-insensitive matching" default:false
// @skill:result "Matching lines with line numbers"
func SearchInFile(pattern string, path string, fixed bool, caseInsensitive bool) (string, error) {
	matches, err := sdk.FsGrep(sdk.GrepOptions{
		Pattern:         pattern,
		Path:            path,
		Fixed:           fixed,
		CaseInsensitive: caseInsensitive,
		WithLines:       true,
		TypeFilter:      "all",
	})
	if err != nil {
		return "", err
	}
	return formatGrepResults(matches, pattern), nil
}

// formatGrepResults formats grep matches as human-readable text for LLM consumption:
//
//	3 matches across 2 files
//
//	src/server.go (2 matches)
//	   42: func handleRequest(...) {
//	   67: // handleRequest is called by the router
//
//	README.md (1 match)
//	   12: handleRequest processes all incoming HTTP requests
func formatGrepResults(matches []sdk.GrepFileMatch, pattern string) string {
	if len(matches) == 0 {
		return fmt.Sprintf("(no matches for: %s)", pattern)
	}
	total := 0
	for _, fm := range matches {
		total += len(fm.Matches)
	}
	var sb strings.Builder
	fmt.Fprintf(&sb, "%d %s across %d %s\n",
		total, plural(total, "match", "matches"),
		len(matches), plural(len(matches), "file", "files"),
	)
	for _, fm := range matches {
		n := len(fm.Matches)
		fmt.Fprintf(&sb, "\n%s (%d %s)\n", fm.Path, n, plural(n, "match", "matches"))
		for _, lm := range fm.Matches {
			fmt.Fprintf(&sb, "  %5d: %s\n", lm.LineNum, lm.Line)
		}
	}
	return strings.TrimRight(sb.String(), "\n")
}

func plural(n int, singular, pluralForm string) string {
	if n == 1 {
		return singular
	}
	return pluralForm
}

func splitComma(s string) []string {
	var out []string
	for _, p := range strings.Split(s, ",") {
		if t := strings.TrimSpace(p); t != "" {
			out = append(out, t)
		}
	}
	return out
}
