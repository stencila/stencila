//! Search index entry data structures

use serde::{Deserialize, Serialize};

/// Structural weights for different node types
///
/// Higher values indicate more important content that should rank higher.
pub mod weights {
    /// Article title
    pub const TITLE: u8 = 10;
    /// Heading level 1
    pub const HEADING_1: u8 = 9;
    /// Heading level 2
    pub const HEADING_2: u8 = 8;
    /// Heading level 3
    pub const HEADING_3: u8 = 7;
    /// Heading level 4
    pub const HEADING_4: u8 = 6;
    /// Heading level 5
    pub const HEADING_5: u8 = 5;
    /// Heading level 6
    pub const HEADING_6: u8 = 4;
    /// Figure/table caption
    pub const CAPTION: u8 = 6;
    /// Datatable column names/description
    pub const DATATABLE: u8 = 5;
    /// Regular paragraph text
    pub const PARAGRAPH: u8 = 2;
    /// Code blocks
    pub const CODE: u8 = 1;
}

/// A single indexed entry in the search index
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchEntry {
    /// The node ID (e.g., "hed_ABC123")
    pub node_id: String,

    /// The type of node (e.g., "Heading", "Paragraph", "Datatable")
    pub node_type: String,

    /// The route where this node appears (e.g., "/docs/guide/")
    pub route: String,

    /// The indexed text content
    pub text: String,

    /// Structural weight for ranking (higher = more important)
    pub weight: u8,

    /// Depth in document (0=root article, 1=top-level section, etc.)
    pub depth: u8,

    /// For datatables: additional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<DatatableMetadata>,

    /// Pre-computed token trigrams for fuzzy matching (internal use only)
    /// Populated during indexing, converted to `tokens` before serialization
    #[serde(skip)]
    pub token_trigrams: Option<Vec<TokenTrigrams>>,

    /// Compact token references: [[defIndex, start, end], ...]
    /// Indexes into the shard's token_defs array
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens: Option<Vec<TokenRef>>,
}

/// Token with pre-computed trigrams for fuzzy matching
///
/// Each token from the entry text is stored with its trigrams and
/// position in the original text for highlighting.
/// Used internally during indexing; not serialized directly to shard files.
#[derive(Debug, Clone)]
pub struct TokenTrigrams {
    /// Normalized token (lowercased, diacritics folded) for trigram matching
    pub token: String,

    /// Pre-computed character trigrams (3-grams)
    pub trigrams: Vec<String>,

    /// Start position in original entry.text (UTF-16 code unit offset)
    pub start: u32,

    /// End position in original entry.text (UTF-16 code unit offset)
    pub end: u32,
}

impl TokenTrigrams {
    /// Create a new TokenTrigrams
    pub fn new(token: String, trigrams: Vec<String>, start: u32, end: u32) -> Self {
        Self {
            token,
            trigrams,
            start,
            end,
        }
    }
}

/// Token definition for fuzzy matching (shard-level, deduplicated)
///
/// Contains the token string and its trigrams, without position information.
/// Stored once per unique token in a shard file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenDef {
    /// Normalized token (lowercased, diacritics folded)
    pub token: String,

    /// Pre-computed character trigrams (3-grams)
    pub trigrams: Vec<String>,
}

impl TokenDef {
    /// Create a new TokenDef
    pub fn new(token: String, trigrams: Vec<String>) -> Self {
        Self { token, trigrams }
    }
}

/// Compact token reference: [tokenDefIndex, start, end]
///
/// References a TokenDef in the shard's token_defs array, with position
/// information for highlighting in this specific entry.
pub type TokenRef = [u32; 3];

/// Additional metadata for datatable nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatatableMetadata {
    /// Column names
    pub columns: Vec<String>,

    /// Description if available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Row count hint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub row_count: Option<usize>,
}

impl SearchEntry {
    /// Create a new search entry
    pub fn new(
        node_id: String,
        node_type: impl Into<String>,
        route: String,
        text: String,
        weight: u8,
        depth: u8,
    ) -> Self {
        Self {
            node_id,
            node_type: node_type.into(),
            route,
            text,
            weight,
            depth,
            metadata: None,
            token_trigrams: None,
            tokens: None,
        }
    }

    /// Create a new search entry with datatable metadata
    pub fn with_metadata(mut self, metadata: DatatableMetadata) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Set token trigrams for fuzzy search
    pub fn with_token_trigrams(mut self, token_trigrams: Vec<TokenTrigrams>) -> Self {
        self.token_trigrams = Some(token_trigrams);
        self
    }
}
