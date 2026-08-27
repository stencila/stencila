//! Errors from merging two nodes
//!
//! Merging is all-or-nothing: an unresolvable path or a failed edit returns an error
//! and no partial document. A difference that the schema cannot express as a
//! suggestion is deliberately *not* an error: it is reported in
//! [`crate::MergeReport::unrepresentable`], because "this change has no content
//! substitution" is an ordinary outcome of merging, not a failure to merge.

use stencila_node_compare::{CompareError, Side};
use stencila_node_path::NodePath;
use stencila_node_type::NodeType;

/// An error from merging two nodes
#[derive(Debug, thiserror::Error)]
pub enum MergeError {
    /// The comparison the merge was given could not be derived, or does not belong
    /// to the two snapshots it was given with
    #[error(transparent)]
    Compare(#[from] CompareError),

    /// A path in the comparison does not resolve in the document it addresses
    #[error("The {side} node has no value at path `{path}`")]
    PathResolution { side: Side, path: NodePath },

    /// A path resolves, but not to the kind of value the edit needs
    ///
    /// Names the offending occurrence, not just the container it was found in: the
    /// container is usually blameless, and knowing which node type could not be read
    /// back is the whole of the diagnosis.
    #[error(
        "Item {index} of the {side} container `{path}` is a `{actual}`, which cannot be read back as a `{expected}`"
    )]
    UnexpectedValue {
        side: Side,
        path: NodePath,
        index: usize,
        actual: NodeType,
        expected: &'static str,
    },

    /// An edit could not be applied to the working document
    #[error("Unable to apply an edit at path `{path}`: {message}")]
    Apply { path: NodePath, message: String },

    /// The root of the left document is not a type the merge can attach comments to
    #[error("Cannot merge into a `{node_type}` root; an `Article` is required")]
    UnsupportedRoot { node_type: NodeType },
}

/// The result of merging two nodes
pub type MergeResult<T> = Result<T, MergeError>;
