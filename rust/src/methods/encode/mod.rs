use defaults::Defaults;
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

#[cfg(feature = "encode-rmd")]
pub mod rmd;

#[cfg(feature = "encode-toml")]
pub mod toml;

#[cfg(feature = "encode-txt")]
pub mod txt;

#[cfg(feature = "encode-yaml")]
pub mod yaml;

/// Common encoding options
///
/// Functions (including in plugins) are encouraged to respect these options
/// but are not required to. Indeed, some options do not apply for some formats.
/// For example, a PDF is always `standalone` (so if that option is set to `false` is it will be ignored).
/// Futhermore, some combinations of options are ineffectual e.g. a `theme` when `standalone: false`
#[derive(Clone, Debug, Defaults)]
pub struct Options {
    /// Whether to ensure that the encoded document is standalone
    #[def = "false"]
    pub standalone: bool,

    /// Whether to bundle local media files into the encoded document
    ///
    /// Some formats (e.g. DOCX, PDF) always bundle. For HTML,
    /// bundling means including media as data URIs rather than
    /// links to files.
    #[def = "false"]
    pub bundle: bool,

    /// The theme to apply to the encoded document
    #[def = "\"stencila\".to_string()"]
    pub theme: String,
}

/// Encode a `Node` to content in a particular format
///
/// # Arguments
///
/// - `node`: the node to encode
/// - `output`: the destination file path (if applicable for the format)
/// - `format`: the format of the content (e.g. `json`, `md`)
/// - `options`: any additional options
///
/// # Returns
///
/// The encoded content, or if a file was written, a file:// URL with the
/// path of the file (which should equal the `output` argument).
pub async fn encode(
    node: &Node,
    output: &str,
    format: &str,
    options: Option<Options>,
) -> Result<String> {
    // Allow these for when no features are enabled
    #[allow(unused_variables, unreachable_code)]
    Ok(match format {
        #[cfg(feature = "decode-docx")]
        "docx" => docx::encode(node, output).await?,

        #[cfg(feature = "encode-html")]
        "html" => html::encode(node, options)?,

        #[cfg(feature = "encode-json")]
        "json" => json::encode(node)?,

        #[cfg(feature = "encode-latex")]
        "latex" => latex::encode(node).await?,

        #[cfg(feature = "encode-md")]
        "md" => md::encode(node)?,

        #[cfg(feature = "encode-pandoc")]
        "pandoc" => pandoc::encode(node, output, "pandoc", &[]).await?,

        #[cfg(feature = "encode-pdf")]
        "pdf" => pdf::encode(node, output, options).await?,

        #[cfg(feature = "encode-rmd")]
        "rmd" => rmd::encode(node)?,

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
        super::encode(&node, &output, &format, None).await
    }
}
