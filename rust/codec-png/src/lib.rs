use std::path::Path;

use stencila_codec::{
    Codec, CodecSupport, EncodeInfo, EncodeOptions, NodeType, async_trait,
    eyre::{Result, bail},
    stencila_format::Format,
    stencila_schema::Node,
};
use stencila_codec_dom_trait::to_dom;
use stencila_codec_latex_trait::{latex_to_image, to_latex};
use stencila_convert::{html_to_png::html_to_png_file, html_to_png_data_uri};
use stencila_images::{ImageResizeOptions, resize_data_uri};

pub use stencila_images::ImageResizeOptions as PngImageResizeOptions;

/// Encode a node as a PNG dataURI
pub fn to_png_data_uri(node: &Node) -> Result<String> {
    html_to_png_data_uri(&to_dom(node))
}

/// Encode a node as a PNG data URI with options
///
/// Like [`to_png_data_uri`] but allows specifying custom [`ImageResizeOptions`] for
/// image resizing and optimization.
///
/// # Example
/// ```ignore
/// use stencila_images::ImageResizeOptions;
/// use stencila_codec_png::to_png_data_uri_with;
///
/// // Use email-optimized settings (600px max width)
/// let data_uri = to_png_data_uri_with(&node, &ImageResizeOptions::for_email())?;
/// ```
pub fn to_png_data_uri_with(node: &Node, options: &ImageResizeOptions) -> Result<String> {
    let data_uri = html_to_png_data_uri(&to_dom(node))?;

    // Resize if options specify max_width
    if options.max_width.is_some() {
        resize_data_uri(&data_uri, options)
    } else {
        Ok(data_uri)
    }
}

/// Encode a node as a PNG file
pub fn to_png_file(node: &Node, path: &Path) -> Result<()> {
    html_to_png_file(&to_dom(node), path)
}

/// A codec for PNG
pub struct PngCodec;

#[async_trait]
impl Codec for PngCodec {
    fn name(&self) -> &str {
        "png"
    }

    fn supports_from_format(&self, _format: &Format) -> CodecSupport {
        CodecSupport::None
    }

    fn supports_to_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Png => CodecSupport::HighLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_from_type(&self, _node_type: NodeType) -> CodecSupport {
        CodecSupport::None
    }

    fn supports_to_type(&self, _node_type: NodeType) -> CodecSupport {
        CodecSupport::HighLoss
    }

    fn supports_from_string(&self) -> bool {
        false
    }

    fn supports_to_string(&self) -> bool {
        false
    }

    async fn to_path(
        &self,
        node: &Node,
        path: &Path,
        options: Option<EncodeOptions>,
    ) -> Result<EncodeInfo> {
        let options = options.unwrap_or_default();
        let tool = options.tool.clone().unwrap_or_default();

        let info = if tool == "latex" || tool.is_empty() {
            let (latex, info) = to_latex(node, Format::Png, false, true, false, false);
            latex_to_image(&latex, path, None)?;
            info
        } else {
            bail!("Tool `{tool}` is not supported for encoding to PNG")
        };

        Ok(info)
    }
}
