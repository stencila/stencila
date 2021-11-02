use codec_trait::Codec;
use defaults::Defaults;
use eyre::{bail, Result};
use stencila_schema::Node;

#[cfg(feature = "encode-docx")]
pub mod docx;

#[cfg(feature = "encode-latex")]
pub mod latex;

#[cfg(feature = "encode-pandoc")]
pub mod pandoc;

#[cfg(feature = "encode-pdf")]
pub mod pdf;

#[cfg(feature = "encode-rpng")]
pub mod rpng;

/// Common encoding options
///
/// Encoding functions (including those in plugins) are encouraged to respect these options
/// but are not required to. Indeed, some options do not apply for some formats.
/// For example, a PDF is always `standalone` (so if that option is set to `false` is it will be ignored).
/// Futhermore, some combinations of options are ineffectual e.g. a `theme` when `standalone: false`
#[derive(Clone, Debug, Defaults)]
pub struct Options {
    /// Whether to encode in compact form.
    ///
    /// Some formats (e.g HTML and JSON) can be encoded in either compact
    /// or "pretty-printed" ie.e. indented forms.
    #[def = "true"]
    pub compact: bool,

    /// Whether to ensure that the encoded document is standalone.
    ///
    /// Some formats (e.g. Markdown, DOCX) are always standalone, others
    /// can be frangments, or standalong documents (e.g HTML).
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
    ///
    /// Only applies to some formats (e.g. HTML, PDF, PNG).
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
#[allow(unused_variables)]
pub async fn encode(
    node: &Node,
    output: &str,
    format: &str,
    options: Option<Options>,
) -> Result<String> {
    // Allow these for when no features are enabled
    // TODO: pass through options when all codecs are refactored
    #[allow(unused_variables, unreachable_code)]
    Ok(match format {
        // Core formats, not behind feature flags
        "html" => codec_html::HtmlCodec::to_string(node, None)?,
        "txt" => codec_txt::TxtCodec::to_string(node, None)?,

        #[cfg(feature = "decode-docx")]
        "docx" => docx::encode(node, output).await?,

        #[cfg(feature = "encode-ipynb")]
        "ipynb" => codec_ipynb::IpynbCodec::to_string(node, None)?,

        #[cfg(feature = "encode-json")]
        "json" => codec_json::JsonCodec::to_string(node, None)?,

        #[cfg(feature = "encode-json5")]
        "json5" => codec_json5::Json5Codec::to_string(node, None)?,

        #[cfg(feature = "encode-latex")]
        "latex" => latex::encode(node).await?,

        #[cfg(feature = "encode-md")]
        "md" => codec_md::MarkdownCodec::to_string(node, None)?,

        #[cfg(feature = "encode-pandoc")]
        "pandoc" => pandoc::encode(node, output, "pandoc", &[]).await?,

        #[cfg(feature = "encode-pdf")]
        "pdf" => pdf::encode(node, output, options).await?,

        #[cfg(feature = "encode-png")]
        "png" => codec_png::PngCodec::to_string_async(node, None).await?,

        #[cfg(feature = "encode-rmd")]
        "rmd" => codec_rmd::RmdCodec::to_string(node, None)?,

        #[cfg(feature = "encode-rpng")]
        "rpng" => rpng::encode(node, output).await?,

        #[cfg(feature = "encode-toml")]
        "toml" => codec_toml::TomlCodec::to_string(node, None)?,

        #[cfg(feature = "encode-yaml")]
        "yaml" => codec_yaml::YamlCodec::to_string(node, None)?,

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
