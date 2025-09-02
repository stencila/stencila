#![recursion_limit = "256"]

mod deserialize;
mod implem;
mod prelude;

#[rustfmt::skip]
mod types;
pub use types::*;

mod patch;
pub use patch::*;

mod probe;
pub use probe::*;

mod walk;
pub use walk::*;

mod replicate;
pub use replicate::*;

mod url;
pub use url::*;

pub mod shortcuts;
pub mod transforms;

pub use stencila_node_id::NodeId;
pub use stencila_node_path::{NodePath, NodeSlot};
pub use stencila_node_strip::{StripNode, StripScope, StripTargets, strip, strip_non_content};
pub use stencila_node_type::{ContentType, NodeProperty, NodeType};

pub mod cord_provenance;

pub use implem::AuthorType;

#[cfg(feature = "proptest")]
mod proptests;
