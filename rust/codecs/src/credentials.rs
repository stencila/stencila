//! Build Content Credentials [`ProvenanceSnapshot`] values for encoded outputs.
//!
//! Other crates assemble snapshots and hand them to
//! [`stencila_content_credentials::CredentialProducer`]. This module owns the
//! per-subject projection used by `to_path_with_info`: the document-level
//! snapshot uses the root node as its subject, and each side asset uses its
//! originating node (e.g. a `CodeChunk` for a generated figure) so per-asset
//! credentials carry that node's own execution facts.

use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use chrono::{DateTime, Utc};
use stencila_codec::stencila_schema::{
    Article, CodeBlock, CodeChunk, CodeExpression, CompilationDigest, Duration,
    ExecutionDependency, ExecutionMessage, Figure, Inline, Node, Table, Timestamp,
};
use stencila_codec_text_trait::to_text;
use stencila_codec_utils::{closest_git_repo, git_file_info, git_head_sha, git_patch_digest};
use stencila_content_credentials::{
    ActivitySnapshot, AssetSnapshot, DependencySnapshot, DocumentSnapshot, EnvironmentSnapshot,
    ExecutionDigestSnapshot, ExecutionMessageSnapshot, ExecutionSnapshot, FileDigestSnapshot,
    KernelSnapshot, ProducerSnapshot, ProvenanceSnapshot, RuntimeSnapshot, SourceRangeSnapshot,
    SourceSnapshot, media,
};

/// Build a provenance snapshot for an asset emitted by `to_path_with_info`.
///
/// `root` is the document root, used for source/document context. `subject`
/// is the node whose execution and outputs describe this asset:
/// - For the primary document export, `subject == root` (typically an Article).
/// - For a side asset (e.g. an extracted figure), `subject` is the originating
///   executable node (`CodeChunk`, `CodeExpression`, etc.).
///
/// `source_path` is the original path the document was decoded from (when
/// known). `asset_path` is the file just written. `primary` is true when
/// `asset_path` is the principal output, false for side assets.
pub(crate) fn build_export_snapshot(
    root: &Node,
    subject: &Node,
    asset_path: &Path,
    options: ExportSnapshotOptions,
) -> ProvenanceSnapshot {
    let ExportSnapshotOptions {
        source_ranges,
        source_path,
        primary,
        asset_role,
        asset_title,
        codec_name,
        profile_label,
    } = options;

    let mut snapshot = ProvenanceSnapshot::for_asset(AssetSnapshot::default());

    snapshot.asset.id = Some("exported-asset".to_string());
    snapshot.asset.role = Some(
        if primary {
            "document-export"
        } else {
            asset_role.unwrap_or("asset-export")
        }
        .to_string(),
    );

    snapshot.asset.title = asset_title.map(ToString::to_string).or_else(|| {
        asset_path
            .file_name()
            .and_then(|name| name.to_str())
            .map(ToString::to_string)
    });
    if let Ok(metadata) = std::fs::metadata(asset_path) {
        snapshot.asset.size = Some(metadata.len());
    }

    snapshot.root_node = document_snapshot_for(root, true, source_ranges);
    if !primary {
        snapshot.executed_node = Some(document_snapshot_for(subject, false, source_ranges));
        snapshot.output_node = output_node_snapshot_for(subject, asset_path, source_ranges);
    }
    snapshot.producer = Some(producer_snapshot_for(codec_name, primary));
    snapshot.source = source_snapshot_for(source_path);
    snapshot.execution = execution_snapshot_for(subject);
    snapshot.environment = Some(environment_snapshot_for(source_path));

    snapshot.activity = Some(activity_snapshot_for(primary, &snapshot));

    if let Some(privacy) = &mut snapshot.privacy {
        let policy = format!("org.stencila.credentials.{profile_label}.v1");
        privacy.personal_data.policy = Some(policy.clone());
        privacy.secrets.policy = Some(policy);
    }

    snapshot
}

