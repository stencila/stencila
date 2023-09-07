//! Utility functions involving Stencila Schema types
//!
//! These functions are often implemented here, instead of in other
//! internal crates, because of the need to avoid circular imports.

use common::eyre::Result;

use node_store::{automerge::ObjId, get_type, ReadStore};

use crate::NodeType;

/// Get the [`NodeType`] for an object in a store
///
/// Returns `None` if the type can not be resolved for the object.
pub fn node_type<S: ReadStore>(store: &S, obj_id: &ObjId) -> Result<Option<NodeType>> {
    let Some(node_type) = get_type(store, obj_id)? else {
        return Ok(None)
    };

    Ok(Some(NodeType::try_from(node_type.as_str())?))
}
