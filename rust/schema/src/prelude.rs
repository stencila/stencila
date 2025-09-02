pub use async_recursion::async_recursion;
pub use derive_more::{Deref, DerefMut};
pub use eyre::{ErrReport, Result, bail};
pub use itertools::Itertools;
pub use monostate::MustBe;
pub use serde::{Deserialize, Serialize};
pub use serde_json;
pub use serde_with::{serde_as, skip_serializing_none};
pub use smart_default::SmartDefault;
pub use smol_str::SmolStr;
pub use strum::EnumString;

pub use stencila_codec_dom_trait::{DomCodec, DomEncodeContext};
pub use stencila_codec_html_trait::{HtmlCodec, HtmlEncodeContext};
pub use stencila_codec_info::Losses;
pub use stencila_codec_jats_trait::JatsCodec;
pub use stencila_codec_latex_trait::{LatexCodec, LatexEncodeContext};
pub use stencila_codec_markdown_trait::{MarkdownCodec, MarkdownEncodeContext};
pub use stencila_codec_text_trait::TextCodec;
pub use stencila_format::Format;
pub use stencila_node_id::{NodeId, NodeUid};
pub use stencila_node_patch_derive::PatchNode;
pub use stencila_node_path::{NodePath, NodeSlot};
pub use stencila_node_probe_derive::ProbeNode;
pub use stencila_node_store::{ReadNode, WriteNode};
pub use stencila_node_strip::{StripNode, StripScope, StripTargets};
pub use stencila_node_type::{ContentType, NodeProperty, NodeType};
pub use stencila_node_walk_derive::WalkNode;

pub use crate::deserialize::*;
pub use crate::patch::{Patch, PatchContext, PatchNode, PatchOp, PatchValue};
pub use crate::probe::{NodeSet, ProbeNode};
pub use crate::walk::{Visitor, VisitorAsync, VisitorMut, WalkNode};
pub use crate::{Author, AuthorType, Node, ProvenanceCount};

#[cfg(feature = "proptest")]
pub use crate::proptests::*;
