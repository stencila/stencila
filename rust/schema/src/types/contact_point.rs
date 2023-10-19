// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::image_object::ImageObject;
use super::property_value_or_string::PropertyValueOrString;
use super::string::String;
use super::text::Text;

/// A contact point, usually within an organization.
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, ReadNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct ContactPoint {
    /// The type of this item.
    pub r#type: MustBe!("ContactPoint"),

    /// The identifier for this item.
    #[strip(id)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// Email address for correspondence.
    pub emails: Option<Vec<String>>,

    /// Telephone numbers for the contact point.
    pub telephone_numbers: Option<Vec<String>>,

    /// Non-core optional fields
    #[serde(flatten)]
    #[html(flatten)]
    #[jats(flatten)]
    #[markdown(flatten)]
    pub options: Box<ContactPointOptions>,
}

#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, ReadNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct ContactPointOptions {
    /// Alternate names (aliases) for the item.
    pub alternate_names: Option<Vec<String>>,

    /// A description of the item.
    pub description: Option<Text>,

    /// Any kind of identifier for any kind of Thing.
    pub identifiers: Option<Vec<PropertyValueOrString>>,

    /// Images of the item.
    pub images: Option<Vec<ImageObject>>,

    /// The name of the item.
    pub name: Option<String>,

    /// The URL of the item.
    pub url: Option<String>,

    /// Languages (human not programming) in which it is possible to communicate
    /// with the organization/department etc.
    pub available_languages: Option<Vec<String>>,
}

impl ContactPoint {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}
