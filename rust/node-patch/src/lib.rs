use common::{
    eyre::Result, itertools::Itertools, serde::Deserialize, strum::Display, tokio::sync::mpsc,
};
use node_store::{
    automerge::{transaction::Transactable, ObjId, Prop},
    ReadNode, ReadStore, WriteNode, WriteStore,
};
use schema::{
    Block, Duration, ExecutionMessage, ExecutionRequired, ExecutionStatus, Node, NodeId, Section, SuggestionBlockType, SuggestionInlineType, Timestamp
};

/// Replace a property of a node with a value
pub fn replace_property(
    store: &mut WriteStore,
    node_id: &NodeId,
    property: Property,
    value: Value,
) -> Result<()> {
    value.put_prop(store, &obj_id(node_id)?, property.into_prop())
}

/// Load a property of a node from a store
pub fn load_property<S, T>(store: &S, node_id: &NodeId, property: Property) -> Result<T>
where
    S: ReadStore,
    T: ReadNode,
{
    T::load_prop(store, &obj_id(node_id)?, property.into_prop())
}

/// Get the Automerge [`ObjId`] corresponding to the [`NodeId`] of the patch
///
/// Used when applying the patch to an Automerge store.
fn obj_id(node_id: &NodeId) -> Result<ObjId> {
    Ok(ObjId::try_from(node_id.uid())?)
}

/// A patch to a [`Node`] within a node tree
#[derive(Debug, Deserialize)]
#[serde(crate = "common::serde")]
pub struct NodePatch {
    /// The id of the node to apply operations to
    #[serde(alias = "nodeId")]
    pub node_id: NodeId,

    /// The operations to apply to the node
    pub ops: Vec<Operation>,
}

impl NodePatch {
    /// Apply a patch to a store
    pub fn apply(self, store: &mut WriteStore) -> Result<()> {
        let obj_id = obj_id(&self.node_id)?;

        for op in self.ops {
            match op {
                Operation::ReplaceProperty(ReplaceProperty { property, value }) => {
                    value.put_prop(store, &obj_id, property.into_prop())?;
                }
            };
        }

        Ok(())
    }
}

/// A [`NodePatch`] channel sender for sending patches to be applied to a node
///
/// This is an [`mpsc::UnboundedSender`] because we do not want the sending function
/// to have to `await` the send.
pub type NodePatchSender = mpsc::UnboundedSender<NodePatch>;

/// A [`NodePatch`] channel receiver for receiving patches to be applied to a node
pub type NodePatchReceiver = mpsc::UnboundedReceiver<NodePatch>;

/// An operation within a [`NodePatch`]
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase", crate = "common::serde")]
pub enum Operation {
    ReplaceProperty(ReplaceProperty),
}

impl Operation {
    pub fn replace_property(property: Property, value: Value) -> Operation {
        Operation::ReplaceProperty(ReplaceProperty { property, value })
    }
}

/// Replace the value of a property of a `struct` node type (e.g. `Article`, `CodeChunk`)
#[derive(Debug, Deserialize)]
#[serde(crate = "common::serde")]
pub struct ReplaceProperty {
    property: Property,
    value: Value,
}

/// A property of a node
#[derive(Debug, Display, Clone, Copy, Deserialize)]
// Must be `snake_case` so that `to_string` produces a value
// with the same casing used in the store
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub enum Property {
    Content,
    ExecutionCount,
    ExecutionDuration,
    ExecutionEnded,
    ExecutionMessages,
    ExecutionRequired,
    ExecutionStatus,
    IsActive,
    Iterations,
    Output,
    Outputs,
    Suggestion,
}

impl Property {
    /// Get the Automerge [`Prop`] corresponding to the property
    ///
    /// Used when applying the patch operation to an Automerge store.
    fn into_prop(self) -> Prop {
        Prop::Map(self.to_string())
    }
}

/// A value to set a property of a node
#[derive(Debug, Deserialize)]
#[serde(untagged, crate = "common::serde")]
pub enum Value {
    // Order is important for deserialization
    Many(Vec<Node>),
    One(Node),
    None,
}

impl WriteNode for Value {
    fn put_prop(&self, store: &mut WriteStore, obj_id: &ObjId, prop: Prop) -> Result<()> {
        match self {
            Value::None => Ok(store.delete(obj_id, prop)?),
            Value::One(node) => node.put_prop(store, obj_id, prop),
            Value::Many(nodes) => nodes.put_prop(store, obj_id, prop),
        }
    }
}

// These implementation of `From<T> for Value` are done as needed
// (largely based on which properties need to be patched)

impl<T> From<Option<T>> for Value
where
    T: Into<Value>,
{
    fn from(value: Option<T>) -> Self {
        match value {
            Some(value) => value.into(),
            None => Value::None,
        }
    }
}

impl From<Node> for Value {
    fn from(value: Node) -> Self {
        Value::One(value)
    }
}

impl From<Vec<Node>> for Value {
    fn from(value: Vec<Node>) -> Self {
        Value::Many(value)
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::One(Node::Boolean(value))
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::One(Node::Integer(value))
    }
}

impl From<Duration> for Value {
    fn from(value: Duration) -> Self {
        Value::One(Node::Duration(value))
    }
}

impl From<Timestamp> for Value {
    fn from(value: Timestamp) -> Self {
        Value::One(Node::Timestamp(value))
    }
}

impl From<ExecutionRequired> for Value {
    fn from(value: ExecutionRequired) -> Self {
        Value::One(Node::String(value.to_string()))
    }
}

impl From<ExecutionStatus> for Value {
    fn from(value: ExecutionStatus) -> Self {
        Value::One(Node::String(value.to_string()))
    }
}

impl From<Vec<ExecutionMessage>> for Value {
    fn from(value: Vec<ExecutionMessage>) -> Self {
        Value::Many(value.into_iter().map(Node::ExecutionMessage).collect_vec())
    }
}

impl From<SuggestionBlockType> for Value {
    fn from(value: SuggestionBlockType) -> Self {
        Value::One(match value {
            SuggestionBlockType::InsertBlock(block) => Node::InsertBlock(block),
            SuggestionBlockType::ModifyBlock(block) => Node::ModifyBlock(block),
            SuggestionBlockType::ReplaceBlock(block) => Node::ReplaceBlock(block),
            SuggestionBlockType::DeleteBlock(block) => Node::DeleteBlock(block),
        })
    }
}

impl From<SuggestionInlineType> for Value {
    fn from(value: SuggestionInlineType) -> Self {
        Value::One(match value {
            SuggestionInlineType::InsertInline(block) => Node::InsertInline(block),
            SuggestionInlineType::ModifyInline(block) => Node::ModifyInline(block),
            SuggestionInlineType::ReplaceInline(block) => Node::ReplaceInline(block),
            SuggestionInlineType::DeleteInline(block) => Node::DeleteInline(block),
        })
    }
}

impl From<Vec<Block>> for Value {
    fn from(value: Vec<Block>) -> Self {
        Value::Many(value.into_iter().map(|item| item.into()).collect_vec())
    }
}

impl From<Vec<Section>> for Value {
    fn from(value: Vec<Section>) -> Self {
        Value::Many(value.into_iter().map(Node::Section).collect_vec())
    }
}
