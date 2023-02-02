//! Generated file, do not edit

use crate::prelude::*;

use super::block::Block;
use super::string::String;

/// Styled block content
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct Division {
    /// A Cascading Style Sheet (CSS) transpiled from the output of evaluating the `text` property.
    css: Option<String>,

    /// A list of class names associated with the document node
    classes: Option<Vec<String>>,

    /// The content within the division
    content: Vec<Block>,
}
