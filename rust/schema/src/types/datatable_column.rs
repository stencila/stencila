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
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "DatatableColumn")]
pub struct DatatableColumn {
    /// The type of this item.
    pub r#type: MustBe!("DatatableColumn"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The name of the item.
    #[strip(metadata)]
    pub name: String,

    /// The data values of the column.
    #[serde(alias = "value")]
    #[serde(deserialize_with = "one_or_many")]
    pub values: Vec<Primitive>,

    /// The validator to use to validate data in the column.
    pub validator: Option<ArrayValidator>,

    /// Non-core optional fields
    #[serde(flatten)]
    #[html(flatten)]
    #[jats(flatten)]
    pub options: Box<DatatableColumnOptions>,

    /// A universally unique identifier for this node
    
    #[serde(skip)]
    pub uuid: NodeUuid
}

#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct DatatableColumnOptions {
    /// Alternate names (aliases) for the item.
    #[serde(alias = "alternate-names", alias = "alternate_names", alias = "alternateName", alias = "alternate-name", alias = "alternate_name")]
    #[serde(default, deserialize_with = "option_csv_or_array")]
    #[strip(metadata)]
    pub alternate_names: Option<Vec<String>>,

    /// A description of the item.
    #[strip(metadata)]
    pub description: Option<Text>,

    /// Any kind of identifier for any kind of Thing.
    #[serde(alias = "identifier")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(metadata)]
    pub identifiers: Option<Vec<PropertyValueOrString>>,

    /// Images of the item.
    #[serde(alias = "image")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(metadata)]
    pub images: Option<Vec<ImageObject>>,

    /// The URL of the item.
    #[strip(metadata)]
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

impl Entity for DatatableColumn {
    const NICK: &'static str = "dat";

    fn node_type(&self) -> NodeType {
        NodeType::DatatableColumn
    }

    fn node_id(&self) -> NodeId {
        NodeId::new(Self::NICK, &self.uuid)
    }
}
