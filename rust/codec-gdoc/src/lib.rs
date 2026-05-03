use std::path::{Path, PathBuf};

use eyre::{Result, bail, eyre};
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;
use thiserror::Error;
use url::Url;

use stencila_codec::{
    Codec, EncodeOptions, PushDryRunFile, PushDryRunOptions, PushResult,
    stencila_format::Format,
    stencila_schema::{
        Block, CreativeWorkVariant, ImageObject, Inline, Node, Visitor, WalkControl, WalkNode,
    },
};
use stencila_codec_docx::DocxCodec;

/// Error type for Google Docs operations
#[derive(Debug, Error, Serialize, Deserialize)]
pub enum GDocError {
    /// Google account not linked to Stencila account
    ///
    /// The user needs to connect their Google account via the Stencila Cloud picker.
    /// The `connect_url` may contain a tenant-specific URL for managed environments.
    #[error("Google account not linked. Use Google Picker to connect and grant access.")]
    NotLinked {
        /// URL to connect the Google account (if available, may be tenant-specific)
        connect_url: Option<String>,
    },

    /// Access denied to the document (403)
    ///
    /// The user needs to grant access via the Google Picker.
    /// The `doc_id` can be used to construct a picker URL.
    #[error("Access denied to Google Doc '{doc_id}'. Use Google Picker to grant access.")]
    AccessDenied {
        /// The Google Doc ID that access was denied to
        doc_id: String,
    },

    /// Document not found (404)
    ///
    /// The document doesn't exist or has been deleted. Unlike AccessDenied,
    /// this error should not trigger the picker flow.
    #[error(
        "Google Doc '{doc_id}' not found. The document may have been deleted or the URL is incorrect."
    )]
    NotFound {
        /// The Google Doc ID that was not found
        doc_id: String,
    },

    /// Other error
    #[error("{0}")]
    #[serde(skip)]
    Other(#[from] eyre::Report),
}

impl From<stencila_cloud::mirror::Error> for GDocError {
    fn from(error: stencila_cloud::mirror::Error) -> Self {
        GDocError::Other(eyre!(error))
    }
}

/// Information about a Google Doc
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GDocInfo {
    /// The Google Drive file ID
    pub id: String,
    /// The Google Docs URL
    pub url: Url,
}

/// Push a document to Google Docs
///
/// If `existing_url` is provided, updates the existing document.
/// Otherwise, creates a new document.
///
/// This function will obtain a Google Drive access token from Stencila Cloud.
///
/// Returns `GDocError::NotLinked` if the Google account is not connected.
/// Returns `GDocError::AccessDenied` if the app doesn't have access to the
/// document (when updating an existing document).
///
/// Returns a PushResult with the URL of the Google Doc on success.
pub async fn push(
    node: &Node,
    path: Option<&Path>,
    title: Option<&str>,
    url: Option<&Url>,
    dry_run: Option<PushDryRunOptions>,
) -> Result<PushResult> {
    if let Some(dry_run) = dry_run {
        if let Some(url) = url {
            // Update existing document
            let doc_id = extract_doc_id(url)?;
            return update_dry_run(node, path, &doc_id, dry_run).await;
        }

        // Create new document
        return upload_dry_run(node, path, title, dry_run).await;
    }

    let workspace_id = if workspace_required(node) {
        let workspace_path = path.unwrap_or_else(|| Path::new("."));
        Some(stencila_cloud::ensure_workspace(workspace_path).await?.0)
    } else {
        None
    };

    let remote_url: Url = stencila_cloud::mirror::call::<_, _, GDocError>(
        "codec-gdoc.push",
        PushParams {
            node,
            path: path.map(PathBuf::from),
            title,
            url,
            workspace_id,
        },
    )
    .await?;

    Ok(PushResult::Uploaded(remote_url))
}

#[derive(Serialize)]
struct PushParams<'a> {
    node: &'a Node,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<&'a Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    workspace_id: Option<String>,
}

