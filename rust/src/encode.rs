use crate::nodes::Node;
use anyhow::Result;

#[cfg(any(feature = "request", feature = "serve"))]
pub mod rpc {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Params {
        pub node: Node,

        pub format: Option<String>,
    }

    pub fn encode(params: Params) -> Result<String> {
        let Params { node, format } = params;
        super::encode(node, format.unwrap_or_default())
    }
}

// Allow these for when no features are enabled
#[allow(unused_variables, unreachable_code)]
pub fn encode(node: Node, format: String) -> Result<String> {
    let content = match format.as_str() {
        #[cfg(feature = "json")]
        "json" => serde_json::to_string(&node)?,
        #[cfg(feature = "yaml")]
        "yaml" => serde_yaml::to_string(&node)?,
        #[cfg(feature = "cli")]
        "cli" => encode_cli(node)?,
        _ => {
            #[cfg(feature = "request")]
            {
                let node = crate::delegate::delegate(
                    crate::methods::Method::Encode,
                    serde_json::json!({
                        "node": node,
                        "format": format,
                    }),
                )?;
                // Delegate returns a node so always convert it to a string
                return Ok(node.to_string());
            };

            #[cfg(not(feature = "request"))]
            anyhow::bail!("Unable to encode a node to format \"{}\"", format)
        }
    };
    Ok(content)
}

#[cfg(feature = "cli")]
fn encode_cli(node: Node) -> Result<String> {
    match node {
        Node::String(node) => Ok(node),
        _ => Ok(serde_json::to_string_pretty(&node)?),
    }
}
