use common::{
    eyre::Result,
    serde::{de::DeserializeOwned, Serialize},
    serde_json
};

/// Replicate a node to produce clone having different `node_id`
///
/// This is currently done by serializing/deserializing to/from a `serde_json::Value`
/// (because a node's `uid` is skipped during serialization). A more efficient
/// method may be implemented in the future.
pub fn replicate<T: Serialize + DeserializeOwned>(node: &T) -> Result<T> {
    Ok(serde_json::from_value(serde_json::to_value(node)?)?)
}
