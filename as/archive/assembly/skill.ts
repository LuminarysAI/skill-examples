/**
 * @skill:id      ai.luminarys.as.archive
 * @skill:name    "Archive Skill (AS)"
 * @skill:version 1.0.0
 * @skill:desc    "Create and extract tar.gz/zip archives with exclude filters."
 */

import { Context,  archivePack, archiveUnpack, archiveList, ArchiveEntry } from "@luminarys/sdk-as";

// @skill:method pack "Create a tar.gz or zip archive from a directory."
// @skill:param  source  required "Absolute path to the directory to archive"
// @skill:param  output  required "Absolute path for the output archive file"
// @skill:param  format  optional "Archive format: tar.gz (default) or zip"
// @skill:param  exclude optional "Comma-separated exclude glob patterns"
// @skill:result "Number of files packed"
export function pack(_ctx: Context, source: string, output: string, format: string, exclude: string): string {
  const fmt = format.length > 0 ? format : "tar.gz";
  const r = archivePack(source, output, fmt, exclude);
  return "packed " + r.files_count.toString() + " files → " + output + " (" + r.format + ")";
}

// @skill:method unpack "Extract a tar.gz or zip archive to a directory."
// @skill:param  archive required "Absolute path to the archive file"
// @skill:param  dest    required "Absolute path to the destination directory"
// @skill:param  format  optional "Archive format (auto-detect by extension if empty)"
// @skill:param  exclude optional "Comma-separated exclude glob patterns"
// @skill:param  strip   optional "Strip N leading path components"
// @skill:result "Number of files extracted"
export function unpack(_ctx: Context, archive: string, dest: string, format: string, exclude: string, strip: i64): string {
  const count = archiveUnpack(archive, dest, format, exclude, strip as i32);
  return "extracted " + count.toString() + " files → " + dest;
}

// @skill:method list "List contents of a tar.gz or zip archive."
// @skill:param  archive required "Absolute path to the archive file"
// @skill:param  format  optional "Archive format (auto-detect by extension if empty)"
// @skill:param  exclude optional "Comma-separated exclude glob patterns"
// @skill:result "Archive contents listing"
export function list(_ctx: Context, archive: string, format: string, exclude: string): string {
  const entries = archiveList(archive, format, exclude);
  const lines: string[] = [];
  for (let i = 0; i < entries.length; i++) {
    const e = entries[i];
    const kind = e.is_dir ? "dir " : "file";
    lines.push(kind + "  " + e.size.toString().padStart(8, " ") + "  " + e.name);
  }
  if (lines.length == 0) return "(empty archive)";
  return lines.join("\n");
}
