// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::cord::Cord;
use super::image_object::ImageObject;
use super::person_or_organization::PersonOrOrganization;
use super::property_value_or_string::PropertyValueOrString;
use super::string::String;
use super::thing::Thing;

/// A grant, typically financial or otherwise quantifiable, of resources.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "Grant")]
pub struct Grant {
    /// The type of this item.
    pub r#type: MustBe!("Grant"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// Non-core optional fields
    #[serde(flatten)]
    #[html(flatten)]
    #[jats(flatten)]
    pub options: Box<GrantOptions>,

    /// A unique identifier for a node within a document
    
    #[serde(skip)]
    pub uid: NodeUid
}

#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct GrantOptions {
    /// Alternate names (aliases) for the item.
    #[serde(alias = "alternate-names", alias = "alternate_names", alias = "alternateName", alias = "alternate-name", alias = "alternate_name")]
    #[serde(default, deserialize_with = "option_csv_or_array")]
    #[strip(metadata)]
    pub alternate_names: Option<Vec<String>>,

    /// A description of the item.
    #[strip(metadata)]
    pub description: Option<Cord>,

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

    /// Indicates an item funded or sponsored through a Grant.
    #[serde(alias = "funded-items", alias = "funded_items", alias = "fundedItem", alias = "funded-item", alias = "funded_item")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    pub funded_items: Option<Vec<Thing>>,

    /// A person or organization that supports a thing through a pledge, promise, or financial contribution.
    #[serde(alias = "sponsor")]
    #[serde(default, deserialize_with = "option_one_or_many_string_or_object")]
    pub sponsors: Option<Vec<PersonOrOrganization>>,
}

impl Grant {
    const NICK: [u8; 3] = [103, 114, 97];
    
    pub fn node_type(&self) -> NodeType {
        NodeType::Grant
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
