//! Build and sign C2PA ingredients for Stencila codec exports.
//!
//! The exported asset may reference a source document, component side assets,
//! and executable source snippets. This module prepares those ingredient
//! snapshots and creates temporary signed ingredient manifests when the C2PA
//! graph needs a concrete manifest to link against.

use std::{
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

use chrono::{DateTime, Utc};
use stencila_codec_utils::{git_file_first_committed_at, git_file_last_committed_at};
use tempfile::{TempDir, tempdir};
use tokio::fs::write;

use crate::{
    ActivitySnapshot, AssetSnapshot, CredentialProducer, CredentialProfile, DocumentSnapshot,
    IngredientRelationship, IngredientSnapshot, IngredientThumbnailSnapshot, ProvenanceSnapshot,
    Result, SignAssetRequest, media,
};

use super::source::{
    source_informational_uri, source_informational_uri_with_range, source_range_display_end_line,
    source_range_text,
};

/// Build the source document ingredient snapshot.
///
/// The source file is the authored input to a document export.
pub(super) fn source_ingredient_snapshot(source_path: Option<&Path>) -> Option<IngredientSnapshot> {
    let source_path = source_path?;
    if !source_path.is_file() {
        return None;
    }

    let media_type = media::guess_media_type(source_path).ok();
    let content_digest = media::sha256_file(source_path).ok();
    let title = source_path
        .file_name()
        .and_then(|name| name.to_str())
        .map(ToString::to_string);
    let thumbnail = media_type
        .as_deref()
        .and_then(|media_type| image_ingredient_thumbnail(source_path, media_type));

    Some(IngredientSnapshot {
        label: Some("source".to_string()),
        title,
        media_type,
        content_digest,
        relationship: IngredientRelationship::InputTo,
        informational_uri: source_informational_uri(source_path),
        thumbnail,
        ..Default::default()
    })
}

/// Build an explicit ingredient thumbnail for image assets.
///
/// The signing layer embeds this as `c2pa.thumbnail.ingredient.*` in the parent
/// manifest. Non-image ingredient thumbnails, such as generated source-code or
/// dataset icons, can use [`IngredientThumbnailSnapshot`] directly.
pub(super) fn image_ingredient_thumbnail(
    path: &Path,
    media_type: &str,
) -> Option<IngredientThumbnailSnapshot> {
    is_thumbnail_media_type(media_type)
        .then(|| IngredientThumbnailSnapshot::from_path_with_media_type(path, media_type))
}

fn is_thumbnail_media_type(media_type: &str) -> bool {
    matches!(
        media_type,
        "image/png" | "image/jpeg" | "image/jpg" | "image/gif" | "image/svg+xml" | "image/webp"
    )
}

/// Temporary signed child manifest used while signing a parent asset.
///
/// The directory handle is kept with the returned asset path so the signed
/// ingredient file remains on disk until the parent C2PA builder has linked it.
pub(super) struct TemporaryIngredientManifest {
    _temp_dir: TempDir,
    pub(super) asset_path: PathBuf,
}

/// Sign the source document into a temporary child manifest.
///
/// C2PA ingredient links need concrete manifest data, not just a path and hash.
/// Signing a temporary copy lets exported assets link back to the source document
/// without modifying the user's original source file.
pub(super) async fn source_ingredient_manifest(
    producer: &CredentialProducer,
    source_path: Option<&Path>,
    source_ingredient: Option<&IngredientSnapshot>,
    credential_profile: CredentialProfile,
) -> Result<Option<TemporaryIngredientManifest>> {
    let Some(source_path) = source_path.filter(|path| path.is_file()) else {
        return Ok(None);
    };

    let Some(file_name) = source_path.file_name() else {
        return Ok(None);
    };

    let temp_dir = tempdir()?;
    let asset_path = temp_dir.path().join(file_name);
    let signed = producer
        .sign_exported_asset(SignAssetRequest {
            input_path: source_path.to_path_buf(),
            output_path: Some(asset_path.clone()),
            media_type: source_ingredient.and_then(|ingredient| ingredient.media_type.clone()),
            title: source_ingredient.and_then(|ingredient| ingredient.title.clone()),
            credential_profile,
            provenance: input_ingredient_provenance(source_created_at(source_path)),
        })
        .await?;

    Ok(Some(TemporaryIngredientManifest {
        _temp_dir: temp_dir,
        asset_path: signed.asset_path,
    }))
}

/// Build minimal provenance for input ingredient manifests.
///
/// Strict C2PA verifiers expect a manifest action, so input ingredients retain a
/// `c2pa.created` action even though the export did not create their source
/// bytes. Supplying the source creation timestamp keeps that required action as
/// close as possible to the input's own history.
fn input_ingredient_provenance(created_at: Option<String>) -> Option<ProvenanceSnapshot> {
    let created_at = created_at?;

    Some(ProvenanceSnapshot {
        activity: Some(ActivitySnapshot {
            kind: Some("create".to_string()),
            started_at: Some(created_at.clone()),
            ended_at: Some(created_at),
            ..Default::default()
        }),
        ..ProvenanceSnapshot::for_asset(AssetSnapshot::default())
    })
}

fn source_created_at(source_path: &Path) -> Option<String> {
    git_source_created_at(source_path).or_else(|| file_created_at(source_path))
}

fn git_source_created_at(source_path: &Path) -> Option<String> {
    git_file_first_committed_at(source_path).or_else(|| git_file_last_committed_at(source_path))
}

fn file_created_at(path: &Path) -> Option<String> {
    let metadata = fs::metadata(path).ok()?;
    metadata
        .created()
        .or_else(|_| metadata.modified())
        .ok()
        .map(system_time_to_rfc3339)
}

fn system_time_to_rfc3339(time: SystemTime) -> String {
    DateTime::<Utc>::from(time).to_rfc3339()
}

/// Add source and executed-code ingredients to an export provenance snapshot.
///
/// Source documents and executable code snippets have different provenance
/// scope. Document exports declare their source document as an `inputTo`
/// ingredient. Executable side assets instead declare only the exact code range
/// that generated the output, so figure manifests do not also point at the wider
/// document.
pub(super) async fn add_source_and_executed_ingredients(
    producer: &CredentialProducer,
    provenance: &mut ProvenanceSnapshot,
    source_ingredient: Option<IngredientSnapshot>,
    source_path: Option<&Path>,
    source_manifest_path: Option<&Path>,
    credential_profile: CredentialProfile,
) -> Result<Vec<TemporaryIngredientManifest>> {
    let mut temporary_manifests = Vec::new();

    if provenance.executed_node.is_none()
        && let Some(ingredient) = source_ingredient.map(|ingredient| {
            source_ingredient_for_snapshot(ingredient, provenance, source_manifest_path)
        })
    {
        provenance.ingredients.push(ingredient);
    }

    if let Some(mut ingredient) = executed_node_ingredient_snapshot(provenance, source_path) {
        if let Some(manifest) = executed_node_ingredient_manifest(
            producer,
            provenance,
            source_path,
            &ingredient,
            credential_profile,
        )
        .await?
        {
            ingredient.manifest_source = Some(manifest.asset_path.clone());
            temporary_manifests.push(manifest);
        }
        provenance.ingredients.push(ingredient);
    }

    Ok(temporary_manifests)
}

/// Adjust a source ingredient for a document export snapshot.
///
/// This prefers the document title for display so ingredient lists are
/// meaningful to reviewers.
fn source_ingredient_for_snapshot(
    mut ingredient: IngredientSnapshot,
    provenance: &ProvenanceSnapshot,
    source_manifest_path: Option<&Path>,
) -> IngredientSnapshot {
    if let Some(title) = provenance
        .root_node
        .title
        .as_deref()
        .map(str::trim)
        .filter(|title| !title.is_empty())
    {
        ingredient.title = Some(title.to_string());
    }

    if let Some(path) = source_manifest_path {
        ingredient.manifest_source = Some(path.to_path_buf());
    }

    ingredient
}

/// Sign the executed source snippet as a temporary ingredient asset.
///
/// The child manifest gives the parent asset a verifiable ingredient link for
/// the exact code region that generated it. If source range text is unavailable,
/// a descriptive fallback still creates a linkable provenance node rather than
/// dropping the execution input entirely.
async fn executed_node_ingredient_manifest(
    producer: &CredentialProducer,
    provenance: &ProvenanceSnapshot,
    source_path: Option<&Path>,
    ingredient: &IngredientSnapshot,
    credential_profile: CredentialProfile,
) -> Result<Option<TemporaryIngredientManifest>> {
    let Some(executed_node) = provenance.executed_node.as_ref() else {
        return Ok(None);
    };

    let source_text = source_path
        .and_then(|path| {
            executed_node
                .source_range
                .as_ref()
                .and_then(|range| source_range_text(path, range))
        })
        .or_else(|| ingredient.description.clone())
        .or_else(|| ingredient.title.clone())
        .unwrap_or_else(|| "Stencila ingredient".to_string());

    let temp_dir = tempdir()?;
    let asset_path = temp_dir
        .path()
        .join(executed_node_ingredient_file_name(executed_node));
    write(&asset_path, source_text).await?;

    let signed = producer
        .sign_exported_asset(SignAssetRequest {
            input_path: asset_path.clone(),
            media_type: ingredient.media_type.clone(),
            title: ingredient.title.clone(),
            credential_profile,
            provenance: input_ingredient_provenance(
                source_path
                    .and_then(source_created_at)
                    .or_else(|| file_created_at(&asset_path)),
            ),
            ..Default::default()
        })
        .await?;

    Ok(Some(TemporaryIngredientManifest {
        _temp_dir: temp_dir,
        asset_path: signed.asset_path,
    }))
}

/// Build the ingredient snapshot for an executed node's source.
///
/// Executed outputs need an `inputTo` ingredient for the code that produced them.
/// The snapshot records source range, digest, language media type, and display
/// metadata so both C2PA actions and Stencila assertions can reference the same
/// input.
fn executed_node_ingredient_snapshot(
    provenance: &ProvenanceSnapshot,
    source_path: Option<&Path>,
) -> Option<IngredientSnapshot> {
    let executed_node = provenance.executed_node.as_ref()?;
    let source_range = executed_node.source_range.as_ref();
    let language = executed_node.programming_language.as_deref();
    let source_format = source_code_format(language);
    let source_text =
        source_path.and_then(|path| source_range.and_then(|range| source_range_text(path, range)));
    let content_digest = source_text
        .as_deref()
        .map(|text| media::sha256_bytes(text.as_bytes()));

    Some(IngredientSnapshot {
        label: Some(executed_node_ingredient_label(executed_node)),
        title: Some(executed_node_ingredient_title(executed_node)),
        media_type: Some(source_format.media_type.to_string()),
        content_digest,
        relationship: IngredientRelationship::InputTo,
        informational_uri: source_path
            .and_then(|path| source_informational_uri_with_range(path, source_range)),
        description: Some(executed_node_ingredient_description(executed_node)),
        ..Default::default()
    })
}

/// Build a stable C2PA ingredient label for an executed node.
///
/// Action ingredient IDs refer to ingredient labels before the C2PA SDK resolves
/// them to hashed references. Using persistent or structural ids keeps those
/// references deterministic across repeated exports.
fn executed_node_ingredient_label(node: &DocumentSnapshot) -> String {
    node.persistent_id
        .as_deref()
        .or(node.node_id.as_deref())
        .and_then(safe_label_fragment)
        .map_or_else(
            || "executed-node".to_string(),
            |id| format!("executed-node-{id}"),
        )
}

/// Build the temporary file name for an executed-code ingredient.
///
/// The file name is mostly diagnostic, but giving it a language extension helps
/// C2PA tools and humans recognize the snippet when temporary manifests are
/// inspected during debugging.
fn executed_node_ingredient_file_name(node: &DocumentSnapshot) -> String {
    format!(
        "{}.{}",
        executed_node_ingredient_label(node),
        source_code_format(node.programming_language.as_deref()).extension
    )
}

/// Convert a node identifier into a label-safe fragment.
///
/// C2PA labels are easier to consume when they avoid punctuation and mixed case.
/// This keeps author-supplied ids useful without letting unusual characters leak
/// into action ingredient references.
fn safe_label_fragment(value: &str) -> Option<String> {
    let fragment = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();

    (!fragment.is_empty()).then_some(fragment)
}

/// Build a human-facing title for an executed-code ingredient.
///
/// Ingredient titles appear in verifier output. Combining language, node type,
/// and a stable identifier makes the source input recognizable without embedding
/// the full source text in every display surface.
fn executed_node_ingredient_title(node: &DocumentSnapshot) -> String {
    let language = node
        .programming_language
        .as_deref()
        .map(display_programming_language);
    let identifier = node
        .persistent_id
        .as_deref()
        .or(node.node_id.as_deref())
        .or(node.label.as_deref());

    match (language, identifier) {
        (Some(language), Some(identifier)) => {
            format!("{language} {} {identifier}", node.node_type)
        }
        (Some(language), None) => format!("{language} {}", node.node_type),
        (None, Some(identifier)) => format!("{} {identifier}", node.node_type),
        (None, None) => node.node_type.clone(),
    }
}

/// Format a programming language name for display.
///
/// Language identifiers are stored in compact machine form. Display text should
/// still respect common spelling for short names such as R.
fn display_programming_language(language: &str) -> String {
    if language.eq_ignore_ascii_case("r") {
        "R".to_string()
    } else {
        let mut chars = language.chars();
        match chars.next() {
            Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
            None => "Code".to_string(),
        }
    }
}

/// Build a concise description for an executed-code ingredient.
///
/// The description gives C2PA consumers context for why the snippet is present.
/// Including source lines when available makes the ingredient useful even if the
/// verifier UI does not expose Stencila-specific node fields.
fn executed_node_ingredient_description(node: &DocumentSnapshot) -> String {
    let range = node.source_range.as_ref().map(|range| {
        let end_line = source_range_display_end_line(range);
        if range.start_line == end_line {
            format!("line {}", range.start_line)
        } else {
            format!("lines {}-{end_line}", range.start_line)
        }
    });

    let language = node
        .programming_language
        .as_deref()
        .map(display_programming_language);

    match (language, range) {
        (Some(language), Some(range)) => {
            format!(
                "{language} {} source that generated this asset ({range})",
                node.node_type
            )
        }
        (Some(language), None) => {
            format!(
                "{language} {} source that generated this asset",
                node.node_type
            )
        }
        (None, Some(range)) => {
            format!(
                "{} source that generated this asset ({range})",
                node.node_type
            )
        }
        (None, None) => format!("{} source that generated this asset", node.node_type),
    }
}

/// Media type and file extension for an executable source snippet.
///
/// The same language lookup drives both the C2PA ingredient format and temporary
/// file name. Keeping them together prevents drift where a snippet is named as
/// one language but advertised as another.
struct SourceCodeFormat {
    media_type: &'static str,
    extension: &'static str,
}

/// Resolve source-code format metadata from a Stencila language string.
///
/// Stencila documents use flexible language labels such as `python`, `py`, or
/// `shell`. Normalizing them here gives C2PA ingredients a stable media type
/// while preserving a useful extension for temporary child assets.
fn source_code_format(language: Option<&str>) -> SourceCodeFormat {
    match language.map(str::to_ascii_lowercase).as_deref() {
        Some("r") => SourceCodeFormat {
            media_type: "text/x-r",
            extension: "r",
        },
        Some("python" | "py") => SourceCodeFormat {
            media_type: "text/x-python",
            extension: "py",
        },
        Some("javascript" | "js") => SourceCodeFormat {
            media_type: "text/javascript",
            extension: "js",
        },
        Some("typescript" | "ts") => SourceCodeFormat {
            media_type: "text/typescript",
            extension: "ts",
        },
        Some("bash" | "sh" | "shell") => SourceCodeFormat {
            media_type: "text/x-shellscript",
            extension: "sh",
        },
        Some("sql") => SourceCodeFormat {
            media_type: "application/sql",
            extension: "sql",
        },
        _ => SourceCodeFormat {
            media_type: "text/plain",
            extension: "txt",
        },
    }
}
