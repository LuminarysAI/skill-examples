/// @skill:id      ai.luminarys.rust.archive
/// @skill:name    "Archive Skill (Rust)"
/// @skill:version 1.0.0
/// @skill:desc    "Create and extract tar.gz/zip archives with exclude filters."

use luminarys_sdk as sdk;

/// @skill:method pack "Create a tar.gz or zip archive from a directory."
/// @skill:param  source  required "Absolute path to the directory to archive"
/// @skill:param  output  required "Absolute path for the output archive file"
/// @skill:param  format  optional "Archive format: tar.gz (default) or zip"
/// @skill:param  exclude optional "Comma-separated exclude glob patterns"
/// @skill:result "Pack result"
pub fn pack(_ctx: &sdk::Context, source: String, output: String, format: String, exclude: String) -> Result<String, sdk::SkillError> {
    let fmt = if format.is_empty() { "tar.gz".to_string() } else { format };
    let result = sdk::archive_pack(&source, &output, &fmt, &exclude)?;
    Ok(format!("packed {} files → {} ({})", result.files_count, output, result.format))
}

/// @skill:method unpack "Extract a tar.gz or zip archive to a directory."
/// @skill:param  archive required "Absolute path to the archive file"
/// @skill:param  dest    required "Absolute path to the destination directory"
/// @skill:param  format  optional "Archive format (auto-detect by extension if empty)"
/// @skill:param  exclude optional "Comma-separated exclude glob patterns"
/// @skill:param  strip   optional "Strip N leading path components"
/// @skill:result "Unpack result"
pub fn unpack(_ctx: &sdk::Context, archive: String, dest: String, format: String, exclude: String, strip: i64) -> Result<String, sdk::SkillError> {
    let count = sdk::archive_unpack(&archive, &dest, &format, &exclude, strip as i32)?;
    Ok(format!("extracted {} files → {}", count, dest))
}

/// @skill:method list "List contents of a tar.gz or zip archive."
/// @skill:param  archive required "Absolute path to the archive file"
/// @skill:param  format  optional "Archive format (auto-detect by extension if empty)"
/// @skill:param  exclude optional "Comma-separated exclude glob patterns"
/// @skill:result "Archive contents listing"
pub fn list(_ctx: &sdk::Context, archive: String, format: String, exclude: String) -> Result<String, sdk::SkillError> {
    let entries = sdk::archive_list(&archive, &format, &exclude)?;
    let mut lines = Vec::new();
    for e in &entries {
        let kind = if e.is_dir { "dir " } else { "file" };
        lines.push(format!("{}  {:>8}  {}", kind, e.size, e.name));
    }
    if lines.is_empty() {
        Ok("(empty archive)".into())
    } else {
        Ok(lines.join("\n"))
    }
}