#[derive(Clone, Copy)]
pub(crate) struct ExportSnapshotOptions<'a> {
    pub source_ranges: Option<&'a BTreeMap<String, SourceRangeSnapshot>>,
    pub source_path: Option<&'a Path>,
    pub primary: bool,
    pub asset_role: Option<&'a str>,
    pub asset_title: Option<&'a str>,
    pub codec_name: Option<&'a str>,
    pub profile_label: &'a str,
}

fn document_snapshot_for(
    subject: &Node,
    root_document: bool,
    source_ranges: Option<&BTreeMap<String, SourceRangeSnapshot>>,
) -> DocumentSnapshot {
    let node_type = subject.node_type().to_string();
    // Drop the 3-character node-type nickname (e.g. `img_`) so the assertion
    // exposes the bare stabilized path (e.g. `content-2-outputs-0`). The
    // `nodeType` field already conveys the kind of node.
    //
    // The root node's stabilized UID is empty (the root is the entry point
    // of the structural path), so omit `nodeId` entirely there rather than
    // recording an empty string.
    let node_id = if root_document {
        None
    } else {
        subject
            .node_id()
            .map(|id| id.uid_str().to_string())
            .filter(|uid| !uid.is_empty())
    };
    let full_node_id = subject.node_id().map(|id| id.to_string());
    let persistent_id = persistent_id_of(subject);
    let source_range = full_node_id
        .as_ref()
        .and_then(|id| source_ranges.and_then(|ranges| ranges.get(id).cloned()));

    let title = match subject {
        Node::Article(Article { title, .. }) => inlines_to_text(title.as_deref()),
        _ => None,
    };

    let mut snapshot = DocumentSnapshot {
        node_type,
        node_id,
        persistent_id,
        source_range,
        title,
        ..Default::default()
    };

    if root_document {
        return snapshot;
    }

    match subject {
        Node::Article(Article { .. }) => {}
        Node::CodeChunk(chunk) => {
            snapshot.label_type = chunk.label_type.as_ref().map(ToString::to_string);
            snapshot.label.clone_from(&chunk.label);
            snapshot
                .programming_language
                .clone_from(&chunk.programming_language);
        }
        Node::CodeExpression(expression) => {
            snapshot
                .programming_language
                .clone_from(&expression.programming_language);
        }
        _ => {}
    };

    snapshot
}

/// Return the author-supplied persistent identifier for a node, when present.
///
/// This reads the Stencila Schema `id` field directly. It is distinct from
/// `Node::node_id()`, which returns the structural runtime identifier.
fn persistent_id_of(node: &Node) -> Option<String> {
    let id = match node {
        Node::Article(Article { id, .. })
        | Node::CodeBlock(CodeBlock { id, .. })
        | Node::CodeChunk(CodeChunk { id, .. })
        | Node::CodeExpression(CodeExpression { id, .. })
        | Node::Figure(Figure { id, .. })
        | Node::Table(Table { id, .. }) => id.as_deref(),
        Node::AudioObject(object) => object.id.as_deref(),
        Node::ImageObject(object) => object.id.as_deref(),
        Node::MediaObject(object) => object.id.as_deref(),
        Node::VideoObject(object) => object.id.as_deref(),
        _ => None,
    };

    id.map(ToString::to_string)
}

fn output_node_snapshot_for(
    subject: &Node,
    asset_path: &Path,
    source_ranges: Option<&BTreeMap<String, SourceRangeSnapshot>>,
) -> Option<DocumentSnapshot> {
    let asset_name = asset_path.file_name().and_then(|name| name.to_str());

    match subject {
        Node::CodeChunk(CodeChunk {
            outputs: Some(nodes),
            ..
        }) => {
            let candidates = nodes
                .iter()
                .filter_map(|node| media_node_snapshot_for(node, source_ranges))
                .collect::<Vec<_>>();

            candidates
                .iter()
                .find(|node| {
                    asset_name.is_some_and(|asset_name| {
                        node.content_url
                            .as_deref()
                            .is_some_and(|url| url.ends_with(asset_name))
                    })
                })
                .cloned()
                .or_else(|| {
                    if candidates.len() == 1 {
                        candidates.into_iter().next()
                    } else {
                        None
                    }
                })
        }
        Node::CodeExpression(CodeExpression {
            output: Some(node), ..
        }) => media_node_snapshot_for(node, source_ranges),
        _ => None,
    }
}

