//! Build Content Credentials [`ProvenanceSnapshot`] values for encoded outputs.
//!
//! Other crates assemble snapshots and hand them to
//! [`stencila_content_credentials::CredentialProducer`]. This module owns the
//! document/source/producer projection used by `to_path_with_info`. It is
//! intentionally narrow — execution, environment, workflow, and per-asset
//! profiles arrive in later phases.

use std::path::Path;

use stencila_codec::stencila_schema::{Article, Inline, Node};
use stencila_codec_text_trait::to_text;
use stencila_codec_utils::{closest_git_repo, git_file_info, git_head_sha, git_patch_digest};
use stencila_content_credentials::{
    AssetSnapshot, DocumentSnapshot, ProducerSnapshot, ProvenanceSnapshot, SourceSnapshot,
};

/// Build a document-level provenance snapshot for an asset emitted by
/// `to_path_with_info`.
///
/// `node` is the root document node. `source_path` is the original path the
/// node was decoded from (when known). `asset_path` is the file just written.
/// `primary` is true when `asset_path` is the principal output, false for
/// side assets like extracted media.
pub(crate) fn build_export_snapshot(
    node: &Node,
    source_path: Option<&Path>,
    asset_path: &Path,
    primary: bool,
    codec_name: Option<&str>,
    profile_label: &str,
) -> ProvenanceSnapshot {
    let mut snapshot = ProvenanceSnapshot::for_asset(AssetSnapshot::default());

    snapshot.profile = Some(
        if primary {
            "document-export"
        } else {
            "computational-output"
        }
        .to_string(),
    );

    snapshot.asset.title = asset_path
        .file_name()
        .and_then(|name| name.to_str())
        .map(ToString::to_string);
    if let Ok(metadata) = std::fs::metadata(asset_path) {
        snapshot.asset.size = Some(metadata.len());
    }

    snapshot.document = document_snapshot_for(node, source_path);
    snapshot.producer = Some(producer_snapshot_for(codec_name, primary));
    snapshot.source = source_snapshot_for(source_path);

    if let Some(privacy) = &mut snapshot.privacy {
        let policy = format!("org.stencila.credentials.{profile_label}.v1");
        privacy.personal_data.policy = Some(policy.clone());
        privacy.secrets.policy = Some(policy);
    }

    snapshot
}

fn document_snapshot_for(node: &Node, source_path: Option<&Path>) -> DocumentSnapshot {
    let node_type = node.node_type().to_string();
    let node_id = node.node_id().map(|id| id.to_string());

    // Only Article carries a title in this phase; other titled variants
    // (CreativeWork, Periodical, etc.) will be added when their export paths
    // start emitting Content Credentials.
    let title = match node {
        Node::Article(Article { title, .. }) => inlines_to_text(title.as_deref()),
        _ => None,
    };

    let node_path = source_path.map(|path| path.to_string_lossy().into_owned());

    DocumentSnapshot {
        node_type,
        node_id,
        node_path,
        title,
        ..Default::default()
    }
}

fn inlines_to_text(inlines: Option<&[Inline]>) -> Option<String> {
    let inlines = inlines?;
    if inlines.is_empty() {
        return None;
    }
    let text = inlines
        .iter()
        .map(to_text)
        .collect::<Vec<_>>()
        .join("")
        .trim()
        .to_string();
    (!text.is_empty()).then_some(text)
}

fn producer_snapshot_for(codec_name: Option<&str>, primary: bool) -> ProducerSnapshot {
    // `renderer` distinguishes the principal output (`stencila-codecs`) from
    // side assets emitted by the same encode call (`stencila-codecs/asset`,
    // e.g. extracted media). Verifiers use it to tell document-level
    // assertions apart from per-asset ones when they share a codec.
    ProducerSnapshot {
        name: Some("Stencila".to_string()),
        version: Some(stencila_version::STENCILA_VERSION.to_string()),
        codec: codec_name.map(ToString::to_string),
        renderer: Some(if primary {
            "stencila-codecs".to_string()
        } else {
            "stencila-codecs/asset".to_string()
        }),
        ..Default::default()
    }
}