/// Check whether a hosted Google Docs push needs a Stencila Cloud workspace.
///
/// Most pushes only need a Google access token, so the mirror can handle them
/// without any caller-local project context. Some node types, such as local
/// images, citations, code expressions, and visible code-chunk outputs, are
/// first uploaded to Stencila Cloud assets or links. Those callbacks require a
/// workspace id, which must be resolved by the CLI before the request is sent
/// to the deployed mirror container because the container cannot discover the
/// caller's local Git repository from the serialized source path.
fn workspace_required(node: &Node) -> bool {
    struct Detector {
        required: bool,
    }

    fn cloud_uploaded_image(image: &ImageObject) -> bool {
        !image.content_url.is_empty()
            && !starts_with_ignore_ascii_case(&image.content_url, "http://")
            && !starts_with_ignore_ascii_case(&image.content_url, "https://")
    }

    fn starts_with_ignore_ascii_case(value: &str, prefix: &str) -> bool {
        value
            .get(..prefix.len())
            .is_some_and(|start| start.eq_ignore_ascii_case(prefix))
    }

    impl Visitor for Detector {
        fn visit_node(&mut self, node: &Node) -> WalkControl {
            if let Node::ImageObject(image) = node
                && cloud_uploaded_image(image)
            {
                self.required = true;
                return WalkControl::Break;
            }

            WalkControl::Continue
        }

        fn visit_work(&mut self, work: &CreativeWorkVariant) -> WalkControl {
            if let CreativeWorkVariant::ImageObject(image) = work
                && cloud_uploaded_image(image)
            {
                self.required = true;
                return WalkControl::Break;
            }

            WalkControl::Continue
        }

        fn visit_block(&mut self, block: &Block) -> WalkControl {
            match block {
                Block::CodeChunk(code_chunk)
                    if code_chunk.outputs.as_deref().is_some_and(|outputs| {
                        outputs
                            .iter()
                            .any(|output| matches!(output, Node::Datatable(..) | Node::Table(..)))
                    }) =>
                {
                    self.required = true;
                    WalkControl::Break
                }
                Block::ImageObject(image) if cloud_uploaded_image(image) => {
                    self.required = true;
                    WalkControl::Break
                }
                _ => WalkControl::Continue,
            }
        }

        fn visit_inline(&mut self, inline: &Inline) -> WalkControl {
            match inline {
                Inline::Citation(..) | Inline::CodeExpression(..) => {
                    self.required = true;
                    WalkControl::Break
                }
                Inline::ImageObject(image) if cloud_uploaded_image(image) => {
                    self.required = true;
                    WalkControl::Break
                }
                _ => WalkControl::Continue,
            }
        }
    }

    let mut detector = Detector { required: false };
    node.walk(&mut detector);
    detector.required
}

#[derive(Serialize)]
struct UrlParams<'a> {
    url: &'a Url,
}

/// Upload a new document to Google Docs in dry-run mode.
async fn upload_dry_run(
    node: &Node,
    path: Option<&Path>,
    title: Option<&str>,
    dry_run: PushDryRunOptions,
) -> Result<PushResult> {
    let temp_file = NamedTempFile::new()?;
    let temp_path = temp_file.path();

    DocxCodec
        .to_path(
            node,
            temp_path,
            Some(EncodeOptions {
                format: Some(Format::GDocx),
                from_path: path.map(PathBuf::from),
                reproducible: Some(true),
                ..Default::default()
            }),
        )
        .await?;

    let metadata = tokio::fs::metadata(temp_path).await?;
    let file_size = metadata.len();

    let filename = format!("{}.docx", title.unwrap_or("Untitled"));

    let local_path = if let Some(output_dir) = &dry_run.output_dir {
        let dest_path = output_dir.join(&filename);
        tokio::fs::create_dir_all(output_dir).await?;
        tokio::fs::copy(temp_path, &dest_path).await?;
        Some(dest_path)
    } else {
        None
    };

    let mock_url = Url::parse("https://docs.google.com/document/d/...")?;

    let dry_run_file = PushDryRunFile {
        storage_path: filename,
        local_path,
        size: file_size,
        compressed: false,
        route: None,
    };

    Ok(PushResult::DryRun {
        url: mock_url,
        files: vec![dry_run_file],
        output_dir: dry_run.output_dir,
    })
}

