use common::{
    eyre::Result,
    serde::{de::DeserializeOwned, Serialize},
    serde_json,
};

/// Replicate a node to produce clone having different `node_id`
///
/// This is currently done by serializing/deserializing to/from a JSON string
/// (because a node's `uid` is skipped during serialization). A more efficient
/// method may be implemented in the future. An initial version used
/// `serde_json::Value` but that errored for union types like Block (related to Cord?)
pub fn replicate<T: Serialize + DeserializeOwned>(node: &T) -> Result<T> {
    Ok(serde_json::from_str(&serde_json::to_string(node)?)?)
}
