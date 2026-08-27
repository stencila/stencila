//! An account of what a merge produced, and what it could not
//!
//! Not every difference between two documents is a substitution of content, and the
//! ones that are not cannot be carried by a suggestion. Reordering and moves pair the
//! node on both sides, so there is nothing to insert or delete; a right-only list item
//! or table row has no slot that accepts a suggestion; a change to a property under
//! `authors` has no block or inline ancestor to attach one to.
//!
//! Those are reported rather than swallowed, because whether the list is empty is
//! exactly the question "does accepting every suggestion in this merge reproduce the
//! right document?". A caller that needs that guarantee can check it instead of
//! assuming it.

use serde::{Deserialize, Serialize};

use stencila_node_compare::NodeRef;
use stencila_node_type::NodeProperty;

/// What a merge produced, and what it could not express
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MergeReport {
    /// The number of insertion suggestions
    pub inserts: usize,

    /// The number of deletion suggestions
    pub deletes: usize,

    /// The number of replacement suggestions
    pub replaces: usize,

    /// The number of comments
    pub comments: usize,

    /// The differences and one-sided subtrees that no suggestion can express
    ///
    /// Empty exactly when accepting every suggestion reproduces the right document,
    /// and rejecting every suggestion reproduces the left one.
    pub unrepresentable: Vec<Unrepresentable>,
}

impl MergeReport {
    /// The total number of suggestions
    pub fn suggestions(&self) -> usize {
        self.inserts + self.deletes + self.replaces
    }

    /// Whether every difference was expressed as a suggestion
    pub fn is_complete(&self) -> bool {
        self.unrepresentable.is_empty()
    }
}

/// A difference that no suggestion can express
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Unrepresentable {
    /// Where it occurs on the left, when it occurs there
    pub left: Option<NodeRef>,

    /// Where it occurs on the right, when it occurs there
    pub right: Option<NodeRef>,

    /// Why no suggestion could carry it
    pub reason: UnrepresentableReason,
}

/// Why a difference could not be expressed as a suggestion
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum UnrepresentableReason {
    /// The containing property holds neither blocks nor inlines
    ///
    /// `SuggestionBlock` is a `Block` and `SuggestionInline` is an `Inline`, so a
    /// property whose slot is a `ListItem`, `TableRow`, `Author` or `Reference` has
    /// nowhere to put one.
    NotContentContainer { slot: String },

    /// The containing property is absent on the left, so it has no container to
    /// insert into
    ContainerAbsentOnLeft { property: NodeProperty },

    /// The changed property is metadata, and no ancestor sits in a block or inline
    /// container to be replaced whole
    NoContentAncestor { property: Option<NodeProperty> },

    /// The occurrence moved between parents
    ///
    /// It is paired on both sides, so there is no one-sided content to insert or
    /// delete. Representing a move as a deletion plus an insertion would duplicate
    /// the content.
    Moved,

    /// The occurrence changed its position among its siblings
    ///
    /// Paired on both sides, for the same reason as a move.
    Reordered,
}
