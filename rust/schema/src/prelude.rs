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

pub use codec_dom_trait::{DomCodec, DomEncodeContext};
pub use codec_html_trait::{HtmlCodec, HtmlEncodeContext};
pub use codec_info::Losses;
pub use codec_jats_trait::JatsCodec;
pub use codec_latex_trait::{LatexCodec, LatexEncodeContext};
pub use codec_markdown_trait::{MarkdownCodec, MarkdownEncodeContext};
pub use codec_text_trait::TextCodec;
pub use format::Format;
pub use node_id::{NodeId, NodeUid};
pub use node_patch_derive::PatchNode;
pub use node_path::{NodePath, NodeSlot};
pub use node_probe_derive::ProbeNode;
pub use node_store::{ReadNode, WriteNode};
pub use node_strip::{StripNode, StripScope, StripTargets};
pub use node_type::{ContentType, NodeProperty, NodeType};
pub use node_walk_derive::WalkNode;

pub use crate::deserialize::*;
pub use crate::patch::{Patch, PatchContext, PatchNode, PatchOp, PatchValue};
pub use crate::probe::{NodeSet, ProbeNode};
pub use crate::walk::{Visitor, VisitorAsync, VisitorMut, WalkNode};
pub use crate::{Author, AuthorType, Node, ProvenanceCount};

#[cfg(feature = "proptest")]
pub use crate::proptests::*;