fn source_snapshot_for(source_path: Option<&Path>) -> Option<SourceSnapshot> {
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

    // Dirty/untracked sentinels in `info.commit` aren't real SHAs; resolve
    // HEAD from the repo so the assertion still binds the patch digest to a
    // base commit. Clean files keep the per-file commit returned above.
    let commit = if dirty || untracked {
        repo_root.as_deref().and_then(git_head_sha)
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

    let dirty_flag = if dirty {
        Some(true)
    } else if untracked {
        None
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

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        process::{Command, Stdio},
    };

    use stencila_codec::stencila_schema::{Article, Inline, Node, Text};
    use tempfile::TempDir;

    use super::*;

    fn article_with_title(title: &str) -> Node {
        Node::Article(Article {
            title: Some(vec![Inline::Text(Text::from(title))]),
            ..Default::default()
        })
    }

    #[test]
    fn document_snapshot_extracts_article_title_and_source_path() {
        let node = article_with_title("Hello world");
        let source = PathBuf::from("/tmp/example/article.smd");

        let snapshot = document_snapshot_for(&node, Some(&source));

        assert_eq!(snapshot.node_type, "Article");
        assert!(snapshot.node_id.as_deref().is_some_and(|id| !id.is_empty()));
        assert_eq!(snapshot.title.as_deref(), Some("Hello world"));
        assert_eq!(
            snapshot.node_path.as_deref(),
            Some("/tmp/example/article.smd")
        );
    }

    #[test]
    fn document_snapshot_for_non_article_omits_title() {
        let node = Node::String("hi".to_string());

        let snapshot = document_snapshot_for(&node, None);

        assert_eq!(snapshot.node_type, "String");
        assert!(snapshot.title.is_none());
        assert!(snapshot.node_path.is_none());
    }

    #[test]
    fn producer_snapshot_records_codec_and_renderer() {
        let primary = producer_snapshot_for(Some("markdown"), true);
        assert_eq!(primary.name.as_deref(), Some("Stencila"));
        assert_eq!(primary.codec.as_deref(), Some("markdown"));
        assert_eq!(primary.renderer.as_deref(), Some("stencila-codecs"));
        assert!(primary.version.is_some());

        let asset = producer_snapshot_for(Some("markdown"), false);
        assert_eq!(asset.renderer.as_deref(), Some("stencila-codecs/asset"));
    }

    #[test]
    fn build_export_snapshot_without_source_omits_source() {
        let tmp = TempDir::new().expect("tmp");
        let asset_path = tmp.path().join("article.md");
        fs::write(&asset_path, b"hello").expect("write");

        let snapshot = build_export_snapshot(
            &article_with_title("Untitled"),
            None,
            &asset_path,
            true,
            Some("markdown"),
            "public",
        );

        assert_eq!(snapshot.profile.as_deref(), Some("document-export"));
        assert_eq!(snapshot.asset.title.as_deref(), Some("article.md"));
        assert_eq!(snapshot.asset.size, Some(5));
        assert!(snapshot.source.is_none());
        assert_eq!(
            snapshot.producer.as_ref().and_then(|p| p.codec.as_deref()),
            Some("markdown")
        );
        let privacy = snapshot.privacy.expect("privacy");
        assert_eq!(
            privacy.personal_data.policy.as_deref(),
            Some("org.stencila.credentials.public.v1")
        );
    }

    fn run_git(repo: &std::path::Path, args: &[&str]) {
        let status = Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(args)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .expect("run git");
        assert!(status.success(), "git {args:?} failed");
    }

    fn init_test_repo() -> (TempDir, PathBuf) {
        let tmp = TempDir::new().expect("tmp");
        let repo = tmp.path();
        run_git(repo, &["init", "-q", "-b", "main"]);
        run_git(repo, &["config", "user.email", "test@example.com"]);
        run_git(repo, &["config", "user.name", "Test"]);
        run_git(repo, &["config", "commit.gpgsign", "false"]);
        let article_path = repo.join("article.smd");
        fs::write(&article_path, "# Title\n\nbody\n").expect("write");
        run_git(repo, &["add", "article.smd"]);
        run_git(repo, &["commit", "-q", "-m", "init"]);
        (tmp, article_path)
    }

    fn git_available() -> bool {
        Command::new("git")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }

    #[test]
    fn source_snapshot_for_clean_repo_has_commit_and_path() {
        if !git_available() {
            return;
        }

        let (_tmp, article_path) = init_test_repo();
        let snapshot = source_snapshot_for(Some(&article_path)).expect("snapshot");

        assert_eq!(snapshot.path.as_deref(), Some("article.smd"));
        assert_eq!(snapshot.dirty, Some(false));
        assert!(snapshot.patch_digest.is_none());
        let commit = snapshot.commit.as_deref().expect("commit sha");
        assert_eq!(commit.len(), 40);
    }

    #[test]
    fn source_snapshot_for_dirty_repo_records_patch_digest_and_head() {
        if !git_available() {
            return;
        }

        let (_tmp, article_path) = init_test_repo();
        fs::write(&article_path, "# Title\n\nedited body\n").expect("edit");

        let snapshot = source_snapshot_for(Some(&article_path)).expect("snapshot");

        assert_eq!(snapshot.dirty, Some(true));
        let digest = snapshot.patch_digest.as_deref().expect("patch digest");
        assert!(digest.starts_with("sha256:"));
        let commit = snapshot.commit.as_deref().expect("HEAD commit");
        assert_eq!(commit.len(), 40, "expected HEAD SHA, got {commit}");
    }

    #[test]
    fn source_snapshot_for_path_outside_repo_is_none() {
        let tmp = TempDir::new().expect("tmp");
        let article_path = tmp.path().join("article.smd");
        fs::write(&article_path, "body").expect("write");

        // Some test environments place the system temp dir inside a stray git
        // repo (eg. an empty `/tmp/.git`). Skip when that happens so the test
        // can keep asserting the no-repo behaviour on clean machines.
        if closest_git_repo(&article_path).is_ok() {
            return;
        }

        let snapshot = source_snapshot_for(Some(&article_path));
        assert!(snapshot.is_none(), "no source outside a repo: {snapshot:?}");
    }
}
