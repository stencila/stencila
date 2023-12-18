pub use monostate::MustBe;
pub use serde_with::{self, serde_as, skip_serializing_none};

pub use common::{
    derive_more::{self, Deref, DerefMut},
    eyre::{bail, eyre, ErrReport, Result},
    itertools::Itertools,
    serde::{self, Deserialize, Serialize},
    serde_json,
    smart_default::SmartDefault,
    strum,
};

pub use codec_html_trait::{HtmlCodec, HtmlEncodeContext};
pub use codec_jats_trait::JatsCodec;
pub use codec_losses::Losses;
pub use codec_markdown_trait::{MarkdownCodec, MarkdownEncodeContext};
pub use codec_text_trait::TextCodec;
pub use node_store::{ReadNode, WriteNode};
pub use node_strip::StripNode;
pub use node_walk_derive::WalkNode;

pub use crate::deserialize::*;
pub use crate::shortcuts::*;
pub use crate::traits::{
    entity::{Entity, NodeId},
    walk::{Visitor, VisitorMut, WalkNode},
};
pub use crate::types::NodeType;

#[cfg(feature = "proptest")]
pub use crate::proptests::*;
