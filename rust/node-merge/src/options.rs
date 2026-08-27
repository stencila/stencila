//! Options for merging two nodes
//!
//! Deliberately free of thresholds and weights. Which occurrences correspond is
//! decided by `stencila-node-compare`, and nothing here can change it: these options
//! decide only how an already-decided difference is *expressed* in the merged
//! document.

use serde::{Deserialize, Serialize};

use stencila_node_compare::CompareOptions;
use stencila_schema::{Author, SuggestionStatus};

/// Options for merging two nodes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct MergeOptions {
    /// Options for the comparison, when the merge derives one itself
    ///
    /// Ignored by [`crate::merge_comparison`], which is given a comparison that has
    /// already been derived.
    pub compare: CompareOptions,

    /// How to handle adjacent edits within a container
    ///
    /// With this on, a run of deletions immediately followed by a run of insertions
    /// becomes a single replacement, which is both what a reader expects of a
    /// rewritten paragraph and what makes accepting or rejecting it exact.
    pub coalesce: EditCoalescing,

    /// How a change to a non-content property is expressed
    pub metadata_changes: MetadataChanges,

    /// Whether to emit comments
    ///
    /// With comments off the merged document contains only suggestions, so rejecting
    /// every suggestion restores the left document exactly. With them on, rejection
    /// restores it only up to the comments and generated target identifiers.
    pub comments: CommentMode,

    /// The status stamped on every suggestion
    ///
    /// `None` leaves the suggestions unresolved, which is what a proposal is.
    pub suggestion_status: Option<SuggestionStatus>,

    /// The authors stamped on every suggestion and comment
    pub authors: Option<Vec<Author>>,

    /// The prefix for generated suggestion identifiers
    pub id_prefix: String,
}

impl Default for MergeOptions {
    fn default() -> Self {
        Self {
            compare: CompareOptions::default(),
            coalesce: EditCoalescing::default(),
            metadata_changes: MetadataChanges::default(),
            comments: CommentMode::default(),
            suggestion_status: None,
            authors: None,
            id_prefix: "mg".to_string(),
        }
    }
}

/// How adjacent edits within one container are represented
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EditCoalescing {
    /// Combine adjacent deletions and insertions into fewer suggestions
    #[default]
    Coalesce,

    /// Keep each planned edit as its own suggestion
    KeepSeparate,
}

/// Whether merge comments are included in the output document
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CommentMode {
    /// Include comments describing changes suggestions cannot express directly
    #[default]
    Include,

    /// Omit merge-generated comments
    Omit,
}

/// How a change to a non-content property is expressed
///
/// A change to a property such as `Heading.level` or `Link.target` is not a
/// substitution of content, so there is nothing for a suggestion to hold. It can still
/// be *described*, and it can be carried by replacing the whole node that owns it,
/// which is the only way accepting the merge reproduces it.
///
/// Carrying it that way costs more than it looks. When the owning node also has content
/// edits, the whole-node replacement subsumes those finer edits so that accepting and
/// rejecting remain exact, but the replacement can be visually noisy. Describing it is
/// the default for that reason; carrying it is for callers who need accepting the merge
/// to be exact and will take the noise in exchange.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MetadataChanges {
    /// Describe the change in a comment, and leave the document otherwise unchanged
    #[default]
    CommentOnly,

    /// Describe the change in a comment, and also replace the whole node that owns
    /// the property, where that node sits in a block or inline container
    CommentAndSuggest,
}
