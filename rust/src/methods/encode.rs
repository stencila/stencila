use super::encode_html::encode_html;
use crate::plugins;
use eyre::{bail, Result};
use stencila_schema::Node;

// Allow these for when no features are enabled
#[allow(unused_variables, unreachable_code)]
pub async fn encode(node: &Node, format: &str) -> Result<String> {
    let content = match format {
        #[cfg(feature = "format-json")]
        "json" => serde_json::to_string(node)?,

        #[cfg(feature = "format-yaml")]
        "yaml" => serde_yaml::to_string(node)?,

        #[cfg(feature = "format-html")]
        "html" => encode_html(node)?,

        _ => {
            #[cfg(feature = "request")]
            {
                let node = plugins::delegate(
                    super::Method::Encode,
                    &serde_json::json!({
                        "node": node,
                        "format": format,
                    }),
                )
                .await?;
                // Delegate returns a node so always convert it to a string
                let string = match node {
                    Node::String(string) => string,
                    _ => bail!("Unexpectedly got a non-string type"),
                };
                return Ok(string);
            };

            #[cfg(not(feature = "request"))]
            eyre::bail!("Unable to encode a node to format \"{}\"", format)
        }
    };
    Ok(content)
}

#[cfg(any(feature = "request", feature = "serve"))]
pub mod rpc {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Params {
        pub node: Node,

        pub format: Option<String>,
    }

    pub async fn encode(params: Params) -> Result<String> {
        let Params { node, format } = params;
        super::encode(&node, &format.unwrap_or_else(|| "json".to_string())).await
    }
}
