//! Project Stencila document nodes into export provenance snapshots.
//!
//! This module owns the Stencila Schema-specific projection used by export
//! signing. It builds document, media output, producer, and activity snapshots
//! while delegating source-control, execution, and environment facts to sibling
//! modules. Non-primary assets distinguish executable subjects from direct
//! media/container renditions so manifests do not over-claim execution.

use std::{collections::BTreeMap, path::Path};

use stencila_codec_text_trait::to_text;
use stencila_schema::{
    Article, Block, CodeBlock, CodeChunk, CodeExpression, Figure, Inline, Node, Table,
};

use crate::{
    ActivitySnapshot, AssetSnapshot, CredentialProfile, DocumentSnapshot, ProducerSnapshot,
    ProvenanceSnapshot, SourceRangeSnapshot,
};

use super::{
    environment::environment_snapshot_for, execution::execution_snapshot_for,
    source::source_snapshot_for,
};

/// Build a provenance snapshot for an asset emitted by a codec export.
///
/// Export signing has to bridge two domains: Stencila document structure and the
/// source bytes represented by a manifest. The C2PA hard binding validates the
/// final signed asset bytes; this projection keeps those facts separate so
/// document exports, executable outputs, and plain media renditions can all share
/// the same assertion schema without overstating what happened.
pub(super) fn build_export_snapshot(
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
        profile,
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
        if is_executable_node(subject) {
            snapshot.executed_node = Some(document_snapshot_for(subject, false, source_ranges));
            snapshot.output_node = output_node_snapshot_for(subject, asset_path, source_ranges);
        } else {
            snapshot.output_node = Some(document_snapshot_for(subject, false, source_ranges));
        }
    }
    snapshot.producer = Some(producer_snapshot_for(codec_name, primary));
    snapshot.source = source_snapshot_for(source_path);
    snapshot.execution = execution_snapshot_for(subject);
    snapshot.environment = Some(environment_snapshot_for(source_path));

    snapshot.activity = Some(activity_snapshot_for(primary, &snapshot));

    if let Some(privacy) = &mut snapshot.privacy {
        let policy = profile.policy_name().to_string();
        privacy.personal_data.policy = Some(policy.clone());
        privacy.secrets.policy = Some(policy);
    }

    snapshot
}

/// Return whether a side-asset subject represents executable document behavior.
///
/// Only executable subjects should populate `executedNode` and create executable
/// input ingredients. Plain figures and media objects may still be signed assets,
/// but treating them as executed code would mislead C2PA consumers.
fn is_executable_node(node: &Node) -> bool {
    matches!(
        node,
        Node::Button(_)
            | Node::CallBlock(_)
            | Node::CodeChunk(_)
            | Node::CodeExpression(_)
            | Node::ForBlock(_)
            | Node::Form(_)
            | Node::IfBlock(_)
            | Node::IfBlockClause(_)
            | Node::IncludeBlock(_)
            | Node::InstructionBlock(_)
            | Node::InstructionInline(_)
            | Node::Parameter(_)
            | Node::PromptBlock(_)
    )
}

/// Options for building a codec export provenance snapshot.
///
/// These values vary between the primary document and each side asset. Grouping
/// them keeps the projection call readable and makes it explicit which context
/// comes from codec dispatch rather than from the Stencila node itself.
#[derive(Clone, Copy)]
pub(super) struct ExportSnapshotOptions<'a> {
    /// Source ranges keyed by full node id.
    ///
    /// Ranges are optional because many source formats are binary, lossless, or
    /// lack a mapping back to the original authoring file.
    pub source_ranges: Option<&'a BTreeMap<String, SourceRangeSnapshot>>,
    /// Original source document path used for repository and line-link facts.
    pub source_path: Option<&'a Path>,
    /// Whether this snapshot is for the primary exported document.
    ///
    /// Primary snapshots describe the root as the represented work; side-asset
    /// snapshots can point at a more specific executed or output node.
    pub primary: bool,
    /// Codec-provided role for a side asset.
    ///
    /// Roles such as `computational-output` or `figure` explain why an otherwise
    /// generic media file exists in the export.
    pub asset_role: Option<&'a str>,
    /// Codec-provided display title for a side asset.
    pub asset_title: Option<&'a str>,
    /// Name of the codec that produced the exported bytes.
    pub codec_name: Option<&'a str>,
    /// Privacy profile used to annotate policy decisions in the snapshot.
    pub profile: CredentialProfile,
}

/// Project a Stencila node into the compact node snapshot used in credentials.
///
/// The assertion should identify the relevant document structure without
/// embedding full node JSON. This function selects stable identifiers and a small
/// set of human-facing fields that help consumers correlate manifests back to
/// source documents.
fn document_snapshot_for(
    subject: &Node,
    root_document: bool,
    source_ranges: Option<&BTreeMap<String, SourceRangeSnapshot>>,
) -> DocumentSnapshot {
    let node_type = subject.node_type().to_string();
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

    let title = node_title_for(subject);

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
    }

    snapshot
}

fn node_title_for(subject: &Node) -> Option<String> {
    match subject {
        Node::Article(article) => article_title(article),
        _ => None,
    }
}

fn article_title(article: &Article) -> Option<String> {
    inlines_to_text(article.title.as_deref()).or_else(|| {
        article.content.iter().find_map(|block| match block {
            Block::Heading(heading) if (0..=1).contains(&heading.level) => {
                inlines_to_text(Some(&heading.content))
            }
            _ => None,
        })
    })
}

/// Extract an author-supplied persistent id from supported node kinds.
///
/// Persistent ids are distinct from structural node ids: they are written by the
/// document author and survive reordering better, so they are useful as public
/// anchors when present.
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

/// Find the output node represented by an executable side asset.
///
/// Executable nodes can emit several media objects. Matching by exported file
/// name keeps the signed asset tied to the specific output when possible, while
/// still accepting the common single-output case.
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

/// Project a media output node into a document snapshot.
///
/// The signed bytes already carry the media payload, so this only records compact
/// metadata such as content URL, media type, and title. Large embedded data URIs
/// are dropped to avoid duplicating private or bulky asset bytes inside the
/// assertion.
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

/// Remove embedded data URIs from output-node metadata.
///
/// Data URIs can contain the entire media payload. Omitting them keeps manifests
/// small and avoids publishing bytes that are already bound by C2PA as the signed
/// asset.
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

/// Convert inline title content into compact display text.
///
/// Credential metadata needs a plain string for titles. Using the codec text
/// projection preserves author-visible wording without leaking the inline node
/// structure into the assertion.
fn inlines_to_text(inlines: Option<&[Inline]>) -> Option<String> {
    let inlines = inlines?;
    if inlines.is_empty() {
        return None;
    }
    let text = inlines
        .iter()
        .map(to_text)
        .collect::<String>()
        .trim()
        .to_string();
    (!text.is_empty()).then_some(text)
}

/// Describe the Stencila software component that produced the export.
///
/// The producer record is claim-generator metadata, not authorship credit. Keeping
/// codec and renderer information here helps consumers reproduce the byte-level
/// export path.
fn producer_snapshot_for(codec_name: Option<&str>, primary: bool) -> ProducerSnapshot {
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

/// Build the export activity that generated the signed asset.
///
/// Activities give C2PA and Stencila-aware consumers an operation to attach node
/// and asset relationships to, instead of leaving the manifest as disconnected
/// facts about files and document nodes.
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
