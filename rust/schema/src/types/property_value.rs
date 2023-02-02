//! Generated file, do not edit

use crate::prelude::*;

use super::block::Block;
use super::image_object_or_string::ImageObjectOrString;
use super::primitive::Primitive;
use super::property_value_or_string::PropertyValueOrString;
use super::string::String;

/// A property-value pair.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(crate = "common::serde")]
pub struct PropertyValue {
    /// The type of this item
    #[autosurgeon(with = "autosurgeon_must_be")]
    r#type: MustBe!("PropertyValue"),

    /// The identifier for this item
    id: Option<String>,

    /// A commonly used identifier for the characteristic represented by the property.
    property_id: Option<String>,

    /// The value of the property.
    value: Primitive,

    /// Non-core optional fields
    #[serde(flatten)]
    options: Box<PropertyValueOptions>,
}

#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(crate = "common::serde")]
pub struct PropertyValueOptions {
    /// Alternate names (aliases) for the item.
    alternate_names: Option<Vec<String>>,

    /// A description of the item.
    description: Option<Vec<Block>>,

    /// Any kind of identifier for any kind of Thing.
    identifiers: Option<Vec<PropertyValueOrString>>,

    /// Images of the item.
    images: Option<Vec<ImageObjectOrString>>,

    /// The name of the item.
    name: Option<String>,

    /// The URL of the item.
    url: Option<String>,
}
