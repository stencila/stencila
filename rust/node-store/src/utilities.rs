use automerge::{ObjType, Prop, ScalarValue, Value};

use common::eyre::{bail, Result};
use node_type::NodeType;

use crate::{automerge::ObjId, ReadStore};

/// Get the type property of an object in a CRDT
pub fn get_type<C: ReadStore>(crdt: &C, obj_id: &ObjId) -> Result<Option<String>> {
    // This function is normally only be called for Stencila struct types (not for primitives)
    // However, if the Automerge object is not a `Map` the following `get` call will panic!
    // So its important to do this check, and return the closest Stencila type to the
    // Automerge type.
    match crdt.object_type(obj_id)? {
        ObjType::List => return Ok(Some("Array".to_string())),
        ObjType::Text => return Ok(Some("String".to_string())),
        _ => {}
    };

    let Some((value,..)) = crdt.get(obj_id, Prop::from("type"))? else {
        return Ok(None)
    };

    let Value::Scalar(value) = value else {
        bail!("Expected `type` property to be a scalar");
    };

    let ScalarValue::Str(value) = value.as_ref() else {
        bail!("Expected `type` property to be a string");
    };

    Ok(Some(value.to_string()))
}

/// Get the [`NodeType`] for an object in a CRDT
///
/// Returns `None` if the type can not be resolved for the object.
pub fn get_node_type<C: ReadStore>(crdt: &C, obj_id: &ObjId) -> Result<Option<NodeType>> {
    let Some(node_type) = get_type(crdt, obj_id)? else {
        return Ok(None)
    };

    Ok(Some(NodeType::try_from(node_type.as_str())?))
}
