use eyre::{bail, Result};
use maplit::hashmap;
use stencila_schema::Node;

#[cfg(feature = "encode-docx")]
pub mod docx;

#[cfg(feature = "encode-html")]
#[allow(clippy::deprecated_cfg_attr)]
pub mod html;

#[cfg(feature = "encode-json")]
pub mod json;

#[cfg(feature = "encode-latex")]
pub mod latex;

#[cfg(feature = "encode-md")]
pub mod md;

#[cfg(feature = "encode-pandoc")]
pub mod pandoc;

#[cfg(feature = "encode-pdf")]
pub mod pdf;

#[cfg(feature = "encode-toml")]
pub mod toml;

#[cfg(feature = "encode-txt")]
pub mod txt;

#[cfg(feature = "encode-yaml")]
pub mod yaml;

/// Encode a `Node` to content in a particular format
///
/// # Arguments
///
/// - `node`: the node to encode
/// - `output`: the destination path, if applicable
/// - `format`: the format of the content e.g. `json`, `md`
pub async fn encode(node: &Node, output: &str, format: &str, theme: &str) -> Result<String> {
    // Allow these for when no features are enabled
    #[allow(unused_variables, unreachable_code)]
    Ok(match format {
        #[cfg(feature = "decode-docx")]
        "docx" => docx::encode(node, output).await?,

        #[cfg(feature = "encode-html")]
        "html" => html::encode(node, false, theme)?,

        #[cfg(feature = "encode-json")]
        "json" => json::encode(node)?,

        #[cfg(feature = "encode-latex")]
        "latex" => latex::encode(node).await?,

        #[cfg(feature = "encode-md")]
        "md" => md::encode(node)?,

        #[cfg(feature = "encode-pandoc")]
        "pandoc" => pandoc::encode(node, output, "pandoc", &[]).await?,

        #[cfg(feature = "encode-pdf")]
        "pdf" => pdf::encode(node, output, theme).await?,

        #[cfg(feature = "encode-toml")]
        "toml" => toml::encode(node)?,

        #[cfg(feature = "encode-txt")]
        "txt" => txt::encode(node)?,

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
        pub output: String,
        pub format: String,
    }

    pub async fn encode(params: Params) -> Result<String> {
        let Params {
            node,
            output,
            format,
        } = params;
        super::encode(&node, &output, &format, "").await
    }
}
