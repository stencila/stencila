pub use derive_more::{self, Deref, DerefMut};
pub use monostate::MustBe;
pub use serde_with::{self, serde_as, skip_serializing_none};

pub use common::{
    async_recursion::async_recursion,
    eyre::{ErrReport, Result, bail},
    itertools::Itertools,
    serde::{self, Deserialize, Serialize},
    serde_json,
    smart_default::SmartDefault,
    smol_str::{self, SmolStr},
    strum,
};

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
