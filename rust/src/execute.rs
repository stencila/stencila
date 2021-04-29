use crate::nodes::Node;
use eyre::Result;

#[cfg(any(feature = "request", feature = "serve"))]
pub mod rpc {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Params {
        pub node: Node,
    }

    pub fn execute(params: Params) -> Result<Node> {
        let Params { node } = params;
        super::execute(node)
    }
}

// Allow these for when no features are enabled
#[allow(unused_variables, unreachable_code)]
pub fn execute(node: Node) -> Result<Node> {
    #[cfg(feature = "request")]
    return crate::delegate::delegate(
        crate::methods::Method::Execute,
        serde_json::json!({ "node": node }),
    );

    #[cfg(not(feature = "request"))]
    eyre::bail!("Unable to execute node")
}
