// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::image_object::ImageObject;
use super::primitive::Primitive;
use super::property_value_or_string::PropertyValueOrString;
use super::string::String;
use super::text::Text;

/// A property-value pair.
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, ReadNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "PropertyValue")]
pub struct PropertyValue {
    /// The type of this item.
    pub r#type: MustBe!("PropertyValue"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// A commonly used identifier for the characteristic represented by the property.
    pub property_id: Option<String>,

    /// The value of the property.
    pub value: Primitive,

    /// Non-core optional fields
    #[serde(flatten)]
    #[html(flatten)]
    #[jats(flatten)]
    #[markdown(flatten)]
    pub options: Box<PropertyValueOptions>,
}

#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, ReadNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct PropertyValueOptions {
    /// Alternate names (aliases) for the item.
    #[strip(metadata)]
    pub alternate_names: Option<Vec<String>>,

    /// A description of the item.
    #[strip(metadata)]
    pub description: Option<Text>,

    /// Any kind of identifier for any kind of Thing.
    #[strip(metadata)]
    pub identifiers: Option<Vec<PropertyValueOrString>>,

    /// Images of the item.
    #[strip(metadata)]
    pub images: Option<Vec<ImageObject>>,

    /// The name of the item.
    #[strip(metadata)]
    pub name: Option<String>,

    /// The URL of the item.
    #[strip(metadata)]
    pub url: Option<String>,
}

impl PropertyValue {
    pub fn new(value: Primitive) -> Self {
        Self {
            value,
            ..Default::default()
        }
    }
}
