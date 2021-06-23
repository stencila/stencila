use eyre::{bail, Result};
use maplit::hashmap;
use stencila_schema::Node;

#[cfg(feature = "encode-json")]
mod json;

#[cfg(feature = "encode-html")]
mod html;

#[cfg(feature = "encode-md")]
mod md;

#[cfg(feature = "encode-toml")]
mod toml;

#[cfg(feature = "encode-yaml")]
mod yaml;

/// Encode a `Node` to string content.
///
/// # Arguments
///
/// - `node`: the node to encode
/// - `format`: the format of the content e.g. `json`, `md`
pub async fn encode(node: &Node, format: &str) -> Result<String> {
    // Allow these for when no features are enabled
    #[allow(unused_variables, unreachable_code)]
    Ok(match format {
        #[cfg(feature = "encode-html")]
        "html" => html::encode(node)?,

        #[cfg(feature = "encode-json")]
        "json" => json::encode(node)?,

        #[cfg(feature = "encode-md")]
        "md" => md::encode(node)?,

        #[cfg(feature = "encode-toml")]
        "toml" => toml::encode(node)?,

        #[cfg(feature = "encode-yaml")]
        "yaml" => yaml::encode(node)?,

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

        pub format: String,
    }

    pub async fn encode(params: Params) -> Result<String> {
        let Params { node, format } = params;
        super::encode(&node, &format).await
    }
}
