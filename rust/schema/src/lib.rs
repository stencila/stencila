#![recursion_limit = "256"]

mod deserialize;
mod implem;
mod prelude;

#[rustfmt::skip]
mod types;
pub use types::*;

mod path;
pub use path::*;

mod patch;
pub use patch::*;

mod probe;
pub use probe::*;

mod walk;
pub use walk::*;

mod replicate;
pub use replicate::*;

pub mod shortcuts;
pub mod transforms;

pub use node_id::NodeId;
pub use node_type::{NodeProperty, NodeType};

pub mod cord_provenance;

pub use implem::AuthorType;

#[cfg(feature = "proptest")]
mod proptests;
