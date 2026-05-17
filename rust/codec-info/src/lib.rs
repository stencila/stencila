mod losses;
mod mapping;
mod messages;
mod poshmap;
mod positions;
mod shifter;

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

pub use losses::*;
pub use mapping::*;
pub use messages::*;
pub use poshmap::*;
pub use positions::*;
pub use shifter::*;

pub use stencila_node_type::ContentType;

/// Information which may be returned when decoding content to a node
#[derive(Default)]
pub struct DecodeInfo {
    /// Any messages generated while decoding
    pub messages: Messages,

    /// The losses when the decoding content to a node
    pub losses: Losses,

    /// The mapping between content locations and the decoded node and its children
    pub mapping: Mapping,
}

impl DecodeInfo {
    /// Create an empty set on decoding information
    pub fn none() -> Self {
        Self {
            messages: Messages::none(),
            losses: Losses::none(),
            mapping: Mapping::none(),
        }
    }
}

/// Information which may be returned when encoding a node to content
#[derive(Default)]

pub struct EncodeInfo {
    /// The losses when encoding the node to content
    pub losses: Losses,

    /// The mapping between content location and the node and its children
    pub mapping: Mapping,

    /// Additional filesystem assets written while encoding.
    ///
    /// Examples include extracted media files written next to Markdown, HTML,
    /// or LaTeX outputs. Dispatchers can use this to perform follow-up actions
    /// such as signing side assets without guessing from directory contents.
    pub assets: Vec<EncodedAsset>,
}

impl EncodeInfo {
    /// Create an empty set on encodings information
    pub fn none() -> Self {
        Self {
            losses: Losses::none(),
            mapping: Mapping::none(),
            assets: Vec::new(),
        }
    }
}

/// A filesystem asset emitted alongside an encoded document.
///
/// Carries the originating node's identity so dispatchers can attach
/// per-node provenance (e.g. Content Credentials) to extracted figures,
/// table images, or other side assets. After Content Credentials signing
/// runs, `signed`, manifest fields, and signing warnings describe what the
/// signing layer attached to this asset.
#[skip_serializing_none]
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EncodedAsset {
    /// Path to the file on disk.
    pub path: PathBuf,

    /// Stringified [`NodeId`](stencila_node_id::NodeId) of the node that
    /// originated this asset, when known.
    pub node_id: Option<String>,

    /// Type of the originating node (e.g. `"CodeChunk"`, `"MathBlock"`,
    /// `"ImageObject"`), when known.
    pub node_type: Option<String>,

    /// Role of the asset relative to its source. Conventional values include
    /// `"computational-output"`, `"math-image"`, `"table-image"`, `"figure"`,
    /// `"document"`, and `"sidecar"`.
    pub role: Option<String>,

    /// Human-readable title for this asset, when known.
    ///
    /// For generated media this is usually derived from the nearest labelled
    /// Figure or CodeChunk caption, so downstream manifest metadata can show a
    /// useful title instead of falling back to the file name.
    pub title: Option<String>,

    /// Human-readable description for this asset, when known.
    ///
    /// For generated media this is usually the full nearest labelled Figure or
    /// CodeChunk caption, while `title` may be shortened for display.
    pub description: Option<String>,

    /// Whether Content Credentials were attached to this asset.
    ///
    /// `false` for plain side assets, originating sidecar files, or assets
    /// emitted with credentials disabled. `true` when the signing layer
    /// produced a manifest for this asset.
    #[serde(default, skip_serializing_if = "is_false")]
    pub signed: bool,

    /// Where the C2PA manifest for this asset was written.
    ///
    /// Values are `"embedded"` when the manifest sits inside the asset
    /// bytes (PNG, JPEG, WebP, SVG) and `"sidecar"` when it was detached
    /// to a `.c2pa` file. Absent when this asset is not signed.
    pub manifest_kind: Option<String>,

    /// Active C2PA manifest identifier, when it could be read after signing.
    pub manifest_id: Option<String>,

    /// Path to this asset's `.c2pa` sidecar manifest, when one was written.
    ///
    /// Distinct from sidecar role entries, which represent the sidecar
    /// file itself as its own asset row.
    pub sidecar_path: Option<PathBuf>,

    /// Content Credentials projection profile used when signing this asset.
    pub credential_profile: Option<String>,

    /// User-facing C2PA manifest summary for this signed asset.
    ///
    /// This is a compact projection of standard C2PA fields such as actions,
    /// ingredients, signature issuer/time, and claim generator. It is intended
    /// for display surfaces, not as a verification substitute.
    pub c2pa: Option<serde_json::Value>,

    /// Non-fatal warnings from the signing layer.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub signing_warnings: Vec<String>,
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_false(value: &bool) -> bool {
    !*value
}

impl EncodedAsset {
    /// Build an asset record carrying just a path.
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            ..Default::default()
        }
    }

    /// Build a sidecar asset record.
    pub fn sidecar(path: PathBuf) -> Self {
        Self {
            path,
            role: Some("sidecar".to_string()),
            ..Default::default()
        }
    }
}

impl From<PathBuf> for EncodedAsset {
    fn from(path: PathBuf) -> Self {
        Self::new(path)
    }
}
