/**
 * @skill:id      ai.luminarys.as.file-transfer
 * @skill:name    "File Transfer Skill (AS)"
 * @skill:version 1.0.0
 * @skill:desc    "Copy and list files across cluster nodes. Use node-id:///path for remote paths."
 * @skill:sdk     "@luminarys/sdk-as"
 */

import { Context,  fsCopy, fsLs, DirEntry, fileTransferSend, fileTransferRecv, clusterNodeList, ClusterNodeListResult } from "@luminarys/sdk-as";

function parsePath(vpath: string): string[] {
  const idx = vpath.indexOf("://");
  if (idx <= 0) return ["", vpath];
  return [vpath.substring(0, idx), vpath.substring(idx + 3)];
}

// @skill:method copy "Copy a file locally or between cluster nodes. Use node-id:///path for remote."
// @skill:param  source required "Source path (e.g. slave-1:///data/file.tar.gz or /data/file.tar.gz)"
// @skill:param  dest   required "Destination path (e.g. master:///data/file.tar.gz or /data/file.tar.gz)"
// @skill:result "Transfer result"
export function copy(_ctx: Context, source: string, dest: string): string {
  const src = parsePath(source);
  const dst = parsePath(dest);
  const srcNode = src[0]; const srcPath = src[1];
  const dstNode = dst[0]; const dstPath = dst[1];

  const srcLocal = srcNode.length == 0;
  const dstLocal = dstNode.length == 0;

  if (srcLocal && dstLocal) {
    fsCopy(srcPath, dstPath);
    return "copied " + srcPath + " → " + dstPath + " (local)";
  }
  if (srcLocal && !dstLocal) {
    fileTransferSend(dstNode, srcPath, dstPath);
    return "sent " + srcPath + " → " + dstNode + ":" + dstPath;
  }
  if (!srcLocal && dstLocal) {
    fileTransferRecv(srcNode, srcPath, dstPath);
    return "received " + srcNode + ":" + srcPath + " → " + dstPath;
  }
  return "ERROR: cannot copy between two remote nodes — copy to local first";
}

// @skill:method list "List files in a directory."
// @skill:param  path required "Directory path (absolute)"
// @skill:result "File listing"
export function list(_ctx: Context, path: string): string {
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

// @skill:method nodes "List known cluster nodes."
// @skill:result "Node list with current node indicator"
export function nodes(_ctx: Context): string {
  const resp = clusterNodeList();
  const lines: string[] = [];
  for (let i = 0; i < resp.nodes.length; i++) {
    const node = resp.nodes[i];
    const marker = node.node_id == resp.current_node ? "* " : "  ";
    lines.push(marker + node.node_id + " [" + node.role + "]");
  }
  return lines.join("\n");
}
