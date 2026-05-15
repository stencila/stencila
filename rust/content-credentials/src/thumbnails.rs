//! Static C2PA thumbnail resources for non-image Stencila nodes.

use std::path::Path;

use crate::schema::ProvenanceAssertion;
#[cfg(feature = "export")]
use crate::{DocumentSnapshot, IngredientThumbnailSnapshot};

const SVG_MEDIA_TYPE: &str = "image/svg+xml";

macro_rules! node_icon {
    ($file:literal) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/icons/node-types/",
            $file
        ))
    };
}

/// Static embedded thumbnail bytes.
#[derive(Clone, Copy, Debug)]
pub(crate) struct StaticThumbnail {
    pub(crate) media_type: &'static str,
    pub(crate) bytes: &'static [u8],
}

impl StaticThumbnail {
    #[cfg(feature = "export")]
    #[must_use]
    pub(crate) fn into_ingredient(self) -> IngredientThumbnailSnapshot {
        IngredientThumbnailSnapshot::from_bytes(self.media_type, self.bytes)
    }
}

/// Build a static ingredient thumbnail for a Stencila node type.
#[cfg(feature = "export")]
#[must_use]
pub(crate) fn ingredient_for_node_type(node_type: &str) -> IngredientThumbnailSnapshot {
    for_node(node_type, None, None, None, None).into_ingredient()
}

/// Build a static ingredient thumbnail for a Stencila node.
#[cfg(feature = "export")]
#[must_use]
pub(crate) fn ingredient_for_node(node: &DocumentSnapshot) -> IngredientThumbnailSnapshot {
    for_node(
        &node.node_type,
        node.programming_language.as_deref(),
        node.title.as_deref(),
        node.content_url.as_deref(),
        node.media_type.as_deref(),
    )
    .into_ingredient()
}

/// Build a static ingredient thumbnail for a file-like ingredient.
#[cfg(feature = "export")]
#[must_use]
pub(crate) fn ingredient_for_file(
    path: &Path,
    media_type: Option<&str>,
    title: Option<&str>,
) -> IngredientThumbnailSnapshot {
    for_node("File", None, title, path.to_str(), media_type).into_ingredient()
}

/// Build a static claim thumbnail using manifest-level hints when node metadata
/// does not carry enough detail to select a variant.
#[must_use]
pub(crate) fn claim_for_assertion_with_hints(
    assertion: &ProvenanceAssertion,
    title_hint: Option<&str>,
    media_type_hint: Option<&str>,
) -> StaticThumbnail {
    let node = if assertion.asset.role.as_deref() == Some("document-export") {
        &assertion.root_node
    } else {
        assertion
            .output_node
            .as_ref()
            .or(assertion.executed_node.as_ref())
            .unwrap_or(&assertion.root_node)
    };

    for_node(
        &node.node_type,
        node.programming_language.as_deref(),
        node.title
            .as_deref()
            .or(assertion.asset.title.as_deref())
            .or(title_hint),
        node.content_url.as_deref(),
        node.media_type
            .as_deref()
            .or((!assertion.asset.media_type.is_empty())
                .then_some(assertion.asset.media_type.as_str()))
            .or(media_type_hint),
    )
}

fn for_node(
    node_type: &str,
    programming_language: Option<&str>,
    title: Option<&str>,
    content_url: Option<&str>,
    media_type: Option<&str>,
) -> StaticThumbnail {
    StaticThumbnail {
        media_type: SVG_MEDIA_TYPE,
        bytes: icon_bytes(
            node_type,
            programming_language,
            title,
            content_url,
            media_type,
        ),
    }
}

fn icon_bytes(
    node_type: &str,
    programming_language: Option<&str>,
    title: Option<&str>,
    content_url: Option<&str>,
    media_type: Option<&str>,
) -> &'static [u8] {
    if is_code_node_type(node_type) {
        return code_icon_bytes(programming_language);
    }

    if node_type == "File" {
        return file_icon_bytes(title, content_url, media_type);
    }

    match node_type {
        "Article" => node_icon!("Article.svg"),
        "AudioObject" => node_icon!("AudioObject.svg"),
        "CodeExpression" => node_icon!("CodeExpression.svg"),
        "Datatable" => node_icon!("Datatable.svg"),
        "EnvironmentRecord" => node_icon!("EnvironmentRecord.svg"),
        "File" => node_icon!("File.svg"),
        "ImageObject" => node_icon!("ImageObject.svg"),
        "Table" => node_icon!("Table.svg"),
        "VideoObject" => node_icon!("VideoObject.svg"),
        _ => node_icon!("Default.svg"),
    }
}

fn is_code_node_type(node_type: &str) -> bool {
    matches!(
        node_type,
        "Button"
            | "CallBlock"
            | "CodeChunk"
            | "ForBlock"
            | "Form"
            | "IfBlock"
            | "IfBlockClause"
            | "IncludeBlock"
            | "InstructionBlock"
            | "InstructionInline"
            | "Parameter"
            | "PromptBlock"
    )
}

