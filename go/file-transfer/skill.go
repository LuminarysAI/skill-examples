// Package main implements file-transfer-skill — cross-node file operations
// with virtual path syntax: node-id:///absolute/path
//
// Local copies use fs_copy ABI. Cross-node transfers use
// file_transfer_send / file_transfer_recv via the cluster relay.
// Directory listing uses fs_ls ABI. Node list uses cluster_node_list ABI.
//
// @skill:id      ai.luminarys.go.file-transfer
// @skill:name    "File Transfer Skill"
// @skill:version 3.0.0
// @skill:desc    "Copy and list files across cluster nodes. Use node-id:///path syntax for remote paths."
//
//go:generate lmsk -lang go -verbose .
package main

import (
	"fmt"
	"strings"

	sdk "github.com/LuminarysAI/sdk-go"
)

// parsePath splits "node-id:///absolute/path" into (nodeID, path).
// If no node prefix, returns ("", path) meaning local node.
func parsePath(vpath string) (nodeID, path string) {
	idx := strings.Index(vpath, "://")
	if idx <= 0 {
		return "", vpath
	}
	return vpath[:idx], vpath[idx+3:]
}

// @skill:method copy "Copy a file locally or between cluster nodes. Use node-id:///path for remote paths."
// @skill:param  source required "Source path (e.g. slave-1:///data/file.tar.gz or /data/file.tar.gz)"
// @skill:param  dest   required "Destination path (e.g. master:///data/file.tar.gz or /data/file.tar.gz)"
// @skill:result "Transfer result"
func Copy(ctx *sdk.Context, source, dest string) (string, error) {
	srcNode, srcPath := parsePath(source)
	dstNode, dstPath := parsePath(dest)

	srcIsLocal := srcNode == ""
	dstIsLocal := dstNode == ""

	if srcIsLocal && dstIsLocal {
		// Both local — use fs_copy ABI.
		if err := sdk.FsCopy(srcPath, dstPath); err != nil {
			return "", fmt.Errorf("local copy: %v", err)
		}
		return fmt.Sprintf("copied %s → %s (local)", srcPath, dstPath), nil
	}

	if srcIsLocal && !dstIsLocal {
		// Push: send local file to remote node.
		if err := sdk.FileTransferSend(dstNode, srcPath, dstPath); err != nil {
			return "", fmt.Errorf("send to %s: %v", dstNode, err)
		}
		return fmt.Sprintf("sent %s → %s:%s", srcPath, dstNode, dstPath), nil
	}

	if !srcIsLocal && dstIsLocal {
		// Pull: request file from remote node.
		if err := sdk.FileTransferRecv(srcNode, srcPath, dstPath); err != nil {
			return "", fmt.Errorf("recv from %s: %v", srcNode, err)
		}
		return fmt.Sprintf("received %s:%s → %s", srcNode, srcPath, dstPath), nil
	}

	return "", fmt.Errorf("cannot copy between two remote nodes (%s → %s); "+
		"copy to local first, then to destination", srcNode, dstNode)
}

// @skill:method list "List files in a directory."
// @skill:param  path required "Directory path (absolute)"
// @skill:result "File listing with names and sizes"
func List(ctx *sdk.Context, path string) (string, error) {
	entries, err := sdk.FsLs(path, false)
	if err != nil {
		return "", err
	}
	var lines []string
	for _, e := range entries {
		kind := "file"
		if e.IsDir {
			kind = "dir "
		}
		lines = append(lines, fmt.Sprintf("%s  %8d  %s", kind, e.Size, e.Name))
	}
	if len(lines) == 0 {
		return "(empty directory)", nil
	}
	return strings.Join(lines, "\n"), nil
}

// @skill:method nodes "List known cluster nodes with their roles and skills."
// @skill:result "Node list with current node indicator"
func Nodes(ctx *sdk.Context) (string, error) {
	result, err := sdk.ClusterNodeList()
	if err != nil {
		return "", err
	}
	var lines []string
	for _, node := range result.Nodes {
		marker := "  "
		if node.NodeID == result.CurrentNode {
			marker = "* "
		}
		skills := strings.Join(node.Skills, ", ")
		if skills == "" {
			skills = "(no skills)"
		}
		lines = append(lines, fmt.Sprintf("%s%s [%s]: %s", marker, node.NodeID, node.Role, skills))
	}
	return strings.Join(lines, "\n"), nil
}
