use node_id::NodeId;
use node_type::NodeType;

/// A trait for entity node types to get their type and id
pub trait Entity {
    /// A short nickname for the type
    ///
    /// All lower alpha characters, preferably less than three characters,
    /// and unique across node types.
    const NICK: &'static str;

    /// Get the [`NodeType`] of the entity
    fn node_type(&self) -> NodeType;

    /// Get the [`NodeId`] of the entity
    fn node_id(&self) -> NodeId;
}
