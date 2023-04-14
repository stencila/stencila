// Generated file. Do not edit; see `schema-gen` crate.

use crate::prelude::*;

use super::block::Block;
use super::image_object_or_string::ImageObjectOrString;
use super::primitive::Primitive;
use super::property_value_or_string::PropertyValueOrString;
use super::string::String;

/// A property-value pair.
#[skip_serializing_none]
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Read, Write, ToHtml)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct PropertyValue {
    /// The type of this item
    pub r#type: MustBe!("PropertyValue"),

    /// The identifier for this item
    pub id: Option<String>,

    /// A commonly used identifier for the characteristic represented by the property.
    pub property_id: Option<String>,

    /// The value of the property.
    pub value: Primitive,

    /// Non-core optional fields
    #[serde(flatten)]
    pub options: Box<PropertyValueOptions>,
}

#[skip_serializing_none]
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Read, Write, ToHtml)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct PropertyValueOptions {
    /// Alternate names (aliases) for the item.
    pub alternate_names: Option<Vec<String>>,

    /// A description of the item.
    pub description: Option<Vec<Block>>,

    /// Any kind of identifier for any kind of Thing.
    pub identifiers: Option<Vec<PropertyValueOrString>>,

    /// Images of the item.
    pub images: Option<Vec<ImageObjectOrString>>,

    /// The name of the item.
    pub name: Option<String>,

    /// The URL of the item.
    pub url: Option<String>,
}

impl PropertyValue {
    #[rustfmt::skip]
    pub fn new(value: Primitive) -> Self {
        Self {
            value,
            ..Default::default()
        }
    }
}
