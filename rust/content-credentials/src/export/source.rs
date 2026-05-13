//! Source-control and source-range helpers for export provenance.
//!
//! Export credentials record source repository state, clean/dirty status, patch
//! digests, and source line links. This module keeps those file-system and Git
//! details separate from document-node projection. Untracked source files are
//! reported as dirty without a commit because `HEAD` does not contain their
//! bytes.

use std::path::Path;

use stencila_codec_utils::{closest_git_repo, git_file_info, git_head_sha, git_patch_digest};

use crate::{SourceRangeSnapshot, SourceSnapshot};

/// Capture source-control facts for the exported document.
///
/// Source facts let a verifier locate the authored state behind a signed asset.
/// Dirty and untracked files are handled conservatively because a misleading
/// commit pointer is worse than omitting the commit entirely.
pub(super) fn source_snapshot_for(source_path: Option<&Path>) -> Option<SourceSnapshot> {
    let source_path = source_path?;
    if !source_path.exists() {
        return None;
    }

    let info = git_file_info(source_path).ok()?;
    let relative_path = info.path?;

    let dirty = info.commit.as_deref() == Some("dirty");
    let untracked = info.commit.as_deref() == Some("untracked");

    let repo_root = if dirty || untracked {
        closest_git_repo(source_path).ok()
    } else {
        None
    };

    let commit = if dirty {
        repo_root.as_deref().and_then(git_head_sha)
    } else if untracked {
        tracing::warn!(
            source = %source_path.display(),
            "Signing Content Credentials for an untracked source file"
        );
        None
    } else {
        info.commit
    };

    let patch_digest = if dirty {
        let patch = repo_root
            .as_deref()
            .and_then(|root| git_patch_digest(root, &relative_path));
        if patch.is_none() {
            tracing::warn!(
                source = %source_path.display(),
                "Signing Content Credentials from a dirty worktree without a patch digest"
            );
        }
        patch
    } else {
        None
    };

    let dirty_flag = if dirty || untracked {
        Some(true)
    } else {
        Some(false)
    };

    Some(SourceSnapshot {
        repository: info.origin,
        commit,
        path: Some(relative_path),
        dirty: dirty_flag,
        patch_digest,
        tag: None,
    })
}

/// Extract source text covered by a recorded source range.
///
/// Executed-code ingredients need concrete bytes to sign as their own child
/// manifests. Reading only the mapped range keeps that ingredient scoped to the
/// code that generated the asset instead of copying the entire source document.
pub(super) fn source_range_text(source_path: &Path, range: &SourceRangeSnapshot) -> Option<String> {
    let source = std::fs::read_to_string(source_path).ok()?;
    let start_line = range.start_line.max(1);
    let end_line = source_range_display_end_line(range);
    let lines = source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let line_number = u64::try_from(index).ok()?.saturating_add(1);
            (line_number >= start_line && line_number <= end_line).then_some(line)
        })
        .collect::<Vec<_>>();

    (!lines.is_empty()).then(|| lines.join("\n"))
}

/// Build a repository URL for a source document when it is safely linkable.
///
/// The URL is only emitted for GitHub HTTPS origins with full commit hashes so
/// the link points at immutable source content. Other repository forms may be
/// private, ambiguous, or not directly browseable.
pub(super) fn source_informational_uri(source_path: &Path) -> Option<String> {
    let info = git_file_info(source_path).ok()?;
    let origin = info.origin?;
    let commit = info.commit?;
    let path = info.path?;

    if !(origin.starts_with("https://github.com/") && commit.len() == 40) {
        return None;
    }

    Some(format!("{origin}/blob/{commit}/{path}"))
}

/// Build a source URL with an optional line fragment.
///
/// Linking directly to the executed source range makes C2PA ingredient metadata
/// useful to humans while keeping the full source text out of the parent
/// assertion.
pub(super) fn source_informational_uri_with_range(
    source_path: &Path,
    source_range: Option<&SourceRangeSnapshot>,
) -> Option<String> {
    let mut uri = source_informational_uri(source_path)?;
    if let Some(range) = source_range {
        uri.push_str(&source_range_line_fragment(range));
    }
    Some(uri)
}

fn source_range_line_fragment(range: &SourceRangeSnapshot) -> String {
    let start_line = range.start_line.max(1);
    let end_line = source_range_display_end_line(range).max(start_line);

    if start_line == end_line {
        format!("#L{start_line}")
    } else {
        format!("#L{start_line}-L{end_line}")
    }
}

/// Return the display end line for a source range.
///
/// Range end positions are exclusive in common source maps. When the range ends
/// at column one of the following line, the human-facing line span should stop on
/// the previous line.
pub(super) fn source_range_display_end_line(range: &SourceRangeSnapshot) -> u64 {
    if range.end_column <= 1 && range.end_line > range.start_line {
        range.end_line.saturating_sub(1)
    } else {
        range.end_line
    }
}
