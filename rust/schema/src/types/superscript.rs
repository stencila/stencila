//! Generated file, do not edit

use crate::prelude::*;

use super::inline::Inline;
use super::string::String;

/// Superscripted content.
#[skip_serializing_none]
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct Superscript {
    /// The type of this item
    pub r#type: MustBe!("Superscript"),

    /// The identifier for this item
    pub id: Option<String>,

    /// The content that is marked.
    pub content: Vec<Inline>,
}

impl Superscript {
    pub fn new(content: Vec<Inline>) -> Self {
        Self{
            content,
            ..Default::default()
        }
    }
}

