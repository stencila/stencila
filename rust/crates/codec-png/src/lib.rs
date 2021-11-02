//! A codec for PNG images

use chromiumoxide::{cdp::browser_protocol::page::CaptureScreenshotFormat, Browser, BrowserConfig};
use codec_html::HtmlCodec;
use codec_trait::{
    async_trait::async_trait, eyre::Result, stencila_schema::Node, Codec, EncodeOptions,
};
use futures::StreamExt;
use std::{fs, path::Path};

pub struct PngCodec {}
#[async_trait]
impl Codec for PngCodec {
    /// Encode a `Node` to a Base64 encoded PNG image
    async fn to_string_async(node: &Node, options: Option<EncodeOptions>) -> Result<String> {
        let bytes = nodes_to_pngs(&[node], options).await?;
        let string = ["data:image/png;base64,", &base64::encode(&bytes[0])].concat();
        Ok(string)
    }

    /// Encode a `Node` to a PNG file
    async fn to_path<T: AsRef<Path>>(
        node: &Node,
        path: &T,
        options: Option<EncodeOptions>,
    ) -> Result<()>
    where
        T: Send + Sync,
    {
        let bytes = nodes_to_pngs(&[node], options).await?;
        fs::write(path, &bytes[0])?;
        Ok(())
    }
}

/// Encode a list of `Node`s to PNGs (as bytes)
///
/// This function is based around creating a list of PNGs, rather than a single one, to
/// reduce the per-image overhead of starting the browser, loading the theme etc.
pub async fn nodes_to_pngs(
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
        html.push_str(&format!(r#"<div id="node-{}">{}</div>"#, index, node_html));
    }

    // Wrap the HTML with a header etc so that the theme is set and CSS is loaded
    let EncodeOptions { theme, .. } = options.unwrap_or_default();
    let html = codec_html::wrap_standalone("PNG", &theme, &html);

    // Launch the browser
    let chrome = binaries::require("chrome", "*").await?;
    let config = BrowserConfig::builder()
        .chrome_executable(chrome.path)
        .build()
        .expect("Should build config");
    let (browser, mut handler) = Browser::launch(config).await?;
    tokio::task::spawn(async move {
        loop {
            let _ = handler.next().await.unwrap();
        }
    });

    // Create a page and set its HTML
    let page = browser.new_page("about:blank").await?;
    page.set_content(html).await?.wait_for_navigation().await?;

    // Take a screenshot of each element
    let mut pngs = Vec::with_capacity(nodes.len());
    for index in 0..nodes.len() {
        let element = page.find_element(&format!("#node-{}", index)).await?;
        let bytes = element.screenshot(CaptureScreenshotFormat::Png).await?;
        pngs.push(bytes)
    }

    Ok(pngs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use codec_trait::stencila_schema::CodeChunk;
    use path_slash::PathExt;

    #[cfg(target_os = "linux")]
    #[tokio::test]
    async fn test_encode() -> super::Result<()> {
        let node = Node::CodeChunk(CodeChunk {
            programming_language: "python".to_string(),
            text: "print(\"Hello world!\")".to_string(),
            outputs: Some(vec![Node::String("Hello world!".to_string())]),
            ..Default::default()
        });

        let dir = tempfile::tempdir()?;
        let path = dir.path().join("temp.png");
        PngCodec::to_path(
            &node,
            &path.to_slash_lossy(),
            Some(EncodeOptions {
                theme: "rpng".to_string(),
                ..Default::default()
            }),
        )
        .await?;
        assert!(path.exists());

        let data = PngCodec::to_string_async(&node, None).await?;
        assert!(data.starts_with("data:image/png;base64,"));

        Ok(())
    }
}