fn code_icon_bytes(programming_language: Option<&str>) -> &'static [u8] {
    match normalize_token(programming_language).as_deref() {
        Some("r" | "rscript") => node_icon!("CodeChunk-r.svg"),
        Some("python" | "python3" | "py") => node_icon!("CodeChunk-python.svg"),
        Some("sql" | "postgresql" | "mysql" | "sqlite" | "tsql") => node_icon!("CodeChunk-sql.svg"),
        Some("javascript" | "js" | "node" | "nodejs") => node_icon!("CodeChunk-javascript.svg"),
        Some("typescript" | "ts") => node_icon!("CodeChunk-typescript.svg"),
        Some("mermaid" | "mmd") => node_icon!("CodeChunk-mermaid.svg"),
        _ => node_icon!("CodeChunk.svg"),
    }
}

fn file_icon_bytes(
    title: Option<&str>,
    content_url: Option<&str>,
    media_type: Option<&str>,
) -> &'static [u8] {
    let token = title
        .and_then(extension_token)
        .or_else(|| content_url.and_then(extension_token))
        .or_else(|| media_type.and_then(file_token_from_media_type));

    match token.as_deref() {
        Some("ipynb") => node_icon!("File-ipynb.svg"),
        Some("smd") => node_icon!("File-smd.svg"),
        Some("qmd") => node_icon!("File-qmd.svg"),
        Some("myst") => node_icon!("File-myst.svg"),
        Some("docx") => node_icon!("File-docx.svg"),
        Some("latex" | "tex") => node_icon!("File-latex.svg"),
        Some("typst" | "typ") => node_icon!("File-typst.svg"),
        _ => node_icon!("File.svg"),
    }
}

fn extension_token(value: &str) -> Option<String> {
    let without_fragment = value.split_once('#').map_or(value, |(before, ..)| before);
    let path = without_fragment
        .split_once('?')
        .map_or(without_fragment, |(before, ..)| before);

    Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
        .and_then(|extension| normalize_token(Some(extension)))
}

fn file_token_from_media_type(media_type: &str) -> Option<String> {
    let media_type = media_type
        .split_once(';')
        .map_or(media_type, |(media, ..)| media);
    match normalize_token(Some(media_type)).as_deref() {
        Some("text/smd") => Some("smd".to_string()),
        Some("text/qmd") => Some("qmd".to_string()),
        Some("text/myst") => Some("myst".to_string()),
        Some("text/latex" | "text/x-latex" | "application/x-latex" | "application/latex") => {
            Some("latex".to_string())
        }
        Some("text/typst" | "application/x-typst" | "application/typst") => {
            Some("typst".to_string())
        }
        Some(
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
            | "application/docx",
        ) => Some("docx".to_string()),
        Some(
            "application/x-ipynb+json"
            | "application/x-ipython-notebook+json"
            | "application/vnd.jupyter",
        ) => Some("ipynb".to_string()),
        _ => None,
    }
}

fn normalize_token(value: Option<&str>) -> Option<String> {
    let value = value?.trim().to_ascii_lowercase();
    (!value.is_empty()).then_some(value)
}

#[cfg(test)]
mod tests {
    use crate::{AssetSnapshot, DocumentSnapshot, ProvenanceSnapshot};

    use super::*;

    #[test]
    fn article_claim_thumbnail_uses_svg() {
        let assertion = ProvenanceAssertion::from_snapshot(ProvenanceSnapshot {
            asset: AssetSnapshot {
                role: Some("document-export".to_string()),
                ..Default::default()
            },
            root_node: DocumentSnapshot {
                node_type: "Article".to_string(),
                ..Default::default()
            },
            ..Default::default()
        });

        let thumbnail = claim_for_assertion_with_hints(&assertion, None, None);

        assert_eq!(thumbnail.media_type, SVG_MEDIA_TYPE);
        assert!(thumbnail.bytes.starts_with(b"<svg"));
    }

    #[test]
    fn executable_prompt_blocks_reuse_code_icon() {
        let prompt = icon_bytes("PromptBlock", None, None, None, None);
        let code = icon_bytes("CodeChunk", None, None, None, None);

        assert_eq!(prompt, code);
    }

    #[test]
    fn unknown_node_type_has_default_icon() {
        let unknown = icon_bytes("NewNodeType", None, None, None, None);
        let default = icon_bytes("Default", None, None, None, None);

        assert_eq!(unknown, default);
    }

    #[test]
    fn environment_record_has_dedicated_icon() {
        let environment = icon_bytes("EnvironmentRecord", None, None, None, None);
        let default = icon_bytes("Default", None, None, None, None);

        assert_eq!(environment, node_icon!("EnvironmentRecord.svg"));
        assert_ne!(environment, default);
    }

    #[test]
    fn code_chunk_claim_uses_language_variant() {
        let assertion = ProvenanceAssertion::from_snapshot(ProvenanceSnapshot {
            executed_node: Some(DocumentSnapshot {
                node_type: "CodeChunk".to_string(),
                programming_language: Some("python".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        });

        let thumbnail = claim_for_assertion_with_hints(&assertion, None, None);

        assert_eq!(thumbnail.bytes, node_icon!("CodeChunk-python.svg"));
    }

    #[test]
    fn file_claim_uses_title_extension_variant() {
        let assertion = ProvenanceAssertion::from_snapshot(ProvenanceSnapshot::for_asset(
            AssetSnapshot::new("Text", "text/smd", "sha256:abc"),
        ));

        let thumbnail = claim_for_assertion_with_hints(&assertion, Some("article.smd"), None);

        assert_eq!(thumbnail.bytes, node_icon!("File-smd.svg"));
    }
}
