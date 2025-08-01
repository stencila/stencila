use std::{
    str::FromStr,
    sync::{Arc, Mutex as SyncMutex},
};

use codec_text_trait::to_text;
use kernel_jinja::{
    kernel::{
        common::{
            eyre::Result,
            itertools::Itertools,
            serde_json,
            tracing,
        },
        schema::{
            ExecutionMessage, MessageLevel, Node, NodePath, NodeProperty,
            NodeSet, get,
        },
    },
    minijinja::{
        Error, ErrorKind, State, Value,
        value::{Enumerator, Object, ObjectRepr},
    },
};

use crate::{lock_messages, try_messages};

/// A proxy for a [`Node`] to allow it to be accessed as a minijinja [`Value`]
///
/// This has several advantage over simply converting all nodes to values
/// via `serde_json`:
///
/// 1. We can provide getters for derived properties such as `text`
///
/// 2. We can create an error message if a non-existent property is accessed
///
/// 3. We can chain proxies together and convert to a minijinja value only
///    when appropriate e.g. for primitives
#[derive(Debug, Clone)]
pub(super) struct NodeProxy {
    /// The node being proxied
    node: Node,

    /// Execution messages to be added to when accessing the node
    messages: Arc<SyncMutex<Vec<ExecutionMessage>>>,
}

impl NodeProxy {
    pub fn new(node: Node, messages: Arc<SyncMutex<Vec<ExecutionMessage>>>) -> Self {
        Self { node, messages }
    }

    pub fn nodes(&self) -> Vec<Node> {
        vec![self.node.clone()]
    }

    pub fn text(&self) -> Result<Value, Error> {
        try_messages(&self.messages)?;
        Ok(Value::from(to_text(&self.node)))
    }
}

impl Object for NodeProxy {
    fn repr(self: &Arc<Self>) -> ObjectRepr {
        ObjectRepr::Map
    }

    fn get_value(self: &Arc<Self>, key: &Value) -> Option<Value> {
        if key.is_integer() {
            if let Some(mut msgs) = lock_messages(&self.messages) {
                msgs.push(ExecutionMessage::new(
                    MessageLevel::Error,
                    "Cannot index a single node".into(),
                ))
            };
            return Some(Value::UNDEFINED);
        };

        let property = key.as_str()?;

        if property == "type" {
            return Some(Value::from(self.node.node_type().to_string()));
        }

        let Ok(property) = NodeProperty::from_str(property) else {
            if let Some(mut msgs) = lock_messages(&self.messages) {
                msgs.push(ExecutionMessage::new(
                    MessageLevel::Error,
                    format!("Invalid node property `{property}`"),
                ))
            };
            return Some(Value::UNDEFINED);
        };

        let Ok(property) = get(&self.node, NodePath::from(property)) else {
            if let Some(mut msgs) = lock_messages(&self.messages) {
                msgs.push(ExecutionMessage::new(
                    MessageLevel::Error,
                    format!(
                        "`{property}` is not a property of node type `{}`",
                        self.node.node_type()
                    ),
                ))
            };
            return Some(Value::UNDEFINED);
        };

        match node_set_to_value(property, &self.messages) {
            Ok(value) => Some(value),
            Err(error) => {
                tracing::error!("While converting node to minijinja value: {error}");
                None
            }
        }
    }

    fn call_method(
        self: &Arc<Self>,
        _state: &State,
        method: &str,
        args: &[Value],
    ) -> Result<Value, Error> {
        if method == "text" {
            if !args.is_empty() {
                return Err(Error::new(
                    ErrorKind::TooManyArguments,
                    "Method `text` takes no arguments.",
                ));
            }
            self.text()
        } else {
            Err(Error::new(
                ErrorKind::UnknownMethod,
                format!("Method `{method}` takes no arguments."),
            ))
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct NodeProxies {
    /// The nodes being proxied
    nodes: Vec<Node>,

    /// Execution messages to be added to when accessing the nodes
    messages: Arc<SyncMutex<Vec<ExecutionMessage>>>,
}

impl NodeProxies {
    pub fn new(nodes: Vec<Node>, messages: Arc<SyncMutex<Vec<ExecutionMessage>>>) -> Self {
        Self { nodes, messages }
    }

    pub fn nodes(&self) -> Vec<Node> {
        self.nodes.clone()
    }

    pub fn text(&self) -> Result<Value, Error> {
        try_messages(&self.messages)?;
        Ok(Value::from(self.nodes.iter().map(to_text).join(" ")))
    }
}

impl Object for NodeProxies {
    fn repr(self: &Arc<Self>) -> ObjectRepr {
        ObjectRepr::Seq
    }

    fn enumerate(self: &Arc<Self>) -> Enumerator {
        Enumerator::Seq(self.nodes.len())
    }

    fn get_value(self: &Arc<Self>, key: &Value) -> Option<Value> {
        if let Some(property) = key.as_str() {
            if let Some(mut msgs) = lock_messages(&self.messages) {
                msgs.push(ExecutionMessage::new(
                    MessageLevel::Error,
                    format!("`{property}` is not a property of a node list"),
                ))
            };
            return Some(Value::UNDEFINED);
        };

        let key = key.as_i64()?;

        let index = if key < 0 {
            self.nodes.len() as i64 - key - 1
        } else {
            key
        };

        if index < 0 || index >= self.nodes.len() as i64 {
            return None;
        }

        let node = self.nodes[index as usize].clone();
        Some(Value::from_object(NodeProxy::new(
            node,
            self.messages.clone(),
        )))
    }

    fn call_method(
        self: &Arc<Self>,
        _state: &State,
        method: &str,
        args: &[Value],
    ) -> Result<Value, Error> {
        if method == "text" {
            if !args.is_empty() {
                return Err(Error::new(
                    ErrorKind::TooManyArguments,
                    "Method `text` takes no arguments.",
                ));
            }
            self.text()
        } else {
            Err(Error::new(
                ErrorKind::UnknownMethod,
                format!("Method `{method}` takes no arguments."),
            ))
        }
    }
}

fn node_set_to_value(
    node_set: NodeSet,
    messages: &Arc<SyncMutex<Vec<ExecutionMessage>>>,
) -> Result<Value> {
    match node_set {
        NodeSet::One(node) => node_to_value(node, messages),
        NodeSet::Many(nodes) => Ok(Value::from_object(NodeProxies::new(
            nodes,
            messages.clone(),
        ))),
    }
}

fn node_to_value(node: Node, messages: &Arc<SyncMutex<Vec<ExecutionMessage>>>) -> Result<Value> {
    match node {
        Node::Null(..) => Ok(Value::from(())),
        Node::Boolean(node) => Ok(Value::from(node)),
        Node::Integer(node) => Ok(Value::from(node)),
        Node::UnsignedInteger(node) => Ok(Value::from(node)),
        Node::Number(node) => Ok(Value::from(node)),
        Node::String(node) => Ok(Value::from(node)),
        Node::Array(..) | Node::Object(..) => node_to_value_via_serde(node),
        _ => Ok(Value::from_object(NodeProxy::new(node, messages.clone()))),
    }
}

fn node_to_value_via_serde(node: Node) -> Result<Value> {
    let value = serde_json::to_value(node)?;
    Ok(serde_json::from_value(value)?)
}
