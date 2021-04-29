//use crate::methods::Method;
use crate::methods::Method;
use crate::nodes::Node;
use eyre::Result;
use once_cell::sync::OnceCell;

type Delegator = dyn Fn(Method, serde_json::Value) -> Result<Node> + Send + Sync;
pub static DELEGATOR: OnceCell<Box<Delegator>> = OnceCell::new();

#[tracing::instrument]
pub fn delegate(method: Method, params: serde_json::Value) -> Result<Node> {
    // Get the delegator, falling back to the delegator that does nothing
    let delegator: &Delegator = match DELEGATOR.get() {
        Some(delegator) => delegator,
        None => &delegate_none,
    };

    delegator(method, params)
}

fn delegate_none(_method: Method, _params: serde_json::Value) -> Result<Node> {
    tracing::warn!("No delegator set");
    Ok(Node::Null)
}
