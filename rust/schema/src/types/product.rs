// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::brand::Brand;
use super::image_object::ImageObject;
use super::property_value_or_string::PropertyValueOrString;
use super::string::String;

/// Any offered product or service. For example, a pair of shoes; a haircut; or an episode of a TV show streamed online.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "Product")]
#[jats(elem = "product")]
pub struct Product {
    /// The type of this item.
    pub r#type: MustBe!("Product"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// Non-core optional fields
    #[serde(flatten)]
    #[html(flatten)]
    #[jats(flatten)]
    pub options: Box<ProductOptions>,

    /// A unique identifier for a node within a document
    
    #[serde(skip)]
    pub uid: NodeUid
}

#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct ProductOptions {
    /// Alternate names (aliases) for the item.
    #[serde(alias = "alternate-names", alias = "alternate_names", alias = "alternateName", alias = "alternate-name", alias = "alternate_name")]
    #[serde(default, deserialize_with = "option_csv_or_array")]
    #[strip(metadata)]
    pub alternate_names: Option<Vec<String>>,

    /// A description of the item.
    #[strip(metadata)]
    #[patch(format = "md", format = "myst")]
    pub description: Option<String>,

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

    /// The name of the item.
    #[strip(metadata)]
    pub name: Option<String>,

    /// The URL of the item.
    #[strip(metadata)]
    pub url: Option<String>,

    /// Brands that the product is labelled with.
    #[serde(alias = "brand")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    pub brands: Option<Vec<Brand>>,

    /// The logo of the product.
    pub logo: Option<ImageObject>,

    /// Product identification code.
    #[serde(alias = "product-id", alias = "product_id")]
    pub product_id: Option<String>,
}

impl Product {
    const NICK: [u8; 3] = [112, 114, 111];
    
    pub fn node_type(&self) -> NodeType {
        NodeType::Product
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}
