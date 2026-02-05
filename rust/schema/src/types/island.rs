// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::block::Block;
use super::boolean::Boolean;
use super::label_type::LabelType;
use super::string::String;

/// An island of content in a document.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
#[derive(derive_more::Display)]
#[display("Island")]
pub struct Island {
    /// The type of this item.
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub r#type: MustBe!("Island"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[html(attr = "id")]
    #[jats(attr = "id")]
    pub id: Option<String>,

    /// The content within the section.
    #[serde(deserialize_with = "one_or_many")]
    #[walk]
    #[patch(format = "all")]
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    #[dom(elem = "section")]
    pub content: Vec<Block>,

    /// Whether the island is automatically generated.
    #[serde(alias = "is-automatic", alias = "is_automatic")]
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd", format = "latex")]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub is_automatic: Option<Boolean>,

    /// The type of the label for the island.
    #[serde(alias = "label-type", alias = "label_type")]
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd", format = "latex")]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub label_type: Option<LabelType>,

    /// A short label for the chunk.
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd", format = "latex")]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub label: Option<String>,

    /// Whether the label should be automatically updated.
    #[serde(alias = "label-automatically", alias = "label_automatically")]
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd", format = "latex")]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub label_automatically: Option<Boolean>,

    /// Other IDs for the island, in addition to the primary `id`.
    #[serde(alias = "other-ids", alias = "other_ids", alias = "otherId", alias = "other-id", alias = "other_id")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[patch(format = "latex")]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub other_ids: Option<Vec<String>>,

    /// The style to apply to the island.
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd", format = "latex")]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub style: Option<String>,

    /// A unique identifier for a node within a document
    #[serde(skip)]
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub uid: NodeUid
}

impl Island {
    const NICK: [u8; 3] = *b"isl";
    
    pub fn node_type(&self) -> NodeType {
        NodeType::Island
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new(content: Vec<Block>) -> Self {
        Self {
            content,
            ..Default::default()
        }
    }
}
