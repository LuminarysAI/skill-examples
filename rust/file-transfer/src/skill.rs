/// @skill:id      ai.luminarys.rust.file-transfer
/// @skill:name    "File Transfer Skill (Rust)"
/// @skill:version 1.0.0
/// @skill:desc    "Copy and list files across cluster nodes. Use node-id:///path for remote paths."

use luminarys_sdk as sdk;

fn parse_path(vpath: &str) -> (String, String) {
    if let Some(idx) = vpath.find("://") {
        if idx > 0 {
            return (vpath[..idx].to_string(), vpath[idx + 3..].to_string());
        }
    }
    (String::new(), vpath.to_string())
}

/// @skill:method copy "Copy a file locally or between cluster nodes."
/// @skill:param  source required "Source path (e.g. slave-1:///data/file.tar.gz or /data/file.tar.gz)"
/// @skill:param  dest   required "Destination path (e.g. master:///data/file.tar.gz or /data/file.tar.gz)"
/// @skill:result "Transfer result"
pub fn copy(_ctx: &sdk::Context, source: String, dest: String) -> Result<String, sdk::SkillError> {
    let (src_node, src_path) = parse_path(&source);
    let (dst_node, dst_path) = parse_path(&dest);

    let src_local = src_node.is_empty();
    let dst_local = dst_node.is_empty();

    if src_local && dst_local {
        sdk::fs_copy(&src_path, &dst_path)?;
        return Ok(format!("copied {} → {} (local)", src_path, dst_path));
    }
    if src_local && !dst_local {
        sdk::file_transfer_send(&dst_node, &src_path, &dst_path)?;
        return Ok(format!("sent {} → {}:{}", src_path, dst_node, dst_path));
    }
    if !src_local && dst_local {
        sdk::file_transfer_recv(&src_node, &src_path, &dst_path)?;
        return Ok(format!("received {}:{} → {}", src_node, src_path, dst_path));
    }
    Err(sdk::SkillError("cannot copy between two remote nodes".into()))
}

/// @skill:method list "List files in a directory."
/// @skill:param  path required "Directory path (absolute)"
/// @skill:result "File listing"
pub fn list(_ctx: &sdk::Context, path: String) -> Result<String, sdk::SkillError> {
    let entries = sdk::fs_ls(&path, false)?;
    let mut lines = Vec::new();
    for e in &entries {
        let kind = if e.is_dir { "dir " } else { "file" };
        lines.push(format!("{}  {:>8}  {}", kind, e.size, e.name));
    }
    if lines.is_empty() {
        Ok("(empty directory)".into())
    } else {
        Ok(lines.join("\n"))
    }
}

/// @skill:method nodes "List known cluster nodes."
/// @skill:result "Node list"
pub fn nodes(_ctx: &sdk::Context) -> Result<String, sdk::SkillError> {
    let (current, nodes) = sdk::cluster_node_list()?;
    let mut lines = Vec::new();
    for n in &nodes {
        let marker = if n.node_id == current { "* " } else { "  " };
        let skills = n.skills.join(", ");
        lines.push(format!("{}{} [{}]: {}", marker, n.node_id, n.role, if skills.is_empty() { "(no skills)".into() } else { skills }));
    }
    Ok(lines.join("\n"))
}
