use crate::plugins;
use eyre::Result;
use stencila_schema::Node;

// Allow these for when no features are enabled
#[allow(unused_variables, unreachable_code)]
pub async fn execute(node: Node) -> Result<Node> {
    #[cfg(feature = "request")]
    return plugins::delegate(super::Method::Execute, &serde_json::json!({ "node": node })).await;

    #[cfg(not(feature = "request"))]
    eyre::bail!("Unable to execute node")
}

#[cfg(any(feature = "request", feature = "serve"))]
pub mod rpc {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Params {
        pub node: Node,
    }

    pub async fn execute(params: Params) -> Result<Node> {
        let Params { node } = params;
        super::execute(node).await
    }
}
