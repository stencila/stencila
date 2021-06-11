use eyre::{bail, Result};
use maplit::hashmap;
use stencila_schema::Node;

// Allow these for when no features are enabled
#[allow(unused_variables, unreachable_code)]
pub async fn encode(node: &Node, format: &str) -> Result<String> {
    Ok(match format {
        #[cfg(feature = "format-json")]
        "json" => serde_json::to_string(node)?,

        #[cfg(feature = "format-yaml")]
        "yaml" => serde_yaml::to_string(node)?,

        #[cfg(feature = "format-html")]
        "html" => super::encode_html::encode_html(node)?,

        _ => {
            #[cfg(feature = "request")]
            {
                let node = crate::plugins::delegate(
                    super::Method::Encode,
                    hashmap! {
                        "node".to_string() => serde_json::to_value(node)?,
                        "format".to_string() => serde_json::to_value(format)?
                    },
                )
                .await?;
                // Delegate returns a node so always convert it to a string
                match node {
                    Node::String(string) => string,
                    _ => bail!("Unexpectedly got a non-string type"),
                }
            }

            #[cfg(not(feature = "request"))]
            bail!("Unable to encode to format \"{}\"", format)
        }
    })
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
