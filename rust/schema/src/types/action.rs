// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::action_agent::ActionAgent;
use super::action_status_type::ActionStatusType;
use super::container_image_or_string::ContainerImageOrString;
use super::date_time::DateTime;
use super::image_object::ImageObject;
use super::node::Node;
use super::property_value::PropertyValue;
use super::property_value_or_string::PropertyValueOrString;
use super::string::String;
use super::thing_variant_or_string::ThingVariantOrString;

/// An action performed by an agent.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase")]
#[derive(derive_more::Display)]
#[display("Action")]
pub struct Action {
    /// The type of this item.
    pub r#type: MustBe!("Action"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    #[jats(attr = "id")]
    pub id: Option<String>,

    /// The current status of the action.
    #[serde(alias = "action-status", alias = "action_status")]
    pub action_status: Option<ActionStatusType>,

    /// The direct performer or driver of the action.
    pub agent: Option<ActionAgent>,

    /// The objects or input values upon which the action is carried out.
    #[serde(alias = "object")]
    #[serde(default)]
    pub objects: Option<Vec<Node>>,

    /// The objects or values produced by the action.
    #[serde(alias = "result")]
    #[serde(default)]
    pub results: Option<Vec<Node>>,

    /// When the action started.
    #[serde(alias = "start-time", alias = "start_time")]
    #[strip(timestamps)]
    pub start_time: Option<DateTime>,

    /// When the action ended.
    #[serde(alias = "end-time", alias = "end_time")]
    #[strip(timestamps)]
    pub end_time: Option<DateTime>,

    /// Non-core optional fields
    #[serde(flatten)]
    #[html(flatten)]
    #[jats(flatten)]
    pub options: Box<ActionOptions>,

    /// A unique identifier for a node within a document
    #[serde(skip)]
    pub uid: NodeUid
}

#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase")]
pub struct ActionOptions {
    /// Alternate names (aliases) for the item.
    #[serde(alias = "alternate-names", alias = "alternate_names", alias = "alternateName", alias = "alternate-name", alias = "alternate_name")]
    #[serde(default, deserialize_with = "option_csv_or_array")]
    #[strip(metadata)]
    pub alternate_names: Option<Vec<String>>,

    /// A description of the item.
    #[strip(metadata)]
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
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

    /// Other agents that participated in the action.
    #[serde(alias = "participant")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    pub participants: Option<Vec<ActionAgent>>,

    /// The service provider, service operator, or performer responsible for the action.
    pub provider: Option<ActionAgent>,

    /// The object, software, or other instrument that helped perform the action.
    pub instrument: Option<ThingVariantOrString>,

    /// Environment variables or settings that affected the action.
    #[serde(default, deserialize_with = "option_one_or_many")]
    pub environment: Option<Vec<PropertyValue>>,

    /// Container images used by the action.
    #[serde(alias = "containerImage", alias = "container-image", alias = "container_image", alias = "container-images", alias = "container_images")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    pub container_images: Option<Vec<ContainerImageOrString>>,

    /// An error produced by the action.
    pub error: Option<ThingVariantOrString>,
}

impl Action {
    const NICK: [u8; 3] = *b"act";
    
    pub fn node_type(&self) -> NodeType {
        NodeType::Action
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
