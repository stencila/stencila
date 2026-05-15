//! Capture environment facts for export provenance.
//!
//! This module records process-level runtime details and common lockfile
//! digests near the source document or Git repository root. Those facts help
//! verifiers understand the environment used to render an exported asset.

use std::{collections::BTreeSet, path::Path};

use stencila_codec_utils::{closest_git_repo, git_file_info, git_head_sha};

use crate::{EnvironmentSnapshot, FileDigestSnapshot, RuntimeSnapshot, media};

/// Capture reproducibility-relevant environment facts for an export.
///
/// Environment facts help reviewers understand the context that rendered the
/// asset, but they can also reveal private infrastructure. This function records
/// a deliberately small baseline and lets the privacy projection decide what is
/// safe to publish.
pub(super) fn environment_snapshot_for(source_path: Option<&Path>) -> EnvironmentSnapshot {
    let (repository, commit, informational_uri) = environment_git_info(source_path);

    EnvironmentSnapshot {
        os: Some(std::env::consts::OS.to_string()),
        architecture: Some(std::env::consts::ARCH.to_string()),
        runtimes: vec![RuntimeSnapshot {
            name: Some("stencila".to_string()),
            version: Some(stencila_version::STENCILA_VERSION.to_string()),
        }],
        manifests: file_digest_snapshots(source_path, COMMON_MANIFESTS),
        lockfiles: file_digest_snapshots(source_path, COMMON_LOCKFILES),
        repository,
        commit,
        informational_uri,
        ..Default::default()
    }
}

/// Collect digests for common environment files near the source document.
///
/// Checking both the source directory and Git root catches the common cases
/// where documents live below project-level environment files.
fn file_digest_snapshots(source_path: Option<&Path>, names: &[&str]) -> Vec<FileDigestSnapshot> {
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
    let mut files = Vec::new();
    for dir in dirs {
        for name in names {
            let path = dir.join(name);
            if !path.is_file() || !seen.insert(path.clone()) {
                continue;
            }

            let Some(digest) = media::sha256_file(&path).ok() else {
                continue;
            };

            files.push(FileDigestSnapshot {
                path: Some(display_environment_file_path(
                    &path,
                    repo_root.as_deref().unwrap_or(source_dir),
                )),
                digest: Some(digest),
            });
        }
    }

    files
}

const COMMON_MANIFESTS: &[&str] = &[
    "Cargo.toml",
    "package.json",
    "pyproject.toml",
    "requirements.txt",
    "requirements.in",
    "Pipfile",
    "environment.yml",
    "environment.yaml",
    "pixi.toml",
    "devbox.json",
    "mise.toml",
];

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

/// Capture repository details for the environment context.
fn environment_git_info(
    source_path: Option<&Path>,
) -> (Option<String>, Option<String>, Option<String>) {
    let Some(source_path) = source_path else {
        return (None, None, None);
    };

    let repository = git_file_info(source_path).ok().and_then(|info| info.origin);
    let commit = closest_git_repo(source_path)
        .ok()
        .as_deref()
        .and_then(git_head_sha);
    let informational_uri = match (&repository, &commit) {
        (Some(repository), Some(commit))
            if repository.starts_with("https://github.com/") && commit.len() == 40 =>
        {
            Some(format!("{repository}/tree/{commit}"))
        }
        _ => None,
    };

    (repository, commit, informational_uri)
}

/// Render an environment file path relative to the project context.
///
/// Absolute host paths are noisy and often private. Relative, slash-normalized
/// paths are enough for a verifier to locate the file within the signed project.
fn display_environment_file_path(path: &Path, base_dir: &Path) -> String {
    path.strip_prefix(base_dir)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