/// Update an existing Google Doc in dry-run mode.
async fn update_dry_run(
    node: &Node,
    path: Option<&Path>,
    doc_id: &str,
    dry_run: PushDryRunOptions,
) -> Result<PushResult> {
    let temp_file = NamedTempFile::new()?;
    let temp_path = temp_file.path();

    DocxCodec
        .to_path(
            node,
            temp_path,
            Some(EncodeOptions {
                format: Some(Format::GDocx),
                from_path: path.map(PathBuf::from),
                reproducible: Some(true),
                ..Default::default()
            }),
        )
        .await?;

    let metadata = tokio::fs::metadata(temp_path).await?;
    let file_size = metadata.len();

    let filename = format!("{doc_id}.docx");

    let local_path = if let Some(output_dir) = &dry_run.output_dir {
        let dest_path = output_dir.join(&filename);
        tokio::fs::create_dir_all(output_dir).await?;
        tokio::fs::copy(temp_path, &dest_path).await?;
        Some(dest_path)
    } else {
        None
    };

    let url = Url::parse(&format!("https://docs.google.com/document/d/{doc_id}"))?;

    let dry_run_file = PushDryRunFile {
        storage_path: filename,
        local_path,
        size: file_size,
        compressed: false,
        route: None,
    };

    Ok(PushResult::DryRun {
        url,
        files: vec![dry_run_file],
        output_dir: dry_run.output_dir,
    })
}

/// Pull a document from Google Docs
///
/// Downloads the document as DOCX and saves it to the specified path.
///
/// Returns `GDocError::NotLinked` if the Google account is not connected.
/// Returns `GDocError::AccessDenied` if the app doesn't have access to the
/// document. The caller should handle both by prompting the user via the
/// Google Picker, then retrying.
///
/// This function uses a non-retrying token fetch so callers can handle
/// connection errors appropriately (e.g., by opening the picker).
pub async fn pull(url: &Url, dest: &Path) -> Result<(), GDocError> {
    let node: Node =
        stencila_cloud::mirror::call::<_, _, GDocError>("codec-gdoc.pull", UrlParams { url })
            .await?;

    DocxCodec
        .to_path(
            &node,
            dest,
            Some(EncodeOptions {
                format: Some(Format::GDocx),
                reproducible: Some(true),
                ..Default::default()
            }),
        )
        .await
        .map_err(GDocError::Other)?;

    Ok(())
}

/// Time that a Google Doc was last modified as a Unix timestamp
///
/// This function will obtain a Google Drive access token from Stencila Cloud,
/// prompting the user to connect their account if necessary.
pub async fn modified_at(url: &Url) -> Result<u64> {
    let modified_at = stencila_cloud::mirror::call::<_, _, GDocError>(
        "codec-gdoc.modified_at",
        UrlParams { url },
    )
    .await?;

    Ok(modified_at)
}

/// Extract the document ID from a Google Docs URL
///
/// Supports URLs in the format:
/// - https://docs.google.com/document/d/{id}/edit
/// - https://docs.google.com/document/d/{id}
pub fn extract_doc_id(url: &Url) -> Result<String> {
    // Check that it's a Google Docs URL
    if url.host_str() != Some("docs.google.com") {
        bail!("Not a Google Docs URL: {url}");
    }

    // Parse path segments
    let segments: Vec<&str> = url
        .path_segments()
        .ok_or_else(|| eyre!("Invalid URL"))?
        .collect();

    // Expected format: /document/d/{id}/... or /document/d/{id}
    if segments.len() >= 3 && segments[0] == "document" && segments[1] == "d" {
        Ok(segments[2].to_string())
    } else {
        bail!("Invalid Google Docs URL format: {url}");
    }
}
