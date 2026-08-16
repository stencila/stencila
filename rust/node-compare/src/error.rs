//! Typed errors
//!
//! Comparison is all-or-nothing: a projection failure, resource-limit exhaustion, or
//! an invariant violation returns an error and no partial artifact.

use std::fmt::{self, Display};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use stencila_node_path::NodePath;
use stencila_node_type::NodeType;

/// Which of the two caller-selected inputs an error refers to
///
/// Neither side is presumed correct: they are simply the two snapshots the caller
/// selected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Side {
    Left,
    Right,
}

impl Side {
    /// The other side
    pub fn invert(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

impl Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Left => "left",
            Self::Right => "right",
        })
    }
}

/// An error from aligning or comparing two nodes
#[derive(Debug, Error)]
pub enum CompareError {
    /// A value could not be projected because schema introspection reported
    /// something the projection does not know how to represent
    #[error("Unable to project the {side} node at path `{path}`: {message}")]
    Projection {
        side: Side,
        path: NodePath,
        message: String,
    },

    /// A scalar value could not be represented in the canonical scalar model
    #[error("Unsupported scalar value in the {side} node at path `{path}`: {message}")]
    Scalar {
        side: Side,
        path: NodePath,
        message: String,
    },

    /// A path recorded in an artifact does not resolve to the recorded node type
    #[error("Path `{path}` in the {side} node does not resolve to a `{expected}` occurrence")]
    PathResolution {
        side: Side,
        path: NodePath,
        expected: NodeType,
    },

    /// A node is nested more deeply than the projection can represent
    ///
    /// An operational limit, not a semantic one: it exists so that a pathologically
    /// deep input returns an error rather than exhausting the stack, which is the one
    /// outcome the all-or-nothing rule cannot express.
    #[error(
        "The {side} node at path `{path}` is nested {depth} levels deep, but the limit is {allowed}"
    )]
    DepthExceeded {
        side: Side,
        path: NodePath,
        depth: usize,
        allowed: usize,
    },

    /// The alignment of a repeated property needed more candidate cells than the
    /// budget allows
    #[error(
        "Aligning `{left_path}` with `{right_path}` requires {required} candidate cells but the budget is {allowed}"
    )]
    BudgetExhausted {
        left_path: NodePath,
        right_path: NodePath,
        required: usize,
        allowed: usize,
    },

    /// A sealed alignment or value policy returned a result outside its contract
    #[error("Invalid internal policy output: {message}")]
    InvalidPolicy { message: String },

    /// An alignment does not cover every projected occurrence
    #[error("The {side} alignment covers {covered} occurrences but the projection has {projected}")]
    Completeness {
        side: Side,
        covered: usize,
        projected: usize,
    },

    /// An occurrence appears more than once in an alignment
    #[error("The {side} path `{path}` occurs more than once in the alignment")]
    Uniqueness { side: Side, path: NodePath },

    /// A result failed the swap-symmetry invariant
    #[error("The comparison is not swap-symmetric: {message}")]
    Symmetry { message: String },

    /// A serialized artifact uses a format version this crate does not support
    #[error("Unsupported {artifact} format version `{version}`")]
    UnsupportedVersion {
        artifact: &'static str,
        version: String,
    },

    /// An internal invariant without a more specific public category was violated
    #[error("Invariant violated: {message}")]
    Invariant { message: String },
}

/// A result from aligning or comparing two nodes
pub type CompareResult<T> = Result<T, CompareError>;
