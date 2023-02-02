//! Generated file, do not edit

use crate::prelude::*;

use super::array_validator::ArrayValidator;
use super::block::Block;
use super::image_object_or_string::ImageObjectOrString;
use super::primitive::Primitive;
use super::property_value_or_string::PropertyValueOrString;
use super::string::String;

/// A column of data within a Datatable.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct DatatableColumn {
    /// The type of this item
    r#type: MustBe!("DatatableColumn"),

    /// The identifier for this item
    id: String,

    /// The name of the item.
    name: String,

    /// The data values of the column.
    values: Vec<Primitive>,

    /// The validator to use to validate data in the column.
    validator: Option<ArrayValidator>,

    /// Non-core optional fields
    #[serde(flatten)]
    options: Box<DatatableColumnOptions>,
}

#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct DatatableColumnOptions {
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
}
