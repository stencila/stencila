// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::boolean::Boolean;
use super::string::String;

/// An icon, typically rendered using an icon font.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, HtmlCodec, JatsCodec, LatexCodec, TextCodec)]
#[serde(rename_all = "camelCase")]
#[derive(derive_more::Display)]
#[display("Icon")]
pub struct Icon {
    /// The type of this item.
    pub r#type: MustBe!("Icon"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    #[jats(attr = "id")]
    pub id: Option<String>,

    /// The name of the icon e.g. "clock" or "lucide:clock".
    pub name: String,

    /// An accessible text label for the icon.
    pub label: Option<String>,

    /// Whether the icon is purely decorative.
    pub decorative: Option<Boolean>,

    /// Tailwind utility classes to apply to the icon.
    pub style: Option<String>,

    /// A unique identifier for a node within a document
    #[serde(skip)]
    pub uid: NodeUid
}

impl Icon {
    const NICK: [u8; 3] = *b"ico";
    
    pub fn node_type(&self) -> NodeType {
        NodeType::Icon
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
