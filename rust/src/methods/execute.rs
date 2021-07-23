use eyre::Result;
use maplit::hashmap;
use stencila_schema::Node;

// Allow these for when no features are enabled
#[allow(unused_variables, unreachable_code)]
pub async fn execute(node: Node) -> Result<Node> {
    #[cfg(feature = "plugins")]
    return crate::plugins::delegate(
        super::Method::Execute,
        hashmap! { "node".to_string() => serde_json::to_value(node)? },
    )
    .await;

    #[cfg(not(feature = "plugins"))]
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
