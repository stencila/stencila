use std::{fs, path::Path};

use chromiumoxide::{
    cdp::browser_protocol::page::CaptureScreenshotFormat, handler::viewport::Viewport, Browser,
    BrowserConfig,
};
use futures::StreamExt;
use tokio::time::{sleep, Duration, Instant};

use codec::{
    async_trait::async_trait,
    eyre::Result,
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
    let mut html = String::new();
    for (index, node) in nodes.iter().enumerate() {
        let node_html = HtmlCodec::to_string(
            node,
            Some(EncodeOptions {
                standalone: false,
                bundle: true,
                ..Default::default()
            }),
        )?;
        html.push_str(&format!(
            r#"<div class="node" id="node-{}">{}</div>"#,
            index, node_html
        ));
    }

    let EncodeOptions { theme, .. } = options.unwrap_or_default();
    let theme = theme.unwrap_or_else(|| "rpng".to_string());

    // Wrap the HTML with a header etc so that the theme is set and CSS is loaded
    let base_url = "http://127.0.0.1:9000/~static/dev"; // TODO: start served on demand as get current url
    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
    <head>
        <title>PNG Codec</title>
        <meta charset="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <link href="{base_url}/themes/themes/{theme}/styles.css" rel="stylesheet">
        <script src="{base_url}/components/stencila-components.esm.js" type="module"></script>
        <style>
            body {{
                width: 640px; /* Avoid having images of block node that are too wide */
            }}
            div.node {{
                margin: 10px; /* Mainly to improve spacing when previewing HTML during development */
            }}
        </style>
    </head>
    <body data-root="">
        {html}
    </body>
</html>"#,
        theme = theme,
        html = html
    );

    // This can be useful during debugging to preview the HTML
    use std::io::Write;
    std::fs::File::create("temp-png.html")
        .expect("Unable to create file")
        .write_all(html.as_bytes())
        .expect("Unable to write data");

    // Launch the browser
    let chrome = binaries::require_any(&[("chrome", "*"), ("chromium", "*")]).await?;
    let config = BrowserConfig::builder()
        .chrome_executable(chrome.path)
        .viewport(Viewport {
            // Increase the scale for higher resolution images. See https://github.com/puppeteer/puppeteer/issues/571#issuecomment-325404760
            device_scale_factor: Some(2.0),
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

    // Wait up to a further 5s for Web Components on the page to be hydrated
    // An alternative to this would be to listen for the StencilJS `appload`
    // event https://stenciljs.com/docs/api#the-appload-event
    let deadline = Instant::now() + Duration::from_secs(5);
    while page.find_element("html.hydrated").await.is_err() && Instant::now() < deadline {
        sleep(Duration::from_millis(5)).await;
    }

    // Take a screenshot of each element
    // This uses `:first-child`, rather than screen-shotting the entire div, so that for
    // inline elements we do not get wide (page width) images. Assumes that the node is represented
    // by one element.
    let mut pngs = Vec::with_capacity(nodes.len());
    for index in 0..nodes.len() {
        let element = page
            .find_element(&format!("#node-{} *:first-child", index))
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
    use super::*;
    use codec::stencila_schema::CodeChunk;
    use test_utils::tempfile;

    #[tokio::test]
    async fn encode() -> super::Result<()> {
        let node = Node::CodeChunk(CodeChunk {
            programming_language: "python".to_string(),
            text: "print(\"Hello world!\")".to_string(),
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
