//! Generated file, do not edit

use crate::prelude::*;

use super::array_validator::ArrayValidator;
use super::block::Block;
use super::image_object_or_string::ImageObjectOrString;
use super::primitive::Primitive;
use super::property_value_or_string::PropertyValueOrString;
use super::string::String;

/// A column of data within a Datatable.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct DatatableColumn {
    /// The type of this item
    #[autosurgeon(with = "autosurgeon_must_be")]
    pub r#type: MustBe!("DatatableColumn"),

    /// The identifier for this item
    #[key]
    pub id: Option<String>,

    /// The name of the item.
    pub name: String,

    /// The data values of the column.
    pub values: Vec<Primitive>,

    /// The validator to use to validate data in the column.
    pub validator: Option<ArrayValidator>,

    /// Non-core optional fields
    #[serde(flatten)]
    pub options: Box<DatatableColumnOptions>,
}

#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct DatatableColumnOptions {
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
}
