mod losses;
mod mapping;
mod messages;
mod poshmap;
mod positions;
mod shifter;

use std::path::PathBuf;

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
/// table images, or other side assets.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
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
