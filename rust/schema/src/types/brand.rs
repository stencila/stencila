// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::image_object::ImageObject;
use super::property_value_or_string::PropertyValueOrString;
use super::string::String;
use super::text::Text;

/// A brand used by an organization or person for labeling a product, product group, or similar.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "Brand")]
pub struct Brand {
    /// The type of this item.
    pub r#type: MustBe!("Brand"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The name of the item.
    #[strip(metadata)]
    pub name: String,

    /// Non-core optional fields
    #[serde(flatten)]
    #[html(flatten)]
    #[jats(flatten)]
    pub options: Box<BrandOptions>,

    /// A unique identifier for a node within a document
    
    #[serde(skip)]
    pub uid: NodeUid
}

#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct BrandOptions {
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

    /// A logo associated with the brand.
    pub logo: Option<ImageObject>,

    /// Reviews of the brand.
    #[serde(alias = "review")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    pub reviews: Option<Vec<String>>,
}

impl Brand {
    const NICK: [u8; 3] = [98, 114, 97];
    
    pub fn node_type(&self) -> NodeType {
        NodeType::Brand
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }
}
