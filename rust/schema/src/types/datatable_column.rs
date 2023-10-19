// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::array_validator::ArrayValidator;
use super::image_object::ImageObject;
use super::primitive::Primitive;
use super::property_value_or_string::PropertyValueOrString;
use super::string::String;
use super::text::Text;

/// A column of data within a `Datatable`.
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, ReadNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct DatatableColumn {
    /// The type of this item.
    pub r#type: MustBe!("DatatableColumn"),

    /// The identifier for this item.
    #[strip(id)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The name of the item.
    pub name: String,

    /// The data values of the column.
    pub values: Vec<Primitive>,

    /// The validator to use to validate data in the column.
    pub validator: Option<ArrayValidator>,

    /// Non-core optional fields
    #[serde(flatten)]
    #[html(flatten)]
    #[jats(flatten)]
    #[markdown(flatten)]
    pub options: Box<DatatableColumnOptions>,
}

#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, ReadNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct DatatableColumnOptions {
    /// Alternate names (aliases) for the item.
    pub alternate_names: Option<Vec<String>>,

    /// A description of the item.
    pub description: Option<Text>,

    /// Any kind of identifier for any kind of Thing.
    pub identifiers: Option<Vec<PropertyValueOrString>>,

    /// Images of the item.
    pub images: Option<Vec<ImageObject>>,

    /// The URL of the item.
    pub url: Option<String>,
}

impl DatatableColumn {
    pub fn new(name: String, values: Vec<Primitive>) -> Self {
        Self {
            name,
            values,
            ..Default::default()
        }
    }
}