fn media_node_snapshot_for(
    node: &Node,
    source_ranges: Option<&BTreeMap<String, SourceRangeSnapshot>>,
) -> Option<DocumentSnapshot> {
    let mut snapshot = document_snapshot_for(node, false, source_ranges);

    let (content_url, media_type, title) = match node {
        Node::AudioObject(object) => (
            Some(object.content_url.clone()),
            object.media_type.clone(),
            inlines_to_text(object.title.as_deref()),
        ),
        Node::ImageObject(object) => (
            Some(object.content_url.clone()),
            object.media_type.clone(),
            inlines_to_text(object.title.as_deref()),
        ),
        Node::MediaObject(object) => (
            Some(object.content_url.clone()),
            object.media_type.clone(),
            inlines_to_text(object.options.title.as_deref()),
        ),
        Node::VideoObject(object) => (
            Some(object.content_url.clone()),
            object.media_type.clone(),
            inlines_to_text(object.title.as_deref()),
        ),
        _ => return None,
    };

    snapshot.content_url = compact_media_content_url(content_url);
    snapshot.media_type = media_type;
    snapshot.title = title;
    Some(snapshot)
}

fn compact_media_content_url(content_url: Option<String>) -> Option<String> {
    let content_url = content_url?;
    let trimmed = content_url.trim_start();
    if trimmed
        .get(..5)
        .is_some_and(|scheme| scheme.eq_ignore_ascii_case("data:"))
    {
        None
    } else {
        Some(content_url)
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

fn activity_snapshot_for(primary: bool, snapshot: &ProvenanceSnapshot) -> ActivitySnapshot {
    let used_node_ids = if primary {
        snapshot.root_node.node_id.clone()
    } else {
        snapshot
            .output_node
            .as_ref()
            .and_then(|node| node.node_id.clone())
    }
    .into_iter()
    .collect();

    ActivitySnapshot {
        kind: Some("export".to_string()),
        name: Some(if primary {
            "Export document".to_string()
        } else {
            "Export asset".to_string()
        }),
        used_node_ids,
        generated_asset_ids: snapshot.asset.id.clone().into_iter().collect(),
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

/// Borrowed view of the executable fields shared across executable node
/// variants. Each variant has its own `*Options` struct, but the field names
/// match — this lets execution/input projection be generic over the subject.
struct ExecutableView<'a> {
    status: Option<String>,
    ended_at: Option<&'a Timestamp>,
    duration: Option<&'a Duration>,
    digest: Option<&'a CompilationDigest>,
    execution_count: Option<i64>,
    execution_instance: Option<&'a str>,
    language: Option<&'a str>,
    dependencies: &'a [ExecutionDependency],
    messages: &'a [ExecutionMessage],
}

fn executable_view(node: &Node) -> Option<ExecutableView<'_>> {
    match node {
        Node::Article(article) => Some(ExecutableView {
            status: article
                .options
                .execution_status
                .as_ref()
                .map(ToString::to_string),
            ended_at: article.options.execution_ended.as_ref(),
            duration: article.options.execution_duration.as_ref(),
            digest: article.options.execution_digest.as_ref(),
            execution_count: article.options.execution_count,
            execution_instance: article.options.execution_instance.as_deref(),
            language: None,
            dependencies: article
                .options
                .execution_dependencies
                .as_deref()
                .unwrap_or(&[]),
            messages: article.options.execution_messages.as_deref().unwrap_or(&[]),
        }),
        Node::CodeChunk(chunk) => Some(ExecutableView {
            status: chunk
                .options
                .execution_status
                .as_ref()
                .map(ToString::to_string),
            ended_at: chunk.options.execution_ended.as_ref(),
            duration: chunk.options.execution_duration.as_ref(),
            digest: chunk.options.execution_digest.as_ref(),
            execution_count: chunk.options.execution_count,
            execution_instance: chunk.options.execution_instance.as_deref(),
            language: chunk.programming_language.as_deref(),
            dependencies: chunk
                .options
                .execution_dependencies
                .as_deref()
                .unwrap_or(&[]),
            messages: chunk.options.execution_messages.as_deref().unwrap_or(&[]),
        }),
        Node::CodeExpression(expression) => Some(ExecutableView {
            status: expression
                .options
                .execution_status
                .as_ref()
                .map(ToString::to_string),
            ended_at: expression.options.execution_ended.as_ref(),
            duration: expression.options.execution_duration.as_ref(),
            digest: expression.options.execution_digest.as_ref(),
            execution_count: expression.options.execution_count,
            execution_instance: expression.options.execution_instance.as_deref(),
            language: expression.programming_language.as_deref(),
            dependencies: expression
                .options
                .execution_dependencies
                .as_deref()
                .unwrap_or(&[]),
            messages: expression
                .options
                .execution_messages
                .as_deref()
                .unwrap_or(&[]),
        }),
        _ => None,
    }
}

/// Project execution facts from a single subject node (no recursion).
///
/// Returns `None` when the subject has no execution metadata of its own.
fn execution_snapshot_for(subject: &Node) -> Option<ExecutionSnapshot> {
    let view = executable_view(subject)?;

    let has_execution = view.status.is_some()
        || view.ended_at.is_some()
        || view.duration.is_some()
        || view.digest.is_some()
        || view.execution_count.is_some()
        || view.execution_instance.is_some()
        || view.language.is_some()
        || !view.dependencies.is_empty()
        || !view.messages.is_empty();

    if !has_execution {
        return None;
    }

    let mut snapshot = ExecutionSnapshot {
        status: view.status,
        ended_at: view.ended_at.and_then(timestamp_to_rfc3339),
        duration_ms: view.duration.and_then(duration_to_ms),
        digest: view.digest.map(execution_digest_snapshot),
        count: view.execution_count,
        ..Default::default()
    };

    if view.execution_instance.is_some() || view.language.is_some() {
        snapshot.kernel = Some(KernelSnapshot {
            name: view.execution_instance.map(ToString::to_string),
            language: view.language.map(ToString::to_string),
            ..Default::default()
        });
    }

    snapshot.dependencies = view.dependencies.iter().map(dependency_snapshot).collect();
    snapshot.messages = view
        .messages
        .iter()
        .map(execution_message_snapshot)
        .collect();

    Some(snapshot)
}

fn environment_snapshot_for(source_path: Option<&Path>) -> EnvironmentSnapshot {
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

fn display_lockfile_path(path: &Path, base_dir: &Path) -> String {
    path.strip_prefix(base_dir)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn execution_digest_snapshot(digest: &CompilationDigest) -> ExecutionDigestSnapshot {
    ExecutionDigestSnapshot {
        state_digest: Some(value_to_digest(digest.state_digest)),
        semantic_digest: digest.semantic_digest.map(value_to_digest),
        dependencies_digest: digest.dependencies_digest.map(value_to_digest),
        dependencies_stale: digest.dependencies_stale,
        dependencies_failed: digest.dependencies_failed,
    }
}

fn dependency_snapshot(dependency: &ExecutionDependency) -> DependencySnapshot {
    DependencySnapshot {
        node_id: Some(dependency.dependency_id.clone()),
        node_type: Some(dependency.dependency_type.clone()),
        relation: Some(dependency.dependency_relation.to_string()),
        digest: None,
    }
}

fn execution_message_snapshot(message: &ExecutionMessage) -> ExecutionMessageSnapshot {
    ExecutionMessageSnapshot {
        level: Some(message.level.to_string()),
        error_type: message.error_type.clone(),
        message: Some(message.message.clone()),
    }
}

fn value_to_digest(value: u64) -> String {
    format!("stencila:{value:016x}")
}

fn duration_to_ms(duration: &Duration) -> Option<u64> {
    let amount = duration.value;
    let multiplier = match duration.time_unit.to_string().as_str() {
        "Year" => 31_536_000_000,
        "Month" => 2_592_000_000,
        "Week" => 604_800_000,
        "Day" => 86_400_000,
        "Hour" => 3_600_000,
        "Minute" => 60_000,
        "Second" => 1_000,
        "Millisecond" => 1,
        "Microsecond" => return u64::try_from(amount).ok().map(|amount| amount / 1_000),
        "Nanosecond" => return u64::try_from(amount).ok().map(|amount| amount / 1_000_000),
        _ => return None,
    };
    u64::try_from(amount).ok()?.checked_mul(multiplier)
}

fn timestamp_to_rfc3339(timestamp: &Timestamp) -> Option<String> {
    let amount = timestamp.value;
    let millis = match timestamp.time_unit.to_string().as_str() {
        "Second" => amount.checked_mul(1_000)?,
        "Millisecond" => amount,
        "Microsecond" => amount.checked_div(1_000)?,
        "Nanosecond" => amount.checked_div(1_000_000)?,
        _ => return None,
    };
    DateTime::<Utc>::from_timestamp_millis(millis).map(|time| time.to_rfc3339())
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        process::{Command, Stdio},
    };

    use stencila_codec::stencila_schema::{Article, ImageObject, Inline, Node, Text};
    use tempfile::TempDir;

    use super::*;

    fn article_with_title(title: &str) -> Node {
        Node::Article(Article {
            title: Some(vec![Inline::Text(Text::from(title))]),
            ..Default::default()
        })
    }

    #[test]
    fn document_snapshot_uses_root_title_with_document_identity() {
        let root = article_with_title("Hello world");

        let snapshot = document_snapshot_for(&root, true, None);

        assert_eq!(snapshot.node_type, "Article");
        // Root nodes do not record a `nodeId` — the path is empty by
        // definition, and `nodeType` already conveys the kind of node.
        assert!(snapshot.node_id.is_none());
        assert_eq!(snapshot.title.as_deref(), Some("Hello world"));
        assert!(snapshot.node_path.is_none());
    }

    #[test]
    fn document_snapshot_strips_node_id_nickname() {
        // A non-root subject reports its stabilized path-based UID without
        // the 3-character node-type prefix that `NodeId::to_string()` adds.
        use stencila_codec::stencila_schema::{Block, Paragraph};

        let mut root = Node::Article(Article {
            content: vec![Block::Paragraph(Paragraph {
                content: vec![Inline::Text(stencila_codec::stencila_schema::Text::from(
                    "hi",
                ))],
                ..Default::default()
            })],
            ..Default::default()
        });
        stencila_node_stabilize::stabilize(&mut root);

        let Node::Article(article) = root else {
            panic!("Expected Article");
        };
        let Block::Paragraph(paragraph) =
            article.content.into_iter().next().expect("paragraph block")
        else {
            panic!("Expected Paragraph");
        };
        let subject = Node::from(Block::Paragraph(paragraph));

        let snapshot = document_snapshot_for(&subject, false, None);

        assert_eq!(snapshot.node_type, "Paragraph");
        assert_eq!(snapshot.node_id.as_deref(), Some("content-0"));
    }

    #[test]
    fn document_snapshot_for_non_article_subject_omits_title() {
        let root = Node::String("hi".to_string());

        let snapshot = document_snapshot_for(&root, false, None);

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

        let root = article_with_title("Untitled");
        let snapshot = build_export_snapshot(
            &root,
            &root,
            &asset_path,
            ExportSnapshotOptions {
                source_ranges: None,
                source_path: None,
                primary: true,
                asset_role: None,
                asset_title: None,
                codec_name: Some("markdown"),
                profile_label: "public",
            },
        );

        assert_eq!(snapshot.asset.role.as_deref(), Some("document-export"));
        assert_eq!(snapshot.asset.title.as_deref(), Some("article.md"));
        assert_eq!(snapshot.asset.size, Some(5));
        assert!(snapshot.executed_node.is_none());
        assert!(snapshot.output_node.is_none());
        assert!(snapshot.source.is_none());
        assert!(snapshot.execution.is_none());
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

    #[test]
    fn build_export_snapshot_uses_side_asset_role() {
        let tmp = TempDir::new().expect("tmp");
        let asset_path = tmp.path().join("table.png");
        fs::write(&asset_path, b"image").expect("write");

        let root = article_with_title("Untitled");
        let snapshot = build_export_snapshot(
            &root,
            &root,
            &asset_path,
            ExportSnapshotOptions {
                source_ranges: None,
                source_path: None,
                primary: false,
                asset_role: Some("table-image"),
                asset_title: Some("Table 1: Example table."),
                codec_name: Some("markdown"),
                profile_label: "public",
            },
        );

        assert_eq!(snapshot.asset.role.as_deref(), Some("table-image"));
        assert_eq!(
            snapshot.asset.title.as_deref(),
            Some("Table 1: Example table.")
        );
        assert!(snapshot.executed_node.is_some());
    }

    #[test]
    fn persistent_id_of_reads_schema_level_id() {
        let mut chunk = CodeChunk::new("plot()".into());
        chunk.id = Some("fig-plot".to_string());
        let node = Node::CodeChunk(chunk);

        assert_eq!(persistent_id_of(&node).as_deref(), Some("fig-plot"));
    }

    #[test]
    fn document_snapshot_records_persistent_id_when_set() {
        let article = Article {
            id: Some("paper-2026".to_string()),
            ..Default::default()
        };
        let root = Node::Article(article);

        let snapshot = document_snapshot_for(&root, true, None);

        assert_eq!(snapshot.persistent_id.as_deref(), Some("paper-2026"));
    }

    #[test]
    fn document_snapshot_uses_full_node_id_for_source_range_lookup() {
        use stencila_codec::stencila_schema::Block;

        let mut chunk = CodeChunk::new("plot()".into());
        chunk.id = Some("fig-1".to_string());
        let mut root = Node::Article(Article {
            content: vec![Block::CodeChunk(chunk)],
            ..Default::default()
        });
        stencila_node_stabilize::stabilize(&mut root);
        let Node::Article(article) = root else {
            panic!("Expected Article");
        };
        let Block::CodeChunk(chunk) = article.content.into_iter().next().expect("code chunk")
        else {
            panic!("Expected CodeChunk");
        };
        let subject = Node::CodeChunk(chunk);

        let mut source_ranges = BTreeMap::new();
        source_ranges.insert(
            "fig_fig-1".to_string(),
            SourceRangeSnapshot {
                start_line: 7,
                start_column: 1,
                end_line: 29,
                end_column: 1,
            },
        );
        source_ranges.insert(
            "cdc_fig-1".to_string(),
            SourceRangeSnapshot {
                start_line: 12,
                start_column: 1,
                end_line: 22,
                end_column: 1,
            },
        );

        let snapshot = document_snapshot_for(&subject, false, Some(&source_ranges));

        assert_eq!(snapshot.node_id.as_deref(), Some("fig-1"));
        assert_eq!(
            snapshot
                .source_range
                .as_ref()
                .map(|range| (range.start_line, range.end_line)),
            Some((12, 22))
        );
    }

    #[test]
    fn document_snapshot_omits_persistent_id_when_unset() {
        let root = article_with_title("Untitled");

        let snapshot = document_snapshot_for(&root, true, None);

        assert!(snapshot.persistent_id.is_none());
    }

    #[test]
    fn output_node_snapshot_matches_media_output() {
        let mut chunk = CodeChunk::new("plot()".into());
        chunk.outputs = Some(vec![Node::ImageObject(ImageObject::new(
            "figures/plot.png".to_string(),
        ))]);

        let output_node =
            output_node_snapshot_for(&Node::CodeChunk(chunk), Path::new("plot.png"), None)
                .expect("output node");

        assert_eq!(output_node.node_type, "ImageObject");
        assert_eq!(output_node.content_url.as_deref(), Some("figures/plot.png"));
    }

    #[test]
    fn output_node_snapshot_omits_data_uri_media_output() {
        let mut chunk = CodeChunk::new("plot()".into());
        chunk.outputs = Some(vec![Node::ImageObject(ImageObject::new(
            "data:image/png;base64,abc123".to_string(),
        ))]);

        let output_node =
            output_node_snapshot_for(&Node::CodeChunk(chunk), Path::new("plot.png"), None)
                .expect("output node");

        assert_eq!(output_node.node_type, "ImageObject");
        assert!(output_node.content_url.is_none());
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
