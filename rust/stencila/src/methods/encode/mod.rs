use codec_trait::{Codec, EncodeOptions};
use eyre::{bail, Result};
use stencila_schema::Node;

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
#[allow(unused_variables)]
pub async fn encode(
    node: &Node,
    output: &str,
    format: &str,
    options: Option<EncodeOptions>,
) -> Result<String> {
    // Allow these for when no features are enabled
    // TODO: pass through options when all codecs are refactored
    #[allow(unused_variables, unreachable_code)]
    Ok(match format {
        // Core formats, not behind feature flags
        "html" => codec_html::HtmlCodec::to_string(node, options)?,
        "txt" => codec_txt::TxtCodec::to_string(node, options)?,

        #[cfg(feature = "decode-docx")]
        "docx" => codec_docx::DocxCodec::to_string_async(node, options).await?,

        #[cfg(feature = "encode-ipynb")]
        "ipynb" => codec_ipynb::IpynbCodec::to_string(node, options)?,

        #[cfg(feature = "encode-json")]
        "json" => codec_json::JsonCodec::to_string(node, options)?,

        #[cfg(feature = "encode-json5")]
        "json5" => codec_json5::Json5Codec::to_string(node, options)?,

        #[cfg(feature = "encode-latex")]
        "latex" => codec_latex::LatexCodec::to_string_async(node, options).await?,

        #[cfg(feature = "encode-md")]
        "md" => codec_md::MarkdownCodec::to_string(node, options)?,

        #[cfg(feature = "encode-pandoc")]
        "pandoc" => codec_pandoc::PandocCodec::to_string_async(node, options).await?,

        #[cfg(feature = "encode-pdf")]
        "pdf" => codec_pdf::PdfCodec::to_string_async(node, options).await?,

        #[cfg(feature = "encode-png")]
        "png" => codec_png::PngCodec::to_string_async(node, options).await?,

        #[cfg(feature = "encode-rmd")]
        "rmd" => codec_rmd::RmdCodec::to_string(node, options)?,

        #[cfg(feature = "encode-rpng")]
        "rpng" => codec_rpng::RpngCodec::to_string_async(node, options).await?,

        #[cfg(feature = "encode-toml")]
        "toml" => codec_toml::TomlCodec::to_string(node, options)?,

        #[cfg(feature = "encode-yaml")]
        "yaml" => codec_yaml::YamlCodec::to_string(node, options)?,

        _ => {
            #[cfg(feature = "request")]
            {
                let node = crate::plugins::delegate(
                    super::Method::Encode,
                    maplit::hashmap! {
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
