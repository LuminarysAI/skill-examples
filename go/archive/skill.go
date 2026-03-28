// Package main implements archive — tar.gz/zip packing and unpacking
// via the host archive ABI.
//
// @skill:id      ai.luminarys.go.archive
// @skill:name    "Archive Skill"
// @skill:version 2.0.0
// @skill:desc    "Create and extract tar.gz/zip archives with exclude filters."
//
// @skill:require fs /data/**:rw
//
//go:generate lmsk -lang go -verbose .
package main

import (
	"fmt"
	"strings"

	sdk "github.com/LuminarysAI/sdk-go"
)

// @skill:method pack "Create a tar.gz or zip archive from a directory."
// @skill:param  source  required "Absolute path to the directory to archive"
// @skill:param  output  required "Absolute path for the output archive file"
// @skill:param  format  optional "Archive format: tar.gz (default) or zip"
// @skill:param  exclude optional "Comma-separated exclude glob patterns (e.g. .git,node_modules)"
// @skill:result "Pack result with file count"
func Pack(ctx *sdk.Context, source, output, format, exclude string) (string, error) {
	if format == "" {
		format = "tar.gz"
	}
	result, err := sdk.ArchivePack(source, output, format, exclude)
	if err != nil {
		return "", err
	}
	return fmt.Sprintf("packed %d files → %s (%s)", result.FilesCount, output, result.Format), nil
}

// @skill:method unpack "Extract a tar.gz or zip archive to a directory."
// @skill:param  archive required "Absolute path to the archive file"
// @skill:param  dest    required "Absolute path to the destination directory"
// @skill:param  format  optional "Archive format (auto-detect by extension if empty)"
// @skill:param  exclude optional "Comma-separated exclude glob patterns"
// @skill:param  strip   optional "Strip N leading path components (like tar --strip-components)"
// @skill:result "Unpack result with file count"
func Unpack(ctx *sdk.Context, archive, dest, format, exclude string, strip int64) (string, error) {
	result, err := sdk.ArchiveUnpack(archive, dest, format, exclude, int(strip))
	if err != nil {
		return "", err
	}
	return fmt.Sprintf("extracted %d files to %s", result.FilesCount, dest), nil
}

// @skill:method list "List contents of a tar.gz or zip archive."
// @skill:param  archive required "Absolute path to the archive file"
// @skill:param  format  optional "Archive format (auto-detect by extension if empty)"
// @skill:param  exclude optional "Comma-separated exclude glob patterns"
// @skill:result "Archive contents listing"
func List(ctx *sdk.Context, archive, format, exclude string) (string, error) {
	entries, err := sdk.ArchiveList(archive, format, exclude)
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
		return "(empty archive)", nil
	}
	return strings.Join(lines, "\n"), nil
}
