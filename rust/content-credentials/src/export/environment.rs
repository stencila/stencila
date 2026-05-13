//! Capture environment facts for export provenance.
//!
//! This module records process-level runtime details and common lockfile
//! digests near the source document or Git repository root. Those facts help
//! verifiers understand the environment used to render an exported asset.

use std::{collections::BTreeSet, path::Path};

use stencila_codec_utils::closest_git_repo;

use crate::{EnvironmentSnapshot, FileDigestSnapshot, RuntimeSnapshot, media};

/// Capture reproducibility-relevant environment facts for an export.
///
/// Environment facts help reviewers understand the context that rendered the
/// asset, but they can also reveal private infrastructure. This function records
/// a deliberately small baseline and lets the privacy projection decide what is
/// safe to publish.
pub(super) fn environment_snapshot_for(source_path: Option<&Path>) -> EnvironmentSnapshot {
    EnvironmentSnapshot {
        os: Some(std::env::consts::OS.to_string()),
        architecture: Some(std::env::consts::ARCH.to_string()),
        runtimes: vec![RuntimeSnapshot {
            name: Some("stencila".to_string()),
            version: Some(stencila_version::STENCILA_VERSION.to_string()),
        }],
        lockfiles: lockfile_snapshots(source_path),
        ..Default::default()
    }
}

/// Collect digests for common lockfiles near the source document.
///
/// Lockfiles are a compact way to attest dependency state without embedding full
/// package manifests. Checking both the source directory and Git root catches the
/// common cases where documents live below a project-level environment file.
fn lockfile_snapshots(source_path: Option<&Path>) -> Vec<FileDigestSnapshot> {
    let Some(source_path) = source_path else {
        return Vec::new();
    };

    let Some(source_dir) = source_path.parent() else {
        return Vec::new();
    };

    let repo_root = closest_git_repo(source_path).ok();
    let mut dirs = vec![source_dir.to_path_buf()];
    if let Some(repo_root) = &repo_root
        && repo_root != source_dir
    {
        dirs.push(repo_root.clone());
    }

    let mut seen = BTreeSet::new();
    let mut lockfiles = Vec::new();
    for dir in dirs {
        for name in COMMON_LOCKFILES {
            let path = dir.join(name);
            if !path.is_file() || !seen.insert(path.clone()) {
                continue;
            }

            let Some(digest) = media::sha256_file(&path).ok() else {
                continue;
            };

            lockfiles.push(FileDigestSnapshot {
                path: Some(display_lockfile_path(
                    &path,
                    repo_root.as_deref().unwrap_or(source_dir),
                )),
                digest: Some(digest),
            });
        }
    }

    lockfiles
}

const COMMON_LOCKFILES: &[&str] = &[
    "Cargo.lock",
    "package-lock.json",
    "pnpm-lock.yaml",
    "yarn.lock",
    "bun.lockb",
    "uv.lock",
    "poetry.lock",
    "Pipfile.lock",
    "renv.lock",
];

/// Render a lockfile path relative to the project context.
///
/// Absolute host paths are noisy and often private. Relative, slash-normalized
/// paths are enough for a verifier to locate the file within the signed project.
fn display_lockfile_path(path: &Path, base_dir: &Path) -> String {
    path.strip_prefix(base_dir)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
