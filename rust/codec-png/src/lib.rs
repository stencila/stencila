use std::{fmt::Write, fs, path::Path};

use chromiumoxide::{
    cdp::browser_protocol::page::CaptureScreenshotFormat, handler::viewport::Viewport, Browser,
    BrowserConfig,
};

use codec::{
    common::{async_trait::async_trait, base64, eyre::Result, futures::StreamExt, tokio},
    stencila_schema::{ImageObject, Node},
    utils::vec_string,
    Codec, CodecTrait, DecodeOptions, EncodeOptions,
};
use codec_html::HtmlCodec;

/// Encode and decode a document node to a PNG image.
///
/// This codec will "decode" strings or files to an `ImageObject` and uses a
/// headless browser to encode nodes as a screenshot of the HTML
/// encoding of the node.
pub struct PngCodec {}

#[async_trait]
impl CodecTrait for PngCodec {
    fn spec() -> Codec {
        Codec {
            status: "alpha".to_string(),
            formats: vec_string!["png"],
            root_types: vec_string!["*"],
            ..Default::default()
        }
    }

    /// Decode a document node from a string
    ///
    /// Simply returns an `ImageObject` with the string as its `content_url`
    /// (i.e assumes that the string is a DataURI with a Base64 encoded PNG)
    fn from_str(str: &str, _options: Option<DecodeOptions>) -> Result<Node> {
        Ok(Node::ImageObject(ImageObject {
            content_url: str.to_string(),
            ..Default::default()
        }))
    }

    /// Decode a document node from a file system path
    ///
    /// Simply returns an `ImageObject` with the path as its `content_url`
    async fn from_path(path: &Path, _options: Option<DecodeOptions>) -> Result<Node> {
        Ok(Node::ImageObject(ImageObject {
            content_url: path.display().to_string(),
            ..Default::default()
        }))
    }

    /// Encode a document node to a string
    ///
    /// Returns a Base64 encoded dataURI with media type `image/png`.
    async fn to_string_async(node: &Node, options: Option<EncodeOptions>) -> Result<String> {
        let bytes = nodes_to_bytes(&[node], options).await?;
        let string = ["data:image/png;base64,", &base64::encode(&bytes[0])].concat();
        Ok(string)
    }

    /// Encode a document node to a file system path
    ///
    /// This override is necessary to avoid the dataURI prefix and Base64 encoding that `to_string_async`
    /// does. It simply writes that bytes to a file at the path.
    async fn to_path(node: &Node, path: &Path, options: Option<EncodeOptions>) -> Result<()> {
        let bytes = nodes_to_bytes(&[node], options).await?;
        fs::write(path, &bytes[0])?;
        Ok(())
    }
}

/// Encode a set of document nodes to PNGs as bytes
///
/// This function is based around creating multiple PNGs, rather than a single one, to
/// reduce the per-image overhead of starting the browser, loading the theme etc.
pub async fn nodes_to_bytes(
    nodes: &[&Node],
    options: Option<EncodeOptions>,
) -> Result<Vec<Vec<u8>>> {
    // Return early if possible to avoid the following, including requiring Chrome.
    if nodes.is_empty() {
        return Ok(Vec::new());
    }

    // Generate HTML for each node
    let mut nodes_html = String::new();
    for (index, node) in nodes.iter().enumerate() {
        let node_html = HtmlCodec::to_string(
            node,
            Some(EncodeOptions {
                standalone: false,
                bundle: true,
                ..Default::default()
            }),
        )?;
        write!(
            nodes_html,
            r#"<div class="node" id="node-{}">{}</div>"#,
            index, node_html
        )?;
    }

    let theme = options
        .unwrap_or_default()
        .theme
        .unwrap_or_else(|| "rpng".to_string());

    // Wrap the generated HTML into a standalone page with bundled CSS & JS
    let html = codec_html::wrap_standalone(
        &nodes_html,
        EncodeOptions {
            bundle: true,
            theme: Some(theme),
            components: false,
            ..Default::default()
        },
        "PngCodec",
        r"
        body {
            width: 640px; /* Avoid having images of block nodes that are too wide */
        }
        div.node {
            margin: 10px; /* Mainly to improve spacing when previewing HTML during development */
            min-height: 1px; /* To avoid 'Error -32000: Cannot take screenshot with 0 height' */
            min-width: 1px;
        }",
        "",
    );

    // It can be useful to insect the generated HTML during development. Uncomment this to do so..
    // std::fs::write("rpng.html", &html)?;

    // Launch the browser
    let chrome = binaries::require_any(&[("chrome", "*"), ("chromium", "*")]).await?;
    let config = BrowserConfig::builder()
        .chrome_executable(chrome.path)
        .viewport(Viewport {
            // Increase the scale for higher resolution images. See https://github.com/puppeteer/puppeteer/issues/571#issuecomment-325404760
            device_scale_factor: Some(2.0),
            // Increase size of viewport to be able to encompass the largest images
            height: 10000,
            width: 10000,
            ..Default::default()
        })
        .build()
        .expect("Should build config");
    let (browser, mut handler) = Browser::launch(config).await?;
    let handler_task = tokio::task::spawn(async move {
        loop {
            let _ = handler.next().await.unwrap();
        }
    });

    // Create a page, set its HTML and wait for "navigation"
    let page = browser.new_page("about:blank").await?;
    page.set_content(html).await?.wait_for_navigation().await?;

    // Take a screenshot of each element
    // This uses `:first-child`, rather than screen-shotting the entire div, so that for
    // inline elements we do not get wide (page width) images. Assumes that the node is represented
    // by one element.
    let mut pngs = Vec::with_capacity(nodes.len());
    for index in 0..nodes.len() {
        let element = page
            .find_element(&format!("#node-{} :first-child", index))
            .await?;
        let bytes = element.screenshot(CaptureScreenshotFormat::Png).await?;
        pngs.push(bytes)
    }

    // Abort the handler task (if this is not done can get a `ResetWithoutClosingHandshake`
    // when this function ends)
    handler_task.abort();

    Ok(pngs)
}

#[cfg(test)]
mod tests {
    use codec::stencila_schema::CodeChunk;
    use test_utils::common::tempfile;

    use super::*;

    #[tokio::test]
    async fn encode() -> super::Result<()> {
        let node = Node::CodeChunk(CodeChunk {
            programming_language: "python".to_string(),
            code: "print(\"Hello world!\")".to_string(),
            outputs: Some(vec![Node::String("Hello world!".to_string())]),
            ..Default::default()
        });

        let dir = tempfile::tempdir()?;
        let path = dir.path().join("temp.png");
        PngCodec::to_path(&node, &path, None).await?;
        assert!(path.exists());

        let data = PngCodec::to_string_async(&node, None).await?;
        assert!(data.starts_with("data:image/png;base64,"));

        Ok(())
    }
}
