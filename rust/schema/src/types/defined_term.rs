//! Generated file, do not edit

use crate::prelude::*;

use super::block::Block;
use super::image_object_or_string::ImageObjectOrString;
use super::property_value_or_string::PropertyValueOrString;
use super::string::String;

/// A word, name, acronym, phrase, etc. with a formal definition.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct DefinedTerm {
    /// The type of this item
    r#type: MustBe!("DefinedTerm"),

    /// The identifier for this item
    id: String,

    /// The name of the item.
    name: String,

    /// Non-core optional fields
    #[serde(flatten)]
    options: Box<DefinedTermOptions>,
}

#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct DefinedTermOptions {
    /// Alternate names (aliases) for the item.
    alternate_names: Option<Vec<String>>,

    /// A description of the item.
    description: Option<Vec<Block>>,

    /// Any kind of identifier for any kind of Thing.
    identifiers: Option<Vec<PropertyValueOrString>>,

    /// Images of the item.
    images: Option<Vec<ImageObjectOrString>>,

    /// The URL of the item.
    url: Option<String>,

    /// A code that identifies this DefinedTerm within a DefinedTermSet
    term_code: Option<String>,
}
