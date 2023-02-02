//! Generated file, do not edit

use crate::prelude::*;

use super::block::Block;
use super::call_argument::CallArgument;
use super::string::String;

/// Call another document, optionally with arguments, and include its executed content.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct Call {
    /// The external source of the content, a file path or URL.
    source: String,

    /// Media type of the source content.
    media_type: Option<String>,

    /// A query to select a subset of content from the source
    select: Option<String>,

    /// The structured content decoded from the source.
    content: Option<Vec<Block>>,

    /// The value of the source document's parameters to call it with
    arguments: Vec<CallArgument>,
}
