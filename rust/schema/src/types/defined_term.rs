//! Generated file, do not edit

use crate::prelude::*;

use super::block::Block;
use super::image_object_or_string::ImageObjectOrString;
use super::property_value_or_string::PropertyValueOrString;
use super::string::String;

/// A word, name, acronym, phrase, etc. with a formal definition.
#[skip_serializing_none]
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct DefinedTerm {
    /// The type of this item
    pub r#type: MustBe!("DefinedTerm"),

    /// The identifier for this item
    pub id: Option<String>,

    /// The name of the item.
    pub name: String,

    /// Non-core optional fields
    #[serde(flatten)]
    pub options: Box<DefinedTermOptions>,
}

#[skip_serializing_none]
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct DefinedTermOptions {
    /// Alternate names (aliases) for the item.
    pub alternate_names: Option<Vec<String>>,

    /// A description of the item.
    pub description: Option<Vec<Block>>,

    /// Any kind of identifier for any kind of Thing.
    pub identifiers: Option<Vec<PropertyValueOrString>>,

    /// Images of the item.
    pub images: Option<Vec<ImageObjectOrString>>,

    /// The URL of the item.
    pub url: Option<String>,

    /// A code that identifies this DefinedTerm within a DefinedTermSet
    pub term_code: Option<String>,
}

impl DefinedTerm {
    #[rustfmt::skip]
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }
}
